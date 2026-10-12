//! The menu tree itself -- creating, walking, listing and freeing it.
//!
//! A menu is a linked list of sibling [`Menu`] nodes, each with a `children`
//! list of its own. This is everything that treats that tree as a data
//! structure: [`find_menu`] resolves a path to a node,
//! [`menu_get_recursive`]/[`menu_get`] dump it as the nested Dict
//! `menu_get()` returns, [`show_menus`] and [`show_menus_recursive`] print
//! the `:menu` listing, [`remove_menu`] unlinks and [`free_menu`] releases a
//! subtree, and [`menu_enable_recurse`] flips the `enabled` flag.
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
use crate::getchar::state::got_int;
use crate::highlight_group::{HLF_8, HLF_D};
use crate::message::e_menu_only_exists_in_another_mode;
use crate::message::{
    msg_display, msg_display_keys, msg_outnum, msg_putchar, msg_str, msg_str_hl, msg_title,
};
use crate::types::{List, VarNumber};

/// Enable or disable the (sub)menus `name` reaches in `list`, recursively.
///
/// An empty name -- which is what `:menu enable *` becomes -- or a `*`
/// component means "every sibling at this level", and then the walk does not
/// stop at the first match.
///
/// Only the node the path *names* has its flag changed; the walk does not
/// descend past it. No textual channel reads an intermediate node's flag
/// (`menu_info()` answers an empty Dict for a submenu and the listing's `-`
/// marker needs an rhs), so `:menu disable Foo` and `:menu disable *` change
/// nothing observable. That is upstream's shape, not an omission here.
pub(crate) fn menu_enable_recurse(
    list: &mut [VimMenu],
    name: &[u8],
    modes: c_int,
    enable: bool,
) -> Result<(), MenuError> {
    if list.is_empty() {
        // Bottom of the hierarchy.
        return Ok(());
    }
    let (component, rest) = take_component(name);
    let every = component.first().is_none_or(|&b| b == b'*');

    let mut found = false;
    for node in list.iter_mut() {
        if !every && !name_equal(&component, node) {
            continue;
        }
        if !rest.is_empty() {
            if node.children.is_empty() {
                return Err(MenuError::Msg(E_NOTSUBMENU));
            }
            menu_enable_recurse(&mut node.children, rest, modes, enable)?;
        } else if enable {
            node.enabled |= modes;
        } else {
            node.enabled &= !modes;
        }
        if !every {
            found = true;
            break;
        }
    }

    if !every && !found {
        return Err(MenuError::NoMenu(component));
    }
    Ok(())
}

/// Remove the (sub)menu `name` reaches in `list` from the modes in `modes`,
/// recursively, and delete whatever is left in no mode at all.
///
/// An empty name means "every sibling at this level"; `:unmenu *` reaches
/// it that way.
pub(crate) fn remove_menu(
    list: &mut Vec<VimMenu>,
    name: &[u8],
    modes: c_int,
) -> Result<(), MenuError> {
    if list.is_empty() {
        // Bottom of the hierarchy.
        return Ok(());
    }
    let (component, rest) = take_component(name);
    let named = !component.is_empty();

    let mut at = 0;
    let mut found = None;
    while at < list.len() {
        let node = &mut list[at];
        if named && !name_equal(&component, node) {
            at += 1;
            continue;
        }
        if !rest.is_empty() && node.children.is_empty() {
            return Err(MenuError::Msg(E_NOTSUBMENU));
        }
        if node.modes & modes != 0 {
            remove_menu(&mut node.children, rest, modes)?;
        } else if named {
            return Err(MenuError::Msg(e_menu_only_exists_in_another_mode));
        }
        if named {
            found = Some(at);
            break;
        }

        // Drop these modes; when none are left the node goes with them.
        node.modes &= !modes;
        if modes & MENU_TIP_MODE != 0 {
            node.strings[MENU_INDEX_TIP as usize] = None;
        }
        if node.modes & MENU_ALL_MODES == 0 {
            list.remove(at);
        } else {
            at += 1;
        }
    }

    if named {
        let Some(at) = found else {
            return Err(MenuError::NoMenu(component));
        };
        let node = &mut list[at];
        // Recalculate the modes from the children that survived.
        node.modes &= !modes;
        node.modes |= node.children.iter().fold(0, |all, child| all | child.modes);
        if modes & MENU_TIP_MODE != 0 {
            node.strings[MENU_INDEX_TIP as usize] = None;
        }
        if node.modes & MENU_ALL_MODES == 0 {
            list.remove(at);
        }
    }
    Ok(())
}

/// One node as the nested Dict `menu_get()` answers with, or `None` when the
/// node is in none of `modes`.
fn menu_get_recursive(menu: &VimMenu, modes: c_int) -> Option<DictRef> {
    if menu.modes & modes == 0 {
        return None;
    }

    let dict_held = dict_alloc();
    let dict = dict_held.edit();
    let _ = dict.add_str(b"name", Some(&menu.dname));
    let _ = dict.add_number(b"priority", VarNumber::from(menu.priority));
    let _ = dict.add_number(b"hidden", VarNumber::from(is_hidden(menu.dname.as_bytes())));
    if menu.mnemonic != 0 {
        let _ = dict.add_str(b"shortcut", Some(&char_as_text(menu.mnemonic)));
    }
    if let Some(actext) = &menu.actext {
        let _ = dict.add_str(b"actext", Some(actext));
    }
    if menu.modes & MENU_TIP_MODE != 0
        && let Some(tooltip) = &menu.strings[MENU_INDEX_TIP as usize]
    {
        let _ = dict.add_str(b"tooltip", Some(tooltip));
    }

    if menu.children.is_empty() {
        let commands_held = dict_alloc();
        let _ = dict.add_dict(b"mappings", Some(commands_held.clone()));
        let commands = commands_held.edit();
        for (bit, mode) in MODE_CHARS.iter().enumerate() {
            if menu.modes & modes & (1 << bit) == 0 {
                continue;
            }
            let mapping_held = dict_alloc();
            let mapping = mapping_held.edit();
            let rhs = menu.strings[bit].as_deref().map(special_text);
            let _ = mapping.add_allocated_str(b"rhs", rhs);
            let _ = mapping.add_number(b"silent", VarNumber::from(menu.silent[bit]));
            let enabled = menu.enabled & (1 << bit) != 0;
            let _ = mapping.add_number(b"enabled", VarNumber::from(enabled));
            // `noremap` holds 0, REMAP_NONE (-1) or REMAP_SCRIPT (-2), and
            // these two report it as *bit tests* on values that overlap --
            // so a `:noremenu` entry answers `sid` 1 as well. `menu_info()`
            // next door compares for equality and does not agree; the
            // documented `menu_get()` example carries the bit tests' answer.
            let noremap = menu.noremap[bit];
            let _ = mapping.add_number(b"noremap", VarNumber::from(noremap & REMAP_NONE != 0));
            let _ = mapping.add_number(b"sid", VarNumber::from(noremap & REMAP_SCRIPT != 0));
            // One byte of the mode letters, so `tl` files under `t`.
            let _ = commands.add_dict(&mode.to_bytes()[..1], Some(mapping_held));
        }
    } else {
        let mut list = list_alloc();
        for child in &menu.children {
            let entry = menu_get_recursive(child, modes);
            if entry.as_ref().is_some_and(|d| !d.is_empty()) {
                list.push_dict(entry);
            }
        }
        let _ = dict.add_list(b"submenus", Some(list));
    }
    Some(dict_held)
}

/// Export the menus matching `path` into `list` -- the `menu_get()` builtin.
/// An empty path exports every top-level menu.
pub(crate) fn menu_get(path: &CStr, modes: c_int, list: &mut List) -> bool {
    let found = with_tree(|tree| {
        let nodes: Vec<&VimMenu> = if path.is_empty() {
            tree.iter().collect()
        } else {
            // A non-empty query only wants the node `find_menu` reached.
            vec![find_menu(tree, path.to_bytes(), modes)?]
        };
        for node in nodes {
            let entry = menu_get_recursive(node, modes);
            if entry.as_ref().is_some_and(|d| !d.is_empty()) {
                list.push_dict(entry);
            }
        }
        Ok(())
    });
    found.map_err(|err: MenuError| err.report()).is_ok()
}

/// Resolve `path` against the top-level menus `tree`, reporting why it
/// failed. Does not handle an empty path.
fn find_menu<'a>(tree: &'a [VimMenu], path: &[u8], modes: c_int) -> Result<&'a VimMenu, MenuError> {
    debug_assert!(!path.is_empty(), "find_menu wants a path");
    let mut list = tree;
    let mut name = path;

    loop {
        let (component, rest) = take_component(name);
        let Some(node) = list.iter().find(|node| name_equal(&component, node)) else {
            return Err(MenuError::NoMenu(component));
        };
        if !rest.is_empty() && node.children.is_empty() {
            return Err(MenuError::Msg(E_NOTSUBMENU));
        }
        if node.modes & modes == 0 {
            return Err(MenuError::Msg(e_menu_only_exists_in_another_mode));
        }
        if rest.is_empty() {
            return Ok(node);
        }
        // Found a match; search its sub-menu.
        name = rest;
        list = &node.children;
    }
}

/// The `:menu` listing: the mappings of one menu, or of the whole tree.
pub(crate) fn show_menus(path: &[u8], modes: c_int) -> bool {
    // A copy of what is listed: the listing can stop at a `--More--` prompt,
    // which holds nothing of the tree.
    let listed = with_tree(|tree| {
        if path.is_empty() {
            Ok(None)
        } else {
            find_menu(tree, path, modes).map(|node| Some(node.clone()))
        }
    });
    let listed = match listed {
        Ok(listed) => listed,
        Err(err) => {
            err.report();
            return false;
        }
    };
    let roots = if listed.is_none() {
        with_tree(<[VimMenu]>::to_vec)
    } else {
        Vec::new()
    };

    // Hold the tree still while it is listed.
    with_menus_locked(|| {
        put_title(c"\n--- Menus ---");
        match &listed {
            Some(node) => show_menus_recursive(node, modes, 0),
            None => show_children(&roots, modes, 0),
        }
    });
    true
}

/// Print `node` and everything under it, indented by `depth`.
fn show_menus_recursive(node: &VimMenu, modes: c_int, depth: c_int) {
    if node.modes & modes == 0 {
        return;
    }
    put_char(b'\n');
    if got_int.get() {
        // "q" hit at the --more-- prompt.
        return;
    }
    for _ in 0..depth {
        put(c"  ");
    }
    if node.priority != 0 {
        put_num(node.priority);
        put(c" ");
    }
    // The same highlighting as for directories.
    put_trans(&node.name, HLF_D);

    if !node.children.is_empty() {
        show_children(&node.children, modes, depth + 1);
        return;
    }
    for (bit, &mode) in MODE_CHARS.iter().enumerate() {
        if node.modes & modes & (1 << bit) == 0 {
            continue;
        }
        put_char(b'\n');
        if got_int.get() {
            return;
        }
        for _ in 0..depth + 2 {
            put(c"  ");
        }
        put(mode);
        put_char(match node.noremap[bit] {
            REMAP_NONE => b'*',
            REMAP_SCRIPT => b'&',
            _ => b' ',
        });
        put_char(if node.silent[bit] { b's' } else { b' ' });
        put_char(if node.modes & node.enabled & (1 << bit) == 0 {
            b'-'
        } else {
            b' '
        });
        put(c" ");
        match node.strings[bit].as_deref() {
            Some(rhs) if !rhs.is_empty() => put_special(rhs),
            _ => put_hl(c"<Nop>", HLF_8),
        }
    }
}

/// Recursively show `children`, skipping PopUp[nvoci].
fn show_children(children: &[VimMenu], modes: c_int, depth: c_int) {
    for child in children {
        if got_int.get() {
            break;
        }
        if !is_hidden(child.dname.as_bytes()) {
            show_menus_recursive(child, modes, depth);
        }
    }
}

// The message entry points this listing is built from.

fn put(s: &CStr) {
    msg_str(s);
}

fn put_title(s: &CStr) {
    msg_title(s);
}

fn put_hl(s: &CStr, hl_id: c_int) {
    msg_str_hl(s, hl_id, false);
}

fn put_char(byte: u8) {
    msg_putchar(c_int::from(byte));
}

fn put_num(n: c_int) {
    msg_outnum(n);
}

fn put_trans(s: &CStr, hl_id: c_int) {
    msg_display(s, hl_id, false);
}

fn put_special(s: &CStr) {
    msg_display_keys(s, false, 0);
}
