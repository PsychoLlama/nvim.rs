#![deny(unsafe_op_in_unsafe_fn)]

//! The `redraw` fast path.
//!
//! A UI server's `redraw` notification is the one message body the tree
//! parser never sees: a `grid_line` event would allocate an `Object` per
//! screen cell, so it is decoded a token at a time straight into the shared
//! line buffers. Everything here reads through a [`Cursor`], whose
//! *construction* is the unsafe step, which is what leaves the state machine
//! itself ordinary Rust.

use core::ffi::c_char;

use crate::grid::schar_from_buf;

use crate::global_cell::GlobalCell;
use crate::memory::ARENA_EMPTY;
use crate::mpack::conv::mpack_unpack_boolean;
use crate::mpack::mpack_core::mpack_rtoken;
use crate::types::{
    Array, GridLineEvent, RawLine, Unpacker, mpack_token_t, mpack_token_type_t, sattr_T, schar_T,
    size_t,
};
use crate::ui_client::{ui_client_event_grid_line, ui_client_get_redraw_handler};
use ::libc::abort;

use super::protocol;

/// The cells of the `grid_line` event being decoded.
///
/// The decoder writes them straight in rather than building an array per
/// event, and [`ui_client_event_raw_line`] hands the run to the TUI.
///
/// [`ui_client_event_raw_line`]: crate::ui_client::ui_client_event_raw_line
static GRID_LINE_BUF: GlobalCell<RawLine> = GlobalCell::new(RawLine::empty());

/// The decode buffer, as a handle.
///
/// Named rather than borrowed, for the reason every handle in this tree is:
/// the cells outlive the decode -- `ui_client_event_raw_line` reads them
/// after the unpacker has returned -- so no `&mut` spans their life. Every
/// borrow lasts one accessor call.
#[derive(Clone, Copy)]
struct RawLineRef(*mut RawLine);

impl ::core::ops::Deref for RawLineRef {
    type Target = RawLine;

    fn deref(&self) -> &RawLine {
        // SAFETY: the only constructor names a `static`.
        unsafe { &*self.0 }
    }
}

impl ::core::ops::DerefMut for RawLineRef {
    fn deref_mut(&mut self) -> &mut RawLine {
        // SAFETY: the only constructor names a `static`.
        unsafe { &mut *self.0 }
    }
}

/// The one `grid_line` decode buffer.
fn grid_line_buf() -> RawLineRef {
    RawLineRef(GRID_LINE_BUF.ptr())
}

impl Unpacker {
    /// Widen the shared `grid_line` decode buffer to `width` cells.
    ///
    /// The server may not send more cells than the grid it announced, so the
    /// buffer only has to hold the widest one: `ui_client_event_grid_resize`
    /// calls this, and a column past the width is a protocol error.
    pub fn widen_grid_line_buf(width: size_t) {
        grid_line_buf().widen(width);
    }

    /// The decoded cells, for the `tui_raw_line` call that takes them by
    /// pointer. Nothing between here and that call can widen the buffer.
    pub(crate) fn grid_line_cells() -> (*const schar_T, *const sattr_T) {
        grid_line_buf().as_ptrs()
    }
}
use super::{MPACK_EOF, TOKEN_ARRAY, TOKEN_BOOLEAN, TOKEN_SINT, TOKEN_STR, TOKEN_UINT};

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
///
/// Constructing one is the unsafe step — it is the promise that `data` is
/// `size` readable bytes — so every read through it afterwards is ordinary
/// Rust.
struct Cursor {
    data: *const c_char,
    size: size_t,
}

impl Cursor {
    /// # Safety
    /// `data` points at `size` readable bytes that stay live and unwritten
    /// for the cursor's life.
    unsafe fn new(data: *const c_char, size: size_t) -> Self {
        Cursor { data, size }
    }

    /// Reads one token and checks it is the kind this position calls for.
    fn next(&mut self, expected: mpack_token_type_t) -> Result<mpack_token_t, Halt> {
        // SAFETY: the promise made at construction. `mpack_rtoken` reads
        // within `size` and advances both fields by what it consumed.
        let (result, tok) = unsafe {
            let mut tok: mpack_token_t = core::mem::zeroed();
            let result = mpack_rtoken(&raw mut self.data, &raw mut self.size, &raw mut tok);
            (result, tok)
        };
        if result == MPACK_EOF {
            return Err(Halt::Incomplete);
        }
        if result != 0 || !protocol::token_matches(expected, tok.type_0) {
            return Err(Halt::Invalid);
        }
        Ok(tok)
    }

    /// Consumes `len` bytes of payload, handing back where they started.
    fn take(&mut self, len: size_t) -> *const c_char {
        let taken = self.data;
        self.data = self.data.wrapping_add(len);
        self.size -= len;
        taken
    }
}

/// # Safety
/// [`unpacker_advance`]'s contract.
pub(super) unsafe fn unpacker_parse_redraw(p: *mut Unpacker) -> bool {
    // SAFETY: the caller's unpacker; nothing in the redraw decoder re-enters
    // it, and `read_ptr`/`read_size` are the bytes that have arrived.
    let (u, mut cursor) = unsafe {
        let u = &mut *p;
        let cursor = Cursor::new(u.read_ptr, u.read_size);
        (u, cursor)
    };
    match parse_redraw(u, &mut cursor) {
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
fn parse_redraw(u: &mut Unpacker, cursor: &mut Cursor) -> Result<bool, Halt> {
    let Unpacker {
        grid_line_event: g,
        state,
        nevents,
        ncalls,
        ui_handler,
        read_ptr,
        read_size,
        arena,
        ..
    } = u;
    let mut stage = *state;

    if stage == protocol::REDRAW_ARGS {
        return Ok(true);
    }
    // `REDRAW_ARGS_DONE` belongs to the tree parser, not to this one, and its
    // caller filters it out; anything else means the machine lost track.
    let known = protocol::REDRAW_EVENTS..=protocol::GRID_LINE_WRAP;
    if stage == protocol::REDRAW_ARGS_DONE || !known.contains(&stage) {
        // SAFETY: `abort` takes no arguments and does not return.
        unsafe { abort() };
    }

    if stage == protocol::REDRAW_EVENTS {
        *nevents = cursor.next(TOKEN_ARRAY)?.length.cast_signed();
        stage = protocol::REDRAW_CALL;
    }

    if stage == protocol::REDRAW_CALL {
        *ncalls = cursor.next(TOKEN_ARRAY)?.length.cast_signed();
        let had_calls = *ncalls;
        *ncalls -= 1;
        if had_calls == 0 {
            return Err(Halt::Invalid);
        }

        let tok = cursor.next(TOKEN_STR)?;
        if tok.length as size_t > cursor.size {
            return Err(Halt::Incomplete);
        }
        // SAFETY: the name is `tok.length` readable bytes of the cursor's
        // buffer, which the check above bounds.
        *ui_handler = unsafe {
            ui_client_get_redraw_handler(cursor.data, tok.length as size_t, core::ptr::null_mut())
        };
        cursor.take(tok.length as size_t);

        *nevents -= 1;
        *read_ptr = cursor.data;
        *read_size = cursor.size;

        let is_grid_line = ui_handler.fn_0.is_some_and(|f| {
            core::ptr::fn_addr_eq(f, ui_client_event_grid_line as unsafe fn(Array))
        });
        if !is_grid_line {
            *state = protocol::REDRAW_ARGS;
            return Ok(true);
        }
        *state = protocol::GRID_LINE_EVENT;
        *arena = ARENA_EMPTY;
        stage = protocol::GRID_LINE_EVENT;
    }

    if stage == protocol::GRID_LINE_EVENT {
        // [grid, row, startcol, [cells], wrap]
        if cursor.next(TOKEN_ARRAY)?.length != 5 {
            return Err(Halt::Invalid);
        }
        for slot in 0..3 {
            // SAFETY: a uint token carries its value in the `value` arm.
            let lo = unsafe { cursor.next(TOKEN_UINT)?.data.value.lo };
            g.args[slot] = lo.cast_signed();
        }
        g.ncells = cursor.next(TOKEN_ARRAY)?.length.cast_signed();
        g.icell = 0;
        g.coloff = 0;
        g.cur_attr = -1;
        *read_ptr = cursor.data;
        *read_size = cursor.size;
        *state = protocol::GRID_LINE_CELLS;
        stage = protocol::GRID_LINE_CELLS;
    }

    if stage == protocol::GRID_LINE_CELLS {
        while g.icell != g.ncells {
            parse_grid_line_cell(g, cursor)?;
            *read_ptr = cursor.data;
            *read_size = cursor.size;
            g.icell += 1;
        }
        *state = protocol::GRID_LINE_WRAP;
    }

    g.wrap = mpack_unpack_boolean(cursor.next(TOKEN_BOOLEAN)?);
    *read_ptr = cursor.data;
    *read_size = cursor.size;
    Ok(true)
}

/// Decodes `[text, attr?, repeat?]` into the shared line buffers.
///
/// `attr` persists across cells that omit it, which is what makes the wire
/// form compact. A run of spaces at the end of the line is not written at
/// all: it becomes the event's `clear_width`.
fn parse_grid_line_cell(g: &mut GridLineEvent, cursor: &mut Cursor) -> Result<(), Halt> {
    let arity = cursor.next(TOKEN_ARRAY)?.length.cast_signed();
    if !(1..=3).contains(&arity) {
        return Err(Halt::Invalid);
    }

    let tok = cursor.next(TOKEN_STR)?;
    if tok.length as size_t > cursor.size {
        return Err(Halt::Incomplete);
    }
    let cell_len = tok.length as size_t;
    let cell = cursor.take(cell_len);

    if arity >= 2 {
        // SAFETY: a sint token carries its value in the `value` arm.
        let lo = unsafe { cursor.next(TOKEN_SINT)?.data.value.lo };
        g.cur_attr = lo.cast_signed();
    }
    let repeat = if arity >= 3 {
        // SAFETY: as above, for a uint token.
        let lo = unsafe { cursor.next(TOKEN_UINT)?.data.value.lo };
        lo.cast_signed()
    } else {
        1
    };

    g.clear_width = 0;
    // SAFETY: `cell` is `cell_len` bytes the cursor just handed over, and
    // `schar_from_buf` reads within that length.
    let (cell_bytes, sc): (&[u8], schar_T) = unsafe {
        (
            core::slice::from_raw_parts(cell.cast::<u8>(), cell_len),
            schar_from_buf(cell, cell_len),
        )
    };
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
        let mut cells = grid_line_buf();
        if coloff >= cells.width() {
            return Err(Halt::Invalid);
        }
        cells.put(coloff, sc, g.cur_attr as _);
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
    use crate::types::{
        Integer, Object, kObjectTypeBoolean, kObjectTypeFloat, kObjectTypeInteger, kObjectTypeNil,
    };

    /// A cursor over `bytes`. Every caller keeps the slice alive for the whole
    /// test, which is the promise `Cursor::new` asks for.
    fn cursor(bytes: &[u8]) -> Cursor {
        // SAFETY: the caller's slice outlives the cursor.
        unsafe { Cursor::new(bytes.as_ptr().cast::<c_char>(), bytes.len()) }
    }

    /// The integer a scalar token decoded to, or `None` if it did not decode
    /// to an integer at all.
    fn integer_of(object: Object) -> Option<Integer> {
        if object.type_0 != kObjectTypeInteger {
            return None;
        }
        // SAFETY: guarded by the type tag.
        Some(unsafe { object.data.integer })
    }

    #[test]
    fn a_cursor_walks_a_sequence_of_tokens() {
        // [1, 2] then a bare 3, as three separate reads.
        let bytes = [0x92u8, 0x01, 0x02, 0x03];
        let mut c = cursor(&bytes);
        assert_eq!(c.next(TOKEN_ARRAY).ok().map(|t| t.length), Some(2));
        assert_eq!(c.size, 3);
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
        assert_eq!(c.size, 0);
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
        let len = c.next(TOKEN_STR).unwrap().length as usize;
        assert_eq!(len, 3);
        let start = c.take(len);
        assert_eq!(c.size, 0);
        // SAFETY: `start` is the three payload bytes the cursor just left.
        let payload = unsafe { core::slice::from_raw_parts(start.cast::<u8>(), len) };
        assert_eq!(payload, b"abc");
    }

    #[test]
    fn scalars_decode_to_the_object_they_stand_for() {
        let mut c = cursor(&[0xc0u8]);
        assert_eq!(
            scalar_object(c.next(TOKEN_NIL).unwrap()).unwrap().type_0,
            kObjectTypeNil
        );

        let mut c = cursor(&[0xc3u8]);
        let boolean = scalar_object(c.next(TOKEN_BOOLEAN).unwrap()).unwrap();
        assert_eq!(boolean.type_0, kObjectTypeBoolean);
        // SAFETY: guarded by the type tag.
        assert!(unsafe { boolean.data.boolean });

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
        assert_eq!(float.type_0, kObjectTypeFloat);
        // SAFETY: guarded by the type tag.
        assert!((unsafe { float.data.floating } - 1.0).abs() < f64::EPSILON);

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
