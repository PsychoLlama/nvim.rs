//! The Vimscript face: `getcmdline()`, `setcmdline()`, `getcmdpos()`, …
//!
//! All of them read or write the *current* `ccline` through
//! [`get_ccline_ptr`], which is what makes them answer nothing outside a
//! command line and answer the enclosing one from inside `<C-r>=`.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn get_cmdline_info() -> *mut CmdlineInfo {
    return ccline.ptr();
}

pub unsafe extern "C" fn get_cmdline_last_prompt_id() -> ::core::ffi::c_uint {
    return last_prompt_id.get();
}

pub(crate) unsafe extern "C" fn get_ccline_ptr() -> *mut CmdlineInfo {
    unsafe {
        if State.get() & MODE_CMDLINE == 0 as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<CmdlineInfo>();
        } else if !(*ccline.ptr()).cmdbuff.is_null() {
            return ccline.ptr();
        } else if !(*ccline.ptr()).prev_ccline.is_null()
            && !(*(*ccline.ptr()).prev_ccline).cmdbuff.is_null()
        {
            return (*ccline.ptr()).prev_ccline;
        } else {
            return ::core::ptr::null_mut::<CmdlineInfo>();
        };
    }
}

pub(crate) unsafe extern "C" fn get_cmdline_type() -> ::core::ffi::c_int {
    unsafe {
        let mut p: *mut CmdlineInfo = get_ccline_ptr();
        if p.is_null() {
            return NUL;
        }
        if (*p).cmdfirstc == NUL {
            return if (*p).input_fn != 0 {
                '@' as ::core::ffi::c_int
            } else {
                '-' as ::core::ffi::c_int
            };
        }
        return (*p).cmdfirstc;
    }
}

pub(crate) unsafe extern "C" fn get_cmdline_str() -> *mut ::core::ffi::c_char {
    unsafe {
        if cmdline_star.get() > 0 as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut p: *mut CmdlineInfo = get_ccline_ptr();
        if p.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        return xstrnsave((*p).cmdbuff, (*p).cmdlen as size_t);
    }
}

pub(crate) unsafe extern "C" fn get_cmdline_completion_pattern() -> *mut ::core::ffi::c_char {
    unsafe {
        if cmdline_star.get() > 0 as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut p: *mut CmdlineInfo = get_ccline_ptr();
        if p.is_null() || (*p).xpc.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut xp_context: ::core::ffi::c_int = (*(*p).xpc).xp_context;
        if xp_context == EXPAND_NOTHING as ::core::ffi::c_int {
            set_expand_context((*p).xpc);
            xp_context = (*(*p).xpc).xp_context;
            (*(*p).xpc).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
        }
        if xp_context == EXPAND_UNSUCCESSFUL as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut compl_pat: *mut ::core::ffi::c_char = (*(*p).xpc).xp_pattern;
        if compl_pat.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        return xstrdup(compl_pat);
    }
}

pub(crate) unsafe extern "C" fn get_cmdline_completion() -> *mut ::core::ffi::c_char {
    unsafe {
        if cmdline_star.get() > 0 as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut p: *mut CmdlineInfo = get_ccline_ptr();
        if p.is_null() || (*p).xpc.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut xp_context: ::core::ffi::c_int = (*(*p).xpc).xp_context;
        if xp_context == EXPAND_NOTHING as ::core::ffi::c_int {
            set_expand_context((*p).xpc);
            xp_context = (*(*p).xpc).xp_context;
            (*(*p).xpc).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
        }
        if xp_context == EXPAND_UNSUCCESSFUL as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        return cmdcomplete_type_to_str(xp_context, (*(*p).xpc).xp_arg);
    }
}

pub unsafe extern "C" fn f_getcmdcomplpat(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = get_cmdline_completion_pattern();
    }
}

pub unsafe extern "C" fn f_getcmdcompltype(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = get_cmdline_completion();
    }
}

pub unsafe extern "C" fn f_getcmdline(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = get_cmdline_str();
    }
}

pub unsafe extern "C" fn f_getcmdpos(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut p: *mut CmdlineInfo = get_ccline_ptr();
        (*rettv).vval.v_number = (if !p.is_null() {
            (*p).cmdpos + 1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as varnumber_T;
    }
}

pub unsafe extern "C" fn f_getcmdprompt(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut p: *mut CmdlineInfo = get_ccline_ptr();
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = if !p.is_null() && !(*p).cmdprompt.is_null() {
            xstrdup((*p).cmdprompt)
        } else {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        };
    }
}

pub unsafe extern "C" fn f_getcmdscreenpos(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut p: *mut CmdlineInfo = get_ccline_ptr();
        (*rettv).vval.v_number = (if !p.is_null() {
            (*p).cmdspos + 1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as varnumber_T;
    }
}

pub unsafe extern "C" fn f_getcmdtype(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = xmallocz(1 as size_t) as *mut ::core::ffi::c_char;
        *(*rettv)
            .vval
            .v_string
            .offset(0 as ::core::ffi::c_int as isize) = get_cmdline_type() as ::core::ffi::c_char;
    }
}

pub(crate) unsafe extern "C" fn set_cmdline_str(
    mut str: *const ::core::ffi::c_char,
    mut pos: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut p: *mut CmdlineInfo = get_ccline_ptr();
        if p.is_null() {
            return 1 as ::core::ffi::c_int;
        }
        let mut len: ::core::ffi::c_int = strlen(str) as ::core::ffi::c_int;
        realloc_cmdbuff(len + 1 as ::core::ffi::c_int);
        (*p).cmdlen = len;
        strcpy((*p).cmdbuff, str as *mut ::core::ffi::c_char);
        (*p).cmdpos = if pos < 0 as ::core::ffi::c_int || pos > (*p).cmdlen {
            (*p).cmdlen
        } else {
            pos
        };
        new_cmdpos.set((*p).cmdpos);
        (*p).cmdbuff_replaced = true_0 != 0;
        redrawcmd();
        do_autocmd_cmdlinechanged(get_cmdline_type());
        return 0 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn set_cmdline_pos(mut pos: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        let mut p: *mut CmdlineInfo = get_ccline_ptr();
        if p.is_null() {
            return 1 as ::core::ffi::c_int;
        }
        new_cmdpos.set(if 0 as ::core::ffi::c_int > pos {
            0 as ::core::ffi::c_int
        } else {
            pos
        });
        return 0 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn f_setcmdline(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        if tv_check_for_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
            || tv_check_for_opt_number_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
        {
            return;
        }
        let mut pos: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut error: bool = false_0 != 0;
            pos = tv_get_number_chk(
                argvars.offset(1 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int;
            if error {
                return;
            }
            if pos < 0 as ::core::ffi::c_int {
                emsg(gettext(&raw const e_positive as *const ::core::ffi::c_char));
                return;
            }
        }
        (*rettv).vval.v_number = set_cmdline_str(
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
            pos,
        ) as varnumber_T;
    }
}

pub unsafe extern "C" fn f_setcmdpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let pos: ::core::ffi::c_int =
            tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int;
        if pos >= 0 as ::core::ffi::c_int {
            (*rettv).vval.v_number = set_cmdline_pos(pos) as varnumber_T;
        }
    }
}

pub unsafe extern "C" fn get_cmdline_firstc() -> ::core::ffi::c_int {
    unsafe {
        return (*ccline.ptr()).cmdfirstc;
    }
}

pub unsafe extern "C" fn f_wildtrigger(
    mut _argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        if State.get() & MODE_CMDLINE == 0
            || char_avail() as ::core::ffi::c_int != 0
            || wild_menu_showing.get() != 0
            || cmdline_pum_active() as ::core::ffi::c_int != 0
        {
            return;
        }
        let mut cmd_type: ::core::ffi::c_int = get_cmdline_type();
        if cmd_type == ':' as ::core::ffi::c_int
            || cmd_type == '/' as ::core::ffi::c_int
            || cmd_type == '?' as ::core::ffi::c_int
        {
            let mut key_string: [uint8_t; 4] = [0; 4];
            key_string[0 as ::core::ffi::c_int as usize] = K_SPECIAL as uint8_t;
            key_string[1 as ::core::ffi::c_int as usize] = KS_EXTRA as uint8_t;
            key_string[2 as ::core::ffi::c_int as usize] = KE_WILD as ::core::ffi::c_int as uint8_t;
            key_string[3 as ::core::ffi::c_int as usize] = NUL as uint8_t;
            ins_typebuf(
                &raw mut key_string as *mut uint8_t as *mut ::core::ffi::c_char,
                REMAP_NONE as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                true_0 != 0,
                false_0 != 0,
            );
        }
    }
}
