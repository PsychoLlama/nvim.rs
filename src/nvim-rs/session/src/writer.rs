//! Session file writer using libc FILE* output
//!
//! Provides `SessionWriter` for writing to C FILE* handles via `libc::fwrite`,
//! matching the pattern used in `src/nvim-rs/log/src/core.rs`.

use std::ffi::{c_char, c_int};
use std::fmt;

/// OK return value (matches C OK = 1)
const OK: c_int = 1;
/// FAIL return value (matches C FAIL = 0)
const FAIL: c_int = 0;

/// Wrapper around a C `FILE*` for writing session file content.
pub struct SessionWriter {
    fd: *mut libc::FILE,
}

impl SessionWriter {
    /// Create a new `SessionWriter` from a C `FILE*`.
    ///
    /// # Safety
    /// The `fd` must be a valid, open `FILE*` for the lifetime of this writer.
    #[must_use]
    pub const unsafe fn new(fd: *mut libc::FILE) -> Self {
        Self { fd }
    }

    /// Write raw bytes to the file.
    ///
    /// Returns `true` on success, `false` on write error.
    pub fn write_bytes(&mut self, s: &[u8]) -> bool {
        if s.is_empty() {
            return true;
        }
        let written =
            unsafe { libc::fwrite(s.as_ptr().cast::<libc::c_void>(), 1, s.len(), self.fd) };
        written == s.len()
    }

    /// Write a newline to the file.
    ///
    /// Returns OK on success, FAIL on error.
    pub fn put_eol(&mut self) -> c_int {
        if self.write_bytes(b"\n") {
            OK
        } else {
            FAIL
        }
    }

    /// Write a string followed by a newline.
    ///
    /// Returns OK on success, FAIL on error.
    pub fn put_line(&mut self, s: &[u8]) -> c_int {
        if self.write_bytes(s) && self.write_bytes(b"\n") {
            OK
        } else {
            FAIL
        }
    }

    /// Write formatted data to the file.
    ///
    /// Returns `true` on success, `false` on error.
    pub fn write_fmt_str(&mut self, args: fmt::Arguments<'_>) -> bool {
        let s = fmt::format(args);
        self.write_bytes(s.as_bytes())
    }

    /// Write a C string (pointer + length) to the file.
    ///
    /// # Safety
    /// `s` must be a valid pointer to at least `len` bytes.
    pub unsafe fn write_c_bytes(&mut self, s: *const u8, len: usize) -> bool {
        if s.is_null() || len == 0 {
            return true;
        }
        let slice = std::slice::from_raw_parts(s, len);
        self.write_bytes(slice)
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Write a newline to fd. Returns OK (1) or FAIL (0).
///
/// # Safety
/// `fd` must be a valid, open `FILE*`.
#[no_mangle]
pub unsafe extern "C" fn rs_put_eol(fd: *mut libc::FILE) -> c_int {
    let mut w = SessionWriter::new(fd);
    w.put_eol()
}

/// Write a NUL-terminated string followed by a newline.
/// Returns OK (1) or FAIL (0).
///
/// # Safety
/// `fd` must be a valid, open `FILE*`. `s` must be a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_put_line(fd: *mut libc::FILE, s: *const c_char) -> c_int {
    let mut w = SessionWriter::new(fd);
    if s.is_null() {
        return w.put_eol();
    }
    let cstr = std::ffi::CStr::from_ptr(s);
    w.put_line(cstr.to_bytes())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_eol() {
        unsafe {
            let fd = libc::tmpfile();
            assert!(!fd.is_null());

            let result = rs_put_eol(fd);
            assert_eq!(result, OK);

            // Verify content
            libc::rewind(fd);
            let mut buf = [0u8; 16];
            let n = libc::fread(buf.as_mut_ptr().cast(), 1, buf.len(), fd);
            assert_eq!(n, 1);
            assert_eq!(buf[0], b'\n');

            libc::fclose(fd);
        }
    }

    #[test]
    fn test_put_line() {
        unsafe {
            let fd = libc::tmpfile();
            assert!(!fd.is_null());

            let s = c"wincmd =";
            let result = rs_put_line(fd, s.as_ptr());
            assert_eq!(result, OK);

            // Verify content
            libc::rewind(fd);
            let mut buf = [0u8; 64];
            let n = libc::fread(buf.as_mut_ptr().cast(), 1, buf.len(), fd);
            assert_eq!(n, 9); // "wincmd =" + "\n"
            assert_eq!(&buf[..9], b"wincmd =\n");

            libc::fclose(fd);
        }
    }

    #[test]
    fn test_put_line_null() {
        unsafe {
            let fd = libc::tmpfile();
            assert!(!fd.is_null());

            let result = rs_put_line(fd, std::ptr::null());
            assert_eq!(result, OK);

            // Verify just a newline was written
            libc::rewind(fd);
            let mut buf = [0u8; 16];
            let n = libc::fread(buf.as_mut_ptr().cast(), 1, buf.len(), fd);
            assert_eq!(n, 1);
            assert_eq!(buf[0], b'\n');

            libc::fclose(fd);
        }
    }

    #[test]
    fn test_session_writer_write_bytes() {
        unsafe {
            let fd = libc::tmpfile();
            assert!(!fd.is_null());

            let mut w = SessionWriter::new(fd);
            assert!(w.write_bytes(b"hello world"));

            libc::rewind(fd);
            let mut buf = [0u8; 32];
            let n = libc::fread(buf.as_mut_ptr().cast(), 1, buf.len(), fd);
            assert_eq!(n, 11);
            assert_eq!(&buf[..11], b"hello world");

            libc::fclose(fd);
        }
    }

    #[test]
    fn test_session_writer_empty_bytes() {
        unsafe {
            let fd = libc::tmpfile();
            assert!(!fd.is_null());

            let mut w = SessionWriter::new(fd);
            assert!(w.write_bytes(b""));

            libc::fclose(fd);
        }
    }
}
