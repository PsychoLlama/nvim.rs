//! Finding and sourcing files along 'runtimepath' -- `:runtime` and everything
//! built on it.
//!
//! `do_in_path` is the primitive: split a path list on commas, glob each entry
//! against a pattern, and hand every match to a callback, optionally stopping
//! at the first.  `do_in_path_and_pp` adds 'packpath''s `pack/*/start` and
//! `pack/*/opt` trees for the `DIP_START`/`DIP_OPT` flags, and
//! `do_in_runtimepath` is the 'runtimepath' entry point that prefers the
//! cached search path when there is one (see `cache.rs`).  The
//! `source_runtime*` wrappers pick the callback that sources what was found,
//! with the Vim-then-Lua ordering `:runtime` promises; `runtime_get_named*`
//! and `runtime_inspect` are the API's read-only views of the same search.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn runtime_init() {
    unsafe {
        uv_mutex_init(runtime_search_path_mutex.ptr());
    }
}

unsafe extern "C" fn get_runtime_cmd_flags(
    mut argp: *mut *mut ::core::ffi::c_char,
    mut where_len: size_t,
) -> ::core::ffi::c_int {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = *argp;
        if where_len == 0 as size_t {
            return 0 as ::core::ffi::c_int;
        }
        if strncmp(arg, c"START".as_ptr(), where_len) == 0 as ::core::ffi::c_int {
            *argp = skipwhite(arg.add(where_len));
            return DIP_START as ::core::ffi::c_int + DIP_NORTP as ::core::ffi::c_int;
        }
        if strncmp(arg, c"OPT".as_ptr(), where_len) == 0 as ::core::ffi::c_int {
            *argp = skipwhite(arg.add(where_len));
            return DIP_OPT as ::core::ffi::c_int + DIP_NORTP as ::core::ffi::c_int;
        }
        if strncmp(arg, c"PACK".as_ptr(), where_len) == 0 as ::core::ffi::c_int {
            *argp = skipwhite(arg.add(where_len));
            return DIP_START as ::core::ffi::c_int
                + DIP_OPT as ::core::ffi::c_int
                + DIP_NORTP as ::core::ffi::c_int;
        }
        if strncmp(arg, c"ALL".as_ptr(), where_len) == 0 as ::core::ffi::c_int {
            *argp = skipwhite(arg.add(where_len));
            return DIP_START as ::core::ffi::c_int + DIP_OPT as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
}

pub unsafe fn ex_runtime(mut eap: *mut exarg_T) {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        let mut flags: ::core::ffi::c_int = if (*eap).forceit != 0 {
            DIP_ALL as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
        let mut p: *mut ::core::ffi::c_char = skiptowhite(arg);
        flags += get_runtime_cmd_flags(&raw mut arg, p.offset_from(arg) as size_t);
        debug_assert!(!arg.is_null(), "arg != NULL");
        source_runtime(arg, flags);
    }
}

pub unsafe extern "C" fn set_context_in_runtime_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = skiptowhite(arg);
        runtime_expand_flags.set(if *p as ::core::ffi::c_int != NUL {
            get_runtime_cmd_flags(
                &raw mut arg as *mut *mut ::core::ffi::c_char,
                p.offset_from(arg) as size_t,
            )
        } else {
            0 as ::core::ffi::c_int
        });
        loop {
            p = skiptowhite_esc(arg);
            if *p as ::core::ffi::c_int == NUL {
                break;
            }
            if runtime_expand_flags.get() == 0 as ::core::ffi::c_int {
                runtime_expand_flags.set(DIP_ALL as ::core::ffi::c_int);
            }
            arg = skipwhite(p);
        }
        (*xp).xp_context = EXPAND_RUNTIME;
        (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
    }
}

pub(crate) unsafe extern "C" fn source_callback_vim_lua(
    mut num_fnames: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut all: bool,
    mut cookie: *mut ::core::ffi::c_void,
) -> bool {
    unsafe {
        let mut did_one: bool = false_0 != 0;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < num_fnames {
            if path_with_extension(*fnames.offset(i as isize), c"vim".as_ptr()) {
                do_source(
                    *fnames.offset(i as isize),
                    false_0 != 0,
                    DOSO_NONE,
                    cookie as *mut ::core::ffi::c_int,
                );
                did_one = true_0 != 0;
                if !all {
                    return true_0 != 0;
                }
            }
            i += 1;
        }
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < num_fnames {
            if path_with_extension(*fnames.offset(i_0 as isize), c"lua".as_ptr()) {
                do_source(
                    *fnames.offset(i_0 as isize),
                    false_0 != 0,
                    DOSO_NONE,
                    cookie as *mut ::core::ffi::c_int,
                );
                did_one = true_0 != 0;
                if !all {
                    return true_0 != 0;
                }
            }
            i_0 += 1;
        }
        return did_one;
    }
}

pub(crate) unsafe extern "C" fn source_callback(
    mut num_fnames: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut all: bool,
    mut cookie: *mut ::core::ffi::c_void,
) -> bool {
    unsafe {
        let mut did_one: bool = source_callback_vim_lua(num_fnames, fnames, all, cookie);
        if !all && did_one as ::core::ffi::c_int != 0 {
            return true_0 != 0;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < num_fnames {
            if !path_with_extension(*fnames.offset(i as isize), c"vim".as_ptr())
                && !path_with_extension(*fnames.offset(i as isize), c"lua".as_ptr())
            {
                do_source(
                    *fnames.offset(i as isize),
                    false_0 != 0,
                    DOSO_NONE,
                    cookie as *mut ::core::ffi::c_int,
                );
                did_one = true_0 != 0;
                if !all {
                    return true_0 != 0;
                }
            }
            i += 1;
        }
        return did_one;
    }
}

pub unsafe extern "C" fn do_in_path(
    mut path: *const ::core::ffi::c_char,
    mut prefix: *const ::core::ffi::c_char,
    mut name: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut callback: DoInRuntimepathCB,
    mut cookie: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        let mut did_one: bool = false_0 != 0;
        let mut rtp_copy: *mut ::core::ffi::c_char = xstrdup(path);
        let mut buf: *mut ::core::ffi::c_char =
            xmallocz(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
        let mut tail: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if p_verbose.get() > 10 as OptInt && !name.is_null() {
            verbose_enter();
            if *prefix as ::core::ffi::c_int != NUL {
                smsg_c!(
                    0 as ::core::ffi::c_int,
                    gettext(c"Searching for \"%s\" under \"%s\" in \"%s\"".as_ptr()),
                    name,
                    prefix,
                    path,
                );
            } else {
                smsg_c!(
                    0 as ::core::ffi::c_int,
                    gettext(c"Searching for \"%s\" in \"%s\"".as_ptr()),
                    name,
                    path,
                );
            }
            verbose_leave();
        }
        let mut do_all: bool = flags & DIP_ALL as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
        let mut rtp: *mut ::core::ffi::c_char = rtp_copy;
        while *rtp as ::core::ffi::c_int != NUL && (do_all as ::core::ffi::c_int != 0 || !did_one) {
            let mut buflen: size_t = copy_option_part(
                &raw mut rtp,
                buf,
                MAXPATHL as size_t,
                c",".as_ptr() as *mut ::core::ffi::c_char,
            );
            if flags & (DIP_NOAFTER as ::core::ffi::c_int | DIP_AFTER as ::core::ffi::c_int) != 0 {
                let mut is_after: bool = path_is_after(buf, buflen);
                if is_after as ::core::ffi::c_int != 0
                    && flags & DIP_NOAFTER as ::core::ffi::c_int != 0
                    || !is_after && flags & DIP_AFTER as ::core::ffi::c_int != 0
                {
                    continue;
                }
            }
            if name.is_null() {
                Some(callback.expect("non-null function pointer"))
                    .expect("non-null function pointer")(
                    1 as ::core::ffi::c_int,
                    &raw mut buf,
                    do_all,
                    cookie,
                );
                did_one = true_0 != 0;
            } else if buflen
                .wrapping_add(2 as size_t)
                .wrapping_add(strlen(prefix))
                .wrapping_add(strlen(name))
                < MAXPATHL as size_t
            {
                add_pathsep(buf);
                strcat(buf, prefix);
                tail = buf.add(strlen(buf));
                let mut np: *mut ::core::ffi::c_char = name;
                while *np as ::core::ffi::c_int != NUL
                    && (do_all as ::core::ffi::c_int != 0 || !did_one)
                {
                    debug_assert!(
                        4096_isize >= tail.offset_from(buf),
                        "MAXPATHL >= (tail - buf)"
                    );
                    copy_option_part(
                        &raw mut np,
                        tail,
                        (MAXPATHL as isize - tail.offset_from(buf)) as size_t,
                        c"\t ".as_ptr() as *mut ::core::ffi::c_char,
                    );
                    if p_verbose.get() > 10 as OptInt {
                        verbose_enter();
                        smsg_c!(
                            0 as ::core::ffi::c_int,
                            gettext(c"Searching for \"%s\"".as_ptr()),
                            buf,
                        );
                        verbose_leave();
                    }
                    let mut ew_flags: ::core::ffi::c_int =
                        (if flags & DIP_DIR as ::core::ffi::c_int != 0 {
                            EW_DIR
                        } else {
                            EW_FILE
                        }) | (if flags & DIP_DIRFILE as ::core::ffi::c_int != 0 {
                            EW_DIR | EW_FILE
                        } else {
                            0 as ::core::ffi::c_int
                        });
                    did_one = did_one as ::core::ffi::c_int
                        | (gen_expand_wildcards_and_cb(
                            1 as ::core::ffi::c_int,
                            &raw mut buf,
                            ew_flags,
                            do_all,
                            callback,
                            cookie,
                        ) == OK) as ::core::ffi::c_int
                        != 0;
                }
            }
        }
        xfree(buf as *mut ::core::ffi::c_void);
        xfree(rtp_copy as *mut ::core::ffi::c_void);
        if !did_one && !name.is_null() {
            let mut basepath: *mut ::core::ffi::c_char =
                (if path == p_rtp.get() as *const ::core::ffi::c_char {
                    c"runtimepath".as_ptr()
                } else {
                    c"packpath".as_ptr()
                }) as *mut ::core::ffi::c_char;
            if flags & DIP_ERR as ::core::ffi::c_int != 0 {
                semsg_c!(
                    gettext(&raw const e_dirnotf as *const ::core::ffi::c_char),
                    basepath,
                    name,
                );
            } else if p_verbose.get() > 1 as OptInt {
                verbose_enter();
                smsg_c!(
                    0 as ::core::ffi::c_int,
                    gettext(c"not found in '%s': \"%s\"".as_ptr()),
                    basepath,
                    name,
                );
                verbose_leave();
            }
        }
        return if did_one as ::core::ffi::c_int != 0 {
            OK
        } else {
            FAIL
        };
    }
}

pub unsafe extern "C" fn runtime_inspect(mut arena: *mut Arena) -> Array {
    unsafe {
        let mut path: RuntimeSearchPath = runtime_search_path.get();
        let mut rv: Array = arena_array(arena, path.size);
        let mut i: size_t = 0 as size_t;
        while i < path.size {
            let mut item: *mut SearchPathItem = path.items.add(i);
            let mut entry: Dict = arena_dict(arena, 5 as size_t);
            let c2rust_fresh8 = entry.size;
            entry.size = entry.size.wrapping_add(1);
            *entry.items.add(c2rust_fresh8) = key_value_pair {
                key: cstr_as_string(c"path".as_ptr()),
                value: object {
                    type_0: kObjectTypeString,
                    data: object_data {
                        string: cstr_as_string((*item).path),
                    },
                },
            };
            if (*item).after {
                let c2rust_fresh9 = entry.size;
                entry.size = entry.size.wrapping_add(1);
                *entry.items.add(c2rust_fresh9) = key_value_pair {
                    key: cstr_as_string(c"after".as_ptr()),
                    value: object {
                        type_0: kObjectTypeBoolean,
                        data: object_data { boolean: true },
                    },
                };
            }
            if (*item).pack_inserted {
                let c2rust_fresh10 = entry.size;
                entry.size = entry.size.wrapping_add(1);
                *entry.items.add(c2rust_fresh10) = key_value_pair {
                    key: cstr_as_string(c"pack_inserted".as_ptr()),
                    value: object {
                        type_0: kObjectTypeBoolean,
                        data: object_data { boolean: true },
                    },
                };
            }
            if (*item).has_lua as ::core::ffi::c_int != kNone as ::core::ffi::c_int {
                let c2rust_fresh11 = entry.size;
                entry.size = entry.size.wrapping_add(1);
                *entry.items.add(c2rust_fresh11) = key_value_pair {
                    key: cstr_as_string(c"has_lua".as_ptr()),
                    value: object {
                        type_0: kObjectTypeBoolean,
                        data: object_data {
                            boolean: (*item).has_lua as ::core::ffi::c_int
                                == kTrue as ::core::ffi::c_int,
                        },
                    },
                };
            }
            let c2rust_fresh12 = entry.size;
            entry.size = entry.size.wrapping_add(1);
            *entry.items.add(c2rust_fresh12) = key_value_pair {
                key: cstr_as_string(c"pos_in_rtp".as_ptr()),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: object_data {
                        integer: (*item).pos_in_rtp as Integer,
                    },
                },
            };
            let c2rust_fresh13 = rv.size;
            rv.size = rv.size.wrapping_add(1);
            *rv.items.add(c2rust_fresh13) = object {
                type_0: kObjectTypeDict,
                data: object_data { dict: entry },
            };
            i = i.wrapping_add(1);
        }
        return rv;
    }
}

pub unsafe extern "C" fn runtime_get_named(
    mut lua: bool,
    mut pat: Array,
    mut all: bool,
    mut arena: *mut Arena,
) -> Array {
    unsafe {
        let mut ref_0: ::core::ffi::c_int = 0;
        let mut path: RuntimeSearchPath = runtime_search_path_get_cached(&raw mut ref_0);
        static buf: GlobalCell<[::core::ffi::c_char; 4096]> = GlobalCell::new([0; 4096]);
        let mut rv: Array = runtime_get_named_common(
            lua,
            pat,
            all,
            path,
            buf.ptr() as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
            arena,
        );
        runtime_search_path_unref(path, &raw mut ref_0);
        return rv;
    }
}

pub unsafe extern "C" fn runtime_get_named_thread(
    mut lua: bool,
    mut pat: Array,
    mut all: bool,
) -> Array {
    unsafe {
        uv_mutex_lock(runtime_search_path_mutex.ptr());
        static buf: GlobalCell<[::core::ffi::c_char; 4096]> = GlobalCell::new([0; 4096]);
        let mut rv: Array = runtime_get_named_common(
            lua,
            pat,
            all,
            runtime_search_path_thread.get(),
            buf.ptr() as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
            ::core::ptr::null_mut::<Arena>(),
        );
        uv_mutex_unlock(runtime_search_path_mutex.ptr());
        return rv;
    }
}

unsafe extern "C" fn runtime_get_named_common(
    mut lua: bool,
    mut pat: Array,
    mut all: bool,
    mut path: RuntimeSearchPath,
    mut buf: *mut ::core::ffi::c_char,
    mut buf_len: size_t,
    mut arena: *mut Arena,
) -> Array {
    unsafe {
        let mut rv: Array = arena_array(arena, path.size.wrapping_mul(pat.size));
        let mut i: size_t = 0 as size_t;
        '_done: while i < path.size {
            let mut item: *mut SearchPathItem = path.items.add(i);
            's_6: {
                if lua {
                    if (*item).has_lua as ::core::ffi::c_int == kNone as ::core::ffi::c_int {
                        let mut size: size_t =
                            snprintf(buf, buf_len, c"%s/lua/".as_ptr(), (*item).path) as size_t;
                        (*item).has_lua =
                            (size < buf_len && os_isdir(buf) as ::core::ffi::c_int != 0)
                                as ::core::ffi::c_int as TriState;
                    }
                    if (*item).has_lua as ::core::ffi::c_int == kFalse as ::core::ffi::c_int {
                        break 's_6;
                    }
                }
                let mut j: size_t = 0 as size_t;
                loop {
                    if j >= pat.size {
                        break 's_6;
                    }
                    let mut pat_item: Object = *pat.items.add(j);
                    if pat_item.type_0 as ::core::ffi::c_uint
                        == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        let mut size_0: size_t = snprintf(
                            buf,
                            buf_len,
                            c"%s/%s".as_ptr(),
                            (*item).path,
                            pat_item.data.string.data,
                        ) as size_t;
                        if size_0 < buf_len {
                            if os_file_is_readable(buf) {
                                let c2rust_fresh14 = rv.size;
                                rv.size = rv.size.wrapping_add(1);
                                *rv.items.add(c2rust_fresh14) = object {
                                    type_0: kObjectTypeString,
                                    data: object_data {
                                        string: arena_string(arena, cstr_as_string(buf)),
                                    },
                                };
                                if !all {
                                    break '_done;
                                }
                            }
                        }
                    }
                    j = j.wrapping_add(1);
                }
            }
            i = i.wrapping_add(1);
        }
        return rv;
    }
}

pub unsafe extern "C" fn do_in_path_and_pp(
    mut path: *mut ::core::ffi::c_char,
    mut name: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut callback: DoInRuntimepathCB,
    mut cookie: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        let mut done: ::core::ffi::c_int = FAIL;
        if flags & DIP_NORTP as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            done |= do_in_path(
                path,
                c"".as_ptr(),
                if !name.is_null() && *name == 0 {
                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                } else {
                    name
                },
                flags,
                callback,
                cookie,
            );
        }
        if (done == FAIL || flags & DIP_ALL as ::core::ffi::c_int != 0)
            && flags & DIP_START as ::core::ffi::c_int != 0
        {
            let mut prefix: *const ::core::ffi::c_char =
                if flags & DIP_AFTER as ::core::ffi::c_int != 0 {
                    c"pack/*/start/*/after/".as_ptr()
                } else {
                    c"pack/*/start/*/".as_ptr()
                };
            done |= do_in_path(
                p_pp.get(),
                prefix,
                name,
                flags & !(DIP_AFTER as ::core::ffi::c_int),
                callback,
                cookie,
            );
            if done == FAIL || flags & DIP_ALL as ::core::ffi::c_int != 0 {
                prefix = if flags & DIP_AFTER as ::core::ffi::c_int != 0 {
                    c"start/*/after/".as_ptr()
                } else {
                    c"start/*/".as_ptr()
                };
                done |= do_in_path(
                    p_pp.get(),
                    prefix,
                    name,
                    flags & !(DIP_AFTER as ::core::ffi::c_int),
                    callback,
                    cookie,
                );
            }
        }
        if (done == FAIL || flags & DIP_ALL as ::core::ffi::c_int != 0)
            && flags & DIP_OPT as ::core::ffi::c_int != 0
        {
            done |= do_in_path(
                p_pp.get(),
                c"pack/*/opt/*/".as_ptr(),
                name,
                flags,
                callback,
                cookie,
            );
            if done == FAIL || flags & DIP_ALL as ::core::ffi::c_int != 0 {
                done |= do_in_path(
                    p_pp.get(),
                    c"opt/*/".as_ptr(),
                    name,
                    flags,
                    callback,
                    cookie,
                );
            }
        }
        return done;
    }
}

pub unsafe extern "C" fn do_in_runtimepath(
    mut name: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut callback: DoInRuntimepathCB,
    mut cookie: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        let mut success: ::core::ffi::c_int = FAIL;
        if flags & DIP_NORTP as ::core::ffi::c_int == 0 {
            success |= do_in_cached_path(
                if !name.is_null() && *name == 0 {
                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                } else {
                    name
                },
                flags,
                callback,
                cookie,
            );
            flags = flags & !(DIP_START as ::core::ffi::c_int) | DIP_NORTP as ::core::ffi::c_int;
        }
        if flags & (DIP_START as ::core::ffi::c_int | DIP_OPT as ::core::ffi::c_int) != 0
            && (success == FAIL || flags & DIP_ALL as ::core::ffi::c_int != 0)
        {
            success |= do_in_path_and_pp(p_rtp.get(), name, flags, callback, cookie);
        }
        return success;
    }
}

pub unsafe extern "C" fn source_runtime(
    mut name: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        return do_in_runtimepath(
            name,
            flags,
            Some(
                source_callback
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *mut ::core::ffi::c_char,
                        bool,
                        *mut ::core::ffi::c_void,
                    ) -> bool,
            ),
            NULL_0,
        );
    }
}

pub unsafe extern "C" fn source_runtime_vim_lua(
    mut name: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        return do_in_runtimepath(
            name,
            flags,
            Some(
                source_callback_vim_lua
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *mut ::core::ffi::c_char,
                        bool,
                        *mut ::core::ffi::c_void,
                    ) -> bool,
            ),
            NULL_0,
        );
    }
}

pub unsafe extern "C" fn source_in_path_vim_lua(
    mut path: *mut ::core::ffi::c_char,
    mut name: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        return do_in_path_and_pp(
            path,
            name,
            flags,
            Some(
                source_callback_vim_lua
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *mut ::core::ffi::c_char,
                        bool,
                        *mut ::core::ffi::c_void,
                    ) -> bool,
            ),
            NULL_0,
        );
    }
}

pub(crate) unsafe extern "C" fn gen_expand_wildcards_and_cb(
    mut num_pat: ::core::ffi::c_int,
    mut pats: *mut *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut all: bool,
    mut callback: DoInRuntimepathCB,
    mut cookie: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        let mut num_files: ::core::ffi::c_int = 0;
        let mut files: *mut *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        if gen_expand_wildcards(num_pat, pats, &raw mut num_files, &raw mut files, flags) != OK {
            return FAIL;
        }
        Some(callback.expect("non-null function pointer")).expect("non-null function pointer")(
            num_files, files, all, cookie,
        );
        FreeWild(num_files, files);
        return OK;
    }
}
