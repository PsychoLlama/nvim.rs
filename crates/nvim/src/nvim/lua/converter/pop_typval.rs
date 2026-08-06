//! `nlua_pop_typval()`: a Lua value as a Vimscript one.
//!
//! The Lua->typval direction, and the mirror of [`super::push`].  One
//! explicit stack of [`TVPopStackItem`]s rather than recursion, because a
//! Lua table may nest arbitrarily deep and the conversion has to be able to
//! refuse (`E5100`) rather than overflow.  Tables are classified by
//! [`nlua_traverse_table`] first, so a table's *shape* -- list, dictionary,
//! empty-dict, or a `{_TYPE, _VAL}` special -- is decided once.

// `nlua_pop_typval` is one 900-line transpiled body, and the four-space
// shift an `unsafe {}` wrap costs would put this file back over the
// 1,000-line cap.  Opt out until the rewrite shortens it.
#![allow(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C-unwind" fn nlua_pop_typval(
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
                                tv: &raw mut (*tv_list_last(kv_pair)).li_tv,
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
                            tv: &raw mut (*tv_list_last((*cur.tv).vval.v_list)).li_tv,
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
