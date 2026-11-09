#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// c2rust's names for `VTermState`'s anonymous members, inside libvterm's
// ABI.
#![allow(non_camel_case_types)]

//! libvterm's public types -- the terminal, its state and screen, cells, pens,
//! colours -- laid out exactly as the library's C ABI lays them out.

// These are libvterm's types, Copyright (c) 2008 Paul Evans, under the MIT
// license; the notice is reproduced in licenses/libvterm-LICENSE.txt.
//
// A `_pad` field is the tail of a bitfield storage unit: C rounds a bitfield
// run up to its storage unit, so the bytes exist on both sides and the
// struct's layout -- pinned by `tools/ffigen/unit-cdefs.h`, which the unit
// suite's LuaJIT `ffi.cdef` and C fixtures both compile against -- depends on
// them. `ffigen` drops the field when it re-expands the run into real C
// bitfield members, and matches it by that exact name.
use crate::mbyte::GraphemeState;
use crate::os::libc::{size_t, uint8_t, uint16_t, uint32_t};
use crate::types::ScreenChar;
use crate::vterm::internal::{
    VTermEncodingInstance, VTermKeyEncodingStack, VTermParserState, VTermPen,
};
use core::mem::offset_of;

/// The C callback a consumer installs to receive the bytes the terminal
/// writes back.
pub type VTermOutputCallback =
    unsafe extern "C" fn(*const ::core::ffi::c_char, size_t, *mut ::core::ffi::c_void) -> ();
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct ScreenCell {
    pub schar: ScreenChar,
    pub pen: ScreenPen,
}
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct ScreenPen {
    pub fg: VTermColor,
    pub bg: VTermColor,
    pub uri: ::core::ffi::c_int,
    pub bold_underline_italic_blink_reverse_conceal_strike_font_small_baseline_dim_overline_protected_cell_dwl_dhl:
        [u8; 3],
    pub _pad: [u8; 1],
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
/// A terminal. What C reads of it -- the size, the mode bits and the parser
/// block, through the unit fixtures and the battery's own declaration -- keeps
/// libvterm's layout; everything else it owns is Rust's, behind `inner`.
#[repr(C)]
#[derive(Debug)]
pub struct VTerm {
    pub rows: ::core::ffi::c_int,
    pub cols: ::core::ffi::c_int,
    pub mode: VTerm_mode,
    pub parser: VTerm_parser,
    pub(crate) inner: Box<crate::vterm::vterm::VTermInner>,
}
pub type VTermAttr = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub union VTermColor {
    pub type_0: uint8_t,
    pub rgb: VTermColor_rgb,
    pub indexed: VTermColor_indexed,
}
/// The type byte names the live arm, so this prints that arm and no other.
impl ::core::fmt::Debug for VTermColor {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        use crate::vterm::vterm::{VTERM_COLOR_INDEXED, VTERM_COLOR_TYPE_MASK};
        let flags = self.type_byte();
        let mut out = f.debug_struct("VTermColor");
        out.field("type", &flags);
        if flags & VTERM_COLOR_TYPE_MASK == VTERM_COLOR_INDEXED {
            out.field("index", &self.index());
        } else {
            out.field("rgb", &self.channels());
        }
        out.finish()
    }
}
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct VTermColor_indexed {
    pub type_0: uint8_t,
    pub idx: uint8_t,
}
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct VTermColor_rgb {
    pub type_0: uint8_t,
    pub red: uint8_t,
    pub green: uint8_t,
    pub blue: uint8_t,
}
pub type VTermDamageSize = ::core::ffi::c_uint;
#[repr(C)]
#[derive(Debug)]
pub struct VTermGlyphInfo {
    pub schar: ScreenChar,
    pub width: ::core::ffi::c_int,
    pub protected_cell_dwl_dhl: [u8; 1],
    pub _pad: [u8; 3],
}
crate::bitfield_accessors! {
    impl VTermGlyphInfo.protected_cell_dwl_dhl {
        0..=0 => protected_cell, set_protected_cell: ::core::ffi::c_uint;
        1..=1 => dwl, set_dwl: ::core::ffi::c_uint;
        2..=3 => dhl, set_dhl: ::core::ffi::c_uint;
    }
}
#[derive(Copy, Clone, Debug)]
// align(4): `unsigned` bitfields, as on `VTermScreenCellAttrs`.
#[repr(C, align(4))]
pub struct VTermLineInfo {
    pub doublewidth_doubleheight_continuation: [u8; 1],
    pub _pad: [u8; 3],
}
crate::bitfield_accessors! {
    impl VTermLineInfo.doublewidth_doubleheight_continuation {
        0..=0 => doublewidth, set_doublewidth: ::core::ffi::c_uint;
        1..=2 => doubleheight, set_doubleheight: ::core::ffi::c_uint;
        3..=3 => continuation, set_continuation: ::core::ffi::c_uint;
    }
}
#[repr(C)]
#[derive(Debug)]
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
#[derive(Copy, Clone, Debug)]
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
/// A terminal's screen. The C side reads the fields up to `reflow`; the cell
/// grids and the host's callbacks are owned behind `owned`.
#[repr(C)]
#[derive(Debug)]
pub struct VTermScreen {
    /// The terminal this screen belongs to. Only the C entry points read it,
    /// to get from a screen pointer back to the terminal that owns it.
    pub vt: *mut VTerm,
    /// The terminal's state machine, for a C reader; the screen itself is
    /// handed what it needs of the state by the call that reaches it.
    pub state: *mut VTermState,
    /// The callback table a C host installed, as C sees it.
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
    pub _pad: [u8; 3],
    pub(crate) owned: Box<crate::vterm::screen::ScreenOwned>,
    pub pen: ScreenPen,
}
crate::bitfield_accessors! {
    impl VTermScreen.global_reverse_reflow {
        0..=0 => global_reverse, set_global_reverse: ::core::ffi::c_uint;
        1..=1 => reflow, set_reflow: ::core::ffi::c_uint;
    }
}
#[derive(Copy, Clone, Debug)]
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
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct VTermScreenCell {
    pub schar: ScreenChar,
    pub width: ::core::ffi::c_char,
    pub attrs: VTermScreenCellAttrs,
    pub fg: VTermColor,
    pub bg: VTermColor,
    pub uri: ::core::ffi::c_int,
}
#[derive(Copy, Clone, Debug)]
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
    pub _pad: [u8; 1],
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
#[repr(C)]
#[derive(Debug)]
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
/// The state machine. Its layout is libvterm's, field for field, because the
/// unit fixture and the battery read nearly all of it from C; what it owns
/// (the tab stops, the line marks, the selection buffer and the Rust
/// callbacks) lives behind `owned`, in the slot the tab-stop pointer had.
#[repr(C)]
#[derive(Debug)]
pub struct VTermState {
    /// The terminal this state belongs to. Only the C entry points read it,
    /// to get from a state pointer back to the terminal that owns it.
    pub vt: *mut VTerm,
    /// The callback and fallback tables a C consumer installed, as C sees
    /// them.
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
    pub(crate) owned: Box<crate::vterm::state::StateOwned>,
    /// Where libvterm kept its two line-mark arrays and the live one; they
    /// are in `owned` now, and the slots stay so that every field after them
    /// is where C reads it.
    pub _lineinfo_slots: [usize; 3],
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
    pub _pad: [u8; 3],
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
#[repr(C)]
#[derive(Debug)]
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
#[repr(C)]
#[derive(Debug)]
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
#[repr(C)]
#[derive(Debug)]
pub struct VTermStateFields {
    pub pos: VTermPos,
    pub lineinfos: [*mut VTermLineInfo; 2],
}
#[derive(Copy, Clone, Debug)]
// align(4): `unsigned` bitfields, as on `VTermScreenCellAttrs`.
#[repr(C, align(4))]
pub struct VTermState_mode {
    pub keypad_cursor_autowrap_insert_newline_cursor_visible_cursor_blink_cursor_shape_alt_screen_origin_screen_leftrightmargin_bracketpaste_report_focus_theme_updates_synchronized_output:
        [u8; 3],
    pub _pad: [u8; 1],
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
#[repr(C)]
#[derive(Debug)]
pub struct VTermState_saved {
    pub pos: VTermPos,
    pub pen: VTermPen,
    pub mode: VTermState_saved_mode,
}
#[derive(Copy, Clone, Debug)]
// align(4): `unsigned` bitfields, as on `VTermScreenCellAttrs`.
#[repr(C, align(4))]
pub struct VTermState_saved_mode {
    pub cursor_visible_cursor_blink_cursor_shape: [u8; 1],
    pub _pad: [u8; 3],
}
crate::bitfield_accessors! {
    impl VTermState_saved_mode.cursor_visible_cursor_blink_cursor_shape {
        0..=0 => cursor_visible, set_cursor_visible: ::core::ffi::c_uint;
        1..=1 => cursor_blink, set_cursor_blink: ::core::ffi::c_uint;
        2..=3 => cursor_shape, set_cursor_shape: ::core::ffi::c_uint;
    }
}
#[repr(C)]
#[derive(Debug)]
pub struct VTermState_selection {
    /// The table a C consumer installed, as C sees it.
    pub callbacks: *const VTermSelectionCallbacks,
    pub user: *mut ::core::ffi::c_void,
    /// Where libvterm kept the staging buffer; it is in the state's `owned`
    /// now, and the slot stays so that `buflen` is where C reads it.
    pub _buffer_slot: usize,
    pub buflen: size_t,
}
/// What a control string being read in pieces has got to: an OSC 52's
/// selection progress, or -- in the same first four bytes, as libvterm's
/// union had it -- a DECRQSS request's name.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct VTermState_tmp {
    pub selection: VTermState_tmp_selection,
}
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct VTermState_tmp_selection {
    pub mask: uint16_t,
    pub state: [u8; 1],
    pub _pad: [u8; 1],
    pub recvpartial: uint32_t,
    pub sendpartial: uint32_t,
}
crate::bitfield_accessors! {
    impl VTermState_tmp_selection.state {
        0..=7 => state, set_state: VTermState_tmp_selection_state;
    }
}
pub type VTermState_tmp_selection_state = ::core::ffi::c_uint;
#[derive(Copy, Clone, Debug)]
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
// The value's type is the attribute's (`attr_value_type`), not the union's.
crate::untagged_union_debug!(VTermValue);
pub type VTermValueType = ::core::ffi::c_uint;
#[derive(Copy, Clone, Debug)]
// align(4): `unsigned` bitfields, as on `VTermScreenCellAttrs`.
#[repr(C, align(4))]
pub struct VTerm_mode {
    pub utf8_ctrl8bit: [u8; 1],
    pub _pad: [u8; 3],
}
crate::bitfield_accessors! {
    impl VTerm_mode.utf8_ctrl8bit {
        0..=0 => utf8, set_utf8: ::core::ffi::c_uint;
        1..=1 => ctrl8bit, set_ctrl8bit: ::core::ffi::c_uint;
    }
}
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct VTerm_parser {
    pub state: VTermParserState,
    pub in_esc: [u8; 1],
    pub _pad: [u8; 3],
    pub intermedlen: ::core::ffi::c_int,
    pub intermed: [u8; 16],
    pub v: ParserScratch,
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
/// What the parser has gathered of the sequence in progress.
///
/// libvterm kept this as a union of three views; they overlap the same way
/// here, as one struct. A control sequence uses all of it: the private
/// marker bytes (`leader`, `leaderlen` of them), then the parameters. An OSC
/// keeps its command number in `leaderlen`; a DCS keeps its command bytes in
/// `leader` and their count in `leaderlen`.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct ParserScratch {
    pub leaderlen: ::core::ffi::c_int,
    pub leader: [u8; 16],
    pub argi: ::core::ffi::c_int,
    pub args: [::core::ffi::c_long; 32],
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
    assert!(size_of::<ParserScratch>() == 280 && align_of::<ParserScratch>() == 8);
    assert!(size_of::<VTerm_parser>() == 336 && align_of::<VTerm_parser>() == 8);
    assert!(size_of::<VTermParserCallbacks>() == 80 && align_of::<VTermParserCallbacks>() == 8);
    assert!(size_of::<VTermStateCallbacks>() == 104 && align_of::<VTermStateCallbacks>() == 8);
    assert!(size_of::<VTermStateFallbacks>() == 56 && align_of::<VTermStateFallbacks>() == 8);
    assert!(size_of::<VTermScreenCallbacks>() == 80 && align_of::<VTermScreenCallbacks>() == 8);
    assert!(
        size_of::<VTermSelectionCallbacks>() == 16 && align_of::<VTermSelectionCallbacks>() == 8
    );
    assert!(size_of::<VTermState>() == 544 && align_of::<VTermState>() == 8);
    assert!(size_of::<VTermScreen>() == 112 && align_of::<VTermScreen>() == 8);
    assert!(size_of::<VTerm>() == 360 && align_of::<VTerm>() == 8);

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
    assert!(offset_of!(VTermState, owned) == 80);
    assert!(offset_of!(VTermState, _lineinfo_slots) == 88);
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
    assert!(offset_of!(VTermState_selection, buflen) == 24);
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
    assert!(offset_of!(VTermScreen, owned) == 88);
    assert!(offset_of!(VTermScreen, pen) == 96);
    assert!(offset_of!(VTerm, rows) == 0);
    assert!(offset_of!(VTerm, cols) == 4);
    assert!(offset_of!(VTerm, mode) == 8);
    assert!(offset_of!(VTerm, parser) == 16);
    assert!(offset_of!(VTerm, inner) == 352);
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
