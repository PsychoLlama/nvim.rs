//! The `charset` readers and writers, folded into one number each over
//! `support::corpus` (plus a corpus of numbers). The digests were recorded
//! from the pointer forms upstream ported, which read a C string: each case
//! hands the slice forms the bytes such a string held -- `text[i..]` up to
//! its first NUL -- and has to reproduce every digest; see
//! `mbyte_oracle.rs` for the rule.

#![cfg(not(miri))]

use neovim::mbyte::Codepoint;
use std::ffi::c_int;

use neovim::charset::display::{foldcase, trans_characters_in, transstr_into, transstr_owned};
use neovim::charset::{
    Str2NrBases, backslash_halved, escapes_next, getdigits_at, getdigits_int_at, hexhex_at,
    is_word_at, ptr2cells_at, skip, str2nr_scan, strnsize, strsize, transchar_hex_into,
};
use neovim::ex_docmd::do_cmdline_cmd;
use neovim::types::typval::{UVarNumber, VarNumber};
use neovim::winlayer::Buf;

use crate::support::corpus::{Digest, corpus};
use crate::support::editor_lock;

/// The bytes up to the first NUL: what a C reader handed `text` sees.
fn until_nul(text: &[u8]) -> &[u8] {
    &text[..text.iter().position(|&b| b == 0).unwrap_or(text.len())]
}

/// `text` with `room` bytes of NUL slack after it.
fn padded(text: &[u8], room: usize) -> Vec<u8> {
    text.iter()
        .copied()
        .chain(std::iter::repeat_n(0, room + 1))
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
        for i in 0..=text.len() {
            let c = until_nul(&text[i..]);
            for offset in [
                skip::white(c),
                skip::digits(c),
                skip::bin(c),
                skip::hex(c),
                skip::to_digit(&text[i..]),
                skip::to_white(c),
                skip::to_white_esc(c),
            ] {
                d.add((i + offset) as i64);
            }
            d.add(i64::from(escapes_next(c)));
            if i + 2 <= text.len() {
                d.add(i64::from(hexhex_at(&text[i..])));
            }
            d.add(i64::from(is_word_at(c, Buf::current())));
            d.add(i64::from(ptr2cells_at(&text[i..])));
        }
    });
    assert_eq!(got, 0xd22b63655c7bd5a5, "skippers");
}

#[test]
fn the_digit_readers() {
    let got = digest(|d, text| {
        for i in 0..=text.len() {
            let c = until_nul(&text[i..]);
            for strict in [false, true] {
                let (n, used) = getdigits_at(c, 0, strict, -7);
                d.add(n as i64);
                d.add((i + used) as i64);
                // `getdigits_int` and its `int32_t` twin, one reader now.
                for _ in 0..2 {
                    let (n, used) = getdigits_int_at(c, 0, strict, -7);
                    d.add(i64::from(n));
                    d.add((i + used) as i64);
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
                    // A positive `maxlen` bounds the bytes the parse sees,
                    // as the pointer form's `strncmp`-style span did.
                    let bounded = match usize::try_from(maxlen) {
                        Ok(n @ 1..) => &c[..n.min(c.len())],
                        _ => c,
                    };
                    for strict in [false, true] {
                        let (mut pre, mut n, mut un): (c_int, VarNumber, UVarNumber) = (-1, -1, 1);
                        let parsed = str2nr_scan(bounded, maxlen, what, strict, true);
                        let len = c_int::try_from(parsed.len).unwrap_or(c_int::MAX);
                        let overflow = parsed.overflow;
                        if !parsed.rejected {
                            pre = c_int::from(parsed.prefix);
                            n = parsed.value;
                            un = parsed.magnitude;
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
        let c = until_nul(text);
        d.add(i64::from(strsize(text)));
        for n in [0, 1, 2, 5, text.len() as c_int, 1000] {
            d.add(i64::from(strnsize(text, n)));
        }
        for untab in [false, true] {
            // `transstr_len`, gone with its one caller: the length
            // `transstr` allocates for.
            let shown = transstr_owned(c, untab);
            d.add(shown.as_cstr().to_bytes().len() as i64);
            d.bytes(shown.as_cstr().to_bytes());
            for buflen in [1, 2, 5, 9, 64] {
                let mut out = vec![0x55u8; buflen];
                for slen in [-1, 0, 3, text.len() as isize] {
                    let n = transstr_into(c, usize::try_from(slen).ok(), &mut out, untab);
                    d.add(n as i64);
                    d.bytes(until_nul(&out));
                }
            }
        }
        for bufsize in [text.len() + 1, text.len() + 4, text.len() + 40] {
            let mut copy = padded(text, 40);
            trans_characters_in(&mut copy[..bufsize]);
            d.bytes(until_nul(&copy));
        }
        let orglen = text.len() as c_int;
        d.bytes(until_nul(&foldcase(text, None)));
        for buflen in [1, 4, orglen + 1, orglen + 8] {
            let folded = foldcase(text, Some(buflen as usize));
            d.bytes(until_nul(&folded));
        }
        let halved = backslash_halved(c);
        d.bytes(until_nul(&halved));
        d.bytes(until_nul(&backslash_halved(c)));
    });
    assert_eq!(got, 0x2823cae5750b6133, "display forms");
}

#[test]
fn the_hex_form() {
    let _editor = editor_lock();
    let mut d = Digest::new();
    for c in (0..0x400).chain([0xffff, 0x10000, 0x10ffff, 0x7fff_ffff]) {
        let mut buf = [0u8; 16];
        let n = transchar_hex_into(&mut buf, Codepoint::new(c));
        d.add(n as i64);
        d.bytes(until_nul(&buf));
    }
    assert_eq!(d.value(), 0x8903a5a44af10085, "hex form");
}
