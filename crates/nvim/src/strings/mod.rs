#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::eval::typval::tv_get_bool_chk;
use crate::keycodes::Ctrl_V;
use crate::mbyte::{utf_char2bytes, utfc_ptr2len};
use crate::memory::{xmalloc, xmallocz};
use crate::os::cshim::{strchr, strstr};
use crate::semsg;
use crate::types::{VAR_UNKNOWN, keyvalue_T, size_t, typval_T};
use ::libc::{qsort, strcasecmp};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::{ptr, slice};

// The carve of the transpiled module; see each child's docs.
mod case;
mod charindex;
mod escape;
mod eval;
mod printf;

pub use self::case::*;
pub use self::charindex::*;
pub use self::escape::*;
pub use self::eval::*;
pub use self::printf::*;

/// Was this optional builtin argument given?
///
/// A Vimscript builtin's argument array is terminated by a `VAR_UNKNOWN`
/// entry rather than by a count, so an absent argument is readable and the
/// question is a type test. Taking a reference keeps this safe — the
/// caller's own block already had to produce one.
pub(crate) fn given(tv: &typval_T) -> bool {
    tv.v_type != VAR_UNKNOWN
}

/// Read an optional boolean argument that must be spelled `0` or `1`.
///
/// Returns `None` after raising the error, which both callers turn into a
/// silent `-1` result.
pub(crate) unsafe fn strict_bool_arg(tv: *mut typval_T) -> Option<bool> {
    let mut error = false;
    let value = unsafe { tv_get_bool_chk(tv, &raw mut error) };
    if error {
        return None;
    }
    if !(0..=1).contains(&value) {
        semsg!("E1023: Using a Number as a Bool: {}", value);
        return None;
    }
    Some(value != 0)
}

/// `strnlen`: bytes before the terminator, reading at most `maxlen` bytes.
unsafe fn strnlen(s: *const c_char, maxlen: size_t) -> size_t {
    let mut n = 0;
    while n < maxlen && unsafe { *s.add(n) } != 0 {
        n += 1;
    }
    n
}

/// Any byte outside 7-bit ASCII?
fn any_non_ascii(s: &[u8]) -> bool {
    s.iter().any(|&b| b >= 0x80)
}

/// Index where removable trailing whitespace starts: spaces/tabs not
/// preceded by a backslash or Ctrl-V, never including the first byte.
fn trailing_spaces_start(s: &[u8]) -> usize {
    let mut end = s.len();
    while end > 1
        && matches!(s[end - 1], b' ' | b'\t')
        && s[end - 2] != b'\\'
        && s[end - 2] != Ctrl_V as u8
    {
        end -= 1;
    }
    end
}

/// ASCII-case-insensitive `strncmp`. Bytes fold exactly as the C code's
/// signed chars did: only A–Z map (down) to lowercase, and bytes ≥ 0x80
/// compare negative.
fn strnicmp_asc(a: &[u8], b: &[u8], len: size_t) -> c_int {
    let fold = |c: u8| -> c_int {
        let c = c as i8 as c_int;
        if !('A' as c_int..='Z' as c_int).contains(&c) {
            c
        } else {
            c + 0x20
        }
    };
    let mut diff = 0;
    for k in 0..len {
        let ca = a.get(k).copied().unwrap_or(0);
        let cb = b.get(k).copied().unwrap_or(0);
        diff = fold(ca) - fold(cb);
        if diff != 0 || ca == 0 {
            break;
        }
    }
    diff
}

/// Copy at most `len` bytes of `string` into a fresh NUL-terminated
/// buffer, zero-filling the remainder (strncpy semantics).
pub unsafe fn xstrnsave(string: *const c_char, len: size_t) -> *mut c_char {
    let n = unsafe { strnlen(string, len) };
    let ret = unsafe { xmallocz(len) as *mut c_char };
    let out = unsafe { slice::from_raw_parts_mut(ret as *mut u8, len) };
    if n != 0 {
        out[..n].copy_from_slice(unsafe { slice::from_raw_parts(string as *const u8, n) });
    }
    out[n..].fill(0);
    ret
}

/// Truncate unescaped trailing spaces and tabs in place.
pub unsafe fn del_trailing_spaces(ptr: *mut c_char) {
    let len = unsafe { CStr::from_ptr(ptr) }.to_bytes().len();
    let s = unsafe { slice::from_raw_parts_mut(ptr as *mut u8, len) };
    let end = trailing_spaces_start(s);
    s[end..].fill(0);
}

/// Case-insensitive `strcmp` equality where NULL only equals NULL.
/// strcasecmp is locale-aware, so the libc call stays.
pub unsafe fn striequal(a: *const c_char, b: *const c_char) -> bool {
    (a.is_null() && b.is_null())
        || (!a.is_null() && !b.is_null() && unsafe { strcasecmp(a, b) } == 0)
}

pub unsafe fn vim_strnicmp_asc(s1: *const c_char, s2: *const c_char, len: size_t) -> c_int {
    strnicmp_asc(
        unsafe { CStr::from_ptr(s1) }.to_bytes(),
        unsafe { CStr::from_ptr(s2) }.to_bytes(),
        len,
    )
}

/// Find character `c` (a codepoint, not a byte) in `string`.
pub unsafe fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char {
    if c <= 0 {
        ptr::null_mut()
    } else if c < 0x80 {
        unsafe { strchr(string, c) }
    } else {
        let mut u8char = [0 as c_char; 22];
        let len = unsafe { utf_char2bytes(c, u8char.as_mut_ptr()) };
        u8char[len as usize] = 0;
        unsafe { strstr(string, u8char.as_ptr()) }
    }
}

unsafe extern "C" fn sort_compare(
    s1: *const ::core::ffi::c_void,
    s2: *const ::core::ffi::c_void,
) -> c_int {
    unsafe { cstr::cmp(*(s1 as *const *const c_char), *(s2 as *const *const c_char)) as c_int }
}

pub unsafe fn sort_strings(files: *mut *mut c_char, count: c_int) {
    type Compare = unsafe extern "C" fn(*const c_void, *const c_void) -> c_int;
    let base = files as *mut c_void;
    let count = count as size_t;
    let width = ::core::mem::size_of::<*mut c_char>();
    unsafe { qsort(base, count, width, Some(sort_compare as Compare)) };
}

pub unsafe fn has_non_ascii(s: *const c_char) -> bool {
    unsafe { !s.is_null() && any_non_ascii(CStr::from_ptr(s).to_bytes()) }
}

/// Freshly allocated `str1 ++ str2`, NUL-terminated.
pub unsafe fn concat_str(str1: *const c_char, str2: *const c_char) -> *mut c_char {
    let a = unsafe { CStr::from_ptr(str1) }.to_bytes();
    let b = unsafe { CStr::from_ptr(str2) }.to_bytes_with_nul();
    let dest = unsafe { xmalloc(a.len() + b.len()) as *mut c_char };
    let out = unsafe { slice::from_raw_parts_mut(dest as *mut u8, a.len() + b.len()) };
    out[..a.len()].copy_from_slice(a);
    out[a.len()..].copy_from_slice(b);
    dest
}
/// Reverse `s` character by character into freshly allocated memory.
///
/// Composing sequences move as a unit — `utfc_ptr2len` gives the length of
/// the whole character at each position — so the source is walked forwards
/// while the destination is filled from the back.
pub unsafe extern "C" fn reverse_text(s: *mut c_char) -> *mut c_char {
    let len = unsafe { CStr::from_ptr(s) }.to_bytes().len();
    // `xmallocz` writes the terminator the C wrote by hand.
    let rev = unsafe { xmallocz(len) as *mut c_char };
    let src = unsafe { slice::from_raw_parts(s as *const u8, len) };
    let dst = unsafe { slice::from_raw_parts_mut(rev as *mut u8, len) };
    let mut at = len;
    let mut i = 0;
    while i < len {
        // Never past the terminator: `utfc_ptr2len` stops there.
        let char_len = unsafe { utfc_ptr2len(s.add(i)) as usize };
        at -= char_len;
        dst[at..at + char_len].copy_from_slice(&src[i..i + char_len]);
        i += char_len;
    }
    rev
}

/// Every occurrence of `what` in `src` replaced by `rep`, freshly
/// allocated, or NULL when `what` does not occur at all.
pub unsafe fn strrep(src: *const c_char, what: *const c_char, rep: *const c_char) -> *mut c_char {
    let what_len = unsafe { cstr::bytes_at(what) }.len();

    let mut count: size_t = 0;
    let mut pos = src;
    loop {
        pos = unsafe { strstr(pos, what) };
        if pos.is_null() {
            break;
        }
        count += 1;
        pos = unsafe { pos.add(what_len) };
    }
    if count == 0 {
        return ptr::null_mut();
    }

    // `replen - whatlen` underflows when the replacement is shorter;
    // the product then wraps back to the right (smaller) total.
    let rep_len = unsafe { cstr::bytes_at(rep) }.len();
    let size = unsafe { cstr::bytes_at(src) }
        .len()
        .wrapping_add(count.wrapping_mul(rep_len.wrapping_sub(what_len)))
        .wrapping_add(1);
    let ret = unsafe { xmalloc(size) as *mut c_char };

    let mut src = src;
    let mut out = ret;
    loop {
        pos = unsafe { strstr(src, what) };
        if pos.is_null() {
            break;
        }
        let prefix = unsafe { pos.offset_from(src) as size_t };
        unsafe { ptr::copy_nonoverlapping(src, out, prefix) };
        out = unsafe { out.add(prefix) };
        unsafe { ptr::copy_nonoverlapping(rep, out, rep_len) };
        out = unsafe { out.add(rep_len) };
        src = unsafe { pos.add(what_len) };
    }
    let tail = unsafe { cstr::bytes_at(src) }.len();
    unsafe { ptr::copy_nonoverlapping(src, out, tail + 1) };
    ret
}

/// `qsort` comparator: two `keyvalue_T` by value, case-sensitively.
///
/// The comparison length is the *longer* of the two, so a prefix sorts
/// before the string it prefixes — `strncmp` stops at the shorter one's
/// terminator either way.
pub unsafe fn cmp_keyvalue_value_n(a: *const c_void, b: *const c_void) -> ::core::ffi::c_int {
    let kv1 = unsafe { &*(a as *const keyvalue_T) };
    let kv2 = unsafe { &*(b as *const keyvalue_T) };
    let n = kv1.length.max(kv2.length);
    // SAFETY: two NUL-terminated keys of the caller's table.
    unsafe { cstr::prefix_cmp(kv1.value, kv2.value, n) as c_int }
}
#[cfg(test)]
mod tests {
    use super::{any_non_ascii, ascii_upcase, strnicmp_asc, trailing_spaces_start, unquote};

    fn unquote_all(src: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        unquote(src, &mut |b| out.push(b));
        out
    }

    #[test]
    fn unquote_mirrors_the_unit_spec_cases() {
        assert_eq!(unquote_all(b"abc"), b"abc"); // unquoted copies as-is
        assert_eq!(unquote_all(br#""abc""#), b"abc"); // fully quoted word
        assert_eq!(unquote_all(br#"a"b"c"#), b"abc"); // partially quoted
        assert_eq!(unquote_all(br#"a""b"#), b"ab"); // removes ""
        assert_eq!(unquote_all(br#""a\"b""#), br#"a"b"#); // unescapes \"
        assert_eq!(unquote_all(br#""a\\b""#), br#"a\b"#); // unescapes doubled backslash
        assert_eq!(unquote_all(br#"a\\b"#), br#"a\\b"#); // but not outside quotes
        assert_eq!(unquote_all(br#""a\nb""#), br#"a\nb"#); // \n is not unescaped
        assert_eq!(unquote_all(br#""abc"#), b"abc"); // unpaired quote stripped
        assert_eq!(unquote_all(br#"a\"#), br#"a\"#); // may end with one backslash
    }

    #[test]
    fn strnicmp_folds_only_ascii_and_stops_at_len_diff_or_nul() {
        assert_eq!(strnicmp_asc(b"abc", b"ABC", 3), 0);
        assert!(strnicmp_asc(b"abc", b"abd", 3) < 0);
        assert_eq!(strnicmp_asc(b"abX", b"abY", 2), 0); // len clamps the compare
        assert!(strnicmp_asc(b"ab", b"abc", 5) < 0); // terminator vs 'c'
        assert_eq!(strnicmp_asc(b"", b"", 4), 0);
        // Bytes >= 0x80 compare as signed chars, exactly like the C code.
        assert!(strnicmp_asc(b"\x80", b"\x7f", 1) < 0);
    }

    #[test]
    fn trailing_spaces_respect_escapes_and_never_take_byte_zero() {
        assert_eq!(trailing_spaces_start(b"ab  "), 2);
        assert_eq!(trailing_spaces_start(b"ab\t "), 2);
        assert_eq!(trailing_spaces_start(b"ab\\  "), 4); // escaped space stays
        assert_eq!(trailing_spaces_start(&[b'a', 22, b' ']), 3); // Ctrl-V escapes
        assert_eq!(trailing_spaces_start(b" "), 1); // first byte never stripped
        assert_eq!(trailing_spaces_start(b""), 0);
    }

    #[test]
    fn upcase_and_ascii_scan() {
        let mut buf = *b"aZ9\x80!";
        ascii_upcase(&mut buf);
        assert_eq!(&buf, b"AZ9\x80!");
        assert!(any_non_ascii(b"caf\xc3\xa9"));
        assert!(!any_non_ascii(b"cafe"));
    }
}
