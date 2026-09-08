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
#![allow(unsafe_code)]

use crate::memline::MlFlags;
use crate::message_fmt::c_str;
use crate::types::AutoEvent;
use crate::winlayer::WinId;
use core::ffi::c_int;
use core::ptr;

use super::*;
use crate::allocator::Owned;
use crate::autocmd::aubuflocal_remove;
use crate::autocmd::state::autocmd_busy;
use crate::buffer_updates::{buf_free_callbacks, buf_updates_unload};
use crate::change::deleted_lines_mark;
use crate::diff::{diff_buf_delete, diffopt_hiddenoff};
use crate::drawscreen::state::updating_screen;
use crate::eval::typval::{callback_free, tv_dict_add, tv_dict_item_copy};
use crate::eval::vars::{unref_var_dict, vars_clear};
use crate::extmark::extmark_free_all;
use crate::garray::ga_clear;
use crate::hashtab::{hash_find, hash_remove};
use crate::mapping::map_clear_mode;
use crate::mark::{clear_fmark, free_fmark, mark_adjust_buf, mark_forget_file, set_last_cursor};
use crate::memline::ml_close;
use crate::message::e_auabort;
use crate::normal::visual_active;
use crate::pos::MAXLNUM;
use crate::semsg;
use crate::startup::exiting;
use crate::state::MAP_ALL_MODES;
use crate::syntax::syntax_clear;
use crate::terminal::terminal_close;
use crate::types::{
    Callback, ColNr, DictItem, FileMark, FileMarkView, GArray, Handle, HashTab, LineNr, MemFile,
    Pos, Refcount, SynBlock, Timestamp, WinInfo,
};
use crate::undo::u_clearallandblockfree;
use crate::usercmd::{Table, uc_clear};
use crate::window::{free_wininfo, goto_tabpage_win, one_window};
use crate::winlayer::graph::{firstbuf, lastbuf, leave_curbuf};
use crate::winlayer::{Buf, TabPage, Win, defer_free_buffer, forget_buffer, tab_windows, windows};

/// A mark that has never been set, as `CLEAR_FIELD()` leaves one: all zero,
/// which is *not* `INIT_FMARK` (that seeds `topline_offset` with `MAXLNUM`).
const ZERO_FMARK: FileMark = FileMark {
    mark: Pos {
        lnum: 0 as LineNr,
        col: 0 as ColNr,
        coladd: 0 as ColNr,
    },
    fnum: 0,
    timestamp: 0 as Timestamp,
    view: FileMarkView {
        topline_offset: 0 as LineNr,
        skipcol: 0 as ColNr,
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
fn valid_win(win: WinId) -> Option<Win> {
    tab_windows().find(|wp| wp.id() == win)
}

/// Whether `win` is the only non-floating window of its tab page.
fn is_only_window(win: Win) -> bool {
    one_window(win, None)
}

/// Make `win` in `tabpage` current again, without firing autocommands.
fn goto_win(tabpage: TabPage, win: Win) {
    goto_tabpage_win(tabpage, win);
}

/// Remember `win`'s cursor as the buffer's last position.
fn remember_last_cursor(win: Win) {
    set_last_cursor(win);
}

/// Forget every mark and jump-list entry naming buffer `fnum` in `win`.
fn forget_file(win: Win, fnum: c_int) {
    // SAFETY: a live window.
    unsafe { mark_forget_file(win, fnum) };
}

fn detach_updates(buffer: Buf) {
    buf_updates_unload(buffer, false);
}

fn free_update_callbacks(buffer: Buf) {
    buf_free_callbacks(buffer);
}

fn diff_forget(buffer: Buf) {
    diff_buf_delete(buffer);
}

/// Whether `'diffopt'` contains `hiddenoff`.
fn diff_hidden_off() -> bool {
    diffopt_hiddenoff()
}

fn free_extmarks(buffer: Buf) {
    extmark_free_all(buffer);
}

fn free_user_commands(buffer: Buf) {
    // SAFETY: a live buffer. `uc_clear` leaves the table empty and usable,
    // which is what the buffers that outlive this -- `:bdel`'s, and the
    // `curbuf` `buflist_new` reuses -- need.
    // SAFETY: module contract.
    unsafe { uc_clear(Table::Buffer(buffer)) };
}

fn free_garray(ga: &mut GArray) {
    // SAFETY: a growable array inside a live buffer.
    unsafe { ga_clear(ga) };
}

/// Drop every buffer-local mapping (`abbrev` picks the abbreviation table).
fn clear_mappings(buffer: Buf, abbrev: bool) {
    // SAFETY: a live buffer.
    unsafe { map_clear_mode(buffer, MAP_ALL_MODES, true, abbrev) };
}

fn free_callback(cb: &mut Callback) {
    // SAFETY: a callback inside a live buffer.
    unsafe { callback_free(cb) };
}

fn clear_mark(mark: &mut FileMark) {
    // SAFETY: a mark inside a live buffer; `0` is upstream's timestamp.
    unsafe { clear_fmark(mark, 0 as Timestamp) };
}

fn drop_mark(mark: FileMark) {
    // SAFETY: a mark copied out of a live buffer.
    unsafe { free_fmark(mark) };
}

/// Move every mark in `buffer` up by `count` lines from line 1 -- what an
/// emptied buffer needs so a reload starts from a clean slate.
fn forget_lines(buffer: Buf, count: LineNr) {
    let last = MAXLNUM;
    // SAFETY: a live buffer.
    unsafe {
        mark_adjust_buf(
            buffer,
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

fn free_undo(buffer: Buf) {
    u_clearallandblockfree(buffer);
}

fn clear_syntax(syn: &mut SynBlock) {
    // SAFETY: the syntax block of a live buffer.
    unsafe { syntax_clear(syn) };
}

/// Close the memline and delete the swap file.
fn close_memline(buffer: Buf) {
    // SAFETY: a live buffer; `true` is upstream's `del_file`.
    unsafe { ml_close(buffer, 1) };
}

fn mark_lines_deleted(count: LineNr) {
    // SAFETY: reads the current buffer, which the caller has just emptied.
    unsafe { deleted_lines_mark(1 as LineNr, count as c_int) };
}

fn free_entry(entry: *mut WinInfo) {
    // SAFETY: an entry of a live buffer's `b_wininfo`.
    unsafe { free_wininfo(entry) };
}

/// `buffer.b_vars->dv_hashtab`.
fn buf_vars(mut buffer: Buf) -> *mut HashTab {
    // SAFETY: a live buffer's variable dictionary is live.
    unsafe { &raw mut (*buffer.b_vars).dv_hashtab }
}

/// Free every buffer-local variable.
///
/// `b:changedtick` lives in a field of `Buffer` rather than in the dictionary's
/// own storage, so it is removed from the hash table first: clearing it would
/// go through `clear_tv()` and zero the counter.
fn clear_buf_vars(buffer: Buf) {
    let vars = buf_vars(buffer);
    // SAFETY: the hash table of a live buffer's variable dictionary; the
    // `changedtick` entry is put there when the buffer is created.
    unsafe {
        let changedtick_hi = hash_find(vars, c"changedtick".as_ptr());
        debug_assert!(changedtick_hi.is_kept(), "changedtick is in the table");
        hash_remove(vars, changedtick_hi);
        vars_clear(vars);
    }
}

/// Hand `b:changedtick` to the dictionary before the buffer goes away, for the
/// script that is still holding a reference to it.
fn rescue_changedtick(mut buffer: Buf) {
    let (vars, di) = (
        buffer.b_vars,
        &raw mut buffer.changedtick_di as *mut DictItem,
    );
    // SAFETY: a live buffer's dictionary, and its own `changedtick` item.
    let _ = unsafe { tv_dict_add(vars, tv_dict_item_copy(di)) };
}

fn release_vars(buffer: Buf) {
    // SAFETY: a live buffer's variable dictionary.
    unsafe { unref_var_dict(buffer.b_vars) };
}

fn forget_autocmds(buffer: Buf) {
    aubuflocal_remove(buffer);
}

/// Take the buffer's number out of the registry, so that nothing can look it
/// up again, and with it the allocation the registry owned.
///
/// The first thing [`free_buffer`] does, which is what lets the registry
/// promise that everything in it is live. Dropping what this answers is the
/// free; [`free_buffer`] holds it until the point the `xfree` used to be.
#[must_use = "dropping the answer is the free"]
fn forget_handle(fnum: Handle) -> Owned<Buffer> {
    // Every buffer reaching a free path was registered when it was given its
    // number, and a number is given exactly once.
    forget_buffer(fnum).expect("a buffer being freed is a registered buffer")
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
    fn of(buffer: Buf, action: c_int) -> Self {
        let mut it = Disposition {
            unload: action != 0,
            del: action == DOBUF_DEL as c_int || action == DOBUF_WIPE as c_int,
            wipe: action == DOBUF_WIPE as c_int,
        };
        // The caller must take care of NOT deleting/freeing when 'bufhidden'
        // is "hide" (otherwise we could never free or delete a buffer).
        if buffer.terminal.is_null() {
            // SAFETY: `'bufhidden'` is a NUL-terminated option value.
            match unsafe { *buffer.b_p_bh } as u8 {
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

/// Whether `buffer` may be unloaded, with the error message when it may not.
///
/// A buffer is locked while it is halfway through a command that relies on
/// it, and cannot be unloaded from under a redraw that is showing it.
pub(crate) fn can_unload_buffer(buffer: Buf) -> bool {
    let mut can_unload = buffer.b_locked == 0;

    if can_unload && updating_screen.get() {
        can_unload = !windows().any(|wp| wp.w_buffer == buffer.raw());
    }
    // Don't unload the buffer while it's still being saved
    if can_unload && buffer.b_saving {
        can_unload = false;
    }

    if !can_unload {
        let fname = if buffer.b_fname.is_null() {
            buffer.b_ffname
        } else {
            buffer.b_fname
        };
        // SAFETY: a buffer's own name, NUL-terminated.
        let name = unsafe { c_str(fname) };
        let name = if fname.is_null() {
            "[No Name]".into()
        } else {
            name.to_string()
        };
        semsg!("E937: Attempt to delete a buffer that is in use: {name}");
    }
    can_unload
}

pub fn buf_close_terminal(mut buffer: Buf) {
    debug_assert!(!buffer.terminal.is_null(), "buf->terminal");
    buffer.b_locked += 1;
    // SAFETY: a live terminal, the assertion above having ruled out null.
    unsafe { terminal_close(&raw mut buffer.terminal, -1) };
    buffer.b_locked -= 1;
}

// ---------------------------------------------------------------------------
// Closing the link to a buffer

/// Close the link between `win` and `buffer`, and act on `action` once no window
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
pub fn close_buffer(
    win: Option<Win>,
    buffer: Buf,
    action: c_int,
    abort_if_last: bool,
    ignore_abort: bool,
) -> bool {
    close_buffer_inner(win, buffer, action, abort_if_last, ignore_abort)
}

fn close_buffer_inner(
    win: Option<Win>,
    mut buffer: Buf,
    action: c_int,
    abort_if_last: bool,
    ignore_abort: bool,
) -> bool {
    let mut how = Disposition::of(buffer, action);
    let is_curwin = current_win().is_some_and(|wp| wp.w_buffer == buffer.raw());
    let the_curwin = Win::current().id();
    let the_curtab = TabPage::current();
    // Upstream's CHECK_CURBUF sits here; it is a no-op outside
    // ABORT_ON_INTERNAL_ERROR builds.

    // Disallow deleting the buffer when it is locked (already being closed or
    // halfway a command that relies on it). Unloading is allowed.
    if (how.del || how.wipe) && !can_unload_buffer(buffer) {
        return false;
    }

    // check no autocommands closed the window
    if let Some(wp) = win.map(Win::id).and_then(valid_win) {
        // Set b_last_cursor when closing the last window for the buffer.
        // Remember the last cursor position and window options of the buffer.
        // This used to be only for the current window, but then options like
        // 'foldmethod' may be lost with a ":only" command.
        if buffer.b_nwindows == 1 {
            remember_last_cursor(wp);
        }
        let cursor = wp.w_cursor;
        let lnum = if cursor.lnum == 1 { 0 } else { cursor.lnum };
        buflist_setfpos(buffer, Some(wp), lnum, cursor.col, true);
    }

    let bufref = BufRef::of(buffer);

    // When the buffer is no longer in a window, trigger BufWinLeave
    if buffer.b_nwindows == 1 {
        let Some(kept) = leave_last_window(buffer, bufref, win, &how, abort_if_last) else {
            return false;
        };
        buffer = kept;
        // autocmds may abort script processing
        if !ignore_abort && aborting_now() {
            return false;
        }
    }

    // If the buffer was in curwin and the window has changed, go back to that
    // window, if it still exists.  This avoids that ":edit x" triggering a
    // "tabnext" BufUnload autocmd leaves a window behind without a buffer.
    restore_curwin(is_curwin, the_curwin, the_curtab);

    let nwindows = buffer.b_nwindows;

    // decrease the link count from windows (unless not in any window)
    if buffer.b_nwindows > 0 {
        buffer.b_nwindows -= 1;
    }

    if diff_hidden_off() && !how.unload && buffer.b_nwindows == 0 {
        diff_forget(buffer); // Clear 'diff' for hidden buffer.
    }

    // Return when a window is displaying the buffer or when it's not unloaded.
    if buffer.b_nwindows > 0 || !how.unload {
        return true;
    }

    // Always remove the buffer when there is no file name.
    if buffer.b_ffname.is_null() {
        how.del = true;
    }

    // Free all things allocated for this buffer.  Also calls the "BufDelete"
    // autocommands when del_buf is true.  Remember if we are closing the
    // current buffer.  Restore the number of windows, so that autocommands in
    // buf_freeall() don't get confused.
    let is_curbuf = buffer.raw() == Buf::current_raw();

    // When closing the current buffer stop Visual mode before freeing
    // anything.
    if is_curbuf && visual_active() {
        end_visual();
    }

    buffer.b_nwindows = nwindows;

    buf_freeall(buffer, how.free_flags(ignore_abort));

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
    if buf.raw() == Buf::current_raw() && !is_curbuf {
        return false;
    }

    // Defer clearing w_buffer until after operations that may invoke dict
    // watchers (e.g., buf_clear_file()), so callers like tabpagebuflist()
    // never see a window in the winlist with a NULL buffer.
    let clear_w_buf = win
        .map(Win::id)
        .and_then(valid_win)
        .filter(|wp| wp.w_buffer == buf.raw());

    // Autocommands may have opened or closed windows for this buffer.
    // Decrement the count for the close we do here.  Don't decrement
    // b_nwindows if the buffer wasn't displayed in any window before calling
    // buf_freeall().
    if nwindows > 0 && buf.b_nwindows > 0 {
        buf.b_nwindows -= 1;
    }

    // Remove the buffer from the list.  Do not wipe out the buffer if it is
    // used in a window, or if autocommands wiped out all other buffers.
    let last_standing = buf.b_prev.is_none() && buf.b_next.is_none();
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
            buf.b_flags = BufFlags::CHECK_RO | BufFlags::NEVERLOADED;

            // Init the options when loaded again.
            buf.b_p_initialized = false;
        }
        buf_clear_file(buf);
        if let Some(mut wp) = clear_w_buf {
            wp.w_buffer = ptr::null_mut();
        }
        if how.del {
            buf.b_p_bl = 0;
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
    mut buffer: Buf,
    bufref: BufRef,
    win: Option<Win>,
    how: &Disposition,
    abort_if_last: bool,
) -> Option<Buf> {
    // When the buffer becomes hidden, but is not unloaded, trigger BufHidden
    // after BufWinLeave.
    let events: &[_] = if how.unload {
        &[AutoEvent::BufWinLeave]
    } else {
        &[AutoEvent::BufWinLeave, AutoEvent::BufHidden]
    };
    for &event in events {
        buffer.b_locked += 1;
        buffer.b_locked_split += 1;
        if fire_named(event, buffer) && !bufref.valid() {
            // Autocommands deleted the buffer.
            err_raw(tr_raw(e_auabort.as_ptr()));
            return None;
        }
        buffer = bufref.get()?;
        buffer.b_locked -= 1;
        buffer.b_locked_split -= 1;
        if abort_if_last && win.is_some_and(is_only_window) {
            // Autocommands made this the only window.
            err_raw(tr_raw(e_auabort.as_ptr()));
            return None;
        }
    }
    Some(buffer)
}

/// Go back to the window the caller started in, if an autocommand left us
/// somewhere else and it still exists.
///
/// Takes the *identity* rather than the window: the caller means "the window
/// I was in, if it is still there", and an id cannot be mistaken for a value
/// that may be dereferenced.
fn restore_curwin(was_curwin: bool, the_curwin: WinId, tabpage: TabPage) {
    if !was_curwin || Win::current_or_none().map(Win::id) == Some(the_curwin) {
        return;
    }
    let Some(wp) = valid_win(the_curwin) else {
        return;
    };
    block_autocmds_now();
    goto_win(tabpage, wp);
    unblock_autocmds_now();
}

/// The wipe arm: forget the buffer everywhere, unlink it from the buffer list
/// and free it.
fn unlink_and_free(mut buffer: Buf, clear_w_buf: Option<Win>) {
    if let Some(mut wp) = clear_w_buf {
        wp.w_buffer = ptr::null_mut();
    }
    let fnum = buffer.handle as c_int;
    for wp in tab_windows() {
        forget_file(wp, fnum);
    }
    if buffer.b_sfname != buffer.b_ffname {
        xfree_clear(&mut buffer.b_sfname);
    } else {
        buffer.b_sfname = ptr::null_mut();
    }
    xfree_clear(&mut buffer.b_ffname);
    match buffer.prev() {
        None => firstbuf.set(buffer.b_next),
        Some(mut prev) => prev.b_next = buffer.b_next,
    }
    match buffer.next() {
        None => lastbuf.set(buffer.b_prev),
        Some(mut next) => next.b_prev = buffer.b_prev,
    }
    free_buffer(buffer);
}

/// Make buffer not contain a file.
///
pub fn buf_clear_file(mut buffer: Buf) {
    buffer.b_ml.ml_line_count = 1 as LineNr;
    unchanged_now(buffer, true, true);
    buffer.b_p_eof = 0;
    buffer.b_start_eof = 0;
    buffer.b_p_eol = 1;
    buffer.b_start_eol = 1;
    buffer.b_p_bomb = 0;
    buffer.b_start_bomb = 0;
    buffer.b_ml.ml_mfp = ptr::null_mut::<MemFile>();
    // Upstream's `ml_flags = ML_EMPTY` also dropped the ownership of the
    // cached line, without freeing it; the memfile it pointed into is gone
    // either way.
    buffer.b_ml.forget_line();
    buffer.b_ml.ml_flags = MlFlags::EMPTY; // empty buffer
}

/// Clear the current buffer's contents.
pub fn buf_clear() {
    let buf = Buf::current();
    let line_count = buf.line_count();
    free_extmarks(buf); // delete any extmarks
    while !Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY) {
        delete_line(1 as LineNr);
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
pub fn buf_freeall(buffer: Buf, flags: c_int) {
    let is_curbuf = buffer.raw() == Buf::current_raw();
    let is_curwin = current_win().is_some_and(|wp| wp.w_buffer == buffer.raw());
    let the_curwin = Win::current().id();
    let the_curtab = TabPage::current();

    let Some(mut buf) = announce_unload(buffer, flags) else {
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
    if buf.raw() == Buf::current_raw() && !is_curbuf {
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
            clear_window_folds(win);
        }
    }

    // Autocommands may have opened another terminal. Block them this time.
    if !buf.terminal.is_null() {
        block_autocmds_now();
        buf_close_terminal(buf);
        unblock_autocmds_now();
    }

    let count = buf.line_count();
    close_memline(buf); // close and delete the memline/memfile
    buf.b_ml.ml_line_count = 0 as LineNr; // no lines in buffer

    // Ensure marks are adjusted for cleared buffer in case buffer not on
    // disk: if it is reloaded the buffer will be empty.
    if buf_is_nofilename(Some(buf)) && !exiting.get() {
        forget_lines(buf, count);
    }

    if flags & BFA_KEEP_UNDO as c_int == 0 {
        // free the memory allocated for undo and reset all undo information
        free_undo(buf);
    }
    clear_syntax(&mut buf.b_s); // reset syntax info
    buf.b_flags.clear(BufFlags::READERR); // a read error is no longer relevant
}

/// The autocommand half of [`buf_freeall`]: `BufUnload`, then `BufDelete` and
/// `BufWipeout` if the flags ask for them.
///
/// The buffer is pinned across all three (`b_locked`), but an autocommand can
/// still delete it -- `None` says so, and the caller returns without
/// unpinning, as upstream does.
fn announce_unload(mut buffer: Buf, flags: c_int) -> Option<Buf> {
    // Make sure the buffer isn't closed by autocommands.
    buffer.b_locked += 1;
    buffer.b_locked_split += 1;

    let bufref = BufRef::of(buffer);

    if !buffer.terminal.is_null() {
        buf_close_terminal(buffer);
    }
    detach_updates(buffer);

    let loaded = !buffer.b_ml.ml_mfp.is_null();
    if loaded && fire_named(AutoEvent::BufUnload, buffer) && !bufref.valid() {
        // Autocommands deleted the buffer.
        return None;
    }
    let mut buf = bufref.get()?;
    if flags & BFA_DEL as c_int != 0
        && buf.b_p_bl != 0
        && fire_named(AutoEvent::BufDelete, buf)
        && !bufref.valid()
    {
        // Autocommands may delete the buffer.
        return None;
    }
    buf = bufref.get()?;
    if flags & BFA_WIPE as c_int != 0 && fire_named(AutoEvent::BufWipeout, buf) && !bufref.valid() {
        // Autocommands may delete the buffer.
        return None;
    }
    bufref.get()
}

// ---------------------------------------------------------------------------
// Freeing the buffer itself

/// Free the buffer structure and everything belonging to the *buffer* rather
/// than to the file, which must have been freed already.
fn free_buffer(mut buffer: Buf) {
    // The allocation, out of the registry from here on. `buffer` is still the
    // address to work through; `owned` is only who gives the memory back.
    let owned = forget_handle(buffer.handle());
    note_buffer_freed();
    // b:changedtick uses an item in Buffer.
    free_buffer_stuff(buffer, kBffClearWinInfo as c_int);
    // SAFETY: a live buffer's variable dictionary is live.
    if unsafe { (*buffer.b_vars).dv_refcount } > Refcount::new(DO_NOT_FREE_CNT as c_int) {
        rescue_changedtick(buffer);
    }
    release_vars(buffer);
    forget_autocmds(buffer);
    free(buffer.additional_data);
    free(buffer.b_prompt_text);
    destroy_wininfo(buffer);
    free_callback(&mut buffer.b_prompt_callback);
    free_callback(&mut buffer.b_prompt_interrupt);
    clear_mark(&mut buffer.b_last_cursor);
    clear_mark(&mut buffer.b_last_insert);
    clear_mark(&mut buffer.b_last_change);
    clear_mark(&mut buffer.b_prompt_start);
    for i in 0..NMARKS as usize {
        drop_mark(buffer.b_namedm[i].clone());
    }
    for i in 0..buffer.b_changelistlen as usize {
        drop_mark(buffer.b_changelist[i].clone());
    }
    if autocmd_busy.get() {
        // Do not free the buffer structure while autocommands are executing,
        // it's still needed. Free it when autocmd_busy is reset.
        buffer.b_namedm = [ZERO_FMARK; NMARKS as usize];
        buffer.b_changelist = [ZERO_FMARK; 100];
        defer_free_buffer(owned);
    } else {
        // The free: `Buffer`'s destructor runs and the memory goes back.
        drop(owned);
        if Buf::current_raw() == buffer.raw() {
            leave_curbuf(); // make clear it's not to be used
        }
    }
}

/// `kv_destroy(buf->b_wininfo)`.
fn destroy_wininfo(mut buffer: Buf) {
    let kv = &mut buffer.b_wininfo;
    free(kv.items);
    kv.capacity = 0;
    kv.size = 0;
    kv.items = ptr::null_mut::<*mut WinInfo>();
}

/// Free the `b_wininfo` list for buffer `buffer`.
pub(crate) fn clear_wininfo(mut buffer: Buf) {
    let kv = &mut buffer.b_wininfo;
    for i in 0..kv.size {
        // SAFETY: the first `size` slots of a kvec hold live entries.
        free_entry(unsafe { *kv.items.add(i) });
    }
    kv.size = 0;
}

/// Free what `:bdel` and a wipe-out drop: the window memory, the local
/// options, the variables, the user commands, the extmarks and the mappings.
pub(crate) fn free_buffer_stuff(mut buffer: Buf, free_flags: c_int) {
    if free_flags & kBffClearWinInfo as c_int != 0 {
        clear_wininfo(buffer); // including window-local options
        free_buf_options(buffer, true);
        free_garray(&mut buffer.b_s.b_langp);
    }
    clear_buf_vars(buffer); // free all internal variables
    if free_flags & kBffInitChangedtick as c_int != 0 {
        buf_init_changedtick(buffer);
    }
    free_user_commands(buffer); // clear local user commands
    free_extmarks(buffer); // delete any extmarks
    clear_mappings(buffer, false); // clear local mappings
    clear_mappings(buffer, true); // clear local abbrevs
    xfree_clear(&mut buffer.b_start_fenc);

    free_update_callbacks(buffer);
}

/// Wipe out `buffer` outright, with autocommands blocked unless `aucmd` says the
/// caller is already inside one.
///
pub fn wipe_buffer(buffer: Buf, aucmd: bool) {
    if !aucmd {
        block_autocmds_now();
    }
    close_buffer(None, buffer, DOBUF_WIPE as c_int, false, true);
    if !aucmd {
        unblock_autocmds_now();
    }
}
