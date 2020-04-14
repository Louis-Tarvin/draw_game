use actix::prelude::*;
use actix_web_actors::ws;

use crate::server::*;

use log::{error, warn};

pub struct Session {
    pub id: usize,
    pub game_server: Addr<GameServer>,
    pub room: Option<String>,
}

impl Actor for Session {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let session_addr = ctx.address();

        self.game_server
            .send(ConnectMessage {
                recipient: session_addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(session_id) => {
                        act.id = session_id;
                        ctx.text(format!("c{}", session_id));
                    }
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.game_server.do_send(DisconnectMessage {
            session_id: self.id,
            room: self.room.clone(),
        });
        Running::Stop
    }
}

impl Handler<Event> for Session {
    type Result = ();

    fn handle(&mut self, event: Event, ctx: &mut Self::Context) {
        if self.id == 0 {
            error!("server wants to send event but id was 0 (uninitialised), this is an internal error");
            return;
        }
        let message = match event {
            Event::Message(username, msg) => format!("m{},{}", username, msg),
            Event::EnterRoom(room_name, users) => {
                self.room = Some(room_name.clone());
                let mut output = String::with_capacity(1024);
                output.push_str(&format!("e{}", room_name));
                for (session_id, username) in users {
                    output.push_str(&format!(",{},{}", session_id, username));
                }
                output
            }
            Event::LeaveRoom => {
                self.room = None;
                "q".to_string()
            }
            Event::Draw(x1, y1, x2, y2, pen_size) => {
                format!("d{},{},{},{},{}", x1, y1, x2, y2, pen_size)
            }
            Event::NewRound(username) => format!("r{}", username),
            Event::NewLeader(word) => format!("l{}", word),
            Event::Winner(session_id, word) => format!("w{},{}", session_id, word),
            Event::UserJoin(session_id, username) => format!("j{},{}", session_id, username),
            Event::UserGone(session_id) => format!("g{}", session_id),
            Event::EnterLobby(host_id) => format!("o{}", host_id),
            Event::SettingsData(wordpacks) => {
                let mut string = "s".to_string();
                for (id, name, description) in wordpacks {
                    string.push_str(&format!("\n{},{},{}", id, name, description));
                }
                string
            }
        };
        ctx.text(message);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Session {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(err) => {
                error!("Websocket result (uid={}) was an error {:?}", self.id, err);
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        match msg {
            ws::Message::Ping(msg) => {
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {}
            ws::Message::Text(text) => {
                if self.id != 0 {
                    self.game_server.do_send(ClientMessage {
                        session_id: self.id,
                        content: text,
                        room: self.room.clone(),
                    });
                } else {
                    warn!("Client sent message when it's id was 0");
                }
            }
            ws::Message::Close(_) => {
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                warn!("Client tried to send contintuation message");
            }
            ws::Message::Binary(_) => {
                warn!("Client tried to send binary message");
            }
            ws::Message::Nop => {}
        }
    }
}
