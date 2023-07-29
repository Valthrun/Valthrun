//! Deferred Procedure Calls (DPC).

use super::{_LIST_ENTRY, KSPIN_LOCK};

extern "system"
{
	pub fn KeInitializeDpc(Dpc: *mut KDPC, DeferredRoutine: PDEFERRED_ROUTINE, DeferredContext: *mut u8);
	pub fn KeInsertQueueDpc(Dpc: *mut KDPC, SystemArgument1: *const u8, SystemArgument2: *const u8) -> bool;
	pub fn KeRemoveQueueDpc(Dpc: *mut KDPC) -> bool;
	pub fn KeFlushQueuedDpcs();
	pub fn KeGenericCallDpc(DeferredRoutine: PDEFERRED_ROUTINE, DeferredContext: *mut u8);
}

pub type PDEFERRED_ROUTINE = extern "system" fn (Dpc: *const KDPC, DeferredContext: *mut u8, SystemArgument1: *const u8, SystemArgument2: *const u8);

/// Deferred Procedure Call object.
#[repr(C)]
pub struct KDPC
{
	Type: u8,
	Number: u8,
	Importance: u8,

	DpcListEntry: _LIST_ENTRY,
	DeferredRoutine: PDEFERRED_ROUTINE,
	DeferredContext: *mut u8,
	SystemArgument1: *mut u8,
	SystemArgument2: *mut u8,

	DpcData: *mut KDPC_DATA,
}

/// DPC data structure definition.
#[repr(C)]
pub struct KDPC_DATA
{
	DpcListHead: _LIST_ENTRY,
	DpcLock: KSPIN_LOCK,
	DpcQueueDepth: i32,
	DpcCount: u32,
}
