use crate::src::nvim::api::buffer::buf_collect_lines;
use crate::src::nvim::api::private::helpers::arena_array;

use crate::src::nvim::log::logmsg;
use crate::src::nvim::lua::executor::{api_free_luaref, nlua_call_ref};
use crate::src::nvim::main::{cmdpreview, curbuf, curwin, textlock};
use crate::src::nvim::memline::ml_flush_deleted_bytes;
use crate::src::nvim::memory::{arena_finish, arena_mem_free, xfree, xrealloc, ARENA_EMPTY};
use crate::src::nvim::msgpack_rpc::channel::rpc_send_event;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, Arena, ArenaMem, Array, BoolVarValue, Boolean,
    BufUpdateCallbacks, Callback, CallbackType, Callback_data as C2Rust_Unnamed_5,
    ChangedtickDictItem, DecorExt, DecorHighlightInline, DecorInlineData, DecorPriority,
    DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2, Dict, Error, ErrorType,
    ExtmarkUndoObject, FileID, Float, FloatAnchor, FloatRelative, GridView, Integer, Intersection,
    KeyValuePair, LuaRef, LuaRetMode, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t,
    Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, Object, ObjectType,
    OptInt, ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t,
    SpecialVarValue, StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_12, String_0,
    Terminal, Timestamp, VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos,
    WinConfig, WinInfo, WinSplit, WinStyle, Window, __time_t, alist_T, bcount_t, bhdr_T, blob_T,
    blobvar_S, blocknr_T, buf_T, bufstate_T, chunksize_T, colnr_T, consumed_blk, dict_T, dictvar_S,
    disptick_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_3, file_buffer_b_wininfo as C2Rust_Unnamed_11,
    file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, key_value_pair, lcs_chars_T,
    linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T,
    lua_State, mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T,
    mfdirty_T, mtnode_inner_s, mtnode_s, object, object_data as C2Rust_Unnamed, partial_S,
    partial_T, pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue,
    reg_extmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T, size_t,
    syn_state, syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T,
    taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T, u_header,
    u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_8, u_header_uh_alt_prev as C2Rust_Unnamed_7,
    u_header_uh_next as C2Rust_Unnamed_10, u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T,
    uint16_t, uint32_t, uint64_t, uint8_t, undo_object, varnumber_T, virt_line, visualinfo_T,
    win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T, QUEUE,
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
pub const kVPosWinCol: VirtTextPos = 5;
pub const kVPosRightAlign: VirtTextPos = 4;
pub const kVPosOverlay: VirtTextPos = 3;
pub const kVPosInline: VirtTextPos = 2;
pub const kVPosEndOfLineRightAlign: VirtTextPos = 1;
pub const kVPosEndOfLine: VirtTextPos = 0;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
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
pub const kStlClickFuncRun: C2Rust_Unnamed_12 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_12 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_12 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_12 = 0;
pub const kAlignRight: AlignTextPos = 2;
pub const kAlignCenter: AlignTextPos = 1;
pub const kAlignLeft: AlignTextPos = 0;
pub const kWinStyleMinimal: WinStyle = 1;
pub const kWinStyleUnused: WinStyle = 0;
pub const kWinSplitBelow: WinSplit = 3;
pub const kWinSplitAbove: WinSplit = 2;
pub const kWinSplitRight: WinSplit = 1;
pub const kWinSplitLeft: WinSplit = 0;
pub const kFloatRelativeLaststatus: FloatRelative = 5;
pub const kFloatRelativeTabline: FloatRelative = 4;
pub const kFloatRelativeMouse: FloatRelative = 3;
pub const kFloatRelativeCursor: FloatRelative = 2;
pub const kFloatRelativeWindow: FloatRelative = 1;
pub const kFloatRelativeEditor: FloatRelative = 0;
pub const MF_DIRTY_YES_NOSYNC: mfdirty_T = 2;
pub const MF_DIRTY_YES: mfdirty_T = 1;
pub const MF_DIRTY_NO: mfdirty_T = 0;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed_13 = 2147483647;
pub const kRetMulti: LuaRetMode = 3;
pub const kRetLuaref: LuaRetMode = 2;
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
        .wrapping_mul(8 as usize)
        .wrapping_sub(1 as usize);
pub const VIML_INTERNAL_CALL: uint64_t = INTERNAL_CALL_MASK;
pub const LUA_INTERNAL_CALL: uint64_t = VIML_INTERNAL_CALL.wrapping_add(1 as uint64_t);
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn buf_get_changedtick(buf: *const buf_T) -> varnumber_T {
    return (*buf).changedtick_di.di_tv.vval.v_number;
}
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
        *(*buf).update_callbacks.items.offset(c2rust_fresh0 as isize) = cb;
        if cb.utf_sizes {
            (*buf).update_need_codepoints = true_0 != 0;
        }
        return true_0 != 0;
    }
    let mut size: size_t = (*buf).update_channels.size;
    let mut i: size_t = 0 as size_t;
    while i < size {
        if *(*buf).update_channels.items.offset(i as isize) == channel_id {
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
    *(*buf).update_channels.items.offset(c2rust_fresh1 as isize) = channel_id;
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
        *args.items.offset(c2rust_fresh2 as isize) = object {
            type_0: kObjectTypeBuffer,
            data: C2Rust_Unnamed {
                integer: (*buf).handle as Integer,
            },
        };
        let c2rust_fresh3 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh3 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: buf_get_changedtick(buf),
            },
        };
        let c2rust_fresh4 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh4 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: 0 as Integer,
            },
        };
        let c2rust_fresh5 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh5 as isize) = object {
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
        *args.items.offset(c2rust_fresh6 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed { array: linedata },
        };
        let c2rust_fresh7 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh7 as isize) = object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed { boolean: false },
        };
        rpc_send_event(
            channel_id,
            b"nvim_buf_lines_event\0".as_ptr() as *const ::core::ffi::c_char,
            args,
        );
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
    *args.items.offset(c2rust_fresh10 as isize) = object {
        type_0: kObjectTypeBuffer,
        data: C2Rust_Unnamed {
            integer: (*buf).handle as Integer,
        },
    };
    rpc_send_event(
        channelid,
        b"nvim_buf_detach_event\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
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
        if *(*buf).update_channels.items.offset(i as isize) == channelid {
            found = found.wrapping_add(1);
        } else {
            if i != j {
                *(*buf).update_channels.items.offset(j as isize) =
                    *(*buf).update_channels.items.offset(i as isize);
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
        buffer_update_callbacks_free(*(*buf).update_callbacks.items.offset(i as isize));
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
            buf_updates_send_end(buf, *(*buf).update_channels.items.offset(i as isize));
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
        let mut cb: BufUpdateCallbacks = *(*buf).update_callbacks.items.offset(i_0 as isize);
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
            *args.items.offset(c2rust_fresh11 as isize) = object {
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
                    b"reload\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"detach\0".as_ptr() as *const ::core::ffi::c_char
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
            *(*buf)
                .update_callbacks
                .items
                .offset(c2rust_fresh12 as isize) =
                *(*buf).update_callbacks.items.offset(i_0 as isize);
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
        let mut channelid: uint64_t = *(*buf).update_channels.items.offset(i as isize);
        let mut args: Array = ARRAY_DICT_INIT;
        let mut args__items: [Object; 6] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 6];
        args.capacity = 6 as size_t;
        args.items = &raw mut args__items as *mut Object;
        let c2rust_fresh13 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh13 as isize) = object {
            type_0: kObjectTypeBuffer,
            data: C2Rust_Unnamed {
                integer: (*buf).handle as Integer,
            },
        };
        let c2rust_fresh14 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh14 as isize) = if send_tick as ::core::ffi::c_int != 0 {
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
        *args.items.offset(c2rust_fresh15 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (firstline - 1 as linenr_T) as Integer,
            },
        };
        let c2rust_fresh16 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh16 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (firstline - 1 as linenr_T) as int64_t + num_removed,
            },
        };
        let c2rust_fresh17 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh17 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed { array: linedata },
        };
        let c2rust_fresh18 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh18 as isize) = object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed { boolean: false },
        };
        if !rpc_send_event(
            channelid,
            b"nvim_buf_lines_event\0".as_ptr() as *const ::core::ffi::c_char,
            args,
        ) {
            badchannelid = channelid;
        }
        i = i.wrapping_add(1);
    }
    if badchannelid != 0 as uint64_t {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"buf_updates_send_changes\0".as_ptr() as *const ::core::ffi::c_char,
            258 as ::core::ffi::c_int,
            true_0 != 0,
            b"Disabling buffer updates for dead channel %lu\0".as_ptr()
                as *const ::core::ffi::c_char,
            badchannelid,
        );
        buf_updates_unregister(buf, badchannelid);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    let mut j: size_t = 0 as size_t;
    let mut i_0: size_t = 0 as size_t;
    while i_0 < (*buf).update_callbacks.size {
        let mut cb: BufUpdateCallbacks = *(*buf).update_callbacks.items.offset(i_0 as isize);
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
            *args_0.items.offset(c2rust_fresh19 as isize) = object {
                type_0: kObjectTypeBuffer,
                data: C2Rust_Unnamed {
                    integer: (*buf).handle as Integer,
                },
            };
            let c2rust_fresh20 = args_0.size;
            args_0.size = args_0.size.wrapping_add(1);
            *args_0.items.offset(c2rust_fresh20 as isize) = if send_tick as ::core::ffi::c_int != 0
            {
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
            *args_0.items.offset(c2rust_fresh21 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (firstline - 1 as linenr_T) as Integer,
                },
            };
            let c2rust_fresh22 = args_0.size;
            args_0.size = args_0.size.wrapping_add(1);
            *args_0.items.offset(c2rust_fresh22 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (firstline - 1 as linenr_T) as int64_t + num_removed,
                },
            };
            let c2rust_fresh23 = args_0.size;
            args_0.size = args_0.size.wrapping_add(1);
            *args_0.items.offset(c2rust_fresh23 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (firstline - 1 as linenr_T) as int64_t + num_added,
                },
            };
            let c2rust_fresh24 = args_0.size;
            args_0.size = args_0.size.wrapping_add(1);
            *args_0.items.offset(c2rust_fresh24 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: deleted_bytes as Integer,
                },
            };
            if cb.utf_sizes {
                let c2rust_fresh25 = args_0.size;
                args_0.size = args_0.size.wrapping_add(1);
                *args_0.items.offset(c2rust_fresh25 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: deleted_codepoints as Integer,
                    },
                };
                let c2rust_fresh26 = args_0.size;
                args_0.size = args_0.size.wrapping_add(1);
                *args_0.items.offset(c2rust_fresh26 as isize) = object {
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
                b"lines\0".as_ptr() as *const ::core::ffi::c_char,
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
            *(*buf)
                .update_callbacks
                .items
                .offset(c2rust_fresh27 as isize) =
                *(*buf).update_callbacks.items.offset(i_0 as isize);
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
        let mut cb: BufUpdateCallbacks = *(*buf).update_callbacks.items.offset(i as isize);
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
            *args.items.offset(c2rust_fresh28 as isize) = object {
                type_0: kObjectTypeBuffer,
                data: C2Rust_Unnamed {
                    integer: (*buf).handle as Integer,
                },
            };
            let c2rust_fresh29 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh29 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: buf_get_changedtick(buf),
                },
            };
            let c2rust_fresh30 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh30 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: start_row as Integer,
                },
            };
            let c2rust_fresh31 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh31 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: start_col as Integer,
                },
            };
            let c2rust_fresh32 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh32 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: start_byte as i64,
                },
            };
            let c2rust_fresh33 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh33 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: old_row as Integer,
                },
            };
            let c2rust_fresh34 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh34 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: old_col as Integer,
                },
            };
            let c2rust_fresh35 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh35 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: old_byte as i64,
                },
            };
            let c2rust_fresh36 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh36 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: new_row as Integer,
                },
            };
            let c2rust_fresh37 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh37 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: new_col as Integer,
                },
            };
            let c2rust_fresh38 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh38 as isize) = object {
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
                b"bytes\0".as_ptr() as *const ::core::ffi::c_char,
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
            *(*buf)
                .update_callbacks
                .items
                .offset(c2rust_fresh39 as isize) =
                *(*buf).update_callbacks.items.offset(i as isize);
        }
        i = i.wrapping_add(1);
    }
    (*buf).update_callbacks.size = j;
}
pub unsafe extern "C" fn buf_updates_changedtick(mut buf: *mut buf_T) {
    let mut i: size_t = 0 as size_t;
    while i < (*buf).update_channels.size {
        let mut channel_id: uint64_t = *(*buf).update_channels.items.offset(i as isize);
        buf_updates_changedtick_single(buf, channel_id);
        i = i.wrapping_add(1);
    }
    let mut j: size_t = 0 as size_t;
    let mut i_0: size_t = 0 as size_t;
    while i_0 < (*buf).update_callbacks.size {
        let mut cb: BufUpdateCallbacks = *(*buf).update_callbacks.items.offset(i_0 as isize);
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
            *args.items.offset(c2rust_fresh40 as isize) = object {
                type_0: kObjectTypeBuffer,
                data: C2Rust_Unnamed {
                    integer: (*buf).handle as Integer,
                },
            };
            let c2rust_fresh41 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh41 as isize) = object {
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
                b"changedtick\0".as_ptr() as *const ::core::ffi::c_char,
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
            *(*buf)
                .update_callbacks
                .items
                .offset(c2rust_fresh42 as isize) =
                *(*buf).update_callbacks.items.offset(i_0 as isize);
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
    *args.items.offset(c2rust_fresh8 as isize) = object {
        type_0: kObjectTypeBuffer,
        data: C2Rust_Unnamed {
            integer: (*buf).handle as Integer,
        },
    };
    let c2rust_fresh9 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh9 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: buf_get_changedtick(buf),
        },
    };
    rpc_send_event(
        channel_id,
        b"nvim_buf_changedtick_event\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
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
