//! The file searcher: `findfile()`, `finddir()`, `:find`, `:cd` over
//! 'cdpath', `<cfile>` and `file_in_path` completion, over a tree of the
//! case's own.
//!
//! `find_file_in_path` and `find_directory_in_path` sit under all of these,
//! `vim_findfile`'s stack under them: downward `**` with its depth limit,
//! upward `;` with its stop directories, 'suffixesadd', the empty 'path'
//! entry that means the current directory, and the order a `count` of -1
//! lists every match in. `file_name_at_cursor` answers `<cfile>`.
//!
//! Written against the behaviour before the searcher's pointer interfaces go
//! (`find_file_in_path(_option)` answering an owned name, `expand` over a
//! slice); answers are relative to the sandbox the case stands in, or have
//! its path masked, so a row that moves is a behaviour change.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};

/// `root/` with a tree three levels deep. `target.txt` sits at two depths so
/// a count of -1 has an order to show; `deep.txt` only at the bottom, so a
/// depth limit can miss it; `notes.md` only for 'suffixesadd'.
fn tree(name: &str) -> Sandbox {
    let sandbox = Sandbox::dir(name);
    for dir in ["root/a/b/c", "root/x", "root/src/inc"] {
        sandbox.mkdir(dir);
    }
    for file in [
        "root/target.txt",
        "root/a/target.txt",
        "root/a/b/c/deep.txt",
        "root/x/notes.md",
        "root/src/inc/defs.h",
        "root/src/main.c",
    ] {
        sandbox.write(file, b"x\n");
    }
    sandbox
}

/// Back to the defaults this file changes, and to one empty window.
fn restore(fx: &Probe) {
    fx.home();
    fx.run("set path& suffixesadd& cdpath& isfname&");
}

/// `expr`'s value with the sandbox's path written `<R>`: an upward search
/// answers absolute names, and where the temp directory is varies.
#[track_caller]
fn check_under(fx: &Probe, root: &str, expr: &str, want: &str) {
    assert_eq!(fx.eval(expr).replace(root, "<R>"), want, "{expr}");
}

#[test]
fn downward_search_lists_every_match_subdirectories_first() {
    let sandbox = tree("file_search_down");
    let fx = Probe::new(sandbox.editor());
    fx.expect_all(&[
        // The subdirectories are searched before the directory itself.
        ("findfile('target.txt', 'root/**')", "'root/a/target.txt'"),
        (
            "findfile('target.txt', 'root/**', -1)",
            "['root/a/target.txt', 'root/target.txt']",
        ),
        ("findfile('target.txt', 'root/**', 2)", "'root/target.txt'"),
        ("findfile('target.txt', 'root/**', 3)", "''"),
        ("findfile('deep.txt', 'root/**')", "'root/a/b/c/deep.txt'"),
        // `**N` bounds the depth: `c/` is three below `root/`.
        ("findfile('deep.txt', 'root/**2')", "''"),
        ("findfile('deep.txt', 'root/**3')", "'root/a/b/c/deep.txt'"),
        // A wildcard directory, one level.
        ("findfile('target.txt', 'root/*')", "'root/a/target.txt'"),
        ("findfile('missing.txt', 'root/**', -1)", "[]"),
    ]);
    restore(&fx);
}

#[test]
fn upward_search_stops_where_it_is_told() {
    let sandbox = tree("file_search_up");
    let fx = Probe::new(sandbox.editor());
    let root = sandbox.as_str();
    fx.run("cd root/a/b/c");
    // Every climb is bounded by the sandbox: unbounded, it reaches `/` and
    // finds whatever this machine keeps there.
    let top = format!("{root}/root");
    for (expr, want) in [
        (
            format!("findfile('target.txt', '.;{top}')"),
            "'<R>/root/a/target.txt'",
        ),
        (
            format!("findfile('target.txt', '.;{top}', -1)"),
            "['<R>/root/a/target.txt', '<R>/root/target.txt']",
        ),
        // A nearer stop directory ends the climb before `root/`.
        (
            format!("findfile('target.txt', '.;{top}/a', -1)"),
            "['<R>/root/a/target.txt']",
        ),
        (format!("finddir('b', '.;{top}')"), "'<R>/root/a/b'"),
        (format!("finddir('x', '.;{top}', -1)"), "['<R>/root/x']"),
    ] {
        check_under(&fx, root, &expr, want);
    }
    fx.run(&format!("cd {root}"));
    restore(&fx);
}

#[test]
fn path_entries_suffixes_and_absolute_names() {
    let sandbox = tree("file_search_path");
    let fx = Probe::new(sandbox.editor());
    let root = sandbox.as_str();
    fx.run("set path=root/src/inc,root/x,,");
    fx.expect_all(&[
        // The option's own entries, first match wins.
        ("findfile('defs.h')", "'root/src/inc/defs.h'"),
        // The empty entry is the current directory.
        ("findfile('root/src/main.c')", "'root/src/main.c'"),
        ("findfile('notes')", "''"),
    ]);
    fx.run("set suffixesadd=.md,.h");
    fx.check("findfile('notes')", "'root/x/notes.md'");
    fx.check("findfile('defs')", "'root/src/inc/defs.h'");
    // A name that says where it is ignores 'path'.
    fx.check(
        &format!("findfile('{root}/root/target.txt') ==# '{root}/root/target.txt'"),
        "1",
    );
    fx.check(
        "findfile('./root/target.txt', 'nowhere')",
        "'./root/target.txt'",
    );
    fx.check("findfile('../target.txt', 'nowhere')", "''");
    restore(&fx);
}

#[test]
fn find_edits_the_match_and_complains_when_there_is_none() {
    let sandbox = tree("file_search_find");
    let fx = Probe::new(sandbox.editor());
    fx.run("set path=root/**");
    fx.check_error("find deep.txt", "none");
    fx.check("bufname()", "'root/a/b/c/deep.txt'");
    fx.check_error("2find target.txt", "none");
    fx.check("bufname()", "'root/target.txt'");
    fx.check_error(
        "find nothing.txt",
        "E345: Can''t find file \"nothing.txt\" in path",
    );
    fx.check_error(
        "3find target.txt",
        "E347: No more file \"target.txt\" found in path",
    );
    fx.check_error("find", "E471: Argument required: silent! find");
    restore(&fx);
}

#[test]
fn cdpath_finds_directories() {
    let sandbox = tree("file_search_cdpath");
    let fx = Probe::new(sandbox.editor());
    let root = sandbox.as_str();
    fx.run("set cdpath=root/src,root");
    fx.check_error("cd inc", "none");
    fx.check(&format!("getcwd() ==# '{root}/root/src/inc'"), "1");
    fx.run(&format!("cd {root}"));
    fx.check_error("cd a", "none");
    fx.check(&format!("getcwd() ==# '{root}/root/a'"), "1");
    fx.run(&format!("cd {root}"));
    // E344 is given, then overwritten by the command's own failure.
    fx.check_error("cd nowhere", "E472: Command failed");
    fx.run(&format!("cd {root}"));
    restore(&fx);
}

#[test]
fn the_file_name_under_the_cursor() {
    let sandbox = tree("file_search_cfile");
    let fx = Probe::new(sandbox.editor());
    fx.run(
        "call setline(1, ['see root/src/main.c:12 here', 'x \"root/x/notes.md\", y', '<root/a/target.txt>'])",
    );
    // ':' is not in the default 'isfname'.
    for (line, col, want) in [
        ("1", "5", "'root/src/main.c'"),
        ("1", "1", "'see'"),
        ("2", "6", "'root/x/notes.md'"),
        ("3", "3", "'root/a/target.txt'"),
    ] {
        fx.run(&format!("call cursor({line}, {col})"));
        fx.check("expand('<cfile>')", want);
    }
    fx.run("call cursor(1, 5)");
    fx.run("set isfname+=:");
    fx.check("expand('<cfile>')", "'root/src/main.c:12'");
    fx.run("set isfname&");
    // `gF` takes the `:12` as a line number; the file has one line.
    fx.run("set path=,");
    fx.check_error("normal! gF", "none");
    fx.check("[bufname(), line('.')]", "['root/src/main.c', 1]");
    restore(&fx);
}

#[test]
fn completion_lists_the_names_the_path_reaches() {
    let sandbox = tree("file_search_complete");
    let fx = Probe::new(sandbox.editor());
    fx.run("set path=root/**");
    fx.expect_all(&[
        // Two `target.txt`s: each is shortened to what tells it apart.
        ("getcompletion('tar', 'file_in_path')", "['./root/target.txt', 'a/target.txt']"),
        ("getcompletion('de', 'file_in_path')", "['deep.txt', 'defs.h']"),
        (
            "getcompletion('', 'file_in_path')",
            "['./root/target.txt', 'a/', 'a/target.txt', 'b/', 'c/', 'deep.txt', 'defs.h', 'inc/', 'main.c', 'notes.md', 'src/', 'x/']",
        ),
    ]);
    restore(&fx);
}
