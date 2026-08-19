use super::*;
use crate::types::{VAR_STRING, VAR_UNKNOWN};
use crate::window::{WSP_ABOVE, WSP_BELOW, WSP_VERT};

/// "getwinpos({timeout})" function
pub unsafe fn f_getwinpos(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    tv_list_alloc_ret(rettv, 2 as ptrdiff_t);
    tv_list_append_number((*rettv).vval.v_list, -1);
    tv_list_append_number((*rettv).vval.v_list, -1);
}
/// "getwinposx()" function
pub unsafe fn f_getwinposx(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    (*rettv).vval.v_number = -1;
}
/// "getwinposy()" function
pub unsafe fn f_getwinposy(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    (*rettv).vval.v_number = -1;
}
/// "win_move_separator()" function
pub unsafe fn f_win_move_separator(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = 0;
    let mut wp: *mut win_T = find_win_by_nr_or_id(argvars.offset(0));
    if wp.is_null() || (*wp).w_floating {
        return;
    }
    if !win_valid(wp) {
        crate::semsg!("E1308: Cannot resize a window in another tab page");
        return;
    }
    let mut offset: c_int = tv_get_number(argvars.offset(1)) as c_int;
    win_drag_vsep_line(wp, offset);
    (*rettv).vval.v_number = 1;
}
/// "win_move_statusline()" function
pub unsafe fn f_win_move_statusline(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let mut wp: *mut win_T = ptr::null_mut();
    let mut offset: c_int = 0;
    (*rettv).vval.v_number = 0;
    wp = find_win_by_nr_or_id(argvars.offset(0));
    if wp.is_null() || (*wp).w_floating {
        return;
    }
    if !win_valid(wp) {
        crate::semsg!("E1308: Cannot resize a window in another tab page");
        return;
    }
    offset = tv_get_number(argvars.offset(1)) as c_int;
    win_drag_status_line(wp, offset);
    (*rettv).vval.v_number = 1;
}
/// "win_screenpos()" function
pub unsafe fn f_win_screenpos(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    tv_list_alloc_ret(rettv, 2 as ptrdiff_t);
    let wp: *const win_T = find_win_by_nr_or_id(argvars.offset(0));
    tv_list_append_number(
        (*rettv).vval.v_list,
        (if wp.is_null() { 0 } else { (*wp).w_winrow + 1 }) as varnumber_T,
    );
    tv_list_append_number(
        (*rettv).vval.v_list,
        (if wp.is_null() { 0 } else { (*wp).w_wincol + 1 }) as varnumber_T,
    );
}
/// "win_splitmove()" function
pub unsafe fn f_win_splitmove(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut wp: *mut win_T = find_win_by_nr_or_id(argvars.offset(0));
    let mut targetwin: *mut win_T = find_win_by_nr_or_id(argvars.offset(1));
    let mut oldwin: *mut win_T = curwin.get();
    (*rettv).vval.v_number = -1;
    if wp.is_null()
        || targetwin.is_null()
        || wp == targetwin
        || !win_valid(wp)
        || !win_valid(targetwin)
        || (*targetwin).w_floating
    {
        crate::semsg!("E957: Invalid window number");
        return;
    }
    let mut flags: c_int = 0;
    let mut size: c_int = 0;
    if (*argvars.offset(2)).v_type != VAR_UNKNOWN {
        let mut d: *mut dict_T = ptr::null_mut();
        let mut di: *mut dictitem_T = ptr::null_mut();
        if tv_check_for_nonnull_dict_arg(argvars, 2) == FAIL {
            return;
        }
        d = (*argvars.offset(2)).vval.v_dict;
        if tv_dict_get_number(d, c"vertical".as_ptr()) != 0 {
            flags |= WSP_VERT as c_int;
        }
        di = tv_dict_find(d, c"rightbelow".as_ptr(), -1 as ptrdiff_t);
        if !di.is_null() {
            flags |= if tv_get_number(&raw mut (*di).di_tv) != 0 {
                WSP_BELOW as c_int
            } else {
                WSP_ABOVE as c_int
            };
        }
        size = tv_dict_get_number(d, c"size".as_ptr()) as c_int;
    }
    if is_aucmd_win(wp) || text_or_buf_locked() || check_split_disallowed(wp) == FAIL {
        return;
    }
    if curwin.get() != targetwin {
        win_goto(targetwin);
    }
    if curwin.get() == targetwin && win_valid(wp) {
        if win_splitmove(wp, size, flags) == OK {
            (*rettv).vval.v_number = 0;
        }
    } else {
        crate::semsg!("E855: Autocommands caused command to abort");
    }
    if oldwin != curwin.get() && win_valid(oldwin) {
        win_goto(oldwin);
    }
}
/// "wincol()" function
pub unsafe fn f_wincol(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    validate_cursor(curwin.get());
    (*rettv).vval.v_number = ((*curwin.get()).w_wcol + 1) as varnumber_T;
}
/// "winline()" function
pub unsafe fn f_winline(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    validate_cursor(curwin.get());
    (*rettv).vval.v_number = ((*curwin.get()).w_wrow + 1) as varnumber_T;
}
/// "winheight(nr)" function
pub unsafe fn f_winheight(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut wp: *mut win_T = find_win_by_nr_or_id(argvars.offset(0));
    if wp.is_null() {
        (*rettv).vval.v_number = -1;
    } else {
        (*rettv).vval.v_number = (*wp).w_view_height as varnumber_T;
    };
}
/// "winwidth(nr)" function
pub unsafe fn f_winwidth(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut wp: *mut win_T = find_win_by_nr_or_id(argvars.offset(0));
    if wp.is_null() {
        (*rettv).vval.v_number = -1;
    } else {
        (*rettv).vval.v_number = (*wp).w_view_width as varnumber_T;
    };
}
/// "winrestcmd()" function
pub unsafe fn f_winrestcmd(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut buf: [c_char; 50] = [0; 50];
    let mut ga: garray_T = mem::zeroed();
    ga_init(&raw mut ga, size_of::<c_char>() as c_int, 70);
    let mut i: c_int = 0;
    while i < 2 {
        let mut winnr: c_int = 1;
        let mut wp: *mut win_T = tab_firstwin(curtab.get());
        while !wp.is_null() {
            if win_has_winnr(wp, curtab.get()) {
                let mut buflen: size_t = vim_snprintf_safelen(
                    &raw mut buf as *mut c_char,
                    size_of::<[c_char; 50]>(),
                    c"%dresize %d|".as_ptr(),
                    winnr,
                    (*wp).w_height,
                );
                ga_concat_len(&raw mut ga, &raw mut buf as *mut c_char, buflen);
                buflen = vim_snprintf_safelen(
                    &raw mut buf as *mut c_char,
                    size_of::<[c_char; 50]>(),
                    c"vert %dresize %d|".as_ptr(),
                    winnr,
                    (*wp).w_width,
                );
                ga_concat_len(&raw mut ga, &raw mut buf as *mut c_char, buflen);
                winnr += 1;
            }
            wp = (*wp).w_next;
        }
        i += 1;
    }
    ga_append(&raw mut ga, NUL as uint8_t);
    (*rettv).vval.v_string = ga.ga_data as *mut c_char;
    (*rettv).v_type = VAR_STRING;
}
/// "winrestview()" function
pub unsafe fn f_winrestview(argvars: *mut typval_T, _rettv: *mut typval_T, _fptr: EvalFuncData) {
    if tv_check_for_nonnull_dict_arg(argvars, 0) == FAIL {
        return;
    }
    let dict: *mut dict_T = (*argvars.offset(0)).vval.v_dict;
    // Every key is optional: what the dictionary does not mention keeps its
    // current value.
    let entry = |key: &CStr| {
        let di = tv_dict_find(dict, key.as_ptr(), key.count_bytes() as ptrdiff_t);
        if di.is_null() {
            None
        } else {
            Some(tv_get_number(&raw mut (*di).di_tv))
        }
    };
    let win = curwin.get();

    if let Some(v) = entry(c"lnum") {
        (*win).w_cursor.lnum = v as linenr_T;
    }
    if let Some(v) = entry(c"col") {
        (*win).w_cursor.col = v as colnr_T;
    }
    if let Some(v) = entry(c"coladd") {
        (*win).w_cursor.coladd = v as colnr_T;
    }
    if let Some(v) = entry(c"curswant") {
        (*win).w_curswant = v as colnr_T;
        (*win).w_set_curswant = 0;
    }
    if let Some(v) = entry(c"topline") {
        set_topline(win, v as linenr_T);
    }
    if let Some(v) = entry(c"topfill") {
        (*win).w_topfill = v as c_int;
    }
    if let Some(v) = entry(c"leftcol") {
        (*win).w_leftcol = v as colnr_T;
    }
    if let Some(v) = entry(c"skipcol") {
        (*win).w_skipcol = v as colnr_T;
    }

    check_cursor(win);
    win_new_height(win, (*win).w_height);
    win_new_width(win, (*win).w_width);
    changed_window_setting(win);
    // A saved view from a buffer that has since shrunk can name a line that no
    // longer exists.
    if (*win).w_topline <= 0 {
        (*win).w_topline = 1;
    }
    if (*win).w_topline > (*curbuf.get()).b_ml.ml_line_count {
        (*win).w_topline = (*curbuf.get()).b_ml.ml_line_count;
    }
    check_topfill(win, true);
}
/// "winsaveview()" function
pub unsafe fn f_winsaveview(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    tv_dict_alloc_ret(rettv);
    let dict: *mut dict_T = (*rettv).vval.v_dict;
    let nr = |key: &CStr, value: varnumber_T| {
        tv_dict_add_nr(dict, key.as_ptr(), key.count_bytes(), value);
    };
    let win = curwin.get();

    nr(c"lnum", (*win).w_cursor.lnum as varnumber_T);
    nr(c"col", (*win).w_cursor.col as varnumber_T);
    nr(c"coladd", (*win).w_cursor.coladd as varnumber_T);
    update_curswant();
    nr(c"curswant", (*win).w_curswant as varnumber_T);
    nr(c"topline", (*win).w_topline as varnumber_T);
    nr(c"topfill", (*win).w_topfill as varnumber_T);
    nr(c"leftcol", (*win).w_leftcol as varnumber_T);
    nr(c"skipcol", (*win).w_skipcol as varnumber_T);
}
