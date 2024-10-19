use std::ffi::CStr;

use anyhow::Context;
use cs2_schema_generated::cs2::client::C_CSObserverPawn;
use obfstr::obfstr;
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    CEntityIdentityEx,
    ClassNameCache,
    EntitySystem,
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
        let entities = states.resolve::<EntitySystem>(())?;
        let class_name_cache = states.resolve::<ClassNameCache>(())?;

        let mut spectators = Vec::new();
        for entity_identity in entities.all_identities() {
            let entity_class = class_name_cache.lookup(&entity_identity.entity_class_info()?)?;
            if entity_class
                .map(|name| *name != "C_CSObserverPawn")
                .unwrap_or(true)
            {
                continue;
            }

            let observer_pawn = entity_identity
                .entity_ptr::<C_CSObserverPawn>()?
                .read_schema()?;

            let observer_target_handle = {
                let observer_services = observer_pawn
                    .m_pObserverServices()?
                    .try_reference_schema()
                    .with_context(|| obfstr!("failed to read observer services").to_string())?;

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
            let current_player_controller = entities.get_by_handle(&observer_controller_handle)?;
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
        let entities = states.resolve::<EntitySystem>(())?;

        let local_player_controller = entities
            .get_local_player_controller()?
            .try_reference_schema()
            .with_context(|| obfstr!("failed to read local player controller").to_string())?;

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
                match { entities.get_by_handle(&player_controller.m_hObserverPawn()?)? } {
                    Some(pawn) => pawn.entity()?.reference_schema()?,
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
                .reference_schema()?
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
