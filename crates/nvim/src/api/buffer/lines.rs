//! Whole-line reads and replacements.
//!
//! `nvim_buf_get_lines`/`nvim_buf_set_lines` are the line-granular half of
//! the buffer API, and `nvim_buf_get_text` the read that takes a byte range
//! but still hands back whole lines.  `push_linestr` is the shared
//! line-to-`String_0` step every one of them ends in, and
//! `buf_collect_lines` the loop over a range that `buffer_updates.rs`
//! reaches too, to build the `nvim_buf_attach` change notifications.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::api::private::helpers::Reported;
use crate::cstr;
use crate::normal::{visual_active, visual_anchor, with_visual_anchor};
use crate::types::NUL;
use crate::winlayer::{Buf, tab_windows};
use ::libc::memchr;

pub fn nvim_buf_line_count(buf: BufferHandle) -> Result<Integer, Error> {
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(0 as Integer);
    };
    if b.b_ml.ml_mfp.is_null() {
        return Ok(0 as Integer);
    }
    Ok(b.line_count() as Integer)
}

/// # Safety
///
/// `arena` must point at a live arena, which the memory this answers with is
/// taken from and must outlive. `lstate` must point at the Lua state this
/// call runs on.
pub unsafe fn nvim_buf_get_lines(
    channel_id: uint64_t,
    buf: BufferHandle,
    mut start: Integer,
    mut end: Integer,
    strict_indexing: Boolean,
    lstate: *mut lua_State,
) -> Result<Array, Error> {
    let mut error = Error::none();
    let mut rv: Array = Array::EMPTY;
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(rv);
    };
    // SAFETY: non-null, so the handle named a live buffer.
    if b.b_ml.ml_mfp.is_null() {
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
    let size: size_t = (end - start) as size_t;
    unsafe { init_line_array(lstate, &raw mut rv, size) };
    let at = start as LineNr;
    let nl = channel_id != VIML_INTERNAL_CALL;
    let rvp = &raw mut rv;
    // SAFETY: `b` is the live buffer and `rvp` this call's own array.
    unsafe { buf_collect_lines(b, size, at, 0, nl, rvp, lstate) };
    rv.reported(error)
}

/// # Safety
///
/// `replacement` must be a well-formed API array, its `size` elements
/// initialized. `arena` must point at a live arena, which the memory this
/// answers with is taken from and must outlive.
pub unsafe fn nvim_buf_set_lines(
    channel_id: uint64_t,
    buf: BufferHandle,
    mut start: Integer,
    mut end: Integer,
    strict_indexing: Boolean,
    replacement: Array,
    arena: *mut Arena,
) -> Result<(), Error> {
    let mut error = Error::none();
    let Some(buffer) = api_buf_ensure_loaded(buf)? else {
        return Ok(());
    };
    let mut oob: bool = false;
    start = unsafe { normalize_index(buffer, start as int64_t, true, &raw mut oob) } as Integer;
    end = unsafe { normalize_index(buffer, end as int64_t, true, &raw mut oob) } as Integer;
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
    let disallow_nl: bool = channel_id != VIML_INTERNAL_CALL;
    // SAFETY: `replacement` is the caller's array.
    check_string_array(&replacement, c"replacement string", disallow_nl)?;
    let new_len: size_t = replacement.len();
    let old_len: size_t = (end - start) as size_t;
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
        // Every item is a String: `check_string_array` above turned anything
        // else into an error.
        // SAFETY: `i` is below `replacement.size`.
        let l: String_0 = replacement[i]
            .as_string()
            .expect("check_string_array accepted only Strings")
            .clone();
        unsafe { *lines.add(i) = arena_memdupz(arena, l.data(), l.len()) };
        // SAFETY: `i` is below `new_len`, so the slot was just written.
        let line = unsafe { *lines.add(i) } as *mut ::core::ffi::c_void;
        // SAFETY: `line` holds `l.len()` bytes.
        unsafe { memchrsub(line, nul, nl, l.len()) };
        i = i.wrapping_add(1);
    }
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<Exception>(),
        private_msg_list: ::core::ptr::null_mut::<MsgList>(),
        msg_list: ::core::ptr::null::<*const MsgList>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    unsafe { try_enter(&raw mut tstate) };
    let buf = buffer;
    's_382: {
        if buf.b_p_ma == 0 {
            let why = c"Buffer is not 'modifiable'";
            error = Error::exception(why);
        } else if u_save_buf(buf, (start - 1 as Integer) as LineNr, end as LineNr).is_err() {
            let why = c"Failed to save undo information";
            error = Error::exception(why);
        } else {
            let deleted_bytes: BCount =
                get_region_bytecount(buf, start as LineNr, end as LineNr, 0 as ColNr, 0 as ColNr);
            let to_delete: size_t = if new_len < old_len {
                old_len.wrapping_sub(new_len)
            } else {
                0 as size_t
            };
            let mut i_0: size_t = 0 as size_t;
            while i_0 < to_delete {
                if ml_delete_buf(buffer, start as LineNr, false).is_err() {
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
            let to_replace: size_t = if old_len < new_len { old_len } else { new_len };
            let mut inserted_bytes: BCount = 0 as BCount;
            let mut i_1: size_t = 0 as size_t;
            while i_1 < to_replace {
                let lnum: int64_t = start as int64_t + i_1 as int64_t;
                if !(lnum < MAXLNUM as int64_t) {
                    let why = c"Index out of bounds";
                    error = Error::validation(why);
                    break 's_382;
                } else if {
                    // SAFETY: `i_1` is below `new_len`.
                    let line = unsafe { *lines.add(i_1) };
                    // SAFETY: `b` is the live buffer, `lnum` one of its lines.
                    unsafe { ml_replace_buf(buffer, lnum as LineNr, line, false, true) }
                }
                .is_err()
                {
                    let why = c"Failed to replace line";
                    error = Error::exception(why);
                    break 's_382;
                } else {
                    inserted_bytes +=
                        unsafe { cstr::bytes_at(*lines.add(i_1)) }.len() as BCount + 1 as BCount;
                    i_1 = i_1.wrapping_add(1);
                }
            }
            let mut i_2: size_t = to_replace;
            while i_2 < new_len {
                let lnum_0: int64_t = start as int64_t + i_2 as int64_t - 1 as int64_t;
                if !(lnum_0 < MAXLNUM as int64_t) {
                    let why = c"Index out of bounds";
                    error = Error::validation(why);
                    break 's_382;
                } else if {
                    // SAFETY: `i_2` is below `new_len`.
                    let line = unsafe { *lines.add(i_2) };
                    let at = lnum_0 as LineNr;
                    // SAFETY: `b` is the live buffer.
                    unsafe { ml_append_buf(buffer, at, line, 0 as ColNr, false) }
                }
                .is_err()
                {
                    let why = c"Failed to insert line";
                    error = Error::exception(why);
                    break 's_382;
                } else {
                    inserted_bytes +=
                        unsafe { cstr::bytes_at(*lines.add(i_2)) }.len() as BCount + 1 as BCount;
                    extra += 1;
                    i_2 = i_2.wrapping_add(1);
                }
            }
            let adjust: LineNr = if end > start {
                MAXLNUM as LineNr
            } else {
                0 as LineNr
            };
            mark_adjust_buf(
                buffer,
                start as LineNr,
                (end - 1 as Integer) as LineNr,
                adjust,
                extra as LineNr,
                true,
                kMarkAdjustApi,
                kExtmarkNOOP,
            );
            if visual_active() as ::core::ffi::c_int != 0
                && Some(buffer) == Buf::current_or_none()
                && visual_anchor().lnum >= start as LineNr
            {
                if visual_anchor().lnum >= end as LineNr {
                    with_visual_anchor(|a| a.lnum += extra as LineNr);
                }
                check_visual_pos();
            }
            extmark_splice(
                buffer,
                start as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                0 as ColNr,
                (end - start) as ::core::ffi::c_int,
                0 as ColNr,
                deleted_bytes,
                new_len as ::core::ffi::c_int,
                0 as ColNr,
                inserted_bytes,
                kExtmarkUndo,
            );
            changed_lines(
                buffer,
                start as LineNr,
                0 as ColNr,
                end as LineNr,
                extra as LineNr,
                true,
            );
            for win in tab_windows() {
                if win.w_buffer == buffer.raw() {
                    let (lo, hi) = (start as LineNr, end as LineNr);
                    fix_cursor(win, lo, hi, extra as LineNr);
                }
            }
        }
    }
    // The bracket outranks whatever the body wrote into `error`, which is
    // the order the two had when both went through one slot.
    unsafe { try_leave(&raw mut tstate) }?;
    ().reported(error)
}

/// # Safety
///
/// `_opts` must point at the `KeyDict_empty` the dispatcher filled in, live
/// for the call. `arena` must point at a live arena, which the memory this
/// answers with is taken from and must outlive. `lstate` must point at the
/// Lua state this call runs on.
pub unsafe fn nvim_buf_get_text(
    channel_id: uint64_t,
    buf: BufferHandle,
    mut start_row: Integer,
    start_col: Integer,
    mut end_row: Integer,
    end_col: Integer,
    _opts: *mut KeyDict_empty,
    lstate: *mut lua_State,
) -> Result<Array, Error> {
    let mut error = Error::none();
    let mut str: String_0;
    let mut rv: Array = Array::EMPTY;
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(rv);
    };
    // SAFETY: non-null, so the handle named a live buffer.
    if b.b_ml.ml_mfp.is_null() {
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
    let replace_nl: bool = channel_id != VIML_INTERNAL_CALL;
    let size: size_t = ((end_row - start_row) as size_t).wrapping_add(1 as size_t);
    unsafe { init_line_array(lstate, &raw mut rv, size) };
    let rvp = &raw mut rv;
    let first = start_row as int64_t;
    if start_row == end_row {
        let (from, to) = (start_col as int64_t, end_col as int64_t);
        let line: String_0 = buf_get_text(b, first, from, to)?;
        let (data, len) = (line.data(), line.len());
        // SAFETY: `data` holds `len` bytes; `rvp` is this call's array.
        unsafe { push_linestr(lstate, rvp, data, len, 0, replace_nl) };
        return Ok(rv);
    }
    let from = start_col as int64_t;
    let to = (MAXCOL as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int64_t;
    str = buf_get_text(b, first, from, to)?;
    let (data, len) = (str.data(), str.len());
    // SAFETY: `data` holds `len` bytes; `rvp` is this call's array.
    unsafe { push_linestr(lstate, rvp, data, len, 0, replace_nl) };
    if size > 2 as size_t {
        let n = size.wrapping_sub(2 as size_t);
        let at = start_row as LineNr + 1 as LineNr;
        // SAFETY: `b` is the live buffer and `rvp` this call's array.
        unsafe { buf_collect_lines(b, n, at, 1, replace_nl, rvp, lstate) };
    }
    let last = end_row as int64_t;
    let to = end_col as int64_t;
    str = buf_get_text(b, last, 0 as int64_t, to)?;
    let (data, len) = (str.data(), str.len());
    let at = size.wrapping_sub(1 as size_t) as ::core::ffi::c_int;
    // SAFETY: `data` holds `len` bytes; `rvp` is this call's array.
    unsafe { push_linestr(lstate, rvp, data, len, at, replace_nl) };
    Ok(rv)
}

pub fn nvim_buf_get_offset(buf: BufferHandle, index: Integer) -> Result<Integer, Error> {
    let mut error = Error::none();
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(0 as Integer);
    };
    if b.b_ml.ml_mfp.is_null() {
        return (-1 as Integer).reported(error);
    }
    if !(index >= 0 as Integer && index <= b.line_count() as Integer) {
        let why = c"Index out of bounds";
        error = Error::validation(why);
        return (0 as Integer).reported(error);
    }
    let lnum = index as LineNr + 1 as LineNr;
    let no_lnum = ::core::ptr::null_mut::<::core::ffi::c_int>();
    // SAFETY: `b` is the live buffer and `lnum` one past its last line at
    // most, which is what this asks for.
    let offset = unsafe { ml_find_line_or_offset(b, lnum, no_lnum, true) };
    (offset as Integer).reported(error)
}

/// # Safety
///
/// `lstate` must point at the Lua state this call runs on. `a` must point at
/// an API array the caller owns, unaliased for the call. `arena` must point
/// at a live arena, which the memory this answers with is taken from and must
/// outlive.
#[inline]
unsafe fn init_line_array(lstate: *mut lua_State, a: *mut Array, size: size_t) {
    if !lstate.is_null() {
        unsafe { lua_createtable(lstate, size as ::core::ffi::c_int, 0 as ::core::ffi::c_int) };
    } else {
        unsafe { *a = Array::with_capacity(size) };
    };
}

/// # Safety
///
/// `lstate` must point at the Lua state this call runs on. `a` must point at
/// an API array the caller owns, unaliased for the call. `s` must point at
/// `len` readable bytes. `arena` must point at a live arena, which the memory
/// this answers with is taken from and must outlive.
unsafe fn push_linestr(
    lstate: *mut lua_State,
    a: *mut Array,
    s: *const ::core::ffi::c_char,
    len: size_t,
    idx: ::core::ffi::c_int,
    replace_nl: bool,
) {
    if !lstate.is_null() {
        // The question is only whether these `len` bytes hold a NL, and the
        // caller already has the length: `memchr` answers it in one early-
        // exiting pass, where reading `s` back as a C string pays a `strlen`
        // over the whole line plus a scalar scan -- 860,000 lines a run on
        // the memline bench, and 0.6 % of it.
        //
        // SAFETY: the caller's promise that `s` points at `len` readable
        // bytes; a null `s` is ruled out first.
        let holds_nl = !s.is_null()
            && replace_nl
            && !unsafe { memchr(s.cast::<::core::ffi::c_void>(), NL, len) }.is_null();
        if holds_nl {
            let tmp: *mut ::core::ffi::c_char =
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
        let mut str: String_0 = String_0::NULL;
        if len > 0 as size_t {
            // SAFETY: the caller's promise about `s` and `len`.
            str = unsafe { String_0::from_raw_bytes(s, len) };
            if replace_nl {
                let (nl, nul) = ('\n' as ::core::ffi::c_char, NUL as ::core::ffi::c_char);
                // SAFETY: `str` names its own bytes.
                unsafe { strchrsub(str.data(), nl, nul) };
            }
        }
        // SAFETY: `a` is the caller's array.
        unsafe { (*a).push(Object::string(str)) };
    };
}

/// # Safety
///
/// `l` must point at an API array the caller owns, unaliased for the call.
/// `lstate` must point at the Lua state this call runs on. `arena` must point
/// at a live arena, which the memory this answers with is taken from and must
/// outlive.
pub unsafe fn buf_collect_lines(
    buffer: Buf,
    n: size_t,
    start: LineNr,
    start_idx: ::core::ffi::c_int,
    replace_nl: bool,
    l: *mut Array,
    lstate: *mut lua_State,
) {
    let mut i: size_t = 0 as size_t;
    while i < n {
        let lnum: LineNr = start + i as LineNr;
        let bufstr: *mut ::core::ffi::c_char = unsafe { ml_get_buf(buffer, lnum) };
        let len: size_t = ml_get_buf_len(buffer, lnum) as size_t;
        let at = start_idx + i as ::core::ffi::c_int;
        // SAFETY: `bufstr` holds `len` bytes, and `l`/`lstate` are the
        // caller's.
        unsafe { push_linestr(lstate, l, bufstr, len, at, replace_nl) };
        i = i.wrapping_add(1);
    }
}
