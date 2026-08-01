//! Looking a name up along `'path'` or `'cdpath'`.
//!
//! [`find_file_in_path_option`] is the loop over the option's entries: it
//! expands environment variables in the name, decides whether the name is
//! absolute enough to skip the option entirely, and otherwise drives
//! [`vim_findfile`](super::vim_findfile) once per entry, remembering where
//! it got to so that a repeat call answers the next match.
//! `'suffixesadd'` is tried at every candidate.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn find_file_in_path(
    mut ptr: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut options: ::core::ffi::c_int,
    mut first: ::core::ffi::c_int,
    mut rel_fname: *mut ::core::ffi::c_char,
    mut file_to_find: *mut *mut ::core::ffi::c_char,
    mut search_ctx: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        return find_file_in_path_option(
            ptr,
            len,
            options,
            first,
            if *(*curbuf.get()).b_p_path as ::core::ffi::c_int == NUL {
                p_path.get()
            } else {
                (*curbuf.get()).b_p_path
            },
            FINDFILE_BOTH as ::core::ffi::c_int,
            rel_fname,
            (*curbuf.get()).b_p_sua,
            file_to_find,
            search_ctx,
        );
    }
}

pub unsafe extern "C" fn find_directory_in_path(
    mut ptr: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut options: ::core::ffi::c_int,
    mut rel_fname: *mut ::core::ffi::c_char,
    mut file_to_find: *mut *mut ::core::ffi::c_char,
    mut search_ctx: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        return find_file_in_path_option(
            ptr,
            len,
            options,
            true_0,
            p_cdpath.get(),
            FINDFILE_DIR as ::core::ffi::c_int,
            rel_fname,
            b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            file_to_find,
            search_ctx,
        );
    }
}

pub unsafe extern "C" fn find_file_in_path_option(
    mut ptr: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut options: ::core::ffi::c_int,
    mut first: ::core::ffi::c_int,
    mut path_option: *mut ::core::ffi::c_char,
    mut find_what: ::core::ffi::c_int,
    mut rel_fname: *mut ::core::ffi::c_char,
    mut suffixes: *mut ::core::ffi::c_char,
    mut file_to_find: *mut *mut ::core::ffi::c_char,
    mut search_ctx_arg: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut search_ctx: *mut *mut ff_search_ctx_T = search_ctx_arg as *mut *mut ff_search_ctx_T;
        static dir: GlobalCell<*mut ::core::ffi::c_char> =
            GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
        static did_findfile_init: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        let mut file_name: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        static file_to_findlen: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
        if !rel_fname.is_null() && path_with_url(rel_fname) != 0 {
            rel_fname = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if first == true_0 {
            if len == 0 as size_t {
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            let mut save_char: ::core::ffi::c_char = *ptr.offset(len as isize);
            *ptr.offset(len as isize) = NUL as ::core::ffi::c_char;
            file_to_findlen.set(expand_env_esc(
                ptr,
                NameBuff.ptr() as *mut ::core::ffi::c_char,
                MAXPATHL as ::core::ffi::c_int,
                false_0 != 0,
                true_0 != 0,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ));
            *ptr.offset(len as isize) = save_char;
            xfree(*file_to_find as *mut ::core::ffi::c_void);
            *file_to_find = xmemdupz(
                NameBuff.ptr() as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                file_to_findlen.get(),
            ) as *mut ::core::ffi::c_char;
            if options & FNAME_UNESC as ::core::ffi::c_int != 0 {
                ptr = *file_to_find;
                while *ptr as ::core::ffi::c_int != NUL {
                    if *ptr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int
                        && *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == ' ' as ::core::ffi::c_int
                    {
                        memmove(
                            ptr as *mut ::core::ffi::c_void,
                            ptr.offset(1 as ::core::ffi::c_int as isize)
                                as *const ::core::ffi::c_void,
                            ((*file_to_find)
                                .offset(file_to_findlen.get() as isize)
                                .offset_from(ptr.offset(1 as ::core::ffi::c_int as isize))
                                as size_t)
                                .wrapping_add(1 as size_t),
                        );
                        file_to_findlen.set((*file_to_findlen.ptr()).wrapping_sub(1));
                    }
                    ptr = ptr.offset(1);
                }
            }
        }
        let mut rel_to_curdir: bool = *(*file_to_find).offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
            && (*(*file_to_find).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == NUL
                || vim_ispathsep(
                    *(*file_to_find).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                ) as ::core::ffi::c_int
                    != 0
                || *(*file_to_find).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '.' as ::core::ffi::c_int
                    && (*(*file_to_find).offset(2 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        == NUL
                        || vim_ispathsep(*(*file_to_find).offset(2 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                            != 0));
        '_theend: {
            's_300: {
                if vim_isAbsName(*file_to_find) as ::core::ffi::c_int != 0
                    || rel_to_curdir as ::core::ffi::c_int != 0
                {
                    if first == true_0 {
                        if path_with_url(*file_to_find) != 0 {
                            file_name = xmemdupz(
                                *file_to_find as *const ::core::ffi::c_void,
                                file_to_findlen.get(),
                            ) as *mut ::core::ffi::c_char;
                            break '_theend;
                        } else {
                            let mut rel_fnamelen: size_t = if !rel_fname.is_null() {
                                strlen(rel_fname)
                            } else {
                                0 as size_t
                            };
                            let mut run: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                            loop {
                                if run > 2 as ::core::ffi::c_int {
                                    break 's_300;
                                }
                                let mut l: size_t = file_to_findlen.get();
                                if run == 1 as ::core::ffi::c_int
                                    && rel_to_curdir as ::core::ffi::c_int != 0
                                    && options & FNAME_REL as ::core::ffi::c_int != 0
                                    && !rel_fname.is_null()
                                    && rel_fnamelen.wrapping_add(l) < MAXPATHL as size_t
                                {
                                    l = vim_snprintf(
                                        NameBuff.ptr() as *mut ::core::ffi::c_char,
                                        MAXPATHL as size_t,
                                        b"%.*s%s\0".as_ptr() as *const ::core::ffi::c_char,
                                        path_tail(rel_fname).offset_from(rel_fname)
                                            as ::core::ffi::c_int,
                                        rel_fname,
                                        *file_to_find,
                                    ) as size_t;
                                    '_c2rust_label: {
                                        if l < 4096 as size_t {
                                        } else {
                                            __assert_fail(
                                            b"l < MAXPATHL\0".as_ptr() as *const ::core::ffi::c_char,
                                            b"src/nvim/file_search.rs\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                            1499 as ::core::ffi::c_uint,
                                            b"char *find_file_in_path_option(char *, size_t, int, int, char *, int, char *, char *, char **, char **)\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        );
                                        }
                                    };
                                } else {
                                    strcpy(
                                        NameBuff.ptr() as *mut ::core::ffi::c_char,
                                        *file_to_find,
                                    );
                                    run = 2 as ::core::ffi::c_int;
                                }
                                let mut NameBufflen: size_t = l;
                                let mut suffix: *mut ::core::ffi::c_char = suffixes;
                                loop {
                                    if os_path_exists(NameBuff.ptr() as *mut ::core::ffi::c_char)
                                        as ::core::ffi::c_int
                                        != 0
                                        && (find_what == FINDFILE_BOTH as ::core::ffi::c_int
                                            || (find_what == FINDFILE_DIR as ::core::ffi::c_int)
                                                as ::core::ffi::c_int
                                                == os_isdir(
                                                    NameBuff.ptr() as *mut ::core::ffi::c_char
                                                )
                                                    as ::core::ffi::c_int)
                                    {
                                        file_name = xmemdupz(
                                            NameBuff.ptr() as *mut ::core::ffi::c_char
                                                as *const ::core::ffi::c_void,
                                            NameBufflen,
                                        )
                                            as *mut ::core::ffi::c_char;
                                        break '_theend;
                                    } else {
                                        if *suffix as ::core::ffi::c_int == NUL {
                                            break;
                                        }
                                        '_c2rust_label_0: {
                                            if 4096 as size_t >= l {
                                            } else {
                                                __assert_fail(
                                                b"MAXPATHL >= l\0".as_ptr() as *const ::core::ffi::c_char,
                                                b"src/nvim/file_search.rs\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                1518 as ::core::ffi::c_uint,
                                                b"char *find_file_in_path_option(char *, size_t, int, int, char *, int, char *, char *, char **, char **)\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            );
                                            }
                                        };
                                        NameBufflen = l.wrapping_add(copy_option_part(
                                            &raw mut suffix,
                                            (NameBuff.ptr() as *mut ::core::ffi::c_char)
                                                .offset(l as isize),
                                            (MAXPATHL as size_t).wrapping_sub(l),
                                            b",\0".as_ptr() as *const ::core::ffi::c_char
                                                as *mut ::core::ffi::c_char,
                                        ));
                                    }
                                }
                                run += 1;
                            }
                        }
                    }
                } else {
                    if first == true_0 {
                        vim_findfile_free_visited(*search_ctx as *mut ::core::ffi::c_void);
                        dir.set(path_option);
                        did_findfile_init.set(false_0 != 0);
                    }
                    loop {
                        if did_findfile_init.get() {
                            file_name = vim_findfile(*search_ctx as *mut ::core::ffi::c_void);
                            if !file_name.is_null() {
                                break;
                            }
                            did_findfile_init.set(false_0 != 0);
                        } else {
                            let mut r_ptr: *mut ::core::ffi::c_char =
                                ::core::ptr::null_mut::<::core::ffi::c_char>();
                            if (*dir.ptr()).is_null() || *dir.get() as ::core::ffi::c_int == NUL {
                                vim_findfile_cleanup(*search_ctx as *mut ::core::ffi::c_void);
                                *search_ctx = ::core::ptr::null_mut::<ff_search_ctx_T>();
                                break;
                            } else {
                                let mut buf: *mut ::core::ffi::c_char =
                                    xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
                                *buf.offset(0 as ::core::ffi::c_int as isize) =
                                    NUL as ::core::ffi::c_char;
                                copy_option_part(
                                    dir.ptr(),
                                    buf,
                                    MAXPATHL as size_t,
                                    b" ,\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
                                );
                                r_ptr = vim_findfile_stopdir(buf);
                                *search_ctx = vim_findfile_init(
                                    buf,
                                    *file_to_find,
                                    file_to_findlen.get(),
                                    r_ptr,
                                    100 as ::core::ffi::c_int,
                                    false_0,
                                    find_what,
                                    *search_ctx as *mut ::core::ffi::c_void,
                                    false_0,
                                    rel_fname,
                                )
                                    as *mut ff_search_ctx_T;
                                if !(*search_ctx).is_null() {
                                    did_findfile_init.set(true_0 != 0);
                                }
                                xfree(buf as *mut ::core::ffi::c_void);
                            }
                        }
                    }
                }
            }
            if file_name.is_null() && options & FNAME_MESS as ::core::ffi::c_int != 0 {
                if first == true_0 {
                    if find_what == FINDFILE_DIR as ::core::ffi::c_int {
                        semsg(
                            gettext(
                                &raw const e_cant_find_directory_str_in_cdpath
                                    as *const ::core::ffi::c_char,
                            ),
                            *file_to_find,
                        );
                    } else {
                        semsg(
                            gettext(
                                &raw const e_cant_find_file_str_in_path
                                    as *const ::core::ffi::c_char,
                            ),
                            *file_to_find,
                        );
                    }
                } else if find_what == FINDFILE_DIR as ::core::ffi::c_int {
                    semsg(
                        gettext(
                            &raw const e_no_more_directory_str_found_in_cdpath
                                as *const ::core::ffi::c_char,
                        ),
                        *file_to_find,
                    );
                } else {
                    semsg(
                        gettext(
                            &raw const e_no_more_file_str_found_in_path
                                as *const ::core::ffi::c_char,
                        ),
                        *file_to_find,
                    );
                }
            }
        }
        return file_name;
    }
}
