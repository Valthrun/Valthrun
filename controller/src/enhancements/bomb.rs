use std::ffi::CStr;

use anyhow::Context;
use cs2::CEntityIdentityEx;
use cs2_schema_generated::cs2::client::C_PlantedC4;
use obfstr::obfstr;

use crate::UpdateContext;

use super::Enhancement;


#[derive(Debug)]
pub struct BombDefuser {
    /// Totoal time remaining for a successfull bomb defuse
    pub time_remaining: f32,

    /// The defusers player name
    pub player_name: String,
}

#[derive(Debug)]
pub enum BombState {
    /// Bomb hasn't been planted
    Unset,

    /// Bomb is currently actively ticking
    Active { 
        /// Planted bomb site
        /// 0 = A
        /// 1 = B
        bomb_site: u8,

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
    bomb_state: BombState,
}

impl BombInfo {
    pub fn new() -> Self {
        Self { bomb_state: BombState::Unset }
    }

    fn read_state(&self, ctx: &UpdateContext) -> anyhow::Result<BombState> {
        let entities = ctx.cs2_entities.all_identities()
            .with_context(|| obfstr!("failed to read entity list").to_string())?;

        for entity in entities.iter() {
            let vtable = entity.entity_vtable()?.read_schema()?.address()?;
            let class_name = ctx.class_name_cache.lookup(vtable).context("class name")?;
            if !(*class_name).as_ref().map(|name| name == "C_PlantedC4").unwrap_or(false) {
                /* Entity isn't the bomb. */
                continue;
            }

            let bomb = entity.entity_ptr::<C_PlantedC4>()?.read_schema().context("bomb schame")?;
            if !bomb.m_bC4Activated()? {
                /* This bomb hasn't been activated (yet) */
                continue;
            }

            if bomb.m_bBombDefused()? {
                return Ok(BombState::Defused);
            }

            let time_blow = bomb.m_flC4Blow()?.m_Value()?;
            let bomb_site = bomb.m_nBombSite()? as u8;

            if time_blow <= ctx.globals.time_2()? {
                return Ok(BombState::Detonated);
            }

            let is_defusing = bomb.m_bBeingDefused()?;
            let defusing = if is_defusing {
                let time_defuse = bomb.m_flDefuseCountDown()?.m_Value()?;

                let handle_defuser = bomb.m_hBombDefuser()?;
                let defuser = ctx.cs2_entities.get_by_handle(&handle_defuser)?
                    .with_context(|| obfstr!("missing bomb defuser player pawn").to_string())?
                    .reference_schema()?;

                let defuser_controller = defuser.m_hController()?;
                let defuser_controller = ctx.cs2_entities.get_by_handle(&defuser_controller)?
                    .with_context(|| obfstr!("missing bomb defuser controller").to_string())?
                    .reference_schema()?;
                    
                let defuser_name = CStr::from_bytes_until_nul(&defuser_controller.m_iszPlayerName()?)
                    .ok()
                    .map(CStr::to_string_lossy)
                    .unwrap_or("Name Error".into())
                    .to_string();

                Some(BombDefuser{ 
                    time_remaining: time_defuse - ctx.globals.time_2()?,
                    player_name: defuser_name
                })
            } else {
                None
            };

            return Ok(BombState::Active { 
                bomb_site, time_detonation: time_blow - ctx.globals.time_2()?, 
                defuse: defusing
            });
        }

        return Ok(BombState::Unset);
    }
}

impl Enhancement for BombInfo {
    fn update(&mut self, ctx: &crate::UpdateContext) -> anyhow::Result<()> {
        if !ctx.settings.bomb_timer {
            return Ok(());
        }

        self.bomb_state = self.read_state(ctx)?;
        Ok(())
    }

    fn render(&self, settings: &crate::settings::AppSettings, ui: &imgui::Ui, _view: &crate::view::ViewController) {
        if !settings.bomb_timer {
            return;
        }

        let group = ui.begin_group();

        let line_height = ui.text_line_height_with_spacing();
        ui.set_cursor_pos([ 10.0, ui.window_size()[1] * 0.95 - line_height * 5.0 ]); // ui.frame_height() - line_height * 5.0 

        match &self.bomb_state {
            BombState::Unset => {},
            BombState::Active { bomb_site, time_detonation, defuse } => {
                ui.text(&format!("Bomb planted on {}", if *bomb_site == 0 { "A" } else { "B" }));
                ui.text(&format!("Damage:"));
                ui.same_line();
                ui.text_colored([ 0.0, 0.0, 0.0, 0.0 ], "???");
                ui.text(&format!("Time: {:.3}", time_detonation));
                if let Some(defuse) = defuse.as_ref() {
                    let color = if defuse.time_remaining > *time_detonation {
                        [ 0.79, 0.11, 0.11, 1.0 ]
                    } else {
                        [ 0.11, 0.79, 0.26, 1.0 ]
                    };

                    ui.text_colored(color, &format!("Defused in {:.3} by {}", defuse.time_remaining, defuse.player_name));
                } else {
                    ui.text("Not defusing");
                }
            },
            BombState::Defused => {
                ui.text("Bomb has been defused");
            },
            BombState::Detonated => {
                ui.text("Bomb has been detonated");
            }
        }

        group.end();
    }
}