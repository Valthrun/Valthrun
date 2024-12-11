use valthrun_driver_protocol::{
    command::{
        DriverCommandInitialize,
        InitializeResult,
        VersionInfo,
    },
    types::DriverFeature,
    PROTOCOL_VERSION,
};

fn driver_version() -> VersionInfo {
    let mut info = VersionInfo::default();
    info.set_application_name("um-driver");

    info.version_major = env!("CARGO_PKG_VERSION_MAJOR").parse::<u32>().unwrap();
    info.version_minor = env!("CARGO_PKG_VERSION_MINOR").parse::<u32>().unwrap();
    info.version_patch = env!("CARGO_PKG_VERSION_PATCH").parse::<u32>().unwrap();

    return info;
}

pub fn init(command: &mut DriverCommandInitialize) -> anyhow::Result<()> {
    command.driver_protocol_version = PROTOCOL_VERSION;
    if command.client_protocol_version != PROTOCOL_VERSION {
        /* invalid protocol */
        return Ok(());
    }

    /* We do not need to initialize any libraries */
    command.driver_version = driver_version();
    command.driver_features = DriverFeature::ProcessList
        | DriverFeature::ProcessModules
        | DriverFeature::MemoryRead
        | DriverFeature::InputMouse
        | DriverFeature::InputKeyboard;
    command.result = InitializeResult::Success;
    Ok(())
}
