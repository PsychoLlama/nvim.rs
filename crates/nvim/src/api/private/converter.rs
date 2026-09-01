//! Between a Vimscript `typval_T` and an API `Object`, in both directions.
//!
//! Out is [`vim_to_object`], one [`TypvalSink`] replacing the
//! `TYPVAL_ENCODE_NAME object` instantiation of `typval_encode.c.h` — the
//! seventh and last of them.  Back in is [`object_to_vim`], which needs no
//! walk at all: an `Object` tree is finite and shallow enough to recurse over.
//!
//! The sink assembles its answer on a stack of half-built `Object`s, one entry
//! per open container plus the value being converted.  Sizing is decided *up
//! front*: `conv_list_start` takes an array of exactly the list's length out of
//! the arena, and every item is written into a slot that already exists.  That
//! is why the hooks assert `size < capacity` on the way in and `size ==
//! capacity` on the way out — a mismatch means the walk and the arena disagree
//! about how many items a container has, which the type system cannot catch.
//!
//! It does **not** read `{_TYPE, _VAL}` special dictionaries: an API client
//! gets the two-key dictionary as it stands.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};

use crate::api::private::helpers::{arena_array, arena_dict, arena_string};
use crate::eval::decode::decode_string;
use crate::eval::typval::{
    tv_dict_add, tv_dict_alloc, tv_dict_item_alloc, tv_list_alloc, tv_list_append_owned_tv,
    tv_list_ref,
};
use crate::eval::typval_encode::{
    ConvPath, ConvType, Flow, InlineStack, TypvalSink, encode_typval,
};
use crate::eval::userfunc::{find_func, register_luafunc};
use crate::lua::executor::api_new_luaref;
use crate::memory::xstrdup;
use crate::types::{
    Arena, Array, BoolVarValue, Dict, Float, Integer, KeyValuePair, LuaRef, Object, String_0,
    VAR_BOOL, VAR_DICT, VAR_FLOAT, VAR_FUNC, VAR_LIST, VAR_NUMBER, VAR_SPECIAL, VAR_UNKNOWN,
    VarLock, blob_T, dict_T, dictitem_T, float_T, int64_t, kBoolVarFalse, kBoolVarTrue,
    kSpecialVarNull, list_T, ptrdiff_t, size_t, typval_T, typval_vval_union,
};
use crate::winlayer::Live;

/// `FC_LUAREF`: the `ufunc_T` flag that marks a funcref which is really a Lua
/// function, and so can go back to the API as a `LuaRef`.
const FC_LUAREF: c_int = 0x800;

/// `LUA_NOREF`.
const LUA_NOREF: LuaRef = -2;

/// The key a dictionary entry gets when its key did not convert to a string.
/// Unreachable through the walk, whose dictionary keys are always strings, but
/// upstream writes it rather than assert, so it is here too.
const INVALID_KEY: &CStr = c"__INVALID_KEY__";

/// How many half-built objects fit without allocating.
///
/// Upstream says two, which covers a scalar and a flat container and nothing
/// else: a list of dictionaries -- the shape most of the API deals in -- is
/// already deeper than that, so *every* conversion of one spills to the heap.
/// Eight is 256 bytes of stack for a function that does not recurse, and it
/// keeps the whole walk of an ordinary value inline.
const INLINE_OBJECTS: usize = 8;

/// The `vim_to_object()` sink: upstream's `EncodedData`.
struct ObjectSink {
    /// Containers already opened, innermost last, with the value most recently
    /// converted on top of them.
    stack: InlineStack<Object, INLINE_OBJECTS>,
    /// Where strings and item arrays come from.  May be null, in which case
    /// the allocations are plain `xmalloc`.
    arena: *mut Arena,
    /// Point the answer's strings straight at the typval's own bytes instead
    /// of copying them into the arena.  Only safe where the answer does not
    /// outlive the value it came from.
    reuse_strdata: bool,
}

impl ObjectSink {
    /// A string object over `len` bytes at `data`.
    ///
    /// # Safety
    /// `data` must point at `len` readable bytes.
    unsafe fn cbuf_to_obj(&mut self, data: *const c_char, len: size_t) -> Object {
        let string = if self.reuse_strdata {
            String_0::from_raw_parts(if len != 0 { data } else { c"".as_ptr() }.cast_mut(), len)
        } else {
            unsafe { arena_string(self.arena, String_0::from_raw_parts(data.cast_mut(), len)) }
        };
        Object::String(string)
    }

    /// Take the value on top of the stack, leaving the container it belongs to
    /// exposed: upstream's `kv_pop(edata->stack)`.
    fn take_top(&mut self) -> Object {
        let top = self.stack.last();
        self.stack.pop();
        top
    }

    /// Move the value on top of the stack into the next free slot of the array
    /// below it.
    fn close_list_item(&mut self) {
        let item = self.take_top();
        let Object::Array(array) = self.stack.last_mut() else {
            unreachable!("the walk is inside an array");
        };
        debug_assert!(array.size < array.capacity);
        // SAFETY: `conv_list_start` sized the array for every item the walk
        // will hand over, so the next slot is inside it.
        unsafe {
            *array.items.add(array.size) = item;
        }
        array.size += 1;
    }

    /// The dictionary the walk is currently filling.
    ///
    /// # Safety
    /// The caller must be inside a dictionary, which every use here is.
    unsafe fn open_dict(&mut self) -> &mut Dict {
        let Object::Dict(dict) = self.stack.last_mut() else {
            unreachable!("the walk is inside a dictionary");
        };
        debug_assert!(dict.size < dict.capacity);
        dict
    }
}

impl TypvalSink for ObjectSink {
    const ALLOW_SPECIALS: bool = false;
    const CONVERT_FN_NAME: &'static CStr = c"_typval_encode_object_convert_one_value()";

    unsafe fn conv_nil(&mut self, _tv: *mut typval_T) {
        self.stack.push(Object::Nil);
    }

    unsafe fn conv_bool(&mut self, _tv: *mut typval_T, num: bool) {
        self.stack.push(Object::Boolean(num));
    }

    unsafe fn conv_number(&mut self, _tv: *mut typval_T, num: int64_t) {
        self.stack.push(Object::Integer(num as Integer));
    }

    unsafe fn conv_unsigned_number(&mut self, _tv: *mut typval_T, num: u64) {
        self.stack.push(Object::Integer(num as Integer));
    }

    unsafe fn conv_float(&mut self, _tv: *mut typval_T, flt: float_T) -> Flow {
        self.stack.push(Object::Float(flt as Float));
        Flow::Go
    }

    unsafe fn conv_string(&mut self, _tv: *mut typval_T, buf: *mut c_char, len: size_t) -> Flow {
        debug_assert!(len == 0 || !buf.is_null());
        // SAFETY: the walk hands over `len` readable bytes.
        let obj = unsafe { self.cbuf_to_obj(buf, len) };
        self.stack.push(obj);
        Flow::Go
    }

    /// An `ext` value has no API image, so it comes out as nil — and falling
    /// through leaves its buffer for the walk to free.
    unsafe fn conv_ext_string(
        &mut self,
        _tv: *mut typval_T,
        _buf: *mut c_char,
        _len: size_t,
        _ext_type: i8,
    ) -> Flow {
        self.stack.push(Object::Nil);
        Flow::Go
    }

    /// A blob is bytes, and so is a `String` object.
    unsafe fn conv_blob(&mut self, _tv: *mut typval_T, blob: *const blob_T, len: c_int) {
        let len = len as size_t;
        // SAFETY: a non-empty blob has a `bv_ga` holding `len` bytes.
        let obj = unsafe {
            let data = if len != 0 {
                (*blob).bv_ga.ga_data.cast::<c_char>()
            } else {
                c"".as_ptr()
            };
            self.cbuf_to_obj(data, len)
        };
        self.stack.push(obj);
    }

    /// A funcref that is really a Lua function goes back as a `LuaRef`;
    /// anything else is nil.  Either way the walk stops here, so a partial's
    /// arguments and self dictionary are never visited.
    unsafe fn conv_func_start(
        &mut self,
        _tv: *mut typval_T,
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
            if fp.is_null() || (*fp).uf_flags & FC_LUAREF == 0 {
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

    unsafe fn conv_empty_list(&mut self, _tv: *mut typval_T) {
        self.stack.push(Object::Array(Array::EMPTY));
    }

    unsafe fn conv_empty_dict(&mut self, _tv: *mut typval_T, _dictp: Option<*mut *mut dict_T>) {
        self.stack.push(Object::Dict(Dict::EMPTY));
    }

    /// Reserve the whole array now; the items fill it in place.
    unsafe fn conv_list_start(&mut self, _tv: *mut typval_T, len: c_int) -> Flow {
        self.stack
            .push(Object::Array(arena_array(self.arena, len as size_t)));
        Flow::Go
    }

    unsafe fn conv_list_between_items(&mut self, _tv: *mut typval_T) {
        self.close_list_item();
    }

    unsafe fn conv_list_end(&mut self, _tv: *mut typval_T) {
        self.close_list_item();
        debug_assert!(matches!(
            self.stack.last(),
            Object::Array(a) if a.size == a.capacity
        ));
    }

    unsafe fn conv_dict_start(&mut self, _tv: *mut typval_T, len: size_t) -> Flow {
        self.stack.push(Object::Dict(arena_dict(self.arena, len)));
        Flow::Go
    }

    /// The key lands in the next free slot but does not claim it — the value
    /// that follows is what advances `size`.
    unsafe fn conv_dict_after_key(&mut self, _tv: *mut typval_T, _dictp: Option<*mut *mut dict_T>) {
        let key = self.take_top();
        // SAFETY: the walk is inside a dictionary; `key` is the object it just
        // converted, and a `String` object owns its bytes.
        let key = key.as_string().unwrap_or_else(|| {
            String_0::from_raw_parts(INVALID_KEY.as_ptr().cast_mut(), INVALID_KEY.count_bytes())
        });
        unsafe {
            let dict = self.open_dict();
            (*dict.items.add(dict.size)).key = key;
        }
    }

    unsafe fn conv_dict_between_items(
        &mut self,
        _tv: *mut typval_T,
        _dictp: Option<*mut *mut dict_T>,
    ) {
        let value = self.take_top();
        // SAFETY: as `conv_dict_after_key`, whose slot this completes.
        unsafe {
            let dict = self.open_dict();
            (*dict.items.add(dict.size)).value = value;
            dict.size += 1;
        }
    }

    unsafe fn conv_dict_end(&mut self, tv: *mut typval_T, dictp: Option<*mut *mut dict_T>) {
        // SAFETY: as `conv_dict_between_items`.
        unsafe { self.conv_dict_between_items(tv, dictp) };
        debug_assert!(matches!(
            self.stack.last(),
            Object::Dict(d) if d.size == d.capacity
        ));
    }

    /// An `Object` tree is acyclic, so a container that references itself
    /// cannot be represented: the second sighting becomes nil.
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

/// Convert a Vimscript value to an API `Object`, recursively.
///
/// `arena` may be null, in which case the tree is heap-allocated and
/// `api_free_object` takes it apart.  `reuse_strdata` points the answer's
/// strings at `obj`'s own bytes rather than copying them, and takes no effect
/// without an arena.
///
/// # Safety
/// `obj` must point at a live typval, and `arena` be null or a live arena.
pub unsafe fn vim_to_object(obj: *mut typval_T, arena: *mut Arena, reuse_strdata: bool) -> Object {
    let mut sink = ObjectSink {
        stack: InlineStack::new(),
        arena,
        reuse_strdata,
    };
    // SAFETY: the caller's typval, walked by a sink that cannot fail on any
    // value a live one can hold.
    let converted = unsafe { encode_typval(&mut sink, obj, c"vim_to_object argument".as_ptr()) };
    debug_assert!(converted);
    debug_assert!(sink.stack.len() == 1);
    if sink.stack.is_empty() {
        // Only a `VAR_UNKNOWN` gets here, which upstream calls impossible and
        // then reads its stack's uninitialised first slot for.
        return Object::Nil;
    }
    sink.stack.last()
}

/// Convert an API `Object` to a Vimscript value.
///
/// On failure `tv`'s `v_type` is left `VAR_UNKNOWN` and nothing was
/// allocated for it.
///
/// # Safety
/// `tv` must point at writable typval storage.
pub unsafe fn object_to_vim(obj: Object, tv: *mut typval_T) {
    let mut obj = obj;
    unsafe { object_to_vim_take_luaref(&raw mut obj, tv, false) };
}
/// As [`object_to_vim`], but consuming every `LuaRef` nested in `obj`.
///
/// Useful where `obj` sits on an arena, which cannot free the Lua registry
/// references its objects hold.
///
/// Upstream threads an `Error` out-parameter through the recursion and never
/// reads it; there is nothing here that can fail, so this does not take one.
///
/// # Safety
/// As [`object_to_vim`]; `obj` must point at a live object tree.
pub unsafe fn object_to_vim_take_luaref(obj: *mut Object, tv: *mut typval_T, take_luaref: bool) {
    // SAFETY: the caller's promise -- `tv` is writable typval storage and
    // `obj` a live both for the length of the call.
    let mut tv = unsafe { Live::<typval_T>::new(tv) };
    // SAFETY: as above.
    let mut obj = unsafe { Live::<Object>::new(obj) };
    tv.v_type = VAR_UNKNOWN;
    tv.v_lock = VarLock::Unlocked;
    let value = *obj;
    match value {
        Object::Nil => {
            tv.v_type = VAR_SPECIAL;
            tv.vval.v_special = kSpecialVarNull;
        }
        Object::Boolean(on) => {
            tv.v_type = VAR_BOOL;
            tv.vval.v_bool = if on { kBoolVarTrue } else { kBoolVarFalse } as BoolVarValue;
        }
        // A handle is an integer with a wire type of its own; Vimscript has
        // no separate notion of one.
        Object::Buffer(number)
        | Object::Window(number)
        | Object::Tabpage(number)
        | Object::Integer(number) => {
            tv.v_type = VAR_NUMBER;
            tv.vval.v_number = number;
        }
        Object::Float(float) => {
            tv.v_type = VAR_FLOAT;
            tv.vval.v_float = float as float_T;
        }
        Object::String(str) => {
            // SAFETY: the string names `len` readable bytes.
            *tv = unsafe { decode_string(str.data(), str.len(), false, false) };
        }
        Object::Array(array) => {
            // SAFETY: the list is this call's until it is handed to `tv`.
            let list: *mut list_T = unsafe { tv_list_alloc(array.size as ptrdiff_t) };
            for i in 0..array.size {
                let mut li_tv: typval_T = typval_T {
                    v_type: VAR_UNKNOWN,
                    v_lock: VarLock::Unlocked,
                    vval: typval_vval_union { v_number: 0 },
                };
                // SAFETY: `i` is below `size`, so the slot is inside
                // `items`, and `li_tv` is this frame's.
                unsafe {
                    object_to_vim_take_luaref(array.items.add(i), &raw mut li_tv, take_luaref);
                    tv_list_append_owned_tv(list, li_tv);
                }
            }
            // SAFETY: `list` is the list just built.
            unsafe { tv_list_ref(list) };
            tv.v_type = VAR_LIST;
            tv.vval.v_list = list;
        }
        Object::Dict(pairs) => {
            // SAFETY: the dictionary is this call's until it is handed over.
            let dict: *mut dict_T = unsafe { tv_dict_alloc() };
            for i in 0..pairs.size {
                // SAFETY: `i` is below `size`, so the pair is inside `items`,
                // and its key is a NUL-terminated name.
                unsafe {
                    let item: *mut KeyValuePair = pairs.items.add(i);
                    let di: *mut dictitem_T = tv_dict_item_alloc((*item).key.data());
                    let value = &raw mut (*item).value;
                    object_to_vim_take_luaref(value, &raw mut (*di).di_tv, take_luaref);
                    let _ = tv_dict_add(dict, di);
                }
            }
            // SAFETY: `dict` is the dictionary just built; the reference the
            // typval is about to hold is what this counts.
            unsafe { (*dict).dv_refcount.retain() };
            tv.v_type = VAR_DICT;
            tv.vval.v_dict = dict;
        }
        Object::LuaRef(mut ref_0) => {
            if take_luaref {
                // The object gives its reference up rather than lending it.
                *obj = Object::LuaRef(LUA_NOREF);
            } else {
                // SAFETY: a registry index, not a pointer.
                ref_0 = unsafe { api_new_luaref(ref_0) };
            }
            // SAFETY: as above; `register_luafunc` answers a NUL-terminated
            // name owned by the registry.
            let name = unsafe { register_luafunc(ref_0) };
            tv.v_type = VAR_FUNC;
            // SAFETY: `name` is that name.
            tv.vval.v_string = unsafe { xstrdup(name) };
        }
    }
}
