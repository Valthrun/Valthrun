use std::mem;

use valthrun_driver_protocol::types::ProcessId;
use windows::{
    core::Error,
    Win32::{
        Foundation::HMODULE,
        System::{
            ProcessStatus::{
                EnumProcessModules,
                EnumProcesses,
            },
            Threading::{
                OpenProcess,
                PROCESS_ACCESS_RIGHTS,
            },
        },
    },
};

use crate::handle::OwnedHandle;

pub fn open_process_by_id(id: u32, access: PROCESS_ACCESS_RIGHTS) -> Result<OwnedHandle, Error> {
    unsafe {
        match OpenProcess(access, false, id) {
            Ok(handle) => Ok(OwnedHandle::from_raw_handle(handle)),
            Err(err) => Err(err),
        }
    }
}

pub fn list_process_modules(
    process: &OwnedHandle,
    max_modules: Option<usize>,
) -> anyhow::Result<Vec<HMODULE>> {
    let mut modules = Vec::new();
    modules.resize(max_modules.unwrap_or(1000), HMODULE::default());

    loop {
        let mut bytes_needed = 0;
        let success = unsafe {
            EnumProcessModules(
                process.raw_handle(),
                modules.as_mut_ptr(),
                (modules.len() * mem::size_of::<HMODULE>()) as u32,
                &mut bytes_needed,
            )
        };
        if !success.as_bool() {
            anyhow::bail!("EnumProcessModules");
        }

        let module_count = bytes_needed as usize / mem::size_of::<HMODULE>();
        if max_modules.is_none() && module_count == modules.len() {
            modules.resize(modules.len() * 2, HMODULE::default());
            continue;
        }

        unsafe { modules.set_len(module_count) };
        return Ok(modules);
    }
}

pub fn list_system_process_ids() -> anyhow::Result<Vec<ProcessId>> {
    let mut processes = Vec::new();
    processes.resize(1000, u32::default());

    loop {
        let mut bytes_needed = 0;
        let success = unsafe {
            EnumProcesses(
                processes.as_mut_ptr(),
                (processes.len() * mem::size_of::<u32>()) as u32,
                &mut bytes_needed,
            )
        };
        if !success.as_bool() {
            anyhow::bail!("EnumProcesses")
        }

        let process_count = bytes_needed as usize / mem::size_of::<u32>();
        if process_count == processes.len() {
            processes.resize(processes.len() * 2, u32::default());
            continue;
        }

        unsafe { processes.set_len(process_count) };
        return Ok(processes);
    }
}
