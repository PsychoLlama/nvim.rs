//! Incsearch pattern parsing for command-line completion.
//!
//! Provides `parse_pattern_and_range`, which determines the search pattern
//! and its location in the cmdline for incsearch highlighting.

use libc::{c_char, c_int};
use nvim_ex_cmds_types::{CmdMod, ExArg};

// =============================================================================
// External C/Rust functions
// =============================================================================

extern "C" {
    // Command-line buffer accessors (ex_getln.c)
    fn nvim_get_ccline_cmdbuff() -> *mut c_char;
    fn nvim_get_ccline_cmdlen() -> c_int;

    // vim_isIDc wrapper (charset.h - inline, needs C wrapper)
    fn nvim_cmdexpand_vim_isIDc(c: c_int) -> c_int;

    // Character utilities
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn vim_strchr(s: *const c_char, c: c_int) -> *const c_char;

    // Command parsing (exported from Rust with same C names)
    fn parse_command_modifiers(
        eap: *mut ExArg,
        errormsg: *mut *const c_char,
        cmod: *mut CmdMod,
        skip_only: c_int,
    ) -> c_int;
    fn skip_range(cmd: *const c_char, ctx: *mut c_int) -> *const c_char;
    fn skip_regexp_ex(
        p: *mut c_char,
        delim: c_int,
        magic: c_int,
        search_delim: *mut *mut c_char,
        result: *mut c_int,
        magic_out: *mut c_int,
    ) -> *mut c_char;
    fn parse_cmd_address(eap: *mut ExArg, errormsg: *mut *const c_char, silent: bool) -> c_int;

    // Pattern helpers (from cmdline crate)
    fn rs_empty_pattern_magic(p: *const c_char, len: usize, magic_val: c_int) -> c_int;
    fn rs_magic_isset() -> c_int;

    // search_first_line / search_last_line setters (search_shim.c)
    fn nvim_set_search_first_line(value: i32);
    fn nvim_set_search_last_line(value: i32);

    // Cursor position save/restore via direct WinStruct access
    fn nvim_get_curwin() -> nvim_window::WinHandle;

    // magic_overruled global (in C globals.h, via search)
    static mut magic_overruled: c_int;
}

/// Position in a buffer (mirrors `pos_T`).
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PosT {
    lnum: i32,
    col: i32,
    coladd: i32,
}

/// Copy curwin->w_cursor into *pos.
#[inline]
unsafe fn get_cursor_pos(pos: *mut PosT) {
    let cur = nvim_window::win_struct::win_ref(nvim_get_curwin());
    (*pos).lnum = cur.w_cursor.lnum;
    (*pos).col = cur.w_cursor.col;
    (*pos).coladd = cur.w_cursor.coladd;
}

/// Set curwin->w_cursor from *pos.
#[inline]
unsafe fn set_cursor_pos(pos: *const PosT) {
    let cur = nvim_window::win_struct::win_mut(nvim_get_curwin());
    cur.w_cursor.lnum = (*pos).lnum;
    cur.w_cursor.col = (*pos).col;
    cur.w_cursor.coladd = (*pos).coladd;
}

// Constants
const ADDR_LINES: c_int = 0;
const MAXLNUM: i32 = 0x7fff_ffff;
const OPTION_MAGIC_ON: c_int = 1;
const OPTION_MAGIC_OFF: c_int = 2;

/// Check if a byte is an ASCII alphabetic character (A-Z, a-z).
#[inline]
const fn ascii_isalpha(c: u8) -> bool {
    c.is_ascii_alphabetic()
}

/// Parse the command pattern and range for incsearch highlighting.
///
/// Sets `search_first_line`, `search_last_line`, and the skip/pat lengths.
/// Returns true if a valid non-empty pattern was found, false otherwise.
///
/// Rust replacement for C `parse_pattern_and_range`.
///
/// # Safety
///
/// `incsearch_start` must be a valid pointer to a `pos_T`.
/// `search_delim`, `skiplen`, `patlen` must be valid non-null output pointers.
#[allow(clippy::too_many_lines)]
#[unsafe(export_name = "parse_pattern_and_range")]
pub unsafe extern "C" fn rs_parse_pattern_and_range(
    incsearch_start: *mut PosT,
    search_delim: *mut c_int,
    skiplen: *mut c_int,
    patlen: *mut c_int,
) -> bool {
    let mut delim_optional = false;
    let mut magic: c_int = 0;

    *skiplen = 0;
    *patlen = nvim_get_ccline_cmdlen();

    // Default range: search all lines
    nvim_set_search_first_line(0);
    nvim_set_search_last_line(MAXLNUM);

    // Build a minimal ExArg on the stack
    let mut ea = std::mem::zeroed::<ExArg>();
    ea.line1 = 1;
    ea.line2 = 1;
    ea.cmd = nvim_get_ccline_cmdbuff();
    ea.addr_type = ADDR_LINES;

    let mut dummy_cmod = std::mem::zeroed::<CmdMod>();
    let mut dummy_errormsg: *const c_char = std::ptr::null();

    // Skip over command modifiers (skip_only = 1 means don't apply them)
    parse_command_modifiers(&raw mut ea, &raw mut dummy_errormsg, &raw mut dummy_cmod, 1);

    // Skip over the range to find the command
    let cmd = skip_range(ea.cmd, std::ptr::null_mut()).cast_mut();
    if cmd.is_null() {
        return false;
    }

    // Check if the command is one of: s, g, v, l (or longer forms)
    let first_char = *cmd as u8;
    if vim_strchr(c"sgvl".as_ptr(), c_int::from(first_char)).is_null() {
        return false;
    }

    // Skip over command name to find pattern separator
    let mut p: *mut c_char = cmd;
    while ascii_isalpha(*p as u8) {
        p = p.add(1);
    }

    let p_ws = skipwhite(p);
    if *p_ws == 0 {
        return false;
    }

    // Determine which command this is and set magic/delim_optional accordingly
    let cmd_len = p.offset_from(cmd) as usize;
    let cmd_bytes = std::slice::from_raw_parts(cmd as *const u8, cmd_len);

    if cmd_matches(cmd_bytes, b"substitute")
        || cmd_matches(cmd_bytes, b"smagic")
        || cmd_matches_min(cmd_bytes, b"snomagic", 3)
        || cmd_matches(cmd_bytes, b"vglobal")
    {
        if first_char == b's' && cmd_len >= 2 && *cmd.add(1) as u8 == b'm' {
            magic_overruled = OPTION_MAGIC_ON;
        } else if first_char == b's' && cmd_len >= 2 && *cmd.add(1) as u8 == b'n' {
            magic_overruled = OPTION_MAGIC_OFF;
        }
    } else if cmd_matches_min(cmd_bytes, b"sort", 3) || cmd_matches_min(cmd_bytes, b"uniq", 3) {
        // Skip over ! and flags
        if *p as u8 == b'!' {
            p = skipwhite(p.add(1));
        }
        loop {
            p = skipwhite(p);
            if !ascii_isalpha(*p as u8) {
                break;
            }
            p = p.add(1);
        }
        if *p == 0 {
            return false;
        }
    } else if cmd_matches_min(cmd_bytes, b"vimgrep", 3)
        || cmd_matches_min(cmd_bytes, b"vimgrepadd", 8)
        || cmd_matches_min(cmd_bytes, b"lvimgrep", 2)
        || cmd_matches_min(cmd_bytes, b"lvimgrepadd", 9)
        || cmd_matches(cmd_bytes, b"global")
    {
        // Skip optional "!"
        if *p as u8 == b'!' {
            p = p.add(1);
            if *skipwhite(p) == 0 {
                return false;
            }
        }
        if first_char != b'g' {
            delim_optional = true;
        }
    } else {
        return false;
    }

    p = skipwhite(p);
    let delim: c_int = if delim_optional && nvim_cmdexpand_vim_isIDc(c_int::from(*p as u8)) != 0 {
        c_int::from(b' ')
    } else {
        let d = c_int::from(*p as u8);
        p = p.add(1);
        d
    };
    *search_delim = delim;

    let end = skip_regexp_ex(
        p,
        delim,
        rs_magic_isset(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        &raw mut magic,
    );

    let use_last_pat = end == p && !end.is_null() && c_int::from(*end) == delim;

    if end == p && !use_last_pat {
        return false;
    }

    // Check if the pattern matches everything (empty pattern)
    if !use_last_pat {
        let c = *end;
        *end = 0;
        let empty = rs_empty_pattern_magic(p, end.offset_from(p) as usize, magic) != 0;
        *end = c;
        if empty {
            return false;
        }
    }

    // Found a non-empty pattern
    *skiplen = p.offset_from(nvim_get_ccline_cmdbuff()) as c_int;
    *patlen = end.offset_from(p) as c_int;

    // Parse the address range (cursor at incsearch_start)
    let mut save_cursor = PosT::default();
    get_cursor_pos(&raw mut save_cursor);
    set_cursor_pos(incsearch_start);

    parse_cmd_address(&raw mut ea, &raw mut dummy_errormsg, true);

    if ea.addr_count > 0 {
        // Allow for reverse match
        let first = ea.line1.min(ea.line2);
        let last = ea.line1.max(ea.line2);
        nvim_set_search_first_line(first);
        nvim_set_search_last_line(last);
    } else if first_char == b's' && *cmd.add(1) as u8 != b'o' {
        // :s defaults to current line. The cursor was set to incsearch_start above,
        // so curwin->w_cursor.lnum == incsearch_start->lnum at this point.
        let lnum = (*incsearch_start).lnum;
        nvim_set_search_first_line(lnum);
        nvim_set_search_last_line(lnum);
    }

    set_cursor_pos(&raw const save_cursor);

    true
}

/// Check if `cmd` is a prefix of `keyword` (any length >= 1).
/// Equivalent to C: `strncmp(cmd, keyword, cmd_len) == 0`.
fn cmd_matches(cmd: &[u8], keyword: &[u8]) -> bool {
    let len = cmd.len();
    len <= keyword.len() && &keyword[..len] == cmd
}

/// Check if `cmd` is a prefix of `keyword` with at least `min_len` characters.
/// Equivalent to C: `strncmp(cmd, keyword, MAX(cmd_len, min_len)) == 0`
/// where `cmd_len` == `cmd.len()`.
fn cmd_matches_min(cmd: &[u8], keyword: &[u8], min_len: usize) -> bool {
    let cmd_len = cmd.len();
    // Must provide at least min_len chars
    if cmd_len < min_len {
        return false;
    }
    // cmd must be a prefix of keyword
    cmd_len <= keyword.len() && &keyword[..cmd_len] == cmd
}
