use core::{
    slice,
    str,
};
use std::mem;

use valthrun_driver_protocol::{
    command::{
        DriverCommandProcessModules,
        ProcessModulesResult,
    },
    types::ProcessFilter,
};
use windows::Win32::System::{
    ProcessStatus::{
        GetModuleBaseNameA,
        GetModuleInformation,
    },
    Threading::{
        PROCESS_QUERY_INFORMATION,
        PROCESS_VM_READ,
    },
};

use crate::util::{
    self,
    list_process_modules,
    list_system_process_ids,
    ProcessId,
};

fn find_process_id_by_name(name: &str) -> anyhow::Result<Vec<ProcessId>> {
    let processes = list_system_process_ids()?;
    let mut matching_ids = Vec::new();
    for process_id in processes {
        let Ok(process) =
            util::open_process_by_id(process_id, PROCESS_QUERY_INFORMATION | PROCESS_VM_READ)
        else {
            continue;
        };

        let modules = list_process_modules(&process, Some(1))?;
        let Some(main_module) = modules.first() else {
            continue;
        };

        let mut name_buffer = [0; 0xFF];
        let name_length =
            unsafe { GetModuleBaseNameA(process.raw_handle(), *main_module, &mut name_buffer) };
        if name_length == 0 {
            continue;
        }

        let process_name =
            unsafe { str::from_utf8_unchecked(&name_buffer[0..name_length as usize]) };
        if process_name == name {
            matching_ids.push(process_id);
        }
    }

    Ok(matching_ids)
}

pub fn get_modules(command: &mut DriverCommandProcessModules) -> anyhow::Result<()> {
    command.process_id = match command.target_process {
        ProcessFilter::None => {
            command.result = ProcessModulesResult::ProcessUnknown;
            return Ok(());
        }
        ProcessFilter::Id { id } => id as u32,
        ProcessFilter::ImageBaseName { name, name_length } => {
            let name = unsafe { slice::from_raw_parts(name, name_length) };
            let name = String::from_utf8_lossy(name);

            let process_ids = find_process_id_by_name(&*name)?;
            if process_ids.is_empty() {
                command.result = ProcessModulesResult::ProcessUnknown;
                return Ok(());
            } else if process_ids.len() > 1 {
                command.result = ProcessModulesResult::ProcessUbiquitous;
                return Ok(());
            }

            process_ids[0]
        }
    };
    let process = match util::open_process_by_id(
        command.process_id,
        PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
    ) {
        Ok(handle) => handle,
        Err(err) => {
            log::warn!(
                "Failed to open process {} for enumeration: {}",
                command.process_id,
                err
            );
            command.result = ProcessModulesResult::ProcessUnknown;
            return Ok(());
        }
    };

    let modules = util::list_process_modules(&process, None)?;
    log::debug!("Process module count: {}", modules.len());

    command.module_count = modules.len();
    if modules.len() > command.module_buffer_length {
        command.result = ProcessModulesResult::BufferTooSmall;
        return Ok(());
    }

    let module_buffer =
        unsafe { slice::from_raw_parts_mut(command.module_buffer, command.module_buffer_length) };

    let mut module_buffer_index = 0;
    for hmodule in modules.iter() {
        let bytes_copied = unsafe {
            GetModuleBaseNameA(
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

        module_buffer[module_buffer_index].module_size = module_info.SizeOfImage as u64;
        module_buffer[module_buffer_index].base_address = module_info.lpBaseOfDll as u64;
        module_buffer_index += 1;
    }

    command.result = ProcessModulesResult::Success;
    Ok(())
}
