use std::ffi::CStr;

use anyhow::Context;
use cs2_schema_generated::cs2::client::{
    C_CSObserverPawn,
    C_CSPlayerPawnBase,
};
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
    pub target_entity_index: u32,
    pub spectators: Vec<SpectatorInfo>,
}

impl State for SpectatorList {
    type Parameter = u32;

    fn create(
        states: &StateRegistry,
        target_entity_index: Self::Parameter,
    ) -> anyhow::Result<Self> {
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
                let observer_services_ptr = observer_pawn.m_pObserverServices();
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

            let current_observer_target = entities.get_by_handle(&observer_target_handle)?;
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

            if target_controller_handle.get_entity_index() != target_entity_index {
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
            target_entity_index,
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}

/// Get the controller id which we're currently following
pub struct LocalCameraControllerTarget {
    pub is_local_entity: bool,
    pub target_controller_entity_id: Option<u32>,
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
                    target_controller_entity_id: None,
                    is_local_entity: false,
                });
            }
        };

        if player_controller.m_bPawnIsAlive()? {
            /*
             * Our player pawn is alive.
             * This most certenly means we're currently following our pawn.
             */
            let entity_index = player_controller
                .m_hOriginalControllerOfCurrentPawn()?
                .get_entity_index();
            Ok(Self {
                target_controller_entity_id: Some(entity_index),
                is_local_entity: true,
            })
        } else {
            let observer_pawn =
                match { entities.get_by_handle(&player_controller.m_hObserverPawn()?)? } {
                    Some(pawn) => pawn.entity()?.reference_schema()?,
                    None => {
                        /* this is odd... */
                        return Ok(Self {
                            target_controller_entity_id: None,
                            is_local_entity: false,
                        });
                    }
                };

            let observer_target_handle = observer_pawn
                .m_pObserverServices()?
                .reference_schema()?
                .m_hObserverTarget()?;

            let local_observed_controller = entities
                .get_by_handle(&observer_target_handle)?
                .map(|identity| {
                    identity
                        .entity()?
                        .cast::<C_CSPlayerPawnBase>()
                        .try_reference_schema()
                        .with_context(|| {
                            obfstr!("failed to read local observer target pawn").to_string()
                        })
                })
                .transpose()?
                .flatten()
                .map(|player_pawn| player_pawn.m_hController())
                .transpose()?;

            Ok(Self {
                is_local_entity: false,
                target_controller_entity_id: local_observed_controller
                    .map(|identity| identity.get_entity_index()),
            })
        }
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}
