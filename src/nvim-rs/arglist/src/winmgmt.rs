//! Window management for the argument list
//!
//! Phase 8: arg_all_close_unused_windows, arg_all_open_windows, do_arg_all, ex_all

use std::ffi::{c_int, c_void};

use crate::ffi::{self, BufPtr, ExargPtr, TabpagePtr, WinPtr};
use crate::{
    CMD_DROP, ECMD_HIDE, ECMD_OLDBUF, ECMD_ONE, FAIL, K_EQUAL_FILES, OK, WSP_BELOW, WSP_ROOM,
};

// =============================================================================
// arg_all_close_unused_windows (static in C)
// =============================================================================

/// State for arg_all operations.
/// Fields mirror C's arg_all_state_T exactly in layout.
struct ArgAllState {
    alist: ffi::AlistPtr,
    had_tab: c_int,
    keep_tabs: bool,
    forceit: bool,
    use_firstwin: bool,
    opened: *mut u8,
    opened_len: c_int,
    new_curwin: WinPtr,
    new_curtab: TabpagePtr,
}

/// Close all windows containing files not in the argument list.
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::too_many_lines)]
unsafe fn arg_all_close_unused_windows(aall: &mut ArgAllState) {
    let old_curwin = ffi::nvim_al_get_curwin();
    let old_curtab = ffi::nvim_al_get_curtab();

    if aall.had_tab > 0 {
        ffi::nvim_al_goto_tabpage_tp(ffi::nvim_al_get_first_tabpage(), 1, 1);
    }

    // moving tabpages around in an autocommand may cause an endless loop
    let tmd = ffi::nvim_al_get_tabpage_move_disallowed();
    ffi::nvim_al_set_tabpage_move_disallowed(tmd + 1);

    loop {
        let mut wpnext: WinPtr;
        let curtab = ffi::nvim_al_get_curtab();
        let tpnext = ffi::nvim_al_tp_get_next(curtab);

        // Try to close floating windows first
        let lastwin = ffi::nvim_al_get_lastwin();
        let firstwin = ffi::nvim_al_get_firstwin();
        let mut wp = if ffi::nvim_al_win_is_floating(lastwin) != 0 {
            lastwin
        } else {
            firstwin
        };

        while !wp.is_null() {
            // Compute wpnext
            if ffi::nvim_al_win_is_floating(wp) != 0 {
                let prev = ffi::nvim_al_win_get_prev(wp);
                wpnext = if ffi::nvim_al_win_is_floating(prev) != 0 {
                    prev
                } else {
                    ffi::nvim_al_get_firstwin()
                };
            } else {
                let next = ffi::nvim_al_win_get_next(wp);
                wpnext = if next.is_null() || ffi::nvim_al_win_is_floating(next) != 0 {
                    std::ptr::null_mut()
                } else {
                    next
                };
            }

            let buf = ffi::nvim_al_win_get_buffer(wp);
            let buf_ffname = ffi::nvim_al_buf_get_ffname(buf);

            let i = if buf_ffname.is_null()
                || (!aall.keep_tabs
                    && (ffi::nvim_al_buf_get_nwindows(buf) > 1
                        || ffi::nvim_al_win_get_width(wp) != ffi::nvim_al_get_Columns()
                        || (ffi::nvim_al_win_is_floating(wp) != 0
                            && !ffi::nvim_al_is_aucmd_win(wp))))
            {
                aall.opened_len
            } else {
                // check if the buffer in this window is in the arglist
                find_arg_in_list(aall, buf, old_curwin, old_curtab, wp)
            };
            ffi::nvim_al_win_set_arg_idx(wp, i);

            if i == aall.opened_len && !aall.keep_tabs {
                // close this window
                if ffi::nvim_al_buf_hide(buf)
                    || aall.forceit
                    || ffi::nvim_al_buf_get_nwindows(buf) > 1
                    || !ffi::nvim_al_bufIsChanged(buf)
                {
                    // If the buffer was changed, and we would like to hide it, try autowriting.
                    if !ffi::nvim_al_buf_hide(buf)
                        && ffi::nvim_al_buf_get_nwindows(buf) <= 1
                        && ffi::nvim_al_bufIsChanged(buf)
                    {
                        let bufref = ffi::nvim_al_bufref_create(buf);
                        ffi::nvim_al_autowrite(buf, 0);
                        // Check if autocommands removed the window.
                        if ffi::nvim_al_win_valid(wp) == 0 || ffi::nvim_al_bufref_valid(bufref) == 0
                        {
                            ffi::nvim_al_bufref_destroy(bufref);
                            let lastwin = ffi::nvim_al_get_lastwin();
                            wpnext = if ffi::nvim_al_win_is_floating(lastwin) != 0 {
                                lastwin
                            } else {
                                ffi::nvim_al_get_firstwin()
                            };
                            wp = wpnext;
                            continue;
                        }
                        ffi::nvim_al_bufref_destroy(bufref);
                    }
                    // don't close last window
                    if ffi::nvim_al_ONE_WINDOW() != 0
                        && (ffi::nvim_al_tp_get_next(ffi::nvim_al_get_first_tabpage()).is_null()
                            || aall.had_tab == 0)
                    {
                        aall.use_firstwin = true;
                    } else {
                        ffi::nvim_al_win_close(
                            wp,
                            c_int::from(
                                !ffi::nvim_al_buf_hide(buf) && !ffi::nvim_al_bufIsChanged(buf),
                            ),
                            0,
                        );
                        // check if autocommands removed the next window
                        if ffi::nvim_al_win_valid(wpnext) == 0 {
                            let lastwin = ffi::nvim_al_get_lastwin();
                            wpnext = if ffi::nvim_al_win_is_floating(lastwin) != 0 {
                                lastwin
                            } else {
                                ffi::nvim_al_get_firstwin()
                            };
                        }
                    }
                }
            }

            wp = wpnext;
        }

        // Without the ":tab" modifier only do the current tab page.
        if aall.had_tab == 0 || tpnext.is_null() {
            break;
        }

        // check if autocommands removed the next tab page
        let tpnext = if ffi::nvim_al_valid_tabpage(tpnext) == 0 {
            ffi::nvim_al_get_first_tabpage() // start all over...
        } else {
            tpnext
        };
        ffi::nvim_al_goto_tabpage_tp(tpnext, 1, 1);
    }

    let tmd = ffi::nvim_al_get_tabpage_move_disallowed();
    ffi::nvim_al_set_tabpage_move_disallowed(tmd - 1);
}

/// Find if buffer is in the arglist, update opened weights.
/// Returns the index (or opened_len if not found).
#[allow(clippy::cast_sign_loss)]
unsafe fn find_arg_in_list(
    aall: &mut ArgAllState,
    buf: BufPtr,
    old_curwin: WinPtr,
    old_curtab: TabpagePtr,
    wp: WinPtr,
) -> c_int {
    let buf_fnum = ffi::nvim_al_buf_get_fnum(buf);
    let buf_ffname = ffi::nvim_al_buf_get_ffname(buf);

    for i in 0..aall.opened_len {
        let ga = ffi::nvim_al_ga_ptr(aall.alist);
        let ga_len = ffi::nvim_al_ga_get_len(ga);
        if i < ga_len {
            let ae = ffi::nvim_al_AARGLIST(aall.alist, i);
            let ae_fnum = ffi::nvim_al_ae_get_fnum(ae);
            let ae_name = crate::query::alist_name(ae);
            if ae_fnum == buf_fnum
                || (ffi::nvim_al_path_full_compare(ae_name, buf_ffname, 1, 1) & K_EQUAL_FILES != 0)
            {
                let mut weight: u8 = 1;

                let curtab = ffi::nvim_al_get_curtab();
                if old_curtab == curtab {
                    weight += 1;
                    if old_curwin == wp {
                        weight += 1;
                    }
                }

                if weight > *aall.opened.add(i as usize) {
                    *aall.opened.add(i as usize) = weight;
                    if i == 0 {
                        if !aall.new_curwin.is_null() {
                            ffi::nvim_al_win_set_arg_idx(aall.new_curwin, aall.opened_len);
                        }
                        aall.new_curwin = wp;
                        let curtab = ffi::nvim_al_get_curtab();
                        aall.new_curtab = curtab;
                    }
                } else if aall.keep_tabs {
                    return aall.opened_len;
                }

                if ffi::nvim_al_win_get_alist(wp) != aall.alist {
                    // Use the current argument list for all windows
                    // containing a file from it.
                    let wp_alist = ffi::nvim_al_win_get_alist(wp);
                    crate::core::rs_alist_unlink(wp_alist);
                    ffi::nvim_al_win_set_alist(wp, aall.alist);
                    ffi::nvim_al_inc_refcount(aall.alist);
                }
                return i;
            }
        }
    }
    aall.opened_len
}

// =============================================================================
// arg_all_open_windows (static in C)
// =============================================================================

/// Callback data for finding windows by arg_idx in tab.
struct FindWinByArgIdxData {
    arg_idx: c_int,
    found_wp: WinPtr,
}

unsafe extern "C" fn find_win_by_arg_idx_cb(wp: WinPtr, ud: *mut c_void) -> c_int {
    let data = &mut *(ud.cast::<FindWinByArgIdxData>());
    if ffi::nvim_al_win_get_arg_idx(wp) == data.arg_idx {
        data.found_wp = wp;
        return 1; // stop iteration
    }
    0
}

/// Open up to "count" windows for the files in the argument list.
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::too_many_lines)]
unsafe fn arg_all_open_windows(aall: &mut ArgAllState, count: c_int) {
    let mut tab_drop_empty_window = false;

    // ":tab drop file" should re-use an empty window
    let curbuf = ffi::nvim_al_get_curbuf();
    if aall.keep_tabs
        && ffi::nvim_al_buf_is_empty(curbuf) != 0
        && ffi::nvim_al_buf_get_nwindows(curbuf) == 1
        && ffi::nvim_al_buf_get_ffname(curbuf).is_null()
        && ffi::nvim_al_buf_get_changed(curbuf) == 0
    {
        aall.use_firstwin = true;
        tab_drop_empty_window = true;
    }

    let mut split_ret = OK;

    let mut i = 0;
    while i < count && ffi::nvim_al_get_got_int() == 0 {
        if aall.alist == ffi::nvim_al_get_global_alist() {
            let ga = ffi::nvim_al_ga_ptr(aall.alist);
            let ga_len = ffi::nvim_al_ga_get_len(ga);
            if i == ga_len - 1 {
                ffi::nvim_al_set_arg_had_last(1);
            }
        }

        if *aall.opened.add(i as usize) > 0 {
            // Move the already present window to below the current window
            let curwin = ffi::nvim_al_get_curwin();
            if ffi::nvim_al_win_get_arg_idx(curwin) != i {
                let curtab = ffi::nvim_al_get_curtab();
                let mut data = FindWinByArgIdxData {
                    arg_idx: i,
                    found_wp: std::ptr::null_mut(),
                };
                ffi::nvim_al_foreach_windows_in_tab(
                    find_win_by_arg_idx_cb,
                    curtab,
                    std::ptr::addr_of_mut!(data).cast::<c_void>(),
                );
                if !data.found_wp.is_null() {
                    let found_wp = data.found_wp;
                    if aall.keep_tabs {
                        aall.new_curwin = found_wp;
                        let curtab = ffi::nvim_al_get_curtab();
                        aall.new_curtab = curtab;
                    } else if ffi::nvim_al_win_is_floating(found_wp) != 0 {
                        // break from the search — do nothing
                    } else {
                        let curwin = ffi::nvim_al_get_curwin();
                        if ffi::nvim_al_win_get_frame_parent(found_wp)
                            != ffi::nvim_al_win_get_frame_parent(curwin)
                        {
                            ffi::nvim_al_emsg_e_window_layout();
                            i = count;
                            i += 1;
                            continue;
                        }
                        let curwin = ffi::nvim_al_get_curwin();
                        ffi::nvim_al_win_move_after(found_wp, curwin);
                    }
                }
            }
        } else if split_ret == OK {
            // trigger events for tab drop
            if tab_drop_empty_window && i == count - 1 {
                let v = ffi::nvim_al_get_autocmd_no_enter();
                ffi::nvim_al_set_autocmd_no_enter(v - 1);
            }
            if aall.use_firstwin {
                // first window: do autocmd for leaving this buffer
                let v = ffi::nvim_al_get_autocmd_no_leave();
                ffi::nvim_al_set_autocmd_no_leave(v - 1);
            } else {
                // split current window
                let p_ea_save = ffi::nvim_al_get_p_ea();
                ffi::nvim_al_set_p_ea(1); // use space from all windows
                split_ret = ffi::nvim_al_win_split(0, WSP_ROOM | WSP_BELOW);
                ffi::nvim_al_set_p_ea(p_ea_save);
                if split_ret == FAIL {
                    i += 1;
                    continue;
                }
            }

            // edit file "i"
            let curwin = ffi::nvim_al_get_curwin();
            ffi::nvim_al_win_set_arg_idx(curwin, i);
            if i == 0 {
                aall.new_curwin = curwin;
                let curtab = ffi::nvim_al_get_curtab();
                aall.new_curtab = curtab;
            }
            let ae = ffi::nvim_al_AARGLIST(aall.alist, i);
            let ae_name = crate::query::alist_name(ae);
            let curwin = ffi::nvim_al_get_curwin();
            let win_buf = ffi::nvim_al_win_get_buffer(curwin);
            let flags = (if ffi::nvim_al_buf_hide(win_buf) || ffi::nvim_al_bufIsChanged(win_buf) {
                ECMD_HIDE
            } else {
                0
            }) + ECMD_OLDBUF;
            ffi::nvim_al_do_ecmd(
                0,
                ae_name,
                std::ptr::null(),
                std::ptr::null_mut(),
                ECMD_ONE,
                flags,
                curwin,
            );
            if tab_drop_empty_window && i == count - 1 {
                let v = ffi::nvim_al_get_autocmd_no_enter();
                ffi::nvim_al_set_autocmd_no_enter(v + 1);
            }
            if aall.use_firstwin {
                let v = ffi::nvim_al_get_autocmd_no_leave();
                ffi::nvim_al_set_autocmd_no_leave(v + 1);
            }
            aall.use_firstwin = false;
        }
        ffi::nvim_al_os_breakcheck();

        // When ":tab" was used open a new tab for a new window repeatedly.
        if aall.had_tab > 0
            && ffi::nvim_al_tabpage_index(std::ptr::null_mut()) <= ffi::nvim_al_get_p_tpm()
        {
            ffi::nvim_al_set_cmdmod_cmod_tab(9999);
        }

        i += 1;
    }
}

// =============================================================================
// do_arg_all
// =============================================================================

/// Open up to 'count' windows, one for each argument.
#[allow(clippy::cast_sign_loss)]
unsafe fn do_arg_all(mut count: c_int, forceit: c_int, keep_tabs: c_int) {
    let prev_arglist_locked = ffi::nvim_al_get_arglist_locked();

    if ffi::nvim_al_get_cmdwin_type() != 0 {
        ffi::nvim_al_emsg_e_cmdwin();
        return;
    }
    let argcount = ffi::nvim_al_ARGCOUNT();
    if argcount <= 0 {
        return;
    }
    ffi::nvim_al_setpcmark();

    let opened_len = ffi::nvim_al_ARGCOUNT();
    let mut aall = ArgAllState {
        use_firstwin: false,
        had_tab: ffi::nvim_al_get_cmdmod_cmod_tab(),
        new_curwin: std::ptr::null_mut(),
        new_curtab: std::ptr::null_mut(),
        forceit: forceit != 0,
        keep_tabs: keep_tabs != 0,
        opened_len,
        opened: ffi::nvim_al_xcalloc(opened_len as usize, 1).cast::<u8>(),
        alist: std::ptr::null_mut(), // set below
    };

    // Lock the argument list
    let curwin = ffi::nvim_al_get_curwin();
    aall.alist = ffi::nvim_al_win_get_alist(curwin);
    ffi::nvim_al_inc_refcount(aall.alist);
    ffi::nvim_al_set_arglist_locked(1);

    let curtab = ffi::nvim_al_get_curtab();
    let new_lu_tp = curtab;

    // Stop Visual mode
    ffi::nvim_al_reset_VIsual_and_resel();

    // Close unused windows
    arg_all_close_unused_windows(&mut aall);

    // Open windows for files
    if count > aall.opened_len || count <= 0 {
        count = aall.opened_len;
    }

    // Don't execute Win/Buf Enter/Leave autocommands here.
    let ane = ffi::nvim_al_get_autocmd_no_enter();
    ffi::nvim_al_set_autocmd_no_enter(ane + 1);
    let anl = ffi::nvim_al_get_autocmd_no_leave();
    ffi::nvim_al_set_autocmd_no_leave(anl + 1);
    let last_curwin = ffi::nvim_al_get_curwin();
    let last_curtab = ffi::nvim_al_get_curtab();
    // lastwin may be aucmd_win
    ffi::nvim_al_win_enter(ffi::nvim_al_lastwin_nofloating(), 0);

    // Open up to "count" windows.
    arg_all_open_windows(&mut aall, count);

    // Remove the "lock" on the argument list.
    crate::core::rs_alist_unlink(aall.alist);
    ffi::nvim_al_set_arglist_locked(prev_arglist_locked);

    let ane = ffi::nvim_al_get_autocmd_no_enter();
    ffi::nvim_al_set_autocmd_no_enter(ane - 1);

    // restore last referenced tabpage's curwin
    if last_curtab != aall.new_curtab {
        if ffi::nvim_al_valid_tabpage(last_curtab) != 0 {
            ffi::nvim_al_goto_tabpage_tp(last_curtab, 1, 1);
        }
        if ffi::nvim_al_win_valid(last_curwin) != 0 {
            ffi::nvim_al_win_enter(last_curwin, 0);
        }
    }
    // to window with first arg
    if ffi::nvim_al_valid_tabpage(aall.new_curtab) != 0 {
        ffi::nvim_al_goto_tabpage_tp(aall.new_curtab, 1, 1);
    }

    // Now set the last used tabpage to where we started.
    if ffi::nvim_al_valid_tabpage(new_lu_tp) != 0 {
        ffi::nvim_al_set_lastused_tabpage(new_lu_tp);
    }

    if ffi::nvim_al_win_valid(aall.new_curwin) != 0 {
        ffi::nvim_al_win_enter(aall.new_curwin, 0);
    }

    let anl = ffi::nvim_al_get_autocmd_no_leave();
    ffi::nvim_al_set_autocmd_no_leave(anl - 1);
    ffi::nvim_al_xfree(aall.opened.cast::<c_void>());
}

// =============================================================================
// ex_all
// =============================================================================

/// ":all" and ":sall".
/// Also used for ":tab drop file ..." after setting the argument list.
#[export_name = "ex_all"]
pub extern "C" fn rs_ex_all(eap: ExargPtr) {
    unsafe {
        let addr_count = ffi::nvim_al_eap_get_addr_count(eap);
        if addr_count == 0 {
            ffi::nvim_al_eap_set_line2(eap, 9999);
        }
        let line2 = ffi::nvim_al_eap_get_line2(eap);
        let forceit = ffi::nvim_al_eap_get_forceit(eap);
        let cmdidx = ffi::nvim_al_eap_get_cmdidx(eap);
        do_arg_all(line2, forceit, c_int::from(cmdidx == CMD_DROP));
    }
}
