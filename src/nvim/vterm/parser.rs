use ::c2rust_bitfields;
extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn strncpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> *mut ::core::ffi::c_char;
}
pub type size_t = usize;
pub type int32_t = i32;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
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
    pub bold_underline_italic_blink_reverse_conceal_strike_font_small_baseline_dim_overline_protected_cell_dwl_dhl:
        [u8; 3],
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
#[repr(C, align(4))] // align(4): C declares these as `unsigned` bitfields (4-byte-aligned storage unit); c2rust emitted an align-1 byte array, shifting fg/bg/uri offsets in VTermScreenCell vs the C ABI.
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
    pub bold_underline_italic_blink_reverse_conceal_strike_font_dwl_dhl_small_baseline_dim_overline:
        [u8; 3],
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
        unsafe extern "C" fn(VTermSelectionMask, *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
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
    pub bold_underline_italic_blink_reverse_conceal_strike_font_small_baseline_dim_overline:
        [u8; 3],
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
    pub init: Option<unsafe extern "C" fn(*mut VTermEncoding, *mut ::core::ffi::c_void) -> ()>,
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
    pub keypad_cursor_autowrap_insert_newline_cursor_visible_cursor_blink_cursor_shape_alt_screen_origin_screen_leftrightmargin_bracketpaste_report_focus_theme_updates_synchronized_output:
        [u8; 3],
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
pub type VTermOutputCallback =
    unsafe extern "C" fn(*const ::core::ffi::c_char, size_t, *mut ::core::ffi::c_void) -> ();
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
    pub malloc:
        Option<unsafe extern "C" fn(size_t, *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void>,
    pub free:
        Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut ::core::ffi::c_void) -> ()>,
}
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
