use std::time::{Duration, Instant};
use anyhow::Context;
use cs2::{EntitySystem, LocalCameraControllerTarget, CEntityIdentityEx};
use cs2_schema_generated::cs2::client::C_CSPlayerPawn;
use nalgebra::Vector3;
use obfstr::obfstr;
use cs2::BoneFlags;
use cs2::CS2Model;
use cs2::ClassNameCache;

use crate::settings::AppSettings;
use crate::view::{KeyToggle, LocalCrosshair, ViewController};
use crate::UpdateContext;
use super::Enhancement;
use crate::enhancements::mouse_controllers::MouseController;
use std::thread::sleep;

pub struct Aimbot {
    toggle: KeyToggle,
    target_bone: String,
    fov: f32,
    aim_speed: f32,
    is_active: bool,
    last_mouse_move: Instant, // To track the time for constant mouse down movement
}

impl Aimbot {
    pub fn new() -> Self {
        Self {
            toggle: KeyToggle::new(),
            target_bone: "head".to_string(),
            fov: 5.0,
            aim_speed: 1.5,
            is_active: false,
            last_mouse_move: Instant::now(), // Initialize the timer
        }
    }

    fn world_to_screen(&self, view: &ViewController, world_position: &Vector3<f32>) -> Option<[f32; 2]> {
        view.world_to_screen(world_position, true)
    }

    fn move_mouse_down(&mut self, mouse: &MouseController) {
        if self.last_mouse_move.elapsed() >= Duration::from_millis(100) {
            mouse.move_mouse_down(5);
            self.last_mouse_move = Instant::now();
        }
    }

    fn find_best_target(&self, ctx: &UpdateContext) -> Option<[f32; 2]> {

        best_target
    }

    fn aim_at_target(&self, ctx: &UpdateContext, target_screen_position: [f32; 2]) {
        let view = ctx.states.resolve::<ViewController>(()).unwrap();
        let crosshair_pos = [view.screen_bounds.x / 2.0, view.screen_bounds.y / 2.0];
        let aim_adjustment = [
            (target_screen_position[0] - crosshair_pos[0]) / self.aim_speed,
            (target_screen_position[1] - crosshair_pos[1]) / self.aim_speed,
        ];

        let mouse = MouseController::new();
        mouse.move_mouse(aim_adjustment[0] as i32, aim_adjustment[1] as i32); // Move the mouse accordingly
    }
}

impl Enhancement for Aimbot {
    fn update(&mut self, ctx: &UpdateContext) -> anyhow::Result<()> {
        let settings = ctx.states.resolve::<AppSettings>(())?;
        
        // Check if the constant mouse down movement is enabled
        if settings.enable_constant_mouse_down {
            let mouse = MouseController::new();
            self.move_mouse_down(&mouse);
        }

        // Other aimbot logic
        if self.toggle.update(&settings.aimbot_mode, ctx.input, &settings.key_aimbot) {
            ctx.cs2.add_metrics_record(
                obfstr!("feature-aimbot-toggle"),
                &format!("enabled: {}, mode: {:?}", self.toggle.enabled, settings.aimbot_mode),
            );
        }

        if self.toggle.enabled {
            if let Some(target_screen_position) = self.find_best_target(ctx) {
                self.aim_at_target(ctx, target_screen_position);
            }
        }

        Ok(())
    }

    fn render(&self, _states: &utils_state::StateRegistry, _ui: &imgui::Ui) -> anyhow::Result<()> {
        Ok(())
    }
}
