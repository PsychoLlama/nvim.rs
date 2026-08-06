//! `nlua_pop_Object()`: a Lua value as an API [`Object`].
//!
//! The same explicit-stack walk as [`super::pop_typval`], over
//! [`ObjPopStackItem`] and producing api types instead of `typval_T`s.  It
//! is a separate walk because the two type systems disagree at the leaves:
//! an `Object` has no `VAR_SPECIAL`, carries `LuaRef`s for functions, and
//! allocates into an [`Arena`] when it is given one.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C-unwind" fn nlua_pop_Object(
    lstate: *mut lua_State,
    mut ref_0: bool,
    mut arena: *mut Arena,
    err: *mut Error,
) -> Object {
    unsafe {
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
                                lua_settop(
                                    lstate,
                                    -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                                );
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
                                            stack.capacity.wrapping_mul(::core::mem::size_of::<
                                                ObjPopStackItem,
                                            >(
                                            )),
                                        )
                                    }
                                })
                                    as *mut ObjPopStackItem;
                            } else {
                            };
                            let c2rust_fresh17 = stack.size;
                            stack.size = stack.size.wrapping_add(1);
                            *stack.items.offset(c2rust_fresh17 as isize) = cur;
                            cur = ObjPopStackItem {
                                obj: &raw mut (*(*cur.obj).data.dict.items.offset(idx as isize))
                                    .value,
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
                                        stack.capacity = if stack.capacity
                                            << 1 as ::core::ffi::c_int
                                            > ::core::mem::size_of::<[ObjPopStackItem; 2]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<ObjPopStackItem>(),
                                                )
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
                                                .wrapping_div(
                                                    ::core::mem::size_of::<ObjPopStackItem>(),
                                                )
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
                                                .wrapping_div(
                                                    ::core::mem::size_of::<ObjPopStackItem>(),
                                                )
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
                                                    &raw mut stack.init_array
                                                        as *mut ObjPopStackItem
                                                        as *mut ::core::ffi::c_void,
                                                    stack.items as *mut ::core::ffi::c_void,
                                                    stack.size.wrapping_mul(
                                                        ::core::mem::size_of::<ObjPopStackItem>(),
                                                    ),
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
                                                    stack.size.wrapping_mul(
                                                        ::core::mem::size_of::<ObjPopStackItem>(),
                                                    ),
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
                                        stack.capacity = if stack.capacity
                                            << 1 as ::core::ffi::c_int
                                            > ::core::mem::size_of::<[ObjPopStackItem; 2]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<ObjPopStackItem>(),
                                                )
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
                                                .wrapping_div(
                                                    ::core::mem::size_of::<ObjPopStackItem>(),
                                                )
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
                                                .wrapping_div(
                                                    ::core::mem::size_of::<ObjPopStackItem>(),
                                                )
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
                                                    &raw mut stack.init_array
                                                        as *mut ObjPopStackItem
                                                        as *mut ::core::ffi::c_void,
                                                    stack.items as *mut ::core::ffi::c_void,
                                                    stack.size.wrapping_mul(
                                                        ::core::mem::size_of::<ObjPopStackItem>(),
                                                    ),
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
                                                    stack.size.wrapping_mul(
                                                        ::core::mem::size_of::<ObjPopStackItem>(),
                                                    ),
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
                        let mut is_nil: bool = lua_rawequal(
                            lstate,
                            -2 as ::core::ffi::c_int,
                            -1 as ::core::ffi::c_int,
                        ) != 0;
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
                    b"lua_gettop(lstate) == initial_size - 1\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/lua/converter.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1211 as ::core::ffi::c_uint,
                    b"Object nlua_pop_Object(lua_State *const, _Bool, Arena *, Error *const)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        return ret;
    }
}
