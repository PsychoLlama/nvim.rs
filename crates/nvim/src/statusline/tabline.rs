//! The tab line -- `draw_tabline()` and its `ext_tabline` form.
//!
//! [`draw_tabline`] draws the built-in tab line: one label per tab page,
//! each showing the window count, the modified marker and the shortened
//! buffer name of the tab's current window, sharing the width evenly, with
//! `'showcmd'` and the close button on the right. `'tabline'` replaces the
//! whole thing with a user format, which is [`win_redr_custom`]'s business,
//! and a UI that has taken the tab line over gets the same information as
//! data through [`ui_ext_tabline_update`].
//!
//! The tab-page click definitions are recorded as the labels are drawn --
//! one entry per screen cell, so that `jump_to_mouse()` can name the tab a
//! click landed on. Unlike a status line's, they carry no strings, so the
//! arena is only ever written here.
//!
//! Original: `src/nvim/statusline.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]

use crate::option::vars::{P_SLOC, P_TAL, p_sc};
use crate::winlayer::Buf;
use core::ffi::c_int;

use super::*;
use crate::charset::{ptr2cells_at, strsize};
use crate::drawscreen::state::redraw_tabline;
use crate::grid::{default_grid_ref, default_gridview, schar_from_ascii};
use crate::highlight_group::{HLF_T, HLF_TP, HLF_TPF, HLF_TPS};
use crate::mbyte::c_cluster_len;
use crate::normal::showcmd_buf;
use crate::path::shorten_dir;
use crate::statusline::state::tab_page_click_defs;
use crate::types::ui::kUITabline;
use crate::types::{BufferHandle, Object, StlClickDefinition_type_0, String_0, TabpageHandle};
use crate::ui::state::{Columns, t_colors};
use crate::ui::{ui_call_tabline_update, ui_has};
use crate::undo::buf_is_changed;
use crate::window::tabline_height;
use crate::winlayer::graph::topframe;
use crate::winlayer::{TabPage, buffers, tabs, windows_in_tab};

/// The window whose buffer names tab page `tabpage`.
fn current_window_of(tabpage: TabPage) -> Win {
    // As with [`windows_in_tab`], the current tab page's cursor window is the
    // `curwin` global rather than its own `tp_curwin`, which is only recorded
    // when the tab page is left.
    if tabpage.is_current() {
        Win::current()
    } else {
        tabpage
            .current_window()
            .expect("a live tab page has a cursor window")
    }
}

/// Push the tab pages and the listed buffers to a UI that draws the tab line
/// itself.
fn ui_ext_tabline_update() {
    // Each container is sized by the same walk that fills it.
    let mut tab_infos = Array::with_capacity(tabs().count());
    for tp in tabs() {
        let mut info = ApiDict::with_capacity(2);
        let (handle, cwp) = (tp.handle as TabpageHandle, current_window_of(tp));
        put(&mut info, c"tab", Object::tabpage(handle));
        let name = trans_bufname(cwp.buffer());
        put(
            &mut info,
            c"name",
            Object::string(String_0::from_bytes(&name)),
        );
        push(&mut tab_infos, Object::dict(info));
    }

    // Unlisted buffers are left out of the event.
    let listed = || buffers().filter(|buf| buf.b_p_bl != 0);
    let mut bufs = Array::with_capacity(listed().count());
    for buf in listed() {
        let mut info = ApiDict::with_capacity(2);
        put(&mut info, c"buffer", Object::buffer(buf.handle));
        let name = trans_bufname(buf);
        put(
            &mut info,
            c"name",
            Object::string(String_0::from_bytes(&name)),
        );
        push(&mut bufs, Object::dict(info));
    }

    let (tab, buf) = (
        TabPage::current().handle as TabpageHandle,
        Buf::current().handle as BufferHandle,
    );
    ui_call_tabline_update(tab, tab_infos, buf, bufs);
}

/// Draw the tab pages line at the top of the editor.
pub fn draw_tabline() {
    // Before the first resize the screen grid has no cells yet.
    if !default_grid_ref().is_allocated() {
        return;
    }
    redraw_tabline.set(false);

    if ui_has(kUITabline) {
        ui_ext_tabline_update();
        return;
    }
    if tabline_height() < 1 {
        return;
    }

    // Clicking outside of any tab has no effect, so the whole line is
    // cleared first.
    debug_assert!(
        tab_page_click_defs.with(Vec::len) >= Columns.get() as usize,
        "tab_page_click_defs_size >= (size_t)Columns"
    );
    with_tab_click_defs(|defs| clear_click_defs(defs));

    if P_TAL.first_byte() != 0 {
        // Use the 'tabline' option instead.
        win_redr_custom(None, false, false, false);
    } else {
        draw_default_tabline();
    }

    // Reset the flag again, in case evaluating 'tabline' set it.
    redraw_tabline.set(false);
}

/// The built-in tab line: one label per tab page.
fn draw_default_tabline() {
    let attr_nosel = hl_attr(HLF_TP);
    let attr_fill = hl_attr(HLF_TPF);
    // Without colours the tabs are separated by `|` and underlined with `_`.
    let use_sep_chars = t_colors.get() < 8;

    view_line_start(default_gridview(), 0);
    let count = tabs().count() as c_int;
    let tabwidth = if count > 0 {
        (Columns.get() - 1 + count / 2) / count
    } else {
        0
    }
    .max(6);

    let mut attr = attr_nosel;
    let mut col = 0;
    let mut tabcount = 0;
    for tp in tabs() {
        if col >= Columns.get() - 4 {
            break;
        }
        let scol = col;
        let (cwp, current) = (current_window_of(tp), tp.tp_topframe == topframe.get());
        if current {
            attr = win_hl(cwp, HLF_TPS);
        }
        if use_sep_chars && col > 0 {
            paint_schar(col, schar_from_ascii(b'|'), attr);
            col += 1;
        }
        if !current {
            attr = win_hl(cwp, HLF_TP);
        }
        paint_schar(col, schar_from_ascii(b' '), attr);
        col += 1;

        // How many windows the tab page holds, and whether any of their
        // buffers is modified. A window that is not focusable or is hidden
        // is not counted -- upstream spells that as a `wincount--` inside
        // the loop that increments it.
        let mut modified = false;
        let mut wincount = 0;
        for win in windows_in_tab(tp) {
            if !win.w_config.focusable || win.w_config.hide {
                wincount -= 1;
            } else if buf_is_changed(win.buffer()) {
                modified = true;
            }
            wincount += 1;
        }

        if modified || wincount > 1 {
            if wincount > 1 {
                let Some(len) = paint_wincount(col, wincount, attr, cwp) else {
                    break;
                };
                col += len;
            }
            if modified {
                paint_schar(col, schar_from_ascii(b'+'), attr);
                col += 1;
            }
            paint_schar(col, schar_from_ascii(b' '), attr);
            col += 1;
        }

        let room = scol - col + tabwidth - 1;
        if room > 0 {
            col += paint_bufname(col, room, attr, trans_bufname(cwp.buffer()));
        }
        paint_schar(col, schar_from_ascii(b' '), attr);
        col += 1;

        // Record the tab page number for every cell of the label, so that
        // `jump_to_mouse()` knows where each one is.
        tabcount += 1;
        set_tab_clicks(scol..col, kStlClickTabSwitch, tabcount);
    }

    // Past the last label, tab page zero: a double click there opens a new
    // tab after the last one and a single click goes to the next.
    set_tab_clicks(col..Columns.get(), kStlClickTabSwitch, 0);

    let fill = if use_sep_chars { b'_' } else { b' ' };
    paint_fill(col, Columns.get(), schar_from_ascii(fill), attr_fill);

    if p_sc() && c_int::from(P_SLOC.first_byte()) == c_int::from(b't') {
        paint_showcmd(col, tabcount, attr_nosel);
    }

    // An "X" to close the current tab, if there are several.
    if tabcount > 1 {
        paint_schar(Columns.get() - 1, schar_from_ascii(b'X'), attr_nosel);
        set_tab_clicks(Columns.get() - 1..Columns.get(), kStlClickTabClose, 999);
    }
    paint_flush();
}

/// Draw the window count of a tab page, answering how many cells it took --
/// or `None` when it does not fit, which ends the whole line.
fn paint_wincount(col: c_int, wincount: c_int, attr: c_int, cwp: Win) -> Option<c_int> {
    let name = wincount.to_string();
    let len = name.len() as c_int;
    if col + len >= Columns.get() - 3 {
        return None;
    }
    paint_text(col, name.as_bytes(), combine_attr(attr, win_hl(cwp, HLF_T)));
    Some(len)
}

/// Draw the buffer name `name`, its directories shortened and cut to fit in
/// `room` cells, answering how many cells it took.
///
/// The name is cut from the *front*: what matters about a path is its tail.
fn paint_bufname(col: c_int, room: c_int, attr: c_int, mut name: Vec<u8>) -> c_int {
    name.push(0);
    shorten_dir(&mut name);
    let name = crate::cstr::until_nul(&name);
    let mut len = strsize(name);
    let mut at = 0;
    // The walk cannot pass the end: what is left there is zero cells wide.
    while len > room {
        len -= ptr2cells_at(&name[at..]);
        at += c_cluster_len(&name[at..]);
    }
    paint_text(col, &name[at..], attr);
    len.min(Columns.get() - col - 1)
}

/// Draw the `'showcmd'` text at the right-hand end of the tab line.
fn paint_showcmd(col: c_int, tabcount: c_int, attr: c_int) {
    // Leave room for the close button when there is one.
    let room = Columns.get() - col - c_int::from(tabcount > 1) * 3;
    let width = 10.min(room);
    if width <= 0 {
        return;
    }
    let sc = showcmd_buf.get();
    paint_text(
        Columns.get() - width - c_int::from(tabcount > 1) * 2,
        crate::cstr::as_bytes(sc.cells(width as usize)),
        attr,
    );
}

/// Claim `cols` of the tab page line for tab page `tabnr`.
fn set_tab_clicks(cols: core::ops::Range<c_int>, kind: StlClickDefinition_type_0, tabnr: c_int) {
    with_tab_click_defs(|defs| set_click_defs(defs, cols, kind, tabnr));
}
