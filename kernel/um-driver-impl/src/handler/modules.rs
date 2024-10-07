use core::slice;
use std::mem;

use valthrun_driver_shared::{
    requests::{
        ProcessFilter,
        RequestCSModule,
        RequestProcessModules,
        ResponseProcessModules,
    },
    ProcessModuleInfo,
};
use windows::Win32::{
    Foundation::HMODULE,
    System::{
        ProcessStatus::{
            EnumProcessModules,
            GetModuleFileNameExA,
            GetModuleInformation,
        },
        Threading::{
            PROCESS_QUERY_INFORMATION,
            PROCESS_VM_READ,
        },
    },
};

use crate::util;

pub fn get_cs2_modules(
    req: &RequestCSModule,
    res: &mut ResponseProcessModules,
) -> anyhow::Result<()> {
    let process_name = "cs2.exe";
    self::get_modules(
        &RequestProcessModules {
            filter: ProcessFilter::Name {
                name: process_name.as_ptr(),
                name_length: process_name.len(),
            },
            module_buffer: req.module_buffer,
            module_buffer_length: req.module_buffer_length,
        },
        res,
    )
}

pub fn get_modules(
    req: &RequestProcessModules,
    res: &mut ResponseProcessModules,
) -> anyhow::Result<()> {
    let process = match req.filter {
        ProcessFilter::Id { id } => {
            match util::open_process_by_id(id as u32, PROCESS_QUERY_INFORMATION | PROCESS_VM_READ) {
                Ok(handle) => handle,
                Err(err) => {
                    log::warn!("Failed to open process {} for enumeration: {}", id, err);
                    *res = ResponseProcessModules::NoProcess;
                    return Ok(());
                }
            }
        }
        ProcessFilter::Name { .. } => anyhow::bail!("not supported"),
    };

    let modules = unsafe {
        let mut modules = Vec::new();
        modules.resize(1000, HMODULE::default());

        let mut bytes_needed = 0;
        let success = EnumProcessModules(
            process.raw_handle(),
            modules.as_mut_ptr(),
            (modules.len() * mem::size_of::<HMODULE>()) as u32,
            &mut bytes_needed,
        );
        if !success.as_bool() {
            anyhow::bail!("EnumProcessModules failed");
        }

        modules.set_len(bytes_needed as usize / mem::size_of::<HMODULE>());
        modules
    };
    log::debug!("Process module count: {}", modules.len());

    if modules.len() > req.module_buffer_length {
        *res = ResponseProcessModules::BufferTooSmall {
            expected: modules.len(),
        };
        return Ok(());
    }

    let module_buffer =
        unsafe { slice::from_raw_parts_mut(req.module_buffer, req.module_buffer_length) };

    let mut module_buffer_index = 0;
    for hmodule in modules.iter() {
        let bytes_copied = unsafe {
            GetModuleFileNameExA(
                process.raw_handle(),
                *hmodule,
                &mut module_buffer[module_buffer_index].base_dll_name,
            )
        };
        if bytes_copied == 0 {
            log::warn!(
                "Skipping process module {:X} as GetModuleFileNameExA failed",
                hmodule.0
            );
            continue;
        }

        let mut module_info = Default::default();
        let success = unsafe {
            GetModuleInformation(
                process.raw_handle(),
                *hmodule,
                &mut module_info,
                mem::size_of_val(&module_info) as u32,
            )
        };
        if !success.as_bool() {
            log::warn!(
                "Skipping process module {:X} as GetModuleInformation failed",
                hmodule.0
            );
            continue;
        }

        module_buffer[module_buffer_index].module_size = module_info.SizeOfImage as usize;
        module_buffer[module_buffer_index].base_address = module_info.lpBaseOfDll as usize;
        module_buffer_index += 1;
    }

    *res = ResponseProcessModules::Success(ProcessModuleInfo {
        module_count: module_buffer_index,
        process_id: 0,
    });
    Ok(())
}
