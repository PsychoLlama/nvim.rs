//! Setting up a downward-and-upward file search.
//!
//! [`vim_findfile_init`] takes one entry of `'path'`, `'tags'` or
//! `'cdpath'` apart into the fixed leading part, the wildcard tail, and the
//! directory the search starts from, then pushes the first directory onto
//! the context's stack for [`vim_findfile`](super::vim_findfile) to walk.
//! The `**` wildcard's depth limiter is parsed here — `**3` is stored as
//! `**` followed by a byte holding 3 — and so is the `;` that asks for the
//! upward search ([`vim_findfile_stopdir`]).

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn vim_findfile_init(
    mut path: *mut ::core::ffi::c_char,
    mut filename: *mut ::core::ffi::c_char,
    mut filenamelen: size_t,
    mut stopdirs: *mut ::core::ffi::c_char,
    mut level: ::core::ffi::c_int,
    mut free_visited: ::core::ffi::c_int,
    mut find_what: ::core::ffi::c_int,
    mut search_ctx_arg: *mut ::core::ffi::c_void,
    mut tagfile: ::core::ffi::c_int,
    mut rel_fname: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_void {
    unsafe {
        let mut wc_part: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut add_sep: bool = false;
        let mut sptr: *mut ff_stack_T = ::core::ptr::null_mut::<ff_stack_T>();
        let mut search_ctx: *mut ff_search_ctx_T = ::core::ptr::null_mut::<ff_search_ctx_T>();
        if !search_ctx_arg.is_null() {
            search_ctx = search_ctx_arg as *mut ff_search_ctx_T;
        } else {
            search_ctx = xcalloc(1 as size_t, ::core::mem::size_of::<ff_search_ctx_T>())
                as *mut ff_search_ctx_T;
        }
        (*search_ctx).ffsc_find_what = find_what;
        (*search_ctx).ffsc_tagfile = tagfile;
        ff_clear(search_ctx);
        '_error_return: {
            if free_visited == true_0 {
                vim_findfile_free_visited(search_ctx as *mut ::core::ffi::c_void);
            } else {
                (*search_ctx).ffsc_visited_list = ff_get_visited_list(
                    filename,
                    filenamelen,
                    &raw mut (*search_ctx).ffsc_visited_lists_list,
                );
                if (*search_ctx).ffsc_visited_list.is_null() {
                    break '_error_return;
                } else {
                    (*search_ctx).ffsc_dir_visited_list = ff_get_visited_list(
                        filename,
                        filenamelen,
                        &raw mut (*search_ctx).ffsc_dir_visited_lists_list,
                    );
                    if (*search_ctx).ffsc_dir_visited_list.is_null() {
                        break '_error_return;
                    }
                }
            }
            if (*ff_expand_buffer.ptr()).data.is_null() {
                (*ff_expand_buffer.ptr()).size = 0 as size_t;
                (*ff_expand_buffer.ptr()).data =
                    xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
            }
            if *path.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
                && (vim_ispathsep(
                    *path.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                ) as ::core::ffi::c_int
                    != 0
                    || *path.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL)
                && (tagfile == 0 || vim_strchr(p_cpo.get(), CPO_DOTTAG).is_null())
                && !rel_fname.is_null()
            {
                let mut len: size_t = path_tail(rel_fname).offset_from(rel_fname) as size_t;
                if !vim_isAbsName(rel_fname) && len.wrapping_add(1 as size_t) < MAXPATHL as size_t {
                    xmemcpyz(
                        (*ff_expand_buffer.ptr()).data as *mut ::core::ffi::c_void,
                        rel_fname as *const ::core::ffi::c_void,
                        len,
                    );
                    (*ff_expand_buffer.ptr()).size = len;
                    (*search_ctx).ffsc_start_dir =
                        cstr_as_string(FullName_save((*ff_expand_buffer.ptr()).data, false_0 != 0));
                } else {
                    (*search_ctx).ffsc_start_dir = cbuf_to_string(rel_fname, len);
                }
                path = path.offset(1);
                if *path as ::core::ffi::c_int != NUL {
                    path = path.offset(1);
                }
            } else if *path as ::core::ffi::c_int == NUL || !vim_isAbsName(path) {
                if os_dirname((*ff_expand_buffer.ptr()).data, MAXPATHL as size_t) == FAIL {
                    break '_error_return;
                } else {
                    (*ff_expand_buffer.ptr()).size = strlen((*ff_expand_buffer.ptr()).data);
                    (*search_ctx).ffsc_start_dir =
                        copy_string(ff_expand_buffer.get(), ::core::ptr::null_mut::<Arena>());
                }
            }
            if !stopdirs.is_null() {
                let mut walker: *mut ::core::ffi::c_char = stopdirs;
                while *walker as ::core::ffi::c_int == ';' as ::core::ffi::c_int {
                    walker = walker.offset(1);
                }
                let mut dircount: size_t = 1 as size_t;
                (*search_ctx).ffsc_stopdirs_v =
                    xmalloc(::core::mem::size_of::<String_0>()) as *mut String_0;
                loop {
                    let mut helper: *mut ::core::ffi::c_char = walker;
                    let mut ptr: *mut ::core::ffi::c_void = xrealloc(
                        (*search_ctx).ffsc_stopdirs_v as *mut ::core::ffi::c_void,
                        dircount
                            .wrapping_add(1 as size_t)
                            .wrapping_mul(::core::mem::size_of::<String_0>()),
                    );
                    (*search_ctx).ffsc_stopdirs_v = ptr as *mut String_0;
                    walker = vim_strchr(walker, ';' as ::core::ffi::c_int);
                    '_c2rust_label: {
                        if walker.is_null() || walker.offset_from(helper) >= 0 as isize {
                        } else {
                            __assert_fail(
                            b"!walker || walker - helper >= 0\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            b"src/nvim/file_search.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            359 as ::core::ffi::c_uint,
                            b"void *vim_findfile_init(char *, char *, size_t, char *, int, int, int, void *, int, char *)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                        }
                    };
                    let mut len_0: size_t = if !walker.is_null() {
                        walker.offset_from(helper) as size_t
                    } else {
                        strlen(helper)
                    };
                    if *helper as ::core::ffi::c_int != NUL
                        && !vim_isAbsName(helper)
                        && len_0.wrapping_add(1 as size_t) < MAXPATHL as size_t
                    {
                        xmemcpyz(
                            (*ff_expand_buffer.ptr()).data as *mut ::core::ffi::c_void,
                            helper as *const ::core::ffi::c_void,
                            len_0,
                        );
                        (*ff_expand_buffer.ptr()).size = len_0;
                        *(*search_ctx)
                            .ffsc_stopdirs_v
                            .offset(dircount.wrapping_sub(1 as size_t) as isize) =
                            cstr_as_string(FullName_save(helper, len_0 != 0));
                    } else {
                        *(*search_ctx)
                            .ffsc_stopdirs_v
                            .offset(dircount.wrapping_sub(1 as size_t) as isize) =
                            cbuf_to_string(helper, len_0);
                    }
                    if !walker.is_null() {
                        walker = walker.offset(1);
                    }
                    dircount = dircount.wrapping_add(1);
                    if walker.is_null() {
                        break;
                    }
                }
                *(*search_ctx)
                    .ffsc_stopdirs_v
                    .offset(dircount.wrapping_sub(1 as size_t) as isize) = NULL_STRING;
            }
            (*search_ctx).ffsc_level = level;
            wc_part = vim_strchr(path, '*' as ::core::ffi::c_int);
            if !wc_part.is_null() {
                let mut llevel: int64_t = 0;
                let mut errpt: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                '_c2rust_label_0: {
                    if wc_part.offset_from(path) >= 0 as isize {
                    } else {
                        __assert_fail(
                        b"wc_part - path >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/file_search.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        390 as ::core::ffi::c_uint,
                        b"void *vim_findfile_init(char *, char *, size_t, char *, int, int, int, void *, int, char *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                    }
                };
                (*search_ctx).ffsc_fix_path =
                    cbuf_to_string(path, wc_part.offset_from(path) as size_t);
                (*ff_expand_buffer.ptr()).size = 0 as size_t;
                while *wc_part as ::core::ffi::c_int != NUL {
                    if (*ff_expand_buffer.ptr()).size.wrapping_add(5 as size_t)
                        >= MAXPATHL as size_t
                    {
                        emsg(gettext(
                            (e_path_too_long_for_completion.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        ));
                        break;
                    } else if strncmp(
                        wc_part,
                        b"**\0".as_ptr() as *const ::core::ffi::c_char,
                        2 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        let c2rust_fresh0 = wc_part;
                        wc_part = wc_part.offset(1);
                        let c2rust_fresh1 = (*ff_expand_buffer.ptr()).size;
                        (*ff_expand_buffer.ptr()).size =
                            (*ff_expand_buffer.ptr()).size.wrapping_add(1);
                        *(*ff_expand_buffer.ptr())
                            .data
                            .offset(c2rust_fresh1 as isize) = *c2rust_fresh0;
                        let c2rust_fresh2 = wc_part;
                        wc_part = wc_part.offset(1);
                        let c2rust_fresh3 = (*ff_expand_buffer.ptr()).size;
                        (*ff_expand_buffer.ptr()).size =
                            (*ff_expand_buffer.ptr()).size.wrapping_add(1);
                        *(*ff_expand_buffer.ptr())
                            .data
                            .offset(c2rust_fresh3 as isize) = *c2rust_fresh2;
                        llevel =
                            strtol(wc_part, &raw mut errpt, 10 as ::core::ffi::c_int) as int64_t;
                        if errpt != wc_part && llevel > 0 as int64_t && llevel < 255 as int64_t {
                            let c2rust_fresh4 = (*ff_expand_buffer.ptr()).size;
                            (*ff_expand_buffer.ptr()).size =
                                (*ff_expand_buffer.ptr()).size.wrapping_add(1);
                            *(*ff_expand_buffer.ptr())
                                .data
                                .offset(c2rust_fresh4 as isize) = llevel as ::core::ffi::c_char;
                        } else if errpt != wc_part && llevel == 0 as int64_t {
                            (*ff_expand_buffer.ptr()).size =
                                (*ff_expand_buffer.ptr()).size.wrapping_sub(2 as size_t);
                        } else {
                            let c2rust_fresh5 = (*ff_expand_buffer.ptr()).size;
                            (*ff_expand_buffer.ptr()).size =
                                (*ff_expand_buffer.ptr()).size.wrapping_add(1);
                            *(*ff_expand_buffer.ptr())
                                .data
                                .offset(c2rust_fresh5 as isize) =
                                FF_MAX_STAR_STAR_EXPAND as ::core::ffi::c_char;
                        }
                        wc_part = errpt;
                        if !(*wc_part as ::core::ffi::c_int != NUL
                            && !vim_ispathsep(*wc_part as ::core::ffi::c_int))
                        {
                            continue;
                        }
                        semsg(
                        gettext(
                            b"E343: Invalid path: '**[number]' must be at the end of the path or be followed by '%s'.\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        PATHSEPSTR.as_ptr(),
                    );
                        break '_error_return;
                    } else {
                        let c2rust_fresh6 = wc_part;
                        wc_part = wc_part.offset(1);
                        let c2rust_fresh7 = (*ff_expand_buffer.ptr()).size;
                        (*ff_expand_buffer.ptr()).size =
                            (*ff_expand_buffer.ptr()).size.wrapping_add(1);
                        *(*ff_expand_buffer.ptr())
                            .data
                            .offset(c2rust_fresh7 as isize) = *c2rust_fresh6;
                    }
                }
                *(*ff_expand_buffer.ptr())
                    .data
                    .offset((*ff_expand_buffer.ptr()).size as isize) = NUL as ::core::ffi::c_char;
                (*search_ctx).ffsc_wc_path =
                    copy_string(ff_expand_buffer.get(), ::core::ptr::null_mut::<Arena>());
            } else {
                (*search_ctx).ffsc_fix_path = cstr_to_string(path);
            }
            if (*search_ctx).ffsc_start_dir.data.is_null() {
                (*search_ctx).ffsc_start_dir = copy_string(
                    (*search_ctx).ffsc_fix_path,
                    ::core::ptr::null_mut::<Arena>(),
                );
                *(*search_ctx)
                    .ffsc_fix_path
                    .data
                    .offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
                (*search_ctx).ffsc_fix_path.size = 0 as size_t;
            }
            if (*search_ctx)
                .ffsc_start_dir
                .size
                .wrapping_add((*search_ctx).ffsc_fix_path.size)
                .wrapping_add(3 as size_t)
                >= MAXPATHL as size_t
            {
                emsg(gettext(
                    (e_path_too_long_for_completion.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ));
            } else {
                add_sep = after_pathsep(
                    (*search_ctx).ffsc_start_dir.data,
                    (*search_ctx)
                        .ffsc_start_dir
                        .data
                        .offset((*search_ctx).ffsc_start_dir.size as isize),
                ) == 0;
                (*ff_expand_buffer.ptr()).size = vim_snprintf(
                    (*ff_expand_buffer.ptr()).data,
                    MAXPATHL as size_t,
                    b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                    (*search_ctx).ffsc_start_dir.data,
                    if add_sep as ::core::ffi::c_int != 0 {
                        PATHSEPSTR.as_ptr()
                    } else {
                        b"\0".as_ptr() as *const ::core::ffi::c_char
                    },
                ) as size_t;
                '_c2rust_label_1: {
                    if (*ff_expand_buffer.ptr()).size < 4096 as size_t {
                    } else {
                        __assert_fail(
                        b"ff_expand_buffer.size < MAXPATHL\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/file_search.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        458 as ::core::ffi::c_uint,
                        b"void *vim_findfile_init(char *, char *, size_t, char *, int, int, int, void *, int, char *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                    }
                };
                let mut bufsize: size_t = (*ff_expand_buffer.ptr())
                    .size
                    .wrapping_add((*search_ctx).ffsc_fix_path.size)
                    .wrapping_add(1 as size_t);
                let mut buf: *mut ::core::ffi::c_char =
                    xmalloc(bufsize) as *mut ::core::ffi::c_char;
                vim_snprintf(
                    buf,
                    bufsize,
                    b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                    (*ff_expand_buffer.ptr()).data,
                    (*search_ctx).ffsc_fix_path.data,
                );
                if os_isdir(buf) {
                    if (*search_ctx).ffsc_fix_path.size > 0 as size_t {
                        add_sep = after_pathsep(
                            (*search_ctx).ffsc_fix_path.data,
                            (*search_ctx)
                                .ffsc_fix_path
                                .data
                                .offset((*search_ctx).ffsc_fix_path.size as isize),
                        ) == 0;
                        (*ff_expand_buffer.ptr()).size = (*ff_expand_buffer.ptr())
                            .size
                            .wrapping_add(vim_snprintf(
                                (*ff_expand_buffer.ptr())
                                    .data
                                    .offset((*ff_expand_buffer.ptr()).size as isize),
                                (MAXPATHL as size_t).wrapping_sub((*ff_expand_buffer.ptr()).size),
                                b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                                (*search_ctx).ffsc_fix_path.data,
                                if add_sep as ::core::ffi::c_int != 0 {
                                    PATHSEPSTR.as_ptr()
                                } else {
                                    b"\0".as_ptr() as *const ::core::ffi::c_char
                                },
                            ) as size_t);
                        '_c2rust_label_2: {
                            if (*ff_expand_buffer.ptr()).size < 4096 as size_t {
                            } else {
                                __assert_fail(
                                b"ff_expand_buffer.size < MAXPATHL\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/file_search.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                478 as ::core::ffi::c_uint,
                                b"void *vim_findfile_init(char *, char *, size_t, char *, int, int, int, void *, int, char *)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                            }
                        };
                    }
                } else {
                    let mut p: *mut ::core::ffi::c_char =
                        path_tail((*search_ctx).ffsc_fix_path.data);
                    let mut len_1: ::core::ffi::c_int =
                        (*search_ctx).ffsc_fix_path.size as ::core::ffi::c_int;
                    if p > (*search_ctx).ffsc_fix_path.data {
                        len_1 = p.offset_from((*search_ctx).ffsc_fix_path.data)
                            as ::core::ffi::c_int
                            - 1 as ::core::ffi::c_int;
                        if len_1 >= 2 as ::core::ffi::c_int
                            && strncmp(
                                (*search_ctx).ffsc_fix_path.data,
                                b"..\0".as_ptr() as *const ::core::ffi::c_char,
                                2 as size_t,
                            ) == 0 as ::core::ffi::c_int
                            && (len_1 == 2 as ::core::ffi::c_int
                                || *(*search_ctx)
                                    .ffsc_fix_path
                                    .data
                                    .offset(2 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == PATHSEP)
                        {
                            xfree(buf as *mut ::core::ffi::c_void);
                            break '_error_return;
                        } else {
                            add_sep = after_pathsep(
                                (*search_ctx).ffsc_fix_path.data,
                                (*search_ctx)
                                    .ffsc_fix_path
                                    .data
                                    .offset((*search_ctx).ffsc_fix_path.size as isize),
                            ) == 0;
                            (*ff_expand_buffer.ptr()).size = (*ff_expand_buffer.ptr())
                                .size
                                .wrapping_add(vim_snprintf(
                                    (*ff_expand_buffer.ptr())
                                        .data
                                        .offset((*ff_expand_buffer.ptr()).size as isize),
                                    (MAXPATHL as size_t)
                                        .wrapping_sub((*ff_expand_buffer.ptr()).size),
                                    b"%.*s%s\0".as_ptr() as *const ::core::ffi::c_char,
                                    len_1,
                                    (*search_ctx).ffsc_fix_path.data,
                                    if add_sep as ::core::ffi::c_int != 0 {
                                        PATHSEPSTR.as_ptr()
                                    } else {
                                        b"\0".as_ptr() as *const ::core::ffi::c_char
                                    },
                                ) as size_t);
                            '_c2rust_label_3: {
                                if (*ff_expand_buffer.ptr()).size < 4096 as size_t {
                                } else {
                                    __assert_fail(
                                    b"ff_expand_buffer.size < MAXPATHL\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/file_search.rs\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    501 as ::core::ffi::c_uint,
                                    b"void *vim_findfile_init(char *, char *, size_t, char *, int, int, int, void *, int, char *)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                                }
                            };
                        }
                    }
                    if !(*search_ctx).ffsc_wc_path.data.is_null() {
                        let mut tempsize: size_t = (*search_ctx)
                            .ffsc_fix_path
                            .size
                            .wrapping_sub(len_1 as size_t)
                            .wrapping_add((*search_ctx).ffsc_wc_path.size)
                            .wrapping_add(1 as size_t);
                        let mut temp: *mut ::core::ffi::c_char =
                            xmalloc(tempsize) as *mut ::core::ffi::c_char;
                        (*search_ctx).ffsc_wc_path.size = vim_snprintf(
                            temp,
                            tempsize,
                            b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                            (*search_ctx).ffsc_fix_path.data.offset(len_1 as isize),
                            (*search_ctx).ffsc_wc_path.data,
                        ) as size_t;
                        '_c2rust_label_4: {
                            if (*search_ctx).ffsc_wc_path.size < tempsize {
                            } else {
                                __assert_fail(
                                b"search_ctx->ffsc_wc_path.size < tempsize\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/file_search.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                513 as ::core::ffi::c_uint,
                                b"void *vim_findfile_init(char *, char *, size_t, char *, int, int, int, void *, int, char *)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                            }
                        };
                        xfree((*search_ctx).ffsc_wc_path.data as *mut ::core::ffi::c_void);
                        (*search_ctx).ffsc_wc_path.data = temp;
                    }
                }
                xfree(buf as *mut ::core::ffi::c_void);
                sptr = ff_create_stack_element(
                    (*ff_expand_buffer.ptr()).data,
                    (*ff_expand_buffer.ptr()).size,
                    (*search_ctx).ffsc_wc_path.data,
                    (*search_ctx).ffsc_wc_path.size,
                    level,
                    0 as ::core::ffi::c_int,
                );
                ff_push(search_ctx, sptr);
                (*search_ctx).ffsc_file_to_search = cbuf_to_string(filename, filenamelen);
                return search_ctx as *mut ::core::ffi::c_void;
            }
        }
        vim_findfile_cleanup(search_ctx as *mut ::core::ffi::c_void);
        return NULL;
    }
}

pub unsafe extern "C" fn vim_findfile_stopdir(
    mut buf: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        while *buf as ::core::ffi::c_int != NUL
            && *buf as ::core::ffi::c_int != ';' as ::core::ffi::c_int
            && (*buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '\\' as ::core::ffi::c_int
                || *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != ';' as ::core::ffi::c_int)
        {
            buf = buf.offset(1);
        }
        let mut dst: *mut ::core::ffi::c_char = buf;
        's_91: {
            '_is_semicolon: {
                if *buf as ::core::ffi::c_int != ';' as ::core::ffi::c_int {
                    if *buf as ::core::ffi::c_int != NUL {
                        's_61: loop {
                            let c2rust_fresh8 = dst;
                            dst = dst.offset(1);
                            *c2rust_fresh8 = ';' as ::core::ffi::c_char;
                            buf = buf.offset(2 as ::core::ffi::c_int as isize);
                            loop {
                                if !(*buf as ::core::ffi::c_int != NUL
                                    && *buf as ::core::ffi::c_int != ';' as ::core::ffi::c_int)
                                {
                                    break 's_61;
                                }
                                if *buf.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == '\\' as ::core::ffi::c_int
                                    && *buf.offset(1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == ';' as ::core::ffi::c_int
                                {
                                    break;
                                }
                                let c2rust_fresh9 = buf;
                                buf = buf.offset(1);
                                let c2rust_fresh10 = dst;
                                dst = dst.offset(1);
                                *c2rust_fresh10 = *c2rust_fresh9;
                            }
                        }
                        '_c2rust_label: {
                            if dst < buf {
                            } else {
                                __assert_fail(
                                    b"dst < buf\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"src/nvim/file_search.rs\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    561 as ::core::ffi::c_uint,
                                    b"char *vim_findfile_stopdir(char *)\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        *dst = NUL as ::core::ffi::c_char;
                        if *buf as ::core::ffi::c_int == ';' as ::core::ffi::c_int {
                            break '_is_semicolon;
                        }
                    }
                    buf = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    break 's_91;
                }
            }
            *buf = NUL as ::core::ffi::c_char;
            buf = buf.offset(1);
        }
        return buf;
    }
}

pub unsafe extern "C" fn vim_findfile_cleanup(mut ctx: *mut ::core::ffi::c_void) {
    unsafe {
        if ctx.is_null() {
            return;
        }
        vim_findfile_free_visited(ctx);
        ff_clear(ctx as *mut ff_search_ctx_T);
        xfree(ctx);
    }
}
