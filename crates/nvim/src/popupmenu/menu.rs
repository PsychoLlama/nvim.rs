//! The `:popup` terminal menu.
//!
//! [`pum_show_popupmenu`] runs its own key loop over a menu tree, drawing
//! through the same grid the completion menu uses and dispatching the
//! chosen entry. [`pum_select_mouse_pos`] maps a mouse position back to an
//! item for both this loop and the completion menu.
//!
//! This menu owns its items, unlike the completion menu, whose array belongs
//! to `insexpand`. They live in the two `Vec`s below for as long as the key
//! loop runs; a mapping run from `vgetc` can call `pum_undisplay`, which is
//! why the loop re-checks that the menu is still up on every key.
//!
//! The tree itself is read through [`Menu`], the menu family's own wrapper,
//! and the mouse position through [`MousePos`]: both walks are then safe
//! code and the only unsafe left is the editor entry points they call.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::keycodes::{Ctrl_C, Key};
use core::ptr;
use std::ffi::CString;

use super::*;
use crate::grid::default_grid_ref;
use crate::menu::{Menu, is_separator};
use crate::mouse::{MousePos, find_win_outer};
use crate::winlayer::Win;

/// The width `:popup` asks for even when the entries are narrower.
const PUM_POPUP_MIN_WIDTH: c_int = 20;

/// Point `pum_selected` at the item under the mouse, or clear it.
///
/// Three cases: the mouse is over the menu's own grid (the row is the item,
/// less the top border), it is over the grid the menu is anchored to (the row
/// is an offset from `pum_row`), or it is somewhere else entirely.
///
/// # Safety
/// The item array must be live and the placement settled.
pub(crate) unsafe fn pum_select_mouse_pos() {
    let mut pos = MousePos::current();
    if pos.grid == 0 {
        // Grid zero means "ask the compositor which window that was".
        find_win_outer(&mut pos);
    }

    if pos.grid == pum_grid_ref().handle {
        // On the menu itself. A box border (width 2) takes the top row.
        // SAFETY: reads the 'pumborder' option.
        let border_offset = c_int::from(unsafe { pum_border_width() } == 2);
        let item = pos.row - border_offset;
        pum_selected.set(if item >= 0 && item < pum_height.get() {
            item
        } else {
            -1
        });
        return;
    }

    if pos.grid != pum_anchor_grid.get()
        || pos.col < pum_left_col.get() - pum_win_col_offset.get()
        || pos.col >= pum_right_col.get() - pum_win_col_offset.get()
    {
        pum_selected.set(-1);
        return;
    }

    let idx = pos.row - (pum_row.get() - pum_win_row_offset.get());
    if idx < 0 || idx >= pum_height.get() {
        pum_selected.set(-1);
        return;
    }
    // SAFETY: the caller's promise -- a live item array of `pum_size`.
    let empty = unsafe { *pum_items()[idx as usize].pum_text == 0 };
    if !empty {
        // A separator has empty text and cannot be selected; the selection
        // stays where it was.
        pum_selected.set(idx);
    }
}

/// The entries of `menu` that are shown in `mode`, in the order the loop
/// numbers them.
///
/// A separator counts and takes a row; anything else has to be enabled in
/// `mode`. Both tests read the node's own flags, so this is the one walk the
/// numbering, the drawing and the dispatch all agree on.
fn shown_entries(menu: Menu, mode: c_int) -> impl Iterator<Item = Menu> {
    menu.children()
        .into_iter()
        .flat_map(Menu::siblings)
        .filter(move |mp| is_separator(mp.dname()) || mp.modes & mp.enabled & mode != 0)
}

/// Run the selected entry of `menu`.
///
/// # Safety
/// Running a right-hand side re-enters the editor, so nothing may be held
/// across this.
unsafe fn pum_execute_menu(menu: Menu, mode: c_int) {
    // A separator is not selectable, so only the enabled entries are
    // numbered here -- which is why this is not `shown_entries`.
    let enabled = menu
        .children()
        .into_iter()
        .flat_map(Menu::siblings)
        .filter(|mp| mp.modes & mp.enabled & mode != 0);
    for (idx, mp) in enabled.enumerate() {
        if idx as c_int == pum_selected.get() {
            let mut ea = exarg_T::default();
            // SAFETY: a live node and this frame's own `exarg_T`. The call
            // may redefine the tree, which is why the walk stops here.
            unsafe { execute_menu(&raw mut ea, mp.raw(), -1) };
            return;
        }
    }
}

/// The text of each entry `menu` shows in `mode`, as owned strings.
///
/// The text is copied because a callback can redefine the menu while the key
/// loop below is running.
fn pum_menu_entries(menu: Menu, mode: c_int) -> Vec<CString> {
    shown_entries(menu, mode)
        .map(|mp| {
            if is_separator(mp.dname()) {
                CString::default()
            } else {
                mp.dname().to_owned()
            }
        })
        .collect()
}

/// What one key of the `:popup` loop asked for.
enum MenuStep {
    /// Keep the menu up.
    Continue,
    /// Close it.
    Close,
    /// Run the selected entry, then close.
    Execute,
}

/// Handle one key of the `:popup` loop.
///
/// # Safety
/// The menu must be up; `items` must describe it.
unsafe fn pum_menu_key(c: c_int, items: &[CString]) -> MenuStep {
    match c {
        ESC | Ctrl_C => MenuStep::Close,
        CAR | NL => MenuStep::Execute,
        _ if c == 'k' as c_int || c == Key::Up.code() || c == Key::Mouseup.code() => {
            // Previous selectable item; separators are skipped over.
            while pum_selected.get() > 0 {
                pum_selected.set(pum_selected.get() - 1);
                if !items[pum_selected.get() as usize].as_bytes().is_empty() {
                    break;
                }
            }
            MenuStep::Continue
        }
        _ if c == 'j' as c_int || c == Key::Down.code() || c == Key::Mousedown.code() => {
            while pum_selected.get() < pum_size.get() - 1 {
                pum_selected.set(pum_selected.get() + 1);
                if !items[pum_selected.get() as usize].as_bytes().is_empty() {
                    break;
                }
            }
            MenuStep::Continue
        }
        _ if c == Key::Rightmouse.code() => {
            // Reposition the menu: hand the click back to the caller.
            vungetc(c);
            MenuStep::Close
        }
        _ if c == Key::Leftdrag.code()
            || c == Key::Rightdrag.code()
            || c == Key::Mousemove.code() =>
        {
            // SAFETY: the caller's promise -- the placement is settled.
            unsafe { pum_select_mouse_pos() };
            MenuStep::Continue
        }
        _ if c == Key::Leftmouse.code()
            || c == Key::LeftmouseNm.code()
            || c == Key::Rightrelease.code() =>
        {
            // A left click always closes; a right release only closes when
            // it landed on an item.
            // SAFETY: as above.
            unsafe { pum_select_mouse_pos() };
            if pum_selected.get() >= 0 {
                MenuStep::Execute
            } else if c == Key::Rightrelease.code() {
                MenuStep::Continue
            } else {
                MenuStep::Close
            }
        }
        _ => MenuStep::Continue,
    }
}

/// Show `menu` as a terminal popup and do not return until it is closed.
///
/// # Safety
/// `menu` must be live. This pumps the event loop, so nothing may be held
/// across it.
pub unsafe fn pum_show_popupmenu(menu: *mut vimmenu_T) {
    // SAFETY: the caller's promise.
    let menu = unsafe { Menu::new(menu) };
    // SAFETY: takes the completion menu down, if one was up.
    unsafe { pum_undisplay(true) };
    let mode = get_menu_mode_flag();
    let entries = pum_menu_entries(menu, mode);

    // "popup Edit" with only Terminal-mode entries lands here.
    pum_size.set(entries.len() as c_int);
    if entries.is_empty() {
        emsg(gettext(e_menu_only_exists_in_another_mode));
        return;
    }

    let mut array: Vec<pumitem_T> = entries
        .iter()
        .map(|text| pumitem_T {
            pum_text: text.as_ptr().cast_mut(),
            ..Default::default()
        })
        .collect();

    // SAFETY: `array` outlives `pum_array`, which the `pum_undisplay` at the
    // end clears; the placement calls read the editor's own state.
    pum_array.set(array.as_mut_ptr());
    unsafe { pum_compute_size() };
    pum_scrollbar.set(0);
    pum_height.set(pum_size.get());
    pum_rl.set(unsafe { Win::current() }.w_onebuf_opt.wo_rl != 0);
    unsafe { pum_position_at_mouse(PUM_POPUP_MIN_WIDTH) };

    pum_selected.set(-1);
    pum_first.set(0);
    if p_mousemev.get() == 0 {
        // Pretend 'mousemoveevent' is set so the menu can follow the
        // pointer, and put it back afterwards.
        set_mousemoveevent(true);
    }

    loop {
        pum_is_visible.set(true);
        pum_is_drawn.set(true);
        // Above the cmdline area: #23275.
        let mut grid = pum_grid_ref();
        grid.zindex = kZIndexCmdlinePopupMenu as c_int;
        // SAFETY: the menu is placed and its grid allocated; `vgetc` pumps
        // the event loop, so nothing is held across it.
        let c = unsafe {
            pum_redraw();
            setcursor_mayforce(curwin.get(), true);
            vgetc()
        };
        // A callback or <expr> mapping run from `vgetc` may have taken the
        // menu down under us.
        if pum_array.get().is_null() {
            break;
        }
        // SAFETY: the menu is still up and `entries` describes it.
        match unsafe { pum_menu_key(c, &entries) } {
            MenuStep::Continue => {}
            MenuStep::Close => break,
            MenuStep::Execute => {
                // SAFETY: a live node; running its rhs re-enters the editor.
                unsafe { pum_execute_menu(menu, mode) };
                break;
            }
        }
    }

    drop(array);
    // SAFETY: clears `pum_array` before `entries` goes out of scope.
    unsafe { pum_undisplay(true) };
    if p_mousemev.get() == 0 {
        set_mousemoveevent(false);
    }
}

/// Tell the UI whether to send mouse-move events.
fn set_mousemoveevent(on: bool) {
    ui_call_option_set(
        String_0::from_raw_parts(c"mousemoveevent".as_ptr().cast_mut(), 14),
        Object::Boolean(on),
    );
}

/// `:popup` -- show the menu at `path_name`, at the mouse or at the cursor.
///
/// # Safety
/// `path_name` must be NUL-terminated. Pumps the event loop.
pub unsafe fn pum_make_popup(path_name: *const c_char, use_mouse_pos: c_int) {
    if use_mouse_pos == 0 {
        // Put the mouse where the cursor is, so the menu pops up there.
        // SAFETY: `curwin` is live from startup to exit.
        let win = unsafe { Win::current() };
        mouse_row.set(win.w_grid.row_offset + win.w_wrow);
        mouse_col.set(
            win.w_grid.col_offset
                + if win.w_onebuf_opt.wo_rl != 0 {
                    win.w_view_width - win.w_wcol - 1
                } else {
                    win.w_wcol
                },
        );
        if ui_has(kUIMultigrid) {
            // Only under multigrid is the window's own grid guaranteed to
            // have been allocated -- headless it can still be null, which is
            // why this read stays inside the branch.
            // SAFETY: a live window's own grid.
            mouse_grid.set(unsafe { (*win.w_grid.target).handle } as c_int);
        } else if !ptr::eq(win.w_grid.target, default_grid_ref().raw()) {
            // Without multigrid the window's own grid is composed onto the
            // screen, so the position has to be screen-relative.
            mouse_grid.set(0);
            mouse_row.set(mouse_row.get() + win.w_winrow);
            mouse_col.set(mouse_col.get() + win.w_wincol);
        }
    }

    // SAFETY: the caller's promise; `menu_find` answers a live menu or null.
    let menu = unsafe { menu_find(path_name) };
    if !menu.is_null() {
        // SAFETY: a live menu; this pumps the event loop.
        unsafe { pum_show_popupmenu(menu) };
    }
}
