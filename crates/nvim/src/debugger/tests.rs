//! Breakpoints and profiling points end to end, through the commands that
//! keep them: `:breakadd` of each kind, `:breaklist`'s rendering of them,
//! `:breakdel` by number, by name and of all, the lookups a function call
//! and a watch expression make, and `:profile`'s points beside them.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::{dbg_find_breakpoint_named, has_profiling_named};
use crate::eval::test_fixture::{Fixture, Scratch};

/// What `:breaklist` says, a line per entry, with the numbers dropped: they
/// count every breakpoint the process ever made.
fn listed(fx: &Fixture) -> String {
    fx.eval(
        r#"map(split(execute('breaklist'), "\n"), 'substitute(v:val, ''^ *\d\+'', ''#'', '''')')"#,
    )
}

#[test]
fn breakpoints_are_added_listed_and_deleted() {
    let fx = Fixture::new();
    fx.run("breakdel *");
    assert_eq!(listed(&fx), "['No breakpoints defined']");
    fx.run("breakadd func EeTarget");
    fx.run("breakadd func 3 g:EeOther");
    fx.run("let g:ee_watch = 1");
    fx.run("breakadd expr g:ee_watch");
    assert_eq!(
        listed(&fx),
        "['#  func EeTarget  line 1', '#  func EeOther  line 3', \
         '#  expr g:ee_watch']"
    );
    // The lookups the callers make.
    assert_eq!(dbg_find_breakpoint_named(false, c"EeTarget", 0), 1);
    assert_eq!(dbg_find_breakpoint_named(false, c"EeTarget", 1), 0);
    assert_eq!(dbg_find_breakpoint_named(false, c"EeOther", 0), 3);
    assert_eq!(dbg_find_breakpoint_named(false, c"Nothing", 0), 0);
    // A function's breakpoint is not a file's.
    assert_eq!(dbg_find_breakpoint_named(true, c"EeTarget", 0), 0);
    // The watch has not moved, so it stops nothing; once it moves it stops
    // at the line asked about, once.
    assert_eq!(dbg_find_breakpoint_named(false, c"Nothing", 4), 0);
    fx.run("let g:ee_watch = 2");
    assert_eq!(dbg_find_breakpoint_named(false, c"Nothing", 4), 4);
    assert_eq!(dbg_find_breakpoint_named(false, c"Nothing", 4), 0);

    // By name: the closest line, then the next.
    fx.run("breakadd func 9 EeTarget");
    fx.run("breakdel func EeTarget");
    assert!(!listed(&fx).contains("EeTarget  line 1"), "{}", listed(&fx));
    assert!(listed(&fx).contains("EeTarget  line 9"), "{}", listed(&fx));
    fx.run("breakdel func 9 EeTarget");
    assert!(!listed(&fx).contains("EeTarget"), "{}", listed(&fx));
    fx.run("breakdel expr g:ee_watch");
    assert!(!listed(&fx).contains("expr"), "{}", listed(&fx));

    for (cmd, err) in [
        ("breakadd", "E475"),
        ("breakadd func", "E475"),
        ("breakadd func F()", "E475"),
        ("breakadd nonsense x", "E475"),
        ("breakdel func NotThere", "E161"),
    ] {
        fx.run("let v:errmsg = ''");
        fx.run(&format!("silent! {cmd}"));
        let msg = fx.eval("v:errmsg");
        assert!(msg.starts_with(&format!("'{err}")), "{cmd}: {msg}");
    }
    fx.run("breakdel *");
    assert_eq!(listed(&fx), "['No breakpoints defined']");
    fx.run("unlet g:ee_watch");
}

#[test]
#[cfg_attr(miri, ignore = "`:breakdel {nr}` reads the number with libc's `atoi`")]
fn a_breakpoint_is_deleted_by_its_number() {
    let fx = Fixture::new();
    fx.run("breakdel *");
    // The next one made is the highest.
    fx.run("breakadd func EeFirst");
    fx.run("breakadd func EeLast");
    fx.run(r#"execute 'breakdel' matchstr(split(execute('breaklist'), "\n")[-1], '\d\+')"#);
    assert_eq!(listed(&fx), "['#  func EeFirst  line 1']");
    for cmd in ["breakdel 99999", "breakdel 0"] {
        fx.run("let v:errmsg = ''");
        fx.run(&format!("silent! {cmd}"));
        let msg = fx.eval("v:errmsg");
        assert!(msg.starts_with("'E161"), "{cmd}: {msg}");
    }
    fx.run("breakdel *");
}

#[test]
#[cfg_attr(miri, ignore = "a file name is made full through libuv")]
fn a_file_breakpoint_names_the_full_path() {
    let fx = Fixture::new();
    let scratch = Scratch::new("filebreak");
    let script = scratch.file("x.vim", "");
    fx.run("breakdel *");
    fx.run(&format!("breakadd file 2 {script}"));
    fx.run("breakadd file */dir/y.vim");
    fx.run("breakadd file 7 *.vim");
    assert_eq!(
        listed(&fx),
        format!(
            "['#  file {script}  line 2', '#  file */dir/y.vim  line 1', '#  file *.vim  line 7']"
        )
    );
    let full = std::ffi::CString::new(script).expect("no NUL");
    assert_eq!(dbg_find_breakpoint_named(true, &full, 0), 2);
    assert_eq!(dbg_find_breakpoint_named(true, &full, 2), 7);
    assert_eq!(dbg_find_breakpoint_named(true, c"/a/dir/y.vim", 0), 1);
    assert_eq!(dbg_find_breakpoint_named(true, c"/elsewhere/x.c", 0), 0);
    assert_eq!(dbg_find_breakpoint_named(false, c"x.vim", 0), 0);
    fx.run("breakdel *");
}

#[test]
#[cfg_attr(miri, ignore = "the profiler reads the clock through libuv")]
fn profiling_points_sit_beside_breakpoints() {
    let fx = Fixture::new();
    let scratch = Scratch::new("profpoints");
    let log = scratch.file("profile.log", "");
    fx.run("breakdel *");
    fx.run(&format!("profile start {log}"));
    fx.run("profdel *");
    fx.run("breakadd func EeBoth");
    fx.run("profile func EeProf*");
    fx.run("profile! file /x/*.vim");
    // Not a breakpoint, so not listed.
    assert_eq!(listed(&fx), "['#  func EeBoth  line 1']");
    assert!(has_profiling_named(false, c"EeProfiled"));
    assert!(!has_profiling_named(false, c"EeBoth"));
    assert!(has_profiling_named(true, c"/x/a.vim"));
    assert!(!has_profiling_named(true, c"/y/a.vim"));
    assert_eq!(dbg_find_breakpoint_named(false, c"EeProfiled", 0), 0);
    fx.run("profdel func EeProf*");
    assert!(!has_profiling_named(false, c"EeProfiled"));
    for (cmd, err) in [("profdel func EeNone", "E161"), ("profile func", "E475")] {
        fx.run("let v:errmsg = ''");
        fx.run(&format!("silent! {cmd}"));
        let msg = fx.eval("v:errmsg");
        assert!(msg.starts_with(&format!("'{err}")), "{cmd}: {msg}");
    }
    fx.run("profdel *");
    fx.run("profile stop");
    fx.run("breakdel *");
}

#[test]
#[cfg_attr(miri, ignore = "`:source` resolves its file through libuv")]
fn a_function_breakpoint_is_found_by_its_call() {
    let fx = Fixture::new();
    fx.run("breakdel *");
    let scratch = Scratch::new("snrbreak");
    let script = scratch.file(
        "local.vim",
        "function! s:EeLocal()\n  return 1\nendfunction\nlet g:EeLocalRef = function('s:EeLocal')\n",
    );
    fx.run(&format!("source {script}"));
    // A script-local function is matched by its `<SNR>` spelling.
    fx.run("breakadd func <SNR>*EeLocal");
    let name = fx.eval("get(g:EeLocalRef, 'name')");
    let name = name.trim_matches('\'');
    let snr = name.strip_prefix("<SNR>").expect("a script-local name");
    let mut internal = vec![0x80_u8, 253, 83];
    internal.extend_from_slice(snr.as_bytes());
    internal.push(0);
    let internal = std::ffi::CStr::from_bytes_with_nul(&internal).expect("one NUL");
    assert_eq!(dbg_find_breakpoint_named(false, internal, 0), 1);
    fx.run("breakdel *");
    fx.run("unlet g:EeLocalRef");
}
