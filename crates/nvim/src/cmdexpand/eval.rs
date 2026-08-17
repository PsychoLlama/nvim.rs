//! The Vimscript face: `getcompletion()`, `getcompletiontype()`,
//! `cmdcomplete_info()`.
//!
//! [`f_getcompletion`] runs the whole classify-then-expand pipeline against a
//! string instead of the real command line, which is what makes it the
//! completion layer's differential oracle.  All three keep `extern "C"`:
//! they are rows in the generated eval function table.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cmdexpand::{WildMode, WildOpts};
use crate::semsg_c;
use crate::types::{VAR_STRING, VAR_UNKNOWN};
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

/// `ExpandOne`'s `orig` argument, which this caller never has.
const NO_ORIG: *mut c_char = ptr::null_mut();

pub unsafe extern "C" fn f_getcompletion(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe {
        let mut xpc: expand_T = core::mem::zeroed();
        let mut filtered = false;
        let mut options = GETCOMPLETION;

        if tv_check_for_string_arg(argvars, 1) == FAIL {
            return;
        }
        let type_0 = tv_get_string(argvars.add(1));

        if (*argvars.add(2)).v_type != VAR_UNKNOWN {
            filtered = tv_get_number_chk(argvars.add(2), ptr::null_mut()) != 0;
        }

        if p_wic.get() != 0 {
            options |= WildOpts::ICASE;
        }

        // For filtered results, 'wildignore' is used.
        if !filtered {
            options |= WildOpts::KEEP_ALL;
        }

        if (*argvars).v_type != VAR_STRING {
            emsg(gettext(&raw const e_invarg as *const c_char));
            return;
        }
        let pattern = tv_get_string(argvars);
        let mut pattern_start = pattern;

        // C's `goto theend`: the "cmdline" type takes the whole classifier and
        // skips the per-type switch entirely.
        if strcmp(type_0, c"cmdline".as_ptr()) == 0 {
            let cmdline_len = strlen(pattern) as c_int;
            set_cmd_context(
                &raw mut xpc,
                pattern as *mut c_char,
                cmdline_len,
                cmdline_len,
                false,
            );
            pattern_start = xpc.xp_pattern;
            xpc.xp_pattern_len = strlen(xpc.xp_pattern);
            xpc.xp_col = cmdline_len;
        } else {
            ExpandInit(&raw mut xpc);
            xpc.xp_pattern = pattern as *mut c_char;
            xpc.xp_pattern_len = strlen(xpc.xp_pattern);
            xpc.xp_line = pattern as *mut c_char;

            xpc.xp_context = cmdcomplete_str_to_type(type_0);
            match xpc.xp_context {
                EXPAND_NOTHING => {
                    semsg_c!(gettext(&raw const e_invarg2 as *const c_char), type_0);
                    return;
                }
                EXPAND_USER_DEFINED => {
                    // Must be "custom,funcname" pattern.
                    if strncmp(type_0, c"custom,".as_ptr(), 7) != 0 {
                        semsg_c!(gettext(&raw const e_invarg2 as *const c_char), type_0);
                        return;
                    }
                    xpc.xp_arg = type_0.add(7) as *mut c_char;
                }
                EXPAND_USER_LIST => {
                    // Must be "customlist,funcname" pattern.
                    if strncmp(type_0, c"customlist,".as_ptr(), 11) != 0 {
                        semsg_c!(gettext(&raw const e_invarg2 as *const c_char), type_0);
                        return;
                    }
                    xpc.xp_arg = type_0.add(11) as *mut c_char;
                }
                // The four generators below move `xp_pattern` forward inside
                // the string, so the length has to follow it.
                EXPAND_MENUS => {
                    set_context_in_menu_cmd(&raw mut xpc, c"menu".as_ptr(), xpc.xp_pattern, false);
                    xpc.xp_pattern_len -= xpc.xp_pattern.offset_from(pattern_start) as size_t;
                }
                EXPAND_SIGN => {
                    set_context_in_sign_cmd(&raw mut xpc, xpc.xp_pattern);
                    xpc.xp_pattern_len -= xpc.xp_pattern.offset_from(pattern_start) as size_t;
                }
                EXPAND_RUNTIME => {
                    set_context_in_runtime_cmd(&raw mut xpc, xpc.xp_pattern);
                    xpc.xp_pattern_len -= xpc.xp_pattern.offset_from(pattern_start) as size_t;
                }
                EXPAND_SHELLCMDLINE => {
                    let mut context = EXPAND_SHELLCMDLINE;
                    set_context_for_wildcard_arg(
                        ptr::null_mut(),
                        xpc.xp_pattern,
                        false,
                        &raw mut xpc,
                        &raw mut context,
                    );
                    xpc.xp_pattern_len -= xpc.xp_pattern.offset_from(pattern_start) as size_t;
                }
                EXPAND_FILETYPECMD => filetype_expand_what.set(EXP_FILETYPECMD_ALL),
                _ => {}
            }
        }

        if xpc.xp_context == EXPAND_LUA {
            xpc.xp_col = strlen(xpc.xp_line) as c_int;
            nlua_expand_pat(&raw mut xpc);
            xpc.xp_pattern_len -= xpc.xp_pattern.offset_from(pattern_start) as size_t;
        }

        let pat = if cmdline_fuzzy_completion_supported(&raw mut xpc) {
            // When fuzzy matching, don't modify the search string.
            xmemdupz(xpc.xp_pattern as *const c_void, xpc.xp_pattern_len) as *mut c_char
        } else {
            addstar(xpc.xp_pattern, xpc.xp_pattern_len, xpc.xp_context)
        };

        ExpandOne(&raw mut xpc, pat, NO_ORIG, options, WildMode::AllKeep);
        tv_list_alloc_ret(rettv, xpc.xp_numfiles as ptrdiff_t);

        for i in 0..xpc.xp_numfiles {
            tv_list_append_string((*rettv).vval.v_list, *xpc.xp_files.offset(i as isize), -1);
        }
        xfree(pat as *mut c_void);
        ExpandCleanup(&raw mut xpc);
    }
}

/// `getcompletiontype()`: the completion type name a command line would use.
pub unsafe extern "C" fn f_getcompletiontype(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = ptr::null_mut();

        if tv_check_for_string_arg(argvars, 0) == FAIL {
            return;
        }

        let pat = tv_get_string(argvars);
        let mut xpc: expand_T = core::mem::zeroed();
        ExpandInit(&raw mut xpc);

        let cmdline_len = strlen(pat) as c_int;
        set_cmd_context(
            &raw mut xpc,
            pat as *mut c_char,
            cmdline_len,
            cmdline_len,
            false,
        );
        (*rettv).vval.v_string = cmdcomplete_type_to_str(xpc.xp_context, xpc.xp_arg);

        ExpandCleanup(&raw mut xpc);
    }
}

/// `cmdcomplete_info()`: the state of the completion in progress.
pub unsafe extern "C" fn f_cmdcomplete_info(
    _argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe {
        let ccline: *mut CmdlineInfo = get_cmdline_info();

        tv_dict_alloc_ret(rettv);
        if ccline.is_null() || (*ccline).xpc.is_null() || (*(*ccline).xpc).xp_files.is_null() {
            return;
        }
        let xpc = (*ccline).xpc;
        let retdict: *mut dict_T = (*rettv).vval.v_dict;

        // C's S_LEN(): `tv_dict_add_*` copies exactly `key_len` bytes, so the
        // key type is a plain `&str`.
        let add_str = |k: &str, v| tv_dict_add_str(retdict, k.as_ptr().cast(), k.len(), v);
        let add_nr = |k: &str, v| tv_dict_add_nr(retdict, k.as_ptr().cast(), k.len(), v);
        let add_list = |k: &str, v| tv_dict_add_list(retdict, k.as_ptr().cast(), k.len(), v);

        let mut ret = add_str("cmdline_orig", cmdline_orig.get());
        if ret == OK {
            ret = add_nr("pum_visible", pum_visible() as varnumber_T);
        }
        if ret == OK {
            ret = add_nr("selected", (*xpc).xp_selected as varnumber_T);
        }
        if ret == OK {
            let li = tv_list_alloc((*xpc).xp_numfiles as ptrdiff_t);
            ret = add_list("matches", li);
            let mut idx = 0;
            while ret == OK && idx < (*xpc).xp_numfiles {
                tv_list_append_string(li, *(*xpc).xp_files.offset(idx as isize), -1);
                idx += 1;
            }
        }
    }
}
