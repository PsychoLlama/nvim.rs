//! `nlua_pop_typval()`: a Lua value as a Vimscript one.
//!
//! The Lua->typval direction, and the mirror of [`super::push`].  One
//! explicit stack of [`TVPopStackItem`]s rather than recursion, because a
//! Lua table may nest arbitrarily deep and the conversion has to be able to
//! refuse (`E5100`) rather than overflow.  Tables are classified by
//! [`nlua_traverse_table`] first, so a table's *shape* -- list, dictionary,
//! empty-dict, or a `{_TYPE, _VAL}` special -- is decided once.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/` row in docs/perimeter.md.
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::semsg;
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
use crate::lua::state::nlua_global_refs;
use crate::memory::xstrdup;
use crate::message::emsg;
use crate::narrow::float_as_i64;
use crate::os::cshim::gettext;
use crate::types::{
    LuaRef, TypVal, VAR_DICT, VAR_LIST, kBoolVarFalse, kBoolVarTrue, kObjectTypeArray,
    kObjectTypeDict, kObjectTypeFloat, kObjectTypeNil, kSpecialVarNull, lua_Number, lua_State,
    size_t,
};
use ::libc::abort;

/// Refused for a table that is neither a list nor a dictionary.
const E5100_MIXED_KEYS: &CStr = c"E5100: Cannot convert given Lua table: table should contain \
                                 either only integer keys or only string keys";
/// Refused for a Lua value with no Vimscript image at all.
const E5101_BAD_TYPE: &CStr = c"E5101: Cannot convert given Lua type";

/// One suspended container in the walk.
#[derive(Copy, Clone)]
pub struct TVPopStackItem {
    /// Where the conversion's result is to be stored.
    pub tv: *mut TypVal,
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
    const fn leaf(tv: *mut TypVal) -> Self {
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
pub unsafe fn nlua_pop_typval(lstate: *mut lua_State, ret_tv: *mut TypVal) -> bool {
    unsafe {
        // Make `tv` a fresh, referenced, empty dictionary carrying `ref_`.
        let new_dict = |tv: *mut TypVal, ref_: LuaRef| {
            (*tv).write_dict(tv_dict_alloc());
            (*(*tv).dict_or_null()).dv_refcount.retain();
            (*(*tv).dict_or_null()).lua_table_ref = ref_;
        };

        let mut ret = true;
        let initial_size = lua_gettop(lstate);
        let mut stack = TVPopStack::new();
        stack.push(TVPopStackItem::leaf(ret_tv));
        while ret && !stack.is_empty() {
            if lua_checkstack(lstate, lua_gettop(lstate) + 3) == 0 {
                let need = lua_gettop(lstate) + 3;
                semsg!("E1502: Lua failed to grow stack to {need}");
                ret = false;
                break;
            }
            let mut cur = stack.last();
            stack.pop();
            if cur.container {
                if cur.special || (*cur.tv).v_type() == VAR_DICT {
                    debug_assert!(
                        (*cur.tv).v_type() == if cur.special { VAR_LIST } else { VAR_DICT }
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
                        tv_list_append_list((*cur.tv).list_or_null(), kv_pair);
                        cur = TVPopStackItem::leaf(&raw mut (*tv_list_last(kv_pair)).li_tv);
                    } else {
                        let di = tv_dict_item_alloc_len(s, len);
                        if tv_dict_add((*cur.tv).dict_or_null(), di).is_err() {
                            abort();
                        }
                        stack.push(cur);
                        cur = TVPopStackItem::leaf(&raw mut (*di).di_tv);
                    }
                } else {
                    debug_assert!((*cur.tv).v_type() == VAR_LIST);
                    let list = (*cur.tv).list_or_null();
                    if usize::try_from(tv_list_len(list)).is_ok_and(|n| n == cur.list_len) {
                        lua_pop(lstate, 1);
                        continue;
                    }
                    lua_rawgeti(lstate, -1, tv_list_len(list) + 1);
                    // Not populated yet; append a list item to fill.
                    tv_list_append_owned_tv(list, TV_INITIAL_VALUE);
                    stack.push(cur);
                    // TODO(ZyX-I): use indexes, the list item *will* be
                    // reallocated here.
                    cur = TVPopStackItem::leaf(&raw mut (*tv_list_last(list)).li_tv);
                }
            }
            debug_assert!(!cur.container);
            *cur.tv = TypVal::Number(0);
            'converted: {
                match lua_type(lstate, -1) {
                    LUA_TNIL => {
                        (*cur.tv).write_special(kSpecialVarNull);
                    }
                    LUA_TBOOLEAN => {
                        (*cur.tv).write_boolean(if lua_toboolean(lstate, -1) != 0 {
                            kBoolVarTrue
                        } else {
                            kBoolVarFalse
                        });
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
                            || float_as_i64(n) as lua_Number != n
                        {
                            (*cur.tv).write_float(n);
                        } else {
                            (*cur.tv).write_number(float_as_i64(n));
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
                                (*cur.tv)
                                    .write_list(tv_list_alloc(table_props.maxidx.cast_signed()));
                                (*(*cur.tv).list_or_null()).lua_table_ref = table_ref;
                                tv_list_ref((*cur.tv).list_or_null());
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
                                            table_props.string_keys_num.cast_signed(),
                                        );
                                        debug_assert!((*cur.tv).v_type() == VAR_DICT);
                                        let val_di = tv_dict_find(
                                            (*cur.tv).dict_or_null(),
                                            c"_VAL".as_ptr(),
                                            4,
                                        );
                                        debug_assert!(!val_di.is_null());
                                        cur.tv = &raw mut (*val_di).di_tv;
                                        (*(*cur.tv).list_or_null()).lua_table_ref = table_ref;
                                        debug_assert!((*cur.tv).v_type() == VAR_LIST);
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
                                (*cur.tv).write_float(table_props.val);
                            }
                            kObjectTypeNil => {
                                emsg(gettext(E5100_MIXED_KEYS));
                                ret = false;
                            }
                            _ => abort(),
                        }
                    }
                    LUA_TFUNCTION => {
                        let func = nlua_ref_global(lstate, -1);
                        let name = register_luafunc(func);
                        (*cur.tv).write_func_name(xstrdup(name));
                    }
                    LUA_TUSERDATA => {
                        // TODO(bfredl): check mt.__call and convert to a
                        // function?
                        nlua_pushref(lstate, (*nlua_global_refs.get()).nil_ref);
                        let is_nil = lua_rawequal(lstate, -2, -1) != 0;
                        lua_pop(lstate, 1);
                        if is_nil {
                            (*cur.tv).write_special(kSpecialVarNull);
                        } else {
                            emsg(gettext(E5101_BAD_TYPE));
                            ret = false;
                        }
                    }
                    _ => {
                        emsg(gettext(E5101_BAD_TYPE));
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
            *ret_tv = TypVal::Number(0);
            lua_pop(lstate, lua_gettop(lstate) - initial_size + 1);
        }
        debug_assert!(lua_gettop(lstate) == initial_size - 1);
        ret
    }
}
