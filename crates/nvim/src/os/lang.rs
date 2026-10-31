//! The locale layer: `setlocale`, `:language`, and locale-name completion.
//!
//! # Boundary
//!
//! Every answer here comes from the C library — `setlocale` for what is in
//! effect, `bindtextdomain`/`textdomain` for where the message catalogues
//! live, and the `locale -a` command for what could be selected. None of it
//! has a Rust equivalent: `std` deliberately does not touch the global
//! locale, and nvim's whole point in this file is to set it.
//!
//! Only the POSIX build was transpiled. Upstream's `lang_init` queries
//! CoreServices for the system language when `$LANG` is unset, which only
//! happens on macOS; here it is the no-op the `#ifdef` leaves behind, kept
//! because `option/defaults.rs` calls it unconditionally.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `os/` row in docs/perimeter.md.
#![allow(unsafe_code)]

use crate::buffer::maketitle;
use crate::cstr;
use crate::eval::vars::{set_vim_var_string, vim_var_string};
use crate::global_cell::GlobalCell;
use crate::memory::ThinCString;

use crate::message_fmt::msg_cstr;
use crate::option::{PROJECT_NAME, set_helplang_default};
use crate::os::cshim::{bindtextdomain, bump_catalogue_epoch, textdomain};
use crate::os::env::env_set;
use crate::path::{tail_index, tail_with_sep_index};
use crate::profile::time_msg_at;
use crate::semsg;
use crate::shell::{ShellOpts, get_cmd_output};
use crate::smsg;
use crate::types::Candidate;
use crate::types::{ExArg, Expand, MAXPATHL, Vv};
use ::libc::setlocale;
use core::ffi::{CStr, c_int};
use core::ptr;
use std::ffi::CString;

// The `<locale.h>` categories, glibc's numbering. `LC_MESSAGES` is the one
// upstream compiles conditionally: where it does not exist (Windows) the
// `:language messages` selector becomes a sentinel that reaches no
// `setlocale` call at all. On POSIX the two are the same number, so
// `VIM_LC_MESSAGES` below reads as upstream's `#define`.
const LC_CTYPE: c_int = 0;
const LC_NUMERIC: c_int = 1;
const LC_TIME: c_int = 2;
const LC_COLLATE: c_int = 3;
const LC_MESSAGES: c_int = 5;
const LC_ALL: c_int = 6;
const VIM_LC_MESSAGES: c_int = LC_MESSAGES;

/// The locale in effect for `what`, as libc names it, or `None` when it
/// names none.
fn locale(what: c_int) -> Option<CString> {
    // SAFETY: a NULL locale name queries rather than sets; the answer is
    // libc's NUL-terminated static storage (or NULL), copied before the next
    // `setlocale` can replace it.
    unsafe {
        setlocale(what, ptr::null())
            .as_ref()
            .map(|name| CStr::from_ptr(name).to_owned())
    }
}

/// Set the locale for `what` to `name`: whether libc accepted it.
fn set_locale(what: c_int, name: &CStr) -> bool {
    // SAFETY: a NUL-terminated name; the answer is only tested for NULL.
    !unsafe { setlocale(what, name.as_ptr()) }.is_null()
}

/// The locale's character encoding, as `nl_langinfo(CODESET)` names it, or
/// `None` when the C library names none.
pub(crate) fn locale_codeset() -> Option<Vec<u8>> {
    // SAFETY: `nl_langinfo` answers a NUL-terminated string in static
    // storage (or NULL), copied here before anything can replace it.
    let name = unsafe {
        ::libc::nl_langinfo(::libc::CODESET)
            .as_ref()
            .map(|name| CStr::from_ptr(name))
    }?;
    (!name.is_empty()).then(|| name.to_bytes().to_vec())
}

/// The locale in effect for character classification (`LC_CTYPE`), or
/// `None` when libc reports none.
pub(crate) fn ctype_locale() -> Option<Vec<u8>> {
    locale(LC_CTYPE)
        .filter(|name| !name.is_empty())
        .map(CString::into_bytes)
}

/// Whether `lang` starts with a language name — two letters. Rejects `""`,
/// `"C"` and `"C.UTF-8"`, which are not languages anybody has help files
/// for.
fn is_valid_mess_lang(lang: &CStr) -> bool {
    lang.to_bytes()
        .get(..2)
        .is_some_and(|head| head.iter().all(u8::is_ascii_alphabetic))
}

/// The messages language, as the default for `'helplang'`, if there is one.
pub(crate) fn mess_lang_of() -> Option<crate::memory::XString> {
    locale(LC_MESSAGES)
        .filter(|lang| is_valid_mess_lang(lang))
        .map(|lang| crate::memory::XString::from_cstr(&lang))
}

/// The language messages are shown in.
///
/// On POSIX this is just `LC_MESSAGES`; upstream's fallback chain through
/// `$LC_ALL`/`$LC_MESSAGES`/`$LANG` exists only for the platforms that lack
/// the category.
fn get_mess_env() -> Option<CString> {
    locale(LC_MESSAGES)
}

/// Publish the effective locale as `v:ctype`, `v:lang`, `v:lc_time` and
/// `v:collate`.
pub fn set_lang_var() {
    for (var, loc) in [
        (Vv::Ctype, locale(LC_CTYPE)),
        (Vv::Lang, get_mess_env()),
        (Vv::LcTime, locale(LC_TIME)),
        (Vv::Collate, locale(LC_COLLATE)),
    ] {
        // A locale libc names none for clears the variable.
        set_vim_var_string(var, loc.as_deref().map(CStr::to_bytes));
    }
}

/// Adopt the environment's locale, and point gettext at the message
/// catalogues shipped beside the binary.
///
/// `LC_NUMERIC` is forced back to `"C"` so `strtod` keeps reading a decimal
/// point rather than whatever the user's locale uses; vimscript's number
/// syntax is not localised.
pub fn init_locale() {
    set_locale(LC_ALL, c"");
    set_locale(LC_NUMERIC, c"C");

    // `$prefix/bin/nvim` -> `$prefix/share/locale`: drop the executable,
    // then overwrite the directory it sat in. `v:progpath` is set from the
    // executable path in `init_path`, which startup runs before this.
    let progpath = vim_var_string(Vv::Progpath).unwrap_or_else(ThinCString::empty);
    let progpath = progpath.as_cstr().to_bytes();
    let mut path = progpath[..progpath.len().min(MAXPATHL as usize - 1)].to_vec();
    path.truncate(tail_with_sep_index(&path));
    path.truncate(tail_index(&path));
    path.extend_from_slice(b"share/locale");
    path.truncate(MAXPATHL as usize - 1);
    let localepath = cstr::owned(&path);
    // SAFETY: two NUL-terminated strings; gettext copies the directory.
    unsafe {
        bindtextdomain(PROJECT_NAME.as_ptr(), localepath.as_ptr());
        textdomain(PROJECT_NAME.as_ptr());
    }

    time_msg_at(c"locale set");
}

/// The `:language` sub-commands, each with the category it selects and the
/// word `:language` echoes back for it.
///
/// Upstream requires at least three characters typed so that `me` and `ct`
/// stay available as two-letter *language* names.
const SELECTORS: [(&CStr, c_int, &CStr); 4] = [
    (c"messages", VIM_LC_MESSAGES, c"messages "),
    (c"ctype", LC_CTYPE, c"ctype "),
    (c"time", LC_TIME, c"time "),
    (c"collate", LC_COLLATE, c"collate "),
];

/// The selector `word` abbreviates, if any. `word` matches case-insensitively
/// and may be shortened, but never lengthened — which is what `STRNICMP` over
/// `word`'s length answers, since the ninth byte of `"messages"` is its NUL
/// and no byte of `word` can be.
fn selector_for(word: &[u8]) -> Option<(c_int, &'static CStr)> {
    if word.len() < 3 {
        return None;
    }
    SELECTORS
        .iter()
        .find(|(name, ..)| {
            name.to_bytes()
                .get(..word.len())
                .is_some_and(|n| n.eq_ignore_ascii_case(word))
        })
        .map(|&(_, what, whatstr)| (what, whatstr))
}

/// `:language [messages|ctype|time|collate] [{name}]` — report or set the
/// locale.
pub fn ex_language(excmd: &mut ExArg) {
    // A copy: setting the locale runs autocommands and reaches the
    // environment while the name is still read.
    let text = cstr::until_nul(excmd.line.arg()).to_vec();
    let is_white = |b: &u8| *b == b' ' || *b == b'\t';
    let word_end = text.iter().position(is_white).unwrap_or(text.len());
    let rest = &text[word_end..];
    let rest = &rest[rest.iter().take_while(|b| is_white(b)).count()..];

    let (what, whatstr, name) = match selector_for(&text[..word_end]) {
        Some((what, whatstr)) => (what, whatstr, rest),
        // No sub-command: the whole argument is the locale name, and
        // `:language` alone reports every category at once.
        None => (LC_ALL, c"", &text[..]),
    };
    if name.is_empty() {
        report(what, whatstr);
        return;
    }

    let name = cstr::owned(name);
    // `LC_NUMERIC` is restored for the same reason `init_locale` sets it:
    // `strtod` must keep seeing a decimal point.
    let accepted = set_locale(what, &name);
    set_locale(LC_NUMERIC, c"C");
    if !accepted {
        semsg!("E197: Cannot set language to \"{}\"", msg_cstr(&name));
        return;
    }
    // Stop cached translations from being reused: the catalogue selection
    // has just changed.
    bump_catalogue_epoch();
    // $LC_ALL would overrule everything set below.
    env_set(c"LC_ALL", c"");
    if what != LC_TIME && what != LC_COLLATE {
        // gettext does not consult the effective locale, so it has to be
        // told separately what to translate to.
        if what == LC_ALL {
            env_set(c"LANG", &name);
            // GNU gettext prefers $LANGUAGE over $LANG.
            env_set(c"LANGUAGE", c"");
        }
        if what != LC_CTYPE {
            env_set(c"LC_MESSAGES", &name);
            set_helplang_default(name.to_bytes());
        }
    }
    set_lang_var();
    maketitle();
}

/// `:language {category}` with no name: echo what is in effect.
fn report(what: c_int, whatstr: &CStr) {
    let current = if what == VIM_LC_MESSAGES {
        get_mess_env()
    } else {
        locale(what)
    };
    let current = current.filter(|name| !name.is_empty());
    let current = current.as_deref().unwrap_or(c"Unknown");
    smsg!(
        0,
        "Current {}language: \"{}\"",
        msg_cstr(whatstr),
        msg_cstr(current)
    );
}

/// Every locale `locale -a` reports, discovered once and then kept for the
/// life of the process — the completion machinery copies what it is handed,
/// so these never have to be freed and leaking them is what upstream does.
static LOCALES: GlobalCell<Option<&'static [CString]>> = GlobalCell::new(None);
static DID_INIT_LOCALES: GlobalCell<bool> = GlobalCell::new(false);

/// Run `locale -a` and split its output. `None` when the command could not be
/// run, which just means there is no locale completion.
fn find_locales() -> Option<&'static [CString]> {
    let text = get_cmd_output(c"locale -a", ShellOpts::SILENT)?;

    let locales: Vec<CString> = text
        .split(|&b| b == b'\n')
        // `strtok` yields no empty tokens, so neither do we.
        .filter(|line| !line.is_empty())
        .map(|line| CString::new(line).expect("no NUL inside a NUL-terminated buffer"))
        .collect();
    Some(Vec::leak(locales))
}

/// Populate [`LOCALES`] the first time completion asks for it.
fn init_locales() {
    if DID_INIT_LOCALES.get() {
        return;
    }
    DID_INIT_LOCALES.set(true);
    LOCALES.set(find_locales());
}

/// The `idx`th known locale name, or NULL past the end — the shape
/// `expand_generic` walks.
fn locale_name(idx: usize) -> Option<Candidate> {
    init_locales();
    // The slice is leaked, so the borrow outlives this one.
    LOCALES
        .get()
        .and_then(|locales| locales.get(idx))
        .map(|name| Candidate::Borrowed(name.as_c_str()))
}

/// `expand_generic` source for `:language`'s argument: the four sub-commands
/// first, then every locale (because `:language {name}` takes one directly).
pub fn get_lang_arg(_expand: &Expand, idx: usize) -> Option<Candidate> {
    match SELECTORS.get(idx) {
        Some((name, ..)) => Some(Candidate::Borrowed(name)),
        None => locale_name(idx - SELECTORS.len()),
    }
}

/// `expand_generic` source for `:language`'s locale names alone.
pub fn get_locales(_expand: &Expand, idx: usize) -> Option<Candidate> {
    locale_name(idx)
}

/// Nothing to do on POSIX: see the module docs.
pub fn lang_init() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_selector_may_be_abbreviated_but_not_lengthened() {
        assert_eq!(selector_for(b"mes").map(|s| s.0), Some(VIM_LC_MESSAGES));
        assert_eq!(
            selector_for(b"MESSAGES").map(|s| s.0),
            Some(VIM_LC_MESSAGES)
        );
        assert_eq!(selector_for(b"CoLl").map(|s| s.0), Some(LC_COLLATE));
        assert_eq!(selector_for(b"messagesx"), None);
        assert_eq!(selector_for(b"collates"), None);
    }

    #[test]
    fn two_letters_stay_available_as_language_names() {
        // "me" and "ct" would otherwise shadow real language names.
        assert_eq!(selector_for(b"me"), None);
        assert_eq!(selector_for(b"ct"), None);
        assert_eq!(selector_for(b""), None);
    }

    #[test]
    fn a_message_language_needs_two_leading_letters() {
        assert!(is_valid_mess_lang(c"de_DE.UTF-8"));
        assert!(is_valid_mess_lang(c"en"));
        assert!(!is_valid_mess_lang(c"C"));
        assert!(!is_valid_mess_lang(c"C.UTF-8"));
        assert!(!is_valid_mess_lang(c"e"));
        assert!(!is_valid_mess_lang(c""));
    }
}
