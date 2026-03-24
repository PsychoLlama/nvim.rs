//! Command-line completion and expansion for Neovim
//!
//! This crate provides the command-line completion engine, including:
//! - Wildcard expansion
//! - Completion source management
//! - Fuzzy matching integration
//! - Popup menu support for completions

#![allow(unsafe_code)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

pub mod callbacks;
pub mod context;
pub mod context_helpers;
pub mod display;
pub mod expand;
pub mod files;
pub mod helpers;
pub mod navigation;
pub mod pattern;
pub mod pum;
pub mod set_context;
pub mod viml;
pub mod wildmenu;

pub use context::*;

use libc::{c_char, c_int};
use std::ffi::CStr;

// =============================================================================
// VimL type constants (from eval/typval_defs.h)
// =============================================================================

/// `VAR_UNKNOWN` — unspecified value type.
pub const VAR_UNKNOWN: c_int = 0;
/// `VAR_NUMBER` — number value type.
pub const VAR_NUMBER: c_int = 1;
/// `VAR_STRING` — string value type.
pub const VAR_STRING: c_int = 2;

// =============================================================================
// expand_T repr(C) struct
// =============================================================================

/// Script context (matches `sctx_T`).
/// Layout: `sc_sid:i32@0`, `sc_seq:i32@4`, `sc_lnum:i32@8`, pad:4@12, `sc_chan:u64@16` = 24 bytes.
#[repr(C)]
pub struct SctxT {
    pub sc_sid: i32,
    pub sc_seq: i32,
    pub sc_lnum: i32,
    _pad: i32,
    pub sc_chan: u64,
}

/// Position in file or buffer (matches `pos_T`).
/// Layout: lnum:i32@0, col:i32@4, coladd:i32@8 = 12 bytes.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PosT {
    pub lnum: i32,
    pub col: i32,
    pub coladd: i32,
}

/// Command-line expansion struct (matches `expand_T` from `cmdexpand_defs.h`).
///
/// Layout verified by `_Static_assert` in `cmdexpand.c`:
/// - `sizeof(expand_T)` == 392
/// - `xp_pattern`@0, `xp_context`@8, `xp_pattern_len`@16, `xp_prefix`@24
/// - `xp_arg`@32, `xp_luaref`@40, `xp_script_ctx`@48, `xp_backslash`@72
/// - `xp_shell`@76, `xp_numfiles`@80, `xp_col`@84, `xp_selected`@88
/// - `xp_orig`@96, `xp_files`@104, `xp_line`@112, `xp_buf`@120
/// - `xp_search_dir`@376, `xp_pre_incsearch_pos`@380
#[repr(C)]
pub struct ExpandT {
    pub xp_pattern: *mut c_char, // offset 0
    pub xp_context: c_int,       // offset 8
    _pad1: i32,
    pub xp_pattern_len: usize, // offset 16
    pub xp_prefix: c_int,      // offset 24 (xp_prefix_T enum)
    _pad2: i32,
    pub xp_arg: *mut c_char, // offset 32
    pub xp_luaref: c_int,    // offset 40 (LuaRef = int)
    _pad3: i32,
    pub xp_script_ctx: SctxT, // offset 48, 24 bytes
    pub xp_backslash: c_int,  // offset 72
    pub xp_shell: bool,       // offset 76 (Linux only, #ifndef BACKSLASH_IN_FILENAME)
    _pad4: [u8; 3],
    pub xp_numfiles: c_int, // offset 80
    pub xp_col: c_int,      // offset 84
    pub xp_selected: c_int, // offset 88
    _pad5: i32,
    pub xp_orig: *mut c_char,       // offset 96
    pub xp_files: *mut *mut c_char, // offset 104
    pub xp_line: *mut c_char,       // offset 112
    pub xp_buf: [c_char; 256],      // offset 120
    pub xp_search_dir: c_int,       // offset 376 (Direction enum)
    pub xp_pre_incsearch_pos: PosT, // offset 380, 12 bytes = 392 total
}

/// Handle to `expand_T` (C struct).
pub type ExpandHandle = *mut ExpandT;

impl ExpandT {
    /// Create a zeroed `ExpandT` instance (safe to pass to `ExpandInit`).
    #[must_use]
    pub const fn zeroed() -> Self {
        // SAFETY: ExpandT is repr(C) with all-bytes-zero as a valid initial state
        // before ExpandInit() is called. Pointer fields are null (safe), numeric
        // fields are zero (valid for their respective types).
        unsafe { std::mem::zeroed() }
    }
}

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    fn nvim_get_wop_flags() -> libc::c_uint;
    fn nvim_get_compl_match_array_not_null() -> c_int;

    // CmdlineInfo accessors for set_expand_context
    fn nvim_cmdexpand_get_cmdfirstc() -> c_int;
    fn nvim_cmdexpand_get_input_fn() -> c_int;
    fn nvim_cmdexpand_get_cmdbuff() -> *mut c_char;
    fn nvim_cmdexpand_get_cmdlen() -> c_int;
    fn nvim_cmdexpand_get_cmdpos() -> c_int;
    fn nvim_cmdexpand_get_may_expand_pattern() -> c_int;
    fn nvim_cmdexpand_set_search_first_line(val: c_int);

    // CmdlineInfo accessors for set_cmd_context
    fn nvim_cmdexpand_get_ccline_xp_context() -> c_int;
    fn nvim_cmdexpand_get_ccline_xp_arg() -> *mut c_char;
    fn nvim_cmdexpand_set_context_for_expression(
        xp: *mut ExpandT,
        str_: *mut c_char,
        cmdidx: c_int,
    );

    // Rust context helpers (already in Rust, called via C FFI)
    fn rs_set_context_for_wildcard_arg(
        arg: *const c_char,
        is_shell_cmd: c_int,
        xp: *mut ExpandT,
        complp: *mut c_int,
    );
    fn rs_set_one_cmd_context(xp: *mut ExpandT, buff: *const c_char) -> *const c_char;

    // expand_cmdline helpers
    fn nvim_cmdexpand_addstar(fname: *mut c_char, len: usize, context: c_int) -> *mut c_char;
    fn nvim_cmdexpand_get_p_wic() -> c_int;
    fn nvim_cmdexpand_expand_from_context(
        xp: *mut ExpandT,
        pat: *const c_char,
        options: c_int,
    ) -> c_int;
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xfree(ptr: *mut libc::c_void);
    fn beep_flush();
}

// =============================================================================
// Fuzzy completion support
// =============================================================================

/// Returns true if fuzzy completion is supported for the given context.
///
/// Not all completion contexts support fuzzy matching. This function
/// checks the context type and returns whether fuzzy completion can be used.
#[must_use]
pub const fn cmdline_fuzzy_completion_supported(context: i32) -> bool {
    // These contexts do NOT support fuzzy completion
    let Some(ctx) = ExpandContext::from_raw(context) else {
        return false;
    };

    !matches!(
        ctx,
        ExpandContext::BoolSettings
            | ExpandContext::Colors
            | ExpandContext::Compiler
            | ExpandContext::Directories
            | ExpandContext::DirsInCdpath
            | ExpandContext::Files
            | ExpandContext::FilesInPath
            | ExpandContext::Filetype
            | ExpandContext::Filetypecmd
            | ExpandContext::Findfunc
            | ExpandContext::Help
            | ExpandContext::Keymap
            | ExpandContext::Lua
            | ExpandContext::OldSetting
            | ExpandContext::StringSetting
            | ExpandContext::SettingSubtract
            | ExpandContext::Ownsyntax
            | ExpandContext::Packadd
            | ExpandContext::Runtime
            | ExpandContext::Shellcmd
            | ExpandContext::Shellcmdline
            | ExpandContext::Tags
            | ExpandContext::TagsListfiles
            | ExpandContext::UserList
            | ExpandContext::UserLua
    )
}

/// Check if fuzzy completion is enabled and the pattern is not empty.
///
/// Returns true if:
/// 1. The 'wildoptions' setting has the fuzzy flag set
/// 2. The fuzzy string is not empty
#[must_use]
pub fn cmdline_fuzzy_complete(fuzzystr: &str) -> bool {
    if fuzzystr.is_empty() {
        return false;
    }

    // Check if fuzzy flag is set in wildoptions
    // SAFETY: nvim_get_wop_flags is a simple accessor that reads a global variable
    let wop_flags = unsafe { nvim_get_wop_flags() };
    (wop_flags & K_OPT_WOP_FLAG_FUZZY) != 0
}

/// Check if the cmdline popup menu is active.
#[must_use]
pub fn cmdline_pum_active() -> bool {
    // SAFETY: nvim_get_compl_match_array_not_null is a simple accessor
    unsafe { nvim_get_compl_match_array_not_null() != 0 }
}

// =============================================================================
// set_cmd_context
// =============================================================================

/// `CMD_SIZE` sentinel value for `set_context_for_expression`.
const CMD_SIZE: c_int = 556;

/// `EXPAND_SHELLCMDLINE` context value.
const EXPAND_SHELLCMDLINE: c_int = ExpandContext::Shellcmdline.to_raw();

/// Parse command line string to set xp expansion context.
///
/// Sets `xp->xp_context` and related fields based on the command line text.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle. `str_` must be a valid C string of
/// at least `len` bytes. Must be called from cmdline context when `use_ccline` != 0.
#[unsafe(export_name = "set_cmd_context")]
pub unsafe extern "C" fn rs_set_cmd_context(
    xp: *mut ExpandT,
    str_: *mut c_char,
    len: c_int,
    col: c_int,
    use_ccline: c_int,
) {
    // Avoid a UMR warning from Purify, only save the character if it has been written before.
    let old_char: c_char = if col < len {
        *str_.add(col as usize)
    } else {
        0
    };
    *str_.add(col as usize) = 0; // NUL-terminate at cursor position

    if use_ccline != 0 && nvim_cmdexpand_get_cmdfirstc() == c_int::from(b'=') {
        // pass CMD_SIZE because there is no real command
        nvim_cmdexpand_set_context_for_expression(xp, str_, CMD_SIZE);
    } else if use_ccline != 0 && nvim_cmdexpand_get_input_fn() != 0 {
        (*xp).xp_context = nvim_cmdexpand_get_ccline_xp_context();
        (*xp).xp_pattern = nvim_cmdexpand_get_cmdbuff();
        (*xp).xp_arg = nvim_cmdexpand_get_ccline_xp_arg();
        if (*xp).xp_context == EXPAND_SHELLCMDLINE {
            let mut ctx = (*xp).xp_context;
            rs_set_context_for_wildcard_arg((*xp).xp_pattern, 0, xp, &raw mut ctx);
        }
    } else {
        let mut nextcomm: *const c_char = str_;
        while !nextcomm.is_null() {
            nextcomm = rs_set_one_cmd_context(xp, nextcomm);
        }
    }

    // Store the string here so that call_user_expand_func() can get to them easily.
    (*xp).xp_line = str_;
    (*xp).xp_col = col;

    *str_.add(col as usize) = old_char;
}

// =============================================================================
// expand_cmdline
// =============================================================================

/// Return values from `expand_cmdline`.
const EXPAND_UNSUCCESSFUL: c_int = ExpandContext::Unsuccessful.to_raw();
const EXPAND_NOTHING: c_int = ExpandContext::Nothing.to_raw();
const EXPAND_OK: c_int = ExpandContext::Ok.to_raw();

/// Expand the command line `str` from context `xp`.
///
/// `xp` must have been set by `set_cmd_context()`.
/// `xp->xp_pattern` points into `str`, to where the text to be expanded starts.
///
/// Returns `EXPAND_UNSUCCESSFUL` when there is something illegal before the cursor.
/// Returns `EXPAND_NOTHING` when there is nothing to expand.
/// Returns `EXPAND_OK` otherwise.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle. `str_` must be a valid C string pointer.
#[unsafe(export_name = "expand_cmdline")]
pub unsafe extern "C" fn rs_expand_cmdline(
    xp: *mut ExpandT,
    str_: *const c_char,
    col: c_int,
    matchcount: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    use crate::context::wild_options::{WILD_ADD_SLASH, WILD_ICASE, WILD_SILENT};

    if (*xp).xp_context == EXPAND_UNSUCCESSFUL {
        beep_flush();
        return EXPAND_UNSUCCESSFUL;
    }
    if (*xp).xp_context == EXPAND_NOTHING {
        return EXPAND_NOTHING;
    }

    // add star to file name, or convert to regexp if not exp. files.
    let pattern_offset = (str_.add(col as usize) as usize).wrapping_sub((*xp).xp_pattern as usize);
    (*xp).xp_pattern_len = pattern_offset;

    let file_str = if cmdline_fuzzy_completion_supported((*xp).xp_context) {
        // If fuzzy matching, don't modify the search string
        xstrdup((*xp).xp_pattern)
    } else {
        nvim_cmdexpand_addstar((*xp).xp_pattern, (*xp).xp_pattern_len, (*xp).xp_context)
    };

    let mut options = WILD_ADD_SLASH | WILD_SILENT;
    if nvim_cmdexpand_get_p_wic() != 0 {
        options |= WILD_ICASE;
    }

    // find all files that match the description
    let fail = nvim_cmdexpand_expand_from_context(xp, file_str, options);
    if fail != 0 {
        // FAIL
        *matchcount = 0;
        *matches = std::ptr::null_mut();
    }
    xfree(file_str.cast());

    EXPAND_OK
}

// =============================================================================
// set_expand_context
// =============================================================================

/// Must parse the command line so far to work out what context we are in.
///
/// Sets xp->xp_context and related fields for command-line completion.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle. Must be called from cmdline context.
#[unsafe(export_name = "set_expand_context")]
pub unsafe extern "C" fn rs_set_expand_context(xp: *mut ExpandT) {
    if xp.is_null() {
        return;
    }

    let cmdfirstc = nvim_cmdexpand_get_cmdfirstc();
    let may_expand = nvim_cmdexpand_get_may_expand_pattern() != 0;

    // Handle search commands: '/' or '?'
    if (cmdfirstc == c_int::from(b'/') || cmdfirstc == c_int::from(b'?')) && may_expand {
        (*xp).xp_context = ExpandContext::PatternInBuf.to_raw();
        // FORWARD=1 when '/', BACKWARD=0 (but actual enum values: FORWARD=1, BACKWARD=0)
        (*xp).xp_search_dir = i32::from(cmdfirstc == c_int::from(b'/'));
        (*xp).xp_pattern = nvim_cmdexpand_get_cmdbuff();
        (*xp).xp_pattern_len = nvim_cmdexpand_get_cmdpos() as usize;
        nvim_cmdexpand_set_search_first_line(0); // Search entire buffer
        return;
    }

    let input_fn = nvim_cmdexpand_get_input_fn() != 0;

    // Only handle ':', '>', or '=' command-lines, or expression input
    if cmdfirstc != c_int::from(b':')
        && cmdfirstc != c_int::from(b'>')
        && cmdfirstc != c_int::from(b'=')
        && !input_fn
    {
        (*xp).xp_context = ExpandContext::Nothing.to_raw();
        return;
    }

    // Fallback to command-line expansion
    let cmdbuff = nvim_cmdexpand_get_cmdbuff();
    let cmdlen = nvim_cmdexpand_get_cmdlen();
    let cmdpos = nvim_cmdexpand_get_cmdpos();
    rs_set_cmd_context(xp, cmdbuff, cmdlen, cmdpos, 1);
}

// =============================================================================
// FFI Interface
// =============================================================================

/// Convert C string pointer to Rust &str
///
/// # Safety
///
/// `ptr` must be a valid null-terminated C string or null.
unsafe fn cstr_to_str<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    CStr::from_ptr(ptr).to_str().ok()
}

/// Check if fuzzy completion is enabled for the given string (FFI version).
///
/// # Safety
///
/// `fuzzystr` must be a valid null-terminated C string or null.
#[must_use]
#[unsafe(export_name = "cmdline_fuzzy_complete")]
pub unsafe extern "C" fn rs_cmdline_fuzzy_complete(fuzzystr: *const c_char) -> c_int {
    let Some(s) = cstr_to_str(fuzzystr) else {
        return 0;
    };

    c_int::from(cmdline_fuzzy_complete(s))
}

/// Check if cmdline popup menu is active (FFI version).
#[must_use]
#[unsafe(export_name = "cmdline_pum_active")]
pub extern "C" fn rs_cmdline_pum_active() -> c_int {
    c_int::from(cmdline_pum_active())
}

/// Check if fuzzy completion is supported for the given context (FFI version).
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmdline_fuzzy_completion_supported(context: c_int) -> c_int {
    c_int::from(cmdline_fuzzy_completion_supported(context))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_completion_supported() {
        // Files/directories do NOT support fuzzy completion
        assert!(!cmdline_fuzzy_completion_supported(
            ExpandContext::Files.to_raw()
        ));
        assert!(!cmdline_fuzzy_completion_supported(
            ExpandContext::Directories.to_raw()
        ));
        assert!(!cmdline_fuzzy_completion_supported(
            ExpandContext::Help.to_raw()
        ));
        assert!(!cmdline_fuzzy_completion_supported(
            ExpandContext::Tags.to_raw()
        ));

        // Commands and other contexts DO support fuzzy completion
        assert!(cmdline_fuzzy_completion_supported(
            ExpandContext::Commands.to_raw()
        ));
        assert!(cmdline_fuzzy_completion_supported(
            ExpandContext::Buffers.to_raw()
        ));
        assert!(cmdline_fuzzy_completion_supported(
            ExpandContext::Functions.to_raw()
        ));

        // Invalid context
        assert!(!cmdline_fuzzy_completion_supported(999));
    }
}
