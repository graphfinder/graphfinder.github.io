//! Word-ladder: change one word into another, one letter at a time, with every
//! intermediate word in a dictionary.

use std::collections::HashSet;

use crate::traits::{Graph, Heuristic};

/// A word-ladder domain over a fixed dictionary of equal-length lowercase words.
/// A **state** is a `String`; its neighbours are the dictionary words that
/// differ in exactly one letter (cost `1`).
#[derive(Clone, Debug)]
pub struct WordLadder {
    dict: HashSet<String>,
}

impl WordLadder {
    /// Build from any iterator of words. Words are lowercased; the caller is
    /// expected to use a single word length (the classic ladder rule).
    pub fn new<I, S>(words: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let dict = words.into_iter().map(|w| w.into().to_lowercase()).collect();
        Self { dict }
    }

    /// Add a word to the dictionary (e.g. to guarantee the start/goal are nodes).
    pub fn insert(&mut self, word: impl Into<String>) {
        self.dict.insert(word.into().to_lowercase());
    }

    /// Whether a word is in the dictionary.
    pub fn contains(&self, word: &str) -> bool {
        self.dict.contains(word)
    }
}

impl Graph for WordLadder {
    type Node = String;

    fn neighbors(&self, word: &String) -> Vec<(String, f64)> {
        let mut out = Vec::new();
        let mut buf = word.clone().into_bytes();
        for i in 0..buf.len() {
            let original = buf[i];
            for c in b'a'..=b'z' {
                if c == original {
                    continue;
                }
                buf[i] = c;
                // SAFETY: we only substitute ASCII lowercase letters.
                if let Ok(candidate) = std::str::from_utf8(&buf) {
                    if self.dict.contains(candidate) {
                        out.push((candidate.to_string(), 1.0));
                    }
                }
            }
            buf[i] = original;
        }
        out
    }
}

/// Number of differing letters between two equal-length words (Hamming
/// distance). Admissible and consistent: each step fixes at most one letter.
#[derive(Clone, Copy, Debug, Default)]
pub struct LadderHamming;

impl Heuristic<String> for LadderHamming {
    fn estimate(&self, node: &String, goal: &String) -> f64 {
        node.bytes()
            .zip(goal.bytes())
            .filter(|(a, b)| a != b)
            .count() as f64
    }
}
