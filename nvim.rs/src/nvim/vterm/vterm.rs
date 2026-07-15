use ::c2rust_bitfields;
extern "C" {
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn vsnprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        __arg: ::core::ffi::VaList,
    ) -> ::core::ffi::c_int;
    fn abs(__x: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn vterm_state_free(state: *mut VTermState);
    fn vterm_screen_free(screen: *mut VTermScreen);
}
pub type __builtin_va_list = [__va_list_tag; 1];
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __va_list_tag {
    pub gp_offset: ::core::ffi::c_uint,
    pub fp_offset: ::core::ffi::c_uint,
    pub overflow_arg_area: *mut ::core::ffi::c_void,
    pub reg_save_area: *mut ::core::ffi::c_void,
}
pub type __gnuc_va_list = __builtin_va_list;
pub type va_list = __gnuc_va_list;
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
pub type VTermValueType = ::core::ffi::c_uint;
pub const VTERM_N_VALUETYPES: VTermValueType = 5;
pub const VTERM_VALUETYPE_COLOR: VTermValueType = 4;
pub const VTERM_VALUETYPE_STRING: VTermValueType = 3;
pub const VTERM_VALUETYPE_INT: VTermValueType = 2;
pub const VTERM_VALUETYPE_BOOL: VTermValueType = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermBuilder {
    pub ver: ::core::ffi::c_int,
    pub rows: ::core::ffi::c_int,
    pub cols: ::core::ffi::c_int,
    pub allocator: *const VTermAllocatorFunctions,
    pub allocdata: *mut ::core::ffi::c_void,
    pub outbuffer_len: size_t,
    pub tmpbuffer_len: size_t,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
    ::core::ffi::c_void,
>();
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
unsafe extern "C" fn default_malloc(
    mut size: size_t,
    mut allocdata: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_void {
    let mut ptr: *mut ::core::ffi::c_void = xmalloc(size);
    if !ptr.is_null() {
        memset(ptr, 0 as ::core::ffi::c_int, size);
    }
    return ptr;
}
unsafe extern "C" fn default_free(
    mut ptr: *mut ::core::ffi::c_void,
    mut allocdata: *mut ::core::ffi::c_void,
) {
    xfree(ptr);
}
static mut default_allocator: VTermAllocatorFunctions = VTermAllocatorFunctions {
    malloc: Some(
        default_malloc
            as unsafe extern "C" fn(
                size_t,
                *mut ::core::ffi::c_void,
            ) -> *mut ::core::ffi::c_void,
    ),
    free: Some(
        default_free
            as unsafe extern "C" fn(
                *mut ::core::ffi::c_void,
                *mut ::core::ffi::c_void,
            ) -> (),
    ),
};
#[no_mangle]
pub unsafe extern "C" fn vterm_new(
    mut rows: ::core::ffi::c_int,
    mut cols: ::core::ffi::c_int,
) -> *mut VTerm {
    let c2rust_lvalue: VTermBuilder = VTermBuilder {
        ver: 0,
        rows: rows,
        cols: cols,
        allocator: ::core::ptr::null::<VTermAllocatorFunctions>(),
        allocdata: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        outbuffer_len: 0,
        tmpbuffer_len: 0,
    };
    return vterm_build(&raw const c2rust_lvalue);
}
#[no_mangle]
pub unsafe extern "C" fn vterm_build(mut builder: *const VTermBuilder) -> *mut VTerm {
    let mut allocator: *const VTermAllocatorFunctions = if !(*builder)
        .allocator
        .is_null()
    {
        (*builder).allocator
    } else {
        &raw mut default_allocator as *const VTermAllocatorFunctions
    };
    let mut vt: *mut VTerm = Some(
            (*allocator).malloc.expect("non-null function pointer"),
        )
        .expect(
            "non-null function pointer",
        )(::core::mem::size_of::<VTerm>(), (*builder).allocdata) as *mut VTerm;
    (*vt).allocator = allocator;
    (*vt).allocdata = (*builder).allocdata;
    (*vt).rows = (*builder).rows;
    (*vt).cols = (*builder).cols;
    (*vt).parser.state = NORMAL;
    (*vt).parser.callbacks = ::core::ptr::null::<VTermParserCallbacks>();
    (*vt).parser.cbdata = NULL;
    (*vt).parser.emit_nul = false_0 != 0;
    (*vt).outfunc = None;
    (*vt).outdata = NULL;
    (*vt).outbuffer_len = if (*builder).outbuffer_len != 0 {
        (*builder).outbuffer_len
    } else {
        4096 as size_t
    };
    (*vt).outbuffer_cur = 0 as size_t;
    (*vt).outbuffer = vterm_allocator_malloc(vt, (*vt).outbuffer_len)
        as *mut ::core::ffi::c_char;
    (*vt).tmpbuffer_len = if (*builder).tmpbuffer_len != 0 {
        (*builder).tmpbuffer_len
    } else {
        4096 as size_t
    };
    (*vt).tmpbuffer = vterm_allocator_malloc(vt, (*vt).tmpbuffer_len)
        as *mut ::core::ffi::c_char;
    return vt;
}
#[no_mangle]
pub unsafe extern "C" fn vterm_free(mut vt: *mut VTerm) {
    if !(*vt).screen.is_null() {
        vterm_screen_free((*vt).screen);
    }
    if !(*vt).state.is_null() {
        vterm_state_free((*vt).state);
    }
    vterm_allocator_free(vt, (*vt).outbuffer as *mut ::core::ffi::c_void);
    vterm_allocator_free(vt, (*vt).tmpbuffer as *mut ::core::ffi::c_void);
    vterm_allocator_free(vt, vt as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn vterm_allocator_malloc(
    mut vt: *mut VTerm,
    mut size: size_t,
) -> *mut ::core::ffi::c_void {
    return Some((*(*vt).allocator).malloc.expect("non-null function pointer"))
        .expect("non-null function pointer")(size, (*vt).allocdata);
}
#[no_mangle]
pub unsafe extern "C" fn vterm_allocator_free(
    mut vt: *mut VTerm,
    mut ptr: *mut ::core::ffi::c_void,
) {
    Some((*(*vt).allocator).free.expect("non-null function pointer"))
        .expect("non-null function pointer")(ptr, (*vt).allocdata);
}
#[no_mangle]
pub unsafe extern "C" fn vterm_get_size(
    mut vt: *const VTerm,
    mut rowsp: *mut ::core::ffi::c_int,
    mut colsp: *mut ::core::ffi::c_int,
) {
    if !rowsp.is_null() {
        *rowsp = (*vt).rows;
    }
    if !colsp.is_null() {
        *colsp = (*vt).cols;
    }
}
#[no_mangle]
pub unsafe extern "C" fn vterm_set_size(
    mut vt: *mut VTerm,
    mut rows: ::core::ffi::c_int,
    mut cols: ::core::ffi::c_int,
) {
    if rows < 1 as ::core::ffi::c_int || cols < 1 as ::core::ffi::c_int {
        return;
    }
    (*vt).rows = rows;
    (*vt).cols = cols;
    if !(*vt).parser.callbacks.is_null() && (*(*vt).parser.callbacks).resize.is_some() {
        Some((*(*vt).parser.callbacks).resize.expect("non-null function pointer"))
            .expect("non-null function pointer")(rows, cols, (*vt).parser.cbdata);
    }
}
#[no_mangle]
pub unsafe extern "C" fn vterm_set_utf8(
    mut vt: *mut VTerm,
    mut is_utf8: ::core::ffi::c_int,
) {
    (*vt).mode.set_utf8(is_utf8 as ::core::ffi::c_uint as ::core::ffi::c_uint);
}
#[no_mangle]
pub unsafe extern "C" fn vterm_output_set_callback(
    mut vt: *mut VTerm,
    mut func: Option<VTermOutputCallback>,
    mut user: *mut ::core::ffi::c_void,
) {
    (*vt).outfunc = func;
    (*vt).outdata = user;
}
#[no_mangle]
pub unsafe extern "C" fn vterm_push_output_bytes(
    mut vt: *mut VTerm,
    mut bytes: *const ::core::ffi::c_char,
    mut len: size_t,
) {
    if (*vt).outfunc.is_some() {
        (*vt).outfunc.expect("non-null function pointer")(bytes, len, (*vt).outdata);
        return;
    }
    if len > (*vt).outbuffer_len.wrapping_sub((*vt).outbuffer_cur) {
        return;
    }
    memcpy(
        (*vt).outbuffer.offset((*vt).outbuffer_cur as isize) as *mut ::core::ffi::c_void,
        bytes as *const ::core::ffi::c_void,
        len,
    );
    (*vt).outbuffer_cur = (*vt).outbuffer_cur.wrapping_add(len);
}
#[no_mangle]
pub unsafe extern "C" fn vterm_push_output_sprintf(
    mut vt: *mut VTerm,
    mut format: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) {
    let mut args: ::core::ffi::VaListImpl;
    args = c2rust_args.clone();
    let mut len: size_t = vsnprintf(
        (*vt).tmpbuffer,
        (*vt).tmpbuffer_len,
        format,
        args.as_va_list(),
    ) as size_t;
    vterm_push_output_bytes(vt, (*vt).tmpbuffer, len);
}
#[no_mangle]
pub unsafe extern "C" fn vterm_push_output_sprintf_ctrl(
    mut vt: *mut VTerm,
    mut ctrl: uint8_t,
    mut fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) {
    let mut cur: size_t = 0;
    if ctrl as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int
        && (*vt).mode.ctrl8bit() == 0
    {
        cur = snprintf(
            (*vt).tmpbuffer,
            (*vt).tmpbuffer_len,
            b"\x1B%c\0".as_ptr() as *const ::core::ffi::c_char,
            ctrl as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int,
        ) as size_t;
    } else {
        cur = snprintf(
            (*vt).tmpbuffer,
            (*vt).tmpbuffer_len,
            b"%c\0".as_ptr() as *const ::core::ffi::c_char,
            ctrl as ::core::ffi::c_int,
        ) as size_t;
    }
    if cur >= (*vt).tmpbuffer_len {
        return;
    }
    let mut args: ::core::ffi::VaListImpl;
    args = c2rust_args.clone();
    cur = cur
        .wrapping_add(
            vsnprintf(
                (*vt).tmpbuffer.offset(cur as isize),
                (*vt).tmpbuffer_len.wrapping_sub(cur),
                fmt,
                args.as_va_list(),
            ) as size_t,
        );
    if cur >= (*vt).tmpbuffer_len {
        return;
    }
    vterm_push_output_bytes(vt, (*vt).tmpbuffer, cur);
}
#[no_mangle]
pub unsafe extern "C" fn vterm_push_output_sprintf_str(
    mut vt: *mut VTerm,
    mut ctrl: uint8_t,
    mut term: bool,
    mut fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) {
    let mut cur: size_t = 0 as size_t;
    if ctrl != 0 {
        if ctrl as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int
            && (*vt).mode.ctrl8bit() == 0
        {
            cur = snprintf(
                (*vt).tmpbuffer,
                (*vt).tmpbuffer_len,
                b"\x1B%c\0".as_ptr() as *const ::core::ffi::c_char,
                ctrl as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int,
            ) as size_t;
        } else {
            cur = snprintf(
                (*vt).tmpbuffer,
                (*vt).tmpbuffer_len,
                b"%c\0".as_ptr() as *const ::core::ffi::c_char,
                ctrl as ::core::ffi::c_int,
            ) as size_t;
        }
        if cur >= (*vt).tmpbuffer_len {
            return;
        }
    }
    let mut args: ::core::ffi::VaListImpl;
    args = c2rust_args.clone();
    cur = cur
        .wrapping_add(
            vsnprintf(
                (*vt).tmpbuffer.offset(cur as isize),
                (*vt).tmpbuffer_len.wrapping_sub(cur),
                fmt,
                args.as_va_list(),
            ) as size_t,
        );
    if cur >= (*vt).tmpbuffer_len {
        return;
    }
    if term {
        cur = cur
            .wrapping_add(
                snprintf(
                    (*vt).tmpbuffer.offset(cur as isize),
                    (*vt).tmpbuffer_len.wrapping_sub(cur),
                    if (*vt).mode.ctrl8bit() as ::core::ffi::c_int != 0 {
                        b"\x9C\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b"\x1B\\\0".as_ptr() as *const ::core::ffi::c_char
                    },
                ) as size_t,
            );
        if cur >= (*vt).tmpbuffer_len {
            return;
        }
    }
    vterm_push_output_bytes(vt, (*vt).tmpbuffer, cur);
}
#[no_mangle]
pub unsafe extern "C" fn vterm_get_attr_type(mut attr: VTermAttr) -> VTermValueType {
    match attr as ::core::ffi::c_uint {
        1 => return VTERM_VALUETYPE_BOOL,
        2 => return VTERM_VALUETYPE_INT,
        3 => return VTERM_VALUETYPE_BOOL,
        4 => return VTERM_VALUETYPE_BOOL,
        5 => return VTERM_VALUETYPE_BOOL,
        6 => return VTERM_VALUETYPE_BOOL,
        7 => return VTERM_VALUETYPE_BOOL,
        8 => return VTERM_VALUETYPE_INT,
        9 => return VTERM_VALUETYPE_COLOR,
        10 => return VTERM_VALUETYPE_COLOR,
        11 => return VTERM_VALUETYPE_BOOL,
        12 => return VTERM_VALUETYPE_INT,
        13 => return VTERM_VALUETYPE_INT,
        14 => return VTERM_VALUETYPE_BOOL,
        15 => return VTERM_VALUETYPE_BOOL,
        16 => return 0 as VTermValueType,
        _ => {}
    }
    return 0 as VTermValueType;
}
#[no_mangle]
pub unsafe extern "C" fn vterm_scroll_rect(
    mut rect: VTermRect,
    mut downward: ::core::ffi::c_int,
    mut rightward: ::core::ffi::c_int,
    mut moverect: Option<
        unsafe extern "C" fn(
            VTermRect,
            VTermRect,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    mut eraserect: Option<
        unsafe extern "C" fn(
            VTermRect,
            ::core::ffi::c_int,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    mut user: *mut ::core::ffi::c_void,
) {
    let mut src: VTermRect = VTermRect {
        start_row: 0,
        end_row: 0,
        start_col: 0,
        end_col: 0,
    };
    let mut dest: VTermRect = VTermRect {
        start_row: 0,
        end_row: 0,
        start_col: 0,
        end_col: 0,
    };
    if abs(downward) >= rect.end_row - rect.start_row
        || abs(rightward) >= rect.end_col - rect.start_col
    {
        Some(eraserect.expect("non-null function pointer"))
            .expect("non-null function pointer")(rect, 0 as ::core::ffi::c_int, user);
        return;
    }
    if rightward >= 0 as ::core::ffi::c_int {
        dest.start_col = rect.start_col;
        dest.end_col = rect.end_col - rightward;
        src.start_col = rect.start_col + rightward;
        src.end_col = rect.end_col;
    } else {
        let mut leftward: ::core::ffi::c_int = -rightward;
        dest.start_col = rect.start_col + leftward;
        dest.end_col = rect.end_col;
        src.start_col = rect.start_col;
        src.end_col = rect.end_col - leftward;
    }
    if downward >= 0 as ::core::ffi::c_int {
        dest.start_row = rect.start_row;
        dest.end_row = rect.end_row - downward;
        src.start_row = rect.start_row + downward;
        src.end_row = rect.end_row;
    } else {
        let mut upward: ::core::ffi::c_int = -downward;
        dest.start_row = rect.start_row + upward;
        dest.end_row = rect.end_row;
        src.start_row = rect.start_row;
        src.end_row = rect.end_row - upward;
    }
    if moverect.is_some() {
        Some(moverect.expect("non-null function pointer"))
            .expect("non-null function pointer")(dest, src, user);
    }
    if downward > 0 as ::core::ffi::c_int {
        rect.start_row = rect.end_row - downward;
    } else if downward < 0 as ::core::ffi::c_int {
        rect.end_row = rect.start_row - downward;
    }
    if rightward > 0 as ::core::ffi::c_int {
        rect.start_col = rect.end_col - rightward;
    } else if rightward < 0 as ::core::ffi::c_int {
        rect.end_col = rect.start_col - rightward;
    }
    Some(eraserect.expect("non-null function pointer"))
        .expect("non-null function pointer")(rect, 0 as ::core::ffi::c_int, user);
}
