//! Saving and restoring where the engine is: the input position, the
//! submatch registers and the backtracking stacks.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub(crate) unsafe extern "C" fn reg_save(mut save: *mut regsave_T, mut gap: *mut garray_T) {
    if (*rex.ptr()).reg_match.is_null() {
        (*save).rs_u.pos.col = (*rex.ptr()).input.offset_from((*rex.ptr()).line) as colnr_T;
        (*save).rs_u.pos.lnum = (*rex.ptr()).lnum;
    } else {
        (*save).rs_u.ptr = (*rex.ptr()).input;
    }
    (*save).rs_len = (*gap).ga_len;
}
pub(crate) unsafe extern "C" fn reg_restore(mut save: *mut regsave_T, mut gap: *mut garray_T) {
    if (*rex.ptr()).reg_match.is_null() {
        if (*rex.ptr()).lnum != (*save).rs_u.pos.lnum {
            (*rex.ptr()).lnum = (*save).rs_u.pos.lnum;
            (*rex.ptr()).line = reg_getline((*rex.ptr()).lnum) as *mut uint8_t;
        }
        (*rex.ptr()).input = (*rex.ptr()).line.offset((*save).rs_u.pos.col as isize);
    } else {
        (*rex.ptr()).input = (*save).rs_u.ptr;
    }
    (*gap).ga_len = (*save).rs_len;
}
pub(crate) unsafe extern "C" fn reg_save_equal(mut save: *const regsave_T) -> bool {
    if (*rex.ptr()).reg_match.is_null() {
        return (*rex.ptr()).lnum == (*save).rs_u.pos.lnum
            && (*rex.ptr()).input == (*rex.ptr()).line.offset((*save).rs_u.pos.col as isize);
    }
    return (*rex.ptr()).input == (*save).rs_u.ptr;
}
pub(crate) unsafe extern "C" fn save_se_multi(mut savep: *mut save_se_T, mut posp: *mut lpos_T) {
    (*savep).se_u.pos = *posp;
    (*posp).lnum = (*rex.ptr()).lnum;
    (*posp).col = (*rex.ptr()).input.offset_from((*rex.ptr()).line) as colnr_T;
}
pub(crate) unsafe extern "C" fn save_se_one(mut savep: *mut save_se_T, mut pp: *mut *mut uint8_t) {
    (*savep).se_u.ptr = *pp;
    *pp = (*rex.ptr()).input;
}
pub(crate) unsafe extern "C" fn regstack_push(
    mut state: regstate_T,
    mut scan: *mut uint8_t,
) -> *mut regitem_T {
    let mut rp: *mut regitem_T = ::core::ptr::null_mut::<regitem_T>();
    if ((*regstack.ptr()).ga_len as ::core::ffi::c_uint >> 10 as ::core::ffi::c_int) as int64_t
        >= p_mmp.get()
    {
        emsg(gettext(
            E_PATTERN_USES_MORE_MEMORY_THAN_MAXMEMPATTERN.as_ptr(),
        ));
        return ::core::ptr::null_mut::<regitem_T>();
    }
    ga_grow(
        regstack.ptr(),
        ::core::mem::size_of::<regitem_T>() as ::core::ffi::c_int,
    );
    rp = ((*regstack.ptr()).ga_data as *mut ::core::ffi::c_char)
        .offset((*regstack.ptr()).ga_len as isize) as *mut regitem_T;
    (*rp).rs_state = state;
    (*rp).rs_scan = scan;
    (*regstack.ptr()).ga_len += ::core::mem::size_of::<regitem_T>() as ::core::ffi::c_int;
    return rp;
}
pub(crate) unsafe extern "C" fn regstack_pop(mut scan: *mut *mut uint8_t) {
    let mut rp: *mut regitem_T = ::core::ptr::null_mut::<regitem_T>();
    rp = (((*regstack.ptr()).ga_data as *mut ::core::ffi::c_char)
        .offset((*regstack.ptr()).ga_len as isize) as *mut regitem_T)
        .offset(-(1 as ::core::ffi::c_int as isize));
    *scan = (*rp).rs_scan;
    (*regstack.ptr()).ga_len -= ::core::mem::size_of::<regitem_T>() as ::core::ffi::c_int;
}
pub(crate) unsafe extern "C" fn save_subexpr(mut bp: *mut regbehind_T) {
    (*bp).save_need_clear_subexpr = (*rex.ptr()).need_clear_subexpr;
    if (*rex.ptr()).need_clear_subexpr != 0 {
        return;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < NSUBEXP as ::core::ffi::c_int {
        if (*rex.ptr()).reg_match.is_null() {
            (*bp).save_start[i as usize].se_u.pos = *(*rex.ptr()).reg_startpos.offset(i as isize);
            (*bp).save_end[i as usize].se_u.pos = *(*rex.ptr()).reg_endpos.offset(i as isize);
        } else {
            (*bp).save_start[i as usize].se_u.ptr = *(*rex.ptr()).reg_startp.offset(i as isize);
            (*bp).save_end[i as usize].se_u.ptr = *(*rex.ptr()).reg_endp.offset(i as isize);
        }
        i += 1;
    }
}
pub(crate) unsafe extern "C" fn restore_subexpr(mut bp: *mut regbehind_T) {
    (*rex.ptr()).need_clear_subexpr = (*bp).save_need_clear_subexpr;
    if (*rex.ptr()).need_clear_subexpr != 0 {
        return;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < NSUBEXP as ::core::ffi::c_int {
        if (*rex.ptr()).reg_match.is_null() {
            *(*rex.ptr()).reg_startpos.offset(i as isize) = (*bp).save_start[i as usize].se_u.pos;
            *(*rex.ptr()).reg_endpos.offset(i as isize) = (*bp).save_end[i as usize].se_u.pos;
        } else {
            *(*rex.ptr()).reg_startp.offset(i as isize) = (*bp).save_start[i as usize].se_u.ptr;
            *(*rex.ptr()).reg_endp.offset(i as isize) = (*bp).save_end[i as usize].se_u.ptr;
        }
        i += 1;
    }
}
