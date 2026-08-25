//! The Vimscript face: `getcmdline()`, `setcmdline()`, `getcmdpos()`, …
//!
//! All of them read or write the *current* `ccline` through
//! [`get_ccline_ptr`], which is what makes them answer nothing outside a
//! command line and answer the enclosing one from inside `<C-r>=`.
//!
//! The `f_*` rows are the `VimLFunc` function pointers in
//! `eval/funcs/table`.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::keycodes::KE_WILD;
use crate::types::{ExpandContext, FAIL, NUL, VAR_STRING, VAR_UNKNOWN};

/// The command line being edited, whether or not it holds any text.
///
/// Unlike [`get_ccline_ptr`] this is the raw `ccline` static: it is what the
/// redraw and completion code needs, which runs while `cmdbuff` is still
/// being built.
pub fn get_cmdline_info() -> *mut CmdlineInfo {
    Cc::current().raw()
}

/// Whether a command line is being edited at all: C's
/// `get_cmdline_info()->cmdbuff != NULL`, which nothing outside `ex_getln/`
/// can ask now that the buffer is owned.
pub(crate) fn cmdline_in_use() -> bool {
    Cc::current().in_use()
}

/// The id of the most recently *started* command line, which
/// [`super::color::color_cmdline`] compares against a cached colouring.
pub fn get_cmdline_last_prompt_id() -> ::core::ffi::c_uint {
    last_prompt_id.get()
}

/// The command line the Vimscript functions should answer about, if any.
///
/// `save_cmdline()` suspends `ccline` onto the saved stack, so inside a
/// `<C-r>=` expression the *enclosing* command line -- one level out -- is
/// the one with the text.
pub(crate) fn get_ccline_ptr() -> Option<Cc> {
    if State.get() & MODE_CMDLINE == 0 {
        return None;
    }
    let depth = usize::from(!Cc::current().in_use());
    cmdline_at(depth).filter(|cc| cc.in_use())
}

/// The current command-line type: `:`, `/`, `?`, `@`, `>` or `-`, and `NUL`
/// when no command line is being edited.
pub(crate) fn get_cmdline_type() -> ::core::ffi::c_int {
    let Some(p) = get_ccline_ptr() else {
        return NUL;
    };
    if p.cmdfirstc == NUL {
        // No first character: `input()` reports '@', a `:insert` style
        // line-getter '-'.
        return if p.input_fn != 0 {
            '@' as ::core::ffi::c_int
        } else {
            '-' as ::core::ffi::c_int
        };
    }
    p.cmdfirstc
}

/// The current command line, allocated; NULL when there is none or the line
/// is obscured (`inputsecret()`).
pub(crate) unsafe fn get_cmdline_str() -> *mut ::core::ffi::c_char {
    unsafe {
        if cmdline_star.get() > 0 {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let Some(p) = get_ccline_ptr() else {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        };
        xstrnsave(p.text(), p.len() as size_t)
    }
}

/// The completion state of the current command line, computed on demand:
/// the `expand_T` and the context it resolved to.
///
/// When nothing has asked yet the context is `ExpandContext::Nothing`, so
/// `set_expand_context` runs and the field is then put *back* to
/// `ExpandContext::Nothing` — the real completion has to recompute it at the wildcard
/// key.  Hence the context is returned rather than left to the caller to
/// re-read: after the restore the field no longer holds it.
///
/// `None` means there is nothing to report: no command line, an obscured one
/// (`inputsecret()`), or `ExpandContext::Unsuccessful`.
unsafe fn cmdline_completion_state() -> Option<(*mut expand_T, ExpandContext)> {
    unsafe {
        if cmdline_star.get() > 0 {
            return None;
        }
        let xpc = get_ccline_ptr().map_or(::core::ptr::null_mut(), |p| p.xpc);
        if xpc.is_null() {
            return None;
        }
        let mut xp_context = (*xpc).xp_context;
        if xp_context == ExpandContext::Nothing {
            set_expand_context(xpc);
            xp_context = (*xpc).xp_context;
            (*xpc).xp_context = ExpandContext::Nothing;
        }
        if xp_context == ExpandContext::Unsuccessful {
            return None;
        }
        Some((xpc, xp_context))
    }
}

/// `getcmdcomplpat()` function: the pattern completion would expand.
pub unsafe fn f_getcmdcomplpat(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
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
pub unsafe fn f_getcmdcompltype(
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
pub unsafe fn f_getcmdline(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = get_cmdline_str();
    }
}

/// `getcmdpos()` function.
pub unsafe fn f_getcmdpos(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        (*rettv).vval.v_number = get_ccline_ptr().map_or(0, |p| (p.cmdpos + 1) as varnumber_T);
    }
}

/// `getcmdprompt()` function.
pub unsafe fn f_getcmdprompt(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = get_ccline_ptr()
            .filter(|p| !p.cmdprompt.is_null())
            .map_or(::core::ptr::null_mut(), |p| xstrdup(p.cmdprompt));
    }
}

/// `getcmdscreenpos()` function.
pub unsafe fn f_getcmdscreenpos(
    _argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_number = get_ccline_ptr().map_or(0, |p| (p.cmdspos + 1) as varnumber_T);
    }
}

/// `getcmdtype()` function.
pub unsafe fn f_getcmdtype(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
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
        let Some(mut p) = get_ccline_ptr() else {
            return 1;
        };

        // `p` is not always `ccline`: inside `<C-r>=` it is the command line
        // one level out. C resized `ccline` here whichever line it then wrote
        // to, and overran the other one -- see the upstream note on
        // `set_cmdline_str`.
        p.set_cstr(str);

        p.cmdpos = if pos < 0 || pos > p.len() {
            p.len()
        } else {
            pos
        };
        new_cmdpos.set(p.cmdpos);
        p.cmdbuff_replaced = true;

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
pub(crate) fn set_cmdline_pos(pos: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if get_ccline_ptr().is_none() {
        return 1;
    }
    new_cmdpos.set(pos.max(0));
    0
}

/// `setcmdline()` function.
pub unsafe fn f_setcmdline(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
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
                emsg(gettext(e_positive.as_ptr()));
                return;
            }
        }

        // tv_get_string() so that a NULL string reads as an empty one.
        (*rettv).vval.v_number =
            set_cmdline_str(tv_get_string(argvars.offset(0)), pos) as varnumber_T;
    }
}

/// `setcmdpos()` function.
pub unsafe fn f_setcmdpos(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        let pos = tv_get_number(argvars.offset(0)) as ::core::ffi::c_int - 1;
        if pos >= 0 {
            (*rettv).vval.v_number = set_cmdline_pos(pos) as varnumber_T;
        }
    }
}

/// The first character of the current command line (`:`, `/`, `?`, …).
pub fn get_cmdline_firstc() -> ::core::ffi::c_int {
    Cc::current().cmdfirstc
}

/// `wildtrigger()` function: ask the key loop to complete, as if `'wildchar'`
/// had been typed.
pub unsafe fn f_wildtrigger(_argvars: *mut typval_T, _rettv: *mut typval_T, _fptr: EvalFuncData) {
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
