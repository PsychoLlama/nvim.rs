//! Manipulation helpers for the argument list
//!
//! Phase 4: alist_check_arg_idx, alist_add_list, arglist_del_files, do_arglist, set_arglist

use std::ffi::{c_char, c_int, c_void};

use crate::ffi::{self, GarrayPtr, WinPtr};
use crate::{
    AL_ADD, AL_DEL, AL_SET, BLN_CURBUF, BLN_LISTED, EW_ADDSLASH, EW_DIR, EW_FILE, EW_NOTFOUND,
    FAIL, OK, RE_MAGIC,
};

const NUL_CHAR: c_char = 0;

// =============================================================================
// alist_check_arg_idx (static in C)
// =============================================================================

/// Callback for nvim_al_foreach_tab_window: checks arg_idx for windows sharing
/// the current window's argument list.
unsafe extern "C" fn check_arg_idx_cb(wp: WinPtr, ud: *mut c_void) -> c_int {
    let curwin_alist = ud;
    if ffi::nvim_al_win_get_alist(wp) == curwin_alist {
        ffi::nvim_al_check_arg_idx(wp);
    }
    0 // continue iteration
}

/// Check the validity of the arg_idx for each other window.
unsafe fn alist_check_arg_idx() {
    let curwin = ffi::nvim_al_get_curwin();
    let curwin_alist = ffi::nvim_al_win_get_alist(curwin);
    ffi::nvim_al_foreach_tab_window(check_arg_idx_cb, curwin_alist);
}

// =============================================================================
// alist_add_list (static in C)
// =============================================================================

/// Add files[count] to the arglist of the current window after arg "after".
/// The file names in files[count] must have been allocated and are taken over.
/// Files[] itself is not taken over.
#[allow(clippy::cast_sign_loss)]
unsafe fn alist_add_list(count: c_int, files: *mut *mut c_char, after: c_int, will_edit: bool) {
    let curwin = ffi::nvim_al_get_curwin();
    let al = ffi::nvim_al_ALIST_curwin();
    let old_argcount = ffi::nvim_al_ARGCOUNT();
    let ga = ffi::nvim_al_ga_ptr(al);
    ffi::nvim_al_ga_grow(ga, count);

    if crate::core::check_arglist_locked() != FAIL {
        let after = after.max(0).min(ffi::nvim_al_ARGCOUNT());
        let argcount = ffi::nvim_al_ARGCOUNT();
        if after < argcount {
            let dst = ffi::nvim_al_AARGLIST(al, after + count);
            let src = ffi::nvim_al_AARGLIST(al, after);
            ffi::nvim_al_memmove_aentry(dst, src, argcount - after);
        }
        ffi::nvim_al_set_arglist_locked(1);
        ffi::nvim_al_win_set_locked(curwin, 1);
        let flags = BLN_LISTED | (if will_edit { BLN_CURBUF } else { 0 });
        for i in 0..count {
            let file_ptr = *files.add(i as usize);
            let ae = ffi::nvim_al_AARGLIST(al, after + i);
            ffi::nvim_al_ae_set_fname(ae, file_ptr);
            let fnum = ffi::nvim_al_buflist_add(file_ptr, flags);
            ffi::nvim_al_ae_set_fnum(ae, fnum);
        }
        ffi::nvim_al_set_arglist_locked(0);
        ffi::nvim_al_win_set_locked(curwin, 0);
        let ga = ffi::nvim_al_ga_ptr(al);
        let ga_len = ffi::nvim_al_ga_get_len(ga);
        ffi::nvim_al_ga_set_len(ga, ga_len + count);
        if old_argcount > 0 && ffi::nvim_al_win_get_arg_idx(curwin) >= after {
            let idx = ffi::nvim_al_win_get_arg_idx(curwin);
            ffi::nvim_al_win_set_arg_idx(curwin, idx + count);
        }
    }
}

// =============================================================================
// arglist_del_files (static in C)
// =============================================================================

/// Delete the file names in `alist_ga` from the argument list.
#[allow(clippy::cast_sign_loss)]
unsafe fn arglist_del_files(alist_ga: GarrayPtr) {
    let rm = ffi::nvim_al_regmatch_alloc();
    ffi::nvim_al_regmatch_set_ic(rm, ffi::nvim_al_get_p_fic());

    let ga_len = ffi::nvim_al_ga_get_len(alist_ga);
    let ga_data = ffi::nvim_al_ga_get_data(alist_ga).cast::<*mut c_char>();
    let curwin = ffi::nvim_al_get_curwin();
    let al = ffi::nvim_al_ALIST_curwin();

    let mut i = 0;
    while i < ga_len && ffi::nvim_al_get_got_int() == 0 {
        let p_orig = *ga_data.add(i as usize);
        let p = ffi::nvim_al_file_pat_to_reg_pat(p_orig);
        if p.is_null() {
            break;
        }
        let re_flags = if ffi::nvim_al_magic_isset() != 0 {
            RE_MAGIC
        } else {
            0
        };
        if ffi::nvim_al_regmatch_compile(rm, p, re_flags) == 0 {
            ffi::nvim_al_xfree(p.cast::<c_void>());
            break;
        }

        let mut didone = false;
        let mut m = 0;
        while m < ffi::nvim_al_ARGCOUNT() {
            let ae = ffi::nvim_al_AARGLIST(al, m);
            let name = ffi::nvim_al_alist_name(ae);
            if ffi::nvim_al_regmatch_exec(rm, name) != 0 {
                didone = true;
                let fname = ffi::nvim_al_ae_get_fname(ae);
                ffi::nvim_al_xfree(fname.cast::<c_void>());
                let argcount = ffi::nvim_al_ARGCOUNT();
                let dst = ffi::nvim_al_AARGLIST(al, m);
                let src = ffi::nvim_al_AARGLIST(al, m + 1);
                ffi::nvim_al_memmove_aentry(dst, src, argcount - m - 1);
                let ga = ffi::nvim_al_ga_ptr(al);
                let len = ffi::nvim_al_ga_get_len(ga);
                ffi::nvim_al_ga_set_len(ga, len - 1);
                let idx = ffi::nvim_al_win_get_arg_idx(curwin);
                if idx > m {
                    ffi::nvim_al_win_set_arg_idx(curwin, idx - 1);
                }
                m -= 1;
            }
            m += 1;
        }

        ffi::nvim_al_regmatch_free_prog(rm);
        ffi::nvim_al_xfree(p.cast::<c_void>());
        if !didone {
            ffi::nvim_al_semsg_nomatch2(*ga_data.add(i as usize));
        }
        i += 1;
    }
    ffi::nvim_al_xfree(rm);
    ffi::nvim_al_ga_clear(alist_ga);
}

// =============================================================================
// do_arglist (static in C)
// =============================================================================

/// Redefine, add to, or delete from the argument list.
///
/// what: AL_SET, AL_ADD, or AL_DEL
/// after: 0 means before first one
/// will_edit: will edit added argument
unsafe fn do_arglist(str: *mut c_char, what: c_int, after: c_int, will_edit: bool) -> c_int {
    if crate::core::check_arglist_locked() == FAIL {
        return FAIL;
    }

    let (str, arg_escaped) = if what == AL_ADD && *str == NUL_CHAR {
        let ffname = ffi::nvim_al_curbuf_b_ffname();
        if ffname.is_null() {
            return FAIL;
        }
        (ffi::nvim_al_curbuf_b_fname(), false)
    } else {
        (str, true)
    };

    // Collect all file name arguments in "new_ga".
    let new_ga = ffi::nvim_al_alloc_garray();
    crate::parsing::get_arglist(new_ga, str, arg_escaped);

    if what == AL_DEL {
        arglist_del_files(new_ga);
        ffi::nvim_al_free_garray(new_ga);
    } else {
        let ga_len = ffi::nvim_al_ga_get_len(new_ga);
        let ga_data = ffi::nvim_al_ga_get_data(new_ga).cast::<*mut c_char>();
        let mut exp_count: c_int = 0;
        let mut exp_files: *mut *mut c_char = std::ptr::null_mut();
        let i = ffi::nvim_al_expand_wildcards(
            ga_len,
            ga_data,
            std::ptr::addr_of_mut!(exp_count),
            std::ptr::addr_of_mut!(exp_files),
            EW_DIR | EW_FILE | EW_ADDSLASH | EW_NOTFOUND,
        );
        ffi::nvim_al_ga_clear(new_ga);
        ffi::nvim_al_free_garray(new_ga);
        if i == FAIL || exp_count == 0 {
            ffi::nvim_al_emsg_nomatch();
            return FAIL;
        }

        if what == AL_ADD {
            alist_add_list(exp_count, exp_files, after, will_edit);
            ffi::nvim_al_xfree(exp_files.cast::<c_void>());
        } else {
            // what == AL_SET
            let al = ffi::nvim_al_ALIST_curwin();
            crate::core::rs_alist_set(
                al,
                exp_count,
                exp_files,
                c_int::from(will_edit),
                std::ptr::null_mut(),
                0,
            );
        }
    }

    alist_check_arg_idx();

    OK
}

// =============================================================================
// set_arglist
// =============================================================================

#[export_name = "set_arglist"]
pub extern "C" fn rs_set_arglist(str: *mut c_char) {
    unsafe {
        do_arglist(str, AL_SET, 0, true);
    }
}

// =============================================================================
// do_arglist — exported for use by ex commands (Phase 6+)
// =============================================================================

#[no_mangle]
pub extern "C" fn rs_do_arglist(
    str: *mut c_char,
    what: c_int,
    after: c_int,
    will_edit: c_int,
) -> c_int {
    unsafe { do_arglist(str, what, after, will_edit != 0) }
}
