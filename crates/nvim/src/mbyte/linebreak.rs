//! Where `'linebreak'` may break a line, between two characters.
//!
//! CJK typesetting forbids two things, and they are *not* the same set: a
//! closing bracket or a comma may not start a line, and an opening bracket
//! may not end one. So the question has two halves — [`utf_allow_break_after`]
//! about the character before the break and [`utf_allow_break_before`] about
//! the character after it — and [`utf_allow_break`] asks both.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

/// Punctuation that may not start a line, so no break is allowed *before* it.
///
/// Sorted, because it is binary-searched.
#[rustfmt::skip]
const NO_BREAK_BEFORE: [c_int; 43] = [
    b'!' as c_int, b'%' as c_int, b')' as c_int, b',' as c_int, b':' as c_int,
    b';' as c_int, b'>' as c_int, b'?' as c_int, b']' as c_int, b'}' as c_int,
    0x2019,  // ’ right single quotation mark
    0x201d,  // ” right double quotation mark
    0x2020,  // † dagger
    0x2021,  // ‡ double dagger
    0x2026,  // … horizontal ellipsis
    0x2030,  // ‰ per mille sign
    0x2031,  // ‱ per ten thousand sign
    0x203c,  // ‼ double exclamation mark
    0x2047,  // ⁇ double question mark
    0x2048,  // ⁈ question exclamation mark
    0x2049,  // ⁉ exclamation question mark
    0x2103,  // ℃ degree celsius
    0x2109,  // ℉ degree fahrenheit
    0x3001,  // 、 ideographic comma
    0x3002,  // 。 ideographic full stop
    0x3009,  // 〉 right angle bracket
    0x300b,  // 》 right double angle bracket
    0x300d,  // 」 right corner bracket
    0x300f,  // 』 right white corner bracket
    0x3011,  // 】 right black lenticular bracket
    0x3015,  // 〕 right tortoise shell bracket
    0x3017,  // 〗 right white lenticular bracket
    0x3019,  // 〙 right white tortoise shell bracket
    0x301b,  // 〛 right white square bracket
    0xff01,  // ！ fullwidth exclamation mark
    0xff09,  // ） fullwidth right parenthesis
    0xff0c,  // ， fullwidth comma
    0xff0e,  // ． fullwidth full stop
    0xff1a,  // ： fullwidth colon
    0xff1b,  // ； fullwidth semicolon
    0xff1f,  // ？ fullwidth question mark
    0xff3d,  // ］ fullwidth right square bracket
    0xff5d,  // ｝ fullwidth right curly bracket
];

/// Punctuation that may not end a line, so no break is allowed *after* it.
///
/// The em dash and the swung dash are commented out in upstream's table,
/// which is worth keeping visible: they are the two whose typesetting rule is
/// disputed, not an omission.
#[rustfmt::skip]
const NO_BREAK_AFTER: [c_int; 19] = [
    b'(' as c_int, b'<' as c_int, b'[' as c_int, b'`' as c_int, b'{' as c_int,
    // 0x2014,  // — em dash
    0x2018,  // ‘ left single quotation mark
    0x201c,  // “ left double quotation mark
    // 0x2053,  // ～ swung dash
    0x3008,  // 〈 left angle bracket
    0x300a,  // 《 left double angle bracket
    0x300c,  // 「 left corner bracket
    0x300e,  // 『 left white corner bracket
    0x3010,  // 【 left black lenticular bracket
    0x3014,  // 〔 left tortoise shell bracket
    0x3016,  // 〖 left white lenticular bracket
    0x3018,  // 〘 left white tortoise shell bracket
    0x301a,  // 〚 left white square bracket
    0xff08,  // （ fullwidth left parenthesis
    0xff3b,  // ［ fullwidth left square bracket
    0xff5b,  // ｛ fullwidth left curly bracket
];

/// Both tables are binary-searched, which needs them sorted. Checked at
/// compile time; the only way to break it is to add a row out of order.
const fn is_sorted<const N: usize>(tab: &[c_int; N]) -> bool {
    let mut i = 1;
    while i < N {
        if tab[i - 1] >= tab[i] {
            return false;
        }
        i += 1;
    }
    true
}
const _: () = assert!(
    is_sorted(&NO_BREAK_BEFORE),
    "NO_BREAK_BEFORE must be sorted"
);
const _: () = assert!(is_sorted(&NO_BREAK_AFTER), "NO_BREAK_AFTER must be sorted");

/// Is `cc` one of the punctuation blocks whose spacing `'linebreak'` may eat?
pub fn utf_eat_space(cc: c_int) -> bool {
    (0x2000..=0x206f).contains(&cc)   // general punctuation
        || (0x2e00..=0x2e7f).contains(&cc) // supplemental punctuation
        || (0x3000..=0x303f).contains(&cc) // CJK symbols and punctuation
        || (0xff01..=0xff0f).contains(&cc) // fullwidth ASCII punctuation
        || (0xff1a..=0xff20).contains(&cc)
        || (0xff3b..=0xff40).contains(&cc)
        || (0xff5b..=0xff65).contains(&cc)
}

/// May a line break immediately before `cc`?
pub fn utf_allow_break_before(cc: c_int) -> bool {
    NO_BREAK_BEFORE.binary_search(&cc).is_err()
}

/// May a line break immediately after `cc`?
pub fn utf_allow_break_after(cc: c_int) -> bool {
    NO_BREAK_AFTER.binary_search(&cc).is_err()
}

/// May a line break between `cc` and `ncc`?
pub fn utf_allow_break(cc: c_int, ncc: c_int) -> bool {
    // A doubled em dash or ellipsis is one piece of punctuation written twice;
    // it must not be split down the middle.
    if cc == ncc && (cc == 0x2014 || cc == 0x2026) {
        return false;
    }
    utf_allow_break_after(cc) && utf_allow_break_before(ncc)
}
