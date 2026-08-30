//! `nvim_win_*`: the window entry points.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::api::private::helpers::{
    ERROR_INIT, NIL, Reported, api_try, arena_array, arena_dict, array_add, buffer_by_handle,
    dict_get_value, dict_put, dict_set_var, has_key, normalize_index, window_by_handle,
};
use crate::autocmd::is_aucmd_win;
use crate::cursor::check_cursor_col;
use crate::drawscreen::{UPD_NOT_VALID, UPD_VALID};
use crate::eval::window::{restore_win, switch_win, win_execute_after, win_execute_before};
use crate::ex_docmd::ex_win_close;

use crate::api::private::validate::err_expected;
use crate::api::private::validate::err_invalid_ptr;
use crate::api::private::validate::err_out_of_range;
use crate::lua::executor::{kRetLuaref, nlua_call_ref};
use crate::main::{cmdwin_buf, cmdwin_old_curwin, cmdwin_win, curtab, e_autocmd_close, e_cmdwin};
use crate::message::emsg;
use crate::r#move::{update_topline, validate_cursor};
use crate::narrow::number_as_int;
use crate::os::cshim::gettext_ptr;
use crate::plines::{win_get_fill, win_text_height};
use crate::pos::MAXCOL;
use crate::types::{
    Arena, Array, Boolean, Buffer, Dict, Error, Integer, KeyDict_win_text_height, LuaRef, Object,
    String_0, Tabpage, Window, buf_T, int64_t, kObjectTypeInteger, linenr_T, size_t, switchwin_T,
    tabpage_T, win_execute_T,
};
use crate::window::{
    can_close_in_cmdwin, win_close, win_close_othertab, win_find_tabpage, win_get_tabwin,
    win_set_buf, win_setheight_win, win_setwidth_win,
};
use crate::winlayer::Win;
use core::ptr;

/// The buffer `win` is showing.
pub fn nvim_win_get_buf(win: Window) -> Result<Buffer, Error> {
    let mut err = ERROR_INIT;
    let Some(w) = window_by_handle(win, &mut err) else {
        return (0 as Buffer).reported(err);
    };
    (w.buffer().handle as Buffer).reported(err)
}

/// Show `buf` in `win`.
pub fn nvim_win_set_buf(win: Window, buf: Buffer) -> Result<(), Error> {
    let mut err = ERROR_INIT;
    let w = window_by_handle(win, &mut err);
    let b = buffer_by_handle(buf, &mut err);
    let (Some(w), Some(b)) = (w, b) else {
        return ().reported(err);
    };
    if w.raw() == cmdwin_win.get()
        || w.raw() == cmdwin_old_curwin.get()
        || b.raw() == cmdwin_buf.get()
    {
        return Err(Error::exception(e_cmdwin));
    }
    // SAFETY: both handles named a live object, and `err` is this frame's own.
    unsafe { win_set_buf(w.raw(), b.raw(), &raw mut err) };
    ().reported(err)
}

/// `win`'s cursor, as a `[line, column]` pair.
///
/// # Safety
/// `arena` must be the caller's, and live for as long as the answer is.
pub unsafe fn nvim_win_get_cursor(win: Window, arena: *mut Arena) -> Result<Array, Error> {
    let mut err = ERROR_INIT;
    let Some(w) = window_by_handle(win, &mut err) else {
        return Array::EMPTY.reported(err);
    };
    let mut rv = arena_array(arena, 2 as size_t);
    let (lnum, col) = (
        Integer::from(w.w_cursor.lnum),
        Integer::from(w.w_cursor.col),
    );
    // SAFETY: `rv` is the two-slot block the arena just handed back.
    unsafe {
        array_add(&mut rv, Object::integer(lnum));
        array_add(&mut rv, Object::integer(col));
    }
    rv.reported(err)
}

/// Move `win`'s cursor to the `[line, column]` `pos` names.
///
/// # Safety
/// `pos` must point at its own elements.
pub unsafe fn nvim_win_set_cursor(win: Window, pos: Array) -> Result<(), Error> {
    let mut err = ERROR_INIT;
    let Some(mut w) = window_by_handle(win, &mut err) else {
        return ().reported(err);
    };
    // SAFETY: `pos` is the caller's array, per this function's contract.
    let rowcol = unsafe {
        let items = (pos.size == 2).then(|| (*pos.items, *pos.items.add(1)));
        items
            .filter(|(row, col)| {
                row.type_0 == kObjectTypeInteger && col.type_0 == kObjectTypeInteger
            })
            .map(|(row, col)| (row.data.integer as int64_t, col.data.integer as int64_t))
    };
    let Some((row, col)) = rowcol else {
        err = err_expected(c"pos", c"[row, col] array", None);
        return ().reported(err);
    };
    if row <= 0 || row > int64_t::from(w.buffer().line_count()) {
        return Err(err_out_of_range(c"cursor line"));
    }
    if col > int64_t::from(MAXCOL) || col < 0 {
        return Err(err_out_of_range(c"cursor column"));
    }
    w.w_cursor.lnum = number_as_int(row);
    w.w_cursor.col = number_as_int(col);
    w.w_cursor.coladd = 0;
    // SAFETY: `w` is live, and `switchwin` is this frame's own -- nothing the
    // callees run can reach it.
    check_cursor_col(w);
    w.w_set_curswant = true;
    let mut switchwin = switchwin_T::default();
    let any_tab = ptr::null_mut::<tabpage_T>();
    unsafe { switch_win(&raw mut switchwin, w.raw(), any_tab, true) };
    update_topline(unsafe { Win::current() });
    validate_cursor(unsafe { Win::current() });
    unsafe { restore_win(&raw mut switchwin, true) };
    w.redraw_later(UPD_VALID);
    w.w_redr_status = true;
    ().reported(err)
}

/// `win`'s height in text lines.
pub fn nvim_win_get_height(win: Window) -> Result<Integer, Error> {
    let mut err = ERROR_INIT;
    let Some(w) = window_by_handle(win, &mut err) else {
        return (0 as Integer).reported(err);
    };
    Integer::from(w.w_height).reported(err)
}

/// Resize `win` to `height` text lines, taking from or giving to its
/// neighbours.
pub fn nvim_win_set_height(win: Window, height: Integer) -> Result<(), Error> {
    let mut err = ERROR_INIT;
    let Some(w) = window_by_handle(win, &mut err) else {
        return ().reported(err);
    };
    // SAFETY: `w` is live; the resize runs Vimscript, which `api_try` catches.
    api_try(&mut err, |_| unsafe {
        win_setheight_win(number_as_int(height), w.raw());
    });
    ().reported(err)
}

/// `win`'s width in screen columns.
pub fn nvim_win_get_width(win: Window) -> Result<Integer, Error> {
    let mut err = ERROR_INIT;
    let Some(w) = window_by_handle(win, &mut err) else {
        return (0 as Integer).reported(err);
    };
    Integer::from(w.w_width).reported(err)
}

/// Resize `win` to `width` screen columns. See [`nvim_win_set_height`].
pub fn nvim_win_set_width(win: Window, width: Integer) -> Result<(), Error> {
    let mut err = ERROR_INIT;
    let Some(w) = window_by_handle(win, &mut err) else {
        return ().reported(err);
    };
    // SAFETY: as `nvim_win_set_height`.
    api_try(&mut err, |_| unsafe {
        win_setwidth_win(number_as_int(width), w.raw());
    });
    ().reported(err)
}

/// The window-scoped variable `name`.
///
/// # Safety
/// `name` must point at its own bytes, and `arena` must be the caller's.
pub unsafe fn nvim_win_get_var(
    win: Window,
    name: String_0,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut err = ERROR_INIT;
    let Some(w) = window_by_handle(win, &mut err) else {
        return NIL.reported(err);
    };
    // SAFETY: `w` is live, so `w_vars` is its own dictionary; `name` and
    // `arena` are the caller's, per this function's contract.
    let value = unsafe { dict_get_value(w.w_vars, name, arena, &raw mut err) };
    value.reported(err)
}

/// Set the window-scoped variable `name`.
///
/// # Safety
/// `name` and `value` must own their bytes: the store takes them over.
pub unsafe fn nvim_win_set_var(win: Window, name: String_0, value: Object) -> Result<(), Error> {
    let mut err = ERROR_INIT;
    let Some(w) = window_by_handle(win, &mut err) else {
        return ().reported(err);
    };
    let no_arena = ptr::null_mut::<Arena>();
    // SAFETY: as `nvim_win_get_var`; the store takes `value` over.
    unsafe { dict_set_var(w.w_vars, name, value, false, false, no_arena, &raw mut err) };
    ().reported(err)
}

/// Remove the window-scoped variable `name`.
///
/// # Safety
/// `name` must point at its own bytes.
pub unsafe fn nvim_win_del_var(win: Window, name: String_0) -> Result<(), Error> {
    let mut err = ERROR_INIT;
    let Some(w) = window_by_handle(win, &mut err) else {
        return ().reported(err);
    };
    let no_arena = ptr::null_mut::<Arena>();
    // SAFETY: as `nvim_win_set_var`, with the deleting flag set.
    unsafe { dict_set_var(w.w_vars, name, NIL, true, false, no_arena, &raw mut err) };
    ().reported(err)
}

/// `win`'s top-left corner, as a `[row, column]` pair of screen cells.
///
/// # Safety
/// `arena` must be the caller's, and live for as long as the answer is.
pub unsafe fn nvim_win_get_position(win: Window, arena: *mut Arena) -> Result<Array, Error> {
    let mut err = ERROR_INIT;
    let Some(w) = window_by_handle(win, &mut err) else {
        return Array::EMPTY.reported(err);
    };
    let mut rv = arena_array(arena, 2 as size_t);
    let (row, col) = (Integer::from(w.w_winrow), Integer::from(w.w_wincol));
    // SAFETY: as `nvim_win_get_cursor`.
    unsafe {
        array_add(&mut rv, Object::integer(row));
        array_add(&mut rv, Object::integer(col));
    }
    rv.reported(err)
}

/// The tab page `win` is on.
pub fn nvim_win_get_tabpage(win: Window) -> Result<Tabpage, Error> {
    let mut err = ERROR_INIT;
    let Some(w) = window_by_handle(win, &mut err) else {
        return (0 as Tabpage).reported(err);
    };
    // SAFETY: `w` is live, and every live window is on a tab page.
    let handle = unsafe { (*win_find_tabpage(w.raw())).handle };
    (handle as Tabpage).reported(err)
}

/// `win`'s 1-based position within its tab page, as `CTRL-W w` counts.
pub fn nvim_win_get_number(win: Window) -> Result<Integer, Error> {
    let mut err = ERROR_INIT;
    let Some(w) = window_by_handle(win, &mut err) else {
        return (0 as Integer).reported(err);
    };
    let mut tabnr: ::core::ffi::c_int = 0;
    let mut winnr: ::core::ffi::c_int = 0;
    // SAFETY: both counters are this frame's own out-parameters.
    unsafe { win_get_tabwin(w.handle, &raw mut tabnr, &raw mut winnr) };
    Integer::from(winnr).reported(err)
}

/// Whether `win` still names a window.
pub fn nvim_win_is_valid(win: Window) -> Boolean {
    let mut stub: Error = ERROR_INIT;
    let ret = window_by_handle(win, &mut stub).is_some();
    // The message the lookup may have left behind is dropped rather than
    // reported.
    stub.clear();
    ret
}

/// Close `win`, keeping its buffer loaded -- `:hide`.
pub fn nvim_win_hide(win: Window) -> Result<(), Error> {
    let mut err = ERROR_INIT;
    // SAFETY: `w` is live, and `err` is this frame's own.
    let Some(w) = window_by_handle(win, &mut err)
        .filter(|w| unsafe { can_close_in_cmdwin(w.raw(), &raw mut err) })
    else {
        return ().reported(err);
    };
    let tabpage = win_find_tabpage(w.raw());
    let refused = e_autocmd_close.as_ptr();
    let is_aucmd = is_aucmd_win(w.raw());
    let same_tab = tabpage == curtab.get();
    api_try(&mut err, |_| {
        if is_aucmd {
            // SAFETY: `e_autocmd_close` is a static message.
            unsafe { emsg(gettext_ptr(refused)) };
        } else if same_tab {
            // SAFETY: `w` is live; closing runs autocommands, which `api_try`
            // catches.
            unsafe { win_close(w.raw(), false, false) };
        } else {
            // SAFETY: as above, in the tab page `w` is in rather than this one.
            unsafe { win_close_othertab(w.raw(), 0, tabpage, false) };
        }
    });
    ().reported(err)
}

/// Close `win`, unloading its buffer when it was the last window on it.
pub fn nvim_win_close(win: Window, force: Boolean) -> Result<(), Error> {
    let mut err = ERROR_INIT;
    // SAFETY: `w` is live, and `err` is this frame's own.
    let Some(w) = window_by_handle(win, &mut err)
        .filter(|w| unsafe { can_close_in_cmdwin(w.raw(), &raw mut err) })
    else {
        return ().reported(err);
    };
    let tabpage = win_find_tabpage(w.raw());
    // `ex_win_close` reads a null tab page as "the current one", which is the
    // only case where it may close the window the user is in.
    let other_tab = if tabpage == curtab.get() {
        ptr::null_mut::<tabpage_T>()
    } else {
        tabpage
    };
    // SAFETY: as `nvim_win_hide`.
    api_try(&mut err, |_| unsafe {
        ex_win_close(::core::ffi::c_int::from(force), w.raw(), other_tab);
    });
    ().reported(err)
}

/// Call the Lua function `fun` with `win` as the current window.
pub fn nvim_win_call(win: Window, fun: LuaRef) -> Result<Object, Error> {
    let mut err = ERROR_INIT;
    let Some(w) = window_by_handle(win, &mut err) else {
        return NIL.reported(err);
    };
    let tabpage = win_find_tabpage(w.raw());
    let res = api_try(&mut err, |err| {
        let mut switch_args = win_execute_T::default();
        let mut res = NIL;
        // SAFETY: `switch_args` is this frame's own and nothing the call runs
        // can reach it.
        let switched = unsafe { win_execute_before(&raw mut switch_args, w.raw(), tabpage) };
        if switched {
            let no_arena = ptr::null_mut::<Arena>();
            let name = ptr::null::<::core::ffi::c_char>();
            // SAFETY: the call runs Lua, which `api_try` catches.
            res = unsafe { nlua_call_ref(fun, name, Array::EMPTY, kRetLuaref, no_arena, err) };
        }
        // SAFETY: the matching restore of the switch above.
        unsafe { win_execute_after(&raw mut switch_args) };
        res
    });
    res.reported(err)
}

/// Point `win` at highlight namespace `ns_id`, or at the global one for -1.
pub fn nvim_win_set_hl_ns(win: Window, ns_id: Integer) -> Result<(), Error> {
    let mut err = ERROR_INIT;
    let Some(mut w) = window_by_handle(win, &mut err) else {
        return ().reported(err);
    };
    if ns_id < -1 {
        let (name, empty) = (c"namespace".as_ptr(), c"".as_ptr());
        // SAFETY: `err` is this frame's own; the arguments are static strings.
        err = unsafe { err_invalid_ptr(name, empty, 0, true) };
        return ().reported(err);
    }
    w.w_ns_hl = number_as_int(ns_id);
    w.w_hl_needs_update = true;
    w.redraw_later(UPD_NOT_VALID);
    ().reported(err)
}

/// How many screen lines a range of `win`'s buffer occupies once wrapping,
/// folds and virtual lines are taken into account.
///
/// # Safety
/// `opts` must point at a filled-in `KeyDict_win_text_height`, and `arena`
/// must be the caller's.
pub unsafe fn nvim_win_text_height(
    win: Window,
    opts: *mut KeyDict_win_text_height,
    arena: *mut Arena,
) -> Result<Dict, Error> {
    // `opts`' keys, by their index in its `is_set` mask. Function-local so
    // that they cannot collide in the flat namespace `tools/ffigen` renders
    // module-level constants into.
    const OPTIDX_END_ROW: ::core::ffi::c_int = 1;
    const OPTIDX_END_VCOL: ::core::ffi::c_int = 2;
    const OPTIDX_START_ROW: ::core::ffi::c_int = 3;
    const OPTIDX_MAX_HEIGHT: ::core::ffi::c_int = 4;
    const OPTIDX_START_VCOL: ::core::ffi::c_int = 5;

    let mut err = ERROR_INIT;
    // Upstream asks for two and writes four (`all`, `fill`, `end_row`,
    // `end_vcol`), so every successful call overruns the arena block by two
    // `KeyValuePair`s.  `dict_put`'s capacity assertion is what found it.
    let mut rv: Dict = arena_dict(arena, 4 as size_t);
    let Some(w) = window_by_handle(win, &mut err) else {
        return rv.reported(err);
    };
    let buf: *mut buf_T = w.buffer().raw();
    let line_count: linenr_T = w.buffer().line_count();

    // SAFETY: `opts` is the caller's, per this function's contract; `set` and
    // the field reads only touch it.
    let set = |key| unsafe { has_key((*opts).is_set__win_text_height_, key) };
    let mut start_lnum: linenr_T = 1 as linenr_T;
    let mut end_lnum: linenr_T = line_count;
    let mut oob: bool = false;
    // SAFETY: as above; `buf` is live and `oob` is this frame's own.
    if set(OPTIDX_START_ROW) {
        let row = unsafe { (*opts).start_row } as int64_t;
        start_lnum = number_as_int(unsafe { normalize_index(buf, row, false, &raw mut oob) });
    }
    if set(OPTIDX_END_ROW) {
        let row = unsafe { (*opts).end_row } as int64_t;
        end_lnum = number_as_int(unsafe { normalize_index(buf, row, false, &raw mut oob) });
    }
    if oob {
        return Err(Error::validation(c"Line index out of bounds"));
    }
    if start_lnum > end_lnum {
        return Err(Error::validation(c"'start_row' is higher than 'end_row'"));
    }

    let mut start_vcol: int64_t = -1;
    if set(OPTIDX_START_VCOL) {
        if !set(OPTIDX_START_ROW) {
            return Err(Error::validation(
                c"'start_vcol' specified without 'start_row'",
            ));
        }
        // SAFETY: as above.
        start_vcol = unsafe { (*opts).start_vcol as int64_t };
        if !(0..=int64_t::from(MAXCOL)).contains(&start_vcol) {
            return Err(err_out_of_range(c"start_vcol"));
        }
    }
    let mut end_vcol: int64_t = -1;
    if set(OPTIDX_END_VCOL) {
        if !set(OPTIDX_END_ROW) {
            return Err(Error::validation(c"'end_vcol' specified without 'end_row'"));
        }
        // SAFETY: as above.
        end_vcol = unsafe { (*opts).end_vcol as int64_t };
        if !(0..=int64_t::from(MAXCOL)).contains(&end_vcol) {
            return Err(err_out_of_range(c"end_vcol"));
        }
    }
    // SAFETY: as above.
    let max: int64_t = if set(OPTIDX_MAX_HEIGHT) {
        let max_height = unsafe { (*opts).max_height };
        if max_height <= 0 {
            return Err(err_out_of_range(c"max_height"));
        }
        max_height as int64_t
    } else {
        int64_t::MAX
    };
    if start_lnum == end_lnum && start_vcol >= 0 && end_vcol >= 0 && start_vcol > end_vcol {
        return Err(Error::validation(c"'start_vcol' is higher than 'end_vcol'"));
    }

    let mut fill: int64_t = 0;
    let last = (&raw mut end_lnum, &raw mut end_vcol, &raw mut fill);
    // SAFETY: `w` is live and the three counters `last` names are this
    // frame's own.
    let mut all: int64_t =
        unsafe { win_text_height(w, start_lnum, start_vcol, last.0, last.1, last.2, max) };
    if !set(OPTIDX_END_ROW) {
        // With no 'end_row' the answer covers the whole buffer, so the virtual
        // lines below its last line count too.
        //
        // SAFETY: `w` is live.
        let end_fill = int64_t::from(unsafe { win_get_fill(w, line_count + 1) });
        fill += end_fill;
        all += end_fill;
    }
    // SAFETY: `rv` is the four-slot arena block allocated above.
    unsafe { dict_put(&mut rv, c"all", Object::integer(all)) };
    unsafe { dict_put(&mut rv, c"fill", Object::integer(fill)) };
    let end_row = Object::integer(Integer::from(end_lnum - 1));
    unsafe { dict_put(&mut rv, c"end_row", end_row) };
    unsafe { dict_put(&mut rv, c"end_vcol", Object::integer(end_vcol)) };
    rv.reported(err)
}
