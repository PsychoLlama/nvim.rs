//! The key buffers over `support::corpus`: every text typed into Insert
//! mode and repeated with `.` (the redo buffer), recorded into a register
//! with `q` and replayed (the record buffer), inserted from a register
//! with `CTRL-R` and `CTRL-R CTRL-R` (the stuff buffer, plain and literal;
//! padded, because `setreg()` of text ending in a lone lead byte took the
//! bytes after it, upstream as here until the fix the case at the end pins --
//! `insert-register-lone-lead-byte-garbage.md` -- and the digests were
//! recorded padded),
//! pasted with `nvim_paste()` and repeated (with six `z`s after it: the
//! redo walk takes a lead byte's count of bytes whatever they are, so a
//! paste ending in an incomplete UTF-8 sequence eats the end-of-paste key and
//! `.` hangs, upstream as here -- `upstream-neovim-bugs`,
//! `paste-repeat-incomplete-utf8-hangs.md`), fed with `feedkeys()` under
//! each flag and read back with `getchar()`/`getcharstr()` (the typeahead),
//! across `inputsave()`/`inputrestore()`, and run through `:normal` nested
//! in `:normal` with typeahead left over. Each family is folded into one
//! number. Recorded on the pointer-chained key buffers and the
//! `xmalloc`ed typeahead the owned vectors replaced; they have to
//! reproduce every digest (see `mbyte_oracle.rs` for the rule).
//!
//! The texts are handed over through Vim script (`"\x.."` literals), so
//! bytes that are not UTF-8 arrive unchanged -- `K_SPECIAL` among them,
//! which the typeahead reads as the start of a key code, exactly as it
//! would from a mapping.

#![cfg(not(miri))]

use crate::support::corpus::{Digest, corpus};
use crate::support::{Probe, Sandbox};

/// The corpus less what a string cannot carry here (a NUL ends it, a
/// newline ends a command line), plus key spellings the buffers escape.
///
/// An overlong NUL (`C0 80` and its longer forms) typed into Insert mode
/// hangs the editor, upstream as here (`upstream-neovim-bugs`,
/// `insert-overlong-nul-hangs.md`): those spellings are replaced by `?`.
fn texts() -> Vec<Vec<u8>> {
    let mut out: Vec<Vec<u8>> = corpus()
        .into_iter()
        .filter(|text| !text.contains(&0) && !text.contains(&b'\n'))
        .map(|text| without_overlong_nul(&text))
        .collect();
    for extra in [
        &b"0"[..],
        b"^",
        b"x0",
        b"x^",
        b"\x1b",
        b"a\x1bb",
        b"\x16\x01",
        b"\x16\xc3\xa9",
        b"\x0f",
        b"\x7f",
        b"\x80\xfe\x58\x80\xfe\x58",
        b"\xc2\x80",
        b"\xc2\x80\xc2\x80\xc2\x80",
        b"\xef\xbf\xbf",
        b"\x80\x6b\x6c\x80\x6b\x72",
        b"\x80\xfc\x04a",
        b"\x80\xfc\x02\x80\x6b\x6c",
    ] {
        out.push(extra.to_vec());
    }
    out
}

/// `text` with every overlong NUL replaced by `?`.
fn without_overlong_nul(text: &[u8]) -> Vec<u8> {
    let forms: [&[u8]; 3] = [b"\xf0\x80\x80\x80", b"\xe0\x80\x80", b"\xc0\x80"];
    let mut out = Vec::with_capacity(text.len());
    let mut at = 0;
    'scan: while at < text.len() {
        for form in forms {
            if text[at..].starts_with(form) {
                out.push(b'?');
                at += form.len();
                continue 'scan;
            }
        }
        out.push(text[at]);
        at += 1;
    }
    out
}

/// `g:texts`, set to [`texts`], and `g:ntexts`, the same with every
/// `K_SPECIAL` byte escaped, for `:normal`: a raw one in the last two bytes
/// of its argument is an incomplete key code that `:normal` waits on
/// forever, upstream as here (`upstream-neovim-bugs`,
/// `normal-trailing-k-special-hangs.md`).
fn load_texts(fx: &Probe) -> usize {
    fx.run("let g:texts = []");
    fx.run("let g:ntexts = []");
    let hex = |text: &[u8]| -> String { text.iter().map(|b| format!("\\x{b:02x}")).collect() };
    for text in texts() {
        fx.run(&format!("call add(g:texts, \"{}\")", hex(&text)));
        let escaped: Vec<u8> = text
            .iter()
            .flat_map(|&b| {
                if b == 0x80 {
                    vec![0x80, 0xfe, 0x58]
                } else {
                    vec![b]
                }
            })
            .collect();
        fx.run(&format!("call add(g:ntexts, \"{}\")", hex(&escaped)));
    }
    fx.eval("len(g:texts)").parse().expect("a count")
}

/// The Vim script half, in functions so one call walks the whole corpus.
const HELPERS: &[&str] = &[
    r"function! GoReset()",
    r"  silent! %delete _",
    r"  call setline(1, 'base')",
    r"  normal! gg0",
    r"endfunction",
    r"function! GoLines()",
    r"  return getline(1, '$')",
    r"endfunction",
    r"function! GoDrain()",
    r"  let got = []",
    r"  while 1",
    r"    let c = getchar(0)",
    r"    if c is 0",
    r"      break",
    r"    endif",
    r"    call add(got, [c, getcharmod()])",
    r"  endwhile",
    r"  return got",
    r"endfunction",
    r"function! GoInsert(texts)",
    r"  let res = []",
    r"  for t in a:texts",
    r"    call GoReset()",
    r"    let v:errmsg = ''",
    r#"    silent! exe "normal! i" . t . "\<Esc>""#,
    r"    call add(res, [GoLines(), getreg('.'), v:errmsg])",
    r"    silent! normal! .",
    r"    call add(res, GoLines())",
    r"    silent! normal! 3.",
    r"    call add(res, [GoLines(), getreg('.')])",
    r"    call GoReset()",
    r#"    silent! exe "normal! 2A" . t . "\<Esc>j.""#,
    r"    call add(res, [GoLines(), getreg('.')])",
    r"    call GoReset()",
    r#"    silent! exe "normal! cw" . t . "\<Esc>""#,
    r"    silent! normal! .",
    r"    call add(res, [GoLines(), getreg('.')])",
    r"    call GoReset()",
    r#"    silent! exe "normal! o" . t . "\<C-O>." . t . "\<Esc>""#,
    r"    call add(res, GoLines())",
    r"  endfor",
    r"  return res",
    r"endfunction",
    r"function! GoRegisters(texts)",
    r"  let res = []",
    r"  for t in a:texts",
    r"    call GoReset()",
    r"    call setreg('a', t . 'zzzzzz')",
    r#"    silent! exe "normal! i\<C-R>a\<Esc>""#,
    r"    silent! normal! .",
    r"    call add(res, [GoLines(), getreg('.')])",
    r"    call GoReset()",
    r#"    silent! exe "normal! a\<C-R>\<C-R>a\<Esc>""#,
    r"    silent! normal! .",
    r"    call add(res, [GoLines(), getreg('.')])",
    r"    call GoReset()",
    r#"    silent! exe "normal! A\<C-R>\<C-O>a\<Esc>""#,
    r"    call add(res, GoLines())",
    r"    call GoReset()",
    r#"    silent! exe "normal! \"ap""#,
    r"    silent! normal! .",
    r"    call add(res, GoLines())",
    r"  endfor",
    r"  return res",
    r"endfunction",
    r"function! GoRecord(texts)",
    r"  let res = []",
    r"  for t in a:texts",
    r"    call GoReset()",
    r"    let v:errmsg = ''",
    r#"    silent! call feedkeys('qqi' . t . "\<Esc>q", 'xt')"#,
    r"    call add(res, [getreg('q'), GoLines(), v:errmsg])",
    r"    silent! normal! @q",
    r"    call add(res, GoLines())",
    r#"    silent! call feedkeys('qwA' . t . "\<C-O>q", 'xt')"#,
    r#"    silent! call feedkeys("\<Esc>", 'xt')"#,
    r"    call add(res, [getreg('w'), GoLines()])",
    r#"    silent! call feedkeys('qeq' . "qE" . t . "\<Esc>q", 'xt')"#,
    r"    call add(res, getreg('e'))",
    r"  endfor",
    r"  return res",
    r"endfunction",
    r"function! GoCmdline(texts)",
    r"  let res = []",
    r"  for t in a:texts",
    r"    call GoReset()",
    r"    let s = substitute(t, '\r', 'r', 'g')",
    r"    let g:got = ''",
    r#"    silent! call feedkeys(':let g:got = ' . string(s) . "\<CR>", 'xt')"#,
    r"    let first = g:got",
    r"    let g:got = ''",
    r"    silent! normal! @:",
    r"    call add(res, [first, g:got, @:])",
    r"  endfor",
    r"  return res",
    r"endfunction",
    r"function! GoFeed(texts)",
    r"  let res = []",
    r"  for t in a:texts",
    r"    for flags in ['n', 'm', 't', 'nt', 'L', 'n!']",
    r"      call feedkeys(t, flags)",
    r"      call add(res, [flags, getchar(1), GoDrain()])",
    r"    endfor",
    r"    call feedkeys('ab', 'n')",
    r"    call feedkeys(t, 'ni')",
    r"    call add(res, GoDrain())",
    r"    call feedkeys(t, 'n')",
    r"    let strs = []",
    r"    while 1",
    r"      let s = getcharstr(0)",
    r"      if s is ''",
    r"        break",
    r"      endif",
    r"      call add(strs, [s, getcharmod()])",
    r"    endwhile",
    r"    call add(res, strs)",
    r"    call feedkeys(t, 'n')",
    r"    call inputsave()",
    r"    call feedkeys('zz', 'n')",
    r"    let inner = GoDrain()",
    r"    call inputrestore()",
    r"    call add(res, [inner, GoDrain()])",
    r"    call feedkeys(t, 'n')",
    r"    call add(res, [getchar(0, {'simplify': v:false}), getchar(0, {'number': v:false}), GoDrain()])",
    r"  endfor",
    r"  return res",
    r"endfunction",
    r"function! GoNested(texts)",
    r"  let res = []",
    r"  for t in a:texts",
    r"    call GoReset()",
    r#"    silent! exe "normal! i" . t . "\<Esc>:normal! A<\<CR>" . "a>" . t"#,
    r"    call add(res, [GoLines(), getreg('.')])",
    r"    call GoReset()",
    r"    call feedkeys(t, 'n')",
    r#"    silent! exe "normal! :normal! I[\<CR>"#,
    r"    call add(res, [GoLines(), GoDrain()])",
    r"  endfor",
    r"  return res",
    r"endfunction",
    r"function! GoPaste(texts)",
    r"  let res = []",
    r"  for u in a:texts",
    r"    let t = u . 'zzzzzz'",
    r"    call GoReset()",
    r"    silent! call nvim_paste(t, v:true, -1)",
    r"    call add(res, GoLines())",
    r"    silent! normal! .",
    r"    call add(res, GoLines())",
    r"    call GoReset()",
    r"    silent! call nvim_paste(t, v:false, 1)",
    r"    silent! call nvim_paste(t . 'q', v:false, 2)",
    r"    silent! call nvim_paste('', v:false, 3)",
    r"    silent! normal! 2.",
    r"    call add(res, GoLines())",
    r"  endfor",
    r"  return res",
    r"endfunction",
];

#[test]
fn key_buffers_over_the_corpus() {
    let sandbox = Sandbox::dir("getchar_oracle");
    let fx = Probe::new(sandbox.editor());
    fx.home();
    // Every option the keys below depend on, at its default.
    fx.run("set all&");
    let lines: Vec<String> = HELPERS.iter().map(|line| (*line).to_owned()).collect();
    let quoted: Vec<String> = lines
        .iter()
        .map(|line| format!("'{}'", line.replace('\'', "''")))
        .collect();
    fx.run(&format!("call execute([{}])", quoted.join(", ")));
    load_texts(&fx);

    let mut digests = Vec::new();
    let mut fold = |expr: &str| {
        if std::env::var_os("GETCHAR_ORACLE_TRACE").is_some() {
            eprintln!("fold {expr}");
        }
        let mut d = Digest::new();
        let got = fx.eval(expr);
        if let Some(dir) = std::env::var_os("GETCHAR_ORACLE_DUMP") {
            let name = expr.split('(').next().unwrap_or("fold");
            std::fs::write(std::path::Path::new(&dir).join(name), got.as_bytes())
                .expect("a dump directory");
        }
        d.bytes(got.as_bytes());
        digests.push(d.value());
    };
    fold("GoInsert(g:ntexts)");
    fold("GoRegisters(g:texts)");
    fold("GoRecord(g:texts)");
    fold("GoCmdline(g:texts)");
    fold("GoFeed(g:texts)");
    fold("GoNested(g:ntexts)");
    fold("GoPaste(g:texts)");

    assert_eq!(digests, EXPECTED, "getchar oracle digests moved");
    for name in [
        "GoReset",
        "GoLines",
        "GoDrain",
        "GoInsert",
        "GoRegisters",
        "GoRecord",
        "GoCmdline",
        "GoFeed",
        "GoNested",
        "GoPaste",
    ] {
        fx.run(&format!("delfunction {name}"));
    }
    fx.run("unlet! g:texts g:ntexts g:got");
    for reg in ["a", "q", "w", "e"] {
        fx.run(&format!("call setreg('{reg}', [])"));
    }
    fx.run("set all&");
    fx.home();
}

/// A register set to text that ends inside a UTF-8 sequence holds that text
/// and nothing after it, so `CTRL-R` inserts exactly it.
///
/// `setreg()` split its string into lines stepping by the length each lead
/// byte *promised*, so the last step went past the string and the line took
/// the bytes behind it: a NUL and whatever followed, a different byte per run
/// (upstream as well -- `insert-register-lone-lead-byte-garbage.md`). The
/// corpus run above pads its registers for that reason; its digests were
/// recorded that way and stay so.
#[test]
fn a_register_ending_inside_a_sequence_holds_only_its_text() {
    let sandbox = Sandbox::dir("getchar_lone_lead");
    let fx = Probe::new(sandbox.editor());
    fx.home();
    // The text as a Vim string literal, the lines it should become, and
    // what `CTRL-R` inserts: each byte of a broken sequence is the
    // character of that number (stuffed and read back as UTF-8).
    for (text, lines, inserted) in [
        (r#""\xe2""#, r#"["\xe2"]"#, r#"["\u00e2"]"#),
        (r#""\xc3""#, r#"["\xc3"]"#, r#"["\u00c3"]"#),
        (r#""x\xe9""#, r#"["x\xe9"]"#, r#"["x\u00e9"]"#),
        (r#""\xe2\x82""#, r#"["\xe2\x82"]"#, r#"["\u00e2\u0082"]"#),
        (
            r#""\xf0\x9f\x98""#,
            r#"["\xf0\x9f\x98"]"#,
            r#"["\u00f0\u009f\u0098"]"#,
        ),
        // After a line break the rest goes in as the bytes it is.
        (r#""a\nb\xf0""#, r#"["a", "b\xf0"]"#, r#"["a", "b\xf0"]"#),
    ] {
        fx.run(&format!("call setreg('a', {text})"));
        fx.check(&format!("getreg('a', 1, 1) ==# {lines}"), "1");
        fx.run("silent! %delete _");
        fx.run(r#"exe "normal! i\<C-R>a\<Esc>""#);
        fx.check(&format!("getline(1, '$') ==# {inserted}"), "1");
    }
    fx.run("silent! %delete _");
    fx.run("call setreg('a', [])");
    fx.home();
}

/// Recorded on the pointer-chained key buffers.
const EXPECTED: [u64; 7] = [
    5_345_496_171_149_790_140,
    6_939_424_516_959_343_494,
    9_860_069_993_658_036_957,
    2_935_787_410_252_239_775,
    7_754_345_155_519_786_951,
    14_930_857_033_007_269_948,
    6_315_384_762_856_546_330,
];
