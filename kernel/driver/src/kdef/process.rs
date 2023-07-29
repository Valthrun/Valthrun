use crate::kapi::NTSTATUS;

use super::{_LIST_ENTRY, PVOID, UNICODE_STRING};

#[link(name = "ntoskrnl")]
extern "system" {
    pub fn PsGetProcessId(process: *const _KPROCESS) -> i32;
    pub fn PsGetProcessPeb(process: *const _KPROCESS) -> *const _PEB;
    pub fn PsLookupProcessByProcessId(process_id: i32, process: *mut *const _KPROCESS) -> NTSTATUS;
    
    pub fn KeStackAttachProcess(process: *const _KPROCESS, apc_state: &mut _KAPC_STATE);
    pub fn KeUnstackDetachProcess(apc_state: &mut _KAPC_STATE);
    
    pub static PsInitialSystemProcess: *const _KPROCESS;
}

#[repr(C)]
#[allow(non_snake_case, non_camel_case_types)]
pub struct _KAPC_STATE {
    pub ApcListHead: [_LIST_ENTRY; 2],
    pub Process: *const _KPROCESS,
    pub InProgressFlags: u8,
    pub KernelApcPending: bool,
    pub UserApcPendingAll: bool
}

#[repr(C)]
#[allow(non_snake_case, non_camel_case_types)]
pub struct _PEB_LDR_DATA {
    pub Length: u32,
    pub Initialized: bool,
    pub SsHandle: PVOID,                                                    
    pub InLoadOrderModuleList: _LIST_ENTRY,                            
    pub InMemoryOrderModuleList: _LIST_ENTRY,                             
    pub InInitializationOrderModuleList: _LIST_ENTRY,                   
    pub EntryInProgress: PVOID,                                              
    pub ShutdownInProgress: u8,                                             
    pub ShutdownThreadId: PVOID,                                           
}

#[repr(C)]
#[allow(non_snake_case, non_camel_case_types)]
pub struct _LDR_DATA_TABLE_ENTRY {
    pub InLoadOrderLinks: _LIST_ENTRY,
    pub InMemoryOrderLinks: _LIST_ENTRY, 
    pub InInitializationOrderLinks: _LIST_ENTRY,                     
    pub DllBase: *const (),                                                     
    pub EntryPoint: *const (),                                                     
    pub SizeOfImage: u32,    
    pub FullDllName: UNICODE_STRING,
    pub BaseDllName: UNICODE_STRING,   

    /* More fields */                                      
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct _PEB {
    pub Reserved1: [u8; 2],
    pub BeingDebugged: bool,
    pub Reserved2: [u8; 1],
    pub Reserved3: [PVOID; 2],
    pub Ldr: *const _PEB_LDR_DATA,
    pub ProcessParameters: *const () /* PRTL_USER_PROCESS_PARAMETERS */,
    pub Reserved4: [PVOID; 3],
    pub AtlThunkSListPtr: *const (),
    pub Reserved5: *const (),
    pub Reserved6: u32,
    pub Reserved7: *const (),
    pub Reserved8: u32,
    pub AtlThunkSListPtr32: u32,
    pub Reserved9: [PVOID; 45],
    pub Reserved10: [u8; 96],
    pub PostProcessInitRoutine: *const () /* PPS_POST_PROCESS_INIT_ROUTINE */,
    pub Reserved11: [u8; 128],
    pub Reserved12: *const (),
    pub SessionId: u32,
}

pub type _KPROCESS = ();
pub type _EPROCESS = ();