//! Debugger support.

use crate::kapi::NTSTATUS;

extern "C" {
	/// `DbgPrint` routine sends a message to the kernel debugger.
	pub fn DbgPrint(Format: *const u8, ...) -> NTSTATUS;
	/// The `DbgPrintEx` routine sends a string to the kernel debugger if certain conditions are met.
	pub fn DbgPrintEx(ComponentId: u32, Level: u32, Format: *const u8, ...) -> NTSTATUS;
}

extern "system" {
    pub fn KeBugCheck(code: u32) -> !;

	/// Breaks into the kernel debugger.
	pub fn DbgBreakPoint();
	/// Breaks into the kernel debugger and sends the value of `Status` to the debugger.
	pub fn DbgBreakPointWithStatus(Status: NTSTATUS);
}

/// `DbgPrintEx` Message severity.
#[repr(C)]
pub enum DPFLTR_LEVEL {
	ERROR = 0,
	WARNING,
	TRACE,
	INFO,
}

/// `DbgPrintEx` Component name.
#[repr(C)]
pub enum DPFLTR_ID {
	SYSTEM = 0,
	SMSS,
	SETUP,
	NTFS,
	// ...
	IHVDRIVER = 77,
	IHVVIDEO,
	IHVAUDIO,
	IHVNETWORK,
	IHVSTREAMING,
	IHVBUS,

	DEFAULT = 99,
}
