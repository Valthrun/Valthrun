use anyhow::Context;

use crate::{
    MemoryHandle,
    SchemaValue,
};

macro_rules! prim_impl {
    ($type:ty) => {
        impl SchemaValue for $type {
            fn from_memory(memory: MemoryHandle) -> anyhow::Result<$type> {
                let mut buffer = [0u8; std::mem::size_of::<$type>()];
                memory.read_slice(0x00, &mut buffer)?;

                Ok(<$type>::from_le_bytes(buffer))
            }

            fn value_size() -> Option<u64> {
                Some(std::mem::size_of::<$type>() as u64)
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
    fn value_size() -> Option<u64> {
        Some(0x01)
    }

    fn from_memory(memory: MemoryHandle) -> anyhow::Result<Self> {
        let mut buffer = [0u8; 1];
        memory.read_slice(0x00, &mut buffer)?;

        Ok(buffer[0] > 0)
    }
}

impl<T: SchemaValue, const N: usize> SchemaValue for [T; N] {
    fn value_size() -> Option<u64> {
        Some(T::value_size()? * N as u64)
    }

    fn from_memory(mut memory: MemoryHandle) -> anyhow::Result<Self> {
        let element_size =
            T::value_size().context("fixed array can't have an unsized schema value")?;
        memory.cache(element_size as usize * N)?;

        std::array::try_from_fn(|index| {
            let memory = memory.clone().with_offset((index as u64) * element_size)?;
            T::from_memory(memory)
        })
    }
}
