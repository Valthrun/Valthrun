use std::sync::{
    Arc,
    Mutex,
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

/// Define HTTP actor
pub struct WebRadar {}

impl Actor for WebRadar {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let address = ctx.address();
        // Now you can use my_address to add it to the global list or do whatever you need.
        if let Ok(mut clients) = CLIENTS.lock() {
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
