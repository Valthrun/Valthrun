#![no_std]
#![feature(sync_unsafe_cell)]
#![feature(pointer_byte_offsets)]
#![feature(result_flattening)]

use core::{cell::SyncUnsafeCell, mem::size_of_val, ffi::CStr};

use alloc::{sync::Arc, string::{String, ToString}, vec::Vec};
use anyhow::anyhow;
use handler::HandlerRegistry;
use kapi::{DeviceHandle, FastMutex, UnicodeStringEx, NTStatusEx};
use kdef::{ProbeForRead, ProbeForWrite, _OB_OPERATION_REGISTRATION, OB_OPERATION_HANDLE_CREATE, OB_OPERATION_HANDLE_DUPLICATE, _OB_CALLBACK_REGISTRATION, OB_FLT_REGISTRATION_VERSION, ObRegisterCallbacks, _OB_PRE_OPERATION_INFORMATION, ObUnRegisterCallbacks, PsProcessType};
use log::Level;
use valthrun_driver_shared::requests::{RequestHealthCheck, RequestCSModule, RequestRead, RequestProtectionToggle};
use winapi::{shared::{ntdef::{UNICODE_STRING, NTSTATUS, PVOID}, ntstatus::{STATUS_SUCCESS, STATUS_INVALID_PARAMETER, STATUS_FAILED_DRIVER_ENTRY}}, km::wdm::{DRIVER_OBJECT, DEVICE_TYPE, DEVICE_FLAGS, IoCreateSymbolicLink, IoDeleteSymbolicLink, DEVICE_OBJECT, IRP, IoGetCurrentIrpStackLocation, PEPROCESS, DbgPrintEx}};

use crate::{logger::APP_LOGGER, handler::{handler_get_modules, handler_read, handler_protection_toggle}, kdef::{DPFLTR_LEVEL, PsGetProcessId, IoGetCurrentProcess, _OB_PRE_DUPLICATE_HANDLE_INFORMATION, _OB_PRE_CREATE_HANDLE_INFORMATION}, kapi::IrpEx};

mod panic_hook;
mod logger;
mod handler;
mod kapi;
mod kdef;

extern crate alloc;

static REQUEST_HANDLER: SyncUnsafeCell<Option<HandlerRegistry>> = SyncUnsafeCell::new(Option::None);
static VARHAL_DEVICE: SyncUnsafeCell<Option<VarhalDevice>> = SyncUnsafeCell::new(Option::None);
static PROCESS_PROTECTION: SyncUnsafeCell<Option<ProcessProtection>> = SyncUnsafeCell::new(Option::None);

struct VarhalDevice {
    _device: DeviceHandle,
    dos_link_name: UNICODE_STRING,
}

unsafe impl Sync for VarhalDevice {}
impl VarhalDevice {
    pub fn create(driver: &mut DRIVER_OBJECT) -> anyhow::Result<Self> {
        let dos_name = UNICODE_STRING::from_bytes(obfstr::wide!("\\DosDevices\\valthrun"));
        let device_name = UNICODE_STRING::from_bytes(obfstr::wide!("\\Device\\valthrun"));

        let mut device = DeviceHandle::create(
            driver,  
            &device_name, 
            DEVICE_TYPE::FILE_DEVICE_UNKNOWN, // FILE_DEVICE_UNKNOWN
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
    log::info!("Unloading...");

    /* Remove the device */
    let device_handle = unsafe { &mut *VARHAL_DEVICE.get() };
    let _ = device_handle.take();
    
    /* Delete request handler registry */
    let request_handler = unsafe { &mut *REQUEST_HANDLER.get() };
    let _ = request_handler.take();

    /* Uninstall process protection */
    let process_protection = unsafe { &mut *PROCESS_PROTECTION.get() };
    let _ = process_protection.take();

    log::info!("Driver Unloaded");
}

extern "system" fn irp_create(_device: &mut DEVICE_OBJECT, irp: &mut IRP) -> NTSTATUS {
    log::debug!("IRP create callback");

    irp.complete_request(STATUS_SUCCESS)
}

extern "system" fn irp_close(_device: &mut DEVICE_OBJECT, irp: &mut IRP) -> NTSTATUS {
    log::debug!("IRP close callback");
    irp.complete_request(STATUS_SUCCESS)
}

extern "system" fn irp_control(_device: &mut DEVICE_OBJECT, irp: &mut IRP) -> NTSTATUS {
    let outbuffer = irp.UserBuffer;
    let stack = unsafe { &mut *IoGetCurrentIrpStackLocation(irp) };
    let param = unsafe { stack.Parameters.DeviceIoControl() };
    let request_code = param.IoControlCode;

    let handler = match unsafe { REQUEST_HANDLER.get().as_ref() }.map(Option::as_ref).flatten() {
        Some(handler) => handler,
        None => {
            log::warn!("Missing request handlers");
            return irp.complete_request(STATUS_INVALID_PARAMETER);
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
        return irp.complete_request(STATUS_INVALID_PARAMETER);
    }

    let outbuffer = unsafe {
        core::slice::from_raw_parts_mut(outbuffer as *mut u8, param.OutputBufferLength as usize)
    };
    let outbuffer_probe = kapi::try_seh(|| unsafe {
        ProbeForWrite(outbuffer.as_mut_ptr() as *mut (), outbuffer.len(), 1);
    });
    if let Err(err) = outbuffer_probe {
        log::warn!("IRP request outbuffer invalid: {}", err);
        return irp.complete_request(STATUS_INVALID_PARAMETER);
    }

    match handler.handle(request_code, inbuffer, outbuffer) {
        Ok(_) => irp.complete_request(STATUS_SUCCESS),
        Err(error) => {
            log::error!("IRP handle error: {}", error);
            irp.complete_request(STATUS_INVALID_PARAMETER)
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

type ProtectionContextHandle = Arc<FastMutex<ProtectionContext>>;
struct ProtectionContext {
    protected_process_ids: Vec<i32>
}

impl Drop for ProtectionContext {
    fn drop(&mut self) {
        log::debug!("ProtectionContext dropped!");
    }
}

fn get_process_name<'a>(handle: PEPROCESS) -> Option<String> {
    let image_file_name = unsafe {
        (handle as *const ()).byte_offset(0x5a8) // FIXME: Hardcoded offset ImageFileName
            .cast::<[u8; 15]>()
            .read()
    };

    CStr::from_bytes_until_nul(image_file_name.as_slice())
        .map(|value| value.to_str().ok())
        .ok()
        .flatten()
        .map(|s| s.to_string())
}

extern "system" fn process_protection_callback(ctx: PVOID, info: *const _OB_PRE_OPERATION_INFORMATION) -> u32 {
    let info = unsafe { &*info };

    let current_process = unsafe { IoGetCurrentProcess() };
    if current_process == info.Object || (info.Flags & 0x01) > 0 {
        /* own attachments and attachments from the kernel are allowed */
        return 0;
    }

    let target_process_id = unsafe { PsGetProcessId(info.Object) };
    if log::log_enabled!(target: "ProcessAttachments", Level::Trace) && false {
        let current_process_name = get_process_name(current_process).unwrap_or_default();
        if current_process_name != "svchost.exe" && current_process_name != "WmiPrvSE.exe" {
            let current_process_id = unsafe { PsGetProcessId(current_process) };
            log::trace!("process_protection_callback. Caller: {:X} ({:?}), Target: {:X} ({:?}) Flags: {:X}, Operation: {:X}", 
                current_process_id, current_process_name, 
                target_process_id, get_process_name(info.Object as PEPROCESS), 
                info.Flags, info.Operation);
        }
    }
    
    let context = unsafe { &*core::mem::transmute::<_, *const FastMutex<ProtectionContext>>(ctx) };
    let is_protected = {
        let context = context.lock();
        context.protected_process_ids.contains(&target_process_id)
    };

    if !is_protected {
        /* all is good :) */
        return 0;
    }

    let current_process_name = get_process_name(current_process).unwrap_or_default();
    let current_process_id = unsafe { PsGetProcessId(current_process) };
    log::debug!("Process {:X} ({:?}) tries to open a handle to the protected process {:X} ({:?}) (Operation: {:X})", 
        current_process_id, current_process_name, 
        target_process_id, get_process_name(info.Object as PEPROCESS), 
        info.Operation);

    match info.Operation {
        OB_OPERATION_HANDLE_CREATE => {
            let parameters = unsafe {
                &mut *core::mem::transmute::<_, *mut _OB_PRE_CREATE_HANDLE_INFORMATION>(info.Parameters)
            };
            
            // SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION
            parameters.DesiredAccess = 0x00100000 | 0x1000;
        },
        OB_OPERATION_HANDLE_DUPLICATE => {
            let parameters = unsafe {
                &mut *core::mem::transmute::<_, *mut _OB_PRE_DUPLICATE_HANDLE_INFORMATION>(info.Parameters)
            };

            // SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION
            parameters.DesiredAccess = 0x00100000 | 0x1000;
        },
        op => log::warn!("Tried to protect {target_process_id:X} but operation {op} unknown."),
    }
    0
}

struct ProcessProtection {
    ob_registration: Option<PVOID>,
    context: ProtectionContextHandle,
}

unsafe impl Send for ProcessProtection { }
unsafe impl Sync for ProcessProtection { }

impl ProcessProtection {
    pub fn new() -> anyhow::Result<Self> {
        let mut result = Self{
            ob_registration: None,
            context: Arc::new(FastMutex::new(ProtectionContext {
                protected_process_ids: Vec::with_capacity(16)
            }))
        };
        result.register_ob_callback()?;
        Ok(result)
    }

    pub fn toggle_protection(&self, target_process_id: i32, target: bool) {
        let mut context = self.context.lock();
        if target {
            if !context.protected_process_ids.contains(&target_process_id) {
                context.protected_process_ids.push(target_process_id);
            }

            log::debug!("Enabled process protection for {}", target_process_id);
        } else {
            if let Some(index) = context.protected_process_ids.iter().position(|id| *id == target_process_id) {
                context.protected_process_ids.swap_remove(index);
                log::debug!("Disabled process protection for {}", target_process_id);
            }
        }
    }

    fn unregister_ob_callback(&mut self) {
        let result = match self.ob_registration.take() {
            Some(value) => value,
            None => return,
        };

        unsafe { 
            ObUnRegisterCallbacks(result);
            Arc::decrement_strong_count(&self.context);
        };
    }

    fn register_ob_callback(&mut self) -> anyhow::Result<()> {
        if self.ob_registration.is_some() {
            anyhow::bail!("ob callback already registered");
        }

        let mut reg_handle = core::ptr::null_mut();
        self.ob_registration = unsafe {
            let mut operation_reg = core::mem::zeroed::<_OB_OPERATION_REGISTRATION>();
            operation_reg.ObjectType = PsProcessType;
            operation_reg.Operations = OB_OPERATION_HANDLE_CREATE | OB_OPERATION_HANDLE_DUPLICATE;
            operation_reg.PostOperation = None;
            operation_reg.PreOperation = Some(process_protection_callback);

            let mut callback_reg = core::mem::zeroed::<_OB_CALLBACK_REGISTRATION>();
            callback_reg.Version = OB_FLT_REGISTRATION_VERSION;
            callback_reg.Altitude = UNICODE_STRING::from_bytes(obfstr::wide!("666")); /* Yes we wan't to be one of the first */
            callback_reg.RegistrationContext = Arc::as_ptr(&self.context) as PVOID;
            callback_reg.OperationRegistration = &operation_reg;
            callback_reg.OperationRegistrationCount = 1;

            ObRegisterCallbacks(&callback_reg, &mut reg_handle)
                .ok()
                .map_err(|err| anyhow!("ObRegisterCallbacks {}", err))?;

            Arc::increment_strong_count(&self.context);
            Some(reg_handle)
        };

        Ok(())
    }
}

impl Drop for ProcessProtection {
    fn drop(&mut self) {
        self.unregister_ob_callback();
    }
}

#[no_mangle]
pub extern "system" fn driver_entry(driver: &mut DRIVER_OBJECT) -> NTSTATUS {
    log::set_max_level(log::LevelFilter::Trace);
    if log::set_logger(&APP_LOGGER).is_err() {
        unsafe { 
            DbgPrintEx(0, DPFLTR_LEVEL::ERROR as u32, "[VT] Failed to initialize app logger!\n\0".as_ptr());
        }
        return STATUS_FAILED_DRIVER_ENTRY;
    }

    log::info!("Initialize driver");
    driver.DriverUnload = Some(driver_unload);
    driver.MajorFunction[0x00] = Some(irp_create); /* IRP_MJ_CREATE */
    driver.MajorFunction[0x02] = Some(irp_close); /* IRP_MJ_CLOSE */
    driver.MajorFunction[0x0E] = Some(irp_control); /* IRP_MJ_DEVICE_CONTROL */
    
    // TODO: PsSetCreateProcessNotifyRoutineEx(ProcessNotifyCallbackEx, FALSE);

    let process_protection = match ProcessProtection::new() {
        Ok(process_protection) => process_protection,
        Err(error) => {
            log::error!("Failed to initialized process protection: {}", error);
            return STATUS_FAILED_DRIVER_ENTRY;
        }
    };
    unsafe { *PROCESS_PROTECTION.get() = Some(process_protection) };

    let device = match VarhalDevice::create(driver) {
        Ok(device) => device,
        Err(error) => {
            log::error!("Failed to initialize device: {}", error);
            return STATUS_FAILED_DRIVER_ENTRY;
        }
    };
    log::debug!("Driver Object at 0x{:X}, Device Object at 0x{:X}", driver as *const _ as u64, device._device.0 as *const _ as u64);
    unsafe { *VARHAL_DEVICE.get() = Some(device) };

    let mut handler = HandlerRegistry::new();
    handler.register::<RequestHealthCheck>(&|_req, res| {
        res.success = true;
        Ok(())
    });
    handler.register::<RequestCSModule>(&handler_get_modules);
    handler.register::<RequestRead>(&handler_read);
    handler.register::<RequestProtectionToggle>(&handler_protection_toggle);

    unsafe { *REQUEST_HANDLER.get() = Some(handler) };

    log::warn!("TODO: RegisterOBCallback!");

    log::info!("Driver Initialized");
    STATUS_SUCCESS
}
