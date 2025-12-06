//! Memory/string utility functions for Neovim
//!
//! This crate provides Rust implementations of pure memory and string
//! utility functions from Neovim's memory.c.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_safety_doc)]
#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
#![allow(clippy::cast_sign_loss)] // c_char to u8 is intentional
#![allow(clippy::option_if_let_else)] // match is clearer for FFI code
#![allow(clippy::naive_bytecount)] // No external dependencies wanted

use std::ffi::c_char;

/// Get the length of a string up to a maximum of `n` bytes.
///
/// Like `strnlen` but always available.
#[no_mangle]
pub unsafe extern "C" fn rs_xstrnlen(s: *const c_char, n: usize) -> usize {
    if s.is_null() {
        return 0;
    }

    let bytes = std::slice::from_raw_parts(s.cast::<u8>(), n);
    bytes.iter().position(|&b| b == 0).unwrap_or(n)
}

/// Find the first occurrence of character `c` in string `str`, or return
/// a pointer to the NUL terminator if not found.
///
/// Like `strchr` but never returns NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_xstrchrnul(str: *const c_char, c: c_char) -> *const c_char {
    if str.is_null() {
        return str;
    }

    let mut p = str;
    let c_byte = c as u8;

    loop {
        let byte = *p as u8;
        if byte == c_byte || byte == 0 {
            return p;
        }
        p = p.add(1);
    }
}

/// Find the first occurrence of byte `c` in memory, or return a pointer
/// one past the end if not found.
///
/// Like `memscan` in Linux kernel.
#[no_mangle]
pub unsafe extern "C" fn rs_xmemscan(addr: *const u8, c: c_char, size: usize) -> *const u8 {
    if addr.is_null() || size == 0 {
        return addr;
    }

    let bytes = std::slice::from_raw_parts(addr, size);
    let c_byte = c as u8;

    match bytes.iter().position(|&b| b == c_byte) {
        Some(pos) => addr.add(pos),
        None => addr.add(size),
    }
}

/// Count occurrences of character `c` in NUL-terminated string `str`.
///
/// Warning: `c` must not be NUL.
#[no_mangle]
pub unsafe extern "C" fn rs_strcnt(str: *const c_char, c: c_char) -> usize {
    if str.is_null() {
        return 0;
    }

    let c_byte = c as u8;
    let mut count = 0usize;
    let mut p = str;

    loop {
        let byte = *p as u8;
        if byte == 0 {
            break;
        }
        if byte == c_byte {
            count += 1;
        }
        p = p.add(1);
    }

    count
}

/// Count occurrences of byte `c` in memory buffer `data` of length `len`.
#[no_mangle]
pub unsafe extern "C" fn rs_memcnt(data: *const u8, c: c_char, len: usize) -> usize {
    if data.is_null() || len == 0 {
        return 0;
    }

    let bytes = std::slice::from_raw_parts(data, len);
    let c_byte = c as u8;

    bytes.iter().filter(|&&b| b == c_byte).count()
}

/// Reverse memchr - find the last occurrence of byte `c` in memory.
///
/// Returns NULL if not found.
#[no_mangle]
pub unsafe extern "C" fn rs_xmemrchr(src: *const u8, c: u8, len: usize) -> *const u8 {
    if src.is_null() || len == 0 {
        return std::ptr::null();
    }

    let bytes = std::slice::from_raw_parts(src, len);

    match bytes.iter().rposition(|&b| b == c) {
        Some(pos) => src.add(pos),
        None => std::ptr::null(),
    }
}

/// Null-safe string equality check.
///
/// Returns true if both are NULL, or both are non-NULL and equal.
#[no_mangle]
pub unsafe extern "C" fn rs_strequal(a: *const c_char, b: *const c_char) -> bool {
    if a.is_null() && b.is_null() {
        return true;
    }
    if a.is_null() || b.is_null() {
        return false;
    }

    // Both are non-null, compare strings
    let mut pa = a;
    let mut pb = b;

    loop {
        let ca = *pa as u8;
        let cb = *pb as u8;

        if ca != cb {
            return false;
        }
        if ca == 0 {
            return true;
        }

        pa = pa.add(1);
        pb = pb.add(1);
    }
}

/// Null-safe bounded string equality check.
///
/// Returns true if both are NULL, or both are non-NULL and equal up to `n` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_strnequal(a: *const c_char, b: *const c_char, n: usize) -> bool {
    if a.is_null() && b.is_null() {
        return true;
    }
    if a.is_null() || b.is_null() {
        return false;
    }
    if n == 0 {
        return true;
    }

    // Both are non-null, compare up to n bytes
    let mut pa = a;
    let mut pb = b;
    let mut remaining = n;

    loop {
        if remaining == 0 {
            return true;
        }

        let ca = *pa as u8;
        let cb = *pb as u8;

        if ca != cb {
            return false;
        }
        if ca == 0 {
            return true;
        }

        pa = pa.add(1);
        pb = pb.add(1);
        remaining -= 1;
    }
}

// Hash type (matches C hash_T = size_t).
type HashT = usize;

/// Writes a `time_t` value to an 8-byte buffer in big-endian format.
///
/// The `time_t` value is converted to an unsigned 64-bit integer and
/// written to the buffer with the most significant byte first.
///
/// # Safety
///
/// `buf` must point to a buffer of at least 8 bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_time_to_bytes(time_: i64, buf: *mut u8) {
    if buf.is_null() {
        return;
    }

    let time_u64 = time_ as u64;
    // Write in big-endian order (most significant byte first)
    for i in 0..8 {
        *buf.add(i) = ((time_u64 >> ((7 - i) * 8)) & 0xFF) as u8;
    }
}

/// Compute hash for a null-terminated string.
///
/// Uses the same polynomial hash algorithm as nvim's hashtab.c:
/// hash = hash * 101 + byte
///
/// # Safety
///
/// `key` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_hash_hash(key: *const c_char) -> HashT {
    if key.is_null() {
        return 0;
    }

    let first_byte = *key as u8;
    if first_byte == 0 {
        return 0;
    }

    let mut hash = first_byte as HashT;
    let mut p = key.add(1);

    loop {
        let byte = *p as u8;
        if byte == 0 {
            break;
        }
        hash = hash.wrapping_mul(101).wrapping_add(byte as HashT);
        p = p.add(1);
    }

    hash
}

/// Compute hash for a string with known length.
///
/// Uses the same polynomial hash algorithm as nvim's hashtab.c.
///
/// # Safety
///
/// `key` must be a valid pointer to at least `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_hash_hash_len(key: *const c_char, len: usize) -> HashT {
    if key.is_null() || len == 0 {
        return 0;
    }

    let first_byte = *key as u8;
    let mut hash = first_byte as HashT;

    for i in 1..len {
        let byte = *key.add(i) as u8;
        hash = hash.wrapping_mul(101).wrapping_add(byte as HashT);
    }

    hash
}

#[cfg(test)]
#[allow(clippy::cast_possible_wrap)]
mod tests {
    use super::*;

    #[test]
    fn test_xstrnlen() {
        unsafe {
            let s = b"hello\0world";
            assert_eq!(rs_xstrnlen(s.as_ptr().cast(), 10), 5);
            assert_eq!(rs_xstrnlen(s.as_ptr().cast(), 3), 3);
            assert_eq!(rs_xstrnlen(s.as_ptr().cast(), 100), 5);

            let s2 = b"no null";
            assert_eq!(rs_xstrnlen(s2.as_ptr().cast(), 7), 7);
        }
    }

    #[test]
    fn test_xstrchrnul() {
        unsafe {
            let s = b"hello world\0";
            // Find 'o'
            let p = rs_xstrchrnul(s.as_ptr().cast(), b'o' as c_char);
            assert_eq!(p.offset_from(s.as_ptr().cast()), 4);

            // Find 'z' - should return pointer to NUL
            let p = rs_xstrchrnul(s.as_ptr().cast(), b'z' as c_char);
            assert_eq!(p.offset_from(s.as_ptr().cast()), 11);

            // Find NUL
            let p = rs_xstrchrnul(s.as_ptr().cast(), 0);
            assert_eq!(p.offset_from(s.as_ptr().cast()), 11);
        }
    }

    #[test]
    fn test_strcnt() {
        unsafe {
            let s = b"hello world\0";
            assert_eq!(rs_strcnt(s.as_ptr().cast(), b'l' as c_char), 3);
            assert_eq!(rs_strcnt(s.as_ptr().cast(), b'o' as c_char), 2);
            assert_eq!(rs_strcnt(s.as_ptr().cast(), b'z' as c_char), 0);
        }
    }

    #[test]
    fn test_memcnt() {
        unsafe {
            let data = b"hello\0world";
            assert_eq!(rs_memcnt(data.as_ptr(), b'l' as c_char, 11), 3);
            assert_eq!(rs_memcnt(data.as_ptr(), b'o' as c_char, 11), 2);
            assert_eq!(rs_memcnt(data.as_ptr(), 0, 11), 1); // count the NUL
        }
    }

    #[test]
    fn test_xmemrchr() {
        unsafe {
            let data = b"hello";
            // Find last 'l'
            let p = rs_xmemrchr(data.as_ptr(), b'l', 5);
            assert!(!p.is_null());
            assert_eq!(p.offset_from(data.as_ptr()), 3);

            // Find 'z' - not found
            let p = rs_xmemrchr(data.as_ptr(), b'z', 5);
            assert!(p.is_null());
        }
    }

    #[test]
    fn test_strequal() {
        unsafe {
            let s1 = b"hello\0";
            let s2 = b"hello\0";
            let s3 = b"world\0";

            assert!(rs_strequal(s1.as_ptr().cast(), s2.as_ptr().cast()));
            assert!(!rs_strequal(s1.as_ptr().cast(), s3.as_ptr().cast()));

            // NULL handling
            assert!(rs_strequal(std::ptr::null(), std::ptr::null()));
            assert!(!rs_strequal(s1.as_ptr().cast(), std::ptr::null()));
            assert!(!rs_strequal(std::ptr::null(), s1.as_ptr().cast()));
        }
    }

    #[test]
    fn test_strnequal() {
        unsafe {
            let s1 = b"hello\0";
            let s2 = b"helloworld\0";

            assert!(rs_strnequal(s1.as_ptr().cast(), s2.as_ptr().cast(), 5));
            assert!(!rs_strnequal(s1.as_ptr().cast(), s2.as_ptr().cast(), 10));

            // NULL handling
            assert!(rs_strnequal(std::ptr::null(), std::ptr::null(), 5));
            assert!(!rs_strnequal(s1.as_ptr().cast(), std::ptr::null(), 5));

            // n == 0
            assert!(rs_strnequal(s1.as_ptr().cast(), s2.as_ptr().cast(), 0));
        }
    }

    #[test]
    fn test_hash_hash() {
        unsafe {
            let key1 = b"hello\0";
            let key2 = b"world\0";
            let key3 = b"hello\0";

            let hash1 = rs_hash_hash(key1.as_ptr().cast());
            let hash2 = rs_hash_hash(key2.as_ptr().cast());
            let hash3 = rs_hash_hash(key3.as_ptr().cast());

            // Same string should have same hash
            assert_eq!(hash1, hash3);

            // Different strings should (usually) have different hashes
            assert_ne!(hash1, hash2);

            // Empty string should hash to 0
            let empty = b"\0";
            assert_eq!(rs_hash_hash(empty.as_ptr().cast()), 0);

            // NULL should hash to 0
            assert_eq!(rs_hash_hash(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_hash_hash_len() {
        unsafe {
            let key = b"hello world\0";

            let hash_full = rs_hash_hash_len(key.as_ptr().cast(), 11);
            let hash_hello = rs_hash_hash_len(key.as_ptr().cast(), 5);

            let hello_only = b"hello\0";
            let hello_hash = rs_hash_hash(hello_only.as_ptr().cast());

            // Hash of first 5 chars should match hash of "hello"
            assert_eq!(hash_hello, hello_hash);

            // Full hash should be different
            assert_ne!(hash_full, hash_hello);

            // Empty length should hash to 0
            assert_eq!(rs_hash_hash_len(key.as_ptr().cast(), 0), 0);
        }
    }

    #[test]
    fn test_time_to_bytes() {
        unsafe {
            let mut buf = [0u8; 8];

            // Test with zero
            rs_time_to_bytes(0, buf.as_mut_ptr());
            assert_eq!(buf, [0, 0, 0, 0, 0, 0, 0, 0]);

            // Test with 1 - should be in last byte
            rs_time_to_bytes(1, buf.as_mut_ptr());
            assert_eq!(buf, [0, 0, 0, 0, 0, 0, 0, 1]);

            // Test with 0x0102030405060708
            rs_time_to_bytes(0x0102030405060708, buf.as_mut_ptr());
            assert_eq!(buf, [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);

            // Test with a realistic timestamp (e.g., 1700000000 = 2023-11-14)
            rs_time_to_bytes(1700000000, buf.as_mut_ptr());
            // 1700000000 = 0x6553f100 in hex
            assert_eq!(buf, [0, 0, 0, 0, 0x65, 0x53, 0xF1, 0x00]);

            // Test with negative value (rare but possible)
            rs_time_to_bytes(-1, buf.as_mut_ptr());
            assert_eq!(buf, [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        }
    }
}
