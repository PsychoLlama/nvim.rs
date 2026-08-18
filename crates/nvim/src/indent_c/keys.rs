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
use core::ffi::{CStr, c_char, c_int};

/// Whether C indenting is on: `'cindent'` or a non-empty `'indentexpr'`, and
/// not `'paste'`.
///
/// # Safety
/// Reads the current buffer.
pub unsafe fn cindent_on() -> bool {
    unsafe {
        p_paste.get() == 0 && ((*curbuf.get()).b_p_cin != 0 || *(*curbuf.get()).b_p_inde != 0)
    }
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
    unsafe {
        if keytyped == NUL {
            // Can happen with CTRL-Y and CTRL-E on a short line.
            return false;
        }

        // 'indentexpr' set means 'indentkeys' rather than 'cinkeys'.
        let mut look = if *(*curbuf.get()).b_p_inde != 0 {
            (*curbuf.get()).b_p_indk
        } else {
            (*curbuf.get()).b_p_cink
        };

        while *look != 0 {
            let mut try_match = wants(when, *look as u8);
            if *look as u8 == b'*' || *look as u8 == b'!' {
                look = look.add(1);
            }

            // A '0' means "only when the line is empty" -- but the word forms
            // below may still match on the last character of the word.
            let try_match_word = if *look as u8 == b'0' {
                let word = try_match;
                try_match &= line_is_empty;
                look = look.add(1);
                word
            } else {
                false
            };

            if *look as u8 == b'^' && (b'?'..=b'_').contains(&(*look.add(1) as u8)) {
                // A control character, spelled `^X`.
                // `CTRL_CHR(x)` is `TOUPPER_ASC(x) ^ 0x40`; the guarded range
                // holds no lower-case letter, so the fold is a no-op there.
                if try_match
                    && keytyped == c_int::from((*look.add(1) as u8).to_ascii_uppercase() ^ 0x40)
                {
                    return true;
                }
                look = look.add(2);
            } else if *look as u8 == b'o' {
                // The "o" command: open a line forward.
                if try_match && keytyped == KEY_OPEN_FORW {
                    return true;
                }
                look = look.add(1);
            } else if *look as u8 == b'O' {
                // The "O" command: open a line backward.
                if try_match && keytyped == KEY_OPEN_BACK {
                    return true;
                }
                look = look.add(1);
            } else if *look as u8 == b'e' {
                // Check for "else" at the start of the line and just before
                // the cursor.
                if try_match && keytyped == c_int::from(b'e') && (*curwin.get()).w_cursor.col >= 4 {
                    let p = get_cursor_line_ptr();
                    let at = p
                        .offset(((*curwin.get()).w_cursor.col - 4) as isize)
                        .cast_const();
                    if skipwhite(p).cast_const() == at
                        && CStr::from_ptr(at).to_bytes().starts_with(b"else")
                    {
                        return true;
                    }
                }
                look = look.add(1);
            } else if *look as u8 == b':' {
                if try_match && keytyped == c_int::from(b':') && colon_reindents() {
                    return true;
                }
                look = look.add(1);
            } else if *look as u8 == b'<' {
                if try_match {
                    // Some made-up named keys -- <o>, <O>, <e>, <0>, <>>,
                    // <<>, <*>, <:> and <!> -- so that o, O, e, 0, <, >, *, :
                    // and ! can be re-indent keys for anyone who wants them.
                    let named = *look.add(1) as u8;
                    if b"<>!*oOe0:".contains(&named) && keytyped == c_int::from(named) {
                        return true;
                    }
                    if keytyped == get_special_key_code(look.add(1)) {
                        return true;
                    }
                }
                while *look != 0 && *look as u8 != b'>' {
                    look = look.add(1);
                }
                while *look as u8 == b'>' {
                    look = look.add(1);
                }
            } else if *look as u8 == b'=' && *look.add(1) as u8 != b',' && *look.add(1) != 0 {
                // "=word": the key is the last character of a word.
                look = look.add(1);
                let icase = *look as u8 == b'~';
                if icase {
                    look = look.add(1);
                }
                let end = vim_strchr(look, c_int::from(b','));
                let end = if end.is_null() {
                    look.add(strlen(look))
                } else {
                    end
                };
                let len = end.offset_from(look) as usize;
                if (try_match || try_match_word)
                    && (*curwin.get()).w_cursor.col >= len as colnr_T
                    && word_matches(keytyped, look, len, icase, try_match, try_match_word)
                {
                    return true;
                }
                look = end;
            } else {
                // A boring generic character.
                if try_match && c_int::from(*look as u8) == keytyped {
                    return true;
                }
                if *look != 0 {
                    look = look.add(1);
                }
            }

            look = skip_to_option_part(look);
        }
        false
    }
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
    unsafe {
        let mut p = get_cursor_line_ptr();
        if cin_iscase(p, false) || cin_isscopedecl(p) || cin_islabel() {
            return true;
        }
        // `cin_islabel` may have unlocked the line.
        p = get_cursor_line_ptr();
        let col = (*curwin.get()).w_cursor.col;
        let col = col as isize;
        if col <= 2 || *p.offset(col - 1) as u8 != b':' || *p.offset(col - 2) as u8 != b':' {
            return false;
        }
        *p.offset(col - 1) = b' ' as c_char;
        let looks_like_one = cin_iscase(p, false) || cin_isscopedecl(p) || cin_islabel();
        p = get_cursor_line_ptr();
        *p.offset(col - 1) = b':' as c_char;
        looks_like_one
    }
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
    unsafe {
        // A closure, not a free function: two call sites in one body, so it
        // inherits this block rather than needing one of its own.
        let same = |a: *const c_char, b: *const c_char| {
            if icase {
                mb_strnicmp(a, b, len) == 0
            } else {
                strncmp(a, b, len) == 0
            }
        };

        let matched = if keytyped == KEY_COMPLETE {
            // A word was just completed: search back for its start and check
            // that it begins with `word`.
            let line = get_cursor_line_ptr();
            let mut s = line.offset((*curwin.get()).w_cursor.col as isize);
            while s > line {
                let n = mb_prevptr(line, s);
                if !vim_iswordp(n) {
                    break;
                }
                s = n;
            }
            s.add(len) <= line.offset((*curwin.get()).w_cursor.col as isize) && same(s, look)
        } else {
            // TODO(@brammool): multi-byte.
            // `look[len - 1]` is upstream's `p[-1]`, read off the *end* of
            // the item: with `cinkeys==~,` the word is empty and that byte is
            // the `~` in front of it, which is still inside the option.
            let last = *look.add(len).sub(1) as u8;
            if keytyped != c_int::from(last)
                && !(icase
                    && (0..256).contains(&keytyped)
                    && tolower(keytyped) == tolower(c_int::from(last)))
            {
                return false;
            }
            let line = get_cursor_pos_ptr();
            ((*curwin.get()).w_cursor.col == len as colnr_T
                || !vim_iswordc(c_int::from(*line.sub(len + 1) as u8)))
                && same(line.sub(len), look)
        };

        // "0=word" also requires that only blanks precede the word.
        if matched && try_match_word && !try_match {
            return getwhitecols_curline()
                == ((*curwin.get()).w_cursor.col as isize) - len as isize;
        }
        matched
    }
}

/// Reindent the current line with 'indentexpr' or the C indent.
///
/// # Safety
/// Reads the current buffer and rewrites the current line.
pub unsafe fn do_c_expr_indent() {
    unsafe {
        if *(*curbuf.get()).b_p_inde != 0 {
            fixthisline(Some(get_expr_indent));
        } else {
            fixthisline(Some(get_c_indent));
        }
    }
}

/// `cindent(lnum)`: what `get_c_indent` would answer for line `lnum`, or -1
/// when the line is out of range.
///
/// # Safety
/// Moves the cursor and restores it; `rettv` must be a valid number typval.
pub unsafe fn f_cindent(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        let pos = (*curwin.get()).w_cursor;
        let lnum = tv_get_lnum(argvars) as linenr_T;
        (*rettv).vval.v_number = if lnum >= 1 && lnum <= (*curbuf.get()).b_ml.ml_line_count {
            (*curwin.get()).w_cursor.lnum = lnum;
            let amount = varnumber_T::from(get_c_indent());
            (*curwin.get()).w_cursor = pos;
            amount
        } else {
            -1
        };
    }
}
