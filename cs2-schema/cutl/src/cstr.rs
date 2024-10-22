use std::{
    borrow::Cow,
    ffi::CStr,
    string::FromUtf8Error,
};

use raw_struct::{
    builtins::Ptr64,
    AccessError,
    AccessMode,
    MemoryView,
};

pub trait CStringUtil {
    fn read_string(&self, memory: &dyn MemoryView) -> Result<Option<String>, AccessError>;
}

impl CStringUtil for Ptr64<[i8]> {
    fn read_string(&self, memory: &dyn MemoryView) -> Result<Option<String>, AccessError> {
        let address = self.address;
        if address == 0 {
            Ok(None)
        } else {
            // Using 8 as we don't know how far we can read
            let mut expected_length = 8;
            let mut buffer = Vec::new();

            // FIXME: Do cstring reading via shortcutting!
            loop {
                buffer.resize(expected_length, 0u8);
                memory
                    .read_memory(address, buffer.as_mut_slice())
                    .map_err(|err| AccessError {
                        source: err,

                        mode: AccessMode::Read,
                        offset: address,
                        size: expected_length,

                        member: None,
                        object: "PtrCStr".into(),
                    })?;

                if let Ok(str) = CStr::from_bytes_until_nul(&buffer) {
                    return Ok(Some(String::from_utf8_lossy(str.to_bytes()).to_string()));
                }

                expected_length += 8;
            }
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct PtrCStr(Ptr64<[i8]>);

impl CStringUtil for PtrCStr {
    fn read_string(&self, memory: &dyn MemoryView) -> Result<Option<String>, AccessError> {
        self.0.read_string(memory)
    }
}

pub trait FixedCStringUtil {
    fn actual_length(&self) -> usize;

    fn to_string(&self) -> Result<String, FromUtf8Error>;
    fn to_string_lossy(&self) -> Cow<'_, str>;
}

impl<const N: usize> FixedCStringUtil for [u8; N] {
    fn actual_length(&self) -> usize {
        CStr::from_bytes_until_nul(self).map_or(0, CStr::count_bytes)
    }

    fn to_string(&self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self[0..self.actual_length()].to_vec())
    }

    fn to_string_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self[0..self.actual_length()])
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct FixedCStr<const N: usize>([u8; N]);

impl<const N: usize> FixedCStringUtil for FixedCStr<N> {
    fn actual_length(&self) -> usize {
        self.0.actual_length()
    }

    fn to_string(&self) -> Result<String, FromUtf8Error> {
        self.0.to_string()
    }

    fn to_string_lossy(&self) -> Cow<'_, str> {
        self.0.to_string_lossy()
    }
}
