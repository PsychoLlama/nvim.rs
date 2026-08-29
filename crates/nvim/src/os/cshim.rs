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

    // GNU gettext, none of which the `libc` crate declares. The two lookups
    // are wrapped further down -- `gettext`/`ngettext` in this module are the
    // safe `&CStr` forms every caller uses -- so only the raw declarations
    // are here, under their link names.
    pub fn bind_textdomain_codeset(
        __domainname: *const ::core::ffi::c_char,
        __codeset: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn bindtextdomain(
        __domainname: *const ::core::ffi::c_char,
        __dirname: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    #[cfg(not(miri))]
    #[link_name = "gettext"]
    fn gettext_raw(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    #[cfg(not(miri))]
    #[link_name = "ngettext"]
    fn ngettext_raw(
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
/// # Safety
/// None; the definition only mirrors the foreign one it stands in for.
#[cfg(miri)]
unsafe fn gettext_raw(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    __msgid.cast_mut()
}

/// The plural sibling of the [`gettext_raw`] stand-in: English's rule, which
/// is what a catalogue-less lookup falls back to.
///
/// # Safety
/// None; the definition only mirrors the foreign one it stands in for.
#[cfg(miri)]
unsafe fn ngettext_raw(
    __msgid1: *const ::core::ffi::c_char,
    __msgid2: *const ::core::ffi::c_char,
    __n: ::core::ffi::c_ulong,
) -> *mut ::core::ffi::c_char {
    (if __n == 1 { __msgid1 } else { __msgid2 }).cast_mut()
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

// ---------------------------------------------------------------------------
// Message translation
//
// `gettext` and `ngettext` are wrapped rather than re-exported: their answers
// are `&'static CStr`, which is what the ~1,200 call sites and the whole
// `emsg`/`msg` family want, and getting there honestly is what the interning
// table below is for.

/// A translation, interned so that the answer really does live as long as the
/// process.
///
/// # The lifetime ruling
///
/// GNU `gettext` answers one of two things. With no catalogue entry it hands
/// back *the pointer it was given*, which is `'static` when the msgid is --
/// that is the fast path below, and today it is the only path this tree
/// takes, because it ships no catalogues. With an entry it hands back a
/// pointer into the loaded catalogue, and GNU documents that only as valid
/// until the next `bindtextdomain`, not for the process lifetime; glibc
/// happens never to unload a catalogue, but a `&'static` handed out on that
/// basis would rest on an implementation detail of one libc.
///
/// So a translated answer is *copied* the first time it is seen and the copy
/// is leaked. The reference is then `'static` by construction, and every
/// later lookup of the same text answers the same address. Nothing is ever
/// freed, so a reference taken before a `:language` stays valid after it --
/// which matters, because message text outlives its lookup in `v:errmsg`, in
/// the `:redir` sink and in the message history.
///
/// The table is keyed by the *translated bytes*, not by the msgid, which is
/// what makes it obviously correct: whatever comes back has the content that
/// was asked for and lives forever, whichever catalogue produced it and
/// whichever plural form `ngettext` picked.
///
/// `epoch` is GNU gettext's own `_nl_msg_cat_cntr`, the counter it bumps
/// whenever the catalogue selection can have changed -- `setlocale`,
/// `bindtextdomain`, and `:language`'s explicit bump in `os::lang`. It is not
/// needed for correctness; it bounds the table, which is cleared when the
/// epoch moves so that a session flipping locales does not hold every
/// catalogue's text at once. Entries handed out under an earlier epoch are
/// unaffected: they were leaked, not owned by the table.
static INTERNED: ::std::sync::Mutex<Interned> = ::std::sync::Mutex::new(Interned {
    epoch: 0,
    texts: ::std::collections::BTreeMap::new(),
});

/// [`INTERNED`]'s contents: the catalogue epoch the entries were looked up
/// under, and the entries.
struct Interned {
    epoch: ::core::ffi::c_int,
    texts: ::std::collections::BTreeMap<Box<[u8]>, &'static ::core::ffi::CStr>,
}

/// `text` as a string that outlives the process, interned per [`INTERNED`].
fn intern(epoch: ::core::ffi::c_int, text: &::core::ffi::CStr) -> &'static ::core::ffi::CStr {
    let mut interned = INTERNED
        .lock()
        .unwrap_or_else(::std::sync::PoisonError::into_inner);
    if interned.epoch != epoch {
        interned.epoch = epoch;
        interned.texts.clear();
    }
    if let Some(&held) = interned.texts.get(text.to_bytes()) {
        return held;
    }
    let held: &'static ::core::ffi::CStr = Box::leak(text.to_owned().into_boxed_c_str());
    interned.texts.insert(text.to_bytes().into(), held);
    held
}

/// GNU gettext's catalogue-selection counter.
fn catalogue_epoch() -> ::core::ffi::c_int {
    // SAFETY: a plain `int` glibc exports and only ever increments. Nvim is
    // single-threaded wherever messages are, as `GlobalCell` documents.
    unsafe { _nl_msg_cat_cntr }
}

/// The translation of `msgid` in the current message catalogue -- C's `_()`.
///
/// Answers `msgid` itself when the catalogue has no entry for it, which is
/// every message in a build that ships no catalogues. See [`INTERNED`] for
/// why the answer is `'static` when it does have one.
pub(crate) fn gettext(msgid: &'static ::core::ffi::CStr) -> &'static ::core::ffi::CStr {
    // SAFETY: `msgid` is NUL-terminated by its type, which is all `gettext`
    // reads, and its answer is a NUL-terminated string of the library's own.
    let answer = unsafe { gettext_raw(msgid.as_ptr()) };
    if ::core::ptr::eq(answer.cast_const(), msgid.as_ptr()) {
        return msgid;
    }
    // SAFETY: as above -- a live NUL-terminated string, copied by `intern`
    // before anything can invalidate it.
    intern(catalogue_epoch(), unsafe {
        ::core::ffi::CStr::from_ptr(answer)
    })
}

/// [`gettext`] for a msgid whose own storage is not `'static` -- one some
/// other code *formatted*, typically, like an option error carrying the
/// offending value.
///
/// Answers an owned copy rather than a `'static` borrow precisely because
/// those msgids are not a fixed set: interning them would grow [`INTERNED`]
/// by a string per distinct error, which is a leak with a user-facing tap on
/// it. Callers show the message and drop it.
pub(crate) fn gettext_owned(msgid: &::core::ffi::CStr) -> ::std::ffi::CString {
    // SAFETY: as [`gettext`].
    let answer = unsafe { gettext_raw(msgid.as_ptr()) };
    // SAFETY: as [`gettext`] -- a live NUL-terminated string, copied here.
    unsafe { ::core::ffi::CStr::from_ptr(answer) }.to_owned()
}

/// [`gettext`] for a message still held as a raw pointer.
///
/// Nothing is interned: the answer is either `msgid` itself or the
/// catalogue's own text, and it is handed back with an unbounded lifetime,
/// which is [`crate::cstr`]'s convention for exactly this shape. It is the
/// form the transpiled call sites want -- they were holding a `*mut c_char`
/// from `gettext` and are held to the same obligation they already had.
///
/// # Safety
/// `msgid` must point at a live NUL-terminated string, and the answer must
/// not be held past a `:language`: it is the catalogue's text, not a copy.
/// Use [`gettext`] where the answer outlives the lookup.
pub(crate) unsafe fn gettext_ptr<'a>(msgid: *const ::core::ffi::c_char) -> &'a ::core::ffi::CStr {
    // SAFETY: the caller's contract, and `gettext` answers a NUL-terminated
    // string of its own for one.
    unsafe { ::core::ffi::CStr::from_ptr(gettext_raw(msgid)) }
}

/// The translation of a message with a count -- C's `NGETTEXT()`. `one` is
/// the singular msgid, `many` the plural one, and `n` selects between them by
/// the catalogue's plural rule (English's, when there is no catalogue).
pub(crate) fn ngettext(
    one: &'static ::core::ffi::CStr,
    many: &'static ::core::ffi::CStr,
    n: ::core::ffi::c_ulong,
) -> &'static ::core::ffi::CStr {
    // SAFETY: both msgids are NUL-terminated by their type, and the answer is
    // one of them or a NUL-terminated string of the library's own.
    let answer = unsafe { ngettext_raw(one.as_ptr(), many.as_ptr(), n) };
    for msgid in [one, many] {
        if ::core::ptr::eq(answer.cast_const(), msgid.as_ptr()) {
            return msgid;
        }
    }
    // SAFETY: as above.
    intern(catalogue_epoch(), unsafe {
        ::core::ffi::CStr::from_ptr(answer)
    })
}

#[cfg(test)]
mod gettext_tests {
    use super::{_nl_msg_cat_cntr, catalogue_epoch, gettext, gettext_owned, intern, ngettext};
    use core::ffi::CStr;
    use std::ffi::CString;

    /// The same text interned twice is the same string, and it carries the
    /// bytes it was interned with.
    #[test]
    fn interning_is_idempotent() {
        let first = intern(7, c"E1: translated");
        let again = intern(7, c"E1: translated");
        assert!(core::ptr::eq(first, again));
        assert_eq!(first.to_bytes(), b"E1: translated");
    }

    /// The copy does not borrow the caller's buffer: an owned `CString`
    /// dropped right after interning leaves the answer intact.
    #[test]
    fn interning_copies() {
        let held = {
            let scratch = CString::new("E2: from a temporary").expect("no interior NUL");
            intern(11, &scratch)
        };
        assert_eq!(held.to_bytes(), b"E2: from a temporary");
    }

    /// A catalogue flip re-interns rather than answering the previous
    /// locale's text, and the reference handed out before the flip stays
    /// valid and unchanged -- which is the whole lifetime claim.
    #[test]
    fn a_catalogue_flip_re_interns() {
        let before: &'static CStr = intern(21, c"Datei nicht gefunden");
        let after: &'static CStr = intern(22, c"fichier introuvable");
        assert_eq!(before.to_bytes(), b"Datei nicht gefunden");
        assert_eq!(after.to_bytes(), b"fichier introuvable");
        assert!(!core::ptr::eq(before, after));

        // Back to the first text under a third epoch: the table was cleared,
        // so this is a fresh entry -- and the old reference still reads.
        let again = intern(23, c"Datei nicht gefunden");
        assert_eq!(again.to_bytes(), before.to_bytes());
        assert_eq!(before.to_bytes(), b"Datei nicht gefunden");
    }

    /// With no catalogue -- what this tree ships, and what the test lane runs
    /// with -- a lookup answers the msgid itself, allocating nothing, and the
    /// owned form answers the same bytes.
    #[test]
    fn an_untranslated_lookup_answers_the_msgid() {
        let msgid = c"E474: Invalid argument";
        assert!(core::ptr::eq(gettext(msgid), msgid));
        assert_eq!(gettext_owned(msgid).as_c_str(), msgid);
        assert_eq!(
            ngettext(c"%ld line", c"%ld lines", 1).to_bytes(),
            b"%ld line"
        );
        assert_eq!(
            ngettext(c"%ld line", c"%ld lines", 2).to_bytes(),
            b"%ld lines"
        );
    }

    /// Flipping the locale mid-session leaves messages looked up on either
    /// side of the flip valid and correct.
    ///
    /// This tree ships no `.mo` catalogues, so both sides are the untranslated
    /// msgid and what is really checked here is that the epoch moves and that
    /// neither answer is disturbed by it. The interning a catalogue would
    /// exercise is covered by `a_catalogue_flip_re_interns` above.
    #[test]
    fn flipping_the_locale_mid_session() {
        let before = gettext(c"E32: No file name");
        let epoch_before = catalogue_epoch();

        // SAFETY: `setlocale` with a NUL-terminated name; "C" always exists,
        // and it is the locale the test lane already runs in -- the call is
        // here for its catalogue-epoch side effect.
        let ok = unsafe { ::libc::setlocale(::libc::LC_ALL, c"C".as_ptr()) };
        assert!(!ok.is_null(), "the C locale is always available");
        // SAFETY: `:language`'s own bump, spelled the same way -- a single
        // increment of a counter nothing else in the process touches.
        unsafe { _nl_msg_cat_cntr += 1 };

        let after = gettext(c"E32: No file name");
        assert!(catalogue_epoch() > epoch_before, "the epoch moved");
        assert_eq!(before.to_bytes(), b"E32: No file name");
        assert_eq!(after.to_bytes(), b"E32: No file name");
    }
}
