//! Turning a context into a match list.
//!
//! [`expand_from_context`] is the dispatcher: file-like contexts go to
//! `expand_wildcards`, everything else to a generator, and the answer is
//! sorted and escaped.  [`expand_generic`] is the generic generator loop
//! every `get_*_name` callback is driven by, and [`map_wildopts_to_ewflags`]
//! translates the `WILD_*` options into `EW_*`.

#![forbid(unsafe_code)]

use super::*;
use crate::buffer::buf_name_matches;
use crate::ex_docmd::expand_argopt;
use crate::fuzzy::{FUZZY_SCORE_NONE, fuzzy_match_str};
use crate::help::help_tag_matches;
use crate::lua::executor::nlua_expand_matches;
use crate::mapping::expand_mappings;
use crate::menu::get_menu_names;
use crate::option::{
    expand_old_setting, expand_setting_subtract, expand_settings, expand_string_setting,
    magic_isset,
};
use crate::path::ExpandFlags;
use crate::regexp::{OwnedMatch, RE_MAGIC, vim_regexec};
use crate::runtime::{RuntimeOpts, packadd_dir_matches, runtime_cmd_matches, runtime_dir_matches};
use crate::search::ignorecase_of;
use crate::strings::escaped_bytes;
use crate::syntax::reset_expand_highlight;
use crate::tag::tag_matches;
use crate::types::{CompleteListItemGetter, ExpandContext, Failed, RegMatch};
use core::ffi::CStr;
use std::ffi::CString;

/// The `WILD_*` options that name an `EW_*` flag one-for-one.
const WILDOPT_TO_EW: [(WildOpts, ExpandFlags); 6] = [
    (WildOpts::LIST_NOTFOUND, ExpandFlags::NOTFOUND),
    (WildOpts::ADD_SLASH, ExpandFlags::ADDSLASH),
    (WildOpts::KEEP_ALL, ExpandFlags::KEEPALL),
    (WildOpts::SILENT, ExpandFlags::SILENT),
    (WildOpts::NOERROR, ExpandFlags::NOERROR),
    (WildOpts::ALLLINKS, ExpandFlags::ALLLINKS),
];

/// Translate the `WILD_*` options into the `EW_*` flags `expand_wildcards`
/// takes.  `ExpandFlags::DIR` — include directories — is always on.
pub(crate) fn map_wildopts_to_ewflags(options: WildOpts) -> ExpandFlags {
    WILDOPT_TO_EW
        .iter()
        .fold(ExpandFlags::DIR, |flags, &(wild, ew)| {
            if options.has(wild) { flags | ew } else { flags }
        })
}

/// A runtime-directory completion that wants nothing special: `'runtimepath'`
/// as it stands, no package trees and no `after/` filter. Upstream's bare `0`.
const RTP_ONLY: RuntimeOpts = RuntimeOpts::NONE;

/// Do the expansion based on `expand.context` and `pat`.
///
/// `options` is a set of `WILD_*` flags.  Most contexts have a generator of
/// their own; the ones that do not fall through to [`expand_other`]'s table,
/// and all of those run against a compiled regexp (or, under
/// `'wildoptions'`=fuzzy, against `fuzzy_match_str`).
pub(crate) fn expand_from_context(
    expand: &mut Expand,
    pat: &CStr,
    options: WildOpts,
) -> Result<Vec<XString>, Failed> {
    let flags = map_wildopts_to_ewflags(options);
    let fuzzy =
        cmdline_fuzzy_complete(pat.to_bytes()) && cmdline_fuzzy_completion_supported(expand);
    let context = expand.context;

    if matches!(
        context,
        ExpandContext::Files
            | ExpandContext::Directories
            | ExpandContext::FilesInPath
            | ExpandContext::Findfunc
            | ExpandContext::DirsInCdpath
    ) {
        return expand_files_and_dirs(expand, pat, flags, options);
    }

    // The contexts with a generator of their own.
    match context {
        ExpandContext::Help => {
            // With an empty argument we would get all the help tags,
            // which is very slow.  Get matches for "help" instead.
            let arg = if pat.is_empty() { c"help" } else { pat };
            return help_tag_matches(arg);
        }
        ExpandContext::ShellCmd => return Ok(expand_shellcmd(pat, flags)),
        ExpandContext::OldSetting => return expand_old_setting(),
        ExpandContext::Buffers => return buf_name_matches(pat, options),
        ExpandContext::DiffBuffers => return buf_name_matches(pat, options | BUF_DIFF_FILTER),
        ExpandContext::Tags | ExpandContext::TagsListFiles => {
            return tag_matches(context == ExpandContext::Tags, pat);
        }
        ExpandContext::Colors => {
            let opts = RuntimeOpts::START | RuntimeOpts::OPT;
            return runtime_dir_matches(pat, opts, &[c"colors"]);
        }
        ExpandContext::Compiler => return runtime_dir_matches(pat, RTP_ONLY, &[c"compiler"]),
        ExpandContext::Ownsyntax => return runtime_dir_matches(pat, RTP_ONLY, &[c"syntax"]),
        ExpandContext::Filetype => {
            let dirs = [c"syntax", c"indent", c"ftplugin"];
            return runtime_dir_matches(pat, RTP_ONLY, &dirs);
        }
        ExpandContext::Keymap => return runtime_dir_matches(pat, RTP_ONLY, &[c"keymap"]),
        ExpandContext::UserList => return expand_user_list(expand),
        ExpandContext::UserLua => return expand_user_lua(expand),
        ExpandContext::Packadd => return packadd_dir_matches(pat),
        ExpandContext::Runtime => return runtime_cmd_matches(pat),
        ExpandContext::PatternInBuf => return expand_pattern_in_buf(pat, expand.search_dir),
        _ => {}
    }

    // When expanding a function name starting with s:, match the <SNR>nr_
    // prefix.
    let snr;
    let pat = match pat.to_bytes().strip_prefix(b"^s:") {
        Some(name) if context == ExpandContext::UserFunc => {
            let mut text = b"^<SNR>\\d\\+_".to_vec();
            text.extend_from_slice(name);
            snr = CString::new(text).expect("a pattern holds no NUL");
            snr.as_c_str()
        }
        _ => pat,
    };

    if context == ExpandContext::Lua {
        return nlua_expand_matches();
    }

    let mut regmatch = if fuzzy {
        OwnedMatch::none()
    } else {
        let flags = if magic_isset() { RE_MAGIC } else { 0 };
        // Set ignore-case according to 'ignorecase', 'smartcase' and pat.
        OwnedMatch::compile(pat, flags, ignorecase_of(pat)).ok_or(Failed)?
    };
    let regmatch: &mut RegMatch = &mut regmatch;

    match context {
        ExpandContext::Settings | ExpandContext::BoolSettings => {
            expand_settings(expand, regmatch, pat, fuzzy)
        }
        ExpandContext::StringSetting => expand_string_setting(expand, regmatch),
        ExpandContext::SettingSubtract => expand_setting_subtract(expand, regmatch),
        ExpandContext::Mappings => expand_mappings(pat, regmatch),
        ExpandContext::Argopt => expand_argopt(pat, expand, regmatch),
        ExpandContext::UserDefined => expand_user_defined(pat, expand, regmatch),
        _ => expand_other(pat, expand, regmatch),
    }
}

/// Expand a list of names.
///
/// The generic command-line completion loop: `func` is called with rising
/// indices until it answers `None`, each name is matched against `regmatch`
/// (or scored by `fuzzy_match_str`), and the survivors are copied out.
///
/// `escaped` asks for spaces, tabs, backslashes and dots to be escaped in
/// each match.
pub fn expand_generic(
    pat: &CStr,
    expand: &Expand,
    regmatch: &mut RegMatch,
    func: CompleteListItemGetter,
    escaped: bool,
) -> Vec<XString> {
    let fuzzy = cmdline_fuzzy_complete(pat.to_bytes());
    let mut found = Vec::new();
    let mut scored = Vec::new();

    for i in 0.. {
        let Some(candidate) = func(expand, i) else {
            break; // end of list
        };
        if candidate.is_empty() {
            continue; // skip empty strings
        }

        // An empty pattern matches everything; otherwise every
        // candidate is tested, and under 'wildoptions'=fuzzy also scored.
        let mut score = 0;
        let matched = if expand.pattern_is_empty() {
            true
        } else if fuzzy {
            score = fuzzy_match_str(&candidate, pat);
            score != FUZZY_SCORE_NONE
        } else {
            vim_regexec(regmatch, &candidate, 0)
        };
        if !matched {
            continue;
        }

        let mut text = if escaped {
            escaped_bytes(&candidate, c" \t\\.")
        } else {
            XString::from_cstr(&candidate)
        };

        if core::ptr::fn_addr_eq(func, get_menu_names as CompleteListItemGetter)
            && text.last() == Some(&1)
        {
            // Undo the separator get_menu_names() added, in the copy that
            // is kept.
            text.truncate(text.len() - 1);
            text.push_byte(b'.');
        }

        if fuzzy {
            let idx = scored.len();
            scored.push(Scored { text, score, idx });
        } else {
            found.push(text);
        }
    }

    if found.is_empty() && scored.is_empty() {
        return Vec::new();
    }

    // Sort the matches when using regular expression matching and sorting
    // applies to the completion context.  Menus and scriptnames should be
    // kept in the order they were given in.
    let sort_matches = !fuzzy
        && !matches!(
            expand.context,
            ExpandContext::Menunames
                | ExpandContext::StringSetting
                | ExpandContext::Menus
                | ExpandContext::Scriptnames
                | ExpandContext::Argopt
        );
    // <SNR> functions should be sorted to the end.
    let funcsort = matches!(
        expand.context,
        ExpandContext::Expression | ExpandContext::Functions | ExpandContext::UserFunc
    );

    if sort_matches {
        if funcsort {
            found.sort_unstable_by(|a: &XString, b: &XString| {
                (a.first() == Some(&b'<'), &a[..]).cmp(&(b.first() == Some(&b'<'), &b[..]))
            });
        } else {
            found.sort_unstable_by(|a: &XString, b: &XString| a[..].cmp(&b[..]));
        }
    }

    let found = if fuzzy {
        fuzzy_sorted(scored, funcsort)
    } else {
        found
    };

    // Reset the variables used for special highlight names expansion, so
    // that they don't show up when getting normal highlight names by ID.
    reset_expand_highlight();
    found
}
