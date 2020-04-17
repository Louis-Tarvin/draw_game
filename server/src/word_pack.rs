use std::fs::File;
use std::io::{Read, Result};
use std::path::Path;

use log::{trace, warn};

pub struct WordPack {
    name: String,
    description: String,
    list: Vec<(String, Vec<String>)>,
}

impl WordPack {
    pub fn new<P: std::fmt::Debug + AsRef<Path>>(path: &P) -> Result<WordPack> {
        let mut file = File::open(path)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let mut list = Vec::new();

        let mut lines = contents.lines();
        let name = lines
            .next()
            .expect("Wordpack did not have a title")
            .to_string();
        let description = lines
            .next()
            .expect("Wordpack did not have a title")
            .to_string();

        for line in lines {
            let mut parts = line
                .split(',')
                .map(|part| part.trim().to_lowercase())
                .filter(|part| !part.is_empty());
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

        Ok(WordPack {
            name,
            description,
            list,
        })
    }

    pub fn list_len(&self) -> usize {
        self.list.len()
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_description(&self) -> &String {
        &self.description
    }

    pub fn get_word(&self, index: usize) -> &String {
        &self.list[index].0
    }

    pub fn word_matches(&self, index: usize, guess: &str) -> bool {
        let (word, alternates) = &self.list[index];
        word == guess || alternates.iter().any(|alternate| alternate == guess)
    }
}

pub fn load_word_packs<P: std::fmt::Debug + AsRef<std::path::Path>>(
    word_pack_path: P,
) -> std::io::Result<Vec<WordPack>> {
    use std::fs;
    trace!("loading wordpacks in directory `{:?}`", word_pack_path);
    let mut word_packs = Vec::new();
    for entry in fs::read_dir(word_pack_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let word_pack = WordPack::new(&path)?;
            trace!(
                "loaded word pack {} with {} words",
                word_pack.get_name(),
                word_pack.list_len()
            );
            word_packs.push(word_pack);
        }
    }
    Ok(word_packs)
}
