use std::ffi::CString;

use windows::{
    core::PCSTR,
    Win32::{
        Foundation::HWND,
        UI::{
            Shell::ShellExecuteA,
            WindowsAndMessaging::SW_SHOW,
        },
    },
};

mod imgui;
pub use self::imgui::*;

mod console_io;
pub use console_io::*;

pub fn open_url(url: &str) {
    unsafe {
        let url = match CString::new(url) {
            Ok(url) => url,
            Err(_) => return,
        };

        ShellExecuteA(
            HWND::default(),
            PCSTR::null(),
            PCSTR(url.as_bytes().as_ptr()),
            PCSTR::null(),
            PCSTR::null(),
            SW_SHOW,
        );
    }
}
