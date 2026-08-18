use super::*;
use crate::types::{VAR_UNKNOWN, kListLenMayKnow};

pub unsafe fn win_id2wp(id: c_int) -> *mut win_T {
    win_id2wp_tp(id, ptr::null_mut())
}
/// Return the window and tab pointer of window "id".
/// Returns NULL when not found.
pub unsafe fn win_id2wp_tp(id: c_int, mut tpp: *mut *mut tabpage_T) -> *mut win_T {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = tab_firstwin(tp);
        while !wp.is_null() {
            if (*wp).handle == id {
                if !tpp.is_null() {
                    *tpp = tp as *mut tabpage_T;
                }
                return wp;
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    ptr::null_mut()
}
/// Find window specified by "vp" in tabpage "tp".
///
/// `tp` — NULL for current tab page
/// Returns current window if "vp" is number zero.
///          NULL if not found.
pub unsafe fn find_win_by_nr(vp: *mut typval_T, tp: *mut tabpage_T) -> *mut win_T {
    let mut nr: c_int = tv_get_number_chk(vp, ptr::null_mut()) as c_int;
    if nr < 0 {
        return ptr::null_mut();
    }
    if nr == 0 {
        return curwin.get();
    }
    // A NULL tab page means the current one.
    let tp = if tp.is_null() { curtab.get() } else { tp };
    let mut wp: *mut win_T = tab_firstwin(tp);
    while !wp.is_null() {
        if nr >= LOWEST_WIN_ID as c_int {
            if (*wp).handle == nr {
                return wp;
            }
        } else {
            nr -= 1;
            if nr <= 0 {
                return wp;
            }
        }
        wp = (*wp).w_next;
    }
    ptr::null_mut()
}
/// Find a window: When using a Window ID in any tab page, when using a number
/// in the current tab page.
/// Returns NULL when not found.
pub unsafe fn find_win_by_nr_or_id(vp: *mut typval_T) -> *mut win_T {
    let mut nr: c_int = tv_get_number_chk(vp, ptr::null_mut()) as c_int;
    if nr >= LOWEST_WIN_ID as c_int {
        return win_id2wp(tv_get_number(vp) as c_int);
    }
    find_win_by_nr(vp, ptr::null_mut())
}
/// Find window specified by "wvp" in tabpage "tvp".
pub unsafe fn find_tabwin(wvp: *mut typval_T, mut tvp: *mut typval_T) -> *mut win_T {
    let mut wp: *mut win_T = ptr::null_mut();
    let mut tp: *mut tabpage_T = ptr::null_mut();
    if (*wvp).v_type != VAR_UNKNOWN {
        if (*tvp).v_type != VAR_UNKNOWN {
            let mut n: c_int = tv_get_number(tvp) as c_int;
            if n >= 0 {
                tp = find_tabpage(n);
            }
        } else {
            tp = curtab.get();
        }
        if !tp.is_null() {
            wp = find_win_by_nr(wvp, tp);
        }
    } else {
        wp = curwin.get();
    }
    wp
}
/// Common code for tabpagewinnr() and winnr().
unsafe fn get_winnr(tp: *mut tabpage_T, mut argvar: *mut typval_T) -> c_int {
    let mut nr: c_int = 1;
    let mut twin: *mut win_T = if tp == curtab.get() {
        curwin.get()
    } else {
        (*tp).tp_curwin
    };
    if (*argvar).v_type != VAR_UNKNOWN {
        let mut invalid_arg: bool = false;
        let arg: *const c_char = tv_get_string_chk(argvar);
        if arg.is_null() {
            nr = 0;
        } else if strcmp(arg, c"$".as_ptr()) == 0 {
            twin = if tp == curtab.get() {
                lastwin.get()
            } else {
                (*tp).tp_lastwin
            };
        } else if strcmp(arg, c"#".as_ptr()) == 0 {
            twin = if tp == curtab.get() {
                prevwin.get()
            } else {
                (*tp).tp_prevwin
            };
            if twin.is_null() {
                nr = 0;
            }
        } else {
            let mut endp: *mut c_char = ptr::null_mut();
            let mut count: c_int = strtol(arg, &raw mut endp, 10) as c_int;
            if count <= 0 {
                count = 1;
            }
            if !endp.is_null() && *endp as c_int != NUL {
                if strequal(endp, c"j".as_ptr()) {
                    twin = win_vert_neighbor(tp, twin, false, count);
                } else if strequal(endp, c"k".as_ptr()) {
                    twin = win_vert_neighbor(tp, twin, true, count);
                } else if strequal(endp, c"h".as_ptr()) {
                    twin = win_horz_neighbor(tp, twin, true, count);
                } else if strequal(endp, c"l".as_ptr()) {
                    twin = win_horz_neighbor(tp, twin, false, count);
                } else {
                    invalid_arg = true;
                }
            } else {
                invalid_arg = true;
            }
        }
        if invalid_arg {
            crate::semsg!(
                "E15: Invalid expression: \"{}\"",
                CStr::from_ptr(arg).to_string_lossy()
            );
            nr = 0;
        }
    } else if !win_has_winnr(twin, tp) {
        nr = 0;
    }
    if nr <= 0 {
        return 0;
    }
    nr = 0;
    let mut wp: *mut win_T = tab_firstwin(tp);
    while !wp.is_null() {
        nr += win_has_winnr(wp, tp) as c_int;
        if wp == twin {
            break;
        }
        wp = (*wp).w_next;
    }
    if wp.is_null() {
        nr = 0;
    }
    nr
}
/// "win_getid()" function
pub unsafe fn f_win_getid(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    if (*argvars.offset(0)).v_type == VAR_UNKNOWN {
        (*rettv).vval.v_number = (*curwin.get()).handle as varnumber_T;
        return;
    }
    let mut winnr: c_int = tv_get_number(argvars.offset(0)) as c_int;
    if winnr <= 0 {
        (*rettv).vval.v_number = 0;
        return;
    }
    // A second argument names the tab page; without one, the current one.
    // This is not `find_tabpage()`, which answers the *current* tab page for
    // 0 where `win_getid()` has always rejected it.
    let tp: *mut tabpage_T = if (*argvars.offset(1)).v_type == VAR_UNKNOWN {
        curtab.get()
    } else {
        let mut tabnr: c_int = tv_get_number(argvars.offset(1)) as c_int;
        let mut found: *mut tabpage_T = ptr::null_mut();
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            tabnr -= 1;
            if tabnr == 0 {
                found = tp;
                break;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        if found.is_null() {
            // Unlike every other failure here, a bad tab page answers -1.
            (*rettv).vval.v_number = -1;
            return;
        }
        found
    };
    let mut wp: *mut win_T = tab_firstwin(tp);
    while !wp.is_null() {
        winnr -= win_has_winnr(wp, tp) as c_int;
        if winnr == 0 {
            (*rettv).vval.v_number = (*wp).handle as varnumber_T;
            return;
        }
        wp = (*wp).w_next;
    }
    (*rettv).vval.v_number = 0;
}
/// "win_id2tabwin()" function
pub unsafe fn f_win_id2tabwin(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let id: handle_T = tv_get_number(argvars.offset(0)) as handle_T;
    let mut winnr: c_int = 1;
    let mut tabnr: c_int = 1;
    win_get_tabwin(id, &raw mut tabnr, &raw mut winnr);
    let list: *mut list_T = tv_list_alloc_ret(rettv, 2 as ptrdiff_t);
    tv_list_append_number(list, tabnr as varnumber_T);
    tv_list_append_number(list, winnr as varnumber_T);
}
/// "win_id2win()" function
pub unsafe fn f_win_id2win(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let id: c_int = tv_get_number(argvars.offset(0)) as c_int;
    let mut nr: c_int = 1;
    let mut wp: *mut win_T = tab_firstwin(curtab.get());
    while !wp.is_null() {
        if (*wp).handle == id {
            // A window the numbering skips (a hidden float) answers 0.
            (*rettv).vval.v_number = if win_has_winnr(wp, curtab.get()) {
                nr as varnumber_T
            } else {
                0
            };
            return;
        }
        nr += win_has_winnr(wp, curtab.get()) as c_int;
        wp = (*wp).w_next;
    }
    (*rettv).vval.v_number = 0;
}
/// "win_findbuf()" function
pub unsafe fn f_win_findbuf(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let list = tv_list_alloc_ret(rettv, kListLenMayKnow as ptrdiff_t);
    let bufnr: c_int = tv_get_number(argvars.offset(0)) as c_int;
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = tab_firstwin(tp);
        while !wp.is_null() {
            if (*(*wp).w_buffer).handle == bufnr {
                tv_list_append_number(list, (*wp).handle as varnumber_T);
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
/// "win_gotoid()" function
pub unsafe fn f_win_gotoid(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut id: c_int = tv_get_number(argvars.offset(0)) as c_int;
    if (*curwin.get()).handle == id {
        (*rettv).vval.v_number = 1;
        return;
    }
    if text_or_buf_locked() {
        return;
    }
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = tab_firstwin(tp);
        while !wp.is_null() {
            if (*wp).handle == id {
                if VIsual_active.get() && (*wp).w_buffer != curbuf.get() {
                    end_visual_mode();
                }
                goto_tabpage_win(tp as *mut tabpage_T, wp);
                (*rettv).vval.v_number = 1;
                return;
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
/// "winnr()" function
pub unsafe fn f_winnr(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    (*rettv).vval.v_number = get_winnr(curtab.get(), argvars.offset(0)) as varnumber_T;
}
/// "tabpagenr()" function
pub unsafe fn f_tabpagenr(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut nr: c_int = 1;
    if (*argvars.offset(0)).v_type != VAR_UNKNOWN {
        let arg: *const c_char = tv_get_string_chk(argvars.offset(0));
        nr = 0;
        if !arg.is_null() {
            if strcmp(arg, c"$".as_ptr()) == 0 {
                nr = tabpage_index(ptr::null_mut()) - 1;
            } else if strcmp(arg, c"#".as_ptr()) == 0 {
                nr = if valid_tabpage(lastused_tabpage.get()) {
                    tabpage_index(lastused_tabpage.get())
                } else {
                    0
                };
            } else {
                crate::semsg!(
                    "E15: Invalid expression: \"{}\"",
                    CStr::from_ptr(arg).to_string_lossy()
                );
            }
        }
    } else {
        nr = tabpage_index(curtab.get());
    }
    (*rettv).vval.v_number = nr as varnumber_T;
}
/// "tabpagewinnr()" function
pub unsafe fn f_tabpagewinnr(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut nr: c_int = 1;
    let tp: *mut tabpage_T = find_tabpage(tv_get_number(argvars.offset(0)) as c_int);
    if tp.is_null() {
        nr = 0;
    } else {
        nr = get_winnr(tp, argvars.offset(1));
    }
    (*rettv).vval.v_number = nr as varnumber_T;
}
/// "winbufnr(nr)" function
pub unsafe fn f_winbufnr(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut wp: *mut win_T = find_win_by_nr_or_id(argvars.offset(0));
    if wp.is_null() {
        (*rettv).vval.v_number = -1;
    } else {
        (*rettv).vval.v_number = (*(*wp).w_buffer).handle as varnumber_T;
    };
}
