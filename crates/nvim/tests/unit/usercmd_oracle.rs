//! A user command's run: every `<...>` code of a replacement expanded over
//! `support::corpus` as the arguments, folded into one number per family,
//! recorded on the measure-then-write pointer builder that upstream ported.
//! The owned builder that replaced it has to reproduce every digest (see
//! `mbyte_oracle.rs` for the rule).
//!
//! What a run expands to is observed through what it then does: each
//! replacement adds its expansion to `g:e` -- `<q-args>` and `<f-args>` as
//! the literals they are built to be, `<args>` inside a `'...'` literal that
//! the hostile half of the corpus breaks -- and the error a broken line
//! raises quotes the expanded text back, so a byte that moves moves the
//! digest either way. The texts are handed over through Vim script
//! (`"\x.."` literals and `:execute`), so bytes that are not UTF-8 arrive
//! unchanged.

#![cfg(not(miri))]

use crate::support::corpus::{Digest, corpus};
use crate::support::{Probe, Sandbox};

/// The texts: the corpus, less what a command line cannot carry (a NUL ends
/// it, a newline starts the next one), plus the escapes and separators the
/// splitter and the quoters treat specially.
fn texts() -> Vec<Vec<u8>> {
    let mut out: Vec<Vec<u8>> = corpus()
        .into_iter()
        .filter(|text| !text.contains(&0) && !text.contains(&b'\n'))
        .collect();
    for extra in [
        &br"a\ b"[..],
        br"a\\ b",
        br"a\\\ b",
        br"\ lead",
        br"trail\ ",
        br"trail\",
        b"  two  spaces  ",
        b"\ttab\tsep\t",
        br#""dq" 'sq'"#,
        br#"\"\'"#,
        b"| bar | here",
        b"<args> <lt> <q-args>",
        b"<f-args>",
        b"%",
        b"#",
        b"\xe2\x80\x83em\xe3\x80\x80ideographic",
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

/// The families: a definition, and the command the arguments follow.
const FAMILIES: &[(&str, &str)] = &[
    ("command -nargs=* UoRaw call add(g:e, '<args>')", "UoRaw"),
    ("command -nargs=* UoQ call add(g:e, <q-args>)", "UoQ"),
    ("command -nargs=* UoF call add(g:e, [<f-args>])", "UoF"),
    (
        "command -nargs=1 UoOne call add(g:e, [<f-args>, <q-args>])",
        "UoOne",
    ),
    (
        "command -nargs=* -bar UoBar call add(g:e, [<f-args>])",
        "UoBar",
    ),
    (
        "command -nargs=* -bang -register -range UoMix call add(g:e, [<q-bang>, '<bang>', <q-reg>, <line1>, <line2>, <range>, <count>, <q-mods>, '<lt>', <q-args>, '<nosuch>', '<', '<q-nosuch>', '<ARGS>' ==# <q-args>, [<F-args>]])",
        "2,4UoMix!",
    ),
];

/// What one run left behind: the list it built and the error it raised.
fn outcome(fx: &Probe, cmd: &str) -> String {
    fx.run("let g:e = [] | let v:errmsg = ''");
    fx.run(&format!("silent! {cmd}"));
    format!("{}|{}", fx.eval("g:e"), fx.eval("v:errmsg"))
}

/// The modifier prefixes `<mods>` renders, alone and stacked.
const MODS: &[&str] = &[
    "",
    "silent",
    "silent!",
    "unsilent",
    "verbose",
    "2verbose",
    "10verbose",
    "browse confirm hide keepalt keepjumps keepmarks keeppatterns lockmarks noswapfile",
    "noautocmd sandbox",
    "aboveleft",
    "belowright",
    "botright",
    "topleft",
    "vertical",
    "horizontal",
    "leftabove rightbelow",
    "tab",
    "0tab",
    "1tab",
    "2tab",
    "-tab",
    "+tab",
    "$tab",
    "silent! vertical botright 3verbose tab keepjumps",
];

#[test]
fn every_code_over_the_corpus() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fx.home();
    fx.run("silent! comclear");
    fx.run("call setline(1, ['a', 'b', 'c', 'd', 'e']) | 3");
    let n = load_texts(&fx);
    let mut digests = Vec::new();

    // The arguments.
    for (define, invoke) in FAMILIES {
        fx.run(define);
        let mut d = Digest::new();
        for i in 0..n {
            let run = format!("execute '{invoke} ' . g:texts[{i}]");
            d.bytes(outcome(&fx, &run).as_bytes());
        }
        digests.push(d.value());
    }

    // The replacement: each text around `<args>`, `<q-args>` and
    // `<f-args>`, and the listing of what was stored.
    let mut d = Digest::new();
    for i in 0..n {
        fx.run(&format!(
            "execute 'command! -nargs=* UoRep call add(g:e, [''' . g:texts[{i}] . '<args>'', <q-args> . ''' . g:texts[{i}] . ''', [<f-args>]])'"
        ));
        d.bytes(outcome(&fx, r"UoRep x\ y z").as_bytes());
        d.bytes(fx.output("command UoRep").as_bytes());
    }
    digests.push(d.value());

    // `<mods>` and `<q-mods>` under every modifier, in a body that is
    // nothing but codes as well.
    fx.run("tabnew | tabnew | tabnext 2");
    fx.run("command! -nargs=* UoMods call add(g:e, [<q-mods>, '<mods>', '<mods><mods>'])");
    let mut d = Digest::new();
    for mods in MODS {
        d.bytes(outcome(&fx, &format!("{mods} UoMods")).as_bytes());
    }
    digests.push(d.value());
    fx.run("silent! tabonly!");

    assert_eq!(digests, EXPECTED, "usercmd oracle digests moved");
    fx.run("silent! comclear | unlet! g:e g:texts");
    fx.home();
}

/// Recorded on the pointer builder.
const EXPECTED: [u64; 8] = [
    8_790_351_474_074_701_146,
    3_066_553_259_727_392_273,
    12_123_984_397_657_548_744,
    7_106_667_403_099_579_519,
    11_252_960_206_467_678_052,
    12_427_340_075_347_388_001,
    5_137_995_129_230_845_955,
    18_277_816_961_303_949_039,
];
