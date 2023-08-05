use std::{sync::Arc, any::Any};
pub trait SchemaValue : Sized {
    fn value_size() -> usize;
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

            fn value_size() -> usize {
                std::mem::size_of::<$type>()
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

pub trait MemoryHandle : Any  {
    fn as_any(&self) -> &dyn Any;
    fn read_slice(&self, offset: u64, slice: &mut [u8]) -> anyhow::Result<()>;
}

#[macro_export]
macro_rules! define_schema {
    (@var) => {};

    (@var pub $var_name:ident: $var_type:ty = $var_offset:literal $(, $($next:tt)*)?) => {
        pub fn $var_name(&self) -> anyhow::Result<$var_type> {
            SchemaValue::from_memory(&self.memory, self.offset + $var_offset)
        }

        $(
            define_schema!(@var $($next)*);
        )*
    };
    
    ($(pub struct $name:ident[$size:literal] $(: $parent:ident)? { $($contents:tt)* })*) => {$(
        pub struct $name {
            $(parent: $parent,)*
            pub offset: u64,
            pub memory: std::sync::Arc<dyn MemoryHandle>,
        }

        impl $name {
            define_schema!(@var $($contents)*);
        }

        impl SchemaValue for $name {
            fn value_size() -> usize {
                $size
            }

            fn from_memory(memory: &Arc<dyn MemoryHandle>, offset: u64) -> anyhow::Result<Self> {
                Ok(Self {
                    $(parent: $parent::from_memory(memory, offset)?,)*
                    offset,
                    memory: memory.clone(),
                })
            }
        }

        $(
            impl Deref for $name {
                type Target = $parent;

                fn deref(&self) -> &Self::Target {
                    &self.parent
                }
            }
        )*
    )*};
}