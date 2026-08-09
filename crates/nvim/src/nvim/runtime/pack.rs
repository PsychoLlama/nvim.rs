//! Packages: `:packadd`, `:packloadall`, and the `pack/*/start` trees loaded at
//! startup.
//!
//! A package is a directory under 'packpath' that gets *added to
//! 'runtimepath'* and then sourced.  `add_pack_dir_to_rtp` is the hard half --
//! it has to insert the package at the right point in the option string,
//! after the last non-`after` entry that is a prefix of it, and insert its own
//! `after/` directory symmetrically at the other end, so that a package's
//! `after/` still runs after everything it should.  `load_pack_plugin` sources
//! what the package contains, and the `add_*_pack_plugins` family walks the
//! `start` (loaded at startup) and `opt` (loaded on demand) trees.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn add_pack_dir_to_rtp(
    mut fname: *mut ::core::ffi::c_char,
    mut is_pack: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut afterlen: size_t = 0;
        let mut oldlen: size_t = 0;
        let mut addlen: size_t = 0;
        let mut new_rtp_capacity: size_t = 0;
        let mut new_rtp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut keep: size_t = 0;
        let mut first_pos: size_t = 0;
        let mut new_rtp_len: size_t = 0;
        let mut after_pos: size_t = 0;
        let mut was_valid: bool = false;
        let mut afterdir: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut retval: ::core::ffi::c_int = FAIL;
        let mut p1: *mut ::core::ffi::c_char = get_past_head(fname);
        let mut p2: *mut ::core::ffi::c_char = p1;
        let mut p3: *mut ::core::ffi::c_char = p1;
        let mut p4: *mut ::core::ffi::c_char = p1;
        let mut p: *mut ::core::ffi::c_char = p1;
        while *p != 0 {
            if vim_ispathsep_nocolon(*p as ::core::ffi::c_int) {
                p4 = p3;
                p3 = p2;
                p2 = p1;
                p1 = p;
            }
            p = p.offset(utfc_ptr2len(p) as isize);
        }
        p4 = p4.offset(1);
        let mut c: ::core::ffi::c_char = *p4;
        *p4 = NUL as ::core::ffi::c_char;
        let ffname: *mut ::core::ffi::c_char = fix_fname(fname);
        *p4 = c;
        if ffname.is_null() {
            return FAIL;
        }
        let mut fname_len: size_t = strlen(ffname);
        let mut buf: [::core::ffi::c_char; 4096] = [0; 4096];
        let mut insp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut after_insp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut entry: *const ::core::ffi::c_char = p_rtp.get();
        '_theend: {
            while *entry as ::core::ffi::c_int != NUL {
                let mut cur_entry: *const ::core::ffi::c_char = entry;
                copy_option_part(
                    &raw mut entry as *mut *mut ::core::ffi::c_char,
                    &raw mut buf as *mut ::core::ffi::c_char,
                    MAXPATHL as size_t,
                    c",".as_ptr() as *mut ::core::ffi::c_char,
                );
                let mut p_0: *mut ::core::ffi::c_char =
                    strstr(&raw mut buf as *mut ::core::ffi::c_char, c"after".as_ptr());
                let mut is_after: bool = !p_0.is_null()
                    && p_0 > &raw mut buf as *mut ::core::ffi::c_char
                    && vim_ispathsep(
                        *p_0.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0
                    && (vim_ispathsep(
                        *p_0.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0
                        || *p_0.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == NUL
                        || *p_0.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == ',' as ::core::ffi::c_int);
                if is_after {
                    if insp.is_null() {
                        insp = cur_entry;
                    }
                    after_insp = cur_entry;
                    break;
                } else {
                    if !insp.is_null() {
                        continue;
                    }
                    add_pathsep(&raw mut buf as *mut ::core::ffi::c_char);
                    let rtp_ffname: *mut ::core::ffi::c_char =
                        fix_fname(&raw mut buf as *mut ::core::ffi::c_char);
                    if rtp_ffname.is_null() {
                        break '_theend;
                    }
                    if path_fnamencmp(rtp_ffname, ffname, fname_len) == 0 as ::core::ffi::c_int {
                        insp = entry;
                    }
                    xfree(rtp_ffname as *mut ::core::ffi::c_void);
                }
            }
            if insp.is_null() {
                insp = (*p_rtp.ptr()).add(strlen(p_rtp.get()));
            }
            afterdir = concat_fnames(fname, c"after".as_ptr(), true_0 != 0);
            afterlen = 0 as size_t;
            if if is_pack as ::core::ffi::c_int != 0 {
                pack_has_entries(afterdir) as ::core::ffi::c_int
            } else {
                os_isdir(afterdir) as ::core::ffi::c_int
            } != 0
            {
                afterlen = strlen(afterdir).wrapping_add(1 as size_t);
            }
            oldlen = strlen(p_rtp.get());
            addlen = strlen(fname).wrapping_add(1 as size_t);
            new_rtp_capacity = oldlen
                .wrapping_add(addlen)
                .wrapping_add(afterlen)
                .wrapping_add(1 as size_t);
            new_rtp = try_malloc(new_rtp_capacity) as *mut ::core::ffi::c_char;
            if !new_rtp.is_null() {
                keep = insp.offset_from(p_rtp.get()) as size_t;
                first_pos = keep;
                memmove(
                    new_rtp as *mut ::core::ffi::c_void,
                    p_rtp.get() as *const ::core::ffi::c_void,
                    keep,
                );
                new_rtp_len = keep;
                if *insp as ::core::ffi::c_int == NUL {
                    let c2rust_fresh15 = new_rtp_len;
                    new_rtp_len = new_rtp_len.wrapping_add(1);
                    *new_rtp.add(c2rust_fresh15) = ',' as ::core::ffi::c_char;
                    first_pos = first_pos.wrapping_add(1);
                }
                memmove(
                    new_rtp.add(new_rtp_len) as *mut ::core::ffi::c_void,
                    fname as *const ::core::ffi::c_void,
                    addlen.wrapping_sub(1 as size_t),
                );
                new_rtp_len = new_rtp_len.wrapping_add(addlen.wrapping_sub(1 as size_t));
                if *insp as ::core::ffi::c_int != NUL {
                    let c2rust_fresh16 = new_rtp_len;
                    new_rtp_len = new_rtp_len.wrapping_add(1);
                    *new_rtp.add(c2rust_fresh16) = ',' as ::core::ffi::c_char;
                }
                after_pos = 0 as size_t;
                if afterlen > 0 as size_t && !after_insp.is_null() {
                    let mut keep_after: size_t = after_insp.offset_from(p_rtp.get()) as size_t;
                    memmove(
                        new_rtp.add(new_rtp_len) as *mut ::core::ffi::c_void,
                        (*p_rtp.ptr()).add(keep) as *const ::core::ffi::c_void,
                        keep_after.wrapping_sub(keep),
                    );
                    new_rtp_len = new_rtp_len.wrapping_add(keep_after.wrapping_sub(keep));
                    memmove(
                        new_rtp.add(new_rtp_len) as *mut ::core::ffi::c_void,
                        afterdir as *const ::core::ffi::c_void,
                        afterlen.wrapping_sub(1 as size_t),
                    );
                    new_rtp_len = new_rtp_len.wrapping_add(afterlen.wrapping_sub(1 as size_t));
                    let c2rust_fresh17 = new_rtp_len;
                    new_rtp_len = new_rtp_len.wrapping_add(1);
                    *new_rtp.add(c2rust_fresh17) = ',' as ::core::ffi::c_char;
                    keep = keep_after;
                    after_pos = keep_after;
                }
                if *(*p_rtp.ptr()).add(keep) as ::core::ffi::c_int != NUL {
                    memmove(
                        new_rtp.add(new_rtp_len) as *mut ::core::ffi::c_void,
                        (*p_rtp.ptr()).add(keep) as *const ::core::ffi::c_void,
                        oldlen.wrapping_sub(keep).wrapping_add(1 as size_t),
                    );
                } else {
                    *new_rtp.add(new_rtp_len) = NUL as ::core::ffi::c_char;
                }
                if afterlen > 0 as size_t && after_insp.is_null() {
                    after_pos = xstrlcat(new_rtp, c",".as_ptr(), new_rtp_capacity);
                    xstrlcat(new_rtp, afterdir, new_rtp_capacity);
                }
                was_valid = runtime_search_path_valid.get();
                set_option_value_give_err(
                    kOptRuntimepath,
                    OptVal {
                        type_0: kOptValTypeString,
                        data: OptValData {
                            string: cstr_as_string(new_rtp),
                        },
                    },
                    0 as ::core::ffi::c_int,
                );
                debug_assert!(
                    !runtime_search_path_valid.get(),
                    "!runtime_search_path_valid"
                );
                if was_valid as ::core::ffi::c_int != 0
                    && !is_pack
                    && (*runtime_search_path_ref.ptr()).is_null()
                {
                    runtime_search_path_valid.set(true_0 != 0);
                    runtime_search_path_valid_thread.set(false_0 != 0);
                    if (*runtime_search_path.ptr()).size == (*runtime_search_path.ptr()).capacity {
                        (*runtime_search_path.ptr()).capacity =
                            if (*runtime_search_path.ptr()).capacity != 0 {
                                (*runtime_search_path.ptr()).capacity << 1 as ::core::ffi::c_int
                            } else {
                                8 as size_t
                            };
                        (*runtime_search_path.ptr()).items = xrealloc(
                            (*runtime_search_path.ptr()).items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<SearchPathItem>()
                                .wrapping_mul((*runtime_search_path.ptr()).capacity),
                        )
                            as *mut SearchPathItem;
                    } else {
                    };
                    (*runtime_search_path.ptr()).size =
                        (*runtime_search_path.ptr()).size.wrapping_add(1);
                    let mut i: ssize_t =
                        (*runtime_search_path.ptr()).size as ssize_t - 1 as ssize_t;
                    if afterlen > 0 as size_t {
                        if (*runtime_search_path.ptr()).size
                            == (*runtime_search_path.ptr()).capacity
                        {
                            (*runtime_search_path.ptr()).capacity =
                                if (*runtime_search_path.ptr()).capacity != 0 {
                                    (*runtime_search_path.ptr()).capacity << 1 as ::core::ffi::c_int
                                } else {
                                    8 as size_t
                                };
                            (*runtime_search_path.ptr()).items = xrealloc(
                                (*runtime_search_path.ptr()).items as *mut ::core::ffi::c_void,
                                ::core::mem::size_of::<SearchPathItem>()
                                    .wrapping_mul((*runtime_search_path.ptr()).capacity),
                            )
                                as *mut SearchPathItem;
                        } else {
                        };
                        (*runtime_search_path.ptr()).size =
                            (*runtime_search_path.ptr()).size.wrapping_add(1);
                        i += 1 as ssize_t;
                        while i >= 1 as ssize_t {
                            if i > 1 as ssize_t
                                && (*(*runtime_search_path.ptr())
                                    .items
                                    .offset((i - 2 as ssize_t) as isize))
                                .pos_in_rtp
                                    >= after_pos
                            {
                                *(*runtime_search_path.ptr()).items.offset(i as isize) =
                                    *(*runtime_search_path.ptr())
                                        .items
                                        .offset((i - 2 as ssize_t) as isize);
                                (*(*runtime_search_path.ptr()).items.offset(i as isize))
                                    .pos_in_rtp =
                                    (*(*runtime_search_path.ptr()).items.offset(i as isize))
                                        .pos_in_rtp
                                        .wrapping_add(addlen.wrapping_add(afterlen));
                                i -= 1;
                            } else {
                                *(*runtime_search_path.ptr()).items.offset(i as isize) =
                                    SearchPathItem {
                                        path: xstrdup(afterdir),
                                        after: true_0 != 0,
                                        pack_inserted: true_0 != 0,
                                        has_lua: kNone,
                                        pos_in_rtp: after_pos.wrapping_add(addlen),
                                    };
                                i -= 1;
                                break;
                            }
                        }
                    }
                    while i >= 0 as ssize_t {
                        if i > 0 as ssize_t
                            && (*(*runtime_search_path.ptr())
                                .items
                                .offset((i - 1 as ssize_t) as isize))
                            .pos_in_rtp
                                >= first_pos
                        {
                            *(*runtime_search_path.ptr()).items.offset(i as isize) =
                                *(*runtime_search_path.ptr())
                                    .items
                                    .offset((i - 1 as ssize_t) as isize);
                            (*(*runtime_search_path.ptr()).items.offset(i as isize)).pos_in_rtp =
                                (*(*runtime_search_path.ptr()).items.offset(i as isize))
                                    .pos_in_rtp
                                    .wrapping_add(addlen);
                            i -= 1;
                        } else {
                            *(*runtime_search_path.ptr()).items.offset(i as isize) =
                                SearchPathItem {
                                    path: xstrdup(fname),
                                    after: false_0 != 0,
                                    pack_inserted: true_0 != 0,
                                    has_lua: kNone,
                                    pos_in_rtp: first_pos,
                                };
                            break;
                        }
                    }
                }
                xfree(new_rtp as *mut ::core::ffi::c_void);
                retval = OK;
            }
        }
        xfree(ffname as *mut ::core::ffi::c_void);
        xfree(afterdir as *mut ::core::ffi::c_void);
        return retval;
    }
}

unsafe extern "C" fn load_pack_plugin(
    mut opt: bool,
    mut fname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        static plugpat: GlobalCell<[::core::ffi::c_char; 15]> = GlobalCell::new(unsafe {
            ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"%s/plugin/**/*\0")
        });
        static ftpat: GlobalCell<[::core::ffi::c_char; 14]> = GlobalCell::new(unsafe {
            ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b"%s/ftdetect/*\0")
        });
        let ffname: *mut ::core::ffi::c_char = fix_fname(fname);
        let mut len: size_t =
            strlen(ffname).wrapping_add(::core::mem::size_of::<[::core::ffi::c_char; 15]>());
        let mut pat: *mut ::core::ffi::c_char = xmallocz(len) as *mut ::core::ffi::c_char;
        vim_snprintf(
            pat,
            len,
            (plugpat.ptr() as *const _) as *const ::core::ffi::c_char,
            ffname,
        );
        gen_expand_wildcards_and_cb(
            1 as ::core::ffi::c_int,
            &raw mut pat,
            EW_FILE,
            true_0 != 0,
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
        let mut cmd: *mut ::core::ffi::c_char = xstrdup(c"g:did_load_filetypes".as_ptr());
        if opt as ::core::ffi::c_int != 0 && eval_to_number(cmd, false_0 != 0) > 0 as varnumber_T {
            do_cmdline_cmd(c"augroup filetypedetect".as_ptr());
            vim_snprintf(
                pat,
                len,
                (ftpat.ptr() as *const _) as *const ::core::ffi::c_char,
                ffname,
            );
            gen_expand_wildcards_and_cb(
                1 as ::core::ffi::c_int,
                &raw mut pat,
                EW_FILE,
                true_0 != 0,
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
            do_cmdline_cmd(c"augroup END".as_ptr());
        }
        xfree(cmd as *mut ::core::ffi::c_void);
        xfree(pat as *mut ::core::ffi::c_void);
        xfree(ffname as *mut ::core::ffi::c_void);
        return OK;
    }
}

unsafe extern "C" fn add_pack_plugins(
    mut opt: bool,
    mut num_fnames: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut all: bool,
    mut cookie: *mut ::core::ffi::c_void,
) {
    unsafe {
        let mut did_one: bool = false_0 != 0;
        if cookie != APP_LOAD.ptr() as *mut ::core::ffi::c_void {
            let mut buf: [::core::ffi::c_char; 4096] = [0; 4096];
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < num_fnames {
                let mut found: bool = false_0 != 0;
                let mut p: *const ::core::ffi::c_char = p_rtp.get();
                while *p as ::core::ffi::c_int != NUL {
                    copy_option_part(
                        &raw mut p as *mut *mut ::core::ffi::c_char,
                        &raw mut buf as *mut ::core::ffi::c_char,
                        MAXPATHL as size_t,
                        c",".as_ptr() as *mut ::core::ffi::c_char,
                    );
                    if path_fnamecmp(
                        &raw mut buf as *mut ::core::ffi::c_char,
                        *fnames.offset(i as isize),
                    ) != 0 as ::core::ffi::c_int
                    {
                        continue;
                    }
                    found = true_0 != 0;
                    break;
                }
                if !found {
                    if add_pack_dir_to_rtp(*fnames.offset(i as isize), false_0 != 0) == FAIL {
                        return;
                    }
                }
                did_one = true_0 != 0;
                if !all {
                    break;
                }
                i += 1;
            }
        }
        if !all && did_one as ::core::ffi::c_int != 0 {
            return;
        }
        if cookie != APP_ADD_DIR.ptr() as *mut ::core::ffi::c_void {
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_0 < num_fnames {
                load_pack_plugin(opt, *fnames.offset(i_0 as isize));
                if !all {
                    break;
                }
                i_0 += 1;
            }
        }
    }
}

unsafe extern "C" fn add_start_pack_plugins(
    mut num_fnames: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut all: bool,
    mut cookie: *mut ::core::ffi::c_void,
) -> bool {
    unsafe {
        add_pack_plugins(false_0 != 0, num_fnames, fnames, all, cookie);
        return num_fnames > 0 as ::core::ffi::c_int;
    }
}

unsafe extern "C" fn add_opt_pack_plugins(
    mut num_fnames: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut all: bool,
    mut cookie: *mut ::core::ffi::c_void,
) -> bool {
    unsafe {
        add_pack_plugins(true_0 != 0, num_fnames, fnames, all, cookie);
        return num_fnames > 0 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn add_pack_start_dirs() {
    unsafe {
        do_in_path(
            p_pp.get(),
            c"".as_ptr(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            DIP_ALL as ::core::ffi::c_int + DIP_DIR as ::core::ffi::c_int,
            Some(
                add_pack_start_dir
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

unsafe extern "C" fn pack_has_entries(mut buf: *mut ::core::ffi::c_char) -> bool {
    unsafe {
        let mut num_files: ::core::ffi::c_int = 0;
        let mut files: *mut *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        let mut pat: [*mut ::core::ffi::c_char; 1] = [buf as *mut ::core::ffi::c_char];
        if gen_expand_wildcards(
            1 as ::core::ffi::c_int,
            &raw mut pat as *mut *mut ::core::ffi::c_char,
            &raw mut num_files,
            &raw mut files,
            EW_DIR,
        ) == OK
        {
            FreeWild(num_files, files);
        }
        return num_files > 0 as ::core::ffi::c_int;
    }
}

unsafe extern "C" fn add_pack_start_dir(
    mut num_fnames: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut all: bool,
    mut _cookie: *mut ::core::ffi::c_void,
) -> bool {
    unsafe {
        static buf: GlobalCell<[::core::ffi::c_char; 4096]> = GlobalCell::new([0; 4096]);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < num_fnames {
            let mut start_pat: [*mut ::core::ffi::c_char; 2] = [
                c"/start/*".as_ptr() as *mut ::core::ffi::c_char,
                c"/pack/*/start/*".as_ptr() as *mut ::core::ffi::c_char,
            ];
            let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while j < 2 as ::core::ffi::c_int {
                if strlen(*fnames.offset(i as isize))
                    .wrapping_add(strlen(start_pat[j as usize] as *const ::core::ffi::c_char))
                    .wrapping_add(1 as size_t)
                    <= MAXPATHL as size_t
                {
                    xstrlcpy(
                        buf.ptr() as *mut ::core::ffi::c_char,
                        *fnames.offset(i as isize),
                        MAXPATHL as size_t,
                    );
                    xstrlcat(
                        buf.ptr() as *mut ::core::ffi::c_char,
                        start_pat[j as usize] as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
                    );
                    if pack_has_entries(buf.ptr() as *mut ::core::ffi::c_char) {
                        add_pack_dir_to_rtp(buf.ptr() as *mut ::core::ffi::c_char, true_0 != 0);
                    }
                }
                j += 1;
            }
            if !all {
                break;
            }
            i += 1;
        }
        return num_fnames > 1 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn load_start_packages() {
    unsafe {
        did_source_packages.set(true_0 != 0);
        do_in_path(
            p_pp.get(),
            c"".as_ptr(),
            c"pack/*/start/*".as_ptr() as *mut ::core::ffi::c_char,
            DIP_ALL as ::core::ffi::c_int + DIP_DIR as ::core::ffi::c_int,
            Some(
                add_start_pack_plugins
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *mut ::core::ffi::c_char,
                        bool,
                        *mut ::core::ffi::c_void,
                    ) -> bool,
            ),
            APP_LOAD.ptr() as *mut ::core::ffi::c_void,
        );
        do_in_path(
            p_pp.get(),
            c"".as_ptr(),
            c"start/*".as_ptr() as *mut ::core::ffi::c_char,
            DIP_ALL as ::core::ffi::c_int + DIP_DIR as ::core::ffi::c_int,
            Some(
                add_start_pack_plugins
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *mut ::core::ffi::c_char,
                        bool,
                        *mut ::core::ffi::c_void,
                    ) -> bool,
            ),
            APP_LOAD.ptr() as *mut ::core::ffi::c_void,
        );
        update_runtime_search_path_thread(false_0 != 0);
    }
}

pub unsafe fn ex_packloadall(mut eap: *mut exarg_T) {
    unsafe {
        if !did_source_packages.get() || (*eap).forceit != 0 {
            add_pack_start_dirs();
            load_start_packages();
        }
    }
}

pub unsafe extern "C" fn load_plugins() {
    unsafe {
        if p_lpl.get() != 0 {
            let mut rtp_copy: *mut ::core::ffi::c_char = p_rtp.get();
            let plugin_pattern: *mut ::core::ffi::c_char =
                c"plugin/**/*".as_ptr() as *mut ::core::ffi::c_char;
            if !did_source_packages.get() {
                rtp_copy = xstrdup(p_rtp.get());
                add_pack_start_dirs();
            }
            source_in_path_vim_lua(
                rtp_copy,
                plugin_pattern,
                DIP_ALL as ::core::ffi::c_int | DIP_NOAFTER as ::core::ffi::c_int,
            );
            if !(*time_fd.ptr()).is_null() {
                time_msg(
                    c"loading rtp plugins".as_ptr(),
                    ::core::ptr::null::<proftime_T>(),
                );
            }
            if !did_source_packages.get() {
                xfree(rtp_copy as *mut ::core::ffi::c_void);
                load_start_packages();
            }
            if !(*time_fd.ptr()).is_null() {
                time_msg(
                    c"loading packages".as_ptr(),
                    ::core::ptr::null::<proftime_T>(),
                );
            }
            source_runtime_vim_lua(
                plugin_pattern,
                DIP_ALL as ::core::ffi::c_int | DIP_AFTER as ::core::ffi::c_int,
            );
            if !(*time_fd.ptr()).is_null() {
                time_msg(
                    c"loading after plugins".as_ptr(),
                    ::core::ptr::null::<proftime_T>(),
                );
            }
        }
    }
}

pub unsafe fn ex_packadd(mut eap: *mut exarg_T) {
    unsafe {
        static plugpat: GlobalCell<[::core::ffi::c_char; 13]> = GlobalCell::new(unsafe {
            ::core::mem::transmute::<[u8; 13], [::core::ffi::c_char; 13]>(*b"pack/*/%s/%s\0")
        });
        let mut res: ::core::ffi::c_int = OK;
        let len: size_t = ::core::mem::size_of::<[::core::ffi::c_char; 13]>()
            .wrapping_add(strlen((*eap).arg))
            .wrapping_add(5 as size_t);
        let mut pat: *mut ::core::ffi::c_char = xmallocz(len) as *mut ::core::ffi::c_char;
        let mut cookie: *mut ::core::ffi::c_void = (if (*eap).forceit != 0 {
            APP_ADD_DIR.ptr()
        } else {
            APP_BOTH.ptr()
        }) as *mut ::core::ffi::c_void;
        if !did_source_packages.get() {
            vim_snprintf(
                pat,
                len,
                (plugpat.ptr() as *const _) as *const ::core::ffi::c_char,
                c"start".as_ptr(),
                (*eap).arg,
            );
            res = do_in_path(
                p_pp.get(),
                c"".as_ptr(),
                pat,
                DIP_ALL as ::core::ffi::c_int + DIP_DIR as ::core::ffi::c_int,
                Some(
                    add_start_pack_plugins
                        as unsafe extern "C" fn(
                            ::core::ffi::c_int,
                            *mut *mut ::core::ffi::c_char,
                            bool,
                            *mut ::core::ffi::c_void,
                        ) -> bool,
                ),
                cookie,
            );
        }
        vim_snprintf(
            pat,
            len,
            (plugpat.ptr() as *const _) as *const ::core::ffi::c_char,
            c"opt".as_ptr(),
            (*eap).arg,
        );
        do_in_path(
            p_pp.get(),
            c"".as_ptr(),
            pat,
            DIP_ALL as ::core::ffi::c_int
                + DIP_DIR as ::core::ffi::c_int
                + (if res == FAIL {
                    DIP_ERR as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }),
            Some(
                add_opt_pack_plugins
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *mut ::core::ffi::c_char,
                        bool,
                        *mut ::core::ffi::c_void,
                    ) -> bool,
            ),
            cookie,
        );
        update_runtime_search_path_thread(false_0 != 0);
        xfree(pat as *mut ::core::ffi::c_void);
    }
}
