//! The character search: `f`, `t`, `F`, `T` and their `;`/`,` repeats.
//!
//! One line, one character, `cap->count1` times. The five statics are
//! what `;` and `,` replay; `set_last_csearch` and friends exist so that
//! `getcharsearch()`/`setcharsearch()` can read and write them.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::mbyte::MAX_SCHAR_SIZE;
use core::ffi::{c_char, c_int};
use core::ptr;

/// The character `f`/`t` last looked for, as a byte and as its full
/// (possibly multi-byte, possibly composed) sequence.
///
/// `lastc` is the first byte only; it is what the single-byte fast path
/// compares against and what `last_csearch_*` reports. `lastc_bytelen > 1`
/// switches the comparison over to `lastc_bytes`.
static lastc: GlobalCell<u8> = GlobalCell::new(NUL as u8);
static lastcdir: GlobalCell<Direction> = GlobalCell::new(FORWARD);
static last_t_cmd: GlobalCell<bool> = GlobalCell::new(true);
static lastc_bytes: GlobalCell<[c_char; SCHAR_BYTES]> = GlobalCell::new([0; SCHAR_BYTES]);
static lastc_bytelen: GlobalCell<c_int> = GlobalCell::new(1);

/// One `schar_T`'s bytes plus its NUL — what `lastc_bytes` holds.
const SCHAR_BYTES: usize = MAX_SCHAR_SIZE as usize + 1;

pub unsafe extern "C" fn last_csearch() -> *const c_char {
    lastc_bytes.ptr() as *const c_char
}

pub unsafe extern "C" fn last_csearch_forward() -> c_int {
    c_int::from(lastcdir.get() as c_int == FORWARD as c_int)
}

pub unsafe extern "C" fn last_csearch_until() -> c_int {
    c_int::from(last_t_cmd.get())
}

/// Remember a character search, for `:let @/`-style state restoration.
///
/// # Safety
/// `s` must point at `len` readable bytes.
pub unsafe extern "C" fn set_last_csearch(c: c_int, s: *mut c_char, len: c_int) {
    unsafe {
        lastc.set(c as u8);
        lastc_bytelen.set(len);
        let bytes = lastc_bytes.ptr();
        if len != 0 {
            ptr::copy_nonoverlapping(s, bytes.cast::<c_char>(), len as usize);
        } else {
            (*bytes).fill(0);
        }
    }
}

pub unsafe extern "C" fn set_csearch_direction(cdir: Direction) {
    lastcdir.set(cdir);
}

pub unsafe extern "C" fn set_csearch_until(t_cmd: c_int) {
    last_t_cmd.set(t_cmd != 0);
}

/// Search for a character in the current line, `cap->count1` times.
///
/// With `t_cmd` the cursor lands just before the character rather than on
/// it. A NUL `cap->nchar` repeats the last character search instead of
/// starting a new one — that is `;` and `,`.
///
/// # Safety
/// `cap` and `cap->oap` must be valid.
pub unsafe extern "C" fn searchc(cap: *mut cmdarg_T, t_cmd: bool) -> c_int {
    unsafe {
        let mut c = (*cap).nchar; // char to search for
        let mut dir = (*cap).arg; // true for searching forward
        let mut t_cmd = t_cmd;
        let mut count = (*cap).count1; // repeat count
        let mut stop = true;

        if c != NUL {
            // Normal search: remember the arguments for a later repeat,
            // but not while redoing (the remembered ones are in play).
            if KeyStuffed.get() == 0 {
                lastc.set(c as u8);
                set_csearch_direction(dir as Direction);
                set_csearch_until(c_int::from(t_cmd));
                if (*cap).nchar_len != 0 {
                    lastc_bytelen.set((*cap).nchar_len);
                    ptr::copy_nonoverlapping(
                        (&raw const (*cap).nchar_composing).cast::<c_char>(),
                        lastc_bytes.ptr().cast::<c_char>(),
                        (*cap).nchar_len as usize,
                    );
                } else {
                    lastc_bytelen.set(utf_char2bytes(c, lastc_bytes.ptr().cast::<c_char>()));
                }
            }
        } else {
            // Repeat the previous search.
            if lastc.get() as c_int == NUL && lastc_bytelen.get() <= 1 {
                return FAIL;
            }
            dir = if dir != 0 {
                -(lastcdir.get() as c_int) // repeat in the opposite direction
            } else {
                lastcdir.get() as c_int
            };
            t_cmd = last_t_cmd.get();
            c = lastc.get() as c_int;
            // For multi-byte re-use lastc_bytes[] and lastc_bytelen.

            // Force a move of at least one character, so that ";" and ","
            // move the cursor even when it is right in front of the
            // character being looked for.
            if vim_strchr(p_cpo.get(), CPO_SCOLON).is_null() && count == 1 && t_cmd {
                stop = false;
            }
        }

        (*(*cap).oap).inclusive = dir != BACKWARD as c_int;

        let line = get_cursor_line_ptr();
        let len = get_cursor_line_len();
        let bytelen = lastc_bytelen.get();
        let bytes = lastc_bytes.ptr().cast::<c_char>();
        let mut col = (*curwin.get()).w_cursor.col as c_int;

        while count > 0 {
            count -= 1;
            loop {
                if dir > 0 {
                    col += utfc_ptr2len(line.offset(col as isize));
                    if col >= len {
                        return FAIL;
                    }
                } else {
                    if col == 0 {
                        return FAIL;
                    }
                    col -= utf_head_off(line, line.offset(col as isize - 1)) + 1;
                }
                let hit = if bytelen <= 1 {
                    *line.offset(col as isize) as c_int == c
                } else {
                    strncmp(line.offset(col as isize), bytes, bytelen as size_t) == 0
                };
                if hit && stop {
                    break;
                }
                stop = true;
            }
        }

        if t_cmd {
            // Back up to before the character (which may be multi-byte).
            col -= dir;
            if dir < 0 {
                // Landed on the search char, which is bytelen bytes long.
                col += bytelen - 1;
            } else {
                col -= utf_head_off(line, line.offset(col as isize));
            }
        }
        (*curwin.get()).w_cursor.col = col as colnr_T;
        OK
    }
}
