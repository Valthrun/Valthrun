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

    fn render(&self, settings: &AppSettings, ui: &imgui::Ui, view: &ViewController);
    fn render_debug_window(&mut self, _settings: &mut AppSettings, _ui: &imgui::Ui) {}
}

mod bomb;
pub use bomb::*;

mod player;
pub use player::*;

mod trigger;
pub use trigger::*;

mod spectators_list;
pub use spectators_list::*;

mod aim;
pub use aim::*;

use crate::{
    view::ViewController,
    UpdateContext,
};
