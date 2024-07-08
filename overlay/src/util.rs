pub fn to_wide_chars(s: &str) -> Vec<u16> {
    use std::{
        ffi::OsStr,
        os::windows::ffi::OsStrExt,
    };

    OsStr::new(s)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect::<Vec<_>>()
}
