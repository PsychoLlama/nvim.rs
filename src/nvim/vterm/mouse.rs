pub use crate::src::nvim::types::{
    int32_t, schar_T, size_t, uint16_t, uint32_t, uint64_t, uint8_t, utf8proc_int32_t,
    GraphemeState, ScreenCell, ScreenPen, VTerm, VTermAllocatorFunctions, VTermAttr, VTermColor,
    VTermColor_indexed as C2Rust_Unnamed, VTermColor_rgb as C2Rust_Unnamed_0, VTermDamageSize,
    VTermEncoding, VTermEncodingInstance, VTermGlyphInfo, VTermKeyEncodingFlags,
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
    fn fill_utf8(
        codepoint: ::core::ffi::c_int,
        str: *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
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
pub const C1_CSI: C2Rust_Unnamed_15 = 155;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const C1_OSC: C2Rust_Unnamed_15 = 157;
pub const C1_ST: C2Rust_Unnamed_15 = 156;
pub const C1_DCS: C2Rust_Unnamed_15 = 144;
pub const C1_SS3: C2Rust_Unnamed_15 = 143;
pub const MOUSE_WANT_DRAG: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MOUSE_WANT_MOVE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
unsafe extern "C" fn output_mouse(
    mut state: *mut VTermState,
    mut code: ::core::ffi::c_int,
    mut pressed: ::core::ffi::c_int,
    mut modifiers: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut row: ::core::ffi::c_int,
) {
    modifiers <<= 2 as ::core::ffi::c_int;
    match (*state).mouse_protocol as ::core::ffi::c_uint {
        0 => {
            if col + 0x21 as ::core::ffi::c_int > 0xff as ::core::ffi::c_int {
                col = 0xff as ::core::ffi::c_int - 0x21 as ::core::ffi::c_int;
            }
            if row + 0x21 as ::core::ffi::c_int > 0xff as ::core::ffi::c_int {
                row = 0xff as ::core::ffi::c_int - 0x21 as ::core::ffi::c_int;
            }
            if pressed == 0 {
                code = 3 as ::core::ffi::c_int;
            }
            if code & 0x80 as ::core::ffi::c_int == 0 {
                vterm_push_output_sprintf_ctrl(
                    (*state).vt,
                    C1_CSI as ::core::ffi::c_int as uint8_t,
                    b"M%c%c%c\0".as_ptr() as *const ::core::ffi::c_char,
                    (code | modifiers) + 0x20 as ::core::ffi::c_int,
                    col + 0x21 as ::core::ffi::c_int,
                    row + 0x21 as ::core::ffi::c_int,
                );
            }
        }
        1 => {
            let mut utf8: [::core::ffi::c_char; 18] = [0; 18];
            let mut len: size_t = 0 as size_t;
            if pressed == 0 {
                code = 3 as ::core::ffi::c_int;
            }
            len = len.wrapping_add(fill_utf8(
                (code | modifiers) + 0x20 as ::core::ffi::c_int,
                (&raw mut utf8 as *mut ::core::ffi::c_char).offset(len as isize),
            ) as size_t);
            len = len.wrapping_add(fill_utf8(
                col + 0x21 as ::core::ffi::c_int,
                (&raw mut utf8 as *mut ::core::ffi::c_char).offset(len as isize),
            ) as size_t);
            len = len.wrapping_add(fill_utf8(
                row + 0x21 as ::core::ffi::c_int,
                (&raw mut utf8 as *mut ::core::ffi::c_char).offset(len as isize),
            ) as size_t);
            utf8[len as usize] = 0 as ::core::ffi::c_char;
            vterm_push_output_sprintf_ctrl(
                (*state).vt,
                C1_CSI as ::core::ffi::c_int as uint8_t,
                b"M%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut utf8 as *mut ::core::ffi::c_char,
            );
        }
        2 => {
            vterm_push_output_sprintf_ctrl(
                (*state).vt,
                C1_CSI as ::core::ffi::c_int as uint8_t,
                b"<%d;%d;%d%c\0".as_ptr() as *const ::core::ffi::c_char,
                code | modifiers,
                col + 1 as ::core::ffi::c_int,
                row + 1 as ::core::ffi::c_int,
                if pressed != 0 {
                    'M' as ::core::ffi::c_int
                } else {
                    'm' as ::core::ffi::c_int
                },
            );
        }
        3 => {
            if pressed == 0 {
                code = 3 as ::core::ffi::c_int;
            }
            vterm_push_output_sprintf_ctrl(
                (*state).vt,
                C1_CSI as ::core::ffi::c_int as uint8_t,
                b"%d;%d;%dM\0".as_ptr() as *const ::core::ffi::c_char,
                code | modifiers,
                col + 1 as ::core::ffi::c_int,
                row + 1 as ::core::ffi::c_int,
            );
        }
        _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn vterm_mouse_move(
    mut vt: *mut VTerm,
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut mod_0: VTermModifier,
) {
    let mut state: *mut VTermState = (*vt).state;
    if col == (*state).mouse_col && row == (*state).mouse_row {
        return;
    }
    (*state).mouse_col = col;
    (*state).mouse_row = row;
    if (*state).mouse_flags & MOUSE_WANT_DRAG != 0 && (*state).mouse_buttons != 0
        || (*state).mouse_flags & MOUSE_WANT_MOVE != 0
    {
        if (*state).mouse_buttons != 0 {
            let mut button: ::core::ffi::c_int =
                ((*state).mouse_buttons as uint64_t).trailing_zeros() as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int;
            if button < 4 as ::core::ffi::c_int {
                output_mouse(
                    state,
                    button - 1 as ::core::ffi::c_int + 0x20 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                    mod_0 as ::core::ffi::c_int,
                    col,
                    row,
                );
            } else if button >= 8 as ::core::ffi::c_int && button < 12 as ::core::ffi::c_int {
                output_mouse(
                    state,
                    button - 8 as ::core::ffi::c_int
                        + 0x80 as ::core::ffi::c_int
                        + 0x20 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                    mod_0 as ::core::ffi::c_int,
                    col,
                    row,
                );
            }
        } else {
            output_mouse(
                state,
                3 as ::core::ffi::c_int + 0x20 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
                mod_0 as ::core::ffi::c_int,
                col,
                row,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn vterm_mouse_button(
    mut vt: *mut VTerm,
    mut button: ::core::ffi::c_int,
    mut pressed: bool,
    mut mod_0: VTermModifier,
) {
    let mut state: *mut VTermState = (*vt).state;
    let mut old_buttons: ::core::ffi::c_int = (*state).mouse_buttons;
    if button > 0 as ::core::ffi::c_int && button <= 3 as ::core::ffi::c_int
        || button >= 8 as ::core::ffi::c_int && button <= 11 as ::core::ffi::c_int
    {
        if pressed {
            (*state).mouse_buttons |= (1 as ::core::ffi::c_int) << button - 1 as ::core::ffi::c_int;
        } else {
            (*state).mouse_buttons &=
                !((1 as ::core::ffi::c_int) << button - 1 as ::core::ffi::c_int);
        }
    }
    if (*state).mouse_buttons == old_buttons
        && (button < 4 as ::core::ffi::c_int || button > 7 as ::core::ffi::c_int)
    {
        return;
    }
    if (*state).mouse_flags == 0 {
        return;
    }
    if button < 4 as ::core::ffi::c_int {
        output_mouse(
            state,
            button - 1 as ::core::ffi::c_int,
            pressed as ::core::ffi::c_int,
            mod_0 as ::core::ffi::c_int,
            (*state).mouse_col,
            (*state).mouse_row,
        );
    } else if button < 8 as ::core::ffi::c_int {
        output_mouse(
            state,
            button - 4 as ::core::ffi::c_int + 0x40 as ::core::ffi::c_int,
            pressed as ::core::ffi::c_int,
            mod_0 as ::core::ffi::c_int,
            (*state).mouse_col,
            (*state).mouse_row,
        );
    } else if button < 12 as ::core::ffi::c_int {
        output_mouse(
            state,
            button - 8 as ::core::ffi::c_int + 0x80 as ::core::ffi::c_int,
            pressed as ::core::ffi::c_int,
            mod_0 as ::core::ffi::c_int,
            (*state).mouse_col,
            (*state).mouse_row,
        );
    }
}
