use alloc::{format, ffi::CString};
use winapi::km::wdm::DbgPrintEx;

use crate::kdef::DPFLTR_LEVEL;

pub struct KernelLogger;

impl log::Log for KernelLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return
        }

        let (level_prefix, log_level) = match record.level() {
            log::Level::Trace => ("T", DPFLTR_LEVEL::TRACE),
            log::Level::Debug => ("D", DPFLTR_LEVEL::TRACE),
            log::Level::Info => ("I", DPFLTR_LEVEL::INFO),
            log::Level::Warn => ("W", DPFLTR_LEVEL::WARNING),
            log::Level::Error => ("E", DPFLTR_LEVEL::ERROR)
        };
        let payload = format!("[{}] {}", level_prefix, record.args());
        let payload = if let Ok(payload) = CString::new(payload) {
            payload
        } else {
            CString::new("logging message contains null char").unwrap()
        };

        unsafe {
            DbgPrintEx(0, log_level as u32, "[VT]%s\n\0".as_ptr(), payload.as_ptr());
        }
    }

    fn flush(&self) { }
}

pub static APP_LOGGER: KernelLogger = KernelLogger;