//! Translating menu names, and describing one -- `:menutranslate` and
//! `menu_info()`.
//!
//! [`TRANSLATIONS`] is the table `:menutranslate` fills. A translation is
//! applied when a menu is *defined*, so [`menutrans_lookup`] runs from
//! `add_menu_path` and the English name is kept alongside the translated one
//! -- which is why `:menutranslate clear` cannot rename anything back.
//! [`menu_translate_tab_and_shift`] rewrites `<Tab>` in a name before it is
//! parsed at all. [`menuitem_getinfo`] and [`f_menu_info`] build the Dict
//! `menu_info()` answers with.
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

use core::ffi::c_int;
use std::ffi::CString;

use super::*;
use crate::eval::typval::{NumBuf, tv_dict_alloc_ret};
use crate::eval::vars::del_menutrans_vars;
use crate::ex_docmd::ends_excmd;
use crate::global_cell::GlobalCell;
use crate::keycodes::Ctrl_V;
use crate::mbyte::cluster_len;
use crate::message::e_invarg;
use crate::types::{Dict, EvalFuncData, ExArg, TypVal, VarNumber};

/// One `:menutranslate from to` entry.
struct Translation {
    /// The English name, as typed.
    from: CString,
    /// The same, with the `&` mnemonic markers taken out.
    from_noamp: CString,
    /// What a menu of that name is really called.
    to: CString,
}

/// The translation table, in the order the entries were added -- the first
/// match wins.
static TRANSLATIONS: GlobalCell<Vec<Translation>> = GlobalCell::new(Vec::new());

/// `:menutranslate from to`, and `:menutranslate clear`.
pub(crate) fn ex_menutranslate(excmd: &mut ExArg) {
    let arg = excmd.line.arg();

    if let Some(rest) = arg.strip_prefix(b"clear")
        && ends_of_command(skip_white(rest))
    {
        TRANSLATIONS.with_mut(Vec::clear);
        // And every "menutrans_" global variable with them.
        clear_menutrans_vars();
        return;
    }

    let from_end = menu_skip_part(arg);
    let to = skip_white(&arg[from_end..]);
    let to_end = menu_skip_part(to);
    if to_end == 0 {
        emsg_shared(e_invarg);
        return;
    }
    let (from, to) = (&arg[..from_end], &to[..to_end]);

    // The `&`-free form is taken before the name is unescaped, so that
    // `menutrans_lookup`'s second pass compares like with like.
    let from_noamp = menu_text(from).display;

    let from = menu_unescape_name(&menu_translate_tab_and_shift(from).0);
    let to = menu_unescape_name(&menu_translate_tab_and_shift(to).0);

    let owned = |bytes: Vec<u8>| CString::new(bytes).expect("a menu name holds no NUL");
    TRANSLATIONS.with_mut(|table| {
        table.push(Translation {
            from: owned(from),
            from_noamp,
            to: owned(to),
        });
    });
}

/// How long the part of a menu name at the start of `p` is: up to a `.` or
/// white space that is not escaped.
fn menu_skip_part(p: &[u8]) -> usize {
    let mut i = 0;
    while i < p.len() && p[i] != b'.' && !ascii_iswhite(c_int::from(p[i])) {
        let escaped = p[i] == b'\\' || c_int::from(p[i]) == Ctrl_V;
        if escaped && i + 1 < p.len() {
            i += 1;
        }
        i += 1;
    }
    i
}

/// What `name` has been translated to, if anything.
///
/// Matched twice: against the name as typed, then -- so that `&F&ile` finds a
/// translation registered for `File` -- against the name with its mnemonic
/// markers removed. Both compares ignore case.
pub(crate) fn menutrans_lookup(name: &[u8]) -> Option<CString> {
    let direct = TRANSLATIONS.with(|table| {
        table
            .iter()
            .find(|entry| entry.from.as_bytes().eq_ignore_ascii_case(name))
            .map(|entry| entry.to.clone())
    });
    if direct.is_some() {
        return direct;
    }

    let dname = menu_text(name).display;
    TRANSLATIONS.with(|table| {
        table
            .iter()
            .find(|entry| {
                entry
                    .from_noamp
                    .as_bytes()
                    .eq_ignore_ascii_case(dname.as_bytes())
            })
            .map(|entry| entry.to.clone())
    })
}

/// A translation table name with its `\`-escapes taken out -- up to its
/// first `.`, as upstream's walk stops there.
fn menu_unescape_name(name: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(name.len());
    let mut i = 0;
    while i < name.len() && name[i] != b'.' {
        if name[i] == b'\\' {
            i += 1;
        }
        let len = cluster_len(&name[i..]);
        out.extend_from_slice(&name[i..i + len]);
        i += len;
    }
    out.extend_from_slice(&name[i..]);
    out
}

/// Isolate the menu name at the start of `arg`, turning `<Tab>` into a real
/// TAB, and answer it with what follows it.
///
/// This is what makes the accelerator separator a TAB byte: a `\t` in a
/// `:menu` argument is a backslash escaping the letter `t` and stays two
/// characters.
pub(crate) fn menu_translate_tab_and_shift(arg: &[u8]) -> (Vec<u8>, &[u8]) {
    let mut name = Vec::with_capacity(arg.len());
    let mut i = 0;
    while i < arg.len() && !ascii_iswhite(c_int::from(arg[i])) {
        let escaped = arg[i] == b'\\' || c_int::from(arg[i]) == Ctrl_V;
        if escaped && i + 1 < arg.len() {
            name.extend_from_slice(&arg[i..i + 2]);
            i += 2;
        } else if arg[i..]
            .get(..5)
            .is_some_and(|head| head.eq_ignore_ascii_case(b"<TAB>"))
        {
            name.push(b'\t');
            i += 5;
        } else {
            name.push(arg[i]);
            i += 1;
        }
    }
    let rest = arg.get(i + 1..).unwrap_or_default();
    (name, skip_white(rest))
}

/// Describe one menu item, or -- for an empty name -- list the top-level
/// menus `tree` holds.
fn menuitem_getinfo(
    menu_name: &[u8],
    menu: &VimMenu,
    tree: &[VimMenu],
    modes: c_int,
    dict: &mut Dict,
) {
    if menu_name.is_empty() {
        // All the top-level menus, skipping PopUp[nvoci].
        let mut list = list_alloc();
        for top in tree {
            if !is_hidden(top.dname.as_bytes()) {
                list.push_str(Some(&top.dname));
            }
        }
        let _ = dict.add_list(b"submenus", Some(list));
        return;
    }

    let _ = dict.add_str(b"name", Some(&menu.name));
    let _ = dict.add_str(b"display", Some(&menu.dname));
    if let Some(accel) = &menu.actext {
        let _ = dict.add_str(b"accel", Some(accel));
    }
    let _ = dict.add_number(b"priority", VarNumber::from(menu.priority));
    let _ = dict.add_str(b"modes", Some(menu_mode_str(menu.modes)));
    let _ = dict.add_str(b"shortcut", Some(&char_as_text(menu.mnemonic)));

    if menu.children.is_empty() {
        // A leaf: describe the first mode it is available in. There is
        // always one, but Coverity does not know that.
        let Some(bit) = (0..MENU_MODES).find(|bit| (1 << bit) & modes != 0) else {
            return;
        };
        if let Some(rhs) = menu.strings[bit].as_deref() {
            let text = if rhs.is_empty() {
                ThinCString::from_bytes(b"<Nop>")
            } else {
                special_text(rhs)
            };
            let _ = dict.add_allocated_str(b"rhs", Some(text));
        }
        let flag = |on: bool| crate::types::BoolVarValue::from(on);
        let _ = dict.add_bool(b"noremenu", flag(menu.noremap[bit] == REMAP_NONE));
        let _ = dict.add_bool(b"script", flag(menu.noremap[bit] == REMAP_SCRIPT));
        let _ = dict.add_bool(b"silent", flag(menu.silent[bit]));
        let _ = dict.add_bool(b"enabled", flag(menu.enabled & (1 << bit) != 0));
        return;
    }

    // Otherwise all the submenu display names.
    let mut list = list_alloc();
    for child in &menu.children {
        list.push_str(Some(&child.dname));
    }
    let _ = dict.add_list(b"submenus", Some(list));
}

/// `menu_info({name} [, {mode}])`: everything known about a menu, its child
/// menus included.
pub(crate) fn f_menu_info(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    tv_dict_alloc_ret(result);
    let Some(menu_name) = numbuf.string_chk(&args[0]) else {
        // Before the second argument is looked at: `tv_get_string_chk`
        // answers a shared scratch buffer, so converting one argument can
        // invalidate the other, and a bad first argument must report once.
        return;
    };
    let which = match args.get(1) {
        Some(second) => numbuf2.string_chk(second),
        // The default is the modes of plain ":menu".
        None => Some(c""),
    };
    let Some(which) = which else {
        return;
    };

    let modes = cmd_modes(which.to_bytes(), which.to_bytes().first() == Some(&b'!')).0;
    let Some(dict) = result.dict_mut() else {
        return;
    };
    let menu_name = menu_name.to_bytes();
    with_tree(|tree| {
        if let Some(menu) = find_by_name(tree, menu_name)
            && menu.modes & modes != 0
        {
            menuitem_getinfo(menu_name, menu, tree, modes, dict);
        }
    });
}

/// Locate the menu or menu item `menu_name` names, silently.
///
/// An empty name answers the first top-level menu, which is what
/// [`menuitem_getinfo`] walks to list them all.
fn find_by_name<'a>(tree: &'a [VimMenu], menu_name: &[u8]) -> Option<&'a VimMenu> {
    let mut list = tree;
    if menu_name.is_empty() {
        return list.first();
    }
    let mut name = menu_name;
    loop {
        let (component, rest) = take_component(name);
        let menu = list.iter().find(|node| name_equal(&component, node));
        match menu {
            Some(node) if !rest.is_empty() => {
                list = &node.children;
                name = rest;
            }
            found => return found,
        }
    }
}

fn ends_of_command(p: &[u8]) -> bool {
    ends_excmd(c_int::from(crate::cstr::byte_at(p, 0))) != 0
}

fn clear_menutrans_vars() {
    del_menutrans_vars();
}
