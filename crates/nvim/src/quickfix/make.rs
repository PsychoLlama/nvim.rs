//! `:make` and `:grep`, which run an external command.
//!
//! [`ex_make`] builds the command line from `'makeprg'`/`'grepprg'`, runs
//! it with its output redirected to a temporary file ([`get_mef_name`]) and
//! then reads that file as an error file. `:grep` with
//! `'grepprg'` set to `internal` is handled by `:vimgrep` instead.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::cstr;
use crate::ex_cmds::do_shell_cmd;
use crate::fileio::temp_name;
use crate::memory::XString;
use crate::option::vars::P_GEFM;
use crate::option::vars::P_GP;
use crate::option::vars::P_MENC;
use crate::option::vars::P_SHQ;
use crate::option::vars::P_SP;
use crate::option::vars::{P_EFM, P_MEF};
use crate::optionstr::{OptString, local_or_global};
use crate::os::fs::link_exists;
use crate::os::shell::ShellOpts;
use crate::types::CmdIdx;
use core::ffi::{CStr, c_int};
use std::ffi::CString;

/// True when `:grep` is to be run by `:vimgrep`, which is what `'grepprg'`
/// set to `internal` asks for. Only the `:grep` family can say it; `:make`
/// always runs a shell command.
pub fn grep_internal(cmdidx: CmdIdx) -> bool {
    if !matches!(
        cmdidx,
        CmdIdx::grep | CmdIdx::lgrep | CmdIdx::grepadd | CmdIdx::lgrepadd
    ) {
        return false;
    }
    local_or_global(&Buf::current().b_p_gp, P_GP).get() == c"internal"
}

/// The name the `QuickFixCmdPre`/`QuickFixCmdPost` autocommands are matched
/// against, which is the command without its leading colon.
fn make_get_auname(cmdidx: CmdIdx) -> Option<&'static CStr> {
    Some(match cmdidx {
        CmdIdx::make => c"make",
        CmdIdx::lmake => c"lmake",
        CmdIdx::grep => c"grep",
        CmdIdx::lgrep => c"lgrep",
        CmdIdx::grepadd => c"grepadd",
        CmdIdx::lgrepadd => c"lgrepadd",
        _ => return None,
    })
}

/// Form the complete command line to invoke `'makeprg'`/`'grepprg'`: quote
/// it with `'shellquote'` and append the `'shellpipe'` redirection to
/// `fname`. Echoes the result, so that the user sees what is being run.
fn make_get_fullcmd(makecmd: &[u8], fname: &CStr) -> CString {
    // Copies of the two option values: the command line is built out of
    // them and `append_redir` reads one past the end of a projection.
    let (shq, sp) = (P_SHQ.get(), P_SP.get());
    // If 'shellpipe' is empty the output is not redirected at all.
    let (redirect, pipe) = (!sp.is_empty(), sp.as_cstr());
    let quote = &*shq;

    let mut cmd: Vec<u8> = Vec::new();
    cmd.extend_from_slice(quote);
    cmd.extend_from_slice(makecmd);
    cmd.extend_from_slice(quote);
    if redirect {
        append_redir(&mut cmd, pipe, fname);
    }
    let cmd = cstr::owned(&cmd);

    // Display the fully formed command. Output a newline if there is
    // something else than the :make command that was typed, in which
    // case the cursor is in column 0.
    if msg_col.get() == 0 {
        msg_didout.set(false);
    }
    msg_start();
    msg_str(c":!");
    msg_display(&cmd, 0, false);

    cmd
}

/// `:make`, `:lmake`, `:grep`, `:lgrep`, `:grepadd` and `:lgrepadd`.
pub fn ex_make(excmd: &mut ExArg) {
    // Redirect ":grep" to ":vimgrep" if 'grepprg' is "internal".
    if grep_internal(excmd.cmdidx) {
        ex_vimgrep(excmd);
        return;
    }

    let enc = local_or_global(&Buf::current().b_p_menc, P_MENC).get();

    let au_name = make_get_auname(excmd.cmdidx);
    if let Some(name) = au_name {
        let claimed = fire_qf_autocmd(AutoEvent::QuickFixCmdPre, name, true);
        if claimed && aborting() {
            return;
        }
    }

    let wp = is_loclist_cmd(excmd.cmdidx).then(Win::current);

    autowrite_all();
    let Some(fname) = get_mef_name() else {
        return;
    };
    // In case the name is not unique after all.
    os_remove(fname.as_cstr());

    let cmd = make_get_fullcmd(excmd.line.arg(), fname.as_cstr());
    do_shell_cmd(&cmd, ShellOpts::NONE);

    let busy = QuickfixBusy::hold();

    let is_make = matches!(excmd.cmdidx, CmdIdx::make | CmdIdx::lmake);
    let errorformat = if is_make {
        P_EFM.get()
    } else {
        local_or_global(&Buf::current().b_p_gefm, P_GEFM).get()
    };
    let newlist = !matches!(excmd.cmdidx, CmdIdx::grepadd | CmdIdx::lgrepadd);

    let title = qf_cmdtitle(excmd.line.line());
    let res = qf_init(
        wp,
        fname.as_cstr(),
        errorformat.as_cstr(),
        // `:make` reads the global 'errorformat'; `:grep` reads
        // 'grepformat', which is never the buffer's own.
        is_make,
        newlist,
        Some(&title),
        Some(enc.as_cstr()),
    );

    // A location list command may have found no list to add to, in
    // which case there is nothing left to do but clean up.
    if let Some(qi) = stack_of(wp) {
        if res >= 0 {
            qi.current_slot().changed();
        }
        // Remember the current quickfix list identifier, so that a
        // QuickFixCmdPost autocommand changing the list is noticed.
        let save_qfid = qi.current_list().id;
        if let Some(name) = au_name {
            fire_qf_autocmd(AutoEvent::QuickFixCmdPost, name, true);
        }
        if res > 0 && !excmd.forceit && qflist_valid(wp, save_qfid) {
            // Display the first error.
            qf_jump_first(qi, save_qfid, false);
        }
    }

    drop(busy);
    os_remove(fname.as_cstr());
    drop(cmd);
}

/// The name of the error file `:make` redirects into, or `None` when there
/// is none to be had. An empty `'makeef'` asks for a temporary name; a
/// `'makeef'` holding `##` has that replaced by a number pair chosen so that
/// the file does not exist yet.
fn get_mef_name() -> Option<XString> {
    /// The process id, picked up once and then reused, with `off` counting
    /// up so that repeated calls in one session choose different names.
    static START: GlobalCell<c_int> = GlobalCell::new(-1);
    static OFF: GlobalCell<c_int> = GlobalCell::new(0);

    if p_mef(CStr::is_empty) {
        let name = temp_name();
        if name.is_none() {
            qf_emsg(e_notmp);
        }
        return name;
    }

    // A copy: the name is built out of it below.
    let mef = P_MEF.get();
    let makeef = &*mef;
    let Some(at) = makeef.windows(2).position(|pair| pair == b"##") else {
        return Some(mef);
    };

    // Keep trying until the name doesn't exist yet.
    loop {
        if START.get() == -1 {
            // Upstream's `(int)` narrowing of the process id.
            #[allow(clippy::cast_possible_truncation)]
            START.set(os_get_pid() as c_int);
        } else {
            OFF.set(OFF.get() + 19);
        }

        let digits = format!("{}{}", START.get(), OFF.get());
        // Upstream writes the digits into the copy of 'makeef' with
        // `strlen(name)` as the bound, i.e. the length of 'makeef'
        // itself rather than the room left at `at`, so the pair is
        // truncated to one byte short of that. `'makeef'` of "##" thus
        // names a file after the first digit of the process id alone.
        let kept = digits.len().min(makeef.len() - 1);

        let mut name = Vec::with_capacity(makeef.len() + 30);
        name.extend_from_slice(&makeef[..at]);
        name.extend_from_slice(&digits.as_bytes()[..kept]);
        name.extend_from_slice(&makeef[at + 2..]);
        let name = XString::from_bytes(&name);

        // Don't accept a symbolic link, it's a security risk.
        if !link_exists(name.as_cstr()) {
            return Some(name);
        }
    }
}
