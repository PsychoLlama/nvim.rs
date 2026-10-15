//! `:mksession` and `:mkview`: the script each writes, line for line.
//!
//! `makeopens` writes the buffer list, the window layout and sizes, the tab
//! pages and the current directory; `put_view` each window's file, cursor
//! and folds; 'sessionoptions' and 'viewoptions' choose which. Each case
//! sets up a layout, writes the script and compares every line, with the
//! sandbox's path written `<DIR>`. The two errors for an existing file
//! without `!` close the file.
//!
//! Written against the behaviour before the writer leaves its `FILE *` and
//! `fprintf!` for a `Write`; a row that moves is a format change.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};

fn sources(name: &str) -> Sandbox {
    let sandbox = Sandbox::dir(name);
    let lines: String = (1..=30).map(|n| format!("line {n}\n")).collect();
    sandbox.write("a.txt", lines.as_bytes());
    sandbox.write("b.txt", b"b one\nb two\n");
    sandbox
}

/// Run `cmd` (which writes `out.vim`), and answer the script's lines.
///
/// The cases open their files with `:badd` + `:buffer` rather than `:edit`:
/// `:edit` from the empty first buffer would reuse buffer 1, whose
/// remembered cursor (`badd +N`) is whatever an earlier case in this shared
/// editor left there.
fn script(fx: &Probe, sandbox: &Sandbox, cmd: &str) -> Vec<String> {
    let _ = std::fs::remove_file(sandbox.path("out.vim"));
    fx.check_error(&format!("{cmd} out.vim"), "none");
    std::fs::read_to_string(sandbox.path("out.vim"))
        .unwrap_or_default()
        .replace(sandbox.as_str(), "<DIR>")
        .lines()
        .map(str::to_owned)
        .collect()
}

fn restore(fx: &Probe) {
    fx.home();
    // The screen size reaches the script (`winheight(0)` arithmetic, the
    // `resize` lines), and other cases in the shared editor change it.
    // A lone window in this editor may never have been given a height
    // (none is drawn), and `put_view` writes a different cursor line for a
    // window of height 0, so give it one.
    fx.run("set lines=24 columns=80");
    fx.run("resize 10 | resize 22");
    fx.run("set sessionoptions& viewoptions&");
    fx.run("unlet! g:SessNum g:SessStr g:sess_lower");
}

#[test]
fn buffers_and_the_current_directory() {
    let sandbox = sources("ex_session_buffers");
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    fx.run("set sessionoptions=buffers,curdir");
    fx.run("badd a.txt | buffer a.txt");
    fx.run("call cursor(12, 3)");
    fx.run("badd b.txt");
    assert_eq!(
        script(&fx, &sandbox, "mksession"),
        [
            "let SessionLoad = 1",
            "let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1",
            "let v:this_session=expand(\"<sfile>:p\")",
            "doautoall SessionLoadPre",
            "silent only",
            "cd <DIR>",
            "if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''",
            "  let s:wipebuf = bufnr('%')",
            "endif",
            "let s:shortmess_save = &shortmess",
            "set shortmess+=aoO",
            "badd +0 a.txt",
            "badd +0 b.txt",
            "argglobal",
            "%argdel",
            "edit a.txt",
            "argglobal",
            "let s:l = 12 - ((11 * winheight(0) + 11) / 22)",
            "if s:l < 1 | let s:l = 1 | endif",
            "keepjumps exe s:l",
            "normal! zt",
            "keepjumps 12",
            "normal! 01|",
            "if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0 && getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'",
            "  silent exe 'bwipe ' . s:wipebuf",
            "endif",
            "unlet! s:wipebuf",
            "set winheight=1 winwidth=20",
            "let &shortmess = s:shortmess_save",
            "let s:sx = expand(\"<sfile>:p:r\").\"x.vim\"",
            "if filereadable(s:sx)",
            "  exe \"source \" . fnameescape(s:sx)",
            "endif",
            "let &g:so = s:so_save | let &g:siso = s:siso_save",
            "set hlsearch",
            "doautoall SessionLoadPost",
            "unlet SessionLoad",
            "\" vim: set ft=vim :",
        ]
    );
    restore(&fx);
}

#[test]
fn split_windows_and_their_sizes() {
    let sandbox = sources("ex_session_windows");
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    fx.run("set sessionoptions=buffers,winsize,sesdir");
    fx.run("badd a.txt | buffer a.txt");
    fx.run("split b.txt");
    fx.run("vsplit a.txt");
    fx.run("call cursor(20, 1)");
    fx.run("resize 5");
    assert_eq!(
        script(&fx, &sandbox, "mksession"),
        [
            "let SessionLoad = 1",
            "let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1",
            "let v:this_session=expand(\"<sfile>:p\")",
            "doautoall SessionLoadPre",
            "silent only",
            "exe \"cd \" . escape(expand(\"<sfile>:p:h\"), ' ')",
            "if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''",
            "  let s:wipebuf = bufnr('%')",
            "endif",
            "let s:shortmess_save = &shortmess",
            "set shortmess+=aoO",
            "badd +0 a.txt",
            "badd +0 b.txt",
            "argglobal",
            "%argdel",
            "edit a.txt",
            "let s:save_splitbelow = &splitbelow",
            "let s:save_splitright = &splitright",
            "set splitbelow splitright",
            "wincmd _ | wincmd |",
            "split",
            "1wincmd k",
            "wincmd _ | wincmd |",
            "vsplit",
            "1wincmd h",
            "wincmd w",
            "wincmd w",
            "let &splitbelow = s:save_splitbelow",
            "let &splitright = s:save_splitright",
            "wincmd t",
            "let s:save_winminheight = &winminheight",
            "let s:save_winminwidth = &winminwidth",
            "set winminheight=0",
            "set winheight=1",
            "set winminwidth=0",
            "set winwidth=1",
            "exe '1resize ' . ((&lines * 5 + 12) / 24)",
            "exe 'vert 1resize ' . ((&columns * 40 + 40) / 80)",
            "exe '2resize ' . ((&lines * 5 + 12) / 24)",
            "exe 'vert 2resize ' . ((&columns * 39 + 40) / 80)",
            "exe '3resize ' . ((&lines * 16 + 12) / 24)",
            "argglobal",
            "balt b.txt",
            "let s:l = 20 - ((2 * winheight(0) + 2) / 5)",
            "if s:l < 1 | let s:l = 1 | endif",
            "keepjumps exe s:l",
            "normal! zt",
            "keepjumps 20",
            "normal! 0",
            "wincmd w",
            "argglobal",
            "if bufexists(fnamemodify(\"b.txt\", \":p\")) | buffer b.txt | else | edit b.txt | endif",
            "if &buftype ==# 'terminal'",
            "  silent file b.txt",
            "endif",
            "balt a.txt",
            "let s:l = 1 - ((0 * winheight(0) + 2) / 5)",
            "if s:l < 1 | let s:l = 1 | endif",
            "keepjumps exe s:l",
            "normal! zt",
            "keepjumps 1",
            "normal! 0",
            "wincmd w",
            "argglobal",
            "if bufexists(fnamemodify(\"a.txt\", \":p\")) | buffer a.txt | else | edit a.txt | endif",
            "if &buftype ==# 'terminal'",
            "  silent file a.txt",
            "endif",
            "balt b.txt",
            "let s:l = 1 - ((0 * winheight(0) + 8) / 16)",
            "if s:l < 1 | let s:l = 1 | endif",
            "keepjumps exe s:l",
            "normal! zt",
            "keepjumps 1",
            "normal! 0",
            "wincmd w",
            "exe '1resize ' . ((&lines * 5 + 12) / 24)",
            "exe 'vert 1resize ' . ((&columns * 40 + 40) / 80)",
            "exe '2resize ' . ((&lines * 5 + 12) / 24)",
            "exe 'vert 2resize ' . ((&columns * 39 + 40) / 80)",
            "exe '3resize ' . ((&lines * 16 + 12) / 24)",
            "if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0 && getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'",
            "  silent exe 'bwipe ' . s:wipebuf",
            "endif",
            "unlet! s:wipebuf",
            "set winheight=1 winwidth=20",
            "let &shortmess = s:shortmess_save",
            "let &winminheight = s:save_winminheight",
            "let &winminwidth = s:save_winminwidth",
            "let s:sx = expand(\"<sfile>:p:r\").\"x.vim\"",
            "if filereadable(s:sx)",
            "  exe \"source \" . fnameescape(s:sx)",
            "endif",
            "let &g:so = s:so_save | let &g:siso = s:siso_save",
            "set hlsearch",
            "doautoall SessionLoadPost",
            "unlet SessionLoad",
            "\" vim: set ft=vim :",
        ]
    );
    restore(&fx);
}

#[test]
fn tab_pages() {
    let sandbox = sources("ex_session_tabs");
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    fx.run("set sessionoptions=buffers,tabpages,sesdir");
    fx.run("badd a.txt | buffer a.txt");
    fx.run("tabedit b.txt");
    fx.run("tabnext 1");
    assert_eq!(
        script(&fx, &sandbox, "mksession"),
        [
            "let SessionLoad = 1",
            "let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1",
            "let v:this_session=expand(\"<sfile>:p\")",
            "doautoall SessionLoadPre",
            "silent only",
            "silent tabonly",
            "exe \"cd \" . escape(expand(\"<sfile>:p:h\"), ' ')",
            "if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''",
            "  let s:wipebuf = bufnr('%')",
            "endif",
            "let s:shortmess_save = &shortmess",
            "set shortmess+=aoO",
            "badd +0 a.txt",
            "badd +0 b.txt",
            "argglobal",
            "%argdel",
            "set stal=2",
            "tabnew +setlocal\\ bufhidden=wipe",
            "tabrewind",
            "edit a.txt",
            "argglobal",
            "let s:l = 1 - ((0 * winheight(0) + 10) / 21)",
            "if s:l < 1 | let s:l = 1 | endif",
            "keepjumps exe s:l",
            "normal! zt",
            "keepjumps 1",
            "normal! 0",
            "tabnext",
            "edit b.txt",
            "argglobal",
            "balt a.txt",
            "let s:l = 1 - ((0 * winheight(0) + 10) / 21)",
            "if s:l < 1 | let s:l = 1 | endif",
            "keepjumps exe s:l",
            "normal! zt",
            "keepjumps 1",
            "normal! 0",
            "tabnext 1",
            "set stal=1",
            "if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0 && getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'",
            "  silent exe 'bwipe ' . s:wipebuf",
            "endif",
            "unlet! s:wipebuf",
            "set winheight=1 winwidth=20",
            "let &shortmess = s:shortmess_save",
            "let s:sx = expand(\"<sfile>:p:r\").\"x.vim\"",
            "if filereadable(s:sx)",
            "  exe \"source \" . fnameescape(s:sx)",
            "endif",
            "let &g:so = s:so_save | let &g:siso = s:siso_save",
            "set hlsearch",
            "doautoall SessionLoadPost",
            "unlet SessionLoad",
            "\" vim: set ft=vim :",
        ]
    );
    restore(&fx);
}

#[test]
fn global_variables() {
    let sandbox = sources("ex_session_globals");
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    fx.run("set sessionoptions=globals");
    fx.run("let g:SessNum = 7 | let g:SessStr = \"it's\" | let g:sess_lower = 1");
    // In the order the variables' hash table walks, which depends on every
    // other global the shared editor holds: compared sorted.
    let mut got: Vec<String> = script(&fx, &sandbox, "mksession")
        .into_iter()
        .filter(|line| line.contains("Sess") || line.contains("sess_"))
        .collect();
    got.sort();
    assert_eq!(
        got,
        [
            "doautoall SessionLoadPost",
            "doautoall SessionLoadPre",
            "let SessNum =  7 ",
            "let SessStr = \"it's\"",
            "let SessionLoad = 1",
            "unlet SessionLoad",
        ]
    );
    restore(&fx);
}

#[test]
fn a_view_keeps_folds_and_the_cursor() {
    let sandbox = sources("ex_session_view");
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    fx.run("set viewoptions=folds,cursor");
    fx.run("badd a.txt | buffer a.txt");
    fx.run("setlocal foldmethod=manual");
    fx.run("3,8fold");
    fx.run("10,20fold");
    // Open it first: a range inside a closed fold extends to the whole fold,
    // so `12,14fold` would fold 10-20 a second time instead of nesting.
    fx.run("normal! zR");
    fx.run("12,14fold");
    fx.run("normal! zR");
    fx.run("12foldclose");
    fx.run("call cursor(25, 4)");
    assert_eq!(
        script(&fx, &sandbox, "mkview"),
        [
            "let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1",
            "argglobal",
            "if bufexists(fnamemodify(\"<DIR>/a.txt\", \":p\")) | buffer <DIR>/a.txt | else | edit <DIR>/a.txt | endif",
            "if &buftype ==# 'terminal'",
            "  silent file <DIR>/a.txt",
            "endif",
            "setlocal foldmethod=manual",
            "setlocal foldexpr=0",
            "setlocal foldmarker={{{,}}}",
            "setlocal foldignore=#",
            "setlocal foldlevel=2",
            "setlocal foldminlines=1",
            "setlocal foldnestmax=20",
            "setlocal foldenable",
            "silent! normal! zE",
            "sil! 3,8fold",
            "sil! 12,14fold",
            "sil! 10,20fold",
            "let &fdl = &fdl",
            "10",
            "sil! normal! zo",
            "12",
            "sil! normal! zc",
            "let s:l = 25 - ((24 * winheight(0) + 11) / 22)",
            "if s:l < 1 | let s:l = 1 | endif",
            "keepjumps exe s:l",
            "normal! zt",
            "keepjumps 25",
            "normal! 01|",
            "let &g:so = s:so_save | let &g:siso = s:siso_save",
            "set hlsearch",
            "doautoall SessionLoadPost",
            "\" vim: set ft=vim :",
        ]
    );
    fx.run("setlocal foldmethod&");
    restore(&fx);
}

#[test]
fn an_existing_file_is_kept_without_a_bang() {
    let sandbox = sources("ex_session_exists");
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    sandbox.write("there.vim", b"kept\n");
    fx.check_error(
        "mksession there.vim",
        "E189: \"there.vim\" exists (add ! to override)",
    );
    fx.check_error(
        "mkview there.vim",
        "E189: \"there.vim\" exists (add ! to override)",
    );
    assert_eq!(
        std::fs::read(sandbox.path("there.vim")).ok(),
        Some(b"kept\n".to_vec())
    );
    restore(&fx);
}

#[test]
fn float_globals_are_written_as_printf_writes_them() {
    let sandbox = sources("ex_session_floats");
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    fx.run("set sessionoptions=globals");
    fx.run("let g:FltNeg = -1.5 | let g:FltPos = 2.0e10 | let g:FltTiny = 1.0e-7");
    fx.run("let g:FltInf = 1.0 / 0 | let g:FltNinf = -1.0 / 0 | let g:FltNan = str2float('nan')");
    fx.run("let g:FltKey_é = 3");
    let mut got: Vec<String> = script(&fx, &sandbox, "mksession")
        .into_iter()
        .filter(|line| line.starts_with("let Flt"))
        .collect();
    got.sort();
    assert_eq!(
        got,
        [
            "let FltInf =  inf",
            "let FltNan =  nan",
            "let FltNeg = -1.500000",
            "let FltNinf = -inf",
            "let FltPos =  20000000000.000000",
            "let FltTiny =  0.000000",
        ]
    );
    fx.run("unlet g:FltNeg g:FltPos g:FltTiny g:FltInf g:FltNinf g:FltNan");
    fx.run("unlet! g:FltKey_é");
    restore(&fx);
}
