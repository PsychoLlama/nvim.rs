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

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::types::{VAR_LIST, VAR_STRING};
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
        buf: *mut buf_T,
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
        unsafe {
            if let Source::File(fd) = self.source
                && !fd.is_null()
            {
                fclose(fd);
            }
            if self.vc.vc_type != CONV_NONE as c_int {
                convert_setup(&raw mut self.vc, ptr::null_mut(), ptr::null_mut());
            }
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
        buf: *mut buf_T,
        lnumfirst: linenr_T,
        lnumlast: linenr_T,
    ) -> Option<Reader> {
        let mut reader = Reader {
            source: Source::Buffer {
                buf,
                lnum: lnumfirst,
                last: lnumlast,
            },
            line: vec![0; READ_CHUNK],
            len: 0,
            room: 0,
            vc: vimconv_T {
                vc_type: CONV_NONE as c_int,
                vc_factor: 0,
                vc_fd: ptr::null_mut(),
                vc_fail: false,
            },
        };
        // SAFETY: the caller's strings are NUL-terminated.
        unsafe {
            if !enc.is_null() && *enc != 0 {
                convert_setup(&raw mut reader.vc, enc, p_enc.get());
            }
            if !efile.is_null() {
                let fd = if strequal(efile, c"-".as_ptr()) {
                    fdopen(os_open_stdin_fd(), c"r".as_ptr())
                } else {
                    os_fopen(efile, c"r".as_ptr())
                };
                if fd.is_null() {
                    semsg(gettext(&raw const e_openerrf as *const c_char), efile);
                    // Dropping tears the conversion down again.
                    return None;
                }
                reader.source = Source::File(fd);
            } else if !tv.is_null() {
                reader.source = if (*tv).v_type == VAR_STRING as VarType {
                    Source::Text((*tv).vval.v_string)
                } else if (*tv).v_type == VAR_LIST as VarType {
                    Source::List(tv_list_first((*tv).vval.v_list))
                } else {
                    Source::Unusable
                };
            }
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
        unsafe {
            if self.len > 0 && self.line[self.len - 1] == b'\n' as c_char {
                self.line[self.len - 1] = 0;
            }
            remove_bom(self.line());
        }
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
        unsafe {
            if *at == 0 {
                return Status::EndOfInput;
            }
            let newline = vim_strchr(at, c_int::from(b'\n'));
            let want = if newline.is_null() {
                strlen(at)
            } else {
                newline.offset_from(at) as usize + 1
            };
            self.len = self.fit(want);
            ptr::copy_nonoverlapping(at, self.line(), self.len);
            self.line[self.len] = 0;
            // Advance by the whole line, so that the part of an over-long
            // line that did not fit is discarded rather than re-read.
            self.source = Source::Text(at.add(want));
        }
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
        unsafe {
            while !at.is_null()
                && ((*at).li_tv.v_type != VAR_STRING as VarType
                    || (*at).li_tv.vval.v_string.is_null())
            {
                at = (*at).li_next;
            }
            if at.is_null() {
                self.source = Source::List(ptr::null_mut());
                return Status::EndOfInput;
            }
            let text = (*at).li_tv.vval.v_string;
            self.len = self.fit(strlen(text));
            xstrlcpy(self.line(), text, self.len + 1);
            self.source = Source::List((*at).li_next);
        }
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
        unsafe {
            let text = ml_get_buf(buf, lnum);
            self.len = self.fit(ml_get_buf_len(buf, lnum) as usize);
            xstrlcpy(self.line(), text, self.len + 1);
        }
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
        unsafe {
            loop {
                *__errno_location() = 0;
                if !fgets(self.line(), READ_CHUNK as c_int, fd).is_null() {
                    break;
                }
                if *__errno_location() != EINTR {
                    return Status::EndOfInput;
                }
            }
            self.len = strlen(self.line.as_ptr());
            if self.len != READ_CHUNK - 1 || self.line[self.len - 1] == b'\n' as c_char {
                return self.convert();
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
                *__errno_location() = 0;
                let room = (self.room - filled) as c_int;
                if fgets(self.line().add(filled), room, fd).is_null() {
                    if *__errno_location() != EINTR {
                        break;
                    }
                    continue;
                }
                self.len = strlen(self.line.as_ptr().add(filled));
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
                    *__errno_location() = 0;
                    if fgets(scrap.as_mut_ptr(), READ_CHUNK as c_int, fd).is_null() {
                        if *__errno_location() != EINTR {
                            break;
                        }
                    } else if strlen(scrap.as_ptr()) < READ_CHUNK - 1
                        || scrap[READ_CHUNK - 2] == b'\n' as c_char
                    {
                        break;
                    }
                }
            }
            self.len = filled;
            self.convert()
        }
    }

    /// Convert the line just read out of the error file's encoding, if one
    /// was given and the line is not plain ASCII.
    ///
    /// # Safety
    ///
    /// `len` bytes of the buffer must be initialised and NUL-terminated.
    unsafe fn convert(&mut self) -> Status {
        if self.vc.vc_type == CONV_NONE as c_int {
            return Status::Ok;
        }
        // SAFETY: the line is NUL-terminated; `string_convert` answers a
        // freshly allocated string and writes back its length.
        unsafe {
            if !has_non_ascii(self.line.as_ptr()) {
                return Status::Ok;
            }
            let converted = string_convert(&raw const self.vc, self.line(), &raw mut self.len);
            if converted.is_null() {
                return Status::Ok;
            }
            if self.line.len() < self.len + 1 {
                self.line.resize(self.len + 1, 0);
            }
            xstrlcpy(self.line(), converted, self.len + 1);
            xfree(converted.cast());
            // Upstream adopts the converted allocation outright when the
            // line no longer fits one chunk, and records how much of it the
            // long-line reader may fill. Only that cap matters here, since
            // the buffer is owned either way.
            if self.len >= READ_CHUNK {
                self.room = self.room.max(self.len.min(LINE_MAXLEN));
            }
        }
        Status::Ok
    }
}

/// Read the error file `efile` into memory line by line, building the error
/// list, and set its title to `qf_title`.
///
/// # Safety
///
/// `wp`, when given, must be a live window; the strings must be
/// NUL-terminated.
pub unsafe fn qf_init(
    wp: *mut win_T,
    efile: *const c_char,
    errorformat: *mut c_char,
    newlist: c_int,
    qf_title: *const c_char,
    enc: *mut c_char,
) -> c_int {
    // SAFETY: forwarded from the caller.
    unsafe {
        let qi = if wp.is_null() {
            ql_info.get()
        } else {
            ll_get_or_alloc_list(wp)
        };
        debug_assert!(!qi.is_null());
        qf_init_ext(
            qi,
            (*qi).qf_curlist,
            efile,
            curbuf.get(),
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
pub(crate) unsafe fn qf_init_ext(
    qi: *mut qf_info_T,
    mut qf_idx: c_int,
    efile: *const c_char,
    buf: *mut buf_T,
    tv: *mut typval_T,
    errorformat: *mut c_char,
    newlist: bool,
    lnumfirst: linenr_T,
    lnumlast: linenr_T,
    qf_title: *const c_char,
    enc: *mut c_char,
) -> c_int {
    // SAFETY: forwarded from the caller.
    unsafe {
        // Do not use the cached buffer, it may have been wiped out.
        forget_last_buffer();

        let mut old_last: *mut qfline_T = ptr::null_mut();
        let mut retval = -1;
        let reader = Reader::open(enc, efile, tv, buf, lnumfirst, lnumlast);

        if let Some(mut reader) = reader {
            let mut adding = false;
            let qfl = if newlist || qf_idx == (*qi).qf_listcount {
                // Make place for a new list.
                qf_new_list(qi, qf_title);
                qf_idx = (*qi).qf_curlist;
                qf_get_list(qi, qf_idx)
            } else {
                // Adding to an existing list; remember its last entry.
                adding = true;
                let qfl = qf_get_list(qi, qf_idx);
                if !qf_list_empty(qfl) {
                    old_last = (*qfl).qf_last;
                }
                qfl
            };

            // Use the buffer-local 'errorformat' when it has one.
            let efm = if errorformat == p_efm.get()
                && tv.is_null()
                && !buf.is_null()
                && *(*buf).b_p_efm != 0
            {
                (*buf).b_p_efm
            } else {
                errorformat
            };

            // Take the compiled option out of the cache for the length of
            // this read and put it back at the end. Adding an entry can
            // fire `BufNew`, an autocommand can run another `:cexpr`, and
            // upstream — which keeps the compiled option in a bare static
            // and frees it whenever the option text changes — would then
            // free what this loop is still walking. Owning it here costs
            // the re-entrant call a recompile and nothing otherwise.
            let mut compiled = (*EFM_CACHE.ptr()).take();
            let text = CStr::from_ptr(efm).to_bytes();
            if !compiled.as_ref().is_some_and(|(had, _)| had == text) {
                compiled = Efm::compile(efm).map(|parsed| (text.to_vec(), parsed));
            }
            let built = match compiled.as_mut() {
                Some((_, parsed)) => read_lines(qfl, &mut reader, parsed),
                None => false,
            };
            *EFM_CACHE.ptr() = compiled;

            if built {
                retval = (*qfl).qf_count;
            } else if !adding {
                // The new list came to nothing; free it again.
                qf_free(qfl);
                (*qi).qf_listcount -= 1;
                if (*qi).qf_curlist > 0 {
                    (*qi).qf_curlist -= 1;
                }
            }
        }

        if qf_idx == (*qi).qf_curlist {
            qf_update_buffer(qi, old_last);
        }
        retval
    }
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
    unsafe {
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
                qf_add_entry(qfl, &fields.entry(qfl));
            }
            line_breakcheck();
        }

        if reader.had_error() {
            emsg(gettext(&raw const e_readerrf as *const c_char));
            return false;
        }
        if (*qfl).qf_index == 0 {
            // No valid entry was found.
            (*qfl).qf_ptr = (*qfl).qf_start;
            (*qfl).qf_index = 1;
            (*qfl).qf_nonevalid = true;
        } else {
            (*qfl).qf_nonevalid = false;
            if (*qfl).qf_ptr.is_null() {
                (*qfl).qf_ptr = (*qfl).qf_start;
            }
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
    unsafe {
        xfree((*qfl).qf_title.cast());
        (*qfl).qf_title = ptr::null_mut();
        if title.is_null() {
            return;
        }
        let len = strlen(title) + 1;
        let p: *mut c_char = xmallocz(len).cast();
        (*qfl).qf_title = p;
        xstrlcpy(p, title, len + 1);
    }
}

/// A list's default title is the command that created it, with a `:` in
/// front. Answers a pointer to a shared buffer, so the caller must be done
/// with it before the next call.
///
/// # Safety
///
/// `cmd` must be NUL-terminated.
pub(crate) unsafe fn qf_cmdtitle(cmd: *const c_char) -> *mut c_char {
    static TITLE: GlobalCell<[c_char; READ_CHUNK + 1]> = GlobalCell::new([0; READ_CHUNK + 1]);
    let title = TITLE.ptr().cast::<c_char>();
    // SAFETY: the caller's command is NUL-terminated, and the buffer holds
    // the IOSIZE bytes `snprintf` is told about.
    unsafe {
        snprintf(title, READ_CHUNK, c":%s".as_ptr(), cmd);
    }
    title
}
