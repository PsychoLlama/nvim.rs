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

use core::ffi::{c_char, c_int};
use core::ptr;

use super::*;
use crate::arglist::check_arg_idx;
use crate::autocmd::{EVENT_BUFENTER, EVENT_BUFLEAVE, EVENT_BUFWINENTER};
use crate::channel::channel_job_running;
use crate::diff::diff_buf_add;
use crate::digraph::keymap_init;
use crate::drawscreen::UPD_NOT_VALID;
use crate::eval::typval::tv_dict_add;
use crate::ex_docmd::cmdmod_has;
use crate::file_search::vim_chdirfile;
use crate::fileio::{buf_check_timestamp, shorten_fnames};
use crate::indent::inindent;
use crate::main::{
    State, VIsual_reselect, curbuf, curwin, e_job_still_running,
    e_job_still_running_add_bang_to_end_the_job, e_no_write_since_last_change,
    e_no_write_since_last_change_add_bang_to_override,
    e_no_write_since_last_change_for_buffer_nr_add_bang_to_override, last_chdir_reason, msg_silent,
    need_fileinfo, p_acd, starting,
};
use crate::r#move::{WinValid, scroll_cursor_halfway};
use crate::normal::visual_active;
use crate::option::buf_copy_options;
use crate::spell::parse_spelllang;
use crate::state::MODE_INSERT;
use crate::terminal::terminal_check_size;
use crate::types::{
    ChangedtickDictItem, CmdModFlags, NUL, OK, OptInt, ShmFlag, Terminal, VAR_FIXED, VAR_NUMBER,
    colnr_T, dictitem_T, linenr_T, time_t, typval_T, typval_vval_union, uint8_t, uint64_t, win_T,
};
use crate::undo::u_sync;
use crate::window::{get_last_winid, win_valid};
use ::libc::time;

// ---------------------------------------------------------------------------
// The neighbours, wrapped

/// The id the last window created was given -- the C's cheap "did an
/// autocommand open a window?" probe.
fn last_winid() -> c_int {
    get_last_winid()
}

/// The window `win` names, if it is still in the current tab page.
///
/// `win_valid` walks the window list comparing pointers and never
/// dereferences its argument, so asking about a possibly-closed window is a
/// safe operation.
fn valid_win(win: *mut win_T) -> Option<Win> {
    // SAFETY: the pointer is only compared; a hit means a live window.
    unsafe { win_valid(win).then(|| Win::new(win)) }
}

/// Whether `buf` may stay loaded when it is no longer shown -- `'hidden'`,
/// `'bufhidden'` or a `:hide` modifier.
fn may_hide(mut buf: Buf) -> bool {
    // SAFETY: a live buffer.
    unsafe { buf_hide(buf.raw()) }
}

/// Sync the undo state, so that what follows starts a new change.
fn sync_undo() {
    // SAFETY: reads the current buffer's undo tree.
    unsafe { u_sync(false) };
}

/// Remember `win`'s cursor position for the alternate file.
fn remember_altfpos(mut win: Win) {
    // SAFETY: a live window.
    unsafe { buflist_altfpos(win.raw()) };
}

/// Restore the window-local options `win` last used with this buffer.
fn restore_winopts(mut buf: Buf) {
    // SAFETY: a live buffer.
    unsafe { get_winopts(buf.raw()) };
}

/// Copy the buffer-local option values into `buf`.
fn copy_options_into(mut buf: Buf, flags: c_int) {
    // SAFETY: a live buffer.
    unsafe { buf_copy_options(buf.raw(), flags) };
}

fn diff_add(mut buf: Buf) {
    // SAFETY: a live buffer.
    unsafe { diff_buf_add(buf.raw()) };
}

/// Load the buffer that has just been made current.
fn load_current_buffer() {
    // SAFETY: `curbuf` and `curwin` are set; a null `eap` is the no-command form.
    unsafe { open_buffer(false, ptr::null_mut(), 0) };
}

/// Warn if the file changed on disk since the buffer was read.
fn check_timestamp(mut buf: Buf) {
    // SAFETY: a live buffer; `false` is upstream's `focus` flag.
    unsafe { buf_check_timestamp(buf.raw()) };
}

/// Whether the cursor is in the indent of its line.
fn cursor_in_indent() -> bool {
    // SAFETY: reads the current window's cursor and line.
    unsafe { inindent(0) }
}

/// Put the cursor back where this window last was in this buffer.
fn restore_position() {
    // SAFETY: reads the current window and buffer, both set.
    unsafe { buflist_getfpos() };
}

/// Re-check the argument-list index after the buffer changed.
fn recheck_arg_idx(mut win: Win) {
    // SAFETY: a live window.
    unsafe { check_arg_idx(win.raw()) };
}

/// Rebuild `'title'` and `'icon'`.
fn rebuild_title() {
    // SAFETY: reads the current window and buffer.
    unsafe { maketitle() };
}

/// Scroll so that the cursor line sits in the middle of the window.
fn scroll_halfway(mut win: Win) {
    // SAFETY: a live window.
    unsafe { scroll_cursor_halfway(win.raw(), false, false) };
}

/// Load the keymap `'keymap'` names.
fn init_keymap() {
    keymap_init();
}

/// Work out the spell-checking languages for `win`.
fn set_spelllang(mut win: Win) {
    // SAFETY: a live window with a syntax block.
    unsafe { parse_spelllang(win.raw()) };
}

/// Whether the window's `'spelllang'` is set. It lives in the syntax block
/// the window shares with its buffer.
fn has_spelllang(mut win: Win) -> bool {
    // SAFETY: a live window's syntax block is live, and `'spelllang'` a
    // NUL-terminated option value.
    unsafe { *(*win.w_s).b_p_spl as c_int != NUL }
}

fn resize_terminal(term: *mut Terminal) {
    // SAFETY: a live terminal, the caller having ruled out null.
    unsafe { terminal_check_size(term) };
}

/// Whether the job behind terminal buffer `buf` is still running.
fn job_running(mut buf: Buf) -> bool {
    // SAFETY: reads the buffer's `'channel'` and looks it up.
    unsafe { channel_job_running(buf.b_p_channel as uint64_t) }
}

/// Change to the directory of `fname`.
fn chdir_to_file(fname: *mut c_char) -> c_int {
    // SAFETY: a NUL-terminated file name.
    unsafe { vim_chdirfile(fname, kCdCauseAuto) }
}

/// Recompute every buffer's short file name against the new directory.
fn reshorten_fnames() {
    // SAFETY: walks the buffer list only.
    unsafe { shorten_fnames(1) };
}

/// The wall clock, for `b_last_used`.
fn now() -> time_t {
    // SAFETY: a null argument asks for the answer by value.
    unsafe { time(ptr::null_mut::<time_t>()) }
}

/// Add `b:changedtick` to the buffer's variable dictionary.
fn add_changedtick(mut buf: Buf) {
    let (vars, di) = (buf.b_vars, &raw mut buf.changedtick_di as *mut dictitem_T);
    // SAFETY: a live buffer's dictionary, and its own `changedtick` item.
    unsafe { tv_dict_add(vars, di) };
}

// ---------------------------------------------------------------------------
// Leaving one buffer for another

/// Make `buf` the current buffer, closing the one being left as `action` says
/// (`DOBUF_GOTO` frees or hides it, `DOBUF_SPLIT` leaves it alone, and
/// `DOBUF_UNLOAD`/`DEL`/`WIPE` do what they say).
///
/// With `update_jumplist` the position being left joins the jump list.
///
/// # Safety
/// `buf` must be a live buffer, and `curbuf`/`curwin` be set.
pub unsafe fn set_curbuf(buf: *mut buf_T, action: c_int, update_jumplist: bool) {
    // SAFETY: the caller's promise -- a live buffer.
    let buf = unsafe { Buf::new(buf) };
    let unload = action == DOBUF_UNLOAD as c_int
        || action == DOBUF_DEL as c_int
        || action == DOBUF_WIPE as c_int;
    let old_tw: OptInt = cur_buf().b_p_tw;
    let winid_before = last_winid();

    if update_jumplist {
        set_pcmark();
    }

    let mut win = cur_win();
    if !cmdmod_has(CmdModFlags::KEEPALT) {
        win.w_alt_fnum = cur_buf().handle as c_int; // remember alternate file
    }
    remember_altfpos(win); // remember curpos

    // Don't restart Select mode after switching to another buffer.
    VIsual_reselect.set(0);

    // close_windows() or apply_autocmds() may change curbuf and wipe out "buf"
    let prevbuf = cur_buf();
    let prevbufref = BufRef::of(prevbuf);
    let newbufref = BufRef::of(buf);
    let prev_nwindows = prevbuf.b_nwindows;
    // The re-entry rule: `buf` is about to be held across two calls that can
    // wipe it, so its identity is taken now, while it is provably live.
    let buf_id = buf.id();

    // Autocommands may delete the current buffer and/or the buffer we want to
    // go to.  In those cases don't close the buffer.
    if !fire(EVENT_BUFLEAVE, cur_buf())
        || prevbufref.valid() && newbufref.valid() && !aborting_now()
    {
        leave_prevbuf(prevbufref, action, unload, prev_nwindows, winid_before);
    }

    // An autocommand may have deleted "buf", already entered it (e.g., when it
    // did ":bunload") or aborted the script processing!  If curwin->w_buffer is
    // null, enter_buffer() will make it valid again.
    // The other half of the rule: ask the registry by the identity taken
    // above, not the buffer list by `buf`'s address. Stricter, too — a buffer
    // wiped and a new one allocated at the same address would pass an address
    // comparison, the hazard `bufref_T` carries `br_buf_free_count` for.
    let valid = buf_id.valid();
    if valid && buf.raw() != curbuf.get() && !aborting_now() || cur_win().w_buffer.is_null() {
        // autocommands changed curbuf and we will move to another buffer soon,
        // so decrement curbuf->b_nwindows
        if let Some(mut cur) = current_buf().filter(|c| *c != prevbuf) {
            cur.b_nwindows -= 1;
        }
        // If the buffer is not valid but curwin->w_buffer is NULL we must enter
        // some buffer.  Using the last one is hopefully OK.
        enter_buffer(if valid {
            buf
        } else {
            last_buf().expect("lastbuf != NULL")
        });
        if old_tw != cur_buf().b_p_tw {
            recheck_colorcolumn(cur_win());
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
    let prevraw = prevbufref.raw();
    if prevraw == cur_win().w_buffer {
        reset_syntax(cur_win());
    }
    // autocommands may have opened a new window with prevbuf, grr
    // SAFETY: the caller's guard has just said `prevbuf` is still the buffer
    // it was -- either `BufLeave` ran nothing, or `bufref_valid` answered yes.
    let prevbuf = unsafe { Buf::new(prevraw) };
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
    let previouswin = curwin.get();

    // Do not sync when in Insert mode and the buffer is open in another
    // window, might be a timer doing something in another window.
    if prevraw == curbuf.get() && (State.get() & MODE_INSERT == 0 || cur_buf().b_nwindows <= 1) {
        sync_undo();
    }
    let win = if prevraw == cur_win().w_buffer {
        curwin.get()
    } else {
        ptr::null_mut::<win_T>()
    };
    let how = if unload {
        action
    } else if action == DOBUF_GOTO as c_int && !may_hide(prevbuf) && !is_changed(prevbuf) {
        DOBUF_UNLOAD as c_int
    } else {
        0
    };
    // SAFETY: `prevbuf` is still live, the guard above having said so.
    unsafe { close_buffer(win, prevraw, how, false, false) };
    if curwin.get() != previouswin && valid_win(previouswin).is_some() {
        // autocommands changed curwin, Grr!
        curwin.set(previouswin);
    }
}

/// Enter a new current buffer.
///
/// The old `curbuf` must have been abandoned already -- which also means it
/// may be pointing at freed memory, so nothing here reads it.
pub(crate) fn enter_buffer(mut buf: Buf) {
    // when closing the current buffer stop Visual mode
    if visual_active() {
        end_visual();
    }

    // Get the buffer in the current window.
    let mut win = cur_win();
    win.w_buffer = buf.raw();
    curbuf.set(buf.raw());
    buf.b_nwindows += 1;

    // Copy buffer and window local option values.  Not for a help buffer.
    copy_options_into(buf, BCO_ENTER as c_int | BCO_NOHELP as c_int);
    if !buf.b_help {
        restore_winopts(buf);
    } else {
        // Remove all folds in the window.
        clear_window_folds(win);
    }
    invalidate_window_folds(win); // update folds (later).

    if win.w_onebuf_opt.wo_diff != 0 {
        diff_add(cur_buf());
    }

    win.w_s = &raw mut buf.b_s;

    // Cursor on first line by default.
    let mut cursor = win.cursor();
    cursor.lnum = 1 as linenr_T;
    cursor.col = 0 as colnr_T;
    cursor.coladd = 0 as colnr_T;
    win.w_set_curswant = true;
    win.w_topline_was_set = false;

    // mark cursor position as being invalid
    win.w_valid = WinValid::NONE;

    // Make sure the buffer is loaded.
    if buf.b_ml.ml_mfp.is_null() {
        // need to load the file
        //
        // If there is no filetype, allow for detecting one.  Esp. useful for
        // ":ball" used in an autocommand.  If there already is a filetype we
        // might prefer to keep it.
        // SAFETY: `'filetype'` is a NUL-terminated option value.
        if unsafe { *buf.b_p_ft } as c_int == NUL {
            buf.b_did_filetype = false;
        }
        load_current_buffer();
    } else {
        if msg_silent.get() == 0 && !shortmess(ShmFlag::FILEINFO) {
            need_fileinfo.set(true); // display file info after redraw
        }
        check_timestamp(cur_buf()); // check if file changed

        let mut win = cur_win();
        win.w_topline = 1 as linenr_T;
        win.w_topfill = 0;
        fire(EVENT_BUFENTER, cur_buf());
        fire(EVENT_BUFWINENTER, cur_buf());
    }

    // If autocommands did not change the cursor position, restore cursor lnum
    // and possibly cursor col.
    if cur_win().cursor().lnum == 1 as linenr_T && cursor_in_indent() {
        restore_position();
    }

    recheck_arg_idx(cur_win()); // check for valid arg_idx
    rebuild_title();
    // when autocmds didn't change it
    let win = cur_win();
    if win.w_topline == 1 as linenr_T && !win.w_topline_was_set {
        scroll_halfway(win); // redisplay at correct position
    }

    // Change directories when the 'acd' option is set.
    do_autochdir_now();

    if cur_buf().b_kmap_state as c_int & KEYMAP_INIT != 0 {
        init_keymap();
    }
    // May need to set the spell language.  Can only do this after the buffer
    // has been properly setup.
    let (buf, win) = (cur_buf(), cur_win());
    if !buf.b_help && win.w_onebuf_opt.wo_spell != 0 && has_spelllang(win) {
        set_spelllang(win);
    }
    cur_buf().b_last_used = now();

    if !cur_buf().terminal.is_null() {
        resize_terminal(cur_buf().terminal);
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
    let fname = cur_buf().b_ffname;
    if starting.get() == 0 && !fname.is_null() && chdir_to_file(fname) == OK {
        last_chdir_reason.set(c"autochdir".as_ptr().cast_mut());
        reshorten_fnames();
    }
}

// ---------------------------------------------------------------------------
// "No write since last change"

/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn no_write_message_buf(buf: *mut buf_T) {
    // SAFETY: the caller's promise -- a live buffer.
    let mut buf = unsafe { Buf::new(buf) };
    if !buf.terminal.is_null() && job_running(buf) {
        err_static(&raw const e_job_still_running_add_bang_to_end_the_job);
    } else {
        let fmt = &raw const e_no_write_since_last_change_for_buffer_nr_add_bang_to_override;
        err_num(tr_raw(fmt.cast::<c_char>()), buf.handle as c_int);
    }
}

pub fn no_write_message() {
    let buf = cur_buf();
    if !buf.terminal.is_null() && job_running(buf) {
        err_static(&raw const e_job_still_running_add_bang_to_end_the_job);
    } else {
        err_static(&raw const e_no_write_since_last_change_add_bang_to_override);
    }
}

/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn no_write_message_nobang(buf: *const buf_T) {
    // SAFETY: the caller's promise -- a live buffer, which is only read.
    let buf = unsafe { Buf::new(buf.cast_mut()) };
    if !buf.terminal.is_null() && job_running(buf) {
        err_static(&raw const e_job_still_running);
    } else {
        err_static(&raw const e_no_write_since_last_change);
    }
}

/// `emsg(_(msg))` over one of `main.rs`'s message statics, which are byte
/// arrays rather than pointers.
fn err_static<const N: usize>(msg: *const [c_char; N]) {
    err_raw(tr_raw(msg.cast::<c_char>()));
}

// ---------------------------------------------------------------------------
// b:changedtick

/// `"changedtick"`, in the fixed-size key `dictitem_T` carries. The static
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
pub(crate) fn buf_init_changedtick(mut buf: Buf) {
    buf.changedtick_di = ChangedtickDictItem {
        di_tv: typval_T {
            v_type: VAR_NUMBER,
            v_lock: VAR_FIXED,
            vval: typval_vval_union {
                // SAFETY: a live buffer.
                v_number: unsafe { buf_get_changedtick(buf.raw()) },
            },
        },
        // Must not include DI_FLAGS_ALLOC.
        di_flags: (DI_FLAGS_RO as c_int | DI_FLAGS_FIX as c_int) as uint8_t,
        di_key: CHANGEDTICK_KEY,
    };
    add_changedtick(buf);
}
