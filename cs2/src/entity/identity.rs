use anyhow::Context;
use obfstr::obfstr;

use crate::{EntityHandle, CS2Handle, Module};


#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct EntityIdentity {
    pub entity_ptr: u64,
    ptr_2: u64,

    pub handle: EntityHandle,
    pub name_stringable_index: u32,
    pub name: u64,

    pub designer_name: u64,
    pad_0: u64,

    pub flags: u64,
    pub world_group_id: u32,
    pub data_object_types: u32,

    pub path_index: u64,
    pad_1: u64,

    pad_2: u64,
    pub p_prev: u64,

    pub p_next: u64,
    pub p_prev_by_class: u64,

    pub p_next_by_class: u64,
}
const _: [u8; 120] = [0; std::mem::size_of::<EntityIdentity>()];

impl EntityIdentity {
    pub fn collect_all_of_class(&self, cs2: &CS2Handle) -> anyhow::Result<Vec<EntityIdentity>> {
        let mut result = Vec::new();
        result.reserve(128);
        result.push(self.clone());

        let mut prev_entity = self.p_prev_by_class;
        while prev_entity > 0 {
            let entity = cs2
                .read::<EntityIdentity>(Module::Absolute, &[prev_entity])
                .context(obfstr!("failed to read prev entity identity of class").to_string())?;
            prev_entity = entity.p_prev_by_class;
            result.push(entity);
        }

        let mut next_entity = self.p_next_by_class;
        while next_entity > 0 {
            let entity = cs2
                .read::<EntityIdentity>(Module::Absolute, &[next_entity])
                .context(obfstr!("failed to read next entity identity of class").to_string())?;
            next_entity = entity.p_next_by_class;
            result.push(entity);
        }

        Ok(result)
    }
}
