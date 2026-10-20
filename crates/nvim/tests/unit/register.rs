//! The register store: what goes in, what comes out, and how it is listed.
//!
//! The oracle the register retype is checked against. Text reaches a register
//! through `str_to_reg` (from `setreg()`, `:let @r =`, the clipboard and the
//! API's paste) and comes back through `getreg()`/`getregtype()`, `:put` and
//! `:registers`; the computed registers (`%`, `#`, `:`, `/`, `.`, `=`) never
//! hold a `YankReg` at all and are answered by `get_spec_reg`. Each case pins
//! today's answer, odd corners included: the NUL-for-newline translation
//! inside a line, an append that joins a characterwise tail to its last line,
//! a blockwise width that counts cells and not bytes, and the `:registers`
//! listing's type column and truncation.
//!
//! Everything reads the editor's state back through Vimscript, so the cases
//! survive the store changing shape underneath them; only the
//! `get_spec_reg` cases name a function.

#![cfg(not(miri))]

use std::ffi::c_int;

use crate::support::{Probe, Sandbox};
use neovim::register::get_spec_reg;

/// Every register these cases write, cleared on the way in and out.
const CLEAR: &str = "for r in split('abcdefxyz', '\\zs') | call setreg(r, []) | endfor";

fn fresh(fx: &Probe<'_>) {
    fx.home();
    fx.run(CLEAR);
}

#[test]
fn setreg_decides_the_type_from_the_text() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fresh(&fx);
    fx.run("call setreg('a', 'abc')");
    fx.run("call setreg('b', \"abc\\n\")");
    fx.run("call setreg('c', \"abc\\r\")");
    fx.run("call setreg('d', ['one', 'two'])");
    fx.run("call setreg('e', \"one\\ntwo\")");
    fx.run("call setreg('f', '')");
    fx.expect_all(&[
        ("[getreg('a'), getregtype('a')]", "['abc', 'v']"),
        // A trailing NL makes it linewise, and the NL is the line's end, not
        // part of it.
        ("[getreg('b', 1, 1), getregtype('b')]", "[['abc'], 'V']"),
        // So does a trailing CR -- but the CR stays in the line.
        ("[getreg('c', 1, 1), getregtype('c')]", "[['abc\r'], 'V']"),
        // A list is always linewise.
        ("[getreg('d'), getregtype('d')]", "['one\ntwo\n', 'V']"),
        // An inner NL splits a characterwise string into lines.
        (
            "[getreg('e', 1, 1), getregtype('e')]",
            "[['one', 'two'], 'v']",
        ),
        // An empty string is one empty characterwise line, not nothing.
        ("[getreg('f', 1, 1), getregtype('f')]", "[[''], 'v']"),
    ]);
    fresh(&fx);
}

#[test]
fn a_newline_inside_a_list_item_is_a_nul_in_the_line() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fresh(&fx);
    // A list item cannot hold a line break, so its NL is the NUL the line
    // holds in memory; it comes back out as NL.
    fx.run("call setreg('a', [\"x\\ny\", 'z'])");
    fx.expect_all(&[
        ("getreg('a', 1, 1)", "['x\ny', 'z']"),
        ("getreg('a')", "'x\ny\nz\n'"),
        ("getregtype('a')", "'V'"),
    ]);
    // Put into the buffer it is one line holding the NUL.
    fx.run("silent put a");
    fx.expect_all(&[
        ("line('$')", "3"),
        ("getline(2)", "'x\ny'"),
        ("strlen(getline(2))", "3"),
        ("char2nr(getline(2)[1])", "10"),
    ]);
    fresh(&fx);
}

#[test]
fn an_upper_case_name_appends() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fresh(&fx);
    // Characterwise onto characterwise joins the new text to the last line.
    fx.run("call setreg('a', 'abc') | call setreg('A', 'def')");
    // Characterwise onto linewise adds a line -- and the register takes the
    // type of the text appended, so it is characterwise now.
    fx.run("call setreg('b', \"one\\n\") | call setreg('B', 'two')");
    // Linewise onto characterwise: the first new line joins the old last
    // one, and the register becomes linewise.
    fx.run("call setreg('c', 'one') | call setreg('C', \"two\\n\")");
    // `let @A` is the same append.
    fx.run("let @d = 'x' | let @D = \"y\\nz\"");
    // A list appended to a characterwise register.
    fx.run("call setreg('e', 'one') | call setreg('E', ['two', 'three'])");
    // setreg()'s 'a' flag with a lower-case name.
    fx.run("call setreg('f', 'one') | call setreg('f', 'two', 'a')");
    fx.expect_all(&[
        ("[getreg('a', 1, 1), getregtype('a')]", "[['abcdef'], 'v']"),
        (
            "[getreg('b', 1, 1), getregtype('b')]",
            "[['one', 'two'], 'v']",
        ),
        ("[getreg('c', 1, 1), getregtype('c')]", "[['onetwo'], 'V']"),
        ("[getreg('d', 1, 1), getregtype('d')]", "[['xy', 'z'], 'v']"),
        (
            "[getreg('e', 1, 1), getregtype('e')]",
            "[['one', 'two', 'three'], 'V']",
        ),
        ("[getreg('f', 1, 1), getregtype('f')]", "[['onetwo'], 'v']"),
    ]);
    // Appending to a register that was never set is a plain set.
    fx.run("call setreg('x', []) | call setreg('X', 'new')");
    fx.check("[getreg('x', 1, 1), getregtype('x')]", "[['new'], 'v']");
    fresh(&fx);
}

#[test]
fn a_blockwise_register_is_as_wide_as_its_widest_line_in_cells() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fresh(&fx);
    fx.run("call setreg('a', ['ab', 'cdef', 'g'], 'b')");
    // An explicit width wins, even a narrower one.
    fx.run("call setreg('b', ['ab', 'cdef'], 'b10')");
    fx.run("call setreg('c', ['ab', 'cdef'], 'b1')");
    // Cells, not bytes and not characters: two double-width characters are
    // four cells in six bytes; a combining mark adds nothing.
    fx.run("call setreg('d', ['日本', 'e\u{301}'], 'b')");
    // A TAB counts as one cell here -- it is measured with no column.
    fx.run("call setreg('e', [\"\\tx\", 'ab'], 'b')");
    // A string with NLs, blockwise.
    fx.run("call setreg('f', \"abc\\nd\", 'b')");
    fx.expect_all(&[
        ("getregtype('a')", "'\u{16}4'"),
        ("getregtype('b')", "'\u{16}10'"),
        ("getregtype('c')", "'\u{16}1'"),
        ("getregtype('d')", "'\u{16}4'"),
        ("getregtype('e')", "'\u{16}2'"),
        (
            "[getreg('f', 1, 1), getregtype('f')]",
            "[['abc', 'd'], '\u{16}3']",
        ),
    ]);
    // Put blockwise into a buffer, the short lines are padded to the width
    // only when text follows them.
    fx.run("call setline(1, ['1234', '5678', '9'])");
    fx.run("call cursor(1, 2)");
    fx.run("silent normal! \"ap");
    fx.check("getline(1, '$')", "['12ab  34', '56cdef78', '9 g']");
    fresh(&fx);
}

#[test]
fn reading_back_as_a_list_and_as_a_string() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fresh(&fx);
    fx.run("call setreg('a', ['one', '', 'three'], 'b')");
    fx.run("call setreg('b', \"one\\n\\n\")");
    fx.expect_all(&[
        // Blockwise: lines joined with NL, no trailing one.
        ("getreg('a')", "'one\n\nthree'"),
        ("getreg('a', 1, 1)", "['one', '', 'three']"),
        // Linewise: every line ends in NL.
        ("getreg('b')", "'one\n\n'"),
        ("getreg('b', 1, 1)", "['one', '']"),
        ("getreginfo('b').regcontents", "['one', '']"),
        ("getreginfo('b').regtype", "'V'"),
        // An unset register.
        ("getreg('z', 1, 1)", "[]"),
        ("getregtype('z')", "''"),
    ]);
    fresh(&fx);
}

#[test]
fn the_computed_registers() {
    let sandbox = Sandbox::dir("register_computed");
    let fx = Probe::new(sandbox.editor());
    fresh(&fx);
    fx.run("silent file one.txt");
    fx.run("silent file two.txt");
    fx.run("call setline(1, 'hello')");
    fx.run("let @/ = 'pat'");
    fx.run("silent execute 'normal! Ainserted'");
    fx.expect_all(&[
        ("getreg('%')", "'two.txt'"),
        // `:file` leaves the old name as the alternate.
        ("getreg('#')", "'one.txt'"),
        ("getreg('/')", "'pat'"),
        ("getreg('.')", "'inserted'"),
        ("getregtype('%')", "'v'"),
        ("getregtype('.')", "'v'"),
    ]);
    // The expression register: read plainly it evaluates, with `1` it is
    // the expression's text.
    fx.run("let @= = '6 * 7'");
    fx.expect_all(&[("getreg('=')", "'42'"), ("getreg('=', 1)", "'6 * 7'")]);
    fx.run("silent 0file");
    fresh(&fx);
}

/// `get_spec_reg` directly: the answer, and the registers it does not answer.
#[test]
fn get_spec_reg_answers_the_computed_registers_only() {
    let sandbox = Sandbox::dir("register_spec");
    let fx = Probe::new(sandbox.editor());
    fresh(&fx);
    fx.run("silent file spec.txt");
    fx.run("let @/ = 'needle'");

    // `None`: not computed; `Some(None)`: computed, and nothing in it.
    let ask = |name: char, errmsg: bool| -> Option<Option<Vec<u8>>> {
        get_spec_reg(name as c_int, errmsg).map(|value| value.map(|text| text.to_vec()))
    };

    assert_eq!(ask('%', false), Some(Some(b"spec.txt".to_vec())));
    assert_eq!(ask('/', false), Some(Some(b"needle".to_vec())));
    // The black hole reads as empty, not as nothing.
    assert_eq!(ask('_', false), Some(Some(Vec::new())));
    // Not computed: the caller reads the store instead.
    assert_eq!(ask('a', false), None);
    assert_eq!(ask('"', false), None);
    // CTRL-W, the word under the cursor, needs `errmsg` to be asked at all.
    fx.run("call setline(1, 'word here')");
    fx.run("call cursor(1, 1)");
    assert_eq!(ask('\u{17}', false), None);
    assert_eq!(ask('\u{17}', true), Some(Some(b"word".to_vec())));
    // No word under the cursor.
    fx.run("call setline(1, '   ')");
    assert_eq!(ask('\u{17}', true), Some(None));
    fx.run("silent 0file");
    fresh(&fx);
}

#[test]
fn the_registers_listing() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fresh(&fx);
    fx.run("call setreg('a', 'chars')");
    fx.run("call setreg('b', ['line one', 'line two'])");
    fx.run("call setreg('c', ['ab', 'cd'], 'b')");
    // A control character is shown as `^X`, and a long register is cut at
    // the screen's width.
    fx.run("call setreg('d', \"tab\\there\\x01\")");
    fx.run("call setreg('e', repeat('x', 200))");
    fx.check_output(
        "registers abcd",
        "\nType Name Content\n  c  \"a   chars\n  l  \"b   line one^Jline two^J\n  b  \"c   ab^Jcd\n  c  \"d   tab^Ihere^A",
    );
    let wide = fx.output("registers e");
    let last = wide.lines().last().unwrap_or_default().to_string();
    // Cut to the 80-column screen less one.
    fx.check(&format!("strdisplaywidth('{last}')"), "79");
    // An empty register is not listed.
    fx.check_output("registers z", "\nType Name Content");
    fresh(&fx);
}

#[test]
fn a_yank_into_an_upper_case_name_appends() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fresh(&fx);
    fx.run("call setline(1, ['one two', 'three four'])");
    // Linewise onto linewise adds the lines.
    fx.run("silent normal! gg\"ayy");
    fx.run("silent normal! j\"Ayy");
    // Characterwise onto characterwise joins the first new piece to the last
    // old line...
    fx.run("silent normal! gg0\"byiw");
    fx.run("silent normal! j0\"Byiw");
    // ...unless 'cpoptions' has `>`, which starts a new line instead.
    fx.run("set cpoptions+=>");
    fx.run("silent normal! gg0\"cyiw");
    fx.run("silent normal! j0\"Cyiw");
    fx.run("set cpoptions-=>");
    // Linewise onto characterwise makes the whole register linewise.
    fx.run("silent normal! gg0\"dyiw");
    fx.run("silent normal! j\"Dyy");
    // A blockwise yank appended to a characterwise one.
    fx.run("silent normal! gg0\"eyiw");
    fx.run("silent execute \"normal! gg0\\<C-v>j2l\\\"Ey\"");
    fx.expect_all(&[
        (
            "[getreg('a', 1, 1), getregtype('a')]",
            "[['one two', 'three four'], 'V']",
        ),
        (
            "[getreg('b', 1, 1), getregtype('b')]",
            "[['onethree'], 'v']",
        ),
        (
            "[getreg('c', 1, 1), getregtype('c')]",
            "[['one', 'three'], 'v']",
        ),
        (
            "[getreg('d', 1, 1), getregtype('d')]",
            "[['one', 'three four'], 'V']",
        ),
        (
            "[getreg('e', 1, 1), getregtype('e')]",
            "[['oneone', 'thr'], 'v']",
        ),
    ]);
    fresh(&fx);
}

#[test]
fn a_delete_shifts_the_numbered_registers() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fresh(&fx);
    fx.run("for r in range(1, 9) | call setreg(r, 'old' . r) | endfor");
    fx.run("call setline(1, ['l1', 'l2', 'l3', 'w1 w2'])");
    fx.run("silent normal! ggdd");
    fx.run("silent normal! dd");
    // A delete inside one line goes to `"-`, not `"1`.
    fx.run("silent normal! j0dw");
    fx.expect_all(&[
        ("getreg('1', 1, 1)", "['l2']"),
        ("getreg('2', 1, 1)", "['l1']"),
        ("getreg('3')", "'old1'"),
        ("getreg('9')", "'old7'"),
        ("getreg('-')", "'w1 '"),
        // `""` is the last register written.
        ("getreg('\"')", "'w1 '"),
    ]);
    // A named delete writes the name and `"1` both.
    fx.run("call setline(1, ['x', 'y'])");
    fx.run("silent normal! gg\"zdd");
    fx.expect_all(&[
        ("getreg('z', 1, 1)", "['x']"),
        ("getreg('1', 1, 1)", "['x']"),
        ("getreg('2', 1, 1)", "['l2']"),
    ]);
    fx.run("for r in range(1, 9) | call setreg(r, []) | endfor");
    fx.run("call setreg('-', [])");
    fresh(&fx);
}

#[test]
fn text_yank_post_sees_the_register() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fresh(&fx);
    fx.run("call setline(1, ['abc', 'defg'])");
    fx.run("let g:ev = {}");
    fx.run("autocmd TextYankPost * let g:ev = copy(v:event)");
    fx.run("silent execute \"normal! gg0\\<C-v>jl\\\"ay\"");
    fx.expect_all(&[
        ("g:ev.regcontents", "['ab', 'de']"),
        ("g:ev.regtype", "'\u{16}2'"),
        ("g:ev.regname", "'a'"),
        ("g:ev.operator", "'y'"),
        ("g:ev.visual", "v:true"),
    ]);
    fx.run("silent normal! gg\"bdd");
    fx.expect_all(&[
        ("g:ev.regcontents", "['abc']"),
        ("g:ev.regname", "'b'"),
        ("g:ev.operator", "'d'"),
    ]);
    fx.run("autocmd! TextYankPost");
    fx.run("unlet g:ev");
    fresh(&fx);
}

#[test]
fn executing_a_register() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fresh(&fx);
    // `:@a` runs each line as a command, and joins `\` continuation lines,
    // dropping `"\ ` comments inside them.
    fx.run("let g:x = 0");
    fx.run("call setreg('a', ['let g:x = 1', '\\ + 2', '\"\\ a comment', '\\ + 4'])");
    fx.run("silent execute \"normal! :@a\\<CR>\"");
    fx.check("g:x", "7");
    // `@b` replays keys.
    fx.run("call setline(1, 'abc')");
    fx.run("call setreg('b', 'x')");
    fx.run("silent normal! gg03@b");
    fx.check("getline(1)", "''");
    // `@@` repeats the last one.
    fx.run("call setline(1, 'abcd')");
    fx.run("silent normal! gg0@b@@");
    fx.check("getline(1)", "'cd'");
    // An unset register fails and replays nothing.
    fx.run("call setline(1, 'abcd')");
    fx.run("silent! normal! gg0@z");
    fx.check("getline(1)", "'abcd'");
    fresh(&fx);
}

#[test]
fn inserting_a_register() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fresh(&fx);
    fx.run("call setline(1, 'ab')");
    // CTRL-R a: the text, as if typed -- through the read buffer, which
    // `:normal` only drains once its own keys are done, so the text lands
    // where the `<Esc>` left the cursor (the real editor does the same).
    fx.run("call setreg('a', ['one', 'two'])");
    fx.run("silent execute \"normal! gg0li\\<C-R>a\\<Esc>\"");
    fx.check("getline(1, '$')", "['one', 'two', 'ab']");
    fx.run("silent %delete _");
    // CTRL-R - of a characterwise small delete goes in through a put.
    fx.run("call setline(1, ['xy'])");
    fx.run("call setreg('-', 'MID')");
    fx.run("silent execute \"normal! gg0li\\<C-R>-\\<Esc>\"");
    fx.check("getline(1, '$')", "['xMIDy']");
    fx.run("silent %delete _");
    // CTRL-R CTRL-O puts linewise text above the line.
    fx.run("call setline(1, ['xy'])");
    fx.run("call setreg('b', ['L1', 'L2'])");
    fx.run("silent execute \"normal! gg0li\\<C-R>\\<C-O>b\\<Esc>\"");
    fx.check("getline(1, '$')", "['L1', 'L2', 'xy']");
    // On the command line, CTRL-R pastes the lines joined by a CR that is a
    // character of the command line, not its end: the two `:let`s are one
    // command, which fails.
    fx.run("call setreg('c', ['let g:y = 5', 'let g:y += 1'])");
    fx.run("let g:y = 0");
    fx.run("silent! execute \"normal! :\\<C-R>c\\<CR>\"");
    fx.check("g:y", "0");
    fx.run("call setreg('-', [])");
    fresh(&fx);
}

#[test]
fn putting_with_a_count_and_over_a_selection() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fresh(&fx);
    fx.run("call setline(1, ['1234', '5678'])");
    fx.run("call setreg('a', ['ab', 'c'], 'b')");
    fx.run("silent normal! gg02\"ap");
    fx.check("getline(1, '$')", "['1abab234', '5c c 678']");
    fx.run("silent %delete _");
    // A Visual put replaces the selection, and the deleted text goes to `""`
    // -- but the register being put is the one read before the delete.
    fx.run("call setline(1, ['one two'])");
    fx.run("call setreg('\"', 'NEW')");
    fx.run("silent normal! gg0viwp");
    fx.expect_all(&[("getline(1)", "'NEW two'"), ("getreg('\"')", "'one'")]);
    // `P` over a selection keeps the register.
    fx.run("call setline(1, ['one two'])");
    fx.run("call setreg('\"', 'NEW')");
    fx.run("silent normal! gg0viwP");
    fx.expect_all(&[("getline(1)", "'NEW two'"), ("getreg('\"')", "'NEW'")]);
    // A charwise put of several lines splits the cursor line.
    fx.run("call setline(1, ['abcd'])");
    fx.run("call setreg('b', \"X\\nY\")");
    fx.run("silent normal! gg0l\"bp");
    fx.check("getline(1, '$')", "['abX', 'Ycd']");
    fx.run("silent %delete _");
    // `]p` reindents to the cursor line.
    fx.run("call setline(1, ['    top'])");
    fx.run("call setreg('c', ['x', '  y'], 'l')");
    fx.run("silent normal! gg\"c]p");
    fx.check("getline(1, '$')", "['    top', '    x', '      y']");
    // Nothing to put.
    fx.check_error("normal! \"zp", "E353: Nothing in register z");
    fresh(&fx);
}
