//! Ex command handlers for the argument list
//!
//! Phase 6: ex_previous, ex_rewind, ex_last, ex_argument, do_argfile, ex_next, ex_argdedupe

use std::ffi::{c_char, c_int, c_void};

use crate::ffi::{self, ExargPtr};
use crate::{
    AL_ADD, AL_DEL, AL_SET, CCGD_AW, CCGD_EXCMD, CCGD_FORCEIT, CCGD_MULTWIN, CMD_ARGDO,
    CMD_ARGGLOBAL, CMD_ARGLOCAL, CMD_ARGS, CMD_SNEXT, ECMD_FORCEIT, ECMD_HIDE, ECMD_LAST, FAIL,
};

const NUL_CHAR: c_char = 0;
#[allow(clippy::cast_possible_wrap)]
const APOSTROPHE: c_char = b'\'' as c_char;
#[allow(clippy::cast_possible_wrap)]
const LOWER_S: c_char = b's' as c_char;

// =============================================================================
// do_argfile
// =============================================================================

/// Edit file "argn" of the argument lists.
#[allow(clippy::cast_sign_loss)]
unsafe fn do_argfile(eap: ExargPtr, argn: c_int) {
    let cmd = ffi::nvim_al_eap_get_cmd(eap);
    let is_split_cmd = *cmd == LOWER_S;

    let curwin = ffi::nvim_al_get_curwin();
    let old_arg_idx = ffi::nvim_al_win_get_arg_idx(curwin);
    let argcount = ffi::nvim_al_ARGCOUNT();

    if argn < 0 || argn >= argcount {
        if argcount <= 1 {
            ffi::nvim_al_emsg_E163();
        } else if argn < 0 {
            ffi::nvim_al_emsg_E164();
        } else {
            ffi::nvim_al_emsg_E165();
        }
        return;
    }

    let curbuf = ffi::nvim_al_get_curbuf();
    let al = ffi::nvim_al_ALIST_curwin();
    let ae = ffi::nvim_al_AARGLIST(al, argn);
    let ae_fnum = ffi::nvim_al_ae_get_fnum(ae);
    let buf_fnum = ffi::nvim_al_buf_get_fnum(curbuf);

    if !is_split_cmd
        && ae_fnum != buf_fnum
        && ffi::nvim_al_check_can_set_curbuf_forceit(ffi::nvim_al_eap_get_forceit(eap)) == 0
    {
        return;
    }

    ffi::nvim_al_setpcmark();

    // split window or create new tab page first
    let cmod_tab = ffi::nvim_al_get_cmdmod_cmod_tab();
    if is_split_cmd || cmod_tab != 0 {
        if ffi::nvim_al_win_split(0, 0) == FAIL {
            return;
        }
        let curwin = ffi::nvim_al_get_curwin();
        ffi::nvim_al_reset_binding(curwin);
    } else {
        // if 'hidden' set, only check for changed file when re-editing
        // the same buffer
        let curbuf = ffi::nvim_al_get_curbuf();
        let mut other = true;
        if ffi::nvim_al_buf_hide(curbuf) {
            let al = ffi::nvim_al_ALIST_curwin();
            let ae = ffi::nvim_al_AARGLIST(al, argn);
            let ae_name = crate::query::alist_name(ae);
            let p = ffi::nvim_al_fix_fname(ae_name);
            other = ffi::nvim_al_otherfile(p);
            ffi::nvim_al_xfree(p.cast::<c_void>());
        }
        let curbuf = ffi::nvim_al_get_curbuf();
        let forceit = ffi::nvim_al_eap_get_forceit(eap);
        if (!ffi::nvim_al_buf_hide(curbuf) || !other)
            && ffi::nvim_al_check_changed(
                curbuf,
                CCGD_AW
                    | (if other { 0 } else { CCGD_MULTWIN })
                    | (if forceit != 0 { CCGD_FORCEIT } else { 0 })
                    | CCGD_EXCMD,
            ) != 0
        {
            return;
        }
    }

    let curwin = ffi::nvim_al_get_curwin();
    ffi::nvim_al_win_set_arg_idx(curwin, argn);
    let argcount = ffi::nvim_al_ARGCOUNT();
    if argn == argcount - 1 && ffi::nvim_al_win_get_alist(curwin) == ffi::nvim_al_get_global_alist()
    {
        ffi::nvim_al_set_arg_had_last(1);
    }

    // Edit the file; always use the last known line number.
    let curwin = ffi::nvim_al_get_curwin();
    let al = ffi::nvim_al_ALIST_curwin();
    let cur_arg_idx = ffi::nvim_al_win_get_arg_idx(curwin);
    let ae = ffi::nvim_al_AARGLIST(al, cur_arg_idx);
    let ae_name = crate::query::alist_name(ae);
    let win_buf = ffi::nvim_al_win_get_buffer(curwin);
    let forceit = ffi::nvim_al_eap_get_forceit(eap);
    let flags = (if ffi::nvim_al_buf_hide(win_buf) {
        ECMD_HIDE
    } else {
        0
    }) + (if forceit != 0 { ECMD_FORCEIT } else { 0 });

    if ffi::nvim_al_do_ecmd(0, ae_name, std::ptr::null(), eap, ECMD_LAST, flags, curwin) == FAIL {
        let curwin = ffi::nvim_al_get_curwin();
        ffi::nvim_al_win_set_arg_idx(curwin, old_arg_idx);
    } else {
        let cmdidx = ffi::nvim_al_eap_get_cmdidx(eap);
        if cmdidx != CMD_ARGDO {
            // like Vi: set the mark where the cursor is in the file.
            ffi::nvim_al_setmark(c_int::from(APOSTROPHE));
        }
    }
}

#[export_name = "do_argfile"]
pub extern "C" fn rs_do_argfile(eap: ExargPtr, argn: c_int) {
    unsafe { do_argfile(eap, argn) }
}

// =============================================================================
// ex_previous
// =============================================================================

/// ":previous", ":sprevious", ":Next" and ":sNext".
#[export_name = "ex_previous"]
pub extern "C" fn rs_ex_previous(eap: ExargPtr) {
    unsafe {
        let curwin = ffi::nvim_al_get_curwin();
        let arg_idx = ffi::nvim_al_win_get_arg_idx(curwin);
        let line2 = ffi::nvim_al_eap_get_line2(eap);
        let argcount = ffi::nvim_al_ARGCOUNT();
        // If past the last one already, go to the last one.
        if arg_idx - line2 >= argcount {
            do_argfile(eap, argcount - 1);
        } else {
            do_argfile(eap, arg_idx - line2);
        }
    }
}

// =============================================================================
// ex_rewind
// =============================================================================

/// ":rewind", ":first", ":sfirst" and ":srewind".
#[export_name = "ex_rewind"]
pub extern "C" fn rs_ex_rewind(eap: ExargPtr) {
    unsafe { do_argfile(eap, 0) }
}

// =============================================================================
// ex_last
// =============================================================================

/// ":last" and ":slast".
#[export_name = "ex_last"]
pub extern "C" fn rs_ex_last(eap: ExargPtr) {
    unsafe {
        let argcount = ffi::nvim_al_ARGCOUNT();
        do_argfile(eap, argcount - 1);
    }
}

// =============================================================================
// ex_argument
// =============================================================================

/// ":argument" and ":sargument".
#[export_name = "ex_argument"]
pub extern "C" fn rs_ex_argument(eap: ExargPtr) {
    unsafe {
        let i = if ffi::nvim_al_eap_get_addr_count(eap) > 0 {
            ffi::nvim_al_eap_get_line2(eap) - 1
        } else {
            let curwin = ffi::nvim_al_get_curwin();
            ffi::nvim_al_win_get_arg_idx(curwin)
        };
        do_argfile(eap, i);
    }
}

// =============================================================================
// ex_next
// =============================================================================

/// ":next", and commands that behave like it.
#[export_name = "ex_next"]
pub extern "C" fn rs_ex_next(eap: ExargPtr) {
    unsafe {
        let curbuf = ffi::nvim_al_get_curbuf();
        let forceit = ffi::nvim_al_eap_get_forceit(eap);
        let cmdidx = ffi::nvim_al_eap_get_cmdidx(eap);

        // check for changed buffer now, if this fails the argument list is not redefined.
        if ffi::nvim_al_buf_hide(curbuf)
            || cmdidx == CMD_SNEXT
            || ffi::nvim_al_check_changed(
                curbuf,
                CCGD_AW | (if forceit != 0 { CCGD_FORCEIT } else { 0 }) | CCGD_EXCMD,
            ) == 0
        {
            let arg = ffi::nvim_al_eap_get_arg(eap);
            let i = if *arg == NUL_CHAR {
                let curwin = ffi::nvim_al_get_curwin();
                let line2 = ffi::nvim_al_eap_get_line2(eap);
                ffi::nvim_al_win_get_arg_idx(curwin) + line2
            } else {
                // redefine file list
                if crate::manipulation::rs_do_arglist(arg, AL_SET, 0, 1) == FAIL {
                    return;
                }
                0
            };
            do_argfile(eap, i);
        }
    }
}

// =============================================================================
// ex_argdedupe
// =============================================================================

/// ":argdedupe"
#[export_name = "ex_argdedupe"]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_ex_argdedupe() {
    unsafe {
        let mut i = 0;
        while i < ffi::nvim_al_ARGCOUNT() {
            let al = ffi::nvim_al_ALIST_curwin();
            let ae_i = ffi::nvim_al_AARGLIST(al, i);
            let fname_i = ffi::nvim_al_ae_get_fname(ae_i);
            let first_full = ffi::nvim_al_FullName_save(fname_i, 0);

            let mut j = i + 1;
            while j < ffi::nvim_al_ARGCOUNT() {
                let al = ffi::nvim_al_ALIST_curwin();
                let ae_j = ffi::nvim_al_AARGLIST(al, j);
                let fname_j = ffi::nvim_al_ae_get_fname(ae_j);
                let second_full = ffi::nvim_al_FullName_save(fname_j, 0);
                let are_dups = ffi::nvim_al_path_fnamecmp(first_full, second_full) == 0;
                ffi::nvim_al_xfree(second_full.cast::<c_void>());

                if are_dups {
                    // remove one duplicate argument
                    let al = ffi::nvim_al_ALIST_curwin();
                    let ae_j = ffi::nvim_al_AARGLIST(al, j);
                    let fname_j = ffi::nvim_al_ae_get_fname(ae_j);
                    ffi::nvim_al_xfree(fname_j.cast::<c_void>());
                    let argcount = ffi::nvim_al_ARGCOUNT();
                    let dst = ffi::nvim_al_AARGLIST(al, j);
                    let src = ffi::nvim_al_AARGLIST(al, j + 1);
                    ffi::nvim_al_memmove_aentry(dst, src, argcount - j - 1);
                    let ga = ffi::nvim_al_ga_ptr(al);
                    let len = ffi::nvim_al_ga_get_len(ga);
                    ffi::nvim_al_ga_set_len(ga, len - 1);

                    let curwin = ffi::nvim_al_get_curwin();
                    let arg_idx = ffi::nvim_al_win_get_arg_idx(curwin);
                    if arg_idx == j {
                        ffi::nvim_al_win_set_arg_idx(curwin, i);
                    } else if arg_idx > j {
                        ffi::nvim_al_win_set_arg_idx(curwin, arg_idx - 1);
                    }

                    j -= 1;
                }
                j += 1;
            }

            ffi::nvim_al_xfree(first_full.cast::<c_void>());
            i += 1;
        }
    }
}

// =============================================================================
// Phase 7: Complex Ex Commands
// =============================================================================

// =============================================================================
// ex_args
// =============================================================================

/// ":args", ":arglocal" and ":argglobal".
#[export_name = "ex_args"]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_ex_args(eap: ExargPtr) {
    unsafe {
        let cmdidx = ffi::nvim_al_eap_get_cmdidx(eap);

        if cmdidx != CMD_ARGS {
            if crate::core::check_arglist_locked() == FAIL {
                return;
            }
            let curwin = ffi::nvim_al_get_curwin();
            let al = ffi::nvim_al_win_get_alist(curwin);
            crate::core::rs_alist_unlink(al);
            if cmdidx == CMD_ARGGLOBAL {
                let curwin = ffi::nvim_al_get_curwin();
                ffi::nvim_al_win_set_alist(curwin, ffi::nvim_al_get_global_alist());
            } else {
                // cmdidx == CMD_arglocal
                crate::core::rs_alist_new();
            }
        }

        // ":args file ..": define new argument list, handle like ":next"
        let arg = ffi::nvim_al_eap_get_arg(eap);
        if *arg != NUL_CHAR {
            if crate::core::check_arglist_locked() == FAIL {
                return;
            }
            rs_ex_next(eap);
            return;
        }

        // ":args": list arguments.
        let cmdidx = ffi::nvim_al_eap_get_cmdidx(eap);
        if cmdidx == CMD_ARGS {
            let argcount = ffi::nvim_al_ARGCOUNT();
            if argcount <= 0 {
                return; // empty argument list
            }

            // Allocate char** array
            let items =
                ffi::nvim_al_xmalloc((argcount as usize) * std::mem::size_of::<*mut c_char>())
                    .cast::<*mut c_char>();

            // Overwrite the command, for a short list there is no scrolling
            // required and no wait_return().
            ffi::nvim_al_gotocmdline(1);

            let al = ffi::nvim_al_ALIST_curwin();
            for i in 0..argcount {
                let ae = ffi::nvim_al_AARGLIST(al, i);
                *items.add(i as usize) = crate::query::alist_name(ae);
            }
            let curwin = ffi::nvim_al_get_curwin();
            ffi::list_in_columns(items, argcount, ffi::nvim_al_win_get_arg_idx(curwin));
            ffi::nvim_al_xfree(items.cast::<c_void>());

            return;
        }

        // ":argslocal": make a local copy of the global argument list.
        let cmdidx = ffi::nvim_al_eap_get_cmdidx(eap);
        if cmdidx == CMD_ARGLOCAL {
            let curwin = ffi::nvim_al_get_curwin();
            let al = ffi::nvim_al_win_get_alist(curwin);
            let ga = ffi::nvim_al_ga_ptr(al);
            let gargcount = ffi::nvim_al_GARGCOUNT();
            ffi::nvim_al_ga_grow(ga, gargcount);

            let global_al = ffi::nvim_al_get_global_alist();
            for i in 0..gargcount {
                let gae = ffi::nvim_al_AARGLIST(global_al, i);
                let fname = ffi::nvim_al_ae_get_fname(gae);
                if !fname.is_null() {
                    let ga_len = ffi::nvim_al_ga_get_len(ga);
                    let curwin = ffi::nvim_al_get_curwin();
                    let al = ffi::nvim_al_win_get_alist(curwin);
                    let new_ae = ffi::nvim_al_AARGLIST(al, ga_len);
                    ffi::nvim_al_ae_set_fname(new_ae, ffi::nvim_al_xstrdup(fname));
                    let fnum = ffi::nvim_al_ae_get_fnum(gae);
                    ffi::nvim_al_ae_set_fnum(new_ae, fnum);
                    let ga = ffi::nvim_al_ga_ptr(al);
                    ffi::nvim_al_ga_set_len(ga, ga_len + 1);
                }
            }
        }
    }
}

// =============================================================================
// ex_argedit
// =============================================================================

/// ":argedit"
#[export_name = "ex_argedit"]
pub extern "C" fn rs_ex_argedit(eap: ExargPtr) {
    unsafe {
        let addr_count = ffi::nvim_al_eap_get_addr_count(eap);
        let mut i = if addr_count != 0 {
            ffi::nvim_al_eap_get_line2(eap)
        } else {
            let curwin = ffi::nvim_al_get_curwin();
            ffi::nvim_al_win_get_arg_idx(curwin) + 1
        };
        // Whether curbuf will be reused, curbuf->b_ffname will be set.
        let curbuf_is_reusable = ffi::nvim_al_curbuf_reusable();

        let arg = ffi::nvim_al_eap_get_arg(eap);
        if crate::manipulation::rs_do_arglist(arg, AL_ADD, i, 1) == FAIL {
            return;
        }
        ffi::nvim_al_maketitle();

        let curwin = ffi::nvim_al_get_curwin();
        if ffi::nvim_al_win_get_arg_idx(curwin) == 0
            && ffi::nvim_al_curbuf_ml_empty() != 0
            && (ffi::nvim_al_curbuf_b_ffname().is_null() || curbuf_is_reusable)
        {
            i = 0;
        }
        // Edit the argument.
        let argcount = ffi::nvim_al_ARGCOUNT();
        if i < argcount {
            do_argfile(eap, i);
        }
    }
}

// =============================================================================
// ex_argadd
// =============================================================================

/// ":argadd"
#[export_name = "ex_argadd"]
pub extern "C" fn rs_ex_argadd(eap: ExargPtr) {
    unsafe {
        let arg = ffi::nvim_al_eap_get_arg(eap);
        let addr_count = ffi::nvim_al_eap_get_addr_count(eap);
        let after = if addr_count > 0 {
            ffi::nvim_al_eap_get_line2(eap)
        } else {
            let curwin = ffi::nvim_al_get_curwin();
            ffi::nvim_al_win_get_arg_idx(curwin) + 1
        };
        crate::manipulation::rs_do_arglist(arg, AL_ADD, after, 0);
        ffi::nvim_al_maketitle();
    }
}

// =============================================================================
// ex_argdelete
// =============================================================================

/// ":argdelete"
#[export_name = "ex_argdelete"]
pub extern "C" fn rs_ex_argdelete(eap: ExargPtr) {
    unsafe {
        if crate::core::check_arglist_locked() == FAIL {
            return;
        }

        let addr_count = ffi::nvim_al_eap_get_addr_count(eap);
        let arg = ffi::nvim_al_eap_get_arg(eap);

        if addr_count > 0 || *arg == NUL_CHAR {
            // ":argdel" works like ":.argdel"
            if addr_count == 0 {
                let curwin = ffi::nvim_al_get_curwin();
                let argcount = ffi::nvim_al_ARGCOUNT();
                if ffi::nvim_al_win_get_arg_idx(curwin) >= argcount {
                    ffi::nvim_al_emsg_E610();
                    return;
                }
                let idx = ffi::nvim_al_win_get_arg_idx(curwin) + 1;
                ffi::nvim_al_eap_set_line1(eap, idx);
                ffi::nvim_al_eap_set_line2(eap, idx);
            } else {
                let line2 = ffi::nvim_al_eap_get_line2(eap);
                let argcount = ffi::nvim_al_ARGCOUNT();
                if line2 > argcount {
                    // ":1,4argdel": Delete all arguments in the range.
                    ffi::nvim_al_eap_set_line2(eap, argcount);
                }
            }
            let line1 = ffi::nvim_al_eap_get_line1(eap);
            let line2 = ffi::nvim_al_eap_get_line2(eap);
            let n = line2 - line1 + 1;
            let arg = ffi::nvim_al_eap_get_arg(eap);
            if *arg != NUL_CHAR {
                // Can't have both a range and an argument.
                ffi::nvim_al_emsg_invarg();
            } else if n <= 0 {
                // Don't give an error for ":%argdel" if the list is empty.
                if line1 != 1 || line2 != 0 {
                    ffi::nvim_al_emsg_invrange();
                }
            } else {
                // Free the filenames in the range
                let al = ffi::nvim_al_ALIST_curwin();
                for i in line1..=line2 {
                    let ae = ffi::nvim_al_AARGLIST(al, i - 1);
                    let fname = ffi::nvim_al_ae_get_fname(ae);
                    ffi::nvim_al_xfree(fname.cast::<c_void>());
                }
                // Move remaining entries
                let al = ffi::nvim_al_ALIST_curwin();
                let dst = ffi::nvim_al_AARGLIST(al, line1 - 1);
                let src = ffi::nvim_al_AARGLIST(al, line2);
                let argcount = ffi::nvim_al_ARGCOUNT();
                ffi::nvim_al_memmove_aentry(dst, src, argcount - line2);
                let ga = ffi::nvim_al_ga_ptr(al);
                let ga_len = ffi::nvim_al_ga_get_len(ga);
                ffi::nvim_al_ga_set_len(ga, ga_len - n);

                let curwin = ffi::nvim_al_get_curwin();
                let arg_idx = ffi::nvim_al_win_get_arg_idx(curwin);
                if arg_idx >= line2 {
                    ffi::nvim_al_win_set_arg_idx(curwin, arg_idx - n);
                } else if arg_idx > line1 {
                    ffi::nvim_al_win_set_arg_idx(curwin, line1);
                }
                let argcount = ffi::nvim_al_ARGCOUNT();
                let curwin = ffi::nvim_al_get_curwin();
                let arg_idx = ffi::nvim_al_win_get_arg_idx(curwin);
                if argcount == 0 {
                    ffi::nvim_al_win_set_arg_idx(curwin, 0);
                } else if arg_idx >= argcount {
                    ffi::nvim_al_win_set_arg_idx(curwin, argcount - 1);
                }
            }
        } else {
            let arg = ffi::nvim_al_eap_get_arg(eap);
            crate::manipulation::rs_do_arglist(arg, AL_DEL, 0, 0);
        }
        ffi::nvim_al_maketitle();
    }
}
