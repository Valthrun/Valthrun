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
use imgui::ImColor32;
use obfstr::obfstr;

use super::Enhancement;
use crate::{
    settings::{
        AppSettings,
        EspBoxType,
        EspConfig,
        EspHealthBar,
        EspPlayerSettings,
        EspSelector,
        EspTracePosition,
    },
    view::{
        KeyToggle,
        ViewController,
    },
    weapon::WeaponId,
};

pub struct PlayerInfo {
    pub controller_entity_id: u32,
    pub team_id: u8,

    pub player_health: i32,
    pub player_has_defuser: bool,
    pub player_name: String,
    pub weapon: WeaponId,
    pub player_flashtime: f32,

    pub position: nalgebra::Vector3<f32>,
    pub model: Arc<CS2Model>,
    pub bone_states: Vec<BoneStateData>,
}

impl PlayerInfo {
    pub fn calculate_player_box(
        &self,
        view: &ViewController,
    ) -> Option<(nalgebra::Vector2<f32>, nalgebra::Vector2<f32>)> {
        view.calculate_box_2d(
            &(self.model.vhull_min + self.position),
            &(self.model.vhull_max + self.position),
        )
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
    toggle: KeyToggle,
    players: Vec<PlayerInfo>,
    local_team_id: u8,
    local_pos: Option<nalgebra::Vector3<f32>>,
}

impl PlayerESP {
    pub fn new() -> Self {
        PlayerESP {
            toggle: KeyToggle::new(),
            players: Default::default(),
            local_team_id: 0,
            local_pos: Default::default(),
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
            /*
             * This is the case for pawns which are not controllel by a player controller.
             * An example would be the main screen player pawns.
             *
             * Note: We're assuming, that uncontroller player pawns are neglectable while being in a match as the do not occurr.
             * Bots (and controller bots) always have a player pawn controller.
             */
            // log::warn!(
            //     "Handle at address {:p} has no valid controller!",
            //     &controller_handle
            // );
            return Ok(None);
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

        let player_flashtime = player_pawn.m_flFlashBangTime()?;

        Ok(Some(PlayerInfo {
            controller_entity_id: controller_handle.get_entity_index(),
            team_id: player_team,

            player_name,
            player_has_defuser,
            player_health,
            weapon: WeaponId::from_id(weapon_type).unwrap_or(WeaponId::Unknown),
            player_flashtime,

            position,
            bone_states,
            model: model.clone(),
        }))
    }

    fn resolve_esp_player_config<'a>(
        &self,
        settings: &'a AppSettings,
        target: &PlayerInfo,
    ) -> Option<&'a EspPlayerSettings> {
        let mut esp_target = Some(EspSelector::PlayerTeamVisibility {
            enemy: target.team_id != self.local_team_id,
            visible: true, // TODO: Implement visibility, maybe rename it to spottet!
        });

        while let Some(target) = esp_target.take() {
            let config_key = target.config_key();

            if settings
                .esp_settings_enabled
                .get(&config_key)
                .cloned()
                .unwrap_or_default()
            {
                if let Some(settings) = settings.esp_settings.get(&config_key) {
                    if let EspConfig::Player(settings) = settings {
                        return Some(settings);
                    }
                }
            }

            esp_target = target.parent();
        }

        None
    }
}

struct PlayerInfoLayout<'a> {
    ui: &'a imgui::Ui,
    draw: &'a imgui::DrawListMut<'a>,

    vmin: nalgebra::Vector2<f32>,
    vmax: nalgebra::Vector2<f32>,

    line_count: usize,
    font_scale: f32,

    has_2d_box: bool,
}

impl<'a> PlayerInfoLayout<'a> {
    pub fn new(
        ui: &'a imgui::Ui,
        draw: &'a imgui::DrawListMut<'a>,
        screen_bounds: mint::Vector2<f32>,
        vmin: nalgebra::Vector2<f32>,
        vmax: nalgebra::Vector2<f32>,
        has_2d_box: bool,
    ) -> Self {
        let target_scale_raw = (vmax.y - vmin.y) / screen_bounds.y * 8.0;
        let target_scale = target_scale_raw.clamp(0.5, 1.25);
        ui.set_window_font_scale(target_scale);

        Self {
            ui,
            draw,

            vmin,
            vmax,

            line_count: 0,
            font_scale: target_scale,

            has_2d_box,
        }
    }

    pub fn add_line(&mut self, color: impl Into<ImColor32>, text: &str) {
        let [text_width, _] = self.ui.calc_text_size(text);

        let mut pos = if self.has_2d_box {
            let mut pos = self.vmin;
            pos.x = self.vmax.x + 5.0;
            pos
        } else {
            let mut pos = self.vmax.clone();
            pos.x -= (self.vmax.x - self.vmin.x) / 2.0;
            pos.x -= text_width / 2.0;
            pos
        };
        pos.y += self.line_count as f32 * self.font_scale * (self.ui.text_line_height())
            + 4.0 * self.line_count as f32;

        self.draw.add_text([pos.x, pos.y], color, text);
        self.line_count += 1;
    }
}

impl Drop for PlayerInfoLayout<'_> {
    fn drop(&mut self) {
        self.ui.set_window_font_scale(1.0);
    }
}
impl Enhancement for PlayerESP {
    fn update(&mut self, ctx: &crate::UpdateContext) -> anyhow::Result<()> {
        if self
            .toggle
            .update(&ctx.settings.esp_mode, ctx.input, &ctx.settings.esp_toogle)
        {
            ctx.cs2.add_metrics_record(
                obfstr!("feature-esp-toggle"),
                &format!(
                    "enabled: {}, mode: {:?}",
                    self.toggle.enabled, ctx.settings.esp_mode
                ),
            );
        }

        self.players.clear();
        if !self.toggle.enabled {
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
                let local_pawn = entity_identity
                    .entity_ptr::<C_CSPlayerPawn>()?
                    .read_schema()?;
                let local_pos =
                    nalgebra::Vector3::<f32>::from_column_slice(&local_pawn.m_vOldOrigin()?);
                self.local_pos = Some(local_pos);
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
        const UNITS_TO_METERS: f32 = 0.01905;
        for entry in self.players.iter() {
            let distance = if let Some(local_pos) = self.local_pos {
                let distance = (entry.position - local_pos).norm() * UNITS_TO_METERS;
                distance
            } else {
                0.0
            };
            let esp_settings = match self.resolve_esp_player_config(settings, entry) {
                Some(settings) => settings,
                None => continue,
            };
            if esp_settings.near_players {
                if distance > esp_settings.near_players_distance {
                    continue;
                }
            }

            let player_rel_health = (entry.player_health as f32 / 100.0).clamp(0.0, 1.0);
            let player_2d_box = entry.calculate_player_box(view);

            if esp_settings.skeleton {
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

                    draw.add_line(
                        parent_position,
                        bone_position,
                        esp_settings
                            .skeleton_color
                            .calculate_color(player_rel_health, distance),
                    )
                    .thickness(esp_settings.skeleton_width)
                    .build();
                }
            }

            match esp_settings.box_type {
                EspBoxType::Box2D => {
                    if let Some((vmin, vmax)) = &player_2d_box {
                        draw.add_rect(
                            [vmin.x, vmin.y],
                            [vmax.x, vmax.y],
                            esp_settings
                                .box_color
                                .calculate_color(player_rel_health, distance),
                        )
                        .thickness(esp_settings.box_width)
                        .build();
                    }
                }
                EspBoxType::Box3D => {
                    view.draw_box_3d(
                        &draw,
                        &(entry.model.vhull_min + entry.position),
                        &(entry.model.vhull_max + entry.position),
                        esp_settings
                            .box_color
                            .calculate_color(player_rel_health, distance)
                            .into(),
                        esp_settings.box_width,
                    );
                }
                EspBoxType::None => {}
            }

            if let Some((vmin, vmax)) = &player_2d_box {
                let box_bounds = match esp_settings.health_bar {
                    EspHealthBar::None => None,
                    EspHealthBar::Left => {
                        let xoffset =
                            vmin.x - esp_settings.box_width / 2.0 - esp_settings.health_bar_width;

                        Some([
                            xoffset,
                            vmin.y - esp_settings.box_width / 2.0,
                            esp_settings.health_bar_width,
                            vmax.y - vmin.y + esp_settings.box_width,
                        ])
                    }
                    EspHealthBar::Right => {
                        let xoffset = vmax.x + esp_settings.box_width / 2.0;

                        Some([
                            xoffset,
                            vmin.y - esp_settings.box_width / 2.0,
                            esp_settings.health_bar_width,
                            vmax.y - vmin.y + esp_settings.box_width,
                        ])
                    }
                    EspHealthBar::Top => {
                        let yoffset =
                            vmin.y - esp_settings.box_width / 2.0 - esp_settings.health_bar_width;

                        Some([
                            vmin.x - esp_settings.box_width / 2.0,
                            yoffset,
                            vmax.x - vmin.x + esp_settings.box_width,
                            esp_settings.health_bar_width,
                        ])
                    }
                    EspHealthBar::Bottom => {
                        let yoffset = vmax.y + esp_settings.box_width / 2.0;

                        Some([
                            vmin.x - esp_settings.box_width / 2.0,
                            yoffset,
                            vmax.x - vmin.x + esp_settings.box_width,
                            esp_settings.health_bar_width,
                        ])
                    }
                };

                if let Some([mut box_x, mut box_y, mut box_width, mut box_height]) = box_bounds {
                    const BORDER_WIDTH: f32 = 1.0;
                    draw.add_rect(
                        [box_x + BORDER_WIDTH / 2.0, box_y + BORDER_WIDTH / 2.0],
                        [
                            box_x + box_width - BORDER_WIDTH / 2.0,
                            box_y + box_height - BORDER_WIDTH / 2.0,
                        ],
                        [0.0, 0.0, 0.0, 1.0],
                    )
                    .filled(false)
                    .thickness(BORDER_WIDTH)
                    .build();

                    box_x += BORDER_WIDTH / 2.0 + 1.0;
                    box_y += BORDER_WIDTH / 2.0 + 1.0;

                    box_width -= BORDER_WIDTH + 2.0;
                    box_height -= BORDER_WIDTH + 2.0;

                    if box_width < box_height {
                        /* vertical */
                        let yoffset = box_y + (1.0 - player_rel_health) * box_height;
                        draw.add_rect(
                            [box_x, box_y],
                            [box_x + box_width, yoffset],
                            [1.0, 0.0, 0.0, 1.0],
                        )
                        .filled(true)
                        .build();

                        draw.add_rect(
                            [box_x, yoffset],
                            [box_x + box_width, box_y + box_height],
                            [0.0, 1.0, 0.0, 1.0],
                        )
                        .filled(true)
                        .build();
                    } else {
                        /* horizontal */
                        let xoffset = box_x + (1.0 - player_rel_health) * box_width;
                        draw.add_rect(
                            [box_x, box_y],
                            [xoffset, box_y + box_height],
                            [1.0, 0.0, 0.0, 1.0],
                        )
                        .filled(true)
                        .build();

                        draw.add_rect(
                            [xoffset, box_y],
                            [box_x + box_width, box_y + box_height],
                            [0.0, 1.0, 0.0, 1.0],
                        )
                        .filled(true)
                        .build();
                    }
                }
            }

            if let Some((vmin, vmax)) = player_2d_box {
                let mut player_info = PlayerInfoLayout::new(
                    ui,
                    &draw,
                    view.screen_bounds,
                    vmin,
                    vmax,
                    esp_settings.box_type == EspBoxType::Box2D,
                );

                if esp_settings.info_name {
                    player_info.add_line(
                        esp_settings
                            .info_name_color
                            .calculate_color(player_rel_health, distance),
                        &entry.player_name,
                    );
                }

                if esp_settings.info_weapon {
                    let text = entry.weapon.display_name();
                    player_info.add_line(
                        esp_settings
                            .info_weapon_color
                            .calculate_color(player_rel_health, distance),
                        &text,
                    );
                }

                if esp_settings.info_hp_text {
                    let text = format!("{} HP", entry.player_health);
                    player_info.add_line(
                        esp_settings
                            .info_hp_text_color
                            .calculate_color(player_rel_health, distance),
                        &text,
                    );
                }

                let mut player_flags = Vec::new();
                if esp_settings.info_flag_kit && entry.player_has_defuser {
                    player_flags.push("Kit");
                }

                if esp_settings.info_flag_flashed && entry.player_flashtime > 0.0 {
                    player_flags.push("flashed");
                }

                if !player_flags.is_empty() {
                    player_info.add_line(
                        esp_settings
                            .info_flags_color
                            .calculate_color(player_rel_health, distance),
                        &player_flags.join(", "),
                    );
                }
                if esp_settings.info_distance {
                    let text = format!("{:.0}m", distance);
                    player_info.add_line(
                        esp_settings
                            .info_distance_color
                            .calculate_color(player_rel_health, distance),
                        &text,
                    );
                }
            }

            if let Some(pos) = view.world_to_screen(&entry.position, false) {
                let tracer_origin = match esp_settings.tracer_lines {
                    EspTracePosition::TopLeft => Some([0.0, 0.0]),
                    EspTracePosition::TopCenter => Some([view.screen_bounds.x / 2.0, 0.0]),
                    EspTracePosition::TopRight => Some([view.screen_bounds.x, 0.0]),
                    EspTracePosition::Center => {
                        Some([view.screen_bounds.x / 2.0, view.screen_bounds.y / 2.0])
                    }
                    EspTracePosition::BottomLeft => Some([0.0, view.screen_bounds.y]),
                    EspTracePosition::BottomCenter => {
                        Some([view.screen_bounds.x / 2.0, view.screen_bounds.y])
                    }
                    EspTracePosition::BottomRight => {
                        Some([view.screen_bounds.x, view.screen_bounds.y])
                    }
                    EspTracePosition::None => None,
                };

                if let Some(origin) = tracer_origin {
                    draw.add_line(
                        origin,
                        pos,
                        esp_settings
                            .tracer_lines_color
                            .calculate_color(player_rel_health, distance),
                    )
                    .thickness(esp_settings.tracer_lines_width)
                    .build();
                }
            }
        }
    }
}
