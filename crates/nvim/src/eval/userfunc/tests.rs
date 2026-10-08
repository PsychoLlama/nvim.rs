//! User functions end to end, driven by the commands that reach them: a
//! definition, its calls, and the ways a function's life and a call's life
//! cross -- a function redefined or deleted while it runs, a closure that
//! outlives the call it captured, `:defer` during an exception, a funcref
//! that outlives its function, script-local names across scripts, and the
//! profiler's per-call and per-line counts.
//!
//! Each case defines its own `g:Ut*` functions and deletes them again, so the
//! function table is what it was when the case started.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::eval::test_fixture::{Fixture, Scratch};
use crate::option::vars::P_MFD;

#[test]
fn a_function_is_defined_called_and_deleted() {
    let fx = Fixture::new();
    fx.block(&[
        "function! g:UtAdd(a, b = 2, ...)",
        "  let l:sum = a:a + a:b",
        "  return [l:sum, a:0, a:000, a:firstline <= a:lastline]",
        "endfunction",
    ]);
    assert_eq!(fx.eval("g:UtAdd(1)"), "[3, 0, [], 1]");
    assert_eq!(fx.eval("g:UtAdd(1, 5, 'x', [7])"), "[6, 2, ['x', [7]], 1]");
    assert_eq!(fx.error_of("g:UtAdd()"), "E119");
    assert_eq!(fx.eval("exists('*g:UtAdd')"), "1");
    // Both calls returned their `a:000`, so both funccalls were kept for
    // the collector, each holding the function: it cannot go yet.
    fx.run("let v:errmsg = ''");
    fx.run("silent! delfunction g:UtAdd");
    let held = fx.eval("v:errmsg");
    assert!(held.contains("It is being used internally"), "{held}");
    fx.run("call test_garbagecollect_now()");
    fx.run("let v:errmsg = ''");
    fx.run("delfunction g:UtAdd");
    assert_eq!(fx.eval("v:errmsg"), "''");
    assert_eq!(fx.eval("exists('*g:UtAdd')"), "0");
    assert_eq!(fx.error_of("g:UtAdd(1)"), "E117");
}

#[test]
fn a_running_function_can_be_neither_redefined_nor_deleted() {
    let fx = Fixture::new();
    fx.block(&[
        "function! g:UtSelf()",
        "  let g:UtRedefine = 'none'",
        "  try",
        "    execute \"function! g:UtSelf()\\nreturn 2\\nendfunction\"",
        "  catch",
        "    let g:UtRedefine = v:exception",
        "  endtry",
        "  let g:UtDelete = 'none'",
        "  try",
        "    delfunction g:UtSelf",
        "  catch",
        "    let g:UtDelete = v:exception",
        "  endtry",
        "  return 1",
        "endfunction",
    ]);
    assert_eq!(fx.eval("g:UtSelf()"), "1");
    let errors = fx.eval("[g:UtRedefine, g:UtDelete]");
    assert!(
        errors.contains("E127: Cannot redefine function UtSelf: It is in use"),
        "{errors}"
    );
    assert!(
        errors.contains("E131: Cannot delete function g:UtSelf: It is in use"),
        "{errors}"
    );
    // Not running any more: both are allowed now.
    fx.block(&["function! g:UtSelf()", "return 2", "endfunction"]);
    assert_eq!(fx.eval("g:UtSelf()"), "2");
    fx.run("unlet g:UtRedefine g:UtDelete");
    fx.delete(&["g:UtSelf"]);
}

#[test]
fn a_function_held_by_a_funcref_outlives_its_redefinition_and_deletion() {
    let fx = Fixture::new();
    fx.block(&["function! g:UtHeld()", "return 1", "endfunction"]);
    fx.run("let g:UtRef = funcref('g:UtHeld')");
    fx.run("let g:UtName = function('g:UtHeld')");
    // Redefined beside the funcref: the funcref keeps the old body, the
    // name answers the new one.
    fx.block(&["function! g:UtHeld()", "return 2", "endfunction"]);
    assert_eq!(fx.eval("[g:UtRef(), g:UtName(), g:UtHeld()]"), "[1, 2, 2]");
    // Deleted while a funcref holds it: the name is gone, the funcref
    // reports the deletion.
    fx.run("let g:UtRef2 = funcref('g:UtHeld')");
    fx.run("delfunction g:UtHeld");
    assert_eq!(fx.eval("exists('*g:UtHeld')"), "0");
    assert_eq!(fx.error_of("g:UtRef2()"), "E933");
    assert_eq!(fx.error_of("g:UtName()"), "E117");
    assert_eq!(fx.eval("g:UtRef()"), "1");
    fx.run("unlet g:UtRef g:UtRef2 g:UtName");
}

#[test]
fn a_closure_keeps_the_scopes_of_the_call_that_made_it() {
    let fx = Fixture::new();
    fx.block(&[
        "function! g:UtOuter(x)",
        "  let l:count = 10",
        "  function! g:UtInner() closure",
        "    let l:count += 1",
        "    return [a:x, l:count]",
        "  endfunction",
        "  return {n -> [n, a:x, l:count]}",
        "endfunction",
    ]);
    fx.run("let g:UtLam = g:UtOuter(['arg'])");
    // The call has returned; its `a:` and `l:` live on in the closures.
    assert_eq!(fx.eval("g:UtInner()"), "[['arg'], 11]");
    assert_eq!(fx.eval("g:UtInner()"), "[['arg'], 12]");
    assert_eq!(fx.eval("g:UtLam(5)"), "[5, ['arg'], 12]");
    fx.run("call test_garbagecollect_now()");
    assert_eq!(fx.eval("g:UtLam(6)"), "[6, ['arg'], 12]");
    assert_eq!(fx.eval("g:UtInner()"), "[['arg'], 13]");
    // A second call captures a scope of its own.
    fx.run("let g:UtLam2 = g:UtOuter('two')");
    assert_eq!(
        fx.eval("[g:UtLam(1), g:UtLam2(1)]"),
        "[[1, ['arg'], 13], [1, 'two', 10]]"
    );
    fx.run("unlet g:UtLam g:UtLam2");
    fx.run("call test_garbagecollect_now()");
    fx.delete(&["g:UtOuter", "g:UtInner"]);
}

#[test]
fn a_function_calls_itself() {
    let fx = Fixture::new();
    fx.block(&[
        "function! g:UtFact(n)",
        "  return a:n <= 1 ? 1 : a:n * g:UtFact(a:n - 1)",
        "endfunction",
        "function! g:UtDeep(n)",
        "  return g:UtDeep(a:n + 1)",
        "endfunction",
    ]);
    assert_eq!(fx.eval("g:UtFact(10)"), "3628800");
    // 'maxfuncdepth' stops a runaway recursion with E132; a low one, so
    // that a debug build's frames fit a test thread's stack.
    P_MFD.set(20);
    assert_eq!(fx.error_of("g:UtDeep(0)"), "E132");
    fx.delete(&["g:UtFact", "g:UtDeep"]);
}

#[test]
fn deferred_calls_run_newest_first_through_an_exception() {
    let fx = Fixture::new();
    fx.run("let g:UtLog = []");
    fx.block(&[
        "function! g:UtDefer()",
        "  defer add(g:UtLog, 'first')",
        "  defer add(g:UtLog, 'second')",
        "  call add(g:UtLog, 'body')",
        "  throw 'boom'",
        "  call add(g:UtLog, 'unreached')",
        "endfunction",
        "function! g:UtDeferOk(x)",
        "  defer add(g:UtLog, a:x)",
        "  return a:x",
        "endfunction",
    ]);
    fx.run("try | call g:UtDefer() | catch /boom/ | call add(g:UtLog, 'caught') | endtry");
    assert_eq!(fx.eval("g:UtLog"), "['body', 'second', 'first', 'caught']");
    fx.run("let g:UtLog = []");
    assert_eq!(
        fx.eval("[g:UtDeferOk(1), g:UtDeferOk(2), g:UtLog]"),
        "[1, 2, [1, 2]]"
    );
    fx.run("unlet g:UtLog");
    fx.delete(&["g:UtDefer", "g:UtDeferOk"]);
}

#[test]
fn a_lambda_and_a_numbered_dict_function_are_callable_by_value() {
    let fx = Fixture::new();
    fx.run("let g:UtDict = {'v': 7}");
    fx.block(&[
        "function! g:UtDict.get(n) dict",
        "return self.v + a:n",
        "endfunction",
    ]);
    assert_eq!(fx.eval("g:UtDict.get(1)"), "8");
    fx.run("let g:UtF = g:UtDict.get");
    fx.run("let g:UtLam = {a, ... -> [a, a:000]}");
    assert_eq!(fx.eval("g:UtLam(1, 2, 3)"), "[1, [2, 3]]");
    assert_eq!(fx.eval("call(g:UtLam, [4])"), "[4, []]");
    fx.run("unlet g:UtDict");
    // The entry holding the numbered function is gone, but the funcref
    // still names it.
    assert_eq!(fx.eval("call(g:UtF, [2], {'v': 1})"), "3");
    fx.run("unlet g:UtF g:UtLam");
}

#[test]
#[cfg_attr(miri, ignore = "`:source` resolves its file through libuv")]
fn script_local_functions_are_reached_across_scripts_by_funcref() {
    let fx = Fixture::new();
    let scratch = Scratch::new("sid");
    let a = scratch.file(
        "a.vim",
        "function! s:Local()\n  return 'a'\nendfunction\nlet g:UtFromA = function('s:Local')\nlet g:UtRefA = funcref('s:Local')\n",
    );
    let b = scratch.file(
        "b.vim",
        "function! s:Local()\n  return 'b'\nendfunction\nlet g:UtFromB = [g:UtFromA(), s:Local(), function('s:Local')()]\n",
    );
    fx.run(&format!("source {a}"));
    fx.run(&format!("source {b}"));
    assert_eq!(fx.eval("g:UtFromB"), "['a', 'b', 'b']");
    assert_eq!(fx.eval("[g:UtFromA(), g:UtRefA()]"), "['a', 'a']");
    // Outside any script, `s:` names nothing.
    assert_eq!(fx.error_of("s:Local()"), "E81");
    fx.run("unlet g:UtFromA g:UtRefA g:UtFromB");
}

#[test]
#[cfg_attr(miri, ignore = "the profiler reads the clock through libuv")]
fn the_profiler_counts_a_nested_call() {
    let fx = Fixture::new();
    let scratch = Scratch::new("prof");
    let report = scratch.0.join("profile.txt");
    let report = report.to_str().expect("a UTF-8 path");
    fx.block(&[
        "function! g:UtLeaf(x)",
        "  return a:x + 1",
        "endfunction",
        "function! g:UtNest()",
        "  let l:n = 0",
        "  for l:i in range(3)",
        "    let l:n = g:UtLeaf(l:n)",
        "  endfor",
        "  return l:n",
        "endfunction",
    ]);
    fx.run(&format!("profile start {report}"));
    fx.run("profile func UtLeaf");
    fx.run("profile func UtNest");
    assert_eq!(fx.eval("g:UtNest()"), "3");
    fx.run("profile stop");
    let text = std::fs::read_to_string(report).expect("the report");
    // The count a body line was run, by its text; the times vary.
    let count_of = |body: &str| -> Option<&str> {
        let line = text.lines().find(|line| line.ends_with(body))?;
        Some(line[..5].trim())
    };
    assert!(
        text.contains("FUNCTION  UtLeaf()\nCalled 3 times\n"),
        "{text}"
    );
    assert!(
        text.contains("FUNCTION  UtNest()\nCalled 1 time\n"),
        "{text}"
    );
    assert_eq!(count_of("  return a:x + 1"), Some("3"), "{text}");
    assert_eq!(count_of("  for l:i in range(3)"), Some("4"), "{text}");
    assert_eq!(count_of("  let l:n = g:UtLeaf(l:n)"), Some("3"), "{text}");
    assert_eq!(count_of("  return l:n"), Some("1"), "{text}");
    // The caller's total holds the callee's.
    let by_total = &text[text.find("SORTED ON TOTAL").expect("the total list")..];
    assert!(
        by_total.find("UtNest()") < by_total.find("UtLeaf()"),
        "{text}"
    );
    fx.delete(&["g:UtLeaf", "g:UtNest"]);
}
