use std::{
    net::ToSocketAddrs,
    path::PathBuf,
};

use anyhow::Context;
use clap::Parser;
use radar_server::{
    HttpServeDirectory,
    RadarServer,
};
use tokio::signal;

/// Standalone Valthrun CS2 radar
#[derive(Parser, Debug)]
#[command(long_about = None)]
struct Args {
    /// Server address for the publisher clients to connect to (tcp/ip)
    #[arg(short, long, default_value = "0.0.0.0:7228")]
    pub_address: String,

    /// Server address for the web radar subscribers to connect to (http/tcp/ip)
    #[arg(short, long, default_value = "0.0.0.0:7229")]
    sub_address: String,

    /// Static HTML file directory (optional)
    #[arg(long)]
    static_dir: Option<PathBuf>,
}

// $env:RUST_LOG="trace,tungstenite=info,tokio_tungstenite=info,tokio_util=info"
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .parse_default_env()
        .init();

    let args = Args::parse();

    let server = RadarServer::new();
    {
        let mut server = server.write().await;

        server
            .listen_client(
                args.pub_address
                    .to_socket_addrs()?
                    .next()
                    .context("invalid bind address")?,
            )
            .await?;

        server
            .listen_http(
                args.sub_address
                    .to_socket_addrs()?
                    .next()
                    .context("invalid bind address")?,
                if let Some(path) = args.static_dir.as_ref() {
                    HttpServeDirectory::Disk { path: path.clone() }
                } else {
                    HttpServeDirectory::None
                },
            )
            .await?;
    }

    let _ = signal::ctrl_c().await;
    Ok(())
}
