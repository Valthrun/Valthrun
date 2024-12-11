use core::slice;
use std::mem;

use anyhow::Context;
use valthrun_driver_protocol::{
    command::{
        DriverCommandProcessList,
        DriverCommandProcessModules,
    },
    types::{
        ProcessId,
        ProcessInfo,
        ProcessModuleInfo,
    },
};
use windows::Win32::{
    Foundation::{
        HANDLE,
        HMODULE,
    },
    System::{
        ProcessStatus::{
            GetModuleBaseNameA,
            GetModuleInformation,
        },
        Threading::{
            PROCESS_QUERY_INFORMATION,
            PROCESS_VM_READ,
        },
    },
};

use crate::util::{
    self,
    list_process_modules,
    list_system_process_ids,
};

fn fill_process_info(process_id: ProcessId, output: &mut ProcessInfo) -> anyhow::Result<()> {
    let process = util::open_process_by_id(process_id, PROCESS_QUERY_INFORMATION | PROCESS_VM_READ)
        .context("open process")?;

    let modules = list_process_modules(&process, Some(1)).context("list modules")?;
    let main_module = modules.first().context("missing main module")?;

    let name_length = unsafe {
        GetModuleBaseNameA(
            process.raw_handle(),
            *main_module,
            &mut output.image_base_name,
        )
    } as usize;
    if name_length == 0 {
        anyhow::bail!("failed to get module base name");
    }

    if name_length < output.image_base_name.len() - 1 {
        output.image_base_name[name_length] = 0x00;
    }
    output.process_id = process_id;
    output.directory_table_base = 0;
    Ok(())
}

pub fn get_processes(command: &mut DriverCommandProcessList) -> anyhow::Result<()> {
    let processes = list_system_process_ids()?;
    let buffer = unsafe { slice::from_raw_parts_mut(command.buffer, command.buffer_capacity) };

    for process_id in processes {
        if let Some(output) = buffer.get_mut(command.process_count) {
            if let Err(err) = self::fill_process_info(process_id, output) {
                log::debug!("Failed to fill process info for {}: {}", process_id, err);
                continue;
            }
        }

        command.process_count += 1;
    }

    Ok(())
}

fn fill_module_info(
    hprocess: HANDLE,
    hmodule: HMODULE,
    output: &mut ProcessModuleInfo,
) -> anyhow::Result<()> {
    let bytes_copied =
        unsafe { GetModuleBaseNameA(hprocess, hmodule, &mut output.base_dll_name) } as usize;
    if bytes_copied == 0 {
        anyhow::bail!("GetModuleFileNameExA failed");
    }

    let mut module_info = Default::default();
    let success = unsafe {
        GetModuleInformation(
            hprocess,
            hmodule,
            &mut module_info,
            mem::size_of_val(&module_info) as u32,
        )
    };
    if !success.as_bool() {
        anyhow::bail!("GetModuleInformation failed");
    }

    output.module_size = module_info.SizeOfImage as u64;
    output.base_address = module_info.lpBaseOfDll as u64;
    Ok(())
}

pub fn get_modules(command: &mut DriverCommandProcessModules) -> anyhow::Result<()> {
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
            command.process_unknown = true;
            return Ok(());
        }
    };

    let modules = util::list_process_modules(&process, None)?;

    let module_buffer =
        unsafe { slice::from_raw_parts_mut(command.buffer, command.buffer_capacity) };

    command.process_unknown = false;
    for hmodule in modules.iter() {
        if let Some(output) = module_buffer.get_mut(command.module_count) {
            if let Err(err) = self::fill_module_info(process.raw_handle(), *hmodule, output) {
                log::debug!(
                    "Failed to fill process module info for {:X}: {}",
                    hmodule.0,
                    err
                );
                continue;
            }
        }

        command.module_count += 1;
    }

    Ok(())
}
