//! Finding and sourcing files along 'runtimepath' -- `:runtime` and everything
//! built on it.
//!
//! [`do_in_path_visit`] is the primitive: split a path list on commas, glob
//! each entry against a pattern, and hand every match to a closure, optionally
//! stopping at the first.  `do_in_path_and_pp` adds 'packpath''s
//! `pack/*/start` and `pack/*/opt` trees for the `RuntimeOpts::START`/`RuntimeOpts::OPT` flags,
//! and [`do_in_runtimepath_visit`] is the 'runtimepath' entry point that
//! prefers the cached search path when there is one (see [`super::cache`]).
//! The `source_runtime*` wrappers pick the closure that sources what was
//! found, with the Vim-then-Lua ordering `:runtime` promises;
//! [`runtime_get_named`] and [`runtime_inspect`] are the API's read-only views
//! of the same search.
//!
//! The pattern walk itself lives in [`expand_name_patterns`], which the cached
//! path shares: the two searches differ in where the directory list comes from
//! and in one `ExpandFlags::NOBREAK`, not in what they do with `{name}`.
//!
//! Every name handed to a visit lives in the walk's own buffers for the call,
//! so a visit may run anything -- source a script that rewrites
//! 'runtimepath', say -- without the names moving under it. The name being
//! searched for is the caller's, borrowed for the whole walk: callers hand in
//! a copy of their own, never an option's value or a buffer's line.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::charset::skip;
use crate::cstr;
use crate::memory::XString;
use crate::message_fmt::msg_cstr;
use crate::option::split_option_part;
use crate::option::split_option_part_with;
use crate::option::vars::P_PP;
use crate::option::vars::P_RTP;
use crate::os::fs::dir_exists;
use crate::path::{ExpandFlags, add_pathsep};
use crate::semsg;
use crate::smsg;

use crate::types::{ExpandContext, FAIL, Failed, OK};
use core::ffi::{CStr, c_int};
use std::sync::PoisonError;

/// What a search does with what it finds: handed each batch of names and
/// whether the search wants every match, it answers whether it used one.
/// The names live in the search's own buffers for the call.
pub(crate) type Visit<'v> = dyn FnMut(&[&CStr], bool) -> bool + 'v;

/// The `[where]` qualifiers `:runtime` accepts, and the `DIP_*` set each one
/// selects.  Upstream sums the flags; they are disjoint bits, so this is the
/// same number.
const WHERE_FLAGS: [(&CStr, RuntimeOpts); 4] = [
    (c"START", RuntimeOpts::START.or(RuntimeOpts::NORTP)),
    (c"OPT", RuntimeOpts::OPT.or(RuntimeOpts::NORTP)),
    (
        c"PACK",
        RuntimeOpts::START
            .or(RuntimeOpts::OPT)
            .or(RuntimeOpts::NORTP),
    ),
    (c"ALL", RuntimeOpts::START.or(RuntimeOpts::OPT)),
];

/// What the `[where]` word, the first `where_len` bytes of `arg`, names, and
/// where the name after it starts -- 0 when it names nothing.
///
/// The comparison is `strncmp` over `where_len` bytes, so a *prefix* of a
/// qualifier selects it: `:runtime STA foo` really does mean `START`.
fn runtime_cmd_flags(arg: &[u8], where_len: usize) -> (RuntimeOpts, usize) {
    if where_len == 0 {
        return (RuntimeOpts::NONE, 0);
    }
    for (keyword, flags) in WHERE_FLAGS {
        // `strncmp`: the word may abbreviate the keyword, not run past it.
        let keyword = keyword.to_bytes();
        if where_len <= keyword.len() && arg[..where_len] == keyword[..where_len] {
            return (flags, where_len + skip::white(&arg[where_len..]));
        }
    }
    (RuntimeOpts::NONE, 0)
}

/// `:runtime[!] [where] {name}`.
pub fn ex_runtime(excmd: &mut ExArg) {
    let mut flags = if excmd.forceit {
        RuntimeOpts::ALL
    } else {
        RuntimeOpts::NONE
    };
    let arg = excmd.line.arg();
    let (where_flags, skip) = runtime_cmd_flags(arg, skip::to_white(arg));
    flags |= where_flags;
    let _ = source_runtime(excmd.line.cstr_from(excmd.line.arg + skip), flags);
}

/// Set the completion context for the `:runtime` command, whose argument
/// starts at `arg` in the completion's line.
///
/// The `[where]` qualifier is only offered for a single-argument command line;
/// past the first argument [`RUNTIME_EXPAND_FLAGS`] is forced non-zero so
/// [`runtime_cmd_matches`] stops proposing the qualifiers.
pub fn set_context_in_runtime_cmd(expand: &mut Expand, arg: usize) {
    let line = expand.line_cstr().to_bytes();
    let rest = |at: usize| line.get(at..).unwrap_or_default();
    let mut arg = arg;
    let first_word = skip::to_white(rest(arg));
    let mut flags = RuntimeOpts::NONE;
    if arg + first_word < line.len() {
        let (where_flags, skip) = runtime_cmd_flags(rest(arg), first_word);
        flags = where_flags;
        arg += skip;
    }
    // Skip to the last argument.
    loop {
        let p = arg + skip::to_white_esc(rest(arg));
        if p >= line.len() {
            break;
        }
        if flags == RuntimeOpts::NONE {
            // With multiple arguments and no [where], an unrelated non-zero
            // flag keeps [where] out of the completion.
            flags = RuntimeOpts::ALL;
        }
        arg = p + skip::white(rest(p));
    }
    RUNTIME_EXPAND_FLAGS.set(flags);
    expand.context = ExpandContext::Runtime;
    expand.pattern = arg;
}

/// Source every name `accept` picks out, stopping after the first unless
/// `all`.  Answers whether anything was sourced.
///
/// `ret_sid` is [`do_source`]'s: when set, a script already loaded is not
/// sourced again.
fn source_matching(
    fnames: &[&CStr],
    all: bool,
    ret_sid: &mut Option<&mut c_int>,
    accept: impl Fn(&CStr) -> bool,
) -> bool {
    let mut did_one = false;
    for &fname in fnames {
        if !accept(fname) {
            continue;
        }
        do_source(fname, false, DOSO_NONE, ret_sid.as_deref_mut());
        did_one = true;
        if !all {
            break;
        }
    }
    did_one
}

/// Source all `.vim` and `.lua` files in `fnames`, `.vim` files first.
pub(crate) fn source_found_vim_lua(
    fnames: &[&CStr],
    all: bool,
    mut ret_sid: Option<&mut c_int>,
) -> bool {
    let did_one = source_matching(fnames, all, &mut ret_sid, |f| {
        path_with_extension(f, c"vim")
    });
    if !all && did_one {
        return true;
    }
    did_one
        | source_matching(fnames, all, &mut ret_sid, |f| {
            path_with_extension(f, c"lua")
        })
}

/// Source all files in `fnames`: `.vim` first, then `.lua`, then the rest.
pub(crate) fn source_found(fnames: &[&CStr], all: bool, mut ret_sid: Option<&mut c_int>) -> bool {
    let did_one = source_found_vim_lua(fnames, all, ret_sid.as_deref_mut());
    if !all && did_one {
        return true;
    }
    did_one
        | source_matching(fnames, all, &mut ret_sid, |f| {
            !path_with_extension(f, c"vim") && !path_with_extension(f, c"lua")
        })
}

/// The [`ExpandFlags`] a `DIP_*` set asks a wildcard expansion for.
pub(crate) fn wildcard_flags(flags: RuntimeOpts) -> ExpandFlags {
    (if flags.has(RuntimeOpts::DIR) {
        ExpandFlags::DIR
    } else {
        ExpandFlags::FILE
    }) | (if flags.has(RuntimeOpts::DIRFILE) {
        ExpandFlags::DIR | ExpandFlags::FILE
    } else {
        ExpandFlags::NONE
    })
}

/// Whether `flags` asks for this entry to be skipped for being — or for not
/// being — an `after/` directory.
pub(crate) fn skips_entry(flags: RuntimeOpts, is_after: bool) -> bool {
    flags.has(RuntimeOpts::NOAFTER | RuntimeOpts::AFTER)
        && (is_after && flags.has(RuntimeOpts::NOAFTER)
            || !is_after && flags.has(RuntimeOpts::AFTER))
}

/// Expand each whitespace-separated pattern of `name` in turn after the
/// directory in `buf`, and hand `visit` the matches.
///
/// `buf` holds one directory, path separator and prefix included; the
/// patterns are appended there one after another, each cut to the room
/// upstream's `MAXPATHL` buffer had left, and `buf` is put back as it came.
/// Stops at the first pattern that matched unless `do_all`, and folds what it
/// finds into the caller's running `did_one` — which is also what its own
/// loop tests, so a match found for an earlier directory ends the walk here
/// too. A match counts whatever `visit` answers, as upstream.
pub(crate) fn expand_name_patterns(
    buf: &mut Vec<u8>,
    name: &[u8],
    ew_flags: ExpandFlags,
    do_all: bool,
    did_one: &mut bool,
    visit: &mut Visit,
) {
    let tail = buf.len();
    debug_assert!(tail <= PATH_ROOM, "MAXPATHL >= tail - buf");
    let mut rest = name;
    while !rest.is_empty() && (do_all || !*did_one) {
        buf.truncate(tail);
        rest = split_option_part_with(rest, PATH_ROOM - tail, b"\t ", |run| {
            buf.extend_from_slice(run);
        });
        buf.push(0);
        let pattern = cstr::in_bytes(buf);
        if p_verbose() > 10 {
            verbose_enter();
            let pattern = msg_cstr(pattern);
            smsg!(0, "Searching for \"{pattern}\"");
            verbose_leave();
        }
        *did_one |= expand_and_visit(pattern, ew_flags, do_all, visit).is_ok();
    }
    buf.truncate(tail);
}

/// The `p_verbose > 10` announcement `do_in_path` makes before it starts.
fn announce_search(name: &CStr, prefix: &CStr, path: &CStr) {
    verbose_enter();
    let (name, path) = (msg_cstr(name), msg_cstr(path));
    if prefix.is_empty() {
        smsg!(0, "Searching for \"{name}\" in \"{path}\"");
    } else {
        let prefix = msg_cstr(prefix);
        smsg!(
            0,
            "Searching for \"{name}\" under \"{prefix}\" in \"{path}\""
        );
    }
    verbose_leave();
}

/// Find the patterns in `name` in all directories in `path` and hand `visit`
/// each match.  `prefix` is prepended to each pattern; with `name` `None`
/// `visit` is handed every directory instead.
///
/// `RuntimeOpts::ALL` visits every match rather than stopping at the first,
/// `RuntimeOpts::DIR` looks for directories, and `RuntimeOpts::ERR` turns
/// "nothing found" into an error message rather than a verbose note.
///
/// Answers OK when something was found, FAIL otherwise.
///
/// `path` is the list to walk, which the caller has already copied out of
/// whichever option it came from -- a visit may set that option -- and
/// `path_name` is the option it names, for the "not found" message.
/// Upstream recovered that name by comparing `path` against `p_rtp`'s
/// *address*, which a copied list cannot answer.
pub(crate) fn do_in_path_visit(
    path: XString,
    path_name: &'static CStr,
    prefix: &CStr,
    name: Option<&CStr>,
    flags: RuntimeOpts,
    visit: &mut Visit,
) -> c_int {
    if p_verbose() > 10
        && let Some(name) = name
    {
        announce_search(name, prefix, path.as_cstr());
    }

    let do_all = flags.has(RuntimeOpts::ALL);
    let mut did_one = false;
    let mut rest: &[u8] = &path;
    let mut buf = Vec::with_capacity(PATH_ROOM);
    while !rest.is_empty() && (do_all || !did_one) {
        let (dir, after) = split_option_part(rest, PATH_ROOM, b",");
        rest = after;
        if skips_entry(flags, path_is_after(&dir)) {
            continue;
        }
        let Some(name) = name else {
            cstr::with_terminated(&dir, |dir| visit(&[dir], do_all));
            did_one = true;
            continue;
        };
        if dir.len() + 2 + prefix.count_bytes() + name.count_bytes() >= PATH_ROOM {
            continue;
        }
        buf.clear();
        buf.extend_from_slice(&dir);
        // The length test above proves the separator and the prefix fit.
        add_pathsep(&mut buf);
        buf.extend_from_slice(prefix.to_bytes());
        expand_name_patterns(
            &mut buf,
            name.to_bytes(),
            wildcard_flags(flags),
            do_all,
            &mut did_one,
            visit,
        );
    }

    if !did_one && let Some(name) = name {
        let (basepath, name) = (msg_cstr(path_name), msg_cstr(name));
        if flags.has(RuntimeOpts::ERR) {
            semsg!("E919: Directory not found in '{basepath}': \"{name}\"");
        } else if p_verbose() > 1 {
            verbose_enter();
            smsg!(0, "not found in '{basepath}': \"{name}\"");
            verbose_leave();
        }
    }

    if did_one { OK } else { FAIL }
}

fn boolean_obj(value: bool) -> Object {
    Object::Boolean(value)
}

pub(crate) fn integer_obj(value: Integer) -> Object {
    Object::Integer(value)
}

fn string_obj(string: String_0) -> Object {
    Object::string(string)
}

fn dict_obj(dict: ApiDict) -> Object {
    Object::dict(dict)
}

/// `nvim__runtime_inspect()`: the cached search path as it stands.
///
/// Note that this reads the cache without validating it — see
/// [`super::cache`], and the trap that in `nvim -l` script mode nothing else
/// rebuilds it either.
///
/// The strings are copies: the answer owns its whole tree and outlives the
/// cache it was read out of.
pub fn runtime_inspect() -> Array {
    let path = runtime_search_path_as_is();
    let mut rv = Array::with_capacity(path.len());
    for item in path.iter() {
        let mut entry = ApiDict::with_capacity(5);
        entry.insert(
            c"path",
            string_obj(String_0::from_cstr(item.path.as_cstr())),
        );
        if item.after {
            entry.insert(c"after", boolean_obj(true));
        }
        if item.pack_inserted {
            entry.insert(c"pack_inserted", boolean_obj(true));
        }
        if let Some(has_lua) = item.has_lua.get() {
            entry.insert(c"has_lua", boolean_obj(has_lua));
        }
        let pos_in_rtp = Integer::try_from(item.pos_in_rtp).unwrap_or(Integer::MAX);
        entry.insert(c"pos_in_rtp", integer_obj(pos_in_rtp));
        rv.push(dict_obj(entry));
    }
    rv
}

/// `nvim__get_runtime()`: the readable files named by `pat` along the cached
/// search path.
pub fn runtime_get_named(lua: bool, pat: &Array, all: bool) -> Array {
    let path = runtime_search_path_get_cached();
    runtime_get_named_common(lua, pat, all, &path)
}

/// [`runtime_get_named`] for a worker thread, against the snapshot
/// [`update_runtime_search_path_thread`] keeps for exactly this.
pub fn runtime_get_named_thread(lua: bool, pat: &Array, all: bool) -> Array {
    // TODO(bfredl): avoid contention between multiple worker threads?
    let path = THREAD_SEARCH_PATH
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    runtime_get_named_common(lua, pat, all, &path)
}

/// Whether this search-path entry has a `lua/` subdirectory.
///
/// The answer is cached in the entry, which is why it is a `Cell`: the
/// entry belongs to the shared search path (or to the thread snapshot), not
/// to this call.
fn dir_has_lua(item: &SearchPathItem, buf: &mut Vec<u8>) -> bool {
    if item.has_lua.get().is_none() {
        buf.clear();
        buf.extend_from_slice(&item.path);
        buf.extend_from_slice(b"/lua/");
        let fits = buf.len() < PATH_ROOM;
        buf.push(0);
        item.has_lua
            .set(Some(fits && dir_exists(cstr::in_bytes(buf))));
    }
    item.has_lua.get() != Some(false)
}

/// The shared body of [`runtime_get_named`] and its thread variant.
fn runtime_get_named_common(lua: bool, pat: &Array, all: bool, path: &[SearchPathItem]) -> Array {
    let mut rv = Array::with_capacity(path.len().wrapping_mul(pat.len()));
    let mut buf = Vec::with_capacity(PATH_ROOM);
    for item in path {
        if lua && !dir_has_lua(item, &mut buf) {
            continue;
        }
        for pat_item in pat {
            let Object::String(pattern) = pat_item else {
                continue;
            };
            buf.clear();
            buf.extend_from_slice(&item.path);
            buf.push(b'/');
            // `%s`: the pattern as far as its first NUL.
            buf.extend_from_slice(cstr::until_nul(pattern.as_bytes()));
            if buf.len() >= PATH_ROOM {
                continue;
            }
            buf.push(0);
            let found = cstr::in_bytes(&buf);
            if !os_file_is_readable(found) {
                continue;
            }
            rv.push(string_obj(String_0::from_cstr(found)));
            if !all {
                return rv;
            }
        }
    }
    rv
}

/// Find `name` in `path`, and then — for `RuntimeOpts::START`/`RuntimeOpts::OPT` — in
/// 'packpath''s package trees, handing `visit` each match.
///
/// Answers OK when at least one match was found.  With `name` empty `visit`
/// is handed each 'runtimepath' directory instead.
fn do_in_path_and_pp(path: XString, name: &CStr, flags: RuntimeOpts, visit: &mut Visit) -> c_int {
    let mut done = FAIL;
    // Each round is skipped once something has been found, unless RuntimeOpts::ALL.
    let wants_more = |done: c_int| done == FAIL || flags.has(RuntimeOpts::ALL);

    if !flags.has(RuntimeOpts::NORTP) {
        // An empty `name` means "every directory".
        let dirs_only = (!name.is_empty()).then_some(name);
        done |= do_in_path_visit(path, c"runtimepath", c"", dirs_only, flags, visit);
    }

    if wants_more(done) && flags.has(RuntimeOpts::START) {
        let after = flags.has(RuntimeOpts::AFTER);
        // The `after/` variants are searched under the package, so `AFTER`
        // is spent here and must not filter the packpath entries as well.
        let start_flags = flags.without(RuntimeOpts::AFTER);
        for prefix in [
            if after {
                c"pack/*/start/*/after/"
            } else {
                c"pack/*/start/*/"
            },
            if after {
                c"start/*/after/"
            } else {
                c"start/*/"
            },
        ] {
            done |= do_in_path_visit(
                P_PP.get(),
                c"packpath",
                prefix,
                Some(name),
                start_flags,
                visit,
            );
            if !wants_more(done) {
                break;
            }
        }
    }

    if wants_more(done) && flags.has(RuntimeOpts::OPT) {
        for prefix in [c"pack/*/opt/*/", c"opt/*/"] {
            done |= do_in_path_visit(P_PP.get(), c"packpath", prefix, Some(name), flags, visit);
            if !wants_more(done) {
                break;
            }
        }
    }

    done
}

/// Find `name` along 'runtimepath' -- the cached search path, then the
/// package trees `RuntimeOpts::START`/`RuntimeOpts::OPT` ask for -- and hand
/// `visit` each match. `name` may contain wildcards; an empty `name` hands
/// over the directories themselves.
pub(crate) fn do_in_runtimepath_visit(
    name: &CStr,
    mut flags: RuntimeOpts,
    visit: &mut Visit,
) -> Result<(), Failed> {
    let mut success = FAIL;
    if !flags.has(RuntimeOpts::NORTP) {
        let dirs_only = (!name.is_empty()).then_some(name);
        success |= do_in_cached_path(dirs_only, flags, visit);
        // The cached path already covers 'runtimepath' and the `start`
        // packages spliced into it.
        flags = flags.without(RuntimeOpts::START) | RuntimeOpts::NORTP;
    }
    // TODO(bfredl): we could integrate disabled OPT dirs into the cached path,
    // which would make ":packadd myoptpack" effective as well.
    if flags.has(RuntimeOpts::START | RuntimeOpts::OPT)
        && (success == FAIL || flags.has(RuntimeOpts::ALL))
    {
        success |= do_in_path_and_pp(P_RTP.get(), name, flags, visit);
    }
    if success == FAIL { Err(Failed) } else { Ok(()) }
}

/// Source the file `name` from all directories in 'runtimepath'.  `name` may
/// contain wildcards; `RuntimeOpts::ALL` sources every match rather than the first.
pub fn source_runtime(name: &CStr, flags: RuntimeOpts) -> Result<(), Failed> {
    do_in_runtimepath_visit(name, flags, &mut |found, all| {
        source_found(found, all, None)
    })
}

/// [`source_runtime`], but only `.vim` and `.lua` files.
pub(crate) fn source_runtime_vim_lua(name: &CStr, flags: RuntimeOpts) -> Result<(), Failed> {
    do_in_runtimepath_visit(name, flags, &mut |found, all| {
        source_found_vim_lua(found, all, None)
    })
}

/// [`source_runtime`] over `path` instead of 'runtimepath', and only `.vim`
/// and `.lua` files.
pub(crate) fn source_in_path_vim_lua(path: XString, name: &CStr, flags: RuntimeOpts) -> c_int {
    do_in_path_and_pp(path, name, flags, &mut |found, all| {
        source_found_vim_lua(found, all, None)
    })
}

/// Expand the wildcards in `pattern` and hand `visit` the matches.
///
/// Answers `Ok` when files were found, `Err` otherwise, whatever `visit`
/// makes of them.  `all` is passed through to `visit`, which decides whether
/// to act on more than the first. The matches are this frame's, alive across
/// whatever `visit` runs.
pub(crate) fn expand_and_visit(
    pattern: &CStr,
    flags: ExpandFlags,
    all: bool,
    visit: &mut Visit,
) -> Result<(), Failed> {
    let found = gen_expand_wildcards(&[pattern], flags)?;
    let names: Vec<&CStr> = found.iter().map(XString::as_cstr).collect();
    visit(&names, all);
    Ok(())
}
