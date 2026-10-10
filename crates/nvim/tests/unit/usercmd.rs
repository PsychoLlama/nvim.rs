//! User commands that take a range or a count: `<line1>`, `<line2>`,
//! `<range>` and `<count>` under `-range`, `-range=%`, `-range=N`,
//! `-count=N` and the `-addr=` types, and the errors a bad range raises.
//!
//! In the unit lane because a range is lines in a real buffer, which the
//! lib tests' fixture has no memline for; the rest of `usercmd/` is
//! covered there (`usercmd/tests.rs`). Written against the behaviour before
//! a `UserCmd` owns its strings; a row that moves is a behaviour change.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};

#[test]
fn ranges_counts_and_address_types() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fx.home();
    fx.run("silent! comclear | let g:uc = []");
    fx.check_error("call setline(1, ['a', 'b', 'c', 'd', 'e'])", "none");
    for cmd in [
        "command -range UcRange call add(g:uc, [<line1>, <line2>, <range>])",
        "command -range=% UcAll call add(g:uc, [<line1>, <line2>, <range>])",
        "command -range=3 UcDefCount call add(g:uc, [<line1>, <line2>, <count>, <range>])",
        "command -count=7 UcCount call add(g:uc, [<count>, <line1>, <line2>, <range>])",
        "command -count UcCount0 call add(g:uc, [<count>, <range>])",
        // Buffer numbers depend on every case that ran before; ask where they
        // point instead.
        "command -range -addr=buffers UcBufs call add(g:uc, [<line1> == bufnr(), <line2> == bufnr(), <line2> == bufnr('$')])",
        "command -range -addr=other UcOther call add(g:uc, [<line1>, <line2>])",
        "command -count -addr=tabs UcTabs call add(g:uc, [<count>])",
        "command -range -addr=windows UcWins call add(g:uc, [<line1>, <line2>])",
        "command -range -addr=arguments UcArgs call add(g:uc, [<line1>, <line2>])",
        "command -range -addr=quickfix UcQf call add(g:uc, [<line1>, <line2>])",
    ] {
        fx.run(cmd);
    }
    fx.run("3");
    fx.check("[line('$'), line('.')]", "[5, 3]");
    // Each: the error it raised (`none`), then what it recorded.
    for (cmd, err, got) in [
        ("UcRange", "none", "[[3, 3, 0]]"),
        ("2,4UcRange", "none", "[[2, 4, 2]]"),
        ("%UcRange", "none", "[[1, 5, 2]]"),
        (".,$UcRange", "none", "[[3, 5, 2]]"),
        (
            "'<,'>UcRange",
            "E20: Mark not set: silent! ''<,''>UcRange",
            "[]",
        ),
        ("UcAll", "none", "[[1, 5, 0]]"),
        ("2UcAll", "none", "[[2, 2, 1]]"),
        ("UcDefCount", "none", "[[3, 3, 3, 0]]"),
        ("4UcDefCount", "none", "[[4, 4, 4, 1]]"),
        (
            "UcDefCount 2",
            "E488: Trailing characters: 2: silent! UcDefCount 2",
            "[]",
        ),
        ("UcCount", "none", "[[7, 3, 1, 0]]"),
        ("UcCount 3", "none", "[[3, 3, 3, 1]]"),
        ("2UcCount", "none", "[[2, 2, 2, 1]]"),
        ("UcCount0", "none", "[[0, 0]]"),
        ("UcBufs", "none", "[[1, 1, 1]]"),
        ("%UcBufs", "none", "[[1, 1, 1]]"),
        ("UcOther", "none", "[[3, 1]]"),
        ("5UcOther", "none", "[[5, 5]]"),
        ("UcTabs", "none", "[[0]]"),
        ("UcWins", "none", "[[1, 1]]"),
        ("UcArgs", "none", "[[0, 0]]"),
        ("UcQf", "E42: No Errors: silent! UcQf", "[]"),
    ] {
        fx.run("let g:uc = []");
        fx.check_error(cmd, err);
        fx.check("g:uc", got);
    }
    for (cmd, err) in [
        ("1,2UcCount", "none"),
        ("9UcRange", "E16: Invalid range: silent! 9UcRange"),
        (
            "UcCount x",
            "E488: Trailing characters: x: silent! UcCount x",
        ),
        ("2,1UcRange", "none"),
        ("2UcTabs", "E16: Invalid range: silent! 2UcTabs"),
    ] {
        fx.check_error(cmd, err);
    }
    fx.check_output("command UcR", "\n    Name              Args Address Complete    Definition\n    UcRange           0    .                   call add(g:uc, [<line1>, <line2>, <range>])");
    fx.run("silent! comclear | unlet g:uc");
    fx.home();
}
