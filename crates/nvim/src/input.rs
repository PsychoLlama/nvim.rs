//! The three ways nvim asks the user something outside the normal input
//! loop: a yes/no question ([`ask_yesno`]), one raw keystroke
//! ([`get_keystroke`]), and the cmdline prompt the other two are built on
//! ([`prompt_for_input`]).
//!
//! None of these goes through `vgetc()`, which would sync undo and consume
//! mapped characters; the price is that typeahead is ignored.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::eval::typval::kCallbackNone;
use crate::ex_getln::getcmdline_prompt;
use crate::getchar::{fix_input_buffer, merge_modifiers};
use crate::guard::{Keys, Suppress};
use crate::highlight_group::HLF_R;
use crate::keycodes::{Ctrl_C, K_IGNORE, K_LEFTMOUSE, K_SPECIAL, KS_MODIFIER, key_unescape};
use crate::main::{
    IObuff, State, cmdline_row, keep_msg, keep_msg_hl_id, mapped_ctrl_c, mod_mask, msg_row,
    msg_scrolled, need_wait_return,
};
use crate::mbyte::{utf_ptr2char, utf8len_tab};
use crate::memory::{xfree, xstrdup};
use crate::message::{msg_putchar, set_keep_msg};
use crate::mouse::{is_mouse_key, setmouse};
use crate::os::cshim::{gettext, snprintf};
use crate::os::input::input_get;
use crate::types::ui::kUIMessages;
use crate::types::{Callback, Callback_data, ExpandContext, IOSIZE, MultiQueue, NUL};
use crate::ui::{ui_flush, ui_has};
use ::libc::atoi;
use core::ffi::{c_char, c_int, c_void};

const ESC: c_int = 0x1b;
/// The unset callback, as `CALLBACK_NONE`.
const CALLBACK_NONE: Callback = Callback {
    data: Callback_data {
        funcref: core::ptr::null_mut(),
    },
    type_0: kCallbackNone,
};

/// Ask for a `y` or an `n`, with `str` (already translated) as the question;
/// it is always followed by `" (y/n)?"` and repeated until one of the two —
/// or `CTRL-C`, which answers `n` — is typed.
///
/// # Safety
/// Main-thread editor call; `str` is NUL-terminated.
pub(crate) unsafe fn ask_yesno(str: *const c_char) -> c_int {
    let save_state = State.get();
    let no_prompt = Suppress::wait_return();

    // SAFETY: `IObuff` is the shared scratch buffer, `IOSIZE` chars long,
    // and the question is the one `%s` the format takes. The copy is made
    // under the same borrow, before anything else can overwrite it.
    let prompt = IObuff.with_mut(|buf| unsafe {
        snprintf(
            buf.as_mut_ptr(),
            IOSIZE as usize,
            gettext(c"%s (y/n)?".as_ptr()),
            str,
        );
        xstrdup(buf.as_ptr())
    });

    let mut r = ' ' as c_int;
    while r != 'y' as c_int && r != 'n' as c_int {
        // Same highlighting as for wait_return().
        // SAFETY: `prompt` is the owned NUL-terminated question.
        r = unsafe { prompt_for_input(prompt, HLF_R, true, core::ptr::null_mut()) };
        if r == Ctrl_C || r == ESC {
            r = 'n' as c_int;
            if !ui_has(kUIMessages) {
                // SAFETY: main-thread editor call.
                unsafe { msg_putchar(r) };
            }
        }
    }

    need_wait_return.set(msg_scrolled.get() != 0);
    drop(no_prompt);
    State.set(save_state);
    // SAFETY: main-thread editor call; `prompt` is the string `xstrdup`
    // handed over, and nothing else holds it.
    unsafe {
        setmouse();
        xfree(prompt as *mut c_void);
    }
    r
}

/// Whether `key` is one [`get_keystroke`] swallows rather than returns: a
/// modifier, the ignore key, or any mouse event other than a left click
/// (which the "more" prompt uses).
///
/// # Safety
/// Main-thread editor call (`is_mouse_key` reads the mouse state).
unsafe fn is_swallowed_key(first: u8, key: c_int) -> bool {
    // SAFETY: the caller's contract.
    c_int::from(first) == KS_MODIFIER
        || key == K_IGNORE
        || (is_mouse_key(key) && key != K_LEFTMOUSE)
}

/// Read one keystroke straight from the user, ignoring mouse clicks and
/// scrollbar events (except a left click). The interrupt character comes
/// back as `ESC` on unix.
///
/// # Safety
/// Main-thread editor call; `events` is null or a live queue to drain while
/// waiting.
pub(crate) unsafe fn get_keystroke(events: *mut MultiQueue) -> c_int {
    let save_mapped_ctrl_c = mapped_ctrl_c.get();
    mod_mask.set(0);
    mapped_ctrl_c.set(0); // Mappings are not used here.

    let mut buf: Vec<u8> = Vec::new();
    let mut buflen: c_int = 150;
    let mut len: c_int = 0;
    let key = loop {
        // Flush output before waiting.
        // SAFETY: main-thread editor call.
        unsafe { ui_flush() };

        // Leave some room for check_termcode() to insert a key code into
        // (max 5 chars plus NUL), and fix_input_buffer() can triple the
        // number of bytes.
        let mut maxlen = (buflen - 6 - len) / 3;
        if buf.is_empty() {
            buf = vec![0; buflen as usize];
        } else if maxlen < 10 {
            // Need some more space. This might happen while receiving a
            // long escape sequence.
            buflen += 100;
            buf.resize(buflen as usize, 0);
            maxlen = (buflen - 6 - len) / 3;
        }

        // First time: blocking wait. Second time: wait up to 100ms for a
        // terminal code to complete.
        let timeout = if len == 0 { -1 } else { 100 };
        // SAFETY: `buf[len..]` has at least `maxlen` bytes of room by the
        // arithmetic above, and `fix_input_buffer` rewrites in place within
        // the tripling headroom that leaves.
        let mut n = unsafe {
            input_get(
                buf.as_mut_ptr().add(len as usize),
                maxlen,
                timeout,
                0,
                events,
            )
        };
        if n > 0 {
            // Replace zero and K_SPECIAL by a special key code.
            n = unsafe { fix_input_buffer(buf.as_mut_ptr().add(len as usize), n) };
            len += n;
        }
        if n > 0 {
            // Found a termcode: adjust the length. Upstream *assigns* here,
            // after having just added, so a read that follows an incomplete
            // one forgets the bytes already in the buffer. Kept.
            len = n;
        }
        if len == 0 {
            continue; // Nothing typed yet.
        }

        // Handle a modifier and/or special key code.
        let first = buf[0];
        if c_int::from(first) != K_SPECIAL {
            if c_int::from(utf8len_tab[first as usize]) > len {
                continue; // More bytes to get.
            }
            let end = if len >= buflen { buflen - 1 } else { len };
            buf[end as usize] = NUL as u8;
            // SAFETY: `buf` was just NUL-terminated at or before `buflen`.
            break unsafe { utf_ptr2char(buf.as_ptr() as *const c_char) };
        }

        let key = key_unescape(buf[1], buf[2]);
        // SAFETY: main-thread editor call.
        if !unsafe { is_swallowed_key(buf[1], key) } {
            break key;
        }
        if c_int::from(buf[1]) == KS_MODIFIER {
            mod_mask.set(c_int::from(buf[2]));
        }
        len -= 3;
        if len > 0 {
            buf.copy_within(3..3 + len as usize, 0);
        }
    };

    mapped_ctrl_c.set(save_mapped_ctrl_c);
    // `merge_modifiers` consumes the modifier it folds in, so the global has
    // to be written back.
    let mut modifiers = mod_mask.get();
    // SAFETY: `modifiers` is a live local the callee reads and writes.
    let key = unsafe { merge_modifiers(key, &raw mut modifiers) };
    mod_mask.set(modifiers);
    key
}

/// Ask the user for input through a cmdline prompt.
///
/// `one_key` returns after a single key press; `mouse_used`, when non-null,
/// lets the user click a number instead of typing it. A null `prompt` gets
/// the default "type a number" wording, which depends on `mouse_used`.
///
/// # Safety
/// Main-thread editor call; `prompt` is null or NUL-terminated, and
/// `mouse_used` is null or writable.
pub(crate) unsafe fn prompt_for_input(
    prompt: *mut c_char,
    hl_id: c_int,
    one_key: bool,
    mouse_used: *mut bool,
) -> c_int {
    let mut ret = if one_key { ESC } else { 0 };
    // SAFETY: `keep_msg` is the editor's kept message, NUL-terminated while
    // non-null; the copy is owned here.
    let kmsg = unsafe {
        let kept = keep_msg.get();
        if kept.is_null() {
            core::ptr::null_mut()
        } else {
            xstrdup(kept)
        }
    };

    let prompt = if !prompt.is_null() {
        prompt
    } else if !mouse_used.is_null() {
        // SAFETY: a NUL-terminated literal.
        unsafe {
            gettext(
                c"Type number and <Enter> or click with the mouse (q or empty cancels): ".as_ptr(),
            )
        }
    } else {
        // SAFETY: a NUL-terminated literal.
        unsafe { gettext(c"Type number and <Enter> (q or empty cancels): ".as_ptr()) }
    };

    cmdline_row.set(msg_row.get());
    // SAFETY: main-thread editor call.
    unsafe { ui_flush() };

    // Don't map prompt input, but do allow special keys.
    let raw_key = Keys::unmapped_with_codes();
    // SAFETY: the caller's prompt and mouse flag; the answer is an owned
    // NUL-terminated string, freed here.
    let resp = unsafe {
        getcmdline_prompt(
            -1,
            prompt,
            hl_id,
            ExpandContext::Nothing,
            core::ptr::null(),
            CALLBACK_NONE,
            one_key,
            mouse_used,
        )
    };
    drop(raw_key);

    if !resp.is_null() {
        // SAFETY: non-null, so it is the owned answer.
        unsafe {
            ret = if one_key { *resp as c_int } else { atoi(resp) };
            xfree(resp as *mut c_void);
        }
    }
    if !kmsg.is_null() {
        // SAFETY: the copy taken above; `set_keep_msg` copies it again.
        unsafe {
            set_keep_msg(kmsg, keep_msg_hl_id.get());
            xfree(kmsg as *mut c_void);
        }
    }
    ret
}
