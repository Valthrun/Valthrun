use cs2::{
    LocalCameraControllerTarget,
    SpectatorList,
};
use overlay::UnicodeTextRenderer;

use super::Enhancement;
use crate::{
    settings::AppSettings,
    utils::UnicodeTextWithShadowUi,
};

pub struct SpectatorsListIndicator;
impl SpectatorsListIndicator {
    pub fn new() -> Self {
        Self
    }
}

impl Enhancement for SpectatorsListIndicator {
    fn update(&mut self, _ctx: &crate::UpdateContext) -> anyhow::Result<()> {
        Ok(())
    }

    fn render(
        &self,
        states: &utils_state::StateRegistry,
        ui: &imgui::Ui,
        unicode_text: &UnicodeTextRenderer,
    ) -> anyhow::Result<()> {
        let settings = states.resolve::<AppSettings>(())?;
        if !settings.spectators_list {
            return Ok(());
        }

        let view_target = states.resolve::<LocalCameraControllerTarget>(())?;
        let target_entity_id = match &view_target.target_entity_id {
            Some(value) => *value,
            None => return Ok(()),
        };
        let spectators = states.resolve::<SpectatorList>(target_entity_id)?;

        let group = ui.begin_group();

        let line_count = spectators.spectators.iter().count();
        let text_height = ui.text_line_height_with_spacing() * line_count as f32;

        let offset_x = ui.io().display_size[0] * 0.01;
        let offset_y = (ui.io().display_size[1] - text_height) * 0.5;
        let mut offset_y = offset_y;

        for spectator in &spectators.spectators {
            ui.set_cursor_pos([offset_x, offset_y]);
            ui.unicode_text_with_shadow(unicode_text, &spectator.spectator_name);
            offset_y += ui.text_line_height_with_spacing();
        }

        group.end();
        Ok(())
    }
}
