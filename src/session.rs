use anyhow::{Context, Result};
use std::{
    fmt::Display,
    fs,
    io::Write,
    path::Path,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum TouchTypingError {
    #[error("a 'word count' was expected")]
    FileParseError,
    #[error("invalid path")]
    InvalidPathError,
}
use rand::{
    distributions::{Distribution, WeightedIndex},
    Rng,
};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub(crate) enum Touch {
    Char(char),
    Space,
}

impl From<char> for Touch {
    fn from(value: char) -> Self {
        if value == ' ' {
            Touch::Space
        } else {
            Touch::Char(value)
        }
    }
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

#[derive(Debug, Clone)]
pub(crate) struct Challenge {
    words: Vec<Word>,
    total_count: usize, // total count of chars including spaces
    skip_index: Vec<usize>,
}

impl FromStr for Challenge {
    type Err = (); // TODO should be the never type
    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        Ok(Self::from(
            s.split_whitespace().map(Word::from).collect::<Vec<Word>>(),
        ))
    }
}

impl From<Vec<Word>> for Challenge {
    fn from(words: Vec<Word>) -> Self {
        let total_count = words.iter().map(|w| w.len()).sum::<usize>() + words.len() - 1;
        let mut skip_index: Vec<usize> = vec![];
        let mut skips: usize = 0;
        for w in &words {
            skip_index.push(skips);
            skips += w.len() + 1;
        }

        Challenge {
            words,
            total_count,
            skip_index,
        }
    }
}

impl Challenge {
    pub(crate) fn generate<R: Rng>(rng: &mut R, size: usize, path: &Path) -> Result<Challenge> {
        let (words, freqs): (Vec<Word>, Vec<u32>) = std::fs::read_to_string(path)
            .with_context(|| TouchTypingError::FileParseError)?
            .lines()
            .filter_map(
                |line| match line.split(' ').collect::<Vec<&str>>().as_slice() {
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
        Ok(Challenge::from(words))
    }

    pub(crate) fn iter<'a>(&'a self) -> CIter<'a> {
        CIter {
            challenge: self,
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

pub(crate) struct CIter<'a> {
    challenge: &'a Challenge,
    w_ix: usize,
    ix: usize,
}

impl<'a> Iterator for CIter<'a> {
    type Item = (Touch, usize);
    fn next(&mut self) -> Option<Self::Item> {
        if self.ix < self.challenge.len() {
            // TODO implement it more efficiently here
            let res = match self.challenge.expected_at(self.ix) {
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

#[derive(Debug, Clone, Default)]
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
}

impl Word {
    pub(crate) fn from(s: &str) -> Word {
        Word(s.to_string())
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn char_at(&self, i: usize) -> Option<char> {
        self.0.chars().nth(i)
    }
}

#[derive(Clone)]
pub(crate) struct Practice {
    challenge: Challenge,
    attempt: Attempt,
    name: String,
    /// max(index of next touch in the challenge, challenge.len())
    cursor: usize,
}

impl std::fmt::Debug for Practice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "expected = {:?}",
            self.challenge.expected_at(self.cursor)
        )
    }
}
impl std::fmt::Display for Practice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Practice {
    pub(crate) fn generate<R: Rng>(rng: &mut R, size: usize, path: &Path) -> Result<Practice> {
        let challenge = Challenge::generate(rng, size, path)?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let name = format!("practice_{}.json", now.as_secs());
        Ok(Self::new(challenge, name))
    }
    pub(crate) fn new(challenge: Challenge, name: String) -> Practice {
        Practice {
            challenge,
            attempt: Attempt::new(),
            name,
            cursor: 0,
        }
    }

    pub(crate) fn iter<'a>(&'a self) -> PIter<'a> {
        PIter::new(self)
    }

    pub(crate) fn name<'a>(&'a self) -> &'a String {
        &self.name
    }

    pub(crate) fn save(self, path: &Path) -> Result<String> {
        let path = path.join(self.name);
        let path = path.as_path();
        let succ = self.attempt.keys.iter().filter(|x| **x).count();
        let total = self.attempt.keys.len();
        let mut f = fs::File::create(&path).context(format!("cannot create file at {:?}", path))?;
        writeln!(f, "total = {}", total)?;
        writeln!(f, "succ = {}", succ)?;

        let path = path.to_str().ok_or(TouchTypingError::InvalidPathError)?;
        Ok(path.to_string())
    }

    ///
    /// Returns if the touch is the expected one or None if
    /// no more touches are expected.
    pub(crate) fn check(&self, touch: &Touch) -> Option<bool> {
        let expected = self.challenge.expected_at(self.cursor);

        expected.map(|e| e == *touch)
    }

    pub(crate) fn press(&mut self, touch: &Touch) -> Option<bool> {
        if let Some(success) = self.check(touch) {
            self.attempt.add(success);
            self.cursor += 1;
            Some(success)
        } else {
            None
        }
    }
}
pub(crate) struct PIter<'a> {
    practice: &'a Practice,
    challenge_iter: CIter<'a>,
}

pub(crate) type WordIndex = usize;

#[derive(Clone)]
pub(crate) enum TouchState {
    Attempted(bool),
    Current(bool),
    Next,
    Future,
}

impl<'a> PIter<'a> {
    /// Returns the state of the current
    /// item at position `self.challenge_iter.ix` with
    /// regards to self.practice.cursor
    ///
    fn state(&self) -> TouchState {
        if self.challenge_iter.ix == self.practice.cursor {
            TouchState::Current(self.practice.cursor == 0)
        } else if self.challenge_iter.ix == self.practice.cursor + 1 {
            if self.practice.cursor == 0 {
                TouchState::Future
            } else {
                TouchState::Next
            }
        } else if self.challenge_iter.ix > self.practice.cursor + 1 {
            TouchState::Future
        } else {
            // challenge_iter.x <= self.practice.cursor
            if let Some(b) = self.practice.attempt.get(self.challenge_iter.ix) {
                TouchState::Attempted(*b)
            } else {
                unreachable!("should always have a value")
            }
        }
    }
    fn new(practice: &'a Practice) -> Self {
        let challenge_iter = practice.challenge.iter();
        PIter {
            practice,
            challenge_iter,
        }
    }
}

impl<'a> Iterator for PIter<'a> {
    type Item = (Touch, TouchState, WordIndex);
    fn next(&mut self) -> Option<Self::Item> {
        let state = self.state();
        self.challenge_iter.next().map(|(t, w)| (t, state, w))
    }
}
pub(crate) struct PracticeGenerator<R> {
    rng: R,
    size: usize,
    path: String,
}

impl<R> PracticeGenerator<R> {
    pub(crate) fn new(rng: R, size: usize, path: &str) -> PracticeGenerator<R> {
        PracticeGenerator {
            rng,
            size,
            path: path.to_string(),
        }
    }
    pub(crate) fn generate(&mut self) -> Result<Practice>
    where
        R: rand::Rng,
    {
        Practice::generate(&mut self.rng, self.size, &Path::new(self.path.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use crate::session::Touch;

    use super::{Challenge, FromStr};
    #[test]
    pub fn it_computes_expected_at() {
        let p = Challenge::from_str("this is a practice").unwrap();
        assert_eq!(p.expected_at(1), Some(Touch::Char('h')));
        assert_eq!(p.expected_at(3), Some(Touch::Char('s')));
        assert_eq!(p.expected_at(4), Some(Touch::Space));
        assert_eq!(p.expected_at(7), Some(Touch::Space));
        assert_eq!(p.expected_at(13), Some(Touch::Char('c')));
        assert_eq!(p.expected_at(18), None);
    }
}
