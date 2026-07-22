//! Port of `test/unit/strings_spec.lua`.

use std::ffi::{c_char, c_int, c_long, c_longlong, c_uint, c_ulong, c_ulonglong};

use c2rust_neovim::src::nvim::strings::{
    reverse_text, strcase_save, vim_snprintf, vim_strchr, vim_strnsave_unquoted,
    vim_strsave_escaped,
};
use c2rust_neovim::src::nvim::types::typval::uvarnumber_T;

use crate::support::{cstr, internalize, take_bytes};

fn escaped(s: &str, chars: &str) -> String {
    let s = cstr(s);
    let chars = cstr(chars);
    unsafe { internalize(vim_strsave_escaped(s.as_ptr(), chars.as_ptr())) }
}

#[test]
fn vim_strsave_escaped_precedes_by_a_backslash_all_chars_from_second_argument() {
    assert_eq!(r"\a\b\c\d", escaped("abcd", "abcd"));
}

#[test]
fn vim_strsave_escaped_precedes_by_a_backslash_chars_only_from_second_argument() {
    assert_eq!(r"\a\bcd", escaped("abcd", "ab"));
}

#[test]
fn vim_strsave_escaped_returns_a_copy_of_passed_string_if_second_argument_is_empty() {
    assert_eq!("text \n text", escaped("text \n text", ""));
}

#[test]
fn vim_strsave_escaped_returns_an_empty_string_if_first_argument_is_empty_string() {
    assert_eq!("", escaped("", "\r"));
}

#[test]
fn vim_strsave_escaped_returns_a_copy_if_it_does_not_contain_chars_from_2nd_argument() {
    assert_eq!("some text", escaped("some text", "a"));
}

fn unquoted_n(s: &str, len: usize) -> String {
    let s = cstr(s);
    unsafe { internalize(vim_strnsave_unquoted(s.as_ptr(), len)) }
}

fn unquoted(s: &str) -> String {
    unquoted_n(s, s.len())
}

#[test]
fn vim_strnsave_unquoted_copies_unquoted_strings_as_is() {
    assert_eq!("-c", unquoted("-c"));
    assert_eq!("", unquoted(""));
}

#[test]
fn vim_strnsave_unquoted_respects_length_argument() {
    assert_eq!("", unquoted_n("-c", 0));
    assert_eq!("-", unquoted_n("-c", 1));
    assert_eq!("-", unquoted_n("\"-c", 2));
}

#[test]
fn vim_strnsave_unquoted_unquotes_fully_quoted_word() {
    assert_eq!("/bin/sh", unquoted("\"/bin/sh\""));
}

#[test]
fn vim_strnsave_unquoted_unquotes_partially_quoted_word() {
    assert_eq!("/Program Files/sh", unquoted("/Program\" \"Files/sh"));
}

#[test]
fn vim_strnsave_unquoted_removes_adjacent_quote_pairs() {
    assert_eq!("/Program Files/sh", unquoted("/\"\"Program\" \"Files/sh"));
}

#[test]
fn vim_strnsave_unquoted_performs_unescaping_of_quotes() {
    assert_eq!(
        "/\"Program Files\"/sh",
        unquoted(r#"/"\""Program Files"\""/sh"#)
    );
}

#[test]
fn vim_strnsave_unquoted_performs_unescaping_of_backslashes() {
    assert_eq!(
        r"/\Program Files\foo/sh",
        unquoted(r#"/"\\"Program Files"\\foo"/sh"#)
    );
}

#[test]
fn vim_strnsave_unquoted_strips_quote_when_there_is_no_pair_to_it() {
    assert_eq!("/Program Files/sh", unquoted("/Program\" Files/sh"));
    assert_eq!("", unquoted("\""));
}

#[test]
fn vim_strnsave_unquoted_allows_string_to_end_with_one_backslash_unescaped() {
    assert_eq!("/Program Files/sh\\", unquoted("/Program\" Files/sh\\"));
}

#[test]
fn vim_strnsave_unquoted_does_not_perform_unescaping_out_of_quotes() {
    assert_eq!(r"/Program\ Files/sh\", unquoted(r"/Program\ Files/sh\"));
}

#[test]
fn vim_strnsave_unquoted_does_not_unescape_backslash_n() {
    assert_eq!(r"/Program\nFiles/sh", unquoted("/Program\"\\n\"Files/sh"));
}

fn strchr(s: &[u8], c: c_int) -> Option<isize> {
    let s = cstr(s);
    let found = unsafe { vim_strchr(s.as_ptr(), c) };
    if found.is_null() {
        None
    } else {
        Some(unsafe { found.cast_const().offset_from(s.as_ptr()) })
    }
}

#[test]
fn vim_strchr_handles_nul_and_negative_correctly() {
    assert_eq!(None, strchr(b"abc", 0));
    assert_eq!(None, strchr(b"abc", -1));
}

#[test]
fn vim_strchr_works() {
    assert_eq!(Some(0), strchr(b"abc", 'a' as c_int));
    assert_eq!(Some(1), strchr(b"abc", 'b' as c_int));
    assert_eq!(Some(2), strchr(b"abc", 'c' as c_int));
    assert_eq!(Some(0), strchr("a«b»c".as_bytes(), 'a' as c_int));
    assert_eq!(Some(3), strchr("a«b»c".as_bytes(), 'b' as c_int));
    assert_eq!(Some(6), strchr("a«b»c".as_bytes(), 'c' as c_int));

    // The first *byte* of '«' names U+00C2, which is not in the string.
    assert_eq!(None, strchr("«»".as_bytes(), 0xC2));
    // 0xAB == 171 == '«', but a lone 0xAB byte is not valid UTF-8 for it.
    assert_eq!(None, strchr(b"\xAB", 0xAB));
    assert_eq!(Some(0), strchr("«»".as_bytes(), 0xAB));
    assert_eq!(Some(3), strchr("„«»“".as_bytes(), 0xAB));

    assert_eq!(Some(7), strchr("„«»“".as_bytes(), 0x201C));
    assert_eq!(None, strchr("„«»“".as_bytes(), 0x201D));
    assert_eq!(Some(0), strchr("„«»“".as_bytes(), 0x201E));

    assert_eq!(Some(0), strchr(b"\xF4\x8F\xBC\x80", 0x10FF00));
    assert_eq!(Some(2), strchr("«\u{10FF00}»".as_bytes(), 0x10FF00));
    // 0xDBFF 0xDF00 — a surrogate pair for 0x10FF00 is not a match.
    assert_eq!(
        None,
        strchr(b"\xC2\xAB\xED\xAF\xBF\xED\xBC\x80\xC2\xBB", 0x10FF00)
    );
}

/// Mirror of the spec's `a()` helper: assert `vim_snprintf` into a
/// `bsize`-byte buffer returns the untruncated length and writes
/// `expected` truncated to `bsize - 1` bytes plus NUL.
macro_rules! snp {
    ($expected:expr, $buf:expr, $bsize:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {{
        let expected: &str = $expected;
        let bsize: usize = $bsize;
        let fmt = cstr($fmt);
        let n = unsafe { vim_snprintf($buf, bsize, fmt.as_ptr() $(, $arg)*) };
        let ctx = format!("snprintf(buf, {}, {:?})", bsize, $fmt);
        assert_eq!(expected.len() as c_int, n, "{ctx}");
        if bsize > 0 {
            let take = std::cmp::min(expected.len() + 1, bsize);
            let actual = unsafe { std::slice::from_raw_parts($buf as *const u8, take) };
            let mut want = expected.as_bytes()[..take - 1].to_vec();
            want.push(0);
            assert_eq!(want, actual, "{ctx}");
        }
    }};
}

fn uv(n: u64) -> uvarnumber_T {
    n
}

#[test]
fn vim_snprintf_truncation() {
    for bsize in 0..=14usize {
        let mut storage = vec![0u8; bsize.max(1)];
        let buf = storage.as_mut_ptr() as *mut c_char;
        let one = cstr("one");
        let two = cstr("two");
        let hanyu = cstr("漢語");
        let foo = cstr("foo");
        snp!("1.00000001e7", buf, bsize, "%.8g", 10000000.1f64);
        snp!("1234567", buf, bsize, "%d", 1234567 as c_int);
        snp!("1234567", buf, bsize, "%ld", 1234567 as c_long);
        snp!("  1234567", buf, bsize, "%9ld", 1234567 as c_long);
        snp!("1234567  ", buf, bsize, "%-9ld", 1234567 as c_long);
        snp!("deadbeef", buf, bsize, "%x", 0xdeadbeef_u32 as c_uint);
        snp!("001100", buf, bsize, "%06b", uv(12));
        snp!("one two", buf, bsize, "%s %s", one.as_ptr(), two.as_ptr());
        snp!("1.234000", buf, bsize, "%f", 1.234f64);
        snp!("1.234000e+00", buf, bsize, "%e", 1.234f64);
        snp!("nan", buf, bsize, "%f", f64::NAN);
        snp!("inf", buf, bsize, "%f", f64::INFINITY);
        snp!("-inf", buf, bsize, "%f", f64::NEG_INFINITY);
        snp!("-0.000000", buf, bsize, "%f", -0.0f64);
        snp!("漢語", buf, bsize, "%s", hanyu.as_ptr());
        snp!("  漢語", buf, bsize, "%8s", hanyu.as_ptr());
        snp!("漢語  ", buf, bsize, "%-8s", hanyu.as_ptr());
        snp!("漢", buf, bsize, "%.3s", hanyu.as_ptr());
        snp!("  foo", buf, bsize, "%5S", foo.as_ptr());
        snp!("%%%", buf, bsize, "%%%%%%");
        snp!(
            "0x87654321",
            buf,
            bsize,
            "%p",
            0x87654321usize as *mut c_char
        );
        #[rustfmt::skip]
        snp!("0x0087654321", buf, bsize, "%012p", 0x87654321usize as *mut c_char);
    }
}

#[test]
fn vim_snprintf_positional_arguments() {
    for bsize in 0..=24usize {
        let mut storage = vec![0u8; bsize.max(1)];
        let buf = storage.as_mut_ptr() as *mut c_char;
        let one = cstr("one");
        let two = cstr("two");
        let three = cstr("three");
        let hi = cstr("hi");
        #[rustfmt::skip]
        snp!("1234567  ", buf, bsize, "%1$*2$ld", 1234567 as c_long, -9 as c_int);
        #[rustfmt::skip]
        snp!("1234567  ", buf, bsize, "%1$*2$.*3$ld", 1234567 as c_long, -9 as c_int, 5 as c_int);
        #[rustfmt::skip]
        snp!("1234567  ", buf, bsize, "%1$*3$.*2$ld", 1234567 as c_long, 5 as c_int, -9 as c_int);
        #[rustfmt::skip]
        snp!("1234567  ", buf, bsize, "%3$*1$.*2$ld", -9 as c_int, 5 as c_int, 1234567 as c_long);
        snp!("1234567", buf, bsize, "%1$ld", 1234567 as c_long);
        #[rustfmt::skip]
        snp!("  1234567", buf, bsize, "%1$*2$ld", 1234567 as c_long, 9 as c_int);
        #[rustfmt::skip]
        snp!("9 12345 7654321", buf, bsize, "%2$ld %1$d %3$lu", 12345 as c_int, 9 as c_long, 7654321 as c_ulong);
        #[rustfmt::skip]
        snp!("9 1234567 7654321", buf, bsize, "%2$d %1$ld %3$lu", 1234567 as c_long, 9 as c_int, 7654321 as c_ulong);
        #[rustfmt::skip]
        snp!("9 1234567 7654321", buf, bsize, "%2$d %1$lld %3$lu", 1234567 as c_longlong, 9 as c_int, 7654321 as c_ulong);
        #[rustfmt::skip]
        snp!("9 12345 7654321", buf, bsize, "%2$ld %1$u %3$lu", 12345 as c_uint, 9 as c_long, 7654321 as c_ulong);
        #[rustfmt::skip]
        snp!("9 1234567 7654321", buf, bsize, "%2$d %1$lu %3$lu", 1234567 as c_ulong, 9 as c_int, 7654321 as c_ulong);
        #[rustfmt::skip]
        snp!("9 1234567 7654321", buf, bsize, "%2$d %1$llu %3$lu", 1234567 as c_ulonglong, 9 as c_int, 7654321 as c_ulong);
        #[rustfmt::skip]
        snp!("9 deadbeef 7654321", buf, bsize, "%2$d %1$x %3$lu", 0xdeadbeef_u32 as c_uint, 9 as c_int, 7654321 as c_ulong);
        #[rustfmt::skip]
        snp!("9 c 7654321", buf, bsize, "%2$ld %1$c %3$lu", 'c' as c_int, 9 as c_long, 7654321 as c_ulong);
        #[rustfmt::skip]
        snp!("9 hi 7654321", buf, bsize, "%2$ld %1$s %3$lu", hi.as_ptr(), 9 as c_long, 7654321 as c_ulong);
        #[rustfmt::skip]
        snp!("9 0.000000e+00 7654321", buf, bsize, "%2$ld %1$e %3$lu", 0.0f64, 9 as c_long, 7654321 as c_ulong);
        #[rustfmt::skip]
        snp!("two one two", buf, bsize, "%2$s %1$s %2$s", one.as_ptr(), two.as_ptr(), three.as_ptr());
        #[rustfmt::skip]
        snp!("three one two", buf, bsize, "%3$s %1$s %2$s", one.as_ptr(), two.as_ptr(), three.as_ptr());
        snp!("1234567", buf, bsize, "%1$d", 1234567 as c_int);
        snp!("deadbeef", buf, bsize, "%1$x", 0xdeadbeef_u32 as c_uint);
        snp!("001100", buf, bsize, "%2$0*1$b", 6 as c_int, uv(12));
        snp!("001100", buf, bsize, "%1$0.*2$b", uv(12), 6 as c_int);
        snp!(
            "one two",
            buf,
            bsize,
            "%1$s %2$s",
            one.as_ptr(),
            two.as_ptr()
        );
        snp!("001100", buf, bsize, "%06b", uv(12));
        snp!(
            "two one",
            buf,
            bsize,
            "%2$s %1$s",
            one.as_ptr(),
            two.as_ptr()
        );
        snp!("1.234000", buf, bsize, "%1$f", 1.234f64);
        snp!("1.234000e+00", buf, bsize, "%1$e", 1.234f64);
        snp!("nan", buf, bsize, "%1$f", f64::NAN);
        snp!("inf", buf, bsize, "%1$f", f64::INFINITY);
        snp!("-inf", buf, bsize, "%1$f", f64::NEG_INFINITY);
        snp!("-0.000000", buf, bsize, "%1$f", -0.0f64);
    }
}

#[test]
fn vim_snprintf_zd_and_zu() {
    let bsize = 20usize;
    let mut storage = vec![0u8; bsize];
    let buf = storage.as_mut_ptr() as *mut c_char;
    #[rustfmt::skip]
    snp!("-1234567 -7654321", buf, bsize, "%zd %zd", -1234567 as isize, -7654321 as isize);
    #[rustfmt::skip]
    snp!("-7654321 -1234567", buf, bsize, "%2$zd %1$zd", -1234567 as isize, -7654321 as isize);
    #[rustfmt::skip]
    snp!("1234567 7654321", buf, bsize, "%zu %zu", 1234567 as usize, 7654321 as usize);
    #[rustfmt::skip]
    snp!("7654321 1234567", buf, bsize, "%2$zu %1$zu", 1234567 as usize, 7654321 as usize);
}

#[test]
fn strcase_save_decodes_overlong_encoded_characters() {
    // 0xC1 0x81 is an overlong encoding of 'A'.
    let overlong = cstr(&b"\xC1\x81"[..]);
    unsafe {
        assert_eq!("A", internalize(strcase_save(overlong.as_ptr(), true)));
        assert_eq!("a", internalize(strcase_save(overlong.as_ptr(), false)));
    }
}

fn reversed(s: &[u8]) -> Vec<u8> {
    // reverse_text takes a mutable pointer; give it its own copy.
    let mut buf = s.to_vec();
    buf.push(0);
    unsafe { take_bytes(reverse_text(buf.as_mut_ptr().cast())) }
}

#[test]
fn reverse_text_handles_empty_string() {
    assert_eq!(b"", reversed(b"").as_slice());
}

#[test]
fn reverse_text_handles_simple_cases() {
    assert_eq!(b"a", reversed(b"a").as_slice());
    assert_eq!(b"ba", reversed(b"ab").as_slice());
}

#[test]
fn reverse_text_handles_multibyte_characters() {
    assert_eq!("bα".as_bytes(), reversed("αb".as_bytes()));
    assert_eq!("Yötön yö".as_bytes(), reversed("öy nötöY".as_bytes()));
}

#[test]
fn reverse_text_handles_combining_chars() {
    const RING_ABOVE: &[u8] = b"\xCC\x8A";
    const RING_BELOW: &[u8] = b"\xCC\xA5";
    let input = [b"aaa", RING_ABOVE, RING_BELOW, b"bb"].concat();
    let expected = [b"bba", RING_ABOVE, RING_BELOW, b"aa"].concat();
    assert_eq!(expected, reversed(&input));
}

#[test]
fn reverse_text_treats_invalid_utf_as_separate_characters() {
    assert_eq!(b"\xC0ba", reversed(b"ab\xC0").as_slice());
}

#[test]
fn reverse_text_treats_an_incomplete_utf_continuation_sequence_as_valid() {
    assert_eq!(b"\xC2ba", reversed(b"ab\xC2").as_slice());
}
