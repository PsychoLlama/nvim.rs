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
//! first error. They hold the stack under a [`QuickfixBusy`] throughout,
//! because the autocommands can close the window whose location list it is.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::eval::eval_cmd_arg;
use crate::memory::XString;
use crate::option::vars::P_EF;
use crate::option::vars::P_EFM;
use crate::option::vars::P_MENC;
use crate::optionstr::{OptString, local_or_global};
use crate::types::CmdIdx;
use crate::types::OptStr;
use crate::types::{Failed, NUL, OptionSetFlags, VAR_LIST};
use core::ffi::{CStr, c_int};

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
    let au_name = cfile_get_auname(excmd.cmdidx);
    if let Some(name) = au_name {
        let claimed = fire_qf_autocmd(AutoEvent::QuickFixCmdPre, name, false);
        if claimed && aborting() {
            return;
        }
    }

    if c_int::from(excmd.line.byte_at(excmd.line.arg)) != NUL {
        set_option_direct(
            kOptErrorfile,
            OptVal::String(OptStr::borrowing_bytes(excmd.line.arg())),
            OptionSetFlags::NONE,
            0 as ScriptId,
        );
    }

    let enc = local_or_global(&Buf::current().b_p_menc, P_MENC).get();

    let wp = is_loclist_cmd(excmd.cmdidx).then(Win::current);

    let busy = QuickfixBusy::hold();

    let newlist = !matches!(excmd.cmdidx, CmdIdx::caddfile | CmdIdx::laddfile);
    // Copies: `qf_init` reads a file and fires autocommands.
    let (efile, errorformat) = (P_EF.get(), P_EFM.get());
    let title = qf_cmdtitle(excmd.line.line());
    let res = qf_init(
        wp,
        efile.as_cstr(),
        errorformat.as_cstr(),
        true,
        newlist,
        Some(&title),
        Some(enc.as_cstr()),
    );

    let Some(qi) = stack_of(wp) else {
        drop(busy);
        return;
    };
    if res >= 0 {
        qi.current_slot().changed();
    }
    // Remember the current list, so that an autocommand replacing it is
    // noticed before the jump.
    let save_qfid = qi.current_list().id;
    if let Some(name) = au_name {
        fire_qf_autocmd(AutoEvent::QuickFixCmdPost, name, false);
    }

    let jumps = matches!(excmd.cmdidx, CmdIdx::cfile | CmdIdx::lfile);
    if res > 0 && jumps && qflist_valid(wp, save_qfid) {
        qf_jump_first(qi, save_qfid, excmd.forceit);
    }
    drop(busy);
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
    let buf = if c_int::from(excmd.line.byte_at(excmd.line.arg)) == NUL {
        Buf::current_or_none()
    } else if excmd.line.byte_at(
        excmd
            .line
            .skip_white(excmd.line.skip_digits(excmd.line.arg)),
    ) == 0
    {
        let at = excmd.line.arg;
        let (number, _) = getdigits_int_at(excmd.line.buffer_mut(), at, false, 0);
        find_buf(number)
    } else {
        None
    };

    let Some(buf) = buf else {
        qf_emsg(e_invarg);
        return None;
    };
    if buf.b_ml.ml_mfp.is_null() {
        qf_emsg(e_buffer_is_not_loaded);
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
        qf_emsg(e_invrange);
        return None;
    }
    Some(buf)
}

/// `:cbuffer`, `:cgetbuffer`, `:caddbuffer` and their `:l…` twins: parse a
/// range of lines of a buffer.
pub fn ex_cbuffer(excmd: &mut ExArg) {
    let au_name = cbuffer_get_auname(excmd.cmdidx);
    if let Some(name) = au_name {
        let claimed = fire_qf_autocmd(AutoEvent::QuickFixCmdPre, name, true);
        if claimed && aborting() {
            return;
        }
    }

    let (qi, wp) = stack_or_new_for_cmd(excmd);
    let parsed = cbuffer_process_args(excmd);
    let Some(buf) = parsed else {
        return;
    };

    // The title names the buffer as well as the command.
    let mut title = XString::from_cstr(&qf_cmdtitle(excmd.line.line()));
    if let Some(sfname) = buf.name.short() {
        let mut with_name = title.to_vec();
        with_name.extend_from_slice(b" (");
        with_name.extend_from_slice(sfname.to_bytes());
        with_name.push(b')');
        // Upstream formats this into an `IOSIZE` buffer.
        with_name.truncate(QF_TITLE_MAX);
        title = XString::from_bytes(&with_name);
    }

    let busy = QuickfixBusy::hold();

    let newlist = !matches!(excmd.cmdidx, CmdIdx::caddbuffer | CmdIdx::laddbuffer);
    // A copy: `qf_init_ext` reads the buffer and fires autocommands.
    let errorformat = P_EFM.get();
    let mut res = qf_init_ext(
        qi,
        qi.current,
        Input::Lines {
            buf,
            first: excmd.line1,
            last: excmd.line2,
        },
        Some(buf),
        errorformat.as_cstr(),
        true,
        newlist,
        Some(title.as_cstr()),
        None,
    );

    if qi.is_empty() {
        drop(busy);
        return;
    }
    if res >= 0 {
        qi.current_slot().changed();
    }
    let save_qfid = qi.current_list().id;
    if let Some(name) = au_name {
        let curbuf_old = Buf::current_or_none();
        fire_qf_autocmd(AutoEvent::QuickFixCmdPost, name, true);
        // The autocommand switched buffers: do not jump away from
        // wherever it left the user.
        if Buf::current_or_none() != curbuf_old {
            res = 0;
        }
    }

    let jumps = matches!(excmd.cmdidx, CmdIdx::cbuffer | CmdIdx::lbuffer);
    if res > 0 && jumps && qflist_valid(wp, save_qfid) {
        qf_jump_first(qi, save_qfid, excmd.forceit);
    }
    drop(busy);
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
fn cexpr_core(excmd: &mut ExArg, tv: &TypVal) -> Result<(), Failed> {
    // The stack is asked for first, and so allocated for the current
    // window if it had none, even when the value turns out to be
    // unusable.
    let (qi, wp) = stack_or_new_for_cmd(excmd);

    // A non-string reads as a NULL string, so the tag test is the accessor's.
    let usable = !tv.string_or_null().is_null() || tv.v_type() == VAR_LIST;
    if !usable {
        qf_emsg(c"E777: String or List expected");
        return Err(Failed);
    }

    let au_name = cexpr_get_auname(excmd.cmdidx);

    let busy = QuickfixBusy::hold();

    let newlist = !matches!(excmd.cmdidx, CmdIdx::caddexpr | CmdIdx::laddexpr);
    // A copy: `qf_init_ext` evaluates an expression and fires autocommands.
    let errorformat = P_EFM.get();
    let title = qf_cmdtitle(excmd.line.line());
    let res = qf_init_ext(
        qi,
        qi.current,
        Input::Value(tv),
        None,
        errorformat.as_cstr(),
        true,
        newlist,
        Some(&title),
        None,
    );

    if qi.is_empty() {
        drop(busy);
        return Err(Failed);
    }
    if res >= 0 {
        qi.current_slot().changed();
    }
    let save_qfid = qi.current_list().id;
    if let Some(name) = au_name {
        fire_qf_autocmd(AutoEvent::QuickFixCmdPost, name, true);
    }

    let jumps = matches!(excmd.cmdidx, CmdIdx::cexpr | CmdIdx::lexpr);
    if res > 0 && jumps && qflist_valid(wp, save_qfid) {
        qf_jump_first(qi, save_qfid, excmd.forceit);
    }
    drop(busy);
    Ok(())
}

/// `:cexpr`, `:cgetexpr`, `:caddexpr` and their `:l…` twins.
pub fn ex_cexpr(excmd: &mut ExArg) {
    if !trigger_cexpr_autocmd(excmd.cmdidx) {
        return;
    }
    // Evaluate the expression. When the result is a string or a list of
    // strings, parse each line and add it to the quickfix list.
    let Some(tv) = eval_cmd_arg(excmd) else {
        return;
    };
    let _ = cexpr_core(excmd, &tv);
}
