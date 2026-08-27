//! Where the cursor was -- the per-window remembered position.
//!
//! Every window remembers, for every buffer it has shown, the cursor position
//! and the topline it was at, in a `wininfo_T`.  [`buflist_setfpos`] records
//! one, [`find_wininfo`] picks the entry to restore (preferring this window,
//! then this tab page, then any), [`get_winopts`] restores the window-local
//! options and folds along with it, and [`buflist_findfmark`] answers the
//! same question for a mark rather than a window.
//!
//! The entries live in `buf_T`'s `b_wininfo`, a `klib/kvec.h` vector of
//! `WinInfo *`. [`WinInfos`] borrows its three parts -- which is a safe
//! operation once the buffer pointer is a [`Buf`] -- and hands out a slice of
//! [`Entry`], the pointer-to-one-entry newtype whose `Deref` makes every
//! field access below ordinary code.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_void;
use core::ops::{Deref, DerefMut};
use core::{ptr, slice};

use super::*;
use crate::fold::{clear_folding, clone_fold_list, delete_fold_recurse};
use crate::global_cell::GlobalCell;
use crate::main::p_fdls;
use crate::mark::mark_view_make;
use crate::memory::{xcalloc, xrealloc};
use crate::option::{clear_winopt, copy_winopt, didset_window_options};
use crate::pos::MAXLNUM;
use crate::types::{
    AdditionalData, OptInt, Timestamp, WinInfo, buf_T, colnr_T, fmark_T, fmarkv_T, garray_T,
    linenr_T, pos_T, size_t, win_T, winopt_T,
};
use crate::winfloat::win_set_minimal_style;
use crate::winlayer::{Buf, Win, windows};

// ---------------------------------------------------------------------------
// One remembered position

/// A `WinInfo` the caller has promised is live: one window's memory of one
/// buffer.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub(crate) struct Entry(*mut WinInfo);

impl Deref for Entry {
    type Target = WinInfo;

    fn deref(&self) -> &WinInfo {
        // SAFETY: the constructor's promise -- a live entry.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Entry {
    fn deref_mut(&mut self) -> &mut WinInfo {
        // SAFETY: the constructor's promise -- a live entry.
        unsafe { &mut *self.0 }
    }
}

impl Entry {
    /// A fresh, zeroed entry, as upstream's `xcalloc(1, sizeof(WinInfo))`.
    pub(crate) fn new() -> Self {
        // SAFETY: `xcalloc` aborts rather than answering null, and a zeroed
        // `WinInfo` is the initial value upstream gives one.
        Entry(unsafe { xcalloc(1, size_of::<WinInfo>()) }.cast::<WinInfo>())
    }

    #[inline(always)]
    pub(crate) fn raw(self) -> *mut WinInfo {
        self.0
    }

    /// The window this entry belongs to, null for the entry `:badd` leaves.
    pub(crate) fn window(self) -> *mut win_T {
        self.wi_win
    }

    pub(crate) fn opt(&mut self) -> *mut winopt_T {
        &raw mut self.wi_opt
    }

    pub(crate) fn folds(&mut self) -> *mut garray_T {
        &raw mut self.wi_folds
    }
}

// ---------------------------------------------------------------------------
// The vector they live in

/// `buf->b_wininfo`, borrowed part by part. Most recently used first: every
/// write puts its entry back at the front.
pub(crate) struct WinInfos<'a> {
    size: &'a mut size_t,
    capacity: &'a mut size_t,
    items: &'a mut *mut *mut WinInfo,
}

impl<'a> WinInfos<'a> {
    pub(crate) fn of(buf: &'a mut buf_T) -> Self {
        let kv = &mut buf.b_wininfo;
        WinInfos {
            size: &mut kv.size,
            capacity: &mut kv.capacity,
            items: &mut kv.items,
        }
    }

    fn entries(&self) -> &[Entry] {
        if *self.size == 0 {
            return &[];
        }
        // SAFETY: a kvec's first `size` elements are initialised, and `items`
        // is non-null once anything has been pushed. `Entry` is a transparent
        // wrapper around the element type.
        unsafe { slice::from_raw_parts(self.items.cast::<Entry>(), *self.size) }
    }

    pub(crate) fn entries_mut(&mut self) -> &mut [Entry] {
        if *self.size == 0 {
            return &mut [];
        }
        // SAFETY: as [`WinInfos::entries`].
        unsafe { slice::from_raw_parts_mut(self.items.cast::<Entry>(), *self.size) }
    }

    /// `kv_shift(v, i, 1)`: drop entry `i`, closing the gap.
    pub(crate) fn remove(&mut self, i: usize) {
        self.entries_mut().copy_within(i + 1.., i);
        *self.size -= 1;
    }

    /// `kv_resize` to make room for one more, when the array is full.
    fn reserve_one(&mut self) {
        if *self.size < *self.capacity {
            return;
        }
        *self.capacity = if *self.capacity != 0 {
            *self.capacity << 1
        } else {
            8
        };
        let bytes = size_of::<*mut WinInfo>() * *self.capacity;
        let old = self.items.cast::<c_void>();
        // SAFETY: `items` is null or this array's own allocation, and the
        // new size counts the same element type.
        *self.items = unsafe { xrealloc(old, bytes) }.cast::<*mut WinInfo>();
    }

    /// `kv_push`: append one entry.
    pub(crate) fn push(&mut self, entry: Entry) {
        self.reserve_one();
        let n = *self.size;
        // SAFETY: the array has room for `n + 1` entries, and slot `n` is
        // the free one.
        unsafe { self.items.cast::<Entry>().add(n).write(entry) };
        *self.size = n + 1;
    }

    /// `kv_pushp` followed by the memmove that opens a slot at the front.
    fn push_front(&mut self, entry: Entry) {
        self.reserve_one();
        let items = self.items.cast::<Entry>();
        let n = *self.size;
        // SAFETY: the array has room for `n + 1` entries and its first `n`
        // are initialised; this shifts them up one, leaving slot 0 free.
        unsafe { ptr::copy(items, items.add(1), n) };
        // SAFETY: slot 0 is inside the allocation and no longer read.
        unsafe { items.write(entry) };
        *self.size = n + 1;
    }
}

// ---------------------------------------------------------------------------
// The window-option and fold neighbours, wrapped
//
// Each takes a pointer into a live `WinInfo` or a live window, which the
// argument types below carry; they collapse when option.rs and fold.rs are
// themselves rewritten.

fn clear_options(opt: *mut winopt_T) {
    // SAFETY: an option block inside a live entry or window.
    unsafe { clear_winopt(opt) };
}

fn copy_options(from: *mut winopt_T, to: *mut winopt_T) {
    // SAFETY: two option blocks inside a live entry or window.
    unsafe { copy_winopt(from, to) };
}

fn delete_folds(folds: *mut garray_T) {
    // SAFETY: a fold array inside a live entry.
    unsafe { delete_fold_recurse(folds) };
}

fn clone_folds(from: *mut garray_T, to: *mut garray_T) {
    // SAFETY: two fold arrays inside a live entry or window.
    unsafe { clone_fold_list(from, to) };
}

fn clear_window_folds(mut win: Win) {
    // SAFETY: a live window.
    clear_folding(win);
}

fn didset_options(mut win: Win) {
    // SAFETY: a live window; `false` is upstream's `valid_cursor`.
    unsafe { didset_window_options(win.raw(), false) };
}

fn set_minimal_style(win: Win) {
    win_set_minimal_style(win);
}

/// The view (topline offset and skipcol) `win` would restore `pos` with.
fn view_of(win: Win, pos: pos_T) -> fmarkv_T {
    // SAFETY: a live window.
    unsafe { mark_view_make(win.raw(), pos) }
}

fn current_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

// ---------------------------------------------------------------------------
// Recording and finding a position

/// Remember `lnum`/`col` (and, with `copy_options`, the window-local options
/// and folds) as where `win` was in `buf`.
///
/// `win` is `None` for `:badd`, which records a position for no window at
/// all.
pub unsafe fn buflist_setfpos(
    mut buf: Buf,
    win: Option<Win>,
    mut lnum: linenr_T,
    col: colnr_T,
    copy_options: bool,
) {
    let raw_win = win.map_or(ptr::null_mut(), Win::raw);
    let mut list = WinInfos::of(&mut buf);

    let found = list.entries().iter().position(|e| e.window() == raw_win);
    let mut entry = match found {
        None => {
            let mut entry = Entry::new();
            entry.wi_win = raw_win;
            if lnum == 0 as linenr_T {
                // Set lnum even when it is 0.
                lnum = 1 as linenr_T;
            }
            entry
        }
        Some(i) => {
            let mut entry = list.entries()[i];
            list.remove(i);
            if copy_options && entry.wi_optset {
                clear_options(entry.opt());
                delete_folds(entry.folds());
            }
            entry
        }
    };

    if lnum != 0 as linenr_T {
        entry.wi_mark.mark.lnum = lnum;
        entry.wi_mark.mark.col = col;
        if let Some(win) = win {
            entry.wi_mark.view = view_of(win, entry.wi_mark.mark);
        }
    }
    if let Some(win) = win {
        entry.wi_changelistidx = win.w_changelistidx;
    }
    if copy_options && let Some(mut win) = win {
        // Save the window-specific option values.
        copy_options_from(&mut win, &mut entry);
    }

    list.push_front(entry);
}

/// The `copy_options` half of [`buflist_setfpos`]: `win`'s buffer-local
/// window options and folds, saved into `entry`.
fn copy_options_from(win: &mut Win, entry: &mut Entry) {
    copy_options(&raw mut win.w_onebuf_opt, entry.opt());
    entry.wi_fold_manual = win.w_fold_manual;
    clone_folds(&raw mut win.w_folds, entry.folds());
    entry.wi_optset = true;
}

/// Whether `entry` has `'diff'` set and the diff belongs to another tab page
/// -- a diff is local to a tab page.
fn wininfo_other_tab_diff(entry: Entry) -> bool {
    if entry.wi_opt.wo_diff == 0 {
        return false;
    }
    // A window of the current tab page means the buffer was in diff mode
    // here.
    !windows().any(|wp| entry.window() == wp.raw())
}

/// The entry for the current window in `buf`, or failing that the most
/// recently used one.
///
/// `need_options` skips entries whose options were never saved;
/// `skip_diff_buffer` skips windows whose `'diff'` is another tab page's.
fn find_wininfo(buf: &mut Buf, need_options: bool, skip_diff_buffer: bool) -> Option<Entry> {
    let cur = current_win().raw();
    let raw_buf = buf.raw();
    let list = WinInfos::of(buf);
    let found = list.entries().iter().find(|e| {
        e.window() == cur
            && (!skip_diff_buffer || !wininfo_other_tab_diff(**e))
            && (!need_options || e.wi_optset)
    });
    if let Some(entry) = found {
        return Some(*entry);
    }

    // No entry for curwin: use the first in the list that does not have
    // 'diff' set in another tab page. With "need_options", skip entries
    // whose options were never set -- unless the window is editing "buf",
    // so that the options can be copied from the window itself.
    if skip_diff_buffer {
        return list
            .entries()
            .iter()
            .find(|e| {
                !wininfo_other_tab_diff(**e)
                    && (!need_options
                        || e.wi_optset
                        || !e.window().is_null()
                            // SAFETY: a live window.
                            && unsafe { Win::new(e.window()) }.w_buffer == raw_buf)
            })
            .copied();
    }
    list.entries().first().copied()
}

/// Reset the current window's buffer-local options to the values last used
/// in this window; failing that, to the most recently used window's; failing
/// that, to the window's own global values.
pub unsafe fn get_winopts(mut buf: Buf) {
    let mut cur = current_win();
    clear_options(&raw mut cur.w_onebuf_opt);
    clear_window_folds(cur);

    let entry = find_wininfo(&mut buf, true, true);
    // SAFETY: a live window, or null, which `Option` keeps out of the
    // closure.
    let entry_win =
        entry.and_then(|e| (!e.window().is_null()).then(|| unsafe { Win::new(e.window()) }));

    match (entry, entry_win) {
        // The entry names another window still showing this buffer: copy
        // from the window itself, so that its current values are used.
        (Some(_), Some(mut wp))
            if wp != cur && wp.w_buffer == buf.raw() && wp.w_config.style != kWinStyleMinimal =>
        {
            copy_options(&raw mut wp.w_onebuf_opt, &raw mut cur.w_onebuf_opt);
            cur.w_fold_manual = wp.w_fold_manual;
            cur.w_foldinvalid = true;
            clone_folds(&raw mut wp.w_folds, &raw mut cur.w_folds);
        }
        (Some(mut entry), win)
            if entry.wi_optset
                && win.is_none_or(|wp| wp == cur || wp.w_config.style != kWinStyleMinimal) =>
        {
            copy_options(entry.opt(), &raw mut cur.w_onebuf_opt);
            cur.w_fold_manual = entry.wi_fold_manual;
            cur.w_foldinvalid = true;
            clone_folds(entry.folds(), &raw mut cur.w_folds);
        }
        _ => {
            copy_options(&raw mut cur.w_allbuf_opt, &raw mut cur.w_onebuf_opt);
        }
    }
    if let Some(entry) = entry {
        cur.w_changelistidx = entry.wi_changelistidx;
    }

    if cur.w_config.style == kWinStyleMinimal {
        didset_options(cur);
        set_minimal_style(cur);
    }

    // Set 'foldlevel' to 'foldlevelstart' if it's not negative.
    if p_fdls.get() >= 0 as OptInt {
        cur.w_onebuf_opt.wo_fdl = p_fdls.get();
    }
    didset_options(cur);
}

/// The mark for `buf` in the current window, or a pointer to `no_position`
/// when there is none.
pub unsafe fn buflist_findfmark(mut buf: Buf) -> *mut fmark_T {
    static no_position: GlobalCell<fmark_T> = GlobalCell::new(fmark_T {
        mark: pos_T {
            lnum: 1 as linenr_T,
            col: 0 as colnr_T,
            coladd: 0 as colnr_T,
        },
        fnum: 0,
        timestamp: 0 as Timestamp,
        view: fmarkv_T {
            topline_offset: MAXLNUM as linenr_T,
            skipcol: 0 as colnr_T,
        },
        additional_data: ptr::null_mut::<AdditionalData>(),
    });
    match find_wininfo(&mut buf, false, false) {
        // The one place the shared "no position" is handed out: callers get
        // a pointer to it exactly as they do to an entry's own mark, which
        // is why it is a cell rather than a plain `static`.
        None => no_position.ptr(),
        Some(mut entry) => &raw mut entry.wi_mark,
    }
}

pub unsafe fn buflist_findlnum(buf: Buf) -> linenr_T {
    // SAFETY: the answer is a live mark -- an entry's own, or the shared
    // "no position".
    unsafe { (*buflist_findfmark(buf)).mark.lnum }
}
