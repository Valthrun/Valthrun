use std::sync::{
    Arc,
    Mutex,
    Weak,
};

use cs2::{
    CS2Handle,
    CS2HandleState,
};
use radar_client::{
    CS2RadarGenerator,
    WebRadarPublisher,
};
use tokio::{
    sync::oneshot,
    task::{
        self,
    },
};
use url::Url;
use utils_state::StateRegistry;

pub enum WebRadarState {
    Connecting,
    Connected { session_id: String },
    Disconnected { message: String },
}

pub struct WebRadar {
    ref_self: Weak<Mutex<WebRadar>>,

    endpoint: Url,
    connection_state: WebRadarState,

    disconnect_tx: Option<oneshot::Sender<()>>,
}

impl WebRadar {
    async fn create_connection(
        endpoint: &Url,
        cs2: Arc<CS2Handle>,
    ) -> anyhow::Result<WebRadarPublisher> {
        let radar_generator = {
            let mut states = StateRegistry::new(1024 * 8);
            states.set(CS2HandleState::new(cs2), ())?;

            Box::new(CS2RadarGenerator::new(states)?)
        };

        WebRadarPublisher::connect(radar_generator, endpoint).await
    }

    pub fn endpoint(&self) -> &Url {
        &self.endpoint
    }

    pub fn connection_state(&self) -> &WebRadarState {
        &self.connection_state
    }

    pub fn close_connection(&mut self) {
        if let Some(abort) = self.disconnect_tx.take() {
            let _ = abort.send(());
        }
    }
}

pub fn create_web_radar(endpoint: Url, cs2: Arc<CS2Handle>) -> Arc<Mutex<WebRadar>> {
    let (disconnect_tx, disconnect_rx) = oneshot::channel();
    let instance = Arc::new_cyclic(|ref_self| {
        Mutex::new(WebRadar {
            ref_self: ref_self.clone(),

            connection_state: WebRadarState::Connecting,
            endpoint: endpoint.clone(),

            disconnect_tx: Some(disconnect_tx),
        })
    });

    task::spawn({
        let instance = instance.clone();

        async move {
            let mut publisher = match WebRadar::create_connection(&endpoint, cs2).await {
                Ok(publisher) => {
                    log::info!("Web radar created. Session id: {}", publisher.session_id);
                    let mut instance = instance.lock().unwrap();
                    instance.connection_state = WebRadarState::Connected {
                        session_id: publisher.session_id.clone(),
                    };
                    publisher
                }
                Err(err) => {
                    log::error!("Failed to create web radar session: {:?}", err);
                    let mut instance = instance.lock().unwrap();
                    instance.connection_state = WebRadarState::Disconnected {
                        message: format!("{:#}", err),
                    };
                    return;
                }
            };

            tokio::select! {
                result = &mut publisher => {
                    match result {
                        None => {
                            log::error!("Web radar connection closed");

                            let mut instance = instance.lock().unwrap();
                            instance.connection_state = WebRadarState::Disconnected {
                                message: format!("connection closed"),
                            };
                        }
                        Some(error) => {
                            log::error!("Web radar exited: {:#}", error);

                            let mut instance = instance.lock().unwrap();
                            instance.connection_state = WebRadarState::Disconnected {
                                message: format!("connection error: {:?}", error),
                            };
                        }
                    }
                },
                _ = disconnect_rx => {
                    log::info!("Web radar closed");

                    let mut instance = instance.lock().unwrap();
                    instance.connection_state = WebRadarState::Disconnected {
                        message: format!("locally closed"),
                    };
                }
            }

            publisher.close_connection().await;
            log::trace!("Publisher connection closed");
        }
    });

    instance
}
