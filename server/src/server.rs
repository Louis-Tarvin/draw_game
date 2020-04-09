use actix::prelude::*;
use rand::{distributions::Alphanumeric, prelude::*, rngs::ThreadRng};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use log::{debug, info, trace, warn};

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
    UserGone(usize),
}

pub struct Room {
    key: String,
    occupants: HashMap<usize, (Recipient<Event>, String)>,
    current_leader: usize,
    word_list: Arc<Vec<String>>,
    word: usize,
    rng: ThreadRng,
    queue: VecDeque<usize>,
    excluded_words: VecDeque<usize>,
    max_excluded_words: usize,
}
impl Room {
    fn new(
        key: String,
        word_list: Arc<Vec<String>>,
        session_id: usize,
        recipient: Recipient<Event>,
        username: String,
    ) -> Room {
        let mut occupants = HashMap::new();
        occupants.insert(session_id, (recipient.clone(), username.clone()));
        let mut queue = VecDeque::new();
        queue.push_back(session_id);
        let mut room = Room {
            key: key.clone(),
            occupants,
            current_leader: 0,
            max_excluded_words: word_list.len() / 10,
            word_list,
            word: 0,
            rng: ThreadRng::default(),
            queue,
            excluded_words: VecDeque::new(),
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
            // TODO: try to fix sending leave message when socket disconnects
            // so that this is rare, hence upgrading from
            // trace (common behaviour) to warn (uncommon - indicates bug)
            trace!("Tried to send message to disconnected socket");
        }
    }

    fn broadcast_event(&self, event: Event) {
        for (recipient, _) in self.occupants.values() {
            self.direct_message(recipient, event.clone());
        }
    }

    fn choose_new_word(&mut self) {
        loop {
            let word_index = self.rng.gen_range(0, self.word_list.len());
            if !self.excluded_words.contains(&word_index) {
                self.excluded_words.push_back(self.word);
                self.word = word_index;
                if self.excluded_words.len() > self.max_excluded_words {
                    self.excluded_words.pop_front();
                }
                break;
            }
        }
    }

    fn join(&mut self, session_id: usize, recipient: Recipient<Event>, username: String) {
        trace!("{} ({}) joining room {}", username, session_id, self.key);
        self.broadcast_event(Event::UserJoin(session_id, username.clone()));
        self.occupants
            .insert(session_id, (recipient.clone(), username));
        self.direct_message(
            &recipient,
            Event::EnterRoom(self.key.to_string(), self.get_user_list()),
        );
        self.direct_message(&recipient, Event::NewRound(self.current_leader));
        self.queue.push_back(session_id);
    }

    fn leave(&mut self, session_id: usize) -> bool {
        trace!("{} leaving room {}", session_id, self.key);
        if let Some((recipient, _)) = self.occupants.remove(&session_id) {
            self.direct_message(&recipient, Event::LeaveRoom);
            self.broadcast_event(Event::UserGone(session_id));
            if self.occupants.is_empty() {
                return true;
            }
            if self.current_leader == session_id {
                trace!(
                    "Current leader ({}) left room so new round in room {}",
                    session_id,
                    self.key
                );
                self.new_round();
            }
        } else {
            warn!(
                "User {} tried to leave room {} when it wasn't a member",
                session_id, self.key
            );
        }

        false
    }

    fn new_round(&mut self) {
        self.choose_new_word();
        while let Some(new_leader) = self.queue.pop_front() {
            if self.occupants.get(&new_leader).is_some() {
                self.queue.push_back(self.current_leader);
                self.current_leader = new_leader;
                break;
            }
        }
        for (session_id, (recipient, _)) in self.occupants.iter() {
            if *session_id != self.current_leader {
                self.direct_message(recipient, Event::NewRound(self.current_leader));
            } else {
                self.direct_message(
                    recipient,
                    Event::NewLeader(self.word_list[self.word].clone()),
                );
            }
        }

        trace!(
            "Room {} has new round with word {}, leader {}",
            self.key,
            self.word,
            self.current_leader
        );
    }

    fn handle_guess(&mut self, session_id: usize, message: String) {
        if session_id != self.current_leader {
            self.broadcast_event(Event::Message(session_id, message.clone()));
            if message.trim().to_lowercase() == self.word_list[self.word] {
                self.broadcast_event(Event::Winner(session_id, self.word_list[self.word].clone()));
                self.new_round();
            }
        } else {
            warn!(
                "Leader {} in room {} tried to send guess {}",
                self.current_leader, self.key, message
            );
        }
    }

    fn handle_draw(&self, session_id: usize, data: String) {
        if self.current_leader != session_id {
            warn!(
                "Uid {} in room {} tried to send draw command when {} was leader",
                session_id, self.key, self.current_leader
            );
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
                warn!(
                    "{} in room {} sent a draw command with not enough parts (expected 5 got {})",
                    session_id,
                    self.key,
                    content.len()
                );
            }
        } else {
            warn!(
                "{} in room {} sent draw command that couldn't be parsed into a list of u32s",
                session_id, self.key
            )
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
    word_list: Arc<Vec<String>>,
}

impl GameServer {
    pub fn new() -> Self {
        let word_list = include_str!("words.txt");
        let word_list: Vec<_> = word_list
            .split('\n')
            .map(|word| word.trim().to_string())
            .filter(|word| !word.is_empty())
            .collect();

        info!(
            "Game server instance created with {} words",
            word_list.len()
        );

        GameServer {
            rooms: HashMap::new(),
            recipients: HashMap::new(),
            rng: ThreadRng::default(),
            word_list: Arc::new(word_list),
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
                        Arc::clone(&self.word_list),
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
        if let Some(room) = self.rooms.get_mut(key) {
            let recipient = self
                .recipients
                .get(&session_id)
                .expect("session_id did not exist");
            room.join(session_id, recipient.clone(), username);
        } else {
            // Perfectly normal user behaviour (e.g. enter wrong key by accident)
            debug!(
                "User {} ({}) tried to join non-existant room {}",
                username, session_id, key
            );
        }
    }
    fn leave_room(&mut self, key: &str, session_id: usize) {
        if let Some(room) = self.rooms.get_mut(key) {
            // If room after the session leaving is now empty, delete it
            if room.leave(session_id) {
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
            warn!("User {} sent empty message (no type_char)", msg.session_id,);
            return;
        };
        match (msg.room, type_char) {
            (Some(room_key), 'm') => {
                let chat: String = msg.content.chars().skip(1).collect();
                if let Some(room) = self.rooms.get_mut(&room_key) {
                    room.handle_guess(msg.session_id, chat);
                } else {
                    warn!(
                        "User {} was marked as being in non-existant room {} when sending message",
                        msg.session_id, room_key
                    );
                }
            }
            (Some(room_key), 'd') => {
                let data: String = msg.content.chars().skip(1).collect();
                if let Some(room) = self.rooms.get(&room_key) {
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
                self.leave_room(&room_key, msg.session_id);
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
