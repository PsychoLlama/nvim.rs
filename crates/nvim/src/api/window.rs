//! `nvim_win_*`: the window entry points.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::api::private::helpers::{
    Reported, api_try, arena_array, arena_dict, array_add, dict_get_value, dict_put, dict_set_var,
    find_buffer_by_handle, find_window_by_handle, normalize_index,
};
use crate::autocmd::is_aucmd_win;
use crate::cursor::check_cursor_col;
use crate::drawscreen::{UPD_NOT_VALID, UPD_VALID};
use crate::eval::window::{restore_win, switch_win, win_execute_after, win_execute_before};
use crate::ex_docmd::ex_win_close;
use crate::winlayer::TabPage;

use crate::api::private::validate::{Bad, err_expected, err_invalid, err_out_of_range};
use crate::lua::executor::{kRetLuaref, nlua_call_ref};
use crate::message::emsg;
use crate::message::{e_autocmd_close, e_cmdwin};
use crate::r#move::{update_topline, validate_cursor};
use crate::narrow::number_as_int;
use crate::os::cshim::gettext_ptr;
use crate::plines::{win_get_fill, win_text_height};
use crate::pos::MAXCOL;
use crate::types::{
    ApiDict, Arena, Array, Boolean, BufferHandle, Error, Integer, KeyDict_win_text_height, LineNr,
    LuaRef, Object, String_0, SwitchWin, TabpageHandle, WinExecute, WindowHandle, int64_t, size_t,
};
use crate::window::{
    can_close_in_cmdwin, win_close, win_close_othertab, win_find_tabpage, win_get_tabwin,
    win_set_buf, win_setheight_win, win_setwidth_win,
};
use crate::winlayer::Win;
use crate::winlayer::graph::{cmdwin_buf, cmdwin_old_curwin, cmdwin_win};
use core::ptr;

/// The buffer `win` is showing.
pub fn nvim_win_get_buf(win: WindowHandle) -> Result<BufferHandle, Error> {
    let Some(w) = find_window_by_handle(win)? else {
        return Ok(0 as BufferHandle);
    };
    Ok(w.buffer().handle as BufferHandle)
}

/// Show `buf` in `win`.
pub fn nvim_win_set_buf(win: WindowHandle, buf: BufferHandle) -> Result<(), Error> {
    let w = find_window_by_handle(win)?;
    let b = find_buffer_by_handle(buf)?;
    let (Some(w), Some(b)) = (w, b) else {
        return Ok(());
    };
    if cmdwin_win.get() == Some(w.id())
        || cmdwin_old_curwin.get() == Some(w.id())
        || cmdwin_buf.get() == Some(b.id())
    {
        return Err(Error::exception(e_cmdwin));
    }
    win_set_buf(w, b)
}

/// `win`'s cursor, as a `[line, column]` pair.
///
/// # Safety
/// `arena` must be the caller's, and live for as long as the answer is.
pub unsafe fn nvim_win_get_cursor(win: WindowHandle, arena: *mut Arena) -> Result<Array, Error> {
    let Some(w) = find_window_by_handle(win)? else {
        return Ok(Array::EMPTY);
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
    Ok(rv)
}

/// Move `win`'s cursor to the `[line, column]` `pos` names.
///
/// # Safety
/// `pos` must point at its own elements.
pub unsafe fn nvim_win_set_cursor(win: WindowHandle, pos: Array) -> Result<(), Error> {
    let mut err = Error::none();
    let Some(mut w) = find_window_by_handle(win)? else {
        return Ok(());
    };
    // SAFETY: `pos` is the caller's array, per this function's contract.
    let items = unsafe { (pos.size == 2).then(|| (*pos.items, *pos.items.add(1))) };
    let rowcol = items
        .and_then(|(row, col)| row.as_integer().zip(col.as_integer()))
        .map(|(row, col)| (row as int64_t, col as int64_t));
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
    let mut switchwin = SwitchWin::default();
    // `None`: the window may be on any tab page, and the switch stays here.
    let _ = unsafe { switch_win(&raw mut switchwin, w, None, true) };
    update_topline(Win::current());
    validate_cursor(Win::current());
    unsafe { restore_win(&raw mut switchwin, true) };
    w.redraw_later(UPD_VALID);
    w.w_redr_status = true;
    ().reported(err)
}

/// `win`'s height in text lines.
pub fn nvim_win_get_height(win: WindowHandle) -> Result<Integer, Error> {
    let Some(w) = find_window_by_handle(win)? else {
        return Ok(0 as Integer);
    };
    Ok(Integer::from(w.w_height))
}

/// Resize `win` to `height` text lines, taking from or giving to its
/// neighbours.
pub fn nvim_win_set_height(win: WindowHandle, height: Integer) -> Result<(), Error> {
    let Some(w) = find_window_by_handle(win)? else {
        return Ok(());
    };
    api_try(|| win_setheight_win(number_as_int(height), w))?;
    Ok(())
}

/// `win`'s width in screen columns.
pub fn nvim_win_get_width(win: WindowHandle) -> Result<Integer, Error> {
    let Some(w) = find_window_by_handle(win)? else {
        return Ok(0 as Integer);
    };
    Ok(Integer::from(w.w_width))
}

/// Resize `win` to `width` screen columns. See [`nvim_win_set_height`].
pub fn nvim_win_set_width(win: WindowHandle, width: Integer) -> Result<(), Error> {
    let Some(w) = find_window_by_handle(win)? else {
        return Ok(());
    };
    api_try(|| win_setwidth_win(number_as_int(width), w))?;
    Ok(())
}

/// The window-scoped variable `name`.
///
/// # Safety
/// `name` must point at its own bytes, and `arena` must be the caller's.
pub unsafe fn nvim_win_get_var(
    win: WindowHandle,
    name: String_0,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let Some(w) = find_window_by_handle(win)? else {
        return Ok(Object::Nil);
    };
    // SAFETY: `w` is live, so `w_vars` is its own dictionary; `name` and
    // `arena` are the caller's, per this function's contract.
    unsafe { dict_get_value(w.w_vars, name, arena) }
}

/// Set the window-scoped variable `name`.
///
/// # Safety
/// `name` and `value` must own their bytes: the store takes them over.
pub unsafe fn nvim_win_set_var(
    win: WindowHandle,
    name: String_0,
    value: Object,
) -> Result<(), Error> {
    let Some(w) = find_window_by_handle(win)? else {
        return Ok(());
    };
    let no_arena = ptr::null_mut::<Arena>();
    // SAFETY: as `nvim_win_get_var`; the store takes `value` over.
    unsafe { dict_set_var(w.w_vars, name, value, false, false, no_arena) }.map(|_| ())
}

/// Remove the window-scoped variable `name`.
///
/// # Safety
/// `name` must point at its own bytes.
pub unsafe fn nvim_win_del_var(win: WindowHandle, name: String_0) -> Result<(), Error> {
    let Some(w) = find_window_by_handle(win)? else {
        return Ok(());
    };
    let no_arena = ptr::null_mut::<Arena>();
    // SAFETY: as `nvim_win_set_var`, with the deleting flag set.
    unsafe { dict_set_var(w.w_vars, name, Object::Nil, true, false, no_arena) }.map(|_| ())
}

/// `win`'s top-left corner, as a `[row, column]` pair of screen cells.
///
/// # Safety
/// `arena` must be the caller's, and live for as long as the answer is.
pub unsafe fn nvim_win_get_position(win: WindowHandle, arena: *mut Arena) -> Result<Array, Error> {
    let Some(w) = find_window_by_handle(win)? else {
        return Ok(Array::EMPTY);
    };
    let mut rv = arena_array(arena, 2 as size_t);
    let (row, col) = (Integer::from(w.w_winrow), Integer::from(w.w_wincol));
    // SAFETY: as `nvim_win_get_cursor`.
    unsafe {
        array_add(&mut rv, Object::integer(row));
        array_add(&mut rv, Object::integer(col));
    }
    Ok(rv)
}

/// The tab page a window the API just looked up is on. Every live window is
/// on one, so the `Option` `win_find_tabpage` answers is `Some` here.
fn tab_of(win: Win) -> TabPage {
    win_find_tabpage(win.id()).expect("a live window is on a tab page")
}

/// The tab page `win` is on.
pub fn nvim_win_get_tabpage(win: WindowHandle) -> Result<TabpageHandle, Error> {
    let Some(w) = find_window_by_handle(win)? else {
        return Ok(0 as TabpageHandle);
    };
    let handle = tab_of(w).handle;
    Ok(handle as TabpageHandle)
}

/// `win`'s 1-based position within its tab page, as `CTRL-W w` counts.
pub fn nvim_win_get_number(win: WindowHandle) -> Result<Integer, Error> {
    let Some(w) = find_window_by_handle(win)? else {
        return Ok(0 as Integer);
    };
    let mut tabnr: ::core::ffi::c_int = 0;
    let mut winnr: ::core::ffi::c_int = 0;
    // SAFETY: both counters are this frame's own out-parameters.
    unsafe { win_get_tabwin(w.handle, &raw mut tabnr, &raw mut winnr) };
    Ok(Integer::from(winnr))
}

/// Whether `win` still names a window.
pub fn nvim_win_is_valid(win: WindowHandle) -> Boolean {
    // A handle that names nothing is not an error here, only a `false`.
    find_window_by_handle(win).unwrap_or_default().is_some()
}

/// Close `win`, keeping its buffer loaded -- `:hide`.
pub fn nvim_win_hide(win: WindowHandle) -> Result<(), Error> {
    let Some(w) = find_window_by_handle(win)? else {
        return Ok(());
    };
    if !can_close_in_cmdwin(w)? {
        return Ok(());
    }
    let tabpage = tab_of(w);
    let refused = e_autocmd_close.as_ptr();
    let is_aucmd = is_aucmd_win(w);
    let same_tab = tabpage.is_current();
    api_try(|| {
        if is_aucmd {
            // SAFETY: `e_autocmd_close` is a static message.
            unsafe { emsg(gettext_ptr(refused)) };
        } else if same_tab {
            win_close(w, false, false);
        } else {
            win_close_othertab(w, 0, tabpage, false);
        }
    })
}

/// Close `win`, unloading its buffer when it was the last window on it.
pub fn nvim_win_close(win: WindowHandle, force: Boolean) -> Result<(), Error> {
    let Some(w) = find_window_by_handle(win)? else {
        return Ok(());
    };
    if !can_close_in_cmdwin(w)? {
        return Ok(());
    }
    let tabpage = tab_of(w);
    // `ex_win_close` reads an absent tab page as "the current one", which is
    // the only case where it may close the window the user is in.
    let other_tab = (!tabpage.is_current()).then_some(tabpage);
    api_try(|| {
        ex_win_close(::core::ffi::c_int::from(force), w, other_tab);
    })
}

/// Call the Lua function `fun` with `win` as the current window.
pub fn nvim_win_call(win: WindowHandle, fun: LuaRef) -> Result<Object, Error> {
    let Some(w) = find_window_by_handle(win)? else {
        return Ok(Object::Nil);
    };
    let tabpage = tab_of(w);
    // The inner `Result` is the call's own; the outer one is whatever the
    // bracket caught, which outranks it.
    api_try(|| {
        let mut switch_args = WinExecute::default();
        let mut res = Ok(Object::Nil);
        // SAFETY: `switch_args` is this frame's own and nothing the call runs
        // can reach it.
        let switched = unsafe { win_execute_before(&raw mut switch_args, w, tabpage) };
        if switched {
            let no_arena = ptr::null_mut::<Arena>();
            let name = ptr::null::<::core::ffi::c_char>();
            // SAFETY: the call runs Lua, which `api_try` catches.
            res = unsafe { nlua_call_ref(fun, name, Array::EMPTY, kRetLuaref, no_arena) };
        }
        // SAFETY: the matching restore of the switch above.
        unsafe { win_execute_after(&raw mut switch_args) };
        res
    })?
}

/// Point `win` at highlight namespace `ns_id`, or at the global one for -1.
pub fn nvim_win_set_hl_ns(win: WindowHandle, ns_id: Integer) -> Result<(), Error> {
    let mut err = Error::none();
    let Some(mut w) = find_window_by_handle(win)? else {
        return Ok(());
    };
    if ns_id < -1 {
        err = err_invalid(c"namespace", Bad::Unsaid);
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
    win: WindowHandle,
    opts: *mut KeyDict_win_text_height,
    arena: *mut Arena,
) -> Result<ApiDict, Error> {
    // Upstream asks for two and writes four (`all`, `fill`, `end_row`,
    // `end_vcol`), so every successful call overruns the arena block by two
    // `KeyValuePair`s.  `dict_put`'s capacity assertion is what found it.
    let mut rv: ApiDict = arena_dict(arena, 4 as size_t);
    let Some(w) = find_window_by_handle(win)? else {
        return Ok(rv);
    };
    let buf = w.buffer();
    let line_count: LineNr = w.buffer().line_count();

    // SAFETY: `opts` is the caller's, per this function's contract.
    let opts = unsafe { &*opts };
    let mut start_lnum: LineNr = 1 as LineNr;
    let mut end_lnum: LineNr = line_count;
    let mut oob: bool = false;
    // SAFETY: `buf` is live and `oob` is this frame's own.
    if let Some(row) = opts.start_row {
        start_lnum =
            number_as_int(unsafe { normalize_index(buf, row as int64_t, false, &raw mut oob) });
    }
    if let Some(row) = opts.end_row {
        end_lnum =
            number_as_int(unsafe { normalize_index(buf, row as int64_t, false, &raw mut oob) });
    }
    if oob {
        return Err(Error::validation(c"Line index out of bounds"));
    }
    if start_lnum > end_lnum {
        return Err(Error::validation(c"'start_row' is higher than 'end_row'"));
    }

    let mut start_vcol: int64_t = -1;
    if let Some(vcol) = opts.start_vcol {
        if opts.start_row.is_none() {
            return Err(Error::validation(
                c"'start_vcol' specified without 'start_row'",
            ));
        }
        start_vcol = vcol as int64_t;
        if !(0..=int64_t::from(MAXCOL)).contains(&start_vcol) {
            return Err(err_out_of_range(c"start_vcol"));
        }
    }
    let mut end_vcol: int64_t = -1;
    if let Some(vcol) = opts.end_vcol {
        if opts.end_row.is_none() {
            return Err(Error::validation(c"'end_vcol' specified without 'end_row'"));
        }
        end_vcol = vcol as int64_t;
        if !(0..=int64_t::from(MAXCOL)).contains(&end_vcol) {
            return Err(err_out_of_range(c"end_vcol"));
        }
    }
    let max: int64_t = match opts.max_height {
        Some(max_height) if max_height <= 0 => return Err(err_out_of_range(c"max_height")),
        Some(max_height) => max_height as int64_t,
        None => int64_t::MAX,
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
    if opts.end_row.is_none() {
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
    Ok(rv)
}
