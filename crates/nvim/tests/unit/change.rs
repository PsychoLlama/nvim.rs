//! The text-changing primitives: inserting bytes at the cursor, Replace
//! mode's overwrite, opening a line with its comment leader, and the leader
//! scan itself.
//!
//! The oracle the `change/` retype is checked against. [`ins_char_bytes`] and
//! [`ins_str`] are called directly, the way every editing module calls them;
//! Replace and Virtual Replace (which reach `ins_char_bytes` with the mode
//! flags set) and `open_line` (which every `o`, `O` and Insert-mode `<CR>`
//! reaches, and which writes the split line back through the memline's
//! pointer-form replace) are driven by keys; [`get_leader_len`] is asked about
//! a table of lines. Each answer is today's -- the key-driven rows are also
//! upstream 0.12.6's -- odd corners included: the leader scan's `flags` is
//! written even when nothing matches, except on a blank line.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};
use neovim::change::{get_last_leader_offset, get_leader_len, ins_char_bytes, ins_str};
use neovim::undo::u_save_cursor;

/// The buffer-local options these cases set, back to their defaults: buffer
/// 1 outlives the case.
fn restore(fx: &Probe<'_>) {
    fx.run(
        "setlocal comments& formatoptions& autoindent& smartindent& cindent& expandtab& \
         shiftwidth& tabstop& textwidth&",
    );
    fx.home();
}

fn fresh(fx: &Probe<'_>, lines: &str) {
    fx.home();
    fx.run("setlocal comments& formatoptions& autoindent& noexpandtab shiftwidth=8 tabstop=8");
    fx.run(&format!("call setline(1, {lines})"));
}

/// Insert `bytes` at (`line`, `col`) (one-based column) with
/// `ins_char_bytes`, one character's worth at a time as the callers do.
fn insert_chars(fx: &Probe<'_>, line: i64, col: i64, chars: &[&str]) {
    fx.run(&format!("call cursor({line}, {col})"));
    let _ = u_save_cursor();
    for ch in chars {
        ins_char_bytes(ch.as_bytes());
    }
}

#[test]
fn ins_char_bytes_inserts_and_moves_the_cursor_past() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fresh(&fx, "['abc', '', 'x\u{e9}y']");
    insert_chars(&fx, 1, 2, &["X", "\u{e9}", "日"]);
    fx.expect_all(&[
        ("getline(1)", "'aX\u{e9}日bc'"),
        // The cursor is a byte column, past what went in.
        ("col('.')", "8"),
    ]);
    // Into an empty line, and before a multibyte character.
    insert_chars(&fx, 2, 1, &["z"]);
    insert_chars(&fx, 3, 2, &["-"]);
    fx.expect_all(&[
        ("getline(2, 3)", "['z', 'x-\u{e9}y']"),
        // A combining character is one "character" to the caller.
        ("strchars(getline(1))", "6"),
    ]);
    insert_chars(&fx, 1, 1, &["e\u{301}"]);
    fx.check("getline(1)", "'e\u{301}aX\u{e9}日bc'");
    restore(&fx);
}

#[test]
fn ins_str_inserts_any_bytes_in_one_piece() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fresh(&fx, "['hello world']");
    fx.run("call cursor(1, 6)");
    let _ = u_save_cursor();
    ins_str(b", there");
    fx.expect_all(&[("getline(1)", "'hello, there world'"), ("col('.')", "13")]);
    // At the end of the line, and a zero-length insert changing nothing.
    fx.run("call cursor(1, 19)");
    fx.run("normal! $");
    let _ = u_save_cursor();
    ins_str(b"!");
    ins_str(b"");
    // `$` puts the cursor *on* the last character, so the bang lands before
    // it.
    fx.check("getline(1)", "'hello, there worl!d'");
    restore(&fx);
}

#[test]
fn replace_mode_overwrites_whole_characters() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fresh(&fx, "['abcdef', 'a\u{e9}日b', 'ab']");
    fx.run("call cursor(1, 2) | silent execute \"normal! RXY\\<Esc>\"");
    // A multibyte character -- two bytes or three -- is replaced whole by a
    // single-byte one.
    fx.run("call cursor(2, 2) | silent execute \"normal! Rxy\\<Esc>\"");
    // Past the end of the line Replace mode appends.
    fx.run("call cursor(3, 2) | silent execute \"normal! R1234\\<Esc>\"");
    fx.check("getline(1, 3)", "['aXYdef', 'axyb', 'a1234']");
    // Backspace in Replace mode puts back what was overwritten -- the
    // replace stack `ins_char_bytes` pushed.
    fx.run("call setline(1, 'a\u{e9}日b') | call cursor(1, 1)");
    fx.run("silent execute \"normal! Rxyz\\<BS>\\<BS>\\<Esc>\"");
    fx.check("getline(1)", "'x\u{e9}日b'");
    restore(&fx);
}

#[test]
fn virtual_replace_counts_cells() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fresh(&fx, "[\"\\tabc\", \"ab\\tcd\", '日本語']");
    // Typing over a TAB keeps it until the text reaches its end.
    fx.run("call cursor(1, 1) | silent execute \"normal! gRxy\\<Esc>\"");
    // A TAB typed over `b` and the TAB eats up to the next stop.
    fx.run("call cursor(2, 2) | silent execute \"normal! gR\\<Tab>\\<Esc>\"");
    // One single-width character over a double-width one pads with a space.
    fx.run("call cursor(3, 1) | silent execute \"normal! gRx\\<Esc>\"");
    fx.check("getline(1, 3)", "['xy\tabc', 'a\tcd', 'x 本語']");
    restore(&fx);
}

#[test]
fn open_line_copies_the_comment_leader() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    let cases: &[(&str, &str, &str, &str)] = &[
        // (lines, 'formatoptions', keys, buffer after)
        ("['  // one']", "o", "onext", "['  // one', '  // next']"),
        ("['  // one']", "o", "Oprev", "['  // prev', '  // one']"),
        // Without `o` the leader is not repeated, but the indent is.
        ("['  // one']", "", "onext", "['  // one', '  next']"),
        // A three-piece comment: the start gives way to the middle.
        (
            "['/*', ' * one']",
            "o",
            "Gonext",
            "['/*', ' * one', ' * next']",
        ),
        ("['/* one']", "o", "onext", "['/* one', ' * next']"),
        // After the end part, the indent of the start comes back.
        ("['  /* one */']", "o", "onext", "['  /* one */', '  next']"),
        // Insert-mode <CR> takes the `r` flag, and splits the line.
        (
            "['# one two']",
            "r",
            "6|i\\<CR>\\<Esc>",
            "['# one', '# two']",
        ),
        // An `f` item (only the first line) gives its width as indent.
        ("['- item']", "o", "onext", "['- item', '  next']"),
        // The `b` flag: blank after is required -- `#x` is no leader.
        ("['#x one']", "o", "onext", "['#x one', 'next']"),
        ("['  - item']", "o", "onext", "['  - item', '    next']"),
        // `n`: nested leaders are repeated whole.
        ("['> > quoted']", "o", "onext", "['> > quoted', '> > next']"),
    ];
    let mut moved = Vec::new();
    for (lines, fo, keys, want) in cases {
        fresh(&fx, lines);
        fx.run("setlocal comments=s1:/*,mb:*,ex:*/,://,b:#,n:>,fb:-");
        fx.run(&format!("setlocal formatoptions={fo}"));
        fx.run("call cursor(1, 1)");
        fx.run(&format!("silent execute \"normal! {keys}\""));
        let got = fx.eval("getline(1, '$')");
        if got != *want {
            moved.push(format!("({lines:?}, {fo:?}, {keys:?}, {got:?})"));
        }
    }
    restore(&fx);
    assert!(moved.is_empty(), "rows moved:\n{}", moved.join("\n"));
}

#[test]
fn open_line_splits_and_indents() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    // 'autoindent' copies the indent, and a line left with only it loses it
    // again on <Esc> -- so the `o` after it starts from nothing.
    fresh(&fx, "[\"\\t  if (x) {\"]");
    fx.run("setlocal autoindent");
    fx.run("silent execute \"normal! o\\<Esc>ofoo\\<Esc>\"");
    fx.check("getline(1, '$')", "['\t  if (x) {', '', 'foo']");
    // 'smartindent' adds a level after `{`.
    fresh(&fx, "['if (x) {']");
    fx.run("setlocal autoindent smartindent shiftwidth=4 expandtab");
    fx.run("silent execute \"normal! obody\\<CR>}\\<Esc>\"");
    fx.check("getline(1, '$')", "['if (x) {', '    body', '}']");
    // <CR> in the middle of a line: the white space before the cut stays
    // (only 'autoindent''s own indent is taken back), the tail moves down
    // with the new indent.
    fresh(&fx, "['    one   two']");
    fx.run("setlocal autoindent");
    fx.run("call cursor(1, 11) | silent execute \"normal! i\\<CR>\\<Esc>\"");
    fx.check("getline(1, '$')", "['    one   ', '    two']");
    // 'formatoptions' n: a numbered list item indents the next line to its
    // text.
    fresh(&fx, "['1. first item']");
    fx.run("setlocal autoindent formatoptions=tn textwidth=20");
    fx.run("silent execute \"normal! A and some more words\\<Esc>\"");
    fx.check(
        "getline(1, '$')",
        "['1. first item and', '   some more words']",
    );
    restore(&fx);
}

/// Ask the leader scan about `line`: its length, and the flags it left
/// behind (the bytes up to the item's `:`).
fn leader(line: &str, backward: bool, include_space: bool) -> (i32, String) {
    let (len, flags) = get_leader_len(line.as_bytes(), backward, include_space);
    let flags = flags.map_or_else(
        || "<null>".to_string(),
        |flags| String::from_utf8_lossy(&flags.letters()).into_owned(),
    );
    (len, flags)
}

#[test]
fn get_leader_len_measures_the_leader_and_names_its_item() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fx.home();
    fx.run("setlocal comments=s1:/*,mb:*,ex:*/,://,b:#,n:>,fb:-");
    let cases: &[(&str, bool, bool, (i32, &str))] = &[
        ("// text", false, false, (2, "")),
        ("// text", false, true, (3, "")),
        ("   // text", false, true, (6, "")),
        // Nested: `> > text` is two leaders, measured together.
        ("> > text", false, true, (4, "n")),
        ("/* text", false, true, (3, "s1")),
        (" * text", false, true, (3, "mb")),
        (" */", false, true, (3, "ex")),
        // `b`: blank required after.
        ("# text", false, true, (2, "b")),
        ("#text", false, true, (0, "fb")),
        ("- item", false, true, (2, "fb")),
        // No leader: 0, but the flags name the last item tried.
        ("plain", false, true, (0, "fb")),
        // ... except on a line that is empty or only white space, where no
        // item is tried at all.
        ("", false, true, (0, "<null>")),
        ("    ", false, true, (0, "<null>")),
    ];
    for &(line, backward, space, (want_len, want_flags)) in cases {
        let got = leader(line, backward, space);
        assert_eq!(
            (got.0, got.1.as_str()),
            (want_len, want_flags),
            "get_leader_len({line:?}, {backward}, {space})"
        );
    }
    // The trailing comment of a line of code.
    let offset = |line: &str| get_last_leader_offset(line.as_bytes()).0;
    assert_eq!(offset("code(); // why"), 8);
    // The last leader is the closed comment's end part, not its start.
    assert_eq!(offset("a = b; /* why */"), 14);
    assert_eq!(offset("no comment"), -1);
    // A leader that is the tail of a longer one: the longer one wins.
    fx.run("setlocal comments=b:#,:##,n:>,fb:-");
    let last = |line: &str| {
        let (at, flags) = get_last_leader_offset(line.as_bytes());
        (
            at,
            flags.map(|f| String::from_utf8_lossy(&f.letters()).into_owned()),
        )
    };
    let cases: &[(&str, (i32, Option<&str>))] = &[
        ("x = 1 ## why", (6, Some(""))),
        ("x = 1 # why", (6, Some("b"))),
        ("x > > y", (2, Some("n"))),
        ("a - b", (2, Some("fb"))),
        ("plain", (-1, None)),
    ];
    for &(line, (want_at, want_flags)) in cases {
        let got = last(line);
        assert_eq!((got.0, got.1.as_deref()), (want_at, want_flags), "{line:?}");
    }
    restore(&fx);
}

/// Run `keys` with `normal!` over `lines` under the buffer-local `opts`, and
/// note the buffer if it is not `want`.
fn key_case(fx: &Probe<'_>, moved: &mut Vec<String>, case: (&str, &str, &str, &str)) {
    let (lines, opts, keys, want) = case;
    fresh(fx, lines);
    if !opts.is_empty() {
        fx.run(&format!("setlocal {opts}"));
    }
    fx.run("call cursor(1, 1)");
    fx.run(&format!("silent execute \"normal! {keys}\""));
    let got = fx.eval("getline(1, '$')");
    if got != want {
        moved.push(format!("({lines:?}, {opts:?}, {keys:?}, {got:?})"));
    }
}

#[test]
fn open_line_shapes_the_leader() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    let three = "comments=s1:/*,mb:*,ex:*/ formatoptions=ro";
    let cases: &[(&str, &str, &str, &str)] = &[
        // (lines, options, keys, buffer after)
        // `O` on the end part puts the middle part above, with a space.
        (
            "['/*', ' * one', ' */']",
            three,
            "GOup",
            "['/*', ' * one', ' * up', ' */']",
        ),
        // `O` on the start part inserts no leader.
        ("['/* one']", three, "Oup", "['up', '/* one']"),
        // `o` after a comment that ends on its line: no leader.
        ("['/* one */']", three, "onext", "['/* one */', 'next']"),
        // `x`: typing the end's last character after the middle closes it.
        ("['/* one']", three, "o/", "['/* one', ' */']"),
        // <CR> straight after the start leader gets a space after the
        // middle one.
        ("['/*']", three, "A\\<CR>x", "['/*', ' * x']"),
        // Right-aligned middle under a wider start.
        (
            "['/** one']",
            "comments=sr:/**,mb:*,ex:*/ formatoptions=ro",
            "onext",
            "['/** one', '  * next']",
        ),
        // Left-aligned middle under a wider start.
        (
            "['/** one']",
            "comments=sl:/**,mb:*,ex:*/ formatoptions=ro",
            "onext",
            "['/** one', '*   next']",
        ),
        // A numeric offset in the start item shifts the middle -- and eats
        // the space after it, as upstream does.
        (
            "['/* one']",
            "comments=s3:/*,mb:*,ex:*/ formatoptions=ro",
            "onext",
            "['/* one', '   *next']",
        ),
        // A negative offset with 'autoindent'.
        (
            "['    /* one']",
            "comments=s-2:/*,mb:*,ex:*/ formatoptions=ro autoindent",
            "onext",
            "['    /* one', '  *  next']",
        ),
        // Double-width leaders: blanked out as spaces.
        (
            "['日本 one']",
            "comments=s:日本,m:本,e:語 formatoptions=ro",
            "onext",
            "['日本 one', '本   next']",
        ),
        // Right-aligned over a double-width leader: upstream blanks the
        // first half of `日` and keeps a stray trail byte of it (0.12.6
        // answers the same bytes).
        (
            "['日本 one']",
            "comments=sr:日本,mr:x,er:語 formatoptions=ro",
            "onext",
            "['日本 one', ' \u{fffd}x next']",
        ),
        (
            "['x one']",
            "comments=sr:x,mr:日本,er:語 formatoptions=ro",
            "onext",
            "['x one', '日本 next']",
        ),
        // A TAB after the leader is kept, and the byte in front of it is
        // dropped rather than blanked.
        (
            "[\"/*\\t one\"]",
            "comments=sl:/*,mb:*,ex:*/ formatoptions=ro",
            "onext",
            "['/*\t one', '*\t next']",
        ),
        // A trailing line comment after code, with 'cindent'.
        (
            "['int x; // why']",
            "comments=:// formatoptions=ro cindent",
            "A\\<CR>more",
            "['int x; // why', '       // more']",
        ),
        // 'formatlistpat' under a comment leader: the text lines up.
        (
            "['// 1. first item']",
            "comments=:// formatoptions=tcqn textwidth=20 autoindent",
            "A and some more words",
            "['// 1. first item and', '//    some more', '//    words']",
        ),
        // Virtual Replace: <CR> starts replacing the next line.
        (
            "['abc def', 'xyz uvw']",
            "",
            "4|gR\\<CR>q\\<Esc>",
            "['abc', 'qyz uvw']",
        ),
        (
            "['  // abc', 'xyz uvw']",
            "comments=:// formatoptions=ro autoindent",
            "8|gR\\<CR>q\\<Esc>",
            "['  // ab', '  // qw']",
        ),
    ];
    let mut moved = Vec::new();
    for &case in cases {
        key_case(&fx, &mut moved, case);
    }
    restore(&fx);
    assert!(moved.is_empty(), "rows moved:\n{}", moved.join("\n"));
}

#[test]
fn insert_keys_edit_the_line() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    let sts = "softtabstop=4 tabstop=8";
    let cases: &[(&str, &str, &str, &str)] = &[
        // 'softtabstop' with 'noexpandtab': spaces turn back into TABs.
        ("['']", sts, "i\\<Tab>x\\<Tab>y", "['\tx   y']"),
        ("['']", sts, "i\\<Tab>\\<Tab>x", "['\t\tx']"),
        ("['ab']", sts, "A\\<Tab>\\<Tab>x", "['ab\tx']"),
        ("['']", sts, "a  \\<Tab>x", "['\tx']"),
        // Backspace over a 'softtabstop' stop.
        ("['']", sts, "i\\<Tab>\\<Tab>\\<Tab>\\<BS>x", "['\t\tx']"),
        ("['']", sts, "i\\<Tab>\\<Tab>\\<BS>x", "['\tx']"),
        (
            "['a  ']",
            "softtabstop=4 tabstop=8 backspace=2",
            "A\\<BS>x",
            "['ax']",
        ),
        // 'vartabstop' and 'varsofttabstop'.
        (
            "['']",
            "vartabstop=4,8 tabstop=8",
            "i\\<Tab>x\\<Tab>y",
            "['\t    x\ty']",
        ),
        (
            "['']",
            "varsofttabstop=2,6 tabstop=8",
            "i\\<Tab>\\<Tab>x",
            "['\t\t    x']",
        ),
        // Replace and Virtual Replace with 'softtabstop'.
        (
            "['abcdefghij']",
            sts,
            "R\\<Tab>\\<Tab>\\<Esc>",
            "['\t\t    cdefghij']",
        ),
        (
            "['abcdefghij']",
            sts,
            "gR\\<Tab>\\<Tab>\\<Esc>",
            "['\t\t    ']",
        ),
        (
            "['abcdefghij']",
            sts,
            "R\\<Tab>\\<BS>\\<Esc>",
            "['abcdefghij']",
        ),
        (
            "['ab  cdefgh']",
            sts,
            "3|gR\\<Tab>x\\<Esc>",
            "['ab\t    xh']",
        ),
        // CTRL-V: decimal, hex, Unicode and a key name.
        ("['']", "", "i\\<C-V>065\\<C-V>x41\\<C-V>u00e9", "['AAé']"),
        ("['']", "", "i\\<C-V>\\<Home>", "['<Home>']"),
        ("['']", "", "i\\<C-V>\\<S-Home>", "['<S-Home>']"),
        ("['']", "", "i\\<C-V>\\<C-A>", "['\x01']"),
        // CTRL-A inserts the last insert again; CTRL-@ also leaves.
        ("['']", "", "ifoo\\<Esc>o\\<C-A>bar", "['foo', 'foobar']"),
        ("['']", "", "ifoo\\<Esc>o\\<C-@>x", "['foo', 'fo']"),
        // 'delcombine' off: `x` takes the whole character.
        ("['e\u{301}\u{302}x']", "", "x", "['x']"),
        // `0 CTRL-D` and `^ CTRL-D` in the indent.
        ("[\"\\tone\"]", "", "A\\<CR>0\\<C-D>x", "['\tone', 'x']"),
        (
            "[\"\\tone\"]",
            "autoindent",
            "A\\<CR>^\\<C-D>x\\<CR>y",
            "['\tone', 'x', '\ty']",
        ),
        // Typing past the end of a line in 'virtualedit' insert.
        ("['ab']", "virtualedit=all", "5|ix", "['ab  x']"),
        // `i_CTRL-O` at the end of a line comes back after the last
        // character -- in this harness only for a one-byte one.
        ("['abc']", "", "A\\<C-O>:\\<CR>x", "['abcx']"),
        ("['ab\u{e9}']", "", "A\\<C-O>:\\<CR>x", "['abx\u{e9}']"),
        // 'smartindent' after `{`.
        (
            "['if (x) {']",
            "smartindent shiftwidth=4 expandtab",
            "o  y\\<CR>}",
            "['if (x) {', '      y', '}']",
        ),
    ];
    let mut moved = Vec::new();
    for &case in cases {
        key_case(&fx, &mut moved, case);
    }
    fx.run("setlocal softtabstop& vartabstop& varsofttabstop& virtualedit&");
    // The global ones.
    fx.run("set delcombine");
    key_case(
        &fx,
        &mut moved,
        ("['e\u{301}\u{302}x']", "", "x", "['e\u{301}x']"),
    );
    key_case(
        &fx,
        &mut moved,
        ("['ab\u{301}\u{302}']", "", "$x", "['ab\u{301}']"),
    );
    fx.run("set delcombine& smarttab");
    key_case(
        &fx,
        &mut moved,
        (
            "['']",
            "shiftwidth=4 tabstop=8",
            "i\\<Tab>x\\<Tab>y",
            "['    x\ty']",
        ),
    );
    key_case(
        &fx,
        &mut moved,
        (
            "['  a']",
            "shiftwidth=4 tabstop=8",
            "I\\<Tab>\\<Tab>",
            "['\ta']",
        ),
    );
    fx.run("set smarttab& revins");
    key_case(&fx, &mut moved, ("['']", "", "iabc", "['cba']"));
    fx.run("set revins&");
    restore(&fx);
    assert!(moved.is_empty(), "rows moved:\n{}", moved.join("\n"));
}

#[test]
fn insert_char_pre_replaces_the_character() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    let mut moved = Vec::new();
    fx.run(
        "augroup change_icp | autocmd! | autocmd InsertCharPre * if v:char ==# 'a' | \
         let v:char = 'x\u{e9}y' | endif | augroup END",
    );
    key_case(&fx, &mut moved, ("['']", "", "iab", "['x\u{e9}yb']"));
    // While completion is active the replacement extends the leader.
    key_case(
        &fx,
        &mut moved,
        ("['xéyz', '']", "", "Gix\\<C-N>\\<C-E>a", "['xéyz', 'xxéy']"),
    );
    fx.run("autocmd! change_icp | augroup! change_icp");
    restore(&fx);
    assert!(moved.is_empty(), "rows moved:\n{}", moved.join("\n"));
}

#[test]
fn a_comments_offset_without_digits_reads_as_zero() {
    // `-` with no digits after it is accepted by 'comments' but is no
    // number; upstream's offset parser read nothing there and `o` hung.
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    let mut moved = Vec::new();
    // The same leader as no offset at all.
    let same = "['/* one', '*  next']";
    for com in ["s:/*,mb:*,ex:*/", "s-:/*,mb:*,ex:*/"] {
        let opts = format!("comments={com} formatoptions=ro");
        key_case(&fx, &mut moved, ("['/* one']", &opts, "onext", same));
    }
    restore(&fx);
    assert!(moved.is_empty(), "rows moved:\n{}", moved.join("\n"));
}
