use std::ffi::CStr;

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

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct FixedCStr<const N: usize>([u8; N]);

impl<const N: usize> FixedCStr<N> {
    pub fn to_string_lossy(&self) -> String {
        if let Ok(cstr) = CStr::from_bytes_until_nul(&self.0) {
            String::from_utf8_lossy(cstr.to_bytes()).into_owned()
        } else {
            String::from_utf8_lossy(&self.0).into_owned()
        }
    }
}
