//! The VimL parser scaffolding, driven the way `expressions.rs` drives it.
//!
//! The end-to-end case exists for Miri: `ParserState` embeds two
//! `kvec_withinit_t`s whose `items` point at arrays inside the state itself,
//! and `expressions.rs` pushes onto the stack through the raw pointer it
//! holds. A `&mut ParserState` anywhere in the parser's entry points
//! invalidates those self-pointers — this test is what catches it.

use std::ffi::c_void;
use std::ptr;

use neovim::types::{ParserHighlight, ParserHighlightChunk, ParserLine, ParserPosition};
use neovim::viml::parser::expressions::{viml_pexpr_free_ast, viml_pexpr_parse};
use neovim::viml::parser::parser::{
    PARSER_STATE_INIT, highlight_vec, parser_simple_get_line, viml_parser_destroy,
    viml_parser_get_remaining_line, viml_parser_highlight, viml_parser_init,
};

const EMPTY_LINE: ParserLine = ParserLine {
    data: ptr::null(),
    size: 0,
    allocated: false,
};

fn advance(pstate: &mut neovim::types::ParserState, len: usize) {
    neovim::viml::parser::parser::viml_parser_advance(&mut pstate.pos, &mut pstate.reader, len);
}

/// Parse a real expression end to end. Nothing is asserted beyond "it
/// returns"; the point is the Stacked-Borrows check Miri runs while it does.
#[test]
fn parses_an_expression_without_invalidating_the_parser_state() {
    let mut input = [
        ParserLine {
            data: c"1 + 2 * abs(-3)".as_ptr(),
            size: 15,
            allocated: false,
        },
        EMPTY_LINE,
    ];
    let mut cursor = input.as_mut_ptr();
    let mut pstate = PARSER_STATE_INIT;
    let state = &raw mut pstate;
    unsafe {
        viml_parser_init(
            state,
            Some(parser_simple_get_line),
            &raw mut cursor as *mut c_void,
            ptr::null_mut(),
        );
        let ast = viml_pexpr_parse(state, 0);
        viml_pexpr_free_ast(ast);
        viml_parser_destroy(&mut *state);
    }
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
    let state = &raw mut pstate;
    unsafe {
        viml_parser_init(
            state,
            Some(parser_simple_get_line),
            &raw mut cursor as *mut c_void,
            ptr::null_mut(),
        );

        let first = viml_parser_get_remaining_line(state).expect("first line");
        assert_eq!(first.size, 2);
        advance(&mut *state, 1);
        // Still on the same line, one byte in: the remainder is shorter
        // and starts later, but it is the same buffer.
        let rest = viml_parser_get_remaining_line(state).expect("rest of the first line");
        assert_eq!(rest.size, 1);
        assert_eq!(rest.data, first.data.wrapping_add(1));

        advance(&mut *state, 1);
        assert_eq!((*state).pos.line, 1);
        assert_eq!(
            viml_parser_get_remaining_line(state)
                .expect("second line")
                .size,
            3
        );
        advance(&mut *state, 3);
        assert!(viml_parser_get_remaining_line(state).is_none());

        assert_eq!((*state).reader.lines.size, 3);
        viml_parser_destroy(&mut *state);
    }
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
    let state = &raw mut pstate;
    unsafe {
        viml_parser_init(state, None, ptr::null_mut(), ptr::null_mut());
        viml_parser_highlight(state, ParserPosition { line: 0, col: 0 }, 3, c"A".as_ptr());
        assert_eq!(colors.size, 0);

        (*state).colors = &raw mut colors;
        viml_parser_highlight(state, ParserPosition { line: 0, col: 0 }, 3, c"A".as_ptr());
        // A zero-length chunk is dropped rather than recorded.
        viml_parser_highlight(state, ParserPosition { line: 0, col: 3 }, 0, c"B".as_ptr());
        viml_parser_highlight(state, ParserPosition { line: 0, col: 3 }, 2, c"C".as_ptr());
        let recorded = highlight_vec(&mut colors);
        assert_eq!(recorded.len(), 2);
        assert_eq!(recorded.as_slice()[0].end_col, 3);
        assert_eq!(recorded.as_slice()[1].start.col, 3);
        assert_eq!(recorded.as_slice()[1].end_col, 5);
    }
}
