//! Sign definitions end to end, through `:sign define`/`undefine`/`list`
//! and `sign_define()`/`sign_undefine()`/`sign_getdefined()`: every
//! attribute, redefinition as an update, the `text=` rules (one or two
//! cells, multibyte, double-width, combining characters, the backslash
//! escapes, E239), and the order the table reports in. Placements need
//! lines in a buffer, which this fixture has no memline for; they are in
//! the unit suite's `sign.rs`.
//!
//! Written against the behaviour before a definition owns its strings; a
//! row that moves is a behaviour change.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::eval::test_fixture::Fixture;

fn clean(fx: &Fixture) {
    fx.run("call sign_undefine()");
}

#[test]
fn definitions_are_made_updated_and_removed() {
    let fx = Fixture::new();
    clean(&fx);
    fx.run("sign define Sa text=>> texthl=Search linehl=Error culhl=Visual numhl=Number icon=a\\\\b priority=7");
    fx.check("sign_getdefined('Sa')", "[{'culhl': 'Visual', 'name': 'Sa', 'priority': 7, 'texthl': 'Search', 'numhl': 'Number', 'icon': 'a\\b', 'linehl': 'Error', 'text': '>>'}]");
    fx.check_output("sign list Sa", "\nsign Sa icon=a\\b (not supported) text=>> priority=7 linehl=Error texthl=Search culhl=Visual numhl=Number");
    // An update touches only what it names.
    fx.run("sign define Sa text=x numhl=");
    fx.check("sign_getdefined('Sa')", "[{'culhl': 'Visual', 'name': 'Sa', 'texthl': 'Search', 'icon': 'a\\b', 'linehl': 'Error', 'text': 'x '}]");
    // Leading zeroes name the same sign; a bare zero is kept.
    fx.run("sign define 007 text=7");
    fx.run("sign define 0 text=0");
    fx.check("map(sign_getdefined(), 'v:val.name')", "['Sa', '7', '0']");
    fx.check_output("sign list 7", "\nsign 7 text=7 ");
    fx.check_error(
        "sign define Sb nosuch=1",
        "E475: Invalid argument: nosuch=1",
    );
    fx.check_error("sign define", "E156: Missing sign name");
    fx.check_error("sign list Nope", "E155: Unknown sign: Nope");
    fx.check_error("sign undefine Nope", "E155: Unknown sign: Nope");
    fx.check_error("sign frob", "E160: Unknown sign command: frob");
    fx.run("sign undefine 07");
    fx.check("map(sign_getdefined(), 'v:val.name')", "['Sa', '0']");
    fx.check_output("sign list", "\nsign Sa icon=a\\b (not supported) text=x  linehl=Error texthl=Search culhl=Visual\nsign 0 text=0 ");
    clean(&fx);
}

#[test]
fn the_function_forms_take_dictionaries_and_lists() {
    let fx = Fixture::new();
    clean(&fx);
    fx.expect_all(&[
        ("sign_define('Fa', {'text': 'ab', 'texthl': 'Search', 'linehl': 'Error', 'culhl': 'Visual', 'numhl': 'Number', 'icon': 'i\\\\c', 'priority': 3})", "0"),
        ("sign_getdefined('Fa')", "[{'culhl': 'Visual', 'name': 'Fa', 'priority': 3, 'texthl': 'Search', 'numhl': 'Number', 'icon': 'i\\c', 'linehl': 'Error', 'text': 'ab'}]"),
        ("sign_define('Fa', {'linehl': ''})", "0"),
        ("sign_getdefined('Fa')", "[{'culhl': 'Visual', 'name': 'Fa', 'texthl': 'Search', 'numhl': 'Number', 'icon': 'i\\c', 'text': 'ab'}]"),
        ("sign_define('Fb')", "0"),
        ("sign_define([{'name': 'Fc', 'text': 'c'}, {'text': 'no name'}, 3, {'name': ''}, {'name': 'Fd', 'text': 'toolong'}])", "[0, -1, -1, -1, -1]"),
        ("map(sign_getdefined(), 'v:val.name')", "['Fa', 'Fb', 'Fc', 'Fd']"),
        ("sign_getdefined('Fd')", "[{'name': 'Fd', 'text': 'to'}]"),
        ("sign_undefine(['Fc', 'Nope', 'Fb'])", "[0, -1, 0]"),
        ("map(sign_getdefined(), 'v:val.name')", "['Fa', 'Fd']"),
        ("sign_undefine('Fa')", "0"),
        ("sign_undefine('Fa')", "-1"),
        ("sign_getdefined('Nope')", "[]"),
        ("sign_define('Fe', 3)", "-1"),
        ("sign_undefine()", "0"),
        ("sign_getdefined()", "[]"),
    ]);
    fx.check("v:errmsg", "'E1206: Dictionary required for argument 2'");
    clean(&fx);
}

/// The dictionary a sign is defined from is the caller's: unescaping its
/// `text` must not write the result back into it. Upstream does, in place,
/// through the dictionary's own string -- and so did this port, as a write
/// through a `&CStr` (Miri reported it here).
#[test]
fn defining_from_a_dictionary_leaves_it_alone() {
    let fx = Fixture::new();
    clean(&fx);
    fx.run("let g:d = {'text': 'a\\b', 'icon': 'x\\\\y'}");
    fx.run("call sign_define('Dm', g:d)");
    fx.check("g:d", "{'icon': 'x\\\\y', 'text': 'a\\b'}");
    fx.check(
        "sign_getdefined('Dm')",
        "[{'name': 'Dm', 'icon': 'x\\y', 'text': 'ab'}]",
    );
    fx.run("let g:l = [{'name': 'Dl', 'text': '\\ x'}]");
    fx.run("call sign_define(g:l)");
    fx.check("g:l", "[{'name': 'Dl', 'text': '\\ x'}]");
    fx.check("sign_getdefined('Dl')", "[{'name': 'Dl', 'text': ' x'}]");
    fx.run("unlet g:d g:l");
    clean(&fx);
}

/// `text=`: what each value defines, or the error it raises. The sign is
/// redefined from a known text first, so a failure that leaves cells
/// behind shows.
#[test]
fn sign_text_cells_and_escapes() {
    let fx = Fixture::new();
    clean(&fx);
    let mut rows = Vec::new();
    for text in [
        "",
        "a",
        "ab",
        "abc",
        "\\ ",
        "\\ a",
        "a\\ ",
        "\\\\",
        "a\\",
        "\\",
        "\\\\\\\\",
        "\\a\\b",
        "é",
        "éé",
        "ééé",
        "中",
        "中a",
        "a中",
        "中中",
        "e\u{301}",
        "e\u{301}x",
        "\u{301}",
        "\u{301}\u{302}",
        "x\u{200b}",
        "\u{1f600}",
        "\u{1f600}a",
        "\u{2764}\u{fe0f}",
        "\t",
        "a\u{7f}",
        "\u{a0}",
        "\u{feff}",
        "\\\u{4e2d}",
    ] {
        fx.run("silent! sign undefine Tx");
        fx.run("sign define Tx text=zz");
        fx.run("let v:errmsg = ''");
        let quoted = text.replace('\'', "''");
        fx.run(&format!(
            "silent! call sign_define('Tx', {{'text': '{quoted}'}})"
        ));
        rows.push(format!(
            "{} {}",
            fx.eval("get(get(sign_getdefined('Tx'), 0, {}), 'text', '-')"),
            fx.eval("v:errmsg")
        ));
        // A fresh sign the text fails on is still defined.
        fx.run("silent! sign undefine Tx");
        fx.run(&format!(
            "silent! call sign_define('Tx', {{'text': '{quoted}'}})"
        ));
        rows.push(fx.eval("sign_getdefined('Tx')"));
        fx.run("silent! sign undefine Tx");
        fx.run("sign define Tx text=zz");
        fx.run("let v:errmsg = ''");
        fx.run(&format!("silent! execute 'sign define Tx text={quoted}'"));
        rows.push(format!(
            "{} {}",
            fx.eval("get(get(sign_getdefined('Tx'), 0, {}), 'text', '-')"),
            fx.eval("v:errmsg")
        ));
    }
    assert_eq!(rows, TEXT_ROWS, "sign text rows moved");
    clean(&fx);
}

/// Per text: the function form's text and error, the fresh sign it left,
/// then the command form's text and error.
const TEXT_ROWS: &[&str] = &[
    "'-' ''",
    "[{'name': 'Tx'}]",
    "'-' ''",
    "'a ' ''",
    "[{'name': 'Tx', 'text': 'a '}]",
    "'a ' ''",
    "'ab' ''",
    "[{'name': 'Tx', 'text': 'ab'}]",
    "'ab' ''",
    "'ab' 'E239: Invalid sign text: abc'",
    "[{'name': 'Tx', 'text': 'ab'}]",
    "'ab' 'E239: Invalid sign text: abc'",
    "'  ' ''",
    "[{'name': 'Tx', 'text': '  '}]",
    "'  ' ''",
    "' a' ''",
    "[{'name': 'Tx', 'text': ' a'}]",
    "' a' ''",
    "'a ' ''",
    "[{'name': 'Tx', 'text': 'a '}]",
    "'a ' ''",
    "'\\ ' ''",
    "[{'name': 'Tx', 'text': '\\ '}]",
    "'\\ ' ''",
    "'a\\' ''",
    "[{'name': 'Tx', 'text': 'a\\'}]",
    "'a\\' ''",
    "'\\ ' ''",
    "[{'name': 'Tx', 'text': '\\ '}]",
    "'\\ ' ''",
    "'\\\\' ''",
    "[{'name': 'Tx', 'text': '\\\\'}]",
    "'\\\\' ''",
    "'ab' ''",
    "[{'name': 'Tx', 'text': 'ab'}]",
    "'ab' ''",
    "'é ' ''",
    "[{'name': 'Tx', 'text': 'é '}]",
    "'é ' ''",
    "'éé' ''",
    "[{'name': 'Tx', 'text': 'éé'}]",
    "'éé' ''",
    "'éé' 'E239: Invalid sign text: ééé'",
    "[{'name': 'Tx', 'text': 'éé'}]",
    "'éé' 'E239: Invalid sign text: ééé'",
    "'中' ''",
    "[{'name': 'Tx', 'text': '中'}]",
    "'中' ''",
    "'中' 'E239: Invalid sign text: 中a'",
    "[{'name': 'Tx', 'text': '中'}]",
    "'中' 'E239: Invalid sign text: 中a'",
    "'a中' 'E239: Invalid sign text: a中'",
    "[{'name': 'Tx', 'text': 'a中'}]",
    "'a中' 'E239: Invalid sign text: a中'",
    "'中' 'E239: Invalid sign text: 中中'",
    "[{'name': 'Tx', 'text': '中'}]",
    "'中' 'E239: Invalid sign text: 中中'",
    "'e\u{301} ' ''",
    "[{'name': 'Tx', 'text': 'e\u{301} '}]",
    "'e\u{301} ' ''",
    "'e\u{301}x' ''",
    "[{'name': 'Tx', 'text': 'e\u{301}x'}]",
    "'e\u{301}x' ''",
    "' \u{301} ' ''",
    "[{'name': 'Tx', 'text': ' \u{301} '}]",
    "' \u{301} ' ''",
    "' \u{301}\u{302} ' ''",
    "[{'name': 'Tx', 'text': ' \u{301}\u{302} '}]",
    "' \u{301}\u{302} ' ''",
    "'x\u{200b}' 'E239: Invalid sign text: x\u{200b}'",
    "[{'name': 'Tx', 'text': 'x\u{200b}'}]",
    "'x\u{200b}' 'E239: Invalid sign text: x\u{200b}'",
    "'😀' ''",
    "[{'name': 'Tx', 'text': '😀'}]",
    "'😀' ''",
    "'😀' 'E239: Invalid sign text: 😀a'",
    "[{'name': 'Tx', 'text': '😀'}]",
    "'😀' 'E239: Invalid sign text: 😀a'",
    "'❤\u{fe0f} ' ''",
    "[{'name': 'Tx', 'text': '❤\u{fe0f} '}]",
    "'❤\u{fe0f} ' ''",
    "'\tz' 'E239: Invalid sign text: \t'",
    "[{'name': 'Tx', 'text': '\t'}]",
    "'-' ''",
    "'a\u{7f}' 'E239: Invalid sign text: a\u{7f}'",
    "[{'name': 'Tx', 'text': 'a\u{7f}'}]",
    "'a\u{7f}' 'E239: Invalid sign text: a\u{7f}'",
    "'\u{a0} ' ''",
    "[{'name': 'Tx', 'text': '\u{a0} '}]",
    "'\u{a0} ' ''",
    "'\u{feff}z' 'E239: Invalid sign text: \u{feff}'",
    "[{'name': 'Tx', 'text': '\u{feff}'}]",
    "'\u{feff}z' 'E239: Invalid sign text: \u{feff}'",
    "'中' ''",
    "[{'name': 'Tx', 'text': '中'}]",
    "'中' ''",
];
