//! The dictionaries and lists that describe the layout: `getwininfo()`,
//! `gettabinfo()`, `winlayout()` and `win_gettype()`.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::types::{VAR_STRING, kListLenMayKnow, kListLenUnknown};

/// One `getwininfo()` entry.
///
/// # Safety
/// `wp` must be a live window whose buffer is live.
unsafe fn get_win_info(wp: Win, tpnr: c_int, winnr: c_int) -> *mut dict_T {
    // SAFETY: the caller's obligation. The dictionary is handed straight to
    // the caller's list, so it is not leaked, and it stays alive for every
    // entry the two closures add.
    let buf = wp.buffer();
    // "botline" is one past the last displayed line, hence the -1; the row
    // and column counts are zero-based inside and one-based to vimscript.
    validate_botline_win(wp);
    let (dict, textoff) = unsafe { (tv_dict_alloc(), win_col_off(wp.raw())) };
    let (quickfix, terminal) = (buf_is_quickfix(Some(buf)), buf_is_terminal(Some(buf)));
    let nr = |key: &CStr, value: varnumber_T| {
        // SAFETY: a live dictionary and a NUL-terminated key.
        let _ = unsafe { tv_dict_add_nr(dict, key.as_ptr(), key.count_bytes(), value) };
    };

    nr(c"tabnr", varnumber_T::from(tpnr));
    nr(c"winnr", varnumber_T::from(winnr));
    nr(c"winid", varnumber_T::from(wp.handle));
    nr(c"height", varnumber_T::from(wp.w_view_height));
    nr(c"status_height", varnumber_T::from(wp.w_status_height));
    nr(c"winrow", varnumber_T::from(wp.w_winrow + 1));
    nr(c"topline", varnumber_T::from(wp.w_topline));
    nr(c"botline", varnumber_T::from(wp.w_botline - 1));
    nr(c"leftcol", varnumber_T::from(wp.w_leftcol));
    nr(c"winbar", varnumber_T::from(wp.w_winbar_height));
    nr(c"width", varnumber_T::from(wp.w_view_width));
    nr(c"bufnr", varnumber_T::from(buf.handle));
    nr(c"wincol", varnumber_T::from(wp.w_wincol + 1));
    nr(c"textoff", varnumber_T::from(textoff));
    nr(c"terminal", varnumber_T::from(terminal));
    nr(c"quickfix", varnumber_T::from(quickfix));
    nr(
        c"loclist",
        varnumber_T::from(quickfix && !wp.w_llist_ref.is_null()),
    );
    // SAFETY: a live dictionary and the window's own variable dictionary.
    let vars = c"variables";
    let _ = unsafe { tv_dict_add_dict(dict, vars.as_ptr(), vars.count_bytes(), wp.w_vars) };
    dict
}

/// One `gettabinfo()` entry.
///
/// # Safety
/// `tp` must be a live tab page.
unsafe fn get_tabpage_info(tp: TabPage, tp_idx: c_int) -> *mut dict_T {
    // SAFETY: the caller's obligation; both containers are handed on rather
    // than freed here, so both stay alive for the appends below.
    // The keys go in in upstream's order: a dictionary's iteration order is
    // its hash table's, which insertion order can still perturb.
    let (nrkey, hint) = (c"tabnr", kListLenMayKnow as ptrdiff_t);
    let nr = varnumber_T::from(tp_idx);
    let dict = unsafe { tv_dict_alloc() };
    let _ = unsafe { tv_dict_add_nr(dict, nrkey.as_ptr(), nrkey.count_bytes(), nr) };
    let windows = unsafe { tv_list_alloc(hint) };
    let append = |handle: handle_T| {
        // SAFETY: a live list.
        unsafe { tv_list_append_number(windows, varnumber_T::from(handle)) };
    };
    for wp in windows_in_tab(tp) {
        append(wp.handle);
    }
    // SAFETY: a live dictionary, and the tab page's own variable dictionary.
    let (wins, vars) = (c"windows", c"variables");
    let _ = unsafe { tv_dict_add_list(dict, wins.as_ptr(), wins.count_bytes(), windows) };
    let _ = unsafe { tv_dict_add_dict(dict, vars.as_ptr(), vars.count_bytes(), tp.tp_vars) };
    dict
}

/// `gettabinfo([{tabnr}])` — every tab page, or just the one named.
pub unsafe fn f_gettabinfo(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals; the list belongs to
    // `rettv` for the whole walk.
    // The length hint is upstream's, and is the way round it looks: one entry
    // is expected when *no* tab page was named.
    let one = args.has(0);
    let hint = if one { kListLenMayKnow as ptrdiff_t } else { 1 };
    let list = unsafe { tv_list_alloc_ret(rettv, hint) };
    let wanted = if one {
        let n = number_as_int(arg_number_chk(args, 0));
        match unsafe { TabPage::from_raw(find_tabpage(n)) } {
            Some(tp) => Some(tp),
            None => return,
        }
    } else {
        None
    };
    for (tpnr, tp) in (1..).zip(tabs()) {
        if wanted.is_some_and(|want| want != tp) {
            continue;
        }
        // SAFETY: a live tab page, and a live list `rettv` owns.
        unsafe { tv_list_append_dict(list, get_tabpage_info(tp, tpnr)) };
        if wanted.is_some() {
            return;
        }
    }
}

/// `getwininfo([{winid}])` — every window of every tab page, or just the one
/// the id names.
///
/// The two counters are `int` here and `int16_t` upstream, which is the one
/// place this function knowingly differs: past 32,767 tab pages upstream's
/// `tabnr` wraps negative while `tabpagenr()`, an `int`, stays right. Reaching
/// that takes 33,000 `:tabnew`s, so no test can see either answer.
pub unsafe fn f_getwininfo(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals; the list belongs to
    // `rettv` for the whole walk.
    let list = unsafe { tv_list_alloc_ret(rettv, kListLenMayKnow as ptrdiff_t) };
    let wanted = if args.has(0) {
        match win_by_id(number_as_int(arg_number(args, 0))) {
            Some(wp) => Some(wp),
            None => return,
        }
    } else {
        None
    };
    for (tabnr, tp) in (1..).zip(tabs()) {
        // The window number counts up across the whole tab page even when
        // only one window is wanted, so an unnumbered float still shifts
        // nothing and a numbered one still gets the right ordinal.
        let mut winnr = 0;
        for wp in windows_in_tab(tp) {
            winnr += c_int::from(wp.has_winnr(tp));
            if wanted.is_some_and(|want| want != wp) {
                continue;
            }
            let numbered = if wp.has_winnr(tp) { winnr } else { 0 };
            // SAFETY: a live window in a live tab page, and a live list
            // `rettv` owns.
            unsafe { tv_list_append_dict(list, get_win_info(wp, tabnr, numbered)) };
            if wanted.is_some() {
                return;
            }
        }
    }
}

/// The layout of one frame, as `winlayout()` spells it: `["leaf", winid]`, or
/// `["row"|"col", [child, ...]]`.
///
/// # Safety
/// `l` must be a live list that outlives the call.
unsafe fn get_framelayout(fr: Frame, l: *mut list_T, outer: bool) {
    // SAFETY: the caller's obligation; every list built here is appended to
    // its parent before anything else can fail, so none is leaked.
    // The outer call writes into the caller's list; every nested one gets a
    // two-element list of its own.
    let fr_list = if outer {
        l
    } else {
        let nested = unsafe { tv_list_alloc(2) };
        unsafe { tv_list_append_list(l, nested) };
        nested
    };
    let word = |s: &CStr| {
        // SAFETY: a live list and a NUL-terminated string.
        unsafe { tv_list_append_string(fr_list, s.as_ptr(), s.count_bytes().cast_signed()) };
    };
    if c_int::from(fr.fr_layout) == FR_LEAF {
        // A leaf frame with no window is a frame being taken apart; it is
        // left out of the answer rather than described.
        if let Some(wp) = fr.win() {
            word(c"leaf");
            // SAFETY: a live list.
            unsafe { tv_list_append_number(fr_list, varnumber_T::from(wp.handle)) };
        }
        return;
    }
    word(if c_int::from(fr.fr_layout) == FR_ROW {
        c"row"
    } else {
        c"col"
    });
    // SAFETY: a live list, which the parent list takes over.
    let win_list = unsafe { tv_list_alloc(kListLenUnknown as ptrdiff_t) };
    unsafe { tv_list_append_list(fr_list, win_list) };
    for child in fr.children() {
        // SAFETY: a live child frame and the live list just built.
        unsafe { get_framelayout(child, win_list, false) };
    }
}

/// `winlayout([{tabnr}])` — the tab page's window layout tree.
pub unsafe fn f_winlayout(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals; the list belongs to
    // `rettv` for the whole walk.
    let list = unsafe { tv_list_alloc_ret(rettv, 2) };
    let tp = if !args.has(0) {
        cur_tab()
    } else {
        let n = number_as_int(arg_number(args, 0));
        match unsafe { TabPage::from_raw(find_tabpage(n)) } {
            Some(tp) => tp,
            None => return,
        }
    };
    unsafe { get_framelayout(tp.topframe(), list, true) };
}

/// `win_gettype([{nr}])` — the empty string for an ordinary window.
pub unsafe fn f_win_gettype(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_STRING;
    rettv.vval.v_string = ptr::null_mut();
    // SAFETY: the arguments are live typvals and `curwin` is set.
    let wp = if !args.has(0) {
        cur_win()
    } else {
        match arg_win(args, 0) {
            Some(wp) => wp,
            None => {
                rettv.vval.v_string = unsafe { xstrdup(c"unknown".as_ptr()) };
                return;
            }
        }
    };
    let kind = if is_aucmd_win(wp.raw()) {
        c"autocmd"
    } else if wp.w_onebuf_opt.wo_pvw != 0 {
        c"preview"
    } else if wp.w_floating {
        c"popup"
    } else if wp.raw() == cmdwin_win.get() {
        c"command"
    } else if buf_is_quickfix(wp.buffer_or_none()) {
        if wp.w_llist_ref.is_null() {
            c"quickfix"
        } else {
            c"loclist"
        }
    } else {
        return;
    };
    rettv.vval.v_string = unsafe { xstrdup(kind.as_ptr()) };
}

/// `getcmdwintype()` — the one-character type of the command-line window, or
/// the empty string when it is not open.
pub unsafe fn f_getcmdwintype(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `rettv` is the cleared return value; `xmallocz(1)` hands back
    // two writable bytes, the second already NUL.
    unsafe { (*rettv).v_type = VAR_STRING };
    let s = unsafe { xmallocz(1) }.cast::<c_char>();
    unsafe { *s = cmdwin_type.get().to_le_bytes()[0].cast_signed() };
    unsafe { (*rettv).vval.v_string = s };
}
