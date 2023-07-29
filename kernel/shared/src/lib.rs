#![no_std]

pub const IO_MAX_DEREF_COUNT: usize = 31;

pub mod requests;

#[derive(Debug, Default)]
pub struct ModuleInfo {
    pub base_address: u64,
    pub module_size: usize,
}

#[derive(Debug, Default)]
pub struct CSModuleInfo {
    pub process_id: i32,

    pub client: ModuleInfo,
    pub engine: ModuleInfo,
    pub schemasystem: ModuleInfo,
}