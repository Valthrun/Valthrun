use std::{
    collections::BTreeMap,
    f32,
};

use anyhow::Context;
use cs2::{
    CEntityIdentityEx,
    ClassNameCache,
    CurrentMapState,
    EntitySystem,
    LocalCameraControllerTarget,
};
use cs2_schema_generated::{
    cs2::client::{
        C_BaseEntity,
        C_CSPlayerPawn,
        C_CSPlayerPawnBase,
    },
    EntityHandle,
};
use imgui::Condition;
use nalgebra::{
    Vector2,
    Vector3,
};
use obfstr::obfstr;
use serde::Deserialize;
use utils_state::StateRegistry;

use super::Enhancement;
use crate::{
    settings::AppSettings,
    view::ViewController,
};

#[derive(Debug, Deserialize)]
enum GranadeType {
    Smoke,
    Molotov,
    Flashbang,
    Explosive,
}

impl GranadeType {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Smoke => "Smoke",
            Self::Molotov => "Molotov",
            Self::Flashbang => "Flashbang",
            Self::Explosive => "Explosive",
        }
    }
}

struct GranadeState {
    granade_id: usize,

    display_opacity: f32,
    position_held: bool,
    angle_held: bool,
}

#[derive(Debug, Deserialize)]
struct GranadeInfo {
    pub map_name: String,
    pub granade_types: Vec<GranadeType>,

    pub name: String,
    pub description: String,

    /// The eye position of the player
    pub eye_position: [f32; 3],
    pub eye_direction: [f32; 3],
}

impl GranadeInfo {
    pub fn eye_position(&self) -> Vector3<f32> {
        Vector3::from_column_slice(&self.eye_position)
    }

    pub fn eye_direction(&self) -> Vector3<f32> {
        Vector3::from_column_slice(&self.eye_direction)
    }
}

pub struct GranadeHelper {
    granades: BTreeMap<usize, GranadeInfo>,
    granade_max_id: usize,
    granade_states: BTreeMap<usize, GranadeState>,

    eye_height: Vector3<f32>,
}

impl GranadeHelper {
    pub fn new() -> Self {
        let mut result = Self {
            granades: Default::default(),
            granade_states: Default::default(),
            granade_max_id: 0,

            eye_height: Vector3::new(0.0, 0.0, 64.093811),
        };

        let helper_data = include_bytes!("./granade_helper.json");
        let granade_info: Vec<GranadeInfo> = serde_json::from_slice(helper_data).unwrap();
        for granade in granade_info {
            result.register_granade_info(granade);
        }

        result
    }

    fn register_granade_info(&mut self, info: GranadeInfo) {
        let granade_id = self.granade_max_id + 1;
        self.granade_max_id += 1;

        self.granades.insert(granade_id, info);
        self.granade_states.insert(
            granade_id,
            GranadeState {
                granade_id,

                display_opacity: 0.0,
                angle_held: false,
                position_held: false,
            },
        );
    }
}

impl Enhancement for GranadeHelper {
    fn render_debug_window(
        &mut self,
        states: &StateRegistry,
        ui: &imgui::Ui,
    ) -> anyhow::Result<()> {
        let Some(_window) = ui
            .window("Granade Helper")
            .content_size([200.0, 100.0])
            .begin()
        else {
            return Ok(());
        };

        let entities = states.resolve::<EntitySystem>(())?;
        let view_target = states.resolve::<LocalCameraControllerTarget>(())?;
        let local_entity = entities
            .get_by_handle(&EntityHandle::<C_CSPlayerPawn>::from_index(
                view_target
                    .target_entity_id
                    .context("missing current entity")?,
            ))?
            .context("no local entity")?
            .entity()?
            .reference_schema()?;

        if ui.button("Add current##1") {
            let direction = Vector3::from_column_slice(&local_entity.m_angEyeAngles()?[0..3]);
            let player_position = Vector3::from_column_slice(
                &local_entity
                    .m_pGameSceneNode()?
                    .reference_schema()?
                    .m_vecAbsOrigin()?,
            );
            self.register_granade_info(GranadeInfo {
                map_name: "de_mirage".to_string(),
                granade_types: vec![],

                name: "Mid Window".to_string(),
                description: "Hold D, Jump + Left Click".to_string(),

                eye_position: (player_position + self.eye_height).into(),
                eye_direction: direction.into(),
            });

            log::debug!("{:?}", direction);
        }
        Ok(())
    }

    fn update(&mut self, ctx: &crate::UpdateContext) -> anyhow::Result<()> {
        let settings = ctx.states.resolve::<AppSettings>(())?;
        if !settings.granade_helper {
            return Ok(());
        }

        for state in self.granade_states.values_mut() {
            state.display_opacity = 0.0;
            state.angle_held = false;
            state.position_held = false;
        }

        let current_map = ctx.states.resolve::<CurrentMapState>(())?;
        let Some(current_map) = &current_map.current_map else {
            return Ok(());
        };

        let (local_position, local_direction) = {
            let entities = ctx.states.resolve::<EntitySystem>(())?;
            let class_name_cache = ctx.states.resolve::<ClassNameCache>(())?;

            let view_target = ctx.states.resolve::<LocalCameraControllerTarget>(())?;
            if view_target.target_entity_id.is_none() {
                /* We're currently not spectating an entity. Not showing granade helper. */
                return Ok(());
            }

            let local_entity_identity = entities
                .get_by_handle(&EntityHandle::<C_BaseEntity>::from_index(
                    view_target.target_entity_id.unwrap(),
                ))?
                .with_context(|| obfstr!("no local entity").to_string())?;

            let local_entity_class = class_name_cache
                .lookup(&local_entity_identity.entity_class_info()?)?
                .with_context(|| {
                    obfstr!("failed to resolve entity class for current entity").to_string()
                })?;
            if local_entity_class != "C_CSPlayerPawn" {
                /* We're currently not a CS player, hence no granade helper */
                return Ok(());
            }

            let local_entity = local_entity_identity
                .entity()?
                .cast::<C_CSPlayerPawnBase>()
                .read_schema()?;

            let local_position = Vector3::from_column_slice(
                &local_entity
                    .m_pGameSceneNode()?
                    .reference_schema()?
                    .m_vecAbsOrigin()?,
            );
            let local_direction = Vector3::from_column_slice(&local_entity.m_angEyeAngles()?[0..3]);

            (local_position, local_direction)
        };

        for state in self.granade_states.values_mut() {
            let Some(granade) = self.granades.get(&state.granade_id) else {
                continue;
            };

            if &granade.map_name != current_map {
                /* No need to update the rest */
                continue;
            }

            let dist_xy = granade
                .eye_position()
                .xy()
                .metric_distance(&Vector2::new(local_position.x, local_position.y));

            if dist_xy > settings.granade_helper_circle_distance {
                /* Granade spot is out of view distance */
                continue;
            }

            let fadein_distance = settings.granade_helper_circle_distance * 0.2;
            if dist_xy > settings.granade_helper_circle_distance - fadein_distance {
                state.display_opacity =
                    (settings.granade_helper_circle_distance - dist_xy) / fadein_distance;
            } else {
                state.display_opacity = 1.0;
            }

            state.position_held = dist_xy < settings.granade_helper_circle_radius
                && (granade.eye_position().z - local_position.z).abs() < 100.0;

            let direction_diff = (granade.eye_direction() - local_direction).abs();
            state.angle_held = direction_diff.x < settings.granade_helper_angle_threshold_pitch
                && direction_diff.y < settings.granade_helper_angle_threshold_yaw;
        }

        Ok(())
    }

    fn render(&self, states: &StateRegistry, ui: &imgui::Ui) -> anyhow::Result<()> {
        let view = states.resolve::<ViewController>(())?;
        let settings = states.resolve::<AppSettings>(())?;
        if !settings.granade_helper {
            return Ok(());
        }

        let draw_list = ui.get_window_draw_list();
        for state in self.granade_states.values() {
            if state.display_opacity <= 0.0 {
                continue;
            }

            let Some(granade) = self.granades.get(&state.granade_id) else {
                continue;
            };

            let mut color = if state.position_held {
                settings.granade_helper_color_position_active
            } else {
                settings.granade_helper_color_position
            };
            color.set_alpha_f32(state.display_opacity);

            if let Some(body_position) =
                view.world_to_screen(&(granade.eye_position() - self.eye_height), false)
            {
                {
                    let mut points =
                        Vec::with_capacity(settings.granade_helper_circle_segments + 1);
                    for index in 0..=settings.granade_helper_circle_segments {
                        let offset = index as f32 * f32::consts::TAU
                            / settings.granade_helper_circle_segments as f32;

                        let point_3d = granade.eye_position()
                            + Vector3::new(
                                settings.granade_helper_circle_radius * offset.sin(),
                                settings.granade_helper_circle_radius * offset.cos(),
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
                    if granade.granade_types.is_empty() {
                        ui.text_colored(color.as_f32(), "All granades");
                    } else {
                        let granade_display_names = granade
                            .granade_types
                            .iter()
                            .map(|value| value.display_name())
                            .collect::<Vec<_>>();

                        ui.text_colored(color.as_f32(), granade_display_names.join(", "));
                    }

                    ui.set_cursor_pos([
                        body_position.x + 10.0,
                        body_position.y - text_height / 2.0 + ui.text_line_height(),
                    ]);
                    ui.text_colored(color.as_f32(), &granade.name);
                }
            }

            if state.position_held {
                let vec = Vector3::new(
                    (-granade.eye_direction().x * f32::consts::PI / 180.0).cos()
                        * (granade.eye_direction().y * f32::consts::PI / 180.0).cos(),
                    (-granade.eye_direction().x * f32::consts::PI / 180.0).cos()
                        * (granade.eye_direction().y * f32::consts::PI / 180.0).sin(),
                    (-granade.eye_direction().x * f32::consts::PI / 180.0).sin(),
                );

                if let Some(direction_indicator) =
                    view.world_to_screen(&(granade.eye_position() + 200.0 * vec), true)
                {
                    let color = if state.angle_held {
                        settings.granade_helper_color_angle_active
                    } else {
                        settings.granade_helper_color_angle
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
                        .window(format!("##granade_info_{}", state.granade_id))
                        .position(
                            [direction_indicator.x + 10.0, direction_indicator.y],
                            Condition::Always,
                        )
                        .position_pivot([0.0, 0.5])
                        .no_decoration()
                        .draw_background(true)
                        .begin()
                    {
                        ui.text_colored(color.as_f32(), &granade.name);
                        ui.text_colored(color.as_f32(), &granade.description);
                    }
                }
            }
        }

        Ok(())
    }
}
