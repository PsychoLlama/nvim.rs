//! Assembling a string option's new value for `+=`, `^=` and `-=`.
//!
//! [`stropt_get_newval`] is the entry point: it is handed the option's
//! current value, the operator, and a cursor into the `:set` argument, and
//! answers with a freshly allocated value the caller takes ownership of.
//!
//! **The size contract, which nothing states and everything depends on.**
//! [`stropt_copy_value`] allocates the buffer, and it allocates
//! `strlen(arg) + 1` for a plain `=`, plus `strlen(origval) + 1` on top for
//! any other operator. Every function below writes *into that buffer in
//! place*, so all of them are sized by that one decision: the concatenation
//! can hold both values and one separating comma, and the removal and
//! key-matching passes only ever shrink or rearrange what is already there.
//! An operator that could produce anything longer would have to change the
//! allocation first.
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

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use crate::ascii::ascii_iswhite;
use crate::mbyte::utfc_ptr2len;
use crate::memory::{xfree, xmalloc, xstrdup};
use crate::options::kOptKeywordprg;
use crate::os::cshim::memmove;
use crate::strings::vim_strchr;
use crate::types::{NUL, OptIndex, size_t, uint32_t};
use ::libc::{strcpy, strlen};

use super::{
    OP_ADDING, OP_NONE, OP_PREPENDING, OP_REMOVING, OptSlot, find_dup_item, kOptFlagColon,
    kOptFlagComma, kOptFlagFlagList, kOptFlagNoDup, kOptFlagOneComma, option_expand, option_var,
    set_op_T,
};

/// `memmove` between two points of the same value buffer, without the two
/// `void *` casts that make every call site here go vertical.
///
/// # Safety
///
/// `dst` and `src` must be `n` bytes of one live buffer.
unsafe fn shift(dst: *mut c_char, src: *const c_char, n: size_t) {
    // SAFETY: the caller's buffer.
    unsafe { memmove(dst.cast::<c_void>(), src.cast::<c_void>(), n) };
}

/// How much room the assembled value needs: the argument, plus the current
/// value when the operator keeps it, plus a terminator for each.
///
/// # Safety
/// Both are C strings.
unsafe fn room_for(arg: *const c_char, origval: *const c_char, op: set_op_T) -> size_t {
    // SAFETY: the caller guarantees C strings.
    let mut room = unsafe { strlen(arg) } + 1;
    if op != OP_NONE {
        room += unsafe { strlen(origval) } + 1;
    }
    room
}

/// Copy the `:set` argument into a buffer of its own, dropping the
/// backslashes that escape whitespace and separators, and leave `argp` on
/// the first byte that is not part of the value.
///
/// `set_option_direct` cannot be used for this precisely because it would
/// keep those backslashes. The reverse transformation is
/// `escape_option_str_cmdline`.
///
/// # Safety
/// `origval` is a C string and `argp` points at a cursor into the `:set`
/// argument, which is one too.
pub(crate) unsafe fn stropt_copy_value(
    origval: *const c_char,
    argp: *mut *mut c_char,
    op: set_op_T,
    _flags: uint32_t,
) -> *mut c_char {
    // SAFETY: the caller's cursor and value.
    let mut arg = unsafe { *argp };
    let newval = unsafe { xmalloc(room_for(arg, origval, op)) }.cast::<c_char>();
    let mut s = newval;
    while c_int::from(unsafe { *arg }) != NUL && !ascii_iswhite(c_int::from(unsafe { *arg })) {
        if unsafe { *arg } == b'\\' as c_char && c_int::from(unsafe { *arg.add(1) }) != NUL {
            arg = unsafe { arg.add(1) }; // Remove the backslash.
        }
        let len = unsafe { utfc_ptr2len(arg) } as usize;
        unsafe { shift(s, arg, len) };
        arg = unsafe { arg.add(len) };
        s = unsafe { s.add(len) };
    }
    unsafe { *s = NUL as c_char };
    unsafe { *argp = arg };
    newval
}

/// Expand any environment variables the value names, into a buffer sized by
/// the same rule as the original.
///
/// Returns the value unchanged when there was nothing to expand.
///
/// # Safety
/// `origval` is a C string and `newval` an allocation this takes ownership
/// of.
pub(crate) unsafe fn stropt_expand_envvar(
    opt_idx: OptIndex,
    origval: *const c_char,
    newval: *mut c_char,
    op: set_op_T,
) -> *mut c_char {
    // SAFETY: `option_expand` reads the value and answers an owned copy,
    // or `None` when nothing expanded.
    let Some(expanded) = (unsafe { option_expand(opt_idx, newval) }) else {
        return newval;
    };
    unsafe { xfree(newval.cast::<c_void>()) };
    let room = unsafe { room_for(expanded.as_ptr(), origval, op) };
    let grown = unsafe { xmalloc(room) }.cast::<c_char>();
    unsafe { strcpy(grown, expanded.as_ptr()) };
    grown
}

/// Join the current value and the new one in place, with a comma between
/// them for a comma-separated option.
///
/// A trailing comma on the current value is absorbed rather than doubled,
/// but only for an option whose items are separated by exactly one comma —
/// and only when that comma is not itself escaped.
///
/// # Safety
/// `newval` is the assembled-value buffer, sized as the module docs
/// describe, and holds the new part; `origval` is a C string.
pub(crate) unsafe fn stropt_concat_with_comma(
    origval: *const c_char,
    newval: *mut c_char,
    op: set_op_T,
    flags: uint32_t,
) {
    // SAFETY: the caller's buffer and value, as documented above.
    let separated = flags & kOptFlagComma as uint32_t != 0
        && c_int::from(unsafe { *origval }) != NUL
        && c_int::from(unsafe { *newval }) != NUL;
    let comma = usize::from(separated);
    let len = if op == OP_ADDING {
        let mut len = unsafe { strlen(origval) };
        if separated
            && len > 1
            && flags & kOptFlagOneComma as uint32_t == kOptFlagOneComma as uint32_t
            && unsafe { *origval.add(len - 1) } == b',' as c_char
            && unsafe { *origval.add(len - 2) } != b'\\' as c_char
        {
            len -= 1;
        }
        // Shift the new part along and put the current value in front.
        unsafe { shift(newval.add(len + comma), newval, strlen(newval) + 1) };
        unsafe { shift(newval, origval, len) };
        len
    } else {
        // Prepending: the new part is already in front.
        let len = unsafe { strlen(newval) };
        unsafe { shift(newval.add(len + comma), origval, strlen(origval) + 1) };
        len
    };
    if separated {
        unsafe { *newval.add(len) = b',' as c_char };
    }
}

/// Copy the current value into the buffer with `strval[..len]` cut out of
/// it.
///
/// For a comma-separated option the cut takes a separating comma with it —
/// the one after the item when it is the first, the one before it
/// otherwise — so that the result does not end up with an empty item.
///
/// # Safety
/// `newval` is the assembled-value buffer, `origval` a C string, and
/// `strval` points into `origval` at `len` bytes of it.
pub(crate) unsafe fn stropt_remove_val(
    origval: *const c_char,
    newval: *mut c_char,
    flags: uint32_t,
    strval: *const c_char,
    len: c_int,
) {
    // SAFETY: the caller's buffer and value, as documented above.
    unsafe { strcpy(newval, origval) };
    if c_int::from(unsafe { *strval }) == NUL {
        return;
    }
    let (mut strval, mut len) = (strval, len as usize);
    if flags & kOptFlagComma as uint32_t != 0 {
        if strval == origval {
            if unsafe { *strval.add(len) } == b',' as c_char {
                len += 1;
            }
        } else {
            strval = unsafe { strval.sub(1) };
            len += 1;
        }
    }
    let at = unsafe { strval.offset_from(origval) } as usize;
    unsafe { shift(newval.add(at), strval.add(len), strlen(strval.add(len)) + 1) };
}

/// Find the item of `src` that opens with `key`, and report how long it is.
///
/// An item only counts at the start of the value or just after a comma, so
/// that "ab:1" is not found by the key "b:".
///
/// # Safety
/// `src` and `key` are C strings, `keylen` is within `key`, and `itemlenp`
/// is writable.
pub(crate) unsafe fn find_key_item(
    src: *mut c_char,
    key: *mut c_char,
    keylen: isize,
    itemlenp: *mut isize,
) -> *mut c_char {
    // SAFETY: the caller's strings, walked to the terminator.
    let mut p = src;
    while c_int::from(unsafe { *p }) != NUL {
        if (p == src || unsafe { *p.sub(1) } == b',' as c_char)
            && unsafe { cstr::prefix_eq(p, key, keylen as size_t) }
        {
            let mut end = unsafe { vim_strchr(p, c_int::from(b',')) };
            if end.is_null() {
                end = unsafe { p.add(strlen(p)) };
            }
            unsafe { *itemlenp = end.offset_from(p) };
            return p;
        }
        p = unsafe { p.add(1) };
    }
    ptr::null_mut()
}

/// Cut one item out of a comma-separated value in place, taking whichever
/// of its neighbouring commas exists with it.
///
/// # Safety
/// `item` points at `itemlen` bytes inside the C string starting at `str`.
pub(crate) unsafe fn remove_comma_item(str: *const c_char, item: *mut c_char, itemlen: isize) {
    // SAFETY: the caller's string, as documented above.
    let after = unsafe { item.offset(itemlen) };
    if unsafe { *after } == b',' as c_char {
        unsafe { shift(item, after.add(1), strlen(after.add(1)) + 1) };
    } else if item > str.cast_mut() && unsafe { *item.sub(1) } == b',' as c_char {
        unsafe { shift(item.sub(1), after, strlen(after) + 1) };
    } else {
        // The only item there was.
        unsafe { *item = NUL as c_char };
    }
}

/// Cut every item with this key out of the value, except the one at `skip`.
///
/// # Safety
/// `str` and `key` are C strings; `skip` is null or points into `str`.
pub(crate) unsafe fn remove_key_item(
    str: *mut c_char,
    key: *mut c_char,
    keylen: isize,
    skip: *const c_char,
) {
    // SAFETY: the caller's strings, as documented above.
    loop {
        let mut itemlen: isize = 0;
        let mut found = unsafe { find_key_item(str, key, keylen, &raw mut itemlen) };
        if found.is_null() {
            return;
        }
        if found == skip.cast_mut() {
            // Look past the one being kept, for a second with the same
            // key.
            let mut next = unsafe { found.offset(itemlen) };
            if unsafe { *next } == b',' as c_char {
                next = unsafe { next.add(1) };
            }
            found = unsafe { find_key_item(next, key, keylen, &raw mut itemlen) };
            if found.is_null() {
                return;
            }
        }
        unsafe { remove_comma_item(str, found, itemlen) };
    }
}

/// Add `item` to the end of a comma-separated value in place.
///
/// # Safety
/// `str` is a C string with room for `item_len` more bytes, a comma and a
/// terminator; `item` has at least `item_len` bytes.
pub(crate) unsafe fn append_item(str: *mut c_char, item: *mut c_char, item_len: isize) {
    // SAFETY: the caller's buffer, as documented above.
    let mut len = unsafe { strlen(str) } as isize;
    if len > 0 {
        unsafe { *str.offset(len) = b',' as c_char };
        len += 1;
    }
    unsafe { shift(str.offset(len), item, item_len as size_t) };
    unsafe { *str.offset(len + item_len) = NUL as c_char };
}

/// Add `item` to the front of a comma-separated value in place.
///
/// # Safety
/// As [`append_item`].
pub(crate) unsafe fn prepend_item(str: *mut c_char, item: *mut c_char, item_len: isize) {
    // SAFETY: the caller's buffer, as documented above.
    let len = unsafe { strlen(str) };
    let comma = usize::from(len > 0);
    unsafe { shift(str.offset(item_len).add(comma), str, len + 1) };
    unsafe { shift(str, item, item_len as size_t) };
    if comma != 0 {
        unsafe { *str.offset(item_len) = b',' as c_char };
    }
}

/// Assemble a key-value list, where an added `key:value` *replaces* the
/// item with the same key instead of sitting beside it.
///
/// Returns false — and changes nothing — for an argument that is neither
/// keyed nor a list, which is how a plain value falls through to the
/// ordinary concatenation.
///
/// The buffer is rebuilt from the current value and the argument is walked
/// item by item out of a copy, because the argument is *in* the buffer
/// being rewritten.
///
/// # Safety
/// `origval` is a C string and `newval` the assembled-value buffer holding
/// the argument.
pub(crate) unsafe fn stropt_handle_keymatch(
    origval: *const c_char,
    newval: *mut c_char,
    op: set_op_T,
    _flags: uint32_t,
) -> bool {
    // SAFETY: the caller's buffer and value, as documented above.
    if unsafe { vim_strchr(newval, c_int::from(b':')) }.is_null()
        && unsafe { vim_strchr(newval, c_int::from(b',')) }.is_null()
    {
        return false;
    }
    let argument = unsafe { xstrdup(newval) };
    unsafe { strcpy(newval, origval) };

    let mut item_start = argument;
    loop {
        let next = unsafe { vim_strchr(item_start, c_int::from(b',')) };
        let item_len = if next.is_null() {
            unsafe { strlen(item_start) as isize }
        } else {
            unsafe { next.offset_from(item_start) }
        };
        if item_len > 0 {
            let colon = unsafe { vim_strchr(item_start, c_int::from(b':')) };
            let keyed = !colon.is_null() && colon < unsafe { item_start.offset(item_len) };
            if keyed {
                // The key is everything up to and including the colon.
                let keylen = unsafe { colon.offset_from(item_start) } + 1;
                match op {
                    OP_ADDING | OP_PREPENDING => {
                        let mut old_itemlen: isize = 0;
                        let found = unsafe {
                            find_key_item(newval, item_start, keylen, &raw mut old_itemlen)
                        };
                        if found.is_null() {
                            unsafe { place(newval, item_start, item_len, op) };
                        } else if old_itemlen == item_len
                            && unsafe { cstr::prefix_eq(found, item_start, item_len as size_t) }
                        {
                            // The same item already: keep it where it
                            // is, and drop any later duplicate.
                            unsafe { remove_key_item(newval, item_start, keylen, found) };
                        } else {
                            unsafe { remove_key_item(newval, item_start, keylen, ptr::null()) };
                            unsafe { place(newval, item_start, item_len, op) };
                        }
                    }
                    OP_REMOVING => {
                        unsafe { remove_key_item(newval, item_start, keylen, ptr::null()) };
                    }
                    _ => {}
                }
            } else {
                // An item with no key of its own is matched whole.
                let found = unsafe {
                    find_dup_item(
                        newval,
                        item_start,
                        item_len as size_t,
                        kOptFlagComma as uint32_t,
                    )
                };
                match op {
                    OP_ADDING | OP_PREPENDING if found.is_null() => {
                        unsafe { place(newval, item_start, item_len, op) };
                    }
                    OP_REMOVING if !found.is_null() => {
                        unsafe { remove_comma_item(newval, found.cast_mut(), item_len) };
                    }
                    _ => {}
                }
            }
        }
        if next.is_null() {
            break;
        }
        item_start = unsafe { next.add(1) };
    }
    unsafe { xfree(argument.cast::<c_void>()) };
    true
}

/// Put an item at whichever end the operator asks for.
///
/// # Safety
/// As [`append_item`].
unsafe fn place(str: *mut c_char, item: *mut c_char, item_len: isize, op: set_op_T) {
    // SAFETY: the caller's buffer.
    if op == OP_PREPENDING {
        unsafe { prepend_item(str, item, item_len) };
    } else {
        unsafe { append_item(str, item, item_len) };
    }
}

/// Drop every repeated flag letter, keeping the last of each.
///
/// A one-comma option's letters may each be followed by a comma, so the
/// letter and its comma are dropped together.
///
/// # Safety
/// `newval` is a C string this may shorten in place.
pub(crate) unsafe fn stropt_remove_dupflags(newval: *mut c_char, flags: uint32_t) {
    let one_comma = flags & kOptFlagOneComma as uint32_t != 0;
    let comma_list = flags & kOptFlagComma as uint32_t != 0;
    // SAFETY: the caller's string, walked to the terminator; each cut moves
    // the tail (including its terminator) down over the letter dropped.
    let mut s = newval;
    while unsafe { *s } != 0 {
        let letter = c_int::from(unsafe { *s } as u8);
        let drop = if one_comma {
            (unsafe { *s }) != b',' as c_char
                && unsafe { *s.add(1) } == b',' as c_char
                && !unsafe { vim_strchr(s.add(2), letter) }.is_null()
        } else {
            (!comma_list || unsafe { *s } != b',' as c_char)
                && !unsafe { vim_strchr(s.add(1), letter) }.is_null()
        };
        if !drop {
            s = unsafe { s.add(1) };
            continue;
        }
        let past = if one_comma {
            unsafe { s.add(2) }
        } else {
            unsafe { s.add(1) }
        };
        unsafe { shift(s, past, strlen(past) + 1) };
    }
}

/// Assemble the value a `:set` argument asks for, and answer with a fresh
/// allocation the caller owns.
///
/// `op_arg` may come back as `OP_NONE` even for a `+=`: adding something
/// the value already carries is a no-op, and the caller uses that to skip
/// the whole set.
///
/// # Safety
/// `argp` points at a cursor into the `:set` argument, `varp` is the
/// option's variable in the scope being set, `origval` is its current
/// value, and `op_arg` is writable.
pub(crate) unsafe fn stropt_get_newval(
    opt_idx: OptIndex,
    argp: *mut *mut c_char,
    varp: OptSlot,
    origval: *const c_char,
    op_arg: *mut set_op_T,
    flags: uint32_t,
) -> *mut c_char {
    // A bare `:set keywordprg=` means ":help", not the empty string — but
    // only for the global value. `:setlocal keywordprg=` unsets the buffer's
    // copy, which is how it goes back to reading the global one.
    let global_kp = opt_idx == kOptKeywordprg && varp == option_var(opt_idx);
    // SAFETY: the caller's cursor, variable and value, as documented above.
    // Past the '=' or the operator's second character.
    let mut arg = unsafe { (*argp).add(1) };
    let mut op = unsafe { *op_arg };

    let bare = c_int::from(unsafe { *arg }) == NUL || unsafe { *arg } == b' ' as c_char;
    let save_arg = if global_kp && bare {
        let save = arg;
        arg = c":help".as_ptr().cast_mut();
        Some(save)
    } else {
        None
    };

    let mut newval = unsafe { stropt_copy_value(origval, &raw mut arg, op, flags) };
    // Only a whole value or a comma-separated one is expanded; a `+=`
    // on a non-list option would expand the concatenation.
    if op == OP_NONE || flags & kOptFlagComma as uint32_t != 0 {
        newval = unsafe { stropt_expand_envvar(opt_idx, origval, newval, op) };
    }

    let keyed = flags & kOptFlagComma as uint32_t != 0
        && flags & kOptFlagColon as uint32_t != 0
        && op != OP_NONE;
    if !(keyed && unsafe { stropt_handle_keymatch(origval, newval, op, flags) }) {
        let mut len = 0;
        let mut at: *const c_char = ptr::null();
        if op == OP_REMOVING || flags & kOptFlagNoDup as uint32_t != 0 {
            len = unsafe { strlen(newval) } as c_int;
            at = unsafe { find_dup_item(origval, newval, len as size_t, flags) };
            // Adding something already there changes nothing.
            if (op == OP_ADDING || op == OP_PREPENDING) && !at.is_null() {
                op = OP_NONE;
                unsafe { strcpy(newval, origval) };
            }
            if at.is_null() {
                // Removing something that is not there cuts an empty
                // span off the end.
                at = unsafe { origval.add(strlen(origval)) };
            }
        }
        if op == OP_ADDING || op == OP_PREPENDING {
            unsafe { stropt_concat_with_comma(origval, newval, op, flags) };
        } else if op == OP_REMOVING {
            unsafe { stropt_remove_val(origval, newval, flags, at, len) };
        }
    }

    if flags & kOptFlagFlagList as uint32_t != 0 {
        unsafe { stropt_remove_dupflags(newval, flags) };
    }

    unsafe { *argp = save_arg.unwrap_or(arg) };
    unsafe { *op_arg = op };
    newval
}
