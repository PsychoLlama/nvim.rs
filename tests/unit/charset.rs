//! Port of `test/unit/charset/vim_str2nr_spec.lua`.

use std::ffi::c_int;
use std::ptr;

use c2rust_neovim::src::nvim::charset::{
    vim_str2nr, STR2NR_ALL, STR2NR_BIN, STR2NR_DEC, STR2NR_FORCE, STR2NR_HEX, STR2NR_OCT,
    STR2NR_OOCT, STR2NR_QUOTE,
};
use c2rust_neovim::src::nvim::types::typval::{uvarnumber_T, varnumber_T};

use crate::support::cstr;

const DEC: c_int = STR2NR_DEC as c_int;
const BIN: c_int = STR2NR_BIN as c_int;
const OCT: c_int = STR2NR_OCT as c_int;
const OOCT: c_int = STR2NR_OOCT as c_int;
const HEX: c_int = STR2NR_HEX as c_int;
const ALL: c_int = STR2NR_ALL as c_int;
const FORCE: c_int = STR2NR_FORCE as c_int;
const QUOTE: c_int = STR2NR_QUOTE as c_int;

/// The out-arguments a case expects. A `None` field is never passed to
/// `vim_str2nr` (matching the Lua spec, which only ever passed the keys it
/// asserted on).
#[derive(Clone, Copy)]
struct Exp {
    len: Option<c_int>,
    num: Option<varnumber_T>,
    unum: Option<uvarnumber_T>,
    pre: Option<c_int>,
}

fn all(len: c_int, num: varnumber_T, unum: uvarnumber_T, pre: c_int) -> Exp {
    Exp {
        len: Some(len),
        num: Some(num),
        unum: Some(unum),
        pre: Some(pre),
    }
}

fn only_len(len: c_int) -> Exp {
    Exp {
        len: Some(len),
        num: None,
        unum: None,
        pre: None,
    }
}

/// Mirror of the spec's `test_vim_str2nr`: run the call once for every
/// subset of the expected out-arguments (NULL for the rest) and assert each
/// passed argument comes back with the expected value.
fn t(s: &str, what: c_int, exp: Exp, maxlen: c_int, strict: bool) {
    let cs = cstr(s);
    // Field indices: 0 = len, 1 = num, 2 = unum, 3 = pre.
    let present: Vec<usize> = [
        exp.len.is_some(),
        exp.num.is_some(),
        exp.unum.is_some(),
        exp.pre.is_some(),
    ]
    .iter()
    .enumerate()
    .filter_map(|(i, &p)| p.then_some(i))
    .collect();

    for mask in 0u32..(1 << present.len()) {
        // Sentinels prove vim_str2nr wrote the slot.
        let mut len: c_int = -42;
        let mut num: varnumber_T = -42;
        let mut unum: uvarnumber_T = 4242;
        let mut pre: c_int = -42;

        let mut p_len: *mut c_int = ptr::null_mut();
        let mut p_num: *mut varnumber_T = ptr::null_mut();
        let mut p_unum: *mut uvarnumber_T = ptr::null_mut();
        let mut p_pre: *mut c_int = ptr::null_mut();

        let passed: Vec<usize> = present
            .iter()
            .enumerate()
            .filter_map(|(b, &f)| (mask & (1 << b) == 0).then_some(f))
            .collect();
        for &field in &passed {
            match field {
                0 => p_len = &mut len,
                1 => p_num = &mut num,
                2 => p_unum = &mut unum,
                _ => p_pre = &mut pre,
            }
        }

        unsafe {
            vim_str2nr(
                cs.as_ptr(),
                p_pre,
                p_len,
                what,
                p_num,
                p_unum,
                maxlen,
                strict,
                ptr::null_mut(),
            );
        }

        let ctx = format!("(s={s:?}, w={what}, m={maxlen}, strict={strict}, mask={mask:b})");
        for &field in &passed {
            match field {
                0 => assert_eq!(exp.len.unwrap(), len, "len {ctx}"),
                1 => assert_eq!(exp.num.unwrap(), num, "num {ctx}"),
                2 => assert_eq!(exp.unum.unwrap(), unum, "unum {ctx}"),
                _ => assert_eq!(exp.pre.unwrap(), pre, "pre {ctx}"),
            }
        }
    }
}

#[test]
fn works_fine_when_it_has_nothing_to_do() {
    for what in [
        0,
        ALL,
        BIN,
        OCT,
        OOCT,
        HEX,
        FORCE + DEC,
        FORCE + BIN,
        FORCE + OCT,
        FORCE + OOCT,
        FORCE + OCT + OOCT,
        FORCE + HEX,
    ] {
        t("", what, all(0, 0, 0, 0), 0, true);
    }
}

#[test]
fn works_with_decimal_numbers() {
    for flags in [
        0,
        BIN,
        OCT,
        HEX,
        OOCT,
        BIN + OCT,
        BIN + HEX,
        OCT + HEX,
        OOCT + HEX,
        ALL,
        FORCE + DEC,
    ] {
        // Check that all digits are recognized.
        t("12345", flags, all(5, 12345, 12345, 0), 0, true);
        t("67890", flags, all(5, 67890, 67890, 0), 0, true);
        t("12345A", flags, only_len(0), 0, true);
        t("67890A", flags, only_len(0), 0, true);
        t("12345A", flags, all(5, 12345, 12345, 0), 0, false);
        t("67890A", flags, all(5, 67890, 67890, 0), 0, false);

        t("42", flags, all(2, 42, 42, 0), 0, true);
        t("42", flags, all(1, 4, 4, 0), 1, true);
        t("42", flags, all(2, 42, 42, 0), 2, true);
        t("42", flags, all(2, 42, 42, 0), 3, true); // includes NUL byte in maxlen

        t("42x", flags, only_len(0), 0, true);
        t("42x", flags, only_len(0), 3, true);
        t("42x", flags, all(2, 42, 42, 0), 0, false);
        t("42x", flags, all(2, 42, 42, 0), 3, false);

        t("-42", flags, all(3, -42, 42, 0), 3, true);
        t("-42", flags, all(1, 0, 0, 0), 1, true);

        t("-42x", flags, only_len(0), 0, true);
        t("-42x", flags, only_len(0), 4, true);
        t("-42x", flags, all(3, -42, 42, 0), 0, false);
        t("-42x", flags, all(3, -42, 42, 0), 4, false);
    }
}

#[test]
fn works_with_binary_numbers() {
    for flags in [BIN, BIN + OCT, BIN + HEX, ALL, FORCE + BIN] {
        let forced = flags > FORCE;
        let bin = if forced { 0 } else { 'b' as c_int };
        let cap = if forced { 0 } else { 'B' as c_int };

        t("0b101", flags, all(5, 5, 5, bin), 0, true);
        t("0b101", flags, all(1, 0, 0, 0), 1, true);
        t("0b101", flags, only_len(0), 2, true);
        t("0b101", flags, all(1, 0, 0, 0), 2, false);
        t("0b101", flags, all(3, 1, 1, bin), 3, true);
        t("0b101", flags, all(4, 2, 2, bin), 4, true);
        t("0b101", flags, all(5, 5, 5, bin), 5, true);
        t("0b101", flags, all(5, 5, 5, bin), 6, true);

        t("0b1012", flags, only_len(0), 0, true);
        t("0b1012", flags, only_len(0), 6, true);
        t("0b1012", flags, all(5, 5, 5, bin), 0, false);
        t("0b1012", flags, all(5, 5, 5, bin), 6, false);

        t("-0b101", flags, all(6, -5, 5, bin), 0, true);
        t("-0b101", flags, all(1, 0, 0, 0), 1, true);
        t("-0b101", flags, all(2, 0, 0, 0), 2, true);
        t("-0b101", flags, only_len(0), 3, true);
        t("-0b101", flags, all(2, 0, 0, 0), 3, false);
        t("-0b101", flags, all(4, -1, 1, bin), 4, true);
        t("-0b101", flags, all(5, -2, 2, bin), 5, true);
        t("-0b101", flags, all(6, -5, 5, bin), 6, true);
        t("-0b101", flags, all(6, -5, 5, bin), 7, true);

        t("-0b1012", flags, only_len(0), 0, true);
        t("-0b1012", flags, only_len(0), 7, true);
        t("-0b1012", flags, all(6, -5, 5, bin), 0, false);
        t("-0b1012", flags, all(6, -5, 5, bin), 7, false);

        t("0B101", flags, all(5, 5, 5, cap), 0, true);
        t("0B101", flags, all(1, 0, 0, 0), 1, true);
        t("0B101", flags, only_len(0), 2, true);
        t("0B101", flags, all(1, 0, 0, 0), 2, false);
        t("0B101", flags, all(3, 1, 1, cap), 3, true);
        t("0B101", flags, all(4, 2, 2, cap), 4, true);
        t("0B101", flags, all(5, 5, 5, cap), 5, true);
        t("0B101", flags, all(5, 5, 5, cap), 6, true);

        t("0B1012", flags, only_len(0), 0, true);
        t("0B1012", flags, only_len(0), 6, true);
        t("0B1012", flags, all(5, 5, 5, cap), 0, false);
        t("0B1012", flags, all(5, 5, 5, cap), 6, false);

        t("-0B101", flags, all(6, -5, 5, cap), 0, true);
        t("-0B101", flags, all(1, 0, 0, 0), 1, true);
        t("-0B101", flags, all(2, 0, 0, 0), 2, true);
        t("-0B101", flags, only_len(0), 3, true);
        t("-0B101", flags, all(2, 0, 0, 0), 3, false);
        t("-0B101", flags, all(4, -1, 1, cap), 4, true);
        t("-0B101", flags, all(5, -2, 2, cap), 5, true);
        t("-0B101", flags, all(6, -5, 5, cap), 6, true);
        t("-0B101", flags, all(6, -5, 5, cap), 7, true);

        t("-0B1012", flags, only_len(0), 0, true);
        t("-0B1012", flags, only_len(0), 7, true);
        t("-0B1012", flags, all(6, -5, 5, cap), 0, false);
        t("-0B1012", flags, all(6, -5, 5, cap), 7, false);

        if forced {
            t("-101", flags, all(4, -5, 5, 0), 0, true);
        }
    }
}

#[test]
fn works_with_octal_numbers_zero_prefix() {
    for flags in [
        OCT,
        OCT + BIN,
        OCT + HEX,
        OCT + OOCT,
        ALL,
        FORCE + OCT,
        FORCE + OOCT,
        FORCE + OCT + OOCT,
    ] {
        let forced = flags > FORCE;
        let oct = if forced { 0 } else { '0' as c_int };

        // Check that all digits are recognized.
        t("012345670", flags, all(9, 2739128, 2739128, oct), 0, true);

        t("054", flags, all(3, 44, 44, oct), 0, true);
        t("054", flags, all(1, 0, 0, 0), 1, true);
        t("054", flags, all(2, 5, 5, oct), 2, true);
        t("054", flags, all(3, 44, 44, oct), 3, true);
        t("0548", flags, all(3, 44, 44, oct), 3, true);
        t("054", flags, all(3, 44, 44, oct), 4, true);

        t("054x", flags, only_len(0), 4, true);
        t("054x", flags, only_len(0), 0, true);
        t("054x", flags, all(3, 44, 44, oct), 4, false);
        t("054x", flags, all(3, 44, 44, oct), 0, false);

        t("-054", flags, all(4, -44, 44, oct), 0, true);
        t("-054", flags, all(1, 0, 0, 0), 1, true);
        t("-054", flags, all(2, 0, 0, 0), 2, true);
        t("-054", flags, all(3, -5, 5, oct), 3, true);
        t("-054", flags, all(4, -44, 44, oct), 4, true);
        t("-0548", flags, all(4, -44, 44, oct), 4, true);
        t("-054", flags, all(4, -44, 44, oct), 5, true);

        t("-054x", flags, only_len(0), 5, true);
        t("-054x", flags, only_len(0), 0, true);
        t("-054x", flags, all(4, -44, 44, oct), 5, false);
        t("-054x", flags, all(4, -44, 44, oct), 0, false);

        if forced {
            t("-54", flags, all(3, -44, 44, 0), 0, true);
            t("-0548", flags, only_len(0), 5, true);
            t("-0548", flags, only_len(0), 0, true);
            t("-0548", flags, all(4, -44, 44, 0), 5, false);
            t("-0548", flags, all(4, -44, 44, 0), 0, false);
        } else {
            t("-0548", flags, all(5, -548, 548, 0), 5, true);
            t("-0548", flags, all(5, -548, 548, 0), 0, true);
        }
    }
}

#[test]
fn works_with_octal_numbers_0o_prefix() {
    for flags in [
        OOCT,
        OOCT + BIN,
        OOCT + HEX,
        OCT + OOCT,
        OCT + OOCT + BIN,
        OCT + OOCT + HEX,
        ALL,
        FORCE + OCT,
        FORCE + OOCT,
        FORCE + OCT + OOCT,
    ] {
        let forced = flags > FORCE;
        let oct = if forced { 0 } else { 'o' as c_int };
        let cap = if forced { 0 } else { 'O' as c_int };

        t("0o054", flags, all(5, 44, 44, oct), 0, true);
        t("0o054", flags, all(1, 0, 0, 0), 1, true);
        t("0o054", flags, only_len(0), 2, true);
        t("0o054", flags, all(3, 0, 0, oct), 3, true);
        t("0o054", flags, all(4, 5, 5, oct), 4, true);
        t("0o054", flags, all(5, 44, 44, oct), 5, true);
        t("0o0548", flags, all(5, 44, 44, oct), 5, true);
        t("0o054", flags, all(5, 44, 44, oct), 6, true);

        t("0o054x", flags, only_len(0), 6, true);
        t("0o054x", flags, only_len(0), 0, true);
        t("0o054x", flags, all(5, 44, 44, oct), 6, false);
        t("0o054x", flags, all(5, 44, 44, oct), 0, false);

        t("-0o054", flags, all(6, -44, 44, oct), 0, true);
        t("-0o054", flags, all(1, 0, 0, 0), 1, true);
        t("-0o054", flags, all(2, 0, 0, 0), 2, true);
        t("-0o054", flags, only_len(0), 3, true);
        t("-0o054", flags, all(4, 0, 0, oct), 4, true);
        t("-0o054", flags, all(5, -5, 5, oct), 5, true);
        t("-0o054", flags, all(6, -44, 44, oct), 6, true);
        t("-0o0548", flags, all(6, -44, 44, oct), 6, true);
        t("-0o054", flags, all(6, -44, 44, oct), 7, true);

        t("-0o054x", flags, only_len(0), 7, true);
        t("-0o054x", flags, only_len(0), 0, true);
        t("-0o054x", flags, all(6, -44, 44, oct), 7, false);
        t("-0o054x", flags, all(6, -44, 44, oct), 0, false);

        t("0O054", flags, all(5, 44, 44, cap), 0, true);
        t("0O054", flags, all(1, 0, 0, 0), 1, true);
        t("0O054", flags, only_len(0), 2, true);
        t("0O054", flags, all(3, 0, 0, cap), 3, true);
        t("0O054", flags, all(4, 5, 5, cap), 4, true);
        t("0O054", flags, all(5, 44, 44, cap), 5, true);
        t("0O0548", flags, all(5, 44, 44, cap), 5, true);
        t("0O054", flags, all(5, 44, 44, cap), 6, true);

        t("0O054x", flags, only_len(0), 6, true);
        t("0O054x", flags, only_len(0), 0, true);
        t("0O054x", flags, all(5, 44, 44, cap), 6, false);
        t("0O054x", flags, all(5, 44, 44, cap), 0, false);

        t("-0O054", flags, all(6, -44, 44, cap), 0, true);
        t("-0O054", flags, all(1, 0, 0, 0), 1, true);
        t("-0O054", flags, all(2, 0, 0, 0), 2, true);
        t("-0O054", flags, only_len(0), 3, true);
        t("-0O054", flags, all(4, 0, 0, cap), 4, true);
        t("-0O054", flags, all(5, -5, 5, cap), 5, true);
        t("-0O054", flags, all(6, -44, 44, cap), 6, true);
        t("-0O0548", flags, all(6, -44, 44, cap), 6, true);
        t("-0O054", flags, all(6, -44, 44, cap), 7, true);

        t("-0O054x", flags, only_len(0), 7, true);
        t("-0O054x", flags, only_len(0), 0, true);
        t("-0O054x", flags, all(6, -44, 44, cap), 7, false);
        t("-0O054x", flags, all(6, -44, 44, cap), 0, false);

        if forced {
            t("-0548", flags, only_len(0), 5, true);
            t("-0548", flags, only_len(0), 0, true);
            t("-0548", flags, all(4, -44, 44, 0), 5, false);
            t("-0548", flags, all(4, -44, 44, 0), 0, false);
            t("-055", flags, all(4, -45, 45, 0), 0, true);
        } else {
            t("-0548", flags, all(5, -548, 548, 0), 5, true);
            t("-0548", flags, all(5, -548, 548, 0), 0, true);
        }
    }
}

#[test]
fn works_with_hexadecimal_numbers() {
    for flags in [HEX, HEX + BIN, HEX + OCT, ALL, FORCE + HEX] {
        let forced = flags > FORCE;
        let hex = if forced { 0 } else { 'x' as c_int };
        let cap = if forced { 0 } else { 'X' as c_int };

        // Check that all digits are recognized.
        t("0x12345", flags, all(7, 74565, 74565, hex), 0, true);
        t("0x67890", flags, all(7, 424080, 424080, hex), 0, true);
        t("0xABCDEF", flags, all(8, 11259375, 11259375, hex), 0, true);
        t("0xabcdef", flags, all(8, 11259375, 11259375, hex), 0, true);

        t("0x101", flags, all(5, 257, 257, hex), 0, true);
        t("0x101", flags, all(1, 0, 0, 0), 1, true);
        t("0x101", flags, only_len(0), 2, true);
        t("0x101", flags, all(1, 0, 0, 0), 2, false);
        t("0x101", flags, all(3, 1, 1, hex), 3, true);
        t("0x101", flags, all(4, 16, 16, hex), 4, true);
        t("0x101", flags, all(5, 257, 257, hex), 5, true);
        t("0x101", flags, all(5, 257, 257, hex), 6, true);

        t("0x101G", flags, only_len(0), 0, true);
        t("0x101G", flags, only_len(0), 6, true);
        t("0x101G", flags, all(5, 257, 257, hex), 0, false);
        t("0x101G", flags, all(5, 257, 257, hex), 6, false);

        t("-0x101", flags, all(6, -257, 257, hex), 0, true);
        t("-0x101", flags, all(1, 0, 0, 0), 1, true);
        t("-0x101", flags, all(2, 0, 0, 0), 2, true);
        t("-0x101", flags, only_len(0), 3, true);
        t("-0x101", flags, all(2, 0, 0, 0), 3, false);
        t("-0x101", flags, all(4, -1, 1, hex), 4, true);
        t("-0x101", flags, all(5, -16, 16, hex), 5, true);
        t("-0x101", flags, all(6, -257, 257, hex), 6, true);
        t("-0x101", flags, all(6, -257, 257, hex), 7, true);

        t("-0x101G", flags, only_len(0), 0, true);
        t("-0x101G", flags, only_len(0), 7, true);
        t("-0x101G", flags, all(6, -257, 257, hex), 0, false);
        t("-0x101G", flags, all(6, -257, 257, hex), 7, false);

        t("0X101", flags, all(5, 257, 257, cap), 0, true);
        t("0X101", flags, all(1, 0, 0, 0), 1, true);
        t("0X101", flags, only_len(0), 2, true);
        t("0X101", flags, all(1, 0, 0, 0), 2, false);
        t("0X101", flags, all(3, 1, 1, cap), 3, true);
        t("0X101", flags, all(4, 16, 16, cap), 4, true);
        t("0X101", flags, all(5, 257, 257, cap), 5, true);
        t("0X101", flags, all(5, 257, 257, cap), 6, true);

        t("0X101G", flags, only_len(0), 0, true);
        t("0X101G", flags, only_len(0), 6, true);
        t("0X101G", flags, all(5, 257, 257, cap), 0, false);
        t("0X101G", flags, all(5, 257, 257, cap), 6, false);

        t("-0X101", flags, all(6, -257, 257, cap), 0, true);
        t("-0X101", flags, all(1, 0, 0, 0), 1, true);
        t("-0X101", flags, all(2, 0, 0, 0), 2, true);
        t("-0X101", flags, only_len(0), 3, true);
        t("-0X101", flags, all(2, 0, 0, 0), 3, false);
        t("-0X101", flags, all(4, -1, 1, cap), 4, true);
        t("-0X101", flags, all(5, -16, 16, cap), 5, true);
        t("-0X101", flags, all(6, -257, 257, cap), 6, true);
        t("-0X101", flags, all(6, -257, 257, cap), 7, true);

        t("-0X101G", flags, only_len(0), 0, true);
        t("-0X101G", flags, only_len(0), 7, true);
        t("-0X101G", flags, all(6, -257, 257, cap), 0, false);
        t("-0X101G", flags, all(6, -257, 257, cap), 7, false);

        if forced {
            t("-101", flags, all(4, -257, 257, 0), 0, true);
        }
    }
}

// Test_str2nr() in test_functions.vim already tests normal usage.
#[test]
fn works_with_weirdly_quoted_numbers() {
    let flags = DEC + QUOTE;
    t("'027", flags, only_len(0), 0, true);
    t("'027", flags, only_len(0), 0, false);
    t("1'2'3'4", flags, all(7, 1234, 1234, 0), 0, true);

    // Counter-intuitive, but like Vim, strict=true should partially accept
    // these: (' and - are not alphanumeric)
    t("7''331", flags, all(1, 7, 7, 0), 0, true);
    t("123'x4", flags, all(3, 123, 123, 0), 0, true);
    t("1337'", flags, all(4, 1337, 1337, 0), 0, true);
    t("-'", flags, all(1, 0, 0, 0), 0, true);

    let flags = HEX + QUOTE;
    let hex = 'x' as c_int;
    t("0x'abcd", flags, only_len(0), 0, true);
    t("0x'abcd", flags, all(1, 0, 0, 0), 0, false);
    t("0xab''cd", flags, all(4, 171, 171, hex), 0, true);
}
