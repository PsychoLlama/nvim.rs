//! `nvim_open_win()`: creating a window from a config.
//!
//! The entry point for both kinds of window the config describes: a float,
//! which is created directly, and a split, which goes through `win_split_dir`
//! and `win_split_flags` to turn the `split`/`vertical` keys into the
//! `WSP_*` flags `win_split_ins` takes.  `win_can_move_tp` and
//! `win_find_altwin` are the checks a window has to pass before it can be
//! moved to another tabpage.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported};
use crate::buffer::BufRef;
use crate::winfloat::WIN_CONFIG_INIT;
use crate::winlayer::Buf;
use crate::winlayer::{TabPage, Win};

/// Create a window showing `buf` from the `config` dictionary: a float, a
/// split, or an external window.
///
/// # Safety
/// `config` must be the caller's decoded keyset -- NUL-terminated strings and
/// arrays that name their own items.
pub unsafe fn nvim_open_win(
    buf: Buffer,
    enter: Boolean,
    config: *mut KeyDict_win_config,
) -> Result<Window, Error> {
    let mut error = ERROR_INIT;
    // SAFETY: `error` is this frame's own slot, live for the whole call, and
    // `config` is the caller's keyset.
    let (report, keys) = unsafe { (ErrSlot::new(&mut error), CfgKeys::new(config)) };
    let mut bufref = BufRef::NONE;
    // SAFETY: `error` is this frame's slot; the lookup answers a live buffer or
    // a null.
    let b = unsafe { find_buffer_by_handle(buf, &mut error) };
    if b.is_null() {
        return (0 as Window).reported(error);
    }
    if cmdwin_type.get() != 0 && enter || b == cmdwin_buf.get() {
        // SAFETY: `e_cmdwin` is a static NUL-terminated message.
        unsafe { err_msg_raw(report, kErrorTypeException, e_cmdwin.as_ptr()) };
        return (0 as Window).reported(error);
    }
    let mut fconfig = WIN_CONFIG_INIT;
    // SAFETY: `fconfig` is this frame's own and `keys` the caller's keyset.
    let parsed =
        unsafe { parse_win_config(None, keys, WinCfg::new(&raw mut fconfig), false, report) };
    if !parsed {
        return (0 as Window).reported(error);
    }
    let keys_set = keys.is_set__win_config_;
    let is_split = has_key(keys_set, KEYSET_OPTIDX_win_config__split)
        || has_key(keys_set, KEYSET_OPTIDX_win_config__vertical);
    let mut rv: Window = 0;
    // Read before the config is handed to the window: whichever branch
    // below runs moves it, and all three are wanted afterwards.
    let noautocmd = fconfig.noautocmd;
    let style = fconfig.style;
    let cmdline_offset = fconfig._cmdline_offset;
    if noautocmd {
        // SAFETY: paired with the `unblock_autocmds` at the end.
        unsafe { block_autocmds() };
    }
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut tp: *mut tabpage_T = curtab.get();
    debug_assert!(!curwin.get().is_null(), "curwin != NULL");
    let mut parent: *mut win_T = if keys.win == 0 {
        curwin.get()
    } else {
        ::core::ptr::null_mut::<win_T>()
    };
    '_cleanup: {
        if keys.win > 0 {
            // SAFETY: `error` is this frame's slot.
            parent = unsafe { find_window_by_handle(fconfig.window, &mut error) };
            if parent.is_null() {
                break '_cleanup;
            }
            // SAFETY: `parent` is the live window the lookup answered.
            if is_split && unsafe { (*parent).w_floating } {
                err_msg(
                    report,
                    kErrorTypeException,
                    c"Cannot split a floating window",
                );
                break '_cleanup;
            }
            tp = win_find_tabpage(parent);
        }
        if is_split {
            let target = if parent.is_null() {
                curwin.get()
            } else {
                parent
            };
            // SAFETY: `target` is a live window and `error` this frame's slot.
            if !unsafe { check_split_disallowed_err(target, &mut error) } {
                break '_cleanup;
            }
            // `vertical` without `split` picks the side from 'splitright' and
            // 'splitbelow'.
            if has_key(keys_set, KEYSET_OPTIDX_win_config__vertical)
                && !has_key(keys_set, KEYSET_OPTIDX_win_config__split)
            {
                fconfig.split = if keys.vertical {
                    if p_spr.get() != 0 {
                        kWinSplitRight
                    } else {
                        kWinSplitLeft
                    }
                } else if p_sb.get() != 0 {
                    kWinSplitBelow
                } else {
                    kWinSplitAbove
                };
            }
            let flags = win_split_flags(fconfig.split, parent.is_null())
                | WSP_NOENTER as ::core::ffi::c_int;
            let vertical = flags & WSP_VERT as ::core::ffi::c_int != 0;
            let size = if vertical {
                fconfig.width
            } else {
                fconfig.height
            };
            let mut tstate = TryState::default();
            // SAFETY: `tstate` is this frame's own, live until `try_leave`.
            unsafe { try_enter(&raw mut tstate) };
            if parent.is_null() || parent == curwin.get() {
                // SAFETY: a split of the current window, which is live.
                wp = unsafe { split_ins(size, flags) };
            } else {
                let mut switchwin = switchwin_T {
                    sw_curwin: ::core::ptr::null_mut::<win_T>(),
                    sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
                    sw_same_win: false,
                    sw_visual_active: false,
                };
                // SAFETY: `switchwin` is this frame's own, and `parent`/`tp`
                // are the live window and tab page to split in.
                let result = unsafe { switch_win(&raw mut switchwin, parent, tp, true) };
                debug_assert!(result.is_ok(), "the window was switched to");
                // SAFETY: the window switched to is live.
                wp = unsafe { split_ins(size, flags) };
                // SAFETY: the matching restore of the switch above.
                unsafe { restore_win(&raw mut switchwin, true) };
            }
            // SAFETY: `tstate` is what the `try_enter` above filled in, and
            // `error` is this frame's slot.
            unsafe { try_leave(&raw mut tstate, &mut error) };
            if !wp.is_null() {
                // SAFETY: `wp` is the window the split just made.
                let (width, height) = unsafe {
                    (*wp).w_config = fconfig;
                    ((*wp).w_width, (*wp).w_height)
                };
                if size > 0 {
                    if vertical && width != size {
                        // SAFETY: `wp` is live.
                        unsafe { win_setwidth_win(size, wp) };
                    } else if !vertical && height != size {
                        // SAFETY: `wp` is live.
                        unsafe { win_setheight_win(size, wp) };
                    }
                }
            }
        } else {
            // SAFETY: `curwin` is live for the editor's whole run, and so is
            // the buffer it shows.
            let locked = unsafe { (*(*curwin.get()).w_buffer).b_locked_split } != 0;
            if locked {
                let msg = c"E1159: Cannot open a float when closing the buffer";
                err_msg(report, kErrorTypeException, msg);
                break '_cleanup;
            }
            // SAFETY: `error` is this frame's slot.
            let (none, slot) = (::core::ptr::null_mut::<win_T>(), &mut error);
            wp = unsafe { win_new_float(none, false, fconfig, slot) };
        }
        if wp.is_null() {
            if !report.is_set() {
                err_msg(report, kErrorTypeException, c"Failed to create window");
            }
            break '_cleanup;
        }
        if cmdline_offset < INT_MAX {
            cmdline_win.set(wp);
        }
        // SAFETY: `b` is the live buffer found above.
        bufref = BufRef::of_opt(unsafe { Buf::from_raw(b) });
        if !noautocmd {
            let mut switchwin_0 = switchwin_T {
                sw_curwin: ::core::ptr::null_mut::<win_T>(),
                sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
                sw_same_win: false,
                sw_visual_active: false,
            };
            // SAFETY: `switchwin_0` is this frame's own, and `wp`/`tp` name
            // the window just made and its tab page.
            let result_0 = unsafe { switch_win_noblock(&raw mut switchwin_0, wp, tp, true) };
            debug_assert!(result_0.is_ok(), "the window was switched to");
            // SAFETY: an autocommand with neither a file name nor a pattern.
            let switched = unsafe {
                apply_autocmds(
                    EVENT_WINNEW,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    false,
                    curbuf.get(),
                )
            };
            if switched {
                tp = win_find_tabpage(wp);
            }
            // SAFETY: the matching restore of the switch above.
            unsafe { restore_win_noblock(&raw mut switchwin_0, true) };
        }
        if !tp.is_null() && enter {
            // SAFETY: `tp` still holds `wp`, so both are live.
            unsafe { goto_tabpage_win(tp, wp) };
            tp = win_find_tabpage(wp);
        }
        // SAFETY: `wp` is read only once its tab page still holds it, which
        // is what says the autocommands above did not close it.
        let other_buf = !tp.is_null() && bufref.valid() && b != unsafe { (*wp).w_buffer };
        if other_buf {
            let au_no_enter_leave: bool = curwin.get() != wp && !noautocmd;
            if au_no_enter_leave {
                autocmd_no_enter.set(autocmd_no_enter.get() + 1);
                autocmd_no_leave.set(autocmd_no_leave.get() + 1);
            }
            // SAFETY: `wp` and `b` are live, and `error` is this frame's slot.
            unsafe { win_set_buf(wp, b, &mut error) };
            if !noautocmd {
                tp = win_find_tabpage(wp);
            }
            if au_no_enter_leave {
                autocmd_no_enter.set(autocmd_no_enter.get() - 1);
                autocmd_no_leave.set(autocmd_no_leave.get() - 1);
            }
        }
        if tp.is_null() {
            error.clear();
            err_msg(
                report,
                kErrorTypeException,
                c"Window was closed immediately",
            );
        } else {
            if style == kWinStyleMinimal {
                // SAFETY: `wp` is live -- its tab page still holds it.
                win_set_minimal_style(unsafe { Win::new(wp) });
                // SAFETY: as above.
                unsafe { didset_window_options(wp, true) };
                // SAFETY: as above.
                changed_window_setting(unsafe { Win::new(wp) });
            }
            // SAFETY: as above.
            rv = unsafe { (*wp).handle };
        }
    }
    if noautocmd {
        // SAFETY: paired with the `block_autocmds` above.
        unsafe { unblock_autocmds() };
    }
    rv.reported(error)
}

/// `win_split_ins` as this file calls it: a new window of `size`, with no
/// window or frame to place it against.
///
/// # Safety
/// The current window must be the one to split.
unsafe fn split_ins(size: ::core::ffi::c_int, flags: ::core::ffi::c_int) -> *mut win_T {
    // SAFETY: the caller's promise.
    unsafe {
        win_split_ins(
            size,
            flags,
            ::core::ptr::null_mut::<win_T>(),
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<frame_T>(),
        )
    }
}

/// Which side of its neighbour `win` was split off, for the `split` key.
///
/// A window with no frame, or one whose frame is the tab page's `topframe`,
/// answers the default rather than naming a side.
pub(crate) fn win_split_dir(win: Win) -> WinSplit {
    if win.w_frame.is_null() {
        return kWinSplitLeft;
    }
    let frame = win.frame();
    let Some(parent) = frame.parent() else {
        return kWinSplitLeft;
    };
    // A window in a column was split off the one below it when it still has a
    // sibling ahead of it, and off the one above it otherwise; in a row the
    // same test reads left/right.
    let column = parent.fr_layout as ::core::ffi::c_int == FR_COL;
    match (frame.next().is_none(), column) {
        (false, true) => kWinSplitAbove,
        (true, true) => kWinSplitBelow,
        (false, false) => kWinSplitLeft,
        (true, false) => kWinSplitRight,
    }
}

pub(crate) fn win_split_flags(mut split: WinSplit, mut toplevel: bool) -> ::core::ffi::c_int {
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if split as ::core::ffi::c_uint == kWinSplitAbove as ::core::ffi::c_int as ::core::ffi::c_uint
        || split as ::core::ffi::c_uint
            == kWinSplitBelow as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        flags |= WSP_HOR as ::core::ffi::c_int;
    } else {
        flags |= WSP_VERT as ::core::ffi::c_int;
    }
    if split as ::core::ffi::c_uint == kWinSplitAbove as ::core::ffi::c_int as ::core::ffi::c_uint
        || split as ::core::ffi::c_uint
            == kWinSplitLeft as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        flags |= if toplevel as ::core::ffi::c_int != 0 {
            WSP_TOP as ::core::ffi::c_int
        } else {
            WSP_ABOVE as ::core::ffi::c_int
        };
    } else {
        flags |= if toplevel as ::core::ffi::c_int != 0 {
            WSP_BOT as ::core::ffi::c_int
        } else {
            WSP_BELOW as ::core::ffi::c_int
        };
    }
    flags
}

/// Whether `wp` may be moved to tab page `tp`, reporting why not through
/// `err`.
///
/// # Safety
/// `wp` must be a live window, `tp` a live tab page and `err` the caller's
/// error slot.
pub(crate) unsafe fn win_can_move_tp(wp: *mut win_T, tp: *mut tabpage_T, err: &mut Error) -> bool {
    // SAFETY: the caller's error slot.
    let report = unsafe { ErrSlot::new(err) };
    let other_tab = if tp == curtab.get() {
        ::core::ptr::null_mut::<tabpage_T>()
    } else {
        tp
    };
    // SAFETY: the caller's window and tab page.
    if unsafe { one_window(wp, other_tab) } {
        let msg = c"Cannot move last non-floating window";
        err_msg(report, kErrorTypeException, msg);
        return false;
    }
    // SAFETY: the caller's window.
    if unsafe { win_locked(wp) } != 0 {
        let msg = c"Cannot move window to another tabpage whilst in use";
        err_msg(report, kErrorTypeException, msg);
        return false;
    }
    // SAFETY: the caller's error slot.
    if unsafe { window_layout_locked_err(CMD_SIZE, err) } {
        return false;
    }
    if textlock.get() != 0 || expr_map_locked() {
        // SAFETY: `e_textlock` is a static NUL-terminated message.
        unsafe { err_msg_raw(report, kErrorTypeException, e_textlock.as_ptr()) };
        return false;
    }
    if is_aucmd_win(wp) {
        let msg = c"Cannot move autocmd window to another tabpage";
        err_msg(report, kErrorTypeException, msg);
        return false;
    }
    if wp == cmdwin_win.get() || wp == cmdwin_old_curwin.get() {
        // SAFETY: `e_cmdwin` is a static NUL-terminated message.
        unsafe { err_msg_raw(report, kErrorTypeException, e_cmdwin.as_ptr()) };
        return false;
    }
    true
}

/// The window that takes `win`'s place in tab page `tp` once it leaves: its
/// neighbour in the layout, or the tab page's own choice for a float.
///
/// # Safety
/// `win` must be a live window and `tp` a live tab page.
pub(crate) unsafe fn win_find_altwin(win: *mut win_T, tp: *mut tabpage_T) -> *mut win_T {
    let at = (tp != curtab.get()).then(|| {
        // SAFETY: the caller's tab page.
        unsafe { TabPage::new(tp) }
    });
    let other_tab = at.map_or(::core::ptr::null_mut::<tabpage_T>(), TabPage::raw);
    // SAFETY: the caller's window.
    if unsafe { (*win).w_floating } {
        // SAFETY: as above, and `at` names the tab page to look in.
        unsafe { win_float_find_altwin(win, at) }.map_or(::core::ptr::null_mut(), Win::raw)
    } else {
        let mut dir: ::core::ffi::c_int = 0;
        // SAFETY: as above; `dir` is this frame's own.
        unsafe { winframe_find_altwin(win, &raw mut dir, other_tab, ::core::ptr::null_mut()) }
    }
}
