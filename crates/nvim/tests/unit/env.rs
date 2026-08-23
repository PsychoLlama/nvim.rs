//! The environment block, as `os/env/` reads and writes it.
//!
//! A port of `test/unit/os/env_spec.lua`.
//!
//! The environment is process-global and `cargo test` runs cases on threads,
//! so this is the file where the LuaJIT harness's fork-per-case was doing the
//! most work. Everything here goes through [`Env`], a
//! [`Sandbox`](crate::support::Sandbox) that takes the editor lock for the
//! case and puts every variable it touched back the way it found it — `$PATH`
//! above all, because `os_setenv_append_path` rewrites it and the filesystem
//! cases in this same binary look programs up along it.
//!
//! Reads use `std::env`, which is the same libc block `uv_os_getenv` sees;
//! writes use the entry points under test, which is what the spec did too
//! (`os_setenv` because "Lua doesn't have setenv").

#![cfg(not(miri))]

use std::ffi::{c_char, c_int};
use std::ptr;

use neovim::os::env::expand::expand_env_esc;
use neovim::os::env::{
    os_env_exists, os_get_hostname, os_get_pid, os_getenv, os_getenv_buf, os_getenv_noalloc,
    os_getenvname_at_index, os_setenv_append_path, os_shell_is_cmdexe,
};
use neovim::types::MAXPATHL;

use crate::support::{Sandbox, cstr, internalize};

/// Success, as `os_setenv`/`os_unsetenv` spell it. Not `OK`: these two answer
/// 0 or -1 rather than the editor's `OK`/`FAIL` pair.
const DONE: c_int = 0;

/// The reads and writes of the environment block this file is about, over a
/// [`Sandbox`] that restores every variable a case touched.
struct Env {
    sandbox: Sandbox,
}

impl Env {
    fn new() -> Self {
        Env {
            sandbox: Sandbox::globals(),
        }
    }

    /// Remember `name`'s current value, once, so the sandbox's drop can put
    /// it back — for a variable the *entry point* is about to rewrite.
    fn remember(&mut self, name: &str) {
        self.sandbox.remember_env(name);
    }

    fn set(&mut self, name: &str, value: &str) -> c_int {
        self.sandbox.set_env(name, value)
    }

    /// `os_setenv` with `overwrite` off: an existing value wins.
    fn set_if_unset(&mut self, name: &str, value: &str) -> c_int {
        self.sandbox.set_env_if_unset(name, value)
    }

    fn unset(&mut self, name: &str) -> c_int {
        self.sandbox.unset_env(name)
    }

    /// What the C library says the variable holds — `os.getenv` in the spec.
    /// `Some("")` and `None` are different answers, which is the distinction
    /// `os_getenv` deliberately loses.
    fn raw(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }

    /// `os_getenv`: an owned copy, or `None` for unset *or empty*.
    fn get(&self, name: &str) -> Option<String> {
        // SAFETY: the name is this frame's; a non-null answer is owned.
        unsafe {
            let value = os_getenv(cstr(name).as_ptr());
            (!value.is_null()).then(|| internalize(value))
        }
    }

    /// `os_getenv_buf` into a `size`-byte buffer.
    fn get_buf(&self, name: &str, size: usize) -> Option<String> {
        let mut buf = vec![0_u8; size];
        // SAFETY: the name is this frame's and `buf` is writable for `size`.
        let answer = unsafe { os_getenv_buf(cstr(name).as_ptr(), buf.as_mut_ptr().cast(), size) };
        answer
            .is_null()
            .then_some(())
            .map_or_else(|| Some(read_c_string(&buf)), |()| None)
    }

    /// `os_getenv_noalloc`, which stages the answer in the shared `NameBuff`.
    fn get_noalloc(&self, name: &str) -> Option<String> {
        // SAFETY: the name is this frame's; the answer points into `NameBuff`,
        // which the editor lock makes ours, and is copied out immediately.
        unsafe {
            let value = os_getenv_noalloc(cstr(name).as_ptr());
            (!value.is_null()).then(|| {
                std::ffi::CStr::from_ptr(value)
                    .to_string_lossy()
                    .into_owned()
            })
        }
    }

    fn exists(&self, name: &str, nonempty: bool) -> bool {
        // SAFETY: the name is this frame's and NUL-terminated.
        unsafe { os_env_exists(cstr(name).as_ptr(), nonempty) }
    }
}

/// The NUL-terminated string at the head of `buf`.
fn read_c_string(buf: &[u8]) -> String {
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    String::from_utf8(buf[..end].to_vec()).expect("the fixtures are text")
}

/// `expand_env_esc` into a `size`-byte buffer, told `dstlen` about it.
fn expand(src: &str, dstlen: c_int, one: bool) -> String {
    let mut buf = vec![0_u8; 512];
    let src = cstr(src);
    // SAFETY: `src` is this frame's and NUL-terminated; `buf` is far larger
    // than the `dstlen` the call is told about, so a case that means to
    // measure truncation still has room to show what actually landed.
    unsafe {
        expand_env_esc(
            src.as_ptr(),
            buf.as_mut_ptr().cast(),
            dstlen,
            false,
            one,
            ptr::null_mut(),
        );
    }
    read_c_string(&buf)
}

/// A name no environment has. Every case that asserts "not found" uses one.
const UNSET: &str = "NVIM_UNIT_TEST_GETENV_NOTFOUND";

/// Names that are not names. The spec listed them twice, once per `nonempty`;
/// they are the same list because `os_env_exists` rejects them before it ever
/// looks at a value.
const NOT_NAMES: [&str; 5] = ["", "      ", "\t", "\n", "AaあB <= very weird name..."];

#[test]
fn nothing_shaped_like_a_name_exists_under_either_rule() {
    let env = Env::new();
    for name in NOT_NAMES {
        assert!(!env.exists(name, false), "{name:?} is not a variable");
        assert!(!env.exists(name, true), "{name:?} is not a variable");
    }
}

/// The two rules `os_env_exists` offers: defined, and defined non-empty. They
/// differ on exactly one state, which is why the spec ran the whole sequence
/// twice.
#[test]
fn an_empty_value_exists_only_under_the_lax_rule() {
    let mut env = Env::new();
    let name = "NVIM_UNIT_TEST_os_env_exists";
    assert!(!env.exists(name, false));
    assert!(!env.exists(name, true));

    assert_eq!(env.set(name, ""), DONE);
    assert!(env.exists(name, false), "an empty value is still defined");
    assert!(!env.exists(name, true), "but it is not non-empty");

    assert_eq!(env.set(name, "foo bar baz ..."), DONE);
    assert!(env.exists(name, false));
    assert!(env.exists(name, true));

    // A one-byte value is the edge: `os_env_exists` reads into a two-byte
    // buffer, so this is the shortest value that does not overflow it.
    assert_eq!(env.set(name, "f"), DONE);
    assert!(env.exists(name, false));
    assert!(env.exists(name, true));
}

#[test]
fn setting_a_variable_makes_it_readable_and_overwrite_governs_the_second_set() {
    let mut env = Env::new();
    let name = "NVIM_UNIT_TEST_SETENV_1N";
    assert_eq!(env.raw(name), None, "the fixture name is unused");
    assert_eq!(env.set(name, "value"), DONE);
    assert_eq!(env.raw(name), Some("value".to_string()));

    // Empty is a value, not an absence, as far as the block is concerned.
    assert_eq!(env.set(name, ""), DONE);
    assert_eq!(env.raw(name), Some(String::new()));
    assert_eq!(env.set(name, "non-empty"), DONE);
    assert_eq!(env.raw(name), Some("non-empty".to_string()));

    let other = "NVIM_UNIT_TEST_SETENV_2N";
    assert_eq!(env.set_if_unset(other, "first"), DONE);
    assert_eq!(env.raw(other), Some("first".to_string()));
    assert_eq!(env.set_if_unset(other, "second"), DONE, "still success");
    assert_eq!(env.raw(other), Some("first".to_string()), "but not applied");
    assert_eq!(env.set(other, "second"), DONE);
    assert_eq!(env.raw(other), Some("second".to_string()));
}

#[test]
fn unsetting_a_variable_leaves_nothing_behind() {
    let mut env = Env::new();
    let name = "TEST_UNSETENV";
    assert_eq!(env.set(name, "TESTVALUE"), DONE);
    assert_eq!(env.unset(name), DONE);
    assert_eq!(env.raw(name), None);
    assert_eq!(env.get(name), None);
    assert!(!env.exists(name, false));
}

/// `#7377`: the directory holding `fname` joins `$PATH`, and the separator is
/// added only when there is not one there already.
#[test]
fn appending_to_the_path_adds_one_separator_and_only_for_an_absolute_name() {
    let mut env = Env::new();
    env.remember("PATH");

    let original = env.raw("PATH").expect("$PATH is set");
    // SAFETY: the name is this frame's and NUL-terminated.
    assert!(unsafe { os_setenv_append_path(cstr("/foo/bar/baz.exe").as_ptr()) });
    assert_eq!(env.raw("PATH"), Some(format!("{original}:/foo/bar")));

    assert_eq!(env.set("PATH", "/a/b/c:"), DONE);
    // SAFETY: as above.
    assert!(unsafe { os_setenv_append_path(cstr("/foo/bar/baz.exe").as_ptr()) });
    assert_eq!(env.raw("PATH"), Some("/a/b/c:/foo/bar".to_string()));

    // A relative name has no directory to add, and `$PATH` is left alone.
    let before = env.raw("PATH");
    // SAFETY: as above.
    assert!(!unsafe { os_setenv_append_path(cstr("foo/bar/baz.exe").as_ptr()) });
    assert_eq!(env.raw("PATH"), before);
}

/// The Windows shell test, which runs everywhere because it is pure string
/// work: `cmd`, `cmd.exe`, either case, or `$COMSPEC` resolving to one.
#[test]
fn only_cmd_and_cmd_exe_look_like_cmd_exe() {
    let mut env = Env::new();
    let is_cmdexe = |sh: &str| {
        // SAFETY: the name is this frame's and NUL-terminated.
        unsafe { os_shell_is_cmdexe(cstr(sh).as_ptr()) }
    };
    for yes in ["cmd.exe", "cmd", "CMD.EXE", "CMD"] {
        assert!(is_cmdexe(yes), "{yes:?}");
    }
    for no in ["", "powershell", " cmd.exe ", "cm", "md", "cmd.ex"] {
        assert!(!is_cmdexe(no), "{no:?}");
    }

    // `$COMSPEC` is followed, and only its last component counts.
    env.set("COMSPEC", "/foo/bar/cmd.exe");
    assert!(is_cmdexe("$COMSPEC"));
    env.set("COMSPEC", "/foo/bar/cmd");
    assert!(
        !is_cmdexe("$COMSPEC"),
        "the bare name is only accepted when it is `sh` itself"
    );

    // The spec also claimed `C:\system32\cmd.exe` was recognised. It is not,
    // on this platform: `path_tail` splits on `/` and nothing else, so the
    // whole string is the last component. The spec's assertion never ran —
    // it wrote `$COMSPEC` with `overwrite` off, so the value under test was
    // still the `/foo/bar/cmd.exe` set on the line before.
    env.set("COMSPEC", r"C:\system32\cmd.exe");
    assert!(
        !is_cmdexe("$COMSPEC"),
        "a backslash is not a separator here"
    );
}

/// The three readers answer the same questions, and each loses something
/// different: `os_getenv` allocates and cannot say "empty", `os_getenv_buf`
/// truncates to the caller's buffer, `os_getenv_noalloc` truncates to
/// `NameBuff`.
#[test]
fn the_three_readers_agree_about_what_is_there() {
    let mut env = Env::new();
    let name = "NVIM_UNIT_TEST_GETENV_1N";
    const BUFSIZE: usize = 200;

    assert_eq!(env.get(name), None);
    assert_eq!(env.get_buf(name, BUFSIZE), None);
    assert_eq!(env.get_noalloc(name), None);

    for value in ["NVIM_UNIT_TEST_GETENV_1V", "z", &"x".repeat(256)] {
        assert_eq!(env.set(name, value), DONE);
        assert_eq!(env.get(name).as_deref(), Some(value));
        assert_eq!(env.get_noalloc(name).as_deref(), Some(value));
    }

    // Empty reads as absent through all three: the value is there (the
    // previous case proves that) but none of these can say so.
    assert_eq!(env.set(name, ""), DONE);
    assert_eq!(env.raw(name), Some(String::new()));
    assert_eq!(env.get(name), None);
    assert_eq!(env.get_buf(name, BUFSIZE), None);
    assert_eq!(env.get_noalloc(name), None);

    assert_eq!(env.get(UNSET), None);
    assert_eq!(env.get_buf(UNSET, BUFSIZE), None);
    assert_eq!(env.get_noalloc(UNSET), None);
}

#[test]
fn a_value_too_big_for_the_buffer_is_truncated_to_fit() {
    let mut env = Env::new();
    let name = "NVIM_UNIT_TEST_GETENV_1N";
    const BUFSIZE: usize = 200;

    let long = "y".repeat(BUFSIZE + 10);
    assert_eq!(env.set(name, &long), DONE);
    assert_eq!(
        env.get_buf(name, BUFSIZE),
        Some("y".repeat(BUFSIZE - 1)),
        "the buffer holds BUFSIZE - 1 bytes plus a terminator"
    );
    // The allocating reader has no such limit.
    assert_eq!(env.get(name).as_deref(), Some(long.as_str()));

    // `os_getenv_noalloc`'s buffer is `NameBuff`, so its limit is MAXPATHL.
    let maxpathl = MAXPATHL as usize;
    let longer = "y".repeat(maxpathl);
    assert_eq!(env.set(name, &longer), DONE);
    assert_eq!(env.get_noalloc(name), Some("y".repeat(maxpathl - 1)));
}

#[test]
fn the_environment_block_enumerates_by_index_and_stops_at_the_end() {
    let mut env = Env::new();
    let name = "NVIM_UNIT_TEST_GETENVNAME_AT_INDEX_1N";
    assert_eq!(env.set(name, "value"), DONE);

    let mut names = Vec::new();
    for i in 0.. {
        // SAFETY: the answer is NULL past the end, and owned otherwise.
        let entry = unsafe { os_getenvname_at_index(i) };
        if entry.is_null() {
            break;
        }
        // SAFETY: owned and NUL-terminated.
        names.push(unsafe { internalize(entry) });
    }
    assert!(!names.is_empty());
    assert!(names.contains(&name.to_string()), "{name} is in {names:?}");
    assert!(
        !names.iter().any(|n| n.contains('=')),
        "the name stops at the `=`"
    );

    // Past the end, however far past: the bound is checked by walking the
    // block, not by trusting the index.
    for out_of_bounds in [
        10_000,
        u64::from(u32::MAX) as usize,
        18_446_744_073_709_000_000,
    ] {
        // SAFETY: any index is accepted.
        assert!(unsafe { os_getenvname_at_index(out_of_bounds) }.is_null());
    }
}

#[test]
fn the_process_reports_its_own_pid() {
    let _env = Env::new();
    // SAFETY: `getpid` takes no arguments.
    assert_eq!(os_get_pid(), i64::from(unsafe { libc::getpid() }));
    assert!(os_get_pid() > 0);
}

#[test]
fn the_hostname_is_what_uname_says() {
    let _env = Env::new();
    let mut buf = [0_u8; 256];
    // SAFETY: `buf` is writable for its own length.
    unsafe { os_get_hostname(buf.as_mut_ptr().cast::<c_char>(), buf.len()) };

    // SAFETY: `utsname` is plain data that `uname` fills in.
    let expected = unsafe {
        let mut info: libc::utsname = std::mem::zeroed();
        assert!(libc::uname(&raw mut info) >= 0);
        std::ffi::CStr::from_ptr(info.nodename.as_ptr())
            .to_string_lossy()
            .into_owned()
    };
    assert_eq!(read_c_string(&buf), expected);
}

#[test]
fn a_variable_expands_in_both_spellings() {
    let mut env = Env::new();
    let name = "NVIM_UNIT_TEST_EXPAND_ENV_ESCN";
    let value = "NVIM_UNIT_TEST_EXPAND_ENV_ESCV";
    assert_eq!(env.set(name, value), DONE);

    let expected = format!("{value}/test");
    assert_eq!(expand(&format!("${name}/test"), 255, true), expected);
    assert_eq!(expand(&format!("${{{name}}}/test"), 255, true), expected);
}

/// `one` says "expand the first thing only", which is what a caller handling
/// a single file name wants; with it off every `~` in the string goes.
#[test]
fn the_one_flag_decides_whether_the_second_tilde_expands() {
    let _env = Env::new();
    let home = expand("~", 255, true);
    assert!(home.starts_with('/'), "~ expands to a path: {home:?}");

    assert_eq!(
        expand("~/foo ~ foo", 255, true),
        format!("{home}/foo ~ foo")
    );
    assert_eq!(
        expand("~/foo ~ foo", 255, false),
        format!("{home}/foo {home} foo")
    );
}

/// `#3725`: `~user` followed by a long path used to run off the end. The
/// assertion the spec could make is that the answer is a sane length; what
/// matters is that the call returns at all.
#[test]
fn a_long_path_under_a_named_home_does_not_run_away() {
    let _env = Env::new();
    let user = std::env::var("USER").expect("$USER names the user running the tests");
    let expanded = expand(
        &format!("~{user}/Vcs/django-rest-framework/rest_framework/renderers.py"),
        256,
        false,
    );
    assert!(expanded.len() > 56, "{expanded:?}");
    assert!(expanded.len() < 256, "{expanded:?}");
}

/// `dstlen` is a hard limit on the answer, and it is honoured on both paths:
/// the plain copy, and the one that would have expanded a variable. A
/// variable whose value does not fit is left unexpanded rather than cut in
/// half.
#[test]
fn dstlen_bounds_the_answer_with_or_without_an_expansion() {
    let mut env = Env::new();
    let plain = "this is a very long thing that will not fit";
    assert_eq!(expand(plain, 5, true), plain[..4]);

    let name = "NVIM_UNIT_TEST_EXPAND_ENV_ESC_DSTLENN";
    assert_eq!(env.set(name, "NVIM_UNIT_TEST_EXPAND_ENV_ESC_DSTLENV"), DONE);
    let input = format!("${name}/even more stuff");
    assert_eq!(expand(&input, 5, true), input[..4]);
}
