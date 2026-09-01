//! Turning a [`ShadaEntry`] into msgpack.
//!
//! One entry becomes four things in a row: its type, its timestamp, the byte
//! length of its payload, and the payload — see `unpack` for the reading
//! side. The payload is packed into a scratch buffer first, because the
//! length has to be written before it.
//!
//! Every field whose value equals the default for its entry type is left
//! out; `sd_default_values` is what both sides call the default, so a reader
//! puts it back. That is what keeps a ShaDa file small, and it is why the
//! map sizes below are counted before anything is written.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::message_fmt::c_str;
use crate::semsg;
use core::ffi::{CStr, c_char, c_int, c_void};

use crate::types::builders::static_cstring;

use super::*;
use crate::types::{FAIL, Object, VAR_BLOB, VAR_TYPE_BLOB};

/// Room the packer keeps free, so that a handful of small tokens can be
/// written without checking after each one.
const FREE_SPACE: size_t = 4 * MPACK_ITEM_SIZE as size_t;

/// Make room for the next few tokens, flushing the buffer if it is short.
pub(crate) unsafe fn shada_check_buffer(packer: *mut PackerBuffer) {
    if mpack_remaining(unsafe { &*packer }) < FREE_SPACE {
        unsafe { (*packer).packer_flush.expect("non-null function pointer")(packer) };
    }
}

/// How many map keys or array elements an entry carries that this Nvim has
/// no field for, and therefore stored verbatim.
unsafe fn additional_data_len(src: *mut AdditionalData) -> uint32_t {
    if src.is_null() {
        0
    } else {
        unsafe { (*src).nitems }
    }
}

/// Write those keys or elements back out, exactly as they were read.
unsafe fn dump_additional_data(src: *mut AdditionalData, sbuf: &mut PackerBuffer) {
    if !src.is_null() {
        unsafe {
            mpack_raw(
                (&raw mut (*src).data).cast::<c_char>(),
                (*src).nbytes as size_t,
                sbuf,
            )
        };
    }
}

/// The scratch buffer one entry's payload is packed into.
///
/// Its `packer_flush` grows the buffer rather than writing anything out, so
/// the whole payload ends up in one allocation that [`Payload::packed`]
/// hands back. Dropping it releases that allocation.
struct Payload {
    buf: PackerBuffer,
}

impl Payload {
    fn new() -> Self {
        let mut buf = packer_string_buffer();
        // SAFETY: a fresh string buffer.
        unsafe { shada_check_buffer(&raw mut buf) };
        Payload { buf }
    }

    /// A map key. ShaDa spells them as one- or two-letter codes.
    fn key(&mut self, name: &'static CStr) {
        // SAFETY: `name` is a static string and the buffer has room.
        unsafe { mpack_str(static_cstring(name), &mut self.buf) };
    }

    /// What has been packed so far. Valid until this is dropped.
    fn packed(&self) -> String_0 {
        packer_take_string(&self.buf)
    }
}

impl Drop for Payload {
    fn drop(&mut self) {
        // SAFETY: ours, from `packer_string_buffer`, and `flush` only ever
        // reallocates it in place of this field.
        unsafe { xfree(self.buf.startptr.cast::<c_void>()) };
    }
}

/// 1 when a field differs from its default and so has to be written.
fn written<T: PartialEq>(value: T, default: T) -> uint32_t {
    (value != default) as uint32_t
}

/// Write one entry, and free it.
///
/// Only used where the entry was built for the occasion; an entry that came
/// out of the merger is still owned by it.
pub(crate) unsafe fn shada_pack_pfreed_entry(
    packer: *mut PackerBuffer,
    mut entry: ShadaEntry,
    max_kbyte: size_t,
) -> ShaDaWriteResult {
    let ret = unsafe { shada_pack_entry(packer, entry, max_kbyte) };
    unsafe { shada_free_shada_entry(&raw mut entry) };
    ret
}

/// Write one entry.
///
/// `max_kbyte`, if non-zero, drops an entry whose payload comes out longer
/// than that many kilobytes — quietly, since the file is still valid
/// without it.
pub(crate) unsafe fn shada_pack_entry(
    packer: *mut PackerBuffer,
    entry: ShadaEntry,
    max_kbyte: size_t,
) -> ShaDaWriteResult {
    let mut payload = Payload::new();
    let sbuf = &mut payload.buf;

    let packed = match entry.data {
        ShadaEntryData::Missing => unreachable!("shada: a missing entry is never written"),
        ShadaEntryData::Unknown(item) => {
            unsafe { mpack_raw(item.contents, item.size, sbuf) };
            Ok(())
        }
        ShadaEntryData::Header(header) => {
            unsafe { pack_header(header, sbuf) };
            Ok(())
        }
        ShadaEntryData::HistoryEntry(history) => {
            unsafe { pack_history(&entry, history, sbuf) };
            Ok(())
        }
        ShadaEntryData::Variable(var) => unsafe { pack_variable(&entry, var, sbuf) },
        ShadaEntryData::SubString(sub) => {
            unsafe { pack_sub_string(&entry, sub, sbuf) };
            Ok(())
        }
        ShadaEntryData::SearchPattern(pattern) => {
            unsafe { pack_search_pattern(&entry, pattern, &mut payload) };
            Ok(())
        }
        ShadaEntryData::GlobalMark(mark)
        | ShadaEntryData::LocalMark(mark)
        | ShadaEntryData::Jump(mark)
        | ShadaEntryData::Change(mark) => {
            unsafe { pack_mark(&entry, mark, &mut payload) };
            Ok(())
        }
        ShadaEntryData::Register(reg) => {
            unsafe { pack_register(&entry, reg, &mut payload) };
            Ok(())
        }
        ShadaEntryData::BufferList(list) => {
            unsafe { pack_buffer_list(list, &mut payload) };
            Ok(())
        }
    };
    if let Err(ignorable) = packed {
        return ignorable;
    }

    let packed = payload.packed();
    if max_kbyte != 0 && packed.len() > max_kbyte * 1024 {
        return kSDWriteSuccessful; // too big to keep; not an error
    }

    unsafe { shada_check_buffer(packer) };
    // An unknown entry keeps the type it arrived with.
    mpack_uint64(
        unsafe { &mut (*packer).ptr },
        match entry.data {
            ShadaEntryData::Unknown(item) => item.type_0,
            data => data.kind() as uint64_t,
        },
    );
    mpack_uint64(unsafe { &mut (*packer).ptr }, entry.timestamp);
    if !packed.is_empty() {
        mpack_uint64(unsafe { &mut (*packer).ptr }, packed.len() as uint64_t);
        unsafe { mpack_raw(packed.data(), packed.len(), &mut *packer) };
    }

    if unsafe { (*packer).anyint } != 0 {
        return kSDWriteFailed; // the file's own error code
    }
    kSDWriteSuccessful
}

/// The file header: whatever `shada_write` chose to record about the Nvim
/// that wrote it. Nvim has never read it back — it is there for anyone
/// looking at the file by hand.
unsafe fn pack_header(header: Dict, sbuf: &mut PackerBuffer) {
    mpack_map(&mut sbuf.ptr, header.size as uint32_t);
    for i in 0..header.size {
        let item = unsafe { *header.items.add(i) };
        unsafe { mpack_str(item.key, sbuf) };
        match item.value {
            Object::String(s) => unsafe { mpack_bin(s, sbuf) },
            Object::Integer(n) => mpack_integer(&mut sbuf.ptr, n),
            other => unreachable!("shada: header holds an object of type {}", other.kind()),
        }
    }
}

/// One history line: the history it belongs to, its text, and — for search
/// history only — the character the search was started with.
unsafe fn pack_history(entry: &ShadaEntry, history: history_item, sbuf: &mut PackerBuffer) {
    let is_search = history.histtype as c_int == HIST_SEARCH;
    mpack_array(
        &mut sbuf.ptr,
        2 + is_search as uint32_t + unsafe { additional_data_len(entry.additional_data) },
    );
    mpack_uint(&mut sbuf.ptr, history.histtype as uint32_t);
    unsafe { mpack_bin(cstr_as_string(history.string), sbuf) };
    if is_search {
        mpack_uint(&mut sbuf.ptr, history.sep as uint8_t as uint32_t);
    }
    unsafe { dump_additional_data(entry.additional_data, sbuf) };
}

/// One global variable. A Blob is packed as binary like a String, so it
/// carries a trailing type tag to tell the two apart when read back.
unsafe fn pack_variable(
    entry: &ShadaEntry,
    mut global_var: global_var,
    sbuf: &mut PackerBuffer,
) -> Result<(), ShaDaWriteResult> {
    let is_blob = global_var.value.v_type == VAR_BLOB;
    mpack_array(
        &mut sbuf.ptr,
        2 + is_blob as uint32_t + unsafe { additional_data_len(entry.additional_data) },
    );
    let varname = unsafe { cstr_as_string(global_var.name) };
    unsafe { mpack_bin(varname, sbuf) };

    // What `encode_vim_to_msgpack` calls the value in its complaints.
    // Upstream formats this into a `char[256]` and `memcpy`s the name in
    // unbounded, overrunning it for a long enough variable name; built
    // here in a buffer that fits.
    let mut vardesc = b"variable g:".to_vec();
    vardesc.extend_from_slice(unsafe { varname.as_bytes() });
    vardesc.push(0);

    if unsafe {
        encode_vim_to_msgpack(
            sbuf,
            &raw mut global_var.value,
            vardesc.as_ptr().cast::<c_char>(),
        )
    } == FAIL
    {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let name = unsafe { c_str(global_var.name) };
        semsg!("E574: Failed to write variable {name}");
        // The rest of the file is still worth writing.
        return Err(kSDWriteIgnError);
    }
    if is_blob {
        mpack_check_buffer(sbuf);
        mpack_integer(&mut sbuf.ptr, VAR_TYPE_BLOB as Integer);
    }
    unsafe { dump_additional_data(entry.additional_data, sbuf) };
    Ok(())
}

/// The last `:substitute` replacement string.
unsafe fn pack_sub_string(entry: &ShadaEntry, sub: sub_string, sbuf: &mut PackerBuffer) {
    mpack_array(
        &mut sbuf.ptr,
        1 + unsafe { additional_data_len(entry.additional_data) },
    );
    unsafe { mpack_bin(cstr_as_string(sub.sub), sbuf) };
    unsafe { dump_additional_data(entry.additional_data, sbuf) };
}

/// The last search pattern, and the flags it was used with. Each flag is
/// written only when it differs from the default, and then always as the
/// *negation* of that default — a flag that is present is by definition not
/// the default value.
unsafe fn pack_search_pattern(
    entry: &ShadaEntry,
    pattern: KeyDict__shada_search_pat,
    payload: &mut Payload,
) {
    let default = DEFAULT_SEARCH_PATTERN;
    // Each flag, as (wire key, its value here, its default).
    let flags: [(&'static CStr, bool, bool); 8] = [
        (c"sm", pattern.magic, default.magic),
        (c"su", pattern.is_last_used, default.is_last_used),
        (c"sc", pattern.smartcase, default.smartcase),
        (c"sl", pattern.has_line_offset, default.has_line_offset),
        (
            c"se",
            pattern.place_cursor_at_end,
            default.place_cursor_at_end,
        ),
        (
            c"ss",
            pattern.is_substitute_pattern,
            default.is_substitute_pattern,
        ),
        (c"sh", pattern.highlighted, default.highlighted),
        (c"sb", pattern.search_backward, default.search_backward),
    ];

    let size = 1 // the pattern itself is always there
        + flags.iter().filter(|(_, value, d)| value != d).count() as uint32_t
        + written(pattern.offset, default.offset)
        + unsafe { additional_data_len(entry.additional_data) };
    mpack_map(&mut payload.buf.ptr, size);

    payload.key(c"sp");
    unsafe { mpack_bin(pattern.pat, &mut payload.buf) };
    for (name, _, default) in flags.iter().filter(|(_, value, d)| value != d) {
        payload.key(name);
        mpack_bool(&mut payload.buf.ptr, !default);
    }
    if pattern.offset != default.offset {
        payload.key(c"so");
        mpack_integer(&mut payload.buf.ptr, pattern.offset);
    }
    unsafe { dump_additional_data(entry.additional_data, &mut payload.buf) };
}

/// A global mark, local mark, jump or change: a file name and a position in
/// it, plus the mark's letter for the two kinds that have one.
unsafe fn pack_mark(entry: &ShadaEntry, mark: shada_filemark, payload: &mut Payload) {
    let default = default_filemark(entry.kind());

    let size = 1 // the file name is always there
        + written(mark.mark.lnum, default.mark.lnum)
        + written(mark.mark.col, default.mark.col)
        + written(mark.name, default.name)
        + unsafe { additional_data_len(entry.additional_data) };
    mpack_map(&mut payload.buf.ptr, size);

    payload.key(c"f");
    unsafe { mpack_bin(cstr_as_string(mark.fname), &mut payload.buf) };
    if mark.mark.lnum != default.mark.lnum {
        payload.key(c"l");
        mpack_integer(&mut payload.buf.ptr, mark.mark.lnum as Integer);
    }
    if mark.mark.col != default.mark.col {
        payload.key(c"c");
        mpack_integer(&mut payload.buf.ptr, mark.mark.col as Integer);
    }
    debug_assert!(
        !matches!(
            entry.data,
            ShadaEntryData::Jump(_) | ShadaEntryData::Change(_)
        ) || mark.name == default.name,
        "shada: a jump or change entry has no mark name"
    );
    if mark.name != default.name {
        payload.key(c"n");
        mpack_uint(&mut payload.buf.ptr, mark.name as uint8_t as uint32_t);
    }
    unsafe { dump_additional_data(entry.additional_data, &mut payload.buf) };
}

/// One register: its lines, its name, and how it is put back.
unsafe fn pack_register(entry: &ShadaEntry, reg: reg, payload: &mut Payload) {
    let default = DEFAULT_REGISTER;

    let size = 2 // the contents and the name are always there
        + written(reg.type_0, default.type_0)
        + written(reg.width, default.width)
        + written(reg.is_unnamed, default.is_unnamed)
        + unsafe { additional_data_len(entry.additional_data) };
    mpack_map(&mut payload.buf.ptr, size);

    payload.key(c"rc");
    mpack_array(&mut payload.buf.ptr, reg.contents_size as uint32_t);
    for i in 0..reg.contents_size {
        unsafe { mpack_bin(*reg.contents.add(i), &mut payload.buf) };
    }
    payload.key(c"n");
    mpack_uint(&mut payload.buf.ptr, reg.name as uint8_t as uint32_t);
    if reg.type_0 != default.type_0 {
        payload.key(c"rt");
        mpack_uint(&mut payload.buf.ptr, reg.type_0 as uint8_t as uint32_t);
    }
    if reg.width != default.width {
        payload.key(c"rw");
        mpack_uint64(&mut payload.buf.ptr, reg.width as uint64_t);
    }
    if reg.is_unnamed != default.is_unnamed {
        payload.key(c"ru");
        mpack_bool(&mut payload.buf.ptr, reg.is_unnamed);
    }
    unsafe { dump_additional_data(entry.additional_data, &mut payload.buf) };
}

/// The buffer list: one map per buffer, each a file name and the cursor
/// position in it. The position's defaults are the same for every buffer,
/// so they come from `DEFAULT_POS` rather than from an entry type.
unsafe fn pack_buffer_list(list: buffer_list, payload: &mut Payload) {
    let default = DEFAULT_POS;
    mpack_array(&mut payload.buf.ptr, list.size as uint32_t);
    for i in 0..list.size {
        let buffer = unsafe { *list.buffers.add(i) };
        let size = 1 // the file name is always there
            + written(buffer.pos.lnum, default.lnum)
            + written(buffer.pos.col, default.col)
            + unsafe { additional_data_len(buffer.additional_data) };
        mpack_map(&mut payload.buf.ptr, size);

        payload.key(c"f");
        unsafe { mpack_bin(cstr_as_string(buffer.fname), &mut payload.buf) };
        if buffer.pos.lnum != default.lnum {
            payload.key(c"l");
            mpack_uint64(&mut payload.buf.ptr, buffer.pos.lnum as uint64_t);
        }
        if buffer.pos.col != default.col {
            payload.key(c"c");
            mpack_uint64(&mut payload.buf.ptr, buffer.pos.col as uint64_t);
        }
        unsafe { dump_additional_data(buffer.additional_data, &mut payload.buf) };
    }
}

/// A packer that writes straight into the file's own buffer.
///
/// The file keeps its write position in the buffer, so the packer starts
/// where the file left off and hands the position back on every flush.
pub(crate) unsafe fn packer_buffer_for_file(file: *mut FileDescriptor) -> PackerBuffer {
    if unsafe { file_space(file) } < FREE_SPACE {
        unsafe { file_flush(file) };
    }
    PackerBuffer {
        startptr: unsafe { (*file).buffer },
        ptr: unsafe { (*file).write_pos },
        endptr: unsafe { (*file).buffer.add(ARENA_BLOCK_SIZE as usize) },
        anydata: file.cast::<c_void>(),
        anyint: 0,
        packer_flush: Some(flush_file_buffer),
    }
}

/// Hand what has been packed to the file, and start again at whatever it
/// leaves in its buffer.
unsafe fn flush_file_buffer(buffer: *mut PackerBuffer) {
    let fd = unsafe { (*buffer).anydata.cast::<FileDescriptor>() };
    unsafe { (*fd).write_pos = (*buffer).ptr };
    unsafe { (*buffer).anyint = file_flush(fd) as int64_t };
    unsafe { (*buffer).ptr = (*fd).write_pos };
}
