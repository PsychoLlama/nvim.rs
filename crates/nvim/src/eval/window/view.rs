//! Window geometry: the saved view, the resize commands, and moving a window
//! or one of its separators.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::memory::handoff::owned_cstr;
use crate::window::{WSP_ABOVE, WSP_BELOW, WSP_VERT};

/// `getwinpos([{timeout}])` — the GUI's window position, which a terminal
/// never has.
///
/// # Safety
///
/// `_args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_getwinpos(_args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: `result` is the cleared return value, and the list it is given
    // stays alive for the two appends.
    let list = unsafe { tv_list_alloc_ret(result, 2) };
    unsafe { tv_list_append_number(list, -1) };
    unsafe { tv_list_append_number(list, -1) };
}

/// `getwinposx()` — always -1; there is no GUI window.
///
/// # Safety
///
/// `_args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_getwinposx(_args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: `result` is the cleared return value.
    unsafe { (*result).write_number(-1) };
}

/// `getwinposy()` — always -1; there is no GUI window.
///
/// # Safety
///
/// `_args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_getwinposy(_args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: `result` is the cleared return value.
    unsafe { (*result).write_number(-1) };
}

/// The window and the offset a `win_move_*()` call names, once the window has
/// been checked for the two things neither of them can move: a float, which
/// has no separators, and a window in another tab page, whose sizes are not
/// the ones on screen.
fn drag_target(args: Args<'_>) -> Option<(Win, c_int)> {
    let wp = arg_win(args, 0)?;
    if wp.w_floating {
        return None;
    }
    if !win_valid(wp.id()) {
        crate::semsg!("E1308: Cannot resize a window in another tab page");
        return None;
    }
    Some((wp, number_as_int(arg_number(args, 1))))
}

/// `win_move_separator({nr}, {offset})` — drag a vertical separator.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_win_move_separator(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    result.write_number(0);
    let Some((wp, offset)) = drag_target(args) else {
        return;
    };
    win_drag_vsep_line(wp, offset);
    result.write_number(1);
}

/// `win_move_statusline({nr}, {offset})` — drag a status line.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_win_move_statusline(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    result.write_number(0);
    let Some((wp, offset)) = drag_target(args) else {
        return;
    };
    win_drag_status_line(wp, offset);
    result.write_number(1);
}

/// `win_screenpos({nr})` — the window's top-left cell, one-based; `[0, 0]` for
/// a window that does not exist.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_win_screenpos(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    // SAFETY: the arguments and `result` are live typvals; the list stays
    // alive for the two appends because `result` owns it.
    let list = unsafe { tv_list_alloc_ret(result, 2) };
    let wp = arg_win(args, 0);
    let (row, col) = wp.map_or((0, 0), |wp| (wp.w_winrow + 1, wp.w_wincol + 1));
    unsafe { tv_list_append_number(list, VarNumber::from(row)) };
    unsafe { tv_list_append_number(list, VarNumber::from(col)) };
}

/// The `{options}` dictionary `win_splitmove()` takes: the split flags and the
/// size to give the moved window.
///
/// # Safety
/// `opts` must be a live typval holding a non-null Dictionary.
unsafe fn splitmove_options(opts: *mut TypVal) -> (c_int, c_int) {
    // SAFETY: the caller's obligation; `tv_dict_find` hands back a live entry
    // of the same dictionary or NULL.
    let d = unsafe { (*opts).dict_or_null() };
    let mut flags = 0;
    if unsafe { tv_dict_get_number(d, c"vertical".as_ptr()) } != 0 {
        flags |= WSP_VERT.cast_signed();
    }
    let di = unsafe { tv_dict_find(d, c"rightbelow".as_ptr(), -1) };
    if !di.is_null() {
        let below = unsafe { tv_get_number(&raw mut (*di).di_tv) };
        flags |= if below != 0 {
            WSP_BELOW.cast_signed()
        } else {
            WSP_ABOVE.cast_signed()
        };
    }
    (
        flags,
        number_as_int(unsafe { tv_dict_get_number(d, c"size".as_ptr()) }),
    )
}

/// `win_splitmove({nr}, {target} [, {options}])` — 0 when the window moved.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_win_splitmove(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    result.write_number(-1);
    // SAFETY: the arguments are live typvals; the windows the resolver
    // answers are live, and every callee below re-checks validity because an
    // autocommand may close one under it.
    let wp = arg_win(args, 0);
    let targetwin = arg_win(args, 1);
    let oldwin = Win::current();
    let (Some(wp), Some(targetwin)) = (wp, targetwin) else {
        crate::semsg!("E957: Invalid window number");
        return;
    };
    if wp == targetwin || !win_valid(wp.id()) || !win_valid(targetwin.id()) || targetwin.w_floating
    {
        crate::semsg!("E957: Invalid window number");
        return;
    }
    let (flags, size) = if args.has(2) {
        if unsafe { tv_check_for_nonnull_dict_arg(args.ptr(0), 2) }.is_err() {
            return;
        }
        unsafe { splitmove_options(args.ptr(2)) }
    } else {
        (0, 0)
    };
    if is_aucmd_win(wp) || text_or_buf_locked() || check_split_disallowed(wp) == FAIL {
        return;
    }
    if !targetwin.is_current() {
        win_goto(targetwin);
    }
    if targetwin.is_current() && win_valid(wp.id()) {
        if win_splitmove(wp, size, flags).is_ok() {
            result.write_number(0);
        }
    } else {
        crate::semsg!("E855: Autocommands caused command to abort");
    }
    if !oldwin.is_current() && win_valid(oldwin.id()) {
        win_goto(oldwin);
    }
}

/// `wincol()` — the cursor's screen column within the window, one-based.
///
/// # Safety
///
/// `_args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_wincol(_args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: `curwin` is set and `result` is the cleared return value.
    let win = Win::current();
    validate_cursor(win);
    unsafe { (*result).write_number(VarNumber::from(win.w_wcol + 1)) };
}

/// `winline()` — the cursor's screen row within the window, one-based.
///
/// # Safety
///
/// `_args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_winline(_args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: `curwin` is set and `result` is the cleared return value.
    let win = Win::current();
    validate_cursor(win);
    unsafe { (*result).write_number(VarNumber::from(win.w_wrow + 1)) };
}

/// `winheight({nr})` — text height, -1 for a window that does not exist.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_winheight(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    // SAFETY: the arguments are live typvals.
    let wp = arg_win(args, 0);
    result.write_number(wp.map_or(-1, |wp| VarNumber::from(wp.w_view_height)));
}

/// `winwidth({nr})` — text width, -1 for a window that does not exist.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_winwidth(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    // SAFETY: the arguments are live typvals.
    let wp = arg_win(args, 0);
    result.write_number(wp.map_or(-1, |wp| VarNumber::from(wp.w_view_width)));
}

/// `winrestcmd()` — the `:resize` commands that rebuild the current tab page's
/// window sizes.
///
/// The whole thing is emitted twice: setting one window's height changes its
/// neighbours', so a single pass cannot land on the sizes it names.
///
/// # Safety
///
/// `_args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_winrestcmd(_args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: `curtab` is set, and `result` takes the text over at the end.
    let mut cmds = Vec::<u8>::new();
    let tp = TabPage::current();
    // Scoped so the buffer it borrows is free again for the tail below.
    {
        let mut emit = |prefix: &str, winnr: c_int, size: c_int| {
            cmds.extend_from_slice(format!("{prefix}{winnr}resize {size}|").as_bytes());
        };
        let numbered = || (1..).zip(windows_in_tab(tp).filter(|wp| wp.has_winnr(tp)));
        for _ in 0..2 {
            for (winnr, wp) in numbered() {
                emit("", winnr, wp.w_height);
                emit("vert ", winnr, wp.w_width);
            }
        }
    }
    unsafe { (*result).write_string(owned_cstr(cmds)) };
}

/// `winrestview({dict})` — put back what `winsaveview()` saved.
///
/// Every key is optional: what the dictionary does not mention keeps its
/// current value.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `_result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_winrestview(args: *mut TypVal, _result: *mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the arguments are live typvals, and `curwin` is set.
    if unsafe { tv_check_for_nonnull_dict_arg(args, 0) }.is_err() {
        return;
    }
    let dict = unsafe { (*args).dict_or_null() };
    let mut win = Win::current();
    let entry = |key: &CStr| {
        // SAFETY: a live dictionary, and `tv_dict_find` hands back a live
        // entry of it or NULL.
        let di = unsafe { tv_dict_find(dict, key.as_ptr(), key.count_bytes().cast_signed()) };
        (!di.is_null()).then(|| unsafe { tv_get_number(&raw mut (*di).di_tv) })
    };

    if let Some(v) = entry(c"lnum") {
        win.w_cursor.lnum = number_as_int(v);
    }
    if let Some(v) = entry(c"col") {
        win.w_cursor.col = number_as_int(v);
    }
    if let Some(v) = entry(c"coladd") {
        win.w_cursor.coladd = number_as_int(v);
    }
    if let Some(v) = entry(c"curswant") {
        win.w_curswant = number_as_int(v);
        win.w_set_curswant = false;
    }
    if let Some(v) = entry(c"topline") {
        // Not a plain assignment: 'scrolloff' and folds decide where the
        // window can actually start.
        // SAFETY: a live window.
        set_topline(win, number_as_int(v));
    }
    if let Some(v) = entry(c"topfill") {
        win.w_topfill = number_as_int(v);
    }
    if let Some(v) = entry(c"leftcol") {
        win.w_leftcol = number_as_int(v);
    }
    if let Some(v) = entry(c"skipcol") {
        win.w_skipcol = number_as_int(v);
    }

    // SAFETY: a live window, and `curbuf` is set.
    check_cursor(win);
    win_new_height(win, win.w_height);
    win_new_width(win, win.w_width);
    changed_window_setting(win);
    // SAFETY: `curbuf` is set from startup to exit.
    let line_count = Buf::current().line_count();
    win.w_topline = restored_topline(win.w_topline, line_count);
    // SAFETY: a live window.
    check_topfill(win, true);
}

/// The line `winrestview()` leaves `w_topline` at: a saved view from a buffer
/// that has since shrunk can name a line that no longer exists.
///
/// The two bounds are deliberately *not* a `clamp`. Upstream applies them in
/// this order, so when there are no lines at all the second undoes the first
/// and the answer is 0 rather than 1. Only an unloaded buffer has no lines,
/// and `winrestview()` cannot reach one, but the order is what upstream does
/// and a differential would see any other answer.
fn restored_topline(topline: LineNr, line_count: LineNr) -> LineNr {
    let topline = if topline <= 0 { 1 } else { topline };
    if topline > line_count {
        line_count
    } else {
        topline
    }
}

/// `winsaveview()` — everything `winrestview()` puts back.
///
/// # Safety
///
/// `_args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_winsaveview(_args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: `result` is the cleared return value and `curwin` is set; the
    // dictionary stays alive for the appends because `result` owns it.
    unsafe { tv_dict_alloc_ret(result) };
    let dict = unsafe { (*result).dict_or_null() };
    let win = Win::current();
    let nr = |key: &CStr, value: VarNumber| {
        // SAFETY: a live dictionary and a NUL-terminated key.
        let _ = unsafe { tv_dict_add_nr(dict, key.as_ptr(), key.count_bytes(), value) };
    };

    nr(c"lnum", VarNumber::from(win.w_cursor.lnum));
    nr(c"col", VarNumber::from(win.w_cursor.col));
    nr(c"coladd", VarNumber::from(win.w_cursor.coladd));
    // 'curswant' is only up to date once the cursor move has been resolved.
    update_curswant();
    nr(c"curswant", VarNumber::from(win.w_curswant));
    nr(c"topline", VarNumber::from(win.w_topline));
    nr(c"topfill", VarNumber::from(win.w_topfill));
    nr(c"leftcol", VarNumber::from(win.w_leftcol));
    nr(c"skipcol", VarNumber::from(win.w_skipcol));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_restored_topline_is_pulled_inside_the_buffer() {
        // Inside the buffer, nothing moves.
        assert_eq!(restored_topline(1, 10), 1);
        assert_eq!(restored_topline(10, 10), 10);
        // Above the first line, and past the last.
        assert_eq!(restored_topline(0, 10), 1);
        assert_eq!(restored_topline(-5, 10), 1);
        assert_eq!(restored_topline(11, 10), 10);
    }

    #[test]
    fn an_empty_buffer_leaves_the_restored_topline_at_zero() {
        // Not `clamp(1, line_count)`, which would panic on the crossed
        // bounds, and not `max(1)`, which would answer 1: the second bound is
        // applied after the first and wins.
        assert_eq!(restored_topline(0, 0), 0);
        assert_eq!(restored_topline(5, 0), 0);
    }
}
