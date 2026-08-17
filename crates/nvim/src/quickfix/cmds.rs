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
    CMD_lfile, CMD_lgetbuffer, CMD_lgetexpr, CMD_lgetfile, VAR_LIST, VAR_STRING,
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
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut qi = ql_info.get();
        debug_assert!(!qi.is_null());

        let au_name = cfile_get_auname((*eap).cmdidx);
        if let Some(name) = au_name {
            let claimed = apply_autocmds(
                EVENT_QUICKFIXCMDPRE,
                name.as_ptr().cast_mut(),
                ptr::null_mut(),
                false,
                curbuf.get(),
            );
            if claimed && aborting() {
                return;
            }
        }

        if *(*eap).arg as c_int != NUL {
            set_option_direct(
                kOptErrorfile,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: cstr_as_string((*eap).arg),
                    },
                },
                0,
                0 as scid_T,
            );
        }

        let local_enc = (*curbuf.get()).b_p_menc;
        let enc = if *local_enc as c_int != NUL {
            local_enc
        } else {
            p_menc.get()
        };

        let mut wp: *mut win_T = ptr::null_mut();
        if is_loclist_cmd((*eap).cmdidx as c_int) {
            wp = curwin.get();
        }

        incr_quickfix_busy();

        let newlist = !matches!((*eap).cmdidx, CMD_caddfile | CMD_laddfile);
        let res = qf_init(
            wp,
            p_ef.get(),
            p_efm.get(),
            newlist as c_int,
            qf_cmdtitle(*(*eap).cmdlinep),
            enc,
        );

        if !wp.is_null() {
            qi = win_loclist(wp);
            if qi.is_null() {
                decr_quickfix_busy();
                return;
            }
        }
        if res >= 0 {
            qf_list_changed(qf_get_curlist(qi));
        }
        // Remember the current list, so that an autocommand replacing it is
        // noticed before the jump.
        let save_qfid = (*qf_get_curlist(qi)).qf_id;
        if let Some(name) = au_name {
            apply_autocmds(
                EVENT_QUICKFIXCMDPOST,
                name.as_ptr().cast_mut(),
                ptr::null_mut(),
                false,
                curbuf.get(),
            );
        }

        let jumps = matches!((*eap).cmdidx, CMD_cfile | CMD_lfile);
        if res > 0 && jumps && qflist_valid(wp, save_qfid) {
            qf_jump_first(qi, save_qfid, (*eap).forceit);
        }
        decr_quickfix_busy();
    }
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
    // SAFETY: forwarded from the caller.
    unsafe {
        let buf = if *(*eap).arg as c_int == NUL {
            curbuf.get()
        } else if *skipwhite(skipdigits((*eap).arg)) as c_int == NUL {
            buflist_findnr(atoi((*eap).arg))
        } else {
            ptr::null_mut()
        };

        if buf.is_null() {
            emsg(gettext(&raw const e_invarg as *const c_char));
            return None;
        }
        if (*buf).b_ml.ml_mfp.is_null() {
            emsg(gettext(&raw const e_buffer_is_not_loaded as *const c_char));
            return None;
        }

        if (*eap).addr_count == 0 {
            (*eap).line1 = 1;
            (*eap).line2 = (*buf).b_ml.ml_line_count;
        }
        if (*eap).line1 < 1
            || (*eap).line1 > (*buf).b_ml.ml_line_count
            || (*eap).line2 < 1
            || (*eap).line2 > (*buf).b_ml.ml_line_count
        {
            emsg(gettext(&raw const e_invrange as *const c_char));
            return None;
        }
        Some(buf)
    }
}

/// `:cbuffer`, `:cgetbuffer`, `:caddbuffer` and their `:l…` twins: parse a
/// range of lines of a buffer.
///
/// # Safety
///
/// `eap` must be a live command.
pub unsafe fn ex_cbuffer(eap: *mut exarg_T) {
    // SAFETY: forwarded from the caller.
    unsafe {
        let au_name = cbuffer_get_auname((*eap).cmdidx);
        if let Some(name) = au_name {
            let claimed = apply_autocmds(
                EVENT_QUICKFIXCMDPRE,
                name.as_ptr().cast_mut(),
                (*curbuf.get()).b_fname,
                true,
                curbuf.get(),
            );
            if claimed && aborting() {
                return;
            }
        }

        let mut wp: *mut win_T = ptr::null_mut();
        let qi = qf_cmd_get_or_alloc_stack(eap, &raw mut wp);
        let Some(buf) = cbuffer_process_args(eap) else {
            return;
        };

        // The title names the buffer as well as the command.
        let mut qf_title = qf_cmdtitle(*(*eap).cmdlinep);
        if !(*buf).b_sfname.is_null() {
            vim_snprintf(
                IObuff.ptr().cast(),
                IOSIZE as size_t,
                c"%s (%s)".as_ptr(),
                qf_title,
                (*buf).b_sfname,
            );
            qf_title = IObuff.ptr().cast();
        }

        incr_quickfix_busy();

        let newlist = !matches!((*eap).cmdidx, CMD_caddbuffer | CMD_laddbuffer);
        let mut res = qf_init_ext(
            qi,
            (*qi).qf_curlist,
            ptr::null(),
            buf,
            ptr::null_mut(),
            p_efm.get(),
            newlist,
            (*eap).line1,
            (*eap).line2,
            qf_title,
            ptr::null_mut(),
        );

        if qf_stack_empty(qi) {
            decr_quickfix_busy();
            return;
        }
        if res >= 0 {
            qf_list_changed(qf_get_curlist(qi));
        }
        let save_qfid = (*qf_get_curlist(qi)).qf_id;
        if let Some(name) = au_name {
            let curbuf_old: *const buf_T = curbuf.get();
            apply_autocmds(
                EVENT_QUICKFIXCMDPOST,
                name.as_ptr().cast_mut(),
                (*curbuf.get()).b_fname,
                true,
                curbuf.get(),
            );
            // The autocommand switched buffers: do not jump away from
            // wherever it left the user.
            if !ptr::eq(curbuf.get(), curbuf_old) {
                res = 0;
            }
        }

        let jumps = matches!((*eap).cmdidx, CMD_cbuffer | CMD_lbuffer);
        if res > 0 && jumps && qflist_valid(wp, save_qfid) {
            qf_jump_first(qi, save_qfid, (*eap).forceit);
        }
        decr_quickfix_busy();
    }
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
    unsafe {
        if let Some(name) = cexpr_get_auname(cmdidx) {
            let claimed = apply_autocmds(
                EVENT_QUICKFIXCMDPRE,
                name.as_ptr().cast_mut(),
                (*curbuf.get()).b_fname,
                true,
                curbuf.get(),
            );
            if claimed && aborting() {
                return false;
            }
        }
        true
    }
}

/// Build a list out of an already evaluated string or list of strings.
///
/// # Safety
///
/// `eap` must be a live command and `tv` a live value.
unsafe fn cexpr_core(eap: *const exarg_T, tv: *mut typval_T) -> c_int {
    // SAFETY: forwarded from the caller.
    unsafe {
        // The stack is asked for first, and so allocated for the current
        // window if it had none, even when the value turns out to be
        // unusable.
        let mut wp: *mut win_T = ptr::null_mut();
        let qi = qf_cmd_get_or_alloc_stack(eap, &raw mut wp);

        let usable = (*tv).v_type == VAR_STRING && !(*tv).vval.v_string.is_null()
            || (*tv).v_type == VAR_LIST;
        if !usable {
            emsg(gettext(c"E777: String or List expected".as_ptr()));
            return FAIL;
        }

        let au_name = cexpr_get_auname((*eap).cmdidx);

        incr_quickfix_busy();

        let newlist = !matches!((*eap).cmdidx, CMD_caddexpr | CMD_laddexpr);
        let res = qf_init_ext(
            qi,
            (*qi).qf_curlist,
            ptr::null(),
            ptr::null_mut(),
            tv,
            p_efm.get(),
            newlist,
            0,
            0,
            qf_cmdtitle(*(*eap).cmdlinep),
            ptr::null_mut(),
        );

        if qf_stack_empty(qi) {
            decr_quickfix_busy();
            return FAIL;
        }
        if res >= 0 {
            qf_list_changed(qf_get_curlist(qi));
        }
        let save_qfid: c_uint = (*qf_get_curlist(qi)).qf_id;
        if let Some(name) = au_name {
            apply_autocmds(
                EVENT_QUICKFIXCMDPOST,
                name.as_ptr().cast_mut(),
                (*curbuf.get()).b_fname,
                true,
                curbuf.get(),
            );
        }

        let jumps = matches!((*eap).cmdidx, CMD_cexpr | CMD_lexpr);
        if res > 0 && jumps && qflist_valid(wp, save_qfid) {
            qf_jump_first(qi, save_qfid, (*eap).forceit);
        }
        decr_quickfix_busy();
        OK
    }
}

/// `:cexpr`, `:cgetexpr`, `:caddexpr` and their `:l…` twins.
///
/// # Safety
///
/// `eap` must be a live command.
pub unsafe fn ex_cexpr(eap: *mut exarg_T) {
    // SAFETY: forwarded from the caller.
    unsafe {
        if !trigger_cexpr_autocmd((*eap).cmdidx) {
            return;
        }
        // Evaluate the expression. When the result is a string or a list of
        // strings, parse each line and add it to the quickfix list.
        let tv = eval_expr((*eap).arg, eap);
        if tv.is_null() {
            return;
        }
        cexpr_core(eap, tv);
        tv_free(tv);
    }
}
