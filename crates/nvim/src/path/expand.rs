//! Expanding a list of patterns, the way a command line means them.
//!
//! [`gen_expand_wildcards`] is the entry point for `:edit`, `expand()` and
//! command-line completion: it expands environment variables, hands
//! backticked patterns to the shell ([`expand_backtick`]), searches `'path'`
//! when the caller asked for that ([`expand_path_option`]), and otherwise
//! falls through to the file-system walk in [`glob`](super::glob). The
//! [`ExpandFlags`] say which of those apply.
//!
//! Every expansion answers the names it found as strings the caller owns.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::charset::backslash_halved;
use crate::cmdexpand::WildOpts;
use crate::cstr;
use crate::ex_docmd::Expansion;
use crate::memory::XString;
use crate::option::split_option_part;
use crate::option::vars::{P_CDPATH, P_PATH, P_WIG};
use crate::optionstr::{OptString, local_or_global};
use crate::os::env::expand_env_save_opt_of;
use crate::os::fs::dirname_text;
use crate::winlayer::Buf;
use core::ffi::c_int;
use std::ffi::CStr;

use super::*;
use crate::guard::Suppress;
use crate::os::shell::ShellOpts;
use crate::types::{Failed, MAXPATHL};

/// Whether a `gen_expand_wildcards` is already running. The pieces it calls
/// can come back round to it — a `=expr` item evaluates an expression that
/// starts another expansion — and the inner call has to go straight to the
/// shell instead of recursing.
static RECURSIVE: GlobalCell<bool> = GlobalCell::new(false);

/// Expand every entry of `path_option` — a `'path'` or `'cdpath'` — into the
/// directory it names.
///
/// An entry of `"."` means the current buffer's directory, an empty entry
/// means `curdir`, and a relative entry is taken from `curdir`. A URL, or an
/// entry holding a backtick (which a modeline could have set), is dropped.
///
/// TODO(vim): handle upward search (`;`) and the path limiter (`**N`) by
/// expanding each into the paths it stands for.
pub(crate) fn expand_path_option(curdir: &[u8], path_option: &[u8]) -> Vec<XString> {
    let mut dirs = Vec::new();
    let mut rest = path_option;
    while !rest.is_empty() {
        let (mut buf, after) = split_option_part(rest, MAXPATHL as usize, b" ,");
        rest = after;

        // Do not expand backticks: this could have been set by a modeline.
        if buf.contains(&b'`') {
            continue;
        }

        if buf.first() == Some(&b'.') && buf.get(1).is_none_or(|&b| vim_ispathsep(c_int::from(b))) {
            // Relative to the current buffer:
            //     "/path/file" + "."        -> "/path/"
            //     "/path/file" + "./subdir" -> "/path/subdir"
            let Some(ffname) = Buf::current()
                .name
                .full()
                .map(|name| name.to_bytes().to_vec())
            else {
                continue;
            };
            let plen = tail_index(&ffname);
            if plen + buf.len() >= MAXPATHL as usize {
                continue;
            }
            let entry_rest = if buf.len() == 1 { &[][..] } else { &buf[2..] };
            let mut joined = ffname[..plen].to_vec();
            joined.extend_from_slice(entry_rest);
            joined.push(0);
            let len = simplify_filename(&mut joined);
            joined.truncate(len);
            buf = joined;
        } else if buf.is_empty() {
            // Relative to the current directory.
            buf = cstr::until_nul(curdir).to_vec();
        } else if path_with_url(&cstr::owned(&buf)) != 0 {
            continue; // a URL can't be used here
        } else if !path_is_absolute(&cstr::owned(&buf)) {
            // Expand a relative path to its full equivalent.
            let curdir = cstr::until_nul(curdir);
            // The directory, the separator, the entry, and the NUL.
            if curdir.len() + buf.len() + 3 > MAXPATHL as usize {
                continue;
            }
            let mut joined = curdir.to_vec();
            joined.push(PATHSEP_BYTE);
            joined.extend_from_slice(&buf);
            joined.push(0);
            let len = simplify_filename(&mut joined);
            joined.truncate(len);
            buf = joined;
        }
        dirs.push(XString::from_bytes(cstr::until_nul(&buf)));
    }
    dirs
}

/// Expand `pattern` under every directory the `'path'` (or, for
/// [`ExpandFlags::CDPATH`], the `'cdpath'`) names, adding the matches to
/// `names`.
///
/// Answers how many names `names` holds in total.
pub(crate) fn expand_in_path(
    names: &mut Vec<XString>,
    pattern: &CStr,
    flags: ExpandFlags,
) -> usize {
    let curdir = dirname_text();
    let path_option = if flags.has(ExpandFlags::CDPATH) {
        P_CDPATH.get()
    } else {
        buffer_path()
    };
    let path_dirs = expand_path_option(&curdir, &path_option);
    if path_dirs.is_empty() {
        return 0;
    }
    let mut paths = Vec::new();
    for (i, dir) in path_dirs.iter().enumerate() {
        if i > 0 {
            paths.push(b',');
        }
        paths.extend_from_slice(dir);
    }

    let mut glob_flags = WildOpts::NONE;
    if flags.has(ExpandFlags::ICASE) {
        glob_flags |= WildOpts::ICASE;
    }
    if flags.has(ExpandFlags::ADDSLASH) {
        glob_flags |= WildOpts::ADD_SLASH;
    }
    names.extend(globpath(
        &cstr::owned(&paths),
        pattern,
        glob_flags,
        flags.has(ExpandFlags::CDPATH),
    ));
    names.len()
}

/// The `'path'` in force: the buffer's own, or the global one when it is
/// empty. A copy: a value that outlives the next option write.
pub(crate) fn buffer_path() -> XString {
    local_or_global(&Buf::current().b_p_path, P_PATH).get()
}

/// Does `p` hold what looks like an environment variable? A backslash
/// escapes the character after it.
fn has_env_var(p: &[u8]) -> bool {
    let mut at = 0;
    while at < p.len() {
        if p[at] == b'\\' && at + 1 < p.len() {
            at += 2;
        } else if p[at] == b'$' {
            return true;
        } else {
            at += 1;
        }
    }
    false
}

/// Either of the two flags that send the expansion looking down a search
/// list rather than the pattern's own directory.
const SEARCH_LIST: ExpandFlags = ExpandFlags::PATH.or(ExpandFlags::CDPATH);

/// Should `p` be looked for along the `'path'` rather than from here?
///
/// Only when the caller asked for a path search at all, and the pattern is
/// not one that already says where to look: an absolute name, or one
/// starting `"./"` or `"../"`.
fn wants_path_search(p: &CStr, flags: ExpandFlags) -> bool {
    if !flags.has(SEARCH_LIST) || path_is_absolute(p) {
        return false;
    }
    let p = p.to_bytes();
    let at = |i: usize| cstr::byte_at(p, i);
    let here = at(0) == b'.'
        && (vim_ispathsep(c_int::from(at(1)))
            || at(1) == b'.' && vim_ispathsep(c_int::from(at(2))));
    !here
}

/// Expand `patterns` into the names they match.
///
/// A character that should not be expanded must have a backslash before it,
/// as in `"/path\\ with\\ spaces/my\\*star*"`. `flags` is the set
/// [`expand_wildcards`] documents.
///
/// Answers `Err` when no names were found, unless
/// [`ExpandFlags::EMPTYOK`] asks for an empty answer instead.
///
/// The patterns are the caller's own: nothing here frees them, so a pattern
/// borrowed from a value that user code can replace -- an option, a buffer
/// line -- must be copied first. Backticks and `os_breakcheck` can run user
/// code mid-expansion.
pub fn gen_expand_wildcards(
    patterns: &[&CStr],
    flags: ExpandFlags,
) -> Result<Vec<XString>, Failed> {
    // A recursive call can happen when a `=expr` item evaluates an
    // expression that starts another expansion; go straight to the
    // machine-specific expansion in that case.
    if RECURSIVE.get() {
        return expand_by_shell(patterns, flags);
    }

    // One pattern the shell has to handle sends them all to the shell,
    // so it is started once rather than once per pattern. A `=expr` is
    // ours to evaluate, though.
    if patterns.iter().any(|p| {
        let p = p.to_bytes();
        has_special_wildchar(p, flags) && !(vim_backtick(p) && p[1] == b'=')
    }) {
        return expand_by_shell(patterns, flags);
    }

    RECURSIVE.set(true);
    // Allocated with the first name, as upstream's array is: most runtime
    // searches find nothing.
    let mut names = Vec::new();

    // Upstream never clears this: once one pattern has been looked for
    // along the 'path', every later one is uniquefied too.
    let mut did_expand_in_path = false;
    for pattern in patterns {
        if got_int.get() {
            break;
        }
        // How many names this pattern added, or `None` when nothing tried.
        let mut added = None;
        // The pattern with its environment variables expanded, when it
        // has any.
        let mut expanded: Option<XString> = None;

        if vim_backtick(pattern.to_bytes()) {
            added = expand_backtick(&mut names, pattern.to_bytes(), flags);
            if added.is_none() {
                RECURSIVE.set(false);
                return Err(Failed);
            }
        } else {
            // Environment variables, "~/" and "~user/" first.
            let bytes = pattern.to_bytes();
            if has_env_var(bytes) && !flags.has(ExpandFlags::NOTENV) || bytes.first() == Some(&b'~')
            {
                let env = expand_env_save_opt_of(pattern, true);
                if has_env_var(&env) || env.first() == Some(&b'~') {
                    // What `expand_env` could not expand, the shell can.
                    // Throw away what has been found so far and start over.
                    drop(names);
                    let ret = expand_by_shell(patterns, flags | ExpandFlags::KEEPDOLLAR);
                    RECURSIVE.set(false);
                    return ret;
                }
                expanded = Some(env);
            }
            let p = expanded.as_ref().map_or(*pattern, XString::as_cstr);

            if has_exp_wildcard(p.to_bytes()) || flags.has(ExpandFlags::ICASE) {
                // A recursive `gen_expand_wildcards` can only happen from
                // an event handler in `os_breakcheck`, where it is fine.
                RECURSIVE.set(false);
                if wants_path_search(p, flags) {
                    // `:find` completion, where 'path' is used.
                    added = Some(expand_in_path(&mut names, p, flags));
                    did_expand_in_path = true;
                } else {
                    added = Some(path_expand(&mut names, p.to_bytes(), flags));
                }
                RECURSIVE.set(true);
            }
        }
        let p = expanded.as_ref().map_or(*pattern, XString::as_cstr);

        if added.is_none() || added == Some(0) && flags.has(ExpandFlags::NOTFOUND) {
            let t = XString::from_vec(backslash_halved(p.to_bytes()));
            let t = t.as_cstr();
            // With ExpandFlags::NOTFOUND always add files and directories:
            // that is what makes "vim c:/" work.
            if flags.has(ExpandFlags::NOTFOUND) {
                addfile(&mut names, t, flags | ExpandFlags::DIR | ExpandFlags::FILE);
            } else {
                addfile(&mut names, t, flags);
            }
        }

        if did_expand_in_path && !names.is_empty() && flags.has(SEARCH_LIST) {
            RECURSIVE.set(false);
            // Upstream takes 'path' once, up front, whether or not it will
            // be used -- which during `set_init_1` (expanding a `~` in
            // $CDPATH) is before 'path' has a value. Taken here, it is read
            // only where a 'path' search has already happened.
            let path_option = buffer_path();
            uniquefy_paths(&mut names, p, &path_option);
            RECURSIVE.set(true);
        }
    }

    RECURSIVE.set(false);
    if flags.has(ExpandFlags::EMPTYOK) || !names.is_empty() {
        Ok(names)
    } else {
        Err(Failed)
    }
}

/// [`os_expand_wildcards`] of the caller's patterns.
fn expand_by_shell(patterns: &[&CStr], flags: ExpandFlags) -> Result<Vec<XString>, Failed> {
    let patterns: Vec<XString> = patterns.iter().copied().map(XString::from_cstr).collect();
    os_expand_wildcards(&patterns, flags)
}

/// Is `p` a whole backticked command, `` `cmd` ``?
fn vim_backtick(p: &[u8]) -> bool {
    p.len() > 1 && p[0] == b'`' && p[p.len() - 1] == b'`'
}

/// Run the command in `` `pat` `` and add each line it prints to `names`,
/// or evaluate it as an expression when it starts with `=`.
///
/// Answers how many names were added, or `None` when the command failed.
fn expand_backtick(names: &mut Vec<XString>, pat: &[u8], flags: ExpandFlags) -> Option<usize> {
    // The command is the pattern without its backticks.
    let cmd = &pat[1..pat.len() - 1];
    let buffer = if let Some(expr) = cmd.strip_prefix(b"=") {
        // `={expr}`: expand an expression.
        eval_to_string(expr, true, false)
    } else {
        let opts = if flags.has(ExpandFlags::SILENT) {
            ShellOpts::SILENT
        } else {
            ShellOpts::NONE
        };
        get_cmd_output(&cstr::owned(cmd), opts)
    }?;

    // One name per line, and a line with nothing but white space on it
    // is not a name.
    let text = cstr::until_nul(&buffer);
    let mut count = 0;
    let mut at = 0;
    while at < text.len() {
        while matches!(text.get(at), Some(b' ' | b'\t')) {
            at += 1;
        }
        let start = at;
        while at < text.len() && text[at] != b'\r' && text[at] != b'\n' {
            at += 1;
        }
        if at > start {
            addfile(names, &cstr::owned(&text[start..at]), flags);
            count += 1;
        }
        while matches!(text.get(at), Some(b'\r' | b'\n')) {
            at += 1;
        }
    }
    Some(count)
}

/// [`expand_wildcards`] for one pattern that may start with `%`, `#` or
/// `<cword>` and friends, which are evaluated first.
///
/// A `%` or `#` that names no existing file still answers the name, without
/// the `*` that followed it, so that this works for remote files and for
/// buffers that are not files at all.
pub fn expand_wildcards_eval(pat: &CStr, flags: ExpandFlags) -> Result<Vec<XString>, Failed> {
    let bytes = pat.to_bytes();
    let is_cur_alt_file = matches!(bytes.first(), Some(b'%' | b'#'));
    let mut evaluated: Option<(Vec<u8>, bool)> = None;
    let mut exp_pat = XString::from_cstr(pat);

    if is_cur_alt_file || bytes.first() == Some(&b'<') {
        let no_emsg = Suppress::emsg();
        let expansion = eval_vars(pat, 0, None, true);
        drop(no_emsg);
        if let Expansion::Text { text, used, .. } = expansion {
            let text = cstr::until_nul(&text).to_vec();
            let rest = &bytes[used..];
            let star_follows = rest == b"*";
            exp_pat = XString::from_bytes(&text);
            exp_pat.push_bytes(rest);
            evaluated = Some((text, star_follows));
        }
    }

    let ret = expand_wildcards(&[exp_pat.as_cstr()], flags);
    match evaluated {
        // Nothing found for "%*" or "#*": the name itself, without the star.
        Some((text, true))
            if is_cur_alt_file && !ret.as_ref().is_ok_and(|names| !names.is_empty()) =>
        {
            Ok(vec![XString::from_vec(text)])
        }
        _ => ret,
    }
}

/// [`gen_expand_wildcards`], then drop the names `'wildignore'` matches and
/// move the ones `'suffixes'` matches to the end.
///
/// Answers `Err` when nothing is left.
pub fn expand_wildcards(patterns: &[&CStr], flags: ExpandFlags) -> Result<Vec<XString>, Failed> {
    let mut names = gen_expand_wildcards(patterns, flags)?;
    if flags.has(ExpandFlags::KEEPALL) {
        return Ok(names);
    }

    // Remove the names that match 'wildignore'.
    if P_WIG.first_byte() != 0 {
        names.retain(|name| {
            let ffname = full_name_save(name.as_cstr(), false);
            !p_wig(|value| match_file_list(value, name.as_cstr(), ffname.as_cstr()))
        });
    }

    // Move the names where 'suffixes' match to the end. Skip it when
    // interrupted: the result probably won't be used.
    if names.len() > 1 && !got_int.get() {
        let (plain, suffixed): (Vec<XString>, Vec<XString>) =
            names.into_iter().partition(|name| !match_suffix(name));
        names = plain;
        names.extend(suffixed);
    }

    if names.is_empty() {
        return Err(Failed);
    }
    Ok(names)
}
