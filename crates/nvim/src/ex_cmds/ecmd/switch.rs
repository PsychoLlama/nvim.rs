//! Making another file's buffer the one the current window shows.
//!
//! This is the half of [`do_ecmd`](super::do_ecmd) that can run arbitrary
//! Vimscript at four points -- `buf_check_timestamp`, BufLeave, `close_buffer`
//! (which fires BufUnload/BufDelete/BufWipeout) and `buf_copy_options` -- and
//! that therefore has to re-check after every one of them whether the buffer
//! it is heading for still exists, whether the current buffer is still the one
//! it left, and whether the script was aborted.  Each of those checks is
//! upstream's, in upstream's place: the sequence is the contract.
//!
//! Original: `src/nvim/ex_cmds.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{Ecmd, EcmdArgs};
use crate::autocmd::EVENT_BUFLEAVE;
use crate::buffer::current_buf;
use crate::buffer::{
    BufRef, buf_valid, buflist_altfpos, buflist_findfmark, buflist_new, close_buffer, find_buf,
    get_winopts,
};
use crate::ex_cmds::{
    BCO_ENTER, BLN_CURBUF, BLN_LISTED, BLN_NOCURWIN, DOBUF_UNLOAD, ECMD_ADDBUF, ECMD_ALTBUF,
    ECMD_HIDE, ECMD_LAST, ECMD_LASTL, ECMD_SET_HELP, buf_autocmd,
};
use crate::ex_docmd::cmdmod_has;
use crate::ex_eval::aborting;
use crate::fileio::{buf_check_timestamp, set_file_options, set_forced_fenc};
use crate::main::{
    au_new_curbuf, cmdwin_buf, cmdwin_old_curwin, cmdwin_type, cmdwin_win, curbuf, curwin,
    e_cannot_switch_to_a_closing_buffer,
};
use crate::memory::{xfree, xstrdup};
use crate::message::emsg;
use crate::option::buf_copy_options;
use crate::os::cshim::gettext;
use crate::semsg_c;
use crate::terminal::terminal_running;
use crate::types::{CmdModFlags, linenr_T, win_T};
use crate::undo::u_sync;
use crate::window::{win_valid, win_valid_any_tab};
use crate::winlayer::{Buf, Win};
use ::libc::atol;
use core::ffi::{c_char, c_int};
use core::ptr;

/// What the "edit another file" stage decided.
pub(super) enum Switch {
    /// The buffer is in place; carry on.
    Ready,
    /// Give up and go to the cleanup.
    Abandon,
}

/// Make the target file's buffer the one the current window shows, firing
/// BufLeave for the old one and closing it when it is no longer wanted.
///
/// # Safety
/// The names, `eap` and `oldwin` must be live or NULL, and `old_curbuf` must
/// be the bufref taken on entry to [`do_ecmd`]. `oldwin` is an out-parameter
/// the caller re-checks with `win_valid` after the autocommands below, so it
/// stays a raw pointer -- a [`Win`] would promise the liveness that check
/// exists to doubt.
pub(super) unsafe fn switch_to_other_buffer(
    args: &EcmdArgs,
    oldwin: &mut *mut win_T,
    old_curbuf: &mut BufRef,
    state: &mut Ecmd,
) -> Switch {
    let EcmdArgs {
        fnum,
        ffname,
        sfname,
        flags,
        command,
        ..
    } = *args;
    // SAFETY: `curwin` is live.
    let prev_alt_fnum = cur_win().w_alt_fnum;

    if flags & (ECMD_ADDBUF as c_int | ECMD_ALTBUF as c_int) == 0 {
        // SAFETY: `curwin`/`curbuf` are live, and `oldwin` was validated.
        if !cmdmod_has(CmdModFlags::KEEPALT) {
            cur_win().w_alt_fnum = cur_buf().handle;
        }
        if !oldwin.is_null() {
            unsafe { buflist_altfpos(Win::new(*oldwin)) };
        }
    }

    let buf;
    if fnum != 0 {
        buf = find_buf(fnum).map_or(ptr::null_mut(), |mut b| b.raw());
    } else if flags & (ECMD_ADDBUF as c_int | ECMD_ALTBUF as c_int) != 0 {
        // Default the line number to zero to avoid that a wininfo item is
        // added for the current window.  Add BLN_NOCURWIN for the same reason.
        // SAFETY: `command` and the names are live when non-NULL.
        let mut tlnum = 0;
        if !command.is_null() {
            tlnum = unsafe { atol(command) } as linenr_T;
            if tlnum <= 0 {
                tlnum = 1;
            }
        }
        let newbuf = unsafe {
            buflist_new(
                ffname,
                sfname,
                tlnum,
                BLN_LISTED as c_int | BLN_NOCURWIN as c_int,
            )
        };
        if !newbuf.is_null() && flags & ECMD_ALTBUF as c_int != 0 {
            cur_win().w_alt_fnum = unsafe { (*newbuf).handle };
        }
        return Switch::Abandon;
    } else {
        // SAFETY: the names are live when non-NULL.
        buf = unsafe {
            buflist_new(
                ffname,
                sfname,
                0,
                BLN_CURBUF as c_int
                    | (if flags & ECMD_SET_HELP as c_int != 0 {
                        0
                    } else {
                        BLN_LISTED as c_int
                    }),
            )
        };
        // Autocmds may change curwin and curbuf.
        if !oldwin.is_null() {
            *oldwin = curwin.get();
        }
        *old_curbuf = BufRef::of_opt(current_buf());
    }

    if buf.is_null() {
        return Switch::Abandon;
    }
    // Autocommands try to edit a closing buffer, which -- like splitting --
    // can result in more windows displaying it; abort.
    // SAFETY: `buf` and `curwin` are live.
    if unsafe { (*buf).b_locked_split } != 0 {
        // SAFETY: as above.
        // The window was split, but is not editing the new buffer; reset
        // b_nwindows again.
        if oldwin.is_null()
            && !cur_win().w_buffer.is_null()
            && unsafe { (*cur_win().w_buffer).b_nwindows } > 1
        {
            unsafe { (*cur_win().w_buffer).b_nwindows -= 1 };
        }
        unsafe {
            emsg(gettext(
                &raw const e_cannot_switch_to_a_closing_buffer as *const c_char,
            ))
        };
        return Switch::Abandon;
    }

    // SAFETY: `buf` and `curwin` are live.
    if cur_win().w_alt_fnum == unsafe { (*buf).handle } && prev_alt_fnum != 0 {
        // reusing the buffer, keep the old alternate file
        cur_win().w_alt_fnum = prev_alt_fnum;
    }

    // SAFETY: `buf` is live.
    if unsafe { (*buf).b_ml.ml_mfp.is_null() } {
        // No memfile yet.
        state.oldbuf = false;
    } else {
        // Existing memfile.
        state.oldbuf = true;
        // SAFETY: as above.
        let bufref = BufRef::of_opt(unsafe { Buf::from_raw(buf) });
        unsafe { buf_check_timestamp(Buf::new(buf)) };
        // Check if autocommands made the buffer invalid or changed the
        // current buffer; they may also abort script processing.
        if !bufref.valid() || curbuf.get() != old_curbuf.raw() || aborting() {
            return Switch::Abandon;
        }
    }

    // May jump to last used line number for a loaded buffer or when asked for
    // explicitly.
    if (state.oldbuf && state.newlnum == ECMD_LASTL as linenr_T)
        || state.newlnum == ECMD_LAST as linenr_T
    {
        // SAFETY: `buf` is live.
        let pos = unsafe { &raw mut (*buflist_findfmark(Buf::new(buf))).mark };
        state.newlnum = unsafe { (*pos).lnum };
        state.solcol = unsafe { (*pos).col };
    }

    // Make the (new) buffer the one used by the current window.  If the old
    // buffer becomes unused, free it if ECMD_HIDE is false.  If the current
    // buffer was empty and has no file name, curbuf is returned by
    // buflist_new(), and there is nothing to do here.
    if buf != curbuf.get() {
        // SAFETY: the editor's own state.
        match unsafe { leave_for_buffer(Buf::new(buf), args, *oldwin, old_curbuf, state) } {
            Switch::Abandon => return Switch::Abandon,
            Switch::Ready => {}
        }
    }
    Switch::Ready
}

/// Fire BufLeave for the buffer being left, close it if it is no longer
/// wanted, and make `buf` the current window's.
///
/// # Safety
/// `buf` must be different from the current buffer; `eap` and `oldwin` may be
/// NULL. `oldwin` stays a raw pointer on purpose: the autocommands below may
/// close that window, and `win_valid_any_tab` is what asks -- comparing an
/// address that a [`Win`] would have promised was live.
unsafe fn leave_for_buffer(
    mut buf: Buf,
    args: &EcmdArgs,
    oldwin: *mut win_T,
    old_curbuf: &mut BufRef,
    state: &mut Ecmd,
) -> Switch {
    let (eap, flags) = (args.eap, args.flags);
    // Should only be possible to get here if the cmdwin is closed, or if it's
    // opening and its buffer hasn't been set yet (the new buffer is for it).
    debug_assert!(cmdwin_buf.get().is_null(), "cmdwin_buf == NULL");

    let save_cmdwin_type = cmdwin_type.get();
    let save_cmdwin_win = cmdwin_win.get();
    let save_cmdwin_old_curwin = cmdwin_old_curwin.get();

    // BufLeave applies to the old buffer.
    cmdwin_type.set(0);
    cmdwin_win.set(ptr::null_mut());
    cmdwin_old_curwin.set(ptr::null_mut());

    // Be careful: the autocommands may delete any buffer and change the
    // current buffer.
    // - If the buffer we are going to edit is deleted, give up.
    // - If the current buffer is deleted, prefer to load the new buffer when
    //   loading a buffer is required.  This avoids loading another buffer
    //   which then must be closed again.
    // - If we ended up in the new buffer already, need to skip a few things,
    //   set auto_buf.
    // SAFETY: the buffer's own file name, and `new_name` becomes ours.
    let new_name = if buf.b_fname.is_null() {
        ptr::null_mut()
    } else {
        unsafe { xstrdup(buf.b_fname) }
    };
    let save_au_new_curbuf = au_new_curbuf.get();
    au_new_curbuf.set(BufRef::of(buf).record());
    buf_autocmd(EVENT_BUFLEAVE, cur_buf());

    cmdwin_type.set(save_cmdwin_type);
    cmdwin_win.set(save_cmdwin_win);
    cmdwin_old_curwin.set(save_cmdwin_old_curwin);

    if !au_new_curbuf_valid() {
        // New buffer has been deleted.  `delbuf_msg` frees `new_name`.
        // SAFETY: `new_name` is ours.
        unsafe { delbuf_msg(new_name) };
        au_new_curbuf.set(save_au_new_curbuf);
        return Switch::Abandon;
    }
    if aborting() {
        // autocmds may abort script processing
        // SAFETY: `new_name` is ours.
        unsafe { xfree(new_name.cast()) };
        au_new_curbuf.set(save_au_new_curbuf);
        return Switch::Abandon;
    }

    if buf.raw() == curbuf.get() {
        // already in new buffer
        state.auto_buf = true;
        // SAFETY: `new_name` is ours.
        unsafe { xfree(new_name.cast()) };
        au_new_curbuf.set(save_au_new_curbuf);
        return Switch::Ready;
    }

    let the_curwin = curwin.get();
    let was_curbuf = curbuf.get();

    // SAFETY: the windows and buffers are the editor's own and live.
    let did_decrement = unsafe {
        // Set w_locked to avoid that autocommands close the window.  Set
        // b_locked for the same reason.
        (*the_curwin).w_locked = true;
        buf.b_locked += 1;

        if curbuf.get() == old_curbuf.raw() {
            buf_copy_options(buf.raw(), BCO_ENTER as c_int);
        }

        // Close the link to the current buffer.  This will set
        // oldwin->w_buffer to NULL.
        u_sync(false);
        close_buffer(
            Win::from_raw(oldwin),
            Buf::current(),
            if flags & ECMD_HIDE as c_int != 0
                || !cur_buf().terminal.is_null() && terminal_running(cur_buf().terminal)
            {
                0
            } else {
                DOBUF_UNLOAD as c_int
            },
            false,
            false,
        )
    };

    // SAFETY: `win_valid` tolerates a stale window pointer.
    // Autocommands may have closed the window.
    if win_valid(the_curwin) {
        unsafe { (*the_curwin).w_locked = false };
    }
    buf.b_locked -= 1;

    // autocmds may abort script processing
    // SAFETY: `curwin` is live.
    if aborting() && !cur_win().w_buffer.is_null() {
        // SAFETY: `new_name` is ours.
        unsafe { xfree(new_name.cast()) };
        au_new_curbuf.set(save_au_new_curbuf);
        return Switch::Abandon;
    }
    // Be careful again, like above.
    if !au_new_curbuf_valid() {
        // New buffer has been deleted.  `delbuf_msg` frees `new_name`.
        // SAFETY: `new_name` is ours.
        unsafe { delbuf_msg(new_name) };
        au_new_curbuf.set(save_au_new_curbuf);
        return Switch::Abandon;
    }

    // SAFETY: the windows and buffers are live; `eap` is the caller's.
    if buf.raw() == curbuf.get() {
        // already in new buffer -- close_buffer() has decremented the
        // window count, increment it again here and restore w_buffer.
        if did_decrement && unsafe { buf_valid(was_curbuf) } {
            unsafe { (*was_curbuf).b_nwindows += 1 };
        }
        if win_valid_any_tab(oldwin) && unsafe { (*oldwin).w_buffer.is_null() } {
            unsafe { (*oldwin).w_buffer = was_curbuf };
        }
        state.auto_buf = true;
    } else {
        // <VN> We could instead free the synblock and re-attach to the
        // buffer, perhaps.
        if cur_win().w_buffer.is_null()
            || cur_win().w_s == unsafe { &raw mut (*cur_win().w_buffer).b_s }
        {
            cur_win().w_s = &raw mut buf.b_s;
        }

        cur_win().w_buffer = buf.raw();
        curbuf.set(buf.raw());
        cur_buf().b_nwindows += 1;

        // Set 'fileformat', 'binary' and 'fenc' when forced.
        if !state.oldbuf && !eap.is_null() {
            unsafe { set_file_options(true, eap) };
            unsafe { set_forced_fenc(eap) };
        }
    }

    // May get the window options from the last time this buffer was in
    // this window (or another window).  If not used before, reset the
    // local window options to the global values.  Also restores old
    // folding stuff.
    unsafe { get_winopts(Buf::current()) };
    state.did_get_winopts = true;

    unsafe { xfree(new_name.cast()) };
    au_new_curbuf.set(save_au_new_curbuf);
    Switch::Ready
}

/// Is the buffer `au_new_curbuf` names still alive?
fn au_new_curbuf_valid() -> bool {
    BufRef::of_record(au_new_curbuf.get()).valid()
}
/// An autocommand deleted the buffer that was about to be edited.
///
/// # Safety
/// `name` must be our own allocation, or NULL; it is freed here.
pub(super) unsafe fn delbuf_msg(name: *mut c_char) {
    // SAFETY: caller's contract; one `%s` for one string.
    unsafe {
        semsg_c!(
            gettext(c"E143: Autocommands unexpectedly deleted new buffer %s".as_ptr()),
            if name.is_null() {
                c"".as_ptr()
            } else {
                name as *const c_char
            },
        )
    };
    unsafe { xfree(name.cast()) };
    au_new_curbuf.with_mut(|r| {
        r.br_buf = ptr::null_mut();
        r.br_buf_free_count = 0;
    });
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
