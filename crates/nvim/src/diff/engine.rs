//! Running one diff, internal or external.
//!
//! [`diff_file`] picks between the three: `'diffexpr'`, the external
//! `diff(1)` and the built-in `xdl_diff`.  [`check_external_diff`] is the
//! probe that decides whether the host's `diff` is usable at all (and caches
//! the answer in `diff_a_works`); [`diff_file_internal`] is the `xdl_diff`
//! call, with [`xdiff_out`] as its hunk callback.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::{FAIL, OK};
use core::ffi::{c_char, c_int};
use std::ffi::CStr;

/// Diff two one-line files and see whether the answer is recognisable.
///
/// Answers `OK` if the host has a usable `diff`, and leaves `diff_a_works`
/// saying whether it accepts `-a`.  The probe runs at most twice: the first
/// attempt passes `-a`, and if that produces nothing recognisable the flag is
/// remembered as unsupported and the whole thing is tried again without it.
pub(crate) unsafe fn check_external_diff(diffio: *mut diffio_T) -> c_int {
    unsafe {
        let orig = (*diffio).dio_orig.din_fname;
        let new = (*diffio).dio_new.din_fname;
        let out = (*diffio).dio_diff.dout_fname;
        let mut io_error = false;
        let mut ok = false;
        loop {
            ok = false;
            let mut fd = os_fopen(orig, c"w".as_ptr());
            if fd.is_null() {
                io_error = true;
            } else {
                if fwrite(c"line1\n".as_ptr().cast(), 6, 1, fd) != 1 {
                    io_error = true;
                }
                fclose(fd);
                fd = os_fopen(new, c"w".as_ptr());
                if fd.is_null() {
                    io_error = true;
                } else {
                    if fwrite(c"line2\n".as_ptr().cast(), 6, 1, fd) != 1 {
                        io_error = true;
                    }
                    fclose(fd);
                    fd = if diff_file(diffio) == OK {
                        os_fopen(out, c"r".as_ptr())
                    } else {
                        ::core::ptr::null_mut()
                    };
                    if fd.is_null() {
                        io_error = true;
                    } else {
                        // The two spellings a working `diff` can answer with:
                        // ed-style and unified.
                        let mut linebuf = [0 as c_char; LBUFLEN as usize];
                        while !vim_fgets(linebuf.as_mut_ptr(), LBUFLEN, fd) {
                            if strncmp(linebuf.as_ptr(), c"1c1".as_ptr(), 3) == 0
                                || strncmp(linebuf.as_ptr(), c"@@ -1 +1 @@".as_ptr(), 11) == 0
                            {
                                ok = true;
                            }
                        }
                        fclose(fd);
                    }
                    os_remove(out);
                    os_remove(new);
                }
                os_remove(orig);
            }
            // With `'diffexpr'` set there is no `-a` to retry without.
            if *p_dex.get() != 0 || diff_a_works.get().is_some() {
                break;
            }
            diff_a_works.set(Some(ok));
            if ok {
                break;
            }
        }
        if ok {
            return OK;
        }
        if io_error {
            emsg(gettext(c"E810: Cannot read or write temp files".as_ptr()));
        }
        emsg(gettext(c"E97: Cannot create diffs".as_ptr()));
        diff_a_works.set(None);
        FAIL
    }
}

/// Diff the two memory images with `xdl_diff`, collecting hunks into
/// `dio_diff.dout_ga`.
pub(crate) unsafe fn diff_file_internal(diffio: *mut diffio_T) -> c_int {
    unsafe {
        let flags = diff_flags.get();
        let mut param = xpparam_t {
            // `'diffopt'`'s ignore flags map onto xdiff's own; `icase` does
            // not, and is applied while writing the buffers out instead.
            flags: diff_algorithm.get()
                | if flags & DIFF_IWHITE != 0 {
                    XDF_IGNORE_WHITESPACE_CHANGE
                } else {
                    0
                }
                | if flags & DIFF_IWHITEALL != 0 {
                    XDF_IGNORE_WHITESPACE
                } else {
                    0
                }
                | if flags & DIFF_IWHITEEOL != 0 {
                    XDF_IGNORE_WHITESPACE_AT_EOL
                } else {
                    0
                }
                | if flags & DIFF_IBLANK != 0 {
                    XDF_IGNORE_BLANK_LINES
                } else {
                    0
                },
            // `'diffanchors'` is implemented by splitting the buffers, not
            // by xdiff's anchor list.
            anchors: ::core::ptr::null_mut(),
            anchors_nr: 0,
        };
        let mut emit_cfg = xdemitconf_t {
            // No context lines: nvim wants the hunks, not a patch.
            ctxlen: 0,
            interhunkctxlen: 0,
            flags: 0,
            find_func: None,
            find_func_priv: ::core::ptr::null_mut(),
            hunk_func: Some(xdiff_out),
        };
        let mut emit_cb = xdemitcb_t {
            priv_0: (&raw mut (*diffio).dio_diff).cast(),
            out_hunk: None,
            out_line: None,
        };

        if (*diffio).dio_orig.din_mmfile.size as ::core::ffi::c_long > MAX_XDIFF_SIZE
            || (*diffio).dio_new.din_mmfile.size as ::core::ffi::c_long > MAX_XDIFF_SIZE
            || xdl_diff(
                &raw mut (*diffio).dio_orig.din_mmfile,
                &raw mut (*diffio).dio_new.din_mmfile,
                &raw mut param,
                &raw mut emit_cfg,
                &raw mut emit_cb,
            ) < 0
        {
            emsg(gettext(
                &raw const e_problem_creating_internal_diff as *const c_char,
            ));
            return FAIL;
        }
        OK
    }
}

/// Diff whichever way `'diffopt'` and `'diffexpr'` say.
pub(crate) unsafe fn diff_file(dio: *mut diffio_T) -> c_int {
    unsafe {
        let tmp_orig = (*dio).dio_orig.din_fname;
        let tmp_new = (*dio).dio_new.din_fname;
        let tmp_diff = (*dio).dio_diff.dout_fname;
        if *p_dex.get() != 0 {
            eval_diff(tmp_orig, tmp_new, tmp_diff);
            return OK;
        }
        if (*dio).dio_internal != 0 {
            return diff_file_internal(dio);
        }

        // "diff " plus six two-character flags, three file names, the redirect
        // and the terminator.
        let len = strlen(tmp_orig) + strlen(tmp_new) + strlen(tmp_diff) + strlen(p_srr.get()) + 27;
        let cmd = xmalloc(len) as *mut c_char;
        // The user's own `diff` options would corrupt the output format.
        if os_env_exists(c"DIFF_OPTIONS".as_ptr(), true) {
            os_unsetenv(c"DIFF_OPTIONS".as_ptr());
        }
        let flag = |on: bool, text: &'static CStr| {
            if on { text.as_ptr() } else { c"".as_ptr() }
        };
        vim_snprintf(
            cmd,
            len,
            c"diff %s%s%s%s%s%s%s%s %s".as_ptr(),
            flag(diff_a_works.get() != Some(false), c"-a "),
            c"".as_ptr(),
            flag(diff_flags.get() & DIFF_IWHITE != 0, c"-b "),
            flag(diff_flags.get() & DIFF_IWHITEALL != 0, c"-w "),
            flag(diff_flags.get() & DIFF_IWHITEEOL != 0, c"-Z "),
            flag(diff_flags.get() & DIFF_IBLANK != 0, c"-B "),
            flag(diff_flags.get() & DIFF_ICASE != 0, c"-i "),
            tmp_orig,
            tmp_new,
        );
        append_redir(cmd, len, p_srr.get(), tmp_diff);
        block_autocmds();
        call_shell(
            cmd,
            ShellOpts::FILTER | ShellOpts::SILENT | ShellOpts::DO_OUT,
            ::core::ptr::null_mut(),
        );
        unblock_autocmds();
        xfree(cmd.cast());
        OK
    }
}

/// `xdl_diff`'s hunk callback: append one hunk to the `diffout_T` behind
/// `priv_0`, converting xdiff's zero-based starts to line numbers.
unsafe extern "C" fn xdiff_out(
    start_a: c_int,
    count_a: c_int,
    start_b: c_int,
    count_b: c_int,
    priv_0: *mut ::core::ffi::c_void,
) -> c_int {
    unsafe {
        let dout = priv_0 as *mut diffout_T;
        ga_grow(&raw mut (*dout).dout_ga, 1);
        *((*dout).dout_ga.ga_data as *mut diffhunk_T).offset((*dout).dout_ga.ga_len as isize) =
            diffhunk_T {
                lnum_orig: start_a + 1,
                count_orig: count_a,
                lnum_new: start_b + 1,
                count_new: count_b,
            };
        (*dout).dout_ga.ga_len += 1;
        0
    }
}
