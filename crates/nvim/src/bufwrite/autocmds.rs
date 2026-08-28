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
use crate::semsg_c;
use core::ffi::{c_char, c_int};

use crate::types::{CmdModFlags, CpoFlag, FAIL, OK, event_T};

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
    pub start: pos_T,
    pub end: pos_T,
}

/// What the pre-write autocommands left for `buf_write` to do.
pub(crate) enum PreWrite {
    /// Nothing was written; go ahead with the write.
    Proceed,
    /// The write is over before it began — either a `*WriteCmd` autocommand
    /// did it, or something went wrong. This is `buf_write`'s return value,
    /// and `no_wait_return` has already been decremented.
    Finished(c_int),
}

/// Fire one `*WritePre` event.
///
/// Returns true for the `E676` case: an `acwrite`-style buffer being written
/// over its own name has nothing but a `*WriteCmd` autocommand to write it,
/// and none matched.
unsafe fn apply_pre(
    event: event_T,
    sfname: *mut c_char,
    eap: *mut exarg_T,
    overwriting: bool,
) -> bool {
    if overwriting && buf_is_nofilename(current_buf()) {
        return true;
    }
    // SAFETY: the caller's promise -- a live Ex-command argument and a
    // NUL-terminated short file name.
    unsafe { apply_autocmds_exarg(event, sfname, sfname, false, curbuf.get(), eap) };
    false
}

/// Apply the pre-write autocommands, and work out whether the write should
/// still happen.
///
/// Careful: the autocommands may call `buf_write` recursively.
pub(crate) unsafe fn buf_write_do_autocmds(
    buf: *mut buf_T,
    names: &mut WriteNames,
    start: linenr_T,
    end: &mut linenr_T,
    eap: *mut exarg_T,
    mode: WriteMode,
    orig: OpMarks,
) -> PreWrite {
    let old_line_count = unsafe { (*buf).b_ml.ml_line_count };
    let msg_save = msg_scroll.get();
    let empty_memline = unsafe { (*buf).b_ml.ml_mfp }.is_null();
    let sfname = names.sfname;

    // Which of the three names are the buffer's own, and so have to be
    // re-read if the autocommands rename it.
    let buf_ffname = names.ffname == unsafe { (*buf).b_ffname };
    let buf_sfname = sfname == unsafe { (*buf).b_sfname };
    let buf_fname_f = names.fname == unsafe { (*buf).b_ffname };
    let buf_fname_s = names.fname == unsafe { (*buf).b_sfname };

    // Set curwin/curbuf to buf and save a few things.
    let mut aco = aco_save_T::default();
    unsafe { aucmd_prepbuf(&raw mut aco, buf) };
    let bufref = BufRef::of_opt(unsafe { Buf::from_raw(buf) });

    // Did a "Cmd" autocommand write the file itself?
    let mut did_cmd = false;
    let mut nofile_err = false;
    if mode.req.append {
        did_cmd = unsafe {
            apply_autocmds_exarg(
                EVENT_FILEAPPENDCMD,
                sfname,
                sfname,
                false,
                curbuf.get(),
                eap,
            )
        };
        if !did_cmd {
            nofile_err = unsafe { apply_pre(EVENT_FILEAPPENDPRE, sfname, eap, mode.overwriting) };
        }
    } else if mode.req.filtering {
        // No <afile>: the filter's output file is not what the event is
        // about.
        unsafe {
            apply_autocmds_exarg(
                EVENT_FILTERWRITEPRE,
                core::ptr::null_mut(),
                sfname,
                false,
                curbuf.get(),
                eap,
            )
        };
    } else if mode.req.reset_changed && mode.whole {
        let was_changed = curbuf_is_changed();
        did_cmd = unsafe {
            apply_autocmds_exarg(EVENT_BUFWRITECMD, sfname, sfname, false, curbuf.get(), eap)
        };
        if did_cmd {
            if was_changed && !curbuf_is_changed() {
                // BufWriteCmd wrote everything correctly and reset
                // 'modified': correct the undo information so that an
                // undo now sets it again.
                u_unchanged(unsafe { Buf::current() });
                u_update_save_nr(unsafe { Buf::current() });
            }
        } else {
            nofile_err = unsafe { apply_pre(EVENT_BUFWRITEPRE, sfname, eap, mode.overwriting) };
        }
    } else {
        did_cmd = unsafe {
            apply_autocmds_exarg(EVENT_FILEWRITECMD, sfname, sfname, false, curbuf.get(), eap)
        };
        if !did_cmd {
            nofile_err = unsafe { apply_pre(EVENT_FILEWRITEPRE, sfname, eap, mode.overwriting) };
        }
    }

    // Restore curwin/curbuf and a few other things.
    unsafe { aucmd_restbuf(&raw mut aco) };

    // The buffer is gone if the autocommands deleted or unloaded it.
    let buf = if bufref.valid() {
        buf
    } else {
        core::ptr::null_mut()
    };

    // In three situations the file is not written here: the buffer is
    // gone, script processing was aborted, or one of the "Cmd"
    // autocommands already did it.
    if buf.is_null()
        || (unsafe { (*buf).b_ml.ml_mfp }.is_null() && !empty_memline)
        || did_cmd
        || nofile_err
        || aborting()
    {
        if !buf.is_null() && cmdmod_has(CmdModFlags::LOCKMARKS) {
            unsafe { (*buf).b_op_start = orig.start };
            unsafe { (*buf).b_op_end = orig.end };
        }
        no_wait_return.set(no_wait_return.get() - 1);
        msg_scroll.set(msg_save);
        if nofile_err {
            unsafe {
                semsg_c!(
                    gettext(c"E676: No matching autocommands for buftype=%s buffer".as_ptr()),
                    (*curbuf.get()).b_p_bt,
                )
            };
        }
        if nofile_err || aborting() {
            // An aborting error, interrupt or exception in the
            // autocommands.
            return PreWrite::Finished(FAIL);
        }
        if did_cmd {
            if buf.is_null() {
                // The buffer was deleted. Assume it was written; there is
                // no retrying anyway.
                return PreWrite::Finished(OK);
            }
            if mode.overwriting {
                // Assume the buffer was written; update the timestamp.
                unsafe { ml_timestamp(buf) };
                if mode.req.append {
                    unsafe { (*buf).b_flags.clear(BufFlags::NEW) };
                } else {
                    unsafe { (*buf).b_flags.clear(BufFlags::WRITE_MASK) };
                }
            }
            if mode.req.reset_changed
                && unsafe { (*buf).b_changed } != 0
                && !mode.req.append
                && (mode.overwriting || cpo_has(CpoFlag::PLUS))
            {
                // Buffer still changed: the autocommands didn't work
                // properly.
                return PreWrite::Finished(FAIL);
            }
            return PreWrite::Finished(OK);
        }
        if !aborting() {
            unsafe {
                emsg(gettext(
                    c"E203: Autocommands deleted or unloaded buffer to be written".as_ptr(),
                ))
            };
        }
        return PreWrite::Finished(FAIL);
    }

    // The autocommands may have changed the number of lines in the file.
    // When writing the whole file, adjust the end. When writing part of
    // it, assume they only changed the number of lines to be written
    // (tricky!).
    if unsafe { (*buf).b_ml.ml_line_count } != old_line_count {
        if mode.whole {
            *end = unsafe { (*buf).b_ml.ml_line_count };
        } else if unsafe { (*buf).b_ml.ml_line_count } > old_line_count {
            *end += unsafe { (*buf).b_ml.ml_line_count } - old_line_count;
        } else {
            *end -= old_line_count - unsafe { (*buf).b_ml.ml_line_count };
            if *end < start {
                no_wait_return.set(no_wait_return.get() - 1);
                msg_scroll.set(msg_save);
                unsafe {
                    emsg(gettext(
                        c"E204: Autocommand changed number of lines in unexpected way".as_ptr(),
                    ))
                };
                return PreWrite::Finished(FAIL);
            }
        }
    }

    // The autocommands may have renamed the buffer; the names that came
    // from it have to be re-read.
    if buf_ffname {
        names.ffname = unsafe { (*buf).b_ffname };
    }
    if buf_sfname {
        names.sfname = unsafe { (*buf).b_sfname };
    }
    if buf_fname_f {
        names.fname = unsafe { (*buf).b_ffname };
    }
    if buf_fname_s {
        names.fname = unsafe { (*buf).b_sfname };
    }
    PreWrite::Proceed
}

/// Apply the post-write autocommands.
///
/// Careful: the autocommands may call `buf_write` recursively.
pub(crate) unsafe fn buf_write_do_post_autocmds(
    buf: *mut buf_T,
    fname: *mut c_char,
    eap: *mut exarg_T,
    mode: WriteMode,
) {
    // In case it was set by the previous read.
    unsafe { (*curbuf.get()).b_no_eol_lnum = 0 };

    let mut aco = aco_save_T::default();
    unsafe { aucmd_prepbuf(&raw mut aco, buf) };

    let event = if mode.req.append {
        EVENT_FILEAPPENDPOST
    } else if mode.req.filtering {
        EVENT_FILTERWRITEPOST
    } else if mode.req.reset_changed && mode.whole {
        EVENT_BUFWRITEPOST
    } else {
        EVENT_FILEWRITEPOST
    };
    // As for FilterWritePre, the filter's file is not the <afile>.
    let afile = if mode.req.filtering {
        core::ptr::null_mut()
    } else {
        fname
    };
    unsafe { apply_autocmds_exarg(event, afile, fname, false, curbuf.get(), eap) };

    // Restore curwin/curbuf and a few other things.
    unsafe { aucmd_restbuf(&raw mut aco) };
}
