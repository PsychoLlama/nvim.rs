//! Finding an entry, and resolving the file it names.
//!
//! [`qf_get_fnum`] turns the file name a parsed entry carries into a buffer
//! number, applying the directory stack that `%D`/`%X` maintain
//! ([`qf_push_dir`], [`qf_guess_filepath`]). The `*_valid_entry` walkers
//! are how `:cnext` and friends skip entries that name no real position.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

/// The file name the last entry was filed under, and the buffer it named.
/// Consecutive entries usually name the same file, so remembering the last
/// answer saves a `buflist_new` walk per entry.
pub(crate) static qf_last_bufname: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub(crate) static qf_last_bufref: GlobalCell<bufref_T> = GlobalCell::new(bufref_T::new());

pub(crate) unsafe extern "C" fn qf_get_fnum(
    mut qfl: *mut qf_list_T,
    mut directory: *mut ::core::ffi::c_char,
    mut fname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut bufname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        if fname.is_null() || *fname as ::core::ffi::c_int == NUL {
            return 0 as ::core::ffi::c_int;
        }
        if !directory.is_null() && !vim_isAbsName(fname) {
            ptr = concat_fnames(directory, fname, true_0 != 0);
            if !os_path_exists(ptr) {
                xfree(ptr as *mut ::core::ffi::c_void);
                directory = qf_guess_filepath(qfl, fname);
                if !directory.is_null() {
                    ptr = concat_fnames(directory, fname, true_0 != 0);
                } else {
                    ptr = xstrdup(fname);
                }
            }
            bufname = ptr;
        } else {
            bufname = fname;
        }
        if !(*qf_last_bufname.ptr()).is_null()
            && strcmp(bufname, qf_last_bufname.get()) == 0 as ::core::ffi::c_int
            && bufref_valid(qf_last_bufref.ptr()) as ::core::ffi::c_int != 0
        {
            buf = (*qf_last_bufref.ptr()).br_buf;
            xfree(ptr as *mut ::core::ffi::c_void);
        } else {
            xfree(qf_last_bufname.get() as *mut ::core::ffi::c_void);
            buf = buflist_new(
                bufname,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                0 as linenr_T,
                BLN_NOOPT as ::core::ffi::c_int,
            );
            qf_last_bufname.set(if bufname == ptr {
                bufname
            } else {
                xstrdup(bufname)
            });
            set_bufref(qf_last_bufref.ptr(), buf);
        }
        if buf.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        (*buf).b_has_qf_entry = if (*qfl).qfl_type as ::core::ffi::c_uint
            == QFLT_QUICKFIX as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            BUF_HAS_QF_ENTRY
        } else {
            BUF_HAS_LL_ENTRY
        };
        return (*buf).handle as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_push_dir(
    mut dirbuf: *mut ::core::ffi::c_char,
    mut stackptr: *mut *mut dir_stack_T,
    mut is_file_stack: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut ds_ptr: *mut dir_stack_T = ::core::ptr::null_mut::<dir_stack_T>();
        let mut ds_new: *mut dir_stack_T =
            xmalloc(::core::mem::size_of::<dir_stack_T>()) as *mut dir_stack_T;
        (*ds_new).next = *stackptr;
        *stackptr = ds_new;
        if vim_isAbsName(dirbuf) as ::core::ffi::c_int != 0
            || (**stackptr).next.is_null()
            || is_file_stack as ::core::ffi::c_int != 0
        {
            (**stackptr).dirname = xstrdup(dirbuf);
        } else {
            ds_new = (**stackptr).next;
            (**stackptr).dirname = ::core::ptr::null_mut::<::core::ffi::c_char>();
            while !ds_new.is_null() {
                let mut dirname: *mut ::core::ffi::c_char =
                    concat_fnames((*ds_new).dirname, dirbuf, true_0 != 0);
                if os_isdir(dirname) {
                    xfree((**stackptr).dirname as *mut ::core::ffi::c_void);
                    (**stackptr).dirname = dirname;
                    break;
                } else {
                    xfree(dirname as *mut ::core::ffi::c_void);
                    ds_new = (*ds_new).next;
                }
            }
            while (**stackptr).next != ds_new {
                ds_ptr = (**stackptr).next;
                (**stackptr).next = (*(**stackptr).next).next;
                xfree((*ds_ptr).dirname as *mut ::core::ffi::c_void);
                xfree(ds_ptr as *mut ::core::ffi::c_void);
            }
            if ds_new.is_null() {
                xfree((**stackptr).dirname as *mut ::core::ffi::c_void);
                (**stackptr).dirname = xstrdup(dirbuf);
            }
        }
        if !(**stackptr).dirname.is_null() {
            return (**stackptr).dirname;
        }
        ds_ptr = *stackptr;
        *stackptr = (**stackptr).next;
        xfree(ds_ptr as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

pub(crate) unsafe extern "C" fn qf_pop_dir(
    mut stackptr: *mut *mut dir_stack_T,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if !(*stackptr).is_null() {
            let mut ds_ptr: *mut dir_stack_T = *stackptr;
            *stackptr = (**stackptr).next;
            xfree((*ds_ptr).dirname as *mut ::core::ffi::c_void);
            xfree(ds_ptr as *mut ::core::ffi::c_void);
        }
        return if !(*stackptr).is_null() {
            (**stackptr).dirname
        } else {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        };
    }
}

pub(crate) unsafe extern "C" fn qf_clean_dir_stack(mut stackptr: *mut *mut dir_stack_T) {
    unsafe {
        let mut ds_ptr: *mut dir_stack_T = ::core::ptr::null_mut::<dir_stack_T>();
        loop {
            ds_ptr = *stackptr;
            if ds_ptr.is_null() {
                break;
            }
            *stackptr = (**stackptr).next;
            xfree((*ds_ptr).dirname as *mut ::core::ffi::c_void);
            xfree(ds_ptr as *mut ::core::ffi::c_void);
        }
    }
}

pub(crate) unsafe extern "C" fn qf_guess_filepath(
    mut qfl: *mut qf_list_T,
    mut filename: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if (*qfl).qf_dir_stack.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut ds_ptr: *mut dir_stack_T = (*(*qfl).qf_dir_stack).next;
        let mut fullname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        while !ds_ptr.is_null() {
            xfree(fullname as *mut ::core::ffi::c_void);
            fullname = concat_fnames((*ds_ptr).dirname, filename, true_0 != 0);
            if os_path_exists(fullname) {
                break;
            }
            ds_ptr = (*ds_ptr).next;
        }
        xfree(fullname as *mut ::core::ffi::c_void);
        while (*(*qfl).qf_dir_stack).next != ds_ptr {
            let mut ds_tmp: *mut dir_stack_T = (*(*qfl).qf_dir_stack).next;
            (*(*qfl).qf_dir_stack).next = (*(*(*qfl).qf_dir_stack).next).next;
            xfree((*ds_tmp).dirname as *mut ::core::ffi::c_void);
            xfree(ds_tmp as *mut ::core::ffi::c_void);
        }
        return if ds_ptr.is_null() {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        } else {
            (*ds_ptr).dirname
        };
    }
}

pub(crate) unsafe extern "C" fn qflist_valid(
    mut wp: *mut win_T,
    mut qf_id: ::core::ffi::c_uint,
) -> bool {
    unsafe {
        let mut qi: *mut qf_info_T = ql_info.get();
        if !wp.is_null() {
            if !win_valid(wp) {
                return false_0 != 0;
            }
            qi = if bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0
                && !(*wp).w_llist_ref.is_null()
            {
                (*wp).w_llist_ref
            } else {
                (*wp).w_llist
            };
        }
        if qi.is_null() {
            return false_0 != 0;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*qi).qf_listcount {
            if (*qf_get_list(qi, i)).qf_id == qf_id {
                return true_0 != 0;
            }
            i += 1;
        }
        return false_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn is_qf_entry_present(
    mut qfl: *mut qf_list_T,
    mut qf_ptr: *mut qfline_T,
) -> bool {
    unsafe {
        let mut qfp: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
        let mut i: ::core::ffi::c_int = 0;
        i = 1 as ::core::ffi::c_int;
        qfp = (*qfl).qf_start;
        while !got_int.get() && i <= (*qfl).qf_count && !qfp.is_null() {
            if qfp == qf_ptr {
                break;
            }
            i += 1;
            qfp = (*qfp).qf_next;
        }
        if i > (*qfl).qf_count {
            return false_0 != 0;
        }
        return true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn get_next_valid_entry(
    mut qfl: *mut qf_list_T,
    mut qf_ptr: *mut qfline_T,
    mut qf_index: *mut ::core::ffi::c_int,
    mut dir: ::core::ffi::c_int,
) -> *mut qfline_T {
    unsafe {
        let mut idx: ::core::ffi::c_int = *qf_index;
        let mut old_qf_fnum: ::core::ffi::c_int = (*qf_ptr).qf_fnum;
        loop {
            if idx == (*qfl).qf_count || (*qf_ptr).qf_next.is_null() {
                return ::core::ptr::null_mut::<qfline_T>();
            }
            idx += 1;
            qf_ptr = (*qf_ptr).qf_next;
            if !(!(*qfl).qf_nonevalid && (*qf_ptr).qf_valid == 0
                || dir == FORWARD_FILE as ::core::ffi::c_int && (*qf_ptr).qf_fnum == old_qf_fnum)
            {
                break;
            }
        }
        *qf_index = idx;
        return qf_ptr;
    }
}

pub(crate) unsafe extern "C" fn get_prev_valid_entry(
    mut qfl: *mut qf_list_T,
    mut qf_ptr: *mut qfline_T,
    mut qf_index: *mut ::core::ffi::c_int,
    mut dir: ::core::ffi::c_int,
) -> *mut qfline_T {
    unsafe {
        let mut idx: ::core::ffi::c_int = *qf_index;
        let mut old_qf_fnum: ::core::ffi::c_int = (*qf_ptr).qf_fnum;
        loop {
            if idx == 1 as ::core::ffi::c_int || (*qf_ptr).qf_prev.is_null() {
                return ::core::ptr::null_mut::<qfline_T>();
            }
            idx -= 1;
            qf_ptr = (*qf_ptr).qf_prev;
            if !(!(*qfl).qf_nonevalid && (*qf_ptr).qf_valid == 0
                || dir == BACKWARD_FILE as ::core::ffi::c_int && (*qf_ptr).qf_fnum == old_qf_fnum)
            {
                break;
            }
        }
        *qf_index = idx;
        return qf_ptr;
    }
}

pub(crate) unsafe extern "C" fn get_nth_valid_entry(
    mut qfl: *mut qf_list_T,
    mut errornr: ::core::ffi::c_int,
    mut dir: ::core::ffi::c_int,
    mut new_qfidx: *mut ::core::ffi::c_int,
) -> *mut qfline_T {
    unsafe {
        let mut qf_ptr: *mut qfline_T = (*qfl).qf_ptr;
        let mut qf_idx: ::core::ffi::c_int = (*qfl).qf_index;
        let mut err: *const ::core::ffi::c_char = e_no_more_items.get();
        loop {
            let c2rust_fresh22 = errornr;
            errornr = errornr - 1;
            if c2rust_fresh22 == 0 {
                break;
            }
            let mut prev_qf_ptr: *mut qfline_T = qf_ptr;
            let mut prev_index: ::core::ffi::c_int = qf_idx;
            if dir == FORWARD as ::core::ffi::c_int || dir == FORWARD_FILE as ::core::ffi::c_int {
                qf_ptr = get_next_valid_entry(qfl, qf_ptr, &raw mut qf_idx, dir);
            } else {
                qf_ptr = get_prev_valid_entry(qfl, qf_ptr, &raw mut qf_idx, dir);
            }
            if qf_ptr.is_null() {
                qf_ptr = prev_qf_ptr;
                qf_idx = prev_index;
                if !err.is_null() {
                    emsg(gettext(err));
                    return ::core::ptr::null_mut::<qfline_T>();
                }
                break;
            } else {
                err = ::core::ptr::null::<::core::ffi::c_char>();
            }
        }
        *new_qfidx = qf_idx;
        return qf_ptr;
    }
}

pub(crate) unsafe extern "C" fn get_nth_entry(
    mut qfl: *mut qf_list_T,
    mut errornr: ::core::ffi::c_int,
    mut new_qfidx: *mut ::core::ffi::c_int,
) -> *mut qfline_T {
    unsafe {
        let mut qf_ptr: *mut qfline_T = (*qfl).qf_ptr;
        let mut qf_idx: ::core::ffi::c_int = (*qfl).qf_index;
        while errornr < qf_idx && qf_idx > 1 as ::core::ffi::c_int && !(*qf_ptr).qf_prev.is_null() {
            qf_idx -= 1;
            qf_ptr = (*qf_ptr).qf_prev;
        }
        while errornr > qf_idx && qf_idx < (*qfl).qf_count && !(*qf_ptr).qf_next.is_null() {
            qf_idx += 1;
            qf_ptr = (*qf_ptr).qf_next;
        }
        *new_qfidx = qf_idx;
        return qf_ptr;
    }
}

pub(crate) unsafe extern "C" fn qf_get_entry(
    mut qfl: *mut qf_list_T,
    mut errornr: ::core::ffi::c_int,
    mut dir: ::core::ffi::c_int,
    mut new_qfidx: *mut ::core::ffi::c_int,
) -> *mut qfline_T {
    unsafe {
        let mut qf_ptr: *mut qfline_T = (*qfl).qf_ptr;
        let mut qfidx: ::core::ffi::c_int = (*qfl).qf_index;
        if dir != 0 as ::core::ffi::c_int {
            qf_ptr = get_nth_valid_entry(qfl, errornr, dir, &raw mut qfidx);
        } else if errornr != 0 as ::core::ffi::c_int {
            qf_ptr = get_nth_entry(qfl, errornr, &raw mut qfidx);
        }
        *new_qfidx = qfidx;
        return qf_ptr;
    }
}

pub unsafe extern "C" fn qf_get_size(mut eap: *mut exarg_T) -> size_t {
    unsafe {
        let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
        qi = qf_cmd_get_stack(eap, false_0 != 0);
        if qi.is_null() {
            return 0 as size_t;
        }
        return (*qf_get_curlist(qi)).qf_count as size_t;
    }
}

pub unsafe extern "C" fn qf_get_valid_size(mut eap: *mut exarg_T) -> size_t {
    unsafe {
        let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
        qi = qf_cmd_get_stack(eap, false_0 != 0);
        if qi.is_null() {
            return 0 as size_t;
        }
        let mut prev_fnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut sz: size_t = 0 as size_t;
        let mut qfp: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
        let mut i: ::core::ffi::c_int = 0;
        '_c2rust_label: {
            if (*qf_get_curlist(qi)).qf_count >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"qf_get_curlist(qi)->qf_count >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    4760 as ::core::ffi::c_uint,
                    b"size_t qf_get_valid_size(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let mut qfl: *mut qf_list_T = qf_get_curlist(qi);
        i = 1 as ::core::ffi::c_int;
        qfp = (*qfl).qf_start;
        while !got_int.get() && i <= (*qfl).qf_count && !qfp.is_null() {
            if (*qfp).qf_valid != 0 {
                if (*eap).cmdidx as ::core::ffi::c_int == CMD_cdo as ::core::ffi::c_int
                    || (*eap).cmdidx as ::core::ffi::c_int == CMD_ldo as ::core::ffi::c_int
                {
                    sz = sz.wrapping_add(1);
                } else if (*qfp).qf_fnum > 0 as ::core::ffi::c_int && (*qfp).qf_fnum != prev_fnum {
                    sz = sz.wrapping_add(1);
                    prev_fnum = (*qfp).qf_fnum;
                }
            }
            i += 1;
            qfp = (*qfp).qf_next;
        }
        return sz;
    }
}

pub unsafe extern "C" fn qf_get_cur_idx(mut eap: *mut exarg_T) -> size_t {
    unsafe {
        let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
        qi = qf_cmd_get_stack(eap, false_0 != 0);
        if qi.is_null() {
            return 0 as size_t;
        }
        '_c2rust_label: {
            if (*qf_get_curlist(qi)).qf_index >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"qf_get_curlist(qi)->qf_index >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    4791 as ::core::ffi::c_uint,
                    b"size_t qf_get_cur_idx(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        return (*qf_get_curlist(qi)).qf_index as size_t;
    }
}

pub unsafe extern "C" fn qf_get_cur_valid_idx(mut eap: *mut exarg_T) -> ::core::ffi::c_int {
    unsafe {
        let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
        qi = qf_cmd_get_stack(eap, false_0 != 0);
        if qi.is_null() {
            return 1 as ::core::ffi::c_int;
        }
        let mut qfl: *mut qf_list_T = qf_get_curlist(qi);
        if !qf_list_has_valid_entries(qfl) {
            return 1 as ::core::ffi::c_int;
        }
        let mut prev_fnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut eidx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut qfp: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
        let mut i: size_t = 0;
        '_c2rust_label: {
            if (*qfl).qf_index >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"qfl->qf_index >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    4818 as ::core::ffi::c_uint,
                    b"int qf_get_cur_valid_idx(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        i = 1 as size_t;
        qfp = (*qfl).qf_start;
        while i <= (*qfl).qf_index as size_t && !qfp.is_null() {
            if (*qfp).qf_valid != 0 {
                if (*eap).cmdidx as ::core::ffi::c_int == CMD_cfdo as ::core::ffi::c_int
                    || (*eap).cmdidx as ::core::ffi::c_int == CMD_lfdo as ::core::ffi::c_int
                {
                    if (*qfp).qf_fnum > 0 as ::core::ffi::c_int && (*qfp).qf_fnum != prev_fnum {
                        eidx += 1;
                        prev_fnum = (*qfp).qf_fnum;
                    }
                } else {
                    eidx += 1;
                }
            }
            i = i.wrapping_add(1);
            qfp = (*qfp).qf_next;
        }
        return if eidx != 0 as ::core::ffi::c_int {
            eidx
        } else {
            1 as ::core::ffi::c_int
        };
    }
}

pub(crate) unsafe extern "C" fn qf_get_nth_valid_entry(
    mut qfl: *mut qf_list_T,
    mut n: size_t,
    mut fdo: bool,
) -> size_t {
    unsafe {
        if !qf_list_has_valid_entries(qfl) {
            return 1 as size_t;
        }
        let mut prev_fnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut eidx: size_t = 0 as size_t;
        let mut i: ::core::ffi::c_int = 0;
        let mut qfp: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
        '_c2rust_label: {
            if (*qfl).qf_count >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"qfl->qf_count >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    4857 as ::core::ffi::c_uint,
                    b"size_t qf_get_nth_valid_entry(qf_list_T *, size_t, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        i = 1 as ::core::ffi::c_int;
        qfp = (*qfl).qf_start;
        while !got_int.get() && i <= (*qfl).qf_count && !qfp.is_null() {
            if (*qfp).qf_valid != 0 {
                if fdo {
                    if (*qfp).qf_fnum > 0 as ::core::ffi::c_int && (*qfp).qf_fnum != prev_fnum {
                        eidx = eidx.wrapping_add(1);
                        prev_fnum = (*qfp).qf_fnum;
                    }
                } else {
                    eidx = eidx.wrapping_add(1);
                }
            }
            if eidx == n {
                break;
            }
            i += 1;
            qfp = (*qfp).qf_next;
        }
        return if i <= (*qfl).qf_count {
            i as size_t
        } else {
            1 as size_t
        };
    }
}
