//! `cjson/fpconv.rs` against libc, which is the only real specification it
//! has.
//!
//! The module reimplements C's `%.*g` and `strtod` in safe Rust rather than
//! calling them, because `format!("{}")` and `str::parse` are neither. That
//! is only defensible if the two agree byte for byte and bit for bit, so
//! this walks a randomised sample of the whole f64 space plus the shapes
//! that break naive implementations — subnormals, exact ties, the
//! fixed/scientific switch, and hex floats, which the JSON decoder reaches
//! because `decode_invalid_numbers` is on by default.
//!
//! These live here rather than beside the module because they need libc and
//! the module is `forbid(unsafe_code)`. They are Miri-ignored for the same
//! reason: `snprintf` and `strtod` are foreign functions.

use core::ffi::{c_char, c_double, c_int};

use c2rust_neovim::cjson::fpconv::{append_g_fmt, strtod};

unsafe extern "C" {
    fn snprintf(s: *mut c_char, n: usize, format: *const c_char, ...) -> c_int;
    #[link_name = "strtod"]
    fn libc_strtod(nptr: *const c_char, endptr: *mut *mut c_char) -> c_double;
}

/// `printf("%.*g", precision, num)`.
fn libc_g_fmt(num: f64, precision: u32) -> String {
    let mut buf = [0u8; 64];
    // SAFETY: `%.*g` of an f64 is at most 24 bytes, well inside the buffer,
    // and the two variadic arguments match the format's `int` and `double`.
    let len = unsafe {
        snprintf(
            buf.as_mut_ptr().cast(),
            buf.len(),
            c"%.*g".as_ptr(),
            precision as c_int,
            num,
        )
    };
    String::from_utf8(buf[..len as usize].to_vec()).unwrap()
}

fn our_g_fmt(num: f64, precision: u32) -> String {
    let mut out = Vec::new();
    append_g_fmt(&mut out, num, precision);
    String::from_utf8(out).unwrap()
}

/// `strtod`, answering the value and the bytes it consumed.
fn libc_parse(text: &[u8]) -> (f64, usize) {
    let nul: Vec<u8> = text.iter().copied().chain([0]).collect();
    let mut end: *mut c_char = core::ptr::null_mut();
    // SAFETY: `nul` is NUL-terminated and outlives the call; `end` lands
    // inside it, so the offset is in bounds.
    unsafe {
        let value = libc_strtod(nul.as_ptr().cast(), &raw mut end);
        (value, end.cast::<u8>().offset_from(nul.as_ptr()) as usize)
    }
}

/// A cheap deterministic generator; a seeded PRNG rather than `rand` so a
/// failure is reproducible from the seed alone.
struct Bits(u64);

impl Bits {
    fn next(&mut self) -> u64 {
        // splitmix64.
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
}

/// The values that break naive `%g` implementations: both zeroes, the
/// decade boundaries the fixed/scientific switch sits on, the subnormal
/// range, and numbers whose 17th digit decides the 16th.
fn awkward_doubles() -> Vec<f64> {
    let mut values = vec![
        0.0,
        -0.0,
        1.0,
        -1.0,
        0.1,
        0.5,
        1.0 / 3.0,
        f64::MIN_POSITIVE,
        5e-324,
        1e-323,
        f64::MAX,
        -f64::MAX,
        9007199254740992.0,
        9007199254740993.0,
        123_456_789.123_456_79,
        0.30000000000000004,
        1.005,
        4.35,
        999999.9999999999,
        0.000_1,
    ];
    for exponent in -320i32..=308 {
        values.push(format!("1e{exponent}").parse().unwrap());
        values.push(format!("9.999999999999999e{exponent}").parse().unwrap());
        values.push(format!("-1.234567890123456e{exponent}").parse().unwrap());
    }
    values
}

#[test]
#[cfg_attr(miri, ignore = "snprintf is a foreign function")]
fn g_fmt_matches_printf_over_the_awkward_values() {
    for num in awkward_doubles() {
        for precision in 1..=17 {
            assert_eq!(
                our_g_fmt(num, precision),
                libc_g_fmt(num, precision),
                "%.{precision}g of {num:e}"
            );
        }
    }
}

#[test]
#[cfg_attr(miri, ignore = "snprintf is a foreign function")]
fn g_fmt_matches_printf_over_random_bit_patterns() {
    let mut bits = Bits(0x5eed_1234_5678_9abc);
    for _ in 0..200_000 {
        let num = f64::from_bits(bits.next());
        if !num.is_finite() {
            continue;
        }
        // 16 is the only precision the encoder ever passes; the others are
        // checked above, where the sample is small enough to afford them.
        assert_eq!(
            our_g_fmt(num, 16),
            libc_g_fmt(num, 16),
            "%.16g of {:#018x}",
            num.to_bits()
        );
    }
}

#[test]
#[cfg_attr(miri, ignore = "strtod is a foreign function")]
fn strtod_matches_libc_on_round_trips() {
    let mut bits = Bits(0xc0ff_ee00_1234_5678);
    for _ in 0..50_000 {
        let num = f64::from_bits(bits.next());
        if !num.is_finite() {
            continue;
        }
        let text = our_g_fmt(num, 17);
        let (ours, used) = strtod(text.as_bytes());
        assert_eq!((ours.to_bits(), used), {
            let (theirs, used) = libc_parse(text.as_bytes());
            (theirs.to_bits(), used)
        });
        assert_eq!(ours.to_bits(), num.to_bits(), "{text} did not round trip");
    }
}

#[test]
#[cfg_attr(miri, ignore = "strtod is a foreign function")]
fn strtod_matches_libc_on_the_grammar() {
    // Every shape the C grammar admits, valid and not, including the ones
    // only the JSON decoder's `decode_invalid_numbers` path produces.
    let cases: &[&str] = &[
        "",
        " ",
        "-",
        "+",
        ".",
        "x",
        "e5",
        ".e5",
        "1",
        "-1",
        "+1",
        "1.",
        ".5",
        "-.5",
        "1.5",
        "0",
        "-0",
        "00",
        "01",
        "1e",
        "1e+",
        "1e5",
        "1E5",
        "1e+5",
        "1e-5",
        "1.0e",
        "1.2.3",
        "9x",
        "  12",
        "\t\n 3.5",
        "1e308",
        "1e309",
        "1e-308",
        "1e-324",
        "1e-400",
        "1e400",
        "-1e400",
        "9007199254740993",
        "18446744073709551616",
        "2.2250738585072014e-308",
        "4.9406564584124654e-324",
        "0.000000000000000000000000001",
        "1000000000000000000000000000",
        "0x",
        "0x1",
        "0X1",
        "0x10",
        "0xff",
        "0x1p4",
        "0x1P4",
        "0x1p+4",
        "0x1p-4",
        "0x1.8p1",
        "0x.8",
        "0x.8p1",
        "0x1.",
        "-0xffp-4",
        "0x1p99999",
        "0x1p-99999",
        "0x20000000000001",
        "0x20000000000003",
        "0x1p-1074",
        "0x1p-1075",
        "0x1.8p-1074",
        "0x1fffffffffffff80p-60",
        "0xdeadbeefcafebabe1234567890abcdefp-40",
        "inf",
        "INF",
        "-inf",
        "infinity",
        "Infinity",
        "-Infinity",
        "infin",
        "nan",
        "NaN",
        "-nan",
        "nan(",
        "nan()",
        "nan(123)",
        "nan(1 2)",
    ];
    for case in cases {
        let (ours, our_used) = strtod(case.as_bytes());
        let (theirs, their_used) = libc_parse(case.as_bytes());
        assert_eq!(our_used, their_used, "consumed length of {case:?}");
        assert_eq!(
            ours.is_nan(),
            theirs.is_nan(),
            "NaN-ness of {case:?}: {ours} vs {theirs}"
        );
        if !ours.is_nan() {
            assert_eq!(ours.to_bits(), theirs.to_bits(), "value of {case:?}");
        }
    }
}

#[test]
#[cfg_attr(miri, ignore = "strtod is a foreign function")]
fn strtod_matches_libc_on_random_hex_floats() {
    let mut bits = Bits(0x1234_5678_9abc_def0);
    for _ in 0..20_000 {
        let mantissa = bits.next();
        let exponent = (bits.next() % 2400) as i64 - 1200;
        let digits = (bits.next() % 16 + 1) as usize;
        let text = format!("0x{mantissa:x}.{mantissa:016x}p{exponent}");
        let text = &text[..text.len().min(4 + digits + 20)];
        let (ours, our_used) = strtod(text.as_bytes());
        let (theirs, their_used) = libc_parse(text.as_bytes());
        assert_eq!(our_used, their_used, "consumed length of {text:?}");
        assert_eq!(ours.to_bits(), theirs.to_bits(), "value of {text:?}");
    }
}
