use super::*;
use crate::guard::Suppress;
use crate::types::{VAR_NUMBER, VAR_STRING, VAR_UNKNOWN};

/// Find a buffer by number or exact name.
pub unsafe fn find_buffer(avar: *mut typval_T) -> *mut buf_T {
    let mut buf: *mut buf_T = ptr::null_mut();
    if (*avar).v_type == VAR_NUMBER {
        buf = buflist_findnr((*avar).vval.v_number as c_int);
    } else if (*avar).v_type == VAR_STRING && !(*avar).vval.v_string.is_null() {
        buf = buflist_findname_exp((*avar).vval.v_string);
        if buf.is_null() {
            let mut bp: *mut buf_T = firstbuf.get();
            while !bp.is_null() {
                if !(*bp).b_fname.is_null()
                    && (path_with_url((*bp).b_fname) != 0 || bt_nofilename(bp))
                    && strcmp((*bp).b_fname, (*avar).vval.v_string) == 0
                {
                    buf = bp;
                    break;
                } else {
                    bp = (*bp).b_next;
                }
            }
        }
    }
    buf
}
/// "bufadd(expr)" function
pub unsafe fn f_bufadd(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let name: *mut c_char = tv_get_string(argvars.offset(0)) as *mut c_char;
    (*rettv).vval.v_number = buflist_add(
        if *name as c_int == NUL {
            ptr::null_mut()
        } else {
            name
        },
        0,
    ) as varnumber_T;
}
/// "bufexists(expr)" function
pub unsafe fn f_bufexists(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    (*rettv).vval.v_number = !find_buffer(argvars.offset(0)).is_null() as varnumber_T;
}
/// "buflisted(expr)" function
pub unsafe fn f_buflisted(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let buf: *mut buf_T = find_buffer(argvars.offset(0));
    (*rettv).vval.v_number = (!buf.is_null() && (*buf).b_p_bl != 0) as varnumber_T;
}
/// "bufload(expr)" function
pub unsafe fn f_bufload(argvars: *mut typval_T, _unused: *mut typval_T, _fptr: EvalFuncData) {
    let buf: *mut buf_T = get_buf_arg(argvars.offset(0));
    if !buf.is_null() {
        if swap_exists_action.get() != SEA_READONLY {
            swap_exists_action.set(SEA_NONE);
        }
        buf_ensure_loaded(buf);
    }
}
/// "bufloaded(expr)" function
pub unsafe fn f_bufloaded(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let buf: *mut buf_T = find_buffer(argvars.offset(0));
    (*rettv).vval.v_number = (!buf.is_null() && !(*buf).b_ml.ml_mfp.is_null()) as varnumber_T;
}
/// "bufname(expr)" function
pub unsafe fn f_bufname(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ptr::null_mut();
    let buf: *const buf_T = if (*argvars.offset(0)).v_type == VAR_UNKNOWN {
        curbuf.get()
    } else {
        tv_get_buf_from_arg(argvars.offset(0))
    };
    if !buf.is_null() && !(*buf).b_fname.is_null() {
        (*rettv).vval.v_string = xstrdup((*buf).b_fname);
    }
}
/// "bufnr(expr)" function
pub unsafe fn f_bufnr(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut error: bool = false;
    (*rettv).vval.v_number = -1;
    let mut buf: *const buf_T = if (*argvars.offset(0)).v_type == VAR_UNKNOWN {
        curbuf.get()
    } else {
        if !tv_check_str_or_nr(argvars.offset(0)) {
            return;
        }
        // The lookup itself must not report "no such buffer": a second
        // argument asks for the buffer to be created instead.
        let _no_emsg = Suppress::emsg();
        tv_get_buf(argvars.offset(0), 0)
    };
    let mut name: *const c_char = ptr::null();
    if buf.is_null()
        && (*argvars.offset(1)).v_type != VAR_UNKNOWN
        && tv_get_number_chk(argvars.offset(1), &raw mut error) != 0
        && !error
        && {
            name = tv_get_string_chk(argvars.offset(0));
            !name.is_null()
        }
    {
        buf = buflist_new(name as *mut c_char, ptr::null_mut(), 1, 0);
    }
    if !buf.is_null() {
        (*rettv).vval.v_number = (*buf).handle as varnumber_T;
    }
}
unsafe fn buf_win_common(argvars: *mut typval_T, rettv: *mut typval_T, get_nr: bool) {
    let buf: *const buf_T = tv_get_buf_from_arg(argvars.offset(0));
    if buf.is_null() {
        (*rettv).vval.v_number = -1;
        return;
    }
    let mut winnr: c_int = 0;
    let mut winid: c_int = 0;
    let mut found_buf: bool = false;
    // FOR_ALL_WINDOWS_IN_TAB(curtab), whose tab-page test is a tautology.
    let mut wp: *mut win_T = firstwin.get();
    while !wp.is_null() {
        winnr += win_has_winnr(wp, curtab.get()) as c_int;
        if core::ptr::eq((*wp).w_buffer, buf) && (!get_nr || win_has_winnr(wp, curtab.get())) {
            found_buf = true;
            winid = (*wp).handle as c_int;
            break;
        } else {
            wp = (*wp).w_next;
        }
    }
    (*rettv).vval.v_number = (if found_buf {
        if get_nr { winnr } else { winid }
    } else {
        -1
    }) as varnumber_T;
}
/// "bufwinid(nr)" function
pub unsafe fn f_bufwinid(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    buf_win_common(argvars, rettv, false);
}
/// "bufwinnr(nr)" function
pub unsafe fn f_bufwinnr(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    buf_win_common(argvars, rettv, true);
}
