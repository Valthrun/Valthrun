use valthrun_driver_interface::{
    DriverInterface,
    ProcessFilter,
};

pub fn main() -> anyhow::Result<()> {
    env_logger::builder().parse_default_env().init();
    let interface = DriverInterface::create_from_env()?;

    let (process_id, modules) = interface.request_modules(&ProcessFilter::Id {
        id: std::process::id(),
    })?;
    println!("Process 0x{:X} has {} modules:", process_id, modules.len());
    for module in modules {
        println!(
            " - {:X} {} ({} bytes)",
            module.base_address,
            module.get_base_dll_name().unwrap_or("unknown"),
            module.module_size
        );
    }

    Ok(())
}
