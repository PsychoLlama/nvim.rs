use super::*;
use crate::buffer::buf_get_changedtick;
use crate::types::{VAR_DICT, VAR_UNKNOWN, kListLenMayKnow};

/// Returns buffer options, variables and other attributes in a dictionary.
unsafe fn get_buffer_info(buf: *mut buf_T) -> *mut dict_T {
    let dict: *mut dict_T = tv_dict_alloc();
    let nr = |key: &CStr, value: varnumber_T| {
        tv_dict_add_nr(dict, key.as_ptr(), key.count_bytes(), value);
    };

    nr(c"bufnr", (*buf).handle as varnumber_T);
    tv_dict_add_str(
        dict,
        c"name".as_ptr(),
        c"name".count_bytes(),
        if !(*buf).b_ffname.is_null() {
            (*buf).b_ffname as *const c_char
        } else {
            c"".as_ptr()
        },
    );
    nr(
        c"lnum",
        (if buf == curbuf.get() {
            (*curwin.get()).w_cursor.lnum
        } else {
            buflist_findlnum(buf)
        }) as varnumber_T,
    );
    nr(c"linecount", (*buf).b_ml.ml_line_count as varnumber_T);
    nr(c"loaded", !(*buf).b_ml.ml_mfp.is_null() as varnumber_T);
    nr(c"listed", (*buf).b_p_bl as varnumber_T);
    nr(c"changed", bufIsChanged(buf) as varnumber_T);
    nr(c"changedtick", buf_get_changedtick(&*buf));
    nr(
        c"hidden",
        (!(*buf).b_ml.ml_mfp.is_null() && (*buf).b_nwindows == 0) as varnumber_T,
    );
    nr(c"command", (buf == cmdwin_buf.get()) as varnumber_T);
    tv_dict_add_dict(
        dict,
        c"variables".as_ptr(),
        c"variables".count_bytes(),
        (*buf).b_vars,
    );

    // List of windows displaying this buffer.
    let windows: *mut list_T = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
    for wp in tab_windows().map(Win::raw) {
        if (*wp).w_buffer == buf {
            tv_list_append_number(windows, (*wp).handle as varnumber_T);
        }
    }
    tv_dict_add_list(dict, c"windows".as_ptr(), c"windows".count_bytes(), windows);

    if buf_has_signs(buf) {
        tv_dict_add_list(
            dict,
            c"signs".as_ptr(),
            c"signs".count_bytes(),
            get_buffer_signs(buf),
        );
    }
    nr(c"lastused", (*buf).b_last_used as varnumber_T);
    dict
}
/// "getbufinfo()" function
pub unsafe fn f_getbufinfo(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut argbuf: *mut buf_T = ptr::null_mut();
    let mut filtered: bool = false;
    let mut sel_buflisted: bool = false;
    let mut sel_bufloaded: bool = false;
    let mut sel_bufmodified: bool = false;
    tv_list_alloc_ret(rettv, kListLenMayKnow as c_int as ptrdiff_t);
    if (*argvars.offset(0)).v_type == VAR_DICT {
        let mut sel_d: *mut dict_T = (*argvars.offset(0)).vval.v_dict;
        if !sel_d.is_null() {
            filtered = true;
            let flag = |key: &CStr| {
                let di = tv_dict_find(sel_d, key.as_ptr(), key.count_bytes() as ptrdiff_t);
                !di.is_null() && tv_get_number(&raw mut (*di).di_tv) != 0
            };
            sel_buflisted = flag(c"buflisted");
            sel_bufloaded = flag(c"bufloaded");
            sel_bufmodified = flag(c"bufmodified");
        }
    } else if (*argvars.offset(0)).v_type != VAR_UNKNOWN {
        argbuf = tv_get_buf_from_arg(argvars.offset(0));
        if argbuf.is_null() {
            return;
        }
    }
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        if !(!argbuf.is_null() && argbuf != buf)
            && !(filtered
                && (sel_bufloaded && (*buf).b_ml.ml_mfp.is_null()
                    || sel_buflisted && (*buf).b_p_bl == 0
                    || sel_bufmodified && (*buf).b_changed == 0))
        {
            let d: *mut dict_T = get_buffer_info(buf);
            tv_list_append_dict((*rettv).vval.v_list, d);
            if !argbuf.is_null() {
                return;
            }
        }
        buf = (*buf).b_next;
    }
}
