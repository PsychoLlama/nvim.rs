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
use crate::winlayer::{Buf, tab_windows};

pub unsafe fn nvim_buf_line_count(buf: Buffer) -> Result<Integer, Error> {
    let mut error = ERROR_INIT;
    let mut b: *mut buf_T = unsafe { find_buffer_by_handle(buf, &mut error) };
    if b.is_null() {
        return (0 as Integer).reported(error);
    }
    // SAFETY: non-null, so the handle named a live buffer.
    let b = unsafe { Buf::new(b) };
    if b.b_ml.ml_mfp.is_null() {
        return (0 as Integer).reported(error);
    }
    (b.line_count() as Integer).reported(error)
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
    let mut rv: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut b: *mut buf_T = unsafe { find_buffer_by_handle(buf, &mut error) };
    if b.is_null() {
        return rv.reported(error);
    }
    // SAFETY: non-null, so the handle named a live buffer.
    if unsafe { Buf::new(b) }.b_ml.ml_mfp.is_null() {
        return rv.reported(error);
    }
    let mut oob: bool = false;
    start = unsafe { normalize_index(b, start as int64_t, true, &raw mut oob) } as Integer;
    end = unsafe { normalize_index(b, end as int64_t, true, &raw mut oob) } as Integer;
    if !(!strict_indexing || !oob) {
        let why = c"Index out of bounds";
        error = Error::validation(why);
        return rv.reported(error);
    }
    if start >= end {
        return rv.reported(error);
    }
    let mut size: size_t = (end - start) as size_t;
    unsafe { init_line_array(lstate, &raw mut rv, size, arena) };
    let at = start as linenr_T;
    let nl = channel_id != VIML_INTERNAL_CALL;
    let rvp = &raw mut rv;
    // SAFETY: `b` is the live buffer and `rvp` this call's own array.
    unsafe { buf_collect_lines(b, size, at, 0, nl, rvp, lstate, arena) };
    rv.reported(error)
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
    let mut b: *mut buf_T = unsafe { api_buf_ensure_loaded(buf, &mut error) };
    if b.is_null() {
        return ().reported(error);
    }
    let mut oob: bool = false;
    start = unsafe { normalize_index(b, start as int64_t, true, &raw mut oob) } as Integer;
    end = unsafe { normalize_index(b, end as int64_t, true, &raw mut oob) } as Integer;
    if !(!strict_indexing || !oob) {
        let why = c"Index out of bounds";
        error = Error::validation(why);
        return ().reported(error);
    }
    if !(start <= end) {
        let why = c"'start' is higher than 'end'";
        error = Error::validation(why);
        return ().reported(error);
    }
    let mut disallow_nl: bool = channel_id != VIML_INTERNAL_CALL;
    // SAFETY: `replacement` is the caller's array.
    unsafe { check_string_array(replacement, c"replacement string", disallow_nl) }?;
    let mut new_len: size_t = replacement.size;
    let mut old_len: size_t = (end - start) as size_t;
    let mut extra: ptrdiff_t = 0 as ptrdiff_t;
    let bytes = new_len.wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_char>());
    let lines: *mut *mut ::core::ffi::c_char = (if new_len != 0 as size_t {
        // SAFETY: `arena` is the caller's.
        unsafe { arena_alloc(arena, bytes, true) }
    } else {
        NULL
    }) as *mut *mut ::core::ffi::c_char;
    // `memchrsub` turns embedded NULs back into the newlines they stand for.
    let nul = NUL as ::core::ffi::c_char;
    let nl = NL as ::core::ffi::c_char;
    let mut i: size_t = 0 as size_t;
    while i < new_len {
        // SAFETY: `i` is below `replacement.size`.
        let l: String_0 = unsafe { (*replacement.items.add(i)).data.string };
        unsafe { *lines.add(i) = arena_memdupz(arena, l.data(), l.len()) };
        // SAFETY: `i` is below `new_len`, so the slot was just written.
        let line = unsafe { *lines.add(i) } as *mut ::core::ffi::c_void;
        // SAFETY: `line` holds `l.len()` bytes.
        unsafe { memchrsub(line, nul, nl, l.len()) };
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
    unsafe { try_enter(&raw mut tstate) };
    let buf = unsafe { Buf::new(b) };
    's_382: {
        if buf.b_p_ma == 0 {
            let why = c"Buffer is not 'modifiable'";
            error = Error::exception(why);
        } else if u_save_buf(buf, (start - 1 as Integer) as linenr_T, end as linenr_T).is_err() {
            let why = c"Failed to save undo information";
            error = Error::exception(why);
        } else {
            let mut deleted_bytes: bcount_t = get_region_bytecount(
                buf,
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
                if unsafe { ml_delete_buf(b, start as linenr_T, false) }.is_err() {
                    let why = c"Failed to delete line";
                    error = Error::exception(why);
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
                    let why = c"Index out of bounds";
                    error = Error::validation(why);
                    break 's_382;
                } else if {
                    // SAFETY: `i_1` is below `new_len`.
                    let line = unsafe { *lines.add(i_1) };
                    // SAFETY: `b` is the live buffer, `lnum` one of its lines.
                    unsafe { ml_replace_buf(b, lnum as linenr_T, line, false, true) }
                }
                .is_err()
                {
                    let why = c"Failed to replace line";
                    error = Error::exception(why);
                    break 's_382;
                } else {
                    inserted_bytes +=
                        unsafe { strlen(*lines.add(i_1)) } as bcount_t + 1 as bcount_t;
                    i_1 = i_1.wrapping_add(1);
                }
            }
            let mut i_2: size_t = to_replace;
            while i_2 < new_len {
                let mut lnum_0: int64_t = start as int64_t + i_2 as int64_t - 1 as int64_t;
                if !(lnum_0 < MAXLNUM as ::core::ffi::c_int as int64_t) {
                    let why = c"Index out of bounds";
                    error = Error::validation(why);
                    break 's_382;
                } else if {
                    // SAFETY: `i_2` is below `new_len`.
                    let line = unsafe { *lines.add(i_2) };
                    let at = lnum_0 as linenr_T;
                    // SAFETY: `b` is the live buffer.
                    unsafe { ml_append_buf(b, at, line, 0 as colnr_T, false) }
                }
                .is_err()
                {
                    let why = c"Failed to insert line";
                    error = Error::exception(why);
                    break 's_382;
                } else {
                    inserted_bytes +=
                        unsafe { strlen(*lines.add(i_2)) } as bcount_t + 1 as bcount_t;
                    extra += 1;
                    i_2 = i_2.wrapping_add(1);
                }
            }
            let mut adjust: linenr_T = if end > start {
                MAXLNUM as ::core::ffi::c_int as linenr_T
            } else {
                0 as linenr_T
            };
            unsafe {
                mark_adjust_buf(
                    b,
                    start as linenr_T,
                    (end - 1 as Integer) as linenr_T,
                    adjust,
                    extra as linenr_T,
                    true,
                    kMarkAdjustApi,
                    kExtmarkNOOP,
                )
            };
            if visual_active() as ::core::ffi::c_int != 0
                && b == curbuf.get()
                && visual_anchor().lnum >= start as linenr_T
            {
                if visual_anchor().lnum >= end as linenr_T {
                    with_visual_anchor(|a| a.lnum += extra as linenr_T);
                }
                unsafe { check_visual_pos() };
            }
            unsafe {
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
                )
            };
            changed_lines(
                unsafe { Buf::new(b) },
                start as linenr_T,
                0 as colnr_T,
                end as linenr_T,
                extra as linenr_T,
                true,
            );
            for win in tab_windows() {
                if win.w_buffer == b {
                    let (lo, hi) = (start as linenr_T, end as linenr_T);
                    // SAFETY: a live window showing this buffer.
                    unsafe { fix_cursor(win.raw(), lo, hi, extra as linenr_T) };
                }
            }
        }
    }
    unsafe { try_leave(&raw mut tstate, &mut error) };
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
    let mut str: String_0 = String_0::NULL;
    let mut rv: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut b: *mut buf_T = unsafe { find_buffer_by_handle(buf, &mut error) };
    if b.is_null() {
        return rv.reported(error);
    }
    // SAFETY: non-null, so the handle named a live buffer.
    if unsafe { Buf::new(b) }.b_ml.ml_mfp.is_null() {
        return rv.reported(error);
    }
    let mut oob: bool = false;
    start_row = unsafe { normalize_index(b, start_row as int64_t, false, &raw mut oob) } as Integer;
    end_row = unsafe { normalize_index(b, end_row as int64_t, false, &raw mut oob) } as Integer;
    if oob {
        let why = c"Index out of bounds";
        error = Error::validation(why);
        return rv.reported(error);
    }
    if !(start_row <= end_row) {
        let why = c"'start' is higher than 'end'";
        error = Error::validation(why);
        return rv.reported(error);
    }
    let mut replace_nl: bool = channel_id != VIML_INTERNAL_CALL;
    let mut size: size_t = ((end_row - start_row) as size_t).wrapping_add(1 as size_t);
    unsafe { init_line_array(lstate, &raw mut rv, size, arena) };
    let rvp = &raw mut rv;
    let first = start_row as int64_t;
    if start_row == end_row {
        let (from, to) = (start_col as int64_t, end_col as int64_t);
        // SAFETY: `b` is the live buffer and `error` this call's error slot.
        let line: String_0 = unsafe { buf_get_text(b, first, from, to, &mut error) };
        if !error.is_set() {
            let (data, len) = (line.data(), line.len());
            // SAFETY: `data` holds `len` bytes; `rvp` is this call's array.
            unsafe { push_linestr(lstate, rvp, data, len, 0, replace_nl, arena) };
            return rv.reported(error);
        }
    } else {
        let from = start_col as int64_t;
        let to = (MAXCOL as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int64_t;
        // SAFETY: `b` is the live buffer and `error` this call's error slot.
        str = unsafe { buf_get_text(b, first, from, to, &mut error) };
        if !error.is_set() {
            let (data, len) = (str.data(), str.len());
            // SAFETY: `data` holds `len` bytes; `rvp` is this call's array.
            unsafe { push_linestr(lstate, rvp, data, len, 0, replace_nl, arena) };
            if size > 2 as size_t {
                let n = size.wrapping_sub(2 as size_t);
                let at = start_row as linenr_T + 1 as linenr_T;
                // SAFETY: `b` is the live buffer and `rvp` this call's array.
                unsafe { buf_collect_lines(b, n, at, 1, replace_nl, rvp, lstate, arena) };
            }
            let last = end_row as int64_t;
            let to = end_col as int64_t;
            // SAFETY: `b` is the live buffer and `error` this call's error slot.
            str = unsafe { buf_get_text(b, last, 0 as int64_t, to, &mut error) };
            if !error.is_set() {
                let (data, len) = (str.data(), str.len());
                let at = size.wrapping_sub(1 as size_t) as ::core::ffi::c_int;
                // SAFETY: `data` holds `len` bytes; `rvp` is this call's array.
                unsafe { push_linestr(lstate, rvp, data, len, at, replace_nl, arena) };
            }
        }
    }
    if error.is_set() {
        return Err(error);
    }
    rv.reported(error)
}

pub unsafe fn nvim_buf_get_offset(buf: Buffer, index: Integer) -> Result<Integer, Error> {
    let mut error = ERROR_INIT;
    let mut b: *mut buf_T = unsafe { find_buffer_by_handle(buf, &mut error) };
    if b.is_null() {
        return (0 as Integer).reported(error);
    }
    // SAFETY: non-null, so the handle named a live buffer.
    let b = unsafe { Buf::new(b) };
    if b.b_ml.ml_mfp.is_null() {
        return (-1 as Integer).reported(error);
    }
    if !(index >= 0 as Integer && index <= b.line_count() as Integer) {
        let why = c"Index out of bounds";
        error = Error::validation(why);
        return (0 as Integer).reported(error);
    }
    let lnum = index as linenr_T + 1 as linenr_T;
    let no_lnum = ::core::ptr::null_mut::<::core::ffi::c_int>();
    // SAFETY: `b` is the live buffer and `lnum` one past its last line at
    // most, which is what this asks for.
    let offset = unsafe { ml_find_line_or_offset(b.raw(), lnum, no_lnum, true) };
    (offset as Integer).reported(error)
}

#[inline]
unsafe fn init_line_array(
    mut lstate: *mut lua_State,
    mut a: *mut Array,
    mut size: size_t,
    mut arena: *mut Arena,
) {
    if !lstate.is_null() {
        unsafe { lua_createtable(lstate, size as ::core::ffi::c_int, 0 as ::core::ffi::c_int) };
    } else {
        unsafe { *a = arena_array(arena, size) };
    };
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
    if !lstate.is_null() {
        if !s.is_null()
            && replace_nl as ::core::ffi::c_int != 0
            && !unsafe { strchr(s, '\n' as ::core::ffi::c_int) }.is_null()
        {
            let mut tmp: *mut ::core::ffi::c_char =
                unsafe { xmemdupz(s as *const ::core::ffi::c_void, len) }
                    as *mut ::core::ffi::c_char;
            unsafe { strchrsub(tmp, '\n' as ::core::ffi::c_char, NUL as ::core::ffi::c_char) };
            unsafe { lua_pushlstring(lstate, tmp, len) };
            unsafe { xfree(tmp as *mut ::core::ffi::c_void) };
        } else {
            unsafe { lua_pushlstring(lstate, s, len) };
        }
        let at = idx + 1 as ::core::ffi::c_int;
        // SAFETY: the caller's Lua state, with the table on top.
        unsafe { lua_rawseti(lstate, -2 as ::core::ffi::c_int, at) };
    } else {
        let mut str: String_0 =
            String_0::from_raw_parts(::core::ptr::null_mut::<::core::ffi::c_char>(), 0 as size_t);
        if len > 0 as size_t {
            let borrowed = String_0::from_raw_parts(s as *mut ::core::ffi::c_char, len);
            // SAFETY: the caller's promise about `s` and `len`, and `arena`.
            str = unsafe { arena_string(arena, borrowed) };
            if replace_nl {
                let (nl, nul) = ('\n' as ::core::ffi::c_char, NUL as ::core::ffi::c_char);
                // SAFETY: `str` is the copy the arena just made.
                unsafe { strchrsub(str.data(), nl, nul) };
            }
        }
        unsafe { array_add(&mut (*a), Object::string(str)) };
    };
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
    let mut i: size_t = 0 as size_t;
    while i < n {
        let mut lnum: linenr_T = start + i as linenr_T;
        let mut bufstr: *mut ::core::ffi::c_char = unsafe { ml_get_buf(buf, lnum) };
        let len: size_t = unsafe { ml_get_buf_len(buf, lnum) } as size_t;
        let at = start_idx + i as ::core::ffi::c_int;
        // SAFETY: `bufstr` holds `len` bytes, and `l`/`lstate` are the
        // caller's.
        unsafe { push_linestr(lstate, l, bufstr, len, at, replace_nl, arena) };
        i = i.wrapping_add(1);
    }
}
