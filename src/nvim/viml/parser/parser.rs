use crate::src::nvim::mbyte::string_convert;
use crate::src::nvim::memory::{xfree, xmalloc, xrealloc};
use crate::src::nvim::os::libc::{__assert_fail, memcpy};
pub use crate::src::nvim::types::{
    iconv_t, size_t, vimconv_T, ParserHighlight, ParserHighlightChunk, ParserInputReader,
    ParserInputReader_lines as C2Rust_Unnamed_4, ParserLine, ParserLineGetter, ParserPosition,
    ParserState, ParserStateItem, ParserStateItem_data as C2Rust_Unnamed_0,
    ParserStateItem_data_expr as C2Rust_Unnamed_1,
    ParserStateItem_data_expr_type_0 as C2Rust_Unnamed_2,
    ParserStateItem_type_0 as C2Rust_Unnamed_3, ParserState_stack as C2Rust_Unnamed_5,
};
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const CONV_ICONV: C2Rust_Unnamed = 5;
pub const CONV_TO_LATIN9: C2Rust_Unnamed = 4;
pub const CONV_TO_LATIN1: C2Rust_Unnamed = 3;
pub const CONV_9_TO_UTF8: C2Rust_Unnamed = 2;
pub const CONV_TO_UTF8: C2Rust_Unnamed = 1;
pub const CONV_NONE: C2Rust_Unnamed = 0;
pub const kExprUnknown: C2Rust_Unnamed_2 = 0;
pub const kPTopStateParsingExpression: C2Rust_Unnamed_3 = 1;
pub const kPTopStateParsingCommand: C2Rust_Unnamed_3 = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
#[inline(always)]
unsafe extern "C" fn _memcpy_free(
    dest: *mut ::core::ffi::c_void,
    src: *mut ::core::ffi::c_void,
    size: size_t,
) -> *mut ::core::ffi::c_void {
    memcpy(dest, src, size);
    let mut ptr_: *mut *mut ::core::ffi::c_void = &raw const src as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    return dest;
}
#[no_mangle]
pub unsafe extern "C" fn parser_simple_get_line(
    mut cookie: *mut ::core::ffi::c_void,
    mut ret_pline: *mut ParserLine,
) {
    let mut plines_p: *mut *mut ParserLine = cookie as *mut *mut ParserLine;
    *ret_pline = **plines_p;
    *plines_p = (*plines_p).offset(1);
}
#[no_mangle]
pub unsafe extern "C" fn viml_parser_get_remaining_line(
    pstate: *mut ParserState,
    ret_pline: *mut ParserLine,
) -> bool {
    let num_lines: size_t = (*pstate).reader.lines.size;
    if (*pstate).pos.line == num_lines {
        viml_preader_get_line(&raw mut (*pstate).reader, ret_pline);
    } else {
        *ret_pline = *(*pstate).reader.lines.items.offset(
            (*pstate)
                .reader
                .lines
                .size
                .wrapping_sub(0 as size_t)
                .wrapping_sub(1 as size_t) as isize,
        );
    }
    '_c2rust_label: {
        if (*pstate).pos.line == (*pstate).reader.lines.size.wrapping_sub(1 as size_t) {
        } else {
            __assert_fail(
                b"pstate->pos.line == kv_size(pstate->reader.lines) - 1\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/viml/parser/parser.rs\0".as_ptr() as *const ::core::ffi::c_char,
                30 as ::core::ffi::c_uint,
                b"_Bool viml_parser_get_remaining_line(ParserState *const, ParserLine *const)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if !(*ret_pline).data.is_null() {
        (*ret_pline).data = (*ret_pline).data.offset((*pstate).pos.col as isize);
        (*ret_pline).size = (*ret_pline).size.wrapping_sub((*pstate).pos.col);
    }
    return !(*ret_pline).data.is_null();
}
unsafe extern "C" fn viml_preader_get_line(
    preader: *mut ParserInputReader,
    ret_pline: *mut ParserLine,
) {
    let mut pline: ParserLine = ParserLine {
        data: ::core::ptr::null::<::core::ffi::c_char>(),
        size: 0,
        allocated: false,
    };
    (*preader).get_line.expect("non-null function pointer")((*preader).cookie, &raw mut pline);
    if (*preader).conv.vc_type != CONV_NONE as ::core::ffi::c_int && pline.size != 0 {
        let mut cpline: ParserLine = ParserLine {
            data: ::core::ptr::null::<::core::ffi::c_char>(),
            size: pline.size,
            allocated: true_0 != 0,
        };
        cpline.data = string_convert(
            &raw mut (*preader).conv,
            pline.data as *mut ::core::ffi::c_char,
            &raw mut cpline.size,
        );
        if pline.allocated {
            xfree(pline.data as *mut ::core::ffi::c_void);
        }
        pline = cpline;
    }
    if (*preader).lines.size == (*preader).lines.capacity {
        (*preader).lines.capacity = if (*preader).lines.capacity << 1 as ::core::ffi::c_int
            > ::core::mem::size_of::<[ParserLine; 4]>()
                .wrapping_div(::core::mem::size_of::<ParserLine>())
                .wrapping_div(
                    (::core::mem::size_of::<[ParserLine; 4]>()
                        .wrapping_rem(::core::mem::size_of::<ParserLine>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            (*preader).lines.capacity << 1 as ::core::ffi::c_int
        } else {
            ::core::mem::size_of::<[ParserLine; 4]>()
                .wrapping_div(::core::mem::size_of::<ParserLine>())
                .wrapping_div(
                    (::core::mem::size_of::<[ParserLine; 4]>()
                        .wrapping_rem(::core::mem::size_of::<ParserLine>())
                        == 0) as ::core::ffi::c_int as size_t,
                )
        };
        (*preader).lines.items = (if (*preader).lines.capacity
            == ::core::mem::size_of::<[ParserLine; 4]>()
                .wrapping_div(::core::mem::size_of::<ParserLine>())
                .wrapping_div(
                    (::core::mem::size_of::<[ParserLine; 4]>()
                        .wrapping_rem(::core::mem::size_of::<ParserLine>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            if (*preader).lines.items == &raw mut (*preader).lines.init_array as *mut ParserLine {
                (*preader).lines.items as *mut ::core::ffi::c_void
            } else {
                _memcpy_free(
                    &raw mut (*preader).lines.init_array as *mut ParserLine
                        as *mut ::core::ffi::c_void,
                    (*preader).lines.items as *mut ::core::ffi::c_void,
                    (*preader)
                        .lines
                        .size
                        .wrapping_mul(::core::mem::size_of::<ParserLine>()),
                )
            }
        } else {
            if (*preader).lines.items == &raw mut (*preader).lines.init_array as *mut ParserLine {
                memcpy(
                    xmalloc(
                        (*preader)
                            .lines
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<ParserLine>()),
                    ),
                    (*preader).lines.items as *const ::core::ffi::c_void,
                    (*preader)
                        .lines
                        .size
                        .wrapping_mul(::core::mem::size_of::<ParserLine>()),
                )
            } else {
                xrealloc(
                    (*preader).lines.items as *mut ::core::ffi::c_void,
                    (*preader)
                        .lines
                        .capacity
                        .wrapping_mul(::core::mem::size_of::<ParserLine>()),
                )
            }
        }) as *mut ParserLine;
    } else {
    };
    let c2rust_fresh0 = (*preader).lines.size;
    (*preader).lines.size = (*preader).lines.size.wrapping_add(1);
    *(*preader).lines.items.offset(c2rust_fresh0 as isize) = pline;
    *ret_pline = pline;
}
#[no_mangle]
pub unsafe extern "C" fn viml_parser_destroy(pstate: *mut ParserState) {
    let mut i: size_t = 0 as size_t;
    while i < (*pstate).reader.lines.size {
        let mut pline: ParserLine = *(*pstate).reader.lines.items.offset(i as isize);
        if pline.allocated {
            xfree(pline.data as *mut ::core::ffi::c_void);
        }
        i = i.wrapping_add(1);
    }
    if (*pstate).reader.lines.items != &raw mut (*pstate).reader.lines.init_array as *mut ParserLine
    {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*pstate).reader.lines.items as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
    }
    if (*pstate).stack.items != &raw mut (*pstate).stack.init_array as *mut ParserStateItem {
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut (*pstate).stack.items as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_0;
        let _ = *ptr__0;
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
