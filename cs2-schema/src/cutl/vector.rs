use std::{sync::Arc, marker::PhantomData};

use crate::{MemoryHandle, SchemaValue, Ptr};

/// struct CUtlVector<T> {
///     pub size: u32, // 0x00
///     pub data: *const T // 0x08
/// }
pub struct CUtlVector<T> {
    memory: Arc<dyn MemoryHandle>,
    offset: u64,

    _dummy: PhantomData<T>,
}

impl<T> CUtlVector<T> {
    pub fn element_count(&self) -> anyhow::Result<i32> {
        SchemaValue::from_memory(&self.memory, self.offset + 0x00)
    }

    pub fn elements(&self) -> anyhow::Result<Ptr<[T]>> {
        SchemaValue::from_memory(&self.memory, self.offset + 0x08)
    }
}

impl<T: SchemaValue> CUtlVector<T> {
    /// Reference element at index.
    /// Attention: Element index is not checked on runtime!
    pub fn reference_element(&self, index: usize) -> anyhow::Result<T> {
        assert!(index < (self.element_count()? as usize));
        self.elements()?.reference_element(index)
    }

    /// Read element at index.
    /// Attention: Element index is not checked on runtime!
    pub fn read_element(&self, index: usize) -> anyhow::Result<T> {
        assert!(index < (self.element_count()? as usize));
        self.elements()?.read_element(index)
    }
}

impl<T> SchemaValue for CUtlVector<T> {
    fn value_size() -> Option<usize> {
        Some(0x10)
    }

    fn from_memory(memory: &Arc<dyn MemoryHandle>, offset: u64) -> anyhow::Result<Self> {
        Ok(Self {
            memory: memory.clone(),
            offset,
            _dummy: Default::default(),
        })
    }
}