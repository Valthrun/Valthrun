use std::net::SocketAddr;

use radar_shared::protocol::S2CMessage;
use tokio::sync::mpsc::Sender;

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
}
