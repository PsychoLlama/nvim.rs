//! Menus: `:menu` and the tree it builds.
//!
//! Carved by the stage:
//!
//! | child | what |
//! | --- | --- |
//! | [`define`] | `:menu` and `add_menu_path()` |
//! | [`tree`] | the `VimMenu` tree -- find, list, dump, remove, free |
//! | [`complete`] | command-line completion of a menu path |
//! | [`name`] | names as text: path components, mode letters, accelerators |
//! | [`exec`] | `:emenu`, `:popup` and running a right-hand side |
//! | [`info`] | `:menutranslate` and `menu_info()` |
//!
//! What stays here is what the six share: the mode alphabet (`MENU_*_MODE`,
//! `MENU_INDEX_*`, [`MODE_CHARS`]), the `menus_locked` guard, the
//! `is_*` predicates that classify a name, [`get_menu_mode`] -- "which mode
//! is the editor in?" for the executing side -- and the tree itself.
//!
//! The tree is [`MENU_TREE`]: the top-level menus, each [`VimMenu`] owning
//! its children in display order. It is lent for one closure; what has to
//! outlive a borrow -- `:popup`'s menu, which its key loop holds across user
//! code -- is a [`MenuId`], re-found with [`with_menu`].
//!
//! Original: `src/nvim/menu.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::types::AutoEvent;
use crate::winlayer::Buf;
use core::ffi::{CStr, c_int};
use std::ffi::CString;

use crate::ascii::ascii_iswhite;
use crate::autocmd::fire_autocmds_for;
use crate::eval::typval::{DictRef, ListRef, tv_dict_alloc, tv_list_alloc};
use crate::global_cell::GlobalCell;
use crate::guard::Lock;
pub(crate) use crate::mapping::skip_white;
use crate::mbyte::encode_char;
use crate::memory::ThinCString;
use crate::message::e_cannot_change_menus_while_listing;
use crate::message::{emsg, special_keys_text};
use crate::message_fmt::msg_bytes;
use crate::normal::{visual_active, visual_select};
use crate::os::cshim::gettext;
use crate::popupmenu::pum_show_popupmenu;
use crate::state::mode::{State, finish_op};
use crate::state::{
    MODE_ASKMORE, MODE_CMDLINE, MODE_HITRETURN, MODE_INSERT, MODE_LANGMAP, MODE_NORMAL,
    MODE_TERMINAL,
};
use crate::tr;
use crate::types::{MenuId, VimMenu, kListLenMayKnow, ptrdiff_t};

// The carve of the transpiled module; see each child's docs.
mod complete;
mod define;
mod exec;
mod info;
mod name;
#[cfg(test)]
mod tests;
mod tree;

pub(crate) use self::complete::*;
pub(crate) use self::define::*;
pub(crate) use self::exec::*;
pub(crate) use self::info::*;
pub(crate) use self::name::*;
pub(crate) use self::tree::*;

/// The menu tree: the top-level menus, in menubar order, each owning its
/// children in turn.
pub(crate) struct MenuTree {
    pub(crate) menus: Vec<VimMenu>,
    /// The id the next node gets; never reused.
    next_id: u64,
}

impl MenuTree {
    /// A fresh node id.
    pub(crate) fn new_id(&mut self) -> MenuId {
        let id = self.next_id;
        self.next_id += 1;
        MenuId(id)
    }
}

pub(crate) static MENU_TREE: GlobalCell<MenuTree> = GlobalCell::new(MenuTree {
    menus: Vec::new(),
    next_id: 1,
});

/// Lend the top-level menus for reading. `f` must not reach the tree itself.
pub(crate) fn with_tree<R>(f: impl FnOnce(&[VimMenu]) -> R) -> R {
    MENU_TREE.with(|tree| f(&tree.menus))
}

/// Lend the tree for writing; the same rule as [`with_tree`].
pub(crate) fn with_tree_mut<R>(f: impl FnOnce(&mut MenuTree) -> R) -> R {
    MENU_TREE.with_mut(f)
}

/// The node `id` names, anywhere under `list`.
pub(crate) fn find_id(list: &[VimMenu], id: MenuId) -> Option<&VimMenu> {
    list.iter().find_map(|node| {
        if node.id == id {
            Some(node)
        } else {
            find_id(&node.children, id)
        }
    })
}

/// Lend the node `id` names, or answer `None` when it has been deleted.
pub(crate) fn with_menu<R>(id: MenuId, f: impl FnOnce(&VimMenu) -> R) -> Option<R> {
    with_tree(|tree| find_id(tree, id).map(f))
}

/// Why a walk of the tree gave up, reported once it has let go of the tree.
pub(crate) enum MenuError {
    /// One of the fixed messages, untranslated.
    Msg(&'static CStr),
    /// "E329: No menu", naming the component that was not found.
    NoMenu(Vec<u8>),
}

impl MenuError {
    /// Print the message.
    pub(crate) fn report(&self) {
        match self {
            MenuError::Msg(msg) => emsg_c(msg),
            MenuError::NoMenu(name) => {
                let shown = msg_bytes(name);
                semsg_name(tr!("E329: No menu \"{shown}\""));
            }
        }
    }
}

pub(crate) static sys_menu: GlobalCell<bool> = GlobalCell::new(false);

/// How deep a menu path may go, and so how many priority components
/// `:menu 10.20.30 …` can carry.
pub(crate) const MENUDEPTH: usize = 10;

/// A name starting with this is never shown in the menubar or the listing.
const MNU_HIDDEN_CHAR: u8 = b']';

pub(crate) const MENU_MODES: usize = 8;

pub(crate) const MENU_INDEX_INVALID: c_int = -1;
pub(crate) const MENU_INDEX_NORMAL: c_int = 0;
pub(crate) const MENU_INDEX_VISUAL: c_int = 1;
pub(crate) const MENU_INDEX_SELECT: c_int = 2;
pub(crate) const MENU_INDEX_OP_PENDING: c_int = 3;
pub(crate) const MENU_INDEX_INSERT: c_int = 4;
pub(crate) const MENU_INDEX_CMDLINE: c_int = 5;
pub(crate) const MENU_INDEX_TERMINAL: c_int = 6;
pub(crate) const MENU_INDEX_TIP: c_int = 7;

pub(crate) const MENU_NORMAL_MODE: c_int = 1 << MENU_INDEX_NORMAL;
pub(crate) const MENU_VISUAL_MODE: c_int = 1 << MENU_INDEX_VISUAL;
pub(crate) const MENU_SELECT_MODE: c_int = 1 << MENU_INDEX_SELECT;
pub(crate) const MENU_OP_PENDING_MODE: c_int = 1 << MENU_INDEX_OP_PENDING;
pub(crate) const MENU_INSERT_MODE: c_int = 1 << MENU_INDEX_INSERT;
pub(crate) const MENU_CMDLINE_MODE: c_int = 1 << MENU_INDEX_CMDLINE;
pub(crate) const MENU_TERMINAL_MODE: c_int = 1 << MENU_INDEX_TERMINAL;
pub(crate) const MENU_TIP_MODE: c_int = 1 << MENU_INDEX_TIP;
/// Every mode but the tooltip pseudo-mode.
pub(crate) const MENU_ALL_MODES: c_int = MENU_TIP_MODE - 1;

/// `:amenu` -- the six modes a menu entry can be typed in.
pub(crate) const MENU_AMENU_MODES: c_int = MENU_INSERT_MODE
    | MENU_CMDLINE_MODE
    | MENU_NORMAL_MODE
    | MENU_VISUAL_MODE
    | MENU_SELECT_MODE
    | MENU_OP_PENDING_MODE;

/// `:menu` without a bang.
pub(crate) const MENU_PLAIN_MODES: c_int =
    MENU_NORMAL_MODE | MENU_VISUAL_MODE | MENU_SELECT_MODE | MENU_OP_PENDING_MODE;

/// The letter each mode is spelled with, indexed by `MENU_INDEX_*`. It is
/// both the `:menu` listing's column and the key `menu_get()` files a
/// mapping under, so the order is observable.
pub(crate) const MODE_CHARS: [&CStr; MENU_MODES] =
    [c"n", c"v", c"s", c"o", c"i", c"c", c"tl", c"t"];

/// `noremap` values stored per mode in a `VimMenu`.
pub(crate) const REMAP_SCRIPT: c_int = -2;
pub(crate) const REMAP_NONE: c_int = -1;
pub(crate) const REMAP_YES: c_int = 0;

/// `replace_termcodes()`: also translate `<lt>`.
pub(crate) const REPTERM_DO_LT: c_int = 2;

pub(crate) static E_NOTSUBMENU: &CStr = c"E327: Part of menu-item path is not sub-menu";

/// The translated text of one of the shared `e_*` message constants. The
/// `_c` message macros want a `printf` format string, which a
/// `format_args!` literal cannot be.
pub(crate) fn message(msg: &'static CStr) -> &'static CStr {
    gettext(msg)
}

/// [`message`] for a message this module owns.
pub(crate) fn message_str(msg: &'static CStr) -> &'static CStr {
    gettext(msg)
}

/// `emsg(_(msg))`.
pub(crate) fn emsg_c(msg: &'static CStr) {
    emsg(message_str(msg));
}

/// `emsg(_(msg))` for one of the shared `e_*` constants.
pub(crate) fn emsg_shared(msg: &'static CStr) {
    emsg(message(msg));
}

/// `semsg(fmt, arg)` for the five messages that interpolate a menu name.
///
/// A menu name is arbitrary bytes -- the sweep defines one out of invalid
/// UTF-8 -- so these stay on vim's own `printf`, which copies them through,
/// rather than moving to a `format_args!` literal that would have to lose
/// the bytes it cannot decode.
pub(crate) fn semsg_name(message: String) {
    crate::message_fmt::emsg_text(message);
}

/// While non-zero no menu may be added or removed, so that the listing
/// cannot walk a tree that is changing under it.
static MENUS_LOCKED: GlobalCell<c_int> = GlobalCell::new(0);

/// Whether the menus are locked, reporting it if they are.
pub(crate) fn is_menus_locked() -> bool {
    if MENUS_LOCKED.get() > 0 {
        emsg_shared(e_cannot_change_menus_while_listing);
        return true;
    }
    false
}

/// Holds the menu tree still for the duration of `f`.
pub(crate) fn with_menus_locked<R>(f: impl FnOnce() -> R) -> R {
    let _locked = Lock::held(&MENUS_LOCKED);
    f()
}

// ---------------------------------------------------------------------------
// The Dicts `menu_get()` and `menu_info()` answer with
// ---------------------------------------------------------------------------

pub(crate) fn dict_alloc() -> DictRef {
    tv_dict_alloc()
}

pub(crate) fn list_alloc() -> ListRef {
    tv_list_alloc(kListLenMayKnow as ptrdiff_t)
}

/// A right-hand side with its special keys spelled out.
pub(crate) fn special_text(rhs: &CStr) -> ThinCString {
    ThinCString::from_vec(special_keys_text(rhs.to_bytes(), false, false))
}

/// One codepoint as an owned string -- the mnemonic, for the two Dicts.
/// Answers the empty string for a node with no mnemonic, as C's
/// `buf[utf_char2bytes(0, buf)] = NUL` leaves an empty buffer.
pub(crate) fn char_as_text(c: c_int) -> CString {
    let mut buf = [0u8; 8];
    let len = encode_char(c, &mut buf);
    let bytes = &buf[..len];
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(len);
    CString::new(&bytes[..end]).expect("truncated at the first NUL")
}

// ---------------------------------------------------------------------------
// Classifying a name
// ---------------------------------------------------------------------------

/// Whether `name` can appear in the menubar -- everything that is not one of
/// the three special roots or explicitly hidden.
pub(crate) fn is_menubar(name: &[u8]) -> bool {
    !is_popup(name)
        && !is_toolbar(name)
        && !is_winbar(name)
        && name.first() != Some(&MNU_HIDDEN_CHAR)
}

/// Whether `name` names the right-click menu, whose per-mode copies
/// `PopUpn`, `PopUpi`, ... all match too.
pub(crate) fn is_popup(name: &[u8]) -> bool {
    name.starts_with(b"PopUp")
}

fn is_toolbar(name: &[u8]) -> bool {
    name.starts_with(b"ToolBar")
}

/// Whether `name` names a window toolbar menu.
fn is_winbar(name: &[u8]) -> bool {
    name.starts_with(b"WinBar")
}

/// Whether `name` is a separator: it both starts and ends with `-`.
///
/// A one-byte `"-"` satisfies both, as it does in C.
pub(crate) fn is_separator(name: &[u8]) -> bool {
    name.first() == Some(&b'-') && name.last() == Some(&b'-')
}

/// Whether `name` is kept out of the menubar and the listing: the hidden
/// marker, or one of `PopUp`'s per-mode copies (plain `PopUp` is shown).
pub(crate) fn is_hidden(name: &[u8]) -> bool {
    name.first() == Some(&MNU_HIDDEN_CHAR) || (is_popup(name) && name.len() > 5)
}

// ---------------------------------------------------------------------------
// The mode the editor is in
// ---------------------------------------------------------------------------

/// Which mode's right-hand side a menu invoked right now should run, or
/// [`MENU_INDEX_INVALID`] if the editor is in none of them.
pub(crate) fn get_menu_mode() -> c_int {
    if State.get() & MODE_TERMINAL != 0 {
        return MENU_INDEX_TERMINAL;
    }
    if visual_active() {
        return if visual_select() {
            MENU_INDEX_SELECT
        } else {
            MENU_INDEX_VISUAL
        };
    }
    if State.get() & MODE_INSERT != 0 {
        return MENU_INDEX_INSERT;
    }
    if State.get() & MODE_CMDLINE != 0
        || State.get() == MODE_ASKMORE
        || State.get() == MODE_HITRETURN
    {
        return MENU_INDEX_CMDLINE;
    }
    if finish_op.get() {
        return MENU_INDEX_OP_PENDING;
    }
    if State.get() & MODE_NORMAL != 0 {
        return MENU_INDEX_NORMAL;
    }
    if State.get() & MODE_LANGMAP != 0 {
        // Must be an "r" command, which behaves like Insert mode.
        return MENU_INDEX_INSERT;
    }
    MENU_INDEX_INVALID
}

/// [`get_menu_mode`] as a `MENU_*_MODE` bit, or 0 for no mode at all.
pub(crate) fn get_menu_mode_flag() -> c_int {
    let mode = get_menu_mode();
    if mode == MENU_INDEX_INVALID {
        return 0;
    }
    1 << mode
}

/// Show the `PopUp` menu for the mode the editor is in -- `PopUpn` in
/// Normal mode, `PopUpi` in Insert mode, and so on.
pub(crate) fn show_popupmenu() {
    let menu_mode = get_menu_mode();
    if menu_mode == MENU_INDEX_INVALID {
        return;
    }
    let mode = MODE_CHARS[menu_mode as usize];

    // The autocmd may redefine menus, which is why the search below runs
    // afterwards.
    fire_autocmds_for(
        AutoEvent::MenuPopup,
        Some(mode),
        None,
        false,
        Buf::current_or_none(),
        None,
    );

    // "PopUp" followed by this mode's letters; a longer name still matches,
    // as the C prefix compare did. Only show a popup when it is defined and
    // has entries.
    let found = with_tree(|tree| {
        tree.iter()
            .find(|menu| {
                let name = menu.name.as_bytes();
                name.starts_with(b"PopUp") && name[5..].starts_with(mode.to_bytes())
            })
            .filter(|menu| !menu.children.is_empty())
            .map(|menu| menu.id)
    });
    if let Some(menu) = found {
        pum_show_popupmenu(menu);
    }
}
