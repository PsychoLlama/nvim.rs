//! Reading the lines to be parsed, from wherever they come from.
//!
//! [`qf_init_ext`] is the driver every list-building command reaches. It
//! compiles `'errorformat'`, then pulls lines one at a time out of a
//! [`Reader`] and files what [`parse_line`] makes of each.
//!
//! A `Reader` covers all four sources — an error file, a range of buffer
//! lines, a Vimscript list and a Vimscript string — behind one
//! [`Reader::next_line`]. They share one growable line buffer, because the
//! file source is the awkward one: a line longer than a single `fgets` is
//! assembled by reading on into the same buffer, doubling it up to
//! [`LINE_MAXLEN`] and then throwing away whatever is left of an even
//! longer line. The file is read through [`Fgets`], which is C's `fgets`
//! over a `std::fs::File`, so that those limits fall exactly where
//! upstream's do.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::eval::typval::list_items;
use crate::mbyte::Converter;
use crate::memory::XString;
use crate::message_fmt::msg_bytes;
use crate::optionstr::OptString;
use crate::os::fs::stdin_file;
use crate::semsg;
use crate::types::{IOSIZE, ListRef, VAR_LIST, VAR_STRING};
use core::ffi::{CStr, c_int};
use std::ffi::CString;
use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind};
use std::os::unix::ffi::OsStrExt;

/// The most of one line that is kept. A longer line is truncated one byte
/// short of this, because that last byte is not the newline.
const LINE_MAXLEN: usize = 4096;

/// How much one `fgets` reads, and the smallest the line buffer ever is.
const READ_CHUNK: usize = IOSIZE as usize;

/// How long a list title can be: upstream builds titles in an `IOSIZE`
/// buffer.
pub(crate) const QF_TITLE_MAX: usize = IOSIZE as usize - 1;

/// C's `fgets` over a file: the same chunking, so that a long line is cut
/// where upstream cuts it.
pub(crate) struct Fgets {
    reader: BufReader<File>,
    /// A read failed with something other than an interrupt: `ferror()`.
    error: bool,
}

impl Fgets {
    pub(crate) fn new(file: File) -> Fgets {
        Fgets {
            reader: BufReader::new(file),
            error: false,
        }
    }

    /// Read at most `size - 1` bytes, stopping after a newline, into `out`
    /// (which is cleared first). `false` at the end of the file — or after
    /// an error — with nothing read, as `fgets` answers NULL.
    pub(crate) fn fgets(&mut self, out: &mut Vec<u8>, size: usize) -> bool {
        out.clear();
        let room = size.saturating_sub(1);
        while out.len() < room {
            let available = match self.reader.fill_buf() {
                Ok(available) => available,
                Err(err) if err.kind() == ErrorKind::Interrupted => continue,
                Err(_) => {
                    self.error = true;
                    break;
                }
            };
            if available.is_empty() {
                break;
            }
            let want = (room - out.len()).min(available.len());
            let (take, newline) = match available[..want].iter().position(|&b| b == b'\n') {
                Some(at) => (at + 1, true),
                None => (want, false),
            };
            out.extend_from_slice(&available[..take]);
            self.reader.consume(take);
            if newline {
                break;
            }
        }
        !out.is_empty()
    }

    /// Whether reading failed part-way: `ferror()`.
    pub(crate) fn had_error(&self) -> bool {
        self.error
    }
}

/// The length C's `strlen` gives a line just read: up to the first NUL.
fn c_len(bytes: &[u8]) -> usize {
    bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len())
}

/// Where the lines being read come from, as the caller names it.
pub(crate) enum Input<'a> {
    /// An error file, or standard input when it is named `-`.
    File(&'a CStr),
    /// Lines `first` to `last` of a buffer.
    Lines {
        buf: Buf,
        first: LineNr,
        last: LineNr,
    },
    /// A Vimscript string or list of strings.
    Value(&'a TypVal),
}

/// Where the lines being read come from.
enum Source {
    File(Fgets),
    Buffer {
        buf: Buf,
        lnum: LineNr,
        last: LineNr,
    },
    /// A Vimscript list, one entry per line; non-string entries are
    /// skipped. Held by a reference of its own, and walked by index: adding
    /// an entry can run an autocommand that changes the list.
    List(ListRef, usize),
    /// A Vimscript string, split on newlines: a copy, and how far into it
    /// the read is.
    Text(Vec<u8>, usize),
    /// A Vimscript value that is neither a string nor a list. Upstream
    /// reports this as a read failure rather than as end of input; no
    /// caller passes one, all three checking the type first.
    Unusable,
}

/// One read in progress: where the lines come from, the buffer they are
/// read into and the encoding conversion applied to each.
pub(crate) struct Reader {
    source: Source,
    /// The line last read, NUL-terminated. Reused between lines and never
    /// shrunk; always at least [`READ_CHUNK`] bytes.
    line: Vec<u8>,
    /// How many bytes of `line` the current line occupies. The trailing
    /// newline is counted even after [`Reader::next_line`] overwrites it,
    /// which is what upstream does.
    len: usize,
    /// How much of `line` the long-line reader may fill — upstream's
    /// `growbufsiz`. Zero until a line has needed more than one `fgets`;
    /// reaching [`LINE_MAXLEN`] is what makes the rest be discarded.
    room: usize,
    /// The conversion from the errorfile's encoding, or none.
    conv: Option<Converter>,
    /// One `fgets`' worth, before it is copied into place.
    chunk: Vec<u8>,
}

impl Reader {
    /// Open the source the caller named. Answers `None` — after reporting
    /// it — when the error file cannot be opened.
    fn open(enc: Option<&CStr>, input: Input) -> Option<Reader> {
        let conv = enc
            .filter(|enc| !enc.is_empty())
            .and_then(|enc| Converter::new(enc, P_ENC.get().as_cstr()));
        let source = match input {
            Input::Lines { buf, first, last } => Source::Buffer {
                buf,
                lnum: first,
                last,
            },
            Input::File(efile) => {
                let file = if efile == c"-" {
                    stdin_file()
                } else {
                    File::open(std::ffi::OsStr::from_bytes(efile.to_bytes())).ok()
                };
                let Some(file) = file else {
                    let efile = msg_bytes(efile.to_bytes());
                    semsg!("E40: Can't open errorfile {efile}");
                    return None;
                };
                Source::File(Fgets::new(file))
            }
            Input::Value(tv) => {
                if tv.v_type() == VAR_STRING as VarType {
                    Source::Text(crate::eval::list::string_bytes(tv).to_vec(), 0)
                } else if tv.v_type() == VAR_LIST as VarType {
                    match list_of(tv) {
                        Some(list) => Source::List(list, 0),
                        None => Source::Text(Vec::new(), 0),
                    }
                } else {
                    Source::Unusable
                }
            }
        };
        Some(Reader {
            source,
            line: vec![0; READ_CHUNK],
            len: 0,
            room: 0,
            conv,
            chunk: Vec::new(),
        })
    }

    /// The line just read, up to its terminator.
    fn line(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.line).expect("the line buffer is terminated")
    }

    /// Whether reading the error file failed part-way through.
    fn had_error(&self) -> bool {
        match &self.source {
            Source::File(file) => file.had_error(),
            _ => false,
        }
    }

    /// Make room for a line of `want` bytes and a NUL, answering how many
    /// of those bytes are actually kept: a line longer than [`LINE_MAXLEN`]
    /// is cut one byte short of it, because that byte is not the newline.
    fn fit(&mut self, want: usize) -> usize {
        let len = if want > LINE_MAXLEN {
            LINE_MAXLEN - 1
        } else {
            want
        };
        if self.line.len() < len + 1 {
            self.line.resize(len + 1, 0);
        }
        len
    }

    /// Put `bytes` in the line buffer as the current line.
    fn set_line(&mut self, bytes: &[u8]) {
        self.len = self.fit(bytes.len());
        let len = self.len;
        // `xstrlcpy`: up to the first NUL of the source.
        let copied = c_len(&bytes[..len]);
        self.line[..copied].copy_from_slice(&bytes[..copied]);
        self.line[copied] = 0;
    }

    /// Read the next line from whichever source this is, strip its trailing
    /// newline and any byte-order mark.
    fn next_line(&mut self) -> Status {
        let status = match self.source {
            Source::File(_) => self.read_file(),
            Source::Buffer { .. } => self.read_buffer(),
            Source::List(..) => self.read_list(),
            Source::Text(..) => self.read_text(),
            Source::Unusable => Status::Fail,
        };
        if status != Status::Ok {
            return status;
        }
        // The length still counts the newline; upstream only overwrites it.
        if self.len > 0 && self.line[self.len - 1] == b'\n' {
            self.line[self.len - 1] = 0;
        }
        // A UTF-8 byte-order mark at the start of the line goes.
        if self.line.starts_with(b"\xef\xbb\xbf") {
            let end = c_len(&self.line);
            self.line.copy_within(3..=end, 0);
        }
        Status::Ok
    }

    /// One line from a Vimscript string, up to and including its newline.
    fn read_text(&mut self) -> Status {
        let Source::Text(text, at) = &mut self.source else {
            unreachable!()
        };
        let rest = &text[*at..];
        let rest = &rest[..c_len(rest)];
        if rest.is_empty() {
            return Status::EndOfInput;
        }
        let want = rest
            .iter()
            .position(|&b| b == b'\n')
            .map_or(rest.len(), |nl| nl + 1);
        let line = rest[..want].to_vec();
        // Advance by the whole line, so that the part of an over-long line
        // that did not fit is discarded rather than re-read.
        *at += want;
        self.len = self.fit(want);
        let len = self.len;
        self.line[..len].copy_from_slice(&line[..len]);
        self.line[len] = 0;
        Status::Ok
    }

    /// One line from a Vimscript list. Entries that are not strings are
    /// skipped.
    fn read_list(&mut self) -> Status {
        let Source::List(list, at) = &mut self.source else {
            unreachable!()
        };
        let items = list_items(Some(list));
        while items
            .get(*at)
            .is_some_and(|li| li.li_tv.string_ref().is_none())
        {
            *at += 1;
        }
        let Some(item) = items.get(*at) else {
            return Status::EndOfInput;
        };
        let text = crate::eval::list::string_bytes(&item.li_tv).to_vec();
        *at += 1;
        self.set_line(&text);
        Status::Ok
    }

    /// One line of the buffer range.
    fn read_buffer(&mut self) -> Status {
        let Source::Buffer { buf, lnum, last } = &mut self.source else {
            unreachable!()
        };
        if *lnum > *last {
            return Status::EndOfInput;
        }
        let text = buf.lines().line_cstr(*lnum, 0).to_bytes().to_vec();
        *lnum += 1;
        self.set_line(&text);
        Status::Ok
    }

    /// One `fgets` of at most `size - 1` bytes into the line buffer at
    /// `at`. `false` at the end of the file.
    fn fgets_at(&mut self, at: usize, size: usize) -> bool {
        let Source::File(file) = &mut self.source else {
            unreachable!()
        };
        if !file.fgets(&mut self.chunk, size) {
            return false;
        }
        let read = self.chunk.len();
        if self.line.len() < at + read + 1 {
            self.line.resize(at + read + 1, 0);
        }
        self.line[at..at + read].copy_from_slice(&self.chunk);
        self.line[at + read] = 0;
        true
    }

    /// One line from the error file.
    ///
    /// A line that does not fit a single `fgets` is assembled by reading on
    /// into the same buffer, which doubles up to [`LINE_MAXLEN`]; past that
    /// the rest of the line is read and thrown away.
    fn read_file(&mut self) -> Status {
        if !self.fgets_at(0, READ_CHUNK) {
            return Status::EndOfInput;
        }
        self.len = c_len(&self.line);
        if self.len != READ_CHUNK - 1 || self.line[self.len - 1] == b'\n' {
            self.convert();
            return Status::Ok;
        }

        // The line filled the chunk without ending: keep going.
        if self.room == 0 {
            self.room = 2 * (READ_CHUNK - 1);
        }
        if self.line.len() < self.room {
            self.line.resize(self.room, 0);
        }
        let mut filled = self.len;
        let mut discard = false;
        loop {
            if !self.fgets_at(filled, self.room - filled) {
                break;
            }
            self.len = c_len(&self.line[filled..]);
            filled += self.len;
            if self.line[filled - 1] == b'\n' {
                break;
            }
            if self.room == LINE_MAXLEN {
                discard = true;
                break;
            }
            self.room = (2 * self.room).min(LINE_MAXLEN);
            if self.line.len() < self.room {
                self.line.resize(self.room, 0);
            }
        }
        if discard {
            // Read on, keeping nothing, until the line ends or the file
            // does. This must not use the line buffer: it still holds the
            // 4095 bytes that were kept.
            let Source::File(file) = &mut self.source else {
                unreachable!()
            };
            let mut scrap = Vec::new();
            while file.fgets(&mut scrap, READ_CHUNK) {
                let len = c_len(&scrap);
                if len < READ_CHUNK - 1 || scrap.get(READ_CHUNK - 2) == Some(&b'\n') {
                    break;
                }
            }
        }
        self.len = filled;
        self.convert();
        Status::Ok
    }

    /// Convert the line just read out of the error file's encoding, if one
    /// was given and the line is not plain ASCII.
    fn convert(&mut self) {
        let Some(conv) = &self.conv else {
            return;
        };
        let text = &self.line[..c_len(&self.line[..self.len.min(self.line.len())])];
        if text.is_ascii() {
            return;
        }
        let Some(converted) = conv.convert(&self.line[..self.len]) else {
            return;
        };
        self.len = converted.len();
        if self.line.len() < self.len + 1 {
            self.line.resize(self.len + 1, 0);
        }
        let copied = c_len(&converted);
        self.line[..copied].copy_from_slice(&converted[..copied]);
        self.line[copied] = 0;
        // Upstream adopts the converted allocation outright when the line no
        // longer fits one chunk, and records how much of it the long-line
        // reader may fill. Only that cap matters here, since the buffer is
        // owned either way.
        if self.len >= READ_CHUNK {
            self.room = self.room.max(self.len.min(LINE_MAXLEN));
        }
    }
}

/// Read the error file `efile` into memory line by line, building the error
/// list, and set its title to `qf_title`.
///
/// `window` is `None` for the quickfix stack, which belongs to no window.
pub fn qf_init(
    window: Option<Win>,
    efile: &CStr,
    errorformat: &CStr,
    global_efm: bool,
    newlist: bool,
    qf_title: Option<&CStr>,
    enc: Option<&CStr>,
) -> c_int {
    let qi = match window {
        Some(wp) => wp.location_list_or_new(),
        None => Qi::global(),
    };
    qf_init_ext(
        qi,
        qi.current,
        Input::File(efile),
        Some(Buf::current()),
        errorformat,
        global_efm,
        newlist,
        qf_title,
        enc,
    )
}

/// The compiled `'errorformat'`, kept between calls together with the
/// option text it was compiled from, so that a repeated command does not
/// recompile it.
static EFM_CACHE: GlobalCell<Option<(Vec<u8>, Efm)>> = GlobalCell::new(None);

/// Build a quickfix list out of an error file, a buffer range, or a
/// Vimscript string or list.
///
/// `newlist` starts a new list rather than adding to list `qf_idx`.
/// `efm_buf` is the buffer whose own `'errorformat'` replaces
/// `errorformat` when `global_efm` says that is the global value. Answers
/// the number of entries, or −1 on failure.
#[allow(clippy::too_many_arguments)]
pub(crate) fn qf_init_ext(
    mut qi: Qi,
    mut qf_idx: c_int,
    input: Input,
    efm_buf: Option<Buf>,
    errorformat: &CStr,
    global_efm: bool,
    newlist: bool,
    qf_title: Option<&CStr>,
    enc: Option<&CStr>,
) -> c_int {
    // Do not use the cached buffer, it may have been wiped out.
    forget_last_buffer();

    let mut old_last = None;
    let mut retval = -1;
    let from_value = matches!(input, Input::Value(_));
    let reader = Reader::open(enc, input);

    if let Some(mut reader) = reader {
        let mut adding = false;
        let qfl = if newlist || qf_idx == qi.list_count {
            // Make place for a new list.
            qf_new_list(qi, qf_title);
            qf_idx = qi.current;
            qi.slot(qf_idx)
        } else {
            // Adding to an existing list; remember its last entry.
            adding = true;
            let qfl = qi.slot(qf_idx);
            if !qfl.is_empty() {
                old_last = Some(qfl.entries.len() - 1);
            }
            qfl
        };

        // Use the buffer-local 'errorformat' when the caller asked for the
        // global one and the buffer has its own.
        let local_efm = if global_efm && !from_value {
            efm_buf
                .map(|buf| buf.b_p_efm.get())
                .filter(|efm| !efm.is_empty())
        } else {
            None
        };
        let efm: &CStr = local_efm.as_ref().map_or(errorformat, XString::as_cstr);

        // Take the compiled option out of the cache for the length of
        // this read and put it back at the end. Adding an entry can
        // fire `BufNew`, an autocommand can run another `:cexpr`, and
        // upstream — which keeps the compiled option in a bare static
        // and frees it whenever the option text changes — would then
        // free what this loop is still walking. Owning it here costs
        // the re-entrant call a recompile and nothing otherwise.
        let mut compiled = EFM_CACHE.take();
        let text = efm.to_bytes();
        if !compiled.as_ref().is_some_and(|(had, _)| had == text) {
            compiled = Efm::compile(efm).map(|parsed| (text.to_vec(), parsed));
        }
        let built = match compiled.as_mut() {
            Some((_, parsed)) => read_lines(qfl, &mut reader, parsed),
            None => false,
        };
        EFM_CACHE.set(compiled);

        if built {
            retval = qfl.count();
        } else if !adding {
            // The new list came to nothing; free it again.
            qf_free(qfl);
            qi.list_count -= 1;
            if qi.current > 0 {
                qi.current -= 1;
            }
        }
    }

    if qf_idx == qi.current {
        qf_update_buffer(qi, old_last);
    }
    retval
}

/// Read every line the reader has and add an entry for each one the parser
/// accepts. Answers whether the list was built: a read error, or a line the
/// parser rejected outright, means it was not.
fn read_lines(mut qfl: Qfl, reader: &mut Reader, efm: &mut Efm) -> bool {
    let mut fields = Fields::new();
    // `got_int` is reset here because it was probably set when killing the
    // ":make" command, and the error file should still be read.
    got_int.set(false);
    while !got_int.get() {
        match reader.next_line() {
            Status::EndOfInput => break,
            Status::Ok => {}
            _ => return false,
        }
        let parsed = parse_line(qfl, reader.line(), reader.len, efm, &mut fields);
        if parsed == Status::Fail {
            return false;
        }
        if parsed == Status::Ok {
            let entry = fields.entry(qfl);
            qf_add_entry(qfl, &entry);
        }
        line_breakcheck();
    }

    if reader.had_error() {
        emsg(gettext(e_readerrf));
        return false;
    }
    if qfl.index == 0 {
        // No valid entry was found.
        qfl.cursor = 0;
        qfl.index = 1;
        qfl.no_valid = true;
    } else {
        qfl.no_valid = false;
    }
    true
}

/// A list's default title is the command that created it, with a `:` in
/// front. Upstream answers a shared buffer the next call overwrites.
pub(crate) fn qf_cmdtitle(cmd: &[u8]) -> CString {
    // The `snprintf(":%s")` upstream writes: one colon and as much of the
    // command as fits before the terminator.
    let mut title = Vec::with_capacity(cmd.len() + 1);
    title.push(b':');
    title.extend(cmd.iter().take(READ_CHUNK - 1).take_while(|&&b| b != 0));
    CString::new(title).expect("no NUL was copied")
}
