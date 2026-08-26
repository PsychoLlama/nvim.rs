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
use crate::guard::Suppress;
use crate::main::AucmdWinVec;
use crate::normal::{set_visual_active, visual_active, with_visual_anchor};

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
        // SAFETY: as `len`; the array and its length are updated together.
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
pub unsafe fn is_aucmd_win(win: *mut win_T) -> bool {
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
    unsafe {
        let entry = |idx: usize| aucmd_wins().slot(idx);

        let same_buffer = buf == curbuf.get();

        // A window already showing `buf` is preferred: making it current
        // has the fewest side effects.  Only `curtab` is searched, which is
        // why `FOR_ALL_WINDOWS_IN_TAB(wp, curtab)` starts at `firstwin`.
        let mut win: *mut win_T = ::core::ptr::null_mut();
        if same_buffer {
            win = curwin.get();
        } else {
            let mut wp = firstwin.get();
            while !wp.is_null() {
                if (*wp).w_buffer == buf {
                    win = wp;
                    break;
                }
                wp = (*wp).w_next;
            }
        }

        // Allocate an autocommand window when there is no window to use.
        let mut need_append = true;
        let mut auc_win: *mut win_T = ::core::ptr::null_mut();
        let mut auc_idx = aucmd_wins().len();
        if win.is_null() {
            auc_idx = 0;
            while auc_idx < aucmd_wins().len() && (*entry(auc_idx)).auc_win_used {
                auc_idx += 1;
            }

            // All of them are in use -- an autocommand fired from inside
            // another one -- so push an empty slot for this nesting level.
            if auc_idx == aucmd_wins().len() {
                aucmd_wins().push_empty();
            }

            // The slot may have been pushed empty either just now or by an
            // earlier nesting level that has since given it back.
            if (*entry(auc_idx)).auc_win.is_null() {
                win_alloc_aucmd_win(auc_idx as ::core::ffi::c_int);
                need_append = false;
            }
            auc_win = (*entry(auc_idx)).auc_win;
            (*entry(auc_idx)).auc_win_used = true;
        }

        (*aco).save_curwin_handle = (*curwin.get()).handle;
        (*aco).save_prevwin_handle = if prevwin.get().is_null() {
            0
        } else {
            (*prevwin.get()).handle
        };
        if bt_prompt(curbuf.get()) {
            (*aco).save_prompt_insert = (*curbuf.get()).b_prompt_insert;
        }

        if !win.is_null() {
            (*aco).use_aucmd_win_idx = -1;
            curwin.set(win);
        } else {
            // No window shows "buf", so borrow the autocommand window and
            // put it in the current tab page.
            (*aco).use_aucmd_win_idx = auc_idx as ::core::ffi::c_int;
            (*auc_win).w_buffer = buf;
            (*auc_win).w_s = &raw mut (*buf).b_s;
            (*buf).b_nwindows += 1;
            win_init_empty(auc_win);

            // `w_localdir`, `tp_localdir` and `globaldir` all have to be
            // null, or `win_enter_ext` chdir()s.
            xfree((*auc_win).w_localdir.cast::<::core::ffi::c_void>());
            (*auc_win).w_localdir = ::core::ptr::null_mut();
            (*aco).tp_localdir = (*curtab.get()).tp_localdir;
            (*curtab.get()).tp_localdir = ::core::ptr::null_mut();
            (*aco).globaldir = globaldir.get();
            globaldir.set(::core::ptr::null_mut());

            block_autocmds();
            if need_append {
                win_append(lastwin.get(), auc_win, ::core::ptr::null_mut());
                // The window is findable by handle again for as long as it
                // is on a list; `aucmd_restbuf` takes it back out.
                register_window(Win::new(auc_win));
                win_config_float(auc_win, (*auc_win).w_config.clone());
            }
            // `p_acd` off keeps `win_enter_ext` out of `do_autochdir`;
            // `RedrawingDisabled` keeps it from redrawing or setting the
            // window title.
            let save_acd = p_acd.get();
            p_acd.set(0);
            let redraw_off = Suppress::redraw();
            win_enter(auc_win, false);
            drop(redraw_off);
            p_acd.set(save_acd);
            unblock_autocmds();
            curwin.set(auc_win);
        }

        curbuf.set(buf);
        (*aco).new_curwin_handle = (*curwin.get()).handle;
        set_bufref(&raw mut (*aco).new_curbuf, curbuf.get());

        (*aco).save_VIsual_active = visual_active();
        if !same_buffer {
            // The Visual area's positions mean nothing in another buffer.
            set_visual_active(false);
        }
    }
}

/// Undo [`aucmd_prepbuf`], restoring the window layout as far as what the
/// autocommand did to it allows.
pub unsafe fn aucmd_restbuf(aco: *mut aco_save_T) {
    unsafe {
        if (*aco).use_aucmd_win_idx >= 0 {
            let idx = (*aco).use_aucmd_win_idx as usize;
            let awp = (*aucmd_wins().slot(idx)).auc_win;

            // Go to `awp`.  It cannot have been closed, but the autocommand
            // may have moved it to another tab page.
            block_autocmds();
            if curwin.get() != awp {
                let mut tp = first_tabpage.get();
                'found: while !tp.is_null() {
                    let mut wp = if tp == curtab.get() {
                        firstwin.get()
                    } else {
                        (*tp).tp_firstwin
                    };
                    while !wp.is_null() {
                        if wp == awp {
                            if tp != curtab.get() {
                                goto_tabpage_tp(tp, true, true);
                            }
                            win_goto(awp);
                            break 'found;
                        }
                        wp = (*wp).w_next;
                    }
                    tp = (*tp).tp_next;
                }
            }

            (*curbuf.get()).b_nwindows -= 1;
            win_remove(curwin.get(), ::core::ptr::null_mut());
            // The window is given back, not freed, so it goes out of the
            // registry rather than being forgotten by a free path.
            forget_window((*curwin.get()).handle);
            if (*curwin.get()).w_grid_alloc.is_allocated() {
                ui_comp_remove_grid(&raw mut (*curwin.get()).w_grid_alloc);
                ui_call_win_hide((*curwin.get()).w_grid_alloc.handle as Integer);
                (*curwin.get()).w_grid_alloc.free();
            }

            // The window is given back, not freed: it is used again.
            (*aucmd_wins().slot(idx)).auc_win_used = false;

            if valid_tabpage_win(curtab.get()) == 0 {
                close_tabpage(curtab.get());
            }
            unblock_autocmds();

            let save_curwin = win_find_by_handle((*aco).save_curwin_handle);
            // The original window may have disappeared under the
            // autocommand; the first one is then as good as any.
            curwin.set(if save_curwin.is_null() {
                firstwin.get()
            } else {
                save_curwin
            });
            curbuf.set((*curwin.get()).w_buffer);
            entering_window(curwin.get());
            if bt_prompt(curbuf.get()) {
                (*curbuf.get()).b_prompt_insert = (*aco).save_prompt_insert;
            }

            prevwin.set(win_find_by_handle((*aco).save_prevwin_handle));
            // Free the autocommand window's `w:` variables, keeping the
            // hashtab for the next borrower.
            vars_clear(&raw mut (*(*awp).w_vars).dv_hashtab);
            hash_init(&raw mut (*(*awp).w_vars).dv_hashtab);

            // A `:lcd` inside the autocommand window has to be undone
            // *before* `tp_localdir` and `globaldir` come back.
            if !(*awp).w_localdir.is_null() {
                win_fix_current_dir();
            }
            xfree((*curtab.get()).tp_localdir.cast::<::core::ffi::c_void>());
            (*curtab.get()).tp_localdir = (*aco).tp_localdir;
            xfree(globaldir.get().cast::<::core::ffi::c_void>());
            globaldir.set((*aco).globaldir);

            // The buffer's contents may have changed under the cursor.
            set_visual_active((*aco).save_VIsual_active);
            check_cursor(curwin.get());
            if (*curwin.get()).w_topline > (*curbuf.get()).b_ml.ml_line_count {
                (*curwin.get()).w_topline = (*curbuf.get()).b_ml.ml_line_count;
                (*curwin.get()).w_topfill = 0;
            }
        } else {
            // Restore `curwin` by handle: a window may have been closed and
            // its memory re-used for another one.
            let save_curwin = win_find_by_handle((*aco).save_curwin_handle);
            if !save_curwin.is_null() {
                // Put back the buffer `curwin` was editing, if it changed
                // and we are still the same window with a valid buffer.
                if (*curwin.get()).handle == (*aco).new_curwin_handle
                    && curbuf.get() != (*aco).new_curbuf.br_buf
                    && bufref_valid(&raw mut (*aco).new_curbuf)
                    && !(*(*aco).new_curbuf.br_buf).b_ml.ml_mfp.is_null()
                {
                    if (*curwin.get()).w_s == &raw mut (*curbuf.get()).b_s {
                        (*curwin.get()).w_s = &raw mut (*(*aco).new_curbuf.br_buf).b_s;
                    }
                    (*curbuf.get()).b_nwindows -= 1;
                    curbuf.set((*aco).new_curbuf.br_buf);
                    (*curwin.get()).w_buffer = curbuf.get();
                    (*curbuf.get()).b_nwindows += 1;
                }

                curwin.set(save_curwin);
                curbuf.set((*curwin.get()).w_buffer);
                prevwin.set(win_find_by_handle((*aco).save_prevwin_handle));

                // The autocommand may have left the cursor where curbuf has
                // no such position.
                set_visual_active((*aco).save_VIsual_active);
                check_cursor(curwin.get());
            }
        }

        set_visual_active((*aco).save_VIsual_active);
        // Just in case lines got deleted.
        check_cursor(curwin.get());
        if visual_active() {
            with_visual_anchor(|anchor| check_pos(curbuf.get(), anchor));
        }
    }
}
