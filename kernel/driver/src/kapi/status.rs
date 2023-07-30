//! NT Status codes.
#![allow(non_camel_case_types)]
#![allow(overflowing_literals)]

use winapi::shared::ntdef::NTSTATUS;

pub trait NTStatusEx {
	/// Evaluates to `true` if the `Status` is a success type (`0..0x3FFFFFFF`)
	/// or an informational type (`0x40000000..0x7FFFFFFF`).
	fn is_ok(&self) -> bool;

	/// Status is a warning or error type.
	fn is_err(&self) -> bool;

	/// Status is a success type.
	fn is_success(&self) -> bool;

	/// Status is a information type.
	fn is_information(&self) -> bool;

	/// Status is a warning type.
	fn is_warning(&self) -> bool;

	/// Status is a error type.
	fn is_error(&self) -> bool;

	fn ok(self) -> core::result::Result<(), NTSTATUS>;
}

impl NTStatusEx for NTSTATUS {
	/// Evaluates to `true` if the `Status` is a success type (`0..0x3FFFFFFF`)
	/// or an informational type (`0x40000000..0x7FFFFFFF`).
	fn is_ok(&self) -> bool {
		(*self as i32) >= 0
	}
	/// Status is a warning or error type.
	fn is_err(&self) -> bool {
		(*self as i32) < 0
	}
	/// Status is a success type.
	fn is_success(&self) -> bool {
		let c = *self as u32;
		c > 0 && c < 0x3FFF_FFFF
	}
	/// Status is a information type.
	fn is_information(&self) -> bool {
		let c = *self as u32;
		c > 0x4000_0000 && c < 0x7FFF_FFFF
	}
	/// Status is a warning type.
	fn is_warning(&self) -> bool {
		let c = *self as u32;
		c > 0x8000_0000 && c < 0xBFFF_FFFF
	}
	/// Status is a error type.
	fn is_error(&self) -> bool {
		let c = *self as u32;
		c > 0xC000_0000 && c < 0xFFFF_FFFF
	}

	fn ok(self) -> core::result::Result<(), NTSTATUS> {
		if self.is_ok() {
			Ok(())
		} else {
			Err(self)
		}
	}
}