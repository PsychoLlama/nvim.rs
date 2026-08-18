use crate::main::namedfm;
use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr;

use super::lookup::*;
use super::*;

/// Iterate over global marks
///
/// @warning No mark-editing functions must be called while iteration is in
///          progress.
///
/// `iter` — Iterator. Pass NULL to start iteration.
/// `name` — Mark name.
/// `fm` — Mark definition.
///
/// Returns pointer that needs to be passed to next `mark_global_iter` call or
///         NULL if iteration is over.
pub unsafe fn mark_global_iter(
    iter: *const c_void,
    name: *mut c_char,
    fm: *mut xfmark_T,
) -> *const c_void {
    *name = NUL as c_char;
    let mut iter_mark: *const xfmark_T = if iter.is_null() {
        (namedfm.ptr() as *mut xfmark_T).offset(0) as *const xfmark_T
    } else {
        iter as *const xfmark_T
    };
    while (iter_mark.offset_from((namedfm.ptr() as *mut xfmark_T).offset(0)) as size_t)
        < size_of::<[xfmark_T; 36]>()
            .wrapping_div(size_of::<xfmark_T>())
            .wrapping_div(
                (size_of::<[xfmark_T; 36]>().wrapping_rem(size_of::<xfmark_T>()) == 0) as c_int
                    as usize,
            )
        && (*iter_mark).fmark.mark.lnum == 0
    {
        iter_mark = iter_mark.offset(1);
    }
    if iter_mark.offset_from((namedfm.ptr() as *mut xfmark_T).offset(0)) as size_t
        == size_of::<[xfmark_T; 36]>()
            .wrapping_div(size_of::<xfmark_T>())
            .wrapping_div(
                (size_of::<[xfmark_T; 36]>().wrapping_rem(size_of::<xfmark_T>()) == 0) as c_int
                    as usize,
            )
        || (*iter_mark).fmark.mark.lnum == 0
    {
        return ptr::null();
    }
    let mut iter_off: size_t =
        iter_mark.offset_from((namedfm.ptr() as *mut xfmark_T).offset(0)) as size_t;
    *name = (if iter_off < NMARKS as size_t {
        'A' as c_int + iter_off as c_char as c_int
    } else {
        '0' as c_int + iter_off.wrapping_sub(NMARKS as size_t) as c_char as c_int
    }) as c_char;
    *fm = *iter_mark;
    loop {
        iter_mark = iter_mark.offset(1);
        if (iter_mark.offset_from((namedfm.ptr() as *mut xfmark_T).offset(0)) as size_t)
            >= size_of::<[xfmark_T; 36]>()
                .wrapping_div(size_of::<xfmark_T>())
                .wrapping_div(
                    (size_of::<[xfmark_T; 36]>().wrapping_rem(size_of::<xfmark_T>()) == 0) as c_int
                        as usize,
                )
        {
            break;
        }
        if (*iter_mark).fmark.mark.lnum != 0 {
            return iter_mark as *const c_void;
        }
    }
    return ptr::null();
}

#[inline]
pub(super) unsafe fn next_buffer_mark(buf: *const buf_T, mark_name: *mut c_char) -> *const fmark_T {
    match *mark_name as c_int {
        NUL => {
            *mark_name = '"' as c_char;
            return &raw const (*buf).b_last_cursor;
        }
        34 => {
            *mark_name = '^' as c_char;
            return &raw const (*buf).b_last_insert;
        }
        94 => {
            *mark_name = '.' as c_char;
            return &raw const (*buf).b_last_change;
        }
        46 => {
            *mark_name = 'a' as c_char;
            return (&raw const (*buf).b_namedm as *const fmark_T).offset(0);
        }
        122 => return ptr::null(),
        _ => {
            *mark_name += 1;
            return (&raw const (*buf).b_namedm as *const fmark_T)
                .offset((*mark_name as c_int - 'a' as c_int) as isize);
        }
    };
}

/// Iterate over buffer marks
///
/// @warning No mark-editing functions must be called while iteration is in
///          progress.
///
/// `iter` — Iterator. Pass NULL to start iteration.
/// `buf` — Buffer.
/// `name` — Mark name.
/// `fm` — Mark definition.
///
/// Returns pointer that needs to be passed to next `mark_buffer_iter` call or
///         NULL if iteration is over.
pub unsafe fn mark_buffer_iter(
    iter: *const c_void,
    buf: *const buf_T,
    name: *mut c_char,
    fm: *mut fmark_T,
) -> *const c_void {
    *name = NUL as c_char;
    let mut mark_name: c_char = (if iter.is_null() {
        NUL as isize
    } else if iter == &raw const (*buf).b_last_cursor as *const c_void {
        '"' as isize
    } else if iter == &raw const (*buf).b_last_insert as *const c_void {
        '^' as isize
    } else if iter == &raw const (*buf).b_last_change as *const c_void {
        '.' as isize
    } else {
        (iter as *const fmark_T)
            .offset('a' as c_int as isize)
            .offset_from((&raw const (*buf).b_namedm as *const fmark_T).offset(0))
    }) as c_char;
    let mut iter_mark: *const fmark_T = next_buffer_mark(buf, &raw mut mark_name);
    while !iter_mark.is_null() && (*iter_mark).mark.lnum == 0 {
        iter_mark = next_buffer_mark(buf, &raw mut mark_name);
    }
    if iter_mark.is_null() {
        return ptr::null();
    }
    let mut iter_off: size_t =
        iter_mark.offset_from((&raw const (*buf).b_namedm as *const fmark_T).offset(0)) as size_t;
    if mark_name != 0 {
        *name = mark_name;
    } else {
        *name = ('a' as c_int + iter_off as c_char as c_int) as c_char;
    }
    *fm = *iter_mark;
    return iter_mark as *const c_void;
}

/// Set global mark
///
/// `name` — Mark name.
/// `fm` — Mark to be set.
/// `update` — If true then only set global mark if it was created
///                     later then existing one.
///
/// Returns true on success, false on failure.
pub unsafe fn mark_set_global(name: c_char, fm: xfmark_T, update: bool) -> bool {
    let idx: c_int = mark_global_index(name);
    if idx == -1 {
        return false;
    }
    let fm_tgt: *mut xfmark_T = (namedfm.ptr() as *mut xfmark_T).offset(idx as isize);
    if update && fm.fmark.timestamp <= (*fm_tgt).fmark.timestamp {
        return false;
    }
    if (*fm_tgt).fmark.mark.lnum != 0 {
        free_xfmark(*fm_tgt);
    }
    *fm_tgt = fm;
    return true;
}

/// Set local mark
///
/// `name` — Mark name.
/// `buf` — Pointer to the buffer to set mark in.
/// `fm` — Mark to be set.
/// `update` — If true then only set global mark if it was created
///                     later then existing one.
///
/// Returns true on success, false on failure.
pub unsafe fn mark_set_local(name: c_char, buf: *mut buf_T, fm: fmark_T, update: bool) -> bool {
    let mut fm_tgt: *mut fmark_T = ptr::null_mut();
    if name as c_uint >= 'a' as c_uint && name as c_uint <= 'z' as c_uint {
        fm_tgt = (&raw mut (*buf).b_namedm as *mut fmark_T)
            .offset((name as c_int - 'a' as c_int) as isize);
    } else if name as c_int == '"' as c_int {
        fm_tgt = &raw mut (*buf).b_last_cursor;
    } else if name as c_int == '^' as c_int {
        fm_tgt = &raw mut (*buf).b_last_insert;
    } else if name as c_int == ':' as c_int {
        fm_tgt = &raw mut (*buf).b_prompt_start;
    } else if name as c_int == '.' as c_int {
        fm_tgt = &raw mut (*buf).b_last_change;
    } else {
        return false;
    }
    if update && fm.timestamp <= (*fm_tgt).timestamp {
        return false;
    }
    if (*fm_tgt).mark.lnum != 0 {
        free_fmark(*fm_tgt);
    }
    *fm_tgt = fm;
    return true;
}
