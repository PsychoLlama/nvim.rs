//! `confirm()`, and the console dialog it falls back to.
//!
//! [`do_dialog`] renders the message plus the button list, works out the
//! hotkey letters ([`render_buttons`]) and reads a keystroke until one of
//! them matches.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::cstr;
use crate::guard::{Allow, Suppress};
use crate::keycodes::Ctrl_C;
use crate::keycodes::ModMask;
use crate::mbyte::{char_at, cluster_len};
use crate::types::NUL;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// How many buttons can carry a hotkey. Buttons past this share the default.
const HAS_HOTKEY_LEN: usize = 30;

/// Ask the user to pick one of `buttons`, answering its 1-based index.
///
/// Answers 0 if the dialog was cancelled, and `dfltbutton` if there is no UI
/// to ask through -- without one Nvim would wait for input forever.
///
/// `buttons` is `"Button1\nButton2\n..."`, with `&` marking a hotkey letter.
/// `ex_cmd` allows `:` to dismiss the dialog and start an Ex command.
///
/// [`do_dialog`] with no title, text field or Ex command: `confirm()`.
pub(crate) fn confirm_dialog(
    kind: c_int,
    message: &CStr,
    buttons: &CStr,
    dfltbutton: c_int,
) -> c_int {
    let (message, buttons) = (message.as_ptr(), buttons.as_ptr());
    // SAFETY: two terminated strings, only read.
    unsafe {
        do_dialog(
            kind,
            ptr::null(),
            message,
            buttons,
            dfltbutton,
            ptr::null(),
            0,
        )
    }
}

/// # Safety
/// `message` and `buttons` must be valid C strings.
pub unsafe fn do_dialog(
    _type_0: c_int,
    _title: *const c_char,
    message: *const c_char,
    buttons: *const c_char,
    dfltbutton: c_int,
    _textfield: *const c_char,
    ex_cmd: c_int,
) -> c_int {
    if silent_mode.get() {
        return dfltbutton;
    }

    let old_state = State.get();
    // If the dialog prompts for input, the user needs to see it.
    let loud = Allow::messages();
    // We wait for a keypress, so don't make the user press RETURN as well.
    let no_prompt = Suppress::wait_return();

    // SAFETY: the caller's NUL-terminated strings.
    let (message, buttons) = unsafe { (cstr::at(message), cstr::at(buttons)) };
    let hotkeys = msg_show_console_dialog(message.to_bytes(), buttons.to_bytes(), dfltbutton);
    let retval;
    loop {
        // Without a UI Nvim waits for input forever.
        if ui_active() == 0 && input_available() == 0 {
            retval = dfltbutton;
            break;
        }

        // Get a typed character directly from the user. The prompt is a
        // copy: a dialog the typing reaches renders its own buttons.
        let prompt = confirm_buttons.with(Clone::clone).unwrap_or_default();
        // SAFETY: a main-thread prompt, with no mouse flag.
        let mut c =
            unsafe { prompt_for_input(Some(prompt.as_cstr()), HLF_M, true, ptr::null_mut()) };
        match c {
            CAR | NUL => {
                // User accepts the default option.
                retval = dfltbutton;
                break;
            }
            Ctrl_C | ESC => {
                // User aborts/cancels.
                retval = 0;
                break;
            }
            _ if c < 0 => {
                // Special keys are ignored here.
                msg_didany.set(false);
                msg_didout.set(false);
            }
            _ if c == b':' as c_int && ex_cmd != 0 => {
                retval = dfltbutton;
                ins_char_typebuf(b':' as c_int, ModMask::NONE, false);
                break;
            }
            _ => {
                // Could be a hotkey. Lowercase it, as the ones in
                // "hotkeys" are, and count how many buttons precede it. A
                // NUL hotkey (a button list ending in a separator) ends the
                // list, as the terminator of upstream's string did.
                c = mb_tolower(c);
                let listed = hotkeys.iter().take_while(|&&hotkey| hotkey != 0);
                if let Some(at) = listed.clone().position(|&hotkey| hotkey == c) {
                    retval = c_int::try_from(at).map_or(c_int::MAX, |at| at + 1);
                    break;
                }
                // No hotkey match, so keep waiting.
                msg_didany.set(false);
                msg_didout.set(false);
            }
        }
    }

    confirm_msg.set(None);

    drop(loud);
    State.set(old_state);
    setmouse();
    drop(no_prompt);
    msg_end_prompt();

    retval
}

/// Which buttons name their own hotkey, for the first [`HAS_HOTKEY_LEN`].
fn buttons_with_hotkeys(buttons: &[u8]) -> [bool; HAS_HOTKEY_LEN] {
    let mut has_hotkey = [false; HAS_HOTKEY_LEN];
    let mut idx = 0;
    let mut at = 0;
    while at < buttons.len() {
        if u32::from(buttons[at]) == DLG_BUTTON_SEP {
            if idx < HAS_HOTKEY_LEN - 1 {
                idx += 1;
                has_hotkey[idx] = false;
            }
        } else if u32::from(buttons[at]) == DLG_HOTKEY_CHAR {
            // The character after the `&` is skipped along with it.
            at += 1;
            if idx < HAS_HOTKEY_LEN - 1 {
                has_hotkey[idx] = true;
            }
        }
        at += cluster_len(&buttons[at.min(buttons.len())..]).max(1);
    }
    has_hotkey
}

/// The character at the start of `bytes`, lowercased: a hotkey.
fn hotkey_at(bytes: &[u8]) -> c_int {
    mb_tolower(char_at(bytes))
}

/// Render the button list as the command-line prompt shows it, and the
/// hotkey of each button, in order.
///
/// `&x` makes `x` the button's hotkey and shows it as `(x)`, or `[x]` on the
/// default button; `&&` is a literal `&`. A button with no `&` takes its
/// first character. Buttons are separated by `, ` and the line ends `: `.
fn render_buttons(buttons: &[u8], dfltbutton: c_int) -> (Vec<u8>, Vec<c_int>) {
    let has_hotkey = buttons_with_hotkeys(buttons);
    let mut shown = Vec::with_capacity(buttons.len() + 8);
    // The first button's default hotkey.
    let mut hotkeys = vec![hotkey_at(buttons)];
    let mut default_button_idx = dfltbutton;

    // Is the first char of the button a hotkey? It is when the button
    // does not name one itself.
    let mut first_hotkey = !has_hotkey[0];

    let mut idx = 0;
    let mut at = 0;
    while at < buttons.len() {
        let byte = u32::from(buttons[at]);
        if byte == DLG_BUTTON_SEP {
            shown.extend_from_slice(b", "); // '\n' -> ', '

            // Advance to the next hotkey and set the default one.
            hotkeys.push(hotkey_at(&buttons[at + 1..]));

            if default_button_idx != 0 {
                default_button_idx -= 1;
            }
            // If no hotkey is specified, the first char is used. The
            // increment is inside the short circuit, as upstream's
            // `has_hotkey[++idx]` is.
            if idx < HAS_HOTKEY_LEN - 1 && {
                idx += 1;
                !has_hotkey[idx]
            } {
                first_hotkey = true;
            }
        } else if byte == DLG_HOTKEY_CHAR || first_hotkey {
            if byte == DLG_HOTKEY_CHAR {
                at += 1;
            }
            first_hotkey = false;
            let rest = &buttons[at.min(buttons.len())..];
            if rest
                .first()
                .is_some_and(|&b| u32::from(b) == DLG_HOTKEY_CHAR)
            {
                shown.push(rest[0]); // '&&a' -> '&a'
            } else {
                // '&a' -> '[a]', or '(a)' when it is not the default.
                let default = default_button_idx == 1;
                shown.push(if default { b'[' } else { b'(' });
                shown.extend_from_slice(&rest[..cluster_len(rest)]);
                shown.push(if default { b']' } else { b')' });

                // Redefine the hotkey.
                if let Some(last) = hotkeys.last_mut() {
                    *last = hotkey_at(rest);
                }
            }
        } else {
            // Everything else is copied literally.
            let rest = &buttons[at..];
            shown.extend_from_slice(&rest[..cluster_len(rest)]);
        }
        at += cluster_len(&buttons[at.min(buttons.len())..]);
    }

    shown.extend_from_slice(b": ");
    (shown, hotkeys)
}

/// Format the dialog and display it, answering each button's hotkey.
fn msg_show_console_dialog(message: &[u8], buttons: &[u8], dfltbutton: c_int) -> Vec<c_int> {
    // With `ext_messages` the UI puts the message where it likes, so the
    // blank lines that separate it from the buttons are left out.
    let mut text = Vec::with_capacity(message.len() + 2);
    let framed = !ui_has(kUIMessages);
    if framed {
        text.push(b'\n');
    }
    text.extend_from_slice(message);
    if framed {
        text.push(b'\n');
    }
    confirm_msg.set(Some(XString::from_bytes(&text)));

    let (shown, hotkeys) = render_buttons(buttons, dfltbutton);
    confirm_buttons.set(Some(XString::from_bytes(&shown)));
    display_confirm_msg();
    hotkeys
}

/// Display the `:confirm` message. Also called when the screen is resized.
pub(crate) fn display_confirm_msg() {
    // Avoid that 'q' at the more prompt truncates the message here.
    let _in_use = Suppress::counter(confirm_msg_used);
    // A copy: showing it can redraw, and a redraw can show it again.
    if let Some(text) = confirm_msg.with(Clone::clone) {
        msg_ext_set_kind(c"confirm");
        msg_str_hl(text.as_cstr(), HLF_M, false);
    }
}

/// A yes/no dialog.
///
/// # Safety
/// `title` may be null; `message` must be a valid C string.
pub unsafe fn vim_dialog_yesno(
    type_0: c_int,
    title: *mut c_char,
    message: *mut c_char,
    dflt: c_int,
) -> c_int {
    let title = if title.is_null() {
        gettext(c"Question").as_ptr().cast_mut()
    } else {
        title
    };
    let buttons = gettext(c"&Yes\n&No");
    let nul = ptr::null();
    if unsafe { do_dialog(type_0, title, message, buttons.as_ptr(), dflt, nul, 0) } == 1 {
        return VIM_YES as c_int;
    }
    VIM_NO as c_int
}

/// A yes/no/cancel dialog.
///
/// # Safety
/// As [`vim_dialog_yesno`].
pub unsafe fn vim_dialog_yesnocancel(
    type_0: c_int,
    title: *mut c_char,
    message: *mut c_char,
    dflt: c_int,
) -> c_int {
    let title = if title.is_null() {
        gettext(c"Question").as_ptr().cast_mut()
    } else {
        title
    };
    let buttons = gettext(c"&Yes\n&No\n&Cancel");
    let nul = ptr::null();
    match unsafe { do_dialog(type_0, title, message, buttons.as_ptr(), dflt, nul, 0) } {
        1 => VIM_YES as c_int,
        2 => VIM_NO as c_int,
        _ => VIM_CANCEL as c_int,
    }
}

/// A yes/no/all/discard/cancel dialog, for `:wq` over several changed buffers.
///
/// # Safety
/// As [`vim_dialog_yesno`].
pub unsafe fn vim_dialog_yesnoallcancel(
    type_0: c_int,
    title: *mut c_char,
    message: *mut c_char,
    dflt: c_int,
) -> c_int {
    // Note: unlike its two siblings, this default title is not translated.
    let title = if title.is_null() {
        c"Question".as_ptr()
    } else {
        title.cast_const()
    };
    let buttons = gettext(c"&Yes\n&No\nSave &All\n&Discard All\n&Cancel");
    let nul = ptr::null();
    match unsafe { do_dialog(type_0, title, message, buttons.as_ptr(), dflt, nul, 0) } {
        1 => VIM_YES as c_int,
        2 => VIM_NO as c_int,
        3 => VIM_ALL as c_int,
        4 => VIM_DISCARDALL as c_int,
        _ => VIM_CANCEL as c_int,
    }
}
