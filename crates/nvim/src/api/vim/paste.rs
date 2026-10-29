//! `nvim_paste()` and `nvim_put()`: bulk text insertion.
//!
//! `nvim_paste` is the streaming one -- it takes a chunk and a phase, so a
//! paste can arrive in pieces and be undone as a unit -- and it defers to
//! the `vim.paste()` Lua handler.  `nvim_put` is the register-style
//! insertion instead, taking a whole array of lines and a motion type.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
use super::*;
use crate::api::private::helpers::{Reported, api_try};
use crate::api::private::validate::{err_bad_number, err_bad_value, err_expected};
use crate::getchar::PastePhase;
use crate::guard::Suppress;
use crate::memory::XString;
use crate::normal::{set_visual_active, visual_active};
use crate::types::{PUT_CURSEND, RegLines};
use crate::winlayer::Buf;

/// Hand `data` to the `vim.paste()` Lua handler as one chunk of a paste.
///
/// `phase` is `-1` for a whole paste at once, or `1`/`2`/`3` for the first,
/// a middle and the last chunk of a streamed one. The answer is whether the
/// paste is still wanted: a handler that answers `false` cancels the rest,
/// and every later chunk is dropped until the next `-1` or `1`.
pub fn nvim_paste(
    channel_id: uint64_t,
    data: String_0,
    crlf: Boolean,
    phase: Integer,
) -> Result<Boolean, Error> {
    let mut error = Error::none();
    /// Whether the handler declined this paste: set until the next one
    /// starts, and what makes a later chunk a no-op.
    static CANCELLED: GlobalCell<bool> = GlobalCell::new(false);
    if !(-1..=3).contains(&phase) {
        error = err_bad_number(c"phase", phase);
        return false.reported(error);
    }
    let whole = phase == -1;
    let starting = whole || phase == 1;
    if starting {
        CANCELLED.set(false);
        buf_terminal_set_streamed_paste(Buf::current(), true);
    }
    if !CANCELLED.get() {
        let lines = string_to_array(&data, crlf);
        let mut args = Array::with_capacity(2);
        args.push(Object::array(lines));
        args.push(Object::integer(phase));
        // The handler re-enters the editor through Lua.
        let rv = match nlua_exec(b"return vim.paste(...)", None, args, kRetNilBool) {
            Ok(value) => value,
            Err(e) => {
                error = e;
                Object::Nil
            }
        };
        let refused = rv.as_boolean() == Some(false);
        if error.is_set() || refused {
            CANCELLED.set(true);
        }
        if whole || phase == 3 || CANCELLED.get() {
            buf_terminal_set_streamed_paste(Buf::current(), false);
        }
        // The paste is recorded for `.` even when the handler declined it,
        // so that the redo carries the same text.
        if !CANCELLED.get() && starting {
            paste_store(channel_id, PastePhase::Start, &[], crlf);
        }
        if !CANCELLED.get() {
            paste_store(channel_id, PastePhase::Chunk, data.as_bytes(), crlf);
        }
        if phase == 3 || phase == if CANCELLED.get() { 2 } else { -1 } {
            paste_store(channel_id, PastePhase::End, &[], crlf);
        }
    }
    let retval = !CANCELLED.get();
    if whole || phase == 3 {
        CANCELLED.set(false);
    }
    retval.reported(error)
}

/// Insert `lines` at the cursor the way a register paste would, `type_0`
/// naming the motion type (`""`, `"v"`, `"V"` or `"b"`).
pub fn nvim_put(
    lines: Array,
    type_0: String_0,
    after: Boolean,
    follow: Boolean,
) -> Result<(), Error> {
    let mut error = Error::none();
    let mut reg = YankReg::EMPTY;
    if !prepare_yankreg_from_object(&mut reg, type_0.as_bytes()) {
        error = err_bad_value(c"type", type_0.as_cstr());
        return ().reported(error);
    }
    if lines.is_empty() {
        return ().reported(error);
    }
    let mut owned: Vec<XString> = Vec::with_capacity(lines.len());
    for i in 0..lines.len() {
        let item = &lines[i];
        let Some(line) = item.as_string() else {
            let (want, got) = (api_typename(kObjectTypeString), api_typename(item.kind()));
            error = err_expected(c"line", want, Some(got));
            return ().reported(error);
        };
        // A NUL in an API string stands for a newline, as it does in every
        // buffer line.
        let text = line
            .as_bytes()
            .iter()
            .map(|&b| if b == 0 { b'\n' } else { b })
            .collect();
        owned.push(XString::from_vec(text));
    }
    reg.lines = RegLines::new(owned);
    finish_yankreg_from_object(&mut reg, false);
    let dir = if after { FORWARD } else { BACKWARD };
    let flags = if follow { PUT_CURSEND.cast_signed() } else { 0 };
    api_try(|| {
        // `do_put` can leave Visual mode; the caller's is put back.
        let visual_was_active = visual_active();
        let silenced = Suppress::messages();
        do_put(0, Some(&reg), dir, 1, flags);
        drop(silenced);
        set_visual_active(visual_was_active);
    })?;
    ().reported(error)
}
