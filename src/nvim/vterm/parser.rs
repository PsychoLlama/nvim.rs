use crate::src::nvim::os::libc::{__assert_fail, strncpy};
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
    VTermValue, VTerm_mode as C2Rust_Unnamed_14, VTerm_parser as C2Rust_Unnamed_9,
    VTerm_parser_v as C2Rust_Unnamed_10, VTerm_parser_v_csi as C2Rust_Unnamed_13,
    VTerm_parser_v_dcs as C2Rust_Unnamed_11, VTerm_parser_v_osc as C2Rust_Unnamed_12,
};

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
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 56] = unsafe {
    ::core::mem::transmute::<[u8; 56], [::core::ffi::c_char; 56]>(
        *b"size_t vterm_input_write(VTerm *, const char *, size_t)\0",
    )
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INTERMED_MAX: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const CSI_LEADER_MAX: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
unsafe extern "C" fn is_intermed(mut c: uint8_t) -> bool {
    return c as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int
        && c as ::core::ffi::c_int <= 0x2f as ::core::ffi::c_int;
}
unsafe extern "C" fn do_control(mut vt: *mut VTerm, mut control: uint8_t) {
    if !(*vt).parser.callbacks.is_null() && (*(*vt).parser.callbacks).control.is_some() {
        if Some(
            (*(*vt).parser.callbacks)
                .control
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(control, (*vt).parser.cbdata)
            != 0
        {
            return;
        }
    }
}
unsafe extern "C" fn do_csi(mut vt: *mut VTerm, mut command: ::core::ffi::c_char) {
    if !(*vt).parser.callbacks.is_null() && (*(*vt).parser.callbacks).csi.is_some() {
        if Some(
            (*(*vt).parser.callbacks)
                .csi
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(
            if (*vt).parser.v.csi.leaderlen != 0 {
                &raw mut (*vt).parser.v.csi.leader as *mut ::core::ffi::c_char
            } else {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            },
            &raw mut (*vt).parser.v.csi.args as *mut ::core::ffi::c_long
                as *const ::core::ffi::c_long,
            (*vt).parser.v.csi.argi,
            if (*vt).parser.intermedlen != 0 {
                &raw mut (*vt).parser.intermed as *mut ::core::ffi::c_char
            } else {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            },
            command,
            (*vt).parser.cbdata,
        ) != 0
        {
            return;
        }
    }
}
unsafe extern "C" fn do_escape(mut vt: *mut VTerm, mut command: ::core::ffi::c_char) {
    let mut seq: [::core::ffi::c_char; 17] = [0; 17];
    let mut len: size_t = (*vt).parser.intermedlen as size_t;
    strncpy(
        &raw mut seq as *mut ::core::ffi::c_char,
        &raw mut (*vt).parser.intermed as *mut ::core::ffi::c_char,
        len,
    );
    let c2rust_fresh4 = len;
    len = len.wrapping_add(1);
    seq[c2rust_fresh4 as usize] = command;
    seq[len as usize] = 0 as ::core::ffi::c_char;
    if !(*vt).parser.callbacks.is_null() && (*(*vt).parser.callbacks).escape.is_some() {
        if Some(
            (*(*vt).parser.callbacks)
                .escape
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(
            &raw mut seq as *mut ::core::ffi::c_char,
            len,
            (*vt).parser.cbdata,
        ) != 0
        {
            return;
        }
    }
}
unsafe extern "C" fn string_fragment(
    mut vt: *mut VTerm,
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
    mut final_0: bool,
    mut terminator: VTermTerminator,
) {
    let mut frag: VTermStringFragment = {
        let mut init = VTermStringFragment {
            len_initial_final_0: [0; 4],
            str: str,
            terminator: terminator,
        };
        init.set_len(len);
        init.set_initial((*vt).parser.string_initial);
        init.set_final_0(final_0);
        init
    };
    match (*vt).parser.state as ::core::ffi::c_uint {
        6 => {
            if !(*vt).parser.callbacks.is_null() && (*(*vt).parser.callbacks).osc.is_some() {
                Some(
                    (*(*vt).parser.callbacks)
                        .osc
                        .expect("non-null function pointer"),
                )
                .expect("non-null function pointer")(
                    (*vt).parser.v.osc.command,
                    frag,
                    (*vt).parser.cbdata,
                );
            }
        }
        7 => {
            if !(*vt).parser.callbacks.is_null() && (*(*vt).parser.callbacks).dcs.is_some() {
                Some(
                    (*(*vt).parser.callbacks)
                        .dcs
                        .expect("non-null function pointer"),
                )
                .expect("non-null function pointer")(
                    &raw mut (*vt).parser.v.dcs.command as *mut ::core::ffi::c_char,
                    (*vt).parser.v.dcs.commandlen as size_t,
                    frag,
                    (*vt).parser.cbdata,
                );
            }
        }
        8 => {
            if !(*vt).parser.callbacks.is_null() && (*(*vt).parser.callbacks).apc.is_some() {
                Some(
                    (*(*vt).parser.callbacks)
                        .apc
                        .expect("non-null function pointer"),
                )
                .expect("non-null function pointer")(frag, (*vt).parser.cbdata);
            }
        }
        9 => {
            if !(*vt).parser.callbacks.is_null() && (*(*vt).parser.callbacks).pm.is_some() {
                Some(
                    (*(*vt).parser.callbacks)
                        .pm
                        .expect("non-null function pointer"),
                )
                .expect("non-null function pointer")(frag, (*vt).parser.cbdata);
            }
        }
        10 => {
            if !(*vt).parser.callbacks.is_null() && (*(*vt).parser.callbacks).sos.is_some() {
                Some(
                    (*(*vt).parser.callbacks)
                        .sos
                        .expect("non-null function pointer"),
                )
                .expect("non-null function pointer")(frag, (*vt).parser.cbdata);
            }
        }
        0 | 1 | 2 | 3 | 5 | 4 => return,
        _ => {}
    }
    (*vt).parser.string_initial = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn vterm_input_write(
    mut vt: *mut VTerm,
    mut bytes: *const ::core::ffi::c_char,
    mut len: size_t,
) -> size_t {
    let mut pos: size_t = 0 as size_t;
    let mut string_start: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    match (*vt).parser.state as ::core::ffi::c_uint {
        0 | 1 | 2 | 3 | 5 | 4 => {
            string_start = ::core::ptr::null::<::core::ffi::c_char>();
        }
        6 | 7 | 8 | 9 | 10 => {
            string_start = bytes;
        }
        _ => {}
    }
    while pos < len {
        let mut c: uint8_t = *bytes.offset(pos as isize) as uint8_t;
        let mut c1_allowed: bool = (*vt).mode.utf8() == 0;
        's_46: {
            if c as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                || c as ::core::ffi::c_int == 0x7f as ::core::ffi::c_int
            {
                if (*vt).parser.state as ::core::ffi::c_uint
                    >= OSC_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    string_fragment(
                        vt,
                        string_start,
                        bytes.offset(pos as isize).offset_from(string_start) as size_t,
                        false_0 != 0,
                        VTERM_TERMINATOR_ST,
                    );
                    string_start = bytes
                        .offset(pos as isize)
                        .offset(1 as ::core::ffi::c_int as isize);
                }
                if (*vt).parser.emit_nul {
                    do_control(vt, c);
                }
            } else if c as ::core::ffi::c_int == 0x18 as ::core::ffi::c_int
                || c as ::core::ffi::c_int == 0x1a as ::core::ffi::c_int
            {
                (*vt).parser.set_in_esc((false_0 != 0) as bool);
                (*vt).parser.state = NORMAL;
                string_start = ::core::ptr::null::<::core::ffi::c_char>();
                if (*vt).parser.emit_nul {
                    do_control(vt, c);
                }
            } else if c as ::core::ffi::c_int == 0x1b as ::core::ffi::c_int {
                (*vt).parser.intermedlen = 0 as ::core::ffi::c_int;
                if !((*vt).parser.state as ::core::ffi::c_uint
                    >= OSC_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    (*vt).parser.state = NORMAL;
                }
                (*vt).parser.set_in_esc((true_0 != 0) as bool);
            } else {
                if !(c as ::core::ffi::c_int == 0x7 as ::core::ffi::c_int
                    && (*vt).parser.state as ::core::ffi::c_uint
                        >= OSC_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    if (c as ::core::ffi::c_int) < 0x20 as ::core::ffi::c_int {
                        if (*vt).parser.state as ::core::ffi::c_uint
                            == SOS as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            break 's_46;
                        } else {
                            if (*vt).parser.state as ::core::ffi::c_uint
                                >= OSC_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                string_fragment(
                                    vt,
                                    string_start,
                                    bytes.offset(pos as isize).offset_from(string_start) as size_t,
                                    false_0 != 0,
                                    VTERM_TERMINATOR_ST,
                                );
                            }
                            do_control(vt, c);
                            if (*vt).parser.state as ::core::ffi::c_uint
                                >= OSC_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                string_start = bytes
                                    .offset(pos as isize)
                                    .offset(1 as ::core::ffi::c_int as isize);
                            }
                            break 's_46;
                        }
                    }
                }
                let mut string_len: size_t =
                    bytes.offset(pos as isize).offset_from(string_start) as size_t;
                if (*vt).parser.in_esc() {
                    if (*vt).parser.intermedlen == 0
                        && c as ::core::ffi::c_int >= 0x40 as ::core::ffi::c_int
                        && (c as ::core::ffi::c_int) < 0x60 as ::core::ffi::c_int
                        && (!((*vt).parser.state as ::core::ffi::c_uint
                            >= OSC_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint)
                            || c as ::core::ffi::c_int == 0x5c as ::core::ffi::c_int)
                    {
                        c = (c as ::core::ffi::c_int + 0x40 as ::core::ffi::c_int) as uint8_t;
                        c1_allowed = true_0 != 0;
                        if string_len != 0 {
                            '_c2rust_label: {
                                if string_len > 0 as size_t {
                                } else {
                                    __assert_fail(
                                        b"string_len > 0\0".as_ptr() as *const ::core::ffi::c_char,
                                        b"src/nvim/vterm/parser.rs\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        215 as ::core::ffi::c_uint,
                                        __ASSERT_FUNCTION.as_ptr(),
                                    );
                                }
                            };
                            string_len = string_len.wrapping_sub(1 as size_t);
                        }
                        (*vt).parser.set_in_esc((false_0 != 0) as bool);
                    } else {
                        string_start = ::core::ptr::null::<::core::ffi::c_char>();
                        (*vt).parser.state = NORMAL;
                    }
                }
                's_849: {
                    'c_8046: {
                        'c_8670: {
                            match (*vt).parser.state as ::core::ffi::c_uint {
                                1 => {
                                    if c as ::core::ffi::c_int >= 0x3c as ::core::ffi::c_int
                                        && c as ::core::ffi::c_int <= 0x3f as ::core::ffi::c_int
                                    {
                                        if (*vt).parser.v.csi.leaderlen
                                            < CSI_LEADER_MAX - 1 as ::core::ffi::c_int
                                        {
                                            let c2rust_fresh0 = (*vt).parser.v.csi.leaderlen;
                                            (*vt).parser.v.csi.leaderlen =
                                                (*vt).parser.v.csi.leaderlen + 1;
                                            (*vt).parser.v.csi.leader[c2rust_fresh0 as usize] =
                                                c as ::core::ffi::c_char;
                                        }
                                        break 's_849;
                                    } else {
                                        (*vt).parser.v.csi.leader
                                            [(*vt).parser.v.csi.leaderlen as usize] =
                                            0 as ::core::ffi::c_char;
                                        (*vt).parser.v.csi.argi = 0 as ::core::ffi::c_int;
                                        (*vt).parser.v.csi.args[0 as ::core::ffi::c_int as usize] =
                                            CSI_ARG_MISSING as ::core::ffi::c_long;
                                        (*vt).parser.state = CSI_ARGS;
                                    }
                                }
                                2 => {}
                                3 => {
                                    break 'c_8046;
                                }
                                5 => {
                                    if c as ::core::ffi::c_int >= '0' as ::core::ffi::c_int
                                        && c as ::core::ffi::c_int <= '9' as ::core::ffi::c_int
                                    {
                                        if (*vt).parser.v.osc.command == -1 as ::core::ffi::c_int {
                                            (*vt).parser.v.osc.command = 0 as ::core::ffi::c_int;
                                        } else {
                                            (*vt).parser.v.osc.command *= 10 as ::core::ffi::c_int;
                                        }
                                        (*vt).parser.v.osc.command +=
                                            c as ::core::ffi::c_int - '0' as ::core::ffi::c_int;
                                        break 's_849;
                                    } else if c as ::core::ffi::c_int == ';' as ::core::ffi::c_int {
                                        (*vt).parser.state = OSC;
                                        string_start = bytes
                                            .offset(pos as isize)
                                            .offset(1 as ::core::ffi::c_int as isize);
                                        break 's_849;
                                    } else {
                                        string_start = bytes.offset(pos as isize);
                                        string_len = 0 as size_t;
                                        (*vt).parser.state = OSC;
                                        break 'c_8670;
                                    }
                                }
                                4 => {
                                    if (*vt).parser.v.dcs.commandlen < CSI_LEADER_MAX {
                                        let c2rust_fresh2 = (*vt).parser.v.dcs.commandlen;
                                        (*vt).parser.v.dcs.commandlen =
                                            (*vt).parser.v.dcs.commandlen + 1;
                                        (*vt).parser.v.dcs.command[c2rust_fresh2 as usize] =
                                            c as ::core::ffi::c_char;
                                    }
                                    if c as ::core::ffi::c_int >= 0x40 as ::core::ffi::c_int
                                        && c as ::core::ffi::c_int <= 0x7e as ::core::ffi::c_int
                                    {
                                        string_start = bytes
                                            .offset(pos as isize)
                                            .offset(1 as ::core::ffi::c_int as isize);
                                        (*vt).parser.state = DCS_VTERM;
                                    }
                                    break 's_849;
                                }
                                6 | 7 | 8 | 9 | 10 => {
                                    break 'c_8670;
                                }
                                0 => {
                                    if (*vt).parser.in_esc() {
                                        if is_intermed(c) {
                                            if (*vt).parser.intermedlen
                                                < INTERMED_MAX - 1 as ::core::ffi::c_int
                                            {
                                                let c2rust_fresh3 = (*vt).parser.intermedlen;
                                                (*vt).parser.intermedlen =
                                                    (*vt).parser.intermedlen + 1;
                                                (*vt).parser.intermed[c2rust_fresh3 as usize] =
                                                    c as ::core::ffi::c_char;
                                            }
                                        } else if c as ::core::ffi::c_int
                                            >= 0x30 as ::core::ffi::c_int
                                            && (c as ::core::ffi::c_int)
                                                < 0x7f as ::core::ffi::c_int
                                        {
                                            do_escape(vt, c as ::core::ffi::c_char);
                                            (*vt).parser.set_in_esc(false as bool);
                                            (*vt).parser.state = NORMAL;
                                            string_start =
                                                ::core::ptr::null::<::core::ffi::c_char>();
                                        }
                                        break 's_849;
                                    } else {
                                        if c1_allowed as ::core::ffi::c_int != 0
                                            && c as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int
                                            && (c as ::core::ffi::c_int)
                                                < 0xa0 as ::core::ffi::c_int
                                        {
                                            match c as ::core::ffi::c_int {
                                                144 => {
                                                    (*vt).parser.string_initial = true_0 != 0;
                                                    (*vt).parser.v.dcs.commandlen =
                                                        0 as ::core::ffi::c_int;
                                                    (*vt).parser.state = DCS_COMMAND;
                                                    string_start =
                                                        ::core::ptr::null::<::core::ffi::c_char>();
                                                }
                                                152 => {
                                                    (*vt).parser.string_initial = true_0 != 0;
                                                    (*vt).parser.state = SOS;
                                                    string_start =
                                                        ::core::ptr::null::<::core::ffi::c_char>();
                                                    string_start = bytes
                                                        .offset(pos as isize)
                                                        .offset(1 as ::core::ffi::c_int as isize);
                                                }
                                                155 => {
                                                    (*vt).parser.v.csi.leaderlen =
                                                        0 as ::core::ffi::c_int;
                                                    (*vt).parser.state = CSI_LEADER;
                                                    string_start =
                                                        ::core::ptr::null::<::core::ffi::c_char>();
                                                }
                                                157 => {
                                                    (*vt).parser.v.osc.command =
                                                        -1 as ::core::ffi::c_int;
                                                    (*vt).parser.string_initial = true_0 != 0;
                                                    (*vt).parser.state = OSC_COMMAND;
                                                    string_start =
                                                        ::core::ptr::null::<::core::ffi::c_char>();
                                                }
                                                158 => {
                                                    (*vt).parser.string_initial = true_0 != 0;
                                                    (*vt).parser.state = PM;
                                                    string_start =
                                                        ::core::ptr::null::<::core::ffi::c_char>();
                                                    string_start = bytes
                                                        .offset(pos as isize)
                                                        .offset(1 as ::core::ffi::c_int as isize);
                                                }
                                                159 => {
                                                    (*vt).parser.string_initial = true_0 != 0;
                                                    (*vt).parser.state = APC;
                                                    string_start =
                                                        ::core::ptr::null::<::core::ffi::c_char>();
                                                    string_start = bytes
                                                        .offset(pos as isize)
                                                        .offset(1 as ::core::ffi::c_int as isize);
                                                }
                                                _ => {
                                                    do_control(vt, c);
                                                }
                                            }
                                        } else {
                                            let mut eaten: size_t = 0 as size_t;
                                            if !(*vt).parser.callbacks.is_null()
                                                && (*(*vt).parser.callbacks).text.is_some()
                                            {
                                                eaten = Some(
                                                    (*(*vt).parser.callbacks)
                                                        .text
                                                        .expect("non-null function pointer"),
                                                )
                                                .expect("non-null function pointer")(
                                                    bytes.offset(pos as isize),
                                                    len.wrapping_sub(pos),
                                                    (*vt).parser.cbdata,
                                                )
                                                    as size_t;
                                            }
                                            if eaten == 0 {
                                                eaten = 1 as size_t;
                                            }
                                            pos = pos.wrapping_add(eaten.wrapping_sub(1 as size_t));
                                        }
                                        break 's_849;
                                    }
                                }
                                _ => {
                                    break 's_849;
                                }
                            }
                            if c as ::core::ffi::c_int >= '0' as ::core::ffi::c_int
                                && c as ::core::ffi::c_int <= '9' as ::core::ffi::c_int
                            {
                                if (*vt).parser.v.csi.args[(*vt).parser.v.csi.argi as usize]
                                    as ::core::ffi::c_ulong
                                    == CSI_ARG_MISSING
                                {
                                    (*vt).parser.v.csi.args[(*vt).parser.v.csi.argi as usize] =
                                        0 as ::core::ffi::c_long;
                                }
                                (*vt).parser.v.csi.args[(*vt).parser.v.csi.argi as usize] *=
                                    10 as ::core::ffi::c_long;
                                (*vt).parser.v.csi.args[(*vt).parser.v.csi.argi as usize] +=
                                    (c as ::core::ffi::c_int - '0' as ::core::ffi::c_int)
                                        as ::core::ffi::c_long;
                                break 's_849;
                            } else {
                                if c as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
                                    (*vt).parser.v.csi.args[(*vt).parser.v.csi.argi as usize] |=
                                        CSI_ARG_FLAG_MORE as ::core::ffi::c_long;
                                    c = ';' as uint8_t;
                                }
                                if c as ::core::ffi::c_int == ';' as ::core::ffi::c_int {
                                    (*vt).parser.v.csi.argi += 1;
                                    (*vt).parser.v.csi.args[(*vt).parser.v.csi.argi as usize] =
                                        CSI_ARG_MISSING as ::core::ffi::c_long;
                                    break 's_849;
                                } else {
                                    (*vt).parser.v.csi.argi += 1;
                                    (*vt).parser.intermedlen = 0 as ::core::ffi::c_int;
                                    (*vt).parser.state = CSI_INTERMED;
                                    break 'c_8046;
                                }
                            }
                        }
                        if c as ::core::ffi::c_int == 0x7 as ::core::ffi::c_int
                            || c1_allowed as ::core::ffi::c_int != 0
                                && c as ::core::ffi::c_int == 0x9c as ::core::ffi::c_int
                        {
                            string_fragment(
                                vt,
                                string_start,
                                string_len,
                                true_0 != 0,
                                (if c as ::core::ffi::c_int == 0x7 as ::core::ffi::c_int {
                                    VTERM_TERMINATOR_BEL as ::core::ffi::c_int
                                } else {
                                    VTERM_TERMINATOR_ST as ::core::ffi::c_int
                                }) as VTermTerminator,
                            );
                            (*vt).parser.state = NORMAL;
                            string_start = ::core::ptr::null::<::core::ffi::c_char>();
                        }
                        break 's_849;
                    }
                    if is_intermed(c) {
                        if (*vt).parser.intermedlen < INTERMED_MAX - 1 as ::core::ffi::c_int {
                            let c2rust_fresh1 = (*vt).parser.intermedlen;
                            (*vt).parser.intermedlen = (*vt).parser.intermedlen + 1;
                            (*vt).parser.intermed[c2rust_fresh1 as usize] =
                                c as ::core::ffi::c_char;
                        }
                    } else {
                        if c as ::core::ffi::c_int != 0x1b as ::core::ffi::c_int {
                            if c as ::core::ffi::c_int >= 0x40 as ::core::ffi::c_int
                                && c as ::core::ffi::c_int <= 0x7e as ::core::ffi::c_int
                            {
                                (*vt).parser.intermed[(*vt).parser.intermedlen as usize] =
                                    0 as ::core::ffi::c_char;
                                do_csi(vt, c as ::core::ffi::c_char);
                            }
                        }
                        (*vt).parser.state = NORMAL;
                        string_start = ::core::ptr::null::<::core::ffi::c_char>();
                    }
                }
            }
        }
        pos = pos.wrapping_add(1);
    }
    if !string_start.is_null() {
        let mut string_len_0: size_t =
            bytes.offset(pos as isize).offset_from(string_start) as size_t;
        if string_len_0 > 0 as size_t {
            if (*vt).parser.in_esc() {
                string_len_0 = string_len_0.wrapping_sub(1 as size_t);
            }
            string_fragment(
                vt,
                string_start,
                string_len_0,
                false_0 != 0,
                VTERM_TERMINATOR_ST,
            );
        }
    }
    return len;
}
#[no_mangle]
pub unsafe extern "C" fn vterm_parser_set_callbacks(
    mut vt: *mut VTerm,
    mut callbacks: *const VTermParserCallbacks,
    mut user: *mut ::core::ffi::c_void,
) {
    (*vt).parser.callbacks = callbacks;
    (*vt).parser.cbdata = user;
}
pub const CSI_ARG_FLAG_MORE: ::core::ffi::c_uint =
    (1 as ::core::ffi::c_uint) << 31 as ::core::ffi::c_int;
pub const CSI_ARG_MISSING: ::core::ffi::c_ulong = ((1 as ::core::ffi::c_ulong)
    << 31 as ::core::ffi::c_int)
    .wrapping_sub(1 as ::core::ffi::c_ulong);
