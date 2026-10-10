//! Help lookup and `:helptags`, against a help tree of the case's own.
//!
//! Every case points 'runtimepath' and 'helpfile' at a sandbox holding one
//! `doc/` directory, so a lookup searches tags this file wrote and nothing
//! else: the order `getcompletion(.., 'help')` answers is `find_help_tags`'
//! sort on `help_heuristic`'s score, the subject having gone through
//! `escape_subject`'s rewrites (`CTRL-X`, `i_<Esc>`, `'option'`, `:cmd`,
//! `v_*`, `^X`), and `:help` jumps to the first of them. Paths in an answer
//! are made relative to the sandbox.
//!
//! Written against the behaviour before the help code leaves `IObuff` and
//! `FILE *`; a row that moves is a behaviour change.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};

/// The English help file. Tags chosen to exercise each scoring rule: a
/// `+feature`, a mid-word match, letters against punctuation, case.
const DOC: &str = "*demo.txt*\tDemo help\n\
\n\
*demo-intro* *:democmd* *'demoopt'* *i_CTRL-X_demo*\n\
Some text.\n\
\n\
*CTRL-X* *i_<Esc>* *v_demo* *demo()* *+demofeature* *demo-intro-more*\n\
*:demo* *'dm'* *g:demo_var* *demo.lua* *E9999* *Demo-Upper* *xdemo*\n\
*star* ***  *bar* *|* *quote* *quote\"*\n";

/// The same subject in Chinese, for 'helplang'.
const DOC_CN: &str = "*demo.txt*\tDemo help (cn)\n\
\n\
*demo-intro*\n\
Chinese text.\n";

/// A sandbox with `doc/demo.txt` (and `doc/demo.cnx`), tagged, and the
/// editor looking only there.
fn help_tree(name: &str) -> Sandbox {
    let sandbox = Sandbox::dir(name);
    sandbox.mkdir("doc");
    sandbox.write("doc/demo.txt", DOC.as_bytes());
    sandbox.write("doc/demo.cnx", DOC_CN.as_bytes());
    sandbox
}

/// Point the editor at the sandbox's tree and generate its tags.
fn look_only_here(fx: &Probe, root: &str) {
    fx.run(&format!(
        "set runtimepath={root} helpfile={root}/doc/demo.txt helplang="
    ));
    fx.run(&format!("helptags {root}/doc"));
}

/// Back to one window on an empty buffer, for the next case.
fn close_help(fx: &Probe) {
    fx.run("silent! helpclose");
    fx.home();
    fx.run("set runtimepath& helpfile& helplang&");
}

#[test]
fn helptags_writes_a_sorted_tags_file_per_language() {
    let sandbox = help_tree("help_tags");
    let fx = Probe::new(sandbox.editor());
    let root = sandbox.as_str();
    fx.run(&format!("set runtimepath={root}"));
    fx.check_error(&format!("helptags {root}/doc"), "none");
    fx.check("readfile('doc/tags')", "['''demoopt''\tdemo.txt\t/*''demoopt''*', '''dm''\tdemo.txt\t/*''dm''*', '+demofeature\tdemo.txt\t/*+demofeature*', ':demo\tdemo.txt\t/*:demo*', ':democmd\tdemo.txt\t/*:democmd*', 'CTRL-X\tdemo.txt\t/*CTRL-X*', 'Demo-Upper\tdemo.txt\t/*Demo-Upper*', 'E9999\tdemo.txt\t/*E9999*', 'bar\tdemo.txt\t/*bar*', 'demo()\tdemo.txt\t/*demo()*', 'demo-intro\tdemo.txt\t/*demo-intro*', 'demo-intro-more\tdemo.txt\t/*demo-intro-more*', 'demo.lua\tdemo.txt\t/*demo.lua*', 'demo.txt\tdemo.txt\t/*demo.txt*', 'g:demo_var\tdemo.txt\t/*g:demo_var*', 'i_<Esc>\tdemo.txt\t/*i_<Esc>*', 'i_CTRL-X_demo\tdemo.txt\t/*i_CTRL-X_demo*', 'quote\tdemo.txt\t/*quote*', 'quote\"\tdemo.txt\t/*quote\"*', 'star\tdemo.txt\t/*star*', 'v_demo\tdemo.txt\t/*v_demo*', 'xdemo\tdemo.txt\t/*xdemo*']");
    fx.check(
        "readfile('doc/tags-cn')",
        "['demo-intro\tdemo.cnx\t/*demo-intro*', 'demo.txt\tdemo.cnx\t/*demo.txt*']",
    );
    fx.check(
        "sort(readdir('doc'))",
        "['demo.cnx', 'demo.txt', 'tags', 'tags-cn']",
    );
    // `++t` adds a tag for the help-tags file itself.
    fx.check_error(&format!("helptags ++t {root}/doc"), "none");
    fx.check(
        "filter(readfile('doc/tags'), 'v:val =~ \"^help-tags\"')",
        "['help-tags\ttags\t1']",
    );
    // A tag defined twice is reported and the file is still written.
    sandbox.write("doc/dup.txt", b"*demo-intro* again\n");
    fx.check_error(
        &format!("helptags {root}/doc"),
        "E154: Duplicate tag \"demo-intro\" in file /tmp/nvim-unit-help_tags/doc/dup.txt",
    );
    sandbox.write("doc/dup.txt", b"no tags here\n");
    for (cmd, err) in [
        ("helptags", "E471: Argument required: silent! helptags"),
        ("helptags nosuchdir", "E150: Not a directory: nosuchdir"),
        ("helptags ++x doc", "E150: Not a directory: ++x doc"),
    ] {
        fx.check_error(cmd, err);
    }
    // A directory with no help files writes no tags file.
    sandbox.mkdir("empty");
    fx.check_error(
        &format!("helptags {root}/empty"),
        "E151: No match: /tmp/nvim-unit-help_tags/empty/**",
    );
    fx.check("readdir('empty')", "[]");
    fx.run("set runtimepath&");
}

#[test]
fn lookups_order_matches_by_score() {
    let sandbox = help_tree("help_lookup");
    let fx = Probe::new(sandbox.editor());
    look_only_here(&fx, sandbox.as_str());
    fx.expect_all(&[
        ("getcompletion('demo', 'help')", "[':demo', 'demo()', 'v_demo', 'demo.lua', 'demo.txt@en', ':democmd', 'demo.txt@cn', '''demoopt''', 'g:demo_var', 'demo-intro@en', 'demo-intro@cn', '+demofeature', 'demo-intro-more', 'i_CTRL-X_demo', 'Demo-Upper', 'xdemo']"),
        ("getcompletion('intro', 'help')", "['demo-intro@en', 'demo-intro@cn', 'demo-intro-more']"),
        ("getcompletion('Demo', 'help')", "['Demo-Upper', ':demo', 'demo()', 'v_demo', 'demo.lua', 'demo.txt@en', ':democmd', 'demo.txt@cn', '''demoopt''', 'g:demo_var', 'demo-intro@en', 'demo-intro@cn', '+demofeature', 'demo-intro-more', 'i_CTRL-X_demo', 'xdemo']"),
        ("getcompletion('dm', 'help')", "['''dm''']"),
        ("getcompletion('nosuchsubject', 'help')", "[]"),
    ]);
    // `escape_subject`'s rewrites: what the user types against the tag.
    for (subject, want) in [
        ("CTRL-X", "['CTRL-X', 'i_CTRL-X_demo']"),
        ("^X", "['CTRL-X', 'i_CTRL-X_demo']"),
        ("^x", "['CTRL-X', 'i_CTRL-X_demo']"),
        ("i_<Esc>", "['i_<Esc>']"),
        ("i_esc", "[]"),
        ("<Esc>", "['i_<Esc>']"),
        ("'demoopt'", "['''demoopt''']"),
        ("demoopt", "['''demoopt''']"),
        ("'dm", "['''dm''']"),
        (":democmd", "[':democmd']"),
        ("democmd", "[':democmd']"),
        (":demo", "[':demo', ':democmd', 'g:demo_var']"),
        ("v_demo", "['v_demo']"),
        ("i_CTRL-X_demo", "['i_CTRL-X_demo']"),
        ("i^x_demo", "['i_CTRL-X_demo']"),
        ("demo()", "['demo()']"),
        ("demo(", "['demo()']"),
        ("+demofeature", "['+demofeature']"),
        ("g:demo_var", "['g:demo_var']"),
        ("*", "['star']"),
        ("**", "[]"),
        ("|", "['bar']"),
        ("\"", "['quote', 'quote\"']"),
        ("quote\"", "[]"),
        ("E9999", "['E9999']"),
        ("demo.lua", "['demo.lua']"),
        ("demo.*", "['demo.lua', 'demo.txt@en', 'demo.txt@cn']"),
        ("/demo", "[]"),
    ] {
        let quoted = subject.replace('\'', "''");
        fx.check(&format!("getcompletion('{quoted}', 'help')"), want);
    }
    close_help(&fx);
}

#[test]
fn help_jumps_to_the_best_match_and_reuses_its_window() {
    let sandbox = help_tree("help_jump");
    let fx = Probe::new(sandbox.editor());
    look_only_here(&fx, sandbox.as_str());
    let at = "[fnamemodify(bufname(), ':t'), &buftype, line('.'), col('.'), winnr('$'), &modifiable, &readonly]";
    fx.check_error("help", "E149: No help for help.txt");
    fx.check(at, "['', '', 1, 1, 1, 1, 0]");
    for (subject, want) in [
        ("demo-intro", "['demo.txt', 'help', 3, 1, 2, 0, 1]"),
        ("CTRL-X", "['demo.txt', 'help', 6, 1, 2, 0, 1]"),
        ("'dm'", "['demo.txt', 'help', 7, 9, 2, 0, 1]"),
        (":demo", "['demo.txt', 'help', 7, 1, 2, 0, 1]"),
        ("demo", "['demo.txt', 'help', 7, 1, 2, 0, 1]"),
        ("E9999", "['demo.txt', 'help', 7, 40, 2, 0, 1]"),
        ("intro", "['demo.txt', 'help', 3, 1, 2, 0, 1]"),
    ] {
        fx.run(&format!("help {subject}"));
        fx.check(at, want);
    }
    fx.check_error("help nosuchsubject", "E149: No help for nosuchsubject");
    fx.check_error(
        "help demo-intro extra",
        "E149: No help for demo-intro extra",
    );
    fx.run("help demo-intro | let g:after_bar = 1");
    fx.check("get(g:, 'after_bar', 0)", "1");
    fx.run("helpclose");
    fx.check("winnr('$')", "1");
    fx.check_error("helpclose", "none");
    close_help(&fx);
}

#[test]
fn helplang_prefers_a_translation_and_at_xx_forces_one() {
    let sandbox = help_tree("help_lang");
    let fx = Probe::new(sandbox.editor());
    look_only_here(&fx, sandbox.as_str());
    let at = "[fnamemodify(bufname(), ':t'), line('.')]";
    fx.check(
        "getcompletion('demo-intro', 'help')",
        "['demo-intro@en', 'demo-intro@cn', 'demo-intro-more']",
    );
    fx.run("set helplang=cn");
    fx.check(
        "getcompletion('demo-intro', 'help')",
        "['demo-intro', 'demo-intro@en', 'demo-intro-more']",
    );
    fx.run("help demo-intro");
    fx.check(at, "['demo.cnx', 3]");
    fx.run("help demo-intro@en");
    fx.check(at, "['demo.txt', 3]");
    fx.run("set helplang=en,cn");
    fx.run("help demo-intro");
    fx.check(at, "['demo.txt', 3]");
    fx.run("help demo-intro@cn");
    fx.check(at, "['demo.cnx', 3]");
    fx.check_error("help demo-intro@xx", "E661: No ''xx'' help for demo-intro");
    fx.run("set helplang=xx");
    fx.run("help demo-intro");
    fx.check(at, "['demo.txt', 3]");
    close_help(&fx);
}
