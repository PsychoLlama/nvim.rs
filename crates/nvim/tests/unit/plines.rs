//! How wide a character is on screen, and how many rows a line takes.
//!
//! The oracle the `plines/` retype is checked against. Every editing module
//! and `win_line` ask `plines/` the same two questions -- the cells one
//! character takes at a given virtual column (`charsize_fast` when nothing
//! decorates the line, `charsize_regular` when 'linebreak', 'showbreak',
//! 'breakindent' or inline virtual text may add cells), and the rows a line
//! or a range of lines fills (`plines_win_nofill`, `win_text_height`). The
//! module is `pub(crate)`, so the cases ask through what the editor exports
//! over it: `virtcol()`/`strdisplaywidth()` (`getvcol` and `linetabsize`),
//! `screenpos()`, and `nvim_win_text_height()`. Each answer is today's, and
//! upstream 0.12.6 answers the same in an 80-column window.
//!
//! The unit editor's window is 0 x 0 (nothing ever sized the screen), and
//! with no width neither wrapping nor 'linebreak' means anything: so every
//! case gives the current window 80 x 22 for its own life and takes it back
//! to nothing on the way out.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};
use neovim::window::{win_new_height, win_new_width};
use neovim::winlayer::Win;

/// The current window at 80 x 22 for the value's life.
struct Sized;

impl Sized {
    fn new(_fx: &Probe<'_>) -> Sized {
        win_new_width(Win::current(), 80);
        win_new_height(Win::current(), 22);
        Sized
    }
}

impl Drop for Sized {
    fn drop(&mut self) {
        win_new_width(Win::current(), 0);
        win_new_height(Win::current(), 0);
    }
}

fn fresh(fx: &Probe<'_>, lines: &str) {
    fx.home();
    fx.run(
        "setlocal tabstop=8 vartabstop= nolist listchars=tab:>-,eol:$ nolinebreak breakat& \
         showbreak= nobreakindent breakindentopt= nonumber norelativenumber wrap \
         signcolumn=no foldcolumn=0",
    );
    fx.run(&format!("call setline(1, {lines})"));
    fx.run("call cursor(1, 1)");
}

/// The window and buffer every other case expects: these options are local
/// to window 1 and buffer 1, which outlive the case.
fn restore(fx: &Probe<'_>) {
    fx.run(
        "setlocal tabstop& vartabstop& list& listchars& linebreak& breakat& showbreak& \
         breakindent& breakindentopt& number& relativenumber& wrap& signcolumn& foldcolumn&",
    );
    fx.home();
}

#[test]
fn tabs_and_wide_characters_by_virtcol() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    let _sized = Sized::new(&fx);
    fresh(
        &fx,
        "[\"a\\tb\", \"\\t\\tx\", '日本x', \"e\\u0301x\", \"a\\x01b\"]",
    );
    fx.expect_all(&[
        // virtcol() is the *last* cell a character takes; [start, end] the
        // whole span.
        ("virtcol([1, 1])", "1"),
        ("virtcol([1, 2])", "8"),
        ("virtcol([1, 2], 1)", "[2, 8]"),
        ("virtcol([1, 3])", "9"),
        ("virtcol([2, 2], 1)", "[9, 16]"),
        ("virtcol([2, '$'])", "18"),
        // Double width: two cells each.
        ("virtcol([3, 1], 1)", "[1, 2]"),
        ("virtcol([3, 4], 1)", "[3, 4]"),
        ("virtcol([3, 7])", "5"),
        // A combining mark takes no cell of its own.
        ("virtcol([4, 4])", "2"),
        // A control character is `^A`, two cells.
        ("virtcol([5, 2], 1)", "[2, 3]"),
        ("strdisplaywidth(getline(1))", "9"),
        ("strdisplaywidth(getline(3))", "5"),
        ("strdisplaywidth(getline(5))", "4"),
        // A TAB's width depends on the column it starts at.
        ("strdisplaywidth(\"\\t\", 5)", "3"),
        ("strdisplaywidth(\"\\t\", 8)", "8"),
    ]);
    // 'vartabstop' sets each stop.
    fx.run("setlocal vartabstop=4,2");
    fx.expect_all(&[("virtcol([1, 2])", "4"), ("virtcol([2, '$'])", "8")]);
    // 'list' without a "tab:" item shows a TAB as `^I`; with one, at its
    // full width.
    fx.run("setlocal vartabstop= list listchars=eol:$");
    fx.check("virtcol([1, 2], 1)", "[2, 3]");
    fx.run("setlocal listchars=tab:>-");
    fx.check("virtcol([1, 2], 1)", "[2, 8]");
    fx.run("setlocal nolist listchars&");
    restore(&fx);
}

#[test]
fn linebreak_and_showbreak_add_cells_where_a_line_wraps() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    let _sized = Sized::new(&fx);
    // Words of eleven and a space: the seventh takes columns 73 to 83, so it
    // does not fit the first row.
    fresh(&fx, "[repeat('abcdefghijk ', 12)]");
    fx.check("winwidth(0)", "80");
    let at = |col: u32| {
        format!(
            "[virtcol([1, {col}]), screenpos(win_getid(), 1, {col})->get('row'), screenpos(win_getid(), 1, {col})->get('col')]"
        )
    };
    fx.expect_all(&[(&at(75), "[75, 1, 75]"), (&at(85), "[85, 2, 5]")]);
    // 'linebreak' moves the word at the edge down: the space in front of it
    // is widened to the end of the row, and every column after counts that.
    fx.run("setlocal linebreak");
    fx.expect_all(&[
        (&at(72), "[80, 1, 72]"),
        (&at(73), "[81, 2, 1]"),
        (&at(85), "[93, 2, 13]"),
    ]);
    // 'showbreak' puts its text in front of every continuation row.
    fx.run("setlocal nolinebreak showbreak=>>>");
    fx.expect_all(&[(&at(80), "[80, 1, 80]"), (&at(81), "[84, 2, 4]")]);
    // 'breakindent' repeats the line's indent there; 'breakindentopt' shift
    // adds to it.
    fx.run("setlocal showbreak= breakindent");
    fx.run("call setline(1, '    ' . repeat('x', 100))");
    fx.check(&at(81), "[85, 2, 5]");
    fx.run("setlocal breakindentopt=shift:2");
    fx.check(&at(81), "[87, 2, 7]");
    restore(&fx);
}

/// `nvim_win_text_height(0, range)` as `[all, end_row, end_vcol]`.
fn height(range: &str) -> String {
    format!("map(['all', 'end_row', 'end_vcol'], {{_, k -> nvim_win_text_height(0, {range})[k]}})")
}

#[test]
fn the_rows_a_line_takes() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    let _sized = Sized::new(&fx);
    fresh(
        &fx,
        "['short', repeat('x', 80), repeat('x', 81), repeat('x', 250), '', \"\\t\" . repeat('y', 75)]",
    );
    fx.expect_all(&[
        (&height("{}"), "[11, 5, 83]"),
        // Exactly the width is one row; one more is two.
        (&height("{'start_row': 1, 'end_row': 1}"), "[1, 1, 80]"),
        (&height("{'start_row': 2, 'end_row': 2}"), "[2, 2, 81]"),
        (&height("{'start_row': 3, 'end_row': 3}"), "[4, 3, 250]"),
        // An empty line is a row.
        (&height("{'start_row': 4, 'end_row': 4}"), "[1, 4, 0]"),
        // A TAB at the start pushes the 75 `y`s over the edge.
        (&height("{'start_row': 5, 'end_row': 5}"), "[2, 5, 83]"),
        // Part of a line, by virtual column.
        (
            &height("{'start_row': 3, 'start_vcol': 100, 'end_row': 3, 'end_vcol': 170}"),
            "[2, 3, 170]",
        ),
        // `max_height` stops at the line that reaches it, counted whole.
        (&height("{'max_height': 3}"), "[4, 2, 80]"),
    ]);
    // 'number' narrows the text and so adds rows.
    fx.run("setlocal number");
    fx.expect_all(&[
        (&height("{'start_row': 1, 'end_row': 1}"), "[2, 1, 80]"),
        (&height("{'start_row': 3, 'end_row': 3}"), "[4, 3, 250]"),
    ]);
    // 'nowrap': every line is one row.
    fx.run("setlocal nonumber nowrap");
    fx.check(&height("{}"), "[6, 5, 83]");
    restore(&fx);
}

#[test]
fn inline_virtual_text_takes_cells_in_the_line() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    let _sized = Sized::new(&fx);
    fresh(&fx, "['abcdef', repeat('z', 78)]");
    fx.run("let g:ns = nvim_create_namespace('unit_plines')");
    fx.run("call nvim_buf_set_extmark(0, g:ns, 0, 2, {'virt_text': [['[12345]']], 'virt_text_pos': 'inline'})");
    fx.run("call nvim_buf_set_extmark(0, g:ns, 1, 70, {'virt_text': [['<inline>']], 'virt_text_pos': 'inline'})");
    fx.expect_all(&[
        // The text before the mark is where it was; the character it sits in
        // front of starts after the seven cells.
        ("virtcol([1, 2])", "2"),
        ("virtcol([1, 3], 1)", "[3, 10]"),
        ("virtcol([1, 4])", "11"),
        // strdisplaywidth() has no window and sees none of it.
        ("strdisplaywidth(getline(1))", "6"),
        // The second line now wraps.
        (&height("{'start_row': 1, 'end_row': 1}"), "[2, 1, 86]"),
    ]);
    fx.run("call nvim_buf_clear_namespace(0, g:ns, 0, -1) | unlet g:ns");
    restore(&fx);
}
