use clap::Parser;
use valthrun_driver_interface::DriverInterface;

#[derive(Debug, Parser)]
struct Args {}

pub fn main() -> anyhow::Result<()> {
    env_logger::builder().parse_default_env().init();
    let _args = Args::parse();

    let interface = DriverInterface::create_from_env()?;
    let processes = interface.list_processes()?;
    println!("Process count: {}", processes.len());
    for process in processes {
        println!(
            " - {: >5} {} (directory table base = {:X})",
            process.process_id,
            process.get_image_base_name().unwrap_or("<invalid>"),
            process.directory_table_base
        );
    }

    Ok(())
}
