//! The canonical encoding names, and the aliases that resolve onto them.
//!
//! [`ENCODINGS`] is every encoding nvim knows by name, each with the property
//! bits `enc_canon_props` reports and the MS codepage it corresponds to (0
//! where there is none). The `IDX_*` constants are **indices into it**, so a
//! row may not be moved, inserted or removed without moving them with it.
//!
//! [`ENCODING_ALIASES`] maps every other spelling anyone writes onto one of
//! those indices. Both tables are searched linearly and by exact name, so
//! neither has to be sorted -- and `ENCODING_ALIASES` must **not** be, because
//! it contains `"950"` twice, resolving to `cp950` and then to `big5`, and the
//! first match is the one upstream answers with.
//!
//! `"iso-8859-n"` is deliberately absent from both: `enc_canonize` recognises
//! that shape directly, which is why the table only lists the ones it knows
//! individually.

#![forbid(unsafe_code)]

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use crate::mbyte::*;
use core::ffi::{CStr, c_int};

/// The DBCS codepages, as `enc_dbcs` numbered them. The `U` suffix is the
/// EUC form of the same character set.
pub const DBCS_JPN: c_int = 932;
pub const DBCS_JPNU: c_int = 9932;
pub const DBCS_KOR: c_int = 949;
pub const DBCS_KORU: c_int = 9949;
pub const DBCS_CHS: c_int = 936;
pub const DBCS_CHSU: c_int = 9936;
pub const DBCS_CHT: c_int = 950;
pub const DBCS_CHTU: c_int = 9950;
/// Not a real codepage: the `debug` encoding, for exercising the DBCS paths
/// on a Unix host.
pub const DBCS_DEBUG: c_int = -1;

/// One canonical encoding.
pub struct Encoding {
    /// The name nvim canonicalises to.
    pub name: &'static CStr,
    /// What kind of encoding it is: 8-bit, DBCS or Unicode, plus the byte
    /// order and unit width for the Unicode ones.
    pub prop: EncProps,
    /// The MS-DOS/Windows codepage, or 0 where there is none.
    ///
    /// **Nothing reads this.** It is upstream's, left in place because it is
    /// the only record of which codepage each name corresponds to; the DBCS
    /// support it was for does not exist in this port.
    pub codepage: c_int,
}

pub const IDX_LATIN_1: usize = 0;
pub const IDX_ISO_2: usize = 1;
pub const IDX_ISO_3: usize = 2;
pub const IDX_ISO_4: usize = 3;
pub const IDX_ISO_5: usize = 4;
pub const IDX_ISO_6: usize = 5;
pub const IDX_ISO_7: usize = 6;
pub const IDX_ISO_8: usize = 7;
pub const IDX_ISO_9: usize = 8;
pub const IDX_ISO_10: usize = 9;
pub const IDX_ISO_11: usize = 10;
pub const IDX_ISO_13: usize = 11;
pub const IDX_ISO_14: usize = 12;
pub const IDX_ISO_15: usize = 13;
pub const IDX_KOI8_R: usize = 14;
pub const IDX_KOI8_U: usize = 15;
pub const IDX_UTF8: usize = 16;
pub const IDX_UCS2: usize = 17;
pub const IDX_UCS2LE: usize = 18;
pub const IDX_UTF16: usize = 19;
pub const IDX_UTF16LE: usize = 20;
pub const IDX_UCS4: usize = 21;
pub const IDX_UCS4LE: usize = 22;
pub const IDX_DEBUG: usize = 23;
pub const IDX_EUC_JP: usize = 24;
pub const IDX_SJIS: usize = 25;
pub const IDX_EUC_KR: usize = 26;
pub const IDX_EUC_CN: usize = 27;
pub const IDX_EUC_TW: usize = 28;
pub const IDX_BIG5: usize = 29;
pub const IDX_CP437: usize = 30;
pub const IDX_CP737: usize = 31;
pub const IDX_CP775: usize = 32;
pub const IDX_CP850: usize = 33;
pub const IDX_CP852: usize = 34;
pub const IDX_CP855: usize = 35;
pub const IDX_CP857: usize = 36;
pub const IDX_CP860: usize = 37;
pub const IDX_CP861: usize = 38;
pub const IDX_CP862: usize = 39;
pub const IDX_CP863: usize = 40;
pub const IDX_CP865: usize = 41;
pub const IDX_CP866: usize = 42;
pub const IDX_CP869: usize = 43;
pub const IDX_CP874: usize = 44;
pub const IDX_CP932: usize = 45;
pub const IDX_CP936: usize = 46;
pub const IDX_CP949: usize = 47;
pub const IDX_CP950: usize = 48;
pub const IDX_CP1250: usize = 49;
pub const IDX_CP1251: usize = 50;
pub const IDX_CP1253: usize = 51;
pub const IDX_CP1254: usize = 52;
pub const IDX_CP1255: usize = 53;
pub const IDX_CP1256: usize = 54;
pub const IDX_CP1257: usize = 55;
pub const IDX_CP1258: usize = 56;
pub const IDX_MACROMAN: usize = 57;
pub const IDX_HPROMAN8: usize = 58;
pub const IDX_COUNT: usize = 59;

/// Every encoding nvim knows by name. Indexed by the `IDX_*` constants.
#[rustfmt::skip]
pub static ENCODINGS: [Encoding; IDX_COUNT] = [
    Encoding { name: c"latin1", prop: ENC_8BIT | ENC_LATIN1, codepage: 1252 },
    Encoding { name: c"iso-8859-2", prop: ENC_8BIT, codepage: 0 },
    Encoding { name: c"iso-8859-3", prop: ENC_8BIT, codepage: 0 },
    Encoding { name: c"iso-8859-4", prop: ENC_8BIT, codepage: 0 },
    Encoding { name: c"iso-8859-5", prop: ENC_8BIT, codepage: 0 },
    Encoding { name: c"iso-8859-6", prop: ENC_8BIT, codepage: 0 },
    Encoding { name: c"iso-8859-7", prop: ENC_8BIT, codepage: 0 },
    Encoding { name: c"iso-8859-8", prop: ENC_8BIT, codepage: 0 },
    Encoding { name: c"iso-8859-9", prop: ENC_8BIT, codepage: 0 },
    Encoding { name: c"iso-8859-10", prop: ENC_8BIT, codepage: 0 },
    Encoding { name: c"iso-8859-11", prop: ENC_8BIT, codepage: 0 },
    Encoding { name: c"iso-8859-13", prop: ENC_8BIT, codepage: 0 },
    Encoding { name: c"iso-8859-14", prop: ENC_8BIT, codepage: 0 },
    Encoding { name: c"iso-8859-15", prop: ENC_8BIT | ENC_LATIN9, codepage: 0 },
    Encoding { name: c"koi8-r", prop: ENC_8BIT, codepage: 0 },
    Encoding { name: c"koi8-u", prop: ENC_8BIT, codepage: 0 },
    Encoding { name: c"utf-8", prop: ENC_UNICODE, codepage: 0 },
    Encoding { name: c"ucs-2", prop: ENC_UNICODE | ENC_ENDIAN_B | ENC_2BYTE, codepage: 0 },
    Encoding { name: c"ucs-2le", prop: ENC_UNICODE | ENC_ENDIAN_L | ENC_2BYTE, codepage: 0 },
    Encoding { name: c"utf-16", prop: ENC_UNICODE | ENC_ENDIAN_B | ENC_2WORD, codepage: 0 },
    Encoding { name: c"utf-16le", prop: ENC_UNICODE | ENC_ENDIAN_L | ENC_2WORD, codepage: 0 },
    Encoding { name: c"ucs-4", prop: ENC_UNICODE | ENC_ENDIAN_B | ENC_4BYTE, codepage: 0 },
    Encoding { name: c"ucs-4le", prop: ENC_UNICODE | ENC_ENDIAN_L | ENC_4BYTE, codepage: 0 },
    // For debugging DBCS encoding on Unix.
    Encoding { name: c"debug", prop: ENC_DBCS, codepage: DBCS_DEBUG },
    Encoding { name: c"euc-jp", prop: ENC_DBCS, codepage: DBCS_JPNU },
    Encoding { name: c"sjis", prop: ENC_DBCS, codepage: DBCS_JPN },
    Encoding { name: c"euc-kr", prop: ENC_DBCS, codepage: DBCS_KORU },
    Encoding { name: c"euc-cn", prop: ENC_DBCS, codepage: DBCS_CHSU },
    Encoding { name: c"euc-tw", prop: ENC_DBCS, codepage: DBCS_CHTU },
    Encoding { name: c"big5", prop: ENC_DBCS, codepage: DBCS_CHT },
    // MS-DOS and MS-Windows codepages are included here, so that they can be
    // used on Unix too.  Most of them are similar to ISO-8859 encodings, but
    // not exactly the same.
    Encoding { name: c"cp437", prop: ENC_8BIT, codepage: 437 },                                 // like iso-8859-1
    Encoding { name: c"cp737", prop: ENC_8BIT, codepage: 737 },                                 // like iso-8859-7
    Encoding { name: c"cp775", prop: ENC_8BIT, codepage: 775 },                                 // Baltic
    Encoding { name: c"cp850", prop: ENC_8BIT, codepage: 850 },                                 // like iso-8859-4
    Encoding { name: c"cp852", prop: ENC_8BIT, codepage: 852 },                                 // like iso-8859-1
    Encoding { name: c"cp855", prop: ENC_8BIT, codepage: 855 },                                 // like iso-8859-2
    Encoding { name: c"cp857", prop: ENC_8BIT, codepage: 857 },                                 // like iso-8859-5
    Encoding { name: c"cp860", prop: ENC_8BIT, codepage: 860 },                                 // like iso-8859-9
    Encoding { name: c"cp861", prop: ENC_8BIT, codepage: 861 },                                 // like iso-8859-1
    Encoding { name: c"cp862", prop: ENC_8BIT, codepage: 862 },                                 // like iso-8859-1
    Encoding { name: c"cp863", prop: ENC_8BIT, codepage: 863 },                                 // like iso-8859-8
    Encoding { name: c"cp865", prop: ENC_8BIT, codepage: 865 },                                 // like iso-8859-1
    Encoding { name: c"cp866", prop: ENC_8BIT, codepage: 866 },                                 // like iso-8859-5
    Encoding { name: c"cp869", prop: ENC_8BIT, codepage: 869 },                                 // like iso-8859-7
    Encoding { name: c"cp874", prop: ENC_8BIT, codepage: 874 },                                 // Thai
    Encoding { name: c"cp932", prop: ENC_DBCS, codepage: DBCS_JPN },
    Encoding { name: c"cp936", prop: ENC_DBCS, codepage: DBCS_CHS },
    Encoding { name: c"cp949", prop: ENC_DBCS, codepage: DBCS_KOR },
    Encoding { name: c"cp950", prop: ENC_DBCS, codepage: DBCS_CHT },
    Encoding { name: c"cp1250", prop: ENC_8BIT, codepage: 1250 },                               // Czech, Polish, etc.
    Encoding { name: c"cp1251", prop: ENC_8BIT, codepage: 1251 },                               // Cyrillic
    // cp1252 is considered to be equal to latin1
    Encoding { name: c"cp1253", prop: ENC_8BIT, codepage: 1253 },                               // Greek
    Encoding { name: c"cp1254", prop: ENC_8BIT, codepage: 1254 },                               // Turkish
    Encoding { name: c"cp1255", prop: ENC_8BIT, codepage: 1255 },                               // Hebrew
    Encoding { name: c"cp1256", prop: ENC_8BIT, codepage: 1256 },                               // Arabic
    Encoding { name: c"cp1257", prop: ENC_8BIT, codepage: 1257 },                               // Baltic
    Encoding { name: c"cp1258", prop: ENC_8BIT, codepage: 1258 },                               // Vietnamese
    Encoding { name: c"macroman", prop: ENC_8BIT | ENC_MACROMAN, codepage: 0 },                 // Mac OS
    Encoding { name: c"hp-roman8", prop: ENC_8BIT, codepage: 0 },                               // HP Roman8
];

/// Alternative spellings, each resolving to an index into [`ENCODINGS`].
///
/// Order matters: the first match wins, and `"950"` appears twice.
#[rustfmt::skip]
pub static ENCODING_ALIASES: [(&CStr, usize); 63] = [
    (c"ansi", IDX_LATIN_1),
    (c"iso-8859-1", IDX_LATIN_1),
    (c"latin2", IDX_ISO_2),
    (c"latin3", IDX_ISO_3),
    (c"latin4", IDX_ISO_4),
    (c"cyrillic", IDX_ISO_5),
    (c"arabic", IDX_ISO_6),
    (c"greek", IDX_ISO_7),
    (c"hebrew", IDX_ISO_8),
    (c"latin5", IDX_ISO_9),
    (c"turkish", IDX_ISO_9),        // ?
    (c"latin6", IDX_ISO_10),
    (c"nordic", IDX_ISO_10),        // ?
    (c"thai", IDX_ISO_11),          // ?
    (c"latin7", IDX_ISO_13),
    (c"latin8", IDX_ISO_14),
    (c"latin9", IDX_ISO_15),
    (c"utf8", IDX_UTF8),
    (c"unicode", IDX_UCS2),
    (c"ucs2", IDX_UCS2),
    (c"ucs2be", IDX_UCS2),
    (c"ucs-2be", IDX_UCS2),
    (c"ucs2le", IDX_UCS2LE),
    (c"utf16", IDX_UTF16),
    (c"utf16be", IDX_UTF16),
    (c"utf-16be", IDX_UTF16),
    (c"utf16le", IDX_UTF16LE),
    (c"ucs4", IDX_UCS4),
    (c"ucs4be", IDX_UCS4),
    (c"ucs-4be", IDX_UCS4),
    (c"ucs4le", IDX_UCS4LE),
    (c"utf32", IDX_UCS4),
    (c"utf-32", IDX_UCS4),
    (c"utf32be", IDX_UCS4),
    (c"utf-32be", IDX_UCS4),
    (c"utf32le", IDX_UCS4LE),
    (c"utf-32le", IDX_UCS4LE),
    (c"932", IDX_CP932),
    (c"949", IDX_CP949),
    (c"936", IDX_CP936),
    (c"gbk", IDX_CP936),
    (c"950", IDX_CP950),
    (c"eucjp", IDX_EUC_JP),
    (c"unix-jis", IDX_EUC_JP),
    (c"ujis", IDX_EUC_JP),
    (c"shift-jis", IDX_SJIS),
    (c"pck", IDX_SJIS),             // Sun: PCK
    (c"euckr", IDX_EUC_KR),
    (c"5601", IDX_EUC_KR),          // Sun: KS C 5601
    (c"euccn", IDX_EUC_CN),
    (c"gb2312", IDX_EUC_CN),
    (c"euctw", IDX_EUC_TW),
    (c"japan", IDX_EUC_JP),
    (c"korea", IDX_EUC_KR),
    (c"prc", IDX_EUC_CN),
    (c"zh-cn", IDX_EUC_CN),
    (c"chinese", IDX_EUC_CN),
    (c"zh-tw", IDX_EUC_TW),
    (c"taiwan", IDX_EUC_TW),
    (c"cp950", IDX_BIG5),
    (c"950", IDX_BIG5),
    (c"mac", IDX_MACROMAN),
    (c"mac-roman", IDX_MACROMAN),
];
