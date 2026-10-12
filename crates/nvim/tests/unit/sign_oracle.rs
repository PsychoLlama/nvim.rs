//! A sign's `text=` and `icon=` over `support::corpus`, every pair of its
//! tokens and every token behind a backslash, through the three ways in:
//! `sign_define()`'s dictionary, the `:sign define` command line (which
//! splits on unescaped white space first) and `nvim_buf_set_extmark`'s
//! `sign_text` (which neither unescapes nor reports). Each answer is the
//! call's result, the text and icon the definition then reports, the error
//! it raised and the `:sign list` line, folded into one number per path.
//! Recorded on the in-place unescape; the owned one that replaced it has to
//! reproduce every digest (see `mbyte_oracle.rs` for the rule).
//!
//! The texts are handed over through Vim script (`"\x.."` literals), so
//! bytes that are not UTF-8 arrive unchanged.

#![cfg(not(miri))]

use crate::support::corpus::{Digest, corpus, tokens};
use crate::support::{Probe, Sandbox};

/// The corpus less what a string cannot carry here (a NUL ends it, a
/// newline ends a command line), plus the pairs and the escapes.
fn texts() -> Vec<Vec<u8>> {
    let mut out: Vec<Vec<u8>> = corpus();
    for &a in tokens() {
        let mut escaped = b"\\".to_vec();
        escaped.extend_from_slice(a);
        out.push(escaped);
        for &b in tokens() {
            out.push([a, b].concat());
        }
    }
    out.retain(|text| !text.contains(&0) && !text.contains(&b'\n'));
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

/// What the definition `Or` reports now, and the error the last step raised.
fn defined(fx: &Probe) -> String {
    format!(
        "{}|{}|{}",
        fx.eval("sign_getdefined('Or')"),
        fx.eval("v:errmsg"),
        fx.output("silent! sign list Or")
    )
}

#[test]
fn sign_text_over_the_corpus() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fx.home();
    fx.run("call sign_unplace('*') | call sign_undefine()");
    fx.run("call setline(1, ['a'])");
    let n = load_texts(&fx);
    let mut digests = Vec::new();

    // `sign_define()`: text, then icon, each over a known definition.
    for key in ["text", "icon"] {
        let mut d = Digest::new();
        for i in 0..n {
            fx.run("sign define Or text=zz icon=zz");
            fx.run("let v:errmsg = ''");
            fx.run(&format!(
                "silent! let g:r = sign_define('Or', {{'{key}': g:texts[{i}]}})"
            ));
            d.bytes(fx.eval("g:r").as_bytes());
            d.bytes(defined(&fx).as_bytes());
        }
        digests.push(d.value());
    }

    // `:sign define`, the same two, plus a fresh definition each time.
    for key in ["text", "icon"] {
        let mut d = Digest::new();
        for i in 0..n {
            fx.run("silent! sign undefine Or");
            fx.run("let v:errmsg = ''");
            fx.run(&format!(
                "silent! execute 'sign define Or {key}=' . g:texts[{i}]"
            ));
            d.bytes(defined(&fx).as_bytes());
        }
        digests.push(d.value());
    }

    // `nvim_buf_set_extmark`'s `sign_text`.
    fx.run("let g:ns = nvim_create_namespace('sign_oracle')");
    let mut d = Digest::new();
    for i in 0..n {
        fx.run("call nvim_buf_clear_namespace(0, g:ns, 0, -1)");
        let r = fx.eval(&format!(
            "[execute('silent! call nvim_buf_set_extmark(0, g:ns, 0, 0, {{''sign_text'': g:texts[{i}]}})'), v:errmsg, map(nvim_buf_get_extmarks(0, g:ns, 0, -1, {{'details': 1}}), 'get(v:val[3], ''sign_text'', ''-'')')]"
        ));
        fx.run("let v:errmsg = ''");
        d.bytes(r.as_bytes());
    }
    digests.push(d.value());

    assert_eq!(digests, EXPECTED, "sign oracle digests moved");
    fx.run("call nvim_buf_clear_namespace(0, g:ns, 0, -1)");
    fx.run("call sign_undefine() | unlet! g:texts g:ns g:r");
    fx.home();
}

/// Recorded on the in-place unescape.
const EXPECTED: [u64; 5] = [
    8_876_131_771_981_097_116,
    362_617_716_755_529_551,
    3_558_112_372_096_246_788,
    18_326_618_313_094_905_190,
    14_386_627_065_686_551_340,
];
