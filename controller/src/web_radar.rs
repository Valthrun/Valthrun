use actix::{Actor, StreamHandler, prelude::*};
use actix_web::{middleware::Logger, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use std::sync::{Arc, Mutex};

pub struct RadarAddress {
    pub radar_addr: Option<Addr<WebRadar>>,
}

impl RadarAddress {
    pub fn new() -> Self {
        RadarAddress { radar_addr: None }
    }
}

/// Define HTTP actor
pub struct WebRadar {
    address: Arc<Mutex<RadarAddress>>,
}

impl WebRadar {
    pub fn new(address: Arc<Mutex<RadarAddress>>) -> Self {
        WebRadar { address: address }
    }
}

impl Actor for WebRadar {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let mut address = self.address.lock().unwrap();
        address.radar_addr = Some(ctx.address());
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
pub struct PlayersData {
    pub data: String,
}

pub static CLIENTS: once_cell::sync::Lazy<Arc<Mutex<Vec<Addr<WebRadar>>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

impl Handler<PlayersData> for WebRadar {
    type Result = ();

    fn handle(&mut self, msg: PlayersData, ctx: &mut Self::Context) {
        // Send the data to the WebSocket client
        ctx.text(msg.data);
    }
}

async fn ws(req: HttpRequest, stream: web::Payload, radar_address: web::Data<Arc<Mutex<RadarAddress>>>) -> Result<HttpResponse, Error> {
    let resp = ws::start(WebRadar::new(radar_address.get_ref().clone()), &req, stream);
    println!("{:?}", resp);
    resp
}

pub async fn run_server(radar_address: Arc<Mutex<RadarAddress>>) -> Result<(), anyhow::Error> {
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(radar_address.clone()))
            .route("/ws", web::get().to(ws))
            .service(actix_files::Files::new("/", "./").index_file("index.html"))
    })
        .bind("0.0.0.0:6969")?
        .run()
        .await?;

    Ok(())
}