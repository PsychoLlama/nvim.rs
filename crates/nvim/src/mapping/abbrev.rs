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

/// Whether `mp`'s LHS is exactly the bytes at `word`, in a mode that is
/// current.
///
/// The stored LHS may carry `K_SPECIAL` escapes, which the text in the buffer
/// does not, so it is unescaped into a scratch copy first.
fn abbr_matches(mp: Mb, word: &[u8]) -> bool {
    if mp.m_mode & State.get() == 0 {
        return false;
    }
    let keys = mp.keys();
    if !keys.contains(&(K_SPECIAL as u8)) {
        return keys == word;
    }
    // `vim_unescape_ks` rewrites in place and only ever shortens, so a
    // NUL-terminated copy is enough room.
    let mut scratch = mp.m_keys.as_bytes_with_nul().to_vec();
    // SAFETY: `scratch` is NUL-terminated and outlives the call, which only
    // rewrites what is already there.
    let unescaped = unsafe {
        vim_unescape_ks(scratch.as_mut_ptr().cast());
        cstr::bytes_at(scratch.as_ptr().cast())
    };
    unescaped == word
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
    // SAFETY (every region below): the caller's promise — `ptr` is readable
    // for `col` bytes, and `mb_prevptr`/`utfc_ptr2len`/`vim_iswordp` stay
    // inside `ptr..ptr+col` given that.
    let start = unsafe { ptr.offset(mincol as isize) };
    let mut p = unsafe { mb_prevptr(ptr, ptr.offset(col as isize)) };
    let vim_abbr = !unsafe { vim_iswordp(p) };
    let mut is_id = true;
    if !vim_abbr && p > ptr {
        is_id = unsafe { vim_iswordp(mb_prevptr(ptr, p)) };
    }
    while p > start {
        p = unsafe { mb_prevptr(ptr, p) };
        let stop = unsafe { ascii_isspace(c_int::from(*p)) }
            || (!vim_abbr && is_id != unsafe { vim_iswordp(p) });
        if stop {
            p = unsafe { p.offset(utfc_ptr2len(p) as isize) };
            break;
        }
        clen += 1;
    }
    let mut scol = unsafe { p.offset_from(ptr) } as c_int;
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
    // SAFETY: `mincol <= scol < col`, so this names `len` bytes of the
    // caller's text.
    let word = unsafe { core::slice::from_raw_parts(word.cast::<u8>(), len as usize) };
    let matches = |mp: Mb| abbr_matches(mp, word).then_some(mp);
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
                let escaped = COwned::new(vim_strsave_escape_ks(at));
                if let Some(bytes) = escaped.as_bytes() {
                    tb[j..j + bytes.len()].copy_from_slice(bytes);
                    j += bytes.len();
                }
            }
        }
        tb[j] = NUL as u8;
        // Insert the last typed char.
        let keys = tb.as_mut_ptr().cast();
        let silent = mp.m_silent;
        // SAFETY: `tb` is NUL-terminated at `j` and outlives the call, which
        // copies out of it.
        let _ = unsafe { ins_typebuf(keys, 1, 0, true, silent) };
    }

    // Copy the values out here: eval_map_expr() may make "mp" invalid.
    let noremap = mp.m_noremap;
    let silent = mp.m_silent;
    let expr = mp.m_expr;

    // The RHS bundle is held across the insert: an `<expr>` evaluation can
    // redefine the abbreviation, and the `Rc` is what makes the stored text
    // outlive that.
    let rhs = Rc::clone(&mp.m_rhs);
    let evaluated = if expr {
        // SAFETY: `mp` is still linked — nothing above can have run Vimscript.
        unsafe { eval_map_expr(mp, c) }
    } else {
        None
    };
    if let Some(s) = if expr {
        evaluated.as_ref()
    } else {
        Some(&rhs.str)
    } {
        // Insert the "to" string.
        // SAFETY: `s` is NUL-terminated by `MapStr`'s own invariant and
        // outlives the call, which copies out of it.
        unsafe {
            let _ = ins_typebuf(s.as_mut_ptr(), noremap, 0, true, silent);
            // No abbreviation for these chars.
            typeahead().add_no_abbr_cnt(s.len() as c_int + j as c_int + 1);
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
pub(crate) unsafe fn eval_map_expr(mp: Mb, c: c_int) -> Option<MapStr> {
    let luaref = mp.luaref();
    // Remove the escaping of K_SPECIAL: `m_str` is in the format used for
    // typeahead, not the one the expression is written in.  `vim_unescape_ks`
    // rewrites in place and only shortens, so the copy is room enough.
    let mut expr = mp.m_rhs.str.as_bytes_with_nul().to_vec();
    // SAFETY: `expr` is this frame's own NUL-terminated buffer.
    unsafe { vim_unescape_ks(expr.as_mut_ptr().cast()) };
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

    let answer = if luaref != LUA_NOREF {
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
        // SAFETY: the string the call handed back, then the object it came
        // out of, which is ours to release.
        let answer = unsafe {
            let answer = match ret {
                Object::String(s) => COwned::new(string_to_cstr(s)),
                _ => COwned::new(ptr::null_mut()),
            };
            api_free_object(ret);
            answer
        };
        if err.is_set() {
            // SAFETY: `err.msg` is the NUL-terminated text the call set.
            unsafe {
                semsg_multiline!(c"emsg", "E5108: {}", c_str(err.message_or_empty().as_ptr()));
                err.clear();
            }
        }
        answer
    } else {
        // SAFETY: `expr` is the unescaped copy made above, and the answer is
        // the evaluation's own allocation.
        unsafe { COwned::new(eval_to_string(expr.as_mut_ptr().cast(), false, false)) }
    };

    drop(locked);
    // SAFETY: `curwin` is live again — the evaluation above cannot close the
    // last window, and `w_cursor` is restored into whatever it now names.
    unsafe { (*curwin.get()).w_cursor = save_cursor };
    msg_col.set(save_msg_col);
    msg_row.set(save_msg_row);

    let bytes = answer.as_bytes()?;
    if !replace_keycodes {
        // Escape K_SPECIAL so the result can be used as typeahead.
        // SAFETY: `answer` is the evaluation's NUL-terminated result, and the
        // escaped copy is the guard's.
        let escaped =
            unsafe { COwned::new(vim_strsave_escape_ks(bytes.as_ptr().cast_mut().cast())) };
        return escaped.to_map_str();
    }
    let mut res: *mut c_char = ptr::null_mut();
    let out = &raw mut res;
    let cpo = p_cpo.get();
    let dolt = REPTERM_DO_LT as c_int;
    let simplify = ptr::null_mut();
    // SAFETY: as above; `res` is a live slot for the allocation
    // `replace_termcodes` makes, which the guard releases.
    let replaced = unsafe {
        let at = replace_termcodes(
            bytes.as_ptr().cast(),
            bytes.len(),
            out,
            0,
            dolt,
            simplify,
            cpo,
        );
        let _owned = COwned::new(res);
        MapStr::new(cstr::bytes_at(at))
    };
    Some(replaced)
}
