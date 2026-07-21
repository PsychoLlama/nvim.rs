use crate::src::nvim::global_cell::GlobalCell;
pub use crate::src::nvim::types::{
    cursorentry_T, int64_t, key_value_pair, object, object_data as C2Rust_Unnamed, size_t, uint8_t,
    Arena, Array, Boolean, CursorShape, Dict, Float, Integer, KeyValuePair, LuaRef, Object,
    ObjectType, String_0,
};
extern "C" {
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strncasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn logmsg(
        log_level: ::core::ffi::c_int,
        context: *const ::core::ffi::c_char,
        func_name: *const ::core::ffi::c_char,
        line_num: ::core::ffi::c_int,
        eol: bool,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> bool;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn arena_array(arena: *mut Arena, max_size: size_t) -> Array;
    fn arena_dict(arena: *mut Arena, max_size: size_t) -> Dict;
    static p_guicursor: GlobalCell<*mut ::core::ffi::c_char>;
    static p_sel: GlobalCell<*mut ::core::ffi::c_char>;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn getdigits_int(
        pp: *mut *mut ::core::ffi::c_char,
        strict: bool,
        def: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn cmdline_overstrike() -> bool;
    fn cmdline_at_end() -> bool;
    static VIsual_active: GlobalCell<bool>;
    static State: GlobalCell<::core::ffi::c_int>;
    static finish_op: GlobalCell<bool>;
    fn syn_check_group(name: *const ::core::ffi::c_char, len: size_t) -> ::core::ffi::c_int;
    fn syn_id2attr(hl_id: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn ui_mode_info_set();
}
pub const kObjectTypeTabpage: ObjectType = 10;
pub const kObjectTypeWindow: ObjectType = 9;
pub const kObjectTypeBuffer: ObjectType = 8;
pub const kObjectTypeLuaRef: ObjectType = 7;
pub const kObjectTypeDict: ObjectType = 6;
pub const kObjectTypeArray: ObjectType = 5;
pub const kObjectTypeString: ObjectType = 4;
pub const kObjectTypeFloat: ObjectType = 3;
pub const kObjectTypeInteger: ObjectType = 2;
pub const kObjectTypeBoolean: ObjectType = 1;
pub const kObjectTypeNil: ObjectType = 0;
pub type C2Rust_Unnamed_0 = ::core::ffi::c_uint;
pub const SHAPE_IDX_COUNT: C2Rust_Unnamed_0 = 18;
pub const SHAPE_IDX_TERM: C2Rust_Unnamed_0 = 17;
pub const SHAPE_IDX_SM: C2Rust_Unnamed_0 = 16;
pub const SHAPE_IDX_MOREL: C2Rust_Unnamed_0 = 15;
pub const SHAPE_IDX_MORE: C2Rust_Unnamed_0 = 14;
pub const SHAPE_IDX_VDRAG: C2Rust_Unnamed_0 = 13;
pub const SHAPE_IDX_VSEP: C2Rust_Unnamed_0 = 12;
pub const SHAPE_IDX_SDRAG: C2Rust_Unnamed_0 = 11;
pub const SHAPE_IDX_STATUS: C2Rust_Unnamed_0 = 10;
pub const SHAPE_IDX_CLINE: C2Rust_Unnamed_0 = 9;
pub const SHAPE_IDX_VE: C2Rust_Unnamed_0 = 8;
pub const SHAPE_IDX_O: C2Rust_Unnamed_0 = 7;
pub const SHAPE_IDX_CR: C2Rust_Unnamed_0 = 6;
pub const SHAPE_IDX_CI: C2Rust_Unnamed_0 = 5;
pub const SHAPE_IDX_C: C2Rust_Unnamed_0 = 4;
pub const SHAPE_IDX_R: C2Rust_Unnamed_0 = 3;
pub const SHAPE_IDX_I: C2Rust_Unnamed_0 = 2;
pub const SHAPE_IDX_V: C2Rust_Unnamed_0 = 1;
pub const SHAPE_IDX_N: C2Rust_Unnamed_0 = 0;
pub const SHAPE_VER: CursorShape = 2;
pub const SHAPE_HOR: CursorShape = 1;
pub const SHAPE_BLOCK: CursorShape = 0;
pub const MODE_CMDLINE: C2Rust_Unnamed_1 = 8;
pub const MODE_INSERT: C2Rust_Unnamed_1 = 16;
pub const REPLACE_FLAG: C2Rust_Unnamed_1 = 256;
pub const VREPLACE_FLAG: C2Rust_Unnamed_1 = 512;
pub const MODE_TERMINAL: C2Rust_Unnamed_1 = 128;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_1 = 24592;
pub type C2Rust_Unnamed_1 = ::core::ffi::c_uint;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_1 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_1 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_1 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_1 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_1 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_1 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_1 = 784;
pub const MODE_REPLACE: C2Rust_Unnamed_1 = 272;
pub const MAP_ALL_MODES: C2Rust_Unnamed_1 = 255;
pub const MODE_SELECT: C2Rust_Unnamed_1 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_1 = 32;
pub const MODE_OP_PENDING: C2Rust_Unnamed_1 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_1 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_1 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LOGLVL_WRN: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const SHAPE_MOUSE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SHAPE_CURSOR: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
static e_digit_expected: GlobalCell<[::core::ffi::c_char; 21]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"E548: Digit expected\0")
});
#[no_mangle]
pub static shape_table: GlobalCell<[cursorentry_T; 18]> = GlobalCell::new([
    cursorentry_T {
        full_name: b"normal\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        shape: SHAPE_BLOCK,
        mshape: 0 as ::core::ffi::c_int,
        percentage: 0 as ::core::ffi::c_int,
        blinkwait: 700 as ::core::ffi::c_int,
        blinkon: 400 as ::core::ffi::c_int,
        blinkoff: 250 as ::core::ffi::c_int,
        id: 0 as ::core::ffi::c_int,
        id_lm: 0 as ::core::ffi::c_int,
        name: b"n\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        used_for: (SHAPE_CURSOR + SHAPE_MOUSE) as ::core::ffi::c_char,
    },
    cursorentry_T {
        full_name: b"visual\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        shape: SHAPE_BLOCK,
        mshape: 0 as ::core::ffi::c_int,
        percentage: 0 as ::core::ffi::c_int,
        blinkwait: 700 as ::core::ffi::c_int,
        blinkon: 400 as ::core::ffi::c_int,
        blinkoff: 250 as ::core::ffi::c_int,
        id: 0 as ::core::ffi::c_int,
        id_lm: 0 as ::core::ffi::c_int,
        name: b"v\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        used_for: (SHAPE_CURSOR + SHAPE_MOUSE) as ::core::ffi::c_char,
    },
    cursorentry_T {
        full_name: b"insert\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        shape: SHAPE_BLOCK,
        mshape: 0 as ::core::ffi::c_int,
        percentage: 0 as ::core::ffi::c_int,
        blinkwait: 700 as ::core::ffi::c_int,
        blinkon: 400 as ::core::ffi::c_int,
        blinkoff: 250 as ::core::ffi::c_int,
        id: 0 as ::core::ffi::c_int,
        id_lm: 0 as ::core::ffi::c_int,
        name: b"i\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        used_for: (SHAPE_CURSOR + SHAPE_MOUSE) as ::core::ffi::c_char,
    },
    cursorentry_T {
        full_name: b"replace\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        shape: SHAPE_BLOCK,
        mshape: 0 as ::core::ffi::c_int,
        percentage: 0 as ::core::ffi::c_int,
        blinkwait: 700 as ::core::ffi::c_int,
        blinkon: 400 as ::core::ffi::c_int,
        blinkoff: 250 as ::core::ffi::c_int,
        id: 0 as ::core::ffi::c_int,
        id_lm: 0 as ::core::ffi::c_int,
        name: b"r\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        used_for: (SHAPE_CURSOR + SHAPE_MOUSE) as ::core::ffi::c_char,
    },
    cursorentry_T {
        full_name: b"cmdline_normal\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        shape: SHAPE_BLOCK,
        mshape: 0 as ::core::ffi::c_int,
        percentage: 0 as ::core::ffi::c_int,
        blinkwait: 700 as ::core::ffi::c_int,
        blinkon: 400 as ::core::ffi::c_int,
        blinkoff: 250 as ::core::ffi::c_int,
        id: 0 as ::core::ffi::c_int,
        id_lm: 0 as ::core::ffi::c_int,
        name: b"c\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        used_for: (SHAPE_CURSOR + SHAPE_MOUSE) as ::core::ffi::c_char,
    },
    cursorentry_T {
        full_name: b"cmdline_insert\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        shape: SHAPE_BLOCK,
        mshape: 0 as ::core::ffi::c_int,
        percentage: 0 as ::core::ffi::c_int,
        blinkwait: 700 as ::core::ffi::c_int,
        blinkon: 400 as ::core::ffi::c_int,
        blinkoff: 250 as ::core::ffi::c_int,
        id: 0 as ::core::ffi::c_int,
        id_lm: 0 as ::core::ffi::c_int,
        name: b"ci\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        used_for: (SHAPE_CURSOR + SHAPE_MOUSE) as ::core::ffi::c_char,
    },
    cursorentry_T {
        full_name: b"cmdline_replace\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        shape: SHAPE_BLOCK,
        mshape: 0 as ::core::ffi::c_int,
        percentage: 0 as ::core::ffi::c_int,
        blinkwait: 700 as ::core::ffi::c_int,
        blinkon: 400 as ::core::ffi::c_int,
        blinkoff: 250 as ::core::ffi::c_int,
        id: 0 as ::core::ffi::c_int,
        id_lm: 0 as ::core::ffi::c_int,
        name: b"cr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        used_for: (SHAPE_CURSOR + SHAPE_MOUSE) as ::core::ffi::c_char,
    },
    cursorentry_T {
        full_name: b"operator\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        shape: SHAPE_BLOCK,
        mshape: 0 as ::core::ffi::c_int,
        percentage: 0 as ::core::ffi::c_int,
        blinkwait: 700 as ::core::ffi::c_int,
        blinkon: 400 as ::core::ffi::c_int,
        blinkoff: 250 as ::core::ffi::c_int,
        id: 0 as ::core::ffi::c_int,
        id_lm: 0 as ::core::ffi::c_int,
        name: b"o\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        used_for: (SHAPE_CURSOR + SHAPE_MOUSE) as ::core::ffi::c_char,
    },
    cursorentry_T {
        full_name: b"visual_select\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        shape: SHAPE_BLOCK,
        mshape: 0 as ::core::ffi::c_int,
        percentage: 0 as ::core::ffi::c_int,
        blinkwait: 700 as ::core::ffi::c_int,
        blinkon: 400 as ::core::ffi::c_int,
        blinkoff: 250 as ::core::ffi::c_int,
        id: 0 as ::core::ffi::c_int,
        id_lm: 0 as ::core::ffi::c_int,
        name: b"ve\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        used_for: (SHAPE_CURSOR + SHAPE_MOUSE) as ::core::ffi::c_char,
    },
    cursorentry_T {
        full_name: b"cmdline_hover\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        shape: SHAPE_BLOCK,
        mshape: 0 as ::core::ffi::c_int,
        percentage: 0 as ::core::ffi::c_int,
        blinkwait: 0 as ::core::ffi::c_int,
        blinkon: 0 as ::core::ffi::c_int,
        blinkoff: 0 as ::core::ffi::c_int,
        id: 0 as ::core::ffi::c_int,
        id_lm: 0 as ::core::ffi::c_int,
        name: b"e\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        used_for: SHAPE_MOUSE as ::core::ffi::c_char,
    },
    cursorentry_T {
        full_name: b"statusline_hover\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        shape: SHAPE_BLOCK,
        mshape: 0 as ::core::ffi::c_int,
        percentage: 0 as ::core::ffi::c_int,
        blinkwait: 0 as ::core::ffi::c_int,
        blinkon: 0 as ::core::ffi::c_int,
        blinkoff: 0 as ::core::ffi::c_int,
        id: 0 as ::core::ffi::c_int,
        id_lm: 0 as ::core::ffi::c_int,
        name: b"s\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        used_for: SHAPE_MOUSE as ::core::ffi::c_char,
    },
    cursorentry_T {
        full_name: b"statusline_drag\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        shape: SHAPE_BLOCK,
        mshape: 0 as ::core::ffi::c_int,
        percentage: 0 as ::core::ffi::c_int,
        blinkwait: 0 as ::core::ffi::c_int,
        blinkon: 0 as ::core::ffi::c_int,
        blinkoff: 0 as ::core::ffi::c_int,
        id: 0 as ::core::ffi::c_int,
        id_lm: 0 as ::core::ffi::c_int,
        name: b"sd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        used_for: SHAPE_MOUSE as ::core::ffi::c_char,
    },
    cursorentry_T {
        full_name: b"vsep_hover\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        shape: SHAPE_BLOCK,
        mshape: 0 as ::core::ffi::c_int,
        percentage: 0 as ::core::ffi::c_int,
        blinkwait: 0 as ::core::ffi::c_int,
        blinkon: 0 as ::core::ffi::c_int,
        blinkoff: 0 as ::core::ffi::c_int,
        id: 0 as ::core::ffi::c_int,
        id_lm: 0 as ::core::ffi::c_int,
        name: b"vs\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        used_for: SHAPE_MOUSE as ::core::ffi::c_char,
    },
    cursorentry_T {
        full_name: b"vsep_drag\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        shape: SHAPE_BLOCK,
        mshape: 0 as ::core::ffi::c_int,
        percentage: 0 as ::core::ffi::c_int,
        blinkwait: 0 as ::core::ffi::c_int,
        blinkon: 0 as ::core::ffi::c_int,
        blinkoff: 0 as ::core::ffi::c_int,
        id: 0 as ::core::ffi::c_int,
        id_lm: 0 as ::core::ffi::c_int,
        name: b"vd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        used_for: SHAPE_MOUSE as ::core::ffi::c_char,
    },
    cursorentry_T {
        full_name: b"more\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        shape: SHAPE_BLOCK,
        mshape: 0 as ::core::ffi::c_int,
        percentage: 0 as ::core::ffi::c_int,
        blinkwait: 0 as ::core::ffi::c_int,
        blinkon: 0 as ::core::ffi::c_int,
        blinkoff: 0 as ::core::ffi::c_int,
        id: 0 as ::core::ffi::c_int,
        id_lm: 0 as ::core::ffi::c_int,
        name: b"m\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        used_for: SHAPE_MOUSE as ::core::ffi::c_char,
    },
    cursorentry_T {
        full_name: b"more_lastline\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        shape: SHAPE_BLOCK,
        mshape: 0 as ::core::ffi::c_int,
        percentage: 0 as ::core::ffi::c_int,
        blinkwait: 0 as ::core::ffi::c_int,
        blinkon: 0 as ::core::ffi::c_int,
        blinkoff: 0 as ::core::ffi::c_int,
        id: 0 as ::core::ffi::c_int,
        id_lm: 0 as ::core::ffi::c_int,
        name: b"ml\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        used_for: SHAPE_MOUSE as ::core::ffi::c_char,
    },
    cursorentry_T {
        full_name: b"showmatch\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        shape: SHAPE_BLOCK,
        mshape: 0 as ::core::ffi::c_int,
        percentage: 0 as ::core::ffi::c_int,
        blinkwait: 100 as ::core::ffi::c_int,
        blinkon: 100 as ::core::ffi::c_int,
        blinkoff: 100 as ::core::ffi::c_int,
        id: 0 as ::core::ffi::c_int,
        id_lm: 0 as ::core::ffi::c_int,
        name: b"sm\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        used_for: SHAPE_CURSOR as ::core::ffi::c_char,
    },
    cursorentry_T {
        full_name: b"terminal\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        shape: SHAPE_BLOCK,
        mshape: 0 as ::core::ffi::c_int,
        percentage: 0 as ::core::ffi::c_int,
        blinkwait: 0 as ::core::ffi::c_int,
        blinkon: 0 as ::core::ffi::c_int,
        blinkoff: 0 as ::core::ffi::c_int,
        id: 0 as ::core::ffi::c_int,
        id_lm: 0 as ::core::ffi::c_int,
        name: b"t\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        used_for: SHAPE_CURSOR as ::core::ffi::c_char,
    },
]);
#[no_mangle]
pub unsafe extern "C" fn mode_style_array(mut arena: *mut Arena) -> Array {
    let mut all: Array = arena_array(arena, SHAPE_IDX_COUNT as ::core::ffi::c_int as size_t);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < SHAPE_IDX_COUNT as ::core::ffi::c_int {
        let mut cur: *mut cursorentry_T =
            (shape_table.ptr() as *mut cursorentry_T).offset(i as isize);
        let mut dic: Dict = arena_dict(
            arena,
            (3 as ::core::ffi::c_int
                + (if (*cur).used_for as ::core::ffi::c_int & SHAPE_CURSOR != 0 {
                    9 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                })) as size_t,
        );
        let c2rust_fresh0 = dic.size;
        dic.size = dic.size.wrapping_add(1);
        *dic.items.offset(c2rust_fresh0 as isize) = key_value_pair {
            key: cstr_as_string(b"name\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string((*cur).full_name),
                },
            },
        };
        let c2rust_fresh1 = dic.size;
        dic.size = dic.size.wrapping_add(1);
        *dic.items.offset(c2rust_fresh1 as isize) = key_value_pair {
            key: cstr_as_string(b"short_name\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string((*cur).name),
                },
            },
        };
        if (*cur).used_for as ::core::ffi::c_int & SHAPE_MOUSE != 0 {
            let c2rust_fresh2 = dic.size;
            dic.size = dic.size.wrapping_add(1);
            *dic.items.offset(c2rust_fresh2 as isize) = key_value_pair {
                key: cstr_as_string(b"mouse_shape\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: (*cur).mshape as Integer,
                    },
                },
            };
        }
        if (*cur).used_for as ::core::ffi::c_int & SHAPE_CURSOR != 0 {
            let mut shape_str: String_0 = String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            };
            match (*cur).shape as ::core::ffi::c_uint {
                0 => {
                    shape_str = cstr_as_string(b"block\0".as_ptr() as *const ::core::ffi::c_char);
                }
                2 => {
                    shape_str =
                        cstr_as_string(b"vertical\0".as_ptr() as *const ::core::ffi::c_char);
                }
                1 => {
                    shape_str =
                        cstr_as_string(b"horizontal\0".as_ptr() as *const ::core::ffi::c_char);
                }
                _ => {
                    shape_str = cstr_as_string(b"unknown\0".as_ptr() as *const ::core::ffi::c_char);
                }
            }
            let c2rust_fresh3 = dic.size;
            dic.size = dic.size.wrapping_add(1);
            *dic.items.offset(c2rust_fresh3 as isize) = key_value_pair {
                key: cstr_as_string(b"cursor_shape\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed { string: shape_str },
                },
            };
            let c2rust_fresh4 = dic.size;
            dic.size = dic.size.wrapping_add(1);
            *dic.items.offset(c2rust_fresh4 as isize) = key_value_pair {
                key: cstr_as_string(b"cell_percentage\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: (*cur).percentage as Integer,
                    },
                },
            };
            let c2rust_fresh5 = dic.size;
            dic.size = dic.size.wrapping_add(1);
            *dic.items.offset(c2rust_fresh5 as isize) = key_value_pair {
                key: cstr_as_string(b"blinkwait\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: (*cur).blinkwait as Integer,
                    },
                },
            };
            let c2rust_fresh6 = dic.size;
            dic.size = dic.size.wrapping_add(1);
            *dic.items.offset(c2rust_fresh6 as isize) = key_value_pair {
                key: cstr_as_string(b"blinkon\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: (*cur).blinkon as Integer,
                    },
                },
            };
            let c2rust_fresh7 = dic.size;
            dic.size = dic.size.wrapping_add(1);
            *dic.items.offset(c2rust_fresh7 as isize) = key_value_pair {
                key: cstr_as_string(b"blinkoff\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: (*cur).blinkoff as Integer,
                    },
                },
            };
            let c2rust_fresh8 = dic.size;
            dic.size = dic.size.wrapping_add(1);
            *dic.items.offset(c2rust_fresh8 as isize) = key_value_pair {
                key: cstr_as_string(b"hl_id\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: (*cur).id as Integer,
                    },
                },
            };
            let c2rust_fresh9 = dic.size;
            dic.size = dic.size.wrapping_add(1);
            *dic.items.offset(c2rust_fresh9 as isize) = key_value_pair {
                key: cstr_as_string(b"id_lm\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: (*cur).id_lm as Integer,
                    },
                },
            };
            let c2rust_fresh10 = dic.size;
            dic.size = dic.size.wrapping_add(1);
            *dic.items.offset(c2rust_fresh10 as isize) = key_value_pair {
                key: cstr_as_string(b"attr_id\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: (if (*cur).id != 0 {
                            syn_id2attr((*cur).id)
                        } else {
                            0 as ::core::ffi::c_int
                        }) as Integer,
                    },
                },
            };
            let c2rust_fresh11 = dic.size;
            dic.size = dic.size.wrapping_add(1);
            *dic.items.offset(c2rust_fresh11 as isize) = key_value_pair {
                key: cstr_as_string(b"attr_id_lm\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: (if (*cur).id_lm != 0 {
                            syn_id2attr((*cur).id_lm)
                        } else {
                            0 as ::core::ffi::c_int
                        }) as Integer,
                    },
                },
            };
        }
        let c2rust_fresh12 = all.size;
        all.size = all.size.wrapping_add(1);
        *all.items.offset(c2rust_fresh12 as isize) = object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed { dict: dic },
        };
        i += 1;
    }
    return all;
}
#[no_mangle]
pub unsafe extern "C" fn parse_shape_opt(
    mut what: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut len: ::core::ffi::c_int = 0;
    let mut found_ve: bool = false_0 != 0;
    let mut round: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while round <= 2 as ::core::ffi::c_int {
        if round == 2 as ::core::ffi::c_int || *p_guicursor.get() as ::core::ffi::c_int == NUL {
            clear_shape_table();
            if *p_guicursor.get() as ::core::ffi::c_int == NUL {
                ui_mode_info_set();
                return ::core::ptr::null::<::core::ffi::c_char>();
            }
        }
        let mut modep: *mut ::core::ffi::c_char = p_guicursor.get();
        while !modep.is_null() && *modep as ::core::ffi::c_int != NUL {
            let mut colonp: *mut ::core::ffi::c_char = vim_strchr(modep, ':' as ::core::ffi::c_int);
            let mut commap: *mut ::core::ffi::c_char = vim_strchr(modep, ',' as ::core::ffi::c_int);
            if colonp.is_null() || !commap.is_null() && commap < colonp {
                return b"E545: Missing colon\0".as_ptr() as *const ::core::ffi::c_char;
            }
            if colonp == modep {
                return b"E546: Illegal mode\0".as_ptr() as *const ::core::ffi::c_char;
            }
            let mut all_idx: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
            while modep < colonp || all_idx >= 0 as ::core::ffi::c_int {
                if all_idx < 0 as ::core::ffi::c_int {
                    if *modep.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '-' as ::core::ffi::c_int
                        || *modep.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == ':' as ::core::ffi::c_int
                    {
                        len = 1 as ::core::ffi::c_int;
                    } else {
                        len = 2 as ::core::ffi::c_int;
                    }
                    if len == 1 as ::core::ffi::c_int
                        && (if (*modep.offset(0 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int)
                            < 'A' as ::core::ffi::c_int
                            || *modep.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                > 'Z' as ::core::ffi::c_int
                        {
                            *modep.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        } else {
                            *modep.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                        }) == 'a' as ::core::ffi::c_int
                    {
                        all_idx = SHAPE_IDX_COUNT as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
                    } else {
                        idx = 0 as ::core::ffi::c_int;
                        while idx < SHAPE_IDX_COUNT as ::core::ffi::c_int {
                            if strncasecmp(
                                modep,
                                (*shape_table.ptr())[idx as usize].name,
                                len as size_t,
                            ) == 0 as ::core::ffi::c_int
                            {
                                break;
                            }
                            idx += 1;
                        }
                        if idx == SHAPE_IDX_COUNT as ::core::ffi::c_int
                            || (*shape_table.ptr())[idx as usize].used_for as ::core::ffi::c_int
                                & what
                                == 0 as ::core::ffi::c_int
                        {
                            return b"E546: Illegal mode\0".as_ptr() as *const ::core::ffi::c_char;
                        }
                        if len == 2 as ::core::ffi::c_int
                            && *modep.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == 'v' as ::core::ffi::c_int
                            && *modep.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == 'e' as ::core::ffi::c_int
                        {
                            found_ve = true_0 != 0;
                        }
                    }
                    modep = modep.offset((len + 1 as ::core::ffi::c_int) as isize);
                }
                if all_idx >= 0 as ::core::ffi::c_int {
                    let c2rust_fresh13 = all_idx;
                    all_idx = all_idx - 1;
                    idx = c2rust_fresh13;
                }
                p = colonp.offset(1 as ::core::ffi::c_int as isize);
                while *p as ::core::ffi::c_int != 0
                    && *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                {
                    let mut i: ::core::ffi::c_int = *p as uint8_t as ::core::ffi::c_int;
                    len = 0 as ::core::ffi::c_int;
                    if strncasecmp(
                        p,
                        b"ver\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                        3 as ::core::ffi::c_int as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        len = 3 as ::core::ffi::c_int;
                    } else if strncasecmp(
                        p,
                        b"hor\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                        3 as ::core::ffi::c_int as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        len = 3 as ::core::ffi::c_int;
                    } else if strncasecmp(
                        p,
                        b"blinkwait\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        9 as ::core::ffi::c_int as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        len = 9 as ::core::ffi::c_int;
                    } else if strncasecmp(
                        p,
                        b"blinkon\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        7 as ::core::ffi::c_int as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        len = 7 as ::core::ffi::c_int;
                    } else if strncasecmp(
                        p,
                        b"blinkoff\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        8 as ::core::ffi::c_int as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        len = 8 as ::core::ffi::c_int;
                    }
                    if len != 0 as ::core::ffi::c_int {
                        p = p.offset(len as isize);
                        if !ascii_isdigit(*p as ::core::ffi::c_int) {
                            return (e_digit_expected.ptr() as *const _)
                                as *const ::core::ffi::c_char;
                        }
                        let mut n: ::core::ffi::c_int =
                            getdigits_int(&raw mut p, false_0 != 0, 0 as ::core::ffi::c_int);
                        if len == 3 as ::core::ffi::c_int {
                            if n == 0 as ::core::ffi::c_int {
                                return b"E549: Illegal percentage\0".as_ptr()
                                    as *const ::core::ffi::c_char;
                            }
                            if round == 2 as ::core::ffi::c_int {
                                if (if i < 'A' as ::core::ffi::c_int
                                    || i > 'Z' as ::core::ffi::c_int
                                {
                                    i
                                } else {
                                    i + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                                }) == 'v' as ::core::ffi::c_int
                                {
                                    (*shape_table.ptr())[idx as usize].shape = SHAPE_VER;
                                } else {
                                    (*shape_table.ptr())[idx as usize].shape = SHAPE_HOR;
                                }
                                (*shape_table.ptr())[idx as usize].percentage = n;
                            }
                        } else if round == 2 as ::core::ffi::c_int {
                            if len == 9 as ::core::ffi::c_int {
                                (*shape_table.ptr())[idx as usize].blinkwait = n;
                            } else if len == 7 as ::core::ffi::c_int {
                                (*shape_table.ptr())[idx as usize].blinkon = n;
                            } else {
                                (*shape_table.ptr())[idx as usize].blinkoff = n;
                            }
                        }
                    } else if strncasecmp(
                        p,
                        b"block\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        5 as ::core::ffi::c_int as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        if round == 2 as ::core::ffi::c_int {
                            (*shape_table.ptr())[idx as usize].shape = SHAPE_BLOCK;
                        }
                        p = p.offset(5 as ::core::ffi::c_int as isize);
                    } else {
                        let mut endp: *mut ::core::ffi::c_char =
                            vim_strchr(p, '-' as ::core::ffi::c_int);
                        if commap.is_null() {
                            if endp.is_null() {
                                endp = p.offset(strlen(p) as isize);
                            }
                        } else if endp > commap || endp.is_null() {
                            endp = commap;
                        }
                        let mut slashp: *mut ::core::ffi::c_char =
                            vim_strchr(p, '/' as ::core::ffi::c_int);
                        if !slashp.is_null() && slashp < endp {
                            i = syn_check_group(p, slashp.offset_from(p) as size_t);
                            p = slashp.offset(1 as ::core::ffi::c_int as isize);
                        }
                        if round == 2 as ::core::ffi::c_int {
                            (*shape_table.ptr())[idx as usize].id =
                                syn_check_group(p, endp.offset_from(p) as size_t);
                            (*shape_table.ptr())[idx as usize].id_lm =
                                (*shape_table.ptr())[idx as usize].id;
                            if !slashp.is_null() && slashp < endp {
                                (*shape_table.ptr())[idx as usize].id = i;
                            }
                        }
                        p = endp;
                    }
                    if *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
                        p = p.offset(1);
                    }
                }
            }
            modep = p;
            if !modep.is_null() && *modep as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
                modep = modep.offset(1);
            }
        }
        round += 1;
    }
    if !found_ve {
        (*shape_table.ptr())[SHAPE_IDX_VE as ::core::ffi::c_int as usize].shape =
            (*shape_table.ptr())[SHAPE_IDX_V as ::core::ffi::c_int as usize].shape;
        (*shape_table.ptr())[SHAPE_IDX_VE as ::core::ffi::c_int as usize].percentage =
            (*shape_table.ptr())[SHAPE_IDX_V as ::core::ffi::c_int as usize].percentage;
        (*shape_table.ptr())[SHAPE_IDX_VE as ::core::ffi::c_int as usize].blinkwait =
            (*shape_table.ptr())[SHAPE_IDX_V as ::core::ffi::c_int as usize].blinkwait;
        (*shape_table.ptr())[SHAPE_IDX_VE as ::core::ffi::c_int as usize].blinkon =
            (*shape_table.ptr())[SHAPE_IDX_V as ::core::ffi::c_int as usize].blinkon;
        (*shape_table.ptr())[SHAPE_IDX_VE as ::core::ffi::c_int as usize].blinkoff =
            (*shape_table.ptr())[SHAPE_IDX_V as ::core::ffi::c_int as usize].blinkoff;
        (*shape_table.ptr())[SHAPE_IDX_VE as ::core::ffi::c_int as usize].id =
            (*shape_table.ptr())[SHAPE_IDX_V as ::core::ffi::c_int as usize].id;
        (*shape_table.ptr())[SHAPE_IDX_VE as ::core::ffi::c_int as usize].id_lm =
            (*shape_table.ptr())[SHAPE_IDX_V as ::core::ffi::c_int as usize].id_lm;
    }
    ui_mode_info_set();
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn cursor_is_block_during_visual(mut exclusive: bool) -> bool {
    let mut mode_idx: ::core::ffi::c_int = if exclusive as ::core::ffi::c_int != 0 {
        SHAPE_IDX_VE as ::core::ffi::c_int
    } else {
        SHAPE_IDX_V as ::core::ffi::c_int
    };
    return SHAPE_BLOCK as ::core::ffi::c_int as ::core::ffi::c_uint
        == (*shape_table.ptr())[mode_idx as usize].shape as ::core::ffi::c_uint
        && 0 as ::core::ffi::c_int == (*shape_table.ptr())[mode_idx as usize].blinkon;
}
#[no_mangle]
pub unsafe extern "C" fn cursor_mode_str2int(
    mut mode: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut mode_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while mode_idx < SHAPE_IDX_COUNT as ::core::ffi::c_int {
        if strcmp((*shape_table.ptr())[mode_idx as usize].full_name, mode)
            == 0 as ::core::ffi::c_int
        {
            return mode_idx;
        }
        mode_idx += 1;
    }
    logmsg(
        LOGLVL_WRN,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"cursor_mode_str2int\0".as_ptr() as *const ::core::ffi::c_char,
        297 as ::core::ffi::c_int,
        true_0 != 0,
        b"Unknown mode %s\0".as_ptr() as *const ::core::ffi::c_char,
        mode,
    );
    return -1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cursor_mode_uses_syn_id(mut syn_id: ::core::ffi::c_int) -> bool {
    if *p_guicursor.get() as ::core::ffi::c_int == NUL {
        return false_0 != 0;
    }
    let mut mode_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while mode_idx < SHAPE_IDX_COUNT as ::core::ffi::c_int {
        if (*shape_table.ptr())[mode_idx as usize].id == syn_id
            || (*shape_table.ptr())[mode_idx as usize].id_lm == syn_id
        {
            return true_0 != 0;
        }
        mode_idx += 1;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn cursor_get_mode_idx() -> ::core::ffi::c_int {
    if State.get() == MODE_SHOWMATCH as ::core::ffi::c_int {
        return SHAPE_IDX_SM as ::core::ffi::c_int;
    } else if State.get() == MODE_TERMINAL as ::core::ffi::c_int {
        return SHAPE_IDX_TERM as ::core::ffi::c_int;
    } else if State.get() & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
        return SHAPE_IDX_R as ::core::ffi::c_int;
    } else if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0 {
        return SHAPE_IDX_R as ::core::ffi::c_int;
    } else if State.get() & MODE_INSERT as ::core::ffi::c_int != 0 {
        return SHAPE_IDX_I as ::core::ffi::c_int;
    } else if State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0 {
        if cmdline_at_end() {
            return SHAPE_IDX_C as ::core::ffi::c_int;
        } else if cmdline_overstrike() {
            return SHAPE_IDX_CR as ::core::ffi::c_int;
        } else {
            return SHAPE_IDX_CI as ::core::ffi::c_int;
        }
    } else if finish_op.get() {
        return SHAPE_IDX_O as ::core::ffi::c_int;
    } else if VIsual_active.get() {
        if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
            return SHAPE_IDX_VE as ::core::ffi::c_int;
        } else {
            return SHAPE_IDX_V as ::core::ffi::c_int;
        }
    } else {
        return SHAPE_IDX_N as ::core::ffi::c_int;
    };
}
unsafe extern "C" fn clear_shape_table() {
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while idx < SHAPE_IDX_COUNT as ::core::ffi::c_int {
        (*shape_table.ptr())[idx as usize].shape = SHAPE_BLOCK;
        (*shape_table.ptr())[idx as usize].blinkwait = 0 as ::core::ffi::c_int;
        (*shape_table.ptr())[idx as usize].blinkon = 0 as ::core::ffi::c_int;
        (*shape_table.ptr())[idx as usize].blinkoff = 0 as ::core::ffi::c_int;
        (*shape_table.ptr())[idx as usize].id = 0 as ::core::ffi::c_int;
        (*shape_table.ptr())[idx as usize].id_lm = 0 as ::core::ffi::c_int;
        idx += 1;
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
