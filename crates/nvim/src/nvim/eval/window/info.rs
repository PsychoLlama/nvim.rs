use super::*;
use crate::src::nvim::types::{VAR_STRING, VAR_UNKNOWN, kListLenMayKnow, kListLenUnknown};

/// Returns information about a window as a dictionary.
unsafe extern "C" fn get_win_info(wp: *mut win_T, tpnr: int16_t, winnr: int16_t) -> *mut dict_T {
    let dict: *mut dict_T = tv_dict_alloc();
    let nr = |key: &CStr, value: varnumber_T| {
        tv_dict_add_nr(dict, key.as_ptr(), key.count_bytes(), value);
    };

    // "botline" is one past the last displayed line, hence the -1; the row and
    // column counts are zero-based inside and one-based to vimscript.
    validate_botline_win(wp);
    nr(c"tabnr", tpnr as varnumber_T);
    nr(c"winnr", winnr as varnumber_T);
    nr(c"winid", (*wp).handle as varnumber_T);
    nr(c"height", (*wp).w_view_height as varnumber_T);
    nr(c"status_height", (*wp).w_status_height as varnumber_T);
    nr(c"winrow", ((*wp).w_winrow + 1) as varnumber_T);
    nr(c"topline", (*wp).w_topline as varnumber_T);
    nr(c"botline", ((*wp).w_botline - 1) as varnumber_T);
    nr(c"leftcol", (*wp).w_leftcol as varnumber_T);
    nr(c"winbar", (*wp).w_winbar_height as varnumber_T);
    nr(c"width", (*wp).w_view_width as varnumber_T);
    nr(c"bufnr", (*(*wp).w_buffer).handle as varnumber_T);
    nr(c"wincol", ((*wp).w_wincol + 1) as varnumber_T);
    nr(c"textoff", win_col_off(wp) as varnumber_T);
    nr(c"terminal", bt_terminal((*wp).w_buffer) as varnumber_T);
    nr(c"quickfix", bt_quickfix((*wp).w_buffer) as varnumber_T);
    nr(
        c"loclist",
        (bt_quickfix((*wp).w_buffer) && !(*wp).w_llist_ref.is_null()) as varnumber_T,
    );
    tv_dict_add_dict(
        dict,
        c"variables".as_ptr(),
        c"variables".count_bytes(),
        (*wp).w_vars,
    );
    dict
}
/// Returns information (variables, options, etc.) about a tab page
///          as a dictionary.
unsafe extern "C" fn get_tabpage_info(tp: *mut tabpage_T, tp_idx: c_int) -> *mut dict_T {
    let dict: *mut dict_T = tv_dict_alloc();
    tv_dict_add_nr(
        dict,
        c"tabnr".as_ptr(),
        c"tabnr".count_bytes(),
        tp_idx as varnumber_T,
    );

    let l: *mut list_T = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
    let mut wp: *mut win_T = tab_firstwin(tp);
    while !wp.is_null() {
        tv_list_append_number(l, (*wp).handle as varnumber_T);
        wp = (*wp).w_next;
    }
    tv_dict_add_list(dict, c"windows".as_ptr(), c"windows".count_bytes(), l);
    tv_dict_add_dict(
        dict,
        c"variables".as_ptr(),
        c"variables".count_bytes(),
        (*tp).tp_vars,
    );
    dict
}
/// "gettabinfo()" function
pub unsafe extern "C" fn f_gettabinfo(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let mut tparg: *mut tabpage_T = ptr::null_mut();
    tv_list_alloc_ret(
        rettv,
        (if (*argvars.offset(0)).v_type == VAR_UNKNOWN {
            1
        } else {
            kListLenMayKnow as c_int
        }) as ptrdiff_t,
    );
    if (*argvars.offset(0)).v_type != VAR_UNKNOWN {
        tparg = find_tabpage(tv_get_number_chk(argvars.offset(0), ptr::null_mut()) as c_int);
        if tparg.is_null() {
            return;
        }
    }
    let mut tpnr: c_int = 0;
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        tpnr += 1;
        if !(!tparg.is_null() && tp != tparg) {
            let d: *mut dict_T = get_tabpage_info(tp as *mut tabpage_T, tpnr);
            tv_list_append_dict((*rettv).vval.v_list, d);
            if !tparg.is_null() {
                return;
            }
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
/// "getwininfo()" function
pub unsafe extern "C" fn f_getwininfo(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let mut wparg: *mut win_T = ptr::null_mut();
    tv_list_alloc_ret(rettv, kListLenMayKnow as c_int as ptrdiff_t);
    if (*argvars.offset(0)).v_type != VAR_UNKNOWN {
        wparg = win_id2wp(tv_get_number(argvars.offset(0)) as c_int);
        if wparg.is_null() {
            return;
        }
    }
    let mut tabnr: int16_t = 0 as int16_t;
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        tabnr += 1;
        let mut winnr: int16_t = 0 as int16_t;
        let mut wp: *mut win_T = tab_firstwin(tp);
        while !wp.is_null() {
            winnr = (winnr as c_int + win_has_winnr(wp, tp as *mut tabpage_T) as c_int) as int16_t;
            if !(!wparg.is_null() && wp != wparg) {
                let d: *mut dict_T = get_win_info(
                    wp,
                    tabnr,
                    (if win_has_winnr(wp, tp as *mut tabpage_T) {
                        winnr as c_int
                    } else {
                        0
                    }) as int16_t,
                );
                tv_list_append_dict((*rettv).vval.v_list, d);
                if !wparg.is_null() {
                    return;
                }
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
/// Get the layout of the given tab page for winlayout().
unsafe extern "C" fn get_framelayout(fr: *const frame_T, mut l: *mut list_T, mut outer: bool) {
    if fr.is_null() {
        return;
    }
    let mut fr_list: *mut list_T = ptr::null_mut();
    if outer {
        fr_list = l;
    } else {
        fr_list = tv_list_alloc(2 as ptrdiff_t);
        tv_list_append_list(l, fr_list);
    }
    if (*fr).fr_layout as c_int == FR_LEAF {
        if !(*fr).fr_win.is_null() {
            tv_list_append_string(fr_list, c"leaf".as_ptr(), c"leaf".count_bytes() as ssize_t);
            tv_list_append_number(fr_list, (*(*fr).fr_win).handle as varnumber_T);
        }
    } else {
        if (*fr).fr_layout as c_int == FR_ROW {
            tv_list_append_string(fr_list, c"row".as_ptr(), c"row".count_bytes() as ssize_t);
        } else {
            tv_list_append_string(fr_list, c"col".as_ptr(), c"col".count_bytes() as ssize_t);
        }
        let win_list: *mut list_T = tv_list_alloc(kListLenUnknown as c_int as ptrdiff_t);
        tv_list_append_list(fr_list, win_list);
        let mut child: *const frame_T = (*fr).fr_child;
        while !child.is_null() {
            get_framelayout(child, win_list, false);
            child = (*child).fr_next;
        }
    };
}
/// "winlayout()" function
pub unsafe extern "C" fn f_winlayout(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let mut tp: *mut tabpage_T = ptr::null_mut();
    tv_list_alloc_ret(rettv, 2 as ptrdiff_t);
    if (*argvars.offset(0)).v_type == VAR_UNKNOWN {
        tp = curtab.get();
    } else {
        tp = find_tabpage(tv_get_number(argvars.offset(0)) as c_int);
        if tp.is_null() {
            return;
        }
    }
    get_framelayout((*tp).tp_topframe, (*rettv).vval.v_list, true);
}
/// "win_gettype(nr)" function
pub unsafe extern "C" fn f_win_gettype(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let mut wp: *mut win_T = curwin.get();
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ptr::null_mut();
    if (*argvars.offset(0)).v_type != VAR_UNKNOWN {
        wp = find_win_by_nr_or_id(argvars.offset(0));
        if wp.is_null() {
            (*rettv).vval.v_string = xstrdup(c"unknown".as_ptr());
            return;
        }
    }
    if is_aucmd_win(wp) {
        (*rettv).vval.v_string = xstrdup(c"autocmd".as_ptr());
    } else if (*wp).w_onebuf_opt.wo_pvw != 0 {
        (*rettv).vval.v_string = xstrdup(c"preview".as_ptr());
    } else if (*wp).w_floating {
        (*rettv).vval.v_string = xstrdup(c"popup".as_ptr());
    } else if wp == cmdwin_win.get() {
        (*rettv).vval.v_string = xstrdup(c"command".as_ptr());
    } else if bt_quickfix((*wp).w_buffer) {
        (*rettv).vval.v_string = xstrdup(if !(*wp).w_llist_ref.is_null() {
            c"loclist".as_ptr()
        } else {
            c"quickfix".as_ptr()
        });
    }
}
/// "getcmdwintype()" function
pub unsafe extern "C" fn f_getcmdwintype(
    _argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ptr::null_mut();
    (*rettv).vval.v_string = xmallocz(1) as *mut c_char;
    *(*rettv).vval.v_string.offset(0) = cmdwin_type.get() as c_char;
}
