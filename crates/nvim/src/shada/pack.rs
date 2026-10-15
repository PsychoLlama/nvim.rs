//! Turning a [`ShadaEntry`] into msgpack.
//!
//! One entry becomes four things in a row: its type, its timestamp, the byte
//! length of its payload, and the payload — see `unpack` for the reading
//! side. The payload is packed first, because the length has to be written
//! before it.
//!
//! Every field whose value equals the default for its entry type is left
//! out; both sides agree on the defaults, so a reader puts them back. That
//! is what keeps a ShaDa file small, and it is why the map sizes below are
//! counted before anything is written.
//!
//! The encodings are the API packer's (`msgpack_rpc::packer::format`), not
//! the shortest msgpack allows: an unsigned value goes eight bytes wide
//! above 2^28, and a string or binary of exactly 255 or 65535 bytes steps up
//! a width. Both are on the wire of every ShaDa file ever written, so the
//! writer keeps them byte for byte.

#![forbid(unsafe_code)]

use crate::eval::encode::encode_vim_to_msgpack;
use crate::message_fmt::msg_cstr;
use crate::msgpack_rpc::packer::{format, pack_to_bytes};
use crate::semsg;
use crate::types::{ApiDict, Integer, KeyDict__shada_search_pat, Object, VAR_BLOB, VAR_TYPE_BLOB};
use std::ffi::CString;

use super::{
    DEFAULT_POS, DEFAULT_REGISTER, DEFAULT_SEARCH_PATTERN, HIST_SEARCH, ShadaBufferListItem,
    ShadaEntry, ShadaEntryData, ShadaExtra, ShadaFileMark, ShadaGlobalVar, ShadaHistoryItem,
    ShadaRegister, WriteStatus, default_mark_name,
};

/// A msgpack encoder into a byte vector.
#[derive(Default)]
pub(crate) struct Packer {
    out: Vec<u8>,
}

impl Packer {
    /// What has been packed.
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.out
    }

    /// Take what has been packed, leaving the packer empty.
    pub(crate) fn take(&mut self) -> Vec<u8> {
        core::mem::take(&mut self.out)
    }

    fn uint(&mut self, value: u32) {
        self.out.extend_from_slice(format::uint(value).bytes());
    }

    fn uint64(&mut self, value: u64) {
        self.out.extend_from_slice(format::uint64(value).bytes());
    }

    fn integer(&mut self, value: Integer) {
        self.out.extend_from_slice(format::integer(value).bytes());
    }

    fn boolean(&mut self, value: bool) {
        self.out.extend_from_slice(format::boolean(value).bytes());
    }

    fn array(&mut self, len: u32) {
        self.out
            .extend_from_slice(format::array_header(len).bytes());
    }

    fn map(&mut self, len: u32) {
        self.out.extend_from_slice(format::map_header(len).bytes());
    }

    /// A map key. ShaDa spells them as one- or two-letter codes.
    fn key(&mut self, name: &[u8]) {
        self.str(name);
    }

    fn str(&mut self, bytes: &[u8]) {
        let header = format::str_header(bytes.len()).expect("string too long for msgpack");
        self.out.extend_from_slice(header.bytes());
        self.out.extend_from_slice(bytes);
    }

    fn bin(&mut self, bytes: &[u8]) {
        let header = format::bin_header(bytes.len()).expect("blob too long for msgpack");
        self.out.extend_from_slice(header.bytes());
        self.out.extend_from_slice(bytes);
    }

    fn raw(&mut self, bytes: &[u8]) {
        self.out.extend_from_slice(bytes);
    }

    /// The keys or elements an entry carries that this Nvim has no field
    /// for, exactly as they were read.
    fn extra(&mut self, extra: &Option<ShadaExtra>) {
        if let Some(extra) = extra {
            self.raw(extra.bytes());
        }
    }
}

/// How many map keys or array elements an entry carries that this Nvim has
/// no field for.
fn extra_len(extra: &Option<ShadaExtra>) -> u32 {
    extra.as_ref().map_or(0, ShadaExtra::items)
}

/// 1 when a field differs from its default and so has to be written.
fn written<T: PartialEq>(value: T, default: T) -> u32 {
    u32::from(value != default)
}

/// A count of items as msgpack carries it.
fn count(len: usize) -> u32 {
    u32::try_from(len).expect("container too long for msgpack")
}

/// Append one entry to `out`.
///
/// `max_kbyte`, if non-zero, drops an entry whose payload comes out longer
/// than that many kilobytes — quietly, since the file is still valid
/// without it.
pub(crate) fn pack_entry(out: &mut Packer, entry: &ShadaEntry, max_kbyte: usize) -> WriteStatus {
    let mut payload = Packer::default();
    let extra = &entry.extra;
    match &entry.data {
        ShadaEntryData::Missing => unreachable!("shada: a missing entry is never written"),
        ShadaEntryData::Unknown(item) => payload.raw(&item.contents),
        ShadaEntryData::Header(header) => pack_header(header, &mut payload),
        ShadaEntryData::HistoryEntry(history) => pack_history(history, extra, &mut payload),
        ShadaEntryData::Variable(var) => {
            if let Err(ignorable) = pack_variable(var, extra, &mut payload) {
                return ignorable;
            }
        }
        ShadaEntryData::SubString(sub) => {
            payload.array(1 + extra_len(extra));
            payload.bin(sub.as_cstr().to_bytes());
            payload.extra(extra);
        }
        ShadaEntryData::SearchPattern(pattern) => {
            pack_search_pattern(pattern, extra, &mut payload);
        }
        ShadaEntryData::GlobalMark(mark)
        | ShadaEntryData::LocalMark(mark)
        | ShadaEntryData::Jump(mark)
        | ShadaEntryData::Change(mark) => pack_mark(entry.kind(), mark, extra, &mut payload),
        ShadaEntryData::Register(reg) => pack_register(reg, extra, &mut payload),
        ShadaEntryData::BufferList(list) => pack_buffer_list(list, &mut payload),
    }

    let packed = payload.bytes();
    if max_kbyte != 0 && packed.len() > max_kbyte * 1024 {
        return WriteStatus::Successful; // too big to keep; not an error
    }
    out.uint64(entry.kind());
    out.uint64(entry.timestamp);
    if !packed.is_empty() {
        out.uint64(packed.len() as u64);
        out.raw(packed);
    }
    WriteStatus::Successful
}

/// The file header: whatever the write chose to record about the Nvim that
/// wrote it. Nvim has never read it back — it is there for anyone looking at
/// the file by hand.
fn pack_header(header: &ApiDict, out: &mut Packer) {
    out.map(count(header.len()));
    for item in header {
        out.str(item.key.bytes());
        match &item.value {
            Object::String(s) => out.bin(s.as_bytes()),
            Object::Integer(n) => out.integer(*n),
            other => unreachable!("shada: header holds an object of type {}", other.kind()),
        }
    }
}

/// One history line: the history it belongs to, its text, and — for search
/// history only — the character the search was started with.
fn pack_history(history: &ShadaHistoryItem, extra: &Option<ShadaExtra>, out: &mut Packer) {
    let is_search = history.histtype == HIST_SEARCH;
    out.array(2 + u32::from(is_search) + extra_len(extra));
    out.uint(u32::from(history.histtype));
    out.bin(history.text.as_bytes());
    if is_search {
        out.uint(u32::from(history.sep));
    }
    out.extra(extra);
}

/// One global variable. A Blob is packed as binary like a String, so it
/// carries a trailing type tag to tell the two apart when read back.
fn pack_variable(
    var: &ShadaGlobalVar,
    extra: &Option<ShadaExtra>,
    out: &mut Packer,
) -> Result<(), WriteStatus> {
    let is_blob = var.value.v_type() == VAR_BLOB;
    out.array(2 + u32::from(is_blob) + extra_len(extra));
    let varname = var.name.as_cstr().to_bytes();
    out.bin(varname);

    // What the encoder calls the value in its complaints.
    let mut vardesc = b"variable g:".to_vec();
    vardesc.extend_from_slice(varname);
    let vardesc = CString::new(vardesc).expect("a name up to its first NUL");
    let mut encoded = true;
    let value = pack_to_bytes(|packer| {
        encoded = encode_vim_to_msgpack(packer, &var.value, &vardesc);
    });
    if !encoded {
        let name = msg_cstr(var.name.as_cstr());
        semsg!("E574: Failed to write variable {name}");
        // The rest of the file is still worth writing.
        return Err(WriteStatus::IgnError);
    }
    out.raw(&value);
    if is_blob {
        out.integer(Integer::from(VAR_TYPE_BLOB));
    }
    out.extra(extra);
    Ok(())
}

/// The last search pattern, and the flags it was used with. Each flag is
/// written only when it differs from the default, and then always as the
/// *negation* of that default — a flag that is present is by definition not
/// the default value.
fn pack_search_pattern(
    pattern: &KeyDict__shada_search_pat,
    extra: &Option<ShadaExtra>,
    out: &mut Packer,
) {
    let default = DEFAULT_SEARCH_PATTERN;
    // Each flag, as (wire key, its value here, its default).
    let flags: [(&[u8], Option<bool>, Option<bool>); 8] = [
        (b"sm", pattern.magic, default.magic),
        (b"su", pattern.is_last_used, default.is_last_used),
        (b"sc", pattern.smartcase, default.smartcase),
        (b"sl", pattern.has_line_offset, default.has_line_offset),
        (
            b"se",
            pattern.place_cursor_at_end,
            default.place_cursor_at_end,
        ),
        (
            b"ss",
            pattern.is_substitute_pattern,
            default.is_substitute_pattern,
        ),
        (b"sh", pattern.highlighted, default.highlighted),
        (b"sb", pattern.search_backward, default.search_backward),
    ];

    let size = 1 // the pattern itself is always there
        + count(flags.iter().filter(|(_, value, d)| value != d).count())
        + written(pattern.offset, default.offset)
        + extra_len(extra);
    out.map(size);

    out.key(b"sp");
    out.bin(pattern.pat.as_ref().map_or(&[][..], |pat| pat.as_bytes()));
    for (name, _, default) in flags.iter().filter(|(_, value, d)| value != d) {
        out.key(name);
        out.boolean(!default.unwrap_or(false));
    }
    if pattern.offset != default.offset {
        out.key(b"so");
        out.integer(pattern.offset.unwrap_or(0));
    }
    out.extra(extra);
}

/// A global mark, local mark, jump or change: a file name and a position in
/// it, plus the mark's letter for the two kinds that have one.
fn pack_mark(kind: u64, mark: &ShadaFileMark, extra: &Option<ShadaExtra>, out: &mut Packer) {
    let default_name = default_mark_name(kind);
    let default = DEFAULT_POS;

    let size = 1 // the file name is always there
        + written(mark.mark.lnum, default.lnum)
        + written(mark.mark.col, default.col)
        + written(mark.name, default_name)
        + extra_len(extra);
    out.map(size);

    out.key(b"f");
    out.bin(mark.fname.as_cstr().to_bytes());
    if mark.mark.lnum != default.lnum {
        out.key(b"l");
        out.integer(Integer::from(mark.mark.lnum));
    }
    if mark.mark.col != default.col {
        out.key(b"c");
        out.integer(Integer::from(mark.mark.col));
    }
    if mark.name != default_name {
        out.key(b"n");
        out.uint(u32::from(mark.name as u8));
    }
    out.extra(extra);
}

/// One register: its lines, its name, and how it is put back.
fn pack_register(reg: &ShadaRegister, extra: &Option<ShadaExtra>, out: &mut Packer) {
    let default = DEFAULT_REGISTER;

    let size = 2 // the contents and the name are always there
        + written(reg.motion, default.motion)
        + written(reg.width, default.width)
        + written(reg.is_unnamed, default.is_unnamed)
        + extra_len(extra);
    out.map(size);

    out.key(b"rc");
    out.array(count(reg.contents.len()));
    for line in &reg.contents {
        out.bin(line.as_bytes());
    }
    out.key(b"n");
    out.uint(u32::from(reg.name as u8));
    if reg.motion != default.motion {
        out.key(b"rt");
        out.uint(u32::from(reg.motion as u8));
    }
    if reg.width != default.width {
        out.key(b"rw");
        out.uint64(reg.width as u64);
    }
    if reg.is_unnamed != default.is_unnamed {
        out.key(b"ru");
        out.boolean(reg.is_unnamed);
    }
    out.extra(extra);
}

/// The buffer list: one map per buffer, each a file name and the cursor
/// position in it. The position's defaults are the same for every buffer.
fn pack_buffer_list(list: &[ShadaBufferListItem], out: &mut Packer) {
    let default = DEFAULT_POS;
    out.array(count(list.len()));
    for buffer in list {
        let size = 1 // the file name is always there
            + written(buffer.pos.lnum, default.lnum)
            + written(buffer.pos.col, default.col)
            + extra_len(&buffer.extra);
        out.map(size);

        out.key(b"f");
        out.bin(buffer.fname.as_cstr().to_bytes());
        if buffer.pos.lnum != default.lnum {
            out.key(b"l");
            out.uint64(buffer.pos.lnum as u64);
        }
        if buffer.pos.col != default.col {
            out.key(b"c");
            out.uint64(buffer.pos.col as u64);
        }
        out.extra(&buffer.extra);
    }
}
