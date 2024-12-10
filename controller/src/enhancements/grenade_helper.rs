use std::{
    collections::BTreeMap,
    f32,
};

use anyhow::Context;
use cs2::{
    CEntityIdentityEx,
    ClassNameCache,
    LocalCameraControllerTarget,
    StateCS2Memory,
    StateCurrentMap,
    StateEntityList,
};
use cs2_schema_generated::cs2::client::{
    C_BaseEntity,
    C_CSPlayerPawnBase,
};
use imgui::Condition;
use nalgebra::{
    Vector2,
    Vector3,
};
use obfstr::obfstr;
use overlay::UnicodeTextRenderer;
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use super::Enhancement;
use crate::{
    settings::{
        AppSettings,
        GrenadeSpotInfo,
    },
    view::ViewController,
};

#[derive(Default)]
struct GrenadeState {
    display_opacity: f32,
    position_held: bool,
    angle_held: bool,
}

impl GrenadeSpotInfo {
    pub fn eye_position(&self) -> Vector3<f32> {
        Vector3::from_column_slice(&self.eye_position)
    }

    pub fn eye_direction(&self) -> Vector2<f32> {
        Vector2::from_column_slice(&self.eye_direction)
    }
}

pub const DEFAULT_EYE_HEIGHT: Vector3<f32> = Vector3::new(0.0, 0.0, 64.093811);
pub struct GrenadeHelper {
    grenade_states: BTreeMap<usize, GrenadeState>,
    current_map: String,

    eye_height: Vector3<f32>,
}

impl GrenadeHelper {
    pub fn new() -> Self {
        Self {
            grenade_states: Default::default(),
            current_map: "<empty>".to_string(),

            eye_height: DEFAULT_EYE_HEIGHT,
        }
    }
}

pub enum StateGrenadeHelperPlayerLocation {
    Unknown,
    Valid {
        eye_position: Vector3<f32>,
        eye_direction: Vector2<f32>,
    },
}

impl State for StateGrenadeHelperPlayerLocation {
    type Parameter = ();

    fn create(states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        let entities = states.resolve::<StateEntityList>(())?;
        let class_name_cache = states.resolve::<ClassNameCache>(())?;

        let view_target = states.resolve::<LocalCameraControllerTarget>(())?;
        let Some(target_entity_id) = view_target.target_entity_id else {
            /* We're currently not spectating an entity. Not showing grenade helper. */
            return Ok(Self::Unknown);
        };

        let local_entity_identity = entities
            .identity_from_index(target_entity_id)
            .context("missing view target entity")?;

        let local_entity_class = class_name_cache
            .lookup(&local_entity_identity.entity_class_info()?)?
            .with_context(|| {
                obfstr!("failed to resolve entity class for current entity").to_string()
            })?;
        if local_entity_class != "C_CSPlayerPawn" {
            /* We're currently not a CS player, hence no grenade helper */
            return Ok(Self::Unknown);
        }

        let memory = states.resolve::<StateCS2Memory>(())?;
        let local_entity = local_entity_identity
            .entity_ptr::<dyn C_CSPlayerPawnBase>()?
            .value_reference(memory.view_arc())
            .context("local entity nullptr")?;

        let local_position = Vector3::from_column_slice(
            &local_entity
                .m_pGameSceneNode()?
                .value_reference(memory.view_arc())
                .context("m_pGameSceneNode nullptr")?
                .m_vecAbsOrigin()?,
        );

        let eye_angles = Vector2::from_column_slice(&local_entity.m_angEyeAngles()?[0..2]);
        Ok(Self::Valid {
            eye_position: local_position + DEFAULT_EYE_HEIGHT,
            eye_direction: eye_angles,
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}

impl Enhancement for GrenadeHelper {
    fn update(&mut self, ctx: &crate::UpdateContext) -> anyhow::Result<()> {
        let settings = ctx.states.resolve::<AppSettings>(())?;
        let settings = &settings.grenade_helper;
        if !settings.active {
            return Ok(());
        }

        for state in self.grenade_states.values_mut() {
            state.display_opacity = 0.0;
            state.angle_held = false;
            state.position_held = false;
        }

        {
            let current_map = ctx.states.resolve::<StateCurrentMap>(())?;
            self.current_map = current_map
                .current_map
                .clone()
                .unwrap_or_else(|| "<empty>".to_string());
        }

        let Some(map_grenades) = settings.map_spots.get(&self.current_map) else {
            return Ok(());
        };

        let StateGrenadeHelperPlayerLocation::Valid {
            eye_position: local_position,
            eye_direction: local_direction,
        } = *ctx.states.resolve(())?
        else {
            /* The error message contains, why we do not have a position but this is ignoreable */
            return Ok(());
        };

        for grenade in map_grenades {
            let state = self
                .grenade_states
                .entry(grenade.id)
                .or_insert_with(Default::default);

            let dist_xy = grenade
                .eye_position()
                .xy()
                .metric_distance(&Vector2::new(local_position.x, local_position.y));

            if dist_xy > settings.circle_distance {
                /* grenade spot is out of view distance */
                continue;
            }

            let fadein_distance = settings.circle_distance * 0.2;
            if dist_xy > settings.circle_distance - fadein_distance {
                state.display_opacity = (settings.circle_distance - dist_xy) / fadein_distance;
            } else {
                state.display_opacity = 1.0;
            }

            state.position_held = dist_xy < settings.circle_radius
                && (grenade.eye_position().z - local_position.z).abs() < 100.0;

            let direction_diff = (grenade.eye_direction() - local_direction).abs();
            state.angle_held = direction_diff.x < settings.angle_threshold_pitch
                && direction_diff.y < settings.angle_threshold_yaw;
        }

        Ok(())
    }

    fn render(
        &self,
        states: &StateRegistry,
        ui: &imgui::Ui,
        _unicode_text: &UnicodeTextRenderer,
    ) -> anyhow::Result<()> {
        let view = states.resolve::<ViewController>(())?;
        let settings = states.resolve::<AppSettings>(())?;
        let settings = &settings.grenade_helper;
        if !settings.active {
            return Ok(());
        }

        let Some(grenades) = settings.map_spots.get(&self.current_map) else {
            return Ok(());
        };
        let draw_list = ui.get_window_draw_list();
        for grenade in grenades {
            let Some(state) = self.grenade_states.get(&grenade.id) else {
                continue;
            };
            if state.display_opacity <= 0.0 {
                continue;
            }

            let mut color = if state.position_held {
                settings.color_position_active
            } else {
                settings.color_position
            };
            color.set_alpha_f32(state.display_opacity);

            if let Some(body_position) =
                view.world_to_screen(&(grenade.eye_position() - self.eye_height), false)
            {
                {
                    let mut points = Vec::with_capacity(settings.circle_segments + 1);
                    for index in 0..=settings.circle_segments {
                        let offset =
                            index as f32 * f32::consts::TAU / settings.circle_segments as f32;

                        let point_3d = grenade.eye_position()
                            + Vector3::new(
                            settings.circle_radius * offset.sin(),
                            settings.circle_radius * offset.cos(),
                            0.0,
                        )
                            - self.eye_height;

                        let Some(point) = view.world_to_screen(&point_3d, true) else {
                            continue;
                        };
                        points.push(point);
                    }
                    draw_list.add_polyline(points, color.as_f32()).build();
                    draw_list
                        .add_circle(body_position, 3.0, color.as_f32())
                        .filled(true)
                        .build();
                }

                {
                    let text_height = 2.0 * ui.text_line_height();
                    ui.set_cursor_pos([
                        body_position.x + 10.0,
                        body_position.y - text_height / 2.0,
                    ]);
                    if grenade.grenade_types.is_empty() {
                        ui.text_colored(color.as_f32(), "All grenades");
                    } else {
                        let grenade_display_names = grenade
                            .grenade_types
                            .iter()
                            .map(|value| value.display_name())
                            .collect::<Vec<_>>();

                        ui.text_colored(color.as_f32(), grenade_display_names.join(", "));
                    }

                    ui.set_cursor_pos([
                        body_position.x + 10.0,
                        body_position.y - text_height / 2.0 + ui.text_line_height(),
                    ]);
                    ui.text_colored(color.as_f32(), &grenade.name);
                }
            }

            if state.position_held {
                let vec = Vector3::new(
                    (-grenade.eye_direction().x * f32::consts::PI / 180.0).cos()
                        * (grenade.eye_direction().y * f32::consts::PI / 180.0).cos(),
                    (-grenade.eye_direction().x * f32::consts::PI / 180.0).cos()
                        * (grenade.eye_direction().y * f32::consts::PI / 180.0).sin(),
                    (-grenade.eye_direction().x * f32::consts::PI / 180.0).sin(),
                );

                if let Some(direction_indicator) =
                    view.world_to_screen(&(grenade.eye_position() + 200.0 * vec), true)
                {
                    let color = if state.angle_held {
                        settings.color_angle_active
                    } else {
                        settings.color_angle
                    };

                    {
                        draw_list
                            .add_circle(direction_indicator, 3.0, color.as_f32())
                            .filled(true)
                            .build();

                        draw_list
                            .add_line(
                                direction_indicator,
                                [ui.window_size()[0] / 2.0, ui.window_size()[1]],
                                color.as_f32(),
                            )
                            .build();
                    }


                    if let Some(_window) = ui
                        .window(format!("##grenade_info_{}", grenade.id))
                        .position(
                            [direction_indicator.x + 10.0, direction_indicator.y],
                            Condition::Always,
                        )
                        .position_pivot([0.0, 0.5])
                        .no_decoration()
                        .draw_background(false)
                        .no_inputs()
                        .always_auto_resize(true)
                        .begin()
                    {
                        ui.text_colored(color.as_f32(), &grenade.name);
                        ui.text_colored(color.as_f32(), &grenade.description);
                        let grenade_display_names = grenade
                            .grenade_types
                            .iter()
                            .map(|value| value.display_name())
                            .collect::<Vec<_>>();

                        ui.text_colored(color.as_f32(), grenade_display_names.join(", "));
                    }
                }
            }
        }

        Ok(())
    }
}