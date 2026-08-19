//! `nlua_pop_typval()`: a Lua value as a Vimscript one.
//!
//! The Lua->typval direction, and the mirror of [`super::push`].  One
//! explicit stack of [`TVPopStackItem`]s rather than recursion, because a
//! Lua table may nest arbitrarily deep and the conversion has to be able to
//! refuse (`E5100`) rather than overflow.  Tables are classified by
//! [`nlua_traverse_table`] first, so a table's *shape* -- list, dictionary,
//! empty-dict, or a `{_TYPE, _VAL}` special -- is decided once.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{CStr, c_int};

use super::{VARNUMBER_MAX, VARNUMBER_MIN, nlua_traverse_table};
use crate::eval::decode::{decode_create_map_special_dict, decode_string};
use crate::eval::typval::{
    TV_INITIAL_VALUE, tv_clear, tv_copy, tv_dict_add, tv_dict_alloc, tv_dict_find,
    tv_dict_item_alloc_len, tv_list_alloc, tv_list_append_list, tv_list_append_owned_tv,
    tv_list_last, tv_list_len, tv_list_ref,
};
use crate::eval::typval_encode::InlineStack;
use crate::eval::userfunc::register_luafunc;
use crate::lua::executor::{nlua_pushref, nlua_ref_global};
use crate::lua::ffi::{
    LUA_NOREF, LUA_TBOOLEAN, LUA_TFUNCTION, LUA_TNIL, LUA_TNUMBER, LUA_TSTRING, LUA_TTABLE,
    LUA_TUSERDATA, lua_checkstack, lua_getmetatable, lua_gettop, lua_next, lua_pop, lua_pushnil,
    lua_rawequal, lua_rawgeti, lua_toboolean, lua_tolstring, lua_tonumber, lua_type,
};
use crate::main::nlua_global_refs;
use crate::memory::xstrdup;
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::types::{
    FAIL, LuaRef, VAR_BOOL, VAR_DICT, VAR_FLOAT, VAR_FUNC, VAR_LIST, VAR_NUMBER, VAR_SPECIAL,
    VAR_UNLOCKED, kBoolVarFalse, kBoolVarTrue, kObjectTypeArray, kObjectTypeDict, kObjectTypeFloat,
    kObjectTypeNil, kSpecialVarNull, lua_Number, lua_State, ptrdiff_t, size_t, typval_T,
    typval_vval_union, varnumber_T,
};
use ::libc::abort;

/// Refused when the Lua stack will not grow far enough for the next value.
const E1502_GROW_STACK: &CStr = c"E1502: Lua failed to grow stack to %i";
/// Refused for a table that is neither a list nor a dictionary.
const E5100_MIXED_KEYS: &CStr = c"E5100: Cannot convert given Lua table: table should contain \
                                 either only integer keys or only string keys";
/// Refused for a Lua value with no Vimscript image at all.
const E5101_BAD_TYPE: &CStr = c"E5101: Cannot convert given Lua type";

/// One suspended container in the walk.
#[derive(Copy, Clone)]
pub struct TVPopStackItem {
    /// Where the conversion's result is to be stored.
    pub tv: *mut typval_T,
    /// The list's length, when `tv` is a list.
    pub list_len: size_t,
    /// Whether `tv` is a container: a frame that is suspended rather than
    /// about to be filled in.
    pub container: bool,
    /// Whether `tv` is the `_VAL` half of the special dictionary standing for
    /// a map — in which case it is a *list* of key-value pairs.
    pub special: bool,
    /// Where the container sits on the Lua stack, which is how a
    /// self-referencing structure is detected.
    pub idx: c_int,
}

impl TVPopStackItem {
    /// A frame about to be filled in, not a suspended container.
    const fn leaf(tv: *mut typval_T) -> Self {
        Self {
            tv,
            list_len: 0,
            container: false,
            special: false,
            idx: 0,
        }
    }
}

/// Frames held without allocating: upstream's `kvec_withinit_t(…, 2)`.
type TVPopStack = InlineStack<TVPopStackItem, 2>;

/// Convert the Lua value on top of the stack into `ret_tv`, popping exactly
/// one value.
///
/// `false` when it will not convert, with the reason already reported and
/// `ret_tv` left as a zero number.
///
/// # Safety
/// `lstate` must be a live Lua state with a value on top, and `ret_tv` a
/// writable typval the caller owns.
pub unsafe fn nlua_pop_typval(lstate: *mut lua_State, ret_tv: *mut typval_T) -> bool {
    unsafe {
        // Make `tv` a fresh, referenced, empty dictionary carrying `ref_`.
        let new_dict = |tv: *mut typval_T, ref_: LuaRef| {
            (*tv).v_type = VAR_DICT;
            (*tv).vval.v_dict = tv_dict_alloc();
            (*(*tv).vval.v_dict).dv_refcount += 1;
            (*(*tv).vval.v_dict).lua_table_ref = ref_;
        };

        let mut ret = true;
        let initial_size = lua_gettop(lstate);
        let mut stack = TVPopStack::new();
        stack.push(TVPopStackItem::leaf(ret_tv));
        while ret && !stack.is_empty() {
            if lua_checkstack(lstate, lua_gettop(lstate) + 3) == 0 {
                semsg_c!(gettext(E1502_GROW_STACK.as_ptr()), lua_gettop(lstate) + 3);
                ret = false;
                break;
            }
            let mut cur = stack.last();
            stack.pop();
            if cur.container {
                if cur.special || (*cur.tv).v_type == VAR_DICT {
                    debug_assert!(
                        (*cur.tv).v_type == if cur.special { VAR_LIST } else { VAR_DICT }
                    );
                    // Skip any non-string key: those are not part of the
                    // dictionary being built.
                    let mut next_key_found = false;
                    while lua_next(lstate, -2) != 0 {
                        if lua_type(lstate, -2) == LUA_TSTRING {
                            next_key_found = true;
                            break;
                        }
                        lua_pop(lstate, 1);
                    }
                    if !next_key_found {
                        lua_pop(lstate, 1);
                        continue;
                    }
                    let mut len: size_t = 0;
                    let s = lua_tolstring(lstate, -2, &raw mut len);
                    if cur.special {
                        // A map special dictionary's `_VAL` is a list of
                        // two-element [key, value] lists.
                        let kv_pair = tv_list_alloc(2);
                        let s_tv = decode_string(s, len, true, false);
                        tv_list_append_owned_tv(kv_pair, s_tv);
                        // The value is not there yet; append a slot to fill.
                        tv_list_append_owned_tv(kv_pair, TV_INITIAL_VALUE);
                        stack.push(cur);
                        tv_list_append_list((*cur.tv).vval.v_list, kv_pair);
                        cur = TVPopStackItem::leaf(&raw mut (*tv_list_last(kv_pair)).li_tv);
                    } else {
                        let di = tv_dict_item_alloc_len(s, len);
                        if tv_dict_add((*cur.tv).vval.v_dict, di) == FAIL {
                            abort();
                        }
                        stack.push(cur);
                        cur = TVPopStackItem::leaf(&raw mut (*di).di_tv);
                    }
                } else {
                    debug_assert!((*cur.tv).v_type == VAR_LIST);
                    if tv_list_len((*cur.tv).vval.v_list) as size_t == cur.list_len {
                        lua_pop(lstate, 1);
                        continue;
                    }
                    lua_rawgeti(lstate, -1, tv_list_len((*cur.tv).vval.v_list) + 1);
                    // Not populated yet; append a list item to fill.
                    tv_list_append_owned_tv((*cur.tv).vval.v_list, TV_INITIAL_VALUE);
                    stack.push(cur);
                    // TODO(ZyX-I): use indexes, the list item *will* be
                    // reallocated here.
                    cur =
                        TVPopStackItem::leaf(&raw mut (*tv_list_last((*cur.tv).vval.v_list)).li_tv);
                }
            }
            debug_assert!(!cur.container);
            *cur.tv = typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            'converted: {
                match lua_type(lstate, -1) {
                    LUA_TNIL => {
                        (*cur.tv).v_type = VAR_SPECIAL;
                        (*cur.tv).vval.v_special = kSpecialVarNull;
                    }
                    LUA_TBOOLEAN => {
                        (*cur.tv).v_type = VAR_BOOL;
                        (*cur.tv).vval.v_bool = if lua_toboolean(lstate, -1) != 0 {
                            kBoolVarTrue
                        } else {
                            kBoolVarFalse
                        };
                    }
                    LUA_TSTRING => {
                        let mut len: size_t = 0;
                        let s = lua_tolstring(lstate, -1, &raw mut len);
                        *cur.tv = decode_string(s, len, false, false);
                    }
                    LUA_TNUMBER => {
                        let n = lua_tonumber(lstate, -1);
                        if n > VARNUMBER_MAX as lua_Number
                            || n < VARNUMBER_MIN as lua_Number
                            || (n as varnumber_T) as lua_Number != n
                        {
                            (*cur.tv).v_type = VAR_FLOAT;
                            (*cur.tv).vval.v_float = n;
                        } else {
                            (*cur.tv).v_type = VAR_NUMBER;
                            (*cur.tv).vval.v_number = n as varnumber_T;
                        }
                    }
                    LUA_TTABLE => {
                        // Only worth tracking a table reference when the
                        // table has a metatable of its own.
                        let mut table_ref: LuaRef = LUA_NOREF;
                        if lua_getmetatable(lstate, -1) != 0 {
                            lua_pop(lstate, 1);
                            table_ref = nlua_ref_global(lstate, -1);
                        }

                        let table_props = nlua_traverse_table(lstate);

                        // A container already on the stack is this same
                        // table: share it rather than descend forever.
                        for item in stack.iter() {
                            if item.container && lua_rawequal(lstate, -1, item.idx) != 0 {
                                tv_copy(item.tv, cur.tv);
                                cur.container = false;
                                break 'converted;
                            }
                        }

                        match table_props.type_0 {
                            kObjectTypeArray => {
                                (*cur.tv).v_type = VAR_LIST;
                                (*cur.tv).vval.v_list =
                                    tv_list_alloc(table_props.maxidx as ptrdiff_t);
                                (*(*cur.tv).vval.v_list).lua_table_ref = table_ref;
                                tv_list_ref((*cur.tv).vval.v_list);
                                cur.list_len = table_props.maxidx;
                                if table_props.maxidx != 0 {
                                    cur.container = true;
                                    cur.idx = lua_gettop(lstate);
                                    stack.push(cur);
                                }
                            }
                            kObjectTypeDict => {
                                if table_props.string_keys_num == 0 {
                                    new_dict(cur.tv, table_ref);
                                } else {
                                    cur.special = table_props.has_string_with_nul;
                                    if table_props.has_string_with_nul {
                                        // A key with a NUL in it has no
                                        // Vimscript dictionary image, so the
                                        // whole table becomes the `{_TYPE =
                                        // map, _VAL = [[k, v], …]}` special
                                        // form and `cur` descends into `_VAL`.
                                        decode_create_map_special_dict(
                                            cur.tv,
                                            table_props.string_keys_num as ptrdiff_t,
                                        );
                                        debug_assert!((*cur.tv).v_type == VAR_DICT);
                                        let val_di = tv_dict_find(
                                            (*cur.tv).vval.v_dict,
                                            c"_VAL".as_ptr(),
                                            4,
                                        );
                                        debug_assert!(!val_di.is_null());
                                        cur.tv = &raw mut (*val_di).di_tv;
                                        (*(*cur.tv).vval.v_list).lua_table_ref = table_ref;
                                        debug_assert!((*cur.tv).v_type == VAR_LIST);
                                        cur.list_len = table_props.string_keys_num;
                                    } else {
                                        new_dict(cur.tv, table_ref);
                                    }
                                    cur.container = true;
                                    cur.idx = lua_gettop(lstate);
                                    stack.push(cur);
                                    lua_pushnil(lstate);
                                }
                            }
                            kObjectTypeFloat => {
                                (*cur.tv).v_type = VAR_FLOAT;
                                (*cur.tv).vval.v_float = table_props.val;
                            }
                            kObjectTypeNil => {
                                emsg(gettext(E5100_MIXED_KEYS.as_ptr()));
                                ret = false;
                            }
                            _ => abort(),
                        }
                    }
                    LUA_TFUNCTION => {
                        let func = nlua_ref_global(lstate, -1);
                        let name = register_luafunc(func);
                        (*cur.tv).v_type = VAR_FUNC;
                        (*cur.tv).vval.v_string = xstrdup(name);
                    }
                    LUA_TUSERDATA => {
                        // TODO(bfredl): check mt.__call and convert to a
                        // function?
                        nlua_pushref(lstate, (*nlua_global_refs.get()).nil_ref);
                        let is_nil = lua_rawequal(lstate, -2, -1) != 0;
                        lua_pop(lstate, 1);
                        if is_nil {
                            (*cur.tv).v_type = VAR_SPECIAL;
                            (*cur.tv).vval.v_special = kSpecialVarNull;
                        } else {
                            emsg(gettext(E5101_BAD_TYPE.as_ptr()));
                            ret = false;
                        }
                    }
                    _ => {
                        emsg(gettext(E5101_BAD_TYPE.as_ptr()));
                        ret = false;
                    }
                }
            }
            if !cur.container {
                lua_pop(lstate, 1);
            }
        }
        if !ret {
            tv_clear(ret_tv);
            *ret_tv = typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            lua_pop(lstate, lua_gettop(lstate) - initial_size + 1);
        }
        debug_assert!(lua_gettop(lstate) == initial_size - 1);
        ret
    }
}
