//! Reading the terminal.
//!
//! Bytes arrive from the terminal in whatever chunks the read gives us, and
//! carry three different things tangled together: keys, the terminal's
//! answers to the questions [`negotiate`](super::negotiate) asked, and
//! pasted text. This module untangles them.
//!
//! Keys go through termkey, which turns byte sequences into parsed keys;
//! [`keys`](super::keys) then names them, and the names are staged in
//! `key_buffer` and sent to the editor as `nvim_input`. Paste brackets
//! switch the stream into a mode where nothing is parsed and everything is
//! forwarded as `nvim_paste`. Focus events are recognised here directly --
//! termkey has no notion of them -- and everything else the terminal says
//! reaches the editor as `nvim_ui_term_event` or, for the modes and sizes
//! the TUI cares about itself, as a call into the TUI.
//!
//! A sequence can be split across reads. termkey holds the partial one and
//! says `AGAIN`; what happens then is `'ttimeoutlen'`: wait that long for
//! the rest, and if it does not come, take what is there. That is what the
//! timer here is for, and why a lone `<Esc>` takes a moment to arrive.
//!
//! Almost everything here takes a raw `*mut TermInput`. Reading pumps: the
//! terminal's device-attributes reply is what [`tui_stop`] and
//! [`tui_suspend`] wait for, and their callbacks re-enter this struct
//! through the TUI, so a long-lived `&mut` would be a lie. Staging and
//! sending keys never pump, and take references.
//!
//! [`tui_stop`]: super::tui::tui_stop
//! [`tui_suspend`]: super::tui::tui_suspend

#![deny(unsafe_op_in_unsafe_fn)]

use crate::event::libuv::{
    uv_close, uv_timer_get_due_in, uv_timer_init, uv_timer_start, uv_timer_stop,
};
use crate::event::r#loop::loop_schedule_fast;
use crate::event::rstream::{
    rstream_available, rstream_consume, rstream_init_fd, rstream_may_close, rstream_start,
    rstream_stop,
};
use crate::main::{main_loop, os_exit, p_ttimeout, p_ttm, ui_client_channel_id};
use crate::msgpack_rpc::channel::rpc_send_event;
use crate::tui::keys::{KEYMOD_RECOGNIZED, modified_utf8, mouse_event, simple_utf8};
use crate::tui::negotiate::{
    tui_enable_extended_underline, tui_handle_term_mode, tui_query_bg_color,
};
use crate::tui::termkey::driver_csi::{
    csi_param_value, termkey_interpret_csi, termkey_interpret_modereport,
};
use crate::tui::termkey::termkey::{
    TERMKEY_CANON_DELBS, TERMKEY_EVENT_PRESS, TERMKEY_EVENT_REPEAT, TERMKEY_FLAG_KEEPC0,
    TERMKEY_FLAG_NOSTART, TERMKEY_FLAG_UTF8, TERMKEY_RES_AGAIN, TERMKEY_RES_KEY, TERMKEY_TYPE_APC,
    TERMKEY_TYPE_DCS, TERMKEY_TYPE_FUNCTION, TERMKEY_TYPE_KEYSYM, TERMKEY_TYPE_MODEREPORT,
    TERMKEY_TYPE_MOUSE, TERMKEY_TYPE_OSC, TERMKEY_TYPE_UNICODE, TERMKEY_TYPE_UNKNOWN_CSI,
    termkey_destroy, termkey_get_buffer_remaining, termkey_get_buffer_size, termkey_get_canonflags,
    termkey_getkey, termkey_getkey_force, termkey_hook_terminfo_getstr, termkey_interpret_string,
    termkey_new_abstract, termkey_push_bytes, termkey_set_buffer_size, termkey_set_canonflags,
    termkey_start,
};
use crate::tui::tui::tui_set_size;
use crate::types::builders::ArrayBuf;
use crate::types::libc::STDIN_FILENO;
use crate::types::{
    Array, Event, Integer, KEY_BUFFER_SIZE, KeyEncoding, Loop, Object, RStream, String_0,
    TermInput, TermKey, TermKeyCsiParam, TermKeyKey, TerminfoEntry, size_t, uv_handle_t,
    uv_timer_t,
};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::fmt::Write;

// ------------------------------------------------------------------ the state

// -------------------------------------------------------------- the constants

/// termkey's buffer, holding the sequence being parsed. It is grown for a
/// paste-sized burst and shrunk back to this afterwards.
const INPUT_BUFFER_SIZE: usize = 256;

/// Not inside a paste.
pub const PASTE_NONE: i8 = 0;
/// The phases of a paste, as `nvim_paste` numbers them.
const PASTE_FIRST: i8 = 1;
const PASTE_MIDDLE: i8 = 2;
const PASTE_LAST: i8 = 3;
/// A paste that began and ended within one flush: first and last at once.
const PASTE_ONLY: i8 = -1;

/// What a terminal in bracketed-paste mode wraps pasted text in.
const START_PASTE: &[u8; 6] = b"\x1b[200~";
const END_PASTE: &[u8; 6] = b"\x1b[201~";

/// What a terminal reporting focus sends when its window gains or loses it.
const FOCUS_GAINED: &[u8; 3] = b"\x1b[I";
const FOCUS_LOST: &[u8; 3] = b"\x1b[O";

/// The most CSI parameters read out of one sequence.
const MAX_CSI_PARAMS: usize = 16;

/// The terminal speaks the kitty keyboard protocol.
const KEY_ENCODING_KITTY: KeyEncoding = 1;

/// How long to sit on background-colour queries before making another.
const BG_QUERY_DELAY_MS: u64 = 100;

const ESC: u8 = 0x1b;

// -------------------------------------------------------------- the lifecycle

/// Set up the input layer on `loop_0`, parsing keys as `ti` describes them.
///
/// # Safety
/// `input` must point to a zeroed [`TermInput`] that outlives `loop_0`, and
/// `ti` to a terminfo entry that outlives the parser built from it.
pub unsafe fn tinput_init(input: *mut TermInput, loop_0: *mut Loop, ti: *mut TerminfoEntry) {
    // SAFETY: the caller guarantees all three; the timers and the read
    // stream live in `input` itself.
    unsafe {
        assert!((*input).loop_0.is_null(), "input layer initialised twice");
        (*input).loop_0 = loop_0;
        (*input).paste = PASTE_NONE;
        (*input).in_fd = STDIN_FILENO;
        (*input).ttimeout = p_ttimeout.get() != 0;
        (*input).ttimeoutlen = p_ttm.get();
        rstream_init_fd(loop_0, &raw mut (*input).read_stream, (*input).in_fd);

        // UTF-8 because that is all nvim reads; C0 kept because the editor
        // wants the control characters themselves rather than termkey's
        // names for them; NOSTART because starting is separate, below.
        (*input).tk = termkey_new_abstract(
            ti,
            (TERMKEY_FLAG_UTF8 | TERMKEY_FLAG_NOSTART | TERMKEY_FLAG_KEEPC0) as c_int,
        );
        termkey_set_buffer_size((*input).tk, INPUT_BUFFER_SIZE);
        termkey_hook_terminfo_getstr((*input).tk, (*input).tk_ti_hook_fn, input.cast::<c_void>());
        termkey_start((*input).tk);
        // Backspace and delete are one key as far as the editor is
        // concerned, whichever byte this terminal sends for it.
        let canonflags = termkey_get_canonflags((*input).tk);
        termkey_set_canonflags((*input).tk, canonflags | TERMKEY_CANON_DELBS as c_int);

        uv_timer_init(&raw mut (*loop_0).uv, &raw mut (*input).timer_handle);
        (*input).timer_handle.data = input.cast::<c_void>();
        uv_timer_init(&raw mut (*loop_0).uv, &raw mut (*input).bg_query_timer);
        (*input).bg_query_timer.data = input.cast::<c_void>();
    }
}

/// Let go of the terminal and everything read from it.
///
/// # Safety
/// `input` must point to a live [`TermInput`].
pub unsafe fn tinput_destroy(input: *mut TermInput) {
    // SAFETY: the caller guarantees `input`; the handles are its own.
    unsafe {
        uv_close((&raw mut (*input).timer_handle).cast::<uv_handle_t>(), None);
        uv_close(
            (&raw mut (*input).bg_query_timer).cast::<uv_handle_t>(),
            None,
        );
        rstream_may_close(&raw mut (*input).read_stream);
        termkey_destroy((*input).tk);
        (*input).loop_0 = core::ptr::null_mut();
    }
}

/// Start reading.
///
/// # Safety
/// `input` must point to a live [`TermInput`].
pub unsafe fn tinput_start(input: *mut TermInput) {
    // SAFETY: the caller guarantees `input`, which is what the read
    // callback is handed back.
    unsafe {
        rstream_start(
            &raw mut (*input).read_stream,
            Some(tinput_read_cb),
            input.cast::<c_void>(),
        );
    }
}

/// Stop reading, and stop waiting for anything half-read.
///
/// # Safety
/// `input` must point to a live [`TermInput`].
pub unsafe fn tinput_stop(input: *mut TermInput) {
    // SAFETY: the caller guarantees `input`.
    unsafe {
        rstream_stop(&raw mut (*input).read_stream);
        uv_timer_stop(&raw mut (*input).timer_handle);
        uv_timer_stop(&raw mut (*input).bg_query_timer);
    }
}

// ----------------------------------------------------------------- sending on

/// Send everything staged to the editor, as a paste or as typed keys.
fn tinput_flush(input: &mut TermInput) {
    let keys = String_0::from_raw_parts(
        input.key_buffer.as_mut_ptr().cast::<c_char>(),
        input.key_buffer_len,
    );
    if input.paste != PASTE_NONE {
        let mut args = ArrayBuf::<3>::new();
        args.push(Object::string(keys));
        // Pasted text always keeps its line structure, whatever mode the
        // editor is in.
        args.push(Object::boolean(true));
        args.push(Object::integer(Integer::from(input.paste)));
        send(c"nvim_paste", args.array());
        // Whatever comes next belongs to the same paste.
        if input.paste == PASTE_FIRST {
            input.paste = PASTE_MIDDLE;
        }
    } else if input.key_buffer_len > 0 {
        let mut args = ArrayBuf::<1>::new();
        args.push(Object::string(keys));
        send(c"nvim_input", args.array());
    }
    input.key_buffer_len = 0;
}

/// Stage `keys` for the editor.
///
/// They go out when the buffer fills or the read ends, so a burst of input
/// costs one message rather than one per key.
fn tinput_enqueue(input: &mut TermInput, keys: &[u8]) {
    if input.key_buffer_len + keys.len() > KEY_BUFFER_SIZE {
        tinput_flush(input);
    }
    // What still does not fit is dropped rather than split. Nothing this
    // layer produces is that long: a read hands over at most a buffer's
    // worth, and a named key is a few dozen bytes.
    let room = KEY_BUFFER_SIZE - input.key_buffer_len;
    let taken = keys.len().min(room);
    let end = input.key_buffer_len + taken;
    input.key_buffer[input.key_buffer_len..end].copy_from_slice(&keys[..taken]);
    input.key_buffer_len = end;
}

/// Send an event to the editor over the UI client's channel.
fn send(name: &CStr, args: Array) {
    // SAFETY: the array borrows the caller's frame, and `rpc_send_event`
    // serialises it rather than taking it over.
    unsafe { rpc_send_event(ui_client_channel_id.get(), name.as_ptr(), args) };
}

/// Tell the editor what the terminal said, verbatim.
fn send_term_event(response: &mut [u8]) {
    let mut args = ArrayBuf::<2>::new();
    args.push(Object::literal("termresponse"));
    args.push(Object::string(String_0::from_raw_parts(
        response.as_mut_ptr().cast::<c_char>(),
        response.len(),
    )));
    send(c"nvim_ui_term_event", args.array());
}

// -------------------------------------------------------------------- reading

/// The read callback: take what arrived, and ask for the rest.
///
/// # Safety
/// Called by the read stream with the [`TermInput`] it was started with.
unsafe fn tinput_read_cb(
    _stream: *mut RStream,
    buf: *const c_char,
    count: size_t,
    data: *mut c_void,
    eof: bool,
) -> size_t {
    let input: *mut TermInput = data.cast();
    // SAFETY: the stream holds this input layer's own pointer, and `buf`
    // with `count` describe the stream's own buffer.
    unsafe {
        let consumed = handle_raw_buffer(input, false, bytes(buf, count));
        tinput_flush(&mut *input);
        if eof {
            // The terminal is gone; there is nothing to go on reading. The
            // exit runs on the editor's loop rather than here, so this read
            // finishes first.
            loop_schedule_fast(
                main_loop.ptr(),
                Event {
                    handler: Some(tinput_done_event),
                    argv: [core::ptr::null_mut(); 10],
                },
            );
            return consumed;
        }
        // Bytes left over are an incomplete sequence: give the rest of it
        // `'ttimeoutlen'` to arrive.
        if consumed < count {
            let ms: i64 = if (*input).ttimeout && (*input).ttimeoutlen >= 0 {
                (*input).ttimeoutlen
            } else {
                0
            };
            uv_timer_stop(&raw mut (*input).timer_handle);
            uv_timer_start(
                &raw mut (*input).timer_handle,
                Some(tinput_timer_cb),
                u64::from(ms as u32),
                0,
            );
        }
        consumed
    }
}

/// The end of input, once the editor's loop gets to it.
///
/// # Safety
/// Called by the event loop, which passes arguments this takes none of.
unsafe extern "C" fn tinput_done_event(_argv: *mut *mut c_void) {
    // SAFETY: exiting takes the process down; there is nothing to unwind.
    unsafe { os_exit(1) };
}

/// An incomplete sequence has waited long enough: take it as it stands.
///
/// # Safety
/// Called by libuv with the timer this input layer started.
unsafe extern "C" fn tinput_timer_cb(handle: *mut uv_timer_t) {
    // SAFETY: the timer's `data` is the input layer that started it, and
    // the read stream's unread bytes are its own buffer.
    unsafe {
        let input: *mut TermInput = (*handle).data.cast();
        let available = rstream_available(&raw mut (*input).read_stream);
        if available > 0 {
            let unread = bytes((*input).read_stream.read_pos, available);
            let consumed = handle_raw_buffer(input, true, unread);
            rstream_consume(&raw mut (*input).read_stream, consumed);
        }
        tk_getkeys(input, true);
        tinput_flush(&mut *input);
    }
}

/// Time to ask the terminal for its background colour again.
///
/// # Safety
/// Called by libuv with the timer this input layer started.
unsafe extern "C" fn bg_query_timer_cb(handle: *mut uv_timer_t) {
    // SAFETY: the timer's `data` is the input layer that started it, and it
    // knows the TUI it belongs to.
    unsafe {
        let input: *mut TermInput = (*handle).data.cast();
        tui_query_bg_color((*input).tui_data);
    }
}

/// Take what can be taken out of `data`, and say how much that was.
///
/// Bytes are handed on a sequence at a time, so that a paste bracket or a
/// focus event arriving mid-stream is recognised before the parser swallows
/// it. `force` skips those two checks: it is the timeout path, where what is
/// left is known not to be either.
///
/// # Safety
/// `input` must point to a live [`TermInput`].
unsafe fn handle_raw_buffer(input: *mut TermInput, force: bool, data: &[u8]) -> usize {
    let mut rest = data;
    // SAFETY: the caller guarantees `input`. `data` is the read stream's
    // buffer, which is not part of this struct.
    unsafe {
        loop {
            let mut consumed = 0;
            if !force {
                consumed = handle_focus_event(rest);
                if consumed == 0 {
                    let (taken, incomplete) = handle_bracketed_paste(&mut *input, rest);
                    if incomplete {
                        // Half a paste bracket: leave it for the next read
                        // to complete rather than typing it into the editor.
                        return data.len() - rest.len();
                    }
                    consumed = taken;
                }
            }
            if consumed == 0 {
                consumed = take_one_sequence(&mut *input, rest);
            }
            rest = &rest[consumed..];
            if rest.is_empty() {
                break;
            }
        }
        shrink_parser_buffer((*input).tk);
        data.len() - rest.len()
    }
}

/// Hand over the next sequence in `rest`, returning how many bytes went.
///
/// Everything up to the next escape belongs to one sequence, since an escape
/// can only begin one. Mid-paste there is nothing to parse and the bytes go
/// to the editor as they are.
///
/// # Safety
/// `input` must point to a live [`TermInput`].
unsafe fn take_one_sequence(input: &mut TermInput, rest: &[u8]) -> usize {
    let count = rest
        .iter()
        .skip(1)
        .position(|&byte| byte == ESC)
        .map_or(rest.len(), |offset| offset + 1);
    if input.paste != PASTE_NONE {
        tinput_enqueue(input, &rest[..count]);
        return count;
    }
    // SAFETY: `tk` is this input layer's parser, and the bytes handed to it
    // are the caller's, read and not kept.
    unsafe {
        grow_parser_buffer(input.tk, count);
        let pushed = termkey_push_bytes(input.tk, rest.as_ptr().cast::<c_char>(), count);
        assert!(pushed <= count, "the parser took more than it was given");
        tk_getkeys(input, false);
        pushed
    }
}

/// Make room in the parser's buffer for `wanted` more bytes.
///
/// # Safety
/// `tk` must be a live parser.
unsafe fn grow_parser_buffer(tk: *mut TermKey, wanted: usize) {
    // SAFETY: the caller guarantees `tk`.
    unsafe {
        let remaining = termkey_get_buffer_remaining(tk);
        if wanted <= remaining {
            return;
        }
        let size = termkey_get_buffer_size(tk);
        let grown = (size + (wanted - remaining)).max(size * 2);
        assert!(
            termkey_set_buffer_size(tk, grown) != 0,
            "out of memory growing the key parser's buffer"
        );
    }
}

/// Give back the room a burst of input needed.
///
/// # Safety
/// `tk` must be a live parser.
unsafe fn shrink_parser_buffer(tk: *mut TermKey) {
    // SAFETY: the caller guarantees `tk`.
    unsafe {
        let size = termkey_get_buffer_size(tk);
        let used = size - termkey_get_buffer_remaining(tk);
        if used < INPUT_BUFFER_SIZE && size > INPUT_BUFFER_SIZE {
            assert!(
                termkey_set_buffer_size(tk, INPUT_BUFFER_SIZE) != 0,
                "out of memory shrinking the key parser's buffer"
            );
        }
    }
}

/// The bytes at `ptr`, or none at all.
///
/// # Safety
/// `ptr` must be valid for reads of `len` bytes.
unsafe fn bytes<'a>(ptr: *const c_char, len: usize) -> &'a [u8] {
    if len == 0 {
        // A length of zero says nothing about the pointer, and
        // `from_raw_parts` will not take a null one even so.
        return &[];
    }
    // SAFETY: the caller guarantees the pointer and the length.
    unsafe { core::slice::from_raw_parts(ptr.cast::<u8>(), len) }
}

// ----------------------------------------------------------------- whole keys

/// Send on every key the parser has finished.
///
/// `force` takes an incomplete sequence as whatever it already spells, which
/// is how a lone `<Esc>` eventually arrives.
///
/// # Safety
/// `input` must point to a live [`TermInput`].
unsafe fn tk_getkeys(input: *mut TermInput, force: bool) {
    // SAFETY: the caller guarantees `input`; `key` is this frame's, and a
    // zeroed one is a valid unicode key carrying no modifiers.
    unsafe {
        let mut key: TermKeyKey = core::mem::zeroed();
        let result = loop {
            let result = if force {
                termkey_getkey_force((*input).tk, &raw mut key)
            } else {
                termkey_getkey((*input).tk, &raw mut key)
            };
            if result != TERMKEY_RES_KEY {
                break result;
            }
            // A key going up is not something the editor has a notion of.
            if key.event == TERMKEY_EVENT_PRESS || key.event == TERMKEY_EVENT_REPEAT {
                dispatch_key(input, &key);
            }
        };
        if result != TERMKEY_RES_AGAIN {
            return;
        }
        // Half a sequence: either it is worth waiting for the rest, or
        // `'ttimeout'` says to take it as it is straight away.
        if (*input).ttimeout && (*input).ttimeoutlen >= 0 {
            uv_timer_stop(&raw mut (*input).timer_handle);
            uv_timer_start(
                &raw mut (*input).timer_handle,
                Some(tinput_timer_cb),
                (*input).ttimeoutlen as u64,
                0,
            );
        } else {
            tk_getkeys(input, true);
        }
    }
}

/// Send `key` where it belongs: to the editor, or to the TUI.
///
/// # Safety
/// `input` must point to a live [`TermInput`], and `key` must have come from
/// its parser.
unsafe fn dispatch_key(input: *mut TermInput, key: &TermKeyKey) {
    // SAFETY: the caller guarantees both.
    unsafe {
        let tk = (*input).tk;
        match key.type_0 {
            TERMKEY_TYPE_UNICODE if key.modifiers & KEYMOD_RECOGNIZED == 0 => {
                tinput_enqueue(&mut *input, simple_utf8(key).as_bytes());
            }
            TERMKEY_TYPE_UNICODE | TERMKEY_TYPE_FUNCTION | TERMKEY_TYPE_KEYSYM => {
                tinput_enqueue(&mut *input, modified_utf8(tk, key).as_bytes());
            }
            TERMKEY_TYPE_MOUSE => {
                if let Some(text) = mouse_event(tk, key) {
                    tinput_enqueue(&mut *input, text.as_bytes());
                }
            }
            TERMKEY_TYPE_MODEREPORT => handle_modereport(input, key),
            TERMKEY_TYPE_UNKNOWN_CSI => handle_unknown_csi(input, key),
            TERMKEY_TYPE_OSC | TERMKEY_TYPE_DCS | TERMKEY_TYPE_APC => {
                handle_term_response(input, key);
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------- what is not a key

/// The terminal telling us its window gained or lost focus.
///
/// Returns how many bytes that took, or zero if this is not one.
fn handle_focus_event(data: &[u8]) -> usize {
    let Some(head) = data.get(..FOCUS_GAINED.len()) else {
        return 0;
    };
    if head != FOCUS_GAINED && head != FOCUS_LOST {
        return 0;
    }
    let mut args = ArrayBuf::<1>::new();
    args.push(Object::boolean(head == FOCUS_GAINED));
    send(c"nvim_ui_set_focus", args.array());
    FOCUS_GAINED.len()
}

/// A paste bracket: everything between the two is pasted rather than typed.
///
/// Returns how many bytes the bracket took, and whether what is there is the
/// beginning of one whose rest has not arrived yet.
fn handle_bracketed_paste(input: &mut TermInput, data: &[u8]) -> (usize, bool) {
    let Some(head) = data.get(..START_PASTE.len()) else {
        // Too short to be a whole bracket: either the beginning of one, and
        // worth waiting for, or ordinary input.
        let incomplete = START_PASTE.starts_with(data) || END_PASTE.starts_with(data);
        return (0, incomplete);
    };
    if head != START_PASTE && head != END_PASTE {
        return (0, false);
    }
    let starting = head == START_PASTE;
    let pasting = input.paste != PASTE_NONE;
    if pasting && starting {
        // A paste bracket inside a paste is text: whatever is being pasted
        // contains one, and it goes through untouched.
        return (0, false);
    }
    if pasting == starting {
        // An end with no beginning: swallow it and carry on.
        return (START_PASTE.len(), false);
    }
    if starting {
        // What was typed before the paste belongs to the editor, not to it.
        tinput_flush(input);
        input.paste = PASTE_FIRST;
    } else {
        // A paste short enough never to have been flushed was never
        // announced, so it is the only chunk rather than the last one.
        input.paste = if input.paste == PASTE_MIDDLE {
            PASTE_LAST
        } else {
            PASTE_ONLY
        };
        tinput_flush(input);
        input.paste = PASTE_NONE;
    }
    (START_PASTE.len(), false)
}

/// The terminal answering a question about one of its modes.
///
/// # Safety
/// `input` must point to a live [`TermInput`], and `key` must have come from
/// its parser.
unsafe fn handle_modereport(input: *mut TermInput, key: &TermKeyKey) {
    // SAFETY: the caller guarantees both; the out-parameters are this
    // frame's.
    unsafe {
        let (mut initial, mut mode, mut value) = (0, 0, 0);
        let result = termkey_interpret_modereport(
            (*input).tk,
            key,
            &raw mut initial,
            &raw mut mode,
            &raw mut value,
        );
        if result == TERMKEY_RES_KEY {
            tui_handle_term_mode((*input).tui_data, mode as _, value as _);
        }
    }
}

/// Anything else the terminal says in a CSI sequence.
///
/// # Safety
/// `input` must point to a live [`TermInput`], and `key` must have come from
/// its parser.
unsafe fn handle_unknown_csi(input: *mut TermInput, key: &TermKeyKey) {
    // SAFETY: the caller guarantees both; the parameter array is this
    // frame's, and termkey fills in no more of it than it says it has.
    unsafe {
        let mut params = [TermKeyCsiParam {
            param: core::ptr::null(),
            length: 0,
        }; MAX_CSI_PARAMS];
        let mut nparams = MAX_CSI_PARAMS;
        let mut cmd: core::ffi::c_uint = 0;
        let result = termkey_interpret_csi(
            (*input).tk,
            key,
            params.as_mut_ptr(),
            &raw mut nparams,
            &raw mut cmd,
        );
        if result != TERMKEY_RES_KEY {
            return;
        }
        let params = &params[..nparams];
        let initial = (cmd >> 8 & 0xff) as u8;
        let command = (cmd & 0xff) as u8;
        match (command, initial) {
            // The terminal supports the kitty keyboard protocol, which the
            // TUI has by now asked it to speak.
            (b'u', b'?') => (*input).key_encoding = KEY_ENCODING_KITTY,
            (b'c', b'?') => handle_primary_device_attr(input, params),
            // A resize the terminal is reporting itself.
            (b't', _) if params.len() == 5 => {
                let args: [c_int; 3] = core::array::from_fn(|i| csi_param_value(params[i]));
                if args[0] == REPORT_SIZE_IN_CELLS {
                    let (height, width) = (args[1], args[2]);
                    tui_set_size((*input).tui_data, width, height);
                }
            }
            (b'n', _) if params.len() == 1 => {
                let mut response = Vec::new();
                let _ = write!(Bytes(&mut response), "\x1b[{}n", csi_param_value(params[0]));
                send_term_event(&mut response);
            }
            (b'n', _) if params.len() == 2 => {
                let args: [c_int; 2] = core::array::from_fn(|i| csi_param_value(params[i]));
                // The terminal's colour scheme changed. Terminals send a
                // flurry of these, so the query it provokes is rationed.
                if args[0] == THEME_CHANGED
                    && uv_timer_get_due_in(&raw mut (*input).bg_query_timer) == 0
                {
                    uv_timer_start(
                        &raw mut (*input).bg_query_timer,
                        Some(bg_query_timer_cb),
                        BG_QUERY_DELAY_MS,
                        0,
                    );
                }
            }
            _ => {}
        }
    }
}

/// The first parameter of the resize report that carries a size in cells,
/// as opposed to one in pixels or a window-state change.
const REPORT_SIZE_IN_CELLS: c_int = 48;

/// The first parameter of the report a terminal sends when its colour
/// scheme changes.
const THEME_CHANGED: c_int = 997;

/// The terminal's device attributes: what it is and what it can do.
///
/// # Safety
/// `input` must point to a live [`TermInput`], and `params` must be the
/// parameters of a device-attributes reply.
unsafe fn handle_primary_device_attr(input: *mut TermInput, params: &[TermKeyCsiParam]) {
    // SAFETY: the caller guarantees both. The callback is taken before it
    // is called: it runs once, and what it does reaches back here.
    unsafe {
        if let Some(callback) = (*input).callbacks.primary_device_attr.take() {
            callback((*input).tui_data);
        }
        if params.is_empty() {
            return;
        }
        let mut response = Vec::new();
        response.extend_from_slice(b"\x1b[?");
        for (i, &param) in params.iter().enumerate() {
            if i > 0 {
                response.push(b';');
            }
            let _ = write!(Bytes(&mut response), "{}", csi_param_value(param));
        }
        response.push(b'c');
        send_term_event(&mut response);
    }
}

/// Everything the terminal says outside a CSI sequence: the answers to
/// colour and capability queries, and whatever else it volunteers.
///
/// # Safety
/// `input` must point to a live [`TermInput`], and `key` must have come from
/// its parser.
unsafe fn handle_term_response(input: *mut TermInput, key: &TermKeyKey) {
    // SAFETY: the caller guarantees both; the string termkey hands back
    // borrows its buffer, and is read before anything else is parsed.
    unsafe {
        let mut answer = core::ptr::null::<c_char>();
        if termkey_interpret_string((*input).tk, key, &raw mut answer) != TERMKEY_RES_KEY {
            return;
        }
        assert!(
            !answer.is_null(),
            "the parser reported a string it does not have"
        );
        let text = CStr::from_ptr(answer).to_bytes();

        // A terminal that answers the underline-style query with a style
        // has extended underlines, whatever its terminfo entry says.
        if key.type_0 == TERMKEY_TYPE_DCS
            && (text.starts_with(b"1$r4:3m") || text.starts_with(b"1$r0;4:3m"))
        {
            tui_enable_extended_underline((*input).tui_data);
        }

        // The editor is told what was said, introducer and all, so that a
        // plugin can recognise the answer to its own query.
        let introducer: &[u8] = match key.type_0 {
            TERMKEY_TYPE_OSC => b"\x1b]",
            TERMKEY_TYPE_DCS => b"\x1bP",
            TERMKEY_TYPE_APC => b"\x1b_",
            _ => unreachable!("not a key carrying a string"),
        };
        let mut response = Vec::with_capacity(introducer.len() + text.len());
        response.extend_from_slice(introducer);
        response.extend_from_slice(text);
        send_term_event(&mut response);
    }
}

/// [`core::fmt`] over a byte vector, for building a response out of numbers.
struct Bytes<'a>(&'a mut Vec<u8>);

impl Write for Bytes<'_> {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        self.0.extend_from_slice(text.as_bytes());
        Ok(())
    }
}
