use std::{
    fs::File,
    io::Write,
};

use ash::Entry;

use crate::{
    OverlayError,
    Result,
};

const DRIVER_BYTES: &[u8] = include_bytes!("../resources/vulkan-1.dll");

pub fn get_vulkan_entry() -> Result<ash::Entry> {
    let dll_path = std::env::current_exe()
        .map_err(OverlayError::ExePathInvalid)?
        .parent()
        .ok_or(OverlayError::ExePathMissingParentDirectory)?
        .join("vulkan-1.dll");

    if !dll_path.exists() {
        log::debug!("Writing vulkan-1.dll");
        let mut file = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&dll_path)
            .map_err(OverlayError::VulkanDllError)?;

        file.write_all(&DRIVER_BYTES)
            .map_err(OverlayError::VulkanDllError)?;
    }

    log::debug!("Loading vulkan-1.dll from {}", dll_path.to_string_lossy());
    unsafe { Ok(Entry::load_from(&dll_path)?) }
}
