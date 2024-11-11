use std::mem;

use valthrun_driver_protocol::{
    command::DriverCommand,
    utils::str_to_fixed_buffer,
    CommandResult,
};

trait HandlerInvoker: Send + Sync {
    fn invoke(&self, command: &mut [u8], error_buffer: &mut [u8]) -> CommandResult;
}

struct HandlerImpl<C: DriverCommand + 'static> {
    inner: &'static (dyn Fn(&mut C) -> anyhow::Result<()> + Send + Sync),
}

impl<C: DriverCommand + 'static> HandlerInvoker for HandlerImpl<C> {
    fn invoke(&self, command: &mut [u8], error_buffer: &mut [u8]) -> CommandResult {
        if mem::size_of::<C>() != command.len() {
            let message = format!(
                "command size miss match (expected {}, received: {})",
                mem::size_of::<C>(),
                command.len()
            );

            str_to_fixed_buffer(error_buffer, &message);
            return CommandResult::CommandParameterInvalid;
        }

        let command = unsafe { &mut *(command.as_mut_ptr() as *mut C) };
        match (self.inner)(command) {
            Ok(_) => CommandResult::Success,
            Err(error) => {
                let message = format!("{:#}", error);
                str_to_fixed_buffer(error_buffer, &message);
                return CommandResult::Error;
            }
        }
    }
}

pub struct HandlerRegistry {
    handler: [Option<Box<dyn HandlerInvoker>>; 0x20],
}

pub const COMMAND_ID_MAX: u32 = 0x10;
impl HandlerRegistry {
    pub fn new() -> Self {
        Self {
            handler: [const { None }; 0x20],
        }
    }

    pub fn register<C: DriverCommand>(
        &mut self,
        handler: &'static (dyn Fn(&mut C) -> anyhow::Result<()> + Send + Sync),
    ) {
        assert!(C::COMMAND_ID < COMMAND_ID_MAX);
        self.handler[C::COMMAND_ID as usize] = Some(Box::new(HandlerImpl { inner: handler }))
    }

    pub fn handle(
        &self,
        command_id: u32,
        command: &mut [u8],
        error_buffer: &mut [u8],
    ) -> CommandResult {
        if command_id >= COMMAND_ID_MAX {
            return CommandResult::CommandInvalid;
        }

        let handler = match &self.handler[command_id as usize] {
            Some(handler) => handler,
            None => return CommandResult::CommandInvalid,
        };

        //log::trace!("Invoking handler 0x{:X}", function_code);
        handler.invoke(command, error_buffer)
    }
}
