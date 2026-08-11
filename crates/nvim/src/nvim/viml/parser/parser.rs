//! Scaffolding shared by the VimL parsers: the reader that feeds them lines,
//! the cursor over those lines, and the highlight log they append to.
//!
//! `ParserState` and everything it embeds is `repr(C)` and frozen —
//! `expressions.rs`, `api/vimscript.rs` and `ex_getln.rs` all read the fields
//! directly, and the three collections inside it are `kvec_withinit_t`s
//! (see `kvec::InitVec`).
//!
//! The entry points take `*mut ParserState` rather than `&mut ParserState`
//! deliberately. Two of the collections point at arrays inside the state, and
//! `expressions.rs` pushes onto the stack through the raw pointer it holds;
//! a `&mut` to the whole state invalidates those self-pointers, which Miri
//! reports as soon as the parser is driven end to end. Reborrows here are
//! narrowed to the one collection being touched.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_void};
use core::ptr;

use crate::src::nvim::kvec::InitVec;
use crate::src::nvim::mbyte::string_convert;
use crate::src::nvim::memory::xfree;
use crate::src::nvim::types::{
    CONV_NONE, ParserHighlight, ParserHighlightChunk, ParserInputReader, ParserInputReader_lines,
    ParserLine, ParserLineGetter, ParserPosition, ParserState, ParserState_stack, ParserStateItem,
    ParserStateItem_data, ParserStateItem_data_expr, ParserStateItem_data_expr_type_0,
    ParserStateItem_type_0, vimconv_T,
};

pub const kExprUnknown: ParserStateItem_data_expr_type_0 = 0;
pub const kPTopStateParsingCommand: ParserStateItem_type_0 = 0;
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

/// The caller's highlight log. Everything that reads or rewrites a recorded
/// chunk must come through here rather than through `colors.items`: while the
/// collection is inline that pointer carries whatever provenance the last
/// `viml_parser_highlight` left behind, and the view re-derives it.
pub fn highlight_vec(colors: &mut ParserHighlight) -> InitVec<'_, ParserHighlightChunk> {
    InitVec::new(
        &mut colors.size,
        &mut colors.capacity,
        &mut colors.items,
        &mut colors.init_array,
    )
}

/// The `index`th line the reader has read. Every line the parser has seen
/// stays in the collection for the duration of the parse, because tokens and
/// AST nodes point into them.
///
/// The read goes through the inline array where the collection has not left
/// it: `items` then holds the state's own address with whatever provenance
/// `viml_parser_init` gave it, which nothing on this path can be sure of.
pub fn reader_line(reader: &ParserInputReader, index: usize) -> ParserLine {
    let lines = &reader.lines;
    debug_assert!(
        index < lines.size,
        "the parser has not read that many lines"
    );
    if lines.items.cast_const() == lines.init_array.as_ptr() {
        return lines.init_array[index];
    }
    // SAFETY: the collection is on the heap, where `items` carries the
    // allocation's own provenance and addresses `size` lines.
    unsafe { *lines.items.add(index) }
}

/// Start a parser over the lines `get_line` yields. `colors` may be null when
/// the caller does not want highlighting.
///
/// # Safety
/// `pstate` must point at writable, suitably aligned storage for a
/// `ParserState`, which must then stay put: both collections end up holding
/// the state's own address.
pub unsafe fn viml_parser_init(
    pstate: *mut ParserState,
    get_line: ParserLineGetter,
    cookie: *mut c_void,
    colors: *mut ParserHighlight,
) {
    let init = ParserState {
        reader: ParserInputReader {
            get_line,
            cookie,
            ..PARSER_STATE_INIT.reader
        },
        colors,
        ..PARSER_STATE_INIT
    };
    // SAFETY: the caller's obligation. `kvi_init`'s two self-pointers are
    // written through `pstate` itself rather than through `InitVec::init`:
    // `items` outlives every borrow the parser goes on to take, so it must
    // carry the provenance of the raw pointer the caller owns and not of a
    // `&mut` that dies at the end of this call.
    unsafe {
        pstate.write(init);
        let lines = &raw mut (*pstate).reader.lines;
        (*lines).capacity = (*lines).init_array.len();
        (*lines).items = (&raw mut (*lines).init_array).cast::<ParserLine>();
        let stack = &raw mut (*pstate).stack;
        (*stack).capacity = (*stack).init_array.len();
        (*stack).items = (&raw mut (*stack).init_array).cast::<ParserStateItem>();
    }
}

/// The rest of the current line, from the cursor on, reading one more line
/// from the input if the cursor just walked off the end of the last one.
/// `None` at end of input.
///
/// # Safety
/// `pstate` must point at a parser initialised by [`viml_parser_init`].
pub unsafe fn viml_parser_get_remaining_line(pstate: *mut ParserState) -> Option<ParserLine> {
    // SAFETY: the caller's obligation. `pos` is copied out and the reader is
    // reborrowed on its own, so the reborrow does not reach the stack the
    // caller may be pushing onto.
    let (pos, reader) = unsafe { ((*pstate).pos, &mut (*pstate).reader) };
    remaining_line(pos, reader)
}

/// [`viml_parser_get_remaining_line`] over the two parts of the state it
/// actually touches.
fn remaining_line(pos: ParserPosition, reader: &mut ParserInputReader) -> Option<ParserLine> {
    let mut pline = if pos.line == reader.lines.size {
        // Every line the parser has seen stays in `lines` for the duration,
        // because tokens point into them.
        let fresh = read_line(reader);
        lines_vec(&mut reader.lines).push(fresh);
        fresh
    } else {
        lines_vec(&mut reader.lines).last()
    };
    debug_assert!(pos.line == reader.lines.size - 1);
    if pline.data.is_null() {
        return None;
    }
    // `wrapping_*` because the C did: the cursor is never past the line's end
    // (`viml_parser_advance` wraps to the next line first), so this is exact.
    pline.data = pline.data.wrapping_add(pos.col);
    pline.size = pline.size.wrapping_sub(pos.col);
    Some(pline)
}

/// `viml_preader_get_line`: pull the next line out of the getter, converting
/// it if the reader carries a conversion.
fn read_line(reader: &mut ParserInputReader) -> ParserLine {
    let mut pline = EMPTY_LINE;
    let get_line = reader.get_line.expect("parser has no line getter");
    // SAFETY: the getter is the caller's own, registered at `viml_parser_init`,
    // and writes one `ParserLine` through the pointer it is handed.
    unsafe { get_line(reader.cookie, &raw mut pline) };
    if reader.conv.vc_type == CONV_NONE || pline.size == 0 {
        return pline;
    }
    let mut size = pline.size;
    // SAFETY: `string_convert` reads `size` bytes from the getter's line and
    // answers a fresh allocation, writing the converted length back.
    let data =
        unsafe { string_convert(&raw mut reader.conv, pline.data as *mut c_char, &mut size) };
    if pline.allocated {
        // SAFETY: the getter said the line is on the heap, and it has just
        // been copied out of.
        unsafe { xfree(pline.data as *mut c_void) };
    }
    ParserLine {
        data,
        size,
        allocated: true,
    }
}

/// Advance the cursor by `len` bytes, at most to the start of the next line.
///
/// Takes the cursor and the reader rather than the whole `ParserState`, so
/// that the reborrow does not reach the stack the caller is pushing onto.
pub fn viml_parser_advance(pos: &mut ParserPosition, reader: &mut ParserInputReader, len: usize) {
    debug_assert!(pos.line == reader.lines.size - 1);
    let pline = lines_vec(&mut reader.lines).last();
    if pos.col.wrapping_add(len) >= pline.size {
        pos.line += 1;
        pos.col = 0;
    } else {
        pos.col = pos.col.wrapping_add(len);
    }
}

/// Record the highlighting of `len` bytes at `start`. A no-op when the caller
/// asked for no highlighting. Chunks must arrive in order and must not
/// overlap.
///
/// # Safety
/// `pstate` must point at a parser initialised by [`viml_parser_init`], and
/// `group` must outlive the highlight log.
pub unsafe fn viml_parser_highlight(
    pstate: *mut ParserState,
    start: ParserPosition,
    len: usize,
    group: *const c_char,
) {
    // SAFETY: the caller's obligation.
    let colors = unsafe { (*pstate).colors };
    if colors.is_null() || len == 0 {
        return;
    }
    // `colors` is the caller's own collection, not one embedded in the state,
    // so reborrowing it does not reach anything else the parser holds.
    // SAFETY: non-null, and the caller owns it for the whole parse.
    let mut colors = highlight_vec(unsafe { &mut *colors });
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
    for pline in lines_vec(&mut pstate.reader.lines).as_slice() {
        if pline.allocated {
            // SAFETY: a converted line was allocated by `read_line` and is
            // reachable only from here.
            unsafe { xfree(pline.data as *mut c_void) };
        }
    }
    let lines = lines_vec(&mut pstate.reader.lines).take_heap();
    let stack = stack_vec(&mut pstate.stack).take_heap();
    // SAFETY: each buffer left the inline array through `InitVec`, so it is a
    // live allocation or null.
    unsafe { [lines, stack].into_iter().for_each(|buffer| xfree(buffer)) };
}

/// A `ParserLineGetter` over a null-terminated array of ready-made lines; the
/// cookie is a cursor into it and is advanced past each line handed out.
///
/// # Safety
/// `cookie` must point at a `*mut ParserLine` cursor into an array whose last
/// entry has a null `data`, and `ret_pline` at writable storage for one line.
pub unsafe extern "C" fn parser_simple_get_line(cookie: *mut c_void, ret_pline: *mut ParserLine) {
    let plines = cookie as *mut *mut ParserLine;
    // SAFETY: the caller's obligation; the cursor stops at the null line.
    unsafe {
        *ret_pline = **plines;
        *plines = (*plines).add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn advance(pstate: &mut ParserState, len: usize) {
        viml_parser_advance(&mut pstate.pos, &mut pstate.reader, len);
    }

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

        advance(&mut pstate, 2);
        assert_eq!((pstate.pos.line, pstate.pos.col), (0, 2));
        advance(&mut pstate, 2);
        assert_eq!((pstate.pos.line, pstate.pos.col), (1, 0));
    }
}
