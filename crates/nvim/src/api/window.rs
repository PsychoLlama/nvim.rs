use crate::api::private::helpers::{
    ERROR_INIT, NIL, Reported, api_clear_error, api_set_error, arena_array, arena_dict, array_add,
    dict_get_value, dict_put, dict_set_var, find_buffer_by_handle, find_window_by_handle, has_key,
    normalize_index, try_enter, try_leave,
};
use crate::api::private::validate::{api_err_exp, api_err_invalid};
use crate::autocmd::is_aucmd_win;
use crate::cursor::check_cursor_col;
use crate::drawscreen::{UPD_NOT_VALID, UPD_VALID, redraw_later};
use crate::eval::window::{restore_win, switch_win, win_execute_after, win_execute_before};
use crate::ex_docmd::ex_win_close;

use crate::lua::executor::nlua_call_ref;
use crate::main::{
    cmdwin_buf, cmdwin_old_curwin, cmdwin_win, curtab, curwin, e_autocmd_close, e_cmdwin,
};
use crate::message::emsg;
use crate::r#move::{update_topline, validate_cursor};
use crate::os::cshim::gettext;
use crate::plines::{win_get_fill, win_text_height};
use crate::pos::MAXCOL;
use crate::types::{
    Arena, Array, Boolean, Buffer, Dict, Error, Integer, KeyDict_win_text_height, LuaRef,
    LuaRetMode, NS, Object, String_0, Tabpage, TryState, Window, buf_T, colnr_T, except_T, int64_t,
    kErrorTypeException, kErrorTypeNone, kErrorTypeValidation, kObjectTypeInteger, linenr_T,
    msglist_T, pos_T, size_t, switchwin_T, tabpage_T, win_T, win_execute_T,
};
use crate::window::{
    can_close_in_cmdwin, win_close, win_close_othertab, win_find_tabpage, win_get_tabwin,
    win_set_buf, win_setheight_win, win_setwidth_win,
};
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
pub unsafe fn nvim_win_get_buf(win: Window) -> Result<Buffer, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return (0 as Buffer).reported(error);
    }
    ((*(*w).w_buffer).handle as Buffer).reported(error)
}
pub unsafe fn nvim_win_set_buf(win: Window, buf: Buffer) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if w.is_null() || b.is_null() {
        return ().reported(error);
    }
    if w == cmdwin_win.get() || w == cmdwin_old_curwin.get() || b == cmdwin_buf.get() {
        api_set_error(
            err,
            kErrorTypeException,
            c"%s".as_ptr(),
            &raw const e_cmdwin as *const ::core::ffi::c_char,
        );
        return ().reported(error);
    }
    win_set_buf(w, b, err);
    ().reported(error)
}
pub unsafe fn nvim_win_get_cursor(win: Window, arena: *mut Arena) -> Result<Array, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut rv: Array = ARRAY_DICT_INIT;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if !w.is_null() {
        rv = arena_array(arena, 2 as size_t);
        array_add(&mut rv, Object::integer((*w).w_cursor.lnum as Integer));
        array_add(&mut rv, Object::integer((*w).w_cursor.col as Integer));
    }
    rv.reported(error)
}
pub unsafe fn nvim_win_set_cursor(win: Window, pos: Array) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return ().reported(error);
    }
    if pos.size != 2 as size_t
        || (*pos.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*pos.items.offset(1 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        api_err_exp(
            err,
            c"pos".as_ptr(),
            c"[row, col] array".as_ptr(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
        return ().reported(error);
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
            c"cursor line".as_ptr(),
            c"out of range".as_ptr(),
            0 as int64_t,
            false,
        );
        return ().reported(error);
    }
    if col > MAXCOL as ::core::ffi::c_int as int64_t || col < 0 as int64_t {
        api_err_invalid(
            err,
            c"cursor column".as_ptr(),
            c"out of range".as_ptr(),
            0 as int64_t,
            false,
        );
        return ().reported(error);
    }
    (*w).w_cursor.lnum = row as linenr_T;
    (*w).w_cursor.col = col as colnr_T;
    (*w).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    check_cursor_col(w);
    (*w).w_set_curswant = 1;
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
        true,
    );
    update_topline(curwin.get());
    validate_cursor(curwin.get());
    restore_win(&raw mut switchwin, true);
    redraw_later(w, UPD_VALID);
    (*w).w_redr_status = true;
    ().reported(error)
}
pub unsafe fn nvim_win_get_height(win: Window) -> Result<Integer, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return (0 as Integer).reported(error);
    }
    ((*w).w_height as Integer).reported(error)
}
pub unsafe fn nvim_win_set_height(win: Window, height: Integer) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return ().reported(error);
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
    ().reported(error)
}
pub unsafe fn nvim_win_get_width(win: Window) -> Result<Integer, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return (0 as Integer).reported(error);
    }
    ((*w).w_width as Integer).reported(error)
}
pub unsafe fn nvim_win_set_width(win: Window, width: Integer) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return ().reported(error);
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
    ().reported(error)
}
pub unsafe fn nvim_win_get_var(
    win: Window,
    name: String_0,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return NIL.reported(error);
    }
    dict_get_value((*w).w_vars, name, arena, err).reported(error)
}
pub unsafe fn nvim_win_set_var(win: Window, name: String_0, value: Object) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return ().reported(error);
    }
    dict_set_var(
        (*w).w_vars,
        name,
        value,
        false,
        false,
        ::core::ptr::null_mut::<Arena>(),
        err,
    );
    ().reported(error)
}
pub unsafe fn nvim_win_del_var(win: Window, name: String_0) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return ().reported(error);
    }
    dict_set_var(
        (*w).w_vars,
        name,
        NIL,
        true,
        false,
        ::core::ptr::null_mut::<Arena>(),
        err,
    );
    ().reported(error)
}
pub unsafe fn nvim_win_get_position(win: Window, arena: *mut Arena) -> Result<Array, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut rv: Array = ARRAY_DICT_INIT;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if !w.is_null() {
        rv = arena_array(arena, 2 as size_t);
        array_add(&mut rv, Object::integer((*w).w_winrow as Integer));
        array_add(&mut rv, Object::integer((*w).w_wincol as Integer));
    }
    rv.reported(error)
}
pub unsafe fn nvim_win_get_tabpage(win: Window) -> Result<Tabpage, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut rv: Tabpage = 0 as Tabpage;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if !w.is_null() {
        rv = (*win_find_tabpage(w)).handle as Tabpage;
    }
    rv.reported(error)
}
pub unsafe fn nvim_win_get_number(win: Window) -> Result<Integer, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut rv: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return (rv as Integer).reported(error);
    }
    let mut tabnr: ::core::ffi::c_int = 0;
    win_get_tabwin((*w).handle, &raw mut tabnr, &raw mut rv);
    (rv as Integer).reported(error)
}
pub unsafe fn nvim_win_is_valid(win: Window) -> Boolean {
    let mut stub: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut ret: Boolean = !find_window_by_handle(win, &raw mut stub).is_null();
    api_clear_error(&raw mut stub);
    ret
}
pub unsafe fn nvim_win_hide(win: Window) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() || !can_close_in_cmdwin(w, err) {
        return ().reported(error);
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
    ().reported(error)
}
pub unsafe fn nvim_win_close(win: Window, force: Boolean) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() || !can_close_in_cmdwin(w, err) {
        return ().reported(error);
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
    ().reported(error)
}
pub unsafe fn nvim_win_call(win: Window, fun: LuaRef) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return NIL.reported(error);
    }
    let mut tabpage: *mut tabpage_T = win_find_tabpage(w);
    let mut res: Object = NIL;
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
    res.reported(error)
}
pub unsafe fn nvim_win_set_hl_ns(win: Window, ns_id: Integer) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return ().reported(error);
    }
    if !(ns_id >= -1 as Integer) {
        api_err_invalid(err, c"namespace".as_ptr(), c"".as_ptr(), 0 as int64_t, true);
        return ().reported(error);
    }
    (*w).w_ns_hl = ns_id as NS as ::core::ffi::c_int;
    (*w).w_hl_needs_update = 1;
    redraw_later(w, UPD_NOT_VALID);
    ().reported(error)
}
pub unsafe fn nvim_win_text_height(
    win: Window,
    opts: *mut KeyDict_win_text_height,
    arena: *mut Arena,
) -> Result<Dict, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    // Upstream asks for two and writes four (`all`, `fill`, `end_row`,
    // `end_vcol`), so every successful call overruns the arena block by two
    // `KeyValuePair`s.  `dict_put`'s capacity assertion is what found it.
    let mut rv: Dict = arena_dict(arena, 4 as size_t);
    let w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return rv.reported(error);
    }
    let buf: *mut buf_T = (*w).w_buffer;
    let line_count: linenr_T = (*buf).b_ml.ml_line_count;
    let mut start_lnum: linenr_T = 1 as linenr_T;
    let mut end_lnum: linenr_T = line_count;
    let mut start_vcol: int64_t = -1 as int64_t;
    let mut end_vcol: int64_t = -1 as int64_t;
    let mut oob: bool = false;
    if has_key(
        (*opts).is_set__win_text_height_,
        KEYSET_OPTIDX_win_text_height__start_row,
    ) {
        start_lnum =
            normalize_index(buf, (*opts).start_row as int64_t, false, &raw mut oob) as linenr_T;
    }
    if has_key(
        (*opts).is_set__win_text_height_,
        KEYSET_OPTIDX_win_text_height__end_row,
    ) {
        end_lnum =
            normalize_index(buf, (*opts).end_row as int64_t, false, &raw mut oob) as linenr_T;
    }
    if oob {
        api_set_error(
            err,
            kErrorTypeValidation,
            c"%s".as_ptr(),
            c"Line index out of bounds".as_ptr(),
        );
        return rv.reported(error);
    }
    if !(start_lnum <= end_lnum) {
        api_set_error(
            err,
            kErrorTypeValidation,
            c"%s".as_ptr(),
            c"'start_row' is higher than 'end_row'".as_ptr(),
        );
        return rv.reported(error);
    }
    if has_key(
        (*opts).is_set__win_text_height_,
        KEYSET_OPTIDX_win_text_height__start_vcol,
    ) {
        if !(has_key((*opts).is_set__win_text_height_, 3 as ::core::ffi::c_int)) {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"%s".as_ptr(),
                c"'start_vcol' specified without 'start_row'".as_ptr(),
            );
            return rv.reported(error);
        }
        start_vcol = (*opts).start_vcol as int64_t;
        if !(start_vcol >= 0 as int64_t && start_vcol <= MAXCOL as ::core::ffi::c_int as int64_t) {
            api_err_invalid(
                err,
                c"start_vcol".as_ptr(),
                c"out of range".as_ptr(),
                0 as int64_t,
                false,
            );
            return rv.reported(error);
        }
    }
    if has_key(
        (*opts).is_set__win_text_height_,
        KEYSET_OPTIDX_win_text_height__end_vcol,
    ) {
        if !(has_key((*opts).is_set__win_text_height_, 1 as ::core::ffi::c_int)) {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"%s".as_ptr(),
                c"'end_vcol' specified without 'end_row'".as_ptr(),
            );
            return rv.reported(error);
        }
        end_vcol = (*opts).end_vcol as int64_t;
        if !(end_vcol >= 0 as int64_t && end_vcol <= MAXCOL as ::core::ffi::c_int as int64_t) {
            api_err_invalid(
                err,
                c"end_vcol".as_ptr(),
                c"out of range".as_ptr(),
                0 as int64_t,
                false,
            );
            return rv.reported(error);
        }
    }
    let mut max: int64_t = INT64_MAX as int64_t;
    if has_key(
        (*opts).is_set__win_text_height_,
        KEYSET_OPTIDX_win_text_height__max_height,
    ) {
        if !((*opts).max_height > 0 as Integer) {
            api_err_invalid(
                err,
                c"max_height".as_ptr(),
                c"out of range".as_ptr(),
                0 as int64_t,
                false,
            );
            return rv.reported(error);
        }
        max = (*opts).max_height as int64_t;
    }
    if start_lnum == end_lnum
        && start_vcol >= 0 as int64_t
        && end_vcol >= 0 as int64_t
        && !(start_vcol <= end_vcol)
    {
        api_set_error(
            err,
            kErrorTypeValidation,
            c"%s".as_ptr(),
            c"'start_vcol' is higher than 'end_vcol'".as_ptr(),
        );
        return rv.reported(error);
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
    if !(has_key(
        (*opts).is_set__win_text_height_,
        KEYSET_OPTIDX_win_text_height__end_row,
    )) {
        let end_fill: int64_t = win_get_fill(w, line_count + 1 as linenr_T) as int64_t;
        fill += end_fill;
        all += end_fill;
    }
    dict_put(&mut rv, c"all", Object::integer(all));
    dict_put(&mut rv, c"fill", Object::integer(fill));
    dict_put(
        &mut rv,
        c"end_row",
        Object::integer((end_lnum - 1 as linenr_T) as Integer),
    );
    dict_put(&mut rv, c"end_vcol", Object::integer(end_vcol));
    rv.reported(error)
}
