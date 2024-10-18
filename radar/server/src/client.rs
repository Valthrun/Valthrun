use std::{
    net::SocketAddr,
    sync::Weak,
};

use anyhow::Context;
use futures::{
    SinkExt,
    StreamExt,
};
use radar_shared::protocol::{
    ClientEvent,
    HandshakeMessage,
    HandshakeProtocolV1,
    HandshakeProtocolV2,
    S2CMessage,
    RADAR_PROTOCOL_VERSION,
};
use tokio::sync::{
    mpsc::{
        self,
        Sender,
    },
    RwLock,
};
use warp::filters::ws::{
    Message,
    WebSocket,
};

use crate::RadarServer;

#[derive(Clone)]
pub enum ClientState {
    Uninitialized,
    Publisher { session_id: String },
    Subscriber { session_id: String },
}

pub struct PubClient {
    pub client_id: u32,
    pub address: SocketAddr,

    pub state: ClientState,

    pub tx: Sender<S2CMessage>,
}

impl PubClient {
    pub fn new(tx: Sender<S2CMessage>, address: SocketAddr) -> Self {
        Self {
            client_id: 0,
            address,

            state: ClientState::Uninitialized,
            tx,
        }
    }

    pub fn send_command(&self, command: S2CMessage) {
        let _ = self.tx.try_send(command);
    }

    async fn process_protocol_handshake(socket: &mut WebSocket) -> anyhow::Result<()> {
        let message = socket.next().await.context("eof on protocol handshake")??;
        let message = serde_json::from_slice::<HandshakeMessage>(message.as_bytes())
            .context("failed to parse handshake")?;

        match message {
            HandshakeMessage::V1(_) => {
                let _ = socket
                    .send(Message::text(serde_json::to_string(
                        &HandshakeProtocolV1::ResponseError {
                            error: format!("Outdated client. Please update."),
                        },
                    )?))
                    .await;

                anyhow::bail!("unsupported v1 client")
            }
            HandshakeMessage::V2(message) => {
                let HandshakeProtocolV2::RequestInitialize { client_version } = message else {
                    log::debug!(
                        "Received client with outdated version ({}). Disconnecting client.",
                        1
                    );
                    let _ = socket
                        .send(Message::text(serde_json::to_string(
                            &HandshakeProtocolV2::ResponseGenericFailure {
                                message: format!("invalid request"),
                            },
                        )?))
                        .await;

                    anyhow::bail!("invalid message")
                };

                if client_version != RADAR_PROTOCOL_VERSION {
                    log::debug!(
                        "Received client with outdated version ({}). Disconnecting client.",
                        client_version
                    );
                    let _ = socket
                        .send(Message::text(serde_json::to_string(
                            &HandshakeProtocolV2::ResponseIncompatible {
                                supported_versions: vec![RADAR_PROTOCOL_VERSION],
                            },
                        )?))
                        .await;

                    anyhow::bail!("client version {} unsupported", client_version)
                }

                let _ = socket
                    .send(Message::text(serde_json::to_string(
                        &HandshakeProtocolV2::ResponseSuccess {
                            server_version: RADAR_PROTOCOL_VERSION,
                        },
                    )?))
                    .await;
            }
        }

        Ok(())
    }

    pub async fn serve_from_websocket(
        server: Weak<RwLock<RadarServer>>,
        client_address: SocketAddr,
        mut socket: WebSocket,
    ) {
        match Self::process_protocol_handshake(&mut socket).await {
            Ok(_) => {}
            Err(err) => {
                log::debug!(
                    "Failed to process client protocol handshake: {}: Closing connection.",
                    err
                );
                let _ = socket.flush().await;
                return;
            }
        }

        let (message_tx, mut message_tx_rx) = mpsc::channel(16);
        let (message_rx_tx, message_rx) = mpsc::channel(16);

        {
            let server = match server.upgrade() {
                Some(server) => server,
                None => {
                    log::warn!(
                        "Accepted ws client from {}, but server gone. Dropping client.",
                        client_address
                    );
                    return;
                }
            };

            let mut server = server.write().await;
            let client_fut = server
                .register_client(
                    PubClient::new(message_tx, client_address.clone()),
                    message_rx,
                )
                .await;

            tokio::spawn(client_fut);
        }

        {
            let (mut tx, mut rx) = socket.split();

            let rx_loop = tokio::spawn({
                let message_rx_tx = message_rx_tx.clone();
                async move {
                    while let Some(message) = rx.next().await {
                        let message = match message {
                            Ok(message) => message,
                            Err(err) => {
                                let _ =
                                    message_rx_tx.send(ClientEvent::RecvError(err.into())).await;
                                break;
                            }
                        };

                        if message.is_text() {
                            let message = match serde_json::from_slice(message.as_bytes()) {
                                Ok(message) => message,
                                Err(err) => {
                                    log::trace!(
                                        "Unparseable message ({}): {}",
                                        err,
                                        String::from_utf8_lossy(message.as_bytes())
                                    );
                                    let _ = message_rx_tx
                                        .send(ClientEvent::RecvError(err.into()))
                                        .await;
                                    break;
                                }
                            };

                            if let Err(err) =
                                { message_rx_tx.send(ClientEvent::RecvMessage(message)).await }
                            {
                                log::warn!("Failed to submit message to queue: {}", err);
                            }
                        }
                    }
                }
            });

            let tx_loop = tokio::spawn({
                let message_rx_tx = message_rx_tx.clone();
                async move {
                    while let Some(message) = message_tx_rx.recv().await {
                        let encoded = match serde_json::to_string(&message) {
                            Ok(message) => message,
                            Err(err) => {
                                let _ =
                                    message_rx_tx.send(ClientEvent::SendError(err.into())).await;
                                break;
                            }
                        };

                        if let Err(err) = tx.send(Message::text(encoded)).await {
                            let _ = message_rx_tx.send(ClientEvent::SendError(err.into())).await;
                            break;
                        }
                    }
                }
            });

            /* await until ether the read or write loop has finished */
            tokio::select! {
                _ = rx_loop => {},
                _ = tx_loop => {},
            }

            let _ = message_rx_tx
                .send(ClientEvent::RecvError(anyhow::anyhow!(
                    "client disconnected"
                )))
                .await;
        }
    }
}
