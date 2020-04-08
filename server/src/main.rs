use actix::prelude::*;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

pub mod server;
pub mod session;

use clap::{crate_authors, crate_version, load_yaml};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let yaml = load_yaml!("cli.yml");
    let matches = clap::App::from(yaml)
        .version(crate_version!())
        .author(crate_authors!())
        .get_matches();

    let port = matches.value_of("port")
        .map(|x| x.parse::<u16>().expect("Port must be an integer"))
        .unwrap_or(3007);

    let serve_dir = matches.value_of("serve_dir").map(|x| x.to_string());

    let game_server = server::GameServer::new().start();

    let serve_dir_msg = if let Some(dir) = &serve_dir {
        format!("serving {} at '/'", dir)
    } else {
        "".to_string()
    };

    println!("Starting server at 0.0.0.0:{} {}", port, serve_dir_msg);

    HttpServer::new(move || {
        let app = App::new()
            .data(game_server.clone())
            .service(web::resource("/ws/").to(socket_route));
        if let Some(dir) = &serve_dir {
            app.service(actix_files::Files::new("/", dir).index_file("index.html"))
        } else {
            app
        }
    })
    .bind(("0.0.0.0", port))?
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
