//! `vim_snprintf` and Vimscript's `printf()`, folded into one number each
//! over every conversion, flag, width, precision and length modifier the
//! formatter reads, at every buffer size that truncates the answer.
//!
//! The digests below were recorded from the formatter as upstream ported
//! it -- a `va_list` core with the floats and integers handed to libc's
//! `snprintf` -- and the rewrite onto a typed argument array has to
//! reproduce them byte for byte. A digest that changes is a behaviour
//! change, to be explained or fixed, never re-recorded to make the test
//! pass. With `PRINTF_ORACLE_DUMP=<file>` set, every case is also written
//! out one per line, so two builds' answers can be diffed.

#![cfg(not(miri))]

use std::ffi::{CString, c_char, c_int, c_long, c_uint, c_ulong};
use std::io::Write;

use neovim::{vim_snprintf, vim_snprintf_add, vim_snprintf_safelen};

use crate::support::corpus::{Digest, corpus};
use crate::support::{Probe, editor_lock};

/// Where the cases are dumped, if anywhere.
fn dump() -> Option<std::fs::File> {
    let path = std::env::var_os("PRINTF_ORACLE_DUMP")?;
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .ok()
}

/// One formatting call, run into a roomy buffer and then into every size
/// that cuts the answer short; the return value and the bytes written at
/// each size are folded in, and the bytes past the buffer are checked
/// untouched.
struct Runner {
    digest: Digest,
    out: Option<std::fs::File>,
    /// Sizes 0..=len+1 for every case, or only a sample.
    every_size: bool,
}

const GUARD: u8 = 0xa5;

impl Runner {
    fn new(every_size: bool) -> Runner {
        Runner {
            digest: Digest::new(),
            out: dump(),
            every_size,
        }
    }

    /// Run `call` into a `size`-byte buffer followed by guard bytes.
    fn at(&mut self, size: usize, call: &dyn Fn(*mut c_char, usize) -> c_int) -> (c_int, Vec<u8>) {
        let mut buf = vec![GUARD; size + 16];
        let ret = call(buf.as_mut_ptr().cast(), size);
        assert!(
            buf[size..].iter().all(|&b| b == GUARD),
            "wrote past a {size}-byte buffer"
        );
        // A format the positional pass rejects leaves the buffer as it was,
        // unterminated: then the whole of it is the answer.
        let written = match buf[..size].iter().position(|&b| b == 0) {
            Some(end) => buf[..end].to_vec(),
            None => buf[..size].to_vec(),
        };
        (ret, written)
    }

    fn case(&mut self, label: impl FnOnce() -> String, call: &dyn Fn(*mut c_char, usize) -> c_int) {
        let (ret, full) = self.at(1024, call);
        self.digest.add(i64::from(ret));
        self.digest.bytes(&full);
        if let Some(out) = &mut self.out {
            let _ = writeln!(out, "{}\t{ret}\t{}", label(), full.escape_ascii());
        }
        let len = full.len();
        let sizes: Vec<usize> = if self.every_size {
            (0..=len + 1).collect()
        } else {
            let mut s = vec![0, 1, 2, len / 2, len, len + 1];
            s.dedup();
            s
        };
        for size in sizes {
            let (r, got) = self.at(size, call);
            self.digest.add(i64::from(r));
            self.digest.bytes(&got);
        }
    }
}

fn c(s: &str) -> CString {
    CString::new(s).unwrap()
}

const FLAGS: &[&str] = &[
    "", "-", "+", " ", "#", "0", "-0", "+ ", " +", "#0", "'", "-#",
];
const WIDTHS: &[&str] = &["", "1", "5", "12"];
const PRECISIONS: &[&str] = &["", ".", ".0", ".1", ".3", ".12"];

const SIGNED: &[i64] = &[
    0,
    1,
    -1,
    7,
    -42,
    255,
    70000,
    -70000,
    123_456_789,
    i32::MAX as i64,
    i32::MIN as i64,
    i32::MAX as i64 + 1,
    i64::MAX,
    i64::MIN,
];

#[test]
fn integer_conversions() {
    let _editor = editor_lock();
    let mut r = Runner::new(false);
    for spec in ["d", "i", "u", "o", "x", "X", "b", "B", "D", "U", "O"] {
        for len in ["", "h", "l", "ll", "z"] {
            if spec.chars().all(|ch| ch.is_ascii_uppercase())
                && matches!(spec, "D" | "U" | "O")
                && !len.is_empty()
            {
                continue;
            }
            for flags in FLAGS {
                for width in WIDTHS {
                    for prec in PRECISIONS {
                        let f = format!("<%{flags}{width}{prec}{len}{spec}>");
                        let fc = c(&f);
                        let fp = fc.as_ptr();
                        for &v in SIGNED {
                            let label = || format!("{f} {v}");
                            let signed = matches!(spec, "d" | "i" | "D");
                            let wide = matches!(len, "l" | "ll")
                                || matches!(spec, "D" | "U" | "O" | "b" | "B");
                            // Each value at the type its length modifier
                            // names, as a C caller passes it.
                            match (signed, wide, len) {
                                (_, _, "z") if signed => r.case(label, &|b, n| unsafe {
                                    vim_snprintf!(b, n, fp, v as isize)
                                }),
                                (_, _, "z") => r.case(label, &|b, n| unsafe {
                                    vim_snprintf!(b, n, fp, v as usize)
                                }),
                                (true, true, _) => r.case(label, &|b, n| unsafe {
                                    vim_snprintf!(b, n, fp, v as c_long)
                                }),
                                (false, true, _) => r.case(label, &|b, n| unsafe {
                                    vim_snprintf!(b, n, fp, v as c_ulong)
                                }),
                                (true, false, _) => r.case(label, &|b, n| unsafe {
                                    vim_snprintf!(b, n, fp, v as c_int)
                                }),
                                (false, false, _) => r.case(label, &|b, n| unsafe {
                                    vim_snprintf!(b, n, fp, v as c_uint)
                                }),
                            }
                        }
                    }
                }
            }
        }
    }
    // `*` widths and precisions, negative ones included.
    for spec in ["d", "x", "b", "o"] {
        for flags in ["", "-", "0", "#"] {
            for f in [
                format!("<%{flags}*{spec}>"),
                format!("<%{flags}.*{spec}>"),
                format!("<%{flags}*.*{spec}>"),
            ] {
                let fc = c(&f);
                let fp = fc.as_ptr();
                let stars = f.matches('*').count();
                for (w, p) in [(9, 4), (-9, 4), (0, 0), (3, -1), (-1, 12), (2_000_000, 1)] {
                    for &v in &[0i64, 5, -5, 300] {
                        let label = || format!("{f} {w} {p} {v}");
                        let big = matches!(spec, "b");
                        match (stars, big) {
                            (1, false) => r.case(label, &|b, n| unsafe {
                                vim_snprintf!(b, n, fp, w as c_int, v as c_int)
                            }),
                            (1, true) => r.case(label, &|b, n| unsafe {
                                vim_snprintf!(b, n, fp, w as c_int, v as c_ulong)
                            }),
                            (_, false) => r.case(label, &|b, n| unsafe {
                                vim_snprintf!(b, n, fp, w as c_int, p as c_int, v as c_int)
                            }),
                            (_, true) => r.case(label, &|b, n| unsafe {
                                vim_snprintf!(b, n, fp, w as c_int, p as c_int, v as c_ulong)
                            }),
                        }
                    }
                }
            }
        }
    }
    assert_eq!(r.digest.value(), 0x25b88b2abaa2d442, "integer conversions");
}

/// Doubles at every awkward place: zeros, the `%g` switch points, halfway
/// cases, the edges of the scratch buffer, subnormals, the non-finite ones,
/// and a deterministic spray across the exponent range.
fn doubles() -> Vec<f64> {
    let mut out = vec![
        0.0,
        -0.0,
        1.0,
        -1.0,
        0.5,
        1.5,
        2.5,
        -2.5,
        0.125,
        0.1,
        0.2,
        0.1 + 0.2,
        1.0 / 3.0,
        2.0 / 3.0,
        123.456,
        -123.456,
        1e-5,
        1e-4,
        0.001,
        0.000_999_999,
        0.000_999_999_9,
        9_999_999.0,
        9_999_999.5,
        10_000_000.0,
        10_000_000.1,
        1e15 + 0.3,
        1e16,
        1e21,
        1e22,
        1e100,
        -1e100,
        1e300,
        1e307,
        1.000_000_1e307,
        1e308,
        f64::MAX,
        -f64::MAX,
        f64::MIN_POSITIVE,
        f64::MIN_POSITIVE / 3.0,
        5e-324,
        -5e-324,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
        -f64::NAN,
        0.000_05,
        0.000_15,
        0.000_25,
        1.000_5,
        2.675,
        1_234_567.891_011,
        4.35,
        0.045,
        999.999_5,
        99_999.95,
    ];
    // A spray of bit patterns: every exponent decade, odd mantissas.
    let mut x: u64 = 0x9e37_79b9_7f4a_7c15;
    for _ in 0..120 {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        let v = f64::from_bits(x);
        if v.is_finite() {
            out.push(v);
        }
        // Scaled into the range `%f` actually prints.
        let mantissa = (x >> 11) as f64 / (1u64 << 53) as f64;
        let exp = ((x >> 3) % 60) as i32 - 30;
        out.push(mantissa * 10f64.powi(exp));
    }
    out
}

#[test]
fn float_conversions() {
    let _editor = editor_lock();
    let mut r = Runner::new(false);
    let values = doubles();
    for spec in ["f", "F", "e", "E", "g", "G"] {
        for flags in ["", "-", "+", " ", "0", "#", "+0", "- "] {
            for width in ["", "1", "12", "30"] {
                for prec in [
                    "", ".0", ".1", ".3", ".6", ".10", ".17", ".25", ".60", ".400",
                ] {
                    let f = format!("<%{flags}{width}{prec}{spec}>");
                    let fc = c(&f);
                    let fp = fc.as_ptr();
                    for &v in &values {
                        let label = || format!("{f} {v:e}");
                        r.case(label, &|b, n| unsafe { vim_snprintf!(b, n, fp, v) });
                    }
                }
            }
        }
    }
    // `*` precisions on floats, and positional ones.
    for f in ["%.*f", "%.*e", "%.*g", "%*.*G", "%1$.*2$f", "%2$*1$.3e"] {
        let fc = c(f);
        let fp = fc.as_ptr();
        for &v in &values[..60] {
            let label = || format!("{f} {v:e}");
            if f.starts_with("%2$") || f.starts_with("%*") {
                r.case(label, &|b, n| unsafe {
                    vim_snprintf!(b, n, fp, 14 as c_int, v)
                });
            } else if f.starts_with("%1$") {
                r.case(label, &|b, n| unsafe {
                    vim_snprintf!(b, n, fp, v, 7 as c_int)
                });
            } else {
                r.case(label, &|b, n| unsafe {
                    vim_snprintf!(b, n, fp, 5 as c_int, v)
                });
            }
        }
    }
    assert_eq!(r.digest.value(), 0x14596030ccb7140b, "float conversions");
}

#[test]
fn string_char_and_pointer_conversions() {
    let _editor = editor_lock();
    let mut r = Runner::new(true);
    let mut texts: Vec<Vec<u8>> = vec![
        b"".to_vec(),
        b"a".to_vec(),
        b"hello".to_vec(),
        "漢語".as_bytes().to_vec(),
        "e\u{301}x".as_bytes().to_vec(),
        "😀👍🏽".as_bytes().to_vec(),
        b"\xe9\xe8\xfc".to_vec(),
        b"tab\there".to_vec(),
    ];
    texts.extend(corpus().into_iter().take(40));
    let texts: Vec<CString> = texts
        .into_iter()
        .map(|t| CString::new(t).unwrap())
        .collect();
    for spec in ["s", "S"] {
        for flags in ["", "-", "0", "+", "#"] {
            for width in ["", "1", "3", "8", "20"] {
                for prec in ["", ".", ".0", ".1", ".2", ".4", ".7"] {
                    let f = format!("[%{flags}{width}{prec}{spec}]");
                    let fc = c(&f);
                    let fp = fc.as_ptr();
                    for t in &texts {
                        let tp = t.as_ptr();
                        let label = || format!("{f} {:?}", t);
                        r.case(label, &|b, n| unsafe { vim_snprintf!(b, n, fp, tp) });
                    }
                    let label = || format!("{f} NULL");
                    r.case(label, &|b, n| unsafe {
                        vim_snprintf!(b, n, fp, std::ptr::null::<c_char>())
                    });
                }
            }
        }
        // `*` width and precision, negative ones included.
        for f in [
            "[%*s]", "[%-*s]", "[%.*s]", "[%*.*s]", "[%*S]", "[%-*S]", "[%.*S]", "[%*.*S]",
        ] {
            let f = f.replace("S]", &format!("{spec}]"));
            let fc = c(&f);
            let fp = fc.as_ptr();
            let stars = f.matches('*').count();
            for (w, p) in [(6, 2), (-6, 3), (0, 0), (2, -1), (11, 5)] {
                for t in &texts[..10] {
                    let tp = t.as_ptr();
                    let label = || format!("{f} {w} {p} {t:?}");
                    if stars == 1 {
                        r.case(label, &|b, n| unsafe {
                            vim_snprintf!(b, n, fp, w as c_int, tp)
                        });
                    } else {
                        r.case(label, &|b, n| unsafe {
                            vim_snprintf!(b, n, fp, w as c_int, p as c_int, tp)
                        });
                    }
                }
            }
        }
    }
    // `%.*s` over bytes that are not terminated within the precision's reach
    // past it: the precision bounds the read.
    let unterminated = *b"abcdefgh";
    for p in 0..=8 {
        let fp = c"<%.*s>".as_ptr();
        let at = unterminated.as_ptr().cast::<c_char>();
        r.case(|| format!("%.*s unterminated {p}"), &|b, n| unsafe {
            vim_snprintf!(b, n, fp, p as c_int, at)
        });
    }
    // `%c`: only the low byte of the codepoint goes out.
    for flags in ["", "-", "0"] {
        for width in ["", "3"] {
            let f = format!("<%{flags}{width}c>");
            let fc = c(&f);
            let fp = fc.as_ptr();
            for v in [0x41, 0xe9, 0x1f600, 0x100, 0, -1, 0x7f] {
                r.case(|| format!("{f} {v}"), &|b, n| unsafe {
                    vim_snprintf!(b, n, fp, v as c_int)
                });
            }
        }
    }
    // `%p`: the address, or `(nil)`.
    for f in [
        "<%p>", "<%20p>", "<%-20p>", "<%020p>", "<%.20p>", "<%#p>", "<%+p>",
    ] {
        let fc = c(f);
        let fp = fc.as_ptr();
        for addr in [0usize, 1, 0x8765_4321, usize::MAX] {
            let p: *const c_char = std::ptr::without_provenance(addr);
            r.case(|| format!("{f} {addr:x}"), &|b, n| unsafe {
                vim_snprintf!(b, n, fp, p)
            });
        }
    }
    // Text that is not a conversion, and `%%`.
    for f in [
        "",
        "plain",
        "%%",
        "a%%b",
        "%5%",
        "%-5%",
        "%05%",
        "%y",
        "%5y",
        "%-5y",
        "%",
        "x%",
        "%l",
        "%h",
        "%ll",
        "%z",
        "%hq",
        "%.3",
        "%#",
        "100%%",
        "%%%%%%",
        "%\u{e9}",
        "%\u{1f600}x",
    ] {
        let fc = c(f);
        let fp = fc.as_ptr();
        r.case(|| format!("literal {f:?}"), &|b, n| unsafe {
            vim_snprintf!(b, n, fp)
        });
    }
    assert_eq!(
        r.digest.value(),
        0x98a7caf7da385616,
        "string, char and pointer conversions"
    );
}

#[test]
fn positional_and_mixed_arguments() {
    let _editor = editor_lock();
    let mut r = Runner::new(true);
    let one = c"one";
    let two = c"two";
    let (o, t) = (one.as_ptr(), two.as_ptr());
    let cases: &[(&str, u8)] = &[
        ("%1$s %2$s", 0),
        ("%2$s %1$s %2$s", 0),
        ("%1$s%1$s%1$s", 0),
        ("%2$-6s|%1$6s", 0),
        ("%1$.2s %2$S", 0),
        ("%1$*3$s|%2$s", 1),
        ("%1$s %2$s %3$d", 1),
        ("%3$d %1$s %2$s", 1),
        ("%1$-*3$s|", 1),
        ("%1$.*3$s|%2$s", 1),
        ("%3$x %3$o %3$d", 1),
        ("%1$s %s", 0),
        ("%s %1$s", 0),
        ("%0$s", 0),
        ("%01$s", 0),
        ("%1$s %3$d", 1),
        ("%1$d", 2),
        ("%3$*1$s", 0),
        ("%1$*s", 0),
        ("%1$5$s", 0),
        ("%2147483648$s", 0),
        ("%1$s %1$d", 0),
        ("%2$s %2$*1$d", 0),
        ("%9999999999s", 0),
        ("%.9999999999s", 0),
        ("%1$Y", 0),
    ];
    for &(f, shape) in cases {
        let fc = c(f);
        let fp = fc.as_ptr();
        let label = || format!("pos {f}");
        match shape {
            0 => r.case(label, &|b, n| unsafe { vim_snprintf!(b, n, fp, o, t) }),
            1 => r.case(label, &|b, n| unsafe {
                vim_snprintf!(b, n, fp, o, t, 7 as c_int)
            }),
            _ => r.case(label, &|b, n| unsafe {
                vim_snprintf!(b, n, fp, 12 as c_int)
            }),
        }
    }
    // `vim_snprintf_add` appends; `vim_snprintf_safelen` answers at most
    // the room it had.
    for size in 0..24usize {
        let mut buf = vec![GUARD; size + 16];
        if size > 0 {
            buf[..size.min(4)].fill(b'x');
            buf[size.min(4).min(size - 1)] = 0;
        }
        let ret = unsafe {
            vim_snprintf_add!(
                buf.as_mut_ptr().cast(),
                size,
                c"+%s=%d".as_ptr(),
                o,
                42 as c_int
            )
        };
        r.digest.add(i64::from(ret));
        r.digest.bytes(&buf);
        let mut buf = vec![GUARD; size + 16];
        let ret = unsafe {
            vim_snprintf_safelen!(
                buf.as_mut_ptr().cast(),
                size,
                c"%s-%s-%5.1f".as_ptr(),
                o,
                t,
                2.25
            )
        };
        r.digest.add(ret as i64);
        r.digest.bytes(&buf);
    }
    assert_eq!(r.digest.value(), 0xf2131e8a16617898, "positional arguments");
}

/// Vimscript's `printf()`: its answer as `string()` renders it and the
/// error it raised, folded in.
#[test]
fn vimscript_printf() {
    let editor = editor_lock();
    let probe = Probe::new(&editor);
    let mut d = Digest::new();
    let mut out = dump();
    let mut exprs: Vec<String> = Vec::new();
    for spec in [
        "d", "i", "u", "o", "x", "X", "b", "B", "c", "s", "S", "f", "F", "e", "E", "g", "G",
    ] {
        for flags in ["", "-", "+", " ", "#", "0", "-0"] {
            for width in ["", "5", "*"] {
                for prec in ["", ".0", ".2", ".*"] {
                    for len in ["", "l", "ll", "h", "z"] {
                        let f = format!("%{flags}{width}{prec}{len}{spec}");
                        let stars: Vec<&str> = [width, prec]
                            .iter()
                            .filter(|s| s.contains('*'))
                            .map(|_| "-7")
                            .collect();
                        for arg in [
                            "0",
                            "42",
                            "-42",
                            "0x7fffffffffffffff",
                            "-0x8000000000000000",
                            "3.75",
                            "-0.0",
                            "1.0e21",
                            "'12ab'",
                            "'漢語'",
                            "'é'",
                            "[1, 'a']",
                            "{'k': 1}",
                            "v:true",
                            "v:null",
                            "0z0102",
                            "function('tr')",
                        ] {
                            let mut args = stars.clone();
                            args.push(arg);
                            exprs.push(format!("printf('{f}', {})", args.join(", ")));
                        }
                    }
                }
            }
        }
    }
    for e in [
        "printf('%d')",
        "printf('%d %d', 1)",
        "printf('%d', 1, 2)",
        "printf('%s')",
        "printf('%f', 'x')",
        "printf('%f', [])",
        "printf('%d', 1.5)",
        "printf('%d', [])",
        "printf('%c', 0x1f600)",
        "printf('%c%c', 0xe9, 65)",
        "printf('%1$s %2$s', 'a', 'b')",
        "printf('%2$s %1$s', 'a', 'b')",
        "printf('%1$s %s', 'a', 'b')",
        "printf('%s %1$s', 'a', 'b')",
        "printf('%1$s %3$s', 'a', 'b', 'c')",
        "printf('%3$s', 'a', 'b')",
        "printf('%1$s', 'a', 'b')",
        "printf('%1$d %1$s', 1)",
        "printf('%1$*2$d', 5, 9)",
        "printf('%2$*1$d', 9, 5)",
        "printf('%1$*2$d %2$s', 5, 9)",
        "printf('%1$*2$.*3$f', 1.5, 9, 3)",
        "printf('%0$d', 1)",
        "printf('%01$d', 1)",
        "printf('%1$5$d', 1)",
        "printf('%1$*d', 1, 2)",
        "printf('%*1$d', 1, 2)",
        "printf('%9999999999d', 1)",
        "printf('%.9999999999d', 1)",
        "printf('%*d', 9999999999, 1)",
        "printf('%.*d', 9999999999, 1)",
        "printf('%1048577d', 1)",
        "printf('%1048576d', 1)->len()",
        "printf('%-*S|', 6, '漢語')",
        "printf('%3S', \"\\xe9\\xe8\\xfc\")",
        "printf('%.3S|', '漢語x')",
        "printf('%5.1S|', '😀👍🏽')",
        "printf('%y')",
        "printf('%')",
        "printf('100%%')",
        "printf('%5%|')",
        "printf('%s', repeat('ab', 300))",
        "printf('%.*s', 3, 'abcdef')",
        "printf('%-10.3s|', 'abcdef')",
        "printf('%10.3s|', 'abcdef')",
        "printf('%#o %#x %#X %#b %#B', 8, 255, 255, 5, 5)",
        "printf('%#o', 0)",
        "printf('%.0d|%.0x|%#.0o', 0, 0, 0)",
        "printf('%+.3f %+.3e %+.3g', 1.0, 1.0, 1.0)",
        "printf('%g %g %g %g', 1.0e21, 1.0e-5, 100000.0, 1.0e7)",
        "printf('%e %E', 1.0e-300, 1.0e300)",
        "printf('%f', 1.0e308)",
        "printf('%f %e %g', 1.0/0, -1.0/0, 0.0/0)",
        "printf('%5f|%-5f|%+f|% f', 1.0/0, -1.0/0, 1.0/0, 1.0/0)",
        "printf('%F %E %G', 1.0/0, -1.0/0, 0.0/0)",
        "printf('%010.3f|%-010.3f|%+010.3f', -1.5, -1.5, 1.5)",
        "printf('%f', 5.0e-324)",
        "printf('%.400f', 1.0)->len()",
        "printf('%.400e', 1.0)->len()",
        "printf('%s', 1.5)",
        "printf('%s', -0.0)",
        "printf('%s %s', v:true, v:null)",
        "printf('%d', '0x10')",
        "printf('%d', ' 12')",
        "printf('%x', -1)",
        "printf('%u', -1)",
        "printf('%hd', 70000)",
        "printf('%hu', -1)",
        "printf('%lld %llu %zd %zu', -1, -1, -1, -1)",
        "printf(\"%s\\x00%s\", 'a', 'b')",
        "printf('%s', \"a\\nb\")",
    ] {
        exprs.push(e.to_string());
    }
    for e in &exprs {
        let got = probe.eval(e);
        let err = probe.error_from(&format!("call {e}"));
        d.bytes(got.as_bytes());
        d.bytes(err.as_bytes());
        if let Some(out) = &mut out {
            let _ = writeln!(out, "{e}\t{got}\t{err}");
        }
    }
    assert_eq!(d.value(), 0x93a18330a79ba5bd, "printf()");
}
