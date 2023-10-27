use std::ffi::CStr;

use anyhow::Context;
use cs2::CEntityIdentityEx;
use cs2_schema_generated::cs2::client::{
    C_CSPlayerPawn,
    C_PlantedC4,
};
use obfstr::obfstr;

use super::Enhancement;
use crate::{
    utils::ImguiUiEx,
    UpdateContext,
};

#[derive(Debug)]
pub struct BombDefuser {
    /// Total time remaining for a successfull bomb defuse
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

    //Current position of planted c4
    bomb_pos: nalgebra::Vector3<f32>,
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
    local_pos: Option<nalgebra::Vector3<f32>>,
}

impl BombInfo {
    pub fn new() -> Self {
        Self {
            bomb_state: None,
            local_pos: Default::default(),
        }
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

            let game_screen_node = bomb.m_pGameSceneNode()?.read_schema()?;

            let bomb_pos =
                nalgebra::Vector3::<f32>::from_column_slice(&game_screen_node.m_vecAbsOrigin()?);

            let bomb_site = bomb.m_nBombSite()? as u8;
            if bomb.m_bBombDefused()? {
                return Ok(Some(C4Info {
                    bomb_site,
                    state: C4State::Defused,
                    bomb_pos,
                }));
            }

            let time_blow = bomb.m_flC4Blow()?.m_Value()?;

            if time_blow <= ctx.globals.time_2()? {
                return Ok(Some(C4Info {
                    bomb_site,
                    state: C4State::Detonated,
                    bomb_pos,
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
                bomb_pos,
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
        if !(ctx.settings.bomb_esp || ctx.settings.bomb_timer) {
            return Ok(());
        }

        self.bomb_state = self.read_state(ctx)?;

        let local_player_controller = ctx
            .cs2_entities
            .get_local_player_controller()?
            .try_reference_schema()
            .with_context(|| obfstr!("failed to read local player controller").to_string())?;

        let local_player_controller = match local_player_controller {
            Some(controller) => controller,
            None => {
                /* We're currently not connected */
                return Ok(());
            }
        };

        let observice_entity_handle = if local_player_controller.m_bPawnIsAlive()? {
            local_player_controller.m_hPawn()?.get_entity_index()
        } else {
            let local_obs_pawn = match {
                ctx.cs2_entities
                    .get_by_handle(&local_player_controller.m_hObserverPawn()?)?
            } {
                Some(pawn) => pawn.entity()?.reference_schema()?,
                None => {
                    /* this is odd... */
                    return Ok(());
                }
            };

            local_obs_pawn
                .m_pObserverServices()?
                .read_schema()?
                .m_hObserverTarget()?
                .get_entity_index()
        };

        for entity_identity in ctx.cs2_entities.all_identities() {
            if entity_identity.handle::<()>()?.get_entity_index() == observice_entity_handle {
                /* current pawn we control/observe */
                let local_pawn = entity_identity
                    .entity_ptr::<C_CSPlayerPawn>()?
                    .read_schema()?;
                let local_pos =
                    nalgebra::Vector3::<f32>::from_column_slice(&local_pawn.m_vOldOrigin()?);
                self.local_pos = Some(local_pos);
                continue;
            }
        }
        Ok(())
    }

    fn render(
        &self,
        settings: &crate::settings::AppSettings,
        ui: &imgui::Ui,
        view: &crate::view::ViewController,
    ) {
        if !(settings.bomb_esp || settings.bomb_timer) {
            return;
        }
        let bomb_settings = &settings.bomb_settings.get("bomb");

        if let Some(bomb_settings) = bomb_settings {
            let mut color = [1.0, 1.0, 1.0, 1.0];

            if let (Some(bomb_info), esp_settings) = (&self.bomb_state, bomb_settings) {
                let offset_x = ui.io().display_size[0] * 1730.0 / 2560.0;
                let offset_y = ui.io().display_size[1] * PLAYER_AVATAR_TOP_OFFSET;
                let group = ui.begin_group();
                let line_count = match &bomb_info.state {
                    C4State::Active { .. } => 3,
                    C4State::Defused | C4State::Detonated => 2,
                };
                let text_height = ui.text_line_height_with_spacing() * line_count as f32;
                let offset_y = offset_y
                    + nalgebra::RealField::max(
                        0.0,
                        (ui.io().display_size[1] * PLAYER_AVATAR_SIZE - text_height) / 2.0,
                    );

                if esp_settings.bomb_site {
                    ui.set_cursor_pos([offset_x, offset_y]);
                    ui.text(&format!(
                        "Bomb planted {}",
                        if bomb_info.bomb_site == 0 { "A" } else { "B" }
                    ));
                }

                if esp_settings.bomb_status || settings.bomb_timer {
                    if !settings.bomb_timer {
                        ui.set_cursor_pos_x(offset_x);
                    }
                    match &bomb_info.state {
                        C4State::Active {
                            time_detonation,
                            defuse,
                        } => {
                            ui.text(&format!("Time: {:.3}", time_detonation));

                            if let Some(defuse) = defuse.as_ref() {
                                let color = if defuse.time_remaining > *time_detonation {
                                    [0.79, 0.11, 0.11, 1.0]
                                } else {
                                    [0.11, 0.79, 0.26, 1.0]
                                };

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
                            ui.text("Bomb has been defused");
                        }
                        C4State::Detonated => {
                            ui.text("Bomb has been detonated");
                        }
                    }
                }
                if let C4State::Active {
                    time_detonation, ..
                } = &bomb_info.state
                {
                    let pos = &bomb_info.bomb_pos;
                    if let Some(local_pos) = self.local_pos {
                        let distance = (pos - local_pos).norm() * 0.01905;
                        color = if esp_settings.bomb_position {
                            esp_settings
                                .bomb_position_color
                                .calculate_color(distance, *time_detonation)
                        } else {
                            [1.0, 1.0, 1.0, 1.0]
                        };
                    }
                }

                if esp_settings.bomb_position {
                    if let Some(pos) = view.world_to_screen(&bomb_info.bomb_pos, false) {
                        let y_offset = 0.0;
                        let draw = ui.get_window_draw_list();

                        let text = "BOMB";
                        let [text_width, _] = ui.calc_text_size(&text);
                        let mut pos = pos.clone();
                        pos.x -= text_width / 2.0;
                        pos.y += y_offset;
                        ui.set_cursor_pos_x(offset_x);
                        draw.add_text(pos, color, text);
                    }
                }

                group.end();
            }
        }
    }
}
