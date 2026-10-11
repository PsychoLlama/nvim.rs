//! `split_one_arg`, the argument-list splitter behind `:args`/`:argadd`.
//!
//! The oldtest suite covers it end to end (`Test_args_with_quote`,
//! `Test_large_arg`); this pins the escaping rules directly.

use neovim::arglist::split_one_arg;

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

/// An argument list edited from under a walk: `:argdo` bodies that delete,
/// add and wipe; `:next`/`:argedit` with autocommands that wipe or replace
/// the list (and read it back with `argv()` while they run); `BufAdd`
/// re-entering `:argadd` (E1156); `BufNew` reading the list while `:args`
/// builds it; local and global lists across `:split`/`:tabnew`/
/// `:arglocal`/`:argglobal`, with `argv()`/`argc()`/`arglistid()` aimed at
/// each and at windows that do not exist; `:all` under an autocommand that
/// tries to empty the list, and `:all` closing a window that held the last
/// reference to its own list (upstream's E1156 there is kept); `:argdedupe`;
/// the `:args` listing; `##`.
///
/// The expected text is upstream nvim 0.12.5's own answer to the same
/// script, byte for byte.
#[cfg(not(miri))]
#[test]
fn edits_from_autocommands_and_local_lists() {
    use crate::support::{Probe, Sandbox};
    let sandbox = Sandbox::dir("arglist_edits");
    let fx = Probe::new(sandbox.editor());
    fx.home();
    sandbox.write("al.vim", SCRIPT.as_bytes());
    fx.run("source al.vim");
    fx.run("call writefile(g:out, 'out.txt') | unlet g:out g:seen");
    fx.run("delfunction Log | delfunction Reset | augroup! T");
    fx.home();
    let got = std::fs::read(sandbox.path("out.txt")).expect("the script's log");
    let got = String::from_utf8_lossy(&got);
    for (row, (got, want)) in got.lines().zip(EXPECTED.lines()).enumerate() {
        assert_eq!(got, want, "row {row}");
    }
    assert_eq!(got.lines().count(), EXPECTED.lines().count());
}

/// The script: every case resets to one window on buffer 1 with an empty
/// global list.
const SCRIPT: &str = "let g:out = []\n\
func Log(label, ...)\n\
  call add(g:out, a:label . ': ' . string(a:0 ? a:1 : '') . ' args=' . string(argv()) . ' idx=' . argidx() . ' id=' . arglistid() . ' buf=' . fnamemodify(bufname(), ':t') . ' err=' . v:errmsg)\n\
endfunc\n\
func Reset()\n\
  silent! tabonly!\n\
  silent! only!\n\
  silent! argglobal\n\
  silent! %argdelete\n\
  silent! buffer! 1\n\
  if bufnr('$') > 1 | silent! 2,$bwipeout! | endif\n\
  let v:errmsg = ''\n\
  autocmd! T\n\
endfunc\n\
augroup T | augroup END\n\
for f in ['a', 'b', 'c', 'd', 'e']\n\
  call writefile([f], f)\n\
endfor\n\
\n\
\" :argdo whose body deletes the rest\n\
\" :all closing a window that holds the last reference to its own list\n\
call Reset()\n\
call writefile(['x'], 'x')\n\
args a\n\
split\n\
arglocal x\n\
wincmd w\n\
let v:errmsg = ''\n\
all\n\
call add(g:out, 'all-local: ' . winnr('$') . ' ' . v:errmsg . ' ' . string(argv()) . arglistid())\n\
\n\
call Reset()\n\
args a b c d\n\
let g:seen = []\n\
silent! argdo call add(g:seen, bufname()) | if bufname() == 'b' | 3,$argdelete | endif\n\
call Log('argdo-del', g:seen)\n\
\n\
\" :argdo whose body adds\n\
\" :all closing a window that holds the last reference to its own list\n\
call Reset()\n\
call writefile(['x'], 'x')\n\
args a\n\
split\n\
arglocal x\n\
wincmd w\n\
let v:errmsg = ''\n\
all\n\
call add(g:out, 'all-local: ' . winnr('$') . ' ' . v:errmsg . ' ' . string(argv()) . arglistid())\n\
\n\
call Reset()\n\
args a b\n\
let g:seen = []\n\
silent! argdo call add(g:seen, bufname()) | if len(g:seen) < 4 | $argadd e | endif\n\
call Log('argdo-add', g:seen)\n\
\n\
\" :argdo whose body wipes everything\n\
\" :all closing a window that holds the last reference to its own list\n\
call Reset()\n\
call writefile(['x'], 'x')\n\
args a\n\
split\n\
arglocal x\n\
wincmd w\n\
let v:errmsg = ''\n\
all\n\
call add(g:out, 'all-local: ' . winnr('$') . ' ' . v:errmsg . ' ' . string(argv()) . arglistid())\n\
\n\
call Reset()\n\
args a b c\n\
let g:seen = []\n\
silent! argdo call add(g:seen, bufname()) | %argdelete\n\
call Log('argdo-wipe', g:seen)\n\
\n\
\" :argdo whose body deletes the current and earlier ones\n\
\" :all closing a window that holds the last reference to its own list\n\
call Reset()\n\
call writefile(['x'], 'x')\n\
args a\n\
split\n\
arglocal x\n\
wincmd w\n\
let v:errmsg = ''\n\
all\n\
call add(g:out, 'all-local: ' . winnr('$') . ' ' . v:errmsg . ' ' . string(argv()) . arglistid())\n\
\n\
call Reset()\n\
args a b c d\n\
let g:seen = []\n\
silent! argdo call add(g:seen, bufname()) | 1argdelete\n\
call Log('argdo-del-first', g:seen)\n\
\n\
\" :next with an autocommand that wipes the list\n\
\" :all closing a window that holds the last reference to its own list\n\
call Reset()\n\
call writefile(['x'], 'x')\n\
args a\n\
split\n\
arglocal x\n\
wincmd w\n\
let v:errmsg = ''\n\
all\n\
call add(g:out, 'all-local: ' . winnr('$') . ' ' . v:errmsg . ' ' . string(argv()) . arglistid())\n\
\n\
call Reset()\n\
args a b c\n\
autocmd T BufEnter b %argdelete | call Log('in-autocmd', argv())\n\
silent! next\n\
call Log('next-wipe')\n\
autocmd! T\n\
\n\
\" :next with an autocommand that replaces the list\n\
\" :all closing a window that holds the last reference to its own list\n\
call Reset()\n\
call writefile(['x'], 'x')\n\
args a\n\
split\n\
arglocal x\n\
wincmd w\n\
let v:errmsg = ''\n\
all\n\
call add(g:out, 'all-local: ' . winnr('$') . ' ' . v:errmsg . ' ' . string(argv()) . arglistid())\n\
\n\
call Reset()\n\
args a b c\n\
autocmd T BufEnter b args d e | call Log('in-autocmd2', argv(0))\n\
silent! next\n\
call Log('next-replace')\n\
autocmd! T\n\
\n\
\" BufLeave wipe\n\
\" :all closing a window that holds the last reference to its own list\n\
call Reset()\n\
call writefile(['x'], 'x')\n\
args a\n\
split\n\
arglocal x\n\
wincmd w\n\
let v:errmsg = ''\n\
all\n\
call add(g:out, 'all-local: ' . winnr('$') . ' ' . v:errmsg . ' ' . string(argv()) . arglistid())\n\
\n\
call Reset()\n\
args a b c\n\
autocmd T BufLeave a %argdelete\n\
silent! next\n\
call Log('next-leave-wipe')\n\
autocmd! T\n\
\n\
\" BufAdd recursion -> E1156\n\
\" :all closing a window that holds the last reference to its own list\n\
call Reset()\n\
call writefile(['x'], 'x')\n\
args a\n\
split\n\
arglocal x\n\
wincmd w\n\
let v:errmsg = ''\n\
all\n\
call add(g:out, 'all-local: ' . winnr('$') . ' ' . v:errmsg . ' ' . string(argv()) . arglistid())\n\
\n\
call Reset()\n\
args a\n\
autocmd T BufAdd * argadd c\n\
silent! argadd b\n\
call Log('argadd-recursive')\n\
autocmd! T\n\
\n\
\" BufNew while :args sets\n\
\" :all closing a window that holds the last reference to its own list\n\
call Reset()\n\
call writefile(['x'], 'x')\n\
args a\n\
split\n\
arglocal x\n\
wincmd w\n\
let v:errmsg = ''\n\
all\n\
call add(g:out, 'all-local: ' . winnr('$') . ' ' . v:errmsg . ' ' . string(argv()) . arglistid())\n\
\n\
call Reset()\n\
autocmd T BufNew * call Log('bufnew', argv())\n\
silent! args a b\n\
call Log('args-bufnew')\n\
autocmd! T\n\
\n\
\" argedit with an autocmd deleting\n\
\" :all closing a window that holds the last reference to its own list\n\
call Reset()\n\
call writefile(['x'], 'x')\n\
args a\n\
split\n\
arglocal x\n\
wincmd w\n\
let v:errmsg = ''\n\
all\n\
call add(g:out, 'all-local: ' . winnr('$') . ' ' . v:errmsg . ' ' . string(argv()) . arglistid())\n\
\n\
call Reset()\n\
args a b\n\
autocmd T BufEnter d 1argdelete\n\
silent! argedit d\n\
call Log('argedit-del')\n\
autocmd! T\n\
\n\
\" local vs global\n\
\" :all closing a window that holds the last reference to its own list\n\
call Reset()\n\
call writefile(['x'], 'x')\n\
args a\n\
split\n\
arglocal x\n\
wincmd w\n\
let v:errmsg = ''\n\
all\n\
call add(g:out, 'all-local: ' . winnr('$') . ' ' . v:errmsg . ' ' . string(argv()) . arglistid())\n\
\n\
call Reset()\n\
args a b c\n\
split\n\
call Log('split-shares')\n\
arglocal\n\
call Log('arglocal-copy')\n\
argadd d\n\
call Log('local-add')\n\
wincmd w\n\
call Log('other-global')\n\
wincmd w\n\
tabnew\n\
call Log('tabnew-inherits')\n\
argglobal\n\
call Log('argglobal')\n\
tabonly!\n\
only!\n\
arglocal x y\n\
call Log('arglocal-args')\n\
call add(g:out, 'argv-1: ' . string(argv(-1, -1)) . string(argc(-1)) . string(argv(0, -1)))\n\
call add(g:out, 'winid: ' . string(argv(-1, win_getid())) . string(argc(win_getid())) . string(argc(9999)) . string(argv(-1, 9999)) . string(argv(0, 9999)))\n\
call add(g:out, 'arglistid: ' . arglistid(9999) . ' ' . arglistid(1, 1))\n\
argglobal\n\
call Log('back-global')\n\
\n\
\" :all with an autocommand changing the list\n\
\" :all closing a window that holds the last reference to its own list\n\
call Reset()\n\
call writefile(['x'], 'x')\n\
args a\n\
split\n\
arglocal x\n\
wincmd w\n\
let v:errmsg = ''\n\
all\n\
call add(g:out, 'all-local: ' . winnr('$') . ' ' . v:errmsg . ' ' . string(argv()) . arglistid())\n\
\n\
call Reset()\n\
args a b c\n\
autocmd T BufEnter b %argdelete\n\
silent! all\n\
call add(g:out, 'all: ' . winnr('$') . ' ' . string(argv()) . v:errmsg)\n\
autocmd! T\n\
only!\n\
\n\
\" :argdedupe\n\
\" :all closing a window that holds the last reference to its own list\n\
call Reset()\n\
call writefile(['x'], 'x')\n\
args a\n\
split\n\
arglocal x\n\
wincmd w\n\
let v:errmsg = ''\n\
all\n\
call add(g:out, 'all-local: ' . winnr('$') . ' ' . v:errmsg . ' ' . string(argv()) . arglistid())\n\
\n\
call Reset()\n\
args a b a ./a c b\n\
argdedupe\n\
call Log('dedupe')\n\
\n\
\" :args listing\n\
\" :all closing a window that holds the last reference to its own list\n\
call Reset()\n\
call writefile(['x'], 'x')\n\
args a\n\
split\n\
arglocal x\n\
wincmd w\n\
let v:errmsg = ''\n\
all\n\
call add(g:out, 'all-local: ' . winnr('$') . ' ' . v:errmsg . ' ' . string(argv()) . arglistid())\n\
\n\
call Reset()\n\
args a b c\n\
next\n\
call add(g:out, 'list: ' . execute('args'))\n\
\n\
\" ## expansion\n\
\" :all closing a window that holds the last reference to its own list\n\
call Reset()\n\
call writefile(['x'], 'x')\n\
args a\n\
split\n\
arglocal x\n\
wincmd w\n\
let v:errmsg = ''\n\
all\n\
call add(g:out, 'all-local: ' . winnr('$') . ' ' . v:errmsg . ' ' . string(argv()) . arglistid())\n\
\n\
call Reset()\n\
args a\\ b c\n\
call add(g:out, 'hashhash: ' . expand('##'))\n\
\n\
\" :all closing a window that holds the last reference to its own list\n\
call Reset()\n\
call writefile(['x'], 'x')\n\
args a\n\
split\n\
arglocal x\n\
wincmd w\n\
let v:errmsg = ''\n\
all\n\
call add(g:out, 'all-local: ' . winnr('$') . ' ' . v:errmsg . ' ' . string(argv()) . arglistid())\n\
\n\
call Reset()\n\
";

/// Upstream's log of [`SCRIPT`].
const EXPECTED: &str = "all-local: 1 E1156: Cannot change the argument list recursively ['a']0\n\
argdo-del: ['a', 'b'] args=['a', 'b'] idx=1 id=0 buf=b err=\n\
all-local: 1 E1156: Cannot change the argument list recursively ['a']0\n\
argdo-add: ['a', 'b'] args=['a', 'b', 'e', 'e'] idx=1 id=0 buf=b err=\n\
all-local: 1 E1156: Cannot change the argument list recursively ['a']0\n\
argdo-wipe: ['a'] args=[] idx=0 id=0 buf=a err=E163: There is only one file to edit\n\
all-local: 1 E1156: Cannot change the argument list recursively ['a']0\n\
argdo-del-first: ['a', 'c'] args=['c', 'd'] idx=0 id=0 buf=c err=\n\
all-local: 1 E1156: Cannot change the argument list recursively ['a']0\n\
in-autocmd: [] args=[] idx=0 id=0 buf=b err=\n\
next-wipe: '' args=[] idx=0 id=0 buf=b err=\n\
all-local: 1 E1156: Cannot change the argument list recursively ['a']0\n\
in-autocmd2: 'd' args=['d', 'e'] idx=0 id=0 buf=d err=\n\
next-replace: '' args=['d', 'e'] idx=0 id=0 buf=d err=\n\
all-local: 1 E1156: Cannot change the argument list recursively ['a']0\n\
next-leave-wipe: '' args=[] idx=0 id=0 buf=b err=\n\
all-local: 1 E1156: Cannot change the argument list recursively ['a']0\n\
argadd-recursive: '' args=['a', 'b'] idx=0 id=0 buf=a err=E1156: Cannot change the argument list recursively\n\
all-local: 1 E1156: Cannot change the argument list recursively ['a']0\n\
bufnew: ['a'] args=['a'] idx=0 id=0 buf=a err=\n\
args-bufnew: '' args=['a', 'b'] idx=0 id=0 buf=a err=\n\
all-local: 1 E1156: Cannot change the argument list recursively ['a']0\n\
argedit-del: '' args=['d', 'b'] idx=0 id=0 buf=d err=\n\
all-local: 1 E1156: Cannot change the argument list recursively ['a']0\n\
split-shares: '' args=['a', 'b', 'c'] idx=0 id=0 buf=a err=\n\
arglocal-copy: '' args=['a', 'b', 'c'] idx=0 id=12 buf=a err=\n\
local-add: '' args=['a', 'd', 'b', 'c'] idx=0 id=12 buf=a err=\n\
other-global: '' args=['a', 'b', 'c'] idx=0 id=0 buf=a err=\n\
tabnew-inherits: '' args=['a', 'd', 'b', 'c'] idx=0 id=12 buf= err=\n\
argglobal: '' args=['a', 'b', 'c'] idx=0 id=0 buf= err=\n\
arglocal-args: '' args=['x', 'y'] idx=0 id=13 buf=x err=\n\
argv-1: ['a', 'b', 'c']3'a'\n\
winid: ['x', 'y']2-1[]''\n\
arglistid: -1 13\n\
back-global: '' args=['a', 'b', 'c'] idx=0 id=0 buf=x err=\n\
all-local: 1 E1156: Cannot change the argument list recursively ['a']0\n\
all: 3 ['a', 'b', 'c']\n\
all-local: 1 E1156: Cannot change the argument list recursively ['a']0\n\
dedupe: '' args=['a', 'b', 'c'] idx=0 id=0 buf=a err=\n\
all-local: 1 E1156: Cannot change the argument list recursively ['a']0\n\
list: \0a   [b] c   \n\
all-local: 1 E1156: Cannot change the argument list recursively ['a']0\n\
hashhash: a\\ b c\n\
all-local: 1 E1156: Cannot change the argument list recursively ['a']0\n\
";
