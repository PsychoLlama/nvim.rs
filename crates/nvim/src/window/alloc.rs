//! Allocating and freeing windows and frames, and the lists they live on.
//!
//! [`win_alloc`] creates a `win_T` with its option and variable dictionaries;
//! [`win_free`] tears one down, including the `WinInfo` remembered positions
//! and the autocommand bookkeeping.  [`win_append`]/[`win_remove`] and
//! [`frame_append`]/[`frame_insert`]/[`frame_remove`] are the linked-list
//! splices for the window list and the frame tree.  The `alloc_firstwin`
//! group builds the very first window and frame at startup, and
//! [`win_alloc_aucmd_win`] the invisible window autocommands execute in.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::mem::size_of;
use core::ptr;

use super::*;
use crate::arglist::alist_unlink;
use crate::arglist::global_arglist;
use crate::autocmd::aucmd_wins;
use crate::autocmd::{block_autocmds, unblock_autocmds};
use crate::buffer::{WinInfos, buflist_new};
use crate::decoration::clear_virttext;
use crate::eval::typval::tv_dict_alloc;
use crate::eval::vars::{init_var_dict, unref_var_dict, vars_clear};
use crate::fold::{clear_folding, delete_fold_recurse, fold_init_win};
use crate::grid::grid_assign_handle;
use crate::hashtab::hash_init;
use crate::main::{
    Columns, Rows, autocmd_busy, curbuf, curtab, curwin, firstwin, lastwin, p_ch, prevwin, topframe,
};
use crate::mark::free_jumplist;
use crate::r#match::clear_matches;
use crate::memory::xcalloc;
use crate::option::clear_winopt;
use crate::quickfix::qf_free_all;
use crate::tag::tagstack_clear_entry;
use crate::types::ui::kUIMultigrid;
use crate::types::{
    Error, FAIL, Integer, OK, OptInt, ScreenGrid, VAR_SCOPE, WinConfig, WinInfo, frame_T, handle_T,
    kErrorTypeNone, linenr_T, tabpage_T, win_T, winopt_T,
};
use crate::ui::{ui_call_grid_destroy, ui_has};
use crate::winfloat::{WIN_CONFIG_INIT, win_new_float};
use crate::winlayer::{
    Buf, Frame, TabPage, Win, buffers, defer_free_window, forget_window, register_window, tabs,
};
use ::libc::abort;

// ---------------------------------------------------------------------------
// The neighbours only this file reaches

/// `xcalloc(1, size_of::<T>())`, which never answers null.
fn zeroed<T>() -> *mut T {
    // SAFETY: `xcalloc` aborts rather than answering null, and a zeroed
    // `win_T`/`frame_T` is what upstream starts one from.
    unsafe { xcalloc(1, size_of::<T>()) }.cast::<T>()
}

/// A fresh window, zeroed but for its grid.
///
/// All-zero bytes are not a valid `ScreenGrid` -- its cell buffers are
/// `Vec`s, whose pointers are never null -- so the one field that owns an
/// allocation is written before anything can read or drop it.
fn zeroed_window() -> *mut win_T {
    let wp = zeroed::<win_T>();
    // SAFETY: a fresh allocation this thread alone holds; the zeroed grid is
    // overwritten, never read.
    unsafe { (&raw mut (*wp).w_grid_alloc).write(ScreenGrid::empty()) };
    wp
}

/// Free a window's option block and the folds saved with it.
fn clear_options(opt: *mut winopt_T) {
    // SAFETY: an option block inside a live window or entry.
    unsafe { clear_winopt(opt) };
}

// ---------------------------------------------------------------------------
// The first window, and the one autocommands run in

pub unsafe fn win_alloc_first() {
    if alloc_firstwin(None) == FAIL {
        // SAFETY: aborts the process; nothing comes back.
        unsafe { abort() };
    }
    let first = alloc_tabpage();
    first_tabpage.set(Some(first.id()));
    curtab.set(first.raw());
    // SAFETY: the tab page just allocated.
    unsafe { unuse_tabpage(first.raw()) };
}

pub unsafe fn win_alloc_aucmd_win(idx: c_int) {
    let mut err = Error {
        type_0: kErrorTypeNone,
        msg: ptr::null_mut::<c_char>(),
    };
    let fconfig = WinConfig {
        width: Columns.get(),
        height: 5,
        focusable: false,
        mouse: false,
        ..WIN_CONFIG_INIT
    };
    // SAFETY: a hidden float over a fresh scratch buffer, and a live `Error`.
    let win = unsafe { win_new_float(ptr::null_mut::<win_T>(), true, fconfig, &raw mut err) };
    // SAFETY: `aucmd_win_vec` has been sized for `idx`.
    unsafe { (*aucmd_wins().slot(idx as usize)).auc_win = win };
    // SAFETY: `win_new_float` answers a live window here.
    let mut win = unsafe { Win::new(win) };
    win.buffer().b_nwindows -= 1;
    win.w_onebuf_opt.wo_scb = 0;
    win.w_onebuf_opt.wo_crb = 0;
}

pub(crate) unsafe fn win_alloc_firstwin(oldwin: *mut win_T) -> c_int {
    // SAFETY: the caller's promise -- a live window or null.
    alloc_firstwin(unsafe { Win::from_raw(oldwin) })
}

/// Make the first window of a tab page, taking its settings from `oldwin` when
/// there is one and from the defaults when there is not.
fn alloc_firstwin(oldwin: Option<Win>) -> c_int {
    // SAFETY: `win_alloc` answers a live window.
    let mut win = unsafe { Win::new(win_alloc(ptr::null_mut::<win_T>(), false)) };
    curwin.set(win.raw());
    match oldwin {
        None => {
            // Very first window: make a new empty buffer for it.
            // SAFETY: a new unnamed listed buffer.
            let buf =
                unsafe { buflist_new(ptr::null_mut(), ptr::null_mut(), 1, BLN_LISTED as c_int) };
            curbuf.set(buf);
            // SAFETY: `buflist_new` answers a live buffer or null.
            let Some(mut buf) = (unsafe { Buf::from_raw(buf) }) else {
                return FAIL;
            };
            win.w_buffer = buf.raw();
            win.w_s = &raw mut buf.b_s;
            buf.b_nwindows = 1;
            win.w_alist = global_arglist();
            curwin_init();
        }
        Some(oldwin) => {
            // Make the new window a copy of the old one.
            // SAFETY: two live windows.
            unsafe { win_init(win.raw(), oldwin.raw(), 0) };
            win.w_onebuf_opt.wo_scb = 0;
            win.w_onebuf_opt.wo_crb = 0;
        }
    }
    let mut frame = attach_frame(win);
    topframe.set(frame.raw());
    frame.fr_width = Columns.get();
    frame.fr_height = Rows.get() - p_ch.get() as c_int - global_stl_rows();
    OK
}

/// Give `wp` a fresh leaf frame of its own.
pub(crate) fn attach_frame(wp: Win) -> Frame {
    let mut wp = wp;
    // SAFETY: a fresh zeroed `frame_T`, which is live from here on.
    let mut frp = unsafe { Frame::new(zeroed::<frame_T>()) };
    wp.w_frame = frp.raw();
    frp.fr_layout = FR_LEAF as c_char;
    frp.fr_win = wp.raw();
    frp
}

pub fn win_init_size() {
    let mut win = first_window();
    let mut top = current_topframe();
    let rows = (Rows.get() as OptInt
        - p_ch.get()
        - tabline_rows() as OptInt
        - global_stl_rows() as OptInt) as c_int;
    win.w_height = rows;
    win.w_prev_height = rows;
    win.w_view_height = win.w_height - win.w_winbar_height;
    win.w_height_outer = win.w_height;
    win.w_winrow_off = win.w_winbar_height;
    top.fr_height = rows;
    win.w_width = Columns.get();
    win.w_view_width = win.w_width;
    win.w_width_outer = win.w_width;
    top.fr_width = Columns.get();
}

/// The first window of the current tab page.
fn first_window() -> Win {
    // SAFETY: `firstwin` is set from startup to exit.
    unsafe { Win::new(firstwin.get()) }
}

// ---------------------------------------------------------------------------
// One window's memory

pub unsafe fn win_alloc(after: *mut win_T, hidden: bool) -> *mut win_T {
    // SAFETY: the caller's promise -- a live window or null.
    alloc(unsafe { Win::from_raw(after) }, hidden).raw()
}

/// Allocate a window, link it into the list after `after` unless `hidden`, and
/// give it the defaults a fresh window starts from.
fn alloc(after: Option<Win>, hidden: bool) -> Win {
    // SAFETY: a fresh window, which is live from here on.
    let mut new_wp = unsafe { Win::new(zeroed_window()) };
    last_win_id.set(last_win_id.get() + 1);
    new_wp.handle = last_win_id.get() as handle_T;
    register_window(new_wp);
    new_wp.w_grid_alloc.mouse_enabled = true;
    grid_assign_handle(&mut new_wp.w_grid_alloc);
    // SAFETY: a fresh dictionary, which becomes the window's own.
    new_wp.w_vars = unsafe { tv_dict_alloc() };
    // SAFETY: the dictionary just allocated, and the window's own scope.
    unsafe { init_var_dict(new_wp.w_vars, &raw mut new_wp.w_winvar, VAR_SCOPE) };
    // SAFETY: matched by the `unblock_autocmds` below.
    unsafe { block_autocmds() };
    if !hidden {
        // A window in another tab page goes on that tab page's list.
        let tp = after.and_then(win_tabpage).and_then(TabPage::into_other);
        append(after, new_wp, tp);
    }
    new_wp.w_wincol = 0;
    new_wp.w_width = Columns.get();
    new_wp.w_topline = 1 as linenr_T;
    new_wp.w_topfill = 0;
    new_wp.w_botline = 2 as linenr_T;
    new_wp.w_cursor.lnum = 1 as linenr_T;
    new_wp.w_scbind_pos = 1;
    new_wp.w_floating = false;
    new_wp.w_config = WIN_CONFIG_INIT;
    new_wp.w_viewport_invalid = true;
    new_wp.w_viewport_last_topline = 1 as linenr_T;
    new_wp.w_ns_hl = -1;
    new_wp.w_ns_set = SET_INIT;
    new_wp.w_onebuf_opt.wo_so = -1 as OptInt;
    new_wp.w_allbuf_opt.wo_so = new_wp.w_onebuf_opt.wo_so;
    new_wp.w_onebuf_opt.wo_siso = -1 as OptInt;
    new_wp.w_allbuf_opt.wo_siso = new_wp.w_onebuf_opt.wo_siso;
    new_wp.w_fraction = 0;
    new_wp.w_prev_fraction_row = -1;
    // SAFETY: a freshly allocated window, whose `w_folds` is still zeroed.
    unsafe { fold_init_win(new_wp) };
    // SAFETY: matches the `block_autocmds` above.
    unsafe { unblock_autocmds() };
    // Up to 1000 can be picked by the user.
    new_wp.w_next_match_id = 1000;
    new_wp
}

/// The tab page a window is on, from `win_find_tabpage()`.
fn win_tabpage(win: Win) -> Option<TabPage> {
    // SAFETY: a live window; the answer is a live tab page or null.
    unsafe { TabPage::from_raw(win_find_tabpage(win.raw())) }
}

pub unsafe fn free_wininfo(wip: *mut WinInfo) {
    // SAFETY: the caller's promise -- a live entry, which this consumes.
    if unsafe { (*wip).wi_optset } {
        clear_options(unsafe { &raw mut (*wip).wi_opt });
        // SAFETY: as above -- the entry's own fold array.
        unsafe { delete_fold_recurse(&raw mut (*wip).wi_folds) };
    }
    free(wip);
}

pub unsafe fn win_free(wp: *mut win_T, tp: *mut tabpage_T) {
    // SAFETY: the caller's promise -- a live window and a live tab page or
    // null.
    unsafe { free_win(Win::new(wp), TabPage::from_raw(tp)) };
}

/// Take `wp` off the window list and free everything hanging off it.
fn free_win(wp: Win, tp: Option<TabPage>) {
    let mut wp = wp;
    forget_window(wp.handle());
    // SAFETY: a live window; reduces the reference count to its argument list.
    clear_folding(wp);
    // SAFETY: the window's own argument list.
    unsafe { alist_unlink(wp.w_alist) };
    // Don't execute autocommands while the window is halfway deleted.
    // SAFETY: matched by the `unblock_autocmds` below.
    unsafe { block_autocmds() };
    free(wp.w_ns_set.keys);
    free(wp.w_ns_set.h.hash);
    wp.w_ns_set = SET_INIT;
    clear_options(&raw mut wp.w_onebuf_opt);
    clear_options(&raw mut wp.w_allbuf_opt);
    free(wp.w_p_lcs_chars.multispace);
    free(wp.w_p_lcs_chars.leadmultispace);
    // SAFETY: the window's own variable dictionary.
    let vars = unsafe { &raw mut (*wp.w_vars).dv_hashtab };
    // SAFETY: as above.
    unsafe { vars_clear(vars) };
    // SAFETY: as above.
    unsafe { hash_init(vars) };
    // SAFETY: as above.
    unsafe { unref_var_dict(wp.w_vars) };
    if prevwin.get() == wp.raw() {
        prevwin.set(ptr::null_mut::<win_T>());
    }
    for mut ttp in tabs() {
        if ttp.tp_prevwin == wp.raw() {
            ttp.tp_prevwin = ptr::null_mut::<win_T>();
        }
    }
    free(wp.w_lines);
    for i in 0..wp.w_tagstacklen {
        // SAFETY: an entry of the window's own tag stack.
        unsafe { tagstack_clear_entry(&mut wp.w_tagstack[i as usize]) };
    }
    free(wp.w_localdir);
    free(wp.w_prevdir);
    free_click_defs(wp.w_status_click_defs, wp.w_status_click_defs_size);
    free_click_defs(wp.w_winbar_click_defs, wp.w_winbar_click_defs_size);
    free_click_defs(wp.w_statuscol_click_defs, wp.w_statuscol_click_defs_size);

    for buf in buffers() {
        forget_wininfo(buf, wp);
    }

    // Free the border text.
    // SAFETY: the window's own virtual-text arrays.
    unsafe { clear_virttext(&raw mut wp.w_config.title_chunks) };
    // SAFETY: as above.
    unsafe { clear_virttext(&raw mut wp.w_config.footer_chunks) };
    // SAFETY: a live window, whose matches, jump list and quickfix stacks
    // these are.
    unsafe { clear_matches(wp.raw()) };
    // SAFETY: as above.
    unsafe { free_jumplist(wp.raw()) };
    qf_free_all(Some(wp));
    free(wp.w_p_cc_cols);
    free_grid(wp, false);
    if win_valid_any_tab(wp.raw()) {
        remove(wp, tp);
    }
    if autocmd_busy.get() {
        defer_free_window(wp);
    } else {
        free(wp.raw());
    }
    // SAFETY: matches the `block_autocmds` above.
    unsafe { unblock_autocmds() };
}

/// Drop `wp` from `buf`'s remembered positions, and with it the older of the
/// two entries that would then have no window: only the first such entry is
/// ever used again.
fn forget_wininfo(buf: Buf, wp: Win) {
    let mut buf = buf;
    let mut infos = WinInfos::of(&mut buf);
    let len = infos.entries_mut().len();
    let mut pos_wip = len;
    let mut pos_null = len;
    for (i, entry) in infos.entries_mut().iter().enumerate() {
        if entry.window() == wp.raw() {
            pos_wip = i;
        } else if entry.window().is_null() {
            pos_null = i;
        }
    }
    if pos_wip == len {
        return;
    }
    let entry = &mut infos.entries_mut()[pos_wip];
    entry.wi_win = ptr::null_mut::<win_T>();
    // Discard saved options if the style is minimal.
    if wp.w_config.style == kWinStyleMinimal && entry.wi_optset {
        clear_options(entry.opt());
        // SAFETY: the entry's own fold array.
        unsafe { delete_fold_recurse(entry.folds()) };
        entry.wi_optset = false;
    }
    if pos_null < len {
        let pos_delete = pos_null.max(pos_wip);
        // SAFETY: an entry of this array, which is dropped from it next.
        unsafe { free_wininfo(infos.entries_mut()[pos_delete].raw()) };
        infos.remove(pos_delete);
    }
}

pub unsafe fn win_free_grid(wp: *mut win_T, reinit: bool) {
    // SAFETY: the caller's promise -- a live window.
    free_grid(unsafe { Win::new(wp) }, reinit);
}

/// Give up the window's own grid, optionally leaving it zeroed for reuse.
pub(crate) fn free_grid(wp: Win, reinit: bool) {
    let mut wp = wp;
    if wp.w_grid_alloc.handle != 0 && ui_has(kUIMultigrid) {
        ui_call_grid_destroy(wp.w_grid_alloc.handle as Integer);
    }
    wp.w_grid_alloc.free();
    if reinit {
        wp.w_grid_alloc = ScreenGrid::empty();
    }
}

// ---------------------------------------------------------------------------
// The lists

pub unsafe fn win_append(after: *mut win_T, wp: *mut win_T, tp: *mut tabpage_T) {
    // SAFETY: the caller's promise -- live windows (`after` may be null) and a
    // live tab page or null.
    unsafe { append(Win::from_raw(after), Win::new(wp), TabPage::from_raw(tp)) };
}

/// Put `wp` in the window list of `tp` (or of the current tab page) after
/// `after`, or at the front when there is no `after`.
pub(crate) fn append(after: Option<Win>, wp: Win, tp: Option<TabPage>) {
    let mut wp = wp;
    debug_assert!(
        tp.is_none_or(|tp| !tp.is_current()),
        "tp == NULL || tp != curtab"
    );
    // After `None` is in front of the first.
    let before = match after {
        // SAFETY: a live window's `w_next` is a live window or null.
        Some(after) => unsafe { Win::from_raw(after.w_next) },
        None => list_first(tp),
    };
    wp.w_next = raw_win(before);
    wp.w_prev = raw_win(after);
    match after {
        Some(mut after) => after.w_next = wp.raw(),
        None => set_first(tp, wp.raw()),
    }
    match before {
        Some(mut before) => before.w_prev = wp.raw(),
        None => set_last(tp, wp.raw()),
    }
}

pub unsafe fn win_remove(wp: *mut win_T, tp: *mut tabpage_T) {
    // SAFETY: the caller's promise -- a live window and a live tab page or
    // null.
    unsafe { remove(Win::new(wp), TabPage::from_raw(tp)) };
}

/// Take `wp` out of the window list of `tp` (or of the current tab page).
pub(crate) fn remove(wp: Win, tp: Option<TabPage>) {
    debug_assert!(
        tp.is_none_or(|tp| !tp.is_current()),
        "tp == NULL || tp != curtab"
    );
    // SAFETY: a live window's neighbours are live windows or null.
    let (prev, next) = unsafe { (Win::from_raw(wp.w_prev), Win::from_raw(wp.w_next)) };
    match prev {
        Some(mut prev) => prev.w_next = wp.w_next,
        None => {
            set_first(tp, wp.w_next);
            // Unlike `win_append`, upstream keeps the current tab page's own
            // copy of the head in step here as well.
            sync_tab_first(tp, wp.w_next);
        }
    }
    match next {
        Some(mut next) => next.w_prev = wp.w_prev,
        None => {
            set_last(tp, wp.w_prev);
            sync_tab_last(tp, wp.w_prev);
        }
    }
}

/// The head of `tp`'s window list, or of the current tab page's.
fn list_first(tp: Option<TabPage>) -> Option<Win> {
    // SAFETY: the head of a live window list is a live window or null.
    unsafe { Win::from_raw(tp.map_or_else(|| firstwin.get(), |tp| tp.tp_firstwin)) }
}

/// The current tab page's list head lives in the `firstwin` global; another
/// tab page's in its own `tp_firstwin`.
fn set_first(tp: Option<TabPage>, wp: *mut win_T) {
    match tp {
        Some(mut tp) => tp.tp_firstwin = wp,
        None => firstwin.set(wp),
    }
}

fn set_last(tp: Option<TabPage>, wp: *mut win_T) {
    match tp {
        Some(mut tp) => tp.tp_lastwin = wp,
        None => lastwin.set(wp),
    }
}

/// `win_remove`'s extra write, which `win_append` does not make.
fn sync_tab_first(tp: Option<TabPage>, wp: *mut win_T) {
    if tp.is_none() {
        cur_tab().tp_firstwin = wp;
    }
}

fn sync_tab_last(tp: Option<TabPage>, wp: *mut win_T) {
    if tp.is_none() {
        cur_tab().tp_lastwin = wp;
    }
}

/// Link `frp` in after `after` in its row or column.
pub(crate) fn frame_append(after: Frame, frp: Frame) {
    let (mut after, mut frp) = (after, frp);
    frp.fr_next = after.fr_next;
    after.fr_next = frp.raw();
    if let Some(mut next) = frp.next() {
        next.fr_prev = frp.raw();
    }
    frp.fr_prev = after.raw();
}

/// Link `frp` in before `before` in its row or column.
pub(crate) fn frame_insert(before: Frame, frp: Frame) {
    let (mut before, mut frp) = (before, frp);
    frp.fr_next = before.raw();
    frp.fr_prev = before.fr_prev;
    before.fr_prev = frp.raw();
    match frp.prev() {
        Some(mut prev) => prev.fr_next = frp.raw(),
        None => {
            let mut parent = frp.parent().expect("a linked frame has a parent");
            parent.fr_child = frp.raw();
        }
    }
}

/// Take `frp` out of its row or column, leaving its own links alone so
/// [`frame_append`]/[`frame_insert`] can put it back.
pub(crate) fn frame_remove(frp: Frame) {
    match frp.prev() {
        Some(mut prev) => prev.fr_next = frp.fr_next,
        None => {
            let mut parent = frp.parent().expect("a linked frame has a parent");
            parent.fr_child = frp.fr_next;
        }
    }
    if let Some(mut next) = frp.next() {
        next.fr_prev = frp.fr_prev;
    }
}
