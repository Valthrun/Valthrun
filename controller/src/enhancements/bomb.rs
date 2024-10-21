use cs2::{
    constants,
    PlantedC4,
    PlantedC4State,
};
use overlay::UnicodeTextRenderer;

use super::Enhancement;
use crate::{
    settings::AppSettings,
    utils::ImguiUiEx,
    view::ViewController,
};
pub struct BombInfoIndicator {}

impl BombInfoIndicator {
    pub fn new() -> Self {
        Self {}
    }
}

impl Enhancement for BombInfoIndicator {
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
        let view = states.resolve::<ViewController>(())?;
        if !settings.bomb_timer {
            return Ok(());
        }

        let view_world_position = match view.get_camera_world_position() {
            Some(view_world_position) => view_world_position,
            _ => return Ok(()),
        };

        let bomb_state = states.resolve::<PlantedC4>(())?;

        let group = ui.begin_group();

        let line_count = match &bomb_state.state {
            PlantedC4State::Active { .. } => 3,
            PlantedC4State::Defused
            | PlantedC4State::Detonated
            | PlantedC4State::NotPlanted { .. } => 2,
        };
        let text_height = ui.text_line_height_with_spacing() * line_count as f32;

        /* align to be on the right side after the players */
        let offset_x = ui.io().display_size[0] * 1730.0 / 2560.0;
        let offset_y = ui.io().display_size[1] * constants::PLAYER_AVATAR_TOP_OFFSET;
        let offset_y = offset_y
            + 0_f32
                .max((ui.io().display_size[1] * constants::PLAYER_AVATAR_SIZE - text_height) / 2.0);

        ui.set_cursor_pos([offset_x, offset_y]);

        match &bomb_state.state {
            PlantedC4State::Active {
                time_detonation,
                bomb_position,
            } => {
                let distance =
                    (*bomb_position - view_world_position).norm() * constants::UNITS_TO_METERS;
                ui.text(&format!(
                    "Bomb planted {}",
                    if bomb_state.bomb_site == 0 { "A" } else { "B" }
                ));
                ui.set_cursor_pos_x(offset_x);
                ui.text(&format!("Time: {:.3}", time_detonation));
                if let Some(defuser) = &bomb_state.defuser {
                    let color = if defuser.time_remaining > *time_detonation {
                        [0.79, 0.11, 0.11, 1.0]
                    } else {
                        [0.11, 0.79, 0.26, 1.0]
                    };

                    ui.set_cursor_pos_x(offset_x);
                    unicode_text.text_colored(
                        color,
                        &format!(
                            "Defused in {:.3} by {}",
                            defuser.time_remaining, defuser.player_name
                        ),
                    );
                } else {
                    ui.set_cursor_pos_x(offset_x);
                    ui.text("Not defusing");
                }
                if let Some(pos) = view.world_to_screen(bomb_position, false) {
                    let y_offset = 0.0;
                    let draw = ui.get_window_draw_list();
                    let text = "BOMB";
                    let [text_width, _] = ui.calc_text_size(&text);
                    let mut pos = pos.clone();
                    pos.x -= text_width / 2.0;
                    pos.y += y_offset;
                    draw.add_text(pos, [0.79, 0.11, 0.11, 1.0], text);
                }
                ui.set_cursor_pos_x(offset_x);
                if distance > constants::IS_SAFE {
                    ui.text("You're safe!")
                } else {
                    let test = constants::IS_SAFE - distance;
                    let text = format!("Back {:.0} m", test);
                    ui.text(text)
                };
            }
            PlantedC4State::Defused => {
                ui.set_cursor_pos_x(offset_x);
                ui.text("Bomb has been defused");
            }
            PlantedC4State::Detonated => {
                ui.set_cursor_pos_x(offset_x);
                ui.text("Bomb has been detonated");
            }
            PlantedC4State::NotPlanted {
                c4_owner_entity_index,
                bomb_position,
            } => {
                if *c4_owner_entity_index == constants::BOMB_DROPPED {
                    ui.set_cursor_pos_x(offset_x);
                    ui.text("Bomb is dropped.");
                    if let Some(pos) = view.world_to_screen(bomb_position, false) {
                        let y_offset = 0.0;
                        let draw = ui.get_window_draw_list();
                        let text = "BOMB";
                        let [text_width, _] = ui.calc_text_size(&text);
                        let mut pos = pos.clone();
                        pos.x -= text_width / 2.0;
                        pos.y += y_offset;
                        draw.add_text(pos, [0.79, 0.11, 0.11, 1.0], text);
                    }
                }
            }
        }

        group.end();
        Ok(())
    }
}
