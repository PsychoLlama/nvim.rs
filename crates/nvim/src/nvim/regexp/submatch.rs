//! What the last match captured: the `submatch()` accessors and the
//! list the `\\=` expression evaluator sees.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub(crate) unsafe extern "C" fn fill_submatch_list(
    mut _argc: ::core::ffi::c_int,
    mut argv: *mut typval_T,
    mut argskip: ::core::ffi::c_int,
    mut fp: *mut ufunc_T,
) -> ::core::ffi::c_int {
    let mut listarg: *mut typval_T = argv.offset(argskip as isize);
    if (*fp).uf_varargs == 0 && (*fp).uf_args.ga_len <= argskip {
        return argskip;
    }
    tv_list_init_static10((*listarg).vval.v_list as *mut staticList10_T);
    let mut li: *mut listitem_T = tv_list_first((*listarg).vval.v_list);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 10 as ::core::ffi::c_int {
        let mut s: *mut ::core::ffi::c_char = (*(*rsm.ptr()).sm_match).startp[i as usize];
        if s.is_null() || (*(*rsm.ptr()).sm_match).endp[i as usize].is_null() {
            s = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            s = xstrnsave(
                s,
                (*(*rsm.ptr()).sm_match).endp[i as usize].offset_from(s) as size_t,
            );
        }
        (*li).li_tv.v_type = VAR_STRING;
        (*li).li_tv.vval.v_string = s;
        li = (*li).li_next;
        i += 1;
    }
    return argskip + 1 as ::core::ffi::c_int;
}
pub(crate) unsafe extern "C" fn clear_submatch_list(mut sl: *mut staticList10_T) {
    let l_: *mut list_T = &raw mut (*sl).sl_list;
    if !l_.is_null() {
        let mut li: *mut listitem_T = (*l_).lv_first;
        while !li.is_null() {
            xfree((*li).li_tv.vval.v_string as *mut ::core::ffi::c_void);
            li = (*li).li_next;
        }
    }
}
pub(crate) unsafe extern "C" fn reg_getline_submatch(
    mut lnum: linenr_T,
) -> *mut ::core::ffi::c_char {
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    reg_getline_common(
        lnum,
        (RGLF_LINE as ::core::ffi::c_int | RGLF_SUBMATCH as ::core::ffi::c_int)
            as reg_getline_flags_T,
        &raw mut line,
        ::core::ptr::null_mut::<colnr_T>(),
    );
    return line;
}
pub(crate) unsafe extern "C" fn reg_getline_submatch_len(mut lnum: linenr_T) -> colnr_T {
    let mut length: colnr_T = 0;
    reg_getline_common(
        lnum,
        (RGLF_LENGTH as ::core::ffi::c_int | RGLF_SUBMATCH as ::core::ffi::c_int)
            as reg_getline_flags_T,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        &raw mut length,
    );
    return length;
}
pub unsafe extern "C" fn reg_submatch(mut no: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut round: ::core::ffi::c_int = 0;
    let mut lnum: linenr_T = 0;
    if !can_f_submatch.get() || no < 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if (*rsm.ptr()).sm_match.is_null() {
        let mut len: ssize_t = 0;
        round = 1 as ::core::ffi::c_int;
        while round <= 2 as ::core::ffi::c_int {
            lnum = (*(*rsm.ptr()).sm_mmatch).startpos[no as usize].lnum;
            if lnum < 0 as linenr_T
                || (*(*rsm.ptr()).sm_mmatch).endpos[no as usize].lnum < 0 as linenr_T
            {
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            s = reg_getline_submatch(lnum);
            if s.is_null() {
                break;
            }
            s = s.offset((*(*rsm.ptr()).sm_mmatch).startpos[no as usize].col as isize);
            if (*(*rsm.ptr()).sm_mmatch).endpos[no as usize].lnum == lnum {
                len = ((*(*rsm.ptr()).sm_mmatch).endpos[no as usize].col
                    - (*(*rsm.ptr()).sm_mmatch).startpos[no as usize].col)
                    as ssize_t;
                if round == 2 as ::core::ffi::c_int {
                    xmemcpyz(
                        retval as *mut ::core::ffi::c_void,
                        s as *const ::core::ffi::c_void,
                        len as size_t,
                    );
                }
                len += 1;
            } else {
                len = (reg_getline_submatch_len(lnum)
                    - (*(*rsm.ptr()).sm_mmatch).startpos[no as usize].col)
                    as ssize_t;
                if round == 2 as ::core::ffi::c_int {
                    strcpy(retval, s);
                    *retval.offset(len as isize) = '\n' as ::core::ffi::c_char;
                }
                len += 1;
                lnum += 1;
                while lnum < (*(*rsm.ptr()).sm_mmatch).endpos[no as usize].lnum {
                    s = reg_getline_submatch(lnum);
                    if round == 2 as ::core::ffi::c_int {
                        strcpy(retval.offset(len as isize), s);
                    }
                    len += reg_getline_submatch_len(lnum) as ssize_t;
                    if round == 2 as ::core::ffi::c_int {
                        *retval.offset(len as isize) = '\n' as ::core::ffi::c_char;
                    }
                    len += 1;
                    lnum += 1;
                }
                if round == 2 as ::core::ffi::c_int {
                    strncpy(
                        retval.offset(len as isize),
                        reg_getline_submatch(lnum),
                        (*(*rsm.ptr()).sm_mmatch).endpos[no as usize].col as size_t,
                    );
                }
                len += (*(*rsm.ptr()).sm_mmatch).endpos[no as usize].col as ssize_t;
                if round == 2 as ::core::ffi::c_int {
                    *retval.offset(len as isize) = NUL as ::core::ffi::c_char;
                }
                len += 1;
            }
            if retval.is_null() {
                retval = xmalloc(len as size_t) as *mut ::core::ffi::c_char;
            }
            round += 1;
        }
    } else {
        s = (*(*rsm.ptr()).sm_match).startp[no as usize];
        if s.is_null() || (*(*rsm.ptr()).sm_match).endp[no as usize].is_null() {
            retval = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            retval = xstrnsave(
                s,
                (*(*rsm.ptr()).sm_match).endp[no as usize].offset_from(s) as size_t,
            );
        }
    }
    return retval;
}
pub unsafe extern "C" fn reg_submatch_list(mut no: ::core::ffi::c_int) -> *mut list_T {
    if !can_f_submatch.get() || no < 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<list_T>();
    }
    let mut slnum: linenr_T = 0;
    let mut elnum: linenr_T = 0;
    let mut list: *mut list_T = ::core::ptr::null_mut::<list_T>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*rsm.ptr()).sm_match.is_null() {
        slnum = (*(*rsm.ptr()).sm_mmatch).startpos[no as usize].lnum;
        elnum = (*(*rsm.ptr()).sm_mmatch).endpos[no as usize].lnum;
        if slnum < 0 as linenr_T || elnum < 0 as linenr_T {
            return ::core::ptr::null_mut::<list_T>();
        }
        let mut scol: colnr_T = (*(*rsm.ptr()).sm_mmatch).startpos[no as usize].col;
        let mut ecol: colnr_T = (*(*rsm.ptr()).sm_mmatch).endpos[no as usize].col;
        list = tv_list_alloc((elnum - slnum + 1 as linenr_T) as ptrdiff_t);
        s = reg_getline_submatch(slnum).offset(scol as isize);
        if slnum == elnum {
            tv_list_append_string(list, s, (ecol - scol) as ssize_t);
        } else {
            let mut max_lnum: ::core::ffi::c_int =
                elnum as ::core::ffi::c_int - slnum as ::core::ffi::c_int;
            tv_list_append_string(list, s, -1 as ssize_t);
            let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            while i < max_lnum {
                s = reg_getline_submatch(slnum + i as linenr_T);
                tv_list_append_string(list, s, -1 as ssize_t);
                i += 1;
            }
            s = reg_getline_submatch(elnum);
            tv_list_append_string(list, s, ecol as ssize_t);
        }
    } else {
        s = (*(*rsm.ptr()).sm_match).startp[no as usize];
        if s.is_null() || (*(*rsm.ptr()).sm_match).endp[no as usize].is_null() {
            return ::core::ptr::null_mut::<list_T>();
        }
        list = tv_list_alloc(1 as ptrdiff_t);
        tv_list_append_string(
            list,
            s,
            (*(*rsm.ptr()).sm_match).endp[no as usize].offset_from(s) as ssize_t,
        );
    }
    tv_list_ref(list);
    return list;
}
