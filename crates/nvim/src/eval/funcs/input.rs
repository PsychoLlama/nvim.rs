//! Asking the user: `input()`, `confirm()`, the prompt-buffer accessors
//! and `feedkeys()`.
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::tv_get_buf_from_arg;
use super::wrappers::{arg_number, arg_number_chk};
use crate::api::vim::nvim_feedkeys;
use crate::buffer::buf_is_prompt;
use crate::drawscreen::state::CMDLINE_ROW;
use crate::edit::buf_prompt_text_owned;
use crate::eval::funcs::builtin::EvalFuncData;
use crate::eval::prompt_get_input;
use crate::eval::typval::{NumBuf, list_iter, list_len};
use crate::ex_cmds::check_secure;
use crate::ex_getln::get_user_input;
use crate::getchar::state::{GOT_INT, TypeaheadSave};
use crate::getchar::{restore_typeahead, save_typeahead};
use crate::global_cell::GlobalCell;
use crate::guard::Suppress;
use crate::input::prompt_for_number;
use crate::message::msg_put_ascii;
use crate::message::state::{LINES_LEFT, MSG_ROW, MSG_SCROLL};
use crate::message::{Button, DialogAnswer, DialogKind, E_INVARG};
use crate::message::{
    confirm_dialog, emsg, msg_clr_eos, msg_ext_set_kind, msg_start, msg_str, verb_msg,
};
use crate::mouse::state::MOUSE_ROW;
use crate::narrow::{as_index, number_as_int};
use crate::option::vars::p_verbose;
use crate::os::cshim::gettext;
use crate::os::proc::os_kill;
use crate::semsg;
use crate::types::{ApiString, TypVal, VAR_LIST, VarNumber};
use crate::ui::state::{ROWS, UI_MESSAGES};
use crate::ui::ui_has;
use crate::winlayer::Buf;

/// `{type}` spellings `confirm()` recognises, by their first letter.
/// Anything else leaves the default in place.
const DIALOG_TYPES: [(u8, DialogKind); 5] = [
    (b'E', DialogKind::Error),
    (b'Q', DialogKind::Question),
    (b'I', DialogKind::Info),
    (b'W', DialogKind::Warning),
    (b'G', DialogKind::Generic),
];

/// `confirm({msg} [, {choices} [, {default} [, {type}]]])`
pub fn f_confirm(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut buttons_buf = NumBuf::new();
    let mut type_buf = NumBuf::new();
    let mut buttons = None;
    let mut default = 1;
    let mut kind = DialogKind::Generic;
    let mut error = false;

    let message = numbuf.string_chk(&args[0]).ok();
    if message.is_none() {
        error = true;
    }
    // Each optional argument is only read when the one before it was
    // supplied, and a coercion failure anywhere cancels the dialog --
    // but not the rest of the parse.
    if args.len() > 1 {
        buttons = buttons_buf.string_chk(&args[1]).ok();
        if buttons.is_none() {
            error = true;
        }
        if args.len() > 2 {
            default = number_as_int(arg_number_chk(&args[2], Some(&mut error)));
            if args.len() > 3 {
                let typestr = type_buf.bytes_chk(&args[3]);
                if let Some(typestr) = typestr {
                    let first = typestr.first().copied().unwrap_or(0).to_ascii_uppercase();
                    if let Some(&(_, found)) =
                        DIALOG_TYPES.iter().find(|&&(letter, _)| letter == first)
                    {
                        kind = found;
                    }
                } else {
                    error = true;
                }
            }
        }
    }
    // No {choices}, or an empty one, means a single "Ok".
    let buttons = buttons
        .filter(|buttons| !buttons.is_empty())
        .unwrap_or_else(|| gettext(c"&Ok"));
    if !error && let Some(message) = message {
        // The default is shown only when it names a button, but Enter
        // answers it whatever it is.
        let shown = usize::try_from(default).ok().and_then(Button::nth);
        let chosen = match confirm_dialog(kind, message, buttons, shown) {
            DialogAnswer::Default => VarNumber::from(default),
            DialogAnswer::Pressed(button) => {
                VarNumber::try_from(button.number()).unwrap_or(VarNumber::MAX)
            }
            DialogAnswer::Cancelled => 0,
        };
        result.write_number(chosen);
    }
}

/// `debugbreak({pid})` — SIGINT to a process, which on Windows is how a
/// debugger is attached. Answers 0; there is no success value.
pub fn f_debugbreak(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(0);
    let pid = number_as_int(arg_number(&args[0]));
    if pid == 0 {
        emsg(gettext(E_INVARG));
        return;
    }
    let _ = os_kill(pid, libc::SIGINT);
}

/// `feedkeys({string} [, {mode}])`
pub fn f_feedkeys(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let _rettv = result;
    let mut mode_buf = NumBuf::new();
    if check_secure() {
        return;
    }
    let keys = ApiString::from_cstr(numbuf.string(&args[0]));
    // A missing {mode} is spelled as a null string, not as "".
    let mode = args.get(1).map_or(ApiString::NULL, |mode| {
        ApiString::from_cstr(mode_buf.string(mode))
    });
    nvim_feedkeys(keys, mode, true);
}

/// Whether the prompt currently being read should echo `*` instead of what
/// was typed. Set by `inputsecret()` around its call to `input()`.
static INPUTSECRET: GlobalCell<bool> = GlobalCell::new(false);

/// `input({prompt} [, {text} [, {completion}]])`, or the options-Dict form.
pub fn f_input(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    get_user_input(args, result, false, INPUTSECRET.get());
}

/// `inputdialog()` — as `input()`, but cancelling answers the third
/// argument rather than an empty string.
pub fn f_inputdialog(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    get_user_input(args, result, true, INPUTSECRET.get());
}

/// `inputsecret({prompt} [, {text}])`
pub fn f_inputsecret(args: &[TypVal], result: &mut TypVal, fptr: EvalFuncData) {
    // The two globals are restored on the way out, and `f_input` cannot
    // unwind.
    let secret = Suppress::cmdline_echo();
    INPUTSECRET.set(true);
    f_input(args, result, fptr);
    drop(secret);
    INPUTSECRET.set(false);
}

/// `inputlist({textlist})` — print the list and read a number.
pub fn f_inputlist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    if args[0].v_type() != VAR_LIST {
        let arg0 = "inputlist()";
        semsg!("E686: Argument of {arg0} must be a List");
        return;
    }
    // Start at the bottom of the screen so the whole list is visible.
    msg_ext_set_kind(c"confirm");
    msg_start();
    MSG_ROW.set(ROWS.get() - 1);
    LINES_LEFT.set(ROWS.get());
    MSG_SCROLL.set(1);
    msg_clr_eos();

    // The List is held by the argument for the whole call, and printing it
    // runs no user code.
    let list = args[0].list_ref();
    let len = as_index(list_len(list));
    for (at, li) in list_iter(list).enumerate() {
        msg_str(numbuf.string(&li.li_tv));
        // A UI that owns the message area keeps the items in one message,
        // bar the last separator.
        if !ui_has(UI_MESSAGES) || at + 1 < len {
            msg_put_ascii(b'\n');
        }
    }

    let mut mouse_used = false;
    let mut selected = prompt_for_number(&mut mouse_used);
    // A click names a line rather than an item, so count back from the
    // bottom of the list.
    if mouse_used {
        selected = list_len(args[0].list_ref()) - (CMDLINE_ROW.get() - MOUSE_ROW.get());
    }
    result.write_number(VarNumber::from(selected));
}

/// The typeahead states `inputsave()` has stacked up.
///
/// A `Vec`, not a `GArray`: [`TypeaheadSave`] owns its buffers now, so the stack
/// has to move whole values rather than blit bytes into a grown tail.
static SAVED_TYPEAHEAD: GlobalCell<Vec<TypeaheadSave>> = GlobalCell::new(Vec::new());

/// `inputsave()` — push the typeahead aside so that a prompt reads real
/// keys.
pub fn f_inputsave(_args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    let mut saved = TypeaheadSave::default();
    save_typeahead(&mut saved);
    SAVED_TYPEAHEAD.with_mut(|stack| stack.push(saved));
}

/// `inputrestore()` — pop it back. Answers 1 only for an underflow, and
/// only when 'verbose' is high enough to have said something.
pub fn f_inputrestore(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // The pop happens outside the restore: `restore_typeahead` reaches the
    // typeahead cells, not this one, but keeping the borrow a leaf is the rule.
    if let Some(mut saved) = SAVED_TYPEAHEAD.with_mut(Vec::pop) {
        restore_typeahead(&mut saved);
    } else if p_verbose() > 1 {
        let msg = c"called inputrestore() more often than inputsave()";
        verb_msg(gettext(msg));
        result.write_number(1);
    }
}

/// `interrupt()` — raise the same flag CTRL-C does.
pub(crate) fn f_interrupt(_args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    GOT_INT.set(true);
}

/// The prompt buffer an accessor was asked about, or `None` for anything
/// that is not one.
fn prompt_buffer(arg: &TypVal) -> Option<Buf> {
    let buf = tv_get_buf_from_arg(arg);
    buf.filter(|b| buf_is_prompt(Some(*b)))
}

/// `prompt_getprompt({buf})` — the prompt text, or "" for a buffer that is
/// not a prompt buffer.
pub fn f_prompt_getprompt(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_string(None);
    if let Some(buf) = prompt_buffer(&args[0]) {
        result.write_string(Some(buf_prompt_text_owned(buf)));
    }
}

/// `prompt_getinput({buf})` — what has been typed after the prompt.
pub fn f_prompt_getinput(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_string(None);
    if let Some(buf) = prompt_buffer(&args[0]) {
        result.write_string(prompt_get_input(Some(buf)));
    }
}
