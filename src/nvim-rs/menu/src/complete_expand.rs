//! Menu completion expansion functions.
//!
//! This module provides the actual completion expansion functions that are
//! called by Neovim's command-line completion system. These manage the
//! expansion context and iterate over menu items for wildmenu.

use std::ffi::{c_char, c_int, c_void};
use std::ptr::{self, addr_of_mut};

use crate::handle::VimMenuHandle;
use crate::menu_modes::MENU_ALL_MODES;
use crate::vim_menu::VimMenu;

/// EXPAND_UNSUCCESSFUL from cmdexpand_defs.h
const EXPAND_UNSUCCESSFUL: c_int = -2;
/// EXPAND_NOTHING from cmdexpand_defs.h
const EXPAND_NOTHING: c_int = 0;
/// EXPAND_MENUS from cmdexpand_defs.h (enum value 11)
const EXPAND_MENUS: c_int = 11;
/// EXPAND_MENUNAMES from cmdexpand_defs.h (enum value 21)
const EXPAND_MENUNAMES: c_int = 21;

/// Ctrl_V character value.
const CTRL_V: u8 = 0x16;

/// Buffer length for get_menu_names tbuffer.
const TBUFFER_LEN: usize = 256;

extern "C" {
    // expand_T field accessors (expand_T remains opaque for now)
    fn nvim_menu_xp_set_context(xp: *mut c_void, ctx: c_int);
    fn nvim_menu_xp_set_pattern(xp: *mut c_void, pattern: *mut c_char);

    // Static variable accessors (expand statics still in C, Phase 5 will move them)
    fn nvim_menu_get_expand_menu() -> VimMenuHandle;
    fn nvim_menu_set_expand_menu(menu: VimMenuHandle);
    fn nvim_menu_get_expand_modes() -> c_int;
    fn nvim_menu_set_expand_modes(modes: c_int);
    fn nvim_menu_get_expand_emenu() -> c_int;
    fn nvim_menu_set_expand_emenu(v: c_int);

    // Root menu global (in C)
    static mut root_menu: *mut VimMenu;

    // Already-ported functions
    fn rs_menu_name_equal(name: *const c_char, menu: VimMenuHandle) -> bool;
    fn rs_menu_name_skip(name: *mut c_char) -> *mut c_char;
    fn rs_menu_is_hidden(name: *const c_char) -> bool;
    fn rs_menu_is_separator(name: *const c_char) -> bool;
    fn rs_get_menu_cmd_modes(cmd: *const c_char, forceit: bool) -> crate::commands::MenuCmdResult;

    // Memory
    fn xfree(ptr: *mut c_void);
    fn xmalloc(size: usize) -> *mut c_void;
    fn xstrlcpy(dst: *mut c_char, src: *const c_char, dsize: usize) -> usize;
}

/// Check if a byte is ASCII whitespace (space or tab, matching Neovim's ascii_iswhite).
fn ascii_iswhite(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Check if a byte is an ASCII digit.
fn ascii_isdigit(c: u8) -> bool {
    c.is_ascii_digit()
}

/// Set up the completion context for menu commands.
///
/// This is the Rust implementation of C `set_context_in_menu_cmd()`.
///
/// # Safety
/// - `xp` must be a valid pointer to an `expand_T` structure.
/// - `cmd` must be a valid pointer to a NUL-terminated C string.
/// - `arg` must be a valid pointer to a mutable NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_set_context_in_menu_cmd(
    xp: *mut c_void,
    cmd: *const c_char,
    arg: *mut c_char,
    forceit: bool,
) -> *mut c_char {
    unsafe { nvim_menu_xp_set_context(xp, EXPAND_UNSUCCESSFUL) };

    // Check for priority numbers, enable and disable
    let mut p = arg;
    while unsafe { *p } != 0
        && (ascii_isdigit(unsafe { *p } as u8) || unsafe { *p } == b'.' as c_char)
    {
        p = unsafe { p.add(1) };
    }

    if !ascii_iswhite(unsafe { *p } as u8) {
        if unsafe { libc::strncmp(arg, c"enable".as_ptr(), 6) } == 0
            && (unsafe { *arg.add(6) } == 0 || ascii_iswhite(unsafe { *arg.add(6) } as u8))
        {
            p = unsafe { arg.add(6) };
        } else if unsafe { libc::strncmp(arg, c"disable".as_ptr(), 7) } == 0
            && (unsafe { *arg.add(7) } == 0 || ascii_iswhite(unsafe { *arg.add(7) } as u8))
        {
            p = unsafe { arg.add(7) };
        } else {
            p = arg;
        }
    }

    while unsafe { *p } != 0 && ascii_iswhite(unsafe { *p } as u8) {
        p = unsafe { p.add(1) };
    }

    let after_dot_init = p;
    let mut after_dot = p;

    let mut q = p;
    while unsafe { *q } != 0 && !ascii_iswhite(unsafe { *q } as u8) {
        if (unsafe { *q } == b'\\' as c_char || unsafe { *q } == CTRL_V as c_char)
            && unsafe { *q.add(1) } != 0
        {
            q = unsafe { q.add(1) };
        } else if unsafe { *q } == b'.' as c_char {
            after_dot = unsafe { q.add(1) };
        }
        q = unsafe { q.add(1) };
    }

    // ":popup" only uses menus, not entries
    // ":te..." (tearoff) also only uses menus
    let cmd0 = unsafe { *cmd } as u8;
    let cmd1 = unsafe { *cmd.add(1) } as u8;
    let expand_menus = !((cmd0 == b't' && cmd1 == b'e') || cmd0 == b'p');
    unsafe {
        nvim_menu_set_expand_emenu(if cmd0 == b'e' { 1 } else { 0 });
    }

    if expand_menus && ascii_iswhite(unsafe { *q } as u8) {
        return ptr::null_mut(); // TODO(vim): check for next command?
    }

    if unsafe { *q } == 0 {
        // Complete the menu name
        let result = unsafe { rs_get_menu_cmd_modes(cmd, forceit) };
        let modes = result.modes;
        let unmenu = result.unmenu;
        if !unmenu {
            unsafe { nvim_menu_set_expand_modes(MENU_ALL_MODES) };
        } else {
            unsafe { nvim_menu_set_expand_modes(modes) };
        }

        let mut menu = VimMenuHandle::from_ptr(unsafe { root_menu });
        let mut path_name: *mut c_char = ptr::null_mut();

        if after_dot > after_dot_init {
            let path_len = unsafe { after_dot.offset_from(after_dot_init) } as usize;
            path_name = unsafe { xmalloc(path_len) } as *mut c_char;
            unsafe { xstrlcpy(path_name, after_dot_init, path_len) };
        }

        let mut name = path_name;
        while !name.is_null() && unsafe { *name } != 0 {
            let next = unsafe { rs_menu_name_skip(name) };
            while !menu.is_null() {
                if unsafe { rs_menu_name_equal(name, menu) } {
                    // Found menu
                    if (unsafe { *next } != 0 && menu.children().is_null())
                        || ((menu.modes() & unsafe { nvim_menu_get_expand_modes() }) == 0)
                    {
                        unsafe { xfree(path_name as *mut c_void) };
                        return ptr::null_mut();
                    }
                    break;
                }
                menu = menu.next();
            }
            if menu.is_null() {
                unsafe { xfree(path_name as *mut c_void) };
                return ptr::null_mut();
            }
            name = next;
            menu = menu.children();
        }
        unsafe { xfree(path_name as *mut c_void) };

        let ctx = if expand_menus {
            EXPAND_MENUNAMES
        } else {
            EXPAND_MENUS
        };
        unsafe { nvim_menu_xp_set_context(xp, ctx) };
        unsafe { nvim_menu_xp_set_pattern(xp, after_dot) };
        unsafe { nvim_menu_set_expand_menu(menu) };
    } else {
        // We're in the mapping part
        unsafe { nvim_menu_xp_set_context(xp, EXPAND_NOTHING) };
    }

    ptr::null_mut()
}

/// Get the next menu name for completion (submenus only, not entries).
///
/// This is the Rust implementation of C `get_menu_name()`.
///
/// Function given to ExpandGeneric() to obtain the list of (sub)menus.
///
/// # Safety
/// `xp` must be a valid pointer to an `expand_T` structure.
#[no_mangle]
pub unsafe extern "C" fn rs_get_menu_name(_xp: *mut c_void, idx: c_int) -> *mut c_char {
    // Static state for iteration. Neovim is single-threaded.
    static mut MENU: VimMenuHandle = VimMenuHandle::null_const();
    static mut SHOULD_ADVANCE: bool = false;

    if idx == 0 {
        unsafe {
            MENU = nvim_menu_get_expand_menu();
            SHOULD_ADVANCE = false;
        }
    }

    // Skip PopUp[nvoci], separators, and non-submenus (no children).
    loop {
        let menu = unsafe { MENU };
        if menu.is_null() {
            break;
        }
        if !unsafe { rs_menu_is_hidden(menu.dname()) }
            && !unsafe { rs_menu_is_separator(menu.dname()) }
            && !menu.children().is_null()
        {
            break;
        }
        unsafe { MENU = menu.next() };
    }

    let menu = unsafe { MENU };
    if menu.is_null() {
        return ptr::null_mut();
    }

    let expand_modes = unsafe { nvim_menu_get_expand_modes() };
    let str_ptr;

    if (menu.modes() & expand_modes) != 0 {
        if unsafe { SHOULD_ADVANCE } {
            str_ptr = menu.en_dname();
        } else {
            str_ptr = menu.dname();
            if menu.en_dname().is_null() {
                unsafe { SHOULD_ADVANCE = true };
            }
        }
    } else {
        str_ptr = c"".as_ptr();
    }

    if unsafe { SHOULD_ADVANCE } {
        // Advance to next menu entry.
        unsafe { MENU = menu.next() };
    }

    unsafe { SHOULD_ADVANCE = !SHOULD_ADVANCE };

    str_ptr as *mut c_char
}

/// Get the next menu name for completion (menus and menu entries).
///
/// This is the Rust implementation of C `get_menu_names()`.
///
/// Function given to ExpandGeneric() to obtain the list of menus and entries.
///
/// # Safety
/// `xp` must be a valid pointer to an `expand_T` structure.
#[no_mangle]
pub unsafe extern "C" fn rs_get_menu_names(_xp: *mut c_void, idx: c_int) -> *mut c_char {
    // Static state for iteration. Neovim is single-threaded.
    static mut MENU: VimMenuHandle = VimMenuHandle::null_const();
    static mut TBUFFER: [u8; TBUFFER_LEN] = [0u8; TBUFFER_LEN];
    static mut SHOULD_ADVANCE: bool = false;

    if idx == 0 {
        unsafe {
            MENU = nvim_menu_get_expand_menu();
            SHOULD_ADVANCE = false;
        }
    }

    let expand_emenu = unsafe { nvim_menu_get_expand_emenu() } != 0;

    // Skip Browse-style entries, popup menus and separators.
    loop {
        let menu = unsafe { MENU };
        if menu.is_null() {
            break;
        }
        let dname = menu.dname();
        let is_hidden = unsafe { rs_menu_is_hidden(dname) };
        let is_sep = expand_emenu && unsafe { rs_menu_is_separator(dname) };
        // Check if dname ends with '.'
        let dname_len = unsafe { libc::strlen(dname) };
        let ends_with_dot = dname_len > 0 && unsafe { *dname.add(dname_len - 1) } == b'.' as c_char;

        if !is_hidden && !is_sep && !ends_with_dot {
            break;
        }
        unsafe { MENU = menu.next() };
    }

    let menu = unsafe { MENU };
    if menu.is_null() {
        return ptr::null_mut();
    }

    let expand_modes = unsafe { nvim_menu_get_expand_modes() };
    let str_ptr;

    if (menu.modes() & expand_modes) != 0 {
        if !menu.children().is_null() {
            // Submenu: copy name into tbuffer and append "\001" separator
            let tbuf_ptr = addr_of_mut!(TBUFFER) as *mut c_char;
            if unsafe { SHOULD_ADVANCE } {
                unsafe { xstrlcpy(tbuf_ptr, menu.en_dname(), TBUFFER_LEN) };
            } else {
                unsafe { xstrlcpy(tbuf_ptr, menu.dname(), TBUFFER_LEN) };
                if menu.en_dname().is_null() {
                    unsafe { SHOULD_ADVANCE = true };
                }
            }
            // Append "\001" magic separator
            unsafe { libc::strcat(tbuf_ptr, c"\x01".as_ptr()) };
            str_ptr = tbuf_ptr as *const c_char;
        } else {
            // Leaf entry
            if unsafe { SHOULD_ADVANCE } {
                str_ptr = menu.en_dname();
            } else {
                str_ptr = menu.dname();
                if menu.en_dname().is_null() {
                    unsafe { SHOULD_ADVANCE = true };
                }
            }
        }
    } else {
        str_ptr = c"".as_ptr();
    }

    if unsafe { SHOULD_ADVANCE } {
        unsafe { MENU = menu.next() };
    }

    unsafe { SHOULD_ADVANCE = !SHOULD_ADVANCE };

    str_ptr as *mut c_char
}
