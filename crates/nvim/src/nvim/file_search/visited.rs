//! The search context's stack and its "already been here" lists.
//!
//! The walk keeps a stack of directories still to look at and, beside it, a
//! list of the files and directories it has already reported, so that links
//! and self-referencing directories cannot make it loop.
//! [`ff_check_visited`] is the test: it compares by file id rather than by
//! name, and treats two entries as the same only when their wildcard tails
//! agree as well ([`ff_wc_equal`], which ignores the counter byte behind a
//! `**`).

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn vim_findfile_free_visited(
    mut search_ctx_arg: *mut ::core::ffi::c_void,
) {
    unsafe {
        if search_ctx_arg.is_null() {
            return;
        }
        let mut search_ctx: *mut ff_search_ctx_T = search_ctx_arg as *mut ff_search_ctx_T;
        vim_findfile_free_visited_list(&raw mut (*search_ctx).ffsc_visited_lists_list);
        vim_findfile_free_visited_list(&raw mut (*search_ctx).ffsc_dir_visited_lists_list);
    }
}

pub(crate) unsafe extern "C" fn vim_findfile_free_visited_list(
    mut list_headp: *mut *mut ff_visited_list_hdr_T,
) {
    unsafe {
        let mut vp: *mut ff_visited_list_hdr_T = ::core::ptr::null_mut::<ff_visited_list_hdr_T>();
        while !(*list_headp).is_null() {
            vp = (**list_headp).ffvl_next as *mut ff_visited_list_hdr_T;
            ff_free_visited_list((**list_headp).ffvl_visited_list);
            xfree((**list_headp).ffvl_filename as *mut ::core::ffi::c_void);
            xfree(*list_headp as *mut ::core::ffi::c_void);
            *list_headp = vp;
        }
        *list_headp = ::core::ptr::null_mut::<ff_visited_list_hdr_T>();
    }
}

pub(crate) unsafe extern "C" fn ff_free_visited_list(mut vl: *mut ff_visited_T) {
    unsafe {
        let mut vp: *mut ff_visited_T = ::core::ptr::null_mut::<ff_visited_T>();
        while !vl.is_null() {
            vp = (*vl).ffv_next as *mut ff_visited_T;
            xfree((*vl).ffv_wc_path as *mut ::core::ffi::c_void);
            xfree(vl as *mut ::core::ffi::c_void);
            vl = vp;
        }
        vl = ::core::ptr::null_mut::<ff_visited_T>();
    }
}

pub(crate) unsafe extern "C" fn ff_get_visited_list(
    mut filename: *mut ::core::ffi::c_char,
    mut filenamelen: size_t,
    mut list_headp: *mut *mut ff_visited_list_hdr_T,
) -> *mut ff_visited_list_hdr_T {
    unsafe {
        let mut retptr: *mut ff_visited_list_hdr_T =
            ::core::ptr::null_mut::<ff_visited_list_hdr_T>();
        if !(*list_headp).is_null() {
            retptr = *list_headp;
            while !retptr.is_null() {
                if path_fnamecmp(filename, (*retptr).ffvl_filename) == 0 as ::core::ffi::c_int {
                    return retptr;
                }
                retptr = (*retptr).ffvl_next as *mut ff_visited_list_hdr_T;
            }
        }
        retptr =
            xmalloc(::core::mem::size_of::<ff_visited_list_hdr_T>()) as *mut ff_visited_list_hdr_T;
        (*retptr).ffvl_visited_list = ::core::ptr::null_mut::<ff_visited_T>();
        (*retptr).ffvl_filename = xmemdupz(filename as *const ::core::ffi::c_void, filenamelen)
            as *mut ::core::ffi::c_char;
        (*retptr).ffvl_next = *list_headp as *mut ff_visited_list_hdr;
        *list_headp = retptr;
        return retptr;
    }
}

pub(crate) unsafe extern "C" fn ff_wc_equal(
    mut s1: *mut ::core::ffi::c_char,
    mut s2: *mut ::core::ffi::c_char,
) -> bool {
    unsafe {
        let mut i: ::core::ffi::c_int = 0;
        let mut j: ::core::ffi::c_int = 0;
        let mut prev1: ::core::ffi::c_int = NUL;
        let mut prev2: ::core::ffi::c_int = NUL;
        if s1 == s2 {
            return true_0 != 0;
        }
        if s1.is_null() || s2.is_null() {
            return false_0 != 0;
        }
        i = 0 as ::core::ffi::c_int;
        j = 0 as ::core::ffi::c_int;
        while *s1.offset(i as isize) as ::core::ffi::c_int != NUL
            && *s2.offset(j as isize) as ::core::ffi::c_int != NUL
        {
            let mut c1: ::core::ffi::c_int = utf_ptr2char(s1.offset(i as isize));
            let mut c2: ::core::ffi::c_int = utf_ptr2char(s2.offset(j as isize));
            if (if p_fic.get() != 0 {
                (mb_tolower(c1) != mb_tolower(c2)) as ::core::ffi::c_int
            } else {
                (c1 != c2) as ::core::ffi::c_int
            }) != 0
                && (prev1 != '*' as ::core::ffi::c_int || prev2 != '*' as ::core::ffi::c_int)
            {
                return false_0 != 0;
            }
            prev2 = prev1;
            prev1 = c1;
            i += utfc_ptr2len(s1.offset(i as isize));
            j += utfc_ptr2len(s2.offset(j as isize));
        }
        return *s1.offset(i as isize) as ::core::ffi::c_int
            == *s2.offset(j as isize) as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn ff_check_visited(
    mut visited_list: *mut *mut ff_visited_T,
    mut fname: *mut ::core::ffi::c_char,
    mut fnamelen: size_t,
    mut wc_path: *mut ::core::ffi::c_char,
    mut wc_pathlen: size_t,
) -> ::core::ffi::c_int {
    unsafe {
        let mut vp: *mut ff_visited_T = ::core::ptr::null_mut::<ff_visited_T>();
        let mut url: bool = false_0 != 0;
        let mut file_id: FileID = FileID {
            inode: 0,
            device_id: 0,
        };
        if path_with_url(fname) != 0 {
            xmemcpyz(
                (*ff_expand_buffer.ptr()).data as *mut ::core::ffi::c_void,
                fname as *const ::core::ffi::c_void,
                fnamelen,
            );
            (*ff_expand_buffer.ptr()).size = fnamelen;
            url = true_0 != 0;
        } else {
            *(*ff_expand_buffer.ptr())
                .data
                .offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
            (*ff_expand_buffer.ptr()).size = 0 as size_t;
            if !os_fileid(fname, &raw mut file_id) {
                return FAIL;
            }
        }
        vp = *visited_list;
        while !vp.is_null() {
            if url as ::core::ffi::c_int != 0
                && path_fnamecmp(
                    &raw mut (*vp).ffv_fname as *mut ::core::ffi::c_char,
                    (*ff_expand_buffer.ptr()).data,
                ) == 0 as ::core::ffi::c_int
                || !url
                    && (*vp).file_id_valid as ::core::ffi::c_int != 0
                    && os_fileid_equal(&raw mut (*vp).file_id, &raw mut file_id)
                        as ::core::ffi::c_int
                        != 0
            {
                if ff_wc_equal((*vp).ffv_wc_path, wc_path) {
                    return FAIL;
                }
            }
            vp = (*vp).ffv_next as *mut ff_visited_T;
        }
        vp = xmalloc(
            (40 as size_t)
                .wrapping_add((*ff_expand_buffer.ptr()).size)
                .wrapping_add(1 as size_t),
        ) as *mut ff_visited_T;
        if !url {
            (*vp).file_id_valid = true_0 != 0;
            (*vp).file_id = file_id;
            *(&raw mut (*vp).ffv_fname as *mut ::core::ffi::c_char)
                .offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        } else {
            (*vp).file_id_valid = false_0 != 0;
            strcpy(
                &raw mut (*vp).ffv_fname as *mut ::core::ffi::c_char,
                (*ff_expand_buffer.ptr()).data,
            );
        }
        if !wc_path.is_null() {
            (*vp).ffv_wc_path = xmemdupz(wc_path as *const ::core::ffi::c_void, wc_pathlen)
                as *mut ::core::ffi::c_char;
        } else {
            (*vp).ffv_wc_path = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        (*vp).ffv_next = *visited_list as *mut ff_visited;
        *visited_list = vp;
        return OK;
    }
}

pub(crate) unsafe extern "C" fn ff_create_stack_element(
    mut fix_part: *mut ::core::ffi::c_char,
    mut fix_partlen: size_t,
    mut wc_part: *mut ::core::ffi::c_char,
    mut wc_partlen: size_t,
    mut level: ::core::ffi::c_int,
    mut star_star_empty: ::core::ffi::c_int,
) -> *mut ff_stack_T {
    unsafe {
        let mut stack: *mut ff_stack_T =
            xmalloc(::core::mem::size_of::<ff_stack_T>()) as *mut ff_stack_T;
        (*stack).ffs_prev = ::core::ptr::null_mut::<ff_stack>();
        (*stack).ffs_filearray = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        (*stack).ffs_filearray_size = 0 as ::core::ffi::c_int;
        (*stack).ffs_filearray_cur = 0 as ::core::ffi::c_int;
        (*stack).ffs_stage = 0 as ::core::ffi::c_int;
        (*stack).ffs_level = level;
        (*stack).ffs_star_star_empty = star_star_empty;
        if fix_part.is_null() {
            fix_part = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            fix_partlen = 0 as size_t;
        }
        (*stack).ffs_fix_path = cbuf_to_string(fix_part, fix_partlen);
        if wc_part.is_null() {
            wc_part = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            wc_partlen = 0 as size_t;
        }
        (*stack).ffs_wc_path = cbuf_to_string(wc_part, wc_partlen);
        return stack;
    }
}

pub(crate) unsafe extern "C" fn ff_push(
    mut search_ctx: *mut ff_search_ctx_T,
    mut stack_ptr: *mut ff_stack_T,
) {
    unsafe {
        if stack_ptr.is_null() {
            return;
        }
        (*stack_ptr).ffs_prev = (*search_ctx).ffsc_stack_ptr as *mut ff_stack;
        (*search_ctx).ffsc_stack_ptr = stack_ptr;
    }
}

pub(crate) unsafe extern "C" fn ff_pop(mut search_ctx: *mut ff_search_ctx_T) -> *mut ff_stack_T {
    unsafe {
        let mut sptr: *mut ff_stack_T = (*search_ctx).ffsc_stack_ptr;
        if !(*search_ctx).ffsc_stack_ptr.is_null() {
            (*search_ctx).ffsc_stack_ptr =
                (*(*search_ctx).ffsc_stack_ptr).ffs_prev as *mut ff_stack_T;
        }
        return sptr;
    }
}

pub(crate) unsafe extern "C" fn ff_free_stack_element(stack_ptr: *mut ff_stack_T) {
    unsafe {
        if stack_ptr.is_null() {
            return;
        }
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*stack_ptr).ffs_fix_path.data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        (*stack_ptr).ffs_fix_path.size = 0 as size_t;
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut (*stack_ptr).ffs_wc_path.data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL;
        let _ = *ptr__0;
        (*stack_ptr).ffs_wc_path.size = 0 as size_t;
        if !(*stack_ptr).ffs_filearray.is_null() {
            FreeWild((*stack_ptr).ffs_filearray_size, (*stack_ptr).ffs_filearray);
        }
        xfree(stack_ptr as *mut ::core::ffi::c_void);
    }
}

pub(crate) unsafe extern "C" fn ff_clear(mut search_ctx: *mut ff_search_ctx_T) {
    unsafe {
        let mut sptr: *mut ff_stack_T = ::core::ptr::null_mut::<ff_stack_T>();
        loop {
            sptr = ff_pop(search_ctx);
            if sptr.is_null() {
                break;
            }
            ff_free_stack_element(sptr);
        }
        if !(*search_ctx).ffsc_stopdirs_v.is_null() {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while !(*(*search_ctx).ffsc_stopdirs_v.offset(i as isize))
                .data
                .is_null()
            {
                xfree(
                    (*(*search_ctx).ffsc_stopdirs_v.offset(i as isize)).data
                        as *mut ::core::ffi::c_void,
                );
                i += 1;
            }
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*search_ctx).ffsc_stopdirs_v as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
        }
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut (*search_ctx).ffsc_file_to_search.data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL;
        let _ = *ptr__0;
        (*search_ctx).ffsc_file_to_search.size = 0 as size_t;
        let mut ptr__1: *mut *mut ::core::ffi::c_void =
            &raw mut (*search_ctx).ffsc_start_dir.data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__1);
        *ptr__1 = NULL;
        let _ = *ptr__1;
        (*search_ctx).ffsc_start_dir.size = 0 as size_t;
        let mut ptr__2: *mut *mut ::core::ffi::c_void =
            &raw mut (*search_ctx).ffsc_fix_path.data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__2);
        *ptr__2 = NULL;
        let _ = *ptr__2;
        (*search_ctx).ffsc_fix_path.size = 0 as size_t;
        let mut ptr__3: *mut *mut ::core::ffi::c_void =
            &raw mut (*search_ctx).ffsc_wc_path.data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__3);
        *ptr__3 = NULL;
        let _ = *ptr__3;
        (*search_ctx).ffsc_wc_path.size = 0 as size_t;
        (*search_ctx).ffsc_level = 0 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn ff_path_in_stoplist(
    mut path: *mut ::core::ffi::c_char,
    mut path_len: size_t,
    mut stopdirs_v: *mut String_0,
) -> bool {
    unsafe {
        while path_len > 1 as size_t
            && vim_ispathsep(
                *path.offset(path_len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
            ) as ::core::ffi::c_int
                != 0
        {
            path_len = path_len.wrapping_sub(1);
        }
        if path_len == 0 as size_t {
            return true_0 != 0;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while !(*stopdirs_v.offset(i as isize)).data.is_null() {
            if path_fnamencmp((*stopdirs_v.offset(i as isize)).data, path, path_len)
                == 0 as ::core::ffi::c_int
                && ((*stopdirs_v.offset(i as isize)).size <= path_len
                    || vim_ispathsep(
                        *(*stopdirs_v.offset(i as isize))
                            .data
                            .offset(path_len as isize)
                            as ::core::ffi::c_int,
                    ) as ::core::ffi::c_int
                        != 0)
            {
                return true_0 != 0;
            }
            i += 1;
        }
        return false_0 != 0;
    }
}
