//! The typeahead's escape spelling of a key, end to end: every named special
//! key, alone and under each modifier the side channel carries, spelled as
//! the bytes the typeahead stores (`K_SPECIAL KS_MODIFIER <bits>` and
//! `K_SPECIAL <termcap pair>`), fed with `feedkeys()` and read back with
//! `getchar()`/`getcharmod()` -- which is `vgetc` assembling the key and
//! setting `MOD_MASK`. A few characters ride along: the escaped NUL and
//! `K_SPECIAL`, a multibyte one, and the CTRL forms that fold into a byte.
//!
//! The answers are folded into one digest recorded before the key pipeline
//! was retyped; the halves that are pure functions (`Key` <-> `SpecialKey`,
//! the escape and its inverse, the modifier byte) are asserted directly.

#![cfg(not(miri))]

use neovim::keycodes::{
    K_SPECIAL, KS_MODIFIER, ModMask, SpecialKey, key_escape, key_unescape, special_key_bytes,
};
use neovim::types::Key;

use crate::support::corpus::Digest;
use crate::support::{Probe, Sandbox};

/// The modifier sets every key is fed under: none, each flag alone, a pair,
/// and the click counts that share the two-bit field.
const MODIFIERS: [ModMask; 10] = [
    ModMask::NONE,
    ModMask::SHIFT,
    ModMask::CTRL,
    ModMask::ALT,
    ModMask::META,
    ModMask::CMD,
    ModMask::SHIFT.or(ModMask::CTRL),
    ModMask::TWO_CLICK,
    ModMask::THREE_CLICK,
    ModMask::FOUR_CLICK.or(ModMask::ALT),
];

/// The characters fed beside the named keys.
fn characters() -> Vec<Key> {
    let mut out = vec![
        Key::byte(K_SPECIAL),
        Key::ascii(b'a'),
        Key::ascii(b'6'),
        Key::ascii(b'@'),
        Key::ascii(b'?'),
        Key::ESC,
        Key::from_char('é'),
        Key::from_char('€'),
    ];
    out.extend((b'A'..=b'Z').map(Key::ascii));
    out
}

#[test]
fn every_special_key_converts_both_ways() {
    for &special in &SpecialKey::ALL {
        let key = Key::from(special);
        assert!(key.is_special(), "{special:?}");
        assert_eq!(key.special(), Some(special));
        assert_eq!(key.code(), special.code());
        assert_eq!(Key::from_code(special.code()), special);
        assert_eq!(key.as_char(), None);
        assert_eq!(key.as_byte(), None);
        // The escape and its inverse.
        let [lead, second, third] = key_escape(key);
        assert_eq!(lead, K_SPECIAL);
        assert_eq!(key_unescape(second, third), key, "{special:?}");
        // With every modifier set: the prefix carries the bits as one byte.
        for modifiers in MODIFIERS {
            let bytes = special_key_bytes(key, modifiers, true);
            let bytes = bytes.as_bytes();
            let body = if modifiers.is_empty() {
                bytes
            } else {
                assert_eq!(&bytes[..2], &[K_SPECIAL, KS_MODIFIER]);
                assert_eq!(ModMask::from_byte(bytes[2]), modifiers);
                &bytes[3..]
            };
            assert_eq!(body, key_escape(key), "{special:?} {:#x}", modifiers.bits());
        }
    }
    // The two bytes that are escaped although they are characters.
    assert_eq!(
        key_unescape(key_escape(Key::NUL)[1], key_escape(Key::NUL)[2]),
        SpecialKey::Zero
    );
    let ks = Key::byte(K_SPECIAL);
    assert_eq!(key_unescape(key_escape(ks)[1], key_escape(ks)[2]), ks);
    // Characters are not special keys, and say which character they are.
    for c in characters() {
        assert!(!c.is_special());
        assert_eq!(c.special(), None);
    }
    assert_eq!(Key::from_char('é').as_char(), Some('é'));
    assert!(Key::ascii(b'x').is('x'));
    assert_eq!(Key::ctrl(b'V').code(), 0x16);
    assert_eq!(Key::ctrl(b'['), Key::ESC);
    assert_eq!(Key::from_char('€').utf8_len(), 3);
}

/// `bytes` as the body of a Vim script double-quoted string.
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("\\x{b:02x}")).collect()
}

const HELPERS: &[&str] = &[
    r"function! GoKeys(keys)",
    r"  let res = []",
    r"  for k in a:keys",
    r"    call feedkeys(k, 'n')",
    r"    let got = []",
    r"    while 1",
    r"      let c = getchar(0)",
    r"      if c is 0",
    r"        break",
    r"      endif",
    r"      call add(got, [c, getcharmod()])",
    r"    endwhile",
    r"    call add(res, got)",
    r"  endfor",
    r"  return res",
    r"endfunction",
];

#[test]
fn every_special_key_round_trips_through_the_typeahead() {
    let sandbox = Sandbox::dir("key_oracle");
    let fx = Probe::new(sandbox.editor());
    fx.home();
    fx.run("set all&");
    let quoted: Vec<String> = HELPERS
        .iter()
        .map(|line| format!("'{}'", line.replace('\'', "''")))
        .collect();
    fx.run(&format!("call execute([{}])", quoted.join(", ")));

    // Keys that read more of the typeahead when they arrive (the command a
    // `<Cmd>` carries, a `<Lua>` reference, a paste) are left out: what
    // follows them is theirs, not the next key.
    let skipped = [SpecialKey::Command, SpecialKey::Lua, SpecialKey::PasteStart];
    let keys: Vec<Key> = SpecialKey::ALL
        .iter()
        .filter(|special| !skipped.contains(special))
        .map(|&special| Key::from(special))
        .chain(characters())
        .collect();
    fx.run("let g:keys = []");
    for &key in &keys {
        for modifiers in MODIFIERS {
            let bytes = special_key_bytes(key, modifiers, true);
            fx.run(&format!("call add(g:keys, \"{}\")", hex(bytes.as_bytes())));
        }
    }
    // A NUL is spelled as its escape (a string cannot hold the byte), alone
    // and after a modifier.
    for modifiers in MODIFIERS {
        let mut bytes = special_key_bytes(Key::ascii(b'x'), modifiers, true)
            .as_bytes()
            .to_vec();
        let at = bytes.len() - 1;
        bytes.splice(at.., key_escape(Key::NUL));
        fx.run(&format!("call add(g:keys, \"{}\")", hex(&bytes)));
    }
    let mut d = Digest::new();
    d.bytes(fx.eval("GoKeys(g:keys)").as_bytes());
    let digest = d.value();
    fx.run("delfunction GoKeys");
    fx.run("unlet g:keys");
    fx.run("set all&");
    fx.home();
    assert_eq!(digest, EXPECTED, "key round trip digest moved: {digest:#x}");
}

/// Recorded on the `c_int` pipeline the `Key` newtype replaced.
const EXPECTED: u64 = 0xd6fa_ee74_6683_6007;
