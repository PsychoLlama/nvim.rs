//! Menu names as text -- parsing a path, matching one, and the mode
//! letters.
//!
//! [`skip_component`] steps over one `\`-escaped path component;
//! [`name_equal`] compares a component against a node, ignoring the `&`
//! mnemonic marker and everything past a TAB. [`get_menu_cmd_modes`] maps
//! the command name (`nmenu`, `vnoremenu`, `amenu!`, ...) onto the mode
//! bitmask, the `:noremap` flag and the `:unmenu` flag; [`menu_mode_str`]
//! and [`popup_mode_name`] go the other way, and [`menu_text`] splits a name
//! into the displayed text, the mnemonic and the `<Tab>`-separated
//! accelerator.
//!
//! Original: `src/nvim/menu.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};
use std::ffi::CString;

use super::*;
use crate::keycodes::Ctrl_V;

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

/// Consume one `.`-separated component of `name`, in place, and answer where
/// the next one starts.
///
/// The component's own `\` and `^V` escapes are squeezed out and the `.`
/// that ends it becomes a NUL, so `name` is left naming just the component
/// and the answer names the rest. An escape squeezed at the very end stops
/// the walk, leaving the answer on the terminator.
pub(crate) fn skip_component(name: CText) -> CText {
    let mut i = 0;
    while name.byte(i) != 0 && name.byte(i) != b'.' {
        if name.byte(i) == b'\\' || name.byte(i) == Ctrl_V as u8 {
            name.squeeze(i, 1);
            if name.byte(i) == 0 {
                break;
            }
        }
        // The escaped character is stepped over whole, so an escaped `.`
        // does not end the component.
        i += name.char_len(i);
    }
    if name.byte(i) != 0 {
        name.set(i, 0);
        i += 1;
    }
    name.at(i)
}

/// Whether `name` names `menu`, compared four ways: the raw name and the
/// display name, each in the menu's own language and -- for a node
/// `:menutranslate` renamed -- in English.
pub(crate) fn name_equal(name: &CStr, menu: Menu) -> bool {
    let name = name.to_bytes();
    if let Some(en_name) = menu.en_name() {
        // `add_menu_path` sets the two English names together.
        let en_dname = menu.en_dname().expect("en_dname accompanies en_name");
        if namecmp(name, en_name.to_bytes()) || namecmp(name, en_dname.to_bytes()) {
            return true;
        }
    }
    namecmp(name, menu.name().to_bytes()) || namecmp(name, menu.dname().to_bytes())
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

/// [`cmd_modes`] for the two callers outside this module, which hold the
/// command name as a C string and want the flags through out-parameters.
///
/// # Safety
/// `cmd` must name a NUL-terminated string; `noremap` and `unmenu` must be
/// null or writable.
pub unsafe fn get_menu_cmd_modes(
    cmd: *const c_char,
    forceit: bool,
    noremap: *mut c_int,
    unmenu: *mut bool,
) -> c_int {
    // SAFETY: the caller's obligation.
    let (modes, no, un) = cmd_modes(unsafe { CStr::from_ptr(cmd) }.to_bytes(), forceit);
    // SAFETY: the caller's obligation; both writes finish here.
    unsafe {
        if !noremap.is_null() {
            *noremap = no;
        }
        if !unmenu.is_null() {
            *unmenu = un;
        }
    }
    modes
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
pub(crate) fn popup_mode_name(name: &CStr, idx: c_int) -> CString {
    let (head, tail) = name.to_bytes().split_at(5);
    let mode = MODE_CHARS[idx as usize].to_bytes();
    let mut out = Vec::with_capacity(head.len() + mode.len() + tail.len());
    out.extend_from_slice(head);
    out.extend_from_slice(mode);
    out.extend_from_slice(tail);
    CString::new(out).expect("a menu name holds no interior NUL")
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
pub(crate) fn menu_text(name: &CStr) -> MenuText {
    let bytes = name.to_bytes();
    let (head, actext) = match bytes.iter().position(|&b| b == TAB) {
        Some(tab) => (&bytes[..tab], Some(cstring(&bytes[tab + 1..]))),
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
        display: cstring(&display),
        mnemonic,
        actext,
    }
}

/// `bytes` as an owned C string. Menu names come from the command line,
/// which `ex_docmd` has already split on NUL, so there is never one inside.
pub(crate) fn cstring(bytes: &[u8]) -> CString {
    CString::new(bytes).expect("a menu name holds no interior NUL")
}
