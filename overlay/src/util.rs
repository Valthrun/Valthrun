use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{
            MessageBoxW,
            MB_ICONERROR,
            MB_OK,
        },
    },
};

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

pub fn show_error_message(title: &str, message: &str) {
    let title_wide = to_wide_chars(title);
    let message_wide = to_wide_chars(message);

    unsafe {
        MessageBoxW(
            HWND::default(),
            PCWSTR::from_raw(message_wide.as_ptr()),
            PCWSTR::from_raw(title_wide.as_ptr()),
            MB_ICONERROR | MB_OK,
        );
    }
}
