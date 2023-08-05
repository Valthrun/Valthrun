use std::{sync::Arc, marker::PhantomData};

use cs2_schema::{MemoryHandle, SchemaValue};

use crate::CS2Handle;


pub struct CUtlVector<T> {
    memory: Arc<dyn MemoryHandle>,
    offset: u64,

    _dummy: PhantomData<T>,
}

impl<T> CUtlVector<T> {
    pub fn element_count(&self) -> anyhow::Result<i32> {
        SchemaValue::from_memory(&self.memory, self.offset + 0x00)
    }

    pub fn elements_ptr(&self) -> anyhow::Result<u64> {
        SchemaValue::from_memory(&self.memory, self.offset + 0x08)
    }
}

impl<T: SchemaValue> CUtlVector<T> {
    /// Reference element at index.
    /// Attention: Element index is not checked on runtime!
    pub fn reference_element(&self, cs2: &CS2Handle, index: usize) -> anyhow::Result<T> {
        assert!(index < (self.element_count()? as usize));
        cs2.read_schema(&[ self.elements_ptr()? + (T::value_size() * index) as u64 ])
    }

    /// Read element at index.
    /// Attention: Element index is not checked on runtime!
    pub fn read_element(&self, cs2: &CS2Handle, index: usize) -> anyhow::Result<T> {
        assert!(index < (self.element_count()? as usize));
        cs2.read_schema(&[ self.elements_ptr()? + (T::value_size() * index) as u64 ])
    }
}

impl<T> SchemaValue for CUtlVector<T> {
    fn value_size() -> usize {
        0x10
    }

    fn from_memory(memory: &Arc<dyn MemoryHandle>, offset: u64) -> anyhow::Result<Self> {
        Ok(Self {
            memory: memory.clone(),
            offset,
            _dummy: Default::default(),
        })
    }
}