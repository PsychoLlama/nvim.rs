//! The window match list: `matchadd()`, `matchaddpos()`, `matchdelete()`,
//! `getmatches()`, `setmatches()`, `clearmatches()` and `:match`.
//!
//! Each window keeps its matches in one list ordered by priority, ties in
//! the order they were added, and hands out ids from 1000 (1–3 belong to
//! `:match`, `:2match` and `:3match`). `getmatches()` reads the list back in
//! that order, and `setmatches()` replaces it wholesale. The cases pin the
//! ids, the order, the errors and the round trip.
//!
//! Written against the behaviour before the list stops being an intrusive
//! chain of `MatchItem`s on the window; a row that moves is a behaviour
//! change. Every case works in a window of its own, so the id counter it
//! reads starts fresh whatever ran before it.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};

/// `getmatches()` with each dictionary as its sorted `items()`: `string()`
/// of a dictionary follows the hash order.
const MATCHES: &str = "map(getmatches(), {_, m -> sort(items(m))})";

/// A window of the case's own, holding `lines`.
fn window(fx: &Probe, lines: &str) {
    fx.home();
    fx.run("new");
    fx.run(&format!("call setline(1, {lines})"));
}

fn restore(fx: &Probe) {
    fx.run("silent! clearmatches()");
    fx.run("set nomodified");
    fx.home();
}

#[test]
fn ids_count_from_a_thousand_and_the_list_is_by_priority() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    window(&fx, "['foo bar baz']");
    fx.expect_all(&[
        ("matchadd('Search', 'foo')", "1000"),
        ("matchadd('Error', 'bar', 5)", "1001"),
        ("matchadd('Todo', 'baz', 20, 42)", "42"),
        // The counter does not skip an id a caller chose.
        ("matchadd('Search', 'ba')", "1002"),
        (
            "map(getmatches(), {_, m -> [m.id, m.priority, m.pattern]})",
            "[[1001, 5, 'bar'], [1000, 10, 'foo'], [1002, 10, 'ba'], [42, 20, 'baz']]",
        ),
    ]);
    restore(&fx);
}

#[test]
fn reserved_and_duplicate_ids_are_refused() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    window(&fx, "['x']");
    fx.check("matchadd('Search', 'x', 10, 7)", "7");
    fx.check_error(
        "call matchadd('Search', 'x', 10, 7)",
        "E801: ID already taken: 7",
    );
    fx.check_error(
        "call matchadd('Search', 'x', 10, 2)",
        "E798: ID is reserved for \":match\": 2",
    );
    fx.check_error(
        "call matchadd('Search', 'x', 10, 0)",
        "E799: Invalid ID: 0 (must be greater than or equal to 1)",
    );
    // An empty group or pattern adds nothing, quietly.
    fx.check("matchadd('', 'x')", "-1");
    fx.check("matchadd('Search', '')", "-1");
    fx.check(
        MATCHES,
        "[[['group', 'Search'], ['id', 7], ['pattern', 'x'], ['priority', 10]]]",
    );
    restore(&fx);
}

#[test]
fn positions_are_lines_columns_and_lengths() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    window(&fx, "['one', 'two', 'three']");
    fx.check("matchaddpos('Search', [2, [3, 2], [1, 1, 2]])", "1000");
    fx.check(
        MATCHES,
        "[[['group', 'Search'], ['id', 1000], ['pos1', [2]], ['pos2', [3, 2, 1]], \
         ['pos3', [1, 1, 2]], ['priority', 10]]]",
    );
    // An empty list adds nothing and answers -1; an empty position is an
    // error, after the id was taken; a fourth number in a position is
    // ignored.
    fx.check("matchaddpos('Search', [])", "-1");
    fx.check_error(
        "call matchaddpos('Search', [[]])",
        "E5030: Empty list at position 0",
    );
    fx.check("matchaddpos('Search', [[1, 2, 3, 4]])", "1002");
    fx.check("getmatches()[1].pos1", "[1, 2, 3]");
    restore(&fx);
}

#[test]
fn delete_and_clear() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    window(&fx, "['abc']");
    fx.run("call matchadd('Search', 'a')");
    fx.run("call matchadd('Search', 'b')");
    fx.run("call matchadd('Search', 'c')");
    fx.check("matchdelete(1001)", "0");
    fx.check("map(getmatches(), 'v:val.id')", "[1000, 1002]");
    fx.check_error("call matchdelete(1001)", "E803: ID not found: 1001");
    fx.check_error("call matchdelete(2)", "E803: ID not found: 2");
    fx.check_error(
        "call matchdelete(0)",
        "E802: Invalid ID: 0 (must be greater than or equal to 1)",
    );
    fx.run("call clearmatches()");
    fx.check("getmatches()", "[]");
    // The counter is the window's, and clearing the list does not reset it.
    fx.check("matchadd('Search', 'a')", "1003");
    restore(&fx);
}

#[test]
fn set_replaces_the_list_and_round_trips_get() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    window(&fx, "['alpha beta', 'gamma']");
    fx.run("call matchadd('Search', 'alpha', 3)");
    fx.run("call matchaddpos('Error', [[2, 1, 3]], 7, 50)");
    fx.run("call matchadd('Conceal', 'beta', 10, -1, {'conceal': '*'})");
    fx.run("let g:unit_match_saved = getmatches()");
    fx.run("call clearmatches()");
    fx.run("call matchadd('Todo', 'gamma')");
    fx.check("setmatches(g:unit_match_saved)", "0");
    fx.check("getmatches() == g:unit_match_saved", "1");
    fx.check(
        MATCHES,
        "[[['group', 'Search'], ['id', 1000], ['pattern', 'alpha'], ['priority', 3]], \
         [['group', 'Error'], ['id', 50], ['pos1', [2, 1, 3]], ['priority', 7]], \
         [['conceal', '*'], ['group', 'Conceal'], ['id', 1001], ['pattern', 'beta'], ['priority', 10]]]",
    );
    fx.check_error(
        "call setmatches([{'x': 1}])",
        "E474: List item 0 is missing one of the required keys",
    );
    fx.check_error("call setmatches(1)", "E714: List required");
    fx.run("unlet g:unit_match_saved");
    restore(&fx);
}

#[test]
fn colon_match_takes_the_reserved_ids() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    window(&fx, "['one two three']");
    fx.run("match Search /one/");
    fx.run("2match Error /two/");
    fx.run("3match Todo /three/");
    fx.run("call matchadd('Search', 'e', 0)");
    fx.expect_all(&[
        ("matcharg(1)", "['Search', 'one']"),
        ("matcharg(2)", "['Error', 'two']"),
        ("matcharg(3)", "['Todo', 'three']"),
        ("matcharg(4)", "[]"),
        (
            "map(getmatches(), {_, m -> [m.id, m.priority, m.group]})",
            "[[1000, 0, 'Search'], [1, 10, 'Search'], [2, 10, 'Error'], [3, 10, 'Todo']]",
        ),
    ]);
    fx.run("2match none");
    fx.check("matcharg(2)", "['', '']");
    fx.check("map(getmatches(), 'v:val.id')", "[1000, 1, 3]");
    fx.run("match");
    fx.run("3match none");
    restore(&fx);
}

#[test]
fn another_windows_list_by_id() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    window(&fx, "['left']");
    fx.run("let g:unit_match_win = win_getid()");
    fx.run("new");
    fx.check(
        "matchadd('Search', 'left', 10, -1, {'window': g:unit_match_win})",
        "1000",
    );
    fx.check("getmatches()", "[]");
    fx.check(
        "map(getmatches(g:unit_match_win), 'v:val.pattern')",
        "['left']",
    );
    fx.check("matchdelete(1000, g:unit_match_win)", "0");
    fx.check("getmatches(g:unit_match_win)", "[]");
    fx.check_error(
        "call matchadd('Search', 'x', 10, -1, {'window': 999999})",
        "E957: Invalid window number",
    );
    fx.run("unlet g:unit_match_win");
    restore(&fx);
}
