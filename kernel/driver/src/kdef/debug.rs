//! Debugger support.

#[allow(unused)]
extern "system" {
    pub fn KeBugCheck(code: u32) -> !;
}

/// `DbgPrintEx` Message severity.
#[repr(C)]
pub enum DPFLTR_LEVEL {
	ERROR = 0,
	WARNING,
	TRACE,
	INFO,
}