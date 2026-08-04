//! Port of `test/unit/keycodes_spec.lua`, widened into a golden-source test
//! of the key-name tables.
//!
//! The two tables below are the pre-rewrite behaviour of record, and exist so
//! a rewrite of `keycodes.rs`'s four frozen tables and its generated hash
//! dispatch has an entry-for-entry gate that does not need a running editor.
//!
//! * [`TERMCODES`] is section 8 of the `keysweep` differential (106 key names
//!   x the four `replace_termcodes` flag combinations `nvim_replace_termcodes`
//!   can ask for), lifted verbatim from that sweep's pre-batch baseline.
//! * [`KEY_CODES`] is one row per entry of `key_names_table` — every name the
//!   table carries, the code `get_special_key_code` answers for it, and the
//!   name `get_special_key_name` gives that code back. Together those pin the
//!   name -> code direction, the code -> preferred-name direction (which is
//!   what the `is_alt` column selects), and the table's alternative names.

use std::ffi::{CStr, c_char, c_int};
use std::ptr;

use c2rust_neovim::src::nvim::keycodes::{
    FSK_IN_STRING, MOD_MASK_ALT, MOD_MASK_CMD, MOD_MASK_CTRL, MOD_MASK_META, MOD_MASK_SHIFT,
    REPTERM_DO_LT, REPTERM_FROM_PART, REPTERM_NO_SPECIAL, find_special_key, get_special_key_code,
    get_special_key_name, replace_termcodes,
};

use crate::support::{cstr, take_bytes};

/// `'cpoptions'` as `keysweep` set it — the default, which contains `B`, so
/// `replace_termcodes` does *not* treat a backslash as CTRL-V.
const CPO: &CStr = c"aABceFs_";

/// One row of [`TERMCODES`]: `(name, termcodes, no_special, no_lt,
/// no_from_part)`.
type Encodings = (
    &'static str,
    &'static [u8],
    &'static [u8],
    &'static [u8],
    &'static [u8],
);

/// The four encodings of `<name>` that `nvim_replace_termcodes` can produce,
/// as `(from_part, do_lt, special)` = `(T,T,T)`, `(T,T,F)`, `(T,F,T)`,
/// `(F,T,T)`.
#[rustfmt::skip]
const TERMCODES: &[Encodings] = &[
    ("Esc", b"\x1b", b"<Esc>", b"\x1b", b"\x1b"),
    ("CR", b"\x0d", b"<CR>", b"\x0d", b"\x0d"),
    ("Return", b"\x0d", b"<Return>", b"\x0d", b"\x0d"),
    ("Enter", b"\x0d", b"<Enter>", b"\x0d", b"\x0d"),
    ("NL", b"\x0a", b"<NL>", b"\x0a", b"\x0a"),
    ("LF", b"\x0a", b"<LF>", b"\x0a", b"\x0a"),
    ("Tab", b"\x09", b"<Tab>", b"\x09", b"\x09"),
    ("Space", b" ", b"<Space>", b" ", b" "),
    ("BS", b"\x80kb", b"<BS>", b"\x80kb", b"\x80kb"),
    ("Del", b"\x80kD", b"<Del>", b"\x80kD", b"\x80kD"),
    ("Nul", b"\x80\xffX", b"<Nul>", b"\x80\xffX", b"\x80\xffX"),
    ("Bslash", b"\\", b"<Bslash>", b"\\", b"\\"),
    ("Bar", b"|", b"<Bar>", b"|", b"|"),
    ("lt", b"<", b"<lt>", b"<lt>", b"<"),
    ("Nop", b"<Nop>", b"<Nop>", b"<Nop>", b"<Nop>"),
    ("Ignore", b"\x80\xfd5", b"<Ignore>", b"\x80\xfd5", b"\x80\xfd5"),
    ("Undo", b"\x80&8", b"<Undo>", b"\x80&8", b"\x80&8"),
    ("Help", b"\x80%1", b"<Help>", b"\x80%1", b"\x80%1"),
    ("Insert", b"\x80kI", b"<Insert>", b"\x80kI", b"\x80kI"),
    ("Up", b"\x80ku", b"<Up>", b"\x80ku", b"\x80ku"),
    ("Down", b"\x80kd", b"<Down>", b"\x80kd", b"\x80kd"),
    ("Left", b"\x80kl", b"<Left>", b"\x80kl", b"\x80kl"),
    ("Right", b"\x80kr", b"<Right>", b"\x80kr", b"\x80kr"),
    ("Home", b"\x80kh", b"<Home>", b"\x80kh", b"\x80kh"),
    ("End", b"\x80@7", b"<End>", b"\x80@7", b"\x80@7"),
    ("PageUp", b"\x80kP", b"<PageUp>", b"\x80kP", b"\x80kP"),
    ("PageDown", b"\x80kN", b"<PageDown>", b"\x80kN", b"\x80kN"),
    ("F1", b"\x80k1", b"<F1>", b"\x80k1", b"\x80k1"),
    ("F5", b"\x80k5", b"<F5>", b"\x80k5", b"\x80k5"),
    ("F12", b"\x80F2", b"<F12>", b"\x80F2", b"\x80F2"),
    ("F37", b"\x80FR", b"<F37>", b"\x80FR", b"\x80FR"),
    ("S-F1", b"\x80\xfd\x06", b"<S-F1>", b"\x80\xfd\x06", b"\x80\xfd\x06"),
    ("xF1", b"\x80k1", b"<xF1>", b"\x80k1", b"\x80k1"),
    ("xUp", b"\x80ku", b"<xUp>", b"\x80ku", b"\x80ku"),
    ("xEnd", b"\x80@7", b"<xEnd>", b"\x80@7", b"\x80@7"),
    ("k0", b"\x80KC", b"<k0>", b"\x80KC", b"\x80KC"),
    ("k9", b"\x80KL", b"<k9>", b"\x80KL", b"\x80KL"),
    ("kPlus", b"\x80K6", b"<kPlus>", b"\x80K6", b"\x80K6"),
    ("kMinus", b"\x80K7", b"<kMinus>", b"\x80K7", b"\x80K7"),
    ("kMultiply", b"\x80K9", b"<kMultiply>", b"\x80K9", b"\x80K9"),
    ("kDivide", b"\x80K8", b"<kDivide>", b"\x80K8", b"\x80K8"),
    ("kEnter", b"\x80KA", b"<kEnter>", b"\x80KA", b"\x80KA"),
    ("kPoint", b"\x80KB", b"<kPoint>", b"\x80KB", b"\x80KB"),
    ("kComma", b"\x80KM", b"<kComma>", b"\x80KM", b"\x80KM"),
    ("kEqual", b"\x80KN", b"<kEqual>", b"\x80KN", b"\x80KN"),
    ("KP0", b"\x80\xfdO", b"<KP0>", b"\x80\xfdO", b"\x80\xfdO"),
    ("kUp", b"\x80Ku", b"<kUp>", b"\x80Ku", b"\x80Ku"),
    ("kHome", b"\x80K1", b"<kHome>", b"\x80K1", b"\x80K1"),
    ("C-A", b"\x01", b"<C-A>", b"\x01", b"\x01"),
    ("C-a", b"\x01", b"<C-a>", b"\x01", b"\x01"),
    ("C-@", b"\x80\xffX", b"<C-@>", b"\x80\xffX", b"\x80\xffX"),
    ("C-^", b"\x1e", b"<C-^>", b"\x1e", b"\x1e"),
    ("C-\\", b"\x1c", b"<C-\\>", b"\x1c", b"\x1c"),
    ("C-[", b"\x1b", b"<C-[>", b"\x1b", b"\x1b"),
    ("C-]", b"\x1d", b"<C-]>", b"\x1d", b"\x1d"),
    ("C-_", b"\x1f", b"<C-_>", b"\x1f", b"\x1f"),
    ("C-?", b"\x7f", b"<C-?>", b"\x7f", b"\x7f"),
    ("M-a", b"\x80\xfc\x08a", b"<M-a>", b"\x80\xfc\x08a", b"\x80\xfc\x08a"),
    ("A-a", b"\x80\xfc\x08a", b"<A-a>", b"\x80\xfc\x08a", b"\x80\xfc\x08a"),
    ("M-A", b"\x80\xfc\x08A", b"<M-A>", b"\x80\xfc\x08A", b"\x80\xfc\x08A"),
    ("D-a", b"\x80\xfc\x80a", b"<D-a>", b"\x80\xfc\x80a", b"\x80\xfc\x80a"),
    ("T-a", b"\x80\xfc\x10a", b"<T-a>", b"\x80\xfc\x10a", b"\x80\xfc\x10a"),
    ("S-a", b"A", b"<S-a>", b"A", b"A"),
    ("C-S-a", b"\x80\xfc\x02\x01", b"<C-S-a>", b"\x80\xfc\x02\x01", b"\x80\xfc\x02\x01"),
    ("M-C-a", b"\x80\xfc\x08\x01", b"<M-C-a>", b"\x80\xfc\x08\x01", b"\x80\xfc\x08\x01"),
    ("C-M-S-a", b"\x80\xfc\x0a\x01", b"<C-M-S-a>", b"\x80\xfc\x0a\x01", b"\x80\xfc\x0a\x01"),
    ("S-Left", b"\x80#4", b"<S-Left>", b"\x80#4", b"\x80#4"),
    ("C-Left", b"\x80\xfdU", b"<C-Left>", b"\x80\xfdU", b"\x80\xfdU"),
    ("M-Left", b"\x80\xfc\x08\x80kl", b"<M-Left>", b"\x80\xfc\x08\x80kl", b"\x80\xfc\x08\x80kl"),
    ("S-Tab", b"\x80kB", b"<S-Tab>", b"\x80kB", b"\x80kB"),
    ("C-Tab", b"\x80\xfc\x04\x09", b"<C-Tab>", b"\x80\xfc\x04\x09", b"\x80\xfc\x04\x09"),
    ("S-Del", b"\x80*4", b"<S-Del>", b"\x80*4", b"\x80*4"),
    ("LeftMouse", b"\x80\xfd,", b"<LeftMouse>", b"\x80\xfd,", b"\x80\xfd,"),
    ("LeftDrag", b"\x80\xfd-", b"<LeftDrag>", b"\x80\xfd-", b"\x80\xfd-"),
    ("LeftRelease", b"\x80\xfd.", b"<LeftRelease>", b"\x80\xfd.", b"\x80\xfd."),
    ("RightMouse", b"\x80\xfd2", b"<RightMouse>", b"\x80\xfd2", b"\x80\xfd2"),
    ("MiddleMouse", b"\x80\xfd/", b"<MiddleMouse>", b"\x80\xfd/", b"\x80\xfd/"),
    ("2-LeftMouse", b"\x80\xfc \x80\xfd,", b"<2-LeftMouse>", b"\x80\xfc \x80\xfd,", b"\x80\xfc \x80\xfd,"),
    ("3-LeftMouse", b"\x80\xfc@\x80\xfd,", b"<3-LeftMouse>", b"\x80\xfc@\x80\xfd,", b"\x80\xfc@\x80\xfd,"),
    ("4-LeftMouse", b"\x80\xfc`\x80\xfd,", b"<4-LeftMouse>", b"\x80\xfc`\x80\xfd,", b"\x80\xfc`\x80\xfd,"),
    ("C-LeftMouse", b"\x80\xfc\x04\x80\xfd,", b"<C-LeftMouse>", b"\x80\xfc\x04\x80\xfd,", b"\x80\xfc\x04\x80\xfd,"),
    ("S-LeftMouse", b"\x80\xfc\x02\x80\xfd,", b"<S-LeftMouse>", b"\x80\xfc\x02\x80\xfd,", b"\x80\xfc\x02\x80\xfd,"),
    ("ScrollWheelUp", b"\x80\xfdK", b"<ScrollWheelUp>", b"\x80\xfdK", b"\x80\xfdK"),
    ("ScrollWheelDown", b"\x80\xfdL", b"<ScrollWheelDown>", b"\x80\xfdL", b"\x80\xfdL"),
    ("ScrollWheelLeft", b"\x80\xfdN", b"<ScrollWheelLeft>", b"\x80\xfdN", b"\x80\xfdN"),
    ("MouseMove", b"\x80\xfdd", b"<MouseMove>", b"\x80\xfdd", b"\x80\xfdd"),
    ("X1Mouse", b"\x80\xfdY", b"<X1Mouse>", b"\x80\xfdY", b"\x80\xfdY"),
    ("X2Mouse", b"\x80\xfd\\", b"<X2Mouse>", b"\x80\xfd\\", b"\x80\xfd\\"),
    ("Plug", b"\x80\xfdS", b"<Plug>", b"\x80\xfdS", b"\x80\xfdS"),
    ("SNR", b"\x80\xfdR", b"<SNR>", b"\x80\xfdR", b"\x80\xfdR"),
    ("Cmd", b"\x80\xfdh", b"<Cmd>", b"\x80\xfdh", b"\x80\xfdh"),
    ("ScriptCmd", b"<ScriptCmd>", b"<ScriptCmd>", b"<ScriptCmd>", b"<ScriptCmd>"),
    ("CSI", b"\xc2\x9b", b"<CSI>", b"\xc2\x9b", b"\xc2\x9b"),
    ("xCSI", b"<xCSI>", b"<xCSI>", b"<xCSI>", b"<xCSI>"),
    ("FocusGained", b"<FocusGained>", b"<FocusGained>", b"<FocusGained>", b"<FocusGained>"),
    ("FocusLost", b"<FocusLost>", b"<FocusLost>", b"<FocusLost>", b"<FocusLost>"),
    ("Paste", b"<Paste>", b"<Paste>", b"<Paste>", b"<Paste>"),
    ("PasteStart", b"<PasteStart>", b"<PasteStart>", b"<PasteStart>", b"<PasteStart>"),
    ("PasteEnd", b"<PasteEnd>", b"<PasteEnd>", b"<PasteEnd>", b"<PasteEnd>"),
    ("char-97", b"a", b"<char-97>", b"a", b"a"),
    ("char-0x41", b"A", b"<char-0x41>", b"A", b"A"),
    ("Char-233", b"\xc3\xa9", b"<Char-233>", b"\xc3\xa9", b"\xc3\xa9"),
    ("EOL", b"<EOL>", b"<EOL>", b"<EOL>", b"<EOL>"),
    ("NotAKey", b"<NotAKey>", b"<NotAKey>", b"<NotAKey>", b"<NotAKey>"),
    ("C-", b"<C->", b"<C->", b"<C->", b"<C->"),
    ("", b"<>", b"<>", b"<>", b"<>"),
];

/// `(name, get_special_key_code(name), get_special_key_name(code, 0))`, one row
/// per distinct name in `key_names_table`. Two entries share the name `Tab`
/// (`TAB` and `K_TAB`); only the first is reachable by name, so it appears once.
const KEY_CODES: &[(&str, c_int, &str)] = &[
    ("k0", -17227, "<k0>"),
    ("F1", -12651, "<F1>"),
    ("k1", -17483, "<k1>"),
    ("F2", -12907, "<F2>"),
    ("k2", -17739, "<k2>"),
    ("F3", -13163, "<F3>"),
    ("k3", -17995, "<k3>"),
    ("F4", -13419, "<F4>"),
    ("k4", -18251, "<k4>"),
    ("F5", -13675, "<F5>"),
    ("k5", -18507, "<k5>"),
    ("F6", -13931, "<F6>"),
    ("k6", -18763, "<k6>"),
    ("F7", -14187, "<F7>"),
    ("k7", -19019, "<k7>"),
    ("F8", -14443, "<F8>"),
    ("k8", -19275, "<k8>"),
    ("F9", -14699, "<F9>"),
    ("k9", -19531, "<k9>"),
    ("LF", 10, "<NL>"),
    ("NL", 10, "<NL>"),
    ("Up", -30059, "<Up>"),
    ("CR", 13, "<CR>"),
    ("BS", -25195, "<BS>"),
    ("lt", 60, "<lt>"),
    ("F10", -15211, "<F10>"),
    ("F20", -16710, "<F20>"),
    ("F30", -19270, "<F30>"),
    ("F40", -21830, "<F40>"),
    ("F50", -25926, "<F50>"),
    ("F60", -28486, "<F60>"),
    ("KP0", -20477, "<kInsert>"),
    ("F11", -12614, "<F11>"),
    ("F21", -16966, "<F21>"),
    ("F31", -19526, "<F31>"),
    ("F41", -22086, "<F41>"),
    ("F51", -26182, "<F51>"),
    ("F61", -28742, "<F61>"),
    ("KP1", -13387, "<kEnd>"),
    ("xF1", -14845, "<xF1>"),
    ("F12", -12870, "<F12>"),
    ("F22", -17222, "<F22>"),
    ("F32", -19782, "<F32>"),
    ("F42", -22342, "<F42>"),
    ("F52", -26438, "<F52>"),
    ("F62", -28998, "<F62>"),
    ("KP2", -25675, "<kDown>"),
    ("xF2", -15101, "<xF2>"),
    ("F13", -13126, "<F13>"),
    ("F23", -17478, "<F23>"),
    ("F33", -20038, "<F33>"),
    ("F43", -22598, "<F43>"),
    ("F53", -26694, "<F53>"),
    ("F63", -29254, "<F63>"),
    ("KP3", -13643, "<kPageDown>"),
    ("xF3", -15357, "<xF3>"),
    ("F14", -13382, "<F14>"),
    ("F24", -17734, "<F24>"),
    ("F34", -20294, "<F34>"),
    ("F44", -22854, "<F44>"),
    ("F54", -26950, "<F54>"),
    ("KP4", -27723, "<kLeft>"),
    ("xF4", -15613, "<xF4>"),
    ("F15", -13638, "<F15>"),
    ("F25", -17990, "<F25>"),
    ("F35", -20550, "<F35>"),
    ("F45", -23110, "<F45>"),
    ("F55", -27206, "<F55>"),
    ("KP5", -12875, "<kOrigin>"),
    ("F16", -13894, "<F16>"),
    ("F26", -18246, "<F26>"),
    ("F36", -20806, "<F36>"),
    ("F46", -24902, "<F46>"),
    ("F56", -27462, "<F56>"),
    ("KP6", -29259, "<kRight>"),
    ("F17", -14150, "<F17>"),
    ("F27", -18502, "<F27>"),
    ("F37", -21062, "<F37>"),
    ("F47", -25158, "<F47>"),
    ("F57", -27718, "<F57>"),
    ("KP7", -12619, "<kHome>"),
    ("F18", -14406, "<F18>"),
    ("F28", -18758, "<F28>"),
    ("F38", -21318, "<F38>"),
    ("F48", -25414, "<F48>"),
    ("F58", -27974, "<F58>"),
    ("KP8", -30027, "<kUp>"),
    ("F19", -14662, "<F19>"),
    ("F29", -19014, "<F29>"),
    ("F39", -21574, "<F39>"),
    ("F49", -25670, "<F49>"),
    ("F59", -28230, "<F59>"),
    ("KP9", -13131, "<kPageUp>"),
    ("Tab", 9, "<Tab>"),
    ("Esc", 27, "<Esc>"),
    ("Cmd", -26877, "<Cmd>"),
    ("End", -14144, "<End>"),
    ("CSI", 155, "<CSI>"),
    ("Del", -17515, "<Del>"),
    ("Nul", -22783, "<Nul>"),
    ("kUp", -30027, "<kUp>"),
    ("xUp", -16893, "<xUp>"),
    ("Bar", 124, "<Bar>"),
    ("SNR", -21245, "<SNR>"),
    ("Ins", -18795, "<Insert>"),
    ("Down", -25707, "<Down>"),
    ("Drop", -24573, "<Drop>"),
    ("Find", -12352, "<Find>"),
    ("Help", -12581, "<Help>"),
    ("Home", -26731, "<Home>"),
    ("kDel", -20733, "<kDel>"),
    ("kEnd", -13387, "<kEnd>"),
    ("Left", -27755, "<Left>"),
    ("Plug", -21501, "<Plug>"),
    ("Undo", -14374, "<Undo>"),
    ("xEnd", -15869, "<xEnd>"),
    ("zEnd", -16125, "<zEnd>"),
    ("kDown", -25675, "<kDown>"),
    ("xDown", -17149, "<xDown>"),
    ("kHome", -12619, "<kHome>"),
    ("xHome", -16381, "<xHome>"),
    ("zHome", -16637, "<zHome>"),
    ("Right", -29291, "<Right>"),
    ("kLeft", -27723, "<kLeft>"),
    ("xLeft", -17405, "<xLeft>"),
    ("Enter", 13, "<CR>"),
    ("Mouse", -22779, "<Mouse>"),
    ("KPDiv", -14411, "<kDivide>"),
    ("kPlus", -13899, "<kPlus>"),
    ("Space", 32, "<Space>"),
    ("Escape", 27, "<Esc>"),
    ("X1Drag", -23293, "<X1Drag>"),
    ("X2Drag", -24061, "<X2Drag>"),
    ("PageUp", -20587, "<PageUp>"),
    ("kMinus", -14155, "<kMinus>"),
    ("kRight", -29259, "<kRight>"),
    ("xRight", -17661, "<xRight>"),
    ("Bslash", 92, "<Bslash>"),
    ("Delete", -17515, "<Del>"),
    ("Select", -13866, "<Select>"),
    ("KPMult", -14667, "<kMultiply>"),
    ("Ignore", -13821, "<Ignore>"),
    ("kEnter", -16715, "<kEnter>"),
    ("kComma", -19787, "<kComma>"),
    ("kPoint", -16971, "<kPoint>"),
    ("KPPlus", -13899, "<kPlus>"),
    ("kEqual", -20043, "<kEqual>"),
    ("Insert", -18795, "<Insert>"),
    ("Return", 13, "<CR>"),
    ("kPageUp", -13131, "<kPageUp>"),
    ("KPComma", -19787, "<kComma>"),
    ("KPEnter", -16715, "<kEnter>"),
    ("kDivide", -14411, "<kDivide>"),
    ("KPMinus", -14155, "<kMinus>"),
    ("X1Mouse", -23037, "<X1Mouse>"),
    ("X2Mouse", -23805, "<X2Mouse>"),
    ("kInsert", -20477, "<kInsert>"),
    ("kOrigin", -12875, "<kOrigin>"),
    ("MouseUp", -19709, "<ScrollWheelDown>"),
    ("NewLine", 10, "<NL>"),
    ("KPEquals", -20043, "<kEqual>"),
    ("LeftDrag", -11773, "<LeftDrag>"),
    ("PageDown", -20075, "<PageDown>"),
    ("LineFeed", 10, "<NL>"),
    ("KPPeriod", -20733, "<kDel>"),
    ("BackSpace", -25195, "<BS>"),
    ("kMultiply", -14667, "<kMultiply>"),
    ("kPageDown", -13643, "<kPageDown>"),
    ("LeftMouse", -11517, "<LeftMouse>"),
    ("MouseDown", -19453, "<ScrollWheelUp>"),
    ("MouseMove", -25853, "<MouseMove>"),
    ("RightDrag", -13309, "<RightDrag>"),
    ("X1Release", -23549, "<X1Release>"),
    ("X2Release", -24317, "<X2Release>"),
    ("MiddleDrag", -12541, "<MiddleDrag>"),
    ("RightMouse", -13053, "<RightMouse>"),
    ("MiddleMouse", -12285, "<MiddleMouse>"),
    ("LeftMouseNM", -17917, "<LeftMouseNM>"),
    ("LeftRelease", -12029, "<LeftRelease>"),
    ("RightRelease", -13565, "<RightRelease>"),
    ("LeftReleaseNM", -18173, "<LeftReleaseNM>"),
    ("MiddleRelease", -12797, "<MiddleRelease>"),
    ("ScrollWheelUp", -19453, "<ScrollWheelUp>"),
    ("ScrollWheelDown", -19709, "<ScrollWheelDown>"),
    ("ScrollWheelLeft", -20221, "<ScrollWheelLeft>"),
    ("ScrollWheelRight", -19965, "<ScrollWheelRight>"),
];

/// `replace_termcodes` over an owned buffer, as `nvim_replace_termcodes` calls
/// it: `bufp` starts NULL, so the result is allocated and must be freed.
fn termcodes(src: &str, from_part: bool, do_lt: bool, special: bool) -> Vec<u8> {
    let mut flags: c_int = 0;
    if from_part {
        flags |= REPTERM_FROM_PART as c_int;
    }
    if do_lt {
        flags |= REPTERM_DO_LT as c_int;
    }
    if !special {
        flags |= REPTERM_NO_SPECIAL as c_int;
    }
    let s = cstr(src);
    let mut buf: *mut c_char = ptr::null_mut();
    unsafe {
        replace_termcodes(
            s.as_ptr(),
            src.len(),
            &mut buf,
            0,
            flags,
            ptr::null_mut(),
            CPO.as_ptr(),
        );
        take_bytes(buf)
    }
}

/// The code `name` resolves to, and the name that code prints as.
fn key_code(name: &str) -> c_int {
    let s = cstr(name);
    unsafe { get_special_key_code(s.as_ptr()) }
}

fn key_name(code: c_int, modifiers: c_int) -> String {
    unsafe {
        CStr::from_ptr(get_special_key_name(code, modifiers))
            .to_string_lossy()
            .into_owned()
    }
}

/// Wrapper over `find_special_key`: returns the key code and the modifier
/// mask it reported.
fn special_key(src: &str, flags: c_int) -> (c_int, c_int) {
    let s = cstr(src);
    let mut srcp: *const c_char = s.as_ptr();
    let mut modifiers: c_int = 0;
    let key =
        unsafe { find_special_key(&mut srcp, src.len(), &mut modifiers, flags, ptr::null_mut()) };
    (key, modifiers)
}

#[test]
fn find_special_key_no_keycode() {
    let (key, _) = special_key("abc", 0);
    assert_eq!(0, key);
}

#[test]
fn find_special_key_keycode_with_multiple_modifiers() {
    let (key, modifiers) = special_key("<C-M-S-A>", 0);
    assert_ne!(0, key);
    assert_ne!(0, modifiers);
}

#[test]
fn find_special_key_is_case_insensitive() {
    // Compare other capitalizations to this.
    let (all_caps_key, all_caps_mod) = special_key("<C-A>", 0);
    assert_eq!((all_caps_key, all_caps_mod), special_key("<C-a>", 0));
    assert_eq!((all_caps_key, all_caps_mod), special_key("<c-A>", 0));
    assert_eq!((all_caps_key, all_caps_mod), special_key("<c-a>", 0));
}

#[test]
fn find_special_key_double_quote_in_keycode() {
    let in_string = FSK_IN_STRING as c_int;

    // Unescaped with in_string=false
    assert_eq!('"' as c_int, special_key("<C-\">", 0).0);

    // Unescaped with in_string=true
    assert_eq!(0, special_key("<C-\">", in_string).0);

    // Escaped with in_string=false: should fail because the key is invalid
    // (more than 1 non-modifier character).
    assert_eq!(0, special_key("<C-\\\">", 0).0);

    // Escaped with in_string=true
    assert_eq!('"' as c_int, special_key("<C-\\\">", in_string).0);
}

/// Every `<name>` keysweep records, through all four encodings.
#[test]
fn replace_termcodes_matches_the_sweep_baseline() {
    for &(name, tc, no_special, no_lt, no_from_part) in TERMCODES {
        let spelled = format!("<{name}>");
        assert_eq!(
            tc,
            termcodes(&spelled, true, true, true),
            "<{name}> termcodes"
        );
        assert_eq!(
            no_special,
            termcodes(&spelled, true, true, false),
            "<{name}> no_special"
        );
        assert_eq!(
            no_lt,
            termcodes(&spelled, true, false, true),
            "<{name}> no_lt"
        );
        assert_eq!(
            no_from_part,
            termcodes(&spelled, false, true, true),
            "<{name}> no_from_part"
        );
    }
}

/// Every name in the table resolves to its code, and every code prints as the
/// preferred name for that code — followed by the modifier prefixes and the
/// three fallbacks for a code the table has no name for.
///
/// Every `get_special_key_name` case lives in this one test on purpose: it
/// answers out of a single shared static buffer, so two tests calling it run
/// into each other under the harness's default thread-per-test.
#[test]
fn key_names_table_round_trips() {
    for &(name, code, printed) in KEY_CODES {
        assert_eq!(code, key_code(name), "code of {name}");
        assert_eq!(printed, key_name(code, 0), "name of {name}");
    }

    // The modifier letters, in the order the modifier table lists them.
    let up = key_code("Up");
    assert_eq!("<S-Up>", key_name(up, MOD_MASK_SHIFT));
    assert_eq!("<C-Up>", key_name(up, MOD_MASK_CTRL));
    assert_eq!("<M-Up>", key_name(up, MOD_MASK_ALT));
    assert_eq!("<D-Up>", key_name(up, MOD_MASK_CMD));
    assert_eq!("<T-Up>", key_name(up, MOD_MASK_META));
    assert_eq!(
        "<M-C-S-Up>",
        key_name(up, MOD_MASK_ALT | MOD_MASK_CTRL | MOD_MASK_SHIFT)
    );
    // An unnamed special key prints as t_xx.
    assert_eq!("<t_zz>", key_name(key_code("t_zz"), 0));
    // A printable character prints as itself; a control one gains <C-.
    assert_eq!("<a>", key_name('a' as c_int, 0));
    assert_eq!("<C-A>", key_name(1, 0));
    // Above 0x7f it is a codepoint, not an Alt-ed byte: the un-alting branch
    // asks for `utf_char2len(c) == 1 && (c & 0x80)`, which nothing satisfies.
    assert_eq!("<\u{e1}>", key_name(0xe1, 0));
    // Modifiers on a plain character are still spelled out.
    assert_eq!("<M-a>", key_name('a' as c_int, MOD_MASK_ALT));
}

/// The name lookup folds ASCII case, both ways, for every name in the table.
#[test]
fn key_names_table_is_case_insensitive() {
    for &(name, code, _) in KEY_CODES {
        assert_eq!(code, key_code(&name.to_ascii_uppercase()), "upper {name}");
        assert_eq!(code, key_code(&name.to_ascii_lowercase()), "lower {name}");
    }
}

/// The name is delimited by the first non-identifier byte, not by the NUL, and
/// a `t_xx` name bypasses the table for a raw termcap code.
#[test]
fn get_special_key_code_delimits_and_takes_termcaps() {
    assert_eq!(key_code("Esc"), key_code("Esc>rest"));
    assert_eq!(key_code("Esc"), key_code("Esc-"));
    assert_eq!(0, key_code("Escape2"));
    assert_eq!(0, key_code(""));
    assert_eq!(0, key_code("NotAKey"));
    // t_xx is TERMCAP2KEY(x, x), whatever the two bytes are.
    assert_eq!(key_code("Up"), key_code("t_ku"));
    assert_eq!(key_code("F1"), key_code("t_k1"));
    assert_eq!(-(b'z' as c_int + ((b'z' as c_int) << 8)), key_code("t_zz"));
    // Too short to be a termcap name: falls through to the table and misses.
    assert_eq!(0, key_code("t_k"));
}
