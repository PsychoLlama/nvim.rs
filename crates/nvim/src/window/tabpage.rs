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

use crate::types::AutoEvent;
use crate::types::CmdIdx;
use crate::winlayer::TabId;
use crate::winlayer::last_used_tab;
use core::ffi::{c_char, c_int};
use core::ptr;

use super::*;
use crate::allocator::Owned;
use crate::autocmd::{block_autocmds, unblock_autocmds};
use crate::diff::diff_clear;
use crate::diff::state::diff_need_scrollbind;
use crate::drawscreen::UPD_NOT_VALID;
use crate::drawscreen::state::redraw_tabline;
use crate::eval::typval::tv_dict_alloc;
use crate::eval::vars::{init_var_dict, unref_var_dict, vars_clear};
use crate::eval::window::{restore_win_noblock, switch_win_noblock};
use crate::ex_docmd::state::cmdmod;
use crate::ex_getln::{text_locked, text_locked_msg};
use crate::global_cell::GlobalCell;
use crate::memory::xstrdup;
use crate::message::e_cmdwin;
use crate::message::set_keep_msg;
use crate::mouse::reset_dragwin;
use crate::normal::reset_visual_and_resel;
use crate::option::set_option_value;
use crate::option::vars::{p_ch, p_tpm};
use crate::options::kOptCmdheight;
use crate::startup::starting;
use crate::tag::state::postponed_split_tab;
use crate::types::{
    Failed, Handle, OptInt, OptVal, OptionSetFlags, SwitchWin, Tabpage, VAR_SCOPE, int64_t,
};
use crate::ui::state::{Columns, Rows};
use crate::window::state::{skip_win_fix_scroll, tabpage_move_disallowed};
use crate::winfloat::{win_config_float, win_float_update_statusline};
use crate::winlayer::graph::{
    cmdwin_type, first_tabpage, firstwin, lastused_tabpage, lastwin, prevwin, topframe,
};
use crate::winlayer::{WinId, forget_tabpage, register_tabpage, tabs};

pub unsafe fn unuse_tabpage(tabpage: TabPage) {
    stash_tabpage(tabpage);
}

/// Store the layout the globals currently describe in `tabpage`. To be used before
/// [`adopt_tabpage`].
pub(crate) fn stash_tabpage(tabpage: TabPage) {
    let mut tabpage = tabpage;
    tabpage.tp_topframe = topframe.get();
    tabpage.tp_firstwin = firstwin.get();
    tabpage.tp_lastwin = lastwin.get();
    tabpage.tp_curwin = Win::current_or_none().map(Win::id);
}

pub unsafe fn use_tabpage(tabpage: TabPage) {
    adopt_tabpage(tabpage);
}

/// Point the layout globals at `tabpage`. May want to call [`stash_tabpage`] first.
pub(crate) fn adopt_tabpage(tabpage: TabPage) {
    tabpage.make_current();
    topframe.set(tabpage.tp_topframe);
    firstwin.set(tabpage.tp_firstwin);
    lastwin.set(tabpage.tp_lastwin);
    tabpage
        .current_window()
        .expect("a live tab page has a current window")
        .make_current();
}

/// Allocate a `Tabpage` and fill in its defaults.
pub(crate) fn alloc_tabpage() -> TabPage {
    static LAST_TP_HANDLE: GlobalCell<c_int> = GlobalCell::new(0);
    // SAFETY: all-zero bytes are what upstream's `xcalloc(1, sizeof(Tabpage))`
    // hands a fresh tab page, and every field of `tabpage_S` that owns an
    // allocation is null when zeroed.
    let owned = Owned::new(unsafe { Box::<Tabpage>::new_zeroed().assume_init() });
    // SAFETY: the allocation just made, which `owned` keeps alive until the
    // registry takes it over two lines below.
    let mut tp = unsafe { TabPage::new(owned.address()) };
    LAST_TP_HANDLE.set(LAST_TP_HANDLE.get() + 1);
    tp.handle = LAST_TP_HANDLE.get() as Handle;
    let mut tp = register_tabpage(tp.handle, owned);

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

pub unsafe fn free_tabpage(tabpage: TabPage) {
    free_tab(tabpage);
}

/// Free `tabpage` and everything hanging off it.
pub(crate) fn free_tab(tabpage: TabPage) {
    // The allocation, out of the registry from here on. `tabpage` is still the
    // address to work through; `owned` is only who gives the memory back.
    // Every tab page reaching here was registered by `alloc_tabpage`.
    let owned =
        forget_tabpage(tabpage.handle()).expect("a tab page being freed is a registered one");
    diff_clear(tabpage);
    for idx in 0..SNAP_COUNT {
        drop_snapshot(tabpage, idx);
    }
    let vars = tabpage.tp_vars;
    // SAFETY: the tab page's own dictionary; `vars_clear` frees every t:
    // variable and `hash_init` puts an empty table back.
    unsafe {
        vars_clear(&raw mut (*vars).dv_hashtab);
        unref_var_dict(vars);
    }
    if lastused_tabpage.get() == Some(tabpage.id()) {
        lastused_tabpage.set(None);
    }
    free(tabpage.tp_localdir);
    free(tabpage.tp_prevdir);
    // The free: `tabpage_S`'s destructor runs and the memory goes back.
    drop(owned);
}

pub unsafe fn win_new_tabpage(
    after: c_int,
    filename: *mut c_char,
    enter: bool,
    first: Option<&mut Option<Win>>,
) -> Option<TabPage> {
    let (newtp, opened) = new_tabpage(after, filename, enter)?;
    if let Some(first) = first {
        *first = Some(opened);
    }
    Some(newtp)
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
    let old_curtab = TabPage::current();
    if enter && cmdwin_type.get() != 0 {
        err(e_cmdwin.as_ptr());
        return None;
    }
    if layout_locked(CmdIdx::tabnew) {
        return None;
    }
    let mut newtp = alloc_tabpage();

    // Remember the current windows in this tab page, avoiding the side effects
    // of `stash_tabpage` when not entering.
    if enter {
        if leave_tab(Some(Buf::current()), true).is_err() {
            free(newtp.raw());
            return None;
        }
    } else {
        let mut cur = TabPage::current();
        stash_tabpage(cur);
        // Save this to tell whether room must be made for the tabline.
        cur.tp_old_rows_avail = rows_avail();
        firstwin.set(None);
        lastwin.set(None);
    }

    newtp.tp_localdir = clone_dir(old_curtab.tp_localdir);
    newtp.make_current();

    // Create a new empty window.
    let result = win_alloc_firstwin(old_curtab.current_window());
    debug_assert!(result.is_ok(), "result.is_ok()");
    let opened = Win::current();

    // Make the new tab page the new topframe.
    if after == 1 {
        // New tab page becomes the first one.
        newtp.tp_next = first_tabpage.get();
        first_tabpage.set(Some(newtp.id()));
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
        tp.tp_next = Some(newtp.id());
    }
    newtp.tp_curwin = Some(opened.id());
    newtp.tp_lastwin = Some(opened.id());
    newtp.tp_firstwin = newtp.tp_lastwin;

    win_init_size();
    let mut firstw = first_win();
    firstw.w_winrow = tabline_rows();
    firstw.w_prev_winrow = firstw.w_winrow;
    comp_scroll(Win::current());
    newtp.tp_topframe = topframe.get();
    update_last_status(false);
    resize_terminal(Buf::current());

    if enter {
        redraw_all(UPD_NOT_VALID);
        check_tabpage_windows(old_curtab);
        lastused_tabpage.set(Some(old_curtab.id()));
        enter_window(Win::current());
        fire(AutoEvent::WinNew, Buf::current());
        fire(AutoEvent::WinEnter, Buf::current());
        fire_named(AutoEvent::TabNew, filename, Some(Buf::current()));
        fire(AutoEvent::TabEnter, Buf::current());
    } else {
        stash_tabpage(TabPage::current());
        adopt_tabpage(old_curtab);
        redraw_tabline.set(true); // the tabline may have been added, or changed
        if TabPage::current().tp_old_rows_avail != rows_avail() {
            new_screen_rows();
        }
        // Trigger autocommands in the context of the new window, letting
        // `switch_win_noblock` handle things like resetting `VIsual_active`.
        in_window(newtp, || {
            fire(AutoEvent::WinNew, Buf::current());
            fire_named(AutoEvent::TabNew, filename, Some(Buf::current()));
        });
    }
    Some((newtp, opened))
}

/// Run `body` with `tabpage`'s current window as the current one, and switch back
/// afterwards -- a scope rather than a guard, since the transpiled code has no
/// unwinding path either.
fn in_window(tabpage: TabPage, body: impl FnOnce()) {
    let mut switchwin = SwitchWin {
        sw_curwin: None,
        sw_curtab: None,
        sw_same_win: false,
        sw_visual_active: false,
    };
    let slot = &raw mut switchwin;
    let win = tabpage
        .current_window()
        .expect("a live tab page has a current window");
    // SAFETY: a slot of our own, and a live window of the live tab page.
    let sw_result = unsafe { switch_win_noblock(slot, win, Some(tabpage), true) };
    debug_assert!(sw_result.is_ok(), "the window was switched to");
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
/// with `:split`. `Ok` when a tab page was created.
pub(crate) fn may_open_tabpage() -> Result<(), Failed> {
    let n = match cmdmod.with(|m| m.cmod_tab) {
        0 => postponed_split_tab.get(),
        tab => tab,
    };
    if n == 0 {
        return Err(Failed);
    }
    cmdmod.with_mut(|m| m.cmod_tab = 0);
    postponed_split_tab.set(0);
    let status = if new_tabpage(n, ptr::null_mut(), true).is_some() {
        Ok(())
    } else {
        Err(Failed)
    };
    if status.is_ok() {
        fire(AutoEvent::TabNewEntered, Buf::current());
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

pub(crate) fn valid_tabpage(tpc: TabId) -> bool {
    valid_tab(tpc).is_some()
}

/// The tab page `tpc` names, if it is still on the tab page list.
///
/// Takes an id deliberately: the question is asked about a tab page
/// autocommands may already have closed, and the answer is the bridge back to
/// a value the rest of the family may dereference.
pub(crate) fn valid_tab(tpc: TabId) -> Option<TabPage> {
    tabs().find(|tp| tp.id() == tpc)
}

pub fn valid_tabpage_win(tpc: TabPage) -> c_int {
    let Some(tp) = valid_tab(tpc.id()) else {
        return 0; // shouldn't happen
    };
    windows_in_tab(tp).any(|wp| valid_win_any_tab(wp.id())) as c_int
}

pub unsafe fn close_tabpage(tab: TabPage) {
    close_tab(tab);
}

/// Close tab page `tab`, which must have no windows left in it. There must be
/// another tab page or this will crash.
fn close_tab(tab: TabPage) {
    let id = tab.id();
    let ptp = if first_tabpage.get() == Some(id) {
        first_tabpage.set(tab.tp_next);
        first_tab()
    } else {
        let found = tabs().find(|ptp| ptp.tp_next == Some(id));
        debug_assert!(found.is_some(), "ptp != NULL");
        let mut prev = found.expect("another tab page precedes this one");
        prev.tp_next = tab.tp_next;
        prev
    };
    goto_tab(ptp, false, false);
    free_tab(tab);
}

pub fn find_tabpage(n: c_int) -> Option<TabPage> {
    nth_tab(n)
}

/// Tab page `n`, the first being 1; the current one for zero. `None` when
/// there is no such tab page.
fn nth_tab(n: c_int) -> Option<TabPage> {
    if n == 0 {
        return Some(TabPage::current());
    }
    if n < 0 {
        return None; // the walk runs off the end of the list
    }
    tabs().nth(n as usize - 1)
}

pub(crate) fn tabpage_index(ftp: Option<TabPage>) -> c_int {
    let mut i = 1;
    for tp in tabs() {
        if Some(tp) == ftp {
            break;
        }
        i += 1;
    }
    i
}

/// The index of `tabpage`, the first being 1. The number of tab pages plus one when
/// it is not on the list.
pub(crate) fn tab_index(tabpage: TabPage) -> c_int {
    tabpage_index(Some(tabpage))
}

/// Prepare for leaving the current tab page, `new_curbuf` being what is going
/// to be the new `curbuf` (`None` when that is not known yet).
///
/// `Err` when autocommands changed `curtab`, in which case the tab page is
/// not left. Careful: after `Ok` a new tab page must be entered very soon.
fn leave_tab(new_curbuf: Option<Buf>, trigger_leave_autocmds: bool) -> Result<(), Failed> {
    let mut tp = TabPage::current();
    leave_window(Win::current());
    reset_visual_and_resel(); // stop Visual mode
    if trigger_leave_autocmds {
        if new_curbuf != Buf::current_or_none() {
            fire(AutoEvent::BufLeave, Buf::current());
            if !tp.is_current() {
                return Err(Failed);
            }
        }
        fire(AutoEvent::WinLeave, Buf::current());
        if !tp.is_current() {
            return Err(Failed);
        }
        fire(AutoEvent::TabLeave, Buf::current());
        if !tp.is_current() {
            return Err(Failed);
        }
    }
    reset_dragwin();
    tp.tp_curwin = Win::current_or_none().map(Win::id);
    tp.tp_prevwin = prevwin.get();
    tp.tp_firstwin = firstwin.get();
    tp.tp_lastwin = lastwin.get();
    tp.tp_old_rows_avail = rows_avail();
    if tp.tp_old_columns != -1 as int64_t {
        tp.tp_old_columns = Columns.get() as int64_t;
    }
    firstwin.set(None);
    lastwin.set(None);
    Ok(())
}

/// Start using tab page `tabpage`. Only to be used after [`leave_tab`], or after
/// freeing the current tab page.
fn enter_tab(
    tabpage: TabPage,
    old_curbuf: Buf,
    trigger_enter_autocmds: bool,
    trigger_leave_autocmds: bool,
) {
    let old_off = tabpage
        .tp_firstwin
        .and_then(WinId::get)
        .expect("a live tab page has a first window")
        .w_winrow;
    let next_prevwin = tabpage.tp_prevwin;
    let old_curtab = TabPage::current();
    adopt_tabpage(tabpage);

    if old_curtab.raw() != TabPage::current_raw() {
        check_tabpage_windows(old_curtab);
        if p_ch.get() != TabPage::current().tp_ch_used {
            // Use the stored value of 'cmdheight', which may differ per tab
            // page. Handle the other side effects, but avoid setting frame
            // sizes, which are still correct.
            let new_ch = TabPage::current().tp_ch_used;
            TabPage::current().tp_ch_used = p_ch.get();
            command_frame_height.set(false);
            set_cmdheight(new_ch);
            command_frame_height.set(true);
        }
    }

    // The TabEnter event would ideally come first, but there is no valid
    // current window yet, which would break some commands. This triggers
    // autocommands, and so may make `tabpage` invalid.
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
    // The tab page's own current window, which `adopt_tabpage` just made the
    // editor's.
    enter_ext(
        tabpage
            .current_window()
            .expect("a live tab page has a current window"),
        flags,
    );
    prevwin.set(next_prevwin);

    update_last_status(false); // a status line may appear or disappear
    win_float_update_statusline();
    comp_positions(); // recompute `w_winrow` for all windows
    diff_need_scrollbind.set(true);
    // A click in a window is not usable for a following drag.
    reset_dragwin();

    // The tabline may have appeared or disappeared, so the frames may need
    // resizing; the same when the editor was resized.
    if TabPage::current().tp_old_rows_avail != rows_avail() || old_off != first_win().w_winrow {
        new_screen_rows();
    }
    if TabPage::current().tp_old_columns != Columns.get() as int64_t {
        if starting.get() == 0 {
            new_screen_cols(); // update window widths
            TabPage::current().tp_old_columns = Columns.get() as int64_t;
        } else {
            TabPage::current().tp_old_columns = -1 as int64_t; // update window widths later
        }
    }
    lastused_tabpage.set(Some(old_curtab.id()));

    // Apply autocommands after updating the display, once 'lines' and 'columns'
    // have been set correctly.
    if trigger_enter_autocmds {
        fire(AutoEvent::TabEnter, Buf::current());
        if old_curbuf.raw() != Buf::current_raw() {
            fire(AutoEvent::BufEnter, Buf::current());
        }
    }
    redraw_all(UPD_NOT_VALID);
}

/// `:set cmdheight=n`, without the frame resizing.
fn set_cmdheight(n: OptInt) {
    set_option_value(kOptCmdheight, OptVal::Number(n), OptionSetFlags::NONE);
}

/// Tell an external UI that the windows and inline floats of `old_curtab` are
/// invisible now and the floats of `curtab` visible.
///
/// External floats are independent of tab pages, which is implemented by
/// always moving them to `curtab`.
fn check_tabpage_windows(old_curtab: TabPage) {
    let mut cur = old_curtab.tp_firstwin.and_then(WinId::get);
    while let Some(mut wp) = cur {
        let next_wp = wp.next();
        if wp.w_floating {
            if wp.w_config.external {
                win_remove(wp, Some(old_curtab));
                win_append(Some(lastwin_nofloating(None)), wp, None);
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
fn config_float(window: Win) {
    let (raw, config) = (window.raw(), window.w_config.clone());
    // SAFETY: a live window and its own configuration.
    unsafe { win_config_float(Win::new(raw), config) };
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
        TabPage::current().next().unwrap_or_else(first_tab)
    } else if n < 0 {
        // "gT": go to the previous tab page, wrapping around the end. "N gT"
        // repeats this N times.
        let mut ttp = TabPage::current();
        let mut tp = ttp;
        for _ in n..0 {
            let target = Some(ttp.id());
            tp = first_tab();
            while let Some(next) = tp.next().filter(|_| tp.tp_next != target) {
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
    tabpage: TabPage,
    trigger_enter_autocmds: bool,
    trigger_leave_autocmds: bool,
) {
    goto_tab(tabpage, trigger_enter_autocmds, trigger_leave_autocmds);
}

/// Go to tab page `tabpage`. Note: does not update the GUI tab.
pub(crate) fn goto_tab(
    tabpage: TabPage,
    trigger_enter_autocmds: bool,
    trigger_leave_autocmds: bool,
) {
    if (trigger_enter_autocmds || trigger_leave_autocmds) && cmdwin_type.get() != 0 {
        err(e_cmdwin.as_ptr());
        return;
    }
    // Don't repeat a message in another tab page.
    // SAFETY: a null message clears the kept one.
    unsafe { set_keep_msg(ptr::null(), 0) };

    skip_win_fix_scroll.set(true);
    // Taken while it is live: `leave_tab` fires autocommands that can free it.
    let tabpage_id = tabpage.id();
    let new_curbuf = tabpage.current_window().and_then(Win::buffer_or_none);
    if !tabpage.is_current() && leave_tab(new_curbuf, trigger_leave_autocmds).is_ok() {
        let target = valid_tab(tabpage_id).unwrap_or_else(TabPage::current);
        enter_tab(
            target,
            Buf::current(),
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
    let Some(tp) = last_used_tab() else {
        return false;
    };
    goto_tab(tp, true, true);
    true
}

pub unsafe fn goto_tabpage_win(tabpage: TabPage, window: Win) {
    let (tp, wp) = (tabpage, window);
    goto_tab_win(tp, wp);
}

/// Enter window `wp` in tab page `tabpage`, updating the GUI tab as well.
pub(crate) fn goto_tab_win(tabpage: TabPage, window: Win) {
    // Taken while it is live: `goto_tab` fires autocommands that can free it.
    let window_id = window.id();
    goto_tab(tabpage, true, true);
    if tabpage.is_current()
        && let Some(wp) = valid_win(window_id)
    {
        enter(wp, true);
    }
}

pub fn tabpage_move(nr: c_int) {
    debug_assert!(TabPage::current_or_none().is_some(), "curtab != NULL");
    if first_tab().next().is_none() || tabpage_move_disallowed.get() != 0 {
        return;
    }

    let mut n = 1;
    let mut tp = first_tab();
    while let Some(next) = tp.next().filter(|_| n < nr) {
        n += 1;
        tp = next;
    }
    let mut cur = TabPage::current();
    let id = cur.id();
    if tp.is_current() || (nr > 0 && tp.next().is_some() && tp.tp_next == Some(id)) {
        return;
    }
    let mut tp_dst = tp;

    // Remove the current tab page from the list of tab pages.
    if first_tabpage.get() == Some(id) {
        first_tabpage.set(cur.tp_next);
    } else {
        let Some(mut prev) = tabs().find(|tp2| tp2.tp_next == Some(id)) else {
            return; // "cannot happen"
        };
        prev.tp_next = cur.tp_next;
    }

    // Re-insert it at the position asked for.
    if nr <= 0 {
        cur.tp_next = first_tabpage.get();
        first_tabpage.set(Some(id));
    } else {
        cur.tp_next = tp_dst.tp_next;
        tp_dst.tp_next = Some(id);
    }
    // The tabline needs redrawing; the tab page contents do not change.
    redraw_tabline.set(true);
}
