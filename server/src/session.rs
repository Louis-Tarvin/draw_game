use actix::prelude::*;
use actix_web_actors::ws;

use crate::server::*;

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
                    Ok(res) => act.id = res,
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
            println!("recieved message, but id was 0");
            return;
        }
        let message = match event {
            Event::Message(username, msg) => format!("c{},{}", username, msg),
            Event::JoinRoom(room_name) => {
                self.room = Some(room_name.clone());
                format!("j{}", room_name)
            }
            Event::Draw(x1, y1, x2, y2, pen_size) => {
                format!("d{},{},{},{},{}", x1, y1, x2, y2, pen_size)
            }
            Event::NewRound => "r".to_string(),
            Event::NewDrawer(word) => format!("n{}", word),
        };
        ctx.text(message);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Session {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
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
                self.game_server.do_send(ClientMessage {
                    session_id: self.id,
                    content: text,
                    room: self.room.clone(),
                });
            }
            ws::Message::Close(_) => {
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Binary(_) => {
                ctx.stop();
            }
            ws::Message::Nop => {}
        }
    }
}
