//! Counting, and the one-item append -- `count()` and `add()`.
//!
//! `f_count` dispatches to [`count_string`], [`count_list`] or
//! [`count_dict`]; the String form is the interesting one, since it counts
//! *non-overlapping* occurrences of a substring and honours `ic` with
//! multibyte-aware folding, so it has to step by whole characters rather
//! than bytes.  `f_add` is here because it is the other builtin whose whole
//! job is the container's length.
//!
//! Original: `src/nvim/eval/list.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::{
    Container, DictRef, ListRef, char_len, check_lock, copy_tv, cstr_of_chk, err,
    err_not_countable, err_nr, number_of, starts_with_ic, string_bytes,
};
use crate::eval::typval::NumBuf;
use crate::message::{e_invarg, e_list_index_out_of_range_nr, e_listblobreq};
use crate::narrow::number_as_int;
use crate::types::{EvalFuncData, TypVal, VarNumber, int64_t};

/// `add(container, item)`: append one item to a List or one byte to a Blob.
pub fn f_add(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the caller's contract.
    // Default: failed.
    result.write_number(1);
    match Container::of(&args[0]) {
        Container::List(l) => {
            if !check_lock(l.locked(), c"add() argument") {
                l.append_tv(&args[1]);
                copy_tv(&args[0], result);
            }
        }
        Container::Blob(b) => {
            if !b.is_null() && !check_lock(b.lock(), c"add() argument") {
                let mut error = false;
                let n = number_of(&args[1], &mut error);
                if !error {
                    // Upstream's `(uint8_t)n`: a Number wider than a byte
                    // wraps, and `add(0z, 300)` appending 0x2C is observable.
                    b.push(n.to_le_bytes()[0]);
                    copy_tv(&args[0], result);
                }
            }
        }
        _ => err(e_listblobreq),
    }
}

/// How many times `needle` occurs in `hay`, counting non-overlapping
/// matches; `ic` ignores case.
///
/// The `ic` walk steps a whole character at a time, because folding is
/// per character and a byte-wise scan would find matches inside a multibyte
/// sequence.
fn count_string(hay: &[u8], needle: &[u8], ic: bool) -> VarNumber {
    if needle.is_empty() {
        return 0;
    }
    let mut n = 0;
    let mut at = 0;
    if ic {
        while at < hay.len() {
            if starts_with_ic(&hay[at..], needle) {
                n += 1;
                // A case-insensitive match may be *shorter* than the needle
                // -- two of Unicode's folds change a character's length --
                // so the skip is clamped to what is left.  Upstream adds the
                // needle's length unconditionally and reads past the
                // terminator when the match ended the string.
                at = (at + needle.len()).min(hay.len());
            } else {
                at += char_len(&hay[at..]).min(hay.len() - at);
            }
        }
    } else {
        while at + needle.len() <= hay.len() {
            if &hay[at..at + needle.len()] == needle {
                n += 1;
                at += needle.len();
            } else {
                at += 1;
            }
        }
    }
    n
}

/// How many items of `l` from index `idx` on equal `needle`.
fn count_list(l: ListRef, needle: &TypVal, idx: int64_t, ic: bool) -> VarNumber {
    if l.len() == 0 {
        return 0;
    }
    let Some(first) = l.find(number_as_int(idx)) else {
        err_nr(e_list_index_out_of_range_nr, idx);
        return 0;
    };

    let mut n = 0;
    let mut cur = Some(first);
    while let Some(li) = cur {
        if li.equals(needle, ic) {
            n += 1;
        }
        cur = li.next();
    }
    n
}

/// How many values of `d` equal `needle`.
fn count_dict(d: DictRef, needle: &TypVal, ic: bool) -> VarNumber {
    if d.is_null() {
        return 0;
    }
    let mut n = 0;
    for di in d.items() {
        if di.equals(needle, ic) {
            n += 1;
        }
    }
    n
}

/// `count(container, expr [, ic [, start]])`: how many times `expr` occurs.
///
/// `start` is a List-only index to begin at, and asking a Dict for one is
/// `E474` -- which is why the two optional arguments are read in this order
/// and not as a pair.
pub fn f_count(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the caller's contract.
    let mut error = false;
    let ic = args.len() > 2 && number_of(&args[2], &mut error) != 0;

    let mut n = 0;
    if !error {
        match Container::of(&args[0]) {
            Container::Str(_) => {
                let hay = string_bytes(&args[0]);
                let mut numbuf = NumBuf::new();
                if let Some(needle) = cstr_of_chk(&args[1], &mut numbuf) {
                    n = count_string(hay, needle.to_bytes(), ic);
                }
            }
            Container::List(l) => {
                // `start` is only looked at when `ic` was passed too.
                let idx = if args.len() > 2 && args.len() > 3 {
                    number_of(&args[3], &mut error)
                } else {
                    0
                };
                if !error {
                    n = count_list(l, &args[1], idx, ic);
                }
            }
            Container::Dict(d) if !d.is_null() => {
                if args.len() > 2 && args.len() > 3 {
                    err(e_invarg);
                } else {
                    n = count_dict(d, &args[1], ic);
                }
            }
            // A NULL Dict answers zero without looking at the arguments.
            Container::Dict(_) => {}
            _ => err_not_countable(c"count()"),
        }
    }
    result.write_number(n);
}
