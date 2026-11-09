//! The three ways nvim asks the user something outside the normal input
//! loop: a yes/no question ([`ask_yesno`]), one raw keystroke
//! ([`get_keystroke`]), and the cmdline prompt the other two are built on
//! ([`prompt_for_input`]).
//!
//! None of these goes through `vgetc().code()`, which would sync undo and consume
//! mapped characters; the price is that typeahead is ignored.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::cmdexpand::defs::ExpandContext;
use crate::drawscreen::state::CMDLINE_ROW;
use crate::event_queue::ChildQueue;
use crate::ex_getln::getcmdline_prompt;
use crate::getchar::state::{MAPPED_CTRL_C, MOD_MASK};
use crate::getchar::{fix_input_buffer, merge_modifiers};
use crate::global_cell::GlobalCell;
use crate::guard::{Keys, Suppress};
use crate::highlight_group::HLF_R;
use crate::keycodes::ModMask;
use crate::keycodes::{CTRL_C, K_SPECIAL, KS_MODIFIER, SpecialKey, key_unescape};
use crate::mbyte::{char_at, utf8len_tab};
use crate::message::IOSIZE;
use crate::message::state::{KEEP_MSG, KEEP_MSG_HL_ID, MSG_ROW, MSG_SCROLLED, NEED_WAIT_RETURN};
use crate::message::{msg_putchar, set_keep_msg};
use crate::mouse::{is_mouse_key, setmouse};
use crate::narrow::as_index;
use crate::narrow::len_as_int;
use crate::os::cshim::gettext;
use crate::os::input::{InputEvents, input_get};
use crate::state::mode::STATE;
use crate::types::Callback;
use crate::types::Key;
use crate::ui::state::UI_MESSAGES;
use crate::ui::{ui_flush, ui_has};
use crate::{cstr, formatted};
use core::ffi::{CStr, c_int};

const ESC: c_int = 0x1b;
/// The unset callback, as `CALLBACK_NONE`.
const CALLBACK_NONE: Callback = Callback::None;

/// Ask for a `y` or an `n`, with `str` (already translated) as the question;
/// it is always followed by `" (y/n)?"` and repeated until one of the two —
/// or `CTRL-C`, which answers `n` — is typed.
///
pub(crate) fn ask_yesno(question: &CStr) -> c_int {
    let save_state = STATE.get();
    let no_prompt = Suppress::wait_return();

    // The question, owned by this frame: upstream builds it in the shared
    // `IObuff`, which `prompt_for_input` writes again. The room is that
    // buffer's.
    let fmt = gettext(c"%s (y/n)?");
    let prompt = cstr::owned(&formatted!(IOSIZE as usize, fmt.to_bytes(), question));

    let mut r = c_int::from(b' ');
    while r != c_int::from(b'y') && r != c_int::from(b'n') {
        // Same highlighting as for wait_return().
        r = prompt_for_input(Some(&prompt), HLF_R, true, None);
        if r == CTRL_C || r == ESC {
            r = c_int::from(b'n');
            if !ui_has(UI_MESSAGES) {
                msg_putchar(r);
            }
        }
    }

    NEED_WAIT_RETURN.set(MSG_SCROLLED.get() != 0);
    drop(no_prompt);
    STATE.set(save_state);
    setmouse();
    r
}

/// Whether `key` is one [`get_keystroke`] swallows rather than returns: a
/// modifier, the ignore key, or any mouse event other than a left click
/// (which the "more" prompt uses).
fn is_swallowed_key(first: u8, key: c_int) -> bool {
    first == KS_MODIFIER
        || key == SpecialKey::Ignore.code()
        || (is_mouse_key(Key::from_code(key)) && key != SpecialKey::Leftmouse.code())
}

/// Read one keystroke straight from the user, ignoring mouse clicks and
/// scrollbar events (except a left click). The interrupt character comes
/// back as `ESC` on unix.
///
/// `events` names the queue to drain while waiting, if any.
pub(crate) fn get_keystroke(events: Option<&'static GlobalCell<Option<ChildQueue>>>) -> c_int {
    let events = events.map_or(InputEvents::None, InputEvents::Queue);
    let save_mapped_ctrl_c = MAPPED_CTRL_C.get();
    MOD_MASK.set(ModMask::NONE);
    MAPPED_CTRL_C.set(0); // Mappings are not used here.

    let mut buf: Vec<u8> = Vec::new();
    let mut buflen: c_int = 150;
    let mut len: c_int = 0;
    let key = loop {
        // Flush output before waiting.
        ui_flush();

        // Leave some room for check_termcode() to insert a key code into
        // (max 5 chars plus NUL), and fix_input_buffer() can triple the
        // number of bytes.
        let mut maxlen = (buflen - 6 - len) / 3;
        if buf.is_empty() {
            buf = vec![0; as_index(buflen)];
        } else if maxlen < 10 {
            // Need some more space. This might happen while receiving a
            // long escape sequence.
            buflen += 100;
            buf.resize(as_index(buflen), 0);
            maxlen = (buflen - 6 - len) / 3;
        }

        // First time: blocking wait. Second time: wait up to 100ms for a
        // terminal code to complete.
        let timeout = if len == 0 { -1 } else { 100 };
        // `buf[len..]` has at least `maxlen` bytes of room by the arithmetic
        // above, and `fix_input_buffer` rewrites in place within the
        // tripling headroom that leaves.
        let at = as_index(len);
        let read = &mut buf[at..at + as_index(maxlen.max(0))];
        let mut n = len_as_int(input_get(read, timeout, 0, events));
        if n > 0 {
            // Replace zero and K_SPECIAL by a special key code.
            n = len_as_int(fix_input_buffer(&mut buf[at..], as_index(n)));
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
        if first != K_SPECIAL {
            if c_int::from(utf8len_tab[first as usize]) > len {
                continue; // More bytes to get.
            }
            let end = if len >= buflen { buflen - 1 } else { len };
            break char_at(&buf[..as_index(end)]);
        }

        let key = key_unescape(buf[1], buf[2]).code();
        if !is_swallowed_key(buf[1], key) {
            break key;
        }
        if buf[1] == KS_MODIFIER {
            MOD_MASK.set(ModMask::from_byte(buf[2]));
        }
        len -= 3;
        if len > 0 {
            buf.copy_within(3..3 + as_index(len), 0);
        }
    };

    MAPPED_CTRL_C.set(save_mapped_ctrl_c);
    // `merge_modifiers` consumes the modifier it folds in, so the global has
    // to be written back.
    let mut modifiers = MOD_MASK.get();
    // SAFETY: `modifiers` is a live local the callee reads and writes.
    let key = merge_modifiers(Key::from_code(key), &mut modifiers);
    MOD_MASK.set(modifiers);
    key.code()
}

/// Ask the user for input through a cmdline prompt.
///
/// [`prompt_for_input`] for a number from a list on screen, which a mouse
/// click may pick instead of a typed number: `mouse_used` says which.
pub(crate) fn prompt_for_number(mouse_used: &mut bool) -> c_int {
    prompt_for_input(None, 0, false, Some(mouse_used))
}

/// `one_key` returns after a single key press; `mouse_used`, when given,
/// lets the user click a number instead of typing it. No `prompt` gets the
/// default "type a number" wording, which depends on `mouse_used`.
pub(crate) fn prompt_for_input(
    prompt: Option<&CStr>,
    hl_id: c_int,
    one_key: bool,
    mouse_used: Option<&mut bool>,
) -> c_int {
    let mut ret = if one_key { ESC } else { 0 };
    // A copy: the prompt's own messages replace the kept one.
    let kmsg = KEEP_MSG.with(Clone::clone);

    let prompt = match prompt {
        Some(prompt) => prompt,
        None if mouse_used.is_some() => {
            gettext(c"Type number and <Enter> or click with the mouse (q or empty cancels): ")
        }
        None => gettext(c"Type number and <Enter> (q or empty cancels): "),
    };

    CMDLINE_ROW.set(MSG_ROW.get());
    ui_flush();

    // Don't map prompt input, but do allow special keys.
    let raw_key = Keys::unmapped_with_codes();
    let resp = getcmdline_prompt(
        None,
        Some(prompt),
        hl_id,
        ExpandContext::Nothing,
        None,
        CALLBACK_NONE,
        one_key,
        mouse_used,
    );
    drop(raw_key);

    if let Some(resp) = resp {
        let text = resp.as_cstr().to_bytes();
        // One key is one character, multibyte or not: upstream answers the
        // first *byte* through a signed `char`, which `do_dialog` drops as
        // a special key whenever the hotkey is not ASCII.
        ret = if one_key {
            char_at(text)
        } else {
            crate::charset::atoi(text)
        };
    }
    if let Some(kmsg) = kmsg {
        set_keep_msg(Some(kmsg.as_cstr()), KEEP_MSG_HL_ID.get());
    }
    ret
}
