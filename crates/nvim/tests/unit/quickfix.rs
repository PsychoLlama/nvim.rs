//! The quickfix family's refusals, each with every message it puts out:
//! a bad `'errorformat'` (E372–E378, and a pattern the regexp engine
//! rejects) through `:cexpr`/`:lgetexpr`/`:caddexpr`, `getqflist()` and
//! `setqflist()`'s `lines`; an errorfile that will not open (E40); the
//! stack ends (E380/E381); `setqflist()`'s silent `-1`s; and how `:cexpr`
//! reads its argument -- a String, a List, anything else (E777).
//!
//! Every row reads [`Probe::messages_of`], so a message reported twice, or
//! not at all, moves the row. Recorded before the family's errors became
//! values reported at the command or the builtin.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};

fn setup(fx: &Probe) {
    fx.home();
    fx.run("call setqflist([], 'f')");
    fx.run("call setloclist(0, [], 'f')");
    fx.run("set efm&");
}

#[test]
fn errorformat_refusals_report_once() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    setup(&fx);
    // Every bad format, through each way a list is read.
    for (i, (efm, want)) in [
        ("", "E378: 'errorformat' contains no pattern"),
        ("%f%f", "E372: Too many %f in format string"),
        ("%l:%l", "E372: Too many %l in format string"),
        ("%D%l", "E373: Unexpected %l in format string"),
        ("%r", "E373: Unexpected %r in format string"),
        ("%*x", "E375: Unsupported %x in format string"),
        ("%*[abc", "E374: Missing ] in format string"),
        ("a%Y", "E377: Invalid %Y in format string"),
        ("%Y", "E376: Invalid %Y in format string prefix"),
        ("%-Y", "E376: Invalid %Y in format string prefix"),
        ("%f:%l,%f%f", "E372: Too many %f in format string"),
        ("\\\\(", "E54: Unmatched \\("),
    ]
    .into_iter()
    .enumerate()
    {
        fx.run(&format!("let &efm = '{efm}'"));
        fx.check_messages("cexpr 'a.c:1:x'", want);
        fx.check_messages("lgetexpr 'a.c:1:x'", want);
        fx.check_messages("caddexpr 'a.c:1:x'", want);
        // Nothing of a refused read is kept -- but a full stack (ten
        // lists) has already dropped its oldest to make room.
        fx.check("getqflist({'nr': '$'}).nr", &i.min(9).to_string());
        fx.run("set efm&");
        fx.check_messages(
            &format!("let g:r = getqflist({{'lines': ['a.c:1:x'], 'efm': '{efm}'}})"),
            want,
        );
        fx.check("g:r", "{'items': []}");
        // `setqflist()`'s `' '` makes the new list before it reads.
        fx.check_messages(
            &format!("let g:r = setqflist([], ' ', {{'lines': ['a.c:1:x'], 'efm': '{efm}'}})"),
            want,
        );
        fx.check("g:r", "-1");
    }
    fx.check("getqflist({'nr': '$'}).nr", "10");
}

#[test]
fn errorfile_and_stack_refusals() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    setup(&fx);
    let e40 = "E40: Can't open errorfile /nonexistent/errors.err";
    let bottom = "E380: At bottom of quickfix stack";
    let top = "E381: At top of quickfix stack";
    let two = "error list 1 of 2; 2 errors       :silent cexpr ['a.c:1:x', 'b.c:2:y']";
    let one = "error list 2 of 2; 1 errors       :silent cexpr ['c.c:3:z']";
    for (cmd, want) in [
        ("cfile /nonexistent/errors.err", e40.to_string()),
        ("cgetfile /nonexistent/errors.err", e40.into()),
        ("laddfile /nonexistent/errors.err", e40.into()),
        ("colder", format!("{bottom}\nerror list 1 of 0; 0 errors ")),
        ("cnewer", format!("{top}\nerror list 1 of 0; 0 errors ")),
        ("cexpr ['a.c:1:x', 'b.c:2:y']", "none".into()),
        ("cexpr ['c.c:3:z']", "none".into()),
        ("colder", "none".into()),
        ("colder", format!("{bottom}\n{two}")),
        ("cnewer 5", format!("{top}\n{one}")),
        ("cnewer", format!("{top}\n{one}")),
    ] {
        fx.check_messages(cmd, &want);
    }
    fx.check(
        "getqflist({'nr': 0, 'size': 0, 'title': 0})",
        "{'nr': 2, 'title': ':silent cexpr [''c.c:3:z'']', 'size': 1}",
    );
}

#[test]
fn setqflist_answers() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    setup(&fx);
    for (call, want, answer) in [
        ("setqflist(1)", "E714: List required", "-1"),
        ("setqflist([], 'x')", "E927: Invalid action: 'x'", "-1"),
        ("setqflist([], 'xy')", "E927: Invalid action: 'xy'", "-1"),
        ("setqflist([], ' ', 3)", "E715: Dictionary required", "-1"),
        ("setqflist([], ' ', {'nr': 99})", "none", "-1"),
        ("setqflist([], ' ', {'title': []})", "none", "-1"),
        ("setqflist([], ' ', {'idx': 'x'})", "none", "-1"),
        ("setqflist([], ' ', {'lines': 1})", "none", "-1"),
        ("setqflist([], ' ', {'items': 1})", "none", "-1"),
        ("setqflist([], ' ', {'nosuch': 1})", "none", "-1"),
        ("setqflist([], ' ', {'id': 12345})", "none", "-1"),
        (
            "setqflist([{'filename': 'a.c', 'lnum': 1}], ' ', {'title': 'T'})",
            "E475: Invalid argument: cannot have both a list and a \"what\" argument",
            "-1",
        ),
        ("setqflist([], 'a', {'idx': 5})", "none", "-1"),
        ("setqflist([], 'r', {'context': [1]})", "none", "0"),
        ("setloclist(99, [])", "none", "-1"),
        ("getqflist({'nr': 99})", "none", "{'nr': 0}"),
        (
            "getqflist({'id': 12345, 'all': 1}) == {'id': 0}",
            "none",
            "0",
        ),
        ("getqflist({'lines': 1})", "none", "{}"),
        ("getqflist({'lines': ['a.c:1:x'], 'efm': 1})", "none", "{}"),
        ("getloclist(99, {'all': 1})", "none", "{}"),
    ] {
        fx.check_messages(&format!("let g:r = {call}"), want);
        fx.check("g:r", answer);
    }
}

#[test]
fn cexpr_reads_its_argument() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    setup(&fx);
    fx.run("let &efm = '%f:%l:%m'");
    let d4 = "[['d.c', 4, 'four', 1]]";
    for (cmd, want, list) in [
        ("cexpr 'a.c:1:one'", "none", "[['a.c', 1, 'one', 1]]"),
        (
            "cexpr \"a.c:1:one\\nb.c:2:two\\nnoise\"",
            "none",
            "[['a.c', 1, 'one', 1], ['b.c', 2, 'two', 1], ['', 0, 'noise', 0]]",
        ),
        (
            "cexpr ['a.c:1:one', 'b.c:2:two']",
            "none",
            "[['a.c', 1, 'one', 1], ['b.c', 2, 'two', 1]]",
        ),
        ("cexpr []", "none", "[]"),
        (
            "caddexpr ['c.c:3:three']",
            "none",
            "[['c.c', 3, 'three', 1]]",
        ),
        ("cgetexpr 'd.c:4:four'", "none", d4),
        ("cexpr 1", "E777: String or List expected", d4),
        ("cexpr {}", "E777: String or List expected", d4),
        ("cexpr v:null", "E777: String or List expected", d4),
        ("cexpr 1 +", "E15: Invalid expression: \"1 +\"", d4),
        ("cexpr", "E471: Argument required: silent cexpr", d4),
        ("lexpr ['e.c:5:five']", "none", d4),
        ("laddexpr 2", "E777: String or List expected", d4),
    ] {
        fx.check_messages(cmd, want);
        fx.check(
            "map(getqflist(), {_, e -> [bufname(e.bufnr), e.lnum, e.text, e.valid]})",
            list,
        );
    }
    fx.check(
        "getqflist({'nr': '$', 'title': 0})",
        "{'nr': 5, 'title': ':silent cgetexpr ''d.c:4:four'''}",
    );
    fx.check(
        "getloclist(0, {'nr': '$', 'title': 0, 'size': 0})",
        "{'nr': 1, 'title': ':silent lexpr [''e.c:5:five'']', 'size': 1}",
    );
    fx.run("set efm&");
}
