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

#![forbid(unsafe_code)]

use crate::message_fmt::msg_bytes;
use crate::tr;
use core::ffi::c_int;
use std::ffi::CString;
use std::rc::Rc;

use super::*;
use crate::ascii::ascii_isdigit;
use crate::charset::getdigits_int_at;
use crate::cstr;
use crate::keycodes::{Ctrl_BSL, Ctrl_C, Ctrl_G, Ctrl_O, replace_termcodes};
use crate::mbyte::cluster_len;
use crate::menu::sys_menu;
use crate::option::vars::p_cpo;
use crate::types::ExArg;
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
pub(crate) fn ex_menu(excmd: &mut ExArg) {
    let forceit = excmd.forceit;
    let (modes, noremap, unmenu) = cmd_modes(excmd.line.cmd(), forceit);
    let ranged = (excmd.addr_count != 0 && excmd.line2 != 0).then_some(excmd.line2 as c_int);
    // A copy: the listing can stop at a `--More--` prompt.
    let text = excmd.line.arg().to_vec();
    do_menu(&text, modes, noremap, unmenu, ranged);
}

/// The byte at `i`, or a NUL past the end.
fn at(bytes: &[u8], i: usize) -> u8 {
    cstr::byte_at(bytes, i)
}

/// The body of [`ex_menu`], once the command name has been read.
fn do_menu(arg: &[u8], modes: c_int, mut noremap: c_int, unmenu: bool, ranged: Option<c_int>) {
    let mut arg = arg;
    let mut silent = false;
    loop {
        if arg.starts_with(b"<script>") {
            noremap = REMAP_SCRIPT;
            arg = skip_white(&arg[8..]);
        } else if arg.starts_with(b"<silent>") {
            silent = true;
            arg = skip_white(&arg[8..]);
        } else if arg.starts_with(b"<special>") {
            // Obsolete, and ignored.
            arg = skip_white(&arg[9..]);
        } else {
            break;
        }
    }

    arg = skip_icon(arg);
    let pri_tab = take_priorities(&mut arg, ranged);

    let mut enable = Enable::Unchanged;
    if arg.starts_with(b"enable") && white(at(arg, 6)) {
        enable = Enable::On;
        arg = skip_white(&arg[6..]);
    } else if arg.starts_with(b"disable") && white(at(arg, 7)) {
        enable = Enable::Off;
        arg = skip_white(&arg[7..]);
    }

    // No argument at all: list every menu.
    if arg.is_empty() {
        show_menus(b"", modes);
        return;
    }

    if arg[0] == b'.' {
        let path = msg_bytes(arg);
        semsg_name(tr!("E475: Invalid argument: {path}"));
        return;
    }

    let (menu_path, map_to) = menu_translate_tab_and_shift(arg);

    if map_to.is_empty() && !unmenu && enable == Enable::Unchanged {
        // Only a menu name: display the menus with that name.
        show_menus(&menu_path, modes);
        return;
    }
    if !map_to.is_empty() && (unmenu || enable != Enable::Unchanged) {
        let rest = msg_bytes(map_to);
        semsg_name(tr!("E488: Trailing characters: {rest}"));
        return;
    }

    if enable != Enable::Unchanged {
        change_sensitivity(&menu_path, modes, enable == Enable::On);
    } else if unmenu {
        if is_menus_locked() {
            return;
        }
        delete_menus(&menu_path, modes);
    } else {
        if is_menus_locked() {
            return;
        }
        add_menus(
            &menu_path,
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
/// It is parsed and dropped: upstream has never handed it to a UI.
fn skip_icon(arg: &[u8]) -> &[u8] {
    let Some(arg) = arg.strip_prefix(b"icon=") else {
        return arg;
    };
    let mut i = 0;
    while i < arg.len() && arg[i] != b' ' {
        if arg[i] == b'\\' {
            i += 1; // upstream squeezes the backslash out
        }
        i += cluster_len(&arg[i.min(arg.len())..]);
    }
    if i >= arg.len() {
        return &arg[arg.len()..];
    }
    skip_white(&arg[i + 1..])
}

/// Read the leading `10.20.30` priority list, if there is one, and advance
/// `arg` past it.
///
/// Every level not named takes [`PRIORITY_DEFAULT`], and so does a level
/// written as `0` -- or as a number too large for an `int`, since the digits
/// answer 0 for those. A `:123menu` range counts as the first level.
fn take_priorities(arg: &mut &[u8], ranged: Option<c_int>) -> [c_int; MENUDEPTH + 1] {
    let mut pri_tab = [PRIORITY_DEFAULT; MENUDEPTH + 1];
    pri_tab[MENUDEPTH] = PRIORITY_END;

    let digits = arg
        .iter()
        .take_while(|&&b| ascii_isdigit(c_int::from(b)) || b == b'.')
        .count();
    if white(at(arg, digits)) {
        for slot in pri_tab.iter_mut().take(MENUDEPTH) {
            if white(at(arg, 0)) {
                break;
            }
            let (number, past) = getdigits_int_at(arg, 0, false, 0);
            *arg = &arg[past..];
            *slot = if number == 0 {
                PRIORITY_DEFAULT
            } else {
                number
            };
            if at(arg, 0) == b'.' {
                *arg = &arg[1..];
            }
        }
        *arg = skip_white(arg);
    } else if let Some(line2) = ranged {
        pri_tab[0] = line2;
    }
    pri_tab
}

/// `:menu enable`/`:menu disable`. `*` means every menu, and the `PopUp`
/// menu is flipped one mode at a time because each mode has its own copy.
fn change_sensitivity(menu_path: &[u8], modes: c_int, enable: bool) {
    let menu_path = all_menus_if_star(menu_path);
    if is_popup(menu_path) {
        for i in 0..MENU_INDEX_TIP {
            if modes & (1 << i) != 0 {
                let name = popup_mode_name(menu_path, i);
                let done = with_tree_mut(|tree| {
                    menu_enable_recurse(&mut tree.menus, &name, MENU_ALL_MODES, enable)
                });
                if let Err(err) = done {
                    err.report();
                }
            }
        }
    }
    let done = with_tree_mut(|tree| menu_enable_recurse(&mut tree.menus, menu_path, modes, enable));
    if let Err(err) = done {
        err.report();
    }
}

/// `:unmenu`, in the same shape as [`change_sensitivity`]; the per-mode
/// `PopUp` copies are removed silently.
fn delete_menus(menu_path: &[u8], modes: c_int) {
    let menu_path = all_menus_if_star(menu_path);
    if is_popup(menu_path) {
        for i in 0..MENU_INDEX_TIP {
            if modes & (1 << i) != 0 {
                let name = popup_mode_name(menu_path, i);
                let _ = with_tree_mut(|tree| remove_menu(&mut tree.menus, &name, MENU_ALL_MODES));
            }
        }
    }
    if let Err(err) = with_tree_mut(|tree| remove_menu(&mut tree.menus, menu_path, modes)) {
        err.report();
    }
}

/// `*` as a menu path means all of them, which the walkers spell as the
/// empty name.
fn all_menus_if_star(menu_path: &[u8]) -> &[u8] {
    if menu_path == b"*" { b"" } else { menu_path }
}

/// `:menu path rhs`, and the per-mode `PopUp` copies of it.
fn add_menus(menu_path: &[u8], map_to: &[u8], arg: &MenuArg, pri_tab: &[c_int; MENUDEPTH + 1]) {
    // Replace special key codes, unless this is a tooltip (plain text) or
    // "<Nop>" (which means nothing at all).
    let translated;
    let rhs = if map_to.eq_ignore_ascii_case(b"<nop>") {
        &b""[..]
    } else if arg.modes & MENU_TIP_MODE != 0 {
        map_to
    } else {
        translated = translate_termcodes(map_to);
        &translated
    };

    add_menu_path(menu_path, arg, pri_tab, Some(rhs));
    if is_popup(menu_path) {
        for i in 0..MENU_INDEX_TIP {
            if arg.modes & (1 << i) != 0 {
                // All the modes, so that ":amenu" works.
                let name = popup_mode_name(menu_path, i);
                add_menu_path(&name, arg, pri_tab, Some(rhs));
            }
        }
    }
}

/// `replace_termcodes()`: `<C-x>` and its kind become the keys they name, up
/// to the first NUL a C reader would stop at.
fn translate_termcodes(rhs: &[u8]) -> Vec<u8> {
    let mut translated =
        p_cpo(|cpo| replace_termcodes(rhs, 0, REPTERM_DO_LT, None, cpo, None)).unwrap_or_default();
    let end = translated
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(translated.len());
    translated.truncate(end);
    translated
}

/// Store `rhs` for every mode in `modes`, replacing whatever was there.
///
/// The one copy of the rhs is shared by every mode that takes it verbatim,
/// while the modes an `:amenu` has to prefix get one string each.
fn set_rhs(menu: &mut VimMenu, modes: c_int, arg: &MenuArg, amenu: bool, call_data: Option<&[u8]>) {
    let owned = |bytes: &[u8]| Rc::<CStr>::from(CString::new(bytes).expect("an rhs holds no NUL"));
    let shared = call_data.map(owned);
    for i in 0..MENU_MODES {
        if modes & (1 << i) == 0 {
            continue;
        }
        // ":amenu" inserts the keys that get from the mode it is invoked in
        // back to Normal mode, and back again afterwards. Not for "<Nop>".
        let prefix = match call_data {
            Some(rhs) if amenu && !rhs.is_empty() => match 1 << i {
                MENU_VISUAL_MODE | MENU_SELECT_MODE | MENU_OP_PENDING_MODE | MENU_CMDLINE_MODE => {
                    Some(vec![Ctrl_C as u8])
                }
                MENU_INSERT_MODE => Some(vec![Ctrl_BSL as u8, Ctrl_O as u8]),
                _ => None,
            },
            _ => None,
        };
        menu.strings[i] = match (prefix, call_data) {
            (Some(mut buf), Some(rhs)) => {
                let restore = buf[0] == Ctrl_C as u8;
                buf.extend_from_slice(rhs);
                if restore {
                    // CTRL-C left Visual mode; CTRL-\ CTRL-G returns to it.
                    buf.extend_from_slice(&[Ctrl_BSL as u8, Ctrl_G as u8]);
                }
                Some(owned(&buf))
            }
            _ => shared.clone(),
        };
        menu.noremap[i] = arg.noremap;
        menu.silent[i] = arg.silent;
    }
}

/// The sibling list `path` (indices from the top level) leads into.
fn list_at<'a>(tree: &'a mut Vec<VimMenu>, path: &[usize]) -> &'a mut Vec<VimMenu> {
    path.iter().fold(tree, |list, &at| &mut list[at].children)
}

/// Add `menu_path` to the tree, creating every component that is missing,
/// and store `call_data` as its right-hand side.
///
/// `pri_tab` holds one priority per level: a new node is linked in after the
/// last sibling whose priority is no higher, which is what makes
/// `:menu 10.20 …` order the menubar.
fn add_menu_path(
    menu_path: &[u8],
    arg: &MenuArg,
    pri_tab: &[c_int; MENUDEPTH + 1],
    call_data: Option<&[u8]>,
) {
    let failed = with_tree_mut(|tree| {
        // The nodes walked so far, as indices from the top level.
        let mut path: Vec<usize> = Vec::new();
        let added = add_into(tree, &mut path, menu_path, arg, pri_tab, call_data);
        if added.is_err() {
            unwind_empty(&mut tree.menus, &mut path);
        }
        added.err().flatten()
    });
    if let Some(err) = failed {
        err.report();
    }
}

/// [`add_menu_path`] inside the tree's borrow, answering why it failed --
/// `Err(None)` for a failure the system menus keep quiet about.
fn add_into(
    tree: &mut MenuTree,
    path: &mut Vec<usize>,
    menu_path: &[u8],
    arg: &MenuArg,
    pri_tab: &[c_int; MENUDEPTH + 1],
    call_data: Option<&[u8]>,
) -> Result<(), Option<MenuError>> {
    let quiet = |msg| {
        if sys_menu.get() {
            None
        } else {
            Some(MenuError::Msg(msg))
        }
    };
    let mut name = menu_path;
    let mut pri_idx = 0;
    let mut old_modes = 0;
    let mut modes = arg.modes;

    while !name.is_empty() {
        // The name of this component, and the simplified name the tree is
        // also keyed by. A translation swaps the name and keeps the English
        // one alongside.
        let (component, next_name) = take_component(name);
        let (en_name, name_now) = match menutrans_lookup(&component) {
            Some(to) => (Some(component), to.into_bytes()),
            None => (None, component),
        };
        let text = menu_text(&name_now);
        if text.display.as_bytes().is_empty() {
            // Only a mnemonic or accelerator is not a name.
            return Err(Some(MenuError::Msg(c"E792: Empty menu name")));
        }

        // See if it is already there, remembering the last sibling this one
        // outranks so a new node can be spliced in after it.
        let top = path.is_empty();
        let id = tree.new_id();
        let list = list_at(&mut tree.menus, path);
        let mut insert_at = 0;
        let mut found = None;
        for (at, node) in list.iter().enumerate() {
            if name_equal(&name_now, node) || name_equal(text.display.as_bytes(), node) {
                if next_name.is_empty() && !node.children.is_empty() {
                    return Err(quiet(c"E330: Menu path must not lead to a sub-menu"));
                }
                if !next_name.is_empty() && node.children.is_empty() {
                    return Err(quiet(E_NOTSUBMENU));
                }
                found = Some(at);
                break;
            }
            // Menus outside the menubar (PopUp, ToolBar) do not take part in
            // the ordering.
            if (!top || is_menubar(node.name.as_bytes())) && node.priority <= pri_tab[pri_idx] {
                insert_at = at + 1;
            }
        }

        let at = match found {
            Some(at) => {
                let node = &mut list[at];
                old_modes = node.modes;
                // Available in this mode too now, and enabled either way.
                node.modes |= modes;
                node.enabled |= modes;
                at
            }
            None => {
                if next_name.is_empty() && top {
                    return Err(Some(MenuError::Msg(
                        c"E331: Must not add menu items directly to menu bar",
                    )));
                }
                if is_separator(text.display.as_bytes()) && !next_name.is_empty() {
                    return Err(Some(MenuError::Msg(
                        c"E332: Separator cannot be part of a menu path",
                    )));
                }

                let en_dname = en_name.as_deref().map(|en| menu_text(en).display);
                let owned = |bytes: Vec<u8>| CString::new(bytes).expect("a menu name holds no NUL");
                list.insert(
                    insert_at,
                    VimMenu {
                        id,
                        modes,
                        enabled: MENU_ALL_MODES,
                        name: owned(name_now),
                        dname: text.display,
                        en_name: en_name.map(owned),
                        en_dname,
                        mnemonic: text.mnemonic.unwrap_or(0),
                        actext: text.actext,
                        priority: pri_tab[pri_idx],
                        strings: Default::default(),
                        noremap: [0; MENU_MODES],
                        silent: [false; MENU_MODES],
                        children: Vec::new(),
                    },
                );
                old_modes = 0;
                insert_at
            }
        };
        path.push(at);
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
    if let Some((&last, parents)) = path.split_last()
        && modes != 0
    {
        let menu = &mut list_at(&mut tree.menus, parents)[last];
        set_rhs(menu, modes, arg, amenu, call_data);
    }
    Ok(())
}

/// Delete the empty submenus a failed [`add_menu_path`] created on its way
/// down, innermost first.
fn unwind_empty(tree: &mut Vec<VimMenu>, path: &mut Vec<usize>) {
    while let Some((&last, parents)) = path.split_last() {
        let list = list_at(tree, parents);
        if !list[last].children.is_empty() {
            break;
        }
        list.remove(last);
        path.pop();
    }
}
