//! Match sources that run user code or walk the file system.
//!
//! `'shellcmd'` completion ([`expand_shellcmd`]) walks `$PATH`;
//! [`globpath`] walks a comma-separated directory list; and the
//! `custom,`/`customlist,`/Lua completion functions of `:command` are called
//! through [`expand_user_defined`], [`expand_user_list`] and [`expand_user_lua`].
//!
//! The user's functions are called with copies of what they are told -- the
//! pattern, the line, the cursor -- and the context they complete for is
//! never one they can reach: the command line's is moved out of its state for
//! the length of the call, and every other caller's is a local.

#![forbid(unsafe_code)]

use super::*;
use crate::eval::list::string_tv;
use crate::eval::typval::list_iter;
use crate::eval::{call_func_retlist, call_func_retstr};
use crate::fuzzy::{FUZZY_SCORE_NONE, fuzzy_match_str};
use crate::lua::executor::nlua_call_user_expand_func;
use crate::option::next_option_part;
use crate::os::env::vim_getenv_owned;
use crate::path::{ExpandFlags, expand_wildcards_one, path_is_absolute, vim_ispathsep};
use crate::regexp::vim_regexec;
use crate::runtime::state::current_sctx;
use crate::types::{ExpandContext, Failed, MAXPATHL, RegMatch, TypVal, VarNumber};
use core::ffi::CStr;
use std::collections::HashSet;
use std::ffi::CString;

/// `name` as a C string; a NUL inside ends it, as it would a C caller's.
fn c_string(name: &[u8]) -> CString {
    let end = name.iter().position(|&c| c == 0).unwrap_or(name.len());
    CString::new(&name[..end]).expect("cut at the first NUL")
}

/// Expand shell command matches in one directory of `$PATH`.
///
/// `pathed` is the fully pathed pattern and `pathlen` the length of its path
/// portion (0 if there is no path).  New names, without the path, are
/// appended to `found` and remembered in `seen` so a later directory cannot
/// offer them again.
fn expand_shellcmd_onedir(
    pathed: &CStr,
    pathlen: usize,
    flags: ExpandFlags,
    seen: &mut HashSet<Vec<u8>>,
    found: &mut Vec<XString>,
) {
    let Ok(names) = expand_wildcards_one(pathed, flags) else {
        return;
    };
    for name in names {
        if name.len() > pathlen {
            // Remove the path that was prepended.
            let tail = &name[pathlen..];
            if seen.insert(tail.to_vec()) {
                found.push(owned(tail));
            }
        }
    }
}

/// Complete a shell command.
///
/// `filepat` is a pattern to match with command names; `flagsarg` is the
/// caller's [`ExpandFlags`] set.
pub(crate) fn expand_shellcmd(filepat: &CStr, flagsarg: ExpandFlags) -> Vec<XString> {
    let mut flags = flagsarg;
    let mut did_curdir = false;

    // For ":set path=" and ":set tags=" halve backslashes for escaped
    // space: replace "\ " with " ".
    let mut pat = filepat.to_bytes().to_vec();
    let mut s = 0;
    while s < pat.len() {
        if pat[s] == b'\\' && pat.get(s + 1) == Some(&b' ') {
            pat.remove(s);
        }
        s += 1;
    }

    flags |= ExpandFlags::FILE | ExpandFlags::EXEC | ExpandFlags::SHELLCMD;

    let at = |i: usize| pat.get(i).copied().unwrap_or(0);
    let path: Vec<u8> = if at(0) == b'.'
        && (vim_ispathsep(c_int::from(at(1)))
            || (at(1) == b'.' && vim_ispathsep(c_int::from(at(2)))))
    {
        b".".to_vec()
    } else if path_is_absolute(&c_string(&pat)) {
        // For an absolute name we don't use $PATH.
        Vec::new()
    } else {
        vim_getenv_owned(c"PATH")
            .map(|path| path.to_vec())
            .unwrap_or_default()
    };

    // Go over all directories in $PATH.  Expand matches in that directory
    // and collect them in `found`.  When "." is not in $PATH also expand for
    // the current directory, to find "subdir/cmd".
    let mut found = Vec::new();
    let mut seen = HashSet::new();
    let mut s = 0;
    loop {
        // Where this entry ends, and the length of the path portion of the
        // pathed pattern, including the trailing slash.
        let e;
        let pathlen;
        let seplen;

        if s >= path.len() {
            if did_curdir {
                break;
            }

            // Find directories in the current directory, path is empty.
            did_curdir = true;
            flags |= ExpandFlags::DIR;

            e = s;
            pathlen = 0;
            seplen = 0;
        } else {
            e = path[s..]
                .iter()
                .position(|&c| c == b':')
                .map_or(path.len(), |at| s + at);

            pathlen = e - s;
            if pathlen == 0 || &path[s..e] == b"." {
                did_curdir = true;
                flags |= ExpandFlags::DIR;
            } else {
                // Do not match directories inside a $PATH item.
                flags.clear(ExpandFlags::DIR);
            }

            // Upstream's `after_pathsep`: does the entry already end in one?
            seplen = usize::from(pathlen == 0 || path[e - 1] != b'/');
        }

        // Make sure that the pathed pattern (ie the path and pattern
        // concatenated together) will fit inside the buffer.  If not skip
        // it and move on to the next path.
        // Upstream's `+ 1 <= MAXPATHL` — the one byte is the NUL.
        if pathlen + seplen + pat.len() < MAXPATHL as usize {
            let mut pathed = path[s..s + pathlen].to_vec();
            if pathlen > 0 && seplen > 0 {
                pathed.push(b'/');
            }
            let pathlen = pathed.len();
            pathed.extend_from_slice(&pat);
            expand_shellcmd_onedir(&c_string(&pathed), pathlen, flags, &mut seen, &mut found);
        }

        s = if e < path.len() { e + 1 } else { e };
    }
    found
}

/// The pattern, the whole line and the cursor column, as the arguments a
/// user's completion function takes -- copies, so that nothing the function
/// does reaches back into the context.
fn user_expand_args(expand: &Expand) -> [TypVal; 3] {
    [
        string_tv(expand.pattern_span()),
        string_tv(expand.line_cstr().to_bytes()),
        TypVal::Number(VarNumber::from(expand.col)),
    ]
}

/// Call the user's Vimscript completion function through `call`, with the
/// script context it was defined in. `None` when there is no function to
/// call.
fn call_user_expand_func<R>(
    call: impl FnOnce(&CStr, &[TypVal]) -> Option<R>,
    expand: &Expand,
) -> Option<R> {
    let name = expand.arg.as_ref().filter(|name| !name.is_empty())?;
    let name = c_string(name);
    let args = user_expand_args(expand);

    let save_current_sctx = current_sctx.get();
    current_sctx.set(expand.script_ctx);
    let ret = call(&name, &args);
    current_sctx.set(save_current_sctx);
    ret
}

/// Expand names with a function defined by the user
/// (`ExpandContext::UserDefined`): one candidate per line of what it answers,
/// filtered by `regmatch` (or scored, under `'wildoptions'`=fuzzy).
pub(crate) fn expand_user_defined(
    pat: &CStr,
    expand: &Expand,
    regmatch: &mut RegMatch,
) -> Result<Vec<XString>, Failed> {
    let fuzzy = cmdline_fuzzy_complete(pat.to_bytes());
    let retstr = call_user_expand_func(call_func_retstr, expand).ok_or(Failed)?;

    // Exactly one of these fills: `fuzzy` is fixed for the whole call.
    let mut scored = Vec::new();
    let mut found = Vec::new();

    // The answer is one match per line; an empty line is a match like any
    // other, but a last newline does not start another one.
    let mut s = 0;
    while s < retstr.len() {
        let e = retstr[s..]
            .iter()
            .position(|&c| c == b'\n')
            .map_or(retstr.len(), |at| s + at);
        let line = &retstr[s..e];
        s = if e < retstr.len() { e + 1 } else { e };
        let candidate = c_string(line);

        let mut score = 0;
        let matched = if expand.pattern_is_empty() {
            true // match everything
        } else if fuzzy {
            score = fuzzy_match_str(&candidate, pat);
            score != FUZZY_SCORE_NONE
        } else {
            vim_regexec(regmatch, &candidate, 0)
        };

        if matched {
            let text = owned(line);
            if fuzzy {
                let idx = scored.len();
                scored.push(Scored { text, score, idx });
            } else {
                found.push(text);
            }
        }
    }

    Ok(if fuzzy {
        fuzzy_sorted(scored, false)
    } else {
        found
    })
}

/// The strings of a `customlist,` answer, the rest skipped.
fn user_list_strings(list: &TypVal) -> Vec<XString> {
    list_iter(list.list_ref())
        .filter_map(|li| li.li_tv.string_cstr())
        .map(XString::from_cstr)
        .collect()
}

/// Expand names with a list returned by a function defined by the user.
pub(crate) fn expand_user_list(expand: &Expand) -> Result<Vec<XString>, Failed> {
    let retlist = call_user_expand_func(call_func_retlist, expand).ok_or(Failed)?;
    Ok(user_list_strings(&retlist))
}

/// Expand names with a Lua completion function.
pub(crate) fn expand_user_lua(expand: &Expand) -> Result<Vec<XString>, Failed> {
    let rettv = nlua_call_user_expand_func(
        expand.luaref,
        &c_string(expand.pattern_text()),
        expand.line_cstr(),
        expand.col,
    );
    if !matches!(rettv, TypVal::List(_)) {
        return Err(Failed);
    }
    Ok(user_list_strings(&rettv))
}

/// Expand `file` for all comma-separated directories in `path`, answering
/// the matches.
///
/// If `dirs` is true only directory names are expanded.
pub fn globpath(path: &CStr, file: &CStr, expand_options: WildOpts, dirs: bool) -> Vec<XString> {
    let mut xpc = Expand::new();
    xpc.context = if dirs {
        ExpandContext::Directories
    } else {
        ExpandContext::Files
    };

    let file = file.to_bytes();
    let options = WildOpts::SILENT | expand_options;
    let mut found = Vec::new();
    let mut part = Vec::new();

    // Loop over all entries in {path}.
    let mut rest = path.to_bytes();
    while !rest.is_empty() {
        // Copy one item of the path and concatenate the file name.
        rest = next_option_part(rest, &mut part);
        // `copy_option_part`'s room, terminator included.
        part.truncate(MAXPATHL as usize - 1);
        let seplen = usize::from(!part.is_empty() && part.last() != Some(&b'/'));

        // Upstream's `+ 1 <= MAXPATHL` — the one byte is the NUL.
        if part.len() + seplen + file.len() < MAXPATHL as usize {
            if seplen > 0 {
                part.push(b'/');
            }
            part.extend_from_slice(file);
            let pattern = c_string(&part);

            let mut matches = expand_from_context(&mut xpc, &pattern, options).unwrap_or_default();
            if !matches.is_empty() {
                escape_matches(&mut xpc, pattern.to_bytes(), &mut matches, options);
                // Concatenate new results to previous ones.
                found.append(&mut matches);
            }
        }
    }

    found
}
