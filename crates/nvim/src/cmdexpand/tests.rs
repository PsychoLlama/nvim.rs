//! Command-line completion, driven through what Vimscript reaches it by:
//! `getcompletion()` for a pattern of a given type or a whole command line,
//! and [`expand_one`]'s modes for the walk `<Tab>`/`<S-Tab>` take through a
//! match list once it exists.
//!
//! No Lua and no runtime files: every completion here is answered from the
//! editor's own tables (commands, options, functions, the fixed argument
//! lists) or from a scratch directory tree the test builds. The directory
//! tests read the file system through libuv, which Miri cannot run.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::buffer::{bare_buffer, free_bare_buffer};
use crate::eval::list::{string_bytes, string_tv};
use crate::eval::typval::{list_items, tv_clear};
use crate::getchar::state::got_int;
use crate::global_cell::{editor_state::Held, editor_state_lock};
use crate::memory::XString;
use crate::option::vars::{P_CPO, P_PP, P_RTP, P_SU, P_WIC, P_WIG, p_wic, wop_flags};
use crate::options::kOptWopFlagFuzzy;
use crate::types::{EvalFuncData, ExpandContext, TypVal};
use crate::window::{bare_window, free_bare_window};
use crate::winlayer::graph::{leave_curbuf, leave_curwin};
use crate::winlayer::{Buf, Win};
use core::ffi::{c_int, c_uint};
use std::path::PathBuf;

/// The editor lock, held for the test, with `'wildoptions'` and
/// `'wildignorecase'` as a test found them put back on drop.
struct Fixture {
    window: Win,
    buffer: Buf,
    wop: c_uint,
    wic: bool,
    cpo: Option<XString>,
    _held: Held,
}

impl Fixture {
    fn new() -> Fixture {
        let held = editor_state_lock();
        got_int.set(false);
        // Compiling a pattern reads the current buffer's 'iskeyword'.
        let buffer = bare_buffer();
        buffer.make_current();
        // The user commands are looked up in the window's buffer.
        let mut window = bare_window();
        window.w_buffer = buffer;
        window.make_current();
        // Compiling a pattern reads 'cpoptions'.
        let cpo = P_CPO.swap(Some(XString::from("aABceFs_")));
        Fixture {
            window,
            buffer,
            wop: wop_flags.get(),
            wic: p_wic(),
            cpo,
            _held: held,
        }
    }

    fn fuzzy(&self) {
        wop_flags.set(wop_flags.get() | kOptWopFlagFuzzy);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        wop_flags.set(self.wop);
        P_WIC.set(self.wic);
        P_CPO.restore(self.cpo.take());
        leave_curwin();
        free_bare_window(self.window);
        leave_curbuf();
        free_bare_buffer(self.buffer);
    }
}

/// `getcompletion(pattern, kind)`, as the list of byte strings it answers.
fn complete(pattern: &str, kind: &str) -> Vec<String> {
    let args = [
        string_tv(pattern.as_bytes()),
        string_tv(kind.as_bytes()),
        TypVal::Number(1),
    ];
    let mut result = TypVal::Number(0);
    f_getcompletion(&args, &mut result, EvalFuncData::None);
    let answer = list_items(result.list_ref())
        .iter()
        .map(|li| String::from_utf8_lossy(string_bytes(&li.li_tv)).into_owned())
        .collect();
    tv_clear(&mut result);
    answer
}

/// `getcompletiontype(cmdline)`.
fn complete_type(cmdline: &str) -> String {
    let args = [string_tv(cmdline.as_bytes())];
    let mut result = TypVal::Number(0);
    f_getcompletiontype(&args, &mut result, EvalFuncData::None);
    let answer = String::from_utf8_lossy(string_bytes(&result)).into_owned();
    tv_clear(&mut result);
    answer
}

fn strings(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

#[test]
fn command_names_complete_from_the_table() {
    let _fx = Fixture::new();
    assert_eq!(complete("tabn", "command"), strings(&["tabnew", "tabnext"]));
    assert_eq!(complete("vspl", "command"), strings(&["vsplit"]));
    assert_eq!(complete("zzz", "command"), strings(&[]));
    // The same through the whole-line classifier.
    assert_eq!(complete("tabn", "cmdline"), strings(&["tabnew", "tabnext"]));
    // A range and a modifier are skipped to get at the command name.
    assert_eq!(
        complete("1,2tabn", "cmdline"),
        strings(&["tabnew", "tabnext"])
    );
    assert_eq!(
        complete("silent! tabn", "cmdline"),
        strings(&["tabnew", "tabnext"])
    );
    assert_eq!(complete("tabn", "cmdline"), complete("|tabn", "cmdline"));
    // After a bar, the next command is the one completed.
    assert_eq!(
        complete("echo 1 | tabn", "cmdline"),
        strings(&["tabnew", "tabnext"])
    );
}

#[test]
fn set_completes_option_names_and_values() {
    let _fx = Fixture::new();
    assert_eq!(
        complete("fileform", "option"),
        strings(&["fileformat", "fileformats"])
    );
    assert_eq!(
        complete("set fileform", "cmdline"),
        strings(&["fileformat", "fileformats"])
    );
    assert_eq!(
        complete("setlocal fileform", "cmdline"),
        strings(&["fileformat", "fileformats"])
    );
    // `no` names a boolean option; the answer is the bare name.
    assert_eq!(complete("set nowrapsc", "cmdline"), strings(&["wrapscan"]));
    assert_eq!(complete("set invwrapsc", "cmdline"), strings(&["wrapscan"]));
    // A string option's accepted words.
    assert_eq!(complete("set fileformat=u", "cmdline"), strings(&["unix"]));
    assert_eq!(complete("set ff=d", "cmdline"), strings(&["dos"]));
    // The second item of a comma list.
    assert_eq!(
        complete("set fileformats=unix,m", "cmdline"),
        strings(&["mac"])
    );
    // A flag list offers the letters the value does not hold yet.
    let flags = complete("set whichwrap+=", "cmdline");
    assert!(flags.contains(&"h".to_owned()), "{flags:?}");
    // A boolean option takes no value.
    assert_eq!(complete("set wrapscan=", "cmdline"), strings(&[]));
    // A value has no type name of its own.
    assert_eq!(complete_type("set fileformat="), "");
    assert_eq!(complete_type("set fileform"), "option");
}

#[test]
fn fixed_argument_lists() {
    let _fx = Fixture::new();
    assert_eq!(
        complete("", "filetypecmd"),
        strings(&["indent", "off", "on", "plugin"])
    );
    assert_eq!(
        complete("filetype plugin ", "cmdline"),
        strings(&["indent", "off", "on"])
    );
    assert_eq!(
        complete("filetype plugin indent ", "cmdline"),
        strings(&["off", "on"])
    );
    assert_eq!(complete("", "messages"), strings(&["clear"]));
    assert_eq!(complete("messages c", "cmdline"), strings(&["clear"]));
    assert_eq!(complete("", "mapclear"), strings(&["<buffer>"]));
    assert_eq!(
        complete("breakadd ", "cmdline"),
        strings(&["expr", "file", "func", "here"])
    );
    assert_eq!(
        complete("breakdel ", "cmdline"),
        strings(&["file", "func", "here"])
    );
    assert_eq!(complete("profdel ", "cmdline"), strings(&["file", "func"]));
    assert_eq!(complete("retab -", "cmdline"), strings(&["-indentonly"]));
    assert_eq!(complete_type("breakadd "), "breakpoint");
    assert_eq!(complete_type("filetype "), "filetypecmd");
    assert_eq!(complete_type("echo "), "expression");
    assert_eq!(complete_type("unknowncmd "), "");
}

#[test]
fn builtin_functions_and_expressions() {
    let _fx = Fixture::new();
    assert_eq!(complete("strle", "function"), strings(&["strlen("]));
    assert_eq!(
        complete("getcompletion", "function"),
        strings(&["getcompletion(", "getcompletiontype("])
    );
    // (An expression walks the variables of every tab page, which a lib
    // test does not have.)
}

#[test]
fn fuzzy_matching_scores_and_orders() {
    let fx = Fixture::new();
    fx.fuzzy();
    let found = complete("vspt", "command");
    assert_eq!(
        found.first().map(String::as_str),
        Some("vsplit"),
        "{found:?}"
    );
    let found = complete("set fiform", "cmdline");
    assert!(found.contains(&"fileformat".to_owned()), "{found:?}");
    assert!(found.contains(&"fileformats".to_owned()), "{found:?}");
    // An empty pattern is no fuzzy match: everything, in table order.
    let all = complete("", "filetypecmd");
    assert_eq!(all, strings(&["indent", "plugin", "on", "off"]));
    // File-like contexts never match fuzzily.
    assert!(!fuzzy_supported(ExpandContext::Files));
    assert!(fuzzy_supported(ExpandContext::Commands));
}

/// One expansion of `pattern` in `context`, kept, then the given modes in
/// order; each step's answer, and the selection after it.
fn walk(
    context: ExpandContext,
    pattern: &str,
    orig: &str,
    options: WildOpts,
    first: WildMode,
    then: &[WildMode],
) -> Vec<(Option<String>, c_int)> {
    expand_one_walk(
        context,
        pattern.as_bytes(),
        orig.as_bytes(),
        options,
        first,
        then,
    )
    .into_iter()
    .map(|(text, selected)| {
        (
            text.map(|text| String::from_utf8_lossy(&text).into_owned()),
            selected,
        )
    })
    .collect()
}

fn some(text: &str, selected: c_int) -> (Option<String>, c_int) {
    (Some(text.to_owned()), selected)
}

#[test]
fn expand_one_walks_the_matches() {
    let _fx = Fixture::new();
    let quiet = WildOpts::SILENT | WildOpts::NO_BEEP;
    // Keep, then forward past the end and back to the typed text.
    assert_eq!(
        walk(
            ExpandContext::Commands,
            "^tabn",
            "tabn",
            quiet,
            WildMode::ExpandKeep,
            &[WildMode::Next, WildMode::Next, WildMode::Next],
        ),
        vec![
            some("tabnew", 0),
            some("tabnext", 1),
            some("tabn", -1),
            some("tabnew", 0),
        ]
    );
    // Backwards from the first match wraps to the original text, then
    // the last match.
    assert_eq!(
        walk(
            ExpandContext::Commands,
            "^tabn",
            "tabn",
            quiet,
            WildMode::ExpandKeep,
            &[WildMode::Prev, WildMode::Prev],
        ),
        vec![some("tabnew", 0), some("tabn", -1), some("tabnext", 1)]
    );
    // NOSELECT starts at the original text.
    assert_eq!(
        walk(
            ExpandContext::Commands,
            "^tabn",
            "tabn",
            quiet | WildOpts::NOSELECT,
            WildMode::ExpandKeep,
            &[WildMode::Next],
        ),
        vec![some("tabnew", -1), some("tabnew", 0)]
    );
    // All of them, joined; then the longest common part.
    assert_eq!(
        walk(
            ExpandContext::Commands,
            "^tabn",
            "tabn",
            quiet,
            WildMode::All,
            &[],
        ),
        vec![some("tabnew tabnext", 0)]
    );
    assert_eq!(
        walk(
            ExpandContext::Commands,
            "^tabn",
            "tabn",
            quiet | WildOpts::USE_NL,
            WildMode::AllKeep,
            &[WildMode::All],
        ),
        vec![(None, 0), some("tabnew\ntabnext", 0)]
    );
    assert_eq!(
        walk(
            ExpandContext::Commands,
            "^tabn",
            "tabn",
            quiet,
            WildMode::Longest,
            &[WildMode::Next],
        ),
        vec![some("tabne", -1), some("tabnew", 0)]
    );
    // Apply takes the selection, Cancel the original text.
    assert_eq!(
        walk(
            ExpandContext::Commands,
            "^tabn",
            "tabn",
            quiet,
            WildMode::ExpandKeep,
            &[WildMode::Next, WildMode::Apply],
        ),
        vec![some("tabnew", 0), some("tabnext", 1), some("tabnext", 0)]
    );
    assert_eq!(
        walk(
            ExpandContext::Commands,
            "^tabn",
            "tabn",
            quiet,
            WildMode::ExpandKeep,
            &[WildMode::Cancel],
        ),
        vec![some("tabnew", 0), some("tabn", 0)]
    );
    // No match: nothing, and nothing to step through.
    assert_eq!(
        walk(
            ExpandContext::Commands,
            "^zzz",
            "zzz",
            quiet,
            WildMode::ExpandKeep,
            &[WildMode::Next],
        ),
        vec![(None, 0), (None, 0)]
    );
    // ExpandFree with more than one match answers nothing.
    assert_eq!(
        walk(
            ExpandContext::Commands,
            "^tabn",
            "tabn",
            quiet,
            WildMode::ExpandFree,
            &[],
        ),
        vec![(None, 0)]
    );
    // ... and the one match when there is only one.
    assert_eq!(
        walk(
            ExpandContext::Commands,
            "^vspl",
            "vspl",
            quiet,
            WildMode::ExpandFree,
            &[],
        ),
        vec![some("vsplit", 0)]
    );
}

/// A scratch directory under the system's temporary one, removed on drop,
/// with the options a file expansion reads given values for as long.
struct Scratch(PathBuf, Option<XString>, Option<XString>);

impl Scratch {
    fn new(name: &str, files: &[&str]) -> Scratch {
        let dir =
            std::env::temp_dir().join(format!("nvim-cmdexpand-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for file in files {
            let path = dir.join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            if file.ends_with('/') {
                std::fs::create_dir_all(&path).unwrap();
            } else {
                std::fs::write(&path, b"").unwrap();
            }
        }
        // Nothing ran the option defaults.
        let wig = P_WIG.swap(Some(XString::new()));
        let su = P_SU.swap(Some(XString::new()));
        Scratch(dir, wig, su)
    }

    fn path(&self) -> String {
        self.0.to_str().unwrap().to_owned()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
        P_WIG.restore(self.1.take());
        P_SU.restore(self.2.take());
    }
}

#[test]
#[cfg_attr(miri, ignore = "the directory walk is libuv, a foreign function")]
fn file_names_glob_the_directory() {
    let _fx = Fixture::new();
    let dir = Scratch::new("files", &["alpha.txt", "Apple.txt", "beta.txt", "sub/"]);
    let root = dir.path();
    assert_eq!(
        complete(&format!("{root}/a"), "file"),
        strings(&[&format!("{root}/alpha.txt")])
    );
    assert_eq!(
        complete(&format!("{root}/s"), "dir"),
        strings(&[&format!("{root}/sub/")])
    );
    assert_eq!(
        complete(&format!("e {root}/b"), "cmdline"),
        strings(&[&format!("{root}/beta.txt")])
    );
    // 'wildignorecase' folds case for file names.
    P_WIC.set(true);
    assert_eq!(
        complete(&format!("{root}/a"), "file"),
        strings(&[&format!("{root}/Apple.txt"), &format!("{root}/alpha.txt")])
    );
}

#[test]
#[cfg_attr(miri, ignore = "the directory walk is libuv, a foreign function")]
fn colorschemes_from_the_runtimepath() {
    let _fx = Fixture::new();
    let dir = Scratch::new(
        "rtp",
        &[
            "colors/blue.vim",
            "colors/dark.lua",
            "colors/blue.lua",
            "doc/",
        ],
    );
    std::fs::write(
        dir.0.join("doc/tags"),
        b"foo\tfoo.txt\t/*foo*\nfoobar\tfoo.txt\t/*foobar*\nbaz\tbaz.txt\t/*baz*\n",
    )
    .unwrap();
    let rtp = P_RTP.swap(Some(XString::from(dir.path().as_str())));
    let pp = P_PP.swap(Some(XString::new()));

    let colors = complete("", "color");
    P_RTP.restore(rtp);
    P_PP.restore(pp);

    assert_eq!(colors, strings(&["blue", "dark"]));
    // (`:help` asks Lua for the tags, and a lib test has no Lua.)
}

#[test]
fn argument_contexts_of_other_modules() {
    let _fx = Fixture::new();
    // (line, matches, getcompletiontype)
    let cases: &[(&str, &[&str], &str)] = &[
        (
            "sign ",
            &["define", "jump", "list", "place", "undefine", "unplace"],
            "sign",
        ),
        (
            "sign define x ",
            &[
                "culhl=",
                "icon=",
                "linehl=",
                "numhl=",
                "priority=",
                "text=",
                "texthl=",
            ],
            "sign",
        ),
        ("sign pl", &["place"], "sign"),
        (
            "syntax ",
            &[
                "case",
                "clear",
                "cluster",
                "conceal",
                "enable",
                "foldlevel",
                "include",
                "iskeyword",
                "keyword",
                "list",
                "manual",
                "match",
                "off",
                "on",
                "region",
                "reset",
                "spell",
                "sync",
            ],
            "",
        ),
        ("syntax case ", &["ignore", "match"], ""),
        ("autocmd BufEnt", &["BufEnter"], "event"),
        ("doautocmd BufEnt", &["BufEnter"], "event"),
        ("augroup ", &["END"], "augroup"),
        (
            "profile ",
            &["continue", "dump", "file", "func", "pause", "start", "stop"],
            "",
        ),
        ("profile f", &["file", "func"], ""),
        (
            "history ",
            &[
                "/", ":", "=", ">", "?", "@", "all", "cmd", "debug", "expr", "input", "search",
            ],
            "history",
        ),
        (
            "set nowr",
            &["wrap", "wrapscan", "write", "writeany", "writebackup"],
            "",
        ),
        ("command -n", &["nargs"], ""),
        ("command -complete=cu", &["custom", "customlist"], ""),
        ("delcommand ", &[], ""),
        ("filter ", &[], ""),
        ("filter /x/ tabn", &["tabnew", "tabnext"], "command"),
        ("global/x/tabn", &["tabnew", "tabnext"], "command"),
        ("s/a/b/g", &[], ""),
        ("isearch /x/ ", &[], ""),
        ("e +tabn", &["tabnew", "tabnext"], "command"),
        ("e ++en", &["encoding="], ""),
        ("map <buf", &["<buffer>"], "mapping"),
        ("mapclear ", &["<buffer>"], "mapclear"),
        ("menu ", &[], ""),
        ("emenu ", &[], ""),
        ("set fileformat=", &["unix", "dos", "mac"], ""),
        (
            "set whichwrap=",
            &["b", "s", "h", "l", "<", ">", "[", "]", "~"],
            "",
        ),
        (
            "set cpo+=",
            &[
                "b", "C", "d", "D", "E", "f", "i", "I", "J", "K", "l", "L", "m", "M", "n", "o",
                "O", "p", "P", "q", "r", "R", "S", "t", "u", "v", "W", "x", "X", "y", "Z", "$",
                "!", "%", "+", ">", ";", "~",
            ],
            "",
        ),
        ("set diffopt=algorithm:p", &["patience"], ""),
        (
            "set diffopt=inline:",
            &["none", "simple", "char", "word"],
            "",
        ),
        ("set eventignore=BufWritePo", &["BufWritePost"], ""),
        ("set eventignore=-BufWritePo", &["-BufWritePost"], ""),
        ("set listchars=ta", &["tab"], ""),
        ("set encoding=utf-1", &[], ""),
        (
            "set wildmode=",
            &["full", "longest", "list", "lastused", "noselect"],
            "",
        ),
        ("set t_", &[], "option"),
        ("set <t_", &[], "option"),
        ("scriptnames ", &[], "scriptnames"),
    ];
    for &(line, matches, kind) in cases {
        assert_eq!(complete(line, "cmdline"), strings(matches), "{line:?}");
        assert_eq!(complete_type(line), kind, "{line:?}");
    }
}
