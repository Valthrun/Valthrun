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
                FindWindowExA,
                FindWindowW,
                GetClientRect,
                GetWindowRect,
                GetWindowThreadProcessId,
                MoveWindow,
                SendMessageA,
                WM_PAINT,
            },
        },
    },
};

use crate::{
    error::{
        OverlayError,
        Result,
    },
    util,
};

pub enum OverlayTarget {
    Window(HWND),
    WindowTitle(String),
    WindowOfProcess(u32),
}

impl OverlayTarget {
    pub(crate) fn resolve_target_window(&self) -> Result<HWND> {
        Ok(match self {
            Self::Window(hwnd) => *hwnd,
            Self::WindowTitle(title) => unsafe {
                FindWindowW(
                    PCWSTR::null(),
                    PCWSTR::from_raw(util::to_wide_chars(title).as_ptr()),
                )
            },
            Self::WindowOfProcess(process_id) => {
                const MAX_ITERATIONS: usize = 1_000_000;
                let mut iterations = 0;
                let mut current_hwnd = HWND::default();
                while iterations < MAX_ITERATIONS {
                    iterations += 1;

                    current_hwnd = unsafe { FindWindowExA(None, current_hwnd, None, None) };
                    if current_hwnd.0 == 0 {
                        break;
                    }

                    let mut window_process_id = 0;
                    let success = unsafe {
                        GetWindowThreadProcessId(current_hwnd, Some(&mut window_process_id)) != 0
                    };
                    if !success || window_process_id != *process_id {
                        continue;
                    }

                    let mut window_rect = RECT::default();
                    let success =
                        unsafe { GetWindowRect(current_hwnd, &mut window_rect).as_bool() };
                    if !success {
                        continue;
                    }

                    if window_rect.left == 0
                        && window_rect.bottom == 0
                        && window_rect.right == 0
                        && window_rect.top == 0
                    {
                        /* Window is not intendet to be shown. */
                        continue;
                    }

                    log::debug!(
                        "Found window 0x{:X} which belongs to process {}",
                        current_hwnd.0,
                        process_id
                    );
                    return Ok(current_hwnd);
                }

                if iterations == MAX_ITERATIONS {
                    log::warn!("FindWindowExA seems to be cought in a loop.");
                }

                Default::default()
            }
        })
    }
}

/// Track the CS2 window and adjust overlay accordingly.
/// This is only required when playing in windowed mode.
pub struct WindowTracker {
    cs2_hwnd: HWND,
    current_bounds: RECT,
}

impl WindowTracker {
    pub fn new(target: &OverlayTarget) -> Result<Self> {
        let hwnd = target.resolve_target_window()?;
        if hwnd.0 == 0 {
            return Err(OverlayError::WindowNotFound);
        }

        Ok(Self {
            cs2_hwnd: hwnd,
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
