use obfstr::obfstr;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::HWND,
        System::Console::GetConsoleProcessList,
        UI::WindowsAndMessaging::{
            MessageBoxW,
            MB_ICONERROR,
            MB_OK,
        },
    },
};

pub fn is_console_invoked() -> bool {
    let console_count = unsafe {
        let mut result = [0u32; 128];
        GetConsoleProcessList(&mut result)
    };
    console_count > 1
}

fn to_wide_chars(s: &str) -> Vec<u16> {
    let mut result = s.encode_utf16().collect::<Vec<_>>();
    result.push(0);
    result
}

fn show_error_message(title: &str, message: &str) {
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

pub fn show_critical_error(message: &str) {
    for line in message.lines() {
        log::error!("{}", line);
    }

    if !is_console_invoked() {
        self::show_error_message(obfstr!("Valthrun Radar Client"), message);
    }
}
