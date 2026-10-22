//! `input()`, `inputsecret()` and the `:normal`-style script prompts.
//!
//! [`get_user_input`] is the shared implementation behind the `input*()`
//! family: it takes the prompt, default and completion out of the argument
//! (or the option dict), and drives a command line through
//! [`super::enter::getcmdline_prompt`].

#![forbid(unsafe_code)]

use super::*;
use crate::cstr;
use crate::eval::typval::NumBuf;
use crate::memory::ThinCString;
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
/// Answers the script, or `None` when skipping and on error; it shows no
/// messages of its own.
pub fn script_get(excmd: &mut ExArg) -> Option<XString> {
    let arg = excmd.line.arg();
    if !arg.starts_with(b"<<") || excmd.source.is_none() {
        return (!excmd.skip).then(|| XString::from_bytes(arg));
    }

    let at = excmd.line.arg + 2;
    let held = heredoc_get(excmd, at, true)?;
    if excmd.skip {
        // A skipped here-document is consumed, and answers nothing.
        return None;
    }
    let mut numbuf = NumBuf::new();
    let mut text = Vec::<u8>::new();
    for li in held.items() {
        text.extend_from_slice(numbuf.bytes(&li.li_tv));
        text.push(b'\n');
    }
    Some(XString::from_vec(text))
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

    let prompt: &::core::ffi::CStr;
    let mut defstr: &::core::ffi::CStr = c"";
    // A copy of the positional cancel value; `tv_copy` puts another copy
    // in the answer, and this one is released with the frame.
    let mut cancelreturn: Option<TypVal> = None;
    // Whether `{opts}` has a `cancelreturn`. The value is looked up again
    // once the prompt is done: the prompt runs user code, which may change
    // or remove the entry.
    let mut cancelreturn_in_opts = false;
    let mut xp_name: Option<&::core::ffi::CStr> = None;
    let mut input_callback = Callback::None;
    let mut prompt_buf = NumBuf::new();
    let mut defstr_buf = NumBuf::new();
    let mut cancelreturn_buf = NumBuf::new();
    let mut xp_name_buf = NumBuf::new();

    if args[0].v_type() == VAR_DICT {
        if args.len() > 1 {
            emsg(gettext(c"E5050: {opts} must be the only argument"));
            return;
        }
        let dict = args[0].dict_ref();

        let Some(text) = dict_get_string_buf_chk(dict, b"prompt", &mut prompt_buf, Some(c""))
        else {
            return;
        };
        prompt = text;
        let Some(text) = dict_get_string_buf_chk(dict, b"default", &mut defstr_buf, Some(c""))
        else {
            return;
        };
        defstr = text;
        // `v:_null_dict` is a `VAR_DICT` holding nothing, so the lookup has
        // to tolerate it -- `input(v:_null_dict)` reaches here.
        cancelreturn_in_opts = opts_cancelreturn(&args[0]).is_some();
        // A key that is absent leaves the completion off; one whose value
        // is not a string is an error.
        if dict.and_then(|d| d.find(b"completion")).is_some() {
            let Some(text) = dict_get_string_buf_chk(dict, b"completion", &mut xp_name_buf, None)
            else {
                return;
            };
            xp_name = Some(text);
        }
        if !dict_get_callback(args[0].dict_shared(), b"highlight", &mut input_callback) {
            return;
        }
    } else {
        let Some(text) = prompt_buf.string_chk(&args[0]) else {
            return;
        };
        prompt = text;
        if args.len() > 1 {
            let Some(text) = defstr_buf.string_chk(&args[1]) else {
                return;
            };
            defstr = text;
            if args.len() > 2 {
                let Some(strarg2) = cancelreturn_buf.string_chk(&args[2]) else {
                    return;
                };
                if inputdialog {
                    cancelreturn = Some(TypVal::string_from(strarg2.to_bytes()));
                } else {
                    xp_name = Some(strarg2);
                }
            }
        }
    }

    let mut xp_type = ExpandContext::Nothing;
    let mut xp_arg = None;
    if let Some(name) = xp_name {
        // input() with a third argument: completion
        let name = name.to_bytes();
        let mut argt = ExArgt::NONE;
        if parse_compl_arg(name, name, &mut xp_type, &mut argt, &mut xp_arg).is_err() {
            return;
        }
    }

    // Only the part of the message after the last NL is the command
    // line's prompt, unless the command line is externalised.
    let mut p = prompt;
    if !ui_has(kUICmdline) {
        let text = prompt.to_bytes();
        if let Some(lastnl) = text.iter().rposition(|&b| b == b'\n') {
            p = cstr::suffix(prompt, lastnl + 1);
            msg_start();
            msg_clr_eos();
            msg_bytes(&text[..lastnl + 1], get_echo_hl_id(), false);
            msg_didout.set(false);
            msg_starthere();
        }
    }
    cmdline_row.set(msg_row.get());

    stuff_readbuf_one_line(defstr.to_bytes());

    let save_ex_normal_busy = ex_normal_busy.get();
    ex_normal_busy.set(0);
    let line = getcmdline_prompt(
        if secret {
            NUL
        } else {
            '@' as ::core::ffi::c_int
        },
        Some(p),
        get_echo_hl_id(),
        xp_type,
        xp_arg.as_deref(),
        // The prompt takes the callback over, and releases it when it
        // returns.
        input_callback,
        false,
        None,
    );
    result.write_string(line.map(|line| ThinCString::from_cstr(line.as_cstr())));
    ex_normal_busy.set(save_ex_normal_busy);

    if result.string_ref().is_none() {
        if let Some(value) = &cancelreturn {
            tv_copy(value, result);
        } else if cancelreturn_in_opts && let Some(value) = opts_cancelreturn(&args[0]) {
            tv_copy(value, result);
        }
    }

    // Since the user typed this, no need to wait for return.
    need_wait_return.set(false);
    msg_didout.set(false);
}
