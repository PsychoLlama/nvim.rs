use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::charset::{char2cells, ptr2cells, vim_isprintc, vim_iswordc_tab};
use crate::src::nvim::cursor::get_cursor_pos_ptr;
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, redraw_all_later};
use crate::src::nvim::eval::typval::{
    tv_check_for_string_arg, tv_get_string, tv_get_string_buf, tv_list_alloc, tv_list_alloc_ret,
    tv_list_append_list, tv_list_append_number,
};
use crate::src::nvim::eval::typval::{tv_list_first, tv_list_len};
use crate::src::nvim::getchar::beep_flush;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::schar_from_buf;
use crate::src::nvim::keycodes::K_SPECIAL;
use crate::src::nvim::main::{
    IObuff, cmp_flags, curbuf, curwin, e_listreq, fenc_default, p_ambw, p_emoji, p_enc,
};
use crate::src::nvim::mark::mark_mb_adjustpos;
use crate::src::nvim::memline::ml_get_buf;
use crate::src::nvim::memory::{xfree, xmalloc, xstrdup};
use crate::src::nvim::message::{emsg, msg, semsg};
use crate::src::nvim::r#move::changed_window_setting_all;
use crate::src::nvim::options::{kOptCmpFlagInternal, kOptCmpFlagKeepascii};
use crate::src::nvim::optionstr::check_chars_options;
use crate::src::nvim::os::env::os_getenv_noalloc;
use crate::src::nvim::os::libc::{
    __ctype_b_loc, __errno_location, gettext, iconv, iconv_close, iconv_open, memcmp, memcpy,
    memmove, qsort, setlocale, snprintf, strchr, strcmp, strcpy, strlen, strncasecmp, strncmp,
    tolower, toupper,
};
use crate::src::nvim::pos::MAXCOL;
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{
    CharBoundsOff, CharInfo, EvalFuncData, GraphemeState, StrCharInfo, VAR_LIST, VAR_NUMBER,
    VAR_STRING, colnr_T, expand_T, iconv_t, int8_t, int32_t, int64_t, list_T, listitem_T, pos_T,
    ptrdiff_t, schar_T, size_t, ssize_t, typval_T, uint8_t, uint32_t, uint64_t, uintptr_t,
    utf8proc_int32_t, varnumber_T, vimconv_T, win_T,
};
use crate::src::nvim::utf8proc::{
    UTF8PROC_BOUNDCLASS_CONTROL, UTF8PROC_BOUNDCLASS_CR, UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC,
    UTF8PROC_BOUNDCLASS_OTHER, UTF8PROC_BOUNDCLASS_PREPEND, UTF8PROC_BOUNDCLASS_REGIONAL_INDICATOR,
    UTF8PROC_CASEFOLD, UTF8PROC_CATEGORY_ME, UTF8PROC_CATEGORY_MN, utf8proc_decompose_char,
    utf8proc_get_property, utf8proc_grapheme_break, utf8proc_grapheme_break_stateful,
    utf8proc_property_t, utf8proc_tolower, utf8proc_toupper,
};
unsafe extern "C" {
    #[cfg(not(miri))]
    fn towlower(__wc: wint_t) -> wint_t;
    #[cfg(not(miri))]
    fn towupper(__wc: wint_t) -> wint_t;
    fn nl_langinfo(__item: nl_item) -> *mut ::core::ffi::c_char;
}

// Miri cannot call libc. The tests never call setlocale, so glibc would run
// these in the C locale, where they fold ASCII only — which is exactly what
// these definitions do.
#[cfg(miri)]
fn towlower(__wc: wint_t) -> wint_t {
    u8::try_from(__wc).map_or(__wc, |b| b.to_ascii_lowercase() as wint_t)
}
#[cfg(miri)]
fn towupper(__wc: wint_t) -> wint_t {
    u8::try_from(__wc).map_or(__wc, |b| b.to_ascii_uppercase() as wint_t)
}
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISalnum: C2Rust_Unnamed = 8;
pub const _ISpunct: C2Rust_Unnamed = 4;
pub const _IScntrl: C2Rust_Unnamed = 2;
pub const _ISgraph: C2Rust_Unnamed = 32768;
pub const _ISalpha: C2Rust_Unnamed = 1024;
pub type wint_t = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_2 = ::core::ffi::c_uint;
pub type WorkingStatus = ::core::ffi::c_uint;
pub const kBroken: WorkingStatus = 2;
pub const kWorking: WorkingStatus = 1;
pub const kUnknown: WorkingStatus = 0;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const MB_MAXCHAR: C2Rust_Unnamed_18 = 6;
pub const MB_MAXBYTES: C2Rust_Unnamed_18 = 21;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const ENC_MACROMAN: C2Rust_Unnamed_19 = 2048;
pub const ENC_LATIN9: C2Rust_Unnamed_19 = 1024;
pub const ENC_LATIN1: C2Rust_Unnamed_19 = 512;
pub const ENC_2WORD: C2Rust_Unnamed_19 = 256;
pub const ENC_4BYTE: C2Rust_Unnamed_19 = 128;
pub const ENC_2BYTE: C2Rust_Unnamed_19 = 64;
pub const ENC_ENDIAN_L: C2Rust_Unnamed_19 = 32;
pub const ENC_ENDIAN_B: C2Rust_Unnamed_19 = 16;
pub const ENC_UNICODE: C2Rust_Unnamed_19 = 4;
pub const ENC_DBCS: C2Rust_Unnamed_19 = 2;
pub const ENC_8BIT: C2Rust_Unnamed_19 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const CONV_ICONV: C2Rust_Unnamed_20 = 5;
pub const CONV_TO_LATIN9: C2Rust_Unnamed_20 = 4;
pub const CONV_TO_LATIN1: C2Rust_Unnamed_20 = 3;
pub const CONV_9_TO_UTF8: C2Rust_Unnamed_20 = 2;
pub const CONV_TO_UTF8: C2Rust_Unnamed_20 = 1;
pub const CONV_NONE: C2Rust_Unnamed_20 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_21 {
    pub name: *const ::core::ffi::c_char,
    pub prop: ::core::ffi::c_int,
    pub codepage: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct clinterval {
    pub first: ::core::ffi::c_uint,
    pub last: ::core::ffi::c_uint,
    pub cls: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cw_interval_T {
    pub first: int64_t,
    pub last: int64_t,
    pub width: ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct interval {
    pub first: ::core::ffi::c_int,
    pub last: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_22 {
    pub name: *const ::core::ffi::c_char,
    pub canon: ::core::ffi::c_int,
}
pub const CODESET: C2Rust_Unnamed_23 = 14;
pub type nl_item = ::core::ffi::c_int;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
static corrections: GlobalCell<[uint32_t; 7]> = GlobalCell::new([0; 7]);
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LC_CTYPE: ::core::ffi::c_int = __LC_CTYPE;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const MAX_SCHAR_SIZE: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const DBCS_JPN: ::core::ffi::c_int = 932 as ::core::ffi::c_int;
pub const DBCS_JPNU: ::core::ffi::c_int = 9932 as ::core::ffi::c_int;
pub const DBCS_KOR: ::core::ffi::c_int = 949 as ::core::ffi::c_int;
pub const DBCS_KORU: ::core::ffi::c_int = 9949 as ::core::ffi::c_int;
pub const DBCS_CHS: ::core::ffi::c_int = 936 as ::core::ffi::c_int;
pub const DBCS_CHSU: ::core::ffi::c_int = 9936 as ::core::ffi::c_int;
pub const DBCS_CHT: ::core::ffi::c_int = 950 as ::core::ffi::c_int;
pub const DBCS_CHTU: ::core::ffi::c_int = 9950 as ::core::ffi::c_int;
pub const DBCS_DEBUG: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const KS_SPECIAL: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
pub const KE_FILLER: ::core::ffi::c_int = 'X' as ::core::ffi::c_int;
static e_list_item_nr_is_not_list: GlobalCell<[::core::ffi::c_char; 34]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
            *b"E1109: List item %d is not a List\0",
        )
    });
static e_list_item_nr_does_not_contain_3_numbers: GlobalCell<[::core::ffi::c_char; 47]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 47], [::core::ffi::c_char; 47]>(
            *b"E1110: List item %d does not contain 3 numbers\0",
        )
    });
static e_list_item_nr_range_invalid: GlobalCell<[::core::ffi::c_char; 34]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
            *b"E1111: List item %d range invalid\0",
        )
    });
static e_list_item_nr_cell_width_invalid: GlobalCell<[::core::ffi::c_char; 39]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
            *b"E1112: List item %d cell width invalid\0",
        )
    });
static e_overlapping_ranges_for_nr: GlobalCell<[::core::ffi::c_char; 36]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
            *b"E1113: Overlapping ranges for 0x%lx\0",
        )
    });
static e_only_values_of_0x80_and_higher_supported: GlobalCell<[::core::ffi::c_char; 48]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 48], [::core::ffi::c_char; 48]>(
            *b"E1114: Only values of 0x80 and higher supported\0",
        )
    });
#[unsafe(no_mangle)]
pub static utf8len_tab: GlobalCell<[uint8_t; 256]> = GlobalCell::new([
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    5 as uint8_t,
    5 as uint8_t,
    5 as uint8_t,
    5 as uint8_t,
    6 as uint8_t,
    6 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
]);
pub static utf8len_tab_zero: GlobalCell<[uint8_t; 256]> = GlobalCell::new([
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    3 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    4 as uint8_t,
    5 as uint8_t,
    5 as uint8_t,
    5 as uint8_t,
    5 as uint8_t,
    6 as uint8_t,
    6 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
]);
static enc_canon_table: GlobalCell<[C2Rust_Unnamed_21; 59]> = GlobalCell::new([
    C2Rust_Unnamed_21 {
        name: b"latin1\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int + ENC_LATIN1 as ::core::ffi::c_int,
        codepage: 1252 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-2\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-3\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-4\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-5\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-6\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-7\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-8\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-9\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-10\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-11\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-13\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-14\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-15\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int + ENC_LATIN9 as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"koi8-r\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"koi8-u\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"utf-8\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_UNICODE as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"ucs-2\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_UNICODE as ::core::ffi::c_int
            + ENC_ENDIAN_B as ::core::ffi::c_int
            + ENC_2BYTE as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"ucs-2le\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_UNICODE as ::core::ffi::c_int
            + ENC_ENDIAN_L as ::core::ffi::c_int
            + ENC_2BYTE as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"utf-16\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_UNICODE as ::core::ffi::c_int
            + ENC_ENDIAN_B as ::core::ffi::c_int
            + ENC_2WORD as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"utf-16le\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_UNICODE as ::core::ffi::c_int
            + ENC_ENDIAN_L as ::core::ffi::c_int
            + ENC_2WORD as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"ucs-4\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_UNICODE as ::core::ffi::c_int
            + ENC_ENDIAN_B as ::core::ffi::c_int
            + ENC_4BYTE as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"ucs-4le\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_UNICODE as ::core::ffi::c_int
            + ENC_ENDIAN_L as ::core::ffi::c_int
            + ENC_4BYTE as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"debug\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS as ::core::ffi::c_int,
        codepage: DBCS_DEBUG,
    },
    C2Rust_Unnamed_21 {
        name: b"euc-jp\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS as ::core::ffi::c_int,
        codepage: DBCS_JPNU,
    },
    C2Rust_Unnamed_21 {
        name: b"sjis\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS as ::core::ffi::c_int,
        codepage: DBCS_JPN,
    },
    C2Rust_Unnamed_21 {
        name: b"euc-kr\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS as ::core::ffi::c_int,
        codepage: DBCS_KORU,
    },
    C2Rust_Unnamed_21 {
        name: b"euc-cn\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS as ::core::ffi::c_int,
        codepage: DBCS_CHSU,
    },
    C2Rust_Unnamed_21 {
        name: b"euc-tw\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS as ::core::ffi::c_int,
        codepage: DBCS_CHTU,
    },
    C2Rust_Unnamed_21 {
        name: b"big5\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS as ::core::ffi::c_int,
        codepage: DBCS_CHT,
    },
    C2Rust_Unnamed_21 {
        name: b"cp437\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 437 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp737\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 737 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp775\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 775 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp850\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 850 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp852\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 852 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp855\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 855 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp857\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 857 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp860\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 860 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp861\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 861 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp862\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 862 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp863\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 863 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp865\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 865 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp866\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 866 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp869\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 869 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp874\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 874 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp932\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS as ::core::ffi::c_int,
        codepage: DBCS_JPN,
    },
    C2Rust_Unnamed_21 {
        name: b"cp936\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS as ::core::ffi::c_int,
        codepage: DBCS_CHS,
    },
    C2Rust_Unnamed_21 {
        name: b"cp949\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS as ::core::ffi::c_int,
        codepage: DBCS_KOR,
    },
    C2Rust_Unnamed_21 {
        name: b"cp950\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS as ::core::ffi::c_int,
        codepage: DBCS_CHT,
    },
    C2Rust_Unnamed_21 {
        name: b"cp1250\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 1250 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp1251\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 1251 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp1253\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 1253 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp1254\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 1254 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp1255\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 1255 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp1256\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 1256 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp1257\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 1257 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp1258\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 1258 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"macroman\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int + ENC_MACROMAN as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"hp-roman8\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT as ::core::ffi::c_int,
        codepage: 0 as ::core::ffi::c_int,
    },
]);
pub const IDX_LATIN_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const IDX_ISO_2: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const IDX_ISO_3: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const IDX_ISO_4: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const IDX_ISO_5: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const IDX_ISO_6: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const IDX_ISO_7: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const IDX_ISO_8: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const IDX_ISO_9: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const IDX_ISO_10: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const IDX_ISO_11: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const IDX_ISO_13: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const IDX_ISO_14: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const IDX_ISO_15: ::core::ffi::c_int = 13 as ::core::ffi::c_int;
pub const IDX_UTF8: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const IDX_UCS2: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub const IDX_UCS2LE: ::core::ffi::c_int = 18 as ::core::ffi::c_int;
pub const IDX_UTF16: ::core::ffi::c_int = 19 as ::core::ffi::c_int;
pub const IDX_UTF16LE: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const IDX_UCS4: ::core::ffi::c_int = 21 as ::core::ffi::c_int;
pub const IDX_UCS4LE: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const IDX_EUC_JP: ::core::ffi::c_int = 24 as ::core::ffi::c_int;
pub const IDX_SJIS: ::core::ffi::c_int = 25 as ::core::ffi::c_int;
pub const IDX_EUC_KR: ::core::ffi::c_int = 26 as ::core::ffi::c_int;
pub const IDX_EUC_CN: ::core::ffi::c_int = 27 as ::core::ffi::c_int;
pub const IDX_EUC_TW: ::core::ffi::c_int = 28 as ::core::ffi::c_int;
pub const IDX_BIG5: ::core::ffi::c_int = 29 as ::core::ffi::c_int;
pub const IDX_CP932: ::core::ffi::c_int = 45 as ::core::ffi::c_int;
pub const IDX_CP936: ::core::ffi::c_int = 46 as ::core::ffi::c_int;
pub const IDX_CP949: ::core::ffi::c_int = 47 as ::core::ffi::c_int;
pub const IDX_CP950: ::core::ffi::c_int = 48 as ::core::ffi::c_int;
pub const IDX_MACROMAN: ::core::ffi::c_int = 57 as ::core::ffi::c_int;
pub const IDX_COUNT: ::core::ffi::c_int = 59 as ::core::ffi::c_int;
static enc_alias_table: GlobalCell<[C2Rust_Unnamed_22; 64]> = GlobalCell::new([
    C2Rust_Unnamed_22 {
        name: b"ansi\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_LATIN_1,
    },
    C2Rust_Unnamed_22 {
        name: b"iso-8859-1\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_LATIN_1,
    },
    C2Rust_Unnamed_22 {
        name: b"latin2\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_2,
    },
    C2Rust_Unnamed_22 {
        name: b"latin3\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_3,
    },
    C2Rust_Unnamed_22 {
        name: b"latin4\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_4,
    },
    C2Rust_Unnamed_22 {
        name: b"cyrillic\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_5,
    },
    C2Rust_Unnamed_22 {
        name: b"arabic\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_6,
    },
    C2Rust_Unnamed_22 {
        name: b"greek\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_7,
    },
    C2Rust_Unnamed_22 {
        name: b"hebrew\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_8,
    },
    C2Rust_Unnamed_22 {
        name: b"latin5\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_9,
    },
    C2Rust_Unnamed_22 {
        name: b"turkish\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_9,
    },
    C2Rust_Unnamed_22 {
        name: b"latin6\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_10,
    },
    C2Rust_Unnamed_22 {
        name: b"nordic\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_10,
    },
    C2Rust_Unnamed_22 {
        name: b"thai\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_11,
    },
    C2Rust_Unnamed_22 {
        name: b"latin7\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_13,
    },
    C2Rust_Unnamed_22 {
        name: b"latin8\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_14,
    },
    C2Rust_Unnamed_22 {
        name: b"latin9\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_ISO_15,
    },
    C2Rust_Unnamed_22 {
        name: b"utf8\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UTF8,
    },
    C2Rust_Unnamed_22 {
        name: b"unicode\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS2,
    },
    C2Rust_Unnamed_22 {
        name: b"ucs2\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS2,
    },
    C2Rust_Unnamed_22 {
        name: b"ucs2be\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS2,
    },
    C2Rust_Unnamed_22 {
        name: b"ucs-2be\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS2,
    },
    C2Rust_Unnamed_22 {
        name: b"ucs2le\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS2LE,
    },
    C2Rust_Unnamed_22 {
        name: b"utf16\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UTF16,
    },
    C2Rust_Unnamed_22 {
        name: b"utf16be\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UTF16,
    },
    C2Rust_Unnamed_22 {
        name: b"utf-16be\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UTF16,
    },
    C2Rust_Unnamed_22 {
        name: b"utf16le\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UTF16LE,
    },
    C2Rust_Unnamed_22 {
        name: b"ucs4\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS4,
    },
    C2Rust_Unnamed_22 {
        name: b"ucs4be\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS4,
    },
    C2Rust_Unnamed_22 {
        name: b"ucs-4be\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS4,
    },
    C2Rust_Unnamed_22 {
        name: b"ucs4le\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS4LE,
    },
    C2Rust_Unnamed_22 {
        name: b"utf32\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS4,
    },
    C2Rust_Unnamed_22 {
        name: b"utf-32\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS4,
    },
    C2Rust_Unnamed_22 {
        name: b"utf32be\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS4,
    },
    C2Rust_Unnamed_22 {
        name: b"utf-32be\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS4,
    },
    C2Rust_Unnamed_22 {
        name: b"utf32le\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS4LE,
    },
    C2Rust_Unnamed_22 {
        name: b"utf-32le\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_UCS4LE,
    },
    C2Rust_Unnamed_22 {
        name: b"932\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_CP932,
    },
    C2Rust_Unnamed_22 {
        name: b"949\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_CP949,
    },
    C2Rust_Unnamed_22 {
        name: b"936\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_CP936,
    },
    C2Rust_Unnamed_22 {
        name: b"gbk\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_CP936,
    },
    C2Rust_Unnamed_22 {
        name: b"950\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_CP950,
    },
    C2Rust_Unnamed_22 {
        name: b"eucjp\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_JP,
    },
    C2Rust_Unnamed_22 {
        name: b"unix-jis\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_JP,
    },
    C2Rust_Unnamed_22 {
        name: b"ujis\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_JP,
    },
    C2Rust_Unnamed_22 {
        name: b"shift-jis\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_SJIS,
    },
    C2Rust_Unnamed_22 {
        name: b"pck\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_SJIS,
    },
    C2Rust_Unnamed_22 {
        name: b"euckr\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_KR,
    },
    C2Rust_Unnamed_22 {
        name: b"5601\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_KR,
    },
    C2Rust_Unnamed_22 {
        name: b"euccn\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_CN,
    },
    C2Rust_Unnamed_22 {
        name: b"gb2312\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_CN,
    },
    C2Rust_Unnamed_22 {
        name: b"euctw\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_TW,
    },
    C2Rust_Unnamed_22 {
        name: b"japan\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_JP,
    },
    C2Rust_Unnamed_22 {
        name: b"korea\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_KR,
    },
    C2Rust_Unnamed_22 {
        name: b"prc\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_CN,
    },
    C2Rust_Unnamed_22 {
        name: b"zh-cn\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_CN,
    },
    C2Rust_Unnamed_22 {
        name: b"chinese\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_CN,
    },
    C2Rust_Unnamed_22 {
        name: b"zh-tw\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_TW,
    },
    C2Rust_Unnamed_22 {
        name: b"taiwan\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_EUC_TW,
    },
    C2Rust_Unnamed_22 {
        name: b"cp950\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_BIG5,
    },
    C2Rust_Unnamed_22 {
        name: b"950\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_BIG5,
    },
    C2Rust_Unnamed_22 {
        name: b"mac\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_MACROMAN,
    },
    C2Rust_Unnamed_22 {
        name: b"mac-roman\0".as_ptr() as *const ::core::ffi::c_char,
        canon: IDX_MACROMAN,
    },
    C2Rust_Unnamed_22 {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        canon: 0 as ::core::ffi::c_int,
    },
]);
unsafe extern "C" fn enc_canon_search(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < IDX_COUNT {
        if strcmp(name, (*enc_canon_table.ptr())[i as usize].name) == 0 as ::core::ffi::c_int {
            return i;
        }
        i += 1;
    }
    return -1 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn enc_canon_props(
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = enc_canon_search(name);
    if i >= 0 as ::core::ffi::c_int {
        return (*enc_canon_table.ptr())[i as usize].prop;
    } else if strncmp(
        name,
        b"2byte-\0".as_ptr() as *const ::core::ffi::c_char,
        6 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        return ENC_DBCS as ::core::ffi::c_int;
    } else if strncmp(
        name,
        b"8bit-\0".as_ptr() as *const ::core::ffi::c_char,
        5 as size_t,
    ) == 0 as ::core::ffi::c_int
        || strncmp(
            name,
            b"iso-8859-\0".as_ptr() as *const ::core::ffi::c_char,
            9 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        return ENC_8BIT as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn bomb_size() -> ::core::ffi::c_int {
    let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*curbuf.get()).b_p_bomb != 0 && (*curbuf.get()).b_p_bin == 0 {
        if *(*curbuf.get()).b_p_fenc as ::core::ffi::c_int == NUL
            || strcmp(
                (*curbuf.get()).b_p_fenc,
                b"utf-8\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
        {
            n = 3 as ::core::ffi::c_int;
        } else if strncmp(
            (*curbuf.get()).b_p_fenc,
            b"ucs-2\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
            || strncmp(
                (*curbuf.get()).b_p_fenc,
                b"utf-16\0".as_ptr() as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            n = 2 as ::core::ffi::c_int;
        } else if strncmp(
            (*curbuf.get()).b_p_fenc,
            b"ucs-4\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            n = 4 as ::core::ffi::c_int;
        }
    }
    return n;
}
pub unsafe extern "C" fn remove_bom(mut s: *mut ::core::ffi::c_char) {
    let mut p: *mut ::core::ffi::c_char = s;
    loop {
        p = strchr(p, 0xef as ::core::ffi::c_int);
        if p.is_null() {
            break;
        }
        if *p.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            == 0xbb as ::core::ffi::c_int
            && *p.offset(2 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                == 0xbf as ::core::ffi::c_int
        {
            memmove(
                p as *mut ::core::ffi::c_void,
                p.offset(3 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                strlen(p.offset(3 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
            );
        } else {
            p = p.offset(1);
        }
    }
}
pub unsafe extern "C" fn mb_get_class(mut p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    return mb_get_class_tab(p, &raw mut (*curbuf.get()).b_chartab as *mut uint64_t);
}
pub unsafe extern "C" fn mb_get_class_tab(
    mut p: *const ::core::ffi::c_char,
    chartab: *const uint64_t,
) -> ::core::ffi::c_int {
    if (*utf8len_tab.ptr())[*p.offset(0 as ::core::ffi::c_int as isize) as uint8_t as usize]
        as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
    {
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            || ascii_iswhite(*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
        {
            return 0 as ::core::ffi::c_int;
        }
        if vim_iswordc_tab(
            *p.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int,
            chartab,
        ) {
            return 2 as ::core::ffi::c_int;
        }
        return 1 as ::core::ffi::c_int;
    }
    return utf_class_tab(utf_ptr2char(p), chartab);
}
unsafe extern "C" fn prop_is_emojilike(mut prop: *const utf8proc_property_t) -> bool {
    return (*prop).boundclass as ::core::ffi::c_int
        == UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC as ::core::ffi::c_int
        || (*prop).boundclass as ::core::ffi::c_int
            == UTF8PROC_BOUNDCLASS_REGIONAL_INDICATOR as ::core::ffi::c_int;
}
pub unsafe extern "C" fn utf_char2cells(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if c < 0x80 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    if !vim_isprintc(c) {
        assert!(c <= 0xffff as ::core::ffi::c_int, "c <= 0xFFFF");
        return if c > 0xff as ::core::ffi::c_int {
            6 as ::core::ffi::c_int
        } else {
            4 as ::core::ffi::c_int
        };
    }
    let mut n: ::core::ffi::c_int = cw_value(c);
    if n != 0 as ::core::ffi::c_int {
        return n;
    }
    let mut prop: *const utf8proc_property_t = utf8proc_get_property(c as utf8proc_int32_t);
    if (*prop).charwidth as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
        return 2 as ::core::ffi::c_int;
    }
    if *p_ambw.get() as ::core::ffi::c_int == 'd' as ::core::ffi::c_int && (*prop).ambiguous_width {
        return 2 as ::core::ffi::c_int;
    }
    if p_emoji.get() != 0
        && c >= 0x1f000 as ::core::ffi::c_int
        && !(*prop).ambiguous_width
        && prop_is_emojilike(prop) as ::core::ffi::c_int != 0
    {
        return 2 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn utf_ptr2cells(mut p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut p: *const uint8_t = p_in as *const uint8_t;
    if *p as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int {
        let mut len: ::core::ffi::c_int = (*utf8len_tab.ptr())[*p as usize] as ::core::ffi::c_int;
        let mut c: int32_t = utf_ptr2CharInfo_impl(p, len as uintptr_t);
        if c <= 0 as int32_t {
            return 4 as ::core::ffi::c_int;
        }
        if c < 0x80 as int32_t {
            return char2cells(c as ::core::ffi::c_int);
        }
        let mut cells: ::core::ffi::c_int = utf_char2cells(c as ::core::ffi::c_int);
        if cells == 1 as ::core::ffi::c_int
            && p_emoji.get() != 0
            && prop_is_emojilike(utf8proc_get_property(c as utf8proc_int32_t)) as ::core::ffi::c_int
                != 0
        {
            let mut c2: ::core::ffi::c_int = utf_ptr2char(p_in.offset(len as isize));
            if c2 == 0xfe0f as ::core::ffi::c_int {
                return 2 as ::core::ffi::c_int;
            }
        }
        return cells;
    }
    return 1 as ::core::ffi::c_int;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf_ptr2CharInfo_impl(mut p: *const uint8_t, len: uintptr_t) -> int32_t {
    let corr: uint32_t = (*corrections.ptr())[len as usize];
    let mut cur: uint8_t = 0;
    cur = *p.offset(1 as ::core::ffi::c_int as isize);
    let mut code_point: uint32_t = ((*p.offset(0 as ::core::ffi::c_int as isize) as uint32_t)
        << 6 as ::core::ffi::c_int)
        .wrapping_add(cur as uint32_t);
    if ((cur as ::core::ffi::c_uint & 0xc0 as ::core::ffi::c_uint) as uint8_t
        as ::core::ffi::c_uint
        != 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int as ::core::ffi::c_long
        != 0
    {
        return -1 as int32_t;
    }
    if (len as uint32_t) >= 3 as uint32_t {
        cur = *p.offset(2 as ::core::ffi::c_int as isize);
        code_point = (code_point << 6 as ::core::ffi::c_int).wrapping_add(cur as uint32_t);
        if ((cur as ::core::ffi::c_uint & 0xc0 as ::core::ffi::c_uint) as uint8_t
            as ::core::ffi::c_uint
            != 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int as ::core::ffi::c_long
            != 0
        {
            return -1 as int32_t;
        }
        if len as uint32_t != 3 as uint32_t {
            cur = *p.offset(3 as ::core::ffi::c_int as isize);
            code_point = (code_point << 6 as ::core::ffi::c_int).wrapping_add(cur as uint32_t);
            if ((cur as ::core::ffi::c_uint & 0xc0 as ::core::ffi::c_uint) as uint8_t
                as ::core::ffi::c_uint
                != 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int
                as ::core::ffi::c_long
                != 0
            {
                return -1 as int32_t;
            }
            if len as uint32_t != 4 as uint32_t {
                cur = *p.offset(4 as ::core::ffi::c_int as isize);
                code_point = (code_point << 6 as ::core::ffi::c_int).wrapping_add(cur as uint32_t);
                if ((cur as ::core::ffi::c_uint & 0xc0 as ::core::ffi::c_uint) as uint8_t
                    as ::core::ffi::c_uint
                    != 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int
                    as ::core::ffi::c_long
                    != 0
                {
                    return -1 as int32_t;
                }
                if len as uint32_t != 5 as uint32_t {
                    cur = *p.offset(5 as ::core::ffi::c_int as isize);
                    code_point =
                        (code_point << 6 as ::core::ffi::c_int).wrapping_add(cur as uint32_t);
                    if ((cur as ::core::ffi::c_uint & 0xc0 as ::core::ffi::c_uint) as uint8_t
                        as ::core::ffi::c_uint
                        != 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int
                        as ::core::ffi::c_long
                        != 0
                    {
                        return -1 as int32_t;
                    }
                }
            }
        }
    }
    return code_point.wrapping_add(corr) as int32_t;
}
pub unsafe extern "C" fn utf_ptr2cells_len(
    mut p: *const ::core::ffi::c_char,
    mut size: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if size > 0 as ::core::ffi::c_int
        && *p as uint8_t as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int
    {
        let mut len: ::core::ffi::c_int = utf_ptr2len_len(p, size);
        if len < (*utf8len_tab.ptr())[*p as uint8_t as usize] as ::core::ffi::c_int {
            return 1 as ::core::ffi::c_int;
        }
        let mut c: ::core::ffi::c_int = utf_ptr2char(p);
        if utf_ptr2len(p) == 1 as ::core::ffi::c_int || c == NUL {
            return 4 as ::core::ffi::c_int;
        }
        if c < 0x80 as ::core::ffi::c_int {
            return char2cells(c);
        }
        let mut cells: ::core::ffi::c_int = utf_char2cells(c);
        if cells == 1 as ::core::ffi::c_int
            && p_emoji.get() != 0
            && size > len
            && prop_is_emojilike(utf8proc_get_property(c as utf8proc_int32_t)) as ::core::ffi::c_int
                != 0
            && utf_ptr2len_len(p.offset(len as isize), size - len)
                == (*utf8len_tab.ptr())[*p.offset(len as isize) as uint8_t as usize]
                    as ::core::ffi::c_int
        {
            let mut c2: ::core::ffi::c_int = utf_ptr2char(p.offset(len as isize));
            if c2 == 0xfe0f as ::core::ffi::c_int {
                return 2 as ::core::ffi::c_int;
            }
        }
        return cells;
    }
    return 1 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn mb_string2cells(mut str: *const ::core::ffi::c_char) -> size_t {
    let mut clen: size_t = 0 as size_t;
    let mut p: *const ::core::ffi::c_char = str;
    while *p as ::core::ffi::c_int != NUL {
        clen = clen.wrapping_add(utf_ptr2cells(p) as size_t);
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    return clen;
}
pub unsafe extern "C" fn mb_string2cells_len(
    mut str: *const ::core::ffi::c_char,
    mut size: size_t,
) -> size_t {
    let mut clen: size_t = 0 as size_t;
    let mut p: *const ::core::ffi::c_char = str;
    while *p as ::core::ffi::c_int != NUL && p < str.offset(size as isize) {
        clen = clen.wrapping_add(utf_ptr2cells_len(
            p,
            size as ::core::ffi::c_int - p.offset_from(str) as ::core::ffi::c_int,
        ) as size_t);
        p = p.offset(utfc_ptr2len_len(
            p,
            size as ::core::ffi::c_int - p.offset_from(str) as ::core::ffi::c_int,
        ) as isize);
    }
    return clen;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut p: *mut uint8_t = p_in as *mut uint8_t;
    let v0: uint32_t = *p.offset(0 as ::core::ffi::c_int as isize) as uint32_t;
    if (v0 < 0x80 as uint32_t) as ::core::ffi::c_int as ::core::ffi::c_long != 0 {
        return v0 as ::core::ffi::c_int;
    }
    let len: uint8_t = (*utf8len_tab.ptr())[v0 as usize];
    if ((len as ::core::ffi::c_int) < 2 as ::core::ffi::c_int) as ::core::ffi::c_int
        as ::core::ffi::c_long
        != 0
    {
        return v0 as ::core::ffi::c_int;
    }
    let v1: uint32_t = *p.offset(1 as ::core::ffi::c_int as isize) as uint32_t;
    if ((v1 & 0xc0 as uint32_t) as uint8_t as ::core::ffi::c_uint != 0x80 as ::core::ffi::c_uint)
        as ::core::ffi::c_int as ::core::ffi::c_long
        != 0
    {
        return v0 as ::core::ffi::c_int;
    }
    if len as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
        return (v0 << 6 as ::core::ffi::c_int)
            .wrapping_add(v1)
            .wrapping_sub(
                ((0xc0 as uint32_t) << 6 as ::core::ffi::c_int).wrapping_add(
                    (0x80 as ::core::ffi::c_uint as uint32_t) << 0 as ::core::ffi::c_int,
                ),
            ) as ::core::ffi::c_int;
    }
    let v2: uint32_t = *p.offset(2 as ::core::ffi::c_int as isize) as uint32_t;
    if ((v2 & 0xc0 as uint32_t) as uint8_t as ::core::ffi::c_uint != 0x80 as ::core::ffi::c_uint)
        as ::core::ffi::c_int as ::core::ffi::c_long
        != 0
    {
        return v0 as ::core::ffi::c_int;
    }
    if len as ::core::ffi::c_int == 3 as ::core::ffi::c_int {
        return (v0 << 12 as ::core::ffi::c_int)
            .wrapping_add(v1 << 6 as ::core::ffi::c_int)
            .wrapping_add(v2)
            .wrapping_sub(
                ((0xe0 as uint32_t) << 12 as ::core::ffi::c_int)
                    .wrapping_add(
                        (0x80 as ::core::ffi::c_uint as uint32_t) << 6 as ::core::ffi::c_int,
                    )
                    .wrapping_add(
                        (0x80 as ::core::ffi::c_uint as uint32_t) << 0 as ::core::ffi::c_int,
                    ),
            ) as ::core::ffi::c_int;
    }
    let v3: uint32_t = *p.offset(3 as ::core::ffi::c_int as isize) as uint32_t;
    if ((v3 & 0xc0 as uint32_t) as uint8_t as ::core::ffi::c_uint != 0x80 as ::core::ffi::c_uint)
        as ::core::ffi::c_int as ::core::ffi::c_long
        != 0
    {
        return v0 as ::core::ffi::c_int;
    }
    if len as ::core::ffi::c_int == 4 as ::core::ffi::c_int {
        return (v0 << 18 as ::core::ffi::c_int)
            .wrapping_add(v1 << 12 as ::core::ffi::c_int)
            .wrapping_add(v2 << 6 as ::core::ffi::c_int)
            .wrapping_add(v3)
            .wrapping_sub(
                ((0xf0 as uint32_t) << 18 as ::core::ffi::c_int)
                    .wrapping_add(
                        (0x80 as ::core::ffi::c_uint as uint32_t) << 12 as ::core::ffi::c_int,
                    )
                    .wrapping_add(
                        (0x80 as ::core::ffi::c_uint as uint32_t) << 6 as ::core::ffi::c_int,
                    )
                    .wrapping_add(
                        (0x80 as ::core::ffi::c_uint as uint32_t) << 0 as ::core::ffi::c_int,
                    ),
            ) as ::core::ffi::c_int;
    }
    let v4: uint32_t = *p.offset(4 as ::core::ffi::c_int as isize) as uint32_t;
    if ((v4 & 0xc0 as uint32_t) as uint8_t as ::core::ffi::c_uint != 0x80 as ::core::ffi::c_uint)
        as ::core::ffi::c_int as ::core::ffi::c_long
        != 0
    {
        return v0 as ::core::ffi::c_int;
    }
    if len as ::core::ffi::c_int == 5 as ::core::ffi::c_int {
        return (v0 << 24 as ::core::ffi::c_int)
            .wrapping_add(v1 << 18 as ::core::ffi::c_int)
            .wrapping_add(v2 << 12 as ::core::ffi::c_int)
            .wrapping_add(v3 << 6 as ::core::ffi::c_int)
            .wrapping_add(v4)
            .wrapping_sub(
                ((0xf8 as uint32_t) << 24 as ::core::ffi::c_int)
                    .wrapping_add(
                        (0x80 as ::core::ffi::c_uint as uint32_t) << 18 as ::core::ffi::c_int,
                    )
                    .wrapping_add(
                        (0x80 as ::core::ffi::c_uint as uint32_t) << 12 as ::core::ffi::c_int,
                    )
                    .wrapping_add(
                        (0x80 as ::core::ffi::c_uint as uint32_t) << 6 as ::core::ffi::c_int,
                    )
                    .wrapping_add(
                        (0x80 as ::core::ffi::c_uint as uint32_t) << 0 as ::core::ffi::c_int,
                    ),
            ) as ::core::ffi::c_int;
    }
    let v5: uint32_t = *p.offset(5 as ::core::ffi::c_int as isize) as uint32_t;
    if ((v5 & 0xc0 as uint32_t) as uint8_t as ::core::ffi::c_uint != 0x80 as ::core::ffi::c_uint)
        as ::core::ffi::c_int as ::core::ffi::c_long
        != 0
    {
        return v0 as ::core::ffi::c_int;
    }
    return (v0 << 30 as ::core::ffi::c_int)
        .wrapping_add(v1 << 24 as ::core::ffi::c_int)
        .wrapping_add(v2 << 18 as ::core::ffi::c_int)
        .wrapping_add(v3 << 12 as ::core::ffi::c_int)
        .wrapping_add(v4 << 6 as ::core::ffi::c_int)
        .wrapping_add(v5)
        .wrapping_sub(
            ((0x80 as ::core::ffi::c_uint as uint32_t) << 24 as ::core::ffi::c_int)
                .wrapping_add((0x80 as ::core::ffi::c_uint as uint32_t) << 18 as ::core::ffi::c_int)
                .wrapping_add((0x80 as ::core::ffi::c_uint as uint32_t) << 12 as ::core::ffi::c_int)
                .wrapping_add((0x80 as ::core::ffi::c_uint as uint32_t) << 6 as ::core::ffi::c_int)
                .wrapping_add((0x80 as ::core::ffi::c_uint as uint32_t) << 0 as ::core::ffi::c_int),
        ) as ::core::ffi::c_int;
}
unsafe extern "C" fn utf_safe_read_char_adv(
    mut s: *mut *const ::core::ffi::c_char,
    mut n: *mut size_t,
) -> ::core::ffi::c_int {
    if *n == 0 as size_t {
        return 0 as ::core::ffi::c_int;
    }
    let mut k: uint8_t = (*utf8len_tab_zero.ptr())[**s as uint8_t as usize];
    if k as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        *n = (*n).wrapping_sub(1);
        let c2rust_fresh0 = *s;
        *s = (*s).offset(1);
        return *c2rust_fresh0 as uint8_t as ::core::ffi::c_int;
    }
    if k as size_t <= *n {
        let mut c: ::core::ffi::c_int = utf_ptr2char(*s);
        if c != **s as uint8_t as ::core::ffi::c_int
            || c == 0xc3 as ::core::ffi::c_int
                && *(*s).offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                    == 0x83 as ::core::ffi::c_int
        {
            *s = (*s).offset(k as ::core::ffi::c_int as isize);
            *n = (*n).wrapping_sub(k as size_t);
            return c;
        }
    }
    return -1 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn mb_ptr2char_adv(
    pp: *mut *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = utf_ptr2char(*pp);
    *pp = (*pp).offset(utfc_ptr2len(*pp) as isize);
    return c;
}
pub unsafe extern "C" fn mb_cptr2char_adv(
    mut pp: *mut *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = utf_ptr2char(*pp);
    *pp = (*pp).offset(utf_ptr2len(*pp) as isize);
    return c;
}
pub unsafe extern "C" fn utf_iscomposing_first(mut c: ::core::ffi::c_int) -> bool {
    return c >= 128 as ::core::ffi::c_int
        && !utf8proc_grapheme_break(' ' as utf8proc_int32_t, c as utf8proc_int32_t);
}
pub unsafe extern "C" fn utf_composinglike(
    mut p1: *const ::core::ffi::c_char,
    mut p2: *const ::core::ffi::c_char,
    mut state: *mut GraphemeState,
) -> bool {
    if (*p2 as uint8_t as ::core::ffi::c_int) < 128 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    let mut first: ::core::ffi::c_int = utf_ptr2char(p1);
    let mut second: ::core::ffi::c_int = utf_ptr2char(p2);
    if !utf8proc_grapheme_break_stateful(
        first as utf8proc_int32_t,
        second as utf8proc_int32_t,
        state.as_mut(),
    ) {
        return true_0 != 0;
    }
    return crate::src::nvim::arabic::arabic_combine(first, second);
}
pub unsafe extern "C" fn utf_iscomposing(
    mut c1: ::core::ffi::c_int,
    mut c2: ::core::ffi::c_int,
    mut state: *mut GraphemeState,
) -> bool {
    return !utf8proc_grapheme_break_stateful(
        c1 as utf8proc_int32_t,
        c2 as utf8proc_int32_t,
        state.as_mut(),
    ) || crate::src::nvim::arabic::arabic_combine(c1, c2) as ::core::ffi::c_int != 0;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn utfc_ptr2schar(
    mut p: *const ::core::ffi::c_char,
    mut firstc: *mut ::core::ffi::c_int,
) -> schar_T {
    let mut c: ::core::ffi::c_int = utf_ptr2char(p);
    *firstc = c;
    let mut first_compose: bool = utf_iscomposing_first(c);
    let mut maxlen: size_t =
        (MAX_SCHAR_SIZE - 1 as ::core::ffi::c_int - first_compose as ::core::ffi::c_int) as size_t;
    let mut len: size_t = utfc_ptr2len_len(p, maxlen as ::core::ffi::c_int) as size_t;
    if len == 1 as size_t && *p as uint8_t as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int {
        return 0 as schar_T;
    }
    return schar_from_buf_first(p, len, first_compose);
}
pub unsafe extern "C" fn utfc_ptrlen2schar(
    mut p: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut firstc: *mut ::core::ffi::c_int,
) -> schar_T {
    if len == 1 as ::core::ffi::c_int
        && *p as uint8_t as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int
        || len == 0 as ::core::ffi::c_int
    {
        *firstc = *p as uint8_t as ::core::ffi::c_int;
        return 0 as schar_T;
    }
    let mut c: ::core::ffi::c_int = utf_ptr2char(p);
    *firstc = c;
    let mut first_compose: bool = utf_iscomposing_first(c);
    let mut maxlen: ::core::ffi::c_int =
        MAX_SCHAR_SIZE - 1 as ::core::ffi::c_int - first_compose as ::core::ffi::c_int;
    if len > maxlen {
        len = utfc_ptr2len_len(p, maxlen);
    }
    return schar_from_buf_first(p, len as size_t, first_compose);
}
unsafe extern "C" fn schar_from_buf_first(
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut first_compose: bool,
) -> schar_T {
    if first_compose {
        let mut cbuf: [::core::ffi::c_char; 32] = [0; 32];
        cbuf[0 as ::core::ffi::c_int as usize] = ' ' as ::core::ffi::c_char;
        memcpy(
            (&raw mut cbuf as *mut ::core::ffi::c_char).offset(1 as ::core::ffi::c_int as isize)
                as *mut ::core::ffi::c_void,
            buf as *const ::core::ffi::c_void,
            len,
        );
        return schar_from_buf(
            &raw mut cbuf as *mut ::core::ffi::c_char,
            len.wrapping_add(1 as size_t),
        );
    } else {
        return schar_from_buf(buf, len);
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf_ptr2len(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut p: *mut uint8_t = p_in as *mut uint8_t;
    if *p as ::core::ffi::c_int == NUL {
        return 0 as ::core::ffi::c_int;
    }
    let len: ::core::ffi::c_int = (*utf8len_tab.ptr())[*p as usize] as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i < len {
        if *p.offset(i as isize) as ::core::ffi::c_int & 0xc0 as ::core::ffi::c_int
            != 0x80 as ::core::ffi::c_int
        {
            return 1 as ::core::ffi::c_int;
        }
        i += 1;
    }
    return len;
}
pub unsafe extern "C" fn utf_byte2len(mut b: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return (*utf8len_tab.ptr())[b as usize] as ::core::ffi::c_int;
}
pub unsafe extern "C" fn utf_ptr2len_len(
    mut p: *const ::core::ffi::c_char,
    mut size: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut m: ::core::ffi::c_int = 0;
    let mut len: ::core::ffi::c_int =
        (*utf8len_tab.ptr())[*p as uint8_t as usize] as ::core::ffi::c_int;
    if len == 1 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    if len > size {
        m = size;
    } else {
        m = len;
    }
    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i < m {
        if *p.offset(i as isize) as ::core::ffi::c_int & 0xc0 as ::core::ffi::c_int
            != 0x80 as ::core::ffi::c_int
        {
            return 1 as ::core::ffi::c_int;
        }
        i += 1;
    }
    return len;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut b0: uint8_t = *p as uint8_t;
    if b0 as ::core::ffi::c_int == NUL {
        return 0 as ::core::ffi::c_int;
    }
    if (b0 as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int
        && (*p.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int)
            < 0x80 as ::core::ffi::c_int
    {
        return 1 as ::core::ffi::c_int;
    }
    let mut len: ::core::ffi::c_int = utf_ptr2len(p);
    if len == 1 as ::core::ffi::c_int && b0 as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    let mut prevlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
    loop {
        if (*p.offset(len as isize) as uint8_t as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int
            || !utf_composinglike(
                p.offset(prevlen as isize),
                p.offset(len as isize),
                &raw mut state,
            )
        {
            return len;
        }
        prevlen = len;
        len += utf_ptr2len(p.offset(len as isize));
    }
}
pub unsafe extern "C" fn utfc_ptr2len_len(
    mut p: *const ::core::ffi::c_char,
    mut size: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if size < 1 as ::core::ffi::c_int || *p as ::core::ffi::c_int == NUL {
        return 0 as ::core::ffi::c_int;
    }
    if (*p.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int)
        < 0x80 as ::core::ffi::c_int
        && (size == 1 as ::core::ffi::c_int
            || (*p.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int)
                < 0x80 as ::core::ffi::c_int)
    {
        return 1 as ::core::ffi::c_int;
    }
    let mut len: ::core::ffi::c_int = utf_ptr2len_len(p, size);
    if len == 1 as ::core::ffi::c_int
        && *p.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            >= 0x80 as ::core::ffi::c_int
        || len > size
    {
        return 1 as ::core::ffi::c_int;
    }
    let mut prevlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
    while len < size {
        if (*p.offset(len as isize) as uint8_t as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
            break;
        }
        let mut len_next_char: ::core::ffi::c_int =
            utf_ptr2len_len(p.offset(len as isize), size - len);
        if len_next_char > size - len {
            break;
        }
        if !utf_composinglike(
            p.offset(prevlen as isize),
            p.offset(len as isize),
            &raw mut state,
        ) {
            break;
        }
        prevlen = len;
        len += len_next_char;
    }
    return len;
}
pub fn utf_char2len(c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if c < 0x80 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    } else if c < 0x800 as ::core::ffi::c_int {
        return 2 as ::core::ffi::c_int;
    } else if c < 0x10000 as ::core::ffi::c_int {
        return 3 as ::core::ffi::c_int;
    } else if c < 0x200000 as ::core::ffi::c_int {
        return 4 as ::core::ffi::c_int;
    } else if c < 0x4000000 as ::core::ffi::c_int {
        return 5 as ::core::ffi::c_int;
    } else {
        return 6 as ::core::ffi::c_int;
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf_char2bytes(
    c: ::core::ffi::c_int,
    buf: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if c < 0x80 as ::core::ffi::c_int {
        *buf.offset(0 as ::core::ffi::c_int as isize) = c as ::core::ffi::c_char;
        return 1 as ::core::ffi::c_int;
    } else if c < 0x800 as ::core::ffi::c_int {
        *buf.offset(0 as ::core::ffi::c_int as isize) = (0xc0 as ::core::ffi::c_uint)
            .wrapping_add(c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int)
            as ::core::ffi::c_char;
        *buf.offset(1 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint)
            .wrapping_add(c as ::core::ffi::c_uint & 0x3f as ::core::ffi::c_uint)
            as ::core::ffi::c_char;
        return 2 as ::core::ffi::c_int;
    } else if c < 0x10000 as ::core::ffi::c_int {
        *buf.offset(0 as ::core::ffi::c_int as isize) = (0xe0 as ::core::ffi::c_uint)
            .wrapping_add(c as ::core::ffi::c_uint >> 12 as ::core::ffi::c_int)
            as ::core::ffi::c_char;
        *buf.offset(1 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint).wrapping_add(
            c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_uint,
        ) as ::core::ffi::c_char;
        *buf.offset(2 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint)
            .wrapping_add(c as ::core::ffi::c_uint & 0x3f as ::core::ffi::c_uint)
            as ::core::ffi::c_char;
        return 3 as ::core::ffi::c_int;
    } else if c < 0x200000 as ::core::ffi::c_int {
        *buf.offset(0 as ::core::ffi::c_int as isize) = (0xf0 as ::core::ffi::c_uint)
            .wrapping_add(c as ::core::ffi::c_uint >> 18 as ::core::ffi::c_int)
            as ::core::ffi::c_char;
        *buf.offset(1 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint).wrapping_add(
            c as ::core::ffi::c_uint >> 12 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_uint,
        ) as ::core::ffi::c_char;
        *buf.offset(2 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint).wrapping_add(
            c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_uint,
        ) as ::core::ffi::c_char;
        *buf.offset(3 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint)
            .wrapping_add(c as ::core::ffi::c_uint & 0x3f as ::core::ffi::c_uint)
            as ::core::ffi::c_char;
        return 4 as ::core::ffi::c_int;
    } else if c < 0x4000000 as ::core::ffi::c_int {
        *buf.offset(0 as ::core::ffi::c_int as isize) = (0xf8 as ::core::ffi::c_uint)
            .wrapping_add(c as ::core::ffi::c_uint >> 24 as ::core::ffi::c_int)
            as ::core::ffi::c_char;
        *buf.offset(1 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint).wrapping_add(
            c as ::core::ffi::c_uint >> 18 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_uint,
        ) as ::core::ffi::c_char;
        *buf.offset(2 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint).wrapping_add(
            c as ::core::ffi::c_uint >> 12 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_uint,
        ) as ::core::ffi::c_char;
        *buf.offset(3 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint).wrapping_add(
            c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_uint,
        ) as ::core::ffi::c_char;
        *buf.offset(4 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint)
            .wrapping_add(c as ::core::ffi::c_uint & 0x3f as ::core::ffi::c_uint)
            as ::core::ffi::c_char;
        return 5 as ::core::ffi::c_int;
    } else {
        *buf.offset(0 as ::core::ffi::c_int as isize) = (0xfc as ::core::ffi::c_uint)
            .wrapping_add(c as ::core::ffi::c_uint >> 30 as ::core::ffi::c_int)
            as ::core::ffi::c_char;
        *buf.offset(1 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint).wrapping_add(
            c as ::core::ffi::c_uint >> 24 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_uint,
        ) as ::core::ffi::c_char;
        *buf.offset(2 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint).wrapping_add(
            c as ::core::ffi::c_uint >> 18 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_uint,
        ) as ::core::ffi::c_char;
        *buf.offset(3 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint).wrapping_add(
            c as ::core::ffi::c_uint >> 12 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_uint,
        ) as ::core::ffi::c_char;
        *buf.offset(4 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint).wrapping_add(
            c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_uint,
        ) as ::core::ffi::c_char;
        *buf.offset(5 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint)
            .wrapping_add(c as ::core::ffi::c_uint & 0x3f as ::core::ffi::c_uint)
            as ::core::ffi::c_char;
        return 6 as ::core::ffi::c_int;
    };
}
pub unsafe extern "C" fn utf_iscomposing_legacy(mut c: ::core::ffi::c_int) -> bool {
    let mut prop: *const utf8proc_property_t = utf8proc_get_property(c as utf8proc_int32_t);
    return (*prop).category as ::core::ffi::c_int == UTF8PROC_CATEGORY_MN as ::core::ffi::c_int
        || (*prop).category as ::core::ffi::c_int == UTF8PROC_CATEGORY_ME as ::core::ffi::c_int;
}
unsafe extern "C" fn intable(
    mut table: *const interval,
    mut n_items: size_t,
    mut c: ::core::ffi::c_int,
) -> bool {
    assert!(n_items > 0 as size_t, "n_items > 0");
    if c < (*table.offset(0 as ::core::ffi::c_int as isize)).first {
        return false_0 != 0;
    }
    assert!(
        n_items <= (18446744073709551615 as size_t).wrapping_div(2 as size_t),
        "n_items <= SIZE_MAX / 2"
    );
    let mut bot: size_t = 0 as size_t;
    let mut top: size_t = n_items;
    loop {
        let mut mid: size_t = bot.wrapping_add(top) >> 1 as ::core::ffi::c_int;
        if (*table.offset(mid as isize)).last < c {
            bot = mid.wrapping_add(1 as size_t);
        } else if (*table.offset(mid as isize)).first > c {
            top = mid;
        } else {
            return true_0 != 0;
        }
        if top <= bot {
            break;
        }
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn utf_printable(mut c: ::core::ffi::c_int) -> bool {
    static nonprint: GlobalCell<[interval; 9]> = GlobalCell::new([
        interval {
            first: 0x70f as ::core::ffi::c_int,
            last: 0x70f as ::core::ffi::c_int,
        },
        interval {
            first: 0x180b as ::core::ffi::c_int,
            last: 0x180e as ::core::ffi::c_int,
        },
        interval {
            first: 0x200b as ::core::ffi::c_int,
            last: 0x200f as ::core::ffi::c_int,
        },
        interval {
            first: 0x202a as ::core::ffi::c_int,
            last: 0x202e as ::core::ffi::c_int,
        },
        interval {
            first: 0x2060 as ::core::ffi::c_int,
            last: 0x206f as ::core::ffi::c_int,
        },
        interval {
            first: 0xd800 as ::core::ffi::c_int,
            last: 0xdfff as ::core::ffi::c_int,
        },
        interval {
            first: 0xfeff as ::core::ffi::c_int,
            last: 0xfeff as ::core::ffi::c_int,
        },
        interval {
            first: 0xfff9 as ::core::ffi::c_int,
            last: 0xfffb as ::core::ffi::c_int,
        },
        interval {
            first: 0xfffe as ::core::ffi::c_int,
            last: 0xffff as ::core::ffi::c_int,
        },
    ]);
    return !intable(
        (nonprint.ptr() as *const _) as *const interval,
        ::core::mem::size_of::<[interval; 9]>()
            .wrapping_div(::core::mem::size_of::<interval>())
            .wrapping_div(
                (::core::mem::size_of::<[interval; 9]>()
                    .wrapping_rem(::core::mem::size_of::<interval>())
                    == 0) as ::core::ffi::c_int as size_t,
            ),
        c,
    );
}
pub unsafe extern "C" fn utf_class(c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return utf_class_tab(c, &raw mut (*curbuf.get()).b_chartab as *mut uint64_t);
}
pub unsafe extern "C" fn utf_class_tab(
    c: ::core::ffi::c_int,
    chartab: *const uint64_t,
) -> ::core::ffi::c_int {
    static classes: GlobalCell<[clinterval; 71]> = GlobalCell::new([
        clinterval {
            first: 0x37e as ::core::ffi::c_uint,
            last: 0x37e as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x387 as ::core::ffi::c_uint,
            last: 0x387 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x55a as ::core::ffi::c_uint,
            last: 0x55f as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x589 as ::core::ffi::c_uint,
            last: 0x589 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x5be as ::core::ffi::c_uint,
            last: 0x5be as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x5c0 as ::core::ffi::c_uint,
            last: 0x5c0 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x5c3 as ::core::ffi::c_uint,
            last: 0x5c3 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x5f3 as ::core::ffi::c_uint,
            last: 0x5f4 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x60c as ::core::ffi::c_uint,
            last: 0x60c as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x61b as ::core::ffi::c_uint,
            last: 0x61b as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x61f as ::core::ffi::c_uint,
            last: 0x61f as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x66a as ::core::ffi::c_uint,
            last: 0x66d as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x6d4 as ::core::ffi::c_uint,
            last: 0x6d4 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x700 as ::core::ffi::c_uint,
            last: 0x70d as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x964 as ::core::ffi::c_uint,
            last: 0x965 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x970 as ::core::ffi::c_uint,
            last: 0x970 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xdf4 as ::core::ffi::c_uint,
            last: 0xdf4 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xe4f as ::core::ffi::c_uint,
            last: 0xe4f as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xe5a as ::core::ffi::c_uint,
            last: 0xe5b as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xf04 as ::core::ffi::c_uint,
            last: 0xf12 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xf3a as ::core::ffi::c_uint,
            last: 0xf3d as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xf85 as ::core::ffi::c_uint,
            last: 0xf85 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x104a as ::core::ffi::c_uint,
            last: 0x104f as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x10fb as ::core::ffi::c_uint,
            last: 0x10fb as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x1361 as ::core::ffi::c_uint,
            last: 0x1368 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x166d as ::core::ffi::c_uint,
            last: 0x166e as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x1680 as ::core::ffi::c_uint,
            last: 0x1680 as ::core::ffi::c_uint,
            cls: 0 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x169b as ::core::ffi::c_uint,
            last: 0x169c as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x16eb as ::core::ffi::c_uint,
            last: 0x16ed as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x1735 as ::core::ffi::c_uint,
            last: 0x1736 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x17d4 as ::core::ffi::c_uint,
            last: 0x17dc as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x1800 as ::core::ffi::c_uint,
            last: 0x180a as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x2000 as ::core::ffi::c_uint,
            last: 0x200b as ::core::ffi::c_uint,
            cls: 0 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x200c as ::core::ffi::c_uint,
            last: 0x2027 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x2028 as ::core::ffi::c_uint,
            last: 0x2029 as ::core::ffi::c_uint,
            cls: 0 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x202a as ::core::ffi::c_uint,
            last: 0x202e as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x202f as ::core::ffi::c_uint,
            last: 0x202f as ::core::ffi::c_uint,
            cls: 0 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x2030 as ::core::ffi::c_uint,
            last: 0x205e as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x205f as ::core::ffi::c_uint,
            last: 0x205f as ::core::ffi::c_uint,
            cls: 0 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x2060 as ::core::ffi::c_uint,
            last: 0x206f as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x2070 as ::core::ffi::c_uint,
            last: 0x207f as ::core::ffi::c_uint,
            cls: 0x2070 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x2080 as ::core::ffi::c_uint,
            last: 0x2094 as ::core::ffi::c_uint,
            cls: 0x2080 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x20a0 as ::core::ffi::c_uint,
            last: 0x27ff as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x2800 as ::core::ffi::c_uint,
            last: 0x28ff as ::core::ffi::c_uint,
            cls: 0x2800 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x2900 as ::core::ffi::c_uint,
            last: 0x2998 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x29d8 as ::core::ffi::c_uint,
            last: 0x29db as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x29fc as ::core::ffi::c_uint,
            last: 0x29fd as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x2e00 as ::core::ffi::c_uint,
            last: 0x2e7f as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x3000 as ::core::ffi::c_uint,
            last: 0x3000 as ::core::ffi::c_uint,
            cls: 0 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x3001 as ::core::ffi::c_uint,
            last: 0x3020 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x3030 as ::core::ffi::c_uint,
            last: 0x3030 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x303d as ::core::ffi::c_uint,
            last: 0x303d as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x3040 as ::core::ffi::c_uint,
            last: 0x309f as ::core::ffi::c_uint,
            cls: 0x3040 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x30a0 as ::core::ffi::c_uint,
            last: 0x30ff as ::core::ffi::c_uint,
            cls: 0x30a0 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x3300 as ::core::ffi::c_uint,
            last: 0x9fff as ::core::ffi::c_uint,
            cls: 0x4e00 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xac00 as ::core::ffi::c_uint,
            last: 0xd7a3 as ::core::ffi::c_uint,
            cls: 0xac00 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xf900 as ::core::ffi::c_uint,
            last: 0xfaff as ::core::ffi::c_uint,
            cls: 0x4e00 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xfd3e as ::core::ffi::c_uint,
            last: 0xfd3f as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xfe30 as ::core::ffi::c_uint,
            last: 0xfe6b as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xff00 as ::core::ffi::c_uint,
            last: 0xff0f as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xff1a as ::core::ffi::c_uint,
            last: 0xff20 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xff3b as ::core::ffi::c_uint,
            last: 0xff40 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0xff5b as ::core::ffi::c_uint,
            last: 0xff65 as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x1d000 as ::core::ffi::c_int as ::core::ffi::c_uint,
            last: 0x1d24f as ::core::ffi::c_int as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x1d400 as ::core::ffi::c_int as ::core::ffi::c_uint,
            last: 0x1d7ff as ::core::ffi::c_int as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x1f000 as ::core::ffi::c_int as ::core::ffi::c_uint,
            last: 0x1f2ff as ::core::ffi::c_int as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x1f300 as ::core::ffi::c_int as ::core::ffi::c_uint,
            last: 0x1f9ff as ::core::ffi::c_int as ::core::ffi::c_uint,
            cls: 1 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x20000 as ::core::ffi::c_int as ::core::ffi::c_uint,
            last: 0x2a6df as ::core::ffi::c_int as ::core::ffi::c_uint,
            cls: 0x4e00 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x2a700 as ::core::ffi::c_int as ::core::ffi::c_uint,
            last: 0x2b73f as ::core::ffi::c_int as ::core::ffi::c_uint,
            cls: 0x4e00 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x2b740 as ::core::ffi::c_int as ::core::ffi::c_uint,
            last: 0x2b81f as ::core::ffi::c_int as ::core::ffi::c_uint,
            cls: 0x4e00 as ::core::ffi::c_uint,
        },
        clinterval {
            first: 0x2f800 as ::core::ffi::c_int as ::core::ffi::c_uint,
            last: 0x2fa1f as ::core::ffi::c_int as ::core::ffi::c_uint,
            cls: 0x4e00 as ::core::ffi::c_uint,
        },
    ]);
    let mut bot: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut top: ::core::ffi::c_int = ::core::mem::size_of::<[clinterval; 71]>()
        .wrapping_div(::core::mem::size_of::<clinterval>())
        .wrapping_div(
            (::core::mem::size_of::<[clinterval; 71]>()
                .wrapping_rem(::core::mem::size_of::<clinterval>())
                == 0) as ::core::ffi::c_int as usize,
        )
        .wrapping_sub(1 as usize) as ::core::ffi::c_int;
    if c < 0x100 as ::core::ffi::c_int {
        if c == ' ' as ::core::ffi::c_int
            || c == '\t' as ::core::ffi::c_int
            || c == NUL
            || c == 0xa0 as ::core::ffi::c_int
        {
            return 0 as ::core::ffi::c_int;
        }
        if vim_iswordc_tab(c, chartab) {
            return 2 as ::core::ffi::c_int;
        }
        return 1 as ::core::ffi::c_int;
    }
    let mut prop: *const utf8proc_property_t = utf8proc_get_property(c as utf8proc_int32_t);
    if prop_is_emojilike(prop) {
        return 3 as ::core::ffi::c_int;
    }
    while top >= bot {
        let mut mid: ::core::ffi::c_int = (bot + top) / 2 as ::core::ffi::c_int;
        if (*classes.ptr())[mid as usize].last < c as ::core::ffi::c_uint {
            bot = mid + 1 as ::core::ffi::c_int;
        } else if (*classes.ptr())[mid as usize].first > c as ::core::ffi::c_uint {
            top = mid - 1 as ::core::ffi::c_int;
        } else {
            return (*classes.ptr())[mid as usize].cls as ::core::ffi::c_int;
        }
    }
    return 2 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn utf_ambiguous_width(mut p: *const ::core::ffi::c_char) -> bool {
    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        return false_0 != 0;
    }
    let mut info: CharInfo = utf_ptr2CharInfo(p);
    if info.value >= 0x80 as int32_t {
        let mut prop: *const utf8proc_property_t =
            utf8proc_get_property(info.value as utf8proc_int32_t);
        if (*prop).ambiguous_width || prop_is_emojilike(prop) as ::core::ffi::c_int != 0 {
            return true_0 != 0;
        }
    }
    return memcmp(
        p.offset(info.len as isize) as *const ::core::ffi::c_void,
        b"\xEF\xB8\x8F\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        3 as size_t,
    ) == 0 as ::core::ffi::c_int;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf_fold(mut a: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if a < 0x80 as ::core::ffi::c_int {
        return if a >= 0x41 as ::core::ffi::c_int && a <= 0x5a as ::core::ffi::c_int {
            a + 32 as ::core::ffi::c_int
        } else {
            a
        };
    }
    if a == 0xdf as ::core::ffi::c_int || a == 0x130 as ::core::ffi::c_int {
        return a;
    }
    let mut result: [utf8proc_int32_t; 1] = [0; 1];
    let res = utf8proc_decompose_char(a as utf8proc_int32_t, &mut result, UTF8PROC_CASEFOLD, None);
    return if res == 1 {
        result[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
    } else {
        a
    };
}
pub unsafe extern "C" fn mb_toupper(mut a: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if a < 128 as ::core::ffi::c_int
        && cmp_flags.get() & kOptCmpFlagKeepascii as ::core::ffi::c_int as ::core::ffi::c_uint != 0
    {
        return if a < 'a' as ::core::ffi::c_int || a > 'z' as ::core::ffi::c_int {
            a
        } else {
            a - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        };
    }
    if cmp_flags.get() & kOptCmpFlagInternal as ::core::ffi::c_int as ::core::ffi::c_uint == 0 {
        return towupper(a as wint_t) as ::core::ffi::c_int;
    }
    if a < 128 as ::core::ffi::c_int {
        return toupper(a);
    }
    return utf8proc_toupper(a as utf8proc_int32_t) as ::core::ffi::c_int;
}
pub unsafe extern "C" fn mb_islower(mut a: ::core::ffi::c_int) -> bool {
    return mb_toupper(a) != a;
}
pub unsafe extern "C" fn mb_tolower(mut a: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if a < 128 as ::core::ffi::c_int
        && cmp_flags.get() & kOptCmpFlagKeepascii as ::core::ffi::c_int as ::core::ffi::c_uint != 0
    {
        return if a < 'A' as ::core::ffi::c_int || a > 'Z' as ::core::ffi::c_int {
            a
        } else {
            a + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        };
    }
    if cmp_flags.get() & kOptCmpFlagInternal as ::core::ffi::c_int as ::core::ffi::c_uint == 0 {
        return towlower(a as wint_t) as ::core::ffi::c_int;
    }
    if a < 128 as ::core::ffi::c_int {
        return tolower(a);
    }
    return utf8proc_tolower(a as utf8proc_int32_t) as ::core::ffi::c_int;
}
pub unsafe extern "C" fn mb_isupper(mut a: ::core::ffi::c_int) -> bool {
    return mb_tolower(a) != a;
}
pub unsafe extern "C" fn mb_isalpha(mut a: ::core::ffi::c_int) -> bool {
    return mb_islower(a) as ::core::ffi::c_int != 0 || mb_isupper(a) as ::core::ffi::c_int != 0;
}
pub unsafe extern "C" fn utf_strnicmp(
    mut s1: *const ::core::ffi::c_char,
    mut s2: *const ::core::ffi::c_char,
    mut n1: size_t,
    mut n2: size_t,
) -> ::core::ffi::c_int {
    let mut c1: ::core::ffi::c_int = 0;
    let mut c2: ::core::ffi::c_int = 0;
    let mut buffer: [::core::ffi::c_char; 6] = [0; 6];
    loop {
        c1 = utf_safe_read_char_adv(&raw mut s1, &raw mut n1);
        c2 = utf_safe_read_char_adv(&raw mut s2, &raw mut n2);
        if c1 <= 0 as ::core::ffi::c_int || c2 <= 0 as ::core::ffi::c_int {
            break;
        }
        if c1 == c2 {
            continue;
        }
        let mut cdiff: ::core::ffi::c_int = utf_fold(c1) - utf_fold(c2);
        if cdiff != 0 as ::core::ffi::c_int {
            return cdiff;
        }
    }
    if c1 == 0 as ::core::ffi::c_int || c2 == 0 as ::core::ffi::c_int {
        if c1 == 0 as ::core::ffi::c_int && c2 == 0 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        return if c1 == 0 as ::core::ffi::c_int {
            -1 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
    }
    if c1 != -1 as ::core::ffi::c_int && c2 == -1 as ::core::ffi::c_int {
        n1 = utf_char2bytes(utf_fold(c1), &raw mut buffer as *mut ::core::ffi::c_char) as size_t;
        s1 = &raw mut buffer as *mut ::core::ffi::c_char;
    } else if c2 != -1 as ::core::ffi::c_int && c1 == -1 as ::core::ffi::c_int {
        n2 = utf_char2bytes(utf_fold(c2), &raw mut buffer as *mut ::core::ffi::c_char) as size_t;
        s2 = &raw mut buffer as *mut ::core::ffi::c_char;
    }
    while n1 > 0 as size_t
        && n2 > 0 as size_t
        && *s1 as ::core::ffi::c_int != NUL
        && *s2 as ::core::ffi::c_int != NUL
    {
        let mut cdiff_0: ::core::ffi::c_int =
            *s1 as uint8_t as ::core::ffi::c_int - *s2 as uint8_t as ::core::ffi::c_int;
        if cdiff_0 != 0 as ::core::ffi::c_int {
            return cdiff_0;
        }
        s1 = s1.offset(1);
        s2 = s2.offset(1);
        n1 = n1.wrapping_sub(1);
        n2 = n2.wrapping_sub(1);
    }
    if n1 > 0 as size_t && *s1 as ::core::ffi::c_int == NUL {
        n1 = 0 as size_t;
    }
    if n2 > 0 as size_t && *s2 as ::core::ffi::c_int == NUL {
        n2 = 0 as size_t;
    }
    if n1 == 0 as size_t && n2 == 0 as size_t {
        return 0 as ::core::ffi::c_int;
    }
    return if n1 == 0 as size_t {
        -1 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    };
}
pub unsafe extern "C" fn mb_utflen(
    mut s: *const ::core::ffi::c_char,
    mut len: size_t,
    mut codepoints: *mut size_t,
    mut codeunits: *mut size_t,
) {
    let mut count: size_t = 0 as size_t;
    let mut extra: size_t = 0 as size_t;
    let mut clen: size_t = 0;
    let mut i: size_t = 0 as size_t;
    while i < len {
        clen = utf_ptr2len_len(
            s.offset(i as isize),
            len.wrapping_sub(i) as ::core::ffi::c_int,
        ) as size_t;
        let mut c: ::core::ffi::c_int = if clen > 1 as size_t {
            utf_ptr2char(s.offset(i as isize))
        } else {
            *s.offset(i as isize) as uint8_t as ::core::ffi::c_int
        };
        count = count.wrapping_add(1);
        if c > 0xffff as ::core::ffi::c_int {
            extra = extra.wrapping_add(1);
        }
        i = i.wrapping_add(clen);
    }
    *codepoints = (*codepoints).wrapping_add(count);
    *codeunits = (*codeunits).wrapping_add(count.wrapping_add(extra));
}
pub unsafe extern "C" fn mb_utf_index_to_bytes(
    mut s: *const ::core::ffi::c_char,
    mut len: size_t,
    mut index: size_t,
    mut use_utf16_units: bool,
) -> ssize_t {
    let mut count: size_t = 0 as size_t;
    let mut clen: size_t = 0;
    if index == 0 as size_t {
        return 0 as ssize_t;
    }
    let mut i: size_t = 0 as size_t;
    while i < len {
        clen = utf_ptr2len_len(
            s.offset(i as isize),
            len.wrapping_sub(i) as ::core::ffi::c_int,
        ) as size_t;
        let mut c: ::core::ffi::c_int = if clen > 1 as size_t {
            utf_ptr2char(s.offset(i as isize))
        } else {
            *s.offset(i as isize) as uint8_t as ::core::ffi::c_int
        };
        count = count.wrapping_add(1);
        if use_utf16_units as ::core::ffi::c_int != 0 && c > 0xffff as ::core::ffi::c_int {
            count = count.wrapping_add(1);
        }
        if count >= index {
            return i.wrapping_add(clen) as ssize_t;
        }
        i = i.wrapping_add(clen);
    }
    return -1 as ssize_t;
}
pub unsafe extern "C" fn mb_strnicmp(
    mut s1: *const ::core::ffi::c_char,
    mut s2: *const ::core::ffi::c_char,
    nn: size_t,
) -> ::core::ffi::c_int {
    return utf_strnicmp(s1, s2, nn, nn);
}
pub unsafe extern "C" fn mb_stricmp(
    mut s1: *const ::core::ffi::c_char,
    mut s2: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return mb_strnicmp(s1, s2, MAXCOL as ::core::ffi::c_int as size_t);
}
pub unsafe extern "C" fn show_utf8() {
    let mut line: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
    let mut len: ::core::ffi::c_int = utfc_ptr2len(line);
    if len == 0 as ::core::ffi::c_int {
        msg(
            b"NUL\0".as_ptr() as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
        return;
    }
    let mut rlen: size_t = 0 as size_t;
    let mut clen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < len {
        if clen == 0 as ::core::ffi::c_int {
            if i > 0 as ::core::ffi::c_int {
                strcpy(
                    (IObuff.ptr() as *mut ::core::ffi::c_char).offset(rlen as isize),
                    b"+ \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
                rlen = rlen.wrapping_add(2 as size_t);
            }
            clen = utf_ptr2len(line.offset(i as isize));
        }
        assert!(
            (1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as size_t > rlen,
            "IOSIZE > rlen"
        );
        snprintf(
            (IObuff.ptr() as *mut ::core::ffi::c_char).offset(rlen as isize),
            (IOSIZE as size_t).wrapping_sub(rlen),
            b"%02x \0".as_ptr() as *const ::core::ffi::c_char,
            if *line.offset(i as isize) as ::core::ffi::c_int == NL {
                NUL
            } else {
                *line.offset(i as isize) as uint8_t as ::core::ffi::c_int
            },
        );
        clen -= 1;
        rlen = rlen.wrapping_add(strlen(
            (IObuff.ptr() as *mut ::core::ffi::c_char).offset(rlen as isize),
        ));
        if rlen > (IOSIZE - 20 as ::core::ffi::c_int) as size_t {
            break;
        }
        i += 1;
    }
    msg(
        IObuff.ptr() as *mut ::core::ffi::c_char,
        0 as ::core::ffi::c_int,
    );
}
fn always_break(mut bc: ::core::ffi::c_int) -> bool {
    return bc == UTF8PROC_BOUNDCLASS_CONTROL as ::core::ffi::c_int;
}
fn always_break_two(mut bc1: ::core::ffi::c_int, mut bc2: ::core::ffi::c_int) -> bool {
    return bc1 != UTF8PROC_BOUNDCLASS_PREPEND as ::core::ffi::c_int
        && bc2 == UTF8PROC_BOUNDCLASS_OTHER as ::core::ffi::c_int
        || bc1 >= UTF8PROC_BOUNDCLASS_CR as ::core::ffi::c_int
            && bc1 <= UTF8PROC_BOUNDCLASS_CONTROL as ::core::ffi::c_int
        || bc2 == UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC as ::core::ffi::c_int
            && (bc1 == UTF8PROC_BOUNDCLASS_OTHER as ::core::ffi::c_int
                || bc1 == UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC as ::core::ffi::c_int);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf_head_off(
    mut base_in: *const ::core::ffi::c_char,
    mut p_in: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if (*p_in as uint8_t as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    let mut base: *const uint8_t = base_in as *mut uint8_t;
    let mut p: *const uint8_t = p_in as *mut uint8_t;
    let mut start: *const uint8_t = p;
    while start > base
        && *start as ::core::ffi::c_int & 0xc0 as ::core::ffi::c_int == 0x80 as ::core::ffi::c_int
        && p.offset_from(start) < 6 as isize
    {
        start = start.offset(-1);
    }
    let last_len: uint8_t = (*utf8len_tab.ptr())[*start as usize];
    let mut cur_code: int32_t = utf_ptr2CharInfo_impl(start, last_len as uintptr_t);
    if cur_code < 0 as int32_t || p.offset_from(start) >= last_len as isize {
        return 0 as ::core::ffi::c_int;
    }
    let safe_end: *const uint8_t = start.offset(last_len as ::core::ffi::c_int as isize);
    let mut cur_bc: ::core::ffi::c_int =
        (*utf8proc_get_property(cur_code as utf8proc_int32_t)).boundclass as ::core::ffi::c_int;
    if always_break(cur_bc) as ::core::ffi::c_int != 0 || start == base {
        return p.offset_from(start) as ::core::ffi::c_int;
    }
    let mut cur_pos: *const uint8_t = start;
    let p_start: *const uint8_t = start;
    while *start.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
        start = start.offset(-1);
        if (*start as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
            break;
        }
        while start > base
            && *start as ::core::ffi::c_int & 0xc0 as ::core::ffi::c_int
                == 0x80 as ::core::ffi::c_int
            && cur_pos.offset_from(start) < 6 as isize
        {
            start = start.offset(-1);
        }
        let mut prev_len: ::core::ffi::c_int =
            (*utf8len_tab.ptr())[*start as usize] as ::core::ffi::c_int;
        let mut prev_code: int32_t = utf_ptr2CharInfo_impl(start, prev_len as uintptr_t);
        if prev_code < 0 as int32_t || (prev_len as isize) < cur_pos.offset_from(start) {
            start = cur_pos;
            break;
        } else {
            let mut prev_bc: ::core::ffi::c_int =
                (*utf8proc_get_property(prev_code as utf8proc_int32_t)).boundclass
                    as ::core::ffi::c_int;
            if always_break_two(prev_bc, cur_bc) as ::core::ffi::c_int != 0
                && !crate::src::nvim::arabic::arabic_combine(
                    prev_code as ::core::ffi::c_int,
                    cur_code as ::core::ffi::c_int,
                )
            {
                start = cur_pos;
                break;
            } else {
                if start == base {
                    break;
                }
                cur_pos = start;
                cur_bc = prev_bc;
                cur_code = prev_code;
            }
        }
    }
    if start == p_start && last_len as isize > p.offset_from(start) {
        return p.offset_from(start) as ::core::ffi::c_int;
    }
    let mut q: *const uint8_t = start;
    while q < p {
        let mut len: ::core::ffi::c_int = utfc_ptr2len_len(
            q as *const ::core::ffi::c_char,
            safe_end.offset_from(q) as ::core::ffi::c_int,
        );
        if q.offset(len as isize) > p {
            return p.offset_from(q) as ::core::ffi::c_int;
        }
        q = q.offset(len as isize);
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn utfc_next_impl(mut cur: StrCharInfo) -> StrCharInfo {
    let mut prev_code: int32_t = cur.chr.value;
    let mut next: *mut uint8_t = cur.ptr.offset(cur.chr.len as isize) as *mut uint8_t;
    let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
    assert!(
        *next as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int,
        "*next >= 0x80"
    );
    loop {
        let next_len: uint8_t = (*utf8len_tab.ptr())[*next as usize];
        let next_code: int32_t = utf_ptr2CharInfo_impl(next, next_len as uintptr_t);
        if !utf_iscomposing(
            prev_code as ::core::ffi::c_int,
            next_code as ::core::ffi::c_int,
            &raw mut state,
        ) {
            return StrCharInfo {
                ptr: next as *mut ::core::ffi::c_char,
                chr: CharInfo {
                    value: next_code,
                    len: if next_code < 0 as int32_t {
                        1 as ::core::ffi::c_int
                    } else {
                        next_len as ::core::ffi::c_int
                    },
                },
            };
        }
        prev_code = next_code;
        next = next.offset(next_len as ::core::ffi::c_int as isize);
        if ((*next as ::core::ffi::c_uint) < 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int
            as ::core::ffi::c_long
            != 0
        {
            return StrCharInfo {
                ptr: next as *mut ::core::ffi::c_char,
                chr: CharInfo {
                    value: *next as int32_t,
                    len: 1 as ::core::ffi::c_int,
                },
            };
        }
    }
}
pub unsafe extern "C" fn utf_eat_space(mut cc: ::core::ffi::c_int) -> bool {
    return cc >= 0x2000 as ::core::ffi::c_int && cc <= 0x206f as ::core::ffi::c_int
        || cc >= 0x2e00 as ::core::ffi::c_int && cc <= 0x2e7f as ::core::ffi::c_int
        || cc >= 0x3000 as ::core::ffi::c_int && cc <= 0x303f as ::core::ffi::c_int
        || cc >= 0xff01 as ::core::ffi::c_int && cc <= 0xff0f as ::core::ffi::c_int
        || cc >= 0xff1a as ::core::ffi::c_int && cc <= 0xff20 as ::core::ffi::c_int
        || cc >= 0xff3b as ::core::ffi::c_int && cc <= 0xff40 as ::core::ffi::c_int
        || cc >= 0xff5b as ::core::ffi::c_int && cc <= 0xff65 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn utf_allow_break_before(mut cc: ::core::ffi::c_int) -> bool {
    static BOL_prohibition_punct: GlobalCell<[::core::ffi::c_int; 43]> = GlobalCell::new([
        '!' as ::core::ffi::c_int,
        '%' as ::core::ffi::c_int,
        ')' as ::core::ffi::c_int,
        ',' as ::core::ffi::c_int,
        ':' as ::core::ffi::c_int,
        ';' as ::core::ffi::c_int,
        '>' as ::core::ffi::c_int,
        '?' as ::core::ffi::c_int,
        ']' as ::core::ffi::c_int,
        '}' as ::core::ffi::c_int,
        0x2019 as ::core::ffi::c_int,
        0x201d as ::core::ffi::c_int,
        0x2020 as ::core::ffi::c_int,
        0x2021 as ::core::ffi::c_int,
        0x2026 as ::core::ffi::c_int,
        0x2030 as ::core::ffi::c_int,
        0x2031 as ::core::ffi::c_int,
        0x203c as ::core::ffi::c_int,
        0x2047 as ::core::ffi::c_int,
        0x2048 as ::core::ffi::c_int,
        0x2049 as ::core::ffi::c_int,
        0x2103 as ::core::ffi::c_int,
        0x2109 as ::core::ffi::c_int,
        0x3001 as ::core::ffi::c_int,
        0x3002 as ::core::ffi::c_int,
        0x3009 as ::core::ffi::c_int,
        0x300b as ::core::ffi::c_int,
        0x300d as ::core::ffi::c_int,
        0x300f as ::core::ffi::c_int,
        0x3011 as ::core::ffi::c_int,
        0x3015 as ::core::ffi::c_int,
        0x3017 as ::core::ffi::c_int,
        0x3019 as ::core::ffi::c_int,
        0x301b as ::core::ffi::c_int,
        0xff01 as ::core::ffi::c_int,
        0xff09 as ::core::ffi::c_int,
        0xff0c as ::core::ffi::c_int,
        0xff0e as ::core::ffi::c_int,
        0xff1a as ::core::ffi::c_int,
        0xff1b as ::core::ffi::c_int,
        0xff1f as ::core::ffi::c_int,
        0xff3d as ::core::ffi::c_int,
        0xff5d as ::core::ffi::c_int,
    ]);
    let mut first: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut last: ::core::ffi::c_int = ::core::mem::size_of::<[::core::ffi::c_int; 43]>()
        .wrapping_div(::core::mem::size_of::<::core::ffi::c_int>())
        .wrapping_div(
            (::core::mem::size_of::<[::core::ffi::c_int; 43]>()
                .wrapping_rem(::core::mem::size_of::<::core::ffi::c_int>())
                == 0) as ::core::ffi::c_int as usize,
        )
        .wrapping_sub(1 as usize) as ::core::ffi::c_int;
    while first < last {
        let mid: ::core::ffi::c_int = (first + last) / 2 as ::core::ffi::c_int;
        if cc == (*BOL_prohibition_punct.ptr())[mid as usize] {
            return false_0 != 0;
        } else if cc > (*BOL_prohibition_punct.ptr())[mid as usize] {
            first = mid + 1 as ::core::ffi::c_int;
        } else {
            last = mid - 1 as ::core::ffi::c_int;
        }
    }
    return cc != (*BOL_prohibition_punct.ptr())[first as usize];
}
pub unsafe extern "C" fn utf_allow_break_after(mut cc: ::core::ffi::c_int) -> bool {
    static EOL_prohibition_punct: GlobalCell<[::core::ffi::c_int; 19]> = GlobalCell::new([
        '(' as ::core::ffi::c_int,
        '<' as ::core::ffi::c_int,
        '[' as ::core::ffi::c_int,
        '`' as ::core::ffi::c_int,
        '{' as ::core::ffi::c_int,
        0x2018 as ::core::ffi::c_int,
        0x201c as ::core::ffi::c_int,
        0x3008 as ::core::ffi::c_int,
        0x300a as ::core::ffi::c_int,
        0x300c as ::core::ffi::c_int,
        0x300e as ::core::ffi::c_int,
        0x3010 as ::core::ffi::c_int,
        0x3014 as ::core::ffi::c_int,
        0x3016 as ::core::ffi::c_int,
        0x3018 as ::core::ffi::c_int,
        0x301a as ::core::ffi::c_int,
        0xff08 as ::core::ffi::c_int,
        0xff3b as ::core::ffi::c_int,
        0xff5b as ::core::ffi::c_int,
    ]);
    let mut first: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut last: ::core::ffi::c_int = ::core::mem::size_of::<[::core::ffi::c_int; 19]>()
        .wrapping_div(::core::mem::size_of::<::core::ffi::c_int>())
        .wrapping_div(
            (::core::mem::size_of::<[::core::ffi::c_int; 19]>()
                .wrapping_rem(::core::mem::size_of::<::core::ffi::c_int>())
                == 0) as ::core::ffi::c_int as usize,
        )
        .wrapping_sub(1 as usize) as ::core::ffi::c_int;
    while first < last {
        let mid: ::core::ffi::c_int = (first + last) / 2 as ::core::ffi::c_int;
        if cc == (*EOL_prohibition_punct.ptr())[mid as usize] {
            return false_0 != 0;
        } else if cc > (*EOL_prohibition_punct.ptr())[mid as usize] {
            first = mid + 1 as ::core::ffi::c_int;
        } else {
            last = mid - 1 as ::core::ffi::c_int;
        }
    }
    return cc != (*EOL_prohibition_punct.ptr())[first as usize];
}
pub unsafe extern "C" fn utf_allow_break(
    mut cc: ::core::ffi::c_int,
    mut ncc: ::core::ffi::c_int,
) -> bool {
    if cc == ncc && (cc == 0x2014 as ::core::ffi::c_int || cc == 0x2026 as ::core::ffi::c_int) {
        return false_0 != 0;
    }
    return utf_allow_break_after(cc) as ::core::ffi::c_int != 0
        && utf_allow_break_before(ncc) as ::core::ffi::c_int != 0;
}
pub unsafe extern "C" fn mb_copy_char(
    fp: *mut *const ::core::ffi::c_char,
    tp: *mut *mut ::core::ffi::c_char,
) {
    let l: size_t = utfc_ptr2len(*fp) as size_t;
    memmove(
        *tp as *mut ::core::ffi::c_void,
        *fp as *const ::core::ffi::c_void,
        l,
    );
    *tp = (*tp).offset(l as isize);
    *fp = (*fp).offset(l as isize);
}
pub unsafe extern "C" fn mb_off_next(
    mut base: *const ::core::ffi::c_char,
    mut p: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut head_off: ::core::ffi::c_int = utf_head_off(base, p);
    if head_off == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    return utfc_ptr2len(p.offset(-(head_off as isize))) - head_off;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf_cp_bounds_len(
    mut base: *const ::core::ffi::c_char,
    mut p_in: *const ::core::ffi::c_char,
    mut p_len: ::core::ffi::c_int,
) -> CharBoundsOff {
    assert!(
        base <= p_in && p_len > 0 as ::core::ffi::c_int,
        "base <= p_in && p_len > 0"
    );
    let b: *const uint8_t = base as *mut uint8_t;
    let p: *const uint8_t = p_in as *mut uint8_t;
    if (*p as ::core::ffi::c_uint) < 0x80 as ::core::ffi::c_uint {
        return CharBoundsOff {
            begin_off: 0 as int8_t,
            end_off: 1 as int8_t,
        };
    }
    let max_first_off: ::core::ffi::c_int = -if (p.offset_from(b) as ::core::ffi::c_int)
        < MB_MAXCHAR as ::core::ffi::c_int - 1 as ::core::ffi::c_int
    {
        p.offset_from(b) as ::core::ffi::c_int
    } else {
        MB_MAXCHAR as ::core::ffi::c_int - 1 as ::core::ffi::c_int
    };
    let mut first_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while utf_is_trail_byte(*p.offset(first_off as isize)) {
        if first_off == max_first_off {
            return CharBoundsOff {
                begin_off: 0 as int8_t,
                end_off: 1 as int8_t,
            };
        }
        first_off -= 1;
    }
    let max_end_off: ::core::ffi::c_int =
        (*utf8len_tab.ptr())[*p.offset(first_off as isize) as usize] as ::core::ffi::c_int
            + first_off;
    if max_end_off <= 0 as ::core::ffi::c_int || max_end_off > p_len {
        return CharBoundsOff {
            begin_off: 0 as int8_t,
            end_off: 1 as int8_t,
        };
    }
    let mut end_off: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while end_off < max_end_off {
        if !utf_is_trail_byte(*p.offset(end_off as isize)) {
            return CharBoundsOff {
                begin_off: 0 as int8_t,
                end_off: 1 as int8_t,
            };
        }
        end_off += 1;
    }
    return CharBoundsOff {
        begin_off: -first_off as int8_t,
        end_off: max_end_off as int8_t,
    };
}
pub unsafe extern "C" fn utf_cp_bounds(
    mut base: *const ::core::ffi::c_char,
    mut p_in: *const ::core::ffi::c_char,
) -> CharBoundsOff {
    return utf_cp_bounds_len(base, p_in, INT_MAX);
}
pub unsafe extern "C" fn utf_find_illegal() {
    let mut pos: pos_T = (*curwin.get()).w_cursor;
    let mut vimconv: vimconv_T = vimconv_T {
        vc_type: 0,
        vc_factor: 0,
        vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        vc_fail: false,
    };
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    vimconv.vc_type = CONV_NONE as ::core::ffi::c_int;
    if enc_canon_props((*curbuf.get()).b_p_fenc) & ENC_8BIT as ::core::ffi::c_int != 0 {
        convert_setup(&raw mut vimconv, p_enc.get(), (*curbuf.get()).b_p_fenc);
    }
    (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    '_theend: {
        loop {
            let mut p: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
            if vimconv.vc_type != CONV_NONE as ::core::ffi::c_int {
                xfree(tofree as *mut ::core::ffi::c_void);
                tofree = string_convert(&raw mut vimconv, p, ::core::ptr::null_mut::<size_t>());
                if tofree.is_null() {
                    break;
                }
                p = tofree;
            }
            while *p as ::core::ffi::c_int != NUL {
                let mut len: ::core::ffi::c_int = utf_ptr2len(p);
                if *p as uint8_t as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int
                    && (len == 1 as ::core::ffi::c_int || utf_char2len(utf_ptr2char(p)) != len)
                {
                    if vimconv.vc_type == CONV_NONE as ::core::ffi::c_int {
                        (*curwin.get()).w_cursor.col +=
                            p.offset_from(get_cursor_pos_ptr()) as colnr_T;
                    } else {
                        let mut l: ::core::ffi::c_int = 0;
                        len = p.offset_from(tofree) as ::core::ffi::c_int;
                        p = get_cursor_pos_ptr();
                        while *p as ::core::ffi::c_int != NUL && {
                            let c2rust_fresh1 = len;
                            len = len - 1;
                            c2rust_fresh1 > 0 as ::core::ffi::c_int
                        } {
                            l = utf_ptr2len(p);
                            (*curwin.get()).w_cursor.col += l;
                            p = p.offset(l as isize);
                        }
                    }
                    break '_theend;
                } else {
                    p = p.offset(len as isize);
                }
            }
            if (*curwin.get()).w_cursor.lnum == (*curbuf.get()).b_ml.ml_line_count {
                break;
            }
            (*curwin.get()).w_cursor.lnum += 1;
            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        }
        (*curwin.get()).w_cursor = pos;
        beep_flush();
    }
    xfree(tofree as *mut ::core::ffi::c_void);
    convert_setup(
        &raw mut vimconv,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    );
}
pub unsafe extern "C" fn utf_valid_string(
    mut s: *const ::core::ffi::c_char,
    mut end: *const ::core::ffi::c_char,
) -> bool {
    let mut p: *const uint8_t = s as *mut uint8_t;
    while if end.is_null() {
        (*p as ::core::ffi::c_int != NUL) as ::core::ffi::c_int
    } else {
        (p < end as *mut uint8_t as *const uint8_t) as ::core::ffi::c_int
    } != 0
    {
        let mut l: ::core::ffi::c_int =
            (*utf8len_tab_zero.ptr())[*p as usize] as ::core::ffi::c_int;
        if l == 0 as ::core::ffi::c_int {
            return false_0 != 0;
        }
        if !end.is_null() && p.offset(l as isize) > end as *mut uint8_t as *const uint8_t {
            return false_0 != 0;
        }
        p = p.offset(1);
        loop {
            l -= 1;
            if l <= 0 as ::core::ffi::c_int {
                break;
            }
            let c2rust_fresh12 = p;
            p = p.offset(1);
            if *c2rust_fresh12 as ::core::ffi::c_int & 0xc0 as ::core::ffi::c_int
                != 0x80 as ::core::ffi::c_int
            {
                return false_0 != 0;
            }
        }
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn mb_adjust_cursor() {
    mark_mb_adjustpos(curbuf.get(), &raw mut (*curwin.get()).w_cursor);
}
pub unsafe extern "C" fn mb_check_adjust_col(mut win_: *mut ::core::ffi::c_void) {
    let mut win: *mut win_T = win_ as *mut win_T;
    let mut oldcol: colnr_T = (*win).w_cursor.col;
    if oldcol != 0 as ::core::ffi::c_int {
        let mut p: *mut ::core::ffi::c_char = ml_get_buf((*win).w_buffer, (*win).w_cursor.lnum);
        let mut len: colnr_T = strlen(p) as colnr_T;
        if len == 0 as ::core::ffi::c_int || oldcol < 0 as ::core::ffi::c_int {
            (*win).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        } else {
            if oldcol > len {
                (*win).w_cursor.col =
                    (len as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as colnr_T;
            }
            (*win).w_cursor.col -= utf_head_off(p, p.offset((*win).w_cursor.col as isize));
        }
        if (*win).w_cursor.coladd == 1 as ::core::ffi::c_int
            && *p.offset((*win).w_cursor.col as isize) as ::core::ffi::c_int != TAB
            && vim_isprintc(utf_ptr2char(p.offset((*win).w_cursor.col as isize)))
                as ::core::ffi::c_int
                != 0
            && ptr2cells(p.offset((*win).w_cursor.col as isize)) > 1 as ::core::ffi::c_int
        {
            (*win).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
        }
    }
}
pub unsafe extern "C" fn mb_prevptr(
    mut line: *mut ::core::ffi::c_char,
    mut p: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if p > line {
        p = p.offset(
            -((utf_head_off(line, p.offset(-(1 as ::core::ffi::c_int as isize)))
                + 1 as ::core::ffi::c_int) as isize),
        );
    }
    return p;
}
pub unsafe extern "C" fn mb_charlen(mut str: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = str;
    let mut count: ::core::ffi::c_int = 0;
    if p.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    count = 0 as ::core::ffi::c_int;
    while *p as ::core::ffi::c_int != NUL {
        p = p.offset(utfc_ptr2len(p) as isize);
        count += 1;
    }
    return count;
}
pub unsafe extern "C" fn mb_charlen_len(
    mut str: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = str;
    let mut count: ::core::ffi::c_int = 0;
    count = 0 as ::core::ffi::c_int;
    while *p as ::core::ffi::c_int != NUL && p < str.offset(len as isize) {
        p = p.offset(utfc_ptr2len(p) as isize);
        count += 1;
    }
    return count;
}
pub unsafe extern "C" fn mb_unescape(
    pp: *mut *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    static buf: GlobalCell<[::core::ffi::c_char; 6]> = GlobalCell::new([0; 6]);
    let mut buf_idx: size_t = 0 as size_t;
    let mut str: *mut uint8_t = *pp as *mut uint8_t;
    let mut str_idx: size_t = 0 as size_t;
    while *str.offset(str_idx as isize) as ::core::ffi::c_int != NUL && buf_idx < 4 as size_t {
        if *str.offset(str_idx as isize) as ::core::ffi::c_int == K_SPECIAL
            && *str.offset(str_idx.wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                == KS_SPECIAL
            && *str.offset(str_idx.wrapping_add(2 as size_t) as isize) as ::core::ffi::c_int
                == KE_FILLER
        {
            let c2rust_fresh13 = buf_idx;
            buf_idx = buf_idx.wrapping_add(1);
            (*buf.ptr())[c2rust_fresh13 as usize] = K_SPECIAL as ::core::ffi::c_char;
            str_idx = str_idx.wrapping_add(2 as size_t);
        } else {
            if *str.offset(str_idx as isize) as ::core::ffi::c_int == K_SPECIAL {
                break;
            }
            let c2rust_fresh14 = buf_idx;
            buf_idx = buf_idx.wrapping_add(1);
            (*buf.ptr())[c2rust_fresh14 as usize] =
                *str.offset(str_idx as isize) as ::core::ffi::c_char;
        }
        (*buf.ptr())[buf_idx as usize] = NUL as ::core::ffi::c_char;
        if utf_ptr2len(buf.ptr() as *mut ::core::ffi::c_char) > 1 as ::core::ffi::c_int {
            *pp = (str as *const ::core::ffi::c_char)
                .offset(str_idx as isize)
                .offset(1 as ::core::ffi::c_int as isize);
            return buf.ptr() as *mut ::core::ffi::c_char;
        }
        if ((*buf.ptr())[0 as ::core::ffi::c_int as usize] as uint8_t as ::core::ffi::c_int)
            < 128 as ::core::ffi::c_int
        {
            break;
        }
        str_idx = str_idx.wrapping_add(1);
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn enc_skip(mut p: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    if strncmp(
        p,
        b"2byte-\0".as_ptr() as *const ::core::ffi::c_char,
        6 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        return p.offset(6 as ::core::ffi::c_int as isize);
    }
    if strncmp(
        p,
        b"8bit-\0".as_ptr() as *const ::core::ffi::c_char,
        5 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        return p.offset(5 as ::core::ffi::c_int as isize);
    }
    return p;
}
pub unsafe extern "C" fn enc_canonize(
    mut enc: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if strcmp(enc, b"default\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
        return xstrdup(fenc_default.get());
    }
    let mut r: *mut ::core::ffi::c_char =
        xmalloc(strlen(enc).wrapping_add(3 as size_t)) as *mut ::core::ffi::c_char;
    let mut p: *mut ::core::ffi::c_char = r;
    let mut s: *mut ::core::ffi::c_char = enc;
    while *s as ::core::ffi::c_int != NUL {
        if *s as ::core::ffi::c_int == '_' as ::core::ffi::c_int {
            let c2rust_fresh15 = p;
            p = p.offset(1);
            *c2rust_fresh15 = '-' as ::core::ffi::c_char;
        } else {
            let c2rust_fresh16 = p;
            p = p.offset(1);
            *c2rust_fresh16 = (if (*s as ::core::ffi::c_int) < 'A' as ::core::ffi::c_int
                || *s as ::core::ffi::c_int > 'Z' as ::core::ffi::c_int
            {
                *s as ::core::ffi::c_int
            } else {
                *s as ::core::ffi::c_int + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            }) as ::core::ffi::c_char;
        }
        s = s.offset(1);
    }
    *p = NUL as ::core::ffi::c_char;
    p = enc_skip(r);
    if strncmp(
        p,
        b"microsoft-cp\0".as_ptr() as *const ::core::ffi::c_char,
        12 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        memmove(
            p as *mut ::core::ffi::c_void,
            p.offset(10 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            strlen(p.offset(10 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
        );
    }
    if strncmp(
        p,
        b"iso8859\0".as_ptr() as *const ::core::ffi::c_char,
        7 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        memmove(
            p.offset(4 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            p.offset(3 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            strlen(p.offset(3 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
        );
        *p.offset(3 as ::core::ffi::c_int as isize) = '-' as ::core::ffi::c_char;
    }
    if strncmp(
        p,
        b"iso-8859\0".as_ptr() as *const ::core::ffi::c_char,
        8 as size_t,
    ) == 0 as ::core::ffi::c_int
        && *p.offset(8 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != '-' as ::core::ffi::c_int
    {
        memmove(
            p.offset(9 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            p.offset(8 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            strlen(p.offset(8 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
        );
        *p.offset(8 as ::core::ffi::c_int as isize) = '-' as ::core::ffi::c_char;
    }
    if strncmp(
        p,
        b"latin-\0".as_ptr() as *const ::core::ffi::c_char,
        6 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        memmove(
            p.offset(5 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            p.offset(6 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            strlen(p.offset(6 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
        );
    }
    let mut i: ::core::ffi::c_int = 0;
    if enc_canon_search(p) >= 0 as ::core::ffi::c_int {
        if p != r {
            memmove(
                r as *mut ::core::ffi::c_void,
                p as *const ::core::ffi::c_void,
                strlen(p).wrapping_add(1 as size_t),
            );
        }
    } else {
        i = enc_alias_search(p);
        if i >= 0 as ::core::ffi::c_int {
            xfree(r as *mut ::core::ffi::c_void);
            r = xstrdup((*enc_canon_table.ptr())[i as usize].name);
        }
    }
    return r;
}
unsafe extern "C" fn enc_alias_search(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while !(*enc_alias_table.ptr())[i as usize].name.is_null() {
        if strcmp(name, (*enc_alias_table.ptr())[i as usize].name) == 0 as ::core::ffi::c_int {
            return (*enc_alias_table.ptr())[i as usize].canon;
        }
        i += 1;
    }
    return -1 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn enc_locale() -> *mut ::core::ffi::c_char {
    let mut i: ::core::ffi::c_int = 0;
    let mut buf: [::core::ffi::c_char; 50] = [0; 50];
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    s = nl_langinfo(CODESET as ::core::ffi::c_int);
    if s.is_null() || *s as ::core::ffi::c_int == NUL {
        s = setlocale(LC_CTYPE, ::core::ptr::null::<::core::ffi::c_char>());
        if s.is_null() || *s as ::core::ffi::c_int == NUL {
            s = os_getenv_noalloc(b"LC_ALL\0".as_ptr() as *const ::core::ffi::c_char);
            if !s.is_null() {
                s = os_getenv_noalloc(b"LC_CTYPE\0".as_ptr() as *const ::core::ffi::c_char);
                if !s.is_null() {
                    s = os_getenv_noalloc(b"LANG\0".as_ptr() as *const ::core::ffi::c_char);
                }
            }
        }
    }
    if s.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut p: *const ::core::ffi::c_char = vim_strchr(s, '.' as ::core::ffi::c_int);
    's_140: {
        if !p.is_null() {
            if p > s.offset(2 as ::core::ffi::c_int as isize)
                && strncasecmp(
                    p.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char,
                    b"EUC\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    3 as ::core::ffi::c_int as size_t,
                ) == 0
                && *(*__ctype_b_loc())
                    .offset(*p.offset(4 as ::core::ffi::c_int as isize) as uint8_t
                        as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    & _ISalnum as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    == 0
                && *p.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '-' as ::core::ffi::c_int
                && *p.offset(-3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '_' as ::core::ffi::c_int
            {
                memmove(
                    &raw mut buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                    b"euc-\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                    4 as size_t,
                );
                buf[4 as ::core::ffi::c_int as usize] =
                    (if *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                        >= 'A' as ::core::ffi::c_uint
                        && *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                            <= 'Z' as ::core::ffi::c_uint
                        || *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                            >= 'a' as ::core::ffi::c_uint
                            && *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                                <= 'z' as ::core::ffi::c_uint
                        || ascii_isdigit(
                            *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        ) as ::core::ffi::c_int
                            != 0
                    {
                        if (*p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                            < 'A' as ::core::ffi::c_int
                            || *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                > 'Z' as ::core::ffi::c_int
                        {
                            *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        } else {
                            *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                        }
                    } else {
                        0 as ::core::ffi::c_int
                    }) as ::core::ffi::c_char;
                buf[5 as ::core::ffi::c_int as usize] =
                    (if *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                        >= 'A' as ::core::ffi::c_uint
                        && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                            <= 'Z' as ::core::ffi::c_uint
                        || *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                            >= 'a' as ::core::ffi::c_uint
                            && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                                <= 'z' as ::core::ffi::c_uint
                        || ascii_isdigit(
                            *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        ) as ::core::ffi::c_int
                            != 0
                    {
                        if (*p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                            < 'A' as ::core::ffi::c_int
                            || *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                > 'Z' as ::core::ffi::c_int
                        {
                            *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        } else {
                            *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                        }
                    } else {
                        0 as ::core::ffi::c_int
                    }) as ::core::ffi::c_char;
                buf[6 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
                break 's_140;
            } else {
                s = p.offset(1 as ::core::ffi::c_int as isize);
            }
        }
        i = 0 as ::core::ffi::c_int;
        while i < ::core::mem::size_of::<[::core::ffi::c_char; 50]>() as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int
            && *s.offset(i as isize) as ::core::ffi::c_int != NUL
        {
            if *s.offset(i as isize) as ::core::ffi::c_int == '_' as ::core::ffi::c_int
                || *s.offset(i as isize) as ::core::ffi::c_int == '-' as ::core::ffi::c_int
            {
                buf[i as usize] = '-' as ::core::ffi::c_char;
            } else {
                if !(*s.offset(i as isize) as uint8_t as ::core::ffi::c_uint
                    >= 'A' as ::core::ffi::c_uint
                    && *s.offset(i as isize) as uint8_t as ::core::ffi::c_uint
                        <= 'Z' as ::core::ffi::c_uint
                    || *s.offset(i as isize) as uint8_t as ::core::ffi::c_uint
                        >= 'a' as ::core::ffi::c_uint
                        && *s.offset(i as isize) as uint8_t as ::core::ffi::c_uint
                            <= 'z' as ::core::ffi::c_uint
                    || ascii_isdigit(*s.offset(i as isize) as uint8_t as ::core::ffi::c_int)
                        as ::core::ffi::c_int
                        != 0)
                {
                    break;
                }
                buf[i as usize] = (if (*s.offset(i as isize) as ::core::ffi::c_int)
                    < 'A' as ::core::ffi::c_int
                    || *s.offset(i as isize) as ::core::ffi::c_int > 'Z' as ::core::ffi::c_int
                {
                    *s.offset(i as isize) as ::core::ffi::c_int
                } else {
                    *s.offset(i as isize) as ::core::ffi::c_int
                        + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                }) as ::core::ffi::c_char;
            }
            i += 1;
        }
        buf[i as usize] = NUL as ::core::ffi::c_char;
    }
    return enc_canonize(&raw mut buf as *mut ::core::ffi::c_char);
}
pub unsafe extern "C" fn my_iconv_open(
    mut to: *mut ::core::ffi::c_char,
    mut from: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_void {
    let mut tobuf: [::core::ffi::c_char; 400] = [0; 400];
    static iconv_working: GlobalCell<WorkingStatus> = GlobalCell::new(kUnknown);
    if iconv_working.get() as ::core::ffi::c_uint
        == kBroken as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
            -1 as ::core::ffi::c_int as usize,
        );
    }
    let mut fd: iconv_t = iconv_open(enc_skip(to), enc_skip(from));
    if fd
        != ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
            -1 as ::core::ffi::c_int as usize,
        )
        && iconv_working.get() as ::core::ffi::c_uint
            == kUnknown as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut p: *mut ::core::ffi::c_char = &raw mut tobuf as *mut ::core::ffi::c_char;
        let mut tolen: size_t = ICONV_TESTLEN as size_t;
        iconv(
            fd,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<size_t>(),
            &raw mut p,
            &raw mut tolen,
        );
        if p.is_null() {
            iconv_working.set(kBroken);
            iconv_close(fd);
            fd = ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                -1 as ::core::ffi::c_int as usize,
            );
        } else {
            iconv_working.set(kWorking);
        }
    }
    return fd;
}
pub const ICONV_TESTLEN: ::core::ffi::c_int = 400 as ::core::ffi::c_int;
unsafe extern "C" fn iconv_string(
    vcp: *const vimconv_T,
    mut str: *const ::core::ffi::c_char,
    mut slen: size_t,
    mut unconvlenp: *mut size_t,
    mut resultlenp: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut to: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0 as size_t;
    let mut done: size_t = 0 as size_t;
    let mut result: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut from: *const ::core::ffi::c_char = str;
    let mut fromlen: size_t = slen;
    loop {
        if len == 0 as size_t || *__errno_location() == ICONV_E2BIG {
            len = len
                .wrapping_add(fromlen.wrapping_mul(2 as size_t))
                .wrapping_add(40 as size_t);
            let mut p: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
            if done > 0 as size_t {
                memmove(
                    p as *mut ::core::ffi::c_void,
                    result as *const ::core::ffi::c_void,
                    done,
                );
            }
            xfree(result as *mut ::core::ffi::c_void);
            result = p;
        }
        to = result.offset(done as isize);
        let mut tolen: size_t = len.wrapping_sub(done).wrapping_sub(2 as size_t);
        if iconv(
            (*vcp).vc_fd,
            &raw mut from as *mut ::core::ffi::c_void as *mut *mut ::core::ffi::c_char,
            &raw mut fromlen,
            &raw mut to,
            &raw mut tolen,
        ) != SIZE_MAX as size_t
        {
            *to = NUL as ::core::ffi::c_char;
            break;
        } else if !(*vcp).vc_fail
            && !unconvlenp.is_null()
            && (*__errno_location() == ICONV_EINVAL || *__errno_location() == EINVAL)
        {
            *to = NUL as ::core::ffi::c_char;
            *unconvlenp = fromlen;
            break;
        } else {
            if !(*vcp).vc_fail
                && (*__errno_location() == ICONV_EILSEQ
                    || *__errno_location() == EILSEQ
                    || *__errno_location() == ICONV_EINVAL
                    || *__errno_location() == EINVAL)
            {
                let c2rust_fresh10 = to;
                to = to.offset(1);
                *c2rust_fresh10 = '?' as ::core::ffi::c_char;
                if utf_ptr2cells(from) > 1 as ::core::ffi::c_int {
                    let c2rust_fresh11 = to;
                    to = to.offset(1);
                    *c2rust_fresh11 = '?' as ::core::ffi::c_char;
                }
                let mut l: ::core::ffi::c_int =
                    utfc_ptr2len_len(from, fromlen as ::core::ffi::c_int);
                from = from.offset(l as isize);
                fromlen = fromlen.wrapping_sub(l as size_t);
            } else if *__errno_location() != ICONV_E2BIG {
                let mut ptr_: *mut *mut ::core::ffi::c_void =
                    &raw mut result as *mut *mut ::core::ffi::c_void;
                xfree(*ptr_);
                *ptr_ = NULL;
                let _ = *ptr_;
                break;
            }
            done = to.offset_from(result) as size_t;
        }
    }
    if !resultlenp.is_null() && !result.is_null() {
        *resultlenp = to.offset_from(result) as size_t;
    }
    return result;
}
pub unsafe extern "C" fn f_iconv(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut vimconv: vimconv_T = vimconv_T {
        vc_type: 0,
        vc_factor: 0,
        vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        vc_fail: false,
    };
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let str: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut buf1: [::core::ffi::c_char; 65] = [0; 65];
    let from: *mut ::core::ffi::c_char = enc_canonize(enc_skip(tv_get_string_buf(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut buf1 as *mut ::core::ffi::c_char,
    ) as *mut ::core::ffi::c_char));
    let mut buf2: [::core::ffi::c_char; 65] = [0; 65];
    let to: *mut ::core::ffi::c_char = enc_canonize(enc_skip(tv_get_string_buf(
        argvars.offset(2 as ::core::ffi::c_int as isize),
        &raw mut buf2 as *mut ::core::ffi::c_char,
    ) as *mut ::core::ffi::c_char));
    vimconv.vc_type = CONV_NONE as ::core::ffi::c_int;
    convert_setup(&raw mut vimconv, from, to);
    if vimconv.vc_type == CONV_NONE as ::core::ffi::c_int {
        (*rettv).vval.v_string = xstrdup(str);
    } else {
        (*rettv).vval.v_string = string_convert(
            &raw mut vimconv,
            str as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<size_t>(),
        );
    }
    convert_setup(
        &raw mut vimconv,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    );
    xfree(from as *mut ::core::ffi::c_void);
    xfree(to as *mut ::core::ffi::c_void);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn convert_setup(
    mut vcp: *mut vimconv_T,
    mut from: *mut ::core::ffi::c_char,
    mut to: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return convert_setup_ext(vcp, from, true_0 != 0, to, true_0 != 0);
}
pub unsafe extern "C" fn convert_setup_ext(
    mut vcp: *mut vimconv_T,
    mut from: *mut ::core::ffi::c_char,
    mut from_unicode_is_utf8: bool,
    mut to: *mut ::core::ffi::c_char,
    mut to_unicode_is_utf8: bool,
) -> ::core::ffi::c_int {
    let mut from_is_utf8: ::core::ffi::c_int = 0;
    let mut to_is_utf8: ::core::ffi::c_int = 0;
    if (*vcp).vc_type == CONV_ICONV as ::core::ffi::c_int
        && (*vcp).vc_fd
            != ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                -1 as ::core::ffi::c_int as usize,
            )
    {
        iconv_close((*vcp).vc_fd);
    }
    *vcp = vimconv_T {
        vc_type: CONV_NONE as ::core::ffi::c_int,
        vc_factor: 1 as ::core::ffi::c_int,
        vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        vc_fail: false_0 != 0,
    };
    if from.is_null()
        || *from as ::core::ffi::c_int == NUL
        || to.is_null()
        || *to as ::core::ffi::c_int == NUL
        || strcmp(from, to) == 0 as ::core::ffi::c_int
    {
        return OK;
    }
    let mut from_prop: ::core::ffi::c_int = enc_canon_props(from);
    let mut to_prop: ::core::ffi::c_int = enc_canon_props(to);
    if from_unicode_is_utf8 {
        from_is_utf8 = from_prop & ENC_UNICODE as ::core::ffi::c_int;
    } else {
        from_is_utf8 = (from_prop == ENC_UNICODE as ::core::ffi::c_int) as ::core::ffi::c_int;
    }
    if to_unicode_is_utf8 {
        to_is_utf8 = to_prop & ENC_UNICODE as ::core::ffi::c_int;
    } else {
        to_is_utf8 = (to_prop == ENC_UNICODE as ::core::ffi::c_int) as ::core::ffi::c_int;
    }
    if from_prop & ENC_LATIN1 as ::core::ffi::c_int != 0 && to_is_utf8 != 0 {
        (*vcp).vc_type = CONV_TO_UTF8 as ::core::ffi::c_int;
        (*vcp).vc_factor = 2 as ::core::ffi::c_int;
    } else if from_prop & ENC_LATIN9 as ::core::ffi::c_int != 0 && to_is_utf8 != 0 {
        (*vcp).vc_type = CONV_9_TO_UTF8 as ::core::ffi::c_int;
        (*vcp).vc_factor = 3 as ::core::ffi::c_int;
    } else if from_is_utf8 != 0 && to_prop & ENC_LATIN1 as ::core::ffi::c_int != 0 {
        (*vcp).vc_type = CONV_TO_LATIN1 as ::core::ffi::c_int;
    } else if from_is_utf8 != 0 && to_prop & ENC_LATIN9 as ::core::ffi::c_int != 0 {
        (*vcp).vc_type = CONV_TO_LATIN9 as ::core::ffi::c_int;
    } else {
        (*vcp).vc_fd = my_iconv_open(
            (if to_is_utf8 != 0 {
                b"utf-8\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                to as *const ::core::ffi::c_char
            }) as *mut ::core::ffi::c_char,
            (if from_is_utf8 != 0 {
                b"utf-8\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                from as *const ::core::ffi::c_char
            }) as *mut ::core::ffi::c_char,
        );
        if (*vcp).vc_fd
            != ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                -1 as ::core::ffi::c_int as usize,
            )
        {
            (*vcp).vc_type = CONV_ICONV as ::core::ffi::c_int;
            (*vcp).vc_factor = 4 as ::core::ffi::c_int;
        }
    }
    if (*vcp).vc_type == CONV_NONE as ::core::ffi::c_int {
        return FAIL;
    }
    return OK;
}
pub unsafe extern "C" fn string_convert(
    vcp: *const vimconv_T,
    mut ptr: *mut ::core::ffi::c_char,
    mut lenp: *mut size_t,
) -> *mut ::core::ffi::c_char {
    return string_convert_ext(vcp, ptr, lenp, ::core::ptr::null_mut::<size_t>());
}
pub unsafe extern "C" fn string_convert_ext(
    vcp: *const vimconv_T,
    mut ptr: *mut ::core::ffi::c_char,
    mut lenp: *mut size_t,
    mut unconvlenp: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut retval: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut d: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut c: ::core::ffi::c_int = 0;
    let mut len: size_t = 0;
    if lenp.is_null() {
        len = strlen(ptr);
    } else {
        len = *lenp;
    }
    if len == 0 as size_t {
        return xstrdup(b"\0".as_ptr() as *const ::core::ffi::c_char);
    }
    match (*vcp).vc_type {
        1 => {
            retval =
                xmalloc(len.wrapping_mul(2 as size_t).wrapping_add(1 as size_t)) as *mut uint8_t;
            d = retval;
            let mut i: size_t = 0 as size_t;
            while i < len {
                c = *ptr.offset(i as isize) as uint8_t as ::core::ffi::c_int;
                if c < 0x80 as ::core::ffi::c_int {
                    let c2rust_fresh2 = d;
                    d = d.offset(1);
                    *c2rust_fresh2 = c as uint8_t;
                } else {
                    let c2rust_fresh3 = d;
                    d = d.offset(1);
                    *c2rust_fresh3 = (0xc0 as ::core::ffi::c_int
                        + (c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int) as uint8_t
                            as ::core::ffi::c_int) as uint8_t;
                    let c2rust_fresh4 = d;
                    d = d.offset(1);
                    *c2rust_fresh4 =
                        (0x80 as ::core::ffi::c_int + (c & 0x3f as ::core::ffi::c_int)) as uint8_t;
                }
                i = i.wrapping_add(1);
            }
            *d = NUL as uint8_t;
            if !lenp.is_null() {
                *lenp = d.offset_from(retval) as size_t;
            }
        }
        2 => {
            retval =
                xmalloc(len.wrapping_mul(3 as size_t).wrapping_add(1 as size_t)) as *mut uint8_t;
            d = retval;
            let mut i_0: size_t = 0 as size_t;
            while i_0 < len {
                c = *ptr.offset(i_0 as isize) as uint8_t as ::core::ffi::c_int;
                match c {
                    164 => {
                        c = 0x20ac as ::core::ffi::c_int;
                    }
                    166 => {
                        c = 0x160 as ::core::ffi::c_int;
                    }
                    168 => {
                        c = 0x161 as ::core::ffi::c_int;
                    }
                    180 => {
                        c = 0x17d as ::core::ffi::c_int;
                    }
                    184 => {
                        c = 0x17e as ::core::ffi::c_int;
                    }
                    188 => {
                        c = 0x152 as ::core::ffi::c_int;
                    }
                    189 => {
                        c = 0x153 as ::core::ffi::c_int;
                    }
                    190 => {
                        c = 0x178 as ::core::ffi::c_int;
                    }
                    _ => {}
                }
                d = d.offset(utf_char2bytes(c, d as *mut ::core::ffi::c_char) as isize);
                i_0 = i_0.wrapping_add(1);
            }
            *d = NUL as uint8_t;
            if !lenp.is_null() {
                *lenp = d.offset_from(retval) as size_t;
            }
        }
        3 | 4 => {
            retval = xmalloc(len.wrapping_add(1 as size_t)) as *mut uint8_t;
            d = retval;
            let mut i_1: size_t = 0 as size_t;
            while i_1 < len {
                let mut l: ::core::ffi::c_int = utf_ptr2len_len(
                    ptr.offset(i_1 as isize),
                    len.wrapping_sub(i_1) as ::core::ffi::c_int,
                );
                if l == 0 as ::core::ffi::c_int {
                    let c2rust_fresh5 = d;
                    d = d.offset(1);
                    *c2rust_fresh5 = NUL as uint8_t;
                } else if l == 1 as ::core::ffi::c_int {
                    let mut l_w: uint8_t =
                        (*utf8len_tab_zero.ptr())[*ptr.offset(i_1 as isize) as uint8_t as usize];
                    if l_w as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                        xfree(retval as *mut ::core::ffi::c_void);
                        return ::core::ptr::null_mut::<::core::ffi::c_char>();
                    }
                    if !unconvlenp.is_null() && l_w as size_t > len.wrapping_sub(i_1) {
                        *unconvlenp = len.wrapping_sub(i_1);
                        break;
                    } else {
                        let c2rust_fresh6 = d;
                        d = d.offset(1);
                        *c2rust_fresh6 = *ptr.offset(i_1 as isize) as uint8_t;
                    }
                } else {
                    c = utf_ptr2char(ptr.offset(i_1 as isize));
                    if (*vcp).vc_type == CONV_TO_LATIN9 as ::core::ffi::c_int {
                        match c {
                            8364 => {
                                c = 0xa4 as ::core::ffi::c_int;
                            }
                            352 => {
                                c = 0xa6 as ::core::ffi::c_int;
                            }
                            353 => {
                                c = 0xa8 as ::core::ffi::c_int;
                            }
                            381 => {
                                c = 0xb4 as ::core::ffi::c_int;
                            }
                            382 => {
                                c = 0xb8 as ::core::ffi::c_int;
                            }
                            338 => {
                                c = 0xbc as ::core::ffi::c_int;
                            }
                            339 => {
                                c = 0xbd as ::core::ffi::c_int;
                            }
                            376 => {
                                c = 0xbe as ::core::ffi::c_int;
                            }
                            164 | 166 | 168 | 180 | 184 | 188 | 189 | 190 => {
                                c = 0x100 as ::core::ffi::c_int;
                            }
                            _ => {}
                        }
                    }
                    if !utf_iscomposing_legacy(c) {
                        if c < 0x100 as ::core::ffi::c_int {
                            let c2rust_fresh7 = d;
                            d = d.offset(1);
                            *c2rust_fresh7 = c as uint8_t;
                        } else if (*vcp).vc_fail {
                            xfree(retval as *mut ::core::ffi::c_void);
                            return ::core::ptr::null_mut::<::core::ffi::c_char>();
                        } else {
                            let c2rust_fresh8 = d;
                            d = d.offset(1);
                            *c2rust_fresh8 = 0xbf as uint8_t;
                            if utf_char2cells(c) > 1 as ::core::ffi::c_int {
                                let c2rust_fresh9 = d;
                                d = d.offset(1);
                                *c2rust_fresh9 = '?' as uint8_t;
                            }
                        }
                    }
                    i_1 = i_1.wrapping_add((l as size_t).wrapping_sub(1 as size_t));
                }
                i_1 = i_1.wrapping_add(1);
            }
            *d = NUL as uint8_t;
            if !lenp.is_null() {
                *lenp = d.offset_from(retval) as size_t;
            }
        }
        5 => {
            retval = iconv_string(vcp, ptr, len, unconvlenp, lenp) as *mut uint8_t;
        }
        _ => {}
    }
    return retval as *mut ::core::ffi::c_char;
}
static cw_table: GlobalCell<*mut cw_interval_T> =
    GlobalCell::new(::core::ptr::null_mut::<cw_interval_T>());
static cw_table_size: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
unsafe extern "C" fn cw_value(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if (*cw_table.ptr()).is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if (c as int64_t) < (*(*cw_table.ptr()).offset(0 as ::core::ffi::c_int as isize)).first {
        return 0 as ::core::ffi::c_int;
    }
    let mut bot: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut top: ::core::ffi::c_int =
        cw_table_size.get() as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
    while top >= bot {
        let mut mid: ::core::ffi::c_int = (bot + top) / 2 as ::core::ffi::c_int;
        if (*(*cw_table.ptr()).offset(mid as isize)).last < c as int64_t {
            bot = mid + 1 as ::core::ffi::c_int;
        } else if (*(*cw_table.ptr()).offset(mid as isize)).first > c as int64_t {
            top = mid - 1 as ::core::ffi::c_int;
        } else {
            return (*(*cw_table.ptr()).offset(mid as isize)).width as ::core::ffi::c_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn tv_nr_compare(
    mut a1: *const ::core::ffi::c_void,
    mut a2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let li1: *const listitem_T = tv_list_first(*(a1 as *mut *const list_T));
    let li2: *const listitem_T = tv_list_first(*(a2 as *mut *const list_T));
    let n1: varnumber_T = (*li1).li_tv.vval.v_number;
    let n2: varnumber_T = (*li2).li_tv.vval.v_number;
    return if n1 == n2 {
        0 as ::core::ffi::c_int
    } else if n1 > n2 {
        1 as ::core::ffi::c_int
    } else {
        -1 as ::core::ffi::c_int
    };
}
pub unsafe extern "C" fn f_setcellwidths(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut ptrs: *mut *const list_T = ::core::ptr::null_mut::<*const list_T>();
    let mut item: ::core::ffi::c_int = 0;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_list
            .is_null()
    {
        emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
        return;
    }
    let l: *const list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    let mut table: *mut cw_interval_T = ::core::ptr::null_mut::<cw_interval_T>();
    let table_size: size_t = tv_list_len(l) as size_t;
    if table_size != 0 as size_t {
        ptrs = xmalloc(::core::mem::size_of::<*const list_T>().wrapping_mul(table_size))
            as *mut *const list_T;
        item = 0 as ::core::ffi::c_int;
        let l_: *const list_T = l;
        if !l_.is_null() {
            let mut li: *const listitem_T = (*l_).lv_first;
            while !li.is_null() {
                let li_tv: *const typval_T = &raw const (*li).li_tv;
                if (*li_tv).v_type as ::core::ffi::c_uint
                    != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                    || (*li_tv).vval.v_list.is_null()
                {
                    semsg(
                        gettext(
                            (e_list_item_nr_is_not_list.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        ),
                        item,
                    );
                    xfree(ptrs as *mut ::core::ffi::c_void);
                    return;
                }
                let li_l: *const list_T = (*li_tv).vval.v_list;
                *ptrs.offset(item as isize) = li_l;
                let mut lili: *const listitem_T = tv_list_first(li_l);
                let mut i: ::core::ffi::c_int = 0;
                let mut n1: varnumber_T = 0;
                i = 0 as ::core::ffi::c_int;
                while !lili.is_null() {
                    let lili_tv: *const typval_T = &raw const (*lili).li_tv;
                    if (*lili_tv).v_type as ::core::ffi::c_uint
                        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        break;
                    }
                    if i == 0 as ::core::ffi::c_int {
                        n1 = (*lili_tv).vval.v_number;
                        if n1 < 0x80 as varnumber_T {
                            emsg(gettext(
                                (e_only_values_of_0x80_and_higher_supported.ptr() as *const _)
                                    as *const ::core::ffi::c_char,
                            ));
                            xfree(ptrs as *mut ::core::ffi::c_void);
                            return;
                        }
                    } else if i == 1 as ::core::ffi::c_int && (*lili_tv).vval.v_number < n1 {
                        semsg(
                            gettext(
                                (e_list_item_nr_range_invalid.ptr() as *const _)
                                    as *const ::core::ffi::c_char,
                            ),
                            item,
                        );
                        xfree(ptrs as *mut ::core::ffi::c_void);
                        return;
                    } else if i == 2 as ::core::ffi::c_int
                        && ((*lili_tv).vval.v_number < 1 as varnumber_T
                            || (*lili_tv).vval.v_number > 2 as varnumber_T)
                    {
                        semsg(
                            gettext(
                                (e_list_item_nr_cell_width_invalid.ptr() as *const _)
                                    as *const ::core::ffi::c_char,
                            ),
                            item,
                        );
                        xfree(ptrs as *mut ::core::ffi::c_void);
                        return;
                    }
                    lili = (*lili).li_next;
                    i += 1;
                }
                if i != 3 as ::core::ffi::c_int {
                    semsg(
                        gettext(
                            (e_list_item_nr_does_not_contain_3_numbers.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        ),
                        item,
                    );
                    xfree(ptrs as *mut ::core::ffi::c_void);
                    return;
                }
                item += 1;
                li = (*li).li_next;
            }
        }
        qsort(
            ptrs as *mut ::core::ffi::c_void,
            table_size,
            ::core::mem::size_of::<*const list_T>(),
            Some(
                tv_nr_compare
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
        table = xmalloc(::core::mem::size_of::<cw_interval_T>().wrapping_mul(table_size))
            as *mut cw_interval_T;
        item = 0 as ::core::ffi::c_int;
        while (item as size_t) < table_size {
            let li_l_0: *const list_T = *ptrs.offset(item as isize);
            let mut lili_0: *const listitem_T = tv_list_first(li_l_0);
            let n1_0: varnumber_T = (*lili_0).li_tv.vval.v_number;
            if item > 0 as ::core::ffi::c_int
                && n1_0 <= (*table.offset((item - 1 as ::core::ffi::c_int) as isize)).last
            {
                semsg(
                    gettext(
                        (e_overlapping_ranges_for_nr.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    ),
                    n1_0 as size_t,
                );
                xfree(ptrs as *mut ::core::ffi::c_void);
                xfree(table as *mut ::core::ffi::c_void);
                return;
            }
            (*table.offset(item as isize)).first = n1_0 as int64_t;
            lili_0 = (*lili_0).li_next;
            (*table.offset(item as isize)).last = (*lili_0).li_tv.vval.v_number as int64_t;
            lili_0 = (*lili_0).li_next;
            (*table.offset(item as isize)).width =
                (*lili_0).li_tv.vval.v_number as ::core::ffi::c_char;
            item += 1;
        }
        xfree(ptrs as *mut ::core::ffi::c_void);
    }
    let cw_table_save: *mut cw_interval_T = cw_table.get();
    let cw_table_size_save: size_t = cw_table_size.get();
    cw_table.set(table);
    cw_table_size.set(table_size);
    let error: *const ::core::ffi::c_char = check_chars_options();
    if !error.is_null() {
        emsg(gettext(error));
        cw_table.set(cw_table_save);
        cw_table_size.set(cw_table_size_save);
        xfree(table as *mut ::core::ffi::c_void);
        return;
    }
    xfree(cw_table_save as *mut ::core::ffi::c_void);
    changed_window_setting_all();
    redraw_all_later(UPD_NOT_VALID);
}
pub unsafe extern "C" fn f_getcellwidths(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, cw_table_size.get() as ptrdiff_t);
    let mut i: size_t = 0 as size_t;
    while i < cw_table_size.get() {
        let mut entry: *mut list_T = tv_list_alloc(3 as ptrdiff_t);
        tv_list_append_number(entry, (*(*cw_table.ptr()).offset(i as isize)).first);
        tv_list_append_number(entry, (*(*cw_table.ptr()).offset(i as isize)).last);
        tv_list_append_number(
            entry,
            (*(*cw_table.ptr()).offset(i as isize)).width as varnumber_T,
        );
        tv_list_append_list((*rettv).vval.v_list, entry);
        i = i.wrapping_add(1);
    }
}
pub unsafe extern "C" fn f_charclass(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if tv_check_for_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_string
            .is_null()
    {
        return;
    }
    (*rettv).vval.v_number = mb_get_class(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_string,
    ) as varnumber_T;
}
pub unsafe extern "C" fn get_encoding_name(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if idx
        >= ::core::mem::size_of::<[C2Rust_Unnamed_21; 59]>()
            .wrapping_div(::core::mem::size_of::<C2Rust_Unnamed_21>())
            .wrapping_div(
                (::core::mem::size_of::<[C2Rust_Unnamed_21; 59]>()
                    .wrapping_rem(::core::mem::size_of::<C2Rust_Unnamed_21>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return (*enc_canon_table.ptr())[idx as usize].name as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn mb_strcmp_ic(
    mut ic: bool,
    mut s1: *const ::core::ffi::c_char,
    mut s2: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return if ic as ::core::ffi::c_int != 0 {
        mb_stricmp(s1, s2)
    } else {
        strcmp(s1, s2)
    };
}
pub const GRAPHEME_STATE_INIT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline(always)]
fn utf_is_trail_byte(byte: uint8_t) -> bool {
    return (byte as ::core::ffi::c_uint & 0xc0 as ::core::ffi::c_uint) as uint8_t
        as ::core::ffi::c_uint
        == 0x80 as ::core::ffi::c_uint;
}
/// The codepoint at `p` and the number of bytes it occupies. An invalid
/// sequence reports its first byte negated, with a length of one.
///
/// # Safety
/// `p` must point into a NUL-terminated string.
#[inline(always)]
pub unsafe fn utf_ptr2CharInfo(p_in: *const ::core::ffi::c_char) -> CharInfo {
    let p = p_in as *const uint8_t;
    let first = *p;
    if first < 0x80 {
        return CharInfo {
            value: first as int32_t,
            len: 1,
        };
    }
    let len = (*utf8len_tab.ptr())[first as usize] as ::core::ffi::c_int;
    let code_point = utf_ptr2CharInfo_impl(p, len as uintptr_t);
    CharInfo {
        value: code_point,
        len: if code_point < 0 { 1 } else { len },
    }
}
/// `cur` paired with its codepoint: the start of a character and the
/// character itself. Composing characters are not consulted.
///
/// # Safety
/// `ptr` must point into a NUL-terminated string.
#[inline(always)]
pub unsafe fn utf_ptr2StrCharInfo(ptr: *mut ::core::ffi::c_char) -> StrCharInfo {
    StrCharInfo {
        ptr,
        chr: utf_ptr2CharInfo(ptr),
    }
}
/// The character after `cur`, treating a following composing character as
/// part of the *current* one. The ASCII case is inlined; everything else
/// defers to `utfc_next_impl`.
///
/// # Safety
/// `cur.ptr` must point into a NUL-terminated string, at a character start.
#[inline(always)]
pub unsafe fn utfc_next(cur: StrCharInfo) -> StrCharInfo {
    let next = cur.ptr.offset(cur.chr.len as isize) as *mut uint8_t;
    if *next < 0x80 {
        return StrCharInfo {
            ptr: next as *mut ::core::ffi::c_char,
            chr: CharInfo {
                value: *next as int32_t,
                len: 1,
            },
        };
    }
    utfc_next_impl(cur)
}
pub const E2BIG: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const EINVAL: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const __LC_CTYPE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const ICONV_E2BIG: ::core::ffi::c_int = E2BIG;
pub const ICONV_EINVAL: ::core::ffi::c_int = EINVAL;
pub const ICONV_EILSEQ: ::core::ffi::c_int = EILSEQ;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
pub const EILSEQ: ::core::ffi::c_int = 84 as ::core::ffi::c_int;
unsafe extern "C" fn c2rust_run_static_initializers() {
    corrections.set([
        (1 as uint32_t) << 31 as ::core::ffi::c_int,
        (1 as uint32_t) << 31 as ::core::ffi::c_int,
        (0x80 as uint32_t)
            .wrapping_add((0xc0 as uint32_t) << 6 as ::core::ffi::c_int)
            .wrapping_neg(),
        (0x80 as uint32_t)
            .wrapping_add((0x80 as uint32_t) << 6 as ::core::ffi::c_int)
            .wrapping_add((0xe0 as uint32_t) << 12 as ::core::ffi::c_int)
            .wrapping_neg(),
        (0x80 as uint32_t)
            .wrapping_add((0x80 as uint32_t) << 6 as ::core::ffi::c_int)
            .wrapping_add((0x80 as uint32_t) << 12 as ::core::ffi::c_int)
            .wrapping_add((0xf0 as uint32_t) << 18 as ::core::ffi::c_int)
            .wrapping_neg(),
        (0x80 as uint32_t)
            .wrapping_add((0x80 as uint32_t) << 6 as ::core::ffi::c_int)
            .wrapping_add((0x80 as uint32_t) << 12 as ::core::ffi::c_int)
            .wrapping_add((0x80 as uint32_t) << 18 as ::core::ffi::c_int)
            .wrapping_add((0xf8 as uint32_t) << 24 as ::core::ffi::c_int)
            .wrapping_neg(),
        (0x80 as uint32_t)
            .wrapping_add((0x80 as uint32_t) << 6 as ::core::ffi::c_int)
            .wrapping_add((0x80 as uint32_t) << 12 as ::core::ffi::c_int)
            .wrapping_add((0x80 as uint32_t) << 18 as ::core::ffi::c_int)
            .wrapping_add((0x80 as uint32_t) << 24 as ::core::ffi::c_int)
            .wrapping_neg(),
    ]);
}
#[used]
#[cfg_attr(target_os = "linux", unsafe(link_section = ".init_array"))]
#[cfg_attr(target_os = "windows", unsafe(link_section = ".CRT$XIB"))]
#[cfg_attr(target_os = "macos", unsafe(link_section = "__DATA,__mod_init_func"))]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [c2rust_run_static_initializers];
