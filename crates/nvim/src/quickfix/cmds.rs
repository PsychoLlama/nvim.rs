//! `:cfile`, `:cbuffer`, `:cexpr` and their variants.
//!
//! Each takes lines from somewhere other than a command — a file
//! ([`ex_cfile`]), a buffer range ([`ex_cbuffer`]) or the value of a
//! Vimscript expression ([`ex_cexpr`]) — parses them with `'errorformat'`
//! and either replaces or adds to a list. The `*_get_auname` helpers name
//! the `QuickFixCmdPre`/`QuickFixCmdPost` autocommand each one fires.
//!
//! All three run the same errand afterwards: fire `QuickFixCmdPost` and,
//! for the plain form only — not the `get` and `add` variants — jump to the
//! first error. They keep their own copies of that tail because each fires
//! the autocommand with a different name and `:cbuffer` also has to notice
//! a buffer switch.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::types::CmdIdx;
use crate::types::{Failed, IOSIZE, NUL, OptionSetFlags, VAR_LIST};
use core::ffi::{CStr, c_char, c_int, c_uint};
use core::ptr;

/// The autocommand name of a `:cfile`-family command.
fn cfile_get_auname(cmdidx: CmdIdx) -> Option<&'static CStr> {
    Some(match cmdidx {
        CmdIdx::cfile => c"cfile",
        CmdIdx::cgetfile => c"cgetfile",
        CmdIdx::caddfile => c"caddfile",
        CmdIdx::lfile => c"lfile",
        CmdIdx::lgetfile => c"lgetfile",
        CmdIdx::laddfile => c"laddfile",
        _ => return None,
    })
}

/// `:cfile`, `:cgetfile`, `:caddfile` and their `:l…` twins: read
/// `'errorfile'`, or the file named as the argument.
///
/// # Safety
///
/// `args` must be a live command.
pub unsafe fn ex_cfile(args: *mut ExArg) {
    // SAFETY: the caller's promise -- a live `ExArg`.
    let args = unsafe { Ea::new(args) };
    let mut qi = qf_global();

    let au_name = cfile_get_auname(args.cmdidx);
    if let Some(name) = au_name {
        let claimed = fire_qf_autocmd(AutoEvent::QuickFixCmdPre, name, false);
        if claimed && aborting() {
            return;
        }
    }

    if c_int::from(unsafe { *args.arg }) != NUL {
        set_option_direct(
            kOptErrorfile,
            OptVal::String(unsafe { cstr_as_string(args.arg) }),
            OptionSetFlags::NONE,
            0 as ScriptId,
        );
    }

    let local_enc = Buf::current().b_p_menc;
    let enc = if c_int::from(unsafe { *local_enc }) != NUL {
        local_enc
    } else {
        p_menc.get()
    };

    let wp = is_loclist_cmd(args.cmdidx).then(Win::current);

    incr_quickfix_busy();

    let newlist = !matches!(args.cmdidx, CmdIdx::caddfile | CmdIdx::laddfile);
    let efile = p_ef.get();
    let errorformat2 = p_efm.get();
    let newlist2 = c_int::from(newlist);
    let title = unsafe { qf_cmdtitle(*args.cmdlinep) };
    let qf_title2 = title.as_ptr();
    let res = unsafe { qf_init(wp, efile, errorformat2, newlist2, qf_title2, enc) };

    if let Some(wp) = wp {
        let Some(loclist) = qf_win_loclist(wp) else {
            qf_busy_end();
            return;
        };
        qi = loclist;
    }
    if res >= 0 {
        qfl_changed(qf_current_list(qi));
    }
    // Remember the current list, so that an autocommand replacing it is
    // noticed before the jump.
    let save_qfid = qf_current_list(qi).qf_id;
    if let Some(name) = au_name {
        fire_qf_autocmd(AutoEvent::QuickFixCmdPost, name, false);
    }

    let jumps = matches!(args.cmdidx, CmdIdx::cfile | CmdIdx::lfile);
    if res > 0 && jumps && qf_list_still_valid(wp, save_qfid) {
        unsafe { qf_jump_first(qi.raw(), save_qfid, args.forceit) };
    }
    qf_busy_end();
}

/// The autocommand name of a `:cbuffer`-family command.
fn cbuffer_get_auname(cmdidx: CmdIdx) -> Option<&'static CStr> {
    Some(match cmdidx {
        CmdIdx::cbuffer => c"cbuffer",
        CmdIdx::cgetbuffer => c"cgetbuffer",
        CmdIdx::caddbuffer => c"caddbuffer",
        CmdIdx::lbuffer => c"lbuffer",
        CmdIdx::lgetbuffer => c"lgetbuffer",
        CmdIdx::laddbuffer => c"laddbuffer",
        _ => return None,
    })
}

/// The buffer and line range a `:cbuffer` command names: the current
/// buffer, or the one whose number is the whole argument, over the
/// command's range or the whole buffer. Answers `None` after reporting the
/// error itself.
///
/// # Safety
///
/// `args` must be a live command.
unsafe fn cbuffer_process_args(args: *mut ExArg) -> Option<Buf> {
    // SAFETY: the caller's promise -- a live `ExArg`.
    let mut args = unsafe { Ea::new(args) };
    // SAFETY: forwarded from the caller.
    let buf = if c_int::from(unsafe { *args.arg }) == NUL {
        Buf::current_raw()
    } else if c_int::from(unsafe { *skipwhite(skipdigits(args.arg)) }) == NUL {
        find_buf(unsafe { atoi(args.arg) }).map_or(ptr::null_mut(), |b| b.raw())
    } else {
        ptr::null_mut()
    };

    // SAFETY: `curbuf`/`find_buf` answer a live buffer or null.
    let Some(buf) = (unsafe { Buf::from_raw(buf) }) else {
        qf_emsg(e_invarg.as_ptr());
        return None;
    };
    if buf.b_ml.ml_mfp.is_null() {
        qf_emsg(e_buffer_is_not_loaded.as_ptr());
        return None;
    }

    if args.addr_count == 0 {
        args.line1 = 1;
        args.line2 = buf.b_ml.ml_line_count;
    }
    if args.line1 < 1
        || args.line1 > buf.b_ml.ml_line_count
        || args.line2 < 1
        || args.line2 > buf.b_ml.ml_line_count
    {
        qf_emsg(e_invrange.as_ptr());
        return None;
    }
    Some(buf)
}

/// `:cbuffer`, `:cgetbuffer`, `:caddbuffer` and their `:l…` twins: parse a
/// range of lines of a buffer.
///
/// # Safety
///
/// `args` must be a live command.
pub unsafe fn ex_cbuffer(args: *mut ExArg) {
    // SAFETY: the caller's promise -- a live `ExArg`.
    let args = unsafe { Ea::new(args) };
    let mut title = [0 as c_char; IOSIZE as usize];
    let au_name = cbuffer_get_auname(args.cmdidx);
    if let Some(name) = au_name {
        let claimed = fire_qf_autocmd(AutoEvent::QuickFixCmdPre, name, true);
        if claimed && aborting() {
            return;
        }
    }

    let (qi, wp) = qf_cmd_stack_or_alloc(args);
    let parsed = unsafe { cbuffer_process_args(args.raw()) };
    let Some(buf) = parsed else {
        return;
    };

    // The title names the buffer as well as the command. `qf_init_ext`
    // copies it, so this frame can own it.
    let mut qf_title = unsafe { qf_cmdtitle(*args.cmdlinep) };
    if !buf.b_sfname.is_null() {
        let efile = IOSIZE as size_t;
        let fmt = c"%s (%s)".as_ptr();
        let sfname = buf.b_sfname;
        unsafe { vim_snprintf(title.as_mut_ptr(), efile, fmt, qf_title.as_ptr(), sfname) };
        qf_title[..IOSIZE as usize].copy_from_slice(&title);
    }

    incr_quickfix_busy();

    let newlist = !matches!(args.cmdidx, CmdIdx::caddbuffer | CmdIdx::laddbuffer);
    let qi2 = qi.raw();
    let curlist = qi.qf_curlist;
    let errorformat2 = ptr::null();
    let qf_title2 = None;
    let errorformat3 = p_efm.get();
    let line12 = args.line1;
    let line22 = args.line2;
    let enc2 = ptr::null_mut();
    let mut res = unsafe {
        qf_init_ext(
            qi2,
            curlist,
            errorformat2,
            Some(buf),
            qf_title2,
            errorformat3,
            newlist,
            line12,
            line22,
            qf_title.as_ptr(),
            enc2,
        )
    };

    if qf_is_empty(qi) {
        qf_busy_end();
        return;
    }
    if res >= 0 {
        qfl_changed(qf_current_list(qi));
    }
    let save_qfid = qf_current_list(qi).qf_id;
    if let Some(name) = au_name {
        let curbuf_old: *const Buffer = Buf::current_raw();
        fire_qf_autocmd(AutoEvent::QuickFixCmdPost, name, true);
        // The autocommand switched buffers: do not jump away from
        // wherever it left the user.
        if !ptr::eq(Buf::current_raw(), curbuf_old) {
            res = 0;
        }
    }

    let jumps = matches!(args.cmdidx, CmdIdx::cbuffer | CmdIdx::lbuffer);
    if res > 0 && jumps && qf_list_still_valid(wp, save_qfid) {
        unsafe { qf_jump_first(qi.raw(), save_qfid, args.forceit) };
    }
    qf_busy_end();
}

/// The autocommand name of a `:cexpr`-family command.
fn cexpr_get_auname(cmdidx: CmdIdx) -> Option<&'static CStr> {
    Some(match cmdidx {
        CmdIdx::cexpr => c"cexpr",
        CmdIdx::cgetexpr => c"cgetexpr",
        CmdIdx::caddexpr => c"caddexpr",
        CmdIdx::lexpr => c"lexpr",
        CmdIdx::lgetexpr => c"lgetexpr",
        CmdIdx::laddexpr => c"laddexpr",
        _ => return None,
    })
}

/// Fire `QuickFixCmdPre` for a `:cexpr`-family command. Answers false when
/// an autocommand aborted, in which case the expression is not evaluated at
/// all — which is why this is separate from [`cexpr_core`], whose callers
/// hand it a value that has already been computed.
fn trigger_cexpr_autocmd(cmdidx: CmdIdx) -> bool {
    if let Some(name) = cexpr_get_auname(cmdidx) {
        let claimed = fire_qf_autocmd(AutoEvent::QuickFixCmdPre, name, true);
        if claimed && aborting() {
            return false;
        }
    }
    true
}

/// Build a list out of an already evaluated string or list of strings.
///
/// # Safety
///
/// `args` must be a live command and `tv` a live value.
unsafe fn cexpr_core(args: *const ExArg, tv: &mut TypVal) -> Result<(), Failed> {
    // SAFETY: the caller's promise -- a live `ExArg`.
    let args = unsafe { Ea::new(args.cast_mut()) };
    // SAFETY: forwarded from the caller.
    // The stack is asked for first, and so allocated for the current
    // window if it had none, even when the value turns out to be
    // unusable.
    let (qi, wp) = qf_cmd_stack_or_alloc(args);

    // A non-string reads as a NULL string, so the tag test is the accessor's.
    let usable = !(*tv).string_or_null().is_null() || (*tv).v_type() == VAR_LIST;
    if !usable {
        qf_emsg(c"E777: String or List expected".as_ptr());
        return Err(Failed);
    }

    let au_name = cexpr_get_auname(args.cmdidx);

    incr_quickfix_busy();

    let newlist = !matches!(args.cmdidx, CmdIdx::caddexpr | CmdIdx::laddexpr);
    let qi2 = qi.raw();
    let curlist = qi.qf_curlist;
    let errorformat2 = ptr::null();
    let buf2 = None;
    let errorformat3 = p_efm.get();
    let title = unsafe { qf_cmdtitle(*args.cmdlinep) };
    let enc2 = ptr::null_mut();
    let res = unsafe {
        qf_init_ext(
            qi2,
            curlist,
            errorformat2,
            buf2,
            Some(tv),
            errorformat3,
            newlist,
            0,
            0,
            title.as_ptr(),
            enc2,
        )
    };

    if qf_is_empty(qi) {
        qf_busy_end();
        return Err(Failed);
    }
    if res >= 0 {
        qfl_changed(qf_current_list(qi));
    }
    let save_qfid: c_uint = qf_current_list(qi).qf_id;
    if let Some(name) = au_name {
        fire_qf_autocmd(AutoEvent::QuickFixCmdPost, name, true);
    }

    let jumps = matches!(args.cmdidx, CmdIdx::cexpr | CmdIdx::lexpr);
    if res > 0 && jumps && qf_list_still_valid(wp, save_qfid) {
        unsafe { qf_jump_first(qi.raw(), save_qfid, args.forceit) };
    }
    qf_busy_end();
    Ok(())
}

/// `:cexpr`, `:cgetexpr`, `:caddexpr` and their `:l…` twins.
///
/// # Safety
///
/// `args` must be a live command.
pub unsafe fn ex_cexpr(args: *mut ExArg) {
    // SAFETY: the caller's promise -- a live `ExArg`.
    let args = unsafe { Ea::new(args) };
    if !trigger_cexpr_autocmd(args.cmdidx) {
        return;
    }
    // Evaluate the expression. When the result is a string or a list of
    // strings, parse each line and add it to the quickfix list.
    let tv = unsafe { eval_expr(args.arg, args.raw()) };
    if tv.is_null() {
        return;
    }
    let _ = unsafe { cexpr_core(args.raw().cast_const(), &mut *tv) };
    unsafe { tv_free(tv.as_mut()) };
}
