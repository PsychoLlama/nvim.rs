//! Digraph default data table.
//!
//! Contains the ~1400-entry `digraphdefault[]` array, migrated from C.

use libc::c_int;

use crate::DigrT;

// DG_START_* constants — must match C #define values in digraph.c.
// Verified by _Static_assert in C and test_dg_start_constants in Rust.
pub const DG_START_LATIN: c_int = 0xa1;
pub const DG_START_GREEK: c_int = 0x0386;
pub const DG_START_CYRILLIC: c_int = 0x0401;
pub const DG_START_HEBREW: c_int = 0x05d0;
pub const DG_START_ARABIC: c_int = 0x060c;
pub const DG_START_LATIN_EXTENDED: c_int = 0x1e02;
pub const DG_START_GREEK_EXTENDED: c_int = 0x1f00;
pub const DG_START_PUNCTUATION: c_int = 0x2002;
pub const DG_START_SUB_SUPER: c_int = 0x2070;
pub const DG_START_CURRENCY: c_int = 0x20a4;
pub const DG_START_OTHER1: c_int = 0x2103;
pub const DG_START_ROMAN: c_int = 0x2160;
pub const DG_START_ARROWS: c_int = 0x2190;
pub const DG_START_MATH: c_int = 0x2200;
pub const DG_START_TECHNICAL: c_int = 0x2302;
pub const DG_START_OTHER2: c_int = 0x2423;
pub const DG_START_DRAWING: c_int = 0x2500;
pub const DG_START_BLOCK: c_int = 0x2580;
pub const DG_START_SHAPES: c_int = 0x25a0;
pub const DG_START_SYMBOLS: c_int = 0x2605;
pub const DG_START_DINGBATS: c_int = 0x2713;
pub const DG_START_CJK_SYMBOLS: c_int = 0x3000;
pub const DG_START_HIRAGANA: c_int = 0x3041;
pub const DG_START_KATAKANA: c_int = 0x30a1;
pub const DG_START_BOPOMOFO: c_int = 0x3105;
pub const DG_START_OTHER3: c_int = 0x3220;

/// The default digraph table (~1400 entries from RFC1345).
///
/// Unlike the C version, this does NOT have a NUL sentinel — use slice length.
pub static DIGRAPH_DEFAULT: &[DigrT] = &[
    DigrT {
        char1: b'N',
        char2: b'U',
        result: 0x0a,
    },
    DigrT {
        char1: b'S',
        char2: b'H',
        result: 0x01,
    },
    DigrT {
        char1: b'S',
        char2: b'X',
        result: 0x02,
    },
    DigrT {
        char1: b'E',
        char2: b'X',
        result: 0x03,
    },
    DigrT {
        char1: b'E',
        char2: b'T',
        result: 0x04,
    },
    DigrT {
        char1: b'E',
        char2: b'Q',
        result: 0x05,
    },
    DigrT {
        char1: b'A',
        char2: b'K',
        result: 0x06,
    },
    DigrT {
        char1: b'B',
        char2: b'L',
        result: 0x07,
    },
    DigrT {
        char1: b'B',
        char2: b'S',
        result: 0x08,
    },
    DigrT {
        char1: b'H',
        char2: b'T',
        result: 0x09,
    },
    DigrT {
        char1: b'L',
        char2: b'F',
        result: 0x0a,
    },
    DigrT {
        char1: b'V',
        char2: b'T',
        result: 0x0b,
    },
    DigrT {
        char1: b'F',
        char2: b'F',
        result: 0x0c,
    },
    DigrT {
        char1: b'C',
        char2: b'R',
        result: 0x0d,
    },
    DigrT {
        char1: b'S',
        char2: b'O',
        result: 0x0e,
    },
    DigrT {
        char1: b'S',
        char2: b'I',
        result: 0x0f,
    },
    DigrT {
        char1: b'D',
        char2: b'L',
        result: 0x10,
    },
    DigrT {
        char1: b'D',
        char2: b'1',
        result: 0x11,
    },
    DigrT {
        char1: b'D',
        char2: b'2',
        result: 0x12,
    },
    DigrT {
        char1: b'D',
        char2: b'3',
        result: 0x13,
    },
    DigrT {
        char1: b'D',
        char2: b'4',
        result: 0x14,
    },
    DigrT {
        char1: b'N',
        char2: b'K',
        result: 0x15,
    },
    DigrT {
        char1: b'S',
        char2: b'Y',
        result: 0x16,
    },
    DigrT {
        char1: b'E',
        char2: b'B',
        result: 0x17,
    },
    DigrT {
        char1: b'C',
        char2: b'N',
        result: 0x18,
    },
    DigrT {
        char1: b'E',
        char2: b'M',
        result: 0x19,
    },
    DigrT {
        char1: b'S',
        char2: b'B',
        result: 0x1a,
    },
    DigrT {
        char1: b'E',
        char2: b'C',
        result: 0x1b,
    },
    DigrT {
        char1: b'F',
        char2: b'S',
        result: 0x1c,
    },
    DigrT {
        char1: b'G',
        char2: b'S',
        result: 0x1d,
    },
    DigrT {
        char1: b'R',
        char2: b'S',
        result: 0x1e,
    },
    DigrT {
        char1: b'U',
        char2: b'S',
        result: 0x1f,
    },
    DigrT {
        char1: b'S',
        char2: b'P',
        result: 0x20,
    },
    DigrT {
        char1: b'N',
        char2: b'b',
        result: 0x23,
    },
    DigrT {
        char1: b'D',
        char2: b'O',
        result: 0x24,
    },
    DigrT {
        char1: b'A',
        char2: b't',
        result: 0x40,
    },
    DigrT {
        char1: b'<',
        char2: b'(',
        result: 0x5b,
    },
    DigrT {
        char1: b'/',
        char2: b'/',
        result: 0x5c,
    },
    DigrT {
        char1: b')',
        char2: b'>',
        result: 0x5d,
    },
    DigrT {
        char1: b'\'',
        char2: b'>',
        result: 0x5e,
    },
    DigrT {
        char1: b'\'',
        char2: b'!',
        result: 0x60,
    },
    DigrT {
        char1: b'(',
        char2: b'!',
        result: 0x7b,
    },
    DigrT {
        char1: b'!',
        char2: b'!',
        result: 0x7c,
    },
    DigrT {
        char1: b'!',
        char2: b')',
        result: 0x7d,
    },
    DigrT {
        char1: b'\'',
        char2: b'?',
        result: 0x7e,
    },
    DigrT {
        char1: b'D',
        char2: b'T',
        result: 0x7f,
    },
    DigrT {
        char1: b'P',
        char2: b'A',
        result: 0x80,
    },
    DigrT {
        char1: b'H',
        char2: b'O',
        result: 0x81,
    },
    DigrT {
        char1: b'B',
        char2: b'H',
        result: 0x82,
    },
    DigrT {
        char1: b'N',
        char2: b'H',
        result: 0x83,
    },
    DigrT {
        char1: b'I',
        char2: b'N',
        result: 0x84,
    },
    DigrT {
        char1: b'N',
        char2: b'L',
        result: 0x85,
    },
    DigrT {
        char1: b'S',
        char2: b'A',
        result: 0x86,
    },
    DigrT {
        char1: b'E',
        char2: b'S',
        result: 0x87,
    },
    DigrT {
        char1: b'H',
        char2: b'S',
        result: 0x88,
    },
    DigrT {
        char1: b'H',
        char2: b'J',
        result: 0x89,
    },
    DigrT {
        char1: b'V',
        char2: b'S',
        result: 0x8a,
    },
    DigrT {
        char1: b'P',
        char2: b'D',
        result: 0x8b,
    },
    DigrT {
        char1: b'P',
        char2: b'U',
        result: 0x8c,
    },
    DigrT {
        char1: b'R',
        char2: b'I',
        result: 0x8d,
    },
    DigrT {
        char1: b'S',
        char2: b'2',
        result: 0x8e,
    },
    DigrT {
        char1: b'S',
        char2: b'3',
        result: 0x8f,
    },
    DigrT {
        char1: b'D',
        char2: b'C',
        result: 0x90,
    },
    DigrT {
        char1: b'P',
        char2: b'1',
        result: 0x91,
    },
    DigrT {
        char1: b'P',
        char2: b'2',
        result: 0x92,
    },
    DigrT {
        char1: b'T',
        char2: b'S',
        result: 0x93,
    },
    DigrT {
        char1: b'C',
        char2: b'C',
        result: 0x94,
    },
    DigrT {
        char1: b'M',
        char2: b'W',
        result: 0x95,
    },
    DigrT {
        char1: b'S',
        char2: b'G',
        result: 0x96,
    },
    DigrT {
        char1: b'E',
        char2: b'G',
        result: 0x97,
    },
    DigrT {
        char1: b'S',
        char2: b'S',
        result: 0x98,
    },
    DigrT {
        char1: b'G',
        char2: b'C',
        result: 0x99,
    },
    DigrT {
        char1: b'S',
        char2: b'C',
        result: 0x9a,
    },
    DigrT {
        char1: b'C',
        char2: b'I',
        result: 0x9b,
    },
    DigrT {
        char1: b'S',
        char2: b'T',
        result: 0x9c,
    },
    DigrT {
        char1: b'O',
        char2: b'C',
        result: 0x9d,
    },
    DigrT {
        char1: b'P',
        char2: b'M',
        result: 0x9e,
    },
    DigrT {
        char1: b'A',
        char2: b'C',
        result: 0x9f,
    },
    DigrT {
        char1: b'N',
        char2: b'S',
        result: 0xa0,
    },
    DigrT {
        char1: b'!',
        char2: b'I',
        result: 0xa1,
    },
    DigrT {
        char1: b'~',
        char2: b'!',
        result: 0xa1,
    },
    DigrT {
        char1: b'C',
        char2: b't',
        result: 0xa2,
    },
    DigrT {
        char1: b'c',
        char2: b'|',
        result: 0xa2,
    },
    DigrT {
        char1: b'P',
        char2: b'd',
        result: 0xa3,
    },
    DigrT {
        char1: b'$',
        char2: b'$',
        result: 0xa3,
    },
    DigrT {
        char1: b'C',
        char2: b'u',
        result: 0xa4,
    },
    DigrT {
        char1: b'o',
        char2: b'x',
        result: 0xa4,
    },
    DigrT {
        char1: b'Y',
        char2: b'e',
        result: 0xa5,
    },
    DigrT {
        char1: b'Y',
        char2: b'-',
        result: 0xa5,
    },
    DigrT {
        char1: b'B',
        char2: b'B',
        result: 0xa6,
    },
    DigrT {
        char1: b'|',
        char2: b'|',
        result: 0xa6,
    },
    DigrT {
        char1: b'S',
        char2: b'E',
        result: 0xa7,
    },
    DigrT {
        char1: b'\'',
        char2: b':',
        result: 0xa8,
    },
    DigrT {
        char1: b'C',
        char2: b'o',
        result: 0xa9,
    },
    DigrT {
        char1: b'c',
        char2: b'O',
        result: 0xa9,
    },
    DigrT {
        char1: b'-',
        char2: b'a',
        result: 0xaa,
    },
    DigrT {
        char1: b'<',
        char2: b'<',
        result: 0xab,
    },
    DigrT {
        char1: b'N',
        char2: b'O',
        result: 0xac,
    },
    DigrT {
        char1: b'-',
        char2: b',',
        result: 0xac,
    },
    DigrT {
        char1: b'-',
        char2: b'-',
        result: 0xad,
    },
    DigrT {
        char1: b'R',
        char2: b'g',
        result: 0xae,
    },
    DigrT {
        char1: b'\'',
        char2: b'm',
        result: 0xaf,
    },
    DigrT {
        char1: b'-',
        char2: b'=',
        result: 0xaf,
    },
    DigrT {
        char1: b'D',
        char2: b'G',
        result: 0xb0,
    },
    DigrT {
        char1: b'~',
        char2: b'o',
        result: 0xb0,
    },
    DigrT {
        char1: b'+',
        char2: b'-',
        result: 0xb1,
    },
    DigrT {
        char1: b'2',
        char2: b'S',
        result: 0xb2,
    },
    DigrT {
        char1: b'2',
        char2: b'2',
        result: 0xb2,
    },
    DigrT {
        char1: b'3',
        char2: b'S',
        result: 0xb3,
    },
    DigrT {
        char1: b'3',
        char2: b'3',
        result: 0xb3,
    },
    DigrT {
        char1: b'\'',
        char2: b'\'',
        result: 0xb4,
    },
    DigrT {
        char1: b'M',
        char2: b'y',
        result: 0xb5,
    },
    DigrT {
        char1: b'P',
        char2: b'I',
        result: 0xb6,
    },
    DigrT {
        char1: b'p',
        char2: b'p',
        result: 0xb6,
    },
    DigrT {
        char1: b'.',
        char2: b'M',
        result: 0xb7,
    },
    DigrT {
        char1: b'~',
        char2: b'.',
        result: 0xb7,
    },
    DigrT {
        char1: b'\'',
        char2: b',',
        result: 0xb8,
    },
    DigrT {
        char1: b'1',
        char2: b'S',
        result: 0xb9,
    },
    DigrT {
        char1: b'1',
        char2: b'1',
        result: 0xb9,
    },
    DigrT {
        char1: b'-',
        char2: b'o',
        result: 0xba,
    },
    DigrT {
        char1: b'>',
        char2: b'>',
        result: 0xbb,
    },
    DigrT {
        char1: b'1',
        char2: b'4',
        result: 0xbc,
    },
    DigrT {
        char1: b'1',
        char2: b'2',
        result: 0xbd,
    },
    DigrT {
        char1: b'3',
        char2: b'4',
        result: 0xbe,
    },
    DigrT {
        char1: b'?',
        char2: b'I',
        result: 0xbf,
    },
    DigrT {
        char1: b'~',
        char2: b'?',
        result: 0xbf,
    },
    DigrT {
        char1: b'A',
        char2: b'!',
        result: 0xc0,
    },
    DigrT {
        char1: b'A',
        char2: b'`',
        result: 0xc0,
    },
    DigrT {
        char1: b'A',
        char2: b'\'',
        result: 0xc1,
    },
    DigrT {
        char1: b'A',
        char2: b'>',
        result: 0xc2,
    },
    DigrT {
        char1: b'A',
        char2: b'^',
        result: 0xc2,
    },
    DigrT {
        char1: b'A',
        char2: b'?',
        result: 0xc3,
    },
    DigrT {
        char1: b'A',
        char2: b'~',
        result: 0xc3,
    },
    DigrT {
        char1: b'A',
        char2: b':',
        result: 0xc4,
    },
    DigrT {
        char1: b'A',
        char2: b'"',
        result: 0xc4,
    },
    DigrT {
        char1: b'A',
        char2: b'A',
        result: 0xc5,
    },
    DigrT {
        char1: b'A',
        char2: b'@',
        result: 0xc5,
    },
    DigrT {
        char1: b'A',
        char2: b'E',
        result: 0xc6,
    },
    DigrT {
        char1: b'C',
        char2: b',',
        result: 0xc7,
    },
    DigrT {
        char1: b'E',
        char2: b'!',
        result: 0xc8,
    },
    DigrT {
        char1: b'E',
        char2: b'`',
        result: 0xc8,
    },
    DigrT {
        char1: b'E',
        char2: b'\'',
        result: 0xc9,
    },
    DigrT {
        char1: b'E',
        char2: b'>',
        result: 0xca,
    },
    DigrT {
        char1: b'E',
        char2: b'^',
        result: 0xca,
    },
    DigrT {
        char1: b'E',
        char2: b':',
        result: 0xcb,
    },
    DigrT {
        char1: b'E',
        char2: b'"',
        result: 0xcb,
    },
    DigrT {
        char1: b'I',
        char2: b'!',
        result: 0xcc,
    },
    DigrT {
        char1: b'I',
        char2: b'`',
        result: 0xcc,
    },
    DigrT {
        char1: b'I',
        char2: b'\'',
        result: 0xcd,
    },
    DigrT {
        char1: b'I',
        char2: b'>',
        result: 0xce,
    },
    DigrT {
        char1: b'I',
        char2: b'^',
        result: 0xce,
    },
    DigrT {
        char1: b'I',
        char2: b':',
        result: 0xcf,
    },
    DigrT {
        char1: b'I',
        char2: b'"',
        result: 0xcf,
    },
    DigrT {
        char1: b'D',
        char2: b'-',
        result: 0xd0,
    },
    DigrT {
        char1: b'N',
        char2: b'?',
        result: 0xd1,
    },
    DigrT {
        char1: b'N',
        char2: b'~',
        result: 0xd1,
    },
    DigrT {
        char1: b'O',
        char2: b'!',
        result: 0xd2,
    },
    DigrT {
        char1: b'O',
        char2: b'`',
        result: 0xd2,
    },
    DigrT {
        char1: b'O',
        char2: b'\'',
        result: 0xd3,
    },
    DigrT {
        char1: b'O',
        char2: b'>',
        result: 0xd4,
    },
    DigrT {
        char1: b'O',
        char2: b'^',
        result: 0xd4,
    },
    DigrT {
        char1: b'O',
        char2: b'?',
        result: 0xd5,
    },
    DigrT {
        char1: b'O',
        char2: b'~',
        result: 0xd5,
    },
    DigrT {
        char1: b'O',
        char2: b':',
        result: 0xd6,
    },
    DigrT {
        char1: b'*',
        char2: b'X',
        result: 0xd7,
    },
    DigrT {
        char1: b'/',
        char2: b'\\',
        result: 0xd7,
    },
    DigrT {
        char1: b'O',
        char2: b'/',
        result: 0xd8,
    },
    DigrT {
        char1: b'U',
        char2: b'!',
        result: 0xd9,
    },
    DigrT {
        char1: b'U',
        char2: b'`',
        result: 0xd9,
    },
    DigrT {
        char1: b'U',
        char2: b'\'',
        result: 0xda,
    },
    DigrT {
        char1: b'U',
        char2: b'>',
        result: 0xdb,
    },
    DigrT {
        char1: b'U',
        char2: b'^',
        result: 0xdb,
    },
    DigrT {
        char1: b'U',
        char2: b':',
        result: 0xdc,
    },
    DigrT {
        char1: b'Y',
        char2: b'\'',
        result: 0xdd,
    },
    DigrT {
        char1: b'T',
        char2: b'H',
        result: 0xde,
    },
    DigrT {
        char1: b'I',
        char2: b'p',
        result: 0xde,
    },
    DigrT {
        char1: b's',
        char2: b's',
        result: 0xdf,
    },
    DigrT {
        char1: b'a',
        char2: b'!',
        result: 0xe0,
    },
    DigrT {
        char1: b'a',
        char2: b'`',
        result: 0xe0,
    },
    DigrT {
        char1: b'a',
        char2: b'\'',
        result: 0xe1,
    },
    DigrT {
        char1: b'a',
        char2: b'>',
        result: 0xe2,
    },
    DigrT {
        char1: b'a',
        char2: b'^',
        result: 0xe2,
    },
    DigrT {
        char1: b'a',
        char2: b'?',
        result: 0xe3,
    },
    DigrT {
        char1: b'a',
        char2: b'~',
        result: 0xe3,
    },
    DigrT {
        char1: b'a',
        char2: b':',
        result: 0xe4,
    },
    DigrT {
        char1: b'a',
        char2: b'"',
        result: 0xe4,
    },
    DigrT {
        char1: b'a',
        char2: b'a',
        result: 0xe5,
    },
    DigrT {
        char1: b'a',
        char2: b'@',
        result: 0xe5,
    },
    DigrT {
        char1: b'a',
        char2: b'e',
        result: 0xe6,
    },
    DigrT {
        char1: b'c',
        char2: b',',
        result: 0xe7,
    },
    DigrT {
        char1: b'e',
        char2: b'!',
        result: 0xe8,
    },
    DigrT {
        char1: b'e',
        char2: b'`',
        result: 0xe8,
    },
    DigrT {
        char1: b'e',
        char2: b'\'',
        result: 0xe9,
    },
    DigrT {
        char1: b'e',
        char2: b'>',
        result: 0xea,
    },
    DigrT {
        char1: b'e',
        char2: b'^',
        result: 0xea,
    },
    DigrT {
        char1: b'e',
        char2: b':',
        result: 0xeb,
    },
    DigrT {
        char1: b'e',
        char2: b'"',
        result: 0xeb,
    },
    DigrT {
        char1: b'i',
        char2: b'!',
        result: 0xec,
    },
    DigrT {
        char1: b'i',
        char2: b'`',
        result: 0xec,
    },
    DigrT {
        char1: b'i',
        char2: b'\'',
        result: 0xed,
    },
    DigrT {
        char1: b'i',
        char2: b'>',
        result: 0xee,
    },
    DigrT {
        char1: b'i',
        char2: b'^',
        result: 0xee,
    },
    DigrT {
        char1: b'i',
        char2: b':',
        result: 0xef,
    },
    DigrT {
        char1: b'd',
        char2: b'-',
        result: 0xf0,
    },
    DigrT {
        char1: b'n',
        char2: b'?',
        result: 0xf1,
    },
    DigrT {
        char1: b'n',
        char2: b'~',
        result: 0xf1,
    },
    DigrT {
        char1: b'o',
        char2: b'!',
        result: 0xf2,
    },
    DigrT {
        char1: b'o',
        char2: b'`',
        result: 0xf2,
    },
    DigrT {
        char1: b'o',
        char2: b'\'',
        result: 0xf3,
    },
    DigrT {
        char1: b'o',
        char2: b'>',
        result: 0xf4,
    },
    DigrT {
        char1: b'o',
        char2: b'^',
        result: 0xf4,
    },
    DigrT {
        char1: b'o',
        char2: b'?',
        result: 0xf5,
    },
    DigrT {
        char1: b'o',
        char2: b'~',
        result: 0xf5,
    },
    DigrT {
        char1: b'o',
        char2: b':',
        result: 0xf6,
    },
    DigrT {
        char1: b'-',
        char2: b':',
        result: 0xf7,
    },
    DigrT {
        char1: b'o',
        char2: b'/',
        result: 0xf8,
    },
    DigrT {
        char1: b'u',
        char2: b'!',
        result: 0xf9,
    },
    DigrT {
        char1: b'u',
        char2: b'`',
        result: 0xf9,
    },
    DigrT {
        char1: b'u',
        char2: b'\'',
        result: 0xfa,
    },
    DigrT {
        char1: b'u',
        char2: b'>',
        result: 0xfb,
    },
    DigrT {
        char1: b'u',
        char2: b'^',
        result: 0xfb,
    },
    DigrT {
        char1: b'u',
        char2: b':',
        result: 0xfc,
    },
    DigrT {
        char1: b'y',
        char2: b'\'',
        result: 0xfd,
    },
    DigrT {
        char1: b't',
        char2: b'h',
        result: 0xfe,
    },
    DigrT {
        char1: b'y',
        char2: b':',
        result: 0xff,
    },
    DigrT {
        char1: b'y',
        char2: b'"',
        result: 0xff,
    },
    DigrT {
        char1: b'A',
        char2: b'-',
        result: 0x0100,
    },
    DigrT {
        char1: b'a',
        char2: b'-',
        result: 0x0101,
    },
    DigrT {
        char1: b'A',
        char2: b'(',
        result: 0x0102,
    },
    DigrT {
        char1: b'a',
        char2: b'(',
        result: 0x0103,
    },
    DigrT {
        char1: b'A',
        char2: b';',
        result: 0x0104,
    },
    DigrT {
        char1: b'a',
        char2: b';',
        result: 0x0105,
    },
    DigrT {
        char1: b'C',
        char2: b'\'',
        result: 0x0106,
    },
    DigrT {
        char1: b'c',
        char2: b'\'',
        result: 0x0107,
    },
    DigrT {
        char1: b'C',
        char2: b'>',
        result: 0x0108,
    },
    DigrT {
        char1: b'c',
        char2: b'>',
        result: 0x0109,
    },
    DigrT {
        char1: b'C',
        char2: b'.',
        result: 0x010a,
    },
    DigrT {
        char1: b'c',
        char2: b'.',
        result: 0x010b,
    },
    DigrT {
        char1: b'C',
        char2: b'<',
        result: 0x010c,
    },
    DigrT {
        char1: b'c',
        char2: b'<',
        result: 0x010d,
    },
    DigrT {
        char1: b'D',
        char2: b'<',
        result: 0x010e,
    },
    DigrT {
        char1: b'd',
        char2: b'<',
        result: 0x010f,
    },
    DigrT {
        char1: b'D',
        char2: b'/',
        result: 0x0110,
    },
    DigrT {
        char1: b'd',
        char2: b'/',
        result: 0x0111,
    },
    DigrT {
        char1: b'E',
        char2: b'-',
        result: 0x0112,
    },
    DigrT {
        char1: b'e',
        char2: b'-',
        result: 0x0113,
    },
    DigrT {
        char1: b'E',
        char2: b'(',
        result: 0x0114,
    },
    DigrT {
        char1: b'e',
        char2: b'(',
        result: 0x0115,
    },
    DigrT {
        char1: b'E',
        char2: b'.',
        result: 0x0116,
    },
    DigrT {
        char1: b'e',
        char2: b'.',
        result: 0x0117,
    },
    DigrT {
        char1: b'E',
        char2: b';',
        result: 0x0118,
    },
    DigrT {
        char1: b'e',
        char2: b';',
        result: 0x0119,
    },
    DigrT {
        char1: b'E',
        char2: b'<',
        result: 0x011a,
    },
    DigrT {
        char1: b'e',
        char2: b'<',
        result: 0x011b,
    },
    DigrT {
        char1: b'G',
        char2: b'>',
        result: 0x011c,
    },
    DigrT {
        char1: b'g',
        char2: b'>',
        result: 0x011d,
    },
    DigrT {
        char1: b'G',
        char2: b'(',
        result: 0x011e,
    },
    DigrT {
        char1: b'g',
        char2: b'(',
        result: 0x011f,
    },
    DigrT {
        char1: b'G',
        char2: b'.',
        result: 0x0120,
    },
    DigrT {
        char1: b'g',
        char2: b'.',
        result: 0x0121,
    },
    DigrT {
        char1: b'G',
        char2: b',',
        result: 0x0122,
    },
    DigrT {
        char1: b'g',
        char2: b',',
        result: 0x0123,
    },
    DigrT {
        char1: b'H',
        char2: b'>',
        result: 0x0124,
    },
    DigrT {
        char1: b'h',
        char2: b'>',
        result: 0x0125,
    },
    DigrT {
        char1: b'H',
        char2: b'/',
        result: 0x0126,
    },
    DigrT {
        char1: b'h',
        char2: b'/',
        result: 0x0127,
    },
    DigrT {
        char1: b'I',
        char2: b'?',
        result: 0x0128,
    },
    DigrT {
        char1: b'i',
        char2: b'?',
        result: 0x0129,
    },
    DigrT {
        char1: b'I',
        char2: b'-',
        result: 0x012a,
    },
    DigrT {
        char1: b'i',
        char2: b'-',
        result: 0x012b,
    },
    DigrT {
        char1: b'I',
        char2: b'(',
        result: 0x012c,
    },
    DigrT {
        char1: b'i',
        char2: b'(',
        result: 0x012d,
    },
    DigrT {
        char1: b'I',
        char2: b';',
        result: 0x012e,
    },
    DigrT {
        char1: b'i',
        char2: b';',
        result: 0x012f,
    },
    DigrT {
        char1: b'I',
        char2: b'.',
        result: 0x0130,
    },
    DigrT {
        char1: b'i',
        char2: b'.',
        result: 0x0131,
    },
    DigrT {
        char1: b'I',
        char2: b'J',
        result: 0x0132,
    },
    DigrT {
        char1: b'i',
        char2: b'j',
        result: 0x0133,
    },
    DigrT {
        char1: b'J',
        char2: b'>',
        result: 0x0134,
    },
    DigrT {
        char1: b'j',
        char2: b'>',
        result: 0x0135,
    },
    DigrT {
        char1: b'K',
        char2: b',',
        result: 0x0136,
    },
    DigrT {
        char1: b'k',
        char2: b',',
        result: 0x0137,
    },
    DigrT {
        char1: b'k',
        char2: b'k',
        result: 0x0138,
    },
    DigrT {
        char1: b'L',
        char2: b'\'',
        result: 0x0139,
    },
    DigrT {
        char1: b'l',
        char2: b'\'',
        result: 0x013a,
    },
    DigrT {
        char1: b'L',
        char2: b',',
        result: 0x013b,
    },
    DigrT {
        char1: b'l',
        char2: b',',
        result: 0x013c,
    },
    DigrT {
        char1: b'L',
        char2: b'<',
        result: 0x013d,
    },
    DigrT {
        char1: b'l',
        char2: b'<',
        result: 0x013e,
    },
    DigrT {
        char1: b'L',
        char2: b'.',
        result: 0x013f,
    },
    DigrT {
        char1: b'l',
        char2: b'.',
        result: 0x0140,
    },
    DigrT {
        char1: b'L',
        char2: b'/',
        result: 0x0141,
    },
    DigrT {
        char1: b'l',
        char2: b'/',
        result: 0x0142,
    },
    DigrT {
        char1: b'N',
        char2: b'\'',
        result: 0x0143,
    },
    DigrT {
        char1: b'n',
        char2: b'\'',
        result: 0x0144,
    },
    DigrT {
        char1: b'N',
        char2: b',',
        result: 0x0145,
    },
    DigrT {
        char1: b'n',
        char2: b',',
        result: 0x0146,
    },
    DigrT {
        char1: b'N',
        char2: b'<',
        result: 0x0147,
    },
    DigrT {
        char1: b'n',
        char2: b'<',
        result: 0x0148,
    },
    DigrT {
        char1: b'\'',
        char2: b'n',
        result: 0x0149,
    },
    DigrT {
        char1: b'N',
        char2: b'G',
        result: 0x014a,
    },
    DigrT {
        char1: b'n',
        char2: b'g',
        result: 0x014b,
    },
    DigrT {
        char1: b'O',
        char2: b'-',
        result: 0x014c,
    },
    DigrT {
        char1: b'o',
        char2: b'-',
        result: 0x014d,
    },
    DigrT {
        char1: b'O',
        char2: b'(',
        result: 0x014e,
    },
    DigrT {
        char1: b'o',
        char2: b'(',
        result: 0x014f,
    },
    DigrT {
        char1: b'O',
        char2: b'"',
        result: 0x0150,
    },
    DigrT {
        char1: b'o',
        char2: b'"',
        result: 0x0151,
    },
    DigrT {
        char1: b'O',
        char2: b'E',
        result: 0x0152,
    },
    DigrT {
        char1: b'o',
        char2: b'e',
        result: 0x0153,
    },
    DigrT {
        char1: b'R',
        char2: b'\'',
        result: 0x0154,
    },
    DigrT {
        char1: b'r',
        char2: b'\'',
        result: 0x0155,
    },
    DigrT {
        char1: b'R',
        char2: b',',
        result: 0x0156,
    },
    DigrT {
        char1: b'r',
        char2: b',',
        result: 0x0157,
    },
    DigrT {
        char1: b'R',
        char2: b'<',
        result: 0x0158,
    },
    DigrT {
        char1: b'r',
        char2: b'<',
        result: 0x0159,
    },
    DigrT {
        char1: b'S',
        char2: b'\'',
        result: 0x015a,
    },
    DigrT {
        char1: b's',
        char2: b'\'',
        result: 0x015b,
    },
    DigrT {
        char1: b'S',
        char2: b'>',
        result: 0x015c,
    },
    DigrT {
        char1: b's',
        char2: b'>',
        result: 0x015d,
    },
    DigrT {
        char1: b'S',
        char2: b',',
        result: 0x015e,
    },
    DigrT {
        char1: b's',
        char2: b',',
        result: 0x015f,
    },
    DigrT {
        char1: b'S',
        char2: b'<',
        result: 0x0160,
    },
    DigrT {
        char1: b's',
        char2: b'<',
        result: 0x0161,
    },
    DigrT {
        char1: b'T',
        char2: b',',
        result: 0x0162,
    },
    DigrT {
        char1: b't',
        char2: b',',
        result: 0x0163,
    },
    DigrT {
        char1: b'T',
        char2: b'<',
        result: 0x0164,
    },
    DigrT {
        char1: b't',
        char2: b'<',
        result: 0x0165,
    },
    DigrT {
        char1: b'T',
        char2: b'/',
        result: 0x0166,
    },
    DigrT {
        char1: b't',
        char2: b'/',
        result: 0x0167,
    },
    DigrT {
        char1: b'U',
        char2: b'?',
        result: 0x0168,
    },
    DigrT {
        char1: b'u',
        char2: b'?',
        result: 0x0169,
    },
    DigrT {
        char1: b'U',
        char2: b'-',
        result: 0x016a,
    },
    DigrT {
        char1: b'u',
        char2: b'-',
        result: 0x016b,
    },
    DigrT {
        char1: b'U',
        char2: b'(',
        result: 0x016c,
    },
    DigrT {
        char1: b'u',
        char2: b'(',
        result: 0x016d,
    },
    DigrT {
        char1: b'U',
        char2: b'0',
        result: 0x016e,
    },
    DigrT {
        char1: b'u',
        char2: b'0',
        result: 0x016f,
    },
    DigrT {
        char1: b'U',
        char2: b'"',
        result: 0x0170,
    },
    DigrT {
        char1: b'u',
        char2: b'"',
        result: 0x0171,
    },
    DigrT {
        char1: b'U',
        char2: b';',
        result: 0x0172,
    },
    DigrT {
        char1: b'u',
        char2: b';',
        result: 0x0173,
    },
    DigrT {
        char1: b'W',
        char2: b'>',
        result: 0x0174,
    },
    DigrT {
        char1: b'w',
        char2: b'>',
        result: 0x0175,
    },
    DigrT {
        char1: b'Y',
        char2: b'>',
        result: 0x0176,
    },
    DigrT {
        char1: b'y',
        char2: b'>',
        result: 0x0177,
    },
    DigrT {
        char1: b'Y',
        char2: b':',
        result: 0x0178,
    },
    DigrT {
        char1: b'Z',
        char2: b'\'',
        result: 0x0179,
    },
    DigrT {
        char1: b'z',
        char2: b'\'',
        result: 0x017a,
    },
    DigrT {
        char1: b'Z',
        char2: b'.',
        result: 0x017b,
    },
    DigrT {
        char1: b'z',
        char2: b'.',
        result: 0x017c,
    },
    DigrT {
        char1: b'Z',
        char2: b'<',
        result: 0x017d,
    },
    DigrT {
        char1: b'z',
        char2: b'<',
        result: 0x017e,
    },
    DigrT {
        char1: b'O',
        char2: b'9',
        result: 0x01a0,
    },
    DigrT {
        char1: b'o',
        char2: b'9',
        result: 0x01a1,
    },
    DigrT {
        char1: b'O',
        char2: b'I',
        result: 0x01a2,
    },
    DigrT {
        char1: b'o',
        char2: b'i',
        result: 0x01a3,
    },
    DigrT {
        char1: b'y',
        char2: b'r',
        result: 0x01a6,
    },
    DigrT {
        char1: b'U',
        char2: b'9',
        result: 0x01af,
    },
    DigrT {
        char1: b'u',
        char2: b'9',
        result: 0x01b0,
    },
    DigrT {
        char1: b'Z',
        char2: b'/',
        result: 0x01b5,
    },
    DigrT {
        char1: b'z',
        char2: b'/',
        result: 0x01b6,
    },
    DigrT {
        char1: b'E',
        char2: b'D',
        result: 0x01b7,
    },
    DigrT {
        char1: b'A',
        char2: b'<',
        result: 0x01cd,
    },
    DigrT {
        char1: b'a',
        char2: b'<',
        result: 0x01ce,
    },
    DigrT {
        char1: b'I',
        char2: b'<',
        result: 0x01cf,
    },
    DigrT {
        char1: b'i',
        char2: b'<',
        result: 0x01d0,
    },
    DigrT {
        char1: b'O',
        char2: b'<',
        result: 0x01d1,
    },
    DigrT {
        char1: b'o',
        char2: b'<',
        result: 0x01d2,
    },
    DigrT {
        char1: b'U',
        char2: b'<',
        result: 0x01d3,
    },
    DigrT {
        char1: b'u',
        char2: b'<',
        result: 0x01d4,
    },
    DigrT {
        char1: b'A',
        char2: b'1',
        result: 0x01de,
    },
    DigrT {
        char1: b'a',
        char2: b'1',
        result: 0x01df,
    },
    DigrT {
        char1: b'A',
        char2: b'7',
        result: 0x01e0,
    },
    DigrT {
        char1: b'a',
        char2: b'7',
        result: 0x01e1,
    },
    DigrT {
        char1: b'A',
        char2: b'3',
        result: 0x01e2,
    },
    DigrT {
        char1: b'a',
        char2: b'3',
        result: 0x01e3,
    },
    DigrT {
        char1: b'G',
        char2: b'/',
        result: 0x01e4,
    },
    DigrT {
        char1: b'g',
        char2: b'/',
        result: 0x01e5,
    },
    DigrT {
        char1: b'G',
        char2: b'<',
        result: 0x01e6,
    },
    DigrT {
        char1: b'g',
        char2: b'<',
        result: 0x01e7,
    },
    DigrT {
        char1: b'K',
        char2: b'<',
        result: 0x01e8,
    },
    DigrT {
        char1: b'k',
        char2: b'<',
        result: 0x01e9,
    },
    DigrT {
        char1: b'O',
        char2: b';',
        result: 0x01ea,
    },
    DigrT {
        char1: b'o',
        char2: b';',
        result: 0x01eb,
    },
    DigrT {
        char1: b'O',
        char2: b'1',
        result: 0x01ec,
    },
    DigrT {
        char1: b'o',
        char2: b'1',
        result: 0x01ed,
    },
    DigrT {
        char1: b'E',
        char2: b'Z',
        result: 0x01ee,
    },
    DigrT {
        char1: b'e',
        char2: b'z',
        result: 0x01ef,
    },
    DigrT {
        char1: b'j',
        char2: b'<',
        result: 0x01f0,
    },
    DigrT {
        char1: b'G',
        char2: b'\'',
        result: 0x01f4,
    },
    DigrT {
        char1: b'g',
        char2: b'\'',
        result: 0x01f5,
    },
    DigrT {
        char1: b';',
        char2: b'S',
        result: 0x02bf,
    },
    DigrT {
        char1: b'\'',
        char2: b'<',
        result: 0x02c7,
    },
    DigrT {
        char1: b'\'',
        char2: b'(',
        result: 0x02d8,
    },
    DigrT {
        char1: b'\'',
        char2: b'.',
        result: 0x02d9,
    },
    DigrT {
        char1: b'\'',
        char2: b'0',
        result: 0x02da,
    },
    DigrT {
        char1: b'\'',
        char2: b';',
        result: 0x02db,
    },
    DigrT {
        char1: b'\'',
        char2: b'"',
        result: 0x02dd,
    },
    DigrT {
        char1: b'A',
        char2: b'%',
        result: 0x0386,
    },
    DigrT {
        char1: b'E',
        char2: b'%',
        result: 0x0388,
    },
    DigrT {
        char1: b'Y',
        char2: b'%',
        result: 0x0389,
    },
    DigrT {
        char1: b'I',
        char2: b'%',
        result: 0x038a,
    },
    DigrT {
        char1: b'O',
        char2: b'%',
        result: 0x038c,
    },
    DigrT {
        char1: b'U',
        char2: b'%',
        result: 0x038e,
    },
    DigrT {
        char1: b'W',
        char2: b'%',
        result: 0x038f,
    },
    DigrT {
        char1: b'i',
        char2: b'3',
        result: 0x0390,
    },
    DigrT {
        char1: b'A',
        char2: b'*',
        result: 0x0391,
    },
    DigrT {
        char1: b'B',
        char2: b'*',
        result: 0x0392,
    },
    DigrT {
        char1: b'G',
        char2: b'*',
        result: 0x0393,
    },
    DigrT {
        char1: b'D',
        char2: b'*',
        result: 0x0394,
    },
    DigrT {
        char1: b'E',
        char2: b'*',
        result: 0x0395,
    },
    DigrT {
        char1: b'Z',
        char2: b'*',
        result: 0x0396,
    },
    DigrT {
        char1: b'Y',
        char2: b'*',
        result: 0x0397,
    },
    DigrT {
        char1: b'H',
        char2: b'*',
        result: 0x0398,
    },
    DigrT {
        char1: b'I',
        char2: b'*',
        result: 0x0399,
    },
    DigrT {
        char1: b'K',
        char2: b'*',
        result: 0x039a,
    },
    DigrT {
        char1: b'L',
        char2: b'*',
        result: 0x039b,
    },
    DigrT {
        char1: b'M',
        char2: b'*',
        result: 0x039c,
    },
    DigrT {
        char1: b'N',
        char2: b'*',
        result: 0x039d,
    },
    DigrT {
        char1: b'C',
        char2: b'*',
        result: 0x039e,
    },
    DigrT {
        char1: b'O',
        char2: b'*',
        result: 0x039f,
    },
    DigrT {
        char1: b'P',
        char2: b'*',
        result: 0x03a0,
    },
    DigrT {
        char1: b'R',
        char2: b'*',
        result: 0x03a1,
    },
    DigrT {
        char1: b'S',
        char2: b'*',
        result: 0x03a3,
    },
    DigrT {
        char1: b'T',
        char2: b'*',
        result: 0x03a4,
    },
    DigrT {
        char1: b'U',
        char2: b'*',
        result: 0x03a5,
    },
    DigrT {
        char1: b'F',
        char2: b'*',
        result: 0x03a6,
    },
    DigrT {
        char1: b'X',
        char2: b'*',
        result: 0x03a7,
    },
    DigrT {
        char1: b'Q',
        char2: b'*',
        result: 0x03a8,
    },
    DigrT {
        char1: b'W',
        char2: b'*',
        result: 0x03a9,
    },
    DigrT {
        char1: b'J',
        char2: b'*',
        result: 0x03aa,
    },
    DigrT {
        char1: b'V',
        char2: b'*',
        result: 0x03ab,
    },
    DigrT {
        char1: b'a',
        char2: b'%',
        result: 0x03ac,
    },
    DigrT {
        char1: b'e',
        char2: b'%',
        result: 0x03ad,
    },
    DigrT {
        char1: b'y',
        char2: b'%',
        result: 0x03ae,
    },
    DigrT {
        char1: b'i',
        char2: b'%',
        result: 0x03af,
    },
    DigrT {
        char1: b'u',
        char2: b'3',
        result: 0x03b0,
    },
    DigrT {
        char1: b'a',
        char2: b'*',
        result: 0x03b1,
    },
    DigrT {
        char1: b'b',
        char2: b'*',
        result: 0x03b2,
    },
    DigrT {
        char1: b'g',
        char2: b'*',
        result: 0x03b3,
    },
    DigrT {
        char1: b'd',
        char2: b'*',
        result: 0x03b4,
    },
    DigrT {
        char1: b'e',
        char2: b'*',
        result: 0x03b5,
    },
    DigrT {
        char1: b'z',
        char2: b'*',
        result: 0x03b6,
    },
    DigrT {
        char1: b'y',
        char2: b'*',
        result: 0x03b7,
    },
    DigrT {
        char1: b'h',
        char2: b'*',
        result: 0x03b8,
    },
    DigrT {
        char1: b'i',
        char2: b'*',
        result: 0x03b9,
    },
    DigrT {
        char1: b'k',
        char2: b'*',
        result: 0x03ba,
    },
    DigrT {
        char1: b'l',
        char2: b'*',
        result: 0x03bb,
    },
    DigrT {
        char1: b'm',
        char2: b'*',
        result: 0x03bc,
    },
    DigrT {
        char1: b'n',
        char2: b'*',
        result: 0x03bd,
    },
    DigrT {
        char1: b'c',
        char2: b'*',
        result: 0x03be,
    },
    DigrT {
        char1: b'o',
        char2: b'*',
        result: 0x03bf,
    },
    DigrT {
        char1: b'p',
        char2: b'*',
        result: 0x03c0,
    },
    DigrT {
        char1: b'r',
        char2: b'*',
        result: 0x03c1,
    },
    DigrT {
        char1: b'*',
        char2: b's',
        result: 0x03c2,
    },
    DigrT {
        char1: b's',
        char2: b'*',
        result: 0x03c3,
    },
    DigrT {
        char1: b't',
        char2: b'*',
        result: 0x03c4,
    },
    DigrT {
        char1: b'u',
        char2: b'*',
        result: 0x03c5,
    },
    DigrT {
        char1: b'f',
        char2: b'*',
        result: 0x03c6,
    },
    DigrT {
        char1: b'x',
        char2: b'*',
        result: 0x03c7,
    },
    DigrT {
        char1: b'q',
        char2: b'*',
        result: 0x03c8,
    },
    DigrT {
        char1: b'w',
        char2: b'*',
        result: 0x03c9,
    },
    DigrT {
        char1: b'j',
        char2: b'*',
        result: 0x03ca,
    },
    DigrT {
        char1: b'v',
        char2: b'*',
        result: 0x03cb,
    },
    DigrT {
        char1: b'o',
        char2: b'%',
        result: 0x03cc,
    },
    DigrT {
        char1: b'u',
        char2: b'%',
        result: 0x03cd,
    },
    DigrT {
        char1: b'w',
        char2: b'%',
        result: 0x03ce,
    },
    DigrT {
        char1: b'\'',
        char2: b'G',
        result: 0x03d8,
    },
    DigrT {
        char1: b',',
        char2: b'G',
        result: 0x03d9,
    },
    DigrT {
        char1: b'T',
        char2: b'3',
        result: 0x03da,
    },
    DigrT {
        char1: b't',
        char2: b'3',
        result: 0x03db,
    },
    DigrT {
        char1: b'M',
        char2: b'3',
        result: 0x03dc,
    },
    DigrT {
        char1: b'm',
        char2: b'3',
        result: 0x03dd,
    },
    DigrT {
        char1: b'K',
        char2: b'3',
        result: 0x03de,
    },
    DigrT {
        char1: b'k',
        char2: b'3',
        result: 0x03df,
    },
    DigrT {
        char1: b'P',
        char2: b'3',
        result: 0x03e0,
    },
    DigrT {
        char1: b'p',
        char2: b'3',
        result: 0x03e1,
    },
    DigrT {
        char1: b'\'',
        char2: b'%',
        result: 0x03f4,
    },
    DigrT {
        char1: b'j',
        char2: b'3',
        result: 0x03f5,
    },
    DigrT {
        char1: b'I',
        char2: b'O',
        result: 0x0401,
    },
    DigrT {
        char1: b'D',
        char2: b'%',
        result: 0x0402,
    },
    DigrT {
        char1: b'G',
        char2: b'%',
        result: 0x0403,
    },
    DigrT {
        char1: b'I',
        char2: b'E',
        result: 0x0404,
    },
    DigrT {
        char1: b'D',
        char2: b'S',
        result: 0x0405,
    },
    DigrT {
        char1: b'I',
        char2: b'I',
        result: 0x0406,
    },
    DigrT {
        char1: b'Y',
        char2: b'I',
        result: 0x0407,
    },
    DigrT {
        char1: b'J',
        char2: b'%',
        result: 0x0408,
    },
    DigrT {
        char1: b'L',
        char2: b'J',
        result: 0x0409,
    },
    DigrT {
        char1: b'N',
        char2: b'J',
        result: 0x040a,
    },
    DigrT {
        char1: b'T',
        char2: b's',
        result: 0x040b,
    },
    DigrT {
        char1: b'K',
        char2: b'J',
        result: 0x040c,
    },
    DigrT {
        char1: b'V',
        char2: b'%',
        result: 0x040e,
    },
    DigrT {
        char1: b'D',
        char2: b'Z',
        result: 0x040f,
    },
    DigrT {
        char1: b'A',
        char2: b'=',
        result: 0x0410,
    },
    DigrT {
        char1: b'B',
        char2: b'=',
        result: 0x0411,
    },
    DigrT {
        char1: b'V',
        char2: b'=',
        result: 0x0412,
    },
    DigrT {
        char1: b'G',
        char2: b'=',
        result: 0x0413,
    },
    DigrT {
        char1: b'D',
        char2: b'=',
        result: 0x0414,
    },
    DigrT {
        char1: b'E',
        char2: b'=',
        result: 0x0415,
    },
    DigrT {
        char1: b'Z',
        char2: b'%',
        result: 0x0416,
    },
    DigrT {
        char1: b'Z',
        char2: b'=',
        result: 0x0417,
    },
    DigrT {
        char1: b'I',
        char2: b'=',
        result: 0x0418,
    },
    DigrT {
        char1: b'J',
        char2: b'=',
        result: 0x0419,
    },
    DigrT {
        char1: b'K',
        char2: b'=',
        result: 0x041a,
    },
    DigrT {
        char1: b'L',
        char2: b'=',
        result: 0x041b,
    },
    DigrT {
        char1: b'M',
        char2: b'=',
        result: 0x041c,
    },
    DigrT {
        char1: b'N',
        char2: b'=',
        result: 0x041d,
    },
    DigrT {
        char1: b'O',
        char2: b'=',
        result: 0x041e,
    },
    DigrT {
        char1: b'P',
        char2: b'=',
        result: 0x041f,
    },
    DigrT {
        char1: b'R',
        char2: b'=',
        result: 0x0420,
    },
    DigrT {
        char1: b'S',
        char2: b'=',
        result: 0x0421,
    },
    DigrT {
        char1: b'T',
        char2: b'=',
        result: 0x0422,
    },
    DigrT {
        char1: b'U',
        char2: b'=',
        result: 0x0423,
    },
    DigrT {
        char1: b'F',
        char2: b'=',
        result: 0x0424,
    },
    DigrT {
        char1: b'H',
        char2: b'=',
        result: 0x0425,
    },
    DigrT {
        char1: b'C',
        char2: b'=',
        result: 0x0426,
    },
    DigrT {
        char1: b'C',
        char2: b'%',
        result: 0x0427,
    },
    DigrT {
        char1: b'S',
        char2: b'%',
        result: 0x0428,
    },
    DigrT {
        char1: b'S',
        char2: b'c',
        result: 0x0429,
    },
    DigrT {
        char1: b'=',
        char2: b'"',
        result: 0x042a,
    },
    DigrT {
        char1: b'Y',
        char2: b'=',
        result: 0x042b,
    },
    DigrT {
        char1: b'%',
        char2: b'"',
        result: 0x042c,
    },
    DigrT {
        char1: b'J',
        char2: b'E',
        result: 0x042d,
    },
    DigrT {
        char1: b'J',
        char2: b'U',
        result: 0x042e,
    },
    DigrT {
        char1: b'J',
        char2: b'A',
        result: 0x042f,
    },
    DigrT {
        char1: b'a',
        char2: b'=',
        result: 0x0430,
    },
    DigrT {
        char1: b'b',
        char2: b'=',
        result: 0x0431,
    },
    DigrT {
        char1: b'v',
        char2: b'=',
        result: 0x0432,
    },
    DigrT {
        char1: b'g',
        char2: b'=',
        result: 0x0433,
    },
    DigrT {
        char1: b'd',
        char2: b'=',
        result: 0x0434,
    },
    DigrT {
        char1: b'e',
        char2: b'=',
        result: 0x0435,
    },
    DigrT {
        char1: b'z',
        char2: b'%',
        result: 0x0436,
    },
    DigrT {
        char1: b'z',
        char2: b'=',
        result: 0x0437,
    },
    DigrT {
        char1: b'i',
        char2: b'=',
        result: 0x0438,
    },
    DigrT {
        char1: b'j',
        char2: b'=',
        result: 0x0439,
    },
    DigrT {
        char1: b'k',
        char2: b'=',
        result: 0x043a,
    },
    DigrT {
        char1: b'l',
        char2: b'=',
        result: 0x043b,
    },
    DigrT {
        char1: b'm',
        char2: b'=',
        result: 0x043c,
    },
    DigrT {
        char1: b'n',
        char2: b'=',
        result: 0x043d,
    },
    DigrT {
        char1: b'o',
        char2: b'=',
        result: 0x043e,
    },
    DigrT {
        char1: b'p',
        char2: b'=',
        result: 0x043f,
    },
    DigrT {
        char1: b'r',
        char2: b'=',
        result: 0x0440,
    },
    DigrT {
        char1: b's',
        char2: b'=',
        result: 0x0441,
    },
    DigrT {
        char1: b't',
        char2: b'=',
        result: 0x0442,
    },
    DigrT {
        char1: b'u',
        char2: b'=',
        result: 0x0443,
    },
    DigrT {
        char1: b'f',
        char2: b'=',
        result: 0x0444,
    },
    DigrT {
        char1: b'h',
        char2: b'=',
        result: 0x0445,
    },
    DigrT {
        char1: b'c',
        char2: b'=',
        result: 0x0446,
    },
    DigrT {
        char1: b'c',
        char2: b'%',
        result: 0x0447,
    },
    DigrT {
        char1: b's',
        char2: b'%',
        result: 0x0448,
    },
    DigrT {
        char1: b's',
        char2: b'c',
        result: 0x0449,
    },
    DigrT {
        char1: b'=',
        char2: b'\'',
        result: 0x044a,
    },
    DigrT {
        char1: b'y',
        char2: b'=',
        result: 0x044b,
    },
    DigrT {
        char1: b'%',
        char2: b'\'',
        result: 0x044c,
    },
    DigrT {
        char1: b'j',
        char2: b'e',
        result: 0x044d,
    },
    DigrT {
        char1: b'j',
        char2: b'u',
        result: 0x044e,
    },
    DigrT {
        char1: b'j',
        char2: b'a',
        result: 0x044f,
    },
    DigrT {
        char1: b'i',
        char2: b'o',
        result: 0x0451,
    },
    DigrT {
        char1: b'd',
        char2: b'%',
        result: 0x0452,
    },
    DigrT {
        char1: b'g',
        char2: b'%',
        result: 0x0453,
    },
    DigrT {
        char1: b'i',
        char2: b'e',
        result: 0x0454,
    },
    DigrT {
        char1: b'd',
        char2: b's',
        result: 0x0455,
    },
    DigrT {
        char1: b'i',
        char2: b'i',
        result: 0x0456,
    },
    DigrT {
        char1: b'y',
        char2: b'i',
        result: 0x0457,
    },
    DigrT {
        char1: b'j',
        char2: b'%',
        result: 0x0458,
    },
    DigrT {
        char1: b'l',
        char2: b'j',
        result: 0x0459,
    },
    DigrT {
        char1: b'n',
        char2: b'j',
        result: 0x045a,
    },
    DigrT {
        char1: b't',
        char2: b's',
        result: 0x045b,
    },
    DigrT {
        char1: b'k',
        char2: b'j',
        result: 0x045c,
    },
    DigrT {
        char1: b'v',
        char2: b'%',
        result: 0x045e,
    },
    DigrT {
        char1: b'd',
        char2: b'z',
        result: 0x045f,
    },
    DigrT {
        char1: b'Y',
        char2: b'3',
        result: 0x0462,
    },
    DigrT {
        char1: b'y',
        char2: b'3',
        result: 0x0463,
    },
    DigrT {
        char1: b'O',
        char2: b'3',
        result: 0x046a,
    },
    DigrT {
        char1: b'o',
        char2: b'3',
        result: 0x046b,
    },
    DigrT {
        char1: b'F',
        char2: b'3',
        result: 0x0472,
    },
    DigrT {
        char1: b'f',
        char2: b'3',
        result: 0x0473,
    },
    DigrT {
        char1: b'V',
        char2: b'3',
        result: 0x0474,
    },
    DigrT {
        char1: b'v',
        char2: b'3',
        result: 0x0475,
    },
    DigrT {
        char1: b'C',
        char2: b'3',
        result: 0x0480,
    },
    DigrT {
        char1: b'c',
        char2: b'3',
        result: 0x0481,
    },
    DigrT {
        char1: b'G',
        char2: b'3',
        result: 0x0490,
    },
    DigrT {
        char1: b'g',
        char2: b'3',
        result: 0x0491,
    },
    DigrT {
        char1: b'A',
        char2: b'+',
        result: 0x05d0,
    },
    DigrT {
        char1: b'B',
        char2: b'+',
        result: 0x05d1,
    },
    DigrT {
        char1: b'G',
        char2: b'+',
        result: 0x05d2,
    },
    DigrT {
        char1: b'D',
        char2: b'+',
        result: 0x05d3,
    },
    DigrT {
        char1: b'H',
        char2: b'+',
        result: 0x05d4,
    },
    DigrT {
        char1: b'W',
        char2: b'+',
        result: 0x05d5,
    },
    DigrT {
        char1: b'Z',
        char2: b'+',
        result: 0x05d6,
    },
    DigrT {
        char1: b'X',
        char2: b'+',
        result: 0x05d7,
    },
    DigrT {
        char1: b'T',
        char2: b'j',
        result: 0x05d8,
    },
    DigrT {
        char1: b'J',
        char2: b'+',
        result: 0x05d9,
    },
    DigrT {
        char1: b'K',
        char2: b'%',
        result: 0x05da,
    },
    DigrT {
        char1: b'K',
        char2: b'+',
        result: 0x05db,
    },
    DigrT {
        char1: b'L',
        char2: b'+',
        result: 0x05dc,
    },
    DigrT {
        char1: b'M',
        char2: b'%',
        result: 0x05dd,
    },
    DigrT {
        char1: b'M',
        char2: b'+',
        result: 0x05de,
    },
    DigrT {
        char1: b'N',
        char2: b'%',
        result: 0x05df,
    },
    DigrT {
        char1: b'N',
        char2: b'+',
        result: 0x05e0,
    },
    DigrT {
        char1: b'S',
        char2: b'+',
        result: 0x05e1,
    },
    DigrT {
        char1: b'E',
        char2: b'+',
        result: 0x05e2,
    },
    DigrT {
        char1: b'P',
        char2: b'%',
        result: 0x05e3,
    },
    DigrT {
        char1: b'P',
        char2: b'+',
        result: 0x05e4,
    },
    DigrT {
        char1: b'Z',
        char2: b'j',
        result: 0x05e5,
    },
    DigrT {
        char1: b'Z',
        char2: b'J',
        result: 0x05e6,
    },
    DigrT {
        char1: b'Q',
        char2: b'+',
        result: 0x05e7,
    },
    DigrT {
        char1: b'R',
        char2: b'+',
        result: 0x05e8,
    },
    DigrT {
        char1: b'S',
        char2: b'h',
        result: 0x05e9,
    },
    DigrT {
        char1: b'T',
        char2: b'+',
        result: 0x05ea,
    },
    DigrT {
        char1: b',',
        char2: b'+',
        result: 0x060c,
    },
    DigrT {
        char1: b';',
        char2: b'+',
        result: 0x061b,
    },
    DigrT {
        char1: b'?',
        char2: b'+',
        result: 0x061f,
    },
    DigrT {
        char1: b'H',
        char2: b'\'',
        result: 0x0621,
    },
    DigrT {
        char1: b'a',
        char2: b'M',
        result: 0x0622,
    },
    DigrT {
        char1: b'a',
        char2: b'H',
        result: 0x0623,
    },
    DigrT {
        char1: b'w',
        char2: b'H',
        result: 0x0624,
    },
    DigrT {
        char1: b'a',
        char2: b'h',
        result: 0x0625,
    },
    DigrT {
        char1: b'y',
        char2: b'H',
        result: 0x0626,
    },
    DigrT {
        char1: b'a',
        char2: b'+',
        result: 0x0627,
    },
    DigrT {
        char1: b'b',
        char2: b'+',
        result: 0x0628,
    },
    DigrT {
        char1: b't',
        char2: b'm',
        result: 0x0629,
    },
    DigrT {
        char1: b't',
        char2: b'+',
        result: 0x062a,
    },
    DigrT {
        char1: b't',
        char2: b'k',
        result: 0x062b,
    },
    DigrT {
        char1: b'g',
        char2: b'+',
        result: 0x062c,
    },
    DigrT {
        char1: b'h',
        char2: b'k',
        result: 0x062d,
    },
    DigrT {
        char1: b'x',
        char2: b'+',
        result: 0x062e,
    },
    DigrT {
        char1: b'd',
        char2: b'+',
        result: 0x062f,
    },
    DigrT {
        char1: b'd',
        char2: b'k',
        result: 0x0630,
    },
    DigrT {
        char1: b'r',
        char2: b'+',
        result: 0x0631,
    },
    DigrT {
        char1: b'z',
        char2: b'+',
        result: 0x0632,
    },
    DigrT {
        char1: b's',
        char2: b'+',
        result: 0x0633,
    },
    DigrT {
        char1: b's',
        char2: b'n',
        result: 0x0634,
    },
    DigrT {
        char1: b'c',
        char2: b'+',
        result: 0x0635,
    },
    DigrT {
        char1: b'd',
        char2: b'd',
        result: 0x0636,
    },
    DigrT {
        char1: b't',
        char2: b'j',
        result: 0x0637,
    },
    DigrT {
        char1: b'z',
        char2: b'H',
        result: 0x0638,
    },
    DigrT {
        char1: b'e',
        char2: b'+',
        result: 0x0639,
    },
    DigrT {
        char1: b'i',
        char2: b'+',
        result: 0x063a,
    },
    DigrT {
        char1: b'+',
        char2: b'+',
        result: 0x0640,
    },
    DigrT {
        char1: b'f',
        char2: b'+',
        result: 0x0641,
    },
    DigrT {
        char1: b'q',
        char2: b'+',
        result: 0x0642,
    },
    DigrT {
        char1: b'k',
        char2: b'+',
        result: 0x0643,
    },
    DigrT {
        char1: b'l',
        char2: b'+',
        result: 0x0644,
    },
    DigrT {
        char1: b'm',
        char2: b'+',
        result: 0x0645,
    },
    DigrT {
        char1: b'n',
        char2: b'+',
        result: 0x0646,
    },
    DigrT {
        char1: b'h',
        char2: b'+',
        result: 0x0647,
    },
    DigrT {
        char1: b'w',
        char2: b'+',
        result: 0x0648,
    },
    DigrT {
        char1: b'j',
        char2: b'+',
        result: 0x0649,
    },
    DigrT {
        char1: b'y',
        char2: b'+',
        result: 0x064a,
    },
    DigrT {
        char1: b':',
        char2: b'+',
        result: 0x064b,
    },
    DigrT {
        char1: b'"',
        char2: b'+',
        result: 0x064c,
    },
    DigrT {
        char1: b'=',
        char2: b'+',
        result: 0x064d,
    },
    DigrT {
        char1: b'/',
        char2: b'+',
        result: 0x064e,
    },
    DigrT {
        char1: b'\'',
        char2: b'+',
        result: 0x064f,
    },
    DigrT {
        char1: b'1',
        char2: b'+',
        result: 0x0650,
    },
    DigrT {
        char1: b'3',
        char2: b'+',
        result: 0x0651,
    },
    DigrT {
        char1: b'0',
        char2: b'+',
        result: 0x0652,
    },
    DigrT {
        char1: b'a',
        char2: b'S',
        result: 0x0670,
    },
    DigrT {
        char1: b'p',
        char2: b'+',
        result: 0x067e,
    },
    DigrT {
        char1: b'v',
        char2: b'+',
        result: 0x06a4,
    },
    DigrT {
        char1: b'g',
        char2: b'f',
        result: 0x06af,
    },
    DigrT {
        char1: b'0',
        char2: b'a',
        result: 0x06f0,
    },
    DigrT {
        char1: b'1',
        char2: b'a',
        result: 0x06f1,
    },
    DigrT {
        char1: b'2',
        char2: b'a',
        result: 0x06f2,
    },
    DigrT {
        char1: b'3',
        char2: b'a',
        result: 0x06f3,
    },
    DigrT {
        char1: b'4',
        char2: b'a',
        result: 0x06f4,
    },
    DigrT {
        char1: b'5',
        char2: b'a',
        result: 0x06f5,
    },
    DigrT {
        char1: b'6',
        char2: b'a',
        result: 0x06f6,
    },
    DigrT {
        char1: b'7',
        char2: b'a',
        result: 0x06f7,
    },
    DigrT {
        char1: b'8',
        char2: b'a',
        result: 0x06f8,
    },
    DigrT {
        char1: b'9',
        char2: b'a',
        result: 0x06f9,
    },
    DigrT {
        char1: b'B',
        char2: b'.',
        result: 0x1e02,
    },
    DigrT {
        char1: b'b',
        char2: b'.',
        result: 0x1e03,
    },
    DigrT {
        char1: b'B',
        char2: b'_',
        result: 0x1e06,
    },
    DigrT {
        char1: b'b',
        char2: b'_',
        result: 0x1e07,
    },
    DigrT {
        char1: b'D',
        char2: b'.',
        result: 0x1e0a,
    },
    DigrT {
        char1: b'd',
        char2: b'.',
        result: 0x1e0b,
    },
    DigrT {
        char1: b'D',
        char2: b'_',
        result: 0x1e0e,
    },
    DigrT {
        char1: b'd',
        char2: b'_',
        result: 0x1e0f,
    },
    DigrT {
        char1: b'D',
        char2: b',',
        result: 0x1e10,
    },
    DigrT {
        char1: b'd',
        char2: b',',
        result: 0x1e11,
    },
    DigrT {
        char1: b'F',
        char2: b'.',
        result: 0x1e1e,
    },
    DigrT {
        char1: b'f',
        char2: b'.',
        result: 0x1e1f,
    },
    DigrT {
        char1: b'G',
        char2: b'-',
        result: 0x1e20,
    },
    DigrT {
        char1: b'g',
        char2: b'-',
        result: 0x1e21,
    },
    DigrT {
        char1: b'H',
        char2: b'.',
        result: 0x1e22,
    },
    DigrT {
        char1: b'h',
        char2: b'.',
        result: 0x1e23,
    },
    DigrT {
        char1: b'H',
        char2: b':',
        result: 0x1e26,
    },
    DigrT {
        char1: b'h',
        char2: b':',
        result: 0x1e27,
    },
    DigrT {
        char1: b'H',
        char2: b',',
        result: 0x1e28,
    },
    DigrT {
        char1: b'h',
        char2: b',',
        result: 0x1e29,
    },
    DigrT {
        char1: b'K',
        char2: b'\'',
        result: 0x1e30,
    },
    DigrT {
        char1: b'k',
        char2: b'\'',
        result: 0x1e31,
    },
    DigrT {
        char1: b'K',
        char2: b'_',
        result: 0x1e34,
    },
    DigrT {
        char1: b'k',
        char2: b'_',
        result: 0x1e35,
    },
    DigrT {
        char1: b'L',
        char2: b'_',
        result: 0x1e3a,
    },
    DigrT {
        char1: b'l',
        char2: b'_',
        result: 0x1e3b,
    },
    DigrT {
        char1: b'M',
        char2: b'\'',
        result: 0x1e3e,
    },
    DigrT {
        char1: b'm',
        char2: b'\'',
        result: 0x1e3f,
    },
    DigrT {
        char1: b'M',
        char2: b'.',
        result: 0x1e40,
    },
    DigrT {
        char1: b'm',
        char2: b'.',
        result: 0x1e41,
    },
    DigrT {
        char1: b'N',
        char2: b'.',
        result: 0x1e44,
    },
    DigrT {
        char1: b'n',
        char2: b'.',
        result: 0x1e45,
    },
    DigrT {
        char1: b'N',
        char2: b'_',
        result: 0x1e48,
    },
    DigrT {
        char1: b'n',
        char2: b'_',
        result: 0x1e49,
    },
    DigrT {
        char1: b'P',
        char2: b'\'',
        result: 0x1e54,
    },
    DigrT {
        char1: b'p',
        char2: b'\'',
        result: 0x1e55,
    },
    DigrT {
        char1: b'P',
        char2: b'.',
        result: 0x1e56,
    },
    DigrT {
        char1: b'p',
        char2: b'.',
        result: 0x1e57,
    },
    DigrT {
        char1: b'R',
        char2: b'.',
        result: 0x1e58,
    },
    DigrT {
        char1: b'r',
        char2: b'.',
        result: 0x1e59,
    },
    DigrT {
        char1: b'R',
        char2: b'_',
        result: 0x1e5e,
    },
    DigrT {
        char1: b'r',
        char2: b'_',
        result: 0x1e5f,
    },
    DigrT {
        char1: b'S',
        char2: b'.',
        result: 0x1e60,
    },
    DigrT {
        char1: b's',
        char2: b'.',
        result: 0x1e61,
    },
    DigrT {
        char1: b'T',
        char2: b'.',
        result: 0x1e6a,
    },
    DigrT {
        char1: b't',
        char2: b'.',
        result: 0x1e6b,
    },
    DigrT {
        char1: b'T',
        char2: b'_',
        result: 0x1e6e,
    },
    DigrT {
        char1: b't',
        char2: b'_',
        result: 0x1e6f,
    },
    DigrT {
        char1: b'V',
        char2: b'?',
        result: 0x1e7c,
    },
    DigrT {
        char1: b'v',
        char2: b'?',
        result: 0x1e7d,
    },
    DigrT {
        char1: b'W',
        char2: b'!',
        result: 0x1e80,
    },
    DigrT {
        char1: b'W',
        char2: b'`',
        result: 0x1e80,
    },
    DigrT {
        char1: b'w',
        char2: b'!',
        result: 0x1e81,
    },
    DigrT {
        char1: b'w',
        char2: b'`',
        result: 0x1e81,
    },
    DigrT {
        char1: b'W',
        char2: b'\'',
        result: 0x1e82,
    },
    DigrT {
        char1: b'w',
        char2: b'\'',
        result: 0x1e83,
    },
    DigrT {
        char1: b'W',
        char2: b':',
        result: 0x1e84,
    },
    DigrT {
        char1: b'w',
        char2: b':',
        result: 0x1e85,
    },
    DigrT {
        char1: b'W',
        char2: b'.',
        result: 0x1e86,
    },
    DigrT {
        char1: b'w',
        char2: b'.',
        result: 0x1e87,
    },
    DigrT {
        char1: b'X',
        char2: b'.',
        result: 0x1e8a,
    },
    DigrT {
        char1: b'x',
        char2: b'.',
        result: 0x1e8b,
    },
    DigrT {
        char1: b'X',
        char2: b':',
        result: 0x1e8c,
    },
    DigrT {
        char1: b'x',
        char2: b':',
        result: 0x1e8d,
    },
    DigrT {
        char1: b'Y',
        char2: b'.',
        result: 0x1e8e,
    },
    DigrT {
        char1: b'y',
        char2: b'.',
        result: 0x1e8f,
    },
    DigrT {
        char1: b'Z',
        char2: b'>',
        result: 0x1e90,
    },
    DigrT {
        char1: b'z',
        char2: b'>',
        result: 0x1e91,
    },
    DigrT {
        char1: b'Z',
        char2: b'_',
        result: 0x1e94,
    },
    DigrT {
        char1: b'z',
        char2: b'_',
        result: 0x1e95,
    },
    DigrT {
        char1: b'h',
        char2: b'_',
        result: 0x1e96,
    },
    DigrT {
        char1: b't',
        char2: b':',
        result: 0x1e97,
    },
    DigrT {
        char1: b'w',
        char2: b'0',
        result: 0x1e98,
    },
    DigrT {
        char1: b'y',
        char2: b'0',
        result: 0x1e99,
    },
    DigrT {
        char1: b'A',
        char2: b'2',
        result: 0x1ea2,
    },
    DigrT {
        char1: b'a',
        char2: b'2',
        result: 0x1ea3,
    },
    DigrT {
        char1: b'E',
        char2: b'2',
        result: 0x1eba,
    },
    DigrT {
        char1: b'e',
        char2: b'2',
        result: 0x1ebb,
    },
    DigrT {
        char1: b'E',
        char2: b'?',
        result: 0x1ebc,
    },
    DigrT {
        char1: b'e',
        char2: b'?',
        result: 0x1ebd,
    },
    DigrT {
        char1: b'I',
        char2: b'2',
        result: 0x1ec8,
    },
    DigrT {
        char1: b'i',
        char2: b'2',
        result: 0x1ec9,
    },
    DigrT {
        char1: b'O',
        char2: b'2',
        result: 0x1ece,
    },
    DigrT {
        char1: b'o',
        char2: b'2',
        result: 0x1ecf,
    },
    DigrT {
        char1: b'U',
        char2: b'2',
        result: 0x1ee6,
    },
    DigrT {
        char1: b'u',
        char2: b'2',
        result: 0x1ee7,
    },
    DigrT {
        char1: b'Y',
        char2: b'!',
        result: 0x1ef2,
    },
    DigrT {
        char1: b'Y',
        char2: b'`',
        result: 0x1ef2,
    },
    DigrT {
        char1: b'y',
        char2: b'!',
        result: 0x1ef3,
    },
    DigrT {
        char1: b'y',
        char2: b'`',
        result: 0x1ef3,
    },
    DigrT {
        char1: b'Y',
        char2: b'2',
        result: 0x1ef6,
    },
    DigrT {
        char1: b'y',
        char2: b'2',
        result: 0x1ef7,
    },
    DigrT {
        char1: b'Y',
        char2: b'?',
        result: 0x1ef8,
    },
    DigrT {
        char1: b'y',
        char2: b'?',
        result: 0x1ef9,
    },
    DigrT {
        char1: b';',
        char2: b'\'',
        result: 0x1f00,
    },
    DigrT {
        char1: b',',
        char2: b'\'',
        result: 0x1f01,
    },
    DigrT {
        char1: b';',
        char2: b'!',
        result: 0x1f02,
    },
    DigrT {
        char1: b',',
        char2: b'!',
        result: 0x1f03,
    },
    DigrT {
        char1: b'?',
        char2: b';',
        result: 0x1f04,
    },
    DigrT {
        char1: b'?',
        char2: b',',
        result: 0x1f05,
    },
    DigrT {
        char1: b'!',
        char2: b':',
        result: 0x1f06,
    },
    DigrT {
        char1: b'?',
        char2: b':',
        result: 0x1f07,
    },
    DigrT {
        char1: b'1',
        char2: b'N',
        result: 0x2002,
    },
    DigrT {
        char1: b'1',
        char2: b'M',
        result: 0x2003,
    },
    DigrT {
        char1: b'3',
        char2: b'M',
        result: 0x2004,
    },
    DigrT {
        char1: b'4',
        char2: b'M',
        result: 0x2005,
    },
    DigrT {
        char1: b'6',
        char2: b'M',
        result: 0x2006,
    },
    DigrT {
        char1: b'1',
        char2: b'T',
        result: 0x2009,
    },
    DigrT {
        char1: b'1',
        char2: b'H',
        result: 0x200a,
    },
    DigrT {
        char1: b'-',
        char2: b'1',
        result: 0x2010,
    },
    DigrT {
        char1: b'-',
        char2: b'N',
        result: 0x2013,
    },
    DigrT {
        char1: b'-',
        char2: b'M',
        result: 0x2014,
    },
    DigrT {
        char1: b'-',
        char2: b'3',
        result: 0x2015,
    },
    DigrT {
        char1: b'!',
        char2: b'2',
        result: 0x2016,
    },
    DigrT {
        char1: b'=',
        char2: b'2',
        result: 0x2017,
    },
    DigrT {
        char1: b'\'',
        char2: b'6',
        result: 0x2018,
    },
    DigrT {
        char1: b'\'',
        char2: b'9',
        result: 0x2019,
    },
    DigrT {
        char1: b'.',
        char2: b'9',
        result: 0x201a,
    },
    DigrT {
        char1: b'9',
        char2: b'\'',
        result: 0x201b,
    },
    DigrT {
        char1: b'"',
        char2: b'6',
        result: 0x201c,
    },
    DigrT {
        char1: b'"',
        char2: b'9',
        result: 0x201d,
    },
    DigrT {
        char1: b':',
        char2: b'9',
        result: 0x201e,
    },
    DigrT {
        char1: b'9',
        char2: b'"',
        result: 0x201f,
    },
    DigrT {
        char1: b'/',
        char2: b'-',
        result: 0x2020,
    },
    DigrT {
        char1: b'/',
        char2: b'=',
        result: 0x2021,
    },
    DigrT {
        char1: b'o',
        char2: b'o',
        result: 0x2022,
    },
    DigrT {
        char1: b'.',
        char2: b'.',
        result: 0x2025,
    },
    DigrT {
        char1: b',',
        char2: b'.',
        result: 0x2026,
    },
    DigrT {
        char1: b'%',
        char2: b'0',
        result: 0x2030,
    },
    DigrT {
        char1: b'1',
        char2: b'\'',
        result: 0x2032,
    },
    DigrT {
        char1: b'2',
        char2: b'\'',
        result: 0x2033,
    },
    DigrT {
        char1: b'3',
        char2: b'\'',
        result: 0x2034,
    },
    DigrT {
        char1: b'4',
        char2: b'\'',
        result: 0x2057,
    },
    DigrT {
        char1: b'1',
        char2: b'"',
        result: 0x2035,
    },
    DigrT {
        char1: b'2',
        char2: b'"',
        result: 0x2036,
    },
    DigrT {
        char1: b'3',
        char2: b'"',
        result: 0x2037,
    },
    DigrT {
        char1: b'C',
        char2: b'a',
        result: 0x2038,
    },
    DigrT {
        char1: b'<',
        char2: b'1',
        result: 0x2039,
    },
    DigrT {
        char1: b'>',
        char2: b'1',
        result: 0x203a,
    },
    DigrT {
        char1: b':',
        char2: b'X',
        result: 0x203b,
    },
    DigrT {
        char1: b'\'',
        char2: b'-',
        result: 0x203e,
    },
    DigrT {
        char1: b'/',
        char2: b'f',
        result: 0x2044,
    },
    DigrT {
        char1: b'0',
        char2: b'S',
        result: 0x2070,
    },
    DigrT {
        char1: b'4',
        char2: b'S',
        result: 0x2074,
    },
    DigrT {
        char1: b'5',
        char2: b'S',
        result: 0x2075,
    },
    DigrT {
        char1: b'6',
        char2: b'S',
        result: 0x2076,
    },
    DigrT {
        char1: b'7',
        char2: b'S',
        result: 0x2077,
    },
    DigrT {
        char1: b'8',
        char2: b'S',
        result: 0x2078,
    },
    DigrT {
        char1: b'9',
        char2: b'S',
        result: 0x2079,
    },
    DigrT {
        char1: b'+',
        char2: b'S',
        result: 0x207a,
    },
    DigrT {
        char1: b'-',
        char2: b'S',
        result: 0x207b,
    },
    DigrT {
        char1: b'=',
        char2: b'S',
        result: 0x207c,
    },
    DigrT {
        char1: b'(',
        char2: b'S',
        result: 0x207d,
    },
    DigrT {
        char1: b')',
        char2: b'S',
        result: 0x207e,
    },
    DigrT {
        char1: b'n',
        char2: b'S',
        result: 0x207f,
    },
    DigrT {
        char1: b'0',
        char2: b's',
        result: 0x2080,
    },
    DigrT {
        char1: b'1',
        char2: b's',
        result: 0x2081,
    },
    DigrT {
        char1: b'2',
        char2: b's',
        result: 0x2082,
    },
    DigrT {
        char1: b'3',
        char2: b's',
        result: 0x2083,
    },
    DigrT {
        char1: b'4',
        char2: b's',
        result: 0x2084,
    },
    DigrT {
        char1: b'5',
        char2: b's',
        result: 0x2085,
    },
    DigrT {
        char1: b'6',
        char2: b's',
        result: 0x2086,
    },
    DigrT {
        char1: b'7',
        char2: b's',
        result: 0x2087,
    },
    DigrT {
        char1: b'8',
        char2: b's',
        result: 0x2088,
    },
    DigrT {
        char1: b'9',
        char2: b's',
        result: 0x2089,
    },
    DigrT {
        char1: b'+',
        char2: b's',
        result: 0x208a,
    },
    DigrT {
        char1: b'-',
        char2: b's',
        result: 0x208b,
    },
    DigrT {
        char1: b'=',
        char2: b's',
        result: 0x208c,
    },
    DigrT {
        char1: b'(',
        char2: b's',
        result: 0x208d,
    },
    DigrT {
        char1: b')',
        char2: b's',
        result: 0x208e,
    },
    DigrT {
        char1: b'L',
        char2: b'i',
        result: 0x20a4,
    },
    DigrT {
        char1: b'P',
        char2: b't',
        result: 0x20a7,
    },
    DigrT {
        char1: b'W',
        char2: b'=',
        result: 0x20a9,
    },
    DigrT {
        char1: b'=',
        char2: b'e',
        result: 0x20ac,
    },
    DigrT {
        char1: b'E',
        char2: b'u',
        result: 0x20ac,
    },
    DigrT {
        char1: b'=',
        char2: b'R',
        result: 0x20bd,
    },
    DigrT {
        char1: b'=',
        char2: b'P',
        result: 0x20bd,
    },
    DigrT {
        char1: b'o',
        char2: b'C',
        result: 0x2103,
    },
    DigrT {
        char1: b'c',
        char2: b'o',
        result: 0x2105,
    },
    DigrT {
        char1: b'o',
        char2: b'F',
        result: 0x2109,
    },
    DigrT {
        char1: b'N',
        char2: b'0',
        result: 0x2116,
    },
    DigrT {
        char1: b'P',
        char2: b'O',
        result: 0x2117,
    },
    DigrT {
        char1: b'R',
        char2: b'x',
        result: 0x211e,
    },
    DigrT {
        char1: b'S',
        char2: b'M',
        result: 0x2120,
    },
    DigrT {
        char1: b'T',
        char2: b'M',
        result: 0x2122,
    },
    DigrT {
        char1: b'O',
        char2: b'm',
        result: 0x2126,
    },
    DigrT {
        char1: b'A',
        char2: b'O',
        result: 0x212b,
    },
    DigrT {
        char1: b'1',
        char2: b'3',
        result: 0x2153,
    },
    DigrT {
        char1: b'2',
        char2: b'3',
        result: 0x2154,
    },
    DigrT {
        char1: b'1',
        char2: b'5',
        result: 0x2155,
    },
    DigrT {
        char1: b'2',
        char2: b'5',
        result: 0x2156,
    },
    DigrT {
        char1: b'3',
        char2: b'5',
        result: 0x2157,
    },
    DigrT {
        char1: b'4',
        char2: b'5',
        result: 0x2158,
    },
    DigrT {
        char1: b'1',
        char2: b'6',
        result: 0x2159,
    },
    DigrT {
        char1: b'5',
        char2: b'6',
        result: 0x215a,
    },
    DigrT {
        char1: b'1',
        char2: b'8',
        result: 0x215b,
    },
    DigrT {
        char1: b'3',
        char2: b'8',
        result: 0x215c,
    },
    DigrT {
        char1: b'5',
        char2: b'8',
        result: 0x215d,
    },
    DigrT {
        char1: b'7',
        char2: b'8',
        result: 0x215e,
    },
    DigrT {
        char1: b'1',
        char2: b'R',
        result: 0x2160,
    },
    DigrT {
        char1: b'2',
        char2: b'R',
        result: 0x2161,
    },
    DigrT {
        char1: b'3',
        char2: b'R',
        result: 0x2162,
    },
    DigrT {
        char1: b'4',
        char2: b'R',
        result: 0x2163,
    },
    DigrT {
        char1: b'5',
        char2: b'R',
        result: 0x2164,
    },
    DigrT {
        char1: b'6',
        char2: b'R',
        result: 0x2165,
    },
    DigrT {
        char1: b'7',
        char2: b'R',
        result: 0x2166,
    },
    DigrT {
        char1: b'8',
        char2: b'R',
        result: 0x2167,
    },
    DigrT {
        char1: b'9',
        char2: b'R',
        result: 0x2168,
    },
    DigrT {
        char1: b'a',
        char2: b'R',
        result: 0x2169,
    },
    DigrT {
        char1: b'b',
        char2: b'R',
        result: 0x216a,
    },
    DigrT {
        char1: b'c',
        char2: b'R',
        result: 0x216b,
    },
    DigrT {
        char1: b'1',
        char2: b'r',
        result: 0x2170,
    },
    DigrT {
        char1: b'2',
        char2: b'r',
        result: 0x2171,
    },
    DigrT {
        char1: b'3',
        char2: b'r',
        result: 0x2172,
    },
    DigrT {
        char1: b'4',
        char2: b'r',
        result: 0x2173,
    },
    DigrT {
        char1: b'5',
        char2: b'r',
        result: 0x2174,
    },
    DigrT {
        char1: b'6',
        char2: b'r',
        result: 0x2175,
    },
    DigrT {
        char1: b'7',
        char2: b'r',
        result: 0x2176,
    },
    DigrT {
        char1: b'8',
        char2: b'r',
        result: 0x2177,
    },
    DigrT {
        char1: b'9',
        char2: b'r',
        result: 0x2178,
    },
    DigrT {
        char1: b'a',
        char2: b'r',
        result: 0x2179,
    },
    DigrT {
        char1: b'b',
        char2: b'r',
        result: 0x217a,
    },
    DigrT {
        char1: b'c',
        char2: b'r',
        result: 0x217b,
    },
    DigrT {
        char1: b'<',
        char2: b'-',
        result: 0x2190,
    },
    DigrT {
        char1: b'-',
        char2: b'!',
        result: 0x2191,
    },
    DigrT {
        char1: b'-',
        char2: b'>',
        result: 0x2192,
    },
    DigrT {
        char1: b'-',
        char2: b'v',
        result: 0x2193,
    },
    DigrT {
        char1: b'<',
        char2: b'>',
        result: 0x2194,
    },
    DigrT {
        char1: b'U',
        char2: b'D',
        result: 0x2195,
    },
    DigrT {
        char1: b'<',
        char2: b'=',
        result: 0x21d0,
    },
    DigrT {
        char1: b'=',
        char2: b'>',
        result: 0x21d2,
    },
    DigrT {
        char1: b'=',
        char2: b'=',
        result: 0x21d4,
    },
    DigrT {
        char1: b'F',
        char2: b'A',
        result: 0x2200,
    },
    DigrT {
        char1: b'd',
        char2: b'P',
        result: 0x2202,
    },
    DigrT {
        char1: b'T',
        char2: b'E',
        result: 0x2203,
    },
    DigrT {
        char1: b'/',
        char2: b'0',
        result: 0x2205,
    },
    DigrT {
        char1: b'D',
        char2: b'E',
        result: 0x2206,
    },
    DigrT {
        char1: b'N',
        char2: b'B',
        result: 0x2207,
    },
    DigrT {
        char1: b'(',
        char2: b'-',
        result: 0x2208,
    },
    DigrT {
        char1: b'-',
        char2: b')',
        result: 0x220b,
    },
    DigrT {
        char1: b'*',
        char2: b'P',
        result: 0x220f,
    },
    DigrT {
        char1: b'+',
        char2: b'Z',
        result: 0x2211,
    },
    DigrT {
        char1: b'-',
        char2: b'2',
        result: 0x2212,
    },
    DigrT {
        char1: b'-',
        char2: b'+',
        result: 0x2213,
    },
    DigrT {
        char1: b'*',
        char2: b'-',
        result: 0x2217,
    },
    DigrT {
        char1: b'O',
        char2: b'b',
        result: 0x2218,
    },
    DigrT {
        char1: b'S',
        char2: b'b',
        result: 0x2219,
    },
    DigrT {
        char1: b'R',
        char2: b'T',
        result: 0x221a,
    },
    DigrT {
        char1: b'0',
        char2: b'(',
        result: 0x221d,
    },
    DigrT {
        char1: b'0',
        char2: b'0',
        result: 0x221e,
    },
    DigrT {
        char1: b'-',
        char2: b'L',
        result: 0x221f,
    },
    DigrT {
        char1: b'-',
        char2: b'V',
        result: 0x2220,
    },
    DigrT {
        char1: b'P',
        char2: b'P',
        result: 0x2225,
    },
    DigrT {
        char1: b'A',
        char2: b'N',
        result: 0x2227,
    },
    DigrT {
        char1: b'O',
        char2: b'R',
        result: 0x2228,
    },
    DigrT {
        char1: b'(',
        char2: b'U',
        result: 0x2229,
    },
    DigrT {
        char1: b')',
        char2: b'U',
        result: 0x222a,
    },
    DigrT {
        char1: b'I',
        char2: b'n',
        result: 0x222b,
    },
    DigrT {
        char1: b'D',
        char2: b'I',
        result: 0x222c,
    },
    DigrT {
        char1: b'I',
        char2: b'o',
        result: 0x222e,
    },
    DigrT {
        char1: b'.',
        char2: b':',
        result: 0x2234,
    },
    DigrT {
        char1: b':',
        char2: b'.',
        result: 0x2235,
    },
    DigrT {
        char1: b':',
        char2: b'R',
        result: 0x2236,
    },
    DigrT {
        char1: b':',
        char2: b':',
        result: 0x2237,
    },
    DigrT {
        char1: b'?',
        char2: b'1',
        result: 0x223c,
    },
    DigrT {
        char1: b'C',
        char2: b'G',
        result: 0x223e,
    },
    DigrT {
        char1: b'?',
        char2: b'-',
        result: 0x2243,
    },
    DigrT {
        char1: b'?',
        char2: b'=',
        result: 0x2245,
    },
    DigrT {
        char1: b'?',
        char2: b'2',
        result: 0x2248,
    },
    DigrT {
        char1: b'=',
        char2: b'?',
        result: 0x224c,
    },
    DigrT {
        char1: b'.',
        char2: b'=',
        result: 0x2250,
    },
    DigrT {
        char1: b'H',
        char2: b'I',
        result: 0x2253,
    },
    DigrT {
        char1: b'!',
        char2: b'=',
        result: 0x2260,
    },
    DigrT {
        char1: b'=',
        char2: b'3',
        result: 0x2261,
    },
    DigrT {
        char1: b'=',
        char2: b'<',
        result: 0x2264,
    },
    DigrT {
        char1: b'>',
        char2: b'=',
        result: 0x2265,
    },
    DigrT {
        char1: b'<',
        char2: b'*',
        result: 0x226a,
    },
    DigrT {
        char1: b'*',
        char2: b'>',
        result: 0x226b,
    },
    DigrT {
        char1: b'!',
        char2: b'<',
        result: 0x226e,
    },
    DigrT {
        char1: b'!',
        char2: b'>',
        result: 0x226f,
    },
    DigrT {
        char1: b'(',
        char2: b'C',
        result: 0x2282,
    },
    DigrT {
        char1: b')',
        char2: b'C',
        result: 0x2283,
    },
    DigrT {
        char1: b'(',
        char2: b'_',
        result: 0x2286,
    },
    DigrT {
        char1: b')',
        char2: b'_',
        result: 0x2287,
    },
    DigrT {
        char1: b'0',
        char2: b'.',
        result: 0x2299,
    },
    DigrT {
        char1: b'0',
        char2: b'2',
        result: 0x229a,
    },
    DigrT {
        char1: b'-',
        char2: b'T',
        result: 0x22a5,
    },
    DigrT {
        char1: b'.',
        char2: b'P',
        result: 0x22c5,
    },
    DigrT {
        char1: b':',
        char2: b'3',
        result: 0x22ee,
    },
    DigrT {
        char1: b'.',
        char2: b'3',
        result: 0x22ef,
    },
    DigrT {
        char1: b'E',
        char2: b'h',
        result: 0x2302,
    },
    DigrT {
        char1: b'<',
        char2: b'7',
        result: 0x2308,
    },
    DigrT {
        char1: b'>',
        char2: b'7',
        result: 0x2309,
    },
    DigrT {
        char1: b'7',
        char2: b'<',
        result: 0x230a,
    },
    DigrT {
        char1: b'7',
        char2: b'>',
        result: 0x230b,
    },
    DigrT {
        char1: b'N',
        char2: b'I',
        result: 0x2310,
    },
    DigrT {
        char1: b'(',
        char2: b'A',
        result: 0x2312,
    },
    DigrT {
        char1: b'T',
        char2: b'R',
        result: 0x2315,
    },
    DigrT {
        char1: b'I',
        char2: b'u',
        result: 0x2320,
    },
    DigrT {
        char1: b'I',
        char2: b'l',
        result: 0x2321,
    },
    DigrT {
        char1: b'<',
        char2: b'[',
        result: 0x27e8,
    },
    DigrT {
        char1: b']',
        char2: b'>',
        result: 0x27e9,
    },
    DigrT {
        char1: b'V',
        char2: b's',
        result: 0x2423,
    },
    DigrT {
        char1: b'1',
        char2: b'h',
        result: 0x2440,
    },
    DigrT {
        char1: b'3',
        char2: b'h',
        result: 0x2441,
    },
    DigrT {
        char1: b'2',
        char2: b'h',
        result: 0x2442,
    },
    DigrT {
        char1: b'4',
        char2: b'h',
        result: 0x2443,
    },
    DigrT {
        char1: b'1',
        char2: b'j',
        result: 0x2446,
    },
    DigrT {
        char1: b'2',
        char2: b'j',
        result: 0x2447,
    },
    DigrT {
        char1: b'3',
        char2: b'j',
        result: 0x2448,
    },
    DigrT {
        char1: b'4',
        char2: b'j',
        result: 0x2449,
    },
    DigrT {
        char1: b'1',
        char2: b'.',
        result: 0x2488,
    },
    DigrT {
        char1: b'2',
        char2: b'.',
        result: 0x2489,
    },
    DigrT {
        char1: b'3',
        char2: b'.',
        result: 0x248a,
    },
    DigrT {
        char1: b'4',
        char2: b'.',
        result: 0x248b,
    },
    DigrT {
        char1: b'5',
        char2: b'.',
        result: 0x248c,
    },
    DigrT {
        char1: b'6',
        char2: b'.',
        result: 0x248d,
    },
    DigrT {
        char1: b'7',
        char2: b'.',
        result: 0x248e,
    },
    DigrT {
        char1: b'8',
        char2: b'.',
        result: 0x248f,
    },
    DigrT {
        char1: b'9',
        char2: b'.',
        result: 0x2490,
    },
    DigrT {
        char1: b'h',
        char2: b'h',
        result: 0x2500,
    },
    DigrT {
        char1: b'H',
        char2: b'H',
        result: 0x2501,
    },
    DigrT {
        char1: b'v',
        char2: b'v',
        result: 0x2502,
    },
    DigrT {
        char1: b'V',
        char2: b'V',
        result: 0x2503,
    },
    DigrT {
        char1: b'3',
        char2: b'-',
        result: 0x2504,
    },
    DigrT {
        char1: b'3',
        char2: b'_',
        result: 0x2505,
    },
    DigrT {
        char1: b'3',
        char2: b'!',
        result: 0x2506,
    },
    DigrT {
        char1: b'3',
        char2: b'/',
        result: 0x2507,
    },
    DigrT {
        char1: b'4',
        char2: b'-',
        result: 0x2508,
    },
    DigrT {
        char1: b'4',
        char2: b'_',
        result: 0x2509,
    },
    DigrT {
        char1: b'4',
        char2: b'!',
        result: 0x250a,
    },
    DigrT {
        char1: b'4',
        char2: b'/',
        result: 0x250b,
    },
    DigrT {
        char1: b'd',
        char2: b'r',
        result: 0x250c,
    },
    DigrT {
        char1: b'd',
        char2: b'R',
        result: 0x250d,
    },
    DigrT {
        char1: b'D',
        char2: b'r',
        result: 0x250e,
    },
    DigrT {
        char1: b'D',
        char2: b'R',
        result: 0x250f,
    },
    DigrT {
        char1: b'd',
        char2: b'l',
        result: 0x2510,
    },
    DigrT {
        char1: b'd',
        char2: b'L',
        result: 0x2511,
    },
    DigrT {
        char1: b'D',
        char2: b'l',
        result: 0x2512,
    },
    DigrT {
        char1: b'L',
        char2: b'D',
        result: 0x2513,
    },
    DigrT {
        char1: b'u',
        char2: b'r',
        result: 0x2514,
    },
    DigrT {
        char1: b'u',
        char2: b'R',
        result: 0x2515,
    },
    DigrT {
        char1: b'U',
        char2: b'r',
        result: 0x2516,
    },
    DigrT {
        char1: b'U',
        char2: b'R',
        result: 0x2517,
    },
    DigrT {
        char1: b'u',
        char2: b'l',
        result: 0x2518,
    },
    DigrT {
        char1: b'u',
        char2: b'L',
        result: 0x2519,
    },
    DigrT {
        char1: b'U',
        char2: b'l',
        result: 0x251a,
    },
    DigrT {
        char1: b'U',
        char2: b'L',
        result: 0x251b,
    },
    DigrT {
        char1: b'v',
        char2: b'r',
        result: 0x251c,
    },
    DigrT {
        char1: b'v',
        char2: b'R',
        result: 0x251d,
    },
    DigrT {
        char1: b'V',
        char2: b'r',
        result: 0x2520,
    },
    DigrT {
        char1: b'V',
        char2: b'R',
        result: 0x2523,
    },
    DigrT {
        char1: b'v',
        char2: b'l',
        result: 0x2524,
    },
    DigrT {
        char1: b'v',
        char2: b'L',
        result: 0x2525,
    },
    DigrT {
        char1: b'V',
        char2: b'l',
        result: 0x2528,
    },
    DigrT {
        char1: b'V',
        char2: b'L',
        result: 0x252b,
    },
    DigrT {
        char1: b'd',
        char2: b'h',
        result: 0x252c,
    },
    DigrT {
        char1: b'd',
        char2: b'H',
        result: 0x252f,
    },
    DigrT {
        char1: b'D',
        char2: b'h',
        result: 0x2530,
    },
    DigrT {
        char1: b'D',
        char2: b'H',
        result: 0x2533,
    },
    DigrT {
        char1: b'u',
        char2: b'h',
        result: 0x2534,
    },
    DigrT {
        char1: b'u',
        char2: b'H',
        result: 0x2537,
    },
    DigrT {
        char1: b'U',
        char2: b'h',
        result: 0x2538,
    },
    DigrT {
        char1: b'U',
        char2: b'H',
        result: 0x253b,
    },
    DigrT {
        char1: b'v',
        char2: b'h',
        result: 0x253c,
    },
    DigrT {
        char1: b'v',
        char2: b'H',
        result: 0x253f,
    },
    DigrT {
        char1: b'V',
        char2: b'h',
        result: 0x2542,
    },
    DigrT {
        char1: b'V',
        char2: b'H',
        result: 0x254b,
    },
    DigrT {
        char1: b'F',
        char2: b'D',
        result: 0x2571,
    },
    DigrT {
        char1: b'B',
        char2: b'D',
        result: 0x2572,
    },
    DigrT {
        char1: b'T',
        char2: b'B',
        result: 0x2580,
    },
    DigrT {
        char1: b'L',
        char2: b'B',
        result: 0x2584,
    },
    DigrT {
        char1: b'F',
        char2: b'B',
        result: 0x2588,
    },
    DigrT {
        char1: b'l',
        char2: b'B',
        result: 0x258c,
    },
    DigrT {
        char1: b'R',
        char2: b'B',
        result: 0x2590,
    },
    DigrT {
        char1: b'.',
        char2: b'S',
        result: 0x2591,
    },
    DigrT {
        char1: b':',
        char2: b'S',
        result: 0x2592,
    },
    DigrT {
        char1: b'?',
        char2: b'S',
        result: 0x2593,
    },
    DigrT {
        char1: b'f',
        char2: b'S',
        result: 0x25a0,
    },
    DigrT {
        char1: b'O',
        char2: b'S',
        result: 0x25a1,
    },
    DigrT {
        char1: b'R',
        char2: b'O',
        result: 0x25a2,
    },
    DigrT {
        char1: b'R',
        char2: b'r',
        result: 0x25a3,
    },
    DigrT {
        char1: b'R',
        char2: b'F',
        result: 0x25a4,
    },
    DigrT {
        char1: b'R',
        char2: b'Y',
        result: 0x25a5,
    },
    DigrT {
        char1: b'R',
        char2: b'H',
        result: 0x25a6,
    },
    DigrT {
        char1: b'R',
        char2: b'Z',
        result: 0x25a7,
    },
    DigrT {
        char1: b'R',
        char2: b'K',
        result: 0x25a8,
    },
    DigrT {
        char1: b'R',
        char2: b'X',
        result: 0x25a9,
    },
    DigrT {
        char1: b's',
        char2: b'B',
        result: 0x25aa,
    },
    DigrT {
        char1: b'S',
        char2: b'R',
        result: 0x25ac,
    },
    DigrT {
        char1: b'O',
        char2: b'r',
        result: 0x25ad,
    },
    DigrT {
        char1: b'U',
        char2: b'T',
        result: 0x25b2,
    },
    DigrT {
        char1: b'u',
        char2: b'T',
        result: 0x25b3,
    },
    DigrT {
        char1: b'P',
        char2: b'R',
        result: 0x25b6,
    },
    DigrT {
        char1: b'T',
        char2: b'r',
        result: 0x25b7,
    },
    DigrT {
        char1: b'D',
        char2: b't',
        result: 0x25bc,
    },
    DigrT {
        char1: b'd',
        char2: b'T',
        result: 0x25bd,
    },
    DigrT {
        char1: b'P',
        char2: b'L',
        result: 0x25c0,
    },
    DigrT {
        char1: b'T',
        char2: b'l',
        result: 0x25c1,
    },
    DigrT {
        char1: b'D',
        char2: b'b',
        result: 0x25c6,
    },
    DigrT {
        char1: b'D',
        char2: b'w',
        result: 0x25c7,
    },
    DigrT {
        char1: b'L',
        char2: b'Z',
        result: 0x25ca,
    },
    DigrT {
        char1: b'0',
        char2: b'm',
        result: 0x25cb,
    },
    DigrT {
        char1: b'0',
        char2: b'o',
        result: 0x25ce,
    },
    DigrT {
        char1: b'0',
        char2: b'M',
        result: 0x25cf,
    },
    DigrT {
        char1: b'0',
        char2: b'L',
        result: 0x25d0,
    },
    DigrT {
        char1: b'0',
        char2: b'R',
        result: 0x25d1,
    },
    DigrT {
        char1: b'S',
        char2: b'n',
        result: 0x25d8,
    },
    DigrT {
        char1: b'I',
        char2: b'c',
        result: 0x25d9,
    },
    DigrT {
        char1: b'F',
        char2: b'd',
        result: 0x25e2,
    },
    DigrT {
        char1: b'B',
        char2: b'd',
        result: 0x25e3,
    },
    DigrT {
        char1: b'*',
        char2: b'2',
        result: 0x2605,
    },
    DigrT {
        char1: b'*',
        char2: b'1',
        result: 0x2606,
    },
    DigrT {
        char1: b'<',
        char2: b'H',
        result: 0x261c,
    },
    DigrT {
        char1: b'>',
        char2: b'H',
        result: 0x261e,
    },
    DigrT {
        char1: b'0',
        char2: b'u',
        result: 0x263a,
    },
    DigrT {
        char1: b'0',
        char2: b'U',
        result: 0x263b,
    },
    DigrT {
        char1: b'S',
        char2: b'U',
        result: 0x263c,
    },
    DigrT {
        char1: b'F',
        char2: b'm',
        result: 0x2640,
    },
    DigrT {
        char1: b'M',
        char2: b'l',
        result: 0x2642,
    },
    DigrT {
        char1: b'c',
        char2: b'S',
        result: 0x2660,
    },
    DigrT {
        char1: b'c',
        char2: b'H',
        result: 0x2661,
    },
    DigrT {
        char1: b'c',
        char2: b'D',
        result: 0x2662,
    },
    DigrT {
        char1: b'c',
        char2: b'C',
        result: 0x2663,
    },
    DigrT {
        char1: b'M',
        char2: b'd',
        result: 0x2669,
    },
    DigrT {
        char1: b'M',
        char2: b'8',
        result: 0x266a,
    },
    DigrT {
        char1: b'M',
        char2: b'2',
        result: 0x266b,
    },
    DigrT {
        char1: b'M',
        char2: b'b',
        result: 0x266d,
    },
    DigrT {
        char1: b'M',
        char2: b'x',
        result: 0x266e,
    },
    DigrT {
        char1: b'M',
        char2: b'X',
        result: 0x266f,
    },
    DigrT {
        char1: b'O',
        char2: b'K',
        result: 0x2713,
    },
    DigrT {
        char1: b'X',
        char2: b'X',
        result: 0x2717,
    },
    DigrT {
        char1: b'-',
        char2: b'X',
        result: 0x2720,
    },
    DigrT {
        char1: b'I',
        char2: b'S',
        result: 0x3000,
    },
    DigrT {
        char1: b',',
        char2: b'_',
        result: 0x3001,
    },
    DigrT {
        char1: b'.',
        char2: b'_',
        result: 0x3002,
    },
    DigrT {
        char1: b'+',
        char2: b'"',
        result: 0x3003,
    },
    DigrT {
        char1: b'+',
        char2: b'_',
        result: 0x3004,
    },
    DigrT {
        char1: b'*',
        char2: b'_',
        result: 0x3005,
    },
    DigrT {
        char1: b';',
        char2: b'_',
        result: 0x3006,
    },
    DigrT {
        char1: b'0',
        char2: b'_',
        result: 0x3007,
    },
    DigrT {
        char1: b'<',
        char2: b'/',
        result: 0x3008,
    },
    DigrT {
        char1: b'/',
        char2: b'>',
        result: 0x3009,
    },
    DigrT {
        char1: b'<',
        char2: b'+',
        result: 0x300a,
    },
    DigrT {
        char1: b'>',
        char2: b'+',
        result: 0x300b,
    },
    DigrT {
        char1: b'<',
        char2: b'\'',
        result: 0x300c,
    },
    DigrT {
        char1: b'>',
        char2: b'\'',
        result: 0x300d,
    },
    DigrT {
        char1: b'<',
        char2: b'"',
        result: 0x300e,
    },
    DigrT {
        char1: b'>',
        char2: b'"',
        result: 0x300f,
    },
    DigrT {
        char1: b'(',
        char2: b'"',
        result: 0x3010,
    },
    DigrT {
        char1: b')',
        char2: b'"',
        result: 0x3011,
    },
    DigrT {
        char1: b'=',
        char2: b'T',
        result: 0x3012,
    },
    DigrT {
        char1: b'=',
        char2: b'_',
        result: 0x3013,
    },
    DigrT {
        char1: b'(',
        char2: b'\'',
        result: 0x3014,
    },
    DigrT {
        char1: b')',
        char2: b'\'',
        result: 0x3015,
    },
    DigrT {
        char1: b'(',
        char2: b'I',
        result: 0x3016,
    },
    DigrT {
        char1: b')',
        char2: b'I',
        result: 0x3017,
    },
    DigrT {
        char1: b'-',
        char2: b'?',
        result: 0x301c,
    },
    DigrT {
        char1: b'A',
        char2: b'5',
        result: 0x3041,
    },
    DigrT {
        char1: b'a',
        char2: b'5',
        result: 0x3042,
    },
    DigrT {
        char1: b'I',
        char2: b'5',
        result: 0x3043,
    },
    DigrT {
        char1: b'i',
        char2: b'5',
        result: 0x3044,
    },
    DigrT {
        char1: b'U',
        char2: b'5',
        result: 0x3045,
    },
    DigrT {
        char1: b'u',
        char2: b'5',
        result: 0x3046,
    },
    DigrT {
        char1: b'E',
        char2: b'5',
        result: 0x3047,
    },
    DigrT {
        char1: b'e',
        char2: b'5',
        result: 0x3048,
    },
    DigrT {
        char1: b'O',
        char2: b'5',
        result: 0x3049,
    },
    DigrT {
        char1: b'o',
        char2: b'5',
        result: 0x304a,
    },
    DigrT {
        char1: b'k',
        char2: b'a',
        result: 0x304b,
    },
    DigrT {
        char1: b'g',
        char2: b'a',
        result: 0x304c,
    },
    DigrT {
        char1: b'k',
        char2: b'i',
        result: 0x304d,
    },
    DigrT {
        char1: b'g',
        char2: b'i',
        result: 0x304e,
    },
    DigrT {
        char1: b'k',
        char2: b'u',
        result: 0x304f,
    },
    DigrT {
        char1: b'g',
        char2: b'u',
        result: 0x3050,
    },
    DigrT {
        char1: b'k',
        char2: b'e',
        result: 0x3051,
    },
    DigrT {
        char1: b'g',
        char2: b'e',
        result: 0x3052,
    },
    DigrT {
        char1: b'k',
        char2: b'o',
        result: 0x3053,
    },
    DigrT {
        char1: b'g',
        char2: b'o',
        result: 0x3054,
    },
    DigrT {
        char1: b's',
        char2: b'a',
        result: 0x3055,
    },
    DigrT {
        char1: b'z',
        char2: b'a',
        result: 0x3056,
    },
    DigrT {
        char1: b's',
        char2: b'i',
        result: 0x3057,
    },
    DigrT {
        char1: b'z',
        char2: b'i',
        result: 0x3058,
    },
    DigrT {
        char1: b's',
        char2: b'u',
        result: 0x3059,
    },
    DigrT {
        char1: b'z',
        char2: b'u',
        result: 0x305a,
    },
    DigrT {
        char1: b's',
        char2: b'e',
        result: 0x305b,
    },
    DigrT {
        char1: b'z',
        char2: b'e',
        result: 0x305c,
    },
    DigrT {
        char1: b's',
        char2: b'o',
        result: 0x305d,
    },
    DigrT {
        char1: b'z',
        char2: b'o',
        result: 0x305e,
    },
    DigrT {
        char1: b't',
        char2: b'a',
        result: 0x305f,
    },
    DigrT {
        char1: b'd',
        char2: b'a',
        result: 0x3060,
    },
    DigrT {
        char1: b't',
        char2: b'i',
        result: 0x3061,
    },
    DigrT {
        char1: b'd',
        char2: b'i',
        result: 0x3062,
    },
    DigrT {
        char1: b't',
        char2: b'U',
        result: 0x3063,
    },
    DigrT {
        char1: b't',
        char2: b'u',
        result: 0x3064,
    },
    DigrT {
        char1: b'd',
        char2: b'u',
        result: 0x3065,
    },
    DigrT {
        char1: b't',
        char2: b'e',
        result: 0x3066,
    },
    DigrT {
        char1: b'd',
        char2: b'e',
        result: 0x3067,
    },
    DigrT {
        char1: b't',
        char2: b'o',
        result: 0x3068,
    },
    DigrT {
        char1: b'd',
        char2: b'o',
        result: 0x3069,
    },
    DigrT {
        char1: b'n',
        char2: b'a',
        result: 0x306a,
    },
    DigrT {
        char1: b'n',
        char2: b'i',
        result: 0x306b,
    },
    DigrT {
        char1: b'n',
        char2: b'u',
        result: 0x306c,
    },
    DigrT {
        char1: b'n',
        char2: b'e',
        result: 0x306d,
    },
    DigrT {
        char1: b'n',
        char2: b'o',
        result: 0x306e,
    },
    DigrT {
        char1: b'h',
        char2: b'a',
        result: 0x306f,
    },
    DigrT {
        char1: b'b',
        char2: b'a',
        result: 0x3070,
    },
    DigrT {
        char1: b'p',
        char2: b'a',
        result: 0x3071,
    },
    DigrT {
        char1: b'h',
        char2: b'i',
        result: 0x3072,
    },
    DigrT {
        char1: b'b',
        char2: b'i',
        result: 0x3073,
    },
    DigrT {
        char1: b'p',
        char2: b'i',
        result: 0x3074,
    },
    DigrT {
        char1: b'h',
        char2: b'u',
        result: 0x3075,
    },
    DigrT {
        char1: b'b',
        char2: b'u',
        result: 0x3076,
    },
    DigrT {
        char1: b'p',
        char2: b'u',
        result: 0x3077,
    },
    DigrT {
        char1: b'h',
        char2: b'e',
        result: 0x3078,
    },
    DigrT {
        char1: b'b',
        char2: b'e',
        result: 0x3079,
    },
    DigrT {
        char1: b'p',
        char2: b'e',
        result: 0x307a,
    },
    DigrT {
        char1: b'h',
        char2: b'o',
        result: 0x307b,
    },
    DigrT {
        char1: b'b',
        char2: b'o',
        result: 0x307c,
    },
    DigrT {
        char1: b'p',
        char2: b'o',
        result: 0x307d,
    },
    DigrT {
        char1: b'm',
        char2: b'a',
        result: 0x307e,
    },
    DigrT {
        char1: b'm',
        char2: b'i',
        result: 0x307f,
    },
    DigrT {
        char1: b'm',
        char2: b'u',
        result: 0x3080,
    },
    DigrT {
        char1: b'm',
        char2: b'e',
        result: 0x3081,
    },
    DigrT {
        char1: b'm',
        char2: b'o',
        result: 0x3082,
    },
    DigrT {
        char1: b'y',
        char2: b'A',
        result: 0x3083,
    },
    DigrT {
        char1: b'y',
        char2: b'a',
        result: 0x3084,
    },
    DigrT {
        char1: b'y',
        char2: b'U',
        result: 0x3085,
    },
    DigrT {
        char1: b'y',
        char2: b'u',
        result: 0x3086,
    },
    DigrT {
        char1: b'y',
        char2: b'O',
        result: 0x3087,
    },
    DigrT {
        char1: b'y',
        char2: b'o',
        result: 0x3088,
    },
    DigrT {
        char1: b'r',
        char2: b'a',
        result: 0x3089,
    },
    DigrT {
        char1: b'r',
        char2: b'i',
        result: 0x308a,
    },
    DigrT {
        char1: b'r',
        char2: b'u',
        result: 0x308b,
    },
    DigrT {
        char1: b'r',
        char2: b'e',
        result: 0x308c,
    },
    DigrT {
        char1: b'r',
        char2: b'o',
        result: 0x308d,
    },
    DigrT {
        char1: b'w',
        char2: b'A',
        result: 0x308e,
    },
    DigrT {
        char1: b'w',
        char2: b'a',
        result: 0x308f,
    },
    DigrT {
        char1: b'w',
        char2: b'i',
        result: 0x3090,
    },
    DigrT {
        char1: b'w',
        char2: b'e',
        result: 0x3091,
    },
    DigrT {
        char1: b'w',
        char2: b'o',
        result: 0x3092,
    },
    DigrT {
        char1: b'n',
        char2: b'5',
        result: 0x3093,
    },
    DigrT {
        char1: b'v',
        char2: b'u',
        result: 0x3094,
    },
    DigrT {
        char1: b'"',
        char2: b'5',
        result: 0x309b,
    },
    DigrT {
        char1: b'0',
        char2: b'5',
        result: 0x309c,
    },
    DigrT {
        char1: b'*',
        char2: b'5',
        result: 0x309d,
    },
    DigrT {
        char1: b'+',
        char2: b'5',
        result: 0x309e,
    },
    DigrT {
        char1: b'a',
        char2: b'6',
        result: 0x30a1,
    },
    DigrT {
        char1: b'A',
        char2: b'6',
        result: 0x30a2,
    },
    DigrT {
        char1: b'i',
        char2: b'6',
        result: 0x30a3,
    },
    DigrT {
        char1: b'I',
        char2: b'6',
        result: 0x30a4,
    },
    DigrT {
        char1: b'u',
        char2: b'6',
        result: 0x30a5,
    },
    DigrT {
        char1: b'U',
        char2: b'6',
        result: 0x30a6,
    },
    DigrT {
        char1: b'e',
        char2: b'6',
        result: 0x30a7,
    },
    DigrT {
        char1: b'E',
        char2: b'6',
        result: 0x30a8,
    },
    DigrT {
        char1: b'o',
        char2: b'6',
        result: 0x30a9,
    },
    DigrT {
        char1: b'O',
        char2: b'6',
        result: 0x30aa,
    },
    DigrT {
        char1: b'K',
        char2: b'a',
        result: 0x30ab,
    },
    DigrT {
        char1: b'G',
        char2: b'a',
        result: 0x30ac,
    },
    DigrT {
        char1: b'K',
        char2: b'i',
        result: 0x30ad,
    },
    DigrT {
        char1: b'G',
        char2: b'i',
        result: 0x30ae,
    },
    DigrT {
        char1: b'K',
        char2: b'u',
        result: 0x30af,
    },
    DigrT {
        char1: b'G',
        char2: b'u',
        result: 0x30b0,
    },
    DigrT {
        char1: b'K',
        char2: b'e',
        result: 0x30b1,
    },
    DigrT {
        char1: b'G',
        char2: b'e',
        result: 0x30b2,
    },
    DigrT {
        char1: b'K',
        char2: b'o',
        result: 0x30b3,
    },
    DigrT {
        char1: b'G',
        char2: b'o',
        result: 0x30b4,
    },
    DigrT {
        char1: b'S',
        char2: b'a',
        result: 0x30b5,
    },
    DigrT {
        char1: b'Z',
        char2: b'a',
        result: 0x30b6,
    },
    DigrT {
        char1: b'S',
        char2: b'i',
        result: 0x30b7,
    },
    DigrT {
        char1: b'Z',
        char2: b'i',
        result: 0x30b8,
    },
    DigrT {
        char1: b'S',
        char2: b'u',
        result: 0x30b9,
    },
    DigrT {
        char1: b'Z',
        char2: b'u',
        result: 0x30ba,
    },
    DigrT {
        char1: b'S',
        char2: b'e',
        result: 0x30bb,
    },
    DigrT {
        char1: b'Z',
        char2: b'e',
        result: 0x30bc,
    },
    DigrT {
        char1: b'S',
        char2: b'o',
        result: 0x30bd,
    },
    DigrT {
        char1: b'Z',
        char2: b'o',
        result: 0x30be,
    },
    DigrT {
        char1: b'T',
        char2: b'a',
        result: 0x30bf,
    },
    DigrT {
        char1: b'D',
        char2: b'a',
        result: 0x30c0,
    },
    DigrT {
        char1: b'T',
        char2: b'i',
        result: 0x30c1,
    },
    DigrT {
        char1: b'D',
        char2: b'i',
        result: 0x30c2,
    },
    DigrT {
        char1: b'T',
        char2: b'U',
        result: 0x30c3,
    },
    DigrT {
        char1: b'T',
        char2: b'u',
        result: 0x30c4,
    },
    DigrT {
        char1: b'D',
        char2: b'u',
        result: 0x30c5,
    },
    DigrT {
        char1: b'T',
        char2: b'e',
        result: 0x30c6,
    },
    DigrT {
        char1: b'D',
        char2: b'e',
        result: 0x30c7,
    },
    DigrT {
        char1: b'T',
        char2: b'o',
        result: 0x30c8,
    },
    DigrT {
        char1: b'D',
        char2: b'o',
        result: 0x30c9,
    },
    DigrT {
        char1: b'N',
        char2: b'a',
        result: 0x30ca,
    },
    DigrT {
        char1: b'N',
        char2: b'i',
        result: 0x30cb,
    },
    DigrT {
        char1: b'N',
        char2: b'u',
        result: 0x30cc,
    },
    DigrT {
        char1: b'N',
        char2: b'e',
        result: 0x30cd,
    },
    DigrT {
        char1: b'N',
        char2: b'o',
        result: 0x30ce,
    },
    DigrT {
        char1: b'H',
        char2: b'a',
        result: 0x30cf,
    },
    DigrT {
        char1: b'B',
        char2: b'a',
        result: 0x30d0,
    },
    DigrT {
        char1: b'P',
        char2: b'a',
        result: 0x30d1,
    },
    DigrT {
        char1: b'H',
        char2: b'i',
        result: 0x30d2,
    },
    DigrT {
        char1: b'B',
        char2: b'i',
        result: 0x30d3,
    },
    DigrT {
        char1: b'P',
        char2: b'i',
        result: 0x30d4,
    },
    DigrT {
        char1: b'H',
        char2: b'u',
        result: 0x30d5,
    },
    DigrT {
        char1: b'B',
        char2: b'u',
        result: 0x30d6,
    },
    DigrT {
        char1: b'P',
        char2: b'u',
        result: 0x30d7,
    },
    DigrT {
        char1: b'H',
        char2: b'e',
        result: 0x30d8,
    },
    DigrT {
        char1: b'B',
        char2: b'e',
        result: 0x30d9,
    },
    DigrT {
        char1: b'P',
        char2: b'e',
        result: 0x30da,
    },
    DigrT {
        char1: b'H',
        char2: b'o',
        result: 0x30db,
    },
    DigrT {
        char1: b'B',
        char2: b'o',
        result: 0x30dc,
    },
    DigrT {
        char1: b'P',
        char2: b'o',
        result: 0x30dd,
    },
    DigrT {
        char1: b'M',
        char2: b'a',
        result: 0x30de,
    },
    DigrT {
        char1: b'M',
        char2: b'i',
        result: 0x30df,
    },
    DigrT {
        char1: b'M',
        char2: b'u',
        result: 0x30e0,
    },
    DigrT {
        char1: b'M',
        char2: b'e',
        result: 0x30e1,
    },
    DigrT {
        char1: b'M',
        char2: b'o',
        result: 0x30e2,
    },
    DigrT {
        char1: b'Y',
        char2: b'A',
        result: 0x30e3,
    },
    DigrT {
        char1: b'Y',
        char2: b'a',
        result: 0x30e4,
    },
    DigrT {
        char1: b'Y',
        char2: b'U',
        result: 0x30e5,
    },
    DigrT {
        char1: b'Y',
        char2: b'u',
        result: 0x30e6,
    },
    DigrT {
        char1: b'Y',
        char2: b'O',
        result: 0x30e7,
    },
    DigrT {
        char1: b'Y',
        char2: b'o',
        result: 0x30e8,
    },
    DigrT {
        char1: b'R',
        char2: b'a',
        result: 0x30e9,
    },
    DigrT {
        char1: b'R',
        char2: b'i',
        result: 0x30ea,
    },
    DigrT {
        char1: b'R',
        char2: b'u',
        result: 0x30eb,
    },
    DigrT {
        char1: b'R',
        char2: b'e',
        result: 0x30ec,
    },
    DigrT {
        char1: b'R',
        char2: b'o',
        result: 0x30ed,
    },
    DigrT {
        char1: b'W',
        char2: b'A',
        result: 0x30ee,
    },
    DigrT {
        char1: b'W',
        char2: b'a',
        result: 0x30ef,
    },
    DigrT {
        char1: b'W',
        char2: b'i',
        result: 0x30f0,
    },
    DigrT {
        char1: b'W',
        char2: b'e',
        result: 0x30f1,
    },
    DigrT {
        char1: b'W',
        char2: b'o',
        result: 0x30f2,
    },
    DigrT {
        char1: b'N',
        char2: b'6',
        result: 0x30f3,
    },
    DigrT {
        char1: b'V',
        char2: b'u',
        result: 0x30f4,
    },
    DigrT {
        char1: b'K',
        char2: b'A',
        result: 0x30f5,
    },
    DigrT {
        char1: b'K',
        char2: b'E',
        result: 0x30f6,
    },
    DigrT {
        char1: b'V',
        char2: b'a',
        result: 0x30f7,
    },
    DigrT {
        char1: b'V',
        char2: b'i',
        result: 0x30f8,
    },
    DigrT {
        char1: b'V',
        char2: b'e',
        result: 0x30f9,
    },
    DigrT {
        char1: b'V',
        char2: b'o',
        result: 0x30fa,
    },
    DigrT {
        char1: b'.',
        char2: b'6',
        result: 0x30fb,
    },
    DigrT {
        char1: b'-',
        char2: b'6',
        result: 0x30fc,
    },
    DigrT {
        char1: b'*',
        char2: b'6',
        result: 0x30fd,
    },
    DigrT {
        char1: b'+',
        char2: b'6',
        result: 0x30fe,
    },
    DigrT {
        char1: b'b',
        char2: b'4',
        result: 0x3105,
    },
    DigrT {
        char1: b'p',
        char2: b'4',
        result: 0x3106,
    },
    DigrT {
        char1: b'm',
        char2: b'4',
        result: 0x3107,
    },
    DigrT {
        char1: b'f',
        char2: b'4',
        result: 0x3108,
    },
    DigrT {
        char1: b'd',
        char2: b'4',
        result: 0x3109,
    },
    DigrT {
        char1: b't',
        char2: b'4',
        result: 0x310a,
    },
    DigrT {
        char1: b'n',
        char2: b'4',
        result: 0x310b,
    },
    DigrT {
        char1: b'l',
        char2: b'4',
        result: 0x310c,
    },
    DigrT {
        char1: b'g',
        char2: b'4',
        result: 0x310d,
    },
    DigrT {
        char1: b'k',
        char2: b'4',
        result: 0x310e,
    },
    DigrT {
        char1: b'h',
        char2: b'4',
        result: 0x310f,
    },
    DigrT {
        char1: b'j',
        char2: b'4',
        result: 0x3110,
    },
    DigrT {
        char1: b'q',
        char2: b'4',
        result: 0x3111,
    },
    DigrT {
        char1: b'x',
        char2: b'4',
        result: 0x3112,
    },
    DigrT {
        char1: b'z',
        char2: b'h',
        result: 0x3113,
    },
    DigrT {
        char1: b'c',
        char2: b'h',
        result: 0x3114,
    },
    DigrT {
        char1: b's',
        char2: b'h',
        result: 0x3115,
    },
    DigrT {
        char1: b'r',
        char2: b'4',
        result: 0x3116,
    },
    DigrT {
        char1: b'z',
        char2: b'4',
        result: 0x3117,
    },
    DigrT {
        char1: b'c',
        char2: b'4',
        result: 0x3118,
    },
    DigrT {
        char1: b's',
        char2: b'4',
        result: 0x3119,
    },
    DigrT {
        char1: b'a',
        char2: b'4',
        result: 0x311a,
    },
    DigrT {
        char1: b'o',
        char2: b'4',
        result: 0x311b,
    },
    DigrT {
        char1: b'e',
        char2: b'4',
        result: 0x311c,
    },
    DigrT {
        char1: b'a',
        char2: b'i',
        result: 0x311e,
    },
    DigrT {
        char1: b'e',
        char2: b'i',
        result: 0x311f,
    },
    DigrT {
        char1: b'a',
        char2: b'u',
        result: 0x3120,
    },
    DigrT {
        char1: b'o',
        char2: b'u',
        result: 0x3121,
    },
    DigrT {
        char1: b'a',
        char2: b'n',
        result: 0x3122,
    },
    DigrT {
        char1: b'e',
        char2: b'n',
        result: 0x3123,
    },
    DigrT {
        char1: b'a',
        char2: b'N',
        result: 0x3124,
    },
    DigrT {
        char1: b'e',
        char2: b'N',
        result: 0x3125,
    },
    DigrT {
        char1: b'e',
        char2: b'r',
        result: 0x3126,
    },
    DigrT {
        char1: b'i',
        char2: b'4',
        result: 0x3127,
    },
    DigrT {
        char1: b'u',
        char2: b'4',
        result: 0x3128,
    },
    DigrT {
        char1: b'i',
        char2: b'u',
        result: 0x3129,
    },
    DigrT {
        char1: b'v',
        char2: b'4',
        result: 0x312a,
    },
    DigrT {
        char1: b'n',
        char2: b'G',
        result: 0x312b,
    },
    DigrT {
        char1: b'g',
        char2: b'n',
        result: 0x312c,
    },
    DigrT {
        char1: b'1',
        char2: b'c',
        result: 0x3220,
    },
    DigrT {
        char1: b'2',
        char2: b'c',
        result: 0x3221,
    },
    DigrT {
        char1: b'3',
        char2: b'c',
        result: 0x3222,
    },
    DigrT {
        char1: b'4',
        char2: b'c',
        result: 0x3223,
    },
    DigrT {
        char1: b'5',
        char2: b'c',
        result: 0x3224,
    },
    DigrT {
        char1: b'6',
        char2: b'c',
        result: 0x3225,
    },
    DigrT {
        char1: b'7',
        char2: b'c',
        result: 0x3226,
    },
    DigrT {
        char1: b'8',
        char2: b'c',
        result: 0x3227,
    },
    DigrT {
        char1: b'9',
        char2: b'c',
        result: 0x3228,
    },
    DigrT {
        char1: b'f',
        char2: b'f',
        result: 0xfb00,
    },
    DigrT {
        char1: b'f',
        char2: b'i',
        result: 0xfb01,
    },
    DigrT {
        char1: b'f',
        char2: b'l',
        result: 0xfb02,
    },
    DigrT {
        char1: b'f',
        char2: b't',
        result: 0xfb05,
    },
    DigrT {
        char1: b's',
        char2: b't',
        result: 0xfb06,
    },
];

/// Get pointer to the default digraph table (FFI export for C callers).
#[no_mangle]
pub extern "C" fn rs_get_digraphdefault() -> *const DigrT {
    DIGRAPH_DEFAULT.as_ptr()
}

/// Get length of the default digraph table (FFI export for C callers).
#[no_mangle]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub extern "C" fn rs_get_digraphdefault_len() -> c_int {
    DIGRAPH_DEFAULT.len() as c_int
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_length() {
        assert_eq!(DIGRAPH_DEFAULT.len(), 1366);
    }

    #[test]
    fn test_first_entry() {
        let e = &DIGRAPH_DEFAULT[0];
        assert_eq!(e.char1, b'N');
        assert_eq!(e.char2, b'U');
        assert_eq!(e.result, 0x0a);
    }

    #[test]
    fn test_last_entry() {
        let e = &DIGRAPH_DEFAULT[DIGRAPH_DEFAULT.len() - 1];
        assert_eq!(e.char1, b's');
        assert_eq!(e.char2, b't');
        assert_eq!(e.result, 0xfb06);
    }

    #[test]
    fn test_entry_683() {
        let e = &DIGRAPH_DEFAULT[683];
        assert_eq!(e.char1, b'F');
        assert_eq!(e.char2, b'.');
        assert_eq!(e.result, 0x1e1e);
    }

    #[test]
    fn test_entry_100() {
        let e = &DIGRAPH_DEFAULT[100];
        assert_eq!(e.char1, b'R');
        assert_eq!(e.char2, b'g');
        assert_eq!(e.result, 0xae);
    }

    #[test]
    fn test_entry_500() {
        let e = &DIGRAPH_DEFAULT[500];
        assert_eq!(e.char1, b'Z');
        assert_eq!(e.char2, b'%');
        assert_eq!(e.result, 0x0416);
    }

    #[test]
    fn test_entry_1000() {
        let e = &DIGRAPH_DEFAULT[1000];
        assert_eq!(e.char1, b'3');
        assert_eq!(e.char2, b'/');
        assert_eq!(e.result, 0x2507);
    }

    #[test]
    fn test_entry_1300() {
        let e = &DIGRAPH_DEFAULT[1300];
        assert_eq!(e.char1, b'W');
        assert_eq!(e.char2, b'o');
        assert_eq!(e.result, 0x30f2);
    }

    #[test]
    fn test_no_nul_entries() {
        // Unlike C, our table has no NUL sentinel
        for (i, e) in DIGRAPH_DEFAULT.iter().enumerate() {
            assert!(e.char1 != 0 || e.char2 != 0, "NUL entry at index {i}");
        }
    }

    #[test]
    fn test_dg_start_constants() {
        assert_eq!(DG_START_LATIN, 0xa1);
        assert_eq!(DG_START_GREEK, 0x0386);
        assert_eq!(DG_START_CYRILLIC, 0x0401);
        assert_eq!(DG_START_HEBREW, 0x05d0);
        assert_eq!(DG_START_ARABIC, 0x060c);
        assert_eq!(DG_START_LATIN_EXTENDED, 0x1e02);
        assert_eq!(DG_START_GREEK_EXTENDED, 0x1f00);
        assert_eq!(DG_START_PUNCTUATION, 0x2002);
        assert_eq!(DG_START_SUB_SUPER, 0x2070);
        assert_eq!(DG_START_CURRENCY, 0x20a4);
        assert_eq!(DG_START_OTHER1, 0x2103);
        assert_eq!(DG_START_ROMAN, 0x2160);
        assert_eq!(DG_START_ARROWS, 0x2190);
        assert_eq!(DG_START_MATH, 0x2200);
        assert_eq!(DG_START_TECHNICAL, 0x2302);
        assert_eq!(DG_START_OTHER2, 0x2423);
        assert_eq!(DG_START_DRAWING, 0x2500);
        assert_eq!(DG_START_BLOCK, 0x2580);
        assert_eq!(DG_START_SHAPES, 0x25a0);
        assert_eq!(DG_START_SYMBOLS, 0x2605);
        assert_eq!(DG_START_DINGBATS, 0x2713);
        assert_eq!(DG_START_CJK_SYMBOLS, 0x3000);
        assert_eq!(DG_START_HIRAGANA, 0x3041);
        assert_eq!(DG_START_KATAKANA, 0x30a1);
        assert_eq!(DG_START_BOPOMOFO, 0x3105);
        assert_eq!(DG_START_OTHER3, 0x3220);
    }
}
