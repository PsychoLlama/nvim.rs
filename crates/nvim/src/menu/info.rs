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

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_int};
use std::ffi::CString;

use super::*;
use crate::ascii::ascii_iswhite;
use crate::eval::typval::{tv_dict_alloc_ret, tv_get_string_chk};
use crate::eval::vars::del_menutrans_vars;
use crate::ex_docmd::ends_excmd;
use crate::global_cell::GlobalCell;
use crate::keycodes::Ctrl_V;
use crate::main::e_invarg;
use crate::types::{EvalFuncData, VAR_UNKNOWN, dict_T, exarg_T, typval_T};

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
///
/// # Safety
/// `eap` must name the live `exarg_T` of the command.
pub unsafe fn ex_menutranslate(eap: *mut exarg_T) {
    // SAFETY: the caller's obligation; `arg` names the command line, which
    // this takes apart in place.
    let arg = unsafe { CText::new((*eap).arg) };

    if arg.starts_with(b"clear") && ends_of_command(skip_white(arg.at(5))) {
        TRANSLATIONS.with_mut(Vec::clear);
        // And every "menutrans_" global variable with them.
        clear_menutrans_vars();
        return;
    }

    let from = arg;
    let from_end = menu_skip_part(arg);
    let to = skip_white(from_end);
    from_end.set(0, 0);
    let to_end = menu_skip_part(to);
    if to_end.same(to) {
        emsg_shared(&e_invarg);
        return;
    }

    // The `&`-free form is taken before the name is unescaped, so that
    // `menutrans_lookup`'s second pass compares like with like.
    let from_noamp = menu_text(from.as_cstr()).display;

    let mut from_buf = scratch(from.as_cstr());
    let from = text_of(&mut from_buf);
    let mut to_buf = scratch_bytes(&to.bytes()[..to_end.offset_from(to)]);
    let to = text_of(&mut to_buf);

    menu_translate_tab_and_shift(from);
    menu_translate_tab_and_shift(to);
    menu_unescape_name(from);
    menu_unescape_name(to);

    TRANSLATIONS.with_mut(|table| {
        table.push(Translation {
            from: from.as_cstr().to_owned(),
            from_noamp,
            to: to.as_cstr().to_owned(),
        });
    });
}

/// The character just after one part of a menu name.
fn menu_skip_part(p: CText) -> CText {
    let mut i = 0;
    while p.byte(i) != 0 && p.byte(i) != b'.' && !ascii_iswhite(c_int::from(p.byte(i))) {
        if (p.byte(i) == b'\\' || p.byte(i) == Ctrl_V as u8) && p.byte(i + 1) != 0 {
            i += 1;
        }
        i += 1;
    }
    p.at(i)
}

/// What `name` has been translated to, if anything.
///
/// Matched twice: against the name as typed, then -- so that `&F&ile` finds a
/// translation registered for `File` -- against the name with its mnemonic
/// markers removed. Both compares ignore case.
pub(crate) fn menutrans_lookup(name: CText) -> Option<CString> {
    let direct = TRANSLATIONS.with(|table| {
        table
            .iter()
            .find(|entry| entry.from.as_bytes().eq_ignore_ascii_case(name.bytes()))
            .map(|entry| entry.to.clone())
    });
    if direct.is_some() {
        return direct;
    }

    let dname = menu_text(name.as_cstr()).display;
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

/// Take the `\`-escapes out of a translation table name.
fn menu_unescape_name(name: CText) {
    let mut i = 0;
    while name.byte(i) != 0 && name.byte(i) != b'.' {
        if name.byte(i) == b'\\' {
            name.squeeze(i, 1);
        }
        i += name.char_len(i);
    }
}

/// Isolate the menu name at `arg_start`, turning `<Tab>` into a real TAB,
/// and answer what follows it.
///
/// This is what makes the accelerator separator a TAB byte: a `\t` in a
/// `:menu` argument is a backslash escaping the letter `t` and stays two
/// characters.
pub(crate) fn menu_translate_tab_and_shift(arg_start: CText) -> CText {
    let mut i = 0;
    while arg_start.byte(i) != 0 && !ascii_iswhite(c_int::from(arg_start.byte(i))) {
        let escaped = arg_start.byte(i) == b'\\' || arg_start.byte(i) == Ctrl_V as u8;
        if escaped && arg_start.byte(i + 1) != 0 {
            i += 1;
        } else if arg_start
            .at(i)
            .bytes()
            .get(..5)
            .is_some_and(|head| head.eq_ignore_ascii_case(b"<TAB>"))
        {
            arg_start.set(i, b'\t');
            arg_start.squeeze(i + 1, 4);
        }
        i += 1;
    }
    if arg_start.byte(i) != 0 {
        arg_start.set(i, 0);
        i += 1;
    }
    skip_white(arg_start.at(i))
}

/// Describe one menu item, or -- for an empty name -- list the top-level
/// menus.
fn menuitem_getinfo(menu_name: &CStr, menu: Menu, modes: c_int, dict: *mut dict_T) {
    if menu_name.is_empty() {
        // All the top-level menus, skipping PopUp[nvoci].
        let list = list_alloc();
        dict_add_list(dict, c"submenus", list);
        for top in menu.siblings() {
            if !is_hidden(top.dname()) {
                list_append_str(list, top.dname());
            }
        }
        return;
    }

    dict_add_str(dict, c"name", menu.name());
    dict_add_str(dict, c"display", menu.dname());
    if let Some(accel) = menu.actext() {
        dict_add_str(dict, c"accel", accel);
    }
    dict_add_nr(dict, c"priority", varnumber_T::from(menu.priority));
    dict_add_str(dict, c"modes", menu_mode_str(menu.modes));
    dict_add_str(dict, c"shortcut", &char_as_text(menu.mnemonic));

    let Some(children) = menu.children() else {
        // A leaf: describe the first mode it is available in. There is
        // always one, but Coverity does not know that.
        let Some(bit) = (0..MENU_MODES).find(|bit| (1 << bit) & modes != 0) else {
            return;
        };
        if let Some(rhs) = menu.rhs(bit) {
            let text = if rhs.is_empty() {
                dup(c"<Nop>")
            } else {
                special_text(rhs.as_ptr())
            };
            dict_add_allocated_str(dict, c"rhs", text);
        }
        dict_add_bool(dict, c"noremenu", menu.noremap[bit] == REMAP_NONE);
        dict_add_bool(dict, c"script", menu.noremap[bit] == REMAP_SCRIPT);
        dict_add_bool(dict, c"silent", menu.silent[bit]);
        dict_add_bool(dict, c"enabled", menu.enabled & (1 << bit) != 0);
        return;
    };

    // Otherwise all the submenu display names.
    let list = list_alloc();
    dict_add_list(dict, c"submenus", list);
    for child in children.siblings() {
        list_append_str(list, child.dname());
    }
}

/// `menu_info({name} [, {mode}])`: everything known about a menu, its child
/// menus included.
///
/// # Safety
/// The eval layer must pass live argument and return typvals.
pub unsafe extern "C" fn f_menu_info(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the caller's obligation.
    let (retdict, menu_name) = unsafe {
        tv_dict_alloc_ret(rettv);
        ((*rettv).vval.v_dict, tv_get_string_chk(argvars))
    };
    if menu_name.is_null() {
        // Before the second argument is looked at: `tv_get_string_chk`
        // answers a shared scratch buffer, so converting one argument can
        // invalidate the other, and a bad first argument must report once.
        return;
    }
    // SAFETY: the caller's obligation; the second argument if there is one.
    let which = unsafe {
        let second = argvars.add(1);
        if (*second).v_type != VAR_UNKNOWN {
            tv_get_string_chk(second)
        } else {
            // The default is the modes of plain ":menu".
            c"".as_ptr()
        }
    };
    if which.is_null() {
        return;
    }
    // SAFETY: `tv_get_string_chk` answers a NUL-terminated string or null.
    let (menu_name, which) = unsafe { (CStr::from_ptr(menu_name), CStr::from_ptr(which)) };

    let modes = cmd_modes(which.to_bytes(), which.to_bytes().first() == Some(&b'!')).0;
    if let Some(menu) = find_by_name(menu_name)
        && menu.in_modes(modes)
    {
        menuitem_getinfo(menu_name, menu, modes, retdict);
    }
}

/// Locate the menu or menu item `menu_name` names, silently.
///
/// An empty name answers the first top-level menu, which is what
/// [`menuitem_getinfo`] walks to list them all.
fn find_by_name(menu_name: &CStr) -> Option<Menu> {
    let mut menu = root_first();
    if menu_name.is_empty() {
        return menu;
    }
    let mut buf = scratch(menu_name);
    let mut name = text_of(&mut buf);
    while !name.is_empty() {
        let rest = skip_component(name);
        menu = menu
            .into_iter()
            .flat_map(Menu::siblings)
            .find(|node| name_equal(name.as_cstr(), *node));
        let Some(node) = menu.filter(|_| !rest.is_empty()) else {
            break;
        };
        menu = node.children();
        name = rest;
    }
    menu
}

fn ends_of_command(p: CText) -> bool {
    ends_excmd(c_int::from(p.byte(0))) != 0
}

fn clear_menutrans_vars() {
    // SAFETY: walks the global variable dict, which is always live.
    unsafe { del_menutrans_vars() };
}
