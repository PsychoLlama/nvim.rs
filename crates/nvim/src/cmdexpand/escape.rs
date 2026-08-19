//! Escaping a match, and whether fuzzy matching applies.
//!
//! [`wildescape`] puts back whatever the shell, the command line or a `:set`
//! value would otherwise eat, once per match, and [`escape_matches`] runs it
//! over a whole match array.  [`cmdline_fuzzy_complete`] answers whether
//! `'wildoptions'` asked for fuzzy matching *and* the context supports it —
//! the contexts that expand paths or option values never do.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cmdexpand::WildOpts;
use crate::types::{BackslashEscape, ExpandContext};
use core::ffi::{c_char, c_int, c_void};

/// Is fuzzy completion supported in this cmdline completion context?
///
/// The listed contexts answer no whatever `'wildoptions'` says: each of them
/// expands a path, an option value or a tag, where a fuzzy match would offer
/// something the command being completed cannot use.
pub(crate) unsafe fn cmdline_fuzzy_completion_supported(xp: *const expand_T) -> bool {
    let context = unsafe { (*xp).xp_context };
    match context {
        ExpandContext::BoolSettings
        | ExpandContext::Colors
        | ExpandContext::Compiler
        | ExpandContext::Directories
        | ExpandContext::DirsInCdpath
        | ExpandContext::Files
        | ExpandContext::FilesInPath
        | ExpandContext::Filetype
        | ExpandContext::FiletypeCmd
        | ExpandContext::Findfunc
        | ExpandContext::Help
        | ExpandContext::Keymap
        | ExpandContext::Lua
        | ExpandContext::OldSetting
        | ExpandContext::StringSetting
        | ExpandContext::SettingSubtract
        | ExpandContext::Ownsyntax
        | ExpandContext::Packadd
        | ExpandContext::Runtime
        | ExpandContext::ShellCmd
        | ExpandContext::ShellCmdLine
        | ExpandContext::Tags
        | ExpandContext::TagsListFiles
        | ExpandContext::UserList
        | ExpandContext::UserLua => false,
        _ => wop_flags.get() & kOptWopFlagFuzzy != 0,
    }
}

/// Is fuzzy cmdline completion enabled, with a non-empty pattern to match?
///
/// An empty search pattern never fuzzy-matches: it would score every candidate
/// alike and throw away the sort order the caller wants.
pub unsafe fn cmdline_fuzzy_complete(fuzzystr: *const c_char) -> bool {
    wop_flags.get() & kOptWopFlagFuzzy != 0 && unsafe { *fuzzystr } != 0
}

/// `qsort` comparator for the completion matches: plain `strcmp`, except that
/// `<SNR>` functions sort to the end.
///
/// Stays `extern "C"`: it is handed to `qsort`.
pub(crate) unsafe extern "C" fn sort_func_compare(s1: *const c_void, s2: *const c_void) -> c_int {
    unsafe {
        let p1 = *s1.cast::<*mut c_char>();
        let p2 = *s2.cast::<*mut c_char>();
        match (*p1 == b'<' as c_char, *p2 == b'<' as c_char) {
            (false, true) => -1,
            (true, false) => 1,
            _ => strcmp(p1, p2),
        }
    }
}

/// Escape special characters in the cmdline completion matches.
///
/// `str` is the pattern that produced them, needed only for its leading
/// `"\~"`.  Both callers expand only when there is at least one match, which
/// is what makes the unconditional `matches[0]` at the end in bounds.
pub(crate) unsafe fn wildescape(
    xp: *mut expand_T,
    str: *const c_char,
    matches: &mut [*mut c_char],
) {
    unsafe {
        // Free the string in `slot` and put `escaped` in its place.  Every
        // escaping step builds the new string out of the old one, so the
        // replacement is always computed before the call.  A closure rather
        // than a free function: it inherits this block, where a free
        // function would have to state one of its own.
        let put = |slot: &mut *mut c_char, escaped: *mut c_char| {
            xfree(core::mem::replace(slot, escaped) as *mut c_void)
        };
        let context = (*xp).xp_context;
        if matches!(
            context,
            ExpandContext::Files
                | ExpandContext::FilesInPath
                | ExpandContext::ShellCmd
                | ExpandContext::Buffers
                | ExpandContext::Directories
                | ExpandContext::DirsInCdpath
        ) {
            let vse_what = if context == ExpandContext::Buffers {
                VSE_BUFFER
            } else {
                VSE_NONE
            };
            // Insert a backslash into a file name before a space, \, %, #
            // and wildmatch characters, except '~'.
            for slot in matches.iter_mut() {
                // For ":set path=" we need to escape spaces twice.
                if (*xp).xp_backslash.has(BackslashEscape::THREE) {
                    let pat = if (*xp).xp_backslash.has(BackslashEscape::COMMA) {
                        c" ,"
                    } else {
                        c" "
                    };
                    let escaped = vim_strsave_escaped(*slot, pat.as_ptr());
                    put(slot, escaped);
                } else if (*xp).xp_backslash.has(BackslashEscape::COMMA)
                    && !vim_strchr(*slot, ',' as c_int).is_null()
                {
                    let escaped = vim_strsave_escaped(*slot, c",".as_ptr());
                    put(slot, escaped);
                }
                let escaped = vim_strsave_fnameescape(
                    *slot,
                    if (*xp).xp_shell { VSE_SHELL } else { vse_what },
                );
                put(slot, escaped);

                // If "str" starts with "\~", replace a leading "~" of the
                // match with "\~" as well.
                if *str == b'\\' as c_char
                    && *str.add(1) == b'~' as c_char
                    && **slot == b'~' as c_char
                {
                    escape_fname(slot);
                }
            }
            (*xp).xp_backslash = BackslashEscape::NONE;

            // If the first match starts with a '+' escape it.  Otherwise it
            // could be read as "+cmd".
            if *matches[0] == b'+' as c_char {
                escape_fname(&mut matches[0]);
            }
        } else if context == ExpandContext::Tags {
            // Insert a backslash before characters in a tag name that would
            // terminate the ":tag" command.
            for slot in matches.iter_mut() {
                let escaped = vim_strsave_escaped(*slot, c"\\|\"".as_ptr());
                put(slot, escaped);
            }
        }
    }
}

/// Prepare a freshly expanded match array for use on the command line.
pub(crate) unsafe fn escape_matches(
    xp: *mut expand_T,
    str: *mut c_char,
    matches: &mut [*mut c_char],
    options: WildOpts,
) {
    unsafe {
        // May change home directory back to "~".
        if options.has(WildOpts::HOME_REPLACE) {
            tilde_replace(str, matches.len() as c_int, matches.as_mut_ptr());
        }
        if options.has(WildOpts::ESCAPE) {
            wildescape(xp, str, matches);
        }
    }
}
