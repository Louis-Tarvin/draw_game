use actix::prelude::*;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

pub mod server;
pub mod session;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let game_server = server::GameServer::new().start();

    HttpServer::new(move || {
        App::new()
            .data(game_server.clone())
            .service(web::resource("/ws/").to(socket_route))
    })
    .bind("127.0.0.1:3001")?
    .run()
    .await
}

async fn socket_route(
    req: HttpRequest,
    stream: web::Payload,
    game_server: web::Data<Addr<server::GameServer>>
) -> Result<HttpResponse, Error> {
    ws::start(session::Session {
        id: 0,
        game_server: game_server.get_ref().clone(),
        room: None,
    }, &req, stream)
}
