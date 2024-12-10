use std::{
    thread,
    time::Duration,
};

use anyhow::Context;
use clap::Parser;
use valthrun_driver_interface::{
    DriverInterface,
    ProcessId,
};
use valthrun_driver_protocol::types::DirectoryTableType;

#[derive(Debug, Parser)]
struct Args {
    /// Process id of the process the modules should be listed of.
    /// By default the own process.
    #[arg(short, long)]
    pub process_id: Option<ProcessId>,

    #[command(subcommand)]
    pub directory_table_type: Option<CommandDTT>,
}

#[derive(Parser, Clone, Debug)]
enum CommandDTT {
    /// Use the process directory table base specified by the system
    Default,

    /// Manually specify the directory table base for the target process
    Explicit { directory_table_base: u64 },

    /// Try to mitigate CR3 shenanigans and do not use the directory table base known to the system
    Cr3Shenanigans,
}

pub fn main() -> anyhow::Result<()> {
    env_logger::builder().parse_default_env().init();
    let args = Args::parse();

    let interface = DriverInterface::create_from_env()?;

    let process_id = args.process_id.unwrap_or(std::process::id());
    let directory_table_type = match args.directory_table_type.unwrap_or(CommandDTT::Default) {
        CommandDTT::Default => DirectoryTableType::Default,
        CommandDTT::Explicit {
            directory_table_base,
        } => DirectoryTableType::Explicit {
            directory_table_base,
        },
        CommandDTT::Cr3Shenanigans => {
            log::debug!("Enable CR3 shenanigans in driver.");
            {
                interface
                    .enable_cr3_shenanigan_mitigation(0, 0)
                    .context("enable CR3 shenanigan mitigation")?;

                /* sleep a little so the CR3 shenanigan mitigation can have some effect */
                thread::sleep(Duration::from_millis(250));
            }
            DirectoryTableType::Cr3Shenanigans
        }
    };

    let modules = interface.list_modules(process_id, directory_table_type)?;
    println!("Process has {} modules:", modules.len());
    for module in modules {
        println!(
            " - {:X} {} ({} bytes)",
            module.base_address,
            module.get_base_dll_name().unwrap_or("unknown"),
            module.module_size
        );
    }

    if matches!(directory_table_type, DirectoryTableType::Cr3Shenanigans) {
        match interface.disable_cr3_shenanigan_mitigation() {
            Ok(_) => log::debug!("CR3 shenanigan mitigations disabled again."),
            Err(err) => log::warn!("Failed to disable CR3 shenanigan mitigations: {}", err),
        }
    }

    Ok(())
}
