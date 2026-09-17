//! The autocommand window: running a command "in" a buffer.
//!
//! [`aucmd_prepbuf`] makes `buf` current for the duration of an autocommand
//! -- entering a real window if one already shows the buffer, and otherwise
//! borrowing a hidden autocommand window and pointing it at the buffer --
//! and [`aucmd_restbuf`] puts everything back, which is the harder half:
//! the command may have closed windows, changed buffers or deleted the very
//! buffer it was given.
//!
//! The order of the impure calls in both is load-bearing and unchanged:
//! `block_autocmds` brackets the window surgery so no `BufEnter`/`WinEnter`
//! escapes it, `p_acd` and `RedrawingDisabled` bracket `win_enter` so it
//! cannot `chdir` or redraw, and `aucmd_win[]` entries are re-read after
//! every call that might have grown the vector.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::buffer::BufRef;
use crate::global_cell::GlobalCell;
use crate::guard::Suppress;
use crate::normal::{set_visual_active, visual_active, with_visual_anchor};
use crate::types::{AucmdWin, size_t};
use crate::winlayer::TabPage;
use crate::winlayer::WinId;
use crate::winlayer::{Buf, Win, first_window, last_window, tabs, windows, windows_in_tab};

/// The stack of autocommand windows, one slot per nesting level.
///
/// The autocommand-window stack itself: a grow-only vector of slots, each
/// either free or holding the window an `aucmd_prepbuf` is running in.
///
/// A hand-rolled vector rather than a `Vec` because [`AucmdWins`] hands out
/// raw slot pointers that stay valid across the autocommands run in them.
pub(crate) struct AucmdWinVec {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut AucmdWin,
}

pub(crate) static aucmd_win_vec: GlobalCell<AucmdWinVec> = GlobalCell::new(AucmdWinVec {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<AucmdWin>(),
});

/// A `Copy` handle rather than a borrow: `win_alloc_aucmd_win` writes back
/// into a slot while [`aucmd_prepbuf`] is still choosing one, and the
/// autocommands then run with a slot marked in use, so nothing here can hold
/// a `&mut` across the calls that matter.
#[derive(Clone, Copy)]
pub(crate) struct AucmdWins(*mut AucmdWinVec);

/// The one place the autocommand-window stack's address is taken.
pub(crate) fn aucmd_wins() -> AucmdWins {
    AucmdWins(aucmd_win_vec.ptr())
}

impl AucmdWins {
    /// How many slots the stack has.
    pub(crate) fn len(self) -> usize {
        // SAFETY: the only constructor names a `static`.
        unsafe { (*self.0).size }
    }

    /// Slot `idx`, which must be below [`len`](Self::len).
    pub(crate) fn slot(self, idx: usize) -> *mut AucmdWin {
        // SAFETY: as `len`; the array holds `size` initialised slots.
        unsafe { (*self.0).items.add(idx) }
    }

    /// Push an unused slot, growing the array if it is full.
    fn push_empty(self) {
        let vec = self.0;
        // SAFETY: as `len`; the array and its length are updated together,
        // and this whole run is `kv_pushp` -- every step of it reads or
        // writes through `vec`, so one region around it is as tight as it
        // gets.
        if unsafe { (*vec).size } == unsafe { (*vec).capacity } {
            let capacity = unsafe { (*vec).capacity };
            let grown = if capacity != 0 { capacity << 1 } else { 8 };
            let bytes = ::core::mem::size_of::<AucmdWin>().wrapping_mul(grown);
            unsafe {
                (*vec).capacity = grown;
                (*vec).items = xrealloc((*vec).items.cast(), bytes).cast::<AucmdWin>();
            };
        }
        let empty = AucmdWin {
            auc_win: ::core::ptr::null_mut(),
            auc_win_used: false,
        };
        unsafe {
            *(*vec).items.add((*vec).size) = empty;
            (*vec).size = (*vec).size.wrapping_add(1);
        };
    }
}

/// Whether `win` is one of the autocommand windows currently in use.
///
/// Safe, and it keeps the raw pointer on purpose: `win` is only ever
/// *compared*, never dereferenced, so a caller may hand it an address an
/// autocommand has already freed — exactly as `win_valid` is.
pub(crate) fn is_aucmd_win(win: Win) -> bool {
    let vec = aucmd_wins();
    (0..vec.len()).any(|i| {
        // SAFETY: `i` is below `len`, so the slot is initialised.
        let entry = unsafe { &*vec.slot(i) };
        entry.auc_win_used && core::ptr::eq(entry.auc_win, win.raw())
    })
}

/// Make `buffer` the current buffer for the duration of an autocommand,
/// saving what it takes to undo that in `aco`.
///
/// # Safety
///
/// `aco` must point at a live `AcoSave`, unaliased for the call.
pub unsafe fn aucmd_prepbuf(aco: *mut AcoSave, mut buffer: Buf) {
    let entry = |idx: usize| aucmd_wins().slot(idx);

    let same_buffer = buffer == Buf::current();

    // A window already showing `buffer` is preferred: making it current
    // has the fewest side effects.  Only `curtab` is searched, which is
    // why `FOR_ALL_WINDOWS_IN_TAB(wp, curtab)` starts at `firstwin`.
    let win: Option<Win> = if same_buffer {
        Some(Win::current())
    } else {
        windows().find(|wp| wp.w_buffer == buffer.raw())
    };

    // Allocate an autocommand window when there is no window to use.
    let mut need_append = true;
    let mut auc_win: *mut Window = ::core::ptr::null_mut();
    let mut auc_idx = aucmd_wins().len();
    if win.is_none() {
        auc_idx = 0;
        while auc_idx < aucmd_wins().len() && unsafe { (*entry(auc_idx)).auc_win_used } {
            auc_idx += 1;
        }

        // All of them are in use -- an autocommand fired from inside
        // another one -- so push an empty slot for this nesting level.
        if auc_idx == aucmd_wins().len() {
            aucmd_wins().push_empty();
        }

        // The slot may have been pushed empty either just now or by an
        // earlier nesting level that has since given it back.
        if unsafe { (*entry(auc_idx)).auc_win.is_null() } {
            win_alloc_aucmd_win(auc_idx as ::core::ffi::c_int);
            need_append = false;
        }
        auc_win = unsafe { (*entry(auc_idx)).auc_win };
        unsafe { (*entry(auc_idx)).auc_win_used = true };
    }

    unsafe { (*aco).save_curwin_handle = Win::current().handle };
    unsafe { (*aco).save_prevwin_handle = prevwin.get().map_or(0, WinId::handle) };
    if buf_is_prompt(current_buf()) {
        unsafe { (*aco).save_prompt_insert = Buf::current().b_prompt_insert };
    }

    if let Some(win) = win {
        unsafe { (*aco).use_aucmd_win_idx = -1 };
        win.make_current();
    } else {
        // No window shows "buf", so borrow the autocommand window and
        // put it in the current tab page.
        unsafe { (*aco).use_aucmd_win_idx = auc_idx as ::core::ffi::c_int };
        // SAFETY: the slot's window, allocated or reused just above. The
        // field writes below stay raw on purpose: `w_s` is handed a pointer
        // *into* `buffer`, and a write through a handle would pop it.
        let auc = unsafe { Win::new(auc_win) };
        unsafe { (*auc_win).w_buffer = buffer.raw() };
        unsafe { (*auc_win).w_s = &raw mut buffer.b_s };
        buffer.b_nwindows += 1;
        win_init_empty(auc);

        // `w_localdir`, `tp_localdir` and `globaldir` all have to be
        // null, or `win_enter_ext` chdir()s.
        unsafe { xfree((*auc_win).w_localdir.cast::<::core::ffi::c_void>()) };
        unsafe { (*auc_win).w_localdir = ::core::ptr::null_mut() };
        unsafe { (*aco).tp_localdir = TabPage::current().tp_localdir };
        TabPage::current().tp_localdir = ::core::ptr::null_mut();
        unsafe { (*aco).globaldir = globaldir.get() };
        globaldir.set(::core::ptr::null_mut());

        block_autocmds();
        if need_append {
            // Findable by handle again *before* it goes on a list, not
            // after: the list links are handles, so a window that is on one
            // has to be in the registry or the walk stops at it.
            // `aucmd_restbuf` takes it back out, after the `win_remove`.
            register_window(auc);
            win_append(last_window(), auc, None);
            unsafe { win_config_float(auc, (*auc_win).w_config.clone()) };
        }
        // `p_acd` off keeps `win_enter_ext` out of `do_autochdir`;
        // `RedrawingDisabled` keeps it from redrawing or setting the
        // window title.
        let save_acd = p_acd();
        P_ACD.set(false);
        let redraw_off = Suppress::redraw();
        win_enter(auc, false);
        drop(redraw_off);
        P_ACD.set(save_acd);
        unblock_autocmds();
        auc.make_current();
    }

    // SAFETY: the caller's promise -- a live buffer.
    buffer.make_current();
    unsafe { (*aco).new_curwin_handle = Win::current().handle };
    unsafe { (*aco).new_curbuf = BufRef::of_opt(current_buf()).record() };

    unsafe { (*aco).save_visual_active = visual_active() };
    if !same_buffer {
        // The Visual area's positions mean nothing in another buffer.
        set_visual_active(false);
    }
}

/// Undo [`aucmd_prepbuf`], restoring the window layout as far as what the
/// autocommand did to it allows.
///
/// # Safety
///
/// `aco` must point at a live `AcoSave`, unaliased for the call.
pub unsafe fn aucmd_restbuf(aco: *mut AcoSave) {
    if unsafe { (*aco).use_aucmd_win_idx } >= 0 {
        let idx = unsafe { (*aco).use_aucmd_win_idx } as usize;
        let awp = unsafe { (*aucmd_wins().slot(idx)).auc_win };

        // Go to `awp`.  It cannot have been closed, but the autocommand
        // may have moved it to another tab page.
        block_autocmds();
        if Win::current_raw() != awp {
            'found: for tp in tabs() {
                for wp in windows_in_tab(tp) {
                    if wp.raw() == awp {
                        if !tp.is_current() {
                            goto_tabpage_tp(tp, true, true);
                        }
                        win_goto(wp);
                        // Nothing steps the walk after those two: the
                        // `break` leaves both loops before either iterator
                        // reads a link the tab switch could have moved.
                        break 'found;
                    }
                }
            }
        }

        Buf::current().b_nwindows -= 1;
        win_remove(Win::current(), None);
        // The autocommand window, held as an address across its own
        // deregistration: it is still current and still perfectly alive, but
        // `Win::current()` answers from the registry and would say there is
        // no current window between here and the `make_current` below.
        let mut auc = Win::current();
        // The window is given back, not freed, so it goes out of the
        // registry rather than being forgotten by a free path.
        forget_window(auc.handle);
        if auc.w_grid_alloc.is_allocated() {
            unsafe { ui_comp_remove_grid(&raw mut (*auc.raw()).w_grid_alloc) };
            ui_call_win_hide(auc.w_grid_alloc.handle as Integer);
            auc.w_grid_alloc.free();
        }

        // The window is given back, not freed: it is used again.
        unsafe { (*aucmd_wins().slot(idx)).auc_win_used = false };

        if valid_tabpage_win(TabPage::current()) == 0 {
            close_tabpage(TabPage::current());
        }
        unblock_autocmds();

        let save_curwin = win_find_by_handle(unsafe { (*aco).save_curwin_handle });
        // The original window may have disappeared under the
        // autocommand; the first one is then as good as any. There being
        // neither is the editor tearing itself down, and nothing to enter.
        if let Some(landing) = save_curwin.or_else(first_window) {
            landing.make_current();
            landing.buffer().make_current();
        }
        entering_window(Win::current());
        if buf_is_prompt(current_buf()) {
            Buf::current().b_prompt_insert = unsafe { (*aco).save_prompt_insert };
        }

        prevwin.set(win_find_by_handle(unsafe { (*aco).save_prevwin_handle }).map(Win::id));
        // Free the autocommand window's `w:` variables, keeping the
        // hashtab for the next borrower.
        unsafe { vars_clear(&raw mut (*(*awp).w_vars).dv_hashtab) };
        unsafe { hash_init(&raw mut (*(*awp).w_vars).dv_hashtab) };

        // A `:lcd` inside the autocommand window has to be undone
        // *before* `tp_localdir` and `globaldir` come back.
        if !unsafe { (*awp).w_localdir.is_null() } {
            win_fix_current_dir();
        }
        unsafe { xfree(TabPage::current().tp_localdir.cast::<::core::ffi::c_void>()) };
        unsafe { TabPage::current().tp_localdir = (*aco).tp_localdir };
        unsafe { xfree(globaldir.get().cast::<::core::ffi::c_void>()) };
        globaldir.set(unsafe { (*aco).globaldir });

        // The buffer's contents may have changed under the cursor.
        set_visual_active(unsafe { (*aco).save_visual_active });
        check_cursor(Win::current());
        if Win::current().w_topline > Buf::current().b_ml.ml_line_count {
            Win::current().w_topline = Buf::current().b_ml.ml_line_count;
            Win::current().w_topfill = 0;
        }
    } else {
        // Restore `curwin` by handle: a window may have been closed and
        // its memory re-used for another one.
        let save_curwin = win_find_by_handle(unsafe { (*aco).save_curwin_handle });
        if let Some(save_curwin) = save_curwin {
            // Put back the buffer `curwin` was editing, if it changed
            // and we are still the same window with a valid buffer.
            // SAFETY: `aco` is the caller's, filled in by `aucmd_prepbuf`.
            let new_curbuf = BufRef::of_record(unsafe { (*aco).new_curbuf });
            if Win::current().handle == unsafe { (*aco).new_curwin_handle }
                && !new_curbuf.is(Buf::current_or_none())
                && let Some(mut new_curbuf) = new_curbuf.get()
                && !new_curbuf.b_ml.ml_mfp.is_null()
            {
                if Win::current().w_s == unsafe { &raw mut (*Buf::current_raw()).b_s } {
                    Win::current().w_s = &raw mut new_curbuf.b_s;
                }
                Buf::current().b_nwindows -= 1;
                new_curbuf.make_current();
                Win::current().w_buffer = new_curbuf.raw();
                Buf::current().b_nwindows += 1;
            }

            save_curwin.make_current();
            save_curwin.buffer().make_current();
            prevwin.set(win_find_by_handle(unsafe { (*aco).save_prevwin_handle }).map(Win::id));

            // The autocommand may have left the cursor where curbuf has
            // no such position.
            set_visual_active(unsafe { (*aco).save_visual_active });
            check_cursor(Win::current());
        }
    }

    set_visual_active(unsafe { (*aco).save_visual_active });
    // Just in case lines got deleted.
    check_cursor(Win::current());
    if visual_active() {
        with_visual_anchor(|anchor| check_pos(Buf::current(), anchor));
    }
}
