use ::c2rust_bitfields;
extern "C" {
    fn xctz(x: uint64_t) -> ::core::ffi::c_int;
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
pub type int32_t = i32;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type size_t = usize;
pub type schar_T = uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTerm {
    pub allocator: *const VTermAllocatorFunctions,
    pub allocdata: *mut ::core::ffi::c_void,
    pub rows: ::core::ffi::c_int,
    pub cols: ::core::ffi::c_int,
    pub mode: C2Rust_Unnamed_14,
    pub parser: C2Rust_Unnamed_9,
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
#[derive(Copy, Clone, ::c2rust_bitfields::BitfieldStruct)]
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
    #[bitfield(name = "global_reverse", ty = "::core::ffi::c_uint", bits = "0..=0")]
    #[bitfield(name = "reflow", ty = "::core::ffi::c_uint", bits = "1..=1")]
    pub global_reverse_reflow: [u8; 1],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 3],
    pub buffers: [*mut ScreenCell; 2],
    pub buffer: *mut ScreenCell,
    pub sb_buffer: *mut VTermScreenCell,
    pub pen: ScreenPen,
}
#[derive(Copy, Clone, ::c2rust_bitfields::BitfieldStruct)]
#[repr(C)]
pub struct ScreenPen {
    pub fg: VTermColor,
    pub bg: VTermColor,
    pub uri: ::core::ffi::c_int,
    #[bitfield(name = "bold", ty = "::core::ffi::c_uint", bits = "0..=0")]
    #[bitfield(name = "underline", ty = "::core::ffi::c_uint", bits = "1..=2")]
    #[bitfield(name = "italic", ty = "::core::ffi::c_uint", bits = "3..=3")]
    #[bitfield(name = "blink", ty = "::core::ffi::c_uint", bits = "4..=4")]
    #[bitfield(name = "reverse", ty = "::core::ffi::c_uint", bits = "5..=5")]
    #[bitfield(name = "conceal", ty = "::core::ffi::c_uint", bits = "6..=6")]
    #[bitfield(name = "strike", ty = "::core::ffi::c_uint", bits = "7..=7")]
    #[bitfield(name = "font", ty = "::core::ffi::c_uint", bits = "8..=11")]
    #[bitfield(name = "small", ty = "::core::ffi::c_uint", bits = "12..=12")]
    #[bitfield(name = "baseline", ty = "::core::ffi::c_uint", bits = "13..=14")]
    #[bitfield(name = "dim", ty = "::core::ffi::c_uint", bits = "15..=15")]
    #[bitfield(name = "overline", ty = "::core::ffi::c_uint", bits = "16..=16")]
    #[bitfield(name = "protected_cell", ty = "::core::ffi::c_uint", bits = "17..=17")]
    #[bitfield(name = "dwl", ty = "::core::ffi::c_uint", bits = "18..=18")]
    #[bitfield(name = "dhl", ty = "::core::ffi::c_uint", bits = "19..=20")]
    pub bold_underline_italic_blink_reverse_conceal_strike_font_small_baseline_dim_overline_protected_cell_dwl_dhl: [u8; 3],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union VTermColor {
    pub type_0: uint8_t,
    pub rgb: C2Rust_Unnamed_0,
    pub indexed: C2Rust_Unnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed {
    pub type_0: uint8_t,
    pub idx: uint8_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_0 {
    pub type_0: uint8_t,
    pub red: uint8_t,
    pub green: uint8_t,
    pub blue: uint8_t,
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
#[derive(Copy, Clone, ::c2rust_bitfields::BitfieldStruct)]
#[repr(C)]
pub struct VTermScreenCellAttrs {
    #[bitfield(name = "bold", ty = "::core::ffi::c_uint", bits = "0..=0")]
    #[bitfield(name = "underline", ty = "::core::ffi::c_uint", bits = "1..=2")]
    #[bitfield(name = "italic", ty = "::core::ffi::c_uint", bits = "3..=3")]
    #[bitfield(name = "blink", ty = "::core::ffi::c_uint", bits = "4..=4")]
    #[bitfield(name = "reverse", ty = "::core::ffi::c_uint", bits = "5..=5")]
    #[bitfield(name = "conceal", ty = "::core::ffi::c_uint", bits = "6..=6")]
    #[bitfield(name = "strike", ty = "::core::ffi::c_uint", bits = "7..=7")]
    #[bitfield(name = "font", ty = "::core::ffi::c_uint", bits = "8..=11")]
    #[bitfield(name = "dwl", ty = "::core::ffi::c_uint", bits = "12..=12")]
    #[bitfield(name = "dhl", ty = "::core::ffi::c_uint", bits = "13..=14")]
    #[bitfield(name = "small", ty = "::core::ffi::c_uint", bits = "15..=15")]
    #[bitfield(name = "baseline", ty = "::core::ffi::c_uint", bits = "16..=17")]
    #[bitfield(name = "dim", ty = "::core::ffi::c_uint", bits = "18..=18")]
    #[bitfield(name = "overline", ty = "::core::ffi::c_uint", bits = "19..=19")]
    pub bold_underline_italic_blink_reverse_conceal_strike_font_dwl_dhl_small_baseline_dim_overline: [u8; 3],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ScreenCell {
    pub schar: schar_T,
    pub pen: ScreenPen,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermRect {
    pub start_row: ::core::ffi::c_int,
    pub end_row: ::core::ffi::c_int,
    pub start_col: ::core::ffi::c_int,
    pub end_col: ::core::ffi::c_int,
}
pub type VTermDamageSize = ::core::ffi::c_uint;
pub const VTERM_N_DAMAGES: VTermDamageSize = 4;
pub const VTERM_DAMAGE_SCROLL: VTermDamageSize = 3;
pub const VTERM_DAMAGE_SCREEN: VTermDamageSize = 2;
pub const VTERM_DAMAGE_ROW: VTermDamageSize = 1;
pub const VTERM_DAMAGE_CELL: VTermDamageSize = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermScreenCallbacks {
    pub damage: Option<
        unsafe extern "C" fn(VTermRect, *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    >,
    pub moverect: Option<
        unsafe extern "C" fn(
            VTermRect,
            VTermRect,
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
    pub settermprop: Option<
        unsafe extern "C" fn(
            VTermProp,
            *mut VTermValue,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub bell: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    >,
    pub resize: Option<
        unsafe extern "C" fn(
            ::core::ffi::c_int,
            ::core::ffi::c_int,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub theme: Option<
        unsafe extern "C" fn(*mut bool, *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    >,
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
    pub sb_clear: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    >,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union VTermValue {
    pub boolean: ::core::ffi::c_int,
    pub number: ::core::ffi::c_int,
    pub string: VTermStringFragment,
    pub color: VTermColor,
}
#[derive(Copy, Clone, ::c2rust_bitfields::BitfieldStruct)]
#[repr(C)]
pub struct VTermStringFragment {
    pub str: *const ::core::ffi::c_char,
    #[bitfield(name = "len", ty = "size_t", bits = "0..=29")]
    #[bitfield(name = "initial", ty = "bool", bits = "30..=30")]
    #[bitfield(name = "final_0", ty = "bool", bits = "31..=31")]
    pub len_initial_final_0: [u8; 4],
    pub terminator: VTermTerminator,
}
pub type VTermTerminator = ::core::ffi::c_uint;
pub const VTERM_TERMINATOR_ST: VTermTerminator = 1;
pub const VTERM_TERMINATOR_BEL: VTermTerminator = 0;
pub type VTermProp = ::core::ffi::c_uint;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermPos {
    pub row: ::core::ffi::c_int,
    pub col: ::core::ffi::c_int,
}
#[derive(Copy, Clone, ::c2rust_bitfields::BitfieldStruct)]
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
    pub mouse_protocol: C2Rust_Unnamed_8,
    pub grapheme_buf: [::core::ffi::c_char; 32],
    pub grapheme_len: size_t,
    pub grapheme_last: uint32_t,
    pub grapheme_state: GraphemeState,
    pub combine_width: ::core::ffi::c_int,
    pub combine_pos: VTermPos,
    pub mode: C2Rust_Unnamed_7,
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
    #[bitfield(name = "protected_cell", ty = "::core::ffi::c_uint", bits = "0..=0")]
    pub protected_cell: [u8; 1],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 3],
    pub saved: C2Rust_Unnamed_5,
    pub tmp: C2Rust_Unnamed_2,
    pub selection: C2Rust_Unnamed_1,
    pub key_encoding_stacks: [VTermKeyEncodingStack; 2],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermKeyEncodingStack {
    pub items: [VTermKeyEncodingFlags; 16],
    pub size: uint8_t,
}
#[derive(Copy, Clone, ::c2rust_bitfields::BitfieldStruct)]
#[repr(C)]
pub struct VTermKeyEncodingFlags {
    #[bitfield(name = "disambiguate", ty = "bool", bits = "0..=0")]
    #[bitfield(name = "report_events", ty = "bool", bits = "1..=1")]
    #[bitfield(name = "report_alternate", ty = "bool", bits = "2..=2")]
    #[bitfield(name = "report_all_keys", ty = "bool", bits = "3..=3")]
    #[bitfield(name = "report_associated", ty = "bool", bits = "4..=4")]
    pub disambiguate_report_events_report_alternate_report_all_keys_report_associated: [u8; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_1 {
    pub callbacks: *const VTermSelectionCallbacks,
    pub user: *mut ::core::ffi::c_void,
    pub buffer: *mut ::core::ffi::c_char,
    pub buflen: size_t,
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
        unsafe extern "C" fn(
            VTermSelectionMask,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
}
pub type VTermSelectionMask = ::core::ffi::c_uint;
pub const VTERM_SELECTION_CUT0: VTermSelectionMask = 16;
pub const VTERM_SELECTION_SELECT: VTermSelectionMask = 8;
pub const VTERM_SELECTION_SECONDARY: VTermSelectionMask = 4;
pub const VTERM_SELECTION_PRIMARY: VTermSelectionMask = 2;
pub const VTERM_SELECTION_CLIPBOARD: VTermSelectionMask = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_2 {
    pub decrqss: [::core::ffi::c_char; 4],
    pub selection: C2Rust_Unnamed_3,
}
#[derive(Copy, Clone, ::c2rust_bitfields::BitfieldStruct)]
#[repr(C)]
pub struct C2Rust_Unnamed_3 {
    pub mask: uint16_t,
    #[bitfield(name = "state", ty = "C2Rust_Unnamed_4", bits = "0..=7")]
    pub state: [u8; 1],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 1],
    pub recvpartial: uint32_t,
    pub sendpartial: uint32_t,
}
pub type C2Rust_Unnamed_4 = ::core::ffi::c_uint;
pub const SELECTION_INVALID: C2Rust_Unnamed_4 = 5;
pub const SELECTION_SET: C2Rust_Unnamed_4 = 4;
pub const SELECTION_SET_INITIAL: C2Rust_Unnamed_4 = 3;
pub const SELECTION_QUERY: C2Rust_Unnamed_4 = 2;
pub const SELECTION_SELECTED: C2Rust_Unnamed_4 = 1;
pub const SELECTION_INITIAL: C2Rust_Unnamed_4 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_5 {
    pub pos: VTermPos,
    pub pen: VTermPen,
    pub mode: C2Rust_Unnamed_6,
}
#[derive(Copy, Clone, ::c2rust_bitfields::BitfieldStruct)]
#[repr(C)]
pub struct C2Rust_Unnamed_6 {
    #[bitfield(name = "cursor_visible", ty = "::core::ffi::c_uint", bits = "0..=0")]
    #[bitfield(name = "cursor_blink", ty = "::core::ffi::c_uint", bits = "1..=1")]
    #[bitfield(name = "cursor_shape", ty = "::core::ffi::c_uint", bits = "2..=3")]
    pub cursor_visible_cursor_blink_cursor_shape: [u8; 1],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 3],
}
#[derive(Copy, Clone, ::c2rust_bitfields::BitfieldStruct)]
#[repr(C)]
pub struct VTermPen {
    pub fg: VTermColor,
    pub bg: VTermColor,
    pub uri: ::core::ffi::c_int,
    #[bitfield(name = "bold", ty = "::core::ffi::c_uint", bits = "0..=0")]
    #[bitfield(name = "underline", ty = "::core::ffi::c_uint", bits = "1..=2")]
    #[bitfield(name = "italic", ty = "::core::ffi::c_uint", bits = "3..=3")]
    #[bitfield(name = "blink", ty = "::core::ffi::c_uint", bits = "4..=4")]
    #[bitfield(name = "reverse", ty = "::core::ffi::c_uint", bits = "5..=5")]
    #[bitfield(name = "conceal", ty = "::core::ffi::c_uint", bits = "6..=6")]
    #[bitfield(name = "strike", ty = "::core::ffi::c_uint", bits = "7..=7")]
    #[bitfield(name = "font", ty = "::core::ffi::c_uint", bits = "8..=11")]
    #[bitfield(name = "small", ty = "::core::ffi::c_uint", bits = "12..=12")]
    #[bitfield(name = "baseline", ty = "::core::ffi::c_uint", bits = "13..=14")]
    #[bitfield(name = "dim", ty = "::core::ffi::c_uint", bits = "15..=15")]
    #[bitfield(name = "overline", ty = "::core::ffi::c_uint", bits = "16..=16")]
    pub bold_underline_italic_blink_reverse_conceal_strike_font_small_baseline_dim_overline: [u8; 3],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermEncodingInstance {
    pub enc: *mut VTermEncoding,
    pub data: [::core::ffi::c_char; 16],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermEncoding {
    pub init: Option<
        unsafe extern "C" fn(*mut VTermEncoding, *mut ::core::ffi::c_void) -> (),
    >,
    pub decode: Option<
        unsafe extern "C" fn(
            *mut VTermEncoding,
            *mut ::core::ffi::c_void,
            *mut uint32_t,
            *mut ::core::ffi::c_int,
            ::core::ffi::c_int,
            *const ::core::ffi::c_char,
            *mut size_t,
            size_t,
        ) -> (),
    >,
}
#[derive(Copy, Clone, ::c2rust_bitfields::BitfieldStruct)]
#[repr(C)]
pub struct C2Rust_Unnamed_7 {
    #[bitfield(name = "keypad", ty = "::core::ffi::c_uint", bits = "0..=0")]
    #[bitfield(name = "cursor", ty = "::core::ffi::c_uint", bits = "1..=1")]
    #[bitfield(name = "autowrap", ty = "::core::ffi::c_uint", bits = "2..=2")]
    #[bitfield(name = "insert", ty = "::core::ffi::c_uint", bits = "3..=3")]
    #[bitfield(name = "newline", ty = "::core::ffi::c_uint", bits = "4..=4")]
    #[bitfield(name = "cursor_visible", ty = "::core::ffi::c_uint", bits = "5..=5")]
    #[bitfield(name = "cursor_blink", ty = "::core::ffi::c_uint", bits = "6..=6")]
    #[bitfield(name = "cursor_shape", ty = "::core::ffi::c_uint", bits = "7..=8")]
    #[bitfield(name = "alt_screen", ty = "::core::ffi::c_uint", bits = "9..=9")]
    #[bitfield(name = "origin", ty = "::core::ffi::c_uint", bits = "10..=10")]
    #[bitfield(name = "screen", ty = "::core::ffi::c_uint", bits = "11..=11")]
    #[bitfield(name = "leftrightmargin", ty = "::core::ffi::c_uint", bits = "12..=12")]
    #[bitfield(name = "bracketpaste", ty = "::core::ffi::c_uint", bits = "13..=13")]
    #[bitfield(name = "report_focus", ty = "::core::ffi::c_uint", bits = "14..=14")]
    #[bitfield(name = "theme_updates", ty = "::core::ffi::c_uint", bits = "15..=15")]
    #[bitfield(
        name = "synchronized_output",
        ty = "::core::ffi::c_uint",
        bits = "16..=16"
    )]
    pub keypad_cursor_autowrap_insert_newline_cursor_visible_cursor_blink_cursor_shape_alt_screen_origin_screen_leftrightmargin_bracketpaste_report_focus_theme_updates_synchronized_output: [u8; 3],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 1],
}
pub type GraphemeState = utf8proc_int32_t;
pub type utf8proc_int32_t = int32_t;
pub type C2Rust_Unnamed_8 = ::core::ffi::c_uint;
pub const MOUSE_RXVT: C2Rust_Unnamed_8 = 3;
pub const MOUSE_SGR: C2Rust_Unnamed_8 = 2;
pub const MOUSE_UTF8: C2Rust_Unnamed_8 = 1;
pub const MOUSE_X10: C2Rust_Unnamed_8 = 0;
#[derive(Copy, Clone, ::c2rust_bitfields::BitfieldStruct)]
#[repr(C)]
pub struct VTermLineInfo {
    #[bitfield(name = "doublewidth", ty = "::core::ffi::c_uint", bits = "0..=0")]
    #[bitfield(name = "doubleheight", ty = "::core::ffi::c_uint", bits = "1..=2")]
    #[bitfield(name = "continuation", ty = "::core::ffi::c_uint", bits = "3..=3")]
    pub doublewidth_doubleheight_continuation: [u8; 1],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 3],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermStateFallbacks {
    pub control: Option<
        unsafe extern "C" fn(uint8_t, *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
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
        unsafe extern "C" fn(
            VTermStringFragment,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub pm: Option<
        unsafe extern "C" fn(
            VTermStringFragment,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub sos: Option<
        unsafe extern "C" fn(
            VTermStringFragment,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
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
        unsafe extern "C" fn(
            VTermRect,
            VTermRect,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub erase: Option<
        unsafe extern "C" fn(
            VTermRect,
            ::core::ffi::c_int,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub initpen: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    >,
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
    pub bell: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    >,
    pub resize: Option<
        unsafe extern "C" fn(
            ::core::ffi::c_int,
            ::core::ffi::c_int,
            *mut VTermStateFields,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub theme: Option<
        unsafe extern "C" fn(*mut bool, *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    >,
    pub setlineinfo: Option<
        unsafe extern "C" fn(
            ::core::ffi::c_int,
            *const VTermLineInfo,
            *const VTermLineInfo,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub sb_clear: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    >,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermStateFields {
    pub pos: VTermPos,
    pub lineinfos: [*mut VTermLineInfo; 2],
}
pub type VTermAttr = ::core::ffi::c_uint;
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
#[derive(Copy, Clone, ::c2rust_bitfields::BitfieldStruct)]
#[repr(C)]
pub struct VTermGlyphInfo {
    pub schar: schar_T,
    pub width: ::core::ffi::c_int,
    #[bitfield(name = "protected_cell", ty = "::core::ffi::c_uint", bits = "0..=0")]
    #[bitfield(name = "dwl", ty = "::core::ffi::c_uint", bits = "1..=1")]
    #[bitfield(name = "dhl", ty = "::core::ffi::c_uint", bits = "2..=3")]
    pub protected_cell_dwl_dhl: [u8; 1],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 3],
}
pub type VTermOutputCallback = unsafe extern "C" fn(
    *const ::core::ffi::c_char,
    size_t,
    *mut ::core::ffi::c_void,
) -> ();
#[derive(Copy, Clone, ::c2rust_bitfields::BitfieldStruct)]
#[repr(C)]
pub struct C2Rust_Unnamed_9 {
    pub state: VTermParserState,
    #[bitfield(name = "in_esc", ty = "bool", bits = "0..=0")]
    pub in_esc: [u8; 1],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 3],
    pub intermedlen: ::core::ffi::c_int,
    pub intermed: [::core::ffi::c_char; 16],
    pub v: C2Rust_Unnamed_10,
    pub callbacks: *const VTermParserCallbacks,
    pub cbdata: *mut ::core::ffi::c_void,
    pub string_initial: bool,
    pub emit_nul: bool,
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
    pub control: Option<
        unsafe extern "C" fn(uint8_t, *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    >,
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
        unsafe extern "C" fn(
            VTermStringFragment,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub pm: Option<
        unsafe extern "C" fn(
            VTermStringFragment,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    pub sos: Option<
        unsafe extern "C" fn(
            VTermStringFragment,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
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
pub union C2Rust_Unnamed_10 {
    pub csi: C2Rust_Unnamed_13,
    pub osc: C2Rust_Unnamed_12,
    pub dcs: C2Rust_Unnamed_11,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_11 {
    pub commandlen: ::core::ffi::c_int,
    pub command: [::core::ffi::c_char; 16],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_12 {
    pub command: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_13 {
    pub leaderlen: ::core::ffi::c_int,
    pub leader: [::core::ffi::c_char; 16],
    pub argi: ::core::ffi::c_int,
    pub args: [::core::ffi::c_long; 32],
}
pub type VTermParserState = ::core::ffi::c_uint;
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
#[derive(Copy, Clone, ::c2rust_bitfields::BitfieldStruct)]
#[repr(C)]
pub struct C2Rust_Unnamed_14 {
    #[bitfield(name = "utf8", ty = "::core::ffi::c_uint", bits = "0..=0")]
    #[bitfield(name = "ctrl8bit", ty = "::core::ffi::c_uint", bits = "1..=1")]
    pub utf8_ctrl8bit: [u8; 1],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 3],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermAllocatorFunctions {
    pub malloc: Option<
        unsafe extern "C" fn(
            size_t,
            *mut ::core::ffi::c_void,
        ) -> *mut ::core::ffi::c_void,
    >,
    pub free: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut ::core::ffi::c_void) -> (),
    >,
}
pub type VTermModifier = ::core::ffi::c_uint;
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
            len = len
                .wrapping_add(
                    fill_utf8(
                        (code | modifiers) + 0x20 as ::core::ffi::c_int,
                        (&raw mut utf8 as *mut ::core::ffi::c_char).offset(len as isize),
                    ) as size_t,
                );
            len = len
                .wrapping_add(
                    fill_utf8(
                        col + 0x21 as ::core::ffi::c_int,
                        (&raw mut utf8 as *mut ::core::ffi::c_char).offset(len as isize),
                    ) as size_t,
                );
            len = len
                .wrapping_add(
                    fill_utf8(
                        row + 0x21 as ::core::ffi::c_int,
                        (&raw mut utf8 as *mut ::core::ffi::c_char).offset(len as isize),
                    ) as size_t,
                );
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
            let mut button: ::core::ffi::c_int = xctz((*state).mouse_buttons as uint64_t)
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
            } else if button >= 8 as ::core::ffi::c_int
                && button < 12 as ::core::ffi::c_int
            {
                output_mouse(
                    state,
                    button - 8 as ::core::ffi::c_int + 0x80 as ::core::ffi::c_int
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
            (*state).mouse_buttons
                |= (1 as ::core::ffi::c_int) << button - 1 as ::core::ffi::c_int;
        } else {
            (*state).mouse_buttons
                &= !((1 as ::core::ffi::c_int) << button - 1 as ::core::ffi::c_int);
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
