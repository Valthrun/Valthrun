use std::collections::BTreeMap;

use anyhow::anyhow;
use cs2_schema_cutl::EntityHandle;
use cs2_schema_generated::cs2::client::CEntityIdentity;
use raw_struct::{
    builtins::Ptr64,
    Copy,
    FromMemoryView,
};
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    entity::identity::CEntityIdentityEx,
    CS2Offset,
    StateCS2Memory,
    StateResolvedOffset,
};

type InnerEntityList = [Copy<dyn CEntityIdentity>; 512];
type OuterEntityList = [Ptr64<InnerEntityList>; 64];

#[derive(Clone)]
pub struct StateEntityList {
    entities: Vec<Copy<dyn CEntityIdentity>>,
    handle_lookup: BTreeMap<u32, usize>,
}

impl State for StateEntityList {
    type Parameter = ();

    fn create(_states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        Ok(Self {
            entities: Vec::new(),
            handle_lookup: Default::default(),
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Persistent
    }

    fn update(&mut self, states: &StateRegistry) -> anyhow::Result<()> {
        let memory = states.resolve::<StateCS2Memory>(())?;
        let offset_global_entity_list =
            states.resolve::<StateResolvedOffset>(CS2Offset::GlobalEntityList)?;

        self.entities.clear();
        self.handle_lookup.clear();

        let outer_list =
            Ptr64::<OuterEntityList>::read_object(memory.view(), offset_global_entity_list.address)
                .map_err(|e| anyhow!(e))?;

        let outer_list = outer_list.elements(memory.view(), 0..outer_list.len().unwrap())?;
        for (bulk_index, bulk) in outer_list.into_iter().enumerate() {
            if bulk.is_null() {
                continue;
            }

            let list = bulk.elements(memory.view(), 0..bulk.len().unwrap())?;
            for (entry_index, entry) in list.into_iter().enumerate() {
                let entity_index = ((bulk_index << 9) | entry_index) as u32;
                let handle = entry.handle::<()>()?;
                if handle.get_entity_index() != entity_index {
                    /* entity is invalid */
                    continue;
                }

                self.entities.push(entry);
                self.handle_lookup
                    .insert(entity_index, self.entities.len() - 1);
            }
        }

        Ok(())
    }
}

impl StateEntityList {
    pub fn entities(&self) -> &[Copy<dyn CEntityIdentity>] {
        &self.entities
    }

    pub fn identity_from_index(&self, entity_index: u32) -> Option<&Copy<dyn CEntityIdentity>> {
        self.handle_lookup
            .get(&entity_index)
            .map(|index| self.entities.get(*index))
            .flatten()
    }

    pub fn entity_from_handle<T: ?Sized + 'static>(
        &self,
        handle: &EntityHandle<T>,
    ) -> Option<Ptr64<T>> {
        let entity_index = handle.get_entity_index();
        self.identity_from_index(entity_index)
            .map(|entity| entity.entity_ptr().unwrap())
    }

    pub fn entities_of_class(
        &self,
        reference: &(dyn CEntityIdentity + 'static),
    ) -> anyhow::Result<Vec<&Copy<dyn CEntityIdentity>>> {
        let class_info = reference.entity_class_info()?;

        let mut result = Vec::new();
        result.reserve(512);
        for identity in self.entities() {
            if identity.entity_class_info()?.address != class_info.address {
                continue;
            }

            result.push(identity);
        }

        return Ok(result);
    }
}
