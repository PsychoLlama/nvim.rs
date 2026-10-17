//! Completing a buffer name -- `expand_buf_names()`.
//!
//! The command-line completion side of the buffer list: match every listed
//! (or, with `!`, every) buffer against the pattern, either as a regexp or
//! with the fuzzy matcher, sort the results by score or by last-used time,
//! and return them as the completion candidates.  [`buflist_match`] and
//! [`fname_match`] are the per-buffer test it and `buflist_findpat` share,
//! and [`find_buf`]/[`buflist_nr2name`] the number lookups.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cmdexpand::{BUF_DIFF_FILTER, WildOpts};
use crate::cstr;
use crate::memory::XString;
use core::ffi::c_int;

use super::*;
use crate::cmdexpand::cmdline_fuzzy_complete;
use crate::diff::diff_mode_buf;
use crate::fuzzy::fuzzy_match_str;
use crate::option::vars::{p_fic, p_wic};
use crate::os::env::home_replace_in;
use crate::regexp::{RE_MAGIC, vim_regexec};
use crate::types::{Failed, RegMatch};
use crate::winlayer::{self, Buf, Win, buffers};

fn current_win() -> Win {
    Win::current()
}

/// `home_replace_save`: `name` with `$HOME` written as `~`, freshly made.
/// `buffer` decides whether a help file keeps only its tail.
fn home_replaced(buffer: Option<Buf>, name: &CStr) -> XString {
    home_replace_in(buffer, name, name.to_bytes().len() + 3, true)
}

// ---------------------------------------------------------------------------
// Completing ":buffer"

/// One buffer name `:buffer` completion found, and what orders it.
struct Found {
    buf: Buf,
    name: XString,
    score: c_int,
}

/// Every buffer name matching `pat`, for command-line completion of
/// `:buffer` and `:sbuffer`.
pub(crate) fn buf_name_matches(pat: &CStr, options: WildOpts) -> Result<Vec<XString>, Failed> {
    if options.has(BUF_DIFF_FILTER) && current_win().w_onebuf_opt.wo_diff == 0 {
        return Err(Failed);
    }

    let fuzzy = cmdline_fuzzy_complete(pat.to_bytes());
    // A copy of "pat" with a leading "^" taken off (when matching with a
    // regular expression).
    let mut regmatch = RegMatch::default();
    if !fuzzy {
        let bytes = pat.to_bytes_with_nul();
        let patc = if bytes[0] == b'^' { &bytes[1..] } else { bytes };
        regmatch = RegMatch::compile(cstr::in_bytes(patc), RE_MAGIC, false).unwrap_or_default();
    }

    let mut found: Vec<Found> = Vec::new();
    for buf in buffers() {
        // Skip unlisted buffers.
        if buf.b_p_bl == 0 {
            continue;
        }
        if options.has(BUF_DIFF_FILTER) {
            // Skip buffers not suitable for :diffget or :diffput completion.
            if buf.raw() == Buf::current_raw() || !diff_mode_buf(buf) {
                continue;
            }
        }

        let mut score = 0;
        let name = if fuzzy {
            // First try matching with the short file name, then with the
            // full path file name. An unnamed buffer has neither.
            [buf.name.short(), buf.name.full()]
                .into_iter()
                .flatten()
                .find(|name| {
                    score = fuzzy_match_str(name, pat);
                    score != FUZZY_SCORE_NONE as c_int
                })
        } else {
            if regmatch.regprog.is_none() {
                // An invalid pattern, possibly after recompiling.
                return Err(Failed);
            }
            match buflist_match(&mut regmatch, buf, p_wic()) {
                Some(BufNameKind::Short) => buf.name.short(),
                Some(BufNameKind::Full) => buf.name.full(),
                None => None,
            }
        };
        let Some(name) = name else {
            continue;
        };
        let name = if options.has(WildOpts::HOME_REPLACE) {
            home_replaced(Some(buf), name)
        } else {
            XString::from_cstr(name)
        };
        found.push(Found { buf, name, score });
    }
    drop(regmatch);
    if found.is_empty() {
        return Err(Failed);
    }

    if fuzzy {
        // Best score first; equal scores keep the buffer list's order.
        found.sort_by_key(|f| core::cmp::Reverse(f.score));
    } else if options.has(WildOpts::BUFLASTUSED) {
        return Ok(order_by_last_used(found));
    }
    Ok(found.into_iter().map(|f| f.name).collect())
}

/// The order `qsort` sorts by: the buffer each name came from.
#[derive(Clone, Copy)]
struct ByBuffer(usize, Buf);

impl LastUsed for ByBuffer {
    fn last_used_buffer(&self) -> Buf {
        self.1
    }
}

/// The names in `found`, most recently used buffer first -- but with the
/// current buffer moved last when it would otherwise come first.
///
/// `qsort`, because its order of two buffers entered in the same second is
/// upstream's, and a Rust sort would order those ties differently.
fn order_by_last_used(found: Vec<Found>) -> Vec<XString> {
    let mut order: Vec<ByBuffer> = found
        .iter()
        .enumerate()
        .map(|(i, f)| ByBuffer(i, f.buf))
        .collect();
    sort_by_last_used(&mut order);
    let mut names: Vec<Option<XString>> = found.into_iter().map(|f| Some(f.name)).collect();
    let mut take = |at: &ByBuffer| names[at.0].take().expect("each name is taken once");
    let mut sorted: Vec<XString> = Vec::with_capacity(order.len());
    if Some(order[0].1) == Buf::current_or_none() {
        // The current buffer came first: place it at the end.
        sorted.extend(order[1..].iter().map(&mut take));
        sorted.push(take(&order[0]));
    } else {
        sorted.extend(order.iter().map(&mut take));
    }
    sorted
}

// ---------------------------------------------------------------------------
// Matching one buffer

/// Which of a buffer's names matched.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum BufNameKind {
    /// The short name, `b_sfname`.
    Short,
    /// The full name, `b_ffname`.
    Full,
}

/// Whether `buffer`'s name matches `rmp`: the short file name first, then the
/// long one. `rmp->regprog` may become null when the regexp engine switches.
pub(crate) fn buflist_match(
    rmp: &mut RegMatch,
    buffer: Buf,
    ignore_case: bool,
) -> Option<BufNameKind> {
    if fname_match(rmp, buffer.name.short(), ignore_case) {
        return Some(BufNameKind::Short);
    }
    if rmp.regprog.is_some() && fname_match(rmp, buffer.name.full(), ignore_case) {
        return Some(BufNameKind::Full);
    }
    None
}

/// Whether `name` matches `rmp`. `$HOME` is tried both as itself and as `~`.
fn fname_match(rmp: &mut RegMatch, name: Option<&CStr>, ignore_case: bool) -> bool {
    // An extra check for valid arguments.
    let Some(name) = name.filter(|_| rmp.regprog.is_some()) else {
        return false;
    };

    // Ignore case when 'fileignorecase' or the argument is set.
    rmp.rm_ic = p_fic() || ignore_case;
    if vim_regexec(rmp, name, 0) {
        return true;
    }
    if rmp.regprog.is_none() {
        return false;
    }
    // Replace $(HOME) with '~' and try matching again.
    let p = home_replaced(None, name);
    vim_regexec(rmp, p.as_cstr(), 0)
}

// ---------------------------------------------------------------------------
// Looking one up by number

/// The buffer numbered `nr`, or the alternate file for 0.
///
/// The registry lookup the whole editor's "buffer by number" rests on,
/// upstream's `find_buf` under its Rust name: a number naming no
/// buffer is `None` rather than a null to test for.
pub(crate) fn find_buf(nr: c_int) -> Option<Buf> {
    let nr = if nr == 0 {
        current_win().w_alt_fnum
    } else {
        nr
    };
    winlayer::buffer(nr)
}

/// The name of buffer `n`, shortened with `home_replace`; `None` when there
/// is no such buffer, empty for one with no name.
pub fn buflist_nr2name(n: c_int, fullname: c_int, helptail: c_int) -> Option<XString> {
    let buf = find_buf(n)?;
    let name = if fullname != 0 {
        buf.name.full()
    } else {
        buf.name.shown()
    };
    let helptail = (helptail != 0).then_some(buf);
    Some(name.map_or_else(XString::new, |name| home_replaced(helptail, name)))
}
