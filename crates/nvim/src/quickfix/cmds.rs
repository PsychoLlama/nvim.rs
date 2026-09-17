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
use crate::option::local_or_global;
use crate::option::vars::P_EF;
use crate::option::vars::P_EFM;
use crate::option::vars::P_MENC;
use crate::types::CmdIdx;
use crate::types::OptStr;
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
pub fn ex_cfile(excmd: &mut ExArg) {
    // SAFETY: the caller's promise -- a live `ExArg`.
    let mut qi = qf_global();

    let au_name = cfile_get_auname(excmd.cmdidx);
    if let Some(name) = au_name {
        let claimed = fire_qf_autocmd(AutoEvent::QuickFixCmdPre, name, false);
        if claimed && aborting() {
            return;
        }
    }

    if c_int::from(unsafe { *excmd.arg }) != NUL {
        set_option_direct(
            kOptErrorfile,
            // SAFETY: the command line's own NUL-terminated argument,
            // which the option layer copies.
            OptVal::String(unsafe { OptStr::borrowing(excmd.arg) }),
            OptionSetFlags::NONE,
            0 as ScriptId,
        );
    }

    let enc = local_or_global(&Buf::current().b_p_menc, P_MENC);

    let wp = is_loclist_cmd(excmd.cmdidx).then(Win::current);

    incr_quickfix_busy();

    let newlist = !matches!(excmd.cmdidx, CmdIdx::caddfile | CmdIdx::laddfile);
    // Copies: `qf_init` reads a file and fires autocommands.
    let (efile, errorformat2) = (P_EF.get(), P_EFM.get());
    let newlist2 = c_int::from(newlist);
    let title = unsafe { qf_cmdtitle(*excmd.cmdlinep) };
    let qf_title2 = title.as_ptr();
    let res = unsafe {
        qf_init(
            wp,
            efile.as_ptr().cast_mut(),
            errorformat2.as_ptr().cast_mut(),
            true,
            newlist2,
            qf_title2,
            enc.as_ptr().cast_mut(),
        )
    };

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

    let jumps = matches!(excmd.cmdidx, CmdIdx::cfile | CmdIdx::lfile);
    if res > 0 && jumps && qf_list_still_valid(wp, save_qfid) {
        unsafe { qf_jump_first(qi.raw(), save_qfid, c_int::from(excmd.forceit)) };
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
fn cbuffer_process_args(excmd: &mut ExArg) -> Option<Buf> {
    // SAFETY: the caller's promise -- a live `ExArg`.
    // SAFETY: forwarded from the caller.
    let buf = if c_int::from(unsafe { *excmd.arg }) == NUL {
        Buf::current_raw()
    } else if c_int::from(unsafe { *skipwhite(skipdigits(excmd.arg)) }) == NUL {
        find_buf(unsafe { atoi(excmd.arg) }).map_or(ptr::null_mut(), |b| b.raw())
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

    if excmd.addr_count == 0 {
        excmd.line1 = 1;
        excmd.line2 = buf.b_ml.ml_line_count;
    }
    if excmd.line1 < 1
        || excmd.line1 > buf.b_ml.ml_line_count
        || excmd.line2 < 1
        || excmd.line2 > buf.b_ml.ml_line_count
    {
        qf_emsg(e_invrange.as_ptr());
        return None;
    }
    Some(buf)
}

/// `:cbuffer`, `:cgetbuffer`, `:caddbuffer` and their `:l…` twins: parse a
/// range of lines of a buffer.
pub fn ex_cbuffer(excmd: &mut ExArg) {
    // SAFETY: the caller's promise -- a live `ExArg`.
    let mut title = [0 as c_char; IOSIZE as usize];
    let au_name = cbuffer_get_auname(excmd.cmdidx);
    if let Some(name) = au_name {
        let claimed = fire_qf_autocmd(AutoEvent::QuickFixCmdPre, name, true);
        if claimed && aborting() {
            return;
        }
    }

    let (qi, wp) = qf_cmd_stack_or_alloc(excmd);
    let parsed = cbuffer_process_args(excmd);
    let Some(buf) = parsed else {
        return;
    };

    // The title names the buffer as well as the command. `qf_init_ext`
    // copies it, so this frame can own it.
    let mut qf_title = unsafe { qf_cmdtitle(*excmd.cmdlinep) };
    if !buf.b_sfname.is_null() {
        let efile = IOSIZE as size_t;
        let fmt = c"%s (%s)".as_ptr();
        let sfname = buf.b_sfname;
        unsafe { vim_snprintf(title.as_mut_ptr(), efile, fmt, qf_title.as_ptr(), sfname) };
        qf_title[..IOSIZE as usize].copy_from_slice(&title);
    }

    incr_quickfix_busy();

    let newlist = !matches!(excmd.cmdidx, CmdIdx::caddbuffer | CmdIdx::laddbuffer);
    let qi2 = qi.raw();
    let curlist = qi.qf_curlist;
    let errorformat2 = ptr::null();
    let qf_title2 = None;
    // A copy: `qf_init_ext` reads the buffer and fires autocommands.
    let errorformat3 = P_EFM.get();
    let line12 = excmd.line1;
    let line22 = excmd.line2;
    let enc2 = ptr::null_mut();
    let mut res = unsafe {
        qf_init_ext(
            qi2,
            curlist,
            errorformat2,
            Some(buf),
            qf_title2,
            errorformat3.as_ptr().cast_mut(),
            true,
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

    let jumps = matches!(excmd.cmdidx, CmdIdx::cbuffer | CmdIdx::lbuffer);
    if res > 0 && jumps && qf_list_still_valid(wp, save_qfid) {
        unsafe { qf_jump_first(qi.raw(), save_qfid, c_int::from(excmd.forceit)) };
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
fn cexpr_core(excmd: &mut ExArg, tv: &mut TypVal) -> Result<(), Failed> {
    // The stack is asked for first, and so allocated for the current
    // window if it had none, even when the value turns out to be
    // unusable.
    let (qi, wp) = qf_cmd_stack_or_alloc(excmd);

    // A non-string reads as a NULL string, so the tag test is the accessor's.
    let usable = !(*tv).string_or_null().is_null() || (*tv).v_type() == VAR_LIST;
    if !usable {
        qf_emsg(c"E777: String or List expected".as_ptr());
        return Err(Failed);
    }

    let au_name = cexpr_get_auname(excmd.cmdidx);

    incr_quickfix_busy();

    let newlist = !matches!(excmd.cmdidx, CmdIdx::caddexpr | CmdIdx::laddexpr);
    let qi2 = qi.raw();
    let curlist = qi.qf_curlist;
    let errorformat2 = ptr::null();
    let buf2 = None;
    // A copy: `qf_init_ext` evaluates an expression and fires autocommands.
    let errorformat3 = P_EFM.get();
    let title = unsafe { qf_cmdtitle(*excmd.cmdlinep) };
    let enc2 = ptr::null_mut();
    let res = unsafe {
        qf_init_ext(
            qi2,
            curlist,
            errorformat2,
            buf2,
            Some(tv),
            errorformat3.as_ptr().cast_mut(),
            true,
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

    let jumps = matches!(excmd.cmdidx, CmdIdx::cexpr | CmdIdx::lexpr);
    if res > 0 && jumps && qf_list_still_valid(wp, save_qfid) {
        unsafe { qf_jump_first(qi.raw(), save_qfid, c_int::from(excmd.forceit)) };
    }
    qf_busy_end();
    Ok(())
}

/// `:cexpr`, `:cgetexpr`, `:caddexpr` and their `:l…` twins.
pub fn ex_cexpr(excmd: &mut ExArg) {
    // SAFETY: the caller's promise -- a live `ExArg`.
    if !trigger_cexpr_autocmd(excmd.cmdidx) {
        return;
    }
    // Evaluate the expression. When the result is a string or a list of
    // strings, parse each line and add it to the quickfix list.
    let tv = unsafe { eval_expr(excmd.arg, Some(excmd)) };
    if tv.is_null() {
        return;
    }
    // SAFETY: `tv` is the allocation `eval_expr` just answered.
    let _ = unsafe { cexpr_core(excmd, &mut *tv) };
    unsafe { tv_free(tv.as_mut()) };
}
