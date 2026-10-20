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
//! cannot `chdir` or redraw, and `aucmd_win[]` entries are re-read by
//! index after every call that might have grown the vector.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::buffer::BufRef;
use crate::global_cell::GlobalCell;
use crate::guard::Suppress;
use crate::normal::{set_visual_active, visual_active, with_visual_anchor};
use crate::types::AucmdWin;
use crate::winlayer::TabPage;
use crate::winlayer::WinId;
use crate::winlayer::{Buf, Win, first_window, last_window, tabs, windows, windows_in_tab};

/// The stack of autocommand windows, one slot per nesting level: a
/// grow-only vector of slots, each either free or holding the window an
/// `aucmd_prepbuf` is running in.
///
/// Reached a slot at a time, by index: `win_alloc_aucmd_win` writes back
/// into a slot while [`aucmd_prepbuf`] is still choosing one, and the
/// autocommands then run with a slot marked in use, so nothing holds a
/// borrow of the vector across a call.
static aucmd_win_vec: GlobalCell<Vec<AucmdWin>> = GlobalCell::new(Vec::new());

/// How many slots the stack has.
pub(crate) fn aucmd_win_count() -> usize {
    aucmd_win_vec.with(Vec::len)
}

/// The window slot `idx` holds, if any; `idx` must be below
/// [`aucmd_win_count`].
pub(crate) fn aucmd_window(idx: usize) -> Option<Win> {
    aucmd_win_vec.with(|slots| slots[idx].auc_win)
}

/// Give slot `idx` its window: `win_alloc_aucmd_win`'s write-back.
pub(crate) fn set_aucmd_window(idx: usize, win: Win) {
    aucmd_win_vec.with_mut(|slots| slots[idx].auc_win = Some(win));
}

/// Whether slot `idx` is in use.
fn aucmd_slot_used(idx: usize) -> bool {
    aucmd_win_vec.with(|slots| slots[idx].auc_win_used)
}

/// Mark slot `idx` in use, or free.
fn set_aucmd_slot_used(idx: usize, used: bool) {
    aucmd_win_vec.with_mut(|slots| slots[idx].auc_win_used = used);
}

/// Whether `win` is one of the autocommand windows currently in use.
///
/// `win` is only ever *compared*, never dereferenced, so a caller may hand
/// it an address an autocommand has already freed -- exactly as
/// `win_valid` is.
pub(crate) fn is_aucmd_win(win: Win) -> bool {
    aucmd_win_vec.with(|slots| {
        slots
            .iter()
            .any(|entry| entry.auc_win_used && entry.auc_win == Some(win))
    })
}

/// Make `buffer` the current buffer for the duration of an autocommand,
/// saving what it takes to undo that in `aco`.
pub fn aucmd_prepbuf(aco: &mut AcoSave, mut buffer: Buf) {
    let same_buffer = buffer == Buf::current();

    // A window already showing `buffer` is preferred: making it current
    // has the fewest side effects.  Only `curtab` is searched, which is
    // why `FOR_ALL_WINDOWS_IN_TAB(wp, curtab)` starts at `firstwin`.
    let win: Option<Win> = if same_buffer {
        Some(Win::current())
    } else {
        windows().find(|wp| wp.w_buffer == buffer)
    };

    // Allocate an autocommand window when there is no window to use.
    let mut need_append = true;
    let mut auc_win: Option<Win> = None;
    let mut auc_idx = aucmd_win_count();
    if win.is_none() {
        auc_idx = 0;
        while auc_idx < aucmd_win_count() && aucmd_slot_used(auc_idx) {
            auc_idx += 1;
        }

        // All of them are in use -- an autocommand fired from inside
        // another one -- so push an empty slot for this nesting level.
        if auc_idx == aucmd_win_count() {
            aucmd_win_vec.with_mut(|slots| {
                slots.push(AucmdWin {
                    auc_win: None,
                    auc_win_used: false,
                });
            });
        }

        // The slot may have been pushed empty either just now or by an
        // earlier nesting level that has since given it back.
        if aucmd_window(auc_idx).is_none() {
            win_alloc_aucmd_win(::core::ffi::c_int::try_from(auc_idx).expect("a nesting depth"));
            need_append = false;
        }
        auc_win = aucmd_window(auc_idx);
        set_aucmd_slot_used(auc_idx, true);
    }

    aco.save_curwin_handle = Win::current().handle;
    aco.save_prevwin_handle = prevwin.get().map_or(0, WinId::handle);
    if buf_is_prompt(current_buf()) {
        aco.save_prompt_insert = Buf::current().b_prompt_insert;
    }

    if let Some(win) = win {
        aco.use_aucmd_win_idx = -1;
        win.make_current();
    } else {
        // No window shows "buf", so borrow the autocommand window and
        // put it in the current tab page.
        aco.use_aucmd_win_idx = ::core::ffi::c_int::try_from(auc_idx).expect("a nesting depth");
        // The slot's window, allocated or reused just above.
        let mut auc = auc_win.expect("the slot was given a window above");
        auc.w_buffer = buffer;
        auc.w_s = buffer.syntax_block();
        buffer.b_nwindows += 1;
        win_init_empty(auc);

        // `w_localdir`, `tp_localdir` and `globaldir` all have to be
        // unset, or `win_enter_ext` chdir()s.
        auc.w_localdir = None;
        aco.tp_localdir = TabPage::current().tp_localdir.take();
        aco.globaldir = globaldir.take();

        block_autocmds();
        if need_append {
            // Findable by handle again *before* it goes on a list, not
            // after: the list links are handles, so a window that is on one
            // has to be in the registry or the walk stops at it.
            // `aucmd_restbuf` takes it back out, after the `win_remove`.
            register_window(auc);
            win_append(last_window(), auc, None);
            let config = auc.w_config.clone();
            win_config_float(auc, config);
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

    buffer.make_current();
    aco.new_curwin_handle = Win::current().handle;
    aco.new_curbuf = BufRef::of_opt(current_buf()).record();

    aco.save_visual_active = visual_active();
    if !same_buffer {
        // The Visual area's positions mean nothing in another buffer.
        set_visual_active(false);
    }
}

/// Undo [`aucmd_prepbuf`], restoring the window layout as far as what the
/// autocommand did to it allows.
pub fn aucmd_restbuf(aco: &mut AcoSave) {
    if let Ok(idx) = usize::try_from(aco.use_aucmd_win_idx) {
        let awp = aucmd_window(idx);
        let aw = awp.expect("a slot in use has its window");
        // Go to `awp`.  It cannot have been closed, but the autocommand
        // may have moved it to another tab page.
        block_autocmds();
        if Win::current_or_none() != awp {
            'found: for tp in tabs() {
                for wp in windows_in_tab(tp) {
                    if Some(wp) == awp {
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
            ui_comp_remove_grid(crate::ui_compositor::win_grid(auc));
            ui_call_win_hide(Integer::from(auc.w_grid_alloc.handle));
            auc.w_grid_alloc.free();
        }

        // The window is given back, not freed: it is used again.
        set_aucmd_slot_used(idx, false);

        if valid_tabpage_win(TabPage::current()) == 0 {
            close_tabpage(TabPage::current());
        }
        unblock_autocmds();

        let save_curwin = win_find_by_handle(aco.save_curwin_handle);
        // The original window may have disappeared under the
        // autocommand; the first one is then as good as any. There being
        // neither is the editor tearing itself down, and nothing to enter.
        if let Some(landing) = save_curwin.or_else(first_window) {
            landing.make_current();
            landing.buffer().make_current();
        }
        entering_window(Win::current());
        if buf_is_prompt(current_buf()) {
            Buf::current().b_prompt_insert = aco.save_prompt_insert;
        }

        prevwin.set(win_find_by_handle(aco.save_prevwin_handle).map(Win::id));
        // Free the autocommand window's `w:` variables, keeping the
        // hashtab for the next borrower.
        if let Some(vars) = aw.w_winvar.di_tv.dict_handle() {
            vars_clear(&vars);
        }

        // A `:lcd` inside the autocommand window has to be undone
        // *before* `tp_localdir` and `globaldir` come back.
        if aw.w_localdir.is_some() {
            win_fix_current_dir();
        }
        TabPage::current().tp_localdir = aco.tp_localdir.take();
        globaldir.set(aco.globaldir.take());

        // The buffer's contents may have changed under the cursor.
        set_visual_active(aco.save_visual_active);
        check_cursor(Win::current());
        if Win::current().w_topline > Buf::current().b_ml.ml_line_count {
            Win::current().w_topline = Buf::current().b_ml.ml_line_count;
            Win::current().w_topfill = 0;
        }
    } else {
        // Restore `curwin` by handle: a window may have been closed and
        // its memory re-used for another one.
        let save_curwin = win_find_by_handle(aco.save_curwin_handle);
        if let Some(save_curwin) = save_curwin {
            // Put back the buffer `curwin` was editing, if it changed
            // and we are still the same window with a valid buffer.
            let new_curbuf = BufRef::of_record(aco.new_curbuf);
            if Win::current().handle == aco.new_curwin_handle
                && !new_curbuf.is(Buf::current_or_none())
                && let Some(new_curbuf) = new_curbuf.get()
                && new_curbuf.b_ml.ml_mfp.is_some()
            {
                if Win::current().w_s == Buf::current().syntax_block() {
                    Win::current().w_s = new_curbuf.syntax_block();
                }
                Buf::current().b_nwindows -= 1;
                new_curbuf.make_current();
                Win::current().w_buffer = new_curbuf;
                Buf::current().b_nwindows += 1;
            }

            save_curwin.make_current();
            save_curwin.buffer().make_current();
            prevwin.set(win_find_by_handle(aco.save_prevwin_handle).map(Win::id));

            // The autocommand may have left the cursor where curbuf has
            // no such position.
            set_visual_active(aco.save_visual_active);
            check_cursor(Win::current());
        }
    }

    set_visual_active(aco.save_visual_active);
    // Just in case lines got deleted.
    check_cursor(Win::current());
    if visual_active() {
        with_visual_anchor(|anchor| check_pos(Buf::current(), anchor));
    }
}

/// A buffer made current for the length of a value: [`aucmd_prepbuf`] when
/// it is made, [`aucmd_restbuf`] when it is dropped.
pub(crate) struct AucmdBuf {
    aco: AcoSave,
}

impl AucmdBuf {
    /// Make `buffer` current — in a window showing it, or in an autocommand
    /// window — saving what is needed to put things back.
    pub(crate) fn enter(buffer: Buf) -> AucmdBuf {
        let mut aco = AcoSave::default();
        aucmd_prepbuf(&mut aco, buffer);
        AucmdBuf { aco }
    }
}

impl Drop for AucmdBuf {
    fn drop(&mut self) {
        aucmd_restbuf(&mut self.aco);
    }
}
