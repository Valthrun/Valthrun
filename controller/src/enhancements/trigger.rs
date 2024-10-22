use std::time::Instant;

use anyhow::Context;
use cs2::{
    StateCS2Memory,
    StateEntityList,
    StateLocalPlayerController,
};
use cs2_schema_cutl::EntityHandle;
use cs2_schema_generated::cs2::client::{
    C_BaseEntity,
    C_CSPlayerPawn,
};
use obfstr::obfstr;
use overlay::UnicodeTextRenderer;
use rand::{
    distributions::Uniform,
    prelude::Distribution,
};
use utils_state::StateRegistry;
use valthrun_kernel_interface::MouseState;

use super::Enhancement;
use crate::{
    settings::AppSettings,
    view::{
        KeyToggle,
        StateLocalCrosshair,
    },
    UpdateContext,
};

enum TriggerState {
    Idle,
    Pending { delay: u32, timestamp: Instant },
    Sleep { delay: u32, timestamp: Instant },
    Active,
}

pub struct TriggerBot {
    toggle: KeyToggle,
    state: TriggerState,
    trigger_active: bool,
}

impl TriggerBot {
    pub fn new() -> Self {
        Self {
            toggle: KeyToggle::new(),
            state: TriggerState::Idle,
            trigger_active: false,
        }
    }

    fn should_be_active(&self, ctx: &UpdateContext) -> anyhow::Result<bool> {
        let settings = ctx.states.resolve::<AppSettings>(())?;
        let crosshair = ctx.states.resolve::<StateLocalCrosshair>(())?;
        let entities = ctx.states.resolve::<StateEntityList>(())?;
        let memory = ctx.states.resolve::<StateCS2Memory>(())?;

        let target = match crosshair.current_target() {
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

        if settings.trigger_bot_team_check {
            let crosshair_entity = entities
                .entity_from_handle(&EntityHandle::<dyn C_CSPlayerPawn>::from_index(
                    target.entity_id,
                ))
                .context("missing crosshair player pawn")?
                .value_reference(memory.view_arc())
                .context("entity nullptr")?;

            let local_player_controller = ctx.states.resolve::<StateLocalPlayerController>(())?;
            let Some(local_player_controller) = local_player_controller
                .instance
                .value_reference(memory.view_arc())
            else {
                return Ok(false);
            };

            if crosshair_entity.m_iTeamNum()? == local_player_controller.m_iTeamNum()? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

impl Enhancement for TriggerBot {
    fn update(&mut self, ctx: &UpdateContext) -> anyhow::Result<()> {
        let settings = ctx.states.resolve::<AppSettings>(())?;
        if self.toggle.update(
            &settings.trigger_bot_mode,
            ctx.input,
            &settings.key_trigger_bot,
        ) {
            ctx.cs2.add_metrics_record(
                obfstr!("feature-trigger-bot-toggle"),
                &format!(
                    "enabled: {}, mode: {:?}",
                    self.toggle.enabled, settings.trigger_bot_mode
                ),
            );
        }

        let should_shoot: bool = if self.toggle.enabled {
            self.should_be_active(ctx)?
        } else {
            false
        };

        loop {
            match &self.state {
                TriggerState::Idle => {
                    if !should_shoot {
                        /* nothing changed */
                        break;
                    }

                    let delay_min = settings
                        .trigger_bot_delay_min
                        .min(settings.trigger_bot_delay_max);
                    let delay_max = settings
                        .trigger_bot_delay_min
                        .max(settings.trigger_bot_delay_max);
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

                    if settings.trigger_bot_check_target_after_delay && !should_shoot {
                        self.state = TriggerState::Idle;
                    } else {
                        self.state = TriggerState::Active;
                    }
                    /* regardless of the next state, we always need to execute the current action */
                    break;
                }
                TriggerState::Sleep { delay, timestamp } => {
                    let time_elapsed = timestamp.elapsed().as_millis();
                    if time_elapsed < *delay as u128 {
                        /* still waiting to be activated */
                        break;
                    }
                    self.state = TriggerState::Idle;
                    break;
                }
                TriggerState::Active => {
                    if should_shoot {
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

            self.state = TriggerState::Sleep {
                delay: settings.trigger_bot_shot_duration,
                timestamp: Instant::now(),
            };
        }

        Ok(())
    }

    fn render(
        &self,
        _states: &StateRegistry,
        _ui: &imgui::Ui,
        _unicode_text: &UnicodeTextRenderer,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
