use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::memory::{xfree, xmalloc};
use crate::src::nvim::os::libc::{abs, memcpy, memset, snprintf, vsnprintf};
pub use crate::src::nvim::types::{
    GraphemeState, ScreenCell, ScreenPen, VTerm, VTermAllocatorFunctions, VTermAttr, VTermColor,
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
    __builtin_va_list, __gnuc_va_list, __va_list_tag, int32_t, schar_T, size_t, uint16_t, uint32_t,
    uint8_t, utf8proc_int32_t, va_list,
};
use crate::src::nvim::vterm::screen::vterm_screen_free;
use crate::src::nvim::vterm::state::vterm_state_free;

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
pub struct VTermBuilder {
    pub ver: ::core::ffi::c_int,
    pub rows: ::core::ffi::c_int,
    pub cols: ::core::ffi::c_int,
    pub allocator: *const VTermAllocatorFunctions,
    pub allocdata: *mut ::core::ffi::c_void,
    pub outbuffer_len: size_t,
    pub tmpbuffer_len: size_t,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
unsafe extern "C" fn default_malloc(
    mut size: size_t,
    mut _allocdata: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_void {
    let mut ptr: *mut ::core::ffi::c_void = xmalloc(size);
    if !ptr.is_null() {
        memset(ptr, 0 as ::core::ffi::c_int, size);
    }
    return ptr;
}
unsafe extern "C" fn default_free(
    mut ptr: *mut ::core::ffi::c_void,
    mut _allocdata: *mut ::core::ffi::c_void,
) {
    xfree(ptr);
}
static default_allocator: GlobalCell<VTermAllocatorFunctions> =
    GlobalCell::new(VTermAllocatorFunctions {
        malloc: Some(
            default_malloc
                as unsafe extern "C" fn(
                    size_t,
                    *mut ::core::ffi::c_void,
                ) -> *mut ::core::ffi::c_void,
        ),
        free: Some(
            default_free
                as unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut ::core::ffi::c_void) -> (),
        ),
    });
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
    let mut allocator: *const VTermAllocatorFunctions = if !(*builder).allocator.is_null() {
        (*builder).allocator
    } else {
        default_allocator.ptr() as *const VTermAllocatorFunctions
    };
    let mut vt: *mut VTerm = Some((*allocator).malloc.expect("non-null function pointer"))
        .expect("non-null function pointer")(
        ::core::mem::size_of::<VTerm>(), (*builder).allocdata
    ) as *mut VTerm;
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
    (*vt).outbuffer = vterm_allocator_malloc(vt, (*vt).outbuffer_len) as *mut ::core::ffi::c_char;
    (*vt).tmpbuffer_len = if (*builder).tmpbuffer_len != 0 {
        (*builder).tmpbuffer_len
    } else {
        4096 as size_t
    };
    (*vt).tmpbuffer = vterm_allocator_malloc(vt, (*vt).tmpbuffer_len) as *mut ::core::ffi::c_char;
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
    return Some(
        (*(*vt).allocator)
            .malloc
            .expect("non-null function pointer"),
    )
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
        Some(
            (*(*vt).parser.callbacks)
                .resize
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(rows, cols, (*vt).parser.cbdata);
    }
}
#[no_mangle]
pub unsafe extern "C" fn vterm_set_utf8(mut vt: *mut VTerm, mut is_utf8: ::core::ffi::c_int) {
    (*vt)
        .mode
        .set_utf8(is_utf8 as ::core::ffi::c_uint as ::core::ffi::c_uint);
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
    if ctrl as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int && (*vt).mode.ctrl8bit() == 0 {
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
    cur = cur.wrapping_add(vsnprintf(
        (*vt).tmpbuffer.offset(cur as isize),
        (*vt).tmpbuffer_len.wrapping_sub(cur),
        fmt,
        args.as_va_list(),
    ) as size_t);
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
        if ctrl as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int && (*vt).mode.ctrl8bit() == 0 {
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
    cur = cur.wrapping_add(vsnprintf(
        (*vt).tmpbuffer.offset(cur as isize),
        (*vt).tmpbuffer_len.wrapping_sub(cur),
        fmt,
        args.as_va_list(),
    ) as size_t);
    if cur >= (*vt).tmpbuffer_len {
        return;
    }
    if term {
        cur = cur.wrapping_add(snprintf(
            (*vt).tmpbuffer.offset(cur as isize),
            (*vt).tmpbuffer_len.wrapping_sub(cur),
            if (*vt).mode.ctrl8bit() as ::core::ffi::c_int != 0 {
                b"\x9C\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"\x1B\\\0".as_ptr() as *const ::core::ffi::c_char
            },
        ) as size_t);
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
        unsafe extern "C" fn(VTermRect, VTermRect, *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
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
        Some(eraserect.expect("non-null function pointer")).expect("non-null function pointer")(
            rect,
            0 as ::core::ffi::c_int,
            user,
        );
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
        Some(moverect.expect("non-null function pointer")).expect("non-null function pointer")(
            dest, src, user,
        );
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
    Some(eraserect.expect("non-null function pointer")).expect("non-null function pointer")(
        rect,
        0 as ::core::ffi::c_int,
        user,
    );
}
