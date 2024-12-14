use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use console_io::show_critical_error;
use cs2::{
    CS2Handle,
    InterfaceError,
    StateCS2Handle,
    StateCS2Memory,
};
use obfstr::obfstr;
use radar_client::{
    CS2RadarGenerator,
    WebRadarPublisher,
};
use url::Url;
use utils_state::StateRegistry;

mod console_io;

/// Standalone Valthrun CS2 radar
#[derive(Parser, Debug)]
#[command(long_about = None)]
struct Args {
    /// Target server address used to publish the web radar.
    /// Use ws://127.0.0.1:7229/publish for local development.
    #[arg(short, long, default_value = "wss://radar.valth.run/publish")]
    publish_url: String,

    /// Load the CS2 schema (offsets) from a file
    /// instead of resolving them at runtime by the CS2 schema system.
    #[arg(short, long)]
    schema_file: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    if let Err(error) = real_main(&args).await {
        show_critical_error(&format!("{:#}", error));
    }

    Ok(())
}

async fn real_main(args: &Args) -> anyhow::Result<()> {
    let url = Url::parse(&args.publish_url).context("invalid target server address")?;

    let radar_generator = {
        let cs2 = match CS2Handle::create(true) {
            Ok(cs2) => cs2,
            Err(err) => {
                if let Some(err) = err.downcast_ref::<InterfaceError>() {
                    if let Some(detailed_message) = err.detailed_message() {
                        show_critical_error(&detailed_message);
                        return Ok(());
                    }
                }

                return Err(err);
            }
        };
        let mut states = StateRegistry::new(1024 * 8);
        states.set(StateCS2Memory::new(cs2.create_memory_view()), ())?;
        states.set(StateCS2Handle::new(cs2), ())?;

        if let Some(file) = &args.schema_file {
            log::info!(
                "{} {}",
                obfstr!("Loading CS2 schema (offsets) from file"),
                file.display()
            );

            cs2_schema_provider_impl::setup_schema_from_file(&mut states, file)
                .context("file schema setup")?;
        } else {
            log::info!(
                "{}",
                obfstr!("Loading CS2 schema (offsets) from CS2 schema system")
            );
            cs2_schema_provider_impl::setup_provider(Box::new(
                cs2_schema_provider_impl::RuntimeSchemaProvider::new(&states)
                    .context("load runtime schema")?,
            ));
        }
        log::info!("CS2 schema (offsets) loaded.");

        Box::new(CS2RadarGenerator::new(states)?)
    };
    let radar_client = WebRadarPublisher::connect(radar_generator, &url).await?;

    let mut radar_url = url.clone();
    radar_url.set_path(&format!("/session/{}", radar_client.session_id));
    if radar_url.scheme() == "wss" {
        let _ = radar_url.set_scheme("https");
    } else {
        let _ = radar_url.set_scheme("http");
    }

    log::info!("Radar session {}", radar_client.session_id);
    log::info!("Available at {}", radar_url);

    radar_client.await.context("radar error")?;
    Ok(())
}
