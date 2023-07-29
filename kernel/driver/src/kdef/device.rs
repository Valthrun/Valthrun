//! Device Object.

use crate::kapi::NTSTATUS;

use super::{DRIVER_OBJECT, UNICODE_STRING, IRP, PVOID, KDPC, KDEVICE_QUEUE, KEVENT};

extern "system"
{
	pub fn IoCreateDevice(DriverObject: &mut DRIVER_OBJECT, DeviceExtensionSize: u32, DeviceName: *const UNICODE_STRING,
		DeviceType: u32, DeviceCharacteristics: u32, Exclusive: bool, DeviceObject: *mut*mut DEVICE_OBJECT) -> NTSTATUS;

	pub fn IoDeleteDevice(DeviceObject: &mut DEVICE_OBJECT) -> NTSTATUS;
	pub fn IoCreateSymbolicLink(SymbolicLinkName: &UNICODE_STRING, DeviceName: &UNICODE_STRING) -> NTSTATUS;
	pub fn IoDeleteSymbolicLink(SymbolicLinkName: &UNICODE_STRING) -> NTSTATUS;
}

/// Device object flags.
#[repr(C)]
pub enum DEVICE_FLAGS {
	NONE = 0,
	DO_VERIFY_VOLUME                = 0x00000002,
	DO_BUFFERED_IO                  = 0x00000004,
	DO_EXCLUSIVE                    = 0x00000008,
	DO_DIRECT_IO                    = 0x00000010,
	DO_MAP_IO_BUFFER                = 0x00000020,
	DO_DEVICE_HAS_NAME              = 0x00000040,
	DO_DEVICE_INITIALIZING          = 0x00000080,
	DO_SYSTEM_BOOT_PARTITION        = 0x00000100,
	DO_LONG_TERM_REQUESTS           = 0x00000200,
	DO_NEVER_LAST_DEVICE            = 0x00000400,
	DO_SHUTDOWN_REGISTERED          = 0x00000800,
	DO_BUS_ENUMERATED_DEVICE        = 0x00001000,
	DO_POWER_PAGABLE                = 0x00002000,
	DO_POWER_INRUSH                 = 0x00004000,
	DO_POWER_NOOP                   = 0x00008000,
	DO_LOW_PRIORITY_FILESYSTEM      = 0x00010000,
	DO_XIP                          = 0x00020000
}

/// `IoCompletion` routine result.
#[repr(u32)]
pub enum IO_COMPLETION_ROUTINE_RESULT {
	// STATUS_SUCCESS
	ContinueCompletion = 0,
	// STATUS_MORE_PROCESSING_REQUIRED
	StopCompletion = 0xC0000016,
}

/// The `DEVICE_OBJECT` structure is used by the operating system to represent a device object.
#[repr(C)]
pub struct DEVICE_OBJECT
{
	pub Type: u16,
	pub Size: u16,
	pub ReferenceCount: i32,
	pub DriverObject: *const DRIVER_OBJECT,
	pub NextDevice: *mut DEVICE_OBJECT,
	pub AttachedDevice: *mut DEVICE_OBJECT,
	pub CurrentIrp: *const IRP,
	pub Timer: *mut u8,
	pub Flags: u32,
	pub Characteristics: u32,
	pub Vpb: *mut u8,
	pub DeviceExtension: *mut u8,
	pub DeviceType: u32,
	pub StackSize: u8,
	pub Queue: *mut () /* *mut WAIT_CONTEXT_BLOCK */,
	pub AlignmentRequirement: u32,
	pub DeviceQueue: KDEVICE_QUEUE,
	pub Dpc: KDPC,
	pub ActiveThreadCount: u32,
	pub SecurityDescriptor: *const u8,
	pub DeviceLock: KEVENT,
	pub SectorSize: u16,
	pub Spare1: u16,
	pub DeviceObjectExtension: *mut DEVOBJ_EXTENSION,
	pub Reserved: *const u8,
}

impl DEVICE_OBJECT {
	/// Return a reference to driver-defined data structure.
	pub fn extension<T>(&mut self) -> &mut T {
		unsafe { &mut *(self.DeviceExtension as *mut T) }
	}
}

/// Device object extension structure.
#[repr(C)]
pub struct DEVOBJ_EXTENSION
{
	Type: u16,
	Size: u16,
	DeviceObject: *mut DEVICE_OBJECT,
	PowerFlags: u32,
	Dope: *mut u8,
	ExtensionFlags: u32,
	DeviceNode: *mut u8,
	AttachedTo: *mut DEVICE_OBJECT,
	StartIoCount: i32,
	StartIoKey: i32,
	StartIoFlags: u32,
	Vpb: *mut u8,
}

pub type PDEVICE_OBJECT = *mut DEVICE_OBJECT;

pub type PDRIVER_CANCEL = Option<extern "system" fn (DeviceObject: &mut DEVICE_OBJECT, Irp: &mut IRP)>;

pub type PDRIVER_DISPATCH = Option<extern "system" fn (DeviceObject: &mut DEVICE_OBJECT, Irp: &mut IRP) -> NTSTATUS>;

pub type PIO_COMPLETION_ROUTINE = Option<extern "system" fn (DeviceObject: &mut DEVICE_OBJECT, Irp: &mut IRP, Context: PVOID) -> IO_COMPLETION_ROUTINE_RESULT>;
