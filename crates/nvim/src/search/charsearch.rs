//! The character search: `f`, `t`, `F`, `T` and their `;`/`,` repeats.
//!
//! One line, one character, `cmd_arg.count1` times. [`CharSearch`] is what
//! `;` and `,` replay; `set_last_csearch` and friends exist so that
//! `getcharsearch()`/`setcharsearch()` can read and write them.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::cstr;
use crate::mbyte::MAX_SCHAR_SIZE;
use crate::mbyte::char_len;
use crate::option::cpo_has;
use crate::types::{CpoFlag, Failed, NUL};
use crate::winlayer::Win;
use core::ffi::{c_char, c_int};
use core::ptr;

state_record! {
    /// What `;` and `,` replay: the last `f`/`t`/`F`/`T`.
    struct CharSearch in CSEARCH as CharSearchField;
    /// The character `f`/`t` last looked for, as a byte: the first byte
    /// only, which the single-byte fast path compares against and
    /// `last_csearch_*` reports.
    lastc: u8 = NUL as u8;
    /// Its direction.
    lastcdir: Direction = FORWARD;
    /// Whether it was `t`/`T`, which stop short of the character.
    last_t_cmd: bool = true;
    /// The full (possibly multi-byte, possibly composed) sequence, which
    /// `lastc_bytelen > 1` switches the comparison over to.
    lastc_bytes: [c_char; SCHAR_BYTES] = [0; SCHAR_BYTES];
    lastc_bytelen: c_int = 1;
}

/// One `ScreenChar`'s bytes plus its NUL — what `lastc_bytes` holds.
const SCHAR_BYTES: usize = MAX_SCHAR_SIZE as usize + 1;

/// The bytes `;` and `,` look for, NUL-terminated, by value: `getcharsearch()`
/// reads them while the very dictionary it is building can run code.
pub fn last_csearch() -> [c_char; SCHAR_BYTES] {
    lastc_bytes.get()
}

pub fn last_csearch_forward() -> c_int {
    c_int::from(lastcdir.get() as c_int == FORWARD as c_int)
}

pub fn last_csearch_until() -> c_int {
    c_int::from(last_t_cmd.get())
}

/// Remember a character search, for `setcharsearch()`: `c` is its first
/// character and `text` the whole cluster.
///
/// The store holds one screen cell's worth of bytes, so a cluster longer
/// than that keeps only the characters that fit. Upstream copies the whole
/// cluster and writes past the end of its buffer when a base character
/// carries enough composing characters.
pub fn set_last_csearch(c: c_int, text: &[u8]) {
    let mut len = 0;
    while len < text.len() {
        let step = char_len(&text[len..]);
        if len + step > SCHAR_BYTES - 1 {
            break;
        }
        len += step;
    }
    lastc.set(c as u8);
    lastc_bytelen.set(c_int::try_from(len).expect("a cell's worth of bytes"));
    // Upstream writes over the front of the old value rather than replacing
    // it, and `lastc_bytelen` is what says how much of it counts.
    let mut bytes = if len != 0 {
        lastc_bytes.get()
    } else {
        [0; SCHAR_BYTES]
    };
    for (to, &from) in bytes.iter_mut().zip(&text[..len]) {
        *to = from as c_char;
    }
    lastc_bytes.set(bytes);
}

pub fn set_csearch_direction(cdir: Direction) {
    lastcdir.set(cdir);
}

pub fn set_csearch_until(t_cmd: c_int) {
    last_t_cmd.set(t_cmd != 0);
}

/// Search for a character in the current line, `cmd_arg.count1` times.
///
/// With `t_cmd` the cursor lands just before the character rather than on
/// it. A NUL `cmd_arg.nchar` repeats the last character search instead of
/// starting a new one — that is `;` and `,`.
pub fn searchc(cmd_arg: &mut CmdArg, t_cmd: bool) -> Result<(), Failed> {
    let mut c = cmd_arg.nchar; // char to search for
    let mut dir = cmd_arg.arg; // true for searching forward
    let mut t_cmd = t_cmd;
    let mut count = cmd_arg.count1; // repeat count
    let mut stop = true;

    if c != NUL {
        // Normal search: remember the arguments for a later repeat,
        // but not while redoing (the remembered ones are in play).
        if KeyStuffed.get() == 0 {
            lastc.set(c as u8);
            set_csearch_direction(dir as Direction);
            set_csearch_until(c_int::from(t_cmd));
            let mut bytes = lastc_bytes.get();
            if cmd_arg.nchar_len != 0 {
                lastc_bytelen.set(cmd_arg.nchar_len);
                // SAFETY: `nchar_composing` holds `nchar_len` bytes, and
                // `bytes` is a `MB_MAXBYTES`-sized array of this frame.
                unsafe {
                    let from = (&raw const cmd_arg.nchar_composing).cast::<c_char>();
                    ptr::copy_nonoverlapping(from, bytes.as_mut_ptr(), cmd_arg.nchar_len as usize)
                };
            } else {
                lastc_bytelen.set(unsafe { utf_char2bytes(c, bytes.as_mut_ptr()) });
            }
            lastc_bytes.set(bytes);
        }
    } else {
        // Repeat the previous search.
        if lastc.get() as c_int == NUL && lastc_bytelen.get() <= 1 {
            return Err(Failed);
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
        if !cpo_has(CpoFlag::SCOLON) && count == 1 && t_cmd {
            stop = false;
        }
    }

    cmd_arg.op().inclusive = dir != BACKWARD as c_int;

    let line = get_cursor_line_ptr();
    let len = get_cursor_line_len();
    let bytelen = lastc_bytelen.get();
    let bytes = lastc_bytes.get();
    let mut col = Win::current().w_cursor.col as c_int;

    while count > 0 {
        count -= 1;
        loop {
            if dir > 0 {
                col += unsafe { utfc_ptr2len(line.offset(col as isize)) };
                if col >= len {
                    return Err(Failed);
                }
            } else {
                if col == 0 {
                    return Err(Failed);
                }
                col -= unsafe { utf_head_off(line, line.offset(col as isize - 1)) } + 1;
            }
            let hit = if bytelen <= 1 {
                unsafe { *line.offset(col as isize) as c_int == c }
            } else {
                unsafe {
                    cstr::prefix_eq(line.offset(col as isize), bytes.as_ptr(), bytelen as size_t)
                }
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
            col -= unsafe { utf_head_off(line, line.offset(col as isize)) };
        }
    }
    Win::current().w_cursor.col = col as ColNr;
    Ok(())
}
