/// Domain model for the App.
///
/// Challenge: A randomly generated sequence of words
/// Practice: The challenge on top of which we put attempts and a cursor
/// Touch: Key is overloaded term (GTK) but it just mean a key
/// Attempt: sequence (sometimes uncomplete) of true or false wether key was
/// successfull typed.
///
///
use anyhow::{Context, Result};
use rand::{
    distributions::{Distribution, WeightedIndex},
    Rng,
};
use std::{
    fmt::Display,
    fs,
    io::Write,
    path::Path,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};
use thiserror::Error;

/// Simple type alias for WordIndex
pub(crate) type WordIndex = usize;

#[derive(Error, Debug)]
pub(crate) enum TouchTypingError {
    #[error("A line for form 'word(str) count(usize)' was expected")]
    FileParseError,
    #[error("invalid path")]
    InvalidPathError,
}

/// Differentiates between Space and any other characters.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub(crate) enum Touch {
    Char(char),
    Space,
}

/// Sequence of words that the user will try
#[derive(Debug, Clone)]
pub(crate) struct Challenge {
    /// The words in sequence
    words: Vec<Word>,
    /// number of Touches in the challenge
    total_count: usize, // total count of chars including spaces
    /// If the challenge is a sequence of chars, this index will give the
    /// starting offset for each word.
    skip_index: Vec<WordIndex>,
}

/// A `Touch` iterator for a challenge
pub(crate) struct CIter<'a> {
    challenge: &'a Challenge,
    word_ix: usize,
    ix: usize,
}

/// New type for a Word which is just a string
#[derive(Clone, Debug)]
pub(crate) struct Word(String);

/// Records the current progress in the challenge.
/// `keys[i]` will be true if the touch was expected, otherwise false.
#[derive(Debug, Clone, Default)]
pub(crate) struct Attempt {
    touches: Vec<bool>,
}

#[derive(Clone)]
pub(crate) struct Practice {
    challenge: Challenge,
    attempt: Attempt,
    name: String,
    /// max(index of next touch in the challenge, challenge.len())
    cursor: usize,
}

/// Given an underlying challenge, this is an iterator that
/// helps displaying the current state of progress.
pub(crate) struct PIter<'a> {
    practice: &'a Practice,
    challenge_iter: CIter<'a>,
}

/// a `Touch` state within a practice, mostly to help with UI.
#[derive(Clone)]
pub(crate) enum TouchState {
    /// was attempted successfully or not
    Attempted(bool),
    /// the last one attempted.
    Current(bool),
    /// it's the next expected touch
    Next,
    /// part of the future
    Future,
}

/// A generator for the practice.
///
/// R holds usually a Random Number Generator
pub(crate) struct PracticeGenerator<R> {
    /// Random Number Generator
    rng: R,
    /// Size in number of words
    size: usize,
    /// Path to use for generating words.
    path: String,
}

// Implementations

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

impl Touch {}

impl FromStr for Challenge {
    // TODO should be the never type
    type Err = ();
    /// The challenge as a string of space separated words.
    /// Mostly for test purpose.
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
    /// Returns a random challenge
    ///
    /// # Arguments
    ///
    /// * `rng` : a Random number generator
    /// * `size`: number of words in the challenge
    /// * `path`: path to the file the .freq files that contains the words to
    /// sample from,
    ///
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

    /// Returns an iterator for words in the challenge.
    pub(crate) fn iter<'a>(&'a self) -> CIter<'a> {
        CIter {
            challenge: self,
            word_ix: 0,
            ix: 0,
        }
    }

    /// Returns the `Touch` expected at position `count` or None.
    pub(crate) fn expected_at(&self, position: usize) -> Option<Touch> {
        if position >= self.total_count {
            None
        } else {
            let i = self.skip_index.partition_point(|x| *x <= position);
            if i > 0 {
                let skip = self.skip_index.get(i - 1)?;

                let word = self.words.iter().nth(i - 1)?;
                let char_index = position - skip;
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

    /// Returns the number of `Touch`s expected in the challenge.
    pub(crate) fn len(&self) -> usize {
        self.total_count
    }
}

impl<'a> Iterator for CIter<'a> {
    /// The `Touch` and it's word index.
    /// If it's a space then the word index will be of the previous one.
    type Item = (Touch, WordIndex);
    fn next(&mut self) -> Option<Self::Item> {
        if self.ix < self.challenge.len() {
            // TODO improve the performance
            let res = match self.challenge.expected_at(self.ix) {
                Some(Touch::Space) => {
                    let wix = self.word_ix;
                    let res = Some((Touch::Space, wix));
                    self.word_ix += 1;
                    res
                }
                other => other.map(|x| (x, self.word_ix)),
            };
            self.ix += 1;
            res
        } else {
            None
        }
    }
}

impl Attempt {
    pub(crate) fn new() -> Self {
        Attempt { touches: vec![] }
    }
    pub(crate) fn add(&mut self, success: bool) {
        self.touches.push(success);
    }
    pub(crate) fn get(&self, i: usize) -> Option<&bool> {
        self.touches.get(i)
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
        let name = format!("practice_{}.txt", now.as_secs());
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
        let succ = self.attempt.touches.iter().filter(|x| **x).count();
        let total = self.attempt.touches.len();
        let mut f = fs::File::create(&path).context(format!("cannot create file at {:?}", path))?;
        writeln!(f, "total = {}", total)?;
        writeln!(f, "succ = {}", succ)?;

        let path = path.to_str().ok_or(TouchTypingError::InvalidPathError)?;
        Ok(path.to_string())
    }

    ///
    /// Returns wether the touch is the expected one or None if
    /// no more touches are expected.
    pub(crate) fn check(&self, touch: &Touch) -> Option<bool> {
        self.challenge.expected_at(self.cursor).map(|e| e == *touch)
    }

    /// Records the attempt of pressing a touch in a challenge
    /// if no touch is expected (challenge finished) we return None.
    /// Otherwise we return wether the touch was expected or not.
    pub(crate) fn press(&mut self, touch: &Touch) -> Option<bool> {
        let success = self.check(touch)?;
        self.attempt.add(success);
        self.cursor += 1;
        Some(success)
    }
}

/// Iterates on a practice and give the state of each touch in the challenge.
impl<'a> PIter<'a> {
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

    /// Returns a new iterator for the practice.
    fn new(practice: &'a Practice) -> Self {
        PIter {
            practice,
            challenge_iter: practice.challenge.iter(),
        }
    }
}

impl<'a> Iterator for PIter<'a> {
    /// Touch, state and its word index.
    type Item = (Touch, TouchState, WordIndex);
    fn next(&mut self) -> Option<Self::Item> {
        // gets the state *before* calling `next` on `challenge_iter`
        let state = self.state();
        self.challenge_iter.next().map(|(t, w)| (t, state, w))
    }
}

impl<R> PracticeGenerator<R> {
    /// Returns a new Generator.
    pub(crate) fn new(rng: R, size: usize, path: &str) -> PracticeGenerator<R> {
        PracticeGenerator {
            rng,
            size,
            path: path.to_string(),
        }
    }
    /// Generates a new practice.
    pub(crate) fn generate(&mut self) -> Result<Practice>
    where
        R: rand::Rng,
    {
        Practice::generate(&mut self.rng, self.size, &Path::new(self.path.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use crate::model::Touch;

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
