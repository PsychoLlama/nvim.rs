//! Unloading, deleting and wiping a buffer -- `close_buffer()`.
//!
//! [`close_buffer`] is the one entry point for all three: fire
//! `BufUnload`/`BufDelete`/`BufWipeout`, free the memline, the undo tree, the
//! marks, the folds and the extmarks, and -- for a wipe -- unlink the buffer
//! from the list and free it.  Every one of those autocommands may have freed
//! the buffer in hand, which is why so much of this is written around
//! [`BufRef`] re-validation.  [`buf_freeall`] is the loaded-state teardown
//! the reload path shares.
//!
//! The rule the file follows: **nothing derived from `buf` or `win` survives
//! a call that fires an autocommand.**  A [`BufRef`] is taken before the
//! first one and re-`get`ed after each; the window is kept as a raw pointer
//! and re-checked with `win_valid_any_tab`, which only ever *compares* it.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::ptr;

use super::*;
use crate::autocmd::{
    EVENT_BUFDELETE, EVENT_BUFHIDDEN, EVENT_BUFUNLOAD, EVENT_BUFWINLEAVE, EVENT_BUFWIPEOUT,
    aubuflocal_remove,
};
use crate::buffer_updates::{buf_free_callbacks, buf_updates_unload};
use crate::change::deleted_lines_mark;
use crate::diff::{diff_buf_delete, diffopt_hiddenoff};
use crate::eval::typval::{callback_free, tv_dict_add, tv_dict_item_copy};
use crate::eval::vars::{unref_var_dict, vars_clear};
use crate::extmark::extmark_free_all;
use crate::garray::ga_clear;
use crate::hashtab::{hash_find, hash_init, hash_remove};
use crate::main::{
    VIsual_active, au_pending_free_buf, autocmd_busy, buffer_handles, curbuf, curtab, curwin,
    e_auabort, exiting, firstbuf, lastbuf, updating_screen,
};
use crate::map::map_del_int_ptr_t;
use crate::mapping::map_clear_mode;
use crate::mark::{clear_fmark, free_fmark, mark_adjust_buf, mark_forget_file, set_last_cursor};
use crate::memline::ml_close;
use crate::pos::MAXLNUM;
use crate::semsg_c;
use crate::state::MAP_ALL_MODES;
use crate::syntax::syntax_clear;
use crate::terminal::terminal_close;
use crate::types::{
    Callback, Timestamp, WinInfo, colnr_T, dictitem_T, fmark_T, fmarkv_T, garray_T, hashtab_T,
    linenr_T, memfile_T, pos_T, synblock_T, tabpage_T, win_T,
};
use crate::undo::u_clearallandblockfree;
use crate::usercmd::uc_clear;
use crate::window::{free_wininfo, goto_tabpage_win, one_window, win_valid_any_tab};
use crate::winlayer::{Buf, TabPage, Win, tab_windows, windows};

/// A mark that has never been set, as `CLEAR_FIELD()` leaves one: all zero,
/// which is *not* `INIT_FMARK` (that seeds `topline_offset` with `MAXLNUM`).
const ZERO_FMARK: fmark_T = fmark_T {
    mark: pos_T {
        lnum: 0 as linenr_T,
        col: 0 as colnr_T,
        coladd: 0 as colnr_T,
    },
    fnum: 0,
    timestamp: 0 as Timestamp,
    view: fmarkv_T {
        topline_offset: 0 as linenr_T,
        skipcol: 0 as colnr_T,
    },
    additional_data: ptr::null_mut(),
};

// ---------------------------------------------------------------------------
// The neighbours, wrapped
//
// One safe wrapper per distinct neighbour, each taking the live buffer or
// window the callee needs; the call sites below are then ordinary code. They
// collapse as the callee modules are themselves rewritten.

/// The window `win` names, if autocommands have not closed it.
///
/// `win_valid_any_tab` walks the window lists comparing pointers and never
/// dereferences its argument, so asking about a possibly-freed window is a
/// safe operation -- and a hit means it is live.
fn valid_win(win: *mut win_T) -> Option<Win> {
    // SAFETY: the pointer is only compared; a hit means a live window.
    unsafe { win_valid_any_tab(win).then(|| Win::new(win)) }
}

/// Whether `win` is the only non-floating window of its tab page.
fn is_only_window(win: *mut win_T) -> bool {
    // SAFETY: as [`valid_win`], `one_window` only compares the pointer.
    unsafe { one_window(win, ptr::null_mut::<tabpage_T>()) }
}

/// Make `win` in `tp` current again, without firing autocommands.
fn goto_win(mut tp: TabPage, mut win: Win) {
    // SAFETY: a live tab page and a live window.
    unsafe { goto_tabpage_win(tp.raw(), win.raw()) };
}

/// Remember `win`'s cursor as the buffer's last position.
fn remember_last_cursor(mut win: Win) {
    // SAFETY: a live window.
    unsafe { set_last_cursor(win.raw()) };
}

/// Forget every mark and jump-list entry naming buffer `fnum` in `win`.
fn forget_file(mut win: Win, fnum: c_int) {
    // SAFETY: a live window.
    unsafe { mark_forget_file(win.raw(), fnum) };
}

fn semsg_name(fmt: *mut c_char, name: *const c_char) {
    // SAFETY: a translated format taking one string, and a name that is
    // NUL-terminated or a literal.
    let _: bool = unsafe { semsg_c!(fmt, name) };
}

fn detach_updates(mut buf: Buf) {
    // SAFETY: a live buffer; `false` is upstream's `send_closing`.
    unsafe { buf_updates_unload(buf.raw(), false) };
}

fn free_update_callbacks(mut buf: Buf) {
    // SAFETY: a live buffer.
    unsafe { buf_free_callbacks(buf.raw()) };
}

fn diff_forget(mut buf: Buf) {
    // SAFETY: a live buffer.
    unsafe { diff_buf_delete(buf.raw()) };
}

/// Whether `'diffopt'` contains `hiddenoff`.
fn diff_hidden_off() -> bool {
    diffopt_hiddenoff()
}

fn free_extmarks(mut buf: Buf) {
    // SAFETY: a live buffer.
    unsafe { extmark_free_all(buf.raw()) };
}

fn free_user_commands(ucmds: &mut garray_T) {
    // SAFETY: a growable array of user commands inside a live buffer.
    unsafe { uc_clear(ucmds) };
}

fn free_garray(ga: &mut garray_T) {
    // SAFETY: a growable array inside a live buffer.
    unsafe { ga_clear(ga) };
}

/// Drop every buffer-local mapping (`abbrev` picks the abbreviation table).
fn clear_mappings(mut buf: Buf, abbrev: bool) {
    // SAFETY: a live buffer.
    unsafe { map_clear_mode(buf.raw(), MAP_ALL_MODES, true, abbrev) };
}

fn free_callback(cb: &mut Callback) {
    // SAFETY: a callback inside a live buffer.
    unsafe { callback_free(cb) };
}

fn clear_mark(mark: &mut fmark_T) {
    // SAFETY: a mark inside a live buffer; `0` is upstream's timestamp.
    unsafe { clear_fmark(mark, 0 as Timestamp) };
}

fn drop_mark(mark: fmark_T) {
    // SAFETY: a mark copied out of a live buffer.
    unsafe { free_fmark(mark) };
}

/// Move every mark in `buf` up by `count` lines from line 1 -- what an
/// emptied buffer needs so a reload starts from a clean slate.
fn forget_lines(mut buf: Buf, count: linenr_T) {
    let (raw, last) = (buf.raw(), MAXLNUM as linenr_T);
    // SAFETY: a live buffer.
    unsafe {
        mark_adjust_buf(
            raw,
            1,
            count,
            last,
            -count,
            false,
            kMarkAdjustNormal,
            kExtmarkNoUndo,
        )
    };
}

fn free_undo(mut buf: Buf) {
    // SAFETY: a live buffer.
    unsafe { u_clearallandblockfree(buf.raw()) };
}

fn clear_syntax(syn: &mut synblock_T) {
    // SAFETY: the syntax block of a live buffer.
    unsafe { syntax_clear(syn) };
}

/// Close the memline and delete the swap file.
fn close_memline(mut buf: Buf) {
    // SAFETY: a live buffer; `true` is upstream's `del_file`.
    unsafe { ml_close(buf.raw(), true_0) };
}

fn mark_lines_deleted(count: linenr_T) {
    // SAFETY: reads the current buffer, which the caller has just emptied.
    unsafe { deleted_lines_mark(1 as linenr_T, count as c_int) };
}

fn free_entry(entry: *mut WinInfo) {
    // SAFETY: an entry of a live buffer's `b_wininfo`.
    unsafe { free_wininfo(entry) };
}

/// `buf->b_vars->dv_hashtab`.
fn buf_vars(mut buf: Buf) -> *mut hashtab_T {
    // SAFETY: a live buffer's variable dictionary is live.
    unsafe { &raw mut (*buf.b_vars).dv_hashtab }
}

/// Free every buffer-local variable.
///
/// `b:changedtick` lives in a field of `buf_T` rather than in the dictionary's
/// own storage, so it is removed from the hash table first: clearing it would
/// go through `clear_tv()` and zero the counter.
fn clear_buf_vars(buf: Buf) {
    let vars = buf_vars(buf);
    // SAFETY: the hash table of a live buffer's variable dictionary; the
    // `changedtick` entry is put there when the buffer is created.
    unsafe {
        let changedtick_hi = hash_find(vars, c"changedtick".as_ptr());
        debug_assert!(!changedtick_hi.is_null(), "changedtick_hi != NULL");
        hash_remove(vars, changedtick_hi);
        vars_clear(vars);
        hash_init(vars);
    }
}

/// Hand `b:changedtick` to the dictionary before the buffer goes away, for the
/// script that is still holding a reference to it.
fn rescue_changedtick(mut buf: Buf) {
    let (vars, di) = (buf.b_vars, &raw mut buf.changedtick_di as *mut dictitem_T);
    // SAFETY: a live buffer's dictionary, and its own `changedtick` item.
    unsafe { tv_dict_add(vars, tv_dict_item_copy(di)) };
}

fn release_vars(mut buf: Buf) {
    // SAFETY: a live buffer's variable dictionary.
    unsafe { unref_var_dict(buf.b_vars) };
}

fn forget_autocmds(mut buf: Buf) {
    // SAFETY: a live buffer.
    unsafe { aubuflocal_remove(buf.raw()) };
}

/// Drop the buffer's handle from the global handle map.
fn forget_handle(fnum: c_int) {
    // SAFETY: the map is the editor's own; a null out-parameter means "do
    // not report the removed value".
    buffer_handles.with_mut(|map| unsafe { map_del_int_ptr_t(map, fnum, ptr::null_mut()) });
}

// ---------------------------------------------------------------------------
// What `close_buffer` has been asked to do

/// How far [`close_buffer`] goes, once `'bufhidden'` has had its say.
///
/// The three are cumulative: a wipe is a delete is an unload.
#[derive(Clone, Copy)]
struct Disposition {
    unload: bool,
    del: bool,
    wipe: bool,
}

impl Disposition {
    /// The `action` the caller asked for, forced further by `'bufhidden'` --
    /// and forced all the way for a terminal buffer, which can only be wiped.
    fn of(mut buf: Buf, action: c_int) -> Self {
        let mut it = Disposition {
            unload: action != 0,
            del: action == DOBUF_DEL as c_int || action == DOBUF_WIPE as c_int,
            wipe: action == DOBUF_WIPE as c_int,
        };
        // The caller must take care of NOT deleting/freeing when 'bufhidden'
        // is "hide" (otherwise we could never free or delete a buffer).
        if buf.terminal.is_null() {
            // SAFETY: `'bufhidden'` is a NUL-terminated option value.
            match unsafe { *buf.b_p_bh } as u8 {
                b'd' => (it.del, it.unload) = (true, true),
                b'w' => (it.del, it.unload, it.wipe) = (true, true, true),
                b'u' => it.unload = true,
                _ => {}
            }
        } else if it.unload || it.del || it.wipe {
            it = Disposition {
                unload: true,
                del: true,
                wipe: true,
            };
        }
        it
    }

    /// The `BFA_*` set [`buf_freeall`] takes for this disposition.
    fn free_flags(self, ignore_abort: bool) -> c_int {
        (if self.del { BFA_DEL as c_int } else { 0 })
            + (if self.wipe { BFA_WIPE as c_int } else { 0 })
            + (if ignore_abort {
                BFA_IGNORE_ABORT as c_int
            } else {
                0
            })
    }
}

// ---------------------------------------------------------------------------
// Refusing to unload

/// Whether `buf` may be unloaded, with the error message when it may not.
///
/// A buffer is locked while it is halfway through a command that relies on
/// it, and cannot be unloaded from under a redraw that is showing it.
pub(crate) fn can_unload_buffer(mut buf: Buf) -> bool {
    let mut can_unload = buf.b_locked == 0;

    if can_unload && updating_screen.get() {
        can_unload = !windows().any(|wp| wp.w_buffer == buf.raw());
    }
    // Don't unload the buffer while it's still being saved
    if can_unload && buf.b_saving {
        can_unload = false;
    }

    if !can_unload {
        let fname = if buf.b_fname.is_null() {
            buf.b_ffname
        } else {
            buf.b_fname
        };
        let fmt = tr_raw(e_attempt_to_delete_buffer_that_is_in_use_str.as_ptr());
        let name = if fname.is_null() {
            c"[No Name]".as_ptr()
        } else {
            fname.cast_const()
        };
        semsg_name(fmt, name);
    }
    can_unload
}

pub unsafe fn buf_close_terminal(buf: *mut buf_T) {
    // SAFETY: the caller's promise -- a live buffer.
    let mut buf = unsafe { Buf::new(buf) };
    debug_assert!(!buf.terminal.is_null(), "buf->terminal");
    buf.b_locked += 1;
    // SAFETY: a live terminal, the assertion above having ruled out null.
    unsafe { terminal_close(&raw mut buf.terminal, -1) };
    buf.b_locked -= 1;
}

// ---------------------------------------------------------------------------
// Closing the link to a buffer

/// Close the link between `win` and `buf`, and act on `action` once no window
/// is left showing it.
///
/// `action` is 0 (the buffer becomes hidden), `DOBUF_UNLOAD`, `DOBUF_DEL`
/// (also removed from the buffer list) or `DOBUF_WIPE` (really deleted);
/// `'bufhidden'` can force any of them.  With `abort_if_last`, refuse when
/// autocommands have left `win` the only window showing the buffer -- what
/// `:quit` needs.  With `ignore_abort`, keep going even while `aborting()`.
///
/// The answer is whether `b_nwindows` was decremented by this call itself,
/// rather than by an autocommand.
///
/// # Safety
/// `buf` must be a live buffer; `win` a live window or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn close_buffer(
    win: *mut win_T,
    buf: *mut buf_T,
    action: c_int,
    abort_if_last: bool,
    ignore_abort: bool,
) -> bool {
    // SAFETY: the caller's promise -- a live buffer.
    let buf = unsafe { Buf::new(buf) };
    close_buffer_inner(win, buf, action, abort_if_last, ignore_abort)
}

fn close_buffer_inner(
    win: *mut win_T,
    mut buf: Buf,
    action: c_int,
    abort_if_last: bool,
    ignore_abort: bool,
) -> bool {
    let mut how = Disposition::of(buf, action);
    let is_curwin = current_win().is_some_and(|wp| wp.w_buffer == buf.raw());
    let the_curwin = curwin.get();
    let the_curtab = curtab.get();
    // Upstream's CHECK_CURBUF sits here; it is a no-op outside
    // ABORT_ON_INTERNAL_ERROR builds.

    // Disallow deleting the buffer when it is locked (already being closed or
    // halfway a command that relies on it). Unloading is allowed.
    if (how.del || how.wipe) && !can_unload_buffer(buf) {
        return false;
    }

    // check no autocommands closed the window
    if let Some(mut wp) = valid_win(win) {
        // Set b_last_cursor when closing the last window for the buffer.
        // Remember the last cursor position and window options of the buffer.
        // This used to be only for the current window, but then options like
        // 'foldmethod' may be lost with a ":only" command.
        if buf.b_nwindows == 1 {
            remember_last_cursor(wp);
        }
        let cursor = wp.w_cursor;
        let lnum = if cursor.lnum == 1 { 0 } else { cursor.lnum };
        // SAFETY: a live buffer and a live window.
        unsafe { buflist_setfpos(buf.raw(), wp.raw(), lnum, cursor.col, true) };
    }

    let bufref = BufRef::of(buf);

    // When the buffer is no longer in a window, trigger BufWinLeave
    if buf.b_nwindows == 1 {
        let Some(kept) = leave_last_window(buf, bufref, win, &how, abort_if_last) else {
            return false;
        };
        buf = kept;
        // autocmds may abort script processing
        if !ignore_abort && aborting_now() {
            return false;
        }
    }

    // If the buffer was in curwin and the window has changed, go back to that
    // window, if it still exists.  This avoids that ":edit x" triggering a
    // "tabnext" BufUnload autocmd leaves a window behind without a buffer.
    restore_curwin(is_curwin, the_curwin, the_curtab);

    let nwindows = buf.b_nwindows;

    // decrease the link count from windows (unless not in any window)
    if buf.b_nwindows > 0 {
        buf.b_nwindows -= 1;
    }

    if diff_hidden_off() && !how.unload && buf.b_nwindows == 0 {
        diff_forget(buf); // Clear 'diff' for hidden buffer.
    }

    // Return when a window is displaying the buffer or when it's not unloaded.
    if buf.b_nwindows > 0 || !how.unload {
        return true;
    }

    // Always remove the buffer when there is no file name.
    if buf.b_ffname.is_null() {
        how.del = true;
    }

    // Free all things allocated for this buffer.  Also calls the "BufDelete"
    // autocommands when del_buf is true.  Remember if we are closing the
    // current buffer.  Restore the number of windows, so that autocommands in
    // buf_freeall() don't get confused.
    let is_curbuf = buf.raw() == curbuf.get();

    // When closing the current buffer stop Visual mode before freeing
    // anything.
    if is_curbuf && VIsual_active.get() {
        end_visual();
    }

    buf.b_nwindows = nwindows;

    // SAFETY: a live buffer.
    unsafe { buf_freeall(buf.raw(), how.free_flags(ignore_abort)) };

    // Autocommands may have deleted the buffer.
    let Some(mut buf) = bufref.get() else {
        return false;
    };
    // autocmds may abort script processing.
    if !ignore_abort && aborting_now() {
        return false;
    }

    // It's possible that autocommands change curbuf to the one being deleted.
    // This might cause the previous curbuf to be deleted unexpectedly.  But
    // in some cases it's OK to delete the curbuf, because a new one is
    // obtained anyway.  Therefore only return if curbuf changed to the
    // deleted buffer.
    if buf.raw() == curbuf.get() && !is_curbuf {
        return false;
    }

    // Defer clearing w_buffer until after operations that may invoke dict
    // watchers (e.g., buf_clear_file()), so callers like tabpagebuflist()
    // never see a window in the winlist with a NULL buffer.
    let clear_w_buf = valid_win(win).filter(|wp| wp.w_buffer == buf.raw());

    // Autocommands may have opened or closed windows for this buffer.
    // Decrement the count for the close we do here.  Don't decrement
    // b_nwindows if the buffer wasn't displayed in any window before calling
    // buf_freeall().
    if nwindows > 0 && buf.b_nwindows > 0 {
        buf.b_nwindows -= 1;
    }

    // Remove the buffer from the list.  Do not wipe out the buffer if it is
    // used in a window, or if autocommands wiped out all other buffers.
    let last_standing = buf.b_prev.is_null() && buf.b_next.is_null();
    if how.wipe && buf.b_nwindows <= 0 && !last_standing {
        unlink_and_free(buf, clear_w_buf);
    } else {
        if how.del {
            // Free all internal variables and reset option values, to make
            // ":bdel" compatible with Vim 5.7.
            free_buffer_stuff(
                buf,
                kBffClearWinInfo as c_int | kBffInitChangedtick as c_int,
            );

            // Make it look like a new buffer.
            buf.b_flags = BF_CHECK_RO | BF_NEVERLOADED;

            // Init the options when loaded again.
            buf.b_p_initialized = false;
        }
        // SAFETY: a live buffer.
        unsafe { buf_clear_file(buf.raw()) };
        if let Some(mut wp) = clear_w_buf {
            wp.w_buffer = ptr::null_mut();
        }
        if how.del {
            buf.b_p_bl = false_0;
        }
    }
    // NOTE: at this point "curbuf" may be invalid!
    true
}

/// The `b_nwindows == 1` arm: fire `BufWinLeave`, and `BufHidden` when the
/// buffer is only becoming hidden.
///
/// `None` means the caller must give up -- either an autocommand deleted the
/// buffer, or (with `abort_if_last`) it made `win` the only window.  A `Some`
/// carries the buffer back, re-validated.
fn leave_last_window(
    mut buf: Buf,
    bufref: BufRef,
    win: *mut win_T,
    how: &Disposition,
    abort_if_last: bool,
) -> Option<Buf> {
    // When the buffer becomes hidden, but is not unloaded, trigger BufHidden
    // after BufWinLeave.
    let events: &[_] = if how.unload {
        &[EVENT_BUFWINLEAVE]
    } else {
        &[EVENT_BUFWINLEAVE, EVENT_BUFHIDDEN]
    };
    for &event in events {
        buf.b_locked += 1;
        buf.b_locked_split += 1;
        if fire_named(event, buf) && !bufref.valid() {
            // Autocommands deleted the buffer.
            err_raw(tr_raw(&raw const e_auabort as *const c_char));
            return None;
        }
        buf = bufref.get()?;
        buf.b_locked -= 1;
        buf.b_locked_split -= 1;
        if abort_if_last && !win.is_null() && is_only_window(win) {
            // Autocommands made this the only window.
            err_raw(tr_raw(&raw const e_auabort as *const c_char));
            return None;
        }
    }
    Some(buf)
}

/// Go back to the window the caller started in, if an autocommand left us
/// somewhere else and it still exists.
fn restore_curwin(was_curwin: bool, the_curwin: *mut win_T, the_curtab: *mut tabpage_T) {
    if !was_curwin || curwin.get() == the_curwin {
        return;
    }
    let Some(wp) = valid_win(the_curwin) else {
        return;
    };
    // SAFETY: `the_curtab` was `curtab` when this call began and tab pages
    // outlive the windows in them; `wp` has just been re-validated.
    let tp = unsafe { TabPage::new(the_curtab) };
    block_autocmds_now();
    goto_win(tp, wp);
    unblock_autocmds_now();
}

/// The wipe arm: forget the buffer everywhere, unlink it from the buffer list
/// and free it.
fn unlink_and_free(mut buf: Buf, clear_w_buf: Option<Win>) {
    if let Some(mut wp) = clear_w_buf {
        wp.w_buffer = ptr::null_mut();
    }
    let fnum = buf.handle as c_int;
    for wp in tab_windows() {
        forget_file(wp, fnum);
    }
    if buf.b_sfname != buf.b_ffname {
        xfree_clear(&mut buf.b_sfname);
    } else {
        buf.b_sfname = ptr::null_mut();
    }
    xfree_clear(&mut buf.b_ffname);
    match buf.b_prev.is_null() {
        true => firstbuf.set(buf.b_next),
        // SAFETY: a non-null `b_prev` of a live buffer is a live buffer.
        false => unsafe { Buf::new(buf.b_prev) }.b_next = buf.b_next,
    }
    match buf.b_next.is_null() {
        true => lastbuf.set(buf.b_prev),
        // SAFETY: a non-null `b_next` of a live buffer is a live buffer.
        false => unsafe { Buf::new(buf.b_next) }.b_prev = buf.b_prev,
    }
    free_buffer(buf);
}

/// Make buffer not contain a file.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn buf_clear_file(buf: *mut buf_T) {
    // SAFETY: the caller's promise -- a live buffer.
    let mut buf = unsafe { Buf::new(buf) };
    buf.b_ml.ml_line_count = 1 as linenr_T;
    unchanged_now(buf, true, true);
    buf.b_p_eof = false_0;
    buf.b_start_eof = false_0;
    buf.b_p_eol = true_0;
    buf.b_start_eol = true_0;
    buf.b_p_bomb = false_0;
    buf.b_start_bomb = false_0;
    buf.b_ml.ml_mfp = ptr::null_mut::<memfile_T>();
    buf.b_ml.ml_flags = ML_EMPTY; // empty buffer
}

/// Clear the current buffer's contents.
///
/// # Safety
/// `curbuf` must be set, which it is from startup to exit.
pub unsafe fn buf_clear() {
    let buf = cur_buf();
    let line_count = buf.line_count();
    free_extmarks(buf); // delete any extmarks
    while cur_buf().b_ml.ml_flags & ML_EMPTY == 0 {
        delete_line(1 as linenr_T);
    }
    mark_lines_deleted(line_count); // prepare for display
}

// ---------------------------------------------------------------------------
// Freeing what belongs to the file

/// Free everything allocated for a buffer that belongs to the *file*: the
/// memline, the undo tree, the folds and the syntax state.
///
/// `flags` is the `BFA_*` set: `BFA_DEL`/`BFA_WIPE` say which of
/// `BufDelete`/`BufWipeout` to fire alongside `BufUnload`, `BFA_KEEP_UNDO`
/// keeps the undo tree, and `BFA_IGNORE_ABORT` keeps going while
/// `aborting()`.
///
/// Careful: gets here with `curwin` NULL when exiting.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn buf_freeall(buf: *mut buf_T, flags: c_int) {
    // SAFETY: the caller's promise -- a live buffer.
    let buf = unsafe { Buf::new(buf) };
    let is_curbuf = buf.raw() == curbuf.get();
    let is_curwin = current_win().is_some_and(|wp| wp.w_buffer == buf.raw());
    let the_curwin = curwin.get();
    let the_curtab = curtab.get();

    let Some(mut buf) = announce_unload(buf, flags) else {
        return;
    };
    buf.b_locked -= 1;
    buf.b_locked_split -= 1;

    // If the buffer was in curwin and the window has changed, go back to that
    // window, if it still exists.  This avoids that ":edit x" triggering a
    // "tabnext" BufUnload autocmd leaves a window behind without a buffer.
    restore_curwin(is_curwin, the_curwin, the_curtab);

    // autocmds may abort script processing
    if flags & BFA_IGNORE_ABORT as c_int == 0 && aborting_now() {
        return;
    }

    // It's possible that autocommands change curbuf to the one being deleted.
    // This might cause curbuf to be deleted unexpectedly.  But in some cases
    // it's OK to delete the curbuf, because a new one is obtained anyway.
    // Therefore only return if curbuf changed to the deleted buffer.
    if buf.raw() == curbuf.get() && !is_curbuf {
        return;
    }
    diff_forget(buf); // Can't use 'diff' for unloaded buffer.

    // Remove any ownsyntax, unless exiting.
    if let Some(wp) = current_win().filter(|wp| wp.w_buffer == buf.raw()) {
        reset_syntax(wp);
    }

    // No folds in an empty buffer.
    for win in tab_windows() {
        if win.w_buffer == buf.raw() {
            clear_folding(win);
        }
    }

    // Autocommands may have opened another terminal. Block them this time.
    if !buf.terminal.is_null() {
        block_autocmds_now();
        // SAFETY: a live buffer with a live terminal.
        unsafe { buf_close_terminal(buf.raw()) };
        unblock_autocmds_now();
    }

    let count = buf.line_count();
    close_memline(buf); // close and delete the memline/memfile
    buf.b_ml.ml_line_count = 0 as linenr_T; // no lines in buffer

    // Ensure marks are adjusted for cleared buffer in case buffer not on
    // disk: if it is reloaded the buffer will be empty.
    // SAFETY: a live buffer.
    if unsafe { bt_nofilename(buf.raw()) } && !exiting.get() {
        forget_lines(buf, count);
    }

    if flags & BFA_KEEP_UNDO as c_int == 0 {
        // free the memory allocated for undo and reset all undo information
        free_undo(buf);
    }
    clear_syntax(&mut buf.b_s); // reset syntax info
    buf.b_flags &= !BF_READERR; // a read error is no longer relevant
}

/// The autocommand half of [`buf_freeall`]: `BufUnload`, then `BufDelete` and
/// `BufWipeout` if the flags ask for them.
///
/// The buffer is pinned across all three (`b_locked`), but an autocommand can
/// still delete it -- `None` says so, and the caller returns without
/// unpinning, as upstream does.
fn announce_unload(mut buf: Buf, flags: c_int) -> Option<Buf> {
    // Make sure the buffer isn't closed by autocommands.
    buf.b_locked += 1;
    buf.b_locked_split += 1;

    let bufref = BufRef::of(buf);

    if !buf.terminal.is_null() {
        // SAFETY: a live buffer with a live terminal.
        unsafe { buf_close_terminal(buf.raw()) };
    }
    detach_updates(buf);

    let loaded = !buf.b_ml.ml_mfp.is_null();
    if loaded && fire_named(EVENT_BUFUNLOAD, buf) && !bufref.valid() {
        // Autocommands deleted the buffer.
        return None;
    }
    let mut buf = bufref.get()?;
    if flags & BFA_DEL as c_int != 0
        && buf.b_p_bl != 0
        && fire_named(EVENT_BUFDELETE, buf)
        && !bufref.valid()
    {
        // Autocommands may delete the buffer.
        return None;
    }
    buf = bufref.get()?;
    if flags & BFA_WIPE as c_int != 0 && fire_named(EVENT_BUFWIPEOUT, buf) && !bufref.valid() {
        // Autocommands may delete the buffer.
        return None;
    }
    bufref.get()
}

// ---------------------------------------------------------------------------
// Freeing the buffer itself

/// Free the buffer structure and everything belonging to the *buffer* rather
/// than to the file, which must have been freed already.
fn free_buffer(mut buf: Buf) {
    forget_handle(buf.handle as c_int);
    note_buffer_freed();
    // b:changedtick uses an item in buf_T.
    free_buffer_stuff(buf, kBffClearWinInfo as c_int);
    // SAFETY: a live buffer's variable dictionary is live.
    if unsafe { (*buf.b_vars).dv_refcount } > DO_NOT_FREE_CNT as c_int {
        rescue_changedtick(buf);
    }
    release_vars(buf);
    forget_autocmds(buf);
    free(buf.additional_data);
    free(buf.b_prompt_text);
    destroy_wininfo(buf);
    free_callback(&mut buf.b_prompt_callback);
    free_callback(&mut buf.b_prompt_interrupt);
    clear_mark(&mut buf.b_last_cursor);
    clear_mark(&mut buf.b_last_insert);
    clear_mark(&mut buf.b_last_change);
    clear_mark(&mut buf.b_prompt_start);
    for i in 0..NMARKS as usize {
        drop_mark(buf.b_namedm[i]);
    }
    for i in 0..buf.b_changelistlen as usize {
        drop_mark(buf.b_changelist[i]);
    }
    if autocmd_busy.get() {
        // Do not free the buffer structure while autocommands are executing,
        // it's still needed. Free it when autocmd_busy is reset.
        buf.b_namedm = [ZERO_FMARK; NMARKS as usize];
        buf.b_changelist = [ZERO_FMARK; 100];
        buf.b_next = au_pending_free_buf.get();
        au_pending_free_buf.set(buf.raw());
    } else {
        free(buf.raw());
        if curbuf.get() == buf.raw() {
            curbuf.set(ptr::null_mut()); // make clear it's not to be used
        }
    }
}

/// `kv_destroy(buf->b_wininfo)`.
fn destroy_wininfo(mut buf: Buf) {
    let kv = &mut buf.b_wininfo;
    free(kv.items);
    kv.capacity = 0;
    kv.size = 0;
    kv.items = ptr::null_mut::<*mut WinInfo>();
}

/// Free the `b_wininfo` list for buffer `buf`.
pub(crate) fn clear_wininfo(mut buf: Buf) {
    let kv = &mut buf.b_wininfo;
    for i in 0..kv.size {
        // SAFETY: the first `size` slots of a kvec hold live entries.
        free_entry(unsafe { *kv.items.add(i) });
    }
    kv.size = 0;
}

/// Free what `:bdel` and a wipe-out drop: the window memory, the local
/// options, the variables, the user commands, the extmarks and the mappings.
pub(crate) fn free_buffer_stuff(mut buf: Buf, free_flags: c_int) {
    if free_flags & kBffClearWinInfo as c_int != 0 {
        clear_wininfo(buf); // including window-local options
        // SAFETY: a live buffer.
        unsafe { free_buf_options(buf.raw(), true) };
        free_garray(&mut buf.b_s.b_langp);
    }
    clear_buf_vars(buf); // free all internal variables
    if free_flags & kBffInitChangedtick as c_int != 0 {
        buf_init_changedtick(buf);
    }
    free_user_commands(&mut buf.b_ucmds); // clear local user commands
    free_extmarks(buf); // delete any extmarks
    clear_mappings(buf, false); // clear local mappings
    clear_mappings(buf, true); // clear local abbrevs
    xfree_clear(&mut buf.b_start_fenc);

    free_update_callbacks(buf);
}

/// Wipe out `buf` outright, with autocommands blocked unless `aucmd` says the
/// caller is already inside one.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn wipe_buffer(buf: *mut buf_T, aucmd: bool) {
    if !aucmd {
        block_autocmds_now();
    }
    // SAFETY: the caller's promise -- a live buffer.
    unsafe {
        close_buffer(
            ptr::null_mut::<win_T>(),
            buf,
            DOBUF_WIPE as c_int,
            false,
            true,
        )
    };
    if !aucmd {
        unblock_autocmds_now();
    }
}
