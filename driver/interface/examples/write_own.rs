use valthrun_driver_interface::DriverInterface;

pub fn main() -> anyhow::Result<()> {
    env_logger::builder().parse_default_env().init();

    let interface = DriverInterface::create_from_env()?;

    let target_value = 0x01u64;
    interface.write::<u64>(std::process::id(), &target_value as *const _ as u64, &0x42)?;

    println!("Target value: {:X}", target_value);
    Ok(())
}
