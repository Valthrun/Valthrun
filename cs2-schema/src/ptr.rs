use std::{marker::PhantomData, fmt::Debug, sync::Arc, ffi::CStr};

use anyhow::Context;

use crate::{SchemaValue, MemoryHandle};

pub struct Ptr<T: ?Sized> {
    memory: Arc<dyn MemoryHandle>,
    offset: u64,

    _data: PhantomData<T>,
}

impl<T: ?Sized> Ptr<T> {
    pub fn address(&self) -> anyhow::Result<u64> {
        SchemaValue::from_memory(&self.memory, self.offset + 0x00)
    }

    pub fn is_null(&self) -> anyhow::Result<bool> {
        Ok(self.address()? == 0)
    }

    pub fn cast<V>(&self) -> Ptr<V> {
        Ptr::<V> {
            memory: self.memory.clone(),
            offset: self.offset,

            _data: Default::default()
        }
    }
}

impl<T> Debug for Ptr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:X}", &self.address().unwrap_or(0xFFFFFFFFFFFFFFFF))
    }
}

impl<T: ?Sized> SchemaValue for Ptr<T> {
    fn value_size() -> Option<usize> {
        Some(0x08)
    }

    fn from_memory(memory: &Arc<dyn MemoryHandle>, offset: u64) -> anyhow::Result<Self> {
        Ok(Self {
            memory: memory.clone(),
            offset,

            _data: Default::default(),
        })
    }
}

impl<T: SchemaValue> Ptr<T> {
    pub fn reference_schema(&self) -> anyhow::Result<T> {
        T::from_memory(&self.memory.reference_memory(self.address()?, T::value_size())?, 0x00)
    }

    pub fn read_schema(&self) -> anyhow::Result<T> {
        let size = T::value_size().context("could not read a dynamic sized schema")?;
        T::from_memory(&self.memory.read_memory(self.address()?, size)?, 0x00)
    }
}

/// Unbound array implementation
impl<T: SchemaValue> Ptr<[T]> {
    pub fn reference_element(&self, index: usize) -> anyhow::Result<T> {
        let size = T::value_size().context("could not read an array entry for a dynamic sized schema")?;
        let element_address = self.address()? + (size * index) as u64;

        T::from_memory(&self.memory.reference_memory(element_address, Some(size))?, 0x00)
    }

    pub fn read_element(&self, index: usize) -> anyhow::Result<T> {
        let size = T::value_size().context("could not read an array entry for a dynamic sized schema")?;
        let element_address = self.address()? + (size * index) as u64;

        T::from_memory(&self.memory.read_memory(element_address, size)?, 0x00)
    }

    pub fn read_entries(&self, length: usize) -> anyhow::Result<Vec<T>> {
        let element_size = T::value_size().context("could not read an array entry for a dynamic sized schema")?;
        let memory = self.memory.read_memory(self.address()?, element_size * length)?;

        let mut result = Vec::with_capacity(length);
        for index in 0..length {
            result.push(
                SchemaValue::from_memory(&memory, (element_size * index) as u64)?
            );
        }

        Ok(result)
    }
}

pub type PtrCStr = Ptr<*const i8>;

pub struct FixedCString<const SIZE: usize> {
    memory: Arc<dyn MemoryHandle>,
    offset: u64
}

impl<const SIZE: usize> FixedCString<SIZE> {
    pub fn to_string_lossy(&self) -> anyhow::Result<String> {
        let mut buffer = [0u8; SIZE];
        self.memory.read_slice(self.offset, &mut buffer)?;

        let cstr = CStr::from_bytes_until_nul(&buffer)?;
        Ok(cstr.to_string_lossy().to_string())
    }
}

impl<const SIZE: usize> SchemaValue for FixedCString<SIZE> {
    fn value_size() -> Option<usize> {
        Some(SIZE)
    }

    fn from_memory(memory: &Arc<dyn MemoryHandle>, offset: u64) -> anyhow::Result<Self> {
        Ok(Self {
            memory: memory.clone(),
            offset,
        })
    }
}