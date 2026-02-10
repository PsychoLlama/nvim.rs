//! Address and range parsing types for Ex commands.
//!
//! This module defines types for command address/range parsing,
//! such as `1,5`, `%`, `'a,'b`, etc.

use std::ffi::{c_char, c_int};

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
    fn nvim_eap_get_cmdidx(eap: ExArgHandle) -> c_int;
    fn nvim_eap_get_addr_type(eap: ExArgHandle) -> c_int;
    fn nvim_eap_set_addr_type(eap: ExArgHandle, t: c_int);
    fn nvim_eap_get_addr_count(eap: ExArgHandle) -> c_int;
    fn nvim_eap_set_line1(eap: ExArgHandle, line: i32);
    fn nvim_eap_get_line2(eap: ExArgHandle) -> i32;
    fn nvim_eap_set_line2(eap: ExArgHandle, line: i32);
    fn nvim_eap_get_arg(eap: ExArgHandle) -> *mut c_char;
    fn nvim_eap_get_cmd(eap: ExArgHandle) -> *mut c_char;
    fn nvim_eap_set_errmsg(eap: ExArgHandle, msg: *mut c_char);
    fn nvim_eap_get_cmdlinep(eap: ExArgHandle) -> *mut *mut c_char;

    // CMD enum accessors
    fn nvim_docmd_cmd_size() -> c_int;
    fn nvim_docmd_cmd_wincmd() -> c_int;
    fn nvim_docmd_cmd_cc() -> c_int;
    fn nvim_docmd_cmd_ll() -> c_int;
    fn nvim_docmd_cmd_tabmove() -> c_int;
    fn nvim_docmd_cmd_tabnext() -> c_int;

    // cmdnames table accessor
    fn nvim_docmd_cmdnames_addr_type(idx: c_int) -> c_int;

    // bt_quickfix check
    fn nvim_docmd_bt_quickfix_curbuf() -> c_int;

    // Window/tab navigation
    fn nvim_docmd_current_win_nr() -> c_int;
    fn nvim_docmd_current_tab_nr() -> c_int;
    fn nvim_docmd_last_tab_nr() -> c_int;

    // Cursor and arg accessors
    fn nvim_docmd_get_curwin_cursor_lnum() -> i32;
    fn nvim_docmd_get_curwin_arg_idx() -> c_int;
    fn nvim_docmd_get_argcount() -> c_int;

    // Buffer accessors
    fn nvim_docmd_get_curbuf_line_count() -> i32;
    fn nvim_docmd_get_curbuf_fnum() -> c_int;

    // Quickfix accessors
    fn nvim_docmd_qf_get_cur_idx(eap: ExArgHandle) -> c_int;
    fn nvim_docmd_qf_get_cur_valid_idx(eap: ExArgHandle) -> c_int;
    fn nvim_docmd_qf_get_valid_size(eap: ExArgHandle) -> usize;

    // Buffer list walking
    fn nvim_docmd_first_loaded_buf_fnum() -> c_int;
    fn nvim_docmd_last_loaded_buf_fnum() -> c_int;
    fn nvim_docmd_firstbuf_fnum() -> c_int;
    fn nvim_docmd_lastbuf_fnum() -> c_int;

    // dflall error
    fn nvim_docmd_iemsg_dflall();

    // Tabpage accessors
    fn nvim_docmd_tabpage_index_curtab() -> c_int;
    fn nvim_docmd_valid_lastused_tabpage() -> c_int;
    fn nvim_docmd_tabpage_index_lastused() -> c_int;
    fn nvim_docmd_getdigits(pp: *mut *mut c_char, def: c_int) -> c_int;
    fn nvim_docmd_ex_errmsg_invargval(arg: *const c_char) -> *mut c_char;
    fn nvim_docmd_ex_errmsg_invarg2(arg: *const c_char) -> *mut c_char;
    fn nvim_docmd_get_e_invrange() -> *mut c_char;
    fn nvim_docmd_last_win_nr() -> c_int;
    fn nvim_docmd_ascii_iswhite(c: c_int) -> c_int;
    fn nvim_docmd_ascii_isdigit(c: c_int) -> c_int;
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
            nvim_eap_set_addr_type(eap, ADDR_OTHER);
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
            nvim_eap_set_addr_type(eap, ADDR_OTHER);
        }

        // buffer number → ADDR_BUFFERS
        b'^' => {
            nvim_eap_set_addr_type(eap, ADDR_BUFFERS);
        }
        x if x == CTRL_HAT => {
            nvim_eap_set_addr_type(eap, ADDR_BUFFERS);
        }

        // window number → ADDR_WINDOWS
        b'q' | b'c' | b'o' | b'w' | b'W' | b'x' => {
            nvim_eap_set_addr_type(eap, ADDR_WINDOWS);
        }
        x if x == CTRL_Q || x == CTRL_C || x == CTRL_O || x == CTRL_W || x == CTRL_X => {
            nvim_eap_set_addr_type(eap, ADDR_WINDOWS);
        }

        // no count → ADDR_NONE
        b'z' | b'P' | b't' | b'b' | b'p' | b'=' => {
            nvim_eap_set_addr_type(eap, ADDR_NONE);
        }
        x if x == CTRL_Z || x == CTRL_T || x == CTRL_B || x == CTRL_P || x == CAR => {
            nvim_eap_set_addr_type(eap, ADDR_NONE);
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
#[no_mangle]
pub unsafe extern "C" fn rs_set_cmd_addr_type(eap: ExArgHandle, p: *mut c_char) {
    let cmdidx = nvim_eap_get_cmdidx(eap);

    // User commands have addr_type set by find_ucmd
    if cmdidx < 0 {
        return;
    }

    let cmd_size = nvim_docmd_cmd_size();
    if cmdidx != cmd_size {
        let addr_type = nvim_docmd_cmdnames_addr_type(cmdidx);
        nvim_eap_set_addr_type(eap, addr_type);
    } else {
        nvim_eap_set_addr_type(eap, ADDR_LINES);
    }

    // :wincmd range depends on the argument
    if cmdidx == nvim_docmd_cmd_wincmd() && !p.is_null() {
        let whiteskipped = skipwhite(p);
        get_wincmd_addr_type(whiteskipped, eap);
    }

    // :.cc in quickfix window uses line number
    if (cmdidx == nvim_docmd_cmd_cc() || cmdidx == nvim_docmd_cmd_ll())
        && nvim_docmd_bt_quickfix_curbuf() != 0
    {
        nvim_eap_set_addr_type(eap, ADDR_OTHER);
    }
}

// =============================================================================
// rs_get_cmd_default_range
// =============================================================================

/// Get default range number for command based on its address type.
///
/// Replaces C `get_cmd_default_range()`.
#[no_mangle]
pub unsafe extern "C" fn rs_get_cmd_default_range(eap: ExArgHandle) -> i32 {
    let addr_type = nvim_eap_get_addr_type(eap);
    match addr_type {
        x if x == ADDR_LINES || x == ADDR_OTHER => {
            let cursor_lnum = nvim_docmd_get_curwin_cursor_lnum();
            let line_count = nvim_docmd_get_curbuf_line_count();
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
        x if x == ADDR_QUICKFIX => nvim_docmd_qf_get_cur_idx(eap) as i32,
        x if x == ADDR_QUICKFIX_VALID => nvim_docmd_qf_get_cur_valid_idx(eap) as i32,
        _ => 0,
    }
}

// =============================================================================
// rs_set_cmd_dflall_range
// =============================================================================

/// Set default-all range (% range) for all address types.
///
/// Replaces C `set_cmd_dflall_range()`.
#[no_mangle]
pub unsafe extern "C" fn rs_set_cmd_dflall_range(eap: ExArgHandle) {
    nvim_eap_set_line1(eap, 1);
    let addr_type = nvim_eap_get_addr_type(eap);

    match addr_type {
        x if x == ADDR_LINES || x == ADDR_OTHER => {
            nvim_eap_set_line2(eap, nvim_docmd_get_curbuf_line_count());
        }
        x if x == ADDR_LOADED_BUFFERS => {
            let first_fnum = nvim_docmd_first_loaded_buf_fnum();
            nvim_eap_set_line1(eap, first_fnum);
            let last_fnum = nvim_docmd_last_loaded_buf_fnum();
            nvim_eap_set_line2(eap, last_fnum);
        }
        x if x == ADDR_BUFFERS => {
            nvim_eap_set_line1(eap, nvim_docmd_firstbuf_fnum());
            nvim_eap_set_line2(eap, nvim_docmd_lastbuf_fnum());
        }
        x if x == ADDR_WINDOWS => {
            nvim_eap_set_line2(eap, nvim_docmd_last_win_nr() as i32);
        }
        x if x == ADDR_TABS => {
            nvim_eap_set_line2(eap, nvim_docmd_last_tab_nr() as i32);
        }
        x if x == ADDR_TABS_RELATIVE => {
            nvim_eap_set_line2(eap, 1);
        }
        x if x == ADDR_ARGUMENTS => {
            let argcount = nvim_docmd_get_argcount();
            if argcount == 0 {
                nvim_eap_set_line1(eap, 0);
                nvim_eap_set_line2(eap, 0);
            } else {
                nvim_eap_set_line2(eap, argcount as i32);
            }
        }
        x if x == ADDR_QUICKFIX_VALID => {
            let size = nvim_docmd_qf_get_valid_size(eap) as i32;
            nvim_eap_set_line2(eap, if size == 0 { 1 } else { size });
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
#[no_mangle]
pub unsafe extern "C" fn rs_get_tabpage_arg(eap: ExArgHandle) -> c_int {
    let cmdidx = nvim_eap_get_cmdidx(eap);
    let cmd_tabmove = nvim_docmd_cmd_tabmove();
    let unaccept_arg0: c_int = if cmdidx == cmd_tabmove { 0 } else { 1 };

    let arg = nvim_eap_get_arg(eap);
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
        let mut tab_number = nvim_docmd_getdigits(&mut p, 0);

        if relative == 0 {
            if *p as u8 == b'$' && *p.add(1) == 0 {
                tab_number = last_tab_nr;
            } else if *p as u8 == b'#' && *p.add(1) == 0 {
                if nvim_docmd_valid_lastused_tabpage() != 0 {
                    tab_number = nvim_docmd_tabpage_index_lastused();
                } else {
                    nvim_eap_set_errmsg(eap, nvim_docmd_ex_errmsg_invargval(arg));
                    return 0;
                }
            } else if p == p_save || *p_save as u8 == b'-' || *p != 0 || tab_number > last_tab_nr {
                nvim_eap_set_errmsg(eap, nvim_docmd_ex_errmsg_invarg2(arg));
                return 0;
            }
        } else {
            if *p_save == 0 {
                tab_number = 1;
            } else if p == p_save || *p_save as u8 == b'-' || *p != 0 || tab_number == 0 {
                nvim_eap_set_errmsg(eap, nvim_docmd_ex_errmsg_invarg2(arg));
                return 0;
            }
            tab_number = tab_number * relative + nvim_docmd_tabpage_index_curtab();
            if unaccept_arg0 == 0 && relative == -1 {
                tab_number -= 1;
            }
        }

        if tab_number < unaccept_arg0 || tab_number > last_tab_nr {
            nvim_eap_set_errmsg(eap, nvim_docmd_ex_errmsg_invarg2(arg));
        }
        tab_number
    } else if nvim_eap_get_addr_count(eap) > 0 {
        if unaccept_arg0 != 0 && nvim_eap_get_line2(eap) == 0 {
            nvim_eap_set_errmsg(eap, nvim_docmd_get_e_invrange());
            return 0;
        }
        let mut tab_number = nvim_eap_get_line2(eap);
        if unaccept_arg0 == 0 {
            let mut cmdp = nvim_eap_get_cmd(eap);
            let cmdlinep = nvim_eap_get_cmdlinep(eap);
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
                    nvim_eap_set_errmsg(eap, nvim_docmd_get_e_invrange());
                }
            }
        }
        tab_number
    } else {
        let cmd_tabnext = nvim_docmd_cmd_tabnext();
        if cmdidx == cmd_tabnext {
            let mut tab_number = nvim_docmd_tabpage_index_curtab() + 1;
            if tab_number > last_tab_nr {
                tab_number = 1;
            }
            tab_number
        } else if cmdidx == cmd_tabmove {
            last_tab_nr
        } else {
            nvim_docmd_tabpage_index_curtab()
        }
    }
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
