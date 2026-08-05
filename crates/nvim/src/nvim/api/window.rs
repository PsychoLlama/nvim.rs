use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_set_error, arena_array, arena_dict, cstr_as_string, dict_get_value,
    dict_set_var, find_buffer_by_handle, find_window_by_handle, normalize_index, try_enter,
    try_leave,
};
use crate::src::nvim::api::private::validate::{api_err_exp, api_err_invalid};
use crate::src::nvim::autocmd::is_aucmd_win;
use crate::src::nvim::cursor::check_cursor_col;
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, UPD_VALID, redraw_later};
use crate::src::nvim::eval::window::{
    restore_win, switch_win, win_execute_after, win_execute_before,
};
use crate::src::nvim::ex_docmd::ex_win_close;

use crate::src::nvim::lua::executor::nlua_call_ref;
use crate::src::nvim::main::{
    cmdwin_buf, cmdwin_old_curwin, cmdwin_win, curtab, curwin, e_autocmd_close, e_cmdwin,
};
use crate::src::nvim::message::emsg;
use crate::src::nvim::r#move::{update_topline, validate_cursor};
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::plines::{win_get_fill, win_text_height};
use crate::src::nvim::pos::MAXCOL;
use crate::src::nvim::types::{
    Arena, Array, Boolean, Buffer, Dict, Error, Integer, KeyDict_win_text_height, LuaRef,
    LuaRetMode, NS, Object, String_0, Tabpage, TryState, Window, buf_T, colnr_T, except_T, int64_t,
    kErrorTypeException, kErrorTypeNone, kErrorTypeValidation, kObjectTypeInteger, kObjectTypeNil,
    key_value_pair, linenr_T, msglist_T, object, object_data as C2Rust_Unnamed, pos_T, size_t,
    switchwin_T, tabpage_T, win_T, win_execute_T,
};
use crate::src::nvim::window::{
    can_close_in_cmdwin, win_close, win_close_othertab, win_find_tabpage, win_get_tabwin,
    win_set_buf, win_setheight_win, win_setwidth_win,
};
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const kRetLuaref: LuaRetMode = 2;
pub const kRetNilBool: LuaRetMode = 1;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const KEYSET_OPTIDX_win_text_height__end_row: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_text_height__end_vcol: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_text_height__start_row: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_text_height__max_height: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_text_height__start_vcol: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub unsafe extern "C" fn nvim_win_get_buf(mut win: Window, mut err: *mut Error) -> Buffer {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return 0 as Buffer;
    }
    return (*(*w).w_buffer).handle as Buffer;
}
pub unsafe extern "C" fn nvim_win_set_buf(mut win: Window, mut buf: Buffer, mut err: *mut Error) {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if w.is_null() || b.is_null() {
        return;
    }
    if w == cmdwin_win.get() || w == cmdwin_old_curwin.get() || b == cmdwin_buf.get() {
        api_set_error(
            err,
            kErrorTypeException,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            &raw const e_cmdwin as *const ::core::ffi::c_char,
        );
        return;
    }
    win_set_buf(w, b, err);
}
pub unsafe extern "C" fn nvim_win_get_cursor(
    mut win: Window,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut rv: Array = ARRAY_DICT_INIT;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if !w.is_null() {
        rv = arena_array(arena, 2 as size_t);
        let c2rust_fresh0 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh0 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (*w).w_cursor.lnum as Integer,
            },
        };
        let c2rust_fresh1 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh1 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (*w).w_cursor.col as Integer,
            },
        };
    }
    return rv;
}
pub unsafe extern "C" fn nvim_win_set_cursor(mut win: Window, mut pos: Array, mut err: *mut Error) {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return;
    }
    if pos.size != 2 as size_t
        || (*pos.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*pos.items.offset(1 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        api_err_exp(
            err,
            b"pos\0".as_ptr() as *const ::core::ffi::c_char,
            b"[row, col] array\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
        return;
    }
    let mut row: int64_t = (*pos.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .integer as int64_t;
    let mut col: int64_t = (*pos.items.offset(1 as ::core::ffi::c_int as isize))
        .data
        .integer as int64_t;
    if row <= 0 as int64_t || row > (*(*w).w_buffer).b_ml.ml_line_count as int64_t {
        api_err_invalid(
            err,
            b"cursor line\0".as_ptr() as *const ::core::ffi::c_char,
            b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
            0 as int64_t,
            false_0 != 0,
        );
        return;
    }
    if col > MAXCOL as ::core::ffi::c_int as int64_t || col < 0 as int64_t {
        api_err_invalid(
            err,
            b"cursor column\0".as_ptr() as *const ::core::ffi::c_char,
            b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
            0 as int64_t,
            false_0 != 0,
        );
        return;
    }
    (*w).w_cursor.lnum = row as linenr_T;
    (*w).w_cursor.col = col as colnr_T;
    (*w).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    check_cursor_col(w);
    (*w).w_set_curswant = true_0;
    let mut switchwin: switchwin_T = switchwin_T {
        sw_curwin: ::core::ptr::null_mut::<win_T>(),
        sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
        sw_same_win: false,
        sw_visual_active: false,
    };
    switch_win(
        &raw mut switchwin,
        w,
        ::core::ptr::null_mut::<tabpage_T>(),
        true_0 != 0,
    );
    update_topline(curwin.get());
    validate_cursor(curwin.get());
    restore_win(&raw mut switchwin, true_0 != 0);
    redraw_later(w, UPD_VALID);
    (*w).w_redr_status = true_0 != 0;
}
pub unsafe extern "C" fn nvim_win_get_height(mut win: Window, mut err: *mut Error) -> Integer {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return 0 as Integer;
    }
    return (*w).w_height as Integer;
}
pub unsafe extern "C" fn nvim_win_set_height(
    mut win: Window,
    mut height: Integer,
    mut err: *mut Error,
) {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return;
    }
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    win_setheight_win(height as ::core::ffi::c_int, w);
    try_leave(&raw mut tstate, err);
}
pub unsafe extern "C" fn nvim_win_get_width(mut win: Window, mut err: *mut Error) -> Integer {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return 0 as Integer;
    }
    return (*w).w_width as Integer;
}
pub unsafe extern "C" fn nvim_win_set_width(
    mut win: Window,
    mut width: Integer,
    mut err: *mut Error,
) {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return;
    }
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    win_setwidth_win(width as ::core::ffi::c_int, w);
    try_leave(&raw mut tstate, err);
}
pub unsafe extern "C" fn nvim_win_get_var(
    mut win: Window,
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return dict_get_value((*w).w_vars, name, arena, err);
}
pub unsafe extern "C" fn nvim_win_set_var(
    mut win: Window,
    mut name: String_0,
    mut value: Object,
    mut err: *mut Error,
) {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return;
    }
    dict_set_var(
        (*w).w_vars,
        name,
        value,
        false_0 != 0,
        false_0 != 0,
        ::core::ptr::null_mut::<Arena>(),
        err,
    );
}
pub unsafe extern "C" fn nvim_win_del_var(
    mut win: Window,
    mut name: String_0,
    mut err: *mut Error,
) {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return;
    }
    dict_set_var(
        (*w).w_vars,
        name,
        object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        true_0 != 0,
        false_0 != 0,
        ::core::ptr::null_mut::<Arena>(),
        err,
    );
}
pub unsafe extern "C" fn nvim_win_get_position(
    mut win: Window,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut rv: Array = ARRAY_DICT_INIT;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if !w.is_null() {
        rv = arena_array(arena, 2 as size_t);
        let c2rust_fresh2 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh2 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (*w).w_winrow as Integer,
            },
        };
        let c2rust_fresh3 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh3 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (*w).w_wincol as Integer,
            },
        };
    }
    return rv;
}
pub unsafe extern "C" fn nvim_win_get_tabpage(mut win: Window, mut err: *mut Error) -> Tabpage {
    let mut rv: Tabpage = 0 as Tabpage;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if !w.is_null() {
        rv = (*win_find_tabpage(w)).handle as Tabpage;
    }
    return rv;
}
pub unsafe extern "C" fn nvim_win_get_number(mut win: Window, mut err: *mut Error) -> Integer {
    let mut rv: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return rv as Integer;
    }
    let mut tabnr: ::core::ffi::c_int = 0;
    win_get_tabwin((*w).handle, &raw mut tabnr, &raw mut rv);
    return rv as Integer;
}
pub unsafe extern "C" fn nvim_win_is_valid(mut win: Window) -> Boolean {
    let mut stub: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut ret: Boolean = !find_window_by_handle(win, &raw mut stub).is_null();
    api_clear_error(&raw mut stub);
    return ret;
}
pub unsafe extern "C" fn nvim_win_hide(mut win: Window, mut err: *mut Error) {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() || !can_close_in_cmdwin(w, err) {
        return;
    }
    let mut tabpage: *mut tabpage_T = win_find_tabpage(w);
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    if is_aucmd_win(w) {
        emsg(gettext(
            &raw const e_autocmd_close as *const ::core::ffi::c_char,
        ));
    } else if tabpage == curtab.get() {
        win_close(w, false, false);
    } else {
        win_close_othertab(w, 0 as ::core::ffi::c_int, tabpage, false);
    }
    try_leave(&raw mut tstate, err);
}
pub unsafe extern "C" fn nvim_win_close(mut win: Window, mut force: Boolean, mut err: *mut Error) {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() || !can_close_in_cmdwin(w, err) {
        return;
    }
    let mut tabpage: *mut tabpage_T = win_find_tabpage(w);
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    ex_win_close(
        force as ::core::ffi::c_int,
        w,
        if tabpage == curtab.get() {
            ::core::ptr::null_mut::<tabpage_T>()
        } else {
            tabpage
        },
    );
    try_leave(&raw mut tstate, err);
}
pub unsafe extern "C" fn nvim_win_call(
    mut win: Window,
    mut fun: LuaRef,
    mut err: *mut Error,
) -> Object {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    let mut tabpage: *mut tabpage_T = win_find_tabpage(w);
    let mut res: Object = object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    let mut win_execute_args: win_execute_T = win_execute_T {
        wp: ::core::ptr::null_mut::<win_T>(),
        curpos: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        cwd: [0; 4096],
        cwd_status: 0,
        apply_acd: false,
        save_sfname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        switchwin: switchwin_T {
            sw_curwin: ::core::ptr::null_mut::<win_T>(),
            sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
            sw_same_win: false,
            sw_visual_active: false,
        },
    };
    if win_execute_before(&raw mut win_execute_args, w, tabpage) {
        let mut args: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        res = nlua_call_ref(
            fun,
            ::core::ptr::null::<::core::ffi::c_char>(),
            args,
            kRetLuaref,
            ::core::ptr::null_mut::<Arena>(),
            err,
        );
    }
    win_execute_after(&raw mut win_execute_args);
    try_leave(&raw mut tstate, err);
    return res;
}
pub unsafe extern "C" fn nvim_win_set_hl_ns(
    mut win: Window,
    mut ns_id: Integer,
    mut err: *mut Error,
) {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return;
    }
    if !(ns_id >= -1 as Integer) {
        api_err_invalid(
            err,
            b"namespace\0".as_ptr() as *const ::core::ffi::c_char,
            b"\0".as_ptr() as *const ::core::ffi::c_char,
            0 as int64_t,
            true_0 != 0,
        );
        return;
    }
    (*w).w_ns_hl = ns_id as NS as ::core::ffi::c_int;
    (*w).w_hl_needs_update = true_0;
    redraw_later(w, UPD_NOT_VALID);
}
pub unsafe extern "C" fn nvim_win_text_height(
    mut win: Window,
    mut opts: *mut KeyDict_win_text_height,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut rv: Dict = arena_dict(arena, 2 as size_t);
    let w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return rv;
    }
    let buf: *mut buf_T = (*w).w_buffer;
    let line_count: linenr_T = (*buf).b_ml.ml_line_count;
    let mut start_lnum: linenr_T = 1 as linenr_T;
    let mut end_lnum: linenr_T = line_count;
    let mut start_vcol: int64_t = -1 as int64_t;
    let mut end_vcol: int64_t = -1 as int64_t;
    let mut oob: bool = false_0 != 0;
    if (*opts).is_set__win_text_height_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_text_height__start_row
        != 0 as ::core::ffi::c_ulonglong
    {
        start_lnum = normalize_index(
            buf,
            (*opts).start_row as int64_t,
            false_0 != 0,
            &raw mut oob,
        ) as linenr_T;
    }
    if (*opts).is_set__win_text_height_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_text_height__end_row
        != 0 as ::core::ffi::c_ulonglong
    {
        end_lnum = normalize_index(buf, (*opts).end_row as int64_t, false_0 != 0, &raw mut oob)
            as linenr_T;
    }
    if oob {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            b"Line index out of bounds\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return rv;
    }
    if !(start_lnum <= end_lnum) {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            b"'start_row' is higher than 'end_row'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return rv;
    }
    if (*opts).is_set__win_text_height_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_text_height__start_vcol
        != 0 as ::core::ffi::c_ulonglong
    {
        if !((*opts).is_set__win_text_height_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 3 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong)
        {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"'start_vcol' specified without 'start_row'\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
            return rv;
        }
        start_vcol = (*opts).start_vcol as int64_t;
        if !(start_vcol >= 0 as int64_t && start_vcol <= MAXCOL as ::core::ffi::c_int as int64_t) {
            api_err_invalid(
                err,
                b"start_vcol\0".as_ptr() as *const ::core::ffi::c_char,
                b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                0 as int64_t,
                false_0 != 0,
            );
            return rv;
        }
    }
    if (*opts).is_set__win_text_height_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_text_height__end_vcol
        != 0 as ::core::ffi::c_ulonglong
    {
        if !((*opts).is_set__win_text_height_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong)
        {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"'end_vcol' specified without 'end_row'\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return rv;
        }
        end_vcol = (*opts).end_vcol as int64_t;
        if !(end_vcol >= 0 as int64_t && end_vcol <= MAXCOL as ::core::ffi::c_int as int64_t) {
            api_err_invalid(
                err,
                b"end_vcol\0".as_ptr() as *const ::core::ffi::c_char,
                b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                0 as int64_t,
                false_0 != 0,
            );
            return rv;
        }
    }
    let mut max: int64_t = INT64_MAX as int64_t;
    if (*opts).is_set__win_text_height_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_text_height__max_height
        != 0 as ::core::ffi::c_ulonglong
    {
        if !((*opts).max_height > 0 as Integer) {
            api_err_invalid(
                err,
                b"max_height\0".as_ptr() as *const ::core::ffi::c_char,
                b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                0 as int64_t,
                false_0 != 0,
            );
            return rv;
        }
        max = (*opts).max_height as int64_t;
    }
    if start_lnum == end_lnum && start_vcol >= 0 as int64_t && end_vcol >= 0 as int64_t {
        if !(start_vcol <= end_vcol) {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"'start_vcol' is higher than 'end_vcol'\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return rv;
        }
    }
    let mut fill: int64_t = 0 as int64_t;
    let mut all: int64_t = win_text_height(
        w,
        start_lnum,
        start_vcol,
        &raw mut end_lnum,
        &raw mut end_vcol,
        &raw mut fill,
        max,
    );
    if !((*opts).is_set__win_text_height_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_text_height__end_row
        != 0 as ::core::ffi::c_ulonglong)
    {
        let end_fill: int64_t = win_get_fill(w, line_count + 1 as linenr_T) as int64_t;
        fill += end_fill;
        all += end_fill;
    }
    let c2rust_fresh4 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh4 as isize) = key_value_pair {
        key: cstr_as_string(b"all\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: all },
        },
    };
    let c2rust_fresh5 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh5 as isize) = key_value_pair {
        key: cstr_as_string(b"fill\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: fill },
        },
    };
    let c2rust_fresh6 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh6 as isize) = key_value_pair {
        key: cstr_as_string(b"end_row\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (end_lnum - 1 as linenr_T) as Integer,
            },
        },
    };
    let c2rust_fresh7 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh7 as isize) = key_value_pair {
        key: cstr_as_string(b"end_vcol\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: end_vcol },
        },
    };
    return rv;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
