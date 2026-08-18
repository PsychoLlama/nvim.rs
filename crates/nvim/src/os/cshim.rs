//! The C library surface the `libc` crate does not carry.
//!
//! Everything else comes straight from that crate: modules `use libc::{..}`
//! and link against the platform C library. What is left here is the genuine
//! remainder -- GNU gettext, glibc's `__ctype_b_loc` table, the `getc`/`putc`
//! and `vsnprintf` prototypes the crate omits, `<wctype.h>`'s
//! `towupper`/`towlower`, `strtoimax`, `tzset`, and the four `static mut`
//! globals -- plus the Rust stand-ins the `just miri` lane needs for the
//! handful of string functions it reaches, which Miri cannot call through.

#![deny(unsafe_op_in_unsafe_fn)]

// No forbid(unsafe_code): edition 2024 trips the unsafe_code lint on the
// extern block below, and declaring the foreign surface is this file's
// entire job.

use ::libc::{FILE, intmax_t, size_t};

/// `wchar_t` widened for `towupper`/`towlower`. The `libc` crate has no
/// `<wchar.h>` types, so this boundary owns its own argument type.
pub type wint_t = ::core::ffi::c_uint;

// Miri interprets MIR and cannot call the platform C library, so these six
// take the Rust definitions further down instead of the crate's declarations.
#[cfg(not(miri))]
pub use ::libc::{memmove, snprintf, strchr, strncasecmp, strncmp, strstr};

unsafe extern "C" {
    /// glibc's character-class table, which `isalpha` and friends index.
    pub fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;

    // GNU gettext, none of which the `libc` crate declares. `gettext` alone
    // has 211 importers.
    pub fn bind_textdomain_codeset(
        __domainname: *const ::core::ffi::c_char,
        __codeset: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn bindtextdomain(
        __domainname: *const ::core::ffi::c_char,
        __dirname: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    #[cfg(not(miri))]
    pub fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    pub fn ngettext(
        __msgid1: *const ::core::ffi::c_char,
        __msgid2: *const ::core::ffi::c_char,
        __n: ::core::ffi::c_ulong,
    ) -> *mut ::core::ffi::c_char;
    pub fn textdomain(__domainname: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;

    // `getc`/`putc` are macros in the C standard, so the crate skips them;
    // glibc exports the functions, and this is the spelling upstream used.
    pub fn getc(__stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn putc(__c: ::core::ffi::c_int, __stream: *mut FILE) -> ::core::ffi::c_int;

    // `<wctype.h>`: the locale-sensitive wide-character case folds, absent
    // from the `libc` crate.
    #[cfg(not(miri))]
    pub fn towlower(__wc: wint_t) -> wint_t;
    #[cfg(not(miri))]
    pub fn towupper(__wc: wint_t) -> wint_t;

    /// Takes a `va_list`, which the `libc` crate has no spelling for.
    pub fn vsnprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        __arg: ::core::ffi::VaList,
    ) -> ::core::ffi::c_int;

    /// `<inttypes.h>`, which the `libc` crate does not cover.
    pub fn strtoimax(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> intmax_t;

    /// `<time.h>`, absent from the `libc` crate.
    pub fn tzset();

    /// GNU gettext's "the catalogue selection changed" counter. Bumping it
    /// is how `:language` stops cached translations from being reused.
    pub static mut _nl_msg_cat_cntr: ::core::ffi::c_int;

    // The environment block and the two stdio streams. The `libc` crate
    // declares none of the three.
    pub static mut environ: *mut *mut ::core::ffi::c_char;
    pub static mut stderr: *mut FILE;
    pub static mut stdout: *mut FILE;
}

// Miri interprets MIR and cannot call the platform C library, so the pure
// string functions the `just miri` test lane reaches get Rust definitions
// with C-locale semantics here. Same import path, link-time symbol otherwise.
/// The exact format strings `vim_vsnprintf` constructs for the conversions it
/// delegates to libc — nothing more. `%g` never reaches here (vim rewrites it
/// to `%e`/`%f` first), and neither do inf/nan (vim formats those itself).
/// Anything unrecognized is a loud panic rather than a silent wrong answer.
#[cfg(miri)]
pub unsafe extern "C" fn snprintf(
    __s: *mut ::core::ffi::c_char,
    __maxlen: size_t,
    __format: *const ::core::ffi::c_char,
    mut __args: ...
) -> ::core::ffi::c_int {
    fn exp_notation(v: f64, prec: usize, upper: bool) -> String {
        let s = format!("{v:.prec$e}");
        let (mantissa, exp) = s.split_once('e').expect("exponent in {:e} output");
        let exp: i32 = exp.parse().expect("numeric exponent");
        let e = if upper { 'E' } else { 'e' };
        let sign = if exp < 0 { '-' } else { '+' };
        format!("{mantissa}{e}{sign}{:02}", exp.abs())
    }

    let mut ap: ::core::ffi::VaList;
    ap = __args.clone();
    let fmt = unsafe { ::core::ffi::CStr::from_ptr(__format) }
        .to_str()
        .expect("snprintf shim: non-UTF-8 format");
    let out = match fmt {
        "%p" => {
            let p = unsafe { ap.next_arg::<*mut ::core::ffi::c_void>() };
            if p.is_null() {
                "(nil)".to_string()
            } else {
                format!("{:#x}", p.addr())
            }
        }
        "%ld" => unsafe { ap.next_arg::<::core::ffi::c_long>() }.to_string(),
        "%lu" => unsafe { ap.next_arg::<::core::ffi::c_ulong>() }.to_string(),
        "%lo" => format!("{:o}", unsafe { ap.next_arg::<::core::ffi::c_ulong>() }),
        "%lx" => format!("{:x}", unsafe { ap.next_arg::<::core::ffi::c_ulong>() }),
        "%lX" => format!("{:X}", unsafe { ap.next_arg::<::core::ffi::c_ulong>() }),
        ".%d" => format!(".{}", unsafe { ap.next_arg::<::core::ffi::c_int>() }),
        _ => {
            // The float formats: %[+ ]?(\.\d+)?[fFeE], default precision 6.
            let rest = fmt
                .strip_prefix('%')
                .unwrap_or_else(|| panic!("snprintf shim: unsupported format {fmt:?}"));
            let (sign_flag, rest) = match rest.as_bytes().first() {
                Some(b'+') => (Some('+'), &rest[1..]),
                Some(b' ') => (Some(' '), &rest[1..]),
                _ => (None, rest),
            };
            let (prec, spec) = match rest.strip_prefix('.') {
                Some(r) => {
                    let (digits, spec) = r.split_at(r.len() - 1);
                    (
                        digits
                            .parse()
                            .unwrap_or_else(|_| panic!("snprintf shim: bad format {fmt:?}")),
                        spec,
                    )
                }
                None => (6, rest),
            };
            let v = unsafe { ap.next_arg::<::core::ffi::c_double>() };
            let mut s = match spec {
                "f" | "F" => format!("{v:.prec$}"),
                "e" => exp_notation(v, prec, false),
                "E" => exp_notation(v, prec, true),
                _ => panic!("snprintf shim: unsupported format {fmt:?}"),
            };
            if let Some(sign) = sign_flag {
                if !s.starts_with('-') {
                    s.insert(0, sign);
                }
            }
            s
        }
    };
    let bytes = out.as_bytes();
    if __maxlen > 0 {
        let n = bytes.len().min(__maxlen - 1);
        unsafe {
            ::core::ptr::copy_nonoverlapping(bytes.as_ptr(), __s as *mut u8, n);
            *__s.add(n) = 0;
        }
    }
    bytes.len() as ::core::ffi::c_int
}

/// Untranslated, which is what libc's `gettext` answers for a message with
/// no catalogue entry — and the test lane runs with no catalogue.
///
/// `unsafe` to match the `#[cfg(not(miri))]` declaration above: every caller
/// hands it a raw pointer inside its own `unsafe` block, and a *safe* shim
/// makes each of those an `unused_unsafe` error under `-D warnings`.
#[cfg(miri)]
pub fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    __msgid as *mut ::core::ffi::c_char
}

#[cfg(miri)]
pub unsafe fn memmove(
    __dest: *mut ::core::ffi::c_void,
    __src: *const ::core::ffi::c_void,
    __n: size_t,
) -> *mut ::core::ffi::c_void {
    unsafe { ::core::ptr::copy(__src as *const u8, __dest as *mut u8, __n) };
    __dest
}

#[cfg(miri)]
pub unsafe fn strchr(
    __s: *const ::core::ffi::c_char,
    __c: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let c = __c as u8 as ::core::ffi::c_char;
    let mut p = __s;
    loop {
        let b = unsafe { *p };
        if b == c {
            return p as *mut ::core::ffi::c_char;
        }
        if b == 0 {
            return ::core::ptr::null_mut();
        }
        p = unsafe { p.add(1) };
    }
}

#[cfg(miri)]
pub unsafe fn strstr(
    __haystack: *const ::core::ffi::c_char,
    __needle: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let hay = unsafe { ::core::ffi::CStr::from_ptr(__haystack) }.to_bytes();
    let needle = unsafe { ::core::ffi::CStr::from_ptr(__needle) }.to_bytes();
    if needle.is_empty() {
        return __haystack as *mut ::core::ffi::c_char;
    }
    match hay.windows(needle.len()).position(|w| w == needle) {
        Some(i) => unsafe { __haystack.add(i) as *mut ::core::ffi::c_char },
        None => ::core::ptr::null_mut(),
    }
}

/// Byte-for-byte, stopping at the first NUL — C's `strncmp`, which compares
/// as `unsigned char` however `c_char` is signed on the target.
#[cfg(miri)]
pub unsafe extern "C" fn strncmp(
    __s1: *const ::core::ffi::c_char,
    __s2: *const ::core::ffi::c_char,
    __n: size_t,
) -> ::core::ffi::c_int {
    for i in 0..__n {
        let a = unsafe { *__s1.add(i) } as u8;
        let b = unsafe { *__s2.add(i) } as u8;
        if a != b || a == 0 {
            return a as ::core::ffi::c_int - b as ::core::ffi::c_int;
        }
    }
    0
}

#[cfg(miri)]
pub unsafe extern "C" fn strncasecmp(
    __s1: *const ::core::ffi::c_char,
    __s2: *const ::core::ffi::c_char,
    __n: size_t,
) -> ::core::ffi::c_int {
    for i in 0..__n {
        let a = (unsafe { *__s1.add(i) } as u8).to_ascii_lowercase();
        let b = (unsafe { *__s2.add(i) } as u8).to_ascii_lowercase();
        if a != b || a == 0 {
            return a as ::core::ffi::c_int - b as ::core::ffi::c_int;
        }
    }
    0
}

// Miri cannot call libc. The tests never call setlocale, so glibc would run
// these in the C locale, where they fold ASCII only -- which is exactly what
// these definitions do. Declared unsafe, exactly as the real ones are, so a
// caller's block does not become `unused_unsafe` under `cargo miri`.
/// # Safety
///
/// None; the definition only mirrors the foreign one it stands in for.
#[cfg(miri)]
pub unsafe fn towlower(__wc: wint_t) -> wint_t {
    u8::try_from(__wc).map_or(__wc, |b| b.to_ascii_lowercase() as wint_t)
}
/// # Safety
///
/// None; the definition only mirrors the foreign one it stands in for.
#[cfg(miri)]
pub unsafe fn towupper(__wc: wint_t) -> wint_t {
    u8::try_from(__wc).map_or(__wc, |b| b.to_ascii_uppercase() as wint_t)
}
