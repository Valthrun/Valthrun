use std::ffi::CStr;

use anyhow::Context;
use cs2::CEntityIdentityEx;
use cs2_schema_generated::cs2::client::{
    C_CSObserverPawn,
    C_CSPlayerPawnBase,
};
use obfstr::obfstr;

use super::Enhancement;

pub struct SpectatorInfo {
    pub spectator_name: String,
}

pub struct SpectatorsList {
    spectators: Vec<SpectatorInfo>,
}

impl SpectatorsList {
    pub fn new() -> Self {
        SpectatorsList {
            spectators: Default::default(),
        }
    }
}

impl Enhancement for SpectatorsList {
    fn update(&mut self, ctx: &crate::UpdateContext) -> anyhow::Result<()> {
        self.spectators.clear();

        if !ctx.settings.spectators_list {
            return Ok(());
        }

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

        let actual_entity_index = if local_player_controller.m_bPawnIsAlive()? {
            local_player_controller
                .m_hOriginalControllerOfCurrentPawn()?
                .get_entity_index()
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

            let local_observer_target_handle = local_obs_pawn
                .m_pObserverServices()?
                .read_schema()?
                .m_hObserverTarget()?;

            let current_local_observer_target = ctx
                .cs2_entities
                .get_by_handle(&local_observer_target_handle)?;

            let local_observer_target_pawn = if let Some(identity) = &current_local_observer_target
            {
                identity
                    .entity()?
                    .cast::<C_CSPlayerPawnBase>()
                    .try_reference_schema()
                    .with_context(|| {
                        obfstr!("failed to read local observer target pawn").to_string()
                    })?
            } else {
                return Ok(());
            };

            let local_observer_target_pawn = match local_observer_target_pawn {
                Some(pawn) => pawn,
                None => {
                    return Ok(());
                }
            };

            let local_observed_controller = match local_observer_target_pawn.m_hController() {
                Ok(controller) => controller,
                Err(_e) => {
                    return Ok(());
                }
            };

            local_observed_controller.get_entity_index()
        };

        for entity_identity in ctx.cs2_entities.all_identities() {
            let entity_class = ctx
                .class_name_cache
                .lookup(&entity_identity.entity_class_info()?)?;

            if entity_class
                .map(|name| *name != "C_CSObserverPawn")
                .unwrap_or(true)
            {
                continue;
            }

            let player_pawn_ptr = entity_identity.entity_ptr::<C_CSObserverPawn>()?;
            let player_pawn = player_pawn_ptr.read_schema()?;
            let player_controller_handle = player_pawn.m_hController()?;

            let observer_target_handle = {
                let observer_services_ptr = player_pawn.m_pObserverServices();
                let observer_services = observer_services_ptr?
                    .try_reference_schema()
                    .with_context(|| obfstr!("failed to read observer services").to_string())?;

                match observer_services {
                    Some(observer) => observer.m_hObserverTarget()?,
                    None => {
                        continue;
                    }
                }
            };

            let current_observer_target =
                ctx.cs2_entities.get_by_handle(&observer_target_handle)?;

            let observer_target_pawn = if let Some(identity) = &current_observer_target {
                identity
                    .entity()?
                    .cast::<C_CSPlayerPawnBase>()
                    .try_reference_schema()
                    .with_context(|| obfstr!("failed to observer target pawn").to_string())?
            } else {
                continue;
            };

            let observer_target_pawn = match observer_target_pawn {
                Some(pawn) => pawn,
                None => {
                    continue;
                }
            };

            let target_controller_handle = match observer_target_pawn.m_hController() {
                Ok(controller) => controller,
                Err(_e) => {
                    continue;
                }
            };

            if target_controller_handle.get_entity_index() != actual_entity_index {
                continue;
            }

            let current_player_controller =
                ctx.cs2_entities.get_by_handle(&player_controller_handle)?;

            let player_controller = if let Some(identity) = &current_player_controller {
                identity.entity()?.reference_schema()?
            } else {
                continue;
            };

            let spectator_name = CStr::from_bytes_until_nul(&player_controller.m_iszPlayerName()?)
                .context("player name missing nul terminator")?
                .to_str()
                .context("invalid player name")?
                .to_string();

            self.spectators.push(SpectatorInfo { spectator_name });
        }

        Ok(())
    }

    fn render(
        &self,
        settings: &crate::settings::AppSettings,
        ui: &imgui::Ui,
        _view: &crate::view::ViewController,
    ) {
        if !settings.spectators_list {
            return;
        }

        let group = ui.begin_group();

        let line_count = self.spectators.iter().count();
        let text_height = ui.text_line_height_with_spacing() * line_count as f32;

        let offset_x = ui.io().display_size[0] * 0.01;
        let offset_y = (ui.io().display_size[1] - text_height) * 0.5;
        let mut offset_y = offset_y;

        for spectator in &self.spectators {
            ui.set_cursor_pos([offset_x, offset_y]);
            ui.text(&spectator.spectator_name);
            offset_y += ui.text_line_height_with_spacing();
        }

        group.end();
    }
}
