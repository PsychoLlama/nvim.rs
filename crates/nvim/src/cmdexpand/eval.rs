//! The Vimscript face: `getcompletion()`, `getcompletiontype()`,
//! `cmdcomplete_info()`.
//!
//! [`f_getcompletion`] runs the whole classify-then-expand pipeline against a
//! string instead of the real command line, which is what makes it the
//! completion layer's differential oracle.  All three are rows in the
//! generated eval function table.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::cmdexpand::{WildMode, WildOpts};
use crate::cstr;
use crate::eval::typval::NumBuf;
use crate::message_fmt::c_str;
use crate::semsg;
use crate::types::{ExpandContext, VAR_STRING};
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

pub fn f_getcompletion(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut xpc: Expand = unsafe { core::mem::zeroed() };
    let mut filtered = false;
    let mut options = GETCOMPLETION;

    if tv_check_for_string_arg(args, 1).is_err() {
        return;
    }
    let type_0 = numbuf.string_ptr(&args[1]);

    if args.len() > 2 {
        filtered = tv_get_number_chk(&args[2]).unwrap_or(-1) != 0;
    }

    if p_wic.get() != 0 {
        options |= WildOpts::ICASE;
    }

    // For filtered results, 'wildignore' is used.
    if !filtered {
        options |= WildOpts::KEEP_ALL;
    }

    if args[0].v_type() != VAR_STRING {
        emsg(gettext(e_invarg));
        return;
    }
    let pattern = numbuf2.string_ptr(&args[0]);
    let mut pattern_start = pattern;

    // C's `goto theend`: the "cmdline" type takes the whole classifier and
    // skips the per-type switch entirely.
    if unsafe { cstr::eq_bytes(type_0, b"cmdline") } {
        let cmdline_len = unsafe { cstr::bytes_at(pattern) }.len() as c_int;
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
        xpc.xp_pattern_len = unsafe { cstr::bytes_at(xpc.xp_pattern) }.len();
        xpc.xp_col = cmdline_len;
    } else {
        unsafe { expand_init(&raw mut xpc) };
        xpc.xp_pattern = pattern as *mut c_char;
        xpc.xp_pattern_len = unsafe { cstr::bytes_at(xpc.xp_pattern) }.len();
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
                if !unsafe { cstr::starts_with(type_0, b"custom,") } {
                    // SAFETY: a message argument the caller holds as a NUL-terminated string.
                    let arg0 = unsafe { c_str(type_0) };
                    semsg!("E475: Invalid argument: {arg0}");
                    return;
                }
                xpc.xp_arg = unsafe { type_0.add(7) } as *mut c_char;
            }
            ExpandContext::UserList => {
                // Must be "customlist,funcname" pattern.
                if !unsafe { cstr::starts_with(type_0, b"customlist,") } {
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
                        None,
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
        xpc.xp_col = unsafe { cstr::bytes_at(xpc.xp_line) }.len() as c_int;
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
    tv_list_alloc_ret(result, xpc.xp_numfiles as ptrdiff_t);

    // SAFETY: the frame's return slot, holding the list just allocated.
    let retlist = result.list_or_null();
    for i in 0..xpc.xp_numfiles {
        unsafe { (*retlist).push_string(*xpc.xp_files.offset(i as isize), -1) };
    }
    unsafe { xfree(pat as *mut c_void) };
    unsafe { expand_cleanup(&raw mut xpc) };
}

/// `getcompletiontype()`: the completion type name a command line would use.
pub fn f_getcompletiontype(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_string(ptr::null_mut());

    if tv_check_for_string_arg(args, 0).is_err() {
        return;
    }

    let pat = numbuf.string_ptr(&args[0]);
    let mut xpc: Expand = unsafe { core::mem::zeroed() };
    unsafe { expand_init(&raw mut xpc) };

    let cmdline_len = unsafe { cstr::bytes_at(pat) }.len() as c_int;
    unsafe {
        set_cmd_context(
            &raw mut xpc,
            pat as *mut c_char,
            cmdline_len,
            cmdline_len,
            false,
        )
    };
    unsafe { (*result).write_string(cmdcomplete_type_to_str(xpc.xp_context, xpc.xp_arg)) };

    unsafe { expand_cleanup(&raw mut xpc) };
}

/// `cmdcomplete_info()`: the state of the completion in progress.
pub fn f_cmdcomplete_info(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let xpc = Cc::current().xpc();

    tv_dict_alloc_ret(result);
    if xpc.is_null() || unsafe { (*xpc).xp_files }.is_null() {
        return;
    }
    let retdict: *mut Dict = result.dict_or_null();

    // C's S_LEN(): `tv_dict_add_*` copies exactly `key_len` bytes, so the
    // key type is a plain `&str`.
    let add_str = |k: &str, v| unsafe { (*retdict).add_str(k.as_bytes(), v) };
    let add_nr = |k: &str, v| unsafe { (*retdict).add_number(k.as_bytes(), v) };
    let add_list = |k: &str, v| unsafe { (*retdict).add_list(k.as_bytes(), v) };

    let mut ret = add_str("cmdline_orig", cmdline_orig.get());
    if ret.is_ok() {
        ret = add_nr("pum_visible", pum_visible() as VarNumber);
    }
    if ret.is_ok() {
        ret = add_nr("selected", unsafe { (*xpc).xp_selected } as VarNumber);
    }
    if ret.is_ok() {
        let li = tv_list_alloc(unsafe { (*xpc).xp_numfiles } as ptrdiff_t);
        // A borrow of the list the dictionary is about to own: the matches
        // go in after it is in place, as upstream's did.
        let into = li.as_ptr();
        ret = add_list("matches", Some(li));
        let mut idx = 0;
        while ret.is_ok() && idx < unsafe { (*xpc).xp_numfiles } {
            unsafe { (*into).push_string(*(*xpc).xp_files.offset(idx as isize), -1) };
            idx += 1;
        }
    }
}
