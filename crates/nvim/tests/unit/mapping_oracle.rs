//! The mapping table and the menu tree over `support::corpus`: every text
//! defined as a mapping's lhs and rhs (round-robin over the modes, the
//! `:map-arguments` and the global/buffer-local tables), then everything
//! that reads the tables back -- `maparg()`/`mapcheck()` both shapes,
//! `maplist()`, `nvim_get_keymap`/`nvim_buf_get_keymap`, the `:map` listing
//! per mode, abbreviations, `:mkexrc`'s and `:mkview`'s map commands, a
//! `mapset()` round trip and `:unmap`/`:mapclear` -- and every text as a menu
//! item (`menu_get()`, the `:menu` listing, `menu_info()`, `:unmenu`). Each
//! family is folded into one number. Recorded on the intrusive pointer lists
//! the owned tables replaced; they have to reproduce every digest (see
//! `mbyte_oracle.rs` for the rule).
//!
//! The texts are handed over through Vim script (`"\x.."` literals), so
//! bytes that are not UTF-8 arrive unchanged. A dict's `sid` and `lnum` name
//! whatever script the suite sourced before, so they are dropped.

#![cfg(not(miri))]

use crate::support::corpus::{Digest, corpus};
use crate::support::{Probe, Sandbox};

/// The corpus less what a string cannot carry here (a NUL ends it, a
/// newline ends a command line), plus spellings the parsers treat specially.
fn texts() -> Vec<Vec<u8>> {
    let mut out: Vec<Vec<u8>> = corpus()
        .into_iter()
        .filter(|text| !text.contains(&0) && !text.contains(&b'\n'))
        .collect();
    for extra in [
        &b"<C-H>"[..],
        b"<Tab>",
        b"<C-I>",
        b"<M-x><F2>",
        b"<lt>x",
        b"<Nop>",
        b"<nop>",
        b"<Bar>|",
        b"<Space> ",
        b"\\<Esc>",
        b"\x16x",
        b"<SID>f",
        b"<Plug>(p)",
        b"<Cmd>echo<CR>",
        b"a b",
        b"#i",
        b"x\xc3\xa9",
        b"\xc3\xa9t\xc3\xa9",
        b"\x80\xfd\x04",
    ] {
        out.push(extra.to_vec());
    }
    out
}

/// `g:texts`, set to [`texts`].
fn load_texts(fx: &Probe) -> usize {
    fx.run("let g:texts = []");
    for text in texts() {
        let hex: String = text.iter().map(|b| format!("\\x{b:02x}")).collect();
        fx.run(&format!("call add(g:texts, \"{hex}\")"));
    }
    fx.eval("len(g:texts)").parse().expect("a count")
}

/// The Vim script half: definitions and the readers, in functions so one
/// call walks the whole corpus.
const HELPERS: &[&str] = &[
    r"function! MoStrip(d)",
    r"  return filter(copy(a:d), {k, _ -> k !~# '^\(sid\|lnum\)$'})",
    r"endfunction",
    r"function! MoDefine(texts)",
    r"  let res = []",
    r"  let modes = ['n', 'v', 'x', 's', 'o', 'i', 'c', 't', 'l', '!', '']",
    r"  let i = 0",
    r"  for t in a:texts",
    r"    let v:errmsg = ''",
    r"    let m = modes[i % len(modes)]",
    r"    let opts = {'silent': i % 2 == 1, 'nowait': i % 3 == 0, 'noremap': i % 5 == 0, 'expr': i % 7 == 0}",
    r"    if i % 11 == 4 | let opts.desc = t | endif",
    r"    try",
    r"      if i % 4 == 3",
    r"        call nvim_buf_set_keymap(0, m, t, 'R' . t, opts)",
    r"      else",
    r"        call nvim_set_keymap(m, t, 'R' . t, opts)",
    r"      endif",
    r"      call add(res, 'ok')",
    r"    catch",
    r"      call add(res, v:exception)",
    r"    endtry",
    r"    call add(res, MoStrip(maparg(t, m, 0, 1)))",
    r"    call add(res, [maparg(t, m), mapcheck(t, m), mapcheck(t[:1], m), hasmapto('R' . t, m), v:errmsg])",
    r"    let i += 1",
    r"  endfor",
    r"  return res",
    r"endfunction",
    r"function! MoAbbrevs(texts)",
    r"  let res = []",
    r"  let cmds = ['iabbrev', 'cnoreabbrev', 'abbreviate', 'inoreabbrev <buffer>', 'iabbrev <expr>']",
    r"  let i = 0",
    r"  for t in a:texts",
    r"    let v:errmsg = ''",
    r"    silent! execute cmds[i % len(cmds)] . ' ' . t . ' A' . t",
    r"    call add(res, [v:errmsg, maparg(t, 'i', 1), maparg(t, 'c', 1)])",
    r"    let i += 1",
    r"  endfor",
    r"  return res",
    r"endfunction",
    r"function! MoListing()",
    r"  let res = []",
    r"  for cmd in ['map', 'map!', 'nmap', 'vmap', 'xmap', 'smap', 'omap', 'imap', 'cmap', 'tmap', 'lmap', 'map <buffer>', 'imap <buffer>', 'abbreviate', 'iabbrev', 'cabbrev', 'abbreviate <buffer>']",
    r"    call add(res, execute(cmd))",
    r"  endfor",
    r"  return res",
    r"endfunction",
    r"function! MoTables()",
    r"  let res = [map(maplist(), 'MoStrip(v:val)'), map(maplist(1), 'MoStrip(v:val)')]",
    r"  for m in ['n', 'v', 'x', 's', 'o', 'i', 'c', 't', 'l', '!', '', 'ia', 'ca', '!a']",
    r"    call add(res, map(nvim_get_keymap(m), 'MoStrip(v:val)'))",
    r"    call add(res, map(nvim_buf_get_keymap(0, m), 'MoStrip(v:val)'))",
    r"  endfor",
    r"  return res",
    r"endfunction",
    r"function! MoUnmap(texts)",
    r"  let res = []",
    r"  let i = 0",
    r"  for t in a:texts",
    r"    let v:errmsg = ''",
    r"    for cmd in ['sunmap', 'iunmap', 'ounmap <buffer>', 'nunmap', 'cunmap', 'unmap!']",
    r"      if i % 3 == 0",
    r"        silent! execute cmd . ' ' . t",
    r"      endif",
    r"      let i += 1",
    r"    endfor",
    r"    call add(res, v:errmsg)",
    r"  endfor",
    r"  return res",
    r"endfunction",
    r"function! MoMapset()",
    r"  let before = maplist()",
    r"  for d in before",
    r"    call mapset(d)",
    r"  endfor",
    r"  let after = maplist()",
    r"  return [len(before), map(copy(after), 'MoStrip(v:val)'), sort(map(copy(before), 'string(MoStrip(v:val))')) == sort(map(copy(after), 'string(MoStrip(v:val))'))]",
    r"endfunction",
    r"function! MoMenus(texts)",
    r"  let res = []",
    r"  let cmds = ['amenu', 'nmenu', 'inoremenu', 'vmenu <silent>', 'tlmenu', 'menu <script>', 'cmenu', 'omenu']",
    r"  let i = 0",
    r"  for t in a:texts",
    r"    let v:errmsg = ''",
    r"    let name = escape(t, ' .\')",
    r"    silent! execute cmds[i % len(cmds)] . ' ' . (i % 9) . '.' . (i % 4 + 1) . ' Mo' . (i % 3) . '.' . name . ' :' . t . '<CR>'",
    r"    if i % 6 == 2",
    r"      silent! execute 'tmenu Mo' . (i % 3) . '.' . name . ' tip ' . t",
    r"    endif",
    r"    if i % 10 == 5",
    r"      silent! execute 'amenu Mo' . (i % 3) . '.-sep' . i . '- <Nop>'",
    r"    endif",
    r"    call add(res, [v:errmsg, menu_info('Mo' . (i % 3) . '.' . t), menu_info('Mo' . (i % 3) . '.' . t, 'i')])",
    r"    let i += 1",
    r"  endfor",
    r"  return res",
    r"endfunction",
    r"function! MoMenuReaders()",
    r"  let res = [menu_get(''), menu_get('', 'i'), menu_get('Mo1'), menu_info(''), menu_info('Mo2')]",
    r"  for cmd in ['menu', 'amenu Mo0', 'imenu', 'tlmenu', 'tmenu', 'menu Mo1']",
    r"    call add(res, execute(cmd))",
    r"  endfor",
    r"  return res",
    r"endfunction",
];

#[test]
fn mapping_tables_over_the_corpus() {
    let sandbox = Sandbox::dir("mapping_oracle");
    let fx = Probe::new(sandbox.editor());
    fx.home();
    // Every option the readers below depend on, at its default.
    fx.run("set all&");
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
        "aunmenu *",
        "tlunmenu *",
        "tunmenu *",
    ] {
        fx.run(&format!("silent! {cmd}"));
    }
    let lines: Vec<String> = HELPERS.iter().map(|line| (*line).to_owned()).collect();
    let quoted: Vec<String> = lines
        .iter()
        .map(|line| format!("'{}'", line.replace('\'', "''")))
        .collect();
    fx.run(&format!("call execute([{}])", quoted.join(", ")));
    load_texts(&fx);

    let mut digests = Vec::new();
    let mut fold = |expr: &str| {
        let mut d = Digest::new();
        d.bytes(fx.eval(expr).as_bytes());
        digests.push(d.value());
    };
    fold("MoDefine(g:texts)");
    fold("MoAbbrevs(g:texts)");
    fold("MoListing()");
    fold("MoTables()");
    let exrc = sandbox.path("exrc.vim");
    let view = sandbox.path("view.vim");
    let (exrc, view) = (exrc.to_str().unwrap(), view.to_str().unwrap());
    fx.run(&format!("silent! mkexrc! {exrc}"));
    fold(&format!("[v:errmsg, readfile('{exrc}', 'b')]"));
    fx.run("set viewoptions=localoptions");
    fx.run(&format!("silent! mkview! {view}"));
    fold(&format!(
        "filter(readfile('{view}', 'b'), 'v:val =~# ''map\\|abbr\\|cpo''')"
    ));
    fx.run("set viewoptions&");
    fold("MoUnmap(g:texts)");
    fold("[MoListing(), MoTables()]");
    fold("MoMapset()");
    // Read the exrc back in: the same tables again.
    fx.run("mapclear | mapclear! | tmapclear | lmapclear | abclear");
    fx.run(&format!("silent! source {exrc}"));
    fold("[v:errmsg, MoTables()]");
    fx.run("mapclear | mapclear! | tmapclear | lmapclear | abclear");
    fx.run("mapclear <buffer> | mapclear! <buffer> | tmapclear <buffer> | lmapclear <buffer> | abclear <buffer>");
    fold("[maplist(), maplist(1), execute('map'), execute('abbreviate')]");

    fold("MoMenus(g:texts)");
    fold("MoMenuReaders()");
    fx.run("silent! aunmenu Mo1");
    fx.run("silent! iunmenu Mo2");
    fx.run("silent! tunmenu *");
    fold("MoMenuReaders()");
    fx.run("silent! aunmenu *");
    fx.run("silent! tlunmenu *");
    fold("MoMenuReaders()");

    assert_eq!(digests, EXPECTED, "mapping oracle digests moved");
    for name in [
        "MoStrip",
        "MoDefine",
        "MoAbbrevs",
        "MoListing",
        "MoTables",
        "MoUnmap",
        "MoMapset",
        "MoMenus",
        "MoMenuReaders",
    ] {
        fx.run(&format!("delfunction {name}"));
    }
    fx.run("unlet! g:texts");
    fx.run("set all&");
    fx.home();
}

/// Recorded on the intrusive lists.
const EXPECTED: [u64; 15] = [
    18_325_338_693_757_688_101,
    10_546_707_823_912_418_311,
    14_512_619_515_441_789_438,
    8_011_306_603_908_761_186,
    11_407_086_432_930_885_848,
    1_307_663_555_832_762_864,
    14_621_063_569_095_319_848,
    9_336_110_215_120_305_768,
    8_641_896_876_056_060_866,
    8_944_349_779_487_404_315,
    7_797_769_361_626_727_342,
    8_101_208_710_798_376_915,
    10_752_954_196_177_704_387,
    11_028_288_259_730_776_395,
    18_080_044_942_998_795_006,
];
