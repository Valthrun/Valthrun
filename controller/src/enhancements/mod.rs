use crate::settings::AppSettings;

pub trait Enhancement {
    fn update(&mut self, ctx: &UpdateContext) -> anyhow::Result<()>;
    fn update_settings(
        &mut self,
        _ui: &imgui::Ui,
        _settings: &mut AppSettings,
    ) -> anyhow::Result<bool> {
        Ok(false)
    }

    fn render(
        &self,
        states: &StateRegistry,
        ui: &imgui::Ui,
        unicode_text: &UnicodeTextRenderer,
    ) -> anyhow::Result<()>;
    fn render_debug_window(
        &mut self,
        _states: &StateRegistry,
        _ui: &imgui::Ui,
        _unicode_text: &UnicodeTextRenderer,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

mod bomb;
pub use bomb::*;

mod player;
use overlay::UnicodeTextRenderer;
pub use player::*;

mod trigger;
pub use trigger::*;

mod aimbot;
pub use aimbot::*;

mod spectators_list;
pub use spectators_list::*;

mod aim;
pub use aim::*;

mod grenade_helper;
pub use grenade_helper::*;
use utils_state::StateRegistry;

use crate::UpdateContext;
