use valthrun_kernel_interface::KernelInterface;

pub fn main() -> anyhow::Result<()> {
    env_logger::builder().parse_default_env().init();

    let interface = KernelInterface::create("\\\\.\\GLOBALROOT\\Device\\valthrun")?;

    let target_value = 0x01u64;
    interface.write::<u64>(
        std::process::id() as i32,
        &target_value as *const _ as u64,
        &0x42,
    )?;

    println!("Target value: {:X}", target_value);
    Ok(())
}
