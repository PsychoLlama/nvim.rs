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

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cmdexpand::{BUF_DIFF_FILTER, WildOpts};
use core::ffi::{c_char, c_int, c_void};
use core::{ptr, slice};

use super::*;
use crate::cmdexpand::cmdline_fuzzy_complete;
use crate::diff::diff_mode_buf;
use crate::fuzzy::{fuzzy_match_str, fuzzymatches_to_strmatches};
use crate::main::{curbuf, p_fic, p_wic};
use crate::memory::{xfree, xmalloc, xstrdup};
use crate::os::env::home_replace_save;
use crate::regexp::{RE_MAGIC, vim_regcomp, vim_regexec, vim_regfree};
use crate::types::{Failed, buf_T, colnr_T, fuzmatch_str_T, regmatch_T, regprog_T, size_t};
use crate::winlayer::{self, Buf, Win, buffers};
use ::libc::qsort;

/// A `regmatch_T` holding no compiled program.
pub(crate) const NO_REGMATCH: regmatch_T = regmatch_T {
    regprog: ptr::null_mut::<regprog_T>(),
    startp: [ptr::null_mut::<c_char>(); 10],
    endp: [ptr::null_mut::<c_char>(); 10],
    rm_matchcol: 0,
    rm_ic: false,
};

// ---------------------------------------------------------------------------
// The neighbours, wrapped

fn free(p: *mut c_char) {
    // SAFETY: an owned allocation or null.
    unsafe { xfree(p.cast::<c_void>()) };
}

fn dup(p: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated buffer name.
    unsafe { xstrdup(p) }
}

/// `xmalloc` of `n` elements, for the arrays this file hands back to the
/// completion machinery (which frees them with `xfree`).
fn alloc_array<T>(n: c_int) -> *mut T {
    // SAFETY: `xmalloc` aborts rather than answering null.
    unsafe { xmalloc(n as size_t * size_of::<T>()) }.cast::<T>()
}

/// `array[i] = value`, for the three arrays filled in round two. Each index
/// is below the count round one measured, which is what the array was sized
/// from.
fn set_at<T>(array: *mut T, i: c_int, value: T) {
    // SAFETY: `i` is inside the array `alloc_array` sized for `count`.
    unsafe { array.add(i as usize).write(value) };
}

/// Whether the pattern asks for fuzzy matching (`'wildoptions'`).
fn wants_fuzzy(pat: *const c_char) -> bool {
    // SAFETY: a NUL-terminated pattern.
    unsafe { cmdline_fuzzy_complete(pat) }
}

fn regcomp(pat: *const c_char, flags: c_int) -> *mut regprog_T {
    // SAFETY: a NUL-terminated pattern; the answer is null on a bad one.
    unsafe { vim_regcomp(pat, flags) }
}

fn regfree(prog: *mut regprog_T) {
    // SAFETY: a compiled program or null.
    unsafe { vim_regfree(prog) };
}

fn regexec(rmp: &mut regmatch_T, name: *mut c_char) -> bool {
    // SAFETY: a live match state with a compiled program, and a
    // NUL-terminated string to match it against.
    unsafe { vim_regexec(rmp, name, 0 as colnr_T) }
}

/// `home_replace_save`: `name` with `$HOME` written as `~`, freshly
/// allocated. `buf` decides whether a help file keeps only its tail.
fn home_replaced(buf: *mut buf_T, name: *const c_char) -> *mut c_char {
    // SAFETY: a live buffer or null, and a NUL-terminated name.
    unsafe { home_replace_save(buf, name) }
}

fn fuzzy_score(name: *const c_char, pat: *const c_char) -> c_int {
    // SAFETY: a NUL-terminated name (`fuzzy_match_str` tests for null
    // itself) and pattern.
    unsafe { fuzzy_match_str(name, pat) }
}

fn diff_mode(mut buf: Buf) -> bool {
    // SAFETY: a live buffer.
    diff_mode_buf(buf)
}

fn current_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

// ---------------------------------------------------------------------------
// Completing ":buffer"

/// Every buffer name matching `pat`, for command-line completion of
/// `:buffer` and `:sbuffer`.
pub unsafe fn expand_buf_names(
    pat: *mut c_char,
    num_file: *mut c_int,
    file: *mut *mut *mut c_char,
    options: WildOpts,
) -> Result<(), Failed> {
    let mut matches: *mut bufmatch_T = ptr::null_mut();
    let mut to_free = false;

    // SAFETY: the caller's promise -- two out-parameters to fill in.
    let (num_file, file) = unsafe { (&mut *num_file, &mut *file) };
    // The return values in case of FAIL.
    *num_file = 0;
    *file = ptr::null_mut();

    if options.has(BUF_DIFF_FILTER) && current_win().w_onebuf_opt.wo_diff == 0 {
        return Err(Failed);
    }

    let fuzzy = wants_fuzzy(pat);
    let mut patc: *mut c_char = ptr::null_mut();
    let mut fuzmatch: *mut fuzmatch_str_T = ptr::null_mut();
    let mut regmatch = NO_REGMATCH;

    // Make a copy of "pat" and change "^" to "\(^\|[\/]\)" (when matching
    // with a regular expression).
    if !fuzzy {
        // SAFETY: a NUL-terminated pattern.
        let anchored = unsafe { *pat } == b'^' as c_char;
        // SAFETY: past a byte that is not the terminator, so there is a
        // second one -- which is why upstream reads it only here.
        let next = if anchored { unsafe { *pat.add(1) } } else { 0 };
        if anchored && next != 0 {
            patc = dup(pat.wrapping_add(1));
            to_free = true;
        } else if anchored {
            patc = c"".as_ptr().cast_mut();
        } else {
            patc = pat;
        }
        regmatch.regprog = regcomp(patc, RE_MAGIC);
    }

    let mut count = 0;
    let mut score = 0;
    // round == 1: count the matches. round == 2: build the array to keep
    // them in.
    for round in 1..=2 {
        count = 0;
        for buf in buffers() {
            // Skip unlisted buffers.
            if buf.b_p_bl == 0 {
                continue;
            }
            if options.has(BUF_DIFF_FILTER) {
                // Skip buffers not suitable for :diffget or :diffput
                // completion.
                if buf.raw() == curbuf.get() || !diff_mode(buf) {
                    continue;
                }
            }

            let mut p: *mut c_char = ptr::null_mut();
            if !fuzzy {
                if regmatch.regprog.is_null() {
                    // An invalid pattern, possibly after recompiling.
                    if to_free {
                        free(patc);
                    }
                    return Err(Failed);
                }
                p = buflist_match(&mut regmatch, buf, p_wic.get() != 0);
            } else {
                // First try matching with the short file name.
                score = fuzzy_score(buf.b_sfname, pat);
                if score != FUZZY_SCORE_NONE as c_int {
                    p = buf.b_sfname;
                }
                if p.is_null() {
                    // Next try matching with the full path file name.
                    score = fuzzy_score(buf.b_ffname, pat);
                    if score != FUZZY_SCORE_NONE as c_int {
                        p = buf.b_ffname;
                    }
                }
            }

            if p.is_null() {
                continue;
            }
            if round == 1 {
                count += 1;
                continue;
            }

            p = if options.has(WildOpts::HOME_REPLACE) {
                home_replaced(buf.raw(), p)
            } else {
                dup(p)
            };

            if fuzzy {
                let entry = fuzmatch_str_T {
                    idx: count,
                    str: p,
                    score,
                };
                set_at(fuzmatch, count, entry);
            } else if !matches.is_null() {
                let entry = bufmatch_T {
                    buf: buf.raw(),
                    match_0: p,
                };
                set_at(matches, count, entry);
            } else {
                set_at(*file, count, p);
            }
            count += 1;
        }
        if count == 0 {
            // No match found, stop here.
            break;
        }
        if round == 1 {
            if fuzzy {
                fuzmatch = alloc_array::<fuzmatch_str_T>(count);
            } else {
                *file = alloc_array::<*mut c_char>(count);
                if options.has(WildOpts::BUFLASTUSED) {
                    matches = alloc_array::<bufmatch_T>(count);
                }
            }
        }
    }

    if !fuzzy {
        regfree(regmatch.regprog);
        if to_free {
            free(patc);
        }
        if !matches.is_null() {
            // SAFETY: the out-parameter holds the `count` slots allocated
            // above.
            let files = unsafe { slice::from_raw_parts_mut(*file, count as usize) };
            order_by_last_used(matches, files);
            // SAFETY: this function's own array.
            unsafe { xfree(matches.cast::<c_void>()) };
        }
    } else {
        // SAFETY: the array filled above, and the caller's out-parameter.
        unsafe { fuzzymatches_to_strmatches(fuzmatch, file, count, false) };
    }

    *num_file = count;
    if count == 0 { Err(Failed) } else { Ok(()) }
}

/// Sort `matches` by last-used time into `files`, putting the current buffer
/// last when it would otherwise come first.
///
/// `qsort` and the comparison stay upstream's: `buf_time_compare` answers 0
/// for two buffers entered in the same second, and a stable Rust sort would
/// order those ties differently.
fn order_by_last_used(matches: *mut bufmatch_T, files: &mut [*mut c_char]) {
    let count = files.len();
    if count > 1 {
        let (base, width) = (matches.cast::<c_void>(), size_of::<bufmatch_T>());
        // SAFETY: `count` initialised elements of this function's own array,
        // and a comparison function over two of them.
        unsafe { qsort(base, count, width, Some(buf_time_compare)) };
    }
    if count == 0 {
        // Unreachable: round two walks the list round one counted, so a
        // non-null `matches` means at least one entry. Upstream indexes
        // `matches[0]` and `(*file)[count - 1]` without the test.
        return;
    }
    // SAFETY: `count` initialised elements.
    let matches = unsafe { slice::from_raw_parts(matches, count) };
    if matches[0].buf == curbuf.get() {
        // The current buffer came first: place it at the end.
        for i in 1..count {
            files[i - 1] = matches[i].match_0;
        }
        files[count - 1] = matches[0].match_0;
    } else {
        for i in 0..count {
            files[i] = matches[i].match_0;
        }
    }
}

// ---------------------------------------------------------------------------
// Matching one buffer

/// Whether `buf`'s name matches `rmp`: the short file name first, then the
/// long one. `rmp->regprog` may become null when the regexp engine switches.
pub(crate) fn buflist_match(rmp: &mut regmatch_T, buf: Buf, ignore_case: bool) -> *mut c_char {
    let mut matched = fname_match(rmp, buf.b_sfname, ignore_case);
    if matched.is_null() && !rmp.regprog.is_null() {
        matched = fname_match(rmp, buf.b_ffname, ignore_case);
    }
    matched
}

/// `name` when it matches `rmp`, null when it does not. `$HOME` is tried
/// both as itself and as `~`.
fn fname_match(rmp: &mut regmatch_T, name: *mut c_char, ignore_case: bool) -> *mut c_char {
    // An extra check for valid arguments.
    if name.is_null() || rmp.regprog.is_null() {
        return ptr::null_mut();
    }

    // Ignore case when 'fileignorecase' or the argument is set.
    rmp.rm_ic = p_fic.get() != 0 || ignore_case;
    if regexec(rmp, name) {
        return name;
    }
    if rmp.regprog.is_null() {
        return ptr::null_mut();
    }
    // Replace $(HOME) with '~' and try matching again.
    let p = home_replaced(ptr::null_mut(), name);
    let matched = if regexec(rmp, p) {
        name
    } else {
        ptr::null_mut()
    };
    free(p);
    matched
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

/// The name of buffer `n`, shortened with `home_replace`, freshly allocated;
/// null when there is no such buffer.
pub fn buflist_nr2name(n: c_int, fullname: c_int, helptail: c_int) -> *mut c_char {
    let Some(mut buf) = find_buf(n) else {
        return ptr::null_mut();
    };
    let name = if fullname != 0 {
        buf.b_ffname
    } else {
        buf.b_fname
    };
    let tail_only = if helptail != 0 {
        buf.raw()
    } else {
        ptr::null_mut()
    };
    home_replaced(tail_only, name)
}
