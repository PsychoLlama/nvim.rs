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

use crate::ascii::ascii_iswhite;
use crate::buffer::maketitle;
use crate::charset::{skiptowhite, skipwhite};
use crate::eval::vars::{get_vim_var_str, set_vim_var_string};
use crate::global_cell::GlobalCell;
use crate::main::time_fd;
use crate::memory::{xfree, xstrlcpy};
use crate::option::{PROJECT_NAME, set_helplang_default};
use crate::os::cshim::_nl_msg_cat_cntr;
use crate::os::cshim::{bindtextdomain, gettext, textdomain};
use crate::os::env::os_setenv;
use crate::os::shell::{ShellOpts, get_cmd_output};
use crate::path::{path_tail, path_tail_with_sep};
use crate::profile::time_msg;
use crate::types::{
    MAXPATHL, VV_COLLATE, VV_CTYPE, VV_LANG, VV_LC_TIME, VV_PROGPATH, exarg_T, expand_T,
};
use crate::{semsg_c, smsg_c};
use ::libc::setlocale;
use core::ffi::{CStr, c_char, c_int};
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

/// The locale currently in effect for `what`, as libc reports it.
///
/// The pointer is libc's own static storage and is only valid until the next
/// `setlocale` call, which is why every caller consumes it immediately.
fn get_locale_val(what: c_int) -> *mut c_char {
    // SAFETY: a NULL locale name queries rather than sets. No pointer of
    // ours is involved.
    unsafe { setlocale(what, ptr::null()) }
}

/// Whether `lang` starts with a language name — two letters. Rejects NULL,
/// `""`, `"C"` and `"C.UTF-8"`, which are not languages anybody has help
/// files for.
fn is_valid_mess_lang(lang: *const c_char) -> bool {
    if lang.is_null() {
        return false;
    }
    // SAFETY: a non-NULL locale name is NUL-terminated, and `&&`
    // short-circuits before the second byte exactly as the C `&&` does, so a
    // one-character name is never read past its NUL.
    unsafe { (*lang as u8).is_ascii_alphabetic() && (*lang.add(1) as u8).is_ascii_alphabetic() }
}

/// The messages language, as the default for `'helplang'`. May be NULL.
pub fn get_mess_lang() -> *mut c_char {
    let p = get_locale_val(LC_MESSAGES);
    if is_valid_mess_lang(p) {
        p
    } else {
        ptr::null_mut()
    }
}

/// The language messages are shown in.
///
/// On POSIX this is just `LC_MESSAGES`; upstream's fallback chain through
/// `$LC_ALL`/`$LC_MESSAGES`/`$LANG` exists only for the platforms that lack
/// the category.
fn get_mess_env() -> *mut c_char {
    get_locale_val(LC_MESSAGES)
}

/// Publish the effective locale as `v:ctype`, `v:lang`, `v:lc_time` and
/// `v:collate`.
pub fn set_lang_var() {
    for (var, loc) in [
        (VV_CTYPE, get_locale_val(LC_CTYPE)),
        (VV_LANG, get_mess_env()),
        (VV_LC_TIME, get_locale_val(LC_TIME)),
        (VV_COLLATE, get_locale_val(LC_COLLATE)),
    ] {
        // SAFETY: each value is libc's NUL-terminated locale name (or NULL,
        // which `set_vim_var_string` documents as clearing the variable),
        // and is consumed before the next `setlocale` invalidates it. -1 asks
        // for the whole string.
        unsafe { set_vim_var_string(var, loc, -1) };
    }
}

/// Adopt the environment's locale, and point gettext at the message
/// catalogues shipped beside the binary.
///
/// `LC_NUMERIC` is forced back to `"C"` so `strtod` keeps reading a decimal
/// point rather than whatever the user's locale uses; vimscript's number
/// syntax is not localised.
pub fn init_locale() {
    let mut localepath: [c_char; MAXPATHL as usize] = [0; MAXPATHL as usize];
    // SAFETY: both locale names are static NUL-terminated literals, and
    // `v:progpath` is set from the executable path in `init_path`, which
    // startup runs before this, so it is a non-NULL NUL-terminated string.
    // Both `path_tail*` answer a pointer inside the buffer they are given, so
    // `used` is in bounds and the second `xstrlcpy` gets exactly the room
    // left. Everything is derived from the one `base` pointer.
    unsafe {
        setlocale(LC_ALL, c"".as_ptr());
        setlocale(LC_NUMERIC, c"C".as_ptr());

        // `$prefix/bin/nvim` -> `$prefix/share/locale`: drop the executable,
        // then overwrite the directory it sat in.
        let base = localepath.as_mut_ptr();
        xstrlcpy(base, get_vim_var_str(VV_PROGPATH), MAXPATHL as usize);
        *path_tail_with_sep(base) = 0;
        let tail = path_tail(base);
        let used = tail.offset_from(base) as usize;
        xstrlcpy(tail, c"share/locale".as_ptr(), MAXPATHL as usize - used);
        bindtextdomain(PROJECT_NAME.as_ptr(), base);
        textdomain(PROJECT_NAME.as_ptr());
    }

    if !time_fd.get().is_null() {
        // SAFETY: a static message and no start time, which `time_msg`
        // documents as "report the elapsed total".
        unsafe { time_msg(c"locale set".as_ptr(), ptr::null()) };
    }
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
///
/// # Safety
/// `eap` must point at a live [`exarg_T`] whose `arg` is NUL-terminated.
pub unsafe fn ex_language(eap: *mut exarg_T) {
    // SAFETY: the caller's contract. `skiptowhite` stays inside `arg`, so the
    // slice between them is in bounds and initialised.
    let (arg, word, name) = unsafe {
        let arg = (*eap).arg;
        let p = skiptowhite(arg);
        let len = p.offset_from(arg) as usize;
        let ends_word = *p == 0 || ascii_iswhite(*p as c_int);
        let word = if ends_word {
            core::slice::from_raw_parts(arg as *const u8, len)
        } else {
            b""
        };
        (arg, word, skipwhite(p))
    };

    let (what, whatstr, name) = match selector_for(word) {
        Some((what, whatstr)) => (what, whatstr, name),
        // No sub-command: the whole argument is the locale name, and
        // `:language` alone reports every category at once.
        None => (LC_ALL, c"", arg),
    };

    // SAFETY: `name` points into `eap->arg`, so reading its first byte is in
    // bounds.
    if unsafe { *name } == 0 {
        report(what, whatstr);
        return;
    }

    // SAFETY: `name` is NUL-terminated. `LC_NUMERIC` is restored for the same
    // reason `init_locale` sets it: `strtod` must keep seeing a decimal point.
    let loc = unsafe {
        let loc = setlocale(what, name);
        setlocale(LC_NUMERIC, c"C".as_ptr());
        loc
    };
    if loc.is_null() {
        // SAFETY: `semsg` is printf-shaped and `name` outlives the call.
        unsafe {
            semsg_c!(
                gettext(c"E197: Cannot set language to \"%s\"".as_ptr()),
                name,
            )
        };
        return;
    }
    // SAFETY: `_nl_msg_cat_cntr` is GNU gettext's "the catalogue selection
    // changed" counter; bumping it is how upstream stops cached translations
    // from being reused, and nothing else in the process touches it. Every
    // environment variable name below is a static literal, and `name` is
    // NUL-terminated.
    unsafe {
        _nl_msg_cat_cntr += 1;

        // $LC_ALL would overrule everything set below.
        os_setenv(c"LC_ALL".as_ptr(), c"".as_ptr(), 1);
        if what != LC_TIME && what != LC_COLLATE {
            // gettext does not consult the effective locale, so it has to be
            // told separately what to translate to.
            if what == LC_ALL {
                os_setenv(c"LANG".as_ptr(), name, 1);
                // GNU gettext prefers $LANGUAGE over $LANG.
                os_setenv(c"LANGUAGE".as_ptr(), c"".as_ptr(), 1);
            }
            if what != LC_CTYPE {
                os_setenv(c"LC_MESSAGES".as_ptr(), name, 1);
                set_helplang_default(name);
            }
        }
        set_lang_var();
        maketitle();
    }
}

/// `:language {category}` with no name: echo what is in effect.
fn report(what: c_int, whatstr: &CStr) {
    // SAFETY: a NULL locale name queries rather than sets, and the answer is
    // libc's own NUL-terminated storage, valid until the next `setlocale`.
    // `smsg` is printf-shaped, so neither argument becomes the format string,
    // and both outlive the call.
    unsafe {
        let mut p = if what == VIM_LC_MESSAGES {
            get_mess_env()
        } else {
            setlocale(what, ptr::null())
        };
        if p.is_null() || *p == 0 {
            p = c"Unknown".as_ptr().cast_mut();
        }
        smsg_c!(
            0,
            gettext(c"Current %slanguage: \"%s\"".as_ptr()),
            whatstr.as_ptr(),
            p.cast_const(),
        );
    }
}

/// Every locale `locale -a` reports, discovered once and then kept for the
/// life of the process — the completion machinery copies what it is handed,
/// so these never have to be freed and leaking them is what upstream does.
static LOCALES: GlobalCell<Option<&'static [CString]>> = GlobalCell::new(None);
static DID_INIT_LOCALES: GlobalCell<bool> = GlobalCell::new(false);

/// Run `locale -a` and split its output. `None` when the command could not be
/// run, which just means there is no locale completion.
fn find_locales() -> Option<&'static [CString]> {
    // SAFETY: a static command line, no input file and no length wanted. The
    // buffer that comes back is NUL-terminated and ours to free; nothing
    // refers to it past the `xfree`, because `text` owns a copy.
    let text = unsafe {
        let out = get_cmd_output(
            c"locale -a".as_ptr().cast_mut(),
            ptr::null_mut(),
            ShellOpts::SILENT,
            ptr::null_mut(),
        );
        if out.is_null() {
            return None;
        }
        let text = CStr::from_ptr(out).to_bytes().to_vec();
        xfree(out.cast());
        text
    };

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
/// `ExpandGeneric` walks.
fn locale_name(idx: c_int) -> *mut c_char {
    init_locales();
    let idx = usize::try_from(idx).ok();
    match (LOCALES.get(), idx) {
        // The slice is leaked, so the pointer outlives this borrow.
        (Some(locales), Some(idx)) => locales
            .get(idx)
            .map_or(ptr::null_mut(), |name| name.as_ptr().cast_mut()),
        _ => ptr::null_mut(),
    }
}

/// `ExpandGeneric` source for `:language`'s argument: the four sub-commands
/// first, then every locale (because `:language {name}` takes one directly).
pub fn get_lang_arg(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    match SELECTORS.get(idx as usize) {
        Some((name, ..)) => name.as_ptr().cast::<c_char>().cast_mut(),
        None => locale_name(idx - SELECTORS.len() as c_int),
    }
}

/// `ExpandGeneric` source for `:language`'s locale names alone.
pub fn get_locales(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
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
        assert!(is_valid_mess_lang(c"de_DE.UTF-8".as_ptr()));
        assert!(is_valid_mess_lang(c"en".as_ptr()));
        assert!(!is_valid_mess_lang(c"C".as_ptr()));
        assert!(!is_valid_mess_lang(c"C.UTF-8".as_ptr()));
        assert!(!is_valid_mess_lang(c"".as_ptr()));
        assert!(!is_valid_mess_lang(ptr::null()));
    }
}
