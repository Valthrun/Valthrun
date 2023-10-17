use std::time::Instant;

use anyhow::Context;
use cs2_schema_generated::{
    cs2::client::C_CSPlayerPawn,
    EntityHandle,
};
use rand::{
    distributions::Uniform,
    prelude::Distribution,
};
use valthrun_kernel_interface::MouseState;

use super::Enhancement;
use crate::{
    settings::AppSettings,
    view::{
        LocalCrosshair,
        ViewController,
    },
    UpdateContext,
};

enum TriggerState {
    Idle,
    Pending { delay: u32, timestamp: Instant },
    Active,
}

pub struct TriggerBot {
    state: TriggerState,
    trigger_active: bool,
    crosshair: LocalCrosshair,
}

impl TriggerBot {
    pub fn new(crosshair: LocalCrosshair) -> Self {
        Self {
            state: TriggerState::Idle,
            trigger_active: false,

            crosshair,
        }
    }

    fn should_be_active(&self, ctx: &UpdateContext) -> anyhow::Result<bool> {
        let target = match self.crosshair.current_target() {
            Some(target) => target,
            None => return Ok(false),
        };

        if !target
            .entity_type
            .as_ref()
            .map(|t| t == "C_CSPlayerPawn")
            .unwrap_or(false)
        {
            return Ok(false);
        }

        if ctx.settings.trigger_bot_team_check {
            let crosshair_entity = ctx
                .cs2_entities
                .get_by_handle(&EntityHandle::<C_CSPlayerPawn>::from_index(
                    target.entity_id,
                ))?
                .context("missing crosshair player pawn")?
                .entity()?
                .read_schema()?;

            let local_player_controller = ctx.cs2_entities.get_local_player_controller()?;
            if local_player_controller.is_null()? {
                return Ok(false);
            }

            let local_player_controller = local_player_controller.reference_schema()?;

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
        let should_be_active: bool = if ctx.settings.hold_enable_trigger_bot {
            self.crosshair.update(ctx)?;
            self.should_be_active(ctx)?
        } else {
            if let Some(key) = &ctx.settings.key_trigger_bot {
                if ctx.input.is_key_down(key.0) {
                    self.crosshair.update(ctx)?;
                    self.should_be_active(ctx)?
                } else {
                    false
                }
            } else {
                false
            }
        };

        loop {
            match &self.state {
                TriggerState::Idle => {
                    if !should_be_active {
                        /* nothing changed */
                        break;
                    }

                    let delay_min = ctx
                        .settings
                        .trigger_bot_delay_min
                        .min(ctx.settings.trigger_bot_delay_max);
                    let delay_max = ctx
                        .settings
                        .trigger_bot_delay_min
                        .max(ctx.settings.trigger_bot_delay_max);
                    let selected_delay = if delay_max == delay_min {
                        delay_min
                    } else {
                        let dist = Uniform::new_inclusive(delay_min, delay_max);
                        dist.sample(&mut rand::thread_rng())
                    };

                    log::trace!(
                        "Setting trigger bot into pending mode with a delay of {}ms",
                        selected_delay
                    );
                    self.state = TriggerState::Pending {
                        delay: selected_delay,
                        timestamp: Instant::now(),
                    };
                }
                TriggerState::Pending { delay, timestamp } => {
                    let time_elapsed = timestamp.elapsed().as_millis();
                    if time_elapsed < *delay as u128 {
                        /* still waiting to be activated */
                        break;
                    }

                    if ctx.settings.trigger_bot_check_target_after_delay && !should_be_active {
                        self.state = TriggerState::Idle;
                    } else {
                        self.state = TriggerState::Active;
                    }
                    /* regardsless of the next state, we always need to execute the current action */
                    break;
                }
                TriggerState::Active => {
                    if should_be_active {
                        /* nothing changed */
                        break;
                    }

                    self.state = TriggerState::Idle;
                }
            }
        }

        let should_be_active = matches!(self.state, TriggerState::Active);
        if should_be_active != self.trigger_active {
            self.trigger_active = should_be_active;

            let mut state = MouseState {
                ..Default::default()
            };
            state.buttons[0] = Some(self.trigger_active);
            ctx.cs2.send_mouse_state(&[state])?;
            log::trace!("Setting shoot state to {}", self.trigger_active);
        }

        Ok(())
    }

    fn render(&self, _settings: &AppSettings, _ui: &imgui::Ui, _view: &ViewController) {
        /* We have nothing to render */
    }
}
