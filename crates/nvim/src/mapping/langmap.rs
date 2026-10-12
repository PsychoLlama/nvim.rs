//! `'langmap'`: a second keyboard layout for command keys.
//!
//! Keyboards with a language mode send characters Vim has no command for.
//! `'langmap'` says which command character each of them stands for, so the
//! keyboard does not have to be switched back to ASCII to leave Insert mode.
//!
//! Two tables hold the answer, and both are written only from here.
//! `langmap_mapchar` covers the 256 single-byte characters directly;
//! [`LANGMAP_MULTIBYTE`] is a sorted table of `from`/`to` pairs doing the same
//! job for everything at or above U+0100, searched by [`langmap_adjust_mb`].
//! [`did_set_langmap`] parses the option into both.

#![forbid(unsafe_code)]

use super::*;
use crate::formatted;
use crate::mbyte::{char_at, char_len};
use crate::memory::XString;
use crate::message_fmt::msg_bytes;
use crate::option::vars::P_LANGMAP;
use crate::swmsg;
use crate::types::{NUL, OptError};
use core::ffi::c_int;

/// One `'langmap'` pair for a character that does not fit `langmap_mapchar`.
struct LangmapEntry {
    from: c_int,
    to: c_int,
}

/// `'langmap'` pairs for characters >= 256, sorted ascending by `from`.
///
/// Upstream is a `GArray` of the same rows kept in the same order, because
/// both readers binary-search it. Nothing outside this file names it.
static LANGMAP_MULTIBYTE: GlobalCell<Vec<LangmapEntry>> = GlobalCell::new(Vec::new());

/// Point `from` at `to`, replacing any existing pair for `from`.
fn langmap_set_entry(from: c_int, to: c_int) {
    LANGMAP_MULTIBYTE.with_mut(|entries| {
        match entries.binary_search_by_key(&from, |entry| entry.from) {
            Ok(at) => entries[at].to = to,
            Err(at) => entries.insert(at, LangmapEntry { from, to }),
        }
    });
}

/// Apply `'langmap'` to multi-byte character `c` and return the result.
pub(crate) fn langmap_adjust_mb(c: c_int) -> c_int {
    LANGMAP_MULTIBYTE.with(|entries| {
        match entries.binary_search_by_key(&c, |entry| entry.from) {
            Ok(at) => entries[at].to,
            // No entry: the character maps to itself.
            Err(_) => c,
        }
    })
}

/// Reset both tables to the identity mapping.
///
/// Upstream spells this `ga_clear` + `ga_init`, which are one `Vec::clear`
/// here: the free and the re-initialise are the same operation on an owning
/// container.
pub(crate) fn langmap_init() {
    let mut identity = [0u8; 256];
    for (i, slot) in identity.iter_mut().enumerate() {
        *slot = i as u8;
    }
    langmap_mapchar.set(identity);
    LANGMAP_MULTIBYTE.with_mut(Vec::clear);
}

/// Called when the `'langmap'` option is set; the language map can be changed
/// at any time.
///
/// The option is a comma-separated list of pairs in either of two forms:
/// `aAbBcC` names them one after the other, and `abc;ABC` gives all the
/// `from` characters and then all the `to` characters.
///
/// Upstream walks the option with two `char *`; here they are two indices
/// into the same slice.
pub fn did_set_langmap(_args: &mut OptSet) -> Result<(), OptError> {
    langmap_init(); // back to a one-to-one map
    // A copy: the walk below outlives a projection's borrow.
    let langmap = P_LANGMAP.get();
    let opt = langmap.as_cstr().to_bytes();
    // The byte at `at`, with the option's own NUL past the end.
    let byte = |at: usize| opt.get(at).copied().unwrap_or(0);
    let tail = |at: usize| opt.get(at..).unwrap_or_default();
    let char_at = |at: usize| char_at(tail(at));
    let len_at = |at: usize| cluster_len(tail(at));
    // Advance past one character, honouring a `\` escape.
    let skip = |at: usize| {
        let at = if byte(at) == b'\\' && byte(at + 1) != 0 {
            at + 1
        } else {
            at
        };
        at + len_at(at)
    };
    // The error texts, which both name what they did not like through
    // their one `%s`.
    let fail = |fmt: &'static CStr, arg: &[u8]| {
        let message = formatted!(OptError::ROOM, gettext(fmt).to_bytes(), arg);
        Err(OptError::Owned(XString::from_vec(message)))
    };

    let mut at = 0;
    while byte(at) != 0 {
        // Find the ';' of an "abc;ABC" pair, if this comma-separated
        // group has one; `alt` then walks the second half alongside `at`.
        let mut end = at;
        while !matches!(byte(end), 0 | b',' | b';') {
            end = skip(end);
        }
        let mut alt = (byte(end) == b';').then(|| end + 1);

        loop {
            if byte(at) == 0 {
                break;
            }
            if byte(at) == b',' {
                at += 1;
                break;
            }
            if byte(at) == b'\\' && byte(at + 1) != 0 {
                at += 1;
            }
            let (from, from_at) = (char_at(at), at);
            let mut to = NUL;
            let mut to_at = None;
            match alt {
                None => {
                    at += len_at(at);
                    if byte(at) != b',' {
                        if byte(at) == b'\\' {
                            at += 1;
                        }
                        to_at = Some(at);
                        to = char_at(at);
                    }
                }
                Some(mut second) if byte(second) != b',' => {
                    if byte(second) == b'\\' {
                        second += 1;
                        alt = Some(second);
                    }
                    to_at = Some(second);
                    to = char_at(second);
                }
                Some(_) => {}
            }
            if to == NUL {
                let missing = c"E357: 'langmap': Matching character missing for %s";
                return fail(missing, crate::cstr::in_chars(&transchar(from)).to_bytes());
            }

            if from >= 256 {
                langmap_set_entry(from, to);
            } else {
                if to > UCHAR_MAX {
                    let to_at = to_at.unwrap_or(from_at);
                    // Each `%.*s` is the one character at its offset.
                    let one = |at: usize| &tail(at)[..char_len(tail(at))];
                    swmsg!(
                        true,
                        "'langmap': Mapping from {} to {} will not work properly",
                        msg_bytes(one(from_at)),
                        msg_bytes(one(to_at))
                    );
                }
                // The closure is a store; it cannot re-enter the cell.
                langmap_mapchar.with_mut(|map| map[(from & 255) as usize] = to as u8);
            }

            // Advance to the next pair.
            at += len_at(at);
            let Some(second) = alt else { continue };
            let second = second + len_at(second);
            alt = Some(second);
            if byte(at) != b';' {
                continue;
            }
            // The first half is exhausted; the rest of this group is
            // whatever the second half has left, which must be a comma or
            // the end.
            at = second;
            if byte(at) != 0 {
                if byte(at) != b',' {
                    let extra = c"E358: 'langmap': Extra characters after semicolon: %s";
                    return fail(extra, tail(at));
                }
                at += 1;
            }
            break;
        }
    }
    Ok(())
}
