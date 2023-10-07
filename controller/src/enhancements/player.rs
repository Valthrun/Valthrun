use std::{ffi::CStr, sync::Arc};

use anyhow::Context;
use cs2::{BoneFlags, CEntityIdentityEx, CS2Model};
use cs2_schema_declaration::{define_schema, Ptr};
use cs2_schema_generated::cs2::client::{CModelState, CSkeletonInstance, C_CSPlayerPawn};
use num_derive::FromPrimitive as DFromPrimitive;
use num_traits::FromPrimitive;
use obfstr::obfstr;

use crate::{
    settings::{AppSettings, EspBoxType},
    view::ViewController,
};

use super::Enhancement;

#[derive(DFromPrimitive)]
pub enum WeaponId {
    Unknown = 0,
    Deagle = 1,
    Elite = 2,
    Fiveseven = 3,
    Glock = 4,
    Ak47 = 7,
    Aug = 8,
    Awp = 9,
    Famas = 10,
    G3sg1 = 11,
    Galilar = 13,
    M249 = 14,
    M4a1 = 16,
    Mac10 = 17,
    P90 = 19,
    Ump45 = 24,
    Xm1014 = 25,
    Bizon = 26,
    Mag7 = 27,
    Negev = 28,
    Sawedoff = 29,
    Tec9 = 30,
    Taser = 31,
    Hkp2000 = 32,
    Mp7 = 33,
    Mp9 = 34,
    Nova = 35,
    P250 = 36,
    Scar20 = 38,
    Sg556 = 39,
    Ssg08 = 40,
    Knife = 42,
    Flashbang = 43,
    Hegrenade = 44,
    Smokegrenade = 45,
    Molotov = 46,
    Decoy = 47,
    Incgrenade = 48,
    C4 = 49,
    KnifeT = 59,
    M4a1silencer = 60,
    UspSilencer = 61,
    CZ75a = 63,
    Revolver = 64,
    KnifeBayonet = 500,
    KnifeFlip = 505,
    KnifeGut = 506,
    KnifeKarambit = 507,
    KnifeM9Bayonet = 508,
    KnifeTactical = 509,
    KnifeFalchion = 512,
    KnifeSurvivalBowie = 514,
    KnifeButterfly = 515,
    KnifePush = 516,
}

impl WeaponId {
    pub fn display_name(&self) -> Option<&'static str> {
        Some(match self {
            Self::Deagle => "Deagle",
            Self::Elite => "Elite",
            Self::Fiveseven => "Five & Seven",
            Self::Glock => "Glock",
            Self::Ak47 => "Ak-47",
            Self::Aug => "Aug",
            Self::Awp => "Awp",
            Self::Famas => "Famas",
            Self::G3sg1 => "G3sg1",
            Self::Galilar => "Galilar",
            Self::M249 => "M249",
            Self::M4a1 => "M4a1",
            Self::Mac10 => "Mac-10",
            Self::P90 => "P90",
            Self::Ump45 => "Ump45",
            Self::Xm1014 => "Xm1014-Shotgun",
            Self::Bizon => "Bizon",
            Self::Mag7 => "Mag7-Shotgun",
            Self::Negev => "Negev",
            Self::Sawedoff => "Sawed-off-Shotgun",
            Self::Tec9 => "Tec9",
            Self::Taser => "Taser",
            Self::Hkp2000 => "P2000",
            Self::Mp7 => "Mp7",
            Self::Mp9 => "Mp9",
            Self::Nova => "Nova-Shotgun",
            Self::P250 => "P250",
            Self::Scar20 => "Scar-20",
            Self::Sg556 => "Sg556",
            Self::Ssg08 => "Ssg08",
            Self::Knife => "Knife",
            Self::Flashbang => "Flashbang",
            Self::Hegrenade => "Hegrenade",
            Self::Smokegrenade => "Smokegrenade",
            Self::Molotov => "Molotov",
            Self::Decoy => "Decoy",
            Self::Incgrenade => "Incgrenade",
            Self::C4 => "C4",
            Self::KnifeT => "KnifeT",
            Self::M4a1silencer => "M4a1-silencer",
            Self::UspSilencer => "Usp-Silencer",
            Self::CZ75a => "CZ75a",
            Self::Revolver => "Revolver",
            Self::KnifeBayonet => "KnifeBayonet",
            Self::KnifeFlip => "KnifeFlip",
            Self::KnifeGut => "KnifeGut",
            Self::KnifeKarambit => "KnifeKarambit",
            Self::KnifeM9Bayonet => "KnifeM9Bayonet",
            Self::KnifeTactical => "KnifeTactical",
            Self::KnifeFalchion => "KnifeFalchion",
            Self::KnifeSurvivalBowie => "KnifeSurvivalBowie",
            Self::KnifeButterfly => "KnifeButterfly",
            Self::KnifePush => "KnifePush",
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum TeamType {
    Local,
    Enemy,
    Friendly,
}

pub struct PlayerInfo {
    pub controller_entity_id: u32,
    pub team_id: u8,

    pub player_health: i32,
    pub player_name: String,
    pub weapon: WeaponId,

    pub position: nalgebra::Vector3<f32>,
    pub model: Arc<CS2Model>,
    pub bone_states: Vec<BoneStateData>,
}

impl PlayerInfo {
    pub fn calculate_screen_height(&self, view: &ViewController) -> Option<f32> {
        let entry_lower = view.world_to_screen(&(self.model.vhull_min + self.position), true)?;
        let entry_upper = view.world_to_screen(&(self.model.vhull_max + self.position), true)?;

        Some((entry_lower.y - entry_upper.y).abs())
    }
}

pub struct BoneStateData {
    pub position: nalgebra::Vector3<f32>,
}

impl TryFrom<CBoneStateData> for BoneStateData {
    type Error = anyhow::Error;

    fn try_from(value: CBoneStateData) -> Result<Self, Self::Error> {
        Ok(Self {
            position: nalgebra::Vector3::from_row_slice(&value.position()?),
        })
    }
}

define_schema! {
    pub struct CBoneStateData[0x20] {
        pub position: [f32; 3] = 0x00,
        pub scale: f32 = 0x0C,
        pub rotation: [f32; 4] = 0x10,
    }
}

trait CModelStateEx {
    #[allow(non_snake_case)]
    fn m_hModel(&self) -> anyhow::Result<Ptr<Ptr<()>>>;
    fn bone_state_data(&self) -> anyhow::Result<Ptr<[CBoneStateData]>>;
}

impl CModelStateEx for CModelState {
    #[allow(non_snake_case)]
    fn m_hModel(&self) -> anyhow::Result<Ptr<Ptr<()>>> {
        self.memory.reference_schema(0xA0)
    }

    fn bone_state_data(&self) -> anyhow::Result<Ptr<[CBoneStateData]>> {
        self.memory.reference_schema(0x80)
    }
}

pub struct PlayerESP {
    players: Vec<PlayerInfo>,
    local_team_id: u8,
}

impl PlayerESP {
    pub fn new() -> Self {
        PlayerESP {
            players: Default::default(),
            local_team_id: 0,
        }
    }

    fn generate_player_info(
        &self,
        ctx: &crate::UpdateContext,
        player_pawn: &Ptr<C_CSPlayerPawn>,
    ) -> anyhow::Result<Option<PlayerInfo>> {
        let player_pawn = player_pawn
            .read_schema()
            .with_context(|| obfstr!("failed to read player pawn data").to_string())?;

        let player_health = player_pawn.m_iHealth()?;
        if player_health <= 0 {
            return Ok(None);
        }

        /* Will be an instance of CSkeletonInstance */
        let game_screen_node = player_pawn
            .m_pGameSceneNode()?
            .cast::<CSkeletonInstance>()
            .read_schema()?;
        if game_screen_node.m_bDormant()? {
            return Ok(None);
        }

        let controller_handle = player_pawn.m_hController()?;
        let current_controller = ctx.cs2_entities.get_by_handle(&controller_handle)?;

        let player_team = player_pawn.m_iTeamNum()?;
        let player_name = if let Some(identity) = &current_controller {
            let player_controller = identity.entity()?.reference_schema()?;
            CStr::from_bytes_until_nul(&player_controller.m_iszPlayerName()?)
                .context("player name missing nul terminator")?
                .to_str()
                .context("invalid player name")?
                .to_string()
        } else {
            "unknown".to_string()
        };

        let position =
            nalgebra::Vector3::<f32>::from_column_slice(&game_screen_node.m_vecAbsOrigin()?);

        let model = game_screen_node
            .m_modelState()?
            .m_hModel()?
            .read_schema()?
            .address()?;

        let model = ctx.model_cache.lookup(model)?;
        let bone_states = game_screen_node
            .m_modelState()?
            .bone_state_data()?
            .read_entries(model.bones.len())?
            .into_iter()
            .map(|bone| bone.try_into())
            .try_collect()?;

        let weapon = player_pawn.m_pClippingWeapon()?.read_schema()?;
        let weapon_type = weapon
            .m_AttributeManager()?
            .m_Item()?
            .m_iItemDefinitionIndex()?;

        Ok(Some(PlayerInfo {
            controller_entity_id: controller_handle.get_entity_index(),
            team_id: player_team,

            player_name,
            player_health,
            weapon: WeaponId::from_u16(weapon_type).unwrap_or(WeaponId::Unknown),

            position,
            bone_states,
            model: model.clone(),
        }))
    }
}

impl Enhancement for PlayerESP {
    fn update_settings(
        &mut self,
        ui: &imgui::Ui,
        settings: &mut AppSettings,
    ) -> anyhow::Result<bool> {
        let mut updated = false;

        if let Some(hotkey) = &settings.esp_toogle {
            if ui.is_key_pressed_no_repeat(hotkey.0) {
                log::debug!("Toggle player ESP");
                settings.esp = !settings.esp;
                updated = true;
            }
        }

        Ok(updated)
    }

    fn update(&mut self, ctx: &crate::UpdateContext) -> anyhow::Result<()> {
        self.players.clear();

        if !ctx.settings.esp
            || !(ctx.settings.esp_boxes
                || ctx.settings.esp_skeleton
                || ctx.settings.esp_info_health)
        {
            return Ok(());
        }

        self.players.reserve(16);

        let local_player_controller = ctx
            .cs2_entities
            .get_local_player_controller()?
            .try_reference_schema()
            .with_context(|| obfstr!("failed to read local player controller").to_string())?;

        let local_player_controller = match local_player_controller {
            Some(controller) => controller,
            None => {
                /* We're currently not connected */
                return Ok(());
            }
        };

        let observice_entity_handle = if local_player_controller.m_bPawnIsAlive()? {
            local_player_controller.m_hPawn()?.get_entity_index()
        } else {
            let local_obs_pawn = match {
                ctx.cs2_entities
                    .get_by_handle(&local_player_controller.m_hObserverPawn()?)?
            } {
                Some(pawn) => pawn.entity()?.reference_schema()?,
                None => {
                    /* this is odd... */
                    return Ok(());
                }
            };

            local_obs_pawn
                .m_pObserverServices()?
                .read_schema()?
                .m_hObserverTarget()?
                .get_entity_index()
        };

        self.local_team_id = local_player_controller.m_iPendingTeamNum()?;

        for entity_identity in ctx.cs2_entities.all_identities() {
            if entity_identity.handle::<()>()?.get_entity_index() == observice_entity_handle {
                /* current pawn we control/observe */
                continue;
            }

            let entity_class = ctx
                .class_name_cache
                .lookup(&entity_identity.entity_class_info()?)?;
            if !entity_class
                .map(|name| *name == "C_CSPlayerPawn")
                .unwrap_or(false)
            {
                /* entity is not a player pawn */
                continue;
            }

            let player_pawn = entity_identity.entity_ptr::<C_CSPlayerPawn>()?;
            match self.generate_player_info(ctx, &player_pawn) {
                Ok(Some(info)) => self.players.push(info),
                Ok(None) => {}
                Err(error) => {
                    log::warn!(
                        "Failed to generate player pawn ESP info for {:X}: {:#}",
                        player_pawn.address()?,
                        error
                    );
                }
            }
        }

        Ok(())
    }

    fn render(&self, settings: &AppSettings, ui: &imgui::Ui, view: &ViewController) {
        let draw = ui.get_window_draw_list();
        for entry in self.players.iter() {
            let esp_color = if entry.team_id == self.local_team_id {
                if !settings.esp_enabled_team {
                    continue;
                }

                &settings.esp_color_team
            } else {
                if !settings.esp_enabled_enemy {
                    continue;
                }

                &settings.esp_color_enemy
            };

            if settings.esp_skeleton {
                let bones = entry.model.bones.iter().zip(entry.bone_states.iter());

                for (bone, state) in bones {
                    if (bone.flags & BoneFlags::FlagHitbox as u32) == 0 {
                        continue;
                    }

                    let parent_index = if let Some(parent) = bone.parent {
                        parent
                    } else {
                        continue;
                    };

                    let parent_position = match view
                        .world_to_screen(&entry.bone_states[parent_index].position, true)
                    {
                        Some(position) => position,
                        None => continue,
                    };
                    let bone_position = match view.world_to_screen(&state.position, true) {
                        Some(position) => position,
                        None => continue,
                    };

                    draw.add_line(parent_position, bone_position, *esp_color)
                        .thickness(settings.esp_skeleton_thickness)
                        .build();
                }
            }

            if settings.esp_boxes {
                match settings.esp_box_type {
                    EspBoxType::Box2D => {
                        view.draw_box_2d(
                            &draw,
                            &(entry.model.vhull_min + entry.position),
                            &(entry.model.vhull_max + entry.position),
                            (*esp_color).into(),
                            settings.esp_boxes_thickness,
                        );
                    }
                    EspBoxType::Box3D => {
                        view.draw_box_3d(
                            &draw,
                            &(entry.model.vhull_min + entry.position),
                            &(entry.model.vhull_max + entry.position),
                            (*esp_color).into(),
                            settings.esp_boxes_thickness,
                        );
                    }
                }
            }

            if settings.esp_info_health || settings.esp_info_weapon {
                if let Some(pos) = view.world_to_screen(&entry.position, false) {
                    let entry_height = entry.calculate_screen_height(view).unwrap_or(100.0);
                    let target_scale = entry_height * 15.0 / view.screen_bounds.y;
                    let target_scale = target_scale.clamp(0.5, 1.25);
                    ui.set_window_font_scale(target_scale);

                    let mut y_offset = 0.0;
                    if settings.esp_info_health {
                        let text = format!("{} HP", entry.player_health);
                        let [text_width, _] = ui.calc_text_size(&text);

                        let mut pos = pos.clone();
                        pos.x -= text_width / 2.0;
                        pos.y += y_offset;
                        draw.add_text(pos, esp_color.clone(), text);

                        y_offset += ui.text_line_height_with_spacing() * target_scale;
                    }

                    if settings.esp_info_weapon {
                        let text = entry.weapon.display_name().unwrap_or("Unknown Weapon");
                        let [text_width, _] = ui.calc_text_size(&text);

                        let mut pos = pos.clone();
                        pos.x -= text_width / 2.0;
                        pos.y += y_offset;

                        draw.add_text(pos, esp_color.clone(), text);

                        // y_offset += ui.text_line_height_with_spacing() * target_scale;
                    }

                    ui.set_window_font_scale(1.0);
                }
            }
        }
    }
}
