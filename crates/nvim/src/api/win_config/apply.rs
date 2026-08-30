//! `nvim_win_set_config()`: reconfiguring an existing window.
//!
//! The two directions a reconfiguration can take: `win_config_split` turns a
//! window into (or moves) a split, which may mean splitting a different parent,
//! changing the direction, or leaving the float layout entirely; and
//! `win_config_float_tp` applies a float config, including the tabpage move a
//! `relative` window may need.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported};
use crate::api_error;
use crate::winlayer::{TabPage, Win};

/// `NULL` for "the current tab page", which is how the window family spells
/// it throughout.
fn other_tab(tp: *mut tabpage_T) -> *mut tabpage_T {
    if tp == curtab.get() {
        ::core::ptr::null_mut::<tabpage_T>()
    } else {
        tp
    }
}

/// How many frames sit in `frame`'s row or column, `frame` included.
///
/// # Safety
/// `frame` must be a live frame.
unsafe fn sibling_count(frame: *mut frame_T) -> ::core::ffi::c_int {
    // SAFETY: the caller's frame, whose `fr_child`/`fr_next` links are live
    // frames or null.
    let first = unsafe { (*frame).fr_child };
    let mut n = 0;
    let mut fr = first;
    while !fr.is_null() {
        n += 1;
        // SAFETY: as above.
        fr = unsafe { (*fr).fr_next };
    }
    n
}

/// Apply the split half of `fconfig` to `win`: make it a split, move it to
/// another parent, or change which side of one it is on.
///
/// # Safety
/// `win` must be a live window, and `config`, `fconfig` and `err` must name
/// live objects for the whole call.
unsafe fn win_config_split(
    win: *mut win_T,
    config: CfgKeys,
    mut fconfig: WinCfg,
    err: ErrSlot,
) -> bool {
    let keys = config.is_set__win_config_;
    let set = |key| has_key(keys, key);
    // SAFETY: the caller's window.
    let was_split = !unsafe { (*win).w_floating };
    let has_split = set(KEYSET_OPTIDX_win_config__split);
    let has_vertical = set(KEYSET_OPTIDX_win_config__vertical);
    // SAFETY: the caller's window.
    let old_split = win_split_dir(unsafe { Win::new(win) });
    if has_vertical && !has_split {
        fconfig.split = if config.vertical {
            if old_split == kWinSplitRight || p_spr.get() != 0 {
                kWinSplitRight
            } else {
                kWinSplitLeft
            }
        } else if old_split == kWinSplitBelow || p_sb.get() != 0 {
            kWinSplitBelow
        } else {
            kWinSplitAbove
        };
    }
    // Nothing about the layout is changing when neither key was given, or
    // when the window is already a split on the same side of the same
    // parent; then only the size below is applied.
    let stays_put = !has_vertical && !has_split
        || was_split && !set(KEYSET_OPTIDX_win_config__win) && old_split == fconfig.split;
    '_resize: {
        if stays_put {
            break '_resize;
        }
        let mut parent: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut parent_tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
        if config.win == 0 {
            parent = curwin.get();
            parent_tp = curtab.get();
        } else if config.win > 0 {
            // SAFETY: `err` names the caller's error slot.
            parent = unsafe { find_window_by_handle(fconfig.window, slot_mut(err)) };
            if parent.is_null() {
                return false;
            }
            parent_tp = win_find_tabpage(parent);
        }
        let mut win_tp: *mut tabpage_T = win_find_tabpage(win);
        if !parent.is_null() {
            // SAFETY: `parent` is the live window found above.
            if unsafe { (*parent).w_floating } {
                err_msg(err, kErrorTypeException, c"Cannot split a floating window");
                return false;
            }
            // SAFETY: both windows are live, and `err` is the caller's slot.
            if win_tp != parent_tp && !unsafe { win_can_move_tp(win, win_tp, slot_mut(err)) } {
                return false;
            }
        }
        // SAFETY: the caller's window and error slot.
        if !unsafe { check_split_disallowed_err(win, slot_mut(err)) } {
            return false;
        }
        let mut to_split_ok = false;
        let curwin_moving_tp = win == curwin.get() && !parent.is_null() && win_tp != parent_tp;
        let mut dir: ::core::ffi::c_int = 0;
        let mut unflat_altfr: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
        let mut altwin_0: *mut win_T = ::core::ptr::null_mut::<win_T>();
        '_restore_curwin: {
            if curwin_moving_tp {
                // SAFETY: the caller's window, still in its tab page.
                let altwin = unsafe { win_find_altwin(win, win_tp) };
                debug_assert!(!altwin.is_null(), "altwin");
                // SAFETY: `altwin` is the live neighbour just found.
                unsafe { win_goto(altwin) };
                if curwin.get() == win {
                    // SAFETY: the caller's window.
                    let handle = unsafe { (*win).handle };
                    let why = api_error!(
                        kErrorTypeException,
                        "Failed to switch away from window {handle}"
                    );
                    store(err, why);
                    return false;
                }
                win_tp = win_find_tabpage(win);
                // `win_valid_any_tab` is the check for whether `parent` is
                // still there at all, so it takes the pointer.
                if win_tp.is_null() || !win_valid_any_tab(parent) {
                    err_msg(err, kErrorTypeException, c"Windows to split were closed");
                    break '_restore_curwin;
                }
                // SAFETY: both windows are live -- the check above says so.
                let changed = unsafe { was_split == (*win).w_floating || (*parent).w_floating };
                if changed {
                    let msg = c"Floating state of windows to split changed";
                    err_msg(err, kErrorTypeException, msg);
                    break '_restore_curwin;
                }
            }
            if was_split {
                // SAFETY: a non-floating window sits in a frame of the layout
                // tree.
                let frame = unsafe { (*win).w_frame };
                // SAFETY: as above.
                if unsafe { (*frame).fr_parent }.is_null() {
                    let msg = c"Cannot move last non-floating window";
                    err_msg(err, kErrorTypeException, msg);
                    break '_restore_curwin;
                }
                // SAFETY: both windows are live.
                let into_itself = !parent.is_null() && unsafe { (*parent).handle == (*win).handle };
                if into_itself {
                    // SAFETY: the frame's parent is live -- checked above.
                    let n_frames = unsafe { sibling_count((*frame).fr_parent) };
                    let mut neighbor: *mut win_T = ::core::ptr::null_mut::<win_T>();
                    if n_frames > 2 {
                        // SAFETY: as above.
                        let nested = !unsafe { (*(*frame).fr_parent).fr_parent }.is_null();
                        if nested {
                            let ahead =
                                fconfig.split == kWinSplitAbove || fconfig.split == kWinSplitLeft;
                            // SAFETY: the caller's window.
                            let live = unsafe { Win::new(win) };
                            neighbor = raw_win(if ahead { live.next() } else { live.prev() });
                        }
                        // SAFETY: the caller's window, and `dir`/`unflat_altfr`
                        // are this frame's own.
                        altwin_0 = unsafe {
                            winframe_remove(
                                win,
                                &raw mut dir,
                                other_tab(win_tp),
                                &raw mut unflat_altfr,
                            )
                        };
                    } else if n_frames == 2 {
                        // SAFETY: as above.
                        altwin_0 = unsafe {
                            winframe_remove(
                                win,
                                &raw mut dir,
                                other_tab(win_tp),
                                &raw mut unflat_altfr,
                            )
                        };
                        neighbor = altwin_0;
                    } else {
                        let msg = c"Cannot split window into itself";
                        err_msg(err, kErrorTypeException, msg);
                        break '_restore_curwin;
                    }
                    parent = neighbor;
                } else {
                    // SAFETY: as above.
                    altwin_0 = unsafe {
                        winframe_remove(win, &raw mut dir, other_tab(win_tp), &raw mut unflat_altfr)
                    };
                }
            } else {
                // SAFETY: the caller's window and its tab page.
                altwin_0 = unsafe {
                    let at = (win_tp != curtab.get()).then(|| TabPage::new(win_tp));
                    win_float_find_altwin(win, at).map_or(::core::ptr::null_mut(), Win::raw)
                };
            }
            // SAFETY: the caller's window, taken out of `win_tp`'s list.
            unsafe { win_remove(win, other_tab(win_tp)) };
            if win_tp == curtab.get() {
                last_status(false);
                win_comp_pos();
            }
            let flags = win_split_flags(fconfig.split, parent.is_null())
                | WSP_NOENTER as ::core::ffi::c_int;
            parent_tp = if parent.is_null() {
                curtab.get()
            } else {
                win_find_tabpage(parent)
            };
            let mut tstate = TryState::default();
            // SAFETY: `tstate` is this frame's own, live until `try_leave`.
            unsafe { try_enter(&raw mut tstate) };
            let need_switch: bool = !parent.is_null() && parent != curwin.get();
            let mut switchwin = switchwin_T {
                sw_curwin: ::core::ptr::null_mut::<win_T>(),
                sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
                sw_same_win: false,
                sw_visual_active: false,
            };
            if need_switch {
                // SAFETY: `switchwin` is this frame's own, and `parent`/
                // `parent_tp` are the live window and tab page to split in.
                let result = unsafe { switch_win(&raw mut switchwin, parent, parent_tp, true) };
                debug_assert!(result == 1 as ::core::ffi::c_int, "result == OK");
            }
            // SAFETY: the caller's window, and `unflat_altfr` the frame the
            // removal above left behind.
            to_split_ok = !unsafe {
                win_split_ins(
                    0 as ::core::ffi::c_int,
                    flags,
                    win,
                    0 as ::core::ffi::c_int,
                    unflat_altfr,
                )
            }
            .is_null();
            if !to_split_ok {
                // SAFETY: the caller's window, put back where it was.
                unsafe {
                    let prev = raw_win(Win::new(win).prev());
                    win_append(prev, win, other_tab(win_tp));
                }
            }
            if need_switch {
                // SAFETY: the matching restore of the switch above.
                unsafe { restore_win(&raw mut switchwin, true) };
            }
            // SAFETY: `tstate` is what the `try_enter` above filled in, and
            // `err` is the caller's slot.
            unsafe { try_leave(&raw mut tstate, slot_mut(err)) };
            if to_split_ok {
                // SAFETY: `win_tp` is a live tab page.
                let stale = win_tp != parent_tp && unsafe { (*win_tp).tp_curwin } == win;
                if stale {
                    // SAFETY: as above.
                    unsafe { (*win_tp).tp_curwin = altwin_0 };
                }
                break '_resize;
            }
            if was_split {
                // SAFETY: the caller's window and the frame the removal left.
                unsafe { winframe_restore(win, dir, unflat_altfr) };
            }
            if !err.is_set() {
                // SAFETY: the caller's window.
                let handle = unsafe { (*win).handle };
                let why = api_error!(
                    kErrorTypeException,
                    "Failed to move window {handle} into split"
                );
                store(err, why);
            }
        }
        if curwin_moving_tp && win_valid(win) {
            // SAFETY: the caller's window, still valid -- just checked.
            unsafe { win_goto(win) };
        }
        return false;
    }
    if set(KEYSET_OPTIDX_win_config__width) {
        // SAFETY: the caller's window.
        unsafe { win_setwidth_win(fconfig.width, win) };
    }
    if set(KEYSET_OPTIDX_win_config__height) {
        // SAFETY: as above.
        unsafe { win_setheight_win(fconfig.height, win) };
    }
    if !was_split {
        // SAFETY: the caller's config.
        unsafe { clear_float_config(fconfig.raw(), false) };
    }
    let merged = (*fconfig).clone();
    // SAFETY: the caller's window, whose config field is live with it.
    unsafe { merge_win_config(&raw mut (*win).w_config, merged) };
    true
}

/// Apply the float half of `fconfig` to `win`, including the move to another
/// tab page a `win` key may ask for.
///
/// # Safety
/// `win` must be a live window, and `config`, `fconfig` and `err` must name
/// live objects for the whole call.
unsafe fn win_config_float_tp(
    win: *mut win_T,
    config: CfgKeys,
    fconfig: WinCfg,
    err: ErrSlot,
) -> bool {
    let mut win_tp: *mut tabpage_T = win_find_tabpage(win);
    let mut parent: *mut win_T = win;
    let mut parent_tp: *mut tabpage_T = win_tp;
    if has_key(config.is_set__win_config_, KEYSET_OPTIDX_win_config__win) {
        // SAFETY: `err` names the caller's error slot.
        parent = unsafe { find_window_by_handle(fconfig.window, slot_mut(err)) };
        if parent.is_null() {
            return false;
        }
        parent_tp = win_find_tabpage(parent);
    }
    let mut curwin_moving_tp = false;
    let mut altwin: *mut win_T = ::core::ptr::null_mut::<win_T>();
    '_restore_curwin: {
        if win_tp != parent_tp {
            // SAFETY: the caller's window and error slot.
            if !unsafe { win_can_move_tp(win, win_tp, slot_mut(err)) } {
                return false;
            }
            // SAFETY: the caller's window, still in its tab page.
            altwin = unsafe { win_find_altwin(win, win_tp) };
            debug_assert!(!altwin.is_null(), "altwin");
            if curwin.get() == win {
                curwin_moving_tp = true;
                // SAFETY: `altwin` is the live neighbour just found.
                unsafe { win_goto(altwin) };
                if curwin.get() == win {
                    // SAFETY: the caller's window.
                    let handle = unsafe { (*win).handle };
                    let why = api_error!(
                        kErrorTypeException,
                        "Failed to switch away from window {handle}"
                    );
                    store(err, why);
                    return false;
                }
                win_tp = win_find_tabpage(win);
                parent_tp = win_find_tabpage(parent);
                if win_tp.is_null() || parent_tp.is_null() {
                    err_msg(err, kErrorTypeException, c"Target windows were closed");
                    break '_restore_curwin;
                }
                // SAFETY: as above.
                if win_tp != parent_tp && !unsafe { win_can_move_tp(win, win_tp, slot_mut(err)) } {
                    break '_restore_curwin;
                }
                // SAFETY: as above.
                altwin = unsafe { win_find_altwin(win, win_tp) };
                debug_assert!(!altwin.is_null(), "altwin");
            }
        }
        // SAFETY: the caller's window.
        if !unsafe { (*win).w_floating } {
            let config = (*fconfig).clone();
            // SAFETY: the caller's window and error slot.
            if unsafe { win_new_float(win, false, config, slot_mut(err)) }.is_null() {
                break '_restore_curwin;
            }
            // SAFETY: as above.
            unsafe { redraw_later(win, UPD_NOT_VALID) };
        }
        if win_tp != parent_tp {
            let append_tp = other_tab(parent_tp);
            // SAFETY: the caller's window, moved from one tab page's list to
            // the other's.
            unsafe {
                win_remove(win, other_tab(win_tp));
                win_append(lastwin_nofloating(append_tp), win, append_tp);
            }
            // SAFETY: `win_tp` is a live tab page.
            let stale = win_tp != curtab.get() && unsafe { (*win_tp).tp_curwin } == win;
            if stale {
                // SAFETY: as above.
                unsafe { (*win_tp).tp_curwin = altwin };
            }
            // SAFETY: the window's own grid, which is live with it.
            unsafe {
                ui_comp_remove_grid(&raw mut (*win).w_grid_alloc);
                redraw_later(win, UPD_NOT_VALID);
            }
            set_must_redraw(UPD_NOT_VALID);
        }
        let config = (*fconfig).clone();
        // SAFETY: the caller's window.
        win_config_float(unsafe { Win::new(win) }, config);
        return true;
    }
    if curwin_moving_tp && win_valid(win) {
        // SAFETY: the caller's window, still valid -- just checked.
        unsafe { win_goto(win) };
    }
    false
}

/// Reconfigure `win` from the `config` dictionary.
///
/// # Safety
/// `config` must be the caller's decoded keyset -- NUL-terminated strings and
/// arrays that name their own items.
pub unsafe fn nvim_win_set_config(
    win: Window,
    config: *mut KeyDict_win_config,
) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    // SAFETY: `error` is this frame's own slot, live for the whole call, and
    // `config` is the caller's keyset.
    let (report, keys) = unsafe { (ErrSlot::new(&mut error), CfgKeys::new(config)) };
    // SAFETY: `error` is this frame's slot; the lookup answers a live window or
    // a null.
    let w = unsafe { find_window_by_handle(win, &mut error) };
    if w.is_null() {
        return ().reported(error);
    }
    // SAFETY: `w` is the live window the lookup answered.
    let live = unsafe { Win::new(w) };
    let was_split = !live.w_floating;
    let key_set = keys.is_set__win_config_;
    let has_split = has_key(key_set, KEYSET_OPTIDX_win_config__split);
    let has_vertical = has_key(key_set, KEYSET_OPTIDX_win_config__vertical);
    let old_style = live.w_config.style;
    let mut fconfig = live.w_config.clone();
    let external = has_key(key_set, KEYSET_OPTIDX_win_config__external) && keys.external;
    let to_split =
        keys.relative.is_empty() && !external && (has_split || has_vertical || was_split);
    // SAFETY: `fconfig` is this frame's own, and `keys` the caller's keyset.
    let parsed = unsafe {
        parse_win_config(
            Some(live),
            keys,
            WinCfg::new(&raw mut fconfig),
            !was_split || to_split,
            report,
        )
    };
    if !parsed {
        return ().reported(error);
    }
    // SAFETY: `w` is live, `fconfig` this frame's own, and `keys`/`report`
    // the caller's.
    let applied = unsafe {
        let fc = WinCfg::new(&raw mut fconfig);
        if to_split {
            win_config_split(w, keys, fc, report)
        } else {
            win_config_float_tp(w, keys, fc, report)
        }
    };
    if !applied {
        return ().reported(error);
    }
    if fconfig.style == kWinStyleMinimal && old_style != fconfig.style {
        // SAFETY: `w` is live.
        win_set_minimal_style(unsafe { Win::new(w) });
        // SAFETY: as above.
        unsafe { didset_window_options(w, true) };
        // SAFETY: as above.
        changed_window_setting(unsafe { Win::new(w) });
    }
    if fconfig._cmdline_offset < INT_MAX {
        cmdline_win.set(w);
    } else if w == cmdline_win.get() && fconfig._cmdline_offset == INT_MAX {
        cmdline_win.set(::core::ptr::null_mut::<win_T>());
    }
    ().reported(error)
}

/// `Win::raw`, or a null for "no neighbour" — the shape the transpiled
/// window family still takes.
fn raw_win(wp: Option<Win>) -> *mut win_T {
    wp.map_or(::core::ptr::null_mut(), Win::raw)
}
