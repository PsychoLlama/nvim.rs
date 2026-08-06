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

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::CStr;
use std::ffi::CString;

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::keycodes::Ctrl_C;

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
    // SAFETY: `mouse_find_win_outer` writes through the three locals.
    unsafe {
        let mut grid = mouse_grid.get();
        let mut row = mouse_row.get();
        let mut col = mouse_col.get();

        if grid == 0 {
            mouse_find_win_outer(&raw mut grid, &raw mut row, &raw mut col);
        }

        if grid == (*pum_grid.ptr()).handle {
            // On the menu itself. A box border (width 2) takes the top row.
            let border_offset = c_int::from(pum_border_width() == 2);
            let item = row - border_offset;
            pum_selected.set(if item >= 0 && item < pum_height.get() {
                item
            } else {
                -1
            });
            return;
        }

        if grid != pum_anchor_grid.get()
            || col < pum_left_col.get() - pum_win_col_offset.get()
            || col >= pum_right_col.get() - pum_win_col_offset.get()
        {
            pum_selected.set(-1);
            return;
        }

        let idx = row - (pum_row.get() - pum_win_row_offset.get());
        if idx < 0 || idx >= pum_height.get() {
            pum_selected.set(-1);
        } else if *pum_items()[idx as usize].pum_text != 0 {
            // A separator has empty text and cannot be selected; the
            // selection stays where it was.
            pum_selected.set(idx);
        }
    }
}

/// Run the selected entry of `menu`.
///
/// The menu tree is walked in the same order `pum_show_popupmenu` numbered it
/// in, so the n-th entry that is enabled in `mode` is the n-th row.
///
/// # Safety
/// `menu` must be live.
unsafe fn pum_execute_menu(menu: *mut vimmenu_T, mode: c_int) {
    // SAFETY: the menu tree is the editor's own; `execute_menu` may redefine
    // it, which is why the walk stops as soon as the entry is found.
    unsafe {
        let mut idx = 0;
        let mut mp = (*menu).children;
        while !mp.is_null() {
            if (*mp).modes & (*mp).enabled & mode != 0 {
                if idx == pum_selected.get() {
                    let mut ea = exarg_T::default();
                    execute_menu(&raw mut ea, mp, -1);
                    return;
                }
                idx += 1;
            }
            mp = (*mp).next;
        }
    }
}

/// The entries `menu` shows in `mode`, as owned strings.
///
/// A separator is an entry with empty text: it takes a row and cannot be
/// selected. The text is copied because a callback can redefine the menu
/// while the key loop below is running.
///
/// # Safety
/// `menu` must be live.
unsafe fn pum_menu_entries(menu: *mut vimmenu_T, mode: c_int) -> Vec<CString> {
    // SAFETY: the menu tree is the editor's own and `dname` is
    // NUL-terminated.
    unsafe {
        let mut entries = Vec::new();
        let mut mp = (*menu).children;
        while !mp.is_null() {
            if menu_is_separator((*mp).dname) {
                entries.push(CString::default());
            } else if (*mp).modes & (*mp).enabled & mode != 0 {
                entries.push(CStr::from_ptr((*mp).dname).to_owned());
            }
            mp = (*mp).next;
        }
        entries
    }
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
    // SAFETY: `pum_select_mouse_pos` reads the live placement.
    unsafe {
        match c {
            ESC | Ctrl_C => MenuStep::Close,
            CAR | NL => MenuStep::Execute,
            _ if c == 'k' as c_int || c == K_UP || c == K_MOUSEUP => {
                // Previous selectable item; separators are skipped over.
                while pum_selected.get() > 0 {
                    pum_selected.set(pum_selected.get() - 1);
                    if !items[pum_selected.get() as usize].as_bytes().is_empty() {
                        break;
                    }
                }
                MenuStep::Continue
            }
            _ if c == 'j' as c_int || c == K_DOWN || c == K_MOUSEDOWN => {
                while pum_selected.get() < pum_size.get() - 1 {
                    pum_selected.set(pum_selected.get() + 1);
                    if !items[pum_selected.get() as usize].as_bytes().is_empty() {
                        break;
                    }
                }
                MenuStep::Continue
            }
            _ if c == K_RIGHTMOUSE => {
                // Reposition the menu: hand the click back to the caller.
                vungetc(c);
                MenuStep::Close
            }
            _ if c == K_LEFTDRAG || c == K_RIGHTDRAG || c == K_MOUSEMOVE => {
                pum_select_mouse_pos();
                MenuStep::Continue
            }
            _ if c == K_LEFTMOUSE || c == K_LEFTMOUSE_NM || c == K_RIGHTRELEASE => {
                // A left click always closes; a right release only closes
                // when it landed on an item.
                pum_select_mouse_pos();
                if pum_selected.get() >= 0 {
                    MenuStep::Execute
                } else if c == K_RIGHTRELEASE {
                    MenuStep::Continue
                } else {
                    MenuStep::Close
                }
            }
            _ => MenuStep::Continue,
        }
    }
}

/// Show `menu` as a terminal popup and do not return until it is closed.
///
/// # Safety
/// `menu` must be live. This pumps the event loop, so nothing may be held
/// across it.
pub unsafe fn pum_show_popupmenu(menu: *mut vimmenu_T) {
    // SAFETY: `items` outlives `pum_array`, which is cleared by the
    // `pum_undisplay` at the end.
    unsafe {
        pum_undisplay(true);
        let mode = get_menu_mode_flag();
        let entries = pum_menu_entries(menu, mode);

        // "popup Edit" with only Terminal-mode entries lands here.
        pum_size.set(entries.len() as c_int);
        if entries.is_empty() {
            emsg(gettext(
                &raw const e_menu_only_exists_in_another_mode as *const c_char,
            ));
            return;
        }

        let mut array: Vec<pumitem_T> = entries
            .iter()
            .map(|text| pumitem_T {
                pum_text: text.as_ptr().cast_mut(),
                ..Default::default()
            })
            .collect();

        pum_array.set(array.as_mut_ptr());
        pum_compute_size();
        pum_scrollbar.set(0);
        pum_height.set(pum_size.get());
        pum_rl.set((*curwin.get()).w_onebuf_opt.wo_rl != 0);
        pum_position_at_mouse(PUM_POPUP_MIN_WIDTH);

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
            (*pum_grid.ptr()).zindex = kZIndexCmdlinePopupMenu as c_int;
            pum_redraw();
            setcursor_mayforce(curwin.get(), true);

            let c = vgetc();
            // A callback or <expr> mapping run from `vgetc` may have taken
            // the menu down under us.
            if pum_array.get().is_null() {
                break;
            }
            match pum_menu_key(c, &entries) {
                MenuStep::Continue => {}
                MenuStep::Close => break,
                MenuStep::Execute => {
                    pum_execute_menu(menu, mode);
                    break;
                }
            }
        }

        drop(array);
        pum_undisplay(true);
        if p_mousemev.get() == 0 {
            set_mousemoveevent(false);
        }
    }
}

/// Tell the UI whether to send mouse-move events.
fn set_mousemoveevent(on: bool) {
    ui_call_option_set(
        String_0 {
            data: c"mousemoveevent".as_ptr().cast_mut(),
            size: 14,
        },
        Object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed_12 { boolean: on },
        },
    );
}

/// `:popup` — show the menu at `path_name`, at the mouse or at the cursor.
///
/// # Safety
/// `path_name` must be NUL-terminated. Pumps the event loop.
pub unsafe fn pum_make_popup(path_name: *const c_char, use_mouse_pos: c_int) {
    // SAFETY: `curwin` is live and `menu_find` answers a live menu or null.
    unsafe {
        if use_mouse_pos == 0 {
            // Put the mouse where the cursor is, so the menu pops up there.
            let win = curwin.get();
            mouse_row.set((*win).w_grid.row_offset + (*win).w_wrow);
            mouse_col.set(
                (*win).w_grid.col_offset
                    + if (*win).w_onebuf_opt.wo_rl != 0 {
                        (*win).w_view_width - (*win).w_wcol - 1
                    } else {
                        (*win).w_wcol
                    },
            );
            if ui_has(kUIMultigrid) {
                mouse_grid.set((*(*win).w_grid.target).handle as c_int);
            } else if (*win).w_grid.target != default_grid.ptr() {
                mouse_grid.set(0);
                mouse_row.set(mouse_row.get() + (*win).w_winrow);
                mouse_col.set(mouse_col.get() + (*win).w_wincol);
            }
        }

        let menu = menu_find(path_name);
        if !menu.is_null() {
            pum_show_popupmenu(menu);
        }
    }
}
