use std::cell::SyncUnsafeCell;

use valthrun_driver_shared::requests::RequestProtectionToggle;
use valthrun_kernel_interface::KernelInterface;

const BUFFER_ENTRY_COUNT: usize = 10_000;
static BUFFER_PTR: SyncUnsafeCell<SPtr> = SyncUnsafeCell::new(SPtr(std::ptr::null()));

static UBUFFER_PTR: SyncUnsafeCell<SPtr> = SyncUnsafeCell::new(SPtr(std::ptr::null()));

struct SPtr(*const u64);
unsafe impl Sync for SPtr {}

fn thread_worker() {
    {
        let mut buffer = Box::new(Vec::new());
        
        buffer.resize(BUFFER_ENTRY_COUNT, 0u64);
        unsafe { *UBUFFER_PTR.get() = SPtr(buffer.as_ptr()) };

        for (index, entry) in buffer.iter_mut().enumerate() {
            *entry = index as u64;
        }

        std::mem::forget(buffer);
    }

    loop {
        let mut buffer = Box::new(Vec::new());
        
        buffer.resize(BUFFER_ENTRY_COUNT, 0u64);
        unsafe { *BUFFER_PTR.get() = SPtr(buffer.as_ptr()) };

        {
            let mut x = Vec::new();
            x.resize(88, 0u8);
            std::mem::forget(x);
        }

        for (index, entry) in buffer.iter_mut().enumerate() {
            *entry = index as u64;
        }

        buffer.resize(0, 0);
        drop(buffer);
    }
}

pub fn main() -> anyhow::Result<()> {
    let interface = KernelInterface::create("\\\\.\\valthrun")?;
    interface.execute_request(&RequestProtectionToggle{ enabled: true })?;

    let _worker = std::thread::spawn(thread_worker);
    loop {
        /* Long read */
        let result = interface.read_vec::<u64>(std::process::id() as i32, &[
            &BUFFER_PTR as *const _ as *const () as u64,
            0x0
        ], BUFFER_ENTRY_COUNT);

        if let Err(err) = result {
            println!("Long {:#}", err);
        }

        /* Only a few bytes read */
        let result = interface.read_vec::<u64>(std::process::id() as i32, &[
            &BUFFER_PTR as *const _ as *const () as u64,
            0x0
        ], 16);

        if let Err(err) = result {
            println!("Short {:#}", err);
        }
        
        /* Long read U */
        let result = interface.read_vec::<u64>(std::process::id() as i32, &[
            &UBUFFER_PTR as *const _ as *const () as u64,
            0x0
        ], BUFFER_ENTRY_COUNT);

        if let Err(err) = result {
            println!("Long U {:#}", err);
        }
    }
    Ok(())
}