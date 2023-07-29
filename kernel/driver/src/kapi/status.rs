//! NT Status codes.
#![allow(non_camel_case_types)]
#![allow(overflowing_literals)]

use core::fmt::Display;

/// NT Status type.
pub type NTSTATUS = Status;

/// NT Status code.
#[repr(C)]
#[derive(Clone, Copy)]
pub enum Status {
	Success = 0,
	Failure = 0xC0000001,
	
	InvalidParameter = 0xC000000D,
}

impl Display for Status {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:X}", *self as u32))
    }
}

impl ::core::default::Default for Status {
	#[inline]
	fn default() -> Status {
		Status::Success
	}
}

impl Status {
	/// Evaluates to `true` if the `Status` is a success type (`0..0x3FFFFFFF`)
	/// or an informational type (`0x40000000..0x7FFFFFFF`).
	pub fn is_ok(&self) -> bool {
		(*self as i32) >= 0
	}
	/// Status is a warning or error type.
	pub fn is_err(&self) -> bool {
		(*self as i32) < 0
	}
	/// Status is a success type.
	pub fn is_success(&self) -> bool {
		let c = *self as u32;
		c > 0 && c < 0x3FFF_FFFF
	}
	/// Status is a information type.
	pub fn is_information(&self) -> bool {
		let c = *self as u32;
		c > 0x4000_0000 && c < 0x7FFF_FFFF
	}
	/// Status is a warning type.
	pub fn is_warning(&self) -> bool {
		let c = *self as u32;
		c > 0x8000_0000 && c < 0xBFFF_FFFF
	}
	/// Status is a error type.
	pub fn is_error(&self) -> bool {
		let c = *self as u32;
		c > 0xC000_0000 && c < 0xFFFF_FFFF
	}

	pub fn ok(self) -> core::result::Result<(), Status> {
		if self.is_ok() {
			Ok(())
		} else {
			Err(self)
		}
	}
}