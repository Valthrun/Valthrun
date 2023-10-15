use std::{
    ffi::CStr,
};
use anyhow::Context;
use cs2::CEntityIdentityEx;
use obfstr::obfstr;
use cs2_schema_generated::cs2::client::{
    C_BasePlayerPawn,
};
use super::Enhancement;

pub struct LocalInfo {
    pub local_position: Result<[f32; 3], anyhow::Error>,
}
pub struct LocalPos {
    local_pos: Option<[f32; 3]>,
}

impl LocalPos {
    pub fn new() -> Self {
        LocalPos {
            local_pos: None,
        }
    }
}

impl Enhancement for LocalPos {
    fn update(&mut self, ctx: &crate::UpdateContext) -> anyhow::Result<()> {
        self.local_pos = None;

        if !ctx.settings.show_local_pos {
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

        for entity_identity in ctx.cs2_entities.all_identities() {
            let entity_class = ctx
                .class_name_cache
                .lookup(&entity_identity.entity_class_info()?)?;

            if !entity_class
                .map(|name| *name == "C_BasePlayerPawn")
                .unwrap_or(false)
            {
                /* entity is not a player pawn */
                continue;
            }

            let player_pawn_ptr = entity_identity.entity_ptr::<C_BasePlayerPawn>()?;
            let player_pawn = player_pawn_ptr.read_schema()?;
            let player_controller_handle = player_pawn.m_hController()?;
            let current_player_controller = ctx.cs2_entities.get_by_handle(&player_controller_handle)?;

            let player_controller = if let Some(identity) = &current_player_controller
            {
                identity.entity()?.reference_schema()?
            } else {
                continue;
            };

            let v_old_origin_ptr = player_pawn.m_vOldOrigin();
            let local_position = v_old_origin_ptr;

            // Atribuir local_position a self.local_pos se for Ok
            if let Ok(local_position) = local_position {
                self.local_pos = Some(local_position);
            } else {
                log::error!("Local pos not found!");
            }
        }

        Ok(())
    }

    fn render(
        &self,
        settings: &crate::settings::AppSettings,
        ui: &imgui::Ui,
        _view: &crate::view::ViewController,
    ) {
        // Verificar se a posição local do jogador deve ser exibida
        if !settings.show_local_pos {
            return;
        }

        let group = ui.begin_group();

        if let Some(local_pos) = &self.local_pos {
            let line_count = 1;
            let text_height = ui.text_line_height_with_spacing() * line_count as f32;

            let offset_x = ui.io().display_size[0] * 0.01;
            let offset_y = (ui.io().display_size[1] + text_height) * 0.5;

            ui.set_cursor_pos([offset_x, offset_y]);
            ui.text(&format!("Local Position: {:?}", local_pos));
        }

        group.end();
    }
}