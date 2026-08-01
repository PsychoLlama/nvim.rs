//! Which files hold the tags.
//!
//! [`get_tagfname`] walks the `'tags'` option one name at a time, expanding
//! wildcards, following `./` relative to the current file and, in a help
//! buffer, visiting every `doc/tags` in `'runtimepath'` first.
//! [`expand_tag_fname`] is the other half of `'tagrelative'`: it turns a
//! file name a tags file mentions into one the editor can open.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn found_tagfile_cb(
    mut num_fnames: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut all: bool,
    mut _cookie: *mut ::core::ffi::c_void,
) -> bool {
    unsafe {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < num_fnames {
            let tag_fname: *mut ::core::ffi::c_char = xstrdup(*fnames.offset(i as isize));
            simplify_filename(tag_fname);
            ga_grow(tag_fnames.ptr(), 1 as ::core::ffi::c_int);
            *((*tag_fnames.ptr()).ga_data as *mut *mut ::core::ffi::c_char)
                .offset((*tag_fnames.ptr()).ga_len as isize) = tag_fname;
            (*tag_fnames.ptr()).ga_len += 1;
            if !all {
                break;
            }
            i += 1;
        }
        return num_fnames > 0 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn get_tagfname(
    mut tnp: *mut tagname_T,
    mut first: ::core::ffi::c_int,
    mut buf: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if first != 0 {
            memset(
                tnp as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<tagname_T>(),
            );
        }
        if (*curbuf.get()).b_help {
            if first != 0 {
                ga_clear_strings(tag_fnames.ptr());
                ga_init(
                    tag_fnames.ptr(),
                    ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
                    10 as ::core::ffi::c_int,
                );
                do_in_runtimepath(
                    b"doc/tags doc/tags-??\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    DIP_ALL as ::core::ffi::c_int,
                    Some(
                        found_tagfile_cb
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
            if (*tnp).tn_hf_idx >= (*tag_fnames.ptr()).ga_len {
                if (*tnp).tn_hf_idx > (*tag_fnames.ptr()).ga_len
                    || *p_hf.get() as ::core::ffi::c_int == NUL
                {
                    return FAIL;
                }
                (*tnp).tn_hf_idx += 1;
                xstrlcpy(
                    buf,
                    p_hf.get(),
                    (MAXPATHL as size_t).wrapping_sub(
                        ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                            .wrapping_sub(1 as size_t),
                    ),
                );
                strcpy(
                    path_tail(buf),
                    b"tags\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
                simplify_filename(buf);
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < (*tag_fnames.ptr()).ga_len {
                    if strcmp(
                        buf,
                        *((*tag_fnames.ptr()).ga_data as *mut *mut ::core::ffi::c_char)
                            .offset(i as isize),
                    ) == 0 as ::core::ffi::c_int
                    {
                        return FAIL;
                    }
                    i += 1;
                }
            } else {
                let c2rust_fresh5 = (*tnp).tn_hf_idx;
                (*tnp).tn_hf_idx = (*tnp).tn_hf_idx + 1;
                xstrlcpy(
                    buf,
                    *((*tag_fnames.ptr()).ga_data as *mut *mut ::core::ffi::c_char)
                        .offset(c2rust_fresh5 as isize),
                    MAXPATHL as size_t,
                );
            }
            return OK;
        }
        if first != 0 {
            (*tnp).tn_tags = xstrdup(if *(*curbuf.get()).b_p_tags as ::core::ffi::c_int != NUL {
                (*curbuf.get()).b_p_tags
            } else {
                p_tags.get()
            });
            (*tnp).tn_np = (*tnp).tn_tags;
        }
        loop {
            if (*tnp).tn_did_filefind_init != 0 {
                fname = vim_findfile((*tnp).tn_search_ctx);
                if !fname.is_null() {
                    break;
                }
                (*tnp).tn_did_filefind_init = false_0;
            } else {
                let mut filename: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                if *(*tnp).tn_np as ::core::ffi::c_int == NUL {
                    vim_findfile_cleanup((*tnp).tn_search_ctx);
                    (*tnp).tn_search_ctx = NULL_0;
                    return FAIL;
                }
                *buf.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
                copy_option_part(
                    &raw mut (*tnp).tn_np,
                    buf,
                    (MAXPATHL - 1 as ::core::ffi::c_int) as size_t,
                    b" ,\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
                let mut r_ptr: *mut ::core::ffi::c_char = vim_findfile_stopdir(buf);
                filename = path_tail(buf);
                if !r_ptr.is_null() {
                    memmove(
                        r_ptr.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                        r_ptr as *const ::core::ffi::c_void,
                        strlen(r_ptr).wrapping_add(1 as size_t),
                    );
                    r_ptr = r_ptr.offset(1);
                }
                memmove(
                    filename.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                    filename as *const ::core::ffi::c_void,
                    strlen(filename).wrapping_add(1 as size_t),
                );
                let c2rust_fresh6 = filename;
                filename = filename.offset(1);
                *c2rust_fresh6 = NUL as ::core::ffi::c_char;
                (*tnp).tn_search_ctx = vim_findfile_init(
                    buf,
                    filename,
                    strlen(filename),
                    r_ptr,
                    100 as ::core::ffi::c_int,
                    false,
                    FINDFILE_FILE as ::core::ffi::c_int,
                    (*tnp).tn_search_ctx,
                    true,
                    (*curbuf.get()).b_ffname,
                );
                if !(*tnp).tn_search_ctx.is_null() {
                    (*tnp).tn_did_filefind_init = true_0;
                }
            }
        }
        strcpy(buf, fname);
        xfree(fname as *mut ::core::ffi::c_void);
        return OK;
    }
}

pub unsafe extern "C" fn tagname_free(mut tnp: *mut tagname_T) {
    unsafe {
        xfree((*tnp).tn_tags as *mut ::core::ffi::c_void);
        vim_findfile_cleanup((*tnp).tn_search_ctx);
        (*tnp).tn_search_ctx = NULL_0;
        ga_clear_strings(tag_fnames.ptr());
    }
}

pub(crate) unsafe extern "C" fn expand_tag_fname(
    mut fname: *mut ::core::ffi::c_char,
    tag_fname: *mut ::core::ffi::c_char,
    expand: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut expanded_fname: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut xpc: expand_T = expand_T {
            xp_pattern: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            xp_context: 0,
            xp_pattern_len: 0,
            xp_prefix: XP_PREFIX_NONE,
            xp_arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            xp_luaref: 0,
            xp_script_ctx: sctx_T {
                sc_sid: 0,
                sc_seq: 0,
                sc_lnum: 0,
                sc_chan: 0,
            },
            xp_backslash: 0,
            xp_shell: false,
            xp_numfiles: 0,
            xp_col: 0,
            xp_selected: 0,
            xp_orig: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            xp_files: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            xp_line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            xp_buf: [0; 256],
            xp_search_dir: kDirectionNotSet,
            xp_pre_incsearch_pos: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
        };
        if expand as ::core::ffi::c_int != 0
            && path_has_wildcard(fname) as ::core::ffi::c_int != 0
            && vim_strchr(fname, '`' as ::core::ffi::c_int).is_null()
        {
            ExpandInit(&raw mut xpc);
            xpc.xp_context = EXPAND_FILES as ::core::ffi::c_int;
            expanded_fname = ExpandOne(
                &raw mut xpc,
                fname,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                WILD_LIST_NOTFOUND as ::core::ffi::c_int | WILD_SILENT as ::core::ffi::c_int,
                WILD_EXPAND_FREE as ::core::ffi::c_int,
            );
            if !expanded_fname.is_null() {
                fname = expanded_fname;
            }
        }
        let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if (p_tr.get() != 0 || (*curbuf.get()).b_help as ::core::ffi::c_int != 0)
            && !vim_isAbsName(fname)
            && {
                p = path_tail(tag_fname);
                p != tag_fname
            }
        {
            retval = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
            strcpy(retval, tag_fname);
            xstrlcpy(
                retval.offset(p.offset_from(tag_fname) as isize),
                fname,
                (MAXPATHL as isize - p.offset_from(tag_fname)) as size_t,
            );
            simplify_filename(retval);
        } else {
            retval = xstrdup(fname);
        }
        xfree(expanded_fname as *mut ::core::ffi::c_void);
        return retval;
    }
}
