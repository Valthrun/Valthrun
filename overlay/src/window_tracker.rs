use imgui_winit_support::winit::{
    platform::windows::WindowExtWindows,
    window::Window,
};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{
            GetLastError,
            ERROR_INVALID_WINDOW_HANDLE,
            HWND,
            LPARAM,
            POINT,
            RECT,
            WPARAM,
        },
        Graphics::Gdi::ClientToScreen,
        UI::{
            Input::KeyboardAndMouse::GetFocus,
            WindowsAndMessaging::{
                FindWindowW,
                GetClientRect,
                MoveWindow,
                SendMessageA,
                WM_PAINT, FindWindowExA, IsWindowVisible, GetWindowThreadProcessId, GetParent, GetClassNameW,
            },
        }, System::SystemInformation::GetTickCount,
    },
};

use crate::error::{
    OverlayError,
    Result,
};

/// Track the CS2 window and adjust overlay accordingly.
/// This is only required when playing in windowed mode.
pub struct WindowTracker {
    cs2_hwnd: HWND,
    current_bounds: RECT,
}

fn get_window_handle_by_process_id(
    process_id: u32,
    window_class: Option<&str>,
    timeout: Option<u32>,
    visible: Option<bool>,
) -> u32 {
    let start_time: u32 = unsafe { GetTickCount() };
    let end_time = match timeout {
        Some(t) if t > 0 => t,
        _ => 31536000,
    };
    let visible: bool = visible.unwrap_or(true);
    let mut hwnd: isize = 0;
    let mut process_id_of_window: u32 = 0;
    let mut class_name: String;

    loop {
        // 判断是否超时
        if unsafe { GetTickCount() } - start_time >= end_time {
            break;
        }
        hwnd = unsafe { FindWindowExA(None, HWND(hwnd as isize), None, None).0 };
        if hwnd == 0 {
            break;
        }
        if visible {
            if unsafe { IsWindowVisible(HWND(hwnd as isize)) } == false {
                continue;
            }
        }
        unsafe { GetWindowThreadProcessId(HWND(hwnd as isize), Some(&mut process_id_of_window)) };
        if process_id_of_window == process_id && unsafe { GetParent(HWND(hwnd as isize)) }.0 == 0 {
            class_name = get_window_class_name(hwnd.try_into().unwrap()); // 自定义函数，用于获取窗口类名
            if let Some(window_class) = window_class {
                if class_name.contains(window_class) {
                    return hwnd.try_into().unwrap();
                }
            } else {
                return hwnd.try_into().unwrap();
            }
        }
    }
    0
}

fn get_window_class_name(hwnd: u32) -> String {
    let mut buffer = [0u16; 256];
    let len = unsafe { GetClassNameW(HWND(hwnd as isize), &mut buffer) };
    String::from_utf16_lossy(&buffer[..len as usize])
}

impl WindowTracker {
    pub fn new(target: u32) -> Result<Self> {
        log::trace!("Looking for a game window with PID {:?}", target);
        let cs2_hwnd = match get_window_handle_by_process_id(target, None, Some(10000), None) {
            v if v <= 0 => return Err(OverlayError::WindowNotFound),
            v => HWND(v as isize)
        };

        if cs2_hwnd.0 == 0 {
            return Err(OverlayError::WindowNotFound);
        }

        Ok(Self {
            cs2_hwnd,
            current_bounds: Default::default(),
        })
    }

    pub fn mark_force_update(&mut self) {
        self.current_bounds = Default::default();
    }

    pub fn update(&mut self, overlay: &Window) -> bool {
        let mut rect: RECT = Default::default();
        let success = unsafe { GetClientRect(self.cs2_hwnd, &mut rect) };
        if !success.as_bool() {
            let error = unsafe { GetLastError() };
            if error == ERROR_INVALID_WINDOW_HANDLE {
                return false;
            }

            log::warn!("GetClientRect failed for tracked window: {:?}", error);
            return true;
        }

        unsafe {
            ClientToScreen(self.cs2_hwnd, &mut rect.left as *mut _ as *mut POINT);
            ClientToScreen(self.cs2_hwnd, &mut rect.right as *mut _ as *mut POINT);
        }

        if unsafe { GetFocus() } != self.cs2_hwnd {
            /*
             * CS2 will render a black screen as soon as CS2 does not have the focus and is completely covered by
             * another window. To prevent the overlay covering CS2 we make it one pixel less then the actual CS2 window.
             */
            rect.bottom -= 1;
        }

        if rect == self.current_bounds {
            return true;
        }

        self.current_bounds = rect;
        log::debug!("Window bounds changed: {:?}", rect);
        unsafe {
            let overlay_hwnd = HWND(overlay.hwnd());
            MoveWindow(
                overlay_hwnd,
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
                false, // Don't do a complete repaint (may flicker)
            );

            // Request repaint, so we acknoledge the new bounds
            SendMessageA(overlay_hwnd, WM_PAINT, WPARAM::default(), LPARAM::default());
        }

        true
    }
}