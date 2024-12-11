use ash::Entry;
use libloading::Library;

use crate::VulkanError;

pub fn get_vulkan_entry() -> Result<ash::Entry, VulkanError> {
    unsafe {
        Library::new("CFGMGR32.dll").unwrap();
        Library::new("advapi32.dll").unwrap();
        Library::new("kernel32.dll").unwrap();
    }
    unsafe { Ok(Entry::load()?) }
}
