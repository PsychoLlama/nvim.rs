//! The bundled libtermkey, driven the way the TUI drives it: bytes in one end,
//! keys out the other.
//!
//! This is a port of `test/unit/termkey_spec.lua`, which was itself a port of
//! libtermkey's own C test suite. The module's leaf pieces — UTF-8 decoding,
//! the CSI grammar, the key-name tables, the report packing, the formatter —
//! all have in-crate tests already; what none of them covers, and what the Lua
//! spec was the only oracle for, is the pipeline that joins them:
//! `tui/termkey/{termkey,driver_csi,driver_ti}.rs`, 1,774 lines that had never
//! been executed by a test in this tree.
//!
//! Several assertions the Lua spec carried as comments are made properly here.
//! They were commented out during the C-to-Lua translation because reading a
//! union arm through LuaJIT's FFI needed a cast the translator did not write;
//! they all hold.

use std::ffi::{c_int, c_uint};

use neovim::tui::termkey::termkey::{
    TERMKEY_CANON_DELBS, TERMKEY_CANON_SPACESYMBOL, TERMKEY_FLAG_SPACESYMBOL, TERMKEY_FLAG_UTF8,
    TERMKEY_FORMAT_LOWERSPACE, TERMKEY_FORMAT_MOUSE_POS, TERMKEY_KEYMOD_ALT, TERMKEY_KEYMOD_CTRL,
    TERMKEY_MOUSE_DRAG, TERMKEY_MOUSE_PRESS, TERMKEY_MOUSE_RELEASE, TERMKEY_RES_AGAIN,
    TERMKEY_RES_KEY, TERMKEY_RES_NONE, TERMKEY_SYM_BACKSPACE, TERMKEY_SYM_DEL, TERMKEY_SYM_ESCAPE,
    TERMKEY_SYM_PAGEUP, TERMKEY_SYM_RIGHT, TERMKEY_SYM_SPACE, TERMKEY_SYM_UP, TERMKEY_TYPE_DCS,
    TERMKEY_TYPE_FUNCTION, TERMKEY_TYPE_KEYSYM, TERMKEY_TYPE_MODEREPORT, TERMKEY_TYPE_MOUSE,
    TERMKEY_TYPE_OSC, TERMKEY_TYPE_POSITION, TERMKEY_TYPE_UNICODE, TERMKEY_TYPE_UNKNOWN_CSI,
    TermKey, interpret_modereport, interpret_mouse, strfkey,
};
use neovim::types::{
    KeyCode, TermKeyFormat, TermKeyKey, TermKeyMouseEvent, TermKeyResult, TermKeyType,
};

/// The default buffer, in bytes. Not a literal in the assertions below: what
/// the cases are about is that pushing N bytes costs N of whatever it is.
const DEFAULT_BUFFER: usize = 256;

/// A key reader, driven the way the TUI drives one, with the case-level
/// helpers that make the cases read as the protocol they are about.
struct Reader(TermKey);

impl Reader {
    fn new(flags: c_int) -> Self {
        // No terminfo entry: the "nothing is known about this terminal" case.
        Reader(TermKey::new(None, flags))
    }

    /// The reader itself, for the assertions about its state.
    fn state(&self) -> &TermKey {
        &self.0
    }

    fn start(&mut self) {
        self.0.start(None);
    }

    fn canonflags(&self) -> c_int {
        self.0.canonflags()
    }

    fn set_canonflags(&mut self, flags: c_int) {
        self.0.set_canonflags(flags);
    }

    fn buffer_size(&self) -> usize {
        self.0.buffer_size()
    }

    fn set_buffer_size(&mut self, size: usize) {
        self.0.set_buffer_size(size);
    }

    fn free(&self) -> usize {
        self.0.buffer_remaining()
    }

    /// Hand the reader input; answers how much of it was taken.
    fn push(&mut self, bytes: &[u8]) -> usize {
        self.0.push_bytes(bytes).expect("the buffer has room")
    }

    fn getkey(&mut self) -> (TermKeyResult, TermKeyKey) {
        let mut key = TermKeyKey::default();
        let res = self.0.getkey(&mut key);
        (res, key)
    }

    /// Read whatever is buffered as a key even if more input could still change
    /// the answer — what the TUI does when the escape timeout expires.
    fn getkey_force(&mut self) -> (TermKeyResult, TermKeyKey) {
        let mut key = TermKeyKey::default();
        let res = self.0.getkey_force(&mut key);
        (res, key)
    }

    /// Push `bytes` and read exactly one key out, insisting that one is there.
    #[track_caller]
    fn key_from(&mut self, bytes: &[u8]) -> TermKeyKey {
        assert_eq!(self.push(bytes), bytes.len(), "the buffer was too small");
        let (res, key) = self.getkey();
        assert_eq!(res, TERMKEY_RES_KEY, "no key from {bytes:?}");
        key
    }

    /// (the parameters, the packed command) of an unrecognised control
    /// sequence. Only valid while the sequence is still the head of the buffer.
    fn csi(&self, key: &TermKeyKey) -> (usize, c_uint) {
        let csi = self.0.interpret_csi(key).expect("an unrecognised CSI");
        (csi.params().len(), csi.command)
    }

    /// The payload of a DCS, OSC or APC key.
    fn string(&self, key: &TermKeyKey) -> Option<String> {
        let text = self.0.interpret_string(key)?;
        Some(String::from_utf8_lossy(text).into_owned())
    }
}

/// Render `key` whole.
fn render(key: &TermKeyKey, format: TermKeyFormat) -> String {
    String::from_utf8(strfkey(key, format)).expect("the fixtures are ASCII")
}

/// (event, button, line, column) of a mouse key.
fn mouse(key: &TermKeyKey) -> (TermKeyMouseEvent, c_int, c_int, c_int) {
    interpret_mouse(key).expect("a mouse key")
}

/// (introducer, mode, value) of a DECRPM mode report.
fn modereport(key: &TermKeyKey) -> (c_int, c_int, c_int) {
    interpret_modereport(key).expect("a mode report")
}

/// A key built by hand, the way a consumer formatting one it invented does.
fn made_key(kind: TermKeyType, code: c_int, modifiers: c_int) -> TermKeyKey {
    TermKeyKey {
        kind,
        code: KeyCode::scalar(code),
        modifiers,
        ..TermKeyKey::default()
    }
}

/// The scalar reading of a key's code; which reading is meaningful is what
/// `kind` says, and every caller here checks it first.
fn code(key: &TermKeyKey) -> c_int {
    key.code.codepoint()
}

fn utf8(key: &TermKeyKey) -> String {
    let end = key.utf8.iter().position(|&b| b == 0).unwrap_or(7);
    String::from_utf8(key.utf8[..end].to_vec()).expect("the reader only writes valid UTF-8 here")
}

#[test]
fn a_reader_is_started_when_it_is_made_and_starting_it_again_is_a_no_op() {
    let mut tk = Reader::new(0);
    assert_eq!(tk.buffer_size(), DEFAULT_BUFFER);
    assert!(tk.state().is_started);

    tk.start();
    assert!(tk.state().is_started);
}

#[test]
fn bytes_turn_into_keys_and_give_their_buffer_space_back() {
    let mut tk = Reader::new(0);
    assert_eq!(tk.free(), DEFAULT_BUFFER);
    assert_eq!(tk.getkey().0, TERMKEY_RES_NONE, "nothing has been pushed");

    assert_eq!(tk.push(b"h"), 1);
    assert_eq!(tk.free(), DEFAULT_BUFFER - 1);
    let (res, key) = tk.getkey();
    assert_eq!(res, TERMKEY_RES_KEY);
    assert_eq!(key.kind, TERMKEY_TYPE_UNICODE);
    assert_eq!(code(&key), i32::from(b'h'));
    assert_eq!(key.modifiers, 0);
    assert_eq!(utf8(&key), "h");
    assert_eq!(tk.free(), DEFAULT_BUFFER, "the byte was consumed");
    assert_eq!(tk.getkey().0, TERMKEY_RES_NONE);

    // A C0 byte with no symbol of its own is the Ctrl of its letter.
    let key = tk.key_from(b"\x01");
    assert_eq!(key.kind, TERMKEY_TYPE_UNICODE);
    assert_eq!(code(&key), i32::from(b'a'));
    assert_eq!(key.modifiers, TERMKEY_KEYMOD_CTRL as c_int);

    // SS3, from the terminfo driver's key table.
    let key = tk.key_from(b"\x1bOA");
    assert_eq!(key.kind, TERMKEY_TYPE_KEYSYM);
    assert_eq!(code(&key), TERMKEY_SYM_UP);
    assert_eq!(key.modifiers, 0);

    // Split across two writes: the first half is not yet a key.
    assert_eq!(tk.push(b"\x1bO"), 2);
    assert_eq!(tk.free(), DEFAULT_BUFFER - 2);
    assert_eq!(tk.getkey().0, TERMKEY_RES_AGAIN);
    let key = tk.key_from(b"C");
    assert_eq!(key.kind, TERMKEY_TYPE_KEYSYM);
    assert_eq!(code(&key), TERMKEY_SYM_RIGHT);
    assert_eq!(key.modifiers, 0);
    assert_eq!(tk.free(), DEFAULT_BUFFER);

    // The modifyOtherKeys encoding of a modified control key.
    let key = tk.key_from(b"\x1b[27;5u");
    assert_eq!(key.kind, TERMKEY_TYPE_KEYSYM);
    assert_eq!(code(&key), TERMKEY_SYM_ESCAPE);
    assert_eq!(key.modifiers, TERMKEY_KEYMOD_CTRL as c_int);

    // NUL is Ctrl-Space, whose codepoint is the space it modifies.
    let key = tk.key_from(b"\0");
    assert_eq!(key.kind, TERMKEY_TYPE_UNICODE);
    assert_eq!(code(&key), i32::from(b' '));
    assert_eq!(key.modifiers, TERMKEY_KEYMOD_CTRL as c_int);
}

#[test]
fn utf8_sequences_decode_at_every_width() {
    let mut tk = Reader::new(TERMKEY_FLAG_UTF8 as c_int);
    let mut one = |bytes: &[u8], want: c_int| {
        let key = tk.key_from(bytes);
        assert_eq!(key.kind, TERMKEY_TYPE_UNICODE, "{bytes:?}");
        assert_eq!(code(&key), want, "{bytes:?}");
    };

    one(b"a", i32::from(b'a'));
    // The two-byte range, starting past the C1 block.
    one(b"\xC2\xA0", 0x00A0);
    one(b"\xDF\xBF", 0x07FF);
    // The three-byte range.
    one(b"\xE0\xA0\x80", 0x0800);
    one(b"\xEF\xBF\xBD", 0xFFFD);
    // The four-byte range.
    one(b"\xF0\x90\x80\x80", 0x10000);
    one(b"\xF4\x8F\xBF\xBF", 0x10FFFF);
}

/// A byte that cannot continue the sequence in progress ends it as U+FFFD and
/// is then read on its own — the reader never swallows the byte that broke it.
#[test]
fn a_broken_utf8_sequence_yields_the_replacement_and_keeps_the_byte_after_it() {
    let mut tk = Reader::new(TERMKEY_FLAG_UTF8 as c_int);
    for broken in [
        &b"\xC2!"[..],
        &b"\xE0!"[..],
        &b"\xE0\xA0!"[..],
        &b"\xF0!"[..],
        &b"\xF0\x90!"[..],
        &b"\xF0\x90\x80!"[..],
    ] {
        let key = tk.key_from(broken);
        assert_eq!(code(&key), 0xFFFD, "{broken:?}");
        let (res, key) = tk.getkey();
        assert_eq!(res, TERMKEY_RES_KEY, "{broken:?}");
        assert_eq!(code(&key), i32::from(b'!'), "{broken:?}");
    }
}

/// The partial-feed protocol: a sequence arriving one byte at a time asks for
/// more (`RES_AGAIN`) until its last byte, and only then is it a key. This is
/// the contract the TUI's read loop is written against, and it is what the
/// escape-timeout path distinguishes from "nothing is coming".
#[test]
fn a_utf8_sequence_fed_one_byte_at_a_time_asks_for_more_until_it_is_whole() {
    let mut tk = Reader::new(TERMKEY_FLAG_UTF8 as c_int);
    let mut drip = |bytes: &[u8], want: c_int| {
        let (last, rest) = bytes.split_last().expect("a sequence has a last byte");
        for byte in rest {
            assert_eq!(tk.push(&[*byte]), 1);
            assert_eq!(
                tk.getkey().0,
                TERMKEY_RES_AGAIN,
                "{bytes:?} after {byte:#x}"
            );
        }
        let key = tk.key_from(&[*last]);
        assert_eq!(code(&key), want, "{bytes:?}");
    };

    drip(b"\xC2\xA0", 0x00A0);
    drip(b"\xE0\xA0\x80", 0x0800);
    drip(b"\xF0\x90\x80\x80", 0x10000);
}

/// Canonicalisation is what makes two spellings of one keypress into one key,
/// and it is reached only on the way out of the decoder — so the way to see it
/// is to decode the same byte under each setting of the flag that governs it.
#[test]
fn the_space_symbol_flag_decides_whether_a_space_is_a_character() {
    let mut tk = Reader::new(0);
    let key = tk.key_from(b" ");
    assert_eq!(key.kind, TERMKEY_TYPE_UNICODE);
    assert_eq!(code(&key), i32::from(b' '));
    assert_eq!(key.modifiers, 0);

    // Asking for it at construction is the direction the TUI uses.
    let mut symbolic = Reader::new(TERMKEY_FLAG_SPACESYMBOL as c_int);
    let key = symbolic.key_from(b" ");
    assert_eq!(key.kind, TERMKEY_TYPE_KEYSYM);
    assert_eq!(code(&key), TERMKEY_SYM_SPACE);
    assert_eq!(key.modifiers, 0);

    // The flag and the canonicalisation flag are two spellings of one thing and
    // are kept in step in both directions.
    assert_ne!(
        symbolic.canonflags() & TERMKEY_CANON_SPACESYMBOL as c_int,
        0
    );
    tk.set_canonflags(TERMKEY_CANON_SPACESYMBOL as c_int);
    assert_ne!(tk.state().flags & TERMKEY_FLAG_SPACESYMBOL as c_int, 0);
    let key = tk.key_from(b" ");
    assert_eq!(key.kind, TERMKEY_TYPE_KEYSYM);
    assert_eq!(code(&key), TERMKEY_SYM_SPACE);

    // And back: clearing it puts the character spelling back.
    tk.set_canonflags(0);
    assert_eq!(tk.state().flags & TERMKEY_FLAG_SPACESYMBOL as c_int, 0);
    assert_eq!(tk.key_from(b" ").kind, TERMKEY_TYPE_UNICODE);
}

/// The other canonicalisation, and the one the TUI actually turns on
/// (`tui/input.rs` ORs `TERMKEY_CANON_DELBS` in at startup): DEL is delivered
/// as Backspace.
#[test]
fn the_delbs_flag_delivers_del_as_backspace() {
    let mut tk = Reader::new(0);
    let key = tk.key_from(b"\x7f");
    assert_eq!(key.kind, TERMKEY_TYPE_KEYSYM);
    assert_eq!(code(&key), TERMKEY_SYM_DEL);

    tk.set_canonflags(tk.canonflags() | TERMKEY_CANON_DELBS as c_int);
    let key = tk.key_from(b"\x7f");
    assert_eq!(key.kind, TERMKEY_TYPE_KEYSYM);
    assert_eq!(code(&key), TERMKEY_SYM_BACKSPACE);
}

#[test]
fn resizing_the_buffer_keeps_what_is_already_in_it() {
    let mut tk = Reader::new(0);
    assert_eq!(tk.free(), DEFAULT_BUFFER);
    assert_eq!(tk.buffer_size(), DEFAULT_BUFFER);

    assert_eq!(tk.push(b"h"), 1);
    assert_eq!(tk.free(), DEFAULT_BUFFER - 1);

    tk.set_buffer_size(2 * DEFAULT_BUFFER);
    assert_eq!(tk.buffer_size(), 2 * DEFAULT_BUFFER);
    assert_eq!(tk.free(), 2 * DEFAULT_BUFFER - 1);

    let (res, key) = tk.getkey();
    assert_eq!(
        res, TERMKEY_RES_KEY,
        "the buffered byte survived the resize"
    );
    assert_eq!(code(&key), i32::from(b'h'));
}

/// A key renders as its name, its modifiers and, for a mouse key asked for
/// it, the position. Upstream rendered into the caller's buffer with a
/// running `len - pos` that wrapped once the modifiers alone overflowed it;
/// the rendering is whole now, and this is the case that pins its text.
#[test]
fn a_key_renders_whole() {
    let pageup = made_key(TERMKEY_TYPE_KEYSYM, TERMKEY_SYM_PAGEUP, 0);
    assert_eq!(render(&pageup, 0), "PageUp");
    assert_eq!(render(&pageup, TERMKEY_FORMAT_LOWERSPACE), "page up");

    // The modifiers are part of the rendering, and a mouse position makes it
    // long enough that upstream's running `len - pos` wrapped.
    let mut mouse = made_key(TERMKEY_TYPE_MOUSE, 0, TERMKEY_KEYMOD_CTRL as c_int);
    mouse.code = KeyCode::report([0, 40, 40, 0]);
    assert_eq!(
        render(&mouse, TERMKEY_FORMAT_MOUSE_POS),
        "C-MousePress(1) @ (40,40)"
    );
}

/// The three mouse wire protocols a terminal may speak. None of them was parsed
/// by anything in this tree: `report.rs` covers the *packing*, and these are
/// the sequences that produce it.
#[test]
fn mouse_reports_parse_in_all_three_protocols() {
    let mut tk = Reader::new(0);

    // X10: a CSI M and three bytes biased by 0x20.
    let key = tk.key_from(b"\x1b[M !!");
    assert_eq!(key.kind, TERMKEY_TYPE_MOUSE);
    assert_eq!(mouse(&key), (TERMKEY_MOUSE_PRESS, 1, 1, 1));
    assert_eq!(key.modifiers, 0);
    assert_eq!(render(&key, 0), "MousePress(1)");
    assert_eq!(
        render(&key, TERMKEY_FORMAT_MOUSE_POS),
        "MousePress(1) @ (1,1)"
    );

    let key = tk.key_from(b"\x1b[M@\"!");
    assert_eq!(mouse(&key), (TERMKEY_MOUSE_DRAG, 1, 1, 2));

    let key = tk.key_from(b"\x1b[M##!");
    assert_eq!(mouse(&key), (TERMKEY_MOUSE_RELEASE, 0, 1, 3));

    // The modifier bits ride in the button byte.
    let key = tk.key_from(b"\x1b[M0++");
    assert_eq!(mouse(&key), (TERMKEY_MOUSE_PRESS, 1, 11, 11));
    assert_eq!(key.modifiers, TERMKEY_KEYMOD_CTRL as c_int);
    assert_eq!(render(&key, 0), "C-MousePress(1)");

    // The wheel is buttons 4 through 7.
    let key = tk.key_from(b"\x1b[M`!!");
    assert_eq!(mouse(&key).1, 4);
    let key = tk.key_from(b"\x1b[Mb!!");
    assert_eq!(mouse(&key).1, 6);

    // rxvt: CSI with three decimal parameters and an M.
    let key = tk.key_from(b"\x1b[0;20;20M");
    assert_eq!(key.kind, TERMKEY_TYPE_MOUSE);
    assert_eq!(mouse(&key), (TERMKEY_MOUSE_PRESS, 1, 20, 20));
    assert_eq!(key.modifiers, 0);

    let key = tk.key_from(b"\x1b[3;20;20M");
    assert_eq!(key.kind, TERMKEY_TYPE_MOUSE);
    assert_eq!(mouse(&key), (TERMKEY_MOUSE_RELEASE, 0, 20, 20));

    // SGR: the release is a lowercase final byte rather than a button code, so
    // press and release share a button number.
    let key = tk.key_from(b"\x1b[<0;30;30M");
    assert_eq!(key.kind, TERMKEY_TYPE_MOUSE);
    assert_eq!(mouse(&key), (TERMKEY_MOUSE_PRESS, 1, 30, 30));
    assert_eq!(key.modifiers, 0);

    let key = tk.key_from(b"\x1b[<0;30;30m");
    assert_eq!(key.kind, TERMKEY_TYPE_MOUSE);
    assert_eq!(mouse(&key).0, TERMKEY_MOUSE_RELEASE);

    // Past a byte's worth of either coordinate, which is where the packing's
    // high bits earn their keep.
    let key = tk.key_from(b"\x1b[<0;500;300M");
    let (_, _, line, col) = mouse(&key);
    assert_eq!((line, col), (300, 500));
}

#[test]
fn a_cursor_position_report_parses_and_a_bare_csi_r_is_still_f3() {
    let mut tk = Reader::new(0);
    let key = tk.key_from(b"\x1b[?15;7R");
    assert_eq!(key.kind, TERMKEY_TYPE_POSITION);

    // Without the private-mode introducer the same final byte is a function
    // key, which is why the reports the TUI asks for use `CSI ?`.
    let key = tk.key_from(b"\x1b[R");
    assert_eq!(key.kind, TERMKEY_TYPE_FUNCTION);
    assert_eq!(code(&key), 3);
}

#[test]
fn a_mode_report_parses_its_introducer_mode_and_value() {
    let mut tk = Reader::new(0);
    let key = tk.key_from(b"\x1b[?1;2$y");
    assert_eq!(key.kind, TERMKEY_TYPE_MODEREPORT);
    assert_eq!(modereport(&key), (i32::from(b'?'), 1, 2));

    // An ANSI mode has no introducer, which is reported as zero rather than as
    // a missing field.
    let key = tk.key_from(b"\x1b[4;1$y");
    assert_eq!(key.kind, TERMKEY_TYPE_MODEREPORT);
    assert_eq!(modereport(&key), (0, 4, 1));
}

/// An unrecognised control sequence is handed back whole: the parameters are
/// still in the buffer and the command packs the private-mode introducer and
/// any intermediate byte alongside the final one.
#[test]
fn an_unrecognised_control_sequence_can_be_re_read() {
    let mut tk = Reader::new(0);
    let key = tk.key_from(b"\x1b[5;25v");
    assert_eq!(key.kind, TERMKEY_TYPE_UNKNOWN_CSI);
    assert_eq!(tk.csi(&key), (2, c_uint::from(b'v')));

    let key = tk.key_from(b"\x1b[?w");
    assert_eq!(key.kind, TERMKEY_TYPE_UNKNOWN_CSI);
    assert_eq!(
        tk.csi(&key).1,
        (c_uint::from(b'?') << 8) | c_uint::from(b'w')
    );

    let key = tk.key_from(b"\x1b[?$x");
    assert_eq!(key.kind, TERMKEY_TYPE_UNKNOWN_CSI);
    assert_eq!(
        tk.csi(&key).1,
        (c_uint::from(b'$') << 16) | (c_uint::from(b'?') << 8) | c_uint::from(b'x')
    );
}

/// Control strings — DCS and OSC — in both their seven-bit and eight-bit
/// framings, and the false alarm: an ESC that turns out not to introduce one.
#[test]
fn control_strings_come_back_whole_in_both_framings() {
    let mut tk = Reader::new(0);

    // DCS, 7-bit: ESC P ... ESC \.
    let key = tk.key_from(b"\x1bP1$r1 q\x1b\\");
    assert_eq!(key.kind, TERMKEY_TYPE_DCS);
    assert_eq!(key.modifiers, 0);
    assert_eq!(tk.string(&key).as_deref(), Some("1$r1 q"));
    assert_eq!(
        tk.getkey().0,
        TERMKEY_RES_NONE,
        "the whole string was eaten"
    );

    // DCS, 8-bit: the single bytes 0x90 and 0x9c.
    let key = tk.key_from(b"\x901$r2 q\x9c");
    assert_eq!(key.kind, TERMKEY_TYPE_DCS);
    assert_eq!(key.modifiers, 0);
    assert_eq!(tk.string(&key).as_deref(), Some("1$r2 q"));
    assert_eq!(tk.getkey().0, TERMKEY_RES_NONE);

    // OSC, 7-bit.
    let key = tk.key_from(b"\x1b]15;abc\x1b\\");
    assert_eq!(key.kind, TERMKEY_TYPE_OSC);
    assert_eq!(key.modifiers, 0);
    assert_eq!(tk.string(&key).as_deref(), Some("15;abc"));
    assert_eq!(tk.getkey().0, TERMKEY_RES_NONE);

    // A key held past the next control string cannot read a payload that is no
    // longer its own: each string gets a serial number.
    let stale = key;
    let fresh = tk.key_from(b"\x1b]9;xyz\x1b\\");
    assert_eq!(tk.string(&fresh).as_deref(), Some("9;xyz"));
    assert_eq!(tk.string(&stale), None);

    // False alarm: an ESC P with nothing after it. Waiting says "more may be
    // coming"; forcing says it was Alt-P all along.
    assert_eq!(tk.push(b"\x1bP"), 2);
    assert_eq!(tk.getkey().0, TERMKEY_RES_AGAIN);
    let (res, key) = tk.getkey_force();
    assert_eq!(res, TERMKEY_RES_KEY);
    assert_eq!(key.kind, TERMKEY_TYPE_UNICODE);
    assert_eq!(code(&key), i32::from(b'P'));
    assert_eq!(key.modifiers, TERMKEY_KEYMOD_ALT as c_int);
}
