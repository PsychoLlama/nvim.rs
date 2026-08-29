//! Running the user's `'shell'`.
//!
//! # Boundary
//!
//! Nothing here talks to the OS directly: the child is spawned through
//! libuv ([`system`]), and its output reaches the screen through the message
//! layer ([`throttle`]). What this file owns is everything *around* that —
//! turning `'shell'`, `'shellcmdflag'`, `'shellxquote'` and `'shellxescape'`
//! into an argument vector, deciding where the output goes, and pushing it
//! into the current buffer for `:r !cmd`.
//!
//! - [`shell_build_argv`] and friends: the argument vector.
//! - [`os_call_shell`] / [`call_shell`]: run a command, honouring the
//!   `ShellOpts` below. `call_shell` is the one that also does `'verbose'`,
//!   `:profile` and `v:shell_error`.
//! - [`get_cmd_output`]: run a command and hand back its stdout.
//! - [`expand`]: wildcard expansion, which is a shell command too.

#![deny(unsafe_op_in_unsafe_fn)]

pub mod expand;
pub mod system;
mod throttle;

use crate::{semsg_c, smsg_c};
pub use expand::os_expand_wildcards;
pub use system::os_system;

use crate::buffer::read_buffer_into;
use crate::charset::skipwhite;
use crate::eval::vars::set_vim_var_nr;
use crate::event::multiqueue::multiqueue_put_event;
use crate::ex_cmds::{check_secure, make_filter_cmd};
use crate::fileio::vim_tempname;
use crate::global_cell::GlobalCell;
use crate::kvec::Kvec;
use crate::main::{
    State, curbuf, curwin, do_profiling, e_cannot_read_from_str_2, e_cant_read_file_str, e_notmp,
    e_shellempty, emsg_silent, main_loop, no_check_timestamps, p_sh, p_shcf, p_sxe, p_sxq,
    p_verbose,
};
use crate::memline::ml_append;
use crate::memory::{xcalloc, xfree, xmalloc, xstrdup, xstrlcat};
use crate::message::{
    emsg, msg_ext_set_kind, msg_outnum, msg_putchar, msg_puts, verbose_enter, verbose_leave,
};
use crate::os::cshim::gettext;
use crate::os::fs::{os_fopen, os_remove};
use crate::os::signal::{signal_accept_deadly, signal_reject_deadly};
use crate::profile::{prof_child_enter, prof_child_exit};
use crate::state::MODE_EXTERNCMD;
use crate::strings::{vim_snprintf, vim_strnsave_unquoted, vim_strsave_escaped_ext};
use crate::tag::tag_freematch;
use crate::types::ui::kUIMessages;
use crate::types::{
    NUL, READBIN, StringBuilder, Vv, linenr_T, proftime_T, size_t, stream_read_cb, varnumber_T,
};
use crate::ui::{ui_flush, ui_has};
use crate::winlayer::Buf;
use ::libc::{fclose, fopen, fread, fseek, ftell, strcmp, strcpy, strlen};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;
use std::ffi::CString;

crate::flag_set! {
    /// How a shell command's input and output are wired up — upstream's
    /// `ShellOpts` enum, whose values are always combined with `|`.
    pub struct ShellOpts;

    /// The command filters the buffer (`:{range}!cmd`). Only `ex_cmds/`
    /// and `diff/` set it, and only [`call_shell`]'s `'shelltemp'` path
    /// and the `:diff` writer read it back.
    const FILTER = 1;
    /// The command is a wildcard expansion, not something the user typed:
    /// no mode change, no output forwarding.
    const EXPAND = 2;
    /// The caller has already redirected the output somewhere.
    const DO_OUT = 4;
    /// Do not report a non-zero exit code.
    const SILENT = 8;
    /// Capture the output and write it into the editor.
    const READ = 16;
    /// Feed the current buffer to the command's standard input.
    const WRITE = 32;
    /// Run without any message at all, not even the mode change.
    const HIDE_MESS = 64;
}

const PROF_YES: c_int = 1;
const SEEK_SET: c_int = 0;
const SEEK_END: c_int = 2;

const TAB: c_int = '\t' as c_int;
const NL: c_int = '\n' as c_int;
const CAR: c_int = '\r' as c_int;

/// Characters a shell would act on, so they get a backslash when a pattern is
/// pasted into a command line.
const SHELL_SPECIAL: &[u8] = b"\t \"&'$;<>()\\|\n";

/// An empty `StringBuilder` — klib's `KV_INITIAL_VALUE`.
const STRINGBUILDER_INIT: StringBuilder = StringBuilder {
    size: 0,
    capacity: 0,
    items: ptr::null_mut(),
};

/// The argument vector for running `'shell'` with an optional command
/// prefixed by `'shellcmdflag'`, e.g.
/// `["shell", "-extra_args", "-shellcmdflag", "command with spaces"]`.
///
/// `cmd` is NULL for an interactive shell, `extra_args` NULL for none. The
/// result is newly allocated and must be freed with [`shell_free_argv`].
///
/// # Safety
/// `cmd` and `extra_args` must be NUL-terminated strings or NULL.
pub unsafe fn shell_build_argv(cmd: *const c_char, extra_args: *const c_char) -> *mut *mut c_char {
    // SAFETY: the caller's contract; `p_sh`/`p_shcf` are option values.
    unsafe {
        // Counted first, because the vector is allocated once: the words of
        // 'shell', the words of 'shellcmdflag', `extra_args`, `cmd`, NULL.
        let argc = tokenize(p_sh.get(), ptr::null_mut())
            + if cmd.is_null() {
                0
            } else {
                tokenize(p_shcf.get(), ptr::null_mut())
            };
        let rv = xmalloc((argc + 4) * size_of::<*mut c_char>()) as *mut *mut c_char;

        let mut i = tokenize(p_sh.get(), rv);
        if !extra_args.is_null() {
            *rv.add(i) = xstrdup(extra_args);
            i += 1;
        }
        if !cmd.is_null() {
            i += tokenize(p_shcf.get(), rv.add(i));
            *rv.add(i) = shell_xescape_xquote(cmd);
            i += 1;
        }
        *rv.add(i) = ptr::null_mut();
        debug_assert!(!(*rv).is_null());
        rv
    }
}

/// Release what [`shell_build_argv`] allocated.
///
/// # Safety
/// `argv` must be NULL or a vector from [`shell_build_argv`].
pub unsafe fn shell_free_argv(argv: *mut *mut c_char) {
    if argv.is_null() {
        return;
    }
    // SAFETY: the caller's contract; the vector is NULL-terminated.
    unsafe {
        let mut p = argv;
        while !(*p).is_null() {
            xfree((*p).cast());
            p = p.add(1);
        }
        xfree(argv.cast());
    }
}

/// Join `argv` into one quoted string, for reporting. Truncated with an
/// ellipsis if it does not fit in 256 bytes.
///
/// # Safety
/// `argv` must be a NULL-terminated vector of NUL-terminated strings.
pub unsafe fn shell_argv_to_str(argv: *mut *mut c_char) -> *mut c_char {
    const MAXSIZE: usize = 256;
    // SAFETY: the caller's contract. `xstrlcat` always terminates and answers
    // the length the result would have had.
    unsafe {
        let rv = xcalloc(MAXSIZE, size_of::<c_char>()) as *mut c_char;
        let mut p = argv;
        if (*p).is_null() {
            return rv;
        }
        let mut n = 0;
        while !(*p).is_null() {
            xstrlcat(rv, c"'".as_ptr(), MAXSIZE);
            xstrlcat(rv, *p, MAXSIZE);
            n = xstrlcat(rv, c"' ".as_ptr(), MAXSIZE);
            if n >= MAXSIZE {
                break;
            }
            p = p.add(1);
        }
        if n < MAXSIZE {
            // Drop the trailing space.
            *rv.add(n - 1) = 0;
        } else {
            // Too long: "/bin/bash 'foo' 'bar'..."
            strcpy(rv.add(MAXSIZE - 4), c"...".as_ptr());
        }
        rv
    }
}

/// Run `cmd` through `'shell'`, or start an interactive shell when it is
/// NULL. `opts` says how the command's streams are wired up.
///
/// Answers the command's exit code.
///
/// # Safety
/// `cmd` and `extra_args` must be NUL-terminated strings or NULL.
pub unsafe fn os_call_shell(cmd: *mut c_char, opts: ShellOpts, extra_args: *mut c_char) -> c_int {
    let mut input = STRINGBUILDER_INIT;
    let mut output: *mut c_char = ptr::null_mut();
    let mut output_ptr: *mut *mut c_char = ptr::null_mut();
    let current_state = State.get();
    let mut forward_output = true;

    // Terminating signals are ignored while the child runs.
    signal_reject_deadly();

    // SAFETY: the caller's contract; `input`, `output` and `nread` are locals
    // whose addresses outlive the call.
    let exitcode = unsafe {
        if opts.has(ShellOpts::HIDE_MESS | ShellOpts::EXPAND) {
            forward_output = false;
        } else {
            State.set(MODE_EXTERNCMD);
            if opts.has(ShellOpts::WRITE) {
                read_input(&raw mut input);
            }
            if opts.has(ShellOpts::READ) {
                output_ptr = &raw mut output;
                forward_output = false;
            } else if opts.has(ShellOpts::DO_OUT) {
                // The caller has already redirected the output.
                forward_output = false;
            }
        }

        let mut nread: size_t = 0;
        let exitcode = system::do_os_system(
            shell_build_argv(cmd, extra_args),
            input.items,
            input.size,
            output_ptr,
            &raw mut nread,
            emsg_silent.get() != 0,
            forward_output,
        );
        xfree(input.items.cast());

        if !output.is_null() {
            write_output(output, nread, true);
            xfree(output.cast());
        }

        if emsg_silent.get() == 0 && exitcode != 0 && !opts.has(ShellOpts::SILENT) {
            msg_ext_set_kind(c"shell_ret".as_ptr());
            if !ui_has(kUIMessages) {
                msg_putchar(NL);
            }
            msg_puts(gettext(c"shell returned ").as_ptr());
            msg_outnum(exitcode);
        }
        exitcode
    };

    State.set(current_state);
    signal_accept_deadly();
    exitcode
}

/// [`os_call_shell`] plus the editor-level bookkeeping: `'verbose'`,
/// `:profile`, `v:shell_error`, and invalidating the cached tags.
///
/// # Safety
/// As [`os_call_shell`].
pub unsafe fn call_shell(cmd: *mut c_char, opts: ShellOpts, extra_shell_arg: *mut c_char) -> c_int {
    let mut wait_time: proftime_T = 0;
    // SAFETY: the caller's contract. `smsg` is printf-shaped.
    unsafe {
        if p_verbose.get() > 3 {
            verbose_enter();
            smsg_c!(
                0,
                gettext(c"Executing command: \"%s\"").as_ptr(),
                if cmd.is_null() { p_sh.get() } else { cmd },
            );
            msg_putchar(NL);
            verbose_leave();
        }

        if do_profiling.get() == PROF_YES {
            wait_time = prof_child_enter();
        }

        let retval = if *p_sh.get() == NUL as c_char {
            emsg(gettext(e_shellempty));
            -1
        } else {
            // The command may have updated a tags file.
            tag_freematch();
            os_call_shell(cmd, opts, extra_shell_arg)
        };

        set_vim_var_nr(Vv::ShellError, retval as varnumber_T);
        if do_profiling.get() == PROF_YES {
            prof_child_exit(wait_time);
        }
        retval
    }
}

/// The stdout of an external command, newly allocated, or NULL on error.
///
/// When `ret_len` is NULL every NUL byte in the output becomes SOH, because
/// the answer is used as a C string; when it is not NULL the length goes
/// there and the bytes come back untouched.
///
/// # Safety
/// `cmd` and `infile` must be NUL-terminated strings or NULL, and `ret_len`
/// writable or NULL.
pub unsafe fn get_cmd_output(
    cmd: *mut c_char,
    infile: *mut c_char,
    flags: ShellOpts,
    ret_len: *mut size_t,
) -> *mut c_char {
    // SAFETY: the caller's contract; `tempname` and `command` are owned here
    // and freed on every path out.
    unsafe {
        if check_secure() {
            return ptr::null_mut();
        }
        let tempname = vim_tempname();
        if tempname.is_null() {
            emsg(gettext(e_notmp));
            return ptr::null_mut();
        }

        // Add the redirection, and run it. Errors are ignored, and timestamps
        // deliberately not checked.
        let command = make_filter_cmd(cmd, infile, tempname, false);
        no_check_timestamps.set(no_check_timestamps.get() + 1);
        call_shell(
            command,
            ShellOpts::DO_OUT | ShellOpts::EXPAND | flags,
            ptr::null_mut(),
        );
        no_check_timestamps.set(no_check_timestamps.get() - 1);
        xfree(command.cast());

        let buffer = read_output(tempname, ret_len);
        xfree(tempname.cast());
        buffer
    }
}

/// Read back what [`get_cmd_output`]'s command redirected into `tempname`.
///
/// # Safety
/// `tempname` must be a NUL-terminated path and `ret_len` writable or NULL.
unsafe fn read_output(tempname: *mut c_char, ret_len: *mut size_t) -> *mut c_char {
    // SAFETY: the caller's contract; `fd` is closed on every path out.
    unsafe {
        // Not being able to seek means the file cannot be read.
        let fd = os_fopen(tempname, READBIN.as_ptr());
        if fd.is_null() || fseek(fd, 0, SEEK_END) == -1 {
            semsg_c!(gettext(e_cannot_read_from_str_2), tempname,);
            if !fd.is_null() {
                fclose(fd);
            }
            return ptr::null_mut();
        }
        let len_l = ftell(fd);
        if len_l == -1 || fseek(fd, 0, SEEK_SET) == -1 {
            semsg_c!(gettext(e_cannot_read_from_str_2), tempname,);
            fclose(fd);
            return ptr::null_mut();
        }

        let len = len_l as usize;
        let buffer = xmalloc(len + 1) as *mut c_char;
        let read = fread(buffer.cast(), 1, len, fd);
        fclose(fd);
        os_remove(tempname);
        if read as usize != len {
            semsg_c!(gettext(e_cant_read_file_str), tempname);
            xfree(buffer.cast());
            return ptr::null_mut();
        }
        if ret_len.is_null() {
            // A NUL would truncate the string, so it becomes SOH instead.
            let bytes = core::slice::from_raw_parts_mut(buffer.cast::<u8>(), len);
            for b in bytes.iter_mut() {
                if *b == 0 {
                    *b = 1;
                }
            }
            *buffer.add(len) = 0;
        } else {
            *ret_len = len;
        }
        buffer
    }
}

/// Split a command string into words, respecting double quotes. `argv` may be
/// NULL to count without copying. Answers the number of words.
///
/// # Safety
/// `str` must be a NUL-terminated string, and `argv` NULL or writable for as
/// many words as it holds.
unsafe fn tokenize(str: *const c_char, argv: *mut *mut c_char) -> usize {
    // SAFETY: the caller's contract.
    unsafe {
        let mut argc = 0;
        let mut p = str;
        while *p != 0 {
            let len = word_length(p);
            if !argv.is_null() {
                *argv.add(argc) = vim_strnsave_unquoted(p, len);
            }
            argc += 1;
            p = skipwhite(p.add(len).cast_mut());
        }
        argc
    }
}

/// How long the shell word starting at `str` is.
///
/// # Safety
/// `str` must be a NUL-terminated string.
unsafe fn word_length(str: *const c_char) -> usize {
    // SAFETY: the caller's contract; the walk stops at the NUL.
    let bytes = unsafe { CStr::from_ptr(str) }.to_bytes();
    let mut inquote = false;
    let mut at = 0;
    // Advance while inside a quote, or on a non-whitespace character.
    while at < bytes.len() && (inquote || (bytes[at] != b' ' && bytes[at] != TAB as u8)) {
        if bytes[at] == b'"' {
            inquote = !inquote;
        } else if bytes[at] == b'\\' && inquote {
            // The escaped byte is part of the word whatever it is.
            at += 1;
        }
        at += 1;
    }
    at.min(bytes.len())
}

/// Copy the range `:!` was given into `buf`, before the event loop starts.
///
/// The whole text is copied up front rather than written in `ml_get` chunks,
/// because reading from the child can trigger a change to the very buffer
/// still being written from.
///
/// # Safety
/// `buf` must point at a live [`StringBuilder`].
unsafe fn read_input(buf: *mut StringBuilder) {
    // SAFETY: the caller's contract; `curbuf` is always live.
    unsafe {
        read_buffer_into(
            Buf::current(),
            (*curbuf.get()).b_op_start.lnum,
            (*curbuf.get()).b_op_end.lnum,
            buf,
        );
    }
}

/// Append the child's output to the current buffer, one line per newline.
///
/// Answers how many bytes were consumed: a trailing partial line is left for
/// the next call unless `eof`.
///
/// # Safety
/// `output` must be writable for `remaining` bytes — the line ends are
/// overwritten with NULs in place, which is what lets `ml_append` take them
/// as C strings.
unsafe fn write_output(output: *mut c_char, remaining: size_t, eof: bool) -> size_t {
    if output.is_null() {
        return 0;
    }
    // SAFETY: the caller's contract; `off` never passes `remaining`, and the
    // one-past read below lands inside it because a CR at the very end
    // fails the `off + 1 < remaining` guard first.
    unsafe {
        let start = output;
        let mut output = output;
        let mut remaining = remaining;
        let mut off: usize = 0;
        while off < remaining {
            let binary = (*curbuf.get()).b_p_bin != 0;
            let byte = *output.add(off) as c_int;
            // CRLF, except in binary mode, where the CR is kept.
            let skip = if byte == CAR
                && off + 1 < remaining
                && *output.add(off + 1) as c_int == NL
                && !binary
            {
                off + 2
            } else if (byte == CAR && !binary) || byte == NL {
                off + 1
            } else {
                if byte == NUL {
                    // A NUL becomes a newline.
                    *output.add(off) = NL as c_char;
                }
                off += 1;
                continue;
            };
            *output.add(off) = 0;
            let lnum = (*curwin.get()).w_cursor.lnum;
            (*curwin.get()).w_cursor.lnum += 1;
            ml_append(lnum, output, off as c_int + 1, false);
            output = output.add(skip);
            remaining -= skip;
            off = 0;
        }

        if eof {
            if remaining != 0 {
                // An unfinished last line, and a note that its ending was
                // missing.
                let lnum = (*curwin.get()).w_cursor.lnum;
                (*curwin.get()).w_cursor.lnum += 1;
                ml_append(lnum, output, 0, false);
                (*curbuf.get()).b_no_eol_lnum = (*curwin.get()).w_cursor.lnum;
                output = output.add(remaining);
            } else {
                (*curbuf.get()).b_no_eol_lnum = 0 as linenr_T;
            }
        }

        ui_flush();
        output.offset_from(start) as size_t
    }
}

/// Apply `'shellxescape'` and `'shellxquote'` to a command.
///
/// # Safety
/// `cmd` must be a NUL-terminated string.
unsafe fn shell_xescape_xquote(cmd: *const c_char) -> *mut c_char {
    // SAFETY: the caller's contract; `p_sxq`/`p_sxe` are option values, and
    // `ecmd` is only freed when `vim_strsave_escaped_ext` allocated it.
    unsafe {
        if *p_sxq.get() == NUL as c_char {
            return xstrdup(cmd);
        }

        let mut ecmd = cmd;
        if *p_sxe.get() != NUL as c_char && strcmp(p_sxq.get(), c"(".as_ptr()) == 0 {
            ecmd = vim_strsave_escaped_ext(cmd, p_sxe.get(), '^' as c_char, false);
        }
        let ncmd_size = strlen(ecmd) + strlen(p_sxq.get()) * 2 + 1;
        let ncmd = xmalloc(ncmd_size) as *mut c_char;

        // 'shellxquote' of "(" appends ")", of "\"(" appends ")\"".
        if strcmp(p_sxq.get(), c"(".as_ptr()) == 0 {
            vim_snprintf(ncmd, ncmd_size, c"(%s)".as_ptr(), ecmd);
        } else if strcmp(p_sxq.get(), c"\"(".as_ptr()) == 0 {
            vim_snprintf(ncmd, ncmd_size, c"\"(%s)\"".as_ptr(), ecmd);
        } else {
            vim_snprintf(
                ncmd,
                ncmd_size,
                c"%s%s%s".as_ptr(),
                p_sxq.get(),
                ecmd,
                p_sxq.get(),
            );
        }

        if ecmd != cmd {
            xfree(ecmd.cast_mut().cast());
        }
        ncmd
    }
}
