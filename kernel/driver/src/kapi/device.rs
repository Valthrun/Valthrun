use crate::kdef::{PDEVICE_OBJECT, DRIVER_OBJECT, UNICODE_STRING, IoCreateDevice, DEVICE_FLAGS, IoDeleteDevice};

pub struct DeviceHandle(pub PDEVICE_OBJECT);
unsafe impl Sync for DeviceHandle {}

impl DeviceHandle {
    pub fn create(driver: &mut DRIVER_OBJECT, device_name: &UNICODE_STRING, device_type: u32, characteristics: u32, exclusive: bool) -> anyhow::Result<Self> {
        let mut device_ptr: PDEVICE_OBJECT = core::ptr::null_mut();
        let result = unsafe {
            IoCreateDevice(
                driver, 0, 
                device_name, 
                device_type,
                characteristics,
                exclusive, 
                &mut device_ptr
            )
        };

        if !result.is_ok() {
            anyhow::bail!("IoCreateDevice failed with {}", result)
        }

        Ok(Self(device_ptr))
    }
    
    pub fn flags(&self) -> u32 {
        unsafe { (*self.0).Flags }
    }

    pub fn flags_mut(&mut self) -> &mut u32 {
        unsafe { &mut (*self.0).Flags }
    }

    pub fn mark_initialized(&mut self) {
        unsafe {
            (*self.0).Flags &= !(DEVICE_FLAGS::DO_DEVICE_INITIALIZING as u32);
        }
    }
}

impl Drop for DeviceHandle {
    fn drop(&mut self) {
        let result = unsafe { IoDeleteDevice(&mut *self.0) };

        if !result.is_success() {
            log::warn!("Failed to destroy device: {}", result)
        }
    }
}