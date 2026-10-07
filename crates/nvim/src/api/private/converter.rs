//! Between a Vimscript [`TypVal`] and an API [`Object`], in both directions.
//!
//! Out is `impl From<&TypVal> for Object`, whose body is one [`TypvalSink`]
//! replacing the `TYPVAL_ENCODE_NAME object` instantiation of
//! `typval_encode.c.h` — the seventh and last of them. Back in is
//! `impl From<Object> for TypVal`, which needs no walk at all: an `Object`
//! tree is finite and shallow enough to recurse over.
//!
//! The sink assembles its answer on a stack of half-built `Object`s, one
//! entry per *open container* — a converted value goes straight into the
//! container below it rather than onto the stack of its own, so a leaf is
//! moved once instead of twice and the walk's "between items" hooks have
//! nothing to do. A container is sized up front — `conv_list_start`
//! reserves exactly the list's length — and every item is pushed into the
//! reservation, so a container that ends up a different size than the walk
//! announced is a `debug_assert`, not a write past an allocation.
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

use core::ffi::{CStr, c_int};

use crate::eval::decode::decode_string;
use crate::eval::typval::{tv_dict_alloc, tv_list_alloc};
use crate::eval::typval_encode::{Container, ConvPath, ConvType, Flow, TypvalSink, encode_typval};
use crate::eval::userfunc::FuncFlags;
use crate::eval::userfunc::{find_func, register_luafunc};
use crate::lua::executor::api_new_luaref;
use crate::types::{
    ApiDict, Array, BoolVarValue, DictItem, DictKey, Error, Float, Integer, KeyValuePair, ListItem,
    Object, String_0, TypVal, kBoolVarFalse, kBoolVarTrue, kObjectTypeArray, kObjectTypeBoolean,
    kObjectTypeBuffer, kObjectTypeDict, kObjectTypeFloat, kObjectTypeInteger, kObjectTypeLuaRef,
    kObjectTypeNil, kObjectTypeString, kObjectTypeTabpage, kObjectTypeWindow, kSpecialVarNull,
};

/// The `From<&TypVal> for Object` sink: upstream's `EncodedData`.
struct ObjectSink {
    /// The containers the walk has opened and not yet closed, innermost
    /// last. A converted value is not on it: it goes straight into the
    /// innermost container, or -- with nothing open -- into [`Self::root`].
    stack: Vec<Object>,
    /// The whole answer, once the walk is done.
    root: Object,
}

impl ObjectSink {
    /// A sink with nothing open and nothing converted.
    fn new() -> Self {
        Self {
            stack: Vec::new(),
            root: Object::Nil,
        }
    }

    /// Put a finished value where it belongs: the next slot of the innermost
    /// open array, the entry the last key opened in the innermost open
    /// dictionary, or the answer itself.
    ///
    /// A dictionary's key claims its entry with a nil value when it
    /// converts, so the value that follows *fills* rather than appends --
    /// which is what keeps a *stack* of half-built entries rather than one
    /// pending key, since a dictionary nested inside another's value would
    /// overwrite a single slot.
    fn emit(&mut self, value: Object) {
        match self.stack.last_mut() {
            Some(Object::Array(array)) => {
                debug_assert!(array.len() < array.capacity());
                array.push(value);
            }
            Some(Object::Dict(dict)) => {
                dict.last_mut().expect("a key precedes its value").value = value;
            }
            Some(_) => unreachable!("only a container is left open"),
            None => self.root = value,
        }
    }

    /// The dictionary the walk is currently filling.
    fn open_dict(&mut self) -> &mut ApiDict {
        let Some(Object::Dict(dict)) = self.stack.last_mut() else {
            unreachable!("the walk is inside a dictionary");
        };
        dict
    }

    /// Close the innermost container and hand it to whatever holds it.
    fn close(&mut self) {
        let done = self.stack.pop().expect("the walk opened a container");
        self.emit(done);
    }
}

impl TypvalSink for ObjectSink {
    const ALLOW_SPECIALS: bool = false;
    const CONVERT_FN_NAME: &'static CStr = c"_typval_encode_object_convert_one_value()";

    fn conv_nil(&mut self) {
        self.emit(Object::Nil);
    }

    fn conv_bool(&mut self, num: bool) {
        self.emit(Object::Boolean(num));
    }

    fn conv_number(&mut self, num: i64) {
        self.emit(Object::Integer(num as Integer));
    }

    fn conv_unsigned_number(&mut self, num: u64) {
        self.emit(Object::Integer(num.cast_signed()));
    }

    fn conv_float(&mut self, flt: Float) -> Flow {
        self.emit(Object::Float(flt));
        Flow::Go
    }

    fn conv_string(&mut self, bytes: &[u8]) -> Flow {
        self.emit(Object::string(String_0::from_bytes(bytes)));
        Flow::Go
    }

    /// An `ext` value has no API image, so it comes out as nil.
    fn conv_ext_string(&mut self, _bytes: &[u8], _ext_type: i8) -> Flow {
        self.emit(Object::Nil);
        Flow::Go
    }

    /// A blob is bytes, and so is a `String` object.
    fn conv_blob(&mut self, bytes: &[u8]) {
        self.emit(Object::string(String_0::from_bytes(bytes)));
    }

    /// A funcref that is really a Lua function goes back as a `LuaRef`;
    /// anything else is nil.  Either way the walk stops here, so a partial's
    /// arguments and self dictionary are never visited.
    fn conv_func_start(
        &mut self,
        fun: Option<&CStr>,
        _prefix: &'static CStr,
        _path: &ConvPath<'_, '_>,
    ) -> Flow {
        let luaref = fun
            .and_then(|fun| find_func(fun.to_bytes()))
            .filter(|func| func.has_flag(FuncFlags::LUAREF))
            .map(|func| api_new_luaref(func.luaref.get()));
        self.emit(match luaref {
            Some(luaref) => Object::LuaRef(luaref),
            None => Object::Nil,
        });
        Flow::Stop
    }

    fn conv_empty_list(&mut self) {
        self.emit(Object::array(Array::EMPTY));
    }

    fn conv_empty_dict(&mut self) {
        self.emit(Object::dict(ApiDict::EMPTY));
    }

    /// Reserve the whole array now; the items push into the reservation.
    fn conv_list_start(&mut self, len: c_int) -> Flow {
        let len = usize::try_from(len).expect("a list length is never negative");
        self.stack.push(Object::array(Array::with_capacity(len)));
        Flow::Go
    }

    /// Nothing: an item goes into the array as it converts.
    fn conv_list_between_items(&mut self) {}

    fn conv_list_end(&mut self) {
        debug_assert!(matches!(
            self.stack.last(),
            Some(Object::Array(a)) if a.len() == a.capacity()
        ));
        self.close();
    }

    fn conv_dict_start(&mut self, len: usize) -> Flow {
        self.stack.push(Object::dict(ApiDict::with_capacity(len)));
        Flow::Go
    }

    /// The key claims its entry, and the value that follows fills it in.
    ///
    /// The key never becomes an `Object`: a [`DictKey`] copies a short one --
    /// which an API key almost always is -- into the entry itself, where a
    /// `String_0` would be an `xmalloc` and an `xfree` per key, and half of
    /// the allocations a conversion makes are keys.
    ///
    fn conv_dict_key(&mut self, key: &[u8]) -> Flow {
        let dict = self.open_dict();
        debug_assert!(dict.len() < dict.capacity());
        dict.insert(DictKey::new(key), Object::Nil);
        Flow::Go
    }

    /// Nothing: [`Self::conv_dict_key`] already claimed the entry, and this
    /// sink refuses specials, so the `[key, value]` pair walk -- the only
    /// other caller -- never runs.
    fn conv_dict_after_key(&mut self) {}

    fn conv_dict_between_items(&mut self) {}

    fn conv_dict_end(&mut self) {
        debug_assert!(matches!(
            self.stack.last(),
            Some(Object::Dict(d)) if d.len() == d.capacity()
        ));
        self.close();
    }

    /// An `Object` tree is acyclic, so a container that references itself
    /// cannot be represented: the second sighting becomes nil.
    fn conv_recurse(
        &mut self,
        _container: Container<'_>,
        _conv_type: ConvType,
        _path: &ConvPath<'_, '_>,
    ) -> Flow {
        self.emit(Object::Nil);
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
    let mut sink = ObjectSink::new();
    // SAFETY: the caller's typval, walked by a sink that cannot fail on any
    // value a live one can hold.
    let name = c"vim_to_object argument";
    let converted = encode_typval(&mut sink, value, name);
    debug_assert!(converted);
    debug_assert!(sink.stack.is_empty());
    // A `VAR_UNKNOWN` emits nothing, which upstream calls impossible and
    // then reads its stack's uninitialised first slot for; the answer is
    // `Object::Nil` here.
    sink.root
}

impl From<Object> for TypVal {
    /// An API object as a Vimscript value, **taking** what the object owns:
    /// every Lua reference below it moves into the funcref that names it,
    /// rather than a second reference being made.
    fn from(value: Object) -> Self {
        object_to_vim(value, true)
    }
}

impl From<&Object> for TypVal {
    /// [`From<Object>`](TypVal::from) over a borrow: the object keeps its
    /// Lua references and the answer gets its own.
    fn from(value: &Object) -> Self {
        object_to_vim(value.clone(), false)
    }
}

/// The shared body of the two conversions above.
///
/// `take_luaref` says whether a `LuaRef` arm hands its registry reference to
/// the funcref it becomes (the owned direction) or lends it (the borrowed
/// direction, where `value` is already a clone and the clone's reference is
/// the one being handed over -- so the flag is `false` and the clone's
/// reference is released with it).
fn object_to_vim(value: Object, take_luaref: bool) -> TypVal {
    match value.kind() {
        kObjectTypeNil => TypVal::Special(kSpecialVarNull),
        kObjectTypeBoolean => {
            let on = value.as_boolean().expect("the tag says Boolean");
            TypVal::Bool(if on { kBoolVarTrue } else { kBoolVarFalse } as BoolVarValue)
        }
        kObjectTypeInteger => TypVal::Number(value.as_integer().expect("the tag says Integer")),
        // A handle is an integer with a wire type of its own; Vimscript has
        // no separate notion of one.
        kObjectTypeBuffer | kObjectTypeWindow | kObjectTypeTabpage => {
            TypVal::Number(value.as_handle().expect("the tag says a handle"))
        }
        kObjectTypeFloat => TypVal::Float(value.as_float().expect("the tag says Float")),
        kObjectTypeString => {
            let str = value.into_string().expect("the tag says String");
            if str.is_null() {
                TypVal::string(None)
            } else {
                decode_string(str.as_bytes(), false)
            }
        }
        kObjectTypeArray => {
            let array = value.into_array().expect("the tag says Array");
            let mut list = tv_list_alloc(array.len().cast_signed());
            for item in array {
                list.lv_items
                    .push(ListItem::new(object_to_vim(item, take_luaref)));
            }
            TypVal::list(Some(list))
        }
        kObjectTypeDict => {
            let pairs = value.into_dict().expect("the tag says Dict");
            let mut dict = tv_dict_alloc();
            for KeyValuePair { key, value } in pairs {
                let item_tv = object_to_vim(value, take_luaref);
                // The key up to its NUL, as the C's `strlen` read it.
                let mut item = DictItem::boxed(key.as_c_str().to_bytes());
                item.di_tv = item_tv;
                let _ = dict.add_item(item);
            }
            TypVal::dict(Some(dict))
        }
        kObjectTypeLuaRef => {
            let reference = if take_luaref {
                // The object gives its reference up rather than lending it.
                value.into_luaref().expect("the tag says LuaRef")
            } else {
                let borrowed = value.as_luaref().expect("the tag says LuaRef");
                api_new_luaref(borrowed)
            };
            TypVal::func(Some(register_luafunc(reference)))
        }
        // `kind()` answers one of the eleven above.
        _ => unreachable!("an Object carries one of the eleven tags"),
    }
}

/// Call the RPC handler in row `row` of the method table with `args` on
/// behalf of Vimscript, and convert its answer to a value before the arena
/// the call allocated its scratch in is released.
pub(crate) fn call_handler_for_vimscript(row: usize, args: Array) -> Result<TypVal, Error> {
    let handler = crate::api::private::dispatch::method_handlers[row];
    let call = handler.fn_0.expect("non-null function pointer");
    let mut arena = crate::memory::ARENA_EMPTY;
    let answer = call(crate::eval::funcs::VIML_INTERNAL_CALL, args, &mut arena).map(TypVal::from);
    // SAFETY: the arena is this call's own, and its answer has been converted
    // out of it above; an error owns its message.
    unsafe { crate::memory::arena_mem_free(crate::memory::arena_finish(&raw mut arena)) };
    answer
}
