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
    Arena, ArenaMem, Array, ArrayBuilder, Boolean, Dict, Error, Float, HlMessage, HlMessageChunk,
    Integer, KeyValuePair, LuaRef, Object, ObjectType, String_0, consumed_blk,
    kErrorTypeValidation, kObjectTypeArray, kObjectTypeBoolean, kObjectTypeBuffer, kObjectTypeDict,
    kObjectTypeFloat, kObjectTypeInteger, kObjectTypeLuaRef, kObjectTypeNil, kObjectTypeString,
    kObjectTypeTabpage, kObjectTypeWindow, key_value_pair, size_t,
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
    // SAFETY: `arr` is the caller's builder, live for the call, and the four
    // fields are its own.
    let mut items = unsafe {
        InitVec::new(
            &mut (*arr).size,
            &mut (*arr).capacity,
            &mut (*arr).items,
            &mut (*arr).init_array,
        )
    };
    let mut ret = arena_array(arena, items.len());
    ret.size = items.len();
    let (dest, src) = (ret.items, items.as_slice().as_ptr());
    // SAFETY: `ret` was sized for exactly this many objects.
    unsafe { memcpy(dest.cast(), src.cast(), size_of::<Object>() * ret.size) };
    // The vector is the builder's inline array or one heap block; only the
    // second has anything to free.
    let heap = items.take_heap();
    // SAFETY: `heap` is null or that block, which nothing names now.
    unsafe { xfree(heap) };
    ret
}

// -- Freeing ---------------------------------------------------------------

pub(crate) unsafe fn api_free_string(value: String_0) {
    // SAFETY: `value` owns its allocation.
    unsafe { xfree(value.data().cast()) };
}

/// Free `value` and everything below it. Only for objects that were built on
/// the heap; an arena-allocated object is freed with its arena.
pub unsafe fn api_free_object(value: Object) {
    // SAFETY (every arm): the tag says which arm of the union is live, and
    // `value` owns whatever it points at.
    match value.type_0 {
        kObjectTypeString => unsafe { api_free_string(value.data.string) },
        kObjectTypeArray => unsafe { api_free_array(value.data.array) },
        kObjectTypeDict => unsafe { api_free_dict(value.data.dict) },
        kObjectTypeLuaRef => unsafe { api_free_luaref(value.data.luaref) },
        _ => {}
    }
}

pub(crate) unsafe fn api_free_array(value: Array) {
    for i in 0..value.size {
        // SAFETY: as `api_free_object`; `i` is below `size`.
        unsafe { api_free_object(*value.items.add(i)) };
    }
    // SAFETY: `items` is the array's own allocation.
    unsafe { xfree(value.items.cast()) };
}

pub(crate) unsafe fn api_free_dict(value: Dict) {
    for i in 0..value.size {
        // SAFETY: as `api_free_object`; `i` is below `size`.
        unsafe {
            let pair = *value.items.add(i);
            api_free_string(pair.key);
            api_free_object(pair.value);
        }
    }
    // SAFETY: `items` is the dictionary's own allocation.
    unsafe { xfree(value.items.cast()) };
}

/// Release the Lua references `value` holds, without freeing `value` itself.
/// For arena-allocated objects, whose memory the arena reclaims but whose
/// references the Lua registry does not.
pub(crate) unsafe fn api_luarefs_free_object(value: Object) {
    // SAFETY (every arm): the tag says which arm of the union is live, and
    // `value` owns the references it names.
    match value.type_0 {
        kObjectTypeLuaRef => unsafe { api_free_luaref(value.data.luaref) },
        kObjectTypeArray => unsafe { api_luarefs_free_array(value.data.array) },
        kObjectTypeDict => unsafe { api_luarefs_free_dict(value.data.dict) },
        _ => {}
    }
}

pub(crate) unsafe fn api_luarefs_free_array(value: Array) {
    for i in 0..value.size {
        // SAFETY: as `api_luarefs_free_object`; `i` is below `size`.
        unsafe { api_luarefs_free_object(*value.items.add(i)) };
    }
}

pub(crate) unsafe fn api_luarefs_free_dict(value: Dict) {
    for i in 0..value.size {
        // SAFETY: as `api_luarefs_free_object`; `i` is below `size`.
        unsafe { api_luarefs_free_object((*value.items.add(i)).value) };
    }
}

// -- Copying ---------------------------------------------------------------

/// A copy of `str` in `arena`. Unlike [`arena_string`] a null string stays
/// null rather than becoming the empty one.
pub(crate) unsafe fn copy_string(str: String_0, arena: *mut Arena) -> String_0 {
    if str.data().is_null() {
        return String_0::NULL;
    }
    // SAFETY: `str` has `size` readable bytes and `arena` is the caller's.
    let copy = unsafe { arena_memdupz(arena, str.data(), str.len()) };
    String_0::from_raw_parts(copy, str.len())
}

pub(crate) unsafe fn copy_array(array: Array, arena: *mut Arena) -> Array {
    // Sized for exactly this many items, so it cannot need to grow.
    let mut rv = arena_array(arena, array.size);
    for i in 0..array.size {
        // SAFETY: `array` is live for the call and `rv` is the same size, so
        // `i` is inside both.
        unsafe { *rv.items.add(i) = copy_object(*array.items.add(i), arena) };
    }
    rv.size = array.size;
    rv
}

pub(crate) unsafe fn copy_dict(dict: Dict, arena: *mut Arena) -> Dict {
    let mut rv = arena_dict(arena, dict.size);
    for i in 0..dict.size {
        // SAFETY: `dict` is live for the call and `rv` is the same size, so
        // `i` is inside both. The key's length is re-derived rather than
        // copied, so a key holding a NUL comes back truncated -- upstream's
        // shape.
        unsafe {
            let item = *dict.items.add(i);
            *rv.items.add(i) = key_value_pair {
                key: cstr_as_string(copy_string(item.key, arena).data()),
                value: copy_object(item.value, arena),
            };
        }
    }
    rv.size = dict.size;
    rv
}

/// A deep copy of `obj` in `arena`. Handles and scalars copy as they stand;
/// a Lua reference gets a second registry reference of its own.
pub(crate) unsafe fn copy_object(obj: Object, arena: *mut Arena) -> Object {
    // SAFETY (every arm): the tag says which arm of the union is live, and
    // `obj` is live for the call.
    match obj.type_0 {
        kObjectTypeString => Object::string(unsafe { copy_string(obj.data.string, arena) }),
        kObjectTypeArray => Object::array(unsafe { copy_array(obj.data.array, arena) }),
        kObjectTypeDict => Object::dict(unsafe { copy_dict(obj.data.dict, arena) }),
        kObjectTypeLuaRef => Object::luaref(unsafe { api_new_luaref(obj.data.luaref) }),
        _ => obj,
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
    if METADATA.with(|m| m.type_0) == kObjectTypeNil {
        let mut arena = ARENA_EMPTY;
        let mut err = Error::none();
        let blob = PACKED_API_METADATA.as_ptr() as *mut c_char;
        let (len, ar, e) = (PACKED_API_METADATA.len(), &raw mut arena, &raw mut err);
        // SAFETY: the blob is a compile-time constant of `len` bytes and a
        // valid msgpack map; `arena` and `err` are this frame's.
        METADATA.set(unsafe { unpack(blob, len, ar, e) });
        if err.is_set() || METADATA.with(|m| m.type_0) != kObjectTypeDict {
            // SAFETY: `abort` takes nothing.
            unsafe { abort() };
        }
        // SAFETY: `arena` is this frame's, and the tree it holds is kept
        // alive by the static below for the life of the process.
        METADATA_ARENA.set(unsafe { arena_finish(&raw mut arena) });
    }
    METADATA.get()
}

/// [`api_metadata`] still packed, for a caller that is going to forward it
/// over the wire unchanged.
pub(crate) fn api_metadata_raw() -> String_0 {
    String_0::from_raw_parts(
        PACKED_API_METADATA.as_ptr() as *mut c_char,
        PACKED_API_METADATA.len(),
    )
}

// -- Reading an object -----------------------------------------------------
//
// The tag decides which arm of the union carries the value, so a reader that
// asks the tag first is safe by construction. Every `if o.type_0 ==
// kObjectTypeX { o.data.x }` in the api/ family was an `unsafe` region
// spelling out this one line, and the transpile has hundreds of them.

// `then_some` would read the union arm *before* the tag was consulted, which
// is the whole bug these readers exist to prevent: a `bool` or a pointer built
// from the bytes of another arm is not merely wrong, it is undefined. The
// laziness is load-bearing, so clippy's suggestion is refused here.
#[allow(clippy::unnecessary_lazy_evaluations)]
impl Object {
    /// The boolean payload, when the tag says there is one. A `Boolean`
    /// object only -- an integer that happens to be 0 or 1 is not one, which
    /// is the distinction every `nvim_*` option check makes.
    pub(crate) fn as_boolean(self) -> Option<Boolean> {
        // SAFETY: the tag says the payload is the boolean.
        (self.type_0 == kObjectTypeBoolean).then(|| unsafe { self.data.boolean })
    }

    /// The integer payload, when the tag says there is one. Handles carry
    /// their id in the same arm but under their own tags, so
    /// [`as_handle`](Self::as_handle) is the reader for those.
    pub(crate) fn as_integer(self) -> Option<Integer> {
        // SAFETY: the tag says the payload is the integer.
        (self.type_0 == kObjectTypeInteger).then(|| unsafe { self.data.integer })
    }

    /// The float payload, when the tag says there is one.
    pub(crate) fn as_float(self) -> Option<Float> {
        // SAFETY: the tag says the payload is the float.
        (self.type_0 == kObjectTypeFloat).then(|| unsafe { self.data.floating })
    }

    /// The string payload, when the tag says there is one. Borrowed from
    /// whoever owns the object; this reads the pair, not the bytes.
    pub(crate) fn as_string(self) -> Option<String_0> {
        // SAFETY: the tag says the payload is the string.
        (self.type_0 == kObjectTypeString).then(|| unsafe { self.data.string })
    }

    /// The array payload, when the tag says there is one.
    pub(crate) fn as_array(self) -> Option<Array> {
        // SAFETY: the tag says the payload is the array.
        (self.type_0 == kObjectTypeArray).then(|| unsafe { self.data.array })
    }

    /// The dictionary payload, when the tag says there is one.
    pub(crate) fn as_dict(self) -> Option<Dict> {
        // SAFETY: the tag says the payload is the dictionary.
        (self.type_0 == kObjectTypeDict).then(|| unsafe { self.data.dict })
    }

    /// The Lua registry reference, when the tag says there is one. Still
    /// owned by the object -- taking it over means clearing the object's
    /// copy, as the keyset readers do.
    pub(crate) fn as_luaref(self) -> Option<LuaRef> {
        // SAFETY: the tag says the payload is the reference.
        (self.type_0 == kObjectTypeLuaRef).then(|| unsafe { self.data.luaref })
    }
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
    if let Some(on) = obj.as_boolean() {
        return on;
    }
    if let Some(number) = obj.as_integer() {
        return number != 0;
    }
    if obj.type_0 == kObjectTypeNil {
        return nil_value;
    }
    // SAFETY: the caller's promise about `what` and `err`.
    unsafe { api_err_exp(err, what, c"boolean".as_ptr(), ptr::null()) };
    false
}

/// `obj` as a highlight group id, defining the group if it was named and does
/// not exist yet. Zero for the empty name and for an id out of range.
pub(crate) unsafe fn object_to_hl_id(obj: Object, what: *const c_char, err: *mut Error) -> c_int {
    if let Some(str) = obj.as_string() {
        if str.is_empty() {
            return 0;
        }
        // SAFETY: `str` names its own bytes, per this function's contract.
        return unsafe { syn_check_group(str.data(), str.len()) };
    }
    if let Some(number) = obj.as_integer() {
        let known = highlight_num_groups();
        let id = number as c_int;
        return if (1..=known).contains(&id) { id } else { 0 };
    }
    // SAFETY: the caller's promise about `what` and `err`.
    unsafe { api_err_invalid(err, c"hl_group".as_ptr(), what, 0, true) };
    0
}

/// `kv_push` for a plain kvec, which starts empty and doubles from 8.
unsafe fn push_chunk(msg: &mut HlMessage, chunk: HlMessageChunk) {
    if msg.size == msg.capacity {
        msg.capacity = if msg.capacity != 0 {
            msg.capacity * 2
        } else {
            8
        };
        let bytes = size_of::<HlMessageChunk>() * msg.capacity;
        // SAFETY: `items` is null with a zero capacity, or the allocation
        // this function made last time.
        msg.items = unsafe { xrealloc(msg.items.cast(), bytes) }.cast();
    }
    // SAFETY: the grow above left room for one more chunk.
    unsafe { *msg.items.add(msg.size) = chunk };
    msg.size += 1;
}

/// Parse `[[text, hl], …]` — the shape `nvim_echo` and friends take — into a
/// highlighted message. Empty, with `err` set, on the first bad chunk.
pub(crate) unsafe fn parse_hl_msg(chunks: Array, is_err: bool, err: *mut Error) -> HlMessage {
    let mut hl_msg = EMPTY_HL_MESSAGE;
    for i in 0..chunks.size {
        // SAFETY: `i` is below `size`, so the item is inside `items`.
        let item = unsafe { *chunks.items.add(i) };
        let Some(chunk) = item.as_array() else {
            let (want, got) = (api_typename(kObjectTypeArray), api_typename(item.type_0));
            // SAFETY: `err` is the caller's slot and the names are static.
            unsafe { api_err_exp(err, c"chunk".as_ptr(), want, got) };
            // SAFETY: `hl_msg` is this frame's, and owns its chunks.
            unsafe { hl_msg_free(hl_msg) };
            return EMPTY_HL_MESSAGE;
        };
        // SAFETY: a non-empty array has a first item.
        let head = (1..=2)
            .contains(&chunk.size)
            .then(|| unsafe { *chunk.items });
        let Some(text) = head.and_then(Object::as_string) else {
            let msg = c"Invalid chunk: expected Array with 1 or 2 Strings".as_ptr();
            // SAFETY: `err` is the caller's slot; the format takes the one
            // C string it is given.
            unsafe { api_set_error(err, kErrorTypeValidation, c"%s".as_ptr(), msg) };
            // SAFETY: as above.
            unsafe { hl_msg_free(hl_msg) };
            return EMPTY_HL_MESSAGE;
        };
        // Heap-allocated: the message outlives the caller's arena.
        // SAFETY: `text` names its own bytes.
        let text = unsafe { copy_string(text, ptr::null_mut()) };
        let hl_id = if chunk.size == 2 {
            // SAFETY: the array has two items, and `err` is the caller's.
            unsafe { object_to_hl_id(*chunk.items.add(1), c"text highlight".as_ptr(), err) }
        } else if is_err {
            HLF_E
        } else {
            0
        };
        // SAFETY: `hl_msg` is this frame's growable vector.
        unsafe { push_chunk(&mut hl_msg, HlMessageChunk { text, hl_id }) };
    }
    hl_msg
}
