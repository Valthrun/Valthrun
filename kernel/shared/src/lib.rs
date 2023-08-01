#![no_std]
#![feature(iterator_try_collect)]

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
