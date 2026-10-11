//! Case conversion.
//!
//! [`strcase_copy`] folds per character through `mb_toupper`/`mb_tolower`,
//! and a folded character may encode longer or shorter than its original.
//! (The ASCII-only, locale-independent folds upstream keeps beside it had no
//! caller left; `[u8]::make_ascii_uppercase` is that.)

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::mbyte::{char_info_at, encode_char, mb_tolower, mb_toupper};
use crate::memory::XString;
use crate::types::MB_MAXCHAR;

/// A copy of `text` with every character upper- (`upper`) or lower-cased,
/// multibyte-aware.
///
/// A byte that starts no character is folded as the codepoint of its own
/// value and written back UTF-8 encoded -- a lone `0xC1` comes out as
/// `U+00C1`'s two bytes -- which is upstream's `strcase_save`, byte for
/// byte. Folding stops at a NUL, as the C string did -- one read from the
/// text or one an overlong sequence decodes to.
pub fn strcase_copy(text: &[u8], upper: bool) -> XString {
    let text = &text[..text.iter().position(|&b| b == 0).unwrap_or(text.len())];
    let mut out = Vec::with_capacity(text.len());
    let mut at = 0;
    while at < text.len() {
        let info = char_info_at(&text[at..]);
        let c = if info.value < 0 {
            i32::from(text[at])
        } else {
            info.value
        };
        let folded = if upper { mb_toupper(c) } else { mb_tolower(c) };
        if folded == 0 {
            // An overlong NUL (`C0 80`) folds to the terminator, and the C
            // string ended there.
            break;
        }
        let mut encoded = [0u8; MB_MAXCHAR];
        let len = encode_char(folded, &mut encoded);
        out.extend_from_slice(&encoded[..len]);
        at += usize::try_from(info.len).unwrap_or(1).max(1);
    }
    XString::from_vec(out)
}
