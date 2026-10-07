//! Asking the user: `input()`, `confirm()`, the prompt-buffer accessors
//! and `feedkeys()`.
#![forbid(unsafe_code)]

use super::wrappers::{arg_number, arg_number_chk};
use super::{
    SIGINT, VIM_ERROR, VIM_GENERIC, VIM_INFO, VIM_QUESTION, VIM_WARNING, tv_get_buf_from_arg,
};
use crate::api::vim::nvim_feedkeys;
use crate::buffer::buf_is_prompt;
use crate::drawscreen::state::cmdline_row;
use crate::edit::buf_prompt_text_owned;
use crate::eval::prompt_get_input;
use crate::eval::typval::{NumBuf, list_iter, list_len};
use crate::ex_cmds::check_secure;
use crate::ex_getln::get_user_input;
use crate::getchar::state::got_int;
use crate::getchar::{restore_typeahead, save_typeahead};
use crate::global_cell::GlobalCell;
use crate::guard::Suppress;
use crate::input::prompt_for_number;
use crate::message::e_invarg;
use crate::message::state::{lines_left, msg_row, msg_scroll};
use crate::message::{
    confirm_dialog, emsg, msg_clr_eos, msg_ext_set_kind, msg_putchar, msg_start, msg_str, verb_msg,
};
use crate::mouse::state::mouse_row;
use crate::option::vars::p_verbose;
use crate::os::cshim::gettext;
use crate::os::proc::os_kill;
use crate::semsg;
use crate::types::ui::kUIMessages;
use crate::types::{EvalFuncData, FAIL, String_0, TypVal, TypeaheadSave, VAR_LIST, VarNumber};
use crate::ui::state::Rows;
use crate::ui::ui_has;
use crate::winlayer::Buf;
use core::ffi::c_int;

/// `{type}` spellings `confirm()` recognises, by their first letter.
/// Anything else leaves the default in place.
const DIALOG_TYPES: [(u8, c_int); 5] = [
    (b'E', VIM_ERROR as c_int),
    (b'Q', VIM_QUESTION as c_int),
    (b'I', VIM_INFO as c_int),
    (b'W', VIM_WARNING as c_int),
    (b'G', VIM_GENERIC as c_int),
];

/// `confirm({msg} [, {choices} [, {default} [, {type}]]])`
pub fn f_confirm(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut buttons_buf = NumBuf::new();
    let mut type_buf = NumBuf::new();
    let mut buttons = None;
    let mut default = 1;
    let mut kind = VIM_GENERIC as c_int;
    let mut error = false;

    let message = numbuf.string_chk(&args[0]);
    if message.is_none() {
        error = true;
    }
    // Each optional argument is only read when the one before it was
    // supplied, and a coercion failure anywhere cancels the dialog --
    // but not the rest of the parse.
    if args.len() > 1 {
        buttons = buttons_buf.string_chk(&args[1]);
        if buttons.is_none() {
            error = true;
        }
        if args.len() > 2 {
            default = arg_number_chk(&args[2], Some(&mut error)) as c_int;
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
        let chosen = confirm_dialog(kind, message, buttons, default);
        result.write_number(VarNumber::from(chosen));
    }
}

/// `debugbreak({pid})` — SIGINT to a process, which on Windows is how a
/// debugger is attached. Answers FAIL; there is no success value.
pub fn f_debugbreak(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(FAIL as VarNumber);
    let pid = arg_number(&args[0]) as c_int;
    if pid == 0 {
        emsg(gettext(e_invarg));
        return;
    }
    let _ = os_kill(pid, SIGINT);
}

/// `feedkeys({string} [, {mode}])`
pub fn f_feedkeys(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let _rettv = result;
    let mut mode_buf = NumBuf::new();
    if check_secure() {
        return;
    }
    let keys = String_0::from_cstr(numbuf.string(&args[0]));
    // A missing {mode} is spelled as a null string, not as "".
    let mode = args.get(1).map_or(String_0::NULL, |mode| {
        String_0::from_cstr(mode_buf.string(mode))
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
    msg_row.set(Rows.get() - 1);
    lines_left.set(Rows.get());
    msg_scroll.set(1);
    msg_clr_eos();

    // The List is held by the argument for the whole call, and printing it
    // runs no user code.
    let list = args[0].list_ref();
    let len = list_len(list) as usize;
    for (at, li) in list_iter(list).enumerate() {
        msg_str(numbuf.string(&li.li_tv));
        // A UI that owns the message area keeps the items in one message,
        // bar the last separator.
        if !ui_has(kUIMessages) || at + 1 < len {
            msg_putchar('\n' as c_int);
        }
    }

    let mut mouse_used = false;
    let mut selected = prompt_for_number(&mut mouse_used);
    // A click names a line rather than an item, so count back from the
    // bottom of the list.
    if mouse_used {
        selected = list_len(args[0].list_ref()) - (cmdline_row.get() - mouse_row.get());
    }
    result.write_number(selected as VarNumber);
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
    got_int.set(true);
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
