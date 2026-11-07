//! The editor's refusals that say something, each with every message it
//! puts out and what was (not) done: the window family's E36/E1513 (each
//! split, status-line and window-bar site), `:ldo`/`:bufdo` leaving a
//! 'winfixbuf' window, `getcwd()`/`haslocaldir()`'s scope arguments,
//! `resolve()`'s link cycle, `writefile()`'s write errors, callbacks,
//! 'spellsuggest' expressions, `:help` without a help file, tab-stop
//! values, `<Char->` key names, menu paths, `printf()` arguments,
//! `strchars()`' strict bool, `:autocmd` event names, and manual folds.
//!
//! Every row reads [`Probe::messages_of`], so a message reported twice, or
//! not at all, moves the row. Recorded before these refusals answered
//! `Err` instead of a `false`/`None`/`0` after their message.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};

/// Split with `cmd` until a split is refused: exactly one refusal.
fn split_until_refused(fx: &Probe, cmd: &str) -> String {
    fx.run("unlet! g:stop");
    let line = format!(
        "for i in range(200) | if !exists('g:stop') | let w = winnr('$') | {cmd} | \
         if winnr('$') == w | let g:stop = 1 | endif | endif | endfor"
    );
    let got = fx.messages_of(&line);
    fx.run("unlet! g:stop");
    got
}

fn window_home(fx: &Probe) {
    // A float left current keeps `home`'s `:only` from closing the rest.
    fx.run(
        "for w in nvim_list_wins() | if win_gettype(w) == 'popup' | \
         call nvim_win_close(w, 1) | endif | endfor",
    );
    fx.home();
    // 'winminheight' before 'winheight': neither may pass the other.
    fx.run("set ls& wmh& wh& wmw& wiw& winbar& nowfb ea&");
    fx.settle();
}

/// Leave the only window too short to give up a line: 'laststatus' 1
/// and 'winminheight' its whole height.
fn no_room_to_split(fx: &Probe) {
    fx.run("set ls=1");
    fx.run("exe 'set wh=' .. winheight(0) .. ' wmh=' .. winheight(0)");
}

#[test]
fn window_room_refusals() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    window_home(&fx);

    no_room_to_split(&fx);
    fx.check_messages("split", "E36: Not enough room");
    fx.check("winnr('$')", "1");
    window_home(&fx);

    // Horizontal splits until no window can give up a line.
    fx.run("set ls=2");
    let got = split_until_refused(&fx, "split");
    fx.check(&format!("{got:?}"), "'E36: Not enough room'");
    let wins = fx.eval("winnr('$')");
    fx.check_messages("split", "E36: Not enough room");
    fx.check("winnr('$')", &wins);
    window_home(&fx);

    // Vertical splits until no window can give up a column.
    let got = split_until_refused(&fx, "vsplit");
    fx.check(&format!("{got:?}"), "'E36: Not enough room'");
    let wins = fx.eval("winnr('$')");
    fx.check_messages("vsplit", "E36: Not enough room");
    fx.check("winnr('$')", &wins);
    // Without 'equalalways' only the current window's room counts.
    fx.run("set noea");
    fx.check_messages("vsplit", "E36: Not enough room");
    fx.check("winnr('$')", &wins);
    window_home(&fx);

    // No status lines, every line taken: the last window cannot get one.
    fx.run("set ls=0");
    let _ = split_until_refused(&fx, "split");
    let wins = fx.eval("winnr('$')");
    fx.check_messages("set ls=2", "E36: Not enough room");
    fx.check("[winnr('$'), &ls]", &format!("[{wins}, 2]"));
    // Nor a window bar.
    fx.check_messages("set winbar=x", "E36: Not enough room");
    fx.check("winnr('$')", &wins);
    window_home(&fx);

    // A one-line float cannot take a window bar.
    fx.run(
        "let g:f = nvim_open_win(0, 1, {'relative': 'editor', 'row': 1, 'col': 1, \
         'width': 5, 'height': 1})",
    );
    fx.check_messages("setlocal winbar=x", "E36: Not enough room");
    fx.check("winnr('$')", "2");
    window_home(&fx);
}

#[test]
fn winfixbuf_refusals() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    window_home(&fx);
    let e1513 = "E1513: Cannot switch buffer. 'winfixbuf' is enabled";

    fx.run("set wfb");
    fx.run("let g:b = bufnr()");
    fx.check_messages("enew", e1513);
    fx.check("bufnr() == g:b", "1");
    window_home(&fx);

    // `:ldo` would have to leave the location list's own window.
    fx.run("call setloclist(0, [{'filename': 'a.c', 'lnum': 1}])");
    fx.run("set wfb");
    fx.run("let g:b = bufnr()");
    fx.check_messages("ldo echo", e1513);
    fx.check("[bufnr() == g:b, &l:wfb]", "[1, 1]");
    window_home(&fx);

    // `:bufdo` splits off a window without 'winfixbuf' -- unless there is
    // no room.
    fx.run("badd b.c");
    fx.run("set wfb");
    no_room_to_split(&fx);
    fx.check_messages("bufdo echo", &format!("E36: Not enough room\n{e1513}"));
    fx.check("winnr('$')", "1");
    window_home(&fx);
    fx.run("badd b.c");
    fx.run("set wfb");
    fx.check_messages("bufdo let g:last = bufname()", "none");
    fx.check("[winnr('$'), g:last, &l:wfb]", "[2, 'b.c', 0]");
    window_home(&fx);
}

#[test]
fn eval_file_refusals() {
    let sandbox = Sandbox::dir("eval_file_refusals");
    let fx = Probe::new(sandbox.editor());
    fx.home();
    for (cmd, want) in [
        ("let g:r = haslocaldir('x')", "E474: Invalid argument"),
        ("let g:r = haslocaldir(-2)", "E474: Invalid argument"),
        (
            "let g:r = haslocaldir(0, 99)",
            "E5000: Cannot find tab number.",
        ),
        (
            "let g:r = haslocaldir(0, -1)",
            "E5001: Higher scope cannot be -1 if lower scope is >= 0.",
        ),
        (
            "let g:r = haslocaldir(99, 0)",
            "E5002: Cannot find window number.",
        ),
        ("let g:r = getcwd(99)", "E5002: Cannot find window number."),
        ("let g:r = getcwd(1, 99)", "E5000: Cannot find tab number."),
        ("let g:r = getcwd(1, 'x')", "E474: Invalid argument"),
    ] {
        fx.run("let g:r = 'untouched'");
        fx.check_messages(cmd, want);
        fx.check("g:r", if cmd.contains("getcwd") { "''" } else { "0" });
    }

    std::os::unix::fs::symlink("loop", sandbox.path("loop")).expect("a link cycle");
    fx.run("let g:r = 'untouched'");
    fx.check_messages(
        "let g:r = resolve('loop')",
        "E655: Too many symbolic links (cycle?)",
    );
    fx.check("g:r", "''");

    let e80 = "E80: Error while writing: no space left on device";
    fx.check_messages("let g:r = writefile(['a'], '/dev/full')", e80);
    fx.check("g:r", "-1");
    fx.check_messages("let g:r = writefile(0z0102, '/dev/full')", e80);
    fx.check("g:r", "-1");
    fx.check_messages("let g:r = writefile(['a', 'b'], '/dev/full', 'b')", e80);
    fx.check("g:r", "-1");
    fx.settle();
}

#[test]
fn eval_callback_and_spell_refusals() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fx.home();
    let e921 = "E921: Invalid callback argument";
    fx.check_messages("let g:r = timer_start(0, 1)", e921);
    fx.check_messages("call prompt_setcallback(bufnr(), 1)", e921);
    fx.check_messages(
        "let g:r = setqflist([], ' ', {'quickfixtextfunc': 1})",
        e921,
    );
    fx.check("g:r", "0");
    fx.run("set qftf&");

    fx.run("set spellsuggest=expr:[[1]]");
    fx.check_messages(
        "let g:r = spellsuggest('helo')",
        "E5700: Expression from 'spellsuggest' must yield lists with exactly two values",
    );
    fx.check("g:r", "[]");
    fx.run("set spellsuggest&");
    fx.settle();
}

#[test]
fn option_value_and_key_refusals() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fx.home();
    for (cmd, want) in [
        (
            "set vts=1,0",
            "E487: Argument must be positive\nE474: Invalid argument: vts=1,0",
        ),
        ("set vts=-1", "E474: Invalid argument: vts=-1"),
        ("set vts=a", "E474: Invalid argument: vts=a"),
        ("set vsts=2,x", "E474: Invalid argument: vsts=2,x"),
    ] {
        fx.run("set vts=8 vsts=4");
        fx.check_messages(cmd, want);
        fx.check("[&vts, &vsts]", "['8', '4']");
    }
    fx.run("set vts& vsts&");

    // Said twice, once by the length scan and once by the conversion; the
    // name is then kept as typed.
    fx.check_messages(
        "let g:r = \"\\<char-x>\"",
        "E474: Invalid argument\nE474: Invalid argument",
    );
    fx.check("g:r", "'<char-x>'");
    fx.check_messages("map <char-> x", "E474: Invalid argument");
    fx.check("maparg('<char->')", "'x'");
    fx.run("unmap <char->");
    // Not a key name at all: no message, kept as typed.
    fx.check_messages("let g:r = \"\\<Nope>\"", "none");
    fx.check("g:r", "'<Nope>'");

    let e766 = "E766: Insufficient arguments for printf()";
    fx.check_messages("let g:r = printf('%d')", e766);
    fx.check_messages("let g:r = printf('%s %s', 1)", e766);
    fx.check_messages(
        "let g:r = strchars('abc', 2)",
        "E1023: Using a Number as a Bool: 2",
    );
    fx.check("g:r", "0");
    // The argument check refuses 2 before the strict read sees it.
    fx.run("let g:r = 'untouched'");
    fx.check_messages(
        "let g:r = charidx('abc', 1, 2)",
        "E1212: Bool required for argument 3",
    );
    fx.check("g:r", "-1");
    fx.check_messages(
        "let g:r = utf16idx('abc', 1, 2)",
        "E1212: Bool required for argument 3",
    );
    fx.check("g:r", "-1");
    fx.settle();
}

#[test]
fn command_refusals() {
    let sandbox = Sandbox::dir("command_refusals");
    let fx = Probe::new(sandbox.editor());
    fx.home();

    fx.check_messages(
        "autocmd *x echo",
        "E215: Illegal character after *: *x echo",
    );
    fx.check_messages(
        "autocmd Nope echo",
        "E216: No such group or event: Nope echo",
    );
    fx.run("augroup Refusals | augroup END");
    fx.check_messages(
        "autocmd Refusals Nope echo",
        "E216: No such event: Nope echo",
    );
    fx.run("augroup! Refusals");
    fx.check("exists('#Nope')", "0");

    fx.run("call setline(1, ['a', 'b', 'c'])");
    fx.run("set foldmethod=indent");
    let e350 = "E350: Cannot create fold with current 'foldmethod'";
    fx.check_messages("normal zfj", e350);
    fx.check_messages(
        "normal zd",
        "E351: Cannot delete fold with current 'foldmethod'",
    );
    fx.check_messages("1,2fold", e350);
    fx.run("set foldmethod&");
    fx.run("%delete _");

    fx.run("menu Refusals.Item :echo<CR>");
    for (cmd, want) in [
        ("popup Nope", "E337: Menu not found - check menu names"),
        (
            "popup Refusals.Item.Deeper",
            "E327: Part of menu-item path is not sub-menu",
        ),
        (
            "popup Refusals.Item",
            "E336: Menu path must lead to a sub-menu",
        ),
    ] {
        fx.check_messages(cmd, want);
    }
    fx.run("aunmenu Refusals");

    fx.run("helpclose");
    fx.run("set helpfile=/nonexistent/help.txt");
    fx.check_output("help", "\nHelp file \"/nonexistent/help.txt\" not found");
    fx.check("[winnr('$'), &buftype]", "[1, '']");
    fx.run("set helpfile&");
    fx.home();
    fx.settle();
}

/// `setcellwidths()`'s list checks, each message once and the old table
/// kept.
#[test]
fn cell_width_refusals() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fx.run("call setcellwidths([[0x2103, 0x2103, 2]])");
    for (call, want) in [
        ("call setcellwidths(1)", "E714: List required"),
        (
            "call setcellwidths([1])",
            "E1109: List item 0 is not a List",
        ),
        (
            "call setcellwidths([[0x100, 0x101, 1], [0x101, 0x102, 1]])",
            "E1113: Overlapping ranges for 0x101",
        ),
        (
            "call setcellwidths([[0x10, 0x20, 1]])",
            "E1114: Only values of 0x80 and higher supported",
        ),
        (
            "call setcellwidths([[0x200, 0x100, 1]])",
            "E1111: List item 0 range invalid",
        ),
        (
            "call setcellwidths([[0x100, 0x101, 3]])",
            "E1112: List item 0 cell width invalid",
        ),
        (
            "call setcellwidths([[0x100, 0x101]])",
            "E1110: List item 0 does not contain 3 numbers",
        ),
        (
            "call setcellwidths([[0x100, 0x101, 1, 1]])",
            "E1110: List item 0 does not contain 3 numbers",
        ),
        (
            "call setcellwidths([[0x100, 'x', 1]])",
            "E1110: List item 0 does not contain 3 numbers",
        ),
        ("call setcellwidths([[0x2103, 0x2103, 1]])", "none"),
    ] {
        fx.check_messages(call, want);
    }
    fx.check("getcellwidths()", "[[8451, 8451, 1]]");
    fx.run("call setcellwidths([])");
    fx.settle();
}

/// The shell's refusals: an empty 'shell' for `system()`, `:!` and a
/// backtick expansion, with `v:shell_error` after each.
#[test]
fn shell_refusals() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fx.run("let g:rf_save_sh = &shell");
    fx.run("set shell=");
    fx.check_messages("call system('true')", "none");
    fx.check("v:shell_error", "-1");
    fx.check_messages("!true", "E91: 'shell' option is empty");
    fx.check("v:shell_error", "-1");
    fx.check_messages("echo expand('`true`')", "E91: 'shell' option is empty");
    fx.check_messages("r !true", "E91: 'shell' option is empty");
    fx.run("let &shell = g:rf_save_sh | unlet g:rf_save_sh");
    fx.check_messages("call system('exit 3')", "none");
    fx.check("v:shell_error", "3");
    fx.settle();
}

/// The Lua converter's and executor's refusals, each said once: a table
/// with mixed keys, a value of no Vimscript type, a userdata that is not
/// `vim.NIL`, a `:command` callback that errors, and the answer each leaves.
#[test]
fn lua_refusals() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    for (cmd, want) in [
        (
            "let g:lr = luaeval('{1, a = 2}')",
            "E5100: Cannot convert given Lua table: table should contain either only integer keys or only string keys",
        ),
        (
            "let g:lr = luaeval('coroutine.create(function() end)')",
            "E5101: Cannot convert given Lua type",
        ),
        (
            "let g:lr = luaeval('io.stdout')",
            "E5101: Cannot convert given Lua type",
        ),
        (
            "let g:lr = luaeval('{{1, a = 2}}')",
            "E5100: Cannot convert given Lua table: table should contain either only integer keys or only string keys",
        ),
        ("let g:lr = luaeval('vim.NIL')", "none"),
        ("let g:lr = v:lua.vim.fn.type(1)", "none"),
    ] {
        fx.check_messages(cmd, want);
    }
    fx.check("exists('g:lr')", "1");
    fx.run("unlet! g:lr");
    fx.run("lua vim.api.nvim_create_user_command('LrFail', function() error('lr boom') end, {})");
    let got = fx.messages_of("LrFail");
    let first = got.lines().next().unwrap_or_default().to_owned();
    assert_eq!(first, "Lua :command callback: [string \":lua\"]:1: lr boom");
    fx.run("delcommand LrFail");
    fx.settle();
}

/// The channel's refusal to close a stream it does not have, the dynamic
/// loader's two, and the directory `:write ++p` could not make.
#[test]
fn channel_libcall_and_mkdir_refusals() {
    let sandbox = Sandbox::dir("refusals_os");
    let fx = Probe::new(sandbox.editor());
    fx.check_messages("echo rpcstop(999)", "E900: Invalid channel id");
    fx.check_messages("call libcall('/nonexistent/libno.so', 'f', '')", "dlerror = \"/nonexistent/libno.so: cannot open shared object file: No such file or directory\"\nE364: Library call failed for \"f()\"");
    fx.check_messages("call libcall('libc.so.6', 'rt_no_such_symbol', '')", "dlerror = \"/nix/store/ias8xacs1h3jy7xgwi2awvim61k2ji6c-glibc-2.42-67/lib/libc.so.6: undefined symbol: rt_no_such_symbol\"\nE364: Library call failed for \"rt_no_such_symbol()\"");
    fx.check_messages("echo serverstop('/nonexistent/rt.sock')", "none");
    sandbox.write("blocker", b"");
    fx.check_messages(
        "write ++p blocker/sub/x.txt",
        "E739: Cannot create directory blocker: file already exists",
    );
    fx.check_messages("write ++p sub/", "E32: No file name");
    fx.settle();
    fx.home();
}
