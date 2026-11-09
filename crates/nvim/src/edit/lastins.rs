//! The text of the last insert, and repeating it.
//!
//! `LAST_INSERT` is a copy of the redo buffer taken when Insert mode was
//! left, so it still has the command that *started* the insert on the front
//! -- `LAST_INSERT_SKIP` is how many bytes that command took, and every
//! reader goes through [`get_last_insert`], which skips them.
//!
//! [`get_last_insert`] is `".` and `i_CTRL-A`; [`get_last_insert_save`]
//! answers a copy with the trailing `<Esc>` removed; [`stuff_inserted`] is
//! `.`/CTRL-A/CTRL-@, which pushes the text back into the read buffer so the
//! main loop types it again, `count` times.  [`set_last_insert`] is the
//! single-character case `r` uses, which has to build the redo-buffer
//! spelling itself.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::types::Key;
use core::ffi::c_int;

use super::*;
use crate::memory::XString;
use crate::types::Failed;

/// Set the last inserted text to the single character `c`.
///
/// Used by `r`.  What is stored is the *redo buffer* spelling: a CTRL-V in
/// front of a control character, then the character, then the `<Esc>` that
/// ends an insert.
pub(crate) fn set_last_insert(c: Key) {
    let mut text = Vec::with_capacity(MB_MAXBYTES * 3 + 4);
    // The CTRL-V is only needed to enter a special character.
    if c.code() < 0x20 || c == Key::DEL {
        text.push(0x16); // Ctrl_V
    }
    push_char_escaped(c, &mut text);
    text.push(0x1b); // ESC
    LAST_INSERT.set(ApiString::from_bytes(&text));
    LAST_INSERT_SKIP.set(0);
}

/// `.`, `i_CTRL-A` and `i_CTRL-@`: stuff the last inserted text into the read
/// buffer so the main loop types it again.
///
/// `c` is the command character that starts Insert mode (NUL for none),
/// `count` how many times, and `no_esc` says to leave the insert open at the
/// end.
///
/// `Err` -- with `E29` -- when there is nothing to insert.
pub(crate) fn stuff_inserted(c: Key, mut count: c_int, no_esc: bool) -> Result<(), Failed> {
    let mut insert = get_last_insert(); // text to be inserted
    if insert.is_null() {
        emsg(gettext(E_NOINSTEXT));
        return Err(Failed);
    }

    // May want to stuff the command character, to start Insert mode.
    if !c.is_nul() {
        stuff_readbuf_char(c);
    }

    // Cut the text at the last ESC: what follows it is not part of the
    // insert.
    if let Some(i) = insert.as_bytes().iter().rposition(|&b| b == 0x1b) {
        insert = ApiString::from_bytes(&insert.as_bytes()[..i]);
    }

    // A trailing `0` or `^` has to be quoted, because either would be
    // read as the start of `0 CTRL-D`/`^ CTRL-D` -- but only when
    // nothing follows it (no ESC is coming) or when the text is repeated
    // and starts with CTRL-D.  -- Acevedo
    let mut last = b'\0';
    if let (Some(&first), Some(&end)) = (insert.as_bytes().first(), insert.as_bytes().last())
        && matches!(end, b'0' | b'^')
        && (no_esc || (Key::byte(first) == Key::CTRL_D && count > 1))
    {
        last = end;
        let shorter = insert.len() - 1;
        insert = ApiString::from_bytes(&insert.as_bytes()[..shorter]);
    }

    loop {
        stuff_readbuf_len(insert.as_bytes());
        // The quoted forms: `0` as `<C-V>048`, `^` as `<C-V>^`.
        if last == b'0' {
            stuff_readbuf_len(b"\x16048");
        } else if last == b'^' {
            stuff_readbuf_len(b"\x16^");
        }
        count -= 1;
        if count <= 0 {
            break;
        }
    }

    // May want to stuff a trailing ESC, to get out of Insert mode.
    if !no_esc {
        stuff_readbuf_char(Key::ESC);
    }
    Ok(())
}

/// The last inserted text, without the command that started the insert.
///
/// A copy: both callers shorten what they get, and the stored text is not
/// theirs to shorten.
pub(crate) fn get_last_insert() -> ApiString {
    // `LAST_INSERT_SKIP` counts bytes this module put on the front, so it
    // never runs past the buffer.
    let skip = size_t::try_from(LAST_INSERT_SKIP.get()).unwrap_or(0);
    LAST_INSERT.with(|all| {
        if all.is_null() {
            ApiString::NULL
        } else {
            ApiString::from_bytes(&all.as_bytes()[skip..])
        }
    })
}

/// The last inserted text as its own copy, with the trailing `<Esc>`
/// removed, as far as a C string would read it (to a NUL). `None` when there
/// is none.
pub(crate) fn get_last_insert_save() -> Option<XString> {
    let insert = get_last_insert();
    if insert.is_null() {
        return None;
    }
    let mut text = insert.as_bytes();
    // The copy is shortened, not the string it came from.
    if text.last() == Some(&0x1b) {
        text = &text[..text.len() - 1];
    }
    Some(XString::from_bytes(cstr::until_nul(text)))
}
