use std::ffi::NulError;

use thiserror::Error;
use valthrun_driver_shared::IO_MAX_DEREF_COUNT;

#[derive(Error, Debug)]
pub enum KInterfaceError {
    #[error("initialization returned invalid status code ({0:X})")]
    InitializeInvalidStatus(u32),

    #[error("kernel driver is too old (version: {driver_version_string}, requested: {requested_version_string})")]
    DriverTooOld {
        driver_version: u32,
        driver_version_string: String,

        requested_version: u32,
        requested_version_string: String,
    },

    #[error("kernel driver (version: {driver_version_string}) is newer then the requested version {requested_version_string} and does not support the requested version")]
    DriverTooNew {
        driver_version: u32,
        driver_version_string: String,

        requested_version: u32,
        requested_version_string: String,
    },

    #[error("kernel interface path contains invalid characters")]
    DeviceInvalidPath(NulError),

    #[error("kernel interface unavailable: {0}")]
    DeviceUnavailable(windows::core::Error),

    #[error("request failed (DeviceIoControl)")]
    RequestFailed,

    #[error("provided {provided} offsets but only {limit} are supported")]
    TooManyOffsets { provided: usize, limit: usize },

    #[error(
        "failed to acceess memory at 0x{target_address:X} ({resolved_offset_count}/{offset_count})"
    )]
    InvalidAddress {
        target_address: u64,

        resolved_offsets: [u64; IO_MAX_DEREF_COUNT],
        resolved_offset_count: usize,

        offsets: [u64; IO_MAX_DEREF_COUNT],
        offset_count: usize,
    },

    #[error("the target process does no longer exists")]
    ProcessDoesNotExists,

    #[error("could not identify process as the name is not ubiquitous")]
    ProcessNotUbiquitous,

    #[error("the requested memory access mode is unavailable")]
    AccessModeUnavailable,

    #[error("unknown data store error")]
    Unknown,
}

pub type KResult<T> = std::result::Result<T, KInterfaceError>;
