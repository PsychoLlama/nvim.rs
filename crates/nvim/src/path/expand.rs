//! Expanding a list of patterns, the way a command line means them.
//!
//! [`gen_expand_wildcards`] is the entry point for `:edit`, `expand()` and
//! command-line completion: it expands environment variables, hands
//! backticked patterns to the shell ([`expand_backtick`]), searches `'path'`
//! when the caller asked for that ([`expand_path_option`]), and otherwise
//! falls through to the file-system walk in [`glob`](super::glob). The `EW_*`
//! flags say which of those apply.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cmdexpand::WildOpts;
use crate::cstr;
use core::ffi::{c_char, c_int};
use std::ffi::CStr;

use super::*;
use crate::guard::Suppress;
use crate::os::shell::ShellOpts;
use crate::types::{Failed, MAXPATHL};

/// Whether a `gen_expand_wildcards` is already running. The pieces it calls
/// can come back round to it — `expand_env` falls back on `expand_one` — and
/// the inner call has to go straight to the shell instead of recursing.
static RECURSIVE: GlobalCell<bool> = GlobalCell::new(false);

/// Expand every entry of `path_option` — a `'path'` or `'cdpath'` — into the
/// directory it names, and add each to `gap`.
///
/// An entry of `"."` means the current buffer's directory, an empty entry
/// means `curdir`, and a relative entry is taken from `curdir`. A URL, or an
/// entry holding a backtick (which a modeline could have set), is dropped.
///
/// TODO(vim): handle upward search (`;`) and the path limiter (`**N`) by
/// expanding each into the paths it stands for.
///
/// # Safety
/// `curdir` and `path_option` must be NUL-terminated strings, and `gap` an
/// initialised array of allocated strings.
pub(crate) unsafe fn expand_path_option(
    curdir: *mut c_char,
    mut path_option: *mut c_char,
    gap: *mut garray_T,
) {
    let mut buf = vec![0 as c_char; MAXPATHL as usize];
    let buf = buf.as_mut_ptr();
    let mut curdirlen = 0;
    while unsafe { *path_option } != 0 {
        let mut buflen = unsafe {
            copy_option_part(
                &raw mut path_option,
                buf,
                MAXPATHL as size_t,
                c" ,".as_ptr().cast_mut(),
            )
        };

        // Do not expand backticks: this could have been set by a modeline.
        if !unsafe { vim_strchr(buf, c_int::from(b'`')) }.is_null() {
            continue;
        }

        if unsafe { *buf } == b'.' as c_char
            && (unsafe { *buf.add(1) } == 0 || vim_ispathsep(unsafe { *buf.add(1) } as c_int))
        {
            // Relative to the current buffer:
            //     "/path/file" + "."        -> "/path/"
            //     "/path/file" + "./subdir" -> "/path/subdir"
            let ffname = unsafe { (*curbuf.get()).b_ffname };
            if ffname.is_null() {
                continue;
            }
            // SAFETY: `path_tail` answers a pointer into `ffname`.
            let plen = unsafe { path_tail(ffname).offset_from(ffname) } as usize;
            if plen + buflen >= MAXPATHL as usize {
                continue;
            }
            if unsafe { *buf.add(1) } == 0 {
                unsafe { *buf.add(plen) = 0 };
            } else {
                // The entry past its ".", and the NUL with it.
                unsafe { core::ptr::copy(buf.add(2), buf.add(plen), buflen - 2 + 1) };
            }
            unsafe { core::ptr::copy(ffname, buf, plen) };
            buflen = unsafe { simplify_filename(buf) } as usize;
        } else if unsafe { *buf } == 0 {
            // Relative to the current directory.
            if curdirlen == 0 {
                curdirlen = unsafe { CStr::from_ptr(curdir) }.to_bytes().len();
            }
            unsafe { core::ptr::copy_nonoverlapping(curdir, buf, curdirlen + 1) };
            buflen = curdirlen;
        } else if unsafe { path_with_url(buf) } != 0 {
            continue; // a URL can't be used here
        } else if !unsafe { path_is_absolute(buf) } {
            // Expand a relative path to its full equivalent.
            if curdirlen == 0 {
                curdirlen = unsafe { CStr::from_ptr(curdir) }.to_bytes().len();
            }
            // The directory, the separator, the entry, and the NUL.
            if curdirlen + buflen + 3 > MAXPATHL as usize {
                continue;
            }
            unsafe { core::ptr::copy(buf, buf.add(curdirlen + 1), buflen + 1) };
            unsafe { core::ptr::copy_nonoverlapping(curdir, buf, curdirlen) };
            unsafe { *buf.add(curdirlen) = PATHSEP as c_char };
            buflen = unsafe { simplify_filename(buf) } as usize;
        }

        unsafe { ga_grow(gap, 1) };
        unsafe {
            *(*gap)
                .ga_data
                .cast::<*mut c_char>()
                .add((*gap).ga_len as usize) = xmemdupz(buf.cast(), buflen as size_t).cast()
        };
        unsafe { (*gap).ga_len += 1 };
    }
}

/// Expand `pattern` under every directory the `'path'` (or, for
/// [`ExpandFlags::CDPATH`], the `'cdpath'`) names, adding the matches to `gap`.
///
/// Answers how many names `gap` holds in total.
///
/// # Safety
/// `pattern` must be a NUL-terminated string and `gap` an initialised array
/// of allocated strings.
pub(crate) unsafe fn expand_in_path(
    gap: *mut garray_T,
    pattern: *mut c_char,
    flags: ExpandFlags,
) -> c_int {
    let mut curdir = vec![0 as c_char; MAXPATHL as usize];
    let _ = unsafe { os_dirname(curdir.as_mut_ptr(), MAXPATHL as size_t) };

    let mut path_ga = garray_T::default();
    unsafe { ga_init(&raw mut path_ga, size_of::<*mut c_char>() as c_int, 1) };
    let path_option = if flags.has(ExpandFlags::CDPATH) {
        p_cdpath.get()
    } else {
        unsafe { buffer_path() }
    };
    unsafe { expand_path_option(curdir.as_mut_ptr(), path_option, &raw mut path_ga) };
    if path_ga.ga_len <= 0 {
        return 0;
    }

    let paths = unsafe { ga_concat_strings(&raw mut path_ga, c",".as_ptr()) };
    unsafe { ga_clear_strings(&raw mut path_ga) };

    let mut glob_flags = WildOpts::NONE;
    if flags.has(ExpandFlags::ICASE) {
        glob_flags |= WildOpts::ICASE;
    }
    if flags.has(ExpandFlags::ADDSLASH) {
        glob_flags |= WildOpts::ADD_SLASH;
    }
    unsafe {
        globpath(
            paths,
            pattern,
            gap,
            glob_flags,
            flags.has(ExpandFlags::CDPATH),
        )
    };
    unsafe { xfree(paths.cast()) };

    unsafe { (*gap).ga_len }
}

/// The `'path'` in force: the buffer's own, or the global one when it is
/// empty.
///
/// # Safety
/// There must be a current buffer.
pub(crate) unsafe fn buffer_path() -> *mut c_char {
    if unsafe { *(*curbuf.get()).b_p_path } == 0 {
        p_path.get()
    } else {
        unsafe { (*curbuf.get()).b_p_path }
    }
}

/// Does `p` hold what looks like an environment variable? A backslash
/// escapes the character after it.
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub(crate) unsafe fn has_env_var(p: *mut c_char) -> bool {
    // SAFETY: the caller's promise.
    let p = unsafe { CStr::from_ptr(p) }.to_bytes();
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

/// Should `p` be looked for along the `'path'` rather than from here?
///
/// Only when the caller asked for a path search at all, and the pattern is
/// not one that already says where to look: an absolute name, or one
/// starting `"./"` or `"../"`.
///
/// # Safety
/// `p` must be a NUL-terminated string.
/// Either of the two flags that send the expansion looking down a search
/// list rather than the pattern's own directory.
const SEARCH_LIST: ExpandFlags = ExpandFlags::PATH.or(ExpandFlags::CDPATH);

unsafe fn wants_path_search(p: *const c_char, flags: ExpandFlags) -> bool {
    if !flags.has(SEARCH_LIST) || unsafe { path_is_absolute(p) } {
        return false;
    }
    let here = unsafe { *p } == b'.' as c_char
        && (vim_ispathsep(unsafe { *p.add(1) } as c_int)
            || unsafe { *p.add(1) } == b'.' as c_char
                && vim_ispathsep(unsafe { *p.add(2) } as c_int));
    !here
}

/// Expand the `num_pat` patterns in `pat` into `file`, and how many are
/// there into `num_file`.
///
/// A character that should not be expanded must have a backslash before it,
/// as in `"/path\\ with\\ spaces/my\\*star*"`. `flags` is the `EW_*` set
/// [`expand_wildcards`] documents.
///
/// Answers `Ok` when names were found, and `Err` otherwise — in which case
/// `num_file` and `file` are either untouched or set to zero and NULL. What
/// lands in `file` is the caller's, to be freed with [`free_wild`].
///
/// # Safety
/// `pat` must hold `num_pat` NUL-terminated strings, and `num_file` and
/// `file` must be writable.
pub unsafe fn gen_expand_wildcards(
    num_pat: c_int,
    pat: *mut *mut c_char,
    num_file: *mut c_int,
    file: *mut *mut *mut c_char,
    flags: ExpandFlags,
) -> Result<(), Failed> {
    // `expand_env` is called below to expand things like "~user". If
    // that fails it calls `expand_one`, which brings us back here; go
    // straight to the machine-specific expansion in that case.
    if RECURSIVE.get() {
        return unsafe { os_expand_wildcards(num_pat, pat, num_file, file, flags) };
    }

    // One pattern the shell has to handle sends them all to the shell,
    // so it is started once rather than once per pattern. A `=expr` is
    // ours to evaluate, though.
    for i in 0..num_pat {
        let p = unsafe { *pat.add(i as usize) };
        if unsafe { has_special_wildchar(p, flags) }
            && !(unsafe { vim_backtick(p) } && unsafe { *p.add(1) } == b'=' as c_char)
        {
            return unsafe { os_expand_wildcards(num_pat, pat, num_file, file, flags) };
        }
    }

    let path_option = unsafe { buffer_path() };
    RECURSIVE.set(true);
    let mut ga = garray_T::default();
    unsafe { ga_init(&raw mut ga, size_of::<*mut c_char>() as c_int, 30) };

    // Upstream never clears this: once one pattern has been looked for
    // along the 'path', every later one is uniquefied too.
    let mut did_expand_in_path = false;
    let mut i = 0;
    while i < num_pat && !got_int.get() {
        // How many names this pattern added, or -1 when nothing tried.
        let mut add_pat = -1;
        let mut p = unsafe { *pat.add(i as usize) };

        if unsafe { vim_backtick(p) } {
            add_pat = unsafe { expand_backtick(&raw mut ga, p, flags) };
            if add_pat == -1 {
                RECURSIVE.set(false);
                unsafe { ga_clear_strings(&raw mut ga) };
                unsafe { *num_file = 0 };
                unsafe { *file = core::ptr::null_mut() };
                return Err(Failed);
            }
        } else {
            // Environment variables, "~/" and "~user/" first.
            if unsafe { has_env_var(p) } && !flags.has(ExpandFlags::NOTENV)
                || unsafe { *p } == b'~' as c_char
            {
                let expanded = unsafe { expand_env_save_opt(p, true) };
                if !expanded.is_null() {
                    if unsafe { has_env_var(expanded) } || unsafe { *expanded } == b'~' as c_char {
                        // What `expand_env` could not expand, the shell
                        // can. Throw away what has been found so far and
                        // start over.
                        unsafe { xfree(expanded.cast()) };
                        unsafe { ga_clear_strings(&raw mut ga) };
                        let ret = unsafe {
                            os_expand_wildcards(
                                num_pat,
                                pat,
                                num_file,
                                file,
                                flags | ExpandFlags::KEEPDOLLAR,
                            )
                        };
                        RECURSIVE.set(false);
                        return ret;
                    }
                    p = expanded;
                }
            }

            if unsafe { path_has_exp_wildcard(p) } || flags.has(ExpandFlags::ICASE) {
                // A recursive `gen_expand_wildcards` can only happen from
                // an event handler in `os_breakcheck`, where it is fine.
                RECURSIVE.set(false);
                if unsafe { wants_path_search(p, flags) } {
                    // `:find` completion, where 'path' is used.
                    add_pat = unsafe { expand_in_path(&raw mut ga, p, flags) };
                    did_expand_in_path = true;
                } else {
                    let found = unsafe { path_expand(&raw mut ga, p, flags) };
                    debug_assert!(found <= c_int::MAX as usize, "path: too many matches");
                    add_pat = found as c_int;
                }
                RECURSIVE.set(true);
            }
        }

        if add_pat == -1 || add_pat == 0 && flags.has(ExpandFlags::NOTFOUND) {
            let t = unsafe { backslash_halve_save(p) };
            // With ExpandFlags::NOTFOUND always add files and directories: that is
            // what makes "vim c:/" work.
            if flags.has(ExpandFlags::NOTFOUND) {
                unsafe { addfile(&raw mut ga, t, flags | ExpandFlags::DIR | ExpandFlags::FILE) };
            } else {
                unsafe { addfile(&raw mut ga, t, flags) };
            }
            if t != p {
                unsafe { xfree(t.cast()) };
            }
        }

        if did_expand_in_path && ga.ga_len > 0 && flags.has(SEARCH_LIST) {
            RECURSIVE.set(false);
            unsafe { uniquefy_paths(&raw mut ga, p, path_option) };
            RECURSIVE.set(true);
        }
        if p != unsafe { *pat.add(i as usize) } {
            unsafe { xfree(p.cast()) };
        }
        i += 1;
    }

    unsafe { *num_file = ga.ga_len };
    unsafe { *file = ga.ga_data.cast() };
    RECURSIVE.set(false);

    if flags.has(ExpandFlags::EMPTYOK) || !ga.ga_data.is_null() {
        Ok(())
    } else {
        Err(Failed)
    }
}

/// Free the `count` names [`expand_wildcards`] and its neighbours answer.
///
/// # Safety
/// `files` must be NULL, or an allocated array of `count` allocated strings.
pub unsafe fn free_wild(count: c_int, files: *mut *mut c_char) {
    if count <= 0 || files.is_null() {
        return;
    }
    for i in (0..count as usize).rev() {
        unsafe { xfree((*files.add(i)).cast()) };
    }
    unsafe { xfree(files.cast()) };
}

/// Is `p` a whole backticked command, `` `cmd` ``?
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub(crate) unsafe fn vim_backtick(p: *mut c_char) -> bool {
    // SAFETY: the caller's promise.
    let p = unsafe { CStr::from_ptr(p) }.to_bytes();
    p.len() > 1 && p[0] == b'`' && p[p.len() - 1] == b'`'
}

/// Run the command in `` `pat` `` and add each line it prints to `gap`, or
/// evaluate it as an expression when it starts with `=`.
///
/// Answers how many names were added, or -1 when the command failed.
///
/// # Safety
/// `pat` must be a NUL-terminated string of at least two characters, and
/// `gap` an initialised array of allocated strings.
pub(crate) unsafe fn expand_backtick(
    gap: *mut garray_T,
    pat: *mut c_char,
    flags: ExpandFlags,
) -> c_int {
    // The command is the pattern without its backticks.
    let quoted = unsafe { CStr::from_ptr(pat) }.to_bytes();
    let cmd: *mut c_char =
        unsafe { xmemdupz(pat.add(1).cast(), (quoted.len() - 2) as size_t) }.cast();
    let buffer = if unsafe { *cmd } == b'=' as c_char {
        // `={expr}`: expand an expression.
        unsafe { eval_to_string(cmd.add(1), true, false) }
    } else {
        let opts = if flags.has(ExpandFlags::SILENT) {
            ShellOpts::SILENT
        } else {
            ShellOpts::NONE
        };
        unsafe { get_cmd_output(cmd, core::ptr::null_mut(), opts, core::ptr::null_mut()) }
    };
    unsafe { xfree(cmd.cast()) };
    if buffer.is_null() {
        return -1;
    }

    // One name per line, and a line with nothing but white space on it
    // is not a name.
    let mut cnt = 0;
    let mut cmd = buffer;
    while unsafe { *cmd } != 0 {
        cmd = unsafe { skipwhite(cmd) };
        let mut end = cmd;
        while unsafe { *end } != 0
            && unsafe { *end } != b'\r' as c_char
            && unsafe { *end } != b'\n' as c_char
        {
            end = unsafe { end.add(1) };
        }
        if end > cmd {
            // The name is terminated in place for the call, as upstream
            // does — the buffer is this function's own.
            let saved = unsafe { *end };
            unsafe { *end = 0 };
            unsafe { addfile(gap, cmd, flags) };
            unsafe { *end = saved };
            cnt += 1;
        }
        cmd = end;
        while unsafe { *cmd } == b'\r' as c_char || unsafe { *cmd } == b'\n' as c_char {
            cmd = unsafe { cmd.add(1) };
        }
    }

    unsafe { xfree(buffer.cast()) };
    cnt
}

/// [`expand_wildcards`] for one pattern that may start with `%`, `#` or
/// `<cword>` and friends, which are evaluated first.
///
/// A `%` or `#` that names no existing file still answers the name, without
/// the `*` that followed it, so that this works for remote files and for
/// buffers that are not files at all.
///
/// # Safety
/// `pat` must name a NUL-terminated string, and `num_file` and `file` must
/// be writable.
pub unsafe fn expand_wildcards_eval(
    pat: *mut *mut c_char,
    num_file: *mut c_int,
    file: *mut *mut *mut c_char,
    flags: ExpandFlags,
) -> Result<(), Failed> {
    let mut ret = Err(Failed);
    let mut eval_pat = core::ptr::null_mut();
    let mut exp_pat = unsafe { *pat };
    let mut ignored_msg: *const c_char = core::ptr::null();
    let mut usedlen: size_t = 0;
    let is_cur_alt_file =
        unsafe { *exp_pat } == b'%' as c_char || unsafe { *exp_pat } == b'#' as c_char;
    let mut star_follows = false;

    if is_cur_alt_file || unsafe { *exp_pat } == b'<' as c_char {
        let no_emsg = Suppress::emsg();
        eval_pat = unsafe {
            eval_vars(
                exp_pat,
                exp_pat,
                &raw mut usedlen,
                core::ptr::null_mut(),
                &raw mut ignored_msg,
                core::ptr::null_mut(),
                true,
            )
        };
        drop(no_emsg);
        if !eval_pat.is_null() {
            let rest = unsafe { exp_pat.add(usedlen as usize) };
            star_follows = unsafe { cstr::eq_bytes(rest, b"*") };
            exp_pat = unsafe { concat_str(eval_pat, rest) };
        }
    }

    if !exp_pat.is_null() {
        ret = unsafe { expand_wildcards(1, &raw mut exp_pat, num_file, file, flags) };
    }

    if !eval_pat.is_null() {
        if unsafe { *num_file } == 0 && is_cur_alt_file && star_follows {
            unsafe { *file = xmalloc(size_of::<*mut c_char>()).cast() };
            unsafe { **file = eval_pat };
            eval_pat = core::ptr::null_mut();
            unsafe { *num_file = 1 };
            ret = Ok(());
        }
        unsafe { xfree(exp_pat.cast()) };
        unsafe { xfree(eval_pat.cast()) };
    }
    ret
}

/// [`gen_expand_wildcards`], then drop the names `'wildignore'` matches and
/// move the ones `'suffixes'` matches to the end.
///
/// Answers `Ok` when `file` is set to an allocated array of matches and
/// `num_file` to how many there are, and `Err` otherwise — in which case
/// `num_file` and `file` are either untouched or set to zero and NULL.
///
/// # Safety
/// `pat` must hold `num_pat` NUL-terminated strings, and `num_files` and
/// `files` must be writable.
pub unsafe fn expand_wildcards(
    num_pat: c_int,
    pat: *mut *mut c_char,
    num_files: *mut c_int,
    files: *mut *mut *mut c_char,
    flags: ExpandFlags,
) -> Result<(), Failed> {
    let retval = unsafe { gen_expand_wildcards(num_pat, pat, num_files, files, flags) };
    if flags.has(ExpandFlags::KEEPALL) || retval.is_err() {
        return retval;
    }

    // Remove the names that match 'wildignore'.
    if unsafe { *p_wig.get() } != 0 {
        debug_assert!(
            unsafe { *num_files } == 0 || !unsafe { *files }.is_null(),
            "path: matches without an array to hold them"
        );
        let mut kept = 0;
        for i in 0..unsafe { *num_files } as usize {
            let name = unsafe { *(*files).add(i) };
            debug_assert!(!name.is_null(), "path: a match with no name");
            let ffname = unsafe { full_name_save(name, false) };
            debug_assert!(!ffname.is_null(), "path: a match with no full name");
            if unsafe { match_file_list(p_wig.get(), name, ffname) } {
                unsafe { xfree(name.cast()) };
            } else {
                unsafe { *(*files).add(kept) = name };
                kept += 1;
            }
            unsafe { xfree(ffname.cast()) };
        }
        unsafe { *num_files = kept as c_int };
    }

    // Move the names where 'suffixes' match to the end. Skip it when
    // interrupted: the result probably won't be used.
    debug_assert!(
        unsafe { *num_files } == 0 || !unsafe { *files }.is_null(),
        "path: matches without an array to hold them"
    );
    if unsafe { *num_files } > 1 && !got_int.get() {
        // How many names without a matching suffix are at the front.
        let mut non_suf_match = 0;
        for i in 0..unsafe { *num_files } as usize {
            if unsafe { match_suffix(*(*files).add(i)) } {
                continue;
            }
            let name = unsafe { *(*files).add(i) };
            unsafe {
                core::ptr::copy(
                    (*files).add(non_suf_match),
                    (*files).add(non_suf_match + 1),
                    i - non_suf_match,
                )
            };
            unsafe { *(*files).add(non_suf_match) = name };
            non_suf_match += 1;
        }
    }

    if unsafe { *num_files } == 0 {
        unsafe { xfree((*files).cast()) };
        unsafe { *files = core::ptr::null_mut() };
        return Err(Failed);
    }
    retval
}
