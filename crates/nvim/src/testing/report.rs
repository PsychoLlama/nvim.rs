//! Building the line a failed `assert_*()` appends to `v:errors`.
//!
//! [`prepare_assert_error`] opens the buffer with the sourcing position,
//! [`fill_assert_error`] says what was expected and what arrived, and the
//! caller publishes it with [`report_assert_error`]. Unprintable bytes are
//! escaped on the way in, and a long run of one character is collapsed, so
//! that a failure over binary data is still readable.
//!
//! Every string in here is matched on by tests. None of it may drift.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::CStr;

use crate::eval::encode::{encode_tv2echo, encode_tv2string};
use crate::eval::typval::{tv_dict_alloc, tv_equal};
use crate::eval::vars::assert_error;
use crate::mbyte::{char_at, char_len};
use crate::memory::ThinCString;
use crate::runtime::estack_sfile_owned;
use crate::types::{LineNr, TypVal};

use super::{AssertType, ESTACK_NONE};

/// Append a literal message part.
pub(super) fn push_lit(message: &mut Vec<u8>, text: &'static CStr) {
    message.extend_from_slice(text.to_bytes());
}

/// Line number being sourced or executed: the top of the exestack.
pub(super) fn sourcing_lnum() -> LineNr {
    crate::runtime::innermost_frame().es_lnum
}

/// A fresh message, opened with the sourcing position: `script line N: `.
pub(super) fn prepare_assert_error() -> Vec<u8> {
    let mut message = Vec::new();
    let source = estack_sfile_owned(ESTACK_NONE);
    let lnum = sourcing_lnum();
    if let Some(source) = &source {
        message.extend_from_slice(source.as_cstr().to_bytes());
        if lnum > 0 {
            message.push(b' ');
        }
    }
    if lnum > 0 {
        message.extend_from_slice(format!("line {lnum}").as_bytes());
    }
    if source.is_some() || lnum > 0 {
        push_lit(&mut message, c": ");
    }
    message
}

/// Publish `message` as one `v:errors` entry.
pub(super) fn report_assert_error(message: &[u8]) {
    assert_error(message);
}

// ---------------------------------------------------------------------------
// Escaping
// ---------------------------------------------------------------------------

/// Append one character, escaping it when it is an unprintable single byte.
///
/// NL becomes `\n`, CR `\r`, and anything else below space (or DEL) becomes
/// `\xNN`. A multibyte character goes through unchanged.
fn push_escaped(message: &mut Vec<u8>, character: &[u8]) {
    let &[byte] = character else {
        message.extend_from_slice(character);
        return;
    };
    let escaped = match byte {
        b'\x08' => Some(c"\\b"),
        b'\x1b' => Some(c"\\e"),
        b'\x0c' => Some(c"\\f"),
        b'\n' => Some(c"\\n"),
        b'\t' => Some(c"\\t"),
        b'\r' => Some(c"\\r"),
        b'\\' => Some(c"\\\\"),
        _ => None,
    };
    if let Some(text) = escaped {
        push_lit(message, text);
    } else if byte < b' ' || byte == 0x7f {
        message.extend_from_slice(format!("\\x{byte:02x}").as_bytes());
    } else {
        message.push(byte);
    }
}

/// Append `text` escaped, collapsing a run of more than 20 identical
/// characters into `\[c occurs N times]` so a long message stays readable.
///
/// The text ends at its first NUL, as the C string it was did.
fn push_shortened(message: &mut Vec<u8>, text: &[u8]) {
    let text = text.split(|&byte| byte == 0).next().unwrap_or_default();
    let mut at = 0;
    while at < text.len() {
        let rest = &text[at..];
        let character = char_at(rest);
        let clen = char_len(rest);
        // How many times the character repeats, counted in steps of its
        // own length.
        let mut same_len = 1;
        let mut next = clen;
        while next < rest.len() && char_at(&rest[next..]) == character {
            same_len += 1;
            next += clen;
        }
        if same_len > 20 {
            push_lit(message, c"\\[");
            push_escaped(message, &rest[..clen]);
            push_lit(message, c" occurs ");
            message.extend_from_slice(format!("{same_len}").as_bytes());
            push_lit(message, c" times]");
            at += next;
        } else {
            push_escaped(message, &rest[..clen]);
            at += clen;
        }
    }
}

// ---------------------------------------------------------------------------
// The failure message
// ---------------------------------------------------------------------------

/// Prefix `message` with the caller's own, when they gave one.
///
/// An empty string counts as no message, which is what lets every
/// `assert_*()`'s optional `msg` argument be passed through unconditionally.
fn append_opt_msg(message: &mut Vec<u8>, opt_msg_tv: Option<&TypVal>) {
    let Some(msg) = opt_msg_tv else {
        return;
    };
    let blank = msg.is_string() && msg.string_ref().is_none_or(ThinCString::is_empty);
    if blank {
        return;
    }
    message.extend_from_slice(encode_tv2echo(msg).as_bytes());
    push_lit(message, c": ");
}

/// Copies of both dictionaries holding only the entries that differ, and how
/// many equal ones were dropped. `None` unless both hold a dictionary.
///
/// Comparing two large dictionaries is unreadable unless the equal items go
/// away. The two answers own their dictionaries.
///
/// The keys are listed first and every entry is found again by key: copying
/// a value that is one of these dictionaries counts a reference on it, which
/// must not happen while its slots are being walked.
fn prune_equal_dict_items(exp_tv: &TypVal, got_tv: &TypVal) -> Option<(TypVal, TypVal, usize)> {
    let (exp_dict, got_dict) = (exp_tv.dict_ref()?, got_tv.dict_ref()?);
    let exp_keys: Vec<Vec<u8>> = exp_dict.items().map(|item| item.key().to_vec()).collect();
    let got_keys: Vec<Vec<u8>> = got_dict.items().map(|item| item.key().to_vec()).collect();
    let mut exp_pruned = TypVal::dict(Some(tv_dict_alloc()));
    let mut got_pruned = TypVal::dict(Some(tv_dict_alloc()));
    fn find<'a>(tv: &'a TypVal, key: &[u8]) -> Option<&'a TypVal> {
        tv.dict_ref()
            .and_then(|dict| dict.find(key))
            .map(|item| &item.di_tv)
    }
    let add = |pruned: &mut TypVal, key: &[u8], value: &TypVal| {
        if let Some(dict) = pruned.dict_mut() {
            let _ = dict.add_tv(key, value);
        }
    };

    let mut omitted = 0;
    for key in &exp_keys {
        let Some(expected) = find(exp_tv, key) else {
            continue;
        };
        let got = find(got_tv, key);
        if got.is_some_and(|got| tv_equal(expected, got, false)) {
            omitted += 1;
            continue;
        }
        // Absent from the actual value, or present with a different one.
        add(&mut exp_pruned, key, expected);
        if let Some(got) = find(got_tv, key) {
            add(&mut got_pruned, key, got);
        }
    }
    // Entries only the actual value has.
    for key in &got_keys {
        if find(exp_tv, key).is_none()
            && let Some(got) = find(got_tv, key)
        {
            add(&mut got_pruned, key, got);
        }
    }
    Some((exp_pruned, got_pruned, omitted))
}

/// What a failed check expected: text already formatted (`True`, a range,
/// the pattern `assert_fails()` was given), or a value to encode.
#[derive(Clone, Copy)]
pub(super) enum Expected<'a> {
    Text(&'a [u8]),
    Value(&'a TypVal),
}

/// Fill `message` with what was expected and what arrived.
///
/// `ASSERT_NOTEQUAL` prints no "but got" half — for it the actual value
/// *is* the expected one.
pub(super) fn fill_assert_error(
    message: &mut Vec<u8>,
    opt_msg_tv: Option<&TypVal>,
    expected: Expected<'_>,
    got_tv: &TypVal,
    atype: AssertType,
) {
    // Two dictionaries read better with their equal entries taken out; the
    // pruned copies belong to this frame and go with it.
    let pruned = match expected {
        Expected::Value(exp_tv) if atype != AssertType::NotEqual => {
            prune_equal_dict_items(exp_tv, got_tv)
        }
        _ => None,
    };
    let (expected, got_tv, omitted) = match &pruned {
        Some((exp, got, omitted)) => (Expected::Value(exp), got, *omitted),
        None => (expected, got_tv, 0),
    };

    append_opt_msg(message, opt_msg_tv);
    push_lit(
        message,
        match atype {
            AssertType::Match | AssertType::NotMatch => c"Pattern ",
            AssertType::NotEqual => c"Expected not equal to ",
            _ => c"Expected ",
        },
    );

    match expected {
        Expected::Value(exp_tv) => push_shortened(message, encode_tv2string(exp_tv).as_bytes()),
        Expected::Text(text) => {
            let quoted = atype == AssertType::Fails;
            if quoted {
                push_lit(message, c"'");
            }
            push_shortened(message, text);
            if quoted {
                push_lit(message, c"'");
            }
        }
    }

    if atype != AssertType::NotEqual {
        push_lit(
            message,
            match atype {
                AssertType::Match => c" does not match ",
                AssertType::NotMatch => c" does match ",
                _ => c" but got ",
            },
        );
        push_shortened(message, encode_tv2string(got_tv).as_bytes());

        if omitted != 0 {
            let plural = if omitted == 1 { "" } else { "s" };
            let text = format!(" - {omitted} equal item{plural} omitted");
            message.extend_from_slice(text.as_bytes());
        }
    }
}
