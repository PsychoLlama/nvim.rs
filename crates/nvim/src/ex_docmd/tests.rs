//! The Ex loop end to end, through the sources it reads its lines from: a
//! function body, a sourced script, `execute()`'s list, an autocommand's
//! command list and a line given whole. What each case pins is where a line
//! comes from and what is replayed: the body of a `:while`/`:for` stored on
//! the first pass and replayed on the rest, at depth and across a `:function`
//! defined inside it; what each source answers when a command asks which kind
//! it is (`:finish`, a missing `:endif`, `:loadkeymap`). The breakpoints a
//! function or a file stops at need the debugger's prompt, which needs a
//! screen: `test_debugger.vim` has those.
//!
//! Each case records what ran in `g:log` and reads it back; the functions it
//! defines are `g:Ed*`, deleted again at the end.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::eval::test_fixture::{Fixture, Scratch};

/// Run `lines` as one block (through `execute()`'s list) and answer
/// `g:log` as `string()` renders it.
fn logged(fx: &Fixture, lines: &[&str]) -> String {
    fx.run("let g:log = []");
    fx.block(lines);
    fx.eval("g:log")
}

/// Define `g:EdBody` as `lines`, call it, and answer `g:log`.
fn logged_in_function(fx: &Fixture, lines: &[&str]) -> String {
    let mut body = vec!["function! g:EdBody()"];
    body.extend_from_slice(lines);
    body.push("endfunction");
    fx.block(&body);
    fx.run("let g:log = []");
    fx.run("call g:EdBody()");
    let log = fx.eval("g:log");
    fx.delete(&["g:EdBody"]);
    log
}

/// Source `lines` as a script file and answer `g:log`.
fn logged_sourced(fx: &Fixture, scratch: &Scratch, lines: &[&str]) -> String {
    let mut text = lines.join("\n");
    text.push('\n');
    let script = scratch.file("body.vim", &text);
    fx.run("let g:log = []");
    fx.run(&format!("source {script}"));
    fx.eval("g:log")
}

/// Loops nested four deep, mixing `:while` and `:for`, with `:continue` and
/// `:break` at the inner levels: every stored body is replayed from the right
/// line, and an inner loop's store does not swallow the outer one's.
const NESTED: &[&str] = &[
    "let i = 0",
    "while i < 2",
    "  let i += 1",
    "  for j in [1, 2]",
    "    let k = 0",
    "    while k < 3",
    "      let k += 1",
    "      if k == 2",
    "        continue",
    "      endif",
    "      for m in ['a', 'b', 'c']",
    "        if m == 'c'",
    "          break",
    "        endif",
    "        call add(g:log, i . j . k . m)",
    "      endfor",
    "    endwhile",
    "  endfor",
    "endwhile",
];

const NESTED_LOG: &str = "['111a', '111b', '113a', '113b', '121a', '121b', '123a', '123b', \
                          '211a', '211b', '213a', '213b', '221a', '221b', '223a', '223b']";

#[test]
fn a_loop_body_is_replayed_at_every_depth() {
    let fx = Fixture::new();
    assert_eq!(logged(&fx, NESTED), NESTED_LOG);
    assert_eq!(logged_in_function(&fx, NESTED), NESTED_LOG);
}

#[test]
#[cfg_attr(miri, ignore = "`:source` resolves its file through libuv")]
fn a_sourced_loop_body_is_replayed_at_every_depth() {
    let fx = Fixture::new();
    let scratch = Scratch::new("edloop");
    assert_eq!(logged_sourced(&fx, &scratch, NESTED), NESTED_LOG);
}

/// A `|`-joined loop on one line: each command is stored on its own, so the
/// `:endwhile` jumps back to the one after the `:while`. The command that
/// runs is handed a copy, which it cuts at its `|` -- the stored line must
/// still be whole on the next pass.
const ONE_LINE: &[&str] = &[
    "let i = 0 | while i < 3 | let i += 1 | call add(g:log, i) | endwhile | call add(g:log, 'end')",
];

#[test]
fn a_bar_joined_loop_replays_each_command_whole() {
    let fx = Fixture::new();
    assert_eq!(logged(&fx, ONE_LINE), "[1, 2, 3, 'end']");
    assert_eq!(logged_in_function(&fx, ONE_LINE), "[1, 2, 3, 'end']");
    // A command that rewrites the line it was handed -- `:execute` builds its
    // own, a heredoc terminates the line at its marker -- leaves the stored
    // copy alone.
    let rewrites = [
        "let i = 0",
        "while i < 2",
        "  let i += 1",
        "  let text =<< trim END",
        "    one",
        "    two",
        "  END",
        "  call add(g:log, i . join(text))",
        "  execute 'call add(g:log, ' . i . ')' | call add(g:log, 'after')",
        "endwhile",
    ];
    assert_eq!(
        logged_in_function(&fx, &rewrites),
        "['1one two', 1, 'after', '2one two', 2, 'after']"
    );
}

/// A `:function` defined inside a loop reads its body through the loop's
/// store: on the first pass from the source underneath, on the next ones by
/// replay.
#[test]
fn a_function_defined_in_a_loop_reads_its_body_through_the_store() {
    let fx = Fixture::new();
    let lines = [
        "for n in [1, 2]",
        "  function! g:EdInner()",
        "    return 'inner'",
        "  endfunction",
        "  call add(g:log, n . g:EdInner())",
        "endfor",
    ];
    assert_eq!(logged(&fx, &lines), "['1inner', '2inner']");
    assert_eq!(logged_in_function(&fx, &lines), "['1inner', '2inner']");
    fx.delete(&["g:EdInner"]);
}

/// The line numbers a replayed line reports are the ones it was read at.
#[test]
fn a_replayed_line_reports_the_line_it_was_read_at() {
    let fx = Fixture::new();
    let lines = [
        "for n in [1, 2]",
        "  call add(g:log, expand('<slnum>'))",
        "  while 0",
        "  endwhile",
        "  call add(g:log, expand('<slnum>'))",
        "endfor",
    ];
    assert_eq!(logged_in_function(&fx, &lines), "['2', '5', '2', '5']");
}

#[test]
#[cfg_attr(miri, ignore = "`:source` resolves its file through libuv")]
fn finish_is_a_sourced_scripts_and_reaches_out_of_its_loops() {
    let fx = Fixture::new();
    let scratch = Scratch::new("edfinish");
    let lines = [
        "let i = 0",
        "while 1",
        "  let i += 1",
        "  for x in [1]",
        "    if i == 2",
        "      finish",
        "    endif",
        "  endfor",
        "  call add(g:log, i)",
        "endwhile",
        "call add(g:log, 'not reached')",
    ];
    assert_eq!(logged_sourced(&fx, &scratch, &lines), "[1]");
    // A `:finish` run by `:execute` in the script is the script's too.
    let lines = [
        "call add(g:log, 1)",
        "execute 'finish'",
        "call add(g:log, 2)",
    ];
    assert_eq!(logged_sourced(&fx, &scratch, &lines), "[1]");
    // ... and so is one inside a loop inside an `:execute`.
    let lines = [
        "call add(g:log, 1)",
        "execute \"while 1\\nfinish\\nendwhile\"",
        "call add(g:log, 2)",
    ];
    assert_eq!(logged_sourced(&fx, &scratch, &lines), "[1]");
    // Inside a loop, a missing :endwhile is not reported after a :finish.
    let lines = ["while 1", "  finish"];
    fx.run("let v:errmsg = ''");
    assert_eq!(logged_sourced(&fx, &scratch, &lines), "[]");
    fx.check("v:errmsg", "''");
}

#[test]
fn which_source_a_command_reads_from_is_what_it_is_allowed() {
    let fx = Fixture::new();
    // Not a sourced file: a function body, `execute()`'s list and a line
    // given whole are none of them.
    fx.check_error("finish", "E168: :finish used outside of a sourced file");
    fx.check_error(
        "call execute(['for x in [1]', 'finish', 'endfor'])",
        "E168: :finish used outside of a sourced file",
    );
    fx.check_error(
        "scriptencoding utf-8",
        "E167: :scriptencoding used outside of a sourced file",
    );
    fx.check_error(
        "loadkeymap",
        "E105: Using :loadkeymap not in a sourced file",
    );
    fx.block(&[
        "function! g:EdFinish()",
        "  for x in [1]",
        "    finish",
        "  endfor",
        "endfunction",
    ]);
    fx.check_error(
        "call g:EdFinish()",
        "E168: :finish used outside of a sourced file",
    );
    fx.delete(&["g:EdFinish"]);
    // A function body that ends inside a loop reports it; `execute()`'s list
    // does not, since it is neither a function nor a script.
    fx.block(&["function! g:EdOpen()", "  while 0", "endfunction"]);
    fx.check_error("call g:EdOpen()", "E170: Missing :endwhile");
    fx.delete(&["g:EdOpen"]);
    fx.check_error("call execute(['while 0'])", "none");
}

#[test]
#[cfg_attr(miri, ignore = "`:source` resolves its file through libuv")]
fn a_sourced_script_is_a_script_inside_its_loops() {
    let fx = Fixture::new();
    let scratch = Scratch::new("edscript");
    // `:scriptencoding` in a loop is the script's; `:loadkeymap` reads the
    // rest of the file from inside one.
    let lines = [
        "for x in [1, 2]",
        "  scriptencoding utf-8",
        "  call add(g:log, x)",
        "endfor",
    ];
    fx.run("let v:errmsg = ''");
    assert_eq!(logged_sourced(&fx, &scratch, &lines), "[1, 2]");
    fx.check("v:errmsg", "''");
    // A function defined in a script reports its body's lines from the file.
    let lines = [
        "function! g:EdLines()",
        "  return expand('<sflnum>')",
        "endfunction",
        "call add(g:log, g:EdLines())",
    ];
    assert_eq!(logged_sourced(&fx, &scratch, &lines), "['2']");
    fx.delete(&["g:EdLines"]);
}

/// An autocommand's commands are read one handler at a time; a `|` joins
/// commands in one handler, and a handler may run a multi-line `:execute`.
#[test]
fn an_autocommand_runs_each_handler_and_every_command_in_it() {
    let fx = Fixture::new();
    fx.run("augroup EdAu | autocmd! | augroup END");
    fx.run("autocmd EdAu User EdEvent call add(g:log, 'a') | call add(g:log, 'b')");
    fx.run("autocmd EdAu User EdEvent let i = 0 | while i < 2 | let i += 1 | call add(g:log, i) | endwhile");
    fx.run(
        "autocmd EdAu User EdEvent execute \"call add(g:log, 'c')\\nfor x in [1, 2]\\ncall add(g:log, x)\\nendfor\"",
    );
    fx.run("autocmd EdAu User EdEvent call add(g:log, expand('<amatch>'))");
    fx.run("let g:log = []");
    fx.run("doautocmd EdAu User EdEvent");
    fx.check("g:log", "['a', 'b', 1, 2, 'c', 1, 2, 'EdEvent']");
    // In a function, the autocommand fired from a loop runs once per pass.
    let lines = ["for n in [1, 2]", "  doautocmd EdAu User EdEvent", "endfor"];
    assert_eq!(
        logged_in_function(&fx, &lines),
        "['a', 'b', 1, 2, 'c', 1, 2, 'EdEvent', 'a', 'b', 1, 2, 'c', 1, 2, 'EdEvent']"
    );
    fx.run("autocmd! EdAu");
    fx.run("augroup! EdAu");
}
