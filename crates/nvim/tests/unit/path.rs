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

use std::ffi::{CStr, c_int};
use std::path::Path;

use neovim::option::vars::P_FIC;
use neovim::path::{
    append_path, invocation_path_tail, kBothFilesMissing, kDifferentFiles, kEqualFileNames,
    kEqualFiles, kOneFileMissing, next_component_index, path_fix_case, path_fnamecmp,
    path_fnamencmp, path_full_compare, path_full_dir_name, path_guess_exepath, path_is_absolute,
    path_shorten_fname, path_tail, path_try_shorten_fname, path_with_extension, path_with_url,
    pathcmp, shorten_dir_name, tail_with_sep_index, vim_full_name,
};
use neovim::types::Failed;

use crate::support::{Sandbox, cstr};

/// Bytes an entry point answered, as the text the fixtures are.
fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("the fixtures are text")
}

/// A borrowed C string, as text.
fn borrowed(s: &CStr) -> String {
    s.to_string_lossy().into_owned()
}

#[test]
fn a_directory_name_is_resolved_against_the_working_directory() {
    let sandbox = Sandbox::dir("path-full-dir-name");
    let real = sandbox.mkdir("unit-test-directory");
    std::os::unix::fs::symlink(&real, sandbox.path("unit-test-symlink")).expect("a link");

    let resolve = |directory: &str| {
        let result = path_full_dir_name(&cstr(directory), sandbox.as_str().len() + 64);
        result.map(|name| text(&name))
    };

    // The empty name is the working directory.
    assert_eq!(resolve(""), Ok(sandbox.as_str().to_string()));

    let parent = sandbox
        .root()
        .parent()
        .expect("the sandbox has a parent")
        .to_str()
        .expect("text")
        .to_string();
    assert_eq!(resolve(".."), Ok(parent));

    // A link is resolved, absolutely or relatively; a name that does not
    // exist is still made absolute.
    let expected = real.to_str().expect("text").to_string();
    for name in [
        "unit-test-directory",
        "unit-test-symlink",
        real.to_str().expect("text"),
        sandbox.path("unit-test-symlink").to_str().expect("text"),
    ] {
        assert_eq!(resolve(name), Ok(expected.clone()), "{name}");
    }
    assert_eq!(
        resolve("does-not-exist"),
        Ok(format!("{}/does-not-exist", sandbox.as_str()))
    );

    // ...but an absolute name that does not exist cannot be resolved at all.
    assert_eq!(resolve("/does_not_exist"), Err(Failed));
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
        let result = path_full_dir_name(&cstr(name), sandbox.as_str().len() + 64);
        assert_eq!(result.map(|name| text(&name)), Ok(expected.clone()));
    }
}

/// The five answers `path_full_compare` has, which depend on whether each
/// name exists as well as on what it says.
#[test]
fn comparing_two_names_says_which_of_the_five_relations_holds() {
    let sandbox = Sandbox::dir("path-full-compare");
    sandbox.touch("f1.o");
    sandbox.touch("f2.o");

    let compare =
        |a: &str, b: &str, checkname: bool| path_full_compare(&cstr(a), &cstr(b), checkname, true);

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
    let tail = |s: &str| borrowed(path_tail(&cstr(s)));
    assert_eq!(tail("directory/file.txt"), "file.txt");
    assert_eq!(tail("directory/"), "", "a trailing separator has no tail");

    let with_sep = |s: &str| s[tail_with_sep_index(s.as_bytes())..].to_string();
    assert_eq!(with_sep("directory///file.txt"), "///file.txt");
    assert_eq!(with_sep(""), "");
    assert_eq!(with_sep("some/directory/"), "/");
    assert_eq!(with_sep("/file.txt"), "file.txt", "a leading one is cut");
    assert_eq!(with_sep("/"), "");
    assert_eq!(with_sep("file.txt"), "file.txt");

    let next = |s: &str| s[next_component_index(s.as_bytes())..].to_string();
    assert_eq!(next("some/directory/file.txt"), "directory/file.txt");
    assert_eq!(next("file.txt"), "", "nothing follows a name with no sep");
}

/// `invocation_path_tail` is `path_tail` for a command line: the arguments
/// are not part of the executable's name, so a separator inside them does
/// not move the split.
#[test]
fn an_invocations_tail_stops_at_the_first_space() {
    let _sandbox = Sandbox::dir("path-invocation");
    let split = |s: &str| borrowed(invocation_path_tail(&cstr(s)));

    assert_eq!(split("directory/exe a b c"), "exe a b c");
    assert_eq!(split("/usr/bin/z a b"), "z a b");
    assert_eq!(
        split("exe a/b\\c"),
        "exe a/b\\c",
        "a separator in the arguments is not the executable's"
    );
    assert_eq!(
        split("exe-a+b_c[]()|#!@$%^&*"),
        "exe-a+b_c[]()|#!@$%^&*",
        "only whitespace terminates the name"
    );

    // The two agree exactly when the arguments hold no separator.
    assert_eq!(
        borrowed(path_tail(&cstr("a/b/c x y z"))),
        split("a/b/c x y z")
    );
    assert_ne!(
        borrowed(path_tail(&cstr("a/b/c x y/z"))),
        split("a/b/c x y/z")
    );
}

/// `#37080`: a name is shortened against a directory only when it really is
/// under it, and repeated separators do not stop that.
#[test]
fn a_name_shortens_against_a_directory_it_is_under_and_nothing_else() {
    let _sandbox = Sandbox::dir("path-shorten");
    let shorten = |full: Option<&str>, dir: &str| {
        let full = cstr(full?);
        path_shorten_fname(&full, dir.as_bytes()).map(borrowed)
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
        let full = cstr(full?);
        Some(borrowed(path_try_shorten_fname(&full)))
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
    let guess = |name: &str| text(&path_guess_exepath(&cstr(name), 255));

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
        // The name the buffer is left with either way, and whether it is
        // a full path.
        match vim_full_name(&cstr(name.unwrap_or_default()), len, force) {
            Ok(name) => (Ok(()), text(&name)),
            Err(name) => (Err(Failed), text(&name)),
        }
    };
    let room = |a: &str, b: &str| a.len().max(b.len()) + 1;
    let here = sandbox.as_str().to_string();

    // `#5737`: a buffer too short gets a truncated, terminated copy.
    let long = "foo/bar/bazzzzzzz/buz/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/a";
    assert_eq!(
        full(Some(long), 8, true),
        (Err(Failed), long[..7].to_string())
    );

    // A URL is not a file name and is used as it stands.
    let url = "http://www.neovim.org";
    assert_eq!(
        full(Some(url), url.len() + 1, true),
        (Ok(()), url.to_string())
    );

    // A name under a directory that does not exist fails, at every length.
    for rep in 1..=10 {
        let name = format!("{}dir/test.file", "non_existing_".repeat(rep));
        assert_eq!(
            full(Some(&name), name.len() + 1, true),
            (Err(Failed), name.clone())
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
            (Ok(()), expected.clone()),
            "{name}"
        );
    }

    // `~` is not expanded here, and there is no directory of that name.
    assert!(!Path::new("~").is_dir(), "no literal ~ directory");
    let tilde = "~/home.file";
    assert_eq!(
        full(Some(tilde), tilde.len() + 1, true),
        (Err(Failed), tilde.to_string())
    );

    // Without `force` an absolute name is copied rather than resolved.
    let absolute = "/absolute/path";
    assert_eq!(
        full(Some(absolute), absolute.len() + 1, false),
        (Ok(()), absolute.to_string())
    );

    // A buffer exactly the length of the answer is room enough.
    let expected = format!("{here}/unit-test-directory/test.file");
    assert_eq!(
        full(
            Some("unit-test-directory/test.file"),
            expected.len() + 1,
            true
        ),
        (Ok(()), expected)
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
        let mut buf = name.as_bytes().to_vec();
        path_fix_case(&mut buf);
        assert_eq!(text(&buf), name);
    }
}

/// Joining two names, which adds exactly one separator and only where one is
/// needed.
#[test]
fn appending_to_a_path_adds_a_separator_only_where_one_is_missing() {
    let _sandbox = Sandbox::dir("path-append");
    let join = |head: &str, tail: &str, room: usize| {
        let mut path = head.as_bytes().to_vec();
        let result = append_path(&mut path, tail.as_bytes(), room);
        (result, text(&path))
    };

    assert_eq!(
        join("path1", "path2", 100),
        (Ok(()), "path1/path2".to_string())
    );
    assert_eq!(
        join("path1/", "path2", 100),
        (Ok(()), "path1/path2".to_string())
    );
    assert_eq!(join("", "/path2", 7), (Ok(()), "/path2".to_string()));
    assert_eq!(
        join("path1", "", 6),
        (Ok(()), "path1".to_string()),
        "no tail"
    );
    assert_eq!(
        join("path1", ".", 6),
        (Ok(()), "path1".to_string()),
        "no dot"
    );
    // Eleven bytes is one short of "path1/path2" plus its terminator.
    assert_eq!(join("path1/", "path2", 11).0, Err(Failed));
}

#[test]
fn a_name_is_absolute_when_it_starts_with_a_separator_or_a_tilde() {
    let _sandbox = Sandbox::dir("path-absolute");
    let absolute = |s: &str| path_is_absolute(&cstr(s));
    assert!(absolute("/some/directory/"));
    assert!(absolute("~/in/my/home~/directory"));
    assert!(!absolute("not/in/my/home~/directory"));
    // The empty name has no first byte, and upstream reads its terminator.
    assert!(!absolute(""));
    assert!(absolute("/"), "a bare separator is still absolute");
}

/// Whether a name ends in a given extension, which `'fileignorecase'`
/// decides the case-sensitivity of.
#[test]
fn an_extension_matches_case_insensitively_only_when_the_option_says_so() {
    let _sandbox = Sandbox::dir("path-extension");
    let has = |name: &str, extension: &str| path_with_extension(&cstr(name), &cstr(extension));

    assert!(has("/some/path/file.lua", "lua"));
    assert!(!has("/some/path/file.vim", "lua"));
    assert!(!has("/some/path/file", "lua"), "no extension at all");
    assert!(!has("", "lua"), "the empty name has no dot");
    assert!(
        has("/some/path/file.", ""),
        "an empty extension after the dot"
    );
    assert!(
        has("/some/path/naïve.lua", "lua"),
        "the bytes before the dot are not read"
    );

    let saved = P_FIC.get();
    P_FIC.set(false);
    assert!(!has("/some/path/file.VIM", "vim"));
    assert!(!has("/some/path/file.LUA", "lua"));
    P_FIC.set(true);
    assert!(has("/some/path/file.VIM", "vim"));
    assert!(has("/some/path/file.LUA", "lua"));
    P_FIC.set(saved);
}

/// A name is a URL when it starts with a scheme: letters, then any of
/// letters, digits, `+`, `-` and `.`, then a colon and a separator. The
/// answer says which separator, so a single-letter scheme — a Windows drive
/// letter — is deliberately not one.
#[test]
fn a_scheme_makes_a_name_a_url_and_says_which_separator_follows_it() {
    let _sandbox = Sandbox::dir("path-url");
    let url = |s: &str| path_with_url(&cstr(s));
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
    // Names with no scheme at all, the empty one included.
    assert_eq!(url(""), NO);
    assert_eq!(url("test"), NO, "a scheme with nothing after it");
    assert_eq!(url("test:"), NO, "a colon with no separator");
    assert_eq!(url("/absolute/path"), NO);
    assert_eq!(url("naïve://xyz"), NO, "a scheme is ASCII");
}

/// `path_fnamecmp` is the file-name comparison `'fileignorecase'` decides
/// the case-sensitivity of. It reads both names whole, so a trailing
/// separator *is* a difference — unlike `pathcmp` below.
#[test]
fn comparing_two_file_names_reads_them_whole_and_folds_case_by_the_option() {
    let _sandbox = Sandbox::globals();
    let cmp = |a: &str, b: &str| path_fnamecmp(a.as_bytes(), b.as_bytes()).signum();

    let saved = P_FIC.get();
    P_FIC.set(false);
    assert_eq!(cmp("a/b", "a/b"), 0);
    assert_eq!(cmp("", ""), 0, "two empty names are the same name");
    assert_eq!(cmp("", "a"), -1, "the empty name sorts first");
    assert_eq!(
        cmp("a/b", "a/b/"),
        -1,
        "a trailing separator is a difference"
    );
    assert_eq!(cmp("a/B", "a/b"), -1, "and so is case, with the option off");
    assert_eq!(cmp("naïve", "naïve"), 0, "multibyte bytes compare as bytes");
    P_FIC.set(true);
    assert_eq!(cmp("a/B", "a/b"), 0, "with the option on, case is not");
    P_FIC.set(saved);

    // The bounded form stops at `len`, or at either name's NUL first.
    assert_eq!(path_fnamencmp(b"a/bcd", b"a/bzz", 3), 0);
    assert_eq!(path_fnamencmp(b"a/b", b"a/bzz", 9).signum(), -1);
    assert_eq!(path_fnamencmp(b"", b"", 4), 0);
}

/// `pathcmp` is the text-only comparison that sorts names and decides
/// whether two of them are the same. Three rules are peculiar to it and all
/// three are here: a path separator sorts before any other byte, a trailing
/// separator does not make a name different, and `maxlen` cuts both sides.
///
/// The sign is what matters, not the magnitude — for two names that differ
/// in an ordinary byte it is the folded difference of the two codepoints.
#[test]
fn comparing_names_sorts_a_separator_first_and_ignores_a_trailing_one() {
    let _sandbox = Sandbox::globals();
    #[track_caller]
    fn cmp(a: &str, b: &str, maxlen: c_int) -> c_int {
        let maxlen = usize::try_from(maxlen).ok();
        pathcmp(a.as_bytes(), b.as_bytes(), maxlen).signum()
    }

    assert_eq!(cmp("a/b", "a/b", -1), 0);
    assert_eq!(cmp("", "", -1), 0);
    // A trailing separator is not a difference; two of them are.
    assert_eq!(cmp("a/b", "a/b/", -1), 0);
    assert_eq!(cmp("a/b/", "a/b", -1), 0);
    assert_eq!(cmp("a/b", "a/b//", -1), -1);
    // A separator sorts before anything else, whichever side it is on.
    assert_eq!(cmp("foo/bar", "foo-bar", -1), -1);
    assert_eq!(cmp("foo-bar", "foo/bar", -1), 1);
    // A prefix comes first, and the difference decides otherwise.
    assert_eq!(cmp("ab", "abc", -1), -1);
    assert_eq!(cmp("abc", "ab", -1), 1);
    assert_eq!(cmp("abc", "abd", -1), -1);
    // `maxlen` stops the comparison before the difference.
    assert_eq!(cmp("abcd", "abce", 3), 0);
    assert_eq!(cmp("abcd", "abce", 4), -1);
    // Multibyte characters are compared as characters.
    assert_eq!(cmp("\u{e9}", "\u{e9}", -1), 0);
    assert_eq!(cmp("a\u{4e2d}b", "a\u{4e2d}c", -1), -1);
    assert_eq!(cmp("\u{4e2d}/x", "\u{4e2d}-x", -1), -1);
}

/// `shorten_dir_len` keeps `trim_len` characters of every component but the
/// last, and a leading `~` or `.` is free — it does not count towards the
/// length, so `".bar"` keeps two characters at a `trim_len` of one.
#[test]
fn shortening_a_directory_keeps_the_first_characters_of_each_component() {
    let _sandbox = Sandbox::globals();
    #[track_caller]
    fn shorten(name: &str, trim_len: c_int) -> String {
        let mut buf = cstr(name).into_bytes_with_nul();
        shorten_dir_name(&mut buf, trim_len);
        borrowed(CStr::from_bytes_until_nul(&buf).expect("terminated"))
    }

    assert_eq!(shorten("~/foo/../.bar/fname", 1), "~/f/../.b/fname");
    assert_eq!(shorten("~/foo/../.bar/fname", 2), "~/fo/../.ba/fname");
    assert_eq!(shorten("fname", 1), "fname");
    assert_eq!(shorten("", 1), "");
    assert_eq!(shorten("/a/bb/ccc/name", 1), "/a/b/c/name");
    // A character is kept whole, however many bytes it takes.
    assert_eq!(shorten("/\u{4e2d}\u{6587}/name", 1), "/\u{4e2d}/name");
    assert_eq!(shorten("/e\u{301}x/name", 1), "/e\u{301}/name");
}
