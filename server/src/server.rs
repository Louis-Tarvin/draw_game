use actix::prelude::*;
use rand::{distributions::Alphanumeric, prelude::*, rngs::ThreadRng};
use std::collections::HashMap;
use std::sync::Arc;

use crate::word_pack::{load_word_packs, WordPack};
use crate::Room;

use log::{info, trace, warn};

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub enum Event {
    /// Chat message containing username followed by content
    Message(usize, String),
    /// Draw event containing: (x1, y1, x2, y2, penSize)
    Draw(u32, u32, u32, u32, u32),
    /// Clears the canvas
    ClearCanvas,
    /// Start of a new round
    NewRound(usize),
    /// Assign the session a word to draw
    NewLeader(bool, String),
    /// Join a room. Contains the room code and user list
    EnterRoom(String, Vec<(usize, String)>),
    /// Error that indicates that a username already exists within a room
    UsernameExists(String),
    /// Error that indicates that a room key doesn't exist
    NonExistantRoom(String),
    /// Leave a room
    LeaveRoom,
    /// When a user has won. Contains the username, points, word guessed, and alternate
    Winner(Option<usize>, usize, String, Option<String>),
    /// When another user joins
    UserJoin(usize, String),
    /// When another user leaves
    UserGone(usize),
    /// Join a lobby. Contains the id of the host
    EnterLobby(usize),
    // Settings suplementary data for client. Wordpack id followed by name and description
    SettingsData(Vec<(usize, String, String)>),
}

pub struct GameServer {
    rooms: HashMap<String, Room>,
    recipients: HashMap<usize, Recipient<Event>>,
    rng: ThreadRng,
    word_packs: Arc<Vec<WordPack>>,
}

impl GameServer {
    pub fn new<P: std::fmt::Debug + AsRef<std::path::Path>>(word_pack_dir: P) -> Self {
        let word_packs = load_word_packs(word_pack_dir).expect("Error loading the word packs");

        info!(
            "Game server instance created with {} word packs",
            word_packs.len()
        );

        GameServer {
            rooms: HashMap::new(),
            recipients: HashMap::new(),
            rng: ThreadRng::default(),
            word_packs: Arc::new(word_packs),
        }
    }

    fn create_room(&mut self, session_id: usize, username: String) {
        loop {
            let key: String = std::iter::repeat(())
                .map(|()| self.rng.sample(Alphanumeric))
                .take(5)
                .collect();
            if self.rooms.get(&key).is_none() {
                if let Some(recipient) = self.recipients.get(&session_id) {
                    let room = Room::new(
                        key.clone(),
                        Arc::clone(&self.word_packs),
                        session_id,
                        recipient.clone(),
                        username.clone(),
                    );

                    self.rooms.insert(key.clone(), room);

                    trace!(
                        "Room {} was created by user {} ({}), there are now {} rooms",
                        key,
                        username,
                        session_id,
                        self.rooms.len(),
                    );
                } else {
                    warn!("User creating a room didn't exist");
                }
                return;
            } else {
                trace!("Tried to create room with key {} but it was taken", key);
            }
        }
    }

    fn join_room(&mut self, key: &str, username: String, session_id: usize) {
        let recipient = self
            .recipients
            .get(&session_id)
            .expect("session_id did not exist");
        if let Some(room) = self.rooms.get_mut(key) {
            room.join(session_id, recipient.clone(), username);
        } else {
            // Perfectly normal user behaviour (e.g. enter wrong key by accident)
            let _ = recipient.do_send(Event::NonExistantRoom(key.to_string()));
            trace!(
                "User {} ({}) tried to join non-existant room {}",
                username, session_id, key
            );
        }
    }

    fn leave_room(&mut self, key: &str, session_id: usize, ctx: &mut Context<GameServer>) {
        if let Some(room) = self.rooms.get_mut(key) {
            // If room after the session leaving is now empty, delete it
            if room.leave(session_id, ctx) {
                self.rooms.remove(key);
                trace!(
                    "Room {} is empty so removing it, {} room(s) left",
                    key,
                    self.rooms.len(),
                );
            }
        } else {
            warn!(
                "User {} tried to leave non-existant room {}",
                session_id, key
            );
        }
    }

    fn start_room(&mut self, key: &str, session_id: usize, lines: Vec<String>, ctx: &mut Context<GameServer>) {
        if let Some(room) = self.rooms.get_mut(key) {
            room.start(session_id, lines, ctx);
        } else {
            warn!(
                "User {} tried to start non-existant room {}",
                session_id, key
            );
        }
    }

    fn handle_clear(&mut self, key: &str, session_id: usize) {
        if let Some(room) = self.rooms.get_mut(key) {
            room.clear(session_id);
        } else {
            warn!(
                "User {} tried to clear non-existant room {}",
                session_id, key
            );
        }
    }

    pub fn round_timeout(&mut self, key: &str, round_id: usize, ctx: &mut Context<GameServer>) {
        if let Some(room) = self.rooms.get_mut(key) {
            room.round_timeout(round_id, ctx);
        } else {
            trace!("Round timeout on non-existant room {}", key);
        }
    }

    #[allow(clippy::map_entry)]
    fn connect(&mut self, recipient: Recipient<Event>) -> usize {
        loop {
            let id: usize = self.rng.gen();
            if !self.recipients.contains_key(&id) && id != 0 {
                self.recipients.insert(id, recipient);
                info!(
                    "Recipient given id {}, there are now {} user(s) connected",
                    id,
                    self.recipients.len()
                );
                return id;
            }
        }
    }

    fn disconnect(&mut self, id: usize) {
        self.recipients.remove(&id);
        trace!(
            "Id {} disconnected, {} user(s) left",
            id,
            self.recipients.len()
        );
    }

    pub fn new_round(&mut self, room_key: String, ctx: &mut Context<GameServer>) {
        if let Some(room) = self.rooms.get_mut(&room_key) {
            room.new_round(ctx);
        }
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

    fn handle(&mut self, msg: ClientMessage, ctx: &mut Context<Self>) {
        let type_char = if let Some(char) = msg.content.chars().next() {
            char
        } else {
            warn!("User {} sent empty message (no type_char)", msg.session_id,);
            return;
        };
        match (msg.room, type_char) {
            (Some(room_key), 'm') => {
                let chat: String = msg.content.chars().skip(1).collect();

                if chat.is_empty() {
                    warn!(
                        "User {} tried to send empty message in room {}",
                        msg.session_id, room_key
                    );
                    return;
                }

                if let Some(room) = self.rooms.get_mut(&room_key) {
                    room.handle_guess(msg.session_id, chat, ctx);
                } else {
                    warn!(
                        "User {} was marked as being in non-existant room {} when sending message",
                        msg.session_id, room_key
                    );
                }
            }
            (Some(room_key), 'd') => {
                let data: String = msg.content.chars().skip(1).collect();
                if let Some(room) = self.rooms.get_mut(&room_key) {
                    room.handle_draw(msg.session_id, data);
                } else {
                    warn!(
                        "User {} was marked as being in non-existant room {} when sending draw command",
                        msg.session_id,
                        room_key
                    );
                }
            }
            (Some(room_key), 'q') => {
                self.leave_room(&room_key, msg.session_id, ctx);
            }
            (Some(room_key), 's') => {
                let lines: Vec<String> = msg.content.lines().skip(1).map(|x| x.to_string()).collect();
                self.start_room(&room_key, msg.session_id, lines, ctx);
            }
            (Some(room_key), 'c') => {
                self.handle_clear(&room_key, msg.session_id);
            }
            (None, 'j') => {
                let data = msg.content.chars().skip(1).collect::<String>();
                let components = data.split(',').collect::<Vec<_>>();
                if let [key, username] = *components {
                    if validate_username(username) {
                        self.join_room(&key, username.to_string(), msg.session_id);
                    } else {
                        warn!(
                            "{} sent invalid username {} when joining room {}",
                            msg.session_id, username, key
                        );
                    }
                } else {
                    warn!(
                        "{} tried to join room without the correct number of components (expected 2 got {})",
                        msg.session_id,
                        components.len(),
                    );
                }
            }
            (None, 'n') => {
                let username: String = msg.content.chars().skip(1).collect();
                if validate_username(&username) {
                    self.create_room(msg.session_id, username);
                } else {
                    warn!(
                        "{} sent invalid username {} when creating room",
                        msg.session_id, username
                    );
                }
            }
            (room, c) => {
                warn!(
                    "Invalid message: got type_char {}, was in room {:?}",
                    c, room
                );
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

    fn handle(&mut self, msg: DisconnectMessage, ctx: &mut Context<Self>) {
        self.disconnect(msg.session_id);
        if let Some(room) = msg.room {
            self.leave_room(&room, msg.session_id, ctx);
        }
    }
}

fn validate_username(username: &str) -> bool {
    !username.contains(',') && username.len() < 15
}
