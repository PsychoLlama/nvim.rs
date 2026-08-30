//! Running one diff, internal or external.
//!
//! [`diff_file`] picks between the three: `'diffexpr'`, the external
//! `diff(1)` and the built-in `xdl_diff`.  [`check_external_diff`] is the
//! probe that decides whether the host's `diff` is usable at all (and caches
//! the answer in `diff_a_works`); [`diff_file_internal`] is the `xdl_diff`
//! call, with [`xdiff_out`] as its hunk callback.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::Failed;
use core::ffi::{c_char, c_int};
use std::ffi::CStr;

/// Diff two one-line files and see whether the answer is recognisable.
///
/// Answers `OK` if the host has a usable `diff`, and leaves `diff_a_works`
/// saying whether it accepts `-a`.  The probe runs at most twice: the first
/// attempt passes `-a`, and if that produces nothing recognisable the flag is
/// remembered as unsupported and the whole thing is tried again without it.
pub(crate) unsafe fn check_external_diff(diffio: *mut diffio_T) -> Result<(), Failed> {
    let orig = unsafe { (*diffio).dio_orig.din_fname };
    let new = unsafe { (*diffio).dio_new.din_fname };
    let out = unsafe { (*diffio).dio_diff.dout_fname };
    let mut io_error = false;
    let mut ok = false;
    loop {
        ok = false;
        let mut fd = unsafe { os_fopen(orig, c"w".as_ptr()) };
        if fd.is_null() {
            io_error = true;
        } else {
            if unsafe { fwrite(c"line1\n".as_ptr().cast(), 6, 1, fd) } != 1 {
                io_error = true;
            }
            unsafe { fclose(fd) };
            fd = unsafe { os_fopen(new, c"w".as_ptr()) };
            if fd.is_null() {
                io_error = true;
            } else {
                if unsafe { fwrite(c"line2\n".as_ptr().cast(), 6, 1, fd) } != 1 {
                    io_error = true;
                }
                unsafe { fclose(fd) };
                fd = if unsafe { diff_file(diffio) }.is_ok() {
                    unsafe { os_fopen(out, c"r".as_ptr()) }
                } else {
                    ::core::ptr::null_mut()
                };
                if fd.is_null() {
                    io_error = true;
                } else {
                    // The two spellings a working `diff` can answer with:
                    // ed-style and unified.
                    let mut linebuf = [0 as c_char; LBUFLEN as usize];
                    while !unsafe { vim_fgets(linebuf.as_mut_ptr(), LBUFLEN, fd) } {
                        if unsafe { strncmp(linebuf.as_ptr(), c"1c1".as_ptr(), 3) } == 0
                            || unsafe { strncmp(linebuf.as_ptr(), c"@@ -1 +1 @@".as_ptr(), 11) }
                                == 0
                        {
                            ok = true;
                        }
                    }
                    unsafe { fclose(fd) };
                }
                unsafe { os_remove(out) };
                unsafe { os_remove(new) };
            }
            unsafe { os_remove(orig) };
        }
        // With `'diffexpr'` set there is no `-a` to retry without.
        if unsafe { *p_dex.get() } != 0 || diff_a_works.get().is_some() {
            break;
        }
        diff_a_works.set(Some(ok));
        if ok {
            break;
        }
    }
    if ok {
        return Ok(());
    }
    if io_error {
        emsg(gettext(c"E810: Cannot read or write temp files"));
    }
    emsg(gettext(c"E97: Cannot create diffs"));
    diff_a_works.set(None);
    Err(Failed)
}

/// Diff the two memory images with `xdl_diff`, collecting hunks into
/// `dio_diff.dout_ga`.
pub(crate) unsafe fn diff_file_internal(diffio: *mut diffio_T) -> Result<(), Failed> {
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
        priv_0: unsafe { &raw mut (*diffio).dio_diff }.cast(),
        out_hunk: None,
        out_line: None,
    };

    if unsafe { (*diffio).dio_orig.din_mmfile.size } as ::core::ffi::c_long > MAX_XDIFF_SIZE
        || unsafe { (*diffio).dio_new.din_mmfile.size } as ::core::ffi::c_long > MAX_XDIFF_SIZE
        || unsafe {
            xdl_diff(
                &raw mut (*diffio).dio_orig.din_mmfile,
                &raw mut (*diffio).dio_new.din_mmfile,
                &raw mut param,
                &raw mut emit_cfg,
                &raw mut emit_cb,
            )
        } < 0
    {
        emsg(gettext(e_problem_creating_internal_diff));
        return Err(Failed);
    }
    Ok(())
}

/// Diff whichever way `'diffopt'` and `'diffexpr'` say.
pub(crate) unsafe fn diff_file(dio: *mut diffio_T) -> Result<(), Failed> {
    let tmp_orig = unsafe { (*dio).dio_orig.din_fname };
    let tmp_new = unsafe { (*dio).dio_new.din_fname };
    let tmp_diff = unsafe { (*dio).dio_diff.dout_fname };
    if unsafe { *p_dex.get() } != 0 {
        unsafe { eval_diff(tmp_orig, tmp_new, tmp_diff) };
        return Ok(());
    }
    if unsafe { (*dio).dio_internal } != 0 {
        return unsafe { diff_file_internal(dio) };
    }

    // "diff " plus six two-character flags, three file names, the redirect
    // and the terminator.
    let len = unsafe { strlen(tmp_orig) }
        + unsafe { strlen(tmp_new) }
        + unsafe { strlen(tmp_diff) }
        + unsafe { strlen(p_srr.get()) }
        + 27;
    let cmd = unsafe { xmalloc(len) } as *mut c_char;
    // The user's own `diff` options would corrupt the output format.
    if unsafe { os_env_exists(c"DIFF_OPTIONS".as_ptr(), true) } {
        unsafe { os_unsetenv(c"DIFF_OPTIONS".as_ptr()) };
    }
    let flag = |on: bool, text: &'static CStr| {
        if on { text.as_ptr() } else { c"".as_ptr() }
    };
    unsafe {
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
        )
    };
    unsafe { append_redir(cmd, len, p_srr.get(), tmp_diff) };
    unsafe { block_autocmds() };
    unsafe {
        call_shell(
            cmd,
            ShellOpts::FILTER | ShellOpts::SILENT | ShellOpts::DO_OUT,
            ::core::ptr::null_mut(),
        )
    };
    unsafe { unblock_autocmds() };
    unsafe { xfree(cmd.cast()) };
    Ok(())
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
    let dout = priv_0 as *mut diffout_T;
    unsafe { ga_grow(&raw mut (*dout).dout_ga, 1) };
    unsafe {
        *((*dout).dout_ga.ga_data as *mut diffhunk_T).offset((*dout).dout_ga.ga_len as isize) =
            diffhunk_T {
                lnum_orig: start_a + 1,
                count_orig: count_a,
                lnum_new: start_b + 1,
                count_new: count_b,
            }
    };
    unsafe { (*dout).dout_ga.ga_len += 1 };
    0
}
