//! `eval/encode.c`: the shared half of the typval encoders.
//!
//! The four sinks upstream instantiates out of `typval_encode.c.h` live in the
//! three children beside this file — [`json`], [`msgpack`] and [`text`]
//! (`string()` and `:echo` are one `impl` there).  What stays here is what
//! they share, plus what belongs to no sink at all:
//!
//! - [`conv_error`], the failure path all three report through, which renders
//!   the walk's stack as `key foo, index 2, key bar`;
//! - [`convert_to_json_string`], JSON's string escaping — the one hook whose
//!   body is byte arithmetic rather than punctuation;
//! - [`encode_check_json_key`], the special-dictionary key test;
//! - the three `encode_tv2*` entry points; and
//! - the `readfile()`-style list codec ([`encode_list_write`],
//!   [`ListReader`], [`encode_vim_list_to_buf`]) that msgpack channels,
//!   `msgpackdump()` and `system()` read and write through.  Its one
//!   convention: a list item is a line, and a NUL inside a line is stored as a
//!   newline, because a Vimscript string cannot hold a newline.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::memory::ThinCString;
use core::ffi::{CStr, c_int};

use crate::eval::typval::{dict_find, list_len};
use crate::eval::typval_encode::{ConvPath, Flow, Frame, PartialStage};
use crate::eval::vars::msgpack_type_list_is;
use crate::global_cell::GlobalCell;
use crate::mbyte::{char_at, char_len, utf_char2len, utf_printable};
use crate::message::emsg;
use crate::message_fmt::{emsg_text, msg_bytes, msg_cstr, to_message};
use crate::os::cshim::gettext;
use crate::tr_c;
use crate::tr_plural;
use crate::types::{
    Failed, IOSIZE, List, MessagePackType, TypVal, VAR_DICT, VAR_FUNC, VAR_LIST, VAR_STRING,
};

// The sinks carved out of this module's `typval_encode.c.h` instantiations.
mod json;
use self::json::encode_vim_to_json;
mod msgpack;
pub use self::msgpack::*;
mod text;
use self::text::{encode_vim_to_echo, encode_vim_to_string};

pub const kMPString: MessagePackType = 4;

/// The UTF-16 surrogate range, which a JSON `\u` escape has to spell a
/// character above the BMP with — and which a *string* may not contain.
pub const SURROGATE_HI_START: c_int = 0xd800;
pub const SURROGATE_HI_END: c_int = 0xdbff;
pub const SURROGATE_LO_START: c_int = 0xdc00;
pub const SURROGATE_LO_END: c_int = 0xdfff;
pub const SURROGATE_FIRST_CHAR: c_int = 0x10000;

/// How `string()` spells a `v:false`/`v:true`, by `BoolVarValue`.
pub(crate) const BOOL_VAR_NAMES: [&CStr; 2] = [c"v:false", c"v:true"];
/// How `string()` spells a `v:null`, by `SpecialVarValue`.
pub(crate) const SPECIAL_VAR_NAMES: [&CStr; 1] = [c"v:null"];

/// Set once a `string()`/`echo` dump has reported a self-reference, so the
/// user is told once rather than once per cycle.
static did_echo_string_emsg: GlobalCell<bool> = GlobalCell::new(false);

/// Report a self-referencing container, once per dump: a cycle usually shows
/// up many times over.
pub(crate) fn report_self_reference() {
    if !did_echo_string_emsg.get() {
        did_echo_string_emsg.set(true);
        emsg(gettext(
            c"E724: unable to correctly dump variable with self-referencing container",
        ));
    }
}

/// Append `n` in decimal: `%ld`, `%lu`, `%d` and `%zu` alike, without
/// going through a formatter for the commonest thing an encoder writes.
pub(crate) fn push_decimal(out: &mut Vec<u8>, n: impl Into<i128>) {
    let n: i128 = n.into();
    if n < 0 {
        out.push(b'-');
    }
    let mut digits = [0u8; 40];
    let mut at = digits.len();
    let mut rest = n.unsigned_abs();
    loop {
        at -= 1;
        digits[at] = b'0' + u8::try_from(rest % 10).expect("a decimal digit");
        rest /= 10;
        if rest == 0 {
            break;
        }
    }
    out.extend_from_slice(&digits[at..]);
}

/// Append `flt` as C's `%g` spells it.
pub(crate) fn push_float_g(out: &mut Vec<u8>, flt: crate::types::Float) {
    let mut numbuf = [0u8; crate::eval::typval::NUMBUFLEN as usize];
    let len = crate::strings::format_float_g(flt, &mut numbuf);
    out.extend_from_slice(&numbuf[..len]);
}

/// Append `byte` as two uppercase hexadecimal digits: `%02X`.
pub(crate) fn push_hex_byte(out: &mut Vec<u8>, byte: u8) {
    out.push(XDIGITS[usize::from(byte >> 4)]);
    out.push(XDIGITS[usize::from(byte & 0xf)]);
}

/// The bytes of the string `l[at]` holds; empty for a NULL string or no such
/// item, which is an empty line.
#[inline(always)]
fn item_bytes(l: &List, at: usize) -> &[u8] {
    l.items()
        .get(at)
        .and_then(|li| li.li_tv.string_ref())
        .map_or(&[], ThinCString::as_bytes)
}

/// Store a line the way a `readfile()`-style list does: NUL bytes become
/// newlines, because a Vimscript string can hold the former and not the
/// latter.
fn store_nuls_as_newlines(line: &mut [u8]) {
    for byte in line {
        if *byte == 0 {
            *byte = b'\n';
        }
    }
}

/// `line` as a fresh NUL-terminated allocation the list takes over.
fn own_line(line: &[u8]) -> ThinCString {
    let mut copied = line.to_vec();
    store_nuls_as_newlines(&mut copied);
    ThinCString::from_vec(copied)
}

/// Write `bytes` to a `readfile()`-style list.
///
/// Each newline in `bytes` starts a new item; whatever came before the first
/// one continues the item already there.
pub fn encode_list_write(list: &mut List, bytes: &[u8]) {
    if bytes.is_empty() {
        return;
    }
    let len = bytes.len();

    /// The index just past the next newline, and the line before it.
    fn split(bytes: &[u8], from: usize) -> (&[u8], usize) {
        let rest = &bytes[from..];
        let end = rest.iter().position(|&b| b == b'\n').unwrap_or(rest.len());
        (&rest[..end], from + end + 1)
    }

    let mut at = 0;
    if let Some(last) = list.lv_items.last_mut() {
        // Continue the last item, unless the write starts with a newline.
        let (line, next) = split(bytes, 0);
        if !line.is_empty() {
            let mut tail = line.to_vec();
            store_nuls_as_newlines(&mut tail);
            let tv = &mut last.li_tv;
            match tv.string_mut() {
                Some(text) => text.push_bytes(&tail),
                None => tv.write_string(Some(ThinCString::from_vec(tail))),
            }
        }
        at = next;
    }
    while at < len {
        let (line, next) = split(bytes, at);
        let owned = (!line.is_empty()).then(|| own_line(line));
        list.push(TypVal::string(owned));
        at = next;
    }
    if at == len {
        // The write ended on a newline, so it opened one more empty item.
        list.push(TypVal::string(None));
    }
}

/// Report a failed dump, naming the path down to the value that failed.
///
/// `msg` must carry exactly two `%s`: the object being dumped, then the path
/// — "key foo, index 2, key bar" — which this builds out of the walk's stack.
/// Always answers [`Flow::Fail`], because that is all its callers do with it.
pub(crate) fn conv_error(msg: &'static CStr, path: &ConvPath<'_, '_>) -> Flow {
    let mut msg_ga = Vec::<u8>::new();
    // Upstream formats each part in the shared `IObuff`, which cuts it at
    // `IOSIZE - 1` bytes; `to_message` cuts where that did.
    let mut append = |part: String| {
        msg_ga.extend_from_slice(to_message(part, IOSIZE as usize).as_bytes());
    };

    for (i, frame) in path.stack.iter().enumerate() {
        if i != 0 {
            append(", ".to_owned());
        }
        match *frame {
            Frame::Dict { dict, slot, .. } => {
                // The key most recently handed out: the slot before the one
                // the walk will look at next.
                let key = dict
                    .item_at(slot.saturating_sub(1))
                    .map_or(&[][..], |di| di.key());
                let key = tv2string_bytes(&TypVal::string_from(key));
                append(crate::tr!("key {}", msg_bytes(&key)));
            }
            Frame::List { list, at } | Frame::Pairs { list, at } => {
                let items = list.items();
                // The item most recently handed out: one back from the
                // cursor, or the last one once the walk has run off the end.
                let cur = at
                    .checked_sub(1)
                    .map(|back| back.min(items.len().saturating_sub(1)));
                let idx = c_int::try_from(cur.unwrap_or(0)).unwrap_or(c_int::MAX);
                let pairs = matches!(frame, Frame::Pairs { .. });
                let pair_key = cur.filter(|_| pairs).and_then(|at| {
                    let value = &items[at].li_tv;
                    if value.v_type() != VAR_LIST && list_len(value.list_ref()) <= 0 {
                        return None;
                    }
                    // A special map's item is a [key, value] pair, so the
                    // path can name the key rather than the index.
                    let key_tv = &value.list_ref()?.items().first()?.li_tv;
                    Some(tv2echo_bytes(key_tv))
                });
                append(match pair_key {
                    None => crate::tr!("index {}", idx),
                    Some(key) => {
                        crate::tr!("key {} at index {} from special map", msg_bytes(&key), idx)
                    }
                });
            }
            Frame::Partial { stage, .. } => {
                append(match stage {
                    // The walk pushes a partial already past its arguments.
                    PartialStage::Args => unreachable!("a partial frame past its arguments"),
                    PartialStage::Self_ => crate::tr!("partial"),
                    PartialStage::End => crate::tr!("partial self dictionary"),
                });
            }
            Frame::PartialArgs { at, .. } => {
                let idx = c_int::try_from(at).unwrap_or(c_int::MAX) - 1;
                append(crate::tr!("argument {}", idx));
            }
        }
    }

    let where_0 = if path.stack.is_empty() {
        crate::tr!("itself")
    } else {
        format!("{}", msg_bytes(&msg_ga))
    };
    emsg_text(tr_plural!(gettext(msg), msg_cstr(path.objname), where_0));
    Flow::Fail
}

/// Convert a `readfile()`-style list to one buffer: the items joined with
/// newlines, each stored newline turned back into the NUL it stood for.
/// Not NUL-terminated: a special string's bytes may hold NULs.
///
/// `None` when any item is not a string. A NULL list is an empty one.
pub fn encode_vim_list_to_buf(list: Option<&List>) -> Option<Vec<u8>> {
    let items = list.map_or(&[][..], List::items);
    let mut len = 0;
    for li in items {
        if li.li_tv.v_type() != VAR_STRING {
            return None;
        }
        // One separator per item, so the total is one too many.
        len += 1 + li.li_tv.string_bytes().len();
    }
    let mut buf = Vec::with_capacity(len.saturating_sub(1));
    for (at, li) in items.iter().enumerate() {
        if at > 0 {
            buf.push(b'\n');
        }
        buf.extend(
            li.li_tv
                .string_bytes()
                .iter()
                .map(|&ch| if ch == b'\n' { 0 } else { ch }),
        );
    }
    Some(buf)
}

/// Which of the two sides of [`ListReader::read`] ran out first.
///
/// The C answered `OK` for the list and `NOTDONE` for the buffer, and a
/// caller that read `OK` as "it worked" would loop for ever on a list too
/// long for one buffer -- which is why this is a value and not the `Ok`
/// half of the `Result`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ListRead {
    /// The list ran out: the buffer holds the last of it.
    Drained,
    /// The buffer ran out and the list has more. The C's `NOTDONE`.
    More,
}

/// A position in a `readfile()`-style list being read back as bytes:
/// upstream's `ListReaderState`.
pub struct ListReader<'a> {
    list: &'a List,
    /// The item being read.
    at: usize,
    /// How far into it.
    offset: usize,
    /// Its length.
    li_length: usize,
}

impl<'a> ListReader<'a> {
    /// Start reading `list` from its first item, which must hold a string
    /// or nothing.
    pub fn new(list: &'a List) -> Self {
        ListReader {
            list,
            at: 0,
            offset: 0,
            li_length: item_bytes(list, 0).len(),
        }
    }

    /// Read bytes into `out`, the stored newlines turning back into NULs on
    /// the way -- which is what [`encode_list_write`] wrote them for.
    ///
    /// Answers which of the two ran out and how many bytes were written, or
    /// [`Failed`] (with the count so far) on an item that is not a string.
    pub fn read(&mut self, out: &mut [u8]) -> (Result<ListRead, Failed>, usize) {
        let nbuf = out.len();
        let mut p = 0;
        while p < nbuf {
            let text = item_bytes(self.list, self.at);
            while self.offset < self.li_length && p < nbuf {
                let ch = text[self.offset];
                self.offset += 1;
                out[p] = if ch == b'\n' { 0 } else { ch };
                p += 1;
            }
            if p < nbuf {
                self.at += 1;
                let Some(item) = self.list.items().get(self.at) else {
                    return (Ok(ListRead::Drained), p);
                };
                out[p] = b'\n';
                p += 1;
                if item.li_tv.v_type() != VAR_STRING {
                    return (Err(Failed), p);
                }
                self.offset = 0;
                self.li_length = item_bytes(self.list, self.at).len();
            }
        }
        let more = self.at + 1 < self.list.items().len();
        if self.offset < self.li_length || more {
            (Ok(ListRead::More), nbuf)
        } else {
            (Ok(ListRead::Drained), nbuf)
        }
    }
}

const E474_BAD_UTF8: &CStr =
    c"E474: String \"%.*s\" contains byte that does not start any UTF-8 character";
const E474_SURROGATE: &CStr =
    c"E474: UTF-8 string contains code point which belongs to a surrogate pair: %.*s";

/// The hexadecimal digits a `\uNNNN` escape is spelled with.
const XDIGITS: &[u8; 16] = b"0123456789ABCDEF";

/// Upstream's `escapes[]`: the two-character escape for every character that
/// has one, indexed by the character itself.  A zero first byte means none.
static JSON_ESCAPES: [[u8; 2]; 0x5d] = {
    let mut table = [[0u8; 2]; 0x5d];
    table[8] = *b"\\b";
    table[9] = *b"\\t";
    table[10] = *b"\\n";
    table[12] = *b"\\f";
    table[13] = *b"\\r";
    table[b'"' as usize] = *b"\\\"";
    table[b'\\' as usize] = *b"\\\\";
    table
};

/// The two-character escape JSON spells `ch` with, if it has one.
#[inline(always)]
fn json_escape_of(ch: c_int) -> Option<&'static [u8; 2]> {
    let escape = JSON_ESCAPES.get(usize::try_from(ch).ok()?)?;
    (escape[0] != 0).then_some(escape)
}

/// Upstream's `ENCODE_RAW`: may `ch` go into the output as itself?
///
/// Everything else becomes `\uNNNN`, so that a JSON value stays displayable
/// outside Neovim.  0x7F is caught by `utf_printable`, not by the range.
#[inline(always)]
fn json_encode_raw(ch: c_int) -> bool {
    ch >= 0x20 && utf_printable(ch)
}

/// `\uNNNN` for a code unit.
#[inline(always)]
fn json_unicode_escape(unit: c_int) -> [u8; 6] {
    let digit = |shift: u32| XDIGITS[usize::try_from((unit >> (4 * shift)) & 0xf).unwrap_or(0)];
    [b'\\', b'u', digit(3), digit(2), digit(1), digit(0)]
}

/// The UTF-16 surrogate pair for a character above the BMP.
///
/// The low half is upstream's: it counts up from `SURROGATE_LO_END`, not from
/// `SURROGATE_LO_START`, so `U+10000` would come out as `𐏿` rather
/// than `𐀀`.  Kept as it is because the arm is **unreachable**:
/// reaching it needs a character above the BMP that `utf_printable` refuses,
/// and its table stops at `U+FFFF`.  Should that table ever grow, this is
/// where to look.
#[inline(always)]
fn json_surrogate_pair(ch: c_int) -> (c_int, c_int) {
    let tmp = ch - SURROGATE_FIRST_CHAR;
    (
        SURROGATE_HI_START + ((tmp >> 10) & 0x3ff),
        SURROGATE_LO_END + (tmp & 0x3ff),
    )
}

/// `semsg(_(msg), (int)tail.len(), tail)` — the two `%.*s` refusals below.
fn err_tail(msg: &'static CStr, tail: &[u8]) {
    let len = c_int::try_from(tail.len()).unwrap_or(c_int::MAX);
    emsg_text(tr_c!(msg, len, msg_bytes(tail)));
}

/// How long the escaped form of `text` will be, or `None` once the refusal
/// has been reported.
///
/// This is upstream's first pass: the one that decides whether the string can
/// be JSON at all.
#[inline(always)]
fn json_escaped_len(text: &[u8]) -> Option<usize> {
    let mut str_len = 0;
    let mut i = 0;
    while i < text.len() {
        let ch = char_at(&text[i..]);
        let shift = if ch == 0 { 1 } else { char_len(&text[i..]) };
        debug_assert!(shift > 0, "shift > 0");
        i += shift;
        if json_escape_of(ch).is_some() {
            str_len += 2;
        } else if ch > 0x7f && shift == 1 {
            err_tail(E474_BAD_UTF8, &text[i - shift..]);
            return None;
        } else if (SURROGATE_HI_START..=SURROGATE_HI_END).contains(&ch)
            || (SURROGATE_LO_START..=SURROGATE_LO_END).contains(&ch)
        {
            err_tail(E474_SURROGATE, &text[i - shift..]);
            return None;
        } else if json_encode_raw(ch) {
            str_len += shift;
        } else {
            // Six bytes per `\uNNNN`, and twice that for a surrogate pair.
            str_len += 6 * (1 + usize::from(ch >= SURROGATE_FIRST_CHAR));
        }
    }
    Some(str_len)
}

/// Convert a string to a JSON string literal, quotes included.
///
/// Two passes, exactly as upstream: the first sizes the result and is where
/// the refusals happen, the second writes it.  A NULL string is `""`.
///
/// Upstream measures each character with the pointer forms, which read as
/// many bytes as the lead byte promises -- past the end of a special
/// `{'_TYPE': string}` value, whose buffer is sized exactly. The slice ends
/// where the value does, which is the answer a NUL-terminated `VAR_STRING`
/// always got.
#[inline(always)]
pub(crate) fn convert_to_json_string(gap: &mut Vec<u8>, text: &[u8]) -> Result<(), Failed> {
    let Some(str_len) = json_escaped_len(text) else {
        return Err(Failed);
    };
    gap.push(b'"');
    gap.reserve(str_len);
    let mut i = 0;
    while i < text.len() {
        let ch = char_at(&text[i..]);
        // The write pass measures the *character*, not the bytes; the two
        // agree except at a NUL, which is one byte and not one character.
        let shift = if ch == 0 {
            1
        } else {
            usize::try_from(utf_char2len(ch)).unwrap_or(1)
        };
        debug_assert!(shift > 0, "shift > 0");
        debug_assert!(
            ch == 0 || shift == char_len(&text[i..]),
            "ch == 0 || shift == ((size_t)utf_ptr2len(utf_buf + i))"
        );
        if let Some(escape) = json_escape_of(ch) {
            gap.extend_from_slice(escape);
        } else if json_encode_raw(ch) {
            gap.extend_from_slice(&text[i..i + shift]);
        } else if ch < SURROGATE_FIRST_CHAR {
            gap.extend_from_slice(&json_unicode_escape(ch));
        } else {
            let (hi, lo) = json_surrogate_pair(ch);
            gap.extend_from_slice(&json_unicode_escape(hi));
            gap.extend_from_slice(&json_unicode_escape(lo));
        }
        i += shift;
    }
    gap.push(b'"');
    Ok(())
}

/// May `tv` be a key in `json_encode()`'s output?
///
/// A plain string may.  So may a `{'_TYPE': v:msgpack_types.string, '_VAL':
/// [...]}` special dictionary, provided every part of its `_VAL` is a string
/// — that is how a key holding a NUL is spelled.
pub fn encode_check_json_key(tv: &TypVal) -> bool {
    if tv.v_type() == VAR_STRING {
        return true;
    }
    if tv.v_type() != VAR_DICT {
        return false;
    }
    let Some(spdict) = tv.dict_ref() else {
        return false;
    };
    if spdict.dv_hashtab.ht_used != 2 {
        return false;
    }
    let (Some(type_di), Some(val_di)) = (
        dict_find(Some(spdict), b"_TYPE"),
        dict_find(Some(spdict), b"_VAL"),
    ) else {
        return false;
    };
    let type_tv = &type_di.di_tv;
    if type_tv.v_type() != VAR_LIST
        || !type_tv
            .list_ref()
            .is_some_and(|list| msgpack_type_list_is(kMPString, list))
    {
        return false;
    }
    let val_tv = &val_di.di_tv;
    if val_tv.v_type() != VAR_LIST {
        return false;
    }
    val_tv
        .list_ref()
        .map_or(&[][..], List::items)
        .iter()
        .all(|li| li.li_tv.v_type() == VAR_STRING)
}

/// The string representation of `tv`, quoted so `eval()` can read it back.
pub(crate) fn tv2string_bytes(tv: &TypVal) -> Vec<u8> {
    let mut ga = Vec::<u8>::new();
    let evs_ret = encode_vim_to_string(&mut ga, tv, c"encode_tv2string() argument");
    debug_assert!(evs_ret);
    did_echo_string_emsg.set(false);
    ga
}

/// `tv2string_bytes` as an owned C string.
pub fn encode_tv2string(tv: &TypVal) -> ThinCString {
    ThinCString::from_vec(tv2string_bytes(tv))
}

/// The string representation of `tv` as `:echo` displays it — no quotes.
pub(crate) fn tv2echo_bytes(tv: &TypVal) -> Vec<u8> {
    let mut ga = Vec::<u8>::new();
    // A string or function reference echoes as its own bytes, which is
    // the whole difference between `:echo` and `string()` at the top
    // level; below it, the sink says it again.
    if tv.v_type() == VAR_STRING || tv.v_type() == VAR_FUNC {
        let text = if tv.v_type() == VAR_STRING {
            tv.string_cstr()
        } else {
            tv.callable_name()
        };
        if let Some(text) = text {
            ga.extend_from_slice(text.to_bytes());
        }
    } else {
        let eve_ret = encode_vim_to_echo(&mut ga, tv, c":echo argument");
        debug_assert!(eve_ret);
    }
    ga
}

/// `tv2echo_bytes` as an owned C string.
pub fn encode_tv2echo(tv: &TypVal) -> ThinCString {
    ThinCString::from_vec(tv2echo_bytes(tv))
}

/// `tv` as JSON, or an empty string once the refusal has been reported.
pub fn encode_tv2json(tv: &TypVal) -> ThinCString {
    let mut ga = Vec::<u8>::new();
    let evj_ret = encode_vim_to_json(&mut ga, tv, c"encode_tv2json() argument");
    if !evj_ret {
        ga.clear();
    }
    did_echo_string_emsg.set(false);
    ThinCString::from_vec(ga)
}

#[cfg(test)]
mod tests {
    //! The text sinks over the container walk, end to end: the shapes the
    //! walk's frames exist for -- nests past the inline budget, containers
    //! that reach themselves, a partial's arguments and self dictionary --
    //! with the exact text each prints.

    use super::*;
    use crate::eval::list::string_tv;
    use crate::eval::typval::{
        DictRef, ListRef, PartialRef, tv_blob_alloc, tv_dict_alloc, tv_list_alloc,
    };
    use crate::global_cell::editor_state_lock;
    use crate::memory::ThinCString;
    use crate::types::{Partial, kBoolVarTrue, kSpecialVarNull};

    /// The editor lock, with messages suppressed: there is no screen to
    /// draw them on.
    fn ready() -> (impl Drop, crate::guard::Quiet) {
        let held = editor_state_lock();
        (held, crate::guard::Suppress::output())
    }

    fn text(tv: &TypVal) -> String {
        String::from_utf8(tv2string_bytes(tv)).expect("ASCII")
    }

    fn echo(tv: &TypVal) -> String {
        String::from_utf8(tv2echo_bytes(tv)).expect("ASCII")
    }

    fn json(tv: &TypVal) -> Option<String> {
        let mut ga = Vec::new();
        let ok = encode_vim_to_json(&mut ga, tv, c"test");
        did_echo_string_emsg.set(false);
        ok.then(|| String::from_utf8(ga).expect("UTF-8"))
    }

    fn list(items: Vec<TypVal>) -> ListRef {
        let mut l = tv_list_alloc(-1);
        for tv in items {
            l.push(tv);
        }
        l
    }

    fn partial(name: &str, args: Vec<TypVal>, dict: Option<DictRef>) -> PartialRef {
        PartialRef::new(Partial {
            pt_name: Some(ThinCString::from_bytes(name.as_bytes())),
            pt_argv: args,
            pt_dict: dict,
            ..Partial::EMPTY
        })
    }

    #[test]
    fn scalars_and_flat_containers_print_as_their_literals() {
        let _serial = ready();
        let mut blob = tv_blob_alloc();
        blob.extend(&[1, 0xab]);
        let mut d = tv_dict_alloc();
        d.add_number(b"n", 1).expect("fresh key");
        let tv = TypVal::list(Some(list(vec![
            TypVal::Number(-7),
            string_tv(b"it's"),
            TypVal::Float(1.5),
            TypVal::list(Some(tv_list_alloc(0))),
            TypVal::dict(Some(tv_dict_alloc())),
            TypVal::blob(Some(blob)),
            TypVal::Special(kSpecialVarNull),
            TypVal::Bool(kBoolVarTrue),
            TypVal::dict(Some(d)),
            TypVal::string(None),
        ])));
        assert_eq!(
            text(&tv),
            "[-7, 'it''s', 1.5, [], {}, 0z01AB, v:null, v:true, {'n': 1}, '']"
        );
        assert_eq!(
            json(&tv).as_deref(),
            Some("[-7, \"it's\", 1.5, [], {}, [1, 171], null, true, {\"n\": 1}, \"\"]")
        );
    }

    /// A list that holds itself, and a dictionary that does, two levels down.
    #[test]
    fn a_container_that_reaches_itself_is_marked_not_followed() {
        let _serial = ready();
        let mut l = list(vec![TypVal::Number(1)]);
        let again = l.clone();
        l.push(TypVal::list(Some(again)));
        let tv = TypVal::list(Some(l.clone()));
        // `:echo`'s marker: `string()` and JSON say the same with an error
        // message, which wants a screen.
        assert_eq!(echo(&tv), "[1, [...@0]]");

        let mut d = tv_dict_alloc();
        let inner = list(vec![TypVal::dict(Some(d.clone()))]);
        d.add_list(b"l", Some(inner)).expect("fresh key");
        let dtv = TypVal::dict(Some(d.clone()));
        assert_eq!(echo(&dtv), "{'l': [{...@0}]}");

        // A container met twice, side by side, is not a cycle.
        let shared = list(vec![TypVal::Number(2)]);
        let twice = TypVal::list(Some(list(vec![
            TypVal::list(Some(shared.clone())),
            TypVal::list(Some(shared)),
        ])));
        assert_eq!(text(&twice), "[[2], [2]]");

        // Break the cycles, or nothing frees them.
        l.remove_range(1, 1);
        drop(d.edit().remove_key(b"l"));
    }

    /// Thirty levels: past the walk's eight inline frames and back.
    #[test]
    fn a_deep_nest_prints_every_level() {
        let _serial = ready();
        let mut tv = TypVal::Number(0);
        for _ in 0..30 {
            tv = TypVal::list(Some(list(vec![tv])));
        }
        let expected = format!("{}0{}", "[".repeat(30), "]".repeat(30));
        assert_eq!(text(&tv), expected);
        assert_eq!(json(&tv).as_deref(), Some(expected.as_str()));
    }

    /// A partial walks its arguments, then its self dictionary -- which here
    /// holds the partial: a cycle through a partial frame.
    #[test]
    fn a_partial_prints_its_arguments_and_its_dictionary() {
        let _serial = ready();
        let mut d = tv_dict_alloc();
        d.add_number(b"n", 1).expect("fresh key");
        let args = vec![
            TypVal::Number(1),
            TypVal::list(Some(list(vec![string_tv(b"x")]))),
        ];
        let pt = partial("tr", args, Some(d.clone()));
        let tv = TypVal::partial(Some(pt.clone()));
        assert_eq!(text(&tv), "function('tr', [1, ['x']], {'n': 1})");

        d.add_value(b"p", TypVal::partial(Some(pt)))
            .expect("fresh key");
        // The marker counts frames from the bottom: the partial's own frame
        // is 0, its self dictionary's 1.
        assert_eq!(
            echo(&tv),
            "function('tr', [1, ['x']], {'p': function('tr', [1, ['x']], {...@1}), 'n': 1})"
        );
        drop(d.edit().remove_key(b"p"));
    }
}
