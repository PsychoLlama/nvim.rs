//! Escape sequences the child program sent that vterm did not recognise.
//!
//! OSC, DCS and APC strings arrive as fragments — vterm hands over whatever
//! part of the sequence it has parsed so far, flagged `initial` and/or
//! `final`. This module reassembles them into
//! [`Terminal::termrequest_buffer`](crate::types::Terminal) and
//! reports the finished sequence to `TermRequest` autocommands.
//!
//! Reporting is deferred. The fragments arrive while vterm is parsing the
//! child's output, which is far too deep to run Vimscript from, so the
//! finished sequence is queued as a [`TermRequest`] on the main loop and
//! reported once the refresh has caught up. Anything the handler writes back
//! to the child is held in [`TermRequest::pending_send`] until reporting is
//! done, so that a reply cannot overtake the request that prompted it.
//!
//! OSC 8 is the exception: hyperlinks are a display attribute, so they are
//! applied to vterm's pen immediately rather than waiting for the queue.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::autocmd::{EVENT_TERMREQUEST, apply_autocmds_group, has_event};
use crate::channel::main_loop_events;
use crate::eval::vars::set_vim_var_string;
use crate::event::multiqueue::multiqueue_put_event;
use crate::highlight::hl_add_url;
use crate::types::builders::{ArrayBuf, DictBuf};
use crate::types::{
    Event, Object, String_0, VTermStateFallbacks, VTermStringFragment, VTermTerminator, VTermValue,
    Vv, exarg_T, handle_T, ptrdiff_t, size_t,
};
use crate::vterm::pen::set_pen_attr;
use crate::winlayer::Buf;
use core::ffi::{CStr, c_char, c_int, c_void};

use super::{AUGROUP_ALL, Term, buf_for_handle, row_to_linenr, terminal_send};
use crate::vterm::vterm::{VTERM_ATTR_URI, VTERM_TERMINATOR_BEL, VTERM_VALUETYPE_INT};

/// The sequences vterm hands over rather than acting on itself.
///
/// Only the string-carrying ones are taken: a control or CSI sequence vterm
/// does not know is not something an autocommand could make sense of
/// either.
pub static FALLBACKS: VTermStateFallbacks = VTermStateFallbacks {
    control: None,
    csi: None,
    osc: Some(on_osc),
    dcs: Some(on_dcs),
    apc: Some(on_apc),
    pm: None,
    sos: None,
};

/// A finished escape sequence, waiting on the main loop to be reported.
///
/// Owned by the queued event: [`schedule_termrequest`] leaks the box and
/// [`emit_termrequest`] reclaims it, possibly after re-queueing itself once.
pub struct TermRequest {
    /// The buffer rather than the terminal: by the time this runs the
    /// terminal may have been destroyed, and a handle can be checked.
    buf_handle: handle_T,
    /// The sequence as the child sent it, terminator excluded.
    sequence: Vec<u8>,
    /// Cursor position when the sequence arrived, in buffer coordinates.
    line: c_int,
    col: c_int,
    /// Scrollback rows evicted as of then. Rows evicted since shift `line`.
    sb_deleted: usize,
    terminator: VTermTerminator,
    /// Writes to the child made while the handler runs. See
    /// [`TerminalPending::send`](crate::types::TerminalPending).
    pending_send: Vec<u8>,
}

/// Report `request` to `TermRequest` autocommands, or drop it if the
/// terminal it came from is gone.
///
/// Deferred a second time when the refresh still owes the buffer scrollback
/// rows: the reported cursor line is a buffer line number, and appending
/// those rows is what makes it correct.
unsafe extern "C" fn emit_termrequest(argv: *mut *mut c_void) {
    // SAFETY: the event carries the request `schedule_termrequest` leaked,
    // and this is the only thing that reclaims it.
    let mut request = unsafe { Box::from_raw(*argv as *mut TermRequest) };
    let Some(buf) = buf_for_handle(request.buf_handle).filter(|buf| !buf.terminal.is_null()) else {
        return;
    };
    // SAFETY: a buffer that still has its terminal.
    let term = unsafe { Term::new(buf.terminal) };
    if term.sb.pending() > 0 {
        let events = term.pending.events;
        let event = Event::new(Some(emit_termrequest), [Box::into_raw(request).cast()]);
        // SAFETY: the terminal's own queue, drained by the refresh.
        unsafe { multiqueue_put_event(events, event) };
        return;
    }
    report(&mut request, term, buf);
}

/// The body of [`emit_termrequest`] once the terminal is known to be alive.
fn report(request: &mut TermRequest, mut term: Term, buf: Buf) {
    let sequence = String_0::from_raw_parts(
        request.sequence.as_ptr().cast::<c_char>().cast_mut(),
        request.sequence.len(),
    );
    let (data, size) = (sequence.data(), sequence.len() as ptrdiff_t);
    // SAFETY: `v:termrequest` takes a string of `size` readable bytes,
    // which it copies.
    unsafe { set_vim_var_string(Vv::Termrequest, data, size) };

    // Rows evicted since the sequence arrived have shifted every buffer
    // line up by one.
    let scrolled = (term.sb.deleted() - request.sb_deleted) as i64;
    let mut cursor = ArrayBuf::<2>::new();
    cursor.push(Object::integer(request.line as i64 - scrolled));
    cursor.push(Object::integer(request.col as i64));

    let mut data = DictBuf::<3>::new();
    data.insert(c"sequence", Object::string(sequence));
    data.insert(c"cursor", cursor.object());
    data.insert(
        c"terminator",
        Object::literal(if request.terminator == VTERM_TERMINATOR_BEL {
            "\x07"
        } else {
            "\x1b\\"
        }),
    );

    // The handler can close the terminal; hold it open across the call so
    // the writes below still have somewhere to go.
    term.refcount += 1;
    // Pre-bound so that the eight-argument call still fits on one line.
    let mut event = data.object();
    let (data, none) = (&mut event, ::core::ptr::null_mut());
    let (exarg, group) = (::core::ptr::null_mut::<exarg_T>(), AUGROUP_ALL);
    let buf = buf.raw();
    // SAFETY: TermRequest against a live buffer; nothing of the terminal is
    // borrowed across it.
    unsafe { apply_autocmds_group(EVENT_TERMREQUEST, none, none, true, group, buf, exarg, data) };
    term.refcount -= 1;

    // Let writes through again before flushing what the handler wrote, or
    // it would be appended to the buffer it is being read from.
    let held = term.pending.send;
    term.pending.send = ::core::ptr::null_mut();
    if !request.pending_send.is_empty() {
        terminal_send(term, &request.pending_send);
        request.pending_send.clear();
    }
    // A handler that produced a request of its own left a newer buffer in
    // place; that one is still filling.
    if !::core::ptr::eq(held, &raw mut request.pending_send) {
        term.pending.send = held;
    }

    if term.buf_handle == 0 && term.refcount == 0 {
        term.destroy = true;
        // Read out before the call: the channel's close callback is free to
        // free the terminal.
        let (close_cb, data) = (term.opts.close_cb, term.opts.data);
        // SAFETY: the callback the channel registered, taking the data it
        // registered with it.
        unsafe { close_cb.expect("non-null function pointer")(data) };
    }
}

/// Queue the sequence assembled so far for reporting on the main loop.
pub fn schedule_termrequest(mut term: Term) {
    let request = Box::into_raw(Box::new(TermRequest {
        buf_handle: term.buf_handle,
        sequence: term.termrequest_buffer.clone(),
        line: row_to_linenr(term, term.cursor.row),
        col: term.cursor.col,
        sb_deleted: term.sb.deleted(),
        terminator: term.termrequest_terminator,
        pending_send: Vec::new(),
    }));
    // Valid until emit_termrequest drops the box, and that is the last
    // thing it does.
    //
    // SAFETY: the box just allocated, reachable from nowhere else yet.
    term.pending.send = unsafe { &raw mut (*request).pending_send };
    let event = Event::new(Some(emit_termrequest), [request.cast()]);
    // SAFETY: the main loop's queue, live from startup to exit.
    unsafe { multiqueue_put_event(main_loop_events(), event) };
}

/// The bytes of a fragment vterm handed over.
///
/// # Safety
/// `frag.str` must point at `frag.len()` readable bytes, as vterm's
/// contract for a fragment callback promises.
unsafe fn fragment_bytes(frag: &VTermStringFragment) -> &[u8] {
    if frag.str.is_null() || frag.len() == 0 {
        return &[];
    }
    unsafe { ::core::slice::from_raw_parts(frag.str.cast::<u8>(), frag.len()) }
}

/// Start or continue reassembling a sequence. `prefix` is what vterm ate
/// before handing the payload over, and is re-emitted so that what the
/// autocommand sees is what the child sent.
///
/// # Safety
/// `frag` must be a fragment vterm handed over, as [`fragment_bytes`] wants.
unsafe fn accumulate(mut term: Term, frag: &VTermStringFragment, prefix: &[u8]) {
    if frag.initial() {
        term.termrequest_buffer.clear();
        term.termrequest_buffer.extend_from_slice(prefix);
    }
    // SAFETY: the caller's promise.
    let bytes = unsafe { fragment_bytes(frag) };
    term.termrequest_buffer.extend_from_slice(bytes);
    if frag.final_0() {
        term.termrequest_terminator = frag.terminator;
    }
}

/// `\x1b]8;<params>;<uri>` — a hyperlink, as a highlight attribute id.
///
/// Returns `None` for a payload with no parameter list at all, which is
/// malformed and leaves the pen alone; `Some(0)` for an empty URI, which is
/// how a link is ended.
fn parse_osc8(payload: &[u8]) -> Option<c_int> {
    // C read this out of a NUL-terminated buffer, so an embedded NUL ends
    // the payload before any separator that follows it.
    let payload = match payload.iter().position(|&byte| byte == 0) {
        Some(end) => &payload[..end],
        None => payload,
    };
    let uri = &payload[payload.iter().position(|&byte| byte == b';')? + 1..];
    if uri.is_empty() {
        return Some(0);
    }
    let mut terminated = uri.to_vec();
    terminated.push(0);
    let uri = CStr::from_bytes_with_nul(&terminated).expect("NUL-terminated, no interior NUL");
    // SAFETY: `uri` is NUL-terminated and outlives the call, which copies.
    Some(unsafe { hl_add_url(0, uri.as_ptr()) })
}

/// Apply a finished OSC 8 to vterm's pen, so that the cells written after
/// it carry the link.
fn apply_osc8(term: Term) {
    let buffer: &[u8] = &term.termrequest_buffer;
    // Past the "\x1b]8;" that `accumulate` put back.
    let Some(attr) = buffer.get(b"\x1b]8;".len()..).and_then(parse_osc8) else {
        return;
    };
    let state = term.state();
    let value = VTermValue { number: attr };
    // SAFETY: the emulator's own state machine, and a value read through
    // the arm the type names.
    unsafe { set_pen_attr(&mut *state.0, VTERM_ATTR_URI, VTERM_VALUETYPE_INT, &value) };
}

pub unsafe extern "C" fn on_osc(
    command: c_int,
    frag: VTermStringFragment,
    user: *mut c_void,
) -> c_int {
    // SAFETY: vterm hands back the terminal registered alongside this
    // fallback table.
    let term = unsafe { Term::new(user.cast()) };
    if frag.str.is_null() || frag.len() == 0 {
        return 0;
    }
    // OSC 8 is handled here whether or not anyone is listening.
    if command != 8 && !listening() {
        return 1;
    }
    // SAFETY: a fragment vterm handed over.
    unsafe { accumulate(term, &frag, format!("\x1b]{command};").as_bytes()) };
    if frag.final_0() {
        if listening() {
            schedule_termrequest(term);
        }
        if command == 8 {
            apply_osc8(term);
        }
    }
    1
}

pub unsafe extern "C" fn on_dcs(
    command: *const c_char,
    commandlen: size_t,
    frag: VTermStringFragment,
    user: *mut c_void,
) -> c_int {
    // SAFETY: as in `on_osc`.
    let term = unsafe { Term::new(user.cast()) };
    if command.is_null() || frag.str.is_null() {
        return 0;
    }
    if !listening() {
        return 1;
    }
    let mut prefix = b"\x1bP".to_vec();
    // SAFETY: vterm's own command name, `commandlen` bytes of it.
    let name = unsafe { ::core::slice::from_raw_parts(command.cast::<u8>(), commandlen) };
    prefix.extend_from_slice(name);
    // SAFETY: a fragment vterm handed over.
    unsafe { accumulate(term, &frag, &prefix) };
    if frag.final_0() {
        schedule_termrequest(term);
    }
    1
}

pub unsafe extern "C" fn on_apc(frag: VTermStringFragment, user: *mut c_void) -> c_int {
    // SAFETY: as in `on_osc`.
    let term = unsafe { Term::new(user.cast()) };
    if frag.str.is_null() || frag.len() == 0 {
        return 0;
    }
    if !listening() {
        return 1;
    }
    // SAFETY: a fragment vterm handed over.
    unsafe { accumulate(term, &frag, b"\x1b_") };
    if frag.final_0() {
        schedule_termrequest(term);
    }
    1
}

/// Whether any autocommand is waiting for a `TermRequest`.
fn listening() -> bool {
    // SAFETY: reads the editor's own event table.
    unsafe { has_event(EVENT_TERMREQUEST) }
}
