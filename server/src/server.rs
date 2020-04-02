use actix::prelude::*;
use rand::{distributions::Alphanumeric, rngs::ThreadRng, prelude::*};
use std::{
    collections::{HashMap, HashSet},
    iter,
};

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub enum Event {
    /// Chat message containing username followed by content
    Message(String, String),
    /// Draw event containing: (x1, y1, x2, y2, penSize)
    Draw(u32, u32, u32, u32, u32),
    /// Start of a new round
    NewRound,
    /// Assign the session a word to draw
    NewDrawer(String),
    /// Join a room. Contains the room code
    JoinRoom(String),
}

#[derive(Default)]
pub struct GameServer {
    rooms: HashMap<String, HashSet<usize>>,
    recipients: HashMap<usize, Recipient<Event>>,
    rng: ThreadRng,
}

impl GameServer {
    pub fn new() -> Self {
        GameServer {
            rooms: HashMap::new(),
            recipients: HashMap::new(),
            rng: ThreadRng::default(),
        }
    }
    fn direct_message(&self, session_id: usize, event: Event) {
        if let Some(recipient) = self.recipients.get(&session_id) {
            if recipient.do_send(event).is_err() {
                println!("Couldn't send message");
            }
        } else {
            println!("Couldn't find recipient");
        }
    }
    fn broadcast_event(&self, room: &str, event: Event) {
        if let Some(recipients) = self.rooms.get(room) {
            for session_id in recipients.iter() {
                self.direct_message(*session_id, event.clone());
            }
        }
    }
    fn create_room(&mut self, session_id: usize) {
        for _ in 0..100 {
            let key: String = iter::repeat(())
                .map(|()| self.rng.sample(Alphanumeric))
                .take(5)
                .collect();
            if self.rooms.get(&key).is_none() {
                let mut occupants = HashSet::new();
                // recipient.do_send(Event::JoinRoom(key.clone())).expect("couldn't send to recipient");
                occupants.insert(session_id);
                self.rooms.insert(key.clone(), occupants);
                self.direct_message(session_id, Event::JoinRoom(key));
                return;
            }
        }
        panic!("Couldn't create room key")
    }
    fn join_room(&mut self, key: &str, session_id: usize) {
        if let Some(room) = self.rooms.get_mut(key) {
            // recipient.do_send(Event::JoinRoom(key.to_string())).expect("couldn't send to recipient");
            room.insert(session_id);
            self.direct_message(session_id, Event::JoinRoom(key.to_string()));
        }
    }
    fn leave_room(&mut self, key: &str, session_id: usize) {
        if let Some(room) = self.rooms.get_mut(key) {
            room.remove(&session_id);
            if room.is_empty() {
                self.rooms.remove(key);
            }
        }
    }
    #[allow(clippy::map_entry)]
    fn connect(&mut self, recipient: Recipient<Event>) -> usize {
        for _ in 0..100 {
            let id: usize = self.rng.gen();
            if !self.recipients.contains_key(&id) && id != 0 {
                self.recipients.insert(id, recipient);
                return id;
            }
        }
        panic!("Couldn't assign id");
    }
    fn disconnect(&mut self, id: usize) {
        self.recipients.remove(&id);
    }
}

impl Actor for GameServer {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub session_id: usize,
    pub content: String,
    pub room: Option<String>,
}

#[derive(Message)]
#[rtype(result = "usize")]
pub struct ConnectMessage {
    pub recipient: Recipient<Event>
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct DisconnectMessage {
    pub session_id: usize,
    pub room: Option<String>,
}

// #[derive(Message)]
// #[rtype(result = "()")]
// pub struct JoinRoom {
//     pub session_id: usize,
//     pub room: String,
//     pub recipient: Recipient<Event>,
// }
//
// #[derive(Message)]
// #[rtype(result = "()")]
// pub struct CreateRoom {
//     pub session_id: usize,
//     pub recipient: Recipient<Event>,
// }
//
// #[derive(Message)]
// #[rtype(result = "()")]
// pub struct LeaveRoom {
//     pub session_id: usize,
//     pub room: String,
// }

impl Handler<ClientMessage> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        let type_char = if let Some(char) = msg.content.chars().next() {
            char
        } else {
            return;
        };
        match (msg.room, type_char) {
            (Some(room), 'c') => {
                let chat: String = msg.content.chars().skip(1).collect();
                self.broadcast_event(
                    &room,
                    Event::Message(format!("user_{}", msg.session_id), chat),
                );
            }
            (Some(room), 'd') => {
                let data: String = msg.content.chars().skip(1).collect();
                if let Ok(content) = data
                    .split(',')
                    .map(|x| x.parse::<u32>())
                    .collect::<Result<Vec<_>, _>>()
                {
                    if let [x1, x2, y1, y2, pen_size] = *content {
                        // TODO: check bounds of numbers
                        self.broadcast_event(
                            &room,
                            Event::Draw(x1, x2, y1, y2, pen_size),
                        );
                    } else {
                        return;
                    }
                }
            },
            (None, 'j') => {
                let key: String = msg.content.chars().skip(1).collect();
                self.join_room(&key, msg.session_id);
            }
            (None, 'n') => {
                self.create_room(msg.session_id);
            }
            (room, c) => {
                println!("Got type_char {}, was in room {:?}", c, room);
            },
        }
    }
}
//
// impl Handler<JoinRoom> for GameServer {
//     type Result = ();
//
//     fn handle(&mut self, msg: JoinRoom, _: &mut Context<Self>) {
//         self.join_room(&msg.room, msg.session_id, msg.recipient);
//     }
// }
//
// impl Handler<CreateRoom> for GameServer {
//     type Result = ();
//
//     fn handle(&mut self, msg: CreateRoom, _: &mut Context<Self>) {
//         self.create_room(msg.session_id, msg.recipient);
//     }
// }

impl Handler<ConnectMessage> for GameServer {
    type Result = usize;

    fn handle(&mut self, msg: ConnectMessage, _: &mut Context<Self>) -> usize {
        self.connect(msg.recipient)
    }
}
impl Handler<DisconnectMessage> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: DisconnectMessage, _: &mut Context<Self>) {
        self.disconnect(msg.session_id);
        if let Some(room) = msg.room {
            self.leave_room(&room, msg.session_id);
        }
    }
}
