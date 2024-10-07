use std::{
    ffi::CString,
    os::raw::c_void,
};

use windows::{
    core::PCSTR,
    Win32::{
        Foundation,
        Storage::FileSystem::{
            self,
            CreateFileA,
            FILE_FLAGS_AND_ATTRIBUTES,
        },
        System::IO::DeviceIoControl,
    },
};

use super::DriverInterface;
use crate::{
    KInterfaceError,
    KResult,
};

pub struct IoctrlDriverInterface {
    driver_handle: Foundation::HANDLE,
}

impl IoctrlDriverInterface {
    pub fn create(path: &str) -> KResult<Self> {
        let driver_handle = unsafe {
            let path = CString::new(path).map_err(KInterfaceError::DeviceInvalidPath)?;
            CreateFileA(
                PCSTR::from_raw(path.as_bytes().as_ptr()),
                Foundation::GENERIC_READ.0 | Foundation::GENERIC_WRITE.0,
                FileSystem::FILE_SHARE_READ | FileSystem::FILE_SHARE_WRITE,
                None,
                FileSystem::OPEN_EXISTING,
                FILE_FLAGS_AND_ATTRIBUTES(0),
                None,
            )
            .map_err(KInterfaceError::DeviceUnavailable)?
        };

        Ok(Self { driver_handle })
    }
}

impl DriverInterface for IoctrlDriverInterface {
    fn execute_request(
        &self,
        function_code: u16,
        request: &[u8],
        response: &mut [u8],
    ) -> KResult<()> {
        let control_code = {
            (0x00000022 << 16) | // FILE_DEVICE_UNKNOWN
            (0x00000000 << 14) | // FILE_SPECIAL_ACCESS
            (0x00000001 << 13) | // Custom access code
            ((function_code as u32 & 0x3FF) << 02) |
            (0x00000003 << 00)
        };

        let success = unsafe {
            DeviceIoControl(
                self.driver_handle,
                control_code,
                Some(request.as_ptr() as *const c_void),
                request.len() as u32,
                Some(response.as_mut_ptr() as *mut c_void),
                response.len() as u32,
                None,
                None,
            )
            .as_bool()
        };

        if success {
            Ok(())
        } else {
            /* TOOD: GetLastErrorCode? */
            Err(KInterfaceError::RequestFailed)
        }
    }
}
