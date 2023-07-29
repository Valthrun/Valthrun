#![no_std]
#![feature(sync_unsafe_cell)]
#![feature(pointer_byte_offsets)]
#![feature(result_flattening)]

use core::{cell::SyncUnsafeCell, mem::size_of_val};

use handler::HandlerRegistry;
use kapi::{DeviceHandle, NTSTATUS};
use kdef::{UNICODE_STRING, DRIVER_OBJECT, IoCreateSymbolicLink, DEVICE_FLAGS, IoDeleteSymbolicLink, DEVICE_OBJECT, IRP, ProbeForWrite, ProbeForRead};
use valthrun_driver_shared::requests::{DriverRequestHealthCheck, DriverRequestCSModule, DriverRequestRead};

use crate::{logger::APP_LOGGER, handler::{handler_get_modules, handler_read}, kdef::{DbgPrintEx, DPFLTR_LEVEL}};

mod panic_hook;
mod logger;
mod handler;
mod kapi;
mod kdef;

extern crate alloc;

static REQUEST_HANDLER: SyncUnsafeCell<Option<HandlerRegistry>> = SyncUnsafeCell::new(Option::None);
static VARHAL_DEVICE: SyncUnsafeCell<Option<VarhalDevice>> = SyncUnsafeCell::new(Option::None);

struct VarhalDevice {
    _device: DeviceHandle,
    dos_link_name: UNICODE_STRING,
}

unsafe impl Sync for VarhalDevice {}
impl VarhalDevice {
    pub fn create(driver: &mut DRIVER_OBJECT) -> anyhow::Result<Self> {
        let dos_name = obfstr::wide!("\\DosDevices\\valthrun").into();
        let device_name = obfstr::wide!("\\Device\\valthrun").into();

        let mut device = DeviceHandle::create(
            driver,  
            &device_name, 
            0x00000022, // FILE_DEVICE_UNKNOWN
            0x00000100, // FILE_DEVICE_SECURE_OPEN
            false, 
        )?;
    
        unsafe {
            IoCreateSymbolicLink(&dos_name, &device_name)
                .ok()
                .map_err(|err| anyhow::anyhow!("IoCreateSymbolicLink: {}", err))?;
        };
    
        *device.flags_mut() |= DEVICE_FLAGS::DO_DIRECT_IO as u32;
        device.mark_initialized();
        Ok(Self {
            _device: device,
            dos_link_name: dos_name
        })
    }
}

impl Drop for VarhalDevice {
    fn drop(&mut self) {
        let result = unsafe { IoDeleteSymbolicLink(&self.dos_link_name) };
        if let Err(status) = result.ok() {
            log::warn!("Failed to unlink dos device: {}", status);
        }
    }
}

#[no_mangle]
extern "system" fn driver_unload(_driver: &mut DRIVER_OBJECT) {
    log::info!("Driver Unloaded");

    /* Remove the device */
    let device_handle = unsafe { &mut *VARHAL_DEVICE.get() };
    let _ = device_handle.take();

}

extern "system" fn irp_create(_device: &mut DEVICE_OBJECT, irp: &mut IRP) -> NTSTATUS {
    log::debug!("IRP create callback");
    irp.complete_request(NTSTATUS::Success)
}

extern "system" fn irp_close(_device: &mut DEVICE_OBJECT, irp: &mut IRP) -> NTSTATUS {
    log::debug!("IRP close callback");
    irp.complete_request(NTSTATUS::Success)
}

extern "system" fn irp_control(_device: &mut DEVICE_OBJECT, irp: &mut IRP) -> NTSTATUS {
    let outbuffer = irp.UserBuffer;
    let stack = irp.get_current_stack_location();
    let param = stack.ParametersDeviceIoControl();
    let request_code = param.IoControlCode;

    let handler = match unsafe { REQUEST_HANDLER.get().as_ref() }.map(Option::as_ref).flatten() {
        Some(handler) => handler,
        None => {
            log::warn!("Missing request handlers");
            return irp.complete_request(NTSTATUS::InvalidParameter);
        }
    };

    /* Note: We do not lock the buffers as it's a sync call and the user should not be able to free the input buffers. */
    let inbuffer = unsafe {
        core::slice::from_raw_parts(param.Type3InputBuffer as *const u8, param.InputBufferLength as usize)
    };
    let inbuffer_probe = kapi::try_seh(|| unsafe {
        ProbeForRead(inbuffer.as_ptr() as *const (), inbuffer.len(), 1);
    });
    if let Err(err) = inbuffer_probe {
        log::warn!("IRP request inbuffer invalid: {}", err);
        return irp.complete_request(NTSTATUS::InvalidParameter);
    }

    let outbuffer = unsafe {
        core::slice::from_raw_parts_mut(outbuffer as *mut u8, param.OutputBufferLength as usize)
    };
    let outbuffer_probe = kapi::try_seh(|| unsafe {
        ProbeForWrite(outbuffer.as_mut_ptr() as *mut (), outbuffer.len(), 1);
    });
    if let Err(err) = outbuffer_probe {
        log::warn!("IRP request outbuffer invalid: {}", err);
        return irp.complete_request(NTSTATUS::InvalidParameter);
    }

    match handler.handle(request_code, inbuffer, outbuffer) {
        Ok(_) => irp.complete_request(NTSTATUS::Success),
        Err(error) => {
            log::error!("IRP handle error: {}", error);
            irp.complete_request(NTSTATUS::InvalidParameter)
        }
    }
}

#[repr(C)]
#[allow(non_snake_case, non_camel_case_types)]
struct _OSVERSIONINFOEXW {
    dwOSVersionInfoSize: u32,
    dwMajorVersion: u32,
    dwMinorVersion: u32,
    dwBuildNumber: u32,
    dwPlatformId: u32,

    szCSDVersion: [u16; 128],
    wServicePackMajor: u16,
    wServicePackMinor: u16,
    wSuiteMask: u16,

    wProductType: u8,
    wReserved: u8
}

extern "system" {
    fn RtlGetVersion(info: &mut _OSVERSIONINFOEXW) -> NTSTATUS;
}

pub fn get_windows_build_number() -> anyhow::Result<u32, NTSTATUS> {
    let mut info: _OSVERSIONINFOEXW = unsafe { core::mem::zeroed() };
    info.dwOSVersionInfoSize = size_of_val(&info) as u32;
    unsafe { RtlGetVersion(&mut info) }
        .ok()
        .map(|_| info.dwBuildNumber)
}

#[no_mangle]
pub extern "system" fn driver_entry(driver: &mut DRIVER_OBJECT) -> NTSTATUS {
    log::set_max_level(log::LevelFilter::Trace);
    if log::set_logger(&APP_LOGGER).is_err() {
        unsafe { 
            DbgPrintEx(0, DPFLTR_LEVEL::ERROR as u32, "[VT] Failed to initialize app logger!\n\0".as_ptr());
        }
        return NTSTATUS::Failure;
    }

    log::info!("Initialize driver");
    driver.DriverUnload = Some(driver_unload);
    driver.MajorFunction[0x00] = Some(irp_create); /* IRP_MJ_CREATE */
    driver.MajorFunction[0x02] = Some(irp_close); /* IRP_MJ_CLOSE */
    driver.MajorFunction[0x0E] = Some(irp_control); /* IRP_MJ_DEVICE_CONTROL */
    
    let device = match VarhalDevice::create(driver) {
        Ok(device) => device,
        Err(error) => {
            log::error!("Failed to initialize device: {}", error);
            return NTSTATUS::Failure;
        }
    };
    log::debug!("Driver Object at 0x{:X}, Device Object at 0x{:X}", driver as *const _ as u64, device._device.0 as *const _ as u64);
    unsafe { *VARHAL_DEVICE.get() = Some(device) };

    let mut handler = HandlerRegistry::new();
    handler.register::<DriverRequestHealthCheck>(&|_req, res| {
        res.success = true;
        Ok(())
    });
    handler.register::<DriverRequestCSModule>(&handler_get_modules);
    handler.register::<DriverRequestRead>(&handler_read);

    unsafe { *REQUEST_HANDLER.get() = Some(handler) };

    log::warn!("TODO: RegisterOBCallback!");

    log::info!("Driver Initialized");
    NTSTATUS::Success
}
