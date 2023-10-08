use std::marker::PhantomData;

use cs2_schema_declaration::{
    MemoryHandle,
    Ptr,
    SchemaValue,
};

pub struct CUtlMemory<T> {
    memory: MemoryHandle,
    _data: PhantomData<T>,
}

impl<T: SchemaValue> CUtlMemory<T> {
    pub fn buffer(&self) -> anyhow::Result<Ptr<[T]>> {
        self.memory.reference_schema(0x00)
    }

    pub fn allocation_count(&self) -> anyhow::Result<u32> {
        self.memory.reference_schema(0x08)
    }

    pub fn grow_size(&self) -> anyhow::Result<u32> {
        self.memory.reference_schema(0x0C)
    }
}

impl<T: SchemaValue> SchemaValue for CUtlMemory<T> {
    fn value_size() -> Option<u64> {
        Some(0x10)
    }

    fn from_memory(memory: MemoryHandle) -> anyhow::Result<Self> {
        Ok(Self {
            memory: memory,
            _data: Default::default(),
        })
    }
}
