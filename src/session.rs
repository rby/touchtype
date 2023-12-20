use anyhow::{anyhow, Context, Error, Result};
use std::{path::Path, vec::IntoIter};
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum TouchTypingError {
    #[error("a 'word count' was expected")]
    FileParseError,
}

use rand::{
    distributions::{Distribution, WeightedIndex},
    thread_rng, Rng,
};
///
/// A session is a set of consecuitive  practices.
///
struct Session {
    practices: Vec<Practice>,
}

#[derive(Debug)]
pub(crate) struct Practice {
    words: Vec<Word>,
}

#[derive(Clone, Debug)]
pub(crate) struct Word(String);

pub(crate) struct Attempt {
    words: Vec<Word>,
}

impl Word {
    pub(crate) fn from(s: &str) -> Word {
        Word(s.to_string())
    }
    pub(crate) fn into_str<'a>(&'a self) -> &'a str {
        self.0.as_str()
    }
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
}

impl Practice {
    pub(crate) fn generate<R: Rng>(rng: &mut R, size: usize, path: &Path) -> Result<Practice> {
        let (words, freqs): (Vec<Word>, Vec<i32>) = std::fs::read_to_string(path)
            .with_context(|| format!("cannot read file {:?}", path.to_str()))?
            .lines()
            .filter_map(
                |line| match line.split(" ").collect::<Vec<&str>>().as_slice() {
                    [word, count] => {
                        let count = count.parse::<i32>().ok()?;
                        Some((Word(word.to_string()), count))
                    }
                    _ => None,
                },
            )
            .unzip();
        let dist = WeightedIndex::new(freqs)?;
        let words = dist
            .sample_iter(rng)
            .map(|i| words[i].clone())
            .take(size)
            .collect();
        Ok(Practice { words })
    }

    pub(crate) fn iter<'a>(&'a self) -> std::slice::Iter<'a, Word> {
        self.words.iter()
    }
}
