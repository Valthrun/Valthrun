use anyhow::Context;
use valthrun_driver_shared::requests::{RequestProtectionToggle, ResponseProtectionToggle};
use winapi::{shared::ntdef::{UNICODE_STRING, PVOID}, km::wdm::PEPROCESS};

use crate::{PROCESS_PROTECTION, kdef::{PsGetProcessId, IoGetCurrentProcess, MmGetSystemRoutineAddress, ProcessProtectionInformation}, kapi::UnicodeStringEx};

/// Gets ta pointer to a function from ntoskrnl exports
fn get_ntoskrnl_exports(function_name: *const UNICODE_STRING) -> PVOID {
    //The MmGetSystemRoutineAddress routine returns a pointer to a function specified by SystemRoutineName.
    return unsafe { MmGetSystemRoutineAddress(function_name) };
}

// Gets function base address
pub fn get_function_base_address(function_name: *const UNICODE_STRING) -> PVOID {
    let base = get_ntoskrnl_exports(function_name);
    return base;
}

/// Get EPROCESS.SignatureLevel offset dynamically
pub fn get_eprocess_signature_level_offset() -> isize {
    let unicode_function_name = UNICODE_STRING::from_bytes(obfstr::wide!("PsGetProcessSignatureLevel\0"));
    
    let base_address = get_function_base_address(&unicode_function_name);
    let function_bytes: &[u8] = unsafe { core::slice::from_raw_parts(base_address as *const u8, 20) };

    let slice = &function_bytes[15..17];
    let signature_level_offset = u16::from_le_bytes(slice.try_into().unwrap());

    return signature_level_offset as isize;
}

/// Add process protection
pub fn protect_process(process: PEPROCESS) {
    let signature_level_offset = get_eprocess_signature_level_offset();
    let ps_protection = unsafe { 
        process.cast::<u8>()
            .offset(signature_level_offset)
            .cast::<ProcessProtectionInformation>()
    };

    unsafe { 
        (*ps_protection).signature_level = 0x3f;
        // We're loading DLLs on demand
        //(*ps_protection).section_signature_level = 0x3f;
        // TODO: Reenable as soon as protection has become optional.
        //       Protection type 2 hinters the user to forcefully terminate the application.
        // (*ps_protection).protection = PSProtection::new()
        //     .with_protection_type(2)
        //     .with_protection_audit(0)
        //     .with_protection_signer(6);
    }
}

pub fn handler_protection_toggle(req: &RequestProtectionToggle, _res: &mut ResponseProtectionToggle) -> anyhow::Result<()> {
    let process_protection = unsafe { &*PROCESS_PROTECTION.get() }
        .as_ref()
        .context("missing protection manager")?;

    let process = unsafe { IoGetCurrentProcess() };
    let current_thread_id = unsafe { PsGetProcessId(process) };
    process_protection.toggle_protection(current_thread_id, req.enabled);

    if req.enabled {
        protect_process(process);
    }

    Ok(())
}