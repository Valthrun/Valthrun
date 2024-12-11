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
            Input::KeyboardAndMouse::{
                GetFocus,
                SetActiveWindow,
            },
            WindowsAndMessaging::{
                FindWindowExA,
                FindWindowW,
                GetClientRect,
                GetWindowLongPtrA,
                GetWindowRect,
                GetWindowThreadProcessId,
                MoveWindow,
                SendMessageA,
                SetWindowLongPtrA,
                GWL_EXSTYLE,
                WM_PAINT,
                WS_EX_NOACTIVATE,
                WS_EX_TRANSPARENT,
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
    overlay_hwnd: HWND,
    target_hwnd: HWND,
    current_bounds: RECT,
}

impl WindowTracker {
    pub fn new(overlay_hwnd: HWND, target: &OverlayTarget) -> Result<Self> {
        let target_hwnd = target.resolve_target_window()?;
        if target_hwnd.0 == 0 {
            return Err(OverlayError::WindowNotFound);
        }

        Ok(Self {
            overlay_hwnd,
            target_hwnd,
            current_bounds: Default::default(),
        })
    }

    pub fn mark_force_update(&mut self) {
        self.current_bounds = Default::default();
    }

    pub fn update(&mut self) -> bool {
        let mut rect: RECT = Default::default();
        let success = unsafe { GetClientRect(self.target_hwnd, &mut rect) };
        if !success.as_bool() {
            let error = unsafe { GetLastError() };
            if error == ERROR_INVALID_WINDOW_HANDLE {
                return false;
            }

            log::warn!("GetClientRect failed for tracked window: {:?}", error);
            return true;
        }

        unsafe {
            ClientToScreen(self.target_hwnd, &mut rect.left as *mut _ as *mut POINT);
            ClientToScreen(self.target_hwnd, &mut rect.right as *mut _ as *mut POINT);
        }

        if unsafe { GetFocus() } != self.target_hwnd {
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
            MoveWindow(
                self.overlay_hwnd,
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
                false, // Don't do a complete repaint (may flicker)
            );

            // Request repaint, so we acknoledge the new bounds
            SendMessageA(
                self.overlay_hwnd,
                WM_PAINT,
                WPARAM::default(),
                LPARAM::default(),
            );
        }

        true
    }
}

/// Toggles the overlay noactive and transparent state
/// according to whenever ImGui wants mouse/cursor grab.
pub struct ActiveTracker {
    hwnd: HWND,
    currently_active: bool,
}

impl ActiveTracker {
    pub fn new(hwnd: HWND) -> Self {
        Self {
            hwnd,
            currently_active: true,
        }
    }

    pub fn update(&mut self, io: &imgui::Io) {
        let window_active = io.want_capture_mouse | io.want_capture_keyboard;
        if window_active == self.currently_active {
            return;
        }

        self.currently_active = window_active;
        unsafe {
            let mut style = GetWindowLongPtrA(self.hwnd, GWL_EXSTYLE);
            if window_active {
                style &= !((WS_EX_NOACTIVATE | WS_EX_TRANSPARENT).0 as isize);
            } else {
                style |= (WS_EX_NOACTIVATE | WS_EX_TRANSPARENT).0 as isize;
            }

            log::trace!("Set UI active: {window_active}");
            SetWindowLongPtrA(self.hwnd, GWL_EXSTYLE, style);
            if window_active {
                SetActiveWindow(self.hwnd);
            }
        }
    }
}
