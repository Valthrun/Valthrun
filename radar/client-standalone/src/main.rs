use std::net::ToSocketAddrs;

use anyhow::Context;
use clap::Parser;
use cs2::{
    offsets_runtime,
    CS2Handle,
};
use radar_client::{
    CS2RadarGenerator,
    WebRadarPublisher,
};
use tokio::net::TcpStream;

/// Standalone Valthrun CS2 radar
#[derive(Parser, Debug)]
#[command(long_about = None)]
struct Args {
    /// Target server address used to publish the web radar
    #[arg(short, long)]
    server_address: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .parse_default_env()
        .init();

    let server_address = args
        .server_address
        .to_socket_addrs()?
        .next()
        .context("invalid target server address")?;

    let cs2 = CS2Handle::create(true)?;
    offsets_runtime::setup_provider(&cs2)?;

    let radar_generator = Box::new(CS2RadarGenerator::new(cs2.clone())?);

    let connection = TcpStream::connect(server_address).await?;
    let radar_client =
        WebRadarPublisher::create_from_transport(radar_generator, connection).await?;

    log::info!("Radar session id: {}", radar_client.session_id);
    if let Some(err) = radar_client.await {
        log::error!("Radar error: {:#}", err);
    }
    Ok(())
}
