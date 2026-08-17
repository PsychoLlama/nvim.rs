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
use core::ffi::{c_char, c_int};
use std::ffi::CStr;

#[allow(unused_imports)]
use super::*;
use crate::os::shell::ShellOpts;

/// Whether a `gen_expand_wildcards` is already running. The pieces it calls
/// can come back round to it — `expand_env` falls back on `ExpandOne` — and
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
    unsafe {
        let mut buf = vec![0 as c_char; MAXPATHL as usize];
        let buf = buf.as_mut_ptr();
        let mut curdirlen = 0;
        while *path_option != 0 {
            let mut buflen = copy_option_part(
                &raw mut path_option,
                buf,
                MAXPATHL as size_t,
                c" ,".as_ptr().cast_mut(),
            );

            // Do not expand backticks: this could have been set by a modeline.
            if !vim_strchr(buf, c_int::from(b'`')).is_null() {
                continue;
            }

            if *buf == b'.' as c_char && (*buf.add(1) == 0 || vim_ispathsep(*buf.add(1) as c_int)) {
                // Relative to the current buffer:
                //     "/path/file" + "."        -> "/path/"
                //     "/path/file" + "./subdir" -> "/path/subdir"
                let ffname = (*curbuf.get()).b_ffname;
                if ffname.is_null() {
                    continue;
                }
                let plen = path_tail(ffname).offset_from(ffname) as usize;
                if plen + buflen >= MAXPATHL as usize {
                    continue;
                }
                if *buf.add(1) == 0 {
                    *buf.add(plen) = 0;
                } else {
                    // The entry past its ".", and the NUL with it.
                    core::ptr::copy(buf.add(2), buf.add(plen), buflen - 2 + 1);
                }
                core::ptr::copy(ffname, buf, plen);
                buflen = simplify_filename(buf) as usize;
            } else if *buf == 0 {
                // Relative to the current directory.
                if curdirlen == 0 {
                    curdirlen = CStr::from_ptr(curdir).to_bytes().len();
                }
                core::ptr::copy_nonoverlapping(curdir, buf, curdirlen + 1);
                buflen = curdirlen;
            } else if path_with_url(buf) != 0 {
                continue; // a URL can't be used here
            } else if !path_is_absolute(buf) {
                // Expand a relative path to its full equivalent.
                if curdirlen == 0 {
                    curdirlen = CStr::from_ptr(curdir).to_bytes().len();
                }
                // The directory, the separator, the entry, and the NUL.
                if curdirlen + buflen + 3 > MAXPATHL as usize {
                    continue;
                }
                core::ptr::copy(buf, buf.add(curdirlen + 1), buflen + 1);
                core::ptr::copy_nonoverlapping(curdir, buf, curdirlen);
                *buf.add(curdirlen) = PATHSEP as c_char;
                buflen = simplify_filename(buf) as usize;
            }

            ga_grow(gap, 1);
            *(*gap)
                .ga_data
                .cast::<*mut c_char>()
                .add((*gap).ga_len as usize) = xmemdupz(buf.cast(), buflen as size_t).cast();
            (*gap).ga_len += 1;
        }
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
    unsafe {
        let mut curdir = vec![0 as c_char; MAXPATHL as usize];
        os_dirname(curdir.as_mut_ptr(), MAXPATHL as size_t);

        let mut path_ga = garray_T::default();
        ga_init(&raw mut path_ga, size_of::<*mut c_char>() as c_int, 1);
        let path_option = if flags.has(ExpandFlags::CDPATH) {
            p_cdpath.get()
        } else {
            buffer_path()
        };
        expand_path_option(curdir.as_mut_ptr(), path_option, &raw mut path_ga);
        if path_ga.ga_len <= 0 {
            return 0;
        }

        let paths = ga_concat_strings(&raw mut path_ga, c",".as_ptr());
        ga_clear_strings(&raw mut path_ga);

        let mut glob_flags = WildOpts::NONE;
        if flags.has(ExpandFlags::ICASE) {
            glob_flags |= WildOpts::ICASE;
        }
        if flags.has(ExpandFlags::ADDSLASH) {
            glob_flags |= WildOpts::ADD_SLASH;
        }
        globpath(
            paths,
            pattern,
            gap,
            glob_flags,
            flags.has(ExpandFlags::CDPATH),
        );
        xfree(paths.cast());

        (*gap).ga_len
    }
}

/// The `'path'` in force: the buffer's own, or the global one when it is
/// empty.
///
/// # Safety
/// There must be a current buffer.
pub(crate) unsafe fn buffer_path() -> *mut c_char {
    unsafe {
        if *(*curbuf.get()).b_p_path == 0 {
            p_path.get()
        } else {
            (*curbuf.get()).b_p_path
        }
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
    unsafe {
        if !flags.has(SEARCH_LIST) || path_is_absolute(p) {
            return false;
        }
        let here = *p == b'.' as c_char
            && (vim_ispathsep(*p.add(1) as c_int)
                || *p.add(1) == b'.' as c_char && vim_ispathsep(*p.add(2) as c_int));
        !here
    }
}

/// Expand the `num_pat` patterns in `pat` into `file`, and how many are
/// there into `num_file`.
///
/// A character that should not be expanded must have a backslash before it,
/// as in `"/path\\ with\\ spaces/my\\*star*"`. `flags` is the `EW_*` set
/// [`expand_wildcards`] documents.
///
/// Answers OK when names were found, and FAIL otherwise — in which case
/// `num_file` and `file` are either untouched or set to zero and NULL. What
/// lands in `file` is the caller's, to be freed with [`FreeWild`].
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
) -> c_int {
    unsafe {
        // `expand_env` is called below to expand things like "~user". If
        // that fails it calls `ExpandOne`, which brings us back here; go
        // straight to the machine-specific expansion in that case.
        if RECURSIVE.get() {
            return os_expand_wildcards(num_pat, pat, num_file, file, flags);
        }

        // One pattern the shell has to handle sends them all to the shell,
        // so it is started once rather than once per pattern. A `=expr` is
        // ours to evaluate, though.
        for i in 0..num_pat {
            let p = *pat.add(i as usize);
            if has_special_wildchar(p, flags) && !(vim_backtick(p) && *p.add(1) == b'=' as c_char) {
                return os_expand_wildcards(num_pat, pat, num_file, file, flags);
            }
        }

        let path_option = buffer_path();
        RECURSIVE.set(true);
        let mut ga = garray_T::default();
        ga_init(&raw mut ga, size_of::<*mut c_char>() as c_int, 30);

        // Upstream never clears this: once one pattern has been looked for
        // along the 'path', every later one is uniquefied too.
        let mut did_expand_in_path = false;
        let mut i = 0;
        while i < num_pat && !got_int.get() {
            // How many names this pattern added, or -1 when nothing tried.
            let mut add_pat = -1;
            let mut p = *pat.add(i as usize);

            if vim_backtick(p) {
                add_pat = expand_backtick(&raw mut ga, p, flags);
                if add_pat == -1 {
                    RECURSIVE.set(false);
                    ga_clear_strings(&raw mut ga);
                    *num_file = 0;
                    *file = core::ptr::null_mut();
                    return FAIL;
                }
            } else {
                // Environment variables, "~/" and "~user/" first.
                if has_env_var(p) && !flags.has(ExpandFlags::NOTENV) || *p == b'~' as c_char {
                    let expanded = expand_env_save_opt(p, true);
                    if !expanded.is_null() {
                        if has_env_var(expanded) || *expanded == b'~' as c_char {
                            // What `expand_env` could not expand, the shell
                            // can. Throw away what has been found so far and
                            // start over.
                            xfree(expanded.cast());
                            ga_clear_strings(&raw mut ga);
                            let ret = os_expand_wildcards(
                                num_pat,
                                pat,
                                num_file,
                                file,
                                flags | ExpandFlags::KEEPDOLLAR,
                            );
                            RECURSIVE.set(false);
                            return ret;
                        }
                        p = expanded;
                    }
                }

                if path_has_exp_wildcard(p) || flags.has(ExpandFlags::ICASE) {
                    // A recursive `gen_expand_wildcards` can only happen from
                    // an event handler in `os_breakcheck`, where it is fine.
                    RECURSIVE.set(false);
                    if wants_path_search(p, flags) {
                        // `:find` completion, where 'path' is used.
                        add_pat = expand_in_path(&raw mut ga, p, flags);
                        did_expand_in_path = true;
                    } else {
                        let found = path_expand(&raw mut ga, p, flags);
                        debug_assert!(found <= c_int::MAX as usize, "path: too many matches");
                        add_pat = found as c_int;
                    }
                    RECURSIVE.set(true);
                }
            }

            if add_pat == -1 || add_pat == 0 && flags.has(ExpandFlags::NOTFOUND) {
                let t = backslash_halve_save(p);
                // With ExpandFlags::NOTFOUND always add files and directories: that is
                // what makes "vim c:/" work.
                if flags.has(ExpandFlags::NOTFOUND) {
                    addfile(&raw mut ga, t, flags | ExpandFlags::DIR | ExpandFlags::FILE);
                } else {
                    addfile(&raw mut ga, t, flags);
                }
                if t != p {
                    xfree(t.cast());
                }
            }

            if did_expand_in_path && ga.ga_len > 0 && flags.has(SEARCH_LIST) {
                RECURSIVE.set(false);
                uniquefy_paths(&raw mut ga, p, path_option);
                RECURSIVE.set(true);
            }
            if p != *pat.add(i as usize) {
                xfree(p.cast());
            }
            i += 1;
        }

        *num_file = ga.ga_len;
        *file = ga.ga_data.cast();
        RECURSIVE.set(false);

        if flags.has(ExpandFlags::EMPTYOK) || !ga.ga_data.is_null() {
            OK
        } else {
            FAIL
        }
    }
}

/// Free the `count` names [`expand_wildcards`] and its neighbours answer.
///
/// # Safety
/// `files` must be NULL, or an allocated array of `count` allocated strings.
pub unsafe fn FreeWild(count: c_int, files: *mut *mut c_char) {
    unsafe {
        if count <= 0 || files.is_null() {
            return;
        }
        for i in (0..count as usize).rev() {
            xfree((*files.add(i)).cast());
        }
        xfree(files.cast());
    }
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
    unsafe {
        // The command is the pattern without its backticks.
        let quoted = CStr::from_ptr(pat).to_bytes();
        let cmd: *mut c_char = xmemdupz(pat.add(1).cast(), (quoted.len() - 2) as size_t).cast();
        let buffer = if *cmd == b'=' as c_char {
            // `={expr}`: expand an expression.
            eval_to_string(cmd.add(1), true, false)
        } else {
            let opts = if flags.has(ExpandFlags::SILENT) {
                ShellOpts::SILENT
            } else {
                ShellOpts::NONE
            };
            get_cmd_output(cmd, core::ptr::null_mut(), opts, core::ptr::null_mut())
        };
        xfree(cmd.cast());
        if buffer.is_null() {
            return -1;
        }

        // One name per line, and a line with nothing but white space on it
        // is not a name.
        let mut cnt = 0;
        let mut cmd = buffer;
        while *cmd != 0 {
            cmd = skipwhite(cmd);
            let mut end = cmd;
            while *end != 0 && *end != b'\r' as c_char && *end != b'\n' as c_char {
                end = end.add(1);
            }
            if end > cmd {
                // The name is terminated in place for the call, as upstream
                // does — the buffer is this function's own.
                let saved = *end;
                *end = 0;
                addfile(gap, cmd, flags);
                *end = saved;
                cnt += 1;
            }
            cmd = end;
            while *cmd == b'\r' as c_char || *cmd == b'\n' as c_char {
                cmd = cmd.add(1);
            }
        }

        xfree(buffer.cast());
        cnt
    }
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
) -> c_int {
    unsafe {
        let mut ret = FAIL;
        let mut eval_pat = core::ptr::null_mut();
        let mut exp_pat = *pat;
        let mut ignored_msg: *const c_char = core::ptr::null();
        let mut usedlen: size_t = 0;
        let is_cur_alt_file = *exp_pat == b'%' as c_char || *exp_pat == b'#' as c_char;
        let mut star_follows = false;

        if is_cur_alt_file || *exp_pat == b'<' as c_char {
            *emsg_off.ptr() += 1;
            eval_pat = eval_vars(
                exp_pat,
                exp_pat,
                &raw mut usedlen,
                core::ptr::null_mut(),
                &raw mut ignored_msg,
                core::ptr::null_mut(),
                true,
            );
            *emsg_off.ptr() -= 1;
            if !eval_pat.is_null() {
                let rest = exp_pat.add(usedlen as usize);
                star_follows = strcmp(rest, c"*".as_ptr()) == 0;
                exp_pat = concat_str(eval_pat, rest);
            }
        }

        if !exp_pat.is_null() {
            ret = expand_wildcards(1, &raw mut exp_pat, num_file, file, flags);
        }

        if !eval_pat.is_null() {
            if *num_file == 0 && is_cur_alt_file && star_follows {
                *file = xmalloc(size_of::<*mut c_char>()).cast();
                **file = eval_pat;
                eval_pat = core::ptr::null_mut();
                *num_file = 1;
                ret = OK;
            }
            xfree(exp_pat.cast());
            xfree(eval_pat.cast());
        }
        ret
    }
}

/// [`gen_expand_wildcards`], then drop the names `'wildignore'` matches and
/// move the ones `'suffixes'` matches to the end.
///
/// Answers OK when `file` is set to an allocated array of matches and
/// `num_file` to how many there are, and FAIL otherwise — in which case
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
) -> c_int {
    unsafe {
        let retval = gen_expand_wildcards(num_pat, pat, num_files, files, flags);
        if flags.has(ExpandFlags::KEEPALL) || retval == FAIL {
            return retval;
        }

        // Remove the names that match 'wildignore'.
        if *p_wig.get() != 0 {
            debug_assert!(
                *num_files == 0 || !(*files).is_null(),
                "path: matches without an array to hold them"
            );
            let mut kept = 0;
            for i in 0..*num_files as usize {
                let name = *(*files).add(i);
                debug_assert!(!name.is_null(), "path: a match with no name");
                let ffname = FullName_save(name, false);
                debug_assert!(!ffname.is_null(), "path: a match with no full name");
                if match_file_list(p_wig.get(), name, ffname) {
                    xfree(name.cast());
                } else {
                    *(*files).add(kept) = name;
                    kept += 1;
                }
                xfree(ffname.cast());
            }
            *num_files = kept as c_int;
        }

        // Move the names where 'suffixes' match to the end. Skip it when
        // interrupted: the result probably won't be used.
        debug_assert!(
            *num_files == 0 || !(*files).is_null(),
            "path: matches without an array to hold them"
        );
        if *num_files > 1 && !got_int.get() {
            // How many names without a matching suffix are at the front.
            let mut non_suf_match = 0;
            for i in 0..*num_files as usize {
                if match_suffix(*(*files).add(i)) {
                    continue;
                }
                let name = *(*files).add(i);
                core::ptr::copy(
                    (*files).add(non_suf_match),
                    (*files).add(non_suf_match + 1),
                    i - non_suf_match,
                );
                *(*files).add(non_suf_match) = name;
                non_suf_match += 1;
            }
        }

        if *num_files == 0 {
            xfree((*files).cast());
            *files = core::ptr::null_mut();
            return FAIL;
        }
        retval
    }
}
