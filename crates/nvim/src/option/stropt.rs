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

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use crate::ascii::ascii_iswhite;
use crate::main::p_kp;
use crate::mbyte::utfc_ptr2len;
use crate::memory::{xfree, xmalloc, xstrdup};
use crate::os::cshim::{memmove, strncmp};
use crate::strings::vim_strchr;
use crate::types::{NUL, OptIndex, size_t, uint32_t};
use ::libc::{strcpy, strlen};

use super::{
    OP_ADDING, OP_NONE, OP_PREPENDING, OP_REMOVING, find_dup_item, kOptFlagColon, kOptFlagComma,
    kOptFlagFlagList, kOptFlagNoDup, kOptFlagOneComma, option_expand, set_op_T,
};

/// How much room the assembled value needs: the argument, plus the current
/// value when the operator keeps it, plus a terminator for each.
///
/// # Safety
/// Both are C strings.
unsafe fn room_for(arg: *const c_char, origval: *const c_char, op: set_op_T) -> size_t {
    // SAFETY: the caller guarantees C strings.
    unsafe {
        let mut room = strlen(arg) + 1;
        if op != OP_NONE {
            room += strlen(origval) + 1;
        }
        room
    }
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
    unsafe {
        let mut arg = *argp;
        let newval = xmalloc(room_for(arg, origval, op)).cast::<c_char>();
        let mut s = newval;
        while c_int::from(*arg) != NUL && !ascii_iswhite(c_int::from(*arg)) {
            if *arg == b'\\' as c_char && c_int::from(*arg.add(1)) != NUL {
                arg = arg.add(1); // Remove the backslash.
            }
            let len = utfc_ptr2len(arg) as usize;
            memmove(s.cast::<c_void>(), arg.cast::<c_void>(), len);
            arg = arg.add(len);
            s = s.add(len);
        }
        *s = NUL as c_char;
        *argp = arg;
        newval
    }
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
    // SAFETY: `option_expand` reads the value and answers with a pointer
    // into its own scratch buffer, or null when nothing expanded.
    unsafe {
        let expanded = option_expand(opt_idx, newval);
        if expanded.is_null() {
            return newval;
        }
        xfree(newval.cast::<c_void>());
        let room = room_for(expanded, origval, op);
        let grown = xmalloc(room).cast::<c_char>();
        strcpy(grown, expanded);
        grown
    }
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
    unsafe {
        let separated = flags & kOptFlagComma as uint32_t != 0
            && c_int::from(*origval) != NUL
            && c_int::from(*newval) != NUL;
        let comma = usize::from(separated);
        let len = if op == OP_ADDING {
            let mut len = strlen(origval);
            if separated
                && len > 1
                && flags & kOptFlagOneComma as uint32_t == kOptFlagOneComma as uint32_t
                && *origval.add(len - 1) == b',' as c_char
                && *origval.add(len - 2) != b'\\' as c_char
            {
                len -= 1;
            }
            // Shift the new part along and put the current value in front.
            memmove(
                newval.add(len + comma).cast::<c_void>(),
                newval.cast::<c_void>(),
                strlen(newval) + 1,
            );
            memmove(newval.cast::<c_void>(), origval.cast::<c_void>(), len);
            len
        } else {
            // Prepending: the new part is already in front.
            let len = strlen(newval);
            memmove(
                newval.add(len + comma).cast::<c_void>(),
                origval.cast::<c_void>(),
                strlen(origval) + 1,
            );
            len
        };
        if separated {
            *newval.add(len) = b',' as c_char;
        }
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
    unsafe {
        strcpy(newval, origval);
        if c_int::from(*strval) == NUL {
            return;
        }
        let (mut strval, mut len) = (strval, len as usize);
        if flags & kOptFlagComma as uint32_t != 0 {
            if strval == origval {
                if *strval.add(len) == b',' as c_char {
                    len += 1;
                }
            } else {
                strval = strval.sub(1);
                len += 1;
            }
        }
        let at = strval.offset_from(origval) as usize;
        memmove(
            newval.add(at).cast::<c_void>(),
            strval.add(len).cast::<c_void>(),
            strlen(strval.add(len)) + 1,
        );
    }
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
    unsafe {
        let mut p = src;
        while c_int::from(*p) != NUL {
            if (p == src || *p.sub(1) == b',' as c_char) && strncmp(p, key, keylen as size_t) == 0 {
                let mut end = vim_strchr(p, c_int::from(b','));
                if end.is_null() {
                    end = p.add(strlen(p));
                }
                *itemlenp = end.offset_from(p);
                return p;
            }
            p = p.add(1);
        }
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
    unsafe {
        let after = item.offset(itemlen);
        if *after == b',' as c_char {
            memmove(
                item.cast::<c_void>(),
                after.add(1).cast::<c_void>(),
                strlen(after.add(1)) + 1,
            );
        } else if item > str.cast_mut() && *item.sub(1) == b',' as c_char {
            memmove(
                item.sub(1).cast::<c_void>(),
                after.cast::<c_void>(),
                strlen(after) + 1,
            );
        } else {
            // The only item there was.
            *item = NUL as c_char;
        }
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
    unsafe {
        loop {
            let mut itemlen: isize = 0;
            let mut found = find_key_item(str, key, keylen, &raw mut itemlen);
            if found.is_null() {
                return;
            }
            if found == skip.cast_mut() {
                // Look past the one being kept, for a second with the same
                // key.
                let mut next = found.offset(itemlen);
                if *next == b',' as c_char {
                    next = next.add(1);
                }
                found = find_key_item(next, key, keylen, &raw mut itemlen);
                if found.is_null() {
                    return;
                }
            }
            remove_comma_item(str, found, itemlen);
        }
    }
}

/// Add `item` to the end of a comma-separated value in place.
///
/// # Safety
/// `str` is a C string with room for `item_len` more bytes, a comma and a
/// terminator; `item` has at least `item_len` bytes.
pub(crate) unsafe fn append_item(str: *mut c_char, item: *mut c_char, item_len: isize) {
    // SAFETY: the caller's buffer, as documented above.
    unsafe {
        let mut len = strlen(str) as isize;
        if len > 0 {
            *str.offset(len) = b',' as c_char;
            len += 1;
        }
        memmove(
            str.offset(len).cast::<c_void>(),
            item.cast::<c_void>(),
            item_len as size_t,
        );
        *str.offset(len + item_len) = NUL as c_char;
    }
}

/// Add `item` to the front of a comma-separated value in place.
///
/// # Safety
/// As [`append_item`].
pub(crate) unsafe fn prepend_item(str: *mut c_char, item: *mut c_char, item_len: isize) {
    // SAFETY: the caller's buffer, as documented above.
    unsafe {
        let len = strlen(str);
        let comma = usize::from(len > 0);
        memmove(
            str.offset(item_len).add(comma).cast::<c_void>(),
            str.cast::<c_void>(),
            len + 1,
        );
        memmove(
            str.cast::<c_void>(),
            item.cast::<c_void>(),
            item_len as size_t,
        );
        if comma != 0 {
            *str.offset(item_len) = b',' as c_char;
        }
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
    unsafe {
        if vim_strchr(newval, c_int::from(b':')).is_null()
            && vim_strchr(newval, c_int::from(b',')).is_null()
        {
            return false;
        }
        let argument = xstrdup(newval);
        strcpy(newval, origval);

        let mut item_start = argument;
        loop {
            let next = vim_strchr(item_start, c_int::from(b','));
            let item_len = if next.is_null() {
                strlen(item_start) as isize
            } else {
                next.offset_from(item_start)
            };
            if item_len > 0 {
                let colon = vim_strchr(item_start, c_int::from(b':'));
                let keyed = !colon.is_null() && colon < item_start.offset(item_len);
                if keyed {
                    // The key is everything up to and including the colon.
                    let keylen = colon.offset_from(item_start) + 1;
                    match op {
                        OP_ADDING | OP_PREPENDING => {
                            let mut old_itemlen: isize = 0;
                            let found =
                                find_key_item(newval, item_start, keylen, &raw mut old_itemlen);
                            if found.is_null() {
                                place(newval, item_start, item_len, op);
                            } else if old_itemlen == item_len
                                && strncmp(found, item_start, item_len as size_t) == 0
                            {
                                // The same item already: keep it where it
                                // is, and drop any later duplicate.
                                remove_key_item(newval, item_start, keylen, found);
                            } else {
                                remove_key_item(newval, item_start, keylen, ptr::null());
                                place(newval, item_start, item_len, op);
                            }
                        }
                        OP_REMOVING => {
                            remove_key_item(newval, item_start, keylen, ptr::null());
                        }
                        _ => {}
                    }
                } else {
                    // An item with no key of its own is matched whole.
                    let found = find_dup_item(
                        newval,
                        item_start,
                        item_len as size_t,
                        kOptFlagComma as uint32_t,
                    );
                    match op {
                        OP_ADDING | OP_PREPENDING if found.is_null() => {
                            place(newval, item_start, item_len, op);
                        }
                        OP_REMOVING if !found.is_null() => {
                            remove_comma_item(newval, found.cast_mut(), item_len);
                        }
                        _ => {}
                    }
                }
            }
            if next.is_null() {
                break;
            }
            item_start = next.add(1);
        }
        xfree(argument.cast::<c_void>());
    }
    true
}

/// Put an item at whichever end the operator asks for.
///
/// # Safety
/// As [`append_item`].
unsafe fn place(str: *mut c_char, item: *mut c_char, item_len: isize, op: set_op_T) {
    // SAFETY: the caller's buffer.
    unsafe {
        if op == OP_PREPENDING {
            prepend_item(str, item, item_len);
        } else {
            append_item(str, item, item_len);
        }
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
    unsafe {
        let mut s = newval;
        while *s != 0 {
            let letter = c_int::from(*s as u8);
            let drop = if one_comma {
                *s != b',' as c_char
                    && *s.add(1) == b',' as c_char
                    && !vim_strchr(s.add(2), letter).is_null()
            } else {
                (!comma_list || *s != b',' as c_char) && !vim_strchr(s.add(1), letter).is_null()
            };
            if !drop {
                s = s.add(1);
                continue;
            }
            let past = if one_comma { s.add(2) } else { s.add(1) };
            memmove(s.cast::<c_void>(), past.cast::<c_void>(), strlen(past) + 1);
        }
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
/// `argp` points at a cursor into the `:set` argument, `varp` at the
/// option's variable, `origval` is its current value, and `op_arg` is
/// writable.
pub(crate) unsafe fn stropt_get_newval(
    _nextchar: c_int,
    opt_idx: OptIndex,
    argp: *mut *mut c_char,
    varp: *mut c_void,
    origval: *const c_char,
    op_arg: *mut set_op_T,
    flags: uint32_t,
) -> *mut c_char {
    // SAFETY: the caller's cursor, variable and value, as documented above.
    unsafe {
        // Past the '=' or the operator's second character.
        let mut arg = (*argp).add(1);
        let mut op = *op_arg;

        // A bare `:set keywordprg=` means ":help", not the empty string.
        let empty_kp = varp == p_kp.ptr().cast::<c_void>()
            && (c_int::from(*arg) == NUL || *arg == b' ' as c_char);
        let save_arg = if empty_kp {
            let save = arg;
            arg = c":help".as_ptr().cast_mut();
            Some(save)
        } else {
            None
        };

        let mut newval = stropt_copy_value(origval, &raw mut arg, op, flags);
        // Only a whole value or a comma-separated one is expanded; a `+=`
        // on a non-list option would expand the concatenation.
        if op == OP_NONE || flags & kOptFlagComma as uint32_t != 0 {
            newval = stropt_expand_envvar(opt_idx, origval, newval, op);
        }

        let keyed = flags & kOptFlagComma as uint32_t != 0
            && flags & kOptFlagColon as uint32_t != 0
            && op != OP_NONE;
        if !(keyed && stropt_handle_keymatch(origval, newval, op, flags)) {
            let mut len = 0;
            let mut at: *const c_char = ptr::null();
            if op == OP_REMOVING || flags & kOptFlagNoDup as uint32_t != 0 {
                len = strlen(newval) as c_int;
                at = find_dup_item(origval, newval, len as size_t, flags);
                // Adding something already there changes nothing.
                if (op == OP_ADDING || op == OP_PREPENDING) && !at.is_null() {
                    op = OP_NONE;
                    strcpy(newval, origval);
                }
                if at.is_null() {
                    // Removing something that is not there cuts an empty
                    // span off the end.
                    at = origval.add(strlen(origval));
                }
            }
            if op == OP_ADDING || op == OP_PREPENDING {
                stropt_concat_with_comma(origval, newval, op, flags);
            } else if op == OP_REMOVING {
                stropt_remove_val(origval, newval, flags, at, len);
            }
        }

        if flags & kOptFlagFlagList as uint32_t != 0 {
            stropt_remove_dupflags(newval, flags);
        }

        *argp = save_arg.unwrap_or(arg);
        *op_arg = op;
        newval
    }
}
