use std::time::Instant;

use anyhow::Context;
use cs2_schema::{EntityHandle, cs2::client::{CEntityInstance, C_CSPlayerPawn}};

use crate::Application;

pub struct CrosshairTarget {
    entity_id: u32,
    entity_type: String,
    timestamp: Instant
}

fn read_crosshair_entity(ctx: &mut Application) -> anyhow::Result<Option<u32>> {
    let local_player_controller = ctx.cs2_entities.get_local_player_controller()?
        .try_reference_schema()?;

    let local_player_controller = match local_player_controller {
        Some(local_player_controller) => local_player_controller,
        None => return Ok(None),
    };

    let local_pawn_ptr = match ctx.cs2_entities.get_by_handle(&local_player_controller.m_hPlayerPawn()?)? {
        Some(ptr) => ptr,
        None => return Ok(None)
    };

    let entity_id = ctx.cs2.read::<u32>(cs2::Module::Absolute, &[
        local_pawn_ptr.address()? + ctx.cs2_offsets.offset_crosshair_id
    ])?;
    if entity_id != 0xFFFFFFFF {
        Ok(Some(entity_id))
    } else {
        Ok(None)
    }
}

pub fn update_crosshair_target(ctx: &mut Application) -> anyhow::Result<()> {
    let crosshair_entity_handle = match read_crosshair_entity(ctx)? {
        Some(entity_id) => EntityHandle::<CEntityInstance>::from_index(entity_id),
        None => {
            ctx.crosshair_target = None;
            return Ok(());
        }
    };

    if let Some(current_target) = &ctx.crosshair_target {
        if current_target.entity_id == crosshair_entity_handle.get_entity_index() {
            /* target entity hasn't changed */
            return Ok(());
        }
    }

    let crosshair_entity = ctx.cs2_entities.get_by_handle(&crosshair_entity_handle)?
        .context("failed to resolve crosshair entity id")?
        .reference_schema()?;

    let target_type = ctx.class_name_cache.lookup(crosshair_entity.vtable()?.address()?)?;
    let target_type = (*target_type).as_ref().context("crosshair entity name")?;

    ctx.crosshair_target = Some(CrosshairTarget {
        entity_id: crosshair_entity_handle.get_entity_index(),
        entity_type: target_type.to_string(),
        timestamp: Instant::now()
    });

    // if target_type != "C_CSPlayerPawn" {
    //     return Ok(());
    // }

    // let target_player = crosshair_entity.as_schema::<C_CSPlayerPawn>()?;
    // if target_player.m_iTeamNum()? == local_player_controller.m_iTeamNum()? {
    //     return Ok(());
    // }
    // log::debug!("X: {:?} -> {}", crosshair_entity_handle.get_entity_index(), target_type);
    Ok(())
}