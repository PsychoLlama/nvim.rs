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

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use super::{
    Container, Dict, List, char_len, check_lock, copy_tv, cstr_of_chk, err, err_not_countable,
    err_nr, frame, number_of, starts_with_ic, string_bytes,
};
use crate::eval::typval::NumBuf;
use crate::main::{e_invarg, e_list_index_out_of_range_nr, e_listblobreq};
use crate::types::{EvalFuncData, int64_t, typval_T, uint8_t, varnumber_T};

/// `add(container, item)`: append one item to a List or one byte to a Blob.
///
/// # Safety
/// `argvars` is the evaluator's own argument vector, arity 2, and `rettv` a
/// cleared result.
pub unsafe fn f_add(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the caller's contract.
    let (mut args, rettv) = frame!(argvars, rettv);
    // Default: failed.
    rettv.vval.v_number = 1;
    match Container::of(args.get_mut(0)) {
        Container::List(l) => {
            if !check_lock(l.locked(), c"add() argument") {
                l.append_tv(args.get_mut(1));
                copy_tv(args.get_mut(0), rettv);
            }
        }
        Container::Blob(b) => {
            if !b.is_null() && !check_lock(b.lock(), c"add() argument") {
                let mut error = false;
                let n = number_of(args.get_mut(1), &mut error);
                if !error {
                    b.push(n as uint8_t);
                    copy_tv(args.get_mut(0), rettv);
                }
            }
        }
        _ => err(&e_listblobreq),
    }
}

/// How many times `needle` occurs in `hay`, counting non-overlapping
/// matches; `ic` ignores case.
///
/// The `ic` walk steps a whole character at a time, because folding is
/// per character and a byte-wise scan would find matches inside a multibyte
/// sequence.
fn count_string(hay: &[u8], needle: &[u8], ic: bool) -> varnumber_T {
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
fn count_list(l: List, needle: &mut typval_T, idx: int64_t, ic: bool) -> varnumber_T {
    if l.len() == 0 {
        return 0;
    }
    let Some(first) = l.find(idx as c_int) else {
        err_nr(&e_list_index_out_of_range_nr, idx);
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
fn count_dict(d: Dict, needle: &mut typval_T, ic: bool) -> varnumber_T {
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
///
/// # Safety
/// `argvars` is the evaluator's own argument vector, arity 2..4, and `rettv`
/// a cleared result.
pub unsafe fn f_count(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the caller's contract.
    let (mut args, rettv) = frame!(argvars, rettv);
    let mut error = false;
    let ic = args.has(2) && number_of(args.get_mut(2), &mut error) != 0;

    let mut n = 0;
    if !error {
        match Container::of(args.get_mut(0)) {
            Container::Str(_) => {
                let hay = string_bytes(args.get_mut(0));
                let mut numbuf = NumBuf::new();
                if let Some(needle) = cstr_of_chk(args.get_mut(1), &mut numbuf) {
                    n = count_string(hay, needle.to_bytes(), ic);
                }
            }
            Container::List(l) => {
                // `start` is only looked at when `ic` was passed too.
                let idx = if args.has(2) && args.has(3) {
                    number_of(args.get_mut(3), &mut error)
                } else {
                    0
                };
                if !error {
                    n = count_list(l, args.get_mut(1), idx, ic);
                }
            }
            Container::Dict(d) if !d.is_null() => {
                if args.has(2) && args.has(3) {
                    err(&e_invarg);
                } else {
                    n = count_dict(d, args.get_mut(1), ic);
                }
            }
            // A NULL Dict answers zero without looking at the arguments.
            Container::Dict(_) => {}
            _ => err_not_countable(c"count()"),
        }
    }
    rettv.vval.v_number = n;
}
