use std::{
    ffi::CStr,
};

use anyhow::Context;
use cs2::CEntityIdentityEx;
use obfstr::obfstr;
use cs2_schema_generated::cs2::client::{
    C_CSObserverPawn,
};

use super::Enhancement;

pub struct SpectatorInfo {
    pub spectator_name: String,
}

pub struct SpectatorsList {
    spectators: Vec<SpectatorInfo>
}

impl SpectatorsList {
    pub fn new() -> Self {
        SpectatorsList{
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

        let observice_entity_handle = if local_player_controller.m_bPawnIsAlive()? {
            local_player_controller.m_hPawn()?.get_entity_index()
        } else {
            return Ok(());
        };

        for entity_identity in ctx.cs2_entities.all_identities() {
            if entity_identity.handle::<()>()?.get_entity_index() == observice_entity_handle {
                /* current pawn we control/observe */
                continue;
            }

            let entity_class = ctx
                .class_name_cache
                .lookup(&entity_identity.entity_class_info()?)?;

            if !entity_class
                .map(|name| *name == "C_CSObserverPawn")
                .unwrap_or(false)
            {
                /* entity is not a player pawn */
                continue;
            }

            let player_pawn_ptr = entity_identity.entity_ptr::<C_CSObserverPawn>()?;
            let player_pawn = player_pawn_ptr.read_schema()?;
            let player_controller_handle = player_pawn.m_hController()?;
            let current_player_controller = ctx.cs2_entities.get_by_handle(&player_controller_handle)?;

            let player_controller = if let Some(identity) = &current_player_controller
            {
                identity.entity()?.reference_schema()?
            } else {
                continue;
            };

            let observer_services_ptr = player_pawn.m_pObserverServices();
            let observer_services = observer_services_ptr?
                .try_reference_schema()
                .with_context(|| obfstr!("failed to read observer services").to_string())?;

            let observer_target_handle = match observer_services {
                Some(observer) => observer.m_hObserverTarget()?,
                None => {
                    continue;
                }
            };

            let current_target = ctx.cs2_entities.get_by_handle(&observer_target_handle)?;

            let observer_target = if let Some(identity) = &current_target
            {
                identity.entity()?.cast::<C_CSObserverPawn>().reference_schema()?
            } else {
                continue;
            };

            let target_controller_handle = observer_target.m_hController()?;
            let target_current_controller = ctx.cs2_entities.get_by_handle(&target_controller_handle)?;

            let target_controller = if let Some(identity) = &target_current_controller
            {
                identity.entity()?.reference_schema()?
            } else {
                continue;
            };

            if target_controller.m_hPawn()?.get_entity_index() != observice_entity_handle {
                continue;
            }

            let spectator_name = CStr::from_bytes_until_nul(&player_controller.m_iszPlayerName()?)
                .context("player name missing nul terminator")?
                .to_str()
                .context("invalid player name")?
                .to_string();

            self.spectators.push(SpectatorInfo{
                spectator_name
            });
            continue;
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
        let offset_y = (ui.io().display_size[1] + text_height) * 0.5;
        let mut offset_y = offset_y;

        for spectator in self.spectators.iter() {
            ui.set_cursor_pos([offset_x, offset_y]);
            ui.text(&format!("{}", spectator.spectator_name));
            offset_y += ui.text_line_height_with_spacing();
        };

        group.end();
    }
}
