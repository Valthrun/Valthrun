#[warn(unused_variables)]

use core::f32;

use cs2::{BoneFlags, CEntityIdentityEx, CS2Model, ClassNameCache, LocalCameraControllerTarget, PlayerPawnState, StateCS2Memory, StateEntityList, StateLocalPlayerController, StatePawnInfo, StatePawnModelInfo};
use overlay::UnicodeTextRenderer;
use valthrun_kernel_interface::MouseState;

use super::Enhancement;
use crate::settings::AppSettings;
use crate::view::{KeyToggle, ViewController};
use cs2_schema_generated::cs2::client::C_BaseEntity;
use nalgebra::Vector3;
use obfstr::obfstr;
use std::time::Instant;

pub struct Aimbot {
    aimbot_toggle: KeyToggle,            // Key toggle for enabling/disabling the aimbot
    aimbot_fov: f32,                     // Field of view for target acquisition
    aimbot_smooth: f32,                  // Speed at which the aim moves
    aimbot_is_active: bool,              // Indicates if the aimbot is currently active
    aimbot_last_mouse_move: Instant,     // Timestamp of the last mouse movement
    aimbot_current_target: Option<[f32; 3]>, // Current target coordinates (x, y, z)
    aimbot_is_mouse_pressed: bool,       // Indicates if the mouse button is being pressed
    aimbot_aim_bone: String,             // The specific bone being targeted (e.g., "head", "chest")

    // Advanced properties
    aimbot_team_check: bool,             // Checks if the target is on the same team
    aimbot_view_fov: bool,               // Only targets within the aimbot's field of view
    aimbot_flash_alpha: f32,             // Threshold to ignore targets affected by flashbangs
    aimbot_ignore_flash: bool,           // Flag to enable/disable ignoring flash effects
    aimbot_visibility_check: bool,       // Ensures the target is visible (not behind obstacles)
}

impl Aimbot {
    pub fn new() -> Self {
        println!("Aimbot::new called");
        Aimbot {
            aimbot_toggle: KeyToggle::new(),
            aimbot_fov: 100.0,
            aimbot_smooth: 1.0,
            aimbot_is_active: false,
            aimbot_last_mouse_move: Instant::now(),
            aimbot_current_target: None,
            aimbot_is_mouse_pressed: false,
            aimbot_aim_bone: "head".to_string(),
            aimbot_team_check: true,
            aimbot_view_fov: true,
            aimbot_flash_alpha: 255.0,
            aimbot_ignore_flash: false,
            aimbot_visibility_check: true,
        }
    }

    fn world_to_screen(&self, view: &ViewController, world_position: &Vector3<f32>) -> Option<[f32; 2]> {
        view.world_to_screen(world_position, true).map(|vec| [vec.x, vec.y])
    }

    fn find_best_target(&mut self, ctx: &crate::UpdateContext) -> Option<[f32; 2]> {
        if self.aimbot_is_mouse_pressed && self.aimbot_current_target.is_some() {
            return self.aimbot_current_target.map(|pos| [pos[0], pos[1]]);
        }

        let memory = ctx.states.resolve::<StateCS2Memory>(()).ok()?;
        let entities = ctx.states.resolve::<StateEntityList>(()).ok()?;
        let class_name_cache = ctx.states.resolve::<ClassNameCache>(()).ok()?;
        let local_controller = ctx.states.resolve::<StateLocalPlayerController>(()).ok()?;
        let local_pawn_handle = local_controller.instance.value_reference(memory.view_arc())?.m_hPlayerPawn().ok()?;
        let local_pawn = entities.entity_from_handle(&local_pawn_handle)?.value_reference(memory.view_arc())?;

        let view = ctx.states.resolve::<ViewController>(()).ok()?;
        let local_player_position = view.get_camera_world_position().unwrap_or(Vector3::new(0.0, 0.0, 0.0));
        let crosshair_pos = [view.screen_bounds.x / 2.0, view.screen_bounds.y / 2.0];
        let mut best_target: Option<[f32; 2]> = None;
        let mut lowest_distance = f32::MAX;

        let view_target = ctx.states.resolve::<LocalCameraControllerTarget>(()).ok()?;
        let view_target_entity_id = match &view_target.target_entity_id {
            Some(value) => *value,
            None => return None,
        };

        const UNITS_TO_METERS: f32 = 0.01905;


        // Adjust the entity class check logic:
        for entity_identity in entities.entities() {
            if entity_identity.handle::<()>().ok()?.get_entity_index() == view_target_entity_id {
                continue;
            }

            let entity_class = class_name_cache.lookup(&entity_identity.entity_class_info().ok()?).ok()?;
            if !entity_class
                .map(|name| *name == "C_CSPlayerPawn")
                .unwrap_or(false)
            {
                /* entity is not a player pawn */
                continue;
            }

            let pawn_info = ctx.states.resolve::<StatePawnInfo>(entity_identity.handle().ok()?).ok()?;

            let pawn_state = ctx
                .states
                .resolve::<PlayerPawnState>(entity_identity.handle().ok()?).ok()?;
            if *pawn_state != PlayerPawnState::Alive {
                continue;
            }
            let pawn_model = ctx
                .states
                .resolve::<StatePawnModelInfo>(entity_identity.handle().ok()?).ok()?;

            if self.aimbot_team_check && local_pawn.m_iTeamNum().unwrap_or(0) == pawn_info.team_id {
                println!("Aimbot: Skipping target due to team check");
                continue;
            }

            // Calculate distance and perform screen transformation
            let distance = (pawn_info.position - local_player_position).norm() * UNITS_TO_METERS;
            if distance < 2.0 {
                continue;
            }

            let entry_model = ctx.states.resolve::<CS2Model>(pawn_model.model_address).ok()?;
            for (bone, state) in entry_model.bones.iter().zip(pawn_model.bone_states.iter()) {
                if (bone.flags & BoneFlags::FlagHitbox as u32) == 0 {
                    continue;
                }

                // Check if the bone matches the target bone for aiming
                if bone.name.to_lowercase().contains(&self.aimbot_aim_bone) {
                    // Convert the bone's world position to screen space
                    if let Some(screen_position) = self.world_to_screen(&view, &state.position) {
                        let dx = screen_position[0] - crosshair_pos[0];
                        let dy = screen_position[1] - crosshair_pos[1];
                        let distance_from_crosshair = (dx * dx + dy * dy).sqrt();
                        let angle_to_target = distance_from_crosshair.atan2(view.screen_bounds.x / 2.0).to_degrees();

                        // Check if the target is within the FOV and is the closest so far
                        if angle_to_target <= self.aimbot_fov / 2.0 && distance_from_crosshair < lowest_distance {
                            lowest_distance = distance_from_crosshair;
                            best_target = Some(screen_position);
                        }
                    }
                }
            }
        }

        if self.aimbot_is_mouse_pressed {
            self.aimbot_current_target = best_target.map(|screen| [screen[0], screen[1], 0.0]);
        }

        best_target
    }

    fn aim_at_target(&self, ctx: &crate::UpdateContext, target_screen_position: [f32; 2]) -> anyhow::Result<bool> {
        let view = ctx.states.resolve::<ViewController>(())?;
        let crosshair_pos = [view.screen_bounds.x / 2.0, view.screen_bounds.y / 2.0];
        let adjustment = [
            (target_screen_position[0] - crosshair_pos[0]) / self.aimbot_smooth,
            (target_screen_position[1] - crosshair_pos[1]) / self.aimbot_smooth,
        ];
        ctx.cs2.send_mouse_state(&[MouseState {
            last_x: adjustment[0] as i32,
            last_y: adjustment[1] as i32,
            ..Default::default()
        }])?;
        Ok(true)
    }

    pub fn on_mouse_pressed(&mut self) {
        self.aimbot_is_mouse_pressed = true;
    }

    pub fn on_mouse_released(&mut self) {
        self.aimbot_is_mouse_pressed = false;
        self.aimbot_current_target = None;
    }
}

impl Enhancement for Aimbot {
    fn update(&mut self, ctx: &crate::UpdateContext) -> anyhow::Result<()> {
        let settings = ctx.states.resolve::<AppSettings>(())?;
        self.aimbot_fov = settings.aimbot_fov;
        self.aimbot_smooth = settings.aimbot_smooth;
        self.aimbot_aim_bone = settings.aimbot_aim_bone.clone();
        self.aimbot_team_check = settings.aimbot_team_check;
        self.aimbot_ignore_flash = settings.aimbot_ignore_flash;

        if self.aimbot_toggle.update(&settings.aimbot_mode, ctx.input, &settings.aimbot_key) {
            ctx.cs2.add_metrics_record(
                obfstr!("feature-aimbot-toggle"),
                &format!("enabled: {}, mode: {:?}", self.aimbot_toggle.enabled, settings.aimbot_mode),
            );
        } else {
            ctx.cs2.add_metrics_record(
                obfstr!("feature-aimbot-toggle"),
                &format!("enabled: {}, mode: {:?}", self.aimbot_toggle.enabled, settings.aimbot_mode),
            );
        }

        if self.aimbot_toggle.enabled {
            if let Some(target_screen_position) = self.find_best_target(ctx) {
                self.aim_at_target(ctx, target_screen_position)?;
            } else {}
        } else {}

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
        let draw_list = ui.get_window_draw_list();
        let cursor_pos = [view.screen_bounds.x / 2.0, view.screen_bounds.y / 2.0];

        fn fov_to_radius(fov: f32, screen_width: f32) -> f32 {
            let fov_radians = fov.to_radians();
            let half_fov = fov_radians / 2.0;
            (screen_width / 2.0) * half_fov.tan()
        }

        if settings.aimbot_view_fov {
            draw_list
                .add_circle(
                    cursor_pos,
                    fov_to_radius(settings.aimbot_fov, view.screen_bounds.x),
                    (1.0, 1.0, 1.0, 1.0),
                )
                .filled(false)
                .build();
        }
        Ok(())
    }
}
