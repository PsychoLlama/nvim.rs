//! What a command's file argument stands for -- `%`, `#`, `<cword>` and the
//! rest of `eval_vars`'s alphabet, each with a chain of `:p:h:t`-style
//! modifiers after it -- folded into one number per entry point over a
//! hostile set of buffer names, recorded from the pointer forms upstream
//! ported. The rewrite onto slices and owned answers has to reproduce every
//! digest (see `mbyte_oracle.rs` for the rule); only the script below may
//! change, and only to add rows that are recorded on the old code first.
//!
//! Three entry points read the same expansion: `expand()` (the leading item
//! only), a user command with `-complete=file` (the whole of
//! `expand_filename`: every item in the argument, the escaping, `$`/`~`, and
//! for `-nargs=1` the wildcard and backslash pass), and `fnamemodify()` (the
//! modifiers alone). The sandbox's path and the home directory are written
//! out of the answers before they are folded, so the digests do not depend
//! on where the test runs.

#![cfg(not(miri))]

use crate::support::corpus::Digest;
use crate::support::{Probe, Sandbox};

/// Buffer names a file argument has to survive: separators at both ends,
/// dots in every position, the characters `expand_filename` escapes or
/// reads as items, multibyte and combining text, and the home forms.
const NAMES: &[&str] = &[
    "a.txt",
    "sub/c.txt",
    "sub/",
    "/",
    "//x",
    "/tmp/../x.y.z",
    "./a.txt",
    "../up.d/x",
    "~",
    "~/x.vim",
    "~nosuchuser/x",
    "$HOME/e.c",
    ".hidden",
    "x.",
    ".",
    "a.b.c.d",
    "sp ace.txt",
    "a\\b.c",
    "per%cent.c",
    "hash#1.c",
    "bar|.c",
    "quote'.c",
    "dq\".c",
    "名前.txt",
    "e\u{301}.txt",
    "tab\t.c",
    "star*.c",
    "q?.c",
    "[br].c",
    "{a,b}.c",
    "!bang.c",
    "<cword>.c",
    "dir.d/noext",
    "x.tar.gz",
    "",
];

/// The modifier chains each item is followed by; the bad ones too.
const MODS: &[&str] = &[
    "",
    ":p",
    ":h",
    ":t",
    ":r",
    ":e",
    ":p:h",
    ":p:h:t",
    ":t:r",
    ":r:r",
    ":e:e",
    ":r:e",
    ":~",
    ":.",
    ":p:~",
    ":p:.",
    ":h:h",
    ":h:h:h",
    ":t:e:e",
    ":s?a?X?",
    ":gs?/?_?",
    ":s?\\.?!?",
    ":S",
    ":p:S",
    ":8",
    ":h:p",
    ":gs/x/yy/:t",
    ":p:h:h:t:r",
    "<",
    ":z",
    "::",
    ":",
    ":t:",
];

/// The items, with the forms of `#` that take a number.
const ITEMS: &[&str] = &["%", "#", "#1", "#<1", "#-", "#0", "##", "%%", "\\%", "x%y"];

/// The script that walks every row and leaves the answers in `g:out`.
const SCRIPT: &str = r#"
let g:out = []
function! s:Rec(cmd, arg) abort
  let g:got = '<unset>'
  let v:errmsg = ''
  try
    silent exe a:cmd a:arg
  catch
    let g:got = 'ERR:' . v:exception
  endtry
  call add(g:out, [a:cmd, a:arg, g:got, v:errmsg])
endfunction
function! g:FnoRun(names, mods, items) abort
  command! -nargs=* -complete=file FnoRec let g:got = <q-args>
  command! -nargs=1 -complete=file FnoOne let g:got = <q-args>
  command! -nargs=* FnoRaw let g:got = <q-args>
  for name in a:names
    if name ==# ''
      silent! 0file
    else
      call nvim_buf_set_name(0, name)
    endif
    for mod in a:mods
      call add(g:out, [name, mod, expand('%' . mod), expand('#' . mod), expand('%<' . mod)])
      call add(g:out, fnamemodify(name, mod))
      for item in a:items
        call s:Rec('FnoRec', item . mod)
        call s:Rec('FnoOne', item . mod)
      endfor
      call s:Rec('FnoRec', 'pre' . '%' . mod . ' %' . mod . '|x')
      call s:Rec('FnoRaw', '%' . mod)
    endfor
  endfor
  silent! 0file
  for arg in ['*.txt', 'a*', 'sub/*', 'sp\ ace.txt', 'a\b', '~/x', '$HOME/y', '$NOSUCHVAR/z',
        \ '[ab].txt', 'a.tx?', 'nomatch*', '`=1+1`', '`=1+1`x', 'a\ b\ c', '\\x', '%:h/*']
    call s:Rec('FnoOne', arg)
    call s:Rec('FnoRec', arg)
  endfor
  silent! argdelete *
  silent! argadd a.txt sp\ ace.txt sub/c.txt
  for mod in ['', ':t', ':p:t']
    call s:Rec('FnoRec', '##' . mod)
  endfor
  silent! argdelete *
  for [line, col] in [['foo/bar.c baz', 3], ['  x.y ', 3], ['a->b c', 1], ['s:var.x[1] t', 2],
        \ ['', 1], ['名前.txt end', 1], ['~/file.vim', 2], ['%:h', 1], ['$HOME/q.c', 2]]
    call setline(1, line)
    call cursor(1, col)
    for item in ['<cword>', '<cWORD>', '<cexpr>', '<cfile>']
      for mod in ['', ':t', ':e', ':p:h:t', ':r', '<']
        call add(g:out, [line, item . mod, expand(item . mod)])
        call s:Rec('FnoRec', item . mod)
      endfor
    endfor
  endfor
  for item in ['<sfile>', '<slnum>', '<stack>', '<script>', '<afile>', '<abuf>', '<amatch>',
        \ '<sflnum>', '<SID>', '<nosuch>']
    call add(g:out, [item, expand(item), expand(item . ':t')])
    call s:Rec('FnoRec', item)
  endfor
  delcommand FnoRec
  delcommand FnoOne
  delcommand FnoRaw
endfunction
"#;

#[test]
fn filename_expansion() {
    let sandbox = Sandbox::dir("filename_oracle");
    for file in [
        "a.txt",
        "b.txt",
        "sp ace.txt",
        "名前.txt",
        ".hidden",
        "sub/c.txt",
        "sub/d.vim",
    ] {
        if let Some(dir) = std::path::Path::new(file).parent() {
            sandbox.mkdir(&dir.to_string_lossy());
        }
        sandbox.touch(file);
    }
    let script = sandbox.write("oracle.vim", SCRIPT.as_bytes());
    let probe = Probe::new(sandbox.editor());
    probe.home();
    // No swap file: renaming the buffer would otherwise look for one, and
    // an earlier case in the lane may have left one behind.
    probe.run("setlocal noswapfile");
    // The alternate file, which every `#` reads.
    probe.run("silent! balt alt.d/alt.c");
    probe.run(&format!("source {}", script.display()));
    let quote = |items: &[&str]| {
        let quoted: Vec<String> = items
            .iter()
            .map(|item| format!("'{}'", item.replace('\'', "''")))
            .collect();
        format!("[{}]", quoted.join(", "))
    };
    probe.run(&format!(
        "call g:FnoRun({}, {}, {})",
        quote(NAMES),
        quote(MODS),
        quote(ITEMS)
    ));
    let out = probe.eval("g:out");
    probe.run("delfunction g:FnoRun");
    probe.run("unlet g:out g:got");
    probe.run("setlocal swapfile<");
    probe.home();

    let home = std::env::var("HOME").unwrap_or_default();
    let mut text = out.replace(sandbox.as_str(), "<D>");
    if !home.is_empty() {
        text = text.replace(&home, "<H>");
    }
    let text = script_ids_out(&text);
    // `FILENAME_ORACLE_DUMP=path` writes the answers out, for a diff
    // against the old code's when a digest moves.
    if let Ok(path) = std::env::var("FILENAME_ORACLE_DUMP") {
        std::fs::write(path, text.replace("], [", "],\n[")).expect("a dump file");
    }
    assert!(
        text.len() > 10_000,
        "the walk ran: {}",
        &text[..text.len().min(400)]
    );
    let mut d = Digest::new();
    d.bytes(text.as_bytes());
    assert_eq!(
        d.value(),
        0x9dea_7fd7_ae57_d562,
        "filename expansion:\n{}",
        &text[..text.len().min(2000)]
    );
}

/// `<SNR>12_` as `<SNR>N_`: a script's number is the order it was first
/// sourced in, which is the order the lane's cases happened to run.
fn script_ids_out(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("<SNR>") {
        out.push_str(&rest[..at + 5]);
        rest = &rest[at + 5..];
        let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
        if digits > 0 {
            out.push('N');
            rest = &rest[digits..];
        }
    }
    out.push_str(rest);
    out
}
