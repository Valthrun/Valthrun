use std::ffi::CStr;

use anyhow::Context;
use cs2::CEntityIdentityEx;
use cs2_schema_generated::cs2::client::{
    CCSPlayer_ItemServices,
    C_CSGameRulesProxy,
    C_CSPlayerPawn,
    C_PlantedC4,
};
use imgui::Condition;
use mint::Vector4;
use obfstr::obfstr;

use super::Enhancement;
use crate::{
    RenderContext,
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

    /// Bomb has been dropped
    Dropped,
}

pub struct BombInfo {
    /// Number of Counter-Terrorists with kit
    num_kit: u8,

    /// Local Team Identifier
    local_team: u8,

    /// Current State of Bomb
    bomb_state: Option<C4Info>,
}

impl BombInfo {
    pub fn new() -> Self {
        Self {
            num_kit: 0,
            local_team: 0,
            bomb_state: None,
        }
    }

    fn read_state(&self, ctx: &UpdateContext) -> anyhow::Result<Option<C4Info>> {
        let entities = ctx.cs2_entities.all_identities();

        for entity_identity in entities.iter() {
            let class_name = ctx
                .class_name_cache
                .lookup(&entity_identity.entity_class_info()?)
                .context("class name")?;

            // Check if bomb is dropped
            if class_name
                .map(|name| name == "C_CSGameRulesProxy")
                .unwrap_or(false)
            {
                /* The bomb is dropped. */
                let rules_proxy = entity_identity
                    .entity_ptr::<C_CSGameRulesProxy>()?
                    .reference_schema()
                    .context("rules proxy missing")?;

                let game_rules = rules_proxy
                    .m_pGameRules()?
                    .reference_schema()
                    .context("game rules missing")?;
                ();

                if game_rules.m_bBombDropped().unwrap_or_default() {
                    return Ok(Some(C4Info {
                        bomb_site: 0,
                        state: C4State::Dropped,
                    }));
                }
            }

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

impl Enhancement for BombInfo {
    fn update(&mut self, ctx: &UpdateContext) -> anyhow::Result<()> {
        if !ctx.settings.bomb_timer {
            return Ok(());
        }

        for entity_identity in ctx.cs2_entities.all_identities() {
            let entity_class = ctx
                .class_name_cache
                .lookup(&entity_identity.entity_class_info()?)?;

            if !entity_class
                .map(|name| *name == "C_CSPlayerPawn")
                .unwrap_or(false)
            {
                /* entity is not a player pawn */
                continue;
            }

            let player_pawn = entity_identity
                .entity_ptr::<C_CSPlayerPawn>()?
                .reference_schema()
                .context("missing player pawn")?;

            let player_has_defuser = player_pawn
                .m_pItemServices()?
                .cast::<CCSPlayer_ItemServices>()
                .reference_schema()?
                .m_bHasDefuser()?;

            if player_has_defuser {
                self.num_kit += 1;
            }
        }

        let local_controller = ctx
            .cs2_entities
            .get_local_player_controller()?
            .try_reference_schema()
            .with_context(|| obfstr!("failed to read local player controller").to_string())?;

        let local_controller = match local_controller {
            Some(controller) => controller,
            None => {
                /* We're currently not connected */
                return Ok(());
            }
        };

        self.local_team = local_controller.m_iPendingTeamNum()?;
        self.bomb_state = self.read_state(ctx)?;
        Ok(())
    }

    fn render(&self, ctx: RenderContext) {
        if !ctx.settings.bomb_timer {
            return;
        }

        let bomb_info = match &self.bomb_state {
            Some(state) => state,
            None => return,
        };

        let show_bg = ctx.settings.bomb_timer_decor || ctx.app.settings_visible;
        let show_bg2 = show_bg && ctx.app.settings_visible;

        ctx.ui
            .window(obfstr!("Bomb Info"))
            .size([250.0, 125.0], Condition::Appearing)
            // Disable all window decorations.
            .resizable(show_bg2)
            .collapsible(show_bg2)
            .title_bar(show_bg2)
            .draw_background(show_bg)
            .movable(!show_bg2)
            .build(|| {
                // Common Colors
                let white = [1.0, 1.0, 1.0, 1.0]; // White

                let mut orange = [1.0, 0.61, 0.11, 1.0]; // Orange
                let mut green = [0.11, 0.79, 0.26, 1.0]; // Green
                let mut red = [0.79, 0.11, 0.11, 1.0]; // Red

                // Reset Colors
                if !ctx.settings.bomb_timer_color {
                    orange = white;
                    green = white;
                    red = white;
                }

                // Helper Function
                fn helper_text_color(
                    ctx: &RenderContext,
                    player_team: &u8,
                    color_t: impl Into<Vector4<f32>>,
                    text_t: &str,
                    color_ct: impl Into<Vector4<f32>>,
                    text_ct: &str,
                ) {
                    match &player_team {
                        2 => {
                            // Terrorists
                            ctx.ui.text_colored(color_t, text_t)
                        }
                        3 => {
                            // Counter-Terrorists
                            ctx.ui.text_colored(color_ct, text_ct)
                        }
                        &_ => {
                            log::warn!("weird team id! {}", &player_team)
                        }
                    }
                }

                if !matches!(&bomb_info.state, C4State::Dropped) {
                    let plant_str = &format!(
                        "Bomb planted {}",
                        if bomb_info.bomb_site == 0 { "A" } else { "B" }
                    );

                    helper_text_color(&ctx, &self.local_team, green, plant_str, red, plant_str);
                }

                match &bomb_info.state {
                    C4State::Active {
                        time_detonation,
                        defuse,
                    } => {
                        let ten_seconds = *time_detonation < 10f32;
                        let five_seconds = *time_detonation < 5f32;
                        let is_terrorist = matches!(&self.local_team, 2);

                        let mut boom_color = [0.0, 0.0, 0.0, 0.0];

                        if *&self.num_kit > 0 {
                            if is_terrorist {
                                boom_color = red;
                                if ten_seconds {
                                    boom_color = orange;
                                }
                                if five_seconds {
                                    boom_color = green;
                                }
                            } else {
                                boom_color = green;
                                if ten_seconds {
                                    boom_color = orange;
                                }
                                if five_seconds {
                                    boom_color = red;
                                }
                            }
                        } else if ten_seconds {
                            boom_color = if is_terrorist { green } else { red };
                        }

                        ctx.ui
                            .text_colored(boom_color, &format!("Time: {:.3}", time_detonation));

                        if let Some(defuse) = defuse.as_ref() {
                            let defuse_str = &format!(
                                "Defused in {:.3} by {}",
                                defuse.time_remaining, defuse.player_name
                            );

                            if defuse.time_remaining > *time_detonation {
                                helper_text_color(
                                    &ctx,
                                    &self.local_team,
                                    green,
                                    defuse_str,
                                    red,
                                    defuse_str,
                                );
                            } else {
                                helper_text_color(
                                    &ctx,
                                    &self.local_team,
                                    red,
                                    defuse_str,
                                    green,
                                    defuse_str,
                                );
                            };
                        } else {
                            let text = "Not being defused";
                            helper_text_color(&ctx, &self.local_team, green, text, red, text);
                        }
                    }
                    C4State::Defused => {
                        let text = "Bomb has been defused";
                        helper_text_color(&ctx, &self.local_team, red, text, green, text);
                    }
                    C4State::Detonated => {
                        let text = "Bomb has been detonated";
                        helper_text_color(&ctx, &self.local_team, green, text, red, text);
                    }
                    C4State::Dropped => {
                        let text = "Bomb has been dropped";
                        helper_text_color(&ctx, &self.local_team, red, text, orange, text);
                    }
                }
            });
    }
}
