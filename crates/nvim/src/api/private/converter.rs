//! Between a Vimscript [`TypVal`] and an API [`Object`], in both directions.
//!
//! Out is `impl From<&TypVal> for Object`, whose body is one [`TypvalSink`]
//! replacing the `TYPVAL_ENCODE_NAME object` instantiation of
//! `typval_encode.c.h` — the seventh and last of them. Back in is
//! `impl From<Object> for TypVal`, which needs no walk at all: an `Object`
//! tree is finite and shallow enough to recurse over.
//!
//! The sink assembles its answer on a stack of half-built `Object`s, one
//! entry per open container plus the value being converted. A container is
//! sized up front — `conv_list_start` reserves exactly the list's length —
//! and every item is pushed into the reservation, so a container that ends
//! up a different size than the walk announced is a `debug_assert`, not a
//! write past an allocation.
//!
//! **Which direction owns what.** An `Object` owns its tree, so converting
//! *from* a borrowed value copies (a Lua reference is retained) and
//! converting *from* an owned one moves — which is what upstream's
//! `object_to_vim_take_luaref` flag decided by hand, and the reason there is
//! no flag here.
//!
//! It does **not** read `{_TYPE, _VAL}` special dictionaries: an API client
//! gets the two-key dictionary as it stands.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The `kObjectType*` constants keep upstream's spelling; upper-casing them is
// a per-module rewrite.
#![allow(non_upper_case_globals)]

use core::ffi::{CStr, c_char, c_int, c_void};

use crate::eval::decode::decode_string;
use crate::eval::typval::{
    DictSlot, TV_INITIAL_VALUE, tv_dict_add, tv_dict_alloc, tv_dict_item_alloc, tv_list_alloc,
    tv_list_append_owned_tv,
};
use crate::eval::typval_encode::{ConvPath, ConvType, Flow, TypvalSink, encode_typval_read};
use crate::eval::userfunc::FuncFlags;
use crate::eval::userfunc::{find_func, register_luafunc};
use crate::lua::executor::api_new_luaref;
use crate::memory::xstrdup;
use crate::types::{
    ApiDict, Array, Blob, BoolVarValue, DictItem, Float, Integer, KeyValuePair, Object, String_0,
    TypVal, kBoolVarFalse, kBoolVarTrue, kObjectTypeArray, kObjectTypeBoolean, kObjectTypeBuffer,
    kObjectTypeDict, kObjectTypeFloat, kObjectTypeInteger, kObjectTypeLuaRef, kObjectTypeNil,
    kObjectTypeString, kObjectTypeTabpage, kObjectTypeWindow, kSpecialVarNull, size_t,
};

/// The key a dictionary entry gets when its key did not convert to a string.
/// Unreachable through the walk, whose dictionary keys are always strings, but
/// upstream writes it rather than assert, so it is here too.
const INVALID_KEY: &CStr = c"__INVALID_KEY__";

/// The `From<&TypVal> for Object` sink: upstream's `EncodedData`.
#[derive(Default)]
struct ObjectSink {
    /// Containers already opened, innermost last, with the value most recently
    /// converted on top of them.
    stack: Vec<Object>,
    /// The key `conv_dict_after_key` converted, waiting for its value.
    pending_key: Option<String_0>,
}

impl ObjectSink {
    /// A string object over `len` bytes at `data`, copied.
    ///
    /// # Safety
    /// `data` must point at `len` readable bytes.
    unsafe fn cbuf_to_obj(data: *const c_char, len: size_t) -> Object {
        // SAFETY: the caller's promise.
        let bytes = unsafe { core::slice::from_raw_parts(data.cast::<u8>(), len) };
        Object::string(String_0::from_bytes(bytes))
    }

    /// Take the value on top of the stack, leaving the container it belongs to
    /// exposed: upstream's `kv_pop(edata->stack)`.
    fn take_top(&mut self) -> Object {
        self.stack.pop().expect("the walk pushed a value")
    }

    /// Move the value on top of the stack into the array below it.
    fn close_list_item(&mut self) {
        let item = self.take_top();
        let Some(Object::Array(array)) = self.stack.last_mut() else {
            unreachable!("the walk is inside an array");
        };
        debug_assert!(array.len() < array.capacity());
        array.push(item);
    }

    /// Move the key and the value on top of the stack into the dictionary
    /// below them.
    fn close_dict_item(&mut self) {
        let value = self.take_top();
        let key = self.pending_key.take().expect("a key precedes its value");
        let Some(Object::Dict(dict)) = self.stack.last_mut() else {
            unreachable!("the walk is inside a dictionary");
        };
        debug_assert!(dict.len() < dict.capacity());
        dict.insert(key, value);
    }
}

impl TypvalSink for ObjectSink {
    const ALLOW_SPECIALS: bool = false;
    const CONVERT_FN_NAME: &'static CStr = c"_typval_encode_object_convert_one_value()";

    fn conv_nil(&mut self, _tv: Option<&mut TypVal>) {
        self.stack.push(Object::Nil);
    }

    fn conv_bool(&mut self, _tv: Option<&mut TypVal>, num: bool) {
        self.stack.push(Object::Boolean(num));
    }

    fn conv_number(&mut self, _tv: Option<&mut TypVal>, num: i64) {
        self.stack.push(Object::Integer(num as Integer));
    }

    fn conv_unsigned_number(&mut self, _tv: Option<&mut TypVal>, num: u64) {
        self.stack.push(Object::Integer(num.cast_signed()));
    }

    fn conv_float(&mut self, _tv: Option<&mut TypVal>, flt: Float) -> Flow {
        self.stack.push(Object::Float(flt));
        Flow::Go
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_string`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_string(
        &mut self,
        _tv: Option<&mut TypVal>,
        buf: *mut c_char,
        len: size_t,
    ) -> Flow {
        debug_assert!(len == 0 || !buf.is_null());
        // SAFETY: the walk hands over `len` readable bytes.
        let obj = unsafe { Self::cbuf_to_obj(if len != 0 { buf } else { c"".as_ptr() }, len) };
        self.stack.push(obj);
        Flow::Go
    }

    /// An `ext` value has no API image, so it comes out as nil — and falling
    /// through leaves its buffer for the walk to free.
    ///
    /// # Safety
    ///
    /// As [`TypvalSink::conv_ext_string`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_ext_string(
        &mut self,
        _tv: Option<&mut TypVal>,
        _buf: *mut c_char,
        _len: size_t,
        _ext_type: i8,
    ) -> Flow {
        self.stack.push(Object::Nil);
        Flow::Go
    }

    /// A blob is bytes, and so is a `String` object.
    ///
    /// # Safety
    ///
    /// As [`TypvalSink::conv_blob`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_blob(&mut self, _tv: Option<&mut TypVal>, blob: *const Blob, len: c_int) {
        let len = usize::try_from(len).expect("a blob length is never negative");
        // SAFETY: a non-empty blob has a `bv_ga` holding `len` bytes.
        let obj = unsafe {
            let data = if len != 0 {
                (*blob).bv_ga.ga_data.cast::<c_char>()
            } else {
                c"".as_ptr()
            };
            Self::cbuf_to_obj(data, len)
        };
        self.stack.push(obj);
    }

    /// A funcref that is really a Lua function goes back as a `LuaRef`;
    /// anything else is nil.  Either way the walk stops here, so a partial's
    /// arguments and self dictionary are never visited.
    ///
    /// # Safety
    ///
    /// As [`TypvalSink::conv_func_start`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_func_start(
        &mut self,
        _tv: Option<&mut TypVal>,
        fun: *mut c_char,
        _prefix: &'static CStr,
        _path: &ConvPath,
    ) -> Flow {
        // SAFETY: `fun` is NULL or a NUL-terminated function name.
        let luaref = unsafe {
            let fp = if fun.is_null() {
                ::core::ptr::null_mut()
            } else {
                find_func(fun)
            };
            if fp.is_null() || !(*fp).uf_flags.has(FuncFlags::LUAREF) {
                None
            } else {
                Some(api_new_luaref((*fp).uf_luaref))
            }
        };
        self.stack.push(match luaref {
            Some(luaref) => Object::LuaRef(luaref),
            None => Object::Nil,
        });
        Flow::Stop
    }

    fn conv_empty_list(&mut self, _tv: Option<&mut TypVal>) {
        self.stack.push(Object::array(Array::EMPTY));
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_empty_dict`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_empty_dict(&mut self, _dictp: Option<DictSlot>) {
        self.stack.push(Object::dict(ApiDict::EMPTY));
    }

    /// Reserve the whole array now; the items push into the reservation.
    fn conv_list_start(&mut self, _tv: Option<&mut TypVal>, len: c_int) -> Flow {
        let len = usize::try_from(len).expect("a list length is never negative");
        self.stack.push(Object::array(Array::with_capacity(len)));
        Flow::Go
    }

    fn conv_list_between_items(&mut self, _tv: Option<&mut TypVal>) {
        self.close_list_item();
    }

    fn conv_list_end(&mut self, _tv: Option<&mut TypVal>) {
        self.close_list_item();
        debug_assert!(matches!(
            self.stack.last(),
            Some(Object::Array(a)) if a.len() == a.capacity()
        ));
    }

    fn conv_dict_start(&mut self, _tv: Option<&mut TypVal>, len: size_t) -> Flow {
        self.stack.push(Object::dict(ApiDict::with_capacity(len)));
        Flow::Go
    }

    /// The key waits in the sink until its value arrives.
    ///
    /// # Safety
    ///
    /// As [`TypvalSink::conv_dict_after_key`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_dict_after_key(&mut self, _dictp: Option<DictSlot>) {
        let key = self.take_top();
        let key = key
            .into_string()
            .unwrap_or_else(|| String_0::from_cstr(INVALID_KEY));
        debug_assert!(self.pending_key.is_none());
        self.pending_key = Some(key);
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_dict_between_items`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_dict_between_items(&mut self, _dictp: Option<DictSlot>) {
        self.close_dict_item();
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_dict_end`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_dict_end(&mut self, _dictp: Option<DictSlot>) {
        self.close_dict_item();
        debug_assert!(matches!(
            self.stack.last(),
            Some(Object::Dict(d)) if d.len() == d.capacity()
        ));
    }

    /// An `Object` tree is acyclic, so a container that references itself
    /// cannot be represented: the second sighting becomes nil.
    ///
    /// # Safety
    ///
    /// As [`TypvalSink::conv_recurse`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_recurse(
        &mut self,
        _val: *mut c_void,
        _conv_type: ConvType,
        _path: &ConvPath,
    ) -> Flow {
        self.stack.push(Object::Nil);
        Flow::Go
    }
}

impl From<&TypVal> for Object {
    /// A Vimscript value as an API object, copied: the answer owns its own
    /// strings, arrays and dictionaries, and a funcref that is a Lua
    /// function comes back as a second registry reference.
    fn from(value: &TypVal) -> Self {
        vim_to_object(value)
    }
}

impl From<TypVal> for Object {
    /// [`From<&TypVal>`](Object::from), releasing the value afterwards.
    ///
    /// There is nothing an owned Vimscript value can hand over rather than
    /// copy: a `List` and a `Dict` are refcounted containers the API has no
    /// image of, and a `String`'s bytes would have to be re-terminated.
    fn from(value: TypVal) -> Self {
        vim_to_object(&value)
    }
}

/// Convert a Vimscript value to an API `Object`, recursively.
fn vim_to_object(value: &TypVal) -> Object {
    let mut sink = ObjectSink::default();
    // SAFETY: the caller's typval, walked by a sink that cannot fail on any
    // value a live one can hold.
    let name = c"vim_to_object argument";
    let converted = unsafe { encode_typval_read(&mut sink, value, name) };
    debug_assert!(converted);
    debug_assert!(sink.stack.len() == 1);
    // Only a `VAR_UNKNOWN` leaves the stack empty, which upstream calls
    // impossible and then reads its stack's uninitialised first slot for.
    sink.stack.pop().unwrap_or(Object::Nil)
}

impl From<Object> for TypVal {
    /// An API object as a Vimscript value, **taking** what the object owns:
    /// every Lua reference below it moves into the funcref that names it,
    /// rather than a second reference being made.
    fn from(value: Object) -> Self {
        let mut tv = TV_INITIAL_VALUE;
        object_to_vim(value, &mut tv, true);
        tv
    }
}

impl From<&Object> for TypVal {
    /// [`From<Object>`](TypVal::from) over a borrow: the object keeps its
    /// Lua references and the answer gets its own.
    fn from(value: &Object) -> Self {
        let mut tv = TV_INITIAL_VALUE;
        object_to_vim(value.clone(), &mut tv, false);
        tv
    }
}

/// The shared body of the two conversions above.
///
/// `take_luaref` says whether a `LuaRef` arm hands its registry reference to
/// the funcref it becomes (the owned direction) or lends it (the borrowed
/// direction, where `value` is already a clone and the clone's reference is
/// the one being handed over -- so the flag is `false` and the clone's
/// reference is released with it).
fn object_to_vim(value: Object, tv: &mut TypVal, take_luaref: bool) {
    match value.kind() {
        kObjectTypeNil => tv.write_special(kSpecialVarNull),
        kObjectTypeBoolean => {
            let on = value.as_boolean().expect("the tag says Boolean");
            tv.write_boolean(if on { kBoolVarTrue } else { kBoolVarFalse } as BoolVarValue);
        }
        // A handle is an integer with a wire type of its own; Vimscript has
        // no separate notion of one.
        kObjectTypeInteger => tv.write_number(value.as_integer().expect("the tag says Integer")),
        kObjectTypeBuffer | kObjectTypeWindow | kObjectTypeTabpage => {
            tv.write_number(value.as_handle().expect("the tag says a handle"));
        }
        kObjectTypeFloat => tv.write_float(value.as_float().expect("the tag says Float")),
        kObjectTypeString => {
            let str = value.into_string().expect("the tag says String");
            // SAFETY: the string names `len` readable bytes.
            *tv = unsafe { decode_string(str.data(), str.len(), false, false) };
        }
        kObjectTypeArray => {
            let array = value.into_array().expect("the tag says Array");
            let list = tv_list_alloc(array.len().cast_signed());
            let into = list.as_ptr();
            for item in array {
                let mut li_tv: TypVal = TV_INITIAL_VALUE;
                object_to_vim(item, &mut li_tv, take_luaref);
                // SAFETY: `into` is the list just allocated, and `li_tv` is
                // this frame's.
                unsafe { tv_list_append_owned_tv(into, li_tv) };
            }
            tv.write_list(Some(list));
        }
        kObjectTypeDict => {
            let pairs = value.into_dict().expect("the tag says Dict");
            let dict_held = tv_dict_alloc();
            let dict = dict_held.as_ptr();
            for KeyValuePair { key, value } in pairs {
                // SAFETY: a key is a NUL-terminated name, and `di` is the
                // item just allocated for it.
                unsafe {
                    let di: *mut DictItem = tv_dict_item_alloc(key.data());
                    object_to_vim(value, &mut (*di).di_tv, take_luaref);
                    let _ = tv_dict_add(dict, di);
                }
            }
            tv.write_dict(Some(dict_held));
        }
        kObjectTypeLuaRef => {
            let reference = if take_luaref {
                // The object gives its reference up rather than lending it.
                value.into_luaref().expect("the tag says LuaRef")
            } else {
                let borrowed = value.as_luaref().expect("the tag says LuaRef");
                // SAFETY: a registry index, not a pointer.
                unsafe { api_new_luaref(borrowed) }
            };
            // SAFETY: `register_luafunc` answers a NUL-terminated name owned
            // by the registry.
            let name = unsafe { register_luafunc(reference) };
            // SAFETY: `name` is that name.
            tv.write_func_name(unsafe { xstrdup(name) });
        }
        // `kind()` answers one of the eleven above.
        _ => unreachable!("an Object carries one of the eleven tags"),
    }
}
