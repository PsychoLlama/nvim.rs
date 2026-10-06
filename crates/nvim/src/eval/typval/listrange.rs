//! Ranges over a list: slicing, assigning through a slice, flattening, joining.
//!
//! [`list_check_range_index_one`] and [`list_check_range_index_two`]
//! are the bounds arithmetic `l[i:j]` shares with `l[i:j] = x`;
//! [`list_slice_or_index`] is the subscript itself.
//! [`list_join`] and [`list_join_inner`] are `join()`, which makes two
//! passes so the result buffer is sized once, and [`f_list2str`] is the
//! codepoint-list-to-string builtin.

#![forbid(unsafe_code)]

use super::*;
use crate::eval::encode::tv2echo_bytes;
use crate::eval::executor::eexe_mod_op;
use crate::mbyte::encode_char;
use crate::memory::XString;
use crate::message::emsg;
use crate::semsg;
use crate::types::Failed;
use core::ffi::CStr;

/// Resolve the first index of `l[n1:n2]`, clamping a negative one that fell
/// off the front and raising `E684` when there is no such item.
///
/// `*n1` is updated to the index actually used.
///
pub fn list_check_range_index_one(
    l: Option<&List>,
    n1: &mut ::core::ffi::c_int,
    quiet: bool,
) -> Option<usize> {
    let at = list_find_index(l, n1);
    if at.is_none() && !quiet {
        let index = int64_t::from(*n1);
        semsg!("E684: List index out of range: {index}");
    }
    at
}

/// Resolve the second index of `l[n1:n2]` against the index `idx1` the first
/// one landed on, normalising both to non-negative indexes.
///
/// `idx1` must be an index of `l`.
pub fn list_check_range_index_two(
    l: Option<&List>,
    n1: &mut ::core::ffi::c_int,
    idx1: usize,
    n2: &mut ::core::ffi::c_int,
    quiet: bool,
) -> Result<(), Failed> {
    if *n2 < 0 {
        let Some(at) = list_index(l, *n2) else {
            if !quiet {
                let index = int64_t::from(*n2);
                semsg!("E684: List index out of range: {index}");
            }
            return Err(Failed);
        };
        *n2 = index_of(at);
    }
    if *n1 < 0 {
        *n1 = index_of(idx1);
    }
    if *n2 < *n1 {
        if !quiet {
            let index = int64_t::from(*n2);
            semsg!("E684: List index out of range: {index}");
        }
        return Err(Failed);
    }
    Ok(())
}

/// `dest[idx1:idx2] = src`, or `dest[idx1:idx2] op= src` when `op` is given,
/// naming the target `varname` in a lock error.
///
/// `empty_idx2` means the range had no upper bound (`dest[idx1:]`). `dest`
/// and `src` may be the same list -- `:let l[0:1] += l[0:1]` -- so each
/// step copies the source value out before the target is touched, and
/// borrows either list for one statement at a time.
pub(crate) fn assign_range(
    dest: &ListRef,
    src: Option<&ListRef>,
    idx1_arg: ::core::ffi::c_int,
    idx2: ::core::ffi::c_int,
    empty_idx2: bool,
    op: Option<u8>,
    varname: &[u8],
) -> Result<(), Failed> {
    let mut idx1 = idx1_arg;
    let first = list_find_index(Some(dest), &mut idx1);
    let srclen = src.map_or(0, |src| src.len());

    // Check whether any of the list items is locked before making any
    // changes.  The walk stops at the end of the range or at the end of the
    // source, whichever comes first -- `dest` may be shorter, and the
    // assignment below grows it.
    let mut idx = idx1;
    let mut at = first;
    for i in 0..srclen {
        let Some(dest_at) = at else { break };
        let lock = dest.items()[dest_at].li_lock;
        if value_check_lock_named(lock, varname) {
            return Err(Failed);
        }
        if i + 1 == srclen || (!empty_idx2 && idx2 == idx) {
            break;
        }
        at = (dest_at + 1 < dest.len()).then_some(dest_at + 1);
        idx += 1;
    }

    // Assign the List values to the list items.
    idx = idx1;
    // `first` is `None` only for an empty target, which the caller's
    // `get_lval` has already refused; the guard below then leaves `i` at
    // zero and the E710 under it reports.
    let mut at = first.unwrap_or(0);
    let mut i = 0;
    let op = op.filter(|&op| op != b'=');
    while let Some(src) = src.filter(|_| i < srclen && at < dest.len()) {
        // Both slots are re-read on every step, and the source value is
        // copied out first: when `dest` *is* `src`, the two slots are one
        // list's, and either may have moved since the last step.
        let from = src.items()[i].li_tv.clone();
        let value = match op {
            Some(op) => {
                // The operator works on a copy of the target's value too --
                // which shares its List or Blob, so `+=` still extends the
                // one in the slot -- and the result goes back.
                let mut current = dest.items()[at].li_tv.clone();
                if eexe_mod_op(&mut current, &from, op).is_ok() {
                    Some(current)
                } else {
                    None
                }
            }
            None => Some(from),
        };
        if let Some(value) = value {
            // The old value is released once the borrow of `dest` has
            // ended: it may name `dest`.
            let old = ::core::mem::replace(&mut dest.edit().items_mut()[at].li_tv, value);
            drop(old);
        }
        i += 1;
        if i == srclen || (!empty_idx2 && idx2 == idx) {
            break;
        }
        if at + 1 == dest.len() {
            // Need to add an empty item.
            dest.edit().push_number(0);
        }
        at += 1;
        idx += 1;
    }

    if i < srclen {
        let msg = gettext(c"E710: List value has more items than target");
        emsg(msg);
        return Err(Failed);
    }
    let short = if empty_idx2 {
        at + 1 < dest.len()
    } else {
        idx != idx2
    };
    if short {
        emsg(gettext(c"E711: List value has not enough items"));
        return Err(Failed);
    }
    Ok(())
}

/// `flatten()`: splice the items of any nested list into `list` in place,
/// starting at `first` and going `maxdepth` levels down.
///
/// `first` must be an index into `list`.
///
/// The nested list an item names may be **the list being flattened** -- a
/// list can hold itself -- which is why the splice goes through
/// [`list_extend`] over two handles rather than a second borrow.
pub fn list_flatten(list: &ListRef, first: usize, maxitems: int64_t, maxdepth: int64_t) {
    if maxdepth == 0 {
        return;
    }

    let mut at = first;
    let mut done = 0;
    while at < list.len() && done < maxitems {
        fast_breakcheck();
        if got_int.get() {
            return;
        }
        let step = if list.items()[at].li_tv.v_type() == VAR_LIST {
            let before = list.len();
            // The item naming the nested list is taken out *without* being
            // released, and held until the splice is done: the nested list
            // may be the very list being flattened, or the item may hold its
            // last reference, and either way freeing it first would pull the
            // items out from under the copy.  That is what upstream's
            // `tv_list_drop_items`-then-`tv_clear` order bought.
            let held = list.edit().take_range(at, at);
            let inner = held[0].li_tv.list_shared();
            list_extend(list, inner, Some(at));

            if maxdepth > 0 {
                let inner_len = int64_t::from(list_len(inner.map(|inner| &**inner)));
                list_flatten(list, at, inner_len, maxdepth - 1);
            }
            drop(held);
            // However many items now stand where the one item stood --
            // the recursion above may have spliced in more.
            list.len() + 1 - before
        } else {
            1
        };

        done += 1;
        at += step;
    }
}

/// A fresh list holding copies of `ol[n1..=n2]`, which the caller has
/// already clamped to the list.
pub(crate) fn list_slice(ol: Option<&List>, n1: VarNumber, n2: VarNumber) -> ListRef {
    let mut l = tv_list_alloc((n2 - n1 + 1) as ptrdiff_t);
    for at in n1..=n2 {
        l.push_copy(&list_items(ol)[at as usize].li_tv);
    }
    l
}

/// `list[n1]` or `list[n1 : n2]`, whichever `range` says.
///
/// `result` holds the list being subscripted on the way in.  An index out of
/// range is an error; a *range* out of range is merely empty.
///
/// `result` holds the list being subscripted on the way in, which is the
/// only list this reads -- upstream passed it a second time and the argument
/// went unread.
pub fn list_slice_or_index(
    range: bool,
    n1_arg: VarNumber,
    n2_arg: VarNumber,
    exclusive: bool,
    result: &mut TypVal,
    verbose: bool,
) -> Result<(), Failed> {
    let len = list_len(result.list_ref());
    let mut n1 = n1_arg;
    let mut n2 = n2_arg;

    if n1 < 0 {
        n1 += VarNumber::from(len);
    }
    if n1 < 0 || n1 >= VarNumber::from(len) {
        // For a range we allow invalid values and return an empty list.
        // A list index out of range is an error.
        if !range {
            if verbose {
                semsg!("E684: List index out of range: {}", n1_arg);
            }
            return Err(Failed);
        }
        n1 = VarNumber::from(len);
    }

    if range {
        if n2 < 0 {
            n2 += VarNumber::from(len);
        } else if n2 >= VarNumber::from(len) {
            n2 = VarNumber::from(len - if exclusive { 0 } else { 1 });
        }
        if exclusive {
            n2 -= 1;
        }
        if n2 < 0 || n2 + 1 < n1 {
            n2 = -1;
        }
        let l = list_slice(result.list_ref(), n1, n2);
        tv_clear(result);
        result.write_list(Some(l));
    } else {
        // copy the item to "var1" to avoid that freeing the list makes it
        // invalid.
        let mut var1 = TV_INITIAL_VALUE;
        let at = usize::try_from(n1).expect("an index of the list");
        tv_copy(&list_items(result.list_ref())[at].li_tv, &mut var1);
        tv_clear(result);
        *result = var1;
    }
    Ok(())
}

/// `join()`: append `l`'s items to `out`, separated by `sep`.
///
/// Two passes: stringify every item, then concatenate them, so the output is
/// grown once. An interrupt stops either pass where it is.
pub fn list_join(out: &mut XString, l: Option<&List>, sep: &CStr) -> Result<(), Failed> {
    if list_len(l) == 0 {
        return Ok(());
    }
    let mut joined: Vec<Vec<u8>> = Vec::with_capacity(list_len(l).cast_unsigned() as usize);
    for item in list_iter(l) {
        if got_int.get() {
            break;
        }
        joined.push(tv2echo_bytes(&item.li_tv));
        line_breakcheck();
    }

    let sep = sep.to_bytes();
    let total: usize =
        joined.iter().map(Vec::len).sum::<usize>() + sep.len() * joined.len().saturating_sub(1);
    let mut text = Vec::with_capacity(total);
    for (i, s) in joined.iter().enumerate() {
        if got_int.get() {
            break;
        }
        if i > 0 {
            text.extend_from_slice(sep);
        }
        text.extend_from_slice(s);
        line_breakcheck();
    }
    out.push_bytes(&text);
    Ok(())
}

/// `join()` the builtin.
pub fn f_join(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    if args[0].v_type() != VAR_LIST {
        emsg(gettext(e_listreq));
        return;
    }
    let sep = if args.len() <= 1 {
        Some(c" ")
    } else {
        numbuf.string_chk(&args[1])
    };

    result.write_empty(VAR_STRING);
    let Some(sep) = sep else {
        result.write_string(None);
        return;
    };

    let mut text = XString::new();
    let _ = list_join(&mut text, args[0].list_ref(), sep);
    result.write_string(Some(text.into()));
}

/// `list2str()`: a list of codepoints as a string.
pub fn f_list2str(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_string(None);
    let arg = &args[0];
    if arg.v_type() != VAR_LIST {
        emsg(gettext(e_invarg));
        return;
    }
    let Some(list) = arg.list_ref() else {
        return;
    };

    let mut text = XString::new();
    let mut buf = [0u8; 22];
    for li in list.items() {
        let n = tv_get_number(&li.li_tv);
        let buflen = encode_char(n as ::core::ffi::c_int, &mut buf);
        // A NUL ends the string there, as it did in the C buffer.
        text.push_bytes(&buf[..buflen]);
    }
    result.write_string(Some(text.into()));
}
