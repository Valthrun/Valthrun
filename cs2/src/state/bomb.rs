use std::ffi::CStr;

use anyhow::Context;
use cs2_schema_generated::cs2::client::{
    C_BasePlayerPawn,
    C_PlantedC4,
};
use obfstr::obfstr;
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use super::StateGlobals;
use crate::{
    CEntityIdentityEx,
    ClassNameCache,
    StateCS2Memory,
    StateEntityList,
};

#[derive(Debug)]
pub struct BombDefuser {
    /// Total time remaining for a successful bomb defuse
    pub time_remaining: f32,

    /// The defuser's player name
    pub player_name: String,
}

#[derive(Debug)]
pub enum PlantedC4State {
    /// Bomb is currently actively ticking
    Active {
        /// Time remaining (in seconds) until detonation
        time_detonation: f32,
    },

    /// Bomb has detonated
    Detonated,

    /// Bomb has been defused
    Defused,

    /// Bomb has not been planted
    NotPlanted,
}

/// Information about the currently active planted C4
pub struct PlantedC4 {
    /// Planted bomb site
    /// 0 = A
    /// 1 = B
    pub bomb_site: u8,

    /// Current state of the planted C4
    pub state: PlantedC4State,

    /// Current bomb defuser
    pub defuser: Option<BombDefuser>,
}

impl State for PlantedC4 {
    type Parameter = ();

    fn create(states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        let memory = states.resolve::<StateCS2Memory>(())?;
        let globals = states.resolve::<StateGlobals>(())?;
        let entities = states.resolve::<StateEntityList>(())?;
        let class_name_cache = states.resolve::<ClassNameCache>(())?;

        for entity_identity in entities.entities().iter() {
            // Optimized class name check
            let is_c4 = class_name_cache
                .lookup(&entity_identity.entity_class_info()?)
                .context("class name")?
                .map(|name| name == "C_PlantedC4")
                .unwrap_or(false);

            if !is_c4 {
                continue;
            }

            let bomb = entity_identity
                .entity_ptr::<dyn C_PlantedC4>()?
                .value_copy(memory.view())?
                .context("bomb entity nullptr")?;

            if !bomb.m_bC4Activated()? {
                continue;
            }

            let bomb_site = bomb.m_nBombSite()? as u8;
            if bomb.m_bBombDefused()? {
                return Ok(Self {
                    bomb_site,
                    defuser: None,
                    state: PlantedC4State::Defused,
                });
            }

            let time_blow = bomb.m_flC4Blow()?.m_Value()?;
            if time_blow <= globals.time_2()? {
                return Ok(Self {
                    bomb_site,
                    defuser: None,
                    state: PlantedC4State::Detonated,
                });
            }

            let defusing = if bomb.m_bBeingDefused()? {
                let time_defuse = bomb.m_flDefuseCountDown()?.m_Value()?;
                let handle_defuser = bomb.m_hBombDefuser()?;

                let defuser = entities
                    .entity_from_handle(&handle_defuser)
                    .context("missing bomb defuser pawn")?
                    .value_reference(memory.view_arc())
                    .context("defuser pawn nullptr")?;

                let defuser_controller = entities
                    .entity_from_handle(&defuser.m_hController()?)
                    .with_context(|| obfstr!("missing bomb defuser controller").to_string())?
                    .value_reference(memory.view_arc())
                    .context("defuser controller nullptr")?;

                let defuser_name =
                    CStr::from_bytes_until_nul(&defuser_controller.m_iszPlayerName()?)
                        .ok()
                        .map(|cstr| cstr.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "Name Error".to_string());

                Some(BombDefuser {
                    time_remaining: time_defuse - globals.time_2()?,
                    player_name: defuser_name,
                })
            } else {
                None
            };

            return Ok(Self {
                bomb_site,
                defuser: defusing,
                state: PlantedC4State::Active {
                    time_detonation: time_blow - globals.time_2()?,
                },
            });
        }

        Ok(Self {
            bomb_site: 0,
            defuser: None,
            state: PlantedC4State::NotPlanted,
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}
