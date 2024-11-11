use windows::Win32::Foundation::{
    CloseHandle,
    HANDLE,
};

pub struct OwnedHandle {
    inner: HANDLE,
}

impl OwnedHandle {
    pub fn from_raw_handle(handle: HANDLE) -> Self {
        OwnedHandle { inner: handle }
    }

    pub fn raw_handle(&self) -> HANDLE {
        self.inner
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.inner);
        }
    }
}
