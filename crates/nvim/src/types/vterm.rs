#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
//
// These are libvterm's types, Copyright (c) 2008 Paul Evans, under the MIT
// license; the notice is reproduced in licenses/libvterm-LICENSE.txt.
use core::mem::offset_of;

use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ScreenCell {
    pub schar: schar_T,
    pub pen: ScreenPen,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ScreenPen {
    pub fg: VTermColor,
    pub bg: VTermColor,
    pub uri: ::core::ffi::c_int,
    pub bold_underline_italic_blink_reverse_conceal_strike_font_small_baseline_dim_overline_protected_cell_dwl_dhl:
        [u8; 3],
    pub c2rust_padding: [u8; 1],
}
crate::bitfield_accessors! {
    impl ScreenPen.bold_underline_italic_blink_reverse_conceal_strike_font_small_baseline_dim_overline_protected_cell_dwl_dhl {
        0..=0 => bold, set_bold: ::core::ffi::c_uint;
        1..=2 => underline, set_underline: ::core::ffi::c_uint;
        3..=3 => italic, set_italic: ::core::ffi::c_uint;
        4..=4 => blink, set_blink: ::core::ffi::c_uint;
        5..=5 => reverse, set_reverse: ::core::ffi::c_uint;
        6..=6 => conceal, set_conceal: ::core::ffi::c_uint;
        7..=7 => strike, set_strike: ::core::ffi::c_uint;
        8..=11 => font, set_font: ::core::ffi::c_uint;
        12..=12 => small, set_small: ::core::ffi::c_uint;
        13..=14 => baseline, set_baseline: ::core::ffi::c_uint;
        15..=15 => dim, set_dim: ::core::ffi::c_uint;
        16..=16 => overline, set_overline: ::core::ffi::c_uint;
        17..=17 => protected_cell, set_protected_cell: ::core::ffi::c_uint;
        18..=18 => dwl, set_dwl: ::core::ffi::c_uint;
        19..=20 => dhl, set_dhl: ::core::ffi::c_uint;
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTerm {
    pub rows: ::core::ffi::c_int,
    pub cols: ::core::ffi::c_int,
    pub mode: VTerm_mode,
    pub parser: VTerm_parser,
    pub outfunc: Option<VTermOutputCallback>,
    pub outdata: *mut ::core::ffi::c_void,
    pub outbuffer: *mut ::core::ffi::c_char,
    pub outbuffer_len: size_t,
    pub outbuffer_cur: size_t,
    pub tmpbuffer: *mut ::core::ffi::c_char,
    pub tmpbuffer_len: size_t,
    pub state: *mut VTermState,
    pub screen: *mut VTermScreen,
}
pub type VTermAttr = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub union VTermColor {
    pub type_0: uint8_t,
    pub rgb: VTermColor_rgb,
    pub indexed: VTermColor_indexed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermColor_indexed {
    pub type_0: uint8_t,
    pub idx: uint8_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermColor_rgb {
    pub type_0: uint8_t,
    pub red: uint8_t,
    pub green: uint8_t,
    pub blue: uint8_t,
}
pub type VTermDamageSize = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermGlyphInfo {
    pub schar: schar_T,
    pub width: ::core::ffi::c_int,
    pub protected_cell_dwl_dhl: [u8; 1],
    pub c2rust_padding: [u8; 3],
}
crate::bitfield_accessors! {
    impl VTermGlyphInfo.protected_cell_dwl_dhl {
        0..=0 => protected_cell, set_protected_cell: ::core::ffi::c_uint;
        1..=1 => dwl, set_dwl: ::core::ffi::c_uint;
        2..=3 => dhl, set_dhl: ::core::ffi::c_uint;
    }
}
#[derive(Copy, Clone)]
// align(4): `unsigned` bitfields, as on `VTermScreenCellAttrs`.
#[repr(C, align(4))]
pub struct VTermLineInfo {
    pub doublewidth_doubleheight_continuation: [u8; 1],
    pub c2rust_padding: [u8; 3],
}
crate::bitfield_accessors! {
    impl VTermLineInfo.doublewidth_doubleheight_continuation {
        0..=0 => doublewidth, set_doublewidth: ::core::ffi::c_uint;
        1..=2 => doubleheight, set_doubleheight: ::core::ffi::c_uint;
        3..=3 => continuation, set_continuation: ::core::ffi::c_uint;
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermParserCallbacks {
    pub text: Option<
        unsafe extern "C" fn(
            *const ::core::ffi::c_char,
            size_t,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub control:
        Option<unsafe extern "C" fn(uint8_t, *mut ::core::ffi::c_void) -> ::core::ffi::c_int>,
    pub escape: Option<
        unsafe extern "C" fn(
            *const ::core::ffi::c_char,
            size_t,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub csi: Option<
        unsafe extern "C" fn(
            *const ::core::ffi::c_char,
            *const ::core::ffi::c_long,
            ::core::ffi::c_int,
            *const ::core::ffi::c_char,
            ::core::ffi::c_char,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub osc: Option<
        unsafe extern "C" fn(
            ::core::ffi::c_int,
            VTermStringFragment,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub dcs: Option<
        unsafe extern "C" fn(
            *const ::core::ffi::c_char,
            size_t,
            VTermStringFragment,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub apc: Option<
        unsafe extern "C" fn(VTermStringFragment, *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    >,
    pub pm: Option<
        unsafe extern "C" fn(VTermStringFragment, *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    >,
    pub sos: Option<
        unsafe extern "C" fn(VTermStringFragment, *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    >,
    pub resize: Option<
        unsafe extern "C" fn(
            ::core::ffi::c_int,
            ::core::ffi::c_int,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermPos {
    pub row: ::core::ffi::c_int,
    pub col: ::core::ffi::c_int,
}
pub type VTermProp = ::core::ffi::c_uint;
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct VTermRect {
    pub start_row: ::core::ffi::c_int,
    pub end_row: ::core::ffi::c_int,
    pub start_col: ::core::ffi::c_int,
    pub end_col: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermScreen {
    pub vt: *mut VTerm,
    pub state: *mut VTermState,
    pub callbacks: *const VTermScreenCallbacks,
    pub cbdata: *mut ::core::ffi::c_void,
    pub damage_merge: VTermDamageSize,
    pub damaged: VTermRect,
    pub pending_scrollrect: VTermRect,
    pub pending_scroll_downward: ::core::ffi::c_int,
    pub pending_scroll_rightward: ::core::ffi::c_int,
    pub rows: ::core::ffi::c_int,
    pub cols: ::core::ffi::c_int,
    pub global_reverse_reflow: [u8; 1],
    pub c2rust_padding: [u8; 3],
    pub buffers: [*mut ScreenCell; 2],
    pub buffer: *mut ScreenCell,
    pub sb_buffer: *mut VTermScreenCell,
    pub pen: ScreenPen,
}
crate::bitfield_accessors! {
    impl VTermScreen.global_reverse_reflow {
        0..=0 => global_reverse, set_global_reverse: ::core::ffi::c_uint;
        1..=1 => reflow, set_reflow: ::core::ffi::c_uint;
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermScreenCallbacks {
    pub damage:
        Option<unsafe extern "C" fn(VTermRect, *mut ::core::ffi::c_void) -> ::core::ffi::c_int>,
    pub moverect: Option<
        unsafe extern "C" fn(VTermRect, VTermRect, *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    >,
    pub movecursor: Option<
        unsafe extern "C" fn(
            VTermPos,
            VTermPos,
            ::core::ffi::c_int,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub settermprop: Option<
        unsafe extern "C" fn(
            VTermProp,
            *mut VTermValue,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub bell: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ::core::ffi::c_int>,
    pub resize: Option<
        unsafe extern "C" fn(
            ::core::ffi::c_int,
            ::core::ffi::c_int,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub theme:
        Option<unsafe extern "C" fn(*mut bool, *mut ::core::ffi::c_void) -> ::core::ffi::c_int>,
    pub sb_pushline: Option<
        unsafe extern "C" fn(
            ::core::ffi::c_int,
            *const VTermScreenCell,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub sb_popline: Option<
        unsafe extern "C" fn(
            ::core::ffi::c_int,
            *mut VTermScreenCell,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub sb_clear: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ::core::ffi::c_int>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermScreenCell {
    pub schar: schar_T,
    pub width: ::core::ffi::c_char,
    pub attrs: VTermScreenCellAttrs,
    pub fg: VTermColor,
    pub bg: VTermColor,
    pub uri: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
// align(4) is the C ABI, not a choice. Every field here is an `unsigned`
// bitfield, so the C lays the struct out in 4-byte storage units and gives it
// 4-byte alignment; c2rust turned the bits into a byte array, which aligns to
// 1. The size is the same either way, so the mistake is invisible until the
// struct is embedded in another -- here it shifted `fg`/`bg`/`uri` in
// `VTermScreenCell`. Every bitfield-only struct below carries the same
// attribute for the same reason.
#[repr(C, align(4))]
pub struct VTermScreenCellAttrs {
    pub bold_underline_italic_blink_reverse_conceal_strike_font_dwl_dhl_small_baseline_dim_overline:
        [u8; 3],
    pub c2rust_padding: [u8; 1],
}
crate::bitfield_accessors! {
    impl VTermScreenCellAttrs.bold_underline_italic_blink_reverse_conceal_strike_font_dwl_dhl_small_baseline_dim_overline {
        0..=0 => bold, set_bold: ::core::ffi::c_uint;
        1..=2 => underline, set_underline: ::core::ffi::c_uint;
        3..=3 => italic, set_italic: ::core::ffi::c_uint;
        4..=4 => blink, set_blink: ::core::ffi::c_uint;
        5..=5 => reverse, set_reverse: ::core::ffi::c_uint;
        6..=6 => conceal, set_conceal: ::core::ffi::c_uint;
        7..=7 => strike, set_strike: ::core::ffi::c_uint;
        8..=11 => font, set_font: ::core::ffi::c_uint;
        12..=12 => dwl, set_dwl: ::core::ffi::c_uint;
        13..=14 => dhl, set_dhl: ::core::ffi::c_uint;
        15..=15 => small, set_small: ::core::ffi::c_uint;
        16..=17 => baseline, set_baseline: ::core::ffi::c_uint;
        18..=18 => dim, set_dim: ::core::ffi::c_uint;
        19..=19 => overline, set_overline: ::core::ffi::c_uint;
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermSelectionCallbacks {
    pub set: Option<
        unsafe extern "C" fn(
            VTermSelectionMask,
            VTermStringFragment,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub query: Option<
        unsafe extern "C" fn(VTermSelectionMask, *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    >,
}
pub type VTermSelectionMask = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermState {
    pub vt: *mut VTerm,
    pub callbacks: *const VTermStateCallbacks,
    pub cbdata: *mut ::core::ffi::c_void,
    pub fallbacks: *const VTermStateFallbacks,
    pub fbdata: *mut ::core::ffi::c_void,
    pub rows: ::core::ffi::c_int,
    pub cols: ::core::ffi::c_int,
    pub pos: VTermPos,
    pub at_phantom: ::core::ffi::c_int,
    pub scrollregion_top: ::core::ffi::c_int,
    pub scrollregion_bottom: ::core::ffi::c_int,
    pub scrollregion_left: ::core::ffi::c_int,
    pub scrollregion_right: ::core::ffi::c_int,
    pub tabstops: *mut uint8_t,
    pub lineinfos: [*mut VTermLineInfo; 2],
    pub lineinfo: *mut VTermLineInfo,
    pub mouse_col: ::core::ffi::c_int,
    pub mouse_row: ::core::ffi::c_int,
    pub mouse_buttons: ::core::ffi::c_int,
    pub mouse_flags: ::core::ffi::c_int,
    pub mouse_protocol: VTermState_mouse_protocol,
    pub grapheme_buf: [::core::ffi::c_char; 32],
    pub grapheme_len: size_t,
    pub grapheme_last: uint32_t,
    pub grapheme_state: GraphemeState,
    pub combine_width: ::core::ffi::c_int,
    pub combine_pos: VTermPos,
    pub mode: VTermState_mode,
    pub encoding: [VTermEncodingInstance; 4],
    pub encoding_utf8: VTermEncodingInstance,
    pub gl_set: ::core::ffi::c_int,
    pub gr_set: ::core::ffi::c_int,
    pub gsingle_set: ::core::ffi::c_int,
    pub pen: VTermPen,
    pub default_fg: VTermColor,
    pub default_bg: VTermColor,
    pub colors: [VTermColor; 16],
    pub bold_is_highbright: ::core::ffi::c_int,
    pub protected_cell: [u8; 1],
    pub c2rust_padding: [u8; 3],
    pub saved: VTermState_saved,
    pub tmp: VTermState_tmp,
    pub selection: VTermState_selection,
    pub key_encoding_stacks: [VTermKeyEncodingStack; 2],
}
crate::bitfield_accessors! {
    impl VTermState.protected_cell {
        0..=0 => protected_cell, set_protected_cell: ::core::ffi::c_uint;
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermStateCallbacks {
    pub putglyph: Option<
        unsafe extern "C" fn(
            *mut VTermGlyphInfo,
            VTermPos,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub movecursor: Option<
        unsafe extern "C" fn(
            VTermPos,
            VTermPos,
            ::core::ffi::c_int,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub scrollrect: Option<
        unsafe extern "C" fn(
            VTermRect,
            ::core::ffi::c_int,
            ::core::ffi::c_int,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub moverect: Option<
        unsafe extern "C" fn(VTermRect, VTermRect, *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    >,
    pub erase: Option<
        unsafe extern "C" fn(
            VTermRect,
            ::core::ffi::c_int,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub initpen: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ::core::ffi::c_int>,
    pub setpenattr: Option<
        unsafe extern "C" fn(
            VTermAttr,
            *mut VTermValue,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub settermprop: Option<
        unsafe extern "C" fn(
            VTermProp,
            *mut VTermValue,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub bell: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ::core::ffi::c_int>,
    pub resize: Option<
        unsafe extern "C" fn(
            ::core::ffi::c_int,
            ::core::ffi::c_int,
            *mut VTermStateFields,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub theme:
        Option<unsafe extern "C" fn(*mut bool, *mut ::core::ffi::c_void) -> ::core::ffi::c_int>,
    pub setlineinfo: Option<
        unsafe extern "C" fn(
            ::core::ffi::c_int,
            *const VTermLineInfo,
            *const VTermLineInfo,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub sb_clear: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ::core::ffi::c_int>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermStateFallbacks {
    pub control:
        Option<unsafe extern "C" fn(uint8_t, *mut ::core::ffi::c_void) -> ::core::ffi::c_int>,
    pub csi: Option<
        unsafe extern "C" fn(
            *const ::core::ffi::c_char,
            *const ::core::ffi::c_long,
            ::core::ffi::c_int,
            *const ::core::ffi::c_char,
            ::core::ffi::c_char,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub osc: Option<
        unsafe extern "C" fn(
            ::core::ffi::c_int,
            VTermStringFragment,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub dcs: Option<
        unsafe extern "C" fn(
            *const ::core::ffi::c_char,
            size_t,
            VTermStringFragment,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub apc: Option<
        unsafe extern "C" fn(VTermStringFragment, *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    >,
    pub pm: Option<
        unsafe extern "C" fn(VTermStringFragment, *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    >,
    pub sos: Option<
        unsafe extern "C" fn(VTermStringFragment, *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    >,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermStateFields {
    pub pos: VTermPos,
    pub lineinfos: [*mut VTermLineInfo; 2],
}
#[derive(Copy, Clone)]
// align(4): `unsigned` bitfields, as on `VTermScreenCellAttrs`.
#[repr(C, align(4))]
pub struct VTermState_mode {
    pub keypad_cursor_autowrap_insert_newline_cursor_visible_cursor_blink_cursor_shape_alt_screen_origin_screen_leftrightmargin_bracketpaste_report_focus_theme_updates_synchronized_output:
        [u8; 3],
    pub c2rust_padding: [u8; 1],
}
crate::bitfield_accessors! {
    impl VTermState_mode.keypad_cursor_autowrap_insert_newline_cursor_visible_cursor_blink_cursor_shape_alt_screen_origin_screen_leftrightmargin_bracketpaste_report_focus_theme_updates_synchronized_output {
        0..=0 => keypad, set_keypad: ::core::ffi::c_uint;
        1..=1 => cursor, set_cursor: ::core::ffi::c_uint;
        2..=2 => autowrap, set_autowrap: ::core::ffi::c_uint;
        3..=3 => insert, set_insert: ::core::ffi::c_uint;
        4..=4 => newline, set_newline: ::core::ffi::c_uint;
        5..=5 => cursor_visible, set_cursor_visible: ::core::ffi::c_uint;
        6..=6 => cursor_blink, set_cursor_blink: ::core::ffi::c_uint;
        7..=8 => cursor_shape, set_cursor_shape: ::core::ffi::c_uint;
        9..=9 => alt_screen, set_alt_screen: ::core::ffi::c_uint;
        10..=10 => origin, set_origin: ::core::ffi::c_uint;
        11..=11 => screen, set_screen: ::core::ffi::c_uint;
        12..=12 => leftrightmargin, set_leftrightmargin: ::core::ffi::c_uint;
        13..=13 => bracketpaste, set_bracketpaste: ::core::ffi::c_uint;
        14..=14 => report_focus, set_report_focus: ::core::ffi::c_uint;
        15..=15 => theme_updates, set_theme_updates: ::core::ffi::c_uint;
        16..=16 => synchronized_output, set_synchronized_output: ::core::ffi::c_uint;
    }
}
pub type VTermState_mouse_protocol = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermState_saved {
    pub pos: VTermPos,
    pub pen: VTermPen,
    pub mode: VTermState_saved_mode,
}
#[derive(Copy, Clone)]
// align(4): `unsigned` bitfields, as on `VTermScreenCellAttrs`.
#[repr(C, align(4))]
pub struct VTermState_saved_mode {
    pub cursor_visible_cursor_blink_cursor_shape: [u8; 1],
    pub c2rust_padding: [u8; 3],
}
crate::bitfield_accessors! {
    impl VTermState_saved_mode.cursor_visible_cursor_blink_cursor_shape {
        0..=0 => cursor_visible, set_cursor_visible: ::core::ffi::c_uint;
        1..=1 => cursor_blink, set_cursor_blink: ::core::ffi::c_uint;
        2..=3 => cursor_shape, set_cursor_shape: ::core::ffi::c_uint;
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermState_selection {
    pub callbacks: *const VTermSelectionCallbacks,
    pub user: *mut ::core::ffi::c_void,
    pub buffer: *mut ::core::ffi::c_char,
    pub buflen: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union VTermState_tmp {
    pub decrqss: [::core::ffi::c_char; 4],
    pub selection: VTermState_tmp_selection,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermState_tmp_selection {
    pub mask: uint16_t,
    pub state: [u8; 1],
    pub c2rust_padding: [u8; 1],
    pub recvpartial: uint32_t,
    pub sendpartial: uint32_t,
}
crate::bitfield_accessors! {
    impl VTermState_tmp_selection.state {
        0..=7 => state, set_state: VTermState_tmp_selection_state;
    }
}
pub type VTermState_tmp_selection_state = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermStringFragment {
    pub str: *const ::core::ffi::c_char,
    pub len_initial_final_0: [u8; 4],
    pub terminator: VTermTerminator,
}
crate::bitfield_accessors! {
    impl VTermStringFragment.len_initial_final_0 {
        0..=29 => len, set_len: size_t;
        30..=30 => initial, set_initial: bool;
        31..=31 => final_0, set_final_0: bool;
    }
}
impl VTermStringFragment {
    /// Whether the fragment carries no bytes. The companion `len_without_is_empty`
    /// asks for; `len` above is a 30-bit C field, but the pair reads the same.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
pub type VTermTerminator = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub union VTermValue {
    pub boolean: ::core::ffi::c_int,
    pub number: ::core::ffi::c_int,
    pub string: VTermStringFragment,
    pub color: VTermColor,
}
pub type VTermValueType = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
// align(4): `unsigned` bitfields, as on `VTermScreenCellAttrs`.
#[repr(C, align(4))]
pub struct VTerm_mode {
    pub utf8_ctrl8bit: [u8; 1],
    pub c2rust_padding: [u8; 3],
}
crate::bitfield_accessors! {
    impl VTerm_mode.utf8_ctrl8bit {
        0..=0 => utf8, set_utf8: ::core::ffi::c_uint;
        1..=1 => ctrl8bit, set_ctrl8bit: ::core::ffi::c_uint;
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTerm_parser {
    pub state: VTermParserState,
    pub in_esc: [u8; 1],
    pub c2rust_padding: [u8; 3],
    pub intermedlen: ::core::ffi::c_int,
    pub intermed: [::core::ffi::c_char; 16],
    pub v: VTerm_parser_v,
    pub callbacks: *const VTermParserCallbacks,
    pub cbdata: *mut ::core::ffi::c_void,
    pub string_initial: bool,
    pub emit_nul: bool,
}
crate::bitfield_accessors! {
    impl VTerm_parser.in_esc {
        0..=0 => in_esc, set_in_esc: bool;
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union VTerm_parser_v {
    pub csi: VTerm_parser_v_csi,
    pub osc: VTerm_parser_v_osc,
    pub dcs: VTerm_parser_v_dcs,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTerm_parser_v_csi {
    pub leaderlen: ::core::ffi::c_int,
    pub leader: [::core::ffi::c_char; 16],
    pub argi: ::core::ffi::c_int,
    pub args: [::core::ffi::c_long; 32],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTerm_parser_v_dcs {
    pub commandlen: ::core::ffi::c_int,
    pub command: [::core::ffi::c_char; 16],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTerm_parser_v_osc {
    pub command: ::core::ffi::c_int,
}

// ---------------------------------------------------------------------------
// The layout, asserted.
//
// These types cross a C ABI in three directions: the emulator hands them to
// `terminal.rs`, the unit specs declare them through LuaJIT's FFI, and
// `unit-fixtures.so` compiles C against those same declarations. `repr(C)`
// fixes the *rules* the compiler lays them out by, not the answer, so a
// field that widens or a pair that swaps stays a valid `repr(C)` type and
// silently disagrees with everything on the other side of the boundary.
//
// The numbers are what x86-64 System V (and any other LP64 target) gives
// libvterm's own declarations. They are the same sizes, alignments and
// offsets the emulator's differential reads back out of a hand-written FFI
// declaration of these types, so a change that moves one moves the other.
// ---------------------------------------------------------------------------
const _: () = {
    assert!(size_of::<VTermPos>() == 8 && align_of::<VTermPos>() == 4);
    assert!(size_of::<VTermRect>() == 16 && align_of::<VTermRect>() == 4);
    assert!(size_of::<VTermColor>() == 4 && align_of::<VTermColor>() == 1);
    assert!(size_of::<VTermColor_rgb>() == 4 && align_of::<VTermColor_rgb>() == 1);
    assert!(size_of::<VTermColor_indexed>() == 2 && align_of::<VTermColor_indexed>() == 1);
    assert!(size_of::<VTermStringFragment>() == 16 && align_of::<VTermStringFragment>() == 8);
    assert!(size_of::<VTermValue>() == 16 && align_of::<VTermValue>() == 8);
    assert!(size_of::<VTermGlyphInfo>() == 12 && align_of::<VTermGlyphInfo>() == 4);
    assert!(size_of::<VTermLineInfo>() == 4 && align_of::<VTermLineInfo>() == 4);
    assert!(size_of::<VTermScreenCellAttrs>() == 4 && align_of::<VTermScreenCellAttrs>() == 4);
    assert!(size_of::<VTermScreenCell>() == 24 && align_of::<VTermScreenCell>() == 4);
    assert!(size_of::<ScreenPen>() == 16 && align_of::<ScreenPen>() == 4);
    assert!(size_of::<ScreenCell>() == 20 && align_of::<ScreenCell>() == 4);
    assert!(size_of::<VTermState_mode>() == 4 && align_of::<VTermState_mode>() == 4);
    assert!(size_of::<VTermState_saved_mode>() == 4 && align_of::<VTermState_saved_mode>() == 4);
    assert!(size_of::<VTermState_saved>() == 28 && align_of::<VTermState_saved>() == 4);
    assert!(
        size_of::<VTermState_tmp_selection>() == 12 && align_of::<VTermState_tmp_selection>() == 4
    );
    assert!(size_of::<VTermState_tmp>() == 12 && align_of::<VTermState_tmp>() == 4);
    assert!(size_of::<VTermState_selection>() == 32 && align_of::<VTermState_selection>() == 8);
    assert!(size_of::<VTermStateFields>() == 24 && align_of::<VTermStateFields>() == 8);
    assert!(size_of::<VTerm_mode>() == 4 && align_of::<VTerm_mode>() == 4);
    assert!(size_of::<VTerm_parser_v_csi>() == 280 && align_of::<VTerm_parser_v_csi>() == 8);
    assert!(size_of::<VTerm_parser_v_osc>() == 4 && align_of::<VTerm_parser_v_osc>() == 4);
    assert!(size_of::<VTerm_parser_v_dcs>() == 20 && align_of::<VTerm_parser_v_dcs>() == 4);
    assert!(size_of::<VTerm_parser_v>() == 280 && align_of::<VTerm_parser_v>() == 8);
    assert!(size_of::<VTerm_parser>() == 336 && align_of::<VTerm_parser>() == 8);
    assert!(size_of::<VTermParserCallbacks>() == 80 && align_of::<VTermParserCallbacks>() == 8);
    assert!(size_of::<VTermStateCallbacks>() == 104 && align_of::<VTermStateCallbacks>() == 8);
    assert!(size_of::<VTermStateFallbacks>() == 56 && align_of::<VTermStateFallbacks>() == 8);
    assert!(size_of::<VTermScreenCallbacks>() == 80 && align_of::<VTermScreenCallbacks>() == 8);
    assert!(
        size_of::<VTermSelectionCallbacks>() == 16 && align_of::<VTermSelectionCallbacks>() == 8
    );
    assert!(size_of::<VTermState>() == 544 && align_of::<VTermState>() == 8);
    assert!(size_of::<VTermScreen>() == 136 && align_of::<VTermScreen>() == 8);
    assert!(size_of::<VTerm>() == 424 && align_of::<VTerm>() == 8);

    assert!(offset_of!(VTermScreenCell, schar) == 0);
    assert!(offset_of!(VTermScreenCell, width) == 4);
    assert!(offset_of!(VTermScreenCell, attrs) == 8);
    assert!(offset_of!(VTermScreenCell, fg) == 12);
    assert!(offset_of!(VTermScreenCell, bg) == 16);
    assert!(offset_of!(VTermScreenCell, uri) == 20);
    assert!(offset_of!(ScreenCell, schar) == 0);
    assert!(offset_of!(ScreenCell, pen) == 4);
    assert!(offset_of!(ScreenPen, fg) == 0);
    assert!(offset_of!(ScreenPen, bg) == 4);
    assert!(offset_of!(ScreenPen, uri) == 8);
    assert!(offset_of!(VTermGlyphInfo, schar) == 0);
    assert!(offset_of!(VTermGlyphInfo, width) == 4);
    assert!(offset_of!(VTermStringFragment, str) == 0);
    assert!(offset_of!(VTermStringFragment, terminator) == 12);
    assert!(offset_of!(VTermState, vt) == 0);
    assert!(offset_of!(VTermState, callbacks) == 8);
    assert!(offset_of!(VTermState, cbdata) == 16);
    assert!(offset_of!(VTermState, fallbacks) == 24);
    assert!(offset_of!(VTermState, fbdata) == 32);
    assert!(offset_of!(VTermState, rows) == 40);
    assert!(offset_of!(VTermState, cols) == 44);
    assert!(offset_of!(VTermState, pos) == 48);
    assert!(offset_of!(VTermState, at_phantom) == 56);
    assert!(offset_of!(VTermState, scrollregion_top) == 60);
    assert!(offset_of!(VTermState, scrollregion_bottom) == 64);
    assert!(offset_of!(VTermState, scrollregion_left) == 68);
    assert!(offset_of!(VTermState, scrollregion_right) == 72);
    assert!(offset_of!(VTermState, tabstops) == 80);
    assert!(offset_of!(VTermState, lineinfos) == 88);
    assert!(offset_of!(VTermState, lineinfo) == 104);
    assert!(offset_of!(VTermState, mouse_col) == 112);
    assert!(offset_of!(VTermState, mouse_row) == 116);
    assert!(offset_of!(VTermState, mouse_buttons) == 120);
    assert!(offset_of!(VTermState, mouse_flags) == 124);
    assert!(offset_of!(VTermState, mouse_protocol) == 128);
    assert!(offset_of!(VTermState, grapheme_buf) == 132);
    assert!(offset_of!(VTermState, grapheme_len) == 168);
    assert!(offset_of!(VTermState, grapheme_last) == 176);
    assert!(offset_of!(VTermState, grapheme_state) == 180);
    assert!(offset_of!(VTermState, combine_width) == 184);
    assert!(offset_of!(VTermState, combine_pos) == 188);
    assert!(offset_of!(VTermState, mode) == 196);
    assert!(offset_of!(VTermState, encoding) == 200);
    assert!(offset_of!(VTermState, encoding_utf8) == 296);
    assert!(offset_of!(VTermState, gl_set) == 320);
    assert!(offset_of!(VTermState, gr_set) == 324);
    assert!(offset_of!(VTermState, gsingle_set) == 328);
    assert!(offset_of!(VTermState, pen) == 332);
    assert!(offset_of!(VTermState, default_fg) == 348);
    assert!(offset_of!(VTermState, default_bg) == 352);
    assert!(offset_of!(VTermState, colors) == 356);
    assert!(offset_of!(VTermState, bold_is_highbright) == 420);
    assert!(offset_of!(VTermState, saved) == 428);
    assert!(offset_of!(VTermState, tmp) == 456);
    assert!(offset_of!(VTermState, selection) == 472);
    assert!(offset_of!(VTermState, key_encoding_stacks) == 504);
    assert!(offset_of!(VTermScreen, vt) == 0);
    assert!(offset_of!(VTermScreen, state) == 8);
    assert!(offset_of!(VTermScreen, callbacks) == 16);
    assert!(offset_of!(VTermScreen, cbdata) == 24);
    assert!(offset_of!(VTermScreen, damage_merge) == 32);
    assert!(offset_of!(VTermScreen, damaged) == 36);
    assert!(offset_of!(VTermScreen, pending_scrollrect) == 52);
    assert!(offset_of!(VTermScreen, pending_scroll_downward) == 68);
    assert!(offset_of!(VTermScreen, pending_scroll_rightward) == 72);
    assert!(offset_of!(VTermScreen, rows) == 76);
    assert!(offset_of!(VTermScreen, cols) == 80);
    assert!(offset_of!(VTermScreen, buffers) == 88);
    assert!(offset_of!(VTermScreen, buffer) == 104);
    assert!(offset_of!(VTermScreen, sb_buffer) == 112);
    assert!(offset_of!(VTermScreen, pen) == 120);
    assert!(offset_of!(VTerm, rows) == 0);
    assert!(offset_of!(VTerm, cols) == 4);
    assert!(offset_of!(VTerm, mode) == 8);
    assert!(offset_of!(VTerm, parser) == 16);
    assert!(offset_of!(VTerm, outfunc) == 352);
    assert!(offset_of!(VTerm, outdata) == 360);
    assert!(offset_of!(VTerm, outbuffer) == 368);
    assert!(offset_of!(VTerm, outbuffer_len) == 376);
    assert!(offset_of!(VTerm, outbuffer_cur) == 384);
    assert!(offset_of!(VTerm, tmpbuffer) == 392);
    assert!(offset_of!(VTerm, tmpbuffer_len) == 400);
    assert!(offset_of!(VTerm, state) == 408);
    assert!(offset_of!(VTerm, screen) == 416);
    assert!(offset_of!(VTerm_parser, state) == 0);
    assert!(offset_of!(VTerm_parser, intermedlen) == 8);
    assert!(offset_of!(VTerm_parser, intermed) == 12);
    assert!(offset_of!(VTerm_parser, v) == 32);
    assert!(offset_of!(VTerm_parser, callbacks) == 312);
    assert!(offset_of!(VTerm_parser, cbdata) == 320);
    assert!(offset_of!(VTermState_saved, pos) == 0);
    assert!(offset_of!(VTermState_saved, pen) == 8);
    assert!(offset_of!(VTermState_saved, mode) == 24);
};
