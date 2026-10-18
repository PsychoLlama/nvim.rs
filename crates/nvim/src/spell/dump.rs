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

#![forbid(unsafe_code)]

use crate::cstr::until_nul;
use crate::spell::WordFlags;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_int, c_uint};

use crate::buffer::buf_is_empty;
use crate::drawscreen::{UPD_NOT_VALID, redraw_later};
use crate::ex_docmd::do_cmdline_cmd;
use crate::getchar::state::got_int;
use crate::insexpand::{ins_compl_add_infercase, ins_compl_check_keys, ins_compl_interrupted};
use crate::mbyte::{c_cluster_len, char_at, strnicmp_in};
use crate::memline::{ml_append_text, ml_delete};
use crate::memory::XString;
use crate::message::{msg_end, msg_ext_set_kind, msg_putchar, msg_start, msg_str};
use crate::option::vars::p_ic;
use crate::option::{boolean_optval, get_option_value, set_option_value_give_err};
use crate::options::{kOptSpell, kOptSpelllang};
use crate::os::input::line_breakcheck;
use crate::search::FORWARD;
use crate::types::{Direction, ExArg, IOSIZE, LineNr, OK, OptVal, OptionSetFlags, SpellLang};

use super::chartab::{captype, make_case_word, onecap_copy, spell_toupper};
use super::check::no_spell_checking;
use super::lookup::valid_word_prefix;
use super::{MAXWLEN, loaded_langs};

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

/// At most `n` bytes of the C string `s`.
fn prefix(s: &[u8], n: usize) -> &[u8] {
    let s = until_nul(s);
    &s[..n.min(s.len())]
}

/// `text` cut as a `snprintf` into a buffer of `IOSIZE` would leave it.
fn io_line(mut text: Vec<u8>) -> Vec<u8> {
    text.truncate(IOSIZE as usize - 1);
    text
}

/// `:spellinfo` — where each loaded language came from, and whatever its
/// `.spl` file recorded about itself.
pub fn ex_spellinfo(_excmd: &mut ExArg) {
    if no_spell_checking(Win::current()) {
        return;
    }

    // Copied out first: the messages may scroll and redraw.
    let entries: Vec<(XString, Option<XString>)> = loaded_langs.with(|langs| {
        Win::current()
            .syn_block()
            .b_langp
            .iter()
            .filter_map(|lp| {
                let slang = langs.get(lp.slang)?;
                Some((slang.sl_fname.clone(), slang.sl_info.clone()))
            })
            .collect()
    });

    msg_ext_set_kind(c"list_cmd");
    msg_start();
    let count = entries.len();
    for (lpi, (fname, info)) in entries.iter().enumerate() {
        if got_int.get() {
            break;
        }
        msg_str(c"file: ");
        msg_str(fname.as_cstr());
        // Always true, as upstream's test is.
        msg_putchar('\n' as c_int);
        if let Some(info) = info {
            msg_str(info.as_cstr());
            if lpi + 1 < count {
                msg_putchar('\n' as c_int);
            }
        }
    }
    msg_end();
}

/// `:spelldump` — open a new window holding every word of the current
/// `'spelllang'`, in `:mkspell` input format. With `!` each word gets its
/// `COMMON` count appended.
pub fn ex_spelldump(excmd: &mut ExArg) {
    if no_spell_checking(Win::current()) {
        return;
    }
    let spl: OptVal = get_option_value(kOptSpelllang, OptionSetFlags::LOCAL);

    let _ = do_cmdline_cmd(c"new");

    // Spell checking has to be on in the new window for the dump to
    // mean anything.
    set_option_value_give_err(kOptSpell, boolean_optval(Some(true)), OptionSetFlags::LOCAL);
    set_option_value_give_err(kOptSpelllang, spl, OptionSetFlags::LOCAL);

    if !buf_is_empty(Buf::current()) {
        return;
    }

    let dumpflags = if excmd.forceit { DUMPFLAG_COUNT } else { 0 };
    spell_dump_compl(None, 0, None, dumpflags);

    // Drop the empty line the new buffer started with.
    if Buf::current().b_ml.ml_line_count > 1 {
        let _ = ml_delete(Buf::current().b_ml.ml_line_count);
    }
    redraw_later(Win::current(), UPD_NOT_VALID);
}

/// Where the words of a walk go: the current buffer, or Insert-mode
/// completion.
struct Sink<'a> {
    /// The completion pattern, or `None` to dump into the buffer.
    pat: Option<&'a [u8]>,
    /// The direction completions are added in; `FORWARD` after the first.
    dir: Option<&'a mut Direction>,
    dumpflags: c_int,
}

/// Walk every word of every loaded language.
///
/// With `pat` absent, dump them into the current buffer. Otherwise offer
/// those starting with `pat` as Insert-mode completions, honouring `ic`
/// and adding them in `dir` (which is set to `FORWARD` after the first, so
/// a `BACKWARD` request applies only once).
pub fn spell_dump_compl(
    pat: Option<&[u8]>,
    ic: c_int,
    dir: Option<&mut Direction>,
    dumpflags_arg: c_int,
) {
    let pat = pat.map(until_nul);
    let mut lnum: LineNr = 0;
    let mut dumpflags = dumpflags_arg;

    // Case handling of the pattern is dump_word()'s problem, but what
    // kind of handling it is gets decided once, here.
    if let Some(pat) = pat {
        if ic != 0 {
            dumpflags |= DUMPFLAG_ICASE;
        } else {
            let n = captype(pat, pat.len());
            if n == WordFlags::ONECAP {
                dumpflags |= DUMPFLAG_ONECAP;
            } else if n == WordFlags::ALLCAP && pat.len() > c_cluster_len(pat) {
                dumpflags |= DUMPFLAG_ALLCAP;
            }
        }
    }

    let langp = Win::current().syn_block().b_langp.clone();

    // Regions can only be dumped when every language agrees on them.
    let mut region_names: Option<Vec<u8>> = None;
    let mut do_region = true;
    for lp in &langp {
        let Some(regions) = loaded_langs.with(|langs| langs.get(lp.slang).map(|sl| sl.sl_regions))
        else {
            continue;
        };
        let regions = until_nul(&regions);
        if !regions.is_empty() {
            match &region_names {
                None => region_names = Some(regions.to_vec()),
                Some(names) if names != regions => {
                    do_region = false;
                    break;
                }
                Some(_) => {}
            }
        }
    }

    if let Some(names) = region_names.as_ref().filter(|_| do_region && pat.is_none()) {
        let header = io_line([b"/regions=", &names[..]].concat());
        let _ = ml_append_text(lnum, &header);
        lnum += 1;
    } else {
        do_region = false;
    }

    let mut sink = Sink {
        pat,
        dir,
        dumpflags,
    };
    for lp in &langp {
        loaded_langs.with(|langs| {
            let Some(slang) = langs.get(lp.slang) else {
                return;
            };
            if slang.sl_fold_tree.is_empty() {
                return; // reloading this language failed
            }
            dump_language(slang, lp.region, do_region, &mut sink, &mut lnum);
        });
    }
}

/// The walk of [`spell_dump_compl`] over one language's two trees.
fn dump_language(
    slang: &SpellLang,
    lp_region: c_int,
    do_region: bool,
    sink: &mut Sink<'_>,
    lnum: &mut LineNr,
) {
    let mut arridx = [0usize; MAXWLEN];
    let mut curi = [0usize; MAXWLEN];
    let mut word = [0u8; MAXWLEN];

    if sink.pat.is_none() {
        let header = io_line([b"# file: ", &slang.sl_fname[..]].concat());
        let _ = ml_append_text(*lnum, &header);
        *lnum += 1;
    }

    // Without prefixes, a pattern can prune the walk; with them, a
    // prefix could still make a non-matching branch match.
    let patlen = match sink.pat {
        Some(pat) if slang.sl_prefix_tree.is_empty() => pat.len() as isize,
        _ => -1,
    };

    // Round 1 is the case-folded tree, round 2 the keep-case one.
    for round in 1..=2 {
        let tree = if round == 1 {
            sink.dumpflags &= !DUMPFLAG_KEEPCASE;
            slang.sl_fold_tree.view()
        } else {
            sink.dumpflags |= DUMPFLAG_KEEPCASE;
            slang.sl_keep_tree.view()
        };
        if tree.is_empty() {
            continue; // this tree is empty
        }

        let mut depth: isize = 0;
        arridx[0] = 0;
        curi[0] = 1;
        while depth >= 0 && !got_int.get() && (sink.pat.is_none() || !ins_compl_interrupted()) {
            let d = depth as usize;
            if curi[d] > tree.node_len(arridx[d]) {
                // Every child of this node is done.
                depth -= 1;
                line_breakcheck();
                ins_compl_check_keys(50, false);
                continue;
            }

            let n = arridx[d] + curi[d];
            curi[d] += 1;
            let c = tree.byte(n);
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
                        || (flags.bits() as c_uint >> 16) & lp_region as c_uint != 0)
                {
                    word[d] = 0;
                    if !do_region {
                        flags.clear(WordFlags::REGION);
                    }

                    // Dump the bare word when it has no prefix, or
                    // when this is the first of its prefixes.
                    let prefix_id = flags.bits() as c_uint >> 24;
                    if prefix_id == 0 || curi[d] == 2 {
                        dump_word(slang, &word, sink, flags, *lnum);
                        if sink.pat.is_none() {
                            *lnum += 1;
                        }
                    }

                    if prefix_id != 0 {
                        *lnum = dump_prefixes(slang, &word, sink, flags, *lnum);
                    }
                }
            } else {
                // An ordinary byte: descend.
                word[d] = c;
                depth += 1;
                arridx[depth as usize] = tree.child_node(n);
                curi[depth as usize] = 1;

                // Prune a branch that cannot match the pattern.
                // Case is always ignored here; dump_word() checks it
                // properly later. That is not exact when folding
                // changes a multi-byte character's length.
                if let Some(pat) = sink.pat
                    && depth <= patlen
                {
                    let n = depth as usize;
                    if strnicmp_in(prefix(&word, n), prefix(pat, n)) != 0 {
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
fn dump_word(
    slang: &SpellLang,
    word: &[u8],
    sink: &mut Sink<'_>,
    wordflags: WordFlags,
    lnum: LineNr,
) {
    let dumpflags = sink.dumpflags;
    let mut keepcap = false;
    let mut cword = [0u8; MAXWLEN];
    let mut flags = wordflags;

    if dumpflags & DUMPFLAG_ONECAP != 0 {
        flags |= WordFlags::ONECAP;
    }
    if dumpflags & DUMPFLAG_ALLCAP != 0 {
        flags |= WordFlags::ALLCAP;
    }

    let word = until_nul(word);
    let tw: &[u8] = if dumpflags & DUMPFLAG_KEEPCASE == 0 && flags.has(WordFlags::CAPMASK) {
        make_case_word(word, &mut cword, flags);
        until_nul(&cword)
    } else {
        if dumpflags & DUMPFLAG_KEEPCASE != 0
            && (!captype(word, word.len()).has(WordFlags::KEEPCAP) || flags.has(WordFlags::FIXCAP))
        {
            keepcap = true;
        }
        word
    };

    let Some(pat) = sink.pat else {
        let mut line = tw.to_vec();
        if flags.has(WordFlags::BANNED | WordFlags::RARE | WordFlags::REGION) || keepcap {
            // Upstream builds this in a buffer of `MAXWLEN + 10`.
            let room = MAXWLEN + 10 - 1;
            line.truncate(room);
            let push = |line: &mut Vec<u8>, text: &[u8]| {
                let n = text.len().min(room - line.len().min(room));
                line.extend_from_slice(&text[..n]);
            };
            push(&mut line, b"/");
            if keepcap {
                push(&mut line, b"=");
            }
            if flags.has(WordFlags::BANNED) {
                push(&mut line, b"!");
            } else if flags.has(WordFlags::RARE) {
                push(&mut line, b"?");
            }
            if flags.has(WordFlags::REGION) {
                for i in 0..7 {
                    if flags.has(WordFlags::from_bits(0x10000 << i)) {
                        push(&mut line, (i + 1).to_string().as_bytes());
                    }
                }
            }
        }

        if dumpflags & DUMPFLAG_COUNT != 0 {
            // ":spelldump!" wants the word's COMMON count.
            if let Some(&count) = slang.sl_wordcount.get(tw) {
                let mut counted = tw.to_vec();
                counted.push(b'\t');
                counted.extend_from_slice(count.to_string().as_bytes());
                line = io_line(counted);
            }
        }

        let _ = ml_append_text(lnum, &line);
        return;
    };

    let matches = if dumpflags & DUMPFLAG_ICASE != 0 {
        strnicmp_in(prefix(tw, pat.len()), pat) == 0
    } else {
        tw.starts_with(pat)
    };
    let want = sink.dir.as_deref().copied().unwrap_or(FORWARD);
    if matches && ins_compl_add_infercase(tw, tw.len(), p_ic(), None, want, false, 0) == OK {
        // A BACKWARD request is honoured only for the first match.
        if let Some(dir) = sink.dir.as_deref_mut() {
            *dir = FORWARD;
        }
    }
}

/// Cross `word` with every prefix that accepts it, emitting each
/// combination, and return the line number reached.
///
/// A prefix with a condition is also tried against the word with its first
/// letter upper-cased, which is how "Un-" style prefixes reach words that
/// are stored lower-case.
fn dump_prefixes(
    slang: &SpellLang,
    word: &[u8],
    sink: &mut Sink<'_>,
    flags: WordFlags,
    startlnum: LineNr,
) -> LineNr {
    let mut arridx = [0usize; MAXWLEN];
    let mut curi = [0usize; MAXWLEN];
    let mut prefix = [0u8; MAXWLEN];
    let mut word_up = [0u8; MAXWLEN];
    let mut has_word_up = false;
    let mut lnum = startlnum;
    let word = until_nul(word);

    // A word starting lower-case gets an upper-case twin to try.
    let c = char_at(word);
    if spell_toupper(c) != c {
        onecap_copy(word, &mut word_up, true);
        has_word_up = true;
    }

    let tree = slang.sl_prefix_tree.view();
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

            // The bare word, and then the upper-cased one — but that only
            // with prefixes that carry a condition.
            let up = until_nul(&word_up);
            let tries: [(&[u8], bool); 2] = [(word, false), (up, true)];
            for (candidate, cond_req) in tries {
                if cond_req && !has_word_up {
                    continue;
                }
                let c = valid_word_prefix(i as c_int, n, flags, candidate, slang, cond_req);
                if c != 0 {
                    put_tail(&mut prefix[d..], candidate);
                    let pfx_flags = if WordFlags::from_bits(c).has(WordFlags::RAREPFX) {
                        flags | WordFlags::RARE
                    } else {
                        flags
                    };
                    dump_word(slang, &prefix, sink, pfx_flags, lnum);
                    if lnum != 0 {
                        lnum += 1;
                    }
                }
            }
        } else {
            // An ordinary byte: descend.
            prefix[d] = tree.byte(n);
            depth += 1;
            arridx[depth as usize] = tree.child_node(n);
            curi[depth as usize] = 1;
        }
    }

    lnum
}

/// Copy `text` into `buf`, NUL terminated, cut to fit: `xstrlcpy`.
fn put_tail(buf: &mut [u8], text: &[u8]) {
    let n = text.len().min(buf.len() - 1);
    buf[..n].copy_from_slice(&text[..n]);
    buf[n] = 0;
}
