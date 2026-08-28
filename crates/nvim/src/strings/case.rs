//! Case conversion.
//!
//! Two families that must not be confused.  The `vim_str*_up` set is
//! *ASCII-only and locale-independent* -- it exists so option names and the
//! like fold the same way everywhere -- and each spelling differs only in
//! whether it allocates, copies, or bounds the length.  `strcase_save` is the
//! multibyte one: it folds per character through `mb_toupper`/`mb_tolower` and
//! grows the result when a folded character encodes longer than its original.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::slice;

use super::strnlen;
use crate::mbyte::{mb_tolower, mb_toupper, utf_char2bytes, utf_char2len, utf_ptr2char_info};
use crate::memory::{xmalloc, xrealloc};
use crate::types::size_t;
use ::libc::strlen;

/// ASCII-uppercase `s` in place (bytes ≥ 0x80 untouched).
pub(crate) fn ascii_upcase(s: &mut [u8]) {
    for b in s {
        if b.is_ascii_lowercase() {
            *b -= 0x20;
        }
    }
}

/// ASCII-uppercased copy of `string`.
pub unsafe fn vim_strsave_up(string: *const c_char) -> *mut c_char {
    let p1 = unsafe { xmalloc(strlen(string).wrapping_add(1)) as *mut c_char };
    unsafe { vim_strcpy_up(p1, string) };
    p1
}

/// ASCII-uppercased copy of at most `len` bytes of `string`.
pub unsafe fn vim_strnsave_up(string: *const c_char, len: size_t) -> *mut c_char {
    let p1 = unsafe { xmalloc(len.wrapping_add(1)) as *mut c_char };
    unsafe { vim_strncpy_up(p1, string, len) };
    p1
}

/// ASCII-uppercase the C string in place.
pub unsafe fn vim_strup(p: *mut c_char) {
    let len = unsafe { CStr::from_ptr(p) }.to_bytes().len();
    ascii_upcase(unsafe { slice::from_raw_parts_mut(p as *mut u8, len) });
}

/// `strcpy` that ASCII-uppercases while copying.
pub unsafe fn vim_strcpy_up(dst: *mut c_char, src: *const c_char) {
    let bytes = unsafe { CStr::from_ptr(src) }.to_bytes_with_nul();
    let out = unsafe { slice::from_raw_parts_mut(dst as *mut u8, bytes.len()) };
    out.copy_from_slice(bytes);
    ascii_upcase(&mut out[..bytes.len() - 1]);
}

/// Like `vim_strcpy_up` but copies at most `n` bytes; always terminates.
pub unsafe fn vim_strncpy_up(dst: *mut c_char, src: *const c_char, n: size_t) {
    let len = unsafe { strnlen(src, n) };
    let out = unsafe { slice::from_raw_parts_mut(dst as *mut u8, len + 1) };
    if len != 0 {
        out[..len].copy_from_slice(unsafe { slice::from_raw_parts(src as *const u8, len) });
        ascii_upcase(&mut out[..len]);
    }
    out[len] = 0;
}

/// `memcpy` that ASCII-uppercases while copying: exactly `n` bytes, no
/// terminator.
pub unsafe fn vim_memcpy_up(dst: *mut c_char, src: *const c_char, n: size_t) {
    if n == 0 {
        return;
    }
    let out = unsafe { slice::from_raw_parts_mut(dst as *mut u8, n) };
    out.copy_from_slice(unsafe { slice::from_raw_parts(src as *const u8, n) });
    ascii_upcase(out);
}

/// Case-fold `orig` per character (multibyte-aware), growing the result
/// when a folded character encodes longer than its original.
pub unsafe fn strcase_save(orig: *const c_char, upper: bool) -> *mut c_char {
    let mut orig_len = unsafe { strlen(orig) };
    let mut res = unsafe { xmalloc(orig_len.wrapping_add(1)) as *mut c_char };
    let mut res_index: size_t = 0;
    let mut p = orig;
    while unsafe { *p } != 0 {
        let char_info = unsafe { utf_ptr2char_info(p) };
        let c = if char_info.value < 0 {
            unsafe { *p as u8 as c_int }
        } else {
            char_info.value as c_int
        };
        let newc = if upper { mb_toupper(c) } else { mb_tolower(c) };
        let newl = utf_char2len(newc) as size_t;
        if res_index.wrapping_add(newl) > orig_len {
            let new_size = res_index.wrapping_add(newl).wrapping_add(1);
            res = unsafe { xrealloc(res as *mut c_void, new_size) as *mut c_char };
            orig_len = new_size.wrapping_sub(1);
        }
        unsafe { utf_char2bytes(newc, res.add(res_index)) };
        res_index = res_index.wrapping_add(newl);
        p = unsafe { p.add(char_info.len as usize) };
    }
    unsafe { *res.add(res_index) = 0 };
    res
}
