//! Whole-line reads and replacements.
//!
//! `nvim_buf_get_lines`/`nvim_buf_set_lines` are the line-granular half of
//! the buffer API, and `nvim_buf_get_text` the read that takes a byte range
//! but still hands back whole lines.  `push_linestr` is the shared
//! line-to-`String_0` step every one of them ends in, and
//! `buf_collect_lines` the loop over a range that `buffer_updates.rs`
//! reaches too, to build the `nvim_buf_attach` change notifications.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported, array_add};
use crate::normal::{visual_active, visual_anchor, with_visual_anchor};
use crate::types::NUL;

pub unsafe fn nvim_buf_line_count(buf: Buffer) -> Result<Integer, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return (0 as Integer).reported(error);
        }
        if (*b).b_ml.ml_mfp.is_null() {
            return (0 as Integer).reported(error);
        }
        ((*b).b_ml.ml_line_count as Integer).reported(error)
    }
}

pub unsafe fn nvim_buf_get_lines(
    channel_id: uint64_t,
    buf: Buffer,
    mut start: Integer,
    mut end: Integer,
    strict_indexing: Boolean,
    arena: *mut Arena,
    lstate: *mut lua_State,
) -> Result<Array, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut rv: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return rv.reported(error);
        }
        if (*b).b_ml.ml_mfp.is_null() {
            return rv.reported(error);
        }
        let mut oob: bool = false;
        start = normalize_index(b, start as int64_t, true, &raw mut oob) as Integer;
        end = normalize_index(b, end as int64_t, true, &raw mut oob) as Integer;
        if !(!strict_indexing || !oob) {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"%s".as_ptr(),
                c"Index out of bounds".as_ptr(),
            );
            return rv.reported(error);
        }
        if start >= end {
            return rv.reported(error);
        }
        let mut size: size_t = (end - start) as size_t;
        init_line_array(lstate, &raw mut rv, size, arena);
        buf_collect_lines(
            b,
            size,
            start as linenr_T,
            0 as ::core::ffi::c_int,
            channel_id != VIML_INTERNAL_CALL,
            &raw mut rv,
            lstate,
            arena,
        );
        rv.reported(error)
    }
}

pub unsafe fn nvim_buf_set_lines(
    channel_id: uint64_t,
    buf: Buffer,
    mut start: Integer,
    mut end: Integer,
    strict_indexing: Boolean,
    replacement: Array,
    arena: *mut Arena,
) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut b: *mut buf_T = api_buf_ensure_loaded(buf, err);
        if b.is_null() {
            return ().reported(error);
        }
        let mut oob: bool = false;
        start = normalize_index(b, start as int64_t, true, &raw mut oob) as Integer;
        end = normalize_index(b, end as int64_t, true, &raw mut oob) as Integer;
        if !(!strict_indexing || !oob) {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"%s".as_ptr(),
                c"Index out of bounds".as_ptr(),
            );
            return ().reported(error);
        }
        if !(start <= end) {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"%s".as_ptr(),
                c"'start' is higher than 'end'".as_ptr(),
            );
            return ().reported(error);
        }
        let mut disallow_nl: bool = channel_id != VIML_INTERNAL_CALL;
        if !check_string_array(
            replacement,
            c"replacement string".as_ptr() as *mut ::core::ffi::c_char,
            disallow_nl,
            err,
        ) {
            return ().reported(error);
        }
        let mut new_len: size_t = replacement.size;
        let mut old_len: size_t = (end - start) as size_t;
        let mut extra: ptrdiff_t = 0 as ptrdiff_t;
        let mut lines: *mut *mut ::core::ffi::c_char = (if new_len != 0 as size_t {
            arena_alloc(
                arena,
                new_len.wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_char>()),
                true,
            )
        } else {
            NULL
        }) as *mut *mut ::core::ffi::c_char;
        let mut i: size_t = 0 as size_t;
        while i < new_len {
            let l: String_0 = (*replacement.items.add(i)).data.string;
            *lines.add(i) = arena_memdupz(arena, l.data(), l.len());
            memchrsub(
                *lines.add(i) as *mut ::core::ffi::c_void,
                NUL as ::core::ffi::c_char,
                NL as ::core::ffi::c_char,
                l.len(),
            );
            i = i.wrapping_add(1);
        }
        let mut tstate: TryState = TryState {
            current_exception: ::core::ptr::null_mut::<except_T>(),
            private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
            msg_list: ::core::ptr::null::<*const msglist_T>(),
            got_int: 0,
            did_throw: false,
            need_rethrow: 0,
            did_emsg: 0,
        };
        try_enter(&raw mut tstate);
        's_382: {
            if (*b).b_p_ma == 0 {
                api_set_error(
                    err,
                    kErrorTypeException,
                    c"Buffer is not 'modifiable'".as_ptr(),
                );
            } else if u_save_buf(b, (start - 1 as Integer) as linenr_T, end as linenr_T)
                == 0 as ::core::ffi::c_int
            {
                api_set_error(
                    err,
                    kErrorTypeException,
                    c"Failed to save undo information".as_ptr(),
                );
            } else {
                let mut deleted_bytes: bcount_t = get_region_bytecount(
                    b,
                    start as linenr_T,
                    end as linenr_T,
                    0 as colnr_T,
                    0 as colnr_T,
                );
                let mut to_delete: size_t = if new_len < old_len {
                    old_len.wrapping_sub(new_len)
                } else {
                    0 as size_t
                };
                let mut i_0: size_t = 0 as size_t;
                while i_0 < to_delete {
                    if ml_delete_buf(b, start as linenr_T, false) == 0 as ::core::ffi::c_int {
                        api_set_error(err, kErrorTypeException, c"Failed to delete line".as_ptr());
                        break 's_382;
                    } else {
                        i_0 = i_0.wrapping_add(1);
                    }
                }
                if to_delete > 0 as size_t {
                    extra -= to_delete as ptrdiff_t;
                }
                let mut to_replace: size_t = if old_len < new_len { old_len } else { new_len };
                let mut inserted_bytes: bcount_t = 0 as bcount_t;
                let mut i_1: size_t = 0 as size_t;
                while i_1 < to_replace {
                    let mut lnum: int64_t = start as int64_t + i_1 as int64_t;
                    if !(lnum < MAXLNUM as ::core::ffi::c_int as int64_t) {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            c"%s".as_ptr(),
                            c"Index out of bounds".as_ptr(),
                        );
                        break 's_382;
                    } else if ml_replace_buf(b, lnum as linenr_T, *lines.add(i_1), false, true)
                        == 0 as ::core::ffi::c_int
                    {
                        api_set_error(err, kErrorTypeException, c"Failed to replace line".as_ptr());
                        break 's_382;
                    } else {
                        inserted_bytes += strlen(*lines.add(i_1)) as bcount_t + 1 as bcount_t;
                        i_1 = i_1.wrapping_add(1);
                    }
                }
                let mut i_2: size_t = to_replace;
                while i_2 < new_len {
                    let mut lnum_0: int64_t = start as int64_t + i_2 as int64_t - 1 as int64_t;
                    if !(lnum_0 < MAXLNUM as ::core::ffi::c_int as int64_t) {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            c"%s".as_ptr(),
                            c"Index out of bounds".as_ptr(),
                        );
                        break 's_382;
                    } else if ml_append_buf(
                        b,
                        lnum_0 as linenr_T,
                        *lines.add(i_2),
                        0 as colnr_T,
                        false,
                    ) == 0 as ::core::ffi::c_int
                    {
                        api_set_error(err, kErrorTypeException, c"Failed to insert line".as_ptr());
                        break 's_382;
                    } else {
                        inserted_bytes += strlen(*lines.add(i_2)) as bcount_t + 1 as bcount_t;
                        extra += 1;
                        i_2 = i_2.wrapping_add(1);
                    }
                }
                let mut adjust: linenr_T = if end > start {
                    MAXLNUM as ::core::ffi::c_int as linenr_T
                } else {
                    0 as linenr_T
                };
                mark_adjust_buf(
                    b,
                    start as linenr_T,
                    (end - 1 as Integer) as linenr_T,
                    adjust,
                    extra as linenr_T,
                    true,
                    kMarkAdjustApi,
                    kExtmarkNOOP,
                );
                if visual_active() as ::core::ffi::c_int != 0
                    && b == curbuf.get()
                    && visual_anchor().lnum >= start as linenr_T
                {
                    if visual_anchor().lnum >= end as linenr_T {
                        with_visual_anchor(|a| a.lnum += extra as linenr_T);
                    }
                    check_visual_pos();
                }
                extmark_splice(
                    b,
                    start as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    0 as colnr_T,
                    (end - start) as ::core::ffi::c_int,
                    0 as colnr_T,
                    deleted_bytes,
                    new_len as ::core::ffi::c_int,
                    0 as colnr_T,
                    inserted_bytes,
                    kExtmarkUndo,
                );
                changed_lines(
                    b,
                    start as linenr_T,
                    0 as colnr_T,
                    end as linenr_T,
                    extra as linenr_T,
                    true,
                );
                let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
                while !tp.is_null() {
                    let mut win: *mut win_T = if tp == curtab.get() {
                        firstwin.get()
                    } else {
                        (*tp).tp_firstwin
                    };
                    while !win.is_null() {
                        if (*win).w_buffer == b {
                            fix_cursor(win, start as linenr_T, end as linenr_T, extra as linenr_T);
                        }
                        win = (*win).w_next;
                    }
                    tp = (*tp).tp_next as *mut tabpage_T;
                }
            }
        }
        try_leave(&raw mut tstate, err);
    }
    ().reported(error)
}

pub unsafe fn nvim_buf_get_text(
    channel_id: uint64_t,
    buf: Buffer,
    mut start_row: Integer,
    start_col: Integer,
    mut end_row: Integer,
    end_col: Integer,
    _opts: *mut KeyDict_empty,
    arena: *mut Arena,
    lstate: *mut lua_State,
) -> Result<Array, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut str: String_0 = String_0::NULL;
        let mut rv: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return rv.reported(error);
        }
        if (*b).b_ml.ml_mfp.is_null() {
            return rv.reported(error);
        }
        let mut oob: bool = false;
        start_row = normalize_index(b, start_row as int64_t, false, &raw mut oob) as Integer;
        end_row = normalize_index(b, end_row as int64_t, false, &raw mut oob) as Integer;
        if oob {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"%s".as_ptr(),
                c"Index out of bounds".as_ptr(),
            );
            return rv.reported(error);
        }
        if !(start_row <= end_row) {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"%s".as_ptr(),
                c"'start' is higher than 'end'".as_ptr(),
            );
            return rv.reported(error);
        }
        let mut replace_nl: bool = channel_id != VIML_INTERNAL_CALL;
        let mut size: size_t = ((end_row - start_row) as size_t).wrapping_add(1 as size_t);
        init_line_array(lstate, &raw mut rv, size, arena);
        if start_row == end_row {
            let mut line: String_0 = buf_get_text(
                b,
                start_row as int64_t,
                start_col as int64_t,
                end_col as int64_t,
                err,
            );
            if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
                push_linestr(
                    lstate,
                    &raw mut rv,
                    line.data(),
                    line.len(),
                    0 as ::core::ffi::c_int,
                    replace_nl,
                    arena,
                );
                return rv.reported(error);
            }
        } else {
            str = buf_get_text(
                b,
                start_row as int64_t,
                start_col as int64_t,
                (MAXCOL as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int64_t,
                err,
            );
            if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
                push_linestr(
                    lstate,
                    &raw mut rv,
                    str.data(),
                    str.len(),
                    0 as ::core::ffi::c_int,
                    replace_nl,
                    arena,
                );
                if size > 2 as size_t {
                    buf_collect_lines(
                        b,
                        size.wrapping_sub(2 as size_t),
                        start_row as linenr_T + 1 as linenr_T,
                        1 as ::core::ffi::c_int,
                        replace_nl,
                        &raw mut rv,
                        lstate,
                        arena,
                    );
                }
                str = buf_get_text(b, end_row as int64_t, 0 as int64_t, end_col as int64_t, err);
                if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
                    push_linestr(
                        lstate,
                        &raw mut rv,
                        str.data(),
                        str.len(),
                        size.wrapping_sub(1 as size_t) as ::core::ffi::c_int,
                        replace_nl,
                        arena,
                    );
                }
            }
        }
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return Err(error);
        }
        rv.reported(error)
    }
}

pub unsafe fn nvim_buf_get_offset(buf: Buffer, index: Integer) -> Result<Integer, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return (0 as Integer).reported(error);
        }
        if (*b).b_ml.ml_mfp.is_null() {
            return (-1 as Integer).reported(error);
        }
        if !(index >= 0 as Integer && index <= (*b).b_ml.ml_line_count as Integer) {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"%s".as_ptr(),
                c"Index out of bounds".as_ptr(),
            );
            return (0 as Integer).reported(error);
        }
        (ml_find_line_or_offset(
            b,
            index as linenr_T + 1 as linenr_T,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            true,
        ) as Integer)
            .reported(error)
    }
}

#[inline]
unsafe fn init_line_array(
    mut lstate: *mut lua_State,
    mut a: *mut Array,
    mut size: size_t,
    mut arena: *mut Arena,
) {
    unsafe {
        if !lstate.is_null() {
            lua_createtable(lstate, size as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        } else {
            *a = arena_array(arena, size);
        };
    }
}

unsafe fn push_linestr(
    mut lstate: *mut lua_State,
    mut a: *mut Array,
    mut s: *const ::core::ffi::c_char,
    mut len: size_t,
    mut idx: ::core::ffi::c_int,
    mut replace_nl: bool,
    mut arena: *mut Arena,
) {
    unsafe {
        if !lstate.is_null() {
            if !s.is_null()
                && replace_nl as ::core::ffi::c_int != 0
                && !strchr(s, '\n' as ::core::ffi::c_int).is_null()
            {
                let mut tmp: *mut ::core::ffi::c_char =
                    xmemdupz(s as *const ::core::ffi::c_void, len) as *mut ::core::ffi::c_char;
                strchrsub(tmp, '\n' as ::core::ffi::c_char, NUL as ::core::ffi::c_char);
                lua_pushlstring(lstate, tmp, len);
                xfree(tmp as *mut ::core::ffi::c_void);
            } else {
                lua_pushlstring(lstate, s, len);
            }
            lua_rawseti(
                lstate,
                -2 as ::core::ffi::c_int,
                idx + 1 as ::core::ffi::c_int,
            );
        } else {
            let mut str: String_0 = String_0::from_raw_parts(
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                0 as size_t,
            );
            if len > 0 as size_t {
                str = arena_string(
                    arena,
                    String_0::from_raw_parts(s as *mut ::core::ffi::c_char, len),
                );
                if replace_nl {
                    strchrsub(
                        str.data(),
                        '\n' as ::core::ffi::c_char,
                        NUL as ::core::ffi::c_char,
                    );
                }
            }
            array_add(&mut (*a), Object::string(str));
        };
    }
}

pub unsafe fn buf_collect_lines(
    mut buf: *mut buf_T,
    mut n: size_t,
    mut start: linenr_T,
    mut start_idx: ::core::ffi::c_int,
    mut replace_nl: bool,
    mut l: *mut Array,
    mut lstate: *mut lua_State,
    mut arena: *mut Arena,
) {
    unsafe {
        let mut i: size_t = 0 as size_t;
        while i < n {
            let mut lnum: linenr_T = start + i as linenr_T;
            let mut bufstr: *mut ::core::ffi::c_char = ml_get_buf(buf, lnum);
            let mut bufstrlen: size_t = ml_get_buf_len(buf, lnum) as size_t;
            push_linestr(
                lstate,
                l,
                bufstr,
                bufstrlen,
                start_idx + i as ::core::ffi::c_int,
                replace_nl,
                arena,
            );
            i = i.wrapping_add(1);
        }
    }
}
