use crate::src::nvim::global_cell::GlobalCell;
use ::c2rust_bitfields;
extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    static mut stderr: *mut FILE;
    fn fprintf(
        __stream: *mut FILE,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn memmove(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn abort() -> !;
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
    fn vterm_state_convert_color_to_rgb(state: *const VTermState, col: *mut VTermColor);
    fn vterm_obtain_state(vt: *mut VTerm) -> *mut VTermState;
    fn vterm_state_reset(state: *mut VTermState, hard: ::core::ffi::c_int);
    fn vterm_state_set_callbacks(
        state: *mut VTermState,
        callbacks: *const VTermStateCallbacks,
        user: *mut ::core::ffi::c_void,
    );
    fn vterm_state_set_unrecognised_fallbacks(
        state: *mut VTermState,
        fallbacks: *const VTermStateFallbacks,
        user: *mut ::core::ffi::c_void,
    );
    fn vterm_state_get_lineinfo(
        state: *const VTermState,
        row: ::core::ffi::c_int,
    ) -> *const VTermLineInfo;
    fn vterm_allocator_malloc(vt: *mut VTerm, size: size_t) -> *mut ::core::ffi::c_void;
    fn vterm_allocator_free(vt: *mut VTerm, ptr: *mut ::core::ffi::c_void);
    fn vterm_get_size(
        vt: *const VTerm,
        rowsp: *mut ::core::ffi::c_int,
        colsp: *mut ::core::ffi::c_int,
    );
}
pub type size_t = usize;
pub type __off_t = ::core::ffi::c_long;
pub type __off64_t = ::core::ffi::c_long;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _IO_FILE {
    pub _flags: ::core::ffi::c_int,
    pub _IO_read_ptr: *mut ::core::ffi::c_char,
    pub _IO_read_end: *mut ::core::ffi::c_char,
    pub _IO_read_base: *mut ::core::ffi::c_char,
    pub _IO_write_base: *mut ::core::ffi::c_char,
    pub _IO_write_ptr: *mut ::core::ffi::c_char,
    pub _IO_write_end: *mut ::core::ffi::c_char,
    pub _IO_buf_base: *mut ::core::ffi::c_char,
    pub _IO_buf_end: *mut ::core::ffi::c_char,
    pub _IO_save_base: *mut ::core::ffi::c_char,
    pub _IO_backup_base: *mut ::core::ffi::c_char,
    pub _IO_save_end: *mut ::core::ffi::c_char,
    pub _markers: *mut _IO_marker,
    pub _chain: *mut _IO_FILE,
    pub _fileno: ::core::ffi::c_int,
    pub _flags2: ::core::ffi::c_int,
    pub _old_offset: __off_t,
    pub _cur_column: ::core::ffi::c_ushort,
    pub _vtable_offset: ::core::ffi::c_schar,
    pub _shortbuf: [::core::ffi::c_char; 1],
    pub _lock: *mut ::core::ffi::c_void,
    pub _offset: __off64_t,
    pub _codecvt: *mut _IO_codecvt,
    pub _wide_data: *mut _IO_wide_data,
    pub _freeres_list: *mut _IO_FILE,
    pub _freeres_buf: *mut ::core::ffi::c_void,
    pub _prevchain: *mut *mut _IO_FILE,
    pub _mode: ::core::ffi::c_int,
    pub _unused2: [::core::ffi::c_char; 20],
}
pub type _IO_lock_t = ();
pub type FILE = _IO_FILE;
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
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const BUFIDX_PRIMARY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const BUFIDX_ALTSCREEN: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn vterm_rect_move(
    mut rect: *mut VTermRect,
    mut row_delta: ::core::ffi::c_int,
    mut col_delta: ::core::ffi::c_int,
) {
    (*rect).start_row += row_delta;
    (*rect).end_row += row_delta;
    (*rect).start_col += col_delta;
    (*rect).end_col += col_delta;
}
#[inline]
unsafe extern "C" fn clearcell(mut screen: *const VTermScreen, mut cell: *mut ScreenCell) {
    (*cell).schar = 0 as schar_T;
    (*cell).pen = (*screen).pen;
}
#[no_mangle]
pub unsafe extern "C" fn getcell(
    mut screen: *const VTermScreen,
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
) -> *mut ScreenCell {
    if row < 0 as ::core::ffi::c_int || row >= (*screen).rows {
        return ::core::ptr::null_mut::<ScreenCell>();
    }
    if col < 0 as ::core::ffi::c_int || col >= (*screen).cols {
        return ::core::ptr::null_mut::<ScreenCell>();
    }
    return (*screen)
        .buffer
        .offset(((*screen).cols * row) as isize)
        .offset(col as isize);
}
unsafe extern "C" fn alloc_buffer(
    mut screen: *mut VTermScreen,
    mut rows: ::core::ffi::c_int,
    mut cols: ::core::ffi::c_int,
) -> *mut ScreenCell {
    let mut new_buffer: *mut ScreenCell = vterm_allocator_malloc(
        (*screen).vt,
        ::core::mem::size_of::<ScreenCell>()
            .wrapping_mul(rows as size_t)
            .wrapping_mul(cols as size_t),
    ) as *mut ScreenCell;
    let mut row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while row < rows {
        let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while col < cols {
            clearcell(screen, new_buffer.offset((row * cols + col) as isize));
            col += 1;
        }
        row += 1;
    }
    return new_buffer;
}
unsafe extern "C" fn damagerect(mut screen: *mut VTermScreen, mut rect: VTermRect) {
    let mut emit: VTermRect = VTermRect {
        start_row: 0,
        end_row: 0,
        start_col: 0,
        end_col: 0,
    };
    match (*screen).damage_merge as ::core::ffi::c_uint {
        0 => {
            emit = rect;
        }
        1 => {
            if rect.end_row > rect.start_row + 1 as ::core::ffi::c_int {
                vterm_screen_flush_damage(screen);
                emit = rect;
            } else if (*screen).damaged.start_row == -1 as ::core::ffi::c_int {
                (*screen).damaged = rect;
                return;
            } else if rect.start_row == (*screen).damaged.start_row {
                if (*screen).damaged.start_col > rect.start_col {
                    (*screen).damaged.start_col = rect.start_col;
                }
                if (*screen).damaged.end_col < rect.end_col {
                    (*screen).damaged.end_col = rect.end_col;
                }
                return;
            } else {
                emit = (*screen).damaged;
                (*screen).damaged = rect;
            }
        }
        2 | 3 => {
            if (*screen).damaged.start_row == -1 as ::core::ffi::c_int {
                (*screen).damaged = rect;
            } else {
                rect_expand(&raw mut (*screen).damaged, &raw mut rect);
            }
            return;
        }
        _ => return,
    }
    if !(*screen).callbacks.is_null() && (*(*screen).callbacks).damage.is_some() {
        Some(
            (*(*screen).callbacks)
                .damage
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(emit, (*screen).cbdata);
    }
}
unsafe extern "C" fn damagescreen(mut screen: *mut VTermScreen) {
    let mut rect: VTermRect = VTermRect {
        start_row: 0 as ::core::ffi::c_int,
        end_row: (*screen).rows,
        start_col: 0 as ::core::ffi::c_int,
        end_col: (*screen).cols,
    };
    damagerect(screen, rect);
}
unsafe extern "C" fn putglyph(
    mut info: *mut VTermGlyphInfo,
    mut pos: VTermPos,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut screen: *mut VTermScreen = user as *mut VTermScreen;
    let mut cell: *mut ScreenCell = getcell(screen, pos.row, pos.col);
    if cell.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    (*cell).schar = (*info).schar;
    if (*info).schar != 0 as schar_T {
        (*cell).pen = (*screen).pen;
    }
    let mut col: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while col < (*info).width {
        (*getcell(screen, pos.row, pos.col + col)).schar =
            -1 as ::core::ffi::c_int as uint32_t as schar_T;
        col += 1;
    }
    let mut rect: VTermRect = VTermRect {
        start_row: pos.row,
        end_row: pos.row + 1 as ::core::ffi::c_int,
        start_col: pos.col,
        end_col: pos.col + (*info).width,
    };
    (*cell)
        .pen
        .set_protected_cell((*info).protected_cell() as ::core::ffi::c_uint);
    (*cell).pen.set_dwl((*info).dwl() as ::core::ffi::c_uint);
    (*cell).pen.set_dhl((*info).dhl() as ::core::ffi::c_uint);
    damagerect(screen, rect);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn sb_pushline_from_row(
    mut screen: *mut VTermScreen,
    mut row: ::core::ffi::c_int,
) {
    let mut pos: VTermPos = VTermPos { row: row, col: 0 };
    pos.col = 0 as ::core::ffi::c_int;
    while pos.col < (*screen).cols {
        vterm_screen_get_cell(screen, pos, (*screen).sb_buffer.offset(pos.col as isize));
        pos.col += 1;
    }
    (*(*screen).callbacks)
        .sb_pushline
        .expect("non-null function pointer")(
        (*screen).cols, (*screen).sb_buffer, (*screen).cbdata
    );
}
unsafe extern "C" fn moverect_internal(
    mut dest: VTermRect,
    mut src: VTermRect,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut screen: *mut VTermScreen = user as *mut VTermScreen;
    if !(*screen).callbacks.is_null()
        && (*(*screen).callbacks).sb_pushline.is_some()
        && dest.start_row == 0 as ::core::ffi::c_int
        && dest.start_col == 0 as ::core::ffi::c_int
        && dest.end_col == (*screen).cols
        && (*screen).buffer == (*screen).buffers[BUFIDX_PRIMARY as usize]
    {
        let mut row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while row < src.start_row {
            sb_pushline_from_row(screen, row);
            row += 1;
        }
    }
    let mut cols: ::core::ffi::c_int = src.end_col - src.start_col;
    let mut downward: ::core::ffi::c_int = src.start_row - dest.start_row;
    let mut init_row: ::core::ffi::c_int = 0;
    let mut test_row: ::core::ffi::c_int = 0;
    let mut inc_row: ::core::ffi::c_int = 0;
    if downward < 0 as ::core::ffi::c_int {
        init_row = dest.end_row - 1 as ::core::ffi::c_int;
        test_row = dest.start_row - 1 as ::core::ffi::c_int;
        inc_row = -1 as ::core::ffi::c_int;
    } else {
        init_row = dest.start_row;
        test_row = dest.end_row;
        inc_row = 1 as ::core::ffi::c_int;
    }
    let mut row_0: ::core::ffi::c_int = init_row;
    while row_0 != test_row {
        memmove(
            getcell(screen, row_0, dest.start_col) as *mut ::core::ffi::c_void,
            getcell(screen, row_0 + downward, src.start_col) as *const ::core::ffi::c_void,
            (cols as size_t).wrapping_mul(::core::mem::size_of::<ScreenCell>()),
        );
        row_0 += inc_row;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn moverect_user(
    mut dest: VTermRect,
    mut src: VTermRect,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut screen: *mut VTermScreen = user as *mut VTermScreen;
    if !(*screen).callbacks.is_null() && (*(*screen).callbacks).moverect.is_some() {
        if (*screen).damage_merge as ::core::ffi::c_uint
            != VTERM_DAMAGE_SCROLL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            vterm_screen_flush_damage(screen);
        }
        if Some(
            (*(*screen).callbacks)
                .moverect
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(dest, src, (*screen).cbdata)
            != 0
        {
            return 1 as ::core::ffi::c_int;
        }
    }
    damagerect(screen, dest);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn erase_internal(
    mut rect: VTermRect,
    mut selective: ::core::ffi::c_int,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut screen: *mut VTermScreen = user as *mut VTermScreen;
    let mut row: ::core::ffi::c_int = rect.start_row;
    while row < (*(*screen).state).rows && row < rect.end_row {
        let mut info: *const VTermLineInfo = vterm_state_get_lineinfo((*screen).state, row);
        let mut col: ::core::ffi::c_int = rect.start_col;
        while col < rect.end_col {
            let mut cell: *mut ScreenCell = getcell(screen, row, col);
            if !(selective != 0 && (*cell).pen.protected_cell() as ::core::ffi::c_int != 0) {
                (*cell).schar = 0 as schar_T;
                (*cell).pen = {
                    let mut init = ScreenPen {
                        bold_underline_italic_blink_reverse_conceal_strike_font_small_baseline_dim_overline_protected_cell_dwl_dhl: [0; 3],
                        c2rust_padding: [0; 1],
                        fg: (*screen).pen.fg,
                        bg: (*screen).pen.bg,
                        uri: 0,
                    };
                    init.set_bold(0);
                    init.set_underline(0);
                    init.set_italic(0);
                    init.set_blink(0);
                    init.set_reverse(0);
                    init.set_conceal(0);
                    init.set_strike(0);
                    init.set_font(0);
                    init.set_small(0);
                    init.set_baseline(0);
                    init.set_dim(0);
                    init.set_overline(0);
                    init.set_protected_cell(0);
                    init.set_dwl(0);
                    init.set_dhl(0);
                    init
                };
                (*cell)
                    .pen
                    .set_dwl((*info).doublewidth() as ::core::ffi::c_uint);
                (*cell)
                    .pen
                    .set_dhl((*info).doubleheight() as ::core::ffi::c_uint);
            }
            col += 1;
        }
        row += 1;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn erase_user(
    mut rect: VTermRect,
    mut _selective: ::core::ffi::c_int,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut screen: *mut VTermScreen = user as *mut VTermScreen;
    damagerect(screen, rect);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn erase(
    mut rect: VTermRect,
    mut selective: ::core::ffi::c_int,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    erase_internal(rect, selective, user);
    return erase_user(rect, 0 as ::core::ffi::c_int, user);
}
unsafe extern "C" fn scrollrect(
    mut rect: VTermRect,
    mut downward: ::core::ffi::c_int,
    mut rightward: ::core::ffi::c_int,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut screen: *mut VTermScreen = user as *mut VTermScreen;
    if (*screen).damage_merge as ::core::ffi::c_uint
        != VTERM_DAMAGE_SCROLL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        vterm_scroll_rect(
            rect,
            downward,
            rightward,
            Some(
                moverect_internal
                    as unsafe extern "C" fn(
                        VTermRect,
                        VTermRect,
                        *mut ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
            Some(
                erase_internal
                    as unsafe extern "C" fn(
                        VTermRect,
                        ::core::ffi::c_int,
                        *mut ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
            screen as *mut ::core::ffi::c_void,
        );
        vterm_screen_flush_damage(screen);
        vterm_scroll_rect(
            rect,
            downward,
            rightward,
            Some(
                moverect_user
                    as unsafe extern "C" fn(
                        VTermRect,
                        VTermRect,
                        *mut ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
            Some(
                erase_user
                    as unsafe extern "C" fn(
                        VTermRect,
                        ::core::ffi::c_int,
                        *mut ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
            screen as *mut ::core::ffi::c_void,
        );
        return 1 as ::core::ffi::c_int;
    }
    if (*screen).damaged.start_row != -1 as ::core::ffi::c_int
        && rect_intersects(&raw mut rect, &raw mut (*screen).damaged) == 0
    {
        vterm_screen_flush_damage(screen);
    }
    if (*screen).pending_scrollrect.start_row == -1 as ::core::ffi::c_int {
        (*screen).pending_scrollrect = rect;
        (*screen).pending_scroll_downward = downward;
        (*screen).pending_scroll_rightward = rightward;
    } else if rect_equal(&raw mut (*screen).pending_scrollrect, &raw mut rect) != 0
        && ((*screen).pending_scroll_downward == 0 as ::core::ffi::c_int
            && downward == 0 as ::core::ffi::c_int
            || (*screen).pending_scroll_rightward == 0 as ::core::ffi::c_int
                && rightward == 0 as ::core::ffi::c_int)
    {
        (*screen).pending_scroll_downward += downward;
        (*screen).pending_scroll_rightward += rightward;
    } else {
        vterm_screen_flush_damage(screen);
        (*screen).pending_scrollrect = rect;
        (*screen).pending_scroll_downward = downward;
        (*screen).pending_scroll_rightward = rightward;
    }
    vterm_scroll_rect(
        rect,
        downward,
        rightward,
        Some(
            moverect_internal
                as unsafe extern "C" fn(
                    VTermRect,
                    VTermRect,
                    *mut ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        Some(
            erase_internal
                as unsafe extern "C" fn(
                    VTermRect,
                    ::core::ffi::c_int,
                    *mut ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        screen as *mut ::core::ffi::c_void,
    );
    if (*screen).damaged.start_row == -1 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    if rect_contains(&raw mut rect, &raw mut (*screen).damaged) != 0 {
        vterm_rect_move(&raw mut (*screen).damaged, -downward, -rightward);
        rect_clip(&raw mut (*screen).damaged, &raw mut rect);
    } else if rect.start_col <= (*screen).damaged.start_col
        && rect.end_col >= (*screen).damaged.end_col
        && rightward == 0 as ::core::ffi::c_int
    {
        if (*screen).damaged.start_row >= rect.start_row
            && (*screen).damaged.start_row < rect.end_row
        {
            (*screen).damaged.start_row -= downward;
            if (*screen).damaged.start_row < rect.start_row {
                (*screen).damaged.start_row = rect.start_row;
            }
            if (*screen).damaged.start_row > rect.end_row {
                (*screen).damaged.start_row = rect.end_row;
            }
        }
        if (*screen).damaged.end_row >= rect.start_row && (*screen).damaged.end_row < rect.end_row {
            (*screen).damaged.end_row -= downward;
            if (*screen).damaged.end_row < rect.start_row {
                (*screen).damaged.end_row = rect.start_row;
            }
            if (*screen).damaged.end_row > rect.end_row {
                (*screen).damaged.end_row = rect.end_row;
            }
        }
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn movecursor(
    mut pos: VTermPos,
    mut oldpos: VTermPos,
    mut visible: ::core::ffi::c_int,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut screen: *mut VTermScreen = user as *mut VTermScreen;
    if !(*screen).callbacks.is_null() && (*(*screen).callbacks).movecursor.is_some() {
        return Some(
            (*(*screen).callbacks)
                .movecursor
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(pos, oldpos, visible, (*screen).cbdata);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn setpenattr(
    mut attr: VTermAttr,
    mut val: *mut VTermValue,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut screen: *mut VTermScreen = user as *mut VTermScreen;
    match attr as ::core::ffi::c_uint {
        1 => {
            (*screen)
                .pen
                .set_bold((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
            return 1 as ::core::ffi::c_int;
        }
        2 => {
            (*screen)
                .pen
                .set_underline((*val).number as ::core::ffi::c_uint as ::core::ffi::c_uint);
            return 1 as ::core::ffi::c_int;
        }
        3 => {
            (*screen)
                .pen
                .set_italic((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
            return 1 as ::core::ffi::c_int;
        }
        4 => {
            (*screen)
                .pen
                .set_blink((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
            return 1 as ::core::ffi::c_int;
        }
        5 => {
            (*screen)
                .pen
                .set_reverse((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
            return 1 as ::core::ffi::c_int;
        }
        6 => {
            (*screen)
                .pen
                .set_conceal((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
            return 1 as ::core::ffi::c_int;
        }
        7 => {
            (*screen)
                .pen
                .set_strike((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
            return 1 as ::core::ffi::c_int;
        }
        8 => {
            (*screen)
                .pen
                .set_font((*val).number as ::core::ffi::c_uint as ::core::ffi::c_uint);
            return 1 as ::core::ffi::c_int;
        }
        9 => {
            (*screen).pen.fg = (*val).color;
            return 1 as ::core::ffi::c_int;
        }
        10 => {
            (*screen).pen.bg = (*val).color;
            return 1 as ::core::ffi::c_int;
        }
        11 => {
            (*screen)
                .pen
                .set_small((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
            return 1 as ::core::ffi::c_int;
        }
        12 => {
            (*screen)
                .pen
                .set_baseline((*val).number as ::core::ffi::c_uint as ::core::ffi::c_uint);
            return 1 as ::core::ffi::c_int;
        }
        13 => {
            (*screen).pen.uri = (*val).number;
            return 1 as ::core::ffi::c_int;
        }
        14 => {
            (*screen)
                .pen
                .set_dim((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
            return 1 as ::core::ffi::c_int;
        }
        15 => {
            (*screen)
                .pen
                .set_overline((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
            return 1 as ::core::ffi::c_int;
        }
        16 => return 0 as ::core::ffi::c_int,
        _ => {}
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn settermprop(
    mut prop: VTermProp,
    mut val: *mut VTermValue,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut screen: *mut VTermScreen = user as *mut VTermScreen;
    match prop as ::core::ffi::c_uint {
        3 => {
            if (*val).boolean != 0 && (*screen).buffers[BUFIDX_ALTSCREEN as usize].is_null() {
                return 0 as ::core::ffi::c_int;
            }
            (*screen).buffer = if (*val).boolean != 0 {
                (*screen).buffers[BUFIDX_ALTSCREEN as usize]
            } else {
                (*screen).buffers[BUFIDX_PRIMARY as usize]
            };
            if (*val).boolean == 0 {
                damagescreen(screen);
            }
        }
        6 => {
            (*screen)
                .set_global_reverse((*val).boolean as ::core::ffi::c_uint as ::core::ffi::c_uint);
            damagescreen(screen);
        }
        _ => {}
    }
    if !(*screen).callbacks.is_null() && (*(*screen).callbacks).settermprop.is_some() {
        return Some(
            (*(*screen).callbacks)
                .settermprop
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(prop, val, (*screen).cbdata);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn bell(mut user: *mut ::core::ffi::c_void) -> ::core::ffi::c_int {
    let mut screen: *mut VTermScreen = user as *mut VTermScreen;
    if !(*screen).callbacks.is_null() && (*(*screen).callbacks).bell.is_some() {
        return Some(
            (*(*screen).callbacks)
                .bell
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")((*screen).cbdata);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn line_popcount(
    mut buffer: *mut ScreenCell,
    mut row: ::core::ffi::c_int,
    mut _rows: ::core::ffi::c_int,
    mut cols: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut col: ::core::ffi::c_int = cols - 1 as ::core::ffi::c_int;
    while col >= 0 as ::core::ffi::c_int
        && (*buffer.offset((row * cols + col) as isize)).schar == 0 as schar_T
    {
        col -= 1;
    }
    return col + 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn resize_buffer(
    mut screen: *mut VTermScreen,
    mut bufidx: ::core::ffi::c_int,
    mut new_rows: ::core::ffi::c_int,
    mut new_cols: ::core::ffi::c_int,
    mut active: bool,
    mut statefields: *mut VTermStateFields,
) {
    let mut old_rows: ::core::ffi::c_int = (*screen).rows;
    let mut old_cols: ::core::ffi::c_int = (*screen).cols;
    let mut old_buffer: *mut ScreenCell = (*screen).buffers[bufidx as usize];
    let mut old_lineinfo: *mut VTermLineInfo = (*statefields).lineinfos[bufidx as usize];
    let mut new_buffer: *mut ScreenCell = vterm_allocator_malloc(
        (*screen).vt,
        ::core::mem::size_of::<ScreenCell>()
            .wrapping_mul(new_rows as size_t)
            .wrapping_mul(new_cols as size_t),
    ) as *mut ScreenCell;
    let mut new_lineinfo: *mut VTermLineInfo = vterm_allocator_malloc(
        (*screen).vt,
        ::core::mem::size_of::<VTermLineInfo>().wrapping_mul(new_rows as size_t),
    ) as *mut VTermLineInfo;
    let mut old_row: ::core::ffi::c_int = old_rows - 1 as ::core::ffi::c_int;
    let mut new_row: ::core::ffi::c_int = new_rows - 1 as ::core::ffi::c_int;
    let mut old_cursor: VTermPos = (*statefields).pos;
    let mut new_cursor: VTermPos = VTermPos {
        row: -1 as ::core::ffi::c_int,
        col: -1 as ::core::ffi::c_int,
    };
    let mut final_blank_row: ::core::ffi::c_int = new_rows;
    let mut do_reflow: bool =
        (*screen).reflow() as ::core::ffi::c_int != 0 && bufidx == BUFIDX_PRIMARY;
    while old_row >= 0 as ::core::ffi::c_int {
        let mut old_row_end: ::core::ffi::c_int = old_row;
        while do_reflow as ::core::ffi::c_int != 0
            && !old_lineinfo.is_null()
            && old_row > 0 as ::core::ffi::c_int
            && (*old_lineinfo.offset(old_row as isize)).continuation() as ::core::ffi::c_int != 0
        {
            old_row -= 1;
        }
        let mut old_row_start: ::core::ffi::c_int = old_row;
        let mut width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut row: ::core::ffi::c_int = old_row_start;
        while row <= old_row_end {
            if do_reflow as ::core::ffi::c_int != 0
                && row < old_rows - 1 as ::core::ffi::c_int
                && (*old_lineinfo.offset((row + 1 as ::core::ffi::c_int) as isize)).continuation()
                    as ::core::ffi::c_int
                    != 0
            {
                width += old_cols;
            } else {
                width += line_popcount(old_buffer, row, old_rows, old_cols);
            }
            row += 1;
        }
        if final_blank_row == new_row + 1 as ::core::ffi::c_int && width == 0 as ::core::ffi::c_int
        {
            final_blank_row = new_row;
        }
        let mut new_height: ::core::ffi::c_int = if do_reflow as ::core::ffi::c_int != 0 {
            if width != 0 {
                (width + new_cols - 1 as ::core::ffi::c_int) / new_cols
            } else {
                1 as ::core::ffi::c_int
            }
        } else {
            1 as ::core::ffi::c_int
        };
        let mut new_row_end: ::core::ffi::c_int = new_row;
        let mut new_row_start: ::core::ffi::c_int = new_row - new_height + 1 as ::core::ffi::c_int;
        old_row = old_row_start;
        let mut old_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut spare_rows: ::core::ffi::c_int = new_rows - final_blank_row;
        if new_row_start < 0 as ::core::ffi::c_int
            && spare_rows >= 0 as ::core::ffi::c_int
            && (!active
                || new_cursor.row == -1 as ::core::ffi::c_int
                || new_cursor.row - new_row_start < new_rows)
        {
            let mut downwards: ::core::ffi::c_int = -new_row_start;
            if downwards > spare_rows {
                downwards = spare_rows;
            }
            let mut rowcount: ::core::ffi::c_int = new_rows - downwards;
            memmove(
                new_buffer.offset((downwards * new_cols) as isize) as *mut ::core::ffi::c_void,
                new_buffer.offset(0 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                (rowcount as size_t)
                    .wrapping_mul(new_cols as size_t)
                    .wrapping_mul(::core::mem::size_of::<ScreenCell>()),
            );
            memmove(
                new_lineinfo.offset(downwards as isize) as *mut ::core::ffi::c_void,
                new_lineinfo.offset(0 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                (rowcount as size_t).wrapping_mul(::core::mem::size_of::<VTermLineInfo>()),
            );
            new_row += downwards;
            new_row_start += downwards;
            new_row_end += downwards;
            if new_cursor.row >= 0 as ::core::ffi::c_int {
                new_cursor.row += downwards;
            }
            final_blank_row += downwards;
        }
        if new_row_start < 0 as ::core::ffi::c_int {
            if old_row_start <= old_cursor.row && old_cursor.row <= old_row_end {
                new_cursor.row = 0 as ::core::ffi::c_int;
                new_cursor.col = old_cursor.col;
                if new_cursor.col >= new_cols {
                    new_cursor.col = new_cols - 1 as ::core::ffi::c_int;
                }
            }
            break;
        } else {
            new_row = new_row_start;
            old_row = old_row_start;
            while new_row <= new_row_end {
                let mut count: ::core::ffi::c_int =
                    if width >= new_cols { new_cols } else { width };
                width -= count;
                let mut new_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while count != 0 {
                    *new_buffer.offset((new_row * new_cols + new_col) as isize) =
                        *old_buffer.offset((old_row * old_cols + old_col) as isize);
                    if old_cursor.row == old_row && old_cursor.col == old_col {
                        new_cursor.row = new_row;
                        new_cursor.col = new_col;
                    }
                    old_col += 1;
                    if old_col == old_cols {
                        old_row += 1;
                        if !do_reflow {
                            new_col += 1;
                            break;
                        } else {
                            old_col = 0 as ::core::ffi::c_int;
                        }
                    }
                    new_col += 1;
                    count -= 1;
                }
                if old_cursor.row == old_row && old_cursor.col >= old_col {
                    new_cursor.row = new_row;
                    new_cursor.col = old_cursor.col - old_col + new_col;
                    if new_cursor.col >= new_cols {
                        new_cursor.col = new_cols - 1 as ::core::ffi::c_int;
                    }
                }
                while new_col < new_cols {
                    clearcell(
                        screen,
                        new_buffer.offset((new_row * new_cols + new_col) as isize),
                    );
                    new_col += 1;
                }
                (*new_lineinfo.offset(new_row as isize)).set_continuation(
                    (new_row > new_row_start) as ::core::ffi::c_int as ::core::ffi::c_uint
                        as ::core::ffi::c_uint,
                );
                new_row += 1;
            }
            old_row = old_row_start - 1 as ::core::ffi::c_int;
            new_row = new_row_start - 1 as ::core::ffi::c_int;
        }
    }
    if old_cursor.row <= old_row {
        new_cursor.row = 0 as ::core::ffi::c_int;
        new_cursor.col = old_cursor.col;
        if new_cursor.col >= new_cols {
            new_cursor.col = new_cols - 1 as ::core::ffi::c_int;
        }
    }
    if active as ::core::ffi::c_int != 0
        && (new_cursor.row == -1 as ::core::ffi::c_int
            || new_cursor.col == -1 as ::core::ffi::c_int)
    {
        fprintf(
            stderr,
            b"screen_resize failed to update cursor position\n\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
        abort();
    }
    if old_row >= 0 as ::core::ffi::c_int && bufidx == BUFIDX_PRIMARY {
        if !(*screen).callbacks.is_null() && (*(*screen).callbacks).sb_pushline.is_some() {
            let mut row_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while row_0 <= old_row {
                sb_pushline_from_row(screen, row_0);
                row_0 += 1;
            }
        }
        if active {
            (*statefields).pos.row -= old_row + 1 as ::core::ffi::c_int;
        }
    }
    if new_row >= 0 as ::core::ffi::c_int
        && bufidx == BUFIDX_PRIMARY
        && !(*screen).callbacks.is_null()
        && (*(*screen).callbacks).sb_popline.is_some()
    {
        while new_row >= 0 as ::core::ffi::c_int {
            if (*(*screen).callbacks)
                .sb_popline
                .expect("non-null function pointer")(
                old_cols,
                (*screen).sb_buffer,
                (*screen).cbdata,
            ) == 0
            {
                break;
            }
            let mut pos: VTermPos = VTermPos {
                row: new_row,
                col: 0,
            };
            pos.col = 0 as ::core::ffi::c_int;
            while pos.col < old_cols && pos.col < new_cols {
                let mut src: *mut VTermScreenCell = (*screen).sb_buffer.offset(pos.col as isize);
                let mut dst: *mut ScreenCell =
                    new_buffer.offset((pos.row * new_cols + pos.col) as isize);
                (*dst).schar = (*src).schar;
                (*dst)
                    .pen
                    .set_bold((*src).attrs.bold() as ::core::ffi::c_uint);
                (*dst)
                    .pen
                    .set_underline((*src).attrs.underline() as ::core::ffi::c_uint);
                (*dst)
                    .pen
                    .set_italic((*src).attrs.italic() as ::core::ffi::c_uint);
                (*dst)
                    .pen
                    .set_blink((*src).attrs.blink() as ::core::ffi::c_uint);
                (*dst).pen.set_reverse(
                    ((*src).attrs.reverse() as ::core::ffi::c_int
                        ^ (*screen).global_reverse() as ::core::ffi::c_int)
                        as ::core::ffi::c_uint as ::core::ffi::c_uint,
                );
                (*dst)
                    .pen
                    .set_conceal((*src).attrs.conceal() as ::core::ffi::c_uint);
                (*dst)
                    .pen
                    .set_strike((*src).attrs.strike() as ::core::ffi::c_uint);
                (*dst)
                    .pen
                    .set_font((*src).attrs.font() as ::core::ffi::c_uint);
                (*dst)
                    .pen
                    .set_small((*src).attrs.small() as ::core::ffi::c_uint);
                (*dst)
                    .pen
                    .set_baseline((*src).attrs.baseline() as ::core::ffi::c_uint);
                (*dst)
                    .pen
                    .set_dim((*src).attrs.dim() as ::core::ffi::c_uint);
                (*dst)
                    .pen
                    .set_overline((*src).attrs.overline() as ::core::ffi::c_uint);
                (*dst).pen.fg = (*src).fg;
                (*dst).pen.bg = (*src).bg;
                (*dst).pen.uri = (*src).uri;
                if (*src).width as ::core::ffi::c_int == 2 as ::core::ffi::c_int
                    && pos.col < new_cols - 1 as ::core::ffi::c_int
                {
                    (*dst.offset(1 as ::core::ffi::c_int as isize)).schar =
                        -1 as ::core::ffi::c_int as uint32_t as schar_T;
                }
                pos.col +=
                    (*(*screen).sb_buffer.offset(pos.col as isize)).width as ::core::ffi::c_int;
            }
            while pos.col < new_cols {
                clearcell(
                    screen,
                    new_buffer.offset((pos.row * new_cols + pos.col) as isize),
                );
                pos.col += 1;
            }
            new_row -= 1;
            if active {
                (*statefields).pos.row += 1;
            }
        }
    }
    if new_row >= 0 as ::core::ffi::c_int {
        let mut moverows: ::core::ffi::c_int = new_rows - new_row - 1 as ::core::ffi::c_int;
        memmove(
            new_buffer.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            new_buffer.offset(((new_row + 1 as ::core::ffi::c_int) * new_cols) as isize)
                as *const ::core::ffi::c_void,
            (moverows as size_t)
                .wrapping_mul(new_cols as size_t)
                .wrapping_mul(::core::mem::size_of::<ScreenCell>()),
        );
        memmove(
            new_lineinfo.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            new_lineinfo.offset((new_row + 1 as ::core::ffi::c_int) as isize)
                as *const ::core::ffi::c_void,
            (moverows as size_t).wrapping_mul(::core::mem::size_of::<VTermLineInfo>()),
        );
        new_cursor.row -= new_row + 1 as ::core::ffi::c_int;
        new_row = moverows;
        while new_row < new_rows {
            let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while col < new_cols {
                clearcell(
                    screen,
                    new_buffer.offset((new_row * new_cols + col) as isize),
                );
                col += 1;
            }
            *new_lineinfo.offset(new_row as isize) = {
                let mut init = VTermLineInfo {
                    doublewidth_doubleheight_continuation: [0; 1],
                    c2rust_padding: [0; 3],
                };
                init.set_doublewidth(0 as ::core::ffi::c_uint);
                init.set_doubleheight(0);
                init.set_continuation(0);
                init
            };
            new_row += 1;
        }
    }
    vterm_allocator_free((*screen).vt, old_buffer as *mut ::core::ffi::c_void);
    (*screen).buffers[bufidx as usize] = new_buffer;
    vterm_allocator_free((*screen).vt, old_lineinfo as *mut ::core::ffi::c_void);
    (*statefields).lineinfos[bufidx as usize] = new_lineinfo;
    if active {
        (*statefields).pos = new_cursor;
    }
}
unsafe extern "C" fn resize(
    mut new_rows: ::core::ffi::c_int,
    mut new_cols: ::core::ffi::c_int,
    mut fields: *mut VTermStateFields,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut screen: *mut VTermScreen = user as *mut VTermScreen;
    let mut altscreen_active: ::core::ffi::c_int = (!(*screen).buffers[BUFIDX_ALTSCREEN as usize]
        .is_null()
        && (*screen).buffer == (*screen).buffers[BUFIDX_ALTSCREEN as usize])
        as ::core::ffi::c_int;
    let mut old_rows: ::core::ffi::c_int = (*screen).rows;
    let mut old_cols: ::core::ffi::c_int = (*screen).cols;
    if new_cols > old_cols {
        if !(*screen).sb_buffer.is_null() {
            vterm_allocator_free(
                (*screen).vt,
                (*screen).sb_buffer as *mut ::core::ffi::c_void,
            );
        }
        (*screen).sb_buffer = vterm_allocator_malloc(
            (*screen).vt,
            ::core::mem::size_of::<VTermScreenCell>().wrapping_mul(new_cols as size_t),
        ) as *mut VTermScreenCell;
    }
    resize_buffer(
        screen,
        0 as ::core::ffi::c_int,
        new_rows,
        new_cols,
        altscreen_active == 0,
        fields,
    );
    if !(*screen).buffers[BUFIDX_ALTSCREEN as usize].is_null() {
        resize_buffer(
            screen,
            1 as ::core::ffi::c_int,
            new_rows,
            new_cols,
            altscreen_active != 0,
            fields,
        );
    } else if new_rows != old_rows {
        vterm_allocator_free(
            (*screen).vt,
            (*fields).lineinfos[BUFIDX_ALTSCREEN as usize] as *mut ::core::ffi::c_void,
        );
        let mut new_lineinfo: *mut VTermLineInfo = vterm_allocator_malloc(
            (*screen).vt,
            ::core::mem::size_of::<VTermLineInfo>().wrapping_mul(new_rows as size_t),
        ) as *mut VTermLineInfo;
        let mut row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while row < new_rows {
            *new_lineinfo.offset(row as isize) = {
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
        (*fields).lineinfos[BUFIDX_ALTSCREEN as usize] = new_lineinfo;
    }
    (*screen).buffer = if altscreen_active != 0 {
        (*screen).buffers[BUFIDX_ALTSCREEN as usize]
    } else {
        (*screen).buffers[BUFIDX_PRIMARY as usize]
    };
    (*screen).rows = new_rows;
    (*screen).cols = new_cols;
    if new_cols <= old_cols {
        if !(*screen).sb_buffer.is_null() {
            vterm_allocator_free(
                (*screen).vt,
                (*screen).sb_buffer as *mut ::core::ffi::c_void,
            );
        }
        (*screen).sb_buffer = vterm_allocator_malloc(
            (*screen).vt,
            ::core::mem::size_of::<VTermScreenCell>().wrapping_mul(new_cols as size_t),
        ) as *mut VTermScreenCell;
    }
    damagescreen(screen);
    if !(*screen).callbacks.is_null() && (*(*screen).callbacks).resize.is_some() {
        return Some(
            (*(*screen).callbacks)
                .resize
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(new_rows, new_cols, (*screen).cbdata);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn theme(
    mut dark: *mut bool,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut screen: *mut VTermScreen = user as *mut VTermScreen;
    if !(*screen).callbacks.is_null() && (*(*screen).callbacks).theme.is_some() {
        return Some(
            (*(*screen).callbacks)
                .theme
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(dark, (*screen).cbdata);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn setlineinfo(
    mut row: ::core::ffi::c_int,
    mut newinfo: *const VTermLineInfo,
    mut oldinfo: *const VTermLineInfo,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut screen: *mut VTermScreen = user as *mut VTermScreen;
    if (*newinfo).doublewidth() as ::core::ffi::c_int
        != (*oldinfo).doublewidth() as ::core::ffi::c_int
        || (*newinfo).doubleheight() as ::core::ffi::c_int
            != (*oldinfo).doubleheight() as ::core::ffi::c_int
    {
        let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while col < (*screen).cols {
            let mut cell: *mut ScreenCell = getcell(screen, row, col);
            (*cell)
                .pen
                .set_dwl((*newinfo).doublewidth() as ::core::ffi::c_uint);
            (*cell)
                .pen
                .set_dhl((*newinfo).doubleheight() as ::core::ffi::c_uint);
            col += 1;
        }
        let mut rect: VTermRect = VTermRect {
            start_row: row,
            end_row: row + 1 as ::core::ffi::c_int,
            start_col: 0 as ::core::ffi::c_int,
            end_col: if (*newinfo).doublewidth() as ::core::ffi::c_int != 0 {
                (*screen).cols / 2 as ::core::ffi::c_int
            } else {
                (*screen).cols
            },
        };
        damagerect(screen, rect);
        if (*newinfo).doublewidth() != 0 {
            rect.start_col = (*screen).cols / 2 as ::core::ffi::c_int;
            rect.end_col = (*screen).cols;
            erase_internal(rect, 0 as ::core::ffi::c_int, user);
        }
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn sb_clear(mut user: *mut ::core::ffi::c_void) -> ::core::ffi::c_int {
    let mut screen: *mut VTermScreen = user as *mut VTermScreen;
    if !(*screen).callbacks.is_null() && (*(*screen).callbacks).sb_clear.is_some() {
        if Some(
            (*(*screen).callbacks)
                .sb_clear
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")((*screen).cbdata)
            != 0
        {
            return 1 as ::core::ffi::c_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
static state_cbs: GlobalCell<VTermStateCallbacks> = GlobalCell::new(VTermStateCallbacks {
    putglyph: Some(
        putglyph
            as unsafe extern "C" fn(
                *mut VTermGlyphInfo,
                VTermPos,
                *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
    ),
    movecursor: Some(
        movecursor
            as unsafe extern "C" fn(
                VTermPos,
                VTermPos,
                ::core::ffi::c_int,
                *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
    ),
    scrollrect: Some(
        scrollrect
            as unsafe extern "C" fn(
                VTermRect,
                ::core::ffi::c_int,
                ::core::ffi::c_int,
                *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
    ),
    moverect: None,
    erase: Some(
        erase
            as unsafe extern "C" fn(
                VTermRect,
                ::core::ffi::c_int,
                *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
    ),
    initpen: None,
    setpenattr: Some(
        setpenattr
            as unsafe extern "C" fn(
                VTermAttr,
                *mut VTermValue,
                *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
    ),
    settermprop: Some(
        settermprop
            as unsafe extern "C" fn(
                VTermProp,
                *mut VTermValue,
                *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
    ),
    bell: Some(bell as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ::core::ffi::c_int),
    resize: Some(
        resize
            as unsafe extern "C" fn(
                ::core::ffi::c_int,
                ::core::ffi::c_int,
                *mut VTermStateFields,
                *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
    ),
    theme: Some(
        theme as unsafe extern "C" fn(*mut bool, *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    ),
    setlineinfo: Some(
        setlineinfo
            as unsafe extern "C" fn(
                ::core::ffi::c_int,
                *const VTermLineInfo,
                *const VTermLineInfo,
                *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
    ),
    sb_clear: Some(
        sb_clear as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    ),
});
unsafe extern "C" fn screen_new(mut vt: *mut VTerm) -> *mut VTermScreen {
    let mut state: *mut VTermState = vterm_obtain_state(vt);
    if state.is_null() {
        return ::core::ptr::null_mut::<VTermScreen>();
    }
    let mut screen: *mut VTermScreen =
        vterm_allocator_malloc(vt, ::core::mem::size_of::<VTermScreen>()) as *mut VTermScreen;
    let mut rows: ::core::ffi::c_int = 0;
    let mut cols: ::core::ffi::c_int = 0;
    vterm_get_size(vt, &raw mut rows, &raw mut cols);
    (*screen).vt = vt;
    (*screen).state = state;
    (*screen).damage_merge = VTERM_DAMAGE_CELL;
    (*screen).damaged.start_row = -1 as ::core::ffi::c_int;
    (*screen).pending_scrollrect.start_row = -1 as ::core::ffi::c_int;
    (*screen).rows = rows;
    (*screen).cols = cols;
    (*screen).set_global_reverse(false_0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    (*screen).set_reflow(false_0 as ::core::ffi::c_uint as ::core::ffi::c_uint);
    (*screen).callbacks = ::core::ptr::null::<VTermScreenCallbacks>();
    (*screen).cbdata = NULL;
    (*screen).buffers[BUFIDX_PRIMARY as usize] = alloc_buffer(screen, rows, cols);
    (*screen).buffer = (*screen).buffers[BUFIDX_PRIMARY as usize];
    (*screen).sb_buffer = vterm_allocator_malloc(
        (*screen).vt,
        ::core::mem::size_of::<VTermScreenCell>().wrapping_mul(cols as size_t),
    ) as *mut VTermScreenCell;
    vterm_state_set_callbacks(
        (*screen).state,
        state_cbs.ptr(),
        screen as *mut ::core::ffi::c_void,
    );
    return screen;
}
#[no_mangle]
pub unsafe extern "C" fn vterm_screen_free(mut screen: *mut VTermScreen) {
    vterm_allocator_free(
        (*screen).vt,
        (*screen).buffers[BUFIDX_PRIMARY as usize] as *mut ::core::ffi::c_void,
    );
    if !(*screen).buffers[BUFIDX_ALTSCREEN as usize].is_null() {
        vterm_allocator_free(
            (*screen).vt,
            (*screen).buffers[BUFIDX_ALTSCREEN as usize] as *mut ::core::ffi::c_void,
        );
    }
    vterm_allocator_free(
        (*screen).vt,
        (*screen).sb_buffer as *mut ::core::ffi::c_void,
    );
    vterm_allocator_free((*screen).vt, screen as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn vterm_screen_reset(
    mut screen: *mut VTermScreen,
    mut hard: ::core::ffi::c_int,
) {
    (*screen).damaged.start_row = -1 as ::core::ffi::c_int;
    (*screen).pending_scrollrect.start_row = -1 as ::core::ffi::c_int;
    vterm_state_reset((*screen).state, hard);
    vterm_screen_flush_damage(screen);
}
#[no_mangle]
pub unsafe extern "C" fn vterm_screen_get_cell(
    mut screen: *const VTermScreen,
    mut pos: VTermPos,
    mut cell: *mut VTermScreenCell,
) -> ::core::ffi::c_int {
    let mut intcell: *mut ScreenCell = getcell(screen, pos.row, pos.col);
    if intcell.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    (*cell).schar = if (*intcell).schar == -1 as ::core::ffi::c_int as uint32_t {
        0 as schar_T
    } else {
        (*intcell).schar
    };
    (*cell)
        .attrs
        .set_bold((*intcell).pen.bold() as ::core::ffi::c_uint);
    (*cell)
        .attrs
        .set_underline((*intcell).pen.underline() as ::core::ffi::c_uint);
    (*cell)
        .attrs
        .set_italic((*intcell).pen.italic() as ::core::ffi::c_uint);
    (*cell)
        .attrs
        .set_blink((*intcell).pen.blink() as ::core::ffi::c_uint);
    (*cell).attrs.set_reverse(
        ((*intcell).pen.reverse() as ::core::ffi::c_int
            ^ (*screen).global_reverse() as ::core::ffi::c_int) as ::core::ffi::c_uint
            as ::core::ffi::c_uint,
    );
    (*cell)
        .attrs
        .set_conceal((*intcell).pen.conceal() as ::core::ffi::c_uint);
    (*cell)
        .attrs
        .set_strike((*intcell).pen.strike() as ::core::ffi::c_uint);
    (*cell)
        .attrs
        .set_font((*intcell).pen.font() as ::core::ffi::c_uint);
    (*cell)
        .attrs
        .set_small((*intcell).pen.small() as ::core::ffi::c_uint);
    (*cell)
        .attrs
        .set_baseline((*intcell).pen.baseline() as ::core::ffi::c_uint);
    (*cell)
        .attrs
        .set_dim((*intcell).pen.dim() as ::core::ffi::c_uint);
    (*cell)
        .attrs
        .set_overline((*intcell).pen.overline() as ::core::ffi::c_uint);
    (*cell)
        .attrs
        .set_dwl((*intcell).pen.dwl() as ::core::ffi::c_uint);
    (*cell)
        .attrs
        .set_dhl((*intcell).pen.dhl() as ::core::ffi::c_uint);
    (*cell).fg = (*intcell).pen.fg;
    (*cell).bg = (*intcell).pen.bg;
    (*cell).uri = (*intcell).pen.uri;
    if pos.col < (*screen).cols - 1 as ::core::ffi::c_int
        && (*getcell(screen, pos.row, pos.col + 1 as ::core::ffi::c_int)).schar
            == -1 as ::core::ffi::c_int as uint32_t
    {
        (*cell).width = 2 as ::core::ffi::c_char;
    } else {
        (*cell).width = 1 as ::core::ffi::c_char;
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn vterm_obtain_screen(mut vt: *mut VTerm) -> *mut VTermScreen {
    if !(*vt).screen.is_null() {
        return (*vt).screen;
    }
    let mut screen: *mut VTermScreen = screen_new(vt);
    (*vt).screen = screen;
    return screen;
}
#[no_mangle]
pub unsafe extern "C" fn vterm_screen_enable_reflow(
    mut screen: *mut VTermScreen,
    mut reflow: bool,
) {
    (*screen).set_reflow(reflow as ::core::ffi::c_uint as ::core::ffi::c_uint);
}
#[no_mangle]
pub unsafe extern "C" fn vterm_screen_set_reflow(mut screen: *mut VTermScreen, mut reflow: bool) {
    vterm_screen_enable_reflow(screen, reflow);
}
#[no_mangle]
pub unsafe extern "C" fn vterm_screen_enable_altscreen(
    mut screen: *mut VTermScreen,
    mut altscreen: ::core::ffi::c_int,
) {
    if (*screen).buffers[BUFIDX_ALTSCREEN as usize].is_null() && altscreen != 0 {
        let mut rows: ::core::ffi::c_int = 0;
        let mut cols: ::core::ffi::c_int = 0;
        vterm_get_size((*screen).vt, &raw mut rows, &raw mut cols);
        (*screen).buffers[BUFIDX_ALTSCREEN as usize] = alloc_buffer(screen, rows, cols);
    }
}
#[no_mangle]
pub unsafe extern "C" fn vterm_screen_set_callbacks(
    mut screen: *mut VTermScreen,
    mut callbacks: *const VTermScreenCallbacks,
    mut user: *mut ::core::ffi::c_void,
) {
    (*screen).callbacks = callbacks;
    (*screen).cbdata = user;
}
#[no_mangle]
pub unsafe extern "C" fn vterm_screen_set_unrecognised_fallbacks(
    mut screen: *mut VTermScreen,
    mut fallbacks: *const VTermStateFallbacks,
    mut user: *mut ::core::ffi::c_void,
) {
    vterm_state_set_unrecognised_fallbacks((*screen).state, fallbacks, user);
}
#[no_mangle]
pub unsafe extern "C" fn vterm_screen_flush_damage(mut screen: *mut VTermScreen) {
    if (*screen).pending_scrollrect.start_row != -1 as ::core::ffi::c_int {
        vterm_scroll_rect(
            (*screen).pending_scrollrect,
            (*screen).pending_scroll_downward,
            (*screen).pending_scroll_rightward,
            Some(
                moverect_user
                    as unsafe extern "C" fn(
                        VTermRect,
                        VTermRect,
                        *mut ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
            Some(
                erase_user
                    as unsafe extern "C" fn(
                        VTermRect,
                        ::core::ffi::c_int,
                        *mut ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
            screen as *mut ::core::ffi::c_void,
        );
        (*screen).pending_scrollrect.start_row = -1 as ::core::ffi::c_int;
    }
    if (*screen).damaged.start_row != -1 as ::core::ffi::c_int {
        if !(*screen).callbacks.is_null() && (*(*screen).callbacks).damage.is_some() {
            Some(
                (*(*screen).callbacks)
                    .damage
                    .expect("non-null function pointer"),
            )
            .expect("non-null function pointer")((*screen).damaged, (*screen).cbdata);
        }
        (*screen).damaged.start_row = -1 as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn vterm_screen_set_damage_merge(
    mut screen: *mut VTermScreen,
    mut size: VTermDamageSize,
) {
    vterm_screen_flush_damage(screen);
    (*screen).damage_merge = size;
}
#[no_mangle]
pub unsafe extern "C" fn vterm_screen_convert_color_to_rgb(
    mut screen: *const VTermScreen,
    mut col: *mut VTermColor,
) {
    vterm_state_convert_color_to_rgb((*screen).state, col);
}
#[no_mangle]
pub unsafe extern "C" fn rect_expand(mut dst: *mut VTermRect, mut src: *mut VTermRect) {
    if (*dst).start_row > (*src).start_row {
        (*dst).start_row = (*src).start_row;
    }
    if (*dst).start_col > (*src).start_col {
        (*dst).start_col = (*src).start_col;
    }
    if (*dst).end_row < (*src).end_row {
        (*dst).end_row = (*src).end_row;
    }
    if (*dst).end_col < (*src).end_col {
        (*dst).end_col = (*src).end_col;
    }
}
#[no_mangle]
pub unsafe extern "C" fn rect_clip(mut dst: *mut VTermRect, mut bounds: *mut VTermRect) {
    if (*dst).start_row < (*bounds).start_row {
        (*dst).start_row = (*bounds).start_row;
    }
    if (*dst).start_col < (*bounds).start_col {
        (*dst).start_col = (*bounds).start_col;
    }
    if (*dst).end_row > (*bounds).end_row {
        (*dst).end_row = (*bounds).end_row;
    }
    if (*dst).end_col > (*bounds).end_col {
        (*dst).end_col = (*bounds).end_col;
    }
    if (*dst).end_row < (*dst).start_row {
        (*dst).end_row = (*dst).start_row;
    }
    if (*dst).end_col < (*dst).start_col {
        (*dst).end_col = (*dst).start_col;
    }
}
#[no_mangle]
pub unsafe extern "C" fn rect_equal(
    mut a: *mut VTermRect,
    mut b: *mut VTermRect,
) -> ::core::ffi::c_int {
    return ((*a).start_row == (*b).start_row
        && (*a).start_col == (*b).start_col
        && (*a).end_row == (*b).end_row
        && (*a).end_col == (*b).end_col) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn rect_contains(
    mut big: *mut VTermRect,
    mut small: *mut VTermRect,
) -> ::core::ffi::c_int {
    if (*small).start_row < (*big).start_row {
        return 0 as ::core::ffi::c_int;
    }
    if (*small).start_col < (*big).start_col {
        return 0 as ::core::ffi::c_int;
    }
    if (*small).end_row > (*big).end_row {
        return 0 as ::core::ffi::c_int;
    }
    if (*small).end_col > (*big).end_col {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn rect_intersects(
    mut a: *mut VTermRect,
    mut b: *mut VTermRect,
) -> ::core::ffi::c_int {
    if (*a).start_row > (*b).end_row || (*b).start_row > (*a).end_row {
        return 0 as ::core::ffi::c_int;
    }
    if (*a).start_col > (*b).end_col || (*b).start_col > (*a).end_col {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
