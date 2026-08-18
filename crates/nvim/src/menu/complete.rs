//! Command-line completion of a menu path -- `:emenu <Tab>` and friends.
//!
//! [`set_context_in_menu_cmd`] parses as much of a half-typed `:menu` command
//! as exists to decide what the next word could be -- a mode prefix, a menu
//! path, or nothing -- and leaves the node the path reached in [`EXPAND_MENU`]
//! for the generator. [`get_menu_name`] and [`get_menu_names`] are then the
//! two generators the completion machinery calls repeatedly, the second
//! walking submenus and offering both the translated and the original name of
//! each.
//!
//! Original: `src/nvim/menu.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use super::*;
use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::global_cell::GlobalCell;
use crate::keycodes::Ctrl_V;
use crate::types::expand_T;

/// How much of a submenu name the generator can answer with, separator
/// included. Upstream's `TBUFFER_LEN`.
const TBUFFER_LEN: usize = 256;

/// The separator `get_menu_names` marks a submenu with, so that a `.` in a
/// name is escaped as text rather than read as a path separator.
const SUBMENU_MARK: u8 = 0x01;

/// The sibling list the generators walk, left here by
/// [`set_context_in_menu_cmd`].
///
/// It is only read during the completion of the command line that set it,
/// which is the same window in which C's `expand_menu` is valid.
static EXPAND_MENU: GlobalCell<Option<Menu>> = GlobalCell::new(None);

/// The modes the candidates must be defined in.
static EXPAND_MODES: GlobalCell<c_int> = GlobalCell::new(0);

/// Whether the command being completed is `:emenu`, which does not offer
/// separators.
static EXPAND_EMENU: GlobalCell<bool> = GlobalCell::new(false);

/// What the completion machinery should do next, and where the word it is
/// completing starts.
struct Context {
    xp_context: c_int,
    pattern: Option<CText>,
}

impl Context {
    const NOTHING: Context = Context {
        xp_context: EXPAND_NOTHING,
        pattern: None,
    };
    const UNSUCCESSFUL: Context = Context {
        xp_context: EXPAND_UNSUCCESSFUL,
        pattern: None,
    };
}

/// Work out what to complete in a half-typed menu command.
///
/// # Safety
/// `xp` must be live, `cmd` a NUL-terminated string, and `arg` a position in
/// the command line being completed.
pub unsafe fn set_context_in_menu_cmd(
    xp: *mut expand_T,
    cmd: *const c_char,
    arg: *mut c_char,
    forceit: bool,
) -> *mut c_char {
    // SAFETY: the caller's obligation.
    let context = unsafe { menu_context(CStr::from_ptr(cmd), CText::new(arg), forceit) };
    // SAFETY: the caller's obligation; both writes finish here, and the
    // pattern is a position in the command line `xp` already describes.
    unsafe {
        (*xp).xp_context = context.xp_context;
        if let Some(pattern) = context.pattern {
            (*xp).xp_pattern = pattern.raw();
        }
    }
    ptr::null_mut()
}

fn white(byte: u8) -> bool {
    ascii_iswhite(c_int::from(byte))
}

/// The body of [`set_context_in_menu_cmd`].
fn menu_context(cmd: &CStr, arg: CText, forceit: bool) -> Context {
    // Step over the priority numbers, then over "enable"/"disable".
    let mut i = 0;
    while arg.byte(i) != 0 && (ascii_isdigit(c_int::from(arg.byte(i))) || arg.byte(i) == b'.') {
        i += 1;
    }
    let mut p = arg.at(i);
    if !white(p.byte(0)) {
        p = if arg.starts_with(b"enable") && (arg.byte(6) == 0 || white(arg.byte(6))) {
            arg.at(6)
        } else if arg.starts_with(b"disable") && (arg.byte(7) == 0 || white(arg.byte(7))) {
            arg.at(7)
        } else {
            arg
        };
    }
    while p.byte(0) != 0 && white(p.byte(0)) {
        p = p.at(1);
    }

    // The path being typed, and where its last component starts.
    let start = p;
    let mut after_dot = start;
    let mut i = 0;
    while start.byte(i) != 0 && !white(start.byte(i)) {
        if (start.byte(i) == b'\\' || start.byte(i) == Ctrl_V as u8) && start.byte(i + 1) != 0 {
            i += 1;
        } else if start.byte(i) == b'.' {
            after_dot = start.at(i + 1);
        }
        i += 1;
    }
    let end = start.at(i);

    // ":popup" and ":tearoff" take a menu, not one of its entries.
    let bytes = cmd.to_bytes();
    let whole_menus = !(bytes.starts_with(b"te") || bytes.first() == Some(&b'p'));
    EXPAND_EMENU.set(bytes.first() == Some(&b'e'));
    if whole_menus && white(end.byte(0)) {
        return Context::UNSUCCESSFUL;
    }
    if !end.is_empty() {
        // Still in the mapping part.
        return Context::NOTHING;
    }

    // With `:unmenu` only the command's own modes can match; with `:menu` a
    // name may be reused in another mode, so match them all.
    let (modes, _, unmenu) = cmd_modes(bytes, forceit);
    EXPAND_MODES.set(if unmenu { modes } else { MENU_ALL_MODES });

    // Everything before the last dot has to resolve to a submenu.
    let mut path: Vec<u8> = Vec::new();
    let typed = after_dot.offset_from(start);
    if typed > 0 {
        path.extend_from_slice(&start.bytes()[..typed - 1]);
        path.push(0);
    }
    let mut menu = root_first();
    let mut name = (!path.is_empty()).then(|| text_of(&mut path));
    while let Some(component) = name.filter(|n| !n.is_empty()) {
        let rest = skip_component(component);
        let matched = menu
            .into_iter()
            .flat_map(Menu::siblings)
            .find(|node| name_equal(component.as_cstr(), *node));
        let Some(node) = matched else {
            // No menu with the name we were looking for.
            return Context::UNSUCCESSFUL;
        };
        if (!rest.is_empty() && node.children().is_none()) || !node.in_modes(EXPAND_MODES.get()) {
            // The path continues past a leaf, or the menu exists only in
            // another mode.
            return Context::UNSUCCESSFUL;
        }
        name = Some(rest);
        menu = node.children();
    }

    EXPAND_MENU.set(menu);
    Context {
        xp_context: if whole_menus {
            EXPAND_MENUNAMES
        } else {
            EXPAND_MENUS
        },
        pattern: Some(after_dot),
    }
}

/// The candidate generators alternate between a node's translated and
/// English display names, advancing to the next node when both are spent.
/// A node without an English name yields one candidate.
struct Generator {
    node: Menu,
    advance: bool,
}

impl Generator {
    /// The names to skip, restart, and the node to answer for -- shared by
    /// both generators.
    fn next(
        idx: c_int,
        menu: &GlobalCell<Option<Menu>>,
        advance: &GlobalCell<bool>,
        skip: impl Fn(Menu) -> bool,
    ) -> Option<Generator> {
        if idx == 0 {
            // First call: start at the first item.
            menu.set(EXPAND_MENU.get());
            advance.set(false);
        }
        let mut cursor = menu.get();
        while let Some(node) = cursor.filter(|node| skip(*node)) {
            cursor = node.next();
        }
        menu.set(cursor);
        cursor.map(|node| Generator {
            node,
            advance: advance.get(),
        })
    }

    /// The display name to answer with, and the bookkeeping that decides
    /// whether the next call moves on.
    fn pick(&self, advance: &GlobalCell<bool>) -> *mut c_char {
        if !self.node.in_modes(EXPAND_MODES.get()) {
            // Not in these modes: an empty candidate, and no bookkeeping.
            return c"".as_ptr().cast_mut();
        }
        if self.advance {
            self.node.en_dname
        } else {
            if self.node.en_dname.is_null() {
                advance.set(true);
            }
            self.node.dname
        }
    }

    fn step(self, menu: &GlobalCell<Option<Menu>>, advance: &GlobalCell<bool>) {
        if advance.get() {
            menu.set(self.node.next());
        }
        advance.set(!advance.get());
    }
}

/// `ExpandGeneric()`'s source for the list of (sub)menus, not entries.
///
/// # Safety
/// Called by the completion machinery with the indices `0..` in order.
pub unsafe extern "C" fn get_menu_name(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    static MENU: GlobalCell<Option<Menu>> = GlobalCell::new(None);
    static ADVANCE: GlobalCell<bool> = GlobalCell::new(false);

    // Skip PopUp[nvoci], separators and leaves.
    let Some(item) = Generator::next(idx, &MENU, &ADVANCE, |node| {
        is_hidden(node.dname()) || is_separator(node.dname()) || node.children().is_none()
    }) else {
        // At the end of the linked list.
        return ptr::null_mut();
    };
    let name = item.pick(&ADVANCE);
    item.step(&MENU, &ADVANCE);
    name
}

/// `ExpandGeneric()`'s source for the list of menus *and* menu entries.
///
/// # Safety
/// As [`get_menu_name`].
pub unsafe extern "C" fn get_menu_names(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    /// Scratch for the one candidate at a time a submenu is answered with.
    static TBUFFER: GlobalCell<[u8; TBUFFER_LEN]> = GlobalCell::new([0; TBUFFER_LEN]);
    static MENU: GlobalCell<Option<Menu>> = GlobalCell::new(None);
    static ADVANCE: GlobalCell<bool> = GlobalCell::new(false);

    // Skip Browse-style entries, popup menus and separators.
    let Some(item) = Generator::next(idx, &MENU, &ADVANCE, |node| {
        is_hidden(node.dname())
            || (EXPAND_EMENU.get() && is_separator(node.dname()))
            || node.dname().to_bytes().last() == Some(&b'.')
    }) else {
        return ptr::null_mut();
    };

    let name = item.pick(&ADVANCE);
    let name = if item.node.children().is_some() && item.node.in_modes(EXPAND_MODES.get()) {
        // Mark it as a submenu with a magic byte. Upstream copies up to the
        // whole buffer and then appends, overrunning it by one for a
        // 255-byte name; the separator is reserved for here.
        // SAFETY: `name` is one of the node's display names.
        let bytes = unsafe { CStr::from_ptr(name) }.to_bytes();
        let kept = bytes.len().min(TBUFFER_LEN - 2);
        TBUFFER.with_mut(|buf| {
            buf[..kept].copy_from_slice(&bytes[..kept]);
            buf[kept] = SUBMENU_MARK;
            buf[kept + 1] = 0;
        });
        // The generator's contract is a borrowed string, so this hands back
        // the scratch buffer itself.
        TBUFFER.ptr().cast()
    } else {
        name
    };

    item.step(&MENU, &ADVANCE);
    name
}
