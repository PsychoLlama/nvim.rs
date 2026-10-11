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
}

/// A command that rewrites the line it was handed -- `:execute` builds its
/// own, a heredoc terminates the line at its marker -- leaves the stored copy
/// alone.
#[test]
#[cfg_attr(
    miri,
    ignore = "a heredoc's marker is classified through libc's ctype table"
)]
fn a_command_that_rewrites_its_line_leaves_the_stored_copy_alone() {
    let fx = Fixture::new();
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
#[cfg_attr(miri, ignore = "an event name is looked up with libc's `strcasecmp`")]
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

/// What `getline_equal` used to answer -- is this a function body, a sourced
/// script, the `:` prompt? -- is `kind()`, and it sees through every loop
/// wrapped around the source; `is_loop()` is the one question that does not.
#[test]
fn a_sources_kind_is_seen_through_the_loops_around_it() {
    use crate::ex_docmd::{LineSource, LoopLines, ReadLines, SourceKind};
    use crate::runtime::SourceCookie;

    for (mut base, kind) in [
        (LineSource::None, SourceKind::None),
        (LineSource::Typed, SourceKind::Typed),
        (LineSource::CmdKey, SourceKind::CmdKey),
    ] {
        assert_eq!(base.kind(), kind);
        assert!(!base.is_loop());
        let (mut outer, mut inner) = (Vec::new(), Vec::new());
        let (mut at_outer, mut at_inner) = (0, 0);
        let mut once = LineSource::Loop(LoopLines {
            store: &mut outer,
            current_line: &mut at_outer,
            repeating: false,
            inner: &mut base,
        });
        assert_eq!(once.kind(), kind);
        assert!(once.is_loop());
        let twice = LineSource::Loop(LoopLines {
            store: &mut inner,
            current_line: &mut at_inner,
            repeating: false,
            inner: &mut once,
        });
        assert_eq!(twice.kind(), kind);
        assert!(twice.function().is_none() && twice.script().is_none());
    }

    // A script is a script through two loops, and what is written through
    // them is the cookie's.
    let mut cookie = SourceCookie::new();
    cookie.level = 7;
    let mut base = LineSource::Script(&mut cookie);
    let (mut outer, mut inner) = (Vec::new(), Vec::new());
    let (mut at_outer, mut at_inner) = (0, 0);
    let mut once = LineSource::Loop(LoopLines {
        store: &mut outer,
        current_line: &mut at_outer,
        repeating: false,
        inner: &mut base,
    });
    let mut twice = LineSource::Loop(LoopLines {
        store: &mut inner,
        current_line: &mut at_inner,
        repeating: false,
        inner: &mut once,
    });
    assert_eq!(twice.kind(), SourceKind::Script);
    assert_eq!(twice.script().map(|cookie| cookie.level), Some(7));
    twice.script_mut().expect("a script").finished = true;
    let pos = twice.debug().expect("a script has a breakpoint");
    assert_eq!(pos.name, c"");
    pos.breakpoint.set(12);
    pos.tick.set(3);
    assert!(cookie.finished);
    assert_eq!((cookie.breakpoint, cookie.dbg_tick), (12, 3));
}

/// The loop store, driven directly: a line past the end is read from the
/// source underneath and stored; a stored line is handed out as a copy with
/// the line number it was read at; past the end of a replay is the end of
/// the body. Nested, the inner store reads through the outer one.
#[test]
#[cfg_attr(miri, ignore = "reading a script line checks for CTRL-C through libuv")]
fn the_loop_store_reads_through_stores_and_replays_at_depth() {
    use crate::ex_docmd::{LineSource, LoopLines, ReadLines, StoredLine};
    use crate::runtime::SourceCookie;
    use std::ffi::CString;

    let _fx = Fixture::new();
    // A frame of the case's own: reading and replaying move the innermost
    // frame's line number, which the next case would otherwise inherit.
    let name = c"edstore";
    crate::runtime::estack_push(crate::runtime::ETYPE_SCRIPT, name.as_ptr().cast_mut(), 0);
    let mut cookie = SourceCookie::new();
    cookie.source_from_buf_or_str = true;
    cookie.buflines = ["one", "two", "three"]
        .into_iter()
        .map(|line| CString::new(line).expect("no NUL"))
        .collect();
    let mut base = LineSource::Script(&mut cookie);
    let mut outer: Vec<StoredLine> = Vec::new();
    let mut inner: Vec<StoredLine> = Vec::new();
    let (mut at_outer, mut at_inner) = (-1, -1);
    {
        let mut once = LineSource::Loop(LoopLines {
            store: &mut outer,
            current_line: &mut at_outer,
            repeating: false,
            inner: &mut base,
        });
        let mut twice = LineSource::Loop(LoopLines {
            store: &mut inner,
            current_line: &mut at_inner,
            repeating: false,
            inner: &mut once,
        });
        let read = |source: &mut dyn ReadLines| {
            source
                .next_line(0, 0, false)
                .map(|line| String::from_utf8_lossy(&line).into_owned())
        };
        assert_eq!(read(&mut twice).as_deref(), Some("one"));
        assert_eq!(read(&mut twice).as_deref(), Some("two"));
    }
    // Both stores kept both lines, with the lines they were read at.
    let lines = |store: &[StoredLine]| -> Vec<(String, i64)> {
        store
            .iter()
            .map(|stored| {
                (
                    String::from_utf8_lossy(&stored.line).into_owned(),
                    i64::from(stored.lnum),
                )
            })
            .collect()
    };
    assert_eq!(lines(&inner), [("one".into(), 1), ("two".into(), 2)]);
    assert_eq!(lines(&outer), lines(&inner));
    assert_eq!((at_outer, at_inner), (1, 1));

    // A replay of the inner store from its start: the copies come back, and
    // a command that cuts the line it was handed leaves the stored one whole.
    at_inner = -1;
    {
        let mut once = LineSource::Loop(LoopLines {
            store: &mut outer,
            current_line: &mut at_outer,
            repeating: true,
            inner: &mut base,
        });
        let mut twice = LineSource::Loop(LoopLines {
            store: &mut inner,
            current_line: &mut at_inner,
            repeating: true,
            inner: &mut once,
        });
        let mut first = twice
            .next_line(0, 0, false)
            .expect("a stored line")
            .into_vec();
        first[0] = 0;
        assert_eq!(twice.next_line(0, 0, false).as_deref(), Some(&b"two"[..]));
        // Replaying, past the end is the end of the body.
        assert!(twice.next_line(0, 0, false).is_none());
    }
    assert_eq!(lines(&inner)[0].0, "one");
    // The script itself is still at its second line.
    assert_eq!(cookie.sourcing_lnum, 2);
    crate::runtime::estack_pop();
}

/// The items that name where a line came from, asked from every kind of
/// source: `expand()` (the leading item alone) and a `-complete=file` user
/// command (the whole argument, through the command line's expansion), plus
/// the `<sfile>` an autocommand defined in a script has replaced at definition
/// time. Where there is no script, the user command's `<sfile>` is an error
/// and adds nothing. Scratch paths are written as `<S>` (its last component
/// `<s>`) and script numbers as `N`.
#[test]
#[cfg_attr(miri, ignore = "`:source` resolves its file through libuv")]
fn where_a_line_came_from_expands_in_every_source() {
    let fx = Fixture::new();
    let scratch = Scratch::new("edwhere");
    fx.run("command! -nargs=* -complete=file EdWhere call add(g:log, <q-args>)");
    let probe = [
        "call add(g:log, [expand('<sfile>'), expand('<slnum>'), expand('<stack>'), expand('<script>'), expand('<sflnum>')])",
        "EdWhere <sfile>:t <slnum> <sflnum> x<script>:t:r",
        "call add(g:log, expand('<sfile>:p:h:t') . '|' . expand('<stack>:t'))",
    ];
    let norm = |text: String| {
        let dir = scratch.0.to_str().expect("a UTF-8 path").to_owned();
        let base = scratch
            .0
            .file_name()
            .and_then(|n| n.to_str())
            .expect("a name");
        let mut out = text.replace(&dir, "<S>").replace(base, "<s>");
        while let Some(at) = out.find("<SNR>") {
            let digits = out[at + 5..].bytes().take_while(u8::is_ascii_digit).count();
            if digits == 0 {
                break;
            }
            out.replace_range(at + 5..at + 5 + digits, "N");
        }
        out
    };
    // Given whole, and through `execute()`'s list.
    let mut got = Vec::new();
    fx.run("let g:log = []");
    for line in probe {
        fx.run(line);
    }
    got.push(norm(fx.eval("g:log")));
    got.push(norm(logged(&fx, &probe)));
    // A function body, and a loop inside one.
    got.push(norm(logged_in_function(&fx, &probe)));
    let mut looped = vec!["for n in [1, 2]"];
    looped.extend_from_slice(&probe);
    looped.push("endfor");
    got.push(norm(logged_in_function(&fx, &looped)));
    // A sourced script, a loop in one, and a function it defines.
    got.push(norm(logged_sourced(&fx, &scratch, &probe)));
    got.push(norm(logged_sourced(&fx, &scratch, &looped)));
    let mut defined = vec!["function! g:EdWhereFn()"];
    defined.extend_from_slice(&probe);
    defined.extend_from_slice(&["endfunction", "call g:EdWhereFn()"]);
    got.push(norm(logged_sourced(&fx, &scratch, &defined)));
    fx.delete(&["g:EdWhereFn"]);
    // An autocommand defined by a script: its `<sfile>` is the script's,
    // replaced when the autocommand was defined.
    let au = [
        "augroup EdWhereAu | autocmd! | augroup END",
        "autocmd EdWhereAu User EdWhere call add(g:log, '<sfile>:t') | call add(g:log, expand('<sfile>'))",
        "autocmd EdWhereAu User EdWhere EdWhere <sfile>:t <amatch> <afile>",
    ];
    logged_sourced(&fx, &scratch, &au);
    fx.run("let g:log = []");
    fx.run("doautocmd EdWhereAu User EdWhere");
    got.push(norm(fx.eval("g:log")));
    fx.run("autocmd! EdWhereAu");
    fx.run("augroup! EdWhereAu");
    fx.run("delcommand EdWhere");
    let want = [
        "[['', '', '', '', ''], '|']",
        "[['', '', '', '', ''], '|']",
        "[['function EdBody', '1', 'function EdBody[1]', '', '1'], 'nvim|function EdBody[3]']",
        "[['function EdBody', '2', 'function EdBody[2]', '', '2'], 'nvim|function EdBody[4]', \
         ['function EdBody', '2', 'function EdBody[2]', '', '2'], 'nvim|function EdBody[4]']",
        "[['<S>/body.vim', '1', '<S>/body.vim[1]', '<S>/body.vim', '1'], 'body.vim 2 2 xbody', \
         '<s>|body.vim[3]']",
        "[['<S>/body.vim', '2', '<S>/body.vim[2]', '<S>/body.vim', '2'], 'body.vim 3 3 xbody', \
         '<s>|body.vim[4]', ['<S>/body.vim', '2', '<S>/body.vim[2]', '<S>/body.vim', '2'], \
         'body.vim 3 3 xbody', '<s>|body.vim[4]']",
        "[['<S>/body.vim[6]..function EdWhereFn', '1', '<S>/body.vim[6]..function EdWhereFn[1]', \
         '<S>/body.vim', '2'], 'body.vim[6]..function\\ EdWhereFn 2 3 xbody', \
         '<s>|body.vim[6]..function EdWhereFn[3]']",
        "['body.vim', '<S>/body.vim', 'body.vim EdWhere EdWhere']",
    ];
    assert_eq!(got, want);
}

/// `eval_vars` over the answers that need no file, no script and no
/// cursor: what is not an item, a quoted item, and the items that fail
/// with nothing to name. Runs under Miri, unlike the sourcing cases above.
#[test]
fn an_item_with_nothing_to_stand_for_says_why() {
    use crate::ex_docmd::{Expansion, eval_vars, expand_sfile};
    let _fx = Fixture::new();
    let failed = |text: &std::ffi::CStr, at: usize| match eval_vars(text, at, None, true) {
        Expansion::Refused(msg) => msg.to_string_lossy().into_owned(),
        other => format!("{other:?}"),
    };
    assert_eq!(eval_vars(c"abc", 0, None, true), Expansion::Skip(1));
    assert_eq!(eval_vars(c"<nosuch>", 0, None, true), Expansion::Skip(1));
    assert_eq!(eval_vars(c"\\%", 1, None, true), Expansion::Quoted);
    assert_eq!(eval_vars(c"\\%", 0, None, true), Expansion::Skip(1));
    // The bare buffer has no name: `%` needs `:p:h` to be anything.
    assert!(failed(c"%", 0).starts_with("E499:"), "{}", failed(c"%", 0));
    assert_eq!(eval_vars(c"%", 0, None, false), Expansion::Skip(1));
    assert_eq!(eval_vars(c"%:t", 0, None, false), Expansion::Skip(3));
    assert!(failed(c"#9999", 0).starts_with("E194:"));
    assert!(failed(c"<abuf>", 0).starts_with("E496:"));
    assert!(failed(c"<afile>", 0).starts_with("E495:"));
    assert!(failed(c"<amatch>", 0).starts_with("E497:"));
    assert!(failed(c"<sfile>", 0).starts_with("E498:"));
    // `<SID>` outside a script.
    assert!(
        failed(c"<SID>", 0).starts_with("E81:"),
        "{}",
        failed(c"<SID>", 0)
    );
    // `<sfile>` in an autocommand's command, read outside any script: the
    // quoted one stays, without its backslash; the bare one fails.
    assert_eq!(expand_sfile(b"echo 1"), Some(b"echo 1".to_vec()));
    assert_eq!(
        expand_sfile(b"x \\<sfile> y"),
        Some(b"x <sfile> y".to_vec())
    );
    assert_eq!(expand_sfile(b"x <sfile>"), None);
}
