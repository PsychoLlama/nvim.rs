//! Running one diff, internal or external.
//!
//! [`diff_file`] picks between the three: `'diffexpr'`, the external
//! `diff(1)` and the built-in xdiff engine.  [`check_external_diff`] is the
//! probe that decides whether the host's `diff` is usable at all (and caches
//! the answer in `diff_a_works`); [`diff_file_internal`] runs the engine
//! over two memory images and collects its hunks.

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
use crate::cstr;
use crate::option::vars::P_DEX;
use crate::types::Failed;
use crate::xdiff::ffi::diff_hunks;
use crate::xdiff::xtypes::Aborted;
use core::ffi::{c_char, c_int};

/// Diff two one-line files and see whether the answer is recognisable.
///
/// Answers `Ok` if the host has a usable `diff`, and leaves `diff_a_works`
/// saying whether it accepts `-a`.  The probe runs at most twice: the first
/// attempt passes `-a`, and if that produces nothing recognisable the flag is
/// remembered as unsupported and the whole thing is tried again without it.
pub(crate) fn check_external_diff(names: TempNames<'_>) -> Result<(), Failed> {
    let [orig, new, out] = names;
    let mut io_error = false;
    let mut ok;
    loop {
        ok = false;
        let mut fd = unsafe { os_fopen(orig.as_ptr(), c"w".as_ptr()) };
        if fd.is_null() {
            io_error = true;
        } else {
            if unsafe { fwrite(c"line1\n".as_ptr().cast(), 6, 1, fd) } != 1 {
                io_error = true;
            }
            unsafe { fclose(fd) };
            fd = unsafe { os_fopen(new.as_ptr(), c"w".as_ptr()) };
            if fd.is_null() {
                io_error = true;
            } else {
                if unsafe { fwrite(c"line2\n".as_ptr().cast(), 6, 1, fd) } != 1 {
                    io_error = true;
                }
                unsafe { fclose(fd) };
                fd = if diff_file_external(names).is_ok() {
                    unsafe { os_fopen(out.as_ptr(), c"r".as_ptr()) }
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
                        if unsafe { cstr::starts_with(linebuf.as_ptr(), b"1c1") }
                            || unsafe { cstr::starts_with(linebuf.as_ptr(), b"@@ -1 +1 @@") }
                        {
                            ok = true;
                        }
                    }
                    unsafe { fclose(fd) };
                }
                os_remove(out);
                os_remove(new);
            }
            os_remove(orig);
        }
        // With `'diffexpr'` set there is no `-a` to retry without.
        if P_DEX.first_byte() != 0 || diff_a_works.get().is_some() {
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

/// Diff the two memory images with the built-in engine, appending the hunks
/// to `hunks`.
pub(crate) fn diff_file_internal(
    orig: &[u8],
    new: &[u8],
    hunks: &mut Vec<DiffHunk>,
) -> Result<(), Failed> {
    let flags = diff_flags.get();
    // `'diffopt'`'s ignore flags map onto xdiff's own; `icase` does not, and
    // is applied while writing the buffers out instead. `'diffanchors'` is
    // implemented by splitting the buffers, not by xdiff's anchor list.
    let xdiff_flags = diff_algorithm.get()
        | [
            (DIFF_IWHITE, XDF_IGNORE_WHITESPACE_CHANGE),
            (DIFF_IWHITEALL, XDF_IGNORE_WHITESPACE),
            (DIFF_IWHITEEOL, XDF_IGNORE_WHITESPACE_AT_EOL),
            (DIFF_IBLANK, XDF_IGNORE_BLANK_LINES),
        ]
        .into_iter()
        .filter(|&(bit, _)| flags & bit != 0)
        .fold(0, |all, (_, xdf)| all | xdf);
    let too_big = |text: &[u8]| i64::try_from(text.len()).map_or(true, |n| n > MAX_XDIFF_SIZE);
    // Each hunk comes back with zero-based starts; the list wants line
    // numbers.
    let mut push = |start_a: i64, count_a: i64, start_b: i64, count_b: i64| {
        let int = |n: i64| c_int::try_from(n).map_err(|_| Aborted);
        hunks.push(DiffHunk {
            lnum_orig: int(start_a + 1)?,
            count_orig: int(count_a)?,
            lnum_new: int(start_b + 1)?,
            count_new: int(count_b)?,
        });
        Ok(())
    };
    if too_big(orig) || too_big(new) || diff_hunks(orig, new, xdiff_flags, &mut push).is_err() {
        emsg(gettext(e_problem_creating_internal_diff));
        return Err(Failed);
    }
    Ok(())
}

/// Diff whichever way `'diffopt'` and `'diffexpr'` say.
///
/// A run that has no temp files is diffed internally whatever `'diffexpr'`
/// says by now: upstream would hand `'diffexpr'` three null names.
pub(crate) fn diff_file(dio: &mut DiffIo) -> Result<(), Failed> {
    match dio.temp_names() {
        Some(names) if !dio.internal || P_DEX.first_byte() != 0 => diff_file_external(names),
        _ => diff_file_internal(&dio.orig.text, &dio.new.text, &mut dio.diff.hunks),
    }
}

/// Diff the first two temp files into the third, through `'diffexpr'` or
/// the host's `diff`.
fn diff_file_external(names: TempNames<'_>) -> Result<(), Failed> {
    let [tmp_orig, tmp_new, tmp_diff] = names;
    if P_DEX.first_byte() != 0 {
        eval_diff(tmp_orig, tmp_new, tmp_diff);
        return Ok(());
    }

    // The user's own `diff` options would corrupt the output format.
    if unsafe { os_env_exists(c"DIFF_OPTIONS".as_ptr(), true) } {
        unsafe { os_unsetenv(c"DIFF_OPTIONS".as_ptr()) };
    }

    // `diff`, its flags, the two input files and the output redirection.
    // Upstream spells this as one `vim_snprintf` of nine `%s`, seven of them
    // a flag or the empty string.
    let mut cmd: Vec<u8> = b"diff ".to_vec();
    let flags = diff_flags.get();
    for (on, text) in [
        (diff_a_works.get() != Some(false), &b"-a "[..]),
        (flags & DIFF_IWHITE != 0, b"-b "),
        (flags & DIFF_IWHITEALL != 0, b"-w "),
        (flags & DIFF_IWHITEEOL != 0, b"-Z "),
        (flags & DIFF_IBLANK != 0, b"-B "),
        (flags & DIFF_ICASE != 0, b"-i "),
    ] {
        if on {
            cmd.extend_from_slice(text);
        }
    }
    cmd.extend_from_slice(tmp_orig.to_bytes());
    cmd.push(b' ');
    cmd.extend_from_slice(tmp_new.to_bytes());
    p_srr(|srr| append_redir(&mut cmd, srr, tmp_diff));
    let cmd = cstr::owned(&cmd);

    block_autocmds();
    unsafe {
        call_shell(
            cmd.as_ptr().cast_mut(),
            ShellOpts::FILTER | ShellOpts::SILENT | ShellOpts::DO_OUT,
            ::core::ptr::null_mut(),
        )
    };
    unblock_autocmds();
    Ok(())
}
