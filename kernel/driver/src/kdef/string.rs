use crate::kapi::NTSTATUS;

/// A counted Unicode string.
#[repr(C)]
pub struct UNICODE_STRING {
	/// The length in **bytes** of the string stored in `Buffer`.
	pub Length: u16,
	/// The length in **bytes** of `Buffer`.
	pub MaximumLength: u16,
	/// Pointer to a buffer used to contain a string of wide characters.
	pub Buffer: *const u16,
}

/// A counted string used for ANSI strings.
#[repr(C)]
pub struct ANSI_STRING {
	/// The length in *bytes* of the string stored in `Buffer`.
	pub Length: u16,
	/// The length in bytes of `Buffer`.
	pub MaximumLength: u16,
	/// Pointer to a buffer used to contain a string of characters.
	pub Buffer: *const u8,
}


pub type AnsiString = ANSI_STRING;
pub type UnicodeString = UNICODE_STRING;
pub type CONST_UNICODE_STRING = UNICODE_STRING;
pub type CONST_ANSI_STRING = ANSI_STRING;

pub type PUNICODE_STRING = *mut UNICODE_STRING;
pub type PCUNICODE_STRING = *const UNICODE_STRING;

extern "system" {
	pub fn RtlIntegerToUnicodeString(Value: u32, Base: u32, String: &mut UNICODE_STRING) -> NTSTATUS;
	pub fn RtlInt64ToUnicodeString(Value: u64, Base: u32, String: &mut UNICODE_STRING) -> NTSTATUS;
	pub fn RtlUnicodeStringToInteger(String: &CONST_UNICODE_STRING, Base: u32, Value: &mut u32) -> NTSTATUS;

	pub fn RtlUnicodeStringToAnsiString(DestinationString: &mut ANSI_STRING, SourceString: &CONST_UNICODE_STRING, AllocateDestination: bool) -> NTSTATUS;
	pub fn RtlUnicodeStringToAnsiSize(SourceString: &CONST_UNICODE_STRING) -> u32;

	pub fn RtlAnsiStringToUnicodeString(DestinationString: &mut UNICODE_STRING, SourceString: &CONST_ANSI_STRING, AllocateDestination: bool) -> NTSTATUS;
	pub fn RtlAnsiStringToUnicodeSize(SourceString: &CONST_ANSI_STRING) -> u32;

	pub fn RtlCompareUnicodeString (String1: &CONST_UNICODE_STRING, String2: &CONST_UNICODE_STRING, CaseInSensitive: bool) -> i32;
	pub fn RtlCompareString (String1: &CONST_ANSI_STRING, String2: &CONST_ANSI_STRING, CaseInSensitive: bool) -> i32;

	pub fn RtlEqualUnicodeString(String1: &CONST_UNICODE_STRING, String2: &CONST_UNICODE_STRING) -> bool;
	pub fn RtlEqualString(String1: &CONST_ANSI_STRING, String2: &CONST_ANSI_STRING) -> bool;

	pub fn RtlFreeAnsiString(UnicodeString: &mut ANSI_STRING);
	pub fn RtlFreeUnicodeString(UnicodeString: &mut UNICODE_STRING);
}