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

use super::*;
use crate::buffer::BufRef;
use crate::guard::Suppress;
use crate::main::AucmdWinVec;
use crate::normal::{set_visual_active, visual_active, with_visual_anchor};
use crate::winlayer::{Buf, Win, first_window, last_window, tabs, windows, windows_in_tab};

/// The stack of autocommand windows, one slot per nesting level.
///
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
    pub(crate) fn slot(self, idx: usize) -> *mut aucmdwin_T {
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
        unsafe {
            if (*vec).size == (*vec).capacity {
                (*vec).capacity = if (*vec).capacity != 0 {
                    (*vec).capacity << 1
                } else {
                    8
                };
                (*vec).items = xrealloc(
                    (*vec).items.cast::<::core::ffi::c_void>(),
                    ::core::mem::size_of::<aucmdwin_T>().wrapping_mul((*vec).capacity),
                )
                .cast::<aucmdwin_T>();
            }
            *(*vec).items.add((*vec).size) = aucmdwin_T {
                auc_win: ::core::ptr::null_mut(),
                auc_win_used: false,
            };
            (*vec).size = (*vec).size.wrapping_add(1);
        }
    }
}

/// Whether `win` is one of the autocommand windows currently in use.
///
/// Safe, and it keeps the raw pointer on purpose: `win` is only ever
/// *compared*, never dereferenced, so a caller may hand it an address an
/// autocommand has already freed — exactly as `win_valid` is.
pub fn is_aucmd_win(win: *mut win_T) -> bool {
    let vec = aucmd_wins();
    (0..vec.len()).any(|i| {
        // SAFETY: `i` is below `len`, so the slot is initialised.
        let entry = unsafe { &*vec.slot(i) };
        entry.auc_win_used && entry.auc_win == win
    })
}

/// Make `buf` the current buffer for the duration of an autocommand,
/// saving what it takes to undo that in `aco`.
pub unsafe fn aucmd_prepbuf(aco: *mut aco_save_T, buf: *mut buf_T) {
    let entry = |idx: usize| aucmd_wins().slot(idx);

    let same_buffer = buf == curbuf.get();

    // A window already showing `buf` is preferred: making it current
    // has the fewest side effects.  Only `curtab` is searched, which is
    // why `FOR_ALL_WINDOWS_IN_TAB(wp, curtab)` starts at `firstwin`.
    let win: *mut win_T = if same_buffer {
        curwin.get()
    } else {
        windows()
            .find(|wp| wp.w_buffer == buf)
            .map_or(::core::ptr::null_mut(), Win::raw)
    };

    // Allocate an autocommand window when there is no window to use.
    let mut need_append = true;
    let mut auc_win: *mut win_T = ::core::ptr::null_mut();
    let mut auc_idx = aucmd_wins().len();
    if win.is_null() {
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
            unsafe { win_alloc_aucmd_win(auc_idx as ::core::ffi::c_int) };
            need_append = false;
        }
        auc_win = unsafe { (*entry(auc_idx)).auc_win };
        unsafe { (*entry(auc_idx)).auc_win_used = true };
    }

    unsafe { (*aco).save_curwin_handle = cur_win().handle };
    unsafe {
        (*aco).save_prevwin_handle = if prevwin.get().is_null() {
            0
        } else {
            (*prevwin.get()).handle
        }
    };
    if buf_is_prompt(current_buf()) {
        unsafe { (*aco).save_prompt_insert = cur_buf().b_prompt_insert };
    }

    if !win.is_null() {
        unsafe { (*aco).use_aucmd_win_idx = -1 };
        curwin.set(win);
    } else {
        // No window shows "buf", so borrow the autocommand window and
        // put it in the current tab page.
        unsafe { (*aco).use_aucmd_win_idx = auc_idx as ::core::ffi::c_int };
        unsafe { (*auc_win).w_buffer = buf };
        unsafe { (*auc_win).w_s = &raw mut (*buf).b_s };
        unsafe { (*buf).b_nwindows += 1 };
        unsafe { win_init_empty(auc_win) };

        // `w_localdir`, `tp_localdir` and `globaldir` all have to be
        // null, or `win_enter_ext` chdir()s.
        unsafe { xfree((*auc_win).w_localdir.cast::<::core::ffi::c_void>()) };
        unsafe { (*auc_win).w_localdir = ::core::ptr::null_mut() };
        unsafe { (*aco).tp_localdir = (*curtab.get()).tp_localdir };
        unsafe { (*curtab.get()).tp_localdir = ::core::ptr::null_mut() };
        unsafe { (*aco).globaldir = globaldir.get() };
        globaldir.set(::core::ptr::null_mut());

        unsafe { block_autocmds() };
        if need_append {
            // Findable by handle again *before* it goes on a list, not
            // after: the list links are handles, so a window that is on one
            // has to be in the registry or the walk stops at it.
            // `aucmd_restbuf` takes it back out, after the `win_remove`.
            register_window(unsafe { Win::new(auc_win) });
            let last = last_window().map_or(::core::ptr::null_mut(), Win::raw);
            unsafe { win_append(last, auc_win, ::core::ptr::null_mut()) };
            unsafe { win_config_float(Win::new(auc_win), (*auc_win).w_config.clone()) };
        }
        // `p_acd` off keeps `win_enter_ext` out of `do_autochdir`;
        // `RedrawingDisabled` keeps it from redrawing or setting the
        // window title.
        let save_acd = p_acd.get();
        p_acd.set(0);
        let redraw_off = Suppress::redraw();
        unsafe { win_enter(auc_win, false) };
        drop(redraw_off);
        p_acd.set(save_acd);
        unsafe { unblock_autocmds() };
        curwin.set(auc_win);
    }

    curbuf.set(buf);
    unsafe { (*aco).new_curwin_handle = cur_win().handle };
    unsafe { (*aco).new_curbuf = BufRef::of_opt(current_buf()).record() };

    unsafe { (*aco).save_VIsual_active = visual_active() };
    if !same_buffer {
        // The Visual area's positions mean nothing in another buffer.
        set_visual_active(false);
    }
}

/// Undo [`aucmd_prepbuf`], restoring the window layout as far as what the
/// autocommand did to it allows.
pub unsafe fn aucmd_restbuf(aco: *mut aco_save_T) {
    if unsafe { (*aco).use_aucmd_win_idx } >= 0 {
        let idx = unsafe { (*aco).use_aucmd_win_idx } as usize;
        let awp = unsafe { (*aucmd_wins().slot(idx)).auc_win };

        // Go to `awp`.  It cannot have been closed, but the autocommand
        // may have moved it to another tab page.
        unsafe { block_autocmds() };
        if curwin.get() != awp {
            'found: for tp in tabs() {
                for wp in windows_in_tab(tp) {
                    if wp.raw() == awp {
                        if !tp.is_current() {
                            unsafe { goto_tabpage_tp(tp.raw(), true, true) };
                        }
                        unsafe { win_goto(awp) };
                        // Nothing steps the walk after those two: the
                        // `break` leaves both loops before either iterator
                        // reads a link the tab switch could have moved.
                        break 'found;
                    }
                }
            }
        }

        cur_buf().b_nwindows -= 1;
        unsafe { win_remove(curwin.get(), ::core::ptr::null_mut()) };
        // The window is given back, not freed, so it goes out of the
        // registry rather than being forgotten by a free path.
        forget_window(cur_win().handle);
        if cur_win().w_grid_alloc.is_allocated() {
            unsafe { ui_comp_remove_grid(&raw mut (*curwin.get()).w_grid_alloc) };
            ui_call_win_hide(cur_win().w_grid_alloc.handle as Integer);
            cur_win().w_grid_alloc.free();
        }

        // The window is given back, not freed: it is used again.
        unsafe { (*aucmd_wins().slot(idx)).auc_win_used = false };

        if valid_tabpage_win(curtab.get()) == 0 {
            unsafe { close_tabpage(curtab.get()) };
        }
        unsafe { unblock_autocmds() };

        let save_curwin = win_find_by_handle(unsafe { (*aco).save_curwin_handle });
        // The original window may have disappeared under the
        // autocommand; the first one is then as good as any.
        curwin.set(if save_curwin.is_null() {
            first_window().map_or(::core::ptr::null_mut(), Win::raw)
        } else {
            save_curwin
        });
        curbuf.set(cur_win().w_buffer);
        unsafe { entering_window(curwin.get()) };
        if buf_is_prompt(current_buf()) {
            cur_buf().b_prompt_insert = unsafe { (*aco).save_prompt_insert };
        }

        prevwin.set(win_find_by_handle(unsafe { (*aco).save_prevwin_handle }));
        // Free the autocommand window's `w:` variables, keeping the
        // hashtab for the next borrower.
        unsafe { vars_clear(&raw mut (*(*awp).w_vars).dv_hashtab) };
        unsafe { hash_init(&raw mut (*(*awp).w_vars).dv_hashtab) };

        // A `:lcd` inside the autocommand window has to be undone
        // *before* `tp_localdir` and `globaldir` come back.
        if !unsafe { (*awp).w_localdir.is_null() } {
            win_fix_current_dir();
        }
        unsafe { xfree((*curtab.get()).tp_localdir.cast::<::core::ffi::c_void>()) };
        unsafe { (*curtab.get()).tp_localdir = (*aco).tp_localdir };
        unsafe { xfree(globaldir.get().cast::<::core::ffi::c_void>()) };
        globaldir.set(unsafe { (*aco).globaldir });

        // The buffer's contents may have changed under the cursor.
        set_visual_active(unsafe { (*aco).save_VIsual_active });
        check_cursor(unsafe { Win::current() });
        if cur_win().w_topline > cur_buf().b_ml.ml_line_count {
            cur_win().w_topline = cur_buf().b_ml.ml_line_count;
            cur_win().w_topfill = 0;
        }
    } else {
        // Restore `curwin` by handle: a window may have been closed and
        // its memory re-used for another one.
        let save_curwin = win_find_by_handle(unsafe { (*aco).save_curwin_handle });
        if !save_curwin.is_null() {
            // Put back the buffer `curwin` was editing, if it changed
            // and we are still the same window with a valid buffer.
            // SAFETY: `aco` is the caller's, filled in by `aucmd_prepbuf`.
            let new_curbuf = BufRef::of_record(unsafe { (*aco).new_curbuf });
            if cur_win().handle == unsafe { (*aco).new_curwin_handle }
                && curbuf.get() != new_curbuf.raw()
                && new_curbuf.valid()
                && !unsafe { (*new_curbuf.raw()).b_ml.ml_mfp.is_null() }
            {
                if unsafe { (*curwin.get()).w_s } == unsafe { &raw mut (*curbuf.get()).b_s } {
                    cur_win().w_s = unsafe { &raw mut (*new_curbuf.raw()).b_s };
                }
                cur_buf().b_nwindows -= 1;
                curbuf.set(new_curbuf.raw());
                cur_win().w_buffer = curbuf.get();
                cur_buf().b_nwindows += 1;
            }

            curwin.set(save_curwin);
            curbuf.set(cur_win().w_buffer);
            prevwin.set(win_find_by_handle(unsafe { (*aco).save_prevwin_handle }));

            // The autocommand may have left the cursor where curbuf has
            // no such position.
            set_visual_active(unsafe { (*aco).save_VIsual_active });
            check_cursor(unsafe { Win::current() });
        }
    }

    set_visual_active(unsafe { (*aco).save_VIsual_active });
    // Just in case lines got deleted.
    check_cursor(unsafe { Win::current() });
    if visual_active() {
        with_visual_anchor(|anchor| unsafe { check_pos(Buf::current(), anchor) });
    }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
