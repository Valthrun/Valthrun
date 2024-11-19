use std::{
    fs::File,
    io::BufWriter,
    path::PathBuf,
};

use clap::Parser;
use cs2::{
    offsets_runtime,
    CS2Handle,
    InterfaceError,
    StateCS2Handle,
    StateCS2Memory,
};
use utils_state::StateRegistry;

#[derive(Debug, Parser)]
#[clap(version)]
struct Args {
    pub target_file: PathBuf,

    #[clap(long, short)]
    pub client_only: bool,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();

    log::info!("Dumping schema. Please wait...");

    let cs2 = match CS2Handle::create(false) {
        Ok(handle) => handle,
        Err(err) => {
            if let Some(err) = err.downcast_ref::<InterfaceError>() {
                if let Some(detailed_message) = err.detailed_message() {
                    for line in detailed_message.lines() {
                        log::error!("{}", line);
                    }
                    return Ok(());
                }
            }

            return Err(err);
        }
    };

    let mut state = StateRegistry::new(64);
    state.set(StateCS2Handle::new(cs2.clone()), ())?;
    state.set(StateCS2Memory::new(cs2.create_memory_view()), ())?;
    offsets_runtime::setup_provider(&state)?;

    let schema = cs2::dump_schema(
        &state,
        if args.client_only {
            Some(&["client.dll", "!GlobalTypes"])
        } else {
            None
        },
    )?;

    let output = File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&args.target_file)?;

    let mut output = BufWriter::new(output);
    serde_json::to_writer_pretty(&mut output, &schema)?;
    log::info!("Schema dumped to {}", args.target_file.to_string_lossy());
    Ok(())
}
