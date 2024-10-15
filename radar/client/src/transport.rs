use anyhow::Context;
use futures_util::{
    SinkExt,
    StreamExt,
};
use radar_shared::protocol::{
    C2SMessage,
    ClientEvent,
    HandshakeProtocolV2,
    S2CMessage,
    RADAR_PROTOCOL_VERSION,
};
use tokio::{
    net::TcpStream,
    sync::mpsc::{
        self,
        Receiver,
        Sender,
    },
};
use tokio_tungstenite::{
    tungstenite::{
        Error,
        Message,
    },
    MaybeTlsStream,
    WebSocketStream,
};

async fn create_ws_socket(
    url: &url::Url,
) -> anyhow::Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
    let (mut socket, _response) =
        tokio_tungstenite::connect_async(url)
            .await
            .map_err(|err| -> anyhow::Error {
                let Error::Http(http) = &err else {
                    return err.into();
                };
                let Some(body) = http.body() else {
                    return err.into();
                };

                let body = String::from_utf8_lossy(body);
                if body.len() > 200 {
                    log::error!(
                        "Failed to connect to radar server. Http status code {}, body: {}",
                        http.status().as_str(),
                        body
                    );
                }

                anyhow::anyhow!(
                    "http status {} ({})",
                    http.status().as_str(),
                    if body.len() > 200 {
                        &body[0..200]
                    } else {
                        &body[..]
                    }
                )
            })?;

    socket
        .send(Message::Text(serde_json::to_string(
            &HandshakeProtocolV2::RequestInitialize {
                client_version: RADAR_PROTOCOL_VERSION,
            },
        )?))
        .await?;

    let response = socket.next().await.context("eof while handshaking")??;
    let response = serde_json::from_slice::<HandshakeProtocolV2>(&response.into_data())?;
    match response {
        HandshakeProtocolV2::ResponseSuccess { server_version } => {
            log::debug!("Server protocol version: {}", server_version);
            return Ok(socket);
        }
        HandshakeProtocolV2::ResponseGenericFailure { message } => {
            anyhow::bail!("generic handshake failure: {}", message);
        }
        HandshakeProtocolV2::ResponseIncompatible { supported_versions } => {
            anyhow::bail!(
                "server unsupported protocol (supported protocols: {:?})",
                supported_versions
            )
        }
        _ => anyhow::bail!("invalid server handshake response"),
    }
}

pub async fn create_ws_transport(
    url: &url::Url,
) -> anyhow::Result<(Sender<C2SMessage>, Receiver<ClientEvent<S2CMessage>>)> {
    let socket = create_ws_socket(&url).await?;
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
