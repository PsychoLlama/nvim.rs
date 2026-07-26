#![deny(unsafe_op_in_unsafe_fn)]

//! Scaffolding shared by the VimL parsers: the reader that feeds them lines,
//! the cursor over those lines, and the highlight log they append to.
//!
//! `ParserState` and everything it embeds is `repr(C)` and frozen —
//! `expressions.rs`, `api/vimscript.rs` and `ex_getln.rs` all read the fields
//! directly, and the three collections inside it are `kvec_withinit_t`s
//! (see `kvec::InitVec`).

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use crate::src::nvim::kvec::InitVec;
use crate::src::nvim::mbyte::string_convert;
use crate::src::nvim::memory::xfree;
pub use crate::src::nvim::types::{
    ParserHighlight, ParserHighlightChunk, ParserInputReader, ParserInputReader_lines, ParserLine,
    ParserLineGetter, ParserPosition, ParserState, ParserState_stack, ParserStateItem,
    ParserStateItem_data, ParserStateItem_data_expr, ParserStateItem_data_expr_type_0,
    ParserStateItem_type_0, vimconv_T,
};

/// `vimconv_T::vc_type` for "the input needs no conversion".
const CONV_NONE: c_int = 0;

pub const kExprUnknown: ParserStateItem_data_expr_type_0 = 0;
pub const kPTopStateParsingCommand: ParserStateItem_type_0 = 0;
pub const kPTopStateParsingExpression: ParserStateItem_type_0 = 1;

const EMPTY_LINE: ParserLine = ParserLine {
    data: ptr::null(),
    size: 0,
    allocated: false,
};

/// A parser that has read nothing and has no input. `viml_parser_init` fills
/// in the getter and points the two collections at their inline arrays; it
/// exists as a constant so callers (and tests) can name a `ParserState`
/// without reaching for `mem::zeroed`.
pub const PARSER_STATE_INIT: ParserState = ParserState {
    reader: ParserInputReader {
        get_line: None,
        cookie: ptr::null_mut(),
        lines: ParserInputReader_lines {
            size: 0,
            capacity: 0,
            items: ptr::null_mut(),
            init_array: [EMPTY_LINE; 4],
        },
        conv: vimconv_T {
            vc_type: CONV_NONE,
            vc_factor: 1,
            vc_fd: ptr::null_mut(),
            vc_fail: false,
        },
    },
    pos: ParserPosition { line: 0, col: 0 },
    stack: ParserState_stack {
        size: 0,
        capacity: 0,
        items: ptr::null_mut(),
        init_array: [ParserStateItem {
            type_0: kPTopStateParsingCommand,
            data: ParserStateItem_data {
                expr: ParserStateItem_data_expr {
                    type_0: kExprUnknown,
                },
            },
        }; 16],
    },
    colors: ptr::null_mut(),
    can_continuate: false,
};

/// The three `kvec_withinit_t` instantiations `ParserState` embeds. Each is a
/// distinct `repr(C)` struct with the same four fields, so they get one
/// constructor apiece rather than a trait.
fn lines_vec(lines: &mut ParserInputReader_lines) -> InitVec<'_, ParserLine> {
    InitVec::new(
        &mut lines.size,
        &mut lines.capacity,
        &mut lines.items,
        &mut lines.init_array,
    )
}

fn stack_vec(stack: &mut ParserState_stack) -> InitVec<'_, ParserStateItem> {
    InitVec::new(
        &mut stack.size,
        &mut stack.capacity,
        &mut stack.items,
        &mut stack.init_array,
    )
}

pub fn highlight_vec(colors: &mut ParserHighlight) -> InitVec<'_, ParserHighlightChunk> {
    InitVec::new(
        &mut colors.size,
        &mut colors.capacity,
        &mut colors.items,
        &mut colors.init_array,
    )
}

/// Start a parser over the lines `get_line` yields. `colors` may be null when
/// the caller does not want highlighting.
pub fn viml_parser_init(
    pstate: &mut ParserState,
    get_line: ParserLineGetter,
    cookie: *mut c_void,
    colors: *mut ParserHighlight,
) {
    *pstate = ParserState {
        reader: ParserInputReader {
            get_line,
            cookie,
            ..PARSER_STATE_INIT.reader
        },
        colors,
        ..PARSER_STATE_INIT
    };
    pstate.reader.lines.capacity = pstate.reader.lines.init_array.len();
    pstate.reader.lines.items = pstate.reader.lines.init_array.as_mut_ptr();
    pstate.stack.capacity = pstate.stack.init_array.len();
    pstate.stack.items = pstate.stack.init_array.as_mut_ptr();
}

/// Pull the next line from the getter, converting it to the parser's encoding
/// if the reader carries a conversion, and remember it: every line the parser
/// has seen stays in `reader.lines` for the duration, because tokens point
/// into them.
fn preader_get_line(preader: &mut ParserInputReader) -> ParserLine {
    let mut pline = EMPTY_LINE;
    let get_line = preader.get_line.expect("parser has no line getter");
    unsafe {
        get_line(preader.cookie, &raw mut pline);
        if preader.conv.vc_type != CONV_NONE && pline.size != 0 {
            let mut converted = ParserLine {
                data: ptr::null(),
                size: pline.size,
                allocated: true,
            };
            converted.data = string_convert(
                &raw mut preader.conv,
                pline.data as *mut c_char,
                &raw mut converted.size,
            );
            if pline.allocated {
                xfree(pline.data as *mut c_void);
            }
            pline = converted;
        }
    }
    lines_vec(&mut preader.lines).push(pline);
    pline
}

/// The rest of the current line, from the cursor on, reading one more line
/// from the input if the cursor just walked off the end of the last one.
/// `None` at end of input.
pub fn viml_parser_get_remaining_line(pstate: &mut ParserState) -> Option<ParserLine> {
    let mut pline = if pstate.pos.line == pstate.reader.lines.size {
        preader_get_line(&mut pstate.reader)
    } else {
        lines_vec(&mut pstate.reader.lines).last()
    };
    assert!(pstate.pos.line == pstate.reader.lines.size - 1);
    if pline.data.is_null() {
        return None;
    }
    // `wrapping_*` because the C did: the cursor is never past the line's end
    // (`viml_parser_advance` wraps to the next line first), so this is exact.
    pline.data = pline.data.wrapping_add(pstate.pos.col);
    pline.size = pline.size.wrapping_sub(pstate.pos.col);
    Some(pline)
}

/// Advance the cursor by `len` bytes, at most to the start of the next line.
pub fn viml_parser_advance(pstate: &mut ParserState, len: usize) {
    assert!(pstate.pos.line == pstate.reader.lines.size - 1);
    let pline = lines_vec(&mut pstate.reader.lines).last();
    if pstate.pos.col.wrapping_add(len) >= pline.size {
        pstate.pos.line += 1;
        pstate.pos.col = 0;
    } else {
        pstate.pos.col = pstate.pos.col.wrapping_add(len);
    }
}

/// Record the highlighting of `len` bytes at `start`. A no-op when the caller
/// asked for no highlighting. Chunks must arrive in order and must not
/// overlap.
pub fn viml_parser_highlight(
    pstate: &mut ParserState,
    start: ParserPosition,
    len: usize,
    group: *const c_char,
) {
    if pstate.colors.is_null() || len == 0 {
        return;
    }
    let mut colors = highlight_vec(unsafe { &mut *pstate.colors });
    debug_assert!(
        colors.is_empty() || {
            let last = colors.last();
            last.start.line < start.line || last.end_col <= start.col
        },
        "highlight chunks must be recorded in order"
    );
    colors.push(ParserHighlightChunk {
        start,
        end_col: start.col.wrapping_add(len),
        group,
    });
}

/// Release everything the parser allocated: the converted input lines and the
/// two collections' heap buffers. The un-converted lines belong to whoever
/// supplied the getter.
pub fn viml_parser_destroy(pstate: &mut ParserState) {
    unsafe {
        for pline in lines_vec(&mut pstate.reader.lines).as_slice() {
            if pline.allocated {
                xfree(pline.data as *mut c_void);
            }
        }
        xfree(lines_vec(&mut pstate.reader.lines).take_heap());
        xfree(stack_vec(&mut pstate.stack).take_heap());
    }
}

/// A `ParserLineGetter` over a null-terminated array of ready-made lines; the
/// cookie is a cursor into it and is advanced past each line handed out.
pub unsafe extern "C" fn parser_simple_get_line(cookie: *mut c_void, ret_pline: *mut ParserLine) {
    unsafe {
        let plines = cookie as *mut *mut ParserLine;
        *ret_pline = **plines;
        *plines = (*plines).add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The cursor stops at the start of the next line rather than running off
    /// the end of this one, and a `len` that lands exactly on the end still
    /// wraps.
    #[test]
    fn advance_wraps_to_the_next_line() {
        let mut lines = [ParserLine {
            data: c"abcd".as_ptr(),
            size: 4,
            allocated: false,
        }];
        let mut pstate = PARSER_STATE_INIT;
        let items = lines.as_mut_ptr();
        pstate.reader.lines.items = items;
        pstate.reader.lines.size = 1;
        pstate.reader.lines.capacity = 1;

        viml_parser_advance(&mut pstate, 2);
        assert_eq!((pstate.pos.line, pstate.pos.col), (0, 2));
        viml_parser_advance(&mut pstate, 2);
        assert_eq!((pstate.pos.line, pstate.pos.col), (1, 0));
    }

    /// The whole loop `expressions.rs` runs: pull lines through the getter,
    /// walk the cursor across them, and tear the state down. The terminating
    /// entry of a `parser_simple_get_line` array is a null line.
    #[test]
    fn reads_lines_through_the_getter_until_the_null_terminator() {
        let mut input = [
            ParserLine {
                data: c"ab".as_ptr(),
                size: 2,
                allocated: false,
            },
            ParserLine {
                data: c"cde".as_ptr(),
                size: 3,
                allocated: false,
            },
            EMPTY_LINE,
        ];
        let mut cursor = input.as_mut_ptr();
        let mut pstate = PARSER_STATE_INIT;
        viml_parser_init(
            &mut pstate,
            Some(parser_simple_get_line),
            &raw mut cursor as *mut c_void,
            ptr::null_mut(),
        );

        let first = viml_parser_get_remaining_line(&mut pstate).expect("first line");
        assert_eq!(first.size, 2);
        viml_parser_advance(&mut pstate, 1);
        // Still on the same line, one byte in: the remainder is shorter and
        // starts later, but it is the same buffer.
        let rest = viml_parser_get_remaining_line(&mut pstate).expect("rest of the first line");
        assert_eq!(rest.size, 1);
        assert_eq!(rest.data, first.data.wrapping_add(1));

        viml_parser_advance(&mut pstate, 1);
        assert_eq!(pstate.pos.line, 1);
        assert_eq!(
            viml_parser_get_remaining_line(&mut pstate)
                .expect("second line")
                .size,
            3
        );
        viml_parser_advance(&mut pstate, 3);
        assert!(viml_parser_get_remaining_line(&mut pstate).is_none());

        assert_eq!(pstate.reader.lines.size, 3);
        viml_parser_destroy(&mut pstate);
    }

    /// Highlighting is off unless the caller supplies a chunk log, and chunks
    /// accumulate in the order they are recorded.
    #[test]
    fn highlight_appends_only_when_colors_were_requested() {
        let mut colors = ParserHighlight {
            size: 0,
            capacity: 0,
            items: ptr::null_mut(),
            init_array: [ParserHighlightChunk {
                start: ParserPosition { line: 0, col: 0 },
                end_col: 0,
                group: ptr::null(),
            }; 16],
        };
        colors.capacity = colors.init_array.len();
        colors.items = colors.init_array.as_mut_ptr();

        let mut pstate = PARSER_STATE_INIT;
        viml_parser_init(&mut pstate, None, ptr::null_mut(), ptr::null_mut());
        viml_parser_highlight(
            &mut pstate,
            ParserPosition { line: 0, col: 0 },
            3,
            c"A".as_ptr(),
        );
        assert_eq!(colors.size, 0);

        pstate.colors = &raw mut colors;
        viml_parser_highlight(
            &mut pstate,
            ParserPosition { line: 0, col: 0 },
            3,
            c"A".as_ptr(),
        );
        // A zero-length chunk is dropped rather than recorded.
        viml_parser_highlight(
            &mut pstate,
            ParserPosition { line: 0, col: 3 },
            0,
            c"B".as_ptr(),
        );
        viml_parser_highlight(
            &mut pstate,
            ParserPosition { line: 0, col: 3 },
            2,
            c"C".as_ptr(),
        );
        let recorded = highlight_vec(&mut colors);
        assert_eq!(recorded.len(), 2);
        assert_eq!(recorded.as_slice()[0].end_col, 3);
        assert_eq!(recorded.as_slice()[1].start.col, 3);
        assert_eq!(recorded.as_slice()[1].end_col, 5);
    }
}
