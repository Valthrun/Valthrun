use std::string::FromUtf8Error;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum VPKError {
    #[error("io error")]
    IOError(#[from] std::io::Error),

    #[error("encoding error")]
    Utf8Error(#[from] FromUtf8Error),

    #[error("invalid file signature")]
    InvalidFileSignature,

    #[error("unsupported archive version ({version})")]
    UnsupportedArchiveVersion { version: u32 },

    #[error("not all data has been consumed: {step}")]
    UnconsumedData { step: String },

    #[error("invalid directory entry terminator ({0:X})")]
    InvalidDirectoryEntryTerminator(u16),

    #[error("the target entry can not be found")]
    EntryUnknown,

    #[error("the target entry is not contained in this archive")]
    EntryNotContainedInThisArchive,

    #[error("entry crc3 miss match (expected {expected:X}, calculated {calculated:X})")]
    EntryCrcMissmatch { expected: u32, calculated: u32 },
}

pub type VResult<T> = Result<T, VPKError>;
