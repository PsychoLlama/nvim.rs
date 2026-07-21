use crate::src::nvim::global_cell::GlobalCell;
pub use crate::src::nvim::types::{
    int32_t, schar_T, size_t, uint16_t, uint32_t, uint8_t, utf8proc_int32_t, GraphemeState,
    ScreenCell, ScreenPen, VTerm, VTermAllocatorFunctions, VTermAttr, VTermColor,
    VTermColor_indexed as C2Rust_Unnamed, VTermColor_rgb as C2Rust_Unnamed_0, VTermDamageSize,
    VTermEncoding, VTermEncodingInstance, VTermGlyphInfo, VTermKey, VTermKeyEncodingFlags,
    VTermKeyEncodingStack, VTermLineInfo, VTermModifier, VTermOutputCallback, VTermParserCallbacks,
    VTermParserState, VTermPen, VTermPos, VTermProp, VTermRect, VTermScreen, VTermScreenCallbacks,
    VTermScreenCell, VTermScreenCellAttrs, VTermSelectionCallbacks, VTermSelectionMask, VTermState,
    VTermStateCallbacks, VTermStateFallbacks, VTermStateFields,
    VTermState_mode as C2Rust_Unnamed_7, VTermState_mouse_protocol as C2Rust_Unnamed_8,
    VTermState_saved as C2Rust_Unnamed_5, VTermState_saved_mode as C2Rust_Unnamed_6,
    VTermState_selection as C2Rust_Unnamed_1, VTermState_tmp as C2Rust_Unnamed_2,
    VTermState_tmp_selection as C2Rust_Unnamed_3,
    VTermState_tmp_selection_state as C2Rust_Unnamed_4, VTermStringFragment, VTermTerminator,
    VTermValue, VTerm_mode as C2Rust_Unnamed_14, VTerm_parser as C2Rust_Unnamed_9,
    VTerm_parser_v as C2Rust_Unnamed_10, VTerm_parser_v_csi as C2Rust_Unnamed_13,
    VTerm_parser_v_dcs as C2Rust_Unnamed_11, VTerm_parser_v_osc as C2Rust_Unnamed_12,
};
use ::c2rust_bitfields;
extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn fill_utf8(
        codepoint: ::core::ffi::c_int,
        str: *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn vterm_push_output_bytes(vt: *mut VTerm, bytes: *const ::core::ffi::c_char, len: size_t);
    fn vterm_push_output_sprintf(vt: *mut VTerm, format: *const ::core::ffi::c_char, ...);
    fn vterm_push_output_sprintf_ctrl(
        vt: *mut VTerm,
        ctrl: uint8_t,
        fmt: *const ::core::ffi::c_char,
        ...
    );
}
pub const VTERM_N_DAMAGES: VTermDamageSize = 4;
pub const VTERM_DAMAGE_SCROLL: VTermDamageSize = 3;
pub const VTERM_DAMAGE_SCREEN: VTermDamageSize = 2;
pub const VTERM_DAMAGE_ROW: VTermDamageSize = 1;
pub const VTERM_DAMAGE_CELL: VTermDamageSize = 0;
pub const VTERM_TERMINATOR_ST: VTermTerminator = 1;
pub const VTERM_TERMINATOR_BEL: VTermTerminator = 0;
pub const VTERM_N_PROPS: VTermProp = 12;
pub const VTERM_PROP_SYNCOUTPUT: VTermProp = 11;
pub const VTERM_PROP_THEMEUPDATES: VTermProp = 10;
pub const VTERM_PROP_FOCUSREPORT: VTermProp = 9;
pub const VTERM_PROP_MOUSE: VTermProp = 8;
pub const VTERM_PROP_CURSORSHAPE: VTermProp = 7;
pub const VTERM_PROP_REVERSE: VTermProp = 6;
pub const VTERM_PROP_ICONNAME: VTermProp = 5;
pub const VTERM_PROP_TITLE: VTermProp = 4;
pub const VTERM_PROP_ALTSCREEN: VTermProp = 3;
pub const VTERM_PROP_CURSORBLINK: VTermProp = 2;
pub const VTERM_PROP_CURSORVISIBLE: VTermProp = 1;
pub const VTERM_SELECTION_CUT0: VTermSelectionMask = 16;
pub const VTERM_SELECTION_SELECT: VTermSelectionMask = 8;
pub const VTERM_SELECTION_SECONDARY: VTermSelectionMask = 4;
pub const VTERM_SELECTION_PRIMARY: VTermSelectionMask = 2;
pub const VTERM_SELECTION_CLIPBOARD: VTermSelectionMask = 1;
pub const SELECTION_INVALID: C2Rust_Unnamed_4 = 5;
pub const SELECTION_SET: C2Rust_Unnamed_4 = 4;
pub const SELECTION_SET_INITIAL: C2Rust_Unnamed_4 = 3;
pub const SELECTION_QUERY: C2Rust_Unnamed_4 = 2;
pub const SELECTION_SELECTED: C2Rust_Unnamed_4 = 1;
pub const SELECTION_INITIAL: C2Rust_Unnamed_4 = 0;
pub const MOUSE_RXVT: C2Rust_Unnamed_8 = 3;
pub const MOUSE_SGR: C2Rust_Unnamed_8 = 2;
pub const MOUSE_UTF8: C2Rust_Unnamed_8 = 1;
pub const MOUSE_X10: C2Rust_Unnamed_8 = 0;
pub const VTERM_N_ATTRS: VTermAttr = 16;
pub const VTERM_ATTR_OVERLINE: VTermAttr = 15;
pub const VTERM_ATTR_DIM: VTermAttr = 14;
pub const VTERM_ATTR_URI: VTermAttr = 13;
pub const VTERM_ATTR_BASELINE: VTermAttr = 12;
pub const VTERM_ATTR_SMALL: VTermAttr = 11;
pub const VTERM_ATTR_BACKGROUND: VTermAttr = 10;
pub const VTERM_ATTR_FOREGROUND: VTermAttr = 9;
pub const VTERM_ATTR_FONT: VTermAttr = 8;
pub const VTERM_ATTR_STRIKE: VTermAttr = 7;
pub const VTERM_ATTR_CONCEAL: VTermAttr = 6;
pub const VTERM_ATTR_REVERSE: VTermAttr = 5;
pub const VTERM_ATTR_BLINK: VTermAttr = 4;
pub const VTERM_ATTR_ITALIC: VTermAttr = 3;
pub const VTERM_ATTR_UNDERLINE: VTermAttr = 2;
pub const VTERM_ATTR_BOLD: VTermAttr = 1;
pub const SOS: VTermParserState = 10;
pub const PM: VTermParserState = 9;
pub const APC: VTermParserState = 8;
pub const DCS_VTERM: VTermParserState = 7;
pub const OSC: VTermParserState = 6;
pub const OSC_COMMAND: VTermParserState = 5;
pub const DCS_COMMAND: VTermParserState = 4;
pub const CSI_INTERMED: VTermParserState = 3;
pub const CSI_ARGS: VTermParserState = 2;
pub const CSI_LEADER: VTermParserState = 1;
pub const NORMAL: VTermParserState = 0;
pub const VTERM_ALL_MODS_MASK: VTermModifier = 7;
pub const VTERM_MOD_CTRL: VTermModifier = 4;
pub const VTERM_MOD_ALT: VTermModifier = 2;
pub const VTERM_MOD_SHIFT: VTermModifier = 1;
pub const VTERM_MOD_NONE: VTermModifier = 0;
pub const VTERM_N_KEYS: VTermKey = 530;
pub const VTERM_KEY_MAX: VTermKey = 530;
pub const VTERM_KEY_KP_EQUAL: VTermKey = 529;
pub const VTERM_KEY_KP_ENTER: VTermKey = 528;
pub const VTERM_KEY_KP_DIVIDE: VTermKey = 527;
pub const VTERM_KEY_KP_PERIOD: VTermKey = 526;
pub const VTERM_KEY_KP_MINUS: VTermKey = 525;
pub const VTERM_KEY_KP_COMMA: VTermKey = 524;
pub const VTERM_KEY_KP_PLUS: VTermKey = 523;
pub const VTERM_KEY_KP_MULT: VTermKey = 522;
pub const VTERM_KEY_KP_9: VTermKey = 521;
pub const VTERM_KEY_KP_8: VTermKey = 520;
pub const VTERM_KEY_KP_7: VTermKey = 519;
pub const VTERM_KEY_KP_6: VTermKey = 518;
pub const VTERM_KEY_KP_5: VTermKey = 517;
pub const VTERM_KEY_KP_4: VTermKey = 516;
pub const VTERM_KEY_KP_3: VTermKey = 515;
pub const VTERM_KEY_KP_2: VTermKey = 514;
pub const VTERM_KEY_KP_1: VTermKey = 513;
pub const VTERM_KEY_KP_0: VTermKey = 512;
pub const VTERM_KEY_FUNCTION_MAX: VTermKey = 511;
pub const VTERM_KEY_FUNCTION_0: VTermKey = 256;
pub const VTERM_KEY_PAGEDOWN: VTermKey = 14;
pub const VTERM_KEY_PAGEUP: VTermKey = 13;
pub const VTERM_KEY_END: VTermKey = 12;
pub const VTERM_KEY_HOME: VTermKey = 11;
pub const VTERM_KEY_DEL: VTermKey = 10;
pub const VTERM_KEY_INS: VTermKey = 9;
pub const VTERM_KEY_RIGHT: VTermKey = 8;
pub const VTERM_KEY_LEFT: VTermKey = 7;
pub const VTERM_KEY_DOWN: VTermKey = 6;
pub const VTERM_KEY_UP: VTermKey = 5;
pub const VTERM_KEY_ESCAPE: VTermKey = 4;
pub const VTERM_KEY_BACKSPACE: VTermKey = 3;
pub const VTERM_KEY_TAB: VTermKey = 2;
pub const VTERM_KEY_ENTER: VTermKey = 1;
pub const VTERM_KEY_NONE: VTermKey = 0;
pub const C1_CSI: C2Rust_Unnamed_16 = 155;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct keycodes_s {
    pub type_0: C2Rust_Unnamed_15,
    pub literal: ::core::ffi::c_int,
    pub csinum: ::core::ffi::c_int,
}
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const KEYCODE_KEYPAD: C2Rust_Unnamed_15 = 8;
pub const KEYCODE_CSINUM: C2Rust_Unnamed_15 = 7;
pub const KEYCODE_CSI_CURSOR: C2Rust_Unnamed_15 = 6;
pub const KEYCODE_CSI: C2Rust_Unnamed_15 = 5;
pub const KEYCODE_SS3: C2Rust_Unnamed_15 = 4;
pub const KEYCODE_ENTER: C2Rust_Unnamed_15 = 3;
pub const KEYCODE_TAB: C2Rust_Unnamed_15 = 2;
pub const KEYCODE_LITERAL: C2Rust_Unnamed_15 = 1;
pub const KEYCODE_NONE: C2Rust_Unnamed_15 = 0;
pub const C1_SS3: C2Rust_Unnamed_16 = 143;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const C1_OSC: C2Rust_Unnamed_16 = 157;
pub const C1_ST: C2Rust_Unnamed_16 = 156;
pub const C1_DCS: C2Rust_Unnamed_16 = 144;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 77] = unsafe {
    ::core::mem::transmute::<[u8; 77], [::core::ffi::c_char; 77]>(
        *b"VTermKeyEncodingFlags vterm_state_get_key_encoding_flags(const VTermState *)\0",
    )
};
pub const ESC_S: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"\x1B\0") };
pub const BUFIDX_PRIMARY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const BUFIDX_ALTSCREEN: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
unsafe extern "C" fn vterm_state_get_key_encoding_flags(
    mut state: *const VTermState,
) -> VTermKeyEncodingFlags {
    let mut screen: ::core::ffi::c_int = if (*state).mode.alt_screen() as ::core::ffi::c_int != 0 {
        BUFIDX_ALTSCREEN
    } else {
        BUFIDX_PRIMARY
    };
    let mut stack: *const VTermKeyEncodingStack = (&raw const (*state).key_encoding_stacks
        as *const VTermKeyEncodingStack)
        .offset(screen as isize);
    '_c2rust_label: {
        if (*stack).size as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"stack->size > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/vterm/keyboard.rs\0".as_ptr() as *const ::core::ffi::c_char,
                15 as ::core::ffi::c_uint,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    return (*stack).items
        [((*stack).size as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as usize];
}
#[no_mangle]
pub unsafe extern "C" fn vterm_keyboard_unichar(
    mut vt: *mut VTerm,
    mut c: uint32_t,
    mut mod_0: VTermModifier,
) {
    let mut passthru: bool = false_0 != 0;
    if c == ' ' as uint32_t {
        passthru = mod_0 as ::core::ffi::c_uint
            == VTERM_MOD_NONE as ::core::ffi::c_int as ::core::ffi::c_uint;
    } else {
        passthru = mod_0 as ::core::ffi::c_uint
            & !(VTERM_MOD_SHIFT as ::core::ffi::c_int) as ::core::ffi::c_uint
            == 0 as ::core::ffi::c_uint;
    }
    if passthru {
        let mut str: [::core::ffi::c_char; 6] = [0; 6];
        let mut seqlen: ::core::ffi::c_int = fill_utf8(
            c as ::core::ffi::c_int,
            &raw mut str as *mut ::core::ffi::c_char,
        );
        vterm_push_output_bytes(
            vt,
            &raw mut str as *mut ::core::ffi::c_char,
            seqlen as size_t,
        );
        return;
    }
    let mut flags: VTermKeyEncodingFlags = vterm_state_get_key_encoding_flags((*vt).state);
    if flags.disambiguate() {
        if c >= 'A' as uint32_t && c <= 'Z' as uint32_t {
            c = c.wrapping_add(('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int) as uint32_t);
            mod_0 = (mod_0 as ::core::ffi::c_uint
                | VTERM_MOD_SHIFT as ::core::ffi::c_int as ::core::ffi::c_uint)
                as VTermModifier;
        }
        vterm_push_output_sprintf_ctrl(
            vt,
            C1_CSI as ::core::ffi::c_int as uint8_t,
            b"%d;%du\0".as_ptr() as *const ::core::ffi::c_char,
            c,
            (mod_0 as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint),
        );
        return;
    }
    if mod_0 as ::core::ffi::c_uint & VTERM_MOD_CTRL as ::core::ffi::c_int as ::core::ffi::c_uint
        != 0
    {
        match c {
            50 | 32 => {
                c = 0 as uint32_t;
            }
            51 | 52 | 53 | 54 | 55 => {
                c = (0x1b as uint32_t)
                    .wrapping_add(c)
                    .wrapping_sub('3' as uint32_t);
            }
            56 => {
                c = 0x7f as uint32_t;
            }
            47 => {
                c = 0x1f as uint32_t;
            }
            _ => {
                if c >= '@' as uint32_t && c <= 0x7f as uint32_t {
                    c &= 0x1f as uint32_t;
                }
            }
        }
    }
    vterm_push_output_sprintf(
        vt,
        b"%s%c\0".as_ptr() as *const ::core::ffi::c_char,
        if mod_0 as ::core::ffi::c_uint & VTERM_MOD_ALT as ::core::ffi::c_int as ::core::ffi::c_uint
            != 0
        {
            ESC_S.as_ptr()
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
        c,
    );
}
static keycodes: GlobalCell<[keycodes_s; 15]> = GlobalCell::new([
    keycodes_s {
        type_0: KEYCODE_NONE,
        literal: NUL,
        csinum: 0 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_ENTER,
        literal: '\r' as ::core::ffi::c_int,
        csinum: 0 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_TAB,
        literal: '\t' as ::core::ffi::c_int,
        csinum: 0 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_LITERAL,
        literal: '\u{7f}' as ::core::ffi::c_int,
        csinum: 0 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_LITERAL,
        literal: '\u{1b}' as ::core::ffi::c_int,
        csinum: 0 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_CSI_CURSOR,
        literal: 'A' as ::core::ffi::c_int,
        csinum: 0 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_CSI_CURSOR,
        literal: 'B' as ::core::ffi::c_int,
        csinum: 0 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_CSI_CURSOR,
        literal: 'D' as ::core::ffi::c_int,
        csinum: 0 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_CSI_CURSOR,
        literal: 'C' as ::core::ffi::c_int,
        csinum: 0 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_CSINUM,
        literal: '~' as ::core::ffi::c_int,
        csinum: 2 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_CSINUM,
        literal: '~' as ::core::ffi::c_int,
        csinum: 3 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_CSI_CURSOR,
        literal: 'H' as ::core::ffi::c_int,
        csinum: 0 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_CSI_CURSOR,
        literal: 'F' as ::core::ffi::c_int,
        csinum: 0 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_CSINUM,
        literal: '~' as ::core::ffi::c_int,
        csinum: 5 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_CSINUM,
        literal: '~' as ::core::ffi::c_int,
        csinum: 6 as ::core::ffi::c_int,
    },
]);
static keycodes_fn: GlobalCell<[keycodes_s; 13]> = GlobalCell::new([
    keycodes_s {
        type_0: KEYCODE_NONE,
        literal: NUL,
        csinum: 0 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_SS3,
        literal: 'P' as ::core::ffi::c_int,
        csinum: 0 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_SS3,
        literal: 'Q' as ::core::ffi::c_int,
        csinum: 0 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_SS3,
        literal: 'R' as ::core::ffi::c_int,
        csinum: 0 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_SS3,
        literal: 'S' as ::core::ffi::c_int,
        csinum: 0 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_CSINUM,
        literal: '~' as ::core::ffi::c_int,
        csinum: 15 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_CSINUM,
        literal: '~' as ::core::ffi::c_int,
        csinum: 17 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_CSINUM,
        literal: '~' as ::core::ffi::c_int,
        csinum: 18 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_CSINUM,
        literal: '~' as ::core::ffi::c_int,
        csinum: 19 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_CSINUM,
        literal: '~' as ::core::ffi::c_int,
        csinum: 20 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_CSINUM,
        literal: '~' as ::core::ffi::c_int,
        csinum: 21 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_CSINUM,
        literal: '~' as ::core::ffi::c_int,
        csinum: 23 as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_CSINUM,
        literal: '~' as ::core::ffi::c_int,
        csinum: 24 as ::core::ffi::c_int,
    },
]);
static keycodes_kp: GlobalCell<[keycodes_s; 18]> = GlobalCell::new([
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: '0' as ::core::ffi::c_int,
        csinum: 'p' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: '1' as ::core::ffi::c_int,
        csinum: 'q' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: '2' as ::core::ffi::c_int,
        csinum: 'r' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: '3' as ::core::ffi::c_int,
        csinum: 's' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: '4' as ::core::ffi::c_int,
        csinum: 't' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: '5' as ::core::ffi::c_int,
        csinum: 'u' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: '6' as ::core::ffi::c_int,
        csinum: 'v' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: '7' as ::core::ffi::c_int,
        csinum: 'w' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: '8' as ::core::ffi::c_int,
        csinum: 'x' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: '9' as ::core::ffi::c_int,
        csinum: 'y' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: '*' as ::core::ffi::c_int,
        csinum: 'j' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: '+' as ::core::ffi::c_int,
        csinum: 'k' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: ',' as ::core::ffi::c_int,
        csinum: 'l' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: '-' as ::core::ffi::c_int,
        csinum: 'm' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: '.' as ::core::ffi::c_int,
        csinum: 'n' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: '/' as ::core::ffi::c_int,
        csinum: 'o' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: '\n' as ::core::ffi::c_int,
        csinum: 'M' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: '=' as ::core::ffi::c_int,
        csinum: 'X' as ::core::ffi::c_int,
    },
]);
static keycodes_kp_csiu: GlobalCell<[keycodes_s; 18]> = GlobalCell::new([
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: 57399 as ::core::ffi::c_int,
        csinum: 'p' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: 57400 as ::core::ffi::c_int,
        csinum: 'q' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: 57401 as ::core::ffi::c_int,
        csinum: 'r' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: 57402 as ::core::ffi::c_int,
        csinum: 's' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: 57403 as ::core::ffi::c_int,
        csinum: 't' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: 57404 as ::core::ffi::c_int,
        csinum: 'u' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: 57405 as ::core::ffi::c_int,
        csinum: 'v' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: 57406 as ::core::ffi::c_int,
        csinum: 'w' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: 57407 as ::core::ffi::c_int,
        csinum: 'x' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: 57408 as ::core::ffi::c_int,
        csinum: 'y' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: 57411 as ::core::ffi::c_int,
        csinum: 'j' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: 57413 as ::core::ffi::c_int,
        csinum: 'k' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: 57416 as ::core::ffi::c_int,
        csinum: 'l' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: 57412 as ::core::ffi::c_int,
        csinum: 'm' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: 57409 as ::core::ffi::c_int,
        csinum: 'n' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: 57410 as ::core::ffi::c_int,
        csinum: 'o' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: 57414 as ::core::ffi::c_int,
        csinum: 'M' as ::core::ffi::c_int,
    },
    keycodes_s {
        type_0: KEYCODE_KEYPAD,
        literal: 57415 as ::core::ffi::c_int,
        csinum: 'X' as ::core::ffi::c_int,
    },
]);
#[no_mangle]
pub unsafe extern "C" fn vterm_keyboard_key(
    mut vt: *mut VTerm,
    mut key: VTermKey,
    mut mod_0: VTermModifier,
) {
    if key as ::core::ffi::c_uint == VTERM_KEY_NONE as ::core::ffi::c_int as ::core::ffi::c_uint {
        return;
    }
    let mut flags: VTermKeyEncodingFlags = vterm_state_get_key_encoding_flags((*vt).state);
    let mut k: keycodes_s = keycodes_s {
        type_0: KEYCODE_NONE,
        literal: 0,
        csinum: 0,
    };
    if (key as ::core::ffi::c_uint)
        < VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if key as usize
            >= ::core::mem::size_of::<[keycodes_s; 15]>()
                .wrapping_div(::core::mem::size_of::<keycodes_s>())
        {
            return;
        }
        k = (*keycodes.ptr())[key as usize];
    } else if key as ::core::ffi::c_uint
        >= VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int as ::core::ffi::c_uint
        && key as ::core::ffi::c_uint
            <= VTERM_KEY_FUNCTION_MAX as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (key as ::core::ffi::c_uint)
            .wrapping_sub(VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int as ::core::ffi::c_uint)
            as usize
            >= ::core::mem::size_of::<[keycodes_s; 13]>()
                .wrapping_div(::core::mem::size_of::<keycodes_s>())
        {
            return;
        }
        k = (*keycodes_fn.ptr())[(key as ::core::ffi::c_uint)
            .wrapping_sub(VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int as ::core::ffi::c_uint)
            as usize];
    } else if key as ::core::ffi::c_uint
        >= VTERM_KEY_KP_0 as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (key as ::core::ffi::c_uint)
            .wrapping_sub(VTERM_KEY_KP_0 as ::core::ffi::c_int as ::core::ffi::c_uint)
            as usize
            >= ::core::mem::size_of::<[keycodes_s; 18]>()
                .wrapping_div(::core::mem::size_of::<keycodes_s>())
        {
            return;
        }
        if flags.disambiguate() {
            k = (*keycodes_kp_csiu.ptr())[(key as ::core::ffi::c_uint)
                .wrapping_sub(VTERM_KEY_KP_0 as ::core::ffi::c_int as ::core::ffi::c_uint)
                as usize];
        } else {
            k = (*keycodes_kp.ptr())[(key as ::core::ffi::c_uint)
                .wrapping_sub(VTERM_KEY_KP_0 as ::core::ffi::c_int as ::core::ffi::c_uint)
                as usize];
        }
    }
    's_322: {
        '_case_CSI: {
            '_case_LITERAL: {
                match k.type_0 as ::core::ffi::c_uint {
                    2 => {
                        if flags.disambiguate() {
                            break '_case_LITERAL;
                        } else if mod_0 as ::core::ffi::c_uint
                            == VTERM_MOD_SHIFT as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            vterm_push_output_sprintf_ctrl(
                                vt,
                                C1_CSI as ::core::ffi::c_int as uint8_t,
                                b"Z\0".as_ptr() as *const ::core::ffi::c_char,
                            );
                            break 's_322;
                        } else if mod_0 as ::core::ffi::c_uint
                            & VTERM_MOD_SHIFT as ::core::ffi::c_int as ::core::ffi::c_uint
                            != 0
                        {
                            vterm_push_output_sprintf_ctrl(
                                vt,
                                C1_CSI as ::core::ffi::c_int as uint8_t,
                                b"1;%dZ\0".as_ptr() as *const ::core::ffi::c_char,
                                (mod_0 as ::core::ffi::c_uint)
                                    .wrapping_add(1 as ::core::ffi::c_uint),
                            );
                            break 's_322;
                        } else {
                            break '_case_LITERAL;
                        }
                    }
                    3 => {
                        if (*(*vt).state).mode.newline() != 0 {
                            vterm_push_output_sprintf(
                                vt,
                                b"\r\n\0".as_ptr() as *const ::core::ffi::c_char,
                            );
                            break 's_322;
                        } else {
                            break '_case_LITERAL;
                        }
                    }
                    1 => {
                        break '_case_LITERAL;
                    }
                    4 => {}
                    5 => {
                        break '_case_CSI;
                    }
                    7 => {
                        if mod_0 as ::core::ffi::c_uint == 0 as ::core::ffi::c_uint {
                            vterm_push_output_sprintf_ctrl(
                                vt,
                                C1_CSI as ::core::ffi::c_int as uint8_t,
                                b"%d%c\0".as_ptr() as *const ::core::ffi::c_char,
                                k.csinum,
                                k.literal,
                            );
                        } else {
                            vterm_push_output_sprintf_ctrl(
                                vt,
                                C1_CSI as ::core::ffi::c_int as uint8_t,
                                b"%d;%d%c\0".as_ptr() as *const ::core::ffi::c_char,
                                k.csinum,
                                (mod_0 as ::core::ffi::c_uint)
                                    .wrapping_add(1 as ::core::ffi::c_uint),
                                k.literal,
                            );
                        }
                        break 's_322;
                    }
                    6 => {
                        if (*(*vt).state).mode.cursor() == 0 {
                            break '_case_CSI;
                        }
                    }
                    8 => {
                        if (*(*vt).state).mode.keypad() != 0 {
                            k.literal = k.csinum;
                        } else {
                            break '_case_LITERAL;
                        }
                    }
                    0 | _ => {
                        break 's_322;
                    }
                }
                if mod_0 as ::core::ffi::c_uint == 0 as ::core::ffi::c_uint {
                    vterm_push_output_sprintf_ctrl(
                        vt,
                        C1_SS3 as ::core::ffi::c_int as uint8_t,
                        b"%c\0".as_ptr() as *const ::core::ffi::c_char,
                        k.literal,
                    );
                    break 's_322;
                } else {
                    break '_case_CSI;
                }
            }
            if flags.disambiguate() {
                match key as ::core::ffi::c_uint {
                    2 | 1 | 3 => {
                        flags.set_disambiguate(
                            (mod_0 as ::core::ffi::c_uint
                                != VTERM_MOD_NONE as ::core::ffi::c_int as ::core::ffi::c_uint)
                                as bool,
                        );
                    }
                    _ => {}
                }
            }
            if flags.disambiguate() {
                vterm_push_output_sprintf_ctrl(
                    vt,
                    C1_CSI as ::core::ffi::c_int as uint8_t,
                    b"%d;%du\0".as_ptr() as *const ::core::ffi::c_char,
                    k.literal,
                    (mod_0 as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint),
                );
            } else {
                vterm_push_output_sprintf(
                    vt,
                    if mod_0 as ::core::ffi::c_uint
                        & VTERM_MOD_ALT as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                    {
                        b"\x1B%c\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b"%c\0".as_ptr() as *const ::core::ffi::c_char
                    },
                    k.literal,
                );
            }
            break 's_322;
        }
        if mod_0 as ::core::ffi::c_uint == 0 as ::core::ffi::c_uint {
            vterm_push_output_sprintf_ctrl(
                vt,
                C1_CSI as ::core::ffi::c_int as uint8_t,
                b"%c\0".as_ptr() as *const ::core::ffi::c_char,
                k.literal,
            );
        } else {
            vterm_push_output_sprintf_ctrl(
                vt,
                C1_CSI as ::core::ffi::c_int as uint8_t,
                b"1;%d%c\0".as_ptr() as *const ::core::ffi::c_char,
                (mod_0 as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint),
                k.literal,
            );
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn vterm_keyboard_start_paste(mut vt: *mut VTerm) {
    if (*(*vt).state).mode.bracketpaste() != 0 {
        vterm_push_output_sprintf_ctrl(
            vt,
            C1_CSI as ::core::ffi::c_int as uint8_t,
            b"200~\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn vterm_keyboard_end_paste(mut vt: *mut VTerm) {
    if (*(*vt).state).mode.bracketpaste() != 0 {
        vterm_push_output_sprintf_ctrl(
            vt,
            C1_CSI as ::core::ffi::c_int as uint8_t,
            b"201~\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
}
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
