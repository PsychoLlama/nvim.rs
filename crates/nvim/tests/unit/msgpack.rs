//! The streaming side of the RPC decoder, and the width the packer picks.
//!
//! `tests/unit/unpacker.rs` covers `unpack`, which is handed a whole message
//! at once. This is the other half: `Unpacker`, which is fed whatever bytes
//! the socket happened to deliver and has to survive being stopped anywhere
//! in the middle of one.
//!
//! **Not runnable under Miri**, and the reason is worth knowing: the redraw
//! fast path is entered only when `Unpacker::advance` recognises the `redraw`
//! handler, which it does with `ptr::fn_addr_eq`. Miri does not give a
//! function one address across separate reifications, so the comparison is
//! false there and every message here decodes down the ordinary body path
//! with no event to show for it — a silently vacuous pass, not a failure.
//!
//! Ported from `test/unit/msgpack_spec.lua`, which carried two `grid_line`
//! regressions (#25184). Its third case — the width
//! `mpack_pack_number` picks at each signed boundary (#37202) — is pure
//! arithmetic over a token, so it went in beside `pack_number` itself in
//! `mpack/token.rs` and took the unsigned boundaries with it.

#![cfg(not(miri))]

use std::ffi::c_int;

use neovim::types::{GridLineEvent, Unpacker};

/// An unpacker and the bytes it is being fed, of which only a prefix has
/// "arrived".
struct Stream {
    unpacker: Unpacker,
    bytes: Vec<u8>,
    /// How much of `bytes` the unpacker has consumed.
    read: usize,
    /// How much of `bytes` has arrived.
    arrived: usize,
}

impl Stream {
    fn new() -> Stream {
        Stream {
            unpacker: Unpacker::new(),
            bytes: Vec::new(),
            read: 0,
            arrived: 0,
        }
    }

    /// Hand the unpacker `bytes`, pretending only `size` of them have
    /// arrived.
    fn feed(&mut self, bytes: &[u8], size: usize) {
        assert!(size <= bytes.len());
        self.bytes = bytes.to_vec();
        self.read = 0;
        self.arrived = size;
    }

    /// One more byte has arrived.
    fn one_more_byte(&mut self) {
        self.arrived += 1;
    }

    fn advance(&mut self) -> bool {
        let mut input = &self.bytes[self.read..self.arrived];
        let done = self.unpacker.advance(&mut input);
        self.read = self.arrived - input.len();
        done
    }

    /// The parse stage the unpacker stopped at; negative means the stream is
    /// finished and the channel would be closed.
    fn state(&self) -> c_int {
        self.unpacker.state
    }

    /// The `grid_line` event the last `advance` completed, if it did.
    fn event(&self) -> Option<GridLineEvent> {
        self.unpacker
            .has_grid_line_event
            .then_some(self.unpacker.grid_line_event)
    }
}

/// `[kMessageTypeNotification, "redraw", [["grid_line", [2, 0, 0, cells,
/// false]]]]`, with `cells` spelled out by the caller.
fn redraw_grid_line(cells: &[u8]) -> Vec<u8> {
    let mut out = vec![0x93, 0x02];
    out.extend(b"\xa6redraw");
    out.extend([0x91, 0x92]);
    out.extend(b"\xa9grid_line");
    // [2, 0, 0, <cells>, false]
    out.extend([0x95, 0x02, 0x00, 0x00]);
    out.extend_from_slice(cells);
    out.push(0xc2);
    out
}

/// A `grid_line` event is decoded incrementally: the arguments before the
/// cell array are read into the unpacker's own `GridLineEvent`, and each
/// cell as it arrives. Pausing between the cells and the trailing `wrap`
/// used to walk off the end of the partially-filled event (#25184).
#[test]
fn a_grid_line_paused_before_its_wrap_flag_resumes() {
    // One cell: [" ", 0, 77].
    let payload = redraw_grid_line(&[0x91, 0x93, 0xa1, b' ', 0x00, 0x4d]);
    let mut stream = Stream::new();

    stream.feed(&payload, payload.len() - 1);
    assert!(!stream.advance(), "the wrap flag has not arrived yet");

    stream.one_more_byte();
    assert!(stream.advance(), "the event completes on the last byte");
    let event = stream.event().expect("a completed grid_line");
    assert_eq!(event.args, [2, 0, 0]);
    assert_eq!(event.ncells, 1);
    assert!(!event.wrap);
}

/// The same event with no cells at all: the decoder used to read the first
/// cell before checking whether there was one (#25184).
#[test]
fn a_grid_line_with_no_cells_decodes() {
    let payload = redraw_grid_line(&[0x90]);
    let mut stream = Stream::new();
    stream.feed(&payload, payload.len());
    assert!(stream.advance());
    assert_eq!(stream.event().expect("a completed grid_line").ncells, 0);
}

/// The UI client's shared decode buffer, made wide enough for a case.
///
/// A cell that is not a trailing run of spaces is written straight into the
/// buffer, which only a `grid_resize` from the server widens. A case that
/// wants to decode one therefore has to widen it itself — under the editor
/// lock, because it is process-wide. Nothing is restored on the way out: the
/// buffer only ever grows, and each event fills what it reports.
fn wide_decode_buffer(width: usize) -> crate::support::Editor {
    let editor = crate::support::editor_lock();
    Unpacker::widen_grid_line_buf(width);
    editor
}

/// Every prefix of the message is a legal pause point, and none of them may
/// report a finished event. The Lua spec only ever paused at one offset,
/// one byte from the end.
#[test]
fn no_prefix_of_a_grid_line_reports_an_event() {
    let cells = [
        0x92, 0x93, 0xa1, b'a', 0x00, 0x02, 0x93, 0xa1, b'b', 0x01, 0x03,
    ];
    let payload = redraw_grid_line(&cells);
    let _bufs = wide_decode_buffer(16);

    for prefix in 0..payload.len() {
        let mut stream = Stream::new();
        stream.feed(&payload, prefix);
        assert!(
            !stream.advance(),
            "a {prefix}-byte prefix must not finish the event"
        );
        assert!(
            stream.state() >= 0,
            "a {prefix}-byte prefix must not invalidate the stream"
        );
    }

    // And the whole thing does finish, with both cells accounted for.
    let mut stream = Stream::new();
    stream.feed(&payload, payload.len());
    assert!(stream.advance());
    let event = stream.event().expect("a completed grid_line");
    assert_eq!(event.ncells, 2);
    assert_eq!(event.coloff, 5, "two cells, repeats 2 and 3");
}

/// Feeding the message one byte at a time — the worst case a socket can
/// produce — reaches the same answer as feeding it whole.
#[test]
fn a_grid_line_delivered_one_byte_at_a_time_still_arrives() {
    let payload = redraw_grid_line(&[0x91, 0x93, 0xa1, b'x', 0x00, 0x05]);
    let _bufs = wide_decode_buffer(16);
    let mut stream = Stream::new();
    stream.feed(&payload, 0);
    let mut finished = false;
    for _ in 0..payload.len() {
        stream.one_more_byte();
        finished = stream.advance();
    }
    assert!(finished, "the last byte completes the event");
    let event = stream.event().expect("a completed grid_line");
    assert_eq!(event.ncells, 1);
    assert_eq!(event.coloff, 5);
}
