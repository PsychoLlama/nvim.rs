//! Matching one line of output against the compiled formats.
//!
//! [`parse_line`] runs an [`Efm`]'s formats over a line, in order, until one
//! matches. [`Format::exec`] does the matching; [`Fields::take_match`] then
//! pulls out the values the format's `%` conversions captured, one arm per
//! conversion.
//!
//! What happens next depends on the format's prefix. A plain format yields
//! an entry. `%D`/`%X` push and pop the directory the following file names
//! are relative to. `%A`…`%N` open a multi-line message that `%C` lines
//! continue and a `%Z` line closes — [`continue_multiline`] folds those into
//! the entry the opening line made. `%O`/`%P`/`%Q` name a file the following
//! lines belong to, and may leave a tail that is re-scanned as a line of its
//! own.

#![forbid(unsafe_code)]

use super::*;
use crate::memory::XString;
use crate::os::env::expand_env_into;
use crate::os::fs::path_exists;
use core::ffi::{CStr, c_char, c_int};

/// How large the fixed field buffers are. A file name, module name or
/// search pattern longer than this is truncated, as upstream does.
const FIELD_MAX: usize = CMDBUFFSIZE as usize;

/// What became of one line, or of one attempt to match it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Status {
    /// Parsed; [`Fields`] describes the entry to add.
    Ok,
    /// Nothing usable here — for one format, that it did not match; for a
    /// whole line, that reading should stop.
    Fail,
    /// The input ran out.
    EndOfInput,
    /// The line was consumed but makes no entry of its own.
    Ignore,
    /// A `%O`/`%P`/`%Q` format claimed a file name and left a tail; the
    /// tail is re-scanned as if it were a line of its own.
    MultiScan,
}

/// The values one parsed line yielded.
///
/// The name, module and pattern are bounded as upstream's fixed buffers
/// were; the message grows, since `%m` and `%+` can copy a whole line into
/// it.
pub(crate) struct Fields {
    namebuf: XString,
    module: XString,
    pattern: XString,
    errmsg: XString,
    /// The list's directory and claimed file, copied for the entry being
    /// made: naming the file can run an autocommand that changes the list.
    directory: Option<XString>,
    current_file: Option<XString>,
    /// Buffer number, from `%b`.
    pub(crate) bnr: c_int,
    /// Line number, from `%l`.
    pub(crate) lnum: LineNr,
    /// End line number, from `%e`.
    pub(crate) end_lnum: LineNr,
    /// Column, from `%c`, `%v` or `%p`.
    pub(crate) col: c_int,
    /// End column, from `%k`.
    pub(crate) end_col: c_int,
    /// The column is a screen column, not a byte index (`%v`, `%p`).
    pub(crate) use_viscol: bool,
    /// Error number, from `%n`.
    pub(crate) enr: c_int,
    /// Error type, from `%t` or from an `%E`/`%W`/`%I`/`%N` prefix.
    pub(crate) kind: c_char,
    /// The line named a real position, so the entry can be jumped to.
    pub(crate) valid: bool,
}

/// `bytes` up to the first NUL in them, as C's string functions see them.
fn until_nul(bytes: &[u8]) -> &[u8] {
    &bytes[..bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len())]
}

/// C's `atol` on the start of `text`: leading white space, a sign, digits.
/// Saturating, as glibc's is, and then narrowed the way upstream's `(int)`
/// cast narrows.
fn parse_long(text: &[u8]) -> i64 {
    let text = &text[text
        .iter()
        .position(|b| !b" \t\n\x0b\x0c\r".contains(b))
        .unwrap_or(text.len())..];
    let (negative, digits) = match text.first() {
        Some(b'-') => (true, &text[1..]),
        Some(b'+') => (false, &text[1..]),
        _ => (false, text),
    };
    let mut n: i64 = 0;
    for &d in digits.iter().take_while(|b| b.is_ascii_digit()) {
        n = n.saturating_mul(10).saturating_add(i64::from(d - b'0'));
    }
    if negative { n.saturating_neg() } else { n }
}

/// [`parse_long`] narrowed to an `int` the way C's cast narrows: by wrapping.
fn parse_int(text: &[u8]) -> c_int {
    let n = parse_long(text);
    #[allow(
        clippy::cast_possible_truncation,
        reason = "C's `(int)parse_long(...)`"
    )]
    let n = n as c_int;
    n
}

impl Fields {
    pub(crate) fn new() -> Fields {
        Fields {
            namebuf: XString::new(),
            module: XString::new(),
            pattern: XString::new(),
            errmsg: XString::new(),
            directory: None,
            current_file: None,
            bnr: 0,
            lnum: 0,
            end_lnum: 0,
            col: 0,
            end_col: 0,
            use_viscol: false,
            enr: 0,
            kind: 0,
            valid: false,
        }
    }

    /// The file name the line named; empty when it named none.
    pub(crate) fn namebuf(&self) -> &CStr {
        self.namebuf.as_cstr()
    }

    /// Whether a file name was found.
    pub(crate) fn has_name(&self) -> bool {
        !self.namebuf.is_empty()
    }

    /// Copy at most `linelen` bytes of `src` into the message, stopping at a
    /// NUL. This is `%m` and `%+`, and it is also what a line matching no
    /// format at all leaves behind.
    fn set_message(&mut self, src: &[u8], linelen: usize) {
        let text = until_nul(&src[..linelen.min(src.len())]);
        self.errmsg = XString::from_bytes(text);
    }

    /// Clear every field a format could set, ready for one attempt.
    ///
    /// The message survives a re-scan: a `%O`/`%P`/`%Q` format matched the
    /// file name off this line already, and what it left is still the
    /// message.
    fn reset(&mut self, multiscan: bool, tail: &mut Option<usize>) {
        self.namebuf.truncate(0);
        self.bnr = 0;
        self.module.truncate(0);
        self.pattern.truncate(0);
        if !multiscan {
            self.errmsg.truncate(0);
        }
        self.lnum = 0;
        self.end_lnum = 0;
        self.col = 0;
        self.end_col = 0;
        self.use_viscol = false;
        self.enr = -1;
        self.kind = 0;
        *tail = None;
    }

    /// Try one format against the line, filling in the fields it captures.
    /// `tail` is set to where a `%r` capture starts in the line.
    fn try_format(
        &mut self,
        line: &CStr,
        linelen: usize,
        fmt: &mut Format,
        multiline: bool,
        multiscan: bool,
        tail: &mut Option<usize>,
    ) -> Status {
        // A re-scan of a tail is offered only to the file-name formats, and
        // leaves the fields the first scan set alone.
        if multiscan && !b"OPQ".contains(&fmt.prefix()) {
            return Status::Fail;
        }
        self.reset(multiscan, tail);

        // Case is always ignored when looking for an error.
        let Some(regmatch) = fmt.exec(line) else {
            return Status::Fail;
        };
        self.take_match(
            line.to_bytes(),
            linelen,
            fmt,
            &regmatch,
            multiline,
            multiscan,
            tail,
        )
    }

    /// Pull the values `fmt`'s conversions captured out of `regmatch`.
    ///
    /// Answers [`Status::Fail`] as soon as one conversion is unusable — a
    /// submatch that did not participate, a `%b` naming no buffer, a `%f`
    /// under `%O`/`%P`/`%Q` naming a file that does not exist.
    #[allow(clippy::too_many_arguments)]
    fn take_match(
        &mut self,
        line: &[u8],
        linelen: usize,
        fmt: &Format,
        regmatch: &RegMatch,
        multiline: bool,
        multiscan: bool,
        tail: &mut Option<usize>,
    ) -> Status {
        let prefix = fmt.prefix();
        if (prefix == b'C' || prefix == b'Z') && !multiline {
            return Status::Fail;
        }
        self.kind = if b"EWIN".contains(&prefix) {
            prefix as c_char
        } else {
            0
        };

        // Check for an actual submatch on each conversion: "\[" and "\]" in
        // 'errorformat' can make the wrong one match.
        for idx in 0..FMT_PATTERNS {
            // Both regexp engines reject a pattern with more than nine
            // groups (E872, E51), so a format that compiled at all names at
            // most nine conversions and this index is within the groups.
            let midx = fmt.submatch(idx);
            let status = match idx {
                0 if midx > 0 => self.take_file(line, regmatch, midx, prefix),
                FMT_PATTERN_M => {
                    if fmt.flags() == b'+' && !multiscan {
                        // %+ : the whole line is the message.
                        self.set_message(line, linelen);
                        Status::Ok
                    } else if midx > 0 {
                        self.take_message(line, regmatch, midx)
                    } else {
                        Status::Ok
                    }
                }
                FMT_PATTERN_R if midx > 0 => {
                    // %r : whatever follows a file name, to be re-scanned.
                    match regmatch.starts[midx] {
                        None => Status::Fail,
                        Some(at) => {
                            *tail = Some(at);
                            Status::Ok
                        }
                    }
                }
                _ if midx > 0 => self.take_conversion(line, regmatch, midx, idx),
                _ => Status::Ok,
            };
            if status != Status::Ok {
                return status;
            }
        }

        Status::Ok
    }

    /// `%f`: the file name, with `~/` and `$HOME/` expanded.
    fn take_file(&mut self, line: &[u8], rmp: &RegMatch, midx: usize, prefix: u8) -> Status {
        let Some(span) = rmp.group(midx) else {
            return Status::Fail;
        };
        let name = XString::from_bytes(until_nul(&line[span.start..span.end]));
        self.namebuf = XString::from_bytes(&expand_env_into(name.as_cstr(), FIELD_MAX + 1));
        // A separate file-name format (%O, %P, %Q) only claims a line
        // when the file it names really exists.
        if b"OPQ".contains(&prefix) && !path_exists(self.namebuf.as_cstr()) {
            return Status::Fail;
        }
        Status::Ok
    }

    /// `%m`: the error message.
    fn take_message(&mut self, line: &[u8], rmp: &RegMatch, midx: usize) -> Status {
        let Some(span) = rmp.group(midx) else {
            return Status::Fail;
        };
        self.set_message(&line[span.start..], span.end - span.start);
        Status::Ok
    }

    /// Every conversion that is neither `%f`, `%m` nor `%r`. `idx` indexes
    /// [`FMT_PAT`], so the arms below must stay in step with that table.
    fn take_conversion(&mut self, line: &[u8], rmp: &RegMatch, midx: usize, idx: usize) -> Status {
        let Some(at) = rmp.starts[midx] else {
            return Status::Fail;
        };
        // The submatch begins inside the line and `atol` stops at the first
        // byte that is not part of a number, so the end of the match does
        // not have to be marked.
        let start = &line[at..];
        let end = rmp.ends[midx];
        match idx {
            1 => {
                // %b: a buffer number, which must name a live buffer.
                let bnr = parse_int(start);
                if find_buf(bnr).is_none() {
                    return Status::Fail;
                }
                self.bnr = bnr;
            }
            2 => self.enr = parse_int(start),      // %n
            3 => self.lnum = parse_int(start),     // %l
            4 => self.end_lnum = parse_int(start), // %e
            5 => self.col = parse_int(start),      // %c
            6 => self.end_col = parse_int(start),  // %k
            7 => self.kind = start.first().copied().unwrap_or(0) as c_char, // %t
            10 => {
                // %p: a pointer line such as "   ^", whose width is the
                // column. A tab advances to the next multiple of eight.
                let Some(end) = end else {
                    return Status::Fail;
                };
                self.col = 0;
                for &b in &line[at..end] {
                    self.col += 1;
                    if c_int::from(b) == TAB {
                        self.col += 7;
                        self.col -= self.col % 8;
                    }
                }
                self.col += 1;
                self.use_viscol = true;
            }
            11 => {
                // %v: a screen column.
                self.col = parse_int(start);
                self.use_viscol = true;
            }
            12 => {
                // %s: the matched text, as a very-nomagic pattern anchored
                // at both ends. Five bytes go around it, so that much less
                // of the match fits.
                let Some(end) = end else {
                    return Status::Fail;
                };
                let len = (end - at).min(FIELD_MAX - 5);
                let mut pattern = b"^\\V".to_vec();
                pattern.extend_from_slice(until_nul(&line[at..at + len]));
                pattern.extend_from_slice(b"\\$");
                self.pattern = XString::from_bytes(&pattern);
            }
            13 => {
                // %o: the module name, appended to whatever is there, within
                // the field's bound.
                let Some(end) = end else {
                    return Status::Fail;
                };
                let have = self.module.len();
                let room = (have + (end - at)).min(FIELD_MAX - 1).saturating_sub(have);
                let add = until_nul(&line[at..]);
                self.module.push_bytes(&add[..room.min(add.len())]);
            }
            _ => unreachable!("conversion {idx} is handled by its caller"),
        }
        Status::Ok
    }
}

/// Match one line against the formats in `efm`, and act on what matched.
///
/// `linelen` is the length the reader measured, which counts a newline it
/// has since overwritten.
pub(crate) fn parse_line(
    mut qfl: Qfl,
    line: &CStr,
    linelen: usize,
    efm: &mut Efm,
    fields: &mut Fields,
) -> Status {
    let mut line = line;
    let mut linelen = linelen;
    // Not reset between scans: a `%r` from the first pass is still what a
    // later pass would re-scan if no format resets it.
    let mut tail: Option<usize> = None;
    // A tail is re-scanned from a copy, since it is a part of `line`.
    let mut rescan;

    // A `%O`/`%P`/`%Q` match can leave a tail, which is then scanned as a
    // line of its own; that is the only way round this loop.
    loop {
        let (multiline, multiscan) = (qfl.multiline, qfl.multiscan);
        fields.valid = true;

        // Start at the first format, or — after a `%>` — at the one that
        // matched last time.
        let mut matched = None;
        for idx in efm.take_resume()..efm.len() {
            let status = fields.try_format(
                line,
                linelen,
                efm.format(idx),
                multiline,
                multiscan,
                &mut tail,
            );
            if status == Status::Ok {
                matched = Some(idx);
                break;
            }
        }
        qfl.multiscan = false;

        let Some(idx) = matched else {
            // Nothing matched: keep the line as a message, and close any
            // multi-line message that was open.
            no_match(line.to_bytes(), linelen, fields);
            qfl.multiline = false;
            qfl.multiignore = false;
            return Status::Ok;
        };

        let (prefix, flags, conthere) = {
            let fmt = efm.format(idx);
            (fmt.prefix(), fmt.flags(), fmt.conthere())
        };

        if prefix == b'D' || prefix == b'X' {
            let status = push_pop_dir(prefix, fields, qfl);
            if status != Status::Ok {
                return status;
            }
            // A directory line is never an entry of its own, but it is kept
            // as a message like any unmatched line.
            no_match(line.to_bytes(), linelen, fields);
            return Status::Ok;
        }

        // Honour a `%>` item: the next line starts matching here.
        if conthere {
            efm.set_resume(idx);
        }

        if b"AEWIN".contains(&prefix) {
            qfl.multiline = true; // start of a multi-line message
            qfl.multiignore = false; // reset continuation
        } else if b"CZ".contains(&prefix) {
            // A continuation line never makes an entry of its own, so this
            // always ends the line.
            return continue_multiline(prefix, qfl, fields);
        } else if b"OPQ".contains(&prefix) {
            let rest = tail.map(|at| &line.to_bytes()[at..]);
            if claim_file(prefix, fields, qfl, rest) == Status::MultiScan {
                let rest = rest.expect("a re-scan has a tail");
                let rest = skip_white(rest);
                if rest.len() >= linelen {
                    // The tail is no shorter than the line it came from, so
                    // re-scanning could not make progress.
                    return Status::Ignore;
                }
                rescan = XString::from_bytes(rest);
                linelen = rescan.len();
                line = rescan.as_cstr();
                continue;
            }
        }

        if flags == b'-' {
            // Generally exclude this line.
            if qfl.multiline {
                // Exclude its continuation lines too.
                qfl.multiignore = true;
            }
            return Status::Ignore;
        }

        return Status::Ok;
    }
}

/// A line that matched no format: keep it as the message, but do not let it
/// be jumped to.
fn no_match(line: &[u8], linelen: usize, fields: &mut Fields) {
    fields.namebuf.truncate(0); // no match found, so no file name
    fields.lnum = 0; // don't jump to this line
    fields.valid = false;
    fields.set_message(line, linelen);
}

/// `%D` and `%X`: enter and leave the directory the following file names
/// are relative to.
fn push_pop_dir(prefix: u8, fields: &Fields, mut qfl: Qfl) -> Status {
    if prefix == b'D' {
        if !fields.has_name() {
            emsg(gettext(c"E379: Missing or empty directory name"));
            return Status::Fail;
        }
        qf_push_dir(fields.namebuf(), &mut qfl.dir_stack, false);
    } else {
        qf_pop_dir(&mut qfl.dir_stack);
    }
    Status::Ok
}

/// `%O`, `%P` and `%Q`: name the file the following lines belong to.
/// `tail` is what follows the `%r` capture, if the format had one.
fn claim_file(prefix: u8, fields: &mut Fields, mut qfl: Qfl, tail: Option<&[u8]>) -> Status {
    fields.valid = false;
    if fields.has_name() && !path_exists(fields.namebuf()) {
        return Status::Ok;
    }
    if fields.has_name() && prefix == b'P' {
        qf_push_dir(fields.namebuf.as_cstr(), &mut qfl.file_stack, true);
    } else if prefix == b'Q' {
        qf_pop_dir(&mut qfl.file_stack);
    }
    fields.namebuf.truncate(0);
    if tail.is_some_and(|tail| !until_nul(tail).is_empty()) {
        qfl.multiscan = true;
        return Status::MultiScan;
    }
    Status::Ok
}

/// `%C` and `%Z`: fold a continuation line into the entry the opening line
/// of the multi-line message made.
fn continue_multiline(prefix: u8, mut qfl: Qfl, fields: &mut Fields) -> Status {
    if !qfl.multiignore {
        if qfl.is_empty() {
            return Status::Fail;
        }
        let needs_fnum = {
            let prev = qfl.entries.last_mut().expect("tested above");
            if !fields.errmsg.is_empty() {
                // Append the continuation as a new line of the message.
                prev.text.push_byte(b'\n');
                prev.text.push_bytes(&fields.errmsg);
            }
            if prev.nr == -1 {
                prev.nr = fields.enr;
            }
            if vim_isprintc(c_int::from(fields.kind)) && prev.kind == 0 {
                // Only printable characters allowed.
                prev.kind = fields.kind;
            }
            if prev.lnum == 0 {
                prev.lnum = fields.lnum;
            }
            if prev.end_lnum == 0 {
                prev.end_lnum = fields.end_lnum;
            }
            if prev.col == 0 {
                prev.col = fields.col;
                prev.viscol = c_char::from(fields.use_viscol);
            }
            if prev.end_col == 0 {
                prev.end_col = fields.end_col;
            }
            prev.fnum == 0
        };
        if needs_fnum {
            // Naming the file can fire `BufNew`, so the entry is found again
            // afterwards.
            let entry = fields.entry(qfl);
            let fnum = qf_get_fnum(qfl, entry.dir, entry.fname);
            if let Some(prev) = qfl.entries.last_mut() {
                prev.fnum = fnum;
            }
        }
    }
    if prefix == b'Z' {
        qfl.multiline = false;
        qfl.multiignore = false;
    }
    line_breakcheck();

    Status::Ignore
}

impl Fields {
    /// The entry this parsed line makes. The strings stay in the fields;
    /// [`qf_add_entry`] copies what it keeps.
    ///
    /// The name is the one the line gave, or the file a `%P` claimed, or
    /// none at all; the list's directory and claimed file are copied into
    /// the fields first, since naming the file can change the list.
    pub(crate) fn entry(&mut self, qfl: Qfl) -> NewEntry<'_> {
        self.directory = qfl.directory().map(|dir| XString::from_bytes(dir.bytes()));
        self.current_file = qfl
            .current_file()
            .map(|file| XString::from_bytes(file.bytes()));
        let fname = if self.has_name() || self.directory.is_some() {
            Some(self.namebuf.as_cstr())
        } else if self.valid {
            self.current_file.as_ref().map(XString::as_cstr)
        } else {
            None
        };
        NewEntry {
            dir: self.directory.as_ref().map(XString::as_cstr),
            fname,
            module: Some(self.module.as_cstr()),
            bufnum: self.bnr,
            mesg: self.errmsg.as_cstr(),
            lnum: self.lnum,
            end_lnum: self.end_lnum,
            col: self.col,
            end_col: self.end_col,
            vis_col: c_char::from(self.use_viscol),
            pattern: Some(self.pattern.as_cstr()),
            nr: self.enr,
            kind: self.kind,
            user_data: None,
            valid: self.valid,
        }
    }
}
