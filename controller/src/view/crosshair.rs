use std::time::Instant;

use anyhow::Context;
use cs2::{
    CEntityIdentityEx,
    ClassNameCache,
    EntitySystem,
};
use cs2_schema_generated::{
    cs2::client::CEntityInstance,
    EntityHandle,
};
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

#[derive(Debug)]
pub struct CrosshairTarget {
    pub entity_id: u32,
    pub entity_type: Option<String>,
    pub timestamp: Instant,
}

pub struct LocalCrosshair {
    current_target: Option<CrosshairTarget>,
}

impl State for LocalCrosshair {
    type Parameter = ();

    fn create(_states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        Ok(Self {
            current_target: None,
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Persistent
    }

    fn update(&mut self, states: &StateRegistry) -> anyhow::Result<()> {
        let crosshair_entity_handle = match self.read_crosshair_entity(states)? {
            Some(entity_id) => EntityHandle::<CEntityInstance>::from_index(entity_id),
            None => {
                self.current_target = None;
                return Ok(());
            }
        };

        let new_target = self
            .current_target
            .as_ref()
            .map(|target| target.entity_id != crosshair_entity_handle.get_entity_index())
            .unwrap_or(true);

        if new_target {
            let entities = states.resolve::<EntitySystem>(())?;
            let class_name_cache = states.resolve::<ClassNameCache>(())?;

            let crosshair_entity_identnity = entities
                .get_by_handle(&crosshair_entity_handle)?
                .context("failed to resolve crosshair entity id")?;

            let target_type =
                class_name_cache.lookup(&crosshair_entity_identnity.entity_class_info()?)?;

            self.current_target = Some(CrosshairTarget {
                entity_id: crosshair_entity_handle.get_entity_index(),
                entity_type: target_type.cloned(),
                timestamp: Instant::now(),
            });
        }

        Ok(())
    }
}

impl LocalCrosshair {
    pub fn current_target(&self) -> Option<&CrosshairTarget> {
        self.current_target.as_ref()
    }

    fn read_crosshair_entity(&self, states: &StateRegistry) -> anyhow::Result<Option<u32>> {
        let entities = states.resolve::<EntitySystem>(())?;

        let local_player_controller = entities
            .get_local_player_controller()?
            .try_reference_schema()?;

        let local_player_controller = match local_player_controller {
            Some(local_player_controller) => local_player_controller,
            None => return Ok(None),
        };

        let local_pawn_ptr =
            match entities.get_by_handle(&local_player_controller.m_hPlayerPawn()?)? {
                Some(ptr) => ptr.entity()?,
                None => return Ok(None),
            };

        //let entity_id = 0xFFFFFFFF;
        let entity_id = local_pawn_ptr.reference_schema()?.m_iIDEntIndex()?;
        if entity_id != 0xFFFFFFFF {
            Ok(Some(entity_id))
        } else {
            Ok(None)
        }
    }
}
