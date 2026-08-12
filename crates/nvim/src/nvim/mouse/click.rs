//! What a click on a statusline, tabline or winbar item does --
//! `%@Func@` click definitions and the popup menu.
//!
//! [`call_click_def_func`] turns a recorded click definition back into a call:
//! it rebuilds the `<LeftMouse>`-style modifier prefix, the click count and the
//! mouse position the handler is documented to receive, and either switches or
//! closes a tab page or calls the user's function.  [`do_popup`] is the
//! `'mousemodel'=popup` right-click path, and [`get_fpos_of_mouse`] the
//! position lookup both of them and the `v:mouse_*` variables share.
//!
//! Original: `src/nvim/mouse.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_char;

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::drawscreen::{
    UPD_INVERTED, UPD_VALID, redraw_curbuf_later, setcursor, update_screen,
};
use crate::src::nvim::eval::call_vim_function;
use crate::src::nvim::eval::typval::tv_clear;
use crate::src::nvim::keycodes::Ctrl_V;
use crate::src::nvim::main::{
    Rows, VIsual, VIsual_active, VIsual_mode, mod_mask, mouse_grid, mouse_row, p_ch,
};
use crate::src::nvim::menu::show_popupmenu;
use crate::src::nvim::pos::{lt, ltoreq};
use crate::src::nvim::types::{
    OptInt, VAR_FIXED, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, typval_T,
    typval_vval_union,
};
use crate::src::nvim::ui::ui_flush;

/// Call the click definition function recorded for column `col` in
/// `click_defs`, for button `which_button`.
pub fn call_click_def_func(click_defs: ClickDefs, col: c_int, which_button: c_int) {
    let def = click_defs.at(col);
    let mut modifiers = modifier_letters(mod_mask.get());
    let number = |v: varnumber_T| typval_T {
        v_type: VAR_NUMBER,
        v_lock: VAR_FIXED,
        vval: typval_vval_union { v_number: v },
    };
    let string = |v: *mut c_char| typval_T {
        v_type: VAR_STRING,
        v_lock: VAR_FIXED,
        vval: typval_vval_union { v_string: v },
    };
    let mut argv = [
        number(def.tabnr as varnumber_T),
        number(click_count(mod_mask.get())),
        string(button_name(which_button).as_ptr().cast_mut()),
        string(modifiers.as_mut_ptr()),
    ];
    let mut rettv = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };

    // SAFETY: `func` is the name the statusline parser recorded, the four
    // arguments are live for the call, and `rettv` is a live typval.
    unsafe {
        let argc = argv.len() as c_int;
        call_vim_function(def.func, argc, argv.as_mut_ptr(), &raw mut rettv);
        tv_clear(&raw mut rettv);
    }

    // Make sure next click does not register as drag when callback absorbs
    // the release event.
    got_click.set(false);
}

/// Translate window coordinates to a buffer position without any side effects.
///
/// Returns `IN_BUFFER` and sets `mpos.col` to the column when in buffer text.
/// The column is one for the first column.
pub fn get_fpos_of_mouse(mut mpos: Pos) -> c_int {
    let mut pos = MousePos::current();
    if pos.row < 0 || pos.col < 0 {
        return IN_UNKNOWN; // check if it makes sense
    }

    // Find the window where the row is in.
    let Some(win) = find_win_inner(&mut pos) else {
        return IN_UNKNOWN;
    };
    let (winrow, wincol) = (pos.row, pos.col);

    // Compute the position in the buffer line from the position on the screen.
    let (lnum, below_buffer) = comp_pos(win, &mut pos.row, &mut pos.col);
    mpos.lnum = lnum;

    if !below_buffer && !win.statuscolumn_empty() && win.in_statuscolumn(wincol) {
        return MOUSE_STATUSCOL;
    }

    // winpos and height may change in win_enter()!
    if winrow >= win.w_view_height + win.w_status_height {
        // Below the window; the global status line spans the whole screen.
        let below_screen = Rows.get() as OptInt - p_ch.get();
        if mouse_grid.get() <= 1
            && (mouse_row.get() as OptInt) < below_screen
            && mouse_row.get() as OptInt >= below_screen - global_stl_height() as OptInt
        {
            return IN_STATUS_LINE;
        }
        return IN_UNKNOWN;
    } else if winrow >= win.w_view_height {
        return IN_STATUS_LINE; // In window status line
    }

    if winrow < 0 && winrow + win.w_winbar_height >= 0 {
        return MOUSE_WINBAR; // In winbar
    }
    if wincol >= win.w_view_width {
        return IN_SEP_LINE; // In vertical separator line
    }
    if !win.is_current() || below_buffer {
        return IN_UNKNOWN;
    }

    (mpos.col, mpos.coladd) = vcol_to_col(win, mpos.lnum, pos.col);
    IN_BUFFER
}

/// Show the `'mousemodel'` popup menu, having first moved the cursor there if
/// the model asks for it and the click landed outside the selection.
pub fn do_popup(which_button: c_int, m_pos_flag: c_int, m_pos: pos_T) -> c_int {
    // First set the cursor position before showing the popup menu.
    let mut jump_flags = if mouse_model_popup_setpos() && leaves_selection(m_pos_flag, m_pos) {
        MOUSE_MAY_STOP_VIS
    } else {
        0
    };

    if jump_flags != 0 {
        // SAFETY: `inclusive` is allowed to be null.
        jump_flags = unsafe { jump_to_mouse(jump_flags, ptr::null_mut(), which_button) };
        let redraw = if VIsual_active.get() {
            UPD_INVERTED
        } else {
            UPD_VALID
        };
        // SAFETY: all four only touch the screen and the current buffer.
        unsafe {
            redraw_curbuf_later(redraw);
            update_screen();
            setcursor();
            ui_flush(); // Update before showing popup menu
        }
    }

    // SAFETY: runs its own modal loop over the menu tree.
    unsafe { show_popupmenu() };
    got_click.set(false); // ignore release events
    jump_flags
}

/// Whether the click at `m_pos` is outside the Visual selection or the current
/// window, so that showing the popup menu should end Visual mode.
///
/// Upstream notes that this "might have false negative here".
fn leaves_selection(m_pos_flag: c_int, mut m_pos: pos_T) -> bool {
    if !VIsual_active.get() {
        return true;
    }
    if m_pos_flag != IN_BUFFER {
        return true;
    }

    // SAFETY: `curwin` is live from startup to exit.
    let win = unsafe { Win::current() };
    let cursor = win.w_cursor;
    let visual = VIsual.get();

    if VIsual_mode.get() == 'V' as c_int {
        return (cursor.lnum <= visual.lnum
            && (m_pos.lnum < cursor.lnum || visual.lnum < m_pos.lnum))
            || (visual.lnum < cursor.lnum
                && (m_pos.lnum < visual.lnum || cursor.lnum < m_pos.lnum));
    }
    if (ltoreq(cursor, visual) && (lt(m_pos, cursor) || lt(visual, m_pos)))
        || (lt(visual, cursor) && (lt(m_pos, visual) || lt(cursor, m_pos)))
    {
        return true;
    }
    if VIsual_mode.get() == Ctrl_V {
        let (leftcol, rightcol) = vcols_between(win, cursor, visual);
        // The click's own virtual column, as the cursor would show it.
        // SAFETY: a live local position in the current buffer.
        m_pos.col = win.vcol_triple(unsafe { Pos::new(&raw mut m_pos) }).1;
        return m_pos.col < leftcol || m_pos.col > rightcol;
    }
    false
}
