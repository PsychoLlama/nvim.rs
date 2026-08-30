//! Abbreviations and `<expr>` right-hand sides.
//!
//! [`check_abbr`] is called after every inserted character: it looks back for
//! a word matching an entry on the abbrlist and, on a match, pushes the
//! deletions and the replacement into the typeahead.  [`eval_map_expr`]
//! evaluates an `<expr>` mapping's RHS, which both this and the mapping match
//! need.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::getchar::typeahead;
use crate::guard::Lock;
use crate::keycodes::{Ctrl_H, Ctrl_RSB, Ctrl_V, key_escape};
use crate::message_fmt::c_str;
use crate::semsg_multiline;
use crate::types::{MB_MAXBYTES, NUL};
use crate::winlayer::Buf;
use core::ffi::{c_char, c_int};
use core::ptr;

/// Whether `mp`'s LHS is exactly the `len` bytes at `word`, in a mode that is
/// current.
///
/// The stored LHS may carry `K_SPECIAL` escapes, which the text in the buffer
/// does not, so it is unescaped into a scratch copy first.
///
/// # Safety
/// `mp` must be a live mapblock and `word` `len` readable bytes.
unsafe fn abbr_matches(mp: Mb, word: *const c_char, len: c_int) -> bool {
    let keys = mp.m_keys;
    let mut qlen = mp.m_keylen;
    let mut q = keys;
    // SAFETY: `m_keys` is NUL-terminated; the unescaped copy is freed below.
    if !unsafe { strchr(keys, K_SPECIAL) }.is_null() {
        // SAFETY: as above.
        unsafe {
            q = xstrdup(keys);
            vim_unescape_ks(q);
            qlen = cstr::bytes_at(q).len() as c_int;
        }
    }
    // SAFETY: the caller's promise — `word` is readable for `len` bytes — and
    // `q` is NUL-terminated, so `strncmp` stops inside both.
    let matched = mp.m_mode & State.get() != 0
        && qlen == len
        && unsafe { cstr::prefix_eq(q, word, len as size_t) };
    if q != keys {
        // SAFETY: the scratch copy made above, freed once.
        unsafe { xfree(q.cast()) };
    }
    matched
}

/// Check for an abbreviation before `ptr[col]` and, if there is one, feed its
/// expansion into the typeahead.
///
/// When inserting, `mincol` is where the insert started; on the command line
/// it is what is to be skipped over.  `c` is the character typed just before
/// the call, possibly with `ABBR_OFF` added to say that it needs no CTRL-V.
///
/// Historic vi practice: the last character of an abbreviation must be an id
/// character, and the ones in front of it must be all id characters or all
/// non-id characters, which is what lets `#i` stand for `#include`.  Vim adds
/// abbreviations ending in a non-keyword character, which instead need white
/// space in front.
///
/// # Safety
/// `ptr` must be readable for at least `col` bytes and `curbuf`/`curwin` live.
pub unsafe fn check_abbr(c: c_int, ptr: *mut c_char, col: c_int, mincol: c_int) -> bool {
    if typeahead().no_abbr_cnt() != 0 {
        return false; // abbreviations are not recursive
    }
    // No remapping implies no abbreviation, except for CTRL-].
    if noremap_keys() && c != Ctrl_RSB {
        return false;
    }
    if col == 0 {
        return false; // cannot be an abbreviation
    }

    // Look back for the word before the cursor. If it ends in a keyword
    // character every character before it must be a keyword character or
    // all of them must not be, but never white space; if it ends in a
    // non-keyword character anything but white space is accepted.
    let mut clen = 1; // length of the word in characters
    let mut scol; // starting column of the abbreviation
    // SAFETY: the caller's promise — `ptr` is readable for `col` bytes, and
    // `mb_prevptr`/`utfc_ptr2len` stay inside `ptr..ptr+col` given that.
    unsafe {
        let mut is_id = true;
        let mut p = mb_prevptr(ptr, ptr.offset(col as isize));
        let vim_abbr = !vim_iswordp(p);
        if !vim_abbr && p > ptr {
            is_id = vim_iswordp(mb_prevptr(ptr, p));
        }
        while p > ptr.offset(mincol as isize) {
            p = mb_prevptr(ptr, p);
            if ascii_isspace(c_int::from(*p)) || (!vim_abbr && is_id != vim_iswordp(p)) {
                p = p.offset(utfc_ptr2len(p) as isize);
                break;
            }
            clen += 1;
        }
        scol = p.offset_from(ptr) as c_int;
    }
    if scol < mincol {
        scol = mincol;
    }
    if scol >= col {
        return false; // no word in front of the cursor
    }

    // SAFETY: `mincol <= scol < col`, so this stays inside the caller's text.
    let word = unsafe { ptr.offset(scol as isize) };
    let len = col - scol;
    // Buffer-local abbreviations first, then the global ones.
    let mut found = None;
    // SAFETY: `curbuf` is set from startup to exit.
    let cur = unsafe { Buf::current() };
    let matches = |mp: Mb| {
        // SAFETY: `word` names the `len` bytes located just above.
        unsafe { abbr_matches(mp, word, len) }.then_some(mp)
    };
    for table in [MapTable::Buffer(cur), MapTable::Global] {
        // SAFETY: the abbrlists are live, and `abbr_matches` only reads them.
        let hit = unsafe { map_walk(table, true, matches) };
        if hit.is_some() {
            found = hit;
            break;
        }
    }
    let Some(mp) = found else {
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
    let mut tb = [0u8; MB_MAXBYTES + 4];
    let mut j = 0usize;
    if c != Ctrl_RSB {
        if c < 0 || c == K_SPECIAL {
            // A special key code, split up.
            tb[..3].copy_from_slice(&key_escape(c));
            j = 3;
        } else {
            let mut c = c;
            if c < ABBR_OFF && !(c_int::from(b' ')..=c_int::from(b'~')).contains(&c) {
                tb[j] = Ctrl_V as u8; // a special char needs CTRL-V
                j += 1;
            }
            if c >= ABBR_OFF {
                c -= ABBR_OFF; // remove the ABBR_OFF added by the caller
            }
            // SAFETY: `j` is at most 1 here and `tb` has room for the longest
            // multi-byte character plus its NUL; `utf_char2bytes` writes no
            // more than that, and `vim_strsave_escape_ks`'s answer is at most
            // twice as long, which still fits.
            unsafe {
                let at = tb.as_mut_ptr().add(j).cast::<c_char>();
                let newlen = utf_char2bytes(c, at) as usize;
                tb[j + newlen] = NUL as u8;
                // Need to escape K_SPECIAL.
                let escaped = vim_strsave_escape_ks(at);
                if !escaped.is_null() {
                    let newlen = cstr::bytes_at(escaped).len() as usize;
                    ptr::copy(escaped.cast::<u8>(), tb.as_mut_ptr().add(j), newlen);
                    j += newlen;
                    xfree(escaped.cast());
                }
            }
        }
        tb[j] = NUL as u8;
        // Insert the last typed char.
        let keys = tb.as_mut_ptr().cast();
        let silent = mp.m_silent != 0;
        // SAFETY: `tb` is NUL-terminated at `j` and outlives the call, which
        // copies out of it.
        let _ = unsafe { ins_typebuf(keys, 1, 0, true, silent) };
    }

    // Copy the values out here: eval_map_expr() may make "mp" invalid.
    let noremap = mp.m_noremap;
    let silent = mp.m_silent != 0;
    let expr = mp.m_expr != 0;

    let s = if expr {
        // SAFETY: `mp` is still linked — nothing above can have run Vimscript.
        unsafe { eval_map_expr(mp, c) }
    } else {
        mp.m_str
    };
    if !s.is_null() {
        // Insert the "to" string.
        // SAFETY: `s` is NUL-terminated, either the mapping's own RHS or the
        // allocation `eval_map_expr` handed back, freed just below.
        unsafe {
            let _ = ins_typebuf(s, noremap, 0, true, silent);
            // No abbreviation for these chars.
            typeahead().add_no_abbr_cnt(cstr::bytes_at(s).len() as c_int + j as c_int + 1);
            if expr {
                xfree(s.cast());
            }
        }
    }

    // Delete the "from" string: characters, not bytes.
    tb[0] = Ctrl_H as u8;
    tb[1] = NUL as u8;
    let keys = tb.as_mut_ptr().cast();
    for _ in 0..clen {
        // SAFETY: as the `ins_typebuf` above — a NUL-terminated `tb`.
        let _ = unsafe { ins_typebuf(keys, 1, 0, true, silent) };
    }
    true
}

/// Evaluate the RHS of an `<expr>` mapping or abbreviation, escaping the
/// result so it can be used as typeahead.
///
/// Careful: after this `mp` is invalid if the mapping was deleted.  `c` is
/// NUL, or the typed character for an abbreviation.
///
/// # Safety
/// `mp` must be a live mapblock, and `curwin` a live window.
pub(crate) unsafe fn eval_map_expr(mp: Mb, c: c_int) -> *mut c_char {
    let luaref = mp.m_luaref;
    // Remove the escaping of K_SPECIAL: `m_str` is in the format used for
    // typeahead, not the one the expression is written in.
    let expr = if luaref == LUA_NOREF {
        // SAFETY: the caller's promise — `mp` is a live mapblock, so `m_str`
        // is its NUL-terminated RHS.  The copy is freed below.
        unsafe {
            let expr = xstrdup(mp.m_str);
            vim_unescape_ks(expr);
            expr
        }
    } else {
        ptr::null_mut()
    };
    let replace_keycodes = mp.m_replace_keycodes;

    // Forbid changing text or using ":normal", which rules out most of the bad
    // side effects, and restore the cursor position afterwards.
    let locked = Lock::expr_map();
    // SAFETY: sets `v:char`, which is a plain vim variable.
    unsafe { set_vim_var_char(c) }; // set v:char to the typed character
    // SAFETY: the caller's promise — `curwin` is a live window.
    let save_cursor = unsafe { (*curwin.get()).w_cursor };
    let save_msg_col = msg_col.get();
    let save_msg_row = msg_row.get();

    let mut p: *mut c_char = ptr::null_mut();
    if luaref != LUA_NOREF {
        let mut err = Error::none();
        // SAFETY: `luaref` is the mapping's own reference, and `err` is a
        // live, initialised slot for the call's error.
        let ret = unsafe {
            nlua_call_ref(
                luaref,
                ptr::null(),
                ARRAY_DICT_INIT,
                kRetObject,
                ptr::null_mut(),
                &mut err,
            )
        };
        if ret.type_0 == kObjectTypeString as _ {
            // SAFETY: the object the call handed back, whose string arm the
            // tag above says is the live one.
            p = unsafe { string_to_cstr(ret.data.string) };
        }
        // SAFETY: the object is ours to release once its string is copied.
        unsafe { api_free_object(ret) };
        if err.is_set() {
            // SAFETY: `err.msg` is the NUL-terminated text the call set.
            unsafe {
                semsg_multiline!(c"emsg", "E5108: {}", c_str(err.message_or_empty().as_ptr()));
                err.clear();
            }
        }
    } else {
        // SAFETY: `expr` is the unescaped copy made above, freed here.
        unsafe {
            p = eval_to_string(expr, false, false);
            xfree(expr.cast());
        }
    }

    drop(locked);
    // SAFETY: `curwin` is live again — the evaluation above cannot close the
    // last window, and `w_cursor` is restored into whatever it now names.
    unsafe { (*curwin.get()).w_cursor = save_cursor };
    msg_col.set(save_msg_col);
    msg_row.set(save_msg_row);

    if p.is_null() {
        return ptr::null_mut();
    }

    let mut res: *mut c_char = ptr::null_mut();
    if replace_keycodes {
        let out = &raw mut res;
        let cpo = p_cpo.get();
        let dolt = REPTERM_DO_LT as c_int;
        let simplify = ptr::null_mut();
        // SAFETY: `p` is the NUL-terminated result of the evaluation, and
        // `res` a live slot for the allocation `replace_termcodes` makes.
        unsafe {
            let len = cstr::bytes_at(p).len();
            replace_termcodes(p, len, out, 0, dolt, simplify, cpo);
        }
    } else {
        // Escape K_SPECIAL so the result can be used as typeahead.
        // SAFETY: as above — `p` is NUL-terminated.
        res = unsafe { vim_strsave_escape_ks(p) };
    }
    // SAFETY: `p` is the evaluation's own allocation, freed once.
    unsafe { xfree(p.cast()) };
    res
}
