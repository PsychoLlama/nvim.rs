//! Parsing one ShaDa entry's payload.
//!
//! `unpack` has read the entry's type, timestamp and length and handed the
//! bytes over; the function here for that type turns them into the fields of
//! a [`ShadaEntry`]. Anything in the payload this Nvim has no field for is
//! collected as *additional data*, which travels with the entry and is
//! written back out unchanged.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{CStr, c_char, c_int, c_void};

use crate::msgpack_rpc::unpacker::MPACK_OK;

use super::*;
use crate::types::VAR_TYPE_BLOB;

/// Parse the payload of an entry whose type this Nvim knows.
pub(crate) unsafe fn parse_known(
    entry: *mut ShadaEntry,
    header: &Header,
    cursor: &mut Cursor,
) -> Result<(), Malformed> {
    // Fields the file leaves out keep the type's documented default.
    unsafe { (*entry).data = sd_default_values[header.type_u64 as usize].data };

    // Map keys this Nvim does not know are collected here and written
    // back out with the entry.
    let mut extra = KV_INITIAL_VALUE;
    let mut error = core::ptr::null_mut::<c_char>();
    let pos = header.fpos;

    // How many elements of an array entry are left for `extra` once the
    // ones this Nvim understands have been taken.
    let trailing = match header.type_u64 as ShadaEntryType {
        kSDItemHeader => {
            // The header is written for the benefit of anyone reading
            // the file by hand; Nvim has never read it back.
            Ok(0)
        }
        kSDItemSearchPattern => unsafe {
            parse_search_pattern(entry, pos, cursor, &mut extra, &mut error)
        },
        kSDItemChange | kSDItemJump | kSDItemGlobalMark | kSDItemLocalMark => unsafe {
            parse_mark(entry, header, cursor, &mut extra, &mut error)
        },
        kSDItemRegister => unsafe { parse_register(entry, pos, cursor, &mut extra, &mut error) },
        kSDItemHistoryEntry => unsafe { parse_history(entry, pos, cursor) },
        kSDItemVariable => unsafe { parse_variable(entry, pos, cursor) },
        kSDItemSubString => unsafe { parse_sub_string(entry, pos, cursor) },
        kSDItemBufferList => unsafe { parse_buffer_list(entry, pos, cursor, &mut error) },
        _ => unreachable!("shada: entry type {} is not read here", header.type_u64),
    };

    let finish = trailing.and_then(|trailing| {
        for _ in 0..trailing {
            let item_start = cursor.at;
            if cursor.skip() != 0 {
                return Err(Malformed);
            }
            unsafe { push_additional_data(
                &mut extra,
                item_start,
                cursor.at.offset_from_unsigned(item_start),
            ) };
        }
        if cursor.left != 0 {
            unsafe { semsg_c!(
                gettext(
                    c"E575: Error while reading ShaDa file: item entry at position %lu additional bytes"
                        .as_ptr(),
                ),
                pos,
            ) };
            return Err(Malformed);
        }
        Ok(())
    });

    match finish {
        Ok(()) => {
            unsafe { (*entry).type_0 = header.type_u64 as ShadaEntryType };
            unsafe { (*entry).additional_data = extra.items.cast::<AdditionalData>() };
            Ok(())
        }
        Err(Malformed) => {
            unsafe { xfree(error.cast::<c_void>()) };
            unsafe { xfree(extra.items.cast::<c_void>()) };
            Err(Malformed)
        }
    }
}

/// The last search pattern, as a map of the two-letter `s*` keys.
unsafe fn parse_search_pattern(
    entry: *mut ShadaEntry,
    pos: uint64_t,
    cursor: &mut Cursor,
    extra: &mut AdditionalDataBuilder,
    error: &mut *mut c_char,
) -> Result<uint32_t, Malformed> {
    let it = unsafe { &raw mut (*entry).data.search_pattern };
    if !cursor.keydict(
        it.cast::<c_void>(),
        Some(key_dict__shada_search_pat_get_field),
        extra,
        error,
    ) {
        unsafe {
            semsg_c!(
            gettext(
                c"E575: Error while reading ShaDa file: search pattern entry at position %lu %s"
                    .as_ptr(),
            ),
            pos,
            *error,
        )
        };
        // The keyset may have been left holding a borrowed pattern.
        unsafe { (*it).pat = String_0::NULL };
        return Err(Malformed);
    }
    if !has_key(
        unsafe { (*it).is_set___shada_search_pat_ },
        KEYSET_OPTIDX__shada_search_pat__sp,
    ) {
        unsafe {
            semsg_c!(
            gettext(
                c"E575: Error while reading ShaDa file: search pattern entry at position %lu has no pattern"
                    .as_ptr(),
            ),
            pos,
        )
        };
        return Err(Malformed);
    }
    // The pattern still points into the entry's bytes; take a copy that
    // outlives them.
    unsafe { (*it).pat = copy_string((*it).pat, core::ptr::null_mut::<Arena>()) };
    Ok(0)
}

/// A global mark, local mark, jump or change: all four are the same map.
unsafe fn parse_mark(
    entry: *mut ShadaEntry,
    header: &Header,
    cursor: &mut Cursor,
    extra: &mut AdditionalDataBuilder,
    error: &mut *mut c_char,
) -> Result<uint32_t, Malformed> {
    let pos = header.fpos;
    let mut it = KeyDict__shada_mark {
        is_set___shada_mark_: 0,
        n: 0,
        l: 0,
        c: 0,
        f: String_0::NULL,
    };
    if !cursor.keydict(
        (&raw mut it).cast::<c_void>(),
        Some(key_dict__shada_mark_get_field),
        extra,
        error,
    ) {
        unsafe {
            semsg_c!(
                gettext(
                    c"E575: Error while reading ShaDa file: mark entry at position %lu %s".as_ptr(),
                ),
                pos,
                *error,
            )
        };
        return Err(Malformed);
    }

    let mark = unsafe { &raw mut (*entry).data.filemark };
    if has_key(it.is_set___shada_mark_, KEYSET_OPTIDX__shada_mark__n) {
        if header.type_u64 == kSDItemJump as uint64_t
            || header.type_u64 == kSDItemChange as uint64_t
        {
            unsafe {
                semsg_c!(
                gettext(
                    c"E575: Error while reading ShaDa file: mark entry at position %lu has n key which is only valid for local and global mark entries"
                        .as_ptr(),
                ),
                pos,
            )
            };
            return Err(Malformed);
        }
        unsafe { (*mark).name = it.n as c_char };
    }
    if has_key(it.is_set___shada_mark_, KEYSET_OPTIDX__shada_mark__l) {
        unsafe { (*mark).mark.lnum = it.l as linenr_T };
    }
    if has_key(it.is_set___shada_mark_, KEYSET_OPTIDX__shada_mark__c) {
        unsafe { (*mark).mark.col = it.c as colnr_T };
    }
    if has_key(it.is_set___shada_mark_, KEYSET_OPTIDX__shada_mark__f) {
        unsafe {
            (*mark).fname = xmemdupz(it.f.data().cast::<c_void>(), it.f.len()).cast::<c_char>()
        };
    }

    if unsafe { (*mark).fname.is_null() } {
        unsafe {
            semsg_c!(
            gettext(
                c"E575: Error while reading ShaDa file: mark entry at position %lu is missing file name"
                    .as_ptr(),
            ),
            pos,
        )
        };
        return Err(Malformed);
    }
    if unsafe { (*mark).mark.lnum } <= 0 {
        unsafe {
            semsg_c!(
            gettext(
                c"E575: Error while reading ShaDa file: mark entry at position %lu has invalid line number"
                    .as_ptr(),
            ),
            pos,
        )
        };
        return Err(Malformed);
    }
    if unsafe { (*mark).mark.col } < 0 {
        unsafe {
            semsg_c!(
            gettext(
                c"E575: Error while reading ShaDa file: mark entry at position %lu has invalid column number"
                    .as_ptr(),
            ),
            pos,
        )
        };
        return Err(Malformed);
    }
    Ok(0)
}

/// One register: a name, a motion type, a width and the lines in it.
unsafe fn parse_register(
    entry: *mut ShadaEntry,
    pos: uint64_t,
    cursor: &mut Cursor,
    extra: &mut AdditionalDataBuilder,
    error: &mut *mut c_char,
) -> Result<uint32_t, Malformed> {
    let mut it = KeyDict__shada_register {
        is_set___shada_register_: 0,
        rc: StringArray {
            size: 0,
            capacity: 0,
            items: core::ptr::null_mut(),
        },
        ru: false,
        rt: 0,
        n: 0,
        rw: 0,
    };
    let ok = cursor.keydict(
        (&raw mut it).cast::<c_void>(),
        Some(key_dict__shada_register_get_field),
        extra,
        error,
    );
    // The contents array is the keyset's own allocation either way.
    let contents = core::mem::replace(
        &mut it.rc,
        StringArray {
            size: 0,
            capacity: 0,
            items: core::ptr::null_mut(),
        },
    );
    let lines = if contents.items.is_null() {
        &[][..]
    } else {
        unsafe { core::slice::from_raw_parts(contents.items, contents.size) }
    };
    let claim = (|| {
        if !ok {
            unsafe {
                semsg_c!(
                    gettext(
                        c"E575: Error while reading ShaDa file: register entry at position %lu %s"
                            .as_ptr(),
                    ),
                    pos,
                    *error,
                )
            };
            return Err(Malformed);
        }
        if lines.is_empty() {
            unsafe {
                semsg_c!(
                gettext(
                    c"E575: Error while reading ShaDa file: register entry at position %lu has rc key with missing or empty array"
                        .as_ptr(),
                ),
                pos,
            )
            };
            return Err(Malformed);
        }
        let reg = unsafe { &raw mut (*entry).data.reg };
        unsafe { (*reg).contents_size = lines.len() };
        unsafe { (*reg).contents = xmalloc(size_of_val(lines)).cast::<String_0>() };
        for (i, line) in lines.iter().enumerate() {
            // Each line still points into the entry's bytes.
            unsafe {
                (*reg)
                    .contents
                    .add(i)
                    .write(copy_string(*line, core::ptr::null_mut::<Arena>()))
            };
        }
        if has_key(
            it.is_set___shada_register_,
            KEYSET_OPTIDX__shada_register__ru,
        ) {
            unsafe { (*reg).is_unnamed = it.ru };
        }
        if has_key(
            it.is_set___shada_register_,
            KEYSET_OPTIDX__shada_register__rt,
        ) {
            unsafe { (*reg).type_0 = it.rt as uint8_t as MotionType };
        }
        if has_key(
            it.is_set___shada_register_,
            KEYSET_OPTIDX__shada_register__n,
        ) {
            unsafe { (*reg).name = it.n as c_char };
        }
        if has_key(
            it.is_set___shada_register_,
            KEYSET_OPTIDX__shada_register__rw,
        ) {
            unsafe { (*reg).width = it.rw as size_t };
        }
        Ok(0)
    })();
    unsafe { xfree(contents.items.cast::<c_void>()) };
    claim
}

/// One history line: `[type, text]`, plus a separator for search history.
///
/// The stored string carries the separator after its NUL, which is what
/// `hms_insert` and the history table expect.
unsafe fn parse_history(
    entry: *mut ShadaEntry,
    pos: uint64_t,
    cursor: &mut Cursor,
) -> Result<uint32_t, Malformed> {
    let len = cursor.array();
    if len < 2 {
        unsafe {
            semsg_c!(
            gettext(
                c"E575: Error while reading ShaDa file: history entry at position %lu is not an array with enough elements"
                    .as_ptr(),
            ),
            pos,
        )
        };
        return Err(Malformed);
    }
    let Some(hist_type) = cursor.integer() else {
        unsafe {
            semsg_c!(
            gettext(
                c"E575: Error while reading ShaDa file: history entry at position %lu has wrong history type type"
                    .as_ptr(),
            ),
            pos,
        )
        };
        return Err(Malformed);
    };
    let item = cursor.string();
    if item.data().is_null() {
        unsafe {
            semsg_c!(
            gettext(
                c"E575: Error while reading ShaDa file: history entry at position %lu has wrong history string type"
                    .as_ptr(),
            ),
            pos,
        )
        };
        return Err(Malformed);
    }
    let text = unsafe { item.as_bytes() };
    if text.contains(&0) {
        unsafe {
            semsg_c!(
            gettext(
                c"E575: Error while reading ShaDa file: history entry at position %lu contains string with zero byte inside"
                    .as_ptr(),
            ),
            pos,
        )
        };
        return Err(Malformed);
    }

    let history = unsafe { &raw mut (*entry).data.history_item };
    unsafe { (*history).histtype = hist_type as uint8_t };
    let is_search = unsafe { (*history).histtype } as c_int == HIST_SEARCH;
    if is_search {
        if len < 3 {
            unsafe {
                semsg_c!(
                gettext(
                    c"E575: Error while reading ShaDa file: search history entry at position %lu does not have separator character"
                        .as_ptr(),
                ),
                pos,
            )
            };
            return Err(Malformed);
        }
        let Some(sep) = cursor.integer() else {
            unsafe {
                semsg_c!(
                gettext(
                    c"E575: Error while reading ShaDa file: search history entry at position %lu has wrong history separator type"
                        .as_ptr(),
                ),
                pos,
            )
            };
            return Err(Malformed);
        };
        unsafe { (*history).sep = sep as c_char };
    }

    // The text, a NUL, then the separator byte after it.
    let stored = unsafe { xmalloc(text.len() + 2) }.cast::<c_char>();
    unsafe {
        stored
            .cast::<u8>()
            .copy_from_nonoverlapping(text.as_ptr(), text.len())
    };
    unsafe { stored.add(text.len()).write(0) };
    unsafe { stored.add(text.len() + 1).write((*history).sep) };
    unsafe { (*history).string = stored };

    Ok((len - (2 + is_search as ssize_t)) as uint32_t)
}

/// One global variable: `[name, value]`, with a trailing type tag when the
/// value is a Blob (which is otherwise indistinguishable from a String).
unsafe fn parse_variable(
    entry: *mut ShadaEntry,
    pos: uint64_t,
    cursor: &mut Cursor,
) -> Result<uint32_t, Malformed> {
    let len = cursor.array();
    if len < 2 {
        unsafe {
            semsg_c!(
            gettext(
                c"E575: Error while reading ShaDa file: variable entry at position %lu is not an array with enough elements"
                    .as_ptr(),
            ),
            pos,
        )
        };
        return Err(Malformed);
    }
    let name = cursor.string();
    if name.data().is_null() {
        unsafe {
            semsg_c!(
            gettext(
                c"E575: Error while reading ShaDa file: variable entry at position %lu has wrong variable name type"
                    .as_ptr(),
            ),
            pos,
        )
        };
        return Err(Malformed);
    }
    let global_var = unsafe { &raw mut (*entry).data.global_var };
    unsafe {
        (*global_var).name = xmemdupz(name.data().cast::<c_void>(), name.len()).cast::<c_char>()
    };

    let binval = cursor.string();
    let mut is_blob = false;
    if !binval.data().is_null() {
        if len > 2 {
            if cursor.integer() != Some(VAR_TYPE_BLOB as Integer) {
                unsafe {
                    semsg_c!(
                    gettext(
                        c"E575: Error while reading ShaDa file: variable entry at position %lu has wrong variable type"
                            .as_ptr(),
                    ),
                    pos,
                )
                };
                return Err(Malformed);
            }
            is_blob = true;
        }
        unsafe { (*global_var).value = decode_string(binval.data(), binval.len(), is_blob, false) };
    } else if cursor.typval(unsafe { &raw mut (*global_var).value }) != MPACK_OK {
        unsafe {
            semsg_c!(
            gettext(
                c"E575: Error while reading ShaDa file: variable entry at position %lu has value that cannot be converted to the Vimscript value"
                    .as_ptr(),
            ),
            pos,
        )
        };
        return Err(Malformed);
    }
    Ok((len - 2 - is_blob as ssize_t) as uint32_t)
}

/// The last `:substitute` replacement string: a one-element array.
unsafe fn parse_sub_string(
    entry: *mut ShadaEntry,
    pos: uint64_t,
    cursor: &mut Cursor,
) -> Result<uint32_t, Malformed> {
    let len = cursor.array();
    if len < 1 {
        unsafe {
            semsg_c!(
            gettext(
                c"E575: Error while reading ShaDa file: sub string entry at position %lu is not an array with enough elements"
                    .as_ptr(),
            ),
            pos,
        )
        };
        return Err(Malformed);
    }
    let sub = cursor.string();
    if sub.data().is_null() {
        unsafe {
            semsg_c!(
            gettext(
                c"E575: Error while reading ShaDa file: sub string entry at position %lu has wrong sub string type"
                    .as_ptr(),
            ),
            pos,
        )
        };
        return Err(Malformed);
    }
    unsafe {
        (*entry).data.sub_string.sub =
            xmemdupz(sub.data().cast::<c_void>(), sub.len()).cast::<c_char>()
    };
    Ok((len - 1) as uint32_t)
}

/// The buffer list: an array of maps, one per buffer, each a file name and
/// the cursor position in it.
unsafe fn parse_buffer_list(
    entry: *mut ShadaEntry,
    pos: uint64_t,
    cursor: &mut Cursor,
    error: &mut *mut c_char,
) -> Result<uint32_t, Malformed> {
    let len = cursor.array();
    if len < 0 {
        unsafe {
            semsg_c!(
            gettext(
                c"E575: Error while reading ShaDa file: buffer list entry at position %lu is not an array"
                    .as_ptr(),
            ),
            pos,
        )
        };
        return Err(Malformed);
    }
    if len == 0 {
        return Ok(0);
    }

    let list = unsafe { &raw mut (*entry).data.buffer_list };
    unsafe {
        (*list).buffers =
            xcalloc(len as size_t, size_of::<buffer_list_buffer>()).cast::<buffer_list_buffer>()
    };
    for i in 0..len as usize {
        // Count it before it is filled in, so that a failure below still
        // frees what has been built.
        unsafe { (*list).size += 1 };
        let mut it = KeyDict__shada_buflist_item {
            is_set___shada_buflist_item_: 0,
            l: 0,
            c: 0,
            f: String_0::NULL,
        };
        let mut item_extra = KV_INITIAL_VALUE;
        if !cursor.keydict(
            (&raw mut it).cast::<c_void>(),
            Some(key_dict__shada_buflist_item_get_field),
            &mut item_extra,
            error,
        ) {
            unsafe {
                semsg_c!(
                gettext(
                    c"E575: Error while reading ShaDa file: buffer list at position %lu contains entry that %s"
                        .as_ptr(),
                ),
                pos,
                *error,
            )
            };
            unsafe { xfree(item_extra.items.cast::<c_void>()) };
            return Err(Malformed);
        }
        let e = unsafe { (*list).buffers.add(i) };
        unsafe { (*e).additional_data = item_extra.items.cast::<AdditionalData>() };
        unsafe { (*e).pos = DEFAULT_POS };
        if has_key(
            it.is_set___shada_buflist_item_,
            KEYSET_OPTIDX__shada_buflist_item__l,
        ) {
            unsafe { (*e).pos.lnum = it.l as linenr_T };
        }
        if has_key(
            it.is_set___shada_buflist_item_,
            KEYSET_OPTIDX__shada_buflist_item__c,
        ) {
            unsafe { (*e).pos.col = it.c as colnr_T };
        }
        if has_key(
            it.is_set___shada_buflist_item_,
            KEYSET_OPTIDX__shada_buflist_item__f,
        ) {
            unsafe {
                (*e).fname = xmemdupz(it.f.data().cast::<c_void>(), it.f.len()).cast::<c_char>()
            };
        }

        let complaint: Option<&CStr> = if unsafe { (*e).pos.lnum } <= 0 {
            Some(c"E575: Error while reading ShaDa file: buffer list at position %lu contains entry with invalid line number")
        } else if unsafe { (*e).pos.col } < 0 {
            Some(c"E575: Error while reading ShaDa file: buffer list at position %lu contains entry with invalid column number")
        } else if unsafe { (*e).fname.is_null() } {
            Some(c"E575: Error while reading ShaDa file: buffer list at position %lu contains entry that does not have a file name")
        } else {
            None
        };
        if let Some(complaint) = complaint {
            unsafe { semsg_c!(gettext(complaint.as_ptr()), pos) };
            return Err(Malformed);
        }
    }
    Ok(0)
}

/// Whether an optional keyset key was present in the map.
fn has_key(is_set: OptionalKeys, index: c_int) -> bool {
    is_set & (1 << index) != 0
}
