use std::{marker::PhantomData, fmt::Debug, sync::Arc, ffi::CStr};

use cs2_schema::{SchemaValue, MemoryHandle};

use crate::{CS2Handle, Module};



pub struct Ptr<T> {
    pub value: u64,
    _data: PhantomData<T>,
}
const _: [u8; 0x08] = [0; std::mem::size_of::<Ptr<()>>()];

impl<T> Ptr<T> {
    pub fn address(&self) -> u64 {
        self.value
    }
}

impl<T> Debug for Ptr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:X}", &self.value)
    }
}

impl<T> Default for Ptr<T> {
    fn default() -> Self {
        Self {
            value: 0,
            _data: Default::default(),
        }
    }
}

impl<T: Sized> Ptr<T> {
    pub fn try_read(&self, cs2: &CS2Handle) -> anyhow::Result<Option<T>> {
        if self.value == 0 {
            Ok(None)
        } else {
            Ok(Some(cs2.read::<T>(Module::Absolute, &[self.value])?))
        }
    }

    pub fn read(&self, cs2: &CS2Handle) -> anyhow::Result<T> {
        cs2.read::<T>(Module::Absolute, &[self.value])
    }
}

impl<T> SchemaValue for Ptr<T> {
    fn value_size() -> usize {
        0x08
    }

    fn from_memory(memory: &std::sync::Arc<dyn MemoryHandle>, offset: u64) -> anyhow::Result<Self> {
        let mut buffer = [0u8; 0x08];
        memory.read_slice(offset, &mut buffer)?;

        Ok(Self {
            _data: Default::default(),
            value: u64::from_le_bytes(buffer)
        })
    }
}

impl<T: SchemaValue> Ptr<T> {
    pub fn reference_schema(&self, cs2: &CS2Handle) -> anyhow::Result<T> {
        cs2.reference_schema(&[ self.value ])
    }

    pub fn read_schema(&self, cs2: &CS2Handle) -> anyhow::Result<T> {
        cs2.read_schema(&[ self.value ])
    }
}

impl Ptr<*const i8> {
    pub fn read_string(&self, cs2: &CS2Handle) -> anyhow::Result<String> {
        cs2.read_string(Module::Absolute, &[self.value], None)
    }

    pub fn try_read_string(&self, cs2: &CS2Handle) -> anyhow::Result<Option<String>> {
        if self.value == 0 {
            Ok(None)
        } else {
            Ok(Some(cs2.read_string(
                Module::Absolute,
                &[self.value],
                None,
            )?))
        }
    }
}

pub type PtrCStr = Ptr<*const i8>;
const _: [u8; 0x08] = [0; std::mem::size_of::<PtrCStr>()];


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
    fn value_size() -> usize {
        SIZE
    }

    fn from_memory(memory: &Arc<dyn MemoryHandle>, offset: u64) -> anyhow::Result<Self> {
        Ok(Self {
            memory: memory.clone(),
            offset,
        })
    }
}