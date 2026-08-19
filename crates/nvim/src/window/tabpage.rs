//! Tab pages -- creating one, switching to it, and closing it.
//!
//! [`win_new_tabpage`] takes the current window out of the layout and gives
//! it a tab page of its own; [`leave_tab`] and [`enter_tab`] save and restore
//! the whole window layout around a switch, which is what makes a tab page a
//! layout rather than a list of windows.  [`goto_tabpage`] and [`goto_tab`]
//! are the entry points, [`tabpage_move`] reorders them, and
//! [`valid_tab`]/[`find_tabpage`]/[`tab_index`] are the lookups the rest of
//! the editor asks.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::ptr;

use super::*;
use crate::autocmd::{
    EVENT_BUFENTER, EVENT_BUFLEAVE, EVENT_TABENTER, EVENT_TABLEAVE, EVENT_TABNEW,
    EVENT_TABNEWENTERED, EVENT_WINENTER, EVENT_WINLEAVE, EVENT_WINNEW, block_autocmds,
    unblock_autocmds,
};
use crate::diff::diff_clear;
use crate::drawscreen::UPD_NOT_VALID;
use crate::eval::typval::tv_dict_alloc;
use crate::eval::vars::{init_var_dict, unref_var_dict, vars_clear};
use crate::eval::window::{restore_win_noblock, switch_win_noblock};
use crate::ex_getln::{text_locked, text_locked_msg};
use crate::global_cell::GlobalCell;
use crate::hashtab::hash_init;
use crate::main::{
    Columns, Rows, cmdmod, cmdwin_type, curbuf, curtab, curwin, diff_need_scrollbind, e_cmdwin,
    first_tabpage, firstwin, lastused_tabpage, lastwin, p_ch, p_tpm, postponed_split_tab, prevwin,
    redraw_tabline, skip_win_fix_scroll, starting, tabpage_handles, tabpage_move_disallowed,
    topframe,
};
use crate::map::map_del_int_ptr_t;
use crate::memory::{xcalloc, xstrdup};
use crate::message::set_keep_msg;
use crate::mouse::reset_dragwin;
use crate::normal::reset_VIsual_and_resel;
use crate::option::set_option_value;
use crate::options::kOptCmdheight;
use crate::types::{
    CMD_tabnew, FAIL, OK, OptInt, OptVal, OptValData, OptionSetFlags, VAR_SCOPE, buf_T, handle_T,
    int64_t, ptr_t, switchwin_T, tabpage_T,
};
use crate::winfloat::{win_config_float, win_float_update_statusline};
use crate::winlayer::tabs;

pub unsafe fn unuse_tabpage(tp: *mut tabpage_T) {
    // SAFETY: the caller's promise -- a live tab page.
    stash_tabpage(unsafe { TabPage::new(tp) });
}

/// Store the layout the globals currently describe in `tp`. To be used before
/// [`adopt_tabpage`].
pub(crate) fn stash_tabpage(tp: TabPage) {
    let mut tp = tp;
    tp.tp_topframe = topframe.get();
    tp.tp_firstwin = firstwin.get();
    tp.tp_lastwin = lastwin.get();
    tp.tp_curwin = curwin.get();
}

pub unsafe fn use_tabpage(tp: *mut tabpage_T) {
    // SAFETY: the caller's promise -- a live tab page.
    adopt_tabpage(unsafe { TabPage::new(tp) });
}

/// Point the layout globals at `tp`. May want to call [`stash_tabpage`] first.
pub(crate) fn adopt_tabpage(tp: TabPage) {
    curtab.set(tp.raw());
    topframe.set(tp.tp_topframe);
    firstwin.set(tp.tp_firstwin);
    lastwin.set(tp.tp_lastwin);
    curwin.set(tp.tp_curwin);
}

/// Allocate a `tabpage_T` and fill in its defaults.
pub(crate) fn alloc_tabpage() -> TabPage {
    static LAST_TP_HANDLE: GlobalCell<c_int> = GlobalCell::new(0);
    // SAFETY: `xcalloc` aborts rather than answering null, so this is a fresh
    // zeroed `tabpage_T`, live from here on.
    let mut tp = unsafe { TabPage::new(xcalloc(1, size_of::<tabpage_T>()) as *mut tabpage_T) };
    LAST_TP_HANDLE.set(LAST_TP_HANDLE.get() + 1);
    tp.handle = LAST_TP_HANDLE.get() as handle_T;
    let (key, val) = (tp.handle as c_int, tp.raw() as ptr_t);
    tabpage_handles.with_mut(|map| map_put_int_ptr_t(map, key, val));

    // Init t: variables.
    // SAFETY: a fresh dictionary, which becomes the tab page's own.
    tp.tp_vars = unsafe { tv_dict_alloc() };
    let (vars, scope) = (tp.tp_vars, &raw mut tp.tp_winvar);
    // SAFETY: the dictionary just allocated, and the tab page's own scope.
    unsafe { init_var_dict(vars, scope, VAR_SCOPE) };
    tp.tp_diff_invalid = 1;
    tp.tp_ch_used = p_ch.get();
    tp
}

pub unsafe fn free_tabpage(tp: *mut tabpage_T) {
    // SAFETY: the caller's promise -- a live tab page.
    free_tab(unsafe { TabPage::new(tp) });
}

/// Free `tp` and everything hanging off it.
pub(crate) fn free_tab(tp: TabPage) {
    let mut tp = tp;
    let key = tp.handle as c_int;
    // SAFETY: the handle map is the editor's own; a null slot means "do not
    // report the old value".
    tabpage_handles.with_mut(|map| unsafe { map_del_int_ptr_t(map, key, ptr::null_mut()) });
    // SAFETY: a live tab page's diff state.
    unsafe { diff_clear(tp.raw()) };
    for idx in 0..SNAP_COUNT {
        drop_snapshot(tp, idx);
    }
    let vars = tp.tp_vars;
    // SAFETY: the tab page's own dictionary; `vars_clear` frees every t:
    // variable and `hash_init` puts an empty table back.
    unsafe {
        vars_clear(&raw mut (*vars).dv_hashtab);
        hash_init(&raw mut (*vars).dv_hashtab);
        unref_var_dict(vars);
    }
    if tp.raw() == lastused_tabpage.get() {
        lastused_tabpage.set(ptr::null_mut::<tabpage_T>());
    }
    free(tp.tp_localdir);
    free(tp.tp_prevdir);
    free(tp.raw());
}

pub unsafe fn win_new_tabpage(
    after: c_int,
    filename: *mut c_char,
    enter: bool,
    first: *mut *mut win_T,
) -> *mut tabpage_T {
    let newtp = new_tabpage(after, filename, enter);
    if let Some((newtp, opened)) = newtp {
        if !first.is_null() {
            // SAFETY: the caller's promise -- a writable slot for the window.
            unsafe { *first = opened.raw() };
        }
        return newtp.raw();
    }
    ptr::null_mut()
}

/// Create a tab page with one window in it, showing the current buffer as
/// after `:split`, and answer it along with the window it opened.
///
/// Does not trigger `WinNewPre`: the window structures are not completely set
/// up yet and the event could dereference null pointers.
///
/// **The answer may already have been freed by autocommands.** `after` puts
/// the new tab page after tab page `after`, or after the current one when it
/// is zero; `filename` is passed to the `TabNew` autocommand.
pub(crate) fn new_tabpage(
    after: c_int,
    filename: *mut c_char,
    enter: bool,
) -> Option<(TabPage, Win)> {
    let old_curtab = cur_tab();
    if enter && cmdwin_type.get() != 0 {
        err(&raw const e_cmdwin as *const c_char);
        return None;
    }
    if layout_locked(CMD_tabnew) {
        return None;
    }
    let mut newtp = alloc_tabpage();

    // Remember the current windows in this tab page, avoiding the side effects
    // of `stash_tabpage` when not entering.
    if enter {
        if leave_tab(Some(cur_buf()), true) == FAIL {
            free(newtp.raw());
            return None;
        }
    } else {
        let mut cur = cur_tab();
        stash_tabpage(cur);
        // Save this to tell whether room must be made for the tabline.
        cur.tp_old_Rows_avail = rows_avail();
        firstwin.set(ptr::null_mut::<win_T>());
        lastwin.set(ptr::null_mut::<win_T>());
    }

    newtp.tp_localdir = clone_dir(old_curtab.tp_localdir);
    curtab.set(newtp.raw());

    // Create a new empty window.
    // SAFETY: the old tab page's current window, which is live.
    let result = unsafe { win_alloc_firstwin(old_curtab.tp_curwin) };
    debug_assert!(result == OK, "result == OK");
    let opened = cur_win();

    // Make the new tab page the new topframe.
    if after == 1 {
        // New tab page becomes the first one.
        newtp.tp_next = first_tabpage.get();
        first_tabpage.set(newtp.raw());
    } else {
        let mut tp = old_curtab;
        if after > 0 {
            // Put the new tab page before tab page `after`.
            let mut n = 2;
            tp = first_tab();
            while let Some(next) = tp.next().filter(|_| n < after) {
                n += 1;
                tp = next;
            }
        }
        newtp.tp_next = tp.tp_next;
        tp.tp_next = newtp.raw();
    }
    newtp.tp_curwin = opened.raw();
    newtp.tp_lastwin = newtp.tp_curwin;
    newtp.tp_firstwin = newtp.tp_lastwin;

    win_init_size();
    let mut firstw = first_win();
    firstw.w_winrow = tabline_rows();
    firstw.w_prev_winrow = firstw.w_winrow;
    comp_scroll(cur_win());
    newtp.tp_topframe = topframe.get();
    update_last_status(false);
    resize_terminal(cur_buf());

    if enter {
        redraw_all(UPD_NOT_VALID);
        check_tabpage_windows(old_curtab);
        lastused_tabpage.set(old_curtab.raw());
        enter_window(cur_win());
        fire(EVENT_WINNEW, cur_buf());
        fire(EVENT_WINENTER, cur_buf());
        fire_named(EVENT_TABNEW, filename, Some(cur_buf()));
        fire(EVENT_TABENTER, cur_buf());
    } else {
        stash_tabpage(cur_tab());
        adopt_tabpage(old_curtab);
        redraw_tabline.set(true); // the tabline may have been added, or changed
        if cur_tab().tp_old_Rows_avail != rows_avail() {
            new_screen_rows();
        }
        // Trigger autocommands in the context of the new window, letting
        // `switch_win_noblock` handle things like resetting `VIsual_active`.
        in_window(newtp, || {
            fire(EVENT_WINNEW, cur_buf());
            fire_named(EVENT_TABNEW, filename, Some(cur_buf()));
        });
    }
    Some((newtp, opened))
}

/// Run `body` with `tp`'s current window as the current one, and switch back
/// afterwards -- a scope rather than a guard, since the transpiled code has no
/// unwinding path either.
fn in_window(tp: TabPage, body: impl FnOnce()) {
    let mut switchwin = switchwin_T {
        sw_curwin: ptr::null_mut::<win_T>(),
        sw_curtab: ptr::null_mut::<tabpage_T>(),
        sw_same_win: false,
        sw_visual_active: false,
    };
    let (slot, win, raw) = (&raw mut switchwin, tp.tp_curwin, tp.raw());
    // SAFETY: a slot of our own, and a live window of the live tab page.
    let sw_result = unsafe { switch_win_noblock(slot, win, raw, true) };
    debug_assert!(sw_result == OK, "sw_result == OK");
    body();
    // SAFETY: the slot `switch_win_noblock` just filled in.
    unsafe { restore_win_noblock(slot, true) };
}

/// `Rows - 'cmdheight' - tabline - global statusline`: the rows a tab page's
/// windows may use.
fn rows_avail() -> int64_t {
    (Rows.get() as OptInt - p_ch.get() - tabline_rows() as OptInt - global_stl_rows() as OptInt)
        as int64_t
}

/// A copy of `dir`, owned by the caller, or null for null.
fn clone_dir(dir: *mut c_char) -> *mut c_char {
    if dir.is_null() {
        return ptr::null_mut();
    }
    // SAFETY: a NUL-terminated path.
    unsafe { xstrdup(dir) }
}

/// Open a new tab page if `:tab cmd` was used. It edits the same buffer, as
/// with `:split`. `OK` when a tab page was created.
pub(crate) fn may_open_tabpage() -> c_int {
    let n = match cmdmod.with(|m| m.cmod_tab) {
        0 => postponed_split_tab.get(),
        tab => tab,
    };
    if n == 0 {
        return FAIL;
    }
    cmdmod.with_mut(|m| m.cmod_tab = 0);
    postponed_split_tab.set(0);
    let status = if new_tabpage(n, ptr::null_mut(), true).is_some() {
        OK
    } else {
        FAIL
    };
    if status == OK {
        fire(EVENT_TABNEWENTERED, cur_buf());
    }
    status
}

pub unsafe fn make_tabpages(maxcount: c_int) -> c_int {
    let count = maxcount.min(p_tpm.get() as c_int);

    // Don't execute autocommands while creating the tab pages: `curwin` and
    // `curbuf` are not set up yet.
    // SAFETY: matched by the `unblock_autocmds` below.
    unsafe { block_autocmds() };
    let mut todo = count - 1;
    while todo > 0 {
        if new_tabpage(0, ptr::null_mut(), true).is_none() {
            break;
        }
        todo -= 1;
    }
    // SAFETY: matches the `block_autocmds` above.
    unsafe { unblock_autocmds() };
    count - todo
}

pub fn valid_tabpage(tpc: *mut tabpage_T) -> bool {
    valid_tab(tpc).is_some()
}

/// The tab page `tpc` names, if it is still on the tab page list.
///
/// Takes a raw pointer deliberately: the question is asked about a tab page
/// autocommands may already have freed, and the answer is the bridge back to a
/// value the rest of the family may dereference.
pub(crate) fn valid_tab(tpc: *mut tabpage_T) -> Option<TabPage> {
    tabs().find(|tp| tp.raw() == tpc)
}

pub fn valid_tabpage_win(tpc: *mut tabpage_T) -> c_int {
    let Some(tp) = valid_tab(tpc) else {
        return 0; // shouldn't happen
    };
    windows_in_tab(tp).any(|wp| valid_win_any_tab(wp.raw())) as c_int
}

pub unsafe fn close_tabpage(tab: *mut tabpage_T) {
    // SAFETY: the caller's promise -- a live tab page.
    close_tab(unsafe { TabPage::new(tab) });
}

/// Close tab page `tab`, which must have no windows left in it. There must be
/// another tab page or this will crash.
fn close_tab(tab: TabPage) {
    let ptp = if tab.raw() == first_tabpage.get() {
        first_tabpage.set(tab.tp_next);
        first_tab()
    } else {
        let found = tabs().find(|ptp| ptp.tp_next == tab.raw());
        debug_assert!(found.is_some(), "ptp != NULL");
        let mut prev = found.expect("another tab page precedes this one");
        prev.tp_next = tab.tp_next;
        prev
    };
    goto_tab(ptp, false, false);
    free_tab(tab);
}

pub fn find_tabpage(n: c_int) -> *mut tabpage_T {
    raw_tab(nth_tab(n))
}

/// Tab page `n`, the first being 1; the current one for zero. `None` when
/// there is no such tab page.
fn nth_tab(n: c_int) -> Option<TabPage> {
    if n == 0 {
        return Some(cur_tab());
    }
    if n < 0 {
        return None; // the walk runs off the end of the list
    }
    tabs().nth(n as usize - 1)
}

pub fn tabpage_index(ftp: *mut tabpage_T) -> c_int {
    index_of_tab(ftp)
}

/// The index of `tp`, the first being 1. The number of tab pages plus one when
/// it is not on the list.
pub(crate) fn tab_index(tp: TabPage) -> c_int {
    index_of_tab(tp.raw())
}

/// [`tab_index`] over a pointer, which is how the C's callers ask it.
fn index_of_tab(ftp: *mut tabpage_T) -> c_int {
    let mut i = 1;
    for tp in tabs() {
        if tp.raw() == ftp {
            break;
        }
        i += 1;
    }
    i
}

/// Prepare for leaving the current tab page, `new_curbuf` being what is going
/// to be the new `curbuf` (`None` when that is not known yet).
///
/// `FAIL` when autocommands changed `curtab`, in which case the tab page is
/// not left. Careful: after `OK` a new tab page must be entered very soon.
fn leave_tab(new_curbuf: Option<Buf>, trigger_leave_autocmds: bool) -> c_int {
    let mut tp = cur_tab();
    leave_window(cur_win());
    reset_VIsual_and_resel(); // stop Visual mode
    if trigger_leave_autocmds {
        if raw_buf(new_curbuf) != curbuf.get() {
            fire(EVENT_BUFLEAVE, cur_buf());
            if !tp.is_current() {
                return FAIL;
            }
        }
        fire(EVENT_WINLEAVE, cur_buf());
        if !tp.is_current() {
            return FAIL;
        }
        fire(EVENT_TABLEAVE, cur_buf());
        if !tp.is_current() {
            return FAIL;
        }
    }
    reset_dragwin();
    tp.tp_curwin = curwin.get();
    tp.tp_prevwin = prevwin.get();
    tp.tp_firstwin = firstwin.get();
    tp.tp_lastwin = lastwin.get();
    tp.tp_old_Rows_avail = rows_avail();
    if tp.tp_old_Columns != -1 as int64_t {
        tp.tp_old_Columns = Columns.get() as int64_t;
    }
    firstwin.set(ptr::null_mut::<win_T>());
    lastwin.set(ptr::null_mut::<win_T>());
    OK
}

/// Start using tab page `tp`. Only to be used after [`leave_tab`], or after
/// freeing the current tab page.
fn enter_tab(
    tp: TabPage,
    old_curbuf: Buf,
    trigger_enter_autocmds: bool,
    trigger_leave_autocmds: bool,
) {
    // SAFETY: the head of a live tab page's window list is a live window.
    let old_off = unsafe { Win::new(tp.tp_firstwin) }.w_winrow;
    let next_prevwin = tp.tp_prevwin;
    let old_curtab = cur_tab();
    adopt_tabpage(tp);

    if old_curtab.raw() != curtab.get() {
        check_tabpage_windows(old_curtab);
        if p_ch.get() != cur_tab().tp_ch_used {
            // Use the stored value of 'cmdheight', which may differ per tab
            // page. Handle the other side effects, but avoid setting frame
            // sizes, which are still correct.
            let new_ch = cur_tab().tp_ch_used;
            cur_tab().tp_ch_used = p_ch.get();
            command_frame_height.set(false);
            set_cmdheight(new_ch);
            command_frame_height.set(true);
        }
    }

    // The TabEnter event would ideally come first, but there is no valid
    // current window yet, which would break some commands. This triggers
    // autocommands, and so may make `tp` invalid.
    let flags = WEE_CURWIN_INVALID as c_int
        | if trigger_enter_autocmds {
            WEE_TRIGGER_ENTER_AUTOCMDS as c_int
        } else {
            0
        }
        | if trigger_leave_autocmds {
            WEE_TRIGGER_LEAVE_AUTOCMDS as c_int
        } else {
            0
        };
    // SAFETY: the tab page's own current window, which `adopt_tabpage` just
    // made the editor's.
    enter_ext(unsafe { Win::new(tp.tp_curwin) }, flags);
    prevwin.set(next_prevwin);

    update_last_status(false); // a status line may appear or disappear
    win_float_update_statusline();
    comp_positions(); // recompute `w_winrow` for all windows
    diff_need_scrollbind.set(true);
    // A click in a window is not usable for a following drag.
    reset_dragwin();

    // The tabline may have appeared or disappeared, so the frames may need
    // resizing; the same when the editor was resized.
    if cur_tab().tp_old_Rows_avail != rows_avail() || old_off != first_win().w_winrow {
        new_screen_rows();
    }
    if cur_tab().tp_old_Columns != Columns.get() as int64_t {
        if starting.get() == 0 {
            new_screen_cols(); // update window widths
            cur_tab().tp_old_Columns = Columns.get() as int64_t;
        } else {
            cur_tab().tp_old_Columns = -1 as int64_t; // update window widths later
        }
    }
    lastused_tabpage.set(old_curtab.raw());

    // Apply autocommands after updating the display, once 'lines' and 'columns'
    // have been set correctly.
    if trigger_enter_autocmds {
        fire(EVENT_TABENTER, cur_buf());
        if old_curbuf.raw() != curbuf.get() {
            fire(EVENT_BUFENTER, cur_buf());
        }
    }
    redraw_all(UPD_NOT_VALID);
}

/// `:set cmdheight=n`, without the frame resizing.
fn set_cmdheight(n: OptInt) {
    let value = OptVal {
        type_0: kOptValTypeNumber,
        data: OptValData { number: n },
    };
    set_option_value(kOptCmdheight, value, OptionSetFlags::NONE);
}

/// Tell an external UI that the windows and inline floats of `old_curtab` are
/// invisible now and the floats of `curtab` visible.
///
/// External floats are independent of tab pages, which is implemented by
/// always moving them to `curtab`.
fn check_tabpage_windows(old_curtab: TabPage) {
    // SAFETY: the head of a live tab page's window list is a live window, or
    // null.
    let mut cur = unsafe { Win::from_raw(old_curtab.tp_firstwin) };
    while let Some(mut wp) = cur {
        let next_wp = wp.next();
        if wp.w_floating {
            if wp.w_config.external {
                remove(wp, Some(old_curtab));
                append(Some(last_nonfloating(None)), wp, None);
            } else {
                drop_grid(wp);
            }
        }
        wp.w_pos_changed = true;
        cur = next_wp;
    }
    for mut wp in windows() {
        if wp.w_floating && !wp.w_config.external {
            config_float(wp);
        }
        wp.w_pos_changed = true;
    }
}

/// Re-place a floating window under its own configuration.
fn config_float(mut wp: Win) {
    let (raw, config) = (wp.raw(), wp.w_config);
    // SAFETY: a live window and its own configuration.
    unsafe { win_config_float(raw, config) };
}

pub fn goto_tabpage(n: c_int) {
    goto_tab_number(n);
}

/// Go to tab page `n`, as `:tab N` and `Ngt` ask it: zero is the next one,
/// negative counts backwards, and 9999 is the last.
pub(crate) fn goto_tab_number(n: c_int) {
    // SAFETY: reads the editor's lock state.
    if unsafe { text_locked() } {
        // Not allowed when editing the command line.
        // SAFETY: prints why.
        unsafe { text_locked_msg() };
        return;
    }
    // If there is only one it can't work.
    if first_tab().next().is_none() {
        if n > 1 {
            beep();
        }
        return;
    }

    let tp = if n == 0 {
        // No count: go to the next tab page, wrapping around the end.
        cur_tab().next().unwrap_or_else(first_tab)
    } else if n < 0 {
        // "gT": go to the previous tab page, wrapping around the end. "N gT"
        // repeats this N times.
        let mut ttp = cur_tab();
        let mut tp = ttp;
        for _ in n..0 {
            tp = first_tab();
            while let Some(next) = tp.next().filter(|_| tp.tp_next != ttp.raw()) {
                tp = next;
            }
            ttp = tp;
        }
        tp
    } else if n == 9999 {
        // Go to the last tab page.
        tabs().last().expect("at least one tab page")
    } else {
        // Go to tab page `n`.
        let Some(tp) = nth_tab(n) else {
            beep();
            return;
        };
        tp
    };
    goto_tab(tp, true, true);
}

pub unsafe fn goto_tabpage_tp(
    tp: *mut tabpage_T,
    trigger_enter_autocmds: bool,
    trigger_leave_autocmds: bool,
) {
    // SAFETY: the caller's promise -- a live tab page.
    let tp = unsafe { TabPage::new(tp) };
    goto_tab(tp, trigger_enter_autocmds, trigger_leave_autocmds);
}

/// Go to tab page `tp`. Note: does not update the GUI tab.
pub(crate) fn goto_tab(tp: TabPage, trigger_enter_autocmds: bool, trigger_leave_autocmds: bool) {
    if (trigger_enter_autocmds || trigger_leave_autocmds) && cmdwin_type.get() != 0 {
        err(&raw const e_cmdwin as *const c_char);
        return;
    }
    // Don't repeat a message in another tab page.
    // SAFETY: a null message clears the kept one.
    unsafe { set_keep_msg(ptr::null(), 0) };

    skip_win_fix_scroll.set(true);
    // SAFETY: the tab page's own current window, which is live.
    let new_curbuf = unsafe { Win::new(tp.tp_curwin) }.buffer_or_none();
    if !tp.is_current() && leave_tab(new_curbuf, trigger_leave_autocmds) == OK {
        let target = valid_tab(tp.raw()).unwrap_or_else(cur_tab);
        enter_tab(
            target,
            cur_buf(),
            trigger_enter_autocmds,
            trigger_leave_autocmds,
        );
    }
    skip_win_fix_scroll.set(false);
}

pub fn goto_tabpage_lastused() -> bool {
    goto_last_used_tab()
}

/// Go to the last accessed tab page, if there still is one.
pub(crate) fn goto_last_used_tab() -> bool {
    let Some(tp) = valid_tab(lastused_tabpage.get()) else {
        return false;
    };
    goto_tab(tp, true, true);
    true
}

pub unsafe fn goto_tabpage_win(tp: *mut tabpage_T, wp: *mut win_T) {
    // SAFETY: the caller's promise -- a live tab page and a live window.
    let (tp, wp) = unsafe { (TabPage::new(tp), Win::new(wp)) };
    goto_tab_win(tp, wp);
}

/// Enter window `wp` in tab page `tp`, updating the GUI tab as well.
pub(crate) fn goto_tab_win(tp: TabPage, wp: Win) {
    goto_tab(tp, true, true);
    if tp.is_current() {
        if let Some(wp) = valid_win(wp.raw()) {
            enter(wp, true);
        }
    }
}

pub fn tabpage_move(nr: c_int) {
    debug_assert!(!curtab.get().is_null(), "curtab != NULL");
    if first_tab().next().is_none() || tabpage_move_disallowed.get() != 0 {
        return;
    }

    let mut n = 1;
    let mut tp = first_tab();
    while let Some(next) = tp.next().filter(|_| n < nr) {
        n += 1;
        tp = next;
    }
    if tp.is_current() || (nr > 0 && tp.next().is_some() && tp.tp_next == curtab.get()) {
        return;
    }
    let mut tp_dst = tp;

    // Remove the current tab page from the list of tab pages.
    let mut cur = cur_tab();
    if cur.raw() == first_tabpage.get() {
        first_tabpage.set(cur.tp_next);
    } else {
        let Some(mut prev) = tabs().find(|tp2| tp2.tp_next == cur.raw()) else {
            return; // "cannot happen"
        };
        prev.tp_next = cur.tp_next;
    }

    // Re-insert it at the position asked for.
    if nr <= 0 {
        cur.tp_next = first_tabpage.get();
        first_tabpage.set(cur.raw());
    } else {
        cur.tp_next = tp_dst.tp_next;
        tp_dst.tp_next = cur.raw();
    }
    // The tabline needs redrawing; the tab page contents do not change.
    redraw_tabline.set(true);
}

/// A buffer argument that may be absent, as `leave_tab` takes it.
fn raw_buf(buf: Option<Buf>) -> *mut buf_T {
    buf.map_or(ptr::null_mut(), Buf::raw)
}
