//! `:echo`, `:echohl`, `:execute` and where a variable was last set.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int, c_void};
use core::ptr::null_mut;

use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::charset::skipwhite;
use crate::src::nvim::eval::encode::{encode_tv2echo, encode_tv2string};
use crate::src::nvim::eval::typval::{tv_clear, tv_get_string};
use crate::src::nvim::eval::userfunc::{restore_funccal, save_funccal};
use crate::src::nvim::eval::vars::set_var;
use crate::src::nvim::eval::{
    DOCMD_NOWAIT, DOCMD_VERBOSE, FAIL, NUL, OK, clear_evalarg, echo_hl_id, eval1, eval1_emsg,
    fill_evalarg_from_eap,
};
use crate::src::nvim::ex_docmd::{check_nextcmd, do_cmdline};
use crate::src::nvim::ex_eval::aborting;
use crate::src::nvim::garray::{ga_clear, ga_grow, ga_init};
use crate::src::nvim::highlight_group::{HLF_E, syn_name2id};
use crate::src::nvim::main::{
    called_emsg, did_emsg, e_invexpr2, emsg_skip, force_abort, got_int, line_msg, msg_didout,
    msg_ext_skip_verbose, need_clr_eos,
};
use crate::src::nvim::memory::xfree;
use crate::src::nvim::message::{
    emsg_multiline, msg, msg_clr_eos, msg_end, msg_ext_set_append, msg_ext_set_kind, msg_multiline,
    msg_outnum, msg_puts, msg_puts_hl, msg_puts_len, msg_sb_eol, msg_start, verbose_enter,
    verbose_leave,
};
use crate::src::nvim::os::libc::{gettext, memcpy, strlen};
use crate::src::nvim::runtime::{get_scriptname, script_is_lua};
use crate::src::nvim::types::ui::kUIMessages;
use crate::src::nvim::types::{
    CMD_echo, CMD_echoerr, CMD_echomsg, CMD_echon, CMD_execute, VAR_FLAVOUR_DEFAULT,
    VAR_FLAVOUR_SESSION, VAR_FLAVOUR_SHADA, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, evalarg_T,
    exarg_T, funccal_entry_T, garray_T, linenr_T, ptrdiff_t, sctx_T, size_t, typval_T,
    typval_vval_union, var_flavour_T,
};
use crate::src::nvim::ui::ui_has;

/// A freshly declared typval.
const UNSET_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VAR_UNLOCKED,
    vval: typval_vval_union { v_number: 0 },
};

/// A freshly declared `evalarg_T`.
const UNSET_EVALARG: evalarg_T = evalarg_T {
    eval_flags: 0,
    eval_getline: None,
    eval_cookie: null_mut(),
    eval_tofree: null_mut(),
};

/// An empty growable array.
const UNSET_GA: garray_T = garray_T {
    ga_len: 0,
    ga_maxlen: 0,
    ga_itemsize: 0,
    ga_growsize: 0,
    ga_data: null_mut(),
};

/// Does this byte end the `:echo` argument list?
fn ends_args(c: c_char) -> bool {
    c as c_int == NUL || c == b'|' as c_char || c == b'\n' as c_char
}

/// `:echo` and `:echon`.
///
/// # Safety
/// `eap` must be valid.
pub unsafe fn ex_echo(eap: *mut exarg_T) {
    unsafe {
        let mut arg: *mut c_char = (*eap).arg;
        let mut rettv = UNSET_TV;
        let mut atstart = true;
        let mut need_clear = true;
        let did_emsg_before = did_emsg.get();
        let called_emsg_before = called_emsg.get();

        let mut evalarg = UNSET_EVALARG;
        fill_evalarg_from_eap(&raw mut evalarg, eap, (*eap).skip != 0);
        if (*eap).skip != 0 {
            *emsg_skip.ptr() += 1;
        }

        while !ends_args(*arg) && !got_int.get() {
            // The flag is set across the evaluation only: an expression
            // that writes to the screen itself must not have the rest of
            // the line cleared out from under it.
            need_clr_eos.set(true);
            let start = arg;
            if eval1(&raw mut arg, &raw mut rettv, &raw mut evalarg) == FAIL {
                if !aborting()
                    && did_emsg.get() == did_emsg_before
                    && called_emsg.get() == called_emsg_before
                {
                    semsg_c!(gettext(e_invexpr2.ptr().cast()), start);
                }
                need_clr_eos.set(false);
                break;
            }
            need_clr_eos.set(false);

            if (*eap).skip == 0 {
                if atstart {
                    atstart = false;
                    msg_ext_set_append((*eap).cmdidx == CMD_echon);
                    msg_ext_set_kind(c"echo".as_ptr());
                    if (*eap).cmdidx == CMD_echo {
                        if !msg_didout.get() {
                            msg_sb_eol();
                        }
                        msg_start();
                    }
                } else if (*eap).cmdidx == CMD_echo {
                    // `:echo` separates its arguments; `:echon` does not.
                    msg_puts_hl(c" ".as_ptr(), echo_hl_id.get(), false);
                }
                let tofree = encode_tv2echo(&raw mut rettv, null_mut::<size_t>());
                msg_multiline(
                    cstr_as_string(tofree),
                    echo_hl_id.get(),
                    true,
                    false,
                    &raw mut need_clear,
                );
                xfree(tofree as *mut c_void);
            }
            tv_clear(&raw mut rettv);
            arg = skipwhite(arg);
        }

        (*eap).nextcmd = check_nextcmd(arg);
        clear_evalarg(&raw mut evalarg, eap);
        msg_ext_set_append(false);

        if (*eap).skip != 0 {
            *emsg_skip.ptr() -= 1;
            return;
        }
        if ui_has(kUIMessages) && ends_args(*(*eap).arg) {
            // A bare `:echo` still has to produce an (empty) message.
            msg_puts_len(c"".as_ptr(), 0 as ptrdiff_t, 0, false);
        } else if need_clear {
            msg_clr_eos();
        }
        if (*eap).cmdidx == CMD_echo {
            msg_end();
        }
    }
}

/// `:echohl`.
///
/// # Safety
/// `eap` must be valid.
pub unsafe fn ex_echohl(eap: *mut exarg_T) {
    unsafe { echo_hl_id.set(syn_name2id((*eap).arg)) }
}

/// The highlight group `:echohl` last named.
pub fn get_echo_hl_id() -> c_int {
    echo_hl_id.get()
}

/// `:execute`, `:echomsg` and `:echoerr` — the three that evaluate every
/// argument, join the results with spaces, and then do something with the
/// one string.
///
/// # Safety
/// `eap` must be valid.
pub unsafe fn ex_execute(eap: *mut exarg_T) {
    unsafe {
        let mut arg: *mut c_char = (*eap).arg;
        let mut rettv = UNSET_TV;
        let mut ret = OK;
        let mut ga = UNSET_GA;
        ga_init(&raw mut ga, 1, 80);

        if (*eap).skip != 0 {
            *emsg_skip.ptr() += 1;
        }
        while !ends_args(*arg) {
            ret = eval1_emsg(&raw mut arg, &raw mut rettv, eap);
            if ret == FAIL {
                break;
            }
            if (*eap).skip == 0 {
                // `:execute` coerces; the two message commands render, and
                // so own what they produce.
                let owned = (*eap).cmdidx != CMD_execute;
                let argstr: *const c_char = if !owned {
                    tv_get_string(&raw mut rettv)
                } else if rettv.v_type == VAR_STRING {
                    encode_tv2echo(&raw mut rettv, null_mut::<size_t>())
                } else {
                    encode_tv2string(&raw mut rettv, null_mut::<size_t>())
                };
                let len = strlen(argstr);
                ga_grow(&raw mut ga, len as c_int + 2);
                if ga.ga_len > 0 {
                    *(ga.ga_data as *mut c_char).offset(ga.ga_len as isize) = b' ' as c_char;
                    ga.ga_len += 1;
                }
                memcpy(
                    (ga.ga_data as *mut c_char).offset(ga.ga_len as isize) as *mut c_void,
                    argstr as *const c_void,
                    len + 1,
                );
                if owned {
                    xfree(argstr as *mut c_void);
                }
                ga.ga_len += len as c_int;
            }
            tv_clear(&raw mut rettv);
            arg = skipwhite(arg);
        }

        if ret != FAIL && !ga.ga_data.is_null() {
            if (*eap).cmdidx == CMD_echomsg {
                msg_ext_set_kind(c"echomsg".as_ptr());
                msg(ga.ga_data as *const c_char, echo_hl_id.get());
            } else if (*eap).cmdidx == CMD_echoerr {
                // `:echoerr` reports without counting as an error unless
                // something is already unwinding.
                let save_did_emsg = did_emsg.get();
                emsg_multiline(
                    ga.ga_data as *const c_char,
                    c"echoerr".as_ptr(),
                    HLF_E,
                    true,
                );
                if !force_abort.get() {
                    did_emsg.set(save_did_emsg);
                }
            } else if (*eap).cmdidx == CMD_execute {
                do_cmdline(
                    ga.ga_data as *mut c_char,
                    (*eap).ea_getline,
                    (*eap).cookie,
                    DOCMD_NOWAIT as c_int | DOCMD_VERBOSE as c_int,
                );
            }
        }

        ga_clear(&raw mut ga);
        if (*eap).skip != 0 {
            *emsg_skip.ptr() -= 1;
        }
        (*eap).nextcmd = check_nextcmd(arg);
    }
}

/// Which persistence a global variable's name asks for: `ALLCAPS` goes to
/// the shada file, `MixedCase` to a session file, anything else nowhere.
///
/// # Safety
/// `varname` must be NUL-terminated.
pub unsafe fn var_flavour(varname: *mut c_char) -> var_flavour_T {
    unsafe {
        if !(*varname >= b'A' as c_char && *varname <= b'Z' as c_char) {
            return VAR_FLAVOUR_DEFAULT;
        }
        let mut p = varname;
        loop {
            p = p.add(1);
            if *p == 0 {
                return VAR_FLAVOUR_SHADA;
            }
            if *p >= b'a' as c_char && *p <= b'z' as c_char {
                return VAR_FLAVOUR_SESSION;
            }
        }
    }
}

/// Set a global variable from outside any function, so that the current
/// function's scope cannot capture it.
///
/// # Safety
/// `name` must be NUL-terminated; `vartv`'s ownership moves here.
pub unsafe fn var_set_global(name: *const c_char, mut vartv: typval_T) {
    unsafe {
        let mut funccall_entry = funccal_entry_T {
            top_funccal: null_mut(),
            next: null_mut(),
        };
        save_funccal(&raw mut funccall_entry);
        set_var(name, strlen(name), &raw mut vartv, false);
        restore_funccal();
    }
}

/// The ":verbose" tail saying where something was last set.
///
/// # Safety
/// Called with a script context from an option or a variable.
pub unsafe fn last_set_msg(script_ctx: sctx_T) {
    unsafe {
        if script_ctx.sc_sid == 0 {
            return;
        }
        let mut should_free = false;
        let p = get_scriptname(script_ctx, &raw mut should_free);
        msg_ext_skip_verbose.set(true);
        verbose_enter();
        msg_puts(gettext(c"\n\tLast set from ".as_ptr()));
        msg_puts(p);
        if script_ctx.sc_lnum > 0 as linenr_T {
            msg_puts(gettext(line_msg.ptr().cast()));
            msg_outnum(script_ctx.sc_lnum as c_int);
        } else if script_is_lua(script_ctx.sc_sid) {
            msg_puts(gettext(c" (run Nvim with -V1 for more details)".as_ptr()));
        }
        if should_free {
            xfree(p as *mut c_void);
        }
        verbose_leave();
    }
}
