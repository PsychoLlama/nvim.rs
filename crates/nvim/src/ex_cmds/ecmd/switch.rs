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
#![allow(unsafe_code)]

use super::{Ecmd, EcmdArgs};
use crate::ex_cmds::EcmdFlags;
use crate::ex_cmds::newlnum;
use crate::message_fmt::msg_cstr;
use crate::types::AutoEvent;
use crate::window::valid_win;
use crate::winlayer::WinId;
use core::ffi::CStr;
use std::ffi::CString;

use crate::autocmd::state::au_new_curbuf;
use crate::buffer::current_buf;
use crate::buffer::{
    BufRef, buf_valid, buflist_altfpos, buflist_findfmark, buflist_new, close_buffer, find_buf,
    get_winopts,
};
use crate::ex_cmds::{BCO_ENTER, BLN_CURBUF, BLN_LISTED, BLN_NOCURWIN, DOBUF_UNLOAD, buf_autocmd};
use crate::ex_docmd::cmdmod_has;
use crate::ex_eval::aborting;
use crate::fileio::{buf_check_timestamp, set_file_options, set_forced_fenc};
use crate::message::e_cannot_switch_to_a_closing_buffer;
use crate::message::emsg;
use crate::option::buf_copy_options;
use crate::os::cshim::gettext;
use crate::semsg;
use crate::terminal::terminal_running;
use crate::types::{CmdModFlags, LineNr};
use crate::undo::u_sync;
use crate::window::win_valid_any_tab;
use crate::winlayer::graph::{cmdwin_buf, cmdwin_old_curwin, cmdwin_type, cmdwin_win};
use crate::winlayer::{Buf, Win};
use ::libc::atol;
use core::ffi::c_int;
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
pub(super) fn switch_to_other_buffer(
    args: &mut EcmdArgs<'_>,
    oldwin: &mut Option<WinId>,
    old_curbuf: &mut BufRef,
    state: &mut Ecmd,
) -> Switch {
    let (fnum, ffname, sfname, flags, command) = (
        args.fnum,
        args.ffname,
        args.sfname,
        args.flags,
        args.command,
    );
    // SAFETY: `curwin` is live.
    let prev_alt_fnum = Win::current().w_alt_fnum;

    if !flags.has(EcmdFlags::ADDBUF | EcmdFlags::ALTBUF) {
        // SAFETY: `curwin`/`curbuf` are live, and `oldwin` was validated.
        if !cmdmod_has(CmdModFlags::KEEPALT) {
            Win::current().w_alt_fnum = Buf::current().handle;
        }
        if let Some(old) = oldwin.and_then(WinId::get) {
            buflist_altfpos(old);
        }
    }

    let buf;
    if fnum != 0 {
        buf = find_buf(fnum).map_or(ptr::null_mut(), |b| b.raw());
    } else if flags.has(EcmdFlags::ADDBUF | EcmdFlags::ALTBUF) {
        // Default the line number to zero to avoid that a wininfo item is
        // added for the current window.  Add BLN_NOCURWIN for the same reason.
        // SAFETY: `command` and the names are live when non-NULL.
        let mut tlnum = 0;
        if !command.is_null() {
            tlnum = unsafe { atol(command) } as LineNr;
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
        if let Some(newbuf) = newbuf.filter(|_| flags.has(EcmdFlags::ALTBUF)) {
            Win::current().w_alt_fnum = newbuf.handle;
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
                    | (if flags.has(EcmdFlags::SET_HELP) {
                        0
                    } else {
                        BLN_LISTED as c_int
                    }),
            )
            .map_or(ptr::null_mut(), Buf::raw)
        };
        // Autocmds may change curwin and curbuf.
        if oldwin.is_some() {
            *oldwin = Win::current_or_none().map(Win::id);
        }
        *old_curbuf = BufRef::of_opt(current_buf());
    }

    if buf.is_null() {
        return Switch::Abandon;
    }
    // SAFETY: not null, and the guard above is what says so.
    let buffer = unsafe { Buf::new(buf) };
    // Autocommands try to edit a closing buffer, which -- like splitting --
    // can result in more windows displaying it; abort.
    if buffer.b_locked_split != 0 {
        // SAFETY: as above.
        // The window was split, but is not editing the new buffer; reset
        // b_nwindows again.
        if oldwin.is_none()
            && !Win::current().w_buffer.is_null()
            && unsafe { (*Win::current().w_buffer).b_nwindows } > 1
        {
            unsafe { (*Win::current().w_buffer).b_nwindows -= 1 };
        }
        emsg(gettext(e_cannot_switch_to_a_closing_buffer));
        return Switch::Abandon;
    }

    if Win::current().w_alt_fnum == buffer.handle && prev_alt_fnum != 0 {
        // reusing the buffer, keep the old alternate file
        Win::current().w_alt_fnum = prev_alt_fnum;
    }

    if buffer.b_ml.ml_mfp.is_null() {
        // No memfile yet.
        state.oldbuf = false;
    } else {
        // Existing memfile.
        state.oldbuf = true;
        let bufref = BufRef::of(buffer);
        unsafe { buf_check_timestamp(buffer) };
        // Check if autocommands made the buffer invalid or changed the
        // current buffer; they may also abort script processing.
        if !bufref.valid() || !old_curbuf.is(Buf::current_or_none()) || aborting() {
            return Switch::Abandon;
        }
    }

    // May jump to last used line number for a loaded buffer or when asked for
    // explicitly.
    if (state.oldbuf && state.newlnum == newlnum::LASTL as LineNr)
        || state.newlnum == newlnum::LAST as LineNr
    {
        // SAFETY: the mark list of a live buffer.
        let pos = unsafe { &raw mut (*buflist_findfmark(buffer)).mark };
        state.newlnum = unsafe { (*pos).lnum };
        state.solcol = unsafe { (*pos).col };
    }

    // Make the (new) buffer the one used by the current window.  If the old
    // buffer becomes unused, free it if EcmdFlags::HIDE is false.  If the current
    // buffer was empty and has no file name, curbuf is returned by
    // buflist_new(), and there is nothing to do here.
    if buffer.raw() != Buf::current_raw() {
        match leave_for_buffer(buffer, args, *oldwin, old_curbuf, state) {
            Switch::Abandon => return Switch::Abandon,
            Switch::Ready => {}
        }
    }
    Switch::Ready
}

/// Fire BufLeave for the buffer being left, close it if it is no longer
/// wanted, and make `buffer` the current window's.
fn leave_for_buffer(
    mut buffer: Buf,
    args: &mut EcmdArgs<'_>,
    oldwin: Option<WinId>,
    old_curbuf: &mut BufRef,
    state: &mut Ecmd,
) -> Switch {
    let flags = args.flags;
    // Should only be possible to get here if the cmdwin is closed, or if it's
    // opening and its buffer hasn't been set yet (the new buffer is for it).
    debug_assert!(cmdwin_buf.get().is_none(), "cmdwin_buf == NULL");

    let save_cmdwin_type = cmdwin_type.get();
    let save_cmdwin_win = cmdwin_win.get();
    let save_cmdwin_old_curwin = cmdwin_old_curwin.get();

    // BufLeave applies to the old buffer.
    cmdwin_type.set(0);
    cmdwin_win.set(None);
    cmdwin_old_curwin.set(None);

    // Be careful: the autocommands may delete any buffer and change the
    // current buffer.
    // - If the buffer we are going to edit is deleted, give up.
    // - If the current buffer is deleted, prefer to load the new buffer when
    //   loading a buffer is required.  This avoids loading another buffer
    //   which then must be closed again.
    // - If we ended up in the new buffer already, need to skip a few things,
    //   set auto_buf.
    // The buffer's name, kept for the message an autocommand that deletes it
    // earns.  Upstream `xstrdup`s it and frees it at five exits and inside
    // `delbuf_msg`; owning it is one `Drop`.
    // SAFETY: the buffer's own file name is NUL-terminated.
    let new_name =
        (!buffer.b_fname.is_null()).then(|| unsafe { CStr::from_ptr(buffer.b_fname) }.into());
    let new_name: Option<CString> = new_name;
    let save_au_new_curbuf = au_new_curbuf.get();
    au_new_curbuf.set(BufRef::of(buffer).record());
    buf_autocmd(AutoEvent::BufLeave, Buf::current());

    cmdwin_type.set(save_cmdwin_type);
    cmdwin_win.set(save_cmdwin_win);
    cmdwin_old_curwin.set(save_cmdwin_old_curwin);

    if !au_new_curbuf_valid() {
        // New buffer has been deleted.
        delbuf_msg(new_name.as_deref());
        au_new_curbuf.set(save_au_new_curbuf);
        return Switch::Abandon;
    }
    if aborting() {
        // autocmds may abort script processing
        au_new_curbuf.set(save_au_new_curbuf);
        return Switch::Abandon;
    }

    if buffer.raw() == Buf::current_raw() {
        // already in new buffer
        state.auto_buf = true;
        au_new_curbuf.set(save_au_new_curbuf);
        return Switch::Ready;
    }

    let the_curwin = Win::current().id();
    let was_curbuf = Buf::current().id();

    // Set w_locked to avoid that autocommands close the window.  Set
    // b_locked for the same reason.
    // SAFETY: the window is the editor's own and live.
    Win::current().w_locked = true;
    buffer.b_locked += 1;

    if old_curbuf.is(Buf::current_or_none()) {
        buf_copy_options(buffer, BCO_ENTER as c_int);
    }

    // A terminal buffer that is still running is hidden, never unloaded.
    // SAFETY: the current buffer is live, and its terminal is its own.
    let unload = !(flags.has(EcmdFlags::HIDE)
        || !Buf::current().terminal.is_null()
            && unsafe { terminal_running(Buf::current().terminal) });

    // Close the link to the current buffer.  This will set
    // oldwin->w_buffer to NULL.
    u_sync(false);
    let mode = if unload { DOBUF_UNLOAD as c_int } else { 0 };
    let win = oldwin.and_then(WinId::get);
    let did_decrement = close_buffer(win, Buf::current(), mode, false, false);

    // Autocommands may have closed the window; a stale id answers `None`.
    if let Some(mut win) = valid_win(the_curwin) {
        win.w_locked = false;
    }
    buffer.b_locked -= 1;

    // autocmds may abort script processing
    // SAFETY: `curwin` is live.
    if aborting() && !Win::current().w_buffer.is_null() {
        au_new_curbuf.set(save_au_new_curbuf);
        return Switch::Abandon;
    }
    // Be careful again, like above.
    if !au_new_curbuf_valid() {
        // New buffer has been deleted.
        delbuf_msg(new_name.as_deref());
        au_new_curbuf.set(save_au_new_curbuf);
        return Switch::Abandon;
    }

    // `close_buffer` may have left the editor with no buffer at all --
    // upstream compares a non-NULL `buf` against a NULL `curbuf` here and
    // finds them unequal, which is what the `Option` says.
    if Some(buffer) == Buf::current_or_none() {
        // already in new buffer -- close_buffer() has decremented the
        // window count, increment it again here and restore w_buffer.
        if did_decrement
            && buf_valid(was_curbuf)
            && let Some(mut buf) = was_curbuf.get()
        {
            buf.b_nwindows += 1;
        }
        if let Some(mut old) = oldwin
            .filter(|&w| win_valid_any_tab(w))
            .and_then(WinId::get)
            && old.w_buffer.is_null()
            && let Some(buf) = was_curbuf.get()
        {
            old.w_buffer = buf.raw();
        }
        state.auto_buf = true;
    } else {
        // <VN> We could instead free the synblock and re-attach to the
        // buffer, perhaps.
        if Win::current().w_buffer.is_null()
            || Win::current().w_s == unsafe { &raw mut (*Win::current().w_buffer).b_s }
        {
            Win::current().w_s = &raw mut buffer.b_s;
        }

        Win::current().w_buffer = buffer.raw();
        buffer.make_current();
        Buf::current().b_nwindows += 1;

        // Set 'fileformat', 'binary' and 'fenc' when forced.
        if !state.oldbuf
            && let Some(asked) = args.excmd.as_deref_mut()
        {
            // SAFETY: the command's `++ff=`/`++enc=` offsets into its own line.
            set_file_options(true, Some(asked));
            set_forced_fenc(asked);
        }
    }

    // May get the window options from the last time this buffer was in
    // this window (or another window).  If not used before, reset the
    // local window options to the global values.  Also restores old
    // folding stuff.
    get_winopts(Buf::current());
    state.did_get_winopts = true;

    au_new_curbuf.set(save_au_new_curbuf);
    Switch::Ready
}

/// Is the buffer `au_new_curbuf` names still alive?
fn au_new_curbuf_valid() -> bool {
    BufRef::of_record(au_new_curbuf.get()).valid()
}
/// An autocommand deleted the buffer that was about to be edited.
///
/// Upstream frees `name` here, which is why its callers hand over a
/// `xstrdup` of the buffer's file name and forget it; the name is owned by
/// the caller now, so this only reads it.
pub(super) fn delbuf_msg(name: Option<&CStr>) {
    let arg0 = msg_cstr(name.unwrap_or(c""));
    semsg!("E143: Autocommands unexpectedly deleted new buffer {arg0}");
    au_new_curbuf.with_mut(|r| {
        r.br_buf = ptr::null_mut();
        r.br_buf_free_count = 0;
    });
}
