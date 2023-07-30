use core::ffi::CStr;

use alloc::vec::Vec;
use valthrun_driver_shared::ModuleInfo;
use winapi::{km::wdm::PEPROCESS, shared::ntdef::{NT_SUCCESS, PLIST_ENTRY}};

use crate::{kdef::{KeUnstackDetachProcess, KeStackAttachProcess, _KAPC_STATE, PsLookupProcessByProcessId, ObfDereferenceObject, IoGetCurrentProcess, ObfReferenceObject, PsInitialSystemProcess, PsGetProcessPeb, _LDR_DATA_TABLE_ENTRY, PsGetProcessId}, get_windows_build_number};

use super::UnicodeStringEx;

#[derive(Debug, Clone)]
pub struct Process {
    eprocess: PEPROCESS,
}

impl Process {
    pub fn eprocess(&self) -> PEPROCESS {
        self.eprocess
    }

    pub fn from_raw(eprocess: PEPROCESS, owns_reference: bool) -> Self {
        if !owns_reference {
            unsafe {
                /* As we dereference the object when Process gets dropped we need to increase it here */
                ObfReferenceObject(eprocess);
            }
        }

        Self { eprocess }
    }

    pub fn current() -> Process {
        Self::from_raw(unsafe { IoGetCurrentProcess() }, false)
    }

    pub fn by_id(process_id: i32) -> Option<Self> {
        let mut process = core::ptr::null_mut();

        let status = unsafe { PsLookupProcessByProcessId(process_id as _, &mut process) };
        if NT_SUCCESS(status) {
            Some(Self { eprocess: process })
        } else {
            None
        }
    }

    pub fn get_id(&self) -> i32 {
        unsafe { PsGetProcessId(self.eprocess()) }
    }

    pub fn attach(&self) -> AttachedProcess {
        let mut apc_state: _KAPC_STATE = unsafe { core::mem::zeroed() };
        unsafe { KeStackAttachProcess(self.eprocess, &mut apc_state) };
        AttachedProcess{
            process: self,
            apc_state
        }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        if !self.eprocess.is_null() {
            unsafe { ObfDereferenceObject(self.eprocess as _) }
        }
    }
}


pub struct AttachedProcess<'a> {
    process: &'a Process,
    apc_state: _KAPC_STATE,
}

impl AttachedProcess<'_> {
    pub fn get_id(&self) -> i32 {
        self.process.get_id()
    }

    pub fn get_module(&self, name: &str) -> Option<ModuleInfo> {
        let peb = unsafe { 
            PsGetProcessPeb(self.process.eprocess())
                .as_ref()?
        };
        let ldr = match unsafe { peb.Ldr.as_ref() } {
            Some(ldr) => ldr,
            None => {
                log::warn!("missing process module list for {:X}", self.process.eprocess() as u64);
                return None;
            }
        };

        let mut current_entry = ldr.InLoadOrderModuleList.Flink as *const _;
        while current_entry != &ldr.InLoadOrderModuleList {
            let entry = unsafe {
                current_entry
                    .byte_offset(0) /* InLoadOrderLinks is the first entry */
                    .cast::<_LDR_DATA_TABLE_ENTRY>()
                    .read()
            };
            let base_name = entry.BaseDllName.as_string_lossy();
            if base_name == name {
                return Some(ModuleInfo{
                    base_address: entry.DllBase as u64,
                    module_size: entry.SizeOfImage as usize
                })
            }
            
            current_entry = unsafe { (*current_entry).Flink };
        }

        None
    }
}

impl Drop for AttachedProcess<'_> {
    fn drop(&mut self) {
        unsafe { KeUnstackDetachProcess(&mut self.apc_state) };
    }
}

pub fn find_processes_by_name(target_name: &str) -> anyhow::Result<Vec<Process>> {
    let build_number = get_windows_build_number()
        .map_err(|status| anyhow::anyhow!("RtlGetVersion {}", status))?;

    let offset_image_file_name;
    let offset_active_threads;
    let offset_active_process_links;

    match build_number {
        22621 => {
            offset_image_file_name = 0x5a8;
            offset_active_threads = 0x5f0;
            offset_active_process_links = 0x448;
        },
        _ => anyhow::bail!("missing EPROCESS offsets for build no {build_number}")
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
                .cast::<PLIST_ENTRY>()
                .read()
                .read()
                .Flink
                .byte_offset(-offset_active_process_links)
                .cast()
        };

        log::debug!("{:X}: {:?} ({}); Next {:X}", current_pep as u64, name, active_threads, next_pep as u64);
        if active_threads > 0 && name == Some(target_name) {
            cs2_candidates.push(
                Process::from_raw(current_pep, false)
            );
        }

        current_pep = next_pep;
        if current_pep == pep_system {
            break;
        }
    }

    Ok(cs2_candidates)
}