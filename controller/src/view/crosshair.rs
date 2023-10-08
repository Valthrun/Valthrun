use std::time::Instant;

use anyhow::Context;
use cs2::CEntityIdentityEx;
use cs2_schema_generated::{
    cs2::client::CEntityInstance,
    EntityHandle,
};

use crate::UpdateContext;

#[derive(Debug)]
pub struct CrosshairTarget {
    pub entity_id: u32,
    pub entity_type: Option<String>,
    pub timestamp: Instant,
}

pub struct LocalCrosshair {
    offset_crosshair_id: u64,
    current_target: Option<CrosshairTarget>,
}

impl LocalCrosshair {
    pub fn new(offset_crosshair_id: u64) -> Self {
        Self {
            offset_crosshair_id,
            current_target: None,
        }
    }

    pub fn current_target(&self) -> Option<&CrosshairTarget> {
        self.current_target.as_ref()
    }

    fn read_crosshair_entity(&self, ctx: &UpdateContext) -> anyhow::Result<Option<u32>> {
        let local_player_controller = ctx
            .cs2_entities
            .get_local_player_controller()?
            .try_reference_schema()?;

        let local_player_controller = match local_player_controller {
            Some(local_player_controller) => local_player_controller,
            None => return Ok(None),
        };

        let local_pawn_ptr = match ctx
            .cs2_entities
            .get_by_handle(&local_player_controller.m_hPlayerPawn()?)?
        {
            Some(ptr) => ptr.entity()?,
            None => return Ok(None),
        };

        let entity_id = ctx
            .cs2
            .reference_schema::<u32>(&[local_pawn_ptr.address()? + self.offset_crosshair_id])?;
        if entity_id != 0xFFFFFFFF {
            Ok(Some(entity_id))
        } else {
            Ok(None)
        }
    }

    pub fn update(&mut self, ctx: &UpdateContext) -> anyhow::Result<Option<&CrosshairTarget>> {
        let crosshair_entity_handle = match self.read_crosshair_entity(ctx)? {
            Some(entity_id) => EntityHandle::<CEntityInstance>::from_index(entity_id),
            None => {
                self.current_target = None;
                return Ok(None);
            }
        };

        let new_target = self
            .current_target
            .as_ref()
            .map(|target| target.entity_id != crosshair_entity_handle.get_entity_index())
            .unwrap_or(true);

        if new_target {
            let crosshair_entity_identnity = ctx
                .cs2_entities
                .get_by_handle(&crosshair_entity_handle)?
                .context("failed to resolve crosshair entity id")?;

            let target_type = ctx
                .class_name_cache
                .lookup(&crosshair_entity_identnity.entity_class_info()?)?;

            self.current_target = Some(CrosshairTarget {
                entity_id: crosshair_entity_handle.get_entity_index(),
                entity_type: target_type.cloned(),
                timestamp: Instant::now(),
            });
        }

        Ok(self.current_target.as_ref())
    }
}
