//! When to reindent: 'cinkeys', and the front doors.
//!
//! [`in_cinkeys`] answers whether a typed character should trigger a
//! reindent, which is the whole of 'cinkeys': a comma-separated list of keys,
//! each optionally prefixed by `*` (reindent *before* inserting), `!` (never
//! insert, just reindent) or `0` (only when it is the first thing on the
//! line), plus the `o`/`O`/`e`/`=` word forms.  [`cindent_on`] is the "is C
//! indenting active at all" test 'cindent'/'indentexpr' share, and
//! [`f_cindent`] is `cindent()`.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::types::NUL;
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_char, c_int};

/// Whether C indenting is on: `'cindent'` or a non-empty `'indentexpr'`, and
/// not `'paste'`.
///
/// # Safety
/// Reads the current buffer.
pub unsafe fn cindent_on() -> bool {
    // SAFETY: 'indentexpr' is a NUL-terminated option string.  The cheaper
    // tests are kept in front of it, as upstream has them.
    p_paste.get() == 0 && (cur_buf().b_p_cin != 0 || unsafe { *cur_buf().b_p_inde } != 0)
}

/// Which prefix of a 'cinkeys' item this call is asking about.
///
/// `'*'` is "reindent *before* inserting", `'!'` "do not insert at all, just
/// reindent", and anything else the plain "reindent afterwards" form -- which
/// matches every item that is *not* prefixed `*`.
fn wants(when: c_int, look: u8) -> bool {
    match when as u8 {
        b'*' => look == b'*',
        b'!' => look == b'!',
        _ => look != b'*',
    }
}

/// Whether 'cinkeys' (or 'indentkeys', with 'indentexpr' set) asks for a
/// reindent on `keytyped`.
///
/// `keytyped` is normally the character just typed, but may also be
/// [`KEY_OPEN_FORW`]/[`KEY_OPEN_BACK`] (the `o`/`O` commands) or
/// [`KEY_COMPLETE`] (a completion just finished).  `when` selects the prefix
/// class -- see [`wants`].  `line_is_empty` allows the `0` forms.
///
/// # Safety
/// Reads the current buffer, window and cursor line; may unlock it.
pub unsafe fn in_cinkeys(keytyped: c_int, when: c_int, line_is_empty: bool) -> bool {
    if keytyped == NUL {
        // Can happen with CTRL-Y and CTRL-E on a short line.
        return false;
    }

    // 'indentexpr' set means 'indentkeys' rather than 'cinkeys'.
    // SAFETY: 'indentexpr' is a NUL-terminated option string.
    let mut look = if unsafe { *cur_buf().b_p_inde } != 0 {
        cur_buf().b_p_indk
    } else {
        cur_buf().b_p_cink
    };

    loop {
        // Every `look.add(1)` below is guarded by the test that has just seen
        // a non-NUL byte at `look`, so the walk never leaves the option.
        // SAFETY: `look` points inside a NUL-terminated option string.
        let mut c = unsafe { *look as u8 };
        if c == 0 {
            break;
        }
        let mut try_match = wants(when, c);
        if c == b'*' || c == b'!' {
            // SAFETY: `c` is not the NUL, so `look.add(1)` is at worst it.
            look = unsafe { look.add(1) };
            // SAFETY: `look` still points inside the option string.
            c = unsafe { *look as u8 };
        }

        // A '0' means "only when the line is empty" -- but the word forms
        // below may still match on the last character of the word.
        let try_match_word = if c == b'0' {
            let word = try_match;
            try_match &= line_is_empty;
            // SAFETY: `c` is not the NUL, so `look.add(1)` is at worst it.
            look = unsafe { look.add(1) };
            // SAFETY: `look` still points inside the option string.
            c = unsafe { *look as u8 };
            word
        } else {
            false
        };

        // SAFETY: the `^` in front of `look.add(1)` is what says that byte is
        // inside the option string, and the chain is left whole so that it
        // keeps doing so.
        if c == b'^' && unsafe { (b'?'..=b'_').contains(&(*look.add(1) as u8)) } {
            // A control character, spelled `^X`.
            // `CTRL_CHR(x)` is `TOUPPER_ASC(x) ^ 0x40`; the guarded range
            // holds no lower-case letter, so the fold is a no-op there.
            // SAFETY: `look.add(1)` is inside the option string, as above.
            let ctrl = unsafe { *look.add(1) as u8 }.to_ascii_uppercase() ^ 0x40;
            if try_match && keytyped == c_int::from(ctrl) {
                return true;
            }
            // SAFETY: neither `look[0]` nor `look[1]` is the NUL, so
            // `look.add(2)` is at worst it.
            look = unsafe { look.add(2) };
        } else if c == b'o' {
            // The "o" command: open a line forward.
            if try_match && keytyped == KEY_OPEN_FORW {
                return true;
            }
            // SAFETY: `c` is not the NUL, so `look.add(1)` is at worst it.
            look = unsafe { look.add(1) };
        } else if c == b'O' {
            // The "O" command: open a line backward.
            if try_match && keytyped == KEY_OPEN_BACK {
                return true;
            }
            // SAFETY: as above.
            look = unsafe { look.add(1) };
        } else if c == b'e' {
            // Check for "else" at the start of the line and just before
            // the cursor.
            if try_match && keytyped == c_int::from(b'e') && cur_win().w_cursor.col >= 4 {
                // SAFETY: the cursor is on a line of the current buffer and
                // `get_cursor_line_ptr` hands back a NUL-terminated one; the
                // `col >= 4` test the `&&` chain keeps in front is what says
                // `col - 4` is a byte of it.
                let is_else = unsafe {
                    let p = get_cursor_line_ptr();
                    let at = p.offset((cur_win().w_cursor.col - 4) as isize).cast_const();
                    skipwhite(p).cast_const() == at
                        && CStr::from_ptr(at).to_bytes().starts_with(b"else")
                };
                if is_else {
                    return true;
                }
            }
            // SAFETY: as above.
            look = unsafe { look.add(1) };
        } else if c == b':' {
            // SAFETY: reads the cursor's line of the current buffer; the two
            // cheap tests are kept in front of it by the `&&` chain.
            if try_match && keytyped == c_int::from(b':') && unsafe { colon_reindents() } {
                return true;
            }
            // SAFETY: as above.
            look = unsafe { look.add(1) };
        } else if c == b'<' {
            if try_match {
                // Some made-up named keys -- <o>, <O>, <e>, <0>, <>>,
                // <<>, <*>, <:> and <!> -- so that o, O, e, 0, <, >, *, :
                // and ! can be re-indent keys for anyone who wants them.
                // SAFETY: `c` is not the NUL, so `look.add(1)` is at worst it,
                // and what follows it is still a NUL-terminated string.
                let named = unsafe { *look.add(1) as u8 };
                if b"<>!*oOe0:".contains(&named) && keytyped == c_int::from(named) {
                    return true;
                }
                // SAFETY: as above.
                if keytyped == unsafe { get_special_key_code(look.add(1)) } {
                    return true;
                }
            }
            // SAFETY: `look` walks the option string and both loops stop at
            // its NUL -- pure pointer work, so one region around it is as
            // tight as this gets.
            while unsafe { *look } != 0 && unsafe { *look } as u8 != b'>' {
                look = unsafe { look.add(1) };
            }
            while unsafe { *look } as u8 == b'>' {
                look = unsafe { look.add(1) };
            }
        // SAFETY: the `=` in front of `look.add(1)` is what says that byte is
        // inside the option string; the chain is left whole.
        } else if c == b'=' && unsafe { *look.add(1) as u8 != b',' && *look.add(1) != 0 } {
            // "=word": the key is the last character of a word.
            // SAFETY: neither `look[0]` nor `look[1]` is the NUL.
            look = unsafe { look.add(1) };
            // SAFETY: `look` points inside the option string.
            let icase = unsafe { *look as u8 } == b'~';
            if icase {
                // SAFETY: the `~` is not the NUL, so `look.add(1)` is at worst
                // it.
                look = unsafe { look.add(1) };
            }
            // SAFETY: `look` points at a NUL-terminated option string, which
            // is all `vim_strchr` and `strlen` ask for; `end` lands inside it.
            let end = unsafe {
                let comma = vim_strchr(look, c_int::from(b','));
                if comma.is_null() {
                    look.add(cstr::bytes_at(look).len())
                } else {
                    comma
                }
            };
            // SAFETY: `end` and `look` point into the same option string.
            let len = unsafe { end.offset_from(look) } as usize;
            if (try_match || try_match_word)
                && cur_win().w_cursor.col >= len as colnr_T
                // SAFETY: `look` has `len` bytes in front of it, and the
                // column test the `&&` chain keeps in front is what says the
                // cursor's line has `len` bytes behind it.
                && unsafe { word_matches(keytyped, look, len, icase, try_match, try_match_word) }
            {
                return true;
            }
            look = end;
        } else {
            // A boring generic character.
            if try_match && c_int::from(c) == keytyped {
                return true;
            }
            if c != 0 {
                // SAFETY: `c` is not the NUL, so `look.add(1)` is at worst it.
                look = unsafe { look.add(1) };
            }
        }

        // SAFETY: `look` points inside the NUL-terminated option string.
        look = unsafe { skip_to_option_part(look) };
    }
    false
}

/// Whether a typed `:` ends a label, a `case` or a scope declaration -- the
/// `:` item of 'cinkeys'.
///
/// `class::method` is the shape this must not fire on, so when the two
/// characters before the cursor are `::` the test is repeated with the first
/// of them blanked out: if the line *only* looks like a label because of the
/// second colon, it is not one.  The line is written to and restored, which
/// is why it has to be re-fetched around every call that may unlock it.
///
/// # Safety
/// Reads and temporarily writes the cursor line.
unsafe fn colon_reindents() -> bool {
    // SAFETY: the cursor is on a line of the current buffer, and
    // `get_cursor_line_ptr` hands back a NUL-terminated one -- which is all
    // the three recognisers ask for.
    let is_label = unsafe {
        let p = get_cursor_line_ptr();
        cin_iscase(p, false) || cin_isscopedecl(p) || cin_islabel()
    };
    if is_label {
        return true;
    }
    // `cin_islabel` may have unlocked the line.
    // SAFETY: as above.
    let mut p = get_cursor_line_ptr();
    let col = cur_win().w_cursor.col as isize;
    // SAFETY: `col > 2` -- which the `||` chain keeps in front -- says that
    // `col - 1` and `col - 2` are bytes of the cursor's line.
    if col <= 2 || unsafe { *p.offset(col - 1) as u8 != b':' || *p.offset(col - 2) as u8 != b':' } {
        return false;
    }
    // SAFETY: `col - 1` is a byte of the cursor's own line, ours to blank out
    // while the recognisers read it; it is put back below.
    let looks_like_one = unsafe {
        *p.offset(col - 1) = b' ' as c_char;
        cin_iscase(p, false) || cin_isscopedecl(p) || cin_islabel()
    };
    // SAFETY: the recognisers may have unlocked the line, so it is fetched
    // again before the colon goes back.
    p = get_cursor_line_ptr();
    unsafe { *p.offset(col - 1) = b':' as c_char };
    looks_like_one
}

/// Whether the `=word` form of a 'cinkeys' item matches what was just typed.
///
/// There are two ways in: [`KEY_COMPLETE`], where a whole word was just
/// completed and the check is on the word behind the cursor, and an ordinary
/// key, where the typed character must be the word's *last* one and the text
/// before the cursor must be the rest of it.
///
/// # Safety
/// Reads the cursor line.
unsafe fn word_matches(
    keytyped: c_int,
    look: *const c_char,
    len: usize,
    icase: bool,
    try_match: bool,
    try_match_word: bool,
) -> bool {
    // A closure, not a free function: two call sites in one body, so it
    // inherits this body's locals rather than needing parameters of its own.
    let same = |a: *const c_char, b: *const c_char| {
        // SAFETY: local to this body and reached only from the two sites
        // below, each of which has just established that `a` and `b` have
        // `len` readable bytes in front of them.
        if icase {
            unsafe { mb_strnicmp(a, b, len) == 0 }
        } else {
            unsafe { cstr::prefix_eq(a, b, len) }
        }
    };

    let matched = if keytyped == KEY_COMPLETE {
        // A word was just completed: search back for its start and check
        // that it begins with `word`.
        // SAFETY: the cursor is on a line of the current buffer, at `col`
        // bytes into it; `mb_prevptr` walks back inside that same line, and
        // `same` is asked only once `s.add(len)` is known to be within it --
        // the `&&` chain is left whole so that it keeps being so.
        let line = get_cursor_line_ptr();
        let mut s = unsafe { line.offset(cur_win().w_cursor.col as isize) };
        while s > line {
            let n = unsafe { mb_prevptr(line, s) };
            if !unsafe { vim_iswordp(n) } {
                break;
            }
            s = n;
        }
        unsafe { s.add(len) <= line.offset(cur_win().w_cursor.col as isize) && same(s, look) }
    } else {
        // TODO(@brammool): multi-byte.
        // `look[len - 1]` is upstream's `p[-1]`, read off the *end* of
        // the item: with `cinkeys==~,` the word is empty and that byte is
        // the `~` in front of it, which is still inside the option.
        // SAFETY: the caller's promise -- `look` has `len` bytes of the
        // option string in front of it -- plus that note for `len == 0`.
        // `tolower` is the C library's own.
        let mismatch = unsafe {
            let last = *look.add(len).sub(1) as u8;
            keytyped != c_int::from(last)
                && !(icase
                    && (0..256).contains(&keytyped)
                    && tolower(keytyped) == tolower(c_int::from(last)))
        };
        if mismatch {
            return false;
        }
        // SAFETY: the cursor is on a line of the current buffer; the caller
        // has established that `col >= len`, so the `len + 1` bytes behind it
        // are on that line -- except when `col == len`, which the `||` chain
        // keeps in front of the read and which is left whole for that reason.
        let line = get_cursor_pos_ptr();
        (cur_win().w_cursor.col == len as colnr_T
            || !unsafe { vim_iswordc(c_int::from(*line.sub(len + 1) as u8)) })
            && same(unsafe { line.sub(len) }, look)
    };

    // "0=word" also requires that only blanks precede the word.
    if matched && try_match_word && !try_match {
        // SAFETY: reads the cursor's line of the current buffer.
        let white = unsafe { getwhitecols_curline() };
        return white == (cur_win().w_cursor.col as isize) - len as isize;
    }
    matched
}

/// Reindent the current line with 'indentexpr' or the C indent.
///
/// # Safety
/// Reads the current buffer and rewrites the current line.
pub unsafe fn do_c_expr_indent() {
    // SAFETY: 'indentexpr' is a NUL-terminated option string.
    if unsafe { *cur_buf().b_p_inde } != 0 {
        // SAFETY: rewrites the current line of the current buffer.
        unsafe { fixthisline(Some(get_expr_indent)) };
    } else {
        // SAFETY: the same.
        unsafe { fixthisline(Some(get_c_indent)) };
    }
}

/// `cindent(lnum)`: what `get_c_indent` would answer for line `lnum`, or -1
/// when the line is out of range.
///
/// # Safety
/// Moves the cursor and restores it; `rettv` must be a valid number typval.
pub unsafe fn f_cindent(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let pos = cur_win().w_cursor;
    // SAFETY: the caller's promise -- `argvars` is the call's argument list.
    let lnum = unsafe { tv_get_lnum(argvars) } as linenr_T;
    let amount = if lnum >= 1 && lnum <= cur_buf().b_ml.ml_line_count {
        cur_win().w_cursor.lnum = lnum;
        // SAFETY: the cursor now sits on a line of the current buffer, and it
        // is put back on the next line.
        let amount = varnumber_T::from(unsafe { get_c_indent() });
        cur_win().w_cursor = pos;
        amount
    } else {
        -1
    };
    // SAFETY: the caller's promise -- `rettv` is a number typval to fill in.
    unsafe { (*rettv).vval.v_number = amount };
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
