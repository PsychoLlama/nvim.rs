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

extern "C" {
    // File I/O
    fn nvim_qf_open_file_for_read(efile: *const c_char) -> *mut libc::FILE;
    fn nvim_qf_fclose(fd: *mut libc::FILE);
    fn nvim_qf_fgets(buf: *mut c_char, size: c_int, fd: *mut libc::FILE) -> bool;
    fn nvim_qf_errno() -> c_int;

    // IObuff / IOSIZE
    fn nvim_qf_get_iobuff_ptr() -> *mut c_char;
    fn nvim_qf_get_iosize() -> c_int;

    // Memory helpers
    fn nvim_qf_xmalloc_buf(sz: usize) -> *mut c_char;
    fn nvim_qf_xrealloc_buf(ptr: *mut c_char, sz: usize) -> *mut c_char;
    fn nvim_qf_xfree_buf(ptr: *mut c_void);
    fn nvim_qf_xstrlcpy(dst: *mut c_char, src: *const c_char, n: usize);

    // Encoding conversion (opaque vimconv_T)
    fn nvim_qf_alloc_vimconv() -> *mut c_void;
    fn nvim_qf_free_vimconv(vc: *mut c_void);
    fn nvim_qf_convert_setup(vc: *mut c_void, enc: *const c_char);
    fn nvim_qf_convert_setup_cleanup(vc: *mut c_void);
    fn nvim_qf_vc_type(vc: *const c_void) -> c_int;
    fn nvim_qf_string_convert_with_len(
        vc: *mut c_void,
        buf: *mut c_char,
        lenp: *mut usize,
    ) -> *mut c_char;

    // String helpers
    fn nvim_qf_has_non_ascii(buf: *const c_char) -> bool;
    fn nvim_qf_remove_bom(buf: *mut c_char);
    fn nvim_qf_strchr_nl(str: *mut c_char) -> *mut c_char;

    // Buffer line access
    fn nvim_qf_ml_get_buf(buf: *mut c_void, lnum: i32) -> *mut c_char;
    fn nvim_qf_ml_get_buf_len(buf: *mut c_void, lnum: i32) -> i32;

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
        state.vc = nvim_qf_alloc_vimconv();
        if !enc.is_null() {
            nvim_qf_convert_setup(state.vc, enc.cast_const());
        }

        // Setup file source
        if !efile.is_null() {
            let fd = nvim_qf_open_file_for_read(efile);
            if fd.is_null() {
                // Error message already emitted by nvim_qf_open_file_for_read
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
            self.growbuf = nvim_qf_xmalloc_buf(self.linelen + 1);
            self.growbufsiz = self.linelen;
        } else if self.linelen > self.growbufsiz {
            self.growbuf = nvim_qf_xrealloc_buf(self.growbuf, self.linelen + 1);
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

        let iosize = nvim_qf_get_iosize() as usize;
        let iobuff = nvim_qf_get_iobuff_ptr();

        // Find newline or end of string
        let nl_ptr = nvim_qf_strchr_nl(p_str);
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
        let iosize = nvim_qf_get_iosize() as usize;
        let iobuff = nvim_qf_get_iobuff_ptr();

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
        nvim_qf_xstrlcpy(self.linebuf, s, self.linelen + 1);

        self.p_li = nvim_qf_list_item_next(self.p_list.cast_const(), p_li.cast_const());
        QF_OK
    }

    /// Read next line from buffer source.
    ///
    /// # Safety
    /// `self.buf` must be a valid buffer handle.
    unsafe fn get_next_buf_line(&mut self) -> c_int {
        let iosize = nvim_qf_get_iosize() as usize;
        let iobuff = nvim_qf_get_iobuff_ptr();

        if self.buflnum > self.lnumlast {
            return QF_END_OF_INPUT;
        }

        let p_buf = nvim_qf_ml_get_buf(self.buf, self.buflnum);
        let len = nvim_qf_ml_get_buf_len(self.buf, self.buflnum) as usize;
        self.buflnum += 1;

        if len > iosize - 2 {
            self.linebuf = self.grow_linebuf(len);
        } else {
            self.linebuf = iobuff;
            self.linelen = len;
        }
        nvim_qf_xstrlcpy(self.linebuf, p_buf, self.linelen + 1);
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
        let iosize = nvim_qf_get_iosize() as usize;
        let iobuff = nvim_qf_get_iobuff_ptr();

        // Retry loop for EINTR
        loop {
            if !nvim_qf_fgets(iobuff, nvim_qf_get_iosize(), self.fd) {
                if nvim_qf_errno() == libc::EINTR {
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
                self.growbuf = nvim_qf_xmalloc_buf(self.growbufsiz);
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
                    if !nvim_qf_fgets(self.growbuf.add(growbuflen), remaining as c_int, self.fd) {
                        if nvim_qf_errno() == libc::EINTR {
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
                self.growbuf = nvim_qf_xrealloc_buf(self.growbuf, self.growbufsiz);
            }

            // Discard loop: read and discard until we find EOL or EOF
            if discard {
                loop {
                    loop {
                        if !nvim_qf_fgets(iobuff, nvim_qf_get_iosize(), self.fd) {
                            if nvim_qf_errno() == libc::EINTR {
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
        if nvim_qf_vc_type(self.vc.cast_const()) != 0 && nvim_qf_has_non_ascii(self.linebuf) {
            let mut converted_len = self.linelen;
            let line =
                nvim_qf_string_convert_with_len(self.vc, self.linebuf, &raw mut converted_len);
            if !line.is_null() {
                if converted_len < iosize {
                    nvim_qf_xstrlcpy(self.linebuf, line, converted_len + 1);
                    nvim_qf_xfree_buf(line.cast());
                } else {
                    nvim_qf_xfree_buf(self.growbuf.cast());
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

        nvim_qf_remove_bom(self.linebuf);
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
                nvim_qf_fclose(self.fd);
                self.fd = std::ptr::null_mut();
            }
            if !self.growbuf.is_null() {
                nvim_qf_xfree_buf(self.growbuf.cast());
                self.growbuf = std::ptr::null_mut();
            }
            if !self.vc.is_null() {
                if nvim_qf_vc_type(self.vc.cast_const()) != 0 {
                    nvim_qf_convert_setup_cleanup(self.vc);
                }
                nvim_qf_free_vimconv(self.vc);
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
