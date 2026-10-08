//! The VimL parser scaffolding, driven the way `expressions.rs` drives it:
//! the reader over the caller's lines, the cursor across them, and the
//! highlight log.

use neovim::types::{ParserPosition, ParserState};
use neovim::viml::parser::expressions::viml_pexpr_parse;

/// Parse a real expression end to end over a parser that keeps no
/// highlight log.
#[test]
fn parses_an_expression_without_a_highlight_log() {
    let input: [&[u8]; 1] = [b"1 + 2 * abs(-3)"];
    let mut pstate = ParserState::new(&input, false);
    let ast = viml_pexpr_parse(&mut pstate, 0);
    assert!(ast.err.is_none());
    assert!(ast.root.is_some());
    assert!(pstate.take_highlight().is_empty());
}

/// The whole loop `expressions.rs` runs: read the lines in order and walk
/// the cursor across them, until the input runs out.
#[test]
fn reads_the_lines_in_order_until_the_input_ends() {
    let input: [&[u8]; 2] = [b"ab", b"cde"];
    let mut pstate = ParserState::new(&input, false);

    let first = pstate.remaining_line().expect("first line");
    assert_eq!(first, b"ab");
    pstate.advance(1);
    // Still on the same line, one byte in: the remainder is shorter and
    // starts later, but it is the same line.
    let rest = pstate.remaining_line().expect("rest of the first line");
    assert_eq!(rest, b"b");
    assert_eq!(rest.as_ptr(), first.as_ptr().wrapping_add(1));

    pstate.advance(1);
    assert_eq!(pstate.pos.line, 1);
    assert_eq!(pstate.remaining_line().expect("second line"), b"cde");
    pstate.advance(3);
    assert!(pstate.remaining_line().is_none());

    // Both lines and the absent one past them.
    assert_eq!(pstate.lines_read(), 3);
}

/// Highlighting is off unless the caller asks for a chunk log, and chunks
/// accumulate in the order they are recorded.
#[test]
fn highlight_appends_only_when_colors_were_requested() {
    let input: [&[u8]; 0] = [];
    let mut pstate = ParserState::new(&input, false);
    pstate.highlight(ParserPosition { line: 0, col: 0 }, 3, c"A");
    assert_eq!(pstate.highlight_count(), None);
    assert!(pstate.take_highlight().is_empty());

    let mut pstate = ParserState::new(&input, true);
    pstate.highlight(ParserPosition { line: 0, col: 0 }, 3, c"A");
    // A zero-length chunk is dropped rather than recorded.
    pstate.highlight(ParserPosition { line: 0, col: 3 }, 0, c"B");
    pstate.highlight(ParserPosition { line: 0, col: 3 }, 2, c"C");
    let recorded = pstate.take_highlight();
    assert_eq!(recorded.len(), 2);
    assert_eq!(recorded[0].end_col, 3);
    assert_eq!(recorded[1].start.col, 3);
    assert_eq!(recorded[1].end_col, 5);
    assert_eq!(recorded[1].group, c"C");
}
