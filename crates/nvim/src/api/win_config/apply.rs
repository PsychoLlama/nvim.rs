//! `nvim_win_set_config()`: reconfiguring an existing window.
//!
//! The two directions a reconfiguration can take: `win_config_split` turns a
//! window into (or moves) a split, which may mean splitting a different parent,
//! changing the direction, or leaving the float layout entirely; and
//! `win_config_float_tp` applies a float config, including the tabpage move a
//! `relative` window may need.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::Reported;
use crate::api_error;
use crate::winlayer::{TabPage, Win};

/// `None` for "the current tab page", which is how the window family spells
/// it throughout.
fn other_tab(tabpage: TabPage) -> Option<TabPage> {
    (tabpage != TabPage::current()).then_some(tabpage)
}

/// The tab page a window was just found on. [`win_find_tabpage`] answers an
/// `Option` because it is asked about windows that may be gone; here it is
/// asked about a window the caller promised is live.
fn expect_tab(tabpage: Option<TabPage>) -> TabPage {
    tabpage.expect("a live window is on a tab page")
}

/// How many frames sit in `frame`'s row or column, `frame` included.
///
/// # Safety
/// `frame` must be a live frame.
unsafe fn sibling_count(frame: *mut Frame) -> ::core::ffi::c_int {
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
    mut win: Win,
    config: CfgKeys,
    mut fconfig: WinCfg,
    err: ErrSlot,
) -> bool {
    // SAFETY: the caller's window, live for the whole call.
    let w = win;
    let keys = config.is_set__win_config_;
    let set = |key| has_key(keys, key);
    // SAFETY: the caller's window.
    let was_split = !win.w_floating;
    let has_split = set(KEYSET_OPTIDX_win_config__split);
    let has_vertical = set(KEYSET_OPTIDX_win_config__vertical);
    let old_split = win_split_dir(w);
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
        let mut parent: Option<Win> = None;
        let mut parent_tp: Option<TabPage> = None;
        if config.win == 0 {
            parent = Some(Win::current());
            parent_tp = Some(TabPage::current());
        } else if config.win > 0 {
            // SAFETY: `err` names the caller's error slot.
            let Some(found) = find_window_by_handle(fconfig.window, slot_mut(err)) else {
                return false;
            };
            parent = Some(found);
            parent_tp = win_find_tabpage(found.id());
        }
        // Both identities, taken while both windows are live: `win_goto`
        // below fires autocommands that can close either, and every question
        // after it is about exactly that.
        let (win_id, mut parent_id) = (win.id(), parent.map(Win::id));
        let mut win_tp = win_find_tabpage(win_id);
        if let Some(p) = parent {
            if p.w_floating {
                err_msg(err, kErrorTypeException, c"Cannot split a floating window");
                return false;
            }
            // SAFETY: `err` is the caller's slot.
            if win_tp != parent_tp
                && !unsafe { win_can_move_tp(win, expect_tab(win_tp), slot_mut(err)) }
            {
                return false;
            }
        }
        // SAFETY: the caller's window and error slot.
        if !unsafe { check_split_disallowed_err(win, slot_mut(err)) } {
            return false;
        }
        let to_split_ok;
        let curwin_moving_tp = win == Win::current() && parent.is_some() && win_tp != parent_tp;
        let mut dir: ::core::ffi::c_int = 0;
        let mut unflat_altfr: *mut Frame = ::core::ptr::null_mut::<Frame>();
        let altwin_0: Option<Win>;
        '_restore_curwin: {
            if curwin_moving_tp {
                // SAFETY: the caller's window, still in its tab page.
                let altwin = unsafe { win_find_altwin(win, expect_tab(win_tp)) }.expect("altwin");
                // SAFETY: `altwin` is the live neighbour just found.
                unsafe { win_goto(altwin) };
                if Win::current_raw() == win.raw() {
                    let handle = win_id.handle();
                    let why = api_error!(
                        kErrorTypeException,
                        "Failed to switch away from window {handle}"
                    );
                    store(err, why);
                    return false;
                }
                win_tp = win_find_tabpage(win_id);
                // `win_valid_any_tab` is the check for whether `parent` is
                // still there at all.
                let live_parent = parent.filter(|_| parent_id.is_some_and(win_valid_any_tab));
                let (Some(_), Some(p)) = (win_tp, live_parent) else {
                    err_msg(err, kErrorTypeException, c"Windows to split were closed");
                    break '_restore_curwin;
                };
                let changed = was_split == win.w_floating || p.w_floating;
                if changed {
                    let msg = c"Floating state of windows to split changed";
                    err_msg(err, kErrorTypeException, msg);
                    break '_restore_curwin;
                }
            }
            if was_split {
                // SAFETY: a non-floating window sits in a frame of the layout
                // tree.
                let frame = win.w_frame;
                // SAFETY: as above.
                if unsafe { (*frame).fr_parent }.is_null() {
                    let msg = c"Cannot move last non-floating window";
                    err_msg(err, kErrorTypeException, msg);
                    break '_restore_curwin;
                }
                // SAFETY: both windows are live.
                let into_itself = parent.is_some_and(|p| p.handle == win.handle);
                if into_itself {
                    // SAFETY: the frame's parent is live -- checked above.
                    let n_frames = unsafe { sibling_count((*frame).fr_parent) };
                    let mut neighbor: Option<Win> = None;
                    if n_frames > 2 {
                        // SAFETY: as above.
                        let nested = !unsafe { (*(*frame).fr_parent).fr_parent }.is_null();
                        let win_tp = expect_tab(win_tp);
                        if nested {
                            let ahead =
                                fconfig.split == kWinSplitAbove || fconfig.split == kWinSplitLeft;
                            let live = w;
                            neighbor = if ahead { live.next() } else { live.prev() };
                        }
                        // SAFETY: the caller's window, and `dir`/`unflat_altfr`
                        // are this frame's own.
                        altwin_0 = unsafe {
                            winframe_remove(
                                w,
                                &raw mut dir,
                                other_tab(win_tp),
                                &raw mut unflat_altfr,
                            )
                        };
                    } else if n_frames == 2 {
                        let win_tp = expect_tab(win_tp);
                        // SAFETY: as above.
                        altwin_0 = unsafe {
                            winframe_remove(
                                w,
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
                    // `parent` is replaced here, so the identity it stands
                    // for is too -- the neighbour is live, just found.
                    parent = neighbor;
                    parent_id = neighbor.map(Win::id);
                } else {
                    let win_tp = expect_tab(win_tp);
                    // SAFETY: `dir` and `unflat_altfr` are this frame's own.
                    altwin_0 = unsafe {
                        winframe_remove(w, &raw mut dir, other_tab(win_tp), &raw mut unflat_altfr)
                    };
                }
            } else {
                // SAFETY: the caller's window and its tab page.
                altwin_0 = unsafe { win_float_find_altwin(win, other_tab(expect_tab(win_tp))) };
            }
            win_remove(w, other_tab(expect_tab(win_tp)));
            if win_tp == Some(TabPage::current()) {
                last_status(false);
                win_comp_pos();
            }
            let flags = win_split_flags(fconfig.split, parent.is_none())
                | WSP_NOENTER as ::core::ffi::c_int;
            parent_tp = match parent_id {
                None => Some(TabPage::current()),
                Some(p) => win_find_tabpage(p),
            };
            let mut tstate = TryState::default();
            // SAFETY: `tstate` is this frame's own, live until `try_leave`.
            unsafe { try_enter(&raw mut tstate) };
            let need_switch: bool = parent.is_some_and(|p| !p.is_current());
            let mut switchwin = SwitchWin {
                sw_curwin: None,
                sw_curtab: None,
                sw_same_win: false,
                sw_visual_active: false,
            };
            if need_switch {
                let parent_tp = expect_tab(parent_tp);
                let parent = parent.expect("`need_switch` says there is a parent");
                // SAFETY: `switchwin` is this frame's own, and `parent`/
                // `parent_tp` are the live window and tab page to split in.
                let result =
                    unsafe { switch_win(&raw mut switchwin, parent, Some(parent_tp), true) };
                debug_assert!(result.is_ok(), "the window was switched to");
            }
            // SAFETY: the caller's window, and `unflat_altfr` the frame the
            // removal above left behind.
            to_split_ok = unsafe {
                win_split_ins(
                    0 as ::core::ffi::c_int,
                    flags,
                    Some(win),
                    0 as ::core::ffi::c_int,
                    unflat_altfr,
                )
            }
            .is_some();
            if !to_split_ok {
                win_append(w.prev(), w, other_tab(expect_tab(win_tp)));
            }
            if need_switch {
                // SAFETY: the matching restore of the switch above.
                unsafe { restore_win(&raw mut switchwin, true) };
            }
            // SAFETY: `tstate` is what the `try_enter` above filled in, and
            // `err` is the caller's slot.
            unsafe { try_leave(&raw mut tstate, slot_mut(err)) };
            if to_split_ok {
                let mut tp = expect_tab(win_tp);
                if win_tp != parent_tp && tp.tp_curwin == Some(win_id) {
                    tp.tp_curwin = altwin_0.map(Win::id);
                }
                break '_resize;
            }
            if was_split {
                // SAFETY: the caller's window and the frame the removal left.
                unsafe { winframe_restore(w, dir, unflat_altfr) };
            }
            if !err.is_set() {
                // SAFETY: the caller's window.
                let handle = win_id.handle();
                let why = api_error!(
                    kErrorTypeException,
                    "Failed to move window {handle} into split"
                );
                store(err, why);
            }
        }
        if curwin_moving_tp && win_valid(win_id) {
            // SAFETY: the caller's window, still valid -- just checked.
            unsafe { win_goto(w) };
        }
        return false;
    }
    if set(KEYSET_OPTIDX_win_config__width) {
        // SAFETY: the caller's window.
        unsafe { win_setwidth_win(fconfig.width, w) };
    }
    if set(KEYSET_OPTIDX_win_config__height) {
        // SAFETY: as above.
        unsafe { win_setheight_win(fconfig.height, w) };
    }
    if !was_split {
        // SAFETY: the caller's config.
        unsafe { clear_float_config(fconfig.raw(), false) };
    }
    let merged = (*fconfig).clone();
    // SAFETY: the caller's window, whose config field is live with it.
    unsafe { merge_win_config(&raw mut win.w_config, merged) };
    true
}

/// Apply the float half of `fconfig` to `win`, including the move to another
/// tab page a `win` key may ask for.
///
/// # Safety
/// `win` must be a live window, and `config`, `fconfig` and `err` must name
/// live objects for the whole call.
unsafe fn win_config_float_tp(
    mut win: Win,
    config: CfgKeys,
    fconfig: WinCfg,
    err: ErrSlot,
) -> bool {
    // SAFETY: the caller's window, live for the whole call.
    let w = win;
    let mut win_tp = win_find_tabpage(win.id());
    let mut parent_id = win.id();
    let mut parent_tp = win_tp;
    if has_key(config.is_set__win_config_, KEYSET_OPTIDX_win_config__win) {
        // SAFETY: `err` names the caller's error slot.
        let Some(found) = find_window_by_handle(fconfig.window, slot_mut(err)) else {
            return false;
        };
        parent_id = found.id();
        parent_tp = win_find_tabpage(parent_id);
    }
    let mut curwin_moving_tp = false;
    let mut altwin: Option<Win> = None;
    '_restore_curwin: {
        if win_tp != parent_tp {
            // SAFETY: the caller's window and error slot.
            if !unsafe { win_can_move_tp(win, expect_tab(win_tp), slot_mut(err)) } {
                return false;
            }
            // SAFETY: the caller's window, still in its tab page.
            altwin = unsafe { win_find_altwin(win, expect_tab(win_tp)) };
            debug_assert!(altwin.is_some(), "altwin");
            if win.is_current() {
                curwin_moving_tp = true;
                // SAFETY: `altwin` is the live neighbour just found.
                unsafe { win_goto(altwin.expect("altwin")) };
                if win.is_current() {
                    let handle = win.id().handle();
                    let why = api_error!(
                        kErrorTypeException,
                        "Failed to switch away from window {handle}"
                    );
                    store(err, why);
                    return false;
                }
                win_tp = win_find_tabpage(win.id());
                parent_tp = win_find_tabpage(parent_id);
                if win_tp.is_none() || parent_tp.is_none() {
                    err_msg(err, kErrorTypeException, c"Target windows were closed");
                    break '_restore_curwin;
                }
                // SAFETY: as above.
                if win_tp != parent_tp
                    && !unsafe { win_can_move_tp(win, expect_tab(win_tp), slot_mut(err)) }
                {
                    break '_restore_curwin;
                }
                // SAFETY: as above.
                altwin = unsafe { win_find_altwin(win, expect_tab(win_tp)) };
                debug_assert!(altwin.is_some(), "altwin");
            }
        }
        if !win.w_floating {
            let config = (*fconfig).clone();
            if win_new_float(Some(win), false, config, slot_mut(err)).is_none() {
                break '_restore_curwin;
            }
            // SAFETY: as above.
            redraw_later(win, UPD_NOT_VALID);
        }
        if win_tp != parent_tp {
            let append_tp = other_tab(expect_tab(parent_tp));
            // The caller's window, moved from one tab page's list to the
            // other's.
            win_remove(w, other_tab(expect_tab(win_tp)));
            win_append(Some(lastwin_nofloating(append_tp)), w, append_tp);
            let mut tp = expect_tab(win_tp);
            if !tp.is_current() && tp.tp_curwin == Some(win.id()) {
                tp.tp_curwin = altwin.map(Win::id);
            }
            // SAFETY: the window's own grid, which is live with it.
            unsafe {
                ui_comp_remove_grid(&raw mut win.w_grid_alloc);
                redraw_later(win, UPD_NOT_VALID);
            }
            set_must_redraw(UPD_NOT_VALID);
        }
        let config = (*fconfig).clone();
        win_config_float(w, config);
        return true;
    }
    if curwin_moving_tp && win_valid(win.id()) {
        // SAFETY: the caller's window, still valid -- just checked.
        unsafe { win_goto(w) };
    }
    false
}

/// Reconfigure `win` from the `config` dictionary.
///
/// # Safety
/// `config` must be the caller's decoded keyset -- NUL-terminated strings and
/// arrays that name their own items.
pub unsafe fn nvim_win_set_config(
    win: WindowHandle,
    config: *mut KeyDict_win_config,
) -> Result<(), Error> {
    let mut error = Error::none();
    // SAFETY: `error` is this frame's own slot, live for the whole call, and
    // `config` is the caller's keyset.
    let (report, keys) = unsafe { (ErrSlot::new(&mut error), CfgKeys::new(config)) };
    // SAFETY: `error` is this frame's slot; the lookup answers a live window or
    // a null.
    let Some(w) = find_window_by_handle(win, &mut error) else {
        return ().reported(error);
    };
    // SAFETY: `w` is the live window the lookup answered.
    let live = w;
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
        win_set_minimal_style(w);
        // SAFETY: as above.
        unsafe { didset_window_options(w, true) };
        changed_window_setting(w);
    }
    if fconfig._cmdline_offset < INT_MAX {
        cmdline_win.set(Some(w.id()));
    } else if cmdline_win.get() == Some(w.id()) && fconfig._cmdline_offset == INT_MAX {
        cmdline_win.set(None);
    }
    ().reported(error)
}
