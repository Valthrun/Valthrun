use std::{
    fs::File,
    io::BufWriter,
    path::{
        self,
        PathBuf,
    },
};

use clap::Parser;
use cs2::{
    CS2Handle,
    InterfaceError,
    StateCS2Handle,
    StateCS2Memory,
};
use log::LevelFilter;
use utils_state::StateRegistry;

#[derive(Debug, Parser)]
#[clap(version)]
struct Args {
    /// Target file path where the dumped schema (offsets) should be stored.
    pub target_file: PathBuf,

    /// Only dump client.dll and !GlobalTypes offsets.  
    /// This reduces the schema file sized but does not contains all classes/enum required
    /// to generate the schema definitions but should be enough for providing runtime offsets.
    #[clap(long, short)]
    pub client_only: bool,
}

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    let args = Args::parse();

    if args.client_only {
        log::info!("Dumping schema (client only). Please wait...");
    } else {
        log::info!("Dumping schema. Please wait...");
    }

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

    let absolute_path = path::absolute(&args.target_file).unwrap_or(args.target_file.clone());
    log::info!("Schema dumped to {}", absolute_path.display());
    Ok(())
}
