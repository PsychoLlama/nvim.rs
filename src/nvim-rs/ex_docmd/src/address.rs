//! Address and range parsing types for Ex commands.
//!
//! This module defines types for command address/range parsing,
//! such as `1,5`, `%`, `'a,'b`, etc.

use std::ffi::{c_char, c_int, c_void};

use crate::ExArgHandle;

// =============================================================================
// Address type enum
// =============================================================================

/// Type of address for an Ex command.
///
/// Determines how the address/range is interpreted for the command.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddrType {
    /// Buffer line numbers (default for most commands)
    Lines = 0,
    /// Window number
    Windows = 1,
    /// Argument number
    Arguments = 2,
    /// Buffer number of loaded buffer
    LoadedBuffers = 3,
    /// Buffer number (any buffer)
    Buffers = 4,
    /// Tab page number
    Tabs = 5,
    /// Tab page that only uses relative addressing
    TabsRelative = 6,
    /// Quickfix list valid entry number
    QuickfixValid = 7,
    /// Quickfix list entry number
    Quickfix = 8,
    /// Positive count or zero, defaults to 1
    Unsigned = 9,
    /// Something else, use line number for '$', '%', etc.
    Other = 10,
    /// No range used
    None = 11,
}

impl AddrType {
    /// Convert from C integer value.
    #[inline]
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Lines),
            1 => Some(Self::Windows),
            2 => Some(Self::Arguments),
            3 => Some(Self::LoadedBuffers),
            4 => Some(Self::Buffers),
            5 => Some(Self::Tabs),
            6 => Some(Self::TabsRelative),
            7 => Some(Self::QuickfixValid),
            8 => Some(Self::Quickfix),
            9 => Some(Self::Unsigned),
            10 => Some(Self::Other),
            11 => Some(Self::None),
            _ => Option::None,
        }
    }

    /// Convert to C integer value.
    #[inline]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if this address type uses line numbers.
    #[inline]
    pub const fn uses_line_numbers(self) -> bool {
        matches!(self, Self::Lines | Self::Other)
    }

    /// Check if this address type uses buffer numbers.
    #[inline]
    pub const fn uses_buffer_numbers(self) -> bool {
        matches!(self, Self::LoadedBuffers | Self::Buffers)
    }

    /// Check if this address type uses window numbers.
    #[inline]
    pub const fn uses_window_numbers(self) -> bool {
        matches!(self, Self::Windows)
    }

    /// Check if this address type uses tab numbers.
    #[inline]
    pub const fn uses_tab_numbers(self) -> bool {
        matches!(self, Self::Tabs | Self::TabsRelative)
    }

    /// Check if this address type uses quickfix entries.
    #[inline]
    pub const fn uses_quickfix(self) -> bool {
        matches!(self, Self::Quickfix | Self::QuickfixValid)
    }
}

// =============================================================================
// FFI functions
// =============================================================================

/// Convert C address type integer to Rust enum.
///
/// Returns -1 if the value is invalid.
#[no_mangle]
pub extern "C" fn rs_addr_type_from_int(val: c_int) -> c_int {
    match AddrType::from_c_int(val) {
        Some(t) => t.to_c_int(),
        Option::None => -1,
    }
}

/// Check if address type uses line numbers.
#[no_mangle]
pub extern "C" fn rs_addr_type_uses_lines(val: c_int) -> c_int {
    match AddrType::from_c_int(val) {
        Some(t) => c_int::from(t.uses_line_numbers()),
        Option::None => 0,
    }
}

/// Check if address type uses buffer numbers.
#[no_mangle]
pub extern "C" fn rs_addr_type_uses_buffers(val: c_int) -> c_int {
    match AddrType::from_c_int(val) {
        Some(t) => c_int::from(t.uses_buffer_numbers()),
        Option::None => 0,
    }
}

/// Check if address type uses window numbers.
#[no_mangle]
pub extern "C" fn rs_addr_type_uses_windows(val: c_int) -> c_int {
    match AddrType::from_c_int(val) {
        Some(t) => c_int::from(t.uses_window_numbers()),
        Option::None => 0,
    }
}

/// Check if address type uses tab numbers.
#[no_mangle]
pub extern "C" fn rs_addr_type_uses_tabs(val: c_int) -> c_int {
    match AddrType::from_c_int(val) {
        Some(t) => c_int::from(t.uses_tab_numbers()),
        Option::None => 0,
    }
}

/// Check if address type uses quickfix entries.
#[no_mangle]
pub extern "C" fn rs_addr_type_uses_quickfix(val: c_int) -> c_int {
    match AddrType::from_c_int(val) {
        Some(t) => c_int::from(t.uses_quickfix()),
        Option::None => 0,
    }
}

// =============================================================================
// C-compatible constants
// =============================================================================

/// ADDR_LINES - buffer line numbers
pub const ADDR_LINES: c_int = 0;
/// ADDR_WINDOWS - window number
pub const ADDR_WINDOWS: c_int = 1;
/// ADDR_ARGUMENTS - argument number
pub const ADDR_ARGUMENTS: c_int = 2;
/// ADDR_LOADED_BUFFERS - loaded buffer number
pub const ADDR_LOADED_BUFFERS: c_int = 3;
/// ADDR_BUFFERS - buffer number
pub const ADDR_BUFFERS: c_int = 4;
/// ADDR_TABS - tab page number
pub const ADDR_TABS: c_int = 5;
/// ADDR_TABS_RELATIVE - relative tab page
pub const ADDR_TABS_RELATIVE: c_int = 6;
/// ADDR_QUICKFIX_VALID - valid quickfix entry
pub const ADDR_QUICKFIX_VALID: c_int = 7;
/// ADDR_QUICKFIX - quickfix entry
pub const ADDR_QUICKFIX: c_int = 8;
/// ADDR_UNSIGNED - unsigned count
pub const ADDR_UNSIGNED: c_int = 9;
/// ADDR_OTHER - other address type
pub const ADDR_OTHER: c_int = 10;
/// ADDR_NONE - no address
pub const ADDR_NONE: c_int = 11;

// =============================================================================
// Phase 5: FFI declarations for address/range helpers
// =============================================================================

// Control character constants (verified with _Static_assert in C)
const CTRL_S: u8 = 19;
const CTRL_N: u8 = 14;
const CTRL_J: u8 = 10;
const CTRL_K: u8 = 11;
const CTRL_R: u8 = 18;
const CTRL_UNDERSCORE: u8 = 31;
const CTRL_RSB: u8 = 29;
const CTRL_G: u8 = 7;
const CTRL_V: u8 = 22;
const CTRL_H: u8 = 8;
const CTRL_L: u8 = 12;
const CTRL_F: u8 = 6;
const CTRL_I: u8 = 9;
const CTRL_D: u8 = 4;
const CTRL_HAT: u8 = 30;
const CTRL_Q: u8 = 17;
const CTRL_C: u8 = 3;
const CTRL_O: u8 = 15;
const CTRL_W: u8 = 23;
const CTRL_X: u8 = 24;
const CTRL_Z: u8 = 26;
const CTRL_T: u8 = 20;
const CTRL_B: u8 = 2;
const CTRL_P: u8 = 16;
const CAR: u8 = 13;

extern "C" {
    fn skipwhite(p: *const c_char) -> *mut c_char;

    // eap field accessors

    // cmdnames table accessor
    fn nvim_docmd_cmdnames_addr_type(idx: c_int) -> c_int;

    // bt_quickfix check (via rs_bt_quickfix since bt_quickfix is inline)
    fn rs_bt_quickfix(buf: *mut c_void) -> bool;
    fn nvim_get_curbuf() -> *mut c_void;

    // Window/tab navigation
    fn nvim_docmd_current_win_nr() -> c_int;
    fn nvim_docmd_current_tab_nr() -> c_int;
    fn nvim_docmd_last_tab_nr() -> c_int;

    // Cursor and arg accessors
    fn nvim_win_get_cursor_lnum(wp: *mut c_void) -> i32;
    fn nvim_docmd_get_curwin_arg_idx() -> c_int;
    fn nvim_docmd_get_argcount() -> c_int;

    // Buffer accessors
    fn nvim_buf_get_line_count(buf: *mut c_void) -> i32;
    fn nvim_docmd_get_curbuf_fnum() -> c_int;

    // Quickfix accessors (directly exported)
    fn qf_get_cur_idx(eap: ExArgHandle) -> usize;
    fn qf_get_cur_valid_idx(eap: ExArgHandle) -> c_int;
    fn qf_get_valid_size(eap: ExArgHandle) -> usize;

    // Buffer list walking
    fn nvim_docmd_first_loaded_buf_fnum() -> c_int;
    fn nvim_docmd_last_loaded_buf_fnum() -> c_int;
    fn nvim_docmd_firstbuf_fnum() -> c_int;
    fn nvim_docmd_lastbuf_fnum() -> c_int;

    // dflall error
    fn nvim_docmd_iemsg_dflall();

    // Tabpage accessors
    fn nvim_get_curtab() -> *mut std::ffi::c_void;
    fn nvim_get_lastused_tabpage() -> *mut std::ffi::c_void;
    #[link_name = "rs_tabpage_index"]
    fn nvim_rs_tabpage_index(tp: *mut std::ffi::c_void) -> c_int;
    #[link_name = "rs_valid_tabpage"]
    fn nvim_rs_valid_tabpage(tp: *mut std::ffi::c_void) -> c_int;
    fn nvim_docmd_last_win_nr() -> c_int;
    #[link_name = "rs_ascii_iswhite"]
    fn nvim_docmd_ascii_iswhite(c: c_int) -> c_int;
    #[link_name = "rs_ascii_isdigit"]
    fn nvim_docmd_ascii_isdigit(c: c_int) -> c_int;
    fn getdigits_int(pp: *mut *mut c_char, strict: bool, def: c_int) -> c_int;
    fn ex_errmsg(msg: *const c_char, arg: *const c_char) -> *mut c_char;
    static e_invargval: [c_char; 1];
    static e_invarg2: [c_char; 1];
}

// =============================================================================
// get_wincmd_addr_type — determine addr_type for :wincmd subcommands
// =============================================================================

/// Determine the address type for `:wincmd` based on its argument character.
///
/// Sets `eap->addr_type` to the appropriate ADDR_* value.
unsafe fn get_wincmd_addr_type(arg: *const c_char, eap: ExArgHandle) {
    let c = *arg as u8;
    match c {
        // window size or any count → ADDR_OTHER
        b'S' | b's' | b'n' | b'j' | b'k' | b'T' | b'r' | b'R' | b'K' | b'J' | b'+' | b'-'
        | b'_' | b'|' | b']' | b'g' | b'v' | b'h' | b'l' | b'H' | b'L' | b'>' | b'<' | b'}'
        | b'f' | b'F' | b'i' | b'd' => {
            (*eap).addr_type = ADDR_OTHER;
        }
        x if x == CTRL_S
            || x == CTRL_N
            || x == CTRL_J
            || x == CTRL_K
            || x == CTRL_R
            || x == CTRL_UNDERSCORE
            || x == CTRL_RSB
            || x == CTRL_G
            || x == CTRL_V
            || x == CTRL_H
            || x == CTRL_L
            || x == CTRL_F
            || x == CTRL_I
            || x == CTRL_D =>
        {
            (*eap).addr_type = ADDR_OTHER;
        }

        // buffer number → ADDR_BUFFERS
        b'^' => {
            (*eap).addr_type = ADDR_BUFFERS;
        }
        x if x == CTRL_HAT => {
            (*eap).addr_type = ADDR_BUFFERS;
        }

        // window number → ADDR_WINDOWS
        b'q' | b'c' | b'o' | b'w' | b'W' | b'x' => {
            (*eap).addr_type = ADDR_WINDOWS;
        }
        x if x == CTRL_Q || x == CTRL_C || x == CTRL_O || x == CTRL_W || x == CTRL_X => {
            (*eap).addr_type = ADDR_WINDOWS;
        }

        // no count → ADDR_NONE
        b'z' | b'P' | b't' | b'b' | b'p' | b'=' => {
            (*eap).addr_type = ADDR_NONE;
        }
        x if x == CTRL_Z || x == CTRL_T || x == CTRL_B || x == CTRL_P || x == CAR => {
            (*eap).addr_type = ADDR_NONE;
        }

        _ => {}
    }
}

// =============================================================================
// rs_set_cmd_addr_type
// =============================================================================

/// Set the address type for a command.
///
/// Replaces C `set_cmd_addr_type()`.
#[export_name = "set_cmd_addr_type"]
pub unsafe extern "C" fn rs_set_cmd_addr_type(eap: ExArgHandle, p: *mut c_char) {
    let cmdidx = (*eap).cmdidx;

    // User commands have addr_type set by find_ucmd
    if cmdidx < 0 {
        return;
    }

    if cmdidx != crate::commands::CMD_SIZE {
        let addr_type = nvim_docmd_cmdnames_addr_type(cmdidx);
        (*eap).addr_type = addr_type;
    } else {
        (*eap).addr_type = ADDR_LINES;
    }

    // :wincmd range depends on the argument
    if cmdidx == crate::commands::CMD_WINCMD && !p.is_null() {
        let whiteskipped = skipwhite(p);
        get_wincmd_addr_type(whiteskipped, eap);
    }

    // :.cc in quickfix window uses line number
    if (cmdidx == crate::commands::CMD_CC || cmdidx == crate::commands::CMD_LL)
        && rs_bt_quickfix(nvim_get_curbuf())
    {
        (*eap).addr_type = ADDR_OTHER;
    }
}

// =============================================================================
// rs_get_cmd_default_range
// =============================================================================

/// Get default range number for command based on its address type.
///
/// Replaces C `get_cmd_default_range()`.
#[export_name = "get_cmd_default_range"]
pub unsafe extern "C" fn rs_get_cmd_default_range(eap: ExArgHandle) -> i32 {
    let addr_type = (*eap).addr_type;
    match addr_type {
        x if x == ADDR_LINES || x == ADDR_OTHER => {
            let cursor_lnum = nvim_win_get_cursor_lnum(nvim_get_curwin());
            let line_count = nvim_buf_get_line_count(nvim_get_curbuf());
            if cursor_lnum < line_count {
                cursor_lnum
            } else {
                line_count
            }
        }
        x if x == ADDR_WINDOWS => nvim_docmd_current_win_nr() as i32,
        x if x == ADDR_ARGUMENTS => {
            let arg_idx = nvim_docmd_get_curwin_arg_idx() + 1;
            let argcount = nvim_docmd_get_argcount();
            if arg_idx < argcount {
                arg_idx as i32
            } else {
                argcount as i32
            }
        }
        x if x == ADDR_LOADED_BUFFERS || x == ADDR_BUFFERS => nvim_docmd_get_curbuf_fnum() as i32,
        x if x == ADDR_TABS => nvim_docmd_current_tab_nr() as i32,
        x if x == ADDR_TABS_RELATIVE || x == ADDR_UNSIGNED => 1,
        x if x == ADDR_QUICKFIX => qf_get_cur_idx(eap) as i32,
        x if x == ADDR_QUICKFIX_VALID => qf_get_cur_valid_idx(eap) as i32,
        _ => 0,
    }
}

// =============================================================================
// rs_set_cmd_dflall_range
// =============================================================================

/// Set default-all range (% range) for all address types.
///
/// Replaces C `set_cmd_dflall_range()`.
#[export_name = "set_cmd_dflall_range"]
pub unsafe extern "C" fn rs_set_cmd_dflall_range(eap: ExArgHandle) {
    (*eap).line1 = 1;
    let addr_type = (*eap).addr_type;

    match addr_type {
        x if x == ADDR_LINES || x == ADDR_OTHER => {
            (*eap).line2 = nvim_buf_get_line_count(nvim_get_curbuf());
        }
        x if x == ADDR_LOADED_BUFFERS => {
            let first_fnum = nvim_docmd_first_loaded_buf_fnum();
            (*eap).line1 = first_fnum;
            let last_fnum = nvim_docmd_last_loaded_buf_fnum();
            (*eap).line2 = last_fnum;
        }
        x if x == ADDR_BUFFERS => {
            (*eap).line1 = nvim_docmd_firstbuf_fnum();
            (*eap).line2 = nvim_docmd_lastbuf_fnum();
        }
        x if x == ADDR_WINDOWS => {
            (*eap).line2 = nvim_docmd_last_win_nr() as i32;
        }
        x if x == ADDR_TABS => {
            (*eap).line2 = nvim_docmd_last_tab_nr() as i32;
        }
        x if x == ADDR_TABS_RELATIVE => {
            (*eap).line2 = 1;
        }
        x if x == ADDR_ARGUMENTS => {
            let argcount = nvim_docmd_get_argcount();
            if argcount == 0 {
                (*eap).line1 = 0;
                (*eap).line2 = 0;
            } else {
                (*eap).line2 = argcount as i32;
            }
        }
        x if x == ADDR_QUICKFIX_VALID => {
            let size = qf_get_valid_size(eap) as i32;
            (*eap).line2 = if size == 0 { 1 } else { size };
        }
        x if x == ADDR_NONE || x == ADDR_UNSIGNED || x == ADDR_QUICKFIX => {
            nvim_docmd_iemsg_dflall();
        }
        _ => {}
    }
}

// =============================================================================
// rs_get_tabpage_arg
// =============================================================================

/// Parse tabpage number argument.
///
/// Replaces C `get_tabpage_arg()`.
#[export_name = "get_tabpage_arg"]
pub unsafe extern "C" fn rs_get_tabpage_arg(eap: ExArgHandle) -> c_int {
    let cmdidx = (*eap).cmdidx;
    let unaccept_arg0: c_int = if cmdidx == crate::commands::CMD_TABMOVE {
        0
    } else {
        1
    };

    let arg = (*eap).arg;
    let last_tab_nr = nvim_docmd_last_tab_nr();

    if !arg.is_null() && *arg != 0 {
        let mut p = arg;
        let mut relative: c_int = 0;

        if *p as u8 == b'-' {
            relative = -1;
            p = p.add(1);
        } else if *p as u8 == b'+' {
            relative = 1;
            p = p.add(1);
        }

        let p_save = p;
        let mut tab_number = getdigits_int(&mut p, false, 0);

        if relative == 0 {
            if *p as u8 == b'$' && *p.add(1) == 0 {
                tab_number = last_tab_nr;
            } else if *p as u8 == b'#' && *p.add(1) == 0 {
                if nvim_rs_valid_tabpage(nvim_get_lastused_tabpage()) != 0 {
                    tab_number = nvim_rs_tabpage_index(nvim_get_lastused_tabpage());
                } else {
                    (*eap).errmsg = ex_errmsg(e_invargval.as_ptr(), arg);
                    return 0;
                }
            } else if p == p_save || *p_save as u8 == b'-' || *p != 0 || tab_number > last_tab_nr {
                (*eap).errmsg = ex_errmsg(e_invarg2.as_ptr(), arg);
                return 0;
            }
        } else {
            if *p_save == 0 {
                tab_number = 1;
            } else if p == p_save || *p_save as u8 == b'-' || *p != 0 || tab_number == 0 {
                (*eap).errmsg = ex_errmsg(e_invarg2.as_ptr(), arg);
                return 0;
            }
            tab_number = tab_number * relative + nvim_rs_tabpage_index(nvim_get_curtab());
            if unaccept_arg0 == 0 && relative == -1 {
                tab_number -= 1;
            }
        }

        if tab_number < unaccept_arg0 || tab_number > last_tab_nr {
            (*eap).errmsg = ex_errmsg(e_invarg2.as_ptr(), arg);
        }
        tab_number
    } else if (*eap).addr_count > 0 {
        if unaccept_arg0 != 0 && (*eap).line2 == 0 {
            (*eap).errmsg = crate::gt(crate::E_INVRANGE_STR.as_ptr()) as *mut c_char;
            return 0;
        }
        let mut tab_number = (*eap).line2;
        if unaccept_arg0 == 0 {
            let mut cmdp = (*eap).cmd;
            let cmdlinep = (*eap).cmdlinep;
            let cmdline_start = *cmdlinep;
            loop {
                cmdp = cmdp.sub(1);
                if cmdp <= cmdline_start {
                    break;
                }
                let ch = *cmdp as u8;
                if nvim_docmd_ascii_iswhite(ch as c_int) == 0
                    && nvim_docmd_ascii_isdigit(ch as c_int) == 0
                {
                    break;
                }
            }
            if *cmdp as u8 == b'-' {
                tab_number -= 1;
                if tab_number < unaccept_arg0 as i32 {
                    (*eap).errmsg = crate::gt(crate::E_INVRANGE_STR.as_ptr()) as *mut c_char;
                }
            }
        }
        tab_number
    } else if cmdidx == crate::commands::CMD_TABNEXT {
        let mut tab_number = nvim_rs_tabpage_index(nvim_get_curtab()) + 1;
        if tab_number > last_tab_nr {
            tab_number = 1;
        }
        tab_number
    } else if cmdidx == crate::commands::CMD_TABMOVE {
        last_tab_nr
    } else {
        nvim_rs_tabpage_index(nvim_get_curtab())
    }
}

// =============================================================================
// get_address — FFI declarations and implementation
// =============================================================================

// Constants (verified with _Static_assert in C)
const MAXLNUM: i32 = 0x7fffffff;
const MAXCOL: i32 = 0x7fffffff;
const FORWARD: c_int = 1;
const BACKWARD: c_int = -1;
const RE_SEARCH: c_int = 0;
const RE_SUBST: c_int = 1;
const SEARCH_HIS: c_int = 0x20;
const SEARCH_MSG: c_int = 0x0c;
const SEARCH_KEEP: c_int = 0x400;
const K_MARK_BUF_LOCAL: c_int = 0;
const K_MARK_ALL: c_int = 1;
const INT32_MAX: i32 = 0x7fffffff;
const NUL: c_char = 0;

extern "C" {
    // Cursor state (set)
    fn nvim_win_set_cursor_lnum(wp: *mut c_void, lnum: i32);
    fn nvim_win_set_cursor_col(wp: *mut c_void, col: i32);
    fn nvim_win_get_cursor_col(wp: *mut c_void) -> i32;
    static mut searchcmdlen: c_int;

    // Buffer handle
    fn nvim_docmd_get_curbuf_handle() -> c_int;

    // Quickfix (qf_get_size, directly exported, returns usize)
    fn qf_get_size(eap: ExArgHandle) -> usize;

    // Search
    fn nvim_docmd_do_search(
        eap: ExArgHandle,
        search_type: c_int,
        dirc: c_int,
        pat: *const c_char,
        patlen: usize,
        count: c_int,
        options: c_int,
    ) -> c_int;
    fn nvim_docmd_searchit(
        dir: c_int,
        re_pat: c_int,
        start_lnum: i32,
        start_col: i32,
        flags: c_int,
    ) -> i32;
    #[link_name = "strlen"]
    fn nvim_docmd_strlen(s: *const c_char) -> usize;
    #[link_name = "rs_magic_isset"]
    fn nvim_docmd_magic_isset() -> c_int;

    // Mark
    fn nvim_docmd_mark_get(flag: c_int, ch: c_int) -> *mut c_void;
    fn mark_check(fm: *mut c_void, errormsg: *mut *const c_char) -> bool;
    fn nvim_docmd_mark_fnum(fm: *const c_void) -> c_int;
    fn nvim_docmd_mark_lnum(fm: *const c_void) -> i32;
    fn mark_move_to(fm: *mut c_void, flags: c_int) -> c_int;

    // Folding
    fn nvim_docmd_hasFolding(lnum: i32) -> i32;

    // Buffer list navigation (for compute_buffer_local_count)
    fn nvim_get_firstbuf() -> *mut c_void;
    fn nvim_buf_get_next(buf: *mut c_void) -> *mut c_void;
    fn nvim_buf_get_prev(buf: *mut c_void) -> *mut c_void;
    fn nvim_buf_get_fnum(buf: *mut c_void) -> c_int;
    fn nvim_buf_get_ml_mfp_null(buf: *mut c_void) -> c_int;

    // Digit parsing (direct C function)
    fn getdigits_int32(pp: *mut *mut c_char, strict: bool, def: i32) -> i32;

    // Error messages

    // skip_regexp (already in Rust, exposed via FFI)
    fn rs_skip_regexp(startp: *mut c_char, delim: c_int, magic: c_int) -> *mut c_char;

    // parse_cmd_address helpers
    fn nvim_docmd_mark_get_visual(ch: c_int) -> *mut c_void;
    fn check_cursor(win: *mut c_void);
    fn check_cursor_col(win: *mut c_void);
    fn nvim_get_curwin() -> *mut c_void;
}

/// Compute the buffer number reached by stepping `offset` buffers from the
/// buffer at `lnum` in the buffer list.
///
/// Mirrors `nvim_docmd_compute_buffer_local_count_impl` in ex_docmd.c.
///
/// # Safety
/// Accesses C globals (firstbuf/buf list). Must only be called from C context.
#[export_name = "nvim_docmd_compute_buffer_local_count_impl"]
pub unsafe extern "C" fn rs_compute_buffer_local_count(
    addr_type: c_int,
    lnum: i32,
    offset: c_int,
) -> c_int {
    let mut count = offset;

    let mut buf = nvim_get_firstbuf();
    // Advance to the buffer with fnum >= lnum.
    while !nvim_buf_get_next(buf).is_null() && nvim_buf_get_fnum(buf) < lnum {
        buf = nvim_buf_get_next(buf);
    }

    while count != 0 {
        count += if count < 0 { 1 } else { -1 };
        let nextbuf = if offset < 0 {
            nvim_buf_get_prev(buf)
        } else {
            nvim_buf_get_next(buf)
        };
        if nextbuf.is_null() {
            break;
        }
        buf = nextbuf;
        if addr_type == ADDR_LOADED_BUFFERS {
            // skip over unloaded buffers
            loop {
                let nextbuf2 = if offset < 0 {
                    nvim_buf_get_prev(buf)
                } else {
                    nvim_buf_get_next(buf)
                };
                if nvim_buf_get_ml_mfp_null(buf) == 0 || nextbuf2.is_null() {
                    break;
                }
                buf = nextbuf2;
            }
        }
    }

    // We might have gone too far; last buffer is not loaded.
    if addr_type == ADDR_LOADED_BUFFERS {
        loop {
            let nextbuf = if offset >= 0 {
                nvim_buf_get_prev(buf)
            } else {
                nvim_buf_get_next(buf)
            };
            if nvim_buf_get_ml_mfp_null(buf) == 0 || nextbuf.is_null() {
                break;
            }
            buf = nextbuf;
        }
    }

    nvim_buf_get_fnum(buf)
}

/// Return the appropriate error message for an invalid address type.
///
/// Mirrors C `addr_error()`.
unsafe fn addr_error(addr_type: c_int) -> *const c_char {
    if addr_type == ADDR_NONE {
        crate::gt(crate::E_NORANGE_STR.as_ptr())
    } else {
        crate::gt(crate::E_INVRANGE_STR.as_ptr())
    }
}

/// Gets a single EX address.
///
/// This is the internal implementation, callable from other Rust code in this
/// crate (e.g. `parse_cmd_address`, `:tab` modifier).
///
/// Sets `*ptr` to the next character after the parsed address, or NULL on error.
/// Returns `MAXLNUM` when no address was found.
#[allow(clippy::too_many_arguments)]
pub unsafe fn get_address_impl(
    eap: ExArgHandle,
    ptr: *mut *mut c_char,
    addr_type: c_int,
    skip: bool,
    silent: bool,
    to_other_file: c_int,
    address_count: c_int,
    errormsg: *mut *const c_char,
) -> i32 {
    let mut cmd: *mut c_char = skipwhite(*ptr);
    let mut lnum: i32 = MAXLNUM;

    // do { ... } while (*cmd == '/' || *cmd == '?')
    loop {
        let switch_char = *cmd as u8;
        match switch_char {
            // '.' - Cursor position
            b'.' => {
                cmd = cmd.add(1);
                match addr_type {
                    ADDR_LINES | ADDR_OTHER => {
                        lnum = nvim_win_get_cursor_lnum(nvim_get_curwin());
                    }
                    ADDR_WINDOWS => {
                        lnum = nvim_docmd_current_win_nr() as i32;
                    }
                    ADDR_ARGUMENTS => {
                        lnum = nvim_docmd_get_curwin_arg_idx() as i32 + 1;
                    }
                    ADDR_LOADED_BUFFERS | ADDR_BUFFERS => {
                        lnum = nvim_docmd_get_curbuf_fnum() as i32;
                    }
                    ADDR_TABS => {
                        lnum = nvim_docmd_current_tab_nr() as i32;
                    }
                    ADDR_NONE | ADDR_TABS_RELATIVE | ADDR_UNSIGNED => {
                        *errormsg = addr_error(addr_type);
                        cmd = std::ptr::null_mut();
                        *ptr = cmd;
                        return lnum;
                    }
                    ADDR_QUICKFIX => {
                        lnum = qf_get_cur_idx(eap) as i32;
                    }
                    ADDR_QUICKFIX_VALID => {
                        lnum = qf_get_cur_valid_idx(eap) as i32;
                    }
                    _ => {}
                }
            }

            // '$' - last line
            b'$' => {
                cmd = cmd.add(1);
                match addr_type {
                    ADDR_LINES | ADDR_OTHER => {
                        lnum = nvim_buf_get_line_count(nvim_get_curbuf());
                    }
                    ADDR_WINDOWS => {
                        lnum = nvim_docmd_last_win_nr() as i32;
                    }
                    ADDR_ARGUMENTS => {
                        lnum = nvim_docmd_get_argcount() as i32;
                    }
                    ADDR_LOADED_BUFFERS => {
                        lnum = nvim_docmd_last_loaded_buf_fnum() as i32;
                    }
                    ADDR_BUFFERS => {
                        lnum = nvim_docmd_lastbuf_fnum() as i32;
                    }
                    ADDR_TABS => {
                        lnum = nvim_docmd_last_tab_nr() as i32;
                    }
                    ADDR_NONE | ADDR_TABS_RELATIVE | ADDR_UNSIGNED => {
                        *errormsg = addr_error(addr_type);
                        cmd = std::ptr::null_mut();
                        *ptr = cmd;
                        return lnum;
                    }
                    ADDR_QUICKFIX => {
                        lnum = qf_get_size(eap) as i32;
                        if lnum == 0 {
                            lnum = 1;
                        }
                    }
                    ADDR_QUICKFIX_VALID => {
                        lnum = qf_get_valid_size(eap) as i32;
                        if lnum == 0 {
                            lnum = 1;
                        }
                    }
                    _ => {}
                }
            }

            // '\'' - mark
            b'\'' => {
                cmd = cmd.add(1);
                if *cmd == NUL {
                    cmd = std::ptr::null_mut();
                    *ptr = cmd;
                    return lnum;
                }
                if addr_type != ADDR_LINES {
                    *errormsg = addr_error(addr_type);
                    cmd = std::ptr::null_mut();
                    *ptr = cmd;
                    return lnum;
                }
                if skip {
                    cmd = cmd.add(1);
                } else {
                    // Only accept a mark in another file when it is
                    // used by itself: ":'M".
                    let flag = if to_other_file != 0 && *cmd.add(1) == NUL {
                        K_MARK_ALL
                    } else {
                        K_MARK_BUF_LOCAL
                    };
                    let fm = nvim_docmd_mark_get(flag, *cmd as u8 as c_int);
                    cmd = cmd.add(1);
                    if !fm.is_null() && nvim_docmd_mark_fnum(fm) != nvim_docmd_get_curbuf_handle() {
                        mark_move_to(fm, 0);
                        // Jumped to another file.
                        lnum = nvim_win_get_cursor_lnum(nvim_get_curwin());
                    } else {
                        if !mark_check(fm, errormsg) {
                            cmd = std::ptr::null_mut();
                            *ptr = cmd;
                            return lnum;
                        }
                        // assert(fm != NULL) — mark_check succeeded so fm is valid
                        lnum = nvim_docmd_mark_lnum(fm);
                    }
                }
            }

            // '/' or '?' - search
            b'/' | b'?' => {
                let c = *cmd as u8 as c_int;
                cmd = cmd.add(1);
                if addr_type != ADDR_LINES {
                    *errormsg = addr_error(addr_type);
                    cmd = std::ptr::null_mut();
                    *ptr = cmd;
                    return lnum;
                }
                if skip {
                    // skip "/pat/"
                    cmd = rs_skip_regexp(cmd, c, nvim_docmd_magic_isset());
                    if *cmd as u8 as c_int == c {
                        cmd = cmd.add(1);
                    }
                } else {
                    // Save curwin->w_cursor
                    let save_lnum = nvim_win_get_cursor_lnum(nvim_get_curwin());
                    let save_col = nvim_win_get_cursor_col(nvim_get_curwin());

                    // When '/' or '?' follows another address, start from there.
                    if lnum > 0 && lnum != MAXLNUM {
                        let line_count = nvim_buf_get_line_count(nvim_get_curbuf());
                        let set_lnum = if lnum > line_count { line_count } else { lnum };
                        nvim_win_set_cursor_lnum(nvim_get_curwin(), set_lnum);
                    }

                    // Start a forward search at the end of the line (unless
                    // before the first line).
                    // Start a backward search at the start of the line.
                    let col =
                        if c == b'/' as c_int && nvim_win_get_cursor_lnum(nvim_get_curwin()) > 0 {
                            MAXCOL
                        } else {
                            0
                        };
                    nvim_win_set_cursor_col(nvim_get_curwin(), col);
                    searchcmdlen = 0;
                    let flags = if silent {
                        SEARCH_KEEP
                    } else {
                        SEARCH_HIS | SEARCH_MSG
                    };
                    let patlen = nvim_docmd_strlen(cmd);
                    if nvim_docmd_do_search(std::ptr::null_mut(), c, c, cmd, patlen, 1, flags) == 0
                    {
                        nvim_win_set_cursor_lnum(nvim_get_curwin(), save_lnum);
                        nvim_win_set_cursor_col(nvim_get_curwin(), save_col);
                        cmd = std::ptr::null_mut();
                        *ptr = cmd;
                        return lnum;
                    }
                    lnum = nvim_win_get_cursor_lnum(nvim_get_curwin());
                    nvim_win_set_cursor_lnum(nvim_get_curwin(), save_lnum);
                    nvim_win_set_cursor_col(nvim_get_curwin(), save_col);
                    // adjust command string pointer
                    cmd = cmd.offset(searchcmdlen as isize);
                }
            }

            // '\\' - "\?", "\/" or "\&", repeat search
            b'\\' => {
                cmd = cmd.add(1);
                if addr_type != ADDR_LINES {
                    *errormsg = addr_error(addr_type);
                    cmd = std::ptr::null_mut();
                    *ptr = cmd;
                    return lnum;
                }
                let i;
                if *cmd as u8 == b'&' {
                    i = RE_SUBST;
                } else if *cmd as u8 == b'?' || *cmd as u8 == b'/' {
                    i = RE_SEARCH;
                } else {
                    *errormsg = crate::gt(crate::E_BACKSLASH_STR.as_ptr());
                    cmd = std::ptr::null_mut();
                    *ptr = cmd;
                    return lnum;
                }

                if !skip {
                    // When search follows another address, start from there.
                    let start_lnum = if lnum != MAXLNUM {
                        lnum
                    } else {
                        nvim_win_get_cursor_lnum(nvim_get_curwin())
                    };
                    // Start the search just like for the above do_search().
                    let start_col = if *cmd as u8 != b'?' { MAXCOL } else { 0 };
                    let dir = if *cmd as u8 == b'?' {
                        BACKWARD
                    } else {
                        FORWARD
                    };
                    let result = nvim_docmd_searchit(dir, i, start_lnum, start_col, SEARCH_MSG);
                    if result != 0 {
                        lnum = result;
                    } else {
                        cmd = std::ptr::null_mut();
                        *ptr = cmd;
                        return lnum;
                    }
                }
                cmd = cmd.add(1);
            }

            // default: absolute line number
            _ => {
                if nvim_docmd_ascii_isdigit(*cmd as u8 as c_int) != 0 {
                    lnum = getdigits_int(&mut cmd, false, 0) as i32;
                }
            }
        }

        // Inner while(true) loop for +/- offset arithmetic
        loop {
            cmd = skipwhite(cmd);
            if *cmd as u8 != b'-'
                && *cmd as u8 != b'+'
                && nvim_docmd_ascii_isdigit(*cmd as u8 as c_int) == 0
            {
                break;
            }

            if lnum == MAXLNUM {
                match addr_type {
                    ADDR_LINES | ADDR_OTHER => {
                        // "+1" is same as ".+1"
                        lnum = nvim_win_get_cursor_lnum(nvim_get_curwin());
                    }
                    ADDR_WINDOWS => {
                        lnum = nvim_docmd_current_win_nr() as i32;
                    }
                    ADDR_ARGUMENTS => {
                        lnum = nvim_docmd_get_curwin_arg_idx() as i32 + 1;
                    }
                    ADDR_LOADED_BUFFERS | ADDR_BUFFERS => {
                        lnum = nvim_docmd_get_curbuf_fnum() as i32;
                    }
                    ADDR_TABS => {
                        lnum = nvim_docmd_current_tab_nr() as i32;
                    }
                    ADDR_TABS_RELATIVE => {
                        lnum = 1;
                    }
                    ADDR_QUICKFIX => {
                        lnum = qf_get_cur_idx(eap) as i32;
                    }
                    ADDR_QUICKFIX_VALID => {
                        lnum = qf_get_cur_valid_idx(eap) as i32;
                    }
                    ADDR_NONE | ADDR_UNSIGNED => {
                        lnum = 0;
                    }
                    _ => {}
                }
            }

            let i: u8;
            if nvim_docmd_ascii_isdigit(*cmd as u8 as c_int) != 0 {
                i = b'+'; // "number" is same as "+number"
            } else {
                i = *cmd as u8;
                cmd = cmd.add(1);
            }

            let n: i32;
            if nvim_docmd_ascii_isdigit(*cmd as u8 as c_int) == 0 {
                n = 1; // '+' is '+1'
            } else {
                // "number", "+number" or "-number"
                n = getdigits_int32(&mut cmd, false, i32::MAX);
                if n == MAXLNUM {
                    *errormsg = crate::gt(crate::E_LINE_NUMBER_OUT_OF_RANGE_STR.as_ptr());
                    cmd = std::ptr::null_mut();
                    *ptr = cmd;
                    return lnum;
                }
            }

            if addr_type == ADDR_TABS_RELATIVE {
                *errormsg = crate::gt(crate::E_INVRANGE_STR.as_ptr());
                cmd = std::ptr::null_mut();
                *ptr = cmd;
                return lnum;
            } else if addr_type == ADDR_LOADED_BUFFERS || addr_type == ADDR_BUFFERS {
                lnum = rs_compute_buffer_local_count(
                    addr_type,
                    lnum,
                    if i == b'-' { -(n as c_int) } else { n as c_int },
                ) as i32;
            } else {
                // Relative line addressing: need to adjust for lines in a
                // closed fold after the first address.
                if addr_type == ADDR_LINES && (i == b'-' || i == b'+') && address_count >= 2 {
                    lnum = nvim_docmd_hasFolding(lnum);
                }
                if i == b'-' {
                    lnum -= n;
                } else {
                    if lnum >= 0 && n >= INT32_MAX - lnum {
                        *errormsg = crate::gt(crate::E_LINE_NUMBER_OUT_OF_RANGE_STR.as_ptr());
                        cmd = std::ptr::null_mut();
                        *ptr = cmd;
                        return lnum;
                    }
                    lnum += n;
                }
            }
        }

        // do/while condition
        if *cmd as u8 != b'/' && *cmd as u8 != b'?' {
            break;
        }
    }

    // error: (fall through — always executed)
    *ptr = cmd;
    lnum
}

/// Gets a single EX address (FFI entry point).
///
/// Replaces C `get_address()`.
#[export_name = "get_address"]
pub unsafe extern "C" fn rs_get_address(
    eap: ExArgHandle,
    ptr: *mut *mut c_char,
    addr_type: c_int,
    skip: bool,
    silent: bool,
    to_other_file: c_int,
    address_count: c_int,
    errormsg: *mut *const c_char,
) -> i32 {
    get_address_impl(
        eap,
        ptr,
        addr_type,
        skip,
        silent,
        to_other_file,
        address_count,
        errormsg,
    )
}

// =============================================================================
// parse_cmd_address — parse Ex command address/range
// =============================================================================

const OK: c_int = 1;

/// Parse the address range for an Ex command.
///
/// Replaces C `parse_cmd_address()`.
#[export_name = "parse_cmd_address"]
pub unsafe extern "C" fn rs_parse_cmd_address(
    eap: ExArgHandle,
    errormsg: *mut *const c_char,
    silent: bool,
) -> c_int {
    let mut address_count: c_int = 1;
    let mut lnum: i32;
    let mut need_check_cursor = false;

    // Repeat for all ',' or ';' separated addresses.
    loop {
        (*eap).line1 = (*eap).line2;
        (*eap).line2 = rs_get_cmd_default_range(eap);
        let mut cmd = skipwhite((*eap).cmd);
        (*eap).cmd = cmd;
        let addr_type = (*eap).addr_type;
        let skip = (*eap).skip != 0;
        let to_other_file = if (*eap).addr_count == 0 { 1 } else { 0 };
        lnum = get_address_impl(
            eap,
            &mut cmd,
            addr_type,
            skip,
            silent,
            to_other_file,
            address_count,
            errormsg,
        );
        (*eap).cmd = cmd;
        address_count += 1;

        if (*eap).cmd.is_null() {
            // error detected
            if need_check_cursor {
                check_cursor(nvim_get_curwin());
            }
            return 0; // FAIL
        }

        if lnum == MAXLNUM {
            let cmd = (*eap).cmd;
            if *cmd as u8 == b'%' {
                // '%' - all lines
                (*eap).cmd = cmd.add(1);
                let addr_type = (*eap).addr_type;
                match addr_type {
                    ADDR_LINES | ADDR_OTHER => {
                        (*eap).line1 = 1;
                        (*eap).line2 = nvim_buf_get_line_count(nvim_get_curbuf());
                    }
                    ADDR_LOADED_BUFFERS => {
                        (*eap).line1 = nvim_docmd_first_loaded_buf_fnum() as i32;
                        (*eap).line2 = nvim_docmd_last_loaded_buf_fnum() as i32;
                    }
                    ADDR_BUFFERS => {
                        (*eap).line1 = nvim_docmd_firstbuf_fnum() as i32;
                        (*eap).line2 = nvim_docmd_lastbuf_fnum() as i32;
                    }
                    ADDR_WINDOWS | ADDR_TABS => {
                        if (*eap).cmdidx < 0 {
                            (*eap).line1 = 1;
                            let last = if addr_type == ADDR_WINDOWS {
                                nvim_docmd_last_win_nr()
                            } else {
                                nvim_docmd_last_tab_nr()
                            };
                            (*eap).line2 = last as i32;
                        } else {
                            // there is no Vim command which uses '%' and
                            // ADDR_WINDOWS or ADDR_TABS
                            *errormsg = crate::gt(crate::E_INVRANGE_STR.as_ptr());
                            if need_check_cursor {
                                check_cursor(nvim_get_curwin());
                            }
                            return 0; // FAIL
                        }
                    }
                    ADDR_TABS_RELATIVE | ADDR_UNSIGNED | ADDR_QUICKFIX => {
                        *errormsg = crate::gt(crate::E_INVRANGE_STR.as_ptr());
                        if need_check_cursor {
                            check_cursor(nvim_get_curwin());
                        }
                        return 0; // FAIL
                    }
                    ADDR_ARGUMENTS => {
                        let argcount = nvim_docmd_get_argcount();
                        if argcount == 0 {
                            (*eap).line1 = 0;
                            (*eap).line2 = 0;
                        } else {
                            (*eap).line1 = 1;
                            (*eap).line2 = argcount as i32;
                        }
                    }
                    ADDR_QUICKFIX_VALID => {
                        (*eap).line1 = 1;
                        let mut size = qf_get_valid_size(eap) as i32;
                        if size == 0 {
                            size = 1;
                        }
                        (*eap).line2 = size;
                    }
                    ADDR_NONE => {
                        // Will give an error later if a range is found.
                    }
                    _ => {}
                }
                let count = (*eap).addr_count + 1;
                (*eap).addr_count = count;
            } else if *cmd as u8 == b'*' {
                // '*' - visual area
                let addr_type = (*eap).addr_type;
                if addr_type != ADDR_LINES {
                    *errormsg = crate::gt(crate::E_INVRANGE_STR.as_ptr());
                    if need_check_cursor {
                        check_cursor(nvim_get_curwin());
                    }
                    return 0; // FAIL
                }

                (*eap).cmd = cmd.add(1);
                if (*eap).skip == 0 {
                    let fm = nvim_docmd_mark_get_visual(b'<' as c_int);
                    if !mark_check(fm, errormsg) {
                        if need_check_cursor {
                            check_cursor(nvim_get_curwin());
                        }
                        return 0; // FAIL
                    }
                    // assert(fm != NULL) — mark_check succeeded
                    (*eap).line1 = nvim_docmd_mark_lnum(fm);
                    let fm = nvim_docmd_mark_get_visual(b'>' as c_int);
                    if !mark_check(fm, errormsg) {
                        if need_check_cursor {
                            check_cursor(nvim_get_curwin());
                        }
                        return 0; // FAIL
                    }
                    // assert(fm != NULL)
                    (*eap).line2 = nvim_docmd_mark_lnum(fm);
                    let count = (*eap).addr_count + 1;
                    (*eap).addr_count = count;
                }
            }
        } else {
            (*eap).line2 = lnum;
        }

        let count = (*eap).addr_count + 1;
        (*eap).addr_count = count;
        let cmd = (*eap).cmd;
        if *cmd as u8 == b';' {
            if (*eap).skip == 0 {
                nvim_win_set_cursor_lnum(nvim_get_curwin(), (*eap).line2);
                // Don't leave the cursor on an illegal line or column, but do
                // accept zero as address, so 0;/PATTERN/ works correctly.
                if (*eap).line2 > 0 {
                    check_cursor(nvim_get_curwin());
                } else {
                    check_cursor_col(nvim_get_curwin());
                }
                need_check_cursor = true;
            }
        } else if *cmd as u8 != b',' {
            break;
        }
        (*eap).cmd = cmd.add(1);
    }

    // One address given: set start and end lines.
    if (*eap).addr_count == 1 {
        (*eap).line1 = (*eap).line2; // ... but only implicit: really no address given
        if lnum == MAXLNUM {
            (*eap).addr_count = 0;
        }
    }

    if need_check_cursor {
        check_cursor(nvim_get_curwin());
    }
    OK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addr_type_from_c_int() {
        assert_eq!(AddrType::from_c_int(0), Some(AddrType::Lines));
        assert_eq!(AddrType::from_c_int(1), Some(AddrType::Windows));
        assert_eq!(AddrType::from_c_int(11), Some(AddrType::None));
        assert_eq!(AddrType::from_c_int(12), Option::None);
        assert_eq!(AddrType::from_c_int(-1), Option::None);
    }

    #[test]
    fn test_addr_type_to_c_int() {
        assert_eq!(AddrType::Lines.to_c_int(), 0);
        assert_eq!(AddrType::Windows.to_c_int(), 1);
        assert_eq!(AddrType::None.to_c_int(), 11);
    }

    #[test]
    fn test_uses_line_numbers() {
        assert!(AddrType::Lines.uses_line_numbers());
        assert!(AddrType::Other.uses_line_numbers());
        assert!(!AddrType::Windows.uses_line_numbers());
        assert!(!AddrType::Buffers.uses_line_numbers());
    }

    #[test]
    fn test_uses_buffer_numbers() {
        assert!(AddrType::Buffers.uses_buffer_numbers());
        assert!(AddrType::LoadedBuffers.uses_buffer_numbers());
        assert!(!AddrType::Lines.uses_buffer_numbers());
    }

    #[test]
    fn test_uses_tab_numbers() {
        assert!(AddrType::Tabs.uses_tab_numbers());
        assert!(AddrType::TabsRelative.uses_tab_numbers());
        assert!(!AddrType::Lines.uses_tab_numbers());
    }

    #[test]
    fn test_uses_quickfix() {
        assert!(AddrType::Quickfix.uses_quickfix());
        assert!(AddrType::QuickfixValid.uses_quickfix());
        assert!(!AddrType::Lines.uses_quickfix());
    }

    #[test]
    fn test_ffi_addr_type_from_int() {
        assert_eq!(rs_addr_type_from_int(0), 0);
        assert_eq!(rs_addr_type_from_int(11), 11);
        assert_eq!(rs_addr_type_from_int(99), -1);
    }

    #[test]
    fn test_ffi_uses_functions() {
        assert_eq!(rs_addr_type_uses_lines(ADDR_LINES), 1);
        assert_eq!(rs_addr_type_uses_lines(ADDR_WINDOWS), 0);

        assert_eq!(rs_addr_type_uses_buffers(ADDR_BUFFERS), 1);
        assert_eq!(rs_addr_type_uses_buffers(ADDR_LINES), 0);

        assert_eq!(rs_addr_type_uses_windows(ADDR_WINDOWS), 1);
        assert_eq!(rs_addr_type_uses_windows(ADDR_LINES), 0);

        assert_eq!(rs_addr_type_uses_tabs(ADDR_TABS), 1);
        assert_eq!(rs_addr_type_uses_tabs(ADDR_LINES), 0);

        assert_eq!(rs_addr_type_uses_quickfix(ADDR_QUICKFIX), 1);
        assert_eq!(rs_addr_type_uses_quickfix(ADDR_LINES), 0);
    }
}
