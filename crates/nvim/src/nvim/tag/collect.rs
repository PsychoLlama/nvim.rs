//! Recording a match.
//!
//! [`findtags_add_match`] files a parsed line under one of the
//! `MT_*` priorities — whether the match was exact, whether it came from
//! this file, whether it is static — and deduplicates it against the
//! matches already found.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn findtags_matchargs_init(
    mut margs: *mut findtags_match_args_T,
    mut flags: ::core::ffi::c_int,
) {
    unsafe {
        (*margs).matchoff = 0 as ::core::ffi::c_int;
        (*margs).match_re = false_0 != 0;
        (*margs).match_no_ic = false_0 != 0;
        (*margs).has_re = flags & TAG_REGEXP as ::core::ffi::c_int != 0;
        (*margs).sortic = false_0 != 0;
        (*margs).sort_error = false_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn findtags_string_convert(mut st: *mut findtags_state_T) {
    unsafe {
        let mut conv_line: *mut ::core::ffi::c_char = string_convert(
            &raw mut (*st).vimconv,
            (*st).lbuf,
            ::core::ptr::null_mut::<size_t>(),
        );
        if conv_line.is_null() {
            return;
        }
        let mut len: ::core::ffi::c_int =
            strlen(conv_line) as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
        if len > (*st).lbuf_size {
            xfree((*st).lbuf as *mut ::core::ffi::c_void);
            (*st).lbuf = conv_line;
            (*st).lbuf_size = len;
        } else {
            strcpy((*st).lbuf, conv_line);
            xfree(conv_line as *mut ::core::ffi::c_void);
        };
    }
}

pub(crate) unsafe extern "C" fn findtags_add_match(
    mut st: *mut findtags_state_T,
    mut tagpp: *mut tagptrs_T,
    mut margs: *mut findtags_match_args_T,
    mut buf_ffname: *mut ::core::ffi::c_char,
    mut hash: *mut hash_T,
) {
    unsafe {
        let name_only: bool = (*st).flags & TAG_NAMES as ::core::ffi::c_int != 0;
        let mut len: size_t = 0 as size_t;
        let mut mfp_size: size_t = 0 as size_t;
        let mut mfp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut is_current: bool = test_for_current(
            (*tagpp).fname,
            (*tagpp).fname_end,
            (*st).tag_fname,
            buf_ffname,
        ) != 0;
        let mut is_static: bool = test_for_static(tagpp);
        let mut mtt: ::core::ffi::c_int = if is_static as ::core::ffi::c_int != 0 {
            if is_current as ::core::ffi::c_int != 0 {
                MT_ST_CUR as ::core::ffi::c_int
            } else {
                MT_ST_OTH as ::core::ffi::c_int
            }
        } else if is_current as ::core::ffi::c_int != 0 {
            MT_GL_CUR as ::core::ffi::c_int
        } else {
            MT_GL_OTH as ::core::ffi::c_int
        };
        if (*(*st).orgpat).regmatch.rm_ic as ::core::ffi::c_int != 0 && !(*margs).match_no_ic {
            mtt += MT_IC_OFF as ::core::ffi::c_int;
        }
        if (*margs).match_re {
            mtt += MT_RE_OFF as ::core::ffi::c_int;
        }
        if (*st).help_only {
            *(*tagpp).tagname_end = NUL as ::core::ffi::c_char;
            len = (*tagpp).tagname_end.offset_from((*tagpp).tagname) as size_t;
            mfp_size = ::core::mem::size_of::<::core::ffi::c_char>()
                .wrapping_add(len as usize)
                .wrapping_add(10 as usize)
                .wrapping_add(ML_EXTRA as usize)
                .wrapping_add(1 as usize) as size_t;
            mfp = xmalloc(mfp_size) as *mut ::core::ffi::c_char;
            let mut p: *mut ::core::ffi::c_char = mfp;
            strcpy(p, (*tagpp).tagname);
            *p.offset(len as isize) = '@' as ::core::ffi::c_char;
            strcpy(
                p.offset(len as isize)
                    .offset(1 as ::core::ffi::c_int as isize),
                &raw mut (*st).help_lang as *mut ::core::ffi::c_char,
            );
            snprintf(
                p.offset(len as isize)
                    .offset(1 as ::core::ffi::c_int as isize)
                    .offset(ML_EXTRA as isize),
                mfp_size.wrapping_sub(
                    len.wrapping_add(1 as size_t)
                        .wrapping_add(ML_EXTRA as size_t),
                ),
                b"%06d\0".as_ptr() as *const ::core::ffi::c_char,
                help_heuristic(
                    (*tagpp).tagname,
                    if (*margs).match_re as ::core::ffi::c_int != 0 {
                        (*margs).matchoff
                    } else {
                        0 as ::core::ffi::c_int
                    },
                    !(*margs).match_no_ic,
                ) + (*st).help_pri,
            );
            *(*tagpp).tagname_end = TAB as ::core::ffi::c_char;
        } else if name_only {
            if (*st).get_searchpat {
                let mut temp_end: *mut ::core::ffi::c_char = (*tagpp).command;
                if *temp_end as ::core::ffi::c_int == '/' as ::core::ffi::c_int {
                    while *temp_end as ::core::ffi::c_int != 0
                        && *temp_end as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
                        && *temp_end as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                        && *temp_end as ::core::ffi::c_int != '$' as ::core::ffi::c_int
                    {
                        temp_end = temp_end.offset(1);
                    }
                }
                if (*tagpp).command.offset(2 as ::core::ffi::c_int as isize) < temp_end {
                    len = (temp_end.offset_from((*tagpp).command) - 2 as isize) as size_t;
                    mfp = xmalloc(len.wrapping_add(2 as size_t)) as *mut ::core::ffi::c_char;
                    xmemcpyz(
                        mfp as *mut ::core::ffi::c_void,
                        (*tagpp).command.offset(2 as ::core::ffi::c_int as isize)
                            as *const ::core::ffi::c_void,
                        len,
                    );
                } else {
                    mfp = ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                (*st).get_searchpat = false_0 != 0;
            } else {
                len = (*tagpp).tagname_end.offset_from((*tagpp).tagname) as size_t;
                mfp = xmalloc(
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_add(len)
                        .wrapping_add(1 as size_t),
                ) as *mut ::core::ffi::c_char;
                xmemcpyz(
                    mfp as *mut ::core::ffi::c_void,
                    (*tagpp).tagname as *const ::core::ffi::c_void,
                    len,
                );
                if State.get() & MODE_INSERT != 0 {
                    (*st).get_searchpat = p_sft.get() != 0;
                }
            }
        } else {
            let mut tag_fname_len: size_t = strlen((*st).tag_fname);
            len = tag_fname_len
                .wrapping_add(strlen((*st).lbuf))
                .wrapping_add(3 as size_t);
            mfp = xmalloc(
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_add(len)
                    .wrapping_add(1 as size_t),
            ) as *mut ::core::ffi::c_char;
            let mut p_0: *mut ::core::ffi::c_char = mfp;
            *p_0.offset(0 as ::core::ffi::c_int as isize) =
                (mtt + 1 as ::core::ffi::c_int) as ::core::ffi::c_char;
            strcpy(
                p_0.offset(1 as ::core::ffi::c_int as isize),
                (*st).tag_fname,
            );
            *p_0.offset(tag_fname_len.wrapping_add(1 as size_t) as isize) =
                TAG_SEP as ::core::ffi::c_char;
            let mut s: *mut ::core::ffi::c_char = p_0
                .offset(1 as ::core::ffi::c_int as isize)
                .offset(tag_fname_len as isize)
                .offset(1 as ::core::ffi::c_int as isize);
            strcpy(s, (*st).lbuf);
        }
        if !mfp.is_null() {
            let mut hi: *mut hashitem_T = ::core::ptr::null_mut::<hashitem_T>();
            *hash = hash_hash(mfp);
            hi = hash_lookup(
                (&raw mut (*st).ht_match as *mut hashtab_T).offset(mtt as isize),
                mfp,
                strlen(mfp),
                *hash,
            );
            if (*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
            {
                hash_add_item(
                    (&raw mut (*st).ht_match as *mut hashtab_T).offset(mtt as isize),
                    hi,
                    mfp,
                    *hash,
                );
                ga_grow(
                    (&raw mut (*st).ga_match as *mut garray_T).offset(mtt as isize),
                    1 as ::core::ffi::c_int,
                );
                *((*st).ga_match[mtt as usize].ga_data as *mut *mut ::core::ffi::c_char)
                    .offset((*st).ga_match[mtt as usize].ga_len as isize) = mfp;
                (*st).ga_match[mtt as usize].ga_len += 1;
                (*st).match_count += 1;
            } else {
                xfree(mfp as *mut ::core::ffi::c_void);
            }
        }
    }
}
