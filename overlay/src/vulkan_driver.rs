use ash::Entry;

use crate::Result;

pub fn get_vulkan_entry() -> Result<ash::Entry> {
    unsafe { Ok(Entry::load()?) }
}
