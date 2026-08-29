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

use crate::semsg_c;
use core::ffi::{CStr, c_char, c_int, c_void};

use crate::types::builders::static_cstring;

use super::*;
use crate::types::{FAIL, VAR_BLOB, VAR_TYPE_BLOB};

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

/// The values a reader assumes for the fields an entry leaves out.
unsafe fn defaults_for(type_0: ShadaEntryType) -> ShadaEntryData {
    sd_default_values[type_0 as usize].data
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

    let packed = match entry.type_0 {
        kSDItemMissing => unreachable!("shada: a missing entry is never written"),
        kSDItemUnknown => {
            unsafe {
                mpack_raw(
                    entry.data.unknown_item.contents,
                    entry.data.unknown_item.size,
                    sbuf,
                )
            };
            Ok(())
        }
        kSDItemHeader => {
            unsafe { pack_header(&entry, sbuf) };
            Ok(())
        }
        kSDItemHistoryEntry => {
            unsafe { pack_history(&entry, sbuf) };
            Ok(())
        }
        kSDItemVariable => unsafe { pack_variable(&entry, sbuf) },
        kSDItemSubString => {
            unsafe { pack_sub_string(&entry, sbuf) };
            Ok(())
        }
        kSDItemSearchPattern => {
            unsafe { pack_search_pattern(&entry, &mut payload) };
            Ok(())
        }
        kSDItemChange | kSDItemGlobalMark | kSDItemLocalMark | kSDItemJump => {
            unsafe { pack_mark(&entry, &mut payload) };
            Ok(())
        }
        kSDItemRegister => {
            unsafe { pack_register(&entry, &mut payload) };
            Ok(())
        }
        kSDItemBufferList => {
            unsafe { pack_buffer_list(&entry, &mut payload) };
            Ok(())
        }
        _ => unreachable!("shada: entry type {} is not written here", entry.type_0),
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
        if entry.type_0 == kSDItemUnknown {
            unsafe { entry.data.unknown_item }.type_0
        } else {
            entry.type_0 as uint64_t
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
unsafe fn pack_header(entry: &ShadaEntry, sbuf: &mut PackerBuffer) {
    let header = unsafe { entry.data.header };
    mpack_map(&mut sbuf.ptr, header.size as uint32_t);
    for i in 0..header.size {
        let item = unsafe { *header.items.add(i) };
        unsafe { mpack_str(item.key, sbuf) };
        match item.value.type_0 {
            kObjectTypeString => unsafe { mpack_bin(item.value.data.string, sbuf) },
            kObjectTypeInteger => mpack_integer(&mut sbuf.ptr, unsafe { item.value.data.integer }),
            other => unreachable!("shada: header holds an object of type {other}"),
        }
    }
}

/// One history line: the history it belongs to, its text, and — for search
/// history only — the character the search was started with.
unsafe fn pack_history(entry: &ShadaEntry, sbuf: &mut PackerBuffer) {
    let history = unsafe { entry.data.history_item };
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
    sbuf: &mut PackerBuffer,
) -> Result<(), ShaDaWriteResult> {
    let mut global_var = unsafe { entry.data.global_var };
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
        unsafe {
            semsg_c!(
                gettext(c"E574: Failed to write variable %s"),
                global_var.name,
            )
        };
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
unsafe fn pack_sub_string(entry: &ShadaEntry, sbuf: &mut PackerBuffer) {
    mpack_array(
        &mut sbuf.ptr,
        1 + unsafe { additional_data_len(entry.additional_data) },
    );
    unsafe { mpack_bin(cstr_as_string(entry.data.sub_string.sub), sbuf) };
    unsafe { dump_additional_data(entry.additional_data, sbuf) };
}

/// The last search pattern, and the flags it was used with. Each flag is
/// written only when it differs from the default, and then always as the
/// *negation* of that default — a flag that is present is by definition not
/// the default value.
unsafe fn pack_search_pattern(entry: &ShadaEntry, payload: &mut Payload) {
    let pattern = unsafe { entry.data.search_pattern };
    let default = unsafe { defaults_for(entry.type_0).search_pattern };
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
unsafe fn pack_mark(entry: &ShadaEntry, payload: &mut Payload) {
    let mark = unsafe { entry.data.filemark };
    let default = unsafe { defaults_for(entry.type_0).filemark };

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
        !(entry.type_0 == kSDItemJump || entry.type_0 == kSDItemChange)
            || mark.name == default.name,
        "shada: a jump or change entry has no mark name"
    );
    if mark.name != default.name {
        payload.key(c"n");
        mpack_uint(&mut payload.buf.ptr, mark.name as uint8_t as uint32_t);
    }
    unsafe { dump_additional_data(entry.additional_data, &mut payload.buf) };
}

/// One register: its lines, its name, and how it is put back.
unsafe fn pack_register(entry: &ShadaEntry, payload: &mut Payload) {
    let reg = unsafe { entry.data.reg };
    let default = unsafe { defaults_for(entry.type_0).reg };

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
unsafe fn pack_buffer_list(entry: &ShadaEntry, payload: &mut Payload) {
    let list = unsafe { entry.data.buffer_list };
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
