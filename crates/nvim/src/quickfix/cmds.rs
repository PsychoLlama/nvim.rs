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

use super::*;
use crate::types::CmdIdx;
use crate::types::{Failed, IOSIZE, NUL, OptionSetFlags, VAR_LIST, VAR_STRING};
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
/// `eap` must be a live command.
pub unsafe fn ex_cfile(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- a live `exarg_T`.
    let eap = unsafe { Ea::new(eap) };
    let mut qi = qf_global();

    let au_name = cfile_get_auname(eap.cmdidx);
    if let Some(name) = au_name {
        let claimed = fire_qf_autocmd(EVENT_QUICKFIXCMDPRE, name, false);
        if claimed && aborting() {
            return;
        }
    }

    if unsafe { *eap.arg } as c_int != NUL {
        set_option_direct(
            kOptErrorfile,
            OptVal::String(unsafe { cstr_as_string(eap.arg) }),
            OptionSetFlags::NONE,
            0 as scid_T,
        );
    }

    let local_enc = cur_buf().b_p_menc;
    let enc = if unsafe { *local_enc } as c_int != NUL {
        local_enc
    } else {
        p_menc.get()
    };

    // SAFETY: a command's `cmdidx` is one of the table's.
    let wp = unsafe { is_loclist_cmd(eap.cmdidx) }.then(cur_win);

    incr_quickfix_busy();

    let newlist = !matches!(eap.cmdidx, CmdIdx::caddfile | CmdIdx::laddfile);
    let efile = p_ef.get();
    let errorformat2 = p_efm.get();
    let newlist2 = newlist as c_int;
    let title = unsafe { qf_cmdtitle(*eap.cmdlinep) };
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
        fire_qf_autocmd(EVENT_QUICKFIXCMDPOST, name, false);
    }

    let jumps = matches!(eap.cmdidx, CmdIdx::cfile | CmdIdx::lfile);
    if res > 0 && jumps && qf_list_still_valid(wp, save_qfid) {
        unsafe { qf_jump_first(qi.raw(), save_qfid, eap.forceit) };
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
/// `eap` must be a live command.
unsafe fn cbuffer_process_args(eap: *mut exarg_T) -> Option<Buf> {
    // SAFETY: the caller's promise -- a live `exarg_T`.
    let mut eap = unsafe { Ea::new(eap) };
    // SAFETY: forwarded from the caller.
    let buf = if unsafe { *eap.arg } as c_int == NUL {
        curbuf.get()
    } else if unsafe { *skipwhite(skipdigits(eap.arg)) } as c_int == NUL {
        find_buf(unsafe { atoi(eap.arg) }).map_or(ptr::null_mut(), |mut b| b.raw())
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

    if eap.addr_count == 0 {
        eap.line1 = 1;
        eap.line2 = buf.b_ml.ml_line_count;
    }
    if eap.line1 < 1
        || eap.line1 > buf.b_ml.ml_line_count
        || eap.line2 < 1
        || eap.line2 > buf.b_ml.ml_line_count
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
/// `eap` must be a live command.
pub unsafe fn ex_cbuffer(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- a live `exarg_T`.
    let eap = unsafe { Ea::new(eap) };
    let mut title = [0 as c_char; IOSIZE as usize];
    // SAFETY: forwarded from the caller.
    let au_name = cbuffer_get_auname(eap.cmdidx);
    if let Some(name) = au_name {
        let claimed = fire_qf_autocmd(EVENT_QUICKFIXCMDPRE, name, true);
        if claimed && aborting() {
            return;
        }
    }

    let (qi, wp) = qf_cmd_stack_or_alloc(eap);
    let args = unsafe { cbuffer_process_args(eap.raw()) };
    let Some(buf) = args else {
        return;
    };

    // The title names the buffer as well as the command. `qf_init_ext`
    // copies it, so this frame can own it.
    let mut qf_title = unsafe { qf_cmdtitle(*eap.cmdlinep) };
    if !buf.b_sfname.is_null() {
        let efile = IOSIZE as size_t;
        let fmt = c"%s (%s)".as_ptr();
        let sfname = buf.b_sfname;
        unsafe { vim_snprintf(title.as_mut_ptr(), efile, fmt, qf_title.as_ptr(), sfname) };
        qf_title[..IOSIZE as usize].copy_from_slice(&title);
    }

    incr_quickfix_busy();

    let newlist = !matches!(eap.cmdidx, CmdIdx::caddbuffer | CmdIdx::laddbuffer);
    let qi2 = qi.raw();
    let curlist = qi.qf_curlist;
    let errorformat2 = ptr::null();
    let qf_title2 = ptr::null_mut();
    let errorformat3 = p_efm.get();
    let line12 = eap.line1;
    let line22 = eap.line2;
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
        let curbuf_old: *const buf_T = curbuf.get();
        fire_qf_autocmd(EVENT_QUICKFIXCMDPOST, name, true);
        // The autocommand switched buffers: do not jump away from
        // wherever it left the user.
        if !ptr::eq(curbuf.get(), curbuf_old) {
            res = 0;
        }
    }

    let jumps = matches!(eap.cmdidx, CmdIdx::cbuffer | CmdIdx::lbuffer);
    if res > 0 && jumps && qf_list_still_valid(wp, save_qfid) {
        unsafe { qf_jump_first(qi.raw(), save_qfid, eap.forceit) };
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
///
/// # Safety
///
/// There must be a current buffer.
unsafe fn trigger_cexpr_autocmd(cmdidx: CmdIdx) -> bool {
    // SAFETY: the caller's promise.
    if let Some(name) = cexpr_get_auname(cmdidx) {
        let claimed = fire_qf_autocmd(EVENT_QUICKFIXCMDPRE, name, true);
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
/// `eap` must be a live command and `tv` a live value.
unsafe fn cexpr_core(eap: *const exarg_T, tv: *mut typval_T) -> Result<(), Failed> {
    // SAFETY: the caller's promise -- a live `exarg_T`.
    let eap = unsafe { Ea::new(eap.cast_mut()) };
    // SAFETY: forwarded from the caller.
    // The stack is asked for first, and so allocated for the current
    // window if it had none, even when the value turns out to be
    // unusable.
    let (qi, wp) = qf_cmd_stack_or_alloc(eap);

    let usable = unsafe { (*tv).v_type } == VAR_STRING && !unsafe { (*tv).vval.v_string.is_null() }
        || unsafe { (*tv).v_type } == VAR_LIST;
    if !usable {
        qf_emsg(c"E777: String or List expected".as_ptr());
        return Err(Failed);
    }

    let au_name = cexpr_get_auname(eap.cmdidx);

    incr_quickfix_busy();

    let newlist = !matches!(eap.cmdidx, CmdIdx::caddexpr | CmdIdx::laddexpr);
    let qi2 = qi.raw();
    let curlist = qi.qf_curlist;
    let errorformat2 = ptr::null();
    let buf2 = None;
    let errorformat3 = p_efm.get();
    let title = unsafe { qf_cmdtitle(*eap.cmdlinep) };
    let enc2 = ptr::null_mut();
    let res = unsafe {
        qf_init_ext(
            qi2,
            curlist,
            errorformat2,
            buf2,
            tv,
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
        fire_qf_autocmd(EVENT_QUICKFIXCMDPOST, name, true);
    }

    let jumps = matches!(eap.cmdidx, CmdIdx::cexpr | CmdIdx::lexpr);
    if res > 0 && jumps && qf_list_still_valid(wp, save_qfid) {
        unsafe { qf_jump_first(qi.raw(), save_qfid, eap.forceit) };
    }
    qf_busy_end();
    Ok(())
}

/// `:cexpr`, `:cgetexpr`, `:caddexpr` and their `:l…` twins.
///
/// # Safety
///
/// `eap` must be a live command.
pub unsafe fn ex_cexpr(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- a live `exarg_T`.
    let eap = unsafe { Ea::new(eap) };
    // SAFETY: forwarded from the caller.
    if !unsafe { trigger_cexpr_autocmd(eap.cmdidx) } {
        return;
    }
    // Evaluate the expression. When the result is a string or a list of
    // strings, parse each line and add it to the quickfix list.
    let tv = unsafe { eval_expr(eap.arg, eap.raw()) };
    if tv.is_null() {
        return;
    }
    let _ = unsafe { cexpr_core(eap.raw().cast_const(), tv) };
    unsafe { tv_free(tv) };
}
