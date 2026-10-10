//! The pointer-form `<>` notation readers and the `K_SPECIAL` escapers,
//! folded into one number each over a corpus of key names and
//! `support::corpus`, recorded from the forms upstream ported. A rewrite onto
//! slices has to reproduce every digest; see `mbyte_oracle.rs` for the rule.

#![cfg(not(miri))]

use std::ffi::{CStr, c_char, c_int};
use std::ptr;

use neovim::keycodes::{
    FSK_IN_STRING, FSK_KEEP_X_KEY, FSK_KEYCODE, FSK_SIMPLIFY, ModMask, REPTERM_DO_LT,
    REPTERM_FROM_PART, REPTERM_NO_SIMPLIFY, REPTERM_NO_SPECIAL, add_char2buf, find_special_key,
    replace_termcodes, special_to_buf, trans_special, vim_strsave_escape_ks, vim_unescape_ks,
};

use crate::support::corpus::{Digest, corpus};
use crate::support::{editor_lock, take_bytes};

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

/// `s` as a NUL-terminated C string.
fn cbuf(s: &[u8]) -> Vec<c_char> {
    s.iter()
        .map(|&b| b as c_char)
        .chain(std::iter::once(0))
        .collect()
}

#[test]
fn reading_one_key_name() {
    let _editor = editor_lock();
    let mut d = Digest::new();
    for text in texts() {
        let buf = cbuf(&text);
        for at in 0..text.len() {
            for flags in [
                0,
                FSK_KEYCODE,
                FSK_KEYCODE | FSK_SIMPLIFY,
                FSK_IN_STRING | FSK_KEEP_X_KEY,
                FSK_KEYCODE | FSK_IN_STRING | FSK_SIMPLIFY,
            ] {
                for len in [text.len() - at, (text.len() - at).min(3)] {
                    let mut src = buf[at..].as_ptr();
                    let mut modifiers = ModMask::NONE;
                    let mut simplified = false;
                    // SAFETY: `len` bytes are readable at `src`; the
                    // out-arguments are locals.
                    let key = unsafe {
                        find_special_key(
                            &raw mut src,
                            len,
                            &raw mut modifiers,
                            flags,
                            &raw mut simplified,
                        )
                    };
                    d.add(i64::from(key));
                    d.add(i64::from(modifiers.bits()));
                    d.add(i64::from(simplified));
                    // SAFETY: `src` was advanced inside `buf`.
                    d.add(unsafe { src.offset_from(buf.as_ptr()) } as i64);

                    let mut src = buf[at..].as_ptr();
                    let mut out = [0 as c_char; 32];
                    for escape_ks in [false, true] {
                        // SAFETY: as above; `out` has the 19 bytes a key's
                        // encoding can take.
                        let n = unsafe {
                            trans_special(
                                &raw mut src,
                                len,
                                out.as_mut_ptr(),
                                flags,
                                escape_ks,
                                ptr::null_mut(),
                            )
                        };
                        d.add(i64::from(n));
                        d.bytes(
                            &out[..n as usize]
                                .iter()
                                .map(|&b| b as u8)
                                .collect::<Vec<_>>(),
                        );
                    }
                }
            }
        }
    }
    assert_eq!(d.value(), 0xecb440fb908c3995, "one key name");
}

#[test]
fn replacing_key_names() {
    let _editor = editor_lock();
    let mut d = Digest::new();
    for text in texts() {
        let buf = cbuf(&text);
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
                    let mut out: *mut c_char = ptr::null_mut();
                    let mut simplified = false;
                    // SAFETY: `buf` holds `text.len()` bytes; `out` starts
                    // null, so the answer is allocated.
                    unsafe {
                        replace_termcodes(
                            buf.as_ptr(),
                            text.len(),
                            &raw mut out,
                            sid,
                            flags,
                            &raw mut simplified,
                            cpo,
                        );
                        d.bytes(&take_bytes(out));
                    }
                    d.add(i64::from(simplified));
                    // The fixed 128-byte buffer a mapping's lhs is read into.
                    let mut fixed = [0x55 as c_char; 128];
                    let mut out = fixed.as_mut_ptr();
                    // SAFETY: as above, into a buffer of the documented size.
                    let res = unsafe {
                        replace_termcodes(
                            buf.as_ptr(),
                            text.len(),
                            &raw mut out,
                            sid,
                            flags,
                            ptr::null_mut(),
                            cpo,
                        )
                    };
                    d.add(i64::from(res.is_null()));
                    if !res.is_null() {
                        let n = fixed.iter().position(|&b| b == 0).unwrap_or(128);
                        d.bytes(&fixed[..n].iter().map(|&b| b as u8).collect::<Vec<_>>());
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
        let mut buf = cbuf(&text);
        // SAFETY: `buf` is NUL-terminated and writable.
        unsafe {
            let escaped = take_bytes(vim_strsave_escape_ks(buf.as_mut_ptr()));
            d.bytes(&escaped);
            vim_unescape_ks(buf.as_mut_ptr());
        }
        let n = buf.iter().position(|&b| b == 0).unwrap_or(0);
        d.bytes(&buf[..n].iter().map(|&b| b as u8).collect::<Vec<_>>());
    }
    for c in (0..0x900).chain([0x10000, 0x10ffff, 0x7fff_ffff, -1, -0x6b80]) {
        let mut out = [0 as c_char; 32];
        // SAFETY: `out` has room for an escaped character, or for three
        // modifier bytes and a key.
        unsafe {
            let end = add_char2buf(c, out.as_mut_ptr());
            d.add(end.offset_from(out.as_ptr()) as i64);
            for modifiers in [ModMask::NONE, ModMask::CTRL | ModMask::SHIFT] {
                for escape_ks in [false, true] {
                    if c >= 0 || !escape_ks {
                        let n = special_to_buf(c, modifiers, escape_ks, out.as_mut_ptr());
                        d.add(i64::from(n));
                        d.bytes(
                            &out[..n as usize]
                                .iter()
                                .map(|&b| b as u8)
                                .collect::<Vec<_>>(),
                        );
                    }
                }
            }
        }
    }
    assert_eq!(d.value(), 0x3975bf91f957d849, "K_SPECIAL escapes");
}
