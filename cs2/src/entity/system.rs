use std::sync::Arc;

use anyhow::Context;
use cs2_schema::offsets;
use obfstr::obfstr;

use crate::{CS2Offsets, CS2Handle, Module, EntityHandle, EntityIdentity};

/// Helper class for CS2 global entity system
pub struct EntitySystem {
    offsets: Arc<CS2Offsets>,
}

impl EntitySystem {
    pub fn new(offsets: Arc<CS2Offsets>) -> Self {
        Self { offsets }
    }

    /* Returns a CSSPlayerController instance */
    pub fn get_local_player_controller(&self, cs2: &CS2Handle) -> anyhow::Result<Option<u64>> {
        let entity = cs2
            .read::<u64>(Module::Client, &[self.offsets.local_controller])
            .with_context(|| obfstr!("failed to read local player controller").to_string())?;

        if entity > 0 {
            Ok(Some(entity))
        } else {
            Ok(None)
        }
    }

    pub fn get_by_handle(
        &self,
        cs2: &CS2Handle,
        handle: &EntityHandle,
    ) -> anyhow::Result<Option<u64>> {
        let (bulk, offset) = handle.entity_array_offsets();
        let identity = cs2.read::<EntityIdentity>(
            Module::Client,
            &[self.offsets.global_entity_list, bulk * 0x08, offset * 120],
        );

        let identity = match identity {
            Ok(identity) => identity,
            Err(error) => {
                return Err(error.context(format!(
                    "{}: {:?}",
                    obfstr!("failed to read global entity list entry for handle"),
                    handle
                )))
            }
        };

        if identity.handle.get_entity_index() == handle.get_entity_index() {
            Ok(Some(identity.entity_ptr))
        } else {
            Ok(None)
        }
    }

    /* Returns a Vec<CSSPlayerController*> */
    pub fn get_player_controllers(&self, cs2: &CS2Handle) -> anyhow::Result<Vec<u64>> {
        let local_controller_identity = cs2.read::<EntityIdentity>(Module::Client, &[
            self.offsets.local_controller,
            offsets::client::CEntityInstance::m_pEntity, /* read the entity identnity index  */
            0, /* read everything */
        ]).with_context(|| obfstr!("failed to read local player controller identity").to_string())?;

        Ok(local_controller_identity
            .collect_all_of_class(cs2)?
            .into_iter()
            .map(|identity| identity.entity_ptr)
            .collect())
    }
}