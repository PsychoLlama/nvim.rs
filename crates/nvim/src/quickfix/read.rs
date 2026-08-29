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
//! longer line.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::semsg_c;
use crate::types::{CONV_NONE, IOSIZE, VAR_LIST, VAR_STRING};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// The most of one line that is kept. A longer line is truncated one byte
/// short of this, because that last byte is not the newline.
const LINE_MAXLEN: usize = 4096;

/// How much one `fgets` reads, and the smallest the line buffer ever is.
/// Keeping the buffer at least this big is what lets a converted line be
/// copied back into it without a bounds check, exactly as upstream relies
/// on its shared `IObuff` being this size.
const READ_CHUNK: usize = IOSIZE as usize;

/// Where the lines being read come from.
enum Source {
    /// An error file, or standard input when it was named `-`.
    File(*mut FILE),
    /// A range of lines in a buffer.
    Buffer {
        buf: Buf,
        lnum: linenr_T,
        last: linenr_T,
    },
    /// A Vimscript list, one entry per line; non-string entries are
    /// skipped.
    List(*mut listitem_T),
    /// A Vimscript string, split on newlines.
    Text(*mut c_char),
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
    line: Vec<c_char>,
    /// How many bytes of `line` the current line occupies. The trailing
    /// newline is counted even after [`Reader::next_line`] overwrites it,
    /// which is what upstream does.
    len: usize,
    /// How much of `line` the long-line reader may fill — upstream's
    /// `growbufsiz`. Zero until a line has needed more than one `fgets`;
    /// reaching [`LINE_MAXLEN`] is what makes the rest be discarded.
    room: usize,
    /// The conversion from the errorfile's encoding, or none.
    vc: vimconv_T,
}

impl Drop for Reader {
    fn drop(&mut self) {
        // SAFETY: the file is this reader's own, and the conversion
        // descriptor is torn down exactly once.
        if let Source::File(fd) = self.source
            && !fd.is_null()
        {
            unsafe { fclose(fd) };
        }
        if self.vc.vc_type != CONV_NONE {
            unsafe { convert_setup(&raw mut self.vc, ptr::null_mut(), ptr::null_mut()) };
        }
    }
}

impl Reader {
    /// Open the source the caller named: the error file `efile`, else the
    /// string or list in `tv`, else lines `lnumfirst` to `lnumlast` of
    /// `buf`. Answers `None` — after reporting it — when the error file
    /// cannot be opened.
    ///
    /// # Safety
    ///
    /// The pointers must be null or point at live objects; `enc` and
    /// `efile` must be NUL-terminated.
    unsafe fn open(
        enc: *mut c_char,
        efile: *const c_char,
        tv: *mut typval_T,
        buf: Option<Buf>,
        lnumfirst: linenr_T,
        lnumlast: linenr_T,
    ) -> Option<Reader> {
        let mut reader = Reader {
            // Without a buffer the source is one of the two set below; a
            // caller that named none of the three gets the read failure
            // `Unusable` is, rather than a walk off a null buffer.
            source: match buf {
                Some(buf) => Source::Buffer {
                    buf,
                    lnum: lnumfirst,
                    last: lnumlast,
                },
                None => Source::Unusable,
            },
            line: vec![0; READ_CHUNK],
            len: 0,
            room: 0,
            vc: vimconv_T {
                vc_type: CONV_NONE,
                vc_factor: 0,
                vc_fd: ptr::null_mut(),
                vc_fail: false,
            },
        };
        // SAFETY: the caller's strings are NUL-terminated.
        if !enc.is_null() && unsafe { *enc } != 0 {
            unsafe { convert_setup(&raw mut reader.vc, enc, p_enc.get()) };
        }
        if !efile.is_null() {
            let fd = if unsafe { strequal(efile, c"-".as_ptr()) } {
                unsafe { fdopen(os_open_stdin_fd(), c"r".as_ptr()) }
            } else {
                unsafe { os_fopen(efile, c"r".as_ptr()) }
            };
            if fd.is_null() {
                unsafe { semsg_c!(gettext(e_openerrf), efile) };
                // Dropping tears the conversion down again.
                return None;
            }
            reader.source = Source::File(fd);
        } else if !tv.is_null() {
            reader.source = if unsafe { (*tv).v_type } == VAR_STRING as VarType {
                Source::Text(unsafe { (*tv).vval.v_string })
            } else if unsafe { (*tv).v_type } == VAR_LIST as VarType {
                Source::List(unsafe { tv_list_first((*tv).vval.v_list) })
            } else {
                Source::Unusable
            };
        }
        Some(reader)
    }

    /// The line just read.
    fn line(&mut self) -> *mut c_char {
        self.line.as_mut_ptr()
    }

    /// Whether reading the error file failed part-way through.
    fn had_error(&self) -> bool {
        match self.source {
            // SAFETY: the file is this reader's own and still open.
            Source::File(fd) => unsafe { ferror(fd) != 0 },
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

    /// Read the next line from whichever source this is, strip its trailing
    /// newline and any byte-order mark.
    ///
    /// # Safety
    ///
    /// The source must still be live: the file open, the buffer loaded, the
    /// Vimscript value not yet freed.
    unsafe fn next_line(&mut self) -> Status {
        let status = match self.source {
            // SAFETY: forwarded from the caller.
            Source::File(fd) => unsafe { self.read_file(fd) },
            Source::Buffer { .. } => unsafe { self.read_buffer() },
            Source::List(_) => unsafe { self.read_list() },
            Source::Text(_) => unsafe { self.read_text() },
            Source::Unusable => Status::Fail,
        };
        if status != Status::Ok {
            return status;
        }
        // The length still counts the newline; upstream only overwrites it.
        // SAFETY: `len` bytes of the buffer are initialised.
        if self.len > 0 && self.line[self.len - 1] == b'\n' as c_char {
            self.line[self.len - 1] = 0;
        }
        unsafe { remove_bom(self.line()) };
        Status::Ok
    }

    /// One line from a Vimscript string, up to and including its newline.
    ///
    /// # Safety
    ///
    /// The string must be NUL-terminated and still allocated.
    unsafe fn read_text(&mut self) -> Status {
        let Source::Text(at) = self.source else {
            unreachable!()
        };
        // SAFETY: the caller's string is NUL-terminated.
        if unsafe { *at } == 0 {
            return Status::EndOfInput;
        }
        let newline = unsafe { vim_strchr(at, c_int::from(b'\n')) };
        let want = if newline.is_null() {
            unsafe { strlen(at) }
        } else {
            unsafe { newline.offset_from(at) as usize + 1 }
        };
        self.len = self.fit(want);
        unsafe { ptr::copy_nonoverlapping(at, self.line(), self.len) };
        self.line[self.len] = 0;
        // Advance by the whole line, so that the part of an over-long
        // line that did not fit is discarded rather than re-read.
        self.source = Source::Text(unsafe { at.add(want) });
        Status::Ok
    }

    /// One line from a Vimscript list. Entries that are not strings are
    /// skipped.
    ///
    /// # Safety
    ///
    /// The list items must still be allocated.
    unsafe fn read_list(&mut self) -> Status {
        let Source::List(mut at) = self.source else {
            unreachable!()
        };
        // SAFETY: the caller's list is live.
        while !at.is_null()
            && (unsafe { (*at).li_tv.v_type } != VAR_STRING as VarType
                || unsafe { (*at).li_tv.vval.v_string }.is_null())
        {
            at = unsafe { (*at).li_next };
        }
        if at.is_null() {
            self.source = Source::List(ptr::null_mut());
            return Status::EndOfInput;
        }
        let text = unsafe { (*at).li_tv.vval.v_string };
        self.len = self.fit(unsafe { strlen(text) });
        unsafe { xstrlcpy(self.line(), text, self.len + 1) };
        self.source = Source::List(unsafe { (*at).li_next });
        Status::Ok
    }

    /// One line of the buffer range.
    ///
    /// # Safety
    ///
    /// The buffer must be loaded.
    unsafe fn read_buffer(&mut self) -> Status {
        let Source::Buffer { buf, lnum, last } = self.source else {
            unreachable!()
        };
        if lnum > last {
            return Status::EndOfInput;
        }
        // SAFETY: the caller's buffer is loaded and `lnum` is within it.
        let text = unsafe { ml_get_buf(buf.raw(), lnum) };
        self.len = self.fit(unsafe { ml_get_buf_len(buf.raw(), lnum) } as usize);
        unsafe { xstrlcpy(self.line(), text, self.len + 1) };
        self.source = Source::Buffer {
            buf,
            lnum: lnum + 1,
            last,
        };
        Status::Ok
    }

    /// One line from the error file.
    ///
    /// A line that does not fit a single `fgets` is assembled by reading on
    /// into the same buffer, which doubles up to [`LINE_MAXLEN`]; past that
    /// the rest of the line is read and thrown away.
    ///
    /// # Safety
    ///
    /// `fd` must be this reader's open file.
    unsafe fn read_file(&mut self, fd: *mut FILE) -> Status {
        // SAFETY: the file is open and the buffer is at least READ_CHUNK.
        loop {
            unsafe { *__errno_location() = 0 };
            if !unsafe { fgets(self.line(), READ_CHUNK as c_int, fd) }.is_null() {
                break;
            }
            if unsafe { *__errno_location() } != EINTR {
                return Status::EndOfInput;
            }
        }
        self.len = unsafe { strlen(self.line.as_ptr()) };
        if self.len != READ_CHUNK - 1 || self.line[self.len - 1] == b'\n' as c_char {
            return unsafe { self.convert() };
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
            unsafe { *__errno_location() = 0 };
            let room = (self.room - filled) as c_int;
            if unsafe { fgets(self.line().add(filled), room, fd) }.is_null() {
                if unsafe { *__errno_location() } != EINTR {
                    break;
                }
                continue;
            }
            self.len = unsafe { strlen(self.line.as_ptr().add(filled)) };
            filled += self.len;
            if self.line[filled - 1] == b'\n' as c_char {
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
            // does. This must not use the line buffer: it still holds
            // the 4095 bytes that were kept.
            let mut scrap = [0 as c_char; READ_CHUNK];
            loop {
                unsafe { *__errno_location() = 0 };
                if unsafe { fgets(scrap.as_mut_ptr(), READ_CHUNK as c_int, fd) }.is_null() {
                    if unsafe { *__errno_location() } != EINTR {
                        break;
                    }
                } else if unsafe { strlen(scrap.as_ptr()) } < READ_CHUNK - 1
                    || scrap[READ_CHUNK - 2] == b'\n' as c_char
                {
                    break;
                }
            }
        }
        self.len = filled;
        unsafe { self.convert() }
    }

    /// Convert the line just read out of the error file's encoding, if one
    /// was given and the line is not plain ASCII.
    ///
    /// # Safety
    ///
    /// `len` bytes of the buffer must be initialised and NUL-terminated.
    unsafe fn convert(&mut self) -> Status {
        if self.vc.vc_type == CONV_NONE {
            return Status::Ok;
        }
        // SAFETY: the line is NUL-terminated; `string_convert` answers a
        // freshly allocated string and writes back its length.
        if !unsafe { has_non_ascii(self.line.as_ptr()) } {
            return Status::Ok;
        }
        let converted =
            unsafe { string_convert(&raw const self.vc, self.line(), &raw mut self.len) };
        if converted.is_null() {
            return Status::Ok;
        }
        if self.line.len() < self.len + 1 {
            self.line.resize(self.len + 1, 0);
        }
        unsafe { xstrlcpy(self.line(), converted, self.len + 1) };
        unsafe { xfree(converted.cast()) };
        // Upstream adopts the converted allocation outright when the
        // line no longer fits one chunk, and records how much of it the
        // long-line reader may fill. Only that cap matters here, since
        // the buffer is owned either way.
        if self.len >= READ_CHUNK {
            self.room = self.room.max(self.len.min(LINE_MAXLEN));
        }
        Status::Ok
    }
}

/// Read the error file `efile` into memory line by line, building the error
/// list, and set its title to `qf_title`.
///
/// `wp` is `None` for the quickfix stack, which belongs to no window.
///
/// # Safety
///
/// The strings must be NUL-terminated.
pub unsafe fn qf_init(
    wp: Option<Win>,
    efile: *const c_char,
    errorformat: *mut c_char,
    newlist: c_int,
    qf_title: *const c_char,
    enc: *mut c_char,
) -> c_int {
    // SAFETY: forwarded from the caller.
    let qi = match wp {
        Some(wp) => ll_get_or_alloc_list(wp),
        None => QfStack::Global.raw(),
    };
    debug_assert!(!qi.is_null());
    unsafe {
        qf_init_ext(
            qi,
            (*qi).qf_curlist,
            efile,
            Some(cur_buf()),
            ptr::null_mut(),
            errorformat,
            newlist != 0,
            0,
            0,
            qf_title,
            enc,
        )
    }
}

/// The compiled `'errorformat'`, kept between calls together with the
/// option text it was compiled from, so that a repeated command does not
/// recompile it.
static EFM_CACHE: GlobalCell<Option<(Vec<u8>, Efm)>> = GlobalCell::new(None);

/// Build a quickfix list out of an error file, a buffer range, or a
/// Vimscript string or list.
///
/// `efile` names an error file; when it is null the lines come from `tv`,
/// or from lines `lnumfirst` to `lnumlast` of `buf`. `newlist` starts a new
/// list rather than adding to list `qf_idx`. Answers the number of entries,
/// or −1 on failure.
///
/// # Safety
///
/// `qi` must be a live stack; the other pointers must be null or live, and
/// the strings NUL-terminated.
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn qf_init_ext(
    qi: *mut qf_info_T,
    mut qf_idx: c_int,
    efile: *const c_char,
    buf: Option<Buf>,
    tv: *mut typval_T,
    errorformat: *mut c_char,
    newlist: bool,
    lnumfirst: linenr_T,
    lnumlast: linenr_T,
    qf_title: *const c_char,
    enc: *mut c_char,
) -> c_int {
    // SAFETY: forwarded from the caller.
    // Do not use the cached buffer, it may have been wiped out.
    forget_last_buffer();

    let mut old_last: *mut qfline_T = ptr::null_mut();
    let mut retval = -1;
    let reader = unsafe { Reader::open(enc, efile, tv, buf, lnumfirst, lnumlast) };

    if let Some(mut reader) = reader {
        let mut adding = false;
        let qfl = if newlist || qf_idx == unsafe { (*qi).qf_listcount } {
            // Make place for a new list.
            unsafe { qf_new_list(qi, qf_title) };
            qf_idx = unsafe { (*qi).qf_curlist };
            unsafe { qf_get_list(qi, qf_idx) }
        } else {
            // Adding to an existing list; remember its last entry.
            adding = true;
            let qfl = unsafe { qf_get_list(qi, qf_idx) };
            if !unsafe { qf_list_empty(qfl) } {
                old_last = unsafe { (*qfl).qf_last };
            }
            qfl
        };

        // Use the buffer-local 'errorformat' when it has one.
        // The two cheap tests stay in front of the buffer's option, as
        // C's `&&` chain had them.
        let local_efm = if errorformat == p_efm.get() && tv.is_null() {
            buf.map(|buf| buf.b_p_efm)
                .filter(|&efm| unsafe { *efm } != 0)
        } else {
            None
        };
        let efm = local_efm.unwrap_or(errorformat);

        // Take the compiled option out of the cache for the length of
        // this read and put it back at the end. Adding an entry can
        // fire `BufNew`, an autocommand can run another `:cexpr`, and
        // upstream — which keeps the compiled option in a bare static
        // and frees it whenever the option text changes — would then
        // free what this loop is still walking. Owning it here costs
        // the re-entrant call a recompile and nothing otherwise.
        let mut compiled = EFM_CACHE.take();
        let text = unsafe { CStr::from_ptr(efm) }.to_bytes();
        if !compiled.as_ref().is_some_and(|(had, _)| had == text) {
            compiled = unsafe { Efm::compile(efm) }.map(|parsed| (text.to_vec(), parsed));
        }
        let built = match compiled.as_mut() {
            Some((_, parsed)) => unsafe { read_lines(qfl, &mut reader, parsed) },
            None => false,
        };
        EFM_CACHE.set(compiled);

        if built {
            retval = unsafe { (*qfl).qf_count };
        } else if !adding {
            // The new list came to nothing; free it again.
            unsafe { qf_free(qfl) };
            unsafe { (*qi).qf_listcount -= 1 };
            if unsafe { (*qi).qf_curlist } > 0 {
                unsafe { (*qi).qf_curlist -= 1 };
            }
        }
    }

    if qf_idx == unsafe { (*qi).qf_curlist } {
        unsafe { qf_update_buffer(qi, old_last) };
    }
    retval
}

/// Read every line the reader has and add an entry for each one the parser
/// accepts. Answers whether the list was built: a read error, or a line the
/// parser rejected outright, means it was not.
///
/// # Safety
///
/// `qfl` must be a live list and `reader` open on a live source.
unsafe fn read_lines(qfl: *mut qf_list_T, reader: &mut Reader, efm: &mut Efm) -> bool {
    let mut fields = Fields::new();
    // `got_int` is reset here because it was probably set when killing the
    // ":make" command, and the error file should still be read.
    got_int.set(false);
    // SAFETY: forwarded from the caller.
    while !got_int.get() {
        match unsafe { reader.next_line() } {
            Status::EndOfInput => break,
            Status::Ok => {}
            _ => return false,
        }
        let parsed = unsafe { parse_line(qfl, reader.line(), reader.len, efm, &mut fields) };
        if parsed == Status::Fail {
            return false;
        }
        if parsed == Status::Ok {
            unsafe { qf_add_entry(qfl, &fields.entry(qfl)) };
        }
        line_breakcheck();
    }

    if reader.had_error() {
        emsg(gettext(e_readerrf));
        return false;
    }
    if unsafe { (*qfl).qf_index } == 0 {
        // No valid entry was found.
        unsafe { (*qfl).qf_ptr = (*qfl).qf_start };
        unsafe { (*qfl).qf_index = 1 };
        unsafe { (*qfl).qf_nonevalid = true };
    } else {
        unsafe { (*qfl).qf_nonevalid = false };
        if unsafe { (*qfl).qf_ptr }.is_null() {
            unsafe { (*qfl).qf_ptr = (*qfl).qf_start };
        }
    }
    true
}

/// Set a list's title, replacing whatever it had.
///
/// # Safety
///
/// `qfl` must be a live list and `title` null or NUL-terminated.
pub(crate) unsafe fn qf_store_title(qfl: *mut qf_list_T, title: *const c_char) {
    // SAFETY: forwarded from the caller.
    unsafe { xfree((*qfl).qf_title.cast()) };
    unsafe { (*qfl).qf_title = ptr::null_mut() };
    if title.is_null() {
        return;
    }
    let len = unsafe { strlen(title) } + 1;
    let p: *mut c_char = unsafe { xmallocz(len) }.cast();
    unsafe { (*qfl).qf_title = p };
    unsafe { xstrlcpy(p, title, len + 1) };
}

/// A list's default title is the command that created it, with a `:` in
/// front, in its own storage. Upstream answers a shared buffer the next
/// call overwrites.
///
/// # Safety
///
/// `cmd` must be NUL-terminated.
pub(crate) unsafe fn qf_cmdtitle(cmd: *const c_char) -> [c_char; READ_CHUNK + 1] {
    let mut title = [0 as c_char; READ_CHUNK + 1];
    // SAFETY: the caller's command is NUL-terminated, and the buffer holds
    // the IOSIZE bytes `snprintf` is told about.
    unsafe { snprintf(title.as_mut_ptr(), READ_CHUNK, c":%s".as_ptr(), cmd) };
    title
}
