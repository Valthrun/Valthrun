use crate::kdef::{_KAPC_STATE, _KPROCESS, KeUnstackDetachProcess, KeStackAttachProcess};

pub struct ProcessAttachGuard {
    _process: *const _KPROCESS,
    apc_state: _KAPC_STATE
}
impl Drop for ProcessAttachGuard {
    fn drop(&mut self) {
        unsafe { KeUnstackDetachProcess(&mut self.apc_state) };
    }
}

pub fn attach_process_stack<'a>(process: *const _KPROCESS) -> ProcessAttachGuard {
    let mut apc_state: _KAPC_STATE = unsafe { core::mem::zeroed() };
    unsafe { KeStackAttachProcess(process, &mut apc_state) };
    ProcessAttachGuard{
        _process: process,
        apc_state
    }
}