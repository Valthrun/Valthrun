//! Driver Object.

use crate::kapi::NTSTATUS;

use super::{DEVICE_OBJECT, UNICODE_STRING, PDRIVER_DISPATCH, IRP};

pub type PDRIVER_INITIALIZE = Option<extern "system" fn (_self: &mut DRIVER_OBJECT, &UNICODE_STRING) -> NTSTATUS>;
pub type PDRIVER_STARTIO = Option<extern "system" fn (_self: &mut DRIVER_OBJECT, &IRP)>;
pub type PDRIVER_UNLOAD = Option<extern "system" fn (_self: &mut DRIVER_OBJECT)>;


/// Represents the image of a loaded kernel-mode driver.
#[repr(C)]
pub struct DRIVER_OBJECT
{
	pub Type: u16,
	pub Size: u16,
	pub DeviceObject: *mut DEVICE_OBJECT,
	pub Flags: u32,
	pub DriverStart: *const u8,
	pub DriverSize: u32,
	pub DriverSection: *const u8,
	pub DriverExtension: *mut u8,
	pub DriverName: UNICODE_STRING,
	pub HardwareDatabase: *const UNICODE_STRING,
	pub FastIoDispatch: *mut u8,
	pub DriverInit: PDRIVER_INITIALIZE,
	pub DriverStartIo: PDRIVER_STARTIO,
	/// The entry point for the driver's Unload routine, if any.
	pub DriverUnload: PDRIVER_UNLOAD,
	/// A dispatch table consisting of an array of entry points for the driver's `DispatchXxx` routines.
	pub MajorFunction: [PDRIVER_DISPATCH; 28],
}
