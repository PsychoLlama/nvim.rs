//! Turning a vimscript argument into a window or a tab page, and the numbering
//! questions that go with it.
//!
//! Three shapes of argument reach here and every entry point below is one
//! combination of them: a **window id** (a `handle`, which is unique across
//! tab pages and starts at [`LOWEST_WIN_ID`]), a **window number** (the
//! `winnr()` ordinal within one tab page, which skips the floats
//! [`Win::has_winnr`] rejects), and a **tab page number** (the `tabpagenr()`
//! ordinal). Zero means "the current one" for a window number and "no tab page
//! given" for a tab page, except in `win_getid()`, which rejects it.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::eval::typval::NumBuf;
use crate::normal::visual_active;
use crate::types::{VAR_UNKNOWN, kListLenMayKnow};

/// Argument `i` as a Number.
///
/// Safe: [`Args`] carries the promise for the whole frame -- every index it
/// answers is a live typval -- which is `tv_get_number`'s only precondition.
/// The same goes for the two below it.
pub(crate) fn arg_number(args: Args<'_>, i: usize) -> varnumber_T {
    // SAFETY: `Args` promises a live typval at every index.
    unsafe { tv_get_number(args.ptr(i)) }
}

/// Argument `i` as a Number, answering 0 for a value that has none.
pub(crate) fn arg_number_chk(args: Args<'_>, i: usize) -> varnumber_T {
    // SAFETY: as [`arg_number`].
    unsafe { tv_get_number_chk(args.ptr(i), ptr::null_mut()) }
}

/// The window argument `i` names: an id in any tab page, or a number in the
/// current one.
pub(crate) fn arg_win(args: Args<'_>, i: usize) -> Option<Win> {
    // SAFETY: as [`arg_number`].
    unsafe { find_win_by_nr_or_id(args.ptr(i)) }
}

/// The window the editor is working in.
pub(crate) fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

/// The buffer the editor is working in.
pub(crate) fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The tab page the editor is working in.
pub(crate) fn cur_tab() -> TabPage {
    // SAFETY: `curtab` is set from startup to exit.
    unsafe { TabPage::current() }
}

/// The window with id `id`, in whichever tab page holds it.
pub fn win_by_id(id: c_int) -> Option<Win> {
    win_and_tab_by_id(id).map(|(wp, _)| wp)
}

/// The window with id `id`, and the tab page it lives in.
pub fn win_and_tab_by_id(id: c_int) -> Option<(Win, TabPage)> {
    tabs().find_map(|tp| {
        windows_in_tab(tp)
            .find(|wp| wp.handle == id)
            .map(|wp| (wp, tp))
    })
}

/// The window that the window *number* `vp` names within tab page `tp` —
/// `None` for the current tab page.
///
/// Number zero is the current window; a value at or above [`LOWEST_WIN_ID`] is
/// taken as an id instead, but only within `tp`.
///
/// # Safety
/// `vp` must point at a live typval.
pub unsafe fn find_win_by_nr(vp: *mut typval_T, tp: Option<TabPage>) -> Option<Win> {
    // SAFETY: the caller's obligation. A value that is not a number reports
    // and answers zero, which reads here as "the current window"; the
    // narrowing is upstream's and is what makes 0x1_0000_0000 read as 0.
    let nr = number_as_int(unsafe { tv_get_number_chk(vp, ptr::null_mut()) });
    if nr < 0 {
        return None;
    }
    if nr == 0 {
        // SAFETY: `curwin` is set from startup to exit.
        return Some(cur_win());
    }
    // SAFETY: `curtab` is set from startup to exit.
    let tp = tp.unwrap_or_else(cur_tab);
    if nr >= LOWEST_WIN_ID {
        return windows_in_tab(tp).find(|wp| wp.handle == nr);
    }
    // Window numbers count from one, and this counting does *not* skip the
    // windows `winnr()` has no number for.
    windows_in_tab(tp).nth(usize::try_from(nr).ok()?.checked_sub(1)?)
}

/// The window `vp` names: a window id in any tab page, or a window number in
/// the current one.
///
/// # Safety
/// `vp` must point at a live typval.
pub unsafe fn find_win_by_nr_or_id(vp: *mut typval_T) -> Option<Win> {
    // SAFETY: the caller's obligation.
    let nr = number_as_int(unsafe { tv_get_number_chk(vp, ptr::null_mut()) });
    if nr >= LOWEST_WIN_ID {
        // The second read is upstream's: `tv_get_number` where the test used
        // `tv_get_number_chk`, so a value that already reported does not
        // report twice.
        return win_by_id(number_as_int(unsafe { tv_get_number(vp) }));
    }
    // SAFETY: the caller's obligation.
    unsafe { find_win_by_nr(vp, None) }
}

/// The window `wvp` names within the tab page `tvp` names.
///
/// An absent window argument answers the current window and never looks at the
/// tab page; an absent tab page argument means the current one.
///
/// # Safety
/// `wvp` and `tvp` must point at live typvals.
pub unsafe fn find_tabwin(wvp: *mut typval_T, tvp: *mut typval_T) -> Option<Win> {
    // SAFETY: the caller's obligation.
    if unsafe { (*wvp).v_type } == VAR_UNKNOWN {
        return Some(cur_win());
    }
    let tp = if unsafe { (*tvp).v_type } == VAR_UNKNOWN {
        Some(cur_tab())
    } else {
        let n = number_as_int(unsafe { tv_get_number(tvp) });
        // A negative tab page number is refused outright; zero reaches
        // `find_tabpage`, which reads it as the current tab page.
        (n >= 0)
            .then(|| unsafe { TabPage::from_raw(find_tabpage(n)) })
            .flatten()
    };
    unsafe { find_win_by_nr(wvp, Some(tp?)) }
}

/// The tab page `nr` names, counting from one — the inverse of
/// [`crate::window::tabpage_index`].
fn tabpage_by_nr(nr: c_int) -> Option<TabPage> {
    tabs().nth(usize::try_from(nr).ok()?.checked_sub(1)?)
}

/// Common code for `tabpagewinnr()` and `winnr()`: the number of the window
/// `argvar` names within `tp`, or 0 when it names none.
///
/// # Safety
/// `argvar` must point at a live typval.
unsafe fn get_winnr(tp: TabPage, argvar: *mut typval_T) -> c_int {
    let mut numbuf = NumBuf::new();
    let mut twin = tp.curwin();
    if unsafe { (*argvar).v_type } == VAR_UNKNOWN {
        // Without an argument the answer is the current window's number,
        // which a float without one does not have.
        if !twin.has_winnr(tp) {
            return 0;
        }
    } else {
        // SAFETY: the caller's obligation; `endp` is a live local and
        // `tv_get_string_chk` hands back a NUL-terminated string or NULL.
        let arg = unsafe { numbuf.string_chk(argvar) };
        let resolved = match arg.is_null() {
            true => None,
            false => unsafe { relative_win(tp, twin, arg) },
        };
        match resolved {
            Some(wp) => twin = wp,
            None => return 0,
        }
    }
    // Count the numbered windows up to and including `twin`. A window that is
    // not in this tab page's list runs the walk off the end and answers 0.
    let mut nr = 0;
    for wp in windows_in_tab(tp) {
        nr += c_int::from(wp.has_winnr(tp));
        if wp == twin {
            return nr;
        }
    }
    0
}

/// The window the `winnr()` argument `arg` names, relative to `twin` in `tp`:
/// `"$"`, `"#"`, or a count followed by one of `hjkl`. `None` is the caller's
/// 0, and the invalid-expression error has already been reported.
///
/// # Safety
/// `arg` must be a NUL-terminated string.
unsafe fn relative_win(tp: TabPage, twin: Win, arg: *const c_char) -> Option<Win> {
    // SAFETY: the caller's obligation; `endp` is a live local that `strtol`
    // leaves pointing into `arg`.
    if unsafe { strcmp(arg, c"$".as_ptr()) } == 0 {
        return Some(tp.lastwin());
    }
    if unsafe { strcmp(arg, c"#".as_ptr()) } == 0 {
        return tp.prevwin();
    }
    let mut endp: *mut c_char = ptr::null_mut();
    let count = number_as_int(unsafe { strtol(arg, &raw mut endp, 10) }).max(1);
    let rest = (!endp.is_null() && c_int::from(unsafe { *endp }) != NUL)
        .then(|| unsafe { CStr::from_ptr(endp) });
    // "j"/"k" walk the layout tree vertically, "h"/"l" horizontally; `count`
    // says how many neighbours to step. The comparison is `strequal`'s, which
    // is a whole-string one.
    let (tpr, twr) = (tp.raw(), twin.raw());
    let direction = match rest.map(CStr::to_bytes) {
        // SAFETY: a live tab page and window.
        Some(b"j") => Some(unsafe { win_vert_neighbor(tpr, twr, false, count) }),
        Some(b"k") => Some(unsafe { win_vert_neighbor(tpr, twr, true, count) }),
        Some(b"h") => Some(unsafe { win_horz_neighbor(tpr, twr, true, count) }),
        Some(b"l") => Some(unsafe { win_horz_neighbor(tpr, twr, false, count) }),
        _ => None,
    };
    match direction {
        // The neighbour walks always answer a window, `twin` itself when
        // there is nothing that way.
        Some(wp) => Some(unsafe { Win::new(wp) }),
        None => {
            // SAFETY: the caller's obligation -- a NUL-terminated argument.
            let text = unsafe { CStr::from_ptr(arg) }.to_string_lossy();
            crate::semsg!("E15: Invalid expression: \"{}\"", text);
            None
        }
    }
}

/// `win_getid([{winnr} [, {tabnr}]])` — the id of a window named by number.
pub unsafe fn f_win_getid(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments are live typvals, and `curwin`/`curtab` are set.
    if !args.has(0) {
        rettv.vval.v_number = varnumber_T::from(cur_win().handle);
        return;
    }
    let winnr = number_as_int(arg_number(args, 0));
    if winnr <= 0 {
        rettv.vval.v_number = 0;
        return;
    }
    // A second argument names the tab page; without one, the current one.
    // This is not `find_tabpage()`, which answers the *current* tab page
    // for 0 where `win_getid()` has always rejected it.
    let tp = if !args.has(1) {
        cur_tab()
    } else {
        match tabpage_by_nr(number_as_int(arg_number(args, 1))) {
            Some(tp) => tp,
            None => {
                // Unlike every other failure here, a bad tab page
                // answers -1.
                rettv.vval.v_number = -1;
                return;
            }
        }
    };
    rettv.vval.v_number = match nth_numbered_win(tp, winnr) {
        Some(wp) => varnumber_T::from(wp.handle),
        None => 0,
    };
}

/// The `winnr`th window of `tp` that [`Win::has_winnr`] gives a number to.
fn nth_numbered_win(tp: TabPage, winnr: c_int) -> Option<Win> {
    let mut left = winnr;
    windows_in_tab(tp).find(|wp| {
        left -= c_int::from(wp.has_winnr(tp));
        left == 0
    })
}

/// `win_id2tabwin({winid})` — `[tabnr, winnr]`, `[0, 0]` for an unknown id.
pub unsafe fn f_win_id2tabwin(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals; the two counters are
    // live locals the callee writes only when it finds the window.
    let id: handle_T = number_as_int(arg_number(args, 0));
    let (mut winnr, mut tabnr) = (1, 1);
    unsafe { win_get_tabwin(id, &raw mut tabnr, &raw mut winnr) };
    let list = unsafe { tv_list_alloc_ret(rettv, 2) };
    unsafe { tv_list_append_number(list, varnumber_T::from(tabnr)) };
    unsafe { tv_list_append_number(list, varnumber_T::from(winnr)) };
}

/// `win_id2win({winid})` — the window's number in the current tab page, or 0.
pub unsafe fn f_win_id2win(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments are live typvals and `curtab` is set.
    let (tp, id) = unsafe { (cur_tab(), number_as_int(tv_get_number(args.ptr(0)))) };
    let mut nr = 0;
    for wp in windows_in_tab(tp) {
        if wp.handle == id {
            // A window the numbering skips (a hidden float) answers 0.
            rettv.vval.v_number = if wp.has_winnr(tp) {
                varnumber_T::from(nr + 1)
            } else {
                0
            };
            return;
        }
        nr += c_int::from(wp.has_winnr(tp));
    }
    rettv.vval.v_number = 0;
}

/// `win_findbuf({bufnr})` — the ids of every window showing that buffer.
pub unsafe fn f_win_findbuf(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals, and the list stays
    // alive for the appends because `rettv` owns it.
    let list = unsafe { tv_list_alloc_ret(rettv, kListLenMayKnow as ptrdiff_t) };
    let bufnr = number_as_int(arg_number(args, 0));
    for wp in tab_windows().filter(|wp| wp.buffer().handle == bufnr) {
        unsafe { tv_list_append_number(list, varnumber_T::from(wp.handle)) };
    }
}

/// `win_gotoid({winid})` — 1 when the window was reached, 0 otherwise.
pub unsafe fn f_win_gotoid(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments are live typvals and `curwin` is set.
    let id = unsafe { number_as_int(tv_get_number(args.ptr(0))) };
    // SAFETY: `curwin` is set from startup to exit.
    if cur_win().handle == id {
        rettv.vval.v_number = 1;
        return;
    }
    // SAFETY: the editor's own tab page and window lists.
    if unsafe { text_or_buf_locked() } {
        return;
    }
    let Some((wp, tp)) = win_and_tab_by_id(id) else {
        return;
    };
    // SAFETY: a live window in a live tab page.
    if visual_active() && wp.buffer().raw() != curbuf.get() {
        end_visual_mode();
    }
    unsafe { goto_tabpage_win(tp.raw(), wp.raw()) };
    rettv.vval.v_number = 1;
}

/// `winnr([{arg}])` — a window number in the current tab page.
pub unsafe fn f_winnr(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments are live typvals and `curtab` is set.
    let nr = unsafe { get_winnr(cur_tab(), args.ptr(0)) };
    rettv.vval.v_number = varnumber_T::from(nr);
}

/// `tabpagenr([{arg}])` — a tab page number.
pub unsafe fn f_tabpagenr(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments are live typvals; the tab page globals are set.
    let nr = if !args.has(0) {
        tabpage_index(curtab.get())
    } else {
        // SAFETY: the arguments are live typvals, and `tv_get_string_chk`
        // hands back a NUL-terminated string or NULL.
        let arg = unsafe { numbuf.string_chk(args.ptr(0)) };
        let word = (!arg.is_null()).then(|| unsafe { CStr::from_ptr(arg) });
        match word.map(CStr::to_bytes) {
            None => 0,
            // `tabpage_index(NULL)` counts one past the last tab page.
            Some(b"$") => tabpage_index(ptr::null_mut()) - 1,
            Some(b"#") => {
                let last = lastused_tabpage.get();
                match valid_tabpage(last) {
                    true => tabpage_index(last),
                    false => 0,
                }
            }
            Some(_) => {
                let text = word.unwrap_or(c"").to_string_lossy();
                crate::semsg!("E15: Invalid expression: \"{}\"", text);
                0
            }
        }
    };
    rettv.vval.v_number = varnumber_T::from(nr);
}

/// `tabpagewinnr({tabnr} [, {arg}])` — a window number in another tab page.
pub unsafe fn f_tabpagewinnr(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments are live typvals.
    // SAFETY: the arguments are live typvals.
    let n = number_as_int(arg_number(args, 0));
    let nr = match unsafe { TabPage::from_raw(find_tabpage(n)) } {
        Some(tp) => unsafe { get_winnr(tp, args.ptr(1)) },
        None => 0,
    };
    rettv.vval.v_number = varnumber_T::from(nr);
}

/// `winbufnr({nr})` — the buffer number of the window `nr` names, -1 for none.
pub unsafe fn f_winbufnr(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments are live typvals.
    let wp = arg_win(args, 0);
    rettv.vval.v_number = match wp {
        Some(wp) => varnumber_T::from(wp.buffer().handle),
        None => -1,
    };
}
