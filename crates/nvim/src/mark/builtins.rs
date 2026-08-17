use crate::buffer::buflist_nr2name;
use crate::eval::typval::{
    tv_dict_add_list, tv_dict_add_str, tv_dict_alloc, tv_list_alloc, tv_list_append_dict,
    tv_list_append_number,
};
use crate::main::{c_bytes, curbuf, curwin, namedfm};
use crate::memory::xfree;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use super::*;
use crate::pos::MAXCOL;
use crate::types::kListLenMayKnow;

/// Add information about mark 'mname' to list 'l'
pub(super) unsafe extern "C" fn add_mark(
    mut l: *mut list_T,
    mut mname: *const c_char,
    mut pos: *const pos_T,
    mut bufnr: c_int,
    mut fname: *const c_char,
) -> c_int {
    if (*pos).lnum <= 0 {
        return OK;
    }
    let mut d: *mut dict_T = tv_dict_alloc();
    tv_list_append_dict(l, d);
    let mut lpos: *mut list_T = tv_list_alloc(kListLenMayKnow as c_int as ptrdiff_t);
    tv_list_append_number(lpos, bufnr as varnumber_T);
    tv_list_append_number(lpos, (*pos).lnum as varnumber_T);
    tv_list_append_number(
        lpos,
        (if (*pos).col < MAXCOL as c_int {
            (*pos).col as c_int + 1
        } else {
            MAXCOL as c_int
        }) as varnumber_T,
    );
    tv_list_append_number(lpos, (*pos).coladd as varnumber_T);
    if tv_dict_add_str(
        d,
        c"mark".as_ptr(),
        size_of::<[c_char; 5]>().wrapping_sub(1),
        mname,
    ) == FAIL
        || tv_dict_add_list(
            d,
            c"pos".as_ptr(),
            size_of::<[c_char; 4]>().wrapping_sub(1),
            lpos,
        ) == FAIL
        || !fname.is_null()
            && tv_dict_add_str(
                d,
                c"file".as_ptr(),
                size_of::<[c_char; 5]>().wrapping_sub(1),
                fname,
            ) == FAIL
    {
        return FAIL;
    }
    return OK;
}

/// Get information about marks local to a buffer.
///
/// `buf` — Buffer to get the marks from
/// `l` — List to store marks
pub unsafe extern "C" fn get_buf_local_marks(mut buf: *const buf_T, mut l: *mut list_T) {
    let mut mname: [c_char; 3] = c_bytes(b"' \0");
    let mut i: c_int = 0;
    while i < NMARKS {
        mname[1] = ('a' as c_int + i) as c_char;
        add_mark(
            l,
            &raw mut mname as *mut c_char,
            &raw const (*(&raw const (*buf).b_namedm as *const fmark_T).offset(i as isize)).mark,
            (*buf).handle as c_int,
            ptr::null(),
        );
        i += 1;
    }
    add_mark(
        l,
        c"''".as_ptr(),
        &raw mut (*curwin.get()).w_pcmark,
        (*curbuf.get()).handle as c_int,
        ptr::null(),
    );
    add_mark(
        l,
        c"'\"".as_ptr(),
        &raw const (*buf).b_last_cursor.mark,
        (*buf).handle as c_int,
        ptr::null(),
    );
    add_mark(
        l,
        c"'[".as_ptr(),
        &raw const (*buf).b_op_start,
        (*buf).handle as c_int,
        ptr::null(),
    );
    add_mark(
        l,
        c"']".as_ptr(),
        &raw const (*buf).b_op_end,
        (*buf).handle as c_int,
        ptr::null(),
    );
    add_mark(
        l,
        c"'^".as_ptr(),
        &raw const (*buf).b_last_insert.mark,
        (*buf).handle as c_int,
        ptr::null(),
    );
    add_mark(
        l,
        c"'.".as_ptr(),
        &raw const (*buf).b_last_change.mark,
        (*buf).handle as c_int,
        ptr::null(),
    );
    add_mark(
        l,
        c"'<".as_ptr(),
        &raw const (*buf).b_visual.vi_start,
        (*buf).handle as c_int,
        ptr::null(),
    );
    add_mark(
        l,
        c"'>".as_ptr(),
        &raw const (*buf).b_visual.vi_end,
        (*buf).handle as c_int,
        ptr::null(),
    );
}

/// Get information about global marks ('A' to 'Z' and '0' to '9')
///
/// `l` — List to store global marks
pub unsafe extern "C" fn get_global_marks(mut l: *mut list_T) {
    let mut mname: [c_char; 3] = c_bytes(b"' \0");
    let mut name: *mut c_char = ptr::null_mut();
    let mut i: c_int = 0;
    while i < NMARKS + EXTRA_MARKS {
        if (*namedfm.ptr())[i as usize].fmark.fnum != 0 {
            name = buflist_nr2name((*namedfm.ptr())[i as usize].fmark.fnum, true_0, true_0);
        } else {
            name = (*namedfm.ptr())[i as usize].fname;
        }
        if !name.is_null() {
            mname[1] = (if i >= NMARKS {
                (i - NMARKS + '0' as c_int) as c_char as c_int
            } else {
                (i + 'A' as c_int) as c_char as c_int
            }) as c_char;
            add_mark(
                l,
                &raw mut mname as *mut c_char,
                &raw mut (*(namedfm.ptr() as *mut xfmark_T).offset(i as isize))
                    .fmark
                    .mark,
                (*namedfm.ptr())[i as usize].fmark.fnum,
                name,
            );
            if (*namedfm.ptr())[i as usize].fmark.fnum != 0 {
                xfree(name as *mut c_void);
            }
        }
        i += 1;
    }
}
