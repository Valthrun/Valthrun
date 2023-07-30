//! Debugger support.

use winapi::shared::ntdef::NTSTATUS;

#[allow(unused)]
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