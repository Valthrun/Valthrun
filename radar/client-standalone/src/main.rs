use anyhow::Context;
use clap::Parser;
use cs2::{
    offsets_runtime,
    CS2Handle,
    InterfaceError,
    StateCS2Handle,
    StateCS2Memory,
};
use radar_client::{
    CS2RadarGenerator,
    WebRadarPublisher,
};
use url::Url;
use utils_state::StateRegistry;

/// Standalone Valthrun CS2 radar
#[derive(Parser, Debug)]
#[command(long_about = None)]
struct Args {
    /// Target server address used to publish the web radar.
    /// Use ws://127.0.0.1:7229/publish for local development.
    #[arg(short, long, default_value = "wss://radar.valth.run/publish")]
    publish_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let url = Url::parse(&args.publish_url).context("invalid target server address")?;

    let radar_generator = {
        let cs2 = match CS2Handle::create(true) {
            Ok(cs2) => cs2,
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
        let mut states = StateRegistry::new(1024 * 8);
        states.set(StateCS2Memory::new(cs2.create_memory_view()), ())?;
        states.set(StateCS2Handle::new(cs2), ())?;

        offsets_runtime::setup_provider(&states)?;
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

    if let Some(err) = radar_client.await {
        log::error!("Radar error: {:#}", err);
    }
    Ok(())
}
