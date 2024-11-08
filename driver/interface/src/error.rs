use thiserror::Error;

#[derive(Error, Debug)]
pub enum InterfaceError {
    #[error("failed to find any memory driver")]
    NoDriverFound,

    #[error("failed to load driver: {0}")]
    DriverLoadingError(#[from] libloading::Error),

    #[error("missing command handler execute export")]
    DriverMissingExecuterExport,

    #[error(
        "protocol miss match (expected {interface_protocol} but driver supports {driver_protocol})"
    )]
    DriverProtocolMissMatch {
        interface_protocol: u32,
        driver_protocol: u32,
    },

    #[error("command failed: {message}")]
    CommandGenericError { message: String },

    #[error("the driver is unavailable")]
    InitializeDriverUnavailable,

    #[error("process unknown")]
    ProcessUnknown,

    #[error("process is ubiquitous")]
    ProcessUbiquitous,

    #[error("failed to access memory")]
    MemoryAccessFailed,

    #[error("metrics report type too long")]
    ReportTypeTooLong,
}

pub type IResult<T> = std::result::Result<T, InterfaceError>;
