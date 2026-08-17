//! `nvim_win_set_config()`: reconfiguring an existing window.
//!
//! The two directions a reconfiguration can take: `win_config_split` turns a
//! window into (or moves) a split, which may mean splitting a different parent,
//! changing the direction, or leaving the float layout entirely; and
//! `win_config_float_tp` applies a float config, including the tabpage move a
//! `relative` window may need.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;

unsafe extern "C" fn win_config_split(
    mut win: *mut win_T,
    mut config: *const KeyDict_win_config,
    mut fconfig: *mut WinConfig,
    mut err: *mut Error,
) -> bool {
    unsafe {
        let mut dir: ::core::ffi::c_int = 0;
        let mut unflat_altfr: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
        let mut altwin_0: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut flags: ::core::ffi::c_int = 0;
        let mut parent: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut parent_tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
        let mut win_tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
        let mut to_split_ok: bool = false;
        let mut curwin_moving_tp: bool = false;
        let mut was_split: bool = !(*win).w_floating;
        let mut has_split: bool = has_key(
            (*config).is_set__win_config_,
            KEYSET_OPTIDX_win_config__split,
        );
        let mut has_vertical: bool = has_key(
            (*config).is_set__win_config_,
            KEYSET_OPTIDX_win_config__vertical,
        );
        let mut old_split: WinSplit = win_split_dir(win);
        if has_vertical as ::core::ffi::c_int != 0 && !has_split {
            if (*config).vertical {
                (*fconfig).split = (if old_split as ::core::ffi::c_uint
                    == kWinSplitRight as ::core::ffi::c_int as ::core::ffi::c_uint
                    || p_spr.get() != 0
                {
                    kWinSplitRight as ::core::ffi::c_int
                } else {
                    kWinSplitLeft as ::core::ffi::c_int
                }) as WinSplit;
            } else {
                (*fconfig).split = (if old_split as ::core::ffi::c_uint
                    == kWinSplitBelow as ::core::ffi::c_int as ::core::ffi::c_uint
                    || p_sb.get() != 0
                {
                    kWinSplitBelow as ::core::ffi::c_int
                } else {
                    kWinSplitAbove as ::core::ffi::c_int
                }) as WinSplit;
            }
        }
        '_resize: {
            if !(!has_vertical && !has_split
                || was_split as ::core::ffi::c_int != 0
                    && !(has_key((*config).is_set__win_config_, KEYSET_OPTIDX_win_config__win))
                    && old_split as ::core::ffi::c_uint == (*fconfig).split as ::core::ffi::c_uint)
            {
                parent = ::core::ptr::null_mut::<win_T>();
                parent_tp = ::core::ptr::null_mut::<tabpage_T>();
                if (*config).win == 0 as ::core::ffi::c_int {
                    parent = curwin.get();
                    parent_tp = curtab.get();
                } else if (*config).win > 0 as ::core::ffi::c_int {
                    parent = find_window_by_handle((*fconfig).window, err);
                    if parent.is_null() {
                        return false;
                    }
                    parent_tp = win_find_tabpage(parent);
                }
                win_tp = win_find_tabpage(win);
                if !parent.is_null() {
                    if (*parent).w_floating {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            c"Cannot split a floating window".as_ptr(),
                        );
                        return false;
                    }
                    if win_tp != parent_tp && !win_can_move_tp(win, win_tp, err) {
                        return false;
                    }
                }
                if !check_split_disallowed_err(win, err) {
                    return false;
                }
                to_split_ok = false;
                curwin_moving_tp = win == curwin.get() && !parent.is_null() && win_tp != parent_tp;
                '_restore_curwin: {
                    if curwin_moving_tp {
                        let mut altwin: *mut win_T = win_find_altwin(win, win_tp);
                        debug_assert!(!altwin.is_null(), "altwin");
                        win_goto(altwin);
                        if curwin.get() == win {
                            api_set_error(
                                err,
                                kErrorTypeException,
                                c"Failed to switch away from window %d".as_ptr(),
                                (*win).handle,
                            );
                            return false;
                        }
                        win_tp = win_find_tabpage(win);
                        if win_tp.is_null() || !win_valid_any_tab(parent) {
                            api_set_error(
                                err,
                                kErrorTypeException,
                                c"Windows to split were closed".as_ptr(),
                            );
                            break '_restore_curwin;
                        } else if was_split as ::core::ffi::c_int
                            == (*win).w_floating as ::core::ffi::c_int
                            || (*parent).w_floating as ::core::ffi::c_int != 0
                        {
                            api_set_error(
                                err,
                                kErrorTypeException,
                                c"Floating state of windows to split changed".as_ptr(),
                            );
                            break '_restore_curwin;
                        }
                    }
                    dir = 0 as ::core::ffi::c_int;
                    unflat_altfr = ::core::ptr::null_mut::<frame_T>();
                    altwin_0 = ::core::ptr::null_mut::<win_T>();
                    if was_split {
                        if (*(*win).w_frame).fr_parent.is_null() {
                            api_set_error(
                                err,
                                kErrorTypeException,
                                c"Cannot move last non-floating window".as_ptr(),
                            );
                            break '_restore_curwin;
                        } else if !parent.is_null() && (*parent).handle == (*win).handle {
                            let mut n_frames: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            let mut fr: *mut frame_T = (*(*(*win).w_frame).fr_parent).fr_child;
                            while !fr.is_null() {
                                n_frames += 1;
                                fr = (*fr).fr_next;
                            }
                            let mut neighbor: *mut win_T = ::core::ptr::null_mut::<win_T>();
                            if n_frames > 2 as ::core::ffi::c_int {
                                let mut frame: *mut frame_T = (*(*win).w_frame).fr_parent;
                                if !(*frame).fr_parent.is_null() {
                                    if (*fconfig).split as ::core::ffi::c_uint
                                        == kWinSplitAbove as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                        || (*fconfig).split as ::core::ffi::c_uint
                                            == kWinSplitLeft as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                    {
                                        neighbor = (*win).w_next;
                                    } else {
                                        neighbor = (*win).w_prev;
                                    }
                                }
                                altwin_0 = winframe_remove(
                                    win,
                                    &raw mut dir,
                                    if win_tp == curtab.get() {
                                        ::core::ptr::null_mut::<tabpage_T>()
                                    } else {
                                        win_tp
                                    },
                                    &raw mut unflat_altfr,
                                );
                            } else if n_frames == 2 as ::core::ffi::c_int {
                                altwin_0 = winframe_remove(
                                    win,
                                    &raw mut dir,
                                    if win_tp == curtab.get() {
                                        ::core::ptr::null_mut::<tabpage_T>()
                                    } else {
                                        win_tp
                                    },
                                    &raw mut unflat_altfr,
                                );
                                neighbor = altwin_0;
                            } else {
                                api_set_error(
                                    err,
                                    kErrorTypeException,
                                    c"Cannot split window into itself".as_ptr(),
                                );
                                break '_restore_curwin;
                            }
                            parent = neighbor;
                        } else {
                            altwin_0 = winframe_remove(
                                win,
                                &raw mut dir,
                                if win_tp == curtab.get() {
                                    ::core::ptr::null_mut::<tabpage_T>()
                                } else {
                                    win_tp
                                },
                                &raw mut unflat_altfr,
                            );
                        }
                    } else {
                        altwin_0 = win_float_find_altwin(
                            win,
                            if win_tp == curtab.get() {
                                ::core::ptr::null_mut::<tabpage_T>()
                            } else {
                                win_tp
                            },
                        );
                    }
                    win_remove(
                        win,
                        if win_tp == curtab.get() {
                            ::core::ptr::null_mut::<tabpage_T>()
                        } else {
                            win_tp
                        },
                    );
                    if win_tp == curtab.get() {
                        last_status(false);
                        win_comp_pos();
                    }
                    flags = win_split_flags((*fconfig).split, parent.is_null())
                        | WSP_NOENTER as ::core::ffi::c_int;
                    parent_tp = if !parent.is_null() {
                        win_find_tabpage(parent)
                    } else {
                        curtab.get()
                    };
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
                    let need_switch: bool = !parent.is_null() && parent != curwin.get();
                    let mut switchwin: switchwin_T = switchwin_T {
                        sw_curwin: ::core::ptr::null_mut::<win_T>(),
                        sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
                        sw_same_win: false,
                        sw_visual_active: false,
                    };
                    if need_switch {
                        let result: ::core::ffi::c_int =
                            switch_win(&raw mut switchwin, parent, parent_tp, true);
                        debug_assert!(result == 1 as ::core::ffi::c_int, "result == OK");
                    }
                    to_split_ok = !win_split_ins(
                        0 as ::core::ffi::c_int,
                        flags,
                        win,
                        0 as ::core::ffi::c_int,
                        unflat_altfr,
                    )
                    .is_null();
                    if !to_split_ok {
                        win_append(
                            (*win).w_prev,
                            win,
                            if win_tp == curtab.get() {
                                ::core::ptr::null_mut::<tabpage_T>()
                            } else {
                                win_tp
                            },
                        );
                    }
                    if need_switch {
                        restore_win(&raw mut switchwin, true);
                    }
                    try_leave(&raw mut tstate, err);
                    if !to_split_ok {
                        if was_split {
                            winframe_restore(win, dir, unflat_altfr);
                        }
                        if !((*err).type_0 as ::core::ffi::c_int
                            != kErrorTypeNone as ::core::ffi::c_int)
                        {
                            api_set_error(
                                err,
                                kErrorTypeException,
                                c"Failed to move window %d into split".as_ptr(),
                                (*win).handle,
                            );
                        }
                    } else {
                        if win_tp != parent_tp && (*win_tp).tp_curwin == win {
                            (*win_tp).tp_curwin = altwin_0;
                        }
                        break '_resize;
                    }
                }
                if curwin_moving_tp as ::core::ffi::c_int != 0
                    && win_valid(win) as ::core::ffi::c_int != 0
                {
                    win_goto(win);
                }
                return false;
            }
        }
        if has_key(
            (*config).is_set__win_config_,
            KEYSET_OPTIDX_win_config__width,
        ) {
            win_setwidth_win((*fconfig).width, win);
        }
        if has_key(
            (*config).is_set__win_config_,
            KEYSET_OPTIDX_win_config__height,
        ) {
            win_setheight_win((*fconfig).height, win);
        }
        if !was_split {
            clear_float_config(fconfig, false);
        }
        merge_win_config(&raw mut (*win).w_config, *fconfig);
        return true;
    }
}

unsafe extern "C" fn win_config_float_tp(
    mut win: *mut win_T,
    mut config: *const KeyDict_win_config,
    mut fconfig: *const WinConfig,
    mut err: *mut Error,
) -> bool {
    unsafe {
        let mut win_tp: *mut tabpage_T = win_find_tabpage(win);
        let mut parent: *mut win_T = win;
        let mut parent_tp: *mut tabpage_T = win_tp;
        if has_key((*config).is_set__win_config_, KEYSET_OPTIDX_win_config__win) {
            parent = find_window_by_handle((*fconfig).window, err);
            if parent.is_null() {
                return false;
            }
            parent_tp = win_find_tabpage(parent);
        }
        let mut curwin_moving_tp: bool = false;
        let mut altwin: *mut win_T = ::core::ptr::null_mut::<win_T>();
        '_restore_curwin: {
            if win_tp != parent_tp {
                if !win_can_move_tp(win, win_tp, err) {
                    return false;
                }
                altwin = win_find_altwin(win, win_tp);
                debug_assert!(!altwin.is_null(), "altwin");
                if curwin.get() == win {
                    curwin_moving_tp = true;
                    win_goto(altwin);
                    if curwin.get() == win {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            c"Failed to switch away from window %d".as_ptr(),
                            (*win).handle,
                        );
                        return false;
                    }
                    win_tp = win_find_tabpage(win);
                    parent_tp = win_find_tabpage(parent);
                    if win_tp.is_null() || parent_tp.is_null() {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            c"Target windows were closed".as_ptr(),
                        );
                        break '_restore_curwin;
                    } else if win_tp != parent_tp && !win_can_move_tp(win, win_tp, err) {
                        break '_restore_curwin;
                    }
                    altwin = win_find_altwin(win, win_tp);
                    debug_assert!(!altwin.is_null(), "altwin");
                }
            }
            if !(*win).w_floating {
                if win_new_float(win, false, *fconfig, err).is_null() {
                    break '_restore_curwin;
                }
                redraw_later(win, UPD_NOT_VALID);
            }
            if win_tp != parent_tp {
                win_remove(
                    win,
                    if win_tp == curtab.get() {
                        ::core::ptr::null_mut::<tabpage_T>()
                    } else {
                        win_tp
                    },
                );
                let mut append_tp: *mut tabpage_T = if parent_tp == curtab.get() {
                    ::core::ptr::null_mut::<tabpage_T>()
                } else {
                    parent_tp
                };
                win_append(lastwin_nofloating(append_tp), win, append_tp);
                if win_tp != curtab.get() && (*win_tp).tp_curwin == win {
                    (*win_tp).tp_curwin = altwin;
                }
                ui_comp_remove_grid(&raw mut (*win).w_grid_alloc);
                redraw_later(win, UPD_NOT_VALID);
                set_must_redraw(UPD_NOT_VALID);
            }
            win_config_float(win, *fconfig);
            return true;
        }
        if curwin_moving_tp as ::core::ffi::c_int != 0 && win_valid(win) as ::core::ffi::c_int != 0
        {
            win_goto(win);
        }
        return false;
    }
}

pub unsafe extern "C" fn nvim_win_set_config(
    mut win: Window,
    mut config: *mut KeyDict_win_config,
    mut err: *mut Error,
) {
    unsafe {
        let mut w: *mut win_T = find_window_by_handle(win, err);
        if w.is_null() {
            return;
        }
        let mut was_split: bool = !(*w).w_floating;
        let mut has_split: bool = has_key(
            (*config).is_set__win_config_,
            KEYSET_OPTIDX_win_config__split,
        );
        let mut has_vertical: bool = has_key(
            (*config).is_set__win_config_,
            KEYSET_OPTIDX_win_config__vertical,
        );
        let mut old_style: WinStyle = (*w).w_config.style;
        let mut fconfig: WinConfig = (*w).w_config;
        let mut to_split: bool = (*config).relative.size == 0 as size_t
            && !(has_key(
                (*config).is_set__win_config_,
                KEYSET_OPTIDX_win_config__external,
            ) && (*config).external as ::core::ffi::c_int != 0)
            && (has_split as ::core::ffi::c_int != 0
                || has_vertical as ::core::ffi::c_int != 0
                || was_split as ::core::ffi::c_int != 0);
        if !parse_win_config(
            w,
            config,
            &raw mut fconfig,
            !was_split || to_split as ::core::ffi::c_int != 0,
            err,
        ) {
            return;
        }
        if to_split {
            if !win_config_split(w, config, &raw mut fconfig, err) {
                return;
            }
        } else if !win_config_float_tp(w, config, &raw mut fconfig, err) {
            return;
        }
        if fconfig.style as ::core::ffi::c_uint
            == kWinStyleMinimal as ::core::ffi::c_int as ::core::ffi::c_uint
            && old_style as ::core::ffi::c_uint != fconfig.style as ::core::ffi::c_uint
        {
            win_set_minimal_style(w);
            didset_window_options(w, true);
            changed_window_setting(w);
        }
        if fconfig._cmdline_offset < INT_MAX {
            cmdline_win.set(w);
        } else if w == cmdline_win.get() && fconfig._cmdline_offset == INT_MAX {
            cmdline_win.set(::core::ptr::null_mut::<win_T>());
        }
    }
}
