//! `nvim_paste()` and `nvim_put()`: bulk text insertion.
//!
//! `nvim_paste` is the streaming one -- it takes a chunk and a phase, so a
//! paste can arrive in pieces and be undone as a unit -- and it defers to
//! the `vim.paste()` Lua handler.  `nvim_put` is the register-style
//! insertion instead, taking a whole array of lines and a motion type.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, NIL, Reported, api_try, array_add};
use crate::getchar::PastePhase;
use crate::guard::Suppress;
use crate::normal::{set_visual_active, visual_active};
use crate::types::{NUL, PUT_CURSEND, Terminal};
use crate::winlayer::Buf;

/// The terminal the current buffer shows, if it shows one.
fn cur_buf_terminal() -> *mut Terminal {
    // SAFETY: `curbuf` names a live buffer for the editor's whole run.
    unsafe { Buf::current() }.terminal
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
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    static cancelled: GlobalCell<bool> = GlobalCell::new(false);
    if !(-1..=3).contains(&phase) {
        let name = c"phase".as_ptr();
        // SAFETY: `err` is this frame's own slot and `name` a literal.
        unsafe { api_err_invalid(err, name, ::core::ptr::null(), phase, false) };
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
        // SAFETY: `data` names its own bytes and `arena` is the caller's.
        let lines = unsafe { string_to_array(data, crlf, arena) };
        let mut args__items: [Object; 2] = [NIL; 2];
        let mut args = Array {
            size: 0 as size_t,
            capacity: 2 as size_t,
            items: (&raw mut args__items).cast::<Object>(),
        };
        // SAFETY: `args` is the two-slot block just declared above it.
        unsafe {
            array_add(&mut args, Object::array(lines));
            array_add(&mut args, Object::integer(phase));
        }
        let handler = String_0::from_cstr(c"return vim.paste(...)");
        let name = ::core::ptr::null::<::core::ffi::c_char>();
        // SAFETY: `args` is this frame's own and `arena`/`err` the caller's
        // and this frame's; the handler re-enters the editor through Lua.
        let rv = unsafe { nlua_exec(handler, name, args, kRetNilBool, arena, err) };
        // SAFETY: the tag says the boolean arm is the live one.
        let refused = rv.type_0 == kObjectTypeBoolean && !unsafe { rv.data.boolean };
        if error.type_0 != kErrorTypeNone || refused {
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
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut reg = yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    };
    // SAFETY: `reg` is this frame's own, and `type_0` names its own bytes.
    let typed = unsafe { prepare_yankreg_from_object(&raw mut reg, type_0, lines.size) };
    if !typed {
        // SAFETY: `err` is this frame's own slot and `type_0` NUL-terminated.
        unsafe { api_err_invalid(err, c"type".as_ptr(), type_0.data(), 0, true) };
        return ().reported(error);
    }
    if lines.size == 0 as size_t {
        return ().reported(error);
    }
    let bytes = lines.size.wrapping_mul(::core::mem::size_of::<String_0>());
    // SAFETY: `arena` is the caller's, and outlives the register below.
    reg.y_array = unsafe { arena_alloc(arena, bytes, true) }.cast::<String_0>();
    reg.y_size = lines.size;
    for i in 0..lines.size {
        // SAFETY: `lines` names its own `size` items.
        let item = unsafe { *lines.items.add(i) };
        if item.type_0 != kObjectTypeString {
            let (want, got) = (api_typename(kObjectTypeString), api_typename(item.type_0));
            // SAFETY: `err` is this frame's own slot, and both type names are
            // static strings.
            unsafe { api_err_exp(err, c"line".as_ptr(), want, got) };
            return ().reported(error);
        }
        // SAFETY: the tag above says the string arm is the live one, and
        // `reg.y_array` is the `size`-slot block just allocated. A NUL in an
        // API string stands for a newline, as it does in every buffer line.
        unsafe {
            let line = item.data.string;
            let copy = copy_string(line, arena);
            *reg.y_array.add(i) = copy;
            let text = copy.data().cast::<::core::ffi::c_void>();
            memchrsub(
                text,
                NUL as ::core::ffi::c_char,
                NL as ::core::ffi::c_char,
                line.len(),
            );
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
        PUT_CURSEND as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    api_try(&mut error, |_| {
        // `do_put` can leave Visual mode; the caller's is put back.
        let visual_was_active = visual_active();
        let silenced = Suppress::messages();
        // SAFETY: `reg` is this frame's own, filled in above.
        unsafe { do_put(0 as ::core::ffi::c_int, &raw mut reg, dir, 1, flags) };
        drop(silenced);
        set_visual_active(visual_was_active);
    });
    ().reported(error)
}
