//! The `strings` copiers, escapers and case folders, and the Vimscript
//! string builtins over them, folded into one number each over
//! `support::corpus`. Recorded from the pointer forms upstream ported; the
//! rewrite onto slices and owned answers has to reproduce every digest (see
//! `mbyte_oracle.rs` for the rule).

#![cfg(not(miri))]

use std::ffi::{CString, c_char, c_int};

use neovim::ex_docmd::do_cmdline_cmd;
use neovim::strings::{
    concat_str, del_trailing_spaces, reverse_text, strcase_save, strrep, vim_strchr,
    vim_strnsave_unquoted, vim_strsave_escaped_ext, vim_strsave_shellescape, xstrnsave,
};

use crate::support::corpus::{Digest, corpus};
use crate::support::{Probe, editor_lock, take_bytes};

/// The corpus plus the shapes the escapers and the quote resolver single out.
fn texts() -> Vec<Vec<u8>> {
    let mut out: Vec<Vec<u8>> = [
        &b""[..],
        b"plain",
        b"it's",
        b"'",
        b"a'b'c",
        b"line\nbreak",
        b"bang!",
        b"back\\slash",
        b"\\",
        b"%",
        b"#",
        b"<cword> and %:p",
        b"<sfile><slnum><SID>",
        b"#<1",
        b"\"quoted\"",
        b"\"a\\\"b\"",
        b"\"a\\\\b\"",
        b"\"unpaired",
        b"trail  ",
        b"trail\\ ",
        b"trail\t \t",
        b"\x16 ",
        b"  ",
        b"ABCdef",
        "ÄÖÜäöüß".as_bytes(),
        "ǅǈǋ".as_bytes(),
        "İı".as_bytes(),
        "ﬀ".as_bytes(),
        b"\xc1\x81",
        b"\xe0\x80\x80",
        b"\xf4\x90\x80\x80",
    ]
    .iter()
    .map(|s| s.to_vec())
    .collect();
    out.extend(corpus());
    out
}

fn c(s: &[u8]) -> CString {
    CString::new(s).unwrap()
}

#[test]
fn escapers_and_quote_resolver() {
    let _editor = editor_lock();
    let mut d = Digest::new();
    for shell in [c"set shell=sh", c"set shell=/bin/csh", c"set shell=fish"] {
        do_cmdline_cmd(shell).expect("valid");
        for text in texts() {
            let s = c(&text);
            for (special, newline) in [(false, false), (true, false), (false, true), (true, true)] {
                // SAFETY: a terminated string; the answer is the caller's.
                d.bytes(&unsafe {
                    take_bytes(vim_strsave_shellescape(s.as_ptr(), special, newline))
                });
            }
        }
    }
    do_cmdline_cmd(c"set shell=sh").expect("valid");
    for text in texts() {
        let s = c(&text);
        for esc in [c"", c" \t\\", c"\"'", c"%#!", c"\xc3\xa9"] {
            for (cc, bsl) in [(b'\\', false), (b'\\', true), (b'^', false), (b'^', true)] {
                // SAFETY: as above.
                d.bytes(&unsafe {
                    take_bytes(vim_strsave_escaped_ext(
                        s.as_ptr(),
                        esc.as_ptr(),
                        cc as c_char,
                        bsl,
                    ))
                });
            }
        }
        for len in 0..=text.len() {
            // SAFETY: `len` bytes of a terminated string.
            d.bytes(&unsafe { take_bytes(vim_strnsave_unquoted(s.as_ptr(), len)) });
        }
    }
    assert_eq!(d.value(), 0x81fcd9d3482c73a6, "escapers");
}

#[test]
fn copiers_and_case_folders() {
    let _editor = editor_lock();
    let mut d = Digest::new();
    let texts = texts();
    for text in &texts {
        let s = c(text);
        // SAFETY: terminated strings throughout; every answer is owned.
        unsafe {
            d.bytes(&take_bytes(strcase_save(s.as_ptr(), true)));
            d.bytes(&take_bytes(strcase_save(s.as_ptr(), false)));
            d.bytes(&take_bytes(reverse_text(s.as_ptr().cast_mut())));
            for n in 0..=text.len() {
                d.bytes(&take_bytes(xstrnsave(s.as_ptr(), n)));
            }
            let mut copy = s.as_bytes_with_nul().to_vec();
            del_trailing_spaces(copy.as_mut_ptr().cast());
            d.bytes(&copy);
            d.bytes(&take_bytes(concat_str(s.as_ptr(), c"<>".as_ptr())));
            d.bytes(&take_bytes(concat_str(c"<>".as_ptr(), s.as_ptr())));
            for (what, rep) in [
                (c"a", c"XYZ"),
                (c"'", c"''"),
                (c"\\", c""),
                (c"\xcc\x81", c"+"),
            ] {
                let got = strrep(s.as_ptr(), what.as_ptr(), rep.as_ptr());
                if got.is_null() {
                    d.add(-1);
                } else {
                    d.bytes(&take_bytes(got));
                }
            }
            for ch in [
                0x61, 0x27, 0x5c, 0x20, 0xe9, 0xc3, 0x301, 0x1f600, 0x200d, 0xfffd, 0x10ffff, 0, -1,
            ] {
                let at = vim_strchr(s.as_ptr(), ch as c_int);
                d.add(if at.is_null() {
                    -1
                } else {
                    at.cast_const().offset_from(s.as_ptr()) as i64
                });
            }
        }
    }
    assert_eq!(d.value(), 0x3dae324ac4dd8722, "copiers");
}

/// `s` as a Vimscript double-quoted string, every byte escaped.
fn vim_quoted(s: &[u8]) -> String {
    let mut out = String::from("\"");
    for b in s {
        out.push_str(&format!("\\x{b:02x}"));
    }
    out.push('"');
    out
}

#[test]
fn string_builtins() {
    let editor = editor_lock();
    let probe = Probe::new(&editor);
    let mut d = Digest::new();
    let texts = texts();
    for text in &texts {
        let q = vim_quoted(text);
        // The quoting reaches the editor intact, or every row below is moot.
        assert_eq!(probe.eval(&format!("strlen({q})")), text.len().to_string());
        for e in [
            format!("tolower({q})"),
            format!("toupper({q})"),
            format!("strtrans({q})"),
            format!("strwidth({q})"),
            format!("strdisplaywidth({q})"),
            format!("strdisplaywidth({q}, 3)"),
            format!("strlen({q})"),
            format!("strchars({q})"),
            format!("strchars({q}, 1)"),
            format!("strcharlen({q})"),
            format!("str2list({q})"),
            format!("str2nr({q})"),
            format!("str2nr({q}, 16)"),
            format!("str2nr({q}, 2)"),
            format!("str2nr({q}, 8, 1)"),
            format!("stridx({q}, 'a')"),
            format!("stridx({q}, '')"),
            format!("stridx({q}, 'a', 2)"),
            format!("stridx({q}, 'a', -3)"),
            format!("strridx({q}, 'a')"),
            format!("strridx({q}, '')"),
            format!("strridx({q}, 'a', 3)"),
            format!("strridx({q}, '', 2)"),
            format!("trim({q})"),
            format!("trim({q}, 'a\\', 1)"),
            format!("tr({q}, 'aé', 'Aè')"),
            format!("shellescape({q})"),
            format!("shellescape({q}, 1)"),
            format!("escape({q}, ' \\')"),
            format!("fnameescape({q})"),
            format!("printf('%5S|%-4.2S|%s', {q}, {q}, {q})"),
        ] {
            d.bytes(probe.eval(&e).as_bytes());
        }
    }
    for e in [
        "str2nr(' -0x1f', 16)",
        "str2nr('+ 12')",
        "str2nr('0b101', 2)",
        "str2nr('1''000', 10, 1)",
        "str2nr('99999999999999999999')",
        "str2nr('-99999999999999999999')",
        "stridx('abc', 'c', 3)",
        "strridx('abcabc', 'b', -1)",
    ] {
        d.bytes(probe.eval(e).as_bytes());
    }
    assert_eq!(d.value(), 0xa0a92d93fbc62791, "builtins");
}
