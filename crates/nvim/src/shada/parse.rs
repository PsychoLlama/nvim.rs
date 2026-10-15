//! Parsing one ShaDa entry's payload.
//!
//! `unpack` has read the entry's type, timestamp and length and handed the
//! bytes over as a [`Cursor`]; the function here for that type turns them
//! into a [`ShadaEntry`]. Anything in the payload this Nvim has no field for
//! is collected as extra data, which travels with the entry and is written
//! back out unchanged.

#![forbid(unsafe_code)]

use crate::eval::decode::decode_string;
use crate::memory::XString;
use crate::message_fmt::{emsg_text, msg_cstr_opt};
use crate::msgpack_rpc::unpacker::MPACK_OK;
use crate::tr_c;
use crate::types::{
    ApiDict, ColNr, Integer, KeyDict__shada_buflist_item, KeyDict__shada_mark,
    KeyDict__shada_register, LineNr, MotionType, VAR_TYPE_BLOB,
};
use core::ffi::{CStr, c_char};
use std::ffi::CString;
use std::rc::Rc;

use super::unpack::{Cursor, Header, Parsed};
use super::{
    DEFAULT_POS, DEFAULT_REGISTER, DEFAULT_SEARCH_PATTERN, HIST_SEARCH, ShadaBufferListItem,
    ShadaEntry, ShadaEntryData, ShadaExtra, ShadaFileMark, ShadaGlobalVar, ShadaHistoryItem,
    ShadaRegister, default_mark_name, kind,
};
use crate::eval::typval::TV_INITIAL_VALUE;

/// Report `E575` for an entry at byte `pos` the file states wrongly.
fn malformed_entry(fmt: &'static CStr, pos: u64) {
    emsg_text(tr_c!(fmt, pos));
}

/// As [`malformed_entry`], for the messages whose `%s` carries the reason
/// the keyset reader gave (or `[NULL]`, when it gave none).
fn malformed_entry_because(fmt: &'static CStr, pos: u64, why: &Option<CString>) {
    let why = msg_cstr_opt(why.as_deref());
    emsg_text(tr_c!(fmt, pos, why));
}

/// A parse failure. The entry is dropped and reported as missing.
pub(crate) struct Malformed;

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
            (ShadaEntryData::Header(ApiDict::EMPTY), 0)
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
            return Err(Malformed);
        }
        let used = item_start.len() - cursor.left();
        ShadaExtra::push(&mut extra, &item_start[..used]);
    }
    if cursor.left() != 0 {
        malformed_entry(
            c"E575: Error while reading ShaDa file: item entry at position %lu additional bytes",
            pos,
        );
        return Err(Malformed);
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
        malformed_entry_because(
            c"E575: Error while reading ShaDa file: search pattern entry at position %lu %s",
            pos,
            &why,
        );
        return Err(Malformed);
    }
    if it.pat.is_none() {
        malformed_entry(c"E575: Error while reading ShaDa file: search pattern entry at position %lu has no pattern", pos);
        return Err(Malformed);
    }
    Ok((ShadaEntryData::SearchPattern(it), 0))
}

/// A global mark, local mark, jump or change: all four are the same map.
fn parse_mark(header: &Header, cursor: &mut Cursor, extra: &mut Option<ShadaExtra>) -> Parsed2 {
    let pos = header.fpos;
    let mut it = KeyDict__shada_mark::default();
    if let Err(why) = cursor.keydict(&mut it, extra) {
        malformed_entry_because(
            c"E575: Error while reading ShaDa file: mark entry at position %lu %s",
            pos,
            &why,
        );
        return Err(Malformed);
    }

    let mut name = default_mark_name(header.kind);
    if let Some(n) = it.n {
        if header.kind == kind::JUMP || header.kind == kind::CHANGE {
            malformed_entry(c"E575: Error while reading ShaDa file: mark entry at position %lu has n key which is only valid for local and global mark entries", pos);
            return Err(Malformed);
        }
        name = n as c_char;
    }
    let mut mark = DEFAULT_POS;
    if let Some(lnum) = it.l {
        mark.lnum = lnum as LineNr;
    }
    if let Some(col) = it.c {
        mark.col = col as ColNr;
    }
    let Some(fname) = it.f.as_ref().map(|f| c_string(f.as_bytes())) else {
        malformed_entry(c"E575: Error while reading ShaDa file: mark entry at position %lu is missing file name", pos);
        return Err(Malformed);
    };
    if mark.lnum <= 0 {
        malformed_entry(c"E575: Error while reading ShaDa file: mark entry at position %lu has invalid line number", pos);
        return Err(Malformed);
    }
    if mark.col < 0 {
        malformed_entry(c"E575: Error while reading ShaDa file: mark entry at position %lu has invalid column number", pos);
        return Err(Malformed);
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
        malformed_entry_because(
            c"E575: Error while reading ShaDa file: register entry at position %lu %s",
            pos,
            &why,
        );
        return Err(Malformed);
    }
    let contents = it.rc.take().map(|rc| rc.into_vec()).unwrap_or_default();
    if contents.is_empty() {
        malformed_entry(c"E575: Error while reading ShaDa file: register entry at position %lu has rc key with missing or empty array", pos);
        return Err(Malformed);
    }
    let mut reg = ShadaRegister {
        contents,
        ..DEFAULT_REGISTER
    };
    if let Some(is_unnamed) = it.ru {
        reg.is_unnamed = is_unnamed;
    }
    if let Some(motion) = it.rt {
        reg.motion = MotionType::from(motion as u8);
    }
    if let Some(name) = it.n {
        reg.name = name as c_char;
    }
    if let Some(width) = it.rw {
        reg.width = width as usize;
    }
    Ok((ShadaEntryData::Register(reg), 0))
}

/// One history line: `[type, text]`, plus a separator for search history.
fn parse_history(pos: u64, cursor: &mut Cursor) -> Parsed2 {
    let len = cursor.array().filter(|&len| len >= 2);
    let Some(len) = len else {
        malformed_entry(c"E575: Error while reading ShaDa file: history entry at position %lu is not an array with enough elements", pos);
        return Err(Malformed);
    };
    let Some(hist_type) = cursor.integer() else {
        malformed_entry(c"E575: Error while reading ShaDa file: history entry at position %lu has wrong history type type", pos);
        return Err(Malformed);
    };
    let Some(text) = cursor.string() else {
        malformed_entry(c"E575: Error while reading ShaDa file: history entry at position %lu has wrong history string type", pos);
        return Err(Malformed);
    };
    let Ok(text) = CString::new(text).map(Rc::<CStr>::from) else {
        malformed_entry(c"E575: Error while reading ShaDa file: history entry at position %lu contains string with zero byte inside", pos);
        return Err(Malformed);
    };

    let histtype = hist_type as u8;
    let is_search = histtype == HIST_SEARCH;
    let mut sep = 0;
    if is_search {
        if len < 3 {
            malformed_entry(c"E575: Error while reading ShaDa file: search history entry at position %lu does not have separator character", pos);
            return Err(Malformed);
        }
        let Some(read) = cursor.integer() else {
            malformed_entry(c"E575: Error while reading ShaDa file: search history entry at position %lu has wrong history separator type", pos);
            return Err(Malformed);
        };
        sep = read as u8;
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
        malformed_entry(c"E575: Error while reading ShaDa file: variable entry at position %lu is not an array with enough elements", pos);
        return Err(Malformed);
    };
    let Some(name) = cursor.string() else {
        malformed_entry(c"E575: Error while reading ShaDa file: variable entry at position %lu has wrong variable name type", pos);
        return Err(Malformed);
    };
    let name = c_string(name);

    let mut is_blob = false;
    let value = if let Some(binval) = cursor.string() {
        if len > 2 {
            if cursor.integer() != Some(Integer::from(VAR_TYPE_BLOB)) {
                malformed_entry(c"E575: Error while reading ShaDa file: variable entry at position %lu has wrong variable type", pos);
                return Err(Malformed);
            }
            is_blob = true;
        }
        decode_string(binval, is_blob)
    } else {
        let mut value = TV_INITIAL_VALUE;
        if cursor.typval(&mut value) != MPACK_OK {
            malformed_entry(c"E575: Error while reading ShaDa file: variable entry at position %lu has value that cannot be converted to the Vimscript value", pos);
            return Err(Malformed);
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
        malformed_entry(c"E575: Error while reading ShaDa file: sub string entry at position %lu is not an array with enough elements", pos);
        return Err(Malformed);
    };
    let Some(sub) = cursor.string() else {
        malformed_entry(c"E575: Error while reading ShaDa file: sub string entry at position %lu has wrong sub string type", pos);
        return Err(Malformed);
    };
    Ok((ShadaEntryData::SubString(c_string(sub)), len - 1))
}

/// The buffer list: an array of maps, one per buffer, each a file name and
/// the cursor position in it.
fn parse_buffer_list(pos: u64, cursor: &mut Cursor) -> Parsed2 {
    let Some(len) = cursor.array() else {
        malformed_entry(c"E575: Error while reading ShaDa file: buffer list entry at position %lu is not an array", pos);
        return Err(Malformed);
    };
    let mut items = Vec::with_capacity(len);
    for _ in 0..len {
        let mut it = KeyDict__shada_buflist_item::default();
        let mut extra = None;
        if let Err(why) = cursor.keydict(&mut it, &mut extra) {
            malformed_entry_because(c"E575: Error while reading ShaDa file: buffer list at position %lu contains entry that %s", pos, &why);
            return Err(Malformed);
        }
        let mut at = DEFAULT_POS;
        if let Some(lnum) = it.l {
            at.lnum = lnum as LineNr;
        }
        if let Some(col) = it.c {
            at.col = col as ColNr;
        }
        let fname = it.f.as_ref().map(|f| c_string(f.as_bytes()));

        let complaint: &CStr = if at.lnum <= 0 {
            c"E575: Error while reading ShaDa file: buffer list at position %lu contains entry with invalid line number"
        } else if at.col < 0 {
            c"E575: Error while reading ShaDa file: buffer list at position %lu contains entry with invalid column number"
        } else if let Some(fname) = fname {
            items.push(ShadaBufferListItem {
                pos: at,
                fname,
                extra,
            });
            continue;
        } else {
            c"E575: Error while reading ShaDa file: buffer list at position %lu contains entry that does not have a file name"
        };
        emsg_text(tr_c!(complaint, pos));
        return Err(Malformed);
    }
    Ok((ShadaEntryData::BufferList(items), 0))
}
