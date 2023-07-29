use core::panic::PanicInfo;

use crate::kdef::{DPFLTR_LEVEL, DbgPrintEx, KeBugCheck};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        DbgPrintEx(0, DPFLTR_LEVEL::ERROR as u32, "[VT] Driver paniced. Trigger BugCheck.\n\0".as_ptr());
        KeBugCheck(1);
    }
}

#[used]
#[no_mangle]
pub static _fltused: i32 = 0;

#[no_mangle]
extern "C" fn __CxxFrameHandler3() -> ! {
    unsafe {
        DbgPrintEx(0, DPFLTR_LEVEL::ERROR as u32, "[VT] __CxxFrameHandler3 has been called. This should no occur.\n\0".as_ptr());
        KeBugCheck(1);
    }
}

// #[lang = "eh_personality"] extern fn eh_personality() {}
// #[lang = "panic_fmt"] extern fn panic_fmt() -> ! { loop {} }