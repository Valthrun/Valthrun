use std::{
    marker::PhantomData,
    ops::{
        Deref,
        DerefMut,
    },
};

use anyhow::{
    Context,
    Ok,
    Result,
};
use cs2_schema_declaration::{
    Ptr,
    SchemaValue,
};
use cs2_schema_generated::{
    cs2::client::{
        CCSPlayerController,
        CEntityIdentity,
    },
    EntityHandle,
};
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    CEntityIdentityEx,
    CS2HandleState,
    CS2Offsets,
    EntityList,
};

pub struct TypedEntityIdentity<T> {
    identity: CEntityIdentity,
    _data: PhantomData<T>,
}

impl<T: SchemaValue> TypedEntityIdentity<T> {
    pub fn entity(&self) -> anyhow::Result<Ptr<T>> {
        self.memory.reference_schema(0x00)
    }
}

impl<T> Deref for TypedEntityIdentity<T> {
    type Target = CEntityIdentity;

    fn deref(&self) -> &Self::Target {
        &self.identity
    }
}

impl<T> DerefMut for TypedEntityIdentity<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.identity
    }
}

/// Helper class for CS2 global entity system
pub struct EntitySystem {
    entity_list: EntityList,
    local_player: Ptr<CCSPlayerController>,
}

impl State for EntitySystem {
    type Parameter = ();

    fn create(states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        let cs2 = states.resolve::<CS2HandleState>(())?;
        let entity_list = states.resolve::<EntityList>(())?;
        let offsets = states.resolve::<CS2Offsets>(())?;

        let local_player =
            cs2.reference_schema::<Ptr<CCSPlayerController>>(&[offsets.local_controller])?;

        Ok(Self {
            entity_list: entity_list.clone(),
            local_player,
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}

impl EntitySystem {
    /* Returns a CSSPlayerController instance */
    pub fn get_local_player_controller(&self) -> anyhow::Result<Ptr<CCSPlayerController>> {
        Ok(self.local_player.clone())
    }

    pub fn all_identities(&self) -> &[CEntityIdentity] {
        self.entity_list.entities()
    }

    pub fn all_identities_of_class(
        &self,
        reference: &CEntityIdentity,
    ) -> anyhow::Result<Vec<CEntityIdentity>> {
        let class_info = reference.entity_class_info()?.address()?;

        let mut result = Vec::new();
        result.reserve(512);
        for identity in self.entity_list.entities() {
            if identity.entity_class_info()?.address()? != class_info {
                continue;
            }

            result.push(identity.clone());
        }

        return Ok(result);
    }

    /// Returns the entity ptr
    pub fn get_by_handle<T: SchemaValue>(
        &self,
        handle: &EntityHandle<T>,
    ) -> anyhow::Result<Option<TypedEntityIdentity<T>>> {
        Ok(self
            .entity_list
            .lookup_entity_index(handle.get_entity_index())
            .map(|identity| TypedEntityIdentity {
                identity: identity.clone(),
                _data: Default::default(),
            }))
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
            .collect::<Result<Vec<_>>>()?)
    }
}
