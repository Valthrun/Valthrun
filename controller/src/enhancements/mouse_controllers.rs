use std::ffi::CString;
use std::ptr::null_mut;
use winapi::um::fileapi::CreateFileA;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::ioapiset::DeviceIoControl;
use winapi::um::winnt::{GENERIC_READ, GENERIC_WRITE, HANDLE};
use winapi::shared::minwindef::{DWORD, LPVOID};

const IOCTL_MOUSE_MOVE: DWORD = (34u32 << 16) | (0u32 << 14) | (73142u32 << 2) | 0u32;

pub struct MouseController {
    h_driver: HANDLE,
}

impl MouseController {
    pub fn new() -> Self {
        let driver_path = CString::new("\\\\.\\Oykyo").unwrap();
        let h_driver = unsafe {
            CreateFileA(
                driver_path.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                0,
                null_mut(),
                3, // OPEN_EXISTING
                0,
                null_mut(),
            )
        };

        if h_driver == INVALID_HANDLE_VALUE {
            eprintln!("Failed to open driver.");
        } else {
            println!("Driver opened successfully.");
        }

        MouseController { h_driver }
    }

    pub fn is_valid(&self) -> bool {
        self.h_driver != INVALID_HANDLE_VALUE
    }

    pub fn move_mouse_down(&self, y: i32) -> bool {
        self.send_mouse_event(0, y)
    }

    pub fn move_mouse(&self, x: i32, y: i32) -> bool {
        self.send_mouse_event(x, y)
    }
    
    fn send_mouse_event(&self, x: i32, y: i32) -> bool {
        if self.h_driver == INVALID_HANDLE_VALUE {
            eprintln!("Invalid driver handle. Cannot send mouse event.");
            return false;
        }

        let mouse_request = MouseRequest { x, y, button_flags: 0 };

        let mut bytes_returned: DWORD = 0;
        let success = unsafe {
            DeviceIoControl(
                self.h_driver,
                IOCTL_MOUSE_MOVE, // Use the correct control code
                &mouse_request as *const _ as LPVOID,
                std::mem::size_of::<MouseRequest>() as DWORD,
                null_mut(),
                0,
                &mut bytes_returned,
                null_mut(),
            )
        };

        if success != 0 {
            println!("Mouse event sent successfully: x={}, y={}", x, y);
        } else {
            eprintln!("Failed to send mouse event.");
        }

        success != 0
    }
}

impl Drop for MouseController {
    fn drop(&mut self) {
        if self.h_driver != INVALID_HANDLE_VALUE {
            unsafe {
                CloseHandle(self.h_driver);
            }
            println!("Driver handle closed.");
        }
    }
}

#[repr(C)]
struct MouseRequest {
    x: i32,
    y: i32,
    button_flags: u16,
}
