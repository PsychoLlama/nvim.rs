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

#[test]
fn history_recall_wraps_the_ring_and_fixes_search_separators() {
    let sandbox = Sandbox::dir("cmdline_history_ring");
    let fx = Probe::new(sandbox.editor());
    setup(&sandbox, &fx);
    let saved = fx.eval("&history");
    // A ring of three: the fifth entry wraps over the first two.
    fx.run("set history=3");
    fx.run("call histdel(':')");
    for n in 1..=5 {
        fx.run(&format!("call histadd(':', 'cmd{n}')"));
    }
    fx.run("call histdel('/')");
    fx.run(r"call histadd('/', 'a/b')");
    fx.run(r"call histadd('/', 'c\/d?e')");
    let rows: &[(&str, &str)] = &[
        (":\\<Up>R", "[['cmd5', 5, ':']]"),
        (":\\<Up>\\<Up>\\<Up>R", "[['cmd3', 5, ':']]"),
        // Past the oldest of the three: a beep.
        (":\\<Up>\\<Up>\\<Up>\\<Up>R", "[]"),
        // Down from the oldest walks forward across the wrap.
        (":\\<Up>\\<Up>\\<Up>\\<Down>R", "[['cmd4', 5, ':']]"),
        (":\\<Up>\\<Up>\\<Up>\\<Down>\\<Down>R", "[['cmd5', 5, ':']]"),
        // Once an expression has run the recalled line is the prefix, and
        // nothing newer starts with it: <Down> gives it back.
        (
            ":\\<Up>\\<Up>\\<Up>\\<Down>R\\<Down>R",
            "[['cmd4', 5, ':'], ['cmd4', 5, ':']]",
        ),
        // Recalled under the separator it was added with: as stored.
        ("/\\<Up>R", r"[['c\/d?e', 7, '/']]"),
        // Under `?` the new separator is escaped; an escaped old one stays.
        ("?\\<Up>R", r"[['c\/d\?e', 8, '?']]"),
        ("?\\<Up>\\<Up>R", "[['a/b', 4, '?']]"),
    ];
    let mut moved = Vec::new();
    for (keys, want) in rows {
        let got = typed(&fx, keys);
        if got != *want {
            moved.push(format!("({keys:?}, {got:?})"));
        }
    }
    fx.run(&format!("set history={saved}"));
    teardown(&fx);
    assert!(moved.is_empty(), "rows moved:\n{}", moved.join("\n"));
}

#[test]
fn ctrl_r_inserts_registers_literally_or_as_typed() {
    let sandbox = Sandbox::dir("cmdline_ctrl_r");
    let fx = Probe::new(sandbox.editor());
    setup(&sandbox, &fx);
    fx.run("call setline(1, 'under cursor')");
    fx.run("let @a = 'xyz'");
    // A register holding a backspace.
    fx.run(r#"let @b = "p\<C-h>q""#);
    let saved_is = fx.eval("&incsearch");
    let rows: &[(&str, &str)] = &[
        (":\\<C-r>aR", "[['xyz', 4, ':']]"),
        (":1\\<C-r>a2R", "[['1xyz2', 6, ':']]"),
        // A backspace from a register goes in as a character, typed or
        // literal -- upstream 0.12.6 answers the same.
        (":\\<C-r>bR", "[['p\u{8}q', 4, ':']]"),
        (":\\<C-r>\\<C-r>bR", "[['p\u{8}q', 4, ':']]"),
        (":\\<C-r>\\<C-o>bR", "[['p\u{8}q', 4, ':']]"),
        // An expression (`\r`, because `R` is the recorder).
        (":\\<C-r>=1+2\\rR", "[['3', 2, ':']]"),
        // <Esc> cancels the register.
        (":ab\\<C-r>\\<Esc>R", "[['ab', 3, ':']]"),
        // An unknown register inserts nothing.
        (":ab\\<C-r>?R", "[['ab', 3, ':']]"),
        // CTRL-R CTRL-W while 'incsearch' skips what is already typed.
        ("/und\\<C-r>\\<C-w>R", "[['under', 6, '/']]"),
    ];
    let mut moved = Vec::new();
    for (keys, want) in rows {
        if keys.starts_with('/') {
            fx.run("set incsearch");
        }
        let got = typed(&fx, keys);
        if got != *want {
            moved.push(format!("({keys:?}, {got:?})"));
        }
    }
    fx.run(&format!("let &incsearch = {saved_is}"));
    teardown(&fx);
    assert!(moved.is_empty(), "rows moved:\n{}", moved.join("\n"));
}

#[test]
fn prompts_answer_what_was_typed() {
    let sandbox = Sandbox::dir("cmdline_prompt");
    let fx = Probe::new(sandbox.editor());
    setup(&sandbox, &fx);
    // What the prompt shows and answers, read from inside it; and what
    // `CmdlineChanged` sees of it, one entry per change.
    sandbox.write(
        "prompt.vim",
        b"func! PromptRec()
  call add(g:rec, [getcmdline(), getcmdprompt(), getcmdtype(), getcmdpos()])
  return ''
endfunc
func! PromptComplete(a, l, p)
  return ['alpha', 'beta']
endfunc
augroup unit_prompt
  autocmd!
  autocmd CmdlineChanged [@-] call add(g:changed, [getcmdline(), expand('<afile>')])
augroup END
",
    );
    fx.run("source prompt.vim");
    let prompt = |feed: &str, call: &str| {
        fx.run("let g:rec = [] | let g:changed = []");
        fx.run(&format!("call feedkeys(\"{feed}\", 'nt')"));
        fx.run(&format!("let g:got = {call}"));
        // Whatever a case left unread must not reach the next one.
        fx.run("while getchar(0) | endwhile");
        format!(
            "{} {} {}",
            fx.eval("g:got"),
            fx.eval("g:rec"),
            fx.eval("g:changed")
        )
    };
    let rows: &[(&str, &str, &str)] = &[
        (
            "ab\\<C-r>=PromptRec()\\<CR>c\\<CR>",
            "input('Name: ')",
            "'abc' [['ab', 'Name: ', '@', 3]] [['a', '@'], ['ab', '@'], ['abc', '@']]",
        ),
        // The default text is typed ahead of what is fed.
        (
            "\\<C-r>=PromptRec()\\<CR>!\\<CR>",
            "input('> ', 'def')",
            "'def!' [['def', '> ', '@', 4]] [['d', '@'], ['de', '@'], ['def', '@'], ['def!', '@']]",
        ),
        // Only the text after the last newline is the prompt.
        (
            "x\\<C-r>=PromptRec()\\<CR>\\<CR>",
            "input(\"two\\nlines: \")",
            "'x' [['x', 'lines: ', '@', 2]] [['x', '@']]",
        ),
        // A secret prompt answers the text but hides it from getcmdline(),
        // and refuses CTRL-R = -- the `=` names no register.
        (
            "pw\\<C-r>=d\\<CR>",
            "inputsecret('Password: ')",
            "'pwd' [] [['', '-'], ['', '-'], ['', '-']]",
        ),
        // <BS> past the start of a prompt does not leave it.
        ("\\<BS>a\\<CR>", "input('> ')", "'a' [] [['a', '@']]"),
        // <Esc> abandons a prompt: the cancel value, or empty.
        (
            "abc\\<Esc>",
            "input('> ')",
            "'' [] [['a', '@'], ['ab', '@'], ['abc', '@']]",
        ),
        (
            "abc\\<Esc>",
            "inputdialog('> ', '', 'gone')",
            "'gone' [] [['a', '@'], ['ab', '@'], ['abc', '@']]",
        ),
        (
            "\\<Esc>",
            "input({'prompt': '> ', 'cancelreturn': 'dict'})",
            "'dict' [] []",
        ),
        // Completion from a list.
        (
            // The list is not filtered by what was typed: the first item.
            "b\\<Tab>\\<C-r>=PromptRec()\\<CR>\\<CR>",
            "input('> ', '', 'customlist,PromptComplete')",
            "'alpha' [['alpha', '> ', '@', 6]] [['b', '@'], ['alpha', '@']]",
        ),
        // inputlist(): a number, or 0 for q.
        (
            "2\\<CR>",
            "inputlist(['pick', '1. a', '2. b'])",
            "2 [] [['2', '-']]",
        ),
        ("q", "inputlist(['pick', '1. a', '2. b'])", "0 [] []"),
    ];
    let mut moved = Vec::new();
    for (feed, call, want) in rows {
        let got = prompt(feed, call);
        if got != *want {
            moved.push(format!("({call:?}, {got:?})"));
        }
    }
    fx.run("silent! autocmd! unit_prompt | silent! augroup! unit_prompt");
    fx.run("silent! delfunction PromptRec");
    fx.run("silent! delfunction PromptComplete");
    fx.run("silent! unlet g:got g:changed");
    teardown(&fx);
    assert!(moved.is_empty(), "rows moved:\n{}", moved.join("\n"));
}

#[test]
fn the_command_line_window_round_trips_the_line() {
    let sandbox = Sandbox::dir("cmdline_cmdwin");
    let fx = Probe::new(sandbox.editor());
    setup(&sandbox, &fx);
    // The editor's lone window may never have been given a height, and the
    // command-line window is a split of it.
    fx.run("set lines=24 columns=80");
    fx.run("resize 10 | resize 22");
    fx.run("call histdel(':')");
    fx.run("call histadd(':', 'let g:old = 1')");
    let window = |feed: &str| {
        fx.run("let g:rec = [] | silent! unlet g:ran");
        fx.run("call histdel(':') | call histadd(':', 'let g:old = 1')");
        let err = fx.error_from(&format!("call feedkeys(\"{feed}\", 'xt')"));
        fx.run("while getchar(0) | endwhile");
        format!(
            "{} {} {err}",
            fx.eval("get(g:, 'ran', 'none')"),
            fx.eval("g:rec")
        )
    };
    // `q:` rather than 'cedit': under `feedkeys(.., 'x')` the key loop is
    // inside `:normal`, where only the window's own key opens it.
    let rows: &[(&str, &str)] = &[
        // The empty last line runs nothing.
        ("q:\\<CR>", "'none' [] none"),
        // Typed in the window and run from there.
        ("q:ilet g:ran = 'typed'\\<Esc>\\<CR>", "'typed' [] none"),
        // An older entry is a line above, edited before it runs.
        ("q:k0Clet g:ran = 'edited'\\<Esc>\\<CR>", "'edited' [] none"),
        // CTRL-C goes back to the command line with the cursor line on it.
        (
            "q:k\\<C-c>\\<C-r>=Rec()\\<CR>\\<C-c>",
            "'none' [['let g:old = 1', 1, ':']] none",
        ),
        ("q:k\\<CR>", "'none' [] none"),
    ];
    let mut moved = Vec::new();
    for (feed, want) in rows {
        let got = window(feed);
        if got != *want {
            moved.push(format!("({feed:?}, {got:?})"));
        }
    }
    let old = fx.eval("get(g:, 'old', 0)");
    fx.run("silent! unlet g:old");
    fx.run("redraw!");
    teardown(&fx);
    if old != "1" {
        moved.push(format!("q: history: {old:?}"));
    }
    assert!(moved.is_empty(), "rows moved:\n{}", moved.join("\n"));
}
