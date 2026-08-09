use crate::src::nvim::api::buffer::buf_collect_lines;
use crate::src::nvim::api::private::helpers::arena_array;
use crate::src::nvim::buffer::buf_get_changedtick;

use crate::src::nvim::log::{LOGLVL_ERR, logmsg_c};
use crate::src::nvim::lua::executor::{api_free_luaref, nlua_call_ref};
use crate::src::nvim::main::{cmdpreview, curbuf, curwin, textlock};
use crate::src::nvim::memline::ml_flush_deleted_bytes;
use crate::src::nvim::memory::{ARENA_EMPTY, arena_finish, arena_mem_free, xfree, xrealloc};
use crate::src::nvim::msgpack_rpc::channel::rpc_send_event;
use crate::src::nvim::types::{
    Arena, Array, BufUpdateCallbacks, Error, Integer, LuaRef, LuaRetMode, Object, bcount_t, buf_T,
    colnr_T, int64_t, kObjectTypeArray, kObjectTypeBoolean, kObjectTypeBuffer, kObjectTypeInteger,
    kObjectTypeNil, linenr_T, lua_State, object, object_data as C2Rust_Unnamed, pos_T, size_t,
    uint64_t,
};
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const INTERNAL_CALL_MASK: uint64_t = (1 as ::core::ffi::c_int as uint64_t)
    << ::core::mem::size_of::<uint64_t>()
        .wrapping_mul(8_usize)
        .wrapping_sub(1_usize);
pub const VIML_INTERNAL_CALL: uint64_t = INTERNAL_CALL_MASK;
pub const LUA_INTERNAL_CALL: uint64_t = VIML_INTERNAL_CALL.wrapping_add(1 as uint64_t);
pub unsafe extern "C" fn buf_updates_register(
    mut buf: *mut buf_T,
    mut channel_id: uint64_t,
    mut cb: BufUpdateCallbacks,
    mut send_buffer: bool,
) -> bool {
    if (*buf).b_ml.ml_mfp.is_null() {
        return false_0 != 0;
    }
    if channel_id == LUA_INTERNAL_CALL {
        if (*buf).update_callbacks.size == (*buf).update_callbacks.capacity {
            (*buf).update_callbacks.capacity = if (*buf).update_callbacks.capacity != 0 {
                (*buf).update_callbacks.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            (*buf).update_callbacks.items = xrealloc(
                (*buf).update_callbacks.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<BufUpdateCallbacks>()
                    .wrapping_mul((*buf).update_callbacks.capacity),
            ) as *mut BufUpdateCallbacks;
        } else {
        };
        let c2rust_fresh0 = (*buf).update_callbacks.size;
        (*buf).update_callbacks.size = (*buf).update_callbacks.size.wrapping_add(1);
        *(*buf).update_callbacks.items.add(c2rust_fresh0) = cb;
        if cb.utf_sizes {
            (*buf).update_need_codepoints = true_0 != 0;
        }
        return true_0 != 0;
    }
    let mut size: size_t = (*buf).update_channels.size;
    let mut i: size_t = 0 as size_t;
    while i < size {
        if *(*buf).update_channels.items.add(i) == channel_id {
            return true_0 != 0;
        }
        i = i.wrapping_add(1);
    }
    if (*buf).update_channels.size == (*buf).update_channels.capacity {
        (*buf).update_channels.capacity = if (*buf).update_channels.capacity != 0 {
            (*buf).update_channels.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        (*buf).update_channels.items = xrealloc(
            (*buf).update_channels.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<uint64_t>().wrapping_mul((*buf).update_channels.capacity),
        ) as *mut uint64_t;
    } else {
    };
    let c2rust_fresh1 = (*buf).update_channels.size;
    (*buf).update_channels.size = (*buf).update_channels.size.wrapping_add(1);
    *(*buf).update_channels.items.add(c2rust_fresh1) = channel_id;
    if send_buffer {
        let mut args: Array = ARRAY_DICT_INIT;
        let mut args__items: [Object; 6] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 6];
        args.capacity = 6 as size_t;
        args.items = &raw mut args__items as *mut Object;
        let c2rust_fresh2 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.add(c2rust_fresh2) = object {
            type_0: kObjectTypeBuffer,
            data: C2Rust_Unnamed {
                integer: (*buf).handle as Integer,
            },
        };
        let c2rust_fresh3 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.add(c2rust_fresh3) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: buf_get_changedtick(buf),
            },
        };
        let c2rust_fresh4 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.add(c2rust_fresh4) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: 0 as Integer,
            },
        };
        let c2rust_fresh5 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.add(c2rust_fresh5) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: -1 as Integer,
            },
        };
        let mut line_count: size_t = (*buf).b_ml.ml_line_count as size_t;
        let mut linedata: Array = ARRAY_DICT_INIT;
        let mut arena: Arena = ARENA_EMPTY;
        if line_count > 0 as size_t {
            linedata = arena_array(&raw mut arena, line_count);
            buf_collect_lines(
                buf,
                line_count,
                1 as linenr_T,
                0 as ::core::ffi::c_int,
                true_0 != 0,
                &raw mut linedata,
                ::core::ptr::null_mut::<lua_State>(),
                &raw mut arena,
            );
        }
        let c2rust_fresh6 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.add(c2rust_fresh6) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed { array: linedata },
        };
        let c2rust_fresh7 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.add(c2rust_fresh7) = object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed { boolean: false },
        };
        rpc_send_event(channel_id, c"nvim_buf_lines_event".as_ptr(), args);
        arena_mem_free(arena_finish(&raw mut arena));
    } else {
        buf_updates_changedtick_single(buf, channel_id);
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn buf_updates_active(mut buf: *mut buf_T) -> bool {
    return (*buf).update_channels.size != 0 || (*buf).update_callbacks.size != 0;
}
pub unsafe extern "C" fn buf_updates_send_end(mut buf: *mut buf_T, mut channelid: uint64_t) {
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh10 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.add(c2rust_fresh10) = object {
        type_0: kObjectTypeBuffer,
        data: C2Rust_Unnamed {
            integer: (*buf).handle as Integer,
        },
    };
    rpc_send_event(channelid, c"nvim_buf_detach_event".as_ptr(), args);
}
pub unsafe extern "C" fn buf_updates_unregister(mut buf: *mut buf_T, mut channelid: uint64_t) {
    let mut size: size_t = (*buf).update_channels.size;
    if size == 0 {
        return;
    }
    let mut j: size_t = 0 as size_t;
    let mut found: size_t = 0 as size_t;
    let mut i: size_t = 0 as size_t;
    while i < size {
        if *(*buf).update_channels.items.add(i) == channelid {
            found = found.wrapping_add(1);
        } else {
            if i != j {
                *(*buf).update_channels.items.add(j) = *(*buf).update_channels.items.add(i);
            }
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    if found != 0 {
        (*buf).update_channels.size = (*buf).update_channels.size.wrapping_sub(found);
        buf_updates_send_end(buf, channelid);
        if found == size {
            xfree((*buf).update_channels.items as *mut ::core::ffi::c_void);
            (*buf).update_channels.capacity = 0 as size_t;
            (*buf).update_channels.size = (*buf).update_channels.capacity;
            (*buf).update_channels.items = ::core::ptr::null_mut::<uint64_t>();
            (*buf).update_channels.capacity = 0 as size_t;
            (*buf).update_channels.size = (*buf).update_channels.capacity;
            (*buf).update_channels.items = ::core::ptr::null_mut::<uint64_t>();
        }
    }
}
pub unsafe extern "C" fn buf_free_callbacks(mut buf: *mut buf_T) {
    xfree((*buf).update_channels.items as *mut ::core::ffi::c_void);
    (*buf).update_channels.capacity = 0 as size_t;
    (*buf).update_channels.size = (*buf).update_channels.capacity;
    (*buf).update_channels.items = ::core::ptr::null_mut::<uint64_t>();
    let mut i: size_t = 0 as size_t;
    while i < (*buf).update_callbacks.size {
        buffer_update_callbacks_free(*(*buf).update_callbacks.items.add(i));
        i = i.wrapping_add(1);
    }
    xfree((*buf).update_callbacks.items as *mut ::core::ffi::c_void);
    (*buf).update_callbacks.capacity = 0 as size_t;
    (*buf).update_callbacks.size = (*buf).update_callbacks.capacity;
    (*buf).update_callbacks.items = ::core::ptr::null_mut::<BufUpdateCallbacks>();
}
pub unsafe extern "C" fn buf_updates_unload(mut buf: *mut buf_T, mut can_reload: bool) {
    let mut size: size_t = (*buf).update_channels.size;
    if size != 0 {
        let mut i: size_t = 0 as size_t;
        while i < size {
            buf_updates_send_end(buf, *(*buf).update_channels.items.add(i));
            i = i.wrapping_add(1);
        }
        xfree((*buf).update_channels.items as *mut ::core::ffi::c_void);
        (*buf).update_channels.capacity = 0 as size_t;
        (*buf).update_channels.size = (*buf).update_channels.capacity;
        (*buf).update_channels.items = ::core::ptr::null_mut::<uint64_t>();
        (*buf).update_channels.capacity = 0 as size_t;
        (*buf).update_channels.size = (*buf).update_channels.capacity;
        (*buf).update_channels.items = ::core::ptr::null_mut::<uint64_t>();
    }
    let mut j: size_t = 0 as size_t;
    let mut i_0: size_t = 0 as size_t;
    while i_0 < (*buf).update_callbacks.size {
        let mut cb: BufUpdateCallbacks = *(*buf).update_callbacks.items.add(i_0);
        let mut thecb: LuaRef = LUA_NOREF;
        let mut keep: bool = false_0 != 0;
        if can_reload as ::core::ffi::c_int != 0 && cb.on_reload != LUA_NOREF {
            keep = true_0 != 0;
            thecb = cb.on_reload;
        } else if cb.on_detach != LUA_NOREF {
            thecb = cb.on_detach;
        }
        if thecb != LUA_NOREF {
            let mut args: Array = ARRAY_DICT_INIT;
            let mut args__items: [Object; 1] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            }; 1];
            args.capacity = 1 as size_t;
            args.items = &raw mut args__items as *mut Object;
            let c2rust_fresh11 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.add(c2rust_fresh11) = object {
                type_0: kObjectTypeBuffer,
                data: C2Rust_Unnamed {
                    integer: (*buf).handle as Integer,
                },
            };
            let save_cursor: pos_T = (*curwin.get()).w_cursor;
            (*textlock.ptr()) += 1;
            nlua_call_ref(
                thecb,
                if keep as ::core::ffi::c_int != 0 {
                    c"reload".as_ptr()
                } else {
                    c"detach".as_ptr()
                },
                args,
                kRetObject,
                ::core::ptr::null_mut::<Arena>(),
                ::core::ptr::null_mut::<Error>(),
            );
            (*textlock.ptr()) -= 1;
            (*curwin.get()).w_cursor = save_cursor;
        }
        if keep {
            let c2rust_fresh12 = j;
            j = j.wrapping_add(1);
            *(*buf).update_callbacks.items.add(c2rust_fresh12) =
                *(*buf).update_callbacks.items.add(i_0);
        } else {
            buffer_update_callbacks_free(cb);
        }
        i_0 = i_0.wrapping_add(1);
    }
    (*buf).update_callbacks.size = j;
    if (*buf).update_callbacks.size == 0 as size_t {
        xfree((*buf).update_callbacks.items as *mut ::core::ffi::c_void);
        (*buf).update_callbacks.capacity = 0 as size_t;
        (*buf).update_callbacks.size = (*buf).update_callbacks.capacity;
        (*buf).update_callbacks.items = ::core::ptr::null_mut::<BufUpdateCallbacks>();
        (*buf).update_callbacks.capacity = 0 as size_t;
        (*buf).update_callbacks.size = (*buf).update_callbacks.capacity;
        (*buf).update_callbacks.items = ::core::ptr::null_mut::<BufUpdateCallbacks>();
    }
}
pub unsafe extern "C" fn buf_updates_send_changes(
    mut buf: *mut buf_T,
    mut firstline: linenr_T,
    mut num_added: int64_t,
    mut num_removed: int64_t,
) {
    let mut deleted_codepoints: size_t = 0;
    let mut deleted_codeunits: size_t = 0;
    let mut deleted_bytes: size_t =
        ml_flush_deleted_bytes(buf, &raw mut deleted_codepoints, &raw mut deleted_codeunits);
    if !buf_updates_active(buf) {
        return;
    }
    let mut send_tick: bool = !(cmdpreview.get() as ::core::ffi::c_int != 0 && buf == curbuf.get());
    let mut badchannelid: uint64_t = 0 as uint64_t;
    let mut arena: Arena = ARENA_EMPTY;
    let mut linedata: Array = ARRAY_DICT_INIT;
    if num_added > 0 as int64_t && (*buf).update_channels.size != 0 {
        linedata = arena_array(&raw mut arena, num_added as size_t);
        buf_collect_lines(
            buf,
            num_added as size_t,
            firstline,
            0 as ::core::ffi::c_int,
            true_0 != 0,
            &raw mut linedata,
            ::core::ptr::null_mut::<lua_State>(),
            &raw mut arena,
        );
    }
    let mut i: size_t = 0 as size_t;
    while i < (*buf).update_channels.size {
        let mut channelid: uint64_t = *(*buf).update_channels.items.add(i);
        let mut args: Array = ARRAY_DICT_INIT;
        let mut args__items: [Object; 6] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 6];
        args.capacity = 6 as size_t;
        args.items = &raw mut args__items as *mut Object;
        let c2rust_fresh13 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.add(c2rust_fresh13) = object {
            type_0: kObjectTypeBuffer,
            data: C2Rust_Unnamed {
                integer: (*buf).handle as Integer,
            },
        };
        let c2rust_fresh14 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.add(c2rust_fresh14) = if send_tick as ::core::ffi::c_int != 0 {
            object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: buf_get_changedtick(buf),
                },
            }
        } else {
            object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            }
        };
        let c2rust_fresh15 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.add(c2rust_fresh15) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (firstline - 1 as linenr_T) as Integer,
            },
        };
        let c2rust_fresh16 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.add(c2rust_fresh16) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (firstline - 1 as linenr_T) as int64_t + num_removed,
            },
        };
        let c2rust_fresh17 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.add(c2rust_fresh17) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed { array: linedata },
        };
        let c2rust_fresh18 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.add(c2rust_fresh18) = object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed { boolean: false },
        };
        if !rpc_send_event(channelid, c"nvim_buf_lines_event".as_ptr(), args) {
            badchannelid = channelid;
        }
        i = i.wrapping_add(1);
    }
    if badchannelid != 0 as uint64_t {
        logmsg_c!(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            c"buf_updates_send_changes".as_ptr(),
            258 as ::core::ffi::c_int,
            true_0 != 0,
            c"Disabling buffer updates for dead channel %lu".as_ptr(),
            badchannelid,
        );
        buf_updates_unregister(buf, badchannelid);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    let mut j: size_t = 0 as size_t;
    let mut i_0: size_t = 0 as size_t;
    while i_0 < (*buf).update_callbacks.size {
        let mut cb: BufUpdateCallbacks = *(*buf).update_callbacks.items.add(i_0);
        let mut keep: bool = true_0 != 0;
        if cb.on_lines != LUA_NOREF && (cb.preview as ::core::ffi::c_int != 0 || !cmdpreview.get())
        {
            let mut args_0: Array = ARRAY_DICT_INIT;
            let mut args__items_0: [Object; 8] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            }; 8];
            args_0.capacity = 8 as size_t;
            args_0.items = &raw mut args__items_0 as *mut Object;
            let c2rust_fresh19 = args_0.size;
            args_0.size = args_0.size.wrapping_add(1);
            *args_0.items.add(c2rust_fresh19) = object {
                type_0: kObjectTypeBuffer,
                data: C2Rust_Unnamed {
                    integer: (*buf).handle as Integer,
                },
            };
            let c2rust_fresh20 = args_0.size;
            args_0.size = args_0.size.wrapping_add(1);
            *args_0.items.add(c2rust_fresh20) = if send_tick as ::core::ffi::c_int != 0 {
                object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: buf_get_changedtick(buf),
                    },
                }
            } else {
                object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                }
            };
            let c2rust_fresh21 = args_0.size;
            args_0.size = args_0.size.wrapping_add(1);
            *args_0.items.add(c2rust_fresh21) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (firstline - 1 as linenr_T) as Integer,
                },
            };
            let c2rust_fresh22 = args_0.size;
            args_0.size = args_0.size.wrapping_add(1);
            *args_0.items.add(c2rust_fresh22) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (firstline - 1 as linenr_T) as int64_t + num_removed,
                },
            };
            let c2rust_fresh23 = args_0.size;
            args_0.size = args_0.size.wrapping_add(1);
            *args_0.items.add(c2rust_fresh23) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (firstline - 1 as linenr_T) as int64_t + num_added,
                },
            };
            let c2rust_fresh24 = args_0.size;
            args_0.size = args_0.size.wrapping_add(1);
            *args_0.items.add(c2rust_fresh24) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: deleted_bytes as Integer,
                },
            };
            if cb.utf_sizes {
                let c2rust_fresh25 = args_0.size;
                args_0.size = args_0.size.wrapping_add(1);
                *args_0.items.add(c2rust_fresh25) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: deleted_codepoints as Integer,
                    },
                };
                let c2rust_fresh26 = args_0.size;
                args_0.size = args_0.size.wrapping_add(1);
                *args_0.items.add(c2rust_fresh26) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: deleted_codeunits as Integer,
                    },
                };
            }
            let mut res: Object = Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
            let save_cursor: pos_T = (*curwin.get()).w_cursor;
            (*textlock.ptr()) += 1;
            res = nlua_call_ref(
                cb.on_lines,
                c"lines".as_ptr(),
                args_0,
                kRetNilBool,
                ::core::ptr::null_mut::<Arena>(),
                ::core::ptr::null_mut::<Error>(),
            );
            (*textlock.ptr()) -= 1;
            (*curwin.get()).w_cursor = save_cursor;
            if res.type_0 as ::core::ffi::c_uint
                == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
                && res.data.boolean as ::core::ffi::c_int == true_0
            {
                buffer_update_callbacks_free(cb);
                keep = false_0 != 0;
            }
        }
        if keep {
            let c2rust_fresh27 = j;
            j = j.wrapping_add(1);
            *(*buf).update_callbacks.items.add(c2rust_fresh27) =
                *(*buf).update_callbacks.items.add(i_0);
        }
        i_0 = i_0.wrapping_add(1);
    }
    (*buf).update_callbacks.size = j;
}
pub unsafe extern "C" fn buf_updates_send_splice(
    mut buf: *mut buf_T,
    mut start_row: ::core::ffi::c_int,
    mut start_col: colnr_T,
    mut start_byte: bcount_t,
    mut old_row: ::core::ffi::c_int,
    mut old_col: colnr_T,
    mut old_byte: bcount_t,
    mut new_row: ::core::ffi::c_int,
    mut new_col: colnr_T,
    mut new_byte: bcount_t,
) {
    if !buf_updates_active(buf) || old_byte == 0 as bcount_t && new_byte == 0 as bcount_t {
        return;
    }
    let mut j: size_t = 0 as size_t;
    let mut i: size_t = 0 as size_t;
    while i < (*buf).update_callbacks.size {
        let mut cb: BufUpdateCallbacks = *(*buf).update_callbacks.items.add(i);
        let mut keep: bool = true_0 != 0;
        if cb.on_bytes != LUA_NOREF && (cb.preview as ::core::ffi::c_int != 0 || !cmdpreview.get())
        {
            let mut args: Array = ARRAY_DICT_INIT;
            let mut args__items: [Object; 11] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            }; 11];
            args.capacity = 11 as size_t;
            args.items = &raw mut args__items as *mut Object;
            let c2rust_fresh28 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.add(c2rust_fresh28) = object {
                type_0: kObjectTypeBuffer,
                data: C2Rust_Unnamed {
                    integer: (*buf).handle as Integer,
                },
            };
            let c2rust_fresh29 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.add(c2rust_fresh29) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: buf_get_changedtick(buf),
                },
            };
            let c2rust_fresh30 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.add(c2rust_fresh30) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: start_row as Integer,
                },
            };
            let c2rust_fresh31 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.add(c2rust_fresh31) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: start_col as Integer,
                },
            };
            let c2rust_fresh32 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.add(c2rust_fresh32) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: start_byte as i64,
                },
            };
            let c2rust_fresh33 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.add(c2rust_fresh33) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: old_row as Integer,
                },
            };
            let c2rust_fresh34 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.add(c2rust_fresh34) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: old_col as Integer,
                },
            };
            let c2rust_fresh35 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.add(c2rust_fresh35) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: old_byte as i64,
                },
            };
            let c2rust_fresh36 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.add(c2rust_fresh36) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: new_row as Integer,
                },
            };
            let c2rust_fresh37 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.add(c2rust_fresh37) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: new_col as Integer,
                },
            };
            let c2rust_fresh38 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.add(c2rust_fresh38) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: new_byte as i64,
                },
            };
            let mut res: Object = Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
            let save_cursor: pos_T = (*curwin.get()).w_cursor;
            (*textlock.ptr()) += 1;
            res = nlua_call_ref(
                cb.on_bytes,
                c"bytes".as_ptr(),
                args,
                kRetNilBool,
                ::core::ptr::null_mut::<Arena>(),
                ::core::ptr::null_mut::<Error>(),
            );
            (*textlock.ptr()) -= 1;
            (*curwin.get()).w_cursor = save_cursor;
            if res.type_0 as ::core::ffi::c_uint
                == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
                && res.data.boolean as ::core::ffi::c_int == true_0
            {
                buffer_update_callbacks_free(cb);
                keep = false_0 != 0;
            }
        }
        if keep {
            let c2rust_fresh39 = j;
            j = j.wrapping_add(1);
            *(*buf).update_callbacks.items.add(c2rust_fresh39) =
                *(*buf).update_callbacks.items.add(i);
        }
        i = i.wrapping_add(1);
    }
    (*buf).update_callbacks.size = j;
}
pub unsafe extern "C" fn buf_updates_changedtick(mut buf: *mut buf_T) {
    let mut i: size_t = 0 as size_t;
    while i < (*buf).update_channels.size {
        let mut channel_id: uint64_t = *(*buf).update_channels.items.add(i);
        buf_updates_changedtick_single(buf, channel_id);
        i = i.wrapping_add(1);
    }
    let mut j: size_t = 0 as size_t;
    let mut i_0: size_t = 0 as size_t;
    while i_0 < (*buf).update_callbacks.size {
        let mut cb: BufUpdateCallbacks = *(*buf).update_callbacks.items.add(i_0);
        let mut keep: bool = true_0 != 0;
        if cb.on_changedtick != LUA_NOREF {
            let mut args: Array = ARRAY_DICT_INIT;
            let mut args__items: [Object; 2] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            }; 2];
            args.capacity = 2 as size_t;
            args.items = &raw mut args__items as *mut Object;
            let c2rust_fresh40 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.add(c2rust_fresh40) = object {
                type_0: kObjectTypeBuffer,
                data: C2Rust_Unnamed {
                    integer: (*buf).handle as Integer,
                },
            };
            let c2rust_fresh41 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.add(c2rust_fresh41) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: buf_get_changedtick(buf),
                },
            };
            let mut res: Object = Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
            let save_cursor: pos_T = (*curwin.get()).w_cursor;
            (*textlock.ptr()) += 1;
            res = nlua_call_ref(
                cb.on_changedtick,
                c"changedtick".as_ptr(),
                args,
                kRetNilBool,
                ::core::ptr::null_mut::<Arena>(),
                ::core::ptr::null_mut::<Error>(),
            );
            (*textlock.ptr()) -= 1;
            (*curwin.get()).w_cursor = save_cursor;
            if res.type_0 as ::core::ffi::c_uint
                == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
                && res.data.boolean as ::core::ffi::c_int == true_0
            {
                buffer_update_callbacks_free(cb);
                keep = false_0 != 0;
            }
        }
        if keep {
            let c2rust_fresh42 = j;
            j = j.wrapping_add(1);
            *(*buf).update_callbacks.items.add(c2rust_fresh42) =
                *(*buf).update_callbacks.items.add(i_0);
        }
        i_0 = i_0.wrapping_add(1);
    }
    (*buf).update_callbacks.size = j;
}
pub unsafe extern "C" fn buf_updates_changedtick_single(
    mut buf: *mut buf_T,
    mut channel_id: uint64_t,
) {
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 2] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 2];
    args.capacity = 2 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh8 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.add(c2rust_fresh8) = object {
        type_0: kObjectTypeBuffer,
        data: C2Rust_Unnamed {
            integer: (*buf).handle as Integer,
        },
    };
    let c2rust_fresh9 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.add(c2rust_fresh9) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: buf_get_changedtick(buf),
        },
    };
    rpc_send_event(channel_id, c"nvim_buf_changedtick_event".as_ptr(), args);
}
pub unsafe extern "C" fn buffer_update_callbacks_free(mut cb: BufUpdateCallbacks) {
    api_free_luaref(cb.on_lines);
    api_free_luaref(cb.on_bytes);
    api_free_luaref(cb.on_changedtick);
    api_free_luaref(cb.on_reload);
    api_free_luaref(cb.on_detach);
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
