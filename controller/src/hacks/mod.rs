use crate::settings::AppSettings;

pub trait Hack {
    fn update(&mut self, ctx: &UpdateContext) -> anyhow::Result<()>;
    fn render(&self, settings: &AppSettings, ui: &imgui::Ui, view: &ViewController);
    fn render_debug_window(&mut self, _settings: &mut AppSettings, _ui: &imgui::Ui) {}
}

mod bomb;
pub use bomb::*;

mod player;
pub use player::*;

mod trigger;
pub use trigger::*;

use crate::{UpdateContext, view::ViewController};