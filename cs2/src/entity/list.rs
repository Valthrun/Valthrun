use std::collections::BTreeMap;

use cs2_schema_declaration::Ptr;
use cs2_schema_generated::cs2::client::CEntityIdentity;
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    CEntityIdentityEx,
    CS2HandleState,
    CS2Offsets,
};

type InnerEntityList = [CEntityIdentity; 512];
type OuterEntityList = [Ptr<InnerEntityList>; 64];

#[derive(Clone)]
pub struct EntityList {
    entities: Vec<CEntityIdentity>,
    handle_lookup: BTreeMap<u32, usize>,
}

impl State for EntityList {
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
        let cs2 = states.resolve::<CS2HandleState>(())?;
        let offsets = states.resolve::<CS2Offsets>(())?;

        self.entities.clear();
        self.handle_lookup.clear();

        let outer_list = cs2.read_schema::<OuterEntityList>(&[offsets.global_entity_list, 0x00])?;
        for (bulk_index, bulk) in outer_list.into_iter().enumerate() {
            let list = match bulk.try_read_schema()? {
                Some(list) => list,
                None => continue,
            };

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

impl EntityList {
    pub fn entities(&self) -> &[CEntityIdentity] {
        &self.entities
    }

    pub fn lookup_entity_index(&self, entity_index: u32) -> Option<&CEntityIdentity> {
        self.handle_lookup
            .get(&entity_index)
            .map(|index| self.entities.get(*index))
            .flatten()
    }
}
