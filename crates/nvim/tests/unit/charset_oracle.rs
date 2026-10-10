//! The pointer-form `charset` readers and writers, folded into one number
//! each over `support::corpus` (plus a corpus of numbers), recorded from the
//! forms upstream ported. A rewrite onto slices has to reproduce every
//! digest; see `mbyte_oracle.rs` for the rule.

#![cfg(not(miri))]

use std::ffi::{c_char, c_int};

use neovim::charset::{
    Str2NrBases, backslash_halve, backslash_halve_save, getdigits, getdigits_int, getdigits_int32,
    hexhex2nr, ptr2cells, rem_backslash, skipbin, skipdigits, skiphex, skiptodigit, skiptowhite,
    skiptowhite_esc, skipwhite, str_foldcase, trans_characters, transchar_hex, transstr,
    transstr_buf, vim_iswordp, vim_str2nr, vim_strnsize, vim_strsize,
};
use neovim::ex_docmd::do_cmdline_cmd;
use neovim::types::typval::{UVarNumber, VarNumber};

use crate::support::corpus::{Digest, corpus};
use crate::support::{editor_lock, take_bytes};

/// `s` as a NUL-terminated C buffer, with `room` bytes of slack after it.
fn cbuf(s: &[u8], room: usize) -> Vec<c_char> {
    s.iter()
        .map(|&b| b as c_char)
        .chain(std::iter::repeat_n(0, room + 1))
        .collect()
}

/// The bytes up to `buf`'s first NUL.
fn until_nul(buf: &[c_char]) -> Vec<u8> {
    buf.iter()
        .take_while(|&&b| b != 0)
        .map(|&b| b as u8)
        .collect()
}

/// Numbers in every shape the digit readers distinguish, and the corpus.
fn texts() -> Vec<Vec<u8>> {
    let mut out: Vec<Vec<u8>> = [
        &b"0"[..],
        b"7",
        b"42",
        b"-5",
        b"+7",
        b" 42",
        b"\t\n 7x",
        b"007",
        b"0x1f",
        b"0X1F",
        b"0b101",
        b"0B2",
        b"0o17",
        b"0O8",
        b"08",
        b"0777",
        b"1'000'000",
        b"99999999999999999999999",
        b"-99999999999999999999999",
        b"9223372036854775807",
        b"9223372036854775808",
        b"-9223372036854775808",
        b"2147483647",
        b"2147483648",
        b"-2147483649",
        b"4294967296",
        b"12ab",
        b"0x",
        b"-",
        b"--1",
        b"1e5",
        b"3.25",
        b"inf",
        b"-inf",
        b"nan",
        b"\\ a\\\\b\\",
        b"ab\\",
        b"\\\\\\",
        b"\x16 x",
        b"a\\ b c",
        b"ff",
        b"fG",
        b"7f80",
    ]
    .iter()
    .map(|s| s.to_vec())
    .collect();
    out.extend(corpus());
    out
}

/// Run `f` over every text with the character tables at their defaults.
fn digest(f: impl Fn(&mut Digest, &[u8])) -> u64 {
    let _editor = editor_lock();
    for cmd in [
        c"set isprint=@,161-255 display=lastline,msgsep",
        c"setlocal iskeyword=@,48-57,_,192-255",
    ] {
        do_cmdline_cmd(cmd).expect("the option values are valid");
    }
    let mut d = Digest::new();
    for uhex in [false, true] {
        let cmd = if uhex {
            c"set display+=uhex"
        } else {
            c"set display-=uhex"
        };
        do_cmdline_cmd(cmd).expect("valid");
        for text in texts() {
            f(&mut d, &text);
        }
    }
    do_cmdline_cmd(c"set display-=uhex").expect("valid");
    d.value()
}

#[test]
fn the_skippers() {
    let got = digest(|d, text| {
        let buf = cbuf(text, 0);
        let p = buf.as_ptr();
        for i in 0..=text.len() {
            // SAFETY: `buf` is NUL-terminated and `i` is inside it; every
            // answer is a pointer into it.
            unsafe {
                let at = p.add(i).cast_mut();
                for q in [
                    skipwhite(at),
                    skipdigits(at),
                    skipbin(at).cast_mut(),
                    skiphex(at),
                    skiptodigit(at),
                    skiptowhite(at),
                    skiptowhite_esc(at),
                ] {
                    d.add(q.offset_from(p) as i64);
                }
                d.add(i64::from(rem_backslash(at)));
                if i + 2 <= text.len() {
                    d.add(i64::from(hexhex2nr(at)));
                }
                d.add(i64::from(vim_iswordp(at)));
                d.add(i64::from(ptr2cells(at)));
            }
        }
    });
    assert_eq!(got, 0xd22b63655c7bd5a5, "skippers");
}

#[test]
fn the_digit_readers() {
    let got = digest(|d, text| {
        for i in 0..=text.len() {
            let buf = cbuf(text, 0);
            let p = buf.as_ptr().cast_mut();
            for strict in [false, true] {
                // SAFETY: `buf` is NUL-terminated and each cursor stays
                // inside it.
                unsafe {
                    let mut cursor = p.add(i);
                    d.add(getdigits(&raw mut cursor, strict, -7) as i64);
                    d.add(cursor.offset_from(p) as i64);
                    let mut cursor = p.add(i);
                    d.add(i64::from(getdigits_int(&raw mut cursor, strict, -7)));
                    d.add(cursor.offset_from(p) as i64);
                    let mut cursor = p.add(i);
                    d.add(i64::from(getdigits_int32(&raw mut cursor, strict, -7)));
                    d.add(cursor.offset_from(p) as i64);
                }
            }
            for what in [
                Str2NrBases::NONE,
                Str2NrBases::ALL,
                Str2NrBases::HEX,
                Str2NrBases::BIN | Str2NrBases::OOCT,
                Str2NrBases::OCT | Str2NrBases::QUOTE,
            ] {
                for maxlen in [0, 1, 3, (text.len() - i) as c_int] {
                    for strict in [false, true] {
                        let (mut pre, mut len, mut overflow) = (-1, -1, false);
                        let (mut n, mut un): (VarNumber, UVarNumber) = (-1, 1);
                        // SAFETY: NUL-terminated, every out-argument a local.
                        unsafe {
                            vim_str2nr(
                                p.add(i),
                                &raw mut pre,
                                &raw mut len,
                                what,
                                &raw mut n,
                                &raw mut un,
                                maxlen,
                                strict,
                                &raw mut overflow,
                            );
                        }
                        for v in [i64::from(pre), i64::from(len), n, un as i64] {
                            d.add(v);
                        }
                        d.add(i64::from(overflow));
                    }
                }
            }
        }
    });
    assert_eq!(got, 0x77aec3102eaf1505, "digit readers");
}

#[test]
fn the_display_forms() {
    let got = digest(|d, text| {
        let buf = cbuf(text, 0);
        let p = buf.as_ptr();
        // SAFETY: every call below reads `buf` to its NUL and writes only
        // into buffers sized as each documents.
        unsafe {
            d.add(i64::from(vim_strsize(p)));
            for n in [0, 1, 2, 5, text.len() as c_int, 1000] {
                d.add(i64::from(vim_strnsize(p, n)));
            }
            for untab in [false, true] {
                // `transstr_len`, gone with its one caller: the length
                // `transstr` allocates for.
                d.add(take_bytes(transstr(p, untab)).len() as i64);
                d.bytes(&take_bytes(transstr(p, untab)));
                for buflen in [1, 2, 5, 9, 64] {
                    let mut out = vec![0x55 as c_char; buflen];
                    for slen in [-1, 0, 3, text.len() as isize] {
                        let n = transstr_buf(p, slen, out.as_mut_ptr(), buflen, untab);
                        d.add(n as i64);
                        d.bytes(&until_nul(&out));
                    }
                }
            }
            for bufsize in [text.len() + 1, text.len() + 4, text.len() + 40] {
                let mut copy = cbuf(text, 40);
                trans_characters(copy.as_mut_ptr(), bufsize as c_int);
                d.bytes(&until_nul(&copy));
            }
            let orglen = text.len() as c_int;
            let mut copy = cbuf(text, 0);
            d.bytes(&take_bytes(str_foldcase(
                copy.as_mut_ptr(),
                orglen,
                std::ptr::null_mut(),
                0,
            )));
            for buflen in [1, 4, orglen + 1, orglen + 8] {
                let mut out = vec![0 as c_char; buflen as usize];
                str_foldcase(copy.as_mut_ptr(), orglen, out.as_mut_ptr(), buflen);
                d.bytes(&until_nul(&out));
            }
            let mut halved = cbuf(text, 0);
            backslash_halve(halved.as_mut_ptr());
            d.bytes(&until_nul(&halved));
            d.bytes(&take_bytes(backslash_halve_save(p)));
        }
    });
    assert_eq!(got, 0x2823cae5750b6133, "display forms");
}

#[test]
fn the_hex_form() {
    let _editor = editor_lock();
    let mut d = Digest::new();
    for c in (0..0x400).chain([0xffff, 0x10000, 0x10ffff, 0x7fff_ffff]) {
        let mut buf = [0 as c_char; 16];
        // SAFETY: sixteen bytes hold the widest form.
        let n = unsafe { transchar_hex(buf.as_mut_ptr(), c) };
        d.add(n as i64);
        d.bytes(&until_nul(&buf));
    }
    assert_eq!(d.value(), 0x8903a5a44af10085, "hex form");
}
