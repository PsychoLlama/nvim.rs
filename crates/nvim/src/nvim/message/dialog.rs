//! `confirm()`, and the console dialog it falls back to.
//!
//! [`do_dialog`] renders the message plus the button list, works out the
//! hotkey letters ([`copy_confirm_hotkeys`]) and reads a keystroke until one
//! of them matches.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::keycodes::Ctrl_C;
use core::ffi::{c_char, c_int};
use core::ptr;

/// How many buttons can carry a hotkey. Buttons past this share the default.
const HAS_HOTKEY_LEN: usize = 30;

/// Bytes reserved per hotkey, which may be a multibyte character.
const HOTK_LEN: c_int = MB_MAXBYTES as c_int;

/// Nonzero while the confirm message is being written, so `q` at the more
/// prompt cannot truncate it away.
pub(crate) static confirm_msg_used: GlobalCell<c_int> = GlobalCell::new(0);

/// The dialog's message text, as [`display_confirm_msg`] prints it.
pub(crate) static confirm_msg: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());

/// The rendered button list, used as the command-line prompt.
pub(crate) static confirm_buttons: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());

/// Ask the user to pick one of `buttons`, answering its 1-based index.
///
/// Answers 0 if the dialog was cancelled, and `dfltbutton` if there is no UI
/// to ask through -- without one Nvim would wait for input forever.
///
/// `buttons` is `"Button1\nButton2\n..."`, with `&` marking a hotkey letter.
/// `ex_cmd` allows `:` to dismiss the dialog and start an Ex command.
///
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
    unsafe {
        if silent_mode.get() {
            return dfltbutton;
        }

        let save_msg_silent = msg_silent.get();
        let old_state = State.get();
        msg_silent.set(0); // if the dialog prompts for input, the user needs to see it
        // We wait for a keypress, so don't make the user press RETURN as well.
        no_wait_return.set(no_wait_return.get() + 1);

        let hotkeys = msg_show_console_dialog(message, buttons, dfltbutton);
        let mut retval = 0;
        loop {
            // Without a UI Nvim waits for input forever.
            if ui_active() == 0 && input_available() == 0 {
                retval = dfltbutton;
                break;
            }

            // Get a typed character directly from the user.
            let mut c = prompt_for_input(confirm_buttons.get(), HLF_M, true, ptr::null_mut());
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
                    ins_char_typebuf(b':' as c_int, 0, false);
                    break;
                }
                _ => {
                    // Could be a hotkey. Lowercase it, as the ones in
                    // "hotkeys" are, and count how many buttons precede it.
                    c = mb_tolower(c);
                    retval = 1;
                    let mut i = 0;
                    while *hotkeys.add(i) != 0 {
                        if utf_ptr2char(hotkeys.add(i)) == c {
                            break;
                        }
                        i += utfc_ptr2len(hotkeys.add(i)) as usize;
                        retval += 1;
                    }
                    if *hotkeys.add(i) != 0 {
                        break;
                    }
                    // No hotkey match, so keep waiting.
                    msg_didany.set(false);
                    msg_didout.set(false);
                }
            }
        }

        xfree(hotkeys.cast());
        xfree(confirm_msg.get().cast());
        confirm_msg.set(ptr::null_mut());

        msg_silent.set(save_msg_silent);
        State.set(old_state);
        setmouse();
        no_wait_return.set(no_wait_return.get() - 1);
        msg_end_prompt();

        retval
    }
}

/// Copy one (possibly multibyte) character from `from` to `to`, answering its
/// length in bytes.
///
/// # Safety
/// `from` must be a valid C string and `to` must have room for the character.
unsafe fn copy_char(from: *const c_char, to: *mut c_char, lowercase: bool) -> c_int {
    unsafe {
        if lowercase {
            return utf_char2bytes(mb_tolower(utf_ptr2char(from)), to);
        }
        let len = utfc_ptr2len(from);
        ptr::copy(from, to, len as usize);
        len
    }
}

/// Size and allocate the dialog's three buffers, and record which buttons
/// name their own hotkey.
///
/// Answers the hotkey buffer; the message and button buffers go into
/// [`confirm_msg`] and [`confirm_buttons`].
///
/// # Safety
/// `message` and `buttons` must be valid C strings.
unsafe fn console_dialog_alloc(
    message: *const c_char,
    buttons: *const c_char,
    has_hotkey: &mut [bool; HAS_HOTKEY_LEN],
) -> *mut c_char {
    unsafe {
        let mut lenhotkey = HOTK_LEN; // count first button
        has_hotkey[0] = false;

        // Compute the size of memory to allocate.
        let mut msg_len = 0;
        let mut button_len = 0;
        let mut idx = 0;
        let mut r = buttons;
        while *r != 0 {
            if *r == DLG_BUTTON_SEP as c_char {
                button_len += 3; // '\n' -> ', '; 'x' -> '(x)'
                lenhotkey += HOTK_LEN; // each button needs a hotkey
                if idx < HAS_HOTKEY_LEN - 1 {
                    idx += 1;
                    has_hotkey[idx] = false;
                }
            } else if *r == DLG_HOTKEY_CHAR as c_char {
                r = r.add(1);
                button_len += 1; // '&a' -> '[a]'
                if idx < HAS_HOTKEY_LEN - 1 {
                    has_hotkey[idx] = true;
                }
            }
            r = r.add(utfc_ptr2len(r) as usize);
        }

        msg_len += strlen(message) as c_int + 3; // for the NLs and NUL
        button_len += strlen(buttons) as c_int + 3; // for the ": " and NUL
        lenhotkey += 1; // for the NUL

        // If no hotkey is specified, the first char is used.
        if !has_hotkey[0] {
            button_len += 2; // "x" -> "[x]"
        }

        confirm_msg.set(xmalloc(msg_len as size_t).cast());
        snprintf(
            confirm_msg.get(),
            msg_len as size_t,
            if ui_has(kUIMessages) {
                c"%s".as_ptr()
            } else {
                c"\n%s\n".as_ptr()
            },
            message,
        );

        xfree(confirm_buttons.get().cast());
        confirm_buttons.set(xmalloc(button_len as size_t).cast());

        xmalloc(lenhotkey as size_t).cast()
    }
}

/// Format the dialog and display it, answering the allocated hotkey string.
///
/// A button with no `&` takes the first character of its name as its hotkey.
///
/// # Safety
/// `message` and `buttons` must be valid C strings.
unsafe fn msg_show_console_dialog(
    message: *const c_char,
    buttons: *const c_char,
    dfltbutton: c_int,
) -> *mut c_char {
    unsafe {
        let mut has_hotkey = [false; HAS_HOTKEY_LEN];
        let hotk = console_dialog_alloc(message, buttons, &mut has_hotkey);
        copy_confirm_hotkeys(buttons, dfltbutton, &has_hotkey, hotk);
        display_confirm_msg();
        hotk
    }
}

/// Render the button list into [`confirm_buttons`] and the hotkey letters,
/// in order, into `hotkeys_ptr`.
///
/// # Safety
/// `buttons` must be a valid C string, and `hotkeys_ptr` must point at a
/// buffer [`console_dialog_alloc`] sized for it.
unsafe fn copy_confirm_hotkeys(
    buttons: *const c_char,
    mut default_button_idx: c_int,
    has_hotkey: &[bool; HAS_HOTKEY_LEN],
    hotkeys_ptr: *mut c_char,
) {
    unsafe {
        let mut hotkeys_ptr = hotkeys_ptr;
        // Define the first default hotkey. The hotkey string is kept NUL
        // terminated throughout, to avoid reading past the end.
        *hotkeys_ptr.add(copy_char(buttons, hotkeys_ptr, true) as usize) = 0;

        // Is the first char of the button a hotkey? It is when the button
        // does not name one itself.
        let mut first_hotkey = !has_hotkey[0];

        // Remember where the choices start; sent as the cmdline prompt.
        let mut msgp = confirm_buttons.get();
        // Takes the cursor by reference rather than capturing it: a closure
        // capturing `msgp` would hold the borrow for the whole loop.
        let push = |msgp: &mut *mut c_char, c: u8| {
            **msgp = c as c_char;
            *msgp = msgp.add(1);
        };

        let mut idx = 0;
        let mut r = buttons;
        while *r != 0 {
            if *r == DLG_BUTTON_SEP as c_char {
                push(&mut msgp, b','); // '\n' -> ', '
                push(&mut msgp, b' ');

                // Advance to the next hotkey and set the default one.
                hotkeys_ptr = hotkeys_ptr.add(strlen(hotkeys_ptr));
                *hotkeys_ptr.add(copy_char(r.add(1), hotkeys_ptr, true) as usize) = 0;

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
            } else if *r == DLG_HOTKEY_CHAR as c_char || first_hotkey {
                if *r == DLG_HOTKEY_CHAR as c_char {
                    r = r.add(1);
                }
                first_hotkey = false;
                if *r == DLG_HOTKEY_CHAR as c_char {
                    push(&mut msgp, *r as u8); // '&&a' -> '&a'
                } else {
                    // '&a' -> '[a]', or '(a)' when it is not the default.
                    let default = default_button_idx == 1;
                    push(&mut msgp, if default { b'[' } else { b'(' });
                    msgp = msgp.add(copy_char(r, msgp, false) as usize);
                    push(&mut msgp, if default { b']' } else { b')' });

                    // Redefine the hotkey.
                    *hotkeys_ptr.add(copy_char(r, hotkeys_ptr, true) as usize) = 0;
                }
            } else {
                // Everything else is copied literally.
                msgp = msgp.add(copy_char(r, msgp, false) as usize);
            }
            r = r.add(utfc_ptr2len(r) as usize);
        }

        push(&mut msgp, b':');
        push(&mut msgp, b' ');
        *msgp = 0;
    }
}

/// Display the `:confirm` message. Also called when the screen is resized.
///
/// # Safety
/// Only that [`confirm_msg`] is null or a valid C string.
pub(crate) unsafe fn display_confirm_msg() {
    unsafe {
        // Avoid that 'q' at the more prompt truncates the message here.
        confirm_msg_used.set(confirm_msg_used.get() + 1);
        if !confirm_msg.get().is_null() {
            msg_ext_set_kind(c"confirm".as_ptr());
            msg_puts_hl(confirm_msg.get(), HLF_M, false);
        }
        confirm_msg_used.set(confirm_msg_used.get() - 1);
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
    unsafe {
        let title = if title.is_null() {
            gettext(c"Question".as_ptr())
        } else {
            title
        };
        if do_dialog(
            type_0,
            title,
            message,
            gettext(c"&Yes\n&No".as_ptr()),
            dflt,
            ptr::null(),
            false_0,
        ) == 1
        {
            return VIM_YES as c_int;
        }
        VIM_NO as c_int
    }
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
    unsafe {
        let title = if title.is_null() {
            gettext(c"Question".as_ptr())
        } else {
            title
        };
        match do_dialog(
            type_0,
            title,
            message,
            gettext(c"&Yes\n&No\n&Cancel".as_ptr()),
            dflt,
            ptr::null(),
            false_0,
        ) {
            1 => VIM_YES as c_int,
            2 => VIM_NO as c_int,
            _ => VIM_CANCEL as c_int,
        }
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
    unsafe {
        // Note: unlike its two siblings, this default title is not translated.
        let title = if title.is_null() {
            c"Question".as_ptr()
        } else {
            title.cast_const()
        };
        match do_dialog(
            type_0,
            title,
            message,
            gettext(c"&Yes\n&No\nSave &All\n&Discard All\n&Cancel".as_ptr()),
            dflt,
            ptr::null(),
            false_0,
        ) {
            1 => VIM_YES as c_int,
            2 => VIM_NO as c_int,
            3 => VIM_ALL as c_int,
            4 => VIM_DISCARDALL as c_int,
            _ => VIM_CANCEL as c_int,
        }
    }
}
