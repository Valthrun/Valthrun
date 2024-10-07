use valthrun_kernel_interface::{
    com_from_env,
    KernelInterface,
    ProcessFilter,
};

pub fn main() -> anyhow::Result<()> {
    env_logger::builder().parse_default_env().init();
    let interface = KernelInterface::create(com_from_env()?)?;

    let (process_id, modules) = interface.request_modules(ProcessFilter::Id {
        id: std::process::id() as i32,
    })?;
    println!("Process 0x{:X} has {} modules:", process_id, modules.len());
    for module in modules {
        println!(
            " - {:X} {} ({} bytes)",
            module.base_address,
            module.base_dll_name(),
            module.module_size
        );
    }

    Ok(())
}
