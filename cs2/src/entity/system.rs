use std::sync::Arc;

use anyhow::{Context, Ok};
use cs2_schema_declaration::{Ptr, SchemaValue};
use cs2_schema_generated::{
    cs2::client::{CCSPlayerController, CEntityIdentity},
    EntityHandle,
};
use obfstr::obfstr;

use crate::{CEntityIdentityEx, CS2Handle, CS2Offsets};

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
    pub fn get_local_player_controller(&self) -> anyhow::Result<Ptr<CCSPlayerController>> {
        self.cs2
            .reference_schema::<Ptr<CCSPlayerController>>(&[self.offsets.local_controller])
    }

    pub fn all_identities(&self) -> anyhow::Result<Vec<CEntityIdentity>> {
        let mut result = Vec::new();
        result.reserve(512);

        let base_identity =
            self.cs2
                .read_schema::<CEntityIdentity>(&[self.offsets.global_entity_list, 0, 0])?;

        result.push(base_identity.clone());

        let mut prev_entity = base_identity.m_pPrev()?;
        while !prev_entity.is_null()? {
            let entity = prev_entity
                .read_schema()
                .context(obfstr!("failed to read prev entity identity").to_string())?;
            prev_entity = entity.m_pPrev()?;
            result.push(entity);
        }

        let mut next_entity = base_identity.m_pNext()?;
        while !next_entity.is_null()? {
            let entity = next_entity
                .read_schema()
                .context(obfstr!("failed to read next entity identity").to_string())?;

            next_entity = entity.m_pNext()?;
            result.push(entity);
        }

        Ok(result)
    }

    pub fn all_identities_of_class(
        &self,
        reference: &CEntityIdentity,
    ) -> anyhow::Result<Vec<CEntityIdentity>> {
        let mut result = Vec::new();
        result.reserve(512);

        result.push(reference.clone());

        let mut prev_entity = reference.m_pPrevByClass()?;
        while !prev_entity.is_null()? {
            let entity = prev_entity
                .read_schema()
                .context(obfstr!("failed to read prev entity identity").to_string())?;
            prev_entity = entity.m_pPrevByClass()?;
            result.push(entity);
        }

        let mut next_entity = reference.m_pNextByClass()?;
        while !next_entity.is_null()? {
            let entity = next_entity
                .read_schema()
                .context(obfstr!("failed to read next entity identity").to_string())?;

            next_entity = entity.m_pNextByClass()?;
            result.push(entity);
        }

        Ok(result)
    }

    /// Returns the entity ptr
    pub fn get_by_handle<T: SchemaValue>(
        &self,
        handle: &EntityHandle<T>,
    ) -> anyhow::Result<Option<Ptr<T>>> {
        let (bulk, offset) = handle.entity_array_offsets();
        let identity = self.cs2.read_schema::<CEntityIdentity>(&[
            self.offsets.global_entity_list,
            bulk * 0x08,
            offset * CEntityIdentity::value_size().context("missing entity identity size")? as u64,
        ])?;

        if identity.handle::<T>()?.get_entity_index() == handle.get_entity_index() {
            Ok(Some(identity.entity_ptr::<T>()?))
        } else {
            Ok(None)
        }
    }

    pub fn get_player_controllers(&self) -> anyhow::Result<Vec<Ptr<CCSPlayerController>>> {
        let local_controller = self
            .get_local_player_controller()?
            .reference_schema()
            .context("missing local player controller")?;

        let local_controller_identitiy = local_controller.m_pEntity()?.read_schema()?;
        let identities = self.all_identities_of_class(&local_controller_identitiy)?;
        Ok(identities
            .into_iter()
            .map(|identity| identity.entity_ptr())
            .try_collect()?)
    }
}
