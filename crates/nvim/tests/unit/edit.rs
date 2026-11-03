//! Insert mode: what a typed key turns into in the buffer.
//!
//! `edit` had no in-crate test. The functional and oldtest suites reach it
//! through every key they send; nothing in the crate pinned the decisions
//! the key handlers make about a *character* -- which one goes in, how many
//! bytes it takes, which key ends an abbreviation and what `v:char` it
//! sees, what CTRL-V and CTRL-K build from the keys after them, how Replace
//! mode puts the overwritten text back, and what the redo buffer records
//! for `.`. Those are the decisions that read the key's integer, and the
//! integer is what the `Key` retype changes.
//!
//! One engine: [`Case::typed`] feeds keys with `feedkeys(.., 'xt')` (as if
//! typed, so abbreviations and 'digraph' apply) into a buffer set up by the
//! case, and the answers are read back as expressions. Every answer is the
//! tree's before the retype; the `ins_char`/abbreviation rows are also
//! upstream's.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};

/// Run `keys` (a Vim-script double-quoted string body, `\<Esc>` notation
/// allowed) as typed, starting in Normal mode on `lines` with the cursor at
/// (1, 1), then read each expression back.
#[track_caller]
fn typed(fx: &Probe<'_>, lines: &str, keys: &str, want: &[(&str, &str)]) {
    fx.home();
    fx.run(&format!("call setline(1, {lines})"));
    fx.run("call cursor(1, 1)");
    fx.run(&format!("call feedkeys(\"{keys}\", 'xt')"));
    fx.expect_all(want);
}

/// The options these cases set, back to their defaults.
fn restore(fx: &Probe<'_>) {
    fx.run(
        "set backspace& textwidth& formatoptions& expandtab& shiftwidth& tabstop& softtabstop& \
         digraph& revins& allowrevins& rightleft& autoindent& smarttab& virtualedit& paste&",
    );
    fx.run("silent! iabclear");
    fx.run("silent! cabclear");
    fx.run("silent! iunmap <buffer> x");
    fx.run("silent! autocmd! EditOracle");
    fx.home();
}

#[test]
fn a_typed_character_goes_in_as_its_utf8_bytes() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    typed(
        &fx,
        "['']",
        r"iab\u00e9\u65e5\U0001F600z\<Esc>",
        &[
            ("getline(1)", "'abé日😀z'"),
            ("strlen(getline(1))", "12"),
            ("col('.')", "12"),
        ],
    );
    // A combining character joins the one before it.
    typed(
        &fx,
        "['']",
        r"ie\u0301x\<Esc>",
        &[("getline(1)", "'e\u{301}x'"), ("strchars(getline(1))", "3")],
    );
    // Insert, append, append-at-end and insert-at-column-one all start from
    // the command's own place, and a count repeats the text.
    typed(&fx, "['abc']", r"lix\<Esc>", &[("getline(1)", "'axbc'")]);
    typed(&fx, "['abc']", r"lax\<Esc>", &[("getline(1)", "'abxc'")]);
    typed(&fx, "['abc']", r"Ax\<Esc>", &[("getline(1)", "'abcx'")]);
    typed(
        &fx,
        "['  abc']",
        r"$Ix\<Esc>",
        &[("getline(1)", "'  xabc'")],
    );
    typed(
        &fx,
        "['  abc']",
        r"$gIx\<Esc>",
        &[("getline(1)", "'x  abc'")],
    );
    typed(
        &fx,
        "['abc']",
        r"3ixy\<Esc>",
        &[("getline(1)", "'xyxyxyabc'")],
    );
    typed(
        &fx,
        "['abc']",
        r"2o\u00e9\<Esc>",
        &[("getline(1, '$')", "['abc', 'é', 'é']")],
    );
    // `.` repeats the insert, multibyte and all.
    typed(
        &fx,
        "['abc']",
        r"a\u00e9\u65e5\<Esc>j.",
        &[("getline(1)", "'aé日é日bc'")],
    );
    restore(&fx);
}

#[test]
fn control_v_builds_a_character_from_the_keys_after_it() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    for (keys, want) in [
        // Decimal, octal, hex, and the two Unicode forms.
        (r"i\<C-V>065\<Esc>", "'A'"),
        // An `x` (or `o`, `u`) anywhere in the number switches the base and
        // is not inserted, upstream as here.
        (r"i\<C-V>65x\<Esc>", "'A'"),
        (r"i\<C-V>o101\<Esc>", "'A'"),
        (r"i\<C-V>O101\<Esc>", "'A'"),
        (r"i\<C-V>x41\<Esc>", "'A'"),
        (r"i\<C-V>X4a\<Esc>", "'J'"),
        (r"i\<C-V>u20ac\<Esc>", "'€'"),
        (r"i\<C-V>U1F600\<Esc>", "'😀'"),
        (r"i\<C-V>u00e9z\<Esc>", "'éz'"),
        // A character above 255 typed in decimal is 255.
        (r"i\<C-V>300\<Esc>", "'ÿ'"),
        // The control characters go in as themselves.
        (r"i\<C-V>\<Esc>\<Esc>", "'\u{1b}'"),
        (r"i\<C-V>\<C-M>\<Esc>", "'\r'"),
        (r"i\<C-V>\<Tab>\<Esc>", "'\t'"),
        (r"i\<C-Q>\<C-A>\<Esc>", "'\u{1}'"),
        (r"i\<C-V>x7f\<Esc>", "'\u{7f}'"),
        // NUL is kept in the line as a NL.
        (r"i\<C-V>000\<Esc>", "'\n'"),
        (r"i\<C-V>x00\<Esc>", "'\n'"),
        // A special key goes in as its name.
        (r"i\<C-V>\<Left>\<Esc>", "'<Left>'"),
        (r"i\<C-V>\<C-Left>\<Esc>", "'<C-Left>'"),
        (r"i\<C-V>\<S-F3>\<Esc>", "'<S-F3>'"),
        // ... but an unmapped keypad key is its ASCII twin by then.
        (r"i\<C-V>\<kEnter>\<Esc>", "'\r'"),
        // A modifier that folds into the character does; one that does not
        // is spelled out.
        (r"i\<C-V>\<C-a>\<Esc>", "'\u{1}'"),
        (r"i\<C-V>\<M-a>\<Esc>", "'<M-a>'"),
        (r"i\<C-V>\<C-S-x>\<Esc>", "'<C-S-X>'"),
        // A non-digit ends the number early and goes in after it.
        (r"i\<C-V>x4g\<Esc>", "'\u{4}g'"),
        (r"i\<C-V>u12\<Esc>", "'\u{12}'"),
    ] {
        typed(&fx, "['']", keys, &[("getline(1)", want)]);
    }
    // `.` repeats what CTRL-V produced, not the keys that produced it.
    typed(
        &fx,
        "['', '']",
        r"i\<C-V>u20ac\<C-V>\<Left>\<Esc>j.",
        &[("getline(1, 2)", "['€<Left>', '€<Left>']")],
    );
    restore(&fx);
}

#[test]
fn digraphs_come_from_control_k_and_from_backspace_under_digraph() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    for (keys, want) in [
        (r"i\<C-K>e:\<Esc>", "'ë'"),
        (r"i\<C-K>:e\<Esc>", "'ë'"),
        (r"i\<C-K>Eu\<Esc>", "'€'"),
        (r"i\<C-K>a*\<Esc>", "'α'"),
        // An unknown pair is the second character.
        (r"i\<C-K>qq\<Esc>", "'q'"),
        // A special key after CTRL-K goes in by name.
        (r"i\<C-K>\<Left>\<Esc>", "'<Left>'"),
        (r"i\<C-K>a\<Left>\<Esc>", "'<Left>'"),
        // CTRL-K <Esc> is cancelled.
        (r"ix\<C-K>\<Esc>y\<Esc>", "'xy'"),
    ] {
        typed(&fx, "['']", keys, &[("getline(1)", want)]);
    }
    fx.run("set digraph");
    typed(&fx, "['']", r"ie\<BS>:x\<Esc>", &[("getline(1)", "'ëx'")]);
    fx.run("set nodigraph");
    typed(&fx, "['']", r"ie\<BS>:x\<Esc>", &[("getline(1)", "':x'")]);
    // In Normal mode, `r` and `f` take a digraph too.
    typed(&fx, "['abc']", r"r\<C-K>e:", &[("getline(1)", "'ëbc'")]);
    typed(&fx, "['abëc']", r"f\<C-K>e:x", &[("getline(1)", "'abc'")]);
    restore(&fx);
}

#[test]
fn replace_mode_overwrites_and_backspace_puts_it_back() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    typed(
        &fx,
        "['abcdef']",
        r"lRxyz\<Esc>",
        &[("getline(1)", "'axyzef'")],
    );
    typed(
        &fx,
        "['abcdef']",
        r"lRxyz\<BS>\<BS>\<Esc>",
        &[("getline(1)", "'axcdef'")],
    );
    // Past the end of the line it appends; backspace takes that away.
    typed(
        &fx,
        "['ab']",
        r"lRxyz\<BS>\<Esc>",
        &[("getline(1)", "'axy'")],
    );
    // A multibyte character replaces one character, whatever the bytes.
    typed(
        &fx,
        "['\u{e9}\u{65e5}c']",
        r"Rab\<Esc>",
        &[("getline(1)", "'abc'")],
    );
    typed(
        &fx,
        "['abc']",
        r"R\u00e9\u65e5\<BS>\<Esc>",
        &[("getline(1)", "'ébc'")],
    );
    // A count repeats the replacement text.
    typed(
        &fx,
        "['abcdefgh']",
        r"2Rxy\<Esc>",
        &[("getline(1)", "'xyxyefgh'")],
    );
    // <Insert> toggles to inserting and back.
    typed(
        &fx,
        "['abcdef']",
        r"Rx\<Insert>y\<Insert>z\<Esc>",
        &[("getline(1)", "'xyzcdef'")],
    );
    // Virtual Replace: a TAB is replaced cell by cell.
    fx.run("set tabstop=8");
    typed(&fx, "['\tx']", r"gRab\<Esc>", &[("getline(1)", "'ab\tx'")]);
    typed(
        &fx,
        "['\tx']",
        r"gR12345678\<Esc>",
        &[("getline(1)", "'12345678x'")],
    );
    typed(&fx, "['abc']", r"gR\<Tab>\<Esc>", &[("getline(1)", "'\t'")]);
    // `gr` and `r` replace one character.
    typed(&fx, "['\tx']", r"grz", &[("getline(1)", "'z\tx'")]);
    typed(&fx, "['abc']", r"r\u00e9", &[("getline(1)", "'ébc'")]);
    typed(&fx, "['abcd']", r"3rx", &[("getline(1)", "'xxxd'")]);
    typed(&fx, "['abcd']", r"r\<C-V>065", &[("getline(1)", "'Abcd'")]);
    // `r<CR>` breaks the line once, whatever the count.
    typed(
        &fx,
        "['abcd']",
        r"l2r\<CR>",
        &[("getline(1, '$')", "['a', 'd']")],
    );
    typed(
        &fx,
        "['abcd']",
        r"lr\<C-V>\<CR>",
        &[("getline(1)", "'a\rcd'")],
    );
    restore(&fx);
}

#[test]
fn an_abbreviation_ends_on_the_key_after_it() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    fx.run("iabbrev teh the");
    fx.run("iabbrev #i #include");
    fx.run("iabbrev ;; semicolon");
    for (keys, want) in [
        (r"iteh \<Esc>", "['the ']"),
        (r"iteh.\<Esc>", "['the.']"),
        (r"iteh\<Esc>", "['the']"),
        (r"iteh\<CR>x\<Esc>", "['the', 'x']"),
        (r"iteh\<Tab>\<Esc>", "['the\t']"),
        (r"iteh\<C-]>x\<Esc>", "['thex']"),
        (r"iteh\<C-O>x\<Esc>", "['th']"),
        (r"ixteh \<Esc>", "['xteh ']"),
        (r"iteh\<C-V> \<Esc>", "['teh ']"),
        (r"i#i \<Esc>", "['#include ']"),
        // ... though not one whose characters repeat, upstream as here.
        (r"i;; \<Esc>", "[';; ']"),
        // A non-word abbreviation after a word character is not one.
        (r"ia;; \<Esc>", "['a;; ']"),
        // A multibyte non-word character ends one too.
        ("iteh\u{2014}\\<Esc>", "['the\u{2014}']"),
        // ... but a multibyte word character does not.
        ("iteh\u{e9}\\<Esc>", "['teh\u{e9}']"),
    ] {
        typed(&fx, "['']", keys, &[("getline(1, '$')", want)]);
    }
    fx.run("set paste");
    typed(&fx, "['']", r"iteh \<Esc>", &[("getline(1)", "'teh '")]);
    fx.run("set nopaste");
    // An `<expr>` abbreviation sees the key that ended it in `v:char`,
    // without the offset the callers add to say "no CTRL-V needed".
    fx.run("let g:seen = []");
    fx.run("inoreabbrev <expr> zz [add(g:seen, v:char), 'ZZ'][1]");
    for keys in [
        r"izz \<Esc>",
        r"izz\<Esc>",
        r"izz\<CR>\<Esc>",
        r"izz\<Tab>\<Esc>",
        r"izz\<C-]>\<Esc>",
        r"izz\<C-O>\<Esc>\<Esc>",
        "izz\u{2014}\\<Esc>",
    ] {
        typed(&fx, "['']", keys, &[]);
    }
    fx.check(
        "g:seen",
        "[' ', '\u{1b}', '\r', '\t', '\u{1d}', '\u{f}', '\u{2014}']",
    );
    fx.run("unlet g:seen");
    restore(&fx);
}

#[test]
fn the_command_line_checks_its_own_abbreviations() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    // The command line checks its own abbreviations, on a non-word
    // character and on <CR>.
    fx.run("cabbrev zq let g:cab = 1");
    fx.run("let g:cab = 0");
    fx.run("call feedkeys(\":zq\\<CR>\", 'xt')");
    fx.check("g:cab", "1");
    fx.run("let g:cseen = []");
    fx.run("cnoreabbrev <expr> zx [add(g:cseen, v:char), 'let g:x = 1'][1]");
    // The character that ended it goes on the line after the expansion.
    // The first one after a `<CR>`-ended abbreviation is not checked: the
    // expansion's no-abbreviation count outlives the line, upstream as here
    // (`upstream-neovim-bugs`, `cmdline-abbrev-cr-blocks-next.md`).
    for (keys, x) in [
        (r":zx\<C-]>\<CR>", "0"),
        (r":zx\<C-]>\<CR>", "1"),
        (r":zx \<CR>", "1"),
        (r":zx\u2014\<CR>", "0"),
    ] {
        fx.run("let g:x = 0");
        fx.run(&format!("call feedkeys(\"{keys}\", 'xt')"));
        fx.check("g:x", x);
    }
    fx.check("g:cseen", "['\u{1d}', ' ', '\u{2014}']");
    fx.run("unlet g:cseen g:cab g:x");
    restore(&fx);
}

#[test]
fn insert_char_pre_can_replace_the_character() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    fx.run("augroup EditOracle");
    fx.run(
        "autocmd InsertCharPre * if v:char ==# 'a' | let v:char = 'AB' | \
         elseif v:char ==# 'n' | let v:char = \"x\\ny\" | \
         elseif v:char ==# 'd' | let v:char = '' | endif",
    );
    fx.run("augroup END");
    typed(&fx, "['']", r"iqadz\<Esc>", &[("getline(1)", "'qABz'")]);
    typed(
        &fx,
        "['']",
        r"iqn\<Esc>",
        &[("getline(1, '$')", "['qx', 'y']")],
    );
    // `.` repeats the replaced text, which the event does not see again.
    typed(
        &fx,
        "['', '']",
        r"iab\<Esc>j.",
        &[("getline(1, 2)", "['ABb', 'ABb']")],
    );
    restore(&fx);
}

#[test]
fn the_control_keys_edit_around_the_cursor() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    fx.run("set backspace=indent,eol,start");
    for (lines, keys, want) in [
        // CTRL-W and CTRL-U delete a word and the line before the cursor.
        ("['foo bar']", r"A\<C-W>x\<Esc>", "['foo x']"),
        ("['foo bar']", r"A baz\<C-U>x\<Esc>", "['foo barx']"),
        ("['foo bar']", r"A\<C-U>x\<Esc>", "['x']"),
        // Backspace joins lines under 'backspace' eol.
        ("['ab', 'cd']", r"ji\<BS>\<BS>x\<Esc>", "['axcd']"),
        ("['ab', 'cd']", r"ji\<Del>x\<Esc>", "['ab', 'xd']"),
        // CTRL-T and CTRL-D change the indent by 'shiftwidth'.
        ("['abc']", r"A\<C-T>\<C-T>\<C-D>\<Esc>", "['\tabc']"),
        ("['\t\tabc']", r"A0\<C-D>\<Esc>", "['abc']"),
        ("['\t\tabc']", r"A^\<C-D>x\<Esc>", "['abcx']"),
        // CTRL-E and CTRL-Y copy the character below and above.
        (
            "['abcd', '', 'wxyz']",
            r"ji\<C-Y>\<C-Y>\<C-E>\<Esc>",
            "['abcd', 'aby', 'wxyz']",
        ),
        (
            "['\u{e9}\u{65e5}', '']",
            r"ji\<C-Y>\<C-Y>\<Esc>",
            "['\u{e9}\u{65e5}', '\u{e9}\u{65e5}']",
        ),
        // CTRL-A inserts what was last inserted; CTRL-@ does and leaves.
        (
            "['', '']",
            r"ifoo\<Esc>ji\<C-A>\<C-A>\<Esc>",
            "['foo', 'foofoo']",
        ),
        ("['', '']", r"ifoo\<Esc>ji\<C-@>x", "['foo', 'fo']"),
        // CTRL-R inserts a register, as typed or literally.
        (
            "['']",
            r":let @a = 'x\<C-V>\<C-V>\<C-V>\<C-H>y'\<CR>i\<C-R>a\<Esc>",
            "['x\u{16}\u{8}y']",
        ),
        (
            "['']",
            r":let @a = 'x\<C-V>\<C-V>\<C-V>\<C-H>y'\<CR>i\<C-R>\<C-R>a\<Esc>",
            "['x\u{16}\u{8}y']",
        ),
        ("['']", r"i\<C-R>=6*7\<CR>\<Esc>", "['42']"),
        // CTRL-O runs one Normal-mode command and comes back.
        ("['abc']", r"A\<C-O>0x\<Esc>", "['xabc']"),
        ("['abc def']", r"i\<C-O>wx\<Esc>", "['abc xdef']"),
        // CTRL-G u splits the undo, CTRL-G j/k move to the start column.
        ("['abc']", r"Ax\<C-G>uy\<Esc>u", "['abcx']"),
        ("['ab', 'cd']", r"lix\<C-G>jy\<Esc>", "['axb', 'cyd']"),
        // The arrow keys move without leaving the mode, and stop `.`.
        (
            "['abc']",
            r"ix\<Right>y\<End>z\<Home>w\<Esc>",
            "['wxaybcz']",
        ),
        (
            "['abc def']",
            r"A\<C-Left>x\<S-Left>y\<Esc>",
            "['abc yxdef']",
        ),
        // Shift-Tab is a Tab; <kEnter> is a CR.
        ("['ab']", r"a\<S-Tab>\<kEnter>x\<Esc>", "['a\t', 'xb']"),
    ] {
        typed(&fx, lines, keys, &[("getline(1, '$')", want)]);
    }
    // 'expandtab', 'softtabstop' and 'smarttab' turn a Tab into spaces.
    fx.run("set expandtab shiftwidth=4 softtabstop=4");
    typed(
        &fx,
        "['']",
        r"i\<Tab>x\<Tab>\<BS>y\<Esc>",
        &[("getline(1)", "'    xy'")],
    );
    fx.run("set noexpandtab shiftwidth=8 softtabstop=0");
    restore(&fx);
}

#[test]
fn text_width_breaks_at_the_blank_typed_last() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    fx.run("set textwidth=10 formatoptions=tq");
    typed(
        &fx,
        "['']",
        r"iaaa bbb ccc ddd\<Esc>",
        &[("getline(1, '$')", "['aaa bbb', 'ccc ddd']")],
    );
    // Backspacing over a blank that was typed in this insert removes it;
    // with 'backspace' empty a blank that was there before stays.
    fx.run("set backspace=");
    typed(
        &fx,
        "['aaa ']",
        r"A \<BS>\<BS>x\<Esc>",
        &[("getline(1, '$')", "['aaa x']")],
    );
    fx.run("set formatoptions+=b");
    typed(
        &fx,
        "['aaaa bbbb']",
        r"A ccc ddd\<Esc>",
        &[("getline(1, '$')", "['aaaa bbbb', 'ccc ddd']")],
    );
    restore(&fx);
}

#[test]
fn revins_and_rightleft_reverse_the_direction() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    fx.run("set allowrevins");
    typed(
        &fx,
        "['']",
        r"iab\<C-_>cd\<C-_>ef\<Esc>",
        &[("getline(1)", "'abdcef'")],
    );
    fx.run("set noallowrevins");
    // Without 'allowrevins' CTRL-_ goes in as a character.
    typed(
        &fx,
        "['']",
        r"ia\<C-_>b\<Esc>",
        &[("getline(1)", "'a\u{1f}b'")],
    );
    fx.run("set revins");
    typed(&fx, "['']", r"iabc\<Esc>", &[("getline(1)", "'cba'")]);
    fx.run("set norevins");
    // In a 'rightleft' window <Left> and <Right> trade places.
    fx.run("set rightleft");
    typed(
        &fx,
        "['abc']",
        r"A\<Right>\<Right>x\<Esc>",
        &[("getline(1)", "'axbc'")],
    );
    fx.run("set norightleft");
    restore(&fx);
}

#[test]
fn normal_mode_reads_its_character_arguments_as_keys() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    for (lines, keys, want) in [
        ("['a\u{e9}b\u{e9}c']", r"2f\u00e9x", "['a\u{e9}bc']"),
        ("['a\u{e9}b\u{e9}c']", r"$F\u00e9x", "['a\u{e9}bc']"),
        ("['abcabc']", r"tcx;x", "['acac']"),
        ("['abcabc']", r"$Tax,x", "['abca']"),
        ("['abc']", r"ma$`ax", "['bc']"),
        ("['abc', 'def']", "\\\"ayyj\\\"ap", "['abc', 'def', 'abc']"),
        ("['abc']", r"qqA!\<Esc>q@q", "['abc!!']"),
        ("['abc']", r"~~", "['ABc']"),
        ("['abc def']", r"gUiw", "['ABC def']"),
        ("['abc']", "r\\<C-V>\\<Tab>", "['\tbc']"),
        ("['a b']", r"f r\<CR>", "['a', 'b']"),
        ("['abc']", r"s\u00e9\<Esc>", "['\u{e9}bc']"),
        ("['abc']", r"ylvp", "['abc']"),
        ("['5']", "\\<C-A>", "['6']"),
        ("['x 5']", "3\\<C-X>", "['x 2']"),
        ("['abc']", r"ix\<Esc>u\<C-R>", "['xabc']"),
        ("['abc']", r"ga", "['abc']"),
        ("['a', 'b', 'c']", r"Jx", "['ab', 'c']"),
        ("['abc', 'xyz']", r"yyjVp", "['abc', 'abc']"),
        ("['abc']", r"z\<CR>x", "['bc']"),
    ] {
        typed(&fx, lines, keys, &[("getline(1, '$')", want)]);
    }
    restore(&fx);
}
