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

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use super::*;
use crate::src::nvim::highlight_group::{HLF_8, HLF_D};
use crate::src::nvim::main::{e_menu_only_exists_in_another_mode, got_int};
use crate::src::nvim::memory::xfree;
use crate::src::nvim::message::{
    msg_outnum, msg_outtrans, msg_outtrans_special, msg_putchar, msg_puts, msg_puts_hl,
    msg_puts_title,
};
use crate::src::nvim::types::{dict_T, list_T, varnumber_T};

/// Enable or disable the (sub)menus `name` reaches, recursively.
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
    menu: Option<Menu>,
    name: CText,
    modes: c_int,
    enable: bool,
) -> bool {
    let Some(first) = menu else {
        // Bottom of the hierarchy.
        return true;
    };
    let rest = skip_component(name);
    let every = name.is_empty() || name.byte(0) == b'*';

    let mut found = false;
    for mut node in first.siblings() {
        if !every && !name_equal(name.as_cstr(), node) {
            continue;
        }
        if !rest.is_empty() {
            let Some(children) = node.children() else {
                emsg_c(E_NOTSUBMENU);
                return false;
            };
            if !menu_enable_recurse(Some(children), rest, modes, enable) {
                return false;
            }
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
        semsg_name(message_str(E_NOMENU), name.raw());
        return false;
    }
    true
}

/// Remove the (sub)menu `name` reaches from the modes in `modes`,
/// recursively, and free whatever is left in no mode at all.
///
/// An empty name means "every sibling at this level"; `:unmenu *` reaches
/// it that way.
pub(crate) fn remove_menu(menup: Link, name: CText, modes: c_int, silent: bool) -> bool {
    if menup.get().is_none() {
        // Bottom of the hierarchy.
        return true;
    }
    let rest = skip_component(name);
    let named = !name.is_empty();

    let mut menup = menup;
    let mut found = None;
    while let Some(mut node) = menup.get() {
        if named && !name_equal(name.as_cstr(), node) {
            menup = node.next_link();
            continue;
        }
        if !rest.is_empty() && node.children().is_none() {
            if !silent {
                emsg_c(E_NOTSUBMENU);
            }
            return false;
        }
        if node.in_modes(modes) {
            if !remove_menu(node.children_link(), rest, modes, silent) {
                return false;
            }
        } else if named {
            if !silent {
                emsg_shared(&e_menu_only_exists_in_another_mode);
            }
            return false;
        }
        if named {
            found = Some(node);
            break;
        }

        // Drop these modes; when none are left the node goes with them.
        node.modes &= !modes;
        if modes & MENU_TIP_MODE != 0 {
            free_menu_string(node, MENU_INDEX_TIP as usize);
        }
        if node.modes & MENU_ALL_MODES == 0 {
            free_menu(menup);
        } else {
            menup = node.next_link();
        }
    }

    if named {
        let Some(mut node) = found else {
            if !silent {
                semsg_name(message_str(E_NOMENU), name.raw());
            }
            return false;
        };
        // Recalculate the modes from the children that survived.
        node.modes &= !modes;
        let children = node.children();
        for child in children.into_iter().flat_map(Menu::siblings) {
            node.modes |= child.modes;
        }
        if modes & MENU_TIP_MODE != 0 {
            free_menu_string(node, MENU_INDEX_TIP as usize);
        }
        if node.modes & MENU_ALL_MODES == 0 {
            // Upstream re-stores `node` into `menup` here; the loop only
            // broke out without advancing, so the slot already holds it.
            free_menu(menup);
        }
    }
    true
}

/// Unlink the node in `menup` and release it.
pub(crate) fn free_menu(menup: Link) {
    let menu = menup.get().expect("free_menu wants an occupied slot");
    menup.set(menu.next());
    free_str(menu.name);
    free_str(menu.dname);
    free_str(menu.en_name);
    free_str(menu.en_dname);
    free_str(menu.actext);
    for idx in 0..MENU_MODES {
        free_menu_string(menu, idx);
    }
    // SAFETY: the node came from `alloc_node`, is now unlinked, and nothing
    // else holds it -- its children were freed before it could get here.
    unsafe { xfree(menu.raw().cast()) };
}

/// Clear the right-hand side stored for one mode.
///
/// Several modes commonly share one buffer (see `set_rhs`), so it is only
/// freed once this is the last mode holding it.
pub(crate) fn free_menu_string(mut menu: Menu, idx: usize) {
    let rhs = menu.strings[idx];
    if menu.strings.iter().filter(|s| **s == rhs).count() == 1 {
        free_str(rhs);
    }
    menu.strings[idx] = ptr::null_mut();
}

/// One node as the nested Dict `menu_get()` answers with, or null when the
/// node is in none of `modes`.
fn menu_get_recursive(menu: Menu, modes: c_int) -> *mut dict_T {
    if !menu.in_modes(modes) {
        return ptr::null_mut();
    }

    let dict = dict_alloc();
    dict_add_str(dict, c"name", menu.dname());
    dict_add_nr(dict, c"priority", varnumber_T::from(menu.priority));
    dict_add_nr(dict, c"hidden", varnumber_T::from(is_hidden(menu.dname())));
    if menu.mnemonic != 0 {
        dict_add_str(dict, c"shortcut", &char_as_text(menu.mnemonic));
    }
    if let Some(actext) = menu.actext() {
        dict_add_str(dict, c"actext", actext);
    }
    if menu.modes & MENU_TIP_MODE != 0
        && let Some(tooltip) = menu.rhs(MENU_INDEX_TIP as usize)
    {
        dict_add_str(dict, c"tooltip", tooltip);
    }

    match menu.children() {
        None => {
            let commands = dict_alloc();
            dict_add_dict(dict, c"mappings".to_bytes(), commands);
            for bit in 0..MENU_MODES {
                if menu.modes & modes & (1 << bit) == 0 {
                    continue;
                }
                let mapping = dict_alloc();
                dict_add_allocated_str(mapping, c"rhs", special_text(menu.strings[bit]));
                dict_add_nr(mapping, c"silent", varnumber_T::from(menu.silent[bit]));
                dict_add_nr(
                    mapping,
                    c"enabled",
                    varnumber_T::from(menu.enabled & (1 << bit) != 0),
                );
                // `noremap` holds 0, REMAP_NONE (-1) or REMAP_SCRIPT (-2),
                // and these two report it as *bit tests* on values that
                // overlap -- so a `:noremenu` entry answers `sid` 1 as well.
                // `menu_info()` next door compares for equality and does not
                // agree; the documented `menu_get()` example carries the bit
                // tests' answer.
                dict_add_nr(
                    mapping,
                    c"noremap",
                    varnumber_T::from(menu.noremap[bit] & REMAP_NONE != 0),
                );
                dict_add_nr(
                    mapping,
                    c"sid",
                    varnumber_T::from(menu.noremap[bit] & REMAP_SCRIPT != 0),
                );
                // One byte of the mode letters, so `tl` files under `t`.
                dict_add_dict(commands, &MODE_CHARS[bit].to_bytes()[..1], mapping);
            }
        }
        Some(children) => {
            let list = list_alloc();
            for child in children.siblings() {
                let entry = menu_get_recursive(child, modes);
                if dict_len(entry) > 0 {
                    list_append_dict(list, entry);
                }
            }
            dict_add_list(dict, c"submenus", list);
        }
    }
    dict
}

/// Export the menus matching `path_name` into `list` -- the `menu_get()`
/// builtin. An empty path exports every top-level menu.
///
/// # Safety
/// `path_name` must name a NUL-terminated string and `list` a live List.
pub unsafe extern "C" fn menu_get(path_name: *mut c_char, modes: c_int, list: *mut list_T) -> bool {
    // SAFETY: the caller's obligation.
    let path = unsafe { CStr::from_ptr(path_name) };

    let mut menu = root_first();
    if !path.is_empty() {
        menu = find_menu(menu, path, modes);
        if menu.is_none() {
            return false;
        }
    }
    for node in menu.into_iter().flat_map(Menu::siblings) {
        let entry = menu_get_recursive(node, modes);
        if !entry.is_null() && dict_len(entry) > 0 {
            list_append_dict(list, entry);
        }
        if !path.is_empty() {
            // A non-empty query only wants the node `find_menu` reached.
            break;
        }
    }
    true
}

/// Resolve `path_name` against `menu`'s sibling list, reporting why it
/// failed. Does not handle an empty path.
fn find_menu(menu: Option<Menu>, path_name: &CStr, modes: c_int) -> Option<Menu> {
    debug_assert!(!path_name.is_empty(), "find_menu wants a path");
    let mut buf = scratch(path_name);
    let mut name = text_of(&mut buf);
    let mut menu = menu;

    while !name.is_empty() {
        let rest = skip_component(name);
        let mut matched = None;
        for node in menu.into_iter().flat_map(Menu::siblings) {
            if !name_equal(name.as_cstr(), node) {
                continue;
            }
            if !rest.is_empty() && node.children().is_none() {
                emsg_c(E_NOTSUBMENU);
                return None;
            }
            if !node.in_modes(modes) {
                emsg_shared(&e_menu_only_exists_in_another_mode);
                return None;
            }
            if rest.is_empty() {
                return Some(node);
            }
            matched = Some(node);
            break;
        }
        let Some(node) = matched else {
            semsg_name(message_str(E_NOMENU), name.raw());
            return None;
        };
        // Found a match; search its sub-menu.
        name = rest;
        menu = node.children();
    }
    menu
}

/// The `:menu` listing: the mappings of one menu, or of the whole tree.
pub(crate) fn show_menus(path_name: &CStr, modes: c_int) -> bool {
    let mut menu = None;
    if !path_name.is_empty() {
        menu = find_menu(root_first(), path_name, modes);
        if menu.is_none() {
            return false;
        }
    }

    // Hold the tree still while it is walked.
    with_menus_locked(|| {
        put_title(c"\n--- Menus ---");
        show_menus_recursive(menu, modes, 0);
    });
    true
}

/// Print `menu` and everything under it, indented by `depth`. A `None`
/// menu means the root list, which is printed one level further out.
fn show_menus_recursive(menu: Option<Menu>, modes: c_int, depth: c_int) {
    if let Some(node) = menu {
        if !node.in_modes(modes) {
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
        put_trans(node.name(), HLF_D);
    }

    let leaf = menu.filter(|node| node.children().is_none());
    if let Some(node) = leaf {
        for bit in 0..MENU_MODES {
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
            put(MODE_CHARS[bit]);
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
            let rhs = node.strings[bit];
            match node.rhs(bit).map(CStr::is_empty) {
                Some(true) | None => put_hl(c"<Nop>", HLF_8),
                Some(false) => put_special(rhs),
            }
        }
        return;
    }

    // Recursively show the children, skipping PopUp[nvoci].
    let (start, depth) = match menu {
        Some(node) => (node.children(), depth + 1),
        None => (root_first(), depth),
    };
    for child in start.into_iter().flat_map(Menu::siblings) {
        if got_int.get() {
            break;
        }
        if !is_hidden(child.dname()) {
            show_menus_recursive(Some(child), modes, depth);
        }
    }
}

// The message entry points this listing is built from. Each hands the
// message layer a `'static` literal or a string owned by a live node.

fn put(s: &CStr) {
    // SAFETY: a NUL-terminated string that outlives the call.
    unsafe { msg_puts(s.as_ptr()) };
}

fn put_title(s: &CStr) {
    // SAFETY: as `put`.
    unsafe { msg_puts_title(s.as_ptr()) };
}

fn put_hl(s: &CStr, hl_id: c_int) {
    // SAFETY: as `put`.
    unsafe { msg_puts_hl(s.as_ptr(), hl_id, false) };
}

fn put_char(byte: u8) {
    // SAFETY: no pointers involved.
    unsafe { msg_putchar(c_int::from(byte)) };
}

fn put_num(n: c_int) {
    // SAFETY: no pointers involved.
    unsafe { msg_outnum(n) };
}

fn put_trans(s: &CStr, hl_id: c_int) {
    // SAFETY: as `put`.
    unsafe { msg_outtrans(s.as_ptr(), hl_id, false) };
}

fn put_special(s: *const c_char) {
    // SAFETY: a node's rhs, NUL-terminated and live.
    unsafe { msg_outtrans_special(s, false, 0) };
}
