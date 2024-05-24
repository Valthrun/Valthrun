use std::sync::Arc;

use radar_shared::protocol::{
    C2SMessage,
    S2CMessage,
};
use tokio::sync::RwLock;

use crate::{
    ClientState,
    PubClient,
    PubSessionSubscribeResult,
    RadarServer,
};

pub struct ServerCommandHandler {
    pub server: Arc<RwLock<RadarServer>>,
    pub client: Arc<RwLock<PubClient>>,
    pub client_id: u32,
}

impl ServerCommandHandler {
    pub async fn handle_command(&self, command: C2SMessage) -> S2CMessage {
        match command {
            C2SMessage::InitializePublish { .. } => {
                let mut server = self.server.write().await;
                let Some(session) = server.pub_session_create(self.client_id).await else {
                    return S2CMessage::ResponseInvalidClientState;
                };

                S2CMessage::ResponseInitializePublish {
                    session_id: session.session_id.clone(),
                    version: 1,
                }
            }
            C2SMessage::InitializeSubscribe { session_id, .. } => {
                let mut server = self.server.write().await;
                match server
                    .pub_session_subscribe(&session_id, self.client_id)
                    .await
                {
                    PubSessionSubscribeResult::Success => S2CMessage::ResponseSubscribeSuccess,
                    PubSessionSubscribeResult::InvalidClientId => {
                        S2CMessage::ResponseInvalidClientState
                    }
                    PubSessionSubscribeResult::InvalidClientState => {
                        S2CMessage::ResponseInvalidClientState
                    }
                    PubSessionSubscribeResult::InvalidSessionId => {
                        S2CMessage::ResponseSessionInvalidId
                    }
                }
            }
            C2SMessage::RadarUpdate { update } => {
                let server = self.server.read().await;
                let client = self.client.read().await;

                let session_id = match &client.state {
                    ClientState::Publisher { session_id } => session_id,
                    _ => return S2CMessage::ResponseInvalidClientState,
                };

                let session = match server.pub_session_find(session_id) {
                    Some(session) => session,
                    None => return S2CMessage::ResponseSessionInvalidId,
                };

                if session.owner_id != client.client_id {
                    return S2CMessage::ResponseError {
                        error: "you're not allowed to send updates".to_string(),
                    };
                }

                session.broadcast(&S2CMessage::NotifyRadarUpdate {
                    update: update.clone(),
                });

                S2CMessage::ResponseSuccess
            }
            C2SMessage::Disconnect { .. } => {
                /* command is already handled within the connection code */
                S2CMessage::ResponseSuccess
            }
        }
    }
}
