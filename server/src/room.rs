use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;

use actix::prelude::*;
use log::{error, trace, warn};
use rand::prelude::*;

use crate::{server::GameServer, word_pack::WordPack, Event};

struct LobbyState {
    pub host: usize,
}

struct RoundState {
    pub word: (usize, usize),
    pub leader: usize,
}

struct WinnerState {
    pub word: (usize, usize),
    pub winner: Option<usize>,
    pub alternate: Option<usize>,
}

#[derive(Default, Debug)]
struct Settings {
    pub round_timer: bool,
    pub allow_clear: bool,
    pub enabled_word_packs: Vec<usize>,
}
impl Settings {
    fn parse_from_lines(lines: Vec<String>, max_wordpack_id: usize) -> Option<Settings> {
        if let [wordpacks, time_limit, canvas_clearing] = &*lines {
            let wordpacks = wordpacks
                .split(',')
                .map(|x| {
                    if let Ok(id) = x.parse() {
                        if id < max_wordpack_id {
                            return Ok(id);
                        }
                    }
                    Err(())
                })
                .collect::<Result<Vec<_>, _>>()
                .ok()?;
            return Some(Settings {
                enabled_word_packs: wordpacks,
                round_timer: time_limit == "T",
                allow_clear: canvas_clearing == "T",
            });
        }
        None
    }
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
    word_packs: Arc<Vec<WordPack>>,
    num_words: usize,
    settings: Settings,
    rng: ThreadRng,
    queue: VecDeque<usize>,
    excluded_words: VecDeque<usize>,
    max_excluded_words: usize,
    draw_history: Vec<(u32, u32, u32, u32, u32)>,
    round_id: usize,
}
impl Room {
    pub fn new(
        key: String,
        word_packs: Arc<Vec<WordPack>>,
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
            max_excluded_words: 0,
            word_packs,
            num_words: 0,
            settings: Settings::default(),
            rng: ThreadRng::default(),
            queue,
            excluded_words: VecDeque::new(),
            draw_history: Vec::new(),
            round_id: 0,
        };
        room.direct_message(
            &recipient,
            Event::EnterRoom(key, vec![(session_id, username)]),
        );
        room.direct_message(&recipient, Event::EnterLobby(session_id));
        room.send_settings_data(&recipient);
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

    fn send_settings_data(&self, recipient: &Recipient<Event>) {
        let data: Vec<_> = self
            .word_packs
            .iter()
            .enumerate()
            .map(|(i, pack)| (i, pack.get_name().clone(), pack.get_description().clone()))
            .collect();
        self.direct_message(&recipient, Event::SettingsData(data));
    }

    fn choose_new_word(&mut self) -> (usize, usize) {
        loop {
            let word_index = self.rng.gen_range(0, self.num_words);
            if !self.excluded_words.contains(&word_index) {
                if self.excluded_words.len() >= self.max_excluded_words {
                    self.excluded_words.pop_front();
                }
                self.excluded_words.push_back(word_index);
                let mut acc = 0;
                for i in &self.settings.enabled_word_packs {
                    if self.word_packs[*i].list_len() + acc > word_index {
                        return (*i, word_index - acc);
                    }
                    acc += self.word_packs[*i].list_len();
                }
                unreachable!("word index was out of bounds");
            }
        }
    }

    fn get_word(&self, word: (usize, usize)) -> &String {
        self.word_packs[word.0].get_word(word.1)
    }

    pub fn start(&mut self, session_id: usize, lines: Vec<String>, ctx: &mut Context<GameServer>) {
        if let RoomState::Lobby(LobbyState { host }) = self.state {
            if session_id == host {
                if let Some(settings) = Settings::parse_from_lines(lines, self.word_packs.len()) {
                    self.num_words = self
                        .word_packs
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| settings.enabled_word_packs.contains(&i))
                        .map(|(_, x)| x.list_len())
                        .sum();
                    if self.num_words == 0 {
                        warn!(
                            "tried to start game with no word packs in room {}",
                            self.key
                        );
                        return;
                    }
                    trace!(
                        "room {} started with {} words, settings: {:?}",
                        self.key,
                        self.num_words,
                        settings
                    );
                    self.settings = settings;
                    self.new_round(ctx);
                } else {
                    warn!(
                        "session id {} sent invalid settings in room {}",
                        session_id, self.key
                    );
                }
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

        if self.occupants.values().any(|(_, u)| *u == username) {
            trace!("Username {} already exists in room {}", username, self.key);
            self.direct_message(&recipient, Event::UsernameExists(username));
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
                    Event::Winner(winner, self.get_word(word).clone()),
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

    pub fn leave(&mut self, session_id: usize, ctx: &mut Context<GameServer>) -> bool {
        trace!("{} leaving room {}", session_id, self.key);
        if let Some((recipient, _)) = self.occupants.remove(&session_id) {
            self.direct_message(&recipient, Event::LeaveRoom);
            self.broadcast_event(Event::UserGone(session_id));
            if self.occupants.is_empty() {
                return true;
            }
            match self.state {
                RoomState::Lobby(LobbyState { host }) => {
                    if host == session_id {
                        let new_leader = self
                            .queue
                            .iter()
                            .find(|id| self.occupants.get(id).is_some())
                            .expect("user was in occupants but not queue");
                        self.state = RoomState::Lobby(LobbyState { host: *new_leader });
                        self.broadcast_event(Event::EnterLobby(*new_leader));
                        self.send_settings_data(&self.occupants.get(new_leader).unwrap().0);
                    }
                }
                RoomState::Round(RoundState { leader, .. }) => {
                    if leader == session_id {
                        trace!(
                            "Current leader ({}) left room so new round in room {}",
                            session_id,
                            self.key
                        );
                        self.new_round(ctx);
                    }
                }
                _ => {}
            }
        } else {
            warn!(
                "User {} tried to leave room {} when it wasn't a member",
                session_id, self.key
            );
        }
        false
    }

    fn end_round(
        &mut self,
        winner: Option<usize>,
        alternate: Option<usize>,
        ctx: &mut Context<GameServer>,
    ) {
        if let RoomState::Round(RoundState { word, leader }) = self.state {
            self.state = RoomState::Winner(WinnerState {
                word,
                winner,
                alternate,
            });
            self.queue.push_back(leader);
            self.broadcast_event(Event::Winner(winner, self.get_word(word).clone()));
            let key = self.key.clone();
            ctx.run_later(Duration::new(5, 0), move |act, ctx| {
                act.new_round(key, ctx);
            });
        } else {
            error!("end_round called with invalid state in room {}", self.key);
        }
    }

    pub fn new_round(&mut self, ctx: &mut Context<GameServer>) {
        self.round_id += 1;
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
                            Event::NewLeader(
                                self.settings.allow_clear,
                                self.get_word(word).clone(),
                            ),
                        );
                    }
                }

                if self.settings.round_timer {
                    let round_id = self.round_id;
                    let key = self.key.clone();
                    ctx.run_later(Duration::from_secs(120), move |server, ctx| {
                        server.round_timeout(&key, round_id, ctx);
                    });
                }

                trace!(
                    "Room {} has new round with word {:?}, leader {}",
                    self.key,
                    word,
                    new_leader,
                );
                return;
            }
        }
        error!("Room {} had no possible leader for new round", self.key);
    }

    pub fn round_timeout(&mut self, round_id: usize, ctx: &mut Context<GameServer>) {
        if let RoomState::Round(RoundState { .. }) = self.state {
            if round_id == self.round_id {
                trace!("Room {} has timed out", self.key);
                self.end_round(None, None, ctx);
            }
        }
    }

    pub fn handle_guess(
        &mut self,
        session_id: usize,
        message: String,
        ctx: &mut Context<GameServer>,
    ) {
        if let RoomState::Round(RoundState { word, leader }) = self.state {
            if session_id != leader {
                self.broadcast_event(Event::Message(session_id, message.clone()));
                if self.word_packs[word.0].word_matches(word.1, &message.trim().to_lowercase()) {
                    //TODO: sort out alternate
                    self.end_round(Some(session_id), None, ctx);
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

    pub fn clear(&mut self, session_id: usize) {
        if let RoomState::Round(RoundState { leader, .. }) = self.state {
            if leader != session_id {
                warn!(
                    "Uid {} in room {} tried to send clear command when {} was leader",
                    session_id, self.key, leader
                );
                return;
            }
            if !self.settings.allow_clear {
                warn!(
                    "Uid {} in room {} tried to send clear command when not enabled",
                    session_id, self.key
                );
                return;
            }
            self.broadcast_event(Event::ClearCanvas);
            self.draw_history.clear();
        } else {
            warn!(
                "clear command sent by {} in invalid state in room {}",
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
