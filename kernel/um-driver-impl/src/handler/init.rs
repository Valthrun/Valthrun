use valthrun_driver_shared::requests::{
    RequestInitialize,
    ResponseInitialize,
    INIT_STATUS_SUCCESS,
};

fn driver_version() -> u32 {
    let major = env!("CARGO_PKG_VERSION_MAJOR").parse::<u32>().unwrap();
    let minor = env!("CARGO_PKG_VERSION_MINOR").parse::<u32>().unwrap();
    let patch = env!("CARGO_PKG_VERSION_PATCH").parse::<u32>().unwrap();
    return (major << 24) | (minor << 16) | (patch << 8);
}

pub fn init(_req: &RequestInitialize, res: &mut ResponseInitialize) -> anyhow::Result<()> {
    res.status_code = INIT_STATUS_SUCCESS;
    res.driver_version = driver_version();

    /* TODO: perform a version check, but as the UM driver is mainly for development purposes we ignore it */
    Ok(())
}
