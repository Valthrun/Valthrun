use std::{
    pin::Pin,
    time::Duration,
};

use anyhow::Context;
use radar_shared::protocol::{
    C2SMessage,
    ClientEvent,
    S2CMessage,
};
use tokio::{
    self,
    sync::mpsc::{
        Receiver,
        Sender,
    },
    time::{
        self,
        Interval,
    },
};
use url::Url;

use crate::{
    create_ws_transport,
    RadarGenerator,
};

pub struct WebRadarPublisher {
    pub session_id: String,
    pub session_auth_token: String,

    generator: Option<Box<dyn RadarGenerator>>,
    generate_interval: Pin<Box<Interval>>,

    transport_tx: Sender<C2SMessage>,
    transport_rx: Receiver<ClientEvent<S2CMessage>>,
}

impl WebRadarPublisher {
    pub async fn connect(url: &Url, session_auth_token: Option<String>) -> anyhow::Result<Self> {
        let (tx, rx) = create_ws_transport(url).await?;
        Self::create_from_transport(session_auth_token, tx, rx).await
    }

    pub async fn create_from_transport(
        session_auth_token: Option<String>,
        tx: Sender<C2SMessage>,
        mut rx: Receiver<ClientEvent<S2CMessage>>,
    ) -> anyhow::Result<Self> {
        let _ = tx
            .send(C2SMessage::InitializePublish { session_auth_token })
            .await;

        let event = tokio::select! {
            message = rx.recv() => message.context("unexpected client disconnect")?,
            _ = time::sleep(Duration::from_secs(5)) => {
                anyhow::bail!("session init timeout");
            }
        };

        let (session_id, session_auth_token) = match event {
            ClientEvent::RecvMessage(message) => match message {
                S2CMessage::ResponseError { error } => {
                    anyhow::bail!("server error: {}", error)
                }
                S2CMessage::ResponseInitializePublish {
                    session_id,
                    session_auth_token,
                } => (session_id, session_auth_token),
                S2CMessage::ResponseSessionInvalidId {} => {
                    anyhow::bail!("session does not exists")
                }
                response => anyhow::bail!("invalid response: {:?}", response),
            },
            ClientEvent::RecvError(err) => anyhow::bail!("recv err: {:#}", err),
            ClientEvent::SendError(err) => anyhow::bail!("send err: {:#}", err),
        };

        log::debug!("Connected with session id {}", session_id);
        Ok(Self {
            session_id,
            session_auth_token,

            generator: None,
            generate_interval: Box::pin(time::interval(Duration::from_millis(50))),

            transport_rx: rx,
            transport_tx: tx,
        })
    }

    pub fn set_generator(&mut self, generator: Box<dyn RadarGenerator>) {
        self.generator = Some(generator);
    }

    pub fn take_generator(&mut self) -> Option<Box<dyn RadarGenerator>> {
        self.generator.take()
    }

    fn send_message(&self, message: C2SMessage) {
        let _ = self.transport_tx.try_send(message);
    }

    pub async fn close_connection(self) {
        let _ = self
            .transport_tx
            .send_timeout(
                C2SMessage::Disconnect {
                    reason: "connection close".to_string(),
                },
                Duration::from_secs(1),
            )
            .await;
    }

    pub async fn execute(&mut self) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                event = self.transport_rx.recv() => {
                    let event = event.context("transport closed unexpectetly")?;
                    self.handle_event(event)?;
                },
                _ = self.generate_interval.tick() => {
                    self.send_radar_state();
                }
            }
        }
    }

    fn send_radar_state(&mut self) {
        let Some(generator) = &mut self.generator else {
            return;
        };

        match generator.generate_state() {
            Ok(state) => self.send_message(C2SMessage::NotifyRadarState { state }),
            Err(err) => {
                log::warn!("Failed to generate radar state: {:#}", err);
            }
        }
    }

    fn handle_event(&mut self, event: ClientEvent<S2CMessage>) -> anyhow::Result<()> {
        match event {
            ClientEvent::RecvError(err) => {
                log::debug!("Recv error: {}", err);
                Err(err)
            }
            ClientEvent::SendError(err) => {
                log::debug!("Send error: {}", err);
                Err(err)
            }
            ClientEvent::RecvMessage(_message) => {
                /* TODO? */
                Ok(())
            }
        }
    }
}
