use crate::src::nvim::global_cell::GlobalCell;
pub use crate::src::nvim::types::{
    int32_t, schar_T, size_t, uint16_t, uint32_t, uint8_t, utf8proc_int32_t, GraphemeState,
    ScreenCell, ScreenPen, VTerm, VTermAllocatorFunctions, VTermAttr, VTermColor,
    VTermColor_indexed as C2Rust_Unnamed, VTermColor_rgb as C2Rust_Unnamed_0, VTermDamageSize,
    VTermEncoding, VTermEncodingInstance, VTermGlyphInfo, VTermKeyEncodingFlags,
    VTermKeyEncodingStack, VTermLineInfo, VTermOutputCallback, VTermParserCallbacks,
    VTermParserState, VTermPen, VTermPos, VTermProp, VTermRect, VTermScreen, VTermScreenCallbacks,
    VTermScreenCell, VTermScreenCellAttrs, VTermSelectionCallbacks, VTermSelectionMask, VTermState,
    VTermStateCallbacks, VTermStateFallbacks, VTermStateFields,
    VTermState_mode as C2Rust_Unnamed_7, VTermState_mouse_protocol as C2Rust_Unnamed_8,
    VTermState_saved as C2Rust_Unnamed_5, VTermState_saved_mode as C2Rust_Unnamed_6,
    VTermState_selection as C2Rust_Unnamed_1, VTermState_tmp as C2Rust_Unnamed_2,
    VTermState_tmp_selection as C2Rust_Unnamed_3,
    VTermState_tmp_selection_state as C2Rust_Unnamed_4, VTermStringFragment, VTermTerminator,
    VTermValue, VTermValueType, VTerm_mode as C2Rust_Unnamed_14, VTerm_parser as C2Rust_Unnamed_9,
    VTerm_parser_v as C2Rust_Unnamed_10, VTerm_parser_v_csi as C2Rust_Unnamed_13,
    VTerm_parser_v_dcs as C2Rust_Unnamed_11, VTerm_parser_v_osc as C2Rust_Unnamed_12,
};
use ::c2rust_bitfields;
extern "C" {
    fn vterm_get_attr_type(attr: VTermAttr) -> VTermValueType;
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
pub const VTERM_N_VALUETYPES: VTermValueType = 5;
pub const VTERM_VALUETYPE_COLOR: VTermValueType = 4;
pub const VTERM_VALUETYPE_STRING: VTermValueType = 3;
pub const VTERM_VALUETYPE_INT: VTermValueType = 2;
pub const VTERM_VALUETYPE_BOOL: VTermValueType = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermRGB {
    pub red: uint8_t,
    pub green: uint8_t,
    pub blue: uint8_t,
}
pub const VTERM_COLOR_RGB: C2Rust_Unnamed_15 = 0;
pub const VTERM_COLOR_DEFAULT_BG: C2Rust_Unnamed_15 = 4;
pub const VTERM_COLOR_DEFAULT_MASK: C2Rust_Unnamed_15 = 6;
pub const VTERM_COLOR_DEFAULT_FG: C2Rust_Unnamed_15 = 2;
pub const VTERM_COLOR_TYPE_MASK: C2Rust_Unnamed_15 = 1;
pub const VTERM_COLOR_INDEXED: C2Rust_Unnamed_15 = 1;
pub const VTERM_BASELINE_NORMAL: C2Rust_Unnamed_17 = 0;
pub const VTERM_BASELINE_LOWER: C2Rust_Unnamed_17 = 2;
pub const VTERM_BASELINE_RAISE: C2Rust_Unnamed_17 = 1;
pub const VTERM_UNDERLINE_DOUBLE: C2Rust_Unnamed_16 = 2;
pub const VTERM_UNDERLINE_CURLY: C2Rust_Unnamed_16 = 3;
pub const VTERM_UNDERLINE_SINGLE: C2Rust_Unnamed_16 = 1;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const VTERM_UNDERLINE_OFF: C2Rust_Unnamed_16 = 0;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static ansi_colors: GlobalCell<[VTermRGB; 16]> = GlobalCell::new([
    VTermRGB {
        red: 0 as uint8_t,
        green: 0 as uint8_t,
        blue: 0 as uint8_t,
    },
    VTermRGB {
        red: 224 as uint8_t,
        green: 0 as uint8_t,
        blue: 0 as uint8_t,
    },
    VTermRGB {
        red: 0 as uint8_t,
        green: 224 as uint8_t,
        blue: 0 as uint8_t,
    },
    VTermRGB {
        red: 224 as uint8_t,
        green: 224 as uint8_t,
        blue: 0 as uint8_t,
    },
    VTermRGB {
        red: 0 as uint8_t,
        green: 0 as uint8_t,
        blue: 224 as uint8_t,
    },
    VTermRGB {
        red: 224 as uint8_t,
        green: 0 as uint8_t,
        blue: 224 as uint8_t,
    },
    VTermRGB {
        red: 0 as uint8_t,
        green: 224 as uint8_t,
        blue: 224 as uint8_t,
    },
    VTermRGB {
        red: 224 as uint8_t,
        green: 224 as uint8_t,
        blue: 224 as uint8_t,
    },
    VTermRGB {
        red: 128 as uint8_t,
        green: 128 as uint8_t,
        blue: 128 as uint8_t,
    },
    VTermRGB {
        red: 255 as uint8_t,
        green: 64 as uint8_t,
        blue: 64 as uint8_t,
    },
    VTermRGB {
        red: 64 as uint8_t,
        green: 255 as uint8_t,
        blue: 64 as uint8_t,
    },
    VTermRGB {
        red: 255 as uint8_t,
        green: 255 as uint8_t,
        blue: 64 as uint8_t,
    },
    VTermRGB {
        red: 64 as uint8_t,
        green: 64 as uint8_t,
        blue: 255 as uint8_t,
    },
    VTermRGB {
        red: 255 as uint8_t,
        green: 64 as uint8_t,
        blue: 255 as uint8_t,
    },
    VTermRGB {
        red: 64 as uint8_t,
        green: 255 as uint8_t,
        blue: 255 as uint8_t,
    },
    VTermRGB {
        red: 255 as uint8_t,
        green: 255 as uint8_t,
        blue: 255 as uint8_t,
    },
]);
static ramp6: GlobalCell<[uint8_t; 6]> = GlobalCell::new([
    0 as uint8_t,
    0x33 as uint8_t,
    0x66 as uint8_t,
    0x99 as uint8_t,
    0xcc as uint8_t,
    0xff as uint8_t,
]);
static ramp24: GlobalCell<[uint8_t; 24]> = GlobalCell::new([
    0 as uint8_t,
    0xb as uint8_t,
    0x16 as uint8_t,
    0x21 as uint8_t,
    0x2c as uint8_t,
    0x37 as uint8_t,
    0x42 as uint8_t,
    0x4d as uint8_t,
    0x58 as uint8_t,
    0x63 as uint8_t,
    0x6e as uint8_t,
    0x79 as uint8_t,
    0x85 as uint8_t,
    0x90 as uint8_t,
    0x9b as uint8_t,
    0xa6 as uint8_t,
    0xb1 as uint8_t,
    0xbc as uint8_t,
    0xc7 as uint8_t,
    0xd2 as uint8_t,
    0xdd as uint8_t,
    0xe8 as uint8_t,
    0xf3 as uint8_t,
    0xff as uint8_t,
]);
unsafe extern "C" fn lookup_default_colour_ansi(
    mut idx: ::core::ffi::c_long,
    mut col: *mut VTermColor,
) {
    if idx >= 0 as ::core::ffi::c_long && idx < 16 as ::core::ffi::c_long {
        vterm_color_rgb(
            col,
            (*ansi_colors.ptr())[idx as usize].red,
            (*ansi_colors.ptr())[idx as usize].green,
            (*ansi_colors.ptr())[idx as usize].blue,
        );
    }
}
unsafe extern "C" fn lookup_colour_ansi(
    mut state: *const VTermState,
    mut index: ::core::ffi::c_long,
    mut col: *mut VTermColor,
) -> bool {
    if index >= 0 as ::core::ffi::c_long && index < 16 as ::core::ffi::c_long {
        *col = (*state).colors[index as usize];
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn lookup_colour_palette(
    mut state: *const VTermState,
    mut index: ::core::ffi::c_long,
    mut col: *mut VTermColor,
) -> bool {
    if index >= 0 as ::core::ffi::c_long && index < 16 as ::core::ffi::c_long {
        return lookup_colour_ansi(state, index, col);
    } else if index >= 16 as ::core::ffi::c_long && index < 232 as ::core::ffi::c_long {
        index -= 16 as ::core::ffi::c_long;
        vterm_color_rgb(
            col,
            (*ramp6.ptr())[(index / 6 as ::core::ffi::c_long / 6 as ::core::ffi::c_long
                % 6 as ::core::ffi::c_long) as usize],
            (*ramp6.ptr())[(index / 6 as ::core::ffi::c_long % 6 as ::core::ffi::c_long) as usize],
            (*ramp6.ptr())[(index % 6 as ::core::ffi::c_long) as usize],
        );
        return true_0 != 0;
    } else if index >= 232 as ::core::ffi::c_long && index < 256 as ::core::ffi::c_long {
        index -= 232 as ::core::ffi::c_long;
        vterm_color_rgb(
            col,
            (*ramp24.ptr())[index as usize],
            (*ramp24.ptr())[index as usize],
            (*ramp24.ptr())[index as usize],
        );
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn lookup_colour(
    mut _state: *const VTermState,
    mut palette: ::core::ffi::c_int,
    mut args: *const ::core::ffi::c_long,
    mut argcount: ::core::ffi::c_int,
    mut col: *mut VTermColor,
) -> ::core::ffi::c_int {
    match palette {
        2 => {
            if argcount < 3 as ::core::ffi::c_int {
                return argcount;
            }
            vterm_color_rgb(
                col,
                (*args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long) as uint8_t,
                (*args.offset(1 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long) as uint8_t,
                (*args.offset(2 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long) as uint8_t,
            );
            return 3 as ::core::ffi::c_int;
        }
        5 => {
            if argcount == 0
                || (*args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long)
                    as ::core::ffi::c_ulong
                    == CSI_ARG_MISSING as ::core::ffi::c_ulong
            {
                return if argcount != 0 {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
            }
            vterm_color_indexed(
                col,
                *args.offset(0 as ::core::ffi::c_int as isize) as uint8_t,
            );
            return if argcount != 0 {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            };
        }
        _ => return 0 as ::core::ffi::c_int,
    };
}
unsafe extern "C" fn setpenattr(
    mut state: *mut VTermState,
    mut attr: VTermAttr,
    mut _type_0: VTermValueType,
    mut val: *mut VTermValue,
) {
    if !(*state).callbacks.is_null() && (*(*state).callbacks).setpenattr.is_some() {
        Some(
            (*(*state).callbacks)
                .setpenattr
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(attr, val, (*state).cbdata);
    }
}
unsafe extern "C" fn setpenattr_bool(
    mut state: *mut VTermState,
    mut attr: VTermAttr,
    mut boolean: ::core::ffi::c_int,
) {
    let mut val: VTermValue = VTermValue { boolean: boolean };
    setpenattr(state, attr, VTERM_VALUETYPE_BOOL, &raw mut val);
}
unsafe extern "C" fn setpenattr_int(
    mut state: *mut VTermState,
    mut attr: VTermAttr,
    mut number: ::core::ffi::c_int,
) {
    let mut val: VTermValue = VTermValue { number: number };
    setpenattr(state, attr, VTERM_VALUETYPE_INT, &raw mut val);
}
unsafe extern "C" fn setpenattr_col(
    mut state: *mut VTermState,
    mut attr: VTermAttr,
    mut color: VTermColor,
) {
    let mut val: VTermValue = VTermValue { color: color };
    setpenattr(state, attr, VTERM_VALUETYPE_COLOR, &raw mut val);
}
unsafe extern "C" fn set_pen_col_ansi(
    mut state: *mut VTermState,
    mut attr: VTermAttr,
    mut col: ::core::ffi::c_long,
) {
    let mut colp: *mut VTermColor = if attr as ::core::ffi::c_uint
        == VTERM_ATTR_BACKGROUND as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        &raw mut (*state).pen.bg
    } else {
        &raw mut (*state).pen.fg
    };
    vterm_color_indexed(colp, col as uint8_t);
    setpenattr_col(state, attr, *colp);
}
#[no_mangle]
pub unsafe extern "C" fn vterm_state_newpen(mut state: *mut VTermState) {
    vterm_color_rgb(
        &raw mut (*state).default_fg,
        240 as uint8_t,
        240 as uint8_t,
        240 as uint8_t,
    );
    vterm_color_rgb(
        &raw mut (*state).default_bg,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    );
    vterm_state_set_default_colors(
        state,
        &raw mut (*state).default_fg,
        &raw mut (*state).default_bg,
    );
    let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while col < 16 as ::core::ffi::c_int {
        lookup_default_colour_ansi(
            col as ::core::ffi::c_long,
            (&raw mut (*state).colors as *mut VTermColor).offset(col as isize),
        );
        col += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn vterm_state_resetpen(mut state: *mut VTermState) {
    (*state)
        .pen
        .set_bold(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    setpenattr_bool(state, VTERM_ATTR_BOLD, 0 as ::core::ffi::c_int);
    (*state)
        .pen
        .set_underline(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    setpenattr_int(state, VTERM_ATTR_UNDERLINE, 0 as ::core::ffi::c_int);
    (*state)
        .pen
        .set_italic(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    setpenattr_bool(state, VTERM_ATTR_ITALIC, 0 as ::core::ffi::c_int);
    (*state)
        .pen
        .set_blink(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    setpenattr_bool(state, VTERM_ATTR_BLINK, 0 as ::core::ffi::c_int);
    (*state)
        .pen
        .set_reverse(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    setpenattr_bool(state, VTERM_ATTR_REVERSE, 0 as ::core::ffi::c_int);
    (*state)
        .pen
        .set_conceal(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    setpenattr_bool(state, VTERM_ATTR_CONCEAL, 0 as ::core::ffi::c_int);
    (*state)
        .pen
        .set_strike(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    setpenattr_bool(state, VTERM_ATTR_STRIKE, 0 as ::core::ffi::c_int);
    (*state)
        .pen
        .set_font(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    setpenattr_int(state, VTERM_ATTR_FONT, 0 as ::core::ffi::c_int);
    (*state)
        .pen
        .set_small(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    setpenattr_bool(state, VTERM_ATTR_SMALL, 0 as ::core::ffi::c_int);
    (*state)
        .pen
        .set_baseline(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    setpenattr_int(state, VTERM_ATTR_BASELINE, 0 as ::core::ffi::c_int);
    (*state)
        .pen
        .set_dim(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    setpenattr_bool(state, VTERM_ATTR_DIM, 0 as ::core::ffi::c_int);
    (*state)
        .pen
        .set_overline(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    setpenattr_bool(state, VTERM_ATTR_OVERLINE, 0 as ::core::ffi::c_int);
    (*state).pen.fg = (*state).default_fg;
    setpenattr_col(state, VTERM_ATTR_FOREGROUND, (*state).default_fg);
    (*state).pen.bg = (*state).default_bg;
    setpenattr_col(state, VTERM_ATTR_BACKGROUND, (*state).default_bg);
    (*state).pen.uri = 0 as ::core::ffi::c_int;
    setpenattr_int(state, VTERM_ATTR_URI, 0 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn vterm_state_savepen(
    mut state: *mut VTermState,
    mut save: ::core::ffi::c_int,
) {
    if save != 0 {
        (*state).saved.pen = (*state).pen;
    } else {
        (*state).pen = (*state).saved.pen;
        setpenattr_bool(
            state,
            VTERM_ATTR_BOLD,
            (*state).pen.bold() as ::core::ffi::c_int,
        );
        setpenattr_int(
            state,
            VTERM_ATTR_UNDERLINE,
            (*state).pen.underline() as ::core::ffi::c_int,
        );
        setpenattr_bool(
            state,
            VTERM_ATTR_ITALIC,
            (*state).pen.italic() as ::core::ffi::c_int,
        );
        setpenattr_bool(
            state,
            VTERM_ATTR_BLINK,
            (*state).pen.blink() as ::core::ffi::c_int,
        );
        setpenattr_bool(
            state,
            VTERM_ATTR_REVERSE,
            (*state).pen.reverse() as ::core::ffi::c_int,
        );
        setpenattr_bool(
            state,
            VTERM_ATTR_CONCEAL,
            (*state).pen.conceal() as ::core::ffi::c_int,
        );
        setpenattr_bool(
            state,
            VTERM_ATTR_STRIKE,
            (*state).pen.strike() as ::core::ffi::c_int,
        );
        setpenattr_int(
            state,
            VTERM_ATTR_FONT,
            (*state).pen.font() as ::core::ffi::c_int,
        );
        setpenattr_bool(
            state,
            VTERM_ATTR_SMALL,
            (*state).pen.small() as ::core::ffi::c_int,
        );
        setpenattr_int(
            state,
            VTERM_ATTR_BASELINE,
            (*state).pen.baseline() as ::core::ffi::c_int,
        );
        setpenattr_bool(
            state,
            VTERM_ATTR_DIM,
            (*state).pen.dim() as ::core::ffi::c_int,
        );
        setpenattr_bool(
            state,
            VTERM_ATTR_OVERLINE,
            (*state).pen.overline() as ::core::ffi::c_int,
        );
        setpenattr_col(state, VTERM_ATTR_FOREGROUND, (*state).pen.fg);
        setpenattr_col(state, VTERM_ATTR_BACKGROUND, (*state).pen.bg);
        setpenattr_int(state, VTERM_ATTR_URI, (*state).pen.uri);
    };
}
#[no_mangle]
pub unsafe extern "C" fn vterm_state_set_default_colors(
    mut state: *mut VTermState,
    mut default_fg: *const VTermColor,
    mut default_bg: *const VTermColor,
) {
    if !default_fg.is_null() {
        (*state).default_fg = *default_fg;
        (*state).default_fg.type_0 = ((*state).default_fg.type_0 as ::core::ffi::c_int
            & !(VTERM_COLOR_DEFAULT_MASK as ::core::ffi::c_int)
            | VTERM_COLOR_DEFAULT_FG as ::core::ffi::c_int)
            as uint8_t;
    }
    if !default_bg.is_null() {
        (*state).default_bg = *default_bg;
        (*state).default_bg.type_0 = ((*state).default_bg.type_0 as ::core::ffi::c_int
            & !(VTERM_COLOR_DEFAULT_MASK as ::core::ffi::c_int)
            | VTERM_COLOR_DEFAULT_BG as ::core::ffi::c_int)
            as uint8_t;
    }
}
#[no_mangle]
pub unsafe extern "C" fn vterm_state_set_palette_color(
    mut state: *mut VTermState,
    mut index: ::core::ffi::c_int,
    mut col: *const VTermColor,
) {
    if index >= 0 as ::core::ffi::c_int && index < 16 as ::core::ffi::c_int {
        (*state).colors[index as usize] = *col;
    }
}
#[no_mangle]
pub unsafe extern "C" fn vterm_state_convert_color_to_rgb(
    mut state: *const VTermState,
    mut col: *mut VTermColor,
) {
    if (*col).type_0 as ::core::ffi::c_int & VTERM_COLOR_TYPE_MASK as ::core::ffi::c_int
        == VTERM_COLOR_INDEXED as ::core::ffi::c_int
    {
        lookup_colour_palette(state, (*col).indexed.idx as ::core::ffi::c_long, col);
    }
    (*col).type_0 = ((*col).type_0 as ::core::ffi::c_int
        & VTERM_COLOR_TYPE_MASK as ::core::ffi::c_int) as uint8_t;
}
#[no_mangle]
pub unsafe extern "C" fn vterm_state_setpen(
    mut state: *mut VTermState,
    mut args: *const ::core::ffi::c_long,
    mut argcount: ::core::ffi::c_int,
) {
    let mut argi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut value: ::core::ffi::c_int = 0;
    while argi < argcount {
        let mut done: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        let mut arg: ::core::ffi::c_long = 0;
        's_495: {
            arg = *args.offset(argi as isize) & CSI_ARG_MASK as ::core::ffi::c_long;
            match arg {
                2147483647 | 0 => {
                    vterm_state_resetpen(state);
                    break 's_495;
                }
                1 => {
                    let mut fg: *const VTermColor = &raw mut (*state).pen.fg;
                    (*state)
                        .pen
                        .set_bold(1 as ::core::ffi::c_uint as ::core::ffi::c_uint);
                    setpenattr_bool(state, VTERM_ATTR_BOLD, 1 as ::core::ffi::c_int);
                    if (*fg).type_0 as ::core::ffi::c_int
                        & VTERM_COLOR_DEFAULT_FG as ::core::ffi::c_int
                        == 0
                        && (*fg).type_0 as ::core::ffi::c_int
                            & VTERM_COLOR_TYPE_MASK as ::core::ffi::c_int
                            == VTERM_COLOR_INDEXED as ::core::ffi::c_int
                        && ((*fg).indexed.idx as ::core::ffi::c_int) < 8 as ::core::ffi::c_int
                        && (*state).bold_is_highbright != 0
                    {
                        set_pen_col_ansi(
                            state,
                            VTERM_ATTR_FOREGROUND,
                            ((*fg).indexed.idx as ::core::ffi::c_int
                                + (if (*state).pen.bold() as ::core::ffi::c_int != 0 {
                                    8 as ::core::ffi::c_int
                                } else {
                                    0 as ::core::ffi::c_int
                                })) as ::core::ffi::c_long,
                        );
                    }
                    break 's_495;
                }
                2 => {
                    (*state)
                        .pen
                        .set_dim(1 as ::core::ffi::c_uint as ::core::ffi::c_uint);
                    setpenattr_bool(state, VTERM_ATTR_DIM, 1 as ::core::ffi::c_int);
                    break 's_495;
                }
                3 => {
                    (*state)
                        .pen
                        .set_italic(1 as ::core::ffi::c_uint as ::core::ffi::c_uint);
                    setpenattr_bool(state, VTERM_ATTR_ITALIC, 1 as ::core::ffi::c_int);
                    break 's_495;
                }
                4 => {
                    (*state).pen.set_underline(
                        VTERM_UNDERLINE_SINGLE as ::core::ffi::c_int as ::core::ffi::c_uint
                            as ::core::ffi::c_uint,
                    );
                    if *args.offset(argi as isize) & CSI_ARG_FLAG_MORE as ::core::ffi::c_long != 0 {
                        argi += 1;
                        match *args.offset(argi as isize) & CSI_ARG_MASK as ::core::ffi::c_long {
                            0 => {
                                (*state)
                                    .pen
                                    .set_underline(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
                            }
                            1 => {
                                (*state).pen.set_underline(
                                    VTERM_UNDERLINE_SINGLE as ::core::ffi::c_int
                                        as ::core::ffi::c_uint
                                        as ::core::ffi::c_uint,
                                );
                            }
                            2 => {
                                (*state).pen.set_underline(
                                    VTERM_UNDERLINE_DOUBLE as ::core::ffi::c_int
                                        as ::core::ffi::c_uint
                                        as ::core::ffi::c_uint,
                                );
                            }
                            3 => {
                                (*state).pen.set_underline(
                                    VTERM_UNDERLINE_CURLY as ::core::ffi::c_int
                                        as ::core::ffi::c_uint
                                        as ::core::ffi::c_uint,
                                );
                            }
                            _ => {}
                        }
                    }
                    setpenattr_int(
                        state,
                        VTERM_ATTR_UNDERLINE,
                        (*state).pen.underline() as ::core::ffi::c_int,
                    );
                    break 's_495;
                }
                5 => {
                    (*state)
                        .pen
                        .set_blink(1 as ::core::ffi::c_uint as ::core::ffi::c_uint);
                    setpenattr_bool(state, VTERM_ATTR_BLINK, 1 as ::core::ffi::c_int);
                    break 's_495;
                }
                7 => {
                    (*state)
                        .pen
                        .set_reverse(1 as ::core::ffi::c_uint as ::core::ffi::c_uint);
                    setpenattr_bool(state, VTERM_ATTR_REVERSE, 1 as ::core::ffi::c_int);
                    break 's_495;
                }
                8 => {
                    (*state)
                        .pen
                        .set_conceal(1 as ::core::ffi::c_uint as ::core::ffi::c_uint);
                    setpenattr_bool(state, VTERM_ATTR_CONCEAL, 1 as ::core::ffi::c_int);
                    break 's_495;
                }
                9 => {
                    (*state)
                        .pen
                        .set_strike(1 as ::core::ffi::c_uint as ::core::ffi::c_uint);
                    setpenattr_bool(state, VTERM_ATTR_STRIKE, 1 as ::core::ffi::c_int);
                    break 's_495;
                }
                10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 => {
                    (*state).pen.set_font(
                        ((*args.offset(argi as isize) & CSI_ARG_MASK as ::core::ffi::c_long)
                            - 10 as ::core::ffi::c_long)
                            as ::core::ffi::c_uint as ::core::ffi::c_uint,
                    );
                    setpenattr_int(
                        state,
                        VTERM_ATTR_FONT,
                        (*state).pen.font() as ::core::ffi::c_int,
                    );
                    break 's_495;
                }
                21 => {
                    (*state).pen.set_underline(
                        VTERM_UNDERLINE_DOUBLE as ::core::ffi::c_int as ::core::ffi::c_uint
                            as ::core::ffi::c_uint,
                    );
                    setpenattr_int(
                        state,
                        VTERM_ATTR_UNDERLINE,
                        (*state).pen.underline() as ::core::ffi::c_int,
                    );
                    break 's_495;
                }
                22 => {
                    (*state)
                        .pen
                        .set_bold(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
                    setpenattr_bool(state, VTERM_ATTR_BOLD, 0 as ::core::ffi::c_int);
                    (*state)
                        .pen
                        .set_dim(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
                    setpenattr_bool(state, VTERM_ATTR_DIM, 0 as ::core::ffi::c_int);
                    break 's_495;
                }
                23 => {
                    (*state)
                        .pen
                        .set_italic(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
                    setpenattr_bool(state, VTERM_ATTR_ITALIC, 0 as ::core::ffi::c_int);
                    break 's_495;
                }
                24 => {
                    (*state)
                        .pen
                        .set_underline(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
                    setpenattr_int(state, VTERM_ATTR_UNDERLINE, 0 as ::core::ffi::c_int);
                    break 's_495;
                }
                25 => {
                    (*state)
                        .pen
                        .set_blink(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
                    setpenattr_bool(state, VTERM_ATTR_BLINK, 0 as ::core::ffi::c_int);
                    break 's_495;
                }
                27 => {
                    (*state)
                        .pen
                        .set_reverse(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
                    setpenattr_bool(state, VTERM_ATTR_REVERSE, 0 as ::core::ffi::c_int);
                    break 's_495;
                }
                28 => {
                    (*state)
                        .pen
                        .set_conceal(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
                    setpenattr_bool(state, VTERM_ATTR_CONCEAL, 0 as ::core::ffi::c_int);
                    break 's_495;
                }
                29 => {
                    (*state)
                        .pen
                        .set_strike(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
                    setpenattr_bool(state, VTERM_ATTR_STRIKE, 0 as ::core::ffi::c_int);
                    break 's_495;
                }
                30 | 31 | 32 | 33 | 34 | 35 | 36 | 37 => {
                    value = ((*args.offset(argi as isize) & CSI_ARG_MASK as ::core::ffi::c_long)
                        - 30 as ::core::ffi::c_long)
                        as ::core::ffi::c_int;
                    if (*state).pen.bold() as ::core::ffi::c_int != 0
                        && (*state).bold_is_highbright != 0
                    {
                        value += 8 as ::core::ffi::c_int;
                    }
                    set_pen_col_ansi(state, VTERM_ATTR_FOREGROUND, value as ::core::ffi::c_long);
                    break 's_495;
                }
                38 => {
                    if argcount - argi < 1 as ::core::ffi::c_int {
                        return;
                    }
                    argi += 1 as ::core::ffi::c_int
                        + lookup_colour(
                            state,
                            (*args.offset((argi + 1 as ::core::ffi::c_int) as isize)
                                & CSI_ARG_MASK as ::core::ffi::c_long)
                                as ::core::ffi::c_int,
                            args.offset(argi as isize)
                                .offset(2 as ::core::ffi::c_int as isize),
                            argcount - argi - 2 as ::core::ffi::c_int,
                            &raw mut (*state).pen.fg,
                        );
                    setpenattr_col(state, VTERM_ATTR_FOREGROUND, (*state).pen.fg);
                    break 's_495;
                }
                39 => {
                    (*state).pen.fg = (*state).default_fg;
                    setpenattr_col(state, VTERM_ATTR_FOREGROUND, (*state).pen.fg);
                    break 's_495;
                }
                40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 => {
                    value = ((*args.offset(argi as isize) & CSI_ARG_MASK as ::core::ffi::c_long)
                        - 40 as ::core::ffi::c_long)
                        as ::core::ffi::c_int;
                    set_pen_col_ansi(state, VTERM_ATTR_BACKGROUND, value as ::core::ffi::c_long);
                    break 's_495;
                }
                48 => {
                    if argcount - argi < 1 as ::core::ffi::c_int {
                        return;
                    }
                    argi += 1 as ::core::ffi::c_int
                        + lookup_colour(
                            state,
                            (*args.offset((argi + 1 as ::core::ffi::c_int) as isize)
                                & CSI_ARG_MASK as ::core::ffi::c_long)
                                as ::core::ffi::c_int,
                            args.offset(argi as isize)
                                .offset(2 as ::core::ffi::c_int as isize),
                            argcount - argi - 2 as ::core::ffi::c_int,
                            &raw mut (*state).pen.bg,
                        );
                    setpenattr_col(state, VTERM_ATTR_BACKGROUND, (*state).pen.bg);
                    break 's_495;
                }
                49 => {
                    (*state).pen.bg = (*state).default_bg;
                    setpenattr_col(state, VTERM_ATTR_BACKGROUND, (*state).pen.bg);
                    break 's_495;
                }
                53 => {
                    (*state)
                        .pen
                        .set_overline(1 as ::core::ffi::c_uint as ::core::ffi::c_uint);
                    setpenattr_bool(state, VTERM_ATTR_OVERLINE, 1 as ::core::ffi::c_int);
                    break 's_495;
                }
                55 => {
                    (*state)
                        .pen
                        .set_overline(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
                    setpenattr_bool(state, VTERM_ATTR_OVERLINE, 0 as ::core::ffi::c_int);
                    break 's_495;
                }
                73 => {}
                74 | 75 => {}
                90 | 91 | 92 | 93 | 94 | 95 | 96 | 97 => {
                    value = ((*args.offset(argi as isize) & CSI_ARG_MASK as ::core::ffi::c_long)
                        - 90 as ::core::ffi::c_long
                        + 8 as ::core::ffi::c_long)
                        as ::core::ffi::c_int;
                    set_pen_col_ansi(state, VTERM_ATTR_FOREGROUND, value as ::core::ffi::c_long);
                    break 's_495;
                }
                100 | 101 | 102 | 103 | 104 | 105 | 106 | 107 => {
                    value = ((*args.offset(argi as isize) & CSI_ARG_MASK as ::core::ffi::c_long)
                        - 100 as ::core::ffi::c_long
                        + 8 as ::core::ffi::c_long)
                        as ::core::ffi::c_int;
                    set_pen_col_ansi(state, VTERM_ATTR_BACKGROUND, value as ::core::ffi::c_long);
                    break 's_495;
                }
                _ => {
                    done = 0 as ::core::ffi::c_int;
                    break 's_495;
                }
            }
            (*state).pen.set_small(
                (arg != 75 as ::core::ffi::c_long) as ::core::ffi::c_int as ::core::ffi::c_uint
                    as ::core::ffi::c_uint,
            );
            (*state).pen.set_baseline(
                (if arg == 73 as ::core::ffi::c_long {
                    VTERM_BASELINE_RAISE as ::core::ffi::c_int
                } else if arg == 74 as ::core::ffi::c_long {
                    VTERM_BASELINE_LOWER as ::core::ffi::c_int
                } else {
                    VTERM_BASELINE_NORMAL as ::core::ffi::c_int
                }) as ::core::ffi::c_uint as ::core::ffi::c_uint,
            );
            setpenattr_bool(
                state,
                VTERM_ATTR_SMALL,
                (*state).pen.small() as ::core::ffi::c_int,
            );
            setpenattr_int(
                state,
                VTERM_ATTR_BASELINE,
                (*state).pen.baseline() as ::core::ffi::c_int,
            );
        }
        let _ = done == 0;
        loop {
            let c2rust_fresh0 = argi;
            argi = argi + 1;
            if *args.offset(c2rust_fresh0 as isize) & CSI_ARG_FLAG_MORE as ::core::ffi::c_long == 0
            {
                break;
            }
        }
    }
}
unsafe extern "C" fn vterm_state_getpen_color(
    mut col: *const VTermColor,
    mut argi: ::core::ffi::c_int,
    mut args: *mut ::core::ffi::c_long,
    mut fg: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if fg != 0
        && (*col).type_0 as ::core::ffi::c_int & VTERM_COLOR_DEFAULT_FG as ::core::ffi::c_int != 0
        || fg == 0
            && (*col).type_0 as ::core::ffi::c_int & VTERM_COLOR_DEFAULT_BG as ::core::ffi::c_int
                != 0
    {
        return argi;
    }
    if (*col).type_0 as ::core::ffi::c_int & VTERM_COLOR_TYPE_MASK as ::core::ffi::c_int
        == VTERM_COLOR_INDEXED as ::core::ffi::c_int
    {
        let idx: uint8_t = (*col).indexed.idx;
        if (idx as ::core::ffi::c_int) < 8 as ::core::ffi::c_int {
            let c2rust_fresh16 = argi;
            argi = argi + 1;
            *args.offset(c2rust_fresh16 as isize) = (idx as ::core::ffi::c_int
                + (if fg != 0 {
                    30 as ::core::ffi::c_int
                } else {
                    40 as ::core::ffi::c_int
                })) as ::core::ffi::c_long;
        } else if (idx as ::core::ffi::c_int) < 16 as ::core::ffi::c_int {
            let c2rust_fresh17 = argi;
            argi = argi + 1;
            *args.offset(c2rust_fresh17 as isize) = (idx as ::core::ffi::c_int
                - 8 as ::core::ffi::c_int
                + (if fg != 0 {
                    90 as ::core::ffi::c_int
                } else {
                    100 as ::core::ffi::c_int
                })) as ::core::ffi::c_long;
        } else {
            let c2rust_fresh18 = argi;
            argi = argi + 1;
            *args.offset(c2rust_fresh18 as isize) = (CSI_ARG_FLAG_MORE
                | (if fg != 0 {
                    38 as ::core::ffi::c_int
                } else {
                    48 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint)
                as ::core::ffi::c_long;
            let c2rust_fresh19 = argi;
            argi = argi + 1;
            *args.offset(c2rust_fresh19 as isize) =
                (CSI_ARG_FLAG_MORE | 5 as ::core::ffi::c_uint) as ::core::ffi::c_long;
            let c2rust_fresh20 = argi;
            argi = argi + 1;
            *args.offset(c2rust_fresh20 as isize) = idx as ::core::ffi::c_long;
        }
    } else if (*col).type_0 as ::core::ffi::c_int & VTERM_COLOR_TYPE_MASK as ::core::ffi::c_int
        == VTERM_COLOR_RGB as ::core::ffi::c_int
    {
        let c2rust_fresh21 = argi;
        argi = argi + 1;
        *args.offset(c2rust_fresh21 as isize) = (CSI_ARG_FLAG_MORE
            | (if fg != 0 {
                38 as ::core::ffi::c_int
            } else {
                48 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint)
            as ::core::ffi::c_long;
        let c2rust_fresh22 = argi;
        argi = argi + 1;
        *args.offset(c2rust_fresh22 as isize) =
            (CSI_ARG_FLAG_MORE | 2 as ::core::ffi::c_uint) as ::core::ffi::c_long;
        let c2rust_fresh23 = argi;
        argi = argi + 1;
        *args.offset(c2rust_fresh23 as isize) =
            (CSI_ARG_FLAG_MORE | (*col).rgb.red as ::core::ffi::c_uint) as ::core::ffi::c_long;
        let c2rust_fresh24 = argi;
        argi = argi + 1;
        *args.offset(c2rust_fresh24 as isize) =
            (CSI_ARG_FLAG_MORE | (*col).rgb.green as ::core::ffi::c_uint) as ::core::ffi::c_long;
        let c2rust_fresh25 = argi;
        argi = argi + 1;
        *args.offset(c2rust_fresh25 as isize) = (*col).rgb.blue as ::core::ffi::c_long;
    }
    return argi;
}
#[no_mangle]
pub unsafe extern "C" fn vterm_state_getpen(
    mut state: *mut VTermState,
    mut args: *mut ::core::ffi::c_long,
    mut _argcount: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut argi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*state).pen.bold() != 0 {
        let c2rust_fresh1 = argi;
        argi = argi + 1;
        *args.offset(c2rust_fresh1 as isize) = 1 as ::core::ffi::c_long;
    }
    if (*state).pen.dim() != 0 {
        let c2rust_fresh2 = argi;
        argi = argi + 1;
        *args.offset(c2rust_fresh2 as isize) = 2 as ::core::ffi::c_long;
    }
    if (*state).pen.italic() != 0 {
        let c2rust_fresh3 = argi;
        argi = argi + 1;
        *args.offset(c2rust_fresh3 as isize) = 3 as ::core::ffi::c_long;
    }
    if (*state).pen.underline() as ::core::ffi::c_int
        == VTERM_UNDERLINE_SINGLE as ::core::ffi::c_int
    {
        let c2rust_fresh4 = argi;
        argi = argi + 1;
        *args.offset(c2rust_fresh4 as isize) = 4 as ::core::ffi::c_long;
    }
    if (*state).pen.underline() as ::core::ffi::c_int == VTERM_UNDERLINE_CURLY as ::core::ffi::c_int
    {
        let c2rust_fresh5 = argi;
        argi = argi + 1;
        *args.offset(c2rust_fresh5 as isize) =
            (4 as ::core::ffi::c_uint | CSI_ARG_FLAG_MORE) as ::core::ffi::c_long;
        let c2rust_fresh6 = argi;
        argi = argi + 1;
        *args.offset(c2rust_fresh6 as isize) = 3 as ::core::ffi::c_long;
    }
    if (*state).pen.blink() != 0 {
        let c2rust_fresh7 = argi;
        argi = argi + 1;
        *args.offset(c2rust_fresh7 as isize) = 5 as ::core::ffi::c_long;
    }
    if (*state).pen.reverse() != 0 {
        let c2rust_fresh8 = argi;
        argi = argi + 1;
        *args.offset(c2rust_fresh8 as isize) = 7 as ::core::ffi::c_long;
    }
    if (*state).pen.conceal() != 0 {
        let c2rust_fresh9 = argi;
        argi = argi + 1;
        *args.offset(c2rust_fresh9 as isize) = 8 as ::core::ffi::c_long;
    }
    if (*state).pen.strike() != 0 {
        let c2rust_fresh10 = argi;
        argi = argi + 1;
        *args.offset(c2rust_fresh10 as isize) = 9 as ::core::ffi::c_long;
    }
    if (*state).pen.font() != 0 {
        let c2rust_fresh11 = argi;
        argi = argi + 1;
        *args.offset(c2rust_fresh11 as isize) = (10 as ::core::ffi::c_int
            + (*state).pen.font() as ::core::ffi::c_int)
            as ::core::ffi::c_long;
    }
    if (*state).pen.underline() as ::core::ffi::c_int
        == VTERM_UNDERLINE_DOUBLE as ::core::ffi::c_int
    {
        let c2rust_fresh12 = argi;
        argi = argi + 1;
        *args.offset(c2rust_fresh12 as isize) = 21 as ::core::ffi::c_long;
    }
    argi = vterm_state_getpen_color(&raw mut (*state).pen.fg, argi, args, true_0);
    argi = vterm_state_getpen_color(&raw mut (*state).pen.bg, argi, args, false_0);
    if (*state).pen.overline() != 0 {
        let c2rust_fresh13 = argi;
        argi = argi + 1;
        *args.offset(c2rust_fresh13 as isize) = 53 as ::core::ffi::c_long;
    }
    if (*state).pen.small() != 0 {
        if (*state).pen.baseline() as ::core::ffi::c_int
            == VTERM_BASELINE_RAISE as ::core::ffi::c_int
        {
            let c2rust_fresh14 = argi;
            argi = argi + 1;
            *args.offset(c2rust_fresh14 as isize) = 73 as ::core::ffi::c_long;
        } else if (*state).pen.baseline() as ::core::ffi::c_int
            == VTERM_BASELINE_LOWER as ::core::ffi::c_int
        {
            let c2rust_fresh15 = argi;
            argi = argi + 1;
            *args.offset(c2rust_fresh15 as isize) = 74 as ::core::ffi::c_long;
        }
    }
    return argi;
}
#[no_mangle]
pub unsafe extern "C" fn vterm_state_set_penattr(
    mut state: *mut VTermState,
    mut attr: VTermAttr,
    mut type_0: VTermValueType,
    mut val: *mut VTermValue,
) -> ::core::ffi::c_int {
    if val.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if type_0 as ::core::ffi::c_uint != vterm_get_attr_type(attr) as ::core::ffi::c_uint {
        return 0 as ::core::ffi::c_int;
    }
    match attr as ::core::ffi::c_uint {
        1 => {
            (*state)
                .pen
                .set_bold((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
        }
        2 => {
            (*state)
                .pen
                .set_underline((*val).number as ::core::ffi::c_uint as ::core::ffi::c_uint);
        }
        3 => {
            (*state)
                .pen
                .set_italic((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
        }
        4 => {
            (*state)
                .pen
                .set_blink((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
        }
        5 => {
            (*state)
                .pen
                .set_reverse((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
        }
        6 => {
            (*state)
                .pen
                .set_conceal((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
        }
        7 => {
            (*state)
                .pen
                .set_strike((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
        }
        8 => {
            (*state)
                .pen
                .set_font((*val).number as ::core::ffi::c_uint as ::core::ffi::c_uint);
        }
        9 => {
            (*state).pen.fg = (*val).color;
        }
        10 => {
            (*state).pen.bg = (*val).color;
        }
        11 => {
            (*state)
                .pen
                .set_small((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
        }
        12 => {
            (*state)
                .pen
                .set_baseline((*val).number as ::core::ffi::c_uint as ::core::ffi::c_uint);
        }
        13 => {
            (*state).pen.uri = (*val).number;
        }
        14 => {
            (*state)
                .pen
                .set_dim((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
        }
        15 => {
            (*state)
                .pen
                .set_overline((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
        }
        _ => return 0 as ::core::ffi::c_int,
    }
    if !(*state).callbacks.is_null() && (*(*state).callbacks).setpenattr.is_some() {
        Some(
            (*(*state).callbacks)
                .setpenattr
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(attr, val, (*state).cbdata);
    }
    return 1 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn vterm_color_rgb(
    mut col: *mut VTermColor,
    mut red: uint8_t,
    mut green: uint8_t,
    mut blue: uint8_t,
) {
    (*col).type_0 = VTERM_COLOR_RGB as ::core::ffi::c_int as uint8_t;
    (*col).rgb.red = red;
    (*col).rgb.green = green;
    (*col).rgb.blue = blue;
}
#[inline]
unsafe extern "C" fn vterm_color_indexed(mut col: *mut VTermColor, mut idx: uint8_t) {
    (*col).type_0 = VTERM_COLOR_INDEXED as ::core::ffi::c_int as uint8_t;
    (*col).indexed.idx = idx;
}
pub const CSI_ARG_FLAG_MORE: ::core::ffi::c_uint =
    (1 as ::core::ffi::c_uint) << 31 as ::core::ffi::c_int;
pub const CSI_ARG_MASK: ::core::ffi::c_uint =
    !((1 as ::core::ffi::c_uint) << 31 as ::core::ffi::c_int);
pub const CSI_ARG_MISSING: ::core::ffi::c_long = 2147483647 as ::core::ffi::c_long;
