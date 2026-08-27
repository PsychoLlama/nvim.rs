//! Asking the user: `input()`, `confirm()`, the prompt-buffer accessors
//! and `feedkeys()`.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::frame;
use super::wrappers::{arg_number, arg_number_chk, arg_string, arg_string_chk};
use super::{
    SIGINT, VIM_ERROR, VIM_GENERIC, VIM_INFO, VIM_QUESTION, VIM_WARNING, tv_get_buf_from_arg,
};
use crate::api::private::helpers::cstr_as_string;
use crate::api::vim::nvim_feedkeys;
use crate::buffer::buf_is_prompt;
use crate::edit::buf_prompt_text;
use crate::eval::prompt_get_input;
use crate::eval::typval::{NumBuf, tv_list_len};
use crate::event::libuv::uv_kill;
use crate::ex_cmds::check_secure;
use crate::ex_getln::get_user_input;
use crate::getchar::{restore_typeahead, save_typeahead};
use crate::global_cell::GlobalCell;
use crate::input::prompt_for_input;
use crate::main::{
    Rows, cmdline_row, cmdline_star, e_invarg, e_listarg, got_int, lines_left, mouse_row, msg_row,
    msg_scroll, p_verbose,
};
use crate::memory::xstrdup;
use crate::message::{
    do_dialog, emsg, msg_clr_eos, msg_ext_set_kind, msg_putchar, msg_puts, msg_start, verb_msg,
};
use crate::os::cshim::gettext;
use crate::semsg_c;
use crate::types::ui::kUIMessages;
use crate::types::{
    EvalFuncData, FAIL, NUL, VAR_LIST, VAR_STRING, listitem_T, tasave_T, typval_T, varnumber_T,
};
use crate::ui::ui_has;
use crate::winlayer::Buf;
use core::ffi::{c_char, c_int};
use core::ptr;

/// The size of a `tv_get_string_buf*` scratch buffer. `NUMBUFLEN` in the C.
const NUMBUFLEN: usize = 65;

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
pub unsafe fn f_confirm(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    let mut buttons_buf = NumBuf::new();
    let mut type_buf = NumBuf::new();
    let mut buttons = ptr::null::<c_char>();
    let mut default = 1;
    let mut kind = VIM_GENERIC as c_int;
    let mut error = false;

    // SAFETY: the frame is live; the two scratch buffers outlive the
    // strings `tv_get_string_buf_chk` may park in them and the dialog runs
    // before they go out of scope.
    let message = arg_string_chk(&mut numbuf, args.get(0));
    if message.is_null() {
        error = true;
    }
    // Each optional argument is only read when the one before it was
    // supplied, and a coercion failure anywhere cancels the dialog --
    // but not the rest of the parse.
    if args.has(1) {
        buttons = arg_string_chk(&mut buttons_buf, args.get(1));
        if buttons.is_null() {
            error = true;
        }
        if args.has(2) {
            default = arg_number_chk(args.get(2), Some(&mut error)) as c_int;
            if args.has(3) {
                let typestr = arg_string_chk(&mut type_buf, args.get(3));
                if typestr.is_null() {
                    error = true;
                } else {
                    let first = (unsafe { *typestr } as u8).to_ascii_uppercase();
                    if let Some(&(_, found)) =
                        DIALOG_TYPES.iter().find(|&&(letter, _)| letter == first)
                    {
                        kind = found;
                    }
                }
            }
        }
    }
    // No {choices}, or an empty one, means a single "Ok".
    if buttons.is_null() || unsafe { *buttons } as c_int == NUL {
        buttons = unsafe { gettext(c"&Ok".as_ptr()) };
    }
    if !error {
        rettv.vval.v_number =
            unsafe { do_dialog(kind, ptr::null(), message, buttons, default, ptr::null(), 0) }
                as varnumber_T;
    }
}

/// `debugbreak({pid})` — SIGINT to a process, which on Windows is how a
/// debugger is attached. Answers FAIL; there is no success value.
pub unsafe fn f_debugbreak(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = FAIL as varnumber_T;
    // SAFETY: the frame is live.
    let pid = arg_number(args.get(0)) as c_int;
    if pid == 0 {
        unsafe { emsg(gettext(e_invarg.as_ptr())) };
        return;
    }
    unsafe { uv_kill(pid, SIGINT) };
}

/// `feedkeys({string} [, {mode}])`
pub unsafe fn f_feedkeys(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, _rettv) = frame!(argvars, rettv);
    let mut mode_buf = NumBuf::new();
    // SAFETY: the frame is live and both strings outlive the call.
    if check_secure() {
        return;
    }
    let keys = arg_string(&mut numbuf, args.get(0));
    // A missing {mode} is spelled as a null string, not as "".
    let mode = if args.has(1) {
        arg_string(&mut mode_buf, args.get(1))
    } else {
        ptr::null()
    };
    unsafe { nvim_feedkeys(cstr_as_string(keys), cstr_as_string(mode), true) };
}

/// Whether the prompt currently being read should echo `*` instead of what
/// was typed. Set by `inputsecret()` around its call to `input()`.
static INPUTSECRET: GlobalCell<bool> = GlobalCell::new(false);

/// `input({prompt} [, {text} [, {completion}]])`, or the options-Dict form.
pub unsafe fn f_input(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the dispatcher's argument array and return value.
    unsafe { get_user_input(argvars, rettv, false, INPUTSECRET.get()) };
}

/// `inputdialog()` — as `input()`, but cancelling answers the third
/// argument rather than an empty string.
pub unsafe fn f_inputdialog(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the dispatcher's argument array and return value.
    unsafe { get_user_input(argvars, rettv, true, INPUTSECRET.get()) };
}

/// `inputsecret({prompt} [, {text}])`
pub unsafe fn f_inputsecret(argvars: *mut typval_T, rettv: *mut typval_T, fptr: EvalFuncData) {
    // SAFETY: the dispatcher's argument array and return value; the two
    // globals are restored on the way out, and `f_input` cannot unwind.
    cmdline_star.set(cmdline_star.get() + 1);
    INPUTSECRET.set(true);
    unsafe { f_input(argvars, rettv, fptr) };
    cmdline_star.set(cmdline_star.get() - 1);
    INPUTSECRET.set(false);
}

/// `inputlist({textlist})` — print the list and read a number.
pub unsafe fn f_inputlist(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live and the List is held by an argument for the
    // whole call.
    if args.ty(0) != VAR_LIST {
        semsg_c!(
            unsafe { gettext(e_listarg.as_ptr()) },
            c"inputlist()".as_ptr(),
        );
        return;
    }
    // Start at the bottom of the screen so the whole list is visible.
    unsafe { msg_ext_set_kind(c"confirm".as_ptr()) };
    unsafe { msg_start() };
    msg_row.set(Rows.get() - 1);
    lines_left.set(Rows.get());
    msg_scroll.set(1);
    unsafe { msg_clr_eos() };

    let list = unsafe { args.get(0).vval.v_list };
    if !list.is_null() {
        let mut li: *const listitem_T = unsafe { (*list).lv_first };
        while !li.is_null() {
            unsafe { msg_puts(numbuf.string(&raw const (*li).li_tv)) };
            // A UI that owns the message area keeps the items in one
            // message, bar the last separator.
            if !ui_has(kUIMessages) || !unsafe { (*li).li_next }.is_null() {
                unsafe { msg_putchar('\n' as c_int) };
            }
            li = unsafe { (*li).li_next };
        }
    }

    let mut mouse_used = false;
    let mut selected = unsafe { prompt_for_input(ptr::null_mut(), 0, false, &raw mut mouse_used) };
    // A click names a line rather than an item, so count back from the
    // bottom of the list.
    if mouse_used {
        selected = unsafe { tv_list_len(list) } - (cmdline_row.get() - mouse_row.get());
    }
    rettv.vval.v_number = selected as varnumber_T;
}

/// The typeahead states `inputsave()` has stacked up.
///
/// A `Vec`, not a `garray_T`: [`tasave_T`] owns its buffers now, so the stack
/// has to move whole values rather than blit bytes into a grown tail.
static SAVED_TYPEAHEAD: GlobalCell<Vec<tasave_T>> = GlobalCell::new(Vec::new());

/// `inputsave()` — push the typeahead aside so that a prompt reads real
/// keys.
pub unsafe fn f_inputsave(_argvars: *mut typval_T, _rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut saved = tasave_T::default();
    // SAFETY: `saved` is a fresh state of the right type, and the stack owns
    // it from here on.
    unsafe { save_typeahead(&raw mut saved) };
    SAVED_TYPEAHEAD.with_mut(|stack| stack.push(saved));
}

/// `inputrestore()` — pop it back. Answers 1 only for an underflow, and
/// only when 'verbose' is high enough to have said something.
pub unsafe fn f_inputrestore(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // The pop happens outside the restore: `restore_typeahead` reaches the
    // typeahead cells, not this one, but keeping the borrow a leaf is the rule.
    if let Some(mut saved) = SAVED_TYPEAHEAD.with_mut(Vec::pop) {
        // SAFETY: filled by the `f_inputsave` that pushed it.
        unsafe { restore_typeahead(&raw mut saved) };
    } else if p_verbose.get() > 1 {
        // SAFETY: a static message, and the caller's return value.
        let msg = c"called inputrestore() more often than inputsave()";
        unsafe { verb_msg(gettext(msg.as_ptr())) };
        unsafe { (*rettv).vval.v_number = 1 };
    }
}

/// `interrupt()` — raise the same flag CTRL-C does.
pub unsafe fn f_interrupt(_argvars: *mut typval_T, _rettv: *mut typval_T, _fptr: EvalFuncData) {
    got_int.set(true);
}

/// The prompt buffer an accessor was asked about, or `None` for anything
/// that is not one.
///
/// # Safety
/// `arg` is a live typval.
unsafe fn prompt_buffer(arg: *mut typval_T) -> Option<Buf> {
    // SAFETY: the caller's obligation -- `tv_get_buf_from_arg` answers a live
    // buffer or null.
    let buf = unsafe { Buf::from_raw(tv_get_buf_from_arg(arg)) };
    buf.filter(|b| buf_is_prompt(Some(*b)))
}

/// `prompt_getprompt({buf})` — the prompt text, or "" for a buffer that is
/// not a prompt buffer.
pub unsafe fn f_prompt_getprompt(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_STRING;
    rettv.vval.v_string = ptr::null_mut();
    // SAFETY: the frame is live and `rettv` owns the duplicate.
    if let Some(buf) = unsafe { prompt_buffer(args.ptr(0)) } {
        rettv.vval.v_string = unsafe { xstrdup(buf_prompt_text(buf)) };
    }
}

/// `prompt_getinput({buf})` — what has been typed after the prompt.
pub unsafe fn f_prompt_getinput(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_STRING;
    rettv.vval.v_string = ptr::null_mut();
    // SAFETY: the frame is live and `prompt_get_input` hands over an
    // allocation `rettv` then owns.
    if let Some(buf) = unsafe { prompt_buffer(args.ptr(0)) } {
        rettv.vval.v_string = unsafe { prompt_get_input(buf.raw()) };
    }
}
