//! `ex_cmds2`'s commands, observed from Ex: `:argdo` and its siblings
//! (`:windo`, `:bufdo`, `:tabdo`, `:cdo`/`:cfdo`, `:ldo`/`:lfdo`) — which
//! entries they visit, in what order, under a range and with a modified
//! buffer; the "may I abandon this buffer" answers (`E37`, `'hidden'`,
//! `:hide`, `!`, 'autowrite'); `:compiler`; `:checktime`; `:drop`; and the
//! script-host commands without a provider.
//!
//! A full editor (the unit lane's) because every one of them walks real
//! windows, buffers and tab pages. Written against the behaviour before the
//! module leaves raw pointers; a row that moves is a behaviour change.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};

/// [`Probe::home`], no argument list and no quickfix list, with every option
/// this file sets at its default.
fn reset(fx: &Probe) {
    fx.home();
    for cmd in [
        "silent! argdelete *",
        "call setqflist([], 'f')",
        "call setloclist(0, [], 'f')",
        "set hidden& autowrite& autowriteall& eventignore& runtimepath& makeprg& errorformat& autoread&",
        "let g:seen = []",
    ] {
        fx.run(cmd);
    }
}

/// The files `a.txt`, `b.txt`, `c.txt`, one line each, in the sandbox.
fn files(sandbox: &Sandbox) {
    for name in ["a", "b", "c"] {
        sandbox.write(&format!("{name}.txt"), format!("{name}\n").as_bytes());
    }
}

const SEEN: &str = "g:seen";
const NOTE: &str = "call add(g:seen, expand('%:t'))";

#[test]
fn argdo_and_bufdo_visit_each_entry_once_in_order() {
    let sandbox = Sandbox::dir("ex_cmds2_argdo");
    let fx = Probe::new(sandbox.editor());
    files(&sandbox);
    reset(&fx);
    fx.run("args a.txt b.txt c.txt");
    fx.run(&format!("argdo {NOTE}"));
    fx.check(SEEN, "['a.txt', 'b.txt', 'c.txt']");
    fx.check("[argidx(), expand('%:t')]", "[2, 'c.txt']");
    fx.run("let g:seen = []");
    fx.run(&format!("2,3argdo {NOTE}"));
    fx.run(&format!("1argdo {NOTE}"));
    fx.check(SEEN, "['b.txt', 'c.txt', 'a.txt']");
    fx.run("let g:seen = []");
    fx.run(&format!("bufdo {NOTE}"));
    fx.check(SEEN, "['a.txt', 'b.txt', 'c.txt']");
    fx.check(
        "map(getbufinfo({'buflisted': 1}), 'fnamemodify(v:val.name, \":t\")')",
        "['a.txt', 'b.txt', 'c.txt']",
    );
    // An unlisted buffer is skipped; a range is buffer numbers (which
    // depend on every case before this one, so they are asked for).
    fx.run("let g:seen = []");
    fx.run("badd d.txt | call setbufvar('d.txt', '&buflisted', 0)");
    fx.run(&format!("bufdo {NOTE}"));
    fx.run(&format!(
        "execute bufnr('b.txt') .. ',' .. bufnr('c.txt') .. \"bufdo {NOTE}\""
    ));
    fx.check(SEEN, "['a.txt', 'b.txt', 'c.txt', 'b.txt', 'c.txt']");
    // A modified buffer stops the walk unless 'hidden' or `!`.
    fx.run("let g:seen = []");
    fx.run("set nohidden");
    fx.run("buffer a.txt | call setline(1, 'changed')");
    fx.check_error(
        &format!("argdo {NOTE}"),
        "E37: No write since last change (add ! to override)",
    );
    fx.check(SEEN, "[]");
    fx.run("let g:seen = []");
    fx.run(&format!("argdo! {NOTE}"));
    fx.check(SEEN, "['a.txt', 'b.txt', 'c.txt']");
    fx.check(
        "map(['a.txt', 'b.txt', 'c.txt'], 'getbufvar(v:val, \"&modified\")')",
        "[0, 0, 0]",
    );
    fx.run("let g:seen = []");
    fx.run("set hidden");
    fx.run(&format!("bufdo {NOTE}"));
    fx.check(SEEN, "['a.txt', 'b.txt', 'c.txt']");
    // The command runs in each buffer; under `silent!` an error in it does
    // not stop the walk.
    fx.run("let g:seen = []");
    fx.check_error("argdo call add(g:seen, expand('%:t')) | if expand('%:t') == 'b.txt' | throw 'stop' | endif", "E605: Exception not caught: stop");
    fx.check(SEEN, "['a.txt', 'b.txt', 'c.txt']");
    fx.check_error("argdo", "E471: Argument required: silent! argdo");
    fx.check_error("5argdo echo", "E16: Invalid range: silent! 5argdo echo");
    reset(&fx);
    fx.check_error(
        &format!("argdo {NOTE}"),
        "E163: There is only one file to edit",
    );
    fx.check(SEEN, "[]");
    reset(&fx);
}

#[test]
fn windo_and_tabdo_visit_windows_and_tab_pages() {
    let sandbox = Sandbox::dir("ex_cmds2_windo");
    let fx = Probe::new(sandbox.editor());
    files(&sandbox);
    reset(&fx);
    fx.run("edit a.txt | split b.txt | split c.txt");
    fx.run(&format!("windo {NOTE}"));
    fx.check(SEEN, "['c.txt', 'b.txt', 'a.txt']");
    fx.check("[winnr(), winnr('$'), expand('%:t')]", "[3, 3, 'a.txt']");
    fx.run("let g:seen = []");
    fx.run(&format!("2windo {NOTE}"));
    fx.run(&format!("1,2windo {NOTE}"));
    fx.check(SEEN, "['b.txt', 'c.txt', 'b.txt']");
    fx.run("let g:seen = []");
    fx.run("tabnew b.txt | tabnew c.txt");
    fx.run("tabdo call add(g:seen, [tabpagenr(), expand('%:t'), winnr('$')])");
    fx.check(SEEN, "[[1, 'b.txt', 3], [2, 'b.txt', 1], [3, 'c.txt', 1]]");
    fx.check("tabpagenr()", "3");
    fx.run("let g:seen = []");
    fx.run("2tabdo call add(g:seen, tabpagenr())");
    fx.run("tabdo windo call add(g:seen, [tabpagenr(), winnr()])");
    fx.check(SEEN, "[2, [1, 1], [1, 2], [1, 3], [2, 1], [3, 1]]");
    fx.check_error("9tabdo echo", "E16: Invalid range: silent! 9tabdo echo");
    fx.check_error("9windo echo", "E16: Invalid range: silent! 9windo echo");
    reset(&fx);
}

#[test]
fn cdo_and_cfdo_visit_entries_and_files() {
    let sandbox = Sandbox::dir("ex_cmds2_cdo");
    let fx = Probe::new(sandbox.editor());
    files(&sandbox);
    reset(&fx);
    fx.run(
        "call setqflist([{'filename': 'a.txt', 'lnum': 1, 'text': 'one'}, \
         {'filename': 'a.txt', 'lnum': 1, 'text': 'two'}, \
         {'filename': 'b.txt', 'lnum': 1, 'text': 'three'}, \
         {'text': 'no file'}, \
         {'filename': 'c.txt', 'lnum': 1, 'text': 'four'}])",
    );
    let entry = "call add(g:seen, [expand('%:t'), getqflist({'idx': 0}).idx])";
    fx.run(&format!("cdo {entry}"));
    fx.check(
        SEEN,
        "[['a.txt', 1], ['a.txt', 2], ['b.txt', 3], ['c.txt', 5]]",
    );
    fx.run("let g:seen = []");
    fx.run(&format!("cfdo {entry}"));
    fx.check(SEEN, "[['a.txt', 1], ['b.txt', 3], ['c.txt', 5]]");
    fx.run("let g:seen = []");
    fx.run(&format!("2,3cdo {entry}"));
    fx.check(SEEN, "[['a.txt', 2], ['b.txt', 3]]");
    fx.run("let g:seen = []");
    fx.run(
        "call setloclist(0, [{'filename': 'c.txt', 'lnum': 1}, {'filename': 'b.txt', 'lnum': 1}])",
    );
    fx.run(&format!("ldo {NOTE}"));
    fx.run(&format!("lfdo {NOTE}"));
    fx.check(SEEN, "['c.txt', 'b.txt', 'c.txt', 'b.txt']");
    fx.run("call setqflist([], 'f')");
    fx.run("let g:seen = []");
    fx.check_error(&format!("cdo {NOTE}"), "none");
    fx.check(SEEN, "[]");
    reset(&fx);
}

#[test]
fn a_modified_buffer_is_abandoned_only_when_allowed() {
    let sandbox = Sandbox::dir("ex_cmds2_abandon");
    let fx = Probe::new(sandbox.editor());
    files(&sandbox);
    reset(&fx);
    fx.run("set nohidden");
    fx.run("edit a.txt | call setline(1, 'changed')");
    for (cmd, err) in [
        (
            "edit b.txt",
            "E37: No write since last change (add ! to override)",
        ),
        ("buffer 1", "none"),
        (
            "next",
            "E37: No write since last change (add ! to override)",
        ),
        (
            "enew",
            "E37: No write since last change (add ! to override)",
        ),
    ] {
        fx.check_error(cmd, err);
    }
    fx.check("[expand('%:t'), &modified]", "['a.txt', 1]");
    fx.run("hide edit b.txt");
    fx.check(
        "[expand('%:t'), getbufvar('a.txt', '&modified'), bufloaded('a.txt')]",
        "['b.txt', 1, 1]",
    );
    fx.run("buffer a.txt");
    fx.run("edit! b.txt");
    fx.check(
        "[expand('%:t'), getbufvar('a.txt', '&modified')]",
        "['b.txt', 0]",
    );
    // 'autowrite' writes the buffer being left.
    fx.run("edit! a.txt | call setline(1, 'autowritten')");
    fx.run("set autowrite");
    fx.check_error("next", "E163: There is only one file to edit");
    fx.check_error("edit b.txt", "none");
    fx.check(
        "[expand('%:t'), readfile('a.txt')]",
        "['b.txt', ['autowritten']]",
    );
    // A buffer that cannot be written is not autowritten.
    fx.run("edit! c.txt | setlocal buftype=nofile | call setline(1, 'scratch')");
    fx.check_error("edit a.txt", "none");
    fx.run("set noautowrite");
    fx.run("edit! a.txt");
    // Leaving through a split never needs an answer.
    fx.run("call setline(1, 'changed again')");
    fx.check_error("split b.txt", "none");
    fx.check("[winnr('$'), getbufvar('a.txt', '&modified')]", "[2, 1]");
    fx.check_error("close", "none");
    fx.check_error(
        "wincmd p | quit",
        "E37: No write since last change (add ! to override)",
    );
    reset(&fx);
}

#[test]
fn compiler_sets_makeprg_globally_or_for_the_buffer() {
    let sandbox = Sandbox::dir("ex_cmds2_compiler");
    let fx = Probe::new(sandbox.editor());
    sandbox.mkdir("compiler");
    sandbox.write(
        "compiler/demo.vim",
        b"let current_compiler = 'demo'\nCompilerSet makeprg=demo\\ build\nCompilerSet errorformat=%f:%l:%m\n",
    );
    reset(&fx);
    fx.run(&format!("set runtimepath={}", sandbox.as_str()));
    let state = "[&g:makeprg, &l:makeprg, &g:errorformat, &l:errorformat, \
                 get(b:, 'current_compiler', '-'), get(g:, 'current_compiler', '-')]";
    fx.run("compiler demo");
    fx.check(state, "['make', 'demo build', '%*[^\"]\"%f\"%*\\D%l: %m,\"%f\"%*\\D%l: %m,%-Gg%\\?make[%*\\d]: *** [%f:%l:%m,%-Gg%\\?make: *** [%f:%l:%m,%-G%f:%l: (Each undeclared identifier is reported only once,%-G%f:%l: for each function it appears in.),%-GIn file included from %f:%l:%c:,%-GIn file included from %f:%l:%c\\,,%-GIn file included from %f:%l:%c,%-GIn file included from %f:%l,%-G%*[ ]from %f:%l:%c,%-G%*[ ]from %f:%l:,%-G%*[ ]from %f:%l\\,,%-G%*[ ]from %f:%l,%f:%l:%c:%m,%f(%l):%m,%f:%l:%m,\"%f\"\\, line %l%*\\D%c%*[^ ] %m,%D%*\\a[%*\\d]: Entering directory %*[`'']%f'',%X%*\\a[%*\\d]: Leaving directory %*[`'']%f'',%D%*\\a: Entering directory %*[`'']%f'',%X%*\\a: Leaving directory %*[`'']%f'',%DMaking %*\\a in %f,%f|%l| %m', '%f:%l:%m', 'demo', '-']");
    fx.run("setlocal makeprg< errorformat< | unlet! b:current_compiler");
    fx.run("compiler! demo");
    fx.check(state, "['demo build', '', '%f:%l:%m', '', 'demo', 'demo']");
    fx.check_error("compiler nosuch", "E666: Compiler not supported: nosuch");
    fx.check_error("compiler", "none");
    fx.check("getcompletion('de', 'compiler')", "['demo']");
    fx.run("unlet! g:current_compiler b:current_compiler");
    reset(&fx);
}

#[test]
fn checktime_drop_and_the_script_hosts() {
    let sandbox = Sandbox::dir("ex_cmds2_misc");
    let fx = Probe::new(sandbox.editor());
    files(&sandbox);
    reset(&fx);
    fx.run("edit a.txt");
    fx.check_error("checktime", "none");
    fx.check_error("checktime 999", "none");
    fx.check_error("checktime a.txt", "none");
    // `:drop` edits a file not shown and jumps to the window that shows it.
    fx.run("drop b.txt");
    fx.check("[expand('%:t'), winnr('$')]", "['b.txt', 1]");
    fx.run("split c.txt");
    fx.run("drop b.txt");
    fx.check("[expand('%:t'), winnr(), winnr('$')]", "['b.txt', 2, 2]");
    fx.run("drop a.txt c.txt");
    fx.check("[expand('%:t'), argc(), winnr('$')]", "['a.txt', 2, 2]");
    fx.check_error("drop", "E471: Argument required: silent! drop");
    // No provider: each answers that it has none, without looking for one.
    fx.run("let g:loaded_python3_provider = 0 | let g:loaded_ruby_provider = 0 | let g:loaded_perl_provider = 0");
    for (cmd, err) in [
        (
            "python3 print(1)",
            "E319: No \"python3\" provider found. Run \":checkhealth vim.provider\"",
        ),
        (
            "py3file x.py",
            "E319: No \"python3\" provider found. Run \":checkhealth vim.provider\"",
        ),
        (
            "py3do return 1",
            "E319: No \"python3\" provider found. Run \":checkhealth vim.provider\"",
        ),
        (
            "ruby puts 1",
            "E319: No \"ruby\" provider found. Run \":checkhealth vim.provider\"",
        ),
        (
            "rubyfile x.rb",
            "E319: No \"ruby\" provider found. Run \":checkhealth vim.provider\"",
        ),
        (
            "rubydo $_",
            "E319: No \"ruby\" provider found. Run \":checkhealth vim.provider\"",
        ),
        (
            "perl print 1",
            "E319: No \"perl\" provider found. Run \":checkhealth vim.provider\"",
        ),
        (
            "perlfile x.pl",
            "E319: No \"perl\" provider found. Run \":checkhealth vim.provider\"",
        ),
        (
            "perldo $_",
            "E319: No \"perl\" provider found. Run \":checkhealth vim.provider\"",
        ),
    ] {
        fx.check_error(cmd, err);
    }
    fx.check_output(
        "silent! python3 print(1)",
        "\nE319: No \"python3\" provider found. Run \":checkhealth vim.provider\"",
    );
    reset(&fx);
}
