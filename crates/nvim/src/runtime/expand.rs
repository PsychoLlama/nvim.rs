//! Command-line completion for the runtime commands.
//!
//! [`runtime_dir_matches`] completes a file name against a set of
//! 'runtimepath' subdirectories -- what `:colorscheme`, `:compiler`,
//! `:runtime` and friends offer -- and [`packadd_dir_matches`] does the same
//! for `:packadd` against 'packpath'.  [`runtime_cmd_matches`] is `:runtime`'s
//! own two-stage completion, where the first word may be one of the
//! `START`/`OPT`/`PACK`/`ALL` qualifiers and everything after it is a path.
//!
//! Every match is offered as a *bare* name: the directory the search started
//! from and the `.vim`/`.lua` extension are trimmed back off again, keeping
//! only as many path components as the pattern itself spelled
//! ([`trim_match`]).

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::cmdexpand::WildOpts;
use crate::cstr;
use crate::mbyte::head_off;
use crate::option::vars::{P_PP, P_RTP};
use crate::path::remove_duplicate_names;

use crate::types::Failed;
use core::ffi::{CStr, c_int};
use std::ffi::CString;

/// What a completion round looks for: script files, or — on the second round,
/// where a directory name is being completed — anything.
const SCRIPTS: &[u8] = b"*.{vim,lua}";
const ANYTHING: &[u8] = b"*";

/// `{prefix}{dir}/{pat}{suffix}`, dropping the directory and its separator
/// when `dir` is empty.
fn build_pattern(prefix: &[u8], dir: &[u8], pat: &[u8], suffix: &[u8]) -> CString {
    let sep: &[u8] = if dir.is_empty() { b"" } else { b"/" };
    cstr::owned(&[prefix, dir, sep, pat, suffix].concat())
}

/// Glob `pat` under `dir` in 'runtimepath' and in 'packpath''s package trees,
/// collecting the matches in `found`.
///
/// When `dir` is empty this runs twice: once for script files, once more with
/// `WildOpts::ADD_SLASH` for directories, which is what makes `:runtime`
/// complete a subdirectory name.
///
/// The two options are copied: the expansion can run user code.
fn glob_rounds(pat: &[u8], dir: &[u8], flags: RuntimeOpts, found: &mut Vec<XString>) {
    let (rtp, pp) = (P_RTP.get(), P_PP.get());
    let mut glob_flags = WildOpts::NONE;
    let mut expand_dirs = false;
    let mut buf = build_pattern(b"", dir, pat, SCRIPTS);
    loop {
        if !flags.has(RuntimeOpts::NORTP) {
            found.extend(globpath(rtp.as_cstr(), &buf, glob_flags, expand_dirs));
        }
        let suffix = if expand_dirs { ANYTHING } else { SCRIPTS };
        if flags.has(RuntimeOpts::START) {
            for prefix in [&b"pack/*/start/*/"[..], b"start/*/"] {
                buf = build_pattern(prefix, dir, pat, suffix);
                found.extend(globpath(pp.as_cstr(), &buf, glob_flags, expand_dirs));
            }
        }
        if flags.has(RuntimeOpts::OPT) {
            for prefix in [&b"pack/*/opt/*/"[..], b"opt/*/"] {
                buf = build_pattern(prefix, dir, pat, suffix);
                found.extend(globpath(pp.as_cstr(), &buf, glob_flags, expand_dirs));
            }
        }
        // Second round, for directories.
        if !dir.is_empty() || expand_dirs {
            return;
        }
        buf = cstr::owned(&[pat, b"*"].concat());
        glob_flags = WildOpts::ADD_SLASH;
        expand_dirs = true;
    }
}

/// Cut a match back to the name the user is completing: no `.vim`/`.lua`
/// extension, and no more leading path components than the pattern spelled.
fn trim_match(matched: &XString, keep_ext: bool, pat_pathsep_cnt: usize) -> XString {
    let mut text: &[u8] = matched;
    if text.len() > 4 && !keep_ext {
        let ext = &text[text.len() - 4..];
        if ext.eq_ignore_ascii_case(b".vim") || ext.eq_ignore_ascii_case(b".lua") {
            text = &text[..text.len() - 4];
        }
    }
    let e = text.len();

    // A trailing slash is a component the pattern did not have to spell.
    let mut match_pathsep_cnt: isize = if text.last() == Some(&b'/') { -1 } else { 0 };
    let pat_pathsep_cnt = isize::try_from(pat_pathsep_cnt).unwrap_or(isize::MAX);
    // Walk back a character at a time from the terminator; upstream reads
    // the byte at the walk's position, which starts at the terminator.
    let mut s = e;
    while s > 0 {
        if vim_ispathsep(c_int::from(cstr::byte_at(text, s))) {
            match_pathsep_cnt += 1;
            if match_pathsep_cnt > pat_pathsep_cnt {
                break;
            }
        }
        s -= head_off(text, s - 1) + 1;
    }
    // An empty match leaves the walk at its start, and upstream's copy of
    // nothing leaves it empty.
    XString::from_bytes(&text[(s + 1).min(e)..e])
}

/// Collect the completion matches for `pat` under each of `dirnames`, trimmed,
/// sorted and without duplicates.
fn collect_runtime_matches(
    pat: &[u8],
    flags: RuntimeOpts,
    keep_ext: bool,
    dirnames: &[&CStr],
) -> Vec<XString> {
    // TODO(bfredl): this is bullshit, expandpath should not reinvent path
    // logic.
    let mut found = Vec::new();
    for dir in dirnames {
        glob_rounds(pat, dir.to_bytes(), flags, &mut found);
    }

    let pat_pathsep_cnt = pat
        .iter()
        .filter(|&&b| vim_ispathsep(c_int::from(b)))
        .count();
    let mut found: Vec<XString> = found
        .iter()
        .map(|matched| trim_match(matched, keep_ext, pat_pathsep_cnt))
        .collect();

    if !found.is_empty() {
        // Sort and remove the duplicates that several `dirnames` produce.
        remove_duplicate_names(&mut found);
    }
    found
}

/// The matches, or `Err` when there are none.
fn some(found: Vec<XString>) -> Result<Vec<XString>, Failed> {
    if found.is_empty() {
        Err(Failed)
    } else {
        Ok(found)
    }
}

/// Expand color scheme, compiler or filetype names.
///
/// Searches `{runtimepath}/{dirs}/{pat}.{vim,lua}`; `RuntimeOpts::START` adds
/// `{packpath}/pack/*/start/*/{dirs}/...` and `RuntimeOpts::OPT` the `opt`
/// equivalent.  `dirs` holds one or more directory names.
pub(crate) fn runtime_dir_matches(
    pat: &CStr,
    flags: RuntimeOpts,
    dirs: &[&CStr],
) -> Result<Vec<XString>, Failed> {
    some(collect_runtime_matches(pat.to_bytes(), flags, false, dirs))
}

/// The `[where]` qualifiers `:runtime` completion offers.
///
/// Deliberately a second copy of the words `runtime_cmd_flags` parses:
/// what completion proposes and what the command accepts are separate
/// questions, and upstream spells them separately too.
const WHERE_VALUES: [&CStr; 4] = [c"START", c"OPT", c"PACK", c"ALL"];

/// Command-line completion for the `:runtime` command.
pub(crate) fn runtime_cmd_matches(pat: &CStr) -> Result<Vec<XString>, Failed> {
    let pat = pat.to_bytes();
    let flags = RUNTIME_EXPAND_FLAGS.get();
    let mut found = collect_runtime_matches(pat, flags, true, &[c""]);

    // Complete the [where] argument too, when none was given.
    if flags == RuntimeOpts::NONE {
        for value in WHERE_VALUES {
            // `strncmp` over the pattern's length: the word starts with it.
            if value.to_bytes().starts_with(pat) {
                found.push(XString::from_cstr(value));
            }
        }
    }
    some(found)
}

/// Expand `:packadd` names: `{packpath}/pack/*/opt/{pat}`.
pub(crate) fn packadd_dir_matches(pat: &CStr) -> Result<Vec<XString>, Failed> {
    let pp = P_PP.get();
    let mut found = Vec::new();
    for prefix in [&b"pack/*/opt/"[..], b"opt/"] {
        let glob = cstr::owned(&[prefix, pat.to_bytes(), b"*"].concat());
        found.extend(globpath(pp.as_cstr(), &glob, WildOpts::NONE, true));
    }

    // Offer the package name, not the path it was found at.
    let mut found: Vec<XString> = found
        .iter()
        .map(|matched| XString::from_bytes(&matched[tail_index(matched)..]))
        .collect();

    if found.is_empty() {
        return Err(Failed);
    }
    // Sort and remove the duplicates the two patterns can produce.
    remove_duplicate_names(&mut found);
    Ok(found)
}
