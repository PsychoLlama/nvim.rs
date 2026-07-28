//! Assembling a string option's new value for `+=`, `^=` and `-=`.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn stropt_copy_value(
    mut origval: *const c_char,
    mut argp: *mut *mut c_char,
    mut op: set_op_T,
    mut _flags: uint32_t,
) -> *mut c_char {
    let mut arg: *mut c_char = *argp;
    let mut newlen: size_t = strlen(arg).wrapping_add(1 as size_t);
    if op as c_uint != OP_NONE as c_int as c_uint {
        newlen = newlen.wrapping_add(strlen(origval).wrapping_add(1 as size_t));
    }
    let mut newval: *mut c_char = xmalloc(newlen) as *mut c_char;
    let mut s: *mut c_char = newval;
    while *arg as c_int != NUL && !ascii_iswhite(*arg as c_int) {
        if *arg as c_int == '\\' as c_int && *arg.offset(1 as c_int as isize) as c_int != NUL {
            arg = arg.offset(1);
        }
        let mut i: c_int = utfc_ptr2len(arg);
        if i > 1 as c_int {
            memmove(s as *mut c_void, arg as *const c_void, i as size_t);
            arg = arg.offset(i as isize);
            s = s.offset(i as isize);
        } else {
            let c2rust_fresh4 = arg;
            arg = arg.offset(1);
            let c2rust_fresh5 = s;
            s = s.offset(1);
            *c2rust_fresh5 = *c2rust_fresh4;
        }
    }
    *s = NUL as c_char;
    *argp = arg;
    return newval;
}

pub(crate) unsafe extern "C" fn stropt_expand_envvar(
    mut opt_idx: OptIndex,
    mut origval: *const c_char,
    mut newval: *mut c_char,
    mut op: set_op_T,
) -> *mut c_char {
    let mut s: *mut c_char = option_expand(opt_idx, newval);
    if s.is_null() {
        return newval;
    }
    xfree(newval as *mut c_void);
    let mut newlen: uint32_t = (strlen(s) as uint32_t).wrapping_add(1 as uint32_t);
    if op as c_uint != OP_NONE as c_int as c_uint {
        newlen = (newlen as c_uint)
            .wrapping_add((strlen(origval) as c_uint).wrapping_add(1 as c_uint))
            as uint32_t;
    }
    newval = xmalloc(newlen as size_t) as *mut c_char;
    strcpy(newval, s);
    return newval;
}

pub(crate) unsafe extern "C" fn stropt_concat_with_comma(
    mut origval: *const c_char,
    mut newval: *mut c_char,
    mut op: set_op_T,
    mut flags: uint32_t,
) {
    let mut len: c_int = 0 as c_int;
    let mut comma: c_int = (flags & kOptFlagComma as c_int as uint32_t != 0
        && *origval as c_int != NUL
        && *newval as c_int != NUL) as c_int;
    if op as c_uint == OP_ADDING as c_int as c_uint {
        len = strlen(origval) as c_int;
        if comma != 0
            && len > 1 as c_int
            && flags & kOptFlagOneComma as c_int as uint32_t
                == kOptFlagOneComma as c_int as uint32_t
            && *origval.offset((len - 1 as c_int) as isize) as c_int == ',' as c_int
            && *origval.offset((len - 2 as c_int) as isize) as c_int != '\\' as c_int
        {
            len -= 1;
        }
        memmove(
            newval.offset(len as isize).offset(comma as isize) as *mut c_void,
            newval as *const c_void,
            strlen(newval).wrapping_add(1 as size_t),
        );
        memmove(
            newval as *mut c_void,
            origval as *const c_void,
            len as size_t,
        );
    } else {
        len = strlen(newval) as c_int;
        memmove(
            newval.offset(len as isize).offset(comma as isize) as *mut c_void,
            origval as *const c_void,
            strlen(origval).wrapping_add(1 as size_t),
        );
    }
    if comma != 0 {
        *newval.offset(len as isize) = ',' as c_char;
    }
}

pub(crate) unsafe extern "C" fn stropt_remove_val(
    mut origval: *const c_char,
    mut newval: *mut c_char,
    mut flags: uint32_t,
    mut strval: *const c_char,
    mut len: c_int,
) {
    strcpy(newval, origval as *mut c_char);
    if *strval != 0 {
        if flags & kOptFlagComma as c_int as uint32_t != 0 {
            if strval == origval {
                if *strval.offset(len as isize) as c_int == ',' as c_int {
                    len += 1;
                }
            } else {
                strval = strval.offset(-1);
                len += 1;
            }
        }
        memmove(
            newval.offset(strval.offset_from(origval) as isize) as *mut c_void,
            strval.offset(len as isize) as *const c_void,
            strlen(strval.offset(len as isize)).wrapping_add(1 as size_t),
        );
    }
}

pub(crate) unsafe extern "C" fn find_key_item(
    mut src: *mut c_char,
    mut key: *mut c_char,
    mut keylen: ptrdiff_t,
    mut itemlenp: *mut ptrdiff_t,
) -> *mut c_char {
    let mut p: *mut c_char = src;
    while *p as c_int != NUL {
        if (p == src || *p.offset(-(1 as c_int as isize)) as c_int == ',' as c_int)
            && strncmp(p, key, keylen as size_t) == 0 as c_int
        {
            let mut end: *mut c_char = vim_strchr(p, ',' as c_int);
            if end.is_null() {
                end = p.offset(strlen(p) as isize);
            }
            *itemlenp = end.offset_from(p) as ptrdiff_t;
            return p;
        }
        p = p.offset(1);
    }
    return ::core::ptr::null_mut::<c_char>();
}

pub(crate) unsafe extern "C" fn remove_comma_item(
    mut str: *const c_char,
    mut item: *mut c_char,
    mut itemlen: ptrdiff_t,
) {
    if *item.offset(itemlen as isize) as c_int == ',' as c_int {
        memmove(
            item as *mut c_void,
            item.offset(itemlen as isize).offset(1 as c_int as isize) as *const c_void,
            strlen(item.offset(itemlen as isize).offset(1 as c_int as isize))
                .wrapping_add(1 as size_t),
        );
    } else if item > str as *mut c_char
        && *item.offset(-(1 as c_int as isize)) as c_int == ',' as c_int
    {
        memmove(
            item.offset(-(1 as c_int as isize)) as *mut c_void,
            item.offset(itemlen as isize) as *const c_void,
            strlen(item.offset(itemlen as isize)).wrapping_add(1 as size_t),
        );
    } else {
        *item = NUL as c_char;
    };
}

pub(crate) unsafe extern "C" fn remove_key_item(
    mut str: *mut c_char,
    mut key: *mut c_char,
    mut keylen: ptrdiff_t,
    mut skip: *const c_char,
) {
    let mut itemlen: ptrdiff_t = 0;
    let mut found: *mut c_char = ::core::ptr::null_mut::<c_char>();
    loop {
        found = find_key_item(str, key, keylen, &raw mut itemlen);
        if found.is_null() {
            break;
        }
        if found == skip as *mut c_char {
            let mut next: *mut c_char = found.offset(itemlen as isize);
            if *next as c_int == ',' as c_int {
                next = next.offset(1);
            }
            found = find_key_item(next, key, keylen, &raw mut itemlen);
            if found.is_null() {
                break;
            }
        }
        remove_comma_item(str, found, itemlen);
    }
}

pub(crate) unsafe extern "C" fn append_item(
    mut str: *mut c_char,
    mut item: *mut c_char,
    mut item_len: ptrdiff_t,
) {
    let mut len: ptrdiff_t = strlen(str) as ptrdiff_t;
    if len > 0 as ptrdiff_t {
        let c2rust_fresh3 = len;
        len = len + 1;
        *str.offset(c2rust_fresh3 as isize) = ',' as c_char;
    }
    memmove(
        str.offset(len as isize) as *mut c_void,
        item as *const c_void,
        item_len as size_t,
    );
    *str.offset((len + item_len) as isize) = NUL as c_char;
}

pub(crate) unsafe extern "C" fn prepend_item(
    mut str: *mut c_char,
    mut item: *mut c_char,
    mut item_len: ptrdiff_t,
) {
    let mut len: ptrdiff_t = strlen(str) as ptrdiff_t;
    let mut comma: c_int = if len > 0 as ptrdiff_t {
        1 as c_int
    } else {
        0 as c_int
    };
    memmove(
        str.offset(item_len as isize).offset(comma as isize) as *mut c_void,
        str as *const c_void,
        (len as size_t).wrapping_add(1 as size_t),
    );
    memmove(
        str as *mut c_void,
        item as *const c_void,
        item_len as size_t,
    );
    if comma != 0 {
        *str.offset(item_len as isize) = ',' as c_char;
    }
}

pub(crate) unsafe extern "C" fn stropt_handle_keymatch(
    mut origval: *const c_char,
    mut newval: *mut c_char,
    mut op: set_op_T,
    mut _flags: uint32_t,
) -> bool {
    if vim_strchr(newval, ':' as c_int).is_null() && vim_strchr(newval, ',' as c_int).is_null() {
        return false_0 != 0;
    }
    let mut newval_copy: *mut c_char = xstrdup(newval);
    strcpy(newval, origval as *mut c_char);
    let mut item_start: *mut c_char = newval_copy;
    loop {
        let mut p: *mut c_char = vim_strchr(item_start, ',' as c_int);
        let mut item_len: ptrdiff_t = if p.is_null() {
            strlen(item_start) as ptrdiff_t
        } else {
            p.offset_from(item_start)
        };
        if item_len > 0 as ptrdiff_t {
            let mut colon: *mut c_char = vim_strchr(item_start, ':' as c_int);
            if !colon.is_null() && colon < item_start.offset(item_len as isize) {
                let mut keylen: ptrdiff_t = colon.offset_from(item_start) + 1 as ptrdiff_t;
                if op as c_uint == OP_ADDING as c_int as c_uint
                    || op as c_uint == OP_PREPENDING as c_int as c_uint
                {
                    let mut old_itemlen: ptrdiff_t = 0;
                    let mut found: *mut c_char =
                        find_key_item(newval, item_start, keylen, &raw mut old_itemlen);
                    if !found.is_null() {
                        if old_itemlen == item_len
                            && strncmp(found, item_start, item_len as size_t) == 0 as c_int
                        {
                            remove_key_item(newval, item_start, keylen, found);
                        } else {
                            remove_key_item(
                                newval,
                                item_start,
                                keylen,
                                ::core::ptr::null::<c_char>(),
                            );
                            if op as c_uint == OP_PREPENDING as c_int as c_uint {
                                prepend_item(newval, item_start, item_len);
                            } else {
                                append_item(newval, item_start, item_len);
                            }
                        }
                    } else if op as c_uint == OP_PREPENDING as c_int as c_uint {
                        prepend_item(newval, item_start, item_len);
                    } else {
                        append_item(newval, item_start, item_len);
                    }
                } else if op as c_uint == OP_REMOVING as c_int as c_uint {
                    remove_key_item(newval, item_start, keylen, ::core::ptr::null::<c_char>());
                }
            } else if op as c_uint == OP_ADDING as c_int as c_uint
                || op as c_uint == OP_PREPENDING as c_int as c_uint
            {
                let mut found_0: *const c_char = find_dup_item(
                    newval,
                    item_start,
                    item_len as size_t,
                    kOptFlagComma as c_int as uint32_t,
                );
                if found_0.is_null() {
                    if op as c_uint == OP_PREPENDING as c_int as c_uint {
                        prepend_item(newval, item_start, item_len);
                    } else {
                        append_item(newval, item_start, item_len);
                    }
                }
            } else if op as c_uint == OP_REMOVING as c_int as c_uint {
                let mut found_1: *mut c_char = find_dup_item(
                    newval,
                    item_start,
                    item_len as size_t,
                    kOptFlagComma as c_int as uint32_t,
                ) as *mut c_char;
                if !found_1.is_null() {
                    remove_comma_item(newval, found_1, item_len);
                }
            }
        }
        if p.is_null() {
            break;
        }
        item_start = p.offset(1 as c_int as isize);
    }
    xfree(newval_copy as *mut c_void);
    return true_0 != 0;
}

pub(crate) unsafe extern "C" fn stropt_remove_dupflags(
    mut newval: *mut c_char,
    mut flags: uint32_t,
) {
    let mut s: *mut c_char = newval;
    s = newval;
    while *s != 0 {
        if flags & kOptFlagOneComma as c_int as uint32_t != 0 {
            if *s as c_int != ',' as c_int
                && *s.offset(1 as c_int as isize) as c_int == ',' as c_int
                && !vim_strchr(s.offset(2 as c_int as isize), *s as uint8_t as c_int).is_null()
            {
                memmove(
                    s as *mut c_void,
                    s.offset(2 as c_int as isize) as *const c_void,
                    strlen(s.offset(2 as c_int as isize)).wrapping_add(1 as size_t),
                );
                continue;
            }
        } else if (flags & kOptFlagComma as c_int as uint32_t == 0 || *s as c_int != ',' as c_int)
            && !vim_strchr(s.offset(1 as c_int as isize), *s as uint8_t as c_int).is_null()
        {
            memmove(
                s as *mut c_void,
                s.offset(1 as c_int as isize) as *const c_void,
                strlen(s.offset(1 as c_int as isize)).wrapping_add(1 as size_t),
            );
            continue;
        }
        s = s.offset(1);
    }
}

pub(crate) unsafe extern "C" fn stropt_get_newval(
    mut _nextchar: c_int,
    mut opt_idx: OptIndex,
    mut argp: *mut *mut c_char,
    mut varp: *mut c_void,
    mut origval: *const c_char,
    mut op_arg: *mut set_op_T,
    mut flags: uint32_t,
) -> *mut c_char {
    let mut arg: *mut c_char = *argp;
    let mut op: set_op_T = *op_arg;
    let mut save_arg: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut newval: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut s: *const c_char = ::core::ptr::null::<c_char>();
    arg = arg.offset(1);
    if varp == p_kp.ptr() as *mut c_void && (*arg as c_int == NUL || *arg as c_int == ' ' as c_int)
    {
        save_arg = arg;
        arg = b":help\0".as_ptr() as *const c_char as *mut c_char;
    }
    newval = stropt_copy_value(origval, &raw mut arg, op, flags);
    if op as c_uint == OP_NONE as c_int as c_uint || flags & kOptFlagComma as c_int as uint32_t != 0
    {
        newval = stropt_expand_envvar(opt_idx, origval, newval, op);
    }
    if !(flags & kOptFlagComma as c_int as uint32_t != 0
        && flags & kOptFlagColon as c_int as uint32_t != 0
        && op as c_uint != OP_NONE as c_int as c_uint
        && stropt_handle_keymatch(origval, newval, op, flags) as c_int != 0)
    {
        let mut len: c_int = 0 as c_int;
        if op as c_uint == OP_REMOVING as c_int as c_uint
            || flags & kOptFlagNoDup as c_int as uint32_t != 0
        {
            len = strlen(newval) as c_int;
            s = find_dup_item(origval, newval, len as size_t, flags);
            if (op as c_uint == OP_ADDING as c_int as c_uint
                || op as c_uint == OP_PREPENDING as c_int as c_uint)
                && !s.is_null()
            {
                op = OP_NONE;
                strcpy(newval, origval as *mut c_char);
            }
            if s.is_null() {
                s = origval.offset(strlen(origval) as c_int as isize);
            }
        }
        if op as c_uint == OP_ADDING as c_int as c_uint
            || op as c_uint == OP_PREPENDING as c_int as c_uint
        {
            stropt_concat_with_comma(origval, newval, op, flags);
        } else if op as c_uint == OP_REMOVING as c_int as c_uint {
            stropt_remove_val(origval, newval, flags, s, len);
        }
    }
    if flags & kOptFlagFlagList as c_int as uint32_t != 0 {
        stropt_remove_dupflags(newval, flags);
    }
    if !save_arg.is_null() {
        arg = save_arg;
    }
    *argp = arg;
    *op_arg = op;
    return newval;
}
