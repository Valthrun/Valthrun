use std::{
    fmt::Debug,
    hash::{
        Hash,
        Hasher,
    },
    marker::PhantomData,
};

/// CS2 32 bit entity handle packed with
/// the entity index and serial number.
#[repr(C)]
#[derive(Default)]
pub struct EntityHandle<T: ?Sized> {
    pub value: u32,
    _data: PhantomData<T>,
}

impl<T: ?Sized> Clone for EntityHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: ?Sized> Copy for EntityHandle<T> {}

impl<T: ?Sized> EntityHandle<T> {
    pub fn from_index(index: u32) -> Self {
        Self {
            value: index,
            _data: Default::default(),
        }
    }

    pub fn get_entity_index(&self) -> u32 {
        self.value & 0x7FFF
    }

    pub fn is_valid(&self) -> bool {
        self.get_entity_index() < 0x7FF0
    }

    pub fn get_serial_number(&self) -> u32 {
        self.value >> 15
    }
}

impl<T: ?Sized> Debug for EntityHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntityHandle")
            .field(
                "entity_index",
                &format_args!("0x{:X}", &self.get_entity_index()),
            )
            .field(
                "serial_number",
                &format_args!("0x{:X}", &self.get_serial_number()),
            )
            .finish()
    }
}

impl<T: ?Sized> PartialEq for EntityHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get_entity_index() == other.get_entity_index()
    }
}

impl<T: ?Sized> Hash for EntityHandle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_entity_index().hash(state);
    }
}

pub type CEntityIndex = u32;
