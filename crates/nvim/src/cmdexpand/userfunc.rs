//! Match sources that run user code or walk the file system.
//!
//! `'shellcmd'` completion ([`expand_shellcmd`]) walks `$PATH`;
//! [`globpath`] walks a comma-separated directory list; and the
//! `custom,`/`customlist,`/Lua completion functions of `:command` are called
//! through [`expand_user_defined`], [`expand_user_list`] and [`expand_user_lua`].

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cmdexpand::WildOpts;
use crate::cstr;
use crate::path::ExpandFlags;
use crate::types::{
    ExpandContext, Failed, MAXPATHL, NUL, PATHSEPSTR, VAR_LIST, VAR_NUMBER, VAR_STRING,
    VAR_UNKNOWN, VarLock,
};
use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of;
use core::ptr;

/// A `:command -complete=custom,…` callback: `f(arg, argc, argv)`.
///
/// Only [`call_user_expand_func`] takes one, and both of its callers pass a
/// real function, so this is the bare pointer rather than upstream's nullable
/// `user_expand_func_T`.
type UserExpandFunc = unsafe fn(*const c_char, c_int, *mut typval_T) -> *mut c_void;

/// Upstream's `STRLEN_LITERAL(PATHSEPSTR)`.
const PATHSEP_LEN: size_t = PATHSEPSTR.count_bytes() as size_t;

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
    let mut pathed_pattern = pathed_pattern;
    if unsafe { expand_wildcards(1, &raw mut pathed_pattern, numMatches, matches, flags) }.is_err()
    {
        return;
    }

    unsafe { ga_grow(gap, *numMatches) };

    for i in 0..unsafe { *numMatches } {
        let mut name = unsafe { *(*matches).offset(i as isize) };
        let namelen = unsafe { cstr::bytes_at(name) }.len();

        if namelen > pathlen {
            // Check if this name was already found.
            let hash = unsafe { hash_hash(name.add(pathlen)) };
            let hi = unsafe { hash_lookup(ht, name.add(pathlen), namelen - pathlen, hash) };
            if !hi.is_kept() {
                // Remove the path that was prepended (+1 for the NUL).
                let into = name.cast::<u8>();
                unsafe { into.copy_from(name.add(pathlen).cast(), namelen - pathlen + 1) };
                unsafe {
                    ((*gap).ga_data as *mut *mut c_char)
                        .offset((*gap).ga_len as isize)
                        .write(name)
                };
                unsafe { (*gap).ga_len += 1 };
                unsafe { hash_add_item(ht, hi, name, hash) };
                name = ptr::null_mut();
            }
        }
        unsafe { xfree(name as *mut c_void) };
    }
    unsafe { xfree(*matches as *mut c_void) };
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
    let buf = unsafe { xmalloc(MAXPATHL as size_t) } as *mut c_char;
    let mut flags = flagsarg;
    let mut did_curdir = false;

    // For ":set path=" and ":set tags=" halve backslashes for escaped
    // space.
    let mut patlen = unsafe { cstr::bytes_at(filepat) }.len();
    let pat = unsafe { xmemdupz(filepat as *const c_void, patlen) } as *mut c_char;
    // Replace "\ " with " ".
    let mut e = unsafe { pat.add(patlen) };
    let mut s = pat;
    while unsafe { *s } as c_int != NUL {
        if unsafe { *s } as c_int == '\\' as c_int {
            let p = unsafe { s.add(1) };
            if unsafe { *p } as c_int == ' ' as c_int {
                let into = s.cast::<u8>();
                unsafe { into.copy_from(p.cast(), e.offset_from(p) as size_t + 1) };
                e = unsafe { e.sub(1) };
            }
        }
        s = unsafe { s.add(1) };
    }
    patlen = unsafe { e.offset_from(pat) } as size_t;

    flags |= ExpandFlags::FILE | ExpandFlags::EXEC | ExpandFlags::SHELLCMD;

    let mut mustfree = false; // Track memory allocation for `path`.
    let mut path;
    if unsafe { *pat } as c_int == '.' as c_int
        && (vim_ispathsep(unsafe { *pat.add(1) } as c_int)
            || (unsafe { *pat.add(1) } as c_int == '.' as c_int
                && vim_ispathsep(unsafe { *pat.add(2) } as c_int)))
    {
        path = c".".as_ptr() as *mut c_char;
    } else {
        // For an absolute name we don't use $PATH.
        path = if unsafe { path_is_absolute(pat) } {
            ptr::null_mut()
        } else {
            unsafe { vim_getenv(c"PATH".as_ptr()) }
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
    unsafe { ga_init(&raw mut ga, size_of::<*mut c_char>() as c_int, 10) };
    let mut found_ht = hashtab_T::init();
    let mut s = path;
    loop {
        // Length of the path portion of buf, including trailing slash.
        let mut pathlen;
        let seplen;

        if unsafe { *s } as c_int == NUL {
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
            e = unsafe { vim_strchr(s, ENV_SEPCHAR) };
            if e.is_null() {
                e = unsafe { s.add(cstr::bytes_at(s).len()) };
            }

            pathlen = unsafe { e.offset_from(s) } as size_t;
            if unsafe { cstr::prefix_eq(s, c".".as_ptr(), pathlen) } {
                did_curdir = true;
                flags |= ExpandFlags::DIR;
            } else {
                // Do not match directories inside a $PATH item.
                flags.clear(ExpandFlags::DIR);
            }

            seplen = if unsafe { after_pathsep(s, e) } == 0 {
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
                unsafe { xmemcpyz(buf as *mut c_void, s as *const c_void, pathlen) };
                if seplen > 0 {
                    unsafe {
                        xmemcpyz(
                            buf.add(pathlen) as *mut c_void,
                            c"/".as_ptr() as *const c_void,
                            PATHSEP_LEN,
                        )
                    };
                    pathlen += seplen;
                }
            }
            unsafe {
                xmemcpyz(
                    buf.add(pathlen) as *mut c_void,
                    pat as *const c_void,
                    patlen,
                )
            };

            unsafe {
                expand_shellcmd_onedir(
                    buf,
                    pathlen,
                    matches,
                    numMatches,
                    flags,
                    &raw mut found_ht,
                    &raw mut ga,
                )
            };
        }

        if unsafe { *e } as c_int != NUL {
            e = unsafe { e.add(1) };
        }
        s = e;
    }
    unsafe { *matches = ga.ga_data as *mut *mut c_char };
    unsafe { *numMatches = ga.ga_len };

    unsafe { xfree(buf as *mut c_void) };
    unsafe { xfree(pat as *mut c_void) };
    if mustfree {
        unsafe { xfree(path as *mut c_void) };
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
    // SAFETY: the caller's contract -- `xp` is the live expansion
    // context, which outlives this call.
    let mut xp = unsafe { Xp::new(xp) };
    let mut args = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VarLock::Unlocked,
        vval: typval_vval_union { v_number: 0 },
    }; 4];
    let save_current_sctx = current_sctx.get();

    if xp.xp_arg.is_null() || unsafe { *xp.xp_arg } as c_int == NUL || xp.xp_line.is_null() {
        return ptr::null_mut();
    }

    // Upstream saves `cmdbuff[cmdlen]` here and puts a NUL in its place
    // for the duration of the callback. The command line's terminator is
    // `CmdBuff`'s invariant now, so the byte it saved is always the NUL it
    // wrote, and both halves are gone.
    let pat = unsafe { xstrnsave(xp.xp_pattern, xp.xp_pattern_len) };
    args[0].v_type = VAR_STRING;
    args[1].v_type = VAR_STRING;
    args[2].v_type = VAR_NUMBER;
    args[3].v_type = VAR_UNKNOWN;
    args[0].vval.v_string = pat;
    args[1].vval.v_string = xp.xp_line;
    args[2].vval.v_number = xp.xp_col as varnumber_T;

    current_sctx.set(xp.xp_script_ctx);

    let ret = unsafe { user_expand_func(xp.xp_arg, 3, args.as_mut_ptr()) };

    current_sctx.set(save_current_sctx);
    unsafe { xfree(pat as *mut c_void) };
    ret
}

/// Expand names with a function defined by the user
/// (`ExpandContext::UserDefined` and `ExpandContext::UserList`).
pub(crate) unsafe fn expand_user_defined(
    pat: *const c_char,
    xp: *mut expand_T,
    regmatch: *mut regmatch_T,
    matches: *mut *mut *mut c_char,
    numMatches: *mut c_int,
) -> Result<(), Failed> {
    // SAFETY: the caller's contract -- `xp` is the live expansion
    // context, which outlives this call.
    let mut xp = unsafe { Xp::new(xp) };
    let fuzzy = unsafe { cmdline_fuzzy_complete(pat) };
    unsafe { *matches = ptr::null_mut() };
    unsafe { *numMatches = 0 };

    let retstr = unsafe { call_user_expand_func(call_func_retstr, xp.raw()) } as *mut c_char;
    if retstr.is_null() {
        return Err(Failed);
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
    unsafe { ga_init(&raw mut ga, itemsize as c_int, 3) };

    // The answer is one match per line.
    let mut s = retstr;
    while unsafe { *s } as c_int != NUL {
        let mut e = unsafe { vim_strchr(s, '\n' as c_int) };
        if e.is_null() {
            e = unsafe { s.add(cstr::bytes_at(s).len()) };
        }
        let keep = unsafe { *e };
        unsafe { *e = NUL as c_char };

        let mut score = 0;
        let matched = if unsafe { *xp.xp_pattern } as c_int == NUL {
            true // match everything
        } else if fuzzy {
            score = unsafe { fuzzy_match_str(s, pat) };
            score != FUZZY_SCORE_NONE
        } else {
            unsafe { vim_regexec(regmatch, s, 0) }
        };

        unsafe { *e = keep };

        if matched {
            let p =
                unsafe { xmemdupz(s as *const c_void, e.offset_from(s) as size_t) } as *mut c_char;

            unsafe { ga_grow(&raw mut ga, 1) };
            if fuzzy {
                let scored = fuzmatch_str_T {
                    idx: ga.ga_len,
                    str: p,
                    score,
                };
                let slot = (ga.ga_data as *mut fuzmatch_str_T).wrapping_offset(ga.ga_len as isize);
                // SAFETY: `ga_grow` above made room for one more entry.
                unsafe { slot.write(scored) };
            } else {
                unsafe {
                    (ga.ga_data as *mut *mut c_char)
                        .offset(ga.ga_len as isize)
                        .write(p)
                };
            }
            ga.ga_len += 1;
        }

        if unsafe { *e } as c_int != NUL {
            e = unsafe { e.add(1) };
        }
        s = e;
    }
    unsafe { xfree(retstr as *mut c_void) };

    if ga.ga_len == 0 {
        return Ok(());
    }

    if fuzzy {
        unsafe {
            fuzzymatches_to_strmatches(ga.ga_data as *mut fuzmatch_str_T, matches, ga.ga_len, false)
        };
    } else {
        unsafe { *matches = ga.ga_data as *mut *mut c_char };
    }
    unsafe { *numMatches = ga.ga_len };
    Ok(())
}

/// Copy the strings of a `customlist,` answer into a fresh match array.
pub(crate) unsafe fn process_user_list(
    retlist: *mut list_T,
    matches: *mut *mut *mut c_char,
    numMatches: *mut c_int,
) {
    let mut ga = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ptr::null_mut(),
    };
    unsafe { ga_init(&raw mut ga, size_of::<*mut c_char>() as c_int, 3) };

    // Loop over the items in the list.
    if !retlist.is_null() {
        let mut li: *const listitem_T = unsafe { (*retlist).lv_first };
        while !li.is_null() {
            // Skip non-string items and empty strings.
            if unsafe { (*li).li_tv.v_type } == VAR_STRING
                && !unsafe { (*li).li_tv.vval.v_string }.is_null()
            {
                let p = unsafe { xstrdup((*li).li_tv.vval.v_string) };
                unsafe { ga_grow(&raw mut ga, 1) };
                unsafe {
                    (ga.ga_data as *mut *mut c_char)
                        .offset(ga.ga_len as isize)
                        .write(p)
                };
                ga.ga_len += 1;
            }
            li = unsafe { (*li).li_next };
        }
    }
    unsafe { tv_list_unref(retlist) };

    unsafe { *matches = ga.ga_data as *mut *mut c_char };
    unsafe { *numMatches = ga.ga_len };
}

/// Expand names with a list returned by a function defined by the user.
pub(crate) unsafe fn expand_user_list(
    xp: *mut expand_T,
    matches: *mut *mut *mut c_char,
    numMatches: *mut c_int,
) -> Result<(), Failed> {
    // SAFETY: the caller's contract -- `xp` is the live expansion
    // context, which outlives this call.
    let mut xp = unsafe { Xp::new(xp) };
    unsafe { *matches = ptr::null_mut() };
    unsafe { *numMatches = 0 };
    let retlist = unsafe { call_user_expand_func(call_func_retlist, xp.raw()) } as *mut list_T;
    if retlist.is_null() {
        return Err(Failed);
    }

    unsafe { process_user_list(retlist, matches, numMatches) };
    Ok(())
}

/// Expand names with a Lua completion function.
pub(crate) unsafe fn expand_user_lua(
    xp: *mut expand_T,
    numMatches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> Result<(), Failed> {
    // SAFETY: the caller's contract -- `xp` is the live expansion
    // context, which outlives this call.
    let mut xp = unsafe { Xp::new(xp) };
    let mut rettv = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VarLock::Unlocked,
        vval: typval_vval_union { v_number: 0 },
    };
    unsafe { nlua_call_user_expand_func(xp.raw(), &raw mut rettv) };
    if rettv.v_type != VAR_LIST {
        unsafe { tv_clear(&raw mut rettv) };
        return Err(Failed);
    }

    unsafe { process_user_list(rettv.vval.v_list, matches, numMatches) };
    Ok(())
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
    let buf = unsafe { xmalloc(MAXPATHL as size_t) } as *mut c_char;

    let mut xpc: expand_T = unsafe { core::mem::zeroed() };
    unsafe { expand_init(&raw mut xpc) };
    xpc.xp_context = if dirs {
        ExpandContext::Directories
    } else {
        ExpandContext::Files
    };

    let filelen = unsafe { cstr::bytes_at(file) }.len();

    // Loop over all entries in {path}.
    let mut path = path;
    while unsafe { *path } as c_int != NUL {
        // Copy one item of the path to buf[] and concatenate the file
        // name.  `pathlen` is the length of the path portion of buf,
        // including the trailing slash.
        let mut pathlen = unsafe {
            copy_option_part(
                &raw mut path,
                buf,
                MAXPATHL as size_t,
                c",".as_ptr() as *mut c_char,
            )
        };
        let seplen = if unsafe { *buf } as c_int != NUL
            && unsafe { after_pathsep(buf, buf.add(pathlen)) } == 0
        {
            PATHSEP_LEN
        } else {
            0
        };

        // Upstream's `+ 1 <= MAXPATHL` — the one byte is the NUL.
        if pathlen + seplen + filelen < MAXPATHL as size_t {
            if seplen > 0 {
                unsafe {
                    xmemcpyz(
                        buf.add(pathlen) as *mut c_void,
                        c"/".as_ptr() as *const c_void,
                        PATHSEP_LEN,
                    )
                };
                pathlen += seplen;
            }
            unsafe {
                xmemcpyz(
                    buf.add(pathlen) as *mut c_void,
                    file as *const c_void,
                    filelen,
                )
            };

            let mut p: *mut *mut c_char = ptr::null_mut();
            let mut num_p = 0;
            let _ = unsafe {
                expand_from_context(
                    &raw mut xpc,
                    buf,
                    &raw mut p,
                    &raw mut num_p,
                    WildOpts::SILENT | expand_options,
                )
            };
            if num_p > 0 {
                unsafe {
                    escape_matches(
                        &raw mut xpc,
                        buf,
                        core::slice::from_raw_parts_mut(p, num_p as usize),
                        WildOpts::SILENT | expand_options,
                    )
                };

                // Concatenate new results to previous ones, taking over
                // the pointers.
                unsafe { ga_grow(ga, num_p) };
                for i in 0..num_p {
                    // SAFETY: `ga_grow` above made room for `num_p` more
                    // pointers, and `p` holds that many.
                    unsafe {
                        let slot = ((*ga).ga_data as *mut *mut c_char)
                            .wrapping_offset((*ga).ga_len as isize);
                        slot.write(*p.offset(i as isize));
                        (*ga).ga_len += 1;
                    }
                }
                unsafe { xfree(p as *mut c_void) };
            }
        }
    }

    unsafe { xfree(buf as *mut c_void) };
}
