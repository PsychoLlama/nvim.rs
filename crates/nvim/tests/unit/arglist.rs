//! `split_one_arg`, the argument-list splitter behind `:args`/`:argadd`.
//!
//! The oldtest suite covers it end to end (`Test_args_with_quote`,
//! `Test_large_arg`); this pins the escaping rules directly.

use c2rust_neovim::src::nvim::arglist::split_one_arg;

/// Runs the splitter over a NUL-terminated buffer the way `get_arglist` does,
/// and answers the arguments it carved out.
fn split_all(input: &str) -> Vec<String> {
    let mut buf: Vec<u8> = input.bytes().chain([0]).collect();
    let total = input.len();
    let mut args = Vec::new();
    let mut at = 0;
    while at < total && buf[at] != 0 {
        let start = at;
        at += split_one_arg(&mut buf[at..]);
        let end = start + buf[start..].iter().position(|&b| b == 0).unwrap();
        args.push(String::from_utf8(buf[start..end].to_vec()).unwrap());
        assert!(at > start, "the splitter must make progress");
    }
    args
}

#[test]
fn splits_on_whitespace() {
    assert_eq!(split_all("one two three"), ["one", "two", "three"]);
    assert_eq!(split_all("one\ttwo"), ["one", "two"]);
    assert_eq!(split_all("one   two"), ["one", "two"]);
}

#[test]
fn a_single_argument_is_returned_whole() {
    assert_eq!(split_all("solo"), ["solo"]);
    assert_eq!(split_all(""), Vec::<String>::new());
}

#[test]
fn leading_whitespace_yields_an_empty_first_argument() {
    // `get_arglist` records the position before splitting, so the caller sees
    // the empty string the terminator leaves behind.
    assert_eq!(split_all(" x"), ["", "x"]);
}

#[test]
fn a_backslash_escapes_the_next_byte_and_both_are_kept() {
    // The backslashes survive here; `alist_add`'s caller halves them later.
    assert_eq!(split_all(r"a\ b c"), [r"a\ b", "c"]);
    assert_eq!(split_all(r"a\\ b"), [r"a\\", "b"]);
    assert_eq!(split_all(r"a\`b c"), [r"a\`b", "c"]);
}

#[test]
fn a_trailing_backslash_is_not_an_escape() {
    // `rem_backslash` needs a byte after the backslash; at the end of the
    // string there is none, so the backslash is an ordinary character.
    assert_eq!(split_all(r"a\"), [r"a\"]);
    assert_eq!(split_all("a\\ "), ["a\\ "]);
}

#[test]
fn backticks_suspend_the_whitespace_rule() {
    assert_eq!(split_all("`echo one two` rest"), ["`echo one two`", "rest"]);
    // An unbalanced backtick swallows the remainder, as upstream does.
    assert_eq!(split_all("`echo one two"), ["`echo one two"]);
    assert_eq!(split_all("a`b c`d e"), ["a`b c`d", "e"]);
}

#[test]
fn only_space_and_tab_separate_arguments() {
    // `ascii_isspace` ends an argument on any of \t \n \v \f \r or space, but
    // `skipwhite` only steps over space and tab. So a newline both ends the
    // argument and is overwritten by its terminator, and everything after it
    // is silently dropped — upstream behaviour, preserved.
    assert_eq!(split_all("a\nb"), ["a"]);
    assert_eq!(split_all("a\rb"), ["a"]);
}
