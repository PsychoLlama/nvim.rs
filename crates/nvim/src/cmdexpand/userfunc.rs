//! Match sources that run user code or walk the file system.
//!
//! `'shellcmd'` completion ([`expand_shellcmd`]) walks `$PATH`;
//! [`globpath`] walks a comma-separated directory list; and the
//! `custom,`/`customlist,`/Lua completion functions of `:command` are called
//! through [`ExpandUserDefined`], [`ExpandUserList`] and [`ExpandUserLua`].

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cmdexpand::WildOpts;
use crate::path::ExpandFlags;
use crate::types::{VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED};
use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of;
use core::ptr;

/// A `:command -complete=custom,…` callback: `f(arg, argc, argv)`.
///
/// Only [`call_user_expand_func`] takes one, and both of its callers pass a
/// real function, so this is the bare pointer rather than upstream's nullable
/// `user_expand_func_T`.
type UserExpandFunc = unsafe extern "C" fn(*const c_char, c_int, *mut typval_T) -> *mut c_void;

/// The length of `PATHSEPSTR`, which is what upstream's
/// `STRLEN_LITERAL(PATHSEPSTR)` comes to on every platform this port builds
/// for.
const PATHSEP_LEN: size_t = 1;

/// Expand shell command matches in one directory of `$PATH`.
///
/// `pathed_pattern` is the fully pathed pattern and `pathlen` the length of
/// its path portion (0 if there is no path).  New names are appended to `gap`
/// and remembered in `ht` so a later directory cannot offer them again.
pub(crate) unsafe fn expand_shellcmd_onedir(
    pathed_pattern: *mut c_char,
    pathlen: size_t,
    matches: *mut *mut *mut c_char,
    numMatches: *mut c_int,
    flags: ExpandFlags,
    ht: *mut hashtab_T,
    gap: *mut garray_T,
) {
    unsafe {
        let mut pathed_pattern = pathed_pattern;
        if expand_wildcards(1, &raw mut pathed_pattern, numMatches, matches, flags) != OK {
            return;
        }

        ga_grow(gap, *numMatches);

        for i in 0..*numMatches {
            let mut name = *(*matches).offset(i as isize);
            let namelen = strlen(name);

            if namelen > pathlen {
                // Check if this name was already found.
                let hash = hash_hash(name.add(pathlen));
                let hi = hash_lookup(ht, name.add(pathlen), namelen - pathlen, hash);
                // HASHITEM_EMPTY().
                if (*hi).hi_key.is_null() || (*hi).hi_key == &raw const hash_removed as *mut c_char
                {
                    // Remove the path that was prepended (+1 for the NUL).
                    memmove(
                        name as *mut c_void,
                        name.add(pathlen) as *const c_void,
                        namelen - pathlen + 1,
                    );
                    ((*gap).ga_data as *mut *mut c_char)
                        .offset((*gap).ga_len as isize)
                        .write(name);
                    (*gap).ga_len += 1;
                    hash_add_item(ht, hi, name, hash);
                    name = ptr::null_mut();
                }
            }
            xfree(name as *mut c_void);
        }
        xfree(*matches as *mut c_void);
    }
}

/// Complete a shell command.
///
/// `filepat` is a pattern to match with command names; `matches` and
/// `numMatches` return the answer, with `*matches` either NULL or allocated.
/// `flagsarg` is the caller's [`ExpandFlags`] set.
pub(crate) unsafe fn expand_shellcmd(
    filepat: *mut c_char,
    matches: *mut *mut *mut c_char,
    numMatches: *mut c_int,
    flagsarg: ExpandFlags,
) {
    unsafe {
        let buf = xmalloc(MAXPATHL as size_t) as *mut c_char;
        let mut flags = flagsarg;
        let mut did_curdir = false;

        // For ":set path=" and ":set tags=" halve backslashes for escaped
        // space.
        let mut patlen = strlen(filepat);
        let pat = xmemdupz(filepat as *const c_void, patlen) as *mut c_char;
        // Replace "\ " with " ".
        let mut e = pat.add(patlen);
        let mut s = pat;
        while *s as c_int != NUL {
            if *s as c_int == '\\' as c_int {
                let p = s.add(1);
                if *p as c_int == ' ' as c_int {
                    memmove(
                        s as *mut c_void,
                        p as *const c_void,
                        e.offset_from(p) as size_t + 1, // +1 for NUL
                    );
                    e = e.sub(1);
                }
            }
            s = s.add(1);
        }
        patlen = e.offset_from(pat) as size_t;

        flags |= ExpandFlags::FILE | ExpandFlags::EXEC | ExpandFlags::SHELLCMD;

        let mut mustfree = false; // Track memory allocation for `path`.
        let mut path;
        if *pat as c_int == '.' as c_int
            && (vim_ispathsep(*pat.add(1) as c_int)
                || (*pat.add(1) as c_int == '.' as c_int && vim_ispathsep(*pat.add(2) as c_int)))
        {
            path = c".".as_ptr() as *mut c_char;
        } else {
            // For an absolute name we don't use $PATH.
            path = if path_is_absolute(pat) {
                ptr::null_mut()
            } else {
                vim_getenv(c"PATH".as_ptr())
            };
            if path.is_null() {
                path = c"".as_ptr() as *mut c_char;
            } else {
                mustfree = true;
            }
        }

        // Go over all directories in $PATH.  Expand matches in that directory
        // and collect them in `ga`.  When "." is not in $PATH also expand for
        // the current directory, to find "subdir/cmd".
        let mut ga = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ptr::null_mut(),
        };
        ga_init(&raw mut ga, size_of::<*mut c_char>() as c_int, 10);
        let mut found_ht: hashtab_T = core::mem::zeroed();
        hash_init(&raw mut found_ht);
        let mut s = path;
        loop {
            // Length of the path portion of buf, including trailing slash.
            let mut pathlen;
            let seplen;

            if *s as c_int == NUL {
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
                e = vim_strchr(s, ENV_SEPCHAR);
                if e.is_null() {
                    e = s.add(strlen(s));
                }

                pathlen = e.offset_from(s) as size_t;
                if strncmp(s, c".".as_ptr(), pathlen) == 0 {
                    did_curdir = true;
                    flags |= ExpandFlags::DIR;
                } else {
                    // Do not match directories inside a $PATH item.
                    flags = flags.without(ExpandFlags::DIR);
                }

                seplen = if after_pathsep(s, e) == 0 {
                    PATHSEP_LEN
                } else {
                    0
                };
            }

            // Make sure that the pathed pattern (ie the path and pattern
            // concatenated together) will fit inside the buffer.  If not skip
            // it and move on to the next path.
            // Upstream's `+ 1 <= MAXPATHL` — the one byte is the NUL.
            if pathlen + seplen + patlen < MAXPATHL as size_t {
                if pathlen > 0 {
                    xmemcpyz(buf as *mut c_void, s as *const c_void, pathlen);
                    if seplen > 0 {
                        xmemcpyz(
                            buf.add(pathlen) as *mut c_void,
                            c"/".as_ptr() as *const c_void,
                            PATHSEP_LEN,
                        );
                        pathlen += seplen;
                    }
                }
                xmemcpyz(
                    buf.add(pathlen) as *mut c_void,
                    pat as *const c_void,
                    patlen,
                );

                expand_shellcmd_onedir(
                    buf,
                    pathlen,
                    matches,
                    numMatches,
                    flags,
                    &raw mut found_ht,
                    &raw mut ga,
                );
            }

            if *e as c_int != NUL {
                e = e.add(1);
            }
            s = e;
        }
        *matches = ga.ga_data as *mut *mut c_char;
        *numMatches = ga.ga_len;

        xfree(buf as *mut c_void);
        xfree(pat as *mut c_void);
        if mustfree {
            xfree(path as *mut c_void);
        }
        hash_clear(&raw mut found_ht);
    }
}

/// Call `user_expand_func` to invoke a user defined Vim script function.
///
/// Returns its result — a string, a List or NULL.  The function is handed the
/// pattern, the whole command line and the cursor column.
pub(crate) unsafe fn call_user_expand_func(
    user_expand_func: UserExpandFunc,
    xp: *mut expand_T,
) -> *mut c_void {
    unsafe {
        let ccline: *mut CmdlineInfo = get_cmdline_info();
        let mut keep = 0 as c_char;
        let mut args = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 4];
        let save_current_sctx = current_sctx.get();

        if (*xp).xp_arg.is_null() || *(*xp).xp_arg as c_int == NUL || (*xp).xp_line.is_null() {
            return ptr::null_mut();
        }

        if !(*ccline).cmdbuff.is_null() {
            keep = *(*ccline).cmdbuff.offset((*ccline).cmdlen as isize);
            *(*ccline).cmdbuff.offset((*ccline).cmdlen as isize) = 0;
        }

        let pat = xstrnsave((*xp).xp_pattern, (*xp).xp_pattern_len);
        args[0].v_type = VAR_STRING;
        args[1].v_type = VAR_STRING;
        args[2].v_type = VAR_NUMBER;
        args[3].v_type = VAR_UNKNOWN;
        args[0].vval.v_string = pat;
        args[1].vval.v_string = (*xp).xp_line;
        args[2].vval.v_number = (*xp).xp_col as varnumber_T;

        current_sctx.set((*xp).xp_script_ctx);

        let ret = user_expand_func((*xp).xp_arg, 3, args.as_mut_ptr());

        current_sctx.set(save_current_sctx);
        if !(*ccline).cmdbuff.is_null() {
            *(*ccline).cmdbuff.offset((*ccline).cmdlen as isize) = keep;
        }

        xfree(pat as *mut c_void);
        ret
    }
}

/// Expand names with a function defined by the user
/// (`EXPAND_USER_DEFINED` and `EXPAND_USER_LIST`).
pub(crate) unsafe fn ExpandUserDefined(
    pat: *const c_char,
    xp: *mut expand_T,
    regmatch: *mut regmatch_T,
    matches: *mut *mut *mut c_char,
    numMatches: *mut c_int,
) -> c_int {
    unsafe {
        let fuzzy = cmdline_fuzzy_complete(pat);
        *matches = ptr::null_mut();
        *numMatches = 0;

        let retstr = call_user_expand_func(call_func_retstr, xp) as *mut c_char;
        if retstr.is_null() {
            return FAIL;
        }

        let mut ga = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ptr::null_mut(),
        };
        let itemsize = if fuzzy {
            size_of::<fuzmatch_str_T>()
        } else {
            size_of::<*mut c_char>()
        };
        ga_init(&raw mut ga, itemsize as c_int, 3);

        // The answer is one match per line.
        let mut s = retstr;
        while *s as c_int != NUL {
            let mut e = vim_strchr(s, '\n' as c_int);
            if e.is_null() {
                e = s.add(strlen(s));
            }
            let keep = *e;
            *e = NUL as c_char;

            let mut score = 0;
            let matched = if *(*xp).xp_pattern as c_int == NUL {
                true // match everything
            } else if fuzzy {
                score = fuzzy_match_str(s, pat);
                score != FUZZY_SCORE_NONE
            } else {
                vim_regexec(regmatch, s, 0)
            };

            *e = keep;

            if matched {
                let p = xmemdupz(s as *const c_void, e.offset_from(s) as size_t) as *mut c_char;

                ga_grow(&raw mut ga, 1);
                if fuzzy {
                    (ga.ga_data as *mut fuzmatch_str_T)
                        .offset(ga.ga_len as isize)
                        .write(fuzmatch_str_T {
                            idx: ga.ga_len,
                            str: p,
                            score,
                        });
                } else {
                    (ga.ga_data as *mut *mut c_char)
                        .offset(ga.ga_len as isize)
                        .write(p);
                }
                ga.ga_len += 1;
            }

            if *e as c_int != NUL {
                e = e.add(1);
            }
            s = e;
        }
        xfree(retstr as *mut c_void);

        if ga.ga_len == 0 {
            return OK;
        }

        if fuzzy {
            fuzzymatches_to_strmatches(
                ga.ga_data as *mut fuzmatch_str_T,
                matches,
                ga.ga_len,
                false,
            );
        } else {
            *matches = ga.ga_data as *mut *mut c_char;
        }
        *numMatches = ga.ga_len;
        OK
    }
}

/// Copy the strings of a `customlist,` answer into a fresh match array.
pub(crate) unsafe fn process_user_list(
    retlist: *mut list_T,
    matches: *mut *mut *mut c_char,
    numMatches: *mut c_int,
) {
    unsafe {
        let mut ga = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ptr::null_mut(),
        };
        ga_init(&raw mut ga, size_of::<*mut c_char>() as c_int, 3);

        // Loop over the items in the list.
        if !retlist.is_null() {
            let mut li: *const listitem_T = (*retlist).lv_first;
            while !li.is_null() {
                // Skip non-string items and empty strings.
                if (*li).li_tv.v_type == VAR_STRING && !(*li).li_tv.vval.v_string.is_null() {
                    let p = xstrdup((*li).li_tv.vval.v_string);
                    ga_grow(&raw mut ga, 1);
                    (ga.ga_data as *mut *mut c_char)
                        .offset(ga.ga_len as isize)
                        .write(p);
                    ga.ga_len += 1;
                }
                li = (*li).li_next;
            }
        }
        tv_list_unref(retlist);

        *matches = ga.ga_data as *mut *mut c_char;
        *numMatches = ga.ga_len;
    }
}

/// Expand names with a list returned by a function defined by the user.
pub(crate) unsafe fn ExpandUserList(
    xp: *mut expand_T,
    matches: *mut *mut *mut c_char,
    numMatches: *mut c_int,
) -> c_int {
    unsafe {
        *matches = ptr::null_mut();
        *numMatches = 0;
        let retlist = call_user_expand_func(call_func_retlist, xp) as *mut list_T;
        if retlist.is_null() {
            return FAIL;
        }

        process_user_list(retlist, matches, numMatches);
        OK
    }
}

/// Expand names with a Lua completion function.
pub(crate) unsafe fn ExpandUserLua(
    xp: *mut expand_T,
    numMatches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    unsafe {
        let mut rettv = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        nlua_call_user_expand_func(xp, &raw mut rettv);
        if rettv.v_type != VAR_LIST {
            tv_clear(&raw mut rettv);
            return FAIL;
        }

        process_user_list(rettv.vval.v_list, matches, numMatches);
        OK
    }
}

/// Expand `file` for all comma-separated directories in `path`, adding the
/// matches to `ga`.
///
/// If `dirs` is true only directory names are expanded.
pub unsafe fn globpath(
    path: *mut c_char,
    file: *mut c_char,
    ga: *mut garray_T,
    expand_options: WildOpts,
    dirs: bool,
) {
    unsafe {
        let buf = xmalloc(MAXPATHL as size_t) as *mut c_char;

        let mut xpc: expand_T = core::mem::zeroed();
        ExpandInit(&raw mut xpc);
        xpc.xp_context = if dirs {
            EXPAND_DIRECTORIES
        } else {
            EXPAND_FILES
        };

        let filelen = strlen(file);

        // Loop over all entries in {path}.
        let mut path = path;
        while *path as c_int != NUL {
            // Copy one item of the path to buf[] and concatenate the file
            // name.  `pathlen` is the length of the path portion of buf,
            // including the trailing slash.
            let mut pathlen = copy_option_part(
                &raw mut path,
                buf,
                MAXPATHL as size_t,
                c",".as_ptr() as *mut c_char,
            );
            let seplen = if *buf as c_int != NUL && after_pathsep(buf, buf.add(pathlen)) == 0 {
                PATHSEP_LEN
            } else {
                0
            };

            // Upstream's `+ 1 <= MAXPATHL` — the one byte is the NUL.
            if pathlen + seplen + filelen < MAXPATHL as size_t {
                if seplen > 0 {
                    xmemcpyz(
                        buf.add(pathlen) as *mut c_void,
                        c"/".as_ptr() as *const c_void,
                        PATHSEP_LEN,
                    );
                    pathlen += seplen;
                }
                xmemcpyz(
                    buf.add(pathlen) as *mut c_void,
                    file as *const c_void,
                    filelen,
                );

                let mut p: *mut *mut c_char = ptr::null_mut();
                let mut num_p = 0;
                ExpandFromContext(
                    &raw mut xpc,
                    buf,
                    &raw mut p,
                    &raw mut num_p,
                    WildOpts::SILENT | expand_options,
                );
                if num_p > 0 {
                    escape_matches(
                        &raw mut xpc,
                        buf,
                        core::slice::from_raw_parts_mut(p, num_p as usize),
                        WildOpts::SILENT | expand_options,
                    );

                    // Concatenate new results to previous ones, taking over
                    // the pointers.
                    ga_grow(ga, num_p);
                    for i in 0..num_p {
                        ((*ga).ga_data as *mut *mut c_char)
                            .offset((*ga).ga_len as isize)
                            .write(*p.offset(i as isize));
                        (*ga).ga_len += 1;
                    }
                    xfree(p as *mut c_void);
                }
            }
        }

        xfree(buf as *mut c_void);
    }
}
