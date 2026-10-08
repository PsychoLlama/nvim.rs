//! `:echo`, `:echohl`, `:execute` and where a variable was last set.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::guard::Suppress;
use crate::semsg;
use crate::types::CmdIdx;
use core::ffi::{CStr, c_int};

use crate::eval::encode::{tv2echo_bytes, tv2string_bytes};
use crate::eval::typval::NumBuf;
use crate::eval::userfunc::CallStackAside;
use crate::eval::vars::{clear_local, set_var};
use crate::eval::{Cursor, echo_hl_id, eval1, eval1_emsg};
use crate::ex_docmd::{DoCmdOpts, do_cmdline_as};
use crate::ex_eval::aborting;
use crate::ex_eval::state::force_abort;
use crate::getchar::state::got_int;
use crate::highlight_group::syn_name2id;
use crate::message::state::{
    called_emsg, did_emsg, line_msg, msg_didout, msg_ext_skip_verbose, need_clr_eos,
};
use crate::message::{
    emsg_multiline_text, msg, msg_bytes as msg_bytes_out, msg_clr_eos, msg_end, msg_ext_set_append,
    msg_ext_set_kind, msg_multiline, msg_outnum, msg_sb_eol, msg_start, msg_str, msg_str_hl,
    verbose_enter, verbose_leave,
};
use crate::message_fmt::msg_bytes;
use crate::os::cshim::gettext;
use crate::runtime::{get_scriptname, script_is_lua};
use crate::types::String_0;
use crate::types::ui::kUIMessages;
use crate::types::{
    ExArg, LineNr, ScriptCtx, TypVal, VAR_FLAVOUR_DEFAULT, VAR_FLAVOUR_SESSION, VAR_FLAVOUR_SHADA,
    VAR_STRING, VarFlavour,
};
use crate::ui::ui_has;

/// A freshly declared typval.
const UNSET_TV: TypVal = TV_INITIAL_VALUE;

/// Does this byte end the `:echo` argument list?
fn ends_args(c: u8) -> bool {
    matches!(c, 0 | b'|' | b'\n')
}

/// `:echo` and `:echon`.
pub fn ex_echo(excmd: &mut ExArg) {
    let mut rettv = UNSET_TV;
    let mut atstart = true;
    let mut need_clear = true;
    let did_emsg_before = did_emsg.get();
    let called_emsg_before = called_emsg.get();
    let skip = excmd.skip;
    let _skipping = skip.then(Suppress::emsg_skip);

    let base = excmd.line.arg;
    let mut cursor = Cursor::new(excmd.line.rest_of(base));
    while !ends_args(cursor.byte()) && !got_int.get() {
        // The flag is set across the evaluation only: an expression
        // that writes to the screen itself must not have the rest of
        // the line cleared out from under it.
        need_clr_eos.set(true);
        let start = cursor.offset();
        if eval1(&mut cursor, &mut rettv, !skip).is_err() {
            if !aborting()
                && did_emsg.get() == did_emsg_before
                && called_emsg.get() == called_emsg_before
            {
                let start = msg_bytes(&cursor.text()[start..]);
                semsg!("E15: Invalid expression: \"{start}\"");
            }
            need_clr_eos.set(false);
            break;
        }
        need_clr_eos.set(false);

        if !skip {
            if atstart {
                atstart = false;
                msg_ext_set_append(excmd.cmdidx == CmdIdx::echon);
                msg_ext_set_kind(c"echo");
                if excmd.cmdidx == CmdIdx::echo {
                    if !msg_didout.get() {
                        msg_sb_eol();
                    }
                    msg_start();
                }
            } else if excmd.cmdidx == CmdIdx::echo {
                // `:echo` separates its arguments; `:echon` does not.
                msg_str_hl(c" ", echo_hl_id.get(), false);
            }
            let text = String_0::from_bytes(&tv2echo_bytes(&rettv));
            msg_multiline(text, echo_hl_id.get(), true, false, &mut need_clear);
        }
        clear_local(&mut rettv);
        cursor.skip_white();
    }

    let end = base + cursor.offset();
    excmd.line.next = excmd.line.check_next(end);
    msg_ext_set_append(false);

    if excmd.skip {
        return;
    }
    if ui_has(kUIMessages) && ends_args(excmd.line.byte_at(excmd.line.arg)) {
        // A bare `:echo` still has to produce an (empty) message.
        msg_bytes_out(b"", 0, false);
    } else if need_clear {
        msg_clr_eos();
    }
    if excmd.cmdidx == CmdIdx::echo {
        msg_end();
    }
}

/// `:echohl`.
pub fn ex_echohl(excmd: &mut ExArg) {
    // SAFETY: the caller's promise -- the argument is NUL-terminated.
    echo_hl_id.set(syn_name2id(excmd.line.cstr_from(excmd.line.arg)));
}

/// The highlight group `:echohl` last named.
pub(crate) fn get_echo_hl_id() -> c_int {
    echo_hl_id.get()
}

/// `:execute`, `:echomsg` and `:echoerr` — the three that evaluate every
/// argument, join the results with spaces, and then do something with the
/// one string.
pub fn ex_execute(excmd: &mut ExArg) {
    let mut numbuf = NumBuf::new();
    let mut rettv = UNSET_TV;
    let mut ret = Ok(());
    let mut text = Vec::<u8>::new();
    // Whether anything was appended at all: with every argument skipped
    // there is no message, which is not the same as an empty one.
    let mut built = false;
    let skip = excmd.skip;

    let _skipping = skip.then(Suppress::emsg_skip);
    let base = excmd.line.arg;
    let mut cursor = Cursor::new(excmd.line.rest_of(base));
    while !ends_args(cursor.byte()) {
        ret = eval1_emsg(&mut cursor, &mut rettv, !skip);
        if ret.is_err() {
            break;
        }
        if !skip {
            if built {
                text.push(b' ');
            }
            // `:execute` coerces; the two message commands render.
            if excmd.cmdidx == CmdIdx::execute {
                text.extend_from_slice(numbuf.string(&rettv).to_bytes());
            } else if rettv.v_type() == VAR_STRING {
                text.extend_from_slice(&tv2echo_bytes(&rettv));
            } else {
                text.extend_from_slice(&tv2string_bytes(&rettv));
            }
            built = true;
        }
        clear_local(&mut rettv);
        cursor.skip_white();
    }
    let end = base + cursor.offset();

    if ret.is_ok() && built {
        text.push(0);
        if excmd.cmdidx == CmdIdx::echomsg {
            msg_ext_set_kind(c"echomsg");
            msg(cstr::in_bytes(&text), echo_hl_id.get());
        } else if excmd.cmdidx == CmdIdx::echoerr {
            // `:echoerr` reports without counting as an error unless
            // something is already unwinding.
            let save_did_emsg = did_emsg.get();
            emsg_multiline_text(cstr::in_bytes(&text), c"echoerr");
            if !force_abort.get() {
                did_emsg.set(save_did_emsg);
            }
        } else if excmd.cmdidx == CmdIdx::execute {
            let _ = do_cmdline_as(excmd, &mut text, DoCmdOpts::NOWAIT | DoCmdOpts::VERBOSE);
        }
    }
    excmd.line.next = excmd.line.check_next(end);
}

/// Which persistence a global variable's name asks for: `ALLCAPS` goes to
/// the shada file, `MixedCase` to a session file, anything else nowhere.
pub fn var_flavour(name: &[u8]) -> VarFlavour {
    match name.split_first() {
        Some((b'A'..=b'Z', rest)) if rest.iter().any(u8::is_ascii_lowercase) => VAR_FLAVOUR_SESSION,
        Some((b'A'..=b'Z', _)) => VAR_FLAVOUR_SHADA,
        _ => VAR_FLAVOUR_DEFAULT,
    }
}

/// Set a global variable from outside any function, so that the current
/// function's scope cannot capture it. The value moves into the variable.
pub fn var_set_global(name: &CStr, mut vartv: TypVal) {
    let call_stack_aside = CallStackAside::new();
    set_var(name.to_bytes(), &mut vartv, false);
    drop(call_stack_aside);
}

/// The ":verbose" tail saying where something was last set.
pub fn last_set_msg(script_ctx: ScriptCtx) {
    if script_ctx.sc_sid == 0 {
        return;
    }
    let p = get_scriptname(script_ctx, true);
    msg_ext_skip_verbose.set(true);
    verbose_enter();
    msg_str(gettext(c"\n\tLast set from "));
    msg_str(&p);
    if script_ctx.sc_lnum > 0 as LineNr {
        msg_str(gettext(line_msg));
        msg_outnum(script_ctx.sc_lnum as c_int);
    // SAFETY: the caller's promise about `script_ctx`.
    } else if script_is_lua(script_ctx.sc_sid) {
        // SAFETY: the hint is a NUL-terminated literal.
        msg_str(gettext(c" (run Nvim with -V1 for more details)"));
    }
    verbose_leave();
}
