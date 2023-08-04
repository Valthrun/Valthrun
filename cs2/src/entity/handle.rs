use std::fmt::Debug;

use obfstr::obfstr;


/// CS2 32 bit entity handle packed with 
/// the entity index and serial number.
#[repr(C)]
#[derive(Default, Clone)]
pub struct EntityHandle {
    pub value: u32,
}

impl EntityHandle {
    pub fn get_entity_index(&self) -> u32 {
        self.value & 0x7FFF
    }

    pub fn is_valid(&self) -> bool {
        self.get_entity_index() < 0x7FF0
    }

    pub fn get_serial_number(&self) -> u32 {
        self.value >> 15
    }

    pub fn entity_array_offsets(&self) -> (u64, u64) {
        let entity_index = self.get_entity_index();
        ((entity_index >> 9) as u64, (entity_index & 0x1FF) as u64)
    }
}

impl Debug for EntityHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(obfstr!("EntityHandle"))
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