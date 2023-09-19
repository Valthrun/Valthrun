use kinterface::KernelInterface;

pub fn main() -> anyhow::Result<()> {
    let interface = KernelInterface::create("\\\\.\\valthrun")?;

    let target_value = 0x42u64;
    let read_value = interface.read::<u64>(std::process::id() as i32, &[
        &target_value as *const _ as u64
    ]);

    println!("Read result: {:X?}", read_value);
    Ok(())
}