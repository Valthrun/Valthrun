use windows::{
    core::Error,
    Win32::System::Threading::{
        OpenProcess,
        PROCESS_ACCESS_RIGHTS,
    },
};

use crate::handle::OwnedHandle;

pub fn open_process_by_id(id: u32, access: PROCESS_ACCESS_RIGHTS) -> Result<OwnedHandle, Error> {
    unsafe {
        match OpenProcess(access, false, id) {
            Ok(handle) => Ok(OwnedHandle::from_raw_handle(handle)),
            Err(err) => Err(err),
        }
    }
}
