//! File names: taking them apart, making them absolute, and comparing them.
//!
//! A port of `test/unit/path_spec.lua`. `path/` had no test in this tree at
//! all; the spec was its only oracle.
//!
//! Almost everything here is about a *relative* name, which means almost
//! everything here depends on the working directory — a process-wide setting
//! that several cases also change. So every case takes the editor lock
//! through [`Sandbox`](crate::support::Sandbox), which gives it a private
//! directory to stand in and puts the old working directory — and anything
//! it wrote to the environment — back on the way out. The LuaJIT harness
//! forked a child per case and could `chdir` as freely as it liked.

#![cfg(not(miri))]

use std::ffi::{CStr, c_char, c_int};
use std::path::Path;
use std::ptr;

use neovim::main::p_fic;
use neovim::path::{
    append_path, invocation_path_tail, kBothFilesMissing, kDifferentFiles, kEqualFileNames,
    kEqualFiles, kOneFileMissing, path_fix_case, path_full_compare, path_full_dir_name,
    path_guess_exepath, path_is_absolute, path_next_component, path_shorten_fname, path_tail,
    path_tail_with_sep, path_try_shorten_fname, path_with_extension, path_with_url, vim_full_name,
};
use neovim::types::{FAIL, OK};

use crate::support::Sandbox;

/// A buffer for an entry point that writes a path into one, read back as the
/// NUL-terminated string it left there.
struct Buffer(Vec<c_char>);

impl Buffer {
    fn new(len: usize) -> Buffer {
        Buffer(vec![0; len])
    }

    fn as_mut_ptr(&mut self) -> *mut c_char {
        self.0.as_mut_ptr()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn text(&self) -> String {
        let end = self.0.iter().position(|&b| b == 0).expect("terminated");
        String::from_utf8(self.0[..end].iter().map(|&b| b as u8).collect())
            .expect("the fixtures are text")
    }
}

/// A NUL-terminated, writable copy of `s` — several entry points below take
/// `*mut c_char` even when they only read it, and `path_fix_case` really does
/// write through it.
fn writable(s: &str) -> Vec<c_char> {
    s.bytes()
        .map(|b| b as c_char)
        .chain(std::iter::once(0))
        .collect()
}

/// The NUL-terminated string at `p`, which must be inside something alive.
///
/// # Safety
/// `p` must be a live NUL-terminated string.
unsafe fn borrowed(p: *const c_char) -> String {
    assert!(!p.is_null(), "the answer is never NULL here");
    // SAFETY: the caller's contract.
    unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned()
}

#[test]
fn a_directory_name_is_resolved_against_the_working_directory() {
    let sandbox = Sandbox::dir("path-full-dir-name");
    let real = sandbox.mkdir("unit-test-directory");
    std::os::unix::fs::symlink(&real, sandbox.path("unit-test-symlink")).expect("a link");

    let resolve = |directory: &str| {
        let mut name = writable(directory);
        let mut buf = Buffer::new(sandbox.as_str().len() + 64);
        // SAFETY: both buffers are this frame's and `buf` is `len` bytes.
        let result = unsafe { path_full_dir_name(name.as_mut_ptr(), buf.as_mut_ptr(), buf.len()) };
        (result, buf.text())
    };

    // The empty name is the working directory.
    assert_eq!(resolve(""), (OK, sandbox.as_str().to_string()));

    let parent = sandbox
        .root()
        .parent()
        .expect("the sandbox has a parent")
        .to_str()
        .expect("text")
        .to_string();
    assert_eq!(resolve(".."), (OK, parent));

    // A link is resolved, absolutely or relatively; a name that does not
    // exist is still made absolute.
    let expected = real.to_str().expect("text").to_string();
    for name in [
        "unit-test-directory",
        "unit-test-symlink",
        real.to_str().expect("text"),
        sandbox.path("unit-test-symlink").to_str().expect("text"),
    ] {
        assert_eq!(resolve(name), (OK, expected.clone()), "{name}");
    }
    assert_eq!(
        resolve("does-not-exist"),
        (OK, format!("{}/does-not-exist", sandbox.as_str()))
    );

    // ...but an absolute name that does not exist cannot be resolved at all.
    assert_eq!(resolve("/does_not_exist").0, FAIL);
}

/// `#28786`: an absolute directory still resolves when the process is
/// standing in a directory that has been removed underneath it.
#[test]
fn an_absolute_directory_resolves_from_a_working_directory_that_is_gone() {
    let sandbox = Sandbox::dir("path-cwd-gone");
    let real = sandbox.mkdir("unit-test-directory");
    std::os::unix::fs::symlink(&real, sandbox.path("unit-test-symlink")).expect("a link");
    let doomed = sandbox.mkdir("dir-to-remove");
    std::env::set_current_dir(&doomed).expect("standing in the doomed directory");
    std::fs::remove_dir(&doomed).expect("removing it underneath ourselves");

    let expected = real.to_str().expect("text").to_string();
    for name in [
        real.to_str().expect("text"),
        sandbox.path("unit-test-symlink").to_str().expect("text"),
    ] {
        let mut name = writable(name);
        let mut buf = Buffer::new(sandbox.as_str().len() + 64);
        // SAFETY: both buffers are this frame's.
        let result = unsafe { path_full_dir_name(name.as_mut_ptr(), buf.as_mut_ptr(), buf.len()) };
        assert_eq!((result, buf.text()), (OK, expected.clone()));
    }
}

/// The five answers `path_full_compare` has, which depend on whether each
/// name exists as well as on what it says.
#[test]
fn comparing_two_names_says_which_of_the_five_relations_holds() {
    let sandbox = Sandbox::dir("path-full-compare");
    sandbox.touch("f1.o");
    sandbox.touch("f2.o");

    let compare = |a: &str, b: &str, checkname: bool| {
        let (mut a, mut b) = (writable(a), writable(b));
        // SAFETY: both names are this frame's and NUL-terminated.
        unsafe { path_full_compare(a.as_mut_ptr(), b.as_mut_ptr(), checkname, true) }
    };

    assert_eq!(compare("f1.o", "f1.o", false), kEqualFiles);
    assert_eq!(compare("f1.o", "f2.o", false), kDifferentFiles);
    assert_eq!(compare("f2.o", "f1.o", false), kDifferentFiles);
    assert_eq!(compare("f1.o", "null.txt", false), kOneFileMissing);
    assert_eq!(compare("null.txt", "f1.o", false), kOneFileMissing);
    assert_eq!(compare("null.txt", "null.txt", false), kBothFilesMissing);
    // `checkname` is what turns two missing files with one name into a match.
    assert_eq!(compare("null.txt", "null.txt", true), kEqualFileNames);
}

/// The three ways of splitting a name at its last separator, and what each
/// keeps.
#[test]
fn the_tail_of_a_name_is_what_follows_its_last_separator() {
    let _sandbox = Sandbox::dir("path-tails");
    let tail = |s: &str| {
        let name = writable(s);
        // SAFETY: `name` outlives the borrow, and the answer points into it.
        unsafe { borrowed(path_tail(name.as_ptr())) }
    };
    assert_eq!(tail("directory/file.txt"), "file.txt");
    assert_eq!(tail("directory/"), "", "a trailing separator has no tail");

    let with_sep = |s: &str| {
        let mut name = writable(s);
        // SAFETY: as above.
        unsafe { borrowed(path_tail_with_sep(name.as_mut_ptr())) }
    };
    assert_eq!(with_sep("directory///file.txt"), "///file.txt");
    assert_eq!(with_sep(""), "");
    assert_eq!(with_sep("some/directory/"), "/");
    assert_eq!(with_sep("/file.txt"), "file.txt", "a leading one is cut");
    assert_eq!(with_sep("/"), "");
    assert_eq!(with_sep("file.txt"), "file.txt");

    let next = |s: &str| {
        let name = writable(s);
        // SAFETY: as above.
        unsafe { borrowed(path_next_component(name.as_ptr())) }
    };
    assert_eq!(next("some/directory/file.txt"), "directory/file.txt");
    assert_eq!(next("file.txt"), "", "nothing follows a name with no sep");
}

/// `invocation_path_tail` is `path_tail` for a command line: the arguments
/// are not part of the executable's name, so a separator inside them does
/// not move the split.
#[test]
fn an_invocations_tail_stops_at_the_first_space() {
    let _sandbox = Sandbox::dir("path-invocation");
    let split = |s: &str| {
        let invocation = writable(s);
        let mut len: usize = 0;
        // SAFETY: `invocation` outlives both calls, and `len` is this
        // frame's; a null `len` is the documented "do not report it".
        unsafe {
            let with_len = invocation_path_tail(invocation.as_ptr(), &raw mut len);
            let without = invocation_path_tail(invocation.as_ptr(), ptr::null_mut());
            assert_eq!(
                borrowed(with_len),
                borrowed(without),
                "the answer does not depend on being asked for the length"
            );
            (borrowed(with_len), len)
        }
    };

    assert_eq!(split("directory/exe a b c"), ("exe a b c".to_string(), 3));
    assert_eq!(split("/usr/bin/z a b"), ("z a b".to_string(), 1));
    assert_eq!(
        split("exe a/b\\c"),
        ("exe a/b\\c".to_string(), 3),
        "a separator in the arguments is not the executable's"
    );
    assert_eq!(
        split("exe-a+b_c[]()|#!@$%^&*").0,
        "exe-a+b_c[]()|#!@$%^&*",
        "only whitespace terminates the name"
    );

    // The two agree exactly when the arguments hold no separator.
    let name = writable("a/b/c x y z");
    // SAFETY: `name` outlives the borrow.
    assert_eq!(
        unsafe { borrowed(path_tail(name.as_ptr())) },
        split("a/b/c x y z").0
    );
    let name = writable("a/b/c x y/z");
    // SAFETY: as above.
    assert_ne!(
        unsafe { borrowed(path_tail(name.as_ptr())) },
        split("a/b/c x y/z").0
    );
}

/// `#37080`: a name is shortened against a directory only when it really is
/// under it, and repeated separators do not stop that.
#[test]
fn a_name_shortens_against_a_directory_it_is_under_and_nothing_else() {
    let _sandbox = Sandbox::dir("path-shorten");
    let shorten = |full: Option<&str>, dir: &str| {
        let mut full = full.map(writable);
        let mut dir = writable(dir);
        let full_ptr = full.as_mut().map_or(ptr::null_mut(), Vec::as_mut_ptr);
        // SAFETY: both are this frame's; the answer points into `full`.
        unsafe {
            let short = path_shorten_fname(full_ptr, dir.as_mut_ptr());
            (!short.is_null()).then(|| borrowed(short))
        }
    };

    assert_eq!(shorten(None, "some/directory/file.txt"), None);
    assert_eq!(shorten(Some("as/this.txt"), "not/the/same"), None);
    assert_eq!(
        shorten(Some("some/very/long/directory/file.txt"), "some/very/long/"),
        None,
        "the directory must not carry its own trailing separator"
    );
    for full in [
        "some/very/long/directory/file.txt",
        "some/very/long//directory/file.txt",
        "some/very/long///directory/file.txt",
    ] {
        assert_eq!(
            shorten(Some(full), "some/very/long"),
            Some("directory/file.txt".to_string()),
            "{full}"
        );
    }
}

#[test]
fn a_name_under_the_working_directory_shortens_against_it() {
    let sandbox = Sandbox::dir("path-try-shorten");
    let inside = sandbox.mkdir("ut_directory");
    std::env::set_current_dir(&inside).expect("standing inside");

    let try_shorten = |full: Option<&str>| {
        let mut full = full.map(writable);
        let ptr = full.as_mut().map_or(ptr::null_mut(), Vec::as_mut_ptr);
        // SAFETY: `full` is this frame's; the answer points into it or is
        // NULL, which is what a NULL argument answers.
        unsafe {
            let short = path_try_shorten_fname(ptr);
            (!short.is_null()).then(|| borrowed(short))
        }
    };

    let under = format!("{}/subdir/file.txt", inside.to_str().expect("text"));
    assert_eq!(
        try_shorten(Some(&under)),
        Some("subdir/file.txt".to_string())
    );

    // Not under the working directory: handed straight back.
    let elsewhere = format!("{}/subdir/file.txt", sandbox.as_str());
    assert_eq!(try_shorten(Some(&elsewhere)), Some(elsewhere.clone()));
    assert_eq!(try_shorten(None), None);
}

/// How nvim works out its own path from `argv[0]`: absolute names stand,
/// anything with a separator or a leading dot is relative to the working
/// directory, and a bare name is looked up along `$PATH`.
#[test]
fn an_executable_name_is_guessed_from_the_working_directory_or_the_path() {
    let mut sandbox = Sandbox::dir("path-guess-exepath");
    let guess = |name: &str| {
        let name = writable(name);
        let mut buf = Buffer::new(255);
        // SAFETY: `name` is this frame's and `buf` is 255 bytes.
        unsafe { path_guess_exepath(name.as_ptr(), buf.as_mut_ptr(), buf.len()) };
        buf.text()
    };

    for name in ["./nvim", ".nvim", "foo/nvim"] {
        assert_eq!(
            guess(name),
            format!("{}/{name}", sandbox.as_str()),
            "{name}"
        );
    }
    assert_eq!(guess("/foo/bar/baz"), "/foo/bar/baz");
    assert_eq!(
        guess("23u0293_not_in_path"),
        "23u0293_not_in_path",
        "a name that is nowhere is handed back"
    );

    // A name that is on `$PATH` comes back as a full path, and a `$PATH`
    // entry longer than MAXPATHL does not stop the search.
    let found = guess("cat");
    assert!(found.ends_with("bin/cat"), "{found:?}");

    let saved = std::env::var("PATH").expect("$PATH is set");
    let insane = format!("{saved}:{}", "x/".repeat(4097));
    sandbox.set_env("PATH", &insane);
    let found = guess("cat");
    assert!(found.ends_with("bin/cat"), "{found:?}");
}

/// `vim_full_name` writes an absolute path into the caller's buffer, or the
/// name itself and FAIL when it cannot. The buffer is filled either way —
/// `#5737` is that a buffer too short still comes back NUL-terminated.
#[test]
fn a_file_name_is_made_absolute_or_handed_back_with_a_failure() {
    let sandbox = Sandbox::dir("path-full-name");
    sandbox.mkdir("unit-test-directory");
    sandbox.touch("unit-test-directory/test.file");

    let full = |name: Option<&str>, len: usize, force: bool| {
        let name = name.map(writable);
        let name_ptr = name.as_ref().map_or(ptr::null(), |n| n.as_ptr());
        let mut buf = Buffer::new(len);
        // SAFETY: `name` is this frame's or NULL, and `buf` is `len` bytes.
        let result = unsafe { vim_full_name(name_ptr, buf.as_mut_ptr(), len, force) };
        (result, buf.text())
    };
    let room = |a: &str, b: &str| a.len().max(b.len()) + 1;
    let here = sandbox.as_str().to_string();

    assert_eq!(full(None, 10, true).0, FAIL, "no name, no path");

    // `#5737`: a buffer too short gets a truncated, terminated copy.
    let long = "foo/bar/bazzzzzzz/buz/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/a";
    assert_eq!(full(Some(long), 8, true), (FAIL, long[..7].to_string()));

    // A URL is not a file name and is used as it stands.
    let url = "http://www.neovim.org";
    assert_eq!(full(Some(url), url.len() + 1, true), (OK, url.to_string()));

    // A name under a directory that does not exist fails, at every length.
    for rep in 1..=10 {
        let name = format!("{}dir/test.file", "non_existing_".repeat(rep));
        assert_eq!(
            full(Some(&name), name.len() + 1, true),
            (FAIL, name.clone())
        );
    }

    // The successes, each with the shape of relative name it is about.
    let parent = sandbox
        .root()
        .parent()
        .expect("a parent")
        .to_str()
        .expect("text")
        .to_string();
    let cases: [(&str, String); 8] = [
        ("test.file", format!("{here}/test.file")),
        ("..", parent.clone()),
        ("../test.file", format!("{parent}/test.file")),
        (
            "unit-test-directory/test.file",
            format!("{here}/unit-test-directory/test.file"),
        ),
        ("/tmp", "/tmp".to_string()),
        // `#20847`: a trailing separator on a directory that does not exist
        // is part of the name and stays.
        ("non_existing_dir/", format!("{here}/non_existing_dir/")),
        // `#7117`, both halves.
        (
            "./unit-test-directory/test.file",
            format!("{here}/unit-test-directory/test.file"),
        ),
        (
            "unit-test-directory/../unit-test-directory/test.file",
            format!("{here}/unit-test-directory/test.file"),
        ),
    ];
    for (name, expected) in cases {
        assert_eq!(
            full(Some(name), room(&expected, name), true),
            (OK, expected.clone()),
            "{name}"
        );
    }

    // `~` is not expanded here, and there is no directory of that name.
    assert!(!Path::new("~").is_dir(), "no literal ~ directory");
    let tilde = "~/home.file";
    assert_eq!(
        full(Some(tilde), tilde.len() + 1, true),
        (FAIL, tilde.to_string())
    );

    // Without `force` an absolute name is copied rather than resolved.
    let absolute = "/absolute/path";
    assert_eq!(
        full(Some(absolute), absolute.len() + 1, false),
        (OK, absolute.to_string())
    );

    // And the name itself is never written through.
    let name = writable("unit-test-directory/test.file");
    let expected = format!("{here}/unit-test-directory/test.file");
    let mut buf = Buffer::new(expected.len() + 1);
    // SAFETY: `name` is this frame's, and `buf` holds `expected.len() + 1`.
    let result = unsafe { vim_full_name(name.as_ptr(), buf.as_mut_ptr(), buf.len(), true) };
    assert_eq!((result, buf.text()), (OK, expected));
    // SAFETY: `name` is still alive and NUL-terminated.
    assert_eq!(
        unsafe { borrowed(name.as_ptr()) },
        "unit-test-directory/test.file"
    );
}

/// `path_fix_case` corrects a name's case against the file system on Windows
/// and macOS. This is neither, and the case-sensitive answer is that a name
/// is already the name it is.
#[test]
fn fixing_the_case_of_a_name_does_nothing_on_a_case_sensitive_system() {
    let sandbox = Sandbox::dir("path-fix-case");
    sandbox.mkdir("CamelCase");
    for name in ["camelcase", "cAMELcASE", "CamelCase"] {
        let mut buf = writable(name);
        // SAFETY: `buf` is this frame's, NUL-terminated and writable.
        unsafe { path_fix_case(buf.as_mut_ptr()) };
        // SAFETY: as above.
        assert_eq!(unsafe { borrowed(buf.as_ptr()) }, name);
    }
}

/// Joining two names, which adds exactly one separator and only where one is
/// needed.
#[test]
fn appending_to_a_path_adds_a_separator_only_where_one_is_missing() {
    let _sandbox = Sandbox::dir("path-append");
    let join = |head: &str, tail: &str, room: usize| {
        let mut path = vec![0 as c_char; room];
        for (slot, byte) in path.iter_mut().zip(head.bytes()) {
            *slot = byte as c_char;
        }
        let tail = writable(tail);
        // SAFETY: `path` is `room` bytes and NUL-terminated, `tail` is this
        // frame's.
        let result = unsafe { append_path(path.as_mut_ptr(), tail.as_ptr(), room) };
        // SAFETY: as above.
        (result, unsafe { borrowed(path.as_ptr()) })
    };

    assert_eq!(join("path1", "path2", 100), (OK, "path1/path2".to_string()));
    assert_eq!(
        join("path1/", "path2", 100),
        (OK, "path1/path2".to_string())
    );
    assert_eq!(join("", "/path2", 7), (OK, "/path2".to_string()));
    assert_eq!(join("path1", "", 6), (OK, "path1".to_string()), "no tail");
    assert_eq!(join("path1", ".", 6), (OK, "path1".to_string()), "no dot");
    // Eleven bytes is one short of "path1/path2" plus its terminator.
    assert_eq!(join("path1/", "path2", 11).0, FAIL);
}

#[test]
fn a_name_is_absolute_when_it_starts_with_a_separator_or_a_tilde() {
    let _sandbox = Sandbox::dir("path-absolute");
    let absolute = |s: &str| {
        let name = writable(s);
        // SAFETY: `name` is this frame's and NUL-terminated.
        unsafe { path_is_absolute(name.as_ptr()) }
    };
    assert!(absolute("/some/directory/"));
    assert!(absolute("~/in/my/home~/directory"));
    assert!(!absolute("not/in/my/home~/directory"));
}

/// Whether a name ends in a given extension, which `'fileignorecase'`
/// decides the case-sensitivity of.
#[test]
fn an_extension_matches_case_insensitively_only_when_the_option_says_so() {
    let _sandbox = Sandbox::dir("path-extension");
    let has = |name: &str, extension: &str| {
        let (name, extension) = (writable(name), writable(extension));
        // SAFETY: both are this frame's and NUL-terminated.
        unsafe { path_with_extension(name.as_ptr(), extension.as_ptr()) }
    };

    assert!(has("/some/path/file.lua", "lua"));
    assert!(!has("/some/path/file.vim", "lua"));
    assert!(!has("/some/path/file", "lua"), "no extension at all");

    let saved = p_fic.get();
    p_fic.set(0);
    assert!(!has("/some/path/file.VIM", "vim"));
    assert!(!has("/some/path/file.LUA", "lua"));
    p_fic.set(1);
    assert!(has("/some/path/file.VIM", "vim"));
    assert!(has("/some/path/file.LUA", "lua"));
    p_fic.set(saved);
}

/// A name is a URL when it starts with a scheme: letters, then any of
/// letters, digits, `+`, `-` and `.`, then a colon and a separator. The
/// answer says which separator, so a single-letter scheme — a Windows drive
/// letter — is deliberately not one.
#[test]
fn a_scheme_makes_a_name_a_url_and_says_which_separator_follows_it() {
    let _sandbox = Sandbox::dir("path-url");
    let url = |s: &str| {
        let name = writable(s);
        // SAFETY: `name` is this frame's and NUL-terminated.
        unsafe { path_with_url(name.as_ptr()) }
    };
    /// Not a URL.
    const NO: c_int = 0;
    /// A URL written with forward slashes.
    const SLASH: c_int = 1;
    /// A URL written with backslashes.
    const BACKSLASH: c_int = 2;

    for scheme in [
        "test",
        "test123",
        "test+abc",
        "test-abc",
        "test.abc",
        "test+abc-123.ghi",
    ] {
        assert_eq!(url(&format!("{scheme}://xyz/foo/b0")), SLASH, "{scheme}");
        assert_eq!(
            url(&format!("{scheme}:\\\\xyz\\foo\\b0")),
            BACKSLASH,
            "{scheme}"
        );
    }

    assert_eq!(
        url("test_abc://xyz/foo/b2"),
        NO,
        "underscore is not allowed"
    );
    for bad in ["-test", "test-", "+test", "test+", ".test", "test."] {
        assert_eq!(
            url(&format!("{bad}://xyz/foo/b4")),
            NO,
            "{bad} begins or ends with a separator character"
        );
    }

    // One separator is enough when the scheme is more than one letter.
    for good in [
        "test-C",
        "test-custom",
        "test+C",
        "test+custom",
        "test.C",
        "test.custom",
    ] {
        assert_eq!(url(&format!("{good}:/xyz/foo/b5")), SLASH, "{good}");
    }
    // ...and a single letter is a drive letter, not a scheme.
    assert_eq!(url("c:/xyz/foo/b5"), NO);
    assert_eq!(url("C:/xyz/foo/b5"), NO);
}
