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

use std::ffi::{CStr, c_char, c_int};

use crate::support::{Probe, Sandbox};
use neovim::memory::xfree;
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

/// `get_spec_reg` directly: the answer, whether the caller owns it, and the
/// registers it does not answer.
#[test]
fn get_spec_reg_answers_the_computed_registers_only() {
    let sandbox = Sandbox::dir("register_spec");
    let fx = Probe::new(sandbox.editor());
    fresh(&fx);
    fx.run("silent file spec.txt");
    fx.run("let @/ = 'needle'");

    let ask = |name: char, errmsg: bool| -> (bool, Option<Vec<u8>>, bool) {
        let mut value: *mut c_char = std::ptr::null_mut();
        let mut allocated = false;
        // SAFETY: both out-parameters are locals.
        let found =
            unsafe { get_spec_reg(name as c_int, &raw mut value, &raw mut allocated, errmsg) };
        let text = (!value.is_null()).then(|| {
            // SAFETY: a found register's value is NUL-terminated.
            unsafe { CStr::from_ptr(value) }.to_bytes().to_vec()
        });
        if allocated {
            // SAFETY: `allocated` hands the block to the caller.
            unsafe { xfree(value.cast()) };
        }
        (found, text, allocated)
    };

    assert_eq!(ask('%', false), (true, Some(b"spec.txt".to_vec()), false));
    // `%` lends the buffer's name; `/` is a copy of the pattern, the
    // caller's to free.
    assert_eq!(ask('/', false), (true, Some(b"needle".to_vec()), true));
    // Not computed: the caller reads the store instead.
    assert!(!ask('a', false).0);
    assert!(!ask('"', false).0);
    // CTRL-W, the word under the cursor, needs `errmsg` to be asked at all.
    fx.run("call setline(1, 'word here')");
    fx.run("call cursor(1, 1)");
    assert_eq!(ask('\u{17}', false), (false, None, false));
    assert_eq!(ask('\u{17}', true), (true, Some(b"word".to_vec()), true));
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
