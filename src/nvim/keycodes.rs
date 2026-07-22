use crate::src::nvim::charset::{transchar, vim_isprintc, vim_str2nr};
use crate::src::nvim::eval::vars::get_var_value;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{current_sctx, e_invarg, e_usingsid};
use crate::src::nvim::mbyte::{
    utf_char2bytes, utf_char2len, utf_ptr2char, utf_ptr2len, utfc_ptr2len, utfc_ptr2len_len,
};
use crate::src::nvim::memory::{xmalloc, xrealloc};
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::{
    __assert_fail, gettext, snprintf, strcpy, strlen, strncasecmp, strncmp,
};
use crate::src::nvim::strings::{vim_strchr, vim_strnicmp_asc};
pub use crate::src::nvim::types::{
    int32_t, int64_t, key_extra, linenr_T, scid_T, sctx_T, size_t, ssize_t, uint16_t, uint64_t,
    uint8_t, uintmax_t, uvarnumber_T, varnumber_T, String_0,
};
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const STR2NR_QUOTE: C2Rust_Unnamed = 16;
pub const STR2NR_NO_OCT: C2Rust_Unnamed = 13;
pub const STR2NR_ALL: C2Rust_Unnamed = 15;
pub const STR2NR_FORCE: C2Rust_Unnamed = 128;
pub const STR2NR_OOCT: C2Rust_Unnamed = 8;
pub const STR2NR_HEX: C2Rust_Unnamed = 4;
pub const STR2NR_OCT: C2Rust_Unnamed = 2;
pub const STR2NR_BIN: C2Rust_Unnamed = 1;
pub const STR2NR_DEC: C2Rust_Unnamed = 0;
pub const KE_WILD: key_extra = 108;
pub const KE_COMMAND: key_extra = 104;
pub const KE_LUA: key_extra = 103;
pub const KE_EVENT: key_extra = 102;
pub const KE_MOUSEMOVE: key_extra = 100;
pub const KE_NOP: key_extra = 97;
pub const KE_DROP: key_extra = 95;
pub const KE_X2RELEASE: key_extra = 94;
pub const KE_X2DRAG: key_extra = 93;
pub const KE_X2MOUSE: key_extra = 92;
pub const KE_X1RELEASE: key_extra = 91;
pub const KE_X1DRAG: key_extra = 90;
pub const KE_X1MOUSE: key_extra = 89;
pub const KE_C_END: key_extra = 88;
pub const KE_C_HOME: key_extra = 87;
pub const KE_C_RIGHT: key_extra = 86;
pub const KE_C_LEFT: key_extra = 85;
pub const KE_CMDWIN: key_extra = 84;
pub const KE_PLUG: key_extra = 83;
pub const KE_SNR: key_extra = 82;
pub const KE_KDEL: key_extra = 80;
pub const KE_KINS: key_extra = 79;
pub const KE_MOUSERIGHT: key_extra = 78;
pub const KE_MOUSELEFT: key_extra = 77;
pub const KE_MOUSEUP: key_extra = 76;
pub const KE_MOUSEDOWN: key_extra = 75;
pub const KE_S_XF4: key_extra = 74;
pub const KE_S_XF3: key_extra = 73;
pub const KE_S_XF2: key_extra = 72;
pub const KE_S_XF1: key_extra = 71;
pub const KE_LEFTRELEASE_NM: key_extra = 70;
pub const KE_LEFTMOUSE_NM: key_extra = 69;
pub const KE_XRIGHT: key_extra = 68;
pub const KE_XLEFT: key_extra = 67;
pub const KE_XDOWN: key_extra = 66;
pub const KE_XUP: key_extra = 65;
pub const KE_ZHOME: key_extra = 64;
pub const KE_XHOME: key_extra = 63;
pub const KE_ZEND: key_extra = 62;
pub const KE_XEND: key_extra = 61;
pub const KE_XF4: key_extra = 60;
pub const KE_XF3: key_extra = 59;
pub const KE_XF2: key_extra = 58;
pub const KE_XF1: key_extra = 57;
pub const KE_S_TAB_OLD: key_extra = 55;
pub const KE_TAB: key_extra = 54;
pub const KE_IGNORE: key_extra = 53;
pub const KE_RIGHTRELEASE: key_extra = 52;
pub const KE_RIGHTDRAG: key_extra = 51;
pub const KE_RIGHTMOUSE: key_extra = 50;
pub const KE_MIDDLERELEASE: key_extra = 49;
pub const KE_MIDDLEDRAG: key_extra = 48;
pub const KE_MIDDLEMOUSE: key_extra = 47;
pub const KE_LEFTRELEASE: key_extra = 46;
pub const KE_LEFTDRAG: key_extra = 45;
pub const KE_LEFTMOUSE: key_extra = 44;
pub const KE_MOUSE: key_extra = 43;
pub const KE_S_F37: key_extra = 42;
pub const KE_S_F36: key_extra = 41;
pub const KE_S_F35: key_extra = 40;
pub const KE_S_F34: key_extra = 39;
pub const KE_S_F33: key_extra = 38;
pub const KE_S_F32: key_extra = 37;
pub const KE_S_F31: key_extra = 36;
pub const KE_S_F30: key_extra = 35;
pub const KE_S_F29: key_extra = 34;
pub const KE_S_F28: key_extra = 33;
pub const KE_S_F27: key_extra = 32;
pub const KE_S_F26: key_extra = 31;
pub const KE_S_F25: key_extra = 30;
pub const KE_S_F24: key_extra = 29;
pub const KE_S_F23: key_extra = 28;
pub const KE_S_F22: key_extra = 27;
pub const KE_S_F21: key_extra = 26;
pub const KE_S_F20: key_extra = 25;
pub const KE_S_F19: key_extra = 24;
pub const KE_S_F18: key_extra = 23;
pub const KE_S_F17: key_extra = 22;
pub const KE_S_F16: key_extra = 21;
pub const KE_S_F15: key_extra = 20;
pub const KE_S_F14: key_extra = 19;
pub const KE_S_F13: key_extra = 18;
pub const KE_S_F12: key_extra = 17;
pub const KE_S_F11: key_extra = 16;
pub const KE_S_F10: key_extra = 15;
pub const KE_S_F9: key_extra = 14;
pub const KE_S_F8: key_extra = 13;
pub const KE_S_F7: key_extra = 12;
pub const KE_S_F6: key_extra = 11;
pub const KE_S_F5: key_extra = 10;
pub const KE_S_F4: key_extra = 9;
pub const KE_S_F3: key_extra = 8;
pub const KE_S_F2: key_extra = 7;
pub const KE_S_F1: key_extra = 6;
pub const KE_S_DOWN: key_extra = 5;
pub const KE_S_UP: key_extra = 4;
pub type C2Rust_Unnamed_0 = ::core::ffi::c_uint;
pub const REPTERM_NO_SIMPLIFY: C2Rust_Unnamed_0 = 8;
pub const REPTERM_NO_SPECIAL: C2Rust_Unnamed_0 = 4;
pub const REPTERM_DO_LT: C2Rust_Unnamed_0 = 2;
pub const REPTERM_FROM_PART: C2Rust_Unnamed_0 = 1;
pub type C2Rust_Unnamed_1 = ::core::ffi::c_uint;
pub const FSK_SIMPLIFY: C2Rust_Unnamed_1 = 8;
pub const FSK_IN_STRING: C2Rust_Unnamed_1 = 4;
pub const FSK_KEEP_X_KEY: C2Rust_Unnamed_1 = 2;
pub const FSK_KEYCODE: C2Rust_Unnamed_1 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct modmasktable {
    pub mod_mask: uint16_t,
    pub mod_flag: uint16_t,
    pub name: ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct key_name_entry {
    pub key: ::core::ffi::c_int,
    pub is_alt: bool,
    pub name: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mousetable {
    pub pseudo_code: ::core::ffi::c_int,
    pub button: ::core::ffi::c_int,
    pub is_click: bool,
    pub is_drag: bool,
}
pub const MOUSE_RELEASE: C2Rust_Unnamed_2 = 3;
pub const MOUSE_X2: C2Rust_Unnamed_2 = 1024;
pub const MOUSE_X1: C2Rust_Unnamed_2 = 768;
pub const MOUSE_RIGHT: C2Rust_Unnamed_2 = 2;
pub const MOUSE_MIDDLE: C2Rust_Unnamed_2 = 1;
pub const MOUSE_LEFT: C2Rust_Unnamed_2 = 0;
pub type C2Rust_Unnamed_2 = ::core::ffi::c_uint;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 53] = unsafe {
    ::core::mem::transmute::<[u8; 53], [::core::ffi::c_char; 53]>(
        *b"unsigned int special_to_buf(int, int, _Bool, char *)\0",
    )
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const CPO_BSLASH: ::core::ffi::c_int = 'B' as ::core::ffi::c_int;
static mod_mask_table: GlobalCell<[modmasktable; 10]> = GlobalCell::new([
    modmasktable {
        mod_mask: MOD_MASK_ALT as uint16_t,
        mod_flag: MOD_MASK_ALT as uint16_t,
        name: 'M' as ::core::ffi::c_char,
    },
    modmasktable {
        mod_mask: MOD_MASK_META as uint16_t,
        mod_flag: MOD_MASK_META as uint16_t,
        name: 'T' as ::core::ffi::c_char,
    },
    modmasktable {
        mod_mask: MOD_MASK_CTRL as uint16_t,
        mod_flag: MOD_MASK_CTRL as uint16_t,
        name: 'C' as ::core::ffi::c_char,
    },
    modmasktable {
        mod_mask: MOD_MASK_SHIFT as uint16_t,
        mod_flag: MOD_MASK_SHIFT as uint16_t,
        name: 'S' as ::core::ffi::c_char,
    },
    modmasktable {
        mod_mask: MOD_MASK_MULTI_CLICK as uint16_t,
        mod_flag: MOD_MASK_2CLICK as uint16_t,
        name: '2' as ::core::ffi::c_char,
    },
    modmasktable {
        mod_mask: MOD_MASK_MULTI_CLICK as uint16_t,
        mod_flag: MOD_MASK_3CLICK as uint16_t,
        name: '3' as ::core::ffi::c_char,
    },
    modmasktable {
        mod_mask: MOD_MASK_MULTI_CLICK as uint16_t,
        mod_flag: MOD_MASK_4CLICK as uint16_t,
        name: '4' as ::core::ffi::c_char,
    },
    modmasktable {
        mod_mask: MOD_MASK_CMD as uint16_t,
        mod_flag: MOD_MASK_CMD as uint16_t,
        name: 'D' as ::core::ffi::c_char,
    },
    modmasktable {
        mod_mask: MOD_MASK_ALT as uint16_t,
        mod_flag: MOD_MASK_ALT as uint16_t,
        name: 'A' as ::core::ffi::c_char,
    },
    modmasktable {
        mod_mask: 0 as uint16_t,
        mod_flag: 0 as uint16_t,
        name: NUL as ::core::ffi::c_char,
    },
]);
pub const MOD_KEYS_ENTRY_SIZE: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
static modifier_keys_table: GlobalCell<[uint8_t; 376]> = GlobalCell::new([
    MOD_MASK_SHIFT as uint8_t,
    '&' as uint8_t,
    '9' as uint8_t,
    '@' as uint8_t,
    '1' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '&' as uint8_t,
    '0' as uint8_t,
    '@' as uint8_t,
    '2' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '*' as uint8_t,
    '1' as uint8_t,
    '@' as uint8_t,
    '4' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '*' as uint8_t,
    '2' as uint8_t,
    '@' as uint8_t,
    '5' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '*' as uint8_t,
    '3' as uint8_t,
    '@' as uint8_t,
    '6' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '*' as uint8_t,
    '4' as uint8_t,
    'k' as uint8_t,
    'D' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '*' as uint8_t,
    '5' as uint8_t,
    'k' as uint8_t,
    'L' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '*' as uint8_t,
    '7' as uint8_t,
    '@' as uint8_t,
    '7' as uint8_t,
    MOD_MASK_CTRL as uint8_t,
    KS_EXTRA as uint8_t,
    KE_C_END as ::core::ffi::c_int as uint8_t,
    '@' as uint8_t,
    '7' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '*' as uint8_t,
    '9' as uint8_t,
    '@' as uint8_t,
    '9' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '*' as uint8_t,
    '0' as uint8_t,
    '@' as uint8_t,
    '0' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '#' as uint8_t,
    '1' as uint8_t,
    '%' as uint8_t,
    '1' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '#' as uint8_t,
    '2' as uint8_t,
    'k' as uint8_t,
    'h' as uint8_t,
    MOD_MASK_CTRL as uint8_t,
    KS_EXTRA as uint8_t,
    KE_C_HOME as ::core::ffi::c_int as uint8_t,
    'k' as uint8_t,
    'h' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '#' as uint8_t,
    '3' as uint8_t,
    'k' as uint8_t,
    'I' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '#' as uint8_t,
    '4' as uint8_t,
    'k' as uint8_t,
    'l' as uint8_t,
    MOD_MASK_CTRL as uint8_t,
    KS_EXTRA as uint8_t,
    KE_C_LEFT as ::core::ffi::c_int as uint8_t,
    'k' as uint8_t,
    'l' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '%' as uint8_t,
    'a' as uint8_t,
    '%' as uint8_t,
    '3' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '%' as uint8_t,
    'b' as uint8_t,
    '%' as uint8_t,
    '4' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '%' as uint8_t,
    'c' as uint8_t,
    '%' as uint8_t,
    '5' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '%' as uint8_t,
    'd' as uint8_t,
    '%' as uint8_t,
    '7' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '%' as uint8_t,
    'e' as uint8_t,
    '%' as uint8_t,
    '8' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '%' as uint8_t,
    'f' as uint8_t,
    '%' as uint8_t,
    '9' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '%' as uint8_t,
    'g' as uint8_t,
    '%' as uint8_t,
    '0' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '%' as uint8_t,
    'h' as uint8_t,
    '&' as uint8_t,
    '3' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '%' as uint8_t,
    'i' as uint8_t,
    'k' as uint8_t,
    'r' as uint8_t,
    MOD_MASK_CTRL as uint8_t,
    KS_EXTRA as uint8_t,
    KE_C_RIGHT as ::core::ffi::c_int as uint8_t,
    'k' as uint8_t,
    'r' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '%' as uint8_t,
    'j' as uint8_t,
    '&' as uint8_t,
    '5' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '!' as uint8_t,
    '1' as uint8_t,
    '&' as uint8_t,
    '6' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '!' as uint8_t,
    '2' as uint8_t,
    '&' as uint8_t,
    '7' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    '!' as uint8_t,
    '3' as uint8_t,
    '&' as uint8_t,
    '8' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_UP as ::core::ffi::c_int as uint8_t,
    'k' as uint8_t,
    'u' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_DOWN as ::core::ffi::c_int as uint8_t,
    'k' as uint8_t,
    'd' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_XF1 as ::core::ffi::c_int as uint8_t,
    KS_EXTRA as uint8_t,
    KE_XF1 as ::core::ffi::c_int as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_XF2 as ::core::ffi::c_int as uint8_t,
    KS_EXTRA as uint8_t,
    KE_XF2 as ::core::ffi::c_int as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_XF3 as ::core::ffi::c_int as uint8_t,
    KS_EXTRA as uint8_t,
    KE_XF3 as ::core::ffi::c_int as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_XF4 as ::core::ffi::c_int as uint8_t,
    KS_EXTRA as uint8_t,
    KE_XF4 as ::core::ffi::c_int as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F1 as ::core::ffi::c_int as uint8_t,
    'k' as uint8_t,
    '1' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F2 as ::core::ffi::c_int as uint8_t,
    'k' as uint8_t,
    '2' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F3 as ::core::ffi::c_int as uint8_t,
    'k' as uint8_t,
    '3' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F4 as ::core::ffi::c_int as uint8_t,
    'k' as uint8_t,
    '4' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F5 as ::core::ffi::c_int as uint8_t,
    'k' as uint8_t,
    '5' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F6 as ::core::ffi::c_int as uint8_t,
    'k' as uint8_t,
    '6' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F7 as ::core::ffi::c_int as uint8_t,
    'k' as uint8_t,
    '7' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F8 as ::core::ffi::c_int as uint8_t,
    'k' as uint8_t,
    '8' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F9 as ::core::ffi::c_int as uint8_t,
    'k' as uint8_t,
    '9' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F10 as ::core::ffi::c_int as uint8_t,
    'k' as uint8_t,
    ';' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F11 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    '1' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F12 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    '2' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F13 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    '3' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F14 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    '4' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F15 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    '5' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F16 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    '6' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F17 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    '7' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F18 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    '8' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F19 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    '9' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F20 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    'A' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F21 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    'B' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F22 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    'C' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F23 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    'D' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F24 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    'E' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F25 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    'F' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F26 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    'G' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F27 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    'H' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F28 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    'I' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F29 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    'J' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F30 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    'K' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F31 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    'L' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F32 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    'M' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F33 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    'N' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F34 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    'O' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F35 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    'P' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F36 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    'Q' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    KS_EXTRA as uint8_t,
    KE_S_F37 as ::core::ffi::c_int as uint8_t,
    'F' as uint8_t,
    'R' as uint8_t,
    MOD_MASK_SHIFT as uint8_t,
    'k' as uint8_t,
    'B' as uint8_t,
    KS_EXTRA as uint8_t,
    KE_TAB as ::core::ffi::c_int as uint8_t,
    NUL as uint8_t,
]);
static mouse_table: GlobalCell<[mousetable; 18]> = GlobalCell::new([
    mousetable {
        pseudo_code: KE_LEFTMOUSE as ::core::ffi::c_int,
        button: MOUSE_LEFT as ::core::ffi::c_int,
        is_click: true_0 != 0,
        is_drag: false_0 != 0,
    },
    mousetable {
        pseudo_code: KE_LEFTDRAG as ::core::ffi::c_int,
        button: MOUSE_LEFT as ::core::ffi::c_int,
        is_click: false_0 != 0,
        is_drag: true_0 != 0,
    },
    mousetable {
        pseudo_code: KE_LEFTRELEASE as ::core::ffi::c_int,
        button: MOUSE_LEFT as ::core::ffi::c_int,
        is_click: false_0 != 0,
        is_drag: false_0 != 0,
    },
    mousetable {
        pseudo_code: KE_MIDDLEMOUSE as ::core::ffi::c_int,
        button: MOUSE_MIDDLE as ::core::ffi::c_int,
        is_click: true_0 != 0,
        is_drag: false_0 != 0,
    },
    mousetable {
        pseudo_code: KE_MIDDLEDRAG as ::core::ffi::c_int,
        button: MOUSE_MIDDLE as ::core::ffi::c_int,
        is_click: false_0 != 0,
        is_drag: true_0 != 0,
    },
    mousetable {
        pseudo_code: KE_MIDDLERELEASE as ::core::ffi::c_int,
        button: MOUSE_MIDDLE as ::core::ffi::c_int,
        is_click: false_0 != 0,
        is_drag: false_0 != 0,
    },
    mousetable {
        pseudo_code: KE_RIGHTMOUSE as ::core::ffi::c_int,
        button: MOUSE_RIGHT as ::core::ffi::c_int,
        is_click: true_0 != 0,
        is_drag: false_0 != 0,
    },
    mousetable {
        pseudo_code: KE_RIGHTDRAG as ::core::ffi::c_int,
        button: MOUSE_RIGHT as ::core::ffi::c_int,
        is_click: false_0 != 0,
        is_drag: true_0 != 0,
    },
    mousetable {
        pseudo_code: KE_RIGHTRELEASE as ::core::ffi::c_int,
        button: MOUSE_RIGHT as ::core::ffi::c_int,
        is_click: false_0 != 0,
        is_drag: false_0 != 0,
    },
    mousetable {
        pseudo_code: KE_X1MOUSE as ::core::ffi::c_int,
        button: MOUSE_X1 as ::core::ffi::c_int,
        is_click: true_0 != 0,
        is_drag: false_0 != 0,
    },
    mousetable {
        pseudo_code: KE_X1DRAG as ::core::ffi::c_int,
        button: MOUSE_X1 as ::core::ffi::c_int,
        is_click: false_0 != 0,
        is_drag: true_0 != 0,
    },
    mousetable {
        pseudo_code: KE_X1RELEASE as ::core::ffi::c_int,
        button: MOUSE_X1 as ::core::ffi::c_int,
        is_click: false_0 != 0,
        is_drag: false_0 != 0,
    },
    mousetable {
        pseudo_code: KE_X2MOUSE as ::core::ffi::c_int,
        button: MOUSE_X2 as ::core::ffi::c_int,
        is_click: true_0 != 0,
        is_drag: false_0 != 0,
    },
    mousetable {
        pseudo_code: KE_X2DRAG as ::core::ffi::c_int,
        button: MOUSE_X2 as ::core::ffi::c_int,
        is_click: false_0 != 0,
        is_drag: true_0 != 0,
    },
    mousetable {
        pseudo_code: KE_X2RELEASE as ::core::ffi::c_int,
        button: MOUSE_X2 as ::core::ffi::c_int,
        is_click: false_0 != 0,
        is_drag: false_0 != 0,
    },
    mousetable {
        pseudo_code: KE_MOUSEMOVE as ::core::ffi::c_int,
        button: MOUSE_RELEASE as ::core::ffi::c_int,
        is_click: false_0 != 0,
        is_drag: true_0 != 0,
    },
    mousetable {
        pseudo_code: KE_IGNORE as ::core::ffi::c_int,
        button: MOUSE_RELEASE as ::core::ffi::c_int,
        is_click: false_0 != 0,
        is_drag: false_0 != 0,
    },
    mousetable {
        pseudo_code: 0 as ::core::ffi::c_int,
        button: 0 as ::core::ffi::c_int,
        is_click: false,
        is_drag: false,
    },
]);
#[no_mangle]
pub unsafe extern "C" fn name_to_mod_mask(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    c = if c < 'a' as ::core::ffi::c_int || c > 'z' as ::core::ffi::c_int {
        c
    } else {
        c - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
    };
    let mut i: size_t = 0 as size_t;
    while (*mod_mask_table.ptr())[i as usize].mod_mask as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if c == (*mod_mask_table.ptr())[i as usize].name as uint8_t as ::core::ffi::c_int {
            return (*mod_mask_table.ptr())[i as usize].mod_flag as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn simplify_key(
    key: ::core::ffi::c_int,
    mut modifiers: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if *modifiers & (MOD_MASK_SHIFT | MOD_MASK_CTRL) == 0 {
        return key;
    }
    if key == TAB && *modifiers & MOD_MASK_SHIFT != 0 {
        *modifiers &= !MOD_MASK_SHIFT;
        return K_S_TAB;
    }
    let key0: ::core::ffi::c_int = -key & 0xff as ::core::ffi::c_int;
    let key1: ::core::ffi::c_int = (-key as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int
        & 0xff as ::core::ffi::c_uint) as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while (*modifier_keys_table.ptr())[i as usize] as ::core::ffi::c_int != NUL {
        if key0
            == (*modifier_keys_table.ptr())[(i + 3 as ::core::ffi::c_int) as usize]
                as ::core::ffi::c_int
            && key1
                == (*modifier_keys_table.ptr())[(i + 4 as ::core::ffi::c_int) as usize]
                    as ::core::ffi::c_int
            && *modifiers & (*modifier_keys_table.ptr())[i as usize] as ::core::ffi::c_int != 0
        {
            *modifiers &= !((*modifier_keys_table.ptr())[i as usize] as ::core::ffi::c_int);
            return -((*modifier_keys_table.ptr())[(i + 1 as ::core::ffi::c_int) as usize]
                as ::core::ffi::c_int
                + (((*modifier_keys_table.ptr())[(i + 2 as ::core::ffi::c_int) as usize]
                    as ::core::ffi::c_int)
                    << 8 as ::core::ffi::c_int));
        }
        i += MOD_KEYS_ENTRY_SIZE;
    }
    return key;
}
#[no_mangle]
pub unsafe extern "C" fn handle_x_keys(key: ::core::ffi::c_int) -> ::core::ffi::c_int {
    match key {
        -16893 => return K_UP,
        -17149 => return K_DOWN,
        -17405 => return K_LEFT,
        -17661 => return K_RIGHT,
        -16381 => return K_HOME,
        -16637 => return K_HOME,
        -15869 => return K_END,
        -16125 => return K_END,
        -14845 => return K_F1,
        -15101 => return K_F2,
        -15357 => return K_F3,
        -15613 => return K_F4,
        K_S_XF1 => {
            return -(253 as ::core::ffi::c_int
                + ((KE_S_F1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
        }
        K_S_XF2 => {
            return -(253 as ::core::ffi::c_int
                + ((KE_S_F2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
        }
        K_S_XF3 => {
            return -(253 as ::core::ffi::c_int
                + ((KE_S_F3 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
        }
        K_S_XF4 => {
            return -(253 as ::core::ffi::c_int
                + ((KE_S_F4 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
        }
        _ => {}
    }
    return key;
}
#[no_mangle]
pub unsafe extern "C" fn get_special_key_name(
    mut c: ::core::ffi::c_int,
    mut modifiers: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    static string: GlobalCell<[::core::ffi::c_char; 33]> = GlobalCell::new([0; 33]);
    (*string.ptr())[0 as ::core::ffi::c_int as usize] = '<' as ::core::ffi::c_char;
    let mut idx: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    if c < 0 as ::core::ffi::c_int && -c & 0xff as ::core::ffi::c_int == KS_KEY {
        c = (-c as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_uint)
            as ::core::ffi::c_int;
    }
    if c < 0 as ::core::ffi::c_int {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while (*modifier_keys_table.ptr())[i as usize] as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if -c & 0xff as ::core::ffi::c_int
                == (*modifier_keys_table.ptr())[(i + 1 as ::core::ffi::c_int) as usize]
                    as ::core::ffi::c_int
                && (-c as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int
                    & 0xff as ::core::ffi::c_uint) as ::core::ffi::c_int
                    == (*modifier_keys_table.ptr())[(i + 2 as ::core::ffi::c_int) as usize]
                        as ::core::ffi::c_int
            {
                modifiers |= (*modifier_keys_table.ptr())[i as usize] as ::core::ffi::c_int;
                c = -((*modifier_keys_table.ptr())[(i + 3 as ::core::ffi::c_int) as usize]
                    as ::core::ffi::c_int
                    + (((*modifier_keys_table.ptr())[(i + 4 as ::core::ffi::c_int) as usize]
                        as ::core::ffi::c_int)
                        << 8 as ::core::ffi::c_int));
                break;
            } else {
                i += MOD_KEYS_ENTRY_SIZE;
            }
        }
    }
    let mut table_idx: ::core::ffi::c_int = find_special_key_in_table(c);
    if c > 0 as ::core::ffi::c_int && utf_char2len(c) == 1 as ::core::ffi::c_int {
        if table_idx < 0 as ::core::ffi::c_int
            && (!vim_isprintc(c) || c & 0x7f as ::core::ffi::c_int == ' ' as ::core::ffi::c_int)
            && c & 0x80 as ::core::ffi::c_int != 0
        {
            c &= 0x7f as ::core::ffi::c_int;
            modifiers |= MOD_MASK_ALT;
            table_idx = find_special_key_in_table(c);
        }
        if table_idx < 0 as ::core::ffi::c_int && !vim_isprintc(c) && c < ' ' as ::core::ffi::c_int
        {
            c += '@' as ::core::ffi::c_int;
            modifiers |= MOD_MASK_CTRL;
        }
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while (*mod_mask_table.ptr())[i_0 as usize].name as ::core::ffi::c_int
        != 'A' as ::core::ffi::c_int
    {
        if modifiers & (*mod_mask_table.ptr())[i_0 as usize].mod_mask as ::core::ffi::c_int
            == (*mod_mask_table.ptr())[i_0 as usize].mod_flag as ::core::ffi::c_int
        {
            let c2rust_fresh0 = idx;
            idx = idx + 1;
            (*string.ptr())[c2rust_fresh0 as usize] = (*mod_mask_table.ptr())[i_0 as usize].name;
            let c2rust_fresh1 = idx;
            idx = idx + 1;
            (*string.ptr())[c2rust_fresh1 as usize] = '-' as ::core::ffi::c_char;
        }
        i_0 += 1;
    }
    if table_idx < 0 as ::core::ffi::c_int {
        if c < 0 as ::core::ffi::c_int {
            let c2rust_fresh2 = idx;
            idx = idx + 1;
            (*string.ptr())[c2rust_fresh2 as usize] = 't' as ::core::ffi::c_char;
            let c2rust_fresh3 = idx;
            idx = idx + 1;
            (*string.ptr())[c2rust_fresh3 as usize] = '_' as ::core::ffi::c_char;
            let c2rust_fresh4 = idx;
            idx = idx + 1;
            (*string.ptr())[c2rust_fresh4 as usize] =
                (-c & 0xff as ::core::ffi::c_int) as uint8_t as ::core::ffi::c_char;
            let c2rust_fresh5 = idx;
            idx = idx + 1;
            (*string.ptr())[c2rust_fresh5 as usize] = (-c as ::core::ffi::c_uint
                >> 8 as ::core::ffi::c_int
                & 0xff as ::core::ffi::c_uint)
                as uint8_t
                as ::core::ffi::c_char;
        } else {
            let mut len: ::core::ffi::c_int = utf_char2len(c);
            if len == 1 as ::core::ffi::c_int && vim_isprintc(c) as ::core::ffi::c_int != 0 {
                let c2rust_fresh6 = idx;
                idx = idx + 1;
                (*string.ptr())[c2rust_fresh6 as usize] = c as uint8_t as ::core::ffi::c_char;
            } else if len > 1 as ::core::ffi::c_int {
                idx += utf_char2bytes(
                    c,
                    (string.ptr() as *mut ::core::ffi::c_char).offset(idx as isize),
                );
            } else {
                let mut s: *mut ::core::ffi::c_char = transchar(c);
                while *s != 0 {
                    let c2rust_fresh7 = s;
                    s = s.offset(1);
                    let c2rust_fresh8 = idx;
                    idx = idx + 1;
                    (*string.ptr())[c2rust_fresh8 as usize] = *c2rust_fresh7;
                }
            }
        }
    } else {
        let mut s_0: *const String_0 = &raw const (*((key_names_table.ptr() as *const _)
            as *const key_name_entry)
            .offset(table_idx as isize))
        .name;
        if (*s_0).size as ::core::ffi::c_int + idx + 2 as ::core::ffi::c_int <= MAX_KEY_NAME_LEN {
            strcpy(
                (string.ptr() as *mut ::core::ffi::c_char).offset(idx as isize),
                (*s_0).data,
            );
            idx += (*s_0).size as ::core::ffi::c_int;
        }
    }
    let c2rust_fresh9 = idx;
    idx = idx + 1;
    (*string.ptr())[c2rust_fresh9 as usize] = '>' as ::core::ffi::c_char;
    (*string.ptr())[idx as usize] = NUL as ::core::ffi::c_char;
    return string.ptr() as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn trans_special(
    srcp: *mut *const ::core::ffi::c_char,
    src_len: size_t,
    dst: *mut ::core::ffi::c_char,
    flags: ::core::ffi::c_int,
    escape_ks: bool,
    did_simplify: *mut bool,
) -> ::core::ffi::c_uint {
    let mut modifiers: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut key: ::core::ffi::c_int =
        find_special_key(srcp, src_len, &raw mut modifiers, flags, did_simplify);
    if key == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_uint;
    }
    return special_to_buf(key, modifiers, escape_ks, dst);
}
#[no_mangle]
pub unsafe extern "C" fn special_to_buf(
    mut key: ::core::ffi::c_int,
    mut modifiers: ::core::ffi::c_int,
    mut escape_ks: bool,
    mut dst: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_uint {
    let mut dlen: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    if modifiers != 0 as ::core::ffi::c_int {
        let c2rust_fresh10 = dlen;
        dlen = dlen.wrapping_add(1);
        *dst.offset(c2rust_fresh10 as isize) = K_SPECIAL as uint8_t as ::core::ffi::c_char;
        let c2rust_fresh11 = dlen;
        dlen = dlen.wrapping_add(1);
        *dst.offset(c2rust_fresh11 as isize) = KS_MODIFIER as uint8_t as ::core::ffi::c_char;
        let c2rust_fresh12 = dlen;
        dlen = dlen.wrapping_add(1);
        *dst.offset(c2rust_fresh12 as isize) = modifiers as uint8_t as ::core::ffi::c_char;
    }
    if key < 0 as ::core::ffi::c_int {
        let c2rust_fresh13 = dlen;
        dlen = dlen.wrapping_add(1);
        *dst.offset(c2rust_fresh13 as isize) = K_SPECIAL as uint8_t as ::core::ffi::c_char;
        let c2rust_fresh14 = dlen;
        dlen = dlen.wrapping_add(1);
        *dst.offset(c2rust_fresh14 as isize) =
            (-key & 0xff as ::core::ffi::c_int) as uint8_t as ::core::ffi::c_char;
        let c2rust_fresh15 = dlen;
        dlen = dlen.wrapping_add(1);
        *dst.offset(c2rust_fresh15 as isize) =
            (-key as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_uint)
                as uint8_t as ::core::ffi::c_char;
    } else if escape_ks {
        let mut after: *mut ::core::ffi::c_char = add_char2buf(key, dst.offset(dlen as isize));
        '_c2rust_label: {
            if after >= dst
                && after.offset_from(dst) as uintmax_t
                    <= (2147483647 as ::core::ffi::c_int as ::core::ffi::c_uint)
                        .wrapping_mul(2 as ::core::ffi::c_uint)
                        .wrapping_add(1 as ::core::ffi::c_uint) as uintmax_t
            {
            } else {
                __assert_fail(
                    b"after >= dst && (uintmax_t)(after - dst) <= UINT_MAX\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/keycodes.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    399 as ::core::ffi::c_uint,
                    __ASSERT_FUNCTION.as_ptr(),
                );
            }
        };
        dlen = after.offset_from(dst) as ::core::ffi::c_uint;
    } else {
        dlen = dlen
            .wrapping_add(utf_char2bytes(key, dst.offset(dlen as isize)) as ::core::ffi::c_uint);
    }
    return dlen;
}
#[no_mangle]
pub unsafe extern "C" fn find_special_key(
    srcp: *mut *const ::core::ffi::c_char,
    src_len: size_t,
    modp: *mut ::core::ffi::c_int,
    flags: ::core::ffi::c_int,
    did_simplify: *mut bool,
) -> ::core::ffi::c_int {
    let mut bp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let end: *const ::core::ffi::c_char = (*srcp)
        .offset(src_len as isize)
        .offset(-(1 as ::core::ffi::c_int as isize));
    let in_string: bool = flags & FSK_IN_STRING as ::core::ffi::c_int != 0;
    let mut n: uvarnumber_T = 0;
    let mut l: ::core::ffi::c_int = 0;
    if src_len == 0 as size_t {
        return 0 as ::core::ffi::c_int;
    }
    let mut src: *const ::core::ffi::c_char = *srcp;
    if *src.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        != '<' as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    if *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '*' as ::core::ffi::c_int
    {
        src = src.offset(1);
    }
    let mut last_dash: *const ::core::ffi::c_char = src;
    bp = src.offset(1 as ::core::ffi::c_int as isize);
    while bp <= end
        && (*bp as ::core::ffi::c_int == '-' as ::core::ffi::c_int
            || ascii_isident(*bp as ::core::ffi::c_int) as ::core::ffi::c_int != 0)
    {
        if *bp as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
            last_dash = bp;
            if bp.offset(1 as ::core::ffi::c_int as isize) <= end {
                l = utfc_ptr2len_len(
                    bp.offset(1 as ::core::ffi::c_int as isize),
                    end.offset_from(bp) as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
                );
                if end.offset_from(bp) > l as isize
                    && !(in_string as ::core::ffi::c_int != 0
                        && *bp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '"' as ::core::ffi::c_int)
                    && *bp.offset((l + 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                        == '>' as ::core::ffi::c_int
                {
                    bp = bp.offset(l as isize);
                } else if end.offset_from(bp) > 2 as isize
                    && in_string as ::core::ffi::c_int != 0
                    && *bp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int
                    && *bp.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '"' as ::core::ffi::c_int
                    && *bp.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '>' as ::core::ffi::c_int
                {
                    bp = bp.offset(2 as ::core::ffi::c_int as isize);
                }
            }
        }
        if end.offset_from(bp) > 3 as isize
            && *bp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 't' as ::core::ffi::c_int
            && *bp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '_' as ::core::ffi::c_int
        {
            bp = bp.offset(3 as ::core::ffi::c_int as isize);
        } else if end.offset_from(bp) > 4 as isize
            && strncasecmp(
                bp as *mut ::core::ffi::c_char,
                b"char-\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                5 as ::core::ffi::c_int as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            vim_str2nr(
                bp.offset(5 as ::core::ffi::c_int as isize),
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                &raw mut l,
                STR2NR_ALL as ::core::ffi::c_int,
                ::core::ptr::null_mut::<varnumber_T>(),
                ::core::ptr::null_mut::<uvarnumber_T>(),
                0 as ::core::ffi::c_int,
                true_0 != 0,
                ::core::ptr::null_mut::<bool>(),
            );
            if l == 0 as ::core::ffi::c_int {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
                return 0 as ::core::ffi::c_int;
            }
            bp = bp.offset((l + 5 as ::core::ffi::c_int) as isize);
            break;
        }
        bp = bp.offset(1);
    }
    if bp <= end && *bp as ::core::ffi::c_int == '>' as ::core::ffi::c_int {
        let mut key: ::core::ffi::c_int = 0;
        let mut end_of_name: *const ::core::ffi::c_char =
            bp.offset(1 as ::core::ffi::c_int as isize);
        let mut modifiers: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        bp = src.offset(1 as ::core::ffi::c_int as isize);
        while bp < last_dash {
            if *bp as ::core::ffi::c_int != '-' as ::core::ffi::c_int {
                let mut bit: ::core::ffi::c_int =
                    name_to_mod_mask(*bp as uint8_t as ::core::ffi::c_int);
                if bit == 0 as ::core::ffi::c_int {
                    break;
                }
                modifiers |= bit;
            }
            bp = bp.offset(1);
        }
        if bp >= last_dash {
            if strncasecmp(
                last_dash.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char,
                b"char-\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                5 as ::core::ffi::c_int as size_t,
            ) == 0 as ::core::ffi::c_int
                && ascii_isdigit(
                    *last_dash.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                ) as ::core::ffi::c_int
                    != 0
            {
                vim_str2nr(
                    last_dash.offset(6 as ::core::ffi::c_int as isize),
                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                    &raw mut l,
                    STR2NR_ALL as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<varnumber_T>(),
                    &raw mut n,
                    0 as ::core::ffi::c_int,
                    true_0 != 0,
                    ::core::ptr::null_mut::<bool>(),
                );
                if l == 0 as ::core::ffi::c_int {
                    emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
                    return 0 as ::core::ffi::c_int;
                }
                key = n as ::core::ffi::c_int;
            } else {
                let mut off: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                if in_string as ::core::ffi::c_int != 0
                    && *last_dash.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int
                    && *last_dash.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '"' as ::core::ffi::c_int
                {
                    l = 2 as ::core::ffi::c_int;
                    off = l;
                } else {
                    l = utfc_ptr2len(last_dash.offset(1 as ::core::ffi::c_int as isize));
                }
                if modifiers != 0 as ::core::ffi::c_int
                    && *last_dash.offset((l + 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        == '>' as ::core::ffi::c_int
                {
                    key = utf_ptr2char(last_dash.offset(off as isize));
                } else {
                    key = get_special_key_code(last_dash.offset(off as isize));
                    if flags & FSK_KEEP_X_KEY as ::core::ffi::c_int == 0 {
                        key = handle_x_keys(key);
                    }
                }
            }
            if key != NUL {
                key = simplify_key(key, &raw mut modifiers);
                if flags & FSK_KEYCODE as ::core::ffi::c_int == 0 {
                    if key == K_BS {
                        key = BS;
                    } else if key == K_DEL
                        || key
                            == -(253 as ::core::ffi::c_int
                                + ((KE_KDEL as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    {
                        key = DEL;
                    }
                }
                if !(key < 0 as ::core::ffi::c_int) {
                    key = extract_modifiers(
                        key,
                        &raw mut modifiers,
                        flags & FSK_SIMPLIFY as ::core::ffi::c_int != 0,
                        did_simplify,
                    );
                }
                *modp = modifiers;
                *srcp = end_of_name;
                return key;
            }
        }
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn extract_modifiers(
    mut key: ::core::ffi::c_int,
    mut modp: *mut ::core::ffi::c_int,
    simplify: bool,
    did_simplify: *mut bool,
) -> ::core::ffi::c_int {
    let mut modifiers: ::core::ffi::c_int = *modp;
    if modifiers & MOD_MASK_SHIFT != 0
        && (key as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && key as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || key as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && key as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint)
    {
        key = if key < 'a' as ::core::ffi::c_int || key > 'z' as ::core::ffi::c_int {
            key
        } else {
            key - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        };
        if modifiers & MOD_MASK_CTRL == 0 {
            modifiers &= !MOD_MASK_SHIFT;
        }
    }
    if modifiers & MOD_MASK_CTRL != 0
        && (key as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && key as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || key as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && key as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint)
    {
        key = if key < 'a' as ::core::ffi::c_int || key > 'z' as ::core::ffi::c_int {
            key
        } else {
            key - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        };
    }
    if simplify as ::core::ffi::c_int != 0
        && modifiers & MOD_MASK_CTRL != 0
        && (key >= '?' as ::core::ffi::c_int && key <= '_' as ::core::ffi::c_int
            || (key as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                && key as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                || key as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                    && key as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint))
    {
        key = (if key < 'a' as ::core::ffi::c_int || key > 'z' as ::core::ffi::c_int {
            key
        } else {
            key - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        }) ^ 0x40 as ::core::ffi::c_int;
        modifiers &= !MOD_MASK_CTRL;
        if key == NUL {
            key = K_ZERO;
        }
        if !did_simplify.is_null() {
            *did_simplify = true_0 != 0;
        }
    }
    *modp = modifiers;
    return key;
}
#[no_mangle]
pub unsafe extern "C" fn find_special_key_in_table(
    mut c: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < ::core::mem::size_of::<[key_name_entry; 187]>()
        .wrapping_div(::core::mem::size_of::<key_name_entry>())
        .wrapping_div(
            (::core::mem::size_of::<[key_name_entry; 187]>()
                .wrapping_rem(::core::mem::size_of::<key_name_entry>())
                == 0) as ::core::ffi::c_int as usize,
        ) as ::core::ffi::c_int
    {
        if c == (*key_names_table.ptr())[i as usize].key
            && !(*key_names_table.ptr())[i as usize].is_alt
        {
            return i;
        }
        i += 1;
    }
    return -1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn get_special_key_code(
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 't' as ::core::ffi::c_int
        && *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '_' as ::core::ffi::c_int
        && *name.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        && *name.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
    {
        return -(*name.offset(2 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            + ((*name.offset(3 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int)
                << 8 as ::core::ffi::c_int));
    }
    let mut name_end: *const ::core::ffi::c_char = name;
    while ascii_isident(*name_end as ::core::ffi::c_int) {
        name_end = name_end.offset(1);
    }
    let mut idx: ::core::ffi::c_int =
        get_special_key_code_hash(name, name_end.offset_from(name) as size_t);
    return if idx >= 0 as ::core::ffi::c_int {
        (*key_names_table.ptr())[idx as usize].key
    } else {
        0 as ::core::ffi::c_int
    };
}
#[no_mangle]
pub unsafe extern "C" fn get_mouse_button(
    mut code: ::core::ffi::c_int,
    mut is_click: *mut bool,
    mut is_drag: *mut bool,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while (*mouse_table.ptr())[i as usize].pseudo_code != 0 {
        if code == (*mouse_table.ptr())[i as usize].pseudo_code {
            *is_click = (*mouse_table.ptr())[i as usize].is_click;
            *is_drag = (*mouse_table.ptr())[i as usize].is_drag;
            return (*mouse_table.ptr())[i as usize].button;
        }
        i += 1;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn replace_termcodes(
    from: *const ::core::ffi::c_char,
    from_len: size_t,
    bufp: *mut *mut ::core::ffi::c_char,
    sid_arg: scid_T,
    flags: ::core::ffi::c_int,
    did_simplify: *mut bool,
    cpo_val: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut dlen: size_t = 0 as size_t;
    let end: *const ::core::ffi::c_char = from
        .offset(from_len as isize)
        .offset(-(1 as ::core::ffi::c_int as isize));
    let do_backslash: bool = vim_strchr(cpo_val, CPO_BSLASH).is_null();
    let do_special: bool = flags & REPTERM_NO_SPECIAL as ::core::ffi::c_int == 0;
    let mut allocated: bool = (*bufp).is_null();
    let buf_len: size_t = if allocated as ::core::ffi::c_int != 0 {
        from_len.wrapping_mul(6 as size_t).wrapping_add(1 as size_t)
    } else {
        128 as size_t
    };
    let mut result: *mut ::core::ffi::c_char = (if allocated as ::core::ffi::c_int != 0 {
        xmalloc(buf_len)
    } else {
        *bufp as *mut ::core::ffi::c_void
    }) as *mut ::core::ffi::c_char;
    let mut src: *const ::core::ffi::c_char = from;
    while src <= end {
        if !allocated && dlen.wrapping_add(64 as size_t) > buf_len {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if do_special as ::core::ffi::c_int != 0
            && (flags & REPTERM_DO_LT as ::core::ffi::c_int != 0
                || end.offset_from(src) >= 3 as isize
                    && strncmp(
                        src,
                        b"<lt>\0".as_ptr() as *const ::core::ffi::c_char,
                        4 as size_t,
                    ) != 0 as ::core::ffi::c_int)
        {
            if end.offset_from(src) >= 4 as isize
                && strncasecmp(
                    src as *mut ::core::ffi::c_char,
                    b"<SID>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    5 as ::core::ffi::c_int as size_t,
                ) == 0 as ::core::ffi::c_int
            {
                if sid_arg < 0 as ::core::ffi::c_int
                    || sid_arg == 0 as ::core::ffi::c_int
                        && (*current_sctx.ptr()).sc_sid <= 0 as ::core::ffi::c_int
                {
                    emsg(gettext(&raw const e_usingsid as *const ::core::ffi::c_char));
                } else {
                    let sid: scid_T = if sid_arg != 0 as ::core::ffi::c_int {
                        sid_arg
                    } else {
                        (*current_sctx.ptr()).sc_sid
                    };
                    src = src.offset(5 as ::core::ffi::c_int as isize);
                    let c2rust_fresh20 = dlen;
                    dlen = dlen.wrapping_add(1);
                    *result.offset(c2rust_fresh20 as isize) = K_SPECIAL as ::core::ffi::c_char;
                    let c2rust_fresh21 = dlen;
                    dlen = dlen.wrapping_add(1);
                    *result.offset(c2rust_fresh21 as isize) = KS_EXTRA as ::core::ffi::c_char;
                    let c2rust_fresh22 = dlen;
                    dlen = dlen.wrapping_add(1);
                    *result.offset(c2rust_fresh22 as isize) =
                        KE_SNR as ::core::ffi::c_int as ::core::ffi::c_char;
                    snprintf(
                        result.offset(dlen as isize),
                        buf_len.wrapping_sub(dlen),
                        b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                        sid,
                    );
                    dlen = dlen.wrapping_add(strlen(result.offset(dlen as isize)));
                    let c2rust_fresh23 = dlen;
                    dlen = dlen.wrapping_add(1);
                    *result.offset(c2rust_fresh23 as isize) = '_' as ::core::ffi::c_char;
                    continue;
                }
            }
            let mut slen: size_t = trans_special(
                &raw mut src,
                (end.offset_from(src) as size_t).wrapping_add(1 as size_t),
                result.offset(dlen as isize),
                FSK_KEYCODE as ::core::ffi::c_int
                    | (if flags & REPTERM_NO_SIMPLIFY as ::core::ffi::c_int != 0 {
                        0 as ::core::ffi::c_int
                    } else {
                        FSK_SIMPLIFY as ::core::ffi::c_int
                    }),
                true_0 != 0,
                did_simplify,
            ) as size_t;
            if slen != 0 {
                dlen = dlen.wrapping_add(slen);
                continue;
            }
        }
        if do_special {
            let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut len: ::core::ffi::c_int = 0;
            if end.offset_from(src) >= 7 as isize
                && strncasecmp(
                    src as *mut ::core::ffi::c_char,
                    b"<Leader>\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    8 as ::core::ffi::c_int as size_t,
                ) == 0 as ::core::ffi::c_int
            {
                len = 8 as ::core::ffi::c_int;
                p = get_var_value(b"g:mapleader\0".as_ptr() as *const ::core::ffi::c_char);
            } else if end.offset_from(src) >= 12 as isize
                && strncasecmp(
                    src as *mut ::core::ffi::c_char,
                    b"<LocalLeader>\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    13 as ::core::ffi::c_int as size_t,
                ) == 0 as ::core::ffi::c_int
            {
                len = 13 as ::core::ffi::c_int;
                p = get_var_value(b"g:maplocalleader\0".as_ptr() as *const ::core::ffi::c_char);
            } else {
                len = 0 as ::core::ffi::c_int;
                p = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            if len != 0 as ::core::ffi::c_int {
                if p.is_null()
                    || *p as ::core::ffi::c_int == NUL
                    || strlen(p) > (8 as ::core::ffi::c_int * 6 as ::core::ffi::c_int) as size_t
                {
                    s = b"\\\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    s = p;
                }
                while *s as ::core::ffi::c_int != NUL {
                    let c2rust_fresh24 = s;
                    s = s.offset(1);
                    let c2rust_fresh25 = dlen;
                    dlen = dlen.wrapping_add(1);
                    *result.offset(c2rust_fresh25 as isize) = *c2rust_fresh24;
                }
                src = src.offset(len as isize);
                continue;
            }
        }
        let mut key: ::core::ffi::c_char = *src;
        if key as ::core::ffi::c_int == Ctrl_V
            || do_backslash as ::core::ffi::c_int != 0
                && key as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
        {
            src = src.offset(1);
            if src > end {
                if flags & REPTERM_FROM_PART as ::core::ffi::c_int != 0 {
                    let c2rust_fresh26 = dlen;
                    dlen = dlen.wrapping_add(1);
                    *result.offset(c2rust_fresh26 as isize) = key;
                }
                break;
            }
        }
        let mut i: ssize_t = utfc_ptr2len_len(
            src,
            end.offset_from(src) as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
        ) as ssize_t;
        while i > 0 as ssize_t {
            if *src as ::core::ffi::c_int == K_SPECIAL as ::core::ffi::c_char as ::core::ffi::c_int
            {
                let c2rust_fresh27 = dlen;
                dlen = dlen.wrapping_add(1);
                *result.offset(c2rust_fresh27 as isize) = K_SPECIAL as ::core::ffi::c_char;
                let c2rust_fresh28 = dlen;
                dlen = dlen.wrapping_add(1);
                *result.offset(c2rust_fresh28 as isize) = KS_SPECIAL as ::core::ffi::c_char;
                let c2rust_fresh29 = dlen;
                dlen = dlen.wrapping_add(1);
                *result.offset(c2rust_fresh29 as isize) = KE_FILLER as ::core::ffi::c_char;
            } else {
                let c2rust_fresh30 = dlen;
                dlen = dlen.wrapping_add(1);
                *result.offset(c2rust_fresh30 as isize) = *src;
            }
            src = src.offset(1);
            i -= 1;
        }
    }
    *result.offset(dlen as isize) = NUL as ::core::ffi::c_char;
    if allocated {
        *bufp = xrealloc(
            result as *mut ::core::ffi::c_void,
            dlen.wrapping_add(1 as size_t),
        ) as *mut ::core::ffi::c_char;
    }
    return *bufp;
}
#[no_mangle]
pub unsafe extern "C" fn add_char2buf(
    mut c: ::core::ffi::c_int,
    mut s: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut temp: [::core::ffi::c_char; 22] = [0; 22];
    let len: ::core::ffi::c_int = utf_char2bytes(c, &raw mut temp as *mut ::core::ffi::c_char);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < len {
        c = temp[i as usize] as uint8_t as ::core::ffi::c_int;
        if c == K_SPECIAL {
            let c2rust_fresh16 = s;
            s = s.offset(1);
            *c2rust_fresh16 = K_SPECIAL as uint8_t as ::core::ffi::c_char;
            let c2rust_fresh17 = s;
            s = s.offset(1);
            *c2rust_fresh17 = KS_SPECIAL as uint8_t as ::core::ffi::c_char;
            let c2rust_fresh18 = s;
            s = s.offset(1);
            *c2rust_fresh18 = KE_FILLER as ::core::ffi::c_char;
        } else {
            let c2rust_fresh19 = s;
            s = s.offset(1);
            *c2rust_fresh19 = c as uint8_t as ::core::ffi::c_char;
        }
        i += 1;
    }
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn vim_strsave_escape_ks(
    mut p: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut res: *mut ::core::ffi::c_char = xmalloc(
        strlen(p)
            .wrapping_mul(4 as size_t)
            .wrapping_add(1 as size_t),
    ) as *mut ::core::ffi::c_char;
    let mut d: *mut ::core::ffi::c_char = res;
    let mut s: *mut ::core::ffi::c_char = p;
    while *s as ::core::ffi::c_int != NUL {
        if *s.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int == K_SPECIAL
            && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            let c2rust_fresh31 = s;
            s = s.offset(1);
            let c2rust_fresh32 = d;
            d = d.offset(1);
            *c2rust_fresh32 = *c2rust_fresh31;
            let c2rust_fresh33 = s;
            s = s.offset(1);
            let c2rust_fresh34 = d;
            d = d.offset(1);
            *c2rust_fresh34 = *c2rust_fresh33;
            let c2rust_fresh35 = s;
            s = s.offset(1);
            let c2rust_fresh36 = d;
            d = d.offset(1);
            *c2rust_fresh36 = *c2rust_fresh35;
        } else {
            d = add_char2buf(utf_ptr2char(s), d);
            s = s.offset(utf_ptr2len(s) as isize);
        }
    }
    *d = NUL as ::core::ffi::c_char;
    return res;
}
#[no_mangle]
pub unsafe extern "C" fn vim_unescape_ks(mut p: *mut ::core::ffi::c_char) {
    let mut s: *mut uint8_t = p as *mut uint8_t;
    let mut d: *mut uint8_t = p as *mut uint8_t;
    while *s as ::core::ffi::c_int != NUL {
        if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == K_SPECIAL
            && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == KS_SPECIAL
            && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == KE_FILLER
        {
            let c2rust_fresh37 = d;
            d = d.offset(1);
            *c2rust_fresh37 = K_SPECIAL as uint8_t;
            s = s.offset(3 as ::core::ffi::c_int as isize);
        } else {
            let c2rust_fresh38 = s;
            s = s.offset(1);
            let c2rust_fresh39 = d;
            d = d.offset(1);
            *c2rust_fresh39 = *c2rust_fresh38;
        }
    }
    *d = NUL as uint8_t;
}
pub const K_SPECIAL: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const KS_SPECIAL: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
pub const KS_EXTRA: ::core::ffi::c_int = 253 as ::core::ffi::c_int;
pub const KS_MODIFIER: ::core::ffi::c_int = 252 as ::core::ffi::c_int;
pub const KS_KEY: ::core::ffi::c_int = 242 as ::core::ffi::c_int;
pub const KE_FILLER: ::core::ffi::c_int = 'X' as ::core::ffi::c_int;
pub const K_ZERO: ::core::ffi::c_int =
    -(255 as ::core::ffi::c_int + (('X' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_UP: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('u' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KUP: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('u' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_DOWN: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('d' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KDOWN: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('d' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_LEFT: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('l' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KLEFT: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('l' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_RIGHT: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('r' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KRIGHT: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('r' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_S_TAB: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('B' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F1: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('1' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F2: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('2' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F3: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('3' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F4: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('4' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F5: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('5' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F6: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('6' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F7: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('7' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F8: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('8' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F9: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('9' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F10: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + ((';' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F11: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('1' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F12: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('2' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F13: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('3' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F14: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('4' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F15: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('5' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F16: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('6' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F17: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('7' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F18: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('8' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F19: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('9' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F20: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('A' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F21: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('B' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F22: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('C' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F23: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('D' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F24: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('E' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F25: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('F' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F26: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('G' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F27: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('H' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F28: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('I' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F29: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('J' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F30: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('K' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F31: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('L' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F32: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('M' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F33: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('N' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F34: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('O' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F35: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('P' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F36: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('Q' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F37: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('R' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F38: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('S' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F39: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('T' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F40: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('U' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F41: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('V' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F42: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('W' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F43: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('X' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F44: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('Y' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F45: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('Z' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F46: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('a' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F47: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('b' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F48: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('c' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F49: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('d' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F50: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('e' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F51: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('f' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F52: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('g' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F53: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('h' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F54: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('i' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F55: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('j' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F56: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('k' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F57: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('l' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F58: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('m' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F59: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('n' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F60: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('o' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F61: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('p' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F62: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('q' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_F63: ::core::ffi::c_int =
    -('F' as ::core::ffi::c_int + (('r' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_S_XF1: ::core::ffi::c_int = -18429;
pub const K_S_XF2: ::core::ffi::c_int = -18685;
pub const K_S_XF3: ::core::ffi::c_int = -18941;
pub const K_S_XF4: ::core::ffi::c_int = -19197;
pub const K_HELP: ::core::ffi::c_int =
    -('%' as ::core::ffi::c_int + (('1' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_UNDO: ::core::ffi::c_int =
    -('&' as ::core::ffi::c_int + (('8' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_FIND: ::core::ffi::c_int =
    -('@' as ::core::ffi::c_int + (('0' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KSELECT: ::core::ffi::c_int =
    -('*' as ::core::ffi::c_int + (('6' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_BS: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('b' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_INS: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('I' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_DEL: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('D' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_HOME: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('h' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KHOME: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('1' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_END: ::core::ffi::c_int =
    -('@' as ::core::ffi::c_int + (('7' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KEND: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('4' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_PAGEUP: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('P' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_PAGEDOWN: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('N' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KPAGEUP: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('3' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KPAGEDOWN: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('5' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KORIGIN: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('2' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KPLUS: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('6' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KMINUS: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('7' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KDIVIDE: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('8' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KMULTIPLY: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('9' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KENTER: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('A' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KPOINT: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('B' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_K0: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('C' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_K1: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('D' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_K2: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('E' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_K3: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('F' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_K4: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('G' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_K5: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('H' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_K6: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('I' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_K7: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('J' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_K8: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('K' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_K9: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('L' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KCOMMA: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('M' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KEQUAL: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('N' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_MOUSE: ::core::ffi::c_int =
    -(251 as ::core::ffi::c_int + (('X' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const MOD_MASK_SHIFT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MOD_MASK_CTRL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const MOD_MASK_ALT: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const MOD_MASK_META: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const MOD_MASK_2CLICK: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const MOD_MASK_3CLICK: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const MOD_MASK_4CLICK: ::core::ffi::c_int = 0x60 as ::core::ffi::c_int;
pub const MOD_MASK_CMD: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const MOD_MASK_MULTI_CLICK: ::core::ffi::c_int =
    MOD_MASK_2CLICK | MOD_MASK_3CLICK | MOD_MASK_4CLICK;
pub const MAX_KEY_NAME_LEN: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
static key_names_table: GlobalCell<[key_name_entry; 187]> = GlobalCell::new([
    key_name_entry {
        key: K_K0,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"k0\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: K_F1,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: K_K1,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"k1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: K_F2,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: K_K2,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"k2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: K_F3,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: K_K3,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"k3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: K_F4,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: K_K4,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"k4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: K_F5,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F5\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: K_K5,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"k5\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: K_F6,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F6\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: K_K6,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"k6\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: K_F7,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F7\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: K_K7,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"k7\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: K_F8,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F8\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: K_K8,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"k8\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: K_F9,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F9\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: K_K9,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"k9\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: NL,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"LF\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: NL,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"NL\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: K_UP,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"Up\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: CAR,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"CR\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: K_BS,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"BS\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: '<' as ::core::ffi::c_int,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"lt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 2 as size_t,
        },
    },
    key_name_entry {
        key: K_F10,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F10\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F20,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F20\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F30,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F30\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F40,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F40\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F50,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F50\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F60,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F60\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_KINS as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"KP0\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F11,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F11\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F21,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F21\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F31,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F31\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F41,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F41\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F51,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F51\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F61,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F61\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_KEND,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"KP1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_XF1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"xF1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F12,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F12\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F22,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F22\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F32,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F32\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F42,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F42\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F52,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F52\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F62,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F62\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_KDOWN,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"KP2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_XF2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"xF2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F13,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F13\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F23,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F23\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F33,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F33\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F43,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F43\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F53,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F53\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F63,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F63\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_KPAGEDOWN,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"KP3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_XF3 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"xF3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F14,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F14\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F24,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F24\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F34,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F34\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F44,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F44\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F54,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F54\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_KLEFT,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"KP4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_XF4 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"xF4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F15,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F15\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F25,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F25\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F35,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F35\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F45,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F45\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F55,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F55\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_KORIGIN,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"KP5\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F16,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F16\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F26,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F26\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F36,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F36\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F46,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F46\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F56,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F56\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_KRIGHT,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"KP6\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F17,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F17\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F27,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F27\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F37,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F37\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F47,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F47\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F57,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F57\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_KHOME,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"KP7\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F18,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F18\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F28,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F28\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F38,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F38\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F48,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F48\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F58,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F58\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_KUP,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"KP8\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F19,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F19\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F29,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F29\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F39,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F39\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F49,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F49\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_F59,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"F59\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_KPAGEUP,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"KP9\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: TAB,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"Tab\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_TAB as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"Tab\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: ESC,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"Esc\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_COMMAND as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"Cmd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_END,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"End\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: CSI,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"CSI\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_DEL,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"Del\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_ZERO,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"Nul\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_KUP,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"kUp\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_XUP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"xUp\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: '|' as ::core::ffi::c_int,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"Bar\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_SNR as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"SNR\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_INS,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"Ins\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 3 as size_t,
        },
    },
    key_name_entry {
        key: K_DOWN,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"Down\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 4 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_DROP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"Drop\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 4 as size_t,
        },
    },
    key_name_entry {
        key: K_FIND,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"Find\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 4 as size_t,
        },
    },
    key_name_entry {
        key: K_HELP,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"Help\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 4 as size_t,
        },
    },
    key_name_entry {
        key: K_HOME,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"Home\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 4 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_KDEL as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"kDel\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 4 as size_t,
        },
    },
    key_name_entry {
        key: K_KEND,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"kEnd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 4 as size_t,
        },
    },
    key_name_entry {
        key: K_LEFT,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"Left\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 4 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_PLUG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"Plug\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 4 as size_t,
        },
    },
    key_name_entry {
        key: K_UNDO,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"Undo\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 4 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_XEND as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"xEnd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 4 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_ZEND as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"zEnd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 4 as size_t,
        },
    },
    key_name_entry {
        key: K_KDOWN,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"kDown\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 5 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_XDOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"xDown\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 5 as size_t,
        },
    },
    key_name_entry {
        key: K_KHOME,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"kHome\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 5 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_XHOME as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"xHome\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 5 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_ZHOME as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"zHome\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 5 as size_t,
        },
    },
    key_name_entry {
        key: K_RIGHT,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"Right\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 5 as size_t,
        },
    },
    key_name_entry {
        key: K_KLEFT,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"kLeft\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 5 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_XLEFT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"xLeft\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 5 as size_t,
        },
    },
    key_name_entry {
        key: CAR,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"Enter\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 5 as size_t,
        },
    },
    key_name_entry {
        key: K_MOUSE,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"Mouse\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 5 as size_t,
        },
    },
    key_name_entry {
        key: K_KDIVIDE,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"KPDiv\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 5 as size_t,
        },
    },
    key_name_entry {
        key: K_KPLUS,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"kPlus\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 5 as size_t,
        },
    },
    key_name_entry {
        key: ' ' as ::core::ffi::c_int,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"Space\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 5 as size_t,
        },
    },
    key_name_entry {
        key: ESC,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"Escape\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 6 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_X1DRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"X1Drag\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 6 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_X2DRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"X2Drag\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 6 as size_t,
        },
    },
    key_name_entry {
        key: K_PAGEUP,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"PageUp\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 6 as size_t,
        },
    },
    key_name_entry {
        key: K_KMINUS,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"kMinus\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 6 as size_t,
        },
    },
    key_name_entry {
        key: K_KRIGHT,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"kRight\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 6 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_XRIGHT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"xRight\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 6 as size_t,
        },
    },
    key_name_entry {
        key: '\\' as ::core::ffi::c_int,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"Bslash\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 6 as size_t,
        },
    },
    key_name_entry {
        key: K_DEL,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"Delete\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 6 as size_t,
        },
    },
    key_name_entry {
        key: K_KSELECT,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"Select\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 6 as size_t,
        },
    },
    key_name_entry {
        key: K_KMULTIPLY,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"KPMult\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 6 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"Ignore\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 6 as size_t,
        },
    },
    key_name_entry {
        key: K_KENTER,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"kEnter\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 6 as size_t,
        },
    },
    key_name_entry {
        key: K_KCOMMA,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"kComma\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 6 as size_t,
        },
    },
    key_name_entry {
        key: K_KPOINT,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"kPoint\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 6 as size_t,
        },
    },
    key_name_entry {
        key: K_KPLUS,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"KPPlus\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 6 as size_t,
        },
    },
    key_name_entry {
        key: K_KEQUAL,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"kEqual\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 6 as size_t,
        },
    },
    key_name_entry {
        key: K_INS,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"Insert\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 6 as size_t,
        },
    },
    key_name_entry {
        key: CAR,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"Return\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 6 as size_t,
        },
    },
    key_name_entry {
        key: K_KPAGEUP,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"kPageUp\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 7 as size_t,
        },
    },
    key_name_entry {
        key: K_KCOMMA,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"KPComma\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 7 as size_t,
        },
    },
    key_name_entry {
        key: K_KENTER,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"KPEnter\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 7 as size_t,
        },
    },
    key_name_entry {
        key: K_KDIVIDE,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"kDivide\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 7 as size_t,
        },
    },
    key_name_entry {
        key: K_KMINUS,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"KPMinus\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 7 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_X1MOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"X1Mouse\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 7 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_X2MOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"X2Mouse\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 7 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_KINS as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"kInsert\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 7 as size_t,
        },
    },
    key_name_entry {
        key: K_KORIGIN,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"kOrigin\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 7 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_MOUSEUP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"MouseUp\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 7 as size_t,
        },
    },
    key_name_entry {
        key: NL,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"NewLine\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 7 as size_t,
        },
    },
    key_name_entry {
        key: K_KEQUAL,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"KPEquals\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 8 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_LEFTDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"LeftDrag\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 8 as size_t,
        },
    },
    key_name_entry {
        key: K_PAGEDOWN,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"PageDown\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 8 as size_t,
        },
    },
    key_name_entry {
        key: NL,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"LineFeed\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 8 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_KDEL as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"KPPeriod\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 8 as size_t,
        },
    },
    key_name_entry {
        key: K_BS,
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"BackSpace\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 9 as size_t,
        },
    },
    key_name_entry {
        key: K_KMULTIPLY,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"kMultiply\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 9 as size_t,
        },
    },
    key_name_entry {
        key: K_KPAGEDOWN,
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"kPageDown\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 9 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_LEFTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"LeftMouse\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 9 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_MOUSEDOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: true_0 != 0,
        name: String_0 {
            data: b"MouseDown\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 9 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"MouseMove\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 9 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_RIGHTDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"RightDrag\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 9 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_X1RELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"X1Release\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 9 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_X2RELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"X2Release\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: 9 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_MIDDLEDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"MiddleDrag\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            size: 10 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_RIGHTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"RightMouse\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            size: 10 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_MIDDLEMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"MiddleMouse\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            size: 11 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_LEFTMOUSE_NM as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"LeftMouseNM\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            size: 11 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_LEFTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"LeftRelease\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            size: 11 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_RIGHTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"RightRelease\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            size: 12 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_LEFTRELEASE_NM as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"LeftReleaseNM\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            size: 13 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_MIDDLERELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"MiddleRelease\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            size: 13 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_MOUSEDOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"ScrollWheelUp\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            size: 13 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_MOUSEUP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"ScrollWheelDown\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            size: 15 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_MOUSERIGHT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"ScrollWheelLeft\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            size: 15 as size_t,
        },
    },
    key_name_entry {
        key: -(253 as ::core::ffi::c_int
            + ((KE_MOUSELEFT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        is_alt: false_0 != 0,
        name: String_0 {
            data: b"ScrollWheelRight\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            size: 16 as size_t,
        },
    },
]);
unsafe extern "C" fn get_special_key_code_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut high: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    match len {
        2 => match *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            48 => {
                low = 0 as ::core::ffi::c_int;
                high = 1 as ::core::ffi::c_int;
            }
            49 => {
                low = 1 as ::core::ffi::c_int;
                high = 3 as ::core::ffi::c_int;
            }
            50 => {
                low = 3 as ::core::ffi::c_int;
                high = 5 as ::core::ffi::c_int;
            }
            51 => {
                low = 5 as ::core::ffi::c_int;
                high = 7 as ::core::ffi::c_int;
            }
            52 => {
                low = 7 as ::core::ffi::c_int;
                high = 9 as ::core::ffi::c_int;
            }
            53 => {
                low = 9 as ::core::ffi::c_int;
                high = 11 as ::core::ffi::c_int;
            }
            54 => {
                low = 11 as ::core::ffi::c_int;
                high = 13 as ::core::ffi::c_int;
            }
            55 => {
                low = 13 as ::core::ffi::c_int;
                high = 15 as ::core::ffi::c_int;
            }
            56 => {
                low = 15 as ::core::ffi::c_int;
                high = 17 as ::core::ffi::c_int;
            }
            57 => {
                low = 17 as ::core::ffi::c_int;
                high = 19 as ::core::ffi::c_int;
            }
            70 | 102 => {
                low = 19 as ::core::ffi::c_int;
                high = 20 as ::core::ffi::c_int;
            }
            76 | 108 => {
                low = 20 as ::core::ffi::c_int;
                high = 21 as ::core::ffi::c_int;
            }
            80 | 112 => {
                low = 21 as ::core::ffi::c_int;
                high = 22 as ::core::ffi::c_int;
            }
            82 | 114 => {
                low = 22 as ::core::ffi::c_int;
                high = 23 as ::core::ffi::c_int;
            }
            83 | 115 => {
                low = 23 as ::core::ffi::c_int;
                high = 24 as ::core::ffi::c_int;
            }
            84 | 116 => {
                low = 24 as ::core::ffi::c_int;
                high = 25 as ::core::ffi::c_int;
            }
            _ => {}
        },
        3 => match *str.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            48 => {
                low = 25 as ::core::ffi::c_int;
                high = 32 as ::core::ffi::c_int;
            }
            49 => {
                low = 32 as ::core::ffi::c_int;
                high = 40 as ::core::ffi::c_int;
            }
            50 => {
                low = 40 as ::core::ffi::c_int;
                high = 48 as ::core::ffi::c_int;
            }
            51 => {
                low = 48 as ::core::ffi::c_int;
                high = 56 as ::core::ffi::c_int;
            }
            52 => {
                low = 56 as ::core::ffi::c_int;
                high = 63 as ::core::ffi::c_int;
            }
            53 => {
                low = 63 as ::core::ffi::c_int;
                high = 69 as ::core::ffi::c_int;
            }
            54 => {
                low = 69 as ::core::ffi::c_int;
                high = 75 as ::core::ffi::c_int;
            }
            55 => {
                low = 75 as ::core::ffi::c_int;
                high = 81 as ::core::ffi::c_int;
            }
            56 => {
                low = 81 as ::core::ffi::c_int;
                high = 87 as ::core::ffi::c_int;
            }
            57 => {
                low = 87 as ::core::ffi::c_int;
                high = 93 as ::core::ffi::c_int;
            }
            66 | 98 => {
                low = 93 as ::core::ffi::c_int;
                high = 95 as ::core::ffi::c_int;
            }
            67 | 99 => {
                low = 95 as ::core::ffi::c_int;
                high = 96 as ::core::ffi::c_int;
            }
            68 | 100 => {
                low = 96 as ::core::ffi::c_int;
                high = 98 as ::core::ffi::c_int;
            }
            73 | 105 => {
                low = 98 as ::core::ffi::c_int;
                high = 99 as ::core::ffi::c_int;
            }
            76 | 108 => {
                low = 99 as ::core::ffi::c_int;
                high = 101 as ::core::ffi::c_int;
            }
            80 | 112 => {
                low = 101 as ::core::ffi::c_int;
                high = 103 as ::core::ffi::c_int;
            }
            82 | 114 => {
                low = 103 as ::core::ffi::c_int;
                high = 105 as ::core::ffi::c_int;
            }
            83 | 115 => {
                low = 105 as ::core::ffi::c_int;
                high = 106 as ::core::ffi::c_int;
            }
            _ => {}
        },
        4 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            68 | 100 => {
                low = 106 as ::core::ffi::c_int;
                high = 108 as ::core::ffi::c_int;
            }
            70 | 102 => {
                low = 108 as ::core::ffi::c_int;
                high = 109 as ::core::ffi::c_int;
            }
            72 | 104 => {
                low = 109 as ::core::ffi::c_int;
                high = 111 as ::core::ffi::c_int;
            }
            75 | 107 => {
                low = 111 as ::core::ffi::c_int;
                high = 113 as ::core::ffi::c_int;
            }
            76 | 108 => {
                low = 113 as ::core::ffi::c_int;
                high = 114 as ::core::ffi::c_int;
            }
            80 | 112 => {
                low = 114 as ::core::ffi::c_int;
                high = 115 as ::core::ffi::c_int;
            }
            85 | 117 => {
                low = 115 as ::core::ffi::c_int;
                high = 116 as ::core::ffi::c_int;
            }
            88 | 120 => {
                low = 116 as ::core::ffi::c_int;
                high = 117 as ::core::ffi::c_int;
            }
            90 | 122 => {
                low = 117 as ::core::ffi::c_int;
                high = 118 as ::core::ffi::c_int;
            }
            _ => {}
        },
        5 => match *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            68 | 100 => {
                low = 118 as ::core::ffi::c_int;
                high = 120 as ::core::ffi::c_int;
            }
            72 | 104 => {
                low = 120 as ::core::ffi::c_int;
                high = 123 as ::core::ffi::c_int;
            }
            73 | 105 => {
                low = 123 as ::core::ffi::c_int;
                high = 124 as ::core::ffi::c_int;
            }
            76 | 108 => {
                low = 124 as ::core::ffi::c_int;
                high = 126 as ::core::ffi::c_int;
            }
            78 | 110 => {
                low = 126 as ::core::ffi::c_int;
                high = 127 as ::core::ffi::c_int;
            }
            79 | 111 => {
                low = 127 as ::core::ffi::c_int;
                high = 128 as ::core::ffi::c_int;
            }
            80 | 112 => {
                low = 128 as ::core::ffi::c_int;
                high = 131 as ::core::ffi::c_int;
            }
            _ => {}
        },
        6 => match *str.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            67 | 99 => {
                low = 131 as ::core::ffi::c_int;
                high = 132 as ::core::ffi::c_int;
            }
            68 | 100 => {
                low = 132 as ::core::ffi::c_int;
                high = 134 as ::core::ffi::c_int;
            }
            71 | 103 => {
                low = 134 as ::core::ffi::c_int;
                high = 135 as ::core::ffi::c_int;
            }
            73 | 105 => {
                low = 135 as ::core::ffi::c_int;
                high = 138 as ::core::ffi::c_int;
            }
            76 | 108 => {
                low = 138 as ::core::ffi::c_int;
                high = 141 as ::core::ffi::c_int;
            }
            77 | 109 => {
                low = 141 as ::core::ffi::c_int;
                high = 142 as ::core::ffi::c_int;
            }
            78 | 110 => {
                low = 142 as ::core::ffi::c_int;
                high = 144 as ::core::ffi::c_int;
            }
            79 | 111 => {
                low = 144 as ::core::ffi::c_int;
                high = 146 as ::core::ffi::c_int;
            }
            80 | 112 => {
                low = 146 as ::core::ffi::c_int;
                high = 147 as ::core::ffi::c_int;
            }
            81 | 113 => {
                low = 147 as ::core::ffi::c_int;
                high = 148 as ::core::ffi::c_int;
            }
            83 | 115 => {
                low = 148 as ::core::ffi::c_int;
                high = 149 as ::core::ffi::c_int;
            }
            84 | 116 => {
                low = 149 as ::core::ffi::c_int;
                high = 150 as ::core::ffi::c_int;
            }
            _ => {}
        },
        7 => match *str.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            65 | 97 => {
                low = 150 as ::core::ffi::c_int;
                high = 151 as ::core::ffi::c_int;
            }
            67 | 99 => {
                low = 151 as ::core::ffi::c_int;
                high = 152 as ::core::ffi::c_int;
            }
            69 | 101 => {
                low = 152 as ::core::ffi::c_int;
                high = 153 as ::core::ffi::c_int;
            }
            73 | 105 => {
                low = 153 as ::core::ffi::c_int;
                high = 154 as ::core::ffi::c_int;
            }
            77 | 109 => {
                low = 154 as ::core::ffi::c_int;
                high = 157 as ::core::ffi::c_int;
            }
            78 | 110 => {
                low = 157 as ::core::ffi::c_int;
                high = 158 as ::core::ffi::c_int;
            }
            82 | 114 => {
                low = 158 as ::core::ffi::c_int;
                high = 159 as ::core::ffi::c_int;
            }
            85 | 117 => {
                low = 159 as ::core::ffi::c_int;
                high = 160 as ::core::ffi::c_int;
            }
            87 | 119 => {
                low = 160 as ::core::ffi::c_int;
                high = 161 as ::core::ffi::c_int;
            }
            _ => {}
        },
        8 => match *str.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            69 | 101 => {
                low = 161 as ::core::ffi::c_int;
                high = 162 as ::core::ffi::c_int;
            }
            70 | 102 => {
                low = 162 as ::core::ffi::c_int;
                high = 163 as ::core::ffi::c_int;
            }
            71 | 103 => {
                low = 163 as ::core::ffi::c_int;
                high = 164 as ::core::ffi::c_int;
            }
            78 | 110 => {
                low = 164 as ::core::ffi::c_int;
                high = 165 as ::core::ffi::c_int;
            }
            80 | 112 => {
                low = 165 as ::core::ffi::c_int;
                high = 166 as ::core::ffi::c_int;
            }
            _ => {}
        },
        9 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            66 | 98 => {
                low = 166 as ::core::ffi::c_int;
                high = 167 as ::core::ffi::c_int;
            }
            75 | 107 => {
                low = 167 as ::core::ffi::c_int;
                high = 169 as ::core::ffi::c_int;
            }
            76 | 108 => {
                low = 169 as ::core::ffi::c_int;
                high = 170 as ::core::ffi::c_int;
            }
            77 | 109 => {
                low = 170 as ::core::ffi::c_int;
                high = 172 as ::core::ffi::c_int;
            }
            82 | 114 => {
                low = 172 as ::core::ffi::c_int;
                high = 173 as ::core::ffi::c_int;
            }
            88 | 120 => {
                low = 173 as ::core::ffi::c_int;
                high = 175 as ::core::ffi::c_int;
            }
            _ => {}
        },
        10 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            77 | 109 => {
                low = 175 as ::core::ffi::c_int;
                high = 176 as ::core::ffi::c_int;
            }
            82 | 114 => {
                low = 176 as ::core::ffi::c_int;
                high = 177 as ::core::ffi::c_int;
            }
            _ => {}
        },
        11 => match *str.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            76 | 108 => {
                low = 177 as ::core::ffi::c_int;
                high = 178 as ::core::ffi::c_int;
            }
            77 | 109 => {
                low = 178 as ::core::ffi::c_int;
                high = 179 as ::core::ffi::c_int;
            }
            82 | 114 => {
                low = 179 as ::core::ffi::c_int;
                high = 180 as ::core::ffi::c_int;
            }
            _ => {}
        },
        12 => {
            low = 180 as ::core::ffi::c_int;
            high = 181 as ::core::ffi::c_int;
        }
        13 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            76 | 108 => {
                low = 181 as ::core::ffi::c_int;
                high = 182 as ::core::ffi::c_int;
            }
            77 | 109 => {
                low = 182 as ::core::ffi::c_int;
                high = 183 as ::core::ffi::c_int;
            }
            83 | 115 => {
                low = 183 as ::core::ffi::c_int;
                high = 184 as ::core::ffi::c_int;
            }
            _ => {}
        },
        15 => match *str.offset(11 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            68 | 100 => {
                low = 184 as ::core::ffi::c_int;
                high = 185 as ::core::ffi::c_int;
            }
            76 | 108 => {
                low = 185 as ::core::ffi::c_int;
                high = 186 as ::core::ffi::c_int;
            }
            _ => {}
        },
        16 => {
            low = 186 as ::core::ffi::c_int;
            high = 187 as ::core::ffi::c_int;
        }
        _ => {}
    }
    let mut i: ::core::ffi::c_int = low;
    while i < high {
        if vim_strnicmp_asc(str, (*key_names_table.ptr())[i as usize].name.data, len) == 0 {
            return i;
        }
        i += 1;
    }
    return -1 as ::core::ffi::c_int;
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const BS: ::core::ffi::c_int = '\u{8}' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
pub const DEL: ::core::ffi::c_int = 0x7f as ::core::ffi::c_int;
pub const CSI: ::core::ffi::c_int = 0x9b as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isident(mut c: ::core::ffi::c_int) -> bool {
    return c as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && c as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || c as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
            && c as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
        || ascii_isdigit(c) as ::core::ffi::c_int != 0
        || c == '_' as ::core::ffi::c_int;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
