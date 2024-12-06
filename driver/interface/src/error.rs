use obfstr::obfstr;
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
    DriverProtocolMismatch {
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

    #[error("failed to access memory because the target memory has been paged out")]
    MemoryAccessPagedOut,

    #[error("metrics report type too long")]
    ReportTypeTooLong,
}

pub type IResult<T> = std::result::Result<T, InterfaceError>;

impl InterfaceError {
    pub fn detailed_message(&self) -> Option<String> {
        Some(match self {
            &InterfaceError::NoDriverFound => {
                [
                    obfstr!("** PLEASE READ CAREFULLY **"),
                    obfstr!("No driver interface for the driver communication found."),
                    obfstr!("Ensure that the according \"driver_[...].dll\" file is present."),
                    obfstr!(""),
                    obfstr!("For more information please refer to"),
                    obfstr!("https://wiki.valth.run/troubleshooting/overlay/driver_interface_missing"),
                ].join("\n")
            },
            &InterfaceError::InitializeDriverUnavailable => {
                [
                    obfstr!("** PLEASE READ CAREFULLY **"),
                    obfstr!("Could not communicate with the driver."),
                    obfstr!("Most likely the Valthrun driver did not load successfully."),
                    obfstr!(""),
                    obfstr!("For more information please refer to"),
                    obfstr!(
                        "https://wiki.valth.run/troubleshooting/overlay/driver_interface_unavailable"
                    ),
                ].join("\n")
            }
            &InterfaceError::DriverProtocolMismatch {
                interface_protocol,
                driver_protocol,
            } => {
                [
                    obfstr!("Driver protocol mismatch."),
                    obfstr!("The driver interface is too old or new to be used with the current version of this application."),
                    &format!("{}: {}", obfstr!("Driver protocol version"), driver_protocol),
                    &format!("{}: {}", obfstr!("Application protocol version"), interface_protocol),
                    obfstr!(""),
                    obfstr!("For more information please refer to"),
                    obfstr!(
                        "https://wiki.valth.run/troubleshooting/overlay/driver_protocol_mismatch"
                    ),
                ].join("\n")
            }
            &InterfaceError::ProcessUnknown => {
                [
                    obfstr!("Could not find CS2 process."),
                    obfstr!("Please start CS2 prior to executing this application!"),
                ].join("\n")
            }
            _ => return None,
        })
    }
}
