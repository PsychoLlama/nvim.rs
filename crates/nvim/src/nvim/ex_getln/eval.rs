//! The Vimscript face: `getcmdline()`, `setcmdline()`, `getcmdpos()`, …
//!
//! All of them read or write the *current* `ccline` through
//! [`get_ccline_ptr`], which is what makes them answer nothing outside a
//! command line and answer the enclosing one from inside `<C-r>=`.
//!
//! The `f_*` rows keep their `extern "C"` ABI: they are the function pointers
//! in `eval/funcs/table`, and `VimLFunc` is a C-ABI pointer type.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

/// The command line being edited, whether or not it holds any text.
///
/// Unlike [`get_ccline_ptr`] this is the raw `ccline` static: it is what the
/// redraw and completion code needs, which runs while `cmdbuff` is still
/// being built.
pub fn get_cmdline_info() -> *mut CmdlineInfo {
    ccline.ptr()
}

/// The id of the most recently *started* command line, which
/// [`super::color::color_cmdline`] compares against a cached colouring.
pub fn get_cmdline_last_prompt_id() -> ::core::ffi::c_uint {
    last_prompt_id.get()
}

/// The command line info the Vimscript functions should answer about, or NULL
/// when there is none.
///
/// `save_cmdline()` clears `ccline` and moves the previous value to
/// `prev_ccline`, so inside a `<C-r>=` expression the *enclosing* command
/// line is the one with the text.
pub(crate) unsafe fn get_ccline_ptr() -> *mut CmdlineInfo {
    unsafe {
        if State.get() & MODE_CMDLINE == 0 {
            ::core::ptr::null_mut::<CmdlineInfo>()
        } else if !(*ccline.ptr()).cmdbuff.is_null() {
            ccline.ptr()
        } else if !(*ccline.ptr()).prev_ccline.is_null()
            && !(*(*ccline.ptr()).prev_ccline).cmdbuff.is_null()
        {
            (*ccline.ptr()).prev_ccline
        } else {
            ::core::ptr::null_mut::<CmdlineInfo>()
        }
    }
}

/// The current command-line type: `:`, `/`, `?`, `@`, `>` or `-`, and `NUL`
/// when no command line is being edited.
pub(crate) unsafe fn get_cmdline_type() -> ::core::ffi::c_int {
    unsafe {
        let p = get_ccline_ptr();
        if p.is_null() {
            return NUL;
        }
        if (*p).cmdfirstc == NUL {
            // No first character: `input()` reports '@', a `:insert` style
            // line-getter '-'.
            return if (*p).input_fn != 0 {
                '@' as ::core::ffi::c_int
            } else {
                '-' as ::core::ffi::c_int
            };
        }
        (*p).cmdfirstc
    }
}

/// The current command line, allocated; NULL when there is none or the line
/// is obscured (`inputsecret()`).
pub(crate) unsafe fn get_cmdline_str() -> *mut ::core::ffi::c_char {
    unsafe {
        if cmdline_star.get() > 0 {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let p = get_ccline_ptr();
        if p.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        xstrnsave((*p).cmdbuff, (*p).cmdlen as size_t)
    }
}

/// The completion state of the current command line, computed on demand:
/// the `expand_T` and the context it resolved to.
///
/// When nothing has asked yet the context is `EXPAND_NOTHING`, so
/// `set_expand_context` runs and the field is then put *back* to
/// `EXPAND_NOTHING` — the real completion has to recompute it at the wildcard
/// key.  Hence the context is returned rather than left to the caller to
/// re-read: after the restore the field no longer holds it.
///
/// `None` means there is nothing to report: no command line, an obscured one
/// (`inputsecret()`), or `EXPAND_UNSUCCESSFUL`.
unsafe fn cmdline_completion_state() -> Option<(*mut expand_T, ::core::ffi::c_int)> {
    unsafe {
        if cmdline_star.get() > 0 {
            return None;
        }
        let p = get_ccline_ptr();
        if p.is_null() || (*p).xpc.is_null() {
            return None;
        }
        let xpc = (*p).xpc;
        let mut xp_context = (*xpc).xp_context;
        if xp_context == EXPAND_NOTHING {
            set_expand_context(xpc);
            xp_context = (*xpc).xp_context;
            (*xpc).xp_context = EXPAND_NOTHING;
        }
        if xp_context == EXPAND_UNSUCCESSFUL {
            return None;
        }
        Some((xpc, xp_context))
    }
}

/// `getcmdcomplpat()` function: the pattern completion would expand.
pub unsafe extern "C" fn f_getcmdcomplpat(
    _argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if let Some((xpc, _)) = cmdline_completion_state() {
            let compl_pat = (*xpc).xp_pattern;
            if !compl_pat.is_null() {
                (*rettv).vval.v_string = xstrdup(compl_pat);
            }
        }
    }
}

/// `getcmdcompltype()` function: the completion type's name.
pub unsafe extern "C" fn f_getcmdcompltype(
    _argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = match cmdline_completion_state() {
            Some((xpc, xp_context)) => cmdcomplete_type_to_str(xp_context, (*xpc).xp_arg),
            None => ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
    }
}

/// `getcmdline()` function.
pub unsafe extern "C" fn f_getcmdline(
    _argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = get_cmdline_str();
    }
}

/// `getcmdpos()` function.
pub unsafe extern "C" fn f_getcmdpos(
    _argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe {
        let p = get_ccline_ptr();
        (*rettv).vval.v_number = if !p.is_null() {
            ((*p).cmdpos + 1) as varnumber_T
        } else {
            0
        };
    }
}

/// `getcmdprompt()` function.
pub unsafe extern "C" fn f_getcmdprompt(
    _argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe {
        let p = get_ccline_ptr();
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = if !p.is_null() && !(*p).cmdprompt.is_null() {
            xstrdup((*p).cmdprompt)
        } else {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        };
    }
}

/// `getcmdscreenpos()` function.
pub unsafe extern "C" fn f_getcmdscreenpos(
    _argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe {
        let p = get_ccline_ptr();
        (*rettv).vval.v_number = if !p.is_null() {
            ((*p).cmdspos + 1) as varnumber_T
        } else {
            0
        };
    }
}

/// `getcmdtype()` function.
pub unsafe extern "C" fn f_getcmdtype(
    _argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).v_type = VAR_STRING;
        // One character plus the terminator `xmallocz` appends.
        (*rettv).vval.v_string = xmallocz(1) as *mut ::core::ffi::c_char;
        *(*rettv).vval.v_string.offset(0) = get_cmdline_type() as ::core::ffi::c_char;
    }
}

/// Replace the command line with `str` and put the cursor at `pos`.
///
/// A negative or out-of-range `pos` means the end of the line.  Answers 1
/// when there is no command line to set, 0 on success.
pub(crate) unsafe fn set_cmdline_str(
    str: *const ::core::ffi::c_char,
    pos: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let p = get_ccline_ptr();
        if p.is_null() {
            return 1;
        }

        let len = strlen(str) as ::core::ffi::c_int;
        realloc_cmdbuff(len + 1);
        (*p).cmdlen = len;
        strcpy((*p).cmdbuff, str as *mut ::core::ffi::c_char);

        (*p).cmdpos = if pos < 0 || pos > (*p).cmdlen {
            (*p).cmdlen
        } else {
            pos
        };
        new_cmdpos.set((*p).cmdpos);
        (*p).cmdbuff_replaced = true;

        redrawcmd();

        // Trigger CmdlineChanged autocommands.
        do_autocmd_cmdlinechanged(get_cmdline_type());

        0
    }
}

/// Remember `pos` as the byte position to put the cursor at, zero-based.
///
/// It is not applied here but after `CTRL-\ e` or `CTRL-R =` has finished
/// changing the command line.  Answers 1 when there is no command line, 0 on
/// success.
pub(crate) unsafe fn set_cmdline_pos(pos: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        let p = get_ccline_ptr();
        if p.is_null() {
            return 1;
        }
        new_cmdpos.set(pos.max(0));
        0
    }
}

/// `setcmdline()` function.
pub unsafe extern "C" fn f_setcmdline(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe {
        if tv_check_for_string_arg(argvars, 0) == FAIL
            || tv_check_for_opt_number_arg(argvars, 1) == FAIL
        {
            return;
        }

        let mut pos = -1;
        if (*argvars.offset(1)).v_type != VAR_UNKNOWN {
            let mut error = false;
            pos = tv_get_number_chk(argvars.offset(1), &raw mut error) as ::core::ffi::c_int - 1;
            if error {
                return;
            }
            if pos < 0 {
                emsg(gettext(e_positive.ptr() as *const ::core::ffi::c_char));
                return;
            }
        }

        // tv_get_string() so that a NULL string reads as an empty one.
        (*rettv).vval.v_number =
            set_cmdline_str(tv_get_string(argvars.offset(0)), pos) as varnumber_T;
    }
}

/// `setcmdpos()` function.
pub unsafe extern "C" fn f_setcmdpos(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe {
        let pos = tv_get_number(argvars.offset(0)) as ::core::ffi::c_int - 1;
        if pos >= 0 {
            (*rettv).vval.v_number = set_cmdline_pos(pos) as varnumber_T;
        }
    }
}

/// The first character of the current command line (`:`, `/`, `?`, …).
pub unsafe fn get_cmdline_firstc() -> ::core::ffi::c_int {
    unsafe { (*ccline.ptr()).cmdfirstc }
}

/// `wildtrigger()` function: ask the key loop to complete, as if `'wildchar'`
/// had been typed.
pub unsafe extern "C" fn f_wildtrigger(
    _argvars: *mut typval_T,
    _rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe {
        if State.get() & MODE_CMDLINE == 0
            || char_avail()
            || wild_menu_showing.get() != 0
            || cmdline_pum_active()
        {
            return;
        }

        let cmd_type = get_cmdline_type();
        if cmd_type == ':' as ::core::ffi::c_int
            || cmd_type == '/' as ::core::ffi::c_int
            || cmd_type == '?' as ::core::ffi::c_int
        {
            // K_WILD as a single special key, pushed into the typeahead.
            let mut key_string: [uint8_t; 4] = [
                K_SPECIAL as uint8_t,
                KS_EXTRA as uint8_t,
                KE_WILD as uint8_t,
                NUL as uint8_t,
            ];
            ins_typebuf(
                key_string.as_mut_ptr() as *mut ::core::ffi::c_char,
                REMAP_NONE,
                0,
                true,
                false,
            );
        }
    }
}
