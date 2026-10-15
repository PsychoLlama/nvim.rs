//! Reading a sourced script: the [`SourceCookie`] `do_cmdline` pulls lines
//! from while a script runs.
//!
//! [`SourceCookie::getsourceline`] is the [`Script`](crate::ex_docmd::LineSource::Script)
//! arm of the command reader: it joins `\` continuation lines (reading one
//! line ahead to find out), applies 'scriptencoding' conversion and gives the
//! debugger its per-line hook. Underneath, a line comes either from the file
//! -- read the way upstream's `fgets` loop read it, a chunk at a time -- or
//! from the lines a buffer range or a string was split into before the run.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::charset::skip;
use crate::cstr;
use crate::ex_docmd::DebugPos;
use crate::option::cpo_has;
use crate::types::CpoFlag;
use core::cell::Cell;
use core::ffi::c_int;
use std::ffi::CString;
use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind};

/// How much one `fgets` read at a time, terminator included: upstream grew
/// its buffer by at least this much before every read, so a longer line
/// arrives in pieces of one fewer byte than this.
const SOURCE_CHUNK: usize = 120;

/// Where a script's lines come from.
pub(crate) enum ScriptLines {
    /// Nothing: the file could not be opened (or nothing was loaded yet).
    None,
    /// A file, read as it runs.
    File(BufReader<File>),
    /// The lines of a buffer range or a string, split before the run;
    /// `next` is the index of the next one to hand out.
    Lines { lines: Vec<CString>, next: usize },
}

/// One script being read: the reader and everything `do_cmdline` and the
/// debugger ask about it while it runs.
pub(crate) struct SourceCookie {
    pub(crate) lines: ScriptLines,
    /// The line read ahead to see whether it continues the current one.
    nextline: Option<Vec<u8>>,
    pub(crate) sourcing_lnum: LineNr,
    pub(crate) finished: bool,
    pub(crate) breakpoint: LineNr,
    /// The script's resolved name, which a breakpoint names; empty for a
    /// cookie that was never run.
    pub(crate) fname: XString,
    pub(crate) dbg_tick: c_int,
    pub(crate) level: c_int,
    pub(crate) conv: VimConv,
}

impl SourceCookie {
    /// A cookie nothing has been loaded into yet -- what upstream's
    /// `CLEAR_FIELD` left behind.
    pub(crate) fn new() -> Self {
        Self {
            lines: ScriptLines::None,
            nextline: None,
            sourcing_lnum: 0,
            finished: false,
            breakpoint: 0,
            fname: XString::new(),
            dbg_tick: 0,
            level: 0,
            conv: VimConv {
                vc_type: 0,
                vc_factor: 0,
                vc_fd: None,
                vc_fail: false,
            },
        }
    }

    /// A cookie reading `lines`, as `:source` of a buffer or a string loads.
    pub(crate) fn from_lines(lines: Vec<CString>) -> Self {
        Self {
            lines: ScriptLines::Lines { lines, next: 0 },
            ..Self::new()
        }
    }

    /// Whether the lines come from a buffer or a string rather than a file:
    /// those have no breakpoints.
    pub(crate) fn reads_buf_or_str(&self) -> bool {
        matches!(self.lines, ScriptLines::Lines { .. })
    }

    /// The lines a buffer or a string was split into; empty for a file.
    pub(crate) fn buffered_lines(&self) -> &[CString] {
        match &self.lines {
            ScriptLines::Lines { lines, .. } => lines,
            _ => &[],
        }
    }

    /// Where the debugger keeps this script's breakpoint, for `do_cmdline`'s
    /// replay of a loop body.
    pub(crate) fn debug_pos(&mut self) -> DebugPos<'_> {
        DebugPos {
            name: self.fname.as_cstr(),
            breakpoint: Cell::from_mut(&mut self.breakpoint),
            tick: Cell::from_mut(&mut self.dbg_tick),
        }
    }

    /// Get one full line from a sourced file, for `do_cmdline()` under
    /// `do_source()`.
    ///
    /// The [`Script`](crate::ex_docmd::LineSource::Script) arm of its line
    /// source. Answers the line, or `None` at end of file or on error.
    pub(crate) fn getsourceline(&mut self, do_concat: bool) -> Option<XString> {
        let from_buf_or_str = self.reads_buf_or_str();

        // If breakpoints have been added or deleted we need to look again.
        if self.dbg_tick < debug_tick.get() && !from_buf_or_str {
            self.refresh_breakpoint();
        }
        if do_profiling.get() == PROF_YES {
            script_line_end();
        }
        // Set the current sourcing line number.
        set_sourcing_lnum(self.sourcing_lnum + 1);

        let mut line = self.next_line();
        if line.is_some() && do_profiling.get() == PROF_YES {
            script_line_start();
        }

        // Only concatenate lines starting with a `\` when 'cpoptions' does
        // not contain the 'C' flag.
        if do_concat && let Some(first) = line.take() {
            line = Some(if cpo_has(CpoFlag::CONCAT) {
                first
            } else {
                self.concat_continuations(first)
            });
        }

        // Convert the encoding of the script line.
        if self.conv.vc_type != CONV_NONE
            && let Some(text) = &line
            && let Some(converted) = self.conv.convert(text)
        {
            line = Some(converted);
        }

        // Did we encounter a breakpoint?
        if !from_buf_or_str && self.breakpoint != 0 && self.breakpoint <= sourcing_lnum() {
            dbg_breakpoint(self.fname.as_cstr(), sourcing_lnum());
            self.refresh_breakpoint();
        }

        line.map(XString::from_vec)
    }

    /// Look up the next breakpoint in the script, and remember the
    /// debugger's tick so we only look again when something changed.
    fn refresh_breakpoint(&mut self) {
        self.breakpoint = dbg_find_breakpoint_named(true, self.fname.as_cstr(), sourcing_lnum());
        self.dbg_tick = debug_tick.get();
    }

    /// The next line of the script, using the one read ahead if there is
    /// one.
    fn next_line(&mut self) -> Option<Vec<u8>> {
        if self.finished || matches!(self.lines, ScriptLines::None) {
            return None;
        }
        match self.nextline.take() {
            Some(line) => {
                self.sourcing_lnum += 1;
                Some(line)
            }
            None => self.get_one_sourceline(),
        }
    }

    /// Join the `\`-continuation lines that follow `line` onto it.
    ///
    /// We always have to read the next line to find out, so it is kept in
    /// `nextline`.  A comment between continuation lines (`"\ `) counts as
    /// one.
    fn concat_continuations(&mut self, mut line: Vec<u8>) -> Vec<u8> {
        // Compensate for the one line read-ahead.
        self.sourcing_lnum -= 1;
        self.nextline = self.get_one_sourceline();
        if !self.nextline.as_deref().is_some_and(starts_continuation) {
            return line;
        }
        while let Some(next) = &self.nextline
            && concat_continued_line(&mut line, next)
        {
            self.nextline = self.get_one_sourceline();
        }
        line
    }

    /// Read one line of the script, or `None` at end of file.
    ///
    /// A file line can span several reads: the chunk may be too short for
    /// it, and a newline escaped with an odd run of CTRL-V's does not end it.
    fn get_one_sourceline(&mut self) -> Option<Vec<u8>> {
        let mut line: Vec<u8> = Vec::new();
        self.sourcing_lnum += 1;

        // Loop until there is a finished line (or end-of-file).
        let mut have_read = false;
        loop {
            match &mut self.lines {
                ScriptLines::None => break,
                ScriptLines::Lines { lines, next } => {
                    let Some(text) = lines.get(*next) else {
                        break;
                    };
                    // A buffer or string line is handed over whole: it cannot
                    // hold a newline, so neither the continuation test nor
                    // the escaped-NL test below can fire for one.
                    line.extend_from_slice(text.to_bytes());
                    *next += 1;
                    have_read = true;
                }
                ScriptLines::File(file) => {
                    let Some(filled_the_chunk) = read_chunk(file, &mut line) else {
                        break;
                    };
                    have_read = true;
                    // If the line was longer than the chunk, read more.
                    if filled_the_chunk && line.last() != Some(&b'\n') {
                        continue;
                    }
                    if line.last() == Some(&b'\n') {
                        if escaped_newline(&line) {
                            self.sourcing_lnum += 1;
                            continue;
                        }
                        // Remove the NL.
                        line.pop();
                    }
                }
            }

            // Check for CTRL-C here now and then, so a recursive `:so` can
            // be broken out of.
            line_breakcheck();
            break;
        }

        have_read.then_some(line)
    }
}

impl Default for SourceCookie {
    fn default() -> Self {
        Self::new()
    }
}

/// `fgets` one chunk of `file` onto the end of `line`: up to and including a
/// newline, or `SOURCE_CHUNK - 1` bytes, or what is left of the file; then
/// cut at the first NUL among the bytes read, which is where the C reader
/// measured the chunk to end. Answers whether the chunk used every byte it
/// was offered -- which is how a line longer than one chunk announces
/// itself -- or `None` at end of file or on a read error.
fn read_chunk(file: &mut BufReader<File>, line: &mut Vec<u8>) -> Option<bool> {
    let room = SOURCE_CHUNK - 1;
    let start = line.len();
    let mut nul = None;
    let mut got = 0;
    while got < room {
        let available = match file.fill_buf() {
            Ok(available) => available,
            // `fgets` is retried when a signal interrupts it.
            Err(err) if err.kind() == ErrorKind::Interrupted => continue,
            // A read error ends the chunk where it struck, as the end of the
            // file would.
            Err(_) => break,
        };
        let take = (room - got).min(available.len());
        if take == 0 {
            break;
        }
        let chunk = &available[..take];
        let end = cstr::find_either(chunk, b'\n', 0);
        // The first NUL and the newline, whichever comes first; past a NUL
        // the newline still ends the read.
        let (taken, ends) = match end {
            Some(at) if chunk[at] == b'\n' => (at + 1, true),
            Some(at) => {
                nul.get_or_insert(got + at);
                match cstr::find_byte(&chunk[at..], b'\n') {
                    Some(nl) => (at + nl + 1, true),
                    None => (take, false),
                }
            }
            None => (take, false),
        };
        line.extend_from_slice(&chunk[..taken]);
        file.consume(taken);
        got += taken;
        if ends {
            break;
        }
    }
    if got == 0 {
        return None;
    }
    let measured = nul.unwrap_or(got);
    line.truncate(start + measured);
    Some(measured == room)
}

/// Is the newline at the end of `line` escaped?
///
/// It is when an odd number of CTRL-V's precede it.  Upstream compares the
/// parity of the length against the parity of the index just before that run,
/// which is faster than counting the run and says the same thing.
fn escaped_newline(line: &[u8]) -> bool {
    let body = &line[..line.len() - 1];
    let run = body
        .iter()
        .rev()
        .take_while(|&&b| c_int::from(b) == Ctrl_V)
        .count();
    run % 2 == 1
}

/// Does `line` begin a continuation -- a `\`, or the `"\ ` that comments one
/// out -- after its leading white space?
fn starts_continuation(line: &[u8]) -> bool {
    let line = &line[skip::white(line)..];
    line.first() == Some(&b'\\') || line.starts_with(b"\"\\ ")
}

/// Append the continuation `line` to `text`, answering whether it was one.
///
/// A `"\ ` comment line is *not* a continuation but does not end one either,
/// so it answers true having appended nothing.
pub(crate) fn concat_continued_line(text: &mut Vec<u8>, line: &[u8]) -> bool {
    let line = &line[skip::white(line)..];
    if line.len() >= 3 && line.starts_with(b"\"\\ ") {
        return true;
    }
    let Some(rest) = line.strip_prefix(b"\\") else {
        return false;
    };
    // The `ga_set_growsize` that used to sit here is gone with the garray:
    // a `Vec` already doubles, so a line with many continuations reallocates
    // logarithmically often on its own.
    text.extend_from_slice(rest);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The parity rule is the run of CTRL-V's: odd escapes the newline.
    #[test]
    fn an_odd_run_of_ctrl_v_escapes_the_newline() {
        assert!(!escaped_newline(b"\n"));
        assert!(!escaped_newline(b"abc\n"));
        assert!(escaped_newline(b"a\x16\n"));
        assert!(!escaped_newline(b"a\x16\x16\n"));
        assert!(escaped_newline(b"\x16\x16\x16\n"));
        assert!(!escaped_newline(b"\x16a\n"));
    }

    #[test]
    fn a_continuation_is_a_backslash_or_a_commented_one() {
        let mut text = b"let x = [".to_vec();
        assert!(concat_continued_line(&mut text, b"   \\ 1,"));
        assert!(concat_continued_line(&mut text, b"\t\"\\ comment"));
        assert!(!concat_continued_line(&mut text, b"  echo"));
        assert!(!concat_continued_line(&mut text, b""));
        assert!(concat_continued_line(&mut text, b"\\]"));
        assert_eq!(text, b"let x = [ 1,]");
        assert!(starts_continuation(b"  \\x"));
        assert!(starts_continuation(b"\"\\ c"));
        assert!(!starts_continuation(b"\"\\c"));
        assert!(!starts_continuation(b"x\\"));
    }
}
