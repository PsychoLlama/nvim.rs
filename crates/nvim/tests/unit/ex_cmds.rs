//! The Ex commands' refusals, each with every message it puts out and the
//! buffer it leaves: how `:s` reads its pattern, replacement, flags and
//! count (E10, E146, E476 after a pattern the regexp engine refused, E488,
//! E939, E1510, E21), `:&&`/`:~`, the `\n` join; `:sort`'s and `:uniq`'s
//! flags and pattern (E474, E475, E654); `:global`'s head (E10, E146,
//! E148); `:w`'s partial write (E140).
//!
//! Every row reads [`Probe::messages_of`], so a message reported twice, or
//! not at all, moves the row. Recorded before these commands' errors became
//! values reported by the command.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};

const LINES: &str = "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']";

fn reset(fx: &Probe) {
    fx.home();
    fx.run("set modifiable");
    fx.run(&format!("call setline(1, {LINES})"));
    fx.run("call cursor(1, 1)");
}

/// Runs each row from the same five lines, checking its messages and then
/// the buffer.
#[track_caller]
fn rows(fx: &Probe, cases: &[(&str, &str, &str)]) {
    for (cmd, want, lines) in cases {
        reset(fx);
        fx.check_messages(cmd, want);
        fx.check("getline(1, '$')", lines);
    }
}

#[test]
fn substitute_reads_its_arguments() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    rows(
        &fx,
        &[
            ("%s/a/x/", "none", "['b 2', 'x 10', 'c 1', 'x 10', 'x  3']"),
            ("%s/a/x/g", "none", "['b 2', 'x 10', 'c 1', 'x 10', 'x  3']"),
            (
                "s/a/x/ 0",
                "E939: Positive count required",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            (
                "s/a/x/ 2147483647",
                "E1510: Value too large: 2147483647",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            ("s/b/x/ 3", "none", "['x 2', 'a 10', 'c 1', 'a 10', 'x  3']"),
            (
                "s/a/x/ junk",
                "E488: Trailing characters: junk",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            (
                "s/b/x/ | s/2/y/",
                "none",
                "['x y', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            (
                "s/b/x/ \" comment",
                "none",
                "['x 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            (
                "s\\x",
                "E10: \\ should be followed by /, ? or &",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            (
                "sa/x/",
                "E163: There is only one file to edit",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            (
                "s/\\(/x/",
                "E54: Unmatched \\(\nE476: Invalid command",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            (
                "s/\\(/x/e",
                "E54: Unmatched \\(",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            (
                "s/zzz/x/",
                "E486: Pattern not found: zzz",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            (
                "s/zzz/x/e",
                "none",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            (
                "set nomodifiable | s/b/x/",
                "E21: Cannot make changes, 'modifiable' is off",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            (
                "set nomodifiable | s/b/x/n",
                "1 match on 1 line",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            ("1s/\\n//", "none", "['b 2a 10', 'c 1', 'a 10', 'x  3']"),
            ("1,3s/\\n/-/", "none", "['b 2-a 10-c 1-a 10', 'x  3']"),
            ("s#b#x#", "none", "['x 2', 'a 10', 'c 1', 'a 10', 'x  3']"),
            ("s?b?x?", "none", "['x 2', 'a 10', 'c 1', 'a 10', 'x  3']"),
            (
                "s/b/x/ | 2&&",
                "E486: Pattern not found: b",
                "['x 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            (
                "s/b/x/ | 2s",
                "E486: Pattern not found: b",
                "['x 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            (
                "s/b/x/ | 2~",
                "E486: Pattern not found: b",
                "['x 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            (
                "%s/a/\\=submatch(0) . 'q'/",
                "none",
                "['b 2', 'aq 10', 'c 1', 'aq 10', 'x  3']",
            ),
            (
                "%s/a\\|b/[&]/",
                "none",
                "['[b] 2', '[a] 10', 'c 1', '[a] 10', 'x  3']",
            ),
        ],
    );
}

#[test]
fn sort_and_uniq_read_their_flags() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    rows(
        &fx,
        &[
            ("sort", "none", "['a 10', 'a 10', 'b 2', 'c 1', 'x  3']"),
            ("sort!", "none", "['x  3', 'c 1', 'b 2', 'a 10', 'a 10']"),
            ("sort n", "none", "['c 1', 'b 2', 'x  3', 'a 10', 'a 10']"),
            (
                "sort nx",
                "E474: Invalid argument",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            (
                "sort fn",
                "E474: Invalid argument",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            ("sort u", "none", "['a 10', 'b 2', 'c 1', 'x  3']"),
            ("sort i", "none", "['a 10', 'a 10', 'b 2', 'c 1', 'x  3']"),
            (
                "sort z",
                "E475: Invalid argument: z",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            (
                "sort /a",
                "E654: Missing delimiter after search pattern: a",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            ("sort /a/", "none", "['b 2', 'c 1', 'x  3', 'a 10', 'a 10']"),
            (
                "sort /a/ r",
                "none",
                "['b 2', 'c 1', 'x  3', 'a 10', 'a 10']",
            ),
            (
                "sort /a/ /b/",
                "E475: Invalid argument: /b/",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            (
                "sort /\\(/",
                "E54: Unmatched \\(",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            (
                "sort | s/a/x/",
                "none",
                "['x 10', 'a 10', 'b 2', 'c 1', 'x  3']",
            ),
            (
                "sort n | s/a/x/",
                "E486: Pattern not found: a",
                "['c 1', 'b 2', 'x  3', 'a 10', 'a 10']",
            ),
            ("uniq", "none", "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']"),
            (
                "uniq z",
                "E475: Invalid argument: z",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            (
                "uniq /a",
                "E654: Missing delimiter after search pattern: a",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            (
                "uniq /\\(/",
                "E54: Unmatched \\(",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            ("uniq i", "none", "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']"),
            ("uniq u", "none", "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']"),
            (
                "uniq d",
                "E475: Invalid argument: d",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
        ],
    );
}

#[test]
fn global_reads_its_head() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    rows(
        &fx,
        &[
            (
                "g",
                "E148: Regular expression missing from global",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            (
                "g\\x",
                "E10: \\ should be followed by /, ? or &",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            (
                "ga",
                "E492: Not an editor command: silent ga",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            (
                "g/\\(/d",
                "E54: Unmatched \\(\nE476: Invalid command",
                "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']",
            ),
            ("g/a/d", "none", "['b 2', 'c 1', 'x  3']"),
            ("v/a/d", "none", "['a 10', 'a 10']"),
            ("g!/a/d", "none", "['a 10', 'a 10']"),
            ("g/a", "none", "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']"),
            ("g/zzz/d", "none", "['b 2', 'a 10', 'c 1', 'a 10', 'x  3']"),
            (
                "g/a/s/a/y/",
                "none",
                "['b 2', 'y 10', 'c 1', 'y 10', 'x  3']",
            ),
        ],
    );
}

#[test]
fn write_refuses_a_partial_buffer() {
    let sandbox = Sandbox::dir("ex_cmds_write");
    let fx = Probe::new(sandbox.editor());
    let file = sandbox.path("x.txt");
    let file = file.to_str().expect("a UTF-8 path");
    reset(&fx);
    fx.run(&format!("file {file}"));
    fx.check_messages("2,3w", "E140: Use ! to write partial buffer");
    fx.check("filereadable(expand('%'))", "0");
    fx.check_messages("2,3w!", "none");
    fx.check("readfile(expand('%'))", "['a 10', 'c 1']");
    fx.run("set nomodified");
    fx.home();
}

/// `:{range}!cmd` replaces the range with the command's output and `:r
/// !cmd` adds it, through temp files and through pipes alike: the lines,
/// the cursor, the `'[`/`']` marks and the report. `:w !cmd` leaves the
/// buffer alone and the cursor where it was.
#[test]
fn filter_runs_the_range_through_the_shell() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    let state = "[getline(1, '$'), line('.'), line(\"'[\"), line(\"']\")]";
    for transport in ["shelltemp", "noshelltemp"] {
        reset(&fx);
        fx.run(&format!("set {transport}"));
        fx.run("call cursor(5, 1)");
        fx.check_messages("2,4!sort", "none");
        fx.check(state, "[['b 2', 'a 10', 'a 10', 'c 1', 'x  3'], 2, 2, 4]");
        fx.check_messages("$r !echo tail", "none");
        fx.check(
            state,
            "[['b 2', 'a 10', 'a 10', 'c 1', 'x  3', 'tail'], 6, 6, 6]",
        );
        fx.run("call cursor(3, 2)");
        fx.check_messages("2,3w !cat >/dev/null", "none");
        fx.check("[line('.'), col('.')]", "[3, 2]");
        fx.check_messages("%!sort -r", "none");
        fx.check(
            state,
            "[['x  3', 'tail', 'c 1', 'b 2', 'a 10', 'a 10'], 1, 1, 6]",
        );
        fx.check("&modified", "1");
    }
    fx.run("set shelltemp&");
    fx.settle();
}
