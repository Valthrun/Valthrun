use anyhow::Context;
use valthrun_driver_shared::requests::{RequestProtectionToggle, ResponseProtectionToggle};

use crate::{PROCESS_PROTECTION, kdef::{PsGetProcessId, IoGetCurrentProcess}};


pub fn handler_protection_toggle(req: &RequestProtectionToggle, _res: &mut ResponseProtectionToggle) -> anyhow::Result<()> {
    let process_protection = unsafe { &*PROCESS_PROTECTION.get() }
        .as_ref()
        .context("missing protection manager")?;

    let current_thread_id = unsafe { PsGetProcessId(IoGetCurrentProcess()) };
    process_protection.toggle_protection(current_thread_id, req.enabled);

    Ok(())
}