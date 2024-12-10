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

    pub fn update_dual(
        &mut self,
        mode: &KeyToggleMode,
        input: &dyn KeyboardInput,
        primary_key: &Option<HotKey>,
        secondary_key: &Option<HotKey>,
    ) -> bool {
        let key_down = |key: &Option<HotKey>| key.as_ref().map_or(false, |k| input.is_key_down(k.0));
        let key_pressed = |key: &Option<HotKey>| key.as_ref().map_or(false, |k| input.is_key_pressed(k.0, false));
        let new_state = match mode {
            KeyToggleMode::AlwaysOn => true,
            KeyToggleMode::Trigger | KeyToggleMode::TriggerInverted => {
                (key_down(primary_key) || key_down(secondary_key))
                    == (*mode == KeyToggleMode::Trigger)
            }
            KeyToggleMode::Toggle => {
                if key_pressed(primary_key) || key_pressed(secondary_key) {
                    if self.last_state_changed.elapsed().as_millis() > 250 {
                        self.last_state_changed = Instant::now();
                        !self.enabled
                    } else {
                        self.enabled
                    }
                } else {
                    self.enabled
                }
            }
            KeyToggleMode::Off => false,
        };
        if self.enabled == new_state {
            return false;  // No state change
        }
        self.enabled = new_state;
        true
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
