use ash::Entry;
use libloading::Library;

use crate::Result;

pub fn get_vulkan_entry() -> Result<ash::Entry> {
    unsafe {
        Library::new("CFGMGR32.dll").unwrap();
        Library::new("advapi32.dll").unwrap();
        Library::new("kernel32.dll").unwrap();
    }
    unsafe { Ok(Entry::load()?) }
}
