use actix::prelude::*;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

use flexi_logger::{opt_format, Cleanup, Criterion, Duplicate, Logger, Naming};
use log::info;

pub mod room;
pub mod server;
pub mod session;
pub mod word_pack;

pub use room::Room;
pub use server::{ClientMessage, Event, GameServer};

use clap::{crate_authors, crate_version, load_yaml};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let yaml = load_yaml!("cli.yml");
    let matches = clap::App::from(yaml)
        .version(crate_version!())
        .author(crate_authors!())
        .get_matches();

    if let Some(log_path) = matches.value_of("log") {
        Logger::with_env_or_str("server=trace")
            .directory(log_path)
            .log_to_file()
            .duplicate_to_stderr(Duplicate::Info)
            .rotate(
                Criterion::Size(500_000),
                Naming::Timestamps,
                Cleanup::KeepLogAndZipFiles(5, 100),
            )
            .format(opt_format)
            .start()
            .expect("Couldn't start logger");
    } else {
        env_logger::init();
    }

    let port = matches
        .value_of("port")
        .map(|x| x.parse::<u16>().expect("Port must be an integer"))
        .unwrap_or(3007);

    let serve_dir = matches.value_of("serve_dir").map(|x| x.to_string());

    let word_pack = matches.value_of("word_pack").unwrap_or("wordpacks");

    let game_server = server::GameServer::new(word_pack).start();

    let serve_dir_msg = if let Some(dir) = &serve_dir {
        format!("serving {} at '/'", dir)
    } else {
        "".to_string()
    };

    info!("Starting server at 0.0.0.0:{} {}", port, serve_dir_msg);

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
    game_server: web::Data<Addr<GameServer>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        session::Session {
            id: 0,
            game_server: game_server.get_ref().clone(),
            room: None,
        },
        &req,
        stream,
    )
}
