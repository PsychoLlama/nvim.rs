//! `input()`, `inputsecret()` and the `:normal`-style script prompts.
//!
//! [`get_user_input`] is the shared implementation behind the `input*()`
//! family: it takes the prompt, default and completion out of the argument
//! (or the option dict), and drives a command line through
//! [`super::enter::getcmdline_prompt`].

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::cstr;
use crate::eval::typval::{NumBuf, list_iter};
use crate::memory::ThinCString;
use crate::memory::handoff::owned_cstr;
use crate::types::{ExArgt, ExpandContext, NUL, VAR_DICT};

/// Read the script body of a command that takes either `:command script` or a
/// heredoc:
///
/// ```text
/// :command << endmarker
///   script
/// endmarker
/// ```
///
/// `lenp` receives the length without the trailing NUL (zero while skipping).
/// Answers an allocated string, or NULL when skipping and on error; it shows
/// no messages of its own.
///
/// # Safety
///
/// `excmd` must point at the command's `ExArg`. `lenp` must point at a
/// writable `size_t` the caller owns.
pub unsafe fn script_get(excmd: &mut ExArg, lenp: *mut size_t) -> *mut ::core::ffi::c_char {
    let mut numbuf = NumBuf::new();
    let mut cmd = excmd.arg_ptr();
    if unsafe { *cmd.offset(0) } as ::core::ffi::c_int != '<' as ::core::ffi::c_int
        || unsafe { *cmd.offset(1) } as ::core::ffi::c_int != '<' as ::core::ffi::c_int
        || excmd.ea_getline.is_none()
    {
        unsafe { *lenp = excmd.line.arg().len() };
        if excmd.skip {
            return ::core::ptr::null_mut();
        }
        return unsafe { xmemdupz(excmd.arg_ptr() as *const ::core::ffi::c_void, *lenp) }
            as *mut ::core::ffi::c_char;
    }
    cmd = unsafe { cmd.offset(2) };

    let at = excmd.line.offset_of(cmd);
    let Some(held) = heredoc_get(excmd, at, true) else {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    };
    let l = held.as_ptr();

    let skip = excmd.skip;
    let mut text = Vec::<u8>::new();
    for li in list_iter(unsafe { l.as_ref() }) {
        if !skip {
            text.extend_from_slice(numbuf.bytes(&li.li_tv));
            text.push(b'\n');
        }
    }

    // The length is the text without the terminator `owned_cstr` adds.
    unsafe { *lenp = text.len() as size_t };
    drop(held);
    // A skipped here-document answered a garray that was never opened, and
    // so a null pointer.
    if skip {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    owned_cstr(text)
}

/// The `cancelreturn` entry of an `{opts}` dictionary, if it has one.
fn opts_cancelreturn(opts: &TypVal) -> Option<&TypVal> {
    opts.dict_ref()
        .and_then(|dict| dict.find(b"cancelreturn"))
        .map(|item| &item.di_tv)
}

/// Drive one `input()`-family prompt and leave its answer in `result`.
///
/// Shared by `input()`, `inputsecret()` and `inputdialog()`.  `args` is
/// either a single `{opts}` dict or up to three positional arguments, whose
/// third means completion for `input()` and the cancel value for
/// `inputdialog()`.
pub fn get_user_input(args: &[TypVal], result: &mut TypVal, inputdialog: bool, secret: bool) {
    result.write_string(None);

    if cmdpreview.get() {
        return;
    }

    let prompt: *const ::core::ffi::c_char;
    let mut defstr: *const ::core::ffi::c_char = c"".as_ptr();
    // A copy of the positional cancel value; `tv_copy` puts another copy
    // in the answer, and this one is released with the frame.
    let mut cancelreturn: Option<TypVal> = None;
    // Whether `{opts}` has a `cancelreturn`. The value is looked up again
    // once the prompt is done: the prompt runs user code, which may change
    // or remove the entry.
    let mut cancelreturn_in_opts = false;
    let mut xp_name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut input_callback = Callback::None;
    let mut prompt_buf = NumBuf::new();
    let mut defstr_buf = NumBuf::new();
    let mut cancelreturn_buf = NumBuf::new();
    let mut xp_name_buf = NumBuf::new();
    // Its *address* is the "argument absent" answer below, so it has to be
    // a distinct object from the `""` literal `defstr` starts as.
    let def_block = [0u8; 1];
    let def = ::core::ffi::CStr::from_bytes_with_nul(&def_block).expect("a lone terminator");

    if args[0].v_type() == VAR_DICT {
        if args.len() > 1 {
            emsg(gettext(c"E5050: {opts} must be the only argument"));
            return;
        }
        let dict = args[0].dict_or_null();
        // C's `S_LEN(key)`: the key pointer and its length, spelled once.
        let dict_str =
            |key: &::core::ffi::CStr, numbuf: &mut NumBuf, def: Option<&::core::ffi::CStr>| {
                // SAFETY: the argument's own dictionary.
                dict_get_string_buf_chk(unsafe { dict.as_ref() }, key.to_bytes(), numbuf, def)
                    .map_or(::core::ptr::null(), ::core::ffi::CStr::as_ptr)
            };

        prompt = dict_str(c"prompt", &mut prompt_buf, Some(c""));
        if prompt.is_null() {
            return;
        }
        defstr = dict_str(c"default", &mut defstr_buf, Some(c""));
        if defstr.is_null() {
            return;
        }
        // `v:_null_dict` is a `VAR_DICT` holding nothing, so the lookup has
        // to tolerate it -- `input(v:_null_dict)` reaches here.
        cancelreturn_in_opts = opts_cancelreturn(&args[0]).is_some();
        xp_name = dict_str(c"completion", &mut xp_name_buf, Some(def));
        if xp_name.is_null() {
            // error
            return;
        }
        if xp_name == def.as_ptr() {
            // key absent: default to NULL
            xp_name = ::core::ptr::null::<::core::ffi::c_char>();
        }
        // SAFETY: the argument's own dictionary, and this frame's callback.
        if !unsafe { dict_get_callback(dict.as_mut(), b"highlight", &mut input_callback) } {
            return;
        }
    } else {
        let Some(text) = prompt_buf.string_chk(&args[0]) else {
            return;
        };
        prompt = text.as_ptr();
        if args.len() > 1 {
            let Some(text) = defstr_buf.string_chk(&args[1]) else {
                return;
            };
            defstr = text.as_ptr();
            if args.len() > 2 {
                let Some(strarg2) = cancelreturn_buf.string_chk(&args[2]) else {
                    return;
                };
                if inputdialog {
                    cancelreturn = Some(TypVal::string_from(strarg2.to_bytes()));
                } else {
                    xp_name = strarg2.as_ptr();
                }
            }
        }
    }

    let mut xp_type = ExpandContext::Nothing;
    let mut xp_arg = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !xp_name.is_null() {
        // input() with a third argument: completion
        let xp_namelen = unsafe { cstr::bytes_at(xp_name) }.len() as ::core::ffi::c_int;
        let mut argt = ExArgt::NONE;
        if unsafe { parse_compl_arg(xp_name, xp_namelen, &mut xp_type, &mut argt, &mut xp_arg) }
            .is_err()
        {
            return;
        }
    }

    // Only the part of the message after the last NL is the command
    // line's prompt, unless the command line is externalised.
    let mut p = prompt;
    if !ui_has(kUICmdline) {
        let lastnl = unsafe { strrchr(prompt, '\n' as ::core::ffi::c_int) };
        if !lastnl.is_null() {
            p = unsafe { lastnl.offset(1) };
            msg_start();
            msg_clr_eos();
            // SAFETY: `p` was found inside the prompt, so the span is
            // readable.
            let head = unsafe { cstr::slice_at(prompt, p.offset_from(prompt).cast_unsigned()) };
            msg_bytes(head, get_echo_hl_id(), false);
            msg_didout.set(false);
            msg_starthere();
        }
    }
    cmdline_row.set(msg_row.get());

    unsafe { stuff_readbuf_one_line(defstr) };

    let save_ex_normal_busy = ex_normal_busy.get();
    ex_normal_busy.set(0);
    // SAFETY: `getcmdline_prompt` answers an allocation of its own, or
    // NULL, which the answer takes over.
    result.write_string(unsafe {
        ThinCString::from_raw(getcmdline_prompt(
            if secret {
                NUL
            } else {
                '@' as ::core::ffi::c_int
            },
            p,
            get_echo_hl_id(),
            xp_type,
            xp_arg,
            // The prompt takes the callback over, and releases it when it
            // returns.
            input_callback,
            false,
            ::core::ptr::null_mut::<bool>(),
        ))
    });
    ex_normal_busy.set(save_ex_normal_busy);

    if result.string_ref().is_none() {
        if let Some(value) = &cancelreturn {
            tv_copy(value, result);
        } else if cancelreturn_in_opts && let Some(value) = opts_cancelreturn(&args[0]) {
            tv_copy(value, result);
        }
    }

    unsafe { xfree(xp_arg as *mut ::core::ffi::c_void) };
    // Since the user typed this, no need to wait for return.
    need_wait_return.set(false);
    msg_didout.set(false);
}
