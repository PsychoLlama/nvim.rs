//! The capability tables the terminfo layer indexes everything by.
//!
//! [`TerminfoEntry`](crate::types::TerminfoEntry) stores its
//! capabilities in three dense arrays, so every producer and consumer of one
//! has to agree on which slot means what. This file is that agreement, and
//! nothing else should spell a slot number out:
//!
//! - `defs` is indexed by the `kTerm_*` constants, in the order of
//!   [`STRING_CAPS`] followed by [`EXT_CAPS`].
//! - `keys` is indexed by [`key_slot`], in the order of [`KEYS`].
//! - `f_keys` holds `key_fN` at index `N - 1`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_uint};

/// unibilium's capability numbers. Nested so they stay out of the flat
/// namespace the unit-test cdefs are generated into -- they are an
/// implementation detail of the tables below.
mod unibi_cap {
    pub const BACK_COLOR_ERASE: super::UnibiCap = 29;
    pub const MAX_COLORS: super::UnibiCap = 59;
    pub const LINES: super::UnibiCap = 48;
    pub const COLUMNS: super::UnibiCap = 46;

    pub(super) const CARRIAGE_RETURN: super::UnibiString = 88;
    pub(super) const CHANGE_SCROLL_REGION: super::UnibiString = 89;
    pub(super) const CLEAR_SCREEN: super::UnibiString = 91;
    pub(super) const CLR_EOL: super::UnibiString = 92;
    pub(super) const CLR_EOS: super::UnibiString = 93;
    pub(super) const CURSOR_ADDRESS: super::UnibiString = 96;
    pub(super) const CURSOR_DOWN: super::UnibiString = 97;
    pub(super) const CURSOR_INVISIBLE: super::UnibiString = 99;
    pub(super) const CURSOR_LEFT: super::UnibiString = 100;
    pub(super) const CURSOR_HOME: super::UnibiString = 98;
    pub(super) const CURSOR_NORMAL: super::UnibiString = 102;
    pub(super) const CURSOR_UP: super::UnibiString = 105;
    pub(super) const CURSOR_RIGHT: super::UnibiString = 103;
    pub(super) const DELETE_LINE: super::UnibiString = 108;
    pub(super) const ENTER_BLINK_MODE: super::UnibiString = 112;
    pub(super) const ENTER_BOLD_MODE: super::UnibiString = 113;
    pub(super) const ENTER_CA_MODE: super::UnibiString = 114;
    pub(super) const ENTER_DIM_MODE: super::UnibiString = 116;
    pub(super) const ENTER_ITALICS_MODE: super::UnibiString = 397;
    pub(super) const ENTER_REVERSE_MODE: super::UnibiString = 120;
    pub(super) const ENTER_SECURE_MODE: super::UnibiString = 118;
    pub(super) const ENTER_STANDOUT_MODE: super::UnibiString = 121;
    pub(super) const ENTER_UNDERLINE_MODE: super::UnibiString = 122;
    pub(super) const ERASE_CHARS: super::UnibiString = 123;
    pub(super) const EXIT_ATTRIBUTE_MODE: super::UnibiString = 125;
    pub(super) const EXIT_CA_MODE: super::UnibiString = 126;
    pub(super) const FROM_STATUS_LINE: super::UnibiString = 133;
    pub(super) const INSERT_LINE: super::UnibiString = 139;
    pub(super) const KEYPAD_LOCAL: super::UnibiString = 174;
    pub(super) const KEYPAD_XMIT: super::UnibiString = 175;
    pub(super) const PARM_DELETE_LINE: super::UnibiString = 192;
    pub(super) const PARM_DOWN_CURSOR: super::UnibiString = 193;
    pub(super) const PARM_INSERT_LINE: super::UnibiString = 196;
    pub(super) const PARM_LEFT_CURSOR: super::UnibiString = 197;
    pub(super) const PARM_RIGHT_CURSOR: super::UnibiString = 198;
    pub(super) const PARM_UP_CURSOR: super::UnibiString = 200;
    pub(super) const SET_A_BACKGROUND: super::UnibiString = 446;
    pub(super) const SET_A_FOREGROUND: super::UnibiString = 445;
    pub(super) const SET_ATTRIBUTES: super::UnibiString = 217;
    pub(super) const SET_LR_MARGIN: super::UnibiString = 454;
    pub(super) const TO_STATUS_LINE: super::UnibiString = 221;

    pub(super) const KEY_BACKSPACE: super::UnibiString = 141;
    pub(super) const KEY_BEG: super::UnibiString = 244;
    pub(super) const KEY_SBEG: super::UnibiString = 272;
    pub(super) const KEY_BTAB: super::UnibiString = 234;
    pub(super) const KEY_CLEAR: super::UnibiString = 143;
    pub(super) const KEY_DC: super::UnibiString = 145;
    pub(super) const KEY_SDC: super::UnibiString = 277;
    pub(super) const KEY_END: super::UnibiString = 250;
    pub(super) const KEY_SEND: super::UnibiString = 280;
    pub(super) const KEY_FIND: super::UnibiString = 253;
    pub(super) const KEY_SFIND: super::UnibiString = 283;
    pub(super) const KEY_HOME: super::UnibiString = 162;
    pub(super) const KEY_SHOME: super::UnibiString = 285;
    pub(super) const KEY_IC: super::UnibiString = 163;
    pub(super) const KEY_SIC: super::UnibiString = 286;
    pub(super) const KEY_NPAGE: super::UnibiString = 167;
    pub(super) const KEY_PPAGE: super::UnibiString = 168;
    pub(super) const KEY_SELECT: super::UnibiString = 279;
    pub(super) const KEY_SUSPEND: super::UnibiString = 270;
    pub(super) const KEY_SSUSPEND: super::UnibiString = 299;
    pub(super) const KEY_UNDO: super::UnibiString = 271;
    pub(super) const KEY_SUNDO: super::UnibiString = 300;
    pub(super) const KEY_LEFT: super::UnibiString = 165;
    pub(super) const KEY_SLEFT: super::UnibiString = 287;
    pub(super) const KEY_RIGHT: super::UnibiString = 169;
    pub(super) const KEY_SRIGHT: super::UnibiString = 296;

    pub(super) const KEY_F1: super::UnibiString = 152;
    pub(super) const KEY_F2: super::UnibiString = 154;
    pub(super) const KEY_F3: super::UnibiString = 155;
    pub(super) const KEY_F4: super::UnibiString = 156;
    pub(super) const KEY_F5: super::UnibiString = 157;
    pub(super) const KEY_F6: super::UnibiString = 158;
    pub(super) const KEY_F7: super::UnibiString = 159;
    pub(super) const KEY_F8: super::UnibiString = 160;
    pub(super) const KEY_F9: super::UnibiString = 161;
    pub(super) const KEY_F10: super::UnibiString = 153;
    pub(super) const KEY_F11: super::UnibiString = 302;
    pub(super) const KEY_F12: super::UnibiString = 303;
    pub(super) const KEY_F13: super::UnibiString = 304;
    pub(super) const KEY_F14: super::UnibiString = 305;
    pub(super) const KEY_F15: super::UnibiString = 306;
    pub(super) const KEY_F16: super::UnibiString = 307;
    pub(super) const KEY_F17: super::UnibiString = 308;
    pub(super) const KEY_F18: super::UnibiString = 309;
    pub(super) const KEY_F19: super::UnibiString = 310;
    pub(super) const KEY_F20: super::UnibiString = 311;
    pub(super) const KEY_F21: super::UnibiString = 312;
    pub(super) const KEY_F22: super::UnibiString = 313;
    pub(super) const KEY_F23: super::UnibiString = 314;
    pub(super) const KEY_F24: super::UnibiString = 315;
    pub(super) const KEY_F25: super::UnibiString = 316;
    pub(super) const KEY_F26: super::UnibiString = 317;
    pub(super) const KEY_F27: super::UnibiString = 318;
    pub(super) const KEY_F28: super::UnibiString = 319;
    pub(super) const KEY_F29: super::UnibiString = 320;
    pub(super) const KEY_F30: super::UnibiString = 321;
    pub(super) const KEY_F31: super::UnibiString = 322;
    pub(super) const KEY_F32: super::UnibiString = 323;
    pub(super) const KEY_F33: super::UnibiString = 324;
    pub(super) const KEY_F34: super::UnibiString = 325;
    pub(super) const KEY_F35: super::UnibiString = 326;
    pub(super) const KEY_F36: super::UnibiString = 327;
    pub(super) const KEY_F37: super::UnibiString = 328;
    pub(super) const KEY_F38: super::UnibiString = 329;
    pub(super) const KEY_F39: super::UnibiString = 330;
    pub(super) const KEY_F40: super::UnibiString = 331;
    pub(super) const KEY_F41: super::UnibiString = 332;
    pub(super) const KEY_F42: super::UnibiString = 333;
    pub(super) const KEY_F43: super::UnibiString = 334;
    pub(super) const KEY_F44: super::UnibiString = 335;
    pub(super) const KEY_F45: super::UnibiString = 336;
    pub(super) const KEY_F46: super::UnibiString = 337;
    pub(super) const KEY_F47: super::UnibiString = 338;
    pub(super) const KEY_F48: super::UnibiString = 339;
    pub(super) const KEY_F49: super::UnibiString = 340;
    pub(super) const KEY_F50: super::UnibiString = 341;
    pub(super) const KEY_F51: super::UnibiString = 342;
    pub(super) const KEY_F52: super::UnibiString = 343;
    pub(super) const KEY_F53: super::UnibiString = 344;
    pub(super) const KEY_F54: super::UnibiString = 345;
    pub(super) const KEY_F55: super::UnibiString = 346;
    pub(super) const KEY_F56: super::UnibiString = 347;
    pub(super) const KEY_F57: super::UnibiString = 348;
    pub(super) const KEY_F58: super::UnibiString = 349;
    pub(super) const KEY_F59: super::UnibiString = 350;
    pub(super) const KEY_F60: super::UnibiString = 351;
    pub(super) const KEY_F61: super::UnibiString = 352;
    pub(super) const KEY_F62: super::UnibiString = 353;
    pub(super) const KEY_F63: super::UnibiString = 354;
}

/// A unibilium capability number, whatever kind of capability it names.
pub type UnibiCap = c_uint;
/// A unibilium *string* capability number.
pub type UnibiString = UnibiCap;

pub use unibi_cap::{BACK_COLOR_ERASE, COLUMNS, LINES, MAX_COLORS};

/// A slot in `TerminfoEntry::defs`.
pub type TerminfoDef = c_uint;

pub const kTerm_carriage_return: TerminfoDef = 0;
pub const kTerm_change_scroll_region: TerminfoDef = 1;
pub const kTerm_clear_screen: TerminfoDef = 2;
pub const kTerm_clr_eol: TerminfoDef = 3;
pub const kTerm_clr_eos: TerminfoDef = 4;
pub const kTerm_cursor_address: TerminfoDef = 5;
pub const kTerm_cursor_down: TerminfoDef = 6;
pub const kTerm_cursor_invisible: TerminfoDef = 7;
pub const kTerm_cursor_left: TerminfoDef = 8;
pub const kTerm_cursor_home: TerminfoDef = 9;
pub const kTerm_cursor_normal: TerminfoDef = 10;
pub const kTerm_cursor_up: TerminfoDef = 11;
pub const kTerm_cursor_right: TerminfoDef = 12;
pub const kTerm_delete_line: TerminfoDef = 13;
pub const kTerm_enter_blink_mode: TerminfoDef = 14;
pub const kTerm_enter_bold_mode: TerminfoDef = 15;
pub const kTerm_enter_ca_mode: TerminfoDef = 16;
pub const kTerm_enter_dim_mode: TerminfoDef = 17;
pub const kTerm_enter_italics_mode: TerminfoDef = 18;
pub const kTerm_enter_reverse_mode: TerminfoDef = 19;
pub const kTerm_enter_secure_mode: TerminfoDef = 20;
pub const kTerm_enter_standout_mode: TerminfoDef = 21;
pub const kTerm_enter_underline_mode: TerminfoDef = 22;
pub const kTerm_erase_chars: TerminfoDef = 23;
pub const kTerm_exit_attribute_mode: TerminfoDef = 24;
pub const kTerm_exit_ca_mode: TerminfoDef = 25;
pub const kTerm_from_status_line: TerminfoDef = 26;
pub const kTerm_insert_line: TerminfoDef = 27;
pub const kTerm_keypad_local: TerminfoDef = 28;
pub const kTerm_keypad_xmit: TerminfoDef = 29;
pub const kTerm_parm_delete_line: TerminfoDef = 30;
pub const kTerm_parm_down_cursor: TerminfoDef = 31;
pub const kTerm_parm_insert_line: TerminfoDef = 32;
pub const kTerm_parm_left_cursor: TerminfoDef = 33;
pub const kTerm_parm_right_cursor: TerminfoDef = 34;
pub const kTerm_parm_up_cursor: TerminfoDef = 35;
pub const kTerm_set_a_background: TerminfoDef = 36;
pub const kTerm_set_a_foreground: TerminfoDef = 37;
pub const kTerm_set_attributes: TerminfoDef = 38;
pub const kTerm_set_lr_margin: TerminfoDef = 39;
pub const kTerm_to_status_line: TerminfoDef = 40;
pub const kTerm_reset_cursor_style: TerminfoDef = 41;
pub const kTerm_set_cursor_style: TerminfoDef = 42;
pub const kTerm_enter_strikethrough_mode: TerminfoDef = 43;
pub const kTerm_set_rgb_foreground: TerminfoDef = 44;
pub const kTerm_set_rgb_background: TerminfoDef = 45;
pub const kTerm_set_cursor_color: TerminfoDef = 46;
pub const kTerm_reset_cursor_color: TerminfoDef = 47;
pub const kTerm_set_underline_style: TerminfoDef = 48;
/// One past the last `defs` slot.
pub const kTermCount: TerminfoDef = 49;

/// A string capability terminfo names in its standard set.
pub struct StringCap {
    /// Its terminfo long name, as `:verbose` terminal dumps print it.
    pub name: &'static str,
    /// unibilium's number for it.
    pub cap: UnibiString,
}

/// The standard string capabilities, in `defs` slot order.
pub const STRING_CAPS: [StringCap; 41] = [
    StringCap {
        name: "carriage_return",
        cap: unibi_cap::CARRIAGE_RETURN,
    },
    StringCap {
        name: "change_scroll_region",
        cap: unibi_cap::CHANGE_SCROLL_REGION,
    },
    StringCap {
        name: "clear_screen",
        cap: unibi_cap::CLEAR_SCREEN,
    },
    StringCap {
        name: "clr_eol",
        cap: unibi_cap::CLR_EOL,
    },
    StringCap {
        name: "clr_eos",
        cap: unibi_cap::CLR_EOS,
    },
    StringCap {
        name: "cursor_address",
        cap: unibi_cap::CURSOR_ADDRESS,
    },
    StringCap {
        name: "cursor_down",
        cap: unibi_cap::CURSOR_DOWN,
    },
    StringCap {
        name: "cursor_invisible",
        cap: unibi_cap::CURSOR_INVISIBLE,
    },
    StringCap {
        name: "cursor_left",
        cap: unibi_cap::CURSOR_LEFT,
    },
    StringCap {
        name: "cursor_home",
        cap: unibi_cap::CURSOR_HOME,
    },
    StringCap {
        name: "cursor_normal",
        cap: unibi_cap::CURSOR_NORMAL,
    },
    StringCap {
        name: "cursor_up",
        cap: unibi_cap::CURSOR_UP,
    },
    StringCap {
        name: "cursor_right",
        cap: unibi_cap::CURSOR_RIGHT,
    },
    StringCap {
        name: "delete_line",
        cap: unibi_cap::DELETE_LINE,
    },
    StringCap {
        name: "enter_blink_mode",
        cap: unibi_cap::ENTER_BLINK_MODE,
    },
    StringCap {
        name: "enter_bold_mode",
        cap: unibi_cap::ENTER_BOLD_MODE,
    },
    StringCap {
        name: "enter_ca_mode",
        cap: unibi_cap::ENTER_CA_MODE,
    },
    StringCap {
        name: "enter_dim_mode",
        cap: unibi_cap::ENTER_DIM_MODE,
    },
    StringCap {
        name: "enter_italics_mode",
        cap: unibi_cap::ENTER_ITALICS_MODE,
    },
    StringCap {
        name: "enter_reverse_mode",
        cap: unibi_cap::ENTER_REVERSE_MODE,
    },
    StringCap {
        name: "enter_secure_mode",
        cap: unibi_cap::ENTER_SECURE_MODE,
    },
    StringCap {
        name: "enter_standout_mode",
        cap: unibi_cap::ENTER_STANDOUT_MODE,
    },
    StringCap {
        name: "enter_underline_mode",
        cap: unibi_cap::ENTER_UNDERLINE_MODE,
    },
    StringCap {
        name: "erase_chars",
        cap: unibi_cap::ERASE_CHARS,
    },
    StringCap {
        name: "exit_attribute_mode",
        cap: unibi_cap::EXIT_ATTRIBUTE_MODE,
    },
    StringCap {
        name: "exit_ca_mode",
        cap: unibi_cap::EXIT_CA_MODE,
    },
    StringCap {
        name: "from_status_line",
        cap: unibi_cap::FROM_STATUS_LINE,
    },
    StringCap {
        name: "insert_line",
        cap: unibi_cap::INSERT_LINE,
    },
    StringCap {
        name: "keypad_local",
        cap: unibi_cap::KEYPAD_LOCAL,
    },
    StringCap {
        name: "keypad_xmit",
        cap: unibi_cap::KEYPAD_XMIT,
    },
    StringCap {
        name: "parm_delete_line",
        cap: unibi_cap::PARM_DELETE_LINE,
    },
    StringCap {
        name: "parm_down_cursor",
        cap: unibi_cap::PARM_DOWN_CURSOR,
    },
    StringCap {
        name: "parm_insert_line",
        cap: unibi_cap::PARM_INSERT_LINE,
    },
    StringCap {
        name: "parm_left_cursor",
        cap: unibi_cap::PARM_LEFT_CURSOR,
    },
    StringCap {
        name: "parm_right_cursor",
        cap: unibi_cap::PARM_RIGHT_CURSOR,
    },
    StringCap {
        name: "parm_up_cursor",
        cap: unibi_cap::PARM_UP_CURSOR,
    },
    StringCap {
        name: "set_a_background",
        cap: unibi_cap::SET_A_BACKGROUND,
    },
    StringCap {
        name: "set_a_foreground",
        cap: unibi_cap::SET_A_FOREGROUND,
    },
    StringCap {
        name: "set_attributes",
        cap: unibi_cap::SET_ATTRIBUTES,
    },
    StringCap {
        name: "set_lr_margin",
        cap: unibi_cap::SET_LR_MARGIN,
    },
    StringCap {
        name: "to_status_line",
        cap: unibi_cap::TO_STATUS_LINE,
    },
];

/// A string capability terminfo only knows as an extension, so it is looked
/// up by name rather than by number.
pub struct ExtCap {
    /// nvim's name for it.
    pub name: &'static str,
    /// The name a terminal description gives it.
    pub terminfo_name: &'static [u8],
}

/// The extension string capabilities, filling the `defs` slots from
/// `kTerm_reset_cursor_style` on.
pub const EXT_CAPS: [ExtCap; 8] = [
    ExtCap {
        name: "reset_cursor_style",
        terminfo_name: b"Se",
    },
    ExtCap {
        name: "set_cursor_style",
        terminfo_name: b"Ss",
    },
    ExtCap {
        name: "enter_strikethrough_mode",
        terminfo_name: b"smxx",
    },
    ExtCap {
        name: "set_rgb_foreground",
        terminfo_name: b"setrgbf",
    },
    ExtCap {
        name: "set_rgb_background",
        terminfo_name: b"setrgbb",
    },
    ExtCap {
        name: "set_cursor_color",
        terminfo_name: b"Cs",
    },
    ExtCap {
        name: "reset_cursor_color",
        terminfo_name: b"Cr",
    },
    ExtCap {
        name: "set_underline_style",
        terminfo_name: b"Smulx",
    },
];

/// A special key, paired with its shifted variant where terminfo describes
/// one. The shifted capability is only consulted when the unshifted one is
/// present.
pub struct KeyCap {
    /// The bare key name: the capability is `key_<stem>`, its shifted form
    /// `key_s<stem>`.
    pub stem: &'static str,
    /// `key_<stem>`, for the hook nvim gives the terminfo key driver.
    pub name: &'static CStr,
    /// `key_s<stem>`, likewise.
    pub shifted_name: &'static CStr,
    pub cap: UnibiString,
    /// `None` when terminfo has no shifted form of this key.
    pub shifted_cap: Option<UnibiString>,
}

/// The special keys, in `TerminfoEntry::keys` slot order.
pub const KEYS: [KeyCap; 16] = [
    KeyCap {
        stem: "backspace",
        name: c"key_backspace",
        shifted_name: c"key_sbackspace",
        cap: unibi_cap::KEY_BACKSPACE,
        shifted_cap: None,
    },
    KeyCap {
        stem: "beg",
        name: c"key_beg",
        shifted_name: c"key_sbeg",
        cap: unibi_cap::KEY_BEG,
        shifted_cap: Some(unibi_cap::KEY_SBEG),
    },
    KeyCap {
        stem: "btab",
        name: c"key_btab",
        shifted_name: c"key_sbtab",
        cap: unibi_cap::KEY_BTAB,
        shifted_cap: None,
    },
    KeyCap {
        stem: "clear",
        name: c"key_clear",
        shifted_name: c"key_sclear",
        cap: unibi_cap::KEY_CLEAR,
        shifted_cap: None,
    },
    KeyCap {
        stem: "dc",
        name: c"key_dc",
        shifted_name: c"key_sdc",
        cap: unibi_cap::KEY_DC,
        shifted_cap: Some(unibi_cap::KEY_SDC),
    },
    KeyCap {
        stem: "end",
        name: c"key_end",
        shifted_name: c"key_send",
        cap: unibi_cap::KEY_END,
        shifted_cap: Some(unibi_cap::KEY_SEND),
    },
    KeyCap {
        stem: "find",
        name: c"key_find",
        shifted_name: c"key_sfind",
        cap: unibi_cap::KEY_FIND,
        shifted_cap: Some(unibi_cap::KEY_SFIND),
    },
    KeyCap {
        stem: "home",
        name: c"key_home",
        shifted_name: c"key_shome",
        cap: unibi_cap::KEY_HOME,
        shifted_cap: Some(unibi_cap::KEY_SHOME),
    },
    KeyCap {
        stem: "ic",
        name: c"key_ic",
        shifted_name: c"key_sic",
        cap: unibi_cap::KEY_IC,
        shifted_cap: Some(unibi_cap::KEY_SIC),
    },
    KeyCap {
        stem: "npage",
        name: c"key_npage",
        shifted_name: c"key_snpage",
        cap: unibi_cap::KEY_NPAGE,
        shifted_cap: None,
    },
    KeyCap {
        stem: "ppage",
        name: c"key_ppage",
        shifted_name: c"key_sppage",
        cap: unibi_cap::KEY_PPAGE,
        shifted_cap: None,
    },
    KeyCap {
        stem: "select",
        name: c"key_select",
        shifted_name: c"key_sselect",
        cap: unibi_cap::KEY_SELECT,
        shifted_cap: None,
    },
    KeyCap {
        stem: "suspend",
        name: c"key_suspend",
        shifted_name: c"key_ssuspend",
        cap: unibi_cap::KEY_SUSPEND,
        shifted_cap: Some(unibi_cap::KEY_SSUSPEND),
    },
    KeyCap {
        stem: "undo",
        name: c"key_undo",
        shifted_name: c"key_sundo",
        cap: unibi_cap::KEY_UNDO,
        shifted_cap: Some(unibi_cap::KEY_SUNDO),
    },
    KeyCap {
        stem: "left",
        name: c"key_left",
        shifted_name: c"key_sleft",
        cap: unibi_cap::KEY_LEFT,
        shifted_cap: Some(unibi_cap::KEY_SLEFT),
    },
    KeyCap {
        stem: "right",
        name: c"key_right",
        shifted_name: c"key_sright",
        cap: unibi_cap::KEY_RIGHT,
        shifted_cap: Some(unibi_cap::KEY_SRIGHT),
    },
];

/// Names for the [`KEYS`] slots. Anything indexing `TerminfoEntry::keys`
/// spells the slot with one of these.
pub mod key_slot {
    pub const BACKSPACE: usize = 0;
    pub const BEG: usize = 1;
    pub const BTAB: usize = 2;
    pub const CLEAR: usize = 3;
    pub const DC: usize = 4;
    pub const END: usize = 5;
    pub const FIND: usize = 6;
    pub const HOME: usize = 7;
    pub const IC: usize = 8;
    pub const NPAGE: usize = 9;
    pub const PPAGE: usize = 10;
    pub const SELECT: usize = 11;
    pub const SUSPEND: usize = 12;
    pub const UNDO: usize = 13;
    pub const LEFT: usize = 14;
    pub const RIGHT: usize = 15;
}

/// The highest `key_fN` capability terminfo defines.
pub const MAX_FUNCTION_KEY: usize = 63;

/// `key_f1`..`key_f63`, in `TerminfoEntry::f_keys` slot order.
pub const FUNCTION_KEYS: [UnibiString; MAX_FUNCTION_KEY] = [
    unibi_cap::KEY_F1,
    unibi_cap::KEY_F2,
    unibi_cap::KEY_F3,
    unibi_cap::KEY_F4,
    unibi_cap::KEY_F5,
    unibi_cap::KEY_F6,
    unibi_cap::KEY_F7,
    unibi_cap::KEY_F8,
    unibi_cap::KEY_F9,
    unibi_cap::KEY_F10,
    unibi_cap::KEY_F11,
    unibi_cap::KEY_F12,
    unibi_cap::KEY_F13,
    unibi_cap::KEY_F14,
    unibi_cap::KEY_F15,
    unibi_cap::KEY_F16,
    unibi_cap::KEY_F17,
    unibi_cap::KEY_F18,
    unibi_cap::KEY_F19,
    unibi_cap::KEY_F20,
    unibi_cap::KEY_F21,
    unibi_cap::KEY_F22,
    unibi_cap::KEY_F23,
    unibi_cap::KEY_F24,
    unibi_cap::KEY_F25,
    unibi_cap::KEY_F26,
    unibi_cap::KEY_F27,
    unibi_cap::KEY_F28,
    unibi_cap::KEY_F29,
    unibi_cap::KEY_F30,
    unibi_cap::KEY_F31,
    unibi_cap::KEY_F32,
    unibi_cap::KEY_F33,
    unibi_cap::KEY_F34,
    unibi_cap::KEY_F35,
    unibi_cap::KEY_F36,
    unibi_cap::KEY_F37,
    unibi_cap::KEY_F38,
    unibi_cap::KEY_F39,
    unibi_cap::KEY_F40,
    unibi_cap::KEY_F41,
    unibi_cap::KEY_F42,
    unibi_cap::KEY_F43,
    unibi_cap::KEY_F44,
    unibi_cap::KEY_F45,
    unibi_cap::KEY_F46,
    unibi_cap::KEY_F47,
    unibi_cap::KEY_F48,
    unibi_cap::KEY_F49,
    unibi_cap::KEY_F50,
    unibi_cap::KEY_F51,
    unibi_cap::KEY_F52,
    unibi_cap::KEY_F53,
    unibi_cap::KEY_F54,
    unibi_cap::KEY_F55,
    unibi_cap::KEY_F56,
    unibi_cap::KEY_F57,
    unibi_cap::KEY_F58,
    unibi_cap::KEY_F59,
    unibi_cap::KEY_F60,
    unibi_cap::KEY_F61,
    unibi_cap::KEY_F62,
    unibi_cap::KEY_F63,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_capability_names_follow_their_stems() {
        for key in &KEYS {
            assert_eq!(key.name.to_bytes(), format!("key_{}", key.stem).as_bytes());
            assert_eq!(
                key.shifted_name.to_bytes(),
                format!("key_s{}", key.stem).as_bytes()
            );
        }
    }

    /// The slot names and the table are two hand-written lists of the same
    /// order; a rename in one without the other would silently swap two keys.
    #[test]
    fn key_slots_address_their_own_entries() {
        assert_eq!(KEYS[key_slot::BACKSPACE].stem, "backspace");
        assert_eq!(KEYS[key_slot::BEG].stem, "beg");
        assert_eq!(KEYS[key_slot::BTAB].stem, "btab");
        assert_eq!(KEYS[key_slot::CLEAR].stem, "clear");
        assert_eq!(KEYS[key_slot::DC].stem, "dc");
        assert_eq!(KEYS[key_slot::END].stem, "end");
        assert_eq!(KEYS[key_slot::FIND].stem, "find");
        assert_eq!(KEYS[key_slot::HOME].stem, "home");
        assert_eq!(KEYS[key_slot::IC].stem, "ic");
        assert_eq!(KEYS[key_slot::NPAGE].stem, "npage");
        assert_eq!(KEYS[key_slot::PPAGE].stem, "ppage");
        assert_eq!(KEYS[key_slot::SELECT].stem, "select");
        assert_eq!(KEYS[key_slot::SUSPEND].stem, "suspend");
        assert_eq!(KEYS[key_slot::UNDO].stem, "undo");
        assert_eq!(KEYS[key_slot::LEFT].stem, "left");
        assert_eq!(KEYS[key_slot::RIGHT].stem, "right");
    }

    /// Same check for the `defs` slots.
    #[test]
    fn def_slots_address_their_own_capabilities() {
        assert_eq!(
            STRING_CAPS[kTerm_carriage_return as usize].name,
            "carriage_return"
        );
        assert_eq!(
            STRING_CAPS[kTerm_change_scroll_region as usize].name,
            "change_scroll_region"
        );
        assert_eq!(
            STRING_CAPS[kTerm_clear_screen as usize].name,
            "clear_screen"
        );
        assert_eq!(STRING_CAPS[kTerm_clr_eol as usize].name, "clr_eol");
        assert_eq!(STRING_CAPS[kTerm_clr_eos as usize].name, "clr_eos");
        assert_eq!(
            STRING_CAPS[kTerm_cursor_address as usize].name,
            "cursor_address"
        );
        assert_eq!(STRING_CAPS[kTerm_cursor_down as usize].name, "cursor_down");
        assert_eq!(
            STRING_CAPS[kTerm_cursor_invisible as usize].name,
            "cursor_invisible"
        );
        assert_eq!(STRING_CAPS[kTerm_cursor_left as usize].name, "cursor_left");
        assert_eq!(STRING_CAPS[kTerm_cursor_home as usize].name, "cursor_home");
        assert_eq!(
            STRING_CAPS[kTerm_cursor_normal as usize].name,
            "cursor_normal"
        );
        assert_eq!(STRING_CAPS[kTerm_cursor_up as usize].name, "cursor_up");
        assert_eq!(
            STRING_CAPS[kTerm_cursor_right as usize].name,
            "cursor_right"
        );
        assert_eq!(STRING_CAPS[kTerm_delete_line as usize].name, "delete_line");
        assert_eq!(
            STRING_CAPS[kTerm_enter_blink_mode as usize].name,
            "enter_blink_mode"
        );
        assert_eq!(
            STRING_CAPS[kTerm_enter_bold_mode as usize].name,
            "enter_bold_mode"
        );
        assert_eq!(
            STRING_CAPS[kTerm_enter_ca_mode as usize].name,
            "enter_ca_mode"
        );
        assert_eq!(
            STRING_CAPS[kTerm_enter_dim_mode as usize].name,
            "enter_dim_mode"
        );
        assert_eq!(
            STRING_CAPS[kTerm_enter_italics_mode as usize].name,
            "enter_italics_mode"
        );
        assert_eq!(
            STRING_CAPS[kTerm_enter_reverse_mode as usize].name,
            "enter_reverse_mode"
        );
        assert_eq!(
            STRING_CAPS[kTerm_enter_secure_mode as usize].name,
            "enter_secure_mode"
        );
        assert_eq!(
            STRING_CAPS[kTerm_enter_standout_mode as usize].name,
            "enter_standout_mode"
        );
        assert_eq!(
            STRING_CAPS[kTerm_enter_underline_mode as usize].name,
            "enter_underline_mode"
        );
        assert_eq!(STRING_CAPS[kTerm_erase_chars as usize].name, "erase_chars");
        assert_eq!(
            STRING_CAPS[kTerm_exit_attribute_mode as usize].name,
            "exit_attribute_mode"
        );
        assert_eq!(
            STRING_CAPS[kTerm_exit_ca_mode as usize].name,
            "exit_ca_mode"
        );
        assert_eq!(
            STRING_CAPS[kTerm_from_status_line as usize].name,
            "from_status_line"
        );
        assert_eq!(STRING_CAPS[kTerm_insert_line as usize].name, "insert_line");
        assert_eq!(
            STRING_CAPS[kTerm_keypad_local as usize].name,
            "keypad_local"
        );
        assert_eq!(STRING_CAPS[kTerm_keypad_xmit as usize].name, "keypad_xmit");
        assert_eq!(
            STRING_CAPS[kTerm_parm_delete_line as usize].name,
            "parm_delete_line"
        );
        assert_eq!(
            STRING_CAPS[kTerm_parm_down_cursor as usize].name,
            "parm_down_cursor"
        );
        assert_eq!(
            STRING_CAPS[kTerm_parm_insert_line as usize].name,
            "parm_insert_line"
        );
        assert_eq!(
            STRING_CAPS[kTerm_parm_left_cursor as usize].name,
            "parm_left_cursor"
        );
        assert_eq!(
            STRING_CAPS[kTerm_parm_right_cursor as usize].name,
            "parm_right_cursor"
        );
        assert_eq!(
            STRING_CAPS[kTerm_parm_up_cursor as usize].name,
            "parm_up_cursor"
        );
        assert_eq!(
            STRING_CAPS[kTerm_set_a_background as usize].name,
            "set_a_background"
        );
        assert_eq!(
            STRING_CAPS[kTerm_set_a_foreground as usize].name,
            "set_a_foreground"
        );
        assert_eq!(
            STRING_CAPS[kTerm_set_attributes as usize].name,
            "set_attributes"
        );
        assert_eq!(
            STRING_CAPS[kTerm_set_lr_margin as usize].name,
            "set_lr_margin"
        );
        assert_eq!(
            STRING_CAPS[kTerm_to_status_line as usize].name,
            "to_status_line"
        );
        assert_eq!(
            EXT_CAPS[kTerm_reset_cursor_style as usize - 41].name,
            "reset_cursor_style"
        );
        assert_eq!(
            EXT_CAPS[kTerm_set_cursor_style as usize - 41].name,
            "set_cursor_style"
        );
        assert_eq!(
            EXT_CAPS[kTerm_enter_strikethrough_mode as usize - 41].name,
            "enter_strikethrough_mode"
        );
        assert_eq!(
            EXT_CAPS[kTerm_set_rgb_foreground as usize - 41].name,
            "set_rgb_foreground"
        );
        assert_eq!(
            EXT_CAPS[kTerm_set_rgb_background as usize - 41].name,
            "set_rgb_background"
        );
        assert_eq!(
            EXT_CAPS[kTerm_set_cursor_color as usize - 41].name,
            "set_cursor_color"
        );
        assert_eq!(
            EXT_CAPS[kTerm_reset_cursor_color as usize - 41].name,
            "reset_cursor_color"
        );
        assert_eq!(
            EXT_CAPS[kTerm_set_underline_style as usize - 41].name,
            "set_underline_style"
        );
        assert_eq!(kTermCount as usize, STRING_CAPS.len() + EXT_CAPS.len());
    }
}
