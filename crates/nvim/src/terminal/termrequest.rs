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

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use std::cell::RefCell;
use std::rc::Rc;

use crate::autocmd::{apply_autocmds_group, has_event};
use crate::eval::vars::set_vim_var_string;
use crate::event::objects::Event;
use crate::event_queue::{self, ParkingQueue, QueueId};
use crate::highlight::hl_add_url;
use crate::types::AutoEvent;
use crate::types::builders::{ArrayBuf, DictBuf};
use crate::types::{ApiString, Handle, Object, RefcountSize, VTermTerminator, Vv};
use crate::vterm::host::{Fallbacks, Fragment, Value};
use crate::vterm::pen::set_pen_attr;
use crate::vterm::state::StateCx;
use crate::winlayer::Buf;
use core::ffi::{CStr, c_int};

use super::{AUGROUP_ALL, Term, buf_for_handle, row_to_linenr, terminal_send};
use crate::vterm::vterm::{VTERM_ATTR_URI, VTERM_TERMINATOR_BEL};

/// The sequences vterm hands over rather than acting on itself, for the
/// terminal they arrive at.
///
/// Only the string-carrying ones are taken: a control or CSI sequence vterm
/// does not know is not something an autocommand could make sense of
/// either.
pub(crate) struct Requests(pub(crate) Term);

impl Fallbacks for Requests {
    fn osc(
        &mut self,
        command: c_int,
        frag: Fragment<'_>,
        state: &mut StateCx<'_>,
    ) -> Option<c_int> {
        Some(on_osc(self.0, command, frag, state))
    }

    fn dcs(&mut self, command: &[u8], frag: Fragment<'_>) -> Option<c_int> {
        Some(on_dcs(self.0, command, frag))
    }

    fn apc(&mut self, frag: Fragment<'_>) -> Option<c_int> {
        Some(on_apc(self.0, frag))
    }
}

/// A finished escape sequence, waiting on the main loop to be reported.
///
/// Owned by the queued event, which hands it to [`emit_termrequest`],
/// possibly after re-queueing itself once.
pub(crate) struct TermRequest {
    /// The buffer rather than the terminal: by the time this runs the
    /// terminal may have been destroyed, and a handle can be checked.
    buf_handle: Handle,
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
    pending_send: Rc<RefCell<Vec<u8>>>,
}

/// Report `request` to `TermRequest` autocommands, or drop it if the
/// terminal it came from is gone.
///
/// Deferred a second time when the refresh still owes the buffer scrollback
/// rows: the reported cursor line is a buffer line number, and appending
/// those rows is what makes it correct.
///
fn emit_termrequest(mut request: TermRequest) {
    let Some(buf) = buf_for_handle(request.buf_handle) else {
        return;
    };
    let Some(term) = buf.terminal_handle() else {
        return;
    };
    if term.sb.pending() > 0 {
        let events = term.pending.events.as_ref().map(ParkingQueue::id);
        let events = events.expect("a live terminal has its pending queue");
        let event = Event::Job(Box::new(move || emit_termrequest(request)));
        event_queue::put(events, event);
        return;
    }
    report(&mut request, term, buf);
}

/// The body of [`emit_termrequest`] once the terminal is known to be alive.
fn report(request: &mut TermRequest, mut term: Term, buffer: Buf) {
    let sequence = ApiString::from_bytes(&request.sequence);
    set_vim_var_string(Vv::Termrequest, Some(sequence.as_bytes()));

    // Rows evicted since the sequence arrived have shifted every buffer
    // line up by one.
    let scrolled = i64::try_from(term.sb.deleted() - request.sb_deleted)
        .expect("a scrollback row count fits an i64");
    let mut cursor = ArrayBuf::<2>::new();
    cursor.push(Object::integer(i64::from(request.line) - scrolled));
    cursor.push(Object::integer(i64::from(request.col)));

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
    term.refcount.retain();
    // Pre-bound so that the eight-argument call still fits on one line.
    let data = data.object();
    let group = AUGROUP_ALL;
    // TermRequest against a live buffer; nothing of the terminal is
    // borrowed across it.
    let (event, buffer) = (AutoEvent::TermRequest, Some(buffer));
    apply_autocmds_group(event, None, None, true, group, buffer, None, Some(&data));
    term.refcount.release();

    // Let writes through again before flushing what the handler wrote, or
    // it would be appended to the buffer it is being read from.
    let held = term.pending.send.take();
    let written = ::core::mem::take(&mut *request.pending_send.borrow_mut());
    if !written.is_empty() {
        terminal_send(term, &written);
    }
    // A handler that produced a request of its own left a newer buffer in
    // place; that one is still filling.
    if let Some(held) = held
        && !Rc::ptr_eq(&held, &request.pending_send)
    {
        term.pending.send = Some(held);
    }

    if term.buf_handle == Handle::NONE && term.refcount == RefcountSize::ZERO {
        term.destroy = true;
        // Read out before the call: the channel's close callback is free to
        // free the terminal.
        let (close_cb, owner) = (term.opts.close_cb, term.opts.owner);
        close_cb.expect("non-null function pointer")(owner);
    }
}

/// Queue the sequence assembled so far for reporting on the main loop.
pub(crate) fn schedule_termrequest(mut term: Term) {
    let pending_send = Rc::new(RefCell::new(Vec::new()));
    let request = TermRequest {
        buf_handle: term.buf_handle,
        sequence: term.termrequest_buffer.clone(),
        line: row_to_linenr(term, term.cursor.row),
        col: term.cursor.col,
        sb_deleted: term.sb.deleted(),
        terminator: term.termrequest_terminator,
        pending_send: Rc::clone(&pending_send),
    };
    // Shared with the request until `report` flushes it.
    term.pending.send = Some(pending_send);
    let event = Event::Job(Box::new(move || emit_termrequest(request)));
    event_queue::put(QueueId::MAIN, event);
}

/// Start or continue reassembling a sequence. `prefix` is what vterm ate
/// before handing the payload over, and is re-emitted so that what the
/// autocommand sees is what the child sent.
fn accumulate(mut term: Term, frag: &Fragment<'_>, prefix: &[u8]) {
    if frag.initial {
        term.termrequest_buffer.clear();
        term.termrequest_buffer.extend_from_slice(prefix);
    }
    term.termrequest_buffer.extend_from_slice(frag.bytes());
    if frag.last {
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
    Some(hl_add_url(0, uri))
}

/// Apply a finished OSC 8 to vterm's pen, so that the cells written after
/// it carry the link.
fn apply_osc8(term: Term, state: &mut StateCx<'_>) {
    let buffer: &[u8] = &term.termrequest_buffer;
    // Past the "\x1b]8;" that `accumulate` put back.
    let Some(attr) = buffer.get(b"\x1b]8;".len()..).and_then(parse_osc8) else {
        return;
    };
    set_pen_attr(state, VTERM_ATTR_URI, Value::Int(attr));
}

fn on_osc(term: Term, command: c_int, frag: Fragment<'_>, state: &mut StateCx<'_>) -> c_int {
    if frag.bytes().is_empty() {
        return 0;
    }
    // OSC 8 is handled here whether or not anyone is listening.
    if command != 8 && !listening() {
        return 1;
    }
    accumulate(term, &frag, format!("\x1b]{command};").as_bytes());
    if frag.last {
        if listening() {
            schedule_termrequest(term);
        }
        if command == 8 {
            apply_osc8(term, state);
        }
    }
    1
}

fn on_dcs(term: Term, command: &[u8], frag: Fragment<'_>) -> c_int {
    if frag.buffer().is_none() {
        return 0;
    }
    if !listening() {
        return 1;
    }
    let mut prefix = b"\x1bP".to_vec();
    prefix.extend_from_slice(command);
    accumulate(term, &frag, &prefix);
    if frag.last {
        schedule_termrequest(term);
    }
    1
}

fn on_apc(term: Term, frag: Fragment<'_>) -> c_int {
    if frag.bytes().is_empty() {
        return 0;
    }
    if !listening() {
        return 1;
    }
    accumulate(term, &frag, b"\x1b_");
    if frag.last {
        schedule_termrequest(term);
    }
    1
}

/// Whether any autocommand is waiting for a `TermRequest`.
fn listening() -> bool {
    has_event(AutoEvent::TermRequest)
}
