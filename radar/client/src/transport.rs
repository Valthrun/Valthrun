use futures_util::{
    SinkExt,
    StreamExt,
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

pub async fn create_ws_connection(
    url: &url::Url,
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
            loop {
                let message = tokio::select! {
                    _ = channel_rx_tx.closed() => {
                        /* channel locally closed */
                        break;
                    },
                    message = socket_rx.next() => message
                };

                let message = match message {
                    Some(Ok(message)) => message,
                    Some(Err(err)) => {
                        /* recv error */
                        let _ = channel_rx_tx.send(ClientEvent::RecvError(err.into())).await;
                        break;
                    }
                    None => {
                        /* channel remotely closed */
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
