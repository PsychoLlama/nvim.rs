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

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::{PROFILE_FNAME, profile_cmp, profile_msg_str, profiled_functions};
use crate::keycodes::K_SPECIAL;
use crate::runtime::{get_scriptname, script_count, with_script_item};
use crate::types::{IOSIZE, ProfTime, ScriptCtx, SnPrl, UserFunc};
use core::ffi::c_int;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::os::unix::ffi::OsStrExt;
use std::rc::Rc;

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
                let _ = script_dump_profile(&mut fd);
                let _ = func_dump_profile(&mut fd);
            }
            Err(_) => {
                crate::semsg!("E484: Can't open file {}", fname.to_string_lossy());
            }
        }
    });
}

/// `"name()"` with a newline, decoding the `<SNR>` mangling.
fn write_func_name(fd: &mut dyn Write, func: &UserFunc) -> io::Result<()> {
    let name = func.name().as_bytes();
    if name.first().is_some_and(|&b| c_int::from(b) == K_SPECIAL) {
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
    total: ProfTime,
    self_: ProfTime,
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
fn prof_sort_list(
    fd: &mut dyn Write,
    sorttab: &[Rc<UserFunc>],
    title: &str,
    prefer_self: bool,
) -> io::Result<()> {
    writeln!(fd, "FUNCTIONS SORTED ON {title} TIME")?;
    writeln!(fd, "count  total (s)   self (s)  function")?;
    for func in sorttab.iter().take(20) {
        let (count, total, own) = {
            let prof = func.prof.borrow();
            (prof.tm_count, prof.tm_total, prof.tm_self)
        };
        prof_func_line(fd, count, total, own, prefer_self)?;
        write!(fd, " ")?;
        write_func_name(fd, func)?;
    }
    writeln!(fd)
}

/// Where a function was defined, as the report's `Defined:` line.
fn write_func_origin(fd: &mut dyn Write, func: &UserFunc) -> io::Result<()> {
    let sctx = func.script_ctx.get();
    let p = get_scriptname(sctx, true);
    write!(fd, "    Defined: ")?;
    fd.write_all(p.to_bytes())?;
    writeln!(fd, ":{}", sctx.sc_lnum)?;
    Ok(())
}

/// Per-function sections plus the sorted lists.
fn func_dump_profile(fd: &mut dyn Write) -> io::Result<()> {
    let mut sorttab = profiled_functions();
    for func in &sorttab {
        write!(fd, "FUNCTION  ")?;
        write_func_name(fd, func)?;
        if func.script_ctx.get().sc_sid != 0 {
            write_func_origin(fd, func)?;
        }
        let prof = func.prof.borrow();
        if prof.tm_count == 1 {
            writeln!(fd, "Called 1 time")?;
        } else {
            writeln!(fd, "Called {} times", prof.tm_count)?;
        }
        writeln!(fd, "Total time: {}", profile_msg_str(prof.tm_total))?;
        writeln!(fd, " Self time: {}", profile_msg_str(prof.tm_self))?;
        write!(fd, "\ncount  total (s)   self (s)\n")?;
        let body = func.body();
        for (i, line) in body.lines.iter().enumerate() {
            let Some(line) = line else {
                continue;
            };
            // The three per-line counters are sized to the body.
            let count = prof.tml_count.get(i).copied().unwrap_or(0);
            let total = prof.tml_total.get(i).copied().unwrap_or(0);
            let own = prof.tml_self.get(i).copied().unwrap_or(0);
            prof_func_line(fd, count, total, own, true)?;
            // Upstream prints the line as a C string.
            let end = line.iter().position(|&b| b == 0).unwrap_or(line.len());
            fd.write_all(&line[..end])?;
            writeln!(fd)?;
        }
        writeln!(fd)?;
    }
    if !sorttab.is_empty() {
        // A stable `sort_by` where upstream `qsort`s, and the divergence is
        // real: `profile_cmp` answers equal for two functions with identical
        // totals, which is the common case (everything never called, and
        // everything under the clock's resolution). Upstream's order between
        // those is whatever `qsort` landed on -- unspecified, and not even
        // fixed across glibc versions, since 2.37 replaced the merge sort
        // that had made it stable in practice with an introsort. A stable
        // sort at least answers the collection order, which is the same
        // `func_hashtab` walk upstream feeds `qsort`. `syntax/syntime.rs`
        // is the sibling that answered the same question the other way and
        // kept `qsort`, for the same comparator.
        sorttab.sort_by(|a, b| {
            profile_cmp(a.prof.borrow().tm_total, b.prof.borrow().tm_total).cmp(&0)
        });
        prof_sort_list(fd, &sorttab, "TOTAL", false)?;
        sorttab
            .sort_by(|a, b| profile_cmp(a.prof.borrow().tm_self, b.prof.borrow().tm_self).cmp(&0));
        prof_sort_list(fd, &sorttab, "SELF", true)?;
    }
    Ok(())
}

/// C's `fgets` over `reader`: at most `size - 1` bytes, up to and including
/// the first newline. `None` at the end of the file with nothing read, or
/// on a read error.
fn fgets(reader: &mut impl BufRead, size: usize) -> Option<Vec<u8>> {
    let limit = size - 1;
    let mut out = Vec::new();
    while out.len() < limit {
        let buf = match reader.fill_buf() {
            Ok(buf) => buf,
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => return None,
        };
        if buf.is_empty() {
            break;
        }
        let room = (limit - out.len()).min(buf.len());
        let take = buf[..room]
            .iter()
            .position(|&b| b == b'\n')
            .map_or(room, |at| at + 1);
        out.extend_from_slice(&buf[..take]);
        reader.consume(take);
        if out.last() == Some(&b'\n') {
            break;
        }
    }
    (!out.is_empty()).then_some(out)
}

/// Whether the byte `fgets` left at the last-but-one place of its buffer
/// says the line did not fit.
fn filled(c: u8) -> bool {
    c != 0 && c != b'\n'
}

/// Upstream's `vim_fgets` with an `IOSIZE` buffer: one line of at most
/// `IOSIZE - 1` bytes. A longer line is cut there and the rest of it read
/// and thrown away, 199 bytes at a time; when that throwing away reaches
/// the end of the file, the answer is `None` as at the end of the file,
/// and the cut line is lost with it.
fn vim_fgets_line(reader: &mut impl BufRead) -> Option<Vec<u8>> {
    const DISCARD: usize = 200;
    let size = IOSIZE as usize;
    let line = fgets(reader, size)?;
    if line.len() == size - 1 && filled(line[size - 2]) {
        // Now throw away the rest of the line.
        loop {
            let chunk = fgets(reader, DISCARD)?;
            if !(chunk.len() == DISCARD - 1 && filled(chunk[DISCARD - 2])) {
                break;
            }
        }
    }
    Some(line)
}

/// One script's source, annotated line by line with its counters. The read
/// runs to the end of file so that trailing continuation lines are listed.
fn script_dump_source(fd: &mut dyn Write, path: &[u8], counters: &[SnPrl]) -> io::Result<()> {
    let Ok(file) = File::open(OsStr::from_bytes(path)) else {
        return writeln!(fd, "Cannot open file!");
    };
    let mut reader = BufReader::new(file);
    let size = IOSIZE as usize;
    let mut i: usize = 0;
    while let Some(mut line) = vim_fgets_line(&mut reader) {
        // When a line has been truncated, append NL, taking care of
        // multibyte characters.
        if line.len() == size - 1 && filled(line[size - 2]) {
            let mut n = size - 2;
            // Move back to the first byte of the char.
            while n > 0 && (line[n] & 0xc0) == 0x80 {
                n -= 1;
            }
            line.truncate(n);
            line.push(b'\n');
        }
        // The buffer is read as a C string.
        let end = line.iter().position(|&b| b == 0).unwrap_or(line.len());
        match counters.get(i).copied().filter(|pp| pp.snp_count > 0) {
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
        fd.write_all(&line[..end])?;
        i += 1;
    }
    Ok(())
}

/// What the report reads of one script's item.
struct ScriptProfile {
    count: c_int,
    total: ProfTime,
    own: ProfTime,
    lines: Vec<SnPrl>,
}

/// Per-script sections: each profiled script's source lines annotated with
/// their counters.
fn script_dump_profile(fd: &mut dyn Write) -> io::Result<()> {
    for id in 1..=script_count() {
        let profile = with_script_item(id, |si| {
            si.sn_prof_on.then(|| ScriptProfile {
                count: si.sn_pr_count,
                total: si.sn_pr_total,
                own: si.sn_pr_self,
                lines: si.sn_prl_ga.clone(),
            })
        });
        let Some(profile) = profile else {
            continue;
        };
        let sctx = ScriptCtx {
            sc_sid: id,
            ..ScriptCtx::default()
        };
        let name = get_scriptname(sctx, false);
        write!(fd, "SCRIPT  ")?;
        fd.write_all(name.to_bytes())?;
        writeln!(fd)?;
        if profile.count == 1 {
            writeln!(fd, "Sourced 1 time")?;
        } else {
            writeln!(fd, "Sourced {} times", profile.count)?;
        }
        writeln!(fd, "Total time: {}", profile_msg_str(profile.total))?;
        writeln!(fd, " Self time: {}", profile_msg_str(profile.own))?;
        write!(fd, "\ncount  total (s)   self (s)\n")?;
        script_dump_source(fd, name.to_bytes(), &profile.lines)?;
        writeln!(fd)?;
    }
    Ok(())
}
