//! The canonical encoding table and its aliases.
//!
//! `enc_canon_table` is the list of names nvim knows, each with its properties and
//! DBCS codepage; the `IDX_*` constants are indices into it, so a row may not move
//! without moving them.  `enc_alias_table` maps every other spelling anyone writes
//! onto one of those indices.  Both are sorted and searched by name.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::super::*;
#[allow(unused_imports)]
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_21 {
    pub name: *const ::core::ffi::c_char,
    pub prop: ::core::ffi::c_int,
    pub codepage: ::core::ffi::c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_22 {
    pub name: *const ::core::ffi::c_char,
    pub canon: ::core::ffi::c_int,
}

pub const DBCS_JPN: ::core::ffi::c_int = 932 as ::core::ffi::c_int;

pub const DBCS_JPNU: ::core::ffi::c_int = 9932 as ::core::ffi::c_int;

pub const DBCS_KOR: ::core::ffi::c_int = 949 as ::core::ffi::c_int;

pub const DBCS_KORU: ::core::ffi::c_int = 9949 as ::core::ffi::c_int;

pub const DBCS_CHS: ::core::ffi::c_int = 936 as ::core::ffi::c_int;

pub const DBCS_CHSU: ::core::ffi::c_int = 9936 as ::core::ffi::c_int;

pub const DBCS_CHT: ::core::ffi::c_int = 950 as ::core::ffi::c_int;

pub const DBCS_CHTU: ::core::ffi::c_int = 9950 as ::core::ffi::c_int;

pub const DBCS_DEBUG: ::core::ffi::c_int = -1 as ::core::ffi::c_int;

pub(crate) static enc_canon_table: GlobalCell<[C2Rust_Unnamed_21; 59]> = GlobalCell::new([
    C2Rust_Unnamed_21 {
        name: b"latin1\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT + ENC_LATIN1,
        codepage: 1252 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-2\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-3\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-4\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-5\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-6\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-7\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-8\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-9\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-10\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-11\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-13\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-14\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"iso-8859-15\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT + ENC_LATIN9,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"koi8-r\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"koi8-u\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"utf-8\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_UNICODE,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"ucs-2\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_UNICODE + ENC_ENDIAN_B + ENC_2BYTE,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"ucs-2le\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_UNICODE + ENC_ENDIAN_L + ENC_2BYTE,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"utf-16\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_UNICODE + ENC_ENDIAN_B + ENC_2WORD,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"utf-16le\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_UNICODE + ENC_ENDIAN_L + ENC_2WORD,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"ucs-4\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_UNICODE + ENC_ENDIAN_B + ENC_4BYTE,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"ucs-4le\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_UNICODE + ENC_ENDIAN_L + ENC_4BYTE,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"debug\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS,
        codepage: DBCS_DEBUG,
    },
    C2Rust_Unnamed_21 {
        name: b"euc-jp\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS,
        codepage: DBCS_JPNU,
    },
    C2Rust_Unnamed_21 {
        name: b"sjis\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS,
        codepage: DBCS_JPN,
    },
    C2Rust_Unnamed_21 {
        name: b"euc-kr\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS,
        codepage: DBCS_KORU,
    },
    C2Rust_Unnamed_21 {
        name: b"euc-cn\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS,
        codepage: DBCS_CHSU,
    },
    C2Rust_Unnamed_21 {
        name: b"euc-tw\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS,
        codepage: DBCS_CHTU,
    },
    C2Rust_Unnamed_21 {
        name: b"big5\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS,
        codepage: DBCS_CHT,
    },
    C2Rust_Unnamed_21 {
        name: b"cp437\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 437 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp737\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 737 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp775\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 775 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp850\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 850 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp852\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 852 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp855\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 855 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp857\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 857 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp860\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 860 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp861\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 861 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp862\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 862 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp863\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 863 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp865\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 865 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp866\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 866 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp869\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 869 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp874\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 874 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp932\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS,
        codepage: DBCS_JPN,
    },
    C2Rust_Unnamed_21 {
        name: b"cp936\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS,
        codepage: DBCS_CHS,
    },
    C2Rust_Unnamed_21 {
        name: b"cp949\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS,
        codepage: DBCS_KOR,
    },
    C2Rust_Unnamed_21 {
        name: b"cp950\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_DBCS,
        codepage: DBCS_CHT,
    },
    C2Rust_Unnamed_21 {
        name: b"cp1250\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 1250 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp1251\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 1251 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp1253\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 1253 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp1254\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 1254 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp1255\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 1255 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp1256\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 1256 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp1257\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 1257 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"cp1258\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
        codepage: 1258 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"macroman\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT + ENC_MACROMAN,
        codepage: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_21 {
        name: b"hp-roman8\0".as_ptr() as *const ::core::ffi::c_char,
        prop: ENC_8BIT,
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

pub(crate) static enc_alias_table: GlobalCell<[C2Rust_Unnamed_22; 64]> = GlobalCell::new([
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
