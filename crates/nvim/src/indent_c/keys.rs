//! When to reindent: 'cinkeys', and the front doors.
//!
//! [`in_cinkeys`] answers whether a typed character should trigger a
//! reindent, which is the whole of 'cinkeys': a comma-separated list of keys,
//! each optionally prefixed by `*` (reindent *before* inserting), `!` (never
//! insert, just reindent) or `0` (only when it is the first thing on the
//! line), plus the `o`/`O`/`e`/`=` word forms.  [`cindent_on`] is the "is C
//! indenting active at all" test 'cindent'/'indentexpr' share, and
//! [`f_cindent`] is `cindent()`.

#![forbid(unsafe_code)]

use super::*;
use crate::charset::is_word_at;
use crate::mbyte::{head_off, strnicmp_in};
use crate::option::skip_option_separator;
use crate::optionstr::OptStringRef;
use crate::os::cshim::byte_tolower;
use crate::types::Indenter;
use crate::types::NUL;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

/// Whether C indenting is on: `'cindent'` or a non-empty `'indentexpr'`, and
/// not `'paste'`.
pub(crate) fn cindent_on() -> bool {
    // The cheaper tests are kept in front, as upstream has them.
    !p_paste() && (Buf::current().b_p_cin != 0 || !Buf::current().b_p_inde.bytes().is_empty())
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
pub fn in_cinkeys(keytyped: c_int, when: c_int, line_is_empty: bool) -> bool {
    if keytyped == NUL {
        // Can happen with CTRL-Y and CTRL-E on a short line.
        return false;
    }

    // 'indentexpr' set means 'indentkeys' rather than 'cinkeys'.
    //
    // The walk below reads the option's own bytes, as upstream's does;
    // nothing between here and the end of it can set an option.
    let buf = Buf::current();
    let look: &[u8] = if !buf.b_p_inde.bytes().is_empty() {
        buf.b_p_indk.bytes()
    } else {
        buf.b_p_cink.bytes()
    };
    // Past the end reads the NUL a C string has there.
    let at = |i: usize| byte_at(look, i);

    let mut i = 0;
    loop {
        let mut c = at(i);
        if c == 0 {
            break;
        }
        let mut try_match = wants(when, c);
        if c == b'*' || c == b'!' {
            i += 1;
            c = at(i);
        }

        // A '0' means "only when the line is empty" -- but the word forms
        // below may still match on the last character of the word.
        let try_match_word = if c == b'0' {
            let word = try_match;
            try_match &= line_is_empty;
            i += 1;
            c = at(i);
            word
        } else {
            false
        };

        if c == b'^' && (b'?'..=b'_').contains(&at(i + 1)) {
            // A control character, spelled `^X`.
            // `CTRL_CHR(x)` is `TOUPPER_ASC(x) ^ 0x40`; the guarded range
            // holds no lower-case letter, so the fold is a no-op there.
            let ctrl = at(i + 1).to_ascii_uppercase() ^ 0x40;
            if try_match && keytyped == c_int::from(ctrl) {
                return true;
            }
            i += 2;
        } else if c == b'o' {
            // The "o" command: open a line forward.
            if try_match && keytyped == KEY_OPEN_FORW {
                return true;
            }
            i += 1;
        } else if c == b'O' {
            // The "O" command: open a line backward.
            if try_match && keytyped == KEY_OPEN_BACK {
                return true;
            }
            i += 1;
        } else if c == b'e' {
            // Check for "else" at the start of the line and just before
            // the cursor.
            if try_match && keytyped == c_int::from(b'e') && Win::current().w_cursor.col >= 4 {
                let back = usize::try_from(Win::current().w_cursor.col - 4).unwrap_or(0);
                let cursor_lnum = Win::current().w_cursor.lnum;
                let mut lines = Lines::current();
                let text = lines.line(cursor_lnum);
                let is_else =
                    skip::white(text) == back && text[back.min(text.len())..].starts_with(b"else");
                if is_else {
                    return true;
                }
            }
            i += 1;
        } else if c == b':' {
            // The two cheap tests are kept in front of the line read.
            if try_match && keytyped == c_int::from(b':') && colon_reindents() {
                return true;
            }
            i += 1;
        } else if c == b'<' {
            if try_match {
                // Some made-up named keys -- <o>, <O>, <e>, <0>, <>>,
                // <<>, <*>, <:> and <!> -- so that o, O, e, 0, <, >, *, :
                // and ! can be re-indent keys for anyone who wants them.
                let named = at(i + 1);
                if b"<>!*oOe0:".contains(&named) && keytyped == c_int::from(named) {
                    return true;
                }
                if keytyped == special_key_code(&look[i + 1..]) {
                    return true;
                }
            }
            while at(i) != 0 && at(i) != b'>' {
                i += 1;
            }
            while at(i) == b'>' {
                i += 1;
            }
        } else if c == b'=' && at(i + 1) != b',' && at(i + 1) != 0 {
            // "=word": the key is the last character of a word.
            i += 1;
            let icase = at(i) == b'~';
            if icase {
                i += 1;
            }
            let end = look[i..]
                .iter()
                .position(|&b| b == b',')
                .map_or(look.len(), |comma| i + comma);
            let len = end - i;
            if (try_match || try_match_word)
                && Win::current().w_cursor.col >= len as ColNr
                && word_matches(keytyped, look, i..end, icase, try_match, try_match_word)
            {
                return true;
            }
            i = end;
        } else {
            // A boring generic character.
            if try_match && c_int::from(c) == keytyped {
                return true;
            }
            if c != 0 {
                i += 1;
            }
        }

        i = look.len() - skip_option_separator(&look[i.min(look.len())..]).len();
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
fn colon_reindents() -> bool {
    let lnum = Win::current().w_cursor.lnum;
    // The chain re-enters at `is_jump_label`, so the borrow the two tests in
    // front of it take is dropped before it runs.
    let labelled = |lnum: LineNr| {
        let claimed = {
            let mut lines = Lines::current();
            let line = lines.line(lnum);
            is_case_label(line, 0, false) || is_scope_decl(line, 0)
        };
        claimed || is_jump_label()
    };
    if labelled(lnum) {
        return true;
    }

    let col = usize::try_from(Win::current().w_cursor.col).unwrap_or(0);
    let double_colon = col > 2 && {
        let mut lines = Lines::current();
        let line = lines.line(lnum);
        byte_at(line, col - 1) == b':' && byte_at(line, col - 2) == b':'
    };
    if !double_colon {
        return false;
    }

    // Blank the first colon out while the recognisers read the line, then
    // put it back -- in place and unbooked, as upstream writes through
    // `ml_get`'s pointer: [`Lines::line_mut`] would book the old text as
    // deleted, an undo-relevant side effect this probe must not have. The
    // recognisers may have unlocked the line, so the restore fetches it
    // again.
    Lines::current().swap_byte_unbooked(lnum, col - 1, b' ');
    let looks_like_one = labelled(lnum);
    Lines::current().swap_byte_unbooked(lnum, col - 1, b':');
    looks_like_one
}

/// Whether the `=word` form of a 'cinkeys' item matches what was just typed:
/// the word is `look[word]`.
///
/// There are two ways in: [`KEY_COMPLETE`], where a whole word was just
/// completed and the check is on the word behind the cursor, and an ordinary
/// key, where the typed character must be the word's *last* one and the text
/// before the cursor must be the rest of it.
///
/// The caller has established that the cursor column is at least the word's
/// length.
fn word_matches(
    keytyped: c_int,
    look: &[u8],
    word: core::ops::Range<usize>,
    icase: bool,
    try_match: bool,
    try_match_word: bool,
) -> bool {
    let end = word.end;
    let word = &look[word];
    let len = word.len();
    // Whether `text` starts with the word; `text` has at least `len` bytes.
    let same = |text: &[u8]| {
        let text = &text[..len.min(text.len())];
        if icase {
            strnicmp_in(text, word) == 0
        } else {
            text == word
        }
    };
    let cursor = Win::current().w_cursor;
    let col = usize::try_from(cursor.col).unwrap_or(0);

    let matched = if keytyped == KEY_COMPLETE {
        // A word was just completed: search back for its start and check
        // that it begins with `word`.
        let mut lines = Lines::current();
        let line = lines.line(cursor.lnum);
        let col = col.min(line.len());
        let mut s = col;
        while s > 0 {
            let n = s - 1 - head_off(line, s - 1);
            if !is_word_at(&line[n..], Buf::current()) {
                break;
            }
            s = n;
        }
        s + len <= col && same(&line[s..])
    } else {
        // TODO(@brammool): multi-byte.
        // `look[end - 1]` is upstream's `p[-1]`, read off the *end* of
        // the item: with `cinkeys==~,` the word is empty and that byte is
        // the `~` in front of it, which is still inside the option.
        let last = look[end - 1];
        let mismatch = keytyped != c_int::from(last)
            && !(icase
                && (0..256).contains(&keytyped)
                && byte_tolower(keytyped) == byte_tolower(c_int::from(last)));
        if mismatch {
            return false;
        }
        // The caller has established `col >= len`, so the `len + 1` bytes
        // behind the cursor are on its line -- except when `col == len`,
        // which the `||` keeps in front of the read.
        let mut lines = Lines::current();
        let line = lines.line(cursor.lnum);
        (col == len || !vim_iswordc(c_int::from(byte_at(line, col - len - 1))))
            && same(line.get(col - len..).unwrap_or_default())
    };

    // "0=word" also requires that only blanks precede the word.
    if matched && try_match_word && !try_match {
        let white = getwhitecols_curline();
        return white == (Win::current().w_cursor.col as isize) - len as isize;
    }
    matched
}

/// Reindent the current line with 'indentexpr' or the C indent.
pub fn do_c_expr_indent() {
    if !Buf::current().b_p_inde.bytes().is_empty() {
        fixthisline(Indenter::Expr);
    } else {
        fixthisline(Indenter::C);
    }
}

/// `cindent(lnum)`: what `get_c_indent` would answer for line `lnum`, or -1
/// when the line is out of range.
pub fn f_cindent(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let pos = Win::current().w_cursor;
    let lnum = tv_get_lnum(&args[0]) as LineNr;
    let amount = if lnum >= 1 && lnum <= Buf::current().b_ml.ml_line_count {
        Win::current().w_cursor.lnum = lnum;
        let amount = VarNumber::from(get_c_indent());
        Win::current().w_cursor = pos;
        amount
    } else {
        -1
    };
    // SAFETY: the caller's promise -- `result` is a number typval to fill in.
    result.write_number(amount);
}
