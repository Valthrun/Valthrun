#![feature(array_try_from_fn)]

use std::sync::Arc;

mod memory;
pub use memory::*;

mod ptr;
pub use ptr::*;

mod basics;
pub use basics::*;

pub trait SchemaValue : Sized {
    fn value_size() -> Option<usize>;
    fn from_memory(memory: &Arc<dyn MemoryHandle>, offset: u64) -> anyhow::Result<Self>;
}

#[macro_export]
macro_rules! define_schema {
    () => {};

    (pub enum $name:ident : $ordinal_type:ty { $($vname:ident = $ordinal:literal,)* } $($next:tt)*) => {
        #[derive(Debug, Copy, Clone)]
        pub enum $name {
            $($vname,)*
        }

        impl cs2_schema_declaration::SchemaValue for $name {
            fn value_size() -> Option<usize> {
                Some(std::mem::size_of::<$ordinal_type>())
            }

            fn from_memory(memory: &std::sync::Arc<dyn cs2_schema_declaration::MemoryHandle>, offset: u64) -> anyhow::Result<Self> {
                let value: $ordinal_type = cs2_schema_declaration::SchemaValue::from_memory(memory, offset)?;
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
            pub memory: std::sync::Arc<dyn cs2_schema_declaration::MemoryHandle>,
        }

        impl $name {
            $(
                $(#[$var_meta])*
                pub fn $var_name(&self) -> anyhow::Result<$var_type> {
                    use anyhow::Context;
        
                    cs2_schema_declaration::SchemaValue::from_memory(&self.memory, self.offset + $var_offset)
                        .context(concat!(stringify!($name), "::", stringify!($var_name)))
                }
            )*

            pub fn as_schema<T: cs2_schema_declaration::SchemaValue>(&self) -> anyhow::Result<T> {
                cs2_schema_declaration::SchemaValue::from_memory(&self.memory, self.offset)
            }
        }

        impl cs2_schema_declaration::SchemaValue for $name {
            fn value_size() -> Option<usize> {
                if $size > 0 {
                    Some($size)
                } else {
                    None
                }
            }

            fn from_memory(memory: &std::sync::Arc<dyn cs2_schema_declaration::MemoryHandle>, offset: u64) -> anyhow::Result<Self> {
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