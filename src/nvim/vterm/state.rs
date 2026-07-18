use ::c2rust_bitfields;
extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn memmove(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn abs(__x: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn schar_from_buf(buf: *const ::core::ffi::c_char, len: size_t) -> schar_T;
    fn utf_ptr2cells_len(
        p: *const ::core::ffi::c_char,
        size: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn utf_iscomposing(
        c1: ::core::ffi::c_int,
        c2: ::core::ffi::c_int,
        state: *mut GraphemeState,
    ) -> bool;
    fn utf_char2bytes(c: ::core::ffi::c_int, buf: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn vterm_scroll_rect(
        rect: VTermRect,
        downward: ::core::ffi::c_int,
        rightward: ::core::ffi::c_int,
        moverect: Option<
            unsafe extern "C" fn(
                VTermRect,
                VTermRect,
                *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
        >,
        eraserect: Option<
            unsafe extern "C" fn(
                VTermRect,
                ::core::ffi::c_int,
                *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
        >,
        user: *mut ::core::ffi::c_void,
    );
    fn vterm_lookup_encoding(
        type_0: VTermEncodingType,
        designation: ::core::ffi::c_char,
    ) -> *mut VTermEncoding;
    fn vterm_parser_set_callbacks(
        vt: *mut VTerm,
        callbacks: *const VTermParserCallbacks,
        user: *mut ::core::ffi::c_void,
    );
    fn vterm_state_newpen(state: *mut VTermState);
    fn vterm_state_resetpen(state: *mut VTermState);
    fn vterm_state_savepen(state: *mut VTermState, save: ::core::ffi::c_int);
    fn vterm_state_setpen(
        state: *mut VTermState,
        args: *const ::core::ffi::c_long,
        argcount: ::core::ffi::c_int,
    );
    fn vterm_state_getpen(
        state: *mut VTermState,
        args: *mut ::core::ffi::c_long,
        argcount: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn vterm_allocator_malloc(vt: *mut VTerm, size: size_t) -> *mut ::core::ffi::c_void;
    fn vterm_allocator_free(vt: *mut VTerm, ptr: *mut ::core::ffi::c_void);
    fn vterm_push_output_bytes(vt: *mut VTerm, bytes: *const ::core::ffi::c_char, len: size_t);
    fn vterm_push_output_sprintf_ctrl(
        vt: *mut VTerm,
        ctrl: uint8_t,
        fmt: *const ::core::ffi::c_char,
        ...
    );
    fn vterm_push_output_sprintf_str(
        vt: *mut VTerm,
        ctrl: uint8_t,
        term: bool,
        fmt: *const ::core::ffi::c_char,
        ...
    );
}
pub type size_t = usize;
pub type int32_t = i32;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type schar_T = uint32_t;
pub type utf8proc_int32_t = int32_t;
pub type GraphemeState = utf8proc_int32_t;
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
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const VTERM_N_PROP_CURSORSHAPES: C2Rust_Unnamed_15 = 4;
pub const VTERM_PROP_CURSORSHAPE_BAR_LEFT: C2Rust_Unnamed_15 = 3;
pub const VTERM_PROP_CURSORSHAPE_UNDERLINE: C2Rust_Unnamed_15 = 2;
pub const VTERM_PROP_CURSORSHAPE_BLOCK: C2Rust_Unnamed_15 = 1;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const VTERM_N_PROP_MOUSES: C2Rust_Unnamed_16 = 4;
pub const VTERM_PROP_MOUSE_MOVE: C2Rust_Unnamed_16 = 3;
pub const VTERM_PROP_MOUSE_DRAG: C2Rust_Unnamed_16 = 2;
pub const VTERM_PROP_MOUSE_CLICK: C2Rust_Unnamed_16 = 1;
pub const VTERM_PROP_MOUSE_NONE: C2Rust_Unnamed_16 = 0;
pub type VTermEncodingType = ::core::ffi::c_uint;
pub const ENC_SINGLE_94: VTermEncodingType = 1;
pub const ENC_UTF8: VTermEncodingType = 0;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const C1_OSC: C2Rust_Unnamed_17 = 157;
pub const C1_ST: C2Rust_Unnamed_17 = 156;
pub const C1_CSI: C2Rust_Unnamed_17 = 155;
pub const C1_DCS: C2Rust_Unnamed_17 = 144;
pub const C1_SS3: C2Rust_Unnamed_17 = 143;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const GRAPHEME_STATE_INIT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const BUFIDX_PRIMARY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const BUFIDX_ALTSCREEN: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEY_ENCODING_DISAMBIGUATE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const KEY_ENCODING_REPORT_EVENTS: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const KEY_ENCODING_REPORT_ALTERNATE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const KEY_ENCODING_REPORT_ALL_KEYS: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const KEY_ENCODING_REPORT_ASSOCIATED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const MOUSE_WANT_CLICK: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const MOUSE_WANT_DRAG: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MOUSE_WANT_MOVE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const VTERM_VERSION_MAJOR: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const VTERM_VERSION_MINOR: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const CSI_ARG_FLAG_MORE: ::core::ffi::c_uint =
    (1 as ::core::ffi::c_uint) << 31 as ::core::ffi::c_int;
pub const CSI_ARG_MASK: ::core::ffi::c_uint =
    !((1 as ::core::ffi::c_uint) << 31 as ::core::ffi::c_int);
pub const CSI_ARG_MISSING: ::core::ffi::c_long = 2147483647 as ::core::ffi::c_long;
#[no_mangle]
pub static mut vterm_primary_device_attr: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"61;22;52\0") };
unsafe extern "C" fn putglyph(
    mut state: *mut VTermState,
    schar: schar_T,
    mut width: ::core::ffi::c_int,
    mut pos: VTermPos,
) {
    let mut info: VTermGlyphInfo = {
        let mut init = VTermGlyphInfo {
            protected_cell_dwl_dhl: [0; 1],
            c2rust_padding: [0; 3],
            schar: schar,
            width: width,
        };
        init.set_protected_cell((*state).protected_cell());
        init.set_dwl((*(*state).lineinfo.offset(pos.row as isize)).doublewidth());
        init.set_dhl((*(*state).lineinfo.offset(pos.row as isize)).doubleheight());
        init
    };
    if !(*state).callbacks.is_null() && (*(*state).callbacks).putglyph.is_some() {
        if Some(
            (*(*state).callbacks)
                .putglyph
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(&raw mut info, pos, (*state).cbdata)
            != 0
        {
            return;
        }
    }
}
unsafe extern "C" fn updatecursor(
    mut state: *mut VTermState,
    mut oldpos: *mut VTermPos,
    mut cancel_phantom: ::core::ffi::c_int,
) {
    if (*state).pos.col == (*oldpos).col && (*state).pos.row == (*oldpos).row {
        return;
    }
    if cancel_phantom != 0 {
        (*state).at_phantom = 0 as ::core::ffi::c_int;
    }
    if !(*state).callbacks.is_null() && (*(*state).callbacks).movecursor.is_some() {
        if Some(
            (*(*state).callbacks)
                .movecursor
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(
            (*state).pos,
            *oldpos,
            (*state).mode.cursor_visible() as ::core::ffi::c_int,
            (*state).cbdata,
        ) != 0
        {
            return;
        }
    }
}
unsafe extern "C" fn erase(
    mut state: *mut VTermState,
    mut rect: VTermRect,
    mut selective: ::core::ffi::c_int,
) {
    if rect.end_col == (*state).cols {
        let mut row: ::core::ffi::c_int = rect.start_row + 1 as ::core::ffi::c_int;
        while row < rect.end_row + 1 as ::core::ffi::c_int && row < (*state).rows {
            (*(*state).lineinfo.offset(row as isize))
                .set_continuation(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
            row += 1;
        }
    }
    if !(*state).callbacks.is_null() && (*(*state).callbacks).erase.is_some() {
        if Some(
            (*(*state).callbacks)
                .erase
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(rect, selective, (*state).cbdata)
            != 0
        {
            return;
        }
    }
}
unsafe extern "C" fn vterm_state_new(mut vt: *mut VTerm) -> *mut VTermState {
    let mut state: *mut VTermState =
        vterm_allocator_malloc(vt, ::core::mem::size_of::<VTermState>()) as *mut VTermState;
    (*state).vt = vt;
    (*state).rows = (*vt).rows;
    (*state).cols = (*vt).cols;
    (*state).mouse_col = 0 as ::core::ffi::c_int;
    (*state).mouse_row = 0 as ::core::ffi::c_int;
    (*state).mouse_buttons = 0 as ::core::ffi::c_int;
    (*state).mouse_protocol = MOUSE_X10;
    (*state).callbacks = ::core::ptr::null::<VTermStateCallbacks>();
    (*state).cbdata = NULL;
    (*state).selection.callbacks = ::core::ptr::null::<VTermSelectionCallbacks>();
    (*state).selection.user = NULL;
    (*state).selection.buffer = ::core::ptr::null_mut::<::core::ffi::c_char>();
    vterm_state_newpen(state);
    (*state).bold_is_highbright = 0 as ::core::ffi::c_int;
    (*state).combine_pos.row = -1 as ::core::ffi::c_int;
    (*state).tabstops = vterm_allocator_malloc(
        (*state).vt,
        ((*state).cols as size_t)
            .wrapping_add(7 as size_t)
            .wrapping_div(8 as size_t),
    ) as *mut uint8_t;
    (*state).lineinfos[BUFIDX_PRIMARY as usize] = vterm_allocator_malloc(
        (*state).vt,
        ((*state).rows as size_t).wrapping_mul(::core::mem::size_of::<VTermLineInfo>()),
    ) as *mut VTermLineInfo;
    (*state).lineinfos[BUFIDX_ALTSCREEN as usize] = vterm_allocator_malloc(
        (*state).vt,
        ((*state).rows as size_t).wrapping_mul(::core::mem::size_of::<VTermLineInfo>()),
    ) as *mut VTermLineInfo;
    (*state).lineinfo = (*state).lineinfos[BUFIDX_PRIMARY as usize];
    (*state).encoding_utf8.enc = vterm_lookup_encoding(ENC_UTF8, 'u' as ::core::ffi::c_char);
    if Some(
        (*(*state).encoding_utf8.enc)
            .init
            .expect("non-null function pointer"),
    )
    .is_some()
    {
        Some(
            (*(*state).encoding_utf8.enc)
                .init
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(
            (*state).encoding_utf8.enc,
            &raw mut (*state).encoding_utf8.data as *mut ::core::ffi::c_char
                as *mut ::core::ffi::c_void,
        );
    }
    let mut i: size_t = 0 as size_t;
    while i < ::core::mem::size_of::<[VTermKeyEncodingStack; 2]>()
        .wrapping_div(::core::mem::size_of::<VTermKeyEncodingStack>())
        .wrapping_div(
            (::core::mem::size_of::<[VTermKeyEncodingStack; 2]>()
                .wrapping_rem(::core::mem::size_of::<VTermKeyEncodingStack>())
                == 0) as ::core::ffi::c_int as usize,
        )
    {
        let mut stack: *mut VTermKeyEncodingStack = (&raw mut (*state).key_encoding_stacks
            as *mut VTermKeyEncodingStack)
            .offset(i as isize);
        let mut j: size_t = 0 as size_t;
        while j < ::core::mem::size_of::<[VTermKeyEncodingFlags; 16]>()
            .wrapping_div(::core::mem::size_of::<VTermKeyEncodingFlags>())
            .wrapping_div(
                (::core::mem::size_of::<[VTermKeyEncodingFlags; 16]>()
                    .wrapping_rem(::core::mem::size_of::<VTermKeyEncodingFlags>())
                    == 0) as ::core::ffi::c_int as usize,
            )
        {
            memset(
                (&raw mut (*stack).items as *mut VTermKeyEncodingFlags).offset(j as isize)
                    as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<VTermKeyEncodingFlags>(),
            );
            j = j.wrapping_add(1);
        }
        (*stack).size = 1 as uint8_t;
        i = i.wrapping_add(1);
    }
    return state;
}
#[no_mangle]
pub unsafe extern "C" fn vterm_state_free(mut state: *mut VTermState) {
    vterm_allocator_free((*state).vt, (*state).tabstops as *mut ::core::ffi::c_void);
    vterm_allocator_free(
        (*state).vt,
        (*state).lineinfos[BUFIDX_PRIMARY as usize] as *mut ::core::ffi::c_void,
    );
    if !(*state).lineinfos[BUFIDX_ALTSCREEN as usize].is_null() {
        vterm_allocator_free(
            (*state).vt,
            (*state).lineinfos[BUFIDX_ALTSCREEN as usize] as *mut ::core::ffi::c_void,
        );
    }
    vterm_allocator_free((*state).vt, state as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn scroll(
    mut state: *mut VTermState,
    mut rect: VTermRect,
    mut downward: ::core::ffi::c_int,
    mut rightward: ::core::ffi::c_int,
) {
    if downward == 0 && rightward == 0 {
        return;
    }
    let mut rows: ::core::ffi::c_int = rect.end_row - rect.start_row;
    if downward > rows {
        downward = rows;
    } else if downward < -rows {
        downward = -rows;
    }
    let mut cols: ::core::ffi::c_int = rect.end_col - rect.start_col;
    if rightward > cols {
        rightward = cols;
    } else if rightward < -cols {
        rightward = -cols;
    }
    if rect.start_col == 0 as ::core::ffi::c_int
        && rect.end_col == (*state).cols
        && rightward == 0 as ::core::ffi::c_int
    {
        let mut height: ::core::ffi::c_int = rect.end_row - rect.start_row - abs(downward);
        if downward > 0 as ::core::ffi::c_int {
            memmove(
                (*state).lineinfo.offset(rect.start_row as isize) as *mut ::core::ffi::c_void,
                (*state)
                    .lineinfo
                    .offset(rect.start_row as isize)
                    .offset(downward as isize) as *const ::core::ffi::c_void,
                (height as size_t).wrapping_mul(::core::mem::size_of::<VTermLineInfo>()),
            );
            let mut row: ::core::ffi::c_int = rect.end_row - downward;
            while row < rect.end_row {
                *(*state).lineinfo.offset(row as isize) = {
                    let mut init = VTermLineInfo {
                        doublewidth_doubleheight_continuation: [0; 1],
                        c2rust_padding: [0; 3],
                    };
                    init.set_doublewidth(0 as ::core::ffi::c_uint);
                    init.set_doubleheight(0);
                    init.set_continuation(0);
                    init
                };
                row += 1;
            }
        } else {
            memmove(
                (*state)
                    .lineinfo
                    .offset(rect.start_row as isize)
                    .offset(-(downward as isize)) as *mut ::core::ffi::c_void,
                (*state).lineinfo.offset(rect.start_row as isize) as *const ::core::ffi::c_void,
                (height as size_t).wrapping_mul(::core::mem::size_of::<VTermLineInfo>()),
            );
            let mut row_0: ::core::ffi::c_int = rect.start_row;
            while row_0 < rect.start_row - downward {
                *(*state).lineinfo.offset(row_0 as isize) = {
                    let mut init = VTermLineInfo {
                        doublewidth_doubleheight_continuation: [0; 1],
                        c2rust_padding: [0; 3],
                    };
                    init.set_doublewidth(0 as ::core::ffi::c_uint);
                    init.set_doubleheight(0);
                    init.set_continuation(0);
                    init
                };
                row_0 += 1;
            }
        }
    }
    if !(*state).callbacks.is_null() && (*(*state).callbacks).scrollrect.is_some() {
        if Some(
            (*(*state).callbacks)
                .scrollrect
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(rect, downward, rightward, (*state).cbdata)
            != 0
        {
            return;
        }
    }
    if !(*state).callbacks.is_null() {
        vterm_scroll_rect(
            rect,
            downward,
            rightward,
            (*(*state).callbacks).moverect,
            (*(*state).callbacks).erase,
            (*state).cbdata,
        );
    }
}
unsafe extern "C" fn linefeed(mut state: *mut VTermState) {
    if (*state).pos.row
        == (if (*state).scrollregion_bottom > -1 as ::core::ffi::c_int {
            (*state).scrollregion_bottom
        } else {
            (*state).rows
        }) - 1 as ::core::ffi::c_int
    {
        let mut rect: VTermRect = VTermRect {
            start_row: (*state).scrollregion_top,
            end_row: if (*state).scrollregion_bottom > -1 as ::core::ffi::c_int {
                (*state).scrollregion_bottom
            } else {
                (*state).rows
            },
            start_col: if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0 {
                (*state).scrollregion_left
            } else {
                0 as ::core::ffi::c_int
            },
            end_col: if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0
                && (*state).scrollregion_right > -1 as ::core::ffi::c_int
            {
                (*state).scrollregion_right
            } else {
                (*state).cols
            },
        };
        scroll(
            state,
            rect,
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
    } else if (*state).pos.row < (*state).rows - 1 as ::core::ffi::c_int {
        (*state).pos.row += 1;
    }
}
unsafe extern "C" fn set_col_tabstop(mut state: *mut VTermState, mut col: ::core::ffi::c_int) {
    let mut mask: uint8_t =
        ((1 as ::core::ffi::c_int) << (col & 7 as ::core::ffi::c_int)) as uint8_t;
    *(*state)
        .tabstops
        .offset((col >> 3 as ::core::ffi::c_int) as isize) =
        (*(*state)
            .tabstops
            .offset((col >> 3 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            | mask as ::core::ffi::c_int) as uint8_t;
}
unsafe extern "C" fn clear_col_tabstop(mut state: *mut VTermState, mut col: ::core::ffi::c_int) {
    let mut mask: uint8_t =
        ((1 as ::core::ffi::c_int) << (col & 7 as ::core::ffi::c_int)) as uint8_t;
    *(*state)
        .tabstops
        .offset((col >> 3 as ::core::ffi::c_int) as isize) =
        (*(*state)
            .tabstops
            .offset((col >> 3 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            & !(mask as ::core::ffi::c_int)) as uint8_t;
}
unsafe extern "C" fn is_col_tabstop(
    mut state: *mut VTermState,
    mut col: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut mask: uint8_t =
        ((1 as ::core::ffi::c_int) << (col & 7 as ::core::ffi::c_int)) as uint8_t;
    return *(*state)
        .tabstops
        .offset((col >> 3 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
        & mask as ::core::ffi::c_int;
}
unsafe extern "C" fn is_cursor_in_scrollregion(mut state: *const VTermState) -> ::core::ffi::c_int {
    if (*state).pos.row < (*state).scrollregion_top
        || (*state).pos.row
            >= (if (*state).scrollregion_bottom > -1 as ::core::ffi::c_int {
                (*state).scrollregion_bottom
            } else {
                (*state).rows
            })
    {
        return 0 as ::core::ffi::c_int;
    }
    if (*state).pos.col
        < (if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0 {
            (*state).scrollregion_left
        } else {
            0 as ::core::ffi::c_int
        })
        || (*state).pos.col
            >= (if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0
                && (*state).scrollregion_right > -1 as ::core::ffi::c_int
            {
                (*state).scrollregion_right
            } else {
                (*state).cols
            })
    {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn tab(
    mut state: *mut VTermState,
    mut count: ::core::ffi::c_int,
    mut direction: ::core::ffi::c_int,
) {
    while count > 0 as ::core::ffi::c_int {
        if direction > 0 as ::core::ffi::c_int {
            if (*state).pos.col
                >= (if (*(*state).lineinfo.offset((*state).pos.row as isize)).doublewidth()
                    as ::core::ffi::c_int
                    != 0
                {
                    (*state).cols / 2 as ::core::ffi::c_int
                } else {
                    (*state).cols
                }) - 1 as ::core::ffi::c_int
            {
                return;
            }
            (*state).pos.col += 1;
        } else if direction < 0 as ::core::ffi::c_int {
            if (*state).pos.col < 1 as ::core::ffi::c_int {
                return;
            }
            (*state).pos.col -= 1;
        }
        if is_col_tabstop(state, (*state).pos.col) != 0 {
            count -= 1;
        }
    }
}
pub const NO_FORCE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FORCE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const DWL_OFF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const DWL_ON: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const DHL_OFF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const DHL_TOP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const DHL_BOTTOM: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
unsafe extern "C" fn set_lineinfo(
    mut state: *mut VTermState,
    mut row: ::core::ffi::c_int,
    mut force: ::core::ffi::c_int,
    mut dwl: ::core::ffi::c_int,
    mut dhl: ::core::ffi::c_int,
) {
    let mut info: VTermLineInfo = *(*state).lineinfo.offset(row as isize);
    if dwl == DWL_OFF {
        info.set_doublewidth(DWL_OFF as ::core::ffi::c_uint as ::core::ffi::c_uint);
    } else if dwl == DWL_ON {
        info.set_doublewidth(DWL_ON as ::core::ffi::c_uint as ::core::ffi::c_uint);
    }
    if dhl == DHL_OFF {
        info.set_doubleheight(DHL_OFF as ::core::ffi::c_uint as ::core::ffi::c_uint);
    } else if dhl == DHL_TOP {
        info.set_doubleheight(DHL_TOP as ::core::ffi::c_uint as ::core::ffi::c_uint);
    } else if dhl == DHL_BOTTOM {
        info.set_doubleheight(DHL_BOTTOM as ::core::ffi::c_uint as ::core::ffi::c_uint);
    }
    if !(*state).callbacks.is_null()
        && (*(*state).callbacks).setlineinfo.is_some()
        && Some(
            (*(*state).callbacks)
                .setlineinfo
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(
            row,
            &raw mut info,
            (*state).lineinfo.offset(row as isize),
            (*state).cbdata,
        ) != 0
        || force != 0
    {
        *(*state).lineinfo.offset(row as isize) = info;
    }
}
unsafe extern "C" fn on_text(
    mut bytes: *const ::core::ffi::c_char,
    mut len: size_t,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut state: *mut VTermState = user as *mut VTermState;
    let mut oldpos: VTermPos = (*state).pos;
    let mut codepoints: *mut uint32_t = (*(*state).vt).tmpbuffer as *mut uint32_t;
    let mut maxpoints: size_t = (*(*state).vt)
        .tmpbuffer_len
        .wrapping_div(::core::mem::size_of::<uint32_t>());
    let mut npoints: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut eaten: size_t = 0 as size_t;
    let mut encoding: *mut VTermEncodingInstance = if (*state).gsingle_set != 0 {
        (&raw mut (*state).encoding as *mut VTermEncodingInstance)
            .offset((*state).gsingle_set as isize)
    } else if *bytes.offset(eaten as isize) as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0
    {
        (&raw mut (*state).encoding as *mut VTermEncodingInstance).offset((*state).gl_set as isize)
    } else if (*(*state).vt).mode.utf8() as ::core::ffi::c_int != 0 {
        &raw mut (*state).encoding_utf8
    } else {
        (&raw mut (*state).encoding as *mut VTermEncodingInstance).offset((*state).gr_set as isize)
    };
    if (*encoding).enc == (*state).encoding_utf8.enc {
        encoding = &raw mut (*state).encoding_utf8;
    }
    Some(
        (*(*encoding).enc)
            .decode
            .expect("non-null function pointer"),
    )
    .expect("non-null function pointer")(
        (*encoding).enc,
        &raw mut (*encoding).data as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        codepoints as *mut uint32_t,
        &raw mut npoints,
        if (*state).gsingle_set != 0 {
            1 as ::core::ffi::c_int
        } else {
            maxpoints as ::core::ffi::c_int
        },
        bytes,
        &raw mut eaten,
        len,
    );
    if npoints == 0 {
        return eaten as ::core::ffi::c_int;
    }
    if (*state).gsingle_set != 0 && npoints != 0 {
        (*state).gsingle_set = 0 as ::core::ffi::c_int;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut grapheme_state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
    let mut grapheme_len: size_t = 0 as size_t;
    let mut recombine: bool = false_0 != 0;
    if (*state).pos.row == (*state).combine_pos.row
        && (*state).pos.col >= (*state).combine_pos.col
        && (*state).pos.col <= (*state).combine_pos.col + (*state).combine_width
    {
        if utf_iscomposing(
            (*state).grapheme_last as ::core::ffi::c_int,
            *codepoints.offset(i as isize) as ::core::ffi::c_int,
            &raw mut (*state).grapheme_state,
        ) {
            grapheme_len = (*state).grapheme_len;
            grapheme_state = (*state).grapheme_state;
            (*state).pos.col = (*state).combine_pos.col;
            (*state).at_phantom = 0 as ::core::ffi::c_int;
            recombine = true_0 != 0;
        }
    }
    while i < npoints {
        loop {
            if grapheme_len
                < ::core::mem::size_of::<[::core::ffi::c_char; 32]>().wrapping_sub(4 as usize)
            {
                grapheme_len = grapheme_len.wrapping_add(utf_char2bytes(
                    *codepoints.offset(i as isize) as ::core::ffi::c_int,
                    (&raw mut (*state).grapheme_buf as *mut ::core::ffi::c_char)
                        .offset(grapheme_len as isize),
                ) as size_t);
            }
            i += 1;
            if !(i < npoints
                && utf_iscomposing(
                    *codepoints.offset((i - 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int,
                    *codepoints.offset(i as isize) as ::core::ffi::c_int,
                    &raw mut grapheme_state,
                ) as ::core::ffi::c_int
                    != 0)
            {
                break;
            }
        }
        let mut width: ::core::ffi::c_int = utf_ptr2cells_len(
            &raw mut (*state).grapheme_buf as *mut ::core::ffi::c_char,
            grapheme_len as ::core::ffi::c_int,
        );
        if (*state).at_phantom != 0
            || (*state).pos.col + width
                > (if (*(*state).lineinfo.offset((*state).pos.row as isize)).doublewidth()
                    as ::core::ffi::c_int
                    != 0
                {
                    (*state).cols / 2 as ::core::ffi::c_int
                } else {
                    (*state).cols
                })
        {
            linefeed(state);
            (*state).pos.col = 0 as ::core::ffi::c_int;
            (*state).at_phantom = 0 as ::core::ffi::c_int;
            (*(*state).lineinfo.offset((*state).pos.row as isize))
                .set_continuation(1 as ::core::ffi::c_uint as ::core::ffi::c_uint);
        }
        if (*state).mode.insert() as ::core::ffi::c_int != 0 && !recombine {
            let mut rect: VTermRect = VTermRect {
                start_row: (*state).pos.row,
                end_row: (*state).pos.row + 1 as ::core::ffi::c_int,
                start_col: (*state).pos.col,
                end_col: if (*(*state).lineinfo.offset((*state).pos.row as isize)).doublewidth()
                    as ::core::ffi::c_int
                    != 0
                {
                    (*state).cols / 2 as ::core::ffi::c_int
                } else {
                    (*state).cols
                },
            };
            scroll(
                state,
                rect,
                0 as ::core::ffi::c_int,
                -1 as ::core::ffi::c_int,
            );
        }
        let mut sc: schar_T = schar_from_buf(
            &raw mut (*state).grapheme_buf as *mut ::core::ffi::c_char,
            grapheme_len,
        );
        putglyph(state, sc, width, (*state).pos);
        if i == npoints {
            (*state).grapheme_len = grapheme_len;
            (*state).grapheme_last = *codepoints.offset((i - 1 as ::core::ffi::c_int) as isize);
            (*state).grapheme_state = grapheme_state;
            (*state).combine_width = width;
            (*state).combine_pos = (*state).pos;
        } else {
            grapheme_len = 0 as size_t;
            recombine = false_0 != 0;
        }
        if (*state).pos.col + width
            >= (if (*(*state).lineinfo.offset((*state).pos.row as isize)).doublewidth()
                as ::core::ffi::c_int
                != 0
            {
                (*state).cols / 2 as ::core::ffi::c_int
            } else {
                (*state).cols
            })
        {
            if (*state).mode.autowrap() != 0 {
                (*state).at_phantom = 1 as ::core::ffi::c_int;
            }
        } else {
            (*state).pos.col += width;
        }
    }
    updatecursor(state, &raw mut oldpos, 0 as ::core::ffi::c_int);
    return eaten as ::core::ffi::c_int;
}
unsafe extern "C" fn on_control(
    mut control: uint8_t,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut state: *mut VTermState = user as *mut VTermState;
    let mut oldpos: VTermPos = (*state).pos;
    match control as ::core::ffi::c_int {
        7 => {
            if !(*state).callbacks.is_null() && (*(*state).callbacks).bell.is_some() {
                Some(
                    (*(*state).callbacks)
                        .bell
                        .expect("non-null function pointer"),
                )
                .expect("non-null function pointer")((*state).cbdata);
            }
        }
        8 => {
            if (*state).pos.col > 0 as ::core::ffi::c_int {
                (*state).pos.col -= 1;
            }
        }
        9 => {
            tab(state, 1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
        }
        10 | 11 | 12 => {
            linefeed(state);
            if (*state).mode.newline() != 0 {
                (*state).pos.col = 0 as ::core::ffi::c_int;
            }
        }
        13 => {
            (*state).pos.col = 0 as ::core::ffi::c_int;
        }
        14 => {
            (*state).gl_set = 1 as ::core::ffi::c_int;
        }
        15 => {
            (*state).gl_set = 0 as ::core::ffi::c_int;
        }
        132 => {
            linefeed(state);
        }
        133 => {
            linefeed(state);
            (*state).pos.col = 0 as ::core::ffi::c_int;
        }
        136 => {
            set_col_tabstop(state, (*state).pos.col);
        }
        141 => {
            if (*state).pos.row == (*state).scrollregion_top {
                let mut rect: VTermRect = VTermRect {
                    start_row: (*state).scrollregion_top,
                    end_row: if (*state).scrollregion_bottom > -1 as ::core::ffi::c_int {
                        (*state).scrollregion_bottom
                    } else {
                        (*state).rows
                    },
                    start_col: if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0 {
                        (*state).scrollregion_left
                    } else {
                        0 as ::core::ffi::c_int
                    },
                    end_col: if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0
                        && (*state).scrollregion_right > -1 as ::core::ffi::c_int
                    {
                        (*state).scrollregion_right
                    } else {
                        (*state).cols
                    },
                };
                scroll(
                    state,
                    rect,
                    -1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
            } else if (*state).pos.row > 0 as ::core::ffi::c_int {
                (*state).pos.row -= 1;
            }
        }
        142 => {
            (*state).gsingle_set = 2 as ::core::ffi::c_int;
        }
        143 => {
            (*state).gsingle_set = 3 as ::core::ffi::c_int;
        }
        _ => {
            if !(*state).fallbacks.is_null() && (*(*state).fallbacks).control.is_some() {
                if Some(
                    (*(*state).fallbacks)
                        .control
                        .expect("non-null function pointer"),
                )
                .expect("non-null function pointer")(control, (*state).fbdata)
                    != 0
                {
                    return 1 as ::core::ffi::c_int;
                }
            }
            return 0 as ::core::ffi::c_int;
        }
    }
    updatecursor(state, &raw mut oldpos, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn settermprop_bool(
    mut state: *mut VTermState,
    mut prop: VTermProp,
    mut v: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut val: VTermValue = VTermValue { boolean: v };
    return vterm_state_set_termprop(state, prop, &raw mut val);
}
unsafe extern "C" fn settermprop_int(
    mut state: *mut VTermState,
    mut prop: VTermProp,
    mut v: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut val: VTermValue = VTermValue { number: v };
    return vterm_state_set_termprop(state, prop, &raw mut val);
}
unsafe extern "C" fn settermprop_string(
    mut state: *mut VTermState,
    mut prop: VTermProp,
    mut frag: VTermStringFragment,
) -> ::core::ffi::c_int {
    let mut val: VTermValue = VTermValue { string: frag };
    return vterm_state_set_termprop(state, prop, &raw mut val);
}
unsafe extern "C" fn savecursor(mut state: *mut VTermState, mut save: ::core::ffi::c_int) {
    if save != 0 {
        (*state).saved.pos = (*state).pos;
        (*state)
            .saved
            .mode
            .set_cursor_visible((*state).mode.cursor_visible() as ::core::ffi::c_uint);
        (*state)
            .saved
            .mode
            .set_cursor_blink((*state).mode.cursor_blink() as ::core::ffi::c_uint);
        (*state)
            .saved
            .mode
            .set_cursor_shape((*state).mode.cursor_shape() as ::core::ffi::c_uint);
        vterm_state_savepen(state, 1 as ::core::ffi::c_int);
    } else {
        let mut oldpos: VTermPos = (*state).pos;
        (*state).pos = (*state).saved.pos;
        settermprop_bool(
            state,
            VTERM_PROP_CURSORVISIBLE,
            (*state).saved.mode.cursor_visible() as ::core::ffi::c_int,
        );
        settermprop_bool(
            state,
            VTERM_PROP_CURSORBLINK,
            (*state).saved.mode.cursor_blink() as ::core::ffi::c_int,
        );
        settermprop_int(
            state,
            VTERM_PROP_CURSORSHAPE,
            (*state).saved.mode.cursor_shape() as ::core::ffi::c_int,
        );
        vterm_state_savepen(state, 0 as ::core::ffi::c_int);
        updatecursor(state, &raw mut oldpos, 1 as ::core::ffi::c_int);
    };
}
unsafe extern "C" fn on_escape(
    mut bytes: *const ::core::ffi::c_char,
    mut len: size_t,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut state: *mut VTermState = user as *mut VTermState;
    match *bytes.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
        32 => {
            if len != 2 as size_t {
                return 0 as ::core::ffi::c_int;
            }
            match *bytes.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
                70 => {
                    (*(*state).vt)
                        .mode
                        .set_ctrl8bit(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
                }
                71 => {
                    (*(*state).vt)
                        .mode
                        .set_ctrl8bit(1 as ::core::ffi::c_uint as ::core::ffi::c_uint);
                }
                _ => return 0 as ::core::ffi::c_int,
            }
            return 2 as ::core::ffi::c_int;
        }
        35 => {
            if len != 2 as size_t {
                return 0 as ::core::ffi::c_int;
            }
            match *bytes.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
                51 => {
                    if (*state).mode.leftrightmargin() == 0 {
                        set_lineinfo(state, (*state).pos.row, NO_FORCE, DWL_ON, DHL_TOP);
                    }
                }
                52 => {
                    if (*state).mode.leftrightmargin() == 0 {
                        set_lineinfo(state, (*state).pos.row, NO_FORCE, DWL_ON, DHL_BOTTOM);
                    }
                }
                53 => {
                    if (*state).mode.leftrightmargin() == 0 {
                        set_lineinfo(state, (*state).pos.row, NO_FORCE, DWL_OFF, DHL_OFF);
                    }
                }
                54 => {
                    if (*state).mode.leftrightmargin() == 0 {
                        set_lineinfo(state, (*state).pos.row, NO_FORCE, DWL_ON, DHL_OFF);
                    }
                }
                56 => {
                    let mut pos: VTermPos = VTermPos { row: 0, col: 0 };
                    let mut E: schar_T = 'E' as ::core::ffi::c_int as schar_T;
                    pos.row = 0 as ::core::ffi::c_int;
                    while pos.row < (*state).rows {
                        pos.col = 0 as ::core::ffi::c_int;
                        while pos.col
                            < (if (*(*state).lineinfo.offset(pos.row as isize)).doublewidth()
                                as ::core::ffi::c_int
                                != 0
                            {
                                (*state).cols / 2 as ::core::ffi::c_int
                            } else {
                                (*state).cols
                            })
                        {
                            putglyph(state, E, 1 as ::core::ffi::c_int, pos);
                            pos.col += 1;
                        }
                        pos.row += 1;
                    }
                }
                _ => return 0 as ::core::ffi::c_int,
            }
            return 2 as ::core::ffi::c_int;
        }
        40 | 41 | 42 | 43 => {
            if len != 2 as size_t {
                return 0 as ::core::ffi::c_int;
            }
            let mut setnum: ::core::ffi::c_int = *bytes.offset(0 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                - 0x28 as ::core::ffi::c_int;
            let mut newenc: *mut VTermEncoding = vterm_lookup_encoding(
                ENC_SINGLE_94,
                *bytes.offset(1 as ::core::ffi::c_int as isize),
            );
            if !newenc.is_null() {
                (*state).encoding[setnum as usize].enc = newenc;
                if (*newenc).init.is_some() {
                    Some((*newenc).init.expect("non-null function pointer"))
                        .expect("non-null function pointer")(
                        newenc,
                        &raw mut (*(&raw mut (*state).encoding as *mut VTermEncodingInstance)
                            .offset(setnum as isize))
                        .data as *mut ::core::ffi::c_char
                            as *mut ::core::ffi::c_void,
                    );
                }
            }
            return 2 as ::core::ffi::c_int;
        }
        55 => {
            savecursor(state, 1 as ::core::ffi::c_int);
            return 1 as ::core::ffi::c_int;
        }
        56 => {
            savecursor(state, 0 as ::core::ffi::c_int);
            return 1 as ::core::ffi::c_int;
        }
        60 => return 1 as ::core::ffi::c_int,
        61 => {
            (*state)
                .mode
                .set_keypad(1 as ::core::ffi::c_uint as ::core::ffi::c_uint);
            return 1 as ::core::ffi::c_int;
        }
        62 => {
            (*state)
                .mode
                .set_keypad(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
            return 1 as ::core::ffi::c_int;
        }
        99 => {
            let mut oldpos: VTermPos = (*state).pos;
            vterm_state_reset(state, 1 as ::core::ffi::c_int);
            if !(*state).callbacks.is_null() && (*(*state).callbacks).movecursor.is_some() {
                Some(
                    (*(*state).callbacks)
                        .movecursor
                        .expect("non-null function pointer"),
                )
                .expect("non-null function pointer")(
                    (*state).pos,
                    oldpos,
                    (*state).mode.cursor_visible() as ::core::ffi::c_int,
                    (*state).cbdata,
                );
            }
            return 1 as ::core::ffi::c_int;
        }
        110 => {
            (*state).gl_set = 2 as ::core::ffi::c_int;
            return 1 as ::core::ffi::c_int;
        }
        111 => {
            (*state).gl_set = 3 as ::core::ffi::c_int;
            return 1 as ::core::ffi::c_int;
        }
        126 => {
            (*state).gr_set = 1 as ::core::ffi::c_int;
            return 1 as ::core::ffi::c_int;
        }
        125 => {
            (*state).gr_set = 2 as ::core::ffi::c_int;
            return 1 as ::core::ffi::c_int;
        }
        124 => {
            (*state).gr_set = 3 as ::core::ffi::c_int;
            return 1 as ::core::ffi::c_int;
        }
        _ => return 0 as ::core::ffi::c_int,
    };
}
unsafe extern "C" fn set_mode(
    mut state: *mut VTermState,
    mut num: ::core::ffi::c_int,
    mut val: ::core::ffi::c_int,
) {
    match num {
        4 => {
            (*state)
                .mode
                .set_insert(val as ::core::ffi::c_uint as ::core::ffi::c_uint);
        }
        20 => {
            (*state)
                .mode
                .set_newline(val as ::core::ffi::c_uint as ::core::ffi::c_uint);
        }
        _ => return,
    };
}
unsafe extern "C" fn set_dec_mode(
    mut state: *mut VTermState,
    mut num: ::core::ffi::c_int,
    mut val: ::core::ffi::c_int,
) {
    match num {
        1 => {
            (*state)
                .mode
                .set_cursor(val as ::core::ffi::c_uint as ::core::ffi::c_uint);
        }
        5 => {
            settermprop_bool(state, VTERM_PROP_REVERSE, val);
        }
        6 => {
            let mut oldpos: VTermPos = (*state).pos;
            (*state)
                .mode
                .set_origin(val as ::core::ffi::c_uint as ::core::ffi::c_uint);
            (*state).pos.row = if (*state).mode.origin() as ::core::ffi::c_int != 0 {
                (*state).scrollregion_top
            } else {
                0 as ::core::ffi::c_int
            };
            (*state).pos.col = if (*state).mode.origin() as ::core::ffi::c_int != 0 {
                if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0 {
                    (*state).scrollregion_left
                } else {
                    0 as ::core::ffi::c_int
                }
            } else {
                0 as ::core::ffi::c_int
            };
            updatecursor(state, &raw mut oldpos, 1 as ::core::ffi::c_int);
        }
        7 => {
            (*state)
                .mode
                .set_autowrap(val as ::core::ffi::c_uint as ::core::ffi::c_uint);
        }
        12 => {
            settermprop_bool(state, VTERM_PROP_CURSORBLINK, val);
        }
        25 => {
            settermprop_bool(state, VTERM_PROP_CURSORVISIBLE, val);
        }
        69 => {
            (*state)
                .mode
                .set_leftrightmargin(val as ::core::ffi::c_uint as ::core::ffi::c_uint);
            if val != 0 {
                let mut row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while row < (*state).rows {
                    set_lineinfo(state, row, FORCE, DWL_OFF, DHL_OFF);
                    row += 1;
                }
            }
        }
        1000 | 1002 | 1003 => {
            settermprop_int(
                state,
                VTERM_PROP_MOUSE,
                if val == 0 {
                    VTERM_PROP_MOUSE_NONE as ::core::ffi::c_int
                } else if num == 1000 as ::core::ffi::c_int {
                    VTERM_PROP_MOUSE_CLICK as ::core::ffi::c_int
                } else if num == 1002 as ::core::ffi::c_int {
                    VTERM_PROP_MOUSE_DRAG as ::core::ffi::c_int
                } else {
                    VTERM_PROP_MOUSE_MOVE as ::core::ffi::c_int
                },
            );
        }
        1004 => {
            settermprop_bool(state, VTERM_PROP_FOCUSREPORT, val);
            (*state)
                .mode
                .set_report_focus(val as ::core::ffi::c_uint as ::core::ffi::c_uint);
        }
        1005 => {
            (*state).mouse_protocol = (if val != 0 {
                MOUSE_UTF8 as ::core::ffi::c_int
            } else {
                MOUSE_X10 as ::core::ffi::c_int
            }) as C2Rust_Unnamed_8;
        }
        1006 => {
            (*state).mouse_protocol = (if val != 0 {
                MOUSE_SGR as ::core::ffi::c_int
            } else {
                MOUSE_X10 as ::core::ffi::c_int
            }) as C2Rust_Unnamed_8;
        }
        1015 => {
            (*state).mouse_protocol = (if val != 0 {
                MOUSE_RXVT as ::core::ffi::c_int
            } else {
                MOUSE_X10 as ::core::ffi::c_int
            }) as C2Rust_Unnamed_8;
        }
        1047 => {
            settermprop_bool(state, VTERM_PROP_ALTSCREEN, val);
        }
        1048 => {
            savecursor(state, val);
        }
        1049 => {
            settermprop_bool(state, VTERM_PROP_ALTSCREEN, val);
            savecursor(state, val);
        }
        2004 => {
            (*state)
                .mode
                .set_bracketpaste(val as ::core::ffi::c_uint as ::core::ffi::c_uint);
        }
        2026 => {
            settermprop_bool(state, VTERM_PROP_SYNCOUTPUT, val);
        }
        2031 => {
            settermprop_bool(state, VTERM_PROP_THEMEUPDATES, val);
        }
        _ => return,
    };
}
unsafe extern "C" fn request_dec_mode(mut state: *mut VTermState, mut num: ::core::ffi::c_int) {
    let mut reply: ::core::ffi::c_int = 0;
    match num {
        1 => {
            reply = (*state).mode.cursor() as ::core::ffi::c_int;
        }
        5 => {
            reply = (*state).mode.screen() as ::core::ffi::c_int;
        }
        6 => {
            reply = (*state).mode.origin() as ::core::ffi::c_int;
        }
        7 => {
            reply = (*state).mode.autowrap() as ::core::ffi::c_int;
        }
        12 => {
            reply = (*state).mode.cursor_blink() as ::core::ffi::c_int;
        }
        25 => {
            reply = (*state).mode.cursor_visible() as ::core::ffi::c_int;
        }
        69 => {
            reply = (*state).mode.leftrightmargin() as ::core::ffi::c_int;
        }
        1000 => {
            reply = ((*state).mouse_flags == MOUSE_WANT_CLICK) as ::core::ffi::c_int;
        }
        1002 => {
            reply =
                ((*state).mouse_flags == MOUSE_WANT_CLICK | MOUSE_WANT_DRAG) as ::core::ffi::c_int;
        }
        1003 => {
            reply =
                ((*state).mouse_flags == MOUSE_WANT_CLICK | MOUSE_WANT_MOVE) as ::core::ffi::c_int;
        }
        1004 => {
            reply = (*state).mode.report_focus() as ::core::ffi::c_int;
        }
        1005 => {
            reply = ((*state).mouse_protocol as ::core::ffi::c_uint
                == MOUSE_UTF8 as ::core::ffi::c_int as ::core::ffi::c_uint)
                as ::core::ffi::c_int;
        }
        1006 => {
            reply = ((*state).mouse_protocol as ::core::ffi::c_uint
                == MOUSE_SGR as ::core::ffi::c_int as ::core::ffi::c_uint)
                as ::core::ffi::c_int;
        }
        1015 => {
            reply = ((*state).mouse_protocol as ::core::ffi::c_uint
                == MOUSE_RXVT as ::core::ffi::c_int as ::core::ffi::c_uint)
                as ::core::ffi::c_int;
        }
        1047 => {
            reply = (*state).mode.alt_screen() as ::core::ffi::c_int;
        }
        2004 => {
            reply = (*state).mode.bracketpaste() as ::core::ffi::c_int;
        }
        2026 => {
            reply = (*state).mode.synchronized_output() as ::core::ffi::c_int;
        }
        2031 => {
            reply = (*state).mode.theme_updates() as ::core::ffi::c_int;
        }
        _ => {
            vterm_push_output_sprintf_ctrl(
                (*state).vt,
                C1_CSI as ::core::ffi::c_int as uint8_t,
                b"?%d;%d$y\0".as_ptr() as *const ::core::ffi::c_char,
                num,
                0 as ::core::ffi::c_int,
            );
            return;
        }
    }
    vterm_push_output_sprintf_ctrl(
        (*state).vt,
        C1_CSI as ::core::ffi::c_int as uint8_t,
        b"?%d;%d$y\0".as_ptr() as *const ::core::ffi::c_char,
        num,
        if reply != 0 {
            1 as ::core::ffi::c_int
        } else {
            2 as ::core::ffi::c_int
        },
    );
}
unsafe extern "C" fn request_version_string(mut state: *mut VTermState) {
    vterm_push_output_sprintf_str(
        (*state).vt,
        C1_DCS as ::core::ffi::c_int as uint8_t,
        true_0 != 0,
        b">|libvterm(%d.%d)\0".as_ptr() as *const ::core::ffi::c_char,
        VTERM_VERSION_MAJOR,
        VTERM_VERSION_MINOR,
    );
}
unsafe extern "C" fn request_key_encoding_flags(mut state: *mut VTermState) {
    let mut screen: ::core::ffi::c_int = if (*state).mode.alt_screen() as ::core::ffi::c_int != 0 {
        BUFIDX_ALTSCREEN
    } else {
        BUFIDX_PRIMARY
    };
    let mut stack: *mut VTermKeyEncodingStack = (&raw mut (*state).key_encoding_stacks
        as *mut VTermKeyEncodingStack)
        .offset(screen as isize);
    let mut reply: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    '_c2rust_label: {
        if (*stack).size as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"stack->size > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/vterm/state.rs\0".as_ptr() as *const ::core::ffi::c_char,
                952 as ::core::ffi::c_uint,
                b"void request_key_encoding_flags(VTermState *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut flags: VTermKeyEncodingFlags =
        (*stack).items[((*stack).size as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as usize];
    if flags.disambiguate() {
        reply |= KEY_ENCODING_DISAMBIGUATE;
    }
    if flags.report_events() {
        reply |= KEY_ENCODING_REPORT_EVENTS;
    }
    if flags.report_alternate() {
        reply |= KEY_ENCODING_REPORT_ALTERNATE;
    }
    if flags.report_all_keys() {
        reply |= KEY_ENCODING_REPORT_ALL_KEYS;
    }
    if flags.report_associated() {
        reply |= KEY_ENCODING_REPORT_ASSOCIATED;
    }
    vterm_push_output_sprintf_ctrl(
        (*state).vt,
        C1_CSI as ::core::ffi::c_int as uint8_t,
        b"?%du\0".as_ptr() as *const ::core::ffi::c_char,
        reply,
    );
}
unsafe extern "C" fn set_key_encoding_flags(
    mut state: *mut VTermState,
    mut arg: ::core::ffi::c_int,
    mut mode: ::core::ffi::c_int,
) {
    let mut set: bool = mode != 3 as ::core::ffi::c_int;
    let mut reset_unset: bool = mode == 1 as ::core::ffi::c_int;
    let mut flags: VTermKeyEncodingFlags = {
        let mut init = VTermKeyEncodingFlags {
            disambiguate_report_events_report_alternate_report_all_keys_report_associated: [0; 1],
        };
        init.set_disambiguate(false);
        init.set_report_events(false);
        init.set_report_alternate(false);
        init.set_report_all_keys(false);
        init.set_report_associated(false);
        init
    };
    if arg & KEY_ENCODING_DISAMBIGUATE != 0 {
        flags.set_disambiguate(set as bool);
    } else if reset_unset {
        flags.set_disambiguate((false_0 != 0) as bool);
    }
    if arg & KEY_ENCODING_REPORT_EVENTS != 0 {
        flags.set_report_events(set as bool);
    } else if reset_unset {
        flags.set_report_events((false_0 != 0) as bool);
    }
    if arg & KEY_ENCODING_REPORT_ALTERNATE != 0 {
        flags.set_report_alternate(set as bool);
    } else if reset_unset {
        flags.set_report_alternate((false_0 != 0) as bool);
    }
    if arg & KEY_ENCODING_REPORT_ALL_KEYS != 0 {
        flags.set_report_all_keys(set as bool);
    } else if reset_unset {
        flags.set_report_all_keys((false_0 != 0) as bool);
    }
    if arg & KEY_ENCODING_REPORT_ASSOCIATED != 0 {
        flags.set_report_associated(set as bool);
    } else if reset_unset {
        flags.set_report_associated((false_0 != 0) as bool);
    }
    let mut screen: ::core::ffi::c_int = if (*state).mode.alt_screen() as ::core::ffi::c_int != 0 {
        BUFIDX_ALTSCREEN
    } else {
        BUFIDX_PRIMARY
    };
    let mut stack: *mut VTermKeyEncodingStack = (&raw mut (*state).key_encoding_stacks
        as *mut VTermKeyEncodingStack)
        .offset(screen as isize);
    '_c2rust_label: {
        if (*stack).size as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"stack->size > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/vterm/state.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1018 as ::core::ffi::c_uint,
                b"void set_key_encoding_flags(VTermState *, int, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    (*stack).items[((*stack).size as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as usize] =
        flags as VTermKeyEncodingFlags;
}
unsafe extern "C" fn push_key_encoding_flags(
    mut state: *mut VTermState,
    mut arg: ::core::ffi::c_int,
) {
    let mut screen: ::core::ffi::c_int = if (*state).mode.alt_screen() as ::core::ffi::c_int != 0 {
        BUFIDX_ALTSCREEN
    } else {
        BUFIDX_PRIMARY
    };
    let mut stack: *mut VTermKeyEncodingStack = (&raw mut (*state).key_encoding_stacks
        as *mut VTermKeyEncodingStack)
        .offset(screen as isize);
    '_c2rust_label: {
        if (*stack).size as usize
            <= ::core::mem::size_of::<[VTermKeyEncodingFlags; 16]>()
                .wrapping_div(::core::mem::size_of::<VTermKeyEncodingFlags>())
                .wrapping_div(
                    (::core::mem::size_of::<[VTermKeyEncodingFlags; 16]>()
                        .wrapping_rem(::core::mem::size_of::<VTermKeyEncodingFlags>())
                        == 0) as ::core::ffi::c_int as usize,
                )
        {
        } else {
            __assert_fail(
                b"stack->size <= ARRAY_SIZE(stack->items)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/vterm/state.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1026 as ::core::ffi::c_uint,
                b"void push_key_encoding_flags(VTermState *, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if (*stack).size as usize
        == ::core::mem::size_of::<[VTermKeyEncodingFlags; 16]>()
            .wrapping_div(::core::mem::size_of::<VTermKeyEncodingFlags>())
            .wrapping_div(
                (::core::mem::size_of::<[VTermKeyEncodingFlags; 16]>()
                    .wrapping_rem(::core::mem::size_of::<VTermKeyEncodingFlags>())
                    == 0) as ::core::ffi::c_int as usize,
            )
    {
        let mut i: size_t = 0 as size_t;
        while i < ::core::mem::size_of::<[VTermKeyEncodingFlags; 16]>()
            .wrapping_div(::core::mem::size_of::<VTermKeyEncodingFlags>())
            .wrapping_div(
                (::core::mem::size_of::<[VTermKeyEncodingFlags; 16]>()
                    .wrapping_rem(::core::mem::size_of::<VTermKeyEncodingFlags>())
                    == 0) as ::core::ffi::c_int as usize,
            )
            .wrapping_sub(1 as usize)
        {
            (*stack).items[i as usize] = (*stack).items[i.wrapping_add(1 as size_t) as usize];
            i = i.wrapping_add(1);
        }
    } else {
        (*stack).size = (*stack).size.wrapping_add(1);
    }
    set_key_encoding_flags(state, arg, 1 as ::core::ffi::c_int);
}
unsafe extern "C" fn pop_key_encoding_flags(
    mut state: *mut VTermState,
    mut arg: ::core::ffi::c_int,
) {
    let mut screen: ::core::ffi::c_int = if (*state).mode.alt_screen() as ::core::ffi::c_int != 0 {
        BUFIDX_ALTSCREEN
    } else {
        BUFIDX_PRIMARY
    };
    let mut stack: *mut VTermKeyEncodingStack = (&raw mut (*state).key_encoding_stacks
        as *mut VTermKeyEncodingStack)
        .offset(screen as isize);
    if arg >= (*stack).size as ::core::ffi::c_int {
        (*stack).size = 1 as uint8_t;
        memset(
            (&raw mut (*stack).items as *mut VTermKeyEncodingFlags)
                .offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<VTermKeyEncodingFlags>(),
        );
    } else if arg > 0 as ::core::ffi::c_int {
        (*stack).size = ((*stack).size as ::core::ffi::c_int - arg) as uint8_t;
    }
}
unsafe extern "C" fn on_csi(
    mut leader: *const ::core::ffi::c_char,
    mut args: *const ::core::ffi::c_long,
    mut argcount: ::core::ffi::c_int,
    mut intermed: *const ::core::ffi::c_char,
    mut command: ::core::ffi::c_char,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut state: *mut VTermState = user as *mut VTermState;
    let mut leader_byte: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut intermed_byte: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cancel_phantom: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    if !leader.is_null()
        && *leader.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 0
    {
        if *leader.offset(1 as ::core::ffi::c_int as isize) != 0 {
            return 0 as ::core::ffi::c_int;
        }
        match *leader.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            63 | 62 | 60 | 61 => {
                leader_byte =
                    *leader.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
            }
            _ => return 0 as ::core::ffi::c_int,
        }
    }
    if !intermed.is_null()
        && *intermed.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 0
    {
        if *intermed.offset(1 as ::core::ffi::c_int as isize) != 0 {
            return 0 as ::core::ffi::c_int;
        }
        match *intermed.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            32 | 33 | 34 | 36 | 39 => {
                intermed_byte =
                    *intermed.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
            }
            _ => return 0 as ::core::ffi::c_int,
        }
    }
    let mut oldpos: VTermPos = (*state).pos;
    let mut count: ::core::ffi::c_int = 0;
    let mut val: ::core::ffi::c_int = 0;
    let mut row: ::core::ffi::c_int = 0;
    let mut col: ::core::ffi::c_int = 0;
    let mut rect: VTermRect = VTermRect {
        start_row: 0,
        end_row: 0,
        start_col: 0,
        end_col: 0,
    };
    let mut selective: ::core::ffi::c_int = 0;
    match intermed_byte << 16 as ::core::ffi::c_int
        | leader_byte << 8 as ::core::ffi::c_int
        | command as ::core::ffi::c_int
    {
        64 => {
            count = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long)
                as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
                || *args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long
                    == 0 as ::core::ffi::c_long
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            if is_cursor_in_scrollregion(state) != 0 {
                rect.start_row = (*state).pos.row;
                rect.end_row = (*state).pos.row + 1 as ::core::ffi::c_int;
                rect.start_col = (*state).pos.col;
                if (*state).mode.leftrightmargin() != 0 {
                    rect.end_col = if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0
                        && (*state).scrollregion_right > -1 as ::core::ffi::c_int
                    {
                        (*state).scrollregion_right
                    } else {
                        (*state).cols
                    };
                } else {
                    rect.end_col = if (*(*state).lineinfo.offset((*state).pos.row as isize))
                        .doublewidth() as ::core::ffi::c_int
                        != 0
                    {
                        (*state).cols / 2 as ::core::ffi::c_int
                    } else {
                        (*state).cols
                    };
                }
                scroll(state, rect, 0 as ::core::ffi::c_int, -count);
            }
        }
        65 => {
            count = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long)
                as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
                || *args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long
                    == 0 as ::core::ffi::c_long
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            (*state).pos.row -= count;
            (*state).at_phantom = 0 as ::core::ffi::c_int;
        }
        66 => {
            count = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long)
                as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
                || *args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long
                    == 0 as ::core::ffi::c_long
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            (*state).pos.row += count;
            (*state).at_phantom = 0 as ::core::ffi::c_int;
        }
        67 => {
            count = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long)
                as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
                || *args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long
                    == 0 as ::core::ffi::c_long
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            (*state).pos.col += count;
            (*state).at_phantom = 0 as ::core::ffi::c_int;
        }
        68 => {
            count = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long)
                as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
                || *args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long
                    == 0 as ::core::ffi::c_long
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            (*state).pos.col -= count;
            (*state).at_phantom = 0 as ::core::ffi::c_int;
        }
        69 => {
            count = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long)
                as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
                || *args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long
                    == 0 as ::core::ffi::c_long
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            (*state).pos.col = 0 as ::core::ffi::c_int;
            (*state).pos.row += count;
            (*state).at_phantom = 0 as ::core::ffi::c_int;
        }
        70 => {
            count = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long)
                as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
                || *args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long
                    == 0 as ::core::ffi::c_long
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            (*state).pos.col = 0 as ::core::ffi::c_int;
            (*state).pos.row -= count;
            (*state).at_phantom = 0 as ::core::ffi::c_int;
        }
        71 => {
            val = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long) as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            (*state).pos.col = val - 1 as ::core::ffi::c_int;
            (*state).at_phantom = 0 as ::core::ffi::c_int;
        }
        72 => {
            row = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long) as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            col = (if argcount < 2 as ::core::ffi::c_int
                || (*args.offset(1 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long)
                    as ::core::ffi::c_ulong
                    == CSI_ARG_MISSING as ::core::ffi::c_ulong
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(1 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            (*state).pos.row = row - 1 as ::core::ffi::c_int;
            (*state).pos.col = col - 1 as ::core::ffi::c_int;
            if (*state).mode.origin() != 0 {
                (*state).pos.row += (*state).scrollregion_top;
                (*state).pos.col += if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0 {
                    (*state).scrollregion_left
                } else {
                    0 as ::core::ffi::c_int
                };
            }
            (*state).at_phantom = 0 as ::core::ffi::c_int;
        }
        73 => {
            count = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long)
                as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
                || *args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long
                    == 0 as ::core::ffi::c_long
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            tab(state, count, 1 as ::core::ffi::c_int);
        }
        74 | 16202 => {
            selective = (leader_byte == '?' as ::core::ffi::c_int) as ::core::ffi::c_int;
            match *args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long
            {
                2147483647 | 0 => {
                    rect.start_row = (*state).pos.row;
                    rect.end_row = (*state).pos.row + 1 as ::core::ffi::c_int;
                    rect.start_col = (*state).pos.col;
                    rect.end_col = (*state).cols;
                    if rect.end_col > rect.start_col {
                        erase(state, rect, selective);
                    }
                    rect.start_row = (*state).pos.row + 1 as ::core::ffi::c_int;
                    rect.end_row = (*state).rows;
                    rect.start_col = 0 as ::core::ffi::c_int;
                    let mut row_: ::core::ffi::c_int = rect.start_row;
                    while row_ < rect.end_row {
                        set_lineinfo(state, row_, FORCE, DWL_OFF, DHL_OFF);
                        row_ += 1;
                    }
                    if rect.end_row > rect.start_row {
                        erase(state, rect, selective);
                    }
                }
                1 => {
                    rect.start_row = 0 as ::core::ffi::c_int;
                    rect.end_row = (*state).pos.row;
                    rect.start_col = 0 as ::core::ffi::c_int;
                    rect.end_col = (*state).cols;
                    let mut row__0: ::core::ffi::c_int = rect.start_row;
                    while row__0 < rect.end_row {
                        set_lineinfo(state, row__0, FORCE, DWL_OFF, DHL_OFF);
                        row__0 += 1;
                    }
                    if rect.end_col > rect.start_col {
                        erase(state, rect, selective);
                    }
                    rect.start_row = (*state).pos.row;
                    rect.end_row = (*state).pos.row + 1 as ::core::ffi::c_int;
                    rect.end_col = (*state).pos.col + 1 as ::core::ffi::c_int;
                    if rect.end_row > rect.start_row {
                        erase(state, rect, selective);
                    }
                }
                2 => {
                    rect.start_row = 0 as ::core::ffi::c_int;
                    rect.end_row = (*state).rows;
                    rect.start_col = 0 as ::core::ffi::c_int;
                    rect.end_col = (*state).cols;
                    let mut row__1: ::core::ffi::c_int = rect.start_row;
                    while row__1 < rect.end_row {
                        set_lineinfo(state, row__1, FORCE, DWL_OFF, DHL_OFF);
                        row__1 += 1;
                    }
                    erase(state, rect, selective);
                }
                3 => {
                    if !(*state).callbacks.is_null() && (*(*state).callbacks).sb_clear.is_some() {
                        if Some(
                            (*(*state).callbacks)
                                .sb_clear
                                .expect("non-null function pointer"),
                        )
                        .expect("non-null function pointer")(
                            (*state).cbdata
                        ) != 0
                        {
                            return 1 as ::core::ffi::c_int;
                        }
                    }
                }
                _ => {}
            }
        }
        75 | 16203 => {
            selective = (leader_byte == '?' as ::core::ffi::c_int) as ::core::ffi::c_int;
            rect.start_row = (*state).pos.row;
            rect.end_row = (*state).pos.row + 1 as ::core::ffi::c_int;
            match *args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long
            {
                2147483647 | 0 => {
                    rect.start_col = (*state).pos.col;
                    rect.end_col = if (*(*state).lineinfo.offset((*state).pos.row as isize))
                        .doublewidth() as ::core::ffi::c_int
                        != 0
                    {
                        (*state).cols / 2 as ::core::ffi::c_int
                    } else {
                        (*state).cols
                    };
                }
                1 => {
                    rect.start_col = 0 as ::core::ffi::c_int;
                    rect.end_col = (*state).pos.col + 1 as ::core::ffi::c_int;
                }
                2 => {
                    rect.start_col = 0 as ::core::ffi::c_int;
                    rect.end_col = if (*(*state).lineinfo.offset((*state).pos.row as isize))
                        .doublewidth() as ::core::ffi::c_int
                        != 0
                    {
                        (*state).cols / 2 as ::core::ffi::c_int
                    } else {
                        (*state).cols
                    };
                }
                _ => return 0 as ::core::ffi::c_int,
            }
            if rect.end_col > rect.start_col {
                erase(state, rect, selective);
            }
        }
        76 => {
            count = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long)
                as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
                || *args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long
                    == 0 as ::core::ffi::c_long
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            if is_cursor_in_scrollregion(state) != 0 {
                rect.start_row = (*state).pos.row;
                rect.end_row = if (*state).scrollregion_bottom > -1 as ::core::ffi::c_int {
                    (*state).scrollregion_bottom
                } else {
                    (*state).rows
                };
                rect.start_col = if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0 {
                    (*state).scrollregion_left
                } else {
                    0 as ::core::ffi::c_int
                };
                rect.end_col = if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0
                    && (*state).scrollregion_right > -1 as ::core::ffi::c_int
                {
                    (*state).scrollregion_right
                } else {
                    (*state).cols
                };
                scroll(state, rect, -count, 0 as ::core::ffi::c_int);
            }
        }
        77 => {
            count = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long)
                as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
                || *args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long
                    == 0 as ::core::ffi::c_long
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            if is_cursor_in_scrollregion(state) != 0 {
                rect.start_row = (*state).pos.row;
                rect.end_row = if (*state).scrollregion_bottom > -1 as ::core::ffi::c_int {
                    (*state).scrollregion_bottom
                } else {
                    (*state).rows
                };
                rect.start_col = if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0 {
                    (*state).scrollregion_left
                } else {
                    0 as ::core::ffi::c_int
                };
                rect.end_col = if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0
                    && (*state).scrollregion_right > -1 as ::core::ffi::c_int
                {
                    (*state).scrollregion_right
                } else {
                    (*state).cols
                };
                scroll(state, rect, count, 0 as ::core::ffi::c_int);
            }
        }
        80 => {
            count = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long)
                as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
                || *args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long
                    == 0 as ::core::ffi::c_long
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            if is_cursor_in_scrollregion(state) != 0 {
                rect.start_row = (*state).pos.row;
                rect.end_row = (*state).pos.row + 1 as ::core::ffi::c_int;
                rect.start_col = (*state).pos.col;
                if (*state).mode.leftrightmargin() != 0 {
                    rect.end_col = if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0
                        && (*state).scrollregion_right > -1 as ::core::ffi::c_int
                    {
                        (*state).scrollregion_right
                    } else {
                        (*state).cols
                    };
                } else {
                    rect.end_col = if (*(*state).lineinfo.offset((*state).pos.row as isize))
                        .doublewidth() as ::core::ffi::c_int
                        != 0
                    {
                        (*state).cols / 2 as ::core::ffi::c_int
                    } else {
                        (*state).cols
                    };
                }
                scroll(state, rect, 0 as ::core::ffi::c_int, count);
            }
        }
        83 => {
            count = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long)
                as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
                || *args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long
                    == 0 as ::core::ffi::c_long
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            rect.start_row = (*state).scrollregion_top;
            rect.end_row = if (*state).scrollregion_bottom > -1 as ::core::ffi::c_int {
                (*state).scrollregion_bottom
            } else {
                (*state).rows
            };
            rect.start_col = if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0 {
                (*state).scrollregion_left
            } else {
                0 as ::core::ffi::c_int
            };
            rect.end_col = if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0
                && (*state).scrollregion_right > -1 as ::core::ffi::c_int
            {
                (*state).scrollregion_right
            } else {
                (*state).cols
            };
            scroll(state, rect, count, 0 as ::core::ffi::c_int);
        }
        84 => {
            count = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long)
                as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
                || *args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long
                    == 0 as ::core::ffi::c_long
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            rect.start_row = (*state).scrollregion_top;
            rect.end_row = if (*state).scrollregion_bottom > -1 as ::core::ffi::c_int {
                (*state).scrollregion_bottom
            } else {
                (*state).rows
            };
            rect.start_col = if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0 {
                (*state).scrollregion_left
            } else {
                0 as ::core::ffi::c_int
            };
            rect.end_col = if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0
                && (*state).scrollregion_right > -1 as ::core::ffi::c_int
            {
                (*state).scrollregion_right
            } else {
                (*state).cols
            };
            scroll(state, rect, -count, 0 as ::core::ffi::c_int);
        }
        88 => {
            count = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long)
                as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
                || *args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long
                    == 0 as ::core::ffi::c_long
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            rect.start_row = (*state).pos.row;
            rect.end_row = (*state).pos.row + 1 as ::core::ffi::c_int;
            rect.start_col = (*state).pos.col;
            rect.end_col = (*state).pos.col + count;
            if rect.end_col
                > (if (*(*state).lineinfo.offset((*state).pos.row as isize)).doublewidth()
                    as ::core::ffi::c_int
                    != 0
                {
                    (*state).cols / 2 as ::core::ffi::c_int
                } else {
                    (*state).cols
                })
            {
                rect.end_col = if (*(*state).lineinfo.offset((*state).pos.row as isize))
                    .doublewidth() as ::core::ffi::c_int
                    != 0
                {
                    (*state).cols / 2 as ::core::ffi::c_int
                } else {
                    (*state).cols
                };
            }
            erase(state, rect, 0 as ::core::ffi::c_int);
        }
        90 => {
            count = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long)
                as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
                || *args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long
                    == 0 as ::core::ffi::c_long
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            tab(state, count, -1 as ::core::ffi::c_int);
        }
        96 => {
            col = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long) as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            (*state).pos.col = col - 1 as ::core::ffi::c_int;
            (*state).at_phantom = 0 as ::core::ffi::c_int;
        }
        97 => {
            count = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long)
                as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
                || *args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long
                    == 0 as ::core::ffi::c_long
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            (*state).pos.col += count;
            (*state).at_phantom = 0 as ::core::ffi::c_int;
        }
        98 => {
            let row_width: ::core::ffi::c_int =
                if (*(*state).lineinfo.offset((*state).pos.row as isize)).doublewidth()
                    as ::core::ffi::c_int
                    != 0
                {
                    (*state).cols / 2 as ::core::ffi::c_int
                } else {
                    (*state).cols
                };
            count = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long)
                as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
                || *args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long
                    == 0 as ::core::ffi::c_long
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            col = (*state).pos.col + count;
            if col > row_width {
                col = row_width;
            }
            let mut sc: schar_T = schar_from_buf(
                &raw mut (*state).grapheme_buf as *mut ::core::ffi::c_char,
                (*state).grapheme_len,
            );
            while (*state).pos.col < col {
                putglyph(state, sc, (*state).combine_width, (*state).pos);
                (*state).pos.col += (*state).combine_width;
            }
            if (*state).pos.col + (*state).combine_width >= row_width {
                if (*state).mode.autowrap() != 0 {
                    (*state).at_phantom = 1 as ::core::ffi::c_int;
                    cancel_phantom = 0 as ::core::ffi::c_int;
                }
            }
        }
        99 => {
            val = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long) as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
            {
                0 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            if val == 0 as ::core::ffi::c_int {
                vterm_push_output_sprintf_ctrl(
                    (*state).vt,
                    C1_CSI as ::core::ffi::c_int as uint8_t,
                    b"?%sc\0".as_ptr() as *const ::core::ffi::c_char,
                    &raw mut vterm_primary_device_attr as *mut ::core::ffi::c_char,
                );
            }
        }
        15971 => {
            vterm_push_output_sprintf_ctrl(
                (*state).vt,
                C1_CSI as ::core::ffi::c_int as uint8_t,
                b">%d;%d;%dc\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
                100 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
        }
        100 => {
            row = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long) as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            (*state).pos.row = row - 1 as ::core::ffi::c_int;
            if (*state).mode.origin() != 0 {
                (*state).pos.row += (*state).scrollregion_top;
            }
            (*state).at_phantom = 0 as ::core::ffi::c_int;
        }
        101 => {
            count = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long)
                as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
                || *args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long
                    == 0 as ::core::ffi::c_long
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            (*state).pos.row += count;
            (*state).at_phantom = 0 as ::core::ffi::c_int;
        }
        102 => {
            row = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long) as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            col = (if argcount < 2 as ::core::ffi::c_int
                || (*args.offset(1 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long)
                    as ::core::ffi::c_ulong
                    == CSI_ARG_MISSING as ::core::ffi::c_ulong
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(1 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            (*state).pos.row = row - 1 as ::core::ffi::c_int;
            (*state).pos.col = col - 1 as ::core::ffi::c_int;
            if (*state).mode.origin() != 0 {
                (*state).pos.row += (*state).scrollregion_top;
                (*state).pos.col += if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0 {
                    (*state).scrollregion_left
                } else {
                    0 as ::core::ffi::c_int
                };
            }
            (*state).at_phantom = 0 as ::core::ffi::c_int;
        }
        103 => {
            val = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long) as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
            {
                0 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            match val {
                0 => {
                    clear_col_tabstop(state, (*state).pos.col);
                }
                3 | 5 => {
                    col = 0 as ::core::ffi::c_int;
                    while col < (*state).cols {
                        clear_col_tabstop(state, col);
                        col += 1;
                    }
                }
                1 | 2 | 4 => {}
                _ => return 0 as ::core::ffi::c_int,
            }
        }
        104 => {
            if !((*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long) as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong)
            {
                set_mode(
                    state,
                    (*args.offset(0 as ::core::ffi::c_int as isize)
                        & CSI_ARG_MASK as ::core::ffi::c_long)
                        as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
            }
        }
        16232 => {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < argcount {
                if !((*args.offset(i as isize) & CSI_ARG_MASK as ::core::ffi::c_long)
                    as ::core::ffi::c_ulong
                    == CSI_ARG_MISSING as ::core::ffi::c_ulong)
                {
                    set_dec_mode(
                        state,
                        (*args.offset(i as isize) & CSI_ARG_MASK as ::core::ffi::c_long)
                            as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                    );
                }
                i += 1;
            }
        }
        106 => {
            count = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long)
                as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
                || *args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long
                    == 0 as ::core::ffi::c_long
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            (*state).pos.col -= count;
            (*state).at_phantom = 0 as ::core::ffi::c_int;
        }
        107 => {
            count = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long)
                as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
                || *args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long
                    == 0 as ::core::ffi::c_long
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            (*state).pos.row -= count;
            (*state).at_phantom = 0 as ::core::ffi::c_int;
        }
        108 => {
            if !((*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long) as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong)
            {
                set_mode(
                    state,
                    (*args.offset(0 as ::core::ffi::c_int as isize)
                        & CSI_ARG_MASK as ::core::ffi::c_long)
                        as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
            }
        }
        16236 => {
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_0 < argcount {
                if !((*args.offset(i_0 as isize) & CSI_ARG_MASK as ::core::ffi::c_long)
                    as ::core::ffi::c_ulong
                    == CSI_ARG_MISSING as ::core::ffi::c_ulong)
                {
                    set_dec_mode(
                        state,
                        (*args.offset(i_0 as isize) & CSI_ARG_MASK as ::core::ffi::c_long)
                            as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                }
                i_0 += 1;
            }
        }
        109 => {
            vterm_state_setpen(state, args, argcount);
        }
        16237 => {
            let mut argi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while argi < argcount {
                let mut arg: ::core::ffi::c_long = 0;
                arg = *args.offset(argi as isize) & CSI_ARG_MASK as ::core::ffi::c_long;
                match arg {
                    4 => {
                        arg = 73 as ::core::ffi::c_long;
                        vterm_state_setpen(
                            state,
                            &raw mut arg as *const ::core::ffi::c_long,
                            1 as ::core::ffi::c_int,
                        );
                    }
                    5 => {
                        arg = 74 as ::core::ffi::c_long;
                        vterm_state_setpen(
                            state,
                            &raw mut arg as *const ::core::ffi::c_long,
                            1 as ::core::ffi::c_int,
                        );
                    }
                    24 => {
                        arg = 75 as ::core::ffi::c_long;
                        vterm_state_setpen(
                            state,
                            &raw mut arg as *const ::core::ffi::c_long,
                            1 as ::core::ffi::c_int,
                        );
                    }
                    _ => {}
                }
                argi += 1;
            }
        }
        110 | 16238 => {
            val = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long) as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
            {
                0 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            let mut qmark: *mut ::core::ffi::c_char = (if leader_byte == '?' as ::core::ffi::c_int {
                b"?\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            }) as *mut ::core::ffi::c_char;
            let mut dark: bool = false_0 != 0;
            match val {
                5 => {
                    vterm_push_output_sprintf_ctrl(
                        (*state).vt,
                        C1_CSI as ::core::ffi::c_int as uint8_t,
                        b"%s0n\0".as_ptr() as *const ::core::ffi::c_char,
                        qmark,
                    );
                }
                6 => {
                    vterm_push_output_sprintf_ctrl(
                        (*state).vt,
                        C1_CSI as ::core::ffi::c_int as uint8_t,
                        b"%s%d;%dR\0".as_ptr() as *const ::core::ffi::c_char,
                        qmark,
                        (*state).pos.row + 1 as ::core::ffi::c_int,
                        (*state).pos.col + 1 as ::core::ffi::c_int,
                    );
                }
                996 => {
                    if !(*state).callbacks.is_null() && (*(*state).callbacks).theme.is_some() {
                        if (*(*state).callbacks)
                            .theme
                            .expect("non-null function pointer")(
                            &raw mut dark, (*state).cbdata
                        ) != 0
                        {
                            vterm_push_output_sprintf_ctrl(
                                (*state).vt,
                                C1_CSI as ::core::ffi::c_int as uint8_t,
                                b"?997;%cn\0".as_ptr() as *const ::core::ffi::c_char,
                                if dark as ::core::ffi::c_int != 0 {
                                    '1' as ::core::ffi::c_int
                                } else {
                                    '2' as ::core::ffi::c_int
                                },
                            );
                        }
                    }
                }
                0 | 1 | 2 | 3 | 4 | _ => {}
            }
        }
        2162800 => {
            vterm_state_reset(state, 0 as ::core::ffi::c_int);
        }
        2375536 => {
            request_dec_mode(
                state,
                (*args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long) as ::core::ffi::c_int,
            );
        }
        15985 => {
            request_version_string(state);
        }
        2097265 => {
            val = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long) as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            match val {
                0 | 1 => {
                    settermprop_bool(state, VTERM_PROP_CURSORBLINK, 1 as ::core::ffi::c_int);
                    settermprop_int(
                        state,
                        VTERM_PROP_CURSORSHAPE,
                        VTERM_PROP_CURSORSHAPE_BLOCK as ::core::ffi::c_int,
                    );
                }
                2 => {
                    settermprop_bool(state, VTERM_PROP_CURSORBLINK, 0 as ::core::ffi::c_int);
                    settermprop_int(
                        state,
                        VTERM_PROP_CURSORSHAPE,
                        VTERM_PROP_CURSORSHAPE_BLOCK as ::core::ffi::c_int,
                    );
                }
                3 => {
                    settermprop_bool(state, VTERM_PROP_CURSORBLINK, 1 as ::core::ffi::c_int);
                    settermprop_int(
                        state,
                        VTERM_PROP_CURSORSHAPE,
                        VTERM_PROP_CURSORSHAPE_UNDERLINE as ::core::ffi::c_int,
                    );
                }
                4 => {
                    settermprop_bool(state, VTERM_PROP_CURSORBLINK, 0 as ::core::ffi::c_int);
                    settermprop_int(
                        state,
                        VTERM_PROP_CURSORSHAPE,
                        VTERM_PROP_CURSORSHAPE_UNDERLINE as ::core::ffi::c_int,
                    );
                }
                5 => {
                    settermprop_bool(state, VTERM_PROP_CURSORBLINK, 1 as ::core::ffi::c_int);
                    settermprop_int(
                        state,
                        VTERM_PROP_CURSORSHAPE,
                        VTERM_PROP_CURSORSHAPE_BAR_LEFT as ::core::ffi::c_int,
                    );
                }
                6 => {
                    settermprop_bool(state, VTERM_PROP_CURSORBLINK, 0 as ::core::ffi::c_int);
                    settermprop_int(
                        state,
                        VTERM_PROP_CURSORSHAPE,
                        VTERM_PROP_CURSORSHAPE_BAR_LEFT as ::core::ffi::c_int,
                    );
                }
                _ => {}
            }
        }
        2228337 => {
            val = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long) as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
            {
                0 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            match val {
                0 | 2 => {
                    (*state).set_protected_cell(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
                }
                1 => {
                    (*state).set_protected_cell(1 as ::core::ffi::c_uint as ::core::ffi::c_uint);
                }
                _ => {}
            }
        }
        114 => {
            (*state).scrollregion_top = ((if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long)
                as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) - 1 as ::core::ffi::c_long)
                as ::core::ffi::c_int;
            (*state).scrollregion_bottom = (if argcount < 2 as ::core::ffi::c_int
                || (*args.offset(1 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long)
                    as ::core::ffi::c_ulong
                    == CSI_ARG_MISSING as ::core::ffi::c_ulong
            {
                -1 as ::core::ffi::c_long
            } else {
                *args.offset(1 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            if (*state).scrollregion_top < 0 as ::core::ffi::c_int {
                (*state).scrollregion_top = 0 as ::core::ffi::c_int;
            }
            if (*state).scrollregion_top > (*state).rows {
                (*state).scrollregion_top = (*state).rows;
            }
            if (*state).scrollregion_bottom < -1 as ::core::ffi::c_int {
                (*state).scrollregion_bottom = -1 as ::core::ffi::c_int;
            }
            if (*state).scrollregion_top == 0 as ::core::ffi::c_int
                && (*state).scrollregion_bottom == (*state).rows
            {
                (*state).scrollregion_bottom = -1 as ::core::ffi::c_int;
            } else if (*state).scrollregion_bottom > (*state).rows {
                (*state).scrollregion_bottom = (*state).rows;
            }
            if (if (*state).scrollregion_bottom > -1 as ::core::ffi::c_int {
                (*state).scrollregion_bottom
            } else {
                (*state).rows
            }) <= (*state).scrollregion_top
            {
                (*state).scrollregion_top = 0 as ::core::ffi::c_int;
                (*state).scrollregion_bottom = -1 as ::core::ffi::c_int;
            }
            (*state).pos.row = 0 as ::core::ffi::c_int;
            (*state).pos.col = 0 as ::core::ffi::c_int;
            if (*state).mode.origin() != 0 {
                (*state).pos.row += (*state).scrollregion_top;
                (*state).pos.col += if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0 {
                    (*state).scrollregion_left
                } else {
                    0 as ::core::ffi::c_int
                };
            }
        }
        115 => {
            (*state).scrollregion_left = ((if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long)
                as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) - 1 as ::core::ffi::c_long)
                as ::core::ffi::c_int;
            (*state).scrollregion_right = (if argcount < 2 as ::core::ffi::c_int
                || (*args.offset(1 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long)
                    as ::core::ffi::c_ulong
                    == CSI_ARG_MISSING as ::core::ffi::c_ulong
            {
                -1 as ::core::ffi::c_long
            } else {
                *args.offset(1 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            if (*state).scrollregion_left < 0 as ::core::ffi::c_int {
                (*state).scrollregion_left = 0 as ::core::ffi::c_int;
            }
            if (*state).scrollregion_left > (*state).cols {
                (*state).scrollregion_left = (*state).cols;
            }
            if (*state).scrollregion_right < -1 as ::core::ffi::c_int {
                (*state).scrollregion_right = -1 as ::core::ffi::c_int;
            }
            if (*state).scrollregion_left == 0 as ::core::ffi::c_int
                && (*state).scrollregion_right == (*state).cols
            {
                (*state).scrollregion_right = -1 as ::core::ffi::c_int;
            } else if (*state).scrollregion_right > (*state).cols {
                (*state).scrollregion_right = (*state).cols;
            }
            if (*state).scrollregion_right > -1 as ::core::ffi::c_int
                && (*state).scrollregion_right <= (*state).scrollregion_left
            {
                (*state).scrollregion_left = 0 as ::core::ffi::c_int;
                (*state).scrollregion_right = -1 as ::core::ffi::c_int;
            }
            (*state).pos.row = 0 as ::core::ffi::c_int;
            (*state).pos.col = 0 as ::core::ffi::c_int;
            if (*state).mode.origin() != 0 {
                (*state).pos.row += (*state).scrollregion_top;
                (*state).pos.col += if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0 {
                    (*state).scrollregion_left
                } else {
                    0 as ::core::ffi::c_int
                };
            }
        }
        16245 => {
            request_key_encoding_flags(state);
        }
        15989 => {
            push_key_encoding_flags(
                state,
                (if (*args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long)
                    as ::core::ffi::c_ulong
                    == CSI_ARG_MISSING as ::core::ffi::c_ulong
                {
                    0 as ::core::ffi::c_long
                } else {
                    *args.offset(0 as ::core::ffi::c_int as isize)
                        & CSI_ARG_MASK as ::core::ffi::c_long
                }) as ::core::ffi::c_int,
            );
        }
        15477 => {
            pop_key_encoding_flags(
                state,
                (if (*args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long)
                    as ::core::ffi::c_ulong
                    == CSI_ARG_MISSING as ::core::ffi::c_ulong
                {
                    1 as ::core::ffi::c_long
                } else {
                    *args.offset(0 as ::core::ffi::c_int as isize)
                        & CSI_ARG_MASK as ::core::ffi::c_long
                }) as ::core::ffi::c_int,
            );
        }
        15733 => {
            val = (if argcount < 2 as ::core::ffi::c_int
                || (*args.offset(1 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long)
                    as ::core::ffi::c_ulong
                    == CSI_ARG_MISSING as ::core::ffi::c_ulong
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(1 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            set_key_encoding_flags(
                state,
                (if (*args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long)
                    as ::core::ffi::c_ulong
                    == CSI_ARG_MISSING as ::core::ffi::c_ulong
                {
                    0 as ::core::ffi::c_long
                } else {
                    *args.offset(0 as ::core::ffi::c_int as isize)
                        & CSI_ARG_MASK as ::core::ffi::c_long
                }) as ::core::ffi::c_int,
                val,
            );
        }
        2556029 => {
            count = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long)
                as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
                || *args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long
                    == 0 as ::core::ffi::c_long
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            if is_cursor_in_scrollregion(state) != 0 {
                rect.start_row = (*state).scrollregion_top;
                rect.end_row = if (*state).scrollregion_bottom > -1 as ::core::ffi::c_int {
                    (*state).scrollregion_bottom
                } else {
                    (*state).rows
                };
                rect.start_col = (*state).pos.col;
                rect.end_col = if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0
                    && (*state).scrollregion_right > -1 as ::core::ffi::c_int
                {
                    (*state).scrollregion_right
                } else {
                    (*state).cols
                };
                scroll(state, rect, 0 as ::core::ffi::c_int, -count);
            }
        }
        2556030 => {
            count = (if (*args.offset(0 as ::core::ffi::c_int as isize)
                & CSI_ARG_MASK as ::core::ffi::c_long)
                as ::core::ffi::c_ulong
                == CSI_ARG_MISSING as ::core::ffi::c_ulong
                || *args.offset(0 as ::core::ffi::c_int as isize)
                    & CSI_ARG_MASK as ::core::ffi::c_long
                    == 0 as ::core::ffi::c_long
            {
                1 as ::core::ffi::c_long
            } else {
                *args.offset(0 as ::core::ffi::c_int as isize) & CSI_ARG_MASK as ::core::ffi::c_long
            }) as ::core::ffi::c_int;
            if is_cursor_in_scrollregion(state) != 0 {
                rect.start_row = (*state).scrollregion_top;
                rect.end_row = if (*state).scrollregion_bottom > -1 as ::core::ffi::c_int {
                    (*state).scrollregion_bottom
                } else {
                    (*state).rows
                };
                rect.start_col = (*state).pos.col;
                rect.end_col = if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0
                    && (*state).scrollregion_right > -1 as ::core::ffi::c_int
                {
                    (*state).scrollregion_right
                } else {
                    (*state).cols
                };
                scroll(state, rect, 0 as ::core::ffi::c_int, count);
            }
        }
        _ => {
            if !(*state).fallbacks.is_null() && (*(*state).fallbacks).csi.is_some() {
                if Some(
                    (*(*state).fallbacks)
                        .csi
                        .expect("non-null function pointer"),
                )
                .expect("non-null function pointer")(
                    leader,
                    args,
                    argcount,
                    intermed,
                    command,
                    (*state).fbdata,
                ) != 0
                {
                    return 1 as ::core::ffi::c_int;
                }
            }
            return 0 as ::core::ffi::c_int;
        }
    }
    if (*state).mode.origin() != 0 {
        if (*state).pos.row < (*state).scrollregion_top {
            (*state).pos.row = (*state).scrollregion_top;
        }
        if (*state).pos.row
            > (if (*state).scrollregion_bottom > -1 as ::core::ffi::c_int {
                (*state).scrollregion_bottom
            } else {
                (*state).rows
            }) - 1 as ::core::ffi::c_int
        {
            (*state).pos.row = (if (*state).scrollregion_bottom > -1 as ::core::ffi::c_int {
                (*state).scrollregion_bottom
            } else {
                (*state).rows
            }) - 1 as ::core::ffi::c_int;
        }
        if (*state).pos.col
            < (if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0 {
                (*state).scrollregion_left
            } else {
                0 as ::core::ffi::c_int
            })
        {
            (*state).pos.col = if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0 {
                (*state).scrollregion_left
            } else {
                0 as ::core::ffi::c_int
            };
        }
        if (*state).pos.col
            > (if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0
                && (*state).scrollregion_right > -1 as ::core::ffi::c_int
            {
                (*state).scrollregion_right
            } else {
                (*state).cols
            }) - 1 as ::core::ffi::c_int
        {
            (*state).pos.col = (if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0
                && (*state).scrollregion_right > -1 as ::core::ffi::c_int
            {
                (*state).scrollregion_right
            } else {
                (*state).cols
            }) - 1 as ::core::ffi::c_int;
        }
    } else {
        if (*state).pos.row < 0 as ::core::ffi::c_int {
            (*state).pos.row = 0 as ::core::ffi::c_int;
        }
        if (*state).pos.row > (*state).rows - 1 as ::core::ffi::c_int {
            (*state).pos.row = (*state).rows - 1 as ::core::ffi::c_int;
        }
        if (*state).pos.col < 0 as ::core::ffi::c_int {
            (*state).pos.col = 0 as ::core::ffi::c_int;
        }
        if (*state).pos.col
            > (if (*(*state).lineinfo.offset((*state).pos.row as isize)).doublewidth()
                as ::core::ffi::c_int
                != 0
            {
                (*state).cols / 2 as ::core::ffi::c_int
            } else {
                (*state).cols
            }) - 1 as ::core::ffi::c_int
        {
            (*state).pos.col = (if (*(*state).lineinfo.offset((*state).pos.row as isize))
                .doublewidth() as ::core::ffi::c_int
                != 0
            {
                (*state).cols / 2 as ::core::ffi::c_int
            } else {
                (*state).cols
            }) - 1 as ::core::ffi::c_int;
        }
    }
    updatecursor(state, &raw mut oldpos, cancel_phantom);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn unbase64one(mut c: ::core::ffi::c_char) -> uint8_t {
    if c as ::core::ffi::c_int >= 'A' as ::core::ffi::c_int
        && c as ::core::ffi::c_int <= 'Z' as ::core::ffi::c_int
    {
        return (c as uint8_t as ::core::ffi::c_int - 'A' as ::core::ffi::c_int) as uint8_t;
    } else if c as ::core::ffi::c_int >= 'a' as ::core::ffi::c_int
        && c as ::core::ffi::c_int <= 'z' as ::core::ffi::c_int
    {
        return (c as uint8_t as ::core::ffi::c_int - 'a' as ::core::ffi::c_int
            + 26 as ::core::ffi::c_int) as uint8_t;
    } else if c as ::core::ffi::c_int >= '0' as ::core::ffi::c_int
        && c as ::core::ffi::c_int <= '9' as ::core::ffi::c_int
    {
        return (c as uint8_t as ::core::ffi::c_int - '0' as ::core::ffi::c_int
            + 52 as ::core::ffi::c_int) as uint8_t;
    } else if c as ::core::ffi::c_int == '+' as ::core::ffi::c_int {
        return 62 as uint8_t;
    } else if c as ::core::ffi::c_int == '/' as ::core::ffi::c_int {
        return 63 as uint8_t;
    }
    return 0xff as uint8_t;
}
unsafe extern "C" fn osc_selection(mut state: *mut VTermState, mut frag: VTermStringFragment) {
    if frag.initial() {
        (*state).tmp.selection.mask = 0 as uint16_t;
        (*state)
            .tmp
            .selection
            .set_state(SELECTION_INITIAL as C2Rust_Unnamed_4);
    }
    while (*state).tmp.selection.state() as u64 == 0 && frag.len() as ::core::ffi::c_int != 0 {
        match *frag.str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                (*state).tmp.selection.mask = ((*state).tmp.selection.mask as ::core::ffi::c_int
                    | VTERM_SELECTION_CLIPBOARD as ::core::ffi::c_int)
                    as uint16_t;
            }
            112 => {
                (*state).tmp.selection.mask = ((*state).tmp.selection.mask as ::core::ffi::c_int
                    | VTERM_SELECTION_PRIMARY as ::core::ffi::c_int)
                    as uint16_t;
            }
            113 => {
                (*state).tmp.selection.mask = ((*state).tmp.selection.mask as ::core::ffi::c_int
                    | VTERM_SELECTION_SECONDARY as ::core::ffi::c_int)
                    as uint16_t;
            }
            115 => {
                (*state).tmp.selection.mask = ((*state).tmp.selection.mask as ::core::ffi::c_int
                    | VTERM_SELECTION_SELECT as ::core::ffi::c_int)
                    as uint16_t;
            }
            48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 => {
                (*state).tmp.selection.mask = ((*state).tmp.selection.mask as ::core::ffi::c_int
                    | (VTERM_SELECTION_CUT0 as ::core::ffi::c_int)
                        << *frag.str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            - '0' as ::core::ffi::c_int)
                    as uint16_t;
            }
            59 => {
                (*state)
                    .tmp
                    .selection
                    .set_state(SELECTION_SELECTED as C2Rust_Unnamed_4);
                if (*state).tmp.selection.mask == 0 {
                    (*state).tmp.selection.mask = (VTERM_SELECTION_SELECT as ::core::ffi::c_int
                        | VTERM_SELECTION_CUT0 as ::core::ffi::c_int)
                        as uint16_t;
                }
            }
            _ => {}
        }
        frag.str = frag.str.offset(1);
        frag.set_len(frag.len() - 1 as size_t);
    }
    if frag.len() == 0 {
        if frag.final_0() as ::core::ffi::c_int != 0
            && (*(*state).selection.callbacks).set.is_some()
        {
            Some(
                (*(*state).selection.callbacks)
                    .set
                    .expect("non-null function pointer"),
            )
            .expect("non-null function pointer")(
                (*state).tmp.selection.mask as VTermSelectionMask,
                {
                    let mut init = VTermStringFragment {
                        len_initial_final_0: [0; 4],
                        str: ::core::ptr::null::<::core::ffi::c_char>(),
                        terminator: VTERM_TERMINATOR_BEL,
                    };
                    init.set_len(0 as size_t);
                    init.set_initial(
                        (*state).tmp.selection.state() as ::core::ffi::c_int
                            != SELECTION_SET as ::core::ffi::c_int,
                    );
                    init.set_final_0(true_0 != 0);
                    init
                },
                (*state).selection.user,
            );
        }
        return;
    }
    if (*state).tmp.selection.state() as ::core::ffi::c_int
        == SELECTION_SELECTED as ::core::ffi::c_int
    {
        if *frag.str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '?' as ::core::ffi::c_int
        {
            (*state)
                .tmp
                .selection
                .set_state(SELECTION_QUERY as C2Rust_Unnamed_4);
        } else {
            (*state)
                .tmp
                .selection
                .set_state(SELECTION_SET_INITIAL as C2Rust_Unnamed_4);
            (*state).tmp.selection.recvpartial = 0 as uint32_t;
        }
    }
    if (*state).tmp.selection.state() as ::core::ffi::c_int == SELECTION_QUERY as ::core::ffi::c_int
    {
        if (*(*state).selection.callbacks).query.is_some() {
            Some(
                (*(*state).selection.callbacks)
                    .query
                    .expect("non-null function pointer"),
            )
            .expect("non-null function pointer")(
                (*state).tmp.selection.mask as VTermSelectionMask,
                (*state).selection.user,
            );
        }
        return;
    }
    if (*state).tmp.selection.state() as ::core::ffi::c_int
        == SELECTION_INVALID as ::core::ffi::c_int
    {
        return;
    }
    if (*(*state).selection.callbacks).set.is_some() {
        let mut bufcur: size_t = 0 as size_t;
        let mut buffer: *mut ::core::ffi::c_char = (*state).selection.buffer;
        let mut x: uint32_t = 0 as uint32_t;
        let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*state).tmp.selection.recvpartial != 0 {
            n = ((*state).tmp.selection.recvpartial >> 24 as ::core::ffi::c_int)
                as ::core::ffi::c_int;
            x = (*state).tmp.selection.recvpartial & 0x3ffff as uint32_t;
            (*state).tmp.selection.recvpartial = 0 as uint32_t;
        }
        while (*state).selection.buflen.wrapping_sub(bufcur) >= 3 as size_t
            && frag.len() as ::core::ffi::c_int != 0
        {
            if *frag.str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '=' as ::core::ffi::c_int
            {
                if n == 2 as ::core::ffi::c_int {
                    *buffer.offset(0 as ::core::ffi::c_int as isize) =
                        (x >> 4 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
                    buffer = buffer.offset(1 as ::core::ffi::c_int as isize);
                    bufcur = bufcur.wrapping_add(1 as size_t);
                }
                if n == 3 as ::core::ffi::c_int {
                    *buffer.offset(0 as ::core::ffi::c_int as isize) =
                        (x >> 10 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
                    *buffer.offset(1 as ::core::ffi::c_int as isize) =
                        (x >> 2 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
                    buffer = buffer.offset(2 as ::core::ffi::c_int as isize);
                    bufcur = bufcur.wrapping_add(2 as size_t);
                }
                while frag.len() as ::core::ffi::c_int != 0
                    && *frag.str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '=' as ::core::ffi::c_int
                {
                    frag.str = frag.str.offset(1);
                    frag.set_len(frag.len() - 1 as size_t);
                }
                n = 0 as ::core::ffi::c_int;
            } else {
                let mut b: uint8_t =
                    unbase64one(*frag.str.offset(0 as ::core::ffi::c_int as isize));
                if b as ::core::ffi::c_int == 0xff as ::core::ffi::c_int {
                    (*state)
                        .tmp
                        .selection
                        .set_state(SELECTION_INVALID as C2Rust_Unnamed_4);
                    if (*(*state).selection.callbacks).set.is_some() {
                        Some(
                            (*(*state).selection.callbacks)
                                .set
                                .expect("non-null function pointer"),
                        )
                        .expect("non-null function pointer")(
                            (*state).tmp.selection.mask as VTermSelectionMask,
                            {
                                let mut init = VTermStringFragment {
                                    len_initial_final_0: [0; 4],
                                    str: ::core::ptr::null::<::core::ffi::c_char>(),
                                    terminator: VTERM_TERMINATOR_BEL,
                                };
                                init.set_len(0 as size_t);
                                init.set_initial(true_0 != 0);
                                init.set_final_0(true_0 != 0);
                                init
                            },
                            (*state).selection.user,
                        );
                    }
                    break;
                } else {
                    x = x << 6 as ::core::ffi::c_int | b as uint32_t;
                    n += 1;
                    frag.str = frag.str.offset(1);
                    frag.set_len(frag.len() - 1 as size_t);
                    if n == 4 as ::core::ffi::c_int {
                        *buffer.offset(0 as ::core::ffi::c_int as isize) =
                            (x >> 16 as ::core::ffi::c_int & 0xff as uint32_t)
                                as ::core::ffi::c_char;
                        *buffer.offset(1 as ::core::ffi::c_int as isize) =
                            (x >> 8 as ::core::ffi::c_int & 0xff as uint32_t)
                                as ::core::ffi::c_char;
                        *buffer.offset(2 as ::core::ffi::c_int as isize) =
                            (x >> 0 as ::core::ffi::c_int & 0xff as uint32_t)
                                as ::core::ffi::c_char;
                        buffer = buffer.offset(3 as ::core::ffi::c_int as isize);
                        bufcur = bufcur.wrapping_add(3 as size_t);
                        x = 0 as uint32_t;
                        n = 0 as ::core::ffi::c_int;
                    }
                }
            }
            if frag.len() == 0 || (*state).selection.buflen.wrapping_sub(bufcur) < 3 as size_t {
                if bufcur != 0 {
                    Some(
                        (*(*state).selection.callbacks)
                            .set
                            .expect("non-null function pointer"),
                    )
                    .expect("non-null function pointer")(
                        (*state).tmp.selection.mask as VTermSelectionMask,
                        {
                            let mut init = VTermStringFragment {
                                len_initial_final_0: [0; 4],
                                str: (*state).selection.buffer,
                                terminator: VTERM_TERMINATOR_BEL,
                            };
                            init.set_len(bufcur);
                            init.set_initial(
                                (*state).tmp.selection.state() as ::core::ffi::c_int
                                    == SELECTION_SET_INITIAL as ::core::ffi::c_int,
                            );
                            init.set_final_0(
                                frag.final_0() as ::core::ffi::c_int != 0 && frag.len() == 0,
                            );
                            init
                        },
                        (*state).selection.user,
                    );
                    (*state)
                        .tmp
                        .selection
                        .set_state(SELECTION_SET as C2Rust_Unnamed_4);
                }
                buffer = (*state).selection.buffer;
                bufcur = 0 as size_t;
            }
        }
        if n != 0 {
            (*state).tmp.selection.recvpartial = (n << 24 as ::core::ffi::c_int) as uint32_t | x;
        }
    }
}
unsafe extern "C" fn on_osc(
    mut command: ::core::ffi::c_int,
    mut frag: VTermStringFragment,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut state: *mut VTermState = user as *mut VTermState;
    match command {
        0 => {
            settermprop_string(state, VTERM_PROP_ICONNAME, frag);
            settermprop_string(state, VTERM_PROP_TITLE, frag);
        }
        1 => {
            settermprop_string(state, VTERM_PROP_ICONNAME, frag);
        }
        2 => {
            settermprop_string(state, VTERM_PROP_TITLE, frag);
        }
        52 => {
            if !(*state).selection.callbacks.is_null() {
                osc_selection(state, frag);
            }
        }
        _ => {}
    }
    if !(*state).fallbacks.is_null() && (*(*state).fallbacks).osc.is_some() {
        if Some(
            (*(*state).fallbacks)
                .osc
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(command, frag, (*state).fbdata)
            != 0
        {
            return 1 as ::core::ffi::c_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn request_status_string(
    mut state: *mut VTermState,
    mut frag: VTermStringFragment,
) {
    let mut vt: *mut VTerm = (*state).vt;
    let mut tmp: *mut ::core::ffi::c_char =
        &raw mut (*state).tmp.decrqss as *mut ::core::ffi::c_char;
    if frag.initial() {
        *tmp.offset(3 as ::core::ffi::c_int as isize) = 0 as ::core::ffi::c_char;
        *tmp.offset(2 as ::core::ffi::c_int as isize) =
            *tmp.offset(3 as ::core::ffi::c_int as isize);
        *tmp.offset(1 as ::core::ffi::c_int as isize) =
            *tmp.offset(2 as ::core::ffi::c_int as isize);
        *tmp.offset(0 as ::core::ffi::c_int as isize) =
            *tmp.offset(1 as ::core::ffi::c_int as isize);
    }
    let mut i: size_t = 0 as size_t;
    while i < ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as usize)
        && *tmp.offset(i as isize) as ::core::ffi::c_int != 0
    {
        i = i.wrapping_add(1);
    }
    while i < ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as usize) && {
        let c2rust_fresh0 = frag.len();
        frag.set_len(frag.len().wrapping_sub(1));
        c2rust_fresh0 != 0
    } {
        let c2rust_fresh1 = frag.str;
        frag.str = frag.str.offset(1);
        let c2rust_fresh2 = i;
        i = i.wrapping_add(1);
        *tmp.offset(c2rust_fresh2 as isize) =
            *c2rust_fresh1.offset(0 as ::core::ffi::c_int as isize);
    }
    *tmp.offset(i as isize) = 0 as ::core::ffi::c_char;
    if !frag.final_0() {
        return;
    }
    match *tmp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        | (*tmp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int
        | (*tmp.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int
    {
        109 => {
            let mut args: [::core::ffi::c_long; 20] = [0; 20];
            let mut argc: ::core::ffi::c_int = vterm_state_getpen(
                state,
                &raw mut args as *mut ::core::ffi::c_long,
                ::core::mem::size_of::<[::core::ffi::c_long; 20]>()
                    .wrapping_div(::core::mem::size_of::<::core::ffi::c_long>())
                    as ::core::ffi::c_int,
            );
            let mut cur: size_t = 0 as size_t;
            cur = cur.wrapping_add(snprintf(
                (*vt).tmpbuffer.offset(cur as isize),
                (*vt).tmpbuffer_len.wrapping_sub(cur),
                if (*vt).mode.ctrl8bit() as ::core::ffi::c_int != 0 {
                    b"\x901$r\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"\x1BP1$r\0".as_ptr() as *const ::core::ffi::c_char
                },
            ) as size_t);
            if cur >= (*vt).tmpbuffer_len {
                return;
            }
            let mut argi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while argi < argc {
                cur = cur.wrapping_add(snprintf(
                    (*vt).tmpbuffer.offset(cur as isize),
                    (*vt).tmpbuffer_len.wrapping_sub(cur),
                    if argi == argc - 1 as ::core::ffi::c_int {
                        b"%ld\0".as_ptr() as *const ::core::ffi::c_char
                    } else if args[argi as usize] & CSI_ARG_FLAG_MORE as ::core::ffi::c_long != 0 {
                        b"%ld:\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b"%ld;\0".as_ptr() as *const ::core::ffi::c_char
                    },
                    args[argi as usize] & CSI_ARG_MASK as ::core::ffi::c_long,
                ) as size_t);
                if cur >= (*vt).tmpbuffer_len {
                    return;
                }
                argi += 1;
            }
            cur = cur.wrapping_add(snprintf(
                (*vt).tmpbuffer.offset(cur as isize),
                (*vt).tmpbuffer_len.wrapping_sub(cur),
                if (*vt).mode.ctrl8bit() as ::core::ffi::c_int != 0 {
                    b"m\x9C\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"m\x1B\\\0".as_ptr() as *const ::core::ffi::c_char
                },
            ) as size_t);
            if cur >= (*vt).tmpbuffer_len {
                return;
            }
            vterm_push_output_bytes(vt, (*vt).tmpbuffer, cur);
            return;
        }
        114 => {
            vterm_push_output_sprintf_str(
                vt,
                C1_DCS as ::core::ffi::c_int as uint8_t,
                true_0 != 0,
                b"1$r%d;%dr\0".as_ptr() as *const ::core::ffi::c_char,
                (*state).scrollregion_top + 1 as ::core::ffi::c_int,
                if (*state).scrollregion_bottom > -1 as ::core::ffi::c_int {
                    (*state).scrollregion_bottom
                } else {
                    (*state).rows
                },
            );
            return;
        }
        115 => {
            vterm_push_output_sprintf_str(
                vt,
                C1_DCS as ::core::ffi::c_int as uint8_t,
                true_0 != 0,
                b"1$r%d;%ds\0".as_ptr() as *const ::core::ffi::c_char,
                (if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0 {
                    (*state).scrollregion_left
                } else {
                    0 as ::core::ffi::c_int
                }) + 1 as ::core::ffi::c_int,
                if (*state).mode.leftrightmargin() as ::core::ffi::c_int != 0
                    && (*state).scrollregion_right > -1 as ::core::ffi::c_int
                {
                    (*state).scrollregion_right
                } else {
                    (*state).cols
                },
            );
            return;
        }
        28960 => {
            let mut reply: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            match (*state).mode.cursor_shape() as ::core::ffi::c_int {
                1 => {
                    reply = 2 as ::core::ffi::c_int;
                }
                2 => {
                    reply = 4 as ::core::ffi::c_int;
                }
                3 => {
                    reply = 6 as ::core::ffi::c_int;
                }
                _ => {}
            }
            if (*state).mode.cursor_blink() != 0 {
                reply -= 1;
            }
            vterm_push_output_sprintf_str(
                vt,
                C1_DCS as ::core::ffi::c_int as uint8_t,
                true_0 != 0,
                b"1$r%d q\0".as_ptr() as *const ::core::ffi::c_char,
                reply,
            );
            return;
        }
        28962 => {
            vterm_push_output_sprintf_str(
                vt,
                C1_DCS as ::core::ffi::c_int as uint8_t,
                true_0 != 0,
                b"1$r%d\"q\0".as_ptr() as *const ::core::ffi::c_char,
                if (*state).protected_cell() as ::core::ffi::c_int != 0 {
                    1 as ::core::ffi::c_int
                } else {
                    2 as ::core::ffi::c_int
                },
            );
            return;
        }
        _ => {}
    }
    vterm_push_output_sprintf_str(
        (*state).vt,
        C1_DCS as ::core::ffi::c_int as uint8_t,
        true_0 != 0,
        b"0$r\0".as_ptr() as *const ::core::ffi::c_char,
    );
}
unsafe extern "C" fn on_dcs(
    mut command: *const ::core::ffi::c_char,
    mut commandlen: size_t,
    mut frag: VTermStringFragment,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut state: *mut VTermState = user as *mut VTermState;
    if commandlen == 2 as size_t
        && strncmp(
            command,
            b"$q\0".as_ptr() as *const ::core::ffi::c_char,
            2 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        request_status_string(state, frag);
        return 1 as ::core::ffi::c_int;
    } else if !(*state).fallbacks.is_null() && (*(*state).fallbacks).dcs.is_some() {
        if Some(
            (*(*state).fallbacks)
                .dcs
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(command, commandlen, frag, (*state).fbdata)
            != 0
        {
            return 1 as ::core::ffi::c_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn on_apc(
    mut frag: VTermStringFragment,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut state: *mut VTermState = user as *mut VTermState;
    if !(*state).fallbacks.is_null() && (*(*state).fallbacks).apc.is_some() {
        if Some(
            (*(*state).fallbacks)
                .apc
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(frag, (*state).fbdata)
            != 0
        {
            return 1 as ::core::ffi::c_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn on_pm(
    mut frag: VTermStringFragment,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut state: *mut VTermState = user as *mut VTermState;
    if !(*state).fallbacks.is_null() && (*(*state).fallbacks).pm.is_some() {
        if Some((*(*state).fallbacks).pm.expect("non-null function pointer"))
            .expect("non-null function pointer")(frag, (*state).fbdata)
            != 0
        {
            return 1 as ::core::ffi::c_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn on_sos(
    mut frag: VTermStringFragment,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut state: *mut VTermState = user as *mut VTermState;
    if !(*state).fallbacks.is_null() && (*(*state).fallbacks).sos.is_some() {
        if Some(
            (*(*state).fallbacks)
                .sos
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(frag, (*state).fbdata)
            != 0
        {
            return 1 as ::core::ffi::c_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn on_resize(
    mut rows: ::core::ffi::c_int,
    mut cols: ::core::ffi::c_int,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut state: *mut VTermState = user as *mut VTermState;
    let mut oldpos: VTermPos = (*state).pos;
    if cols != (*state).cols {
        let mut newtabstops: *mut uint8_t = vterm_allocator_malloc(
            (*state).vt,
            (cols as size_t)
                .wrapping_add(7 as size_t)
                .wrapping_div(8 as size_t),
        ) as *mut uint8_t;
        let mut col: ::core::ffi::c_int = 0;
        col = 0 as ::core::ffi::c_int;
        while col < (*state).cols && col < cols {
            let mut mask: uint8_t =
                ((1 as ::core::ffi::c_int) << (col & 7 as ::core::ffi::c_int)) as uint8_t;
            if *(*state)
                .tabstops
                .offset((col >> 3 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                & mask as ::core::ffi::c_int
                != 0
            {
                *newtabstops.offset((col >> 3 as ::core::ffi::c_int) as isize) =
                    (*newtabstops.offset((col >> 3 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        | mask as ::core::ffi::c_int) as uint8_t;
            } else {
                *newtabstops.offset((col >> 3 as ::core::ffi::c_int) as isize) =
                    (*newtabstops.offset((col >> 3 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        & !(mask as ::core::ffi::c_int)) as uint8_t;
            }
            col += 1;
        }
        while col < cols {
            let mut mask_0: uint8_t =
                ((1 as ::core::ffi::c_int) << (col & 7 as ::core::ffi::c_int)) as uint8_t;
            if col % 8 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                *newtabstops.offset((col >> 3 as ::core::ffi::c_int) as isize) =
                    (*newtabstops.offset((col >> 3 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        | mask_0 as ::core::ffi::c_int) as uint8_t;
            } else {
                *newtabstops.offset((col >> 3 as ::core::ffi::c_int) as isize) =
                    (*newtabstops.offset((col >> 3 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        & !(mask_0 as ::core::ffi::c_int)) as uint8_t;
            }
            col += 1;
        }
        vterm_allocator_free((*state).vt, (*state).tabstops as *mut ::core::ffi::c_void);
        (*state).tabstops = newtabstops;
    }
    (*state).rows = rows;
    (*state).cols = cols;
    if (*state).scrollregion_bottom > -1 as ::core::ffi::c_int {
        if (*state).scrollregion_bottom > (*state).rows {
            (*state).scrollregion_bottom = (*state).rows;
        }
    }
    if (*state).scrollregion_right > -1 as ::core::ffi::c_int {
        if (*state).scrollregion_right > (*state).cols {
            (*state).scrollregion_right = (*state).cols;
        }
    }
    let mut fields: VTermStateFields = VTermStateFields {
        pos: (*state).pos,
        lineinfos: [
            (*state).lineinfos[0 as ::core::ffi::c_int as usize],
            (*state).lineinfos[1 as ::core::ffi::c_int as usize],
        ],
    };
    if !(*state).callbacks.is_null() && (*(*state).callbacks).resize.is_some() {
        Some(
            (*(*state).callbacks)
                .resize
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(rows, cols, &raw mut fields, (*state).cbdata);
        (*state).pos = fields.pos;
        (*state).lineinfos[0 as ::core::ffi::c_int as usize] =
            fields.lineinfos[0 as ::core::ffi::c_int as usize];
        (*state).lineinfos[1 as ::core::ffi::c_int as usize] =
            fields.lineinfos[1 as ::core::ffi::c_int as usize];
    } else if rows != (*state).rows {
        let mut bufidx: ::core::ffi::c_int = BUFIDX_PRIMARY;
        while bufidx <= BUFIDX_ALTSCREEN {
            let mut oldlineinfo: *mut VTermLineInfo = (*state).lineinfos[bufidx as usize];
            if !oldlineinfo.is_null() {
                let mut newlineinfo: *mut VTermLineInfo = vterm_allocator_malloc(
                    (*state).vt,
                    (rows as size_t).wrapping_mul(::core::mem::size_of::<VTermLineInfo>()),
                ) as *mut VTermLineInfo;
                let mut row: ::core::ffi::c_int = 0;
                row = 0 as ::core::ffi::c_int;
                while row < (*state).rows && row < rows {
                    *newlineinfo.offset(row as isize) = *oldlineinfo.offset(row as isize);
                    row += 1;
                }
                while row < rows {
                    *newlineinfo.offset(row as isize) = {
                        let mut init = VTermLineInfo {
                            doublewidth_doubleheight_continuation: [0; 1],
                            c2rust_padding: [0; 3],
                        };
                        init.set_doublewidth(0 as ::core::ffi::c_uint);
                        init.set_doubleheight(0);
                        init.set_continuation(0);
                        init
                    };
                    row += 1;
                }
                vterm_allocator_free(
                    (*state).vt,
                    (*state).lineinfos[bufidx as usize] as *mut ::core::ffi::c_void,
                );
                (*state).lineinfos[bufidx as usize] = newlineinfo;
            }
            bufidx += 1;
        }
    }
    (*state).lineinfo = (*state).lineinfos[(if (*state).mode.alt_screen() as ::core::ffi::c_int != 0
    {
        BUFIDX_ALTSCREEN
    } else {
        BUFIDX_PRIMARY
    }) as usize];
    if (*state).at_phantom != 0 && (*state).pos.col < cols - 1 as ::core::ffi::c_int {
        (*state).at_phantom = 0 as ::core::ffi::c_int;
        (*state).pos.col += 1;
    }
    if (*state).pos.row < 0 as ::core::ffi::c_int {
        (*state).pos.row = 0 as ::core::ffi::c_int;
    }
    if (*state).pos.row >= rows {
        (*state).pos.row = rows - 1 as ::core::ffi::c_int;
    }
    if (*state).pos.col < 0 as ::core::ffi::c_int {
        (*state).pos.col = 0 as ::core::ffi::c_int;
    }
    if (*state).pos.col >= cols {
        (*state).pos.col = cols - 1 as ::core::ffi::c_int;
    }
    updatecursor(state, &raw mut oldpos, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
static mut parser_callbacks: VTermParserCallbacks = VTermParserCallbacks {
    text: Some(
        on_text
            as unsafe extern "C" fn(
                *const ::core::ffi::c_char,
                size_t,
                *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
    ),
    control: Some(
        on_control as unsafe extern "C" fn(uint8_t, *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    ),
    escape: Some(
        on_escape
            as unsafe extern "C" fn(
                *const ::core::ffi::c_char,
                size_t,
                *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
    ),
    csi: Some(
        on_csi
            as unsafe extern "C" fn(
                *const ::core::ffi::c_char,
                *const ::core::ffi::c_long,
                ::core::ffi::c_int,
                *const ::core::ffi::c_char,
                ::core::ffi::c_char,
                *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
    ),
    osc: Some(
        on_osc
            as unsafe extern "C" fn(
                ::core::ffi::c_int,
                VTermStringFragment,
                *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
    ),
    dcs: Some(
        on_dcs
            as unsafe extern "C" fn(
                *const ::core::ffi::c_char,
                size_t,
                VTermStringFragment,
                *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
    ),
    apc: Some(
        on_apc
            as unsafe extern "C" fn(
                VTermStringFragment,
                *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
    ),
    pm: Some(
        on_pm
            as unsafe extern "C" fn(
                VTermStringFragment,
                *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
    ),
    sos: Some(
        on_sos
            as unsafe extern "C" fn(
                VTermStringFragment,
                *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
    ),
    resize: Some(
        on_resize
            as unsafe extern "C" fn(
                ::core::ffi::c_int,
                ::core::ffi::c_int,
                *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
    ),
};
#[no_mangle]
pub unsafe extern "C" fn vterm_obtain_state(mut vt: *mut VTerm) -> *mut VTermState {
    if !(*vt).state.is_null() {
        return (*vt).state;
    }
    let mut state: *mut VTermState = vterm_state_new(vt);
    (*vt).state = state;
    vterm_parser_set_callbacks(
        vt,
        &raw const parser_callbacks,
        state as *mut ::core::ffi::c_void,
    );
    return state;
}
#[no_mangle]
pub unsafe extern "C" fn vterm_state_reset(
    mut state: *mut VTermState,
    mut hard: ::core::ffi::c_int,
) {
    (*state).scrollregion_top = 0 as ::core::ffi::c_int;
    (*state).scrollregion_bottom = -1 as ::core::ffi::c_int;
    (*state).scrollregion_left = 0 as ::core::ffi::c_int;
    (*state).scrollregion_right = -1 as ::core::ffi::c_int;
    (*state)
        .mode
        .set_keypad(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    (*state)
        .mode
        .set_cursor(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    (*state)
        .mode
        .set_autowrap(1 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    (*state)
        .mode
        .set_insert(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    (*state)
        .mode
        .set_newline(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    (*state)
        .mode
        .set_alt_screen(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    (*state)
        .mode
        .set_origin(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    (*state)
        .mode
        .set_leftrightmargin(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    (*state)
        .mode
        .set_bracketpaste(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    (*state)
        .mode
        .set_report_focus(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    (*state).mouse_flags = 0 as ::core::ffi::c_int;
    (*(*state).vt)
        .mode
        .set_ctrl8bit(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while col < (*state).cols {
        if col % 8 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            set_col_tabstop(state, col);
        } else {
            clear_col_tabstop(state, col);
        }
        col += 1;
    }
    let mut row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while row < (*state).rows {
        set_lineinfo(state, row, FORCE, DWL_OFF, DHL_OFF);
        row += 1;
    }
    if !(*state).callbacks.is_null() && (*(*state).callbacks).initpen.is_some() {
        Some(
            (*(*state).callbacks)
                .initpen
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")((*state).cbdata);
    }
    vterm_state_resetpen(state);
    let mut default_enc: *mut VTermEncoding =
        if (*(*state).vt).mode.utf8() as ::core::ffi::c_int != 0 {
            vterm_lookup_encoding(ENC_UTF8, 'u' as ::core::ffi::c_char)
        } else {
            vterm_lookup_encoding(ENC_SINGLE_94, 'B' as ::core::ffi::c_char)
        };
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        (*state).encoding[i as usize].enc = default_enc;
        if (*default_enc).init.is_some() {
            Some((*default_enc).init.expect("non-null function pointer"))
                .expect("non-null function pointer")(
                default_enc,
                &raw mut (*(&raw mut (*state).encoding as *mut VTermEncodingInstance)
                    .offset(i as isize))
                .data as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            );
        }
        i += 1;
    }
    (*state).gl_set = 0 as ::core::ffi::c_int;
    (*state).gr_set = 1 as ::core::ffi::c_int;
    (*state).gsingle_set = 0 as ::core::ffi::c_int;
    (*state).set_protected_cell(0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    settermprop_bool(state, VTERM_PROP_CURSORVISIBLE, 1 as ::core::ffi::c_int);
    settermprop_bool(state, VTERM_PROP_CURSORBLINK, 1 as ::core::ffi::c_int);
    settermprop_int(
        state,
        VTERM_PROP_CURSORSHAPE,
        VTERM_PROP_CURSORSHAPE_BLOCK as ::core::ffi::c_int,
    );
    if hard != 0 {
        (*state).pos.row = 0 as ::core::ffi::c_int;
        (*state).pos.col = 0 as ::core::ffi::c_int;
        (*state).at_phantom = 0 as ::core::ffi::c_int;
        let mut rect: VTermRect = VTermRect {
            start_row: 0 as ::core::ffi::c_int,
            end_row: (*state).rows,
            start_col: 0 as ::core::ffi::c_int,
            end_col: (*state).cols,
        };
        erase(state, rect, 0 as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn vterm_state_set_callbacks(
    mut state: *mut VTermState,
    mut callbacks: *const VTermStateCallbacks,
    mut user: *mut ::core::ffi::c_void,
) {
    if !callbacks.is_null() {
        (*state).callbacks = callbacks;
        (*state).cbdata = user;
        if !(*state).callbacks.is_null() && (*(*state).callbacks).initpen.is_some() {
            Some(
                (*(*state).callbacks)
                    .initpen
                    .expect("non-null function pointer"),
            )
            .expect("non-null function pointer")((*state).cbdata);
        }
    } else {
        (*state).callbacks = ::core::ptr::null::<VTermStateCallbacks>();
        (*state).cbdata = NULL;
    };
}
#[no_mangle]
pub unsafe extern "C" fn vterm_state_set_unrecognised_fallbacks(
    mut state: *mut VTermState,
    mut fallbacks: *const VTermStateFallbacks,
    mut user: *mut ::core::ffi::c_void,
) {
    if !fallbacks.is_null() {
        (*state).fallbacks = fallbacks;
        (*state).fbdata = user;
    } else {
        (*state).fallbacks = ::core::ptr::null::<VTermStateFallbacks>();
        (*state).fbdata = NULL;
    };
}
#[no_mangle]
pub unsafe extern "C" fn vterm_state_set_termprop(
    mut state: *mut VTermState,
    mut prop: VTermProp,
    mut val: *mut VTermValue,
) -> ::core::ffi::c_int {
    if !(*state).callbacks.is_null() && (*(*state).callbacks).settermprop.is_some() {
        if Some(
            (*(*state).callbacks)
                .settermprop
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(prop, val, (*state).cbdata)
            == 0
        {
            return 0 as ::core::ffi::c_int;
        }
    }
    match prop as ::core::ffi::c_uint {
        4 | 5 => return 1 as ::core::ffi::c_int,
        1 => {
            (*state)
                .mode
                .set_cursor_visible((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
            return 1 as ::core::ffi::c_int;
        }
        2 => {
            (*state)
                .mode
                .set_cursor_blink((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
            return 1 as ::core::ffi::c_int;
        }
        7 => {
            (*state)
                .mode
                .set_cursor_shape((*val).number as ::core::ffi::c_uint as ::core::ffi::c_uint);
            return 1 as ::core::ffi::c_int;
        }
        6 => {
            (*state)
                .mode
                .set_screen((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
            return 1 as ::core::ffi::c_int;
        }
        3 => {
            (*state)
                .mode
                .set_alt_screen((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
            (*state).lineinfo =
                (*state).lineinfos[(if (*state).mode.alt_screen() as ::core::ffi::c_int != 0 {
                    BUFIDX_ALTSCREEN
                } else {
                    BUFIDX_PRIMARY
                }) as usize];
            if (*state).mode.alt_screen() != 0 {
                let mut rect: VTermRect = VTermRect {
                    start_row: 0 as ::core::ffi::c_int,
                    end_row: (*state).rows,
                    start_col: 0 as ::core::ffi::c_int,
                    end_col: (*state).cols,
                };
                erase(state, rect, 0 as ::core::ffi::c_int);
            }
            return 1 as ::core::ffi::c_int;
        }
        8 => {
            (*state).mouse_flags = 0 as ::core::ffi::c_int;
            if (*val).number != 0 {
                (*state).mouse_flags |= MOUSE_WANT_CLICK;
            }
            if (*val).number == VTERM_PROP_MOUSE_DRAG as ::core::ffi::c_int {
                (*state).mouse_flags |= MOUSE_WANT_DRAG;
            }
            if (*val).number == VTERM_PROP_MOUSE_MOVE as ::core::ffi::c_int {
                (*state).mouse_flags |= MOUSE_WANT_MOVE;
            }
            return 1 as ::core::ffi::c_int;
        }
        9 => {
            (*state)
                .mode
                .set_report_focus((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
            return 1 as ::core::ffi::c_int;
        }
        10 => {
            (*state)
                .mode
                .set_theme_updates((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
            return 1 as ::core::ffi::c_int;
        }
        11 => {
            (*state).mode.set_synchronized_output(
                (*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint,
            );
            return 1 as ::core::ffi::c_int;
        }
        12 => return 0 as ::core::ffi::c_int,
        _ => {}
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn vterm_state_focus_in(mut state: *mut VTermState) {
    if (*state).mode.report_focus() != 0 {
        vterm_push_output_sprintf_ctrl(
            (*state).vt,
            C1_CSI as ::core::ffi::c_int as uint8_t,
            b"I\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn vterm_state_focus_out(mut state: *mut VTermState) {
    if (*state).mode.report_focus() != 0 {
        vterm_push_output_sprintf_ctrl(
            (*state).vt,
            C1_CSI as ::core::ffi::c_int as uint8_t,
            b"O\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn vterm_state_get_lineinfo(
    mut state: *const VTermState,
    mut row: ::core::ffi::c_int,
) -> *const VTermLineInfo {
    return (*state).lineinfo.offset(row as isize);
}
#[no_mangle]
pub unsafe extern "C" fn vterm_state_set_selection_callbacks(
    mut state: *mut VTermState,
    mut callbacks: *const VTermSelectionCallbacks,
    mut user: *mut ::core::ffi::c_void,
    mut buffer: *mut ::core::ffi::c_char,
    mut buflen: size_t,
) {
    if buflen != 0 && buffer.is_null() {
        buffer = vterm_allocator_malloc((*state).vt, buflen) as *mut ::core::ffi::c_char;
    }
    (*state).selection.callbacks = callbacks;
    (*state).selection.user = user;
    (*state).selection.buffer = buffer;
    (*state).selection.buflen = buflen;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
