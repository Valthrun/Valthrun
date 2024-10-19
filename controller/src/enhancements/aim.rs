use core::f32;

use anyhow::Context;
use cs2::{
    ConVar,
    EntitySystem,
};
use overlay::UnicodeTextRenderer;
use valthrun_kernel_interface::MouseState;

use super::Enhancement;
use crate::settings::AppSettings;

pub struct AntiAimPunsh {
    mouse_sensitivity: ConVar,

    mouse_adjustment_x: i32,
    mouse_adjustment_y: i32,

    punch_vec: Option<mint::Vector2<f32>>,
}

impl AntiAimPunsh {
    pub fn new(mouse_sensitivity: ConVar) -> Self {
        Self {
            mouse_sensitivity,

            mouse_adjustment_x: 0,
            mouse_adjustment_y: 0,

            punch_vec: None,
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
            self.mouse_adjustment_x = 0;
            self.mouse_adjustment_y = 0;
            return Ok(());
        }

        let mouse_sensitivity = self.mouse_sensitivity.fl_value()?;
        let punch_angle = nalgebra::Vector4::from_row_slice(&local_pawn.m_aimPunchAngle()?) * 2.0;

        let mouse_x = (punch_angle.y / (mouse_sensitivity * 0.022)).round() as i32;
        let mouse_y = (punch_angle.x / (mouse_sensitivity * 0.022)).round() as i32;

        let delta_x = mouse_x - self.mouse_adjustment_x;
        let delta_y = mouse_y - self.mouse_adjustment_y;

        if delta_x != 0 || delta_y != 0 {
            ctx.cs2.send_mouse_state(&[MouseState {
                last_y: -delta_y,
                last_x: delta_x,
                ..Default::default()
            }])?;

            self.mouse_adjustment_x = mouse_x;
            self.mouse_adjustment_y = mouse_y;
        }

        Ok(())
    }

    fn render(
        &self,
        _states: &utils_state::StateRegistry,
        _ui: &imgui::Ui,
        _unicode_text: &UnicodeTextRenderer,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
