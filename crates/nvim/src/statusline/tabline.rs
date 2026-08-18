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

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_int};
use core::iter;

use super::*;
use crate::api::private::helpers::{arena_array, arena_dict, arena_string, cstr_as_string};
use crate::charset::{ptr2cells, vim_strsize};
use crate::grid::schar_from_ascii;
use crate::highlight_group::{HLF_T, HLF_TP, HLF_TPF, HLF_TPS};
use crate::main::{
    Columns, curbuf, curtab, curwin, default_grid, default_gridview, first_tabpage, firstbuf,
    firstwin, p_sc, p_sloc, p_tal, redraw_tabline, showcmd_buf, t_colors, tab_page_click_defs,
    tab_page_click_defs_size, topframe,
};
use crate::mbyte::utfc_ptr2len;
use crate::memory::{ARENA_EMPTY, arena_finish, arena_mem_free};
use crate::path::shorten_dir;
use crate::strings::vim_snprintf;
use crate::types::ui::kUITabline;
use crate::types::{
    Arena, Buffer, Object, StlClickDefinition_type_0, String_0, Tabpage, buf_T, tabpage_T, win_T,
};
use crate::ui::{ui_call_tabline_update, ui_has};
use crate::undo::bufIsChanged;
use crate::window::tabline_height;

/// The tab pages, in order. C spells this `FOR_ALL_TABS`.
///
/// # Safety
/// The tab page list must be live, which it is from startup to exit.
unsafe fn tabpages() -> impl Iterator<Item = *mut tabpage_T> {
    let first: *mut tabpage_T = first_tabpage.get().cast();
    iter::successors((!first.is_null()).then_some(first), |tp| {
        // SAFETY: the caller's promise -- a live list.
        let next = unsafe { (**tp).tp_next };
        (!next.is_null()).then_some(next)
    })
}

/// The buffers, in order. C spells this `FOR_ALL_BUFFERS`.
///
/// # Safety
/// The buffer list must be live, which it is from startup to exit.
unsafe fn buffers() -> impl Iterator<Item = *mut buf_T> {
    let first = firstbuf.get();
    iter::successors((!first.is_null()).then_some(first), |buf| {
        // SAFETY: the caller's promise -- a live list.
        let next = unsafe { (**buf).b_next };
        (!next.is_null()).then_some(next)
    })
}

/// The windows of tab page `tp`, in layout order.
///
/// # Safety
/// `tp` must be a live tab page.
unsafe fn windows_of(tp: *mut tabpage_T) -> impl Iterator<Item = Win> {
    // The current tab page's window list hangs off `firstwin`, not off the
    // tab page, which only records it when it is left.
    // SAFETY: the caller's promise.
    let first = if tp == curtab.get() {
        firstwin.get()
    } else {
        unsafe { (*tp).tp_firstwin }
    };
    // SAFETY: a live window list.
    iter::successors(unsafe { win_opt(first) }, |win| win.next())
}

/// The window whose buffer names tab page `tp`.
///
/// # Safety
/// `tp` must be a live tab page.
unsafe fn current_window_of(tp: *mut tabpage_T) -> Win {
    // SAFETY: the caller's promise; as [`windows_of`], the current tab
    // page's cursor window is `curwin`.
    unsafe {
        Win::new(if tp == curtab.get() {
            curwin.get()
        } else {
            (*tp).tp_curwin
        })
    }
}

/// Push the tab pages and the listed buffers to a UI that draws the tab line
/// itself.
///
/// # Safety
/// The editor's tab page and buffer lists must be live.
unsafe fn ui_ext_tabline_update() {
    let mut arena: Arena = ARENA_EMPTY;
    let arenap = &raw mut arena;

    // SAFETY: every list walk, handle read and name copy below is of the
    // editor's own live objects (the caller's promise); the arena outlives
    // every string put in it, and each container is sized by the same walk
    // that fills it.
    let mut tabs = arena_array(arenap, unsafe { tabpages() }.count());
    for tp in unsafe { tabpages() } {
        let mut info = arena_dict(arenap, 2);
        let (handle, cwp) = unsafe { ((*tp).handle as Tabpage, current_window_of(tp)) };
        put(&mut info, c"tab", Object::tabpage(handle));
        unsafe { get_trans_bufname(cwp.buffer().raw()) };
        put(
            &mut info,
            c"name",
            Object::string(unsafe { name_buff_in(arenap) }),
        );
        push(&mut tabs, Object::dict(info));
    }

    // Unlisted buffers are left out of the event. SAFETY: as above.
    let listed = || unsafe { buffers() }.filter(|buf| unsafe { (**buf).b_p_bl } != 0);
    let mut bufs = arena_array(arenap, listed().count());
    for buf in listed() {
        let mut info = arena_dict(arenap, 2);
        put(
            &mut info,
            c"buffer",
            Object::buffer(unsafe { (*buf).handle }),
        );
        unsafe { get_trans_bufname(buf) };
        put(
            &mut info,
            c"name",
            Object::string(unsafe { name_buff_in(arenap) }),
        );
        push(&mut bufs, Object::dict(info));
    }

    // SAFETY: as above; the arena is released once the event has been sent.
    unsafe {
        let (tab, buf) = (
            (*curtab.get()).handle as Tabpage,
            (*curbuf.get()).handle as Buffer,
        );
        ui_call_tabline_update(tab, tabs, buf, bufs);
        arena_mem_free(arena_finish(arenap));
    }
}

/// A copy of `NameBuff`'s contents in `arena`.
///
/// # Safety
/// `arena` must be live for as long as the copy is.
unsafe fn name_buff_in(arena: *mut Arena) -> String_0 {
    with_name_buff(|name| {
        // SAFETY: the caller's promise, and `NameBuff` is NUL-terminated by
        // whoever last filled it.
        unsafe { arena_string(arena, cstr_as_string(name.as_mut_ptr())) }
    })
}

/// Draw the tab pages line at the top of the editor.
///
/// # Safety
/// The editor must be up. `'tabline'` is a user format, so this re-enters.
pub unsafe fn draw_tabline() {
    // SAFETY: `default_grid` is live for the process's lifetime; before the
    // first resize it has no cells yet.
    if unsafe { (*default_grid.ptr()).chars.is_null() } {
        return;
    }
    redraw_tabline.set(false);

    if ui_has(kUITabline) {
        // SAFETY: the editor's own lists.
        unsafe { ui_ext_tabline_update() };
        return;
    }
    // SAFETY: reads the window layout.
    if unsafe { tabline_height() } < 1 {
        return;
    }

    // Clicking outside of any tab has no effect, so the whole line is
    // cleared first.
    debug_assert!(
        tab_page_click_defs_size.get() >= Columns.get() as size_t,
        "tab_page_click_defs_size >= (size_t)Columns"
    );
    tab_click_arena().clear();

    if !opt_is_empty(p_tal.get()) {
        // Use the 'tabline' option instead.
        // SAFETY: a null window means "the tab line"; this evaluates the
        // option.
        unsafe { win_redr_custom(ptr::null_mut::<win_T>(), false, false, false) };
    } else {
        // SAFETY: the editor's own lists.
        unsafe { draw_default_tabline() };
    }

    // Reset the flag again, in case evaluating 'tabline' set it.
    redraw_tabline.set(false);
}

/// The built-in tab line: one label per tab page.
///
/// # Safety
/// The editor's tab page and window lists must be live.
unsafe fn draw_default_tabline() {
    let attr_nosel = hl_attr(HLF_TP);
    let attr_fill = hl_attr(HLF_TPF);
    // Without colours the tabs are separated by `|` and underlined with `_`.
    let use_sep_chars = t_colors.get() < 8;

    // SAFETY: `default_gridview` is live; the batch is flushed below.
    unsafe { view_line_start(default_gridview.ptr(), 0) };
    // SAFETY: the caller's promise.
    let count = unsafe { tabpages() }.count() as c_int;
    let tabwidth = if count > 0 {
        (Columns.get() - 1 + count / 2) / count
    } else {
        0
    }
    .max(6);

    let mut attr = attr_nosel;
    let mut col = 0;
    let mut tabcount = 0;
    // SAFETY: the caller's promise.
    for tp in unsafe { tabpages() } {
        if col >= Columns.get() - 4 {
            break;
        }
        let scol = col;
        // SAFETY: a live tab page and its own frame pointer.
        let (cwp, current) =
            unsafe { (current_window_of(tp), (*tp).tp_topframe == topframe.get()) };
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
        // SAFETY: a live tab page.
        for win in unsafe { windows_of(tp) } {
            if !win.w_config.focusable || win.w_config.hide {
                wincount -= 1;
            // SAFETY: a live window's buffer.
            } else if unsafe { bufIsChanged(win.buffer().raw()) } {
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
            // SAFETY: a live window's buffer.
            unsafe { get_trans_bufname(cwp.buffer().raw()) };
            col += paint_bufname(col, room, attr);
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

    if p_sc.get() != 0 && c_int::from(opt_first(p_sloc.get())) == c_int::from(b't') {
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
    with_name_buff(|name| {
        let (out, room, fmt) = (name.as_mut_ptr(), MAXPATHL as size_t, c"%d".as_ptr());
        // SAFETY: `NameBuff` is `MAXPATHL` bytes and the format takes
        // exactly the one integer.
        let len = unsafe { vim_snprintf(out, room, fmt, wincount) };
        if col + len >= Columns.get() - 3 {
            return None;
        }
        paint_text(
            col,
            &name[..len as usize],
            combine_attr(attr, win_hl(cwp, HLF_T)),
        );
        Some(len)
    })
}

/// Draw the buffer name `get_trans_bufname()` left in `NameBuff`, shortened
/// to fit in `room` cells, answering how many cells it took.
///
/// The name is cut from the *front*: what matters about a path is its tail.
fn paint_bufname(col: c_int, room: c_int, attr: c_int) -> c_int {
    with_name_buff(|name| {
        // SAFETY: `NameBuff` holds a NUL-terminated path, and every walk
        // below stops at that terminator.
        let mut len = unsafe {
            shorten_dir(name.as_mut_ptr());
            vim_strsize(name.as_ptr())
        };
        let mut at = 0;
        while len > room {
            // SAFETY: `at` is a character boundary inside the path, and the
            // walk cannot pass the terminator: its width is zero.
            unsafe {
                len -= ptr2cells(name[at..].as_ptr());
                at += utfc_ptr2len(name[at..].as_ptr()) as usize;
            }
        }
        // SAFETY: `at` is a character boundary inside the path.
        paint_cstr(col, unsafe { CStr::from_ptr(name[at..].as_ptr()) }, attr);
        len.min(Columns.get() - col - 1)
    })
}

/// Draw the `'showcmd'` text at the right-hand end of the tab line.
fn paint_showcmd(col: c_int, tabcount: c_int, attr: c_int) {
    // Leave room for the close button when there is one.
    let room = Columns.get() - col - c_int::from(tabcount > 1) * 3;
    let width = 10.min(room);
    if width <= 0 {
        return;
    }
    showcmd_buf.with(|buf| {
        paint_text(
            Columns.get() - width - c_int::from(tabcount > 1) * 2,
            &buf[..width as usize],
            attr,
        );
    });
}

/// Claim `cols` of the tab page line for tab page `tabnr`.
fn set_tab_clicks(cols: core::ops::Range<c_int>, kind: StlClickDefinition_type_0, tabnr: c_int) {
    tab_click_arena().set(cols, kind, tabnr);
}

/// The tab page line's click definitions.
fn tab_click_arena() -> ClickArena {
    // SAFETY: the global arena and its recorded size, which the screen
    // resize holds at `Columns` or more.
    unsafe { ClickArena::new(tab_page_click_defs.get(), tab_page_click_defs_size.get()) }
}
