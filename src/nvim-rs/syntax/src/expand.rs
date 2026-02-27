//! Tab completion for `:syntax` and `:echohl` commands.
//!
//! Migrated from syntax_accessors.c: set_context_in_syntax_cmd,
//! get_syntax_name, set_context_in_echohl_cmd, reset_expand_highlight.

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // expand_T field accessors (from option_shim.c)
    fn nvim_xp_set_context(xp: *mut c_void, val: c_int);
    fn nvim_xp_set_pattern(xp: *mut c_void, val: *mut c_char);
    fn nvim_xp_get_pattern(xp: *mut c_void) -> *mut c_char;

    // include_link/default/none setters
    fn nvim_syn_set_include_link(val: c_int);
    fn nvim_syn_set_include_default(val: c_int);
    fn nvim_syn_set_include_none(val: c_int);

    // expand_what get/set
    fn nvim_syn_get_expand_what() -> c_int;
    fn nvim_syn_set_expand_what(what: c_int);

    // cluster expansion: format @name into xp->xp_buf and return it
    fn nvim_syn_expand_cluster_name(xp: *mut c_void, idx: c_int) -> *mut c_char;
    fn nvim_syn_get_expand_cluster_count() -> c_int;

    // String helpers
    fn skiptowhite(s: *const c_char) -> *mut c_char;
    fn skipwhite(s: *const c_char) -> *mut c_char;
}

// expand_what values (match C enum in syntax_accessors.c)
const EXP_SUBCMD: c_int = 0;
const EXP_CASE: c_int = 1;
const EXP_SPELL: c_int = 2;
const EXP_SYNC: c_int = 3;
const EXP_CLUSTER: c_int = 4;

// EXPAND_* constants (from cmdexpand_defs.h)
// EXPAND_NOTHING=0, ..., EXPAND_MENUS=11, EXPAND_SYNTAX=12, EXPAND_HIGHLIGHT=13
const EXPAND_NOTHING: c_int = 0;
const EXPAND_SYNTAX: c_int = 12;
const EXPAND_HIGHLIGHT: c_int = 13;

// Subcommand names for :syntax tab completion (NUL-terminated byte slices).
// Mirrors the C subcommand_names[] array formerly in syntax_accessors.c.
pub(crate) static SUBCOMMAND_NAMES: &[&[u8]] = &[
    b"case\0",
    b"clear\0",
    b"cluster\0",
    b"conceal\0",
    b"enable\0",
    b"foldlevel\0",
    b"include\0",
    b"iskeyword\0",
    b"keyword\0",
    b"list\0",
    b"manual\0",
    b"match\0",
    b"on\0",
    b"off\0",
    b"region\0",
    b"reset\0",
    b"spell\0",
    b"sync\0",
    b"\0",
];

// Static argument arrays for tab completion (NUL-terminated byte slices)
static CASE_ARGS: [&[u8]; 2] = [b"match\0", b"ignore\0"];
static SPELL_ARGS: [&[u8]; 3] = [b"toplevel\0", b"notoplevel\0", b"default\0"];
static SYNC_ARGS: [&[u8]; 10] = [
    b"ccomment\0",
    b"clear\0",
    b"fromstart\0",
    b"linebreaks=\0",
    b"linecont\0",
    b"lines=\0",
    b"match\0",
    b"maxlines=\0",
    b"minlines=\0",
    b"region\0",
];

// =============================================================================
// Exported functions
// =============================================================================

/// Reset include_link, include_default, include_none to 0.
///
/// # Safety
/// Must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_reset_expand_highlight() {
    nvim_syn_set_include_link(0);
    nvim_syn_set_include_default(0);
    nvim_syn_set_include_none(0);
}

/// Set completion context for `:echohl` / `:match` command.
///
/// # Safety
/// `xp` must be a valid pointer to expand_T; `arg` must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_set_context_in_echohl_cmd(xp: *mut c_void, arg: *const c_char) {
    nvim_xp_set_context(xp, EXPAND_HIGHLIGHT);
    nvim_xp_set_pattern(xp, arg.cast_mut());
    nvim_syn_set_include_none(1);
}

/// Set completion context for `:syntax` command.
///
/// # Safety
/// `xp` must be a valid pointer to expand_T; `arg` must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_set_context_in_syntax_cmd(xp: *mut c_void, arg: *const c_char) {
    // Default: expand subcommands
    nvim_xp_set_context(xp, EXPAND_SYNTAX);
    nvim_syn_set_expand_what(EXP_SUBCMD);
    nvim_xp_set_pattern(xp, arg.cast_mut());
    nvim_syn_set_include_link(0);
    nvim_syn_set_include_default(0);

    if arg.is_null() || *arg == 0 {
        return;
    }

    // Find end of first word
    let p = skiptowhite(arg);
    if *p == 0 {
        // No space yet - still typing subcommand
        return;
    }

    // Past first word: set xp_pattern to start of second word
    let pattern = skipwhite(p);
    nvim_xp_set_pattern(xp, pattern);

    // Check if there's a third word (no completion for third word)
    if *skiptowhite(pattern) != 0 {
        nvim_xp_set_context(xp, EXPAND_NOTHING);
        return;
    }

    // Determine expansion type based on subcommand
    let cmd_len = p.offset_from(arg) as usize;

    if strnicmp_n(arg, b"case", cmd_len) {
        nvim_syn_set_expand_what(EXP_CASE);
    } else if strnicmp_n(arg, b"spell", cmd_len) {
        nvim_syn_set_expand_what(EXP_SPELL);
    } else if strnicmp_n(arg, b"sync", cmd_len) {
        nvim_syn_set_expand_what(EXP_SYNC);
    } else if strnicmp_n(arg, b"list", cmd_len) {
        // For "list @...", expand cluster names; otherwise expand highlight groups
        let p2 = skipwhite(p);
        if *p2 == b'@' as c_char {
            nvim_syn_set_expand_what(EXP_CLUSTER);
        } else {
            nvim_xp_set_context(xp, EXPAND_HIGHLIGHT);
        }
    } else if strnicmp_n(arg, b"keyword", cmd_len)
        || strnicmp_n(arg, b"region", cmd_len)
        || strnicmp_n(arg, b"match", cmd_len)
    {
        nvim_xp_set_context(xp, EXPAND_HIGHLIGHT);
    } else {
        nvim_xp_set_context(xp, EXPAND_NOTHING);
    }
}

/// Return tab-completion name for `:syntax` by index, based on expand_what.
///
/// # Safety
/// `xp` must be a valid pointer to expand_T.
#[no_mangle]
pub unsafe extern "C" fn rs_get_syntax_name(xp: *mut c_void, idx: c_int) -> *mut c_char {
    if idx < 0 {
        return std::ptr::null_mut();
    }

    let what = nvim_syn_get_expand_what();
    match what {
        w if w == EXP_SUBCMD => {
            let u = idx as usize;
            if u >= SUBCOMMAND_NAMES.len() {
                return std::ptr::null_mut();
            }
            SUBCOMMAND_NAMES[u].as_ptr().cast::<c_char>().cast_mut()
        }
        w if w == EXP_CASE => {
            let u = idx as usize;
            if u < CASE_ARGS.len() {
                CASE_ARGS[u].as_ptr().cast::<c_char>().cast_mut()
            } else {
                std::ptr::null_mut()
            }
        }
        w if w == EXP_SPELL => {
            let u = idx as usize;
            if u < SPELL_ARGS.len() {
                SPELL_ARGS[u].as_ptr().cast::<c_char>().cast_mut()
            } else {
                std::ptr::null_mut()
            }
        }
        w if w == EXP_SYNC => {
            let u = idx as usize;
            if u < SYNC_ARGS.len() {
                SYNC_ARGS[u].as_ptr().cast::<c_char>().cast_mut()
            } else {
                std::ptr::null_mut()
            }
        }
        w if w == EXP_CLUSTER => {
            let count = nvim_syn_get_expand_cluster_count();
            if idx < count {
                nvim_syn_expand_cluster_name(xp, idx)
            } else {
                std::ptr::null_mut()
            }
        }
        _ => std::ptr::null_mut(),
    }
}

// =============================================================================
// Helpers
// =============================================================================

/// Case-insensitive comparison of `s` (C string) with `expected` ASCII slice,
/// checking exactly `len` bytes.
unsafe fn strnicmp_n(s: *const c_char, expected: &[u8], len: usize) -> bool {
    if len != expected.len() {
        return false;
    }
    for (i, &expected_byte) in expected.iter().enumerate().take(len) {
        let c1 = (*s.add(i) as u8).to_ascii_lowercase();
        let c2 = expected_byte.to_ascii_lowercase();
        if c1 != c2 {
            return false;
        }
    }
    true
}
