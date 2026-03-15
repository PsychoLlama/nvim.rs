//! Quickfix parser IO state machine.
//!
//! This module provides `QfParserState`, the Rust equivalent of `qfstate_T` in C,
//! along with line-reading functions for file, string, list, and buffer sources.

#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]

use std::ffi::{c_char, c_int, c_void};

// ===========================================================================
// QF status codes (must match C enum in quickfix_shim.c)
// ===========================================================================

const QF_OK: c_int = 1;
const QF_END_OF_INPUT: c_int = 2;
const QF_FAIL: c_int = 0;

// LINE_MAXLEN matches the C constant in quickfix_shim.c
const LINE_MAXLEN: usize = 4096;

// ===========================================================================
// External C accessor functions
// ===========================================================================

const IOSIZE: usize = 1025;

/// Thin wrapper: calls `fgets` and returns true if data was read (non-null return).
///
/// # Safety
/// `buf` must point to at least `size` bytes; `fd` must be a valid open FILE*.
#[inline]
unsafe fn fgets_not_null(buf: *mut c_char, size: c_int, fd: *mut libc::FILE) -> bool {
    !libc::fgets(buf, size, fd).is_null()
}

extern "C" {
    // File I/O
    fn os_fopen(fname: *const c_char, mode: *const c_char) -> *mut libc::FILE;
    fn os_open_stdin_fd() -> c_int;
    fn fdopen(fd: c_int, mode: *const c_char) -> *mut libc::FILE;
    fn semsg(fmt: *const std::ffi::c_char, ...) -> bool;
    fn emsg(msg: *const std::ffi::c_char) -> bool;

    // IObuff global
    static IObuff: *mut c_char;

    // Memory helpers
    fn xmalloc(sz: usize) -> *mut c_void;
    fn xrealloc(ptr: *mut c_void, sz: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    fn xcalloc(count: usize, size: usize) -> *mut c_void;
    fn xstrlcpy(dst: *mut c_char, src: *const c_char, n: usize) -> usize;
    fn xstrdup(s: *const c_char) -> *mut c_char;

    // Encoding conversion (opaque vimconv_T)
    fn nvim_qf_sizeof_vimconv() -> usize;
    fn nvim_qf_convert_setup(vc: *mut c_void, enc: *const c_char);
    fn nvim_qf_convert_setup_cleanup(vc: *mut c_void);
    fn nvim_qf_vc_type(vc: *const c_void) -> c_int;
    fn string_convert(vc: *mut c_void, buf: *mut c_char, lenp: *mut usize) -> *mut c_char;

    // String helpers
    fn has_non_ascii(s: *const c_char) -> bool;
    fn remove_bom(buf: *mut c_char);
    fn vim_strchr(s: *mut c_char, c: c_int) -> *mut c_char;

    // Buffer line access
    fn ml_get_buf(buf: *mut c_void, lnum: i32) -> *mut c_char;
    fn ml_get_buf_len(buf: *mut c_void, lnum: i32) -> i32;

    // VimL typval (tv) source
    fn nvim_qf_tv_is_string(tv: *const c_void) -> bool;
    fn nvim_qf_tv_get_string(tv: *mut c_void) -> *mut c_char;
    fn nvim_qf_tv_list_first(tv: *mut c_void) -> *mut c_void;
    fn nvim_qf_tv_get_list(tv: *const c_void) -> *mut c_void;
    fn nvim_qf_list_item_next(list: *const c_void, li: *const c_void) -> *mut c_void;
    fn nvim_qf_list_item_is_string(li: *const c_void) -> bool;
    fn nvim_qf_list_item_string(li: *mut c_void) -> *mut c_char;
}

// ===========================================================================
// QfParserState — equivalent of C's qfstate_T
// ===========================================================================

/// Parser state for quickfix initialization.
///
/// Mirrors `qfstate_T` in C but owned by Rust. The `vimconv_T` encoding
/// converter is kept as an opaque heap-allocated pointer managed via C helpers.
pub struct QfParserState {
    /// Current line buffer (points to either `IObuff` or growbuf)
    pub linebuf: *mut c_char,
    /// Length of current line in linebuf (not including NUL)
    pub linelen: usize,
    /// Heap-allocated grow buffer (NULL if not yet needed)
    growbuf: *mut c_char,
    /// Size of growbuf allocation
    growbufsiz: usize,

    /// File descriptor for file-source reading (NULL if not file)
    fd: *mut libc::FILE,
    /// Had a read error on fd (used for ferror check)
    fd_error: bool,

    /// Typval pointer (for string or list source)
    tv: *mut c_void,
    /// Current position in string source
    p_str: *mut c_char,
    /// `VimL` list pointer (for list source)
    p_list: *mut c_void,
    /// Current list item (for list source)
    p_li: *mut c_void,

    /// Buffer handle (for buffer source)
    buf: *mut c_void,
    /// Current buffer line (1-based)
    buflnum: i32,
    /// Last buffer line to read
    lnumlast: i32,

    /// Opaque heap-allocated `vimconv_T` (NULL if no conversion)
    vc: *mut c_void,
}

impl QfParserState {
    /// Create an empty parser state (all null/zero).
    fn new_empty() -> Self {
        Self {
            linebuf: std::ptr::null_mut(),
            linelen: 0,
            growbuf: std::ptr::null_mut(),
            growbufsiz: 0,
            fd: std::ptr::null_mut(),
            fd_error: false,
            tv: std::ptr::null_mut(),
            p_str: std::ptr::null_mut(),
            p_list: std::ptr::null_mut(),
            p_li: std::ptr::null_mut(),
            buf: std::ptr::null_mut(),
            buflnum: 0,
            lnumlast: 0,
            vc: std::ptr::null_mut(),
        }
    }

    /// Setup parser state from parameters.
    ///
    /// Returns `Ok(Self)` on success, `Err(())` on failure (file not found).
    ///
    /// # Errors
    /// Returns `Err(())` if `efile` is provided but cannot be opened for reading.
    ///
    /// # Safety
    /// - `enc` must be a valid C string or NULL
    /// - `efile` must be a valid C string or NULL
    /// - `tv` and `buf` must be valid pointers or NULL
    #[allow(clippy::result_unit_err)]
    pub unsafe fn setup(
        enc: *mut c_char,
        efile: *const c_char,
        tv: *mut c_void,
        buf: *mut c_void,
        lnumfirst: i32,
        lnumlast: i32,
    ) -> Result<Box<Self>, ()> {
        let mut state = Box::new(Self::new_empty());

        // Setup encoding converter if enc is provided
        state.vc = xcalloc(1, nvim_qf_sizeof_vimconv()).cast();
        if !enc.is_null() {
            nvim_qf_convert_setup(state.vc, enc.cast_const());
        }

        // Setup file source
        if !efile.is_null() {
            // Inline nvim_qf_open_file_for_read: open efile for reading, emit error on failure
            let is_stdin = {
                let s = std::ffi::CStr::from_ptr(efile);
                s.to_bytes() == b"-"
            };
            let fd = if is_stdin {
                fdopen(os_open_stdin_fd(), c"r".as_ptr())
            } else {
                os_fopen(efile, c"r".as_ptr())
            };
            if fd.is_null() {
                semsg(c"E40: Can't open errorfile %s".as_ptr(), efile);
                return Err(());
            }
            state.fd = fd;
        }

        // Setup tv source (string or list)
        if !tv.is_null() {
            if nvim_qf_tv_is_string(tv.cast_const()) {
                state.p_str = nvim_qf_tv_get_string(tv);
            } else {
                // VAR_LIST: get first list item
                state.p_list = nvim_qf_tv_get_list(tv);
                state.p_li = nvim_qf_tv_list_first(tv);
            }
            state.tv = tv;
        }

        // Setup buffer source
        state.buf = buf;
        state.buflnum = lnumfirst;
        state.lnumlast = lnumlast;

        Ok(state)
    }

    /// Grow the line buffer to hold at least `newsz` bytes.
    ///
    /// Returns pointer to growbuf on success.
    ///
    /// # Safety
    /// Calls C memory allocators.
    unsafe fn grow_linebuf(&mut self, newsz: usize) -> *mut c_char {
        // If the line exceeds LINE_MAXLEN, cap it (excluding the last byte since
        // it's not a NL character in that case)
        self.linelen = if newsz > LINE_MAXLEN {
            LINE_MAXLEN - 1
        } else {
            newsz
        };
        if self.growbuf.is_null() {
            self.growbuf = xmalloc(self.linelen + 1).cast();
            self.growbufsiz = self.linelen;
        } else if self.linelen > self.growbufsiz {
            self.growbuf = xrealloc(self.growbuf.cast(), self.linelen + 1).cast();
            self.growbufsiz = self.linelen;
        }
        self.growbuf
    }

    /// Read next line from string source (state->p_str).
    ///
    /// # Safety
    /// `p_str` must be a valid C string pointer.
    unsafe fn get_next_str_line(&mut self) -> c_int {
        let p_str = self.p_str;
        if p_str.is_null() || *p_str == 0 {
            return QF_END_OF_INPUT;
        }

        let iosize = IOSIZE;
        let iobuff = IObuff;

        // Find newline or end of string
        let nl_ptr = vim_strchr(p_str, b'\n' as c_int);
        let len = if nl_ptr.is_null() {
            libc::strlen(p_str.cast_const())
        } else {
            nl_ptr.offset_from(p_str) as usize + 1
        };

        if len > iosize - 2 {
            self.linebuf = self.grow_linebuf(len);
        } else {
            self.linebuf = iobuff;
            self.linelen = len;
        }
        // Copy (including potential NL which will be stripped later)
        libc::memcpy(self.linebuf.cast(), p_str.cast(), self.linelen);
        *self.linebuf.add(self.linelen) = 0;

        // Advance p_str by len to discard rest of line if it exceeded limit
        self.p_str = p_str.add(len);
        QF_OK
    }

    /// Read next line from list source (state->p_li).
    ///
    /// # Safety
    /// List pointers must be valid.
    unsafe fn get_next_list_line(&mut self) -> c_int {
        let iosize = IOSIZE;
        let iobuff = IObuff;

        // Skip non-string items
        let mut p_li = self.p_li;
        while !p_li.is_null() && !nvim_qf_list_item_is_string(p_li.cast_const()) {
            p_li = nvim_qf_list_item_next(self.p_list.cast_const(), p_li.cast_const());
        }

        if p_li.is_null() {
            self.p_li = std::ptr::null_mut();
            return QF_END_OF_INPUT;
        }

        let s = nvim_qf_list_item_string(p_li);
        // s is guaranteed non-null because nvim_qf_list_item_is_string returned true
        let len = libc::strlen(s.cast_const());

        if len > iosize - 2 {
            self.linebuf = self.grow_linebuf(len);
        } else {
            self.linebuf = iobuff;
            self.linelen = len;
        }
        xstrlcpy(self.linebuf, s, self.linelen + 1);

        self.p_li = nvim_qf_list_item_next(self.p_list.cast_const(), p_li.cast_const());
        QF_OK
    }

    /// Read next line from buffer source.
    ///
    /// # Safety
    /// `self.buf` must be a valid buffer handle.
    unsafe fn get_next_buf_line(&mut self) -> c_int {
        let iosize = IOSIZE;
        let iobuff = IObuff;

        if self.buflnum > self.lnumlast {
            return QF_END_OF_INPUT;
        }

        let p_buf = ml_get_buf(self.buf, self.buflnum);
        let len = ml_get_buf_len(self.buf, self.buflnum) as usize;
        self.buflnum += 1;

        if len > iosize - 2 {
            self.linebuf = self.grow_linebuf(len);
        } else {
            self.linebuf = iobuff;
            self.linelen = len;
        }
        xstrlcpy(self.linebuf, p_buf, self.linelen + 1);
        QF_OK
    }

    /// Read next line from file source (state->fd).
    ///
    /// Handles the grow-buffer logic for lines exceeding IOSIZE, and the
    /// discard loop for lines exceeding `LINE_MAXLEN`.
    ///
    /// # Safety
    /// `self.fd` must be a valid FILE*.
    unsafe fn get_next_file_line(&mut self) -> c_int {
        let iosize = IOSIZE;
        let iobuff = IObuff;

        // Retry loop for EINTR
        loop {
            if !fgets_not_null(iobuff, IOSIZE as c_int, self.fd) {
                if *libc::__errno_location() == libc::EINTR {
                    continue;
                }
                return QF_END_OF_INPUT;
            }
            break;
        }

        let mut discard = false;
        self.linelen = libc::strlen(iobuff.cast_const());

        if self.linelen == iosize - 1 && *iobuff.add(self.linelen - 1) != b'\n' as c_char {
            // Line exceeds IObuff; use growbuf to accumulate the rest.
            if self.growbuf.is_null() {
                self.growbufsiz = 2 * (iosize - 1);
                self.growbuf = xmalloc(self.growbufsiz).cast();
            }
            // Copy read part (excluding NUL terminator)
            libc::memcpy(self.growbuf.cast(), iobuff.cast(), iosize - 1);
            let mut growbuflen: usize = self.linelen;

            loop {
                // Ensure there is room for at least one more byte
                let remaining = self.growbufsiz - growbuflen;
                if remaining == 0 {
                    // Can't grow further; discard
                    discard = true;
                    break;
                }
                // EINTR retry loop for inner fgets
                loop {
                    if !fgets_not_null(self.growbuf.add(growbuflen), remaining as c_int, self.fd) {
                        if *libc::__errno_location() == libc::EINTR {
                            continue;
                        }
                        // EOF or real error - stop growing
                        break;
                    }
                    break;
                }
                self.linelen = libc::strlen(self.growbuf.add(growbuflen).cast_const());
                growbuflen += self.linelen;
                if *self.growbuf.add(growbuflen - 1) == b'\n' as c_char {
                    break;
                }
                if self.growbufsiz == LINE_MAXLEN {
                    discard = true;
                    break;
                }
                // Grow the buffer
                self.growbufsiz = self.growbufsiz.wrapping_mul(2).min(LINE_MAXLEN);
                self.growbuf = xrealloc(self.growbuf.cast(), self.growbufsiz).cast();
            }

            // Discard loop: read and discard until we find EOL or EOF
            if discard {
                loop {
                    loop {
                        if !fgets_not_null(iobuff, IOSIZE as c_int, self.fd) {
                            if *libc::__errno_location() == libc::EINTR {
                                continue;
                            }
                            break;
                        }
                        break;
                    }
                    let chunk_len = libc::strlen(iobuff.cast_const());
                    if chunk_len < iosize - 1 || *iobuff.add(iosize - 2) == b'\n' as c_char {
                        break;
                    }
                }
            }

            self.linebuf = self.growbuf;
            self.linelen = growbuflen;
        } else {
            self.linebuf = iobuff;
        }

        // Encoding conversion for non-ASCII lines
        if nvim_qf_vc_type(self.vc.cast_const()) != 0 && has_non_ascii(self.linebuf) {
            let mut converted_len = self.linelen;
            let line = string_convert(self.vc, self.linebuf, &raw mut converted_len);
            if !line.is_null() {
                if converted_len < iosize {
                    xstrlcpy(self.linebuf, line, converted_len + 1);
                    xfree(line.cast());
                } else {
                    xfree(self.growbuf.cast());
                    self.linebuf = line;
                    self.growbuf = line;
                    self.growbufsiz = converted_len.min(LINE_MAXLEN);
                }
                self.linelen = converted_len;
            }
        }

        QF_OK
    }

    /// Get the next line from whatever source is configured.
    ///
    /// Sets `self.linebuf` and `self.linelen`. Returns `QF_OK`, `QF_END_OF_INPUT`,
    /// or `QF_FAIL`.
    ///
    /// # Safety
    /// All source pointers must be valid.
    pub unsafe fn get_nextline(&mut self) -> c_int {
        let status = if !self.fd.is_null() {
            self.get_next_file_line()
        } else if !self.tv.is_null() {
            if nvim_qf_tv_is_string(self.tv.cast_const()) {
                self.get_next_str_line()
            } else {
                self.get_next_list_line()
            }
        } else {
            self.get_next_buf_line()
        };

        if status != QF_OK {
            return status;
        }

        // Strip trailing newline
        if self.linelen > 0 && *self.linebuf.add(self.linelen - 1) == b'\n' as c_char {
            *self.linebuf.add(self.linelen - 1) = 0;
            self.linelen -= 1;
            // Also strip CR on Windows-style files
            #[cfg(target_os = "windows")]
            if self.linelen > 0 && *self.linebuf.add(self.linelen - 1) == b'\r' as c_char {
                *self.linebuf.add(self.linelen - 1) = 0;
                self.linelen -= 1;
            }
        }

        remove_bom(self.linebuf);
        QF_OK
    }

    /// Check if the file source had no error (or there is no file source).
    ///
    /// Used after the main parse loop to decide whether to emit `E_READERRF`.
    pub fn no_fd_error(&self) -> bool {
        self.fd.is_null() || !self.fd_error
    }
}

impl Drop for QfParserState {
    fn drop(&mut self) {
        unsafe {
            if !self.fd.is_null() {
                libc::fclose(self.fd);
                self.fd = std::ptr::null_mut();
            }
            if !self.growbuf.is_null() {
                xfree(self.growbuf.cast());
                self.growbuf = std::ptr::null_mut();
            }
            if !self.vc.is_null() {
                if nvim_qf_vc_type(self.vc.cast_const()) != 0 {
                    nvim_qf_convert_setup_cleanup(self.vc);
                }
                xfree(self.vc);
                self.vc = std::ptr::null_mut();
            }
        }
    }
}

// ===========================================================================
// C-callable entry points for the new Rust-owned state
// ===========================================================================

/// Create and setup a `QfParserState`; returns opaque heap pointer or NULL.
///
/// Replaces `nvim_qf_init_setup_state`.
///
/// # Safety
/// All pointer parameters must be valid or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_parser_state_new(
    enc: *mut c_char,
    efile: *const c_char,
    tv: *mut c_void,
    buf: *mut c_void,
    lnumfirst: i32,
    lnumlast: i32,
) -> *mut c_void {
    match QfParserState::setup(enc, efile, tv, buf, lnumfirst, lnumlast) {
        Ok(boxed) => Box::into_raw(boxed).cast(),
        Err(()) => std::ptr::null_mut(),
    }
}

/// Free a `QfParserState` created by `rs_qf_parser_state_new`.
///
/// Replaces `nvim_qf_init_cleanup_state`.
///
/// # Safety
/// `state` must have been created by `rs_qf_parser_state_new` or be NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_parser_state_free(state: *mut c_void) {
    if !state.is_null() {
        drop(Box::from_raw(state.cast::<QfParserState>()));
    }
}

/// Get the next line into the state's linebuf.
///
/// Returns `QF_OK` (1), `QF_END_OF_INPUT` (2), or `QF_FAIL` (0).
///
/// # Safety
/// `state` must be a valid pointer from `rs_qf_parser_state_new`.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_parser_state_get_nextline(state: *mut c_void) -> c_int {
    if state.is_null() {
        return QF_FAIL;
    }
    (*state.cast::<QfParserState>()).get_nextline()
}

/// Get the linebuf pointer from state.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_parser_state_get_linebuf(state: *const c_void) -> *mut c_char {
    if state.is_null() {
        return std::ptr::null_mut();
    }
    (*state.cast::<QfParserState>()).linebuf
}

/// Get the linelen from state.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_parser_state_get_linelen(state: *const c_void) -> usize {
    if state.is_null() {
        return 0;
    }
    (*state.cast::<QfParserState>()).linelen
}

/// Check if state's file source had no error.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_parser_state_no_fd_error(state: *const c_void) -> bool {
    if state.is_null() {
        return true;
    }
    (*state.cast::<QfParserState>()).no_fd_error()
}

// ===========================================================================
// Phase 2: EfmPattern (Rust equivalent of efm_T)
// ===========================================================================

/// Number of % pattern slots in errorformat (matches C `FMT_PATTERNS`)
pub const FMT_PATTERNS: usize = 14;

/// `CMDBUFFSIZE` from C (`os_defs.h`)
pub const CMDBUFFSIZE: usize = 1024;

extern "C" {
    // Errorformat conversion helpers (from parse.rs)
    fn rs_efm_to_regpat(
        efm: *const c_char,
        efm_len: usize,
        addr: *mut c_char,
        out: *mut c_char,
        out_size: usize,
    ) -> EfmToRegpatResult;
    fn rs_efm_regpat_bufsz(efm: *const c_char, efm_len: usize) -> usize;
    fn rs_efm_option_part_len(efm: *const c_char, efm_max_len: usize) -> c_int;
    fn rs_skip_to_option_part(p: *const c_char) -> *const c_char;

    // vim regex
    fn vim_regcomp(pat: *const c_char, flags: c_int) -> *mut c_void;
    fn vim_regfree(prog: *mut c_void);

    // Error messages: semsg and emsg declared in the first extern block
    // xstrdup: declared in the first extern block
}

/// Result type matching C's `EfmToRegpatResult` (must stay in sync with parse.rs)
#[repr(C)]
struct EfmToRegpatResult {
    bytes_written: usize,
    prefix: c_char,
    flags: c_char,
    conthere: bool,
    status: c_int,
    error_code: c_int,
    error_char: c_char,
}

/// One parsed errorformat element.
///
/// Equivalent to C's `efm_T`. Must be ABI-compatible via raw pointer casting.
pub struct EfmPattern {
    /// Pre-compiled regex program (opaque C pointer, owned - free with `vim_regfree`)
    pub prog: *mut c_void,
    /// Next in linked list (NULL if last)
    pub next: *mut EfmPattern,
    /// Indices of used % patterns (`FMT_PATTERNS` elements)
    pub addr: [c_char; FMT_PATTERNS],
    /// Prefix character (E/W/I/N/C/Z/G/P/Q/O/D/X/A)
    pub prefix: c_char,
    /// Flags character ('-' or '+')
    pub flags: c_char,
    /// Whether %> (conthere) was used
    pub conthere: c_int,
}

impl EfmPattern {
    /// Create a zeroed `EfmPattern`.
    fn new_zeroed() -> Self {
        Self {
            prog: std::ptr::null_mut(),
            next: std::ptr::null_mut(),
            addr: [0; FMT_PATTERNS],
            prefix: 0,
            flags: 0,
            conthere: 0,
        }
    }
}

/// Free a linked list of `EfmPattern` nodes.
///
/// Also resets the Rust `FMT_START` via `rs_qf_reset_fmt_start`.
///
/// # Safety
/// `efm_first` must be a valid pointer from `rs_qf_parse_efm_option` or NULL.
pub unsafe fn free_efm_pattern_list(efm_first: *mut EfmPattern) {
    let mut p = efm_first;
    while !p.is_null() {
        let node = Box::from_raw(p);
        p = node.next;
        if !node.prog.is_null() {
            vim_regfree(node.prog);
        }
        // node is dropped here (Box goes out of scope)
    }
    // Reset the fmt_start static in parse.rs
    rs_qf_reset_fmt_start();
}

extern "C" {
    fn rs_qf_reset_fmt_start();
}

/// Parse an errorformat string into a linked list of `EfmPattern` nodes.
///
/// Returns NULL on error (e.g., empty format or compilation failure).
///
/// # Safety
/// `efm` must be a valid null-terminated C string.
unsafe fn parse_efm_option(efm: *const c_char) -> *mut EfmPattern {
    if efm.is_null() || *efm == 0 {
        return std::ptr::null_mut();
    }

    let efm_cstr = std::ffi::CStr::from_ptr(efm);
    let efm_bytes = efm_cstr.to_bytes();
    let total_len = efm_bytes.len();

    let bufsz = rs_efm_regpat_bufsz(efm, total_len);
    if bufsz == 0 {
        return std::ptr::null_mut();
    }
    // Allocate the regex pattern buffer via xmalloc
    let fmtstr: *mut c_char = xmalloc(bufsz).cast();

    let mut fmt_first: *mut EfmPattern = std::ptr::null_mut();
    let mut fmt_last: *mut EfmPattern = std::ptr::null_mut();
    let mut pos: *const c_char = efm;

    loop {
        if *pos == 0 {
            break;
        }

        let remaining = total_len - pos.offset_from(efm) as usize;
        let len = rs_efm_option_part_len(pos, remaining) as usize;

        // Allocate a new EfmPattern
        let mut fmt_ptr = EfmPattern::new_zeroed();

        // Convert this format part to a regex pattern
        let result = rs_efm_to_regpat(pos, len, fmt_ptr.addr.as_mut_ptr(), fmtstr, bufsz);

        if result.status != 1 {
            // 1 = OK in Neovim
            // Emit the appropriate error message
            match result.error_code {
                372 => {
                    semsg(
                        c"E372: Too many %%%c in format string".as_ptr(),
                        result.error_char as c_int,
                    );
                }
                373 => {
                    semsg(
                        c"E373: Unexpected %%%c in format string".as_ptr(),
                        result.error_char as c_int,
                    );
                }
                374 => {
                    emsg(c"E374: Missing ] in format string".as_ptr());
                }
                375 => {
                    semsg(
                        c"E375: Unsupported %%%c in format string".as_ptr(),
                        result.error_char as c_int,
                    );
                }
                376 => {
                    semsg(
                        c"E376: Invalid %%%c in format string prefix".as_ptr(),
                        result.error_char as c_int,
                    );
                }
                377 => {
                    semsg(
                        c"E377: Invalid %%%c in format string".as_ptr(),
                        result.error_char as c_int,
                    );
                }
                _ => {
                    emsg(c"E378: 'errorformat' contains no pattern".as_ptr());
                }
            }
            // fmt_ptr has no prog to free; it will be dropped here (no Drop impl)
            free_efm_pattern_list(fmt_first);
            xfree(fmtstr.cast());
            return std::ptr::null_mut();
        }

        fmt_ptr.prefix = result.prefix;
        fmt_ptr.flags = result.flags;
        fmt_ptr.conthere = result.conthere as c_int;

        // Compile the regex pattern
        let prog = vim_regcomp(fmtstr, 1 + 2); // RE_MAGIC + RE_STRING
        if prog.is_null() {
            // fmt_ptr has no prog to free; it will be dropped here (no Drop impl)
            free_efm_pattern_list(fmt_first);
            xfree(fmtstr.cast());
            return std::ptr::null_mut();
        }
        fmt_ptr.prog = prog;

        // Append to list (box now, after all fields are set)
        let raw_ptr = Box::into_raw(Box::new(fmt_ptr));
        if fmt_first.is_null() {
            fmt_first = raw_ptr;
        } else {
            (*fmt_last).next = raw_ptr;
        }
        fmt_last = raw_ptr;

        // Advance to next part (skip comma + spaces)
        pos = rs_skip_to_option_part(pos.add(len));
    }

    if fmt_first.is_null() {
        emsg(c"E378: 'errorformat' contains no pattern".as_ptr());
    }

    xfree(fmtstr.cast());
    fmt_first
}

/// Static efm cache (equivalent to C's `s_fmt_first` / `s_last_efm`).
///
/// Stores the last-used efm string (heap-allocated) and the compiled pattern list.
struct EfmCache {
    last_efm: *mut c_char,
    fmt_first: *mut EfmPattern,
}

unsafe impl Send for EfmCache {}
unsafe impl Sync for EfmCache {}

static mut EFM_CACHE: EfmCache = EfmCache {
    last_efm: std::ptr::null_mut(),
    fmt_first: std::ptr::null_mut(),
};

// ===========================================================================
// Phase 2: C-callable exports
// ===========================================================================

/// Parse an errorformat string and return the first [`EfmPattern`] as opaque pointer.
///
/// # Safety
/// `efm` must be a valid null-terminated C string or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_parse_efm_option(efm: *const c_char) -> *mut c_void {
    if efm.is_null() {
        return std::ptr::null_mut();
    }
    parse_efm_option(efm).cast()
}

/// Free a linked list of `EfmPattern` nodes created by `rs_qf_parse_efm_option`.
///
/// # Safety
/// `efm_first` must have been created by `rs_qf_parse_efm_option` or be NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_free_efm_list(efm_first: *mut c_void) {
    free_efm_pattern_list(efm_first.cast());
}

/// Update the cached efm parse and return the `fmt_first` pointer.
///
/// Returns NULL if parsing fails.
///
/// # Safety
/// `efm` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_init_update_efm_cache(efm: *mut c_char) -> *mut c_void {
    let cache = &raw mut EFM_CACHE;
    let last = (*cache).last_efm;
    let needs_update = last.is_null() || libc::strcmp(last.cast_const(), efm.cast_const()) != 0;

    if needs_update {
        // Free old cache
        if !last.is_null() {
            xfree(last.cast());
            (*cache).last_efm = std::ptr::null_mut();
        }
        free_efm_pattern_list((*cache).fmt_first);
        (*cache).fmt_first = std::ptr::null_mut();

        // Parse new efm
        (*cache).fmt_first = parse_efm_option(efm);
        if !(*cache).fmt_first.is_null() {
            (*cache).last_efm = xstrdup(efm);
        }
    }

    (*cache).fmt_first.cast()
}

// ===========================================================================
// Phase 2: QfAllFields — full quickfix fields (Rust equivalent of qffields_T)
// ===========================================================================

/// Full quickfix fields with owned heap buffers.
///
/// Equivalent to C's `qffields_T`. The string fields are heap-allocated
/// via C's xmalloc (so they can be passed to C functions that expect them).
pub struct QfAllFields {
    /// Filename buffer (CMDBUFFSIZE + 1 bytes)
    pub namebuf: *mut c_char,
    /// Buffer number
    pub bnr: c_int,
    /// Module name buffer (CMDBUFFSIZE + 1 bytes)
    pub module: *mut c_char,
    /// Error message buffer (initially CMDBUFFSIZE + 1, may grow)
    pub errmsg: *mut c_char,
    /// Allocated size of errmsg buffer
    pub errmsglen: usize,
    /// Line number
    pub lnum: i32,
    /// End line number
    pub end_lnum: i32,
    /// Column number
    pub col: c_int,
    /// End column number
    pub end_col: c_int,
    /// Whether column is visual
    pub use_viscol: bool,
    /// Pattern buffer (CMDBUFFSIZE + 1 bytes)
    pub pattern: *mut c_char,
    /// Error number
    pub enr: c_int,
    /// Error type character
    pub type_char: c_char,
    /// User data (`typval_T`*, always NULL in Rust-managed path)
    pub user_data: *mut c_void,
    /// Whether entry is valid
    pub valid: bool,
}

impl QfAllFields {
    /// Allocate a new `QfAllFields` with heap-allocated string buffers.
    ///
    /// # Safety
    /// Calls C xmalloc.
    pub unsafe fn alloc() -> Box<Self> {
        let bufsz = CMDBUFFSIZE + 1;
        Box::new(Self {
            namebuf: xmalloc(bufsz).cast(),
            bnr: 0,
            module: xmalloc(bufsz).cast(),
            errmsg: xmalloc(bufsz).cast(),
            errmsglen: bufsz,
            lnum: 0,
            end_lnum: 0,
            col: 0,
            end_col: 0,
            use_viscol: false,
            pattern: xmalloc(bufsz).cast(),
            enr: 0,
            type_char: 0,
            user_data: std::ptr::null_mut(),
            valid: false,
        })
    }
}

impl QfAllFields {
    /// Set the error message, growing the buffer if needed.
    ///
    /// Mirrors C's `nvim_qf_fields_set_errmsg`.
    ///
    /// # Safety
    /// `msg` must be a valid pointer to `len` bytes (NUL-terminated at `msg[len]`).
    pub unsafe fn set_errmsg(&mut self, msg: *const c_char, len: usize) {
        if msg.is_null() {
            return;
        }
        if len >= self.errmsglen {
            self.errmsg = xrealloc(self.errmsg.cast(), len + 1).cast();
            self.errmsglen = len + 1;
        }
        xstrlcpy(self.errmsg, msg, len + 1);
    }
}

impl Drop for QfAllFields {
    fn drop(&mut self) {
        unsafe {
            if !self.namebuf.is_null() {
                xfree(self.namebuf.cast());
            }
            if !self.module.is_null() {
                xfree(self.module.cast());
            }
            if !self.errmsg.is_null() {
                xfree(self.errmsg.cast());
            }
            if !self.pattern.is_null() {
                xfree(self.pattern.cast());
            }
        }
    }
}

/// Allocate a new `QfAllFields` and return it as an opaque heap pointer.
///
/// # Safety
/// Returns a heap-allocated pointer. Free with `rs_qf_free_fields`.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_alloc_fields() -> *mut c_void {
    Box::into_raw(QfAllFields::alloc()).cast()
}

/// Free a `QfAllFields` allocated by `rs_qf_alloc_fields`.
///
/// # Safety
/// `fields` must have been created by `rs_qf_alloc_fields` or be NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_free_fields(fields: *mut c_void) {
    if !fields.is_null() {
        drop(Box::from_raw(fields.cast::<QfAllFields>()));
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qf_parser_state_constants() {
        assert_eq!(QF_OK, 1);
        assert_eq!(QF_END_OF_INPUT, 2);
        assert_eq!(QF_FAIL, 0);
        assert_eq!(LINE_MAXLEN, 4096);
    }
}
