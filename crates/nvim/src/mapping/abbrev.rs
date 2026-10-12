//! Abbreviations and `<expr>` right-hand sides.
//!
//! [`check_abbr`] is called after every inserted character: it looks back for
//! a word matching an entry on the abbrlist and, on a match, pushes the
//! deletions and the replacement into the typeahead.  [`eval_map_expr`]
//! evaluates an `<expr>` mapping's RHS, which both this and the mapping match
//! need.

#![forbid(unsafe_code)]

use super::*;
use crate::charset::is_word_at;
use crate::getchar::{ins_typebuf, typeahead};
use crate::guard::Lock;
use crate::keycodes::{Ctrl_RSB, Ctrl_V, key_escape};
use crate::lua::executor::nlua_call_ref_reporting;
use crate::mbyte::head_off;
use crate::message_fmt::msg_cstr;
use crate::semsg_multiline;
use crate::types::NUL;
use crate::winlayer::Buf;
use crate::winlayer::Win;
use core::ffi::{CStr, c_int};

/// Whether `mp`'s LHS is exactly `word`, in a mode that is current.
///
/// The stored LHS may carry `K_SPECIAL` escapes, which the text in the buffer
/// does not, so it is unescaped into a scratch copy first.
fn abbr_matches(mp: &MapBlock, word: &[u8]) -> bool {
    if mp.m_mode & State.get() == 0 {
        return false;
    }
    let keys = mp.keys();
    if !keys.contains(&(K_SPECIAL as u8)) {
        return keys == word;
    }
    unescape_ks(keys) == word
}

/// What an `<expr>` mapping or abbreviation evaluates, copied out of the
/// entry before any of it runs: the evaluation is user code, which can
/// delete or redefine the entry.
pub(crate) struct MapExpr {
    /// The RHS with its `K_SPECIAL` escaping removed: `m_str` is in the
    /// format used for typeahead, not the one the expression is written in.
    text: Vec<u8>,
    /// The Lua callback, held for the call.
    lua: Option<Rc<MapCallback>>,
    replace_keycodes: bool,
}

impl MapExpr {
    /// The expression `mp` evaluates.
    pub(crate) fn of(mp: &MapBlock) -> Self {
        let mut text = mp.rhs().to_vec();
        let len = unescape_ks_in(&mut text);
        text.truncate(len);
        Self {
            text,
            lua: mp.m_rhs.lua.clone(),
            replace_keycodes: mp.m_replace_keycodes,
        }
    }
}

/// What [`check_abbr`] needs of the abbreviation it found, copied out of the
/// table: an `<expr>` one can delete itself.
struct Found {
    noremap: c_int,
    silent: bool,
    /// The expression of an `<expr>` abbreviation, or else the RHS.
    expr: Option<MapExpr>,
    rhs: MapStr,
}

/// The start of the character before byte `at` of `text` -- C's
/// `mb_prevptr`.
fn prev(text: &[u8], at: usize) -> usize {
    at - 1 - head_off(text, at - 1)
}

/// Check for an abbreviation ending at the end of `text` and, if there is
/// one, feed its expansion into the typeahead.
///
/// `text` is a copy of the line up to the cursor: the expansion can run user
/// code, which may change the line it came from.  When inserting, `mincol`
/// is where the insert started; on the command line it is what is to be
/// skipped over.  `c` is the character typed just before the call, possibly
/// with `ABBR_OFF` added to say that it needs no CTRL-V.
///
/// Historic vi practice: the last character of an abbreviation must be an id
/// character, and the ones in front of it must be all id characters or all
/// non-id characters, which is what lets `#i` stand for `#include`.  Vim adds
/// abbreviations ending in a non-keyword character, which instead need white
/// space in front.
pub fn check_abbr(c: c_int, text: Vec<u8>, mincol: c_int) -> bool {
    if typeahead().no_abbr_cnt() != 0 {
        return false; // abbreviations are not recursive
    }
    // No remapping implies no abbreviation, except for CTRL-].
    if noremap_keys() && c != Ctrl_RSB {
        return false;
    }
    let col = text.len();
    if col == 0 {
        return false; // cannot be an abbreviation
    }

    // Look back for the word before the cursor. If it ends in a keyword
    // character every character before it must be a keyword character or
    // all of them must not be, but never white space; if it ends in a
    // non-keyword character anything but white space is accepted.
    let buffer = Buf::current();
    let is_word = |at: usize| is_word_at(&text[at..], buffer);
    let start = usize::try_from(mincol).unwrap_or(0);
    let mut clen = 1; // length of the word in characters
    let mut p = prev(&text, col);
    let vim_abbr = !is_word(p);
    let mut is_id = true;
    if !vim_abbr && p > 0 {
        is_id = is_word(prev(&text, p));
    }
    while p > start {
        p = prev(&text, p);
        let stop = ascii_isspace(c_int::from(text[p])) || (!vim_abbr && is_id != is_word(p));
        if stop {
            p += cluster_len(&text[p..]);
            break;
        }
        clen += 1;
    }
    let scol = p.max(start);
    if scol >= col {
        return false; // no word in front of the cursor
    }

    // Buffer-local abbreviations first, then the global ones.
    let word = &text[scol..];
    let snapshot = |mp: &MapBlock| {
        abbr_matches(mp, word).then(|| Found {
            noremap: mp.m_noremap,
            silent: mp.m_silent,
            expr: mp.m_expr.then(|| MapExpr::of(mp)),
            rhs: if mp.m_expr {
                MapStr::empty()
            } else {
                mp.m_rhs.str.clone()
            },
        })
    };
    let found = [MapTable::Buffer(buffer), MapTable::Global]
        .into_iter()
        .find_map(|table| map_walk(table, true, snapshot));
    let Some(found) = found else {
        return false;
    };

    // Found one. Insert the rest of the abbreviation into the typeahead,
    // from the end to the start.
    //
    // Characters 0x00-0xff are normal and may need a CTRL-V, except
    // K_SPECIAL, which becomes K_SPECIAL KS_SPECIAL KE_FILLER; a key code
    // needs its K_SPECIAL escape; anything carrying ABBR_OFF wants no
    // CTRL-V. CTRL-] is special: it completes the abbreviation but is not
    // inserted into the input stream.
    let mut typed = Vec::new();
    if c != Ctrl_RSB {
        if c < 0 || c == K_SPECIAL {
            // A special key code, split up.
            typed.extend_from_slice(&key_escape(c));
        } else {
            let mut c = c;
            if c < ABBR_OFF && !(c_int::from(b' ')..=c_int::from(b'~')).contains(&c) {
                typed.push(Ctrl_V as u8); // a special char needs CTRL-V
            }
            if c >= ABBR_OFF {
                c -= ABBR_OFF; // remove the ABBR_OFF added by the caller
            }
            // The character, with K_SPECIAL escaped.
            push_char_escaped(c, &mut typed);
        }
        typed.push(NUL as u8);
        // Insert the last typed char.
        let keys = CStr::from_bytes_until_nul(&typed).expect("terminated just above");
        let _ = ins_typebuf(keys.to_bytes(), 1, 0, true, found.silent);
        typed.pop();
    }

    // An `<expr>` evaluation can redefine -- and so free -- the abbreviation,
    // which is why everything it needs was copied out above.
    let evaluated = found.expr.as_ref().map(|expr| eval_map_expr(expr, c));
    let to = match evaluated {
        Some(evaluated) => evaluated,
        None => Some(found.rhs),
    };
    if let Some(s) = to {
        // Insert the "to" string.
        let _ = ins_typebuf(s.as_bytes(), found.noremap, 0, true, found.silent);
        // No abbreviation for these chars.
        let count = s.len() + typed.len() + 1;
        typeahead().add_no_abbr_cnt(c_int::try_from(count).unwrap_or(c_int::MAX));
    }

    // Delete the "from" string: characters, not bytes.
    for _ in 0..clen {
        let _ = ins_typebuf(b"\x08", 1, 0, true, found.silent); // CTRL-H
    }
    true
}

/// Evaluate the RHS of an `<expr>` mapping or abbreviation, escaping the
/// result so it can be used as typeahead.
///
/// `c` is NUL, or the typed character for an abbreviation.
pub(crate) fn eval_map_expr(expr: &MapExpr, c: c_int) -> Option<MapStr> {
    // Forbid changing text or using ":normal", which rules out most of the bad
    // side effects, and restore the cursor position afterwards.
    let locked = Lock::expr_map();
    set_vim_var_char(c); // set v:char to the typed character
    let save_cursor = Win::current().w_cursor;
    let save_msg_col = msg_col.get();
    let save_msg_row = msg_row.get();

    let answer: Option<Vec<u8>> = if let Some(lua) = &expr.lua {
        match nlua_call_ref_reporting(lua.0, None, ARRAY_DICT_INIT, kRetObject) {
            Ok(ret) => ret.as_string().map(|s| c_prefix(s.as_bytes()).to_vec()),
            Err(mut err) => {
                semsg_multiline!(c"emsg", "E5108: {}", msg_cstr(err.message_or_empty()));
                err.clear();
                None
            }
        }
    } else {
        // `text` is the unescaped copy made before any of this ran: what user
        // code does to the mapping cannot reach it.
        let value = eval_to_string(c_prefix(&expr.text), false, false);
        value.map(|value| value.as_cstr().to_bytes().to_vec())
    };

    drop(locked);
    Win::current().w_cursor = save_cursor;
    msg_col.set(save_msg_col);
    msg_row.set(save_msg_row);

    let bytes = answer?;
    if !expr.replace_keycodes {
        // Escape K_SPECIAL so the result can be used as typeahead.
        return Some(MapStr::new(&escape_ks_bytes(&bytes)));
    }
    let dolt = REPTERM_DO_LT.cast_signed();
    let replaced = p_cpo(|cpo| replace_termcodes(&bytes, 0, dolt, None, cpo, None));
    let replaced = replaced.unwrap_or_default();
    Some(MapStr::new(c_prefix(&replaced)))
}

/// The bytes a C reader of `bytes` sees: up to the first NUL.
fn c_prefix(bytes: &[u8]) -> &[u8] {
    bytes.split(|&b| b == 0).next().unwrap_or_default()
}
