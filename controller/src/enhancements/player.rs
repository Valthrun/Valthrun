use cs2::{
    BoneFlags,
    CEntityIdentityEx,
    CS2Model,
    ClassNameCache,
    EntitySystem,
    LocalCameraControllerTarget,
    PlayerPawnInfo,
    PlayerPawnState,
};
use cs2_schema_generated::cs2::client::C_CSPlayerPawn;
use imgui::ImColor32;
use obfstr::obfstr;
use overlay::UnicodeTextRenderer;

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
};

pub struct PlayerESP {
    toggle: KeyToggle,
    players: Vec<PlayerPawnInfo>,
    local_team_id: u8,
}

impl PlayerESP {
    pub fn new() -> Self {
        PlayerESP {
            toggle: KeyToggle::new(),
            players: Default::default(),
            local_team_id: 0,
        }
    }

    fn resolve_esp_player_config<'a>(
        &self,
        settings: &'a AppSettings,
        target: &PlayerPawnInfo,
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

const HEALTH_BAR_MAX_HEALTH: f32 = 100.0;
const HEALTH_BAR_BORDER_WIDTH: f32 = 1.0;
impl Enhancement for PlayerESP {
    fn update(&mut self, ctx: &crate::UpdateContext) -> anyhow::Result<()> {
        let entities = ctx.states.resolve::<EntitySystem>(())?;
        let class_name_cache = ctx.states.resolve::<ClassNameCache>(())?;
        let settings = ctx.states.resolve::<AppSettings>(())?;
        if self
            .toggle
            .update(&settings.esp_mode, ctx.input, &settings.esp_toogle)
        {
            ctx.cs2.add_metrics_record(
                obfstr!("feature-esp-toggle"),
                &format!(
                    "enabled: {}, mode: {:?}",
                    self.toggle.enabled, settings.esp_mode
                ),
            );
        }

        self.players.clear();
        if !self.toggle.enabled {
            return Ok(());
        }

        self.players.reserve(16);

        let local_player_controller = entities.get_local_player_controller()?;
        if local_player_controller.is_null()? {
            return Ok(());
        }

        let local_player_controller = local_player_controller.reference_schema()?;
        self.local_team_id = local_player_controller.m_iPendingTeamNum()?;

        let view_target = ctx.states.resolve::<LocalCameraControllerTarget>(())?;
        let target_entity_id = match &view_target.target_entity_id {
            Some(value) => *value,
            None => return Ok(()),
        };

        for entity_identity in entities.all_identities() {
            if entity_identity.handle::<()>()?.get_entity_index() == target_entity_id {
                continue;
            }

            let entity_class = class_name_cache.lookup(&entity_identity.entity_class_info()?)?;
            if !entity_class
                .map(|name| *name == "C_CSPlayerPawn")
                .unwrap_or(false)
            {
                /* entity is not a player pawn */
                continue;
            }

            let player_pawn = entity_identity.entity_ptr::<C_CSPlayerPawn>()?;
            match ctx
                .states
                .resolve::<PlayerPawnState>(entity_identity.handle::<()>()?.get_entity_index())
            {
                Ok(info) => match &*info {
                    PlayerPawnState::Alive(info) => self.players.push(info.clone()),
                    PlayerPawnState::Dead => continue,
                },
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

    fn render(
        &self,
        states: &utils_state::StateRegistry,
        ui: &imgui::Ui,
        unicode_text: &UnicodeTextRenderer,
    ) -> anyhow::Result<()> {
        let settings = states.resolve::<AppSettings>(())?;
        let view = states.resolve::<ViewController>(())?;

        let draw = ui.get_window_draw_list();
        const UNITS_TO_METERS: f32 = 0.01905;

        let view_world_position = match view.get_camera_world_position() {
            Some(view_world_position) => view_world_position,
            _ => return Ok(()),
        };

        for entry in self.players.iter() {
            let distance = (entry.position - view_world_position).norm() * UNITS_TO_METERS;
            let esp_settings = match self.resolve_esp_player_config(&settings, entry) {
                Some(settings) => settings,
                None => continue,
            };
            if esp_settings.near_players {
                if distance > esp_settings.near_players_distance {
                    continue;
                }
            }

            let player_rel_health = (entry.player_health as f32 / 100.0).clamp(0.0, 1.0);

            let entry_model = states.resolve::<CS2Model>(entry.model_address)?;
            let player_2d_box = view.calculate_box_2d(
                &(entry_model.vhull_min + entry.position),
                &(entry_model.vhull_max + entry.position),
            );

            if esp_settings.skeleton {
                let bones = entry_model.bones.iter().zip(entry.bone_states.iter());

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
                        &(entry_model.vhull_min + entry.position),
                        &(entry_model.vhull_max + entry.position),
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
                    unicode_text.register_unicode_text(&entry.player_name);
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

        Ok(())
    }
}
