use std::fs::File;
use std::io::{Read, Result};
use std::iter::FromIterator;
use std::path::Path;

use log::{trace, warn};

use std::collections::HashSet;

#[derive(Debug)]
pub struct WordPack {
    name: String,
    description: String,
    list: Vec<(String, Vec<String>)>,
}

impl WordPack {
    pub fn new<P: std::fmt::Debug + AsRef<Path>>(
        path: &P,
        word_tracker: &mut HashSet<String>,
    ) -> Result<WordPack> {
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
                    warn!(
                        "Word pack {:?} contains duplicate entry `{}`, not adding again",
                        path, word
                    );
                } else {
                    if !word_tracker.insert(word.clone()) {
                        warn!(
                            "Word pack {:?} contains duplicate entry `{}` from another word pack (still adding)",
                            path, word
                        );
                    }

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
    pub fn get_alternate(&self, word: usize, alternate: usize) -> &String {
        &self.list[word].1[alternate]
    }

    pub fn word_matches(&self, index: usize, guess: &str) -> (bool, Option<usize>) {
        let (word, alternates) = &self.list[index];
        if word == guess {
            (true, None)
        } else if let Some((alternate, _)) = alternates
            .iter()
            .enumerate()
            .find(|(_, alternate)| alternate == &guess)
        {
            (true, Some(alternate))
        } else {
            (false, None)
        }
    }
}

impl FromIterator<(String, Vec<String>)> for WordPack {
    fn from_iter<T: IntoIterator<Item = (String, Vec<String>)>>(iter: T) -> Self {
        WordPack {
            name: "".to_string(),
            description: "".to_string(),
            list: iter.into_iter().collect(),
        }
    }
}

pub fn load_word_packs<P: std::fmt::Debug + AsRef<std::path::Path>>(
    word_pack_path: P,
) -> std::io::Result<Vec<WordPack>> {
    use std::fs;
    let mut tracker = std::collections::HashSet::new();
    trace!("loading wordpacks in directory `{:?}`", word_pack_path);
    let mut word_packs = Vec::new();
    let mut paths: Vec<_> = fs::read_dir(word_pack_path)?
        .filter_map(|r| r.ok())
        .filter(|p| p.path().is_file())
        .collect();
    paths.sort_by_key(|entry| entry.path());
    for entry in paths {
        let word_pack = WordPack::new(&entry.path(), &mut tracker)?;
        trace!(
            "loaded word pack {} with {} words",
            word_pack.get_name(),
            word_pack.list_len()
        );
        word_packs.push(word_pack);
    }
    Ok(word_packs)
}
