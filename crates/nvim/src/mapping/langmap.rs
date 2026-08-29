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

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::message_fmt::c_str_len;
use crate::swmsg;
use crate::types::NUL;
use core::ffi::{c_char, c_int};

/// One `'langmap'` pair for a character that does not fit `langmap_mapchar`.
#[derive(Copy, Clone)]
struct LangmapEntry {
    from: c_int,
    to: c_int,
}

/// `'langmap'` pairs for characters >= 256, sorted ascending by `from`.
///
/// Upstream is a `garray_T` of the same rows kept in the same order, because
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

/// Advance `p` past one character, honouring a `\` escape.
///
/// # Safety
/// `p` must point into a NUL-terminated string, at a byte that is not the NUL.
unsafe fn skip_escaped_char(p: *mut c_char) -> *mut c_char {
    // SAFETY: the caller's promise -- `p` is on a non-NUL byte of a
    // NUL-terminated string, so the byte after it is readable, and
    // `utfc_ptr2len` never steps past the NUL.
    unsafe {
        let p = if *p == b'\\' as c_char && *p.add(1) != 0 {
            p.add(1)
        } else {
            p
        };
        p.add(utfc_ptr2len(p) as usize)
    }
}

/// Called when the `'langmap'` option is set; the language map can be changed
/// at any time.
///
/// The option is a comma-separated list of pairs in either of two forms:
/// `aAbBcC` names them one after the other, and `abc;ABC` gives all the
/// `from` characters and then all the `to` characters.
///
/// # Safety
/// `args` must point at a live `optset_T` whose `os_errbuf` has room for
/// `os_errbuflen` bytes.
pub unsafe fn did_set_langmap(args: *mut optset_T) -> *const c_char {
    // SAFETY: the caller's promise -- `args` is a live `optset_T`, whose
    // `os_errbuf` has room for `os_errbuflen` bytes.
    let opts = unsafe { Live::new(args) };
    langmap_init(); // back to a one-to-one map
    let mut p = p_langmap.get();
    // SAFETY (this body): `p` and `p2` walk `'langmap'`, which is a live
    // NUL-terminated option string; every step below is guarded by the byte
    // it just read being non-NUL, so neither pointer leaves the string.
    loop {
        if unsafe { *p } == 0 {
            break;
        }
        // Find the ';' of an "abc;ABC" pair, if this comma-separated
        // group has one; p2 then walks the second half alongside p.
        let mut p2 = p;
        loop {
            let c = unsafe { *p2 };
            if c == 0 || c == b',' as c_char || c == b';' as c_char {
                break;
            }
            p2 = unsafe { skip_escaped_char(p2) };
        }
        p2 = if unsafe { *p2 } == b';' as c_char {
            unsafe { p2.add(1) } // "abcd;ABCD" form, p2 points at A
        } else {
            core::ptr::null_mut() // "aAbBcCdD" form
        };

        loop {
            let c = unsafe { *p };
            if c == 0 {
                break;
            }
            if c == b',' as c_char {
                p = unsafe { p.add(1) };
                break;
            }
            if c == b'\\' as c_char && unsafe { *p.add(1) } != 0 {
                p = unsafe { p.add(1) };
            }
            let from = unsafe { utf_ptr2char(p) };
            let from_ptr: *const c_char = p;
            let mut to = NUL;
            let mut to_ptr: *const c_char = c"".as_ptr();
            if p2.is_null() {
                p = unsafe { p.add(utfc_ptr2len(p) as usize) };
                if unsafe { *p } != b',' as c_char {
                    if unsafe { *p } == b'\\' as c_char {
                        p = unsafe { p.add(1) };
                    }
                    to_ptr = p;
                    to = unsafe { utf_ptr2char(to_ptr) };
                }
            } else if unsafe { *p2 } != b',' as c_char {
                if unsafe { *p2 } == b'\\' as c_char {
                    p2 = unsafe { p2.add(1) };
                }
                to_ptr = p2;
                to = unsafe { utf_ptr2char(to_ptr) };
            }
            if to == NUL {
                let (errbuf, errlen) = (opts.os_errbuf, opts.os_errbuflen);
                let missing = c"E357: 'langmap': Matching character missing for %s";
                // SAFETY: `os_errbuf` has room for `os_errbuflen` bytes, and
                // the format's one conversion is the rendering below it.
                unsafe {
                    let shown = transchar(from);
                    let fmt = gettext(missing);
                    snprintf(errbuf, errlen, fmt.as_ptr(), shown.as_ptr());
                }
                return errbuf;
            }

            if from >= 256 {
                langmap_set_entry(from, to);
            } else {
                if to > UCHAR_MAX {
                    // SAFETY: both `%.*s` pairs are a length and the bytes it
                    // counts, taken off the option string itself.
                    unsafe {
                        swmsg!(
                            true,
                            "'langmap': Mapping from {} to {} will not work properly",
                            c_str_len(from_ptr, utf_ptr2len(from_ptr) as usize),
                            c_str_len(to_ptr, utf_ptr2len(to_ptr) as usize)
                        );
                    }
                }
                // The closure is a store; it cannot re-enter the cell.
                langmap_mapchar.with_mut(|map| map[(from & 255) as usize] = to as u8);
            }

            // Advance to the next pair.
            p = unsafe { p.add(utfc_ptr2len(p) as usize) };
            if p2.is_null() {
                continue;
            }
            p2 = unsafe { p2.add(utfc_ptr2len(p2) as usize) };
            if unsafe { *p } != b';' as c_char {
                continue;
            }
            // The first half is exhausted; the rest of this group is
            // whatever p2 has left, which must be a comma or the end.
            p = p2;
            if unsafe { *p } != 0 {
                if unsafe { *p } != b',' as c_char {
                    let (errbuf, errlen) = (opts.os_errbuf, opts.os_errbuflen);
                    let extra = c"E358: 'langmap': Extra characters after semicolon: %s";
                    // SAFETY: as the `E357` message above.
                    unsafe {
                        let fmt = gettext(extra);
                        snprintf(errbuf, errlen, fmt.as_ptr(), p);
                    }
                    return errbuf;
                }
                p = unsafe { p.add(1) };
            }
            break;
        }
    }
    core::ptr::null()
}
