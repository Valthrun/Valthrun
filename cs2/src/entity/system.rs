use std::sync::Arc;

use anyhow::Context;
use cs2_schema::offsets;
use obfstr::obfstr;

use crate::{CS2Offsets, CS2Handle, Module, EntityHandle, EntityIdentity};

/// Helper class for CS2 global entity system
pub struct EntitySystem {
    cs2: Arc<CS2Handle>,
    offsets: Arc<CS2Offsets>,
}

impl EntitySystem {
    pub fn new(cs2: Arc<CS2Handle>, offsets: Arc<CS2Offsets>) -> Self {
        Self { cs2, offsets }
    }

    /* Returns a CSSPlayerController instance */
    pub fn get_local_player_controller(&self) -> anyhow::Result<Option<u64>> {
        let entity = self.cs2
            .read::<u64>(Module::Client, &[self.offsets.local_controller])
            .with_context(|| obfstr!("failed to read local player controller").to_string())?;

        if entity > 0 {
            Ok(Some(entity))
        } else {
            Ok(None)
        }
    }

    pub fn all_identities(&self) -> anyhow::Result<Vec<EntityIdentity>> {
        let mut result = Vec::new();
        result.reserve(512);

        let base_identity = self.cs2.read::<EntityIdentity>(
            Module::Client,
            &[self.offsets.global_entity_list, 0, 0],
        )?;
        result.push(base_identity.clone());

        let mut prev_entity = base_identity.p_prev;
        while prev_entity > 0 {
            let entity = self.cs2
                .read::<EntityIdentity>(Module::Absolute, &[prev_entity])
                .context(obfstr!("failed to read prev entity identity").to_string())?;
            prev_entity = entity.p_prev;
            result.push(entity);
        }

        let mut next_entity = base_identity.p_next;
        while next_entity > 0 {
            let entity = self.cs2
                .read::<EntityIdentity>(Module::Absolute, &[next_entity])
                .context(obfstr!("failed to read next entity identity").to_string())?;
            next_entity = entity.p_next;
            result.push(entity);
        }

        Ok(result)
    }

    /// Returns the entity ptr
    pub fn get_by_handle(
        &self,
        handle: &EntityHandle,
    ) -> anyhow::Result<Option<u64>> {
        let (bulk, offset) = handle.entity_array_offsets();
        let identity = self.cs2.read::<EntityIdentity>(
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
    pub fn get_player_controllers(&self) -> anyhow::Result<Vec<u64>> {
        let local_controller_identity = self.cs2.read::<EntityIdentity>(Module::Client, &[
            self.offsets.local_controller,
            offsets::client::CEntityInstance::m_pEntity, /* read the entity identnity index  */
            0, /* read everything */
        ]).with_context(|| obfstr!("failed to read local player controller identity").to_string())?;

        Ok(local_controller_identity
            .collect_all_of_class(&self.cs2)?
            .into_iter()
            .map(|identity| identity.entity_ptr)
            .collect())
    }
}