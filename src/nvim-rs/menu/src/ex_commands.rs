//! Menu command handlers.
//!
//! This module provides Rust implementations of the `:menu`, `:emenu`,
//! and `show_popupmenu` command handlers.

use std::ffi::{c_char, c_int, c_void};

use crate::classify::rs_menu_is_popup;
use crate::commands::{rs_get_menu_cmd_modes, MenuCmdResult};
use crate::display::rs_show_menus;
use crate::execute::rs_get_menu_mode;
use crate::handle::VimMenuHandle;
use crate::lookup::rs_menu_getbyname;
use crate::menu_modes::{
    MENU_ALL_MODES, MENU_INDEX_CMDLINE, MENU_INDEX_INSERT, MENU_INDEX_INVALID, MENU_INDEX_NORMAL,
    MENU_INDEX_OP_PENDING, MENU_INDEX_SELECT, MENU_INDEX_TERMINAL, MENU_INDEX_TIP,
    MENU_INDEX_VISUAL, MENU_TIP_MODE,
};
use crate::mutate::{rs_menu_enable_recurse, rs_remove_menu};
use crate::path::rs_menu_translate_tab_and_shift;
use crate::popup::rs_popup_mode_name;
use crate::vim_menu::VimMenu;

/// Opaque handle to `exarg_T*`.
type ExArgHandle = *mut c_void;

/// REMAP values.
const REMAP_SCRIPT: c_int = -2;
const REPTERM_DO_LT: c_int = 2;

/// TriState values (from types_defs.h).
const K_NONE: c_int = -1;
const K_FALSE: c_int = 0;
const K_TRUE: c_int = 1;

/// Menu depth limit.
const MENUDEPTH: usize = 10;

/// EVENT_MENUPOPUP from generated auevents.
const EVENT_MENUPOPUP: c_int = 82;

// Error message strings.
const E_INVARG2: *const c_char = c"E475: Invalid argument: %s".as_ptr();
const E_TRAILING_ARG: *const c_char = c"E488: Trailing characters: %s".as_ptr();

extern "C" {
    fn semsg(s: *const c_char, ...) -> bool;
    fn nvim_gettext(s: *const c_char) -> *const c_char;
    fn xfree(ptr: *mut c_void);
    fn strlen(s: *const c_char) -> usize;

    // ExArg accessors (exarg_T stays opaque for now)
    fn nvim_menu_eap_get_cmd(eap: ExArgHandle) -> *const c_char;
    fn nvim_menu_eap_get_arg(eap: ExArgHandle) -> *mut c_char;
    fn nvim_menu_eap_get_forceit(eap: ExArgHandle) -> bool;
    fn nvim_menu_eap_get_addr_count(eap: ExArgHandle) -> c_int;
    fn nvim_menu_eap_get_line2(eap: ExArgHandle) -> c_int;

    // String manipulation
    fn skipwhite(s: *const c_char) -> *mut c_char;
    fn getdigits_int(pp: *mut *mut c_char, strict: bool, def: c_int) -> c_int;
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;

    // Keycode replacement
    fn replace_termcodes(
        from: *const c_char,
        from_len: usize,
        bufp: *mut *mut c_char,
        sid_arg: c_int,
        flags: c_int,
        did_simplify: *mut bool,
        cpo_val: *const c_char,
    ) -> *mut c_char;

    // UI and autocmd
    fn ui_call_update_menu();
    fn apply_autocmds(
        event: c_int,
        pat: *const c_char,
        fname_io: *const c_char,
        force: bool,
        buf: *mut c_void,
    ) -> bool;
    fn pum_show_popupmenu(menu: *mut VimMenu);

    // Root menu global (in C)
    static mut root_menu: *mut VimMenu;

    // Global state - direct access
    static p_cpo: *const c_char;
    static curbuf: *mut c_void;

    // Multibyte
    fn utfc_ptr2len(p: *const c_char) -> c_int;
}

/// Helper: check if byte is ASCII whitespace.
fn ascii_iswhite(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Case-insensitive check for "<nop>".
///
/// # Safety
/// `s` must be a valid NUL-terminated C string or null.
unsafe fn stricmp_nop(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }
    let bytes = unsafe { std::slice::from_raw_parts(s as *const u8, strlen(s)) };
    bytes.eq_ignore_ascii_case(b"<nop>")
}

/// The `:menu` command and relatives.
///
/// This is the Rust implementation of C `ex_menu()`.
///
/// # Safety
/// `eap` must be a valid `exarg_T*` pointer.
#[export_name = "ex_menu"]
pub unsafe extern "C" fn rs_ex_menu(eap: ExArgHandle) {
    let mut pri_tab: [c_int; MENUDEPTH + 1] = [0; MENUDEPTH + 1];
    let mut enable: c_int = K_NONE; // TriState

    let cmd = unsafe { nvim_menu_eap_get_cmd(eap) };
    let forceit = unsafe { nvim_menu_eap_get_forceit(eap) };

    let result: MenuCmdResult = unsafe { rs_get_menu_cmd_modes(cmd, forceit) };
    let modes = result.modes;
    let mut noremap = result.noremap;
    let unmenu = result.unmenu;
    let mut silent = false;

    let mut arg = unsafe { nvim_menu_eap_get_arg(eap) };

    // Parse modifiers: <script>, <silent>, <special>
    loop {
        if unsafe { strncmp(arg, c"<script>".as_ptr(), 8) } == 0 {
            noremap = REMAP_SCRIPT;
            arg = unsafe { skipwhite(arg.add(8)) };
            continue;
        }
        if unsafe { strncmp(arg, c"<silent>".as_ptr(), 8) } == 0 {
            silent = true;
            arg = unsafe { skipwhite(arg.add(8)) };
            continue;
        }
        if unsafe { strncmp(arg, c"<special>".as_ptr(), 9) } == 0 {
            // Ignore obsolete "<special>" modifier.
            arg = unsafe { skipwhite(arg.add(9)) };
            continue;
        }
        break;
    }

    // Locate an optional "icon=filename" argument
    // TODO(nvim): Currently this is only parsed. Should expose it to UIs.
    if unsafe { strncmp(arg, c"icon=".as_ptr(), 5) } == 0 {
        arg = unsafe { arg.add(5) };
        while unsafe { *arg } != 0 && unsafe { *arg } as u8 != b' ' {
            if unsafe { *arg } as u8 == b'\\' {
                // STRMOVE(arg, arg + 1) - shift left by 1
                let remaining = unsafe { strlen(arg.add(1)) };
                unsafe {
                    std::ptr::copy(arg.add(1), arg, remaining + 1);
                }
            }
            // MB_PTR_ADV
            let len = unsafe { utfc_ptr2len(arg) };
            arg = unsafe { arg.add(len.max(1) as usize) };
        }
        if unsafe { *arg } != 0 {
            unsafe { *arg = 0 };
            arg = unsafe { arg.add(1) };
            arg = unsafe { skipwhite(arg) };
        }
    }

    // Fill in the priority table.
    let mut p = arg;
    while unsafe { *p } != 0 {
        let ch = unsafe { *p } as u8;
        if !ch.is_ascii_digit() && ch != b'.' {
            break;
        }
        p = unsafe { p.add(1) };
    }

    let mut i: usize;
    if ascii_iswhite(unsafe { *p } as u8) {
        i = 0;
        while i < MENUDEPTH && !ascii_iswhite(unsafe { *arg } as u8) {
            pri_tab[i] = unsafe { getdigits_int(&mut arg, false, 0) };
            if pri_tab[i] == 0 {
                pri_tab[i] = 500;
            }
            if unsafe { *arg } as u8 == b'.' {
                arg = unsafe { arg.add(1) };
            }
            i += 1;
        }
        arg = unsafe { skipwhite(arg) };
    } else if unsafe { nvim_menu_eap_get_addr_count(eap) } != 0
        && unsafe { nvim_menu_eap_get_line2(eap) } != 0
    {
        pri_tab[0] = unsafe { nvim_menu_eap_get_line2(eap) };
        i = 1;
    } else {
        i = 0;
    }
    while i < MENUDEPTH {
        pri_tab[i] = 500;
        i += 1;
    }
    pri_tab[MENUDEPTH] = -1; // mark end of the table

    // Check for "enable" or "disable" argument.
    if unsafe { strncmp(arg, c"enable".as_ptr(), 6) } == 0
        && ascii_iswhite(unsafe { *arg.add(6) } as u8)
    {
        enable = K_TRUE;
        arg = unsafe { skipwhite(arg.add(6)) };
    } else if unsafe { strncmp(arg, c"disable".as_ptr(), 7) } == 0
        && ascii_iswhite(unsafe { *arg.add(7) } as u8)
    {
        enable = K_FALSE;
        arg = unsafe { skipwhite(arg.add(7)) };
    }

    // If there is no argument, display all menus.
    if unsafe { *arg } == 0 {
        unsafe { rs_show_menus(arg, modes) };
        return;
    }

    let menu_path = arg;
    if unsafe { *menu_path } as u8 == b'.' {
        unsafe { semsg(nvim_gettext(E_INVARG2), menu_path) };
        return;
    }

    let map_to = unsafe { rs_menu_translate_tab_and_shift(arg) };

    // If there is only a menu name, display menus with that name.
    if unsafe { *map_to } == 0 && !unmenu && enable == K_NONE {
        unsafe { rs_show_menus(menu_path, modes) };
        return;
    } else if unsafe { *map_to } != 0 && (unmenu || enable != K_NONE) {
        unsafe { semsg(nvim_gettext(E_TRAILING_ARG), map_to) };
        return;
    }

    let root_menu_ptr = &raw mut root_menu;

    if enable != K_NONE {
        // Change sensitivity of the menu.
        let mut path = menu_path;
        if unsafe { strcmp(menu_path, c"*".as_ptr()) } == 0 {
            path = c"".as_ptr() as *mut c_char;
        }

        // For the PopUp menu, process each mode separately.
        if unsafe { rs_menu_is_popup(path as *const c_char) } {
            for mode_i in 0..MENU_INDEX_TIP {
                if (modes & (1 << mode_i)) != 0 {
                    let popup_p = unsafe { rs_popup_mode_name(path as *const c_char, mode_i) };
                    unsafe {
                        rs_menu_enable_recurse(
                            VimMenuHandle::from_ptr(*root_menu_ptr),
                            popup_p,
                            MENU_ALL_MODES,
                            enable,
                        );
                        xfree(popup_p as *mut c_void);
                    }
                }
            }
        }
        unsafe {
            rs_menu_enable_recurse(VimMenuHandle::from_ptr(*root_menu_ptr), path, modes, enable);
        }
    } else if unmenu {
        // Delete menu(s).
        let mut path = menu_path;
        if unsafe { strcmp(menu_path, c"*".as_ptr()) } == 0 {
            path = c"".as_ptr() as *mut c_char;
        }

        // For the PopUp menu, remove each mode separately.
        if unsafe { rs_menu_is_popup(path as *const c_char) } {
            for mode_i in 0..MENU_INDEX_TIP {
                if (modes & (1 << mode_i)) != 0 {
                    let popup_p = unsafe { rs_popup_mode_name(path as *const c_char, mode_i) };
                    unsafe {
                        rs_remove_menu(root_menu_ptr, popup_p, MENU_ALL_MODES, true);
                        xfree(popup_p as *mut c_void);
                    }
                }
            }
        }

        // Careful: remove_menu() changes menu_path
        unsafe { rs_remove_menu(root_menu_ptr, path, modes, false) };
    } else {
        // Add menu(s).
        // Replace special key codes.
        let mut map_buf: *mut c_char = std::ptr::null_mut();
        let mut actual_map_to: *const c_char = map_to;

        if unsafe { stricmp_nop(map_to) } {
            // "<Nop>" means nothing
            actual_map_to = c"".as_ptr();
        } else if (modes & MENU_TIP_MODE) != 0 {
            // Menu tips are plain text.
        } else {
            actual_map_to = unsafe {
                replace_termcodes(
                    map_to,
                    strlen(map_to),
                    &mut map_buf,
                    0,
                    REPTERM_DO_LT,
                    std::ptr::null_mut(),
                    p_cpo,
                )
            };
        }

        // Build a temporary menuarg and call rs_add_menu_path directly
        let mut menuarg = VimMenu {
            modes,
            enabled: 0,
            name: std::ptr::null_mut(),
            dname: std::ptr::null_mut(),
            en_name: std::ptr::null_mut(),
            en_dname: std::ptr::null_mut(),
            mnemonic: 0,
            actext: std::ptr::null_mut(),
            priority: 0,
            strings: [std::ptr::null_mut(); 8],
            noremap: [
                noremap, noremap, noremap, noremap, noremap, noremap, noremap, noremap,
            ],
            silent: [silent; 8],
            children: std::ptr::null_mut(),
            parent: std::ptr::null_mut(),
            next: std::ptr::null_mut(),
        };
        unsafe {
            crate::mutate::rs_add_menu_path(
                menu_path,
                VimMenuHandle::from_ptr(&mut menuarg as *mut VimMenu),
                pri_tab.as_ptr(),
                actual_map_to,
            )
        };

        // For the PopUp menu, add a menu for each mode separately.
        if unsafe { rs_menu_is_popup(menu_path as *const c_char) } {
            for mode_i in 0..MENU_INDEX_TIP {
                if (modes & (1 << mode_i)) != 0 {
                    let popup_p = unsafe { rs_popup_mode_name(menu_path as *const c_char, mode_i) };
                    // Include all modes, to make ":amenu" work
                    let mut popup_menuarg = VimMenu {
                        modes,
                        enabled: 0,
                        name: std::ptr::null_mut(),
                        dname: std::ptr::null_mut(),
                        en_name: std::ptr::null_mut(),
                        en_dname: std::ptr::null_mut(),
                        mnemonic: 0,
                        actext: std::ptr::null_mut(),
                        priority: 0,
                        strings: [std::ptr::null_mut(); 8],
                        noremap: [noremap; 8],
                        silent: [silent; 8],
                        children: std::ptr::null_mut(),
                        parent: std::ptr::null_mut(),
                        next: std::ptr::null_mut(),
                    };
                    unsafe {
                        crate::mutate::rs_add_menu_path(
                            popup_p,
                            VimMenuHandle::from_ptr(&mut popup_menuarg as *mut VimMenu),
                            pri_tab.as_ptr(),
                            actual_map_to,
                        );
                        xfree(popup_p as *mut c_void);
                    }
                }
            }
        }

        unsafe { xfree(map_buf as *mut c_void) };
    }

    unsafe { ui_call_update_menu() };
}

/// The `:emenu` command.
///
/// This is the Rust implementation of C `ex_emenu()`.
///
/// # Safety
/// `eap` must be a valid `exarg_T*` pointer.
#[export_name = "ex_emenu"]
pub unsafe extern "C" fn rs_ex_emenu(eap: ExArgHandle) {
    let mut arg = unsafe { nvim_menu_eap_get_arg(eap) };
    let mut mode_idx: c_int = MENU_INDEX_INVALID;

    if unsafe { *arg } != 0 && ascii_iswhite(unsafe { *arg.add(1) } as u8) {
        match unsafe { *arg } as u8 {
            b'n' => mode_idx = MENU_INDEX_NORMAL,
            b'v' => mode_idx = MENU_INDEX_VISUAL,
            b's' => mode_idx = MENU_INDEX_SELECT,
            b'o' => mode_idx = MENU_INDEX_OP_PENDING,
            b't' => mode_idx = MENU_INDEX_TERMINAL,
            b'i' => mode_idx = MENU_INDEX_INSERT,
            b'c' => mode_idx = MENU_INDEX_CMDLINE,
            _ => {
                unsafe { semsg(nvim_gettext(E_INVARG2), arg) };
                return;
            }
        }
        arg = unsafe { skipwhite(arg.add(2)) };
    }

    let menu = unsafe { rs_menu_getbyname(arg) };
    if menu.is_null() {
        return;
    }

    // Found the menu, so execute (calling Rust implementation directly).
    unsafe { crate::execute_menu::rs_execute_menu(eap, menu, mode_idx) };
}

/// Menu mode chars for popup menu lookup.
const MENU_MODE_CHARS_POPUP: [&[u8]; 8] = [b"n", b"v", b"s", b"o", b"i", b"c", b"tl", b"t"];

/// Find and show the popup menu for the current mode.
///
/// This is the Rust implementation of C `show_popupmenu()`.
///
/// # Safety
/// Accesses global state (root_menu, curbuf).
#[export_name = "show_popupmenu"]
pub unsafe extern "C" fn rs_show_popupmenu() {
    let menu_mode = rs_get_menu_mode();
    if menu_mode == MENU_INDEX_INVALID {
        return;
    }

    let mode = MENU_MODE_CHARS_POPUP[menu_mode as usize];
    let mode_len = mode.len();

    unsafe {
        apply_autocmds(
            EVENT_MENUPOPUP,
            mode.as_ptr() as *const c_char,
            std::ptr::null(),
            false,
            curbuf,
        );
    }

    // Walk root_menu looking for "PopUp" + mode
    let mut menu = VimMenuHandle::from_ptr(unsafe { root_menu });
    while !menu.is_null() {
        let name = menu.name();
        if !name.is_null()
            && unsafe { strncmp(name, c"PopUp".as_ptr(), 5) } == 0
            && unsafe { strncmp(name.add(5), mode.as_ptr() as *const c_char, mode_len) } == 0
        {
            break;
        }
        menu = menu.next();
    }

    // Only show a popup when it is defined and has entries
    if menu.is_null() || menu.children().is_null() {
        return;
    }

    unsafe { pum_show_popupmenu(menu.as_ptr()) };
}
