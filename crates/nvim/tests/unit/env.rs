//! The environment block, as `os/env/` reads and writes it.
//!
//! A port of `test/unit/os/env_spec.lua`.
//!
//! The environment is process-global and `cargo test` runs cases on threads,
//! so this is the file where the LuaJIT harness's fork-per-case was doing the
//! most work. Everything here goes through [`Env`], a
//! [`Sandbox`](crate::support::Sandbox) that takes the editor lock for the
//! case and puts every variable it touched back the way it found it — `$PATH`
//! above all, because `setenv_append_path` rewrites it and the filesystem
//! cases in this same binary look programs up along it.
//!
//! Reads use `std::env`, which is the block the editor reads too; writes use
//! the entry points under test, which is what the spec did too (`env_set`
//! because "Lua doesn't have setenv").

#![cfg(not(miri))]

use std::ffi::{CStr, c_int};

use neovim::env::expand::expand_env_bounded;
use neovim::os::env::{
    env_exists, env_get_bounded, env_get_within, env_name_at, getenv_owned, hostname_into,
    process_id, setenv_append_path, shell_is_cmdexe,
};
use neovim::path::MAXPATHL;

use crate::support::{Sandbox, cstr};

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

    fn set(&mut self, name: &str, value: &str) -> bool {
        self.sandbox.set_env(name, value)
    }

    /// An existing value wins.
    fn set_if_unset(&mut self, name: &str, value: &str) -> bool {
        self.sandbox.set_env_if_unset(name, value)
    }

    fn unset(&mut self, name: &str) -> bool {
        self.sandbox.unset_env(name)
    }

    /// What the C library says the variable holds — `os.getenv` in the spec.
    /// `Some("")` and `None` are different answers, which is the distinction
    /// `getenv_owned` deliberately loses.
    fn raw(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }

    /// `getenv_owned`: an owned copy, or `None` for unset *or empty*.
    fn get(&self, name: &str) -> Option<String> {
        getenv_owned(&cstr(name)).map(|value| text(&value))
    }

    /// `env_get_within` a `size`-byte buffer.
    fn get_buf(&self, name: &str, size: usize) -> Option<String> {
        env_get_within(&cstr(name), size).map(|value| text(&value))
    }

    /// `env_get_bounded`, which cuts to `MAXPATHL`; empty reads as `None`.
    fn get_noalloc(&self, name: &str) -> Option<String> {
        let value = env_get_bounded(&cstr(name));
        (!value.is_empty()).then(|| text(&value))
    }

    fn exists(&self, name: &str, nonempty: bool) -> bool {
        env_exists(&cstr(name), nonempty)
    }
}

/// Bytes the fixtures wrote, as text.
fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("the fixtures are text")
}

/// `expand_env_bounded`, told `dstlen` about its destination.
fn expand(src: &str, dstlen: c_int, one: bool) -> String {
    let dstlen = usize::try_from(dstlen).expect("a positive bound");
    let out = expand_env_bounded(src.as_bytes(), dstlen, false, one, None);
    String::from_utf8(out).expect("the fixtures are text")
}

/// A name no environment has. Every case that asserts "not found" uses one.
const UNSET: &str = "NVIM_UNIT_TEST_GETENV_NOTFOUND";

/// Names that are not names. The spec listed them twice, once per `nonempty`;
/// they are the same list because none of them is ever set, so `env_exists`
/// never gets as far as a value.
const NOT_NAMES: [&str; 5] = ["", "      ", "\t", "\n", "AaあB <= very weird name..."];

#[test]
fn nothing_shaped_like_a_name_exists_under_either_rule() {
    let env = Env::new();
    for name in NOT_NAMES {
        assert!(!env.exists(name, false), "{name:?} is not a variable");
        assert!(!env.exists(name, true), "{name:?} is not a variable");
    }
}

/// The two rules `env_exists` offers: defined, and defined non-empty. They
/// differ on exactly one state, which is why the spec ran the whole sequence
/// twice.
#[test]
fn an_empty_value_exists_only_under_the_lax_rule() {
    let mut env = Env::new();
    let name = "NVIM_UNIT_TEST_env_exists";
    assert!(!env.exists(name, false));
    assert!(!env.exists(name, true));

    assert!(env.set(name, ""));
    assert!(env.exists(name, false), "an empty value is still defined");
    assert!(!env.exists(name, true), "but it is not non-empty");

    assert!(env.set(name, "foo bar baz ..."));
    assert!(env.exists(name, false));
    assert!(env.exists(name, true));

    // A one-byte value: upstream's `os_env_exists` read into a two-byte
    // buffer, so this was the shortest value that did not overflow it.
    assert!(env.set(name, "f"));
    assert!(env.exists(name, false));
    assert!(env.exists(name, true));
}

#[test]
fn setting_a_variable_makes_it_readable_and_overwrite_governs_the_second_set() {
    let mut env = Env::new();
    let name = "NVIM_UNIT_TEST_SETENV_1N";
    assert_eq!(env.raw(name), None, "the fixture name is unused");
    assert!(env.set(name, "value"));
    assert_eq!(env.raw(name), Some("value".to_string()));

    // Empty is a value, not an absence, as far as the block is concerned.
    assert!(env.set(name, ""));
    assert_eq!(env.raw(name), Some(String::new()));
    assert!(env.set(name, "non-empty"));
    assert_eq!(env.raw(name), Some("non-empty".to_string()));

    let other = "NVIM_UNIT_TEST_SETENV_2N";
    assert!(env.set_if_unset(other, "first"));
    assert_eq!(env.raw(other), Some("first".to_string()));
    assert!(env.set_if_unset(other, "second"), "still success");
    assert_eq!(env.raw(other), Some("first".to_string()), "but not applied");
    assert!(env.set(other, "second"));
    assert_eq!(env.raw(other), Some("second".to_string()));
}

#[test]
fn unsetting_a_variable_leaves_nothing_behind() {
    let mut env = Env::new();
    let name = "TEST_UNSETENV";
    assert!(env.set(name, "TESTVALUE"));
    assert!(env.unset(name));
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
    assert!(setenv_append_path(&cstr("/foo/bar/baz.exe")).is_ok());
    assert_eq!(env.raw("PATH"), Some(format!("{original}:/foo/bar")));

    assert!(env.set("PATH", "/a/b/c:"));
    assert!(setenv_append_path(&cstr("/foo/bar/baz.exe")).is_ok());
    assert_eq!(env.raw("PATH"), Some("/a/b/c:/foo/bar".to_string()));

    // A relative name has no directory to add, and `$PATH` is left alone.
    let before = env.raw("PATH");
    assert!(setenv_append_path(&cstr("foo/bar/baz.exe")).is_err());
    assert_eq!(env.raw("PATH"), before);
}

/// The Windows shell test, which runs everywhere because it is pure string
/// work: `cmd`, `cmd.exe`, either case, or `$COMSPEC` resolving to one.
#[test]
fn only_cmd_and_cmd_exe_look_like_cmd_exe() {
    let mut env = Env::new();
    let is_cmdexe = |sh: &str| shell_is_cmdexe(sh.as_bytes());
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
/// different: `getenv_owned` cannot say "empty", `env_get_within` truncates
/// to the caller's room, `env_get_bounded` truncates to
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
        assert!(env.set(name, value));
        assert_eq!(env.get(name).as_deref(), Some(value));
        assert_eq!(env.get_noalloc(name).as_deref(), Some(value));
    }

    // Empty reads as absent through all three: the value is there (the
    // previous case proves that) but none of these can say so.
    assert!(env.set(name, ""));
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
    assert!(env.set(name, &long));
    assert_eq!(
        env.get_buf(name, BUFSIZE),
        Some("y".repeat(BUFSIZE - 1)),
        "the buffer holds BUFSIZE - 1 bytes plus a terminator"
    );
    // The allocating reader has no such limit.
    assert_eq!(env.get(name).as_deref(), Some(long.as_str()));

    // `env_get_bounded`'s room is `NameBuff`'s, so its limit is MAXPATHL.
    let maxpathl = MAXPATHL as usize;
    let longer = "y".repeat(maxpathl);
    assert!(env.set(name, &longer));
    assert_eq!(env.get_noalloc(name), Some("y".repeat(maxpathl - 1)));
}

#[test]
fn the_environment_block_enumerates_by_index_and_stops_at_the_end() {
    let mut env = Env::new();
    let name = "NVIM_UNIT_TEST_GETENVNAME_AT_INDEX_1N";
    assert!(env.set(name, "value"));

    let mut names = Vec::new();
    for i in 0.. {
        let Some(entry) = env_name_at(i) else {
            break;
        };
        names.push(text(&entry));
    }
    assert!(!names.is_empty());
    assert!(names.contains(&name.to_string()), "{name} is in {names:?}");
    assert!(
        !names.iter().any(|n| n.contains('=')),
        "the name stops at the `=`"
    );

    // Past the end, however far past.
    for out_of_bounds in [
        10_000,
        u64::from(u32::MAX) as usize,
        18_446_744_073_709_000_000,
    ] {
        assert!(env_name_at(out_of_bounds).is_none());
    }
}

#[test]
fn the_process_reports_its_own_pid() {
    let _env = Env::new();
    // SAFETY: `getpid` takes no arguments.
    assert_eq!(process_id(), i64::from(unsafe { libc::getpid() }));
    assert!(process_id() > 0);
}

#[test]
fn the_hostname_is_what_uname_says() {
    let _env = Env::new();
    let mut buf = [0_u8; 256];
    hostname_into(&mut buf);

    // SAFETY: `utsname` is plain data that `uname` fills in.
    let expected = unsafe {
        let mut info: libc::utsname = std::mem::zeroed();
        assert!(libc::uname(&raw mut info) >= 0);
        CStr::from_ptr(info.nodename.as_ptr())
            .to_string_lossy()
            .into_owned()
    };
    let got = CStr::from_bytes_until_nul(&buf).expect("terminated");
    assert_eq!(got.to_string_lossy(), expected);
}

#[test]
fn a_variable_expands_in_both_spellings() {
    let mut env = Env::new();
    let name = "NVIM_UNIT_TEST_EXPAND_ENV_ESCN";
    let value = "NVIM_UNIT_TEST_EXPAND_ENV_ESCV";
    assert!(env.set(name, value));

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
    assert!(env.set(name, "NVIM_UNIT_TEST_EXPAND_ENV_ESC_DSTLENV"));
    let input = format!("${name}/even more stuff");
    assert_eq!(expand(&input, 5, true), input[..4]);
}
