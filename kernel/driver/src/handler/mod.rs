use valthrun_driver_shared::requests::DriverRequest;

mod cs_module;
pub use cs_module::*;

mod memory_read;
pub use memory_read::*;

mod protect;
pub use protect::*;

pub const FUNCTION_CODE_MAX: usize = 0x20;

type RequestHandlerGeneric = dyn (Fn(&(), &mut ()) -> anyhow::Result<()>) + Send + Sync;
struct HandlerInfo {
    handler: &'static RequestHandlerGeneric,
    input_buffer_size: usize,
    output_buffer_size: usize,
}

/// Request handler registry for handling kernel requests.
pub struct HandlerRegistry {
    handlers: [Option<HandlerInfo>; FUNCTION_CODE_MAX]
}

impl HandlerRegistry {
    pub fn new() -> Self {
        const INIT: Option<HandlerInfo> = None;
        Self {
            handlers: [INIT; FUNCTION_CODE_MAX]
        }
    }

    /// Register a request handler.
    /// Attention: 
    /// The input and output function parameters are all located in the callers user space!
    /// The struct itself has been probed for read & write and is therefore ensured to be valid.
    pub fn register<R: DriverRequest>(&mut self, handler: &'static dyn Fn(&R, &mut R::Result) -> anyhow::Result<()>) {
        assert!((R::function_code() as usize) < FUNCTION_CODE_MAX);
        self.handlers[R::function_code() as usize] = Some(HandlerInfo{
            handler: unsafe { core::mem::transmute(handler) },
            input_buffer_size: core::mem::size_of::<R>(),
            output_buffer_size: core::mem::size_of::<R::Result>()
        });
    }

    pub fn handle(&self, irp_request_code: u32, inbuffer: &[u8], outbuffer: &mut [u8]) -> anyhow::Result<()> {
        let function_code = ((irp_request_code >> 2) & 0x3F) as usize;
        if function_code >= FUNCTION_CODE_MAX {
            anyhow::bail!("invalid function code ({})", function_code)
        }

        let handler = match &self.handlers[function_code as usize] {
            Some(handler) => handler,
            None => anyhow::bail!("function {} has no handler", function_code),
        };

        if handler.input_buffer_size != inbuffer.len() {
            anyhow::bail!("inbuffer size miss match (expected {}, received: {})", handler.input_buffer_size, inbuffer.len())
        }

        if handler.output_buffer_size != outbuffer.len() {
            anyhow::bail!("outbuffer size miss match (expected {}, received: {})", handler.output_buffer_size, outbuffer.len())
        }

        unsafe {
            (*handler.handler)(&*(inbuffer.as_ptr() as *const ()), &mut *(outbuffer.as_mut_ptr() as *mut ()))
        }
    }
}
