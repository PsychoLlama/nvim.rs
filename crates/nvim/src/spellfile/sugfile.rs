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

#![deny(unsafe_op_in_unsafe_fn)]

use crate::{semsg_c, smsg_c};
use core::ffi::{c_char, c_int, c_uint};

use crate::fileio::{put_bytes, put_time};
use crate::garray::{ga_clear, ga_grow, ga_init};
use crate::main::{IObuff, e_notopen, e_write, got_int};
use crate::memline::{ml_append_buf, ml_get_buf, ml_get_buf_len};
use crate::memory::{xfree, xmalloc, xstrlcpy};
use crate::message::emsg;
use crate::os::cshim::{gettext, putc};
use crate::os::fs::os_fopen;
use crate::os::input::line_breakcheck;
use crate::path::path_full_compare;
use crate::spell::{close_spellbuf, first_lang, open_spellbuf, slang_free, spell_soundfold};
use crate::strings::vim_snprintf;
use crate::types::{
    FILE, IOSIZE, MAXPATHL, NUL, colnr_T, garray_T, idx_T, int16_t, linenr_T, size_t, slang_T,
    uint8_t, uint16_t, uintmax_t,
};
use ::libc::{fclose, fwrite, strlen};

use super::wordtree::{tree_add_word, wordnode_T, wordtree_alloc, wordtree_compress};
use super::write::{clear_node, put_node};
use super::{
    EOF, FAIL, MAXWLEN, OK, VIMSUGMAGIC, VIMSUGMAGICL, VIMSUGVERSION, kEqualFiles, spell_load_file,
    spell_message, spellinfo_T,
};

/// Read the just-written `.spl` back and turn it into a `.sug`.
///
/// The `.spl` is re-read rather than reusing what is still in memory: the
/// word numbers have to match the order the *reader* will see, and the
/// tree in memory has been compressed since.
///
/// # Safety
///
/// `wfname` must be the NUL-terminated path of a readable `.spl`.
pub(super) unsafe fn spell_make_sugfile(spin: &mut spellinfo_T, wfname: *mut c_char) {
    // SAFETY: `wfname` is a valid path and every pointer below is either
    // from `spin` or from the language just loaded.
    unsafe {
        // Prefer an already-loaded copy of this file.
        let mut slang = first_lang.get();
        while !slang.is_null() {
            if path_full_compare(wfname, (*slang).sl_fname, false, true) as c_uint
                == kEqualFiles as c_uint
            {
                break;
            }
            slang = (*slang).sl_next;
        }
        let free_slang = slang.is_null();
        if free_slang {
            spell_message(spin, gettext(c"Reading back spell file...".as_ptr()));
            slang = spell_load_file(wfname, core::ptr::null_mut(), core::ptr::null_mut(), false);
            if slang.is_null() {
                return;
            }
        }

        // Start a fresh arena and tree: the ones the `.spl` was built from
        // have already been released.
        spin.si_arena.clear();
        spin.si_compress_cnt = 0;
        spin.si_free_count = 0;
        spin.si_first_free = core::ptr::null_mut();
        spin.si_foldwcount = 0;

        spell_message(spin, gettext(c"Performing soundfolding...".as_ptr()));
        let mut fname: *mut c_char = core::ptr::null_mut();
        if sug_filltree(spin, slang) != FAIL && sug_maketable(spin) != FAIL {
            smsg_c!(
                0,
                gettext(c"Number of words after soundfolding: %ld".as_ptr()),
                (*spin.si_spellbuf).b_ml.ml_line_count as i64,
            );
            spell_message(spin, gettext(super::wordtree::MSG_COMPRESSING.as_ptr()));
            let foldroot = spin.si_foldroot;
            wordtree_compress(spin, foldroot, c"case-folded");

            // Same path as the `.spl`, with the extension's last two
            // letters swapped: "spl" becomes "sug".
            fname = xmalloc(MAXPATHL as size_t).cast::<c_char>();
            xstrlcpy(fname, wfname, MAXPATHL as size_t);
            let len = strlen(fname) as isize;
            *fname.offset(len - 2) = b'u' as c_char;
            *fname.offset(len - 1) = b'g' as c_char;
            sug_write(spin, fname);
        }

        xfree(fname.cast());
        if free_slang {
            slang_free(slang);
        }
        spin.si_arena.clear();
        close_spellbuf(spin.si_spellbuf);
    }
}

/// Walk the loaded `.spl`'s case-folded tree and add every word's
/// sound-folded form to a new tree, numbering words as they are met.
///
/// The walk doubles as a rewrite of the loaded tree: `sl_fidxs` for a word
/// end is overwritten with the number of words in that sub-tree, which is
/// what the suggestion search will later use to turn a position in the tree
/// into a word number.
///
/// # Safety
///
/// `slang` must be a fully loaded language.
unsafe fn sug_filltree(spin: &mut spellinfo_T, slang: *mut slang_T) -> c_int {
    // SAFETY: the caller promises a loaded language; the walk is bounded by
    // the byte counts the tree itself carries, and depth by MAXWLEN, which
    // is the longest word the tree can hold.
    unsafe {
        let mut arridx: [idx_T; MAXWLEN] = [0; MAXWLEN];
        let mut curi: [c_int; MAXWLEN] = [0; MAXWLEN];
        let mut tword: [c_char; MAXWLEN] = [0; MAXWLEN];
        let mut tsalword: [c_char; MAXWLEN] = [0; MAXWLEN];
        let mut wordcount: [c_int; MAXWLEN] = [0; MAXWLEN];
        let mut words_done: c_uint = 0;

        spin.si_foldroot = wordtree_alloc(spin);
        spin.si_sugtree = 1;

        let byts = (*slang).sl_fbyts;
        let idxs = (*slang).sl_fidxs;
        if byts.is_null() || idxs.is_null() {
            return FAIL;
        }

        arridx[0] = 0;
        curi[0] = 1;
        wordcount[0] = 0;
        let mut depth: usize = 0;
        loop {
            if got_int.get() {
                break;
            }
            if curi[depth] > *byts.offset(arridx[depth] as isize) as c_int {
                // Done with this node: publish the sub-tree's word count
                // where its index used to be, and fold it into the parent.
                *idxs.offset(arridx[depth] as isize) = wordcount[depth] as idx_T;
                let at_root = depth == 0;
                if !at_root {
                    wordcount[depth - 1] += wordcount[depth];
                    depth -= 1;
                }
                line_breakcheck();
                if at_root {
                    break;
                }
                continue;
            }

            let mut n: idx_T = arridx[depth] + curi[depth] as idx_T;
            curi[depth] += 1;
            let c = *byts.offset(n as isize) as c_int;
            if c != 0 {
                // Descend one byte.
                tword[depth] = c as uint8_t as c_char;
                depth += 1;
                arridx[depth] = *idxs.offset(n as isize);
                curi[depth] = 1;
                wordcount[depth] = 0;
                continue;
            }

            // A word end. Its number goes in through the flags and region
            // arguments, split across the two.
            tword[depth] = NUL as c_char;
            spell_soundfold(slang, tword.as_mut_ptr(), true, tsalword.as_mut_ptr());
            let foldroot = spin.si_foldroot;
            if tree_add_word(
                spin,
                tsalword.as_ptr(),
                foldroot,
                (words_done >> 16) as c_int,
                (words_done & 0xffff) as c_int,
                0,
            ) == FAIL
            {
                return FAIL;
            }
            words_done = words_done.wrapping_add(1);
            wordcount[depth] += 1;

            // Compressing here would renumber what has already been
            // written, so hold it off for the whole walk.
            spin.si_arena.block_count = 0;

            // One word may end several times over, once per flag set; they
            // are the same word for this purpose.
            while (n as c_int + 1) < (*slang).sl_fbyts_len
                && *byts.offset(n as isize + 1) as c_int == 0
            {
                n += 1;
                curi[depth] += 1;
            }
        }

        smsg_c!(
            0,
            gettext(c"Total number of words: %d".as_ptr()),
            words_done,
        );
        OK
    }
}

/// Collect each word end's word numbers into one line of a scratch buffer.
unsafe fn sug_maketable(spin: &mut spellinfo_T) -> c_int {
    // SAFETY: the sound-fold tree is built and compressed by now.
    unsafe {
        spin.si_spellbuf = open_spellbuf();

        let mut ga: garray_T = core::mem::zeroed();
        ga_init(&raw mut ga, 1, 100);
        let root = (*spin.si_foldroot).wn_sibling;
        let res = if sug_filltable(spin, root, 0, &raw mut ga) == -1 {
            FAIL
        } else {
            OK
        };
        ga_clear(&raw mut ga);
        res
    }
}

/// Walk the sound-fold tree, and for each run of word ends write one line
/// holding their numbers as byte-encoded differences.
///
/// Returns the next unused word number, or -1 on failure.
///
/// The run of word ends is collapsed to a single node as it goes: the tree
/// that gets written only needs one, since the numbers now live in the
/// buffer line rather than in the nodes.
///
/// # Safety
///
/// `node` must be null or head a live sibling chain of the sound-fold tree,
/// and `gap` an initialised one-byte-item garray.
unsafe fn sug_filltable(
    spin: &mut spellinfo_T,
    node: *mut wordnode_T,
    startwordnr: c_int,
    gap: *mut garray_T,
) -> c_int {
    // SAFETY: the caller promises the chain and the garray; `ga_grow(10)`
    // covers the at-most-five bytes each iteration appends.
    unsafe {
        let mut wordnr = startwordnr;
        let mut p = node;
        while !p.is_null() {
            if (*p).wn_byte as c_int != NUL {
                wordnr = sug_filltable(spin, (*p).wn_child, wordnr, gap);
                if wordnr == -1 {
                    return -1;
                }
                p = (*p).wn_sibling;
                continue;
            }

            (*gap).ga_len = 0;
            let mut prev_nr = 0;
            let mut np = p;
            while !np.is_null() && (*np).wn_byte as c_int == NUL {
                ga_grow(gap, 10);
                let nr = (((*np).wn_flags as c_int) << 16) + ((*np).wn_region as c_int & 0xffff);
                let mut bytes = [0u8; 4];
                let n = offset2bytes(nr - prev_nr, &mut bytes);
                prev_nr = nr;
                let dest = (*gap).ga_data.cast::<u8>().offset((*gap).ga_len as isize);
                core::ptr::copy_nonoverlapping(bytes.as_ptr(), dest, n);
                (*gap).ga_len += n as c_int;
                np = (*np).wn_sibling;
            }
            *(*gap)
                .ga_data
                .cast::<c_char>()
                .offset((*gap).ga_len as isize) = NUL as c_char;
            (*gap).ga_len += 1;

            if ml_append_buf(
                spin.si_spellbuf,
                wordnr as linenr_T,
                (*gap).ga_data.cast::<c_char>(),
                (*gap).ga_len as colnr_T,
                true,
            ) == FAIL
            {
                return -1;
            }
            wordnr += 1;

            // Drop the rest of the run and blank what is left, so the node
            // carries no word number into the file.
            while !(*p).wn_sibling.is_null() && (*(*p).wn_sibling).wn_byte as c_int == NUL {
                (*p).wn_sibling = (*(*p).wn_sibling).wn_sibling;
            }
            (*p).wn_flags = 0 as uint16_t;
            (*p).wn_region = 0 as int16_t;

            p = (*p).wn_sibling;
        }
        wordnr
    }
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
///
/// # Safety
///
/// `fname` must be a NUL-terminated path.
unsafe fn sug_write(spin: &mut spellinfo_T, fname: *mut c_char) {
    // SAFETY: `fname` is a valid path; the tree and the scratch buffer are
    // both built by now.
    unsafe {
        let fd = os_fopen(fname, c"w".as_ptr());
        if fd.is_null() {
            semsg_c!(gettext((&raw const e_notopen).cast()), fname);
            return;
        }
        vim_snprintf(
            IObuff.ptr().cast::<c_char>(),
            IOSIZE as size_t,
            gettext(c"Writing suggestion file %s...".as_ptr()),
            fname,
        );
        spell_message(spin, IObuff.ptr().cast::<c_char>());

        if fwrite(VIMSUGMAGIC.as_ptr().cast(), VIMSUGMAGICL as size_t, 1, fd) != 1 {
            emsg(gettext((&raw const e_write).cast()));
            fclose(fd);
            return;
        }
        putc(VIMSUGVERSION, fd);
        // The same timestamp the `.spl` carries, so a stale pair is
        // detectable.
        put_time(fd, spin.si_sugtime);

        spin.si_memtot = 0;
        let tree = (*spin.si_foldroot).wn_sibling;
        clear_node(tree);
        let nodecount = put_node(core::ptr::null_mut::<FILE>(), tree, 0, 0, false) as usize;
        put_bytes(fd, nodecount as uintmax_t, 4);
        debug_assert!(nodecount + nodecount * size_of::<c_int>() < c_int::MAX as usize);
        spin.si_memtot += (nodecount + nodecount * size_of::<c_int>()) as c_int;
        put_node(fd, tree, 0, 0, false);

        let wcount = (*spin.si_spellbuf).b_ml.ml_line_count;
        debug_assert!(wcount >= 0);
        put_bytes(fd, wcount as uintmax_t, 4);

        let mut failed = false;
        for lnum in 1..=wcount {
            let line = ml_get_buf(spin.si_spellbuf, lnum);
            // The stored terminator goes out with the line.
            let len = ml_get_buf_len(spin.si_spellbuf, lnum) + 1;
            if fwrite(line.cast(), len as size_t, 1, fd) == 0 {
                emsg(gettext((&raw const e_write).cast()));
                failed = true;
                break;
            }
            spin.si_memtot += len;
        }

        if !failed {
            if putc(0, fd) == EOF {
                emsg(gettext((&raw const e_write).cast()));
            }
            vim_snprintf(
                IObuff.ptr().cast::<c_char>(),
                IOSIZE as size_t,
                gettext(c"Estimated runtime memory use: %d bytes".as_ptr()),
                spin.si_memtot,
            );
            spell_message(spin, IObuff.ptr().cast::<c_char>());
        }
        fclose(fd);
    }
}
