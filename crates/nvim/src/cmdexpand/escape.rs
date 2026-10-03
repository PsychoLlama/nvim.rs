//! Escaping a match, and whether fuzzy matching applies.
//!
//! [`wildescape`] puts back whatever the shell, the command line or a `:set`
//! value would otherwise eat, once per match, and [`escape_matches`] runs it
//! over a whole match list.  [`cmdline_fuzzy_complete`] answers whether
//! `'wildoptions'` asked for fuzzy matching *and* the context supports it —
//! the contexts that expand paths or option values never do.

#![forbid(unsafe_code)]

use super::*;
use crate::ex_getln::{VSE_BUFFER, VSE_NONE, VSE_SHELL, fnameescape, tilde_replace_matches};
use crate::option::vars::wop_flags;
use crate::options::kOptWopFlagFuzzy;
use crate::strings::escaped_bytes;
use crate::types::{BackslashEscape, ExpandContext};

/// Is fuzzy completion supported in this cmdline completion context?
///
/// The listed contexts answer no whatever `'wildoptions'` says: each of them
/// expands a path, an option value or a tag, where a fuzzy match would offer
/// something the command being completed cannot use.
pub(crate) fn cmdline_fuzzy_completion_supported(expand: &Expand) -> bool {
    match expand.context {
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
pub fn cmdline_fuzzy_complete(fuzzystr: &[u8]) -> bool {
    wop_flags.get() & kOptWopFlagFuzzy != 0 && !fuzzystr.is_empty()
}

/// `name` with a backslash in front: upstream's `escape_fname`.
fn with_backslash(name: &XString) -> XString {
    let mut escaped = XString::with_capacity(name.len() + 1);
    escaped.push_byte(b'\\');
    escaped.push_bytes(name);
    escaped
}

/// Escape special characters in the cmdline completion matches.
///
/// `pattern` is what produced them, needed only for its leading `"\~"`.
/// Both callers escape only when there is at least one match, which is what
/// makes the unconditional `matches[0]` at the end in bounds.
pub(crate) fn wildescape(expand: &mut Expand, pattern: &[u8], matches: &mut [XString]) {
    let context = expand.context;
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
            if expand.backslash.has(BackslashEscape::THREE) {
                let chars = if expand.backslash.has(BackslashEscape::COMMA) {
                    c" ,"
                } else {
                    c" "
                };
                *slot = escaped_bytes(slot.as_cstr(), chars);
            } else if expand.backslash.has(BackslashEscape::COMMA) && slot.contains(&b',') {
                *slot = escaped_bytes(slot.as_cstr(), c",");
            }
            *slot = fnameescape(
                slot.as_cstr(),
                if expand.shell { VSE_SHELL } else { vse_what },
            );

            // If the pattern starts with "\~", replace a leading "~" of the
            // match with "\~" as well.
            if pattern.starts_with(b"\\~") && slot.first() == Some(&b'~') {
                *slot = with_backslash(slot);
            }
        }
        expand.backslash = BackslashEscape::NONE;

        // If the first match starts with a '+' escape it.  Otherwise it
        // could be read as "+cmd".
        if matches[0].first() == Some(&b'+') {
            matches[0] = with_backslash(&matches[0]);
        }
    } else if context == ExpandContext::Tags {
        // Insert a backslash before characters in a tag name that would
        // terminate the ":tag" command.
        for slot in matches.iter_mut() {
            *slot = escaped_bytes(slot.as_cstr(), c"\\|\"");
        }
    }
}

/// Prepare a freshly expanded match list for use on the command line.
pub(crate) fn escape_matches(
    expand: &mut Expand,
    pattern: &[u8],
    matches: &mut [XString],
    options: WildOpts,
) {
    // May change home directory back to "~".
    if options.has(WildOpts::HOME_REPLACE) {
        tilde_replace_matches(pattern, matches);
    }
    if options.has(WildOpts::ESCAPE) {
        wildescape(expand, pattern, matches);
    }
}

/// [`cmdline_fuzzy_completion_supported`] for a bare context, for the oracle.
#[cfg(test)]
pub(super) fn fuzzy_supported(context: ExpandContext) -> bool {
    let mut xpc = Expand::new();
    xpc.context = context;
    cmdline_fuzzy_completion_supported(&xpc)
}
