use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use actix::prelude::*;
use rand::prelude::*;
use log::{trace, warn};

use crate::{WordPack, Event};

pub struct Room {
    key: String,
    occupants: HashMap<usize, (Recipient<Event>, String)>,
    current_leader: usize,
    word_pack: Arc<WordPack>,
    word: usize,
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
        let mut room = Room {
            key: key.clone(),
            occupants,
            current_leader: 0,
            max_excluded_words: word_pack.list_len() / 2,
            word_pack,
            word: 0,
            rng: ThreadRng::default(),
            queue,
            excluded_words: VecDeque::new(),
            draw_history: Vec::new(),
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
            let word_index = self.rng.gen_range(0, self.word_pack.list_len());
            if !self.excluded_words.contains(&word_index) {
                if self.excluded_words.len() >= self.max_excluded_words {
                    self.excluded_words.pop_front();
                }
                self.excluded_words.push_back(word_index);
                self.word = word_index;
                break;
            }
        }
    }

    pub fn join(&mut self, session_id: usize, recipient: Recipient<Event>, username: String) {
        if self.occupants.get(&session_id).is_some() {
            warn!("User {} ({}) is already in room {}", username, session_id, self.key);
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
        self.direct_message(&recipient, Event::NewRound(self.current_leader));
        self.send_draw_history(session_id, &recipient);
        self.queue.push_back(session_id);
    }

    fn send_draw_history(&self, session_id: usize, recipient: &Recipient<Event>) {
        trace!("Sending draw history of {} commands to {}", self.draw_history.len(), session_id);
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
        self.draw_history.clear();
        while let Some(new_leader) = self.queue.pop_front() {
            if self.occupants.get(&new_leader).is_some() {
                if !self.queue.contains(&self.current_leader) {
                    self.queue.push_back(self.current_leader);
                } else {
                    warn!("detected bug tried to queue user {} who was already in the user queue in room {}", self.current_leader, self.key);
                }
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
                    Event::NewLeader(self.word_pack.get_word(self.word).clone()),
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

    pub fn handle_guess(&mut self, session_id: usize, message: String) {
        if session_id != self.current_leader {
            self.broadcast_event(Event::Message(session_id, message.clone()));
            if self.word_pack.word_matches(self.word, &message.trim().to_lowercase()) {
                self.broadcast_event(Event::Winner(session_id, self.word_pack.get_word(self.word).clone()));
                self.new_round();
            }
        } else {
            warn!(
                "Leader {} in room {} tried to send guess {}",
                self.current_leader, self.key, message
            );
        }
    }

    pub fn handle_draw(&mut self, session_id: usize, data: String) {
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
    }

    fn get_user_list(&self) -> Vec<(usize, String)> {
        self.occupants
            .iter()
            .map(|(session_id, (_, username))| (*session_id, username.clone()))
            .collect()
    }
}
