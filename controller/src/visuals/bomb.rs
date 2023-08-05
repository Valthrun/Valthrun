use anyhow::Context;
use cs2::{Module, EntityHandle};
use cs2_schema::offsets;
use obfstr::obfstr;

use crate::Application;


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

pub fn read_bomb_state(ctx: &Application) -> anyhow::Result<BombState> {
    let entities = ctx.cs2_entities.all_identities()
        .with_context(|| obfstr!("failed to read entity list").to_string())?;

    for entity in entities.iter() {
        let entity_vtable = ctx.cs2.read::<u64>(Module::Absolute, &[
            entity.entity_ptr + 0x00, // V-Table
        ])?;

        let class_name = ctx.class_name_cache.lookup(entity_vtable)?;
        if !(*class_name).as_ref().map(|name| name == "C_PlantedC4").unwrap_or(false) {
            /* Entity isn't the bomb. */
            continue;
        }

        // TODO. Read the whole class at once (we know the class size from the schema)
        //       This would require another schema structure thou...

        let is_activated = ctx.cs2.read::<bool>(Module::Absolute, &[
            entity.entity_ptr + offsets::client::C_PlantedC4::m_bC4Activated
        ])?;
        if !is_activated {
            /* This bomb hasn't been activated (yet) */
            continue;
        }

        let is_defused = ctx.cs2.read::<bool>(Module::Absolute, &[
            entity.entity_ptr + offsets::client::C_PlantedC4::m_bBombDefused
        ])?;
        if is_defused {
            return Ok(BombState::Defused);
        }

        let time_blow = ctx.cs2.read::<f32>(Module::Absolute, &[
            entity.entity_ptr + offsets::client::C_PlantedC4::m_flC4Blow
        ])?;
        let bomb_site = ctx.cs2.read::<u8>(Module::Absolute, &[
            entity.entity_ptr + offsets::client::C_PlantedC4::m_nBombSite
        ])?;

        let globals = ctx.cs2_globals.as_ref().context("missing globals")?;
        if time_blow <= globals.time_2()? {
            return Ok(BombState::Detonated);
        }

        let is_defusing = ctx.cs2.read::<bool>(Module::Absolute, &[
            entity.entity_ptr + offsets::client::C_PlantedC4::m_bBeingDefused
        ])?;
        let defusing = if is_defusing {
            let time_defuse = ctx.cs2.read::<f32>(Module::Absolute, &[
                entity.entity_ptr + offsets::client::C_PlantedC4::m_flDefuseCountDown
            ])?;

            let handle_defuser = ctx.cs2.read::<EntityHandle>(Module::Absolute, &[
                entity.entity_ptr + offsets::client::C_PlantedC4::m_hBombDefuser
            ])?;
            
            let defuser = ctx.cs2_entities.get_by_handle(&handle_defuser)?
                .with_context(|| obfstr!("missing bomb defuser player pawn").to_string())?;

            let handle_controller = ctx.cs2.read::<EntityHandle>(Module::Absolute, &[ 
                defuser + offsets::client::C_BasePlayerPawn::m_hController
            ])?;
            let controller = ctx.cs2_entities.get_by_handle(&handle_controller)?
                .with_context(|| obfstr!("missing pawn controller").to_string())?;
            
            let defuser_name = ctx.cs2.read_string(
                Module::Absolute,
                &[controller + offsets::client::CBasePlayerController::m_iszPlayerName],
                Some(128),
            )?;

            Some(BombDefuser{ 
                time_remaining: time_defuse - globals.time_2()?,
                player_name: defuser_name
            })
        } else {
            None
        };

        return Ok(BombState::Active { 
            bomb_site, time_detonation: time_blow - globals.time_2()?, 
            defuse: defusing
        });
    }

    return Ok(BombState::Unset);
}