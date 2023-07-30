use alloc::string::String;
use winapi::shared::ntdef::UNICODE_STRING;

pub trait UnicodeStringEx {
    fn from_bytes(s: &'static [u16]) -> UNICODE_STRING;
    fn as_string_lossy(&self) -> String;
}

impl UnicodeStringEx for UNICODE_STRING {
    fn from_bytes(s: &'static [u16]) -> UNICODE_STRING {
        let len = s.len();
        let n = if len > 0 && s[len - 1] == 0 { len - 1 } else { len };
    
        UNICODE_STRING {
            Length: (n * 2) as u16,
            MaximumLength: (len * 2) as u16,
            Buffer: s.as_ptr() as _,
        }
    }

    fn as_string_lossy(&self) -> String {
        String::from_utf16_lossy(
            unsafe {
                core::slice::from_raw_parts(self.Buffer, (self.Length / 2) as usize)
            }
        )
    }
}