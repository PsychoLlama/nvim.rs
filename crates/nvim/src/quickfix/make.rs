//! `:make` and `:grep`, which run an external command.
//!
//! [`ex_make`] builds the command line from `'makeprg'`/`'grepprg'`, runs
//! it with its output redirected to a temporary file ([`get_mef_name`]) and
//! then reads that file as an error file. `:grep` with
//! `'grepprg'` set to `internal` is handled by `:vimgrep` instead.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::os::shell::ShellOpts;
use crate::types::{CMD_grep, CMD_grepadd, CMD_lgrep, CMD_lgrepadd, CMD_lmake, CMD_make, NUL};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// True when `:grep` is to be run by `:vimgrep`, which is what `'grepprg'`
/// set to `internal` asks for. Only the `:grep` family can say it; `:make`
/// always runs a shell command.
///
/// # Safety
///
/// Reads the current buffer's options, so there must be one.
pub unsafe fn grep_internal(cmdidx: cmdidx_T) -> bool {
    if !matches!(cmdidx, CMD_grep | CMD_lgrep | CMD_grepadd | CMD_lgrepadd) {
        return false;
    }
    // SAFETY: the option strings of a live buffer are NUL-terminated.
    unsafe {
        let local = (*curbuf.get()).b_p_gp;
        let grepprg = if *local as c_int == NUL {
            p_gp.get()
        } else {
            local
        };
        CStr::from_ptr(grepprg) == c"internal"
    }
}

/// The name the `QuickFixCmdPre`/`QuickFixCmdPost` autocommands are matched
/// against, which is the command without its leading colon.
fn make_get_auname(cmdidx: cmdidx_T) -> Option<&'static CStr> {
    Some(match cmdidx {
        CMD_make => c"make",
        CMD_lmake => c"lmake",
        CMD_grep => c"grep",
        CMD_lgrep => c"lgrep",
        CMD_grepadd => c"grepadd",
        CMD_lgrepadd => c"lgrepadd",
        _ => return None,
    })
}

/// Form the complete command line to invoke `'makeprg'`/`'grepprg'`: quote
/// it with `'shellquote'` and append the `'shellpipe'` redirection to
/// `fname`. Echoes the result, so that the user sees what is being run.
///
/// Answers an `xmalloc`ed string the caller frees.
///
/// # Safety
///
/// Both strings must be NUL-terminated.
unsafe fn make_get_fullcmd(makecmd: *const c_char, fname: *const c_char) -> *mut c_char {
    // SAFETY: forwarded from the caller.
    unsafe {
        let quote = p_shq.get();
        let mut len = strlen(quote) * 2 + strlen(makecmd) + 1;
        // If 'shellpipe' is empty the output is not redirected at all.
        let redirect = *p_sp.get() as c_int != NUL;
        if redirect {
            len += strlen(p_sp.get()) + strlen(fname) + 3;
        }

        let cmd: *mut c_char = xmalloc(len).cast();
        snprintf(cmd, len, c"%s%s%s".as_ptr(), quote, makecmd, quote);
        if redirect {
            append_redir(cmd, len, p_sp.get(), fname);
        }

        // Display the fully formed command. Output a newline if there is
        // something else than the :make command that was typed, in which
        // case the cursor is in column 0.
        if msg_col.get() == 0 {
            msg_didout.set(false);
        }
        msg_start();
        msg_puts(c":!".as_ptr());
        msg_outtrans(cmd, 0, false);

        cmd
    }
}

/// `:make`, `:lmake`, `:grep`, `:lgrep`, `:grepadd` and `:lgrepadd`.
///
/// # Safety
///
/// `eap` must be a live command.
pub unsafe fn ex_make(eap: *mut exarg_T) {
    // SAFETY: forwarded from the caller.
    unsafe {
        // Redirect ":grep" to ":vimgrep" if 'grepprg' is "internal".
        if grep_internal((*eap).cmdidx) {
            ex_vimgrep(eap);
            return;
        }

        let local_enc = (*curbuf.get()).b_p_menc;
        let enc = if *local_enc as c_int != NUL {
            local_enc
        } else {
            p_menc.get()
        };

        let au_name = make_get_auname((*eap).cmdidx);
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

        let wp = if is_loclist_cmd((*eap).cmdidx as c_int) {
            curwin.get()
        } else {
            ptr::null_mut()
        };

        autowrite_all();
        let fname = get_mef_name();
        if fname.is_null() {
            return;
        }
        // In case the name is not unique after all.
        os_remove(fname);

        let cmd = make_get_fullcmd((*eap).arg, fname);
        do_shell(cmd, ShellOpts::NONE);

        incr_quickfix_busy();

        let is_make = matches!((*eap).cmdidx, CMD_make | CMD_lmake);
        let errorformat = if is_make {
            p_efm.get()
        } else {
            let local = (*curbuf.get()).b_p_gefm;
            if *local as c_int != NUL {
                local
            } else {
                p_gefm.get()
            }
        };
        let newlist = !matches!((*eap).cmdidx, CMD_grepadd | CMD_lgrepadd);

        let res = qf_init(
            wp,
            fname,
            errorformat,
            newlist as c_int,
            qf_cmdtitle(*(*eap).cmdlinep),
            enc,
        );

        let mut qi = ql_info.get();
        debug_assert!(!qi.is_null());
        // A location list command may have found no list to add to, in
        // which case there is nothing left to do but clean up.
        if !wp.is_null() {
            qi = win_loclist(wp);
        }
        if !qi.is_null() {
            if res >= 0 {
                qf_list_changed(qf_get_curlist(qi));
            }
            // Remember the current quickfix list identifier, so that a
            // QuickFixCmdPost autocommand changing the list is noticed.
            let save_qfid = (*qf_get_curlist(qi)).qf_id;
            if let Some(name) = au_name {
                apply_autocmds(
                    EVENT_QUICKFIXCMDPOST,
                    name.as_ptr().cast_mut(),
                    (*curbuf.get()).b_fname,
                    true,
                    curbuf.get(),
                );
            }
            if res > 0 && (*eap).forceit == 0 && qflist_valid(wp, save_qfid) {
                // Display the first error.
                qf_jump_first(qi, save_qfid, false as c_int);
            }
        }

        decr_quickfix_busy();
        os_remove(fname);
        xfree(fname.cast());
        xfree(cmd.cast());
    }
}

/// The name of the error file `:make` redirects into, in allocated memory,
/// or null when there is none to be had. An empty `'makeef'` asks for a
/// temporary name; a `'makeef'` holding `##` has that replaced by a number
/// pair chosen so that the file does not exist yet.
///
/// # Safety
///
/// Reads the options, so the editor must be initialised.
unsafe fn get_mef_name() -> *mut c_char {
    /// The process id, picked up once and then reused, with `off` counting
    /// up so that repeated calls in one session choose different names.
    static START: GlobalCell<c_int> = GlobalCell::new(-1);
    static OFF: GlobalCell<c_int> = GlobalCell::new(0);

    // SAFETY: the option strings are NUL-terminated.
    unsafe {
        if *p_mef.get() as c_int == NUL {
            let name = vim_tempname();
            if name.is_null() {
                emsg(gettext(&raw const e_notmp as *const c_char));
            }
            return name;
        }

        let makeef = CStr::from_ptr(p_mef.get()).to_bytes();
        let Some(at) = makeef.windows(2).position(|pair| pair == b"##") else {
            return xstrdup(p_mef.get());
        };

        // Keep trying until the name doesn't exist yet.
        loop {
            if START.get() == -1 {
                START.set(os_get_pid() as c_int);
            } else {
                OFF.set(OFF.get() + 19);
            }

            let mut digits = [0u8; 32];
            let written = snprintf(
                digits.as_mut_ptr().cast(),
                digits.len(),
                c"%d%d".as_ptr(),
                START.get(),
                OFF.get(),
            );
            debug_assert!(written > 0 && (written as usize) < digits.len());
            // Upstream writes the digits into the copy of 'makeef' with
            // `strlen(name)` as the bound, i.e. the length of 'makeef'
            // itself rather than the room left at `at`, so the pair is
            // truncated to one byte short of that. `'makeef'` of "##" thus
            // names a file after the first digit of the process id alone.
            let kept = (written as usize).min(makeef.len() - 1);

            let mut name = Vec::with_capacity(makeef.len() + 30);
            name.extend_from_slice(&makeef[..at]);
            name.extend_from_slice(&digits[..kept]);
            name.extend_from_slice(&makeef[at + 2..]);
            name.push(0);

            // Don't accept a symbolic link, it's a security risk.
            let mut file_info = FileInfo::default();
            if !os_fileinfo_link(name.as_ptr().cast(), &raw mut file_info) {
                return xstrdup(name.as_ptr().cast());
            }
        }
    }
}
