use std::sync::Arc;
use anyhow::Context;

use crate::{ SchemaValue, MemoryHandle };

macro_rules! prim_impl {
    ($type:ty) => {
        impl SchemaValue for $type {
            fn from_memory(memory: &Arc<dyn MemoryHandle>, offset: u64) -> anyhow::Result<$type> {
                let mut buffer = [0u8; std::mem::size_of::<$type>()];
                memory.read_slice(offset, &mut buffer)?;
    
                Ok(<$type>::from_le_bytes(buffer))
            }

            fn value_size() -> Option<usize> {
                Some(std::mem::size_of::<$type>())
            }
        }
    };
}

prim_impl!(i8);
prim_impl!(u8);

prim_impl!(i16);
prim_impl!(u16);

prim_impl!(i32);
prim_impl!(u32);

prim_impl!(i64);
prim_impl!(u64);

prim_impl!(f32);
prim_impl!(f64);

impl SchemaValue for bool {
    fn value_size() -> Option<usize> {
        Some(0x01)
    }

    fn from_memory(memory: &Arc<dyn MemoryHandle>, offset: u64) -> anyhow::Result<Self> {
        let mut buffer = [0u8; 1];
        memory.read_slice(offset, &mut buffer)?;

        Ok(buffer[0] > 0)
    }
}

impl<T: SchemaValue, const N: usize> SchemaValue for [T; N] {
    fn value_size() -> Option<usize> {
        Some(T::value_size()? * N)
    }

    fn from_memory(memory: &Arc<dyn MemoryHandle>, offset: u64) -> anyhow::Result<Self> {
        let element_size = T::value_size().context("fixed array can't have an unsized schema value")?;
        std::array::try_from_fn(|index| T::from_memory(memory, offset + (index * element_size) as u64))
    }
}