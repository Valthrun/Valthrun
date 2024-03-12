use anyhow::Context;
use cs2::{
    EntitySystem,
    Globals,
};
use valthrun_kernel_interface::MouseState;

use super::Enhancement;
use crate::settings::AppSettings;

pub struct AntiAimPunsh {
    mouse_sensitivity: f32,

    mouse_adjustment_x: i32,
    mouse_adjustment_y: i32,

    last_tick_base: u32,
}

impl AntiAimPunsh {
    pub fn new() -> Self {
        Self {
            mouse_sensitivity: 0.8,

            mouse_adjustment_x: 0,
            mouse_adjustment_y: 0,

            last_tick_base: 0,
        }
    }
}

impl Enhancement for AntiAimPunsh {
    fn update(&mut self, ctx: &crate::UpdateContext) -> anyhow::Result<()> {
        let settings = ctx.states.resolve::<AppSettings>(())?;
        if !settings.aim_assist_recoil {
            return Ok(());
        }

        let entities = ctx.states.resolve::<EntitySystem>(())?;
        let local_controller = entities.get_local_player_controller()?;
        if local_controller.is_null()? {
            return Ok(());
        }

        let local_pawn = entities
            .get_by_handle(&local_controller.reference_schema()?.m_hPlayerPawn()?)?
            .context("missing local player pawn")?
            .entity()?
            .read_schema()?;

        if local_pawn.m_iShotsFired()? <= 1 {
            return Ok(());
        }

        let globals = ctx.states.resolve::<Globals>(())?;
        let current_tick = globals.frame_count_2()?;

        let punch_angle = nalgebra::Vector4::from_row_slice(&local_pawn.m_aimPunchAngle()?);
        let punch_vel = nalgebra::Vector4::from_row_slice(&local_pawn.m_aimPunchAngleVel()?);

        let mut punch_base = local_pawn.m_aimPunchTickBase()? as u32;
        if punch_base > current_tick {
            punch_base = current_tick;
        }
        let punch_elapsed = (current_tick - punch_base) as f32;

        let ltime = 20.0;
        let xpunch_elapsed = punch_elapsed;
        let total_punch_angle = if xpunch_elapsed < ltime {
            (punch_angle + punch_vel * xpunch_elapsed / 128.0) * (ltime - xpunch_elapsed) / ltime
        } else {
            nalgebra::Vector4::<f32>::zeros()
        };

        let deg_one = settings.mouse_x_360 as f32 / 360.0;
        let target_mouse_y = (total_punch_angle.x * deg_one * -2.25).round() as i32;
        let delta_mouse_y = target_mouse_y - self.mouse_adjustment_y;
        self.mouse_adjustment_y = target_mouse_y;

        let target_mouse_x = (total_punch_angle.y * deg_one * 2.0).round() as i32;
        let delta_mouse_x = target_mouse_x - self.mouse_adjustment_x;
        self.mouse_adjustment_x = target_mouse_x;

        if delta_mouse_y != 0 || delta_mouse_x != 0 {
            ctx.cs2.send_mouse_state(&[MouseState {
                last_y: delta_mouse_y,
                last_x: delta_mouse_x,
                ..Default::default()
            }])?;
        }

        // self.last_tick_base = punch_base;
        // log::debug!("X: {:?} | {:?} | {} ({}) | {} ({}) | {} ({})", punch_vel, total_punch_angle, punch_base, current_tick - punch_base, target_mouse_x, delta_mouse_x, target_mouse_y, delta_mouse_y);
        Ok(())
    }

    fn render(&self, _states: &utils_state::StateRegistry, _ui: &imgui::Ui) -> anyhow::Result<()> {
        Ok(())
    }
}
