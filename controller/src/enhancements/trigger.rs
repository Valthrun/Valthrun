use std::time::Instant;

use anyhow::Context;
use cs2_schema_generated::{cs2::client::C_CSPlayerPawn, EntityHandle};
use kinterface::MouseState;

use crate::{view::{LocalCrosshair, ViewController}, UpdateContext, settings::AppSettings};

use super::Enhancement;

pub struct CrosshairTarget {
    pub entity_id: u32,
    pub entity_type: Option<String>,
    pub timestamp: Instant
}

pub struct TriggerBot {
    active: bool,
    crosshair: LocalCrosshair,
}

impl TriggerBot {
    pub fn new(crosshair: LocalCrosshair) -> Self {
        Self {
            active: false,
            crosshair
        }
    }
    
    fn should_be_active(&self, ctx: &UpdateContext) -> anyhow::Result<bool> {
        let target = match self.crosshair.current_target() {
            Some(target) => target,
            None => return Ok(false)
        };
    
        if !target.entity_type.as_ref().map(|t| t == "C_CSPlayerPawn").unwrap_or(false) {
            return Ok(false);
        }
    
        if ctx.settings.trigger_bot_team_check {
            let crosshair_entity = ctx.cs2_entities.get_by_handle(
                &EntityHandle::<C_CSPlayerPawn>::from_index(target.entity_id)
            )?
                .context("missing crosshair player pawn")?
                .read_schema()?;
        
            let local_player_controller = ctx.cs2_entities.get_local_player_controller()?;
            if local_player_controller.is_null()? {
                return Ok(false);
            }

            let local_player_controller = local_player_controller
                .reference_schema()?;
        
            let target_player = crosshair_entity.as_schema::<C_CSPlayerPawn>()?;
            if target_player.m_iTeamNum()? == local_player_controller.m_iTeamNum()? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

impl Enhancement for TriggerBot {
    fn update(&mut self, ctx: &UpdateContext) -> anyhow::Result<()> {
        let should_be_active = if let Some(key) = &ctx.settings.key_trigger_bot {
            if ctx.input.is_key_down(key.0) {
                self.crosshair.update(ctx)?;
                self.should_be_active(ctx)?
            } else {
                false
            }
        } else {
            false
        };
        if should_be_active == self.active {
            return Ok(());
        }
        self.active = should_be_active;
    
        let mut state = MouseState{ ..Default::default() };
        state.buttons[0] = Some(self.active);
        ctx.cs2.send_mouse_state(&[ state ])?;
        log::trace!("Setting shoot state to {}", self.active);
        Ok(())
    }

    fn render(&self, _settings: &AppSettings, _ui: &imgui::Ui, _view: &ViewController) {
        /* We have nothing to render */
    }
}