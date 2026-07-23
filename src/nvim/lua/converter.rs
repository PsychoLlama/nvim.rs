use crate::src::nvim::api::private::helpers::{
    api_free_array, api_free_dict, api_free_object, api_set_error, api_typename, arena_array,
    arena_dict, arena_string,
};
use crate::src::nvim::eval::decode::{decode_create_map_special_dict, decode_string};
use crate::src::nvim::eval::encode::encode_vim_list_to_buf;
use crate::src::nvim::eval::typval::{
    tv_clear, tv_copy, tv_dict_add, tv_dict_alloc, tv_dict_find, tv_dict_item_alloc_len,
    tv_list_alloc, tv_list_append_list, tv_list_append_owned_tv,
};
use crate::src::nvim::eval::userfunc::{find_func, register_luafunc};
use crate::src::nvim::eval::vars::eval_msgpack_type_lists;
use crate::src::nvim::eval_1::{get_copyID, partial_name};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::highlight_group::syn_check_group;
use crate::src::nvim::lua::executor::{api_free_luaref, nlua_pushref, nlua_ref_global};
use crate::src::nvim::lua::ffi::{
    lua_checkstack, lua_createtable, lua_getmetatable, lua_gettop, lua_next, lua_pushboolean,
    lua_pushinteger, lua_pushlstring, lua_pushnil, lua_pushnumber, lua_pushstring, lua_pushvalue,
    lua_rawequal, lua_rawgeti, lua_rawset, lua_rawseti, lua_setmetatable, lua_settop,
    lua_toboolean, lua_tolstring, lua_tonumber, lua_type,
};
use crate::src::nvim::main::nlua_global_refs;
use crate::src::nvim::memory::{arena_memdupz, xfree, xmalloc, xrealloc, xstrdup};
use crate::src::nvim::message::{emsg, internal_error, semsg};
use crate::src::nvim::os::libc::{__assert_fail, abort, gettext, memchr, memcpy, memset, strlen};
pub use crate::src::nvim::types::{
    blob_T, blobvar_S, dict_T, dictitem_T, dictvar_S, float_T, funccall_S,
    funccall_S_fc_fixvar as C2Rust_Unnamed_0, funccall_T, garray_T, handle_T, hash_T, hashitem_T,
    hashtab_T, int32_t, int64_t, key_value_pair, linenr_T, list_T, listitem_S, listitem_T,
    listvar_S, listwatch_S, listwatch_T, lua_Integer, lua_Number, lua_State, nlua_ref_state_t,
    object, object_data as C2Rust_Unnamed, partial_S, partial_T, proftime_T, ptrdiff_t, queue,
    scid_T, sctx_T, size_t, typval_T, typval_vval_union, ufunc_S, ufunc_T, uint64_t, uint8_t,
    varnumber_T, Arena, Array, BoolVarValue, Boolean, Buffer, Dict, Error, ErrorType, FieldHashfn,
    Float, Integer, KeySetLink, KeyValuePair, LuaRef, MPConvPartialStage, MPConvStack,
    MPConvStackVal, MPConvStackValType, MPConvStackVal_data as C2Rust_Unnamed_1,
    MPConvStackVal_data_a as C2Rust_Unnamed_2, MPConvStackVal_data_d as C2Rust_Unnamed_5,
    MPConvStackVal_data_l as C2Rust_Unnamed_4, MPConvStackVal_data_p as C2Rust_Unnamed_3,
    MessagePackType, Object, ObjectType, OptKeySet, OptionalKeys, ScopeDictDictItem, ScopeType,
    SpecialVarValue, String_0, Tabpage, VarLockStatus, VarType, Window, QUEUE,
};
pub const kErrorTypeValidation: ErrorType = 1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
pub const kObjectTypeTabpage: ObjectType = 10;
pub const kObjectTypeWindow: ObjectType = 9;
pub const kObjectTypeBuffer: ObjectType = 8;
pub const kObjectTypeLuaRef: ObjectType = 7;
pub const kObjectTypeDict: ObjectType = 6;
pub const kObjectTypeArray: ObjectType = 5;
pub const kObjectTypeString: ObjectType = 4;
pub const kObjectTypeFloat: ObjectType = 3;
pub const kObjectTypeInteger: ObjectType = 2;
pub const kObjectTypeBoolean: ObjectType = 1;
pub const kObjectTypeNil: ObjectType = 0;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kSpecialVarNull: SpecialVarValue = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
pub const VAR_BLOB: VarType = 10;
pub const VAR_PARTIAL: VarType = 9;
pub const VAR_SPECIAL: VarType = 8;
pub const VAR_BOOL: VarType = 7;
pub const VAR_FLOAT: VarType = 6;
pub const VAR_DICT: VarType = 5;
pub const VAR_LIST: VarType = 4;
pub const VAR_FUNC: VarType = 3;
pub const VAR_STRING: VarType = 2;
pub const VAR_NUMBER: VarType = 1;
pub const VAR_UNKNOWN: VarType = 0;
pub const kMPConvPartialList: MPConvStackValType = 4;
pub const kMPConvPartial: MPConvStackValType = 3;
pub const kMPConvPairs: MPConvStackValType = 2;
pub const kMPConvList: MPConvStackValType = 1;
pub const kMPConvDict: MPConvStackValType = 0;
pub const kMPConvPartialEnd: MPConvPartialStage = 2;
pub const kMPConvPartialSelf: MPConvPartialStage = 1;
pub const kMPConvPartialArgs: MPConvPartialStage = 0;
pub const kMPExt: MessagePackType = 7;
pub const kMPMap: MessagePackType = 6;
pub const kMPArray: MessagePackType = 5;
pub const kMPString: MessagePackType = 4;
pub const kMPFloat: MessagePackType = 3;
pub const kMPInteger: MessagePackType = 2;
pub const kMPBoolean: MessagePackType = 1;
pub const kMPNil: MessagePackType = 0;
pub type C2Rust_Unnamed_6 = ::core::ffi::c_uint;
pub const kNluaPushFreeRefs: C2Rust_Unnamed_6 = 2;
pub const kNluaPushSpecial: C2Rust_Unnamed_6 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TVPopStackItem {
    pub tv: *mut typval_T,
    pub list_len: size_t,
    pub container: bool,
    pub special: bool,
    pub idx: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_7 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut TVPopStackItem,
    pub init_array: [TVPopStackItem; 2],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LuaTableProps {
    pub maxidx: size_t,
    pub string_keys_num: size_t,
    pub has_string_with_nul: bool,
    pub type_0: ObjectType,
    pub val: lua_Number,
    pub has_type_key: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ObjPopStackItem {
    pub obj: *mut Object,
    pub container: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_8 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ObjPopStackItem,
    pub init_array: [ObjPopStackItem; 2],
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_TNIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const LUA_TBOOLEAN: ::core::ffi::c_int = 1;
pub const LUA_TNUMBER: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const LUA_TSTRING: ::core::ffi::c_int = 4;
pub const LUA_TTABLE: ::core::ffi::c_int = 5;
pub const LUA_TFUNCTION: ::core::ffi::c_int = 6;
pub const LUA_TUSERDATA: ::core::ffi::c_int = 7;
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const INT8_MIN: ::core::ffi::c_int = -128 as ::core::ffi::c_int;
pub const INT64_MIN: ::core::ffi::c_long =
    -9223372036854775807 as ::core::ffi::c_long - 1 as ::core::ffi::c_long;
pub const INT8_MAX: ::core::ffi::c_int = 127 as ::core::ffi::c_int;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
#[inline(always)]
unsafe extern "C" fn _memcpy_free(
    dest: *mut ::core::ffi::c_void,
    src: *mut ::core::ffi::c_void,
    size: size_t,
) -> *mut ::core::ffi::c_void {
    memcpy(dest, src, size);
    let mut ptr_: *mut *mut ::core::ffi::c_void = &raw const src as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    return dest;
}
pub const API_INTEGER_MAX: ::core::ffi::c_long = INT64_MAX;
pub const API_INTEGER_MIN: ::core::ffi::c_long = INT64_MIN;
pub const VARNUMBER_MAX: ::core::ffi::c_long = INT64_MAX;
pub const VARNUMBER_MIN: ::core::ffi::c_long = INT64_MIN;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn tv_list_ref(l: *mut list_T) {
    if l.is_null() {
        return;
    }
    (*l).lv_refcount += 1;
}
#[inline]
unsafe extern "C" fn tv_list_set_copyid(l: *mut list_T, copyid: ::core::ffi::c_int) {
    (*l).lv_copyID = copyid;
}
#[inline]
unsafe extern "C" fn tv_list_len(l: *const list_T) -> ::core::ffi::c_int {
    if l.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*l).lv_len;
}
#[inline]
unsafe extern "C" fn tv_list_copyid(l: *const list_T) -> ::core::ffi::c_int {
    return (*l).lv_copyID;
}
#[inline]
unsafe extern "C" fn tv_list_first(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    return (*l).lv_first;
}
#[inline]
unsafe extern "C" fn tv_list_last(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    return (*l).lv_last;
}
#[inline]
unsafe extern "C" fn tv_blob_len(b: *const blob_T) -> ::core::ffi::c_int {
    if b.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*b).bv_ga.ga_len;
}
#[inline(always)]
unsafe extern "C" fn tv_strlen(tv: *const typval_T) -> size_t {
    '_c2rust_label: {
        if (*tv).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"tv->v_type == VAR_STRING\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/lua/converter.rs\0".as_ptr() as *const ::core::ffi::c_char,
                77 as ::core::ffi::c_uint,
                b"size_t tv_strlen(const typval_T *const)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    return if (*tv).vval.v_string.is_null() {
        0 as size_t
    } else {
        strlen((*tv).vval.v_string)
    };
}
pub const FC_LUAREF: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const TYPE_IDX_VALUE: ::core::ffi::c_int = true_0;
pub const VAL_IDX_VALUE: ::core::ffi::c_int = false_0;
unsafe extern "C" fn nlua_traverse_table(lstate: *mut lua_State) -> LuaTableProps {
    let mut tsize: size_t = 0 as size_t;
    let mut val_type: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut has_val_key: bool = false_0 != 0;
    let mut other_keys_num: size_t = 0 as size_t;
    let mut ret: LuaTableProps = LuaTableProps {
        maxidx: 0,
        string_keys_num: 0,
        has_string_with_nul: false,
        type_0: kObjectTypeNil,
        val: 0.,
        has_type_key: false,
    };
    memset(
        &raw mut ret as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<LuaTableProps>(),
    );
    if lua_checkstack(lstate, lua_gettop(lstate) + 3 as ::core::ffi::c_int) == 0 {
        semsg(
            gettext(
                b"E1502: Lua failed to grow stack to %i\0".as_ptr() as *const ::core::ffi::c_char
            ),
            lua_gettop(lstate) + 2 as ::core::ffi::c_int,
        );
        ret.type_0 = kObjectTypeNil;
        return ret;
    }
    lua_pushnil(lstate);
    while lua_next(lstate, -2 as ::core::ffi::c_int) != 0 {
        match lua_type(lstate, -2 as ::core::ffi::c_int) {
            LUA_TSTRING => {
                let mut len: size_t = 0;
                let mut s: *const ::core::ffi::c_char =
                    lua_tolstring(lstate, -2 as ::core::ffi::c_int, &raw mut len);
                if !memchr(s as *const ::core::ffi::c_void, NUL, len).is_null() {
                    ret.has_string_with_nul = true_0 != 0;
                }
                ret.string_keys_num = ret.string_keys_num.wrapping_add(1);
            }
            LUA_TNUMBER => {
                let n: lua_Number = lua_tonumber(lstate, -2 as ::core::ffi::c_int);
                if n > SIZE_MAX as lua_Number
                    || n <= 0 as ::core::ffi::c_int as lua_Number
                    || n as size_t as lua_Number != n
                {
                    other_keys_num = other_keys_num.wrapping_add(1);
                } else {
                    let idx: size_t = n as size_t;
                    if idx > ret.maxidx {
                        ret.maxidx = idx;
                    }
                }
            }
            LUA_TBOOLEAN => {
                let b: bool = lua_toboolean(lstate, -2 as ::core::ffi::c_int) != 0;
                if b as ::core::ffi::c_int == TYPE_IDX_VALUE {
                    if lua_type(lstate, -1 as ::core::ffi::c_int) == LUA_TNUMBER {
                        let mut n_0: lua_Number = lua_tonumber(lstate, -1 as ::core::ffi::c_int);
                        if n_0 == kObjectTypeFloat as ::core::ffi::c_int as lua_Number
                            || n_0 == kObjectTypeArray as ::core::ffi::c_int as lua_Number
                            || n_0 == kObjectTypeDict as ::core::ffi::c_int as lua_Number
                        {
                            ret.has_type_key = true_0 != 0;
                            ret.type_0 = n_0 as ObjectType;
                        } else {
                            other_keys_num = other_keys_num.wrapping_add(1);
                        }
                    } else {
                        other_keys_num = other_keys_num.wrapping_add(1);
                    }
                } else {
                    has_val_key = true_0 != 0;
                    val_type = lua_type(lstate, -1 as ::core::ffi::c_int);
                    if val_type == LUA_TNUMBER {
                        ret.val = lua_tonumber(lstate, -1 as ::core::ffi::c_int);
                    }
                }
            }
            _ => {
                other_keys_num = other_keys_num.wrapping_add(1);
            }
        }
        tsize = tsize.wrapping_add(1);
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    }
    if ret.has_type_key {
        '_c2rust_label: {
            if tsize > 0 as size_t {
            } else {
                __assert_fail(
                    b"tsize > 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/lua/converter.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    124 as ::core::ffi::c_uint,
                    b"LuaTableProps nlua_traverse_table(lua_State *const)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if ret.type_0 as ::core::ffi::c_uint
            == kObjectTypeFloat as ::core::ffi::c_int as ::core::ffi::c_uint
            && (!has_val_key || val_type != LUA_TNUMBER)
        {
            ret.type_0 = kObjectTypeNil;
        } else if ret.type_0 as ::core::ffi::c_uint
            == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if ret.maxidx != 0 as size_t
                && ret.maxidx
                    != tsize
                        .wrapping_sub(ret.has_type_key as size_t)
                        .wrapping_sub(other_keys_num)
                        .wrapping_sub(has_val_key as size_t)
                        .wrapping_sub(ret.string_keys_num)
            {
                ret.maxidx = 0 as size_t;
                loop {
                    lua_rawgeti(
                        lstate,
                        -1 as ::core::ffi::c_int,
                        ret.maxidx as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
                    );
                    if lua_type(lstate, -1 as ::core::ffi::c_int) == LUA_TNIL {
                        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                        break;
                    } else {
                        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                        ret.maxidx = ret.maxidx.wrapping_add(1);
                    }
                }
            }
        }
    } else if tsize == 0 as size_t
        || tsize <= ret.maxidx
            && other_keys_num == 0 as size_t
            && ret.string_keys_num == 0 as size_t
    {
        ret.type_0 = kObjectTypeArray;
        if tsize == 0 as size_t && lua_getmetatable(lstate, -1 as ::core::ffi::c_int) != 0 {
            nlua_pushref(lstate, (*nlua_global_refs.get()).empty_dict_ref);
            if lua_rawequal(lstate, -2 as ::core::ffi::c_int, -1 as ::core::ffi::c_int) != 0 {
                ret.type_0 = kObjectTypeDict;
            }
            lua_settop(lstate, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        }
    } else if ret.string_keys_num == tsize {
        ret.type_0 = kObjectTypeDict;
    } else {
        ret.type_0 = kObjectTypeNil;
    }
    return ret;
}
pub unsafe extern "C" fn nlua_pop_typval(
    mut lstate: *mut lua_State,
    mut ret_tv: *mut typval_T,
) -> bool {
    let mut ret: bool = true_0 != 0;
    let initial_size: ::core::ffi::c_int = lua_gettop(lstate);
    let mut stack: C2Rust_Unnamed_7 = C2Rust_Unnamed_7 {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<TVPopStackItem>(),
        init_array: [TVPopStackItem {
            tv: ::core::ptr::null_mut::<typval_T>(),
            list_len: 0,
            container: false,
            special: false,
            idx: 0,
        }; 2],
    };
    stack.capacity = ::core::mem::size_of::<[TVPopStackItem; 2]>()
        .wrapping_div(::core::mem::size_of::<TVPopStackItem>())
        .wrapping_div(
            (::core::mem::size_of::<[TVPopStackItem; 2]>()
                .wrapping_rem(::core::mem::size_of::<TVPopStackItem>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    stack.size = 0 as size_t;
    stack.items = &raw mut stack.init_array as *mut TVPopStackItem;
    if stack.size == stack.capacity {
        stack.capacity = if stack.capacity << 1 as ::core::ffi::c_int
            > ::core::mem::size_of::<[TVPopStackItem; 2]>()
                .wrapping_div(::core::mem::size_of::<TVPopStackItem>())
                .wrapping_div(
                    (::core::mem::size_of::<[TVPopStackItem; 2]>()
                        .wrapping_rem(::core::mem::size_of::<TVPopStackItem>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            stack.capacity << 1 as ::core::ffi::c_int
        } else {
            ::core::mem::size_of::<[TVPopStackItem; 2]>()
                .wrapping_div(::core::mem::size_of::<TVPopStackItem>())
                .wrapping_div(
                    (::core::mem::size_of::<[TVPopStackItem; 2]>()
                        .wrapping_rem(::core::mem::size_of::<TVPopStackItem>())
                        == 0) as ::core::ffi::c_int as size_t,
                )
        };
        stack.items = (if stack.capacity
            == ::core::mem::size_of::<[TVPopStackItem; 2]>()
                .wrapping_div(::core::mem::size_of::<TVPopStackItem>())
                .wrapping_div(
                    (::core::mem::size_of::<[TVPopStackItem; 2]>()
                        .wrapping_rem(::core::mem::size_of::<TVPopStackItem>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            if stack.items == &raw mut stack.init_array as *mut TVPopStackItem {
                stack.items as *mut ::core::ffi::c_void
            } else {
                _memcpy_free(
                    &raw mut stack.init_array as *mut TVPopStackItem as *mut ::core::ffi::c_void,
                    stack.items as *mut ::core::ffi::c_void,
                    stack
                        .size
                        .wrapping_mul(::core::mem::size_of::<TVPopStackItem>()),
                )
            }
        } else {
            if stack.items == &raw mut stack.init_array as *mut TVPopStackItem {
                memcpy(
                    xmalloc(
                        stack
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<TVPopStackItem>()),
                    ),
                    stack.items as *const ::core::ffi::c_void,
                    stack
                        .size
                        .wrapping_mul(::core::mem::size_of::<TVPopStackItem>()),
                )
            } else {
                xrealloc(
                    stack.items as *mut ::core::ffi::c_void,
                    stack
                        .capacity
                        .wrapping_mul(::core::mem::size_of::<TVPopStackItem>()),
                )
            }
        }) as *mut TVPopStackItem;
    } else {
    };
    let c2rust_fresh0 = stack.size;
    stack.size = stack.size.wrapping_add(1);
    *stack.items.offset(c2rust_fresh0 as isize) = TVPopStackItem {
        tv: ret_tv,
        list_len: 0,
        container: false,
        special: false,
        idx: 0,
    };
    while ret as ::core::ffi::c_int != 0 && stack.size != 0 {
        if lua_checkstack(lstate, lua_gettop(lstate) + 3 as ::core::ffi::c_int) == 0 {
            semsg(
                gettext(b"E1502: Lua failed to grow stack to %i\0".as_ptr()
                    as *const ::core::ffi::c_char),
                lua_gettop(lstate) + 3 as ::core::ffi::c_int,
            );
            ret = false_0 != 0;
            break;
        } else {
            stack.size = stack.size.wrapping_sub(1);
            let mut cur: TVPopStackItem = *stack.items.offset(stack.size as isize);
            if cur.container {
                if cur.special as ::core::ffi::c_int != 0
                    || (*cur.tv).v_type as ::core::ffi::c_uint
                        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    '_c2rust_label: {
                        if (*cur.tv).v_type as ::core::ffi::c_uint
                            == (if cur.special as ::core::ffi::c_int != 0 {
                                VAR_LIST as ::core::ffi::c_int
                            } else {
                                VAR_DICT as ::core::ffi::c_int
                            }) as ::core::ffi::c_uint
                        {
                        } else {
                            __assert_fail(
                                b"cur.tv->v_type == (cur.special ? VAR_LIST : VAR_DICT)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/lua/converter.rs\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                207 as ::core::ffi::c_uint,
                                b"_Bool nlua_pop_typval(lua_State *, typval_T *)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    let mut next_key_found: bool = false_0 != 0;
                    while lua_next(lstate, -2 as ::core::ffi::c_int) != 0 {
                        if lua_type(lstate, -2 as ::core::ffi::c_int) == LUA_TSTRING {
                            next_key_found = true_0 != 0;
                            break;
                        } else {
                            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                        }
                    }
                    if next_key_found {
                        let mut len: size_t = 0;
                        let mut s: *const ::core::ffi::c_char =
                            lua_tolstring(lstate, -2 as ::core::ffi::c_int, &raw mut len);
                        if cur.special {
                            let kv_pair: *mut list_T = tv_list_alloc(2 as ptrdiff_t);
                            let mut s_tv: typval_T =
                                decode_string(s, len, true_0 != 0, false_0 != 0);
                            tv_list_append_owned_tv(kv_pair, s_tv);
                            tv_list_append_owned_tv(
                                kv_pair,
                                typval_T {
                                    v_type: VAR_UNKNOWN,
                                    v_lock: VAR_UNLOCKED,
                                    vval: typval_vval_union { v_number: 0 },
                                },
                            );
                            if stack.size == stack.capacity {
                                stack.capacity = if stack.capacity << 1 as ::core::ffi::c_int
                                    > ::core::mem::size_of::<[TVPopStackItem; 2]>()
                                        .wrapping_div(::core::mem::size_of::<TVPopStackItem>())
                                        .wrapping_div(
                                            (::core::mem::size_of::<[TVPopStackItem; 2]>()
                                                .wrapping_rem(
                                                    ::core::mem::size_of::<TVPopStackItem>(),
                                                )
                                                == 0)
                                                as ::core::ffi::c_int
                                                as usize,
                                        ) {
                                    stack.capacity << 1 as ::core::ffi::c_int
                                } else {
                                    ::core::mem::size_of::<[TVPopStackItem; 2]>()
                                        .wrapping_div(::core::mem::size_of::<TVPopStackItem>())
                                        .wrapping_div(
                                            (::core::mem::size_of::<[TVPopStackItem; 2]>()
                                                .wrapping_rem(
                                                    ::core::mem::size_of::<TVPopStackItem>(),
                                                )
                                                == 0)
                                                as ::core::ffi::c_int
                                                as size_t,
                                        )
                                };
                                stack.items =
                                    (if stack.capacity
                                        == ::core::mem::size_of::<[TVPopStackItem; 2]>()
                                            .wrapping_div(::core::mem::size_of::<TVPopStackItem>())
                                            .wrapping_div(
                                                (::core::mem::size_of::<[TVPopStackItem; 2]>()
                                                    .wrapping_rem(::core::mem::size_of::<
                                                        TVPopStackItem,
                                                    >(
                                                    ))
                                                    == 0)
                                                    as ::core::ffi::c_int
                                                    as usize,
                                            )
                                    {
                                        if stack.items
                                            == &raw mut stack.init_array as *mut TVPopStackItem
                                        {
                                            stack.items as *mut ::core::ffi::c_void
                                        } else {
                                            _memcpy_free(
                                                &raw mut stack.init_array as *mut TVPopStackItem
                                                    as *mut ::core::ffi::c_void,
                                                stack.items as *mut ::core::ffi::c_void,
                                                stack.size.wrapping_mul(::core::mem::size_of::<
                                                    TVPopStackItem,
                                                >(
                                                )),
                                            )
                                        }
                                    } else {
                                        if stack.items
                                            == &raw mut stack.init_array as *mut TVPopStackItem
                                        {
                                            memcpy(
                                                xmalloc(stack.capacity.wrapping_mul(
                                                    ::core::mem::size_of::<TVPopStackItem>(),
                                                )),
                                                stack.items as *const ::core::ffi::c_void,
                                                stack.size.wrapping_mul(::core::mem::size_of::<
                                                    TVPopStackItem,
                                                >(
                                                )),
                                            )
                                        } else {
                                            xrealloc(
                                                stack.items as *mut ::core::ffi::c_void,
                                                stack.capacity.wrapping_mul(
                                                    ::core::mem::size_of::<TVPopStackItem>(),
                                                ),
                                            )
                                        }
                                    }) as *mut TVPopStackItem;
                            } else {
                            };
                            let c2rust_fresh1 = stack.size;
                            stack.size = stack.size.wrapping_add(1);
                            *stack.items.offset(c2rust_fresh1 as isize) = cur;
                            tv_list_append_list((*cur.tv).vval.v_list, kv_pair);
                            cur = TVPopStackItem {
                                tv: &raw mut (*(tv_list_last
                                    as unsafe extern "C" fn(*const list_T) -> *mut listitem_T)(
                                    kv_pair,
                                ))
                                .li_tv,
                                list_len: 0,
                                container: false,
                                special: false,
                                idx: 0,
                            };
                        } else {
                            let di: *mut dictitem_T = tv_dict_item_alloc_len(s, len);
                            if tv_dict_add((*cur.tv).vval.v_dict, di) == FAIL {
                                abort();
                            }
                            if stack.size == stack.capacity {
                                stack.capacity = if stack.capacity << 1 as ::core::ffi::c_int
                                    > ::core::mem::size_of::<[TVPopStackItem; 2]>()
                                        .wrapping_div(::core::mem::size_of::<TVPopStackItem>())
                                        .wrapping_div(
                                            (::core::mem::size_of::<[TVPopStackItem; 2]>()
                                                .wrapping_rem(
                                                    ::core::mem::size_of::<TVPopStackItem>(),
                                                )
                                                == 0)
                                                as ::core::ffi::c_int
                                                as usize,
                                        ) {
                                    stack.capacity << 1 as ::core::ffi::c_int
                                } else {
                                    ::core::mem::size_of::<[TVPopStackItem; 2]>()
                                        .wrapping_div(::core::mem::size_of::<TVPopStackItem>())
                                        .wrapping_div(
                                            (::core::mem::size_of::<[TVPopStackItem; 2]>()
                                                .wrapping_rem(
                                                    ::core::mem::size_of::<TVPopStackItem>(),
                                                )
                                                == 0)
                                                as ::core::ffi::c_int
                                                as size_t,
                                        )
                                };
                                stack.items =
                                    (if stack.capacity
                                        == ::core::mem::size_of::<[TVPopStackItem; 2]>()
                                            .wrapping_div(::core::mem::size_of::<TVPopStackItem>())
                                            .wrapping_div(
                                                (::core::mem::size_of::<[TVPopStackItem; 2]>()
                                                    .wrapping_rem(::core::mem::size_of::<
                                                        TVPopStackItem,
                                                    >(
                                                    ))
                                                    == 0)
                                                    as ::core::ffi::c_int
                                                    as usize,
                                            )
                                    {
                                        if stack.items
                                            == &raw mut stack.init_array as *mut TVPopStackItem
                                        {
                                            stack.items as *mut ::core::ffi::c_void
                                        } else {
                                            _memcpy_free(
                                                &raw mut stack.init_array as *mut TVPopStackItem
                                                    as *mut ::core::ffi::c_void,
                                                stack.items as *mut ::core::ffi::c_void,
                                                stack.size.wrapping_mul(::core::mem::size_of::<
                                                    TVPopStackItem,
                                                >(
                                                )),
                                            )
                                        }
                                    } else {
                                        if stack.items
                                            == &raw mut stack.init_array as *mut TVPopStackItem
                                        {
                                            memcpy(
                                                xmalloc(stack.capacity.wrapping_mul(
                                                    ::core::mem::size_of::<TVPopStackItem>(),
                                                )),
                                                stack.items as *const ::core::ffi::c_void,
                                                stack.size.wrapping_mul(::core::mem::size_of::<
                                                    TVPopStackItem,
                                                >(
                                                )),
                                            )
                                        } else {
                                            xrealloc(
                                                stack.items as *mut ::core::ffi::c_void,
                                                stack.capacity.wrapping_mul(
                                                    ::core::mem::size_of::<TVPopStackItem>(),
                                                ),
                                            )
                                        }
                                    }) as *mut TVPopStackItem;
                            } else {
                            };
                            let c2rust_fresh2 = stack.size;
                            stack.size = stack.size.wrapping_add(1);
                            *stack.items.offset(c2rust_fresh2 as isize) = cur;
                            cur = TVPopStackItem {
                                tv: &raw mut (*di).di_tv,
                                list_len: 0,
                                container: false,
                                special: false,
                                idx: 0,
                            };
                        }
                    } else {
                        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                        continue;
                    }
                } else {
                    '_c2rust_label_0: {
                        if (*cur.tv).v_type as ::core::ffi::c_uint
                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                        } else {
                            __assert_fail(
                                b"cur.tv->v_type == VAR_LIST\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/lua/converter.rs\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                245 as ::core::ffi::c_uint,
                                b"_Bool nlua_pop_typval(lua_State *, typval_T *)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    if tv_list_len((*cur.tv).vval.v_list) as size_t == cur.list_len {
                        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                        continue;
                    } else {
                        lua_rawgeti(
                            lstate,
                            -1 as ::core::ffi::c_int,
                            tv_list_len((*cur.tv).vval.v_list) + 1 as ::core::ffi::c_int,
                        );
                        tv_list_append_owned_tv(
                            (*cur.tv).vval.v_list,
                            typval_T {
                                v_type: VAR_UNKNOWN,
                                v_lock: VAR_UNLOCKED,
                                vval: typval_vval_union { v_number: 0 },
                            },
                        );
                        if stack.size == stack.capacity {
                            stack.capacity = if stack.capacity << 1 as ::core::ffi::c_int
                                > ::core::mem::size_of::<[TVPopStackItem; 2]>()
                                    .wrapping_div(::core::mem::size_of::<TVPopStackItem>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[TVPopStackItem; 2]>()
                                            .wrapping_rem(::core::mem::size_of::<TVPopStackItem>())
                                            == 0)
                                            as ::core::ffi::c_int
                                            as usize,
                                    ) {
                                stack.capacity << 1 as ::core::ffi::c_int
                            } else {
                                ::core::mem::size_of::<[TVPopStackItem; 2]>()
                                    .wrapping_div(::core::mem::size_of::<TVPopStackItem>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[TVPopStackItem; 2]>()
                                            .wrapping_rem(::core::mem::size_of::<TVPopStackItem>())
                                            == 0)
                                            as ::core::ffi::c_int
                                            as size_t,
                                    )
                            };
                            stack.items = (if stack.capacity
                                == ::core::mem::size_of::<[TVPopStackItem; 2]>()
                                    .wrapping_div(::core::mem::size_of::<TVPopStackItem>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[TVPopStackItem; 2]>()
                                            .wrapping_rem(::core::mem::size_of::<TVPopStackItem>())
                                            == 0)
                                            as ::core::ffi::c_int
                                            as usize,
                                    ) {
                                if stack.items == &raw mut stack.init_array as *mut TVPopStackItem {
                                    stack.items as *mut ::core::ffi::c_void
                                } else {
                                    _memcpy_free(
                                        &raw mut stack.init_array as *mut TVPopStackItem
                                            as *mut ::core::ffi::c_void,
                                        stack.items as *mut ::core::ffi::c_void,
                                        stack
                                            .size
                                            .wrapping_mul(::core::mem::size_of::<TVPopStackItem>()),
                                    )
                                }
                            } else {
                                if stack.items == &raw mut stack.init_array as *mut TVPopStackItem {
                                    memcpy(
                                        xmalloc(
                                            stack
                                                .capacity
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<TVPopStackItem>(),
                                                ),
                                        ),
                                        stack.items as *const ::core::ffi::c_void,
                                        stack
                                            .size
                                            .wrapping_mul(::core::mem::size_of::<TVPopStackItem>()),
                                    )
                                } else {
                                    xrealloc(
                                        stack.items as *mut ::core::ffi::c_void,
                                        stack
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<TVPopStackItem>()),
                                    )
                                }
                            }) as *mut TVPopStackItem;
                        } else {
                        };
                        let c2rust_fresh3 = stack.size;
                        stack.size = stack.size.wrapping_add(1);
                        *stack.items.offset(c2rust_fresh3 as isize) = cur;
                        cur = TVPopStackItem {
                            tv: &raw mut (*(tv_list_last
                                as unsafe extern "C" fn(*const list_T) -> *mut listitem_T)(
                                (*cur.tv).vval.v_list,
                            ))
                            .li_tv,
                            list_len: 0,
                            container: false,
                            special: false,
                            idx: 0,
                        };
                    }
                }
            }
            '_c2rust_label_1: {
                if !cur.container {
                } else {
                    __assert_fail(
                        b"!cur.container\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/lua/converter.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        260 as ::core::ffi::c_uint,
                        b"_Bool nlua_pop_typval(lua_State *, typval_T *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            *cur.tv = typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union {
                    v_number: 0 as varnumber_T,
                },
            };
            's_523: {
                match lua_type(lstate, -1 as ::core::ffi::c_int) {
                    LUA_TNIL => {
                        (*cur.tv).v_type = VAR_SPECIAL;
                        (*cur.tv).vval.v_special = kSpecialVarNull;
                    }
                    LUA_TBOOLEAN => {
                        (*cur.tv).v_type = VAR_BOOL;
                        (*cur.tv).vval.v_bool =
                            (if lua_toboolean(lstate, -1 as ::core::ffi::c_int) != 0 {
                                kBoolVarTrue as ::core::ffi::c_int
                            } else {
                                kBoolVarFalse as ::core::ffi::c_int
                            }) as BoolVarValue;
                    }
                    LUA_TSTRING => {
                        let mut len_0: size_t = 0;
                        let mut s_0: *const ::core::ffi::c_char =
                            lua_tolstring(lstate, -1 as ::core::ffi::c_int, &raw mut len_0);
                        *cur.tv = decode_string(s_0, len_0, false_0 != 0, false_0 != 0);
                    }
                    LUA_TNUMBER => {
                        let n: lua_Number = lua_tonumber(lstate, -1 as ::core::ffi::c_int);
                        if n > VARNUMBER_MAX as lua_Number
                            || n < VARNUMBER_MIN as lua_Number
                            || n as varnumber_T as lua_Number != n
                        {
                            (*cur.tv).v_type = VAR_FLOAT;
                            (*cur.tv).vval.v_float = n;
                        } else {
                            (*cur.tv).v_type = VAR_NUMBER;
                            (*cur.tv).vval.v_number = n as varnumber_T;
                        }
                    }
                    LUA_TTABLE => {
                        let mut table_ref: LuaRef = LUA_NOREF;
                        if lua_getmetatable(lstate, -1 as ::core::ffi::c_int) != 0 {
                            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                            table_ref = nlua_ref_global(lstate, -1 as ::core::ffi::c_int);
                        }
                        let table_props: LuaTableProps = nlua_traverse_table(lstate);
                        let mut i: size_t = 0 as size_t;
                        while i < stack.size {
                            let item: TVPopStackItem = *stack.items.offset(i as isize);
                            if item.container as ::core::ffi::c_int != 0
                                && lua_rawequal(lstate, -1 as ::core::ffi::c_int, item.idx) != 0
                            {
                                tv_copy(item.tv, cur.tv);
                                cur.container = false_0 != 0;
                                break 's_523;
                            } else {
                                i = i.wrapping_add(1);
                            }
                        }
                        match table_props.type_0 as ::core::ffi::c_uint {
                            5 => {
                                (*cur.tv).v_type = VAR_LIST;
                                (*cur.tv).vval.v_list =
                                    tv_list_alloc(table_props.maxidx as ptrdiff_t);
                                (*(*cur.tv).vval.v_list).lua_table_ref = table_ref;
                                tv_list_ref((*cur.tv).vval.v_list);
                                cur.list_len = table_props.maxidx;
                                if table_props.maxidx != 0 as size_t {
                                    cur.container = true_0 != 0;
                                    cur.idx = lua_gettop(lstate);
                                    if stack.size == stack.capacity {
                                        stack.capacity = if stack.capacity
                                            << 1 as ::core::ffi::c_int
                                            > ::core::mem::size_of::<[TVPopStackItem; 2]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<TVPopStackItem>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[TVPopStackItem; 2]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            TVPopStackItem,
                                                        >(
                                                        ))
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as usize,
                                                ) {
                                            stack.capacity << 1 as ::core::ffi::c_int
                                        } else {
                                            ::core::mem::size_of::<[TVPopStackItem; 2]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<TVPopStackItem>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[TVPopStackItem; 2]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            TVPopStackItem,
                                                        >(
                                                        ))
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as size_t,
                                                )
                                        };
                                        stack.items = (if stack.capacity
                                            == ::core::mem::size_of::<[TVPopStackItem; 2]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<TVPopStackItem>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[TVPopStackItem; 2]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            TVPopStackItem,
                                                        >(
                                                        ))
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as usize,
                                                ) {
                                            if stack.items
                                                == &raw mut stack.init_array as *mut TVPopStackItem
                                            {
                                                stack.items as *mut ::core::ffi::c_void
                                            } else {
                                                _memcpy_free(
                                                    &raw mut stack.init_array as *mut TVPopStackItem
                                                        as *mut ::core::ffi::c_void,
                                                    stack.items as *mut ::core::ffi::c_void,
                                                    stack.size.wrapping_mul(
                                                        ::core::mem::size_of::<TVPopStackItem>(),
                                                    ),
                                                )
                                            }
                                        } else {
                                            if stack.items
                                                == &raw mut stack.init_array as *mut TVPopStackItem
                                            {
                                                memcpy(
                                                    xmalloc(stack.capacity.wrapping_mul(
                                                        ::core::mem::size_of::<TVPopStackItem>(),
                                                    )),
                                                    stack.items as *const ::core::ffi::c_void,
                                                    stack.size.wrapping_mul(
                                                        ::core::mem::size_of::<TVPopStackItem>(),
                                                    ),
                                                )
                                            } else {
                                                xrealloc(
                                                    stack.items as *mut ::core::ffi::c_void,
                                                    stack.capacity.wrapping_mul(
                                                        ::core::mem::size_of::<TVPopStackItem>(),
                                                    ),
                                                )
                                            }
                                        })
                                            as *mut TVPopStackItem;
                                    } else {
                                    };
                                    let c2rust_fresh4 = stack.size;
                                    stack.size = stack.size.wrapping_add(1);
                                    *stack.items.offset(c2rust_fresh4 as isize) = cur;
                                }
                            }
                            6 => {
                                if table_props.string_keys_num == 0 as size_t {
                                    (*cur.tv).v_type = VAR_DICT;
                                    (*cur.tv).vval.v_dict = tv_dict_alloc();
                                    (*(*cur.tv).vval.v_dict).dv_refcount += 1;
                                    (*(*cur.tv).vval.v_dict).lua_table_ref = table_ref;
                                } else {
                                    cur.special = table_props.has_string_with_nul;
                                    if table_props.has_string_with_nul {
                                        decode_create_map_special_dict(
                                            cur.tv,
                                            table_props.string_keys_num as ptrdiff_t,
                                        );
                                        '_c2rust_label_2: {
                                            if (*cur.tv).v_type as ::core::ffi::c_uint
                                                == VAR_DICT as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                            } else {
                                                __assert_fail(
                                                    b"cur.tv->v_type == VAR_DICT\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                    b"src/nvim/lua/converter.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    335 as ::core::ffi::c_uint,
                                                    b"_Bool nlua_pop_typval(lua_State *, typval_T *)\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        let val_di: *mut dictitem_T = tv_dict_find(
                                            (*cur.tv).vval.v_dict,
                                            b"_VAL\0".as_ptr() as *const ::core::ffi::c_char,
                                            ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                                                .wrapping_sub(1 as usize)
                                                as ptrdiff_t,
                                        );
                                        '_c2rust_label_3: {
                                            if !val_di.is_null() {
                                            } else {
                                                __assert_fail(
                                                    b"val_di != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                                                    b"src/nvim/lua/converter.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    338 as ::core::ffi::c_uint,
                                                    b"_Bool nlua_pop_typval(lua_State *, typval_T *)\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        cur.tv = &raw mut (*val_di).di_tv;
                                        (*(*cur.tv).vval.v_list).lua_table_ref = table_ref;
                                        '_c2rust_label_4: {
                                            if (*cur.tv).v_type as ::core::ffi::c_uint
                                                == VAR_LIST as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                            } else {
                                                __assert_fail(
                                                    b"cur.tv->v_type == VAR_LIST\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                    b"src/nvim/lua/converter.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    341 as ::core::ffi::c_uint,
                                                    b"_Bool nlua_pop_typval(lua_State *, typval_T *)\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        cur.list_len = table_props.string_keys_num;
                                    } else {
                                        (*cur.tv).v_type = VAR_DICT;
                                        (*cur.tv).vval.v_dict = tv_dict_alloc();
                                        (*(*cur.tv).vval.v_dict).dv_refcount += 1;
                                        (*(*cur.tv).vval.v_dict).lua_table_ref = table_ref;
                                    }
                                    cur.container = true_0 != 0;
                                    cur.idx = lua_gettop(lstate);
                                    if stack.size == stack.capacity {
                                        stack.capacity = if stack.capacity
                                            << 1 as ::core::ffi::c_int
                                            > ::core::mem::size_of::<[TVPopStackItem; 2]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<TVPopStackItem>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[TVPopStackItem; 2]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            TVPopStackItem,
                                                        >(
                                                        ))
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as usize,
                                                ) {
                                            stack.capacity << 1 as ::core::ffi::c_int
                                        } else {
                                            ::core::mem::size_of::<[TVPopStackItem; 2]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<TVPopStackItem>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[TVPopStackItem; 2]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            TVPopStackItem,
                                                        >(
                                                        ))
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as size_t,
                                                )
                                        };
                                        stack.items = (if stack.capacity
                                            == ::core::mem::size_of::<[TVPopStackItem; 2]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<TVPopStackItem>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[TVPopStackItem; 2]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            TVPopStackItem,
                                                        >(
                                                        ))
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as usize,
                                                ) {
                                            if stack.items
                                                == &raw mut stack.init_array as *mut TVPopStackItem
                                            {
                                                stack.items as *mut ::core::ffi::c_void
                                            } else {
                                                _memcpy_free(
                                                    &raw mut stack.init_array as *mut TVPopStackItem
                                                        as *mut ::core::ffi::c_void,
                                                    stack.items as *mut ::core::ffi::c_void,
                                                    stack.size.wrapping_mul(
                                                        ::core::mem::size_of::<TVPopStackItem>(),
                                                    ),
                                                )
                                            }
                                        } else {
                                            if stack.items
                                                == &raw mut stack.init_array as *mut TVPopStackItem
                                            {
                                                memcpy(
                                                    xmalloc(stack.capacity.wrapping_mul(
                                                        ::core::mem::size_of::<TVPopStackItem>(),
                                                    )),
                                                    stack.items as *const ::core::ffi::c_void,
                                                    stack.size.wrapping_mul(
                                                        ::core::mem::size_of::<TVPopStackItem>(),
                                                    ),
                                                )
                                            } else {
                                                xrealloc(
                                                    stack.items as *mut ::core::ffi::c_void,
                                                    stack.capacity.wrapping_mul(
                                                        ::core::mem::size_of::<TVPopStackItem>(),
                                                    ),
                                                )
                                            }
                                        })
                                            as *mut TVPopStackItem;
                                    } else {
                                    };
                                    let c2rust_fresh5 = stack.size;
                                    stack.size = stack.size.wrapping_add(1);
                                    *stack.items.offset(c2rust_fresh5 as isize) = cur;
                                    lua_pushnil(lstate);
                                }
                            }
                            3 => {
                                (*cur.tv).v_type = VAR_FLOAT;
                                (*cur.tv).vval.v_float = table_props.val;
                            }
                            0 => {
                                emsg(
                                    gettext(
                                        b"E5100: Cannot convert given Lua table: table should contain either only integer keys or only string keys\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    ),
                                );
                                ret = false_0 != 0;
                            }
                            _ => {
                                abort();
                            }
                        }
                    }
                    LUA_TFUNCTION => {
                        let mut func: LuaRef = nlua_ref_global(lstate, -1 as ::core::ffi::c_int);
                        let mut name: *mut ::core::ffi::c_char = register_luafunc(func);
                        (*cur.tv).v_type = VAR_FUNC;
                        (*cur.tv).vval.v_string = xstrdup(name);
                    }
                    LUA_TUSERDATA => {
                        nlua_pushref(lstate, (*nlua_global_refs.get()).nil_ref);
                        let mut is_nil: bool = lua_rawequal(
                            lstate,
                            -2 as ::core::ffi::c_int,
                            -1 as ::core::ffi::c_int,
                        ) != 0;
                        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                        if is_nil {
                            (*cur.tv).v_type = VAR_SPECIAL;
                            (*cur.tv).vval.v_special = kSpecialVarNull;
                        } else {
                            emsg(gettext(b"E5101: Cannot convert given Lua type\0".as_ptr()
                                as *const ::core::ffi::c_char));
                            ret = false_0 != 0;
                        }
                    }
                    _ => {
                        emsg(gettext(b"E5101: Cannot convert given Lua type\0".as_ptr()
                            as *const ::core::ffi::c_char));
                        ret = false_0 != 0;
                    }
                }
            }
            if !cur.container {
                lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            }
        }
    }
    if stack.items != &raw mut stack.init_array as *mut TVPopStackItem {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut stack.items as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
    }
    if !ret {
        tv_clear(ret_tv);
        *ret_tv = typval_T {
            v_type: VAR_NUMBER,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_number: 0 as varnumber_T,
            },
        };
        lua_settop(
            lstate,
            -(lua_gettop(lstate) - initial_size + 1 as ::core::ffi::c_int)
                - 1 as ::core::ffi::c_int,
        );
    }
    '_c2rust_label_5: {
        if lua_gettop(lstate) == initial_size - 1 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"lua_gettop(lstate) == initial_size - 1\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/lua/converter.rs\0".as_ptr() as *const ::core::ffi::c_char,
                412 as ::core::ffi::c_uint,
                b"_Bool nlua_pop_typval(lua_State *, typval_T *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    return ret;
}
static typval_conv_special: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub const TYPVAL_ENCODE_ALLOW_SPECIALS: ::core::ffi::c_int = true_0;
pub unsafe extern "C" fn nlua_push_typval(
    mut lstate: *mut lua_State,
    tv: *mut typval_T,
    mut flags: ::core::ffi::c_int,
) -> bool {
    typval_conv_special.set(flags & kNluaPushSpecial as ::core::ffi::c_int != 0);
    let initial_size: ::core::ffi::c_int = lua_gettop(lstate);
    if lua_checkstack(lstate, initial_size + 2 as ::core::ffi::c_int) == 0 {
        semsg(
            gettext(
                b"E1502: Lua failed to grow stack to %i\0".as_ptr() as *const ::core::ffi::c_char
            ),
            initial_size + 4 as ::core::ffi::c_int,
        );
        return false_0 != 0;
    }
    if encode_vim_to_lua(
        lstate,
        tv,
        b"nlua_push_typval argument\0".as_ptr() as *const ::core::ffi::c_char,
    ) == FAIL
    {
        return false_0 != 0;
    }
    '_c2rust_label: {
        if lua_gettop(lstate) == initial_size + 1 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"lua_gettop(lstate) == initial_size + 1\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/lua/converter.rs\0".as_ptr() as *const ::core::ffi::c_char,
                607 as ::core::ffi::c_uint,
                b"_Bool nlua_push_typval(lua_State *, typval_T *const, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    return true_0 != 0;
}
#[inline]
unsafe extern "C" fn nlua_push_type_idx(mut lstate: *mut lua_State) {
    lua_pushboolean(lstate, TYPE_IDX_VALUE);
}
#[inline]
unsafe extern "C" fn nlua_push_val_idx(mut lstate: *mut lua_State) {
    lua_pushboolean(lstate, VAL_IDX_VALUE);
}
#[inline]
unsafe extern "C" fn nlua_push_type(mut lstate: *mut lua_State, mut type_0: ObjectType) {
    lua_pushnumber(lstate, type_0 as lua_Number);
}
#[inline]
unsafe extern "C" fn nlua_create_typed_table(
    mut lstate: *mut lua_State,
    narr: size_t,
    nrec: size_t,
    type_0: ObjectType,
) {
    lua_createtable(
        lstate,
        narr as ::core::ffi::c_int,
        (1 as size_t).wrapping_add(nrec) as ::core::ffi::c_int,
    );
    nlua_push_type_idx(lstate);
    nlua_push_type(lstate, type_0);
    lua_rawset(lstate, -3 as ::core::ffi::c_int);
}
pub unsafe extern "C" fn nlua_push_String(
    mut lstate: *mut lua_State,
    s: String_0,
    mut _flags: ::core::ffi::c_int,
) {
    lua_pushlstring(
        lstate,
        if s.size != 0 {
            s.data as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
        s.size,
    );
}
pub unsafe extern "C" fn nlua_push_Integer(
    mut lstate: *mut lua_State,
    n: Integer,
    mut _flags: ::core::ffi::c_int,
) {
    lua_pushnumber(lstate, n as lua_Number);
}
pub unsafe extern "C" fn nlua_push_Float(
    mut lstate: *mut lua_State,
    f: Float,
    mut flags: ::core::ffi::c_int,
) {
    if flags & kNluaPushSpecial as ::core::ffi::c_int != 0 {
        nlua_create_typed_table(lstate, 0 as size_t, 1 as size_t, kObjectTypeFloat);
        nlua_push_val_idx(lstate);
        lua_pushnumber(lstate, f);
        lua_rawset(lstate, -3 as ::core::ffi::c_int);
    } else {
        lua_pushnumber(lstate, f);
    };
}
pub unsafe extern "C" fn nlua_push_Boolean(
    mut lstate: *mut lua_State,
    b: Boolean,
    mut _flags: ::core::ffi::c_int,
) {
    lua_pushboolean(lstate, b as ::core::ffi::c_int);
}
pub unsafe extern "C" fn nlua_push_Dict(
    mut lstate: *mut lua_State,
    dict: Dict,
    mut flags: ::core::ffi::c_int,
) {
    lua_createtable(
        lstate,
        0 as ::core::ffi::c_int,
        dict.size as ::core::ffi::c_int,
    );
    if dict.size == 0 as size_t {
        nlua_pushref(lstate, (*nlua_global_refs.get()).empty_dict_ref);
        lua_setmetatable(lstate, -2 as ::core::ffi::c_int);
    }
    let mut i: size_t = 0 as size_t;
    while i < dict.size {
        nlua_push_String(lstate, (*dict.items.offset(i as isize)).key, flags);
        nlua_push_Object(
            lstate,
            &raw mut (*dict.items.offset(i as isize)).value,
            flags,
        );
        lua_rawset(lstate, -3 as ::core::ffi::c_int);
        i = i.wrapping_add(1);
    }
}
pub unsafe extern "C" fn nlua_push_Array(
    mut lstate: *mut lua_State,
    array: Array,
    mut flags: ::core::ffi::c_int,
) {
    lua_createtable(
        lstate,
        array.size as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    let mut i: size_t = 0 as size_t;
    while i < array.size {
        nlua_push_Object(lstate, array.items.offset(i as isize), flags);
        lua_rawseti(
            lstate,
            -2 as ::core::ffi::c_int,
            i as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
        );
        i = i.wrapping_add(1);
    }
}
pub unsafe extern "C" fn nlua_push_handle(
    mut lstate: *mut lua_State,
    item: handle_T,
    mut _flags: ::core::ffi::c_int,
) {
    lua_pushnumber(lstate, item as lua_Number);
}
pub unsafe extern "C" fn nlua_push_Object(
    mut lstate: *mut lua_State,
    mut obj: *mut Object,
    mut flags: ::core::ffi::c_int,
) {
    match (*obj).type_0 as ::core::ffi::c_uint {
        0 => {
            if flags & kNluaPushSpecial as ::core::ffi::c_int != 0 {
                lua_pushnil(lstate);
            } else {
                nlua_pushref(lstate, (*nlua_global_refs.get()).nil_ref);
            }
        }
        7 => {
            nlua_pushref(lstate, (*obj).data.luaref);
            if flags & kNluaPushFreeRefs as ::core::ffi::c_int != 0 {
                api_free_luaref((*obj).data.luaref);
                (*obj).data.luaref = LUA_NOREF as LuaRef;
            }
        }
        1 => {
            nlua_push_Boolean(lstate, (*obj).data.boolean, flags);
        }
        2 => {
            nlua_push_Integer(lstate, (*obj).data.integer, flags);
        }
        3 => {
            nlua_push_Float(lstate, (*obj).data.floating, flags);
        }
        4 => {
            nlua_push_String(lstate, (*obj).data.string, flags);
        }
        5 => {
            nlua_push_Array(lstate, (*obj).data.array, flags);
        }
        6 => {
            nlua_push_Dict(lstate, (*obj).data.dict, flags);
        }
        8 => {
            nlua_push_handle(lstate, (*obj).data.integer as handle_T, flags);
        }
        9 => {
            nlua_push_handle(lstate, (*obj).data.integer as handle_T, flags);
        }
        10 => {
            nlua_push_handle(lstate, (*obj).data.integer as handle_T, flags);
        }
        _ => {}
    };
}
pub unsafe extern "C" fn nlua_pop_String(
    mut lstate: *mut lua_State,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> String_0 {
    if lua_type(lstate, -1 as ::core::ffi::c_int) != LUA_TSTRING {
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Expected Lua string\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0 as size_t,
        };
    }
    let mut ret: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    ret.data = lua_tolstring(lstate, -1 as ::core::ffi::c_int, &raw mut ret.size)
        as *mut ::core::ffi::c_char;
    '_c2rust_label: {
        if !ret.data.is_null() {
        } else {
            __assert_fail(
                b"ret.data != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/lua/converter.rs\0".as_ptr() as *const ::core::ffi::c_char,
                797 as ::core::ffi::c_uint,
                b"String nlua_pop_String(lua_State *, Arena *, Error *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    ret.data = arena_memdupz(arena, ret.data, ret.size);
    lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    return ret;
}
pub unsafe extern "C" fn nlua_pop_Integer(
    mut lstate: *mut lua_State,
    mut _arena: *mut Arena,
    mut err: *mut Error,
) -> Integer {
    if lua_type(lstate, -1 as ::core::ffi::c_int) != LUA_TNUMBER {
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Expected Lua number\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return 0 as Integer;
    }
    let n: lua_Number = lua_tonumber(lstate, -1 as ::core::ffi::c_int);
    lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    if n > API_INTEGER_MAX as lua_Number
        || n < API_INTEGER_MIN as lua_Number
        || n as Integer as lua_Number != n
    {
        api_set_error(
            err,
            kErrorTypeException,
            b"Number is not integral\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return 0 as Integer;
    }
    return n as Integer;
}
pub unsafe extern "C" fn nlua_pop_Boolean(
    mut lstate: *mut lua_State,
    mut _arena: *mut Arena,
    mut _err: *mut Error,
) -> Boolean {
    let ret: Boolean = lua_toboolean(lstate, -1 as ::core::ffi::c_int) != 0;
    lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    return ret;
}
pub unsafe extern "C" fn nlua_pop_Boolean_strict(
    mut lstate: *mut lua_State,
    mut err: *mut Error,
) -> Boolean {
    let mut ret: Boolean = false_0 != 0;
    match lua_type(lstate, -1 as ::core::ffi::c_int) {
        LUA_TBOOLEAN => {
            ret = lua_toboolean(lstate, -1 as ::core::ffi::c_int) != 0;
        }
        LUA_TNUMBER => {
            ret = lua_tonumber(lstate, -1 as ::core::ffi::c_int)
                != 0 as ::core::ffi::c_int as lua_Number;
        }
        LUA_TNIL => {
            ret = false_0 != 0;
        }
        _ => {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"not a boolean\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    }
    lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    return ret;
}
#[inline]
unsafe extern "C" fn nlua_check_type(
    lstate: *mut lua_State,
    err: *mut Error,
    type_0: ObjectType,
) -> LuaTableProps {
    if lua_type(lstate, -1 as ::core::ffi::c_int) != LUA_TTABLE {
        if !err.is_null() {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"Expected Lua %s\0".as_ptr() as *const ::core::ffi::c_char,
                if type_0 as ::core::ffi::c_uint
                    == kObjectTypeFloat as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    b"number\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"table\0".as_ptr() as *const ::core::ffi::c_char
                },
            );
        }
        return LuaTableProps {
            maxidx: 0,
            string_keys_num: 0,
            has_string_with_nul: false,
            type_0: kObjectTypeNil,
            val: 0.,
            has_type_key: false,
        };
    }
    let mut table_props: LuaTableProps = nlua_traverse_table(lstate);
    if type_0 as ::core::ffi::c_uint == kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
        && table_props.type_0 as ::core::ffi::c_uint
            == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
        && table_props.maxidx == 0 as size_t
        && !table_props.has_type_key
    {
        table_props.type_0 = kObjectTypeDict;
    }
    if table_props.type_0 as ::core::ffi::c_uint != type_0 as ::core::ffi::c_uint {
        if !err.is_null() {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"Expected %s-like Lua table\0".as_ptr() as *const ::core::ffi::c_char,
                api_typename(type_0),
            );
        }
    }
    return table_props;
}
pub unsafe extern "C" fn nlua_pop_Float(
    mut lstate: *mut lua_State,
    mut _arena: *mut Arena,
    mut err: *mut Error,
) -> Float {
    if lua_type(lstate, -1 as ::core::ffi::c_int) == LUA_TNUMBER {
        let ret: Float = lua_tonumber(lstate, -1 as ::core::ffi::c_int);
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        return ret;
    }
    let table_props: LuaTableProps = nlua_check_type(lstate, err, kObjectTypeFloat);
    lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    if table_props.type_0 as ::core::ffi::c_uint
        != kObjectTypeFloat as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 0 as ::core::ffi::c_int as Float;
    }
    return table_props.val;
}
unsafe extern "C" fn nlua_pop_Array_unchecked(
    lstate: *mut lua_State,
    table_props: LuaTableProps,
    mut arena: *mut Arena,
    err: *mut Error,
) -> Array {
    let mut ret: Array = arena_array(arena, table_props.maxidx);
    if table_props.maxidx == 0 as size_t {
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        return ret;
    }
    let mut i: size_t = 1 as size_t;
    while i <= table_props.maxidx {
        let mut val: Object = Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
        lua_rawgeti(lstate, -1 as ::core::ffi::c_int, i as ::core::ffi::c_int);
        val = nlua_pop_Object(lstate, false_0 != 0, arena, err);
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            if arena.is_null() {
                api_free_array(ret);
            }
            return Array {
                size: 0 as size_t,
                capacity: 0,
                items: ::core::ptr::null_mut::<Object>(),
            };
        }
        let c2rust_fresh14 = ret.size;
        ret.size = ret.size.wrapping_add(1);
        *ret.items.offset(c2rust_fresh14 as isize) = val;
        i = i.wrapping_add(1);
    }
    lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    return ret;
}
pub unsafe extern "C" fn nlua_pop_Array(
    mut lstate: *mut lua_State,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let table_props: LuaTableProps = nlua_check_type(lstate, err, kObjectTypeArray);
    if table_props.type_0 as ::core::ffi::c_uint
        != kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return Array {
            size: 0 as size_t,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        };
    }
    return nlua_pop_Array_unchecked(lstate, table_props, arena, err);
}
unsafe extern "C" fn nlua_pop_Dict_unchecked(
    mut lstate: *mut lua_State,
    table_props: LuaTableProps,
    mut ref_0: bool,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut ret: Dict = arena_dict(arena, table_props.string_keys_num);
    if table_props.string_keys_num == 0 as size_t {
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        return ret;
    }
    lua_pushnil(lstate);
    let mut i: size_t = 0 as size_t;
    while lua_next(lstate, -2 as ::core::ffi::c_int) != 0 && i < table_props.string_keys_num {
        if lua_type(lstate, -2 as ::core::ffi::c_int) == LUA_TSTRING {
            lua_pushvalue(lstate, -2 as ::core::ffi::c_int);
            let mut key: String_0 = nlua_pop_String(lstate, arena, err);
            if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
                let mut value: Object = nlua_pop_Object(lstate, ref_0, arena, err);
                let c2rust_fresh22 = ret.size;
                ret.size = ret.size.wrapping_add(1);
                *ret.items.offset(c2rust_fresh22 as isize) = key_value_pair {
                    key: key,
                    value: value,
                };
            } else {
                lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            }
            if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                if arena.is_null() {
                    api_free_dict(ret);
                }
                lua_settop(lstate, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                return Dict {
                    size: 0 as size_t,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<KeyValuePair>(),
                };
            }
            i = i.wrapping_add(1);
        } else {
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        }
    }
    lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    return ret;
}
pub unsafe extern "C" fn nlua_pop_Dict(
    mut lstate: *mut lua_State,
    mut ref_0: bool,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let table_props: LuaTableProps = nlua_check_type(lstate, err, kObjectTypeDict);
    if table_props.type_0 as ::core::ffi::c_uint
        != kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        return Dict {
            size: 0 as size_t,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        };
    }
    return nlua_pop_Dict_unchecked(lstate, table_props, ref_0, arena, err);
}
pub unsafe extern "C" fn nlua_pop_Object(
    lstate: *mut lua_State,
    mut ref_0: bool,
    mut arena: *mut Arena,
    err: *mut Error,
) -> Object {
    let mut ret: Object = object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let initial_size: ::core::ffi::c_int = lua_gettop(lstate);
    let mut stack: C2Rust_Unnamed_8 = C2Rust_Unnamed_8 {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<ObjPopStackItem>(),
        init_array: [ObjPopStackItem {
            obj: ::core::ptr::null_mut::<Object>(),
            container: false,
        }; 2],
    };
    stack.capacity = ::core::mem::size_of::<[ObjPopStackItem; 2]>()
        .wrapping_div(::core::mem::size_of::<ObjPopStackItem>())
        .wrapping_div(
            (::core::mem::size_of::<[ObjPopStackItem; 2]>()
                .wrapping_rem(::core::mem::size_of::<ObjPopStackItem>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    stack.size = 0 as size_t;
    stack.items = &raw mut stack.init_array as *mut ObjPopStackItem;
    if stack.size == stack.capacity {
        stack.capacity = if stack.capacity << 1 as ::core::ffi::c_int
            > ::core::mem::size_of::<[ObjPopStackItem; 2]>()
                .wrapping_div(::core::mem::size_of::<ObjPopStackItem>())
                .wrapping_div(
                    (::core::mem::size_of::<[ObjPopStackItem; 2]>()
                        .wrapping_rem(::core::mem::size_of::<ObjPopStackItem>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            stack.capacity << 1 as ::core::ffi::c_int
        } else {
            ::core::mem::size_of::<[ObjPopStackItem; 2]>()
                .wrapping_div(::core::mem::size_of::<ObjPopStackItem>())
                .wrapping_div(
                    (::core::mem::size_of::<[ObjPopStackItem; 2]>()
                        .wrapping_rem(::core::mem::size_of::<ObjPopStackItem>())
                        == 0) as ::core::ffi::c_int as size_t,
                )
        };
        stack.items = (if stack.capacity
            == ::core::mem::size_of::<[ObjPopStackItem; 2]>()
                .wrapping_div(::core::mem::size_of::<ObjPopStackItem>())
                .wrapping_div(
                    (::core::mem::size_of::<[ObjPopStackItem; 2]>()
                        .wrapping_rem(::core::mem::size_of::<ObjPopStackItem>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            if stack.items == &raw mut stack.init_array as *mut ObjPopStackItem {
                stack.items as *mut ::core::ffi::c_void
            } else {
                _memcpy_free(
                    &raw mut stack.init_array as *mut ObjPopStackItem as *mut ::core::ffi::c_void,
                    stack.items as *mut ::core::ffi::c_void,
                    stack
                        .size
                        .wrapping_mul(::core::mem::size_of::<ObjPopStackItem>()),
                )
            }
        } else {
            if stack.items == &raw mut stack.init_array as *mut ObjPopStackItem {
                memcpy(
                    xmalloc(
                        stack
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<ObjPopStackItem>()),
                    ),
                    stack.items as *const ::core::ffi::c_void,
                    stack
                        .size
                        .wrapping_mul(::core::mem::size_of::<ObjPopStackItem>()),
                )
            } else {
                xrealloc(
                    stack.items as *mut ::core::ffi::c_void,
                    stack
                        .capacity
                        .wrapping_mul(::core::mem::size_of::<ObjPopStackItem>()),
                )
            }
        }) as *mut ObjPopStackItem;
    } else {
    };
    let c2rust_fresh15 = stack.size;
    stack.size = stack.size.wrapping_add(1);
    *stack.items.offset(c2rust_fresh15 as isize) = ObjPopStackItem {
        obj: &raw mut ret,
        container: false,
    };
    while !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int)
        && stack.size != 0
    {
        stack.size = stack.size.wrapping_sub(1);
        let mut cur: ObjPopStackItem = *stack.items.offset(stack.size as isize);
        if cur.container {
            if lua_checkstack(lstate, lua_gettop(lstate) + 3 as ::core::ffi::c_int) == 0 {
                api_set_error(
                    err,
                    kErrorTypeException,
                    b"Lua failed to grow stack\0".as_ptr() as *const ::core::ffi::c_char,
                );
                break;
            } else if (*cur.obj).type_0 as ::core::ffi::c_uint
                == kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if (*cur.obj).data.dict.size == (*cur.obj).data.dict.capacity {
                    lua_settop(lstate, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                    continue;
                } else {
                    let mut next_key_found: bool = false_0 != 0;
                    while lua_next(lstate, -2 as ::core::ffi::c_int) != 0 {
                        if lua_type(lstate, -2 as ::core::ffi::c_int) == LUA_TSTRING {
                            next_key_found = true_0 != 0;
                            break;
                        } else {
                            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                        }
                    }
                    if next_key_found {
                        let mut len: size_t = 0;
                        let mut s: *const ::core::ffi::c_char =
                            lua_tolstring(lstate, -2 as ::core::ffi::c_int, &raw mut len);
                        let c2rust_fresh16 = (*cur.obj).data.dict.size;
                        (*cur.obj).data.dict.size = (*cur.obj).data.dict.size.wrapping_add(1);
                        let idx: size_t = c2rust_fresh16;
                        (*(*cur.obj).data.dict.items.offset(idx as isize)).key = arena_string(
                            arena,
                            String_0 {
                                data: s as *mut ::core::ffi::c_char,
                                size: len,
                            },
                        );
                        if stack.size == stack.capacity {
                            stack.capacity = if stack.capacity << 1 as ::core::ffi::c_int
                                > ::core::mem::size_of::<[ObjPopStackItem; 2]>()
                                    .wrapping_div(::core::mem::size_of::<ObjPopStackItem>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[ObjPopStackItem; 2]>()
                                            .wrapping_rem(::core::mem::size_of::<ObjPopStackItem>())
                                            == 0)
                                            as ::core::ffi::c_int
                                            as usize,
                                    ) {
                                stack.capacity << 1 as ::core::ffi::c_int
                            } else {
                                ::core::mem::size_of::<[ObjPopStackItem; 2]>()
                                    .wrapping_div(::core::mem::size_of::<ObjPopStackItem>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[ObjPopStackItem; 2]>()
                                            .wrapping_rem(::core::mem::size_of::<ObjPopStackItem>())
                                            == 0)
                                            as ::core::ffi::c_int
                                            as size_t,
                                    )
                            };
                            stack.items = (if stack.capacity
                                == ::core::mem::size_of::<[ObjPopStackItem; 2]>()
                                    .wrapping_div(::core::mem::size_of::<ObjPopStackItem>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[ObjPopStackItem; 2]>()
                                            .wrapping_rem(::core::mem::size_of::<ObjPopStackItem>())
                                            == 0)
                                            as ::core::ffi::c_int
                                            as usize,
                                    ) {
                                if stack.items == &raw mut stack.init_array as *mut ObjPopStackItem
                                {
                                    stack.items as *mut ::core::ffi::c_void
                                } else {
                                    _memcpy_free(
                                        &raw mut stack.init_array as *mut ObjPopStackItem
                                            as *mut ::core::ffi::c_void,
                                        stack.items as *mut ::core::ffi::c_void,
                                        stack
                                            .size
                                            .wrapping_mul(::core::mem::size_of::<ObjPopStackItem>()),
                                    )
                                }
                            } else {
                                if stack.items == &raw mut stack.init_array as *mut ObjPopStackItem
                                {
                                    memcpy(
                                        xmalloc(
                                            stack
                                                .capacity
                                                .wrapping_mul(::core::mem::size_of::<ObjPopStackItem>()),
                                        ),
                                        stack.items as *const ::core::ffi::c_void,
                                        stack
                                            .size
                                            .wrapping_mul(::core::mem::size_of::<ObjPopStackItem>()),
                                    )
                                } else {
                                    xrealloc(
                                        stack.items as *mut ::core::ffi::c_void,
                                        stack
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<ObjPopStackItem>()),
                                    )
                                }
                            }) as *mut ObjPopStackItem;
                        } else {
                        };
                        let c2rust_fresh17 = stack.size;
                        stack.size = stack.size.wrapping_add(1);
                        *stack.items.offset(c2rust_fresh17 as isize) = cur;
                        cur = ObjPopStackItem {
                            obj: &raw mut (*(*cur.obj).data.dict.items.offset(idx as isize)).value,
                            container: false,
                        };
                    } else {
                        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                        continue;
                    }
                }
            } else if (*cur.obj).data.array.size == (*cur.obj).data.array.capacity {
                lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                continue;
            } else {
                let c2rust_fresh18 = (*cur.obj).data.array.size;
                (*cur.obj).data.array.size = (*cur.obj).data.array.size.wrapping_add(1);
                let idx_0: size_t = c2rust_fresh18;
                lua_rawgeti(
                    lstate,
                    -1 as ::core::ffi::c_int,
                    idx_0 as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
                );
                if stack.size == stack.capacity {
                    stack.capacity = if stack.capacity << 1 as ::core::ffi::c_int
                        > ::core::mem::size_of::<[ObjPopStackItem; 2]>()
                            .wrapping_div(::core::mem::size_of::<ObjPopStackItem>())
                            .wrapping_div(
                                (::core::mem::size_of::<[ObjPopStackItem; 2]>()
                                    .wrapping_rem(::core::mem::size_of::<ObjPopStackItem>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        stack.capacity << 1 as ::core::ffi::c_int
                    } else {
                        ::core::mem::size_of::<[ObjPopStackItem; 2]>()
                            .wrapping_div(::core::mem::size_of::<ObjPopStackItem>())
                            .wrapping_div(
                                (::core::mem::size_of::<[ObjPopStackItem; 2]>()
                                    .wrapping_rem(::core::mem::size_of::<ObjPopStackItem>())
                                    == 0) as ::core::ffi::c_int
                                    as size_t,
                            )
                    };
                    stack.items = (if stack.capacity
                        == ::core::mem::size_of::<[ObjPopStackItem; 2]>()
                            .wrapping_div(::core::mem::size_of::<ObjPopStackItem>())
                            .wrapping_div(
                                (::core::mem::size_of::<[ObjPopStackItem; 2]>()
                                    .wrapping_rem(::core::mem::size_of::<ObjPopStackItem>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        if stack.items == &raw mut stack.init_array as *mut ObjPopStackItem {
                            stack.items as *mut ::core::ffi::c_void
                        } else {
                            _memcpy_free(
                                &raw mut stack.init_array as *mut ObjPopStackItem
                                    as *mut ::core::ffi::c_void,
                                stack.items as *mut ::core::ffi::c_void,
                                stack
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<ObjPopStackItem>()),
                            )
                        }
                    } else {
                        if stack.items == &raw mut stack.init_array as *mut ObjPopStackItem {
                            memcpy(
                                xmalloc(
                                    stack
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<ObjPopStackItem>()),
                                ),
                                stack.items as *const ::core::ffi::c_void,
                                stack
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<ObjPopStackItem>()),
                            )
                        } else {
                            xrealloc(
                                stack.items as *mut ::core::ffi::c_void,
                                stack
                                    .capacity
                                    .wrapping_mul(::core::mem::size_of::<ObjPopStackItem>()),
                            )
                        }
                    }) as *mut ObjPopStackItem;
                } else {
                };
                let c2rust_fresh19 = stack.size;
                stack.size = stack.size.wrapping_add(1);
                *stack.items.offset(c2rust_fresh19 as isize) = cur;
                cur = ObjPopStackItem {
                    obj: (*cur.obj).data.array.items.offset(idx_0 as isize),
                    container: false,
                };
            }
        }
        '_c2rust_label: {
            if !cur.container {
            } else {
                __assert_fail(
                    b"!cur.container\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/lua/converter.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1114 as ::core::ffi::c_uint,
                    b"Object nlua_pop_Object(lua_State *const, _Bool, Arena *, Error *const)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        *cur.obj = object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
        's_341: {
            match lua_type(lstate, -1 as ::core::ffi::c_int) {
                LUA_TNIL => {
                    break 's_341;
                }
                LUA_TBOOLEAN => {
                    *cur.obj = object {
                        type_0: kObjectTypeBoolean,
                        data: C2Rust_Unnamed {
                            boolean: lua_toboolean(lstate, -1 as ::core::ffi::c_int) != 0,
                        },
                    };
                    break 's_341;
                }
                LUA_TSTRING => {
                    let mut len_0: size_t = 0;
                    let mut s_0: *const ::core::ffi::c_char =
                        lua_tolstring(lstate, -1 as ::core::ffi::c_int, &raw mut len_0);
                    *cur.obj = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed {
                            string: arena_string(
                                arena,
                                String_0 {
                                    data: s_0 as *mut ::core::ffi::c_char,
                                    size: len_0,
                                },
                            ),
                        },
                    };
                    break 's_341;
                }
                LUA_TNUMBER => {
                    let n: lua_Number = lua_tonumber(lstate, -1 as ::core::ffi::c_int);
                    if n > API_INTEGER_MAX as lua_Number
                        || n < API_INTEGER_MIN as lua_Number
                        || n as Integer as lua_Number != n
                    {
                        *cur.obj = object {
                            type_0: kObjectTypeFloat,
                            data: C2Rust_Unnamed { floating: n },
                        };
                    } else {
                        *cur.obj = object {
                            type_0: kObjectTypeInteger,
                            data: C2Rust_Unnamed {
                                integer: n as Integer,
                            },
                        };
                    }
                    break 's_341;
                }
                LUA_TTABLE => {
                    let table_props: LuaTableProps = nlua_traverse_table(lstate);
                    match table_props.type_0 as ::core::ffi::c_uint {
                        5 => {
                            *cur.obj = object {
                                type_0: kObjectTypeArray,
                                data: C2Rust_Unnamed {
                                    array: Array {
                                        size: 0 as size_t,
                                        capacity: 0 as size_t,
                                        items: ::core::ptr::null_mut::<Object>(),
                                    },
                                },
                            };
                            if table_props.maxidx != 0 as size_t {
                                (*cur.obj).data.array = arena_array(arena, table_props.maxidx);
                                cur.container = true_0 != 0;
                                '_c2rust_label_0: {
                                    if stack.size < 18446744073709551615 as size_t {
                                    } else {
                                        __assert_fail(
                                            b"kv_size(stack) < SIZE_MAX\0".as_ptr()
                                                as *const ::core::ffi::c_char,
                                            b"src/nvim/lua/converter.rs\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                            1147 as ::core::ffi::c_uint,
                                            b"Object nlua_pop_Object(lua_State *const, _Bool, Arena *, Error *const)\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        );
                                    }
                                };
                                if stack.size == stack.capacity {
                                    stack.capacity = if stack.capacity << 1 as ::core::ffi::c_int
                                        > ::core::mem::size_of::<[ObjPopStackItem; 2]>()
                                            .wrapping_div(::core::mem::size_of::<ObjPopStackItem>())
                                            .wrapping_div(
                                                (::core::mem::size_of::<[ObjPopStackItem; 2]>()
                                                    .wrapping_rem(::core::mem::size_of::<
                                                        ObjPopStackItem,
                                                    >(
                                                    ))
                                                    == 0)
                                                    as ::core::ffi::c_int
                                                    as usize,
                                            ) {
                                        stack.capacity << 1 as ::core::ffi::c_int
                                    } else {
                                        ::core::mem::size_of::<[ObjPopStackItem; 2]>()
                                            .wrapping_div(::core::mem::size_of::<ObjPopStackItem>())
                                            .wrapping_div(
                                                (::core::mem::size_of::<[ObjPopStackItem; 2]>()
                                                    .wrapping_rem(::core::mem::size_of::<
                                                        ObjPopStackItem,
                                                    >(
                                                    ))
                                                    == 0)
                                                    as ::core::ffi::c_int
                                                    as size_t,
                                            )
                                    };
                                    stack.items = (if stack.capacity
                                        == ::core::mem::size_of::<[ObjPopStackItem; 2]>()
                                            .wrapping_div(::core::mem::size_of::<ObjPopStackItem>())
                                            .wrapping_div(
                                                (::core::mem::size_of::<[ObjPopStackItem; 2]>()
                                                    .wrapping_rem(::core::mem::size_of::<
                                                        ObjPopStackItem,
                                                    >(
                                                    ))
                                                    == 0)
                                                    as ::core::ffi::c_int
                                                    as usize,
                                            ) {
                                        if stack.items
                                            == &raw mut stack.init_array as *mut ObjPopStackItem
                                        {
                                            stack.items as *mut ::core::ffi::c_void
                                        } else {
                                            _memcpy_free(
                                                &raw mut stack.init_array as *mut ObjPopStackItem
                                                    as *mut ::core::ffi::c_void,
                                                stack.items as *mut ::core::ffi::c_void,
                                                stack.size.wrapping_mul(::core::mem::size_of::<
                                                    ObjPopStackItem,
                                                >(
                                                )),
                                            )
                                        }
                                    } else {
                                        if stack.items
                                            == &raw mut stack.init_array as *mut ObjPopStackItem
                                        {
                                            memcpy(
                                                xmalloc(stack.capacity.wrapping_mul(
                                                    ::core::mem::size_of::<ObjPopStackItem>(),
                                                )),
                                                stack.items as *const ::core::ffi::c_void,
                                                stack.size.wrapping_mul(::core::mem::size_of::<
                                                    ObjPopStackItem,
                                                >(
                                                )),
                                            )
                                        } else {
                                            xrealloc(
                                                stack.items as *mut ::core::ffi::c_void,
                                                stack.capacity.wrapping_mul(
                                                    ::core::mem::size_of::<ObjPopStackItem>(),
                                                ),
                                            )
                                        }
                                    })
                                        as *mut ObjPopStackItem;
                                } else {
                                };
                                let c2rust_fresh20 = stack.size;
                                stack.size = stack.size.wrapping_add(1);
                                *stack.items.offset(c2rust_fresh20 as isize) = cur;
                            }
                        }
                        6 => {
                            *cur.obj = object {
                                type_0: kObjectTypeDict,
                                data: C2Rust_Unnamed {
                                    dict: Dict {
                                        size: 0 as size_t,
                                        capacity: 0 as size_t,
                                        items: ::core::ptr::null_mut::<KeyValuePair>(),
                                    },
                                },
                            };
                            if table_props.string_keys_num != 0 as size_t {
                                (*cur.obj).data.dict =
                                    arena_dict(arena, table_props.string_keys_num);
                                cur.container = true_0 != 0;
                                '_c2rust_label_1: {
                                    if stack.size < 18446744073709551615 as size_t {
                                    } else {
                                        __assert_fail(
                                            b"kv_size(stack) < SIZE_MAX\0".as_ptr()
                                                as *const ::core::ffi::c_char,
                                            b"src/nvim/lua/converter.rs\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                            1156 as ::core::ffi::c_uint,
                                            b"Object nlua_pop_Object(lua_State *const, _Bool, Arena *, Error *const)\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        );
                                    }
                                };
                                if stack.size == stack.capacity {
                                    stack.capacity = if stack.capacity << 1 as ::core::ffi::c_int
                                        > ::core::mem::size_of::<[ObjPopStackItem; 2]>()
                                            .wrapping_div(::core::mem::size_of::<ObjPopStackItem>())
                                            .wrapping_div(
                                                (::core::mem::size_of::<[ObjPopStackItem; 2]>()
                                                    .wrapping_rem(::core::mem::size_of::<
                                                        ObjPopStackItem,
                                                    >(
                                                    ))
                                                    == 0)
                                                    as ::core::ffi::c_int
                                                    as usize,
                                            ) {
                                        stack.capacity << 1 as ::core::ffi::c_int
                                    } else {
                                        ::core::mem::size_of::<[ObjPopStackItem; 2]>()
                                            .wrapping_div(::core::mem::size_of::<ObjPopStackItem>())
                                            .wrapping_div(
                                                (::core::mem::size_of::<[ObjPopStackItem; 2]>()
                                                    .wrapping_rem(::core::mem::size_of::<
                                                        ObjPopStackItem,
                                                    >(
                                                    ))
                                                    == 0)
                                                    as ::core::ffi::c_int
                                                    as size_t,
                                            )
                                    };
                                    stack.items = (if stack.capacity
                                        == ::core::mem::size_of::<[ObjPopStackItem; 2]>()
                                            .wrapping_div(::core::mem::size_of::<ObjPopStackItem>())
                                            .wrapping_div(
                                                (::core::mem::size_of::<[ObjPopStackItem; 2]>()
                                                    .wrapping_rem(::core::mem::size_of::<
                                                        ObjPopStackItem,
                                                    >(
                                                    ))
                                                    == 0)
                                                    as ::core::ffi::c_int
                                                    as usize,
                                            ) {
                                        if stack.items
                                            == &raw mut stack.init_array as *mut ObjPopStackItem
                                        {
                                            stack.items as *mut ::core::ffi::c_void
                                        } else {
                                            _memcpy_free(
                                                &raw mut stack.init_array as *mut ObjPopStackItem
                                                    as *mut ::core::ffi::c_void,
                                                stack.items as *mut ::core::ffi::c_void,
                                                stack.size.wrapping_mul(::core::mem::size_of::<
                                                    ObjPopStackItem,
                                                >(
                                                )),
                                            )
                                        }
                                    } else {
                                        if stack.items
                                            == &raw mut stack.init_array as *mut ObjPopStackItem
                                        {
                                            memcpy(
                                                xmalloc(stack.capacity.wrapping_mul(
                                                    ::core::mem::size_of::<ObjPopStackItem>(),
                                                )),
                                                stack.items as *const ::core::ffi::c_void,
                                                stack.size.wrapping_mul(::core::mem::size_of::<
                                                    ObjPopStackItem,
                                                >(
                                                )),
                                            )
                                        } else {
                                            xrealloc(
                                                stack.items as *mut ::core::ffi::c_void,
                                                stack.capacity.wrapping_mul(
                                                    ::core::mem::size_of::<ObjPopStackItem>(),
                                                ),
                                            )
                                        }
                                    })
                                        as *mut ObjPopStackItem;
                                } else {
                                };
                                let c2rust_fresh21 = stack.size;
                                stack.size = stack.size.wrapping_add(1);
                                *stack.items.offset(c2rust_fresh21 as isize) = cur;
                                lua_pushnil(lstate);
                            }
                        }
                        3 => {
                            *cur.obj = object {
                                type_0: kObjectTypeFloat,
                                data: C2Rust_Unnamed {
                                    floating: table_props.val,
                                },
                            };
                        }
                        0 => {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"Cannot convert given Lua table\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                        _ => {
                            abort();
                        }
                    }
                    break 's_341;
                }
                LUA_TFUNCTION => {
                    if ref_0 {
                        *cur.obj = object {
                            type_0: kObjectTypeLuaRef,
                            data: C2Rust_Unnamed {
                                luaref: nlua_ref_global(lstate, -1 as ::core::ffi::c_int),
                            },
                        };
                        break 's_341;
                    }
                }
                LUA_TUSERDATA => {
                    nlua_pushref(lstate, (*nlua_global_refs.get()).nil_ref);
                    let mut is_nil: bool =
                        lua_rawequal(lstate, -2 as ::core::ffi::c_int, -1 as ::core::ffi::c_int)
                            != 0;
                    lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                    if is_nil {
                        *cur.obj = object {
                            type_0: kObjectTypeNil,
                            data: C2Rust_Unnamed { boolean: false },
                        };
                    } else {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            b"Cannot convert userdata\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                    break 's_341;
                }
                _ => {}
            }
            api_set_error(
                err,
                kErrorTypeValidation,
                b"Cannot convert given Lua type\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if !cur.container {
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        }
    }
    if stack.items != &raw mut stack.init_array as *mut ObjPopStackItem {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut stack.items as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
    }
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        if arena.is_null() {
            api_free_object(ret);
        }
        ret = object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
        lua_settop(
            lstate,
            -(lua_gettop(lstate) - initial_size + 1 as ::core::ffi::c_int)
                - 1 as ::core::ffi::c_int,
        );
    }
    '_c2rust_label_2: {
        if lua_gettop(lstate) == initial_size - 1 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"lua_gettop(lstate) == initial_size - 1\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/lua/converter.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1211 as ::core::ffi::c_uint,
                b"Object nlua_pop_Object(lua_State *const, _Bool, Arena *, Error *const)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    return ret;
}
pub unsafe extern "C" fn nlua_pop_LuaRef(
    lstate: *mut lua_State,
    mut _arena: *mut Arena,
    mut _err: *mut Error,
) -> LuaRef {
    let mut rv: LuaRef = nlua_ref_global(lstate, -1 as ::core::ffi::c_int);
    lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    return rv;
}
pub unsafe extern "C" fn nlua_pop_handle(
    mut lstate: *mut lua_State,
    mut _arena: *mut Arena,
    mut err: *mut Error,
) -> handle_T {
    let mut ret: handle_T = 0;
    if lua_type(lstate, -1 as ::core::ffi::c_int) != LUA_TNUMBER {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Expected Lua number\0".as_ptr() as *const ::core::ffi::c_char,
        );
        ret = -1 as ::core::ffi::c_int;
    } else {
        ret = lua_tonumber(lstate, -1 as ::core::ffi::c_int) as handle_T;
    }
    lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    return ret;
}
pub unsafe extern "C" fn nlua_init_types(lstate: *mut lua_State) {
    lua_pushlstring(
        lstate,
        b"type_idx\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
    );
    nlua_push_type_idx(lstate);
    lua_rawset(lstate, -3 as ::core::ffi::c_int);
    lua_pushlstring(
        lstate,
        b"val_idx\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
    );
    nlua_push_val_idx(lstate);
    lua_rawset(lstate, -3 as ::core::ffi::c_int);
    lua_pushlstring(
        lstate,
        b"types\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
    );
    lua_createtable(lstate, 0 as ::core::ffi::c_int, 3 as ::core::ffi::c_int);
    lua_pushlstring(
        lstate,
        b"float\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
    );
    lua_pushnumber(lstate, kObjectTypeFloat as ::core::ffi::c_int as lua_Number);
    lua_rawset(lstate, -3 as ::core::ffi::c_int);
    lua_pushnumber(lstate, kObjectTypeFloat as ::core::ffi::c_int as lua_Number);
    lua_pushlstring(
        lstate,
        b"float\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
    );
    lua_rawset(lstate, -3 as ::core::ffi::c_int);
    lua_pushlstring(
        lstate,
        b"array\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
    );
    lua_pushnumber(lstate, kObjectTypeArray as ::core::ffi::c_int as lua_Number);
    lua_rawset(lstate, -3 as ::core::ffi::c_int);
    lua_pushnumber(lstate, kObjectTypeArray as ::core::ffi::c_int as lua_Number);
    lua_pushlstring(
        lstate,
        b"array\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
    );
    lua_rawset(lstate, -3 as ::core::ffi::c_int);
    lua_pushlstring(
        lstate,
        b"dictionary\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as size_t),
    );
    lua_pushnumber(lstate, kObjectTypeDict as ::core::ffi::c_int as lua_Number);
    lua_rawset(lstate, -3 as ::core::ffi::c_int);
    lua_pushnumber(lstate, kObjectTypeDict as ::core::ffi::c_int as lua_Number);
    lua_pushlstring(
        lstate,
        b"dictionary\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as size_t),
    );
    lua_rawset(lstate, -3 as ::core::ffi::c_int);
    lua_rawset(lstate, -3 as ::core::ffi::c_int);
}
pub unsafe extern "C" fn nlua_pop_keydict(
    mut L: *mut lua_State,
    mut retval: *mut ::core::ffi::c_void,
    mut hashy: FieldHashfn,
    mut err_opt: *mut *mut ::core::ffi::c_char,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    if !(lua_type(L, -1 as ::core::ffi::c_int) == LUA_TTABLE) {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Expected Lua table\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_settop(L, -(-1 as ::core::ffi::c_int) - 1 as ::core::ffi::c_int);
        return;
    }
    lua_pushnil(L);
    while lua_next(L, -2 as ::core::ffi::c_int) != 0 {
        let mut len: size_t = 0;
        let mut s: *const ::core::ffi::c_char =
            lua_tolstring(L, -2 as ::core::ffi::c_int, &raw mut len);
        let mut field: *mut KeySetLink = hashy.expect("non-null function pointer")(s, len);
        if field.is_null() {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"invalid key: %.*s\0".as_ptr() as *const ::core::ffi::c_char,
                len as ::core::ffi::c_int,
                s,
            );
            lua_settop(L, -3 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            return;
        }
        if (*field).opt_index >= 0 as ::core::ffi::c_int {
            let mut ks: *mut OptKeySet = retval as *mut OptKeySet;
            (*ks).is_set_ = ((*ks).is_set_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << (*field).opt_index)
                as OptionalKeys;
        }
        let mut mem: *mut ::core::ffi::c_char =
            (retval as *mut ::core::ffi::c_char).offset((*field).ptr_off as isize);
        if (*field).type_0 == kObjectTypeNil as ::core::ffi::c_int {
            *(mem as *mut Object) = nlua_pop_Object(L, true_0 != 0, arena, err);
        } else if (*field).type_0 == kObjectTypeInteger as ::core::ffi::c_int {
            if (*field).is_hlgroup as ::core::ffi::c_int != 0
                && lua_type(L, -1 as ::core::ffi::c_int) == LUA_TSTRING
            {
                let mut name_len: size_t = 0;
                let mut name: *const ::core::ffi::c_char =
                    lua_tolstring(L, -1 as ::core::ffi::c_int, &raw mut name_len);
                lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                *(mem as *mut Integer) = (if name_len > 0 as size_t {
                    syn_check_group(name, name_len)
                } else {
                    0 as ::core::ffi::c_int
                }) as Integer;
            } else {
                *(mem as *mut Integer) = nlua_pop_Integer(L, arena, err);
            }
        } else if (*field).type_0 == kObjectTypeBoolean as ::core::ffi::c_int {
            *(mem as *mut Boolean) = nlua_pop_Boolean_strict(L, err);
        } else if (*field).type_0 == kObjectTypeString as ::core::ffi::c_int {
            *(mem as *mut String_0) = nlua_pop_String(L, arena, err);
        } else if (*field).type_0 == kObjectTypeFloat as ::core::ffi::c_int {
            *(mem as *mut Float) = nlua_pop_Float(L, arena, err);
        } else if (*field).type_0 == kObjectTypeBuffer as ::core::ffi::c_int
            || (*field).type_0 == kObjectTypeWindow as ::core::ffi::c_int
            || (*field).type_0 == kObjectTypeTabpage as ::core::ffi::c_int
        {
            *(mem as *mut handle_T) = nlua_pop_handle(L, arena, err);
        } else if (*field).type_0 == kObjectTypeArray as ::core::ffi::c_int {
            *(mem as *mut Array) = nlua_pop_Array(L, arena, err);
        } else if (*field).type_0 == kObjectTypeDict as ::core::ffi::c_int {
            *(mem as *mut Dict) = nlua_pop_Dict(L, false_0 != 0, arena, err);
        } else if (*field).type_0 == kObjectTypeLuaRef as ::core::ffi::c_int {
            *(mem as *mut LuaRef) = nlua_pop_LuaRef(L, arena, err);
        } else {
            abort();
        }
        if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            continue;
        }
        *err_opt = (*field).str;
        break;
    }
    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
}
pub unsafe extern "C" fn nlua_push_keydict(
    mut L: *mut lua_State,
    mut value: *mut ::core::ffi::c_void,
    mut table: *mut KeySetLink,
) {
    lua_createtable(L, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    let mut i: size_t = 0 as size_t;
    while !(*table.offset(i as isize)).str.is_null() {
        let mut field: *mut KeySetLink = table.offset(i as isize);
        let mut is_set: bool = true_0 != 0;
        if (*field).opt_index >= 0 as ::core::ffi::c_int {
            let mut ks: *mut OptKeySet = value as *mut OptKeySet;
            is_set = (*ks).is_set_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << (*field).opt_index
                != 0;
        }
        if is_set {
            let mut mem: *mut ::core::ffi::c_char =
                (value as *mut ::core::ffi::c_char).offset((*field).ptr_off as isize);
            lua_pushstring(L, (*field).str);
            if (*field).type_0 == kObjectTypeNil as ::core::ffi::c_int {
                nlua_push_Object(L, mem as *mut Object, 0 as ::core::ffi::c_int);
            } else if (*field).type_0 == kObjectTypeInteger as ::core::ffi::c_int {
                lua_pushinteger(L, *(mem as *mut Integer) as lua_Integer);
            } else if (*field).type_0 == kObjectTypeBuffer as ::core::ffi::c_int
                || (*field).type_0 == kObjectTypeWindow as ::core::ffi::c_int
                || (*field).type_0 == kObjectTypeTabpage as ::core::ffi::c_int
            {
                lua_pushinteger(L, *(mem as *mut handle_T) as lua_Integer);
            } else if (*field).type_0 == kObjectTypeFloat as ::core::ffi::c_int {
                lua_pushnumber(L, *(mem as *mut Float) as lua_Number);
            } else if (*field).type_0 == kObjectTypeBoolean as ::core::ffi::c_int {
                lua_pushboolean(L, *(mem as *mut Boolean) as ::core::ffi::c_int);
            } else if (*field).type_0 == kObjectTypeString as ::core::ffi::c_int {
                nlua_push_String(L, *(mem as *mut String_0), 0 as ::core::ffi::c_int);
            } else if (*field).type_0 == kObjectTypeArray as ::core::ffi::c_int {
                nlua_push_Array(L, *(mem as *mut Array), 0 as ::core::ffi::c_int);
            } else if (*field).type_0 == kObjectTypeDict as ::core::ffi::c_int {
                nlua_push_Dict(L, *(mem as *mut Dict), 0 as ::core::ffi::c_int);
            } else if (*field).type_0 == kObjectTypeLuaRef as ::core::ffi::c_int {
                nlua_pushref(L, *(mem as *mut LuaRef));
            } else {
                abort();
            }
            lua_rawset(L, -3 as ::core::ffi::c_int);
        }
        i = i.wrapping_add(1);
    }
}
pub static _typval_encode_lua_nodict_var: GlobalCell<*const dict_T> =
    GlobalCell::new(::core::ptr::null::<dict_T>());
#[inline(always)]
unsafe extern "C" fn _typval_encode_lua_check_self_reference(
    lstate: *mut lua_State,
    val: *mut ::core::ffi::c_void,
    val_copyID: *mut ::core::ffi::c_int,
    mpstack: *const MPConvStack,
    copyID: ::core::ffi::c_int,
    conv_type: MPConvStackValType,
    _objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if *val_copyID == copyID {
        let mut backref: size_t = (*mpstack).size;
        while backref != 0 {
            let mpval: MPConvStackVal = *(*mpstack)
                .items
                .offset(backref.wrapping_sub(1 as size_t) as isize);
            if mpval.type_0 as ::core::ffi::c_uint == conv_type as ::core::ffi::c_uint {
                if if conv_type as ::core::ffi::c_uint
                    == kMPConvDict as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    (mpval.data.d.dict as *mut ::core::ffi::c_void == val) as ::core::ffi::c_int
                } else {
                    (mpval.data.l.list as *mut ::core::ffi::c_void == val) as ::core::ffi::c_int
                } != 0
                {
                    lua_pushvalue(
                        lstate,
                        -((*mpstack)
                            .size
                            .wrapping_sub(backref)
                            .wrapping_add(1 as size_t)
                            .wrapping_mul(2 as size_t)
                            as ::core::ffi::c_int),
                    );
                    break;
                }
            }
            backref = backref.wrapping_sub(1);
        }
        return OK;
    }
    *val_copyID = copyID;
    return NOTDONE;
}
unsafe extern "C" fn _typval_encode_lua_convert_one_value(
    lstate: *mut lua_State,
    mpstack: *mut MPConvStack,
    _cur_mpsv: *mut MPConvStackVal,
    tv: *mut typval_T,
    copyID: ::core::ffi::c_int,
    objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    '_typval_encode_stop_converting_one_item: {
        match (*tv).v_type as ::core::ffi::c_uint {
            2 => {
                lua_pushlstring(lstate, (*tv).vval.v_string, tv_strlen(tv));
            }
            1 => {
                lua_pushnumber(lstate, (*tv).vval.v_number as lua_Number);
            }
            6 => {
                lua_pushnumber(lstate, (*tv).vval.v_float);
            }
            10 => {
                let blob_: *const blob_T = (*tv).vval.v_blob;
                lua_pushlstring(
                    lstate,
                    (if !blob_.is_null() {
                        (*blob_).bv_ga.ga_data
                    } else {
                        b"\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void
                    }) as *const ::core::ffi::c_char,
                    tv_blob_len((*tv).vval.v_blob) as size_t,
                );
            }
            3 => {
                let fun_: *const ::core::ffi::c_char = (*tv).vval.v_string;
                let mut fp: *mut ufunc_T = ::core::ptr::null_mut::<ufunc_T>();
                if !fun_.is_null()
                    && {
                        fp = find_func(fun_);
                        !fp.is_null()
                    }
                    && (*fp).uf_flags & FC_LUAREF != 0
                {
                    nlua_pushref(lstate, (*fp).uf_luaref);
                } else if typval_conv_special.get() {
                    lua_pushnil(lstate);
                } else {
                    nlua_pushref(lstate, (*nlua_global_refs.get()).nil_ref);
                }
            }
            9 => {
                let pt: *mut partial_T = (*tv).vval.v_partial;
                let fun: *mut ::core::ffi::c_char = if pt.is_null() {
                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                } else {
                    partial_name(pt)
                };
                let _prefix: *const ::core::ffi::c_char = if !fun.is_null()
                    && !pt.is_null()
                    && (*pt).pt_name.is_null()
                    && (*fun.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                        >= 'A' as ::core::ffi::c_uint
                        && *fun.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                            <= 'Z' as ::core::ffi::c_uint)
                {
                    b"g:\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                };
                let fun__0: *const ::core::ffi::c_char = fun;
                let mut fp_0: *mut ufunc_T = ::core::ptr::null_mut::<ufunc_T>();
                if !fun__0.is_null()
                    && {
                        fp_0 = find_func(fun__0);
                        !fp_0.is_null()
                    }
                    && (*fp_0).uf_flags & FC_LUAREF != 0
                {
                    nlua_pushref(lstate, (*fp_0).uf_luaref);
                } else if typval_conv_special.get() {
                    lua_pushnil(lstate);
                } else {
                    nlua_pushref(lstate, (*nlua_global_refs.get()).nil_ref);
                }
            }
            4 => {
                if (*tv).vval.v_list.is_null()
                    || tv_list_len((*tv).vval.v_list) == 0 as ::core::ffi::c_int
                {
                    lua_createtable(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
                } else {
                    let saved_copyID: ::core::ffi::c_int = tv_list_copyid((*tv).vval.v_list);
                    let te_csr_ret: ::core::ffi::c_int = _typval_encode_lua_check_self_reference(
                        lstate,
                        (*tv).vval.v_list as *mut ::core::ffi::c_void,
                        &raw mut (*(*tv).vval.v_list).lv_copyID,
                        mpstack,
                        copyID,
                        kMPConvList,
                        objname,
                    );
                    if te_csr_ret != NOTDONE {
                        return te_csr_ret;
                    }
                    if lua_checkstack(lstate, lua_gettop(lstate) + 3 as ::core::ffi::c_int) == 0 {
                        semsg(
                            gettext(b"E5102: Lua failed to grow stack to %i\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            lua_gettop(lstate) + 3 as ::core::ffi::c_int,
                        );
                        return false_0;
                    }
                    lua_createtable(
                        lstate,
                        tv_list_len((*tv).vval.v_list),
                        0 as ::core::ffi::c_int,
                    );
                    lua_pushnumber(lstate, 1 as ::core::ffi::c_int as lua_Number);
                    '_c2rust_label: {
                        if saved_copyID != copyID {
                        } else {
                            __assert_fail(
                                b"saved_copyID != copyID\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/lua/converter.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                383 as ::core::ffi::c_uint,
                                b"int _typval_encode_lua_convert_one_value(lua_State *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    if (*mpstack).size == (*mpstack).capacity {
                        (*mpstack).capacity = if (*mpstack).capacity << 1 as ::core::ffi::c_int
                            > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            (*mpstack).capacity << 1 as ::core::ffi::c_int
                        } else {
                            ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                )
                        };
                        (*mpstack).items = (if (*mpstack).capacity
                            == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            if (*mpstack).items
                                == &raw mut (*mpstack).init_array as *mut MPConvStackVal
                            {
                                (*mpstack).items as *mut ::core::ffi::c_void
                            } else {
                                _memcpy_free(
                                    &raw mut (*mpstack).init_array as *mut MPConvStackVal
                                        as *mut ::core::ffi::c_void,
                                    (*mpstack).items as *mut ::core::ffi::c_void,
                                    (*mpstack)
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                )
                            }
                        } else {
                            if (*mpstack).items
                                == &raw mut (*mpstack).init_array as *mut MPConvStackVal
                            {
                                memcpy(
                                    xmalloc(
                                        (*mpstack)
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                    ),
                                    (*mpstack).items as *const ::core::ffi::c_void,
                                    (*mpstack)
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                )
                            } else {
                                xrealloc(
                                    (*mpstack).items as *mut ::core::ffi::c_void,
                                    (*mpstack)
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                )
                            }
                        }) as *mut MPConvStackVal;
                    } else {
                    };
                    let c2rust_fresh10 = (*mpstack).size;
                    (*mpstack).size = (*mpstack).size.wrapping_add(1);
                    *(*mpstack).items.offset(c2rust_fresh10 as isize) = MPConvStackVal {
                        type_0: kMPConvList,
                        tv: tv,
                        saved_copyID: saved_copyID,
                        data: C2Rust_Unnamed_1 {
                            l: C2Rust_Unnamed_4 {
                                list: (*tv).vval.v_list,
                                li: tv_list_first((*tv).vval.v_list),
                            },
                        },
                    };
                }
            }
            7 => match (*tv).vval.v_bool as ::core::ffi::c_uint {
                1 | 0 => {
                    lua_pushboolean(
                        lstate,
                        ((*tv).vval.v_bool as ::core::ffi::c_uint
                            == kBoolVarTrue as ::core::ffi::c_int as ::core::ffi::c_uint)
                            as ::core::ffi::c_int,
                    );
                }
                _ => {}
            },
            8 => match (*tv).vval.v_special as ::core::ffi::c_uint {
                0 => {
                    if typval_conv_special.get() {
                        lua_pushnil(lstate);
                    } else {
                        nlua_pushref(lstate, (*nlua_global_refs.get()).nil_ref);
                    }
                }
                _ => {}
            },
            5 => {
                if (*tv).vval.v_dict.is_null()
                    || (*(*tv).vval.v_dict).dv_hashtab.ht_used == 0 as size_t
                {
                    if typval_conv_special.get() {
                        nlua_create_typed_table(lstate, 0 as size_t, 0 as size_t, kObjectTypeDict);
                    } else {
                        lua_createtable(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
                        nlua_pushref(lstate, (*nlua_global_refs.get()).empty_dict_ref);
                        lua_setmetatable(lstate, -2 as ::core::ffi::c_int);
                    }
                } else {
                    let mut type_di: *const dictitem_T = ::core::ptr::null::<dictitem_T>();
                    let mut val_di: *const dictitem_T = ::core::ptr::null::<dictitem_T>();
                    's_887: {
                        if TYPVAL_ENCODE_ALLOW_SPECIALS != 0
                            && (*(*tv).vval.v_dict).dv_hashtab.ht_used == 2 as size_t
                            && {
                                type_di = tv_dict_find(
                                    (*tv).vval.v_dict,
                                    b"_TYPE\0".as_ptr() as *const ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 6]>()
                                        .wrapping_sub(1 as usize)
                                        as ptrdiff_t,
                                );
                                !type_di.is_null()
                            }
                            && (*type_di).di_tv.v_type as ::core::ffi::c_uint
                                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                            && {
                                val_di = tv_dict_find(
                                    (*tv).vval.v_dict,
                                    b"_VAL\0".as_ptr() as *const ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                                        .wrapping_sub(1 as usize)
                                        as ptrdiff_t,
                                );
                                !val_di.is_null()
                            }
                        {
                            let mut i: size_t = 0;
                            i = 0 as size_t;
                            while i < ::core::mem::size_of::<[*const list_T; 8]>()
                                .wrapping_div(::core::mem::size_of::<*const list_T>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[*const list_T; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<*const list_T>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                )
                            {
                                if (*type_di).di_tv.vval.v_list
                                    == (*eval_msgpack_type_lists.ptr())[i as usize] as *mut list_T
                                {
                                    break;
                                }
                                i = i.wrapping_add(1);
                            }
                            if i != ::core::mem::size_of::<[*const list_T; 8]>()
                                .wrapping_div(::core::mem::size_of::<*const list_T>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[*const list_T; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<*const list_T>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                )
                            {
                                match i as MessagePackType as ::core::ffi::c_uint {
                                    0 => {
                                        if typval_conv_special.get() {
                                            lua_pushnil(lstate);
                                        } else {
                                            nlua_pushref(lstate, (*nlua_global_refs.get()).nil_ref);
                                        }
                                        break '_typval_encode_stop_converting_one_item;
                                    }
                                    1 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_NUMBER as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            lua_pushboolean(
                                                lstate,
                                                ((*val_di).di_tv.vval.v_number != 0)
                                                    as ::core::ffi::c_int,
                                            );
                                            break '_typval_encode_stop_converting_one_item;
                                        }
                                    }
                                    2 => {
                                        let mut val_list: *const list_T =
                                            ::core::ptr::null::<list_T>();
                                        let mut sign: varnumber_T = 0;
                                        let mut highest_bits: varnumber_T = 0;
                                        let mut high_bits: varnumber_T = 0;
                                        let mut low_bits: varnumber_T = 0;
                                        if !((*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            != VAR_LIST as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            || {
                                                val_list = (*val_di).di_tv.vval.v_list;
                                                tv_list_len(val_list) != 4 as ::core::ffi::c_int
                                            })
                                        {
                                            let sign_li: *const listitem_T =
                                                tv_list_first(val_list);
                                            if !((*sign_li).li_tv.v_type as ::core::ffi::c_uint
                                                != VAR_NUMBER as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                                || {
                                                    sign = (*sign_li).li_tv.vval.v_number;
                                                    sign == 0 as varnumber_T
                                                })
                                            {
                                                let highest_bits_li: *const listitem_T =
                                                    (*sign_li).li_next;
                                                if !((*highest_bits_li).li_tv.v_type
                                                    as ::core::ffi::c_uint
                                                    != VAR_NUMBER as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                    || {
                                                        highest_bits =
                                                            (*highest_bits_li).li_tv.vval.v_number;
                                                        highest_bits < 0 as varnumber_T
                                                    })
                                                {
                                                    let high_bits_li: *const listitem_T =
                                                        (*highest_bits_li).li_next;
                                                    if !((*high_bits_li).li_tv.v_type
                                                        as ::core::ffi::c_uint
                                                        != VAR_NUMBER as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                        || {
                                                            high_bits =
                                                                (*high_bits_li).li_tv.vval.v_number;
                                                            high_bits < 0 as varnumber_T
                                                        })
                                                    {
                                                        let low_bits_li: *const listitem_T =
                                                            tv_list_last(val_list);
                                                        if !((*low_bits_li).li_tv.v_type
                                                            as ::core::ffi::c_uint
                                                            != VAR_NUMBER as ::core::ffi::c_int
                                                                as ::core::ffi::c_uint
                                                            || {
                                                                low_bits = (*low_bits_li)
                                                                    .li_tv
                                                                    .vval
                                                                    .v_number;
                                                                low_bits < 0 as varnumber_T
                                                            })
                                                        {
                                                            let number: uint64_t = (highest_bits
                                                                as uint64_t)
                                                                << 62 as ::core::ffi::c_int
                                                                | (high_bits as uint64_t)
                                                                    << 31 as ::core::ffi::c_int
                                                                | low_bits as uint64_t;
                                                            if sign > 0 as varnumber_T {
                                                                lua_pushnumber(
                                                                    lstate,
                                                                    number as lua_Number,
                                                                );
                                                            } else {
                                                                lua_pushnumber(
                                                                    lstate,
                                                                    number.wrapping_neg()
                                                                        as lua_Number,
                                                                );
                                                            }
                                                            break '_typval_encode_stop_converting_one_item;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    3 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_FLOAT as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            lua_pushnumber(lstate, (*val_di).di_tv.vval.v_float);
                                            break '_typval_encode_stop_converting_one_item;
                                        }
                                    }
                                    4 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                            let mut len: size_t = 0;
                                            let mut buf: *mut ::core::ffi::c_char =
                                                ::core::ptr::null_mut::<::core::ffi::c_char>();
                                            if encode_vim_list_to_buf(
                                                (*val_di).di_tv.vval.v_list,
                                                &raw mut len,
                                                &raw mut buf,
                                            ) {
                                                lua_pushlstring(lstate, buf, len);
                                                xfree(buf as *mut ::core::ffi::c_void);
                                                break '_typval_encode_stop_converting_one_item;
                                            }
                                        }
                                    }
                                    5 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                            let saved_copyID_0: ::core::ffi::c_int =
                                                tv_list_copyid((*val_di).di_tv.vval.v_list);
                                            let te_csr_ret_0: ::core::ffi::c_int =
                                                _typval_encode_lua_check_self_reference(
                                                    lstate,
                                                    (*val_di).di_tv.vval.v_list
                                                        as *mut ::core::ffi::c_void,
                                                    &raw mut (*(*val_di).di_tv.vval.v_list)
                                                        .lv_copyID,
                                                    mpstack,
                                                    copyID,
                                                    kMPConvList,
                                                    objname,
                                                );
                                            if te_csr_ret_0 != NOTDONE {
                                                return te_csr_ret_0;
                                            }
                                            if lua_checkstack(
                                                lstate,
                                                lua_gettop(lstate) + 3 as ::core::ffi::c_int,
                                            ) == 0
                                            {
                                                semsg(
                                                    gettext(
                                                        b"E5102: Lua failed to grow stack to %i\0"
                                                            .as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    ),
                                                    lua_gettop(lstate) + 3 as ::core::ffi::c_int,
                                                );
                                                return false_0;
                                            }
                                            lua_createtable(
                                                lstate,
                                                tv_list_len((*val_di).di_tv.vval.v_list),
                                                0 as ::core::ffi::c_int,
                                            );
                                            lua_pushnumber(
                                                lstate,
                                                1 as ::core::ffi::c_int as lua_Number,
                                            );
                                            '_c2rust_label_0: {
                                                if saved_copyID_0 != copyID
                                                    && saved_copyID_0
                                                        != copyID - 1 as ::core::ffi::c_int
                                                {
                                                } else {
                                                    __assert_fail(
                                                        b"saved_copyID != copyID && saved_copyID != copyID - 1\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        b"src/nvim/lua/converter.rs\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        532 as ::core::ffi::c_uint,
                                                        b"int _typval_encode_lua_convert_one_value(lua_State *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    );
                                                }
                                            };
                                            if (*mpstack).size == (*mpstack).capacity {
                                                (*mpstack).capacity = if (*mpstack).capacity
                                                    << 1 as ::core::ffi::c_int
                                                    > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        ) {
                                                    (*mpstack).capacity << 1 as ::core::ffi::c_int
                                                } else {
                                                    ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as size_t,
                                                        )
                                                };
                                                (*mpstack).items = (if (*mpstack).capacity
                                                    == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        ) {
                                                    if (*mpstack).items
                                                        == &raw mut (*mpstack).init_array
                                                            as *mut MPConvStackVal
                                                    {
                                                        (*mpstack).items as *mut ::core::ffi::c_void
                                                    } else {
                                                        _memcpy_free(
                                                            &raw mut (*mpstack).init_array
                                                                as *mut MPConvStackVal
                                                                as *mut ::core::ffi::c_void,
                                                            (*mpstack).items
                                                                as *mut ::core::ffi::c_void,
                                                            (*mpstack).size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                } else {
                                                    if (*mpstack).items
                                                        == &raw mut (*mpstack).init_array
                                                            as *mut MPConvStackVal
                                                    {
                                                        memcpy(
                                                            xmalloc(
                                                                (*mpstack).capacity.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        MPConvStackVal,
                                                                    >(
                                                                    ),
                                                                ),
                                                            ),
                                                            (*mpstack).items
                                                                as *const ::core::ffi::c_void,
                                                            (*mpstack).size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    } else {
                                                        xrealloc(
                                                            (*mpstack).items
                                                                as *mut ::core::ffi::c_void,
                                                            (*mpstack).capacity.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                })
                                                    as *mut MPConvStackVal;
                                            } else {
                                            };
                                            let c2rust_fresh11 = (*mpstack).size;
                                            (*mpstack).size = (*mpstack).size.wrapping_add(1);
                                            *(*mpstack).items.offset(c2rust_fresh11 as isize) =
                                                MPConvStackVal {
                                                    type_0: kMPConvList,
                                                    tv: tv,
                                                    saved_copyID: saved_copyID_0,
                                                    data: C2Rust_Unnamed_1 {
                                                        l: C2Rust_Unnamed_4 {
                                                            list: (*val_di).di_tv.vval.v_list,
                                                            li: tv_list_first(
                                                                (*val_di).di_tv.vval.v_list,
                                                            ),
                                                        },
                                                    },
                                                };
                                            break '_typval_encode_stop_converting_one_item;
                                        }
                                    }
                                    6 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                            let val_list_0: *mut list_T =
                                                (*val_di).di_tv.vval.v_list;
                                            if val_list_0.is_null()
                                                || tv_list_len(val_list_0)
                                                    == 0 as ::core::ffi::c_int
                                            {
                                                if typval_conv_special.get() {
                                                    nlua_create_typed_table(
                                                        lstate,
                                                        0 as size_t,
                                                        0 as size_t,
                                                        kObjectTypeDict,
                                                    );
                                                } else {
                                                    lua_createtable(
                                                        lstate,
                                                        0 as ::core::ffi::c_int,
                                                        0 as ::core::ffi::c_int,
                                                    );
                                                    nlua_pushref(
                                                        lstate,
                                                        (*nlua_global_refs.get()).empty_dict_ref,
                                                    );
                                                    lua_setmetatable(
                                                        lstate,
                                                        -2 as ::core::ffi::c_int,
                                                    );
                                                }
                                                break '_typval_encode_stop_converting_one_item;
                                            } else {
                                                let l_: *const list_T = val_list_0;
                                                's_755: {
                                                    if !l_.is_null() {
                                                        let mut li: *const listitem_T =
                                                            (*l_).lv_first;
                                                        loop {
                                                            if li.is_null() {
                                                                break 's_755;
                                                            }
                                                            if (*li).li_tv.v_type
                                                                as ::core::ffi::c_uint
                                                                != VAR_LIST as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                                || tv_list_len(
                                                                    (*li).li_tv.vval.v_list,
                                                                ) != 2 as ::core::ffi::c_int
                                                            {
                                                                break 's_887;
                                                            }
                                                            li = (*li).li_next;
                                                        }
                                                    }
                                                }
                                                let saved_copyID_1: ::core::ffi::c_int =
                                                    tv_list_copyid((*val_di).di_tv.vval.v_list);
                                                let te_csr_ret_1: ::core::ffi::c_int =
                                                    _typval_encode_lua_check_self_reference(
                                                        lstate,
                                                        val_list_0 as *mut ::core::ffi::c_void,
                                                        &raw mut (*val_list_0).lv_copyID,
                                                        mpstack,
                                                        copyID,
                                                        kMPConvPairs,
                                                        objname,
                                                    );
                                                if te_csr_ret_1 != NOTDONE {
                                                    return te_csr_ret_1;
                                                }
                                                if lua_checkstack(
                                                    lstate,
                                                    lua_gettop(lstate) + 3 as ::core::ffi::c_int,
                                                ) == 0
                                                {
                                                    semsg(
                                                        gettext(
                                                            b"E5102: Lua failed to grow stack to %i\0".as_ptr()
                                                                as *const ::core::ffi::c_char,
                                                        ),
                                                        lua_gettop(lstate) + 3 as ::core::ffi::c_int,
                                                    );
                                                    return false_0;
                                                }
                                                lua_createtable(
                                                    lstate,
                                                    0 as ::core::ffi::c_int,
                                                    tv_list_len(val_list_0),
                                                );
                                                '_c2rust_label_1: {
                                                    if saved_copyID_1 != copyID
                                                        && saved_copyID_1
                                                            != copyID - 1 as ::core::ffi::c_int
                                                    {
                                                    } else {
                                                        __assert_fail(
                                                            b"saved_copyID != copyID && saved_copyID != copyID - 1\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            b"src/nvim/lua/converter.rs\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            566 as ::core::ffi::c_uint,
                                                            b"int _typval_encode_lua_convert_one_value(lua_State *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                        );
                                                    }
                                                };
                                                if (*mpstack).size == (*mpstack).capacity {
                                                    (*mpstack).capacity =
                                                        if (*mpstack).capacity
                                                            << 1 as ::core::ffi::c_int
                                                            > ::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [MPConvStackVal; 8],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        MPConvStackVal,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as usize,
                                                            )
                                                        {
                                                            (*mpstack).capacity
                                                                << 1 as ::core::ffi::c_int
                                                        } else {
                                                            ::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [MPConvStackVal; 8],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        MPConvStackVal,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as size_t,
                                                            )
                                                        };
                                                    (*mpstack).items =
                                                        (if (*mpstack).capacity
                                                            == ::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [MPConvStackVal; 8],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        MPConvStackVal,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as usize,
                                                            )
                                                        {
                                                            if (*mpstack).items
                                                                == &raw mut (*mpstack).init_array
                                                                    as *mut MPConvStackVal
                                                            {
                                                                (*mpstack).items
                                                                    as *mut ::core::ffi::c_void
                                                            } else {
                                                                _memcpy_free(
                                                                    &raw mut (*mpstack).init_array
                                                                        as *mut MPConvStackVal
                                                                        as *mut ::core::ffi::c_void,
                                                                    (*mpstack).items
                                                                        as *mut ::core::ffi::c_void,
                                                                    (*mpstack).size.wrapping_mul(
                                                                        ::core::mem::size_of::<
                                                                            MPConvStackVal,
                                                                        >(
                                                                        ),
                                                                    ),
                                                                )
                                                            }
                                                        } else {
                                                            if (*mpstack).items
                                                                == &raw mut (*mpstack).init_array
                                                                    as *mut MPConvStackVal
                                                            {
                                                                memcpy(
                                                                xmalloc(
                                                                    (*mpstack)
                                                                        .capacity
                                                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                                                ),
                                                                (*mpstack).items as *const ::core::ffi::c_void,
                                                                (*mpstack)
                                                                    .size
                                                                    .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                                            )
                                                            } else {
                                                                xrealloc(
                                                                    (*mpstack).items
                                                                        as *mut ::core::ffi::c_void,
                                                                    (*mpstack)
                                                                        .capacity
                                                                        .wrapping_mul(
                                                                            ::core::mem::size_of::<
                                                                                MPConvStackVal,
                                                                            >(
                                                                            ),
                                                                        ),
                                                                )
                                                            }
                                                        })
                                                            as *mut MPConvStackVal;
                                                } else {
                                                };
                                                let c2rust_fresh12 = (*mpstack).size;
                                                (*mpstack).size = (*mpstack).size.wrapping_add(1);
                                                *(*mpstack).items.offset(c2rust_fresh12 as isize) =
                                                    MPConvStackVal {
                                                        type_0: kMPConvPairs,
                                                        tv: tv,
                                                        saved_copyID: saved_copyID_1,
                                                        data: C2Rust_Unnamed_1 {
                                                            l: C2Rust_Unnamed_4 {
                                                                list: val_list_0,
                                                                li: tv_list_first(val_list_0),
                                                            },
                                                        },
                                                    };
                                                break '_typval_encode_stop_converting_one_item;
                                            }
                                        }
                                    }
                                    7 => {
                                        let mut val_list_1: *const list_T =
                                            ::core::ptr::null::<list_T>();
                                        let mut type_0: varnumber_T = 0;
                                        if !((*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            != VAR_LIST as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            || {
                                                val_list_1 = (*val_di).di_tv.vval.v_list;
                                                tv_list_len(val_list_1) != 2 as ::core::ffi::c_int
                                            }
                                            || (*tv_list_first(val_list_1)).li_tv.v_type
                                                as ::core::ffi::c_uint
                                                != VAR_NUMBER as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || {
                                                type_0 = (*tv_list_first(val_list_1))
                                                    .li_tv
                                                    .vval
                                                    .v_number;
                                                type_0 > INT8_MAX as varnumber_T
                                            }
                                            || type_0 < INT8_MIN as varnumber_T
                                            || (*tv_list_last(val_list_1)).li_tv.v_type
                                                as ::core::ffi::c_uint
                                                != VAR_LIST as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint)
                                        {
                                            let mut len_0: size_t = 0;
                                            let mut buf_0: *mut ::core::ffi::c_char =
                                                ::core::ptr::null_mut::<::core::ffi::c_char>();
                                            if encode_vim_list_to_buf(
                                                (*tv_list_last(val_list_1)).li_tv.vval.v_list,
                                                &raw mut len_0,
                                                &raw mut buf_0,
                                            ) {
                                                if typval_conv_special.get() {
                                                    lua_pushnil(lstate);
                                                } else {
                                                    nlua_pushref(
                                                        lstate,
                                                        (*nlua_global_refs.get()).nil_ref,
                                                    );
                                                }
                                                xfree(buf_0 as *mut ::core::ffi::c_void);
                                                break '_typval_encode_stop_converting_one_item;
                                            }
                                        }
                                    }
                                    _ => {
                                        break '_typval_encode_stop_converting_one_item;
                                    }
                                }
                            }
                        }
                    }
                    let saved_copyID_2: ::core::ffi::c_int = (*(*tv).vval.v_dict).dv_copyID;
                    let te_csr_ret_2: ::core::ffi::c_int = _typval_encode_lua_check_self_reference(
                        lstate,
                        (*tv).vval.v_dict as *mut ::core::ffi::c_void,
                        &raw mut (*(*tv).vval.v_dict).dv_copyID,
                        mpstack,
                        copyID,
                        kMPConvDict,
                        objname,
                    );
                    if te_csr_ret_2 != NOTDONE {
                        return te_csr_ret_2;
                    }
                    if lua_checkstack(lstate, lua_gettop(lstate) + 3 as ::core::ffi::c_int) == 0 {
                        semsg(
                            gettext(b"E5102: Lua failed to grow stack to %i\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            lua_gettop(lstate) + 3 as ::core::ffi::c_int,
                        );
                        return false_0;
                    }
                    lua_createtable(
                        lstate,
                        0 as ::core::ffi::c_int,
                        (*(*tv).vval.v_dict).dv_hashtab.ht_used as ::core::ffi::c_int,
                    );
                    '_c2rust_label_2: {
                        if saved_copyID_2 != copyID {
                        } else {
                            __assert_fail(
                                b"saved_copyID != copyID\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/lua/converter.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                614 as ::core::ffi::c_uint,
                                b"int _typval_encode_lua_convert_one_value(lua_State *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    if (*mpstack).size == (*mpstack).capacity {
                        (*mpstack).capacity = if (*mpstack).capacity << 1 as ::core::ffi::c_int
                            > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            (*mpstack).capacity << 1 as ::core::ffi::c_int
                        } else {
                            ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                )
                        };
                        (*mpstack).items = (if (*mpstack).capacity
                            == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            if (*mpstack).items
                                == &raw mut (*mpstack).init_array as *mut MPConvStackVal
                            {
                                (*mpstack).items as *mut ::core::ffi::c_void
                            } else {
                                _memcpy_free(
                                    &raw mut (*mpstack).init_array as *mut MPConvStackVal
                                        as *mut ::core::ffi::c_void,
                                    (*mpstack).items as *mut ::core::ffi::c_void,
                                    (*mpstack)
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                )
                            }
                        } else {
                            if (*mpstack).items
                                == &raw mut (*mpstack).init_array as *mut MPConvStackVal
                            {
                                memcpy(
                                    xmalloc(
                                        (*mpstack)
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                    ),
                                    (*mpstack).items as *const ::core::ffi::c_void,
                                    (*mpstack)
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                )
                            } else {
                                xrealloc(
                                    (*mpstack).items as *mut ::core::ffi::c_void,
                                    (*mpstack)
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                )
                            }
                        }) as *mut MPConvStackVal;
                    } else {
                    };
                    let c2rust_fresh13 = (*mpstack).size;
                    (*mpstack).size = (*mpstack).size.wrapping_add(1);
                    *(*mpstack).items.offset(c2rust_fresh13 as isize) = MPConvStackVal {
                        type_0: kMPConvDict,
                        tv: tv,
                        saved_copyID: saved_copyID_2,
                        data: C2Rust_Unnamed_1 {
                            d: C2Rust_Unnamed_5 {
                                dict: (*tv).vval.v_dict,
                                dictp: &raw mut (*tv).vval.v_dict,
                                hi: (*(*tv).vval.v_dict).dv_hashtab.ht_array,
                                todo: (*(*tv).vval.v_dict).dv_hashtab.ht_used,
                            },
                        },
                    };
                }
            }
            0 => {
                internal_error(b"_typval_encode_lua_convert_one_value()\0".as_ptr()
                    as *const ::core::ffi::c_char);
                return FAIL;
            }
            _ => {}
        }
    }
    return OK;
}
unsafe extern "C" fn encode_vim_to_lua(
    lstate: *mut lua_State,
    top_tv: *mut typval_T,
    objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let copyID: ::core::ffi::c_int = get_copyID();
    let mut mpstack: MPConvStack = MPConvStack {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<MPConvStackVal>(),
        init_array: [MPConvStackVal {
            type_0: kMPConvDict,
            tv: ::core::ptr::null_mut::<typval_T>(),
            saved_copyID: 0,
            data: C2Rust_Unnamed_1 {
                d: C2Rust_Unnamed_5 {
                    dict: ::core::ptr::null_mut::<dict_T>(),
                    dictp: ::core::ptr::null_mut::<*mut dict_T>(),
                    hi: ::core::ptr::null_mut::<hashitem_T>(),
                    todo: 0,
                },
            },
        }; 8],
    };
    mpstack.capacity = ::core::mem::size_of::<[MPConvStackVal; 8]>()
        .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
        .wrapping_div(
            (::core::mem::size_of::<[MPConvStackVal; 8]>()
                .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    mpstack.size = 0 as size_t;
    mpstack.items = &raw mut mpstack.init_array as *mut MPConvStackVal;
    '_encode_vim_to__error_ret: {
        if _typval_encode_lua_convert_one_value(
            lstate,
            &raw mut mpstack,
            ::core::ptr::null_mut::<MPConvStackVal>(),
            top_tv,
            copyID,
            objname,
        ) != FAIL
        {
            while mpstack.size != 0 {
                let mut cur_mpsv: *mut MPConvStackVal = mpstack.items.offset(
                    mpstack
                        .size
                        .wrapping_sub(0 as size_t)
                        .wrapping_sub(1 as size_t) as isize,
                );
                let mut tv: *mut typval_T = ::core::ptr::null_mut::<typval_T>();
                match (*cur_mpsv).type_0 as ::core::ffi::c_uint {
                    0 => {
                        if (*cur_mpsv).data.d.todo == 0 {
                            mpstack.size = mpstack.size.wrapping_sub(1);
                            (*(*cur_mpsv).data.d.dict).dv_copyID = (*cur_mpsv).saved_copyID;
                            lua_rawset(lstate, -3 as ::core::ffi::c_int);
                            continue;
                        } else {
                            if (*cur_mpsv).data.d.todo
                                != (*(*cur_mpsv).data.d.dict).dv_hashtab.ht_used
                            {
                                lua_rawset(lstate, -3 as ::core::ffi::c_int);
                            }
                            while (*(*cur_mpsv).data.d.hi).hi_key.is_null()
                                || (*(*cur_mpsv).data.d.hi).hi_key
                                    == &raw const hash_removed as *mut ::core::ffi::c_char
                            {
                                (*cur_mpsv).data.d.hi = (*cur_mpsv).data.d.hi.offset(1);
                            }
                            let di: *mut dictitem_T = (*(*cur_mpsv).data.d.hi)
                                .hi_key
                                .offset(-(17 as ::core::ffi::c_ulong as isize))
                                as *mut dictitem_T;
                            (*cur_mpsv).data.d.todo = (*cur_mpsv).data.d.todo.wrapping_sub(1);
                            (*cur_mpsv).data.d.hi = (*cur_mpsv).data.d.hi.offset(1);
                            lua_pushlstring(
                                lstate,
                                (&raw mut (*di).di_key as *mut ::core::ffi::c_char)
                                    .offset(0 as ::core::ffi::c_int as isize),
                                strlen(
                                    (&raw mut (*di).di_key as *mut ::core::ffi::c_char)
                                        .offset(0 as ::core::ffi::c_int as isize),
                                ),
                            );
                            tv = &raw mut (*di).di_tv;
                        }
                    }
                    1 => {
                        if (*cur_mpsv).data.l.li.is_null() {
                            mpstack.size = mpstack.size.wrapping_sub(1);
                            tv_list_set_copyid((*cur_mpsv).data.l.list, (*cur_mpsv).saved_copyID);
                            lua_rawset(lstate, -3 as ::core::ffi::c_int);
                            continue;
                        } else {
                            if (*cur_mpsv).data.l.li != tv_list_first((*cur_mpsv).data.l.list) {
                                let mut idx: lua_Number =
                                    lua_tonumber(lstate, -2 as ::core::ffi::c_int);
                                lua_rawset(lstate, -3 as ::core::ffi::c_int);
                                lua_pushnumber(lstate, idx + 1 as ::core::ffi::c_int as lua_Number);
                            }
                            tv = &raw mut (*(*cur_mpsv).data.l.li).li_tv;
                            (*cur_mpsv).data.l.li = (*(*cur_mpsv).data.l.li).li_next;
                        }
                    }
                    2 => {
                        if (*cur_mpsv).data.l.li.is_null() {
                            mpstack.size = mpstack.size.wrapping_sub(1);
                            tv_list_set_copyid((*cur_mpsv).data.l.list, (*cur_mpsv).saved_copyID);
                            lua_rawset(lstate, -3 as ::core::ffi::c_int);
                            continue;
                        } else {
                            if (*cur_mpsv).data.l.li != tv_list_first((*cur_mpsv).data.l.list) {
                                lua_rawset(lstate, -3 as ::core::ffi::c_int);
                            }
                            let kv_pair: *const list_T = (*(*cur_mpsv).data.l.li).li_tv.vval.v_list;
                            if _typval_encode_lua_convert_one_value(
                                lstate,
                                &raw mut mpstack,
                                cur_mpsv,
                                &raw mut (*(tv_list_first
                                    as unsafe extern "C" fn(*const list_T) -> *mut listitem_T)(
                                    kv_pair,
                                ))
                                .li_tv,
                                copyID,
                                objname,
                            ) == FAIL
                            {
                                break '_encode_vim_to__error_ret;
                            }
                            tv = &raw mut (*(tv_list_last
                                as unsafe extern "C" fn(*const list_T) -> *mut listitem_T)(
                                kv_pair,
                            ))
                            .li_tv;
                            (*cur_mpsv).data.l.li = (*(*cur_mpsv).data.l.li).li_next;
                        }
                    }
                    3 => {
                        let pt: *mut partial_T = (*cur_mpsv).data.p.pt;
                        tv = (*cur_mpsv).tv;
                        match (*cur_mpsv).data.p.stage as ::core::ffi::c_uint {
                            0 => {
                                (*cur_mpsv).data.p.stage = kMPConvPartialSelf;
                                if !pt.is_null() && (*pt).pt_argc > 0 as ::core::ffi::c_int {
                                    if lua_checkstack(
                                        lstate,
                                        lua_gettop(lstate) + 3 as ::core::ffi::c_int,
                                    ) == 0
                                    {
                                        semsg(
                                            gettext(
                                                b"E5102: Lua failed to grow stack to %i\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            lua_gettop(lstate) + 3 as ::core::ffi::c_int,
                                        );
                                        return false_0;
                                    }
                                    lua_createtable(lstate, (*pt).pt_argc, 0 as ::core::ffi::c_int);
                                    lua_pushnumber(lstate, 1 as ::core::ffi::c_int as lua_Number);
                                    if mpstack.size == mpstack.capacity {
                                        mpstack.capacity = if mpstack.capacity
                                            << 1 as ::core::ffi::c_int
                                            > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<MPConvStackVal>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as usize,
                                                ) {
                                            mpstack.capacity << 1 as ::core::ffi::c_int
                                        } else {
                                            ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<MPConvStackVal>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as size_t,
                                                )
                                        };
                                        mpstack.items = (if mpstack.capacity
                                            == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<MPConvStackVal>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as usize,
                                                ) {
                                            if mpstack.items
                                                == &raw mut mpstack.init_array
                                                    as *mut MPConvStackVal
                                            {
                                                mpstack.items as *mut ::core::ffi::c_void
                                            } else {
                                                _memcpy_free(
                                                    &raw mut mpstack.init_array
                                                        as *mut MPConvStackVal
                                                        as *mut ::core::ffi::c_void,
                                                    mpstack.items as *mut ::core::ffi::c_void,
                                                    mpstack.size.wrapping_mul(
                                                        ::core::mem::size_of::<MPConvStackVal>(),
                                                    ),
                                                )
                                            }
                                        } else {
                                            if mpstack.items
                                                == &raw mut mpstack.init_array
                                                    as *mut MPConvStackVal
                                            {
                                                memcpy(
                                                    xmalloc(mpstack.capacity.wrapping_mul(
                                                        ::core::mem::size_of::<MPConvStackVal>(),
                                                    )),
                                                    mpstack.items as *const ::core::ffi::c_void,
                                                    mpstack.size.wrapping_mul(
                                                        ::core::mem::size_of::<MPConvStackVal>(),
                                                    ),
                                                )
                                            } else {
                                                xrealloc(
                                                    mpstack.items as *mut ::core::ffi::c_void,
                                                    mpstack.capacity.wrapping_mul(
                                                        ::core::mem::size_of::<MPConvStackVal>(),
                                                    ),
                                                )
                                            }
                                        })
                                            as *mut MPConvStackVal;
                                    } else {
                                    };
                                    let c2rust_fresh6 = mpstack.size;
                                    mpstack.size = mpstack.size.wrapping_add(1);
                                    *mpstack.items.offset(c2rust_fresh6 as isize) =
                                        MPConvStackVal {
                                            type_0: kMPConvPartialList,
                                            tv: ::core::ptr::null_mut::<typval_T>(),
                                            saved_copyID: copyID - 1 as ::core::ffi::c_int,
                                            data: C2Rust_Unnamed_1 {
                                                a: C2Rust_Unnamed_2 {
                                                    arg: (*pt).pt_argv,
                                                    argv: (*pt).pt_argv,
                                                    todo: (*pt).pt_argc as size_t,
                                                },
                                            },
                                        };
                                }
                                continue;
                            }
                            1 => {
                                (*cur_mpsv).data.p.stage = kMPConvPartialEnd;
                                let dict: *mut dict_T = if pt.is_null() {
                                    ::core::ptr::null_mut::<dict_T>()
                                } else {
                                    (*pt).pt_dict
                                };
                                if dict.is_null() {
                                    continue;
                                }
                                if (*dict).dv_hashtab.ht_used == 0 as size_t {
                                    if typval_conv_special.get() {
                                        nlua_create_typed_table(
                                            lstate,
                                            0 as size_t,
                                            0 as size_t,
                                            kObjectTypeDict,
                                        );
                                    } else {
                                        lua_createtable(
                                            lstate,
                                            0 as ::core::ffi::c_int,
                                            0 as ::core::ffi::c_int,
                                        );
                                        nlua_pushref(
                                            lstate,
                                            (*nlua_global_refs.get()).empty_dict_ref,
                                        );
                                        lua_setmetatable(lstate, -2 as ::core::ffi::c_int);
                                    }
                                    continue;
                                } else {
                                    let saved_copyID: ::core::ffi::c_int = (*dict).dv_copyID;
                                    let te_csr_ret: ::core::ffi::c_int =
                                        _typval_encode_lua_check_self_reference(
                                            lstate,
                                            dict as *mut ::core::ffi::c_void,
                                            &raw mut (*dict).dv_copyID,
                                            &raw mut mpstack,
                                            copyID,
                                            kMPConvDict,
                                            objname,
                                        );
                                    if te_csr_ret != NOTDONE {
                                        if te_csr_ret == FAIL {
                                            break '_encode_vim_to__error_ret;
                                        } else {
                                            continue;
                                        }
                                    } else {
                                        if lua_checkstack(
                                            lstate,
                                            lua_gettop(lstate) + 3 as ::core::ffi::c_int,
                                        ) == 0
                                        {
                                            semsg(
                                                gettext(
                                                    b"E5102: Lua failed to grow stack to %i\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                lua_gettop(lstate) + 3 as ::core::ffi::c_int,
                                            );
                                            return false_0;
                                        }
                                        lua_createtable(
                                            lstate,
                                            0 as ::core::ffi::c_int,
                                            (*dict).dv_hashtab.ht_used as ::core::ffi::c_int,
                                        );
                                        '_c2rust_label: {
                                            if saved_copyID != copyID
                                                && saved_copyID != copyID - 1 as ::core::ffi::c_int
                                            {
                                            } else {
                                                __assert_fail(
                                                    b"saved_copyID != copyID && saved_copyID != copyID - 1\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    b"src/nvim/lua/converter.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    789 as ::core::ffi::c_uint,
                                                    b"int encode_vim_to_lua(lua_State *const, typval_T *const, const char *const)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        if mpstack.size == mpstack.capacity {
                                            mpstack.capacity =
                                                if mpstack.capacity << 1 as ::core::ffi::c_int
                                                    > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        )
                                                {
                                                    mpstack.capacity << 1 as ::core::ffi::c_int
                                                } else {
                                                    ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as size_t,
                                                        )
                                                };
                                            mpstack.items =
                                                (if mpstack.capacity
                                                    == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        )
                                                {
                                                    if mpstack.items
                                                        == &raw mut mpstack.init_array
                                                            as *mut MPConvStackVal
                                                    {
                                                        mpstack.items as *mut ::core::ffi::c_void
                                                    } else {
                                                        _memcpy_free(
                                                            &raw mut mpstack.init_array
                                                                as *mut MPConvStackVal
                                                                as *mut ::core::ffi::c_void,
                                                            mpstack.items
                                                                as *mut ::core::ffi::c_void,
                                                            mpstack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                } else {
                                                    if mpstack.items
                                                        == &raw mut mpstack.init_array
                                                            as *mut MPConvStackVal
                                                    {
                                                        memcpy(
                                                            xmalloc(mpstack.capacity.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            )),
                                                            mpstack.items
                                                                as *const ::core::ffi::c_void,
                                                            mpstack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    } else {
                                                        xrealloc(
                                                            mpstack.items
                                                                as *mut ::core::ffi::c_void,
                                                            mpstack.capacity.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                })
                                                    as *mut MPConvStackVal;
                                        } else {
                                        };
                                        let c2rust_fresh7 = mpstack.size;
                                        mpstack.size = mpstack.size.wrapping_add(1);
                                        *mpstack.items.offset(c2rust_fresh7 as isize) =
                                            MPConvStackVal {
                                                type_0: kMPConvDict,
                                                tv: ::core::ptr::null_mut::<typval_T>(),
                                                saved_copyID: saved_copyID,
                                                data: C2Rust_Unnamed_1 {
                                                    d: C2Rust_Unnamed_5 {
                                                        dict: dict,
                                                        dictp: &raw mut (*pt).pt_dict,
                                                        hi: (*dict).dv_hashtab.ht_array,
                                                        todo: (*dict).dv_hashtab.ht_used,
                                                    },
                                                },
                                            };
                                        continue;
                                    }
                                }
                            }
                            2 => {
                                mpstack.size = mpstack.size.wrapping_sub(1);
                                continue;
                            }
                            _ => {
                                continue;
                            }
                        }
                    }
                    4 => {
                        if (*cur_mpsv).data.a.todo == 0 {
                            mpstack.size = mpstack.size.wrapping_sub(1);
                            lua_rawset(lstate, -3 as ::core::ffi::c_int);
                            continue;
                        } else {
                            if (*cur_mpsv).data.a.argv != (*cur_mpsv).data.a.arg {
                                let mut idx_0: lua_Number =
                                    lua_tonumber(lstate, -2 as ::core::ffi::c_int);
                                lua_rawset(lstate, -3 as ::core::ffi::c_int);
                                lua_pushnumber(
                                    lstate,
                                    idx_0 + 1 as ::core::ffi::c_int as lua_Number,
                                );
                            }
                            let c2rust_fresh8 = (*cur_mpsv).data.a.arg;
                            (*cur_mpsv).data.a.arg = (*cur_mpsv).data.a.arg.offset(1);
                            tv = c2rust_fresh8;
                            (*cur_mpsv).data.a.todo = (*cur_mpsv).data.a.todo.wrapping_sub(1);
                        }
                    }
                    _ => {}
                }
                '_c2rust_label_0: {
                    if !tv.is_null() {
                    } else {
                        __assert_fail(
                            b"tv != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/lua/converter.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            829 as ::core::ffi::c_uint,
                            b"int encode_vim_to_lua(lua_State *const, typval_T *const, const char *const)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                if _typval_encode_lua_convert_one_value(
                    lstate,
                    &raw mut mpstack,
                    cur_mpsv,
                    tv,
                    copyID,
                    objname,
                ) == FAIL
                {
                    break '_encode_vim_to__error_ret;
                }
            }
            if mpstack.items != &raw mut mpstack.init_array as *mut MPConvStackVal {
                let mut ptr_: *mut *mut ::core::ffi::c_void =
                    &raw mut mpstack.items as *mut *mut ::core::ffi::c_void;
                xfree(*ptr_);
                *ptr_ = NULL_0;
                let _ = *ptr_;
            }
            return OK;
        }
    }
    if mpstack.items != &raw mut mpstack.init_array as *mut MPConvStackVal {
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut mpstack.items as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_0;
        let _ = *ptr__0;
    }
    return FAIL;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
