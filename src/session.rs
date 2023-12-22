use anyhow::{anyhow, Context, Error, Result};
use std::{path::Path, str::FromStr, vec::IntoIter};
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum TouchTypingError {
    #[error("a 'word count' was expected")]
    FileParseError,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub(crate) enum ExpectedKey {
    Char(char),
    Space,
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
    total_count: usize, // total count of chars including spaces
    skip_index: Vec<usize>,
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

    pub(crate) fn char_at(&self, i: usize) -> Option<char> {
        self.0.chars().nth(i)
    }
}

impl FromStr for Practice {
    type Err = (); // TODO should be the never type
    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        Ok(Self::from(
            s.split_whitespace().map(Word::from).collect::<Vec<Word>>(),
        ))
    }
}

impl From<Vec<Word>> for Practice {
    fn from(words: Vec<Word>) -> Self {
        let total_count = words.iter().map(|w| w.len()).sum::<usize>() + words.len() - 1;
        let mut skip_index: Vec<usize> = vec![];
        let mut skips: usize = 0;
        for w in &words {
            skip_index.push(skips);
            skips += w.len() + 1;
        }

        Practice {
            words,
            total_count,
            skip_index,
        }
    }
}

impl Practice {
    pub(crate) fn generate<R: Rng>(rng: &mut R, size: usize, path: &Path) -> Result<Practice> {
        let (words, freqs): (Vec<Word>, Vec<u32>) = std::fs::read_to_string(path)
            .with_context(|| format!("cannot read file {:?}", path.to_str()))?
            .lines()
            .filter_map(
                |line| match line.split(" ").collect::<Vec<&str>>().as_slice() {
                    [word, count] => {
                        let count = count.parse::<u32>().ok()?;
                        Some((Word::from(word), count))
                    }
                    _ => None,
                },
            )
            .unzip();
        let dist = WeightedIndex::new(freqs)?;
        let words: Vec<Word> = dist
            .sample_iter(rng)
            .map(|i| words[i].clone())
            .take(size)
            .collect();
        Ok(Practice::from(words))
    }

    pub(crate) fn iter<'a>(&'a self) -> std::slice::Iter<'a, Word> {
        self.words.iter()
    }

    pub(crate) fn expected_at(&self, count: usize) -> Option<ExpectedKey> {
        if count >= self.total_count {
            None
        } else {
            let i = self.skip_index.partition_point(|x| *x <= count);
            if i > 0 {
                let skip = self.skip_index.get(i - 1)?;

                let word = self.words.iter().nth(i - 1)?;
                println!("word at {count} is {word:?}");
                let char_index = count - skip;
                if char_index >= word.len() {
                    Some(ExpectedKey::Space)
                } else {
                    let c = word.char_at(char_index)?;
                    Some(ExpectedKey::Char(c))
                }
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::session::ExpectedKey;

    use super::{FromStr, Practice};
    #[test]
    pub fn it_computes_expected_at() {
        let p = Practice::from_str("this is a practice").unwrap();
        assert_eq!(p.expected_at(1), Some(ExpectedKey::Char('h')));
        assert_eq!(p.expected_at(3), Some(ExpectedKey::Char('s')));
        assert_eq!(p.expected_at(4), Some(ExpectedKey::Space));
        assert_eq!(p.expected_at(7), Some(ExpectedKey::Space));
        assert_eq!(p.expected_at(13), Some(ExpectedKey::Char('c')));
        assert_eq!(p.expected_at(18), None);
    }
}
