use std::env;

use crate::KResult;

pub trait DriverInterface: Sync + Send {
    /// Execute a request
    ///
    /// Safety:
    /// The caller must statisfy, that the request slice is a proper request.
    #[must_use]
    fn execute_request(
        &self,
        function_code: u16,
        request: &[u8],
        response: &mut [u8],
    ) -> KResult<()>;
}

pub fn com_from_env() -> KResult<Box<dyn DriverInterface>> {
    let interface: Box<dyn DriverInterface> = if let Ok(name) = env::var("VT_UM_DRIVER") {
        Box::new(UmDriverInterface::create(name.as_str())?)
    } else if let Ok(path) = env::var("VT_KM_IOCTRL_PATH") {
        Box::new(IoctrlDriverInterface::create(path.as_str())?)
    } else {
        Box::new(IoctrlDriverInterface::create(
            "\\\\.\\GLOBALROOT\\Device\\valthrun",
        )?)
    };
    Ok(interface)
}

mod ioctrl;
pub use ioctrl::*;

mod um_driver;
pub use um_driver::*;
