use crate::{
    CS2ModuleInfo,
    KeyboardState,
    MouseState,
    IO_MAX_DEREF_COUNT,
};

pub trait DriverRequest: Sized {
    type Result: Sized + Default;

    fn control_code() -> u32 {
        (0x00000022 << 16) | // FILE_DEVICE_UNKNOWN
        (0x00000000 << 14) | // FILE_SPECIAL_ACCESS
        (0x00000001 << 13) | // Custom access code
        ((Self::function_code() as u32 & 0x3FF) << 02) |
        (0x00000003 << 00)
    }

    /// The 10 bit user function code for the request
    fn function_code() -> u16;
}

pub struct RequestHealthCheck;
#[derive(Debug, Default)]
pub struct ResponseHealthCheck {
    pub success: bool,
}

impl DriverRequest for RequestHealthCheck {
    type Result = ResponseHealthCheck;

    fn function_code() -> u16 {
        0x01
    }
}

pub struct RequestCSModule;
#[derive(Debug)]
pub enum ResponseCsModule {
    Success(CS2ModuleInfo),
    UbiquitousProcesses(usize),
    NoProcess,
}
impl Default for ResponseCsModule {
    fn default() -> Self {
        Self::NoProcess
    }
}
impl DriverRequest for RequestCSModule {
    type Result = ResponseCsModule;

    fn function_code() -> u16 {
        0x02
    }
}

pub struct RequestRead {
    pub process_id: i32,

    pub offsets: [u64; IO_MAX_DEREF_COUNT],
    pub offset_count: usize,

    pub buffer: *mut u8,
    pub count: usize,
}
#[derive(Debug)]
pub enum ResponseRead {
    Success,
    InvalidAddress {
        resolved_offsets: [u64; IO_MAX_DEREF_COUNT],
        resolved_offset_count: usize,
    },
    UnknownProcess,
}
impl Default for ResponseRead {
    fn default() -> Self {
        Self::InvalidAddress {
            resolved_offsets: Default::default(),
            resolved_offset_count: 0,
        }
    }
}
impl DriverRequest for RequestRead {
    type Result = ResponseRead;

    fn function_code() -> u16 {
        0x03
    }
}

pub struct RequestProtectionToggle {
    pub enabled: bool,
}
#[derive(Default)]
pub struct ResponseProtectionToggle;

impl DriverRequest for RequestProtectionToggle {
    type Result = ResponseProtectionToggle;

    fn function_code() -> u16 {
        0x04
    }
}

pub struct RequestMouseMove {
    pub buffer: *const MouseState,
    pub state_count: usize,
}
#[derive(Default)]
pub struct ResponseMouseMove;

impl DriverRequest for RequestMouseMove {
    type Result = ResponseMouseMove;

    fn function_code() -> u16 {
        0x05
    }
}

pub struct RequestKeyboardState {
    pub buffer: *const KeyboardState,
    pub state_count: usize,
}
#[derive(Default)]
pub struct ResponseKeyboardState;

impl DriverRequest for RequestKeyboardState {
    type Result = ResponseKeyboardState;

    fn function_code() -> u16 {
        0x06
    }
}
