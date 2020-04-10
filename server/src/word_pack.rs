use std::path::Path;
use std::fs::File;
use std::io::{Result, Read};

use log::warn;

pub struct WordPack {
    list: Vec<(String, Vec<String>)>,
}

impl WordPack {
    pub fn new<P: std::fmt::Debug + AsRef<Path>>(path: &P) -> Result<WordPack> {
        let mut file = File::open(path)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let mut list = Vec::new();

        for line in contents.split_terminator('\n') {
            let mut parts = line.split(',').map(|part| part.trim().to_lowercase()).filter(|part| !part.is_empty());
            if let Some(word) = parts.next() {
                if list.iter().any(|(main, _)| main == &word) {
                    warn!("Word pack {:?} contains duplicate entry `{}`", path, word);
                } else {
                    list.push((word, parts.collect()));
                }
            } else {
                println!("Line was empty after trim in word pack {}", line);
            }
        }

        Ok(WordPack { list })
    }

    pub fn list_len(&self) -> usize {
        self.list.len()
    }

    pub fn get_word(&self, index: usize) -> &String {
        &self.list[index].0
    }

    pub fn word_matches(&self, index: usize, guess: &str) -> bool {
        let (word, alternates) = &self.list[index];
        word == guess || alternates.iter().any(|alternate| alternate == guess)
    }
}
