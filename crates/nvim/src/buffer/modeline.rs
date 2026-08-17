//! Modelines -- `chk_modeline()` and `'modelines'`.
//!
//! [`do_modelines`] scans the first and last `'modelines'` lines of a buffer
//! for a `vim:`/`ex:` marker, and [`chk_modeline`] parses one: the
//! `set`-style option list, the `:`-terminated form, the version-guarded
//! `vim<800:` prefixes, and the `sandbox`/`'modelineexpr'` restrictions that
//! decide which options a file is allowed to set.
//!
//! Both halves work in byte offsets into a slice rather than in moving
//! `char *`s: the marker search reads the buffer line in place, and the
//! option list is parsed inside a local copy, because splitting it means
//! writing NULs over the `:` separators.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::slice;

use super::*;
use crate::ascii::ascii_isspace;
use crate::charset::try_getdigits;
use crate::global_cell::GlobalCell;
use crate::main::{current_sctx, p_mls, secure};
use crate::memline::{ml_get, ml_get_len};
use crate::option::do_set;
use crate::runtime::{estack_pop, estack_push};
use crate::types::{intmax_t, linenr_T, scid_T};
use crate::version::min_vim_version;
use crate::winlayer::Buf;

// ---------------------------------------------------------------------------
// The neighbours, wrapped

fn current_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit. Read afresh at every use
    // below: `do_set` can fire an autocommand that changes it.
    unsafe { Buf::current() }
}

/// Line `lnum` of the current buffer, including its NUL terminator, so that
/// the one-past-the-end reads upstream's `s[1]`/`s[2]`/`s[3]` tests make are
/// in bounds here too.
fn buffer_line(lnum: linenr_T) -> &'static [u8] {
    // SAFETY: `ml_get` answers a live, NUL-terminated line of `ml_get_len`
    // bytes; the line stays put for as long as this function's caller runs.
    unsafe {
        let len = ml_get_len(lnum) as usize;
        slice::from_raw_parts(ml_get(lnum).cast::<u8>(), len + 1)
    }
}

/// `try_getdigits`: the decimal number at `line[off]`, and the offset past
/// it. Answers `None` when the value did not fit, which is upstream's
/// "this is not a version-guarded modeline after all".
fn version_at(line: &[u8], off: usize) -> Option<(intmax_t, usize)> {
    let base = line.as_ptr().cast::<c_char>();
    let mut p = base.wrapping_add(off).cast_mut();
    let mut vers: intmax_t = 0;
    // SAFETY: a position inside a NUL-terminated line, and a local to fill
    // in; `strtoimax` stops at the terminator.
    let ok = unsafe { try_getdigits(&raw mut p, &raw mut vers) };
    ok.then(|| (vers, p as usize - base as usize))
}

/// `do_set` over the option list at `text[off..]`, which it may write into.
fn set_options(text: &mut [u8], off: usize, flags: c_int) -> c_int {
    let arg = text[off..].as_mut_ptr().cast::<c_char>();
    // SAFETY: a NUL-terminated option list inside a buffer this function
    // owns, which is what `do_set` parses and truncates in place.
    unsafe { do_set(arg, flags) }
}

fn push_estack(lnum: linenr_T) {
    // SAFETY: a NUL-terminated name for the "modelines" script level.
    unsafe { estack_push(ETYPE_MODELINE, c"modelines".as_ptr().cast_mut(), lnum) };
}

fn pop_estack() {
    // SAFETY: matches the `push_estack` above.
    unsafe { estack_pop() };
}

/// The first byte at or after `off` that is not a space or tab --
/// `skipwhite`, in offsets.
fn skip_white(line: &[u8], off: usize) -> usize {
    let mut i = off;
    while at(line, i) == b' ' || at(line, i) == b'\t' {
        i += 1;
    }
    i
}

/// `line[i]`, or NUL past the end. Upstream reads a NUL-terminated string,
/// where every byte after the terminator is unreachable rather than absent.
fn at(line: &[u8], i: usize) -> u8 {
    line.get(i).copied().unwrap_or(0)
}

// ---------------------------------------------------------------------------
// :doautocmd's and :edit's modeline pass

/// Process the modelines of the current buffer -- the first and last
/// `'modelines'` lines of it.
///
/// `flags` is `OPT_WINONLY` or `OPT_NOWIN`.
pub unsafe extern "C" fn do_modelines(flags: c_int) {
    static entered: GlobalCell<c_int> = GlobalCell::new(0);

    if current_buf().b_p_ml == 0 {
        return;
    }
    let mut nmlines = p_mls.get() as c_int;
    if nmlines == 0 {
        return;
    }

    // Disallow recursive entry here. That can happen when executing a
    // modeline triggers an autocommand which reloads modelines with a ":do".
    if entered.get() != 0 {
        return;
    }
    entered.set(entered.get() + 1);

    let mut lnum: linenr_T = 1;
    while current_buf().b_p_ml != 0
        && lnum <= current_buf().b_ml.ml_line_count
        && lnum <= nmlines as linenr_T
    {
        if chk_modeline(lnum, flags) == FAIL {
            nmlines = 0;
        }
        lnum += 1;
    }

    lnum = current_buf().b_ml.ml_line_count;
    while current_buf().b_p_ml != 0
        && lnum > 0
        && lnum > nmlines as linenr_T
        && lnum > current_buf().b_ml.ml_line_count - nmlines as linenr_T
    {
        if chk_modeline(lnum, flags) == FAIL {
            nmlines = 0;
        }
        lnum -= 1;
    }
    entered.set(entered.get() - 1);
}

/// Check one line for a mode string, and apply it. `FAIL` when an error was
/// encountered, which stops the whole pass.
fn chk_modeline(lnum: linenr_T, flags: c_int) -> c_int {
    let line = buffer_line(lnum);
    // The NUL the slice ends on is not part of the line: `line_end` is where
    // upstream's `ml_get_len` puts it.
    let line_end = line.len() - 1;
    let Some(marker) = find_marker(line) else {
        return OK;
    };

    // Skip over "ex:", "vi:" or "vim:".
    let mut s = marker;
    loop {
        s += 1;
        if at(line, s - 1) == b':' {
            break;
        }
    }

    // Copy the line, because parsing it means writing NULs over the ':'
    // separators. `line_end` moves with it.
    let mut copy = line[s..line_end].to_vec();
    copy.push(0);
    apply_modeline(&mut copy, lnum, flags)
}

/// The offset of the `ex:`/`vi:`/`vim:` marker in `line`, if it has one.
fn find_marker(line: &[u8]) -> Option<usize> {
    let mut prev: c_int = -1;
    let mut s = 0;
    while at(line, s) != 0 {
        // A version guard whose number could not be read leaves `prev`
        // alone: upstream `continue`s past the assignment below, so the byte
        // before the "vim" stays the one that decides the next position.
        let mut keep_prev = false;
        if prev == -1 || ascii_isspace(prev) {
            if prev != -1 && line[s..].starts_with(b"ex:") || line[s..].starts_with(b"vi:") {
                return Some(s);
            }
            // Accept both "vim" and "Vim".
            if (at(line, s) == b'v' || at(line, s) == b'V')
                && at(line, s + 1) == b'i'
                && at(line, s + 2) == b'm'
            {
                match version_guard_matches(line, s) {
                    None => keep_prev = true,
                    Some(true) => return Some(s),
                    Some(false) => {}
                }
            }
        }
        if !keep_prev {
            prev = at(line, s) as c_int;
        }
        s += 1;
    }
    None
}

/// The `vim<800:`/`vim=800:`/`vim>800:`/`vim800:` guard at `s`: whether this
/// editor's version satisfies it, and the modeline is one to obey.
///
/// `None` when no version number could be read at all, which upstream
/// treats as "not a modeline marker, and do not even count the bytes".
fn version_guard_matches(line: &[u8], s: usize) -> Option<bool> {
    let guard = at(line, s + 3);
    let digits_at = if guard == b'<' || guard == b'=' || guard == b'>' {
        s + 4
    } else {
        s + 3
    };
    let (vers, e) = version_at(line, digits_at)?;

    let vim_version = min_vim_version() as intmax_t;
    Some(
        at(line, e) == b':'
            // "Vim" (capitalised) only counts for a "set" list.
            && (at(line, s) != b'V' || line[skip_white(line, e + 1)..].starts_with(b"set"))
            && (guard == b':'
                || vim_version >= vers && guard.is_ascii_digit()
                || vim_version < vers && guard == b'<'
                || vim_version > vers && guard == b'>'
                || vim_version == vers && guard == b'='),
    )
}

/// Split the modeline's option list on `:` and hand each part to `do_set`.
///
/// `text` is the caller's own NUL-terminated copy, which this writes into:
/// `\:` collapses to `:`, and every separator becomes a NUL.
fn apply_modeline(text: &mut [u8], lnum: linenr_T, flags: c_int) -> c_int {
    let mut retval = OK;
    let mut line_end = text.len() - 1;

    // Prepare for emsg().
    push_estack(lnum);

    let mut s = 0;
    let mut end = false;
    while !end {
        s = skip_white(text, s);
        if at(text, s) == 0 {
            break;
        }

        // Find the end of the set command: ':' or the end of the line,
        // skipping over "\:" and replacing it with ":".
        let mut e = s;
        while at(text, e) != b':' && at(text, e) != 0 {
            if at(text, e) == b'\\' && at(text, e + 1) == b':' {
                text.copy_within(e + 1..line_end + 1, e);
                line_end -= 1;
            }
            e += 1;
        }
        if at(text, e) == 0 {
            end = true;
        }

        // With a "set" command, require a terminating ':' and ignore what
        // follows it. Accept "se" for compatibility with Elvis.
        //   "vi:set opt opt opt: foo" -- foo not interpreted
        //   "vi:opt opt opt: foo"     -- foo interpreted
        if text[s..].starts_with(b"set ") || text[s..].starts_with(b"se ") {
            if at(text, e) != b':' {
                // No terminating ':'.
                break;
            }
            end = true;
            s += if at(text, s + 2) == b' ' { 3 } else { 4 };
        }
        // Truncate the set command.
        text[e] = 0;

        if at(text, s) != 0 {
            // Skip over an empty "::".
            retval = set_one(text, s, lnum, flags);
            if retval == FAIL {
                // Stop if an error was found.
                break;
            }
        }
        // Advance to the next part, carefully not going off the end.
        s = if e == line_end { e } else { e + 1 };
    }

    pop_estack();
    retval
}

/// One `:`-separated part of a modeline, executed with `sandbox` on and the
/// script context pointing at the modeline.
fn set_one(text: &mut [u8], s: usize, lnum: linenr_T, flags: c_int) -> c_int {
    let secure_save = secure.get();
    let save_current_sctx = current_sctx.get();
    current_sctx.with_mut(|sctx| {
        sctx.sc_sid = SID_MODELINE as scid_T;
        sctx.sc_seq = 0;
        sctx.sc_lnum = lnum;
    });
    // Make sure no risky things are executed as a side effect.
    secure.set(1);

    let retval = set_options(text, s, OPT_MODELINE as c_int | OPT_LOCAL as c_int | flags);

    secure.set(secure_save);
    current_sctx.set(save_current_sctx);
    retval
}
