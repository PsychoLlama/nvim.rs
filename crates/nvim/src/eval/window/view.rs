//! Window geometry: the saved view, the resize commands, and moving a window
//! or one of its separators.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::VAR_STRING;
use crate::window::{WSP_ABOVE, WSP_BELOW, WSP_VERT};

/// `getwinpos([{timeout}])` — the GUI's window position, which a terminal
/// never has.
pub unsafe fn f_getwinpos(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `rettv` is the cleared return value, and the list it is given
    // stays alive for the two appends.
    unsafe {
        let list = tv_list_alloc_ret(rettv, 2);
        tv_list_append_number(list, -1);
        tv_list_append_number(list, -1);
    };
}

/// `getwinposx()` — always -1; there is no GUI window.
pub unsafe fn f_getwinposx(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `rettv` is the cleared return value.
    unsafe { (*rettv).vval.v_number = -1 };
}

/// `getwinposy()` — always -1; there is no GUI window.
pub unsafe fn f_getwinposy(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `rettv` is the cleared return value.
    unsafe { (*rettv).vval.v_number = -1 };
}

/// The window and the offset a `win_move_*()` call names, once the window has
/// been checked for the two things neither of them can move: a float, which
/// has no separators, and a window in another tab page, whose sizes are not
/// the ones on screen.
///
/// # Safety
/// The arguments must be live typvals.
unsafe fn drag_target(args: Args<'_>) -> Option<(Win, c_int)> {
    // SAFETY: the caller's obligation.
    unsafe {
        let wp = find_win_by_nr_or_id(args.ptr(0))?;
        if wp.w_floating {
            return None;
        }
        if !win_valid(wp.raw()) {
            crate::semsg!("E1308: Cannot resize a window in another tab page");
            return None;
        }
        Some((wp, number_as_int(tv_get_number(args.ptr(1)))))
    }
}

/// `win_move_separator({nr}, {offset})` — drag a vertical separator.
pub unsafe fn f_win_move_separator(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = 0;
    // SAFETY: the arguments are live typvals and the window is live.
    unsafe {
        let Some((wp, offset)) = drag_target(args) else {
            return;
        };
        win_drag_vsep_line(wp.raw(), offset);
    }
    rettv.vval.v_number = 1;
}

/// `win_move_statusline({nr}, {offset})` — drag a status line.
pub unsafe fn f_win_move_statusline(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = 0;
    // SAFETY: the arguments are live typvals and the window is live.
    unsafe {
        let Some((wp, offset)) = drag_target(args) else {
            return;
        };
        win_drag_status_line(wp.raw(), offset);
    }
    rettv.vval.v_number = 1;
}

/// `win_screenpos({nr})` — the window's top-left cell, one-based; `[0, 0]` for
/// a window that does not exist.
pub unsafe fn f_win_screenpos(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals; the list stays
    // alive for the two appends because `rettv` owns it.
    unsafe {
        let list = tv_list_alloc_ret(rettv, 2);
        let wp = find_win_by_nr_or_id(args.ptr(0));
        let (row, col) = wp.map_or((0, 0), |wp| (wp.w_winrow + 1, wp.w_wincol + 1));
        tv_list_append_number(list, varnumber_T::from(row));
        tv_list_append_number(list, varnumber_T::from(col));
    };
}

/// The `{options}` dictionary `win_splitmove()` takes: the split flags and the
/// size to give the moved window.
///
/// # Safety
/// `opts` must be a live typval holding a non-null Dictionary.
unsafe fn splitmove_options(opts: *mut typval_T) -> (c_int, c_int) {
    // SAFETY: the caller's obligation; `tv_dict_find` hands back a live entry
    // of the same dictionary or NULL.
    unsafe {
        let d = (*opts).vval.v_dict;
        let mut flags = 0;
        if tv_dict_get_number(d, c"vertical".as_ptr()) != 0 {
            flags |= WSP_VERT as c_int;
        }
        let di = tv_dict_find(d, c"rightbelow".as_ptr(), -1);
        if !di.is_null() {
            flags |= if tv_get_number(&raw mut (*di).di_tv) != 0 {
                WSP_BELOW as c_int
            } else {
                WSP_ABOVE as c_int
            };
        }
        (
            flags,
            number_as_int(tv_dict_get_number(d, c"size".as_ptr())),
        )
    }
}

/// `win_splitmove({nr}, {target} [, {options}])` — 0 when the window moved.
pub unsafe fn f_win_splitmove(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = -1;
    // SAFETY: the arguments are live typvals; the windows the resolver
    // answers are live, and every callee below re-checks validity because an
    // autocommand may close one under it.
    unsafe {
        let wp = find_win_by_nr_or_id(args.ptr(0));
        let targetwin = find_win_by_nr_or_id(args.ptr(1));
        let oldwin = Win::current();
        let (Some(wp), Some(targetwin)) = (wp, targetwin) else {
            crate::semsg!("E957: Invalid window number");
            return;
        };
        if wp == targetwin
            || !win_valid(wp.raw())
            || !win_valid(targetwin.raw())
            || targetwin.w_floating
        {
            crate::semsg!("E957: Invalid window number");
            return;
        }
        let (flags, size) = if args.has(2) {
            if tv_check_for_nonnull_dict_arg(argvars, 2) == FAIL {
                return;
            }
            splitmove_options(args.ptr(2))
        } else {
            (0, 0)
        };
        if is_aucmd_win(wp.raw())
            || text_or_buf_locked()
            || check_split_disallowed(wp.raw()) == FAIL
        {
            return;
        }
        if !targetwin.is_current() {
            win_goto(targetwin.raw());
        }
        if targetwin.is_current() && win_valid(wp.raw()) {
            if win_splitmove(wp.raw(), size, flags) == OK {
                rettv.vval.v_number = 0;
            }
        } else {
            crate::semsg!("E855: Autocommands caused command to abort");
        }
        if !oldwin.is_current() && win_valid(oldwin.raw()) {
            win_goto(oldwin.raw());
        }
    }
}

/// `wincol()` — the cursor's screen column within the window, one-based.
pub unsafe fn f_wincol(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `curwin` is set and `rettv` is the cleared return value.
    unsafe {
        let win = Win::current();
        validate_cursor(win.raw());
        (*rettv).vval.v_number = varnumber_T::from(win.w_wcol + 1);
    }
}

/// `winline()` — the cursor's screen row within the window, one-based.
pub unsafe fn f_winline(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `curwin` is set and `rettv` is the cleared return value.
    unsafe {
        let win = Win::current();
        validate_cursor(win.raw());
        (*rettv).vval.v_number = varnumber_T::from(win.w_wrow + 1);
    }
}

/// `winheight({nr})` — text height, -1 for a window that does not exist.
pub unsafe fn f_winheight(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments are live typvals.
    let wp = unsafe { find_win_by_nr_or_id(args.ptr(0)) };
    rettv.vval.v_number = wp.map_or(-1, |wp| varnumber_T::from(wp.w_view_height));
}

/// `winwidth({nr})` — text width, -1 for a window that does not exist.
pub unsafe fn f_winwidth(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments are live typvals.
    let wp = unsafe { find_win_by_nr_or_id(args.ptr(0)) };
    rettv.vval.v_number = wp.map_or(-1, |wp| varnumber_T::from(wp.w_view_width));
}

/// `winrestcmd()` — the `:resize` commands that rebuild the current tab page's
/// window sizes.
///
/// The whole thing is emitted twice: setting one window's height changes its
/// neighbours', so a single pass cannot land on the sizes it names.
pub unsafe fn f_winrestcmd(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `curtab` is set; `ga` and `buf` are live locals of this frame,
    // and the growarray's buffer is handed to `rettv` at the end rather than
    // freed.
    let (mut buf, mut ga, tp) = unsafe {
        let mut ga: garray_T = mem::zeroed();
        ga_init(&raw mut ga, size_of::<c_char>() as c_int, 70);
        ([0 as c_char; 50], ga, TabPage::current())
    };
    // Scoped so the growarray it borrows is free again for the tail below.
    {
        let mut emit = |format: &CStr, winnr: c_int, size: c_int| {
            // SAFETY: a NUL-terminated format with exactly the two `%d`s it
            // is given, formatted into a live local and appended to a live
            // growarray.
            unsafe {
                let len = vim_snprintf_safelen(
                    buf.as_mut_ptr(),
                    size_of_val(&buf),
                    format.as_ptr(),
                    winnr,
                    size,
                );
                ga_concat_len(&raw mut ga, buf.as_mut_ptr(), len);
            }
        };
        let numbered = || (1..).zip(windows_in_tab(tp).filter(|wp| wp.has_winnr(tp)));
        for _ in 0..2 {
            for (winnr, wp) in numbered() {
                emit(c"%dresize %d|", winnr, wp.w_height);
                emit(c"vert %dresize %d|", winnr, wp.w_width);
            }
        }
    }
    // SAFETY: a live growarray, whose buffer `rettv` takes over.
    unsafe {
        ga_append(&raw mut ga, NUL as uint8_t);
        (*rettv).vval.v_string = ga.ga_data as *mut c_char;
        (*rettv).v_type = VAR_STRING;
    }
}

/// `winrestview({dict})` — put back what `winsaveview()` saved.
///
/// Every key is optional: what the dictionary does not mention keeps its
/// current value.
pub unsafe fn f_winrestview(argvars: *mut typval_T, _rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the arguments are live typvals, and `curwin` is set.
    let (dict, mut win) = unsafe {
        if tv_check_for_nonnull_dict_arg(argvars, 0) == FAIL {
            return;
        }
        ((*argvars).vval.v_dict, Win::current())
    };
    let entry = |key: &CStr| {
        // SAFETY: a live dictionary, and `tv_dict_find` hands back a live
        // entry of it or NULL.
        unsafe {
            let di = tv_dict_find(dict, key.as_ptr(), key.count_bytes() as ptrdiff_t);
            (!di.is_null()).then(|| tv_get_number(&raw mut (*di).di_tv))
        }
    };

    if let Some(v) = entry(c"lnum") {
        win.w_cursor.lnum = v as linenr_T;
    }
    if let Some(v) = entry(c"col") {
        win.w_cursor.col = v as colnr_T;
    }
    if let Some(v) = entry(c"coladd") {
        win.w_cursor.coladd = v as colnr_T;
    }
    if let Some(v) = entry(c"curswant") {
        win.w_curswant = v as colnr_T;
        win.w_set_curswant = 0;
    }
    if let Some(v) = entry(c"topline") {
        // Not a plain assignment: 'scrolloff' and folds decide where the
        // window can actually start.
        // SAFETY: a live window.
        unsafe { set_topline(win.raw(), v as linenr_T) };
    }
    if let Some(v) = entry(c"topfill") {
        win.w_topfill = v as c_int;
    }
    if let Some(v) = entry(c"leftcol") {
        win.w_leftcol = v as colnr_T;
    }
    if let Some(v) = entry(c"skipcol") {
        win.w_skipcol = v as colnr_T;
    }

    // SAFETY: a live window, and `curbuf` is set.
    unsafe {
        check_cursor(win.raw());
        win_new_height(win.raw(), win.w_height);
        win_new_width(win.raw(), win.w_width);
        changed_window_setting(win.raw());
    }
    // A saved view from a buffer that has since shrunk can name a line that no
    // longer exists. The two tests are deliberately *not* a `clamp`: they are
    // applied in this order, so a buffer with no lines at all — which only an
    // unloaded one has — ends at 0, not at 1.
    if win.w_topline <= 0 {
        win.w_topline = 1;
    }
    // SAFETY: `curbuf` is set from startup to exit.
    let line_count = unsafe { Buf::current() }.line_count();
    if win.w_topline > line_count {
        win.w_topline = line_count;
    }
    // SAFETY: a live window.
    unsafe { check_topfill(win.raw(), true) };
}

/// `winsaveview()` — everything `winrestview()` puts back.
pub unsafe fn f_winsaveview(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `rettv` is the cleared return value and `curwin` is set; the
    // dictionary stays alive for the appends because `rettv` owns it.
    let (dict, win) = unsafe {
        tv_dict_alloc_ret(rettv);
        ((*rettv).vval.v_dict, Win::current())
    };
    let nr = |key: &CStr, value: varnumber_T| {
        // SAFETY: a live dictionary and a NUL-terminated key.
        unsafe { tv_dict_add_nr(dict, key.as_ptr(), key.count_bytes(), value) };
    };

    nr(c"lnum", varnumber_T::from(win.w_cursor.lnum));
    nr(c"col", varnumber_T::from(win.w_cursor.col));
    nr(c"coladd", varnumber_T::from(win.w_cursor.coladd));
    // 'curswant' is only up to date once the cursor move has been resolved.
    // SAFETY: `curwin` is set.
    unsafe { update_curswant() };
    nr(c"curswant", varnumber_T::from(win.w_curswant));
    nr(c"topline", varnumber_T::from(win.w_topline));
    nr(c"topfill", varnumber_T::from(win.w_topfill));
    nr(c"leftcol", varnumber_T::from(win.w_leftcol));
    nr(c"skipcol", varnumber_T::from(win.w_skipcol));
}
