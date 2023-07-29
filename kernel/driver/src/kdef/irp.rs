//! I/O request packets (IRP).

use crate::kapi::NTSTATUS;

use super::{_LIST_ENTRY, PVOID, PIO_COMPLETION_ROUTINE, IO_STATUS_BLOCK, KPROCESSOR_MODE, PIO_STATUS_BLOCK, PIO_APC_ROUTINE, PDRIVER_CANCEL, PDEVICE_OBJECT, IO_PRIORITY::{IO_NO_INCREMENT, KPRIORITY_BOOST}, KIRQL};

pub type PIRP = *mut IRP;
pub type PIO_STACK_LOCATION = *mut IO_STACK_LOCATION;

extern "system"
{
	fn IoCompleteRequest(Irp: PIRP, PriorityBoost: KPRIORITY_BOOST);

	pub fn IoAllocateIrp(StackSize: i8, ChargeQuota: bool) -> PIRP;
	pub fn IoFreeIrp(Irp: PIRP);
	pub fn IoReuseIrp(Irp: PIRP, Status: NTSTATUS);
	pub fn IoInitializeIrp(Irp: PIRP, PacketSize: u16, StackSize: i8);
	pub fn IoMakeAssociatedIrp(Irp: PIRP, StackSize: i8) -> PIRP;

	// unfortunately following are macro
	// fn IoGetCurrentIrpStackLocation(Irp: PIRP) -> PIO_STACK_LOCATION;
	// fn IoGetNextIrpStackLocation(Irp: PIRP) -> PIO_STACK_LOCATION;
	// fn IoSetNextIrpStackLocation(Irp: PIRP);
	// fn IoSkipCurrentIrpStackLocation(Irp: PIRP);
}

/// `IRP` Major Function Codes.
///
/// For information about these requests, see
/// [IRP Major Function Codes](https://msdn.microsoft.com/en-us/library/windows/hardware/ff548603%28v=vs.85%29.aspx).
#[repr(u8)]
pub enum IRP_MJ
{
	/// The operating system sends this request to open a handle to a file object or device object.
	CREATE,
	CREATE_NAMED_PIPE,
	/// Indicates that the last handle of the file object that is associated with the target device object
	/// has been closed and released. All outstanding I/O requests have been completed or canceled.
	/// See also `CLEANUP`.
	CLOSE,
	/// A user-mode application or Win32 component has requested a data transfer from the device.
	/// Or a higher-level driver has created and set up the read IRP.
	READ,
	/// A user-mode application or Win32 component has requested a data transfer to the device.
	/// Or a higher-level driver has created and set up the write IRP.
	WRITE,
	QUERY_INFORMATION,
	SET_INFORMATION,
	QUERY_EA,
	SET_EA,
	/// Indicates that the driver should flush the device's cache or its internal buffer,
	/// or, possibly, should discard the data in its internal buffer.
	FLUSH_BUFFERS,
	QUERY_VOLUME_INFORMATION,
	SET_VOLUME_INFORMATION,
	DIRECTORY_CONTROL,
	FILE_SYSTEM_CONTROL,
	/// An user-mode thread has called the Microsoft Win32 `DeviceIoControl` function, or a higher-level kernel-mode driver has set up the request.
	DEVICE_CONTROL,
	/// Some driver calls either `IoBuildDeviceIoControlRequest` or `IoAllocateIrp` to create a request.
	INTERNAL_DEVICE_CONTROL,
	/// Indicates that a file system driver is sending notice that the system is being shut down.
	SHUTDOWN,
	LOCK_CONTROL,
	/// Indicates that the last handle for a file object that is associated with the target device object has been closed
	/// (but, due to outstanding I/O requests, might not have been released).
	/// See also `CLOSE`.
	CLEANUP,
	CREATE_MAILSLOT,
	QUERY_SECURITY,
	SET_SECURITY,
	POWER,
	SYSTEM_CONTROL,
	DEVICE_CHANGE,
	QUERY_QUOTA,
	SET_QUOTA,
	PNP,
	MAXIMUM_FUNCTION,
}

/// The `IRP` structure is a partial opaque structure that represents an I/O request packet.
#[repr(C)]
pub struct IRP
{
	pub Type: u16,
	pub Size: u16,
	/// Pointer to an `MDL` describing a user buffer, if the driver is using direct I/O.
	pub MdlAddress: PVOID,
	/// Flags word - used to remember various flags.
	pub Flags: u32,
	/// Pointer to a system-space buffer if the driver is using buffered I/O.
	pub SystemBuffer: PVOID,
	pub ThreadListEntry: _LIST_ENTRY,
	/// I/O status - final status of operation.
	pub IoStatus: IO_STATUS_BLOCK,
	/// Indicates the execution mode of the original requester of the operation.
	pub RequestorMode: KPROCESSOR_MODE,
	/// If set to `TRUE`, a driver has marked the IRP pending.
	pub PendingReturned: bool,
	/// Stack state information.
	pub StackCount: i8,
	/// Stack state information.
	pub CurrentLocation: i8,
	/// If set to `TRUE`, the IRP either is or should be canceled.
	pub Cancel: bool,
	/// Irql at which the cancel spinlock was acquired.
	pub CancelIrql: KIRQL,
	pub ApcEnvironment: u8,
	/// Allocation control flags.
	pub AllocationFlags: u8,
	/// User parameters.
	pub UserIosb: PIO_STATUS_BLOCK,
	pub UserEvent: *const () /* PKEVENT */,

	// union {
	pub UserApcRoutine: PIO_APC_ROUTINE,
	pub UserApcContext: PVOID,
	// } Overlay

	/// Contains the entry point for a driver-supplied `Cancel` routine to be called if the IRP is canceled.
	pub CancelRoutine: PDRIVER_CANCEL,
	/// Contains the address of an output buffer for `IRP_MJ_DEVICE_CONTROL`.
	pub UserBuffer: PVOID,

	/// Kernel structures.
	// union {
	pub Overlay: _IRP_OVERLAY,
	// } Tail
}

/// Kernel structures for IRP.
#[repr(C)]
pub struct _IRP_OVERLAY
{
	pub DriverContext: [PVOID; 4],
	pub Thread: *const () /* PETHREAD */,
	pub AuxiliaryBuffer: PVOID,
	pub ListEntry: _LIST_ENTRY,
	/// Current stack location.
	pub CurrentStackLocation: PIO_STACK_LOCATION,
	pub OriginalFileObject: *const () /* PFILE_OBJECT */,
}

pub const SL_PENDING_RETURNED: u8 = 0x01;
pub const SL_INVOKE_ON_CANCEL: u8 = 0x20;
pub const SL_INVOKE_ON_SUCCESS: u8 = 0x40;
pub const SL_INVOKE_ON_ERROR: u8 = 0x80;

/// I/O Stack Locations.
#[repr(C)]
pub struct IO_STACK_LOCATION
{
	/// The IRP major function code indicating the type of I/O operation to be performed.
	pub MajorFunction: u8,
	/// A subfunction code for `MajorFunction`.
	pub MinorFunction: u8,
	/// Request-type-specific values (see [DEVICE_FLAGS](../device_object/enum.DEVICE_FLAGS.html)).
	pub Flags: u8,
	/// Stack location control flags.
	pub Control: u8,

	/// A union that depends on the major and minor IRP function code values
	/// contained in `MajorFunction` and `MinorFunction`.
	// union Parameters
	pub Parameters: [PVOID; 4],

	/// A pointer to the driver-created `DEVICE_OBJECT` structure
	/// representing the target physical, logical, or virtual device for which this driver is to handle the IRP.
	pub DeviceObject: PDEVICE_OBJECT,
	/// A pointer to a `FILE_OBJECT` structure that represents the file object, if any, that is associated with `DeviceObject` pointer.
	pub FileObject: *const () /* PFILE_OBJECT */,
	/// The following routine is invoked depending on the flags in the above `Flags` field.
	pub CompletionRoutine: PIO_COMPLETION_ROUTINE,
	/// The following is used to store the address of the context parameter that should be passed to the `CompletionRoutine`.
	pub Context: PVOID,
}

/// Parameters for `IRP_MJ_READ`.
#[repr(C)]
pub struct _IO_STACK_LOCATION_READ
{
	pub Length: u32,
	pub Key: u32,
	pub ByteOffset: i64,
}

/// Parameters for `IRP_MJ_DEVICE_CONTROL`.
#[repr(C)]
pub struct _IO_STACK_LOCATION_IRP_MJ_DEVICE_CONTROL
{
	pub OutputBufferLength: u32,
	pub Padding0: [u8; 4],

	pub InputBufferLength: u32,
	pub Padding1: [u8; 4],

	pub IoControlCode: u32,
	pub Padding2: [u8; 4],
	
	pub Type3InputBuffer: PVOID
}


impl IRP {
	pub fn new(StackSize: i8) -> PIRP {
		unsafe { IoAllocateIrp(StackSize, false) }
	}

	pub fn with_quota(StackSize: i8) -> PIRP {
		unsafe { IoAllocateIrp(StackSize, true) }
	}

	pub fn free(&mut self) {
		unsafe { IoFreeIrp(self) };
	}

	/// Returns a pointer to the caller's stack location in the given `IRP`.
	pub fn get_current_stack_location(&mut self) -> &mut IO_STACK_LOCATION {
		unsafe { &mut *self.Overlay.CurrentStackLocation }
	}

	/// Returns a pointer to the next-lower-level driver's I/O stack location.
	pub fn get_next_stack_location(&mut self) -> &mut IO_STACK_LOCATION {
		unsafe { &mut *self.Overlay.CurrentStackLocation.offset(-1) }
	}

	/// Indicates that the caller has completed all processing for a given I/O request
	/// and is returning the given IRP to the I/O manager.
	pub fn complete_request(&mut self, Status: NTSTATUS) -> NTSTATUS {
		self.IoStatus.Status = Status;
		unsafe { IoCompleteRequest(self, IO_NO_INCREMENT) };
		return Status;
	}

	/// Registers an `IoCompletion` routine, which will be called when the next-lower-level driver
	/// has completed the requested operation for the given IRP.
	pub fn set_completion(&mut self, CompletionRoutine: PIO_COMPLETION_ROUTINE, Context: PVOID,
			InvokeOnSuccess: bool, InvokeOnError: bool, InvokeOnCancel: bool)
	{
		let lower = self.get_next_stack_location();
		lower.CompletionRoutine = CompletionRoutine;
		lower.Context = Context;
		lower.Control = 0;
		if InvokeOnSuccess {
			lower.Control |= SL_INVOKE_ON_SUCCESS;
		}
		if InvokeOnError {
			lower.Control |= SL_INVOKE_ON_ERROR;
		}
		if InvokeOnCancel {
			lower.Control |= SL_INVOKE_ON_CANCEL;
		}
	}

	pub fn set_unconditional_completion(&mut self, CompletionRoutine: PIO_COMPLETION_ROUTINE, Context: PVOID) {
		self.set_completion(CompletionRoutine, Context, true, true, true)
	}
}

impl IO_STACK_LOCATION {
	/// Access parameters for `IRP_MJ_READ`.
	pub fn ParametersRead(&mut self) -> &mut _IO_STACK_LOCATION_READ {
		unsafe { ::core::mem::transmute(&mut self.Parameters) }
	}

	/// Access parameters for `IRP_MJ_DEVICE_CONTROL`.
	pub fn ParametersDeviceIoControl(&mut self) -> &mut _IO_STACK_LOCATION_IRP_MJ_DEVICE_CONTROL {
		unsafe { ::core::mem::transmute(&mut self.Parameters) }
	}
}
