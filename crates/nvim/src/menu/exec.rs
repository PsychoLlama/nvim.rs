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

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::guard::Depth;
use crate::message_fmt::msg_bytes;
use crate::option::vars::P_SEL;
use crate::tr;
use core::ffi::{CStr, c_int};
use std::rc::Rc;

use super::*;
use crate::cursor::{check_cursor, gchar_cursor};
use crate::ex_docmd::state::ex_normal_busy;
use crate::ex_docmd::{exec_normal_cmd, restore_current_state, save_current_state};
use crate::getchar::ins_typebuf;
use crate::normal::{VisualMode, set_visual_active, set_visual_anchor, set_visual_mode};
use crate::pos::MAXCOL;
use crate::runtime::state::current_sctx;
use crate::state::mode::{State, VIsual_reselect, restart_edit};
use crate::state::{MODE_CMDLINE, MODE_INSERT, MODE_TERMINAL, MODE_VISUAL, get_real_state};
use crate::types::{Buffer, ColNr, ExArg, LineNr, Pos, SaveState};
use crate::winlayer::Win;

/// The `:emenu` range, when there was one: `addr_count` and the two lines.
pub(crate) type Range = Option<(LineNr, LineNr)>;

/// The range `excmd` gave, if any.
pub(crate) fn range_of(excmd: &ExArg) -> Range {
    (excmd.addr_count != 0).then_some((excmd.line1, excmd.line2))
}

/// Run the right-hand side of the menu item `menu`. Used by `:emenu` and
/// `:popup`.
///
/// `mode_idx` names a `MENU_INDEX_*` value, or [`MENU_INDEX_INVALID`] to use
/// whichever mode the editor is in. Nothing happens when the item has been
/// deleted in the meantime.
pub(crate) fn execute_menu(range: Range, menu: MenuId, mode_idx: c_int) {
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
    if idx == MENU_INDEX_INVALID {
        idx = MENU_INDEX_NORMAL;
    }
    let bit = usize::try_from(idx).expect("a menu mode index is never negative");

    // A copy of what is run: the keys may redefine or delete the menu.
    let rhs = with_menu(menu, |node| {
        let rhs = node.strings[bit]
            .clone()
            .filter(|_| node.modes & (1 << idx) != 0)?;
        Some((rhs, node.noremap[bit], node.silent[bit]))
    })
    .flatten();
    if let Some((rhs, noremap, silent)) = rhs {
        if script_id() != 0 {
            // Executing a script or a function: run the commands right now.
            run_now(&rhs, noremap, silent);
        } else {
            feed_typeahead(&rhs, noremap, silent);
        }
    } else {
        let mode = match idx {
            MENU_INDEX_VISUAL => "Visual",
            MENU_INDEX_SELECT => "Select",
            MENU_INDEX_OP_PENDING => "Op-pending",
            MENU_INDEX_TERMINAL => "Terminal",
            MENU_INDEX_INSERT => "Insert",
            MENU_INDEX_CMDLINE => "Cmdline",
            // MENU_INDEX_TIP cannot happen.
            _ => "Normal",
        };
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
        let mut win = Win::current();
        win.w_cursor = visual.vi_start;
        win.w_curswant = visual.vi_curswant;
        visual.vi_end
    } else {
        // Line-wise over the range.
        set_visual_mode(VisualMode::LINE);
        let mut win = Win::current();
        win.w_cursor.lnum = line1;
        win.w_cursor.col = 1;
        Pos {
            lnum: line2,
            col: MAXCOL as ColNr,
            coladd: 0,
        }
    };

    set_visual_active(true);
    VIsual_reselect.set(1);
    check_cursor_now();
    set_visual_anchor(Win::current().w_cursor);
    Win::current().w_cursor = end;
    check_cursor_now();

    // With an exclusive selection the cursor sits one past the last
    // selected character.
    if selection_style() == b'e' && char_at_cursor() != 0 {
        Win::current().w_cursor.col += 1;
    }
}

/// Run the rhs immediately, inside a saved editor state.
fn run_now(rhs: &Rc<CStr>, noremap: c_int, silent: bool) {
    let mut state = SaveState::default();
    let _busy = Depth::of(ex_normal_busy);
    if save_current_state(&mut state) {
        exec_normal_cmd(rhs, noremap, silent);
    }
    restore_current_state(&mut state);
}

/// Put the rhs into the typeahead, as if the user had typed it.
fn feed_typeahead(rhs: &Rc<CStr>, noremap: c_int, silent: bool) {
    let _ = ins_typebuf(rhs.to_bytes(), noremap, 0, true, silent);
}

/// Why [`menu_getbyname`] found nothing: what it reports first, and whether
/// it goes on to say the path was not found.
type NotFound = (Option<&'static CStr>, bool);

/// The item `path` names, which must be a menu *item*. An empty path names
/// the first top-level menu, as upstream's walk never starts.
fn menu_getbyname(tree: &[VimMenu], path: &[u8]) -> Result<MenuId, NotFound> {
    let mut list = tree;
    let mut name = path;
    if name.is_empty() {
        return tree.first().map(|node| node.id).ok_or((None, true));
    }
    loop {
        let (component, rest) = take_component(name);
        let Some(node) = list.iter().find(|node| name_equal(&component, node)) else {
            return Err((None, true));
        };
        if rest.is_empty() && !node.children.is_empty() {
            return Err((Some(c"E333: Menu path must lead to a menu item"), false));
        }
        if !rest.is_empty() && node.children.is_empty() {
            return Err((Some(E_NOTSUBMENU), true));
        }
        if rest.is_empty() {
            return Ok(node.id);
        }
        list = &node.children;
        name = rest;
    }
}

/// `:emenu` -- find the menu a descriptor like `File.New` names and run it.
pub(crate) fn ex_emenu(excmd: &mut ExArg) {
    let text = excmd.line.arg().to_vec();
    let mut arg = &text[..];

    // An optional leading mode letter, e.g. ":emenu i File.New".
    let mut mode_idx = MENU_INDEX_INVALID;
    if !arg.is_empty() && ascii_iswhite(c_int::from(crate::cstr::byte_at(arg, 1))) {
        mode_idx = match arg[0] {
            b'n' => MENU_INDEX_NORMAL,
            b'v' => MENU_INDEX_VISUAL,
            b's' => MENU_INDEX_SELECT,
            b'o' => MENU_INDEX_OP_PENDING,
            b't' => MENU_INDEX_TERMINAL,
            b'i' => MENU_INDEX_INSERT,
            b'c' => MENU_INDEX_CMDLINE,
            _ => {
                let arg = msg_bytes(arg);
                semsg_name(tr!("E475: Invalid argument: {arg}"));
                return;
            }
        };
        arg = skip_white(&arg[2..]);
    }

    match with_tree(|tree| menu_getbyname(tree, arg)) {
        Ok(menu) => execute_menu(range_of(excmd), menu, mode_idx),
        Err((first, not_found)) => {
            if let Some(msg) = first {
                emsg_c(msg);
            }
            if not_found {
                let path = msg_bytes(arg);
                semsg_name(tr!("E334: Menu not found: {path}"));
            }
        }
    }
}

/// Find the sub-menu `path` names -- what `:popup` wants, as opposed to
/// [`menu_getbyname`]'s item -- reporting why there is none.
pub(crate) fn menu_find(path: &CStr) -> Option<MenuId> {
    let found = with_tree(|tree| {
        let mut list = tree;
        let mut name = path.to_bytes();
        if name.is_empty() {
            // Upstream's walk never starts: the first top-level menu.
            return tree.first().map(|node| Ok(node.id));
        }
        while !name.is_empty() {
            let (component, rest) = take_component(name);
            let node = list.iter().find(|node| name_equal(&component, node))?;
            if node.children.is_empty() {
                // A menu item where a sub-menu was wanted.
                return Some(Err(if rest.is_empty() {
                    c"E336: Menu path must lead to a sub-menu"
                } else {
                    E_NOTSUBMENU
                }));
            }
            if rest.is_empty() {
                return Some(Ok(node.id));
            }
            list = &node.children;
            name = rest;
        }
        None
    });
    match found {
        Some(Ok(menu)) => Some(menu),
        Some(Err(msg)) => {
            emsg_c(msg);
            None
        }
        None => {
            emsg_c(c"E337: Menu not found - check menu names");
            None
        }
    }
}

// The editor state this module reads and writes. Each hands out a reference
// for exactly one statement, so none can span the rhs being run.

fn with_curbuf<R>(f: impl FnOnce(&Buffer) -> R) -> R {
    f(&Buf::current())
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
    gchar_cursor()
}

/// `'selection'`'s first letter: `i`nclusive, `e`xclusive or `o`ld.
fn selection_style() -> u8 {
    P_SEL.first_byte()
}
