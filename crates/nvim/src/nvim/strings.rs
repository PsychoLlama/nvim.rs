#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::charset::{rem_backslash, skipwhite, transstr, vim_str2nr};
use crate::src::nvim::eval::encode::{encode_tv2echo, encode_tv2string};
use crate::src::nvim::eval::typval::{
    tv_check_for_number_arg, tv_check_for_opt_bool_arg, tv_check_for_opt_number_arg,
    tv_check_for_opt_string_arg, tv_check_for_string_arg, tv_get_bool, tv_get_bool_chk,
    tv_get_number, tv_get_number_chk, tv_get_string, tv_get_string_buf_chk, tv_get_string_chk,
    tv_list_alloc_ret, tv_list_append_number,
};
use crate::src::nvim::ex_docmd::find_cmdline_var;
use crate::src::nvim::garray::{ga_append, ga_clear, ga_grow, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::keycodes::Ctrl_V;
use crate::src::nvim::main::{e_invarg, e_invarg2, e_using_number_as_bool_nr, e_val_too_large_len};
use crate::src::nvim::mbyte::{
    mb_copy_char, mb_cptr2char_adv, mb_ptr2char_adv, mb_string2cells, mb_tolower, mb_toupper,
    utf_char2bytes, utf_char2len, utf_head_off, utf_ptr2CharInfo, utf_ptr2cells, utf_ptr2char,
    utf_ptr2len, utfc_ptr2len,
};
use crate::src::nvim::memory::{
    arena_alloc, arena_alloc_block, xcalloc, xfree, xmalloc, xmallocz, xmemdupz, xmemscan,
    xrealloc, xstrchrnul, xstrlcpy,
};
use crate::src::nvim::message::{emsg, semsg, siemsg};
use crate::src::nvim::option::{csh_like_shell, fish_like_shell};
use crate::src::nvim::os::libc::{
    __assert_fail, gettext, log10, memcpy, memmove, memset, qsort, snprintf, strcasecmp, strchr,
    strcmp, strcpy, strlen, strncmp, strstr, vsnprintf,
};
use crate::src::nvim::plines::linetabsize_col;
use crate::src::nvim::types::{
    Arena, EvalFuncData, String_0, StringBuilder, VAR_FLOAT, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN,
    float_T, garray_T, int16_t, int64_t, intmax_t, kListLenUnknown, keyvalue_T, ptrdiff_t, size_t,
    typval_T, uint8_t, uint16_t, uintmax_t, uvarnumber_T, varnumber_T,
};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;
use core::slice;

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
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
/// `strnlen`: bytes before the terminator, reading at most `maxlen` bytes.
unsafe fn strnlen(s: *const c_char, maxlen: size_t) -> size_t {
    unsafe {
        let mut n = 0;
        while n < maxlen && *s.add(n) != 0 {
            n += 1;
        }
        n
    }
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
pub unsafe extern "C" fn xstrnsave(string: *const c_char, len: size_t) -> *mut c_char {
    unsafe {
        let n = strnlen(string, len);
        let ret = xmallocz(len) as *mut c_char;
        let out = slice::from_raw_parts_mut(ret as *mut u8, len);
        if n != 0 {
            out[..n].copy_from_slice(slice::from_raw_parts(string as *const u8, n));
        }
        out[n..].fill(0);
        ret
    }
}

/// Truncate unescaped trailing spaces and tabs in place.
pub unsafe extern "C" fn del_trailing_spaces(ptr: *mut c_char) {
    unsafe {
        let len = CStr::from_ptr(ptr).to_bytes().len();
        let s = slice::from_raw_parts_mut(ptr as *mut u8, len);
        let end = trailing_spaces_start(s);
        s[end..].fill(0);
    }
}

/// Case-insensitive `strcmp` equality where NULL only equals NULL.
/// strcasecmp is locale-aware, so the libc call stays.
pub unsafe extern "C" fn striequal(a: *const c_char, b: *const c_char) -> bool {
    unsafe {
        (a.is_null() && b.is_null()) || (!a.is_null() && !b.is_null() && strcasecmp(a, b) == 0)
    }
}

pub unsafe extern "C" fn vim_strnicmp_asc(
    s1: *const c_char,
    s2: *const c_char,
    len: size_t,
) -> c_int {
    unsafe {
        strnicmp_asc(
            CStr::from_ptr(s1).to_bytes(),
            CStr::from_ptr(s2).to_bytes(),
            len,
        )
    }
}

/// Find character `c` (a codepoint, not a byte) in `string`.
pub unsafe extern "C" fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char {
    unsafe {
        if c <= 0 {
            ptr::null_mut()
        } else if c < 0x80 {
            strchr(string, c)
        } else {
            let mut u8char = [0 as c_char; 22];
            let len = utf_char2bytes(c, u8char.as_mut_ptr());
            u8char[len as usize] = 0;
            strstr(string, u8char.as_ptr())
        }
    }
}

unsafe extern "C" fn sort_compare(
    s1: *const ::core::ffi::c_void,
    s2: *const ::core::ffi::c_void,
) -> c_int {
    unsafe { strcmp(*(s1 as *const *const c_char), *(s2 as *const *const c_char)) }
}

pub unsafe extern "C" fn sort_strings(files: *mut *mut c_char, count: c_int) {
    unsafe {
        qsort(
            files as *mut ::core::ffi::c_void,
            count as size_t,
            ::core::mem::size_of::<*mut c_char>(),
            Some(
                sort_compare
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> c_int,
            ),
        );
    }
}

pub unsafe extern "C" fn has_non_ascii(s: *const c_char) -> bool {
    unsafe { !s.is_null() && any_non_ascii(CStr::from_ptr(s).to_bytes()) }
}

/// Freshly allocated `str1 ++ str2`, NUL-terminated.
pub unsafe extern "C" fn concat_str(str1: *const c_char, str2: *const c_char) -> *mut c_char {
    unsafe {
        let a = CStr::from_ptr(str1).to_bytes();
        let b = CStr::from_ptr(str2).to_bytes_with_nul();
        let dest = xmalloc(a.len() + b.len()) as *mut c_char;
        let out = slice::from_raw_parts_mut(dest as *mut u8, a.len() + b.len());
        out[..a.len()].copy_from_slice(a);
        out[a.len()..].copy_from_slice(b);
        dest
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn reverse_text(mut s: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut len: size_t = strlen(s);
        let mut rev: *mut ::core::ffi::c_char =
            xmalloc(len.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        let mut s_i: size_t = 0 as size_t;
        let mut rev_i: size_t = len;
        while s_i < len {
            let mb_len: ::core::ffi::c_int = utfc_ptr2len(s.offset(s_i as isize));
            rev_i = rev_i.wrapping_sub(mb_len as size_t);
            memmove(
                rev.offset(rev_i as isize) as *mut ::core::ffi::c_void,
                s.offset(s_i as isize) as *const ::core::ffi::c_void,
                mb_len as size_t,
            );
            s_i = s_i.wrapping_add((mb_len as size_t).wrapping_sub(1 as size_t));
            s_i = s_i.wrapping_add(1);
        }
        *rev.offset(len as isize) = NUL as ::core::ffi::c_char;
        return rev;
    }
}
pub unsafe extern "C" fn strrep(
    mut src: *const ::core::ffi::c_char,
    mut what: *const ::core::ffi::c_char,
    mut rep: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut pos: *const ::core::ffi::c_char = src;
        let mut whatlen: size_t = strlen(what);
        let mut count: size_t = 0 as size_t;
        loop {
            pos = strstr(pos, what);
            if pos.is_null() {
                break;
            }
            count = count.wrapping_add(1);
            pos = pos.offset(whatlen as isize);
        }
        if count == 0 as size_t {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut replen: size_t = strlen(rep);
        let mut ret: *mut ::core::ffi::c_char = xmalloc(
            strlen(src)
                .wrapping_add(count.wrapping_mul(replen.wrapping_sub(whatlen)))
                .wrapping_add(1 as size_t),
        ) as *mut ::core::ffi::c_char;
        let mut ptr: *mut ::core::ffi::c_char = ret;
        loop {
            pos = strstr(src, what);
            if pos.is_null() {
                break;
            }
            let mut idx: size_t = pos.offset_from(src) as size_t;
            memcpy(
                ptr as *mut ::core::ffi::c_void,
                src as *const ::core::ffi::c_void,
                idx,
            );
            ptr = ptr.offset(idx as isize);
            strcpy(ptr, rep as *mut ::core::ffi::c_char);
            ptr = ptr.offset(replen as isize);
            src = pos.offset(whatlen as isize);
        }
        strcpy(ptr, src as *mut ::core::ffi::c_char);
        return ret;
    }
}
pub unsafe extern "C" fn cmp_keyvalue_value_n(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        let mut kv1: *mut keyvalue_T = a as *mut keyvalue_T;
        let mut kv2: *mut keyvalue_T = b as *mut keyvalue_T;
        return strncmp(
            (*kv1).value,
            (*kv2).value,
            if (*kv1).length > (*kv2).length {
                (*kv1).length
            } else {
                (*kv2).length
            },
        );
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
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
