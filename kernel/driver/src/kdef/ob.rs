use winapi::shared::ntdef::{PVOID, UNICODE_STRING, PCVOID, NTSTATUS};

use super::POBJECT_TYPE;

pub type OB_OPERATION = u32;

pub const OB_OPERATION_HANDLE_CREATE: OB_OPERATION = 0x00000001;
pub const OB_OPERATION_HANDLE_DUPLICATE: OB_OPERATION = 0x00000002;

/// Registration version for Vista SP1 and Windows Server 2007
pub const OB_FLT_REGISTRATION_VERSION_0100: u16 = 0x0100;
pub const OB_FLT_REGISTRATION_VERSION: u16 = OB_FLT_REGISTRATION_VERSION_0100;

pub type POB_PRE_OPERATION_CALLBACK = Option<extern "system" fn (RegistrationContext: PVOID, OperationInformation: *const _OB_PRE_OPERATION_INFORMATION) -> u32>;
pub type POB_POST_OPERATION_CALLBACK = Option<extern "system" fn (RegistrationContext: PVOID, OperationInformation: *const ()) -> u32>;

#[repr(C)]
pub struct _OB_PRE_CREATE_HANDLE_INFORMATION {
    pub DesiredAccess: u32,
    pub OriginalDesiredAccess: u32,
}

#[repr(C)]
pub struct _OB_PRE_DUPLICATE_HANDLE_INFORMATION {
    pub DesiredAccess: u32,
    pub OriginalDesiredAccess: u32,
    pub SourceProcess: PVOID,
    pub TargetProcess: PVOID
}

#[repr(C)]
pub struct _OB_PRE_OPERATION_INFORMATION {
    pub Operation: OB_OPERATION,
    pub Flags: u32,
    pub Object: PVOID,
    pub ObjectType: POBJECT_TYPE,
    pub CallContext: PVOID,
    pub Parameters: PVOID,
}

#[repr(C)]
pub struct _OB_OPERATION_REGISTRATION {
    pub ObjectType: *const POBJECT_TYPE,
    pub Operations: OB_OPERATION,
    pub PreOperation: POB_PRE_OPERATION_CALLBACK,
    pub PostOperation: POB_POST_OPERATION_CALLBACK,
}

#[repr(C)]
pub struct _OB_CALLBACK_REGISTRATION {
    pub Version: u16,
    pub OperationRegistrationCount: u16,
    pub Altitude: UNICODE_STRING,
    pub RegistrationContext: PCVOID,
    pub OperationRegistration: *const _OB_OPERATION_REGISTRATION,
}

#[allow(unused)]
extern "system" {
    pub fn ObRegisterCallbacks(CallbackRegistration: *const _OB_CALLBACK_REGISTRATION, RegistrationHandle: *mut PVOID) -> NTSTATUS; 
    pub fn ObUnRegisterCallbacks(RegistrationHandle: PVOID);
    pub fn ObGetFilterVersion() -> u16;
}