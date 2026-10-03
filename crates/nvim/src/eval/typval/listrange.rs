//! Ranges over a list: slicing, assigning through a slice, flattening, joining.
//!
//! [`list_check_range_index_one`] and [`list_check_range_index_two`]
//! are the bounds arithmetic `l[i:j]` shares with `l[i:j] = x`;
//! [`list_slice_or_index`] is the subscript itself.
//! [`list_join`] and [`list_join_inner`] are `join()`, which makes two
//! passes so the result buffer is sized once, and [`f_list2str`] is the
//! codepoint-list-to-string builtin.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::cstr;
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

/// `dest[idx1:idx2] = src`, or `dest[idx1:idx2] op= src` when `op` is given.
///
/// `empty_idx2` means the range had no upper bound (`dest[idx1:]`).
///
/// # Safety
///
/// `dest` and `src` must point at live lists, unaliased for the call. `op`
/// must be null or a NUL-terminated operator, and `varname` null or a NUL-
/// terminated name for the lock error; both live for the call.
pub unsafe fn list_assign_range(
    dest: *mut List,
    src: *mut List,
    idx1_arg: ::core::ffi::c_int,
    idx2: ::core::ffi::c_int,
    empty_idx2: bool,
    op: *const ::core::ffi::c_char,
    varname: *const ::core::ffi::c_char,
) -> Result<(), Failed> {
    let mut idx1 = idx1_arg;
    // SAFETY: the caller's promise: two live lists.
    let first = list_find_index(unsafe { dest.as_ref() }, &mut idx1);
    let srclen = list_len(unsafe { src.as_ref() }) as usize;

    // Check whether any of the list items is locked before making any
    // changes.  The walk stops at the end of the range or at the end of the
    // source, whichever comes first -- `dest` may be shorter, and the
    // assignment below grows it.
    let mut idx = idx1;
    let mut at = first;
    for i in 0..srclen {
        let Some(dest_at) = at else { break };
        // SAFETY: an index of `dest`, which nothing has edited yet.
        let lock = unsafe { (*dest).items() }[dest_at].li_lock;
        // SAFETY: the caller's promise about `varname`.
        if unsafe { value_check_lock(lock, varname, TV_CSTRING as size_t) } {
            return Err(Failed);
        }
        if i + 1 == srclen || (!empty_idx2 && idx2 == idx) {
            break;
        }
        // SAFETY: a live list.
        at = (dest_at + 1 < unsafe { (*dest).len() }).then_some(dest_at + 1);
        idx += 1;
    }

    // Assign the List values to the list items.
    idx = idx1;
    // `first` is `None` only for an empty target, which the caller's
    // `get_lval` has already refused; the guard below then leaves `i` at
    // zero and the E710 under it reports.
    let mut at = first.unwrap_or(0);
    let mut i = 0;
    // SAFETY: a live list.
    while i < srclen && at < unsafe { (*dest).len() } {
        // Both slots are re-derived on every step: `eexe_mod_op` runs the
        // evaluator and may have moved either array.  When `dest` *is*
        // `src` -- `:let l[0:1] += l[0:1]`, p30-7's documented aliasing --
        // both come from the *one* derivation of it, because two would pop
        // each other.
        // SAFETY: two live lists.
        let dbase = unsafe { (*dest).items_mut().as_mut_ptr() };
        let sbase = if ::core::ptr::eq(dest, src) {
            dbase
        } else {
            unsafe { (*src).items_mut().as_mut_ptr() }
        };
        // SAFETY: `at` is an index of `dest` and `i` one of `src`.
        let (to, from) = unsafe {
            (
                &raw mut (*dbase.add(at)).li_tv,
                &raw mut (*sbase.add(i)).li_tv,
            )
        };
        if !op.is_null() && unsafe { *op } as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
            let _ = unsafe { eexe_mod_op(to, from, op) };
        } else {
            unsafe { tv_clear(&mut *to) };
            unsafe { tv_copy(&*from, &mut *to) };
        }
        i += 1;
        if i == srclen || (!empty_idx2 && idx2 == idx) {
            break;
        }
        // SAFETY: a live list.
        if at + 1 == unsafe { (*dest).len() } {
            // Need to add an empty item.
            unsafe { (*dest).push_number(0) };
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
        // SAFETY: a live list.
        at + 1 < unsafe { (*dest).len() }
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
/// [`list_extend`]'s branch rather than a second borrow.
pub fn list_flatten(list: &mut List, first: usize, maxitems: int64_t, maxdepth: int64_t) {
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
        let step = if let Some(inner) = list.items()[at].li_tv.as_list() {
            let before = list.len();
            // The item naming the nested list is taken out *without* being
            // released, and held until the splice is done: `inner` may be
            // the very list being flattened, or the item may hold its last
            // reference, and either way freeing it first would pull the
            // items out from under the copy.  That is what upstream's
            // `tv_list_drop_items`-then-`tv_clear` order bought.
            let held = list.take_range(at, at);
            // SAFETY: `inner` is the live list the taken item still names,
            // and may be `list` itself -- which is the branch's business.
            unsafe { list_extend(list, inner, Some(at)) };

            if maxdepth > 0 {
                // SAFETY: as above.
                let inner_len = int64_t::from(list_len(unsafe { inner.as_ref() }));
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
    let mut joined: Vec<String_0> = Vec::with_capacity(list_len(l).cast_unsigned() as usize);
    for item in list_iter(l) {
        if got_int.get() {
            break;
        }
        let mut len: size_t = 0;
        // SAFETY: a live item of the caller's list.
        let data = unsafe { encode_tv2echo(&item.li_tv, &raw mut len) };
        if data.is_null() {
            return Err(Failed);
        }
        // SAFETY: `encode_tv2echo` answers its own NUL-terminated block.
        joined.push(unsafe { String_0::from_owned_parts(data, len) });
        line_breakcheck();
    }

    let sep = sep.to_bytes();
    let total: usize = joined.iter().map(String_0::len).sum::<usize>()
        + sep.len() * joined.len().saturating_sub(1);
    let mut text = Vec::with_capacity(total);
    for (i, s) in joined.iter().enumerate() {
        if got_int.get() {
            break;
        }
        if i > 0 {
            text.extend_from_slice(sep);
        }
        text.extend_from_slice(s.as_bytes());
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
        c" ".as_ptr()
    } else {
        numbuf.string_ptr_chk(&args[1])
    };

    result.write_empty(VAR_STRING);
    if sep.is_null() {
        result.write_string(::core::ptr::null_mut());
        return;
    }

    let mut text = XString::new();
    // SAFETY: `string_ptr_chk` answered a NUL-terminated separator.
    let sep = unsafe { cstr::at(sep) };
    let _ = list_join(&mut text, args[0].list_ref(), sep);
    result.write_string(text.into_raw());
}

/// `list2str()`: a list of codepoints as a string.
pub fn f_list2str(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_string(::core::ptr::null_mut());
    // SAFETY: the builtin's argument array.
    let args = unsafe { Tv::new(core::ptr::from_ref(&args[0]).cast_mut()) };
    if args.v_type() != VAR_LIST {
        emsg(gettext(e_invarg));
        return;
    }
    let l = args.list_or_null();
    if l.is_null() {
        return;
    }

    let mut text = XString::new();
    let mut buf: [::core::ffi::c_char; 22] = [0; 22];
    // SAFETY: the builtin's own argument, borrowed for the walk.
    for li in list_iter(args.list_ref()) {
        let n = tv_get_number(&li.li_tv);
        let buflen = unsafe { utf_char2bytes(n as ::core::ffi::c_int, buf.as_mut_ptr()) } as usize;
        // A NUL ends the string there, as it did in the C buffer.
        text.push_bytes(&buf.map(|c| c.cast_unsigned())[..buflen]);
    }
    result.write_string(text.into_raw());
}
