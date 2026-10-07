//! The dictionaries and lists that describe the layout: `getwininfo()`,
//! `gettabinfo()`, `winlayout()` and `win_gettype()`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::memory::ThinCString;
use crate::types::{VAR_STRING, kListLenMayKnow, kListLenUnknown};

/// One `getwininfo()` entry.
fn get_win_info(window: Win, tpnr: c_int, winnr: c_int) -> DictRef {
    let buf = window.buffer();
    // "botline" is one past the last displayed line, hence the -1; the row
    // and column counts are zero-based inside and one-based to vimscript.
    validate_botline_win(window);
    let (dict, textoff) = (tv_dict_alloc(), window.col_off());
    let (quickfix, terminal) = (buf_is_quickfix(Some(buf)), buf_is_terminal(Some(buf)));
    let nr = |key: &CStr, value: VarNumber| {
        let _ = dict.edit().add_number(key.to_bytes(), value);
    };

    nr(c"tabnr", VarNumber::from(tpnr));
    nr(c"winnr", VarNumber::from(winnr));
    nr(c"winid", VarNumber::from(window.handle));
    nr(c"height", VarNumber::from(window.w_view_height));
    nr(c"status_height", VarNumber::from(window.w_status_height));
    nr(c"winrow", VarNumber::from(window.w_winrow + 1));
    nr(c"topline", VarNumber::from(window.w_topline));
    nr(c"botline", VarNumber::from(window.w_botline - 1));
    nr(c"leftcol", VarNumber::from(window.w_leftcol));
    nr(c"winbar", VarNumber::from(window.w_winbar_height));
    nr(c"width", VarNumber::from(window.w_view_width));
    nr(c"bufnr", VarNumber::from(buf.handle));
    nr(c"wincol", VarNumber::from(window.w_wincol + 1));
    nr(c"textoff", VarNumber::from(textoff));
    nr(c"terminal", VarNumber::from(terminal));
    nr(c"quickfix", VarNumber::from(quickfix));
    nr(
        c"loclist",
        VarNumber::from(quickfix && !window.w_llist_ref.is_none()),
    );
    // The window's own `w:` scope; the answer takes a reference.
    let vars = window.w_winvar.di_tv.dict_handle();
    let _ = dict.edit().add_dict(b"variables", vars);
    dict
}

/// One `gettabinfo()` entry.
fn get_tabpage_info(tabpage: TabPage, tp_idx: c_int) -> DictRef {
    // The keys go in in upstream's order: a dictionary's iteration order is
    // its hash table's, which insertion order can still perturb.
    let dict = tv_dict_alloc();
    let _ = dict.edit().add_number(b"tabnr", VarNumber::from(tp_idx));
    let windows = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
    for wp in windows_in_tab(tabpage) {
        windows.edit().push_number(VarNumber::from(wp.handle));
    }
    let _ = dict.edit().add_list(b"windows", Some(windows));
    // The tab page's own `t:` scope; the answer takes a reference.
    let vars = tabpage.tp_winvar.di_tv.dict_handle();
    let _ = dict.edit().add_dict(b"variables", vars);
    dict
}

/// `gettabinfo([{tabnr}])` — every tab page, or just the one named.
pub fn f_gettabinfo(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // The length hint is upstream's, and is the way round it looks: one entry
    // is expected when *no* tab page was named.
    let one = !args.is_empty();
    let hint = if one { kListLenMayKnow as ptrdiff_t } else { 1 };
    let list = tv_list_alloc_ret(result, hint);
    let wanted = if one {
        let n = number_as_int(arg_number_chk(args, 0));
        match find_tabpage(n) {
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
        (*list).push_dict(Some(get_tabpage_info(tp, tpnr)));
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
pub fn f_getwininfo(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let list = tv_list_alloc_ret(result, kListLenMayKnow as ptrdiff_t);
    let wanted = if !args.is_empty() {
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
            (*list).push_dict(Some(get_win_info(wp, tabnr, numbered)));
            if wanted.is_some() {
                return;
            }
        }
    }
}

/// The layout of one frame, as `winlayout()` spells it, written into `into`:
/// `"leaf", winid`, or `"row"|"col", [child, ...]` where each child is a
/// two-element list of its own.
fn get_framelayout(fr: FrameRef, into: &mut List) {
    if c_int::from(fr.fr_layout) == FR_LEAF {
        // A leaf frame with no window is a frame being taken apart; it is
        // left out of the answer rather than described.
        if let Some(wp) = fr.win() {
            into.push_str(Some(c"leaf"));
            into.push_number(VarNumber::from(wp.handle));
        }
        return;
    }
    into.push_str(Some(if c_int::from(fr.fr_layout) == FR_ROW {
        c"row"
    } else {
        c"col"
    }));
    let children = tv_list_alloc(kListLenUnknown as ptrdiff_t);
    for child in fr.children() {
        let nested = tv_list_alloc(2);
        get_framelayout(child, nested.edit());
        children.edit().push_list(Some(nested));
    }
    into.push_list(Some(children));
}

/// `winlayout([{tabnr}])` — the tab page's window layout tree.
pub fn f_winlayout(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let list = tv_list_alloc_ret(result, 2);
    let tp = if args.is_empty() {
        TabPage::current()
    } else {
        let n = number_as_int(arg_number(args, 0));
        match find_tabpage(n) {
            Some(tp) => tp,
            None => return,
        }
    };
    get_framelayout(tp.topframe(), list);
}

/// `win_gettype([{nr}])` — the empty string for an ordinary window.
pub fn f_win_gettype(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_string(None);
    let wp = if args.is_empty() {
        Win::current()
    } else {
        match arg_win(args, 0) {
            Some(wp) => wp,
            None => {
                result.write_string(Some(ThinCString::from_cstr(c"unknown")));
                return;
            }
        }
    };
    let kind = if is_aucmd_win(wp) {
        c"autocmd"
    } else if wp.w_onebuf_opt.wo_pvw != 0 {
        c"preview"
    } else if wp.w_floating {
        c"popup"
    } else if cmdwin_win.get() == Some(wp.id()) {
        c"command"
    } else if buf_is_quickfix(wp.buffer_or_none()) {
        if wp.w_llist_ref.is_none() {
            c"quickfix"
        } else {
            c"loclist"
        }
    } else {
        return;
    };
    result.write_string(Some(ThinCString::from_cstr(kind)));
}

/// `getcmdwintype()` — the one-character type of the command-line window, or
/// the empty string when it is not open.
pub fn f_getcmdwintype(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_empty(VAR_STRING);
    let kind = cmdwin_type.get().to_le_bytes()[0];
    result.write_string(Some(ThinCString::from_bytes(&[kind])));
}
