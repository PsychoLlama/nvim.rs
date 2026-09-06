//! The autocommands that bracket a write.
//!
//! [`buf_write_do_autocmds`] fires the `*WritePre`/`*WriteCmd` family before
//! anything is written, and has to cope with what they may have done: deleted
//! the buffer, renamed it, changed its line count, or written the file
//! themselves. [`buf_write_do_post_autocmds`] fires the matching `*WritePost`
//! family afterwards.
//!
//! Which event fires depends on how the write was asked for, which is what
//! [`WriteMode`] carries; the names the write is about can be changed
//! underneath it, which is what [`WriteNames`] is for.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::buffer::{BufFlags, buf_is_nofilename, current_buf};
use crate::ex_docmd::cmdmod_has;
use crate::message_fmt::c_str;
use crate::semsg;
use crate::types::AutoEvent;
use core::ffi::c_char;

use crate::types::{CmdModFlags, CpoFlag, Failed};

use super::*;
use crate::buffer::BufRef;
use crate::option::cpo_has;
use crate::winlayer::Buf;

/// How a write was asked for. Chooses which autocommand events fire, and is
/// carried through `buf_write` because most of its decisions turn on these.
#[derive(Copy, Clone)]
pub(crate) struct WriteMode {
    /// What the caller asked for.
    pub req: WriteRequest,
    /// The whole buffer is being written, not just a line range.
    pub whole: bool,
    /// The target is the file the buffer itself was read from.
    pub overwriting: bool,
}

/// The three names a write is about: the one being written (`fname`, the
/// short name on Unix), the short name and the full name.
///
/// Autocommands may rename the buffer while they run. Any of the three that
/// was an alias of the buffer's own `b_ffname`/`b_sfname` has to be re-read
/// from the buffer afterwards, because the old pointer has been freed.
pub(crate) struct WriteNames {
    pub fname: *mut c_char,
    pub sfname: *mut c_char,
    pub ffname: *mut c_char,
}

/// The `'[` and `']` marks as they were before the write set them to the
/// line range, so `:lockmarks` can put them back.
#[derive(Copy, Clone)]
pub(crate) struct OpMarks {
    pub start: Pos,
    pub end: Pos,
}

/// What the pre-write autocommands left for `buf_write` to do.
pub(crate) enum PreWrite {
    /// Nothing was written; go ahead with the write.
    Proceed,
    /// The write is over before it began — either a `*WriteCmd` autocommand
    /// did it, or something went wrong. This is `buf_write`'s return value,
    /// and `no_wait_return` has already been decremented.
    Finished(Result<(), Failed>),
}

/// Fire one `*WritePre` event.
///
/// Returns true for the `E676` case: an `acwrite`-style buffer being written
/// over its own name has nothing but a `*WriteCmd` autocommand to write it,
/// and none matched.
unsafe fn apply_pre(
    event: AutoEvent,
    sfname: *mut c_char,
    args: *mut ExArg,
    overwriting: bool,
) -> bool {
    if overwriting && buf_is_nofilename(current_buf()) {
        return true;
    }
    // SAFETY: the caller's promise -- a live Ex-command argument and a
    // NUL-terminated short file name.
    unsafe { apply_autocmds_exarg(event, sfname, sfname, false, Buf::current_or_none(), args) };
    false
}

/// Apply the pre-write autocommands, and work out whether the write should
/// still happen.
///
/// Careful: the autocommands may call `buf_write` recursively.
pub(crate) unsafe fn buf_write_do_autocmds(
    buffer: Buf,
    names: &mut WriteNames,
    start: LineNr,
    end: &mut LineNr,
    args: *mut ExArg,
    mode: WriteMode,
    orig: OpMarks,
) -> PreWrite {
    let old_line_count = buffer.b_ml.ml_line_count;
    let msg_save = msg_scroll.get();
    let empty_memline = buffer.b_ml.ml_mfp.is_null();
    let sfname = names.sfname;

    // Which of the three names are the buffer's own, and so have to be
    // re-read if the autocommands rename it.
    let buf_ffname = names.ffname == buffer.b_ffname;
    let buf_sfname = sfname == buffer.b_sfname;
    let buf_fname_f = names.fname == buffer.b_ffname;
    let buf_fname_s = names.fname == buffer.b_sfname;

    // Set curwin/curbuf to buf and save a few things.
    let mut aco = AcoSave::default();
    unsafe { aucmd_prepbuf(&raw mut aco, buffer) };
    let bufref = BufRef::of_opt(Some(buffer));

    // Did a "Cmd" autocommand write the file itself?
    let mut did_cmd = false;
    let mut nofile_err = false;
    if mode.req.append {
        let event = AutoEvent::FileAppendCmd;
        did_cmd = unsafe {
            apply_autocmds_exarg(event, sfname, sfname, false, Buf::current_or_none(), args)
        };
        if !did_cmd {
            nofile_err =
                unsafe { apply_pre(AutoEvent::FileAppendPre, sfname, args, mode.overwriting) };
        }
    } else if mode.req.filtering {
        // No <afile>: the filter's output file is not what the event is
        // about.
        let event = AutoEvent::FilterWritePre;
        let no_fname = core::ptr::null_mut();
        unsafe {
            apply_autocmds_exarg(event, no_fname, sfname, false, Buf::current_or_none(), args)
        };
    } else if mode.req.reset_changed && mode.whole {
        let was_changed = curbuf_is_changed();
        let event = AutoEvent::BufWriteCmd;
        did_cmd = unsafe {
            apply_autocmds_exarg(event, sfname, sfname, false, Buf::current_or_none(), args)
        };
        if did_cmd {
            if was_changed && !curbuf_is_changed() {
                // BufWriteCmd wrote everything correctly and reset
                // 'modified': correct the undo information so that an
                // undo now sets it again.
                u_unchanged(Buf::current());
                u_update_save_nr(Buf::current());
            }
        } else {
            nofile_err =
                unsafe { apply_pre(AutoEvent::BufWritePre, sfname, args, mode.overwriting) };
        }
    } else {
        let event = AutoEvent::FileWriteCmd;
        did_cmd = unsafe {
            apply_autocmds_exarg(event, sfname, sfname, false, Buf::current_or_none(), args)
        };
        if !did_cmd {
            nofile_err =
                unsafe { apply_pre(AutoEvent::FileWritePre, sfname, args, mode.overwriting) };
        }
    }

    // Restore curwin/curbuf and a few other things.
    unsafe { aucmd_restbuf(&raw mut aco) };

    // The buffer is gone if the autocommands deleted or unloaded it.
    let live = bufref.valid().then_some(buffer);

    // In three situations the file is not written here: the buffer is
    // gone, script processing was aborted, or one of the "Cmd"
    // autocommands already did it.
    let unloaded = |b: &Buf| b.b_ml.ml_mfp.is_null() && !empty_memline;
    if live.is_none_or(|b| unloaded(&b)) || did_cmd || nofile_err || aborting() {
        if let Some(mut b) = live.filter(|_| cmdmod_has(CmdModFlags::LOCKMARKS)) {
            b.b_op_start = orig.start;
            b.b_op_end = orig.end;
        }
        no_wait_return.set(no_wait_return.get() - 1);
        msg_scroll.set(msg_save);
        if nofile_err {
            let buftype = Buf::current().b_p_bt;
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let buftype = unsafe { c_str(buftype) };
            semsg!("E676: No matching autocommands for buftype={buftype} buffer");
        }
        if nofile_err || aborting() {
            // An aborting error, interrupt or exception in the
            // autocommands.
            return PreWrite::Finished(Err(Failed));
        }
        if did_cmd {
            let Some(mut buffer) = live else {
                // The buffer was deleted. Assume it was written; there is
                // no retrying anyway.
                return PreWrite::Finished(Ok(()));
            };
            if mode.overwriting {
                // Assume the buffer was written; update the timestamp.
                unsafe { ml_timestamp(buffer) };
                if mode.req.append {
                    buffer.b_flags.clear(BufFlags::NEW);
                } else {
                    buffer.b_flags.clear(BufFlags::WRITE_MASK);
                }
            }
            if mode.req.reset_changed
                && buffer.b_changed != 0
                && !mode.req.append
                && (mode.overwriting || cpo_has(CpoFlag::PLUS))
            {
                // Buffer still changed: the autocommands didn't work
                // properly.
                return PreWrite::Finished(Err(Failed));
            }
            return PreWrite::Finished(Ok(()));
        }
        if !aborting() {
            let why = gettext(c"E203: Autocommands deleted or unloaded buffer to be written");
            emsg(why);
        }
        return PreWrite::Finished(Err(Failed));
    }

    // The autocommands may have changed the number of lines in the file.
    // When writing the whole file, adjust the end. When writing part of
    // it, assume they only changed the number of lines to be written
    // (tricky!).
    if unsafe { (*buffer.raw()).b_ml.ml_line_count } != old_line_count {
        if mode.whole {
            *end = unsafe { (*buffer.raw()).b_ml.ml_line_count };
        } else if unsafe { (*buffer.raw()).b_ml.ml_line_count } > old_line_count {
            *end += unsafe { (*buffer.raw()).b_ml.ml_line_count } - old_line_count;
        } else {
            *end -= old_line_count - unsafe { (*buffer.raw()).b_ml.ml_line_count };
            if *end < start {
                no_wait_return.set(no_wait_return.get() - 1);
                msg_scroll.set(msg_save);
                let why = gettext(c"E204: Autocommand changed number of lines in unexpected way");
                emsg(why);
                return PreWrite::Finished(Err(Failed));
            }
        }
    }

    // The autocommands may have renamed the buffer; the names that came
    // from it have to be re-read.
    if buf_ffname {
        names.ffname = unsafe { (*buffer.raw()).b_ffname };
    }
    if buf_sfname {
        names.sfname = unsafe { (*buffer.raw()).b_sfname };
    }
    if buf_fname_f {
        names.fname = unsafe { (*buffer.raw()).b_ffname };
    }
    if buf_fname_s {
        names.fname = unsafe { (*buffer.raw()).b_sfname };
    }
    PreWrite::Proceed
}

/// Apply the post-write autocommands.
///
/// Careful: the autocommands may call `buf_write` recursively.
pub(crate) unsafe fn buf_write_do_post_autocmds(
    buffer: Buf,
    fname: *mut c_char,
    args: *mut ExArg,
    mode: WriteMode,
) {
    // In case it was set by the previous read.
    Buf::current().b_no_eol_lnum = 0;

    let mut aco = AcoSave::default();
    unsafe { aucmd_prepbuf(&raw mut aco, buffer) };

    let event = if mode.req.append {
        AutoEvent::FileAppendPost
    } else if mode.req.filtering {
        AutoEvent::FilterWritePost
    } else if mode.req.reset_changed && mode.whole {
        AutoEvent::BufWritePost
    } else {
        AutoEvent::FileWritePost
    };
    // As for FilterWritePre, the filter's file is not the <afile>.
    let afile = if mode.req.filtering {
        core::ptr::null_mut()
    } else {
        fname
    };
    unsafe { apply_autocmds_exarg(event, afile, fname, false, Buf::current_or_none(), args) };

    // Restore curwin/curbuf and a few other things.
    unsafe { aucmd_restbuf(&raw mut aco) };
}
