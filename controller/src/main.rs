#![feature(iterator_try_collect)]
#![allow(dead_code)]

use anyhow::Context;
use cs2::{CS2Handle, Module, EntityHandle, CS2Offsets, EntitySystem, offsets_manual, CS2Model, BoneFlags};
use cs2_schema::offsets;
use imgui::{Condition, ImColor32};
use obfstr::obfstr;
use view::ViewController;
use std::{
    cell::RefCell,
    collections::{btree_map::Entry, BTreeMap},
    fmt::Debug, sync::Arc, time::Instant,
};

mod view;

struct PlayerInfo {
    local: bool,
    player_health: i32,
    player_name: String,
    position: nalgebra::Vector3<f32>,

    debug_text: String,
    
    model: Arc<CS2Model>,
    bone_states: Vec<BoneStateData>,
}


#[derive(Debug, Clone, Copy)]
pub struct AppSettings {
    pub player_pos_dot: bool,
    pub esp_skeleton: bool,
    pub esp_boxes: bool,
}

struct CachedModel {
    model: Arc<CS2Model>,
    last_use: Instant,
    flag_used: bool,
}

impl CachedModel {
    pub fn create(model: Arc<CS2Model>) -> Self {
        Self {
            model,
            last_use: Instant::now(),
            flag_used: false,
        }
    }

    pub fn flag_use(&mut self) {
        self.flag_used = true;
    }

    /// Commits the used flag.
    /// Returns the seconds since last use.
    pub fn commit_use(&mut self) -> u64 {
        if self.flag_used {
            self.flag_used = false;
            self.last_use = Instant::now();
            0
        } else {
            self.last_use.elapsed().as_secs()
        }
    }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct BoneStateData {
    pub position: nalgebra::Vector3<f32>,
    pub scale: f32,
    pub rotation: nalgebra::Vector4<f32>,
}
const _: [u8; 0x20] = [0; std::mem::size_of::<BoneStateData>()];

struct Application {
    cs2: Arc<CS2Handle>,
    cs2_offsets: Arc<CS2Offsets>,
    cs2_entities: EntitySystem,

    settings_visible: bool,
    model_cache: BTreeMap<u64, CachedModel>,

    players: Vec<PlayerInfo>,
    view_controller: ViewController,

    settings: RefCell<AppSettings>,
}

impl Application {
    pub fn update(&mut self, ui: &imgui::Ui) -> anyhow::Result<()> {
        if ui.is_key_pressed_no_repeat(imgui::Key::Keypad0) {
            log::debug!("Toogle settings");
            self.settings_visible = !self.settings_visible;
        }

        self.view_controller.update_screen_bounds(mint::Vector2::from_slice(&ui.io().display_size));
        self.view_controller
            .update_view_matrix(&self.cs2)?;

        self.players.clear();
        self.players.reserve(16);

        let local_player_controller = self
            .cs2_entities
            .get_local_player_controller(&self.cs2)?
            .with_context(|| obfstr!("missing local player controller").to_string())?;

        for player_controller in self.cs2_entities.get_player_controllers(&self.cs2)? {
            let player_pawn_handle = self
                .cs2
                .read::<EntityHandle>(
                    Module::Absolute,
                    &[player_controller + offsets::client::CCSPlayerController::m_hPlayerPawn],
                )
                .with_context(|| obfstr!("failed to read player pawn handle").to_string())?;

            if !player_pawn_handle.is_valid() {
                continue;
            }

            let player_health = self
                .cs2
                .read::<i32>(
                    Module::Absolute,
                    &[player_controller + offsets::client::CCSPlayerController::m_iPawnHealth],
                )
                .with_context(|| obfstr!("failed to read player controller pawn health").to_string())?;
            if player_health <= 0 {
                continue;
            }

            let player_pawn = self
                .cs2_entities
                .get_by_handle(&self.cs2, &player_pawn_handle)?
                .with_context(|| obfstr!("missing player pawn for player controller").to_string())?;

            /* Will be an instance of CSkeletonInstance */
            let game_sceen_node = self.cs2.read::<u64>(
                Module::Absolute,
                &[player_pawn + offsets::client::C_BaseEntity::m_pGameSceneNode],
            )?;

            let player_dormant = self.cs2.read::<bool>(
                Module::Absolute,
                &[game_sceen_node + offsets::client::CGameSceneNode::m_bDormant],
            )?;
            if player_dormant {
                continue;
            }

            let player_name = self.cs2.read_string(
                Module::Absolute,
                &[player_controller + offsets::client::CBasePlayerController::m_iszPlayerName],
                Some(128),
            )?;

            let position = self.cs2.read::<nalgebra::Vector3<f32>>(
                Module::Absolute,
                &[game_sceen_node + offsets::client::CGameSceneNode::m_vecAbsOrigin],
            )?;

            let model = self.cs2.read::<u64>(
                Module::Absolute,
                &[
                    game_sceen_node
                    + offsets::client::CSkeletonInstance::m_modelState /* model state */
                    + offsets::client::CModelState::m_hModel, /* CModel* */
                    0,
                ],
            )?;

            let model = match self.model_cache.entry(model) {
                Entry::Occupied(value) => value.into_mut(),
                Entry::Vacant(value) => {
                    let model_name =
                        self.cs2
                            .read_string(Module::Absolute, &[model + 0x08, 0], Some(32))?;
                    log::debug!("{} {}. Caching.", obfstr!("Discovered new player model"), model_name);

                    let model = CS2Model::read(&self.cs2, model)?;
                    value.insert(CachedModel::create(Arc::new(model)))
                }
            };
            model.flag_use();

            let bone_states = self.cs2.read_vec::<BoneStateData>(
                Module::Absolute,
                &[
                    game_sceen_node
                    + offsets::client::CSkeletonInstance::m_modelState /* model state */
                    + offsets_manual::client::CModelState::BONE_STATE_DATA,
                    0, /* read the whole array */
                ],
                model.model.bones.len(),
            )?;

            self.players.push(PlayerInfo {
                local: player_controller == local_player_controller,
                player_name,
                player_health,
                position,

                debug_text: "".to_string(),

                bone_states,
                model: model.model.clone(),
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
            .position([0.0, 0.0], Condition::Always)
            .build(|| self.render_overlay(ui));

        if self.settings_visible {
            self.render_settings(ui);
        }
    }

    fn draw_box_3d(
        &self,
        draw: &imgui::DrawListMut,
        vmin: &nalgebra::Vector3<f32>,
        vmax: &nalgebra::Vector3<f32>,
        color: ImColor32,
    ) {
        type Vec3 = nalgebra::Vector3<f32>;

        let lines = [
            /* bottom */
            (
                Vec3::new(vmin.x, vmin.y, vmin.z),
                Vec3::new(vmax.x, vmin.y, vmin.z),
            ),
            (
                Vec3::new(vmax.x, vmin.y, vmin.z),
                Vec3::new(vmax.x, vmin.y, vmax.z),
            ),
            (
                Vec3::new(vmax.x, vmin.y, vmax.z),
                Vec3::new(vmin.x, vmin.y, vmax.z),
            ),
            (
                Vec3::new(vmin.x, vmin.y, vmax.z),
                Vec3::new(vmin.x, vmin.y, vmin.z),
            ),
            /* top */
            (
                Vec3::new(vmin.x, vmax.y, vmin.z),
                Vec3::new(vmax.x, vmax.y, vmin.z),
            ),
            (
                Vec3::new(vmax.x, vmax.y, vmin.z),
                Vec3::new(vmax.x, vmax.y, vmax.z),
            ),
            (
                Vec3::new(vmax.x, vmax.y, vmax.z),
                Vec3::new(vmin.x, vmax.y, vmax.z),
            ),
            (
                Vec3::new(vmin.x, vmax.y, vmax.z),
                Vec3::new(vmin.x, vmax.y, vmin.z),
            ),
            /* corners */
            (
                Vec3::new(vmin.x, vmin.y, vmin.z),
                Vec3::new(vmin.x, vmax.y, vmin.z),
            ),
            (
                Vec3::new(vmax.x, vmin.y, vmin.z),
                Vec3::new(vmax.x, vmax.y, vmin.z),
            ),
            (
                Vec3::new(vmax.x, vmin.y, vmax.z),
                Vec3::new(vmax.x, vmax.y, vmax.z),
            ),
            (
                Vec3::new(vmin.x, vmin.y, vmax.z),
                Vec3::new(vmin.x, vmax.y, vmax.z),
            ),
        ];

        for (start, end) in lines {
            if let (Some(start), Some(end)) = (
                self.view_controller.world_to_screen(&start, true),
                self.view_controller.world_to_screen(&end, true),
            ) {
                draw.add_line(start, end, color).build();
            }
        }
    }

    fn render_overlay(&self, ui: &imgui::Ui) {
        let settings = self.settings.borrow();

        {
            let text_buf;
            let text = obfstr!(text_buf = "Valthrun Overlay");
            
            ui.set_cursor_pos([
                ui.window_size()[0] - ui.calc_text_size(text)[0] - 10.0,
                10.0,
            ]);
            ui.text(text);
        }
        {
            let text = format!("{:.2} FPS", ui.io().framerate);
            ui.set_cursor_pos([
                ui.window_size()[0] - ui.calc_text_size(&text)[0] - 10.0,
                24.0,
            ]);
            ui.text(text)
        }

        ui.set_cursor_pos([10.0, 300.0]);

        ui.text(format!("{} players alive", self.players.len()));
        for entry in self.players.iter() {
            ui.text(format!(
                "{} ({}) | {:?}",
                entry.player_name, entry.player_health, entry.position
            ));
        }

        let draw = ui.get_window_draw_list();
        for entry in self.players.iter() {
            if entry.local {
                continue;
            }

            let position = entry.position;

            if settings.player_pos_dot {
                if let Some(mut screen_position) =
                    self.view_controller.world_to_screen(&position, false)
                {
                    draw.add_circle(screen_position, 8.0, ImColor32::from_rgb(255, 0, 0))
                        .filled(true)
                        .build();

                    screen_position.y -= 10.0;
                    draw.add_text(
                        screen_position,
                        ImColor32::from_rgb(0, 255, 0),
                        &entry.debug_text,
                    );
                }
            }

            if settings.esp_skeleton {
                let bones = entry.model.bones.iter()
                    .zip(entry.bone_states.iter());

                for (bone, state) in bones {
                    if (bone.flags & BoneFlags::FlagHitbox as u32) == 0 {
                        continue;
                    }

                    let parent_index = if let Some(parent) = bone.parent {
                        parent
                    } else {
                        continue;
                    };

                    let parent_position = match self
                        .view_controller
                        .world_to_screen(&entry.bone_states[parent_index].position, true)
                    {
                        Some(position) => position,
                        None => continue,
                    };
                    let bone_position =
                        match self.view_controller.world_to_screen(&state.position, true) {
                            Some(position) => position,
                            None => continue,
                        };

                    draw.add_line(
                        parent_position,
                        bone_position,
                        ImColor32::from_rgb(0, 255, 255),
                    )
                    .build();
                }
            }

            if settings.esp_boxes {
                self.draw_box_3d(
                    &draw,
                    &(entry.model.vhull_min + entry.position),
                    &(entry.model.vhull_max + entry.position),
                    ImColor32::from_rgb(255, 0, 255),
                );
                //self.draw_box_3d(&draw, &(model.vview_min + entry.position), &(model.vview_max + entry.position), ImColor32::from_rgb(0, 0, 255));
            }
        }
    }

    fn render_settings(&self, ui: &imgui::Ui) {
        ui.window(obfstr!("Valthrun"))
            .size([600.0, 300.0], Condition::FirstUseEver)
            .build(|| {
                ui.text(obfstr!("Valthrun an open source CS2 external read only kernel cheat."));
                ui.separator();
                let mouse_pos = ui.io().mouse_pos;
                ui.text(format!(
                    "Mouse Position: ({:.1},{:.1})",
                    mouse_pos[0], mouse_pos[1]
                ));

                let mut settings = self.settings.borrow_mut();
                ui.checkbox(obfstr!("Player Position Dots"), &mut settings.player_pos_dot);
                ui.checkbox(obfstr!("ESP Boxes"), &mut settings.esp_boxes);
                ui.checkbox(obfstr!("ESP Skeletons"), &mut settings.esp_skeleton);
            });
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cs2 = Arc::new(CS2Handle::create()?);

    /*
     * Please no not analyze me:
     * https://www.unknowncheats.me/wiki/Valve_Anti-Cheat:VAC_external_tool_detection_(and_more)
     *
     * Even tough we don't have open handles to CS2 we don't want anybody to read our process.
     */
    cs2.protect_process()
        .with_context(|| obfstr!("failed to protect process").to_string())?;

    let cs2_offsets = Arc::new(
        CS2Offsets::resolve_offsets(&cs2)
            .with_context(|| obfstr!("failed to load CS2 offsets").to_string())?
    );

    let mut app = Application {
        cs2,
        cs2_entities: EntitySystem::new(cs2_offsets.clone()),
        cs2_offsets: cs2_offsets.clone(),

        settings_visible: false,

        players: Vec::with_capacity(16),
        model_cache: Default::default(),

        view_controller: ViewController::new(cs2_offsets.clone()),

        settings: RefCell::new(AppSettings {
            esp_boxes: true,
            esp_skeleton: true,
            player_pos_dot: true,
        }),
    };
    
    let overlay = overlay::init(obfstr!("CS2 Overlay"), obfstr!("Counter-Strike 2"))?;
    overlay.main_loop(move |run, ui| {
        if let Err(err) = app.update(ui) {
            log::error!("{:#}", err);
            *run = false;
            return;
        };

        app.render(ui);
    });
}
