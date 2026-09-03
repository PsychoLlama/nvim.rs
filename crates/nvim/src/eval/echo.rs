//! `:echo`, `:echohl`, `:execute` and where a variable was last set.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::guard::Suppress;
use crate::semsg;
use crate::types::CmdIdx;
use crate::winlayer::Ea;
use core::ffi::{c_char, c_int, c_void};
use core::ptr::null_mut;

use crate::api::private::helpers::cstr_as_string;
use crate::charset::skipwhite;
use crate::eval::encode::{encode_tv2echo, encode_tv2string};
use crate::eval::typval::NumBuf;
use crate::eval::userfunc::{restore_funccal, save_funccal};
use crate::eval::vars::clear_local;
use crate::eval::vars::set_var;
use crate::eval::{clear_evalarg, echo_hl_id, eval1, eval1_emsg, fill_evalarg_from_eap};
use crate::ex_docmd::{DoCmdOpts, check_nextcmd, do_cmdline};
use crate::ex_eval::aborting;
use crate::highlight_group::{HLF_E, syn_name2id};
use crate::main::{
    called_emsg, did_emsg, force_abort, got_int, line_msg, msg_didout, msg_ext_skip_verbose,
    need_clr_eos,
};
use crate::memory::xfree;
use crate::message::{
    emsg_multiline, msg_clr_eos, msg_end, msg_ext_set_append, msg_ext_set_kind, msg_multiline,
    msg_outnum, msg_ptr, msg_puts, msg_puts_hl, msg_puts_len, msg_sb_eol, msg_start, verbose_enter,
    verbose_leave,
};
use crate::message_fmt::c_str;
use crate::os::cshim::gettext;
use crate::runtime::{get_scriptname, script_is_lua};
use crate::types::ui::kUIMessages;
use crate::types::{
    NUL, VAR_FLAVOUR_DEFAULT, VAR_FLAVOUR_SESSION, VAR_FLAVOUR_SHADA, VAR_STRING, VAR_UNKNOWN,
    VarLock, evalarg_T, exarg_T, funccal_entry_T, linenr_T, ptrdiff_t, sctx_T, size_t, typval_T,
    typval_vval_union, var_flavour_T,
};
use crate::ui::ui_has;

/// A freshly declared typval.
const UNSET_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VarLock::Unlocked,
    vval: typval_vval_union { v_number: 0 },
};

/// A freshly declared `evalarg_T`.
const UNSET_EVALARG: evalarg_T = evalarg_T {
    eval_flags: 0,
    eval_getline: None,
    eval_cookie: null_mut(),
    eval_tofree: null_mut(),
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
    // SAFETY: the caller's promise -- the `exarg_T` outlives the command,
    // which the `do_cmdline` frame that owns it discharges.
    let mut eap = unsafe { Ea::new(eap) };
    let mut arg: *mut c_char = eap.arg;
    let mut rettv = UNSET_TV;
    let mut atstart = true;
    let mut need_clear = true;
    let did_emsg_before = did_emsg.get();
    let called_emsg_before = called_emsg.get();

    let mut evalarg = UNSET_EVALARG;
    let (ea, skip) = (eap.raw(), eap.skip != 0);
    // SAFETY: `evalarg` is this frame's and `ea` the caller's `exarg_T`.
    unsafe { fill_evalarg_from_eap(&raw mut evalarg, ea, skip) };
    let _skipping = skip.then(Suppress::emsg_skip);

    // SAFETY: `arg` walks the command line, which is NUL-terminated.
    while !ends_args(unsafe { *arg }) && !got_int.get() {
        // The flag is set across the evaluation only: an expression
        // that writes to the screen itself must not have the rest of
        // the line cleared out from under it.
        need_clr_eos.set(true);
        let start = arg;
        // SAFETY: `arg`, `rettv` and `evalarg` are all this frame's.
        if unsafe { eval1(&raw mut arg, &raw mut rettv, &raw mut evalarg) }.is_err() {
            if !aborting()
                && did_emsg.get() == did_emsg_before
                && called_emsg.get() == called_emsg_before
            {
                // SAFETY: the format takes one string, and `start` is a // NUL-terminated tail of the command line.
                let start = unsafe { c_str(start) };
                semsg!("E15: Invalid expression: \"{start}\"");
            }
            need_clr_eos.set(false);
            break;
        }
        need_clr_eos.set(false);

        if eap.skip == 0 {
            if atstart {
                atstart = false;
                unsafe { msg_ext_set_append(eap.cmdidx == CmdIdx::echon) };
                // SAFETY: the kind is a NUL-terminated literal.
                unsafe { msg_ext_set_kind(c"echo".as_ptr()) };
                if eap.cmdidx == CmdIdx::echo {
                    if !msg_didout.get() {
                        unsafe { msg_sb_eol() };
                    }
                    unsafe { msg_start() };
                }
            } else if eap.cmdidx == CmdIdx::echo {
                // `:echo` separates its arguments; `:echon` does not.
                // SAFETY: the separator is a NUL-terminated literal.
                unsafe { msg_puts_hl(c" ".as_ptr(), echo_hl_id.get(), false) };
            }
            // SAFETY: `rettv` is this frame's.
            let tofree = unsafe { encode_tv2echo(&raw mut rettv, null_mut::<size_t>()) };
            let (hl, clear) = (echo_hl_id.get(), &raw mut need_clear);
            // SAFETY: `tofree` is the NUL-terminated rendering just made and
            // `clear` names this frame's flag.
            unsafe { msg_multiline(cstr_as_string(tofree), hl, true, false, clear) };
            // SAFETY: nothing else owns the rendering.
            unsafe { xfree(tofree as *mut c_void) };
        }
        // SAFETY: `rettv` is this frame's.
        clear_local(&mut rettv);
        // SAFETY: `arg` walks a NUL-terminated command line.
        arg = unsafe { skipwhite(arg) };
    }

    // SAFETY: `arg` is the tail of the command line.
    eap.nextcmd = unsafe { check_nextcmd(arg) };
    // SAFETY: `evalarg` is this frame's and `ea` the caller's `exarg_T`.
    unsafe { clear_evalarg(&raw mut evalarg, ea) };
    unsafe { msg_ext_set_append(false) };

    if eap.skip != 0 {
        return;
    }
    // SAFETY: the command's argument is NUL-terminated.
    if ui_has(kUIMessages) && ends_args(unsafe { *eap.arg }) {
        // A bare `:echo` still has to produce an (empty) message.
        // SAFETY: the literal is NUL-terminated and zero bytes long.
        unsafe { msg_puts_len(c"".as_ptr(), 0 as ptrdiff_t, 0, false) };
    } else if need_clear {
        unsafe { msg_clr_eos() };
    }
    if eap.cmdidx == CmdIdx::echo {
        unsafe { msg_end() };
    }
}

/// `:echohl`.
///
/// # Safety
/// `eap` must be valid.
pub unsafe fn ex_echohl(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- the argument is NUL-terminated.
    echo_hl_id.set(unsafe { syn_name2id((*eap).arg) });
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
    let mut numbuf = NumBuf::new();
    // SAFETY: the caller's promise -- the `exarg_T` outlives the command.
    let mut eap = unsafe { Ea::new(eap) };
    let mut arg: *mut c_char = eap.arg;
    let mut rettv = UNSET_TV;
    let mut ret = Ok(());
    let mut text = Vec::<u8>::new();
    // Whether anything was appended at all: with every argument skipped
    // there is no message, which is not the same as an empty one.
    let mut built = false;

    let _skipping = (eap.skip != 0).then(Suppress::emsg_skip);
    // SAFETY: `arg` walks the command line, which is NUL-terminated.
    while !ends_args(unsafe { *arg }) {
        // SAFETY: `arg` and `rettv` are this frame's, `eap` the caller's.
        ret = unsafe { eval1_emsg(&raw mut arg, &raw mut rettv, eap.raw()) };
        if ret.is_err() {
            break;
        }
        if eap.skip == 0 {
            // `:execute` coerces; the two message commands render, and
            // so own what they produce.
            let owned = eap.cmdidx != CmdIdx::execute;
            // SAFETY: `rettv` is this frame's, holding the value just
            // evaluated; each of the three renderings is NUL-terminated.
            let argstr: *const c_char = if !owned {
                unsafe { numbuf.string(&raw mut rettv) }
            } else if rettv.v_type == VAR_STRING {
                unsafe { encode_tv2echo(&raw mut rettv, null_mut::<size_t>()) }
            } else {
                unsafe { encode_tv2string(&raw mut rettv, null_mut::<size_t>()) }
            };
            if built {
                text.push(b' ');
            }
            // SAFETY: `argstr` is NUL-terminated.
            text.extend_from_slice(unsafe { cstr::bytes_at(argstr) });
            built = true;
            if owned {
                // SAFETY: the two encoders hand back an owned string.
                unsafe { xfree(argstr as *mut c_void) };
            }
        }
        // SAFETY: `rettv` is this frame's.
        clear_local(&mut rettv);
        // SAFETY: `arg` walks a NUL-terminated command line.
        arg = unsafe { skipwhite(arg) };
    }

    if ret.is_ok() && built {
        text.push(0);
        let line = text.as_mut_ptr().cast::<c_char>();
        if eap.cmdidx == CmdIdx::echomsg {
            // SAFETY: the kind is a NUL-terminated literal.
            unsafe { msg_ext_set_kind(c"echomsg".as_ptr()) };
            // SAFETY: `line` is the NUL-terminated message built above.
            unsafe { msg_ptr(line, echo_hl_id.get()) };
        } else if eap.cmdidx == CmdIdx::echoerr {
            // `:echoerr` reports without counting as an error unless
            // something is already unwinding.
            let save_did_emsg = did_emsg.get();
            // SAFETY: `line` is the NUL-terminated message built above, and
            // the kind is a literal.
            unsafe { emsg_multiline(line, c"echoerr".as_ptr(), HLF_E, true) };
            if !force_abort.get() {
                did_emsg.set(save_did_emsg);
            }
        } else if eap.cmdidx == CmdIdx::execute {
            let (getline, cookie) = (eap.ea_getline, eap.cookie);
            let opts = DoCmdOpts::NOWAIT | DoCmdOpts::VERBOSE;
            // SAFETY: `line` is the NUL-terminated command built above, and
            // the getline pair is the caller's own.
            let _ = unsafe { do_cmdline(line, getline, cookie, opts) };
        }
    }
    // SAFETY: `arg` is the tail of the command line.
    eap.nextcmd = unsafe { check_nextcmd(arg) };
}

/// Which persistence a global variable's name asks for: `ALLCAPS` goes to
/// the shada file, `MixedCase` to a session file, anything else nowhere.
///
/// # Safety
/// `varname` must be NUL-terminated.
pub unsafe fn var_flavour(varname: *mut c_char) -> var_flavour_T {
    // SAFETY: the caller's promise -- `varname` is NUL-terminated, so its
    // first byte is readable.
    let first = unsafe { *varname };
    if !(first >= b'A' as c_char && first <= b'Z' as c_char) {
        return VAR_FLAVOUR_DEFAULT;
    }
    let mut p = varname;
    loop {
        // SAFETY: the byte before this one was not the terminator, so this
        // one is still inside the string.
        p = unsafe { p.add(1) };
        let c = unsafe { *p };
        if c == 0 {
            return VAR_FLAVOUR_SHADA;
        }
        if c >= b'a' as c_char && c <= b'z' as c_char {
            return VAR_FLAVOUR_SESSION;
        }
    }
}

/// Set a global variable from outside any function, so that the current
/// function's scope cannot capture it.
///
/// # Safety
/// `name` must be NUL-terminated; `vartv`'s ownership moves here.
pub unsafe fn var_set_global(name: *const c_char, mut vartv: typval_T) {
    let mut funccall_entry = funccal_entry_T {
        top_funccal: null_mut(),
        next: null_mut(),
    };
    // SAFETY: `funccall_entry` is this frame's and outlives the save.
    unsafe { save_funccal(&raw mut funccall_entry) };
    // SAFETY: the caller's promise about `name`; `vartv` is this frame's
    // copy, whose ownership moves into the variable.
    unsafe { set_var(name, cstr::bytes_at(name).len(), &raw mut vartv, false) };
    // SAFETY: this undoes the save above.
    unsafe { restore_funccal() };
}

/// The ":verbose" tail saying where something was last set.
///
/// # Safety
/// Called with a script context from an option or a variable.
pub unsafe fn last_set_msg(script_ctx: sctx_T) {
    if script_ctx.sc_sid == 0 {
        return;
    }
    // SAFETY: the caller's promise -- `script_ctx` names a loaded script.
    let p = unsafe { get_scriptname(script_ctx, true) };
    msg_ext_skip_verbose.set(true);
    unsafe { verbose_enter() };
    // SAFETY: the text is a NUL-terminated literal.
    unsafe { msg_puts(gettext(c"\n\tLast set from ").as_ptr()) };
    // SAFETY: the `CString` `p` outlives the call.
    unsafe { msg_puts(p.as_ptr()) };
    if script_ctx.sc_lnum > 0 as linenr_T {
        // SAFETY: `line_msg` is a shared NUL-terminated message.
        unsafe { msg_puts(gettext(line_msg).as_ptr()) };
        // SAFETY: the number is rendered into the message area.
        unsafe { msg_outnum(script_ctx.sc_lnum as c_int) };
    // SAFETY: the caller's promise about `script_ctx`.
    } else if unsafe { script_is_lua(script_ctx.sc_sid) } {
        // SAFETY: the hint is a NUL-terminated literal.
        unsafe { msg_puts(gettext(c" (run Nvim with -V1 for more details)").as_ptr()) };
    }
    unsafe { verbose_leave() };
}
