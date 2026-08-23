//! The `Object` tree: where one comes from and where it goes.
//!
//! An object is either arena-allocated — `arena_*` and `copy_*` build those,
//! and the arena reclaims the whole tree at once — or heap-allocated, and
//! then `api_free_*` takes it apart member by member. `api_luarefs_free_*`
//! is the third case: an arena-allocated tree still holds Lua registry
//! references, which the arena knows nothing about.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{EMPTY_HL_MESSAGE, NIL, api_set_error, cstr_as_string};
use crate::api::private::metadata::PACKED_API_METADATA;
use crate::api::private::validate::{api_err_exp, api_err_invalid};
use crate::global_cell::GlobalCell;
use crate::highlight_group::{HLF_E, highlight_num_groups, syn_check_group};
use crate::kvec::InitVec;
use crate::lua::executor::{api_free_luaref, api_new_luaref};
use crate::memory::{
    ARENA_EMPTY, arena_alloc, arena_finish, arena_memdupz, xfree, xrealloc, xstrdup,
};
use crate::message::hl_msg_free;
use crate::msgpack_rpc::unpacker::unpack;
use crate::types::builders::static_cstring;
use crate::types::{
    Arena, ArenaMem, Array, ArrayBuilder, Dict, Error, HlMessage, HlMessageChunk, KeyValuePair,
    Object, ObjectType, String_0, consumed_blk, kErrorTypeNone, kErrorTypeValidation,
    kObjectTypeArray, kObjectTypeBoolean, kObjectTypeBuffer, kObjectTypeDict, kObjectTypeFloat,
    kObjectTypeInteger, kObjectTypeLuaRef, kObjectTypeNil, kObjectTypeString, kObjectTypeTabpage,
    kObjectTypeWindow, key_value_pair, object, object_data, size_t,
};
use ::libc::{abort, memcpy};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

// -- Arena allocation ------------------------------------------------------

/// An empty array with room for `max_size` items, taken from `arena` — or
/// from the heap when `arena` is null.
pub(crate) fn arena_array(arena: *mut Arena, max_size: size_t) -> Array {
    // SAFETY: `arena_alloc` accepts a null arena and falls back to `xmalloc`.
    let items = unsafe { arena_alloc(arena, size_of::<Object>() * max_size, true) };
    Array {
        size: 0,
        capacity: max_size,
        items: items.cast(),
    }
}

/// [`arena_array`] for a dictionary.
pub(crate) fn arena_dict(arena: *mut Arena, max_size: size_t) -> Dict {
    // SAFETY: as `arena_array`.
    let items = unsafe { arena_alloc(arena, size_of::<KeyValuePair>() * max_size, true) };
    Dict {
        size: 0,
        capacity: max_size,
        items: items.cast(),
    }
}

/// C's `ADD_C(array, value)`: append to an array whose capacity was reserved
/// up front, by [`arena_array`] or by an on-stack literal.
///
/// The transpile spells this as `let n = a.size; a.size += 1; *a.items.add(n)
/// = value;` at every site — note the order, which is why the capacity check
/// here is a `debug_assert!`: `size` is bumped before `value` is stored either
/// way, and every caller sized the container from the same expression that
/// decides how many times it pushes.
///
/// # Safety
/// `array` must have room, and its `items` must be writable for `capacity`.
pub(crate) unsafe fn array_add(array: &mut Array, value: Object) {
    debug_assert!(array.size < array.capacity, "array_add past capacity");
    // SAFETY: `size` is below `capacity`, so the slot is inside `items`.
    unsafe { *array.items.add(array.size) = value };
    array.size += 1;
}

/// C's `PUT_C(dict, key, value)`. See [`array_add`].
///
/// The key is a `&'static CStr` because these keys are all literals and the
/// consumers (msgpack, the Lua converter, the editor's own hashtables) read
/// one byte past `size`; `count_bytes` is const where the transpile's
/// `cstr_as_string` was a `strlen` per call.
///
/// # Safety
/// As [`array_add`].
pub(crate) unsafe fn dict_put(dict: &mut Dict, key: &'static CStr, value: Object) {
    // SAFETY: as `array_add`.
    unsafe { dict_put_str(dict, static_cstring(key), value) };
}

/// [`dict_put`] where the key is not a literal — an option name, a buffer
/// variable's name, anything the caller built.
///
/// # Safety
/// As [`array_add`]; `key` must outlive the dictionary.
pub(crate) unsafe fn dict_put_str(dict: &mut Dict, key: String_0, value: Object) {
    debug_assert!(dict.size < dict.capacity, "dict_put past capacity");
    // SAFETY: `size` is below `capacity`, so the slot is inside `items`.
    unsafe { *dict.items.add(dict.size) = KeyValuePair { key, value } };
    dict.size += 1;
}

/// A copy of `str` in `arena`, NUL-terminated. The empty string is a shared
/// literal rather than an allocation — but only when there is an arena to
/// outlive it; without one the caller frees what it gets.
pub(crate) unsafe fn arena_string(arena: *mut Arena, str: String_0) -> String_0 {
    // SAFETY: `str` has `size` readable bytes.
    unsafe {
        if !str.is_empty() {
            return String_0::from_raw_parts(
                arena_memdupz(arena, str.data(), str.len()),
                str.len(),
            );
        }
        let empty = if arena.is_null() {
            xstrdup(c"".as_ptr())
        } else {
            c"".as_ptr() as *mut c_char
        };
        String_0::from_raw_parts(empty, 0)
    }
}

/// Move a builder's items into an arena-allocated array of exactly the right
/// size, freeing the builder's own buffer if it had grown onto the heap.
pub(crate) unsafe fn arena_take_arraybuilder(arena: *mut Arena, arr: *mut ArrayBuilder) -> Array {
    // SAFETY: `arr` is the caller's builder, live for the call.
    unsafe {
        let mut items = InitVec::new(
            &mut (*arr).size,
            &mut (*arr).capacity,
            &mut (*arr).items,
            &mut (*arr).init_array,
        );
        let mut ret = arena_array(arena, items.len());
        ret.size = items.len();
        memcpy(
            ret.items.cast(),
            items.as_slice().as_ptr().cast(),
            size_of::<Object>() * ret.size,
        );
        let heap = items.take_heap();
        xfree(heap);
        ret
    }
}

// -- Freeing ---------------------------------------------------------------

pub(crate) unsafe fn api_free_string(value: String_0) {
    // SAFETY: `value` owns its allocation.
    unsafe { xfree(value.data().cast()) };
}

/// Free `value` and everything below it. Only for objects that were built on
/// the heap; an arena-allocated object is freed with its arena.
pub unsafe fn api_free_object(value: Object) {
    // SAFETY: `value` owns whatever it points at.
    unsafe {
        match value.type_0 {
            kObjectTypeString => api_free_string(value.data.string),
            kObjectTypeArray => api_free_array(value.data.array),
            kObjectTypeDict => api_free_dict(value.data.dict),
            kObjectTypeLuaRef => api_free_luaref(value.data.luaref),
            _ => {}
        }
    }
}

pub(crate) unsafe fn api_free_array(value: Array) {
    // SAFETY: as `api_free_object`.
    unsafe {
        for i in 0..value.size {
            api_free_object(*value.items.add(i));
        }
        xfree(value.items.cast());
    }
}

pub(crate) unsafe fn api_free_dict(value: Dict) {
    // SAFETY: as `api_free_object`.
    unsafe {
        for i in 0..value.size {
            api_free_string((*value.items.add(i)).key);
            api_free_object((*value.items.add(i)).value);
        }
        xfree(value.items.cast());
    }
}

/// Release the Lua references `value` holds, without freeing `value` itself.
/// For arena-allocated objects, whose memory the arena reclaims but whose
/// references the Lua registry does not.
pub(crate) unsafe fn api_luarefs_free_object(value: Object) {
    // SAFETY: `value` owns the references it names.
    unsafe {
        match value.type_0 {
            kObjectTypeLuaRef => api_free_luaref(value.data.luaref),
            kObjectTypeArray => api_luarefs_free_array(value.data.array),
            kObjectTypeDict => api_luarefs_free_dict(value.data.dict),
            _ => {}
        }
    }
}

pub(crate) unsafe fn api_luarefs_free_array(value: Array) {
    // SAFETY: as `api_luarefs_free_object`.
    unsafe {
        for i in 0..value.size {
            api_luarefs_free_object(*value.items.add(i));
        }
    }
}

pub(crate) unsafe fn api_luarefs_free_dict(value: Dict) {
    // SAFETY: as `api_luarefs_free_object`.
    unsafe {
        for i in 0..value.size {
            api_luarefs_free_object((*value.items.add(i)).value);
        }
    }
}

// -- Copying ---------------------------------------------------------------

/// A copy of `str` in `arena`. Unlike [`arena_string`] a null string stays
/// null rather than becoming the empty one.
pub(crate) unsafe fn copy_string(str: String_0, arena: *mut Arena) -> String_0 {
    // SAFETY: `str` is null or has `size` readable bytes.
    unsafe {
        if str.data().is_null() {
            return String_0::NULL;
        }
        String_0::from_raw_parts(arena_memdupz(arena, str.data(), str.len()), str.len())
    }
}

pub(crate) unsafe fn copy_array(array: Array, arena: *mut Arena) -> Array {
    // SAFETY: `array` is live for the call.
    unsafe {
        // Sized for exactly this many items, so it cannot need to grow.
        let mut rv = arena_array(arena, array.size);
        for i in 0..array.size {
            *rv.items.add(i) = copy_object(*array.items.add(i), arena);
        }
        rv.size = array.size;
        rv
    }
}

pub(crate) unsafe fn copy_dict(dict: Dict, arena: *mut Arena) -> Dict {
    // SAFETY: `dict` is live for the call.
    unsafe {
        let mut rv = arena_dict(arena, dict.size);
        for i in 0..dict.size {
            let item = *dict.items.add(i);
            *rv.items.add(i) = key_value_pair {
                // The key's length is re-derived rather than copied, so a
                // key holding a NUL comes back truncated. Upstream's shape.
                key: cstr_as_string(copy_string(item.key, arena).data()),
                value: copy_object(item.value, arena),
            };
        }
        rv.size = dict.size;
        rv
    }
}

/// A deep copy of `obj` in `arena`. Handles and scalars copy as they stand;
/// a Lua reference gets a second registry reference of its own.
pub(crate) unsafe fn copy_object(obj: Object, arena: *mut Arena) -> Object {
    // SAFETY: `obj` is live for the call.
    unsafe {
        match obj.type_0 {
            kObjectTypeString => object {
                type_0: kObjectTypeString,
                data: object_data {
                    string: copy_string(obj.data.string, arena),
                },
            },
            kObjectTypeArray => object {
                type_0: kObjectTypeArray,
                data: object_data {
                    array: copy_array(obj.data.array, arena),
                },
            },
            kObjectTypeDict => object {
                type_0: kObjectTypeDict,
                data: object_data {
                    dict: copy_dict(obj.data.dict, arena),
                },
            },
            kObjectTypeLuaRef => object {
                type_0: kObjectTypeLuaRef,
                data: object_data {
                    luaref: api_new_luaref(obj.data.luaref),
                },
            },
            _ => obj,
        }
    }
}

// -- Metadata --------------------------------------------------------------

/// The arena `api_metadata`'s unpacked tree lives in, kept alive for the
/// process's lifetime because the tree is handed out by reference.
static METADATA_ARENA: GlobalCell<ArenaMem> = GlobalCell::new(ptr::null_mut::<consumed_blk>());

/// The API description, as the `nvim_get_api_info` reply carries it. Unpacked
/// from the blob on first use and then shared.
pub(crate) unsafe fn api_metadata() -> Object {
    static METADATA: GlobalCell<Object> = GlobalCell::new(NIL);
    // SAFETY: the blob is a compile-time constant and a valid msgpack map.
    unsafe {
        if (*METADATA.ptr()).type_0 == kObjectTypeNil {
            let mut arena = ARENA_EMPTY;
            let mut err = Error {
                type_0: kErrorTypeNone,
                msg: ptr::null_mut(),
            };
            METADATA.set(unpack(
                PACKED_API_METADATA.as_ptr() as *mut c_char,
                PACKED_API_METADATA.len(),
                &raw mut arena,
                &raw mut err,
            ));
            if err.type_0 != kErrorTypeNone || (*METADATA.ptr()).type_0 != kObjectTypeDict {
                abort();
            }
            METADATA_ARENA.set(arena_finish(&raw mut arena));
        }
        METADATA.get()
    }
}

/// [`api_metadata`] still packed, for a caller that is going to forward it
/// over the wire unchanged.
pub(crate) fn api_metadata_raw() -> String_0 {
    String_0::from_raw_parts(
        PACKED_API_METADATA.as_ptr() as *mut c_char,
        PACKED_API_METADATA.len(),
    )
}

// -- Object conversion -----------------------------------------------------

/// The name of `t` as the API's documentation and error messages spell it.
pub(crate) fn api_typename(t: ObjectType) -> *mut c_char {
    let name = match t {
        kObjectTypeNil => c"nil",
        kObjectTypeBoolean => c"Boolean",
        kObjectTypeInteger => c"Integer",
        kObjectTypeFloat => c"Float",
        kObjectTypeString => c"String",
        kObjectTypeArray => c"Array",
        kObjectTypeDict => c"Dict",
        kObjectTypeLuaRef => c"Function",
        kObjectTypeBuffer => c"Buffer",
        kObjectTypeWindow => c"Window",
        kObjectTypeTabpage => c"Tabpage",
        _ => unreachable!(),
    };
    name.as_ptr() as *mut c_char
}

/// `obj` as a boolean. An integer is true when nonzero and nil takes
/// `nil_value`; anything else is an error naming `what`.
pub(crate) unsafe fn api_object_to_bool(
    obj: Object,
    what: *const c_char,
    nil_value: bool,
    err: *mut Error,
) -> bool {
    // SAFETY: `obj` is live and `what`/`err` are the caller's.
    unsafe {
        match obj.type_0 {
            kObjectTypeBoolean => obj.data.boolean,
            kObjectTypeInteger => obj.data.integer != 0,
            kObjectTypeNil => nil_value,
            _ => {
                api_err_exp(err, what, c"boolean".as_ptr(), ptr::null());
                false
            }
        }
    }
}

/// `obj` as a highlight group id, defining the group if it was named and does
/// not exist yet. Zero for the empty name and for an id out of range.
pub(crate) unsafe fn object_to_hl_id(obj: Object, what: *const c_char, err: *mut Error) -> c_int {
    // SAFETY: `obj` is live and `what`/`err` are the caller's.
    unsafe {
        match obj.type_0 {
            kObjectTypeString => {
                let str = obj.data.string;
                if !str.is_empty() {
                    syn_check_group(str.data(), str.len())
                } else {
                    0
                }
            }
            kObjectTypeInteger => {
                let id = obj.data.integer as c_int;
                if (1..=highlight_num_groups()).contains(&id) {
                    id
                } else {
                    0
                }
            }
            _ => {
                api_err_invalid(err, c"hl_group".as_ptr(), what, 0, true);
                0
            }
        }
    }
}

/// `kv_push` for a plain kvec, which starts empty and doubles from 8.
unsafe fn push_chunk(msg: &mut HlMessage, chunk: HlMessageChunk) {
    // SAFETY: `items` is null with a zero capacity, or an allocation of
    // `capacity` chunks.
    unsafe {
        if msg.size == msg.capacity {
            msg.capacity = if msg.capacity != 0 {
                msg.capacity * 2
            } else {
                8
            };
            let bytes = size_of::<HlMessageChunk>() * msg.capacity;
            msg.items = xrealloc(msg.items.cast(), bytes).cast();
        }
        *msg.items.add(msg.size) = chunk;
        msg.size += 1;
    }
}

/// Parse `[[text, hl], …]` — the shape `nvim_echo` and friends take — into a
/// highlighted message. Empty, with `err` set, on the first bad chunk.
pub(crate) unsafe fn parse_hl_msg(chunks: Array, is_err: bool, err: *mut Error) -> HlMessage {
    // SAFETY: `chunks` is live for the call and `err` is the caller's.
    unsafe {
        let mut hl_msg = EMPTY_HL_MESSAGE;
        for i in 0..chunks.size {
            let item = *chunks.items.add(i);
            if item.type_0 != kObjectTypeArray {
                api_err_exp(
                    err,
                    c"chunk".as_ptr(),
                    api_typename(kObjectTypeArray),
                    api_typename(item.type_0),
                );
                hl_msg_free(hl_msg);
                return EMPTY_HL_MESSAGE;
            }
            let chunk = item.data.array;
            if !((1..=2).contains(&chunk.size) && (*chunk.items).type_0 == kObjectTypeString) {
                let msg = c"Invalid chunk: expected Array with 1 or 2 Strings".as_ptr();
                api_set_error(err, kErrorTypeValidation, c"%s".as_ptr(), msg);
                hl_msg_free(hl_msg);
                return EMPTY_HL_MESSAGE;
            }
            // Heap-allocated: the message outlives the caller's arena.
            let text = copy_string((*chunk.items).data.string, ptr::null_mut());
            let hl_id = if chunk.size == 2 {
                object_to_hl_id(*chunk.items.add(1), c"text highlight".as_ptr(), err)
            } else if is_err {
                HLF_E
            } else {
                0
            };
            push_chunk(&mut hl_msg, HlMessageChunk { text, hl_id });
        }
        hl_msg
    }
}
