//! Kernel Objects.

use crate::kapi::NTSTATUS;

use super::{PVOID, KPROCESSOR_MODE, PDEVICE_OBJECT, IRP, _LIST_ENTRY, KSPIN_LOCK};

extern "system"
{
	pub fn KeWaitForSingleObject(Object: PVOID, WaitReason: u32, WaitMode: KPROCESSOR_MODE, Alertable: bool, Timeout: Option<&i64>) -> NTSTATUS;
}

#[repr(C)]
pub struct WAIT_CONTEXT_BLOCK
{
	WaitQueueEntry: *mut KDEVICE_QUEUE_ENTRY,
	DeviceRoutine: extern "system" fn (_obj: PDEVICE_OBJECT, _irp: *mut IRP, *mut u8, *mut u8) -> IO_ALLOCATION_ACTION,
	DeviceContext: *mut u8,
	NumberOfMapRegisters: u32,
	DeviceObject: *mut u8,
	CurrentIrp: *mut u8,
	BufferChainingDpc: * mut u8,
}

#[repr(C)]
pub enum IO_ALLOCATION_ACTION
{
	KeepObject = 0x01,
	DeallocateObject = 0x02,
	DeallocateObjectKeepRegisters = 0x03,
}

#[repr(C)]
pub struct KDEVICE_QUEUE_ENTRY
{
	DeviceListEntry: _LIST_ENTRY,
	SortKey: u32,
	Inserted: bool,
}

#[repr(C)]
pub struct KDEVICE_QUEUE
{
	Type: u16,
	Size: u16,
	DeviceListHead: _LIST_ENTRY,
	Lock: KSPIN_LOCK,
	Busy: bool,
}
