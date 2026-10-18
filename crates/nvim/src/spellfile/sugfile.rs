//! Building the `.sug` file that sits beside a `.spl`.
//!
//! Sound-a-like suggestions need to go from "what the bad word sounds
//! like" back to "which words sound like that". The `.spl` cannot answer
//! that, so `:mkspell` reads its own output back and builds a second file:
//! a trie over the *sound-folded* form of every word, where each word end
//! carries the list of word numbers that fold to it.
//!
//! The steps, in order:
//!
//! 1. [`sug_filltree`] walks the finished `.spl`'s case-folded tree,
//!    sound-folds each word, and adds the result to a fresh tree. Words are
//!    numbered as they are met, and the number is smuggled through
//!    `tree_add_word`'s flags and region arguments — which is why
//!    `si_sugtree` exists, to stop the tree from merging two entries that
//!    differ only in their number.
//! 2. `wordtree_compress` shares the tails, as for any tree.
//! 3. [`sug_maketable`] and [`sug_filltable`] collect, for each word end,
//!    the numbers that reached it, and store them as one line in a scratch
//!    buffer.
//! 4. [`sug_write`] writes the trie and then those lines.
//!
//! # Word numbers on disk
//!
//! A word end's numbers are stored as *differences*, so they stay small,
//! and each difference goes out in one to four bytes by
//! [`offset2bytes`] — a length-prefix scheme in the style of UTF-8, where
//! no byte is ever zero so that NUL can end the list. Real dictionaries
//! reach the three-byte form; the four-byte form needs more than about
//! four million words.

#![forbid(unsafe_code)]

use crate::semsg;
use crate::smsg;
use core::ffi::{CStr, c_int, c_uint};
use std::ffi::OsStr;
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use crate::getchar::state::got_int;
use crate::memline::ml_append_buf_terminated;
use crate::memory::XString;
use crate::message::e_write;
use crate::message::emsg;
use crate::message_fmt::msg_cstr;
use crate::os::cshim::gettext;
use crate::os::input::line_breakcheck;
use crate::path::path_full_compare;
use crate::spell::{languages, open_spellbuf, put_lang, spell_soundfold, take_lang, with_lang};
use crate::types::{Failed, LineNr, NUL, SpellIdx, SpellLang};

use super::wordtree::{NodeId, tree_add_word, wordtree_alloc, wordtree_compress};
use super::write::{SplWriter, clear_node, put_node};
use super::{
    FAIL, MAXWLEN, OK, SpellInfo, VIMSUGMAGIC, VIMSUGVERSION, kEqualFiles, spell_load_file,
    spell_message, spell_message_fmt,
};

/// Read the just-written `.spl` back and turn it into a `.sug`.
///
/// The `.spl` is re-read rather than reusing what is still in memory: the
/// word numbers have to match the order the *reader* will see, and the
/// tree in memory has been compressed since.
pub(super) fn spell_make_sugfile(spin: &mut SpellInfo, wfname: &CStr) {
    // Prefer an already-loaded copy of this file, taken out of its slot
    // while the walk below rewrites its word counts.
    let loaded = languages().into_iter().find(|&id| {
        with_lang(id, |slang| {
            path_full_compare(wfname, slang.sl_fname.as_cstr(), false, true) as c_uint
                == kEqualFiles as c_uint
        }) == Some(true)
    });
    let (mut slang, slot) = match loaded.and_then(|id| take_lang(id).map(|slang| (slang, id))) {
        Some((slang, id)) => (slang, Some(id)),
        None => {
            spell_message(spin, c"Reading back spell file...");
            let Some(slang) = spell_load_file(wfname, None, false) else {
                return;
            };
            (slang, None)
        }
    };

    // Start a fresh tree: the ones the `.spl` was built from have already
    // been released. Upstream kept their dangling roots, and a compression
    // run the `.sug` walk set off then compressed the freed keep-case tree;
    // here only the new tree is left to compress.
    spin.clear_trees();
    spin.si_keeproot = None;
    spin.si_prefroot = None;
    spin.si_compress_cnt = 0;
    spin.si_foldwcount = 0;

    spell_message(spin, c"Performing soundfolding...");
    if sug_filltree(spin, &mut slang).is_ok() && sug_maketable(spin) != FAIL {
        let done = spin.spellbuf().line_count() as i64;
        smsg!(0, "Number of words after soundfolding: {}", done);
        spell_message(spin, super::wordtree::MSG_COMPRESSING);
        let foldroot = spin.si_foldroot;
        wordtree_compress(spin, foldroot, c"case-folded");

        // Same path as the `.spl`, with the extension's last two
        // letters swapped: "spl" becomes "sug".
        let mut bytes = wfname.to_bytes().to_vec();
        let len = bytes.len();
        bytes[len - 2] = b'u';
        bytes[len - 1] = b'g';
        let fname = XString::from_bytes(&bytes);
        sug_write(spin, fname.as_cstr());
    }

    if let Some(id) = slot {
        put_lang(id, slang);
    }
    spin.clear_trees();
    spin.si_spellbuf = None;
}

/// Walk the loaded `.spl`'s case-folded tree and add every word's
/// sound-folded form to a new tree, numbering words as they are met.
///
/// The walk doubles as a rewrite of the loaded tree: `sl_fidxs` for a word
/// end is overwritten with the number of words in that sub-tree, which is
/// what the suggestion search will later use to turn a position in the tree
/// into a word number.
fn sug_filltree(spin: &mut SpellInfo, slang: &mut SpellLang) -> Result<(), Failed> {
    // The walk is bounded by the byte counts the tree itself carries, and
    // depth by MAXWLEN, which is the longest word the tree can hold.
    let mut arridx = [0usize; MAXWLEN];
    let mut curi = [0usize; MAXWLEN];
    let mut tword = [0u8; MAXWLEN];
    let mut tsalword = [0u8; MAXWLEN];
    let mut wordcount: [SpellIdx; MAXWLEN] = [0; MAXWLEN];
    let mut words_done: c_uint = 0;

    let foldroot = wordtree_alloc(spin);
    spin.si_foldroot = Some(foldroot);
    spin.si_sugtree = 1;

    if slang.sl_fold_tree.is_empty() {
        return Err(Failed);
    }

    curi[0] = 1;
    let mut depth: usize = 0;
    loop {
        if got_int.get() {
            break;
        }
        if curi[depth] > slang.sl_fold_tree.node_len(arridx[depth]) {
            // Done with this node: publish the sub-tree's word count
            // where its index used to be, and fold it into the parent.
            let at = arridx[depth];
            let total = wordcount[depth];
            slang.sl_fold_tree.idxs_mut()[at] = total;
            let at_root = depth == 0;
            if !at_root {
                wordcount[depth - 1] += total;
                depth -= 1;
            }
            line_breakcheck();
            if at_root {
                break;
            }
            continue;
        }

        let mut n = arridx[depth] + curi[depth];
        curi[depth] += 1;
        if !slang.sl_fold_tree.ends_word(n) {
            // Descend one byte.
            tword[depth] = slang.sl_fold_tree.byte(n);
            depth += 1;
            arridx[depth] = slang.sl_fold_tree.child_node(n);
            curi[depth] = 1;
            wordcount[depth] = 0;
            continue;
        }

        // A word end. Its number goes in through the flags and region
        // arguments, split across the two.
        tword[depth] = 0;
        spell_soundfold(slang, &tword, true, &mut tsalword);
        let (hi, lo) = ((words_done >> 16) as c_int, (words_done & 0xffff) as c_int);
        tree_add_word(spin, &tsalword, foldroot, hi, lo, 0);
        words_done = words_done.wrapping_add(1);
        wordcount[depth] += 1;

        // Compressing here would renumber what has already been
        // written, so hold it off for the whole walk.
        spin.si_arena.block_count = 0;

        // One word may end several times over, once per flag set; they
        // are the same word for this purpose.
        while n + 1 < slang.sl_fold_tree.len() && slang.sl_fold_tree.ends_word(n + 1) {
            n += 1;
            curi[depth] += 1;
        }
    }

    smsg!(0, "Total number of words: {}", words_done);
    Ok(())
}

/// Collect each word end's word numbers into one line of a scratch buffer.
fn sug_maketable(spin: &mut SpellInfo) -> c_int {
    spin.si_spellbuf = Some(open_spellbuf());

    let mut line: Vec<u8> = Vec::with_capacity(100);
    let root = spin.si_foldroot.and_then(|root| spin.node(root).wn_sibling);
    if sug_filltable(spin, root, 0, &mut line) == -1 {
        FAIL
    } else {
        OK
    }
}

/// Walk the sound-fold tree, and for each run of word ends write one line
/// holding their numbers as byte-encoded differences.
///
/// Returns the next unused word number, or -1 on failure.
///
/// The run of word ends is collapsed to a single node as it goes: the tree
/// that gets written only needs one, since the numbers now live in the
/// buffer line rather than in the nodes. `line` is scratch the caller lends
/// for each line built.
fn sug_filltable(
    spin: &mut SpellInfo,
    node: Option<NodeId>,
    startwordnr: c_int,
    line: &mut Vec<u8>,
) -> c_int {
    let mut wordnr = startwordnr;
    let mut next = node;
    let spellbuf = spin.spellbuf();
    while let Some(p) = next {
        if spin.node(p).wn_byte != NUL as u8 {
            let child = spin.node(p).wn_child;
            wordnr = sug_filltable(spin, child, wordnr, line);
            if wordnr == -1 {
                return -1;
            }
            next = spin.node(p).wn_sibling;
            continue;
        }

        line.clear();
        let mut prev_nr = 0;
        let mut run = Some(p);
        while let Some(np) = run
            && spin.node(np).wn_byte == NUL as u8
        {
            let n = spin.node(np);
            let nr = (c_int::from(n.wn_flags) << 16) + (c_int::from(n.wn_region) & 0xffff);
            let mut bytes = [0u8; 4];
            let len = offset2bytes(nr - prev_nr, &mut bytes);
            prev_nr = nr;
            line.extend_from_slice(&bytes[..len]);
            run = n.wn_sibling;
        }
        line.push(0);

        let at = wordnr as LineNr;
        if ml_append_buf_terminated(spellbuf, at, line, true).is_err() {
            return -1;
        }
        wordnr += 1;

        // Drop the rest of the run and blank what is left, so the node
        // carries no word number into the file.
        while let Some(sib) = spin.node(p).wn_sibling
            && spin.node(sib).wn_byte == NUL as u8
        {
            spin.node_mut(p).wn_sibling = spin.node(sib).wn_sibling;
        }
        let n = spin.node_mut(p);
        n.wn_flags = 0;
        n.wn_region = 0;

        next = n.wn_sibling;
    }
    wordnr
}

/// Encode one word-number difference into one to four bytes.
///
/// The scheme mirrors UTF-8: the first byte's high bits say how many
/// follow. Every byte is biased by one so none of them can be zero, which
/// lets NUL terminate a list. Returns how many bytes were used.
fn offset2bytes(nr: c_int, buf: &mut [u8; 4]) -> usize {
    let b1 = nr % 255 + 1;
    let mut rem = nr / 255;
    let b2 = rem % 255 + 1;
    rem /= 255;
    let b3 = rem % 255 + 1;
    let b4 = rem / 255 + 1;

    if b4 > 1 || b3 > 0x1f {
        *buf = [(0xe0 + b4) as u8, b3 as u8, b2 as u8, b1 as u8];
        return 4;
    }
    if b3 > 1 || b2 > 0x3f {
        buf[..3].copy_from_slice(&[(0xc0 + b3) as u8, b2 as u8, b1 as u8]);
        return 3;
    }
    if b2 > 1 || b1 > 0x7f {
        buf[..2].copy_from_slice(&[(0x80 + b2) as u8, b1 as u8]);
        return 2;
    }
    buf[0] = b1 as u8;
    1
}

/// Write the `.sug` file: the sound-fold trie, then one line of word
/// numbers per word end.
fn sug_write(spin: &mut SpellInfo, fname: &CStr) {
    let path = Path::new(OsStr::from_bytes(fname.to_bytes()));
    let Ok(file) = File::create(path) else {
        let fname = msg_cstr(fname);
        semsg!("E484: Can't open file {fname}");
        return;
    };
    let name = fname.to_string_lossy();
    spell_message_fmt(spin, format_args!("Writing suggestion file {name}..."));

    let mut w = SplWriter::new(file);
    w.bytes(VIMSUGMAGIC.to_bytes());
    if !w.landed() {
        emsg(gettext(e_write));
        return;
    }
    w.byte(VIMSUGVERSION);
    // The same timestamp the `.spl` carries, so a stale pair is
    // detectable.
    w.bytes(&spin.si_sugtime.to_be_bytes());

    spin.si_memtot = 0;
    // The fold tree is built and compressed by now.
    let tree = spin.si_foldroot.and_then(|root| spin.node(root).wn_sibling);
    clear_node(spin, tree);
    let nodecount = put_node(None, spin, tree, 0, 0, false) as usize;
    w.u32(nodecount);
    debug_assert!(nodecount + nodecount * size_of::<c_int>() < c_int::MAX as usize);
    spin.si_memtot += (nodecount + nodecount * size_of::<c_int>()) as c_int;
    put_node(Some(&mut w), spin, tree, 0, 0, false);

    let spellbuf = spin.spellbuf();
    // The scratch buffer holds one line per word end.
    let wcount = spellbuf.line_count();
    debug_assert!(wcount >= 0);
    w.u32(wcount as usize);

    for lnum in 1..=wcount {
        // The stored terminator goes out with the line.
        let mut lines = spellbuf.lines();
        let line = lines.line_cstr(lnum, 0).to_bytes_with_nul();
        w.bytes(line);
        if !w.landed() {
            emsg(gettext(e_write));
            return;
        }
        spin.si_memtot += line.len() as c_int;
    }

    w.byte(0);
    if !w.finish() {
        emsg(gettext(e_write));
    }
    let used = spin.si_memtot;
    spell_message_fmt(
        spin,
        format_args!("Estimated runtime memory use: {used} bytes"),
    );
}
