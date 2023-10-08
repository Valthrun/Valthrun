use std::{
    fmt::Debug,
    marker::PhantomData,
};

use cs2_schema_declaration::{
    MemoryHandle,
    SchemaValue,
};

/// CS2 32 bit entity handle packed with
/// the entity index and serial number.
#[repr(C)]
#[derive(Default, Clone)]
pub struct EntityHandle<T> {
    pub value: u32,
    _data: PhantomData<T>,
}

impl<T> EntityHandle<T> {
    pub fn from_index(index: u32) -> Self {
        Self {
            value: index,
            _data: Default::default(),
        }
    }

    pub fn get_entity_index(&self) -> u32 {
        self.value & 0x7FFF
    }

    pub fn is_valid(&self) -> bool {
        self.get_entity_index() < 0x7FF0
    }

    pub fn get_serial_number(&self) -> u32 {
        self.value >> 15
    }
}

impl<T> Debug for EntityHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntityHandle")
            .field(
                "entity_index",
                &format_args!("0x{:X}", &self.get_entity_index()),
            )
            .field(
                "serial_number",
                &format_args!("0x{:X}", &self.get_serial_number()),
            )
            .finish()
    }
}

impl<T> SchemaValue for EntityHandle<T> {
    fn value_size() -> Option<u64> {
        Some(0x04)
    }

    fn from_memory(memory: MemoryHandle) -> anyhow::Result<Self> {
        Ok(Self {
            value: SchemaValue::from_memory(memory)?,
            _data: Default::default(),
        })
    }
}

pub type CEntityIndex = u32;
