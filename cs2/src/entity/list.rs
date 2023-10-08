use std::{
    collections::BTreeMap,
    sync::Arc,
};

use cs2_schema_declaration::Ptr;
use cs2_schema_generated::cs2::client::CEntityIdentity;

use crate::{
    CEntityIdentityEx,
    CS2Handle,
};

type InnerEntityList = [CEntityIdentity; 512];
type OuterEntityList = [Ptr<InnerEntityList>; 64];
pub struct EntityList {
    cs2: Arc<CS2Handle>,
    entity_list_offset: u64,

    entities: Vec<CEntityIdentity>,
    handle_lookup: BTreeMap<u32, usize>,
}

impl EntityList {
    pub fn new(cs2: Arc<CS2Handle>, entity_list_address: u64) -> Self {
        Self {
            cs2,
            entity_list_offset: entity_list_address,

            entities: Default::default(),
            handle_lookup: Default::default(),
        }
    }

    pub fn entities(&self) -> &[CEntityIdentity] {
        &self.entities
    }

    pub fn lookup_entity_index(&self, entity_index: u32) -> Option<&CEntityIdentity> {
        self.handle_lookup
            .get(&entity_index)
            .map(|index| self.entities.get(*index))
            .flatten()
    }

    pub fn cache_list(&mut self) -> anyhow::Result<()> {
        self.entities.clear();
        self.handle_lookup.clear();

        let outer_list = self
            .cs2
            .read_schema::<OuterEntityList>(&[self.entity_list_offset, 0x00])?;
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
