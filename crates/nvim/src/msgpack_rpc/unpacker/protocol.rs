#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! The decisions the streaming unpacker makes, separated from the buffers it
//! makes them about: which parse stage comes next, which token types are
//! interchangeable, and how the two hand-rolled vectors grow.

use core::ffi::c_int;

use crate::types::{MessageType, mpack_token_type_t};

/// Parse stages. The unpacker resumes mid-message whenever the stream runs
/// out, so its position is one of these plus the byte offset it stopped at.
///
/// Nested in a module because the unit-test header generator collects every
/// top-level constant into one flat namespace; the `pub use` below is
/// invisible to it.
mod stage {
    use core::ffi::c_int;

    /// The stream is not msgpack-rpc and the channel is finished.
    pub const INVALID: c_int = -1;

    /// Between messages: expecting `[type, ...]`.
    pub const HEADER: c_int = 0;
    /// A response's error slot, which is followed by its result.
    pub const RESPONSE_ERROR: c_int = 1;
    /// A request's arguments, a notification's arguments, or a response's
    /// result — whatever the header left to read.
    pub const BODY: c_int = 2;

    /// The array of events in a `redraw` notification.
    pub const REDRAW_EVENTS: c_int = 10;
    /// One event: its name, then one call's worth of arguments.
    pub const REDRAW_CALL: c_int = 11;
    /// A non-`grid_line` event's arguments, still to be read.
    pub const REDRAW_ARGS: c_int = 12;
    /// A non-`grid_line` event's arguments, now unpacked.
    pub const REDRAW_ARGS_DONE: c_int = 13;

    /// A `grid_line` event's fixed leading arguments.
    pub const GRID_LINE_EVENT: c_int = 14;
    /// Its cell array, decoded straight into the shared line buffers.
    pub const GRID_LINE_CELLS: c_int = 15;
    /// Its trailing `wrap` flag; the event is complete once it is read.
    pub const GRID_LINE_WRAP: c_int = 16;
}

pub use stage::*;

mod token {
    use crate::types::mpack_token_type_t;

    pub const NIL: mpack_token_type_t = 1;
    pub const BOOLEAN: mpack_token_type_t = 2;
    pub const UINT: mpack_token_type_t = 3;
    pub const SINT: mpack_token_type_t = 4;
    pub const FLOAT: mpack_token_type_t = 5;
    pub const CHUNK: mpack_token_type_t = 6;
    pub const ARRAY: mpack_token_type_t = 7;
    pub const MAP: mpack_token_type_t = 8;
    pub const BIN: mpack_token_type_t = 9;
    pub const STR: mpack_token_type_t = 10;
    pub const EXT: mpack_token_type_t = 11;
}

pub use token::*;

/// Whether a token of type `actual` satisfies a demand for `expected`.
///
/// Two pairs are interchangeable on the wire: a binary blob wherever a string
/// is wanted (clients differ on which one they send for method names and
/// cell contents), and an unsigned integer wherever a signed one is wanted
/// (a non-negative highlight id encodes as unsigned).
pub fn token_matches(expected: mpack_token_type_t, actual: mpack_token_type_t) -> bool {
    actual == expected || (expected == STR && actual == BIN) || (expected == SINT && actual == UINT)
}

/// Whether a message header of `array_length` elements may carry this type.
///
/// Three elements means a notification and nothing else; four means a request
/// or a response. Anything else has already been rejected by the caller.
pub fn header_shape_is_valid(array_length: usize, message_type: u32) -> bool {
    if array_length == 3 {
        message_type == kMessageTypeNotification as u32
    } else {
        message_type < kMessageTypeNotification as u32
    }
}

const kMessageTypeNotification: MessageType = 2;

mod limit {
    /// The longest method name the header parser will look at. A longer one
    /// is treated as a decoding failure rather than an unknown method.
    pub const METHOD_NAME_MAX: u32 = 100;
}

pub use limit::METHOD_NAME_MAX;

/// Where the unpacker goes after finishing one redraw call.
///
/// `calls` and `events` are the counts *after* the finished call has been
/// deducted, so both reaching zero means the whole notification is done.
pub fn stage_after_redraw_call(finished: c_int, calls: c_int, events: c_int) -> c_int {
    if calls > 0 {
        // Another call to the same event handler. `grid_line` re-enters its
        // own decoder; everything else goes back through the generic one.
        if finished == GRID_LINE_WRAP {
            GRID_LINE_EVENT
        } else {
            REDRAW_ARGS
        }
    } else if events > 0 {
        REDRAW_CALL
    } else {
        HEADER
    }
}

/// The capacity a hand-rolled vector takes when it has to hold `needed`
/// elements: the next power of two, matching the `roundup32` the transpiled
/// `kv_ensure_space` open-coded.
pub fn capacity_for(needed: usize) -> usize {
    if needed == 0 {
        return 0;
    }
    let mut capacity = needed - 1;
    capacity |= capacity >> 1;
    capacity |= capacity >> 2;
    capacity |= capacity >> 4;
    capacity |= capacity >> 8;
    capacity |= capacity >> 16;
    capacity + 1
}

/// The capacity a hand-rolled vector takes when one more element does not
/// fit: double it, or start at eight.
pub fn grown_capacity(capacity: usize) -> usize {
    if capacity != 0 { capacity << 1 } else { 8 }
}

/// Whether a cell run is the trailing "clear the rest of the line" marker
/// rather than something to write into the line buffers.
pub fn is_clear_run(is_last_cell: bool, cell: &[u8], repeat: c_int) -> bool {
    is_last_cell && cell == b" " && repeat > 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interchangeable_token_types() {
        assert!(token_matches(ARRAY, ARRAY));
        assert!(token_matches(STR, BIN));
        assert!(!token_matches(BIN, STR));
        assert!(token_matches(SINT, UINT));
        assert!(!token_matches(UINT, SINT));
        assert!(!token_matches(ARRAY, MAP));
    }

    #[test]
    fn header_shapes() {
        // [2, method, args]
        assert!(header_shape_is_valid(3, 2));
        assert!(!header_shape_is_valid(3, 0));
        assert!(!header_shape_is_valid(3, 1));
        // [0, id, method, args] and [1, id, error, result]
        assert!(header_shape_is_valid(4, 0));
        assert!(header_shape_is_valid(4, 1));
        assert!(!header_shape_is_valid(4, 2));
        assert!(!header_shape_is_valid(4, 3));
    }

    #[test]
    fn redraw_call_sequencing() {
        assert_eq!(stage_after_redraw_call(REDRAW_ARGS_DONE, 2, 1), REDRAW_ARGS);
        assert_eq!(
            stage_after_redraw_call(GRID_LINE_WRAP, 2, 1),
            GRID_LINE_EVENT
        );
        assert_eq!(stage_after_redraw_call(GRID_LINE_WRAP, 0, 1), REDRAW_CALL);
        assert_eq!(stage_after_redraw_call(REDRAW_ARGS_DONE, 0, 0), HEADER);
    }

    #[test]
    fn capacities_round_up_to_powers_of_two() {
        assert_eq!(capacity_for(0), 0);
        assert_eq!(capacity_for(1), 1);
        assert_eq!(capacity_for(2), 2);
        assert_eq!(capacity_for(3), 4);
        assert_eq!(capacity_for(17), 32);
        assert_eq!(capacity_for(1024), 1024);

        assert_eq!(grown_capacity(0), 8);
        assert_eq!(grown_capacity(8), 16);
    }

    #[test]
    fn clear_runs() {
        assert!(is_clear_run(true, b" ", 40));
        // A single space is a space, not a clear.
        assert!(!is_clear_run(true, b" ", 1));
        // Only the last cell of the line may clear.
        assert!(!is_clear_run(false, b" ", 40));
        assert!(!is_clear_run(true, b"x", 40));
        assert!(!is_clear_run(true, "\u{a0}".as_bytes(), 40));
    }
}
