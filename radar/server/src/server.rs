use std::{
    collections::BTreeMap,
    net::SocketAddr,
    path::PathBuf,
    sync::{
        Arc,
        Weak,
    },
    time::{
        Duration,
        Instant,
    },
};

use futures_util::Future;
use radar_shared::protocol::{
    C2SMessage,
    ClientEvent,
    S2CMessage,
};
use rand::{
    distributions::Alphanumeric,
    Rng,
};
use tokio::{
    sync::{
        mpsc::{
            self,
            Receiver,
        },
        RwLock,
    },
    task::JoinHandle,
    time::{
        self,
    },
};
use warp::{
    self,
    Filter,
};

use crate::{
    client::PubClient,
    handler::ServerCommandHandler,
    ClientId,
    ClientState,
};

pub enum PubSessionOwner {
    Owned { client_id: u32 },
    Unbound { timestamp: Instant },
}

pub struct PubSession {
    pub owner: PubSessionOwner,

    pub session_id: String,
    pub session_auth_token: String,

    subscriber: BTreeMap<u32, mpsc::Sender<S2CMessage>>,
}

impl PubSession {
    pub fn broadcast(&self, message: &S2CMessage) {
        for subscriber in self.subscriber.values() {
            let _ = subscriber.try_send(message.clone());
        }
    }

    pub fn subscriber_count(&self) -> usize {
        self.subscriber.len()
    }
}

pub enum HttpServeDirectory {
    /// Do not serve any static HTTP files
    None,

    /// Serve static HTTP files at a specific path
    Disk { path: PathBuf },

    /// Bundle all static HTTP files with the server executable
    Bundled,
}

impl HttpServeDirectory {}

pub struct RadarServer {
    ref_self: Weak<RwLock<RadarServer>>,
    client_id_counter: u32,

    clients: BTreeMap<u32, Arc<RwLock<PubClient>>>,
    pub_sessions: BTreeMap<String, PubSession>,

    www_acceptor: Option<JoinHandle<()>>,
}

impl RadarServer {
    pub fn new() -> Arc<RwLock<Self>> {
        let mut result = Self {
            ref_self: Default::default(),
            client_id_counter: 1,

            clients: Default::default(),
            pub_sessions: Default::default(),

            www_acceptor: None,
        };

        Arc::new_cyclic(|weak| {
            result.ref_self = weak.clone();
            tokio::spawn(Self::tick_task(weak.clone()));

            RwLock::new(result)
        })
    }

    async fn tick_task(this: Weak<RwLock<Self>>) {
        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            let Some(this) = this.upgrade() else {
                return;
            };

            let mut this = this.write().await;
            this.tick().await;
        }
    }

    async fn tick(&mut self) {
        let expired_sessions = self
            .pub_sessions
            .values()
            .filter(|session| {
                if let PubSessionOwner::Unbound { timestamp } = &session.owner {
                    timestamp.elapsed() > Duration::from_secs(120)
                } else {
                    false
                }
            })
            .map(|session| session.session_id.clone())
            .collect::<Vec<_>>();

        for session_id in expired_sessions {
            log::info!("Session {} expired. Closing session.", &session_id);
            self.pub_session_close(&session_id).await;
        }
    }

    pub async fn listen_http(
        &mut self,
        addr: impl Into<SocketAddr>,
        static_serve: HttpServeDirectory,
    ) -> anyhow::Result<()> {
        if self.www_acceptor.is_some() {
            anyhow::bail!("www already started");
        }

        let server = self.ref_self.clone();
        let ws_route = warp::any()
            .and(warp::path("subscribe").or(warp::path("publish")))
            .and(warp::addr::remote())
            .and(warp::ws())
            .map(move |_, address: Option<SocketAddr>, ws: warp::ws::Ws| {
                let server = server.clone();
                ws.on_upgrade(move |socket| async move {
                    let Some(address) = address else { return };
                    PubClient::serve_from_websocket(server, address, socket).await;
                })
            })
            .boxed();

        let routes: warp::filters::BoxedFilter<(Box<dyn warp::Reply>,)> = match static_serve {
            HttpServeDirectory::Disk { path } => ws_route
                .or(warp::fs::dir(path.clone()))
                .or(warp::fs::file(path.join("index.html")))
                .map(|reply| -> Box<dyn warp::Reply> { Box::new(reply) })
                .boxed(),
            HttpServeDirectory::Bundled => {
                anyhow::bail!("bundled is currently not supported");
            }
            HttpServeDirectory::None => ws_route
                .map(|reply| -> Box<dyn warp::Reply> { Box::new(reply) })
                .boxed(),
        };

        let (address, future) = warp::serve(routes).try_bind_ephemeral(addr)?;
        self.www_acceptor = Some(tokio::spawn(future));

        log::info!("Started server on {}", address);

        Ok(())
    }

    pub async fn unregister_client(&mut self, client_id: u32, clean_disconnect: bool) {
        let client = match self.clients.remove(&client_id) {
            Some(client) => client,
            None => return,
        };

        let client_state = {
            let client = client.read().await;
            client.state.clone()
        };
        match client_state {
            ClientState::Publisher { session_id } => {
                if clean_disconnect {
                    self.pub_session_close(&session_id).await;
                } else {
                    self.pub_session_unbound(&session_id).await;
                }
            }
            ClientState::Subscriber { session_id } => {
                self.pub_session_unsubscribe(&session_id, client_id).await;
            }
            ClientState::Uninitialized => { /* Nothing to do! */ }
        };

        log::debug!("Disconnected pub client {}", client_id);
    }

    pub async fn register_client(
        &mut self,
        mut client: PubClient,
        mut rx: Receiver<ClientEvent<C2SMessage>>,
    ) -> impl Future<Output = ()> {
        let client_id = self.client_id_counter.wrapping_add(1);
        self.client_id_counter = client_id;

        log::debug!(
            "Registered new pub client from {} with client id {}",
            client.address,
            client_id
        );

        client.client_id = client_id;
        let client = Arc::new(RwLock::new(client));
        self.clients.insert(client_id, client.clone());

        let command_handler = ServerCommandHandler {
            server: self.ref_self.upgrade().expect("to be present"),
            client: client.clone(),
            client_id,
        };

        async move {
            let clean_disconnect = loop {
                let Some(event) = rx.recv().await else {
                    break false;
                };

                match event {
                    ClientEvent::RecvMessage(command) => {
                        if let C2SMessage::Disconnect { reason: message } = &command {
                            /* client requested a disconnect */
                            log::debug!("Client send disconnect with reason: {}", message);
                            break true;
                        }

                        let result = command_handler.handle_command(command).await;
                        client.read().await.send_command(result);
                    }
                    ClientEvent::RecvError(err) => {
                        log::debug!("Client {} recv error: {}", command_handler.client_id, err);
                        break false;
                    }
                    ClientEvent::SendError(err) => {
                        log::debug!("Client {} send error: {}", command_handler.client_id, err);
                        break false;
                    }
                }
            };

            command_handler
                .server
                .write()
                .await
                .unregister_client(command_handler.client_id, clean_disconnect)
                .await;
        }
    }

    pub async fn pub_session_create(&mut self, owner_id: ClientId) -> Option<&PubSession> {
        let owner = match self.clients.get(&owner_id) {
            Some(client) => client,
            None => return None,
        };

        let mut owner = owner.write().await;
        if !matches!(owner.state, ClientState::Uninitialized) {
            return None;
        }

        let session_id = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .map(char::from)
            .take(6)
            .collect::<String>();

        let session_auth_token = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .map(char::from)
            .take(12)
            .collect::<String>();

        self.pub_sessions.insert(
            session_id.clone(),
            PubSession {
                owner: PubSessionOwner::Owned {
                    client_id: owner_id,
                },

                session_id: session_id.clone(),
                session_auth_token: session_auth_token.clone(),

                subscriber: Default::default(),
            },
        );

        log::info!("Created new session {}", session_id);
        owner.state = ClientState::Publisher {
            session_id: session_id.clone(),
        };
        self.pub_sessions.get(&session_id)
    }

    pub async fn pub_session_reclaim(
        &mut self,
        client_id: ClientId,
        session_auth_token: &str,
    ) -> Option<&PubSession> {
        let owner = match self.clients.get(&client_id) {
            Some(client) => client,
            None => return None,
        };

        let mut owner = owner.write().await;
        if !matches!(owner.state, ClientState::Uninitialized) {
            return None;
        }

        let session = self
            .pub_sessions
            .values_mut()
            .find(|entry| entry.session_auth_token == session_auth_token)?;

        if !matches!(&session.owner, PubSessionOwner::Unbound { .. }) {
            /* session is owned by a client */
            return None;
        }

        session.owner = PubSessionOwner::Owned { client_id };

        log::info!("Reclaimed session {} by {}", session.session_id, client_id);
        owner.state = ClientState::Publisher {
            session_id: session.session_id.clone(),
        };
        Some(session)
    }

    pub async fn pub_session_unbound(&mut self, session_id: &str) {
        let session = match self.pub_sessions.get_mut(session_id) {
            Some(session) => session,
            None => return,
        };

        if !matches!(&session.owner, PubSessionOwner::Owned { .. }) {
            return;
        }

        log::info!("Unbound session {}", session_id);
        session.owner = PubSessionOwner::Unbound {
            timestamp: Instant::now(),
        };
    }

    pub async fn pub_session_close(&mut self, session_id: &str) {
        let session = match self.pub_sessions.remove(session_id) {
            Some(session) => session,
            None => return,
        };

        log::info!("Session {} closed", session_id);
        session.broadcast(&S2CMessage::NotifySessionClosed {});

        for client_id in session.subscriber.keys() {
            let client = match self.clients.get(client_id) {
                Some(client) => client,
                None => continue,
            };

            let mut client = client.write().await;
            client.state = ClientState::Uninitialized;
        }
    }

    pub fn pub_session_find(&self, session_id: &str) -> Option<&PubSession> {
        self.pub_sessions.get(session_id)
    }

    pub async fn pub_session_unsubscribe(&mut self, session_id: &String, client_id: u32) {
        if let Some(session) = self.pub_sessions.get_mut(session_id) {
            session.subscriber.remove(&client_id);
            session.broadcast(&S2CMessage::NotifyViewCount {
                viewers: session.subscriber_count(),
            });
        }

        if let Some(client) = self.clients.get(&client_id) {
            let mut client = client.write().await;
            if let ClientState::Subscriber {
                session_id: client_session_id,
            } = &client.state
            {
                if client_session_id == session_id {
                    client.state = ClientState::Uninitialized;
                } else {
                    log::warn!(
                        "Client state indicates different session id then we unregister the client"
                    );
                }
            }
        }
    }

    pub async fn pub_session_subscribe(
        &mut self,
        session_id: &String,
        client_id: u32,
    ) -> PubSessionSubscribeResult {
        let client = match self.clients.get(&client_id) {
            Some(client) => client,
            None => return PubSessionSubscribeResult::InvalidClientId,
        };

        let mut client = client.write().await;
        if !matches!(client.state, ClientState::Uninitialized) {
            return PubSessionSubscribeResult::InvalidClientState;
        }

        let session = match self.pub_sessions.get_mut(session_id) {
            Some(session) => session,
            None => return PubSessionSubscribeResult::InvalidSessionId,
        };

        session
            .subscriber
            .insert(client.client_id, client.tx.clone());

        session.broadcast(&S2CMessage::NotifyViewCount {
            viewers: session.subscriber.len(),
        });

        client.state = ClientState::Subscriber {
            session_id: session.session_id.clone(),
        };
        PubSessionSubscribeResult::Success
    }
}

pub enum PubSessionSubscribeResult {
    Success,
    InvalidClientState,
    InvalidSessionId,
    InvalidClientId,
}
