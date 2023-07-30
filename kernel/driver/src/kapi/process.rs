use winapi::km::wdm::PEPROCESS;

use crate::kdef::{KeUnstackDetachProcess, KeStackAttachProcess, _KAPC_STATE};

pub struct ProcessAttachGuard {
    _process: PEPROCESS,
    apc_state: _KAPC_STATE
}
impl Drop for ProcessAttachGuard {
    fn drop(&mut self) {
        unsafe { KeUnstackDetachProcess(&mut self.apc_state) };
    }
}

pub fn attach_process_stack<'a>(process: PEPROCESS) -> ProcessAttachGuard {
    let mut apc_state: _KAPC_STATE = unsafe { core::mem::zeroed() };
    unsafe { KeStackAttachProcess(process, &mut apc_state) };
    ProcessAttachGuard{
        _process: process,
        apc_state
    }
}