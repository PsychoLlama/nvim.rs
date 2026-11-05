//! Text objects: which region `iw`, `as`, `ip`, `i(`, `at`, `i"` and their
//! kin name, read back from a Visual selection (`'<`/`'>`) and from what an
//! operator deletes, and what each does when there is nothing to select.
//!
//! A text object that finds nothing says nothing: the operator is cancelled
//! and the buffer and cursor stay as they were. The refusal rows pin that
//! silence (every message, `:messages` around `:silent`), so a refusal that
//! starts printing -- or a selection that starts failing -- moves a row.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};

/// The last Visual selection, as `line:col-line:col`.
const SELECTION: &str = "getpos(\"'<\")[1] .. ':' .. getpos(\"'<\")[2] .. '-' .. getpos(\"'>\")[1] .. ':' .. getpos(\"'>\")[2]";
/// The buffer and the cursor, as `[lines] line:col`.
const BUFFER: &str = "string(getline(1, '$')) .. ' ' .. line('.') .. ':' .. col('.')";

/// A buffer of `lines` with the cursor at `cursor`.
fn put(fx: &Probe, lines: &str, cursor: (u32, u32)) {
    fx.run("silent! %delete _");
    fx.run(&format!("call setline(1, {lines})"));
    fx.run(&format!("call cursor({}, {})", cursor.0, cursor.1));
}

/// The selection `keys` makes in Visual mode from `cursor`, in a buffer of
/// `lines`, should be `want` (see [`SELECTION`]).
#[track_caller]
fn selects(fx: &Probe, lines: &str, cursor: (u32, u32), keys: &str, want: &str) {
    put(fx, lines, cursor);
    // Mark an impossible selection first, so one that does not happen shows.
    fx.run("call setpos(\"'<\", [0, 0, 0, 0]) | call setpos(\"'>\", [0, 0, 0, 0])");
    fx.run(&format!("exe \"normal! v{keys}\\<Esc>\""));
    fx.check(SELECTION, want);
}

/// What `keys` (an operator and an object) leave of `lines` from `cursor`
/// should be `want` (see [`BUFFER`]).
#[track_caller]
fn leaves(fx: &Probe, lines: &str, cursor: (u32, u32), keys: &str, want: &str) {
    put(fx, lines, cursor);
    fx.run(&format!("exe \"normal! {keys}\""));
    fx.check(BUFFER, want);
}

fn start(fx: &Probe) {
    fx.run("new | only!");
    fx.run("set selection& whichwrap& cpoptions& matchpairs& quoteescape& paragraphs& sections&");
}

fn finish(fx: &Probe) {
    fx.run("bwipe!");
    fx.home();
}

/// Words and WORDS: the inner object is the run of one character class,
/// the outer one takes the white space after it (or before it, at the end
/// of a line); a count takes that many more, and repeating the object in
/// Visual mode extends the selection.
#[test]
fn words_select_one_class_and_its_white_space() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    start(&fx);
    let text = "['foo.bar  baz qux']";
    for (cursor, keys, want) in [
        ((1, 2), "iw", "'1:1-1:3'"),
        ((1, 2), "aw", "'1:1-1:3'"),
        ((1, 2), "iW", "'1:1-1:7'"),
        ((1, 2), "aW", "'1:1-1:9'"),
        ((1, 4), "iw", "'1:4-1:4'"),
        ((1, 9), "iw", "'1:8-1:9'"),
        ((1, 9), "aw", "'1:8-1:12'"),
        ((1, 16), "aw", "'1:13-1:16'"),
        ((1, 2), "3iw", "'1:1-1:7'"),
        ((1, 2), "2aw", "'1:1-1:4'"),
        ((1, 2), "iwiw", "'1:1-1:4'"),
        ((1, 2), "awaw", "'1:1-1:4'"),
    ] {
        selects(&fx, text, cursor, keys, want);
    }
    for (cursor, keys, want) in [
        ((1, 2), "diw", "'[''.bar  baz qux''] 1:1'"),
        ((1, 2), "daw", "'[''.bar  baz qux''] 1:1'"),
        ((1, 2), "daW", "'[''baz qux''] 1:1'"),
        ((1, 16), "daw", "'[''foo.bar  baz''] 1:12'"),
        ((1, 2), "d5aw", "'[''''] 1:1'"),
        ((1, 2), "d9aw", "'[''foo.bar  baz qux''] 1:16'"),
    ] {
        leaves(&fx, text, cursor, keys, want);
    }
    selects(&fx, "['one', '', 'two']", (2, 1), "aw", "'2:1-3:3'");
    leaves(&fx, "['one', '', 'two']", (2, 1), "daw", "'[''one''] 1:1'");
    finish(&fx);
}

/// Sentences end at `.`, `!` or `?` followed by white space or the end of
/// the line; the outer object takes the white space after it.
#[test]
fn sentences_end_at_punctuation_and_white_space() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    start(&fx);
    let text = "['One two.  Three four!  Five', 'six?  Seven.', '', 'Eight.']";
    for (cursor, keys, want) in [
        ((1, 2), "is", "'1:1-1:8'"),
        ((1, 2), "as", "'1:1-1:10'"),
        ((1, 14), "is", "'1:11-1:21'"),
        ((1, 26), "is", "'1:24-2:4'"),
        ((1, 26), "as", "'1:24-2:6'"),
        ((2, 7), "is", "'2:7-2:12'"),
        ((1, 2), "2is", "'1:1-1:10'"),
        ((1, 2), "3as", "'1:1-2:6'"),
        ((1, 11), "is", "'1:11-1:21'"),
        ((3, 1), "is", "'3:1-3:1'"),
        ((3, 1), "as", "'3:1-4:6'"),
        ((1, 2), "isis", "'1:1-1:10'"),
    ] {
        selects(&fx, text, cursor, keys, want);
    }
    for (cursor, keys, want) in [
        (
            (1, 2),
            "dis",
            "'[''  Three four!  Five'', ''six?  Seven.'', '''', ''Eight.''] 1:1'",
        ),
        (
            (1, 2),
            "das",
            "'[''Three four!  Five'', ''six?  Seven.'', '''', ''Eight.''] 1:1'",
        ),
        (
            (1, 14),
            "das",
            "'[''One two.  Five'', ''six?  Seven.'', '''', ''Eight.''] 1:11'",
        ),
        (
            (2, 7),
            "das",
            "'[''One two.  Three four!  Five'', ''six?'', '''', ''Eight.''] 2:4'",
        ),
        (
            (4, 1),
            "das",
            "'[''One two.  Three four!  Five'', ''six?  Seven.'', '''', ''''] 4:1'",
        ),
    ] {
        leaves(&fx, text, cursor, keys, want);
    }
    finish(&fx);
}

/// Paragraphs are runs of non-blank lines; the outer object takes the blank
/// lines after (or, at the end, before) them, and a run of blank lines is a
/// paragraph of its own to the inner object.
#[test]
fn paragraphs_are_runs_of_lines() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    start(&fx);
    let text = "['a', 'b', '', '', 'c', 'd', '', 'e']";
    for (cursor, keys, want) in [
        ((1, 1), "ip", "'1:1-2:2147483647'"),
        ((1, 1), "ap", "'1:1-4:2147483647'"),
        ((3, 1), "ip", "'3:1-4:2147483647'"),
        ((3, 1), "ap", "'3:1-6:2147483647'"),
        ((5, 1), "ap", "'5:1-7:2147483647'"),
        ((8, 1), "ap", "'7:1-8:2147483647'"),
        ((1, 1), "2ip", "'1:1-4:2147483647'"),
        ((1, 1), "3ap", "'1:1-8:2147483647'"),
        ((1, 1), "ipip", "'1:1-4:2147483647'"),
        ((1, 1), "9ap", "'0:0-0:0'"),
    ] {
        selects(&fx, text, cursor, keys, want);
    }
    for (cursor, keys, want) in [
        (
            (1, 1),
            "dip",
            "'['''', '''', ''c'', ''d'', '''', ''e''] 1:1'",
        ),
        ((1, 1), "dap", "'[''c'', ''d'', '''', ''e''] 1:1'"),
        (
            (8, 1),
            "dap",
            "'[''a'', ''b'', '''', '''', ''c'', ''d''] 6:1'",
        ),
        ((5, 1), "d2ap", "'[''a'', ''b''] 2:1'"),
        (
            (1, 1),
            "d9ap",
            "'[''a'', ''b'', '''', '''', ''c'', ''d'', '''', ''e''] 1:1'",
        ),
    ] {
        leaves(&fx, text, cursor, keys, want);
    }
    selects(&fx, "['']", (1, 1), "ip", "'1:1-1:2147483647'");
    leaves(&fx, "['', '']", (1, 1), "dap", "'['''', ''''] 1:1'");
    finish(&fx);
}

/// Blocks: the inner object is what the pair encloses, the outer one the
/// pair as well; a count reaches that many levels out, and a pair can span
/// lines.
#[test]
fn blocks_select_what_a_pair_encloses() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    start(&fx);
    let text = "['f(a, (b), [c]) {x <y>}']";
    for (cursor, keys, want) in [
        ((1, 4), "i(", "'1:3-1:13'"),
        ((1, 4), "a(", "'1:2-1:14'"),
        ((1, 9), "ib", "'1:3-1:13'"),
        ((1, 9), "2i(", "'0:0-0:0'"),
        ((1, 9), "2a)", "'0:0-0:0'"),
        ((1, 14), "i[", "'0:0-0:0'"),
        ((1, 14), "a]", "'0:0-0:0'"),
        ((1, 20), "i{", "'1:17-1:21'"),
        ((1, 20), "aB", "'1:16-1:22'"),
        ((1, 22), "i<", "'0:0-0:0'"),
        ((1, 22), "a>", "'0:0-0:0'"),
        ((1, 2), "i(", "'1:3-1:13'"),
        ((1, 9), "i(i(", "'0:0-0:0'"),
    ] {
        selects(&fx, text, cursor, keys, want);
    }
    for (cursor, keys, want) in [
        ((1, 4), "di(", "'[''f() {x <y>}''] 1:3'"),
        ((1, 4), "da(", "'[''f {x <y>}''] 1:2'"),
        ((1, 20), "ci{z", "'[''f(a, (b), [c]) {z}''] 1:17'"),
        ((1, 9), "d2i(", "'[''f(a, (b), [c]) {x <y>}''] 1:9'"),
    ] {
        leaves(&fx, text, cursor, keys, want);
    }
    let lines = "['if (x) {', '  a;', '  b;', '}']";
    selects(&fx, lines, (2, 3), "i{", "'2:1-3:5'");
    selects(&fx, lines, (2, 3), "a{", "'1:8-4:1'");
    leaves(&fx, lines, (2, 3), "di{", "'[''if (x) {'', ''}''] 2:1'");
    finish(&fx);
}

/// Tag blocks: the inner object is between a matching start and end tag,
/// the outer one the tags as well; a count reaches enclosing tags.
#[test]
fn tag_blocks_select_between_matching_tags() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    start(&fx);
    let text = "['<a><b>x</b> <i>y</i></a>']";
    for (cursor, keys, want) in [
        ((1, 9), "it", "'1:7-1:7'"),
        ((1, 9), "at", "'1:4-1:11'"),
        ((1, 9), "2it", "'1:4-1:20'"),
        ((1, 9), "2at", "'1:1-1:24'"),
        ((1, 5), "it", "'1:7-1:7'"),
        ((1, 2), "it", "'1:4-1:20'"),
        ((1, 14), "it", "'1:16-1:16'"),
        ((1, 9), "itit", "'1:4-1:11'"),
    ] {
        selects(&fx, text, cursor, keys, want);
    }
    for (cursor, keys, want) in [
        ((1, 9), "dit", "'[''<a><b></b> <i>y</i></a>''] 1:7'"),
        ((1, 9), "dat", "'[''<a> <i>y</i></a>''] 1:4'"),
        ((1, 9), "d2it", "'[''<a></a>''] 1:4'"),
    ] {
        leaves(&fx, text, cursor, keys, want);
    }
    finish(&fx);
}

/// Quotes: the inner object is between the quote pair on the line, the
/// outer one the quotes and the white space after them; a backslash in
/// 'quoteescape' escapes a quote, and an inner object with a count of two
/// takes the quotes too.
#[test]
fn quotes_select_within_the_line() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    start(&fx);
    let text = r#"['x "a b" y ''c\''d'' `e`']"#;
    for (cursor, keys, want) in [
        ((1, 5), "i\\\"", "'1:4-1:6'"),
        ((1, 5), "a\\\"", "'1:3-1:8'"),
        ((1, 5), "2i\\\"", "'1:3-1:7'"),
        ((1, 1), "i\\\"", "'1:4-1:6'"),
        ((1, 13), "i'", "'1:12-1:15'"),
        ((1, 13), "a'", "'1:11-1:17'"),
        ((1, 21), "i`", "'1:19-1:19'"),
        ((1, 21), "a`", "'1:17-1:20'"),
        ((1, 8), "i\\\"", "'0:0-0:0'"),
    ] {
        selects(&fx, text, cursor, keys, want);
    }
    for (cursor, keys, want) in [
        (
            (1, 5),
            "di\\\"",
            "'[''x \"\" y ''''c\\''''d'''' `e`''] 1:4'",
        ),
        ((1, 5), "da\\\"", "'[''x y ''''c\\''''d'''' `e`''] 1:3'"),
        ((1, 13), "ci'z", "'[''x \"a b\" y ''''z'''' `e`''] 1:12'"),
        ((1, 1), "da\\\"", "'[''x y ''''c\\''''d'''' `e`''] 1:3'"),
    ] {
        leaves(&fx, text, cursor, keys, want);
    }
    finish(&fx);
}

/// An object with nothing to select says nothing and leaves the buffer and
/// the cursor alone.
#[test]
fn refusals_say_nothing_and_change_nothing() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    start(&fx);
    for (lines, cursor, keys, said, left) in [
        ("['a b c']", (1, 3), "di(", "none", "'[''a b c''] 1:3'"),
        ("['a (b c']", (1, 6), "di(", "none", "'[''a (b c''] 1:6'"),
        ("['a b) c']", (1, 3), "da)", "none", "'[''a b) c''] 1:3'"),
        ("['a b c']", (1, 3), "dit", "none", "'[''a b c''] 1:3'"),
        (
            "['<a>b</c>']",
            (1, 5),
            "dit",
            "none",
            "'[''<a>b</c>''] 1:5'",
        ),
        ("['a b c']", (1, 3), "di\\\"", "none", "'[''a b c''] 1:3'"),
        (
            "['a \"b c']",
            (1, 3),
            "di\\\"",
            "none",
            "'[''a \"b c''] 1:3'",
        ),
        ("['a b c']", (1, 3), "d3i[", "none", "'[''a b c''] 1:3'"),
        ("['f(a)']", (1, 4), "d3i(", "none", "'[''f(a)''] 1:4'"),
        ("['']", (1, 1), "dis", "none", "'[''''] 1:1'"),
        ("['']", (1, 1), "diw", "none", "'[''''] 1:1'"),
        ("['x']", (1, 1), "d3ip", "none", "'[''x''] 1:1'"),
    ] {
        put(&fx, lines, cursor);
        fx.check_messages(&format!("exe \"normal! {keys}\""), said);
        fx.check(BUFFER, left);
    }
    finish(&fx);
}
