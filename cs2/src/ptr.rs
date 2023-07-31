use std::{marker::PhantomData, fmt::Debug};

use crate::{CS2Handle, Module};



#[repr(C)]
pub struct Ptr<T> {
    pub value: u64,
    _data: PhantomData<T>,
}
const _: [u8; 0x08] = [0; std::mem::size_of::<Ptr<()>>()];

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