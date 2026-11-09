//! Parsing one ShaDa entry's payload.
//!
//! `unpack` has read the entry's type, timestamp and length and handed the
//! bytes over as a [`Cursor`]; the function here for that type turns them
//! into a [`ShadaEntry`]. Anything in the payload this Nvim has no field for
//! is collected as extra data, which travels with the entry and is written
//! back out unchanged.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::low_byte;
use crate::api::keysets::{
    KeyDict__shada_buflist_item, KeyDict__shada_mark, KeyDict__shada_register,
};
use crate::eval::decode::decode_string;
use crate::memory::XString;
use crate::mpack::mpack_core::MPACK_OK;
use crate::narrow::{number_as_col, number_as_lnum};
use crate::types::{ApiDict, ColNr, Integer, LineNr, VAR_TYPE_BLOB};
use core::ffi::CStr;
use std::ffi::CString;
use std::rc::Rc;

use super::unpack::{Cursor, Header, Parsed};
use super::{
    DEFAULT_POS, DEFAULT_REGISTER, DEFAULT_SEARCH_PATTERN, EntryFault, HIST_SEARCH, Keyed,
    ShadaBufferListItem, ShadaEntry, ShadaEntryData, ShadaError, ShadaExtra, ShadaFileMark,
    ShadaGlobalVar, ShadaHistoryItem, ShadaRegister, default_mark_name, kind,
};
use crate::eval::typval::TV_INITIAL_VALUE;

/// A parse failure: the entry is dropped, with what to say about it --
/// `None` for a value msgpack could not step over, which upstream drops
/// without a word.
#[derive(Debug)]
pub(crate) struct Malformed(pub(crate) Option<ShadaError>);

/// The `E575` for an entry at `pos` that says something it cannot.
fn fault(fault: EntryFault, pos: u64) -> Malformed {
    Malformed(Some(ShadaError::Entry(fault, pos)))
}

/// The `E575` for an entry at `pos` whose map did not read as its keyset.
fn keyed(entry: Keyed, pos: u64, why: Option<CString>) -> Malformed {
    Malformed(Some(ShadaError::Keyed(Box::new((entry, pos, why)))))
}

/// A file name, mark name or variable name as C reads it: up to its first
/// NUL.
fn c_string(bytes: &[u8]) -> XString {
    XString::from_bytes(bytes.split(|&b| b == 0).next().unwrap_or(&[]))
}

/// Parse the payload of an entry whose type this Nvim knows.
pub(crate) fn parse_known(header: &Header, cursor: &mut Cursor) -> Result<ShadaEntry, Malformed> {
    // Map keys this Nvim does not know, and trailing array elements, are
    // collected here and written back out with the entry.
    let mut extra: Option<ShadaExtra> = None;
    let pos = header.fpos;

    // Each parser answers the payload, and how many elements of an array
    // entry are left for `extra` once the ones this Nvim understands have
    // been taken.
    let (data, trailing) = match header.kind {
        kind::HEADER => {
            // The header is written for the benefit of anyone reading the
            // file by hand; Nvim has never read it back.
            (ShadaEntryData::Header(ApiDict::new()), 0)
        }
        kind::SEARCH_PATTERN => parse_search_pattern(pos, cursor, &mut extra)?,
        kind::CHANGE | kind::JUMP | kind::GLOBAL_MARK | kind::LOCAL_MARK => {
            parse_mark(header, cursor, &mut extra)?
        }
        kind::REGISTER => parse_register(pos, cursor, &mut extra)?,
        kind::HISTORY_ENTRY => parse_history(pos, cursor)?,
        kind::VARIABLE => parse_variable(pos, cursor)?,
        kind::SUB_STRING => parse_sub_string(pos, cursor)?,
        kind::BUFFER_LIST => parse_buffer_list(pos, cursor)?,
        other => unreachable!("shada: entry type {other} is not read here"),
    };

    for _ in 0..trailing {
        let item_start = cursor.rest();
        if cursor.skip() != Parsed::Ok {
            return Err(Malformed(None));
        }
        let used = item_start.len() - cursor.left();
        ShadaExtra::push(&mut extra, &item_start[..used]);
    }
    if cursor.left() != 0 {
        return Err(fault(EntryFault::ItemExtraBytes, pos));
    }
    Ok(ShadaEntry {
        timestamp: header.timestamp,
        data,
        extra,
    })
}

/// What one parser answers: the payload, and how many array elements are
/// left over for the extra data.
type Parsed2 = Result<(ShadaEntryData, usize), Malformed>;

/// The last search pattern, as a map of the two-letter `s*` keys.
fn parse_search_pattern(pos: u64, cursor: &mut Cursor, extra: &mut Option<ShadaExtra>) -> Parsed2 {
    let mut it = DEFAULT_SEARCH_PATTERN;
    if let Err(why) = cursor.keydict(&mut it, extra) {
        return Err(keyed(Keyed::SearchPattern, pos, why));
    }
    if it.pat.is_none() {
        return Err(fault(EntryFault::PatternWithoutPattern, pos));
    }
    Ok((ShadaEntryData::SearchPattern(it), 0))
}

/// A global mark, local mark, jump or change: all four are the same map.
fn parse_mark(header: &Header, cursor: &mut Cursor, extra: &mut Option<ShadaExtra>) -> Parsed2 {
    let pos = header.fpos;
    let mut it = KeyDict__shada_mark::default();
    if let Err(why) = cursor.keydict(&mut it, extra) {
        return Err(keyed(Keyed::Mark, pos, why));
    }

    let mut name = default_mark_name(header.kind);
    if let Some(n) = it.n {
        if header.kind == kind::JUMP || header.kind == kind::CHANGE {
            return Err(fault(EntryFault::MarkWithN, pos));
        }
        name = low_byte(n).cast_signed();
    }
    let mut mark = DEFAULT_POS;
    if let Some(lnum) = it.l {
        mark.lnum = number_as_lnum(lnum);
    }
    if let Some(col) = it.c {
        mark.col = number_as_col(col);
    }
    let Some(fname) = it.f.as_ref().map(|f| c_string(f.as_bytes())) else {
        return Err(fault(EntryFault::MarkWithoutFile, pos));
    };
    if mark.lnum <= LineNr::ZERO {
        return Err(fault(EntryFault::MarkLine, pos));
    }
    if mark.col < ColNr::ZERO {
        return Err(fault(EntryFault::MarkColumn, pos));
    }
    let mark = ShadaFileMark { name, mark, fname };
    let data = match header.kind {
        kind::GLOBAL_MARK => ShadaEntryData::GlobalMark(mark),
        kind::JUMP => ShadaEntryData::Jump(mark),
        kind::LOCAL_MARK => ShadaEntryData::LocalMark(mark),
        _ => ShadaEntryData::Change(mark),
    };
    Ok((data, 0))
}

/// One register: a name, a motion type, a width and the lines in it.
fn parse_register(pos: u64, cursor: &mut Cursor, extra: &mut Option<ShadaExtra>) -> Parsed2 {
    let mut it = KeyDict__shada_register::default();
    if let Err(why) = cursor.keydict(&mut it, extra) {
        return Err(keyed(Keyed::Register, pos, why));
    }
    let contents = it.rc.take().map(|rc| rc.into_vec()).unwrap_or_default();
    if contents.is_empty() {
        return Err(fault(EntryFault::RegisterWithoutContents, pos));
    }
    let mut reg = ShadaRegister {
        contents,
        ..DEFAULT_REGISTER
    };
    if let Some(is_unnamed) = it.ru {
        reg.is_unnamed = is_unnamed;
    }
    if let Some(motion) = it.rt {
        reg.motion = low_byte(motion);
    }
    if let Some(name) = it.n {
        reg.name = low_byte(name).cast_signed();
    }
    if let Some(width) = it.rw {
        // A negative width wraps, as C's conversion to `size_t` does.
        reg.width = usize::try_from(width.cast_unsigned()).unwrap_or(usize::MAX);
    }
    Ok((ShadaEntryData::Register(reg), 0))
}

/// One history line: `[type, text]`, plus a separator for search history.
fn parse_history(pos: u64, cursor: &mut Cursor) -> Parsed2 {
    let len = cursor.array().filter(|&len| len >= 2);
    let Some(len) = len else {
        return Err(fault(EntryFault::HistoryNotArray, pos));
    };
    let Some(hist_type) = cursor.integer() else {
        return Err(fault(EntryFault::HistoryType, pos));
    };
    let Some(text) = cursor.string() else {
        return Err(fault(EntryFault::HistoryString, pos));
    };
    let Ok(text) = CString::new(text).map(Rc::<CStr>::from) else {
        return Err(fault(EntryFault::HistoryNul, pos));
    };

    let histtype = low_byte(hist_type);
    let is_search = histtype == HIST_SEARCH;
    let mut sep = 0;
    if is_search {
        if len < 3 {
            return Err(fault(EntryFault::SearchHistoryWithoutSeparator, pos));
        }
        let Some(read) = cursor.integer() else {
            return Err(fault(EntryFault::SearchHistorySeparatorType, pos));
        };
        sep = low_byte(read);
    }
    let item = ShadaHistoryItem {
        histtype,
        text,
        sep,
    };
    Ok((
        ShadaEntryData::HistoryEntry(item),
        len - 2 - usize::from(is_search),
    ))
}

/// One global variable: `[name, value]`, with a trailing type tag when the
/// value is a Blob (which is otherwise indistinguishable from a String).
fn parse_variable(pos: u64, cursor: &mut Cursor) -> Parsed2 {
    let Some(len) = cursor.array().filter(|&len| len >= 2) else {
        return Err(fault(EntryFault::VariableNotArray, pos));
    };
    let Some(name) = cursor.string() else {
        return Err(fault(EntryFault::VariableNameType, pos));
    };
    let name = c_string(name);

    let mut is_blob = false;
    let value = if let Some(binval) = cursor.string() {
        if len > 2 {
            if cursor.integer() != Some(Integer::from(VAR_TYPE_BLOB)) {
                return Err(fault(EntryFault::VariableType, pos));
            }
            is_blob = true;
        }
        decode_string(binval, is_blob)
    } else {
        let mut value = TV_INITIAL_VALUE;
        if cursor.typval(&mut value) != MPACK_OK {
            return Err(fault(EntryFault::VariableValue, pos));
        }
        value
    };
    let var = ShadaGlobalVar { name, value };
    Ok((
        ShadaEntryData::Variable(var),
        len - 2 - usize::from(is_blob),
    ))
}

/// The last `:substitute` replacement string: a one-element array.
fn parse_sub_string(pos: u64, cursor: &mut Cursor) -> Parsed2 {
    let Some(len) = cursor.array().filter(|&len| len >= 1) else {
        return Err(fault(EntryFault::SubNotArray, pos));
    };
    let Some(sub) = cursor.string() else {
        return Err(fault(EntryFault::SubType, pos));
    };
    Ok((ShadaEntryData::SubString(c_string(sub)), len - 1))
}

/// The buffer list: an array of maps, one per buffer, each a file name and
/// the cursor position in it.
fn parse_buffer_list(pos: u64, cursor: &mut Cursor) -> Parsed2 {
    let Some(len) = cursor.array() else {
        return Err(fault(EntryFault::BufferListNotArray, pos));
    };
    let mut items = Vec::with_capacity(len);
    for _ in 0..len {
        let mut it = KeyDict__shada_buflist_item::default();
        let mut extra = None;
        if let Err(why) = cursor.keydict(&mut it, &mut extra) {
            return Err(keyed(Keyed::BufferList, pos, why));
        }
        let mut at = DEFAULT_POS;
        if let Some(lnum) = it.l {
            at.lnum = number_as_lnum(lnum);
        }
        if let Some(col) = it.c {
            at.col = number_as_col(col);
        }
        let fname = it.f.as_ref().map(|f| c_string(f.as_bytes()));

        let complaint = if at.lnum <= LineNr::ZERO {
            EntryFault::BufferListLine
        } else if at.col < ColNr::ZERO {
            EntryFault::BufferListColumn
        } else if let Some(fname) = fname {
            items.push(ShadaBufferListItem {
                pos: at,
                fname,
                extra,
            });
            continue;
        } else {
            EntryFault::BufferListWithoutFile
        };
        return Err(fault(complaint, pos));
    }
    Ok((ShadaEntryData::BufferList(items), 0))
}
