use actix::prelude::*;
use rand::{distributions::Alphanumeric, prelude::*, rngs::ThreadRng};
use std::{collections::HashMap, iter};

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub enum Event {
    /// Chat message containing username followed by content
    Message(usize, String),
    /// Draw event containing: (x1, y1, x2, y2, penSize)
    Draw(u32, u32, u32, u32, u32),
    /// Start of a new round
    NewRound(usize),
    /// Assign the session a word to draw
    NewLeader(String),
    /// Join a room. Contains the room code and user list
    EnterRoom(String, Vec<(usize, String)>),
    /// Leave a room
    LeaveRoom,
    /// When a user has won. Contains the username and word guessed
    Winner(usize, String),
    /// When another user joins
    UserJoin(usize, String),
    /// When another user leaves
    UserGone(usize)
}

pub struct Room {
    key: String,
    occupants: HashMap<usize, (Recipient<Event>, String)>,
    current_leader: usize,
    word: String,
    rng: ThreadRng,
}
impl Room {
    fn new(key: String, session_id: usize, recipient: Recipient<Event>, username: String) -> Room {
        let mut occupants = HashMap::new();
        occupants.insert(session_id, (recipient.clone(), username.clone()));
        let mut room = Room {
            key: key.clone(),
            occupants,
            current_leader: session_id,
            word: "".to_string(),
            rng: ThreadRng::default(),
        };
        room.direct_message(
            &recipient,
            Event::EnterRoom(key, vec![(session_id, username)]),
        );
        room.new_round();
        room
    }

    fn direct_message(&self, recipient: &Recipient<Event>, event: Event) {
        if recipient.do_send(event).is_err() {
            println!("Couldn't send message");
        }
    }

    fn broadcast_event(&self, event: Event) {
        for (recipient, _) in self.occupants.values() {
            self.direct_message(recipient, event.clone());
        }
    }

    fn choose_new_word(&mut self) {
        let words = vec![
            "cat",
            "banana",
            "liberty",
            "love",
            "people",
            "elephant",
            "house",
            "tomato",
            "spoon",
            "social distancing",
            "laptop",
        ];
        self.word = (*words.as_slice().choose(&mut self.rng).unwrap()).to_string();
    }

    fn join(&mut self, session_id: usize, recipient: Recipient<Event>, username: String) {
        self.broadcast_event(Event::UserJoin(session_id, username.clone()));
        self.occupants.insert(session_id, (recipient.clone(), username));
        self.direct_message(
            &recipient,
            Event::EnterRoom(self.key.to_string(), self.get_user_list()),
        );
        self.direct_message(&recipient, Event::NewRound(self.current_leader));
    }

    fn leave(&mut self, session_id: usize) -> bool {
        if let Some((recipient, _)) = self.occupants.remove(&session_id) {
            self.direct_message(&recipient, Event::LeaveRoom);
            self.broadcast_event(Event::UserGone(session_id));
            if self.occupants.is_empty() {
                return true;
            }
            if self.current_leader == session_id {
                self.new_round();
            }
        }
        false
    }

    fn new_round(&mut self) {
        self.choose_new_word();
        let new_leader = self.occupants.iter().choose(&mut self.rng).unwrap();
        self.current_leader = *new_leader.0;
        for (session_id, (recipient, _)) in self.occupants.iter() {
            if *session_id != self.current_leader {
                self.direct_message(recipient, Event::NewRound(self.current_leader));
            } else {
                self.direct_message(recipient, Event::NewLeader(self.word.clone()));
            }
        }
    }

    fn handle_guess(&mut self, session_id: usize, message: String) {
        self.broadcast_event(Event::Message(session_id, message.clone()));
        if message.trim().to_lowercase() == self.word {
            self.broadcast_event(Event::Winner(session_id, self.word.clone()));
            self.new_round();
        }
    }

    fn handle_draw(&self, session_id: usize, data: String) {
        if self.current_leader != session_id {
            println!("Non-leader tried to send draw command");
            return;
        }

        if let Ok(content) = data
            .split(',')
            .map(|x| x.parse::<u32>())
            .collect::<Result<Vec<_>, _>>()
        {
            if let [x1, x2, y1, y2, pen_size] = *content {
                // TODO: check bounds of numbers
                self.broadcast_event(Event::Draw(x1, x2, y1, y2, pen_size));
            } else {
                println!("invalid draw command");
                return;
            }
        }
    }

    fn get_user_list(&self) -> Vec<(usize, String)> {
        self.occupants
            .iter()
            .map(|(session_id, (_, username))| (*session_id, username.clone()))
            .collect()
    }
}

#[derive(Default)]
pub struct GameServer {
    rooms: HashMap<String, Room>,
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
    fn create_room(&mut self, session_id: usize, username: String) {
        for _ in 0..100 {
            let key: String = iter::repeat(())
                .map(|()| self.rng.sample(Alphanumeric))
                .take(5)
                .collect();
            if self.rooms.get(&key).is_none() {
                let recipient = self
                    .recipients
                    .get(&session_id)
                    .expect("session_id did not exist");
                let room = Room::new(key.clone(), session_id, recipient.clone(), username);

                self.rooms.insert(key, room);
                return;
            }
        }
        panic!("Couldn't create room key")
    }
    fn join_room(&mut self, key: &str, username: String, session_id: usize) {
        if let Some(room) = self.rooms.get_mut(key) {
            let recipient = self
                .recipients
                .get(&session_id)
                .expect("session_id did not exist");
            room.join(session_id, recipient.clone(), username);
        }
    }
    fn leave_room(&mut self, key: &str, session_id: usize) {
        if let Some(room) = self.rooms.get_mut(key) {
            // If room after the session leaving is now empty, delete it
            if room.leave(session_id) {
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
    pub recipient: Recipient<Event>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct DisconnectMessage {
    pub session_id: usize,
    pub room: Option<String>,
}

impl Handler<ClientMessage> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        let type_char = if let Some(char) = msg.content.chars().next() {
            char
        } else {
            return;
        };
        match (msg.room, type_char) {
            (Some(room_key), 'm') => {
                let chat: String = msg.content.chars().skip(1).collect();
                if let Some(room) = self.rooms.get_mut(&room_key) {
                    room.handle_guess(msg.session_id, chat);
                }
            }
            (Some(room_key), 'd') => {
                let data: String = msg.content.chars().skip(1).collect();
                if let Some(room) = self.rooms.get(&room_key) {
                    room.handle_draw(msg.session_id, data);
                }
            }
            (Some(room_key), 'q') => {
                self.leave_room(&room_key, msg.session_id);
            }
            (None, 'j') => {
                let data = msg.content.chars().skip(1).collect::<String>();
                let components = data.split(',').collect::<Vec<_>>();
                if let [key, username] = *components {
                    if validate_username(username) {
                        self.join_room(&key, username.to_string(), msg.session_id);
                    }
                }
            }
            (None, 'n') => {
                let username: String = msg.content.chars().skip(1).collect();
                if validate_username(&username) {
                    self.create_room(msg.session_id, username);
                }
            }
            (room, c) => {
                println!("Got type_char {}, was in room {:?}", c, room);
            }
        }
    }
}

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

fn validate_username(username: &str) -> bool {
    !username.contains(',') && username.len() < 15
}
