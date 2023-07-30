use winapi::{shared::ntdef::PVOID, km::wdm::{KeInitializeEvent, KEVENT, SynchronizationEvent}};

#[repr(C)]
pub struct _FAST_MUTEX {
    Count: i32,
    Owner: PVOID,
    Contention: u32,
    Event: KEVENT,
    OldIrql: u32,
}

pub unsafe fn ExInitializeFastMutex(FastMutex: &mut _FAST_MUTEX) {
    FastMutex.Count = 1;
    FastMutex.Owner = core::ptr::null_mut();
    FastMutex.Contention = 0;
    KeInitializeEvent(&mut FastMutex.Event, SynchronizationEvent as u32, false);
}

#[link(name = "ntoskrnl")]
extern "system" {
    pub fn ExAcquireFastMutex(FastMutex: *mut _FAST_MUTEX);
    pub fn ExReleaseFastMutex(FastMutex: *mut _FAST_MUTEX);
    pub fn ExTryToAcquireFastMutex(FastMutex: *mut _FAST_MUTEX) -> i32;
}