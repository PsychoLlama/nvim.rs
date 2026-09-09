//! Making a buffer current -- `set_curbuf()` and `enter_buffer()`.
//!
//! [`set_curbuf`] leaves the old buffer (`BufLeave`, remembering the cursor
//! position for the window) and [`enter_buffer`] arrives in the new one:
//! apply the window's remembered position, load the buffer if it is not
//! loaded, re-apply the local options and folds, and fire
//! `BufEnter`/`BufWinEnter`.  The `no_write_message*` trio is the "no write
//! since last change" error every caller of these has to be able to
//! raise.
//!
//! `BufLeave` is the family's sharpest re-entrancy: it can free the buffer
//! being left, the buffer being entered, or both, and it can change the
//! current window.  [`set_curbuf`] therefore takes a [`BufRef`] for each of
//! the two before it fires and asks again after every step -- which is what
//! the `prevbufref`/`newbufref` pair is for upstream.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::semsg;
use crate::types::AutoEvent;
use core::ffi::{c_char, c_int};
use core::ptr;

use super::*;
use crate::arglist::check_arg_idx;

use crate::channel::channel_job_running;
use crate::diff::diff_buf_add;
use crate::digraph::keymap_init;
use crate::drawscreen::UPD_NOT_VALID;
use crate::eval::typval::tv_dict_add;
use crate::ex_docmd::cmdmod_has;
use crate::file_search::vim_chdirfile;
use crate::fileio::{buf_check_timestamp, shorten_fnames};
use crate::indent::inindent;
use crate::message::state::{msg_silent, need_fileinfo};
use crate::message::{
    e_job_still_running, e_job_still_running_add_bang_to_end_the_job, e_no_write_since_last_change,
    e_no_write_since_last_change_add_bang_to_override,
};
use crate::r#move::{WinValid, scroll_cursor_halfway};
use crate::normal::visual_active;
use crate::option::buf_copy_options;
use crate::option::vars::p_acd;
use crate::os::state::last_chdir_reason;
use crate::spell::parse_spelllang;
use crate::startup::starting;
use crate::state::MODE_INSERT;
use crate::state::mode::{State, VIsual_reselect};
use crate::terminal::terminal_check_size;
use crate::types::{
    ChangedtickDictItem, CmdModFlags, ColNr, DictItem, Failed, LineNr, NUL, OptInt, ShmFlag,
    Terminal, TypVal, VarLock, time_t, uint8_t, uint64_t,
};
use crate::undo::u_sync;
use crate::window::get_last_winid;
use crate::winlayer::window_at;
use ::libc::time;

// ---------------------------------------------------------------------------
// The neighbours, wrapped

/// The id the last window created was given -- the C's cheap "did an
/// autocommand open a window?" probe.
fn last_winid() -> c_int {
    get_last_winid()
}

/// Whether `buffer` may stay loaded when it is no longer shown -- `'hidden'`,
/// `'bufhidden'` or a `:hide` modifier.
fn may_hide(buffer: Buf) -> bool {
    buf_hide(buffer)
}

/// Sync the undo state, so that what follows starts a new change.
fn sync_undo() {
    // SAFETY: reads the current buffer's undo tree.
    u_sync(false);
}

/// Remember `win`'s cursor position for the alternate file.
fn remember_altfpos(win: Win) {
    buflist_altfpos(win);
}

/// Restore the window-local options `win` last used with this buffer.
fn restore_winopts(buffer: Buf) {
    get_winopts(buffer);
}

/// Copy the buffer-local option values into `buffer`.
fn copy_options_into(buffer: Buf, flags: c_int) {
    // SAFETY: a live buffer.
    unsafe { buf_copy_options(buffer, flags) };
}

fn diff_add(buffer: Buf) {
    diff_buf_add(buffer);
}

/// Load the buffer that has just been made current.
fn load_current_buffer() {
    // SAFETY: `curbuf` and `curwin` are set; a null `eap` is the no-command form.
    let _ = unsafe { open_buffer(false, ptr::null_mut(), 0) };
}

/// Warn if the file changed on disk since the buffer was read.
fn check_timestamp(buffer: Buf) {
    // SAFETY: a live buffer, which survives the autocommands it fires.
    unsafe { buf_check_timestamp(buffer) };
}

/// Whether the cursor is in the indent of its line.
fn cursor_in_indent() -> bool {
    // SAFETY: reads the current window's cursor and line.
    unsafe { inindent(0) }
}

/// Put the cursor back where this window last was in this buffer.
fn restore_position() {
    buflist_getfpos();
}

/// Re-check the argument-list index after the buffer changed.
fn recheck_arg_idx(win: Win) {
    // SAFETY: a live window.
    check_arg_idx(win);
}

/// Rebuild `'title'` and `'icon'`.
fn rebuild_title() {
    maketitle();
}

/// Scroll so that the cursor line sits in the middle of the window.
fn scroll_halfway(win: Win) {
    // SAFETY: a live window.
    scroll_cursor_halfway(win, false, false);
}

/// Load the keymap `'keymap'` names.
fn init_keymap() {
    keymap_init();
}

/// Work out the spell-checking languages for `win`.
fn set_spelllang(win: Win) {
    parse_spelllang(win);
}

/// Whether the window's `'spelllang'` is set. It lives in the syntax block
/// the window shares with its buffer.
fn has_spelllang(win: Win) -> bool {
    // SAFETY: a live window's syntax block is live, and `'spelllang'` a
    // NUL-terminated option value.
    unsafe { *(*win.w_s).b_p_spl as c_int != NUL }
}

fn resize_terminal(term: *mut Terminal) {
    // SAFETY: a live terminal, the caller having ruled out null.
    unsafe { terminal_check_size(term) };
}

/// Whether the job behind terminal buffer `buffer` is still running.
fn job_running(buffer: Buf) -> bool {
    // SAFETY: reads the buffer's `'channel'` and looks it up.
    unsafe { channel_job_running(buffer.b_p_channel as uint64_t) }
}

/// Change to the directory of `fname`.
fn chdir_to_file(fname: *mut c_char) -> Result<(), Failed> {
    // SAFETY: a NUL-terminated file name.
    unsafe { vim_chdirfile(fname, kCdCauseAuto) }
}

/// Recompute every buffer's short file name against the new directory.
fn reshorten_fnames() {
    shorten_fnames(1);
}

/// The wall clock, for `b_last_used`.
fn now() -> time_t {
    // SAFETY: a null argument asks for the answer by value.
    unsafe { time(ptr::null_mut::<time_t>()) }
}

/// Add `b:changedtick` to the buffer's variable dictionary.
fn add_changedtick(mut buffer: Buf) {
    let (vars, di) = (
        buffer.b_vars,
        &raw mut buffer.changedtick_di as *mut DictItem,
    );
    // SAFETY: a live buffer's dictionary, and its own `changedtick` item.
    let _ = unsafe { tv_dict_add(vars, di) };
}

// ---------------------------------------------------------------------------
// Leaving one buffer for another

/// Make `buffer` the current buffer, closing the one being left as `action` says
/// (`DOBUF_GOTO` frees or hides it, `DOBUF_SPLIT` leaves it alone, and
/// `DOBUF_UNLOAD`/`DEL`/`WIPE` do what they say).
///
/// With `update_jumplist` the position being left joins the jump list.
pub fn set_curbuf(buffer: Buf, action: c_int, update_jumplist: bool) {
    let unload = action == DOBUF_UNLOAD as c_int
        || action == DOBUF_DEL as c_int
        || action == DOBUF_WIPE as c_int;
    let old_tw: OptInt = Buf::current().b_p_tw;
    let winid_before = last_winid();

    if update_jumplist {
        set_pcmark();
    }

    let mut win = Win::current();
    if !cmdmod_has(CmdModFlags::KEEPALT) {
        win.w_alt_fnum = Buf::current().handle as c_int; // remember alternate file
    }
    remember_altfpos(win); // remember curpos

    // Don't restart Select mode after switching to another buffer.
    VIsual_reselect.set(0);

    // close_windows() or apply_autocmds() may change curbuf and wipe out "buf"
    let prevbuf = Buf::current();
    let prevbufref = BufRef::of(prevbuf);
    let newbufref = BufRef::of(buffer);
    let prev_nwindows = prevbuf.b_nwindows;

    // Autocommands may delete the current buffer and/or the buffer we want to
    // go to.  In those cases don't close the buffer.
    if !fire(AutoEvent::BufLeave, Buf::current())
        || prevbufref.valid() && newbufref.valid() && !aborting_now()
    {
        leave_prevbuf(prevbufref, action, unload, prev_nwindows, winid_before);
    }

    // An autocommand may have deleted "buf", already entered it (e.g., when it
    // did ":bunload") or aborted the script processing!  If curwin->w_buffer is
    // null, enter_buffer() will make it valid again.
    // The other half of the rule: ask the registry by the identity taken
    // above, not the buffer list by `buffer`'s address. Stricter, too — a buffer
    // wiped and a new one allocated at the same address would pass an address
    // comparison, the hazard `BufferRef` carries `br_buf_free_count` for.
    let valid = buffer.id().valid();
    if valid && buffer.raw() != Buf::current_raw() && !aborting_now()
        || Win::current().w_buffer.is_null()
    {
        // autocommands changed curbuf and we will move to another buffer soon,
        // so decrement curbuf->b_nwindows
        if let Some(mut cur) = current_buf().filter(|c| *c != prevbuf) {
            cur.b_nwindows -= 1;
        }
        // If the buffer is not valid but curwin->w_buffer is NULL we must enter
        // some buffer.  Using the last one is hopefully OK.
        enter_buffer(if valid {
            buffer
        } else {
            last_buf().expect("lastbuf != NULL")
        });
        if old_tw != Buf::current().b_p_tw {
            recheck_colorcolumn(Win::current());
        }
    }

    if let Some(prev) = prevbufref.get().filter(|p| !p.terminal.is_null()) {
        resize_terminal(prev.terminal);
    }
}

/// Close the windows and the buffer being left, if `BufLeave` has not already
/// disposed of them.
fn leave_prevbuf(
    prevbufref: BufRef,
    action: c_int,
    unload: bool,
    prev_nwindows: c_int,
    winid_before: c_int,
) {
    // The caller's guard has just said `prevbuf` is still the buffer it was
    // -- either `BufLeave` ran nothing, or `bufref_valid` answered yes.
    let prevbuf = prevbufref
        .get()
        .expect("the caller has just revalidated the buffer");
    if prevbuf.raw() == Win::current().w_buffer {
        reset_syntax(Win::current());
    }
    if unload
        || prev_nwindows <= 1
            && winid_before != last_winid()
            && action == DOBUF_GOTO as c_int
            && !may_hide(prevbuf)
    {
        close_all_windows(prevbuf, false);
    }
    // `close_windows` fires `WinClosed` and `BufWinLeave`, so ask again.
    let Some(prevbuf) = prevbufref.get().filter(|_| !aborting_now()) else {
        return;
    };
    // The address, not an identity: `close_buffer` below can free this window,
    // and `window_at` compares without reading it. Taking a `WinId` here would
    // be the tidier answer, but the C's own test is `curwin != previouswin`.
    let previouswin = Win::current_raw();

    // Do not sync when in Insert mode and the buffer is open in another
    // window, might be a timer doing something in another window.
    if prevbuf.raw() == Buf::current_raw()
        && (State.get() & MODE_INSERT == 0 || Buf::current().b_nwindows <= 1)
    {
        sync_undo();
    }
    // The window `prevbuf` is leaving, when it is the current one.
    let window = Win::current_or_none().filter(|win| prevbuf.raw() == win.w_buffer);
    let how = if unload {
        action
    } else if action == DOBUF_GOTO as c_int && !may_hide(prevbuf) && !is_changed(prevbuf) {
        DOBUF_UNLOAD as c_int
    } else {
        0
    };

    close_buffer(window, prevbuf, how, false, false);
    if Win::current_raw() != previouswin
        && let Some(previous) = window_at(previouswin)
    {
        // autocommands changed curwin, Grr!
        previous.make_current();
    }
}

/// Enter a new current buffer.
///
/// The old `curbuf` must have been abandoned already -- which also means it
/// may be pointing at freed memory, so nothing here reads it.
pub(crate) fn enter_buffer(mut buffer: Buf) {
    // when closing the current buffer stop Visual mode
    if visual_active() {
        end_visual();
    }

    // Get the buffer in the current window.
    let mut win = Win::current();
    win.w_buffer = buffer.raw();
    buffer.make_current();
    buffer.b_nwindows += 1;

    // Copy buffer and window local option values.  Not for a help buffer.
    copy_options_into(buffer, BCO_ENTER as c_int | BCO_NOHELP as c_int);
    if !buffer.b_help {
        restore_winopts(buffer);
    } else {
        // Remove all folds in the window.
        clear_window_folds(win);
    }
    invalidate_window_folds(win); // update folds (later).

    if win.w_onebuf_opt.wo_diff != 0 {
        diff_add(Buf::current());
    }

    win.w_s = &raw mut buffer.b_s;

    // Cursor on first line by default.
    let mut cursor = win.cursor();
    cursor.lnum = 1 as LineNr;
    cursor.col = 0 as ColNr;
    cursor.coladd = 0 as ColNr;
    win.w_set_curswant = true;
    win.w_topline_was_set = false;

    // mark cursor position as being invalid
    win.w_valid = WinValid::NONE;

    // Make sure the buffer is loaded.
    if buffer.b_ml.ml_mfp.is_null() {
        // need to load the file
        //
        // If there is no filetype, allow for detecting one.  Esp. useful for
        // ":ball" used in an autocommand.  If there already is a filetype we
        // might prefer to keep it.
        // SAFETY: `'filetype'` is a NUL-terminated option value.
        if unsafe { *buffer.b_p_ft } as c_int == NUL {
            buffer.b_did_filetype = false;
        }
        load_current_buffer();
    } else {
        if msg_silent.get() == 0 && !shortmess(ShmFlag::FILEINFO) {
            need_fileinfo.set(true); // display file info after redraw
        }
        check_timestamp(Buf::current()); // check if file changed

        let mut win = Win::current();
        win.w_topline = 1 as LineNr;
        win.w_topfill = 0;
        fire(AutoEvent::BufEnter, Buf::current());
        fire(AutoEvent::BufWinEnter, Buf::current());
    }

    // If autocommands did not change the cursor position, restore cursor lnum
    // and possibly cursor col.
    if Win::current().cursor().lnum == 1 as LineNr && cursor_in_indent() {
        restore_position();
    }

    recheck_arg_idx(Win::current()); // check for valid arg_idx
    rebuild_title();
    // when autocmds didn't change it
    let win = Win::current();
    if win.w_topline == 1 as LineNr && !win.w_topline_was_set {
        scroll_halfway(win); // redisplay at correct position
    }

    // Change directories when the 'acd' option is set.
    do_autochdir_now();

    if Buf::current().b_kmap_state as c_int & KEYMAP_INIT != 0 {
        init_keymap();
    }
    // May need to set the spell language.  Can only do this after the buffer
    // has been properly setup.
    let (buf, win) = (Buf::current(), Win::current());
    if !buf.b_help && win.w_onebuf_opt.wo_spell != 0 && has_spelllang(win) {
        set_spelllang(win);
    }
    Buf::current().b_last_used = now();

    if !Buf::current().terminal.is_null() {
        resize_terminal(Buf::current().terminal);
    }

    win.redraw_later(UPD_NOT_VALID);
}

/// Change to the directory of the current buffer, unless still starting up.
pub fn do_autochdir() {
    do_autochdir_now();
}

fn do_autochdir_now() {
    if p_acd.get() == 0 {
        return;
    }
    let fname = Buf::current().b_ffname;
    if starting.get() == 0 && !fname.is_null() && chdir_to_file(fname).is_ok() {
        last_chdir_reason.set(c"autochdir".as_ptr().cast_mut());
        reshorten_fnames();
    }
}

// ---------------------------------------------------------------------------
// "No write since last change"

pub fn no_write_message_buf(buffer: Buf) {
    if !buffer.terminal.is_null() && job_running(buffer) {
        err_static(e_job_still_running_add_bang_to_end_the_job);
    } else {
        let nr = buffer.handle as c_int;
        semsg!("E89: No write since last change for buffer {nr} (add ! to override)");
    }
}

pub fn no_write_message() {
    let buf = Buf::current();
    if !buf.terminal.is_null() && job_running(buf) {
        err_static(e_job_still_running_add_bang_to_end_the_job);
    } else {
        err_static(e_no_write_since_last_change_add_bang_to_override);
    }
}

pub fn no_write_message_nobang(buffer: Buf) {
    if !buffer.terminal.is_null() && job_running(buffer) {
        err_static(e_job_still_running);
    } else {
        err_static(e_no_write_since_last_change);
    }
}

/// `emsg(_(msg))` over one of `main.rs`'s message statics.
fn err_static(msg: &'static CStr) {
    err_raw(tr_raw(msg.as_ptr()));
}

// ---------------------------------------------------------------------------
// b:changedtick

/// `"changedtick"`, in the fixed-size key `DictItem` carries. The static
/// assertion upstream writes (`sizeof("changedtick") <= sizeof(di_key)`) is
/// the array length below.
const CHANGEDTICK_KEY: [c_char; 12] = {
    let mut key = [0 as c_char; 12];
    let name = b"changedtick";
    let mut i = 0;
    while i < name.len() {
        key[i] = name[i] as c_char;
        i += 1;
    }
    key
};

/// Initialise `b:changedtick` and its `changedtick_val` attribute.
pub(crate) fn buf_init_changedtick(mut buffer: Buf) {
    buffer.changedtick_di = ChangedtickDictItem {
        di_tv: TypVal {
            v_lock: VarLock::Fixed,
            ..TypVal::number(buf_get_changedtick(buffer))
        },
        // Must not include DI_FLAGS_ALLOC.
        di_flags: (DI_FLAGS_RO as c_int | DI_FLAGS_FIX as c_int) as uint8_t,
        di_key: CHANGEDTICK_KEY,
    };
    add_changedtick(buffer);
}
