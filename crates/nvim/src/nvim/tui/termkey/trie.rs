#![forbid(unsafe_code)]

//! A trie over the byte sequences a terminal sends for its special keys.
//!
//! The terminfo driver builds one of these at start-up from the terminal's
//! description, then matches the head of the input buffer against it. Upstream
//! used a hand-rolled arena of 256-wide node arrays with a compaction pass;
//! this is an index arena of sorted child maps, which needs neither.

use crate::src::nvim::types::keyinfo;
use std::collections::BTreeMap;

/// How far a lookup got before it ran out of trie or of input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lookup {
    /// The input's first `consumed` bytes are a complete registered sequence.
    Key { info: keyinfo, consumed: usize },
    /// The input is a proper prefix of at least one registered sequence, so
    /// more bytes could still complete it.
    Partial,
    /// No registered sequence starts this way.
    None,
}

enum Node {
    /// Interior node: next byte -> child index.
    Branch(BTreeMap<u8, usize>),
    /// A sequence ends here.
    Key(keyinfo),
}

pub struct KeyTrie {
    /// Arena; index 0 is the root, always a `Branch`.
    nodes: Vec<Node>,
}

impl Default for KeyTrie {
    fn default() -> Self {
        KeyTrie {
            nodes: vec![Node::Branch(BTreeMap::new())],
        }
    }
}

impl KeyTrie {
    /// Register `seq` as producing `info`.
    ///
    /// Two sequences can conflict: one may be a proper prefix of the other, and
    /// a trie of whole sequences cannot hold both. Upstream noticed neither
    /// case in the right place — inserting a longer sequence over a shorter one
    /// walked into the shorter one's leaf and called `abort()`, killing the
    /// editor on a terminal description that happened to overlap that way,
    /// while inserting a shorter sequence over a longer one fell out of the
    /// insert loop and dropped it silently. Both now drop the *new* sequence,
    /// which is what upstream did in the case it survived.
    pub fn insert(&mut self, seq: &[u8], info: keyinfo) {
        let mut node = 0;
        let mut pos = 0;
        while pos < seq.len() {
            let next = match &self.nodes[node] {
                // A shorter sequence already terminates here; keep it.
                Node::Key(_) => return,
                Node::Branch(children) => children.get(&seq[pos]).copied(),
            };
            match next {
                Some(child) => {
                    node = child;
                    pos += 1;
                }
                None => break,
            }
        }
        // Every byte was already described, so `seq` is a prefix of a longer
        // registered sequence.
        if pos == seq.len() {
            return;
        }
        while pos < seq.len() {
            let child = self.nodes.len();
            self.nodes.push(if pos + 1 == seq.len() {
                Node::Key(info)
            } else {
                Node::Branch(BTreeMap::new())
            });
            match &mut self.nodes[node] {
                Node::Branch(children) => children.insert(seq[pos], child),
                // Unreachable: the walk above stops at any leaf, and every node
                // created here but the last is a branch.
                Node::Key(_) => return,
            };
            node = child;
            pos += 1;
        }
    }

    /// Match the longest registered sequence at the head of `bytes`.
    pub fn lookup(&self, bytes: &[u8]) -> Lookup {
        let mut node = 0;
        for (i, byte) in bytes.iter().enumerate() {
            let child = match &self.nodes[node] {
                Node::Branch(children) => children.get(byte).copied(),
                Node::Key(_) => None,
            };
            let Some(child) = child else {
                return Lookup::None;
            };
            if let Node::Key(info) = self.nodes[child] {
                return Lookup::Key {
                    info,
                    consumed: i + 1,
                };
            }
            node = child;
        }
        Lookup::Partial
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(sym: i32) -> keyinfo {
        keyinfo {
            type_0: 2,
            sym,
            modifier_mask: 0,
            modifier_set: 0,
        }
    }

    fn found(trie: &KeyTrie, bytes: &[u8]) -> Option<(i32, usize)> {
        match trie.lookup(bytes) {
            Lookup::Key { info, consumed } => Some((info.sym, consumed)),
            _ => None,
        }
    }

    #[test]
    fn matches_a_registered_sequence_and_reports_its_length() {
        let mut trie = KeyTrie::default();
        trie.insert(b"\x1bOA", info(7));
        trie.insert(b"\x1b[1;2A", info(8));
        assert_eq!(found(&trie, b"\x1bOA"), Some((7, 3)));
        assert_eq!(found(&trie, b"\x1b[1;2A"), Some((8, 6)));
        // Trailing input beyond the match is left for the next call.
        assert_eq!(found(&trie, b"\x1bOAxyz"), Some((7, 3)));
    }

    #[test]
    fn an_incomplete_sequence_is_partial_and_a_wrong_one_is_none() {
        let mut trie = KeyTrie::default();
        trie.insert(b"\x1bOA", info(7));
        assert_eq!(trie.lookup(b"\x1b"), Lookup::Partial);
        assert_eq!(trie.lookup(b"\x1bO"), Lookup::Partial);
        assert_eq!(trie.lookup(b"\x1bOB"), Lookup::None);
        assert_eq!(trie.lookup(b"x"), Lookup::None);
        assert_eq!(trie.lookup(b""), Lookup::Partial);
    }

    #[test]
    fn an_empty_trie_matches_nothing() {
        let trie = KeyTrie::default();
        assert_eq!(trie.lookup(b"\x1bOA"), Lookup::None);
    }

    #[test]
    fn a_sequence_extending_an_existing_key_is_dropped_not_fatal() {
        let mut trie = KeyTrie::default();
        trie.insert(b"\x1bOA", info(7));
        trie.insert(b"\x1bOAB", info(9));
        assert_eq!(found(&trie, b"\x1bOAB"), Some((7, 3)));
    }

    #[test]
    fn a_sequence_that_prefixes_an_existing_key_is_dropped() {
        let mut trie = KeyTrie::default();
        trie.insert(b"\x1bOAB", info(9));
        trie.insert(b"\x1bOA", info(7));
        assert_eq!(found(&trie, b"\x1bOAB"), Some((9, 4)));
        assert_eq!(trie.lookup(b"\x1bOA"), Lookup::Partial);
    }

    #[test]
    fn re_registering_the_same_sequence_keeps_the_first() {
        let mut trie = KeyTrie::default();
        trie.insert(b"\x1bOA", info(7));
        trie.insert(b"\x1bOA", info(9));
        assert_eq!(found(&trie, b"\x1bOA"), Some((7, 3)));
    }

    #[test]
    fn an_empty_sequence_registers_nothing() {
        let mut trie = KeyTrie::default();
        trie.insert(b"", info(7));
        assert_eq!(trie.lookup(b"\x1b"), Lookup::None);
    }

    #[test]
    fn high_bytes_are_distinct_keys() {
        let mut trie = KeyTrie::default();
        trie.insert(b"\x9bA", info(7));
        trie.insert(b"\xffZ", info(8));
        assert_eq!(found(&trie, b"\x9bA"), Some((7, 2)));
        assert_eq!(found(&trie, b"\xffZ"), Some((8, 2)));
        assert_eq!(trie.lookup(b"\x9bB"), Lookup::None);
    }
}
