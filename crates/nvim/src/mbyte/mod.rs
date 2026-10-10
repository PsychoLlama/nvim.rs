//! Multibyte text: UTF-8, the encodings around it, and everything that has
//! to know how wide a character is.
//!
//! nvim's internal encoding is always UTF-8; this family is what makes that
//! true at the edges and useful in the middle.
//!
//! - [`utf8`] is the codec, and its `tables` child the byte-length tables the
//!   whole tree indexes.
//! - [`walk`] moves around a buffer: character starts, grapheme clusters,
//!   cursor adjustment.
//! - [`cells`] answers display width, which is what the screen is laid out
//!   from, and carries `setcellwidths()`.
//! - [`class`] and [`case`] are the questions word motions and
//!   case-insensitive matching ask.
//! - [`linebreak`] is where `'linebreak'` may break a line.
//! - [`encoding`] canonicalises encoding names and [`convert`] converts
//!   between them -- the only place iconv is touched.
//!
//! What is left here is the shared C-level vocabulary, the foreign
//! declarations the children reach through `use super::*`, and four
//! editor-facing utilities that belong to no seam: `g8` ([`show_utf8`]),
//! `8g8` ([`utf_find_illegal`]), a validity check and the `K_SPECIAL`
//! unescaper.
//!
//! Three symbols in [`utf8`] are exported by name and may not change
//! signature (`utf_ptr2char`, `utf_ptr2len`, `utf_ptr2char_info_impl`), and
//! `utf8len_tab` is compiled *against* by `unit-fixtures.so`.
//!
//! # The slice API
//!
//! The multibyte cursor is the thing every other family reads text through,
//! and its C spelling is a pointer into a NUL-terminated string. The forms a
//! caller should reach for take a `&[u8]` instead, and carry Rust names; the
//! `utf_ptr2*`/`mb_*` spellings are what upstream called them, kept while
//! their callers convert and (for four of them) because something outside
//! the crate resolves the symbol.
//!
//! | upstream | slice form | answers |
//! | --- | --- | --- |
//! | `utf_ptr2char` | [`char_at`] | the codepoint, or the lead byte's own value |
//! | `utf_ptr2char_info_impl` | [`strict_char_at`] | the codepoint, or a negative "not a character" |
//! | `utf_ptr2char_info` | [`char_info_at`] | both of the above, as a [`CharInfo`] |
//! | `utf_ptr2len` | [`char_len`] | bytes in the character, never past the end |
//! | `utf_ptr2len_len` | [`promised_char_len`] | bytes the lead byte *promised* |
//! | `utfc_ptr2len`, `utfc_ptr2len_len` | [`cluster_len`] | bytes in the whole grapheme cluster |
//! | `utf_ptr2cells` | [`cells_at`] | screen cells the character occupies |
//! | `mb_string2cells`, `..._len` | [`string_cells`] | screen cells the whole text occupies |
//! | `mb_charlen`, `mb_charlen_len` | [`char_count`] | grapheme clusters in the text |
//! | `utf_cp_bounds`, `utf_cp_bounds_len` | [`cp_bounds`] | both ends of the codepoint covering a byte |
//! | `utf_head_off` | [`head_off`] | how far back the cluster covering a byte starts |
//! | `utf_iscomposing` | [`iscomposing`] | whether two codepoints cluster |
//! | the `MB_PTR_ADV` loop | [`clusters`], [`chars`] | one item per cluster |
//!
//! One rule covers the whole table, and it is the only behavioural
//! difference: **the end of the slice is the end of the string**. A NUL
//! inside a slice is an ordinary byte -- one character, one cell -- where the
//! pointer forms would stop there. That is what keeps a walk written against
//! these moving instead of standing still on an embedded NUL, and it is the
//! reason a caller passes the bytes it *means*, terminator excluded.
//!
//! # The pointer forms that stay
//!
//! Every reader is written once, in `utf8/text.rs`, over either kind of text,
//! and every pointer spelling that still has callers outside this family is
//! a one-line shim over it in [`utf8`] -- `utfc_ptr2len`, `utf_head_off`,
//! `utf_ptr2cells`, `utf_char2bytes`, `mb_prevptr`, `utfc_next` and the rest
//! -- so the two spellings are the same code and cannot drift. They stay
//! while their callers' cursors are pointers (the regexp engine, spell, the
//! drawing path) and go as those convert; nothing else in this family
//! touches a raw pointer. `utfc_ptr2schar` is what happens when one of them
//! stops being inlined on the drawing path.

// No `unsafe` here, and none allowed: the crate denies it. Not `forbid`,
// because a `forbid` reaches the whole subtree and `utf8/` keeps the
// family's exported symbols and pointer shims, which need the allow.
#![deny(unsafe_op_in_unsafe_fn)]

use core::fmt::Write as _;

use crate::cstr;
use crate::types::NL;
use crate::types::TAB;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use crate::ascii::ascii_iswhite;
use crate::charset::{ptr2cells_at, vim_isprintc, vim_iswordc_tab};
use crate::drawscreen::{UPD_NOT_VALID, redraw_all_later};
use crate::eval::typval::{tv_check_for_string_arg, tv_list_alloc, tv_list_alloc_ret};
use crate::getchar::beep_flush;
use crate::global_cell::GlobalCell;
use crate::grid::schar_from_buf;
use crate::keycodes::{K_SPECIAL, KE_FILLER};
use crate::message::e_listreq;
use crate::message::{emsg, msg};
use crate::r#move::changed_window_setting_all;
use crate::option::vars::{cmp_flags, fenc_default, p_emoji, p_enc};
use crate::options::{kOptCmpFlagInternal, kOptCmpFlagKeepascii};
use crate::optionstr::OptStringRef;
use crate::optionstr::check_chars_options;
use crate::os::cshim::gettext;
use crate::pos::MAXCOL;
use crate::types::{
    CONV_9_TO_UTF8, CONV_ICONV, CONV_NONE, CONV_NONE_INIT, CONV_TO_LATIN1, CONV_TO_LATIN9,
    CONV_TO_UTF8, CharBoundsOff, CharInfo, ColNr, EvalFuncData, Expand, GraphemeState, IOSIZE,
    List, MB_MAXCHAR, ScreenChar, StrCharInfo, TypVal, VarNumber, VimConv, int8_t, int32_t,
    ptrdiff_t, size_t, ssize_t, uint8_t, uint64_t, uintptr_t, utf8proc_int32_t,
};
use crate::utf8proc::{
    UTF8PROC_CASEFOLD, UTF8PROC_CATEGORY_ME, UTF8PROC_CATEGORY_MN, utf8proc_decompose_char,
    utf8proc_get_property, utf8proc_grapheme_break, utf8proc_grapheme_break_stateful,
    utf8proc_property_t, utf8proc_tolower, utf8proc_toupper,
};

// The carve of the transpiled module; see each child's docs.
mod case;
mod cells;
mod class;
mod convert;
mod encoding;
mod linebreak;
mod utf8;
mod walk;

pub use self::case::*;
pub use self::cells::*;
pub use self::class::*;
pub use self::convert::*;
pub use self::encoding::*;
pub use self::linebreak::*;
pub use self::utf8::*;
pub use self::walk::*;

/// The C-level vocabulary every transpiled module carries a copy of;
/// consolidating them tree-wide is a family of its own, not this slice's.
/// `setlocale`'s category for character classification.
pub const LC_CTYPE: c_int = 0;
/// `SIZE_MAX`, which `iconv` answers with on failure.
pub const SIZE_MAX: ::core::ffi::c_ulong = usize::MAX as ::core::ffi::c_ulong;
/// `INT_MAX`, the "no limit" length `utf_cp_bounds` passes.
pub const INT_MAX: c_int = c_int::MAX;
/// How many bytes a `ScreenChar` can hold, NUL included.
pub const MAX_SCHAR_SIZE: c_int = 32;
/// The second byte of an escaped literal `K_SPECIAL`; see [`mb_unescape`].
pub const KS_SPECIAL: c_int = 254;
/// The cursor line from the cursor on, as `get_cursor_pos_ptr` reads it.
fn with_cursor_text<R>(f: impl FnOnce(&[u8]) -> R) -> R {
    let cursor = Win::current().w_cursor;
    let mut lines = Buf::current().lines();
    let line = lines.line(cursor.lnum);
    let col = usize::try_from(cursor.col).unwrap_or(0);
    f(line.get(col..).unwrap_or_default())
}

/// `g8`: show the bytes of the character under the cursor, in hex.
///
/// A composing sequence is shown as its characters separated by `+`.
pub fn show_utf8() {
    // The whole grapheme cluster, composing characters included.
    let cluster = with_cursor_text(|text| text[..cluster_len(text)].to_vec());
    if cluster.is_empty() {
        msg(c"NUL", 0);
        return;
    }

    // Each character's bytes in hex, characters separated by "+".
    let mut hex = String::new();
    let mut at = 0;
    while at < cluster.len() {
        if at > 0 {
            hex.push_str("+ ");
        }
        for &byte in &cluster[at..at + char_len(&cluster[at..])] {
            // A NUL is stored in the buffer as a newline.
            let byte = if c_int::from(byte) == NL { 0 } else { byte };
            let _ = write!(hex, "{byte:02x} ");
        }
        at += char_len(&cluster[at..]);
        // Upstream shares `IObuff` and stops before it overflows; a cluster
        // has no length limit, so the same bound is kept.
        if hex.len() > (IOSIZE - 20) as usize {
            break;
        }
    }
    msg(&cstr::owned(hex.as_bytes()), 0);
}

/// The offset of the first byte of `text` that is not valid UTF-8, up to its
/// first NUL: too few continuation bytes -- which `char_len` reports as a
/// length of 1 -- or too many, an overlong sequence, which is caught by
/// re-encoding the decoded character and finding a shorter length.
fn first_illegal(text: &[u8]) -> Option<usize> {
    let text = &text[..text.iter().position(|&b| b == 0).unwrap_or(text.len())];
    let mut at = 0;
    while at < text.len() {
        let rest = &text[at..];
        let len = char_len(rest);
        if rest[0] >= 0x80
            && (len == 1 || usize::try_from(utf_char2len(char_at(rest))).ok() != Some(len))
        {
            return Some(at);
        }
        at += len;
    }
    None
}

/// `8g8`: move the cursor to the next byte that is not valid UTF-8.
///
/// When the file is being *stored* in an 8-bit encoding, the search runs over
/// the converted line rather than the buffer's, and the column has to be
/// walked back character by character, because the two do not share offsets.
pub fn utf_find_illegal() {
    let start = Win::current().w_cursor;
    let mut vimconv = CONV_NONE_INIT;

    let fenc = Buf::current().b_p_fenc.cstr().to_owned();
    if canon_props(fenc.to_bytes()) & ENC_8BIT != 0 {
        // 'encoding' is utf-8 but the file is 8-bit, so what is illegal is
        // decided after converting back to the file's encoding.
        let _ = p_enc(|value| vimconv.setup(Some(value), Some(fenc.as_c_str())));
    }

    Win::current().w_cursor.coladd = 0;
    let found = loop {
        let at = if vimconv.vc_type == CONV_NONE {
            with_cursor_text(first_illegal).map(|at| ColNr::try_from(at).unwrap_or(MAXCOL))
        } else {
            let Some(converted) = with_cursor_text(|text| vimconv.convert(text)) else {
                break false;
            };
            // `at` is an offset into the *converted* line; step the real
            // line forward by that many bytes' worth of characters to find
            // the matching column.
            first_illegal(&converted).map(|at| {
                with_cursor_text(|text| {
                    let mut col = 0;
                    for _ in 0..at {
                        let len = char_len(&text[col..]);
                        if len == 0 {
                            break;
                        }
                        col += len;
                    }
                    ColNr::try_from(col).unwrap_or(MAXCOL)
                })
            })
        };
        if let Some(at) = at {
            Win::current().w_cursor.col += at;
            break true;
        }

        if Win::current().w_cursor.lnum == Buf::current().b_ml.ml_line_count {
            break false;
        }
        Win::current().w_cursor.lnum += 1;
        Win::current().w_cursor.col = 0;
    };

    if !found {
        Win::current().w_cursor = start;
        beep_flush();
    }
}

/// Is every byte of `bytes` part of a well-formed UTF-8 sequence? A sequence
/// that runs past the end is a failure, not a truncation: this is the strict
/// answer, for callers deciding whether to accept a whole string at all.
pub fn valid_utf8(bytes: &[u8]) -> bool {
    let mut at = 0;
    while at < bytes.len() {
        let len = usize::from(utf8len_tab_zero[usize::from(bytes[at])]);
        if len == 0 {
            return false; // not a lead byte
        }
        let Some(sequence) = bytes.get(at..at + len) else {
            return false; // the sequence runs past the end
        };
        if !sequence[1..].iter().all(|&b| utf_is_trail_byte(b)) {
            return false;
        }
        at += len;
    }
    true
}

/// Decode one character out of the start of a key-encoded string.
///
/// Typeahead and mappings escape a literal `K_SPECIAL` byte as the three-byte
/// `K_SPECIAL KS_SPECIAL KE_FILLER`, so a multibyte character containing that
/// byte cannot be read directly. This unescapes as it goes and answers as
/// soon as it has a *multibyte* character: its bytes (NUL-padded) and how
/// many bytes of `text` they took.
///
/// `None` when the next character is ASCII, when a real special key is
/// reached (one cannot be part of a multibyte character), or when four bytes
/// have gone by without a character — four being the longest sequence anyone
/// types. The end of `text` reads as a NUL.
pub(crate) fn unescape_at(text: &[u8]) -> Option<([u8; MB_MAXCHAR], usize)> {
    let at = |i: usize| cstr::byte_at(text, i);
    let mut out = [0u8; MB_MAXCHAR];
    let mut len = 0;
    let mut used = 0;
    while at(used) != 0 && len < 4 {
        if c_int::from(at(used)) == K_SPECIAL {
            if c_int::from(at(used + 1)) != KS_SPECIAL || c_int::from(at(used + 2)) != KE_FILLER {
                break; // a real special key, which is never multibyte
            }
            out[len] = K_SPECIAL as u8;
            used += 2;
        } else {
            out[len] = at(used);
        }
        len += 1;

        // An illegal sequence answers 1 here, so this only fires on a
        // character that is really multibyte.
        if char_len(&out[..len]) > 1 {
            return Some((out, used + 1));
        }
        if out[0] < 128 {
            break; // ASCII: nothing more can make it multibyte
        }
        used += 1;
    }
    None
}
