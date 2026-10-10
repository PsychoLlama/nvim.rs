//! The pointer-form `<>` notation readers and the `K_SPECIAL` escapers,
//! folded into one number each over a corpus of key names and
//! `support::corpus`, recorded from the forms upstream ported. A rewrite onto
//! slices has to reproduce every digest; see `mbyte_oracle.rs` for the rule.

#![cfg(not(miri))]

use std::ffi::{CStr, c_int};

use neovim::keycodes::{
    FSK_IN_STRING, FSK_KEEP_X_KEY, FSK_KEYCODE, FSK_SIMPLIFY, ModMask, REPTERM_DO_LT,
    REPTERM_FROM_PART, REPTERM_NO_SIMPLIFY, REPTERM_NO_SPECIAL, escape_ks_bytes, push_char_escaped,
    push_special_key, replace_termcodes, special_key_at, trans_special_into, unescape_ks_in,
};

use crate::support::corpus::{Digest, corpus};
use crate::support::editor_lock;

/// Key names in every shape the parser distinguishes, then the corpus, then
/// each name glued to a corpus string.
fn texts() -> Vec<Vec<u8>> {
    let names: &[&[u8]] = &[
        b"<C-a>",
        b"<C-S-Up>",
        b"<lt>",
        b"<lt",
        b"<SID>f",
        b"<sid>",
        b"<Leader>x",
        b"<LocalLeader>",
        b"<leader",
        b"<Char-65>",
        b"<Char-0x41>",
        b"<char-033>",
        b"<Char->",
        b"<Char-x>",
        b"<C-Char-65>",
        b"<M-\">",
        b"<C-\">",
        b"<M-\\\">",
        b"<t_k1>",
        b"<t_>",
        b"<*C-x>",
        b"<xUp>",
        b"<S-Tab>",
        b"<C-@>",
        b"<BS>",
        b"<Del>",
        b"<kDel>",
        b"<C-H>",
        b"<C-h>",
        b"<C-?>",
        b"<C-\xc3\xa9>",
        b"<M-\xc3\xa9>",
        b"<C-\xcc\x80>",
        b"\x16x",
        b"\x16",
        b"\\<CR>",
        b"\\",
        b"<Esc",
        b"<",
        b"<>",
        b"<C->>",
        b"<C-->",
        b"<A-S-a>",
        b"<S-a>",
        b"<D-x>",
        b"<Nop>",
        b"<Plug>(x)",
        b"<F12>",
        b"<ScrollWheelUp>",
        b"<2-LeftMouse>",
        b"<C-S-M-A-D-T-x>",
        b"<Q-x>",
        b"<C-Space>",
        b"<Bar>",
        b"<Bslash>",
        b"<k0>",
        b"<Nul>",
        b"<C-v>",
        b"\x80\xfe\x58",
        b"\x80\xfd\x04",
        b"\x80",
        b"\x80\x80\x80",
    ];
    let mut out: Vec<Vec<u8>> = names.iter().map(|s| s.to_vec()).collect();
    let rest = corpus();
    for (i, name) in names.iter().enumerate() {
        let mut glued = name.to_vec();
        glued.extend_from_slice(&rest[(i * 7) % rest.len()]);
        out.push(glued);
    }
    out.extend(rest);
    out
}

#[test]
fn reading_one_key_name() {
    let _editor = editor_lock();
    let mut d = Digest::new();
    for text in texts() {
        for at in 0..text.len() {
            for flags in [
                0,
                FSK_KEYCODE,
                FSK_KEYCODE | FSK_SIMPLIFY,
                FSK_IN_STRING | FSK_KEEP_X_KEY,
                FSK_KEYCODE | FSK_IN_STRING | FSK_SIMPLIFY,
            ] {
                for len in [text.len() - at, (text.len() - at).min(3)] {
                    let name = &text[at..at + len];
                    let mut simplified = false;
                    let found = special_key_at(name, flags, Some(&mut simplified));
                    let (key, modifiers, used) = found.unwrap_or((0, ModMask::NONE, 0));
                    d.add(i64::from(key));
                    d.add(i64::from(modifiers.bits()));
                    d.add(i64::from(simplified));
                    d.add((at + used) as i64);

                    for escape_ks in [false, true] {
                        let mut out = Vec::new();
                        let _ = trans_special_into(name, flags, escape_ks, &mut out);
                        d.add(out.len() as i64);
                        d.bytes(&out);
                    }
                }
            }
        }
    }
    // Recorded from `find_special_key`/`trans_special`, each call from the
    // start of the name.
    assert_eq!(d.value(), 0x241f680d35c8f649, "one key name");
}

#[test]
fn replacing_key_names() {
    let _editor = editor_lock();
    let mut d = Digest::new();
    for text in texts() {
        // Every combination of the four `REPTERM_*` bits.
        let bits = [
            REPTERM_FROM_PART,
            REPTERM_DO_LT,
            REPTERM_NO_SPECIAL,
            REPTERM_NO_SIMPLIFY,
        ];
        for mask in 0..16 {
            let flags: c_int = (0..4)
                .filter(|i| mask & (1 << i) != 0)
                .map(|i| bits[i])
                .sum();
            for cpo in [c"aABceFs_" as &CStr, c"aAceFs_"] {
                for sid in [0, 3, -1] {
                    let mut simplified = false;
                    let out =
                        replace_termcodes(&text, sid, flags, Some(&mut simplified), cpo, None)
                            .expect("an unbounded translation always answers");
                    d.bytes(c_prefix(&out));
                    d.add(i64::from(simplified));
                    // The fixed 128-byte buffer a mapping's lhs is read into.
                    let fixed = replace_termcodes(&text, sid, flags, None, cpo, Some(128));
                    d.add(i64::from(fixed.is_none()));
                    if let Some(fixed) = fixed {
                        d.bytes(c_prefix(&fixed));
                    }
                }
            }
        }
    }
    assert_eq!(d.value(), 0x10197c582cb59225, "replace_termcodes");
}

#[test]
fn escaping_k_special() {
    let _editor = editor_lock();
    let mut d = Digest::new();
    for text in texts() {
        d.bytes(&escape_ks_bytes(&text));
        let mut copy = text.clone();
        let len = unescape_ks_in(&mut copy);
        d.bytes(&copy[..len]);
    }
    for c in (0..0x900).chain([0x10000, 0x10ffff, 0x7fff_ffff, -1, -0x6b80]) {
        let mut out = Vec::new();
        push_char_escaped(c, &mut out);
        d.add(out.len() as i64);
        for modifiers in [ModMask::NONE, ModMask::CTRL | ModMask::SHIFT] {
            for escape_ks in [false, true] {
                if c >= 0 || !escape_ks {
                    let mut out = Vec::new();
                    push_special_key(c, modifiers, escape_ks, &mut out);
                    d.add(out.len() as i64);
                    d.bytes(&out);
                }
            }
        }
    }
    assert_eq!(d.value(), 0x3975bf91f957d849, "K_SPECIAL escapes");
}

/// What a C reader of `bytes` sees: up to the first NUL.
fn c_prefix(bytes: &[u8]) -> &[u8] {
    bytes.split(|&b| b == 0).next().unwrap_or_default()
}
