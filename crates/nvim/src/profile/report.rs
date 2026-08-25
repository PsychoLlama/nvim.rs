//! The `:profile` report: what `:profile dump` and `:profile stop` write to
//! the file `:profile start` named.
//!
//! Two sections, in this order: every profiled script's source annotated
//! line by line ([`script_dump_profile`]), then every profiled function the
//! same way ([`func_dump_profile`]), followed by the two top-20 lists sorted
//! on total and on self time. The column layout is upstream's to the space:
//! `prof_func_line` is the shared five-count/two-time prefix, and the rule
//! that a time equal to the other one prints as blanks is what makes the
//! report readable.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{
    NL, PROFILE_FNAME, func_line, prl_item, profile_cmp, profile_msg_str, profiled_functions,
};
use crate::fileio::vim_fgets;
use crate::keycodes::K_SPECIAL;
use crate::memory::xfree;
use crate::os::fs::os_fopen;
use crate::runtime::{get_scriptname, script_count, script_item};
use crate::types::{IOSIZE, proftime_T, scriptitem_T, ufunc_T};
use ::libc::fclose;
use core::ffi::{CStr, c_char, c_int, c_void};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::os::unix::ffi::OsStrExt;

// ---------------------------------------------------------------------------
// The report.

/// Write the profiling report to the `:profile start` file, if set.
pub fn profile_dump() {
    PROFILE_FNAME.with(|fname| {
        let Some(fname) = fname else { return };
        match File::create(OsStr::from_bytes(fname.to_bytes())) {
            Ok(file) => {
                let mut fd = BufWriter::new(file);
                // Like the C fprintf-based writer, I/O errors are ignored.
                // SAFETY: main thread; the tables the dump walks are live.
                let _ = unsafe { script_dump_profile(&mut fd) };
                let _ = unsafe { func_dump_profile(&mut fd) };
            }
            Err(_) => {
                crate::semsg!("E484: Can't open file {}", fname.to_string_lossy());
            }
        }
    });
}

/// `"name()"` with a newline, decoding the `<SNR>` mangling.
///
/// # Safety
/// `fp` is a live function-table entry.
unsafe fn write_func_name(fd: &mut dyn Write, fp: *mut ufunc_T) -> io::Result<()> {
    // SAFETY: `uf_name` is the flexible NUL-terminated name at the end of the
    // entry, alive for as long as `fp` is.
    let name = unsafe { CStr::from_ptr(&raw const (*fp).uf_name as *const c_char).to_bytes() };
    if name.first().copied() == Some(K_SPECIAL as u8) {
        write!(fd, "<SNR>")?;
        fd.write_all(name.get(3..).unwrap_or_default())?;
    } else {
        fd.write_all(name)?;
    }
    writeln!(fd, "()")
}

/// One count/total/self report line. With `prefer_self` (function lines),
/// equal totals print only the self time; otherwise only the total.
fn prof_func_line(
    fd: &mut dyn Write,
    count: c_int,
    total: proftime_T,
    self_: proftime_T,
    prefer_self: bool,
) -> io::Result<()> {
    if count > 0 {
        write!(fd, "{count:5} ")?;
        if prefer_self && total == self_ {
            write!(fd, "           ")?;
        } else {
            write!(fd, "{} ", profile_msg_str(total))?;
        }
        if !prefer_self && total == self_ {
            write!(fd, "           ")?;
        } else {
            write!(fd, "{} ", profile_msg_str(self_))?;
        }
    } else {
        write!(fd, "                            ")?;
    }
    Ok(())
}

/// The top-20 list sorted on total or self time.
///
/// # Safety
/// `sorttab` holds live function-table entries.
unsafe fn prof_sort_list(
    fd: &mut dyn Write,
    sorttab: &[*mut ufunc_T],
    title: &str,
    prefer_self: bool,
) -> io::Result<()> {
    writeln!(fd, "FUNCTIONS SORTED ON {title} TIME")?;
    writeln!(fd, "count  total (s)   self (s)  function")?;
    for &fp in sorttab.iter().take(20) {
        // SAFETY: the caller's entries.
        let f = unsafe { &*fp };
        prof_func_line(fd, f.uf_tm_count, f.uf_tm_total, f.uf_tm_self, prefer_self)?;
        write!(fd, " ")?;
        // SAFETY: as above.
        unsafe { write_func_name(fd, fp) }?;
    }
    writeln!(fd)
}

/// Where a function was defined, as the report's `Defined:` line.
///
/// # Safety
/// `fp` is a live function-table entry with a non-zero `uf_script_ctx`.
unsafe fn write_func_origin(fd: &mut dyn Write, fp: &ufunc_T) -> io::Result<()> {
    let mut should_free = false;
    // SAFETY: `get_scriptname` answers with a NUL-terminated name, owned by
    // the caller exactly when it says so.
    unsafe {
        let p = get_scriptname(fp.uf_script_ctx, &raw mut should_free);
        write!(fd, "    Defined: ")?;
        fd.write_all(CStr::from_ptr(p).to_bytes())?;
        writeln!(fd, ":{}", fp.uf_script_ctx.sc_lnum)?;
        if should_free {
            xfree(p as *mut c_void);
        }
    }
    Ok(())
}

/// Per-function sections plus the sorted lists.
///
/// # Safety
/// Main-thread editor call; the function table is live.
unsafe fn func_dump_profile(fd: &mut dyn Write) -> io::Result<()> {
    // SAFETY: the caller's contract.
    let mut sorttab = unsafe { profiled_functions() };
    for &fp in &sorttab {
        // SAFETY: an entry of the function table.
        let f = unsafe { &*fp };
        write!(fd, "FUNCTION  ")?;
        // SAFETY: as above.
        unsafe { write_func_name(fd, fp) }?;
        if f.uf_script_ctx.sc_sid != 0 {
            // SAFETY: as above.
            unsafe { write_func_origin(fd, f) }?;
        }
        if f.uf_tm_count == 1 {
            writeln!(fd, "Called 1 time")?;
        } else {
            writeln!(fd, "Called {} times", f.uf_tm_count)?;
        }
        writeln!(fd, "Total time: {}", profile_msg_str(f.uf_tm_total))?;
        writeln!(fd, " Self time: {}", profile_msg_str(f.uf_tm_self))?;
        write!(fd, "\ncount  total (s)   self (s)\n")?;
        for i in 0..f.uf_lines.ga_len as isize {
            // SAFETY: `i` is below `uf_lines.ga_len`.
            let line = unsafe { func_line(f, i) };
            if line.is_null() {
                continue;
            }
            // SAFETY: the three per-line counters are sized to `uf_lines`.
            let (count, total, self_) = unsafe {
                (
                    *f.uf_tml_count.offset(i),
                    *f.uf_tml_total.offset(i),
                    *f.uf_tml_self.offset(i),
                )
            };
            prof_func_line(fd, count, total, self_, true)?;
            // SAFETY: a NUL-terminated source line owned by the function.
            fd.write_all(unsafe { CStr::from_ptr(line) }.to_bytes())?;
            writeln!(fd)?;
        }
        writeln!(fd)?;
    }
    if !sorttab.is_empty() {
        // SAFETY: the entries this walk collected.
        unsafe {
            sorttab.sort_by(|&a, &b| profile_cmp((*a).uf_tm_total, (*b).uf_tm_total).cmp(&0));
            prof_sort_list(fd, &sorttab, "TOTAL", false)?;
            sorttab.sort_by(|&a, &b| profile_cmp((*a).uf_tm_self, (*b).uf_tm_self).cmp(&0));
            prof_sort_list(fd, &sorttab, "SELF", true)?;
        }
    }
    Ok(())
}

/// One script's source, annotated line by line with its counters. The read
/// runs to the end of file so that trailing continuation lines are listed.
///
/// # Safety
/// `si` is a live script item whose `sn_name` names the script's source.
unsafe fn script_dump_source(fd: &mut dyn Write, si: &scriptitem_T) -> io::Result<()> {
    // SAFETY: `sn_name` is the NUL-terminated source path.
    let sfd = unsafe { os_fopen(si.sn_name, c"r".as_ptr()) };
    if sfd.is_null() {
        return writeln!(fd, "Cannot open file!");
    }
    let mut buf = [0 as c_char; IOSIZE as usize];
    let mut i = 0;
    // SAFETY: `buf` is `IOSIZE` chars, which is the bound handed over, and
    // `sfd` is the handle just opened; it is closed below.
    while !unsafe { vim_fgets(buf.as_mut_ptr(), IOSIZE, sfd) } {
        // When a line has been truncated, append NL, taking care of
        // multibyte characters.
        if buf[IOSIZE as usize - 2] != 0 && buf[IOSIZE as usize - 2] != NL {
            let mut n = IOSIZE as usize - 2;
            // Move back to the first byte of the char.
            while n > 0 && (buf[n] as u8 & 0xc0) == 0x80 {
                n -= 1;
            }
            buf[n] = NL;
            buf[n + 1] = 0;
        }
        // SAFETY: `i` is below `ga_len`, so the counters exist; `buf` was
        // NUL-terminated by `vim_fgets`.
        let (counters, line) = unsafe {
            (
                (i < si.sn_prl_ga.ga_len).then(|| *prl_item(si, i as isize)),
                CStr::from_ptr(buf.as_ptr()),
            )
        };
        match counters.filter(|pp| pp.snp_count > 0) {
            Some(pp) => {
                write!(fd, "{:5} ", pp.snp_count)?;
                if pp.sn_prl_total == pp.sn_prl_self {
                    write!(fd, "           ")?;
                } else {
                    write!(fd, "{} ", profile_msg_str(pp.sn_prl_total))?;
                }
                write!(fd, "{} ", profile_msg_str(pp.sn_prl_self))?;
            }
            None => write!(fd, "                            ")?,
        }
        fd.write_all(line.to_bytes())?;
        i += 1;
    }
    // SAFETY: the handle opened above, used nowhere else.
    unsafe { fclose(sfd) };
    Ok(())
}

/// Per-script sections: each profiled script's source lines annotated with
/// their counters.
///
/// # Safety
/// Main-thread editor call; the script table is live.
unsafe fn script_dump_profile(fd: &mut dyn Write) -> io::Result<()> {
    for id in 1..=script_count() {
        // SAFETY: `1..=ga_len` are the live script ids.
        let si = unsafe { &*script_item(id) };
        if !si.sn_prof_on {
            continue;
        }
        write!(fd, "SCRIPT  ")?;
        // SAFETY: `sn_name` is the NUL-terminated source path.
        fd.write_all(unsafe { CStr::from_ptr(si.sn_name) }.to_bytes())?;
        writeln!(fd)?;
        if si.sn_pr_count == 1 {
            writeln!(fd, "Sourced 1 time")?;
        } else {
            writeln!(fd, "Sourced {} times", si.sn_pr_count)?;
        }
        writeln!(fd, "Total time: {}", profile_msg_str(si.sn_pr_total))?;
        writeln!(fd, " Self time: {}", profile_msg_str(si.sn_pr_self))?;
        write!(fd, "\ncount  total (s)   self (s)\n")?;
        // SAFETY: as above.
        unsafe { script_dump_source(fd, si) }?;
        writeln!(fd)?;
    }
    Ok(())
}
