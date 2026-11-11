//! The character search: `f`, `t`, `F`, `T` and their `;`/`,` repeats.
//!
//! One line, one character, `cmd_arg.count1` times. [`CharSearch`] is what
//! `;` and `,` replay; `set_last_csearch` and friends exist so that
//! `getcharsearch()`/`setcharsearch()` can read and write them.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::ascii::NUL;
use crate::cstr;
use crate::grid::schar::MAX_SCHAR_SIZE;
use crate::mbyte::Codepoint;
use crate::mbyte::{c_cluster_len, char_len, encode_char};
use crate::narrow::len_as_int;
use crate::option::cpo_has;
use crate::option::defs::CpoFlag;
use crate::regexp::low_byte;
use crate::winlayer::Win;
use core::ffi::c_int;

state_record! {
    /// What `;` and `,` replay: the last `f`/`t`/`F`/`T`.
    struct CharSearch in CSEARCH as CharSearchField;
    /// The character `f`/`t` last looked for, as a byte: the first byte
    /// only, which the single-byte fast path compares against and
    /// `last_csearch_*` reports.
    LASTC: u8 = b'\0';
    /// Its direction.
    LASTCDIR: Direction = FORWARD;
    /// Whether it was `t`/`T`, which stop short of the character.
    LAST_T_CMD: bool = true;
    /// The full (possibly multi-byte, possibly composed) sequence, which
    /// `LASTC_BYTELEN > 1` switches the comparison over to.
    LASTC_BYTES: [u8; SCHAR_BYTES] = [0; SCHAR_BYTES];
    LASTC_BYTELEN: c_int = 1;
}

/// One `ScreenChar`'s bytes plus its NUL — what `LASTC_BYTES` holds.
const SCHAR_BYTES: usize = MAX_SCHAR_SIZE + 1;

/// The bytes `;` and `,` look for, NUL-terminated, by value: `getcharsearch()`
/// reads them while the very dictionary it is building can run code.
pub fn last_csearch() -> [u8; SCHAR_BYTES] {
    LASTC_BYTES.get()
}

/// Whether the last character search went forward.
pub fn last_csearch_forward() -> bool {
    LASTCDIR.get() == FORWARD
}

/// Whether the last character search stopped short of the character (`t`).
pub fn last_csearch_until() -> bool {
    LAST_T_CMD.get()
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
    LASTC.set(low_byte(c));
    LASTC_BYTELEN.set(c_int::try_from(len).expect("a cell's worth of bytes"));
    // Upstream writes over the front of the old value rather than replacing
    // it, and `LASTC_BYTELEN` is what says how much of it counts.
    let mut bytes = if len != 0 {
        LASTC_BYTES.get()
    } else {
        [0; SCHAR_BYTES]
    };
    bytes[..len].copy_from_slice(&text[..len]);
    LASTC_BYTES.set(bytes);
}

pub fn set_csearch_direction(cdir: Direction) {
    LASTCDIR.set(cdir);
}

pub fn set_csearch_until(t_cmd: c_int) {
    LAST_T_CMD.set(t_cmd != 0);
}

/// Search for a character in the current line, `cmd_arg.count1` times.
///
/// With `t_cmd` the cursor lands just before the character rather than on
/// it. A NUL `cmd_arg.nchar` repeats the last character search instead of
/// starting a new one — that is `;` and `,`. Not finding it says nothing.
pub(crate) fn searchc(cmd_arg: &mut CmdArg, t_cmd: bool) -> Result<(), NoMatch> {
    let mut c = cmd_arg.nchar.code(); // char to search for
    let mut dir = cmd_arg.arg; // true for searching forward
    let mut t_cmd = t_cmd;
    let mut count = cmd_arg.count1; // repeat count
    let mut stop = true;

    if c != Codepoint::NUL {
        // Normal search: remember the arguments for a later repeat,
        // but not while redoing (the remembered ones are in play).
        if !KEY_STUFFED.get() {
            LASTC.set(low_byte(c.get()));
            set_csearch_direction(dir as Direction);
            set_csearch_until(c_int::from(t_cmd));
            let mut bytes = LASTC_BYTES.get();
            if cmd_arg.nchar_len != 0 {
                LASTC_BYTELEN.set(len_as_int(cmd_arg.nchar_len));
                // `nchar_composing` holds `nchar_len` bytes; the store holds
                // one cell's worth, and upstream copies them all.
                let len = cmd_arg.nchar_len;
                for (to, &from) in bytes.iter_mut().zip(&cmd_arg.nchar_composing[..len]) {
                    *to = from;
                }
            } else {
                let len = encode_char(c, &mut bytes);
                LASTC_BYTELEN.set(c_int::try_from(len).expect("a character's bytes"));
            }
            LASTC_BYTES.set(bytes);
        }
    } else {
        // Repeat the previous search.
        if c_int::from(LASTC.get()) == NUL && LASTC_BYTELEN.get() <= 1 {
            return Err(NoMatch);
        }
        dir = if dir != 0 {
            -(LASTCDIR.get() as c_int) // repeat in the opposite direction
        } else {
            LASTCDIR.get() as c_int
        };
        t_cmd = LAST_T_CMD.get();
        c = Codepoint::from(LASTC.get());
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
    let len = line.len();
    let bytelen = usize::try_from(LASTC_BYTELEN.get()).unwrap_or(0);
    let bytes = LASTC_BYTES.get();
    let mut col = Win::current().w_cursor.col.as_usize();
    // The first `n` bytes of `text`, or up to its NUL: what `strncmp` reads.
    let prefix = |text: &[u8], n: usize| cstr::until_nul(&text[..n.min(text.len())]).to_vec();
    let want = prefix(&bytes, bytelen);

    while count > 0 {
        count -= 1;
        loop {
            if dir > 0 {
                col += c_cluster_len(line.get(col..).unwrap_or_default());
                if col >= len {
                    return Err(NoMatch);
                }
            } else {
                if col == 0 {
                    return Err(NoMatch);
                }
                col -= head_off(line, col - 1) + 1;
            }
            let hit = if bytelen <= 1 {
                c_int::from(line[col]) == c.get()
            } else {
                prefix(&line[col..], bytelen) == want
            };
            if hit && stop {
                break;
            }
            stop = true;
        }
    }

    if t_cmd {
        // Back up to before the character (which may be multi-byte).
        if dir < 0 {
            // Landed on the search char, which is bytelen bytes long.
            col += bytelen;
        } else {
            col -= 1;
            col -= head_off(line, col);
        }
    }
    Win::current().w_cursor.col = ColNr::from_index(col);
    Ok(())
}
