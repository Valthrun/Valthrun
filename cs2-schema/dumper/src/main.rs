use std::{
    collections::BTreeMap,
    fs::File,
    io::BufWriter,
    path::{
        self,
        PathBuf,
    },
};

use anyhow::Context;
use clap::Parser;
use cs2::{
    CS2Handle,
    CS2Offset,
    InterfaceError,
    StateBuildInfo,
    StateCS2Handle,
    StateCS2Memory,
    StateResolvedOffset,
};
use cs2_schema_definition::DumpedSchema;
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

fn dump_offsets(states: &StateRegistry) -> anyhow::Result<BTreeMap<String, u64>> {
    let mut result = BTreeMap::<String, u64>::new();
    for offset in CS2Offset::available_offsets() {
        let resolved = states.resolve::<StateResolvedOffset>(*offset)?;
        result.insert(offset.cache_name().to_string(), resolved.offset);
    }
    Ok(result)
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

    let mut schema = DumpedSchema::default();
    schema.scopes = cs2::dump_schema(
        &state,
        if args.client_only {
            Some(&["client.dll", "!GlobalTypes"])
        } else {
            None
        },
    )?;
    schema.resolved_offsets = self::dump_offsets(&state).context("module offsets")?;

    {
        let build_info = state.resolve::<StateBuildInfo>(())?;
        schema.cs2_build_datetime = build_info.build_datetime.clone();
        schema.cs2_revision = build_info.revision.clone();
    }

    let output = File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&args.target_file)?;

    let mut output = BufWriter::new(output);
    serde_json::to_writer_pretty(&mut output, &schema)?;

    let absolute_path = path::absolute(&args.target_file).unwrap_or(args.target_file.clone());
    log::info!(
        "Schema for CS2 version {} ({}) dumped to {}",
        schema.cs2_revision,
        schema.cs2_build_datetime,
        absolute_path.display()
    );
    Ok(())
}
