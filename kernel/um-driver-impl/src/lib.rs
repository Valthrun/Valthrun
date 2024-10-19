use core::slice;

use lazy_static::lazy_static;
use registry::HandlerRegistry;
use windows::Win32::System::SystemServices::{
    DLL_PROCESS_ATTACH,
    DLL_PROCESS_DETACH,
};

mod handle;
mod handler;
mod registry;
mod util;

lazy_static! {
    static ref REQUEST_HANDLER: HandlerRegistry = init_request_handler();
}

fn init_request_handler() -> HandlerRegistry {
    let mut handler = HandlerRegistry::new();

    handler.register(&handler::health);
    handler.register(&handler::get_cs2_modules);
    handler.register(&handler::read);
    // handler.register(&handler::write);
    // handler.register(&handler::protection_toggle);
    handler.register(&handler::mouse_move);
    handler.register(&handler::keyboard_state);
    handler.register(&handler::init);
    // handler.register(&handler::metrics_record);
    handler.register(&handler::get_modules);

    handler
}

#[no_mangle]
#[allow(non_snake_case)]
extern "system" fn DllMain(_dll_module: *const (), call_reason: u32, _: *mut ()) -> bool {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            env_logger::init();
        }
        DLL_PROCESS_DETACH => (),
        _ => (),
    }

    true
}

#[no_mangle]
extern "C" fn execute_request(
    function_code: u16,
    request: *const u8,
    request_length: usize,
    response: *mut u8,
    response_length: usize,
) -> u32 {
    let request = unsafe { slice::from_raw_parts(request, request_length) };
    let response = unsafe { slice::from_raw_parts_mut(response, response_length) };

    match REQUEST_HANDLER.handle(function_code, request, response) {
        Ok(_) => 1,
        Err(err) => {
            log::error!("Failed to handle {:X}: {:#?}", function_code, err);
            0
        }
    }
}
