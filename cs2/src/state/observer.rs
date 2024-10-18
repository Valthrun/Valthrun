use std::ffi::CStr;

use anyhow::Context;
use cs2_schema_generated::cs2::client::{
    CBasePlayerController,
    C_BasePlayerPawn,
    C_CSObserverPawn,
};
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    CEntityIdentityEx,
    ClassNameCache,
    StateCS2Memory,
    StateEntityList,
    StateLocalPlayerController,
};

pub struct SpectatorInfo {
    pub spectator_name: String,
}

pub struct SpectatorList {
    pub target_entity_id: u32,
    pub spectators: Vec<SpectatorInfo>,
}

impl State for SpectatorList {
    type Parameter = u32;

    fn create(states: &StateRegistry, target_entity_id: Self::Parameter) -> anyhow::Result<Self> {
        let memory = states.resolve::<StateCS2Memory>(())?;
        let entities = states.resolve::<StateEntityList>(())?;
        let class_name_cache = states.resolve::<ClassNameCache>(())?;

        let mut spectators = Vec::new();
        for entity_identity in entities.entities() {
            let entity_class = class_name_cache.lookup(&entity_identity.entity_class_info()?)?;
            if entity_class
                .map(|name| *name != "C_CSObserverPawn")
                .unwrap_or(true)
            {
                continue;
            }

            let observer_pawn = entity_identity
                .entity_ptr::<dyn C_CSObserverPawn>()?
                .value_copy(memory.view())?
                .context("entity nullptr")?;

            let observer_target_handle = {
                let observer_services = observer_pawn
                    .m_pObserverServices()?
                    .value_reference(memory.view_arc());

                match observer_services {
                    Some(observer) => observer.m_hObserverTarget()?,
                    None => {
                        continue;
                    }
                }
            };

            if observer_target_handle.get_entity_index() != target_entity_id {
                continue;
            }

            let observer_controller_handle = observer_pawn.m_hController()?;
            let current_player_controller = entities
                .entity_from_handle(&observer_controller_handle)
                .context("missing observer controller")?
                .value_reference(memory.view_arc())
                .context("nullptr")?;

            let spectator_name =
                CStr::from_bytes_until_nul(&current_player_controller.m_iszPlayerName()?)
                    .context("player name missing nul terminator")?
                    .to_str()
                    .context("invalid player name")?
                    .to_string();

            spectators.push(SpectatorInfo { spectator_name });
        }

        Ok(Self {
            spectators,
            target_entity_id,
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}

/// Get the entity id which we're currently following
pub struct LocalCameraControllerTarget {
    pub is_local_entity: bool,
    pub target_entity_id: Option<u32>,
}

impl State for LocalCameraControllerTarget {
    type Parameter = ();

    fn create(states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        let memory = states.resolve::<StateCS2Memory>(())?;
        let local_player_controller = states.resolve::<StateLocalPlayerController>(())?;
        let entities = states.resolve::<StateEntityList>(())?;

        let local_player_controller = local_player_controller
            .instance
            .value_reference(memory.view_arc());

        let player_controller = match local_player_controller {
            Some(controller) => controller,
            None => {
                /* We're currently not connected */
                return Ok(Self {
                    target_entity_id: None,
                    is_local_entity: false,
                });
            }
        };

        if player_controller.m_bPawnIsAlive()? {
            /*
             * Our player pawn is alive.
             * This most certainly means we're currently following our pawn.
             */

            Ok(Self {
                target_entity_id: Some(player_controller.m_hPawn()?.get_entity_index()),
                is_local_entity: true,
            })
        } else {
            let observer_pawn =
                match { entities.entity_from_handle(&player_controller.m_hObserverPawn()?) } {
                    Some(pawn) => pawn
                        .value_reference(memory.view_arc())
                        .context("entity nullptr")?,
                    None => {
                        /* this is odd... */
                        return Ok(Self {
                            target_entity_id: None,
                            is_local_entity: false,
                        });
                    }
                };

            let observer_target_handle = observer_pawn
                .m_pObserverServices()?
                .value_reference(memory.view_arc())
                .context("m_pObserverServices nullptr")?
                .m_hObserverTarget()?;

            if !observer_target_handle.is_valid() {
                return Ok(Self {
                    target_entity_id: None,
                    is_local_entity: false,
                });
            }
            let target_entity_id = observer_target_handle.get_entity_index();

            Ok(Self {
                is_local_entity: false,
                target_entity_id: Some(target_entity_id),
            })
        }
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}
