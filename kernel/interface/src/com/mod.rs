use crate::KResult;

pub trait DriverInterface: Sync + Send {
    /// Execute a request
    ///
    /// Safety:
    /// The caller must statisfy, that the request slice is a proper request.
    #[must_use]
    fn execute_request(
        &self,
        control_code: u32,
        request: &[u8],
        response: &mut [u8],
    ) -> KResult<()>;
}

mod ioctrl;
pub use ioctrl::*;
