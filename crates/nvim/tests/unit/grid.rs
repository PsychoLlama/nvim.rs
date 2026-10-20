//! The glyph in a screen cell: `ScreenChar` and its intern table.
//!
//! The oracle the `grid/schar.rs` retype is checked against. A cell's glyph
//! is a `u32`: up to four UTF-8 bytes inline, or -- for a longer grapheme
//! cluster -- an index into the process-wide glyph cache, tagged by a low
//! `0xFF` byte. The cases pin both forms, the round trip through
//! `schar_get`/`schar_get_adv`, the cell widths, the longest glyph the cache
//! takes (31 bytes, so that it and its NUL fit `MAX_SCHAR_SIZE`), and Arabic
//! shaping, which rebuilds glyphs from their codepoints.
//!
//! No editor is needed: the cache is a plain table, so these run under Miri
//! too, which watches the inline form's byte reinterpretation and the
//! cache's arena (the two answers that read the editor's character tables,
//! a cell width and the lam-alef ligature, are left to the normal lane). They
//! take a lock of their own there -- the cache is one
//! global -- and the editor lock otherwise, because the statusline cases
//! intern glyphs in the same table.

use std::ffi::{c_char, c_int};

use neovim::grid::{
    MAX_SCHAR_SIZE, line_do_arabic_shape, schar_cells, schar_from_buf, schar_from_char,
    schar_from_str, schar_get, schar_get_adv, schar_get_ascii, schar_get_first_codepoint,
    schar_high, schar_len,
};
use neovim::types::ScreenChar;

#[cfg(not(miri))]
fn lock() -> crate::support::Editor {
    crate::support::editor_lock()
}

#[cfg(miri)]
fn lock() -> std::sync::MutexGuard<'static, ()> {
    static CACHE: std::sync::Mutex<()> = std::sync::Mutex::new(());
    CACHE.lock().unwrap_or_else(|e| e.into_inner())
}

fn intern(text: &str) -> ScreenChar {
    schar_from_buf(text.as_bytes())
}

/// `sc`'s bytes, through `schar_get` into a buffer of exactly
/// `MAX_SCHAR_SIZE`, checking the terminator it writes.
fn text(sc: ScreenChar) -> Vec<u8> {
    let mut buf = [0x55u8; MAX_SCHAR_SIZE as usize];
    // SAFETY: `buf` has room for `MAX_SCHAR_SIZE` bytes.
    let len = unsafe { schar_get(buf.as_mut_ptr().cast::<c_char>(), sc) };
    assert_eq!(buf[len], 0, "schar_get terminates the glyph");
    assert_eq!(schar_len(sc), len);
    buf[..len].to_vec()
}

#[test]
fn up_to_four_bytes_are_stored_inline() {
    let _lock = lock();
    let cases: &[(&str, ScreenChar, c_int)] = &[
        ("a", 0x61, 1),
        ("\u{e9}", 0xa9c3, 1),
        ("日", 0x00a5_97e6, 2),
        ("😀", 0x8098_9ff0, 2),
        // A base and one combining mark, three bytes.
        ("e\u{301}", 0x0081_cc65, 1),
    ];
    for &(glyph, want, cells) in cases {
        let sc = intern(glyph);
        assert_eq!(sc, want, "{glyph:?} inline");
        assert!(!schar_high(sc), "{glyph:?} is not in the cache");
        assert_eq!(text(sc), glyph.as_bytes());
        // Cell widths read the printable-character tables, which only an
        // initialised editor has: under Miri `é` is `<e9>`, four cells.
        #[cfg(not(miri))]
        assert_eq!(schar_cells(sc), cells, "{glyph:?} cells");
        #[cfg(miri)]
        let _ = cells;
    }
    // The empty glyph is zero, which is also the right half of a
    // double-width cell.
    assert_eq!(intern(""), 0);
    assert_eq!(text(0), b"");
    // `schar_from_str` measures to the NUL; the empty string is the empty
    // glyph (upstream's null pointer answers the same).
    assert_eq!(schar_from_str(c"ab"), 0x6261);
    assert_eq!(schar_from_str(c""), 0);
}

#[test]
fn longer_glyphs_are_interned_once() {
    let _lock = lock();
    // Five bytes: a base and two combining marks.
    let five = "e\u{301}\u{302}";
    let a = intern(five);
    assert!(schar_high(a), "five bytes go to the cache");
    assert_eq!(a & 0xff, 0xff);
    // The same bytes intern to the same index; other bytes do not.
    assert_eq!(intern(five), a);
    let other = intern("o\u{301}\u{302}");
    assert!(schar_high(other));
    assert_ne!(other, a);
    assert_eq!(text(a), five.as_bytes());
    assert_eq!(text(other), "o\u{301}\u{302}".as_bytes());
    assert_eq!(schar_cells(a), 1);
    assert_eq!(schar_get_first_codepoint(a), 'e' as c_int);
    // ASCII-only answers ASCII, NUL for anything else.
    assert_eq!(schar_get_ascii(intern("x")) as u8, b'x');
    assert_eq!(schar_get_ascii(a), 0);
    assert_eq!(schar_get_ascii(intern("\u{e9}")), 0);
}

#[test]
fn the_longest_glyph_and_its_terminator_fill_the_buffer() {
    let _lock = lock();
    // A base and fifteen two-byte combining marks: 31 bytes, the most a
    // glyph may have, so its NUL lands on the buffer's last byte.
    let mut long = String::from("a");
    for _ in 0..15 {
        long.push('\u{301}');
    }
    assert_eq!(long.len(), MAX_SCHAR_SIZE as usize - 1);
    let sc = intern(&long);
    assert!(schar_high(sc));
    assert_eq!(text(sc), long.as_bytes());
    assert_eq!(schar_get_first_codepoint(sc), 'a' as c_int);
    assert_eq!(schar_cells(sc), 1);
}

/// A 32-byte glyph leaves no room for the NUL; a debug build refuses it at
/// the door, as upstream's `assert()` does.
#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "len < MAX_SCHAR_SIZE")]
fn a_glyph_of_max_schar_size_bytes_is_refused() {
    let _lock = lock();
    let mut long = String::from("ab");
    for _ in 0..15 {
        long.push('\u{301}');
    }
    assert_eq!(long.len(), MAX_SCHAR_SIZE as usize);
    let _ = intern(&long);
}

#[test]
fn schar_get_adv_builds_a_string_from_glyphs() {
    let _lock = lock();
    let glyphs = [
        intern("a"),
        intern("日"),
        intern("e\u{301}\u{302}"),
        0,
        intern("z"),
    ];
    let mut buf = [0u8; 5 * MAX_SCHAR_SIZE as usize];
    let mut at: &mut [u8] = &mut buf;
    let mut total = 0;
    for &sc in &glyphs {
        total += schar_get_adv(&mut at, sc);
    }
    let want = "a日e\u{301}\u{302}z";
    assert_eq!(total, want.len());
    // The slice advanced past exactly what was written.
    assert_eq!(at.len(), buf.len() - want.len());
    assert_eq!(&buf[..total], want.as_bytes());
    // No terminator is written: the byte after is as it was.
    assert_eq!(buf[total], 0);
}

#[test]
fn schar_from_char_encodes_one_codepoint() {
    let _lock = lock();
    assert_eq!(schar_from_char('A' as c_int), 0x41);
    assert_eq!(text(schar_from_char(0x65e5)), "日".as_bytes());
    assert_eq!(text(schar_from_char(0x1f600)), "😀".as_bytes());
    // Out of range becomes the replacement character.
    assert_eq!(text(schar_from_char(0x20_0000)), "\u{fffd}".as_bytes());
    assert_eq!(schar_from_char(0), 0);
}

#[test]
fn arabic_text_takes_its_contextual_forms() {
    let _lock = lock();
    let shape = |text: &str| -> Vec<u32> {
        let mut cells: Vec<ScreenChar> =
            text.chars().map(|c| schar_from_char(c as c_int)).collect();
        line_do_arabic_shape(&mut cells);
        cells
            .iter()
            .map(|&sc| {
                let bytes = self::text(sc);
                std::str::from_utf8(&bytes)
                    .unwrap()
                    .chars()
                    .next()
                    .map_or(0, |c| c as u32)
            })
            .collect()
    };
    // The cells are in screen order, left to right, and Arabic reads right
    // to left: the cell to a letter's *right* is the one before it. So ب س م
    // laid out left to right shape as final, medial, initial -- upstream's
    // screen shows the same (`screenstring()` under 'arabicshape').
    assert_eq!(shape("\u{628}\u{633}\u{645}"), [0xfe90, 0xfeb4, 0xfee3]);
    // Alone, a letter keeps its isolated form; one that does not join on
    // to the next (ر) breaks the chain.
    assert_eq!(shape("\u{628}"), [0xfe8f]);
    assert_eq!(shape("\u{628}\u{631}\u{628}"), [0xfe8f, 0xfeae, 0xfe91]);
    // Lam and alef in two cells are two isolated letters (read right to
    // left they are alef, lam); in one cell -- the alef composed on the lam,
    // as `win_line` builds it under 'arabicshape' -- they become the
    // ligature, and the cell holds that one codepoint.
    assert_eq!(shape("\u{644}\u{627}"), [0xfedd, 0xfe8d]);
    // The ligature reads the editor's character tables too.
    #[cfg(not(miri))]
    {
        let mut cell = [intern("\u{644}\u{627}")];
        line_do_arabic_shape(&mut cell);
        assert_eq!(text(cell[0]), "\u{fefb}".as_bytes());
    }
    // Text with no Arabic is left alone.
    assert_eq!(shape("ab"), ['a' as u32, 'b' as u32]);
}
