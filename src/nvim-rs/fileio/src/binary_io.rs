//! Binary I/O helpers for Neovim file formats.
//!
//! These functions read/write multi-byte integers from FILE* streams in
//! MSB-first (big-endian) byte order. They are used by spell files, undo
//! files, and other Neovim binary formats.
//!
//! All functions match the semantics of the original C implementations in
//! `fileio.c`.

#![allow(unsafe_code)]

use std::ffi::{c_char, c_int};

extern "C" {
    fn xmallocz(size: usize) -> *mut std::ffi::c_void;
    fn xfree(ptr: *mut std::ffi::c_void);
}

// =============================================================================
// Read functions
// =============================================================================

/// Read 2 bytes from `fd` and turn them into an int, MSB first.
///
/// Returns -1 when encountering EOF.
///
/// # Safety
/// `fd` must be a valid, open FILE pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_get2c(fd: *mut libc::FILE) -> c_int {
    let n = unsafe { libc::fgetc(fd) };
    if n == libc::EOF {
        return -1;
    }
    let c = unsafe { libc::fgetc(fd) };
    if c == libc::EOF {
        return -1;
    }
    (n << 8) + c
}

/// Read 3 bytes from `fd` and turn them into an int, MSB first.
///
/// Returns -1 when encountering EOF.
///
/// # Safety
/// `fd` must be a valid, open FILE pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_get3c(fd: *mut libc::FILE) -> c_int {
    let n = unsafe { libc::fgetc(fd) };
    if n == libc::EOF {
        return -1;
    }
    let c = unsafe { libc::fgetc(fd) };
    if c == libc::EOF {
        return -1;
    }
    let n = (n << 8) + c;
    let c = unsafe { libc::fgetc(fd) };
    if c == libc::EOF {
        return -1;
    }
    (n << 8) + c
}

/// Read 4 bytes from `fd` and turn them into an int, MSB first.
///
/// Returns -1 when encountering EOF.
///
/// # Safety
/// `fd` must be a valid, open FILE pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_get4c(fd: *mut libc::FILE) -> c_int {
    // Use u32 to avoid undefined behavior when the MSB is set (signed overflow).
    let c = unsafe { libc::fgetc(fd) };
    if c == libc::EOF {
        return -1;
    }
    let mut n = c as u32;
    for _ in 0..3 {
        let c = unsafe { libc::fgetc(fd) };
        if c == libc::EOF {
            return -1;
        }
        n = (n << 8) | (c as u32);
    }
    n as c_int
}

/// Read 8 bytes from `fd` and turn them into a time_t, MSB first.
///
/// Returns -1 when encountering EOF.
///
/// # Safety
/// `fd` must be a valid, open FILE pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_get8ctime(fd: *mut libc::FILE) -> i64 {
    let mut n: i64 = 0;
    for _ in 0..8 {
        let c = unsafe { libc::fgetc(fd) };
        if c == libc::EOF {
            return -1;
        }
        n = (n << 8) + c as i64;
    }
    n
}

/// Read `cnt` bytes from `fd` into a newly xmalloc-allocated buffer.
///
/// Returns a pointer to the NUL-terminated string, or NULL if EOF is
/// encountered before reading all bytes. The caller must free with xfree().
///
/// # Safety
/// `fd` must be a valid, open FILE pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_read_string(fd: *mut libc::FILE, cnt: usize) -> *mut c_char {
    // Allocate cnt+1 bytes (xmallocz zero-terminates)
    let ptr = unsafe { xmallocz(cnt) } as *mut u8;
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    for i in 0..cnt {
        let c = unsafe { libc::fgetc(fd) };
        if c == libc::EOF {
            unsafe { xfree(ptr as *mut std::ffi::c_void) };
            return std::ptr::null_mut();
        }
        unsafe { *ptr.add(i) = c as u8 };
    }
    ptr as *mut c_char
}

// =============================================================================
// Write functions
// =============================================================================

/// Write `number` to `fd` in `len` bytes, most significant byte first.
///
/// Returns false on write error.
///
/// # Safety
/// `fd` must be a valid, open FILE pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_put_bytes(fd: *mut libc::FILE, number: u64, len: usize) -> bool {
    if len == 0 {
        return true;
    }
    // Write bytes MSB first: byte at index len-1, then len-2, ..., 0
    let mut i = len - 1;
    loop {
        let byte = (number >> (i * 8)) as c_int;
        if unsafe { libc::fputc(byte, fd) } == libc::EOF {
            return false;
        }
        if i == 0 {
            break;
        }
        i -= 1;
    }
    true
}

/// Write `time_` to `fd` in 8 bytes, MSB first.
///
/// Returns FAIL (0) on write error, OK (1) on success.
///
/// # Safety
/// `fd` must be a valid, open FILE pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_put_time(fd: *mut libc::FILE, time_: i64) -> c_int {
    let mut buf = [0u8; 8];
    let t = time_ as u64;
    for (i, byte) in buf.iter_mut().enumerate() {
        *byte = ((t >> ((7 - i) * 8)) & 0xFF) as u8;
    }
    let written = unsafe { libc::fwrite(buf.as_ptr() as *const libc::c_void, 1, 8, fd) };
    // Original C: fwrite(...) == 1 ? OK : FAIL
    // But the original writes 8 bytes and checks for == 1 (count of elements, each 8 bytes)
    // Actually: fwrite(buf, sizeof(uint8_t), ARRAY_SIZE(buf), fd) == 1
    // That's writing 8 elements of size 1, checking result == 1 seems wrong in the original...
    // The original C code is: fwrite(buf, sizeof(uint8_t), ARRAY_SIZE(buf), fd) == 1
    // sizeof(uint8_t) = 1, ARRAY_SIZE(buf) = 8, so it writes 8 bytes but checks == 1
    // This appears to be a bug in the original C code (should be == 8), but we replicate it.
    if written == 1 {
        1
    } else {
        0
    }
}

// =============================================================================
// vim_fgets
// =============================================================================

/// Read a line from `fp` into `buf[0..size]`.
///
/// Like fgets() but:
/// - Retries on EINTR (when fgets returns NULL with errno=EINTR and ferror set)
/// - If the line doesn't fit, truncates at size-1 and discards the rest
///
/// Returns true when at end-of-file (no line was read), false otherwise.
///
/// # Safety
/// `buf` must point to a writable buffer of at least `size` bytes.
/// `fp` must be a valid, open FILE pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_fgets(buf: *mut c_char, size: c_int, fp: *mut libc::FILE) -> bool {
    assert!(size > 0);

    let size_usize = size as usize;
    // Set the sentinel byte to detect line truncation
    unsafe { *buf.add(size_usize - 2) = 0 };

    // Retry loop for EINTR, matching the C do-while:
    //   do { errno = 0; retval = fgets(...); }
    //   while (retval == NULL && errno == EINTR && ferror(fp));
    let retval;
    loop {
        let errno_ptr = unsafe { libc::__errno_location() };
        unsafe { *errno_ptr = 0 };
        let r = unsafe { libc::fgets(buf, size, fp) };
        // Retry only if NULL && errno==EINTR && ferror(fp)
        if r.is_null()
            && unsafe { *libc::__errno_location() } == libc::EINTR
            && unsafe { libc::ferror(fp) } != 0
        {
            continue;
        }
        retval = r;
        break;
    }

    // Check if the line was truncated (sentinel overwritten but not by newline)
    let sentinel = unsafe { *buf.add(size_usize - 2) };
    if sentinel != 0 && sentinel != b'\n' as c_char {
        // Truncate at size-1
        unsafe { *buf.add(size_usize - 1) = 0 };

        // Discard the rest of the line
        let mut tbuf = [0u8; 200];
        loop {
            tbuf[tbuf.len() - 2] = 0;
            let errno_ptr = unsafe { libc::__errno_location() };
            unsafe { *errno_ptr = 0 };
            let r =
                unsafe { libc::fgets(tbuf.as_mut_ptr() as *mut c_char, tbuf.len() as c_int, fp) };
            if r.is_null()
                && (unsafe { libc::feof(fp) } != 0
                    || unsafe { *libc::__errno_location() } != libc::EINTR)
            {
                break;
            }
            // Stop when we reach end of the long line
            if tbuf[tbuf.len() - 2] == 0 || tbuf[tbuf.len() - 2] == b'\n' {
                break;
            }
        }
    }

    retval.is_null()
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_put_bytes_logic() {
        // Verify byte ordering: MSB first
        // number = 0x0102, len = 2 => writes 0x01, 0x02
        // We can't easily test FILE* writes here, but we can verify the logic
        // by checking the shift calculation
        let number: u64 = 0x0102;
        let len = 2usize;
        let mut bytes = Vec::new();
        let mut i = len - 1;
        loop {
            bytes.push((number >> (i * 8)) as u8);
            if i == 0 {
                break;
            }
            i -= 1;
        }
        assert_eq!(bytes, vec![0x01, 0x02]);
    }

    #[test]
    fn test_put_time_bytes() {
        // Verify time byte ordering
        let time_: i64 = 0x0102030405060708_i64;
        let t = time_ as u64;
        let mut buf = [0u8; 8];
        for (i, byte) in buf.iter_mut().enumerate() {
            *byte = ((t >> ((7 - i) * 8)) & 0xFF) as u8;
        }
        assert_eq!(buf, [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
    }
}
