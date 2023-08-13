use std::{sync::Arc, marker::PhantomData};

use cs2_schema_declaration::{MemoryHandle, SchemaValue, Ptr};

pub struct CUtlMemory<T> {
    memory: Arc<dyn MemoryHandle>,
    offset: u64,

    _data: PhantomData<T>,
}


impl<T: SchemaValue> CUtlMemory<T> {
    pub fn buffer(&self) -> anyhow::Result<Ptr<[T]>> {
        SchemaValue::from_memory(&self.memory, self.offset + 0x00)
    }

    pub fn allocation_count(&self) -> anyhow::Result<u32> {
        SchemaValue::from_memory(&self.memory, self.offset + 0x08)
    }

    pub fn grow_size(&self) -> anyhow::Result<u32> {
        SchemaValue::from_memory(&self.memory, self.offset + 0x0C)
    }
}

impl<T: SchemaValue> SchemaValue for CUtlMemory<T> {
    fn value_size() -> Option<usize> {
        Some(0x10)
    }

    fn from_memory(memory: &Arc<dyn MemoryHandle>, offset: u64) -> anyhow::Result<Self> {
        Ok(Self {
            memory: memory.clone(),
            offset,

            _data: Default::default()
        })
    }
}