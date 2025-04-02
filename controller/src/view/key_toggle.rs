use std::time::Instant;

use crate::{
    settings::{
        HotKey,
        KeyToggleMode,
    },
    KeyboardInput,
};

pub struct KeyToggle {
    pub last_state_changed: Instant,
    pub enabled: bool,
}

impl KeyToggle {
    pub fn new() -> Self {
        Self {
            enabled: false,
            last_state_changed: Instant::now(),
        }
    }

    pub fn update(
        &mut self,
        mode: &KeyToggleMode,
        input: &dyn KeyboardInput,
        hotkey: &Option<HotKey>,
    ) -> bool {
        let new_state = match mode {
            KeyToggleMode::AlwaysOn => true,
            KeyToggleMode::Trigger | KeyToggleMode::TriggerInverted => {
                if let Some(hotkey) = hotkey {
                    input.is_key_down(hotkey.0) == (*mode == KeyToggleMode::Trigger)
                } else {
                    false
                }
            }
            KeyToggleMode::Toggle => {
                if let Some(hotkey) = hotkey {
                    if input.is_key_pressed(hotkey.0, false) {
                        if self.last_state_changed.elapsed().as_millis() > 250 {
                            self.last_state_changed = Instant::now();
                            !self.enabled
                        } else {
                            /* sometimes is_key_pressed with repeating set to false still triggers a few times */
                            self.enabled
                        }
                    } else {
                        self.enabled
                    }
                } else {
                    false
                }
            }
            KeyToggleMode::Off => false,
        };

        if self.enabled == new_state {
            /* no state change */
            return false;
        }

        self.enabled = new_state;
        true
    }
}
