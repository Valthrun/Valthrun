use valthrun_kernel_interface::KernelInterface;

pub fn main() -> anyhow::Result<()> {
    let interface = KernelInterface::create("\\\\.\\valthrun")?;

    let invalid_ptr = 0xFFFFFFFFFFFFFFFFu64;
    let read_value = interface.read::<u64>(std::process::id() as i32, &[
        &invalid_ptr as *const _ as u64,
        0x0
    ]);

    println!("Read result: {:X?}", read_value);
    Ok(())
}