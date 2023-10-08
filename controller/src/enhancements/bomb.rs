use std::ffi::CStr;

use anyhow::Context;
use cs2::CEntityIdentityEx;
use cs2_schema_generated::cs2::client::C_PlantedC4;
use obfstr::obfstr;

use super::Enhancement;
use crate::{
    utils::ImguiUiEx,
    UpdateContext,
};

#[derive(Debug)]
pub struct BombDefuser {
    /// Totoal time remaining for a successfull bomb defuse
    pub time_remaining: f32,

    /// The defusers player name
    pub player_name: String,
}

pub struct C4Info {
    /// Planted bomb site
    /// 0 = A
    /// 1 = B
    bomb_site: u8,

    /// Current state of the C4
    state: C4State,
}

#[derive(Debug)]
pub enum C4State {
    /// Bomb is currently actively ticking
    Active {
        /// Time remaining (in seconds) until detonation
        time_detonation: f32,

        /// Current bomb defuser
        defuse: Option<BombDefuser>,
    },

    /// Bomb has detonated
    Detonated,

    /// Bomb has been defused
    Defused,
}

pub struct BombInfo {
    bomb_state: Option<C4Info>,
}

impl BombInfo {
    pub fn new() -> Self {
        Self { bomb_state: None }
    }

    fn read_state(&self, ctx: &UpdateContext) -> anyhow::Result<Option<C4Info>> {
        let entities = ctx.cs2_entities.all_identities();

        for entity_identity in entities.iter() {
            let class_name = ctx
                .class_name_cache
                .lookup(&entity_identity.entity_class_info()?)
                .context("class name")?;

            if !class_name
                .map(|name| name == "C_PlantedC4")
                .unwrap_or(false)
            {
                /* Entity isn't the bomb. */
                continue;
            }

            let bomb = entity_identity
                .entity_ptr::<C_PlantedC4>()?
                .read_schema()
                .context("bomb schame")?;
            if !bomb.m_bC4Activated()? {
                /* This bomb hasn't been activated (yet) */
                continue;
            }

            let bomb_site = bomb.m_nBombSite()? as u8;
            if bomb.m_bBombDefused()? {
                return Ok(Some(C4Info {
                    bomb_site,
                    state: C4State::Defused,
                }));
            }

            let time_blow = bomb.m_flC4Blow()?.m_Value()?;

            if time_blow <= ctx.globals.time_2()? {
                return Ok(Some(C4Info {
                    bomb_site,
                    state: C4State::Detonated,
                }));
            }

            let is_defusing = bomb.m_bBeingDefused()?;
            let defusing = if is_defusing {
                let time_defuse = bomb.m_flDefuseCountDown()?.m_Value()?;

                let handle_defuser = bomb.m_hBombDefuser()?;
                let defuser = ctx
                    .cs2_entities
                    .get_by_handle(&handle_defuser)?
                    .with_context(|| obfstr!("missing bomb defuser player pawn").to_string())?
                    .entity()?
                    .reference_schema()?;

                let defuser_controller = defuser.m_hController()?;
                let defuser_controller = ctx
                    .cs2_entities
                    .get_by_handle(&defuser_controller)?
                    .with_context(|| obfstr!("missing bomb defuser controller").to_string())?
                    .entity()?
                    .reference_schema()?;

                let defuser_name =
                    CStr::from_bytes_until_nul(&defuser_controller.m_iszPlayerName()?)
                        .ok()
                        .map(CStr::to_string_lossy)
                        .unwrap_or("Name Error".into())
                        .to_string();

                Some(BombDefuser {
                    time_remaining: time_defuse - ctx.globals.time_2()?,
                    player_name: defuser_name,
                })
            } else {
                None
            };

            return Ok(Some(C4Info {
                bomb_site,
                state: C4State::Active {
                    time_detonation: time_blow - ctx.globals.time_2()?,
                    defuse: defusing,
                },
            }));
        }

        return Ok(None);
    }
}

/// % of the screens height
const PLAYER_AVATAR_TOP_OFFSET: f32 = 0.004;

/// % of the screens height
const PLAYER_AVATAR_SIZE: f32 = 0.05;

impl Enhancement for BombInfo {
    fn update(&mut self, ctx: &crate::UpdateContext) -> anyhow::Result<()> {
        if !ctx.settings.bomb_timer {
            return Ok(());
        }

        self.bomb_state = self.read_state(ctx)?;
        Ok(())
    }

    fn render(
        &self,
        settings: &crate::settings::AppSettings,
        ui: &imgui::Ui,
        _view: &crate::view::ViewController,
    ) {
        if !settings.bomb_timer {
            return;
        }

        let bomb_info = match &self.bomb_state {
            Some(state) => state,
            None => return,
        };

        let group = ui.begin_group();

        let line_count = match &bomb_info.state {
            C4State::Active { .. } => 3,
            C4State::Defused | C4State::Detonated => 2,
        };
        let text_height = ui.text_line_height_with_spacing() * line_count as f32;

        /* align to be on the right side after the players */
        let offset_x = ui.io().display_size[0] * 1730.0 / 2560.0;
        let offset_y = ui.io().display_size[1] * PLAYER_AVATAR_TOP_OFFSET;
        let offset_y = offset_y
            + 0_f32.max((ui.io().display_size[1] * PLAYER_AVATAR_SIZE - text_height) / 2.0);

        ui.set_cursor_pos([offset_x, offset_y]);
        ui.text(&format!(
            "Bomb planted {}",
            if bomb_info.bomb_site == 0 { "A" } else { "B" }
        ));

        match &bomb_info.state {
            C4State::Active {
                time_detonation,
                defuse,
            } => {
                ui.set_cursor_pos_x(offset_x);
                ui.text(&format!("Time: {:.3}", time_detonation));
                if let Some(defuse) = defuse.as_ref() {
                    let color = if defuse.time_remaining > *time_detonation {
                        [0.79, 0.11, 0.11, 1.0]
                    } else {
                        [0.11, 0.79, 0.26, 1.0]
                    };

                    ui.set_cursor_pos_x(offset_x);
                    ui.text_colored(
                        color,
                        &format!(
                            "Defused in {:.3} by {}",
                            defuse.time_remaining, defuse.player_name
                        ),
                    );
                } else {
                    ui.set_cursor_pos_x(offset_x);
                    ui.text("Not defusing");
                }
            }
            C4State::Defused => {
                ui.set_cursor_pos_x(offset_x);
                ui.text("Bomb has been defused");
            }
            C4State::Detonated => {
                ui.set_cursor_pos_x(offset_x);
                ui.text("Bomb has been detonated");
            }
        }

        group.end();
    }
}
