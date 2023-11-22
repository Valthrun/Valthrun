use anyhow::Context;
use clap::Parser;
use cs2::{
    offsets_runtime,
    CS2Handle,
};
use futures_util::{
    SinkExt,
    StreamExt,
};
use radar_client::{
    CS2RadarGenerator,
    WebRadarPublisher,
};
use radar_shared::protocol::{
    C2SMessage,
    ClientEvent,
    S2CMessage,
};
use tokio::sync::mpsc::{
    self,
    Receiver,
    Sender,
};
use tokio_tungstenite::tungstenite::Message;

/// Standalone Valthrun CS2 radar
#[derive(Parser, Debug)]
#[command(long_about = None)]
struct Args {
    /// Target server address used to publish the web radar
    #[arg(short, long, default_value = "ws://127.0.0.1:7229/publish")]
    publish_url: String,
}

async fn create_connection(
    url: url::Url,
) -> anyhow::Result<(Sender<C2SMessage>, Receiver<ClientEvent<S2CMessage>>)> {
    let (socket, _) = tokio_tungstenite::connect_async(url).await?;
    let (mut socket_tx, mut socket_rx) = socket.split();

    let (channel_rx_tx, channel_rx) = mpsc::channel(16);
    let (channel_tx, mut channel_tx_rx) = mpsc::channel(16);
    tokio::spawn({
        let channel_rx_tx = channel_rx_tx.clone();
        async move {
            while let Some(message) = channel_tx_rx.recv().await {
                let message = match serde_json::to_string(&message) {
                    Ok(message) => message,
                    Err(err) => {
                        let _ = channel_rx_tx.send(ClientEvent::SendError(err.into())).await;
                        break;
                    }
                };

                if let Err(err) = socket_tx.send(Message::Text(message)).await {
                    let _ = channel_rx_tx.send(ClientEvent::SendError(err.into())).await;
                    break;
                }
            }
        }
    });

    tokio::spawn({
        let channel_rx_tx = channel_rx_tx.clone();
        async move {
            while let Some(message) = socket_rx.next().await {
                let message = match message {
                    Ok(message) => message,
                    Err(err) => {
                        let _ = channel_rx_tx.send(ClientEvent::RecvError(err.into())).await;
                        break;
                    }
                };

                match message {
                    Message::Text(message) => {
                        let message = match serde_json::from_slice(message.as_bytes()) {
                            Ok(message) => message,
                            Err(err) => {
                                let _ =
                                    channel_rx_tx.send(ClientEvent::RecvError(err.into())).await;
                                break;
                            }
                        };

                        if let Err(err) =
                            { channel_rx_tx.send(ClientEvent::RecvMessage(message)).await }
                        {
                            log::warn!("Failed to submit message to queue: {}", err);
                        }
                    }
                    _ => {}
                }
            }
        }
    });

    Ok((channel_tx, channel_rx))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let url = url::Url::parse(&args.publish_url).context("invalid target server address")?;

    let cs2 = CS2Handle::create(true)?;
    offsets_runtime::setup_provider(&cs2)?;
    let radar_generator = Box::new(CS2RadarGenerator::new(cs2.clone())?);

    let (tx, rx) = create_connection(url).await?;
    let radar_client = WebRadarPublisher::create_from_transport(radar_generator, tx, rx).await?;

    log::info!("Radar session id: {}", radar_client.session_id);
    if let Some(err) = radar_client.await {
        log::error!("Radar error: {:#}", err);
    }
    Ok(())
}
