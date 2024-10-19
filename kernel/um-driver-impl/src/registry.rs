use std::mem;

use valthrun_driver_shared::requests::DriverRequest;

trait HandlerInvoker: Send + Sync {
    fn invoke(&self, request: &[u8], response: &mut [u8]) -> anyhow::Result<()>;
}

struct HandlerImpl<R: DriverRequest + 'static> {
    inner: &'static (dyn Fn(&R, &mut R::Result) -> anyhow::Result<()> + Send + Sync),
}

impl<R: DriverRequest + 'static> HandlerInvoker for HandlerImpl<R> {
    fn invoke(&self, request: &[u8], response: &mut [u8]) -> anyhow::Result<()> {
        if mem::size_of::<R>() != request.len() {
            anyhow::bail!(
                "inbuffer size miss match (expected {}, received: {})",
                mem::size_of::<R>(),
                request.len()
            )
        }

        if mem::size_of::<R::Result>() != response.len() {
            anyhow::bail!(
                "outbuffer size miss match (expected {}, received: {})",
                mem::size_of::<R::Result>(),
                response.len()
            )
        }

        unsafe {
            (self.inner)(
                &*(request.as_ptr() as *const R),
                &mut *(response.as_mut_ptr() as *mut R::Result),
            )
        }
    }
}

pub struct HandlerRegistry {
    handler: [Option<Box<dyn HandlerInvoker>>; 0x20],
}

pub const FUNCTION_CODE_MAX: usize = 0x10;
impl HandlerRegistry {
    pub fn new() -> Self {
        Self {
            handler: [const { None }; 0x20],
        }
    }

    pub fn register<R: DriverRequest>(
        &mut self,
        handler: &'static (dyn Fn(&R, &mut R::Result) -> anyhow::Result<()> + Send + Sync),
    ) {
        assert!((R::function_code() as usize) < FUNCTION_CODE_MAX);
        self.handler[R::function_code() as usize] = Some(Box::new(HandlerImpl { inner: handler }))
    }

    pub fn handle(
        &self,
        function_code: u16,
        inbuffer: &[u8],
        outbuffer: &mut [u8],
    ) -> anyhow::Result<()> {
        if function_code as usize >= FUNCTION_CODE_MAX {
            anyhow::bail!("invalid function code (0x{:X})", function_code)
        }

        let handler = match &self.handler[function_code as usize] {
            Some(handler) => handler,
            None => anyhow::bail!("function 0x{:X} has no handler", function_code),
        };

        //log::trace!("Invoking handler 0x{:X}", function_code);
        handler.invoke(inbuffer, outbuffer)
    }
}
