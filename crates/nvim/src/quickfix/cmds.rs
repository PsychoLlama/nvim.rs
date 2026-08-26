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
use crate::types::{
    CMD_caddbuffer, CMD_caddexpr, CMD_caddfile, CMD_cbuffer, CMD_cexpr, CMD_cfile, CMD_cgetbuffer,
    CMD_cgetexpr, CMD_cgetfile, CMD_laddbuffer, CMD_laddexpr, CMD_laddfile, CMD_lbuffer, CMD_lexpr,
    CMD_lfile, CMD_lgetbuffer, CMD_lgetexpr, CMD_lgetfile, FAIL, IOSIZE, NUL, OK, OptionSetFlags,
    VAR_LIST, VAR_STRING,
};
use core::ffi::{CStr, c_char, c_int, c_uint};
use core::ptr;

/// The autocommand name of a `:cfile`-family command.
fn cfile_get_auname(cmdidx: cmdidx_T) -> Option<&'static CStr> {
    Some(match cmdidx {
        CMD_cfile => c"cfile",
        CMD_cgetfile => c"cgetfile",
        CMD_caddfile => c"caddfile",
        CMD_lfile => c"lfile",
        CMD_lgetfile => c"lgetfile",
        CMD_laddfile => c"laddfile",
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

    let au_name = cfile_get_auname((*eap).cmdidx);
    if let Some(name) = au_name {
        let claimed = fire_qf_autocmd(EVENT_QUICKFIXCMDPRE, name, false);
        if claimed && aborting() {
            return;
        }
    }

    if unsafe { *(*eap).arg } as c_int != NUL {
        set_option_direct(
            kOptErrorfile,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: unsafe { cstr_as_string((*eap).arg) },
                },
            },
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

    let mut wp: *mut win_T = ptr::null_mut();
    if unsafe { is_loclist_cmd((*eap).cmdidx as c_int) } {
        wp = curwin.get();
    }

    incr_quickfix_busy();

    let newlist = !matches!((*eap).cmdidx, CMD_caddfile | CMD_laddfile);
    let efile = p_ef.get();
    let errorformat2 = p_efm.get();
    let newlist2 = newlist as c_int;
    let title = unsafe { qf_cmdtitle(*(*eap).cmdlinep) };
    let qf_title2 = title.as_ptr();
    let res = unsafe { qf_init(wp, efile, errorformat2, newlist2, qf_title2, enc) };

    if !wp.is_null() {
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

    let jumps = matches!((*eap).cmdidx, CMD_cfile | CMD_lfile);
    if res > 0 && jumps && qf_list_still_valid(wp, save_qfid) {
        unsafe { qf_jump_first(qi.raw(), save_qfid, (*eap).forceit) };
    }
    qf_busy_end();
}

/// The autocommand name of a `:cbuffer`-family command.
fn cbuffer_get_auname(cmdidx: cmdidx_T) -> Option<&'static CStr> {
    Some(match cmdidx {
        CMD_cbuffer => c"cbuffer",
        CMD_cgetbuffer => c"cgetbuffer",
        CMD_caddbuffer => c"caddbuffer",
        CMD_lbuffer => c"lbuffer",
        CMD_lgetbuffer => c"lgetbuffer",
        CMD_laddbuffer => c"laddbuffer",
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
unsafe fn cbuffer_process_args(eap: *mut exarg_T) -> Option<*mut buf_T> {
    // SAFETY: the caller's promise -- a live `exarg_T`.
    let mut eap = unsafe { Ea::new(eap) };
    // SAFETY: forwarded from the caller.
    let buf = if unsafe { *(*eap).arg } as c_int == NUL {
        curbuf.get()
    } else if unsafe { *skipwhite(skipdigits((*eap).arg)) } as c_int == NUL {
        buflist_findnr(unsafe { atoi((*eap).arg) })
    } else {
        ptr::null_mut()
    };

    if buf.is_null() {
        qf_emsg(&raw const e_invarg as *const c_char);
        return None;
    }
    if unsafe { (*buf).b_ml.ml_mfp.is_null() } {
        qf_emsg(&raw const e_buffer_is_not_loaded as *const c_char);
        return None;
    }

    if (*eap).addr_count == 0 {
        (*eap).line1 = 1;
        unsafe { (*eap).line2 = (*buf).b_ml.ml_line_count };
    }
    if (*eap).line1 < 1
        || (*eap).line1 > unsafe { (*buf).b_ml.ml_line_count }
        || (*eap).line2 < 1
        || (*eap).line2 > unsafe { (*buf).b_ml.ml_line_count }
    {
        qf_emsg(&raw const e_invrange as *const c_char);
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
    let au_name = cbuffer_get_auname((*eap).cmdidx);
    if let Some(name) = au_name {
        let claimed = fire_qf_autocmd(EVENT_QUICKFIXCMDPRE, name, true);
        if claimed && aborting() {
            return;
        }
    }

    let mut wp: *mut win_T = ptr::null_mut();
    let qi = qf_cmd_stack_or_alloc(eap, &raw mut wp);
    let args = unsafe { cbuffer_process_args(eap.raw()) };
    let Some(buf) = args else {
        return;
    };

    // The title names the buffer as well as the command. `qf_init_ext`
    // copies it, so this frame can own it.
    let mut qf_title = unsafe { qf_cmdtitle(*(*eap).cmdlinep) };
    if !unsafe { (*buf).b_sfname.is_null() } {
        let efile = IOSIZE as size_t;
        let fmt = c"%s (%s)".as_ptr();
        let sfname = unsafe { (*buf).b_sfname };
        unsafe { vim_snprintf(title.as_mut_ptr(), efile, fmt, qf_title.as_ptr(), sfname) };
        qf_title[..IOSIZE as usize].copy_from_slice(&title);
    }

    incr_quickfix_busy();

    let newlist = !matches!((*eap).cmdidx, CMD_caddbuffer | CMD_laddbuffer);
    let qi2 = qi.raw();
    let curlist = (*qi).qf_curlist;
    let errorformat2 = ptr::null();
    let qf_title2 = ptr::null_mut();
    let errorformat3 = p_efm.get();
    let line12 = (*eap).line1;
    let line22 = (*eap).line2;
    let enc2 = ptr::null_mut();
    let mut res = unsafe {
        qf_init_ext(
            qi2,
            curlist,
            errorformat2,
            buf,
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

    let jumps = matches!((*eap).cmdidx, CMD_cbuffer | CMD_lbuffer);
    if res > 0 && jumps && qf_list_still_valid(wp, save_qfid) {
        unsafe { qf_jump_first(qi.raw(), save_qfid, (*eap).forceit) };
    }
    qf_busy_end();
}

/// The autocommand name of a `:cexpr`-family command.
fn cexpr_get_auname(cmdidx: cmdidx_T) -> Option<&'static CStr> {
    Some(match cmdidx {
        CMD_cexpr => c"cexpr",
        CMD_cgetexpr => c"cgetexpr",
        CMD_caddexpr => c"caddexpr",
        CMD_lexpr => c"lexpr",
        CMD_lgetexpr => c"lgetexpr",
        CMD_laddexpr => c"laddexpr",
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
unsafe fn trigger_cexpr_autocmd(cmdidx: cmdidx_T) -> bool {
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
unsafe fn cexpr_core(eap: *const exarg_T, tv: *mut typval_T) -> c_int {
    // SAFETY: the caller's promise -- a live `exarg_T`.
    let eap = unsafe { Ea::new(eap.cast_mut()) };
    // SAFETY: forwarded from the caller.
    // The stack is asked for first, and so allocated for the current
    // window if it had none, even when the value turns out to be
    // unusable.
    let mut wp: *mut win_T = ptr::null_mut();
    let qi = qf_cmd_stack_or_alloc(eap, &raw mut wp);

    let usable = unsafe { (*tv).v_type } == VAR_STRING && !unsafe { (*tv).vval.v_string.is_null() }
        || unsafe { (*tv).v_type } == VAR_LIST;
    if !usable {
        qf_emsg(c"E777: String or List expected".as_ptr());
        return FAIL;
    }

    let au_name = cexpr_get_auname((*eap).cmdidx);

    incr_quickfix_busy();

    let newlist = !matches!((*eap).cmdidx, CMD_caddexpr | CMD_laddexpr);
    let qi2 = qi.raw();
    let curlist = (*qi).qf_curlist;
    let errorformat2 = ptr::null();
    let buf2 = ptr::null_mut();
    let errorformat3 = p_efm.get();
    let title = unsafe { qf_cmdtitle(*(*eap).cmdlinep) };
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
        return FAIL;
    }
    if res >= 0 {
        qfl_changed(qf_current_list(qi));
    }
    let save_qfid: c_uint = qf_current_list(qi).qf_id;
    if let Some(name) = au_name {
        fire_qf_autocmd(EVENT_QUICKFIXCMDPOST, name, true);
    }

    let jumps = matches!((*eap).cmdidx, CMD_cexpr | CMD_lexpr);
    if res > 0 && jumps && qf_list_still_valid(wp, save_qfid) {
        unsafe { qf_jump_first(qi.raw(), save_qfid, (*eap).forceit) };
    }
    qf_busy_end();
    OK
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
    if !unsafe { trigger_cexpr_autocmd((*eap).cmdidx) } {
        return;
    }
    // Evaluate the expression. When the result is a string or a list of
    // strings, parse each line and add it to the quickfix list.
    let tv = unsafe { eval_expr((*eap).arg, eap.raw()) };
    if tv.is_null() {
        return;
    }
    unsafe { cexpr_core(eap.raw().cast_const(), tv) };
    unsafe { tv_free(tv) };
}
