//! The `Object` tree: what it owns, and how it is released and copied.
//!
//! An object owns its whole tree -- its string's bytes, its array's and
//! dictionary's elements, its Lua registry reference -- so the three
//! families this file used to hold (`arena_*` to build, `api_free_*` to
//! take apart, `api_luarefs_free_*` to catch the references an arena knew
//! nothing about) collapse into [`Object`]'s own `Drop` and `Clone`.
//!
//! Those two `impl`s live here rather than beside the type because `types/`
//! forbids `unsafe` and both have to reach a `ManuallyDrop`.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::api::private::metadata::PACKED_API_METADATA;
use crate::api::private::validate::{err_bad_value, err_expected};
use crate::cstr;
use crate::global_cell::GlobalCell;
use crate::highlight_group::{HLF_E, highlight_num_groups, syn_check_group};
use crate::lua::executor::{api_free_luaref, api_new_luaref};
use crate::memory::xrealloc;
use crate::msgpack_rpc::unpacker::unpack;
use crate::narrow::number_as_int;
use crate::types::{
    ApiDict, Array, Error, HlMessage, HlMessageChunk, LuaRef, Object, ObjectType, String_0,
    kObjectTypeArray, kObjectTypeBoolean, kObjectTypeBuffer, kObjectTypeDict, kObjectTypeFloat,
    kObjectTypeInteger, kObjectTypeLuaRef, kObjectTypeNil, kObjectTypeString, kObjectTypeTabpage,
    kObjectTypeWindow,
};
use ::libc::abort;
use core::ffi::{CStr, c_char, c_int};
use core::mem::ManuallyDrop;

// -- Ownership -------------------------------------------------------------

impl Drop for Object {
    /// Release the value and everything below it.
    ///
    /// The walk is iterative: an `Object` tree comes off the wire and can be
    /// nested as deeply as a caller cares to nest it, and a recursive
    /// destructor would meet the stack before it met the end of the tree.
    /// The payloads are `ManuallyDrop` so that this is the *only* release
    /// path -- see the note on [`Object`].
    fn drop(&mut self) {
        release(self);
    }
}

/// [`Object`]'s destructor, out of line so the drop shim stays a tail call.
fn release(root: &mut Object) {
    // Nothing is allocated for a scalar or a string: a `Vec` that is never
    // pushed to never reaches the allocator.
    let mut pending: Vec<Object> = Vec::new();
    let mut current = ManuallyDrop::new(root.take());
    loop {
        // SAFETY (every arm): `current` is a value this walk owns and
        // will not look at again -- the next statement overwrites it -- so
        // taking each payload out of it happens exactly once.
        match &mut *current {
            Object::String(str) => unsafe { ManuallyDrop::drop(str) },
            Object::LuaRef(reference) => unsafe { api_free_luaref(*reference) },
            Object::Array(array) => {
                let items = unsafe { ManuallyDrop::take(array) };
                pending.extend(items);
            }
            Object::Dict(dict) => {
                let entries = unsafe { ManuallyDrop::take(dict) };
                pending.extend(entries.into_vec().into_iter().map(|pair| pair.value));
            }
            _ => {}
        }
        match pending.pop() {
            Some(next) => current = ManuallyDrop::new(next),
            None => return,
        }
    }
}

impl Clone for Object {
    /// A deep copy. Handles and scalars copy as they stand; a Lua reference
    /// gets a second registry reference of its own.
    fn clone(&self) -> Self {
        match self {
            Object::Nil => Object::Nil,
            Object::Boolean(on) => Object::Boolean(*on),
            Object::Integer(number) => Object::Integer(*number),
            Object::Float(number) => Object::Float(*number),
            Object::String(str) => Object::string(String_0::clone(str)),
            Object::Array(array) => Object::array(Array::clone(array)),
            Object::Dict(dict) => Object::dict(ApiDict::clone(dict)),
            // SAFETY: `self` holds a live registry reference, so the state
            // it names is on the registry for the call.
            Object::LuaRef(reference) => Object::LuaRef(unsafe { api_new_luaref(*reference) }),
            Object::Buffer(handle) => Object::Buffer(*handle),
            Object::Window(handle) => Object::Window(*handle),
            Object::Tabpage(handle) => Object::Tabpage(*handle),
        }
    }
}

/// Moving a payload out of an [`Object`].
///
/// An `Object` has a destructor, so it cannot be destructured where it
/// stands; each of these replaces the value with [`Object::Nil`] and takes
/// the payload out of the husk.
impl Object {
    /// The string, if this is one. Anything else answers `None` and is
    /// released.
    pub fn into_string(self) -> Option<String_0> {
        let mut this = ManuallyDrop::new(self);
        match &mut *this {
            // SAFETY: the husk is not looked at again.
            Object::String(str) => Some(unsafe { ManuallyDrop::take(str) }),
            _ => {
                ManuallyDrop::into_inner(this);
                None
            }
        }
    }

    /// The array, if this is one. See [`Object::into_string`].
    pub fn into_array(self) -> Option<Array> {
        let mut this = ManuallyDrop::new(self);
        match &mut *this {
            // SAFETY: the husk is not looked at again.
            Object::Array(array) => Some(unsafe { ManuallyDrop::take(array) }),
            _ => {
                ManuallyDrop::into_inner(this);
                None
            }
        }
    }

    /// The dictionary, if this is one. See [`Object::into_string`].
    pub fn into_dict(self) -> Option<ApiDict> {
        let mut this = ManuallyDrop::new(self);
        match &mut *this {
            // SAFETY: the husk is not looked at again.
            Object::Dict(dict) => Some(unsafe { ManuallyDrop::take(dict) }),
            _ => {
                ManuallyDrop::into_inner(this);
                None
            }
        }
    }

    /// The Lua registry reference, if this is one, which becomes the
    /// caller's to release. See [`Object::into_string`].
    pub fn into_luaref(self) -> Option<LuaRef> {
        let this = ManuallyDrop::new(self);
        match &*this {
            Object::LuaRef(reference) => Some(*reference),
            _ => {
                ManuallyDrop::into_inner(this);
                None
            }
        }
    }
}

// -- Metadata --------------------------------------------------------------

/// The API description, as the `nvim_get_api_info` reply carries it.
/// Unpacked from the blob on first use and then shared.
///
/// The answer is a copy: the tree is a dictionary of a few hundred entries
/// and the caller owns whatever it is given, so handing out the shared one
/// would mean handing out something it must not free.
pub(crate) fn api_metadata() -> Object {
    static METADATA: GlobalCell<Object> = GlobalCell::new(Object::Nil);
    if METADATA.with(Object::is_nil) {
        let blob = PACKED_API_METADATA.as_ptr() as *mut c_char;
        // SAFETY: the blob is a compile-time constant of `len` bytes and a
        // valid msgpack map.
        let unpacked = unsafe { unpack(blob, PACKED_API_METADATA.len()) };
        if !unpacked.as_ref().is_ok_and(|o| o.as_dict().is_some()) {
            // SAFETY: `abort` takes nothing.
            unsafe { abort() };
        }
        METADATA.set(unpacked.expect("the check above accepted a Dict"));
    }
    METADATA.with(Object::clone)
}

/// [`api_metadata`] still packed, for a caller that is going to forward it
/// over the wire unchanged.
pub(crate) fn api_metadata_raw() -> String_0 {
    String_0::from_bytes(PACKED_API_METADATA)
}

// -- Object conversion -----------------------------------------------------

/// The name of `t` as the API's documentation and error messages spell it.
pub(crate) fn api_typename(t: ObjectType) -> &'static CStr {
    match t {
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
    }
}

/// `obj` as a boolean. An integer is true when nonzero and nil takes
/// `nil_value`; anything else refuses, naming `what`.
///
/// # Safety
///
/// `what` must point at a NUL-terminated string.
pub(crate) unsafe fn api_object_to_bool(
    obj: &Object,
    what: *const c_char,
    nil_value: bool,
) -> Result<bool, Error> {
    if let Some(on) = obj.as_boolean() {
        return Ok(on);
    }
    if let Some(number) = obj.as_integer() {
        return Ok(number != 0);
    }
    if obj.is_nil() {
        return Ok(nil_value);
    }
    // SAFETY: the names and values are NUL-terminated strings.
    Err(err_expected(unsafe { cstr::at(what) }, c"boolean", None))
}

/// `obj` as a highlight group id, defining the group if it was named and does
/// not exist yet. Zero for the empty name and for an id out of range.
///
/// # Safety
///
/// `what` must point at a NUL-terminated string.
pub(crate) unsafe fn object_to_hl_id(obj: &Object, what: *const c_char) -> Result<c_int, Error> {
    if let Some(str) = obj.as_string() {
        if str.is_empty() {
            return Ok(0);
        }
        // SAFETY: `str` names its own bytes.
        return Ok(unsafe { syn_check_group(str.data(), str.len()) });
    }
    if let Some(number) = obj.as_integer() {
        let known = highlight_num_groups();
        let id = number_as_int(number);
        return Ok(if (1..=known).contains(&id) { id } else { 0 });
    }
    // SAFETY: the names and values are NUL-terminated strings.
    Err(err_bad_value(c"hl_group", unsafe { cstr::at(what) }))
}

/// `kv_push` for a plain kvec, which starts empty and doubles from 8.
fn push_chunk(msg: &mut HlMessage, chunk: HlMessageChunk) {
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
    // SAFETY: the grow above left room for one more chunk, which is
    // *uninitialised* -- a plain assignment would release whatever the
    // allocator left in the slot's owning string.
    unsafe { msg.items.add(msg.size).write(chunk) };
    msg.size += 1;
}

/// Parse `[[text, hl], …]` — the shape `nvim_echo` and friends take — into
/// `hl_msg`, refusing at the first bad chunk. What it managed to push before
/// refusing stays in `hl_msg`, which the caller owns and frees either way.
pub(crate) fn parse_hl_msg(
    hl_msg: &mut HlMessage,
    chunks: &Array,
    is_err: bool,
) -> Result<(), Error> {
    for item in chunks {
        let Some(chunk) = item.as_array() else {
            let (want, got) = (api_typename(kObjectTypeArray), api_typename(item.kind()));
            return Err(err_expected(c"chunk", want, Some(got)));
        };
        let head = (1..=2).contains(&chunk.len()).then(|| &chunk[0]);
        let Some(text) = head.and_then(Object::as_string) else {
            return Err(Error::validation(
                c"Invalid chunk: expected Array with 1 or 2 Strings",
            ));
        };
        let text = text.clone();
        let hl_id = if chunk.len() == 2 {
            // SAFETY: the name is a NUL-terminated literal.
            unsafe { object_to_hl_id(&chunk[1], c"text highlight".as_ptr()) }?
        } else if is_err {
            HLF_E
        } else {
            0
        };
        push_chunk(hl_msg, HlMessageChunk { text, hl_id });
    }
    Ok(())
}
