//! Core argument list management functions
//!
//! Phase 2: check_arglist_locked, alist_clear, alist_init, alist_unlink,
//! alist_new, alist_add, alist_set, alist_expand

use std::ffi::{c_char, c_int, c_void};

use crate::ffi;
use crate::{BLN_CURBUF, BLN_LISTED, FAIL, OK};

// =============================================================================
// check_arglist_locked
// =============================================================================

pub(crate) fn check_arglist_locked() -> c_int {
    if unsafe { ffi::nvim_al_get_arglist_locked() } != 0 {
        unsafe { ffi::nvim_al_emsg_arglist_locked() };
        return FAIL;
    }
    OK
}

#[no_mangle]
pub extern "C" fn rs_check_arglist_locked() -> c_int {
    check_arglist_locked()
}

// =============================================================================
// alist_clear
// =============================================================================

#[export_name = "alist_clear"]
pub extern "C" fn rs_alist_clear(al: ffi::AlistPtr) {
    if check_arglist_locked() == FAIL {
        return;
    }
    unsafe { ffi::nvim_al_deep_clear_aentry(al) };
}

// =============================================================================
// alist_init
// =============================================================================

#[export_name = "alist_init"]
pub extern "C" fn rs_alist_init(al: ffi::AlistPtr) {
    let ga = unsafe { ffi::nvim_al_ga_ptr(al) };
    unsafe { ffi::nvim_al_ga_init_aentry(ga) };
}

// =============================================================================
// alist_unlink
// =============================================================================

#[export_name = "alist_unlink"]
pub extern "C" fn rs_alist_unlink(al: ffi::AlistPtr) {
    let global_al = unsafe { ffi::nvim_al_get_global_alist() };
    if al != global_al {
        let rc = unsafe { ffi::nvim_al_dec_refcount(al) };
        if rc <= 0 {
            rs_alist_clear(al);
            unsafe { ffi::nvim_al_xfree(al) };
        }
    }
}

// =============================================================================
// alist_new
// =============================================================================

#[export_name = "alist_new"]
pub extern "C" fn rs_alist_new() {
    let curwin = unsafe { ffi::nvim_al_get_curwin() };
    let al = unsafe { ffi::nvim_al_alloc_alist() };
    unsafe {
        ffi::nvim_al_set_refcount(al, 1);
        ffi::nvim_al_set_id(al, ffi::nvim_al_inc_max_alist_id());
        ffi::nvim_al_win_set_alist(curwin, al);
    }
    rs_alist_init(al);
}

// =============================================================================
// alist_add
// =============================================================================

#[export_name = "alist_add"]
pub extern "C" fn rs_alist_add(al: ffi::AlistPtr, fname: *mut c_char, set_fnum: c_int) {
    if fname.is_null() {
        return;
    }
    if check_arglist_locked() == FAIL {
        return;
    }
    let curwin = unsafe { ffi::nvim_al_get_curwin() };
    unsafe {
        ffi::nvim_al_set_arglist_locked(1);
        ffi::nvim_al_win_set_locked(curwin, 1);
    }

    let ga = unsafe { ffi::nvim_al_ga_ptr(al) };
    let len = unsafe { ffi::nvim_al_ga_get_len(ga) };
    let ae = unsafe { ffi::nvim_al_AARGLIST(al, len) };
    unsafe { ffi::nvim_al_ae_set_fname(ae, fname) };

    if set_fnum > 0 {
        let flags = BLN_LISTED | if set_fnum == 2 { BLN_CURBUF } else { 0 };
        let fnum = unsafe { ffi::nvim_al_buflist_add(fname, flags) };
        unsafe { ffi::nvim_al_ae_set_fnum(ae, fnum) };
    }

    unsafe {
        ffi::nvim_al_ga_set_len(ga, len + 1);
        ffi::nvim_al_set_arglist_locked(0);
        ffi::nvim_al_win_set_locked(curwin, 0);
    }
}

// =============================================================================
// alist_set
// =============================================================================

#[export_name = "alist_set"]
pub extern "C" fn rs_alist_set(
    al: ffi::AlistPtr,
    count: c_int,
    files: *mut *mut c_char,
    use_curbuf: c_int,
    fnum_list: *const c_int,
    fnum_len: c_int,
) {
    if check_arglist_locked() == FAIL {
        return;
    }

    rs_alist_clear(al);
    let ga = unsafe { ffi::nvim_al_ga_ptr(al) };
    unsafe { ffi::nvim_al_ga_grow(ga, count) };

    #[allow(clippy::cast_sign_loss)]
    let n = count.max(0) as usize;
    #[allow(clippy::cast_sign_loss)]
    let fnum_n = fnum_len.max(0) as usize;
    let mut i = 0usize;
    while i < n {
        if unsafe { ffi::nvim_al_get_got_int() } != 0 {
            while i < n {
                let file_ptr = unsafe { *files.add(i) };
                unsafe { ffi::nvim_al_xfree(file_ptr.cast::<c_void>()) };
                i += 1;
            }
            break;
        }

        if !fnum_list.is_null() && i < fnum_n {
            let fnum = unsafe { *fnum_list.add(i) };
            let file_ptr = unsafe { *files.add(i) };
            unsafe {
                ffi::nvim_al_set_arglist_locked(1);
                ffi::nvim_al_buf_set_name(fnum, file_ptr);
                ffi::nvim_al_set_arglist_locked(0);
            }
        }

        let file_ptr = unsafe { *files.add(i) };
        rs_alist_add(al, file_ptr, if use_curbuf != 0 { 2 } else { 1 });
        unsafe { ffi::nvim_al_os_breakcheck() };

        i += 1;
    }
    unsafe { ffi::nvim_al_xfree(files.cast::<c_void>()) };

    let global_al = unsafe { ffi::nvim_al_get_global_alist() };
    if al == global_al {
        unsafe { ffi::nvim_al_set_arg_had_last(0) };
    }
}

// =============================================================================
// alist_expand - no-op on Linux (guarded by #if !defined(UNIX) in C)
// =============================================================================

#[export_name = "alist_expand"]
pub extern "C" fn rs_alist_expand(_fnum_list: *mut c_int, _fnum_len: c_int) {
    // No-op on Unix/Linux systems
}
