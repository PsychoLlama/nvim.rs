//! The `assert_*()` builtins end to end: what each answers, and the exact
//! line a failing one appends to `v:errors` — the sourcing position, the
//! caller's message, the `Expected … but got …` rendering of every value
//! type, the escaping and run-length shortening, the pruned dictionaries —
//! plus `assert_fails()`'s checks of the error, line and context, and the
//! two `test_*()` builtins.
//!
//! The functions a case defines are `g:Ta*`, deleted again at the end.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::eval::test_fixture::{Fixture, Scratch};

/// `v:errors` as `string()` renders it, emptied for the next check.
fn errors(fx: &Fixture) -> String {
    let errors = fx.eval("v:errors");
    fx.run("let v:errors = []");
    errors
}

/// Evaluate `call` and answer its result and the `v:errors` it left.
fn check(fx: &Fixture, call: &str) -> (String, String) {
    fx.run("let v:errors = []");
    let result = fx.eval(call);
    (result, errors(fx))
}

/// The one line a failing `call` appended, or a panic naming what it did
/// instead.
fn failure(fx: &Fixture, call: &str) -> String {
    let (result, errors) = check(fx, call);
    assert_eq!(result, "1", "{call} should fail; v:errors = {errors}");
    errors
}

/// A `call` that holds answers 0 and appends nothing.
fn holds(fx: &Fixture, call: &str) {
    assert_eq!(check(fx, call), ("0".into(), "[]".into()), "{call}");
}

#[test]
fn assert_equal_renders_every_value_type() {
    let fx = Fixture::new();
    holds(&fx, "assert_equal(1, 1)");
    holds(&fx, "assert_equal('a', 'a')");
    holds(&fx, "assert_equal([1, {'a': 2}], [1, {'a': 2}])");
    // Not equal across types, and case matters.
    assert_eq!(
        failure(&fx, "assert_equal(1, '1')"),
        "['Expected 1 but got ''1''']"
    );
    assert_eq!(
        failure(&fx, "assert_equal('abc', 'ABC')"),
        "['Expected ''abc'' but got ''ABC''']"
    );
    assert_eq!(
        failure(&fx, "assert_equal([1, 2], [1, 3])"),
        "['Expected [1, 2] but got [1, 3]']"
    );
    assert_eq!(
        failure(&fx, "assert_equal(0z0102, 0z01)"),
        "['Expected 0z0102 but got 0z01']"
    );
    assert_eq!(
        failure(&fx, "assert_equal(function('tr'), function('len'))"),
        "['Expected function(''tr'') but got function(''len'')']"
    );
    assert_eq!(
        failure(&fx, "assert_equal(function('tr', [1]), function('tr'))"),
        "['Expected function(''tr'', [1]) but got function(''tr'')']"
    );
    assert_eq!(
        failure(&fx, "assert_equal(v:true, v:false)"),
        "['Expected v:true but got v:false']"
    );
    assert_eq!(
        failure(&fx, "assert_equal(v:null, 0)"),
        "['Expected v:null but got 0']"
    );
    assert_eq!(
        failure(&fx, "assert_equal(v:_null_list, v:_null_dict)"),
        "['Expected [] but got {}']"
    );
    // The caller's message leads; an empty one counts as none, and a
    // non-string one is echoed.
    assert_eq!(
        failure(&fx, "assert_equal(1, 2, 'mine')"),
        "['mine: Expected 1 but got 2']"
    );
    assert_eq!(
        failure(&fx, "assert_equal(1, 2, '')"),
        "['Expected 1 but got 2']"
    );
    assert_eq!(
        failure(&fx, "assert_equal(1, 2, [3])"),
        "['[3]: Expected 1 but got 2']"
    );
}

#[test]
#[cfg_attr(miri, ignore = "a Float literal is read with `strtod`")]
fn assert_equal_renders_floats() {
    let fx = Fixture::new();
    holds(&fx, "assert_equal(1.5, 1.5)");
    assert_eq!(
        failure(&fx, "assert_equal(1.5, 1)"),
        "['Expected 1.5 but got 1']"
    );
}

#[test]
fn recursive_containers_are_cut_short() {
    let fx = Fixture::new();
    fx.run("let g:ta_l = [1]");
    fx.run("call add(g:ta_l, g:ta_l)");
    fx.run("let g:ta_d = {'a': 1}");
    fx.run("let g:ta_d.self = g:ta_d");
    // The pruned copy is a new dictionary, so the walk meets the original
    // once before it knows it is recursive.
    assert_eq!(
        failure(&fx, "assert_equal([], g:ta_l)"),
        "['Expected [] but got [1, {E724@0}]']"
    );
    assert_eq!(
        failure(&fx, "assert_equal(g:ta_d, {})"),
        "['Expected {''a'': 1, ''self'': {''a'': 1, ''self'': {E724@1}}} but got {}']"
    );
    fx.run("unlet g:ta_l g:ta_d");
}

#[test]
fn strings_are_escaped_and_long_runs_shortened() {
    let fx = Fixture::new();
    assert_eq!(
        failure(&fx, r#"assert_equal("a\nb\tc\rd\e\x01\x7f\\", 'x')"#),
        r"['Expected ''a\nb\tc\rd\e\x01\x7f\\'' but got ''x''']"
    );
    assert_eq!(
        failure(&fx, r#"assert_equal("\b\f", 'x')"#),
        r"['Expected ''\b\f'' but got ''x''']"
    );
    // Twenty of a kind stay; twenty-one collapse, multibyte or not.
    let twenty = "x".repeat(20);
    assert_eq!(
        failure(&fx, &format!("assert_equal('{twenty}', 'y')")),
        format!("['Expected ''{twenty}'' but got ''y''']")
    );
    assert_eq!(
        failure(&fx, "assert_equal(repeat('x', 21), 'y')"),
        r"['Expected ''\[x occurs 21 times]'' but got ''y''']"
    );
    assert_eq!(
        failure(
            &fx,
            "assert_equal('ab' . repeat('é', 30) . 'cd', repeat(\"\\n\", 25))"
        ),
        r"['Expected ''ab\[é occurs 30 times]cd'' but got ''\[\n occurs 25 times]''']"
    );
}

#[test]
fn dictionaries_drop_their_equal_items() {
    let fx = Fixture::new();
    assert_eq!(
        failure(
            &fx,
            "assert_equal({'a': 1, 'b': 2, 'c': 3}, {'a': 1, 'b': 5, 'd': 4})"
        ),
        "['Expected {''b'': 2, ''c'': 3} but got {''b'': 5, ''d'': 4} - 1 equal item omitted']"
    );
    assert_eq!(
        failure(
            &fx,
            "assert_equal({'a': 1, 'b': 2, 'c': 3}, {'a': 1, 'b': 2, 'c': 4})"
        ),
        "['Expected {''c'': 3} but got {''c'': 4} - 2 equal items omitted']"
    );
    // `assert_notequal()` prints the value whole.
    assert_eq!(
        failure(&fx, "assert_notequal({'a': 1}, {'a': 1})"),
        "['Expected not equal to {''a'': 1}']"
    );
}

#[test]
fn the_sourcing_position_leads_the_line() {
    let fx = Fixture::new();
    fx.block(&[
        "function! g:TaInner()",
        "  let x = 1",
        "  return assert_equal(1, 2)",
        "endfunction",
        "function! g:TaOuter()",
        "  return g:TaInner()",
        "endfunction",
    ]);
    assert_eq!(
        failure(&fx, "g:TaInner()"),
        "['function TaInner line 2: Expected 1 but got 2']"
    );
    assert_eq!(
        failure(&fx, "g:TaOuter()"),
        "['function TaOuter[1]..TaInner line 2: Expected 1 but got 2']"
    );
    fx.delete(&["g:TaInner", "g:TaOuter"]);
}

#[test]
fn assert_notequal_match_and_notmatch() {
    let fx = Fixture::new();
    holds(&fx, "assert_notequal(1, 2)");
    assert_eq!(
        failure(&fx, "assert_notequal('x', 'x', 'why')"),
        "['why: Expected not equal to ''x''']"
    );
    holds(&fx, "assert_match('^a.c$', 'abc')");
    holds(&fx, "assert_notmatch('^b', 'abc')");
    assert_eq!(
        failure(&fx, "assert_match('^b', 'abc')"),
        "['Pattern ''^b'' does not match ''abc''']"
    );
    assert_eq!(
        failure(&fx, "assert_notmatch('b', 'abc', 'msg')"),
        "['msg: Pattern ''b'' does match ''abc''']"
    );
    // A number is matched as its text.
    holds(&fx, "assert_match('2', 123)");
    // An argument with no string form reports itself and holds.
    assert_eq!(fx.error_of("assert_match([], 'a')"), "E730");
    assert_eq!(check(&fx, "assert_match('a', [])").1, "[]");
}

#[test]
fn assert_true_and_false() {
    let fx = Fixture::new();
    for call in [
        "assert_true(1)",
        "assert_true(-3)",
        "assert_true(v:true)",
        "assert_false(0)",
        "assert_false(v:false)",
    ] {
        holds(&fx, call);
    }
    assert_eq!(
        failure(&fx, "assert_true(0)"),
        "['Expected True but got 0']"
    );
    assert_eq!(
        failure(&fx, "assert_true(v:false, 'm')"),
        "['m: Expected True but got v:false']"
    );
    assert_eq!(
        failure(&fx, "assert_false(1)"),
        "['Expected False but got 1']"
    );
    // Neither a string nor v:null is a boolean.
    assert_eq!(
        failure(&fx, "assert_true('1')"),
        "['Expected True but got ''1''']"
    );
    assert_eq!(
        failure(&fx, "assert_false(v:null)"),
        "['Expected False but got v:null']"
    );
    assert_eq!(
        failure(&fx, "assert_false([])"),
        "['Expected False but got []']"
    );
}

#[test]
fn assert_inrange_with_numbers() {
    let fx = Fixture::new();
    holds(&fx, "assert_inrange(1, 3, 1)");
    holds(&fx, "assert_inrange(1, 3, 3)");
    assert_eq!(
        failure(&fx, "assert_inrange(1, 3, 4)"),
        "['Expected range 1 - 3, but got 4']"
    );
    assert_eq!(
        failure(&fx, "assert_inrange(5, 7, -1, 'low')"),
        "['low: Expected range 5 - 7, but got -1']"
    );
    assert_eq!(fx.error_of("assert_inrange('a', 3, 1)"), "E1219");
    assert_eq!(fx.error_of("assert_inrange(1, 3, 1, 2)"), "E1174");
}

#[test]
#[cfg_attr(miri, ignore = "a Float literal is read with `strtod`")]
fn assert_inrange_with_floats() {
    let fx = Fixture::new();
    holds(&fx, "assert_inrange(1.0, 2.0, 1.5)");
    holds(&fx, "assert_inrange(1, 2, 1.5)");
    assert_eq!(
        failure(&fx, "assert_inrange(1.0, 2.5, 3)"),
        "['Expected range 1.0 - 2.5, but got 3']"
    );
    assert_eq!(
        failure(&fx, "assert_inrange(1, 2, 2.5)"),
        "['Expected range 1.0 - 2.0, but got 2.5']"
    );
}

#[test]
fn assert_fails_checks_the_error() {
    let fx = Fixture::new();
    fx.block(&[
        "function! g:TaFail()",
        "  let x = 1",
        "  call g:TaNoSuch()",
        "endfunction",
    ]);
    holds(&fx, "assert_fails('call g:TaNoSuch()')");
    holds(&fx, "assert_fails('call g:TaNoSuch()', 'E117')");
    holds(&fx, "assert_fails('call g:TaNoSuch()', ['E117:'])");
    holds(
        &fx,
        "assert_fails('call g:TaNoSuch()', ['E117', 'Unknown'])",
    );
    assert_eq!(
        failure(&fx, "assert_fails('let x = 1')"),
        "['command did not fail: let x = 1']"
    );
    assert_eq!(
        failure(&fx, "assert_fails('let x = 1', 'E1', 'label')"),
        "['command did not fail: label']"
    );
    assert_eq!(
        failure(&fx, "assert_fails('call g:TaNoSuch()', 'E999')"),
        "['Expected ''E999'' but got ''E117: Unknown function: TaNoSuch'': call g:TaNoSuch()']"
    );
    assert_eq!(
        failure(&fx, "assert_fails('call g:TaNoSuch()', ['E9'], 'm')"),
        "['m: Expected ''E9'' but got ''E117: Unknown function: TaNoSuch'': m']"
    );
    // The second pattern is matched against `v:errmsg`.
    assert_eq!(
        failure(&fx, "assert_fails('call g:TaNoSuch()', ['E117', 'nope'])"),
        "['Expected ''nope'' but got ''E117: Unknown function: TaNoSuch'': call g:TaNoSuch()']"
    );
    // The line the error came from, and its context. With a third argument
    // present, it names the command, even empty.
    holds(&fx, "assert_fails('call g:TaFail()', 'E117', '', 2)");
    holds(
        &fx,
        "assert_fails('call g:TaFail()', 'E117', '', -1, 'TaFail')",
    );
    assert_eq!(
        failure(&fx, "assert_fails('call g:TaFail()', 'E117', '', 5)"),
        "['Expected 5 but got 2: ']"
    );
    assert_eq!(
        failure(
            &fx,
            "assert_fails('call g:TaFail()', 'E117', '', 2, 'Other')"
        ),
        "['Expected ''Other'' but got ''TaFail'': ']"
    );
    // Its own argument errors.
    assert_eq!(fx.error_of("assert_fails('call g:TaNoSuch()', [])"), "E856");
    assert_eq!(
        fx.error_of("assert_fails('call g:TaNoSuch()', {})"),
        "E1222"
    );
    assert_eq!(
        fx.error_of("assert_fails('call g:TaNoSuch()', 'E117', '', 'x')"),
        "E1210"
    );
    // The failure it expected does not linger.
    fx.run("let v:errmsg = 'before'");
    let _ = check(&fx, "assert_fails('call g:TaNoSuch()')");
    assert_eq!(fx.eval("v:errmsg"), "''");
    fx.delete(&["g:TaFail"]);
}

#[test]
fn assert_report_and_exception() {
    let fx = Fixture::new();
    assert_eq!(failure(&fx, "assert_report('plain')"), "['plain']");
    assert_eq!(failure(&fx, "assert_report(42)"), "['42']");
    assert_eq!(
        failure(&fx, "assert_exception('x')"),
        "['v:exception is not set']"
    );
    fx.block(&[
        "function! g:TaCatch(pat, ...)",
        "  try",
        "    throw 'oops E99: a thing'",
        "  catch",
        "    return call('assert_exception', [a:pat] + a:000)",
        "  endtry",
        "endfunction",
    ]);
    holds(&fx, "g:TaCatch('E99')");
    assert_eq!(
        failure(&fx, "g:TaCatch('E12')"),
        "['function TaCatch line 4: Expected ''E12'' but got ''oops E99: a thing''']"
    );
    assert_eq!(
        failure(&fx, "g:TaCatch('E12', 'msg')"),
        "['function TaCatch line 4: msg: Expected ''E12'' but got ''oops E99: a thing''']"
    );
    fx.delete(&["g:TaCatch"]);
}

#[test]
fn test_garbagecollect_now_and_write_list_log() {
    let fx = Fixture::new();
    fx.run("let g:ta_keep = [[1], {'a': [2]}]");
    fx.run("let g:ta_cycle = [] | call add(g:ta_cycle, g:ta_cycle) | unlet g:ta_cycle");
    assert_eq!(fx.error_of("test_garbagecollect_now()"), "none");
    assert_eq!(fx.eval("g:ta_keep"), "[[1], {'a': [2]}]");
    fx.run("let v:testing = 0");
    assert_eq!(fx.error_of("test_garbagecollect_now()"), "E1142");
    fx.run("let v:testing = 1");
    fx.run("unlet g:ta_keep");
    assert_eq!(fx.error_of("test_write_list_log('Xlog')"), "none");
    assert_eq!(fx.error_of("test_write_list_log([])"), "E730");
}

#[test]
#[cfg_attr(miri, ignore = "the files are opened through libc's `fopen`")]
fn assert_equalfile_reports_the_first_difference() {
    let fx = Fixture::new();
    let scratch = Scratch::new("assert-equalfile");
    let one = scratch.file("one", "abc\ndef\n");
    let same = scratch.file("same", "abc\ndef\n");
    let other = scratch.file("other", "abc\ndxf\n");
    let short = scratch.file("short", "abc\n");
    let long_a = scratch.file("long_a", &format!("{}a", "x".repeat(250)));
    let long_b = scratch.file("long_b", &format!("{}b", "x".repeat(250)));
    holds(&fx, &format!("assert_equalfile('{one}', '{same}')"));
    assert_eq!(
        failure(&fx, &format!("assert_equalfile('{one}', '{other}')")),
        "['difference at byte 5, line 2 after \"de\" vs \"dx\"']"
    );
    assert_eq!(
        failure(&fx, &format!("assert_equalfile('{one}', '{short}', 'm')")),
        "['m: second file is shorter']"
    );
    assert_eq!(
        failure(&fx, &format!("assert_equalfile('{short}', '{one}')")),
        "['first file is shorter']"
    );
    let tail = "x".repeat(150);
    assert_eq!(
        failure(&fx, &format!("assert_equalfile('{long_a}', '{long_b}')")),
        format!("['difference at byte 250, line 1 after \"{tail}a\" vs \"{tail}b\"']")
    );
    let missing = format!("{}/missing", scratch.0.display());
    assert_eq!(
        failure(&fx, &format!("assert_equalfile('{missing}', '{one}')")),
        format!("['E485: Can''t read file {missing}']")
    );
}
