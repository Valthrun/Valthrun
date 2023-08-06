use std::{sync::Arc, any::Any};

use anyhow::Context;

pub trait MemoryHandle : Any  {
    fn as_any(&self) -> &dyn Any;
    fn read_slice(&self, offset: u64, slice: &mut [u8]) -> anyhow::Result<()>;

    fn reference_memory(&self, address: u64, length: Option<usize>) -> anyhow::Result<Arc<dyn MemoryHandle>>;
    fn read_memory(&self, address: u64, length: usize) -> anyhow::Result<Arc<dyn MemoryHandle>>;
}

pub trait SchemaValue : Sized {
    fn value_size() -> Option<usize>;
    fn from_memory(memory: &Arc<dyn MemoryHandle>, offset: u64) -> anyhow::Result<Self>;
}

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

// FIXME: Add support for $(#[$meta:meta])* (incl. doc string)
#[macro_export]
macro_rules! define_schema {
    () => {};

    (pub enum $name:ident : $ordinal_type:ty { $($vname:ident = $ordinal:literal,)* } $($next:tt)*) => {
        #[derive(Debug, Copy, Clone)]
        pub enum $name {
            $($vname,)*
        }

        impl SchemaValue for $name {
            fn value_size() -> Option<usize> {
                Some(std::mem::size_of::<$ordinal_type>())
            }

            fn from_memory(memory: &std::sync::Arc<dyn MemoryHandle>, offset: u64) -> anyhow::Result<Self> {
                let value: $ordinal_type = SchemaValue::from_memory(memory, offset)?;
                let result = match value {
                    $($ordinal => Self::$vname,)*
                    value => anyhow::bail!("unknown $name {}", value)
                };
                Ok(result)
            }
        } 
        
        define_schema!($($next)*);
    };
    
    (
        pub struct $name:ident[$size:literal] $(: $parent:ty)? {
            $( $(#[$var_meta:meta])* pub $var_name:ident: $var_type:ty = $var_offset:literal, )*
        } $($next:tt)*
    ) => {
        #[derive(Clone)]
        pub struct $name {
            $(parent: $parent,)*
            pub offset: u64,
            pub memory: std::sync::Arc<dyn MemoryHandle>,
        }

        impl $name {
            $(
                $(#[$var_meta])*
                pub fn $var_name(&self) -> anyhow::Result<$var_type> {
                    use anyhow::Context;
        
                    SchemaValue::from_memory(&self.memory, self.offset + $var_offset)
                        .context(concat!(stringify!($cname), "::", stringify!($var_name)))
                }
            )*

            pub fn as_schema<T: SchemaValue>(&self) -> anyhow::Result<T> {
                SchemaValue::from_memory(&self.memory, self.offset)
            }
        }

        impl SchemaValue for $name {
            fn value_size() -> Option<usize> {
                if $size > 0 {
                    Some($size)
                } else {
                    None
                }
            }

            fn from_memory(memory: &std::sync::Arc<dyn MemoryHandle>, offset: u64) -> anyhow::Result<Self> {
                Ok(Self {
                    $(parent: <$parent>::from_memory(memory, offset)?,)*
                    offset,
                    memory: memory.clone(),
                })
            }
        }

        $(
            impl std::ops::Deref for $name {
                type Target = $parent;

                fn deref(&self) -> &Self::Target {
                    &self.parent
                }
            }
        )*
        
        define_schema!($($next)*);
    };
}