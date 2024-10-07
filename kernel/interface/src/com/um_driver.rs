use libloading::Library;

use super::DriverInterface;
use crate::{
    KInterfaceError,
    KResult,
};

type FnHandler = unsafe extern "C" fn(
    function_code: u16,
    request: *const u8,
    request_length: usize,
    response: *mut u8,
    response_length: usize,
) -> u32;

pub struct UmDriverInterface {
    _library: Library,
    handler: FnHandler,
}

impl UmDriverInterface {
    pub fn create(file_name: &str) -> KResult<Self> {
        let library = unsafe { Library::new(file_name) }?;

        Ok(Self {
            handler: unsafe { *library.get::<FnHandler>(b"execute_request\0")? },
            _library: library,
        })
    }
}

impl DriverInterface for UmDriverInterface {
    fn execute_request(
        &self,
        function_code: u16,
        request: &[u8],
        response: &mut [u8],
    ) -> KResult<()> {
        let code = unsafe {
            (self.handler)(
                function_code,
                request.as_ptr(),
                request.len(),
                response.as_mut_ptr(),
                response.len(),
            )
        };

        if code == 1 {
            Ok(())
        } else {
            Err(KInterfaceError::RequestFailed)
        }
    }
}
