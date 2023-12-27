use anyhow::{anyhow, Context, Error, Result};
use std::{fmt::Display, path::Path, str::FromStr, vec::IntoIter};
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

#[derive(Debug, PartialEq, PartialOrd)]
pub(crate) enum Touch {
    Char(char),
    Space,
}

impl Display for Touch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_string().as_str())
    }
}

impl Touch {
    pub(crate) fn to_string(&self) -> String {
        match self {
            Self::Char(c) => c.to_string(),
            Self::Space => " ".to_string(),
        }
    }
}

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

    pub(crate) fn iter<'a>(&'a self) -> PIter<'a> {
        PIter {
            practice: self,
            w_ix: 0,
            ix: 0,
        }
    }

    pub(crate) fn expected_at(&self, count: usize) -> Option<Touch> {
        if count >= self.total_count {
            None
        } else {
            let i = self.skip_index.partition_point(|x| *x <= count);
            if i > 0 {
                let skip = self.skip_index.get(i - 1)?;

                let word = self.words.iter().nth(i - 1)?;
                let char_index = count - skip;
                if char_index >= word.len() {
                    Some(Touch::Space)
                } else {
                    let c = word.char_at(char_index)?;
                    Some(Touch::Char(c))
                }
            } else {
                None
            }
        }
    }
    pub(crate) fn len(&self) -> usize {
        self.total_count
    }
}

pub(crate) struct PIter<'a> {
    practice: &'a Practice,
    w_ix: usize,
    ix: usize,
}

impl<'a> Iterator for PIter<'a> {
    type Item = (Touch, usize);
    fn next(&mut self) -> Option<Self::Item> {
        if self.ix < self.practice.len() {
            // TODO implement it more efficiently here
            let res = match self.practice.expected_at(self.ix) {
                Some(Touch::Space) => {
                    let res = Some((Touch::Space, self.w_ix.clone()));
                    self.w_ix += 1;
                    res
                }
                other => other.map(|x| (x, self.w_ix)),
            };
            self.ix += 1;
            res
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Word(String);

pub(crate) struct Attempt {
    keys: Vec<bool>,
}

impl Attempt {
    pub(crate) fn new() -> Self {
        Attempt { keys: vec![] }
    }
    pub(crate) fn add(&mut self, success: bool) {
        self.keys.push(success);
    }
    pub(crate) fn get(&self, i: usize) -> Option<&bool> {
        self.keys.get(i)
    }
    pub(crate) fn len(&self) -> usize {
        self.keys.len()
    }
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

#[cfg(test)]
mod tests {
    use crate::session::Touch;

    use super::{FromStr, Practice};
    #[test]
    pub fn it_computes_expected_at() {
        let p = Practice::from_str("this is a practice").unwrap();
        assert_eq!(p.expected_at(1), Some(Touch::Char('h')));
        assert_eq!(p.expected_at(3), Some(Touch::Char('s')));
        assert_eq!(p.expected_at(4), Some(Touch::Space));
        assert_eq!(p.expected_at(7), Some(Touch::Space));
        assert_eq!(p.expected_at(13), Some(Touch::Char('c')));
        assert_eq!(p.expected_at(18), None);
    }
}
