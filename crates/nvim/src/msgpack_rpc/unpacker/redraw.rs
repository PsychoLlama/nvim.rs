#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! The `redraw` fast path.
//!
//! A UI server's `redraw` notification is the one message body the tree
//! parser never sees: a `grid_line` event would allocate an `Object` per
//! screen cell, so it is decoded a token at a time straight into the shared
//! line buffers. Everything here reads through a [`Cursor`] over the bytes
//! that have arrived, committed back to the caller's input at every stage
//! boundary.

use crate::grid::schar_from_buf;

use crate::global_cell::GlobalCell;
use crate::mpack::mpack_core::{MPACK_EOF, MPACK_OK, read_token};
use crate::mpack::token::{self, Tok};
use crate::types::{
    Array, GridLineEvent, RawLine, ScreenAttr, ScreenChar, Unpacker, mpack_token_type_t, size_t,
};
use crate::ui_client::{ui_client_event_grid_line, ui_client_get_redraw_handler};

use super::protocol;

/// The cells of the `grid_line` event being decoded.
///
/// The decoder writes them straight in rather than building an array per
/// event, and [`ui_client_event_raw_line`] hands the run to the TUI.
///
/// [`ui_client_event_raw_line`]: crate::ui_client::ui_client_event_raw_line
static GRID_LINE_BUF: GlobalCell<RawLine> = GlobalCell::new(RawLine::empty());

impl Unpacker {
    /// Widen the shared `grid_line` decode buffer to `width` cells.
    ///
    /// The server may not send more cells than the grid it announced, so the
    /// buffer only has to hold the widest one: `ui_client_event_grid_resize`
    /// calls this, and a column past the width is a protocol error.
    pub fn widen_grid_line_buf(width: size_t) {
        GRID_LINE_BUF.with_mut(|cells| cells.widen(width));
    }

    /// The decoded cells, for the `tui_raw_line` call that takes them by
    /// pointer. Nothing between here and that call can widen the buffer.
    pub(crate) fn grid_line_cells() -> (*const ScreenChar, *const ScreenAttr, size_t) {
        GRID_LINE_BUF.with(|cells| {
            let (chars, attrs) = cells.as_ptrs();
            (chars, attrs, cells.width())
        })
    }
}
use super::{TOKEN_ARRAY, TOKEN_BOOLEAN, TOKEN_SINT, TOKEN_STR, TOKEN_UINT, token_type};

/// Why a redraw parse stopped short.
#[derive(Debug)]
enum Halt {
    /// Not all of this stage has arrived. The read cursor stays where it was
    /// last committed, so the stage restarts when more bytes turn up.
    Incomplete,
    /// The stream is not a well-formed redraw batch.
    Invalid,
}

/// The bytes still to be read, as libmpack's token reader wants them.
struct Cursor<'a> {
    data: &'a [u8],
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Cursor { data }
    }

    /// Reads one token and checks it is the kind this position calls for.
    fn next(&mut self, expected: mpack_token_type_t) -> Result<Tok, Halt> {
        let (result, tok) = read_token(&mut self.data);
        if result == MPACK_EOF {
            return Err(Halt::Incomplete);
        }
        if result != MPACK_OK || !protocol::token_matches(expected, token_type(&tok)) {
            return Err(Halt::Invalid);
        }
        Ok(tok)
    }

    /// Consumes `len` bytes of payload and hands them back, or answers that
    /// they have not all arrived.
    fn take(&mut self, len: size_t) -> Result<&'a [u8], Halt> {
        if len > self.data.len() {
            return Err(Halt::Incomplete);
        }
        let (taken, rest) = self.data.split_at(len);
        self.data = rest;
        Ok(taken)
    }
}

/// Decodes as much of a redraw batch as `input` holds, committing it past
/// every stage that finished.
pub(super) fn unpacker_parse_redraw(u: &mut Unpacker, input: &mut &[u8]) -> bool {
    let mut cursor = Cursor::new(input);
    match parse_redraw(u, &mut cursor, input) {
        Ok(done) => done,
        Err(Halt::Incomplete) => false,
        Err(Halt::Invalid) => {
            u.state = protocol::INVALID;
            false
        }
    }
}

/// Decodes `[[name, [args], ...], ...]`, one event at a time.
///
/// Each stage falls through into the next, and the read cursor is committed
/// wherever a stage boundary is crossed — so an event that arrives in pieces
/// resumes at the last boundary rather than at the start of the batch.
fn parse_redraw<'a>(
    u: &mut Unpacker,
    cursor: &mut Cursor<'a>,
    input: &mut &'a [u8],
) -> Result<bool, Halt> {
    let Unpacker {
        grid_line_event: g,
        state,
        nevents,
        ncalls,
        ui_handler,
        ..
    } = u;
    let mut stage = *state;

    if stage == protocol::REDRAW_ARGS {
        return Ok(true);
    }
    // `REDRAW_ARGS_DONE` belongs to the tree parser, not to this one, and its
    // caller filters it out; anything else means the machine lost track.
    let known = protocol::REDRAW_EVENTS..=protocol::GRID_LINE_WRAP;
    assert!(
        stage != protocol::REDRAW_ARGS_DONE && known.contains(&stage),
        "the redraw decoder lost track of its stage"
    );

    if stage == protocol::REDRAW_EVENTS {
        *nevents = cursor.next(TOKEN_ARRAY)?.len.cast_signed();
        stage = protocol::REDRAW_CALL;
    }

    if stage == protocol::REDRAW_CALL {
        *ncalls = cursor.next(TOKEN_ARRAY)?.len.cast_signed();
        let had_calls = *ncalls;
        *ncalls -= 1;
        if had_calls == 0 {
            return Err(Halt::Invalid);
        }

        let tok = cursor.next(TOKEN_STR)?;
        let name = cursor.take(tok.len as size_t)?;
        *ui_handler = ui_client_get_redraw_handler(name);

        *nevents -= 1;
        *input = cursor.data;

        let is_grid_line = ui_handler
            .fn_0
            .is_some_and(|f| core::ptr::fn_addr_eq(f, ui_client_event_grid_line as fn(Array)));
        if !is_grid_line {
            *state = protocol::REDRAW_ARGS;
            return Ok(true);
        }
        *state = protocol::GRID_LINE_EVENT;
        stage = protocol::GRID_LINE_EVENT;
    }

    if stage == protocol::GRID_LINE_EVENT {
        // [grid, row, startcol, [cells], wrap]
        if cursor.next(TOKEN_ARRAY)?.len != 5 {
            return Err(Halt::Invalid);
        }
        for slot in 0..3 {
            g.args[slot] = cursor.next(TOKEN_UINT)?.lo.cast_signed();
        }
        g.ncells = cursor.next(TOKEN_ARRAY)?.len.cast_signed();
        g.icell = 0;
        g.coloff = 0;
        g.cur_attr = -1;
        *input = cursor.data;
        *state = protocol::GRID_LINE_CELLS;
        stage = protocol::GRID_LINE_CELLS;
    }

    if stage == protocol::GRID_LINE_CELLS {
        while g.icell != g.ncells {
            parse_grid_line_cell(g, cursor)?;
            *input = cursor.data;
            g.icell += 1;
        }
        *state = protocol::GRID_LINE_WRAP;
    }

    g.wrap = token::unpack_boolean(&cursor.next(TOKEN_BOOLEAN)?);
    *input = cursor.data;
    Ok(true)
}

/// Decodes `[text, attr?, repeat?]` into the shared line buffers.
///
/// `attr` persists across cells that omit it, which is what makes the wire
/// form compact. A run of spaces at the end of the line is not written at
/// all: it becomes the event's `clear_width`.
fn parse_grid_line_cell(g: &mut GridLineEvent, cursor: &mut Cursor) -> Result<(), Halt> {
    let arity = cursor.next(TOKEN_ARRAY)?.len.cast_signed();
    if !(1..=3).contains(&arity) {
        return Err(Halt::Invalid);
    }

    let tok = cursor.next(TOKEN_STR)?;
    let cell_bytes = cursor.take(tok.len as size_t)?;

    if arity >= 2 {
        g.cur_attr = cursor.next(TOKEN_SINT)?.lo.cast_signed();
    }
    let repeat = if arity >= 3 {
        cursor.next(TOKEN_UINT)?.lo.cast_signed()
    } else {
        1
    };

    g.clear_width = 0;
    let sc = schar_from_buf(cell_bytes);
    if protocol::is_clear_run(g.icell == g.ncells - 1, cell_bytes, repeat) {
        g.clear_width = repeat;
        return Ok(());
    }

    for _ in 0..repeat {
        // A negative `coloff` cannot happen — it starts at zero and only
        // grows — and `usize::try_from` turning one into "out of bounds" is
        // the safe reading of upstream's bare `(size_t)` cast either way.
        let Ok(coloff) = usize::try_from(g.coloff) else {
            return Err(Halt::Invalid);
        };
        let fits = GRID_LINE_BUF.with_mut(|cells| {
            let fits = coloff < cells.width();
            if fits {
                cells.put(coloff, sc, g.cur_attr as _);
            }
            fits
        });
        if !fits {
            return Err(Halt::Invalid);
        }
        g.coloff += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msgpack_rpc::unpacker::{
        TOKEN_BIN, TOKEN_FLOAT, TOKEN_NIL, scalar_object, unpack_integer_token,
    };
    use crate::types::{Integer, Object};

    fn cursor(bytes: &[u8]) -> Cursor<'_> {
        Cursor::new(bytes)
    }

    /// The integer a scalar token decoded to, or `None` if it did not decode
    /// to an integer at all.
    fn integer_of(object: Object) -> Option<Integer> {
        object.as_integer()
    }

    #[test]
    fn a_cursor_walks_a_sequence_of_tokens() {
        // [1, 2] then a bare 3, as three separate reads.
        let bytes = [0x92u8, 0x01, 0x02, 0x03];
        let mut c = cursor(&bytes);
        assert_eq!(c.next(TOKEN_ARRAY).ok().map(|t| t.len), Some(2));
        assert_eq!(c.data.len(), 3);
        assert_eq!(
            integer_of(scalar_object(c.next(TOKEN_UINT).unwrap()).unwrap()),
            Some(1)
        );
        assert_eq!(
            integer_of(scalar_object(c.next(TOKEN_UINT).unwrap()).unwrap()),
            Some(2)
        );
        assert_eq!(
            integer_of(scalar_object(c.next(TOKEN_UINT).unwrap()).unwrap()),
            Some(3)
        );
        assert_eq!(c.data.len(), 0);
        assert!(matches!(c.next(TOKEN_UINT), Err(Halt::Incomplete)));
    }

    #[test]
    fn a_truncated_token_is_incomplete_and_a_bad_one_is_invalid() {
        // A uint64 tag with only one of its eight payload bytes.
        let truncated = [0xcfu8, 0x00];
        assert!(matches!(
            cursor(&truncated).next(TOKEN_UINT),
            Err(Halt::Incomplete)
        ));
        // An empty buffer is the same answer: wait for more.
        assert!(matches!(
            cursor(&[]).next(TOKEN_ARRAY),
            Err(Halt::Incomplete)
        ));
        // 0xc1 is msgpack's one reserved byte.
        assert!(matches!(
            cursor(&[0xc1u8]).next(TOKEN_UINT),
            Err(Halt::Invalid)
        ));
        // A well-formed token of the wrong kind is invalid, not incomplete —
        // the stream is not a redraw batch and no amount of waiting helps.
        assert!(matches!(
            cursor(&[0x80u8]).next(TOKEN_ARRAY),
            Err(Halt::Invalid)
        ));
    }

    #[test]
    fn the_two_interchangeable_kinds_are_accepted_through_the_cursor() {
        // A blob header where a string is wanted: clients differ on which
        // they send for a method name or a cell's text.
        assert!(cursor(&[0xc4u8, 0x01, b'x']).next(TOKEN_STR).is_ok());
        assert!(matches!(
            cursor(&[0xa1u8, b'x']).next(TOKEN_BIN),
            Err(Halt::Invalid)
        ));
        // An unsigned integer where a signed one is wanted: a non-negative
        // highlight id encodes as unsigned.
        assert!(cursor(&[0x07u8]).next(TOKEN_SINT).is_ok());
        assert!(matches!(
            cursor(&[0xffu8]).next(TOKEN_UINT),
            Err(Halt::Invalid)
        ));
    }

    #[test]
    fn take_hands_back_the_payload_and_advances() {
        let bytes = [0xa3u8, b'a', b'b', b'c'];
        let mut c = cursor(&bytes);
        let len = c.next(TOKEN_STR).unwrap().len as usize;
        assert_eq!(len, 3);
        assert!(matches!(
            Cursor::new(c.data).take(len + 1),
            Err(Halt::Incomplete)
        ));
        let payload = c.take(len).unwrap();
        assert_eq!(c.data.len(), 0);
        assert_eq!(payload, b"abc");
    }

    #[test]
    fn scalars_decode_to_the_object_they_stand_for() {
        let mut c = cursor(&[0xc0u8]);
        assert!(scalar_object(c.next(TOKEN_NIL).unwrap()).unwrap().is_nil());

        let mut c = cursor(&[0xc3u8]);
        let boolean = scalar_object(c.next(TOKEN_BOOLEAN).unwrap()).unwrap();
        assert_eq!(boolean.as_boolean(), Some(true));

        // Both signs reach the same `Integer`, which is what `msgpackparse()`
        // and the RPC decoder agree on.
        let mut c = cursor(&[0xffu8]);
        assert_eq!(
            integer_of(scalar_object(c.next(TOKEN_SINT).unwrap()).unwrap()),
            Some(-1)
        );
        let mut c = cursor(&[0xcfu8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
        assert_eq!(
            integer_of(scalar_object(c.next(TOKEN_UINT).unwrap()).unwrap()),
            Some(-1)
        );

        let mut c = cursor(&[0xcbu8, 0x3f, 0xf0, 0, 0, 0, 0, 0, 0]);
        let float = scalar_object(c.next(TOKEN_FLOAT).unwrap()).unwrap();
        assert!((float.as_float().unwrap() - 1.0).abs() < f64::EPSILON);

        // A container header is not a scalar: the tree parser allocates for
        // it instead.
        let mut c = cursor(&[0x92u8, 0x01, 0x02]);
        assert!(scalar_object(c.next(TOKEN_ARRAY).unwrap()).is_none());
    }

    #[test]
    fn an_integer_token_reads_the_same_either_way_it_was_encoded() {
        let mut c = cursor(&[0x7fu8]);
        assert_eq!(unpack_integer_token(c.next(TOKEN_UINT).unwrap()), Some(127));
        let mut c = cursor(&[0xd0u8, 0x80]);
        assert_eq!(
            unpack_integer_token(c.next(TOKEN_SINT).unwrap()),
            Some(-128)
        );
        // A string token carries no integer at all.
        let mut c = cursor(&[0xa0u8]);
        assert_eq!(unpack_integer_token(c.next(TOKEN_STR).unwrap()), None);
    }
}
