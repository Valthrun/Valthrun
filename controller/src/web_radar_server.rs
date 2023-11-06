use std::{
    string::ToString,
    sync::{
        Arc,
        Mutex,
        RwLock,
    },
};

use actix::{
    prelude::*,
    Actor,
    StreamHandler,
};
use actix_web::{
    middleware::Logger,
    web,
    App,
    Error,
    HttpRequest,
    HttpResponse,
    HttpServer,
};
use actix_web_actors::ws;
use map::MapInfo;

use crate::map;

/// Define HTTP actor
pub struct WebRadar {}

pub static CURRENT_MAP: once_cell::sync::Lazy<Arc<RwLock<MapInfo>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(MapInfo::new("<Empty>".to_string()))));

impl Actor for WebRadar {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let address = ctx.address();
        if let Ok(mut clients) = CLIENTS.lock() {
            let current_map = CURRENT_MAP.read().unwrap();
            match serde_json::to_string(&current_map.clone()) {
                Ok(data) => {
                    address.do_send(MessageData { data: data.clone() });
                }
                Err(e) => {
                    log::error!("Failed to create json with error: {}", e);
                }
            };
            clients.push(address);
            log::info!("Client connected!");
        }
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        if let Ok(mut clients) = CLIENTS.lock() {
            clients.retain(|addr| *addr != ctx.address());
            log::info!("Client disconnected!");
        }
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebRadar {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct MessageData {
    pub data: String,
}

pub static CLIENTS: once_cell::sync::Lazy<Arc<Mutex<Vec<Addr<WebRadar>>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

impl Handler<MessageData> for WebRadar {
    type Result = ();

    fn handle(&mut self, msg: MessageData, ctx: &mut Self::Context) {
        // Send the data to the WebSocket client
        ctx.text(msg.data);
    }
}

async fn ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(WebRadar {}, &req, stream);
    println!("{:?}", resp);
    resp
}

pub async fn run_server() -> Result<(), anyhow::Error> {
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/ws", web::get().to(ws))
            .service(actix_files::Files::new("/", "./web_radar_server").index_file("index.html"))
    })
    .bind("0.0.0.0:6969")?
    .run()
    .await?;

    Ok(())
}
