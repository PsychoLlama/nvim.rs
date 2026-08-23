//! Running a menu entry -- `:emenu`, `:popup` and the tooltip lookup.
//!
//! [`execute_menu`] decides which mode's right-hand side to use: the mode the
//! editor is really in, unless the command named one, and with a special case
//! for a menu invoked from a script. The rhs is then fed back into the
//! typeahead as if the user had typed it. [`ex_emenu`] parses the command's
//! argument, [`menu_getbyname`] and [`menu_find`] resolve a path for it and
//! for `:popup`.
//!
//! Original: `src/nvim/menu.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use super::*;
use crate::ascii::ascii_iswhite;
use crate::cursor::{check_cursor, gchar_cursor};
use crate::ex_docmd::{exec_normal_cmd, restore_current_state, save_current_state};
use crate::getchar::ins_typebuf;
use crate::main::{
    State, VIsual, VIsual_active, VIsual_mode, VIsual_reselect, curbuf, current_sctx, curwin,
    e_invarg2, ex_normal_busy, p_sel, restart_edit,
};
use crate::pos::MAXCOL;
use crate::state::{MODE_CMDLINE, MODE_INSERT, MODE_TERMINAL, MODE_VISUAL, get_real_state};
use crate::types::{buf_T, colnr_T, exarg_T, linenr_T, pos_T, save_state_T, win_T};

/// The `:emenu` range, when there was one: `eap != NULL` and `addr_count`.
type Range = Option<(linenr_T, linenr_T)>;

/// Run `menu`'s right-hand side. Used by `:emenu`, `:popup` and the window
/// toolbar.
///
/// `mode_idx` names a `MENU_INDEX_*` value, or [`MENU_INDEX_INVALID`] to use
/// whichever mode the editor is in.
///
/// # Safety
/// `menu` must name a live node; `eap` must be null (the window toolbar) or
/// name a live `exarg_T`.
pub(crate) unsafe fn execute_menu(eap: *const exarg_T, menu: *mut vimmenu_T, mode_idx: c_int) {
    // SAFETY: the caller's obligation. The range is copied out rather than
    // borrowed, because running the rhs re-enters the editor.
    let (menu, from_command, range) = unsafe {
        let menu = Menu::new(menu);
        match eap.as_ref() {
            None => (menu, false, None),
            Some(eap) => (
                menu,
                true,
                (eap.addr_count != 0).then_some((eap.line1, eap.line2)),
            ),
        }
    };
    run_menu(menu, mode_idx, from_command, range);
}

fn run_menu(menu: Menu, mode_idx: c_int, from_command: bool, range: Range) {
    let mut idx = mode_idx;
    if idx < 0 {
        idx = if State.get() & MODE_TERMINAL != 0 {
            MENU_INDEX_TERMINAL
        } else if State.get() & MODE_CMDLINE != 0 {
            MENU_INDEX_CMDLINE
        } else if real_state() & MODE_VISUAL != 0 {
            // Really in Visual mode: no guessing needed, the selection is
            // whatever is selected.
            MENU_INDEX_VISUAL
        } else if (State.get() & MODE_INSERT != 0 || restart_edit.get() != 0) && script_id() == 0 {
            // Use the Insert mode entry when returning to Insert mode. A
            // non-zero script id means this came through a script or an API
            // call, which is not "returning" to anything.
            MENU_INDEX_INSERT
        } else if let Some((line1, line2)) = range {
            select_range(line1, line2);
            MENU_INDEX_VISUAL
        } else {
            MENU_INDEX_INVALID
        };
    }
    if idx == MENU_INDEX_INVALID || !from_command {
        idx = MENU_INDEX_NORMAL;
    }
    let bit = idx as usize;

    if !menu.strings[bit].is_null() && menu.modes & (1 << idx) != 0 {
        if !from_command || script_id() != 0 {
            // Executing a script or a function, or the window toolbar: run
            // the commands right now.
            run_now(menu, bit);
        } else {
            feed_typeahead(menu, bit);
        }
    } else if from_command {
        let mode = match idx {
            MENU_INDEX_VISUAL => c"Visual",
            MENU_INDEX_SELECT => c"Select",
            MENU_INDEX_OP_PENDING => c"Op-pending",
            MENU_INDEX_TERMINAL => c"Terminal",
            MENU_INDEX_INSERT => c"Insert",
            MENU_INDEX_CMDLINE => c"Cmdline",
            // MENU_INDEX_TIP cannot happen.
            _ => c"Normal",
        };
        semsg_name(
            message_str(c"E335: Menu not defined for %s mode"),
            mode.as_ptr(),
        );
    }
}

/// Select the command's line range, the way `gv` would.
///
/// A range that matches the buffer's last Visual selection restores that
/// selection exactly -- upstream's own comment calls this "not perfect, but a
/// quick way of detecting whether we are doing this from a selection".
fn select_range(line1: linenr_T, line2: linenr_T) {
    let visual = with_curbuf(|buf| buf.b_visual);
    let end = if visual.vi_start.lnum == line1 && visual.vi_end.lnum == line2 {
        VIsual_mode.set(visual.vi_mode);
        with_curwin(|win| {
            win.w_cursor = visual.vi_start;
            win.w_curswant = visual.vi_curswant;
        });
        visual.vi_end
    } else {
        // Line-wise over the range.
        VIsual_mode.set(c_int::from(b'V'));
        with_curwin(|win| {
            win.w_cursor.lnum = line1;
            win.w_cursor.col = 1;
        });
        pos_T {
            lnum: line2,
            col: MAXCOL as colnr_T,
            coladd: 0,
        }
    };

    VIsual_active.set(true);
    VIsual_reselect.set(1);
    check_cursor_now();
    VIsual.set(with_curwin(|win| win.w_cursor));
    with_curwin(|win| win.w_cursor = end);
    check_cursor_now();

    // With an exclusive selection the cursor sits one past the last
    // selected character.
    if selection_style() == b'e' && char_at_cursor() != 0 {
        with_curwin(|win| win.w_cursor.col += 1);
    }
}

/// Run the rhs immediately, inside a saved editor state.
fn run_now(menu: Menu, bit: usize) {
    // SAFETY: `save_state_T` is a plain aggregate of scalars, pointers and
    // buffers, all valid all-zero, and `save_current_state` fills it before
    // anything reads it -- the same shape `ex_docmd` uses.
    let mut state: save_state_T = unsafe { core::mem::zeroed() };
    ex_normal_busy_adjust(1);
    // SAFETY: `state` is a live local for the whole call, and the rhs is a
    // NUL-terminated string owned by a node that outlives the run.
    unsafe {
        if save_current_state(&raw mut state) {
            exec_normal_cmd(menu.strings[bit], menu.noremap[bit], menu.silent[bit]);
        }
        restore_current_state(&raw mut state);
    }
    ex_normal_busy_adjust(-1);
}

/// Put the rhs into the typeahead, as if the user had typed it.
fn feed_typeahead(menu: Menu, bit: usize) {
    // SAFETY: the rhs is NUL-terminated and `ins_typebuf` copies it.
    unsafe {
        ins_typebuf(
            menu.strings[bit],
            menu.noremap[bit],
            0,
            true,
            menu.silent[bit],
        )
    };
}

/// Find the node `path_name` names, which must be a menu *item*.
fn menu_getbyname(path_name: &CStr) -> Option<Menu> {
    let mut buf = scratch(path_name);
    let mut name = text_of(&mut buf);
    let mut menu = root_first();
    let mut reported = false;

    while !name.is_empty() {
        let rest = skip_component(name);
        let mut matched = None;
        for node in menu.into_iter().flat_map(Menu::siblings) {
            if !name_equal(name.as_cstr(), node) {
                continue;
            }
            if rest.is_empty() && node.children().is_some() {
                emsg_c(c"E333: Menu path must lead to a menu item");
                reported = true;
            } else if !rest.is_empty() && node.children().is_none() {
                emsg_c(E_NOTSUBMENU);
            } else {
                matched = Some(node);
            }
            break;
        }
        menu = matched;
        let Some(node) = matched.filter(|_| !rest.is_empty()) else {
            break;
        };
        menu = node.children();
        name = rest;
    }

    if menu.is_none() && !reported {
        semsg_name(message_str(c"E334: Menu not found: %s"), path_name.as_ptr());
    }
    menu
}

/// `:emenu` -- find the menu a descriptor like `File.New` names and run it.
///
/// # Safety
/// `eap` must name the live `exarg_T` of the command.
pub(crate) unsafe fn ex_emenu(eap: *mut exarg_T) {
    // SAFETY: the caller's obligation; `arg` names the command line.
    let arg = unsafe { CText::new((*eap).arg) };

    // An optional leading mode letter, e.g. ":emenu i File.New".
    let mut mode_idx = MENU_INDEX_INVALID;
    let mut arg = arg;
    if arg.byte(0) != 0 && ascii_iswhite(c_int::from(arg.byte(1))) {
        mode_idx = match arg.byte(0) {
            b'n' => MENU_INDEX_NORMAL,
            b'v' => MENU_INDEX_VISUAL,
            b's' => MENU_INDEX_SELECT,
            b'o' => MENU_INDEX_OP_PENDING,
            b't' => MENU_INDEX_TERMINAL,
            b'i' => MENU_INDEX_INSERT,
            b'c' => MENU_INDEX_CMDLINE,
            _ => {
                semsg_name(message(&e_invarg2), arg.raw());
                return;
            }
        };
        arg = skip_white(arg.at(2));
    }

    let Some(menu) = menu_getbyname(arg.as_cstr()) else {
        return;
    };
    // SAFETY: a live node, and the command's own `exarg_T`.
    unsafe { execute_menu(eap, menu.raw(), mode_idx) };
}

/// Find the sub-menu `path_name` names -- what `:popup` and the window
/// toolbar want, as opposed to [`menu_getbyname`]'s item.
///
/// # Safety
/// `path_name` must name a NUL-terminated string.
pub(crate) unsafe fn menu_find(path_name: *const c_char) -> *mut vimmenu_T {
    // SAFETY: the caller's obligation.
    let path = unsafe { CStr::from_ptr(path_name) };
    let mut buf = scratch(path);
    let mut name = text_of(&mut buf);
    let mut menu = root_first();

    while !name.is_empty() {
        let rest = skip_component(name);
        let mut matched = None;
        for node in menu.into_iter().flat_map(Menu::siblings) {
            if !name_equal(name.as_cstr(), node) {
                continue;
            }
            if node.children().is_none() {
                // A menu item where a sub-menu was wanted.
                emsg_c(if rest.is_empty() {
                    c"E336: Menu path must lead to a sub-menu"
                } else {
                    E_NOTSUBMENU
                });
                return ptr::null_mut();
            }
            if rest.is_empty() {
                return node.raw();
            }
            matched = Some(node);
            break;
        }
        let Some(node) = matched else {
            menu = None;
            break;
        };
        menu = node.children();
        name = rest;
    }

    if menu.is_none() {
        emsg_c(c"E337: Menu not found - check menu names");
    }
    menu.map_or(ptr::null_mut(), Menu::raw)
}

// The editor state this module reads and writes. Each hands out a reference
// for exactly one statement, so none can span the rhs being run.

fn with_curwin<R>(f: impl FnOnce(&mut win_T) -> R) -> R {
    // SAFETY: `curwin` always names a live window on the main thread.
    unsafe { f(&mut *curwin.get()) }
}

fn with_curbuf<R>(f: impl FnOnce(&buf_T) -> R) -> R {
    // SAFETY: `curbuf` always names a live buffer on the main thread.
    unsafe { f(&*curbuf.get()) }
}

/// The script id of whatever is running, 0 for the user's own typing.
fn script_id() -> c_int {
    // SAFETY: a live global whose first field is read here and nowhere else.
    unsafe { (*current_sctx.ptr()).sc_sid }
}

fn real_state() -> c_int {
    get_real_state()
}

fn check_cursor_now() {
    // SAFETY: `curwin` names a live window.
    unsafe { check_cursor(curwin.get()) };
}

fn char_at_cursor() -> c_int {
    // SAFETY: reads the current line at the cursor, which is in bounds.
    unsafe { gchar_cursor() }
}

/// `'selection'`'s first letter: `i`nclusive, `e`xclusive or `o`ld.
fn selection_style() -> u8 {
    // SAFETY: the option always holds a non-empty NUL-terminated string.
    unsafe { *p_sel.get() as u8 }
}

fn ex_normal_busy_adjust(by: c_int) {
    // SAFETY: a plain counter behind the escape hatch.
    unsafe { *ex_normal_busy.ptr() += by };
}
