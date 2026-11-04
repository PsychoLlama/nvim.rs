//! The Vimscript face: `getcmdline()`, `setcmdline()`, `getcmdpos()`, …
//!
//! All of them read or write the *current* `ccline` through
//! [`get_ccline_ptr`], which is what makes them answer nothing outside a
//! command line and answer the enclosing one from inside `<C-r>=`.
//!
//! The `f_*` rows are the `VimLFunc` function pointers in
//! `eval/funcs/table`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::cstr;
use crate::eval::list::string_tv;
use crate::eval::typval::NumBuf;
use crate::getchar::Remap;
use crate::keycodes::KE_WILD;
use crate::memory::{ThinCString, XString};
use crate::narrow::number_as_int;
use crate::types::ExpandContext;

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
pub(crate) fn get_cmdline_type() -> u8 {
    let Some(p) = get_ccline_ptr() else {
        return 0;
    };
    if p.cmdfirstc == 0 {
        // No first character: `input()` reports '@', a `:insert` style
        // line-getter '-'.
        return if p.input_fn { b'@' } else { b'-' };
    }
    p.cmdfirstc
}

/// A copy of the current command line, up to its first NUL; `None` when
/// there is none or the line is obscured (`inputsecret()`).
pub(crate) fn get_cmdline_str() -> Option<ThinCString> {
    if cmdline_star.get() > 0 {
        return None;
    }
    let p = get_ccline_ptr()?;
    Some(ThinCString::from_bytes(cstr::until_nul(p.bytes())))
}

/// The completion state of the current command line, computed on demand:
/// the `Expand` and the context it resolved to.
///
/// When nothing has asked yet the context is `ExpandContext::Nothing`, so
/// `set_expand_context` runs and the field is then put *back* to
/// `ExpandContext::Nothing` — the real completion has to recompute it at the wildcard
/// key.  Hence the context is returned rather than left to the caller to
/// re-read: after the restore the field no longer holds it.
///
/// `None` means there is nothing to report: no command line, an obscured one
/// (`inputsecret()`), or `ExpandContext::Unsuccessful`.
fn cmdline_completion_state<R>(f: impl FnOnce(Cc, &Expand, ExpandContext) -> R) -> Option<R> {
    if cmdline_star.get() > 0 {
        return None;
    }
    let cc = get_ccline_ptr()?;
    cc.with_xpc(|xpc| {
        let xpc = xpc?;
        let mut xp_context = xpc.context;
        if xp_context == ExpandContext::Nothing {
            set_expand_context(xpc);
            xp_context = xpc.context;
            xpc.context = ExpandContext::Nothing;
        }
        if xp_context == ExpandContext::Unsuccessful {
            return None;
        }
        Some(f(cc, xpc, xp_context))
    })
}

/// `getcmdcomplpat()` function: the pattern completion would expand.
pub fn f_getcmdcomplpat(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // The pattern is read against the command line as it stands, which is
    // what upstream's `xp_pattern` pointed into: after a match was
    // inserted, the match.
    let pattern = cmdline_completion_state(|cc, xpc, _| {
        xpc.has_pattern.then(|| {
            cc.text_bytes()
                .get(xpc.pattern..)
                .unwrap_or_default()
                .to_vec()
        })
    })
    .flatten();
    *result = match pattern {
        Some(pattern) => string_tv(&pattern),
        None => TypVal::string(None),
    };
}

/// `getcmdcompltype()` function: the completion type's name.
pub fn f_getcmdcompltype(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let name = cmdline_completion_state(|_, xpc, xp_context| {
        cmdcomplete_type_to_str(xp_context, xpc.arg.as_ref().map(XString::as_cstr))
    })
    .flatten();
    *result = match name {
        Some(name) => string_tv(&name),
        None => TypVal::string(None),
    };
}

/// `getcmdline()` function.
pub fn f_getcmdline(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_string(get_cmdline_str());
}

/// `getcmdpos()` function.
pub fn f_getcmdpos(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(get_ccline_ptr().map_or(0, |p| VarNumber::from(p.cmdpos + 1)));
}

/// `getcmdprompt()` function.
pub fn f_getcmdprompt(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_string(get_ccline_ptr().and_then(|p| {
        p.cmdprompt
            .as_ref()
            .map(|prompt| ThinCString::from_cstr(prompt.as_cstr()))
    }));
}

/// `getcmdscreenpos()` function.
pub fn f_getcmdscreenpos(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(get_ccline_ptr().map_or(0, |p| VarNumber::from(p.cmdspos + 1)));
}

/// `getcmdtype()` function.
pub fn f_getcmdtype(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // One character, NUL outside a command line -- kept byte-for-byte, so
    // that answer reads as the empty string.
    let kind = get_cmdline_type();
    result.write_string(Some(ThinCString::from_bytes(&[kind])));
}

/// Replace the command line with `text` and put the cursor at `pos`.
///
/// A negative or out-of-range `pos` means the end of the line.  Answers
/// whether there was a command line to set.
pub(crate) fn set_cmdline_str(text: &[u8], pos: ::core::ffi::c_int) -> bool {
    let Some(mut p) = get_ccline_ptr() else {
        return false;
    };

    // `p` is not always `ccline`: inside `<C-r>=` it is the command line
    // one level out. C resized `ccline` here whichever line it then wrote
    // to, and overran the other one -- see the upstream note on
    // `set_cmdline_str`.
    p.set_text(text);

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

    true
}

/// Remember `pos` as the byte position to put the cursor at, zero-based.
///
/// It is not applied here but after `CTRL-\ e` or `CTRL-R =` has finished
/// changing the command line.  Answers whether there was a command line.
pub(crate) fn set_cmdline_pos(pos: ::core::ffi::c_int) -> bool {
    if get_ccline_ptr().is_none() {
        return false;
    }
    new_cmdpos.set(pos.max(0));
    true
}

/// `setcmdline()` function.
pub fn f_setcmdline(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    if tv_check_for_string_arg(args, 0).is_err() || tv_check_for_opt_number_arg(args, 1).is_err() {
        return;
    }

    let mut pos = -1;
    if args.len() > 1 {
        let Ok(given) = tv_get_number_chk(&args[1]) else {
            return;
        };
        pos = number_as_int(given) - 1;
        if pos < 0 {
            emsg(gettext(e_positive));
            return;
        }
    }

    // tv_get_string() so that a NULL string reads as an empty one.
    // 0 when it was set, 1 when there is no command line.
    let set = set_cmdline_str(numbuf.string(&args[0]).to_bytes(), pos);
    result.write_number(VarNumber::from(!set));
}

/// `setcmdpos()` function.
pub fn f_setcmdpos(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let pos = number_as_int(tv_get_number(&args[0])) - 1;
    if pos >= 0 {
        result.write_number(VarNumber::from(!set_cmdline_pos(pos)));
    }
}

/// The first character of the current command line (`:`, `/`, `?`, …).
pub fn get_cmdline_firstc() -> u8 {
    Cc::current().cmdfirstc
}

/// `wildtrigger()` function: ask the key loop to complete, as if `'wildchar'`
/// had been typed.
pub fn f_wildtrigger(_args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    if State.get() & MODE_CMDLINE == 0
        || char_avail()
        || wild_menu_showing.get() != 0
        || cmdline_pum_active()
    {
        return;
    }

    if matches!(get_cmdline_type(), b':' | b'/' | b'?') {
        // K_WILD as a single special key, pushed into the typeahead.
        let key_string = [K_SPECIAL, KS_EXTRA, KE_WILD];
        let _ = ins_typebuf(&key_string, Remap::No, 0, true, false);
    }
}
