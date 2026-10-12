//! Menu names as text -- parsing a path, matching one, and the mode
//! letters.
//!
//! [`take_component`] takes one `\`-escaped path component apart;
//! [`name_equal`] compares a component against a node, ignoring the `&`
//! mnemonic marker and everything past a TAB. [`cmd_modes`] maps
//! the command name (`nmenu`, `vnoremenu`, `amenu!`, ...) onto the mode
//! bitmask, the `:noremap` flag and the `:unmenu` flag; [`menu_mode_str`]
//! and [`popup_mode_name`] go the other way, and [`menu_text`] splits a name
//! into the displayed text, the mnemonic and the `<Tab>`-separated
//! accelerator.
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
use std::ffi::CString;

use super::*;
use crate::cstr;
use crate::keycodes::Ctrl_V;
use crate::mbyte::cluster_len;

/// The accelerator separator inside a menu name. `\t` in a `:menu` argument
/// is a backslash escaping the letter `t`; only a literal `<Tab>`, which
/// [`menu_translate_tab_and_shift`] rewrites before the name is parsed,
/// becomes this byte.
const TAB: u8 = b'\t';

/// The byte at `i`, with a NUL for anything past the end -- how C reads a
/// string it has already tested the length of.
fn at(bytes: &[u8], i: usize) -> u8 {
    bytes.get(i).copied().unwrap_or(0)
}

/// One `.`-separated component of `name`, with its own `\\` and `^V`
/// escapes taken out, and the rest of the path after the `.` that ends it.
///
/// The escaped character is taken whole, so an escaped `.` does not end the
/// component; an escape at the very end is dropped, leaving the rest empty.
pub(crate) fn take_component(name: &[u8]) -> (Vec<u8>, &[u8]) {
    let mut component = Vec::with_capacity(name.len());
    let mut i = 0;
    while i < name.len() && name[i] != b'.' {
        if name[i] == b'\\' || c_int::from(name[i]) == Ctrl_V {
            i += 1;
            if i == name.len() {
                break;
            }
        }
        let len = cluster_len(&name[i..]);
        component.extend_from_slice(&name[i..i + len]);
        i += len;
    }
    if i < name.len() {
        i += 1; // the `.`
    }
    (component, &name[i..])
}

/// Whether `name` names `menu`, compared four ways: the raw name and the
/// display name, each in the menu's own language and -- for a node
/// `:menutranslate` renamed -- in English.
pub(crate) fn name_equal(name: &[u8], menu: &VimMenu) -> bool {
    if let Some(en_name) = &menu.en_name {
        // `add_menu_path` sets the two English names together.
        let en_dname = menu
            .en_dname
            .as_ref()
            .expect("en_dname accompanies en_name");
        if namecmp(name, en_name.as_bytes()) || namecmp(name, en_dname.as_bytes()) {
            return true;
        }
    }
    namecmp(name, menu.name.as_bytes()) || namecmp(name, menu.dname.as_bytes())
}

/// Whether two names agree up to the end of either -- where "the end" is the
/// terminator or the first TAB, so the accelerator text never takes part.
fn namecmp(name: &[u8], mname: &[u8]) -> bool {
    let mut i = 0;
    while at(name, i) != 0 && at(name, i) != TAB && at(name, i) == at(mname, i) {
        i += 1;
    }
    matches!(at(name, i), 0 | TAB) && matches!(at(mname, i), 0 | TAB)
}

/// The `MENU_*_MODE` bits a menu command names, e.g. `:menu!` is
/// `MENU_CMDLINE_MODE | MENU_INSERT_MODE`, plus the `noremap` value and
/// whether this was an `:unmenu` form.
///
/// The command name is read one letter at a time; whatever is left after the
/// mode prefix decides the other two, so `nnoremenu` is Normal mode and
/// no-remap while `noremenu` is the default modes and no-remap.
pub(crate) fn cmd_modes(cmd: &[u8], forceit: bool) -> (c_int, c_int, bool) {
    let (modes, tail) = match at(cmd, 0) {
        b'v' => (MENU_VISUAL_MODE | MENU_SELECT_MODE, 1),
        b'x' => (MENU_VISUAL_MODE, 1),
        b's' => (MENU_SELECT_MODE, 1),
        b'o' => (MENU_OP_PENDING_MODE, 1),
        b'i' => (MENU_INSERT_MODE, 1),
        b't' if at(cmd, 1) == b'l' => (MENU_TERMINAL_MODE, 2),
        b't' => (MENU_TIP_MODE, 1),
        b'c' => (MENU_CMDLINE_MODE, 1),
        b'a' => (MENU_AMENU_MODES, 1),
        b'n' if at(cmd, 1) != b'o' => (MENU_NORMAL_MODE, 1),
        // `noremenu`, `unmenu`, plain `menu`: nothing consumed.
        _ if forceit => (MENU_INSERT_MODE | MENU_CMDLINE_MODE, 0),
        _ => (MENU_PLAIN_MODES, 0),
    };
    let noremap = if at(cmd, tail) == b'n' {
        REMAP_NONE
    } else {
        REMAP_YES
    };
    (modes, noremap, at(cmd, tail) == b'u')
}

/// The command letters `modes` would be spelled with -- the opposite of
/// [`cmd_modes`]. `" "` is plain `:menu`, `"!"` is `:menu!`.
pub(crate) fn menu_mode_str(modes: c_int) -> &'static CStr {
    let all = |bits| modes & bits == bits;
    if all(MENU_AMENU_MODES) {
        c"a"
    } else if all(MENU_PLAIN_MODES) {
        c" "
    } else if all(MENU_INSERT_MODE | MENU_CMDLINE_MODE) {
        c"!"
    } else if all(MENU_VISUAL_MODE | MENU_SELECT_MODE) {
        c"v"
    } else if modes & MENU_VISUAL_MODE != 0 {
        c"x"
    } else if modes & MENU_SELECT_MODE != 0 {
        c"s"
    } else if modes & MENU_OP_PENDING_MODE != 0 {
        c"o"
    } else if modes & MENU_INSERT_MODE != 0 {
        c"i"
    } else if modes & MENU_TERMINAL_MODE != 0 {
        c"tl"
    } else if modes & MENU_CMDLINE_MODE != 0 {
        c"c"
    } else if modes & MENU_NORMAL_MODE != 0 {
        c"n"
    } else if modes & MENU_TIP_MODE != 0 {
        c"t"
    } else {
        c""
    }
}

/// `PopUp…` with the mode's letters spliced in after `PopUp`, which is how
/// the per-mode copies (`PopUpn`, `PopUptl`, ...) are named.
///
/// Only reached for a name [`is_popup`] accepted, so the first five bytes
/// are there to split after.
pub(crate) fn popup_mode_name(name: &[u8], idx: c_int) -> Vec<u8> {
    let (head, tail) = name.split_at(5);
    let idx = usize::try_from(idx).expect("a menu mode index is never negative");
    let mode = MODE_CHARS[idx].to_bytes();
    let mut out = Vec::with_capacity(head.len() + mode.len() + tail.len());
    out.extend_from_slice(head);
    out.extend_from_slice(mode);
    out.extend_from_slice(tail);
    out
}

/// A menu name taken apart: what is displayed, the `&` mnemonic, and the
/// accelerator after the first TAB.
pub(crate) struct MenuText {
    pub(crate) display: CString,
    /// The byte after the first `&` that is not `&&`, if there was one.
    pub(crate) mnemonic: Option<c_int>,
    pub(crate) actext: Option<CString>,
}

/// Split `name` into [`MenuText`]'s three parts.
///
/// Everything after the first TAB is the accelerator, kept verbatim. In
/// what is left, each `&` is dropped and the character behind it taken
/// whole, so `&&` reduces to one `&` and contributes no mnemonic; a
/// trailing `&` is kept as text. The *last* mnemonic wins.
pub(crate) fn menu_text(name: &[u8]) -> MenuText {
    let bytes = name;
    let (head, actext) = match bytes.iter().position(|&b| b == TAB) {
        Some(tab) => (&bytes[..tab], Some(cstr::owned(&bytes[tab + 1..]))),
        None => (bytes, None),
    };

    let mut display = Vec::with_capacity(head.len());
    let mut mnemonic = None;
    let mut i = 0;
    while i < head.len() {
        if head[i] == b'&' && i + 1 < head.len() {
            if head[i + 1] != b'&' {
                mnemonic = Some(c_int::from(head[i + 1]));
            }
            display.push(head[i + 1]);
            i += 2;
        } else {
            display.push(head[i]);
            i += 1;
        }
    }

    MenuText {
        display: cstr::owned(&display),
        mnemonic,
        actext,
    }
}
