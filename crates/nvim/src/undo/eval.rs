//! `:undolist` and the `undofile()`/`undotree()` builtins.

use super::file::*;
use super::*;
use crate::highlight_group::HLF_T;
use crate::types::{VAR_STRING, VAR_UNKNOWN, kListLenMayKnow};

pub unsafe fn ex_undolist(mut _eap: *mut exarg_T) {
    let mut changes: c_int = 1;
    (*lastmark.ptr()) += 1;
    let mut mark: c_int = lastmark.get();
    (*lastmark.ptr()) += 1;
    let mut nomark: c_int = lastmark.get();
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ptr::null_mut(),
    };
    ga_init(&raw mut ga, size_of::<*mut c_char>() as c_int, 20);
    let mut uhp: *mut u_header_T = (*curbuf.get()).b_u_oldhead;
    while !uhp.is_null() {
        if (*uhp).uh_prev.ptr.is_null() && (*uhp).uh_walk != nomark && (*uhp).uh_walk != mark {
            vim_snprintf(
                IObuff.ptr() as *mut c_char,
                IOSIZE as size_t,
                c"%6d %7d  ".as_ptr(),
                (*uhp).uh_seq,
                changes,
            );
            undo_fmt_time(
                (IObuff.ptr() as *mut c_char).add(strlen(IObuff.ptr() as *mut c_char)),
                (IOSIZE as size_t).wrapping_sub(strlen(IObuff.ptr() as *mut c_char)),
                (*uhp).uh_time,
            );
            if (*uhp).uh_save_nr > 0 {
                while strlen(IObuff.ptr() as *mut c_char) < 33 {
                    xstrlcat(IObuff.ptr() as *mut c_char, c" ".as_ptr(), IOSIZE as size_t);
                }
                vim_snprintf_add(
                    IObuff.ptr() as *mut c_char,
                    IOSIZE as size_t,
                    c"  %3d".as_ptr(),
                    (*uhp).uh_save_nr,
                );
            }
            ga_grow(&raw mut ga, 1);
            *(ga.ga_data as *mut *mut c_char).offset(ga.ga_len as isize) =
                xstrdup(IObuff.ptr() as *mut c_char);
            ga.ga_len += 1;
        }
        (*uhp).uh_walk = mark;
        if !(*uhp).uh_prev.ptr.is_null()
            && (*(*uhp).uh_prev.ptr).uh_walk != nomark
            && (*(*uhp).uh_prev.ptr).uh_walk != mark
        {
            uhp = (*uhp).uh_prev.ptr;
            changes += 1;
        } else if !(*uhp).uh_alt_next.ptr.is_null()
            && (*(*uhp).uh_alt_next.ptr).uh_walk != nomark
            && (*(*uhp).uh_alt_next.ptr).uh_walk != mark
        {
            uhp = (*uhp).uh_alt_next.ptr;
        } else if !(*uhp).uh_next.ptr.is_null()
            && (*uhp).uh_alt_prev.ptr.is_null()
            && (*(*uhp).uh_next.ptr).uh_walk != nomark
            && (*(*uhp).uh_next.ptr).uh_walk != mark
        {
            uhp = (*uhp).uh_next.ptr;
            changes -= 1;
        } else {
            (*uhp).uh_walk = nomark;
            if !(*uhp).uh_alt_prev.ptr.is_null() {
                uhp = (*uhp).uh_alt_prev.ptr;
            } else {
                uhp = (*uhp).uh_next.ptr;
                changes -= 1;
            }
        }
    }
    msg_ext_set_kind(c"list_cmd".as_ptr());
    if ga.ga_len <= 0 {
        msg(gettext(c"Nothing to undo".as_ptr()), 0);
    } else {
        sort_strings(ga.ga_data as *mut *mut c_char, ga.ga_len);
        msg_start();
        msg_puts_hl(
            gettext(c"number changes  when               saved".as_ptr()),
            HLF_T,
            false,
        );
        let mut i: c_int = 0;
        while i < ga.ga_len && !got_int.get() {
            msg_putchar('\n' as c_int);
            if got_int.get() {
                break;
            }
            msg_puts(*(ga.ga_data as *mut *const c_char).offset(i as isize));
            i += 1;
        }
        msg_end();
        ga_clear_strings(&raw mut ga);
    };
}
pub(crate) unsafe extern "C" fn u_eval_tree(
    buf: *mut buf_T,
    first_uhp: *const u_header_T,
) -> *mut list_T {
    let list: *mut list_T = tv_list_alloc(kListLenMayKnow as c_int as ptrdiff_t);
    let mut uhp: *const u_header_T = first_uhp;
    while !uhp.is_null() {
        let dict: *mut dict_T = tv_dict_alloc();
        tv_dict_add_nr(
            dict,
            c"seq".as_ptr(),
            size_of::<[c_char; 4]>().wrapping_sub(1),
            (*uhp).uh_seq as varnumber_T,
        );
        tv_dict_add_nr(
            dict,
            c"time".as_ptr(),
            size_of::<[c_char; 5]>().wrapping_sub(1),
            (*uhp).uh_time as varnumber_T,
        );
        if uhp == (*buf).b_u_newhead as *const u_header_T {
            tv_dict_add_nr(
                dict,
                c"newhead".as_ptr(),
                size_of::<[c_char; 8]>().wrapping_sub(1),
                1 as varnumber_T,
            );
        }
        if uhp == (*buf).b_u_curhead as *const u_header_T {
            tv_dict_add_nr(
                dict,
                c"curhead".as_ptr(),
                size_of::<[c_char; 8]>().wrapping_sub(1),
                1 as varnumber_T,
            );
        }
        if (*uhp).uh_save_nr > 0 {
            tv_dict_add_nr(
                dict,
                c"save".as_ptr(),
                size_of::<[c_char; 5]>().wrapping_sub(1),
                (*uhp).uh_save_nr as varnumber_T,
            );
        }
        if !(*uhp).uh_alt_next.ptr.is_null() {
            tv_dict_add_list(
                dict,
                c"alt".as_ptr(),
                size_of::<[c_char; 4]>().wrapping_sub(1),
                u_eval_tree(buf, (*uhp).uh_alt_next.ptr),
            );
        }
        tv_list_append_dict(list, dict);
        uhp = (*uhp).uh_prev.ptr;
    }
    return list;
}
pub unsafe extern "C" fn f_undofile(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    let fname: *const c_char = tv_get_string(argvars.offset(0));
    if *fname as c_int == NUL {
        (*rettv).vval.v_string = ptr::null_mut();
    } else {
        let mut ffname: *mut c_char = FullName_save(fname, true);
        if !ffname.is_null() {
            (*rettv).vval.v_string = u_get_undo_file_name(ffname, false);
        }
        xfree(ffname as *mut c_void);
    };
}
pub unsafe extern "C" fn f_undotree(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_dict_alloc_ret(rettv);
    let tv: *mut typval_T = argvars.offset(0);
    let buf: *mut buf_T = if (*tv).v_type as c_uint == VAR_UNKNOWN as c_int as c_uint {
        curbuf.get()
    } else {
        get_buf_arg(tv)
    };
    if buf.is_null() {
        return;
    }
    let mut dict: *mut dict_T = (*rettv).vval.v_dict;
    tv_dict_add_nr(
        dict,
        c"synced".as_ptr(),
        size_of::<[c_char; 7]>().wrapping_sub(1),
        (*buf).b_u_synced as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        c"seq_last".as_ptr(),
        size_of::<[c_char; 9]>().wrapping_sub(1),
        (*buf).b_u_seq_last as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        c"save_last".as_ptr(),
        size_of::<[c_char; 10]>().wrapping_sub(1),
        (*buf).b_u_save_nr_last as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        c"seq_cur".as_ptr(),
        size_of::<[c_char; 8]>().wrapping_sub(1),
        (*buf).b_u_seq_cur as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        c"time_cur".as_ptr(),
        size_of::<[c_char; 9]>().wrapping_sub(1),
        (*buf).b_u_time_cur as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        c"save_cur".as_ptr(),
        size_of::<[c_char; 9]>().wrapping_sub(1),
        (*buf).b_u_save_nr_cur as varnumber_T,
    );
    tv_dict_add_list(
        dict,
        c"entries".as_ptr(),
        size_of::<[c_char; 8]>().wrapping_sub(1),
        u_eval_tree(buf, (*buf).b_u_oldhead),
    );
}
pub unsafe extern "C" fn u_force_get_undo_header(mut buf: *mut buf_T) -> *mut u_header_T {
    let mut uhp: *mut u_header_T = ptr::null_mut();
    if !(*buf).b_u_curhead.is_null() {
        uhp = (*buf).b_u_curhead;
    } else if !(*buf).b_u_newhead.is_null() {
        uhp = (*buf).b_u_newhead;
    }
    if uhp.is_null() {
        u_savecommon(buf, 0, 1, 1, true);
        uhp = (*buf).b_u_curhead;
        if uhp.is_null() {
            uhp = (*buf).b_u_newhead;
            if get_undolevel(buf) > 0 && uhp.is_null() {
                abort();
            }
        }
    }
    return uhp;
}
