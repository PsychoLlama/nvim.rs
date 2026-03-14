//! Command modifier types and utilities for Ex commands.
//!
//! This module defines the types and constants used for command modifiers
//! like `:silent`, `:vertical`, `:noautocmd`, etc.

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

use crate::{CmdModHandle, ExArgHandle};

// =============================================================================
// Command modifier table
// =============================================================================

/// Entry in the command modifier table.
struct CmdMod {
    name: &'static [u8],
    minlen: i32,
    has_count: bool,
}

/// The command modifier table, matching `cmdmods[]` in ex_docmd.c.
/// Values verified against ex_docmd.c lines 3214-3239.
const CMDMODS: &[CmdMod] = &[
    CmdMod {
        name: b"aboveleft",
        minlen: 3,
        has_count: false,
    },
    CmdMod {
        name: b"belowright",
        minlen: 3,
        has_count: false,
    },
    CmdMod {
        name: b"botright",
        minlen: 2,
        has_count: false,
    },
    CmdMod {
        name: b"browse",
        minlen: 3,
        has_count: false,
    },
    CmdMod {
        name: b"confirm",
        minlen: 4,
        has_count: false,
    },
    CmdMod {
        name: b"filter",
        minlen: 4,
        has_count: false,
    },
    CmdMod {
        name: b"hide",
        minlen: 3,
        has_count: false,
    },
    CmdMod {
        name: b"horizontal",
        minlen: 3,
        has_count: false,
    },
    CmdMod {
        name: b"keepalt",
        minlen: 5,
        has_count: false,
    },
    CmdMod {
        name: b"keepjumps",
        minlen: 5,
        has_count: false,
    },
    CmdMod {
        name: b"keepmarks",
        minlen: 3,
        has_count: false,
    },
    CmdMod {
        name: b"keeppatterns",
        minlen: 5,
        has_count: false,
    },
    CmdMod {
        name: b"leftabove",
        minlen: 5,
        has_count: false,
    },
    CmdMod {
        name: b"lockmarks",
        minlen: 3,
        has_count: false,
    },
    CmdMod {
        name: b"noautocmd",
        minlen: 3,
        has_count: false,
    },
    CmdMod {
        name: b"noswapfile",
        minlen: 3,
        has_count: false,
    },
    CmdMod {
        name: b"rightbelow",
        minlen: 6,
        has_count: false,
    },
    CmdMod {
        name: b"sandbox",
        minlen: 3,
        has_count: false,
    },
    CmdMod {
        name: b"silent",
        minlen: 3,
        has_count: false,
    },
    CmdMod {
        name: b"tab",
        minlen: 3,
        has_count: true,
    },
    CmdMod {
        name: b"topleft",
        minlen: 2,
        has_count: false,
    },
    CmdMod {
        name: b"unsilent",
        minlen: 3,
        has_count: false,
    },
    CmdMod {
        name: b"verbose",
        minlen: 4,
        has_count: true,
    },
    CmdMod {
        name: b"vertical",
        minlen: 4,
        has_count: false,
    },
];

// =============================================================================
// Command modifier flags (CMOD_*)
// =============================================================================

/// `:sandbox` - execute in sandbox mode
pub const CMOD_SANDBOX: c_int = 0x0001;
/// `:silent` - suppress messages
pub const CMOD_SILENT: c_int = 0x0002;
/// `:silent!` - suppress error messages too
pub const CMOD_ERRSILENT: c_int = 0x0004;
/// `:unsilent` - cancel silence
pub const CMOD_UNSILENT: c_int = 0x0008;
/// `:noautocmd` - disable autocommands
pub const CMOD_NOAUTOCMD: c_int = 0x0010;
/// `:hide` - hide buffer when leaving
pub const CMOD_HIDE: c_int = 0x0020;
/// `:browse` - invoke file dialog
pub const CMOD_BROWSE: c_int = 0x0040;
/// `:confirm` - invoke yes/no dialog
pub const CMOD_CONFIRM: c_int = 0x0080;
/// `:keepalt` - keep alternate file
pub const CMOD_KEEPALT: c_int = 0x0100;
/// `:keepmarks` - keep marks
pub const CMOD_KEEPMARKS: c_int = 0x0200;
/// `:keepjumps` - keep jump list
pub const CMOD_KEEPJUMPS: c_int = 0x0400;
/// `:lockmarks` - lock marks
pub const CMOD_LOCKMARKS: c_int = 0x0800;
/// `:keeppatterns` - keep search patterns
pub const CMOD_KEEPPATTERNS: c_int = 0x1000;
/// `:noswapfile` - don't create swap file
pub const CMOD_NOSWAPFILE: c_int = 0x2000;

// =============================================================================
// Window split flags (WSP_*)
// =============================================================================

/// Split vertically
pub const WSP_VERT: c_int = 0x02;
/// Split horizontally
pub const WSP_HOR: c_int = 0x04;
/// Split at top
pub const WSP_TOP: c_int = 0x08;
/// Split at bottom
pub const WSP_BOT: c_int = 0x10;
/// Split below current window
pub const WSP_BELOW: c_int = 0x40;
/// Split above current window
pub const WSP_ABOVE: c_int = 0x80;

// =============================================================================
// Flag checking utilities
// =============================================================================

/// Check if the CMOD_SANDBOX flag is set.
#[inline]
pub const fn has_sandbox(flags: c_int) -> bool {
    (flags & CMOD_SANDBOX) != 0
}

/// Check if the CMOD_SILENT flag is set.
#[inline]
pub const fn has_silent(flags: c_int) -> bool {
    (flags & CMOD_SILENT) != 0
}

/// Check if the CMOD_ERRSILENT flag is set.
#[inline]
pub const fn has_errsilent(flags: c_int) -> bool {
    (flags & CMOD_ERRSILENT) != 0
}

/// Check if the CMOD_NOAUTOCMD flag is set.
#[inline]
pub const fn has_noautocmd(flags: c_int) -> bool {
    (flags & CMOD_NOAUTOCMD) != 0
}

/// FFI wrapper to check if CMOD_SANDBOX flag is set.
#[no_mangle]
pub extern "C" fn rs_cmod_has_sandbox(flags: c_int) -> c_int {
    c_int::from(has_sandbox(flags))
}

/// FFI wrapper to check if CMOD_SILENT flag is set.
#[no_mangle]
pub extern "C" fn rs_cmod_has_silent(flags: c_int) -> c_int {
    c_int::from(has_silent(flags))
}

/// FFI wrapper to check if CMOD_ERRSILENT flag is set.
#[no_mangle]
pub extern "C" fn rs_cmod_has_errsilent(flags: c_int) -> c_int {
    c_int::from(has_errsilent(flags))
}

/// FFI wrapper to check if CMOD_NOAUTOCMD flag is set.
#[no_mangle]
pub extern "C" fn rs_cmod_has_noautocmd(flags: c_int) -> c_int {
    c_int::from(has_noautocmd(flags))
}

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    fn skipwhite(p: *const c_char) -> *mut c_char;
    #[link_name = "skipdigits"]
    fn rs_skipdigits(q: *const c_char) -> *const c_char;
    #[link_name = "ends_excmd"]
    fn rs_ends_excmd(c: c_int) -> c_int;
    #[link_name = "checkforcmd"]
    fn rs_checkforcmd(pp: *mut *mut c_char, cmd: *const c_char, len: c_int) -> bool;

    // eap field accessors
    fn nvim_eap_get_cmd(eap: ExArgHandle) -> *mut c_char;
    fn nvim_eap_set_cmd(eap: ExArgHandle, p: *mut c_char);
    fn nvim_eap_set_nextcmd(eap: ExArgHandle, p: *mut c_char);
    fn nvim_eap_get_skip(eap: ExArgHandle) -> c_int;

    // cmdmod_T field accessors
    fn nvim_cmod_clear(cmod: CmdModHandle);
    fn nvim_cmod_or_flags(cmod: CmdModHandle, f: c_int);
    fn nvim_cmod_or_split(cmod: CmdModHandle, f: c_int);
    fn nvim_cmod_set_tab(cmod: CmdModHandle, v: c_int);
    fn nvim_cmod_set_verbose(cmod: CmdModHandle, v: c_int);
    fn nvim_cmod_set_filter_force(cmod: CmdModHandle, v: c_int);
    fn nvim_cmod_set_filter_pat(cmod: CmdModHandle, s: *mut c_char);
    fn nvim_cmod_set_filter_regprog(cmod: CmdModHandle, prog: *mut c_void);

    // Global state accessors
    fn nvim_docmd_get_exmode_active() -> c_int;
    fn nvim_docmd_getline_is_getexline(eap: ExArgHandle) -> c_int;
    fn nvim_docmd_get_exmode_plus() -> *mut c_char;
    fn nvim_docmd_set_ex_pressedreturn(val: c_int);
    fn nvim_docmd_get_curwin_cursor_lnum() -> i32;
    fn nvim_docmd_get_curbuf_line_count() -> i32;
    fn nvim_docmd_vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;
    fn nvim_docmd_vim_regcomp(pat: *const c_char, flags: c_int) -> *mut c_void;
    fn nvim_docmd_xstrdup(s: *const c_char) -> *mut c_char;
    fn nvim_docmd_skip_vimgrep_pat(
        p: *mut c_char,
        s: *mut *mut c_char,
        flags: *mut c_int,
    ) -> *mut c_char;
    fn nvim_get_curtab() -> *mut std::ffi::c_void;
    #[link_name = "rs_tabpage_index"]
    fn nvim_rs_tabpage_index(tp: *mut std::ffi::c_void) -> c_int;
    fn nvim_docmd_LAST_TAB_NR() -> c_int;
    fn nvim_docmd_atoi(s: *const c_char) -> c_int;
    fn nvim_docmd_skip_range(cmd: *const c_char) -> *mut c_char;
    fn nvim_docmd_skipwhite(p: *const c_char) -> *mut c_char;
    fn nvim_docmd_get_e_invrange_msg() -> *mut c_char;
    fn nvim_docmd_ascii_iswhite(c: c_int) -> c_int;
    fn nvim_docmd_ascii_isdigit(c: c_int) -> c_int;

}

// Constants
const FAIL: c_int = 0;
const OK: c_int = 1;
const MAXLNUM: i32 = 0x7fffffff;
const RE_MAGIC: c_int = 1;

// =============================================================================
// parse_command_modifiers
// =============================================================================

/// Parse command modifiers.
///
/// Replaces C `parse_command_modifiers()`.
///
/// Scans the command string for modifiers like `:silent`, `:verbose`, `:tab`,
/// `:vertical`, etc. Sets the corresponding fields in `cmod` via C accessors.
///
/// Returns OK (1) on success, FAIL (0) when the command should not be executed
/// (comment, empty line, etc.).
#[export_name = "parse_command_modifiers"]
pub unsafe extern "C" fn rs_parse_command_modifiers(
    eap: ExArgHandle,
    errormsg: *mut *const c_char,
    cmod: CmdModHandle,
    skip_only: c_int,
) -> c_int {
    let skip_only = skip_only != 0;

    nvim_cmod_clear(cmod);

    // Repeat until no more command modifiers are found.
    loop {
        // Skip whitespace and colons
        let mut cmd = nvim_eap_get_cmd(eap);
        while *cmd as u8 == b' ' || *cmd as u8 == b'\t' || *cmd as u8 == b':' {
            cmd = cmd.add(1);
        }
        nvim_eap_set_cmd(eap, cmd);

        // In ex mode, an empty line works like :+
        cmd = nvim_eap_get_cmd(eap);
        if *cmd == 0
            && nvim_docmd_get_exmode_active() != 0
            && nvim_docmd_getline_is_getexline(eap) != 0
            && nvim_docmd_get_curwin_cursor_lnum() < nvim_docmd_get_curbuf_line_count()
        {
            nvim_eap_set_cmd(eap, nvim_docmd_get_exmode_plus());
            if !skip_only {
                nvim_docmd_set_ex_pressedreturn(1);
            }
        }

        // Ignore comment and empty lines
        cmd = nvim_eap_get_cmd(eap);
        if *cmd as u8 == b'"' {
            // A comment ends at a NL
            let nl = nvim_docmd_vim_strchr(cmd, b'\n' as c_int);
            if !nl.is_null() {
                nvim_eap_set_nextcmd(eap, nl.add(1));
            } else {
                nvim_eap_set_nextcmd(eap, ptr::null_mut());
            }
            return FAIL;
        }
        if *cmd as u8 == b'\n' {
            nvim_eap_set_nextcmd(eap, cmd.add(1));
            return FAIL;
        }
        if *cmd == 0 {
            if !skip_only {
                nvim_docmd_set_ex_pressedreturn(1);
            }
            return FAIL;
        }

        let mut p = nvim_docmd_skip_range(cmd);
        let switch_char = *p as u8;

        let mut matched = false;

        match switch_char {
            b'a' => {
                let mut cmd_ptr = nvim_eap_get_cmd(eap);
                if rs_checkforcmd(&mut cmd_ptr, c"aboveleft".as_ptr(), 3) {
                    nvim_eap_set_cmd(eap, cmd_ptr);
                    nvim_cmod_or_split(cmod, WSP_ABOVE);
                    matched = true;
                }
            }

            b'b' => {
                let mut cmd_ptr = nvim_eap_get_cmd(eap);
                if rs_checkforcmd(&mut cmd_ptr, c"belowright".as_ptr(), 3) {
                    nvim_eap_set_cmd(eap, cmd_ptr);
                    nvim_cmod_or_split(cmod, WSP_BELOW);
                    matched = true;
                } else {
                    cmd_ptr = nvim_eap_get_cmd(eap);
                    if rs_checkforcmd(&mut cmd_ptr, c"browse".as_ptr(), 3) {
                        nvim_eap_set_cmd(eap, cmd_ptr);
                        nvim_cmod_or_flags(cmod, CMOD_BROWSE);
                        matched = true;
                    } else {
                        cmd_ptr = nvim_eap_get_cmd(eap);
                        if rs_checkforcmd(&mut cmd_ptr, c"botright".as_ptr(), 2) {
                            nvim_eap_set_cmd(eap, cmd_ptr);
                            nvim_cmod_or_split(cmod, WSP_BOT);
                            matched = true;
                        }
                    }
                }
            }

            b'c' => {
                let mut cmd_ptr = nvim_eap_get_cmd(eap);
                if rs_checkforcmd(&mut cmd_ptr, c"confirm".as_ptr(), 4) {
                    nvim_eap_set_cmd(eap, cmd_ptr);
                    nvim_cmod_or_flags(cmod, CMOD_CONFIRM);
                    matched = true;
                }
            }

            b'k' => {
                let mut cmd_ptr = nvim_eap_get_cmd(eap);
                if rs_checkforcmd(&mut cmd_ptr, c"keepmarks".as_ptr(), 3) {
                    nvim_eap_set_cmd(eap, cmd_ptr);
                    nvim_cmod_or_flags(cmod, CMOD_KEEPMARKS);
                    matched = true;
                } else {
                    cmd_ptr = nvim_eap_get_cmd(eap);
                    if rs_checkforcmd(&mut cmd_ptr, c"keepalt".as_ptr(), 5) {
                        nvim_eap_set_cmd(eap, cmd_ptr);
                        nvim_cmod_or_flags(cmod, CMOD_KEEPALT);
                        matched = true;
                    } else {
                        cmd_ptr = nvim_eap_get_cmd(eap);
                        if rs_checkforcmd(&mut cmd_ptr, c"keeppatterns".as_ptr(), 5) {
                            nvim_eap_set_cmd(eap, cmd_ptr);
                            nvim_cmod_or_flags(cmod, CMOD_KEEPPATTERNS);
                            matched = true;
                        } else {
                            cmd_ptr = nvim_eap_get_cmd(eap);
                            if rs_checkforcmd(&mut cmd_ptr, c"keepjumps".as_ptr(), 5) {
                                nvim_eap_set_cmd(eap, cmd_ptr);
                                nvim_cmod_or_flags(cmod, CMOD_KEEPJUMPS);
                                matched = true;
                            }
                        }
                    }
                }
            }

            b'f' => {
                // Only accept ":filter {pat} cmd"
                if rs_checkforcmd(&mut p, c"filter".as_ptr(), 4)
                    && *p != 0
                    && rs_ends_excmd(*p as c_int) == 0
                {
                    if *p as u8 == b'!' {
                        nvim_cmod_set_filter_force(cmod, 1);
                        p = nvim_docmd_skipwhite(p.add(1));
                        if *p == 0 || rs_ends_excmd(*p as c_int) != 0 {
                            // break out — not matched
                        } else if skip_only {
                            p = nvim_docmd_skip_vimgrep_pat(p, ptr::null_mut(), ptr::null_mut());
                            if !p.is_null() && *p != 0 {
                                nvim_eap_set_cmd(eap, p);
                                matched = true;
                            }
                        } else {
                            let mut reg_pat: *mut c_char = ptr::null_mut();
                            p = nvim_docmd_skip_vimgrep_pat(p, &mut reg_pat, ptr::null_mut());
                            if !p.is_null() && *p != 0 {
                                nvim_cmod_set_filter_pat(cmod, nvim_docmd_xstrdup(reg_pat));
                                let regprog = nvim_docmd_vim_regcomp(reg_pat, RE_MAGIC);
                                if !regprog.is_null() {
                                    nvim_cmod_set_filter_regprog(cmod, regprog);
                                    nvim_eap_set_cmd(eap, p);
                                    matched = true;
                                }
                            }
                        }
                    } else if skip_only {
                        p = nvim_docmd_skip_vimgrep_pat(p, ptr::null_mut(), ptr::null_mut());
                        if !p.is_null() && *p != 0 {
                            nvim_eap_set_cmd(eap, p);
                            matched = true;
                        }
                    } else {
                        let mut reg_pat: *mut c_char = ptr::null_mut();
                        p = nvim_docmd_skip_vimgrep_pat(p, &mut reg_pat, ptr::null_mut());
                        if !p.is_null() && *p != 0 {
                            nvim_cmod_set_filter_pat(cmod, nvim_docmd_xstrdup(reg_pat));
                            let regprog = nvim_docmd_vim_regcomp(reg_pat, RE_MAGIC);
                            if !regprog.is_null() {
                                nvim_cmod_set_filter_regprog(cmod, regprog);
                                nvim_eap_set_cmd(eap, p);
                                matched = true;
                            }
                        }
                    }
                }
            }

            b'h' => {
                let mut cmd_ptr = nvim_eap_get_cmd(eap);
                if rs_checkforcmd(&mut cmd_ptr, c"horizontal".as_ptr(), 3) {
                    nvim_eap_set_cmd(eap, cmd_ptr);
                    nvim_cmod_or_split(cmod, WSP_HOR);
                    matched = true;
                } else {
                    // ":hide" and ":hide | cmd" are not modifiers
                    cmd = nvim_eap_get_cmd(eap);
                    if p == cmd
                        && rs_checkforcmd(&mut p, c"hide".as_ptr(), 3)
                        && *p != 0
                        && rs_ends_excmd(*p as c_int) == 0
                    {
                        nvim_eap_set_cmd(eap, p);
                        nvim_cmod_or_flags(cmod, CMOD_HIDE);
                        matched = true;
                    }
                }
            }

            b'l' => {
                let mut cmd_ptr = nvim_eap_get_cmd(eap);
                if rs_checkforcmd(&mut cmd_ptr, c"lockmarks".as_ptr(), 3) {
                    nvim_eap_set_cmd(eap, cmd_ptr);
                    nvim_cmod_or_flags(cmod, CMOD_LOCKMARKS);
                    matched = true;
                } else {
                    cmd_ptr = nvim_eap_get_cmd(eap);
                    if rs_checkforcmd(&mut cmd_ptr, c"leftabove".as_ptr(), 5) {
                        nvim_eap_set_cmd(eap, cmd_ptr);
                        nvim_cmod_or_split(cmod, WSP_ABOVE);
                        matched = true;
                    }
                }
            }

            b'n' => {
                let mut cmd_ptr = nvim_eap_get_cmd(eap);
                if rs_checkforcmd(&mut cmd_ptr, c"noautocmd".as_ptr(), 3) {
                    nvim_eap_set_cmd(eap, cmd_ptr);
                    nvim_cmod_or_flags(cmod, CMOD_NOAUTOCMD);
                    matched = true;
                } else {
                    cmd_ptr = nvim_eap_get_cmd(eap);
                    if rs_checkforcmd(&mut cmd_ptr, c"noswapfile".as_ptr(), 3) {
                        nvim_eap_set_cmd(eap, cmd_ptr);
                        nvim_cmod_or_flags(cmod, CMOD_NOSWAPFILE);
                        matched = true;
                    }
                }
            }

            b'r' => {
                let mut cmd_ptr = nvim_eap_get_cmd(eap);
                if rs_checkforcmd(&mut cmd_ptr, c"rightbelow".as_ptr(), 6) {
                    nvim_eap_set_cmd(eap, cmd_ptr);
                    nvim_cmod_or_split(cmod, WSP_BELOW);
                    matched = true;
                }
            }

            b's' => {
                let mut cmd_ptr = nvim_eap_get_cmd(eap);
                if rs_checkforcmd(&mut cmd_ptr, c"sandbox".as_ptr(), 3) {
                    nvim_eap_set_cmd(eap, cmd_ptr);
                    nvim_cmod_or_flags(cmod, CMOD_SANDBOX);
                    matched = true;
                } else {
                    cmd_ptr = nvim_eap_get_cmd(eap);
                    if rs_checkforcmd(&mut cmd_ptr, c"silent".as_ptr(), 3) {
                        nvim_eap_set_cmd(eap, cmd_ptr);
                        nvim_cmod_or_flags(cmod, CMOD_SILENT);
                        cmd_ptr = nvim_eap_get_cmd(eap);
                        if *cmd_ptr as u8 == b'!'
                            && nvim_docmd_ascii_iswhite(*cmd_ptr.sub(1) as c_int) == 0
                        {
                            // ":silent!", but not "silent !cmd"
                            let new_cmd = nvim_docmd_skipwhite(cmd_ptr.add(1));
                            nvim_eap_set_cmd(eap, new_cmd);
                            nvim_cmod_or_flags(cmod, CMOD_ERRSILENT);
                        }
                        matched = true;
                    }
                }
            }

            b't' => {
                if rs_checkforcmd(&mut p, c"tab".as_ptr(), 3) {
                    if !skip_only {
                        let eap_skip = nvim_eap_get_skip(eap) != 0;
                        let mut cmd_ptr = nvim_eap_get_cmd(eap);
                        let tabnr = crate::address::get_address_impl(
                            eap,
                            &mut cmd_ptr,
                            ADDR_TABS,
                            eap_skip,
                            skip_only,
                            0,
                            1,
                            errormsg,
                        );
                        nvim_eap_set_cmd(eap, cmd_ptr);

                        if nvim_eap_get_cmd(eap).is_null() {
                            return 0; // false
                        }

                        if tabnr == MAXLNUM {
                            nvim_cmod_set_tab(cmod, nvim_rs_tabpage_index(nvim_get_curtab()) + 1);
                        } else {
                            if tabnr < 0 || tabnr > nvim_docmd_LAST_TAB_NR() as i32 {
                                *errormsg = nvim_docmd_get_e_invrange_msg();
                                return 0; // false
                            }
                            nvim_cmod_set_tab(cmod, tabnr as c_int + 1);
                        }
                    }
                    nvim_eap_set_cmd(eap, p);
                    matched = true;
                } else {
                    let mut cmd_ptr = nvim_eap_get_cmd(eap);
                    if rs_checkforcmd(&mut cmd_ptr, c"topleft".as_ptr(), 2) {
                        nvim_eap_set_cmd(eap, cmd_ptr);
                        nvim_cmod_or_split(cmod, WSP_TOP);
                        matched = true;
                    }
                }
            }

            b'u' => {
                let mut cmd_ptr = nvim_eap_get_cmd(eap);
                if rs_checkforcmd(&mut cmd_ptr, c"unsilent".as_ptr(), 3) {
                    nvim_eap_set_cmd(eap, cmd_ptr);
                    nvim_cmod_or_flags(cmod, CMOD_UNSILENT);
                    matched = true;
                }
            }

            b'v' => {
                let mut cmd_ptr = nvim_eap_get_cmd(eap);
                if rs_checkforcmd(&mut cmd_ptr, c"vertical".as_ptr(), 4) {
                    nvim_eap_set_cmd(eap, cmd_ptr);
                    nvim_cmod_or_split(cmod, WSP_VERT);
                    matched = true;
                } else if rs_checkforcmd(&mut p, c"verbose".as_ptr(), 4) {
                    cmd = nvim_eap_get_cmd(eap);
                    if nvim_docmd_ascii_isdigit(*cmd as c_int) != 0 {
                        // zero means not set, one is verbose == 0, etc.
                        nvim_cmod_set_verbose(cmod, nvim_docmd_atoi(cmd) + 1);
                    } else {
                        nvim_cmod_set_verbose(cmod, 2); // default: verbose == 1
                    }
                    nvim_eap_set_cmd(eap, p);
                    matched = true;
                }
            }

            _ => {}
        }

        if !matched {
            break;
        }
    }

    OK
}

// =============================================================================
// Address type constant for :tab modifier
// =============================================================================

const ADDR_TABS: c_int = 5;

// =============================================================================
// Modifier length
// =============================================================================

/// Get the length of a command modifier (including optional count prefix).
///
/// Returns 0 if the string does not start with a modifier.
///
/// Matches the C `modifier_len()` function.
///
/// # Safety
///
/// `cmd` must be a valid null-terminated C string.
#[export_name = "modifier_len"]
pub unsafe extern "C" fn rs_modifier_len(cmd: *const c_char) -> c_int {
    if cmd.is_null() {
        return 0;
    }

    let mut p = cmd;

    // Skip optional count prefix (digits then whitespace)
    if (*p as u8).is_ascii_digit() {
        p = skipwhite(rs_skipdigits(cmd.add(1)) as *const c_char) as *const c_char;
    }

    for entry in CMDMODS {
        let mut j = 0i32;
        loop {
            let c = *p.add(j as usize) as u8;
            if c == 0 {
                break;
            }
            if j as usize >= entry.name.len() || c != entry.name[j as usize] {
                break;
            }
            j += 1;
        }
        if j >= entry.minlen
            && !(*p.add(j as usize) as u8).is_ascii_alphabetic()
            && (p == cmd || entry.has_count)
        {
            return j + (p as usize - cmd as usize) as c_int;
        }
    }
    0
}

/// Check if a name matches a command modifier.
///
/// Returns:
/// - 0 if name doesn't match any modifier
/// - 1 if name is a prefix match (abbreviation) of a modifier
/// - 2 if name is an exact match of a modifier
///
/// Used by `cmd_exists()`.
pub unsafe fn check_modifier(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }

    for entry in CMDMODS {
        let mut j = 0usize;
        loop {
            let c = *name.add(j) as u8;
            if c == 0 {
                break;
            }
            if j >= entry.name.len() || c != entry.name[j] {
                break;
            }
            j += 1;
        }
        if *name.add(j) as u8 == 0 && j >= entry.minlen as usize {
            return if j == entry.name.len() { 2 } else { 1 };
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmod_flag_checks() {
        // Test individual flags
        assert!(has_sandbox(CMOD_SANDBOX));
        assert!(!has_sandbox(CMOD_SILENT));

        assert!(has_silent(CMOD_SILENT));
        assert!(!has_silent(CMOD_SANDBOX));

        assert!(has_errsilent(CMOD_ERRSILENT));
        assert!(!has_errsilent(CMOD_SILENT));

        assert!(has_noautocmd(CMOD_NOAUTOCMD));
        assert!(!has_noautocmd(CMOD_SILENT));

        // Test combined flags
        let combined = CMOD_SANDBOX | CMOD_SILENT | CMOD_NOAUTOCMD;
        assert!(has_sandbox(combined));
        assert!(has_silent(combined));
        assert!(!has_errsilent(combined));
        assert!(has_noautocmd(combined));
    }

    #[test]
    fn test_cmod_ffi_wrappers() {
        assert_eq!(rs_cmod_has_sandbox(CMOD_SANDBOX), 1);
        assert_eq!(rs_cmod_has_sandbox(CMOD_SILENT), 0);

        assert_eq!(rs_cmod_has_silent(CMOD_SILENT), 1);
        assert_eq!(rs_cmod_has_silent(CMOD_SANDBOX), 0);
    }

    #[test]
    fn test_wsp_flags() {
        // Verify flag values match C header (window.h)
        assert_eq!(WSP_VERT, 0x02);
        assert_eq!(WSP_HOR, 0x04);
        assert_eq!(WSP_TOP, 0x08);
        assert_eq!(WSP_BOT, 0x10);
        assert_eq!(WSP_BELOW, 0x40);
        assert_eq!(WSP_ABOVE, 0x80);
    }

    #[test]
    fn test_cmdmods_table() {
        // Verify the table has 24 entries matching C
        assert_eq!(CMDMODS.len(), 24);

        // Verify first and last entries
        assert_eq!(CMDMODS[0].name, b"aboveleft");
        assert_eq!(CMDMODS[0].minlen, 3);
        assert!(!CMDMODS[0].has_count);

        assert_eq!(CMDMODS[23].name, b"vertical");
        assert_eq!(CMDMODS[23].minlen, 4);
        assert!(!CMDMODS[23].has_count);

        // Verify the two has_count entries
        let tab_entry = CMDMODS.iter().find(|m| m.name == b"tab").unwrap();
        assert!(tab_entry.has_count);
        assert_eq!(tab_entry.minlen, 3);

        let verbose_entry = CMDMODS.iter().find(|m| m.name == b"verbose").unwrap();
        assert!(verbose_entry.has_count);
        assert_eq!(verbose_entry.minlen, 4);

        // Verify sorted order (matches C)
        for i in 1..CMDMODS.len() {
            assert!(
                CMDMODS[i - 1].name < CMDMODS[i].name,
                "cmdmods table not sorted at index {}",
                i
            );
        }
    }

    #[test]
    fn test_check_modifier() {
        use std::ffi::CString;
        unsafe {
            // Exact match
            let name = CString::new("silent").unwrap();
            assert_eq!(check_modifier(name.as_ptr()), 2);

            // Abbreviation match
            let name = CString::new("sil").unwrap();
            assert_eq!(check_modifier(name.as_ptr()), 1);

            // Too short to match (minlen=3)
            let name = CString::new("si").unwrap();
            assert_eq!(check_modifier(name.as_ptr()), 0);

            // Not a modifier
            let name = CString::new("foobar").unwrap();
            assert_eq!(check_modifier(name.as_ptr()), 0);

            // "tab" exact match
            let name = CString::new("tab").unwrap();
            assert_eq!(check_modifier(name.as_ptr()), 2);

            // "verb" abbreviation for verbose
            let name = CString::new("verb").unwrap();
            assert_eq!(check_modifier(name.as_ptr()), 1);

            // "vertical" exact match
            let name = CString::new("vertical").unwrap();
            assert_eq!(check_modifier(name.as_ptr()), 2);

            // "vert" - this matches "vertical" (minlen=4)
            let name = CString::new("vert").unwrap();
            assert_eq!(check_modifier(name.as_ptr()), 1);
        }
    }
}
