//! The command line: what editing keys do to its text, how `<Up>` walks the
//! history, and the `:history` range argument.
//!
//! The oracle the `ex_getln/` retype is checked against. The command line's
//! text lives in a `CmdBuff` that `handlekey.rs` edits a key at a time; the
//! cases type keys into a real command line through `:normal` and read the
//! line and the cursor back from inside it with `<C-r>=`, which is how a
//! mapping would see them. History recall (`command_line_browse_history`,
//! prefix-matching) and `get_list_range` (`:history 2,4`) are reached the
//! same way. Each answer is today's, and upstream 0.12.6's.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};

/// `Rec()`: append `[getcmdline(), getcmdpos(), getcmdtype()]` to `g:rec`
/// and insert nothing.
const PRELUDE: &str = r#"
func! Rec()
  call add(g:rec, [getcmdline(), getcmdpos(), getcmdtype()])
  return ''
endfunc
"#;

fn setup(sandbox: &Sandbox, fx: &Probe<'_>) {
    fx.home();
    sandbox.write("prelude.vim", PRELUDE.as_bytes());
    fx.run("source prelude.vim");
}

fn teardown(fx: &Probe<'_>) {
    fx.run("silent! delfunction Rec");
    fx.run("silent! unlet g:rec");
    fx.run("call histdel(':') | call histdel('/')");
    fx.home();
}

/// Type `keys` (Vimscript double-quoted notation, `R` standing for
/// `<C-r>=Rec()<CR>`) and abandon the line with `<C-c>` -- not `<Esc>`, which
/// from `:normal` *executes* it -- and answer what `Rec` saw.
fn typed(fx: &Probe<'_>, keys: &str) -> String {
    fx.run("let g:rec = []");
    let keys = keys.replace('R', "\\<C-r>=Rec()\\<CR>");
    fx.run(&format!("silent execute \"normal! {keys}\\<C-c>\""));
    fx.eval("g:rec")
}

#[test]
fn editing_keys_change_the_text_and_the_cursor() {
    let sandbox = Sandbox::dir("cmdline_keys");
    let fx = Probe::new(sandbox.editor());
    setup(&sandbox, &fx);
    fx.run("call setline(1, 'under cursor')");
    let rows: &[(&str, &str)] = &[
        (":abcR", "[['abc', 4, ':']]"),
        (":abc\\<Left>\\<Left>XR", "[['aXbc', 3, ':']]"),
        // <C-w> takes the word before the cursor, then the space.
        (
            ":foo bar baz\\<C-w>R\\<C-w>R",
            "[['foo bar ', 9, ':'], ['foo ', 5, ':']]",
        ),
        (":foo bar\\<C-u>R", "[['', 1, ':']]"),
        // <C-u> only takes what is before the cursor.
        (
            ":foo bar\\<Left>\\<Left>\\<Left>\\<C-u>R",
            "[['bar', 1, ':']]",
        ),
        // Positions are bytes; <BS> takes a whole character.
        (
            ":é日x\\<Left>R\\<BS>R",
            "[['é日x', 6, ':'], ['éx', 3, ':']]",
        ),
        (
            ":abc\\<Home>R\\<Del>R\\<End>R",
            "[['abc', 1, ':'], ['bc', 1, ':'], ['bc', 3, ':']]",
        ),
        (
            ":abc\\<C-b>XR\\<C-e>YR",
            "[['Xabc', 2, ':'], ['XabcY', 6, ':']]",
        ),
        // <C-v> inserts the next key literally, or a codepoint.
        (":a\\<C-v>\\<Tab>b\\<C-v>u00e9R", "[['a\tbé', 6, ':']]"),
        // <S-Right>/<S-Left> by WORD.
        (
            ":one two three\\<S-Left>R\\<S-Left>R",
            "[['one two three', 9, ':'], ['one two three', 5, ':']]",
        ),
        // <C-r><C-w>: the word under the cursor in the buffer.
        (":x \\<C-r>\\<C-w>R", "[['x under', 8, ':']]"),
        // A search line is another type, the same buffer.
        ("/patR", "[['pat', 4, '/']]"),
        // <BS> on an empty line leaves the command line.
        (":\\<BS>R", "[]"),
    ];
    let mut moved = Vec::new();
    for (keys, want) in rows {
        let got = typed(&fx, keys);
        if got != *want {
            moved.push(format!("({keys:?}, {got:?})"));
        }
    }
    // setcmdpos() from an expression moves the cursor for the next key.
    fx.run("let g:rec = []");
    fx.run("silent execute \"normal! :abcd\\<C-r>=setcmdpos(2) ? '' : ''\\<CR>X\\<C-r>=Rec()\\<CR>\\<C-c>\"");
    let setpos = fx.eval("g:rec");
    teardown(&fx);
    if setpos != "[['aXbcd', 3, ':']]" {
        moved.push(format!("setcmdpos: {setpos:?}"));
    }
    assert!(moved.is_empty(), "rows moved:\n{}", moved.join("\n"));
}

#[test]
fn up_recalls_history_that_starts_with_the_typed_text() {
    let sandbox = Sandbox::dir("cmdline_history");
    let fx = Probe::new(sandbox.editor());
    setup(&sandbox, &fx);
    fx.run("call histdel(':')");
    for entry in ["echo 'one'", "let x = 1", "echo 'two'", "echo 'three'"] {
        fx.run(&format!("call histadd(':', \"{entry}\")"));
    }
    // A failed recall beeps, and a beep abandons the rest of `:normal` --
    // which is how "no further match" shows here: nothing is recorded.
    let rows: &[(&str, &str)] = &[
        // No prefix: newest first.
        (":\\<Up>R", "[['echo ''three''', 13, ':']]"),
        (":\\<Up>\\<Up>R", "[['echo ''two''', 11, ':']]"),
        // A prefix skips what does not match it.
        (":ec\\<Up>\\<Up>\\<Up>R", "[['echo ''one''', 11, ':']]"),
        (":let\\<Up>R", "[['let x = 1', 10, ':']]"),
        // Past the oldest match: a beep.
        (":let\\<Up>\\<Up>R", "[]"),
        // <Down> past the newest gives back what was typed.
        (":ec\\<Up>\\<Down>R", "[['ec', 3, ':']]"),
        // <S-Up> ignores the prefix.
        (":ec\\<S-Up>\\<S-Up>R", "[['echo ''two''', 11, ':']]"),
        // Nothing matches at all: a beep.
        (":zz\\<Up>R", "[]"),
        // Once an expression has run, the whole line is the prefix: the next
        // <Up> finds nothing older that starts with it.
        (":\\<Up>R\\<Up>R", "[['echo ''three''', 13, ':']]"),
    ];
    let mut moved = Vec::new();
    for (keys, want) in rows {
        let got = typed(&fx, keys);
        if got != *want {
            moved.push(format!("({keys:?}, {got:?})"));
        }
    }
    teardown(&fx);
    assert!(moved.is_empty(), "rows moved:\n{}", moved.join("\n"));
}

#[test]
fn the_history_listing_takes_a_range() {
    let sandbox = Sandbox::dir("cmdline_history_range");
    let fx = Probe::new(sandbox.editor());
    setup(&sandbox, &fx);
    fx.run("call histdel(':')");
    for n in 1..=5 {
        fx.run(&format!("call histadd(':', 'cmd{n}')"));
    }
    let listing = |range: &str| {
        let out = fx.output(&format!("history : {range}"));
        // Only the entries' text: the numbers' padding is not the point.
        out.lines()
            .skip(2)
            .map(|l| l.split_whitespace().last().unwrap_or_default().to_string())
            .collect::<Vec<_>>()
            .join(",")
    };
    let rows: &[(&str, &str)] = &[
        ("", "cmd1,cmd2,cmd3,cmd4,cmd5"),
        ("2,4", "cmd2,cmd3,cmd4"),
        ("3", "cmd3"),
        // From one end or the other.
        (",2", "cmd1,cmd2"),
        ("4,", "cmd4,cmd5"),
        // Negative: counted back from the newest.
        ("-2", "cmd4"),
        ("-3,-2", "cmd3,cmd4"),
        // Out of range is empty, not an error.
        ("9", ""),
        ("  2 , 3 ", "cmd2,cmd3"),
    ];
    let mut moved = Vec::new();
    for (range, want) in rows {
        let got = listing(range);
        if got != *want {
            moved.push(format!("({range:?}, {got:?})"));
        }
    }
    let errors = [
        fx.error_from("history : ,"),
        fx.error_from("history : 1,x"),
        fx.error_from("history : 99999999999"),
    ];
    teardown(&fx);
    // A lone `,` is reported as a number too large, as upstream does: the
    // parse fails with nothing after it, and that is the message for it.
    let want = [
        "E1510: Value too large: : ,",
        "E488: Trailing characters: x",
        "E1510: Value too large: : 99999999999",
    ];
    for ((what, got), want) in [", alone", "a bad end", "too large"]
        .iter()
        .zip(&errors)
        .zip(want)
    {
        if got != want {
            moved.push(format!("error {what}: {got:?}"));
        }
    }
    assert!(moved.is_empty(), "rows moved:\n{}", moved.join("\n"));
}
