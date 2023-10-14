use std::{
    ffi::CStr,
    sync::Arc,
};

use anyhow::{
    Context,
    Result,
};
use cs2::{
    BoneFlags,
    CEntityIdentityEx,
    CS2Model,
};
use cs2_schema_declaration::{
    define_schema,
    Ptr,
};
use cs2_schema_generated::cs2::client::{
    CCSPlayer_ItemServices,
    CModelState,
    CSkeletonInstance,
    C_CSPlayerPawn,
};
use obfstr::obfstr;

use super::Enhancement;
use crate::{
    settings::{
        AppSettings,
        CrosshairType,
        EspBoxType,
        LineStartPosition,
    },
    view::ViewController,
    weapon::WeaponId,
};

pub struct PlayerInfo {
    pub controller_entity_id: u32,
    pub team_id: u8,

    pub player_health: i32,
    pub player_has_defuser: bool,
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

        let player_has_defuser = player_pawn
            .m_pItemServices()?
            .cast::<CCSPlayer_ItemServices>()
            .reference_schema()?
            .m_bHasDefuser()?;

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
            .collect::<Result<Vec<_>>>()?;

        let weapon = player_pawn.m_pClippingWeapon()?.try_read_schema()?;
        let weapon_type = if let Some(weapon) = weapon {
            weapon
                .m_AttributeManager()?
                .m_Item()?
                .m_iItemDefinitionIndex()?
        } else {
            WeaponId::Knife.id()
        };

        Ok(Some(PlayerInfo {
            controller_entity_id: controller_handle.get_entity_index(),
            team_id: player_team,

            player_name,
            player_has_defuser,
            player_health,
            weapon: WeaponId::from_id(weapon_type).unwrap_or(WeaponId::Unknown),

            position,
            bone_states,
            model: model.clone(),
        }))
    }

    pub fn calculate_rainbow_color(value: f32, alpha: f32) -> [f32; 4] {
        let sin_value =
            |offset: f32| (2.0 * std::f32::consts::PI * value * 0.75 + offset).sin() * 0.5 + 1.0;
        let r: f32 = sin_value(0.0);
        let g: f32 = sin_value(2.0 * std::f32::consts::PI / 3.0);
        let b: f32 = sin_value(4.0 * std::f32::consts::PI / 3.0);
        [r, g, b, alpha]
    }

    pub fn calculate_health_color(health_percentage: f32, alpha: f32) -> [f32; 4] {
        let clamped_percentage = health_percentage.clamp(0.0, 1.0);

        let r = 1.0 - clamped_percentage;
        let g = clamped_percentage;
        let b = 0.0;

        [r, g, b, alpha]
    }
}

const HEALTH_BAR_MAX_HEALTH: f32 = 100.0;
const HEALTH_BAR_BORDER_WIDTH: f32 = 1.0;
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

        if !ctx.settings.esp || !(ctx.settings.esp_boxes || ctx.settings.esp_skeleton) {
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
                        if let Some((vmin, vmax)) = view.calculate_box_2d(
                            &(entry.model.vhull_min + entry.position),
                            &(entry.model.vhull_max + entry.position),
                        ) {
                            draw.add_rect([vmin.x, vmin.y], [vmax.x, vmax.y], *esp_color)
                                .thickness(settings.esp_boxes_thickness)
                                .build();

                            if settings.esp_health_bar {
                                let bar_y = vmin.y - settings.esp_boxes_thickness / 2.0
                                    + HEALTH_BAR_BORDER_WIDTH / 2.0;
                                let bar_x =
                                    vmin.x - settings.esp_health_bar_size - HEALTH_BAR_BORDER_WIDTH;

                                let bar_height = vmax.y - vmin.y + settings.esp_boxes_thickness;
                                let bar_width = settings.esp_health_bar_size;

                                /* player health in [0.0; 1.0] */
                                let normalized_player_health = (entry.player_health as f32)
                                    .clamp(0.0, HEALTH_BAR_MAX_HEALTH)
                                    / HEALTH_BAR_MAX_HEALTH;

                                let bar_color = if settings.esp_health_bar_rainbow {
                                    Self::calculate_rainbow_color(
                                        normalized_player_health,
                                        esp_color[3],
                                    )
                                } else {
                                    Self::calculate_health_color(
                                        normalized_player_health,
                                        esp_color[3],
                                    )
                                };

                                draw.add_rect(
                                    [
                                        bar_x + HEALTH_BAR_BORDER_WIDTH,
                                        bar_y
                                            + HEALTH_BAR_BORDER_WIDTH
                                            + bar_height * (1.0 - normalized_player_health),
                                    ],
                                    [
                                        bar_x + bar_width - HEALTH_BAR_BORDER_WIDTH,
                                        bar_y + bar_height - HEALTH_BAR_BORDER_WIDTH * 2.0,
                                    ],
                                    bar_color,
                                )
                                .filled(true)
                                .build();

                                draw.add_rect(
                                    [bar_x, bar_y],
                                    [
                                        bar_x + bar_width - HEALTH_BAR_BORDER_WIDTH,
                                        bar_y + bar_height - HEALTH_BAR_BORDER_WIDTH,
                                    ],
                                    [0.0, 0.0, 0.0, esp_color[3]],
                                )
                                .thickness(HEALTH_BAR_BORDER_WIDTH)
                                .build();
                            }
                        }
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

            if settings.esp_info_health || settings.esp_info_weapon || settings.esp_info_kit {
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
                        let text = entry.weapon.display_name();
                        let [text_width, _] = ui.calc_text_size(&text);

                        let mut pos = pos.clone();
                        pos.x -= text_width / 2.0;
                        pos.y += y_offset;

                        draw.add_text(pos, esp_color.clone(), text);

                        y_offset += ui.text_line_height_with_spacing() * target_scale;
                    }

                    if entry.player_has_defuser && settings.esp_info_kit {
                        let text = "KIT";
                        let [text_width, _] = ui.calc_text_size(&text);
                        let mut pos = pos.clone();
                        pos.x -= text_width / 2.0;
                        pos.y += y_offset;
                        draw.add_text(pos, esp_color.clone(), text);

                        //y_offset += ui.text_line_height_with_spacing() * target_scale;
                    }

                    ui.set_window_font_scale(1.0);
                }
            }

            if settings.esp_lines {
                if let Some(player_screen_pos) = view.world_to_screen(&entry.position, false) {
                    let screen_size = [view.screen_bounds.x, view.screen_bounds.y];
                    let start_pos = match settings.esp_lines_position {
                        LineStartPosition::TopLeft => [0.0, 0.0],
                        LineStartPosition::TopCenter => [screen_size[0] / 2.0, 0.0],
                        LineStartPosition::TopRight => [screen_size[0], 0.0],
                        LineStartPosition::Center => [screen_size[0] / 2.0, screen_size[1] / 2.0],
                        LineStartPosition::BottomLeft => [0.0, screen_size[1]],
                        LineStartPosition::BottomCenter => [screen_size[0] / 2.0, screen_size[1]],
                        LineStartPosition::BottomRight => [screen_size[0], screen_size[1]],
                    };
                    draw.add_line(start_pos, player_screen_pos, *esp_color)
                        .thickness(1.0)
                        .build();
                }
            }
            if settings.show_crosshair {
                let crosshair_size = settings.crosshair_size;

                let window_size = [view.screen_bounds.x, view.screen_bounds.y];
                let center_x = window_size[0] / 2.0;
                let center_y = window_size[1] / 2.0;

                let crosshair_color = settings.crosshair_color;
                match settings.crosshair_type {
                    CrosshairType::Circle => {
                        let circle_radius = crosshair_size;

                        draw.add_circle([center_x, center_y], circle_radius, crosshair_color)
                            .num_segments(32)
                            .thickness(1.0)
                            .build();
                        if settings.circle_crosshair_filled {
                            draw.add_circle([center_x, center_y], circle_radius, crosshair_color)
                                .num_segments(32)
                                .thickness(1.0)
                                .filled(true)
                                .build();
                        }
                    }
                    CrosshairType::Arrow => {
                        draw.add_line(
                            [center_x - crosshair_size, center_y],
                            [center_x + crosshair_size, center_y],
                            crosshair_color,
                        )
                        .thickness(1.0)
                        .build();

                        draw.add_line(
                            [center_x, center_y - crosshair_size],
                            [center_x, center_y + crosshair_size],
                            crosshair_color,
                        )
                        .thickness(1.0)
                        .build();
                    }
                }
            }
        }
    }
}
