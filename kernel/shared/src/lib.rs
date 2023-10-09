#![no_std]

pub const IO_MAX_DEREF_COUNT: usize = 31;

pub mod requests;

mod pattern;
pub use pattern::*;

extern crate alloc;

#[derive(Debug, Default)]
pub struct ModuleInfo {
    pub base_address: usize,
    pub module_size: usize,
}

#[derive(Debug, Default)]
pub struct CS2ModuleInfo {
    pub process_id: i32,

    pub client: ModuleInfo,
    pub engine: ModuleInfo,
    pub schemasystem: ModuleInfo,
}

#[derive(Debug, Default)]
pub struct MouseState {
    pub buttons: [Option<bool>; 0x05],
    pub hwheel: bool,
    pub wheel: bool,

    pub last_x: i32,
    pub last_y: i32,
}

#[derive(Debug, Default)]
pub struct KeyboardState {
    pub scane_code: u16,
    pub down: bool,
}
