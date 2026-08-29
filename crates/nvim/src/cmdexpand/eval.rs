//! The Vimscript face: `getcompletion()`, `getcompletiontype()`,
//! `cmdcomplete_info()`.
//!
//! [`f_getcompletion`] runs the whole classify-then-expand pipeline against a
//! string instead of the real command line, which is what makes it the
//! completion layer's differential oracle.  All three are rows in the
//! generated eval function table.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cmdexpand::{WildMode, WildOpts};
use crate::eval::typval::NumBuf;
use crate::message_fmt::c_str;
use crate::semsg;
use crate::types::{ExpandContext, FAIL, OK, VAR_STRING, VAR_UNKNOWN};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

/// `getcompletion()`: expand `{pattern}` as `{type}` and answer the matches.
/// What `getcompletion()` asks of every expansion: newline-separated so the
/// caller can split it, quiet, and with `~/` restored.
const GETCOMPLETION: WildOpts = WildOpts::SILENT
    .or(WildOpts::USE_NL)
    .or(WildOpts::ADD_SLASH)
    .or(WildOpts::NO_BEEP)
    .or(WildOpts::HOME_REPLACE);

/// `expand_one`'s `orig` argument, which this caller never has.
const NO_ORIG: *mut c_char = ptr::null_mut();

pub unsafe fn f_getcompletion(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut xpc: expand_T = unsafe { core::mem::zeroed() };
    let mut filtered = false;
    let mut options = GETCOMPLETION;

    if unsafe { tv_check_for_string_arg(argvars, 1) } == FAIL {
        return;
    }
    let type_0 = unsafe { numbuf.string(argvars.add(1)) };

    if unsafe { (*argvars.add(2)).v_type } != VAR_UNKNOWN {
        filtered = unsafe { tv_get_number_chk(argvars.add(2), ptr::null_mut()) } != 0;
    }

    if p_wic.get() != 0 {
        options |= WildOpts::ICASE;
    }

    // For filtered results, 'wildignore' is used.
    if !filtered {
        options |= WildOpts::KEEP_ALL;
    }

    if unsafe { (*argvars).v_type } != VAR_STRING {
        emsg(gettext(e_invarg));
        return;
    }
    let pattern = unsafe { numbuf2.string(argvars) };
    let mut pattern_start = pattern;

    // C's `goto theend`: the "cmdline" type takes the whole classifier and
    // skips the per-type switch entirely.
    if unsafe { strcmp(type_0, c"cmdline".as_ptr()) } == 0 {
        let cmdline_len = unsafe { strlen(pattern) } as c_int;
        unsafe {
            set_cmd_context(
                &raw mut xpc,
                pattern as *mut c_char,
                cmdline_len,
                cmdline_len,
                false,
            )
        };
        pattern_start = xpc.xp_pattern;
        xpc.xp_pattern_len = unsafe { strlen(xpc.xp_pattern) };
        xpc.xp_col = cmdline_len;
    } else {
        unsafe { expand_init(&raw mut xpc) };
        xpc.xp_pattern = pattern as *mut c_char;
        xpc.xp_pattern_len = unsafe { strlen(xpc.xp_pattern) };
        xpc.xp_line = pattern as *mut c_char;

        xpc.xp_context = unsafe { cmdcomplete_str_to_type(type_0) };
        match xpc.xp_context {
            ExpandContext::Nothing => {
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let arg0 = unsafe { c_str(type_0) };
                semsg!("E475: Invalid argument: {arg0}");
                return;
            }
            ExpandContext::UserDefined => {
                // Must be "custom,funcname" pattern.
                if unsafe { strncmp(type_0, c"custom,".as_ptr(), 7) } != 0 {
                    // SAFETY: a message argument the caller holds as a NUL-terminated string.
                    let arg0 = unsafe { c_str(type_0) };
                    semsg!("E475: Invalid argument: {arg0}");
                    return;
                }
                xpc.xp_arg = unsafe { type_0.add(7) } as *mut c_char;
            }
            ExpandContext::UserList => {
                // Must be "customlist,funcname" pattern.
                if unsafe { strncmp(type_0, c"customlist,".as_ptr(), 11) } != 0 {
                    // SAFETY: a message argument the caller holds as a NUL-terminated string.
                    let arg0 = unsafe { c_str(type_0) };
                    semsg!("E475: Invalid argument: {arg0}");
                    return;
                }
                xpc.xp_arg = unsafe { type_0.add(11) } as *mut c_char;
            }
            // The four generators below move `xp_pattern` forward inside
            // the string, so the length has to follow it.
            ExpandContext::Menus => {
                unsafe {
                    set_context_in_menu_cmd(&raw mut xpc, c"menu".as_ptr(), xpc.xp_pattern, false)
                };
                xpc.xp_pattern_len -=
                    unsafe { xpc.xp_pattern.offset_from(pattern_start) } as size_t;
            }
            ExpandContext::Sign => {
                unsafe { set_context_in_sign_cmd(&raw mut xpc, xpc.xp_pattern) };
                xpc.xp_pattern_len -=
                    unsafe { xpc.xp_pattern.offset_from(pattern_start) } as size_t;
            }
            ExpandContext::Runtime => {
                unsafe { set_context_in_runtime_cmd(&raw mut xpc, xpc.xp_pattern) };
                xpc.xp_pattern_len -=
                    unsafe { xpc.xp_pattern.offset_from(pattern_start) } as size_t;
            }
            ExpandContext::ShellCmdLine => {
                let mut context = ExpandContext::ShellCmdLine;
                unsafe {
                    set_context_for_wildcard_arg(
                        ptr::null_mut(),
                        xpc.xp_pattern,
                        false,
                        &raw mut xpc,
                        &raw mut context,
                    )
                };
                xpc.xp_pattern_len -=
                    unsafe { xpc.xp_pattern.offset_from(pattern_start) } as size_t;
            }
            ExpandContext::FiletypeCmd => filetype_expand_what.set(FiletypeWhat::All),
            _ => {}
        }
    }

    if xpc.xp_context == ExpandContext::Lua {
        xpc.xp_col = unsafe { strlen(xpc.xp_line) } as c_int;
        unsafe { nlua_expand_pat(&raw mut xpc) };
        xpc.xp_pattern_len -= unsafe { xpc.xp_pattern.offset_from(pattern_start) } as size_t;
    }

    let pat = if unsafe { cmdline_fuzzy_completion_supported(&raw mut xpc) } {
        // When fuzzy matching, don't modify the search string.
        unsafe { xmemdupz(xpc.xp_pattern as *const c_void, xpc.xp_pattern_len) as *mut c_char }
    } else {
        unsafe { addstar(xpc.xp_pattern, xpc.xp_pattern_len, xpc.xp_context) }
    };

    unsafe { expand_one(&raw mut xpc, pat, NO_ORIG, options, WildMode::AllKeep) };
    unsafe { tv_list_alloc_ret(rettv, xpc.xp_numfiles as ptrdiff_t) };

    for i in 0..xpc.xp_numfiles {
        unsafe {
            tv_list_append_string((*rettv).vval.v_list, *xpc.xp_files.offset(i as isize), -1)
        };
    }
    unsafe { xfree(pat as *mut c_void) };
    unsafe { expand_cleanup(&raw mut xpc) };
}

/// `getcompletiontype()`: the completion type name a command line would use.
pub unsafe fn f_getcompletiontype(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let mut numbuf = NumBuf::new();
    unsafe { (*rettv).v_type = VAR_STRING };
    unsafe { (*rettv).vval.v_string = ptr::null_mut() };

    if unsafe { tv_check_for_string_arg(argvars, 0) } == FAIL {
        return;
    }

    let pat = unsafe { numbuf.string(argvars) };
    let mut xpc: expand_T = unsafe { core::mem::zeroed() };
    unsafe { expand_init(&raw mut xpc) };

    let cmdline_len = unsafe { strlen(pat) } as c_int;
    unsafe {
        set_cmd_context(
            &raw mut xpc,
            pat as *mut c_char,
            cmdline_len,
            cmdline_len,
            false,
        )
    };
    unsafe { (*rettv).vval.v_string = cmdcomplete_type_to_str(xpc.xp_context, xpc.xp_arg) };

    unsafe { expand_cleanup(&raw mut xpc) };
}

/// `cmdcomplete_info()`: the state of the completion in progress.
pub unsafe fn f_cmdcomplete_info(
    _argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let xpc = Cc::current().xpc();

    unsafe { tv_dict_alloc_ret(rettv) };
    if xpc.is_null() || unsafe { (*xpc).xp_files }.is_null() {
        return;
    }
    let retdict: *mut dict_T = unsafe { (*rettv).vval.v_dict };

    // C's S_LEN(): `tv_dict_add_*` copies exactly `key_len` bytes, so the
    // key type is a plain `&str`.
    let add_str = |k: &str, v| unsafe { tv_dict_add_str(retdict, k.as_ptr().cast(), k.len(), v) };
    let add_nr = |k: &str, v| unsafe { tv_dict_add_nr(retdict, k.as_ptr().cast(), k.len(), v) };
    let add_list = |k: &str, v| unsafe { tv_dict_add_list(retdict, k.as_ptr().cast(), k.len(), v) };

    let mut ret = add_str("cmdline_orig", cmdline_orig.get());
    if ret == OK {
        ret = add_nr("pum_visible", pum_visible() as varnumber_T);
    }
    if ret == OK {
        ret = add_nr("selected", unsafe { (*xpc).xp_selected } as varnumber_T);
    }
    if ret == OK {
        let li = unsafe { tv_list_alloc((*xpc).xp_numfiles as ptrdiff_t) };
        ret = add_list("matches", li);
        let mut idx = 0;
        while ret == OK && idx < unsafe { (*xpc).xp_numfiles } {
            unsafe { tv_list_append_string(li, *(*xpc).xp_files.offset(idx as isize), -1) };
            idx += 1;
        }
    }
}
