//! Turning `'shell'`, `'shellcmdflag'`, `'shellxquote'` and `'shellxescape'`
//! into an argument vector, and running one.
//!
//! A port of `test/unit/os/shell_spec.lua`. The four options are process-wide
//! globals that every case here writes, so every case holds the editor lock
//! and puts back what it found — the LuaJIT harness set them in a
//! `before_each` and threw the whole child away afterwards.
//!
//! `os_system` additionally needs the event loop, not just `early_init`:
//! it spawns through libuv and pumps `main_loop` until the child exits. That
//! is why `support::init_editor` runs `event_init` first, exactly as
//! `test/unit/testutil.lua` did.

#![cfg(not(miri))]

use std::ffi::{CStr, CString, c_char, c_int};
use std::ptr;

use c2rust_neovim::main::{p_sh, p_shcf, p_sxe, p_sxq};
use c2rust_neovim::memory::xfree;
use c2rust_neovim::os::shell::system::os_system;
use c2rust_neovim::os::shell::{shell_argv_to_str, shell_build_argv};

use crate::support::{Editor, cstr, editor_lock, internalize};

/// The four options the argument vector is built out of, restored when the
/// case ends. They are raw `char *` the editor normally owns; the values set
/// here are leaked for the duration, which is what makes them safe to hand to
/// code that only reads them.
struct ShellOptions {
    saved: [*mut c_char; 4],
    _editor: Editor,
}

impl ShellOptions {
    /// `/bin/sh -c` with no extra quoting — the spec's `before_each`, and the
    /// only setting under which `os_system` can run at all.
    fn plain() -> Self {
        let options = ShellOptions {
            saved: [p_sh.get(), p_shcf.get(), p_sxq.get(), p_sxe.get()],
            _editor: editor_lock(),
        };
        options.set("/bin/sh", "-c", "", "");
        options
    }

    fn set(&self, sh: &str, shcf: &str, sxq: &str, sxe: &str) {
        p_sh.set(leak(sh));
        p_shcf.set(leak(shcf));
        p_sxq.set(leak(sxq));
        p_sxe.set(leak(sxe));
    }
}

impl Drop for ShellOptions {
    fn drop(&mut self) {
        let [sh, shcf, sxq, sxe] = self.saved;
        p_sh.set(sh);
        p_shcf.set(shcf);
        p_sxq.set(sxq);
        p_sxe.set(sxe);
    }
}

/// An option value that outlives the call that reads it.
fn leak(s: &str) -> *mut c_char {
    CString::new(s)
        .expect("an option value holds no NUL")
        .into_raw()
}

/// `shell_build_argv`, read back as strings and released item by item — the
/// spec freed every element explicitly so that anything not on the heap would
/// crash here rather than silently pass.
fn build_argv(cmd: Option<&str>, extra_args: Option<&str>) -> Vec<String> {
    let cmd = cmd.map(cstr);
    let extra_args = extra_args.map(cstr);
    let as_ptr = |s: &Option<CString>| s.as_ref().map_or(ptr::null(), |s| s.as_ptr());
    // SAFETY: both arguments are this frame's or NULL, and the answer is an
    // owned NULL-terminated vector of owned strings.
    unsafe {
        let argv = shell_build_argv(as_ptr(&cmd), as_ptr(&extra_args));
        let mut words = Vec::new();
        let mut p = argv;
        while !(*p).is_null() {
            words.push(internalize(*p));
            p = p.add(1);
        }
        xfree(argv.cast());
        words
    }
}

/// `shell_argv_to_str` over a borrowed vector.
fn argv_to_str(words: &[&str]) -> String {
    let owned: Vec<CString> = words.iter().copied().map(cstr).collect();
    let mut argv: Vec<*mut c_char> = owned.iter().map(|w| w.as_ptr().cast_mut()).collect();
    argv.push(ptr::null_mut());
    // SAFETY: `argv` is NULL-terminated and every item outlives the call; the
    // answer is an owned string.
    unsafe { internalize(shell_argv_to_str(argv.as_mut_ptr())) }
}

/// Run `cmd` through the shell and hand back (exit code, what it wrote).
fn system(cmd: &str, input: Option<&str>) -> (c_int, String) {
    let cmd = cstr(cmd);
    let mut output: *mut c_char = ptr::null_mut();
    let mut nread: usize = 0;
    // SAFETY: `shell_build_argv`'s vector is what `os_system` consumes;
    // `input` outlives the call and `output`/`nread` are this frame's.
    unsafe {
        let argv = shell_build_argv(cmd.as_ptr(), ptr::null());
        let status = os_system(
            argv,
            input.map_or(ptr::null(), |s| s.as_ptr().cast::<c_char>()),
            input.map_or(0, str::len),
            &raw mut output,
            &raw mut nread,
        );
        let text = if output.is_null() {
            String::new()
        } else {
            let bytes = std::slice::from_raw_parts(output.cast::<u8>(), nread).to_vec();
            xfree(output.cast());
            String::from_utf8(bytes).expect("the fixtures write text")
        };
        (status, text)
    }
}

#[test]
fn a_command_with_no_shell_and_no_flag_is_just_the_shell() {
    let _options = ShellOptions::plain();
    assert_eq!(build_argv(None, None), ["/bin/sh"]);
}

/// The four shapes of the vector: the command brings `'shellcmdflag'` with
/// it, the extra arguments go in front of it, and neither disturbs the other.
#[test]
fn the_command_flag_appears_only_with_a_command() {
    let _options = ShellOptions::plain();
    assert_eq!(
        build_argv(Some("abc  def"), None),
        ["/bin/sh", "-c", "abc  def"]
    );
    assert_eq!(build_argv(None, Some("ghi  jkl")), ["/bin/sh", "ghi  jkl"]);
    assert_eq!(
        build_argv(Some("abc  def"), Some("ghi  jkl")),
        ["/bin/sh", "ghi  jkl", "-c", "abc  def"]
    );
}

/// `'shell'` and `'shellcmdflag'` are word lists, and a quoted run is one
/// word however many spaces are inside it. The quotes come off; the spaces
/// inside `cmd` never split it, because `cmd` is not tokenized at all.
#[test]
fn the_shell_and_its_flag_are_split_on_spaces_and_unquoted() {
    let options = ShellOptions::plain();
    options.set(
        r#"/Program" "Files/zsh -f"#,
        r#"-x -o "sh word split" "-"c"#,
        "",
        "",
    );
    assert_eq!(
        build_argv(Some("abc  def"), Some("ghi  jkl")),
        [
            "/Program Files/zsh",
            "-f",
            "ghi  jkl",
            "-x",
            "-o",
            "sh word split",
            "-c",
            "abc  def",
        ]
    );
}

/// `'shellxquote'` wraps the command and `'shellxescape'` names the
/// characters that get a `^` first. Both apply to the command only, never to
/// the shell or its flag.
#[test]
fn shellxquote_wraps_the_command_and_shellxescape_protects_what_is_inside() {
    let options = ShellOptions::plain();

    options.set("/bin/sh", "-c", "(", r#""&|<>()@^"#);
    assert_eq!(
        build_argv(Some("echo &|<>()@^"), None),
        ["/bin/sh", "-c", "(echo ^&^|^<^>^(^)^@^^)"]
    );

    // A two-character quote closes in mirror image: `"(` opens, `)"` closes.
    // The escaping does *not* follow: it is spelled against a `'shellxquote'`
    // of exactly "(", so with `"(` the same characters go through untouched.
    options.set("/bin/sh", "-c", r#""("#, r#""&|<>()@^"#);
    assert_eq!(
        build_argv(Some("echo -n some text"), None),
        ["/bin/sh", "-c", r#""(echo -n some text)""#]
    );
    assert_eq!(
        build_argv(Some("echo &|<>()@^"), None),
        ["/bin/sh", "-c", r#""(echo &|<>()@^)""#]
    );

    // A quote that is not one of the bracket pairs is used on both ends.
    options.set("/bin/sh", "-c", "\"", "");
    assert_eq!(
        build_argv(Some("echo -n some text"), None),
        ["/bin/sh", "-c", r#""echo -n some text""#]
    );

    // And with neither set the command is passed through untouched.
    options.set("/bin/sh", "-c", "", "");
    assert_eq!(
        build_argv(Some("echo -n some text"), None),
        ["/bin/sh", "-c", "echo -n some text"]
    );
}

/// The report form: every word single-quoted, space-separated, and cut off
/// with an ellipsis at 256 bytes so a long command line cannot flood a
/// message.
#[test]
fn an_argument_vector_reports_as_quoted_words_truncated_at_256_bytes() {
    let _options = ShellOptions::plain();
    assert_eq!(argv_to_str(&[]), "");
    assert_eq!(argv_to_str(&[""]), "''");
    assert_eq!(argv_to_str(&["foo", "", "bar"]), "'foo' '' 'bar'");
    assert_eq!(
        argv_to_str(&["/bin/sh", "-c", "abc  def"]),
        "'/bin/sh' '-c' 'abc  def'"
    );
    assert_eq!(
        argv_to_str(&["abc  def", "ghi  jkl"]),
        "'abc  def' 'ghi  jkl'"
    );

    let long = "x".repeat(999);
    let reported = argv_to_str(&["/bin/sh", "-c", "abc  def", &long]);
    assert_eq!(
        reported,
        format!("'/bin/sh' '-c' 'abc  def' '{}...", "x".repeat(225))
    );
    // The cap is on the buffer, not on the input: 255 bytes plus the
    // terminator, whatever was asked for.
    assert_eq!(reported.len(), 255);
}

#[test]
fn a_command_that_writes_gives_its_output_back() {
    let _options = ShellOptions::plain();
    let (status, output) = system(r#"printf "%s " some text "#, None);
    assert_eq!(status, 0);
    assert_eq!(output, "some text ");
}

#[test]
fn a_command_that_writes_nothing_gives_nothing_back() {
    let _options = ShellOptions::plain();
    let (status, output) = system(r#"printf """#, None);
    assert_eq!(status, 0);
    assert_eq!(output, "");
}

#[test]
fn input_reaches_the_child_on_stdin() {
    let _options = ShellOptions::plain();
    let input = "some text\nsome other text";
    let (status, output) = system("cat -", Some(input));
    assert_eq!(status, 0);
    assert_eq!(output, input);
}

#[test]
fn a_child_that_fails_reports_its_exit_code() {
    let _options = ShellOptions::plain();
    let (status, _) = system("exit 2", None);
    assert_eq!(status, 2);
}

/// Not a spec case: the argument vector the spec's helper freed by hand.
/// `shell_free_argv` is what the editor actually uses, and nothing else here
/// would notice if it stopped walking to the end.
#[test]
fn the_argument_vector_is_one_allocation_per_word_plus_the_vector() {
    let _options = ShellOptions::plain();
    // SAFETY: the vector is `shell_build_argv`'s, released exactly once.
    unsafe {
        let argv = shell_build_argv(cstr("abc def").as_ptr(), cstr("ghi").as_ptr());
        let words: Vec<&CStr> = {
            let mut p = argv;
            let mut words = Vec::new();
            while !(*p).is_null() {
                words.push(CStr::from_ptr(*p));
                p = p.add(1);
            }
            words
        };
        assert_eq!(words.len(), 4, "shell, extra args, flag, command");
        c2rust_neovim::os::shell::shell_free_argv(argv);
    }
}
