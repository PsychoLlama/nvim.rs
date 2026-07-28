//! Marks, jumps, changes and tags.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub unsafe extern "C" fn f_changenr(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = (*curbuf.get()).b_u_seq_cur as varnumber_T;
}
pub unsafe extern "C" fn f_getchangelist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, 2 as ptrdiff_t);
    let mut buf: *const buf_T = ::core::ptr::null::<buf_T>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        buf = curbuf.get();
    } else {
        vim_ignored.set(
            tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int,
        );
        (*emsg_off.ptr()) += 1;
        buf = tv_get_buf(argvars.offset(0 as ::core::ffi::c_int as isize), false_0);
        (*emsg_off.ptr()) -= 1;
    }
    if buf.is_null() {
        return;
    }
    let l: *mut list_T = tv_list_alloc((*buf).b_changelistlen as ptrdiff_t);
    tv_list_append_list((*rettv).vval.v_list, l);
    let mut changelistindex: ::core::ffi::c_int = 0;
    if buf == (*curwin.get()).w_buffer as *const buf_T {
        changelistindex = (*curwin.get()).w_changelistidx;
    } else {
        changelistindex = (*buf).b_changelistlen;
        let mut i: size_t = 0 as size_t;
        while i < (*buf).b_wininfo.size {
            let mut wip: *mut WinInfo = *(*buf).b_wininfo.items.offset(i as isize);
            if (*wip).wi_win == curwin.get() {
                changelistindex = (*wip).wi_changelistidx;
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
    }
    tv_list_append_number((*rettv).vval.v_list, changelistindex as varnumber_T);
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < (*buf).b_changelistlen {
        if (*buf).b_changelist[i_0 as usize].mark.lnum != 0 as linenr_T {
            let d: *mut dict_T = tv_dict_alloc();
            tv_list_append_dict(l, d);
            tv_dict_add_nr(
                d,
                b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                (*buf).b_changelist[i_0 as usize].mark.lnum as varnumber_T,
            );
            tv_dict_add_nr(
                d,
                b"col\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
                (*buf).b_changelist[i_0 as usize].mark.col as varnumber_T,
            );
            tv_dict_add_nr(
                d,
                b"coladd\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
                (*buf).b_changelist[i_0 as usize].mark.coladd as varnumber_T,
            );
        }
        i_0 += 1;
    }
}
pub unsafe extern "C" fn f_getjumplist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    let wp: *mut win_T = find_tabwin(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        argvars.offset(1 as ::core::ffi::c_int as isize),
    );
    if wp.is_null() {
        return;
    }
    cleanup_jumplist(wp, true_0 != 0);
    let l: *mut list_T = tv_list_alloc((*wp).w_jumplistlen as ptrdiff_t);
    tv_list_append_list((*rettv).vval.v_list, l);
    tv_list_append_number((*rettv).vval.v_list, (*wp).w_jumplistidx as varnumber_T);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*wp).w_jumplistlen {
        if (*wp).w_jumplist[i as usize].fmark.mark.lnum != 0 as linenr_T {
            let d: *mut dict_T = tv_dict_alloc();
            tv_list_append_dict(l, d);
            tv_dict_add_nr(
                d,
                b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                (*wp).w_jumplist[i as usize].fmark.mark.lnum as varnumber_T,
            );
            tv_dict_add_nr(
                d,
                b"col\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
                (*wp).w_jumplist[i as usize].fmark.mark.col as varnumber_T,
            );
            tv_dict_add_nr(
                d,
                b"coladd\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
                (*wp).w_jumplist[i as usize].fmark.mark.coladd as varnumber_T,
            );
            tv_dict_add_nr(
                d,
                b"bufnr\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                (*wp).w_jumplist[i as usize].fmark.fnum as varnumber_T,
            );
            if !(*wp).w_jumplist[i as usize].fname.is_null() {
                tv_dict_add_str(
                    d,
                    b"filename\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                    (*wp).w_jumplist[i as usize].fname,
                );
            }
        }
        i += 1;
    }
}
pub unsafe extern "C" fn f_getmarklist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        get_global_marks((*rettv).vval.v_list);
        return;
    }
    let mut buf: *mut buf_T = tv_get_buf(argvars.offset(0 as ::core::ffi::c_int as isize), false_0);
    if buf.is_null() {
        return;
    }
    get_buf_local_marks(buf, (*rettv).vval.v_list);
}
pub unsafe extern "C" fn f_gettagstack(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut wp: *mut win_T = curwin.get();
    tv_dict_alloc_ret(rettv);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        wp = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
        if wp.is_null() {
            return;
        }
    }
    get_tagstack(wp, (*rettv).vval.v_dict);
}
pub unsafe extern "C" fn f_settagstack(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    static e_invact2: GlobalCell<*const ::core::ffi::c_char> =
        GlobalCell::new(b"E962: Invalid action: '%s'\0".as_ptr() as *const ::core::ffi::c_char);
    let mut action: ::core::ffi::c_char = 'r' as ::core::ffi::c_char;
    (*rettv).vval.v_number = -1 as varnumber_T;
    let mut wp: *mut win_T = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
    if wp.is_null() {
        return;
    }
    if tv_check_for_dict_arg(argvars, 1 as ::core::ffi::c_int) == FAIL {
        return;
    }
    let mut d: *mut dict_T = (*argvars.offset(1 as ::core::ffi::c_int as isize))
        .vval
        .v_dict;
    if d.is_null() {
        return;
    }
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if tv_check_for_string_arg(argvars, 2 as ::core::ffi::c_int) == FAIL {
            return;
        } else {
            let mut actstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
            actstr = tv_get_string_chk(argvars.offset(2 as ::core::ffi::c_int as isize));
            if actstr.is_null() {
                return;
            }
            if (*actstr as ::core::ffi::c_int == 'r' as ::core::ffi::c_int
                || *actstr as ::core::ffi::c_int == 'a' as ::core::ffi::c_int
                || *actstr as ::core::ffi::c_int == 't' as ::core::ffi::c_int)
                && *actstr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            {
                action = *actstr;
            } else {
                semsg(gettext(e_invact2.get()), actstr);
                return;
            }
        }
    }
    if set_tagstack(wp, d, action as ::core::ffi::c_int) == OK {
        (*rettv).vval.v_number = 0 as varnumber_T;
    }
}
pub unsafe extern "C" fn f_tagfiles(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
    let mut fname: *mut ::core::ffi::c_char =
        xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    let mut first: bool = true_0 != 0;
    let mut tn: tagname_T = tagname_T {
        tn_tags: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tn_np: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tn_did_filefind_init: 0,
        tn_hf_idx: 0,
        tn_search_ctx: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    while get_tagfname(&raw mut tn, first as ::core::ffi::c_int, fname) == OK {
        tv_list_append_string((*rettv).vval.v_list, fname, -1 as ssize_t);
        first = false_0 != 0;
    }
    tagname_free(&raw mut tn);
    xfree(fname as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn f_taglist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let tag_pattern: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    (*rettv).vval.v_number = false_0 as varnumber_T;
    if *tag_pattern as ::core::ffi::c_int == NUL {
        return;
    }
    let mut fname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        fname = tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
    }
    get_tags(
        tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t),
        tag_pattern as *mut ::core::ffi::c_char,
        fname as *mut ::core::ffi::c_char,
    );
}
