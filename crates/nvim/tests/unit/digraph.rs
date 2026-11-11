//! Default digraph table integrity. The table is user-visible contract
//! (`:digraphs`, CTRL-K, `digraph_get()`), so it is locked down with a
//! checksum computed from the transpiled table it replaced (which itself
//! matched `v0.12.4` `digraph.c` entry for entry).

use neovim::digraph::{default_digraphs, lookup_default};
use std::collections::HashSet;

#[test]
fn default_table_is_unchanged() {
    let table = default_digraphs();
    assert_eq!(table.len(), 1366);

    // FNV-1a over (char1, char2, result as little-endian u32), matching the
    // extraction of the original table.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for d in table {
        let mut bytes = vec![d.char1, d.char2];
        bytes.extend_from_slice(&d.result.raw().to_le_bytes());
        for b in bytes {
            hash = (hash ^ b as u64).wrapping_mul(0x100_0000_01b3);
        }
    }
    assert_eq!(hash, 0x4abb_9183_e972_bd54);
}

#[test]
fn default_table_well_formed() {
    let table = default_digraphs();
    let mut seen = HashSet::new();
    for d in table {
        // Keys are printable ASCII pairs, results are valid nonzero
        // codepoints, and no pair is defined twice.
        assert!(d.char1.is_ascii() && !d.char1.is_ascii_control(), "{d:?}");
        assert!(d.char2.is_ascii() && !d.char2.is_ascii_control(), "{d:?}");
        assert!(!d.result.is_nul() && d.result.as_char().is_some(), "{d:?}");
        assert!(seen.insert((d.char1, d.char2)), "duplicate {d:?}");
    }
}

#[test]
fn lookup_finds_known_digraphs() {
    let get = |c1: u8, c2: u8| lookup_default(c1, c2).map(|key| key.code().get());
    // First and last entries plus well-known digraphs from the docs.
    assert_eq!(get(b'N', b'U'), Some(0x0a));
    assert_eq!(get(b's', b't'), Some(0xfb06));
    assert_eq!(get(b'a', b':'), Some(0xe4)); // ä
    assert_eq!(get(b'E', b'u'), Some(0x20ac)); // €
    assert_eq!(get(b'C', b'o'), Some(0xa9)); // ©
    assert_eq!(get(b'-', b'>'), Some(0x2192)); // →
    assert_eq!(get(b'0', b'u'), Some(0x263a)); // ☺
    assert_eq!(get(b'?', b'I'), Some(0xbf)); // ¿
    assert_eq!(get(b'S', b'E'), Some(0xa7)); // §
    assert_eq!(get(b'x', b'x'), None);
}

/// `:digraphs` and the `digraph_*()` functions' refusals, with every message
/// each puts out and what it answered: a message reported twice, or not at
/// all, moves the row.
#[cfg(not(miri))]
#[test]
fn refusals_report_once() {
    use crate::support::{Probe, Sandbox};
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    for (cmd, want) in [
        (
            "digraphs a",
            "E1214: Digraph must be just two characters: a",
        ),
        ("digraphs ab", "E39: Number expected"),
        ("digraphs ab x", "E39: Number expected"),
        ("digraphs ab 0x41", "none"),
        ("digraphs zq 233", "none"),
        (
            "exe \"digraphs \\<Esc>a 65\"",
            "E104: Escape not allowed in digraph",
        ),
        ("digraphs zq 233 zr", "E39: Number expected"),
        ("digraphs zq 233 zr 234", "none"),
    ] {
        fx.check_messages(cmd, want);
    }
    for (call, want, answer) in [
        (
            "digraph_get('a')",
            "E1214: Digraph must be just two characters: a",
            "''",
        ),
        (
            "digraph_get('abc')",
            "E1214: Digraph must be just two characters: abc",
            "''",
        ),
        ("digraph_get([])", "E730: Using a List as a String", "''"),
        ("digraph_get('zq')", "none", "'é'"),
        (
            "digraph_set('a', 'x')",
            "E1214: Digraph must be just two characters: a",
            "v:false",
        ),
        (
            "digraph_set('abc', 'x')",
            "E1214: Digraph must be just two characters: abc",
            "v:false",
        ),
        (
            "digraph_set('', 'x')",
            "E1214: Digraph must be just two characters: ",
            "v:false",
        ),
        (
            "digraph_set([], 'x')",
            "E730: Using a List as a String\nE1214: Digraph must be just two characters: [NULL]",
            "v:false",
        ),
        (
            "digraph_set('a\\<Esc>', 'x')",
            "E1214: Digraph must be just two characters: a\\<Esc>",
            "v:false",
        ),
        (
            "digraph_set(\"\\<Esc>a\", 'x')",
            "E104: Escape not allowed in digraph",
            "v:false",
        ),
        (
            "digraph_set('zs', 'xy')",
            "E1215: Digraph must be one character: xy",
            "v:false",
        ),
        (
            "digraph_set('zs', [])",
            "E730: Using a List as a String",
            "v:false",
        ),
        ("digraph_set('zs', '')", "none", "v:true"),
        ("digraph_set('zs', 'é')", "none", "v:true"),
        (
            "digraph_setlist(1)",
            "E1216: digraph_setlist() argument must be a list of lists with two items",
            "v:false",
        ),
        (
            "digraph_setlist([1])",
            "E1216: digraph_setlist() argument must be a list of lists with two items",
            "v:false",
        ),
        (
            "digraph_setlist([['zt']])",
            "E1216: digraph_setlist() argument must be a list of lists with two items",
            "v:false",
        ),
        (
            "digraph_setlist([['zt', 'a'], ['zuv', 'b'], ['zw', 'c']])",
            "E1214: Digraph must be just two characters: zuv",
            "v:false",
        ),
        ("digraph_setlist(v:_null_list)", "none", "v:true"),
        (
            "[digraph_get('zs'), digraph_get('zt'), digraph_get('zw')]",
            "none",
            "['é', 'a', 'w']",
        ),
    ] {
        fx.check_messages(&format!("let g:r = {call}"), want);
        fx.check("g:r", answer);
    }
}
