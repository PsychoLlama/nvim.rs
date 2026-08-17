//! Abbreviations and `<expr>` right-hand sides.
//!
//! [`check_abbr`] is called after every inserted character: it looks back for
//! a word matching an entry on the abbrlist and, on a match, pushes the
//! deletions and the replacement into the typeahead.  [`eval_map_expr`]
//! evaluates an `<expr>` mapping's RHS, which both this and the mapping match
//! need.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::keycodes::{Ctrl_H, Ctrl_RSB, Ctrl_V, key_escape};
#[allow(unused_imports)]
use crate::semsg_multiline_c;
use crate::types::{MB_MAXBYTES, kErrorTypeNone};
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
unsafe fn abbr_matches(mp: *mut mapblock_T, word: *const c_char, len: c_int) -> bool {
    unsafe {
        let mut qlen = (*mp).m_keylen;
        let mut q = (*mp).m_keys;
        if !strchr((*mp).m_keys, K_SPECIAL).is_null() {
            q = xstrdup((*mp).m_keys);
            vim_unescape_ks(q);
            qlen = strlen(q) as c_int;
        }
        let matched =
            (*mp).m_mode & State.get() != 0 && qlen == len && strncmp(q, word, len as size_t) == 0;
        if q != (*mp).m_keys {
            xfree(q.cast());
        }
        matched
    }
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
    unsafe {
        if (*typebuf.ptr()).tb_no_abbr_cnt != 0 {
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
        {
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

        let word = ptr.offset(scol as isize);
        let len = col - scol;
        // Buffer-local abbreviations first, then the global ones.
        let mut found = ptr::null_mut();
        for table in [MapTable::Buffer(curbuf.get()), MapTable::Global] {
            if let Some(mp) = map_walk(table, true, |mp| abbr_matches(mp, word, len).then_some(mp))
            {
                found = mp;
                break;
            }
        }
        let mp = found;
        if mp.is_null() {
            return false;
        }

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
                let at = tb.as_mut_ptr().add(j).cast::<c_char>();
                let newlen = utf_char2bytes(c, at) as usize;
                tb[j + newlen] = NUL as u8;
                // Need to escape K_SPECIAL.
                let escaped = vim_strsave_escape_ks(at);
                if !escaped.is_null() {
                    let newlen = strlen(escaped) as usize;
                    ptr::copy(escaped.cast::<u8>(), tb.as_mut_ptr().add(j), newlen);
                    j += newlen;
                    xfree(escaped.cast());
                }
            }
            tb[j] = NUL as u8;
            // Insert the last typed char.
            ins_typebuf(tb.as_mut_ptr().cast(), 1, 0, true, (*mp).m_silent != 0);
        }

        // Copy the values out here: eval_map_expr() may make "mp" invalid.
        let noremap = (*mp).m_noremap;
        let silent = (*mp).m_silent != 0;
        let expr = (*mp).m_expr != 0;

        let s = if expr {
            eval_map_expr(mp, c)
        } else {
            (*mp).m_str
        };
        if !s.is_null() {
            // Insert the "to" string.
            ins_typebuf(s, noremap, 0, true, silent);
            // No abbreviation for these chars.
            (*typebuf.ptr()).tb_no_abbr_cnt += strlen(s) as c_int + j as c_int + 1;
            if expr {
                xfree(s.cast());
            }
        }

        // Delete the "from" string: characters, not bytes.
        tb[0] = Ctrl_H as u8;
        tb[1] = NUL as u8;
        for _ in 0..clen {
            ins_typebuf(tb.as_mut_ptr().cast(), 1, 0, true, silent);
        }
        true
    }
}

/// Evaluate the RHS of an `<expr>` mapping or abbreviation, escaping the
/// result so it can be used as typeahead.
///
/// Careful: after this `mp` is invalid if the mapping was deleted.  `c` is
/// NUL, or the typed character for an abbreviation.
///
/// # Safety
/// `mp` must be a live mapblock, and `curwin` a live window.
pub unsafe fn eval_map_expr(mp: *mut mapblock_T, c: c_int) -> *mut c_char {
    unsafe {
        // Remove the escaping of K_SPECIAL: `m_str` is in the format used for
        // typeahead, not the one the expression is written in.
        let expr = if (*mp).m_luaref == LUA_NOREF {
            let expr = xstrdup((*mp).m_str);
            vim_unescape_ks(expr);
            expr
        } else {
            ptr::null_mut()
        };
        let replace_keycodes = (*mp).m_replace_keycodes;

        // Forbid changing text or using ":normal", which rules out most of
        // the bad side effects, and restore the cursor position afterwards.
        *expr_map_lock.ptr() += 1;
        set_vim_var_char(c); // set v:char to the typed character
        let save_cursor = (*curwin.get()).w_cursor;
        let save_msg_col = msg_col.get();
        let save_msg_row = msg_row.get();

        let mut p: *mut c_char = ptr::null_mut();
        if (*mp).m_luaref != LUA_NOREF {
            let mut err = Error {
                type_0: kErrorTypeNone,
                msg: ptr::null_mut(),
            };
            let ret = nlua_call_ref(
                (*mp).m_luaref,
                ptr::null(),
                ARRAY_DICT_INIT,
                kRetObject,
                ptr::null_mut(),
                &raw mut err,
            );
            if ret.type_0 == kObjectTypeString as _ {
                p = string_to_cstr(ret.data.string);
            }
            api_free_object(ret);
            if err.type_0 != kErrorTypeNone {
                semsg_multiline_c!(c"emsg".as_ptr(), c"E5108: %s".as_ptr(), err.msg);
                api_clear_error(&raw mut err);
            }
        } else {
            p = eval_to_string(expr, false, false);
            xfree(expr.cast());
        }

        *expr_map_lock.ptr() -= 1;
        (*curwin.get()).w_cursor = save_cursor;
        msg_col.set(save_msg_col);
        msg_row.set(save_msg_row);

        if p.is_null() {
            return ptr::null_mut();
        }

        let mut res: *mut c_char = ptr::null_mut();
        if replace_keycodes {
            replace_termcodes(
                p,
                strlen(p),
                &raw mut res,
                0,
                REPTERM_DO_LT as c_int,
                ptr::null_mut(),
                p_cpo.get(),
            );
        } else {
            // Escape K_SPECIAL so the result can be used as typeahead.
            res = vim_strsave_escape_ks(p);
        }
        xfree(p.cast());
        res
    }
}
