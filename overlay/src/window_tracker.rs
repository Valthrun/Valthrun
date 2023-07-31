use std::ffi::CString;

use glium::glutin::{window::Window, platform::windows::WindowExtWindows};
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::{HWND, POINT, RECT},
        Graphics::Gdi::ClientToScreen,
        UI::WindowsAndMessaging::{FindWindowA, GetClientRect, MoveWindow},
    },
};
use crate::error::{OverlayError, Result};

/// Track the CS2 window and adjust overlay accordingly.
/// This is only required when playing in windowed mode.
pub struct WindowTracker {
    cs2_hwnd: HWND,
    current_bounds: RECT,
}

impl WindowTracker {
    pub fn new(target: &str) -> Result<Self> {
        let target = CString::new(target)
            .map_err(OverlayError::WindowInvalidName)?;

        let cs2_hwnd = unsafe {
            FindWindowA(
                PCSTR::null(),
                PCSTR::from_raw(target.as_ptr() as *const u8),
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

    pub fn update(&mut self, overlay: &Window) {
        let mut rect: RECT = Default::default();
        let success = unsafe { GetClientRect(self.cs2_hwnd, &mut rect) };
        if !success.as_bool() {
            return;
        }

        unsafe {
            ClientToScreen(self.cs2_hwnd, &mut rect.left as *mut _ as *mut POINT);
            ClientToScreen(self.cs2_hwnd, &mut rect.right as *mut _ as *mut POINT);
        }

        if rect == self.current_bounds {
            return;
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
                true,
            );
        }
    }
}
