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
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::guard::Depth;
use crate::message_fmt::msg_cstr;
use crate::tr;
use crate::winlayer::Buf;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use super::*;
use crate::ascii::ascii_iswhite;
use crate::cursor::{check_cursor, gchar_cursor};
use crate::ex_docmd::state::ex_normal_busy;
use crate::ex_docmd::{exec_normal_cmd, restore_current_state, save_current_state};
use crate::getchar::ins_typebuf;
use crate::normal::{VisualMode, set_visual_active, set_visual_anchor, set_visual_mode};
use crate::option::vars::p_sel;
use crate::pos::MAXCOL;
use crate::runtime::state::current_sctx;
use crate::state::mode::{State, VIsual_reselect, restart_edit};
use crate::state::{MODE_CMDLINE, MODE_INSERT, MODE_TERMINAL, MODE_VISUAL, get_real_state};
use crate::types::{Buffer, ColNr, ExArg, LineNr, Pos, SaveState, Window};
use crate::winlayer::Win;

/// The `:emenu` range, when there was one: `eap != NULL` and `addr_count`.
type Range = Option<(LineNr, LineNr)>;

/// Run `menu`'s right-hand side. Used by `:emenu`, `:popup` and the window
/// toolbar.
///
/// `mode_idx` names a `MENU_INDEX_*` value, or [`MENU_INDEX_INVALID`] to use
/// whichever mode the editor is in.
///
/// # Safety
/// `menu` must name a live node; `eap` must be null (the window toolbar) or
/// name a live `ExArg`.
pub(crate) unsafe fn execute_menu(args: *const ExArg, menu: *mut VimMenu, mode_idx: c_int) {
    // SAFETY: the caller's obligation. The range is copied out rather than
    // borrowed, because running the rhs re-enters the editor.
    let (menu, from_command, range) = unsafe {
        let menu = Menu::new(menu);
        match args.as_ref() {
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
    let bit = usize::try_from(idx).expect("a menu mode index is never negative");

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
        let mode = mode.to_string_lossy();
        semsg_name(tr!("E335: Menu not defined for {mode} mode"));
    }
}

/// Select the command's line range, the way `gv` would.
///
/// A range that matches the buffer's last Visual selection restores that
/// selection exactly -- upstream's own comment calls this "not perfect, but a
/// quick way of detecting whether we are doing this from a selection".
fn select_range(line1: LineNr, line2: LineNr) {
    let visual = with_curbuf(|buf| buf.b_visual);
    let end = if visual.vi_start.lnum == line1 && visual.vi_end.lnum == line2 {
        set_visual_mode(VisualMode::from_raw(visual.vi_mode));
        with_curwin(|win| {
            win.w_cursor = visual.vi_start;
            win.w_curswant = visual.vi_curswant;
        });
        visual.vi_end
    } else {
        // Line-wise over the range.
        set_visual_mode(VisualMode::LINE);
        with_curwin(|win| {
            win.w_cursor.lnum = line1;
            win.w_cursor.col = 1;
        });
        Pos {
            lnum: line2,
            col: MAXCOL as ColNr,
            coladd: 0,
        }
    };

    set_visual_active(true);
    VIsual_reselect.set(1);
    check_cursor_now();
    set_visual_anchor(with_curwin(|win| win.w_cursor));
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
    let mut state = SaveState::default();
    let _busy = Depth::of(&ex_normal_busy);
    // SAFETY: `state` is a live local for the whole call, and the rhs is a
    // NUL-terminated string owned by a node that outlives the run.
    unsafe {
        if save_current_state(&raw mut state) {
            exec_normal_cmd(menu.strings[bit], menu.noremap[bit], menu.silent[bit]);
        }
        restore_current_state(&raw mut state);
    }
}

/// Put the rhs into the typeahead, as if the user had typed it.
fn feed_typeahead(menu: Menu, bit: usize) {
    // SAFETY: the rhs is NUL-terminated and `ins_typebuf` copies it.
    let _ = unsafe {
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
        let path = msg_cstr(path_name);
        semsg_name(tr!("E334: Menu not found: {path}"));
    }
    menu
}

/// `:emenu` -- find the menu a descriptor like `File.New` names and run it.
///
/// # Safety
/// `args` must name the live `ExArg` of the command.
pub(crate) unsafe fn ex_emenu(args: *mut ExArg) {
    // SAFETY: the caller's obligation; `arg` names the command line.
    let arg = unsafe { CText::new((*args).arg) };

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
                let arg = msg_cstr(arg.as_cstr());
                semsg_name(tr!("E475: Invalid argument: {arg}"));
                return;
            }
        };
        arg = skip_white(arg.at(2));
    }

    let Some(menu) = menu_getbyname(arg.as_cstr()) else {
        return;
    };
    // SAFETY: a live node, and the command's own `ExArg`.
    unsafe { execute_menu(args, menu.raw(), mode_idx) };
}

/// Find the sub-menu `path_name` names -- what `:popup` and the window
/// toolbar want, as opposed to [`menu_getbyname`]'s item.
///
/// # Safety
/// `path_name` must name a NUL-terminated string.
pub(crate) unsafe fn menu_find(path_name: *const c_char) -> *mut VimMenu {
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

fn with_curwin<R>(f: impl FnOnce(&mut Window) -> R) -> R {
    // SAFETY: `curwin` always names a live window on the main thread.
    unsafe { f(&mut *Win::current_raw()) }
}

fn with_curbuf<R>(f: impl FnOnce(&Buffer) -> R) -> R {
    // SAFETY: `curbuf` always names a live buffer on the main thread.
    unsafe { f(&*Buf::current_raw()) }
}

/// The script id of whatever is running, 0 for the user's own typing.
fn script_id() -> c_int {
    current_sctx.get().sc_sid
}

fn real_state() -> c_int {
    get_real_state()
}

fn check_cursor_now() {
    check_cursor(Win::current());
}

fn char_at_cursor() -> c_int {
    // SAFETY: reads the current line at the cursor, which is in bounds.
    gchar_cursor()
}

/// `'selection'`'s first letter: `i`nclusive, `e`xclusive or `o`ld.
fn selection_style() -> u8 {
    // SAFETY: the option always holds a non-empty NUL-terminated string.
    unsafe { *p_sel.get() }.cast_unsigned()
}
