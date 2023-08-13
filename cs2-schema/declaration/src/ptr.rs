use std::{marker::PhantomData, fmt::Debug, ffi::CStr, sync::Arc};

use anyhow::Context;

use crate::{SchemaValue, MemoryHandle, MemoryDriver};

pub struct Ptr<T: ?Sized> {
    driver: Arc<dyn MemoryDriver>,
    address: u64,
    _data: PhantomData<T>,
}

impl<T: ?Sized> Ptr<T> {
    pub fn address(&self) -> anyhow::Result<u64> {
        Ok(self.address)
    }

    pub fn is_null(&self) -> anyhow::Result<bool> {
        Ok(self.address()? == 0)
    }

    pub fn cast<V>(self) -> Ptr<V> {
        Ptr::<V> {
            driver: self.driver,
            address: self.address,
            _data: Default::default()
        }
    }
}

impl<T> Debug for Ptr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:X}", self.address)
    }
}

impl<T: ?Sized> SchemaValue for Ptr<T> {
    fn value_size() -> Option<u64> {
        Some(0x08)
    }

    fn from_memory(memory: MemoryHandle) -> anyhow::Result<Self> {
        let address = memory.reference_schema(0x00)?;
        Ok(Self {
            driver: memory.driver,
            address,
            _data: Default::default(),
        })
    }
}

impl<T: SchemaValue> Ptr<T> {
    pub fn reference_schema(&self) -> anyhow::Result<T> {
        let memory = MemoryHandle::from_driver(&self.driver, self.address()?);
        T::from_memory(memory)
    }

    pub fn read_schema(&self) -> anyhow::Result<T> {
        let size = T::value_size().context("could not read a dynamic sized schema")?;

        let mut memory = MemoryHandle::from_driver(&self.driver, self.address()?);
        memory.cache(size as usize)?;
        T::from_memory(memory)
    }
    
    pub fn try_reference_schema(&self) -> anyhow::Result<Option<T>> {
        let address = self.address()?;
        if address > 0 {
            let memory = MemoryHandle::from_driver(&self.driver, address);
            Ok(Some(T::from_memory(memory)?))
        } else {
            Ok(None)
        }
    }

    pub fn try_read_schema(&self) -> anyhow::Result<Option<T>> {
        let size = T::value_size().context("could not read a dynamic sized schema")?;
        let address = self.address()?;
        if address > 0 {
            let mut memory = MemoryHandle::from_driver(&self.driver, address);
            memory.cache(size as usize)?;
            Ok(Some(T::from_memory(memory)?))
        } else {
            Ok(None)
        }
    }
}

/// Unbound array implementation
impl<T: SchemaValue> Ptr<[T]> {
    pub fn reference_element(&self, index: usize) -> anyhow::Result<T> {
        let size = T::value_size().context("could not read an array entry for a dynamic sized schema")?;
        let element_address = self.address()? + size * (index as u64);

        let memory = MemoryHandle::from_driver(&self.driver, element_address);
        T::from_memory(memory)
    }

    pub fn read_element(&self, index: usize) -> anyhow::Result<T> {
        let size = T::value_size().context("could not read an array entry for a dynamic sized schema")?;
        let element_address = self.address()? + size * (index as u64);

        let memory = MemoryHandle::from_driver(&self.driver, element_address);
        T::from_memory(memory)
    }

    pub fn read_entries(&self, length: usize) -> anyhow::Result<Vec<T>> {
        let element_size = T::value_size().context("could not read an array entry for a dynamic sized schema")? as usize;

        let mut memory = MemoryHandle::from_driver(&self.driver, self.address()?);
        memory.cache(element_size * length)?;

        let mut result = Vec::<T>::with_capacity(length);
        for index in 0..length {
            result.push(
                SchemaValue::from_memory(
                    memory.clone().with_offset((index * element_size) as u64)?
                )?
            );
        }

        Ok(result)
    }
}

pub type PtrCStr = Ptr<*const i8>;

pub struct FixedCString<const SIZE: usize> {
    memory: MemoryHandle,
}

impl<const SIZE: usize> FixedCString<SIZE> {
    pub fn to_string_lossy(&self) -> anyhow::Result<String> {
        let mut buffer = [0u8; SIZE];
        self.memory.read_slice(0x00, &mut buffer)?;

        let cstr = CStr::from_bytes_until_nul(&buffer)?;
        Ok(cstr.to_string_lossy().to_string())
    }
}

impl<const SIZE: usize> SchemaValue for FixedCString<SIZE> {
    fn value_size() -> Option<u64> {
        Some(SIZE as u64)
    }

    fn from_memory(memory: MemoryHandle) -> anyhow::Result<Self> {
        Ok(Self {
            memory,
        })
    }
}