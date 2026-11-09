//! CTRL-W: every letter `do_window` dispatches on, and every other key that
//! does the same thing -- the control character of the letter and the
//! special key, where there is one.
//!
//! The battery's `winsweep` issues each letter through `:wincmd`; nothing
//! pinned that CTRL-W CTRL-S is CTRL-W s, or CTRL-W `<Down>` CTRL-W j. The
//! dispatch matches a key against both halves at once (the letters and the
//! named keys), so each spelling of a command is a separate arm pattern a
//! retype can drop. Each case runs every spelling from the same layout and
//! asserts that they leave the same windows behind.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};

/// Three windows in a column over three named buffers, the middle one
/// current, with `alpha.txt` as its alternate file.
fn layout(fx: &Probe<'_>) {
    fx.home();
    fx.run("silent! pclose");
    fx.run("silent! cclose");
    fx.run("set equalalways&");
    fx.run("silent file alpha.txt");
    fx.run("call setline(1, ['alpha', 'two'])");
    fx.run("set nomodified");
    fx.run("silent new beta.txt");
    fx.run("silent new gamma.txt");
    fx.run("silent edit delta.txt");
    fx.run("wincmd j");
    fx.run("silent edit! epsilon.txt");
    fx.run("silent edit! #");
    fx.run("silent edit! epsilon.txt");
}

/// What a command leaves behind: the layout, which window is current, the
/// tab pages and what the current window shows.
fn snapshot(fx: &Probe<'_>) -> String {
    fx.eval(
        "[winnr(), winnr('$'), tabpagenr(), tabpagenr('$'), bufname(), \
         map(range(1, winnr('$')), {_, n -> \
         [win_screenpos(n), winheight(n), winwidth(n), bufname(winbufnr(n))]})]",
    )
}

/// Each spelling in `keys` (a Vim-script double-quoted string body typed
/// after CTRL-W in Normal mode) leaves the same windows as the first.
#[track_caller]
fn same_command(fx: &Probe<'_>, keys: &[&str]) {
    let mut first: Option<String> = None;
    for k in keys {
        layout(fx);
        let before = snapshot(fx);
        fx.run(&format!("silent! exe \"normal \\<C-W>{k}\""));
        let got = snapshot(fx);
        fx.run("silent! tabonly!");
        match &first {
            None => {
                assert_ne!(got, before, "CTRL-W {k} did nothing");
                first = Some(got);
            }
            Some(want) => assert_eq!(&got, want, "CTRL-W {k} is not CTRL-W {}", keys[0]),
        }
    }
    fx.settle();
}

#[test]
fn every_spelling_of_a_ctrl_w_command_does_the_same() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    for keys in [
        &["s", "S", "\\<C-S>"][..],
        &["v", "\\<C-V>"],
        &["^", "\\<C-^>"],
        &["n", "\\<C-N>"],
        &["q", "\\<C-Q>"],
        &["c", "\\<C-C>"],
        &["o", "\\<C-O>"],
        &["w", "\\<C-W>"],
        &["j", "\\<C-J>", "\\<Down>"],
        &["k", "\\<C-K>", "\\<Up>"],
        &["t", "\\<C-T>"],
        &["b", "\\<C-B>"],
        &["x", "\\<C-X>"],
        &["r", "\\<C-R>"],
        &["_", "\\<C-_>"],
    ] {
        same_command(&fx, keys);
    }
    // `g`: the second character picks the command; `gt` needs a second tab.
    let mut first: Option<String> = None;
    for k in ["gt", "\\<C-G>t"] {
        layout(&fx);
        fx.run("tabnew");
        fx.run("tabfirst");
        fx.run(&format!("silent! exe \"normal \\<C-W>{k}\""));
        let got = snapshot(&fx);
        fx.run("silent! tabonly!");
        match &first {
            None => {
                assert_eq!(fx.eval("tabpagenr()"), "1", "after tabonly");
                first = Some(got);
            }
            Some(want) => assert_eq!(&got, want, "CTRL-W {k}"),
        }
    }
    assert!(
        first.is_some_and(|got| got.starts_with("[1, 1, 2, 2")),
        "CTRL-W gt went to tab 2"
    );
    // Back to where the previous window was: `p` after a move.
    for keys in [&["p", "\\<C-P>"][..]] {
        let mut first: Option<String> = None;
        for k in keys {
            layout(&fx);
            fx.run("wincmd k");
            fx.run("wincmd j");
            fx.run(&format!("silent! exe \"normal \\<C-W>{k}\""));
            let got = snapshot(&fx);
            match &first {
                None => first = Some(got),
                Some(want) => assert_eq!(&got, want, "CTRL-W {k}"),
            }
        }
    }
    fx.home();
}

#[test]
fn the_sideways_and_preview_spellings_do_the_same() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    // Left and right need a window beside the current one.
    for keys in [
        &["h", "\\<C-H>", "\\<Left>", "\\<BS>"][..],
        &["l", "\\<C-L>", "\\<Right>"],
    ] {
        let mut first: Option<String> = None;
        for k in keys {
            layout(&fx);
            fx.run("vsplit");
            if keys[0] == "h" {
                fx.run("wincmd l");
            }
            let before = snapshot(&fx);
            fx.run(&format!("silent! exe \"normal \\<C-W>{k}\""));
            let got = snapshot(&fx);
            match &first {
                None => {
                    assert_ne!(got, before, "CTRL-W {k} did nothing");
                    first = Some(got);
                }
                Some(want) => assert_eq!(&got, want, "CTRL-W {k}"),
            }
        }
    }
    // `z` closes the preview window.
    let mut first: Option<String> = None;
    for k in ["z", "\\<C-Z>"] {
        layout(&fx);
        fx.run("silent pedit alpha.txt");
        let before = snapshot(&fx);
        fx.run(&format!("silent! exe \"normal \\<C-W>{k}\""));
        let got = snapshot(&fx);
        match &first {
            None => {
                assert_ne!(got, before, "CTRL-W {k} did nothing");
                first = Some(got);
            }
            Some(want) => assert_eq!(&got, want, "CTRL-W {k}"),
        }
    }
    fx.run("silent! pclose");
    // `<CR>` and CTRL-M in the quickfix window: the entry in a new split.
    let mut first: Option<String> = None;
    for k in ["\\<CR>", "\\<C-M>", "\\<kEnter>"] {
        layout(&fx);
        fx.run("call setqflist([{'filename': 'alpha.txt', 'lnum': 2, 'text': 'x'}])");
        fx.run("silent copen");
        let before = snapshot(&fx);
        fx.run(&format!("silent! exe \"normal \\<C-W>{k}\""));
        let got = snapshot(&fx);
        fx.run("silent! cclose");
        match &first {
            None => {
                assert_ne!(got, before, "CTRL-W {k} did nothing");
                first = Some(got);
            }
            Some(want) => assert_eq!(&got, want, "CTRL-W {k}"),
        }
    }
    fx.run("call setqflist([])");
    fx.settle();
    fx.home();
}

#[test]
fn the_under_the_cursor_spellings_do_the_same() {
    let sandbox = Sandbox::dir("the_under_the_cursor_spellings_do_the_same");
    let fx = Probe::new(sandbox.editor());
    let file = sandbox.write("zeta.txt", b"zeta\n");
    // `f` edits the file name under the cursor, `i` jumps to the first line
    // with the identifier, `d` to its definition: the text is the case.
    for (text, cursor, keys) in [
        (
            &*format!("['{}']", file.display()),
            1,
            &["f", "\\<C-F>"][..],
        ),
        ("['foo', 'bar', 'foo']", 3, &["i", "\\<C-I>"]),
        ("['#define foo 1', 'bar', 'foo']", 3, &["d", "\\<C-D>"]),
    ] {
        let mut first: Option<String> = None;
        for k in keys {
            layout(&fx);
            fx.run(&format!("call setline(1, {text})"));
            fx.run(&format!("call cursor({cursor}, 1)"));
            fx.run("set nomodified");
            fx.run(r"setlocal define=^\s*#\s*define");
            let before = snapshot(&fx);
            let cursor_before = fx.eval("[line('.'), col('.')]");
            fx.run(&format!("silent! exe \"normal \\<C-W>{k}\""));
            let got = format!("{} {}", snapshot(&fx), fx.eval("[line('.'), col('.')]"));
            match &first {
                None => {
                    assert_ne!(
                        got,
                        format!("{before} {cursor_before}"),
                        "CTRL-W {k} did nothing"
                    );
                    first = Some(got);
                }
                Some(want) => assert_eq!(&got, want, "CTRL-W {k}"),
            }
            fx.run("silent! %bwipeout!");
            fx.run("set define&");
        }
    }
    fx.settle();
    fx.home();
}
