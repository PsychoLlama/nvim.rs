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

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int};

use super::*;
use crate::ascii::ascii_isdigit;
use crate::cstr;
use crate::global_cell::GlobalCell;
use crate::keycodes::Ctrl_V;
use crate::types::{Candidate, Expand, ExpandContext};
use std::ffi::CString;

/// How much of a submenu name the generator can answer with, separator
/// included. Upstream's `TBUFFER_LEN`.
const TBUFFER_LEN: usize = 256;

/// The separator `get_menu_names` marks a submenu with, so that a `.` in a
/// name is escaped as text rather than read as a path separator.
const SUBMENU_MARK: u8 = 0x01;

/// A sibling list of the tree: the top level, or a node's children.
#[derive(Copy, Clone)]
enum Level {
    Top,
    Children(MenuId),
}

impl Level {
    /// Lend the list, or answer `None` when the node is gone.
    fn with<R>(self, f: impl FnOnce(&[VimMenu]) -> R) -> Option<R> {
        match self {
            Level::Top => Some(with_tree(f)),
            Level::Children(id) => with_menu(id, |node| f(&node.children)),
        }
    }
}

/// The sibling list the generators walk, left here by
/// [`set_context_in_menu_cmd`]; `None` when the path reached nothing.
///
/// It is only read during the completion of the command line that set it.
static EXPAND_MENU: GlobalCell<Option<Level>> = GlobalCell::new(None);

/// The modes the candidates must be defined in.
static EXPAND_MODES: GlobalCell<c_int> = GlobalCell::new(0);

/// Whether the command being completed is `:emenu`, which does not offer
/// separators.
static EXPAND_EMENU: GlobalCell<bool> = GlobalCell::new(false);

/// What the completion machinery should do next, and where the word it is
/// completing starts.
struct Context {
    xp_context: ExpandContext,
    pattern: Option<usize>,
}

impl Context {
    const NOTHING: Context = Context {
        xp_context: ExpandContext::Nothing,
        pattern: None,
    };
    const UNSUCCESSFUL: Context = Context {
        xp_context: ExpandContext::Unsuccessful,
        pattern: None,
    };
}

/// Work out what to complete in a half-typed menu command `cmd`, whose
/// argument starts at `arg` in the completion's line. Answers no next
/// command: a menu command takes the rest of the line.
pub(crate) fn set_context_in_menu_cmd(
    expand: &mut Expand,
    cmd: &CStr,
    arg: usize,
    forceit: bool,
) -> Option<usize> {
    let line = expand.line_cstr().to_bytes();
    let context = menu_context(cmd, line, arg.min(line.len()), forceit);
    expand.context = context.xp_context;
    if let Some(pattern) = context.pattern {
        expand.pattern = pattern;
    }
    None
}

fn white(byte: u8) -> bool {
    ascii_iswhite(c_int::from(byte))
}

/// The body of [`set_context_in_menu_cmd`]: `arg` is where the argument
/// starts in `line`, and the pattern answered an offset in it too.
fn menu_context(cmd: &CStr, line: &[u8], arg: usize, forceit: bool) -> Context {
    let at = |i: usize| cstr::byte_at(line, i);
    let starts = |i: usize, word: &[u8]| line[i.min(line.len())..].starts_with(word);
    // Step over the priority numbers, then over "enable"/"disable".
    let mut i = arg;
    while at(i) != 0 && (ascii_isdigit(c_int::from(at(i))) || at(i) == b'.') {
        i += 1;
    }
    let mut p = i;
    if !white(at(p)) {
        p = if starts(arg, b"enable") && (at(arg + 6) == 0 || white(at(arg + 6))) {
            arg + 6
        } else if starts(arg, b"disable") && (at(arg + 7) == 0 || white(at(arg + 7))) {
            arg + 7
        } else {
            arg
        };
    }
    while at(p) != 0 && white(at(p)) {
        p += 1;
    }

    // The path being typed, and where its last component starts.
    let start = p;
    let mut after_dot = start;
    let mut i = start;
    while at(i) != 0 && !white(at(i)) {
        let escaped = at(i) == b'\\' || c_int::from(at(i)) == Ctrl_V;
        if escaped && at(i + 1) != 0 {
            i += 1;
        } else if at(i) == b'.' {
            after_dot = i + 1;
        }
        i += 1;
    }
    let end = i;

    // ":popup" and ":tearoff" take a menu, not one of its entries.
    let bytes = cmd.to_bytes();
    let whole_menus = !(bytes.starts_with(b"te") || bytes.first() == Some(&b'p'));
    EXPAND_EMENU.set(bytes.first() == Some(&b'e'));
    if whole_menus && white(at(end)) {
        return Context::UNSUCCESSFUL;
    }
    if at(end) != 0 {
        // Still in the mapping part.
        return Context::NOTHING;
    }

    // With `:unmenu` only the command's own modes can match; with `:menu` a
    // name may be reused in another mode, so match them all.
    let (modes, _, unmenu) = cmd_modes(bytes, forceit);
    EXPAND_MODES.set(if unmenu { modes } else { MENU_ALL_MODES });

    // Everything before the last dot has to resolve to a submenu.
    let path = if after_dot > start {
        &line[start..after_dot - 1]
    } else {
        &b""[..]
    };
    let level = with_tree(|tree| {
        let mut list = tree;
        let mut level = Level::Top;
        let mut name = path;
        while !name.is_empty() {
            let (component, rest) = take_component(name);
            // No menu with the name we were looking for.
            let node = list.iter().find(|node| name_equal(&component, node))?;
            if (!rest.is_empty() && node.children.is_empty())
                || node.modes & EXPAND_MODES.get() == 0
            {
                // The path continues past a leaf, or the menu exists only
                // in another mode.
                return None;
            }
            name = rest;
            list = &node.children;
            level = Level::Children(node.id);
        }
        Some(level)
    });
    let Some(level) = level else {
        return Context::UNSUCCESSFUL;
    };

    EXPAND_MENU.set(Some(level));
    Context {
        xp_context: if whole_menus {
            ExpandContext::Menunames
        } else {
            ExpandContext::Menus
        },
        pattern: Some(after_dot),
    }
}

/// What a generator needs of the node it answers for, copied out of the
/// tree.
struct Item {
    in_modes: bool,
    has_children: bool,
    dname: CString,
    en_dname: Option<CString>,
}

/// The candidate generators alternate between a node's translated and
/// English display names, advancing to the next node when both are spent.
/// A node without an English name yields one candidate.
struct Generator {
    item: Item,
    advance: bool,
}

impl Generator {
    /// The names to skip, restart, and the node to answer for -- shared by
    /// both generators. `pos` is the index in the level being walked.
    fn next(
        idx: usize,
        pos: &GlobalCell<usize>,
        advance: &GlobalCell<bool>,
        skip: impl Fn(&VimMenu) -> bool,
    ) -> Option<Generator> {
        if idx == 0 {
            // First call: start at the first item.
            pos.set(0);
            advance.set(false);
        }
        let level = EXPAND_MENU.get()?;
        let modes = EXPAND_MODES.get();
        let found = level.with(|list| {
            let mut at = pos.get();
            while list.get(at).is_some_and(&skip) {
                at += 1;
            }
            pos.set(at);
            list.get(at).map(|node| Item {
                in_modes: node.modes & modes != 0,
                has_children: !node.children.is_empty(),
                dname: node.dname.clone(),
                en_dname: node.en_dname.clone(),
            })
        })??;
        Some(Generator {
            item: found,
            advance: advance.get(),
        })
    }

    /// The display name to answer with, and the bookkeeping that decides
    /// whether the next call moves on.
    fn pick(&self, advance: &GlobalCell<bool>) -> &CStr {
        if !self.item.in_modes {
            // Not in these modes: an empty candidate, and no bookkeeping.
            return c"";
        }
        if self.advance {
            self.item.en_dname.as_deref().unwrap_or(c"")
        } else {
            if self.item.en_dname.is_none() {
                advance.set(true);
            }
            &self.item.dname
        }
    }

    fn step(&self, pos: &GlobalCell<usize>, advance: &GlobalCell<bool>) {
        if advance.get() {
            pos.set(pos.get() + 1);
        }
        advance.set(!advance.get());
    }
}

/// `expand_generic()`'s source for the list of (sub)menus, not entries.
pub(crate) fn get_menu_name(_expand: &Expand, idx: usize) -> Option<Candidate> {
    static POS: GlobalCell<usize> = GlobalCell::new(0);
    static ADVANCE: GlobalCell<bool> = GlobalCell::new(false);

    // Skip PopUp[nvoci], separators and leaves.
    let item = Generator::next(idx, &POS, &ADVANCE, |node| {
        let dname = node.dname.as_bytes();
        is_hidden(dname) || is_separator(dname) || node.children.is_empty()
    })?; // At the end of the list.
    let name = Candidate::Owned(item.pick(&ADVANCE).to_owned());
    item.step(&POS, &ADVANCE);
    Some(name)
}

/// `expand_generic()`'s source for the menu entries `:emenu` and friends
/// take. A submenu comes back with [`SUBMENU_MARK`] appended, which the
/// caller turns back into the `.` it stands for.
pub(crate) fn get_menu_names(_expand: &Expand, idx: usize) -> Option<Candidate> {
    static POS: GlobalCell<usize> = GlobalCell::new(0);
    static ADVANCE: GlobalCell<bool> = GlobalCell::new(false);

    // Skip Browse-style entries, popup menus and separators.
    let item = Generator::next(idx, &POS, &ADVANCE, |node| {
        let dname = node.dname.as_bytes();
        is_hidden(dname)
            || (EXPAND_EMENU.get() && is_separator(dname))
            || dname.last() == Some(&b'.')
    })?;

    let name = item.pick(&ADVANCE);
    let name = if item.item.has_children && item.item.in_modes {
        // Mark it as a submenu with a magic byte. Upstream copies up to the
        // whole buffer and then appends, overrunning it by one for a
        // 255-byte name; the separator is reserved for here.
        let bytes = name.to_bytes();
        let mut marked = bytes[..bytes.len().min(TBUFFER_LEN - 2)].to_vec();
        marked.push(SUBMENU_MARK);
        CString::new(marked).expect("a menu name holds no NUL")
    } else {
        name.to_owned()
    };

    item.step(&POS, &ADVANCE);
    Some(Candidate::Owned(name))
}
