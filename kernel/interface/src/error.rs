use std::ffi::NulError;

use thiserror::Error;
use valthrun_driver_shared::IO_MAX_DEREF_COUNT;

#[derive(Error, Debug)]
pub enum KInterfaceError {
    #[error("kernel interface path contains invalid characters")]
    DeviceInvalidPath(NulError),

    #[error("kernel interface unavailable: {0}")]
    DeviceUnavailable(windows::core::Error),

    #[error("request failed (DeviceIoControl)")]
    RequestFailed,

    #[error("provided {provided} offsets but only {limit} are supported")]
    TooManyOffsets { provided: usize, limit: usize },

    #[error("failed to read {target_address:X} ({resolved_offset_count}/{offset_count})")]
    InvalidAddress {
        target_address: u64,

        resolved_offsets: [u64; IO_MAX_DEREF_COUNT],
        resolved_offset_count: usize,

        offsets: [u64; IO_MAX_DEREF_COUNT],
        offset_count: usize,
    },

    #[error("the target process does no longer exists")]
    ProcessDoesNotExists,

    #[error("unknown data store error")]
    Unknown,
}

pub type KResult<T> = std::result::Result<T, KInterfaceError>;
