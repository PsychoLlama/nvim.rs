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
const NO_BREAK_BEFORE: [char; 43] = [
    '!', '%', ')', ',', ':', ';', '>', '?', ']', '}',
    '\u{2019}',  // ’ right single quotation mark
    '\u{201d}',  // ” right double quotation mark
    '\u{2020}',  // † dagger
    '\u{2021}',  // ‡ double dagger
    '\u{2026}',  // … horizontal ellipsis
    '\u{2030}',  // ‰ per mille sign
    '\u{2031}',  // ‱ per ten thousand sign
    '\u{203c}',  // ‼ double exclamation mark
    '\u{2047}',  // ⁇ double question mark
    '\u{2048}',  // ⁈ question exclamation mark
    '\u{2049}',  // ⁉ exclamation question mark
    '\u{2103}',  // ℃ degree celsius
    '\u{2109}',  // ℉ degree fahrenheit
    '\u{3001}',  // 、 ideographic comma
    '\u{3002}',  // 。 ideographic full stop
    '\u{3009}',  // 〉 right angle bracket
    '\u{300b}',  // 》 right double angle bracket
    '\u{300d}',  // 」 right corner bracket
    '\u{300f}',  // 』 right white corner bracket
    '\u{3011}',  // 】 right black lenticular bracket
    '\u{3015}',  // 〕 right tortoise shell bracket
    '\u{3017}',  // 〗 right white lenticular bracket
    '\u{3019}',  // 〙 right white tortoise shell bracket
    '\u{301b}',  // 〛 right white square bracket
    '\u{ff01}',  // ！ fullwidth exclamation mark
    '\u{ff09}',  // ） fullwidth right parenthesis
    '\u{ff0c}',  // ， fullwidth comma
    '\u{ff0e}',  // ． fullwidth full stop
    '\u{ff1a}',  // ： fullwidth colon
    '\u{ff1b}',  // ； fullwidth semicolon
    '\u{ff1f}',  // ？ fullwidth question mark
    '\u{ff3d}',  // ］ fullwidth right square bracket
    '\u{ff5d}',  // ｝ fullwidth right curly bracket
];

/// Punctuation that may not end a line, so no break is allowed *after* it.
///
/// The em dash and the swung dash are commented out in upstream's table,
/// which is worth keeping visible: they are the two whose typesetting rule is
/// disputed, not an omission.
#[rustfmt::skip]
const NO_BREAK_AFTER: [char; 19] = [
    '(', '<', '[', '`', '{',
    // '\u{2014}',  // — em dash
    '\u{2018}',  // ‘ left single quotation mark
    '\u{201c}',  // “ left double quotation mark
    // '\u{2053}',  // ～ swung dash
    '\u{3008}',  // 〈 left angle bracket
    '\u{300a}',  // 《 left double angle bracket
    '\u{300c}',  // 「 left corner bracket
    '\u{300e}',  // 『 left white corner bracket
    '\u{3010}',  // 【 left black lenticular bracket
    '\u{3014}',  // 〔 left tortoise shell bracket
    '\u{3016}',  // 〖 left white lenticular bracket
    '\u{3018}',  // 〘 left white tortoise shell bracket
    '\u{301a}',  // 〚 left white square bracket
    '\u{ff08}',  // （ fullwidth left parenthesis
    '\u{ff3b}',  // ［ fullwidth left square bracket
    '\u{ff5b}',  // ｛ fullwidth left curly bracket
];

/// Both tables are binary-searched, which needs them sorted. Checked at
/// compile time; the only way to break it is to add a row out of order.
const fn is_sorted<const N: usize>(tab: &[char; N]) -> bool {
    let mut i = 1;
    while i < N {
        if tab[i - 1] as u32 >= tab[i] as u32 {
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
    scalar(cc).is_none_or(|c| NO_BREAK_BEFORE.binary_search(&c).is_err())
}

/// May a line break immediately after `cc`?
pub fn utf_allow_break_after(cc: c_int) -> bool {
    scalar(cc).is_none_or(|c| NO_BREAK_AFTER.binary_search(&c).is_err())
}

/// `cc` as a `char`, when it is one: every table entry is, so anything
/// that is not (a surrogate, past U+10FFFF, negative) is in neither.
fn scalar(cc: c_int) -> Option<char> {
    u32::try_from(cc).ok().and_then(char::from_u32)
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
