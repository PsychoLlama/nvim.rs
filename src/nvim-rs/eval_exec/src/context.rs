//! Completion context setting for VimL expressions.
//!
//! Migrated from `src/nvim/eval_shim.c`: `set_context_for_expression`.

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Handle types
// =============================================================================

/// Opaque handle to an expand_T struct.
pub type ExpandHandle = *mut c_void;

// =============================================================================
// Expand context constants (from cmdexpand_defs.h)
// =============================================================================

const EXPAND_NOTHING: c_int = 0;
const EXPAND_COMMANDS: c_int = 1;
const EXPAND_SETTINGS: c_int = 4;
const EXPAND_USER_VARS: c_int = 15;
const EXPAND_FUNCTIONS: c_int = 18;
const EXPAND_EXPRESSION: c_int = 20;
const EXPAND_ENV_VARS: c_int = 26;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    // expand_T field accessors
    fn nvim_expand_set_context(xp: ExpandHandle, context: c_int);
    fn nvim_expand_set_pattern(xp: ExpandHandle, pattern: *mut c_char);
    fn nvim_expand_get_context(xp: ExpandHandle) -> c_int;
    fn nvim_cmdexpand_get_xp_pattern(xp: ExpandHandle) -> *mut c_char;

    // CMD_* enum value accessors
    fn nvim_docmd_cmd_let() -> c_int;
    fn nvim_docmd_cmd_const() -> c_int;
    fn nvim_docmd_cmd_call() -> c_int;

    // String utility functions
    fn nvim_vim_strchr(s: *const c_char, c: c_int) -> *const c_char;
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn skiptowhite(p: *const c_char) -> *const c_char;

    // MB_PTR_BACK helper: utf_head_off(base, p) + 1
    fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;

    // rs_cmd_has_expr_args (in ex_docmd Rust crate)
    fn rs_cmd_has_expr_args(cmdidx: c_int) -> bool;
}

// =============================================================================
// Implementation
// =============================================================================

/// Set the completion context for a VimL expression.
///
/// This is a state machine that parses VimL expression syntax and sets the
/// `xp_context` and `xp_pattern` fields of `expand_T` for command-line
/// completion. Called from command-line completion for `:let`, `:const`,
/// `:call`, `:execute`, etc. and for the `=` register.
///
/// Translated from C `set_context_for_expression` in `eval_shim.c`.
///
/// # Safety
/// - `xp` must be a valid `expand_T*`.
/// - `arg` must be a valid NUL-terminated C string.
#[export_name = "set_context_for_expression"]
pub unsafe extern "C" fn rs_set_context_for_expression(
    xp: ExpandHandle,
    arg: *mut c_char,
    cmdidx: c_int,
) {
    // SAFETY: All pointer operations assume valid pointers from C callers.
    unsafe {
        let cmd_let = nvim_docmd_cmd_let();
        let cmd_const = nvim_docmd_cmd_const();
        let cmd_call = nvim_docmd_cmd_call();

        let mut got_eq = false;
        let mut arg = arg;

        if cmdidx == cmd_let || cmdidx == cmd_const {
            nvim_expand_set_context(xp, EXPAND_USER_VARS);
            // Check if arg contains any special chars
            let special = b"\"'+-*/%.=!?~|&$([<>,#\0";
            let special_ptr = special.as_ptr() as *const c_char;
            // strpbrk: find first occurrence of any char from special_ptr in arg
            let found = libc::strpbrk(arg, special_ptr);
            if found.is_null() {
                // ":let var1 var2 ...": find last space.
                let arg_end = arg.add(libc::strlen(arg));
                let mut p = arg_end;
                loop {
                    nvim_expand_set_pattern(xp, p);
                    // MB_PTR_BACK(arg, p): p -= utf_head_off(arg, p-1) + 1
                    let prev = p.sub(1);
                    let off = utf_head_off(arg, prev);
                    p = p.sub((off + 1) as usize);
                    if p < arg {
                        break;
                    }
                    let c = *p as u8;
                    if c == b' ' || c == b'\t' {
                        break;
                    }
                }
                return;
            }
        } else {
            let ctx = if cmdidx == cmd_call {
                EXPAND_FUNCTIONS
            } else {
                EXPAND_EXPRESSION
            };
            nvim_expand_set_context(xp, ctx);
        }

        // Main loop: scan through special characters
        let special = b"\"'+-*/%.=!?~|&$([<>,#\0";
        let special_ptr = special.as_ptr() as *const c_char;
        loop {
            let pattern = libc::strpbrk(arg, special_ptr);
            if pattern.is_null() {
                break;
            }
            nvim_expand_set_pattern(xp, pattern);

            let c = *pattern as u8;
            if c == b'&' {
                let c2 = *pattern.add(1) as u8;
                if c2 == b'&' {
                    nvim_expand_set_pattern(xp, pattern.add(1));
                    let ctx = if cmdidx != cmd_let || got_eq {
                        EXPAND_EXPRESSION
                    } else {
                        EXPAND_NOTHING
                    };
                    nvim_expand_set_context(xp, ctx);
                } else if c2 != b' ' {
                    nvim_expand_set_context(xp, EXPAND_SETTINGS);
                    if (c2 == b'l' || c2 == b'g') && *pattern.add(2) as u8 == b':' {
                        nvim_expand_set_pattern(xp, pattern.add(2));
                    }
                }
            } else if c == b'$' {
                nvim_expand_set_context(xp, EXPAND_ENV_VARS);
            } else if c == b'=' {
                got_eq = true;
                nvim_expand_set_context(xp, EXPAND_EXPRESSION);
            } else if c == b'#' && nvim_expand_get_context(xp) == EXPAND_EXPRESSION {
                // Autoload function/variable contains '#'
                break;
            } else if (c == b'<' || c == b'#')
                && nvim_expand_get_context(xp) == EXPAND_FUNCTIONS
                && nvim_vim_strchr(nvim_cmdexpand_get_xp_pattern(xp), c_int::from(b'(')).is_null()
            {
                // Function name can start with "<SNR>" and contain '#'.
                break;
            } else if cmdidx != cmd_let || got_eq {
                if c == b'"' {
                    // string: skip to closing quote
                    let mut xp_pat = nvim_cmdexpand_get_xp_pattern(xp);
                    xp_pat = xp_pat.add(1);
                    loop {
                        let ch = *xp_pat as u8;
                        if ch == 0 || ch == b'"' {
                            break;
                        }
                        if ch == b'\\' && *xp_pat.add(1) as u8 != 0 {
                            xp_pat = xp_pat.add(1);
                        }
                        xp_pat = xp_pat.add(1);
                    }
                    nvim_expand_set_pattern(xp, xp_pat);
                    nvim_expand_set_context(xp, EXPAND_NOTHING);
                } else if c == b'\'' {
                    // literal string: skip to closing quote
                    let mut xp_pat = nvim_cmdexpand_get_xp_pattern(xp);
                    xp_pat = xp_pat.add(1);
                    loop {
                        let ch = *xp_pat as u8;
                        if ch == 0 || ch == b'\'' {
                            break;
                        }
                        xp_pat = xp_pat.add(1);
                    }
                    nvim_expand_set_pattern(xp, xp_pat);
                    nvim_expand_set_context(xp, EXPAND_NOTHING);
                } else if c == b'|' {
                    if *pattern.add(1) as u8 == b'|' {
                        nvim_expand_set_pattern(xp, pattern.add(1));
                        nvim_expand_set_context(xp, EXPAND_EXPRESSION);
                    } else {
                        nvim_expand_set_context(xp, EXPAND_COMMANDS);
                    }
                } else {
                    nvim_expand_set_context(xp, EXPAND_EXPRESSION);
                }
            } else {
                // Doesn't look like something valid, expand as an expression anyway.
                nvim_expand_set_context(xp, EXPAND_EXPRESSION);
            }

            arg = nvim_cmdexpand_get_xp_pattern(xp);
            if *arg != 0 {
                arg = arg.add(1);
                loop {
                    let ch = *arg as u8;
                    if ch == 0 || !(ch == b' ' || ch == b'\t') {
                        break;
                    }
                    arg = arg.add(1);
                }
            }
        }

        // ":exe one two" completes "two"
        if rs_cmd_has_expr_args(cmdidx) && nvim_expand_get_context(xp) == EXPAND_EXPRESSION {
            loop {
                let n = skiptowhite(arg);
                if n == arg || {
                    let sw = skipwhite(n);
                    *sw as u8 == 0
                } {
                    break;
                }
                arg = skipwhite(n);
            }
        }

        nvim_expand_set_pattern(xp, arg);
    }
}
