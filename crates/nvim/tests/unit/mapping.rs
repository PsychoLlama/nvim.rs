//! Mappings through the whole editor: what the lib tests in
//! `mapping/tests.rs` cannot reach without a window layout -- `:mkexrc`'s
//! output and reading it back, and `'langmap'`'s parse.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};

/// Every mapping and abbreviation gone, global and buffer-local.
fn clean(fx: &Probe) {
    for cmd in [
        "mapclear",
        "mapclear!",
        "tmapclear",
        "lmapclear",
        "abclear",
        "mapclear <buffer>",
        "mapclear! <buffer>",
        "tmapclear <buffer>",
        "lmapclear <buffer>",
        "abclear <buffer>",
    ] {
        fx.run(&format!("silent! {cmd}"));
    }
}

#[test]
fn mkexrc_writes_maps_that_read_back() {
    let sandbox = Sandbox::dir("mapping_mkexrc");
    let fx = Probe::new(sandbox.editor());
    fx.home();
    clean(&fx);
    let file = sandbox.path("exrc.vim");
    let file = file.to_str().expect("a UTF-8 path");
    for cmd in [
        "nnoremap <silent> <nowait> ,a :echo 'a'<CR>",
        "map ,b <Tab>|<lt>x",
        "sunmap ,b",
        "imap <expr> ,c 'c'",
        "noremap! ,d \\ d",
        "cnoremap <script> ,e e",
        "tmap ,f <C-\\><C-N>",
        "lmap ,g g",
        "nmap ,h <Nop>",
        "nmap ,<Space> sp",
        "nmap <M-x> \u{e9}",
        "iabbrev teh the",
        "cnoreabbrev W w",
    ] {
        fx.run(cmd);
    }
    fx.run(&format!("mkexrc! {file}"));
    fx.run(&format!("let g:mo_exrc = '{file}'"));
    // The map lines and the 'cpo' guard: the options written after them
    // are whatever the suite around this test left set.
    fx.expect_all(&[(r"filter(readfile(g:mo_exrc), 'v:val =~# ''map\|abbr\|cpo''')", "['nmap ,\u{16}  sp', 'let s:cpo_save=&cpo', 'set cpo&vim', 'nmap ,h <Nop>', 'tmap ,f \u{16}\u{1c}\u{16}\u{e}', 'nmap ,b \u{16}\t', 'xmap ,b \u{16}\t', 'omap ,b \u{16}\t', 'nnoremap <nowait> <silent> ,a :echo ''a''\u{16}', 'nmap <M-x> é', 'lmap ,g g', 'noremap! ,d \\ d', 'imap <expr> ,c ''c''', 'cnoreabbr W w', 'iabbr teh the', 'let &cpo=s:cpo_save', 'unlet s:cpo_save']")]);
    clean(&fx);
    fx.run(&format!("source {file}"));
    // What reads back: `<CR>` was written as CTRL-V CTRL-M, whose CR the
    // line reader takes for a DOS line ending.
    fx.expect_all(&[("map(maplist(), '[v:val.mode, v:val.lhs, v:val.rhs]')", "[['n', ',a', ':echo ''a''\u{16}\r'], ['o', ',b', '\u{16}\t'], ['x', ',b', '\u{16}\t'], ['n', ',b', '\u{16}\t'], ['t', ',f', '\u{16}\u{1c}\u{16}\u{e}'], ['n', ',h', '<Nop>'], ['n', ',<Space>', 'sp'], ['n', '<M-x>', 'é'], ['i', ',c', '''c'''], ['!', ',d', '\\ d'], ['l', ',g', 'g']]")]);
    fx.expect_all(&[(
        "map(maplist(1), '[v:val.mode, v:val.lhs, v:val.rhs]')",
        "[['i', 'teh', 'the'], ['c', 'W', 'w']]",
    )]);
    clean(&fx);
    fx.run("unlet g:mo_exrc");
}

#[test]
fn langmap_parses_pairs_halves_and_escapes() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fx.home();
    clean(&fx);
    // A key that `'langmap'` turned into `to` lands in `g:hit` through a
    // mapping of `to`; one it left alone lands as itself.
    fx.run("let g:hit = ''");
    fx.run("call execute(['function! MoHit(c) abort', 'let g:hit .= a:c', 'return \"\"', 'endfunction'])");
    for to in [
        "A", "B", "C", "d", "X", "Y", "Z", "a", ";", ",", "q", "e", "b", "\u{e9}",
    ] {
        fx.run(&format!("nnoremap <expr> {to} MoHit('{to}')"));
    }
    let typed = |keys: &str, want: &str| {
        fx.run("let g:hit = ''");
        fx.run(&format!("call feedkeys(\"{keys}\", 'xt')"));
        fx.check("g:hit", want);
    };
    fx.run("set langmap=abc;ABC");
    typed("abcd", "'ABCd'");
    fx.run("set langmap=xXyY,zZ");
    typed("xyza", "'XYZa'");
    fx.run(r"set langmap=\\,\\;,q\\,");
    typed(",q;", "';,;'");
    // A multi-byte `from` reaches the Normal-mode command, not the mapping
    // match, which looks at the first byte only.
    fx.run("set langmap=\u{e9}e,\u{3b1}a,\u{3b2}b");
    typed("\u{e9}\u{3b1}\u{3b2}", "'\u{e9}'");
    fx.run("set langmap=\u{3b1}l,\u{e9}\u{3b2};lh");
    fx.run("call setline(1, 'abcdef') | call cursor(1, 1)");
    fx.run("call feedkeys(\"\u{3b1}\u{3b1}\u{e9}\u{3b1}\u{3b2}\", 'xt')");
    fx.check("col('.')", "3");
    for (cmd, err) in [
        (
            "set langmap=abc;AB",
            "E357: ''langmap'': Matching character missing for c: langmap=abc;AB",
        ),
        (
            "set langmap=ab;AB;",
            "E358: ''langmap'': Extra characters after semicolon: ;: langmap=ab;AB;",
        ),
        (
            "set langmap=a",
            "E357: ''langmap'': Matching character missing for a: langmap=a",
        ),
        (
            "set langmap=ab;ABx",
            "E358: ''langmap'': Extra characters after semicolon: x: langmap=ab;ABx",
        ),
        ("set langmap=aA", "none"),
    ] {
        fx.check_error(cmd, err);
    }
    fx.run("set langmap=");
    typed("a,", "'a,'");
    clean(&fx);
    fx.run("delfunction MoHit");
    fx.run("unlet g:hit");
    fx.home();
}
