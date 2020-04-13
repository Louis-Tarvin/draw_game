use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use actix::prelude::*;
use log::{error, trace, warn};
use rand::prelude::*;

use crate::{Event, WordPack};

struct LobbyState {
    pub host: usize,
}

struct RoundState {
    pub word: usize,
    pub leader: usize,
}

struct WinnerState {
    pub word: usize,
    pub winner: usize,
    pub alternate: Option<usize>,
}

enum RoomState {
    Lobby(LobbyState),
    Round(RoundState),
    Winner(WinnerState),
}

pub struct Room {
    state: RoomState,
    key: String,
    occupants: HashMap<usize, (Recipient<Event>, String)>,
    word_pack: Arc<WordPack>,
    rng: ThreadRng,
    queue: VecDeque<usize>,
    excluded_words: VecDeque<usize>,
    max_excluded_words: usize,
    draw_history: Vec<(u32, u32, u32, u32, u32)>,
}
impl Room {
    pub fn new(
        key: String,
        word_pack: Arc<WordPack>,
        session_id: usize,
        recipient: Recipient<Event>,
        username: String,
    ) -> Room {
        let mut occupants = HashMap::new();
        occupants.insert(session_id, (recipient.clone(), username.clone()));
        let mut queue = VecDeque::new();
        queue.push_back(session_id);
        let room = Room {
            state: RoomState::Lobby(LobbyState { host: session_id }),
            key: key.clone(),
            occupants,
            max_excluded_words: word_pack.list_len() / 2,
            word_pack,
            rng: ThreadRng::default(),
            queue,
            excluded_words: VecDeque::new(),
            draw_history: Vec::new(),
        };
        room.direct_message(
            &recipient,
            Event::EnterRoom(key, vec![(session_id, username)]),
        );
        room.direct_message(
            &recipient,
            Event::EnterLobby(session_id),
        );
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

    fn choose_new_word(&mut self) -> usize {
        loop {
            let word_index = self.rng.gen_range(0, self.word_pack.list_len());
            if !self.excluded_words.contains(&word_index) {
                if self.excluded_words.len() >= self.max_excluded_words {
                    self.excluded_words.pop_front();
                }
                self.excluded_words.push_back(word_index);
                return word_index;
            }
        }
    }

    pub fn start(&mut self, session_id: usize) {
        if let RoomState::Lobby(LobbyState { host }) = self.state {
            if session_id == host {
                self.new_round();
            } else {
                warn!(
                    "user {} tried to start game when they weren't host in room {}",
                    session_id, self.key
                );
            }
        } else {
            warn!(
                "user {} tried to start game when the state was not in lobby in room {}",
                session_id, self.key
            );
        }
    }

    pub fn join(&mut self, session_id: usize, recipient: Recipient<Event>, username: String) {
        if self.occupants.get(&session_id).is_some() {
            warn!(
                "User {} ({}) is already in room {}",
                username, session_id, self.key
            );
            return;
        }

        trace!("{} ({}) joining room {}", username, session_id, self.key);
        self.broadcast_event(Event::UserJoin(session_id, username.clone()));
        self.occupants
            .insert(session_id, (recipient.clone(), username));
        self.direct_message(
            &recipient,
            Event::EnterRoom(self.key.to_string(), self.get_user_list()),
        );
        match self.state {
            RoomState::Lobby(LobbyState { host }) => {
                self.direct_message(&recipient, Event::EnterLobby(host));
            }
            RoomState::Round(RoundState { leader, .. }) => {
                self.direct_message(&recipient, Event::NewRound(leader));
                self.send_draw_history(session_id, &recipient);
            }
            RoomState::Winner(WinnerState { word, winner, .. }) => {
                self.direct_message(
                    &recipient,
                    Event::Winner(winner, self.word_pack.get_word(word).clone()),
                );
                self.send_draw_history(session_id, &recipient);
            }
        }
        self.queue.push_back(session_id);
    }

    fn send_draw_history(&self, session_id: usize, recipient: &Recipient<Event>) {
        trace!(
            "Sending draw history of {} commands to {}",
            self.draw_history.len(),
            session_id
        );
        for (x1, x2, y1, y2, pen_size) in &self.draw_history {
            self.direct_message(recipient, Event::Draw(*x1, *x2, *y1, *y2, *pen_size));
        }
    }

    pub fn leave(&mut self, session_id: usize) -> bool {
        trace!("{} leaving room {}", session_id, self.key);
        if let Some((recipient, _)) = self.occupants.remove(&session_id) {
            self.direct_message(&recipient, Event::LeaveRoom);
            self.broadcast_event(Event::UserGone(session_id));
            if self.occupants.is_empty() {
                return true;
            }
            if let RoomState::Round(RoundState {
                leader: session_id, ..
            }) = self.state
            {
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

    fn end_round(&mut self, winner: usize, alternate: Option<usize>) {
        if let RoomState::Round(RoundState { word, leader }) = self.state {
            self.state = RoomState::Winner(WinnerState {
                word,
                winner,
                alternate,
            });
            self.queue.push_back(leader);
        } else {
            error!("end_round called with invalid state in room {}", self.key);
        }
    }

    pub fn new_round(&mut self) {
        let word = self.choose_new_word();

        self.draw_history.clear();
        while let Some(new_leader) = self.queue.pop_front() {
            if self.occupants.get(&new_leader).is_some() {
                self.state = RoomState::Round(RoundState {
                    word,
                    leader: new_leader,
                });

                for (session_id, (recipient, _)) in self.occupants.iter() {
                    if *session_id != new_leader {
                        self.direct_message(recipient, Event::NewRound(new_leader));
                    } else {
                        self.direct_message(
                            recipient,
                            Event::NewLeader(self.word_pack.get_word(word).clone()),
                        );
                    }
                }

                trace!(
                    "Room {} has new round with word {}, leader {}",
                    self.key,
                    word,
                    new_leader,
                );
                return;
            }
        }
        error!("Room {} had no possible leader for new round", self.key);
    }

    pub fn handle_guess(&mut self, session_id: usize, message: String) -> bool {
        if let RoomState::Round(RoundState { word, leader }) = self.state {
            if session_id != leader {
                self.broadcast_event(Event::Message(session_id, message.clone()));
                if self
                    .word_pack
                    .word_matches(word, &message.trim().to_lowercase())
                {
                    self.broadcast_event(Event::Winner(
                        session_id,
                        self.word_pack.get_word(word).clone(),
                    ));
                    //TODO: sort out alternate
                    self.end_round(session_id, None);
                    return true;
                }
            } else {
                warn!(
                    "Leader {} in room {} tried to send guess {}",
                    leader, self.key, message
                );
            }
        } else {
            self.broadcast_event(Event::Message(session_id, message));
        }
        false
    }

    pub fn handle_draw(&mut self, session_id: usize, data: String) {
        if let RoomState::Round(RoundState { leader, .. }) = self.state {
            if leader != session_id {
                warn!(
                    "Uid {} in room {} tried to send draw command when {} was leader",
                    session_id, self.key, leader
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
                    self.draw_history.push((x1, x2, y1, y2, pen_size));
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
        } else {
            warn!(
                "draw command sent by {} in invalid state in room {}",
                session_id, self.key
            );
        }
    }

    fn get_user_list(&self) -> Vec<(usize, String)> {
        self.occupants
            .iter()
            .map(|(session_id, (_, username))| (*session_id, username.clone()))
            .collect()
    }
}
