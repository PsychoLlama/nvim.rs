//! Suggestions found by how the word sounds.
//!
//! The edit-distance search can only find words spelled nearly the same.
//! This one finds words *pronounced* nearly the same, which is what turns
//! "nashun" into "nation". It needs the language's `.sug` companion file:
//! a second word tree holding every sound-folded form the dictionary
//! produces, and, for each of them, the numbers of the real words that
//! fold to it.
//!
//! One pass is: sound-fold the bad word, walk the sound-fold tree with the
//! same trie walker the edit-distance search uses (so that inserts,
//! deletes and swaps of *sounds* are tried), and for every sound-folded
//! word it reaches call [`add_sound_suggest`] to turn that back into real
//! words. [`suggest_try_soundalike_prep`] and
//! [`suggest_try_soundalike_finish`] bracket it around a per-language
//! table of soundfolds already handled, because the same soundfold is
//! reached many times over and the work below is far too slow to repeat.
//!
//! # From a soundfold back to words
//!
//! The `.sug` file stores, per soundfold, the *word numbers* of the words
//! that produce it -- as increasing deltas, each in as few bytes as it
//! fits ([`bytes2offset`]). A word number is turned back into letters by
//! walking the case-folded tree and counting, at every branch, how many
//! words hang below the siblings passed over; that count is what the
//! reader stashed in the first index entry of each child.
//!
//! Words flagged `KEEPCAP` cannot be reconstructed that way -- the tree
//! walked is the case-folded one -- so [`find_keepcap_word`] searches the
//! keep-case tree for a word that folds to the same thing, trying each
//! character both folded and upper-case.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::hashtab::{hash_add_item, hash_hash, hash_lookup, hash_reset};
use crate::mbyte::{utf_ptr2char, utf_ptr2len};
use crate::memline::ml_get_buf;
use crate::memory::{xfree, xmalloc};
use crate::message::internal_error;
use crate::spell::WordFlags;
use crate::spell::{allcap_copy, make_case_word, spell_soundfold};
use crate::spellsuggest::collect::add_suggestion;
use crate::spellsuggest::score::{
    EMPTY_SOUND, score_wordcount_adj, spell_edit_score, spell_edit_score_limit, spell_isupper,
    spell_tofold,
};
use crate::spellsuggest::walk::suggest_trie_walk;
use crate::spellsuggest::{
    MAXWLEN, SCORE_ICASE, SCORE_LIMITMAX, SCORE_MAXMAX, SCORE_REGION, SPS_DOUBLE, TAB, sps_flags,
    suginfo_T,
};
use crate::types::{NUL, int16_t, langp_T, linenr_T, slang_T, uint8_t};
use core::ffi::{c_char, c_int, c_void};
use core::{mem, ptr};

/// The best score seen so far for one sound-folded word, with that word
/// stored inline after it. The hash table of soundfolds already handled
/// keys on the inline word.
#[repr(C)]
struct sftword_T {
    sft_score: int16_t,
    sft_word: [uint8_t; 0],
}

/// Where the inline soundfolded word sits inside a `sftword_T`; the hash
/// table keys on that field, so the record is recovered by stepping back.
const SFT_WORD_OFF: usize = mem::offset_of!(sftword_T, sft_word);

/// Does a language have both a sound-folding table and a loaded `.sug`?
/// Without both there is nothing to walk.
///
/// # Safety
///
/// `slang` must be a loaded language.
unsafe fn has_sound_tree(slang: *mut slang_T) -> bool {
    // SAFETY: the caller guarantees the language.
    unsafe { (*slang).has_soundfold() && !(*slang).sl_sound_tree.is_empty() }
}

/// Prepare the per-language table of soundfolds already handled.
///
/// # Safety
///
/// The current window must have its languages loaded.
pub(super) unsafe fn suggest_try_soundalike_prep() {
    // SAFETY: by the contract above, the window's languages are loaded.
    for lp in unsafe { crate::spellsuggest::window_langs() } {
        // SAFETY: `lp` came out of the window's language list, so
        // `lp_slang` is a loaded language -- what both calls here need.
        if unsafe { has_sound_tree(lp.lp_slang) } {
            // SAFETY: as above; `slang_alloc` set the table up.
            hash_reset(unsafe { &mut (*lp.lp_slang).sl_sounddone });
        }
    }
}

/// Find suggestions by comparing the bad word in sound-a-like form.
///
/// Postponed prefixes are not supported here.
///
/// # Safety
///
/// `su` must be valid and the current window must have its languages
/// loaded.
pub(super) unsafe fn suggest_try_soundalike(su: *mut suginfo_T) {
    // SAFETY: by the contract above, the window's languages are loaded.
    for lp in unsafe { crate::spellsuggest::window_langs() } {
        // SAFETY: `lp` came out of the window's language list, so
        // `lp_slang` is a loaded language.
        if !unsafe { has_sound_tree(lp.lp_slang) } {
            continue;
        }
        let mut salword = EMPTY_SOUND;
        // SAFETY: `su` is valid by the contract above, so the address of
        // its `su_fbadword` field is too.
        let fbadword = unsafe { &raw mut (*su).su_fbadword } as *mut c_char;
        // SAFETY: `fbadword` is the bad word's NUL-terminated fold and
        // `salword` has room for `MAXWLEN` bytes, which is what
        // `spell_soundfold` writes at most.
        unsafe { spell_soundfold(lp.lp_slang, fbadword, true, salword.as_mut_ptr()) };
        // The same walker as the edit-distance search, told to treat
        // the tree it walks as sounds rather than letters.
        //
        // SAFETY: `su` and `lp` are valid by the contract above and
        // `salword` is the NUL-terminated soundfold just written.
        unsafe { suggest_trie_walk(su, lp, salword.as_mut_ptr(), true) };
    }
}

/// Release the per-language table of soundfolds already handled.
///
/// # Safety
///
/// The current window must have its languages loaded.
pub(super) unsafe fn suggest_try_soundalike_finish() {
    // SAFETY: by the contract above, the window's languages are loaded.
    for lp in unsafe { crate::spellsuggest::window_langs() } {
        let slang = lp.lp_slang;
        // SAFETY: `slang` came out of the window's language list.
        if !unsafe { has_sound_tree(slang) } {
            continue;
        }

        // SAFETY: `slang` is a loaded language, so it carries a hash table.
        let done = unsafe { &mut (*slang).sl_sounddone };
        for hi in done.items() {
            // SAFETY: every key in this table is the inline word of a
            // `sftword_T` this module allocated, so stepping back by
            // `SFT_WORD_OFF` recovers that allocation's start.
            unsafe { xfree(hi.hi_key.sub(SFT_WORD_OFF) as *mut c_void) };
        }

        // Another region may reuse the table, so leave it empty rather
        // than freed.
        hash_reset(done);
    }
}

/// Read one word-number delta from a `.sug` line.
///
/// The numbers are stored in as few bytes as they fit, with the count in
/// the top bits of the first byte and every byte biased by one so that no
/// NUL can appear inside the line. Returns the delta and steps `pos` past
/// what it consumed.
fn bytes2offset(bytes: &[u8], pos: &mut usize) -> c_int {
    // A missing byte only happens on a damaged file; reading it as zero
    // keeps the walk inside the line where the C would have run on into
    // whatever followed it.
    let mut next = |pos: &mut usize| -> c_int {
        let b = bytes.get(*pos).copied().unwrap_or(0) as c_int;
        *pos += 1;
        b - 1
    };

    let c = bytes.get(*pos).copied().unwrap_or(0) as c_int;
    *pos += 1;
    let (mut nr, extra) = if c & 0x80 == 0 {
        (c - 1, 0)
    } else if c & 0xc0 == 0x80 {
        ((c & 0x3f) - 1, 1)
    } else if c & 0xe0 == 0xc0 {
        ((c & 0x1f) - 1, 2)
    } else {
        ((c & 0x0f) - 1, 3)
    };
    for _ in 0..extra {
        nr = nr * 255 + next(pos);
    }
    nr
}

/// A match with a sound-folded word was found: add the real word or words
/// that produce it.
///
/// `score` is the sound-a-like score the walk arrived at.
///
/// # Safety
///
/// `su` and `lp` must be valid and the language must have a loaded `.sug`.
pub(super) unsafe fn add_sound_suggest(
    su: *mut suginfo_T,
    goodword: *mut c_char,
    score: c_int,
    lp: *mut langp_T,
) {
    // SAFETY: `lp` is valid by the contract above.
    let slang = unsafe { (*lp).lp_slang };

    // The same soundfold turns up many times with different scores and
    // what follows is slow, so only the best score for each is done.
    //
    // SAFETY: `goodword` is a NUL-terminated word by the contract above,
    // and `slang` is a loaded language, so its `sl_sounddone` is a live
    // hash table -- the table `hash_add_item` is then told to insert into,
    // with the `hi`/`hash` pair `hash_lookup` just produced for it.
    let sounddone = unsafe { &raw mut (*slang).sl_sounddone };
    let hash = unsafe { hash_hash(goodword) };
    let goodword_len = unsafe { cstr::bytes_at(goodword).len() };
    let hi = unsafe { hash_lookup(sounddone, goodword, goodword_len, hash) };
    if !hi.is_kept() {
        // SAFETY: the allocation is `SFT_WORD_OFF + goodword_len + 1`
        // bytes, so the record's header and the word after it both fit,
        // and `goodword` really is `goodword_len + 1` bytes with its NUL.
        let sft = unsafe { xmalloc(SFT_WORD_OFF + goodword_len + 1) } as *mut sftword_T;
        unsafe { (*sft).sft_score = score as int16_t };
        let word = unsafe { (sft as *mut u8).add(SFT_WORD_OFF) };
        unsafe { ptr::copy_nonoverlapping(goodword as *const u8, word, goodword_len + 1) };
        unsafe { hash_add_item(sounddone, hi, word as *mut c_char, hash) };
    } else {
        // SAFETY: the key is the inline word of a `sftword_T` allocated
        // above, so stepping back by `SFT_WORD_OFF` recovers the record.
        let sft = unsafe { hi.hi_key.sub(SFT_WORD_OFF) } as *mut sftword_T;
        if score >= unsafe { (*sft).sft_score } as c_int {
            return;
        }
        unsafe { (*sft).sft_score = score as int16_t };
    }

    // SAFETY: the language has a loaded `.sug` by the contract above and
    // `goodword` is NUL-terminated.
    let sfwordnr = unsafe { soundfold_find(slang, goodword) };
    if sfwordnr < 0 {
        // SAFETY: the message is a NUL-terminated literal.
        unsafe { internal_error(c"add_sound_suggest()".as_ptr()) };
        return;
    }

    // Walk the list of word numbers that produce this soundfold.
    //
    // SAFETY: `sl_sugbuf` is the loaded `.sug` buffer and `soundfold_find`
    // returned a line number inside it; the line it hands back is a
    // NUL-terminated string owned by that buffer.
    let nrline = unsafe { ml_get_buf((*slang).sl_sugbuf, sfwordnr as linenr_T + 1) };
    let deltas = unsafe { core::ffi::CStr::from_ptr(nrline) }.to_bytes();
    let mut pos = 0;
    let mut orgnr = 0;

    while pos < deltas.len() {
        orgnr += bytes2offset(deltas, &mut pos);
        // SAFETY: `slang` has a case-folded tree, and the `n`/`i` pair the
        // walk returns is the tree position `emit_word` expects.
        let (mut theword, n, i) = unsafe { word_number_to_letters(slang, orgnr) };
        unsafe { emit_word(su, lp, slang, &mut theword, n, i, score) };
    }
}

/// Turn a word number back into its letters by walking the case-folded
/// tree, counting the words hanging below every sibling passed over.
///
/// Returns the word buffer, its length, the tree node the word ended at
/// and the sibling index to continue the flag scan from.
///
/// # Safety
///
/// `slang` must have a case-folded tree.
unsafe fn word_number_to_letters(
    slang: *mut slang_T,
    orgnr: c_int,
) -> ([c_char; MAXWLEN], usize, usize) {
    // SAFETY: the caller guarantees the tree, and it stays loaded for as
    // long as this walk.
    let tree = unsafe { (*slang).sl_fold_tree.view() };

    let mut theword = [0 as c_char; MAXWLEN];
    let mut n: usize = 0;
    let mut wordcount = 0;
    let mut wlen = 0;
    let mut i: usize = 1;

    while wlen < MAXWLEN - 3 {
        i = 1;
        if wordcount == orgnr && tree.ends_word(n + 1) {
            break; // found the end of the word
        }
        if tree.ends_word(n + 1) {
            wordcount += 1;
        }

        // Skip the NUL bytes; there can be several.
        let mut bad = false;
        while tree.ends_word(n + i) {
            if i > tree.node_len(n) {
                // Safety check: the tree disagrees with the count.
                theword[wlen..wlen + 3].copy_from_slice(&[
                    b'B' as c_char,
                    b'A' as c_char,
                    b'D' as c_char,
                ]);
                wlen += 3;
                bad = true;
                break;
            }
            i += 1;
        }
        if bad {
            break;
        }

        // One of the siblings has the word under it. The index each
        // sibling holds is a node start, and that node's own entry is the
        // number of words below it.
        while i < tree.node_len(n) {
            let wc = tree.idx(tree.child_node(n + i));
            if wordcount + wc > orgnr {
                break;
            }
            wordcount += wc;
            i += 1;
        }

        theword[wlen] = tree.byte(n + i) as c_char;
        n = tree.child_node(n + i);
        wlen += 1;
    }
    theword[wlen] = NUL as c_char;
    (theword, n, i)
}

/// Offer the word `theword` under every flag/region combination the tree
/// records for it.
///
/// # Safety
///
/// All pointers must be valid and `n`/`i` must come from
/// [`word_number_to_letters`].
#[allow(clippy::too_many_arguments)]
unsafe fn emit_word(
    su: *mut suginfo_T,
    lp: *mut langp_T,
    slang: *mut slang_T,
    theword: &mut [c_char; MAXWLEN],
    n: usize,
    mut i: usize,
    score: c_int,
) {
    // SAFETY: the caller guarantees the language, so it has a case-folded
    // tree, and it stays loaded for as long as this walk.
    let tree = unsafe { (*slang).sl_fold_tree.view() };

    // The flags and regions are the NUL-byte children of this node, so the
    // scan stops at the node's child count.
    while i <= tree.node_len(n) && tree.ends_word(n + i) {
        let mut cword = [0 as c_char; MAXWLEN];
        let mut flags = WordFlags::from_bits(tree.idx(n + i));
        i += 1;

        if flags.has(WordFlags::NOSUGGEST) {
            continue;
        }

        let p = if flags.has(WordFlags::KEEPCAP) {
            // The letters came out of the case-folded tree, so the
            // real spelling has to be looked up in the keep-case one.
            //
            // SAFETY: `theword` is NUL-terminated and `cword` has room for
            // `MAXWLEN` bytes, which is the most either call writes.
            unsafe { find_keepcap_word(slang, theword.as_mut_ptr(), cword.as_mut_ptr()) };
            cword.as_mut_ptr()
        } else {
            // SAFETY: `su` is valid by the contract above.
            flags |= unsafe { (*su).su_badflags };
            if flags.has(WordFlags::CAPMASK) {
                // SAFETY: as for `find_keepcap_word` above.
                unsafe { make_case_word(theword.as_mut_ptr(), cword.as_mut_ptr(), flags) };
                cword.as_mut_ptr()
            } else {
                theword.as_mut_ptr()
            }
        };

        if sps_flags.get() & SPS_DOUBLE != 0 {
            // SAFETY: `su` is valid by the contract above.
            if score <= unsafe { (*su).su_maxscore } {
                let sga = unsafe { &raw mut (*su).su_sga };
                let badlen = unsafe { (*su).su_badlen };
                // SAFETY: `su` and `slang` are valid, `sga` is one of
                // `su`'s own growarrays and `p` is a NUL-terminated word
                // in a buffer that outlives the call.
                unsafe { add_suggestion(su, sga, p, badlen, score, 0, false, slang, false) };
            }
            continue;
        }

        // A word from another region is worth less.
        //
        // SAFETY: `lp` is valid by the contract above.
        let mut goodscore = if flags.has(WordFlags::REGION)
            && (flags.bits() as u32 >> 16) & unsafe { (*lp).lp_region } as u32 == 0
        {
            SCORE_REGION
        } else {
            0
        };

        // A small penalty for turning the first letter from lower to
        // upper case: "tath" -> "Kath" is less likely than "tath" ->
        // "path". Not when the letter is the same, which is counted
        // already.
        //
        // SAFETY: `p` is a NUL-terminated word and `su` is valid, so its
        // `su_badword` is one too.
        let gc = unsafe { utf_ptr2char(p) };
        if spell_isupper(gc) {
            let bc = unsafe { utf_ptr2char(&raw const (*su).su_badword as *const c_char) };
            if !spell_isupper(bc) && spell_tofold(bc) != spell_tofold(gc) {
                goodscore += SCORE_ICASE / 2;
            }
        }

        // The edit distance for the good word. REP items are not
        // considered, which may leave the score a little high. A
        // ceiling makes it faster; past a high enough ceiling the
        // depth-first search costs more than filling the table.
        //
        // SAFETY: `su` is valid, so the address of its `su_badword` is a
        // NUL-terminated word, and `slang` is a loaded language.
        let badword = unsafe { &raw const (*su).su_badword } as *const c_char;
        let scored_lang = Some(unsafe { &*slang });
        let limit = (4 * (unsafe { (*su).su_sfmaxscore } - goodscore) - score) / 3;
        goodscore += if limit > SCORE_LIMITMAX {
            unsafe { spell_edit_score(scored_lang, badword, p) }
        } else {
            unsafe { spell_edit_score_limit(scored_lang, badword, p, limit) }
        };

        if goodscore >= SCORE_MAXMAX {
            continue;
        }

        // SAFETY: as above -- `p` is a NUL-terminated word.
        goodscore = unsafe { score_wordcount_adj(&*slang, goodscore, p, false) };
        goodscore = (3 * goodscore + score) / 4;
        // SAFETY: `su` is valid by the contract above.
        if goodscore <= unsafe { (*su).su_sfmaxscore } {
            let ga = unsafe { &raw mut (*su).su_ga };
            let badlen = unsafe { (*su).su_badlen };
            // SAFETY: as for the `su_sga` call above.
            unsafe { add_suggestion(su, ga, p, badlen, goodscore, score, true, slang, true) };
        }
    }
}

/// Find `word` in the sound-fold tree and return its word number, or -1
/// when it is not there.
///
/// # Safety
///
/// `slang` must have a loaded `.sug` and `word` must be NUL-terminated.
pub(super) unsafe fn soundfold_find(slang: *mut slang_T, word: *mut c_char) -> c_int {
    // SAFETY: the caller guarantees the loaded `.sug`, so the sound-fold
    // tree is there and stays loaded for as long as this walk.
    let tree = unsafe { (*slang).sl_sound_tree.view() };
    let ptr = word as *mut u8;

    let mut arridx: usize = 0;
    let mut wlen = 0;
    let mut wordnr = 0;

    // Every `ptr` read below is behind a NUL test on the byte before it,
    // so the walk stops at the word's terminator.
    loop {
        // The first byte of a node is how many bytes may follow.
        let mut len = tree.node_len(arridx);
        arridx += 1;

        // A leading zero byte means a word may end here.
        //
        // SAFETY: the caller's NUL-terminated word.
        let mut c = unsafe { *ptr.add(wlen) };
        if tree.ends_word(arridx) {
            if c == NUL as u8 {
                return wordnr;
            }

            // Skip the zeros; there can be several.
            let ends = tree.word_ends(arridx, len);
            arridx += ends;
            len -= ends;
            if len == 0 {
                return -1; // no children, the word should have ended
            }
            wordnr += 1;
        }

        if c == NUL as u8 {
            return -1; // the word ends but the tree does not
        }

        // Linear search over the accepted bytes, counting the words
        // hanging below the ones passed over. The index a sibling holds
        // is itself a node start, and that node's own entry is the
        // number of words below it.
        if c == TAB as u8 {
            c = b' '; // a tab counts as a space
        }
        while tree.byte(arridx) < c {
            wordnr += tree.idx(tree.child_node(arridx));
            arridx += 1;
            len -= 1;
            if len == 0 {
                return -1; // ran out of bytes without finding it
            }
        }
        if tree.byte(arridx) != c {
            return -1;
        }

        arridx = tree.child_node(arridx);
        wlen += 1;

        // One space in the good word may stand for several in the
        // word being checked.
        if c == b' ' {
            // SAFETY: as above -- the scan stops at the word's NUL.
            while unsafe { *ptr.add(wlen) } == b' ' || unsafe { *ptr.add(wlen) } == TAB as u8 {
                wlen += 1;
            }
        }
    }
}

/// One level of the keep-case search.
#[derive(Clone, Copy, Default)]
struct KeepCapLevel {
    /// Tree node this level starts from.
    arridx: usize,
    /// 0 before either case has been tried, 1 after folded, 2 after upper.
    round: c_int,
    /// How far into the folded and upper-case words this level is.
    fwordidx: usize,
    uwordidx: usize,
    /// How much of the answer has been written.
    kwordlen: usize,
}

/// Find the keep-case spelling of a case-folded word.
///
/// There could in theory be several keep-case words folding to the same
/// thing; this finds one of them. Each character is tried both folded and
/// upper-case, and changing case can change a character's byte length, so
/// the two words are tracked with separate offsets.
///
/// # Safety
///
/// `fword` must be NUL-terminated and `kword` must have room for
/// `MAXWLEN` bytes.
pub(super) unsafe fn find_keepcap_word(
    slang: *mut slang_T,
    fword: *mut c_char,
    kword: *mut c_char,
) {
    // SAFETY: the caller guarantees the language, and it stays loaded for
    // as long as this walk.
    let tree = unsafe { (*slang).sl_keep_tree.view() };
    if tree.is_empty() {
        // The tree is empty: cannot happen.
        //
        // SAFETY: `kword` has room for `MAXWLEN` bytes by the contract.
        unsafe { *kword = NUL as c_char };
        return;
    }

    let mut uword = [0 as c_char; MAXWLEN];
    // SAFETY: `fword` is NUL-terminated and `uword` has room for `MAXWLEN`
    // bytes, which bounds what the upper-cased copy can grow to.
    unsafe { allcap_copy(fword, uword.as_mut_ptr()) };

    let mut stack = [KeepCapLevel::default(); MAXWLEN];
    let mut depth: isize = 0;

    // Every level's offsets are byte counts already matched inside `fword`
    // and `uword`, which are both NUL-terminated; every `byts`/`idxs` index
    // is a node start the tree handed back, stepped forward at most that
    // node's child count. So all the reads below stay in bounds.
    while depth >= 0 {
        let level = stack[depth as usize];
        // SAFETY: as above.
        if unsafe { *fword.add(level.fwordidx) } == NUL as c_char {
            // At the end of the folded word: if the tree lets a word
            // end here, this is the answer.
            //
            // SAFETY: as above; `kwordlen` is what has been written into
            // `kword` so far, which is under `MAXWLEN`.
            if tree.ends_word(level.arridx + 1) {
                unsafe { *kword.add(level.kwordlen) = NUL as c_char };
                return;
            }
            // Otherwise the answer would be too long; back up.
            depth -= 1;
            continue;
        }

        stack[depth as usize].round += 1;
        let round = stack[depth as usize].round;
        if round > 2 {
            // Both cases tried; back up.
            depth -= 1;
            continue;
        }

        // SAFETY: as above -- both words are NUL-terminated and neither
        // offset has reached their terminator.
        let flen = unsafe { utf_ptr2len(fword.add(level.fwordidx)) } as usize;
        let ulen = unsafe { utf_ptr2len(uword.as_ptr().add(level.uwordidx)) } as usize;
        let (mut p, mut l) = if round == 1 {
            (unsafe { fword.add(level.fwordidx) } as *const u8, flen)
        } else {
            (
                unsafe { uword.as_ptr().add(level.uwordidx) } as *const u8,
                ulen,
            )
        };

        // Match the character's bytes one node at a time.
        let mut tryidx = level.arridx;
        while l > 0 {
            let len = tree.node_len(tryidx);
            tryidx += 1;
            // SAFETY: `l` bytes of the character are left, so `p` has that
            // many to read before it reaches the word's end.
            let c = unsafe { *p };
            p = unsafe { p.add(1) };

            let Some(at) = tree.child(tryidx, len, c) else {
                break;
            };
            tryidx = tree.child_node(at);
            l -= 1;
        }

        if l != 0 {
            continue;
        }

        // The whole character matched: keep it and go a level deeper.
        //
        // SAFETY: as above; `taken` is the character's length in whichever
        // word it came from, and `kword` has `MAXWLEN` bytes.
        let (src, taken) = if round == 1 {
            (unsafe { fword.add(level.fwordidx) } as *const c_char, flen)
        } else {
            (unsafe { uword.as_ptr().add(level.uwordidx) }, ulen)
        };
        unsafe { ptr::copy_nonoverlapping(src, kword.add(level.kwordlen), taken) };

        depth += 1;
        stack[depth as usize] = KeepCapLevel {
            arridx: tryidx,
            round: 0,
            fwordidx: level.fwordidx + flen,
            uwordidx: level.uwordidx + ulen,
            kwordlen: level.kwordlen + taken,
        };
    }

    // Not found: cannot happen.
    //
    // SAFETY: `kword` has room for `MAXWLEN` bytes by the contract.
    unsafe { *kword = NUL as c_char };
}
