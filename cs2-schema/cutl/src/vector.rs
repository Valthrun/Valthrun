use std::marker::PhantomData;

use cs2_schema_declaration::{
    MemoryHandle,
    Ptr,
    SchemaValue,
};

/// struct CUtlVector<T> {
///     pub size: u32, // 0x00
///     pub data: *const T // 0x08
/// }
pub struct CUtlVector<T> {
    memory: MemoryHandle,
    _dummy: PhantomData<T>,
}

impl<T> CUtlVector<T> {
    pub fn element_count(&self) -> anyhow::Result<i32> {
        self.memory.reference_schema(0x00)
    }

    pub fn elements(&self) -> anyhow::Result<Ptr<[T]>> {
        self.memory.reference_schema(0x08)
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
    fn value_size() -> Option<u64> {
        Some(0x10)
    }

    fn from_memory(memory: MemoryHandle) -> anyhow::Result<Self> {
        Ok(Self {
            memory,
            _dummy: Default::default(),
        })
    }
}
