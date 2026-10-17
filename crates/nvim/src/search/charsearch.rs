//! The character search: `f`, `t`, `F`, `T` and their `;`/`,` repeats.
//!
//! One line, one character, `cmd_arg.count1` times. [`CharSearch`] is what
//! `;` and `,` replay; `set_last_csearch` and friends exist so that
//! `getcharsearch()`/`setcharsearch()` can read and write them.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::cstr;
use crate::mbyte::MAX_SCHAR_SIZE;
use crate::mbyte::{c_cluster_len, char_len, encode_char};
use crate::option::cpo_has;
use crate::types::{CpoFlag, Failed, NUL};
use crate::winlayer::Win;
use core::ffi::c_int;

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
    lastc_bytes: [u8; SCHAR_BYTES] = [0; SCHAR_BYTES];
    lastc_bytelen: c_int = 1;
}

/// One `ScreenChar`'s bytes plus its NUL — what `lastc_bytes` holds.
const SCHAR_BYTES: usize = MAX_SCHAR_SIZE as usize + 1;

/// The bytes `;` and `,` look for, NUL-terminated, by value: `getcharsearch()`
/// reads them while the very dictionary it is building can run code.
pub fn last_csearch() -> [u8; SCHAR_BYTES] {
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
    bytes[..len].copy_from_slice(&text[..len]);
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
                // `nchar_composing` holds `nchar_len` bytes; the store holds
                // one cell's worth, and upstream copies them all.
                let len = usize::try_from(cmd_arg.nchar_len).unwrap_or(0);
                for (to, &from) in bytes.iter_mut().zip(&cmd_arg.nchar_composing[..len]) {
                    *to = from as u8;
                }
            } else {
                let len = encode_char(c, &mut bytes);
                lastc_bytelen.set(c_int::try_from(len).expect("a character's bytes"));
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

    let mut lines = Lines::current();
    let line = lines.line(Win::current().w_cursor.lnum);
    let len = line.len() as c_int;
    let bytelen = lastc_bytelen.get();
    let bytes = lastc_bytes.get();
    let mut col = Win::current().w_cursor.col as c_int;
    // The first `n` bytes of `text`, or up to its NUL: what `strncmp` reads.
    let prefix = |text: &[u8], n: usize| cstr::until_nul(&text[..n.min(text.len())]).to_vec();
    let want = prefix(&bytes, bytelen as usize);

    while count > 0 {
        count -= 1;
        loop {
            if dir > 0 {
                let at = usize::try_from(col).unwrap_or(0);
                col += c_cluster_len(line.get(at..).unwrap_or_default()) as c_int;
                if col >= len {
                    return Err(Failed);
                }
            } else {
                if col == 0 {
                    return Err(Failed);
                }
                col -= head_off(line, col as usize - 1) as c_int + 1;
            }
            let at = col as usize;
            let hit = if bytelen <= 1 {
                c_int::from(line[at]) == c
            } else {
                prefix(&line[at..], bytelen as usize) == want
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
            col -= head_off(line, col as usize) as c_int;
        }
    }
    Win::current().w_cursor.col = col as ColNr;
    Ok(())
}
