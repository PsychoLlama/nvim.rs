//! Assembling a string option's new value for `+=`, `^=` and `-=`.
//!
//! [`stropt_get_newval`] is the entry point: it is handed the option's
//! current value, the operator, and the `:set` argument after the `=`, and
//! answers the new value and how much of the argument it took.
//!
//! Upstream assembles into one buffer it allocates up front --
//! `strlen(arg) + 1`, plus `strlen(origval) + 1` for any operator but `=` --
//! and rewrites it in place, every step sized by that one decision. Here
//! each step builds or edits a `Vec<u8>`, so nothing has to be sized ahead.
//!
//! Three shapes of value are handled, and which one an option is comes from
//! its flags:
//!
//! - a **plain string** — concatenated, with a comma between the parts for
//!   a comma-separated option;
//! - a **key-value list** (`kOptFlagComma | kOptFlagColon`, e.g.
//!   'listchars', 'fillchars') — an added `key:value` replaces the item
//!   with the same key rather than sitting beside it;
//! - a **flag-letter set** (`kOptFlagFlagList`) — duplicate letters are
//!   dropped after the concatenation.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::strings::find_char;
use core::ffi::c_int;

use crate::ascii::ascii_iswhite;
use crate::mbyte::c_cluster_len;
use crate::memory::XString;
use crate::options::kOptKeywordprg;
use crate::types::{OptIndex, uint32_t};

use super::{
    OP_ADDING, OP_NONE, OP_PREPENDING, OP_REMOVING, OptSlot, SetOp, kOptFlagColon, kOptFlagComma,
    kOptFlagFlagList, kOptFlagNoDup, kOptFlagOneComma, option_expand, option_var,
};

/// The byte at `i`, the end of `text` reading as its terminator.
fn at(text: &[u8], i: usize) -> u8 {
    text.get(i).copied().unwrap_or(0)
}

/// Copy the value at the start of the `:set` argument `arg`, dropping the
/// backslashes that escape whitespace and separators, and answer it with
/// the length of `arg` it took: everything up to the first unescaped white
/// space.
///
/// `set_option_direct` cannot be used for this precisely because it would
/// keep those backslashes. The reverse transformation is
/// `escape_option_str_cmdline`.
pub(crate) fn stropt_copy_value(arg: &[u8]) -> (Vec<u8>, usize) {
    let mut value = Vec::new();
    let mut i = 0;
    while at(arg, i) != 0 && !ascii_iswhite(c_int::from(at(arg, i))) {
        if at(arg, i) == b'\\' && at(arg, i + 1) != 0 {
            i += 1; // Remove the backslash.
        }
        let len = c_cluster_len(&arg[i..]);
        value.extend_from_slice(&arg[i..i + len]);
        i += len;
    }
    (value, i)
}

/// Where `newval` occurs as an item of `origval`, or `None`.
///
/// For a comma-separated option an item only counts when it starts and ends
/// at a separator. A comma is only a separator when it is not escaped, and
/// the backslash count is what decides that: an even number of backslashes
/// before it, or a single backslash that is itself preceded by a comma.
pub(crate) fn find_dup_item(origval: &[u8], newval: &[u8], flags: uint32_t) -> Option<usize> {
    let comma_list = flags & kOptFlagComma as uint32_t != 0;
    let mut bs = 0;
    for s in 0..origval.len() {
        let starts_item = !comma_list || s == 0 || (origval[s - 1] == b',' && bs & 1 == 0);
        let ends_item = || {
            let after = at(origval, s + newval.len());
            !comma_list || after == b',' || after == 0
        };
        if starts_item && origval[s..].starts_with(newval) && ends_item() {
            return Some(s);
        }
        let escaping = (s > 1 && origval[s - 1] == b'\\' && origval[s - 2] != b',')
            || (s == 1 && origval[0] == b'\\');
        bs = if escaping { bs + 1 } else { 0 };
    }
    None
}

/// The current value and the new one joined, with a comma between them for
/// a comma-separated option.
///
/// A trailing comma on the current value is absorbed rather than doubled,
/// but only for an option whose items are separated by exactly one comma —
/// and only when that comma is not itself escaped.
pub(crate) fn stropt_concat_with_comma(
    origval: &[u8],
    newval: &[u8],
    op: SetOp,
    flags: uint32_t,
) -> Vec<u8> {
    let separated =
        flags & kOptFlagComma as uint32_t != 0 && !origval.is_empty() && !newval.is_empty();
    let comma: &[u8] = if separated { b"," } else { b"" };
    if op == OP_ADDING {
        let mut len = origval.len();
        if separated
            && len > 1
            && flags & kOptFlagOneComma as uint32_t == kOptFlagOneComma as uint32_t
            && origval[len - 1] == b','
            && origval[len - 2] != b'\\'
        {
            len -= 1;
        }
        [&origval[..len], comma, newval].concat()
    } else {
        // Prepending.
        [newval, comma, origval].concat()
    }
}

/// The current value with `len` bytes at `at` cut out of it.
///
/// For a comma-separated option the cut takes a separating comma with it —
/// the one after the item when it is the first, the one before it
/// otherwise — so that the result does not end up with an empty item.
pub(crate) fn stropt_remove_val(origval: &[u8], flags: uint32_t, at: usize, len: usize) -> Vec<u8> {
    if at >= origval.len() {
        return origval.to_vec();
    }
    let (mut at, mut len) = (at, len);
    if flags & kOptFlagComma as uint32_t != 0 {
        if at == 0 {
            if self::at(origval, len) == b',' {
                len += 1;
            }
        } else {
            at -= 1;
            len += 1;
        }
    }
    [&origval[..at], &origval[at + len..]].concat()
}

/// The item of `src` that opens with `key`, and how long it is.
///
/// An item only counts at the start of the value or just after a comma, so
/// that "ab:1" is not found by the key "b:".
pub(crate) fn find_key_item(src: &[u8], key: &[u8]) -> Option<(usize, usize)> {
    (0..src.len()).find_map(|p| {
        ((p == 0 || src[p - 1] == b',') && src[p..].starts_with(key)).then(|| {
            let len = src[p..]
                .iter()
                .position(|&b| b == b',')
                .unwrap_or(src.len() - p);
            (p, len)
        })
    })
}

/// Cut one item out of a comma-separated value, taking whichever of its
/// neighbouring commas exists with it.
pub(crate) fn remove_comma_item(value: &mut Vec<u8>, item: usize, itemlen: usize) {
    let after = item + itemlen;
    if value.get(after) == Some(&b',') {
        value.drain(item..=after);
    } else if item > 0 && value[item - 1] == b',' {
        value.drain(item - 1..after);
    } else {
        // The only item there was.
        value.truncate(item);
    }
}

/// Cut every item with this key out of the value, except the one at `skip`.
pub(crate) fn remove_key_item(value: &mut Vec<u8>, key: &[u8], skip: Option<usize>) {
    loop {
        let Some((mut found, mut itemlen)) = find_key_item(value, key) else {
            return;
        };
        if Some(found) == skip {
            // Look past the one being kept, for a second with the same key.
            let mut next = found + itemlen;
            if at(value, next) == b',' {
                next += 1;
            }
            let Some((later, len)) = find_key_item(&value[next..], key) else {
                return;
            };
            (found, itemlen) = (next + later, len);
        }
        remove_comma_item(value, found, itemlen);
    }
}

/// Add `item` to whichever end of a comma-separated value the operator asks
/// for.
fn place(value: &mut Vec<u8>, item: &[u8], op: SetOp) {
    let comma: &[u8] = if value.is_empty() { b"" } else { b"," };
    if op == OP_PREPENDING {
        *value = [item, comma, value].concat();
    } else {
        value.extend_from_slice(comma);
        value.extend_from_slice(item);
    }
}

/// Assemble a key-value list, where an added `key:value` *replaces* the
/// item with the same key instead of sitting beside it.
///
/// `None` — and nothing assembled — for an argument that is neither keyed
/// nor a list, which is how a plain value falls through to the ordinary
/// concatenation.
pub(crate) fn stropt_handle_keymatch(
    origval: &[u8],
    argument: &[u8],
    op: SetOp,
) -> Option<Vec<u8>> {
    if !argument.contains(&b':') && !argument.contains(&b',') {
        return None;
    }
    let mut newval = origval.to_vec();
    let mut item_start = 0;
    loop {
        let rest = &argument[item_start..];
        let next = rest.iter().position(|&b| b == b',');
        let item = &rest[..next.unwrap_or(rest.len())];
        if !item.is_empty() {
            // The key is everything up to and including the colon.
            match item.iter().position(|&b| b == b':') {
                Some(colon) => {
                    let key = &item[..=colon];
                    match op {
                        OP_ADDING | OP_PREPENDING => match find_key_item(&newval, key) {
                            None => place(&mut newval, item, op),
                            Some((found, old_len))
                                if old_len == item.len() && newval[found..].starts_with(item) =>
                            {
                                // The same item already: keep it where it
                                // is, and drop any later duplicate.
                                remove_key_item(&mut newval, key, Some(found));
                            }
                            Some(_) => {
                                remove_key_item(&mut newval, key, None);
                                place(&mut newval, item, op);
                            }
                        },
                        OP_REMOVING => remove_key_item(&mut newval, key, None),
                        _ => {}
                    }
                }
                None => {
                    // An item with no key of its own is matched whole.
                    let found = find_dup_item(&newval, item, kOptFlagComma as uint32_t);
                    match (op, found) {
                        (OP_ADDING | OP_PREPENDING, None) => place(&mut newval, item, op),
                        (OP_REMOVING, Some(found)) => {
                            remove_comma_item(&mut newval, found, item.len());
                        }
                        _ => {}
                    }
                }
            }
        }
        let Some(next) = next else {
            break;
        };
        item_start += next + 1;
    }
    Some(newval)
}

/// Drop every repeated flag letter, keeping the last of each.
///
/// A one-comma option's letters may each be followed by a comma, so the
/// letter and its comma are dropped together. The letter is looked for as
/// a character, as upstream's `vim_strchr` looks: a byte above 0x7f is the
/// two-byte encoding of its code point.
pub(crate) fn stropt_remove_dupflags(newval: &mut Vec<u8>, flags: uint32_t) {
    // Upstream's `flags & kOptFlagOneComma`, which any comma list passes.
    let one_comma = flags & kOptFlagOneComma as uint32_t != 0;
    let comma_list = flags & kOptFlagComma as uint32_t != 0;
    let mut s = 0;
    while s < newval.len() {
        let letter = c_int::from(newval[s]);
        let later =
            |from: usize| find_char(newval.get(from..).unwrap_or_default(), letter).is_some();
        let drop = if one_comma {
            newval[s] != b',' && at(newval, s + 1) == b',' && later(s + 2)
        } else {
            (!comma_list || newval[s] != b',') && later(s + 1)
        };
        if !drop {
            s += 1;
            continue;
        }
        let past = if one_comma { s + 2 } else { s + 1 };
        newval.drain(s..past);
    }
}

/// Assemble the value a `:set` argument asks for: the new value, and how
/// many bytes of `arg` -- the argument after the `=` or the operator's
/// second character -- it took.
///
/// `op` may come back as `OP_NONE` even for a `+=`: adding something the
/// value already carries is a no-op, and the caller uses that to skip the
/// whole set.
pub(crate) fn stropt_get_newval(
    opt_idx: OptIndex,
    arg: &[u8],
    varp: OptSlot,
    origval: &[u8],
    op: &mut SetOp,
    flags: uint32_t,
) -> (XString, usize) {
    // A bare `:set keywordprg=` means ":help", not the empty string — but
    // only for the global value. `:setlocal keywordprg=` unsets the buffer's
    // copy, which is how it goes back to reading the global one.
    let global_kp = opt_idx == kOptKeywordprg && varp == option_var(opt_idx);
    let bare = at(arg, 0) == 0 || at(arg, 0) == b' ';
    let (mut newval, taken) = if global_kp && bare {
        // The argument itself is left where it was.
        (stropt_copy_value(b":help").0, 0)
    } else {
        stropt_copy_value(arg)
    };

    // Only a whole value or a comma-separated one is expanded; a `+=`
    // on a non-list option would expand the concatenation.
    if *op == OP_NONE || flags & kOptFlagComma as uint32_t != 0 {
        let value = XString::from_vec(newval);
        let value = option_expand(opt_idx, Some(value.as_cstr())).unwrap_or(value);
        newval = value.into_vec();
        newval.pop(); // the terminator
    }

    let keyed = flags & kOptFlagComma as uint32_t != 0
        && flags & kOptFlagColon as uint32_t != 0
        && *op != OP_NONE;
    let matched = if keyed {
        stropt_handle_keymatch(origval, &newval, *op)
    } else {
        None
    };
    if let Some(matched) = matched {
        newval = matched;
    } else {
        let mut len = 0;
        let mut at = origval.len();
        if *op == OP_REMOVING || flags & kOptFlagNoDup as uint32_t != 0 {
            len = newval.len();
            let found = find_dup_item(origval, &newval, flags);
            // Adding something already there changes nothing.
            if (*op == OP_ADDING || *op == OP_PREPENDING) && found.is_some() {
                *op = OP_NONE;
                newval = origval.to_vec();
            }
            // Removing something that is not there cuts an empty span off
            // the end.
            at = found.unwrap_or(origval.len());
        }
        if *op == OP_ADDING || *op == OP_PREPENDING {
            newval = stropt_concat_with_comma(origval, &newval, *op, flags);
        } else if *op == OP_REMOVING {
            newval = stropt_remove_val(origval, flags, at, len);
        }
    }

    if flags & kOptFlagFlagList as uint32_t != 0 {
        stropt_remove_dupflags(&mut newval, flags);
    }

    (XString::from_vec(newval), taken)
}
