#![feature(array_try_from_fn)]

mod memory;
pub use memory::*;

mod ptr;
pub use ptr::*;

mod basics;
pub use basics::*;

pub trait SchemaValue : Sized {
    fn value_size() -> Option<u64>;
    fn from_memory(memory: MemoryHandle) -> anyhow::Result<Self>;
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
            fn value_size() -> Option<u64> {
                Some(std::mem::size_of::<$ordinal_type>() as u64)
            }

            fn from_memory(memory: cs2_schema_declaration::MemoryHandle) -> anyhow::Result<Self> {
                let value: $ordinal_type = cs2_schema_declaration::SchemaValue::from_memory(memory)?;
                let result = match value {
                    $($ordinal => Self::$vname,)*
                    value => anyhow::bail!("unknown enum ordinal {} {}", stringify!($name), value)
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
            pub memory: cs2_schema_declaration::MemoryHandle,
        }

        impl $name {
            $(
                $(#[$var_meta])*
                pub fn $var_name(&self) -> anyhow::Result<$var_type> {
                    use anyhow::Context;
        
                    self.memory.reference_schema($var_offset)
                        .context(concat!(stringify!($name), "::", stringify!($var_name)))
                }
            )*

            pub fn cached(self) -> anyhow::Result<Self> {
                use cs2_schema_declaration::SchemaValue;
                
                if $size <= 0 {
                    anyhow::bail!("can not cache a schema with zero size");
                }

                let mut memory = self.memory;
                memory.cache($size)?;

                Ok(Self {
                    $(parent: <$parent>::from_memory(memory.clone())?,)*
                    memory,
                })
            }

            pub fn as_schema<T: cs2_schema_declaration::SchemaValue>(&self) -> anyhow::Result<T> {
                self.memory.reference_schema(0x00)
            }
        }

        impl cs2_schema_declaration::SchemaValue for $name {
            fn value_size() -> Option<u64> {
                if $size > 0 {
                    Some($size)
                } else {
                    None
                }
            }

            fn from_memory(memory: cs2_schema_declaration::MemoryHandle) -> anyhow::Result<Self> {
                Ok(Self {
                    $(parent: <$parent>::from_memory(memory.clone())?,)*
                    memory,
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