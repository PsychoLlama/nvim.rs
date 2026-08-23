//! `test/unit/eval/decode_spec.lua` and `test/unit/eval/tricks_spec.lua`.
//!
//! Every case needs a live editor, which Miri cannot start.

#![cfg(not(miri))]

use std::ffi::c_int;

use c2rust_neovim::eval::decode::json_decode_string;
use c2rust_neovim::eval::typval::tv_clear;
use c2rust_neovim::main::emsg_silent;
use c2rust_neovim::memory::{xfree, xmemdup};
use c2rust_neovim::types::{VAR_UNKNOWN, VAR_UNLOCKED, typval_T, typval_vval_union};

use crate::support::alloc::AllocLog;
use crate::support::tv::{self, Tv};
use crate::support::{check_emsg_bytes, cstr};

/// `emsg_silent` raised for the caller's scope and put back on drop.
///
/// The Lua harness forked a child per case, so it could raise this and
/// never lower it; here the next case would inherit a silent editor and its
/// message assertions would all pass vacuously.
struct Silent(c_int);

impl Silent {
    fn new() -> Silent {
        let saved = emsg_silent.get();
        emsg_silent.set(1);
        Silent(saved)
    }
}

impl Drop for Silent {
    fn drop(&mut self) {
        emsg_silent.set(self.0);
    }
}

/// An unset `typval_T` for the decoder to write into.
fn unset() -> typval_T {
    typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }
}

/// `describe('json_decode_string()')`'s three "does not overflow" cases,
/// spec lines 21, 49 and 60.
///
/// The point of every row is `buf_len`: the decoder must read exactly that
/// many bytes, so `json_decode_string("null", 1, …)` sees `n` and fails
/// rather than seeing `null` and succeeding.
#[test]
fn decoding_reads_no_further_than_the_length_it_was_given() {
    let _log = AllocLog::start();
    let _silent = Silent::new();
    // SAFETY: every buffer outlives the call that reads it, and `rettv` is
    // this case's own.
    unsafe {
        let mut rettv = unset();
        for (text, len) in [
            ("null", 1),
            ("true", 1),
            ("false", 1),
            ("null", 2),
            ("true", 2),
            ("false", 2),
            ("null", 3),
            ("true", 3),
            ("false", 3),
            ("false", 4),
            // A string that is opened and not closed within `len`.
            ("\"t\"", 2),
            ("\"\"", 1),
        ] {
            let buf = cstr(text);
            assert_eq!(
                json_decode_string(buf.as_ptr(), len, &raw mut rettv),
                0,
                "{text:?} at {len}"
            );
            assert_eq!(rettv.v_type, VAR_UNKNOWN, "{text:?} at {len}");
        }
    }
}

/// The same `describe`'s two "does not overflow and crash" cases, spec
/// lines 49 and 128.
///
/// These pass a buffer that is *exactly* one byte with no terminator, so a
/// decoder that read one byte too far would be caught by the sanitizer
/// rather than by an assertion — which is why the case exists at all and
/// why the copy must not be a `CString`.
#[test]
fn decoding_a_lone_byte_reads_only_that_byte() {
    let _log = AllocLog::start();
    let _silent = Silent::new();
    // SAFETY: `one` is a one-byte allocation, freed below; nothing reads
    // past it unless the decoder is wrong, which is the assertion.
    unsafe {
        let mut rettv = unset();
        for &byte in b"ntf\"" {
            let one = xmemdup((&raw const byte).cast(), 1);
            assert_eq!(
                json_decode_string(one.cast(), 1, &raw mut rettv),
                0,
                "{:?}",
                byte as char
            );
            assert_eq!(rettv.v_type, VAR_UNKNOWN);
            xfree(one);
        }
    }
}

/// The same `describe`'s `itp('does not overflow in error messages')`, spec
/// line 78, plus `itp('does not overflow with `-`')` at line 122.
///
/// Every message quotes the input, and the quote must also stop at
/// `buf_len` — each row's trailing `test` is there to be *absent* from the
/// message.
///
/// Input and expectation are both **byte strings**. Two rows quote a byte
/// that is not valid UTF-8, and the spec spelled those `'\194'` and
/// `'\252\144\128\128\128\128'` — one byte and six bytes. Written as Rust
/// `\u{}` escapes they would be two and twelve, and the assertion would be
/// about something else.
#[test]
fn a_decoder_error_quotes_no_more_than_it_read() {
    let log = AllocLog::start();
    // SAFETY: every buffer outlives its call; `rettv` is this case's own.
    unsafe {
        let mut rettv = unset();
        let mut check = |text: &[u8], len: usize, msg: &[u8]| {
            let buf = text.to_vec();
            let ret = check_emsg_bytes(
                log.editor(),
                || json_decode_string(buf.as_ptr().cast(), len, &raw mut rettv),
                Some(msg),
            );
            assert_eq!(ret, 0, "{:?}", String::from_utf8_lossy(text));
            assert_eq!(rettv.v_type, VAR_UNKNOWN);
            log.clear();
        };

        check(b"]test", 1, b"E474: No container to close: ]");
        check(b"[}test", 2, b"E474: Closing list with curly bracket: }");
        check(
            b"{]test",
            2,
            b"E474: Closing dictionary with square bracket: ]",
        );
        check(b"[1,]test", 4, b"E474: Trailing comma: ]");
        check(br#"{"1":}test"#, 6, b"E474: Expected value after colon: }");
        check(br#"{"1"}test"#, 5, b"E474: Expected value: }");
        check(b",test", 1, b"E474: Comma not inside container: ,");
        check(b"[1,,1]test", 6, b"E474: Duplicate comma: ,1]");
        check(br#"{"1":,}test"#, 7, b"E474: Comma after colon: ,}");
        check(
            br#"{"1",}test"#,
            6,
            b"E474: Using comma in place of colon: ,}",
        );
        check(b"{,}test", 3, b"E474: Leading comma: ,}");
        check(b"[,]test", 3, b"E474: Leading comma: ,]");
        check(b":test", 1, b"E474: Colon not inside container: :");
        check(b"[:]test", 3, b"E474: Using colon not in dictionary: :]");
        check(b"{:}test", 3, b"E474: Unexpected colon: :}");
        check(br#"{"1"::1}test"#, 8, b"E474: Duplicate colon: :1}");
        check(b"ntest", 1, b"E474: Expected null: n");
        check(b"ttest", 1, b"E474: Expected true: t");
        check(b"ftest", 1, b"E474: Expected false: f");
        check(br#""\test"#, 2, br#"E474: Unfinished escape sequence: "\"#);
        check(
            br#""\u"test"#,
            4,
            br#"E474: Unfinished unicode escape sequence: "\u""#,
        );
        check(
            br#""\uXXXX"est"#,
            8,
            br#"E474: Expected four hex digits after \u: \uXXXX""#,
        );
        check(br#""\?"test"#, 4, br#"E474: Unknown escape sequence: \?""#);
        check(
            b"\"\t\"test",
            3,
            b"E474: ASCII control characters cannot be present inside string: \t\"",
        );
        check(
            b"\"\xC2\"test",
            3,
            b"E474: Only UTF-8 strings allowed: \xC2\"",
        );
        // `concat!` would not do here: it joins `&str`s, and there is no
        // `&str` holding these six bytes.
        let mut unescaped =
            b"E474: Only UTF-8 code points up to U+10FFFF are allowed to appear unescaped: "
                .to_vec();
        unescaped.extend_from_slice(b"\xFC\x90\x80\x80\x80\x80\"");
        check(b"\"\xFC\x90\x80\x80\x80\x80\"test", 8, &unescaped);
        check(b"\"test", 1, b"E474: Expected string end: \"");
        check(b"-test", 1, b"E474: Missing number after minus sign: -");
        check(
            b"-1.test",
            3,
            b"E474: Missing number after decimal dot: -1.",
        );
        check(b"-1.0etest", 5, b"E474: Missing exponent: -1.0e");
        check(b"?test", 1, b"E474: Unidentified byte: ?");
        check(b"1?test", 2, b"E474: Trailing characters: ?");
        check(b"[1test", 2, b"E474: Unexpected end of input: [1");
        // A valid number cut short is a minus sign on its own.
        check(b"-0", 1, b"E474: Missing number after minus sign: -");
    }
}

/// `test/unit/eval/tricks_spec.lua`, `describe('NULL typval_T')`.
///
/// These three expressions are how every other spec in the tree gets hold
/// of a NULL string, list and dict — the values a `typval_T` can hold that
/// Vimscript has no literal for.
#[test]
fn three_expressions_produce_the_null_containers() {
    let _log = AllocLog::start();
    // SAFETY: each answer is this case's own and is cleared.
    unsafe {
        // Any name that is definitely not in the environment; the spec
        // extended it until it found one rather than trusting a fixed name.
        let mut name = String::from("XXX_UNEXISTENT_VAR_XXX");
        while std::env::var_os(&name).is_some() {
            name.push_str("_XXX");
        }

        for (expr, expected) in [
            (format!("${name}"), Tv::NullStr),
            ("v:_null_list".to_string(), Tv::NullList),
            ("v:_null_dict".to_string(), Tv::NullDict),
        ] {
            let mut tv = tv::eval0(&expr).unwrap_or_else(|| panic!("{expr} evaluates"));
            assert_eq!(tv::read(&raw const tv), expected, "{expr}");
            tv_clear(&raw mut tv);
        }
    }
}
