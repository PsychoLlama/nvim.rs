//! Walking a whole word tree, for `:spelldump` and completion.
//!
//! [`spell_dump_compl`] does a depth-first walk over every word in every
//! loaded language and feeds each one to [`dump_word`]. It has two callers
//! and two modes:
//!
//! * `:spelldump` (`pat` null) writes each word as a line into a new
//!   buffer, in the same format `:mkspell` reads — with the `/` flags
//!   suffix and a `/regions=` header — so that a dictionary can be dumped,
//!   edited and compiled again. `:spelldump!` appends each word's `COMMON`
//!   count.
//! * Insert-mode completion `CTRL-X CTRL-K` (`pat` set) offers every word
//!   starting with what has been typed.
//!
//! # The walk
//!
//! `arridx`/`curi` are an explicit stack of "which node" and "which child
//! of it comes next", so the recursion the tree shape implies is a loop
//! that can be interrupted. Depth is capped at [`MAXWLEN`], and with a
//! pattern the walk is pruned as soon as the prefix built so far cannot
//! match it.
//!
//! Prefixes are not stored with the words they apply to, so a word
//! carrying a prefix ID has to be crossed with the prefix tree separately;
//! [`dump_prefixes`] does that walk.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::memory::xstrlcat;
use crate::spell::WordFlags;
use core::ffi::{c_char, c_int, c_uint};

use crate::buffer::buf_is_empty;
use crate::drawscreen::{UPD_NOT_VALID, redraw_later};
use crate::ex_docmd::do_cmdline_cmd;
use crate::hashtab::hash_find;
use crate::insexpand::{ins_compl_add_infercase, ins_compl_check_keys, ins_compl_interrupted};
use crate::main::{curbuf, curwin, got_int, p_ic};
use crate::mbyte::{mb_strnicmp, utf_ptr2char, utfc_ptr2len};
use crate::memline::{ml_append, ml_delete};
use crate::memory::xstrlcpy;
use crate::message::{msg_end, msg_ext_set_kind, msg_putchar, msg_puts, msg_start};
use crate::option::{get_option_value, optval_free, set_option_value_give_err};
use crate::options::{kOptSpell, kOptSpelllang};
use crate::os::cshim::snprintf;
use crate::os::input::line_breakcheck;
use crate::search::FORWARD;
use crate::strings::vim_snprintf;
use crate::types::{
    Direction, IOSIZE, NUL, OK, OptVal, OptionSetFlags, exarg_T, langp_T, linenr_T, size_t,
    slang_T, wordcount_T,
};

use super::chartab::{captype, make_case_word, onecap_copy, spell_toupper};
use super::check::no_spell_checking;
use super::lookup::valid_word_prefix;
use super::{MAXWLEN, WC_KEY_OFF};

/// Round 2 of the walk: the keep-case tree.
const DUMPFLAG_KEEPCASE: c_int = 1;
/// Append each word's `COMMON` count, for `:spelldump!`.
const DUMPFLAG_COUNT: c_int = 2;
/// Ignore case when matching against the completion pattern.
const DUMPFLAG_ICASE: c_int = 4;
/// The completion pattern starts with a capital.
const DUMPFLAG_ONECAP: c_int = 8;
/// The completion pattern is all capitals.
const DUMPFLAG_ALLCAP: c_int = 16;

/// `:spellinfo` — where each loaded language came from, and whatever its
/// `.spl` file recorded about itself.
pub unsafe fn ex_spellinfo(_eap: *mut exarg_T) {
    if unsafe { no_spell_checking(curwin.get()) } {
        return;
    }

    unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
    unsafe { msg_start() };
    // SAFETY: the current window and its syntax block are live.
    let langp = unsafe { &(*(*curwin.get()).w_s).b_langp };
    let mut lpi = 0;
    while lpi < langp.ga_len && !got_int.get() {
        let lp = unsafe { (langp.ga_data as *mut langp_T).offset(lpi as isize) };
        unsafe { msg_puts(c"file: ".as_ptr()) };
        unsafe { msg_puts((*(*lp).lp_slang).sl_fname) };
        let p = unsafe { (*(*lp).lp_slang).sl_info };
        if lpi < langp.ga_len || !p.is_null() {
            unsafe { msg_putchar('\n' as c_int) };
        }
        if !p.is_null() {
            unsafe { msg_puts(p) };
            if lpi < langp.ga_len - 1 {
                unsafe { msg_putchar('\n' as c_int) };
            }
        }
        lpi += 1;
    }
    unsafe { msg_end() };
}

/// `:spelldump` — open a new window holding every word of the current
/// `'spelllang'`, in `:mkspell` input format. With `!` each word gets its
/// `COMMON` count appended.
pub unsafe fn ex_spelldump(eap: *mut exarg_T) {
    if unsafe { no_spell_checking(curwin.get()) } {
        return;
    }
    let spl: OptVal = get_option_value(kOptSpelllang, OptionSetFlags::LOCAL);

    let _ = unsafe { do_cmdline_cmd(c"new".as_ptr()) };

    // Spell checking has to be on in the new window for the dump to
    // mean anything.
    set_option_value_give_err(kOptSpell, OptVal::Boolean(1), OptionSetFlags::LOCAL);
    set_option_value_give_err(kOptSpelllang, spl, OptionSetFlags::LOCAL);
    optval_free(spl);

    if !unsafe { buf_is_empty(curbuf.get()) } {
        return;
    }

    let dumpflags = if unsafe { (*eap).forceit } != 0 {
        DUMPFLAG_COUNT
    } else {
        0
    };
    let (pat, dir) = (core::ptr::null_mut(), core::ptr::null_mut());
    unsafe { spell_dump_compl(pat, 0, dir, dumpflags) };

    // Drop the empty line the new buffer started with.
    if unsafe { (*curbuf.get()).b_ml.ml_line_count } > 1 {
        let _ = unsafe { ml_delete((*curbuf.get()).b_ml.ml_line_count) };
    }
    unsafe { redraw_later(curwin.get(), UPD_NOT_VALID) };
}

/// Walk every word of every loaded language.
///
/// With `pat` null, dump them into the current buffer. Otherwise offer
/// those starting with `pat` as Insert-mode completions, honouring `ic`
/// and adding them in `dir` (which is set to `FORWARD` after the first, so
/// a `BACKWARD` request applies only once).
pub unsafe fn spell_dump_compl(
    pat: *mut c_char,
    ic: c_int,
    dir: *mut Direction,
    dumpflags_arg: c_int,
) {
    let mut header = [0 as c_char; IOSIZE as usize];
    let mut arridx = [0usize; MAXWLEN];
    let mut curi = [0usize; MAXWLEN];
    let mut word = [0 as c_char; MAXWLEN];
    let mut lnum: linenr_T = 0;
    let mut region_names: *mut c_char = core::ptr::null_mut();
    let mut do_region = true;
    let mut dumpflags = dumpflags_arg;

    // Case handling of the pattern is dump_word()'s problem, but what
    // kind of handling it is gets decided once, here.
    if !pat.is_null() {
        if ic != 0 {
            dumpflags |= DUMPFLAG_ICASE;
        } else {
            let n = unsafe { captype(pat, core::ptr::null()) };
            if n == WordFlags::ONECAP {
                dumpflags |= DUMPFLAG_ONECAP;
            } else if n == WordFlags::ALLCAP
                && unsafe { cstr::bytes_at(pat) }.len() as c_int > unsafe { utfc_ptr2len(pat) }
            {
                dumpflags |= DUMPFLAG_ALLCAP;
            }
        }
    }

    // Regions can only be dumped when every language agrees on them.
    let langp_data = unsafe { (*(*curwin.get()).w_s).b_langp.ga_data } as *mut langp_T;
    let langp_len = unsafe { (*(*curwin.get()).w_s).b_langp.ga_len };
    for lpi in 0..langp_len {
        let lp = unsafe { langp_data.offset(lpi as isize) };
        let p = unsafe { (*(*lp).lp_slang).sl_regions.as_mut_ptr() };
        if unsafe { *p } != 0 {
            if region_names.is_null() {
                region_names = p;
            } else if !unsafe { cstr::eq(region_names, p) } {
                do_region = false;
                break;
            }
        }
    }

    if do_region && !region_names.is_null() && pat.is_null() {
        let (buf, size) = (header.as_mut_ptr(), IOSIZE as size_t);
        let fmt = c"/regions=%s".as_ptr();
        unsafe { vim_snprintf(buf, size, fmt, region_names) };
        let _ = unsafe { ml_append(lnum, header.as_mut_ptr(), 0, false) };
        lnum += 1;
    } else {
        do_region = false;
    }

    for lpi in 0..langp_len {
        let lp = unsafe { langp_data.offset(lpi as isize) };
        let slang = unsafe { (*lp).lp_slang };
        if unsafe { (*slang).sl_fold_tree.is_empty() } {
            continue; // reloading this language failed
        }

        if pat.is_null() {
            let (buf, size) = (header.as_mut_ptr(), IOSIZE as size_t);
            let fmt = c"# file: %s".as_ptr();
            let fname = unsafe { (*slang).sl_fname };
            unsafe { vim_snprintf(buf, size, fmt, fname) };
            let _ = unsafe { ml_append(lnum, header.as_mut_ptr(), 0, false) };
            lnum += 1;
        }

        // Without prefixes, a pattern can prune the walk; with them, a
        // prefix could still make a non-matching branch match.
        let patlen = if !pat.is_null() && unsafe { (*slang).sl_prefix_tree.is_empty() } {
            unsafe { cstr::bytes_at(pat).len() as c_int }
        } else {
            -1
        };

        // Round 1 is the case-folded tree, round 2 the keep-case one.
        for round in 1..=2 {
            let tree = if round == 1 {
                dumpflags &= !DUMPFLAG_KEEPCASE;
                unsafe { &(*slang).sl_fold_tree }
            } else {
                dumpflags |= DUMPFLAG_KEEPCASE;
                unsafe { &(*slang).sl_keep_tree }
            };
            if tree.is_empty() {
                continue; // this tree is empty
            }

            let mut depth: isize = 0;
            arridx[0] = 0;
            curi[0] = 1;
            while depth >= 0 && !got_int.get() && (pat.is_null() || !ins_compl_interrupted()) {
                let d = depth as usize;
                if curi[d] > tree.node_len(arridx[d]) {
                    // Every child of this node is done.
                    depth -= 1;
                    line_breakcheck();
                    unsafe { ins_compl_check_keys(50, false) };
                    continue;
                }

                let n = arridx[d] + curi[d];
                curi[d] += 1;
                let mut c = c_int::from(tree.byte(n));
                if c == 0 || d >= MAXWLEN - 1 {
                    // A word ends here, or the depth limit was hit.
                    // Keep-case words are skipped in the fold-case tree
                    // — they show up in the keep-case one — and words
                    // for other regions are skipped entirely.
                    let mut flags = WordFlags::from_bits(tree.idx(n));
                    if (round == 2 || !flags.has(WordFlags::KEEPCAP))
                        && !flags.has(WordFlags::NEEDCOMP)
                        && (do_region
                            || !flags.has(WordFlags::REGION)
                            || (flags.bits() as c_uint >> 16)
                                & unsafe { (*lp).lp_region } as c_uint
                                != 0)
                    {
                        word[d] = NUL as c_char;
                        if !do_region {
                            flags.clear(WordFlags::REGION);
                        }

                        // Dump the bare word when it has no prefix, or
                        // when this is the first of its prefixes.
                        c = (flags.bits() as c_uint >> 24) as c_int;
                        if c == 0 || curi[d] == 2 {
                            let w = word.as_mut_ptr();
                            unsafe { dump_word(slang, w, pat, dir, dumpflags, flags, lnum) };
                            if pat.is_null() {
                                lnum += 1;
                            }
                        }

                        if c != 0 {
                            let w = word.as_mut_ptr();
                            lnum = unsafe {
                                dump_prefixes(slang, w, pat, dir, dumpflags, flags, lnum)
                            };
                        }
                    }
                } else {
                    // An ordinary byte: descend.
                    word[d] = c as c_char;
                    depth += 1;
                    arridx[depth as usize] = tree.child_node(n);
                    curi[depth as usize] = 1;

                    // Prune a branch that cannot match the pattern.
                    // Case is always ignored here; dump_word() checks it
                    // properly later. That is not exact when folding
                    // changes a multi-byte character's length.
                    if depth <= patlen as isize
                        && unsafe { mb_strnicmp(word.as_ptr(), pat, depth as size_t) } != 0
                    {
                        depth -= 1;
                    }
                }
            }
        }
    }
}

/// Emit one word: apply the case its flags call for, then either append a
/// buffer line or offer it as a completion.
///
/// When dumping, flags that `:mkspell` would need are written after a `/`:
/// `=` to keep the case as written, `!` for banned, `?` for rare, and the
/// region numbers.
unsafe fn dump_word(
    slang: *mut slang_T,
    word: *mut c_char,
    pat: *mut c_char,
    dir: *mut Direction,
    dumpflags: c_int,
    wordflags: WordFlags,
    lnum: linenr_T,
) {
    let mut counted = [0 as c_char; IOSIZE as usize];
    let mut keepcap = false;
    let mut cword = [0 as c_char; MAXWLEN];
    let mut badword = [0 as c_char; MAXWLEN + 10];
    let mut flags = wordflags;

    if dumpflags & DUMPFLAG_ONECAP != 0 {
        flags |= WordFlags::ONECAP;
    }
    if dumpflags & DUMPFLAG_ALLCAP != 0 {
        flags |= WordFlags::ALLCAP;
    }

    let mut p;
    if dumpflags & DUMPFLAG_KEEPCASE == 0 && flags.has(WordFlags::CAPMASK) {
        unsafe { make_case_word(word, cword.as_mut_ptr(), flags) };
        p = cword.as_mut_ptr();
    } else {
        p = word;
        if dumpflags & DUMPFLAG_KEEPCASE != 0
            && (!unsafe { captype(word, core::ptr::null()) }.has(WordFlags::KEEPCAP)
                || flags.has(WordFlags::FIXCAP))
        {
            keepcap = true;
        }
    }
    let tw = p;

    if pat.is_null() {
        if flags.has(WordFlags::BANNED | WordFlags::RARE | WordFlags::REGION) || keepcap {
            let room = badword.len();
            unsafe { xstrlcpy(badword.as_mut_ptr(), p, room) };
            unsafe { xstrlcat(badword.as_mut_ptr(), c"/".as_ptr(), room) };
            if keepcap {
                unsafe { xstrlcat(badword.as_mut_ptr(), c"=".as_ptr(), room) };
            }
            if flags.has(WordFlags::BANNED) {
                unsafe { xstrlcat(badword.as_mut_ptr(), c"!".as_ptr(), room) };
            } else if flags.has(WordFlags::RARE) {
                unsafe { xstrlcat(badword.as_mut_ptr(), c"?".as_ptr(), room) };
            }
            if flags.has(WordFlags::REGION) {
                for i in 0..7 {
                    if flags.has(WordFlags::from_bits(0x10000 << i)) {
                        let badword_len = unsafe { cstr::bytes_at(badword.as_ptr()) }.len();
                        let room = badword.len() - badword_len;
                        let at = unsafe { badword.as_mut_ptr().add(badword_len) };
                        unsafe { snprintf(at, room, c"%d".as_ptr(), i + 1) };
                    }
                }
            }
            p = badword.as_mut_ptr();
        }

        if dumpflags & DUMPFLAG_COUNT != 0 {
            // ":spelldump!" wants the word's COMMON count.
            let hi = unsafe { hash_find(&raw mut (*slang).sl_wordcount, tw) };
            if hi.is_kept() {
                let wc = unsafe { hi.hi_key.offset(-(WC_KEY_OFF as isize)) } as *mut wordcount_T;
                let (buf, size) = (counted.as_mut_ptr(), IOSIZE as size_t);
                let fmt = c"%s\t%d".as_ptr();
                let count = unsafe { (*wc).wc_count } as c_int;
                unsafe { vim_snprintf(buf, size, fmt, tw, count) };
                p = counted.as_mut_ptr();
            }
        }

        let _ = unsafe { ml_append(lnum, p, 0, false) };
    } else {
        let matches = if dumpflags & DUMPFLAG_ICASE != 0 {
            unsafe { mb_strnicmp(p, pat, cstr::bytes_at(pat).len()) == 0 }
        } else {
            unsafe { cstr::starts_with(p, cstr::bytes_at(pat)) }
        };
        let len = unsafe { cstr::bytes_at(p) }.len() as c_int;
        let ic = p_ic.get() != 0;
        let want = unsafe { *dir };
        let none = core::ptr::null_mut();
        if matches && unsafe { ins_compl_add_infercase(p, len, ic, none, want, false, 0) } == OK {
            // A BACKWARD request is honoured only for the first match.
            unsafe { *dir = FORWARD };
        }
    }
}

/// Cross `word` with every prefix that accepts it, emitting each
/// combination, and return the line number reached.
///
/// A prefix with a condition is also tried against the word with its first
/// letter upper-cased, which is how "Un-" style prefixes reach words that
/// are stored lower-case.
unsafe fn dump_prefixes(
    slang: *mut slang_T,
    word: *mut c_char,
    pat: *mut c_char,
    dir: *mut Direction,
    dumpflags: c_int,
    flags: WordFlags,
    startlnum: linenr_T,
) -> linenr_T {
    let mut arridx = [0usize; MAXWLEN];
    let mut curi = [0usize; MAXWLEN];
    let mut prefix = [0 as c_char; MAXWLEN];
    let mut word_up = [0 as c_char; MAXWLEN];
    let mut has_word_up = false;
    let mut lnum = startlnum;

    // A word starting lower-case gets an upper-case twin to try.
    let c = unsafe { utf_ptr2char(word) };
    if spell_toupper(c) != c {
        unsafe { onecap_copy(word, word_up.as_mut_ptr(), true) };
        has_word_up = true;
    }

    let tree = unsafe { &(*slang).sl_prefix_tree };
    if tree.is_empty() {
        return lnum;
    }

    // Build each prefix byte by byte in prefix[]; at the end of one,
    // check whether it accepts "flags".
    let mut depth: isize = 0;
    arridx[0] = 0;
    curi[0] = 1;
    while depth >= 0 && !got_int.get() {
        let d = depth as usize;
        let mut n = arridx[d];
        let len = tree.node_len(n);
        if curi[d] > len {
            depth -= 1;
            line_breakcheck();
            continue;
        }

        n += curi[d];
        curi[d] += 1;
        if tree.ends_word(n) {
            // End of a prefix; count how many IDs share it. `n` is the
            // first of them, and the node's remaining children bound the
            // scan.
            let i = tree.word_ends(n, len + arridx[d] + 1 - n);
            curi[d] += i - 1;

            let c = unsafe { valid_word_prefix(i as c_int, n, flags, word, slang, false) };
            if c != 0 {
                let at = unsafe { prefix.as_mut_ptr().add(d) };
                unsafe { xstrlcpy(at, word, MAXWLEN - d) };
                let pfx_flags = if WordFlags::from_bits(c).has(WordFlags::RAREPFX) {
                    flags | WordFlags::RARE
                } else {
                    flags
                };
                let w = prefix.as_mut_ptr();
                unsafe { dump_word(slang, w, pat, dir, dumpflags, pfx_flags, lnum) };
                if lnum != 0 {
                    lnum += 1;
                }
            }

            // The same, for the upper-cased word — but only prefixes
            // that carry a condition.
            if has_word_up {
                let up = word_up.as_mut_ptr();
                let c = unsafe { valid_word_prefix(i as c_int, n, flags, up, slang, true) };
                if c != 0 {
                    let at = unsafe { prefix.as_mut_ptr().add(d) };
                    let up = word_up.as_mut_ptr();
                    unsafe { xstrlcpy(at, up, MAXWLEN - d) };
                    let pfx_flags = if WordFlags::from_bits(c).has(WordFlags::RAREPFX) {
                        flags | WordFlags::RARE
                    } else {
                        flags
                    };
                    let w = prefix.as_mut_ptr();
                    unsafe { dump_word(slang, w, pat, dir, dumpflags, pfx_flags, lnum) };
                    if lnum != 0 {
                        lnum += 1;
                    }
                }
            }
        } else {
            // An ordinary byte: descend.
            prefix[d] = tree.byte(n) as c_char;
            depth += 1;
            arridx[depth as usize] = tree.child_node(n);
            curi[depth as usize] = 1;
        }
    }

    lnum
}
