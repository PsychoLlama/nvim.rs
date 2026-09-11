//! `nvim_paste()` and `nvim_put()`: bulk text insertion.
//!
//! `nvim_paste` is the streaming one -- it takes a chunk and a phase, so a
//! paste can arrive in pieces and be undone as a unit -- and it defers to
//! the `vim.paste()` Lua handler.  `nvim_put` is the register-style
//! insertion instead, taking a whole array of lines and a motion type.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::api::private::helpers::{Reported, api_try};
use crate::api::private::validate::{err_bad_number, err_bad_value, err_expected};
use crate::cstr;
use crate::getchar::PastePhase;
use crate::guard::Suppress;
use crate::normal::{set_visual_active, visual_active};
use crate::types::{NUL, PUT_CURSEND, Terminal};
use crate::winlayer::Buf;

/// The terminal the current buffer shows, if it shows one.
fn cur_buf_terminal() -> *mut Terminal {
    Buf::current().terminal
}

/// Hand `data` to the `vim.paste()` Lua handler as one chunk of a paste.
///
/// `phase` is `-1` for a whole paste at once, or `1`/`2`/`3` for the first,
/// a middle and the last chunk of a streamed one. The answer is whether the
/// paste is still wanted: a handler that answers `false` cancels the rest,
/// and every later chunk is dropped until the next `-1` or `1`.
///
/// # Safety
/// `data` must name its own bytes and `arena` must be the caller's.
pub unsafe fn nvim_paste(
    channel_id: uint64_t,
    data: String_0,
    crlf: Boolean,
    phase: Integer,
    arena: *mut Arena,
) -> Result<Boolean, Error> {
    let mut error = Error::none();
    static cancelled: GlobalCell<bool> = GlobalCell::new(false);
    if !(-1..=3).contains(&phase) {
        let name = c"phase".as_ptr();
        // SAFETY: `error` is this frame's own slot and `name` a literal.
        error = err_bad_number(unsafe { cstr::at(name) }, phase);
        return false.reported(error);
    }
    let whole = phase == -1;
    's_151: {
        if whole || phase == 1 {
            cancelled.set(false);
            let terminal = cur_buf_terminal();
            if !terminal.is_null() {
                // SAFETY: the current buffer's own terminal.
                unsafe { terminal_set_streamed_paste(terminal, true) };
            }
        } else if cancelled.get() {
            break 's_151;
        }
        let lines = string_to_array(&data, crlf);
        let mut args = Array::with_capacity(2);
        args.push(Object::array(lines));
        args.push(Object::integer(phase));
        let handler = String_0::from_cstr(c"return vim.paste(...)");
        let name = ::core::ptr::null::<::core::ffi::c_char>();
        // SAFETY: `args` is this frame's own and `arena`/`error` the caller's
        // and this frame's; the handler re-enters the editor through Lua.
        let rv = match unsafe { nlua_exec(&handler, name, args, kRetNilBool, arena) } {
            Ok(value) => value,
            Err(e) => {
                error = e;
                Object::Nil
            }
        };
        let refused = rv.as_boolean() == Some(false);
        if error.is_set() || refused {
            cancelled.set(true);
        }
        let terminal = cur_buf_terminal();
        if (whole || phase == 3 || cancelled.get()) && !terminal.is_null() {
            // SAFETY: the current buffer's own terminal.
            unsafe { terminal_set_streamed_paste(terminal, false) };
        }
        // The paste is recorded for `.` even when the handler declined it,
        // so that the redo carries the same text.
        // SAFETY: `data` names its own bytes.
        unsafe {
            if !cancelled.get() && (whole || phase == 1) {
                paste_store(channel_id, PastePhase::Start, String_0::NULL, crlf);
            }
            if !cancelled.get() {
                paste_store(channel_id, PastePhase::Chunk, data, crlf);
            }
            if phase == 3 || phase == if cancelled.get() { 2 } else { -1 } {
                paste_store(channel_id, PastePhase::End, String_0::NULL, crlf);
            }
        }
    }
    let retval = !cancelled.get();
    if whole || phase == 3 {
        cancelled.set(false);
    }
    retval.reported(error)
}

/// Insert `lines` at the cursor the way a register paste would, `type_0`
/// naming the motion type (`""`, `"v"`, `"V"` or `"b"`).
///
/// # Safety
/// `lines` must name its own items, `type_0` its own bytes, and `arena` must
/// be the caller's.
pub unsafe fn nvim_put(
    lines: Array,
    type_0: String_0,
    after: Boolean,
    follow: Boolean,
    arena: *mut Arena,
) -> Result<(), Error> {
    let mut error = Error::none();
    let mut reg = YankReg {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    };
    // SAFETY: `reg` is this frame's own, and `type_0` names its own bytes.
    let typed = unsafe { prepare_yankreg_from_object(&raw mut reg, &type_0, lines.len()) };
    if !typed {
        // SAFETY: `err` is this frame's own slot and `type_0` NUL-terminated.
        error = err_bad_value(c"type", type_0.as_cstr());
        return ().reported(error);
    }
    if lines.len() == 0 as size_t {
        return ().reported(error);
    }
    let bytes = lines.len().wrapping_mul(::core::mem::size_of::<String_0>());
    // SAFETY: `arena` is the caller's, and outlives the register below. The
    // block is zeroed: a slot the walk below assigns into has to hold a
    // releasable string, and all-zero is the null one.
    reg.y_array = unsafe { arena_alloc(arena, bytes, true) }.cast::<String_0>();
    unsafe { reg.y_array.write_bytes(0, lines.len()) };
    reg.y_size = lines.len();
    for i in 0..lines.len() {
        // SAFETY: `lines` names its own `size` items.
        let item = &lines[i];
        let Some(line) = item.as_string() else {
            let (want, got) = (api_typename(kObjectTypeString), api_typename(item.kind()));
            // SAFETY: `err` is this frame's own slot, and both type names are
            // static strings.
            error = err_expected(c"line", want, Some(got));
            return ().reported(error);
        };
        let nul = ::core::ffi::c_char::try_from(NUL).expect("NUL is zero");
        let nl = ::core::ffi::c_char::try_from(NL).expect("NL is an ASCII byte");
        // SAFETY: `reg.y_array` is the `size`-slot block just allocated. A NUL
        // in an API string stands for a newline, as it does in every buffer
        // line.
        unsafe {
            let copy = line.clone();
            let text = copy.data().cast::<::core::ffi::c_void>();
            let len = copy.len();
            *reg.y_array.add(i) = copy;
            memchrsub(text, nul, nl, len);
        }
    }
    // SAFETY: `reg` is this frame's own, now holding `y_size` lines.
    unsafe { finish_yankreg_from_object(&raw mut reg, false) };
    let dir = if after {
        FORWARD as ::core::ffi::c_int
    } else {
        BACKWARD as ::core::ffi::c_int
    };
    let flags = if follow {
        PUT_CURSEND.cast_signed()
    } else {
        0 as ::core::ffi::c_int
    };
    api_try(|| {
        // `do_put` can leave Visual mode; the caller's is put back.
        let visual_was_active = visual_active();
        let silenced = Suppress::messages();
        // SAFETY: `reg` is this frame's own, filled in above.
        unsafe { do_put(0 as ::core::ffi::c_int, &raw mut reg, dir, 1, flags) };
        drop(silenced);
        set_visual_active(visual_was_active);
    })?;
    ().reported(error)
}
