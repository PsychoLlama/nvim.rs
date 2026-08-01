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

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use core::ffi::{c_char, c_int};
use core::ptr;

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
/// The three name buffers are fixed size because that is the interface
/// their writers have: `expand_env` and `xstrlcat` take a bound rather than
/// growing a buffer. The message is the one field that grows, since `%m`
/// and `%+` can copy a whole line into it.
pub(crate) struct Fields {
    namebuf: Vec<c_char>,
    module: Vec<c_char>,
    pattern: Vec<c_char>,
    errmsg: Vec<c_char>,
    /// Buffer number, from `%b`.
    pub(crate) bnr: c_int,
    /// Line number, from `%l`.
    pub(crate) lnum: linenr_T,
    /// End line number, from `%e`.
    pub(crate) end_lnum: linenr_T,
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
    /// Never set by parsing; an entry built from a Vimscript dictionary
    /// carries one, and `qf_add_entry` takes it from the same struct.
    pub(crate) user_data: *mut typval_T,
    /// The line named a real position, so the entry can be jumped to.
    pub(crate) valid: bool,
}

impl Fields {
    pub(crate) fn new() -> Fields {
        Fields {
            namebuf: vec![0; FIELD_MAX + 1],
            module: vec![0; FIELD_MAX + 1],
            pattern: vec![0; FIELD_MAX + 1],
            errmsg: vec![0; FIELD_MAX + 1],
            bnr: 0,
            lnum: 0,
            end_lnum: 0,
            col: 0,
            end_col: 0,
            use_viscol: false,
            enr: 0,
            kind: 0,
            user_data: ptr::null_mut(),
            valid: false,
        }
    }

    /// The file name the line named, NUL-terminated; empty when it named
    /// none.
    pub(crate) fn namebuf(&mut self) -> *mut c_char {
        self.namebuf.as_mut_ptr()
    }

    /// The module name, NUL-terminated.
    pub(crate) fn module(&mut self) -> *mut c_char {
        self.module.as_mut_ptr()
    }

    /// The search pattern, NUL-terminated.
    pub(crate) fn pattern(&mut self) -> *mut c_char {
        self.pattern.as_mut_ptr()
    }

    /// The error message, NUL-terminated.
    pub(crate) fn errmsg(&mut self) -> *mut c_char {
        self.errmsg.as_mut_ptr()
    }

    /// Whether a file name was found.
    pub(crate) fn has_name(&self) -> bool {
        self.namebuf[0] != 0
    }

    /// Copy `linelen` bytes into the message, growing the buffer if they do
    /// not fit. This is `%m` and `%+`, and it is also what a line matching
    /// no format at all leaves behind.
    ///
    /// # Safety
    ///
    /// `src` must be readable for `linelen` bytes or up to an earlier NUL.
    unsafe fn set_message(&mut self, src: *const c_char, linelen: usize) {
        if linelen >= self.errmsg.len() {
            self.errmsg.resize(linelen + 1, 0);
        }
        // SAFETY: the buffer now holds `linelen + 1` bytes, and `xstrlcpy`
        // stops at the source's NUL or at that bound.
        unsafe { xstrlcpy(self.errmsg.as_mut_ptr(), src, linelen + 1) };
    }

    /// Clear every field a format could set, ready for one attempt.
    ///
    /// The message survives a re-scan: a `%O`/`%P`/`%Q` format matched the
    /// file name off this line already, and what it left is still the
    /// message.
    fn reset(&mut self, multiscan: bool, tail: &mut *mut c_char) {
        self.namebuf[0] = 0;
        self.bnr = 0;
        self.module[0] = 0;
        self.pattern[0] = 0;
        if !multiscan {
            self.errmsg[0] = 0;
        }
        self.lnum = 0;
        self.end_lnum = 0;
        self.col = 0;
        self.end_col = 0;
        self.use_viscol = false;
        self.enr = -1;
        self.kind = 0;
        *tail = ptr::null_mut();
    }

    /// Try one format against the line, filling in the fields it captures.
    ///
    /// # Safety
    ///
    /// `linebuf` must be a writable, NUL-terminated buffer of `linelen`
    /// bytes.
    unsafe fn try_format(
        &mut self,
        linebuf: *mut c_char,
        linelen: usize,
        fmt: &mut Format,
        multiline: bool,
        multiscan: bool,
        tail: &mut *mut c_char,
    ) -> Status {
        // A re-scan of a tail is offered only to the file-name formats, and
        // leaves the fields the first scan set alone.
        if multiscan && !b"OPQ".contains(&fmt.prefix()) {
            return Status::Fail;
        }
        self.reset(multiscan, tail);

        // Case is always ignored when looking for an error.
        // SAFETY: the caller's line is NUL-terminated.
        let Some(regmatch) = (unsafe { fmt.exec(linebuf) }) else {
            return Status::Fail;
        };
        // SAFETY: the submatches point into the line just matched.
        unsafe { self.take_match(linebuf, linelen, fmt, &regmatch, multiline, multiscan, tail) }
    }

    /// Pull the values `fmt`'s conversions captured out of `regmatch`.
    ///
    /// Answers [`Status::Fail`] as soon as one conversion is unusable — a
    /// submatch that did not participate, a `%b` naming no buffer, a `%f`
    /// under `%O`/`%P`/`%Q` naming a file that does not exist.
    ///
    /// # Safety
    ///
    /// `regmatch`'s submatch pointers must point into `linebuf`, which must
    /// be writable, NUL-terminated and `linelen` bytes long.
    unsafe fn take_match(
        &mut self,
        linebuf: *mut c_char,
        linelen: usize,
        fmt: &Format,
        regmatch: &regmatch_T,
        multiline: bool,
        multiscan: bool,
        tail: &mut *mut c_char,
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
            // most nine conversions and this index is within `startp`.
            let midx = fmt.submatch(idx);
            let status = match idx {
                // SAFETY (each arm): the submatch is inside the line.
                0 if midx > 0 => unsafe { self.take_file(regmatch, midx, prefix) },
                FMT_PATTERN_M => {
                    if fmt.flags() == b'+' && !multiscan {
                        // %+ : the whole line is the message.
                        unsafe { self.set_message(linebuf, linelen) };
                        Status::Ok
                    } else if midx > 0 {
                        unsafe { self.take_message(regmatch, midx) }
                    } else {
                        Status::Ok
                    }
                }
                FMT_PATTERN_R if midx > 0 => {
                    // %r : whatever follows a file name, to be re-scanned.
                    if regmatch.startp[midx].is_null() {
                        Status::Fail
                    } else {
                        *tail = regmatch.startp[midx];
                        Status::Ok
                    }
                }
                _ if midx > 0 => unsafe { self.take_conversion(regmatch, midx, idx) },
                _ => Status::Ok,
            };
            if status != Status::Ok {
                return status;
            }
        }

        Status::Ok
    }

    /// `%f`: the file name, with `~/` and `$HOME/` expanded.
    ///
    /// # Safety
    ///
    /// The submatch must delimit a range of a writable, NUL-terminated line.
    unsafe fn take_file(&mut self, rmp: &regmatch_T, midx: usize, prefix: u8) -> Status {
        let (start, end) = (rmp.startp[midx], rmp.endp[midx]);
        if start.is_null() || end.is_null() {
            return Status::Fail;
        }
        // SAFETY: `end` points into the line, so terminating there makes
        // the match a C string for the length of the call; the byte it
        // displaced is put straight back.
        unsafe {
            let displaced = *end;
            *end = 0;
            expand_env(start, self.namebuf.as_mut_ptr(), CMDBUFFSIZE);
            *end = displaced;
            // A separate file-name format (%O, %P, %Q) only claims a line
            // when the file it names really exists.
            if b"OPQ".contains(&prefix) && !os_path_exists(self.namebuf.as_ptr()) {
                return Status::Fail;
            }
        }
        Status::Ok
    }

    /// `%m`: the error message.
    ///
    /// # Safety
    ///
    /// The submatch must delimit a range of a NUL-terminated line.
    unsafe fn take_message(&mut self, rmp: &regmatch_T, midx: usize) -> Status {
        let (start, end) = (rmp.startp[midx], rmp.endp[midx]);
        if start.is_null() || end.is_null() {
            return Status::Fail;
        }
        // SAFETY: both pointers are into the same line.
        let len = unsafe { end.offset_from(start) } as usize;
        // SAFETY: the source has at least `len` bytes before its NUL.
        unsafe { self.set_message(start, len) };
        Status::Ok
    }

    /// Every conversion that is neither `%f`, `%m` nor `%r`. `idx` indexes
    /// [`FMT_PAT`], so the arms below must stay in step with that table.
    ///
    /// # Safety
    ///
    /// The submatch must delimit a range of a NUL-terminated line.
    unsafe fn take_conversion(&mut self, rmp: &regmatch_T, midx: usize, idx: usize) -> Status {
        let start = rmp.startp[midx];
        if start.is_null() {
            return Status::Fail;
        }
        // The submatch begins inside a NUL-terminated line and `atol` stops
        // at the first byte that is not part of a number, so the end of the
        // match does not have to be marked.
        // SAFETY: `start` points into the line.
        let number = || unsafe { atol(start) };
        // SAFETY (each arm): `start`/`endp[midx]` are inside the line, and
        // the field buffers hold `FIELD_MAX + 1` bytes.
        unsafe {
            match idx {
                1 => {
                    // %b: a buffer number, which must name a live buffer.
                    let bnr = number() as c_int;
                    if buflist_findnr(bnr).is_null() {
                        return Status::Fail;
                    }
                    self.bnr = bnr;
                }
                2 => self.enr = number() as c_int,         // %n
                3 => self.lnum = number() as linenr_T,     // %l
                4 => self.end_lnum = number() as linenr_T, // %e
                5 => self.col = number() as c_int,         // %c
                6 => self.end_col = number() as c_int,     // %k
                7 => self.kind = *start,                   // %t
                10 => {
                    // %p: a pointer line such as "   ^", whose width is the
                    // column. A tab advances to the next multiple of eight.
                    let end = rmp.endp[midx];
                    if end.is_null() {
                        return Status::Fail;
                    }
                    self.col = 0;
                    let mut at = start;
                    while at != end {
                        self.col += 1;
                        if *at == TAB as c_char {
                            self.col += 7;
                            self.col -= self.col % 8;
                        }
                        at = at.add(1);
                    }
                    self.col += 1;
                    self.use_viscol = true;
                }
                11 => {
                    // %v: a screen column.
                    self.col = number() as c_int;
                    self.use_viscol = true;
                }
                12 => {
                    // %s: the matched text, as a very-nomagic pattern
                    // anchored at both ends. Five bytes go around it, so
                    // that much less of the match fits.
                    let end = rmp.endp[midx];
                    if end.is_null() {
                        return Status::Fail;
                    }
                    let len = (end.offset_from(start) as usize).min(FIELD_MAX - 5);
                    xstrlcpy(self.pattern.as_mut_ptr(), c"^\\V".as_ptr(), 4);
                    xstrlcat(self.pattern.as_mut_ptr(), start, len + 4);
                    self.pattern[len + 3] = b'\\' as c_char;
                    self.pattern[len + 4] = b'$' as c_char;
                    self.pattern[len + 5] = 0;
                }
                13 => {
                    // %o: the module name, appended to whatever is there.
                    let end = rmp.endp[midx];
                    if end.is_null() {
                        return Status::Fail;
                    }
                    let len = end.offset_from(start) as usize;
                    let dsize = (strlen(self.module.as_ptr()) + len + 1).min(FIELD_MAX);
                    xstrlcat(self.module.as_mut_ptr(), start, dsize);
                }
                _ => unreachable!("conversion {idx} is handled by its caller"),
            }
        }
        Status::Ok
    }
}

/// Match one line against the formats in `efm`, and act on what matched.
///
/// # Safety
///
/// `qfl` must be a live list; `linebuf` must be a writable, NUL-terminated
/// buffer of `linelen` bytes.
pub(crate) unsafe fn parse_line(
    qfl: *mut qf_list_T,
    linebuf: *mut c_char,
    linelen: usize,
    efm: &mut Efm,
    fields: &mut Fields,
) -> Status {
    let mut linebuf = linebuf;
    let mut linelen = linelen;
    // Not reset between scans: a `%r` from the first pass is still what a
    // later pass would re-scan if no format resets it.
    let mut tail: *mut c_char = ptr::null_mut();

    // A `%O`/`%P`/`%Q` match can leave a tail, which is then scanned as a
    // line of its own; that is the only way round this loop.
    loop {
        // SAFETY: the caller's list is live.
        let (multiline, multiscan) = unsafe { ((*qfl).qf_multiline, (*qfl).qf_multiscan) };
        fields.valid = true;

        // Start at the first format, or — after a `%>` — at the one that
        // matched last time.
        let mut matched = None;
        for idx in efm.take_resume()..efm.len() {
            // SAFETY: the line is writable and NUL-terminated.
            let status = unsafe {
                fields.try_format(
                    linebuf,
                    linelen,
                    efm.format(idx),
                    multiline,
                    multiscan,
                    &mut tail,
                )
            };
            if status == Status::Ok {
                matched = Some(idx);
                break;
            }
        }
        // SAFETY: the caller's list is live.
        unsafe { (*qfl).qf_multiscan = false };

        let Some(idx) = matched else {
            // Nothing matched: keep the line as a message, and close any
            // multi-line message that was open.
            // SAFETY: the line is readable for `linelen` bytes.
            unsafe { no_match(linebuf, linelen, fields) };
            // SAFETY: the caller's list is live.
            unsafe {
                (*qfl).qf_multiline = false;
                (*qfl).qf_multiignore = false;
            }
            return Status::Ok;
        };

        let (prefix, flags, conthere) = {
            let fmt = efm.format(idx);
            (fmt.prefix(), fmt.flags(), fmt.conthere())
        };

        if prefix == b'D' || prefix == b'X' {
            // SAFETY: the caller's list is live.
            let status = unsafe { push_pop_dir(prefix, fields, qfl) };
            if status != Status::Ok {
                return status;
            }
            // A directory line is never an entry of its own, but it is kept
            // as a message like any unmatched line.
            // SAFETY: the line is readable for `linelen` bytes.
            unsafe { no_match(linebuf, linelen, fields) };
            return Status::Ok;
        }

        // Honour a `%>` item: the next line starts matching here.
        if conthere {
            efm.set_resume(idx);
        }

        if b"AEWIN".contains(&prefix) {
            // SAFETY: the caller's list is live.
            unsafe {
                (*qfl).qf_multiline = true; // start of a multi-line message
                (*qfl).qf_multiignore = false; // reset continuation
            }
        } else if b"CZ".contains(&prefix) {
            // A continuation line never makes an entry of its own, so this
            // always ends the line.
            // SAFETY: the caller's list is live.
            return unsafe { continue_multiline(prefix, qfl, fields) };
        } else if b"OPQ".contains(&prefix) {
            // SAFETY: the caller's list is live; `tail` points into the
            // line when the format captured a `%r`.
            if unsafe { claim_file(prefix, fields, qfl, tail) } == Status::MultiScan {
                // SAFETY: `tail` points into the NUL-terminated line.
                let rest = unsafe { skipwhite(tail) };
                // SAFETY: ditto.
                let rest_len = unsafe { strlen(rest) };
                if rest_len >= linelen {
                    // The tail is no shorter than the line it came from, so
                    // re-scanning could not make progress.
                    return Status::Ignore;
                }
                linebuf = rest;
                linelen = rest_len;
                continue;
            }
        }

        if flags == b'-' {
            // Generally exclude this line.
            // SAFETY: the caller's list is live.
            if unsafe { (*qfl).qf_multiline } {
                // Exclude its continuation lines too.
                // SAFETY: ditto.
                unsafe { (*qfl).qf_multiignore = true };
            }
            return Status::Ignore;
        }

        return Status::Ok;
    }
}

/// A line that matched no format: keep it as the message, but do not let it
/// be jumped to.
///
/// # Safety
///
/// `linebuf` must be readable for `linelen` bytes.
unsafe fn no_match(linebuf: *const c_char, linelen: usize, fields: &mut Fields) {
    fields.namebuf[0] = 0; // no match found, so no file name
    fields.lnum = 0; // don't jump to this line
    fields.valid = false;
    // SAFETY: the caller's line is readable for `linelen` bytes.
    unsafe { fields.set_message(linebuf, linelen) };
}

/// `%D` and `%X`: enter and leave the directory the following file names
/// are relative to.
///
/// # Safety
///
/// `qfl` must be a live list.
unsafe fn push_pop_dir(prefix: u8, fields: &mut Fields, qfl: *mut qf_list_T) -> Status {
    // SAFETY: the caller's list is live; the name buffer is NUL-terminated.
    unsafe {
        if prefix == b'D' {
            if !fields.has_name() {
                emsg(gettext(c"E379: Missing or empty directory name".as_ptr()));
                return Status::Fail;
            }
            (*qfl).qf_directory =
                qf_push_dir(fields.namebuf(), &raw mut (*qfl).qf_dir_stack, false);
            if (*qfl).qf_directory.is_null() {
                return Status::Fail;
            }
        } else {
            (*qfl).qf_directory = qf_pop_dir(&raw mut (*qfl).qf_dir_stack);
        }
    }
    Status::Ok
}

/// `%O`, `%P` and `%Q`: name the file the following lines belong to.
///
/// # Safety
///
/// `qfl` must be a live list; `tail`, when not null, must point into a
/// NUL-terminated line.
unsafe fn claim_file(
    prefix: u8,
    fields: &mut Fields,
    qfl: *mut qf_list_T,
    tail: *const c_char,
) -> Status {
    fields.valid = false;
    // SAFETY: the caller's list is live; the name buffer is NUL-terminated.
    unsafe {
        if fields.has_name() && !os_path_exists(fields.namebuf.as_ptr()) {
            return Status::Ok;
        }
        if fields.has_name() && prefix == b'P' {
            (*qfl).qf_currfile = qf_push_dir(fields.namebuf(), &raw mut (*qfl).qf_file_stack, true);
        } else if prefix == b'Q' {
            (*qfl).qf_currfile = qf_pop_dir(&raw mut (*qfl).qf_file_stack);
        }
        fields.namebuf[0] = 0;
        if !tail.is_null() && *tail != 0 {
            (*qfl).qf_multiscan = true;
            return Status::MultiScan;
        }
    }
    Status::Ok
}

/// `%C` and `%Z`: fold a continuation line into the entry the opening line
/// of the multi-line message made.
///
/// # Safety
///
/// `qfl` must be a live list.
unsafe fn continue_multiline(prefix: u8, qfl: *mut qf_list_T, fields: &mut Fields) -> Status {
    // SAFETY: the caller's list is live, and `qf_last` is its last entry.
    unsafe {
        if !(*qfl).qf_multiignore {
            let prev = (*qfl).qf_last;
            if prev.is_null() {
                return Status::Fail;
            }
            if *fields.errmsg() != 0 {
                // Append the continuation as a new line of the message.
                let textlen = strlen((*prev).qf_text);
                let errlen = strlen(fields.errmsg());
                (*prev).qf_text = xrealloc((*prev).qf_text.cast(), textlen + errlen + 2).cast();
                *(*prev).qf_text.add(textlen) = b'\n' as c_char;
                xstrlcpy(
                    (*prev).qf_text.add(textlen + 1),
                    fields.errmsg(),
                    errlen + 1,
                );
            }
            if (*prev).qf_nr == -1 {
                (*prev).qf_nr = fields.enr;
            }
            if vim_isprintc(c_int::from(fields.kind)) && (*prev).qf_type == 0 {
                // Only printable characters allowed.
                (*prev).qf_type = fields.kind;
            }
            if (*prev).qf_lnum == 0 {
                (*prev).qf_lnum = fields.lnum;
            }
            if (*prev).qf_end_lnum == 0 {
                (*prev).qf_end_lnum = fields.end_lnum;
            }
            if (*prev).qf_col == 0 {
                (*prev).qf_col = fields.col;
                (*prev).qf_viscol = c_char::from(fields.use_viscol);
            }
            if (*prev).qf_end_col == 0 {
                (*prev).qf_end_col = fields.end_col;
            }
            if (*prev).qf_fnum == 0 {
                let name = entry_file_name(fields, qfl);
                (*prev).qf_fnum = qf_get_fnum(qfl, (*qfl).qf_directory, name);
            }
        }
        if prefix == b'Z' {
            (*qfl).qf_multiline = false;
            (*qfl).qf_multiignore = false;
        }
        line_breakcheck();
    }

    Status::Ignore
}

/// Which name the entry this line makes should be filed under: the one the
/// line gave, or the file a `%P` claimed, or none at all.
///
/// # Safety
///
/// `qfl` must be a live list.
pub(crate) unsafe fn entry_file_name(fields: &mut Fields, qfl: *mut qf_list_T) -> *mut c_char {
    // SAFETY: the caller's list is live.
    unsafe {
        if fields.has_name() || !(*qfl).qf_directory.is_null() {
            fields.namebuf()
        } else if !(*qfl).qf_currfile.is_null() && fields.valid {
            (*qfl).qf_currfile
        } else {
            ptr::null_mut()
        }
    }
}

impl Fields {
    /// The entry this parsed line makes. The strings stay in the field
    /// buffers; [`qf_add_entry`] copies what it keeps.
    ///
    /// # Safety
    ///
    /// `qfl` must be the live list the line was parsed against.
    pub(crate) unsafe fn entry(&mut self, qfl: *mut qf_list_T) -> NewEntry {
        // SAFETY: forwarded from the caller.
        unsafe {
            NewEntry {
                dir: (*qfl).qf_directory,
                fname: entry_file_name(self, qfl),
                module: self.module(),
                bufnum: self.bnr,
                mesg: self.errmsg(),
                lnum: self.lnum,
                end_lnum: self.end_lnum,
                col: self.col,
                end_col: self.end_col,
                vis_col: c_char::from(self.use_viscol),
                pattern: self.pattern(),
                nr: self.enr,
                kind: self.kind,
                user_data: self.user_data,
                valid: self.valid,
            }
        }
    }
}
