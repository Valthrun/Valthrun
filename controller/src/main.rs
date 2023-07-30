#![feature(iterator_try_collect)]
#![allow(dead_code)]

use std::{fmt::Debug, collections::{BTreeMap, btree_map::Entry}, cell::RefCell};
use anyhow::Context;
use cs2_schema::offsets;
use glium::glutin::{window::Window, platform::windows::WindowExtWindows};
use imgui::{Condition, ImColor32};
use obfstr::obfstr;
use valthrun_kinterface::ByteSequencePattern;
use windows::{Win32::{UI::WindowsAndMessaging::{FindWindowA, MoveWindow, GetClientRect}, Foundation::{RECT, HWND, POINT}, Graphics::Gdi::ClientToScreen}, core::PCSTR};

use crate::handle::{CS2Handle, Module};

mod handle;
mod schema;
mod overlay;

#[repr(C)]
#[derive(Default, Clone)]
pub struct EntityHandle {
    pub value: u32,
}

impl Debug for EntityHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntityHandle")
            .field("value", &self.value)
            .field("entity_index", &format_args!("0x{:X}", &self.get_entity_index()))
            .field("serial_number", &format_args!("0x{:X}", &self.get_serial_number()))
            .finish()
    }
}

impl EntityHandle {
    pub fn get_entity_index(&self) -> u32 {
        (self.value & 0x7FFF) as u32
    }

    pub fn is_valid(&self) -> bool {
        self.get_entity_index() < 0x7FF0
    }

    pub fn get_serial_number(&self) -> u32 {
        (self.value >> 15) as u32
    }

    pub fn entity_array_offsets(&self) -> (u64, u64) {
        let entity_index = self.get_entity_index();
        ((entity_index >> 9) as u64, (entity_index & 0x1FF) as u64)
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone)]
struct EntityIdentity {
    entity_ptr: u64,
    ptr_2: u64,

    handle: EntityHandle,
    name_stringable_index: u32,
    name: u64,

    designer_name: u64,
    pad_0: u64,

    flags: u64,
    world_group_id: u32,
    data_object_types: u32,

    path_index: u64,
    pad_1: u64,
    
    pad_2: u64,
    p_prev: u64,

    p_next: u64,
    p_prev_by_class: u64,
    
    p_next_by_class: u64,
}
const _: [u8; 120] = [0; std::mem::size_of::<EntityIdentity>()];

impl EntityIdentity {
    fn collect_all_of_class(&self, cs2: &CS2Handle) -> anyhow::Result<Vec<EntityIdentity>> {
        let mut result = Vec::new();
        result.reserve(128);
        result.push(self.clone());

        let mut prev_entity = self.p_prev_by_class;
        while prev_entity > 0 {
            let entity = cs2.read::<EntityIdentity>(Module::Absolute, &[ prev_entity ])
                .context("failed to read prev entity identity of class")?;
            prev_entity = entity.p_prev_by_class;
            result.push(entity);
        }

        let mut next_entity = self.p_next_by_class;
        while next_entity > 0 {
            let entity = cs2.read::<EntityIdentity>(Module::Absolute, &[ next_entity ])
                .context("failed to read next entity identity of class")?;
            next_entity = entity.p_next_by_class;
            result.push(entity);
        }
        
        Ok(result)
    }
}

struct CS2Offsets {
    /// Client offset for the local player controller ptr
    local_controller: u64,

    /// Client offset for the global entity list ptr
    global_entity_list: u64,
}

impl CS2Offsets {
    pub fn load_offsets(cs2: &CS2Handle) -> anyhow::Result<Self> {
        let local_controller = Self::find_local_player_controller_ptr(cs2)?;
        let global_entity_list = Self::find_entity_list(cs2)?;

        Ok(Self {
            local_controller,
            global_entity_list
        })
    }

    fn find_local_player_controller_ptr(cs2: &CS2Handle) -> anyhow::Result<u64> {
        // 48 83 3D ? ? ? ? ? 0F 95 -> IsLocalPlayerControllerValid
        let pattern = ByteSequencePattern::parse("48 83 3D ? ? ? ? ? 0F 95").unwrap();
        let inst_address = cs2.find_pattern(Module::Client, &pattern)?
            .context("failed to find local player controller ptr")?;
    
        let address = inst_address + cs2.read::<i32>(Module::Client, &[ inst_address + 0x03 ])? as u64 + 0x08;
        log::debug!("Local player controller ptr at {:X}", address);
        Ok(address)
    }
    
    fn find_entity_list(cs2: &CS2Handle) -> anyhow::Result<u64> {
        // 4C 8B 0D ? ? ? ? 48 89 5C 24 ? 8B -> Global entity list
        let pattern_entity_list = ByteSequencePattern::parse("4C 8B 0D ? ? ? ? 48 89 5C 24 ? 8B").unwrap();
        let inst_address = cs2.find_pattern(Module::Client, &pattern_entity_list)
            .context("missing entity list")?
            .context("failed to find global entity list pattern")?;
        let entity_list_address = inst_address + cs2.read::<i32>(Module::Client, &[ inst_address + 0x03 ])? as u64 + 0x07;
        log::debug!("Entity list at {:X}", entity_list_address);
        Ok(entity_list_address)
    }
}

struct PlayerInfo {
    local: bool,
    player_health: i32,
    player_name: String,
    position: nalgebra::Vector3<f32>,

    debug_text: String,
    bones: Vec<PlayerBone>,
    model: Option<PlayerModel>
}

#[repr(C)]
#[derive(Debug, Default, Clone)]
struct PlayerModel {
    vhull_min: nalgebra::Vector3<f32>,
    vhull_max: nalgebra::Vector3<f32>,
    
    vview_min: nalgebra::Vector3<f32>,
    vview_max: nalgebra::Vector3<f32>,
}

#[derive(Debug, Clone)]
struct PlayerBone {
    name: String,
    flags: u32,
    parent: Option<usize>,
    position: nalgebra::Vector3<f32>,
}
pub struct ViewController {
    cs2_view_matrix_offset: u64,
    view_matrix: nalgebra::Matrix4<f32>,
    screen: mint::Vector2<f32>,
}

impl ViewController {
    pub fn new(cs2_view_matrix_offset: u64) -> Self {
        Self {
            cs2_view_matrix_offset,
            view_matrix: Default::default(),
            screen: mint::Vector2 { x: 0.0, y: 0.0 }
        }
    }

    pub fn update(&mut self, screen: mint::Vector2<f32>, cs2: &CS2Handle) -> anyhow::Result<()> {
        self.screen = screen;
        self.view_matrix = cs2.read(Module::Client, &[ self.cs2_view_matrix_offset ])?;
        Ok(())
    }

    /// Returning an mint::Vector2<f32> as the result should be used via ImGui.
    pub fn world_to_screen(&self, vec: &nalgebra::Vector3<f32>, allow_of_screen: bool) -> Option<mint::Vector2<f32>> {
        let screen_coords = 
            nalgebra::Vector4::new(vec.x, vec.y, vec.z, 1.0).transpose() * self.view_matrix;

        if screen_coords.w < 0.1 {
            return None;
        }

        if !allow_of_screen && (
            screen_coords.x < -screen_coords.w || screen_coords.x > screen_coords.w || 
            screen_coords.y < -screen_coords.w || screen_coords.y > screen_coords.w
        ) {
            return None;
        }

        let mut screen_pos = mint::Vector2::from_slice(&[
            screen_coords.x / screen_coords.w, 
            screen_coords.y / screen_coords.w
        ]);
        screen_pos.x = (screen_pos.x + 1.0) * self.screen.x / 2.0;
        screen_pos.y = (-screen_pos.y + 1.0) * self.screen.y / 2.0;
        return Some(screen_pos);
    }
}

/// Track the CS2 window and adjust overlay accordingly.
/// This is only required when playing in windowed mode.
pub struct CSWindowTracker {
    cs2_hwnd: HWND,
    current_bounds: RECT
}

impl CSWindowTracker {
    pub fn new() -> anyhow::Result<Self> {
        let cs2_hwnd = unsafe {
            FindWindowA(
                PCSTR::null(), 
                PCSTR::from_raw("Counter-Strike 2\0".as_ptr())
            )
        };
        if cs2_hwnd.0 == 0 {
            anyhow::bail!("failed to locate CS2 window");
        }

        Ok(Self {
            cs2_hwnd,
            current_bounds: Default::default()
        })
    }

    pub fn update_overlay(&mut self, overlay: &Window) {
        let mut rect: RECT = Default::default();
        let success = unsafe { GetClientRect(self.cs2_hwnd, &mut rect) };
        if !success.as_bool() {
            return;
        }

        unsafe {
            ClientToScreen(self.cs2_hwnd, &mut rect.left as *mut _ as *mut POINT);
            ClientToScreen(self.cs2_hwnd, &mut rect.right as *mut _ as *mut POINT);
        }

        if rect == self.current_bounds {
            return;
        }

        self.current_bounds = rect;
        log::debug!("CS2 window changed: {:?}", rect);
        unsafe {
            let overlay_hwnd = HWND(overlay.hwnd() as isize);
            MoveWindow(overlay_hwnd, rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top, true);
        }
    }
}

mod internal_offsets {
    // Sig source: https://www.unknowncheats.me/forum/3725362-post1.html
    // https://www.unknowncheats.me/forum/3713485-post262.html
    #[allow(non_snake_case)]
    pub mod CModel {
        /* 85 D2 78 16 3B 91. Offset is array of u32 */
        pub const BONE_FLAGS: u64 = 0x1A8;
        
        /* 85 D2 78 25 3B 91. Offset is array of *const i8 */
        pub const BONE_NAME: u64 = 0x160;

        /* UC sig does not work. Offset is array of u16 */
        pub const BONE_PARENT: u64 = 0x178;
    }

    #[allow(non_snake_case)]
    pub mod CModelState {
        /* Offset is array of BoneData */
        pub const BONE_STATE_DATA: u64 = 0x80;
    }
}
struct Application {
    cs2: CS2Handle,
    cs2_offsets: CS2Offsets,
    cs2_entities: EntitySystem,

    settings_visible: bool,
    window_tracker: Option<CSWindowTracker>,

    model_cache: BTreeMap<u64, CachedModel>,

    players: Vec<PlayerInfo>,
    view_controller: ViewController,

    settings: RefCell<AppSettings>,
}

#[derive(Debug, Clone, Copy)]
pub struct AppSettings {
    pub player_pos_dot: bool,
    pub esp_skeleton: bool,
    pub esp_boxes: bool,
}

struct CachedModel {
    address: u64,
    bones: Vec<PlayerBone>,
    player_model: PlayerModel
}

impl CachedModel {
    pub fn create(cs2: &CS2Handle, address: u64) -> anyhow::Result<Self> {
        let mut result = Self {
            address,
            bones: Default::default(),
            player_model: Default::default(),
        };
        result.reload_cache(cs2)?;
        Ok(result)
    }

    pub fn reload_cache(&mut self, cs2: &CS2Handle) -> anyhow::Result<()> {
        self.player_model = cs2.read::<PlayerModel>(Module::Absolute, &[
            self.address + 0x18,
        ])?;

        let bone_count = cs2.read::<u64>(Module::Absolute, &[
            self.address + internal_offsets::CModel::BONE_NAME - 0x08
        ])? as usize;
        if bone_count > 1000 {
            anyhow::bail!("model contains too many bones ({bone_count})");
        }

        log::trace!("Reading {} bones", bone_count);
        let model_bone_flags = cs2.read_vec::<u32>(Module::Absolute, &[
            self.address + internal_offsets::CModel::BONE_FLAGS,
            0, /* read the whole array */
        ], bone_count as usize)?;

        let model_bone_parent_index = cs2.read_vec::<u16>(Module::Absolute, &[
            self.address + internal_offsets::CModel::BONE_PARENT,
            0, /* read the whole array */
        ], bone_count as usize)?;

        self.bones.clear();
        self.bones.reserve(bone_count as usize);
        for bone_index in 0..bone_count {
            let name = cs2.read_string(Module::Absolute, &[
                self.address + internal_offsets::CModel::BONE_NAME,
                0x08 * bone_index as u64,
                0
            ], None)?;

            let parent_index = model_bone_parent_index[bone_index];
            let flags = model_bone_flags[bone_index];

            self.bones.push(PlayerBone { 
                name: name.clone(),
                parent: if parent_index as usize >= bone_count { None } else { Some(parent_index as usize) },
                
                position: Default::default(),
                flags
            }); 
        }
        Ok(())
    }
}

enum BoneFlags {
    FlagNoBoneFlags = 0x0,
	FlagBoneflexdriver = 0x4,
	FlagCloth = 0x8,
	FlagPhysics = 0x10,
	FlagAttachment = 0x20,
	FlagAnimation = 0x40,
	FlagMesh = 0x80,
	FlagHitbox = 0x100,
	FlagBoneUsedByVertexLod0 = 0x400,
	FlagBoneUsedByVertexLod1 = 0x800,
	FlagBoneUsedByVertexLod2 = 0x1000,
	FlagBoneUsedByVertexLod3 = 0x2000,
	FlagBoneUsedByVertexLod4 = 0x4000,
	FlagBoneUsedByVertexLod5 = 0x8000,
	FlagBoneUsedByVertexLod6 = 0x10000,
	FlagBoneUsedByVertexLod7 = 0x20000,
	FlagBoneMergeRead = 0x40000,
	FlagBoneMergeWrite = 0x80000,
	FlagAllBoneFlags = 0xfffff,
	BlendPrealigned = 0x100000,
	FlagRigidlength = 0x200000,
	FlagProcedural = 0x400000,
}

struct EntitySystem {
    local_controller_ptr: u64,
    global_list_address: u64
}

impl EntitySystem {
    /* Returns a CSSPlayerController instance */
    pub fn get_local_player_controller(&self, cs2: &CS2Handle) -> anyhow::Result<Option<u64>> {
        let entity = cs2.read::<u64>(Module::Client, &[ 
            self.local_controller_ptr,
        ]).context("failed to read local player controller")?;

        if entity > 0 {
            Ok(Some(entity))
        } else {
            Ok(None)
        }
    }

    pub fn get_by_handle(&self, cs2: &CS2Handle, handle: &EntityHandle) -> anyhow::Result<Option<u64>> {
        let (bulk, offset) = handle.entity_array_offsets();
        let identity = cs2.read::<EntityIdentity>(Module::Client, &[
            self.global_list_address,
            bulk * 0x08,
            offset * 120,
        ]);
        
        let identity = match identity {
            Ok(identity) => identity,
            Err(error) => return Err(error.context(format!("failed to read global entity list entry for handle {:?}", handle)))
        };

        if identity.handle.get_entity_index() == handle.get_entity_index() {
            Ok(Some(identity.entity_ptr))
        } else {
            Ok(None)
        }
    }

    /* Returns a Vec<CSSPlayerController*> */
    pub fn get_player_controllers(&self, cs2: &CS2Handle) -> anyhow::Result<Vec<u64>> {
        let local_controller_identity = cs2.read::<EntityIdentity>(Module::Client, &[ 
            self.local_controller_ptr,
            offsets::client::CEntityInstance::m_pEntity, /* read the entity identnity index  */
            0, /* read everything */
        ]).context("failed to read local player controller identity")?;

        Ok(
            local_controller_identity.collect_all_of_class(cs2)?
                .into_iter()
                .map(|identity| identity.entity_ptr)
                .collect()
        )
    }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct BoneStateData {
    pub position: nalgebra::Vector3<f32>,
    pub scale: f32,
    pub rotation: nalgebra::Vector4<f32>
}
const _: [u8; 0x20] = [0; std::mem::size_of::<BoneStateData>()];

impl Application {
    pub fn update(&mut self, window: &Window, ui: &imgui::Ui) -> anyhow::Result<()> {
        if ui.is_key_pressed_no_repeat(imgui::Key::Keypad0) {
            log::debug!("Toogle settings");
            self.settings_visible = !self.settings_visible;
        }
        
        if let Some(tracker) = self.window_tracker.as_mut() {
            tracker.update_overlay(window);
        }
        self.view_controller.update(mint::Vector2::from_slice(&ui.io().display_size), &self.cs2)?;

        self.players.clear();
        self.players.reserve(16);
        
        let local_player_controller = self.cs2_entities.get_local_player_controller(&self.cs2)?
            .context("missing local player controller")?;

        for player_controller in self.cs2_entities.get_player_controllers(&self.cs2)? {
            let player_pawn_handle = self.cs2.read::<EntityHandle>(Module::Absolute, &[
                player_controller + offsets::client::CCSPlayerController::m_hPlayerPawn
            ]).context("failed to read player pawn handle")?;

            if !player_pawn_handle.is_valid() {
                continue;
            }

            let player_health = self.cs2.read::<i32>(Module::Absolute, &[ 
                player_controller + offsets::client::CCSPlayerController::m_iPawnHealth
            ]).context("failed to read player controller pawn health")?;
            if player_health <= 0 {
                continue;
            }

            let player_pawn = self.cs2_entities.get_by_handle(&self.cs2, &player_pawn_handle)?
                .context("missing player pawn for player controller")?;

            /* Will be an instance of CSkeletonInstance */
            let game_sceen_node = self.cs2.read::<u64>(Module::Absolute, &[
                player_pawn + offsets::client::C_BaseEntity::m_pGameSceneNode
            ])?;

            let player_dormant = self.cs2.read::<bool>(Module::Absolute, &[
                game_sceen_node + offsets::client::CGameSceneNode::m_bDormant
            ])?;
            if player_dormant {
                continue;
            }

            let player_name = self.cs2.read_string(Module::Absolute, &[ 
                player_controller + offsets::client::CBasePlayerController::m_iszPlayerName 
            ], Some(128))?;

            let position = self.cs2.read::<nalgebra::Vector3<f32>>(Module::Absolute, &[
                game_sceen_node + offsets::client::CGameSceneNode::m_vecAbsOrigin
            ])?;
            
            let model = self.cs2.read::<u64>(Module::Absolute, &[
                game_sceen_node 
                    + offsets::client::CSkeletonInstance::m_modelState /* model state */
                    + offsets::client::CModelState::m_hModel, /* CModel* */
                0,
            ])?;

            let model = match self.model_cache.entry(model) {
                Entry::Occupied(value) => value.into_mut(),
                Entry::Vacant(value) => {
                    let model_name = self.cs2.read_string(Module::Absolute, &[ model + 0x08, 0 ], Some(32))?;
                    log::debug!("Discovered new player model {}. Caching.", model_name);

                    let model = CachedModel::create(&self.cs2, model)?;
                    value.insert(model)
                }
            };

            let bone_states = self.cs2.read_vec::<BoneStateData>(Module::Absolute, &[
                game_sceen_node 
                    + offsets::client::CSkeletonInstance::m_modelState /* model state */
                    + internal_offsets::CModelState::BONE_STATE_DATA,
                0, /* read the whole array */
            ], model.bones.len())?;

            let bones = model.bones.iter()
                .zip(bone_states.iter())
                .map(|(bone_info, bone_state)| {
                    PlayerBone {
                        position: bone_state.position,
                        ..(*bone_info).clone()
                    }
                })
                .collect::<Vec<_>>();

            self.players.push(PlayerInfo {
                local: player_controller == local_player_controller,
                player_name,
                player_health,
                position,

                debug_text: "".to_string(),

                bones,
                model: Some(model.player_model.clone())
            });
        }

        Ok(())
    }

    pub fn render(&self, ui: &imgui::Ui) {
        ui.window("overlay")
            .draw_background(false)
            .no_decoration()
            .no_inputs()
            .size(ui.io().display_size, Condition::Always)
            .position([ 0.0, 0.0 ], Condition::Always)
            .build(|| self.render_overlay(ui));

        if self.settings_visible {
            self.render_settings(ui);
        }
    }

    fn draw_box_3d(&self, draw: &imgui::DrawListMut, vmin: &nalgebra::Vector3<f32>, vmax: &nalgebra::Vector3<f32>, color: ImColor32) {
        type Vec3 = nalgebra::Vector3<f32>;

        let lines = [
            /* bottom */
            (Vec3::new(vmin.x, vmin.y, vmin.z), Vec3::new(vmax.x, vmin.y, vmin.z)),
            (Vec3::new(vmax.x, vmin.y, vmin.z), Vec3::new(vmax.x, vmin.y, vmax.z)),
            (Vec3::new(vmax.x, vmin.y, vmax.z), Vec3::new(vmin.x, vmin.y, vmax.z)),
            (Vec3::new(vmin.x, vmin.y, vmax.z), Vec3::new(vmin.x, vmin.y, vmin.z)),
            
            /* top */
            (Vec3::new(vmin.x, vmax.y, vmin.z), Vec3::new(vmax.x, vmax.y, vmin.z)),
            (Vec3::new(vmax.x, vmax.y, vmin.z), Vec3::new(vmax.x, vmax.y, vmax.z)),
            (Vec3::new(vmax.x, vmax.y, vmax.z), Vec3::new(vmin.x, vmax.y, vmax.z)),
            (Vec3::new(vmin.x, vmax.y, vmax.z), Vec3::new(vmin.x, vmax.y, vmin.z)),

            /* corners */
            (Vec3::new(vmin.x, vmin.y, vmin.z), Vec3::new(vmin.x, vmax.y, vmin.z)),
            (Vec3::new(vmax.x, vmin.y, vmin.z), Vec3::new(vmax.x, vmax.y, vmin.z)),
            (Vec3::new(vmax.x, vmin.y, vmax.z), Vec3::new(vmax.x, vmax.y, vmax.z)),
            (Vec3::new(vmin.x, vmin.y, vmax.z), Vec3::new(vmin.x, vmax.y, vmax.z)),
        ];

        for (start, end) in lines {
            if let (Some(start), Some(end)) = (
                self.view_controller.world_to_screen(&start, true),
                self.view_controller.world_to_screen(&end, true)
            ) {
                draw.add_line(start, end, color)
                    .build();
            }
        }
    }

    fn render_overlay(&self, ui: &imgui::Ui) {
        let settings = self.settings.borrow();

        {
            let text = "Valthrun Overlay";
            ui.set_cursor_pos([
                ui.window_size()[0] - ui.calc_text_size(&text)[0] - 10.0,
                10.0
            ]);
            ui.text(text);
        }
        {
            let text = format!("{:.2} FPS", ui.io().framerate);
            ui.set_cursor_pos([
                ui.window_size()[0] - ui.calc_text_size(&text)[0] - 10.0,
                24.0
            ]);
            ui.text(text)
        }
    
        ui.set_cursor_pos([ 10.0, 300.0 ]);
    
        ui.text(format!("{} players alive", self.players.len()));
        for entry in self.players.iter() {
            ui.text(format!("{} ({}) | {:?}", entry.player_name, entry.player_health, entry.position));
        }

        let draw = ui.get_window_draw_list();
        for entry in self.players.iter() {
            if entry.local {
                continue;
            }

            let position = entry.position;

            if settings.player_pos_dot {
                if let Some(mut screen_position) = self.view_controller.world_to_screen(&position, false) {
                    draw.add_circle(screen_position, 8.0, ImColor32::from_rgb(255, 0, 0))
                        .filled(true)
                        .build();
    
                    screen_position.y -= 10.0;
                    draw.add_text(screen_position, ImColor32::from_rgb(0, 255, 0), &entry.debug_text);
                }
            }
          

            if settings.esp_skeleton {
                for bone in entry.bones.iter() {
                    if (bone.flags & BoneFlags::FlagHitbox as u32) == 0 {
                        continue;
                    }
    
                    let parent_index = if let Some(parent) = bone.parent { parent } else { continue };
                    
                    let parent_position = match self.view_controller.world_to_screen(&entry.bones[parent_index].position, true) {
                        Some(position) => position,
                        None => continue,
                    };
                    let bone_position = match self.view_controller.world_to_screen(&bone.position, true) {
                        Some(position) => position,
                        None => continue,
                    };
                    
                    draw.add_line(parent_position, bone_position, ImColor32::from_rgb(0, 255, 255))
                        .build();
                }
            }

            if settings.esp_boxes {
                if let Some(model) = entry.model.as_ref() {
                    self.draw_box_3d(&draw, &(model.vhull_min + entry.position), &(model.vhull_max + entry.position), ImColor32::from_rgb(255, 0, 255));
                    //self.draw_box_3d(&draw, &(model.vview_min + entry.position), &(model.vview_max + entry.position), ImColor32::from_rgb(0, 0, 255));
                }
            }
        }
    } 

    fn render_settings(&self, ui: &imgui::Ui) {
        ui.window(obfstr!("Valthrun"))
            .size([ 600.0, 300.0 ], Condition::FirstUseEver)
            .build(|| {
                ui.text("Valthrun an open source CS2 external read only kernel cheat.");
                ui.separator();
                let mouse_pos = ui.io().mouse_pos;
                ui.text(format!(
                    "Mouse Position: ({:.1},{:.1})",
                    mouse_pos[0], mouse_pos[1]
                ));

                let mut settings = self.settings.borrow_mut();
                ui.checkbox("Player Position Dots", &mut settings.player_pos_dot);
                ui.checkbox("ESP Boxes", &mut settings.esp_boxes);
                ui.checkbox("ESP Skeletons", &mut settings.esp_skeleton);
            });
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cs2 = CS2Handle::create()?;
    let cs2_offsets = CS2Offsets::load_offsets(&cs2)?;
    
    let mut app = Application{
        cs2,
        cs2_entities: EntitySystem {
            global_list_address: cs2_offsets.global_entity_list,
            local_controller_ptr: cs2_offsets.local_controller,
        },
        cs2_offsets,

        settings_visible: false,
        window_tracker: Some(CSWindowTracker::new()?),

        players: Vec::with_capacity(16),
        model_cache: Default::default(),

        // 0x16D1D90 - 48 8D 0D ? ? ? ? 48 C1 E0 06
        view_controller: ViewController::new(0x16D1D90),

        settings: RefCell::new(AppSettings{
            esp_boxes: true,
            esp_skeleton: true,
            player_pos_dot: true
        })
    };
    overlay::init("Test")
        .main_loop(move |run, window, ui| {
            if let Err(err) = app.update(window, ui) {
                log::error!("{:#}", err);
                *run = false;
                return;
            };

            app.render(ui);
        });

    Ok(())
}
