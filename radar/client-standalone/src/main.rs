use std::{
    path::PathBuf,
    time::Duration,
};

use anyhow::Context;
use clap::Parser;
use cs2::{
    CS2Handle,
    InterfaceError,
    StateCS2Handle,
    StateCS2Memory,
};
use obfstr::obfstr;
use radar_client::{
    CS2RadarGenerator,
    DummyRadarGenerator,
    RadarGenerator,
    WebRadarPublisher,
};
use tokio::signal;
use url::Url;
use utils_state::StateRegistry;

mod arch;

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

    /// Use a dummy generator instead of generating the radar data from CS2.
    /// This is usefull when testing the radar client without CS2.
    #[arg(long, hide = true)]
    dummy_generator: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    if let Err(error) = real_main(&args).await {
        arch::show_critical_error(&format!("{:#}", error));
    }

    Ok(())
}

const RECONNECT_INTERVAL: &[Duration] = &[
    Duration::from_secs(1),
    Duration::from_secs(5),
    Duration::from_secs(10),
    Duration::from_secs(30),
    Duration::from_secs(60),
];

async fn real_main(args: &Args) -> anyhow::Result<()> {
    let url = Url::parse(&args.publish_url).context("invalid target server address")?;

    let radar_generator: Box<dyn RadarGenerator> = if args.dummy_generator {
        Box::new(DummyRadarGenerator)
    } else {
        let cs2 = match CS2Handle::create(true) {
            Ok(cs2) => cs2,
            Err(err) => {
                if let Some(err) = err.downcast_ref::<InterfaceError>() {
                    if let Some(detailed_message) = err.detailed_message() {
                        arch::show_critical_error(&detailed_message);
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

    self::radar_publish_loop(radar_generator, &url).await
}

async fn radar_publish_loop(
    radar_generator: Box<dyn RadarGenerator>,
    url: &Url,
) -> anyhow::Result<()> {
    let mut radar_client = WebRadarPublisher::connect(&url, None).await?;
    radar_client.set_generator(radar_generator);

    let mut radar_url = url.clone();
    radar_url.set_path(&format!("/session/{}", radar_client.session_id));
    if radar_url.scheme() == "wss" {
        let _ = radar_url.set_scheme("https");
    } else {
        let _ = radar_url.set_scheme("http");
    }

    log::info!("Radar session {}", radar_client.session_id);
    log::info!("Available at {}", radar_url);
    log::info!("Press CTRL+C to exit");

    loop {
        tokio::select! {
            result = radar_client.execute() => {
                match result {
                    Ok(_) => break,
                    Err(error) => {
                        log::error!("{:#}", error);
                    }
                }
            },
            _ = signal::ctrl_c() => {
                log::info!("Stopping radar...");
                break;
            }
        }

        let radar_generator = radar_client.take_generator().context("missing generator")?;
        let session_auth_token = radar_client.session_auth_token.clone();

        /* ensure to close the connection in order to reconnect */
        drop(radar_client);

        let mut reconnect_index = 0;
        radar_client = loop {
            log::info!("Reconnecting...");
            match WebRadarPublisher::connect(&url, Some(session_auth_token.clone())).await {
                Ok(publisher) => break publisher,
                Err(error) => {
                    log::error!("Reconnect failed: {:#}", error);
                    if reconnect_index >= RECONNECT_INTERVAL.len() {
                        anyhow::bail!("reconnect failed");
                    }

                    let timeout = RECONNECT_INTERVAL[reconnect_index];
                    reconnect_index += 1;

                    log::error!("Try again in {:#?}", timeout);
                    tokio::time::sleep(timeout).await;
                }
            }
        };
        radar_client.set_generator(radar_generator);
        log::info!(
            "Successfully reconnected (session id = {})",
            radar_client.session_id
        );
    }

    radar_client.close_connection().await;
    Ok(())
}
