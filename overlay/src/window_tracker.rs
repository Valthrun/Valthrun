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
                WM_PAINT,
            },
        },
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

fn to_wide_chars(s: &str) -> Vec<u16> {
    use std::{
        ffi::OsStr,
        os::windows::ffi::OsStrExt,
    };
    OsStr::new(s)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect::<Vec<_>>()
}

impl WindowTracker {
    pub fn new(target: &str) -> Result<Self> {
        let cs2_hwnd = unsafe {
            FindWindowW(
                PCWSTR::null(),
                PCWSTR::from_raw(to_wide_chars(target).as_ptr()),
            )
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
