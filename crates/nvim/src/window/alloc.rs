//! Allocating and freeing windows and frames, and the lists they live on.
//!
//! [`win_alloc`] creates a `Window` with its option and variable dictionaries;
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
use crate::autocmd::state::autocmd_busy;
use crate::autocmd::{block_autocmds, unblock_autocmds};
use crate::buffer::{WinInfos, buflist_new};
use crate::decoration::clear_virttext;
use crate::eval::typval::tv_dict_alloc;
use crate::eval::vars::{init_var_dict, unref_var_dict, vars_clear};
use crate::fold::{clear_folding, delete_fold_recurse, fold_init_win};
use crate::grid::grid_assign_handle;
use crate::hashtab::hash_init;
use crate::mark::free_jumplist;
use crate::r#match::clear_matches;
use crate::memory::xcalloc;
use crate::option::clear_winopt;
use crate::option::vars::p_ch;
use crate::quickfix::qf_free_all;
use crate::registry::id_set;
use crate::tag::tagstack_clear_entry;
use crate::types::ui::kUIMultigrid;
use crate::types::{
    Error, Failed, Handle, Integer, LineNr, OptInt, ScreenGrid, VAR_SCOPE, WinConfig, WinInfo,
    WinOpt, Window,
};
use crate::ui::state::{Columns, Rows};
use crate::ui::{ui_call_grid_destroy, ui_has};
use crate::winfloat::{WIN_CONFIG_INIT, win_new_float};
use crate::winlayer::graph::{firstwin, lastwin, leave_curbuf, prevwin, topframe};
use crate::winlayer::{
    Buf, FrameRef, TabPage, Win, WinId, buffers, defer_free_window, forget_window, register_window,
    tabs,
};
use ::libc::abort;

// ---------------------------------------------------------------------------
// The neighbours only this file reaches

/// `xcalloc(1, size_of::<T>())`, which never answers null.
fn zeroed<T>() -> *mut T {
    // SAFETY: `xcalloc` aborts rather than answering null, and a zeroed
    // `Window`/`Frame` is what upstream starts one from.
    unsafe { xcalloc(1, size_of::<T>()) }.cast::<T>()
}

/// A fresh window, zeroed but for the two fields that own an allocation.
///
/// All-zero bytes are not a valid `ScreenGrid` -- its cell buffers are
/// `Vec`s, whose pointers are never null -- nor a valid `w_ns_set`, which is
/// a `HashSet` and carries a hasher, so both are written before anything can
/// read or drop them.
fn zeroed_window() -> Win {
    let wp = zeroed::<Window>();
    // SAFETY: a fresh allocation this thread alone holds; the zeroed grid
    // and set are overwritten, never read. The window is live from here on,
    // which is what `Win::new` asks -- `alloc` registers it a few lines down.
    unsafe {
        (&raw mut (*wp).w_grid_alloc).write(ScreenGrid::empty());
        (&raw mut (*wp).w_ns_set).write(id_set());
        Win::new(wp)
    }
}

/// A registered window with nothing in it but zeroed fields: [`win_alloc`]'s
/// first three lines and none of its body.
///
/// For `arith`'s tests, which build layout trees of their own. A window that
/// is registered is one a [`FrameRef`] can name, and `fr_win` is a handle —
/// so a frame tree over unregistered windows would read as a tree of empty
/// leaves. Nothing here allocates a dictionary, a fold array or a grid
/// handle, so [`free_bare_window`] is a `forget` and an `xfree`.
#[cfg(test)]
pub(crate) fn bare_window() -> Win {
    let mut win = zeroed_window();
    last_win_id.set(last_win_id.get() + 1);
    win.set_handle(last_win_id.get() as Handle);
    register_window(win);
    win
}

/// Give back what [`bare_window`] made.
#[cfg(test)]
pub(crate) fn free_bare_window(win: Win) {
    forget_window(win.handle());
    free(win.raw());
}

/// Free a window's option block and the folds saved with it.
fn clear_options(opt: *mut WinOpt) {
    // SAFETY: an option block inside a live window or entry.
    unsafe { clear_winopt(opt) };
}

// ---------------------------------------------------------------------------
// The first window, and the one autocommands run in

pub unsafe fn win_alloc_first() {
    if win_alloc_firstwin(None).is_err() {
        // SAFETY: aborts the process; nothing comes back.
        unsafe { abort() };
    }
    let first = alloc_tabpage();
    first_tabpage.set(Some(first.id()));
    first.make_current();
    unuse_tabpage(first);
}

pub unsafe fn win_alloc_aucmd_win(idx: c_int) {
    let mut err = Error::none();
    let fconfig = WinConfig {
        width: Columns.get(),
        height: 5,
        focusable: false,
        mouse: false,
        ..WIN_CONFIG_INIT
    };
    // A hidden float over a fresh scratch buffer; it always answers a window.
    let mut win = win_new_float(None, true, fconfig, &mut err).expect("the autocommand window");
    // SAFETY: `aucmd_win_vec` has been sized for `idx`.
    unsafe { (*aucmd_wins().slot(idx as usize)).auc_win = win.raw() };
    win.buffer().b_nwindows -= 1;
    win.w_onebuf_opt.wo_scb = 0;
    win.w_onebuf_opt.wo_crb = 0;
}

pub(crate) fn win_alloc_firstwin(oldwin: Option<Win>) -> Result<(), Failed> {
    let mut win = win_alloc(None, false);
    win.make_current();
    match oldwin {
        None => {
            // Very first window: make a new empty buffer for it.
            // SAFETY: a new unnamed listed buffer.
            let buf =
                unsafe { buflist_new(ptr::null_mut(), ptr::null_mut(), 1, BLN_LISTED as c_int) };
            let Some(mut buf) = buf else {
                leave_curbuf();
                return Err(Failed);
            };
            buf.make_current();
            win.w_buffer = buf.raw();
            win.w_s = &raw mut buf.b_s;
            buf.b_nwindows = 1;
            win.w_alist = global_arglist();
            curwin_init();
        }
        Some(oldwin) => {
            // Make the new window a copy of the old one.
            win_init(win, oldwin, 0);
            win.w_onebuf_opt.wo_scb = 0;
            win.w_onebuf_opt.wo_crb = 0;
        }
    }
    let mut frame = attach_frame(win);
    topframe.set(Some(frame.id()));
    frame.fr_width = Columns.get();
    frame.fr_height = Rows.get() - p_ch.get() as c_int - global_stl_rows();
    Ok(())
}

/// Give `window` a fresh leaf frame of its own.
pub(crate) fn attach_frame(window: Win) -> FrameRef {
    let mut window = window;
    let mut frp = new_frame();
    window.w_frame = Some(frp.id());
    frp.fr_layout = FR_LEAF as c_char;
    frp.fr_win = Some(window.id());
    frp
}

pub fn win_init_size() {
    let mut win = first_win();
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

/// The first window of the current tab page, which exists from startup to
/// exit. The fallible form is `winlayer::first_window`.
fn first_win() -> Win {
    crate::winlayer::first_window().expect("the editor always has a window")
}

// ---------------------------------------------------------------------------
// One window's memory

pub(crate) fn win_alloc(after: Option<Win>, hidden: bool) -> Win {
    let mut new_wp = zeroed_window();
    last_win_id.set(last_win_id.get() + 1);
    new_wp.set_handle(last_win_id.get() as Handle);
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
        win_append(after, new_wp, tp);
    }
    new_wp.w_wincol = 0;
    new_wp.w_width = Columns.get();
    new_wp.w_topline = 1 as LineNr;
    new_wp.w_topfill = 0;
    new_wp.w_botline = 2 as LineNr;
    new_wp.w_cursor.lnum = 1 as LineNr;
    new_wp.w_scbind_pos = 1;
    new_wp.w_floating = false;
    new_wp.w_config = WIN_CONFIG_INIT;
    new_wp.w_viewport_invalid = true;
    new_wp.w_viewport_last_topline = 1 as LineNr;
    new_wp.w_ns_hl = -1;
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
    win_find_tabpage(win.id())
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

pub(crate) fn win_free(window: Win, tabpage: Option<TabPage>) {
    let mut window = window;
    // SAFETY: a live window; reduces the reference count to its argument list.
    clear_folding(window);
    // SAFETY: the window's own argument list.
    unsafe { alist_unlink(window.w_alist) };
    // Don't execute autocommands while the window is halfway deleted.
    // SAFETY: matched by the `unblock_autocmds` below.
    unsafe { block_autocmds() };
    // The window's memory goes back with `free` below, which runs no
    // destructor, so the set's own allocation is released here. `take`
    // rather than `drop_in_place`: what is left is a valid empty set, which
    // the deferred-free path may still be handed.
    drop(core::mem::take(&mut window.w_ns_set));
    clear_options(&raw mut window.w_onebuf_opt);
    clear_options(&raw mut window.w_allbuf_opt);
    free(window.w_p_lcs_chars.multispace);
    free(window.w_p_lcs_chars.leadmultispace);
    // SAFETY: the window's own variable dictionary.
    let vars = unsafe { &raw mut (*window.w_vars).dv_hashtab };
    // SAFETY: as above.
    unsafe { vars_clear(vars) };
    // SAFETY: as above.
    unsafe { hash_init(vars) };
    // SAFETY: as above.
    unsafe { unref_var_dict(window.w_vars) };
    if prevwin.get() == Some(window.id()) {
        prevwin.set(None);
    }
    for mut ttp in tabs() {
        if ttp.tp_prevwin == Some(window.id()) {
            ttp.tp_prevwin = None;
        }
    }
    free(window.w_lines);
    for i in 0..window.w_tagstacklen {
        // SAFETY: an entry of the window's own tag stack.
        unsafe { tagstack_clear_entry(&mut window.w_tagstack[i as usize]) };
    }
    free(window.w_localdir);
    free(window.w_prevdir);
    free_click_defs(window.w_status_click_defs, window.w_status_click_defs_size);
    free_click_defs(window.w_winbar_click_defs, window.w_winbar_click_defs_size);
    free_click_defs(
        window.w_statuscol_click_defs,
        window.w_statuscol_click_defs_size,
    );

    for buf in buffers() {
        forget_wininfo(buf, window);
    }

    // Free the border text.
    // SAFETY: the window's own virtual-text arrays.
    unsafe { clear_virttext(&raw mut window.w_config.title_chunks) };
    // SAFETY: as above.
    unsafe { clear_virttext(&raw mut window.w_config.footer_chunks) };
    // SAFETY: a live window, whose matches, jump list and quickfix stacks
    // these are.
    unsafe { clear_matches(window) };
    // SAFETY: as above.
    unsafe { free_jumplist(window) };
    qf_free_all(Some(window));
    free(window.w_p_cc_cols);
    free_grid(window, false);
    if win_valid_any_tab(window.id()) {
        win_remove(window, tabpage);
    }
    // Out of the registry only now, *after* the unlink: the list links are
    // handles, so a window that is still on a list has to stay findable or
    // every walk stops at it. Upstream forgets the handle at the top of this
    // function, where a pointer link could not care. Nothing between the two
    // points can look a window up -- `block_autocmds` covers all but the two
    // calls above it, and neither reaches the registry.
    forget_window(window.handle());
    if autocmd_busy.get() {
        defer_free_window(window);
    } else {
        free(window.raw());
    }
    // SAFETY: matches the `block_autocmds` above.
    unsafe { unblock_autocmds() };
}

/// Drop `window` from `buffer`'s remembered positions, and with it the older of the
/// two entries that would then have no window: only the first such entry is
/// ever used again.
fn forget_wininfo(buffer: Buf, window: Win) {
    let mut buffer = buffer;
    let mut infos = WinInfos::of(&mut buffer);
    let len = infos.entries_mut().len();
    let mut pos_wip = len;
    let mut pos_null = len;
    for (i, entry) in infos.entries_mut().iter().enumerate() {
        if entry.window() == Some(window) {
            pos_wip = i;
        } else if entry.window().is_none() {
            pos_null = i;
        }
    }
    if pos_wip == len {
        return;
    }
    let entry = &mut infos.entries_mut()[pos_wip];
    entry.wi_win = ptr::null_mut::<Window>();
    // Discard saved options if the style is minimal.
    if window.w_config.style == kWinStyleMinimal && entry.wi_optset {
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

pub fn win_free_grid(window: Win, reinit: bool) {
    free_grid(window, reinit);
}

/// Give up the window's own grid, optionally leaving it zeroed for reuse.
pub(crate) fn free_grid(window: Win, reinit: bool) {
    let mut window = window;
    if window.w_grid_alloc.handle != 0 && ui_has(kUIMultigrid) {
        ui_call_grid_destroy(window.w_grid_alloc.handle as Integer);
    }
    window.w_grid_alloc.free();
    if reinit {
        window.w_grid_alloc = ScreenGrid::empty();
    }
}

// ---------------------------------------------------------------------------
// The lists

/// Put `window` in the window list of `tabpage` (or of the current tab page) after
/// `after`, or at the front when there is no `after`.
pub(crate) fn win_append(after: Option<Win>, window: Win, tabpage: Option<TabPage>) {
    let mut window = window;
    debug_assert!(
        tabpage.is_none_or(|tp| !tp.is_current()),
        "tp == NULL || tp != curtab"
    );
    // After `None` is in front of the first.
    let before = match after {
        Some(after) => after.next(),
        None => list_first(tabpage),
    };
    let id = Some(window.id());
    window.w_next = before.map(Win::id);
    window.w_prev = after.map(Win::id);
    match after {
        Some(mut after) => after.w_next = id,
        None => set_first(tabpage, id),
    }
    match before {
        Some(mut before) => before.w_prev = id,
        None => set_last(tabpage, id),
    }
}

/// Take `window` out of the window list of `tabpage` (or of the current tab page).
pub(crate) fn win_remove(window: Win, tabpage: Option<TabPage>) {
    debug_assert!(
        tabpage.is_none_or(|tp| !tp.is_current()),
        "tp == NULL || tp != curtab"
    );
    let (prev, next) = (window.prev(), window.next());
    match prev {
        Some(mut prev) => prev.w_next = window.w_next,
        None => {
            set_first(tabpage, window.w_next);
            // Unlike `win_append`, upstream keeps the current tab page's own
            // copy of the head in step here as well.
            sync_tab_first(tabpage, window.w_next);
        }
    }
    match next {
        Some(mut next) => next.w_prev = window.w_prev,
        None => {
            set_last(tabpage, window.w_prev);
            sync_tab_last(tabpage, window.w_prev);
        }
    }
}

/// The head of `tabpage`'s window list, or of the current tab page's.
fn list_first(tabpage: Option<TabPage>) -> Option<Win> {
    match tabpage {
        Some(tp) => tp.tp_firstwin.and_then(WinId::get),
        None => crate::winlayer::first_window(),
    }
}

/// The current tab page's list head lives in the `firstwin` global; another
/// tab page's in its own `tp_firstwin`.
fn set_first(tabpage: Option<TabPage>, window: Option<WinId>) {
    match tabpage {
        Some(mut tp) => tp.tp_firstwin = window,
        None => firstwin.set(window),
    }
}

fn set_last(tabpage: Option<TabPage>, window: Option<WinId>) {
    match tabpage {
        Some(mut tp) => tp.tp_lastwin = window,
        None => lastwin.set(window),
    }
}

/// `win_remove`'s extra write, which `win_append` does not make.
fn sync_tab_first(tabpage: Option<TabPage>, window: Option<WinId>) {
    if tabpage.is_none() {
        TabPage::current().tp_firstwin = window;
    }
}

fn sync_tab_last(tabpage: Option<TabPage>, window: Option<WinId>) {
    if tabpage.is_none() {
        TabPage::current().tp_lastwin = window;
    }
}

/// Link `frp` in after `after` in its row or column.
pub(crate) fn frame_append(after: FrameRef, frp: FrameRef) {
    let (mut after, mut frp) = (after, frp);
    frp.fr_next = after.fr_next;
    after.fr_next = Some(frp.id());
    if let Some(mut next) = frp.next() {
        next.fr_prev = Some(frp.id());
    }
    frp.fr_prev = Some(after.id());
}

/// Link `frp` in before `before` in its row or column.
pub(crate) fn frame_insert(before: FrameRef, frp: FrameRef) {
    let (mut before, mut frp) = (before, frp);
    frp.fr_next = Some(before.id());
    frp.fr_prev = before.fr_prev;
    before.fr_prev = Some(frp.id());
    match frp.prev() {
        Some(mut prev) => prev.fr_next = Some(frp.id()),
        None => {
            let mut parent = frp.parent().expect("a linked frame has a parent");
            parent.fr_child = Some(frp.id());
        }
    }
}

/// Take `frp` out of its row or column, leaving its own links alone so
/// [`frame_append`]/[`frame_insert`] can put it back.
pub(crate) fn frame_remove(frp: FrameRef) {
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
