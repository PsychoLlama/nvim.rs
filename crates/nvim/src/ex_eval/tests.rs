//! The condition stack and the exception machinery end to end, driven by the
//! commands that reach them: `:try` nested with a throw in each clause, a
//! `:finally` run on the way out of every kind of exit, rethrow, the values
//! `v:exception`/`v:throwpoint` take, what 'verbose' says an exception went
//! through, the loops nested to the stack's cap, and the error each command
//! gives out of place.
//!
//! Each case records what ran in `g:log` and reads it back; the functions it
//! defines are `g:Ee*`, deleted again at the end.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::eval::test_fixture::Fixture;
use crate::option::vars::P_VERBOSE;

/// Run `lines` as one block and answer `g:log` as `string()` renders it.
fn logged(fx: &Fixture, lines: &[&str]) -> String {
    fx.run("let g:log = []");
    fx.block(lines);
    fx.eval("g:log")
}

/// The error number `lines` end on, read off `v:errmsg`, or `none`.
fn error_after(fx: &Fixture, lines: &[&str]) -> String {
    fx.run("let v:errmsg = ''");
    fx.block(lines);
    let err = fx.eval("v:errmsg[: stridx(v:errmsg, ':') - 1]");
    if err == "''" {
        "none".into()
    } else {
        err.trim_matches('\'').into()
    }
}

#[test]
fn a_throw_in_each_clause_of_nested_tries() {
    let fx = Fixture::new();
    // In the try block: caught by the inner :catch, the finallys run.
    let log = logged(
        &fx,
        &[
            "try",
            "  try",
            "    call add(g:log, 'try')",
            "    throw 'one'",
            "    call add(g:log, 'not reached')",
            "  catch /one/",
            "    call add(g:log, 'caught ' . v:exception)",
            "  finally",
            "    call add(g:log, 'inner finally')",
            "  endtry",
            "finally",
            "  call add(g:log, 'outer finally')",
            "endtry",
        ],
    );
    assert_eq!(
        log,
        "['try', 'caught one', 'inner finally', 'outer finally']"
    );
    // In the catch clause: the inner finally runs, the outer catch takes it.
    let log = logged(
        &fx,
        &[
            "try",
            "  try",
            "    throw 'one'",
            "  catch",
            "    call add(g:log, 'inner ' . v:exception)",
            "    throw 'two'",
            "  finally",
            "    call add(g:log, 'inner finally ' . v:exception)",
            "  endtry",
            "catch",
            "  call add(g:log, 'outer ' . v:exception)",
            "endtry",
            "call add(g:log, 'after [' . v:exception . ']')",
        ],
    );
    assert_eq!(
        log,
        "['inner one', 'inner finally ', 'outer two', 'after []']"
    );
    // In the finally clause: it replaces the pending exception.
    let log = logged(
        &fx,
        &[
            "try",
            "  try",
            "    throw 'one'",
            "  finally",
            "    call add(g:log, 'finally')",
            "    throw 'two'",
            "  endtry",
            "catch",
            "  call add(g:log, 'outer ' . v:exception)",
            "endtry",
        ],
    );
    assert_eq!(log, "['finally', 'outer two']");
    // A catch that does not match passes it on; the first matching one of
    // several takes it.
    let log = logged(
        &fx,
        &[
            "try",
            "  try",
            "    throw 'abc'",
            "  catch /x/",
            "    call add(g:log, 'x')",
            "  catch /b/",
            "    call add(g:log, 'b')",
            "  catch /a/",
            "    call add(g:log, 'a')",
            "  endtry",
            "catch",
            "  call add(g:log, 'outer')",
            "endtry",
        ],
    );
    assert_eq!(log, "['b']");
}

#[test]
fn finally_runs_on_every_way_out() {
    let fx = Fixture::new();
    fx.block(&[
        "function! g:EeReturn()",
        "  try",
        "    return 'from try'",
        "  finally",
        "    call add(g:log, 'finally')",
        "  endtry",
        "  return 'not reached'",
        "endfunction",
        "function! g:EeOverride()",
        "  try",
        "    return 1",
        "  finally",
        "    return 2",
        "  endtry",
        "endfunction",
        "function! g:EeReturnInLoop()",
        "  for i in [1, 2, 3]",
        "    try",
        "      if i == 2",
        "        return i * 10",
        "      endif",
        "    finally",
        "      call add(g:log, 'f' . i)",
        "    endtry",
        "  endfor",
        "endfunction",
    ]);
    fx.run("let g:log = []");
    assert_eq!(fx.eval("g:EeReturn()"), "'from try'");
    assert_eq!(fx.eval("g:log"), "['finally']");
    assert_eq!(fx.eval("g:EeOverride()"), "2");
    fx.run("let g:log = []");
    assert_eq!(fx.eval("g:EeReturnInLoop()"), "20");
    assert_eq!(fx.eval("g:log"), "['f1', 'f2']");

    let log = logged(
        &fx,
        &[
            "let i = 0",
            "while i < 4",
            "  let i += 1",
            "  try",
            "    if i == 1",
            "      continue",
            "    elseif i == 3",
            "      break",
            "    endif",
            "    call add(g:log, 'body ' . i)",
            "  finally",
            "    call add(g:log, 'finally ' . i)",
            "  endtry",
            "endwhile",
            "call add(g:log, 'end ' . i)",
        ],
    );
    assert_eq!(
        log,
        "['finally 1', 'body 2', 'finally 2', 'finally 3', 'end 3']"
    );

    // An error becomes an exception the finally clause sees go by.
    let log = logged(
        &fx,
        &[
            "try",
            "  try",
            "    call add(g:log, 'before')",
            "    call EeNoSuchFunction()",
            "    call add(g:log, 'after')",
            "  finally",
            "    call add(g:log, 'finally')",
            "  endtry",
            "catch /E117/",
            "  call add(g:log, 'caught ' . v:exception)",
            "endtry",
        ],
    );
    assert_eq!(
        log,
        "['before', 'finally', 'caught Vim(call):E117: Unknown function: EeNoSuchFunction']"
    );

    // An interrupt is an exception too.
    let log = logged(
        &fx,
        &[
            "try",
            "  try",
            "    call interrupt()",
            "    call add(g:log, 'not reached')",
            "  finally",
            "    call add(g:log, 'finally')",
            "  endtry",
            "catch /^Vim:Interrupt$/",
            "  call add(g:log, 'caught ' . v:exception)",
            "endtry",
        ],
    );
    assert_eq!(log, "['finally', 'caught Vim:Interrupt']");
    fx.delete(&["g:EeReturn", "g:EeOverride", "g:EeReturnInLoop"]);
}

#[test]
fn rethrow_and_values_that_are_not_strings() {
    let fx = Fixture::new();
    let log = logged(
        &fx,
        &[
            "try",
            "  try",
            "    throw 'first'",
            "  catch",
            "    call add(g:log, 'inner ' . v:exception)",
            "    throw v:exception",
            "  endtry",
            "catch",
            "  call add(g:log, 'outer ' . v:exception)",
            "endtry",
        ],
    );
    assert_eq!(log, "['inner first', 'outer first']");
    let log = logged(
        &fx,
        &[
            "for v in ['42', '1.5', '[1]', '{}', 'v:true', 'v:null']",
            "  try",
            "    execute 'throw ' . v",
            "  catch",
            "    call add(g:log, v . ': ' . v:exception)",
            "  endtry",
            "endfor",
        ],
    );
    assert_eq!(
        log,
        "['42: 42', '1.5: 1.5', '[1]: Vim(throw):E730: Using a List as a String', \
         '{}: Vim(throw):E731: Using a Dictionary as a String', 'v:true: v:true', \
         'v:null: v:null']"
    );
    // A user value may not fake the editor's own.
    let log = logged(
        &fx,
        &[
            "for v in ['Vim', 'Vim:x', 'Vim(x):y', 'Vimx', 'vim:x']",
            "  try",
            "    throw v",
            "  catch",
            "    call add(g:log, v:exception)",
            "  endtry",
            "endfor",
        ],
    );
    assert_eq!(
        log,
        "['Vim(throw):E608: Cannot :throw exceptions with ''Vim'' prefix', \
         'Vim(throw):E608: Cannot :throw exceptions with ''Vim'' prefix', \
         'Vim(throw):E608: Cannot :throw exceptions with ''Vim'' prefix', 'Vimx', 'vim:x']"
    );
}

#[test]
fn exception_and_throwpoint_name_what_was_thrown_where() {
    let fx = Fixture::new();
    fx.block(&[
        "function! g:EeThrower()",
        "  let x = 1",
        "  throw 'from function'",
        "endfunction",
        "function! g:EeErr()",
        "  let x = 1",
        "  let y = no_such_variable",
        "endfunction",
    ]);
    let log = logged(
        &fx,
        &[
            "try",
            "  call g:EeThrower()",
            "catch",
            "  call add(g:log, v:exception)",
            "  call add(g:log, v:throwpoint)",
            "  call add(g:log, len(v:stacktrace) > 0)",
            "endtry",
            "try",
            "  call g:EeErr()",
            "catch",
            "  call add(g:log, v:exception)",
            "  call add(g:log, v:throwpoint)",
            "endtry",
            "call add(g:log, [v:exception, v:throwpoint, v:stacktrace])",
        ],
    );
    assert_eq!(
        log,
        "['from function', 'function EeThrower, line 2', 1, \
         'Vim(let):E121: Undefined variable: no_such_variable', 'function EeErr, line 2', \
         ['', '', []]]"
    );
    // Caught inside a catch clause: the inner one is current, the outer one
    // comes back when it is finished.
    let log = logged(
        &fx,
        &[
            "try",
            "  throw 'outer'",
            "catch",
            "  try",
            "    throw 'inner'",
            "  catch",
            "    call add(g:log, v:exception)",
            "  endtry",
            "  call add(g:log, v:exception)",
            "endtry",
        ],
    );
    assert_eq!(log, "['inner', 'outer']");
    fx.delete(&["g:EeThrower", "g:EeErr"]);
}

#[test]
fn verbose_reports_what_an_exception_went_through() {
    let fx = Fixture::new();
    // Set behind `:set`'s back: `:set` would redraw a screen the fixture
    // does not have.
    let report = |lines: &[&str]| {
        let quoted: Vec<String> = lines
            .iter()
            .map(|line| format!("'{}'", line.replace('\'', "''")))
            .collect();
        P_VERBOSE.set(14);
        let report = fx.eval(&format!(
            "filter(split(execute([{}]), \"\\n\"), 'len(v:val)')",
            quoted.join(", ")
        ));
        P_VERBOSE.set(0);
        report
    };
    assert_eq!(
        report(&["try", "  throw 'x'", "catch", "endtry"]),
        "['Exception thrown: x', 'Exception caught: x', 'Exception finished: x']"
    );
    assert_eq!(
        report(&[
            "try",
            "  try",
            "    throw 'y'",
            "  finally",
            "  endtry",
            "catch",
            "endtry"
        ]),
        "['Exception thrown: y', 'Exception made pending: y', 'Exception resumed: y', \
         'Exception caught: y', 'Exception finished: y']"
    );
    assert_eq!(
        report(&[
            "try",
            "  try",
            "    throw 'z'",
            "  finally",
            "    throw 'w'",
            "  endtry",
            "catch",
            "endtry"
        ]),
        "['Exception thrown: z', 'Exception made pending: z', 'Exception thrown: w', \
         'Exception discarded: z', 'Exception caught: w', 'Exception finished: w']"
    );
    assert_eq!(
        report(&[
            "for i in [1]",
            "  try",
            "    break",
            "  finally",
            "  endtry",
            "endfor",
            "while 1",
            "  try",
            "    continue",
            "  finally",
            "    break",
            "  endtry",
            "endwhile",
        ]),
        // A :break in the finally clause itself is not pending: it ends
        // the loop at once, and discards the :continue.
        "[':break made pending', ':break resumed', ':continue made pending', \
         ':continue discarded']"
    );
    fx.block(&[
        "function! g:EeRet()",
        "  try",
        "    return [1, 'two']",
        "  finally",
        "  endtry",
        "endfunction",
    ]);
    assert_eq!(
        report(&["call g:EeRet()"]),
        "['calling EeRet()', ':return [1, ''two''] made pending', ':return [1, ''two''] resumed', \
         'EeRet returning [1, ''two'']']"
    );
    fx.delete(&["g:EeRet"]);
}

/// `opens` levels of `open`, each counting itself in `g:depth`, closed by
/// `closes` of `close`.
fn nested(open: &str, close: &str, opens: usize, closes: usize) -> Vec<String> {
    let mut lines = vec!["let g:depth = 0".to_owned()];
    for _ in 0..opens {
        lines.push(open.to_owned());
        lines.push("let g:depth += 1".to_owned());
    }
    for _ in 0..closes {
        lines.push(close.to_owned());
    }
    lines
}

/// The error number running `lines` as a function body ends on: the
/// missing `:end…` errors are only given at the end of a function or a
/// sourced file.
fn error_in_function(fx: &Fixture, lines: &[&str]) -> String {
    let mut body = vec!["function! g:EeBody()"];
    body.extend_from_slice(lines);
    body.push("endfunction");
    fx.block(&body);
    let err = error_after(fx, &["call g:EeBody()"]);
    fx.delete(&["g:EeBody"]);
    err
}

#[test]
fn loops_and_conditionals_nest_to_the_stack_cap() {
    let fx = Fixture::new();
    // A `:for` that loops is not taken to 50: upstream checks the depth
    // before it knows the `:for` is its own loop coming round again, so the
    // 50th level's second pass is refused with E585 -- and under `:silent!`
    // never ends.
    let lines = nested("for x in [1]", "endfor", 49, 49);
    let lines: Vec<&str> = lines.iter().map(String::as_str).collect();
    assert_eq!(error_after(&fx, &lines), "none", "for at 49");
    assert_eq!(fx.eval("g:depth"), "49", "for at 49");
    for (open, close, err) in [
        ("while 1", "break | endwhile", "E585"),
        ("if 1", "endif", "E579"),
        ("try", "endtry", "E601"),
    ] {
        let lines = nested(open, close, 50, 50);
        let lines: Vec<&str> = lines.iter().map(String::as_str).collect();
        assert_eq!(error_after(&fx, &lines), "none", "{open} at 50");
        assert_eq!(fx.eval("g:depth"), "50", "{open} at 50");
        if open == "try" {
            // A 51st `:try` run this way never returns, in upstream's build
            // as in this one.
            continue;
        }
        // The 51st is refused, and the body after it runs in the 50th.
        let lines = nested(open, close, 51, 50);
        let lines: Vec<&str> = lines.iter().map(String::as_str).collect();
        assert_eq!(error_after(&fx, &lines), err, "{open} at 51");
        assert_eq!(fx.eval("g:depth"), "51", "{open} at 51");
    }
    // Loops inside a :while body that is replayed run as many times as
    // asked, with a :for over a list edited while it is walked.
    let log = logged(
        &fx,
        &[
            "let i = 0",
            "while i < 3",
            "  let i += 1",
            "  let l = [1, 2, 3]",
            "  for x in l",
            "    if x == 1",
            "      call add(l, 4)",
            "    elseif x == 2",
            "      call remove(l, 2)",
            "    endif",
            "    call add(g:log, i * 10 + x)",
            "  endfor",
            "endwhile",
        ],
    );
    assert_eq!(log, "[11, 12, 14, 21, 22, 24, 31, 32, 34]");
    let log = logged(
        &fx,
        &[
            "for [a, b; rest] in [[1, 2], [3, 4, 5, 6]]",
            "  call add(g:log, [a, b, rest])",
            "endfor",
            "for c in 'añb'",
            "  call add(g:log, c)",
            "endfor",
            "for n in 0z0102",
            "  call add(g:log, n)",
            "endfor",
            "for n in v:_null_list",
            "  call add(g:log, 'never')",
            "endfor",
        ],
    );
    assert_eq!(log, "[[1, 2, []], [3, 4, [5, 6]], 'a', 'ñ', 'b', 1, 2]");
}

#[test]
fn each_command_out_of_place_gives_its_error() {
    let fx = Fixture::new();
    // Each case gives one error and ends: the fixture runs under
    // `:silent!`, where an error does not stop a loop.
    for (lines, err) in [
        (&["endwhile"][..], "E588"),
        (&["endfor"][..], "E588"),
        (&["endif"][..], "E580"),
        (&["else"][..], "E581"),
        (&["elseif 1"][..], "E582"),
        (&["if 1", "else", "else", "endif"][..], "E583"),
        (&["if 1", "else", "elseif 1", "endif"][..], "E584"),
        (&["continue"][..], "E586"),
        (&["break"][..], "E587"),
        (
            &["let i = 0", "while i < 1", "let i += 1", "endfor"][..],
            "E732",
        ),
        (&["for x in []", "endwhile"][..], "E733"),
        (&["if 1", "  while 0", "endif"][..], "E580"),
        (&["catch"][..], "E603"),
        (&["finally"][..], "E606"),
        (&["endtry"][..], "E602"),
        (&["throw"][..], "E471"),
        (&["throw 'uncaught'"][..], "E605"),
        (&["endfunction"][..], "E193"),
        (&["return 1"][..], "E133"),
        (&["for x 1", "endfor"][..], "E690"),
        (&["for x in 1", "endfor"][..], "E1098"),
    ] {
        assert_eq!(error_after(&fx, lines), err, "{lines:?}");
    }
    // An error inside a :try is an error exception: what it says is what
    // an outer :catch sees.
    for (lines, err) in [
        (
            &["try", "finally", "catch", "endtry"][..],
            "E604: :catch after :finally",
        ),
        (
            &["try", "finally", "finally", "endtry"][..],
            "E607: Multiple :finally",
        ),
        (
            &["try", "  while 0", "endtry"][..],
            "E170: Missing :endwhile",
        ),
        (
            &["try", "  for x in []", "endtry"][..],
            "E170: Missing :endfor",
        ),
        (
            &["try", "  if 1", "finally", "endtry"][..],
            "E171: Missing :endif",
        ),
        (
            &["try", "  if 1", "catch", "endtry"][..],
            "E171: Missing :endif",
        ),
    ] {
        let mut block = vec!["try"];
        block.extend_from_slice(lines);
        block.extend_from_slice(&["catch", "  call add(g:log, v:exception)", "endtry"]);
        let log = logged(&fx, &block);
        assert!(log.contains(err), "{lines:?}: {log}");
    }
    // Without a try the script goes on: the fixture is under `:silent!`.
    let log = logged(
        &fx,
        &[
            "call add(g:log, 1)",
            "call EeNoSuchFunction()",
            "call add(g:log, 2)",
        ],
    );
    assert_eq!(log, "[1, 2]");
    assert_eq!(
        fx.eval("v:errmsg"),
        "'E117: Unknown function: EeNoSuchFunction'"
    );
}

#[test]
fn a_function_that_ends_inside_a_conditional_reports_it() {
    let fx = Fixture::new();
    for (lines, err) in [
        (&["while 1", "break"][..], "E170"),
        (&["for x in [1]"][..], "E170"),
        (&["if 1"][..], "E171"),
        (&["if 0"][..], "E171"),
        (&["if 1", "  while 0", "  endwhile"][..], "E171"),
    ] {
        assert_eq!(error_in_function(&fx, lines), err, "{lines:?}");
    }
    // A :try left open in a function resets `:silent!` for the rest of it,
    // so its E600 is an error exception the caller's :try catches.
    for lines in [&["try"][..], &["try", "catch"][..], &["try", "finally"][..]] {
        let mut body = vec!["function! g:EeBody()"];
        body.extend_from_slice(lines);
        body.push("endfunction");
        fx.block(&body);
        let log = logged(
            &fx,
            &[
                "try",
                "  call g:EeBody()",
                "catch",
                "  call add(g:log, v:exception)",
                "endtry",
            ],
        );
        assert_eq!(
            log, "['Vim(endfunction):E600: Missing :endtry']",
            "{lines:?}"
        );
        fx.delete(&["g:EeBody"]);
    }
    // The missing :endif is an error exception an enclosing try catches.
    fx.block(&["function! g:EeOpenIf()", "  if 1", "endfunction"]);
    let log = logged(
        &fx,
        &[
            "try",
            "  call g:EeOpenIf()",
            "catch",
            "  call add(g:log, v:exception)",
            "  call add(g:log, v:throwpoint)",
            "endtry",
        ],
    );
    assert_eq!(
        log,
        "['Vim(endfunction):E171: Missing :endif', 'function EeOpenIf, line 1']"
    );
    fx.delete(&["g:EeOpenIf"]);
}

#[test]
fn an_abort_function_stops_at_its_first_error_unless_caught() {
    let fx = Fixture::new();
    fx.block(&[
        "function! g:EeAbort() abort",
        "  call add(g:log, 'a')",
        "  call EeNoSuchFunction()",
        "  call add(g:log, 'b')",
        "endfunction",
        "function! g:EeNoAbort()",
        "  call add(g:log, 'a')",
        "  call EeNoSuchFunction()",
        "  call add(g:log, 'b')",
        "endfunction",
    ]);
    // Under `:silent!` (the fixture's) an error sets no `did_emsg`, so not
    // even an `abort` function stops.
    assert_eq!(logged(&fx, &["call g:EeAbort()"]), "['a', 'b']");
    assert_eq!(logged(&fx, &["call g:EeNoAbort()"]), "['a', 'b']");
    // A :try turns the error into an exception, which stops both.
    for name in ["g:EeAbort", "g:EeNoAbort"] {
        assert_eq!(
            logged(
                &fx,
                &[
                    "try",
                    &format!("  call {name}()"),
                    "catch",
                    "  call add(g:log, v:exception)",
                    "endtry",
                ]
            ),
            "['a', 'Vim(call):E117: Unknown function: EeNoSuchFunction']",
            "{name}"
        );
    }
    assert_eq!(
        logged(
            &fx,
            &["silent! try | call EeNoSuchFunction() | catch | call add(g:log, 'caught') | endtry"]
        ),
        "['caught']"
    );
    fx.delete(&["g:EeAbort", "g:EeNoAbort"]);
}
