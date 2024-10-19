use valthrun_driver_shared::requests::{
    RequestHealthCheck,
    ResponseHealthCheck,
};

pub fn health(_req: &RequestHealthCheck, res: &mut ResponseHealthCheck) -> anyhow::Result<()> {
    *res = ResponseHealthCheck { success: true };
    Ok(())
}
