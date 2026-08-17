//! Defining a menu -- `:menu` and everything it parses.
//!
//! [`ex_menu`] is the whole command line: the `<silent>`/`<script>`/`<special>`
//! modifiers, the `80.5.10` priority, the mode prefix, the `\ `-escaped path
//! and the right-hand side, plus the `:unmenu` and `:menu`-as-a-listing forms.
//! [`add_menu_path`] then walks the path one component at a time, creating the
//! [`Menu`] nodes that do not exist yet, inserting each at the place its
//! priority asks for, and finally storing the rhs for every mode the command
//! named.
//!
//! Original: `src/nvim/menu.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use super::*;
use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::charset::getdigits_int;
use crate::keycodes::{Ctrl_BSL, Ctrl_C, Ctrl_G, Ctrl_O, replace_termcodes};
use crate::main::{e_invarg2, e_trailing_arg, p_cpo, sys_menu};
use crate::memory::xcalloc;
use crate::types::{exarg_T, vimmenu_T};
use crate::ui::ui_call_update_menu;

/// The default priority: what an unnumbered component gets, and what a
/// literal `0` becomes.
const PRIORITY_DEFAULT: c_int = 500;

/// One past the last priority component, so `add_menu_path` knows when to
/// stop descending the table.
const PRIORITY_END: c_int = -1;

/// Whether `:menu` was asked to enable, disable, or neither.
#[derive(Clone, Copy, PartialEq)]
enum Enable {
    Unchanged,
    On,
    Off,
}

/// The mode-independent half of a `:menu` definition.
struct MenuArg {
    modes: c_int,
    noremap: c_int,
    silent: bool,
}

/// `:menu` and its forty-odd relatives -- every mode prefix, `:noremenu`,
/// `:unmenu` and `:menu enable`/`disable`.
///
/// # Safety
/// `eap` must name the live `exarg_T` of a menu command, whose `arg` points
/// into the command line this rewrites in place.
pub unsafe fn ex_menu(eap: *mut exarg_T) {
    // SAFETY: the caller's obligation. `cmd` and `arg` name the command line,
    // which `ex_docmd` lets a command edit.
    let (cmd, arg, forceit, ranged) = unsafe {
        let eap = &*eap;
        (
            CStr::from_ptr(eap.cmd),
            CText::new(eap.arg),
            eap.forceit != 0,
            (eap.addr_count != 0 && eap.line2 != 0).then_some(eap.line2 as c_int),
        )
    };
    let (modes, noremap, unmenu) = cmd_modes(cmd.to_bytes(), forceit);
    do_menu(arg, modes, noremap, unmenu, ranged);
}

/// The body of [`ex_menu`], once the command name has been read.
fn do_menu(mut arg: CText, modes: c_int, mut noremap: c_int, unmenu: bool, ranged: Option<c_int>) {
    let mut silent = false;
    loop {
        if arg.starts_with(b"<script>") {
            noremap = REMAP_SCRIPT;
            arg = skip_white(arg.at(8));
        } else if arg.starts_with(b"<silent>") {
            silent = true;
            arg = skip_white(arg.at(8));
        } else if arg.starts_with(b"<special>") {
            // Obsolete, and ignored.
            arg = skip_white(arg.at(9));
        } else {
            break;
        }
    }

    arg = skip_icon(arg);
    let pri_tab = take_priorities(&mut arg, ranged);

    let mut enable = Enable::Unchanged;
    if arg.starts_with(b"enable") && white(arg.byte(6)) {
        enable = Enable::On;
        arg = skip_white(arg.at(6));
    } else if arg.starts_with(b"disable") && white(arg.byte(7)) {
        enable = Enable::Off;
        arg = skip_white(arg.at(7));
    }

    // No argument at all: list every menu.
    if arg.is_empty() {
        show_menus(c"", modes);
        return;
    }

    let menu_path = arg;
    if menu_path.byte(0) == b'.' {
        semsg_name(message(&e_invarg2), menu_path.raw());
        return;
    }

    // Careful from here on: the name is NUL-terminated in place, and the
    // three consumers below all edit it further.
    let map_to = menu_translate_tab_and_shift(arg);

    if map_to.is_empty() && !unmenu && enable == Enable::Unchanged {
        // Only a menu name: display the menus with that name.
        show_menus(menu_path.as_cstr(), modes);
        return;
    }
    if !map_to.is_empty() && (unmenu || enable != Enable::Unchanged) {
        semsg_name(message(&e_trailing_arg), map_to.raw());
        return;
    }

    if enable != Enable::Unchanged {
        change_sensitivity(menu_path, modes, enable == Enable::On);
    } else if unmenu {
        if is_menus_locked() {
            return;
        }
        delete_menus(menu_path, modes);
    } else {
        if is_menus_locked() {
            return;
        }
        add_menus(
            menu_path,
            map_to,
            &MenuArg {
                modes,
                noremap,
                silent,
            },
            &pri_tab,
        );
    }

    ui_call_update_menu();
}

fn white(byte: u8) -> bool {
    ascii_iswhite(c_int::from(byte))
}

/// Step over an optional `icon=filename` argument.
///
/// It is parsed and dropped: upstream has never handed it to a UI, and the
/// `\`-escapes are still squeezed out of the command line on the way past.
fn skip_icon(arg: CText) -> CText {
    if !arg.starts_with(b"icon=") {
        return arg;
    }
    let arg = arg.at(5);
    let mut i = 0;
    while arg.byte(i) != 0 && arg.byte(i) != b' ' {
        if arg.byte(i) == b'\\' {
            arg.squeeze(i, 1);
        }
        i += arg.char_len(i);
    }
    if arg.byte(i) == 0 {
        return arg.at(i);
    }
    arg.set(i, 0);
    skip_white(arg.at(i + 1))
}

/// Read the leading `10.20.30` priority list, if there is one, and advance
/// `arg` past it.
///
/// Every level not named takes [`PRIORITY_DEFAULT`], and so does a level
/// written as `0` -- or as a number too large for an `int`, since
/// [`take_digits`] answers 0 for those. A `:123menu` range counts as the
/// first level.
fn take_priorities(arg: &mut CText, ranged: Option<c_int>) -> [c_int; MENUDEPTH + 1] {
    let mut pri_tab = [PRIORITY_DEFAULT; MENUDEPTH + 1];
    pri_tab[MENUDEPTH] = PRIORITY_END;

    let mut i = 0;
    while arg.byte(i) != 0 && (ascii_isdigit(c_int::from(arg.byte(i))) || arg.byte(i) == b'.') {
        i += 1;
    }
    if white(arg.byte(i)) {
        for slot in pri_tab.iter_mut().take(MENUDEPTH) {
            if white(arg.byte(0)) {
                break;
            }
            *slot = take_digits(arg);
            if *slot == 0 {
                *slot = PRIORITY_DEFAULT;
            }
            if arg.byte(0) == b'.' {
                *arg = arg.at(1);
            }
        }
        *arg = skip_white(*arg);
    } else if let Some(line2) = ranged {
        pri_tab[0] = line2;
    }
    pri_tab
}

/// `:menu enable`/`:menu disable`. `*` means every menu, and the `PopUp`
/// menu is flipped one mode at a time because each mode has its own copy.
fn change_sensitivity(menu_path: CText, modes: c_int, enable: bool) {
    let menu_path = all_menus_if_star(menu_path);
    if is_popup(menu_path.as_cstr()) {
        for i in 0..MENU_INDEX_TIP {
            if modes & (1 << i) != 0 {
                let mut buf = scratch(&popup_mode_name(menu_path.as_cstr(), i));
                menu_enable_recurse(root_first(), text_of(&mut buf), MENU_ALL_MODES, enable);
            }
        }
    }
    // Careful: menu_enable_recurse() edits menu_path.
    menu_enable_recurse(root_first(), menu_path, modes, enable);
}

/// `:unmenu`, in the same shape as [`change_sensitivity`].
fn delete_menus(menu_path: CText, modes: c_int) {
    let menu_path = all_menus_if_star(menu_path);
    if is_popup(menu_path.as_cstr()) {
        for i in 0..MENU_INDEX_TIP {
            if modes & (1 << i) != 0 {
                let mut buf = scratch(&popup_mode_name(menu_path.as_cstr(), i));
                remove_menu(root_link(), text_of(&mut buf), MENU_ALL_MODES, true);
            }
        }
    }
    // Careful: remove_menu() edits menu_path.
    remove_menu(root_link(), menu_path, modes, false);
}

/// `*` as a menu path means all of them, which the walkers spell as the
/// empty name. Truncating in place is exactly C's `menu_path = ""`.
fn all_menus_if_star(menu_path: CText) -> CText {
    if menu_path.as_cstr() == c"*" {
        menu_path.set(0, 0);
    }
    menu_path
}

/// `:menu path rhs`, and the per-mode `PopUp` copies of it.
fn add_menus(menu_path: CText, map_to: CText, arg: &MenuArg, pri_tab: &[c_int; MENUDEPTH + 1]) {
    // Replace special key codes, unless this is a tooltip (plain text) or
    // "<Nop>" (which means nothing at all).
    let mut map_buf: *mut c_char = ptr::null_mut();
    let rhs = if map_to.bytes().eq_ignore_ascii_case(b"<nop>") {
        c""
    } else if arg.modes & MENU_TIP_MODE != 0 {
        map_to.as_cstr()
    } else {
        translate_termcodes(map_to.as_cstr(), &mut map_buf)
    };

    add_menu_path(menu_path.as_cstr(), arg, pri_tab, Some(rhs));
    if is_popup(menu_path.as_cstr()) {
        for i in 0..MENU_INDEX_TIP {
            if arg.modes & (1 << i) != 0 {
                // All the modes, so that ":amenu" works.
                let name = popup_mode_name(menu_path.as_cstr(), i);
                add_menu_path(&name, arg, pri_tab, Some(rhs));
            }
        }
    }
    free_str(map_buf);
}

/// `replace_termcodes()`: `<C-x>` and its kind become the keys they name.
/// The answer borrows the buffer left in `owner`, which the caller frees.
fn translate_termcodes<'a>(rhs: &'a CStr, owner: &mut *mut c_char) -> &'a CStr {
    // SAFETY: `rhs` is NUL-terminated and `owner` names a live slot;
    // `replace_termcodes` allocates into it and answers a pointer into it.
    unsafe {
        let translated = replace_termcodes(
            rhs.as_ptr(),
            rhs.count_bytes(),
            owner,
            0,
            REPTERM_DO_LT,
            ptr::null_mut(),
            p_cpo.get(),
        );
        CStr::from_ptr(translated)
    }
}

/// `getdigits_int(&p, false, 0)`: the number `p` starts with, advancing `p`
/// past it. A value too big for an `int` answers 0, which the caller turns
/// into [`PRIORITY_DEFAULT`] -- which is why this is not a plain parse.
fn take_digits(p: &mut CText) -> c_int {
    let mut raw = p.raw();
    // SAFETY: `raw` names a live NUL-terminated buffer and `getdigits_int`
    // only advances it over that string's own digits.
    let digits = unsafe { getdigits_int(&raw mut raw, false, 0) };
    // SAFETY: still inside the same buffer.
    *p = unsafe { CText::new(raw) };
    digits
}

/// A fresh zeroed node, as C's `xcalloc(1, sizeof(vimmenu_T))` gives it.
/// The caller fills `name` and `dname` before the node is linked in, which
/// is what completes [`Menu`]'s invariant.
fn alloc_node() -> Menu {
    // SAFETY: `xcalloc` never answers null and zeroes the whole struct.
    unsafe { Menu::new(xcalloc(1, size_of::<vimmenu_T>()) as *mut vimmenu_T) }
}

/// Store `rhs` for every mode in `modes`, freeing whatever was there.
///
/// The one copy of the rhs is shared by every mode that takes it verbatim --
/// `free_menu_string` counts the sharers before freeing, which is what makes
/// that safe -- while the modes an `:amenu` has to prefix get one buffer
/// each.
fn set_rhs(mut menu: Menu, modes: c_int, arg: &MenuArg, amenu: bool, call_data: Option<&CStr>) {
    let shared = call_data.map_or(ptr::null_mut(), dup);
    for i in 0..MENU_MODES {
        if modes & (1 << i) == 0 {
            continue;
        }
        free_menu_string(menu, i);
        // ":amenu" inserts the keys that get from the mode it is invoked in
        // back to Normal mode, and back again afterwards. Not for "<Nop>".
        let prefix = match call_data.map(CStr::to_bytes) {
            Some(rhs) if amenu && !rhs.is_empty() => match 1 << i {
                MENU_VISUAL_MODE | MENU_SELECT_MODE | MENU_OP_PENDING_MODE | MENU_CMDLINE_MODE => {
                    Some(vec![Ctrl_C as u8])
                }
                MENU_INSERT_MODE => Some(vec![Ctrl_BSL as u8, Ctrl_O as u8]),
                _ => None,
            },
            _ => None,
        };
        menu.strings[i] = match prefix {
            None => shared,
            Some(mut buf) => {
                let restore = buf[0] == Ctrl_C as u8;
                buf.extend_from_slice(call_data.expect("prefixed only with an rhs").to_bytes());
                if restore {
                    // CTRL-C left Visual mode; CTRL-\ CTRL-G returns to it.
                    buf.extend_from_slice(&[Ctrl_BSL as u8, Ctrl_G as u8]);
                }
                dup_bytes(&buf)
            }
        };
        menu.noremap[i] = arg.noremap;
        menu.silent[i] = arg.silent;
    }
}

/// Add `menu_path` to the tree, creating every component that is missing,
/// and store `call_data` as its right-hand side.
///
/// `pri_tab` holds one priority per level: a new node is linked in after the
/// last sibling whose priority is no higher, which is what makes
/// `:menu 10.20 …` order the menubar.
fn add_menu_path(
    menu_path: &CStr,
    arg: &MenuArg,
    pri_tab: &[c_int; MENUDEPTH + 1],
    call_data: Option<&CStr>,
) {
    let mut buf = scratch(menu_path);
    let mut name = text_of(&mut buf);
    let mut menup = root_link();
    let mut parent: Option<Menu> = None;
    let mut menu: Option<Menu> = None;
    let mut pri_idx = 0;
    let mut old_modes = 0;
    let mut modes = arg.modes;

    while !name.is_empty() {
        // The name of this component, and the simplified name the tree is
        // also keyed by. A translation swaps the name and keeps the English
        // one alongside.
        let next_name = skip_component(name);
        let translated = menutrans_lookup(name);
        let (en_name, name_now) = match translated.as_deref() {
            Some(to) => (Some(name.as_cstr().to_owned()), to.to_owned()),
            None => (None, name.as_cstr().to_owned()),
        };
        let text = menu_text(&name_now);
        if text.display.as_bytes().is_empty() {
            // Only a mnemonic or accelerator is not a name.
            emsg_c(c"E792: Empty menu name");
            return unwind_empty(parent);
        }

        // See if it is already there, remembering the last sibling this one
        // outranks so a new node can be spliced in after it.
        let mut lower_pri = menup;
        menu = None;
        let mut cursor = menup.get();
        while let Some(node) = cursor {
            if name_equal(&name_now, node) || name_equal(&text.display, node) {
                if next_name.is_empty() && node.children().is_some() {
                    if !sys_menu.get() {
                        emsg_c(c"E330: Menu path must not lead to a sub-menu");
                    }
                    return unwind_empty(parent);
                }
                if !next_name.is_empty() && node.children().is_none() {
                    if !sys_menu.get() {
                        emsg_c(E_NOTSUBMENU);
                    }
                    return unwind_empty(parent);
                }
                menu = Some(node);
                break;
            }
            menup = node.next_link();
            // Menus outside the menubar (PopUp, ToolBar) do not take part in
            // the ordering.
            if (parent.is_some() || is_menubar(node.name())) && node.priority <= pri_tab[pri_idx] {
                lower_pri = menup;
            }
            cursor = node.next();
        }

        let node = match menu {
            Some(node) => {
                old_modes = node.modes;
                // Available in this mode too now, and enabled either way.
                let mut node = node;
                node.modes |= modes;
                node.enabled |= modes;
                node
            }
            None => {
                if next_name.is_empty() && parent.is_none() {
                    emsg_c(c"E331: Must not add menu items directly to menu bar");
                    return unwind_empty(parent);
                }
                if is_separator(&text.display) && !next_name.is_empty() {
                    emsg_c(c"E332: Separator cannot be part of a menu path");
                    return unwind_empty(parent);
                }

                let mut node = alloc_node();
                node.modes = modes;
                node.enabled = MENU_ALL_MODES;
                node.name = dup(&name_now);
                node.dname = dup(&text.display);
                node.mnemonic = text.mnemonic.unwrap_or(0);
                node.actext = text.actext.as_deref().map_or(ptr::null_mut(), dup);
                if let Some(en_name) = &en_name {
                    node.en_name = dup(en_name);
                    node.en_dname = dup(&menu_text(en_name).display);
                }
                node.priority = pri_tab[pri_idx];
                node.parent = parent.map_or(ptr::null_mut(), Menu::raw);
                // Insert after the last sibling of no higher priority.
                node.next = lower_pri.get().map_or(ptr::null_mut(), Menu::raw);
                lower_pri.set(Some(node));
                old_modes = 0;
                node
            }
        };
        menu = Some(node);
        menup = node.children_link();
        parent = Some(node);
        name = next_name;
        if pri_tab[pri_idx + 1] != PRIORITY_END {
            pri_idx += 1;
        }
    }

    // Was this an ":amenu"? Only then does the rhs get a mode-switching
    // prefix.
    let amenu =
        modes & (MENU_NORMAL_MODE | MENU_INSERT_MODE) == MENU_NORMAL_MODE | MENU_INSERT_MODE;
    if sys_menu.get() {
        // Only add system menu items that have not been defined yet.
        modes &= !old_modes;
    }
    if let Some(menu) = menu.filter(|_| modes != 0) {
        set_rhs(menu, modes, arg, amenu, call_data);
    }
}

/// Delete the empty submenus a failed [`add_menu_path`] created on its way
/// down, innermost first.
fn unwind_empty(mut parent: Option<Menu>) {
    while let Some(node) = parent.filter(|node| node.children().is_none()) {
        let mut menup = match node.parent() {
            Some(grandparent) => grandparent.children_link(),
            None => root_link(),
        };
        while let Some(sibling) = menup.get() {
            if sibling.same(node) {
                break;
            }
            menup = sibling.next_link();
        }
        if menup.get().is_none() {
            // Safety check: not in the list its parent names.
            break;
        }
        parent = node.parent();
        free_menu(menup);
    }
}
