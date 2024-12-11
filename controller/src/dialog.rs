use std::ffi::CString;

use windows::{
    core::PCSTR,
    Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{
            MessageBoxA,
            IDYES,
            MB_DEFBUTTON1,
            MB_DEFBUTTON2,
            MB_ICONERROR,
            MB_YESNO,
        },
    },
};

pub fn show_yes_no(title: &str, content: &str, default_value: bool) -> bool {
    let title = CString::new(title).unwrap();
    let content = CString::new(content).unwrap();

    let result = unsafe {
        MessageBoxA(
            HWND::default(),
            PCSTR::from_raw(content.as_ptr() as *const u8),
            PCSTR::from_raw(title.as_ptr() as *const u8),
            MB_ICONERROR
                | if default_value {
                    MB_DEFBUTTON1
                } else {
                    MB_DEFBUTTON2
                }
                | MB_YESNO,
        )
    };
    result == IDYES
}
