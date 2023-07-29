use core::ffi::CStr;

use alloc::vec::Vec;
use anyhow::{anyhow, Context};
use valthrun_driver_shared::{ModuleInfo, requests::{RequestCSModule, ResponseCsModule}, CSModuleInfo};

use crate::{get_windows_build_number, kapi::{attach_process_stack, UnicodeStringEx}, kdef::{_KPROCESS, PsInitialSystemProcess, _LIST_ENTRY, PsGetProcessPeb, _LDR_DATA_TABLE_ENTRY, PsGetProcessId}};


fn get_cs2_process() -> anyhow::Result<Vec<*const _KPROCESS>> {
    let build_number = get_windows_build_number()
        .map_err(|status| anyhow!("RtlGetVersion {}", status))?;
    log::trace!("Build No {build_number}");

    let offset_image_file_name;
    let offset_active_threads;
    let offset_active_process_links;

    match build_number {
        22621 => {
            offset_image_file_name = 0x5a8;
            offset_active_threads = 0x5f0;
            offset_active_process_links = 0x448;
        },
        _ => anyhow::bail!("TODO: implement PEP offsets")
    }

    let pep_system = unsafe { PsInitialSystemProcess };
    let mut current_pep = pep_system;

    let mut cs2_candidates = Vec::with_capacity(8);
    loop {
        let image_file_name = unsafe {
            current_pep.byte_offset(offset_image_file_name)
                .cast::<[u8; 15]>()
                .read()
        };

        let name = CStr::from_bytes_until_nul(image_file_name.as_slice())
            .map(|value| value.to_str().ok())
            .ok()
            .flatten();

            
        let active_threads = unsafe {
            current_pep.byte_offset(offset_active_threads)
                .cast::<u32>()
                .read()
        };

        let next_pep = unsafe {
            /*
            * 1. current_pep->ActiveProcessLinks
            * 2. ActiveProcessLinks->Flink
            * 3. next_pep = Flink - offset_active_process_links
            */
            current_pep.byte_offset(offset_active_process_links)
                .cast::<*const _LIST_ENTRY>()
                .read()
                .read()
                .Flink
                .byte_offset(-offset_active_process_links)
                .cast()
        };

        //log::debug!("{:X}: {:?} ({}); Next {:X}", current_pep as u64, name, active_threads, next_pep as u64);
        if active_threads > 0 && name == Some("cs2.exe") {
            cs2_candidates.push(current_pep);
        }

        current_pep = next_pep;
        if current_pep == pep_system {
            break;
        }
    }

    Ok(cs2_candidates)
}


fn get_process_module(process: *const _KPROCESS, name: &str) -> anyhow::Result<Option<ModuleInfo>> {
    let _attach_guard = attach_process_stack(process);
    let peb = unsafe { 
        PsGetProcessPeb(process)
            .as_ref() 
            .context("missing pep32 for process")?
    };
    let ldr = match unsafe { peb.Ldr.as_ref() } {
        Some(ldr) => ldr,
        None => anyhow::bail!("missing process module list")
    };

    let mut current_entry = ldr.InLoadOrderModuleList.Flink;
    while current_entry != &ldr.InLoadOrderModuleList {
        let entry = unsafe {
            current_entry
                .byte_offset(0) /* InLoadOrderLinks is the first entry */
                .cast::<_LDR_DATA_TABLE_ENTRY>()
                .read()
        };
        let base_name = entry.BaseDllName.as_string_lossy();
        if base_name == name {
            return Ok(Some(ModuleInfo{
                base_address: entry.DllBase as u64,
                module_size: entry.SizeOfImage as usize
            }))
        }
        
        current_entry = unsafe { (*current_entry).Flink };
    }

    drop(_attach_guard);
    Ok(None)
}

pub fn handler_get_modules(_req: &RequestCSModule, res: &mut ResponseCsModule) -> anyhow::Result<()> {
    log::debug!("Searching for CS process.");
    let cs2_process_candidates = get_cs2_process()?;
    let cs2_process = match cs2_process_candidates.len() {
        0 => {
            *res = ResponseCsModule::NoProcess;
            return Ok(());
        },
        1 => {
            *cs2_process_candidates.first().unwrap()
        },
        count => {
            *res = ResponseCsModule::UbiquitousProcesses(count);
            return Ok(());
        }
    };
    
    let cs2_process_id = unsafe { PsGetProcessId(cs2_process) };
    log::trace!("CS2 process id {}. PEP at {:X}", cs2_process_id, cs2_process as u64);

    let mut module_info = CSModuleInfo{
        ..Default::default()
    };

    module_info.process_id = cs2_process_id;
    module_info.client = get_process_module(cs2_process, "client.dll")
        .map(|result| result.context("missing client.dll"))
        .flatten()?;

    module_info.engine = get_process_module(cs2_process, "engine2.dll")
        .map(|result| result.context("missing engine2.dll"))
        .flatten()?;

    module_info.schemasystem = get_process_module(cs2_process, "schemasystem.dll")
        .map(|result| result.context("missing schemasystem.dll"))
        .flatten()?;

    *res = ResponseCsModule::Success(module_info);
    Ok(())
}
