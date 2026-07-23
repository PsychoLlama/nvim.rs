use crate::src::nvim::api::autocmd::{
    nvim_clear_autocmds, nvim_create_augroup, nvim_create_autocmd, nvim_del_augroup_by_id,
    nvim_del_augroup_by_name, nvim_del_autocmd, nvim_exec_autocmds, nvim_get_autocmds,
};
use crate::src::nvim::api::buffer::{
    nvim__buf_stats, nvim_buf_attach, nvim_buf_call, nvim_buf_del_keymap, nvim_buf_del_mark,
    nvim_buf_del_var, nvim_buf_delete, nvim_buf_get_changedtick, nvim_buf_get_keymap,
    nvim_buf_get_lines, nvim_buf_get_mark, nvim_buf_get_name, nvim_buf_get_offset,
    nvim_buf_get_text, nvim_buf_get_var, nvim_buf_is_loaded, nvim_buf_is_valid,
    nvim_buf_line_count, nvim_buf_set_keymap, nvim_buf_set_lines, nvim_buf_set_mark,
    nvim_buf_set_name, nvim_buf_set_text, nvim_buf_set_var,
};
use crate::src::nvim::api::command::{
    nvim_buf_create_user_command, nvim_buf_del_user_command, nvim_buf_get_commands, nvim_cmd,
    nvim_create_user_command, nvim_del_user_command, nvim_get_commands, nvim_parse_cmd,
};
use crate::src::nvim::api::deprecated::{
    nvim_buf_add_highlight, nvim_buf_clear_highlight, nvim_buf_get_number, nvim_buf_get_option,
    nvim_buf_set_option, nvim_buf_set_virtual_text, nvim_command_output, nvim_err_write,
    nvim_err_writeln, nvim_exec, nvim_get_hl_by_id, nvim_get_hl_by_name, nvim_get_option,
    nvim_get_option_info, nvim_notify, nvim_out_write, nvim_set_option, nvim_win_get_option,
    nvim_win_set_option,
};
use crate::src::nvim::api::extmark::{
    nvim__buf_debug_extmarks, nvim__ns_get, nvim__ns_set, nvim_buf_clear_namespace,
    nvim_buf_del_extmark, nvim_buf_get_extmark_by_id, nvim_buf_get_extmarks, nvim_buf_set_extmark,
    nvim_create_namespace, nvim_get_namespaces, nvim_set_decoration_provider,
};
use crate::src::nvim::api::options::{
    nvim_get_all_options_info, nvim_get_option_info2, nvim_get_option_value, nvim_set_option_value,
};
use crate::src::nvim::api::private::dispatch::{
    buf_attach_table, buf_delete_table, clear_autocmds_table, cmd_opts_table, cmd_table,
    complete_set_table, context_table, create_augroup_table, create_autocmd_table, echo_opts_table,
    empty_table, eval_statusline_table, exec_autocmds_table, exec_opts_table, get_autocmds_table,
    get_commands_table, get_extmark_table, get_extmarks_table, get_highlight_table, get_ns_table,
    highlight_table, keymap_table, ns_opts_table, open_term_table, option_table, redraw_table,
    runtime_table, set_decoration_provider_table, set_extmark_table, tabpage_config_table,
    user_command_table, win_config_table, win_text_height_table, KeyDict_buf_attach_get_field,
    KeyDict_buf_delete_get_field, KeyDict_clear_autocmds_get_field, KeyDict_cmd_get_field,
    KeyDict_cmd_opts_get_field, KeyDict_complete_set_get_field, KeyDict_context_get_field,
    KeyDict_create_augroup_get_field, KeyDict_create_autocmd_get_field,
    KeyDict_echo_opts_get_field, KeyDict_empty_get_field, KeyDict_eval_statusline_get_field,
    KeyDict_exec_autocmds_get_field, KeyDict_exec_opts_get_field, KeyDict_get_autocmds_get_field,
    KeyDict_get_commands_get_field, KeyDict_get_extmark_get_field, KeyDict_get_extmarks_get_field,
    KeyDict_get_highlight_get_field, KeyDict_get_ns_get_field, KeyDict_highlight_get_field,
    KeyDict_keymap_get_field, KeyDict_ns_opts_get_field, KeyDict_open_term_get_field,
    KeyDict_option_get_field, KeyDict_redraw_get_field, KeyDict_runtime_get_field,
    KeyDict_set_decoration_provider_get_field, KeyDict_set_extmark_get_field,
    KeyDict_tabpage_config_get_field, KeyDict_user_command_get_field, KeyDict_win_config_get_field,
    KeyDict_win_text_height_get_field,
};
use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_free_dict, api_free_object, api_free_string, api_luarefs_free_keydict,
    api_luarefs_free_object, api_set_error,
};
use crate::src::nvim::api::tabpage::{
    nvim_open_tabpage, nvim_tabpage_del_var, nvim_tabpage_get_number, nvim_tabpage_get_var,
    nvim_tabpage_get_win, nvim_tabpage_is_valid, nvim_tabpage_list_wins, nvim_tabpage_set_var,
    nvim_tabpage_set_win,
};
use crate::src::nvim::api::ui::nvim_ui_send;
use crate::src::nvim::api::vim::{
    nvim__complete_set, nvim__get_lib_dir, nvim__get_runtime, nvim__id, nvim__id_array,
    nvim__id_dict, nvim__id_float, nvim__inspect_cell, nvim__invalidate_glyph_cache, nvim__redraw,
    nvim__runtime_inspect, nvim__screenshot, nvim__stats, nvim__unpack, nvim_chan_send,
    nvim_create_buf, nvim_del_current_line, nvim_del_keymap, nvim_del_mark, nvim_del_var,
    nvim_echo, nvim_eval_statusline, nvim_feedkeys, nvim_get_chan_info, nvim_get_color_by_name,
    nvim_get_color_map, nvim_get_context, nvim_get_current_buf, nvim_get_current_line,
    nvim_get_current_tabpage, nvim_get_current_win, nvim_get_hl, nvim_get_hl_id_by_name,
    nvim_get_hl_ns, nvim_get_keymap, nvim_get_mark, nvim_get_mode, nvim_get_proc,
    nvim_get_proc_children, nvim_get_runtime_file, nvim_get_var, nvim_get_vvar, nvim_input,
    nvim_input_mouse, nvim_list_bufs, nvim_list_chans, nvim_list_runtime_paths, nvim_list_tabpages,
    nvim_list_uis, nvim_list_wins, nvim_load_context, nvim_open_term, nvim_paste, nvim_put,
    nvim_replace_termcodes, nvim_select_popupmenu_item, nvim_set_current_buf, nvim_set_current_dir,
    nvim_set_current_line, nvim_set_current_tabpage, nvim_set_current_win, nvim_set_hl,
    nvim_set_hl_ns, nvim_set_hl_ns_fast, nvim_set_keymap, nvim_set_var, nvim_set_vvar,
    nvim_strwidth,
};
use crate::src::nvim::api::vimscript::{
    nvim_call_dict_function, nvim_call_function, nvim_command, nvim_eval, nvim_exec2,
    nvim_parse_expression,
};
use crate::src::nvim::api::win_config::{nvim_open_win, nvim_win_get_config, nvim_win_set_config};
use crate::src::nvim::api::window::{
    nvim_win_call, nvim_win_close, nvim_win_del_var, nvim_win_get_buf, nvim_win_get_cursor,
    nvim_win_get_height, nvim_win_get_number, nvim_win_get_position, nvim_win_get_tabpage,
    nvim_win_get_var, nvim_win_get_width, nvim_win_hide, nvim_win_is_valid, nvim_win_set_buf,
    nvim_win_set_cursor, nvim_win_set_height, nvim_win_set_hl_ns, nvim_win_set_var,
    nvim_win_set_width, nvim_win_text_height,
};
use crate::src::nvim::ex_docmd::expr_map_locked;
use crate::src::nvim::ex_getln::{get_text_locked_msg, text_locked};

use crate::src::nvim::lua::converter::{
    nlua_pop_Array, nlua_pop_Boolean, nlua_pop_Dict, nlua_pop_Float, nlua_pop_Integer,
    nlua_pop_LuaRef, nlua_pop_Object, nlua_pop_String, nlua_pop_handle, nlua_pop_keydict,
    nlua_push_Array, nlua_push_Boolean, nlua_push_Dict, nlua_push_Float, nlua_push_Integer,
    nlua_push_Object, nlua_push_String, nlua_push_handle, nlua_push_keydict,
};
use crate::src::nvim::lua::executor::{active_lstate, api_free_luaref, nlua_is_deferred_safe};
use crate::src::nvim::lua::ffi::{
    luaL_error, luaL_where, lua_concat, lua_createtable, lua_error, lua_gettop, lua_pushcclosure,
    lua_pushstring, lua_setfield,
};
use crate::src::nvim::main::{e_fast_api_disabled, e_textlock, textlock};
use crate::src::nvim::memory::{arena_finish, arena_mem_free, ARENA_EMPTY};
pub use crate::src::nvim::types::{
    consumed_blk, handle_T, int64_t, key_value_pair, lua_CFunction, lua_State, object,
    object_data as C2Rust_Unnamed, size_t, uint64_t, Arena, ArenaMem, Array, Boolean, Buffer, Dict,
    Error, ErrorType, FieldHashfn, Float, HLGroupID, Integer, KeyDict_buf_attach,
    KeyDict_buf_delete, KeyDict_clear_autocmds, KeyDict_cmd, KeyDict_cmd_opts,
    KeyDict_complete_set, KeyDict_context, KeyDict_create_augroup, KeyDict_create_autocmd,
    KeyDict_echo_opts, KeyDict_empty, KeyDict_eval_statusline, KeyDict_exec_autocmds,
    KeyDict_exec_opts, KeyDict_get_autocmds, KeyDict_get_commands, KeyDict_get_extmark,
    KeyDict_get_extmarks, KeyDict_get_highlight, KeyDict_get_ns, KeyDict_highlight, KeyDict_keymap,
    KeyDict_ns_opts, KeyDict_open_term, KeyDict_option, KeyDict_redraw, KeyDict_runtime,
    KeyDict_set_decoration_provider, KeyDict_set_extmark, KeyDict_tabpage_config,
    KeyDict_user_command, KeyDict_win_config, KeyDict_win_text_height, KeySetLink, KeyValuePair,
    LuaRef, Object, ObjectType, OptionalKeys, String_0, Tabpage, Window,
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
pub type C2Rust_Unnamed_0 = ::core::ffi::c_uint;
pub const kNluaPushFreeRefs: C2Rust_Unnamed_0 = 2;
pub const kNluaPushSpecial: C2Rust_Unnamed_0 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const INTERNAL_CALL_MASK: uint64_t = (1 as ::core::ffi::c_int as uint64_t)
    << ::core::mem::size_of::<uint64_t>()
        .wrapping_mul(8 as usize)
        .wrapping_sub(1 as usize);
pub const VIML_INTERNAL_CALL: uint64_t = INTERNAL_CALL_MASK;
pub const LUA_INTERNAL_CALL: uint64_t = VIML_INTERNAL_CALL.wrapping_add(1 as uint64_t);
unsafe extern "C" fn nlua_api_nvim_get_autocmds(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut arg1: KeyDict_get_autocmds = KeyDict_get_autocmds {
        is_set__get_autocmds_: 0,
        event: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        group: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        pattern: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        buffer: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        buf: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        id: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_autocmds\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = KeyDict_get_autocmds {
            is_set__get_autocmds_: 0 as OptionalKeys,
            event: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            group: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            pattern: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            buffer: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            buf: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            id: 0,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg1 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_get_autocmds_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_autocmds(&raw mut arg1, &raw mut arena, &raw mut err);
            nlua_push_Array(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
        api_luarefs_free_keydict(
            &raw mut arg1 as *mut ::core::ffi::c_void,
            get_autocmds_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_create_autocmd(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut arg2: KeyDict_create_autocmd = KeyDict_create_autocmd {
        is_set__create_autocmd_: 0,
        buffer: 0,
        buf: 0,
        callback: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        command: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        desc: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        group: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        nested: false,
        once: false,
        pattern: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_create_autocmd\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_create_autocmd {
            is_set__create_autocmd_: 0 as OptionalKeys,
            buffer: 0,
            buf: 0,
            callback: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            command: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            desc: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            group: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            nested: false,
            once: false,
            pattern: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_create_autocmd_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"event\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_create_autocmd(
                    LUA_INTERNAL_CALL,
                    arg1,
                    &raw mut arg2,
                    &raw mut arena,
                    &raw mut err,
                );
                nlua_push_Integer(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
                api_luarefs_free_object(arg1);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            create_autocmd_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_del_autocmd(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_del_autocmd\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_del_autocmd(arg1, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_clear_autocmds(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg1: KeyDict_clear_autocmds = KeyDict_clear_autocmds {
        is_set__clear_autocmds_: 0,
        buffer: 0,
        buf: 0,
        event: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        group: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        pattern: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_clear_autocmds\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = KeyDict_clear_autocmds {
            is_set__clear_autocmds_: 0 as OptionalKeys,
            buffer: 0,
            buf: 0,
            event: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            group: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            pattern: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg1 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_clear_autocmds_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_clear_autocmds(&raw mut arg1, &raw mut arena, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
        api_luarefs_free_keydict(
            &raw mut arg1 as *mut ::core::ffi::c_void,
            clear_autocmds_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_create_augroup(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut arg2: KeyDict_create_augroup = KeyDict_create_augroup {
        is_set__create_augroup_: 0,
        clear: false,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_create_augroup\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_create_augroup {
            is_set__create_augroup_: 0 as OptionalKeys,
            clear: false,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_create_augroup_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_create_augroup(LUA_INTERNAL_CALL, arg1, &raw mut arg2, &raw mut err);
                nlua_push_Integer(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            create_augroup_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_del_augroup_by_id(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_del_augroup_by_id\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_del_augroup_by_id(arg1, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_del_augroup_by_name(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_del_augroup_by_name\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_del_augroup_by_name(arg1, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_exec_autocmds(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg2: KeyDict_exec_autocmds = KeyDict_exec_autocmds {
        is_set__exec_autocmds_: 0,
        buffer: 0,
        buf: 0,
        group: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        modeline: false,
        pattern: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        data: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_exec_autocmds\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_exec_autocmds {
            is_set__exec_autocmds_: 0 as OptionalKeys,
            buffer: 0,
            buf: 0,
            group: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            modeline: false,
            pattern: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            data: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_exec_autocmds_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"event\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_exec_autocmds(arg1, &raw mut arg2, &raw mut arena, &raw mut err);
                active_lstate.set(save_active_lstate);
                api_luarefs_free_object(arg1);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            exec_autocmds_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_line_count(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_line_count\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_buf_line_count(arg1, &raw mut err);
            nlua_push_Integer(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_attach(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Boolean = false;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Boolean = false;
    let mut arg3: KeyDict_buf_attach = KeyDict_buf_attach {
        is_set__buf_attach_: 0,
        on_lines: 0,
        on_bytes: 0,
        on_changedtick: 0,
        on_detach: 0,
        on_reload: 0,
        utf_sizes: false,
        preview: false,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_attach\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = KeyDict_buf_attach {
            is_set__buf_attach_: 0 as OptionalKeys,
            on_lines: 0,
            on_bytes: 0,
            on_changedtick: 0,
            on_detach: 0,
            on_reload: 0,
            utf_sizes: false,
            preview: false,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg3 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_buf_attach_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param = b"send_buffer\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    ret =
                        nvim_buf_attach(LUA_INTERNAL_CALL, arg1, arg2, &raw mut arg3, &raw mut err);
                    nlua_push_Boolean(
                        lstate,
                        ret,
                        kNluaPushSpecial as ::core::ffi::c_int
                            | kNluaPushFreeRefs as ::core::ffi::c_int,
                    );
                    active_lstate.set(save_active_lstate);
                }
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg3 as *mut ::core::ffi::c_void,
            buf_attach_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_lines(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg4: Boolean = false;
    let mut arg3: Integer = 0;
    let mut arg2: Integer = 0;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 4 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 4 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_lines\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg4 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"strict_indexing\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        } else {
            arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"end\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"start\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        save_active_lstate = active_lstate.get();
                        active_lstate.set(lstate);
                        ret = nvim_buf_get_lines(
                            LUA_INTERNAL_CALL,
                            arg1,
                            arg2,
                            arg3,
                            arg4,
                            &raw mut arena,
                            lstate,
                            &raw mut err,
                        );
                        if lua_gettop(lstate) == 0 as ::core::ffi::c_int {
                            nlua_push_Array(
                                lstate,
                                ret,
                                kNluaPushSpecial as ::core::ffi::c_int
                                    | kNluaPushFreeRefs as ::core::ffi::c_int,
                            );
                        }
                        active_lstate.set(save_active_lstate);
                    }
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_set_lines(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg5: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut arg4: Boolean = false;
    let mut arg3: Integer = 0;
    let mut arg2: Integer = 0;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 5 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 5 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_set_lines\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if textlock.get() != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0
        {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
        } else {
            arg5 = nlua_pop_Array(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param = b"replacement\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else {
                arg4 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"strict_indexing\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"end\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                        {
                            err_param = b"start\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                            if err.type_0 as ::core::ffi::c_int
                                != kErrorTypeNone as ::core::ffi::c_int
                            {
                                err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char;
                            } else {
                                save_active_lstate = active_lstate.get();
                                active_lstate.set(lstate);
                                nvim_buf_set_lines(
                                    LUA_INTERNAL_CALL,
                                    arg1,
                                    arg2,
                                    arg3,
                                    arg4,
                                    arg5,
                                    &raw mut arena,
                                    &raw mut err,
                                );
                                active_lstate.set(save_active_lstate);
                            }
                        }
                    }
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_set_text(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg6: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut arg5: Integer = 0;
    let mut arg4: Integer = 0;
    let mut arg3: Integer = 0;
    let mut arg2: Integer = 0;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 6 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 6 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_set_text\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if textlock.get() != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0
        {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
        } else {
            arg6 = nlua_pop_Array(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param = b"replacement\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else {
                arg5 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"end_col\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg4 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"end_row\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                        {
                            err_param = b"start_col\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                            if err.type_0 as ::core::ffi::c_int
                                != kErrorTypeNone as ::core::ffi::c_int
                            {
                                err_param = b"start_row\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char;
                            } else {
                                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                                if err.type_0 as ::core::ffi::c_int
                                    != kErrorTypeNone as ::core::ffi::c_int
                                {
                                    err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char;
                                } else {
                                    save_active_lstate = active_lstate.get();
                                    active_lstate.set(lstate);
                                    nvim_buf_set_text(
                                        LUA_INTERNAL_CALL,
                                        arg1,
                                        arg2,
                                        arg3,
                                        arg4,
                                        arg5,
                                        arg6,
                                        &raw mut arena,
                                        &raw mut err,
                                    );
                                    active_lstate.set(save_active_lstate);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_text(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg5: Integer = 0;
    let mut arg4: Integer = 0;
    let mut arg3: Integer = 0;
    let mut arg2: Integer = 0;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut arg6: KeyDict_empty = KeyDict_empty { is_set__empty_: 0 };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 6 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 6 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_text\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg6 = KeyDict_empty {
            is_set__empty_: 0 as OptionalKeys,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg6 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_empty_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg5 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"end_col\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg4 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"end_row\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"start_col\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                        {
                            err_param = b"start_row\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                            if err.type_0 as ::core::ffi::c_int
                                != kErrorTypeNone as ::core::ffi::c_int
                            {
                                err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char;
                            } else {
                                save_active_lstate = active_lstate.get();
                                active_lstate.set(lstate);
                                ret = nvim_buf_get_text(
                                    LUA_INTERNAL_CALL,
                                    arg1,
                                    arg2,
                                    arg3,
                                    arg4,
                                    arg5,
                                    &raw mut arg6,
                                    &raw mut arena,
                                    lstate,
                                    &raw mut err,
                                );
                                if lua_gettop(lstate) == 0 as ::core::ffi::c_int {
                                    nlua_push_Array(
                                        lstate,
                                        ret,
                                        kNluaPushSpecial as ::core::ffi::c_int
                                            | kNluaPushFreeRefs as ::core::ffi::c_int,
                                    );
                                }
                                active_lstate.set(save_active_lstate);
                            }
                        }
                    }
                }
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg6 as *mut ::core::ffi::c_void,
            empty_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_offset(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg2: Integer = 0;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_offset\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"index\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_buf_get_offset(arg1, arg2, &raw mut err);
                nlua_push_Integer(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_var(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_var\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_buf_get_var(arg1, arg2, &raw mut arena, &raw mut err);
                nlua_push_Object(
                    lstate,
                    &raw mut ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_changedtick(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_changedtick\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_buf_get_changedtick(arg1, &raw mut err);
            nlua_push_Integer(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_keymap(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_keymap\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"mode\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_buf_get_keymap(arg1, arg2, &raw mut arena, &raw mut err);
                nlua_push_Array(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_set_keymap(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg4: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg3: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg5: KeyDict_keymap = KeyDict_keymap {
        is_set__keymap_: 0,
        noremap: false,
        nowait: false,
        silent: false,
        script: false,
        expr: false,
        unique: false,
        callback: 0,
        desc: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        replace_keycodes: false,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 5 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 5 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_set_keymap\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg5 = KeyDict_keymap {
            is_set__keymap_: 0 as OptionalKeys,
            noremap: false,
            nowait: false,
            silent: false,
            script: false,
            expr: false,
            unique: false,
            callback: 0,
            desc: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            replace_keycodes: false,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg5 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_keymap_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg4 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"rhs\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg3 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"lhs\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"mode\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                        {
                            err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            save_active_lstate = active_lstate.get();
                            active_lstate.set(lstate);
                            nvim_buf_set_keymap(
                                LUA_INTERNAL_CALL,
                                arg1,
                                arg2,
                                arg3,
                                arg4,
                                &raw mut arg5,
                                &raw mut err,
                            );
                            active_lstate.set(save_active_lstate);
                        }
                    }
                }
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg5 as *mut ::core::ffi::c_void,
            keymap_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_del_keymap(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg3: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_del_keymap\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"lhs\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"mode\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_buf_del_keymap(LUA_INTERNAL_CALL, arg1, arg2, arg3, &raw mut err);
                    active_lstate.set(save_active_lstate);
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_set_var(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg3: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_set_var\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"value\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_buf_set_var(arg1, arg2, arg3, &raw mut err);
                    active_lstate.set(save_active_lstate);
                }
            }
            api_luarefs_free_object(arg3);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_del_var(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_del_var\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_buf_del_var(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_name(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_name\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_buf_get_name(arg1, &raw mut err);
            nlua_push_String(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_set_name(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_set_name\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_buf_set_name(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_is_loaded(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Boolean = false;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_is_loaded\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_buf_is_loaded(arg1);
            nlua_push_Boolean(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_delete(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg2: KeyDict_buf_delete = KeyDict_buf_delete {
        is_set__buf_delete_: 0,
        force: false,
        unload: false,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_delete\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if text_locked() {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                get_text_locked_msg(),
            );
        } else {
            arg2 = KeyDict_buf_delete {
                is_set__buf_delete_: 0 as OptionalKeys,
                force: false,
                unload: false,
            };
            nlua_pop_keydict(
                lstate,
                &raw mut arg2 as *mut ::core::ffi::c_void,
                Some(
                    KeyDict_buf_delete_get_field
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            size_t,
                        ) -> *mut KeySetLink,
                ),
                &raw mut err_param,
                &raw mut arena,
                &raw mut err,
            );
            if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_buf_delete(arg1, &raw mut arg2, &raw mut err);
                    active_lstate.set(save_active_lstate);
                }
            }
            api_luarefs_free_keydict(
                &raw mut arg2 as *mut ::core::ffi::c_void,
                buf_delete_table.ptr() as *mut KeySetLink,
            );
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_is_valid(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Boolean = false;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_is_valid\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_buf_is_valid(arg1);
            nlua_push_Boolean(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_del_mark(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Boolean = false;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_del_mark\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_buf_del_mark(arg1, arg2, &raw mut err);
                nlua_push_Boolean(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_set_mark(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg4: Integer = 0;
    let mut arg3: Integer = 0;
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Boolean = false;
    let mut arg5: KeyDict_empty = KeyDict_empty { is_set__empty_: 0 };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 5 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 5 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_set_mark\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg5 = KeyDict_empty {
            is_set__empty_: 0 as OptionalKeys,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg5 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_empty_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg4 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"col\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"line\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"name\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                        {
                            err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            save_active_lstate = active_lstate.get();
                            active_lstate.set(lstate);
                            ret = nvim_buf_set_mark(
                                arg1,
                                arg2,
                                arg3,
                                arg4,
                                &raw mut arg5,
                                &raw mut err,
                            );
                            nlua_push_Boolean(
                                lstate,
                                ret,
                                kNluaPushSpecial as ::core::ffi::c_int
                                    | kNluaPushFreeRefs as ::core::ffi::c_int,
                            );
                            active_lstate.set(save_active_lstate);
                        }
                    }
                }
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg5 as *mut ::core::ffi::c_void,
            empty_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_mark(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_mark\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_buf_get_mark(arg1, arg2, &raw mut arena, &raw mut err);
                nlua_push_Array(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_call(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg2: LuaRef = 0;
    let mut arg1: Buffer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_call\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_LuaRef(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"fun\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_buf_call(arg1, arg2, &raw mut err);
                nlua_push_Object(
                    lstate,
                    &raw mut ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
            api_free_luaref(arg2);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__buf_stats(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__buf_stats\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim__buf_stats(arg1, &raw mut arena, &raw mut err);
            nlua_push_Dict(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_parse_cmd(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: KeyDict_cmd = KeyDict_cmd {
        is_set__cmd_: 0,
        cmd: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        range: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        count: 0,
        reg: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        bang: false,
        args: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        magic: Dict {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        },
        mods: Dict {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        },
        nargs: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        addr: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        nextcmd: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
    };
    let mut arg2: KeyDict_empty = KeyDict_empty { is_set__empty_: 0 };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        arg2 = KeyDict_empty {
            is_set__empty_: 0 as OptionalKeys,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_empty_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"str\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_parse_cmd(arg1, &raw mut arg2, &raw mut arena, &raw mut err);
                nlua_push_keydict(
                    lstate,
                    &raw mut ret as *mut ::core::ffi::c_void,
                    cmd_table.ptr() as *mut KeySetLink,
                );
                active_lstate.set(save_active_lstate);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            empty_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_cmd(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: KeyDict_cmd = KeyDict_cmd {
        is_set__cmd_: 0,
        cmd: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        range: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        count: 0,
        reg: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        bang: false,
        args: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        magic: Dict {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        },
        mods: Dict {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        },
        nargs: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        addr: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        nextcmd: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
    };
    let mut arg2: KeyDict_cmd_opts = KeyDict_cmd_opts { output: false };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_cmd\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_cmd_opts { output: false };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_cmd_opts_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = KeyDict_cmd {
                is_set__cmd_: 0 as OptionalKeys,
                cmd: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                },
                range: Array {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<Object>(),
                },
                count: 0,
                reg: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                },
                bang: false,
                args: Array {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<Object>(),
                },
                magic: Dict {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<KeyValuePair>(),
                },
                mods: Dict {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<KeyValuePair>(),
                },
                nargs: Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                },
                addr: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                },
                nextcmd: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                },
            };
            nlua_pop_keydict(
                lstate,
                &raw mut arg1 as *mut ::core::ffi::c_void,
                Some(
                    KeyDict_cmd_get_field
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            size_t,
                        ) -> *mut KeySetLink,
                ),
                &raw mut err_param,
                &raw mut arena,
                &raw mut err,
            );
            if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_cmd(
                    LUA_INTERNAL_CALL,
                    &raw mut arg1,
                    &raw mut arg2,
                    &raw mut arena,
                    &raw mut err,
                );
                nlua_push_String(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
            api_luarefs_free_keydict(
                &raw mut arg1 as *mut ::core::ffi::c_void,
                cmd_table.ptr() as *mut KeySetLink,
            );
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            cmd_opts_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_create_user_command(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg2: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg3: KeyDict_user_command = KeyDict_user_command {
        is_set__user_command_: 0,
        addr: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        bang: false,
        bar: false,
        complete: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        count: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        desc: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        force: false,
        keepscript: false,
        nargs: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        preview: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        range: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        register_: false,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_create_user_command\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = KeyDict_user_command {
            is_set__user_command_: 0 as OptionalKeys,
            addr: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            bang: false,
            bar: false,
            complete: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            count: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            desc: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            force: false,
            keepscript: false,
            nargs: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            preview: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            range: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            register_: false,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg3 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_user_command_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg2 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"cmd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"name\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_create_user_command(
                        LUA_INTERNAL_CALL,
                        arg1,
                        arg2,
                        &raw mut arg3,
                        &raw mut err,
                    );
                    active_lstate.set(save_active_lstate);
                }
                api_luarefs_free_object(arg2);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg3 as *mut ::core::ffi::c_void,
            user_command_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_del_user_command(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_del_user_command\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_del_user_command(arg1, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_create_user_command(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg3: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg4: KeyDict_user_command = KeyDict_user_command {
        is_set__user_command_: 0,
        addr: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        bang: false,
        bar: false,
        complete: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        count: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        desc: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        force: false,
        keepscript: false,
        nargs: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        preview: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        range: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        register_: false,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 4 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 4 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_create_user_command\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg4 = KeyDict_user_command {
            is_set__user_command_: 0 as OptionalKeys,
            addr: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            bang: false,
            bar: false,
            complete: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            count: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            desc: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            force: false,
            keepscript: false,
            nargs: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            preview: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            range: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            register_: false,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg4 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_user_command_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg3 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"cmd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"name\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        save_active_lstate = active_lstate.get();
                        active_lstate.set(lstate);
                        nvim_buf_create_user_command(
                            LUA_INTERNAL_CALL,
                            arg1,
                            arg2,
                            arg3,
                            &raw mut arg4,
                            &raw mut err,
                        );
                        active_lstate.set(save_active_lstate);
                    }
                }
                api_luarefs_free_object(arg3);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg4 as *mut ::core::ffi::c_void,
            user_command_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_del_user_command(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_del_user_command\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_buf_del_user_command(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_commands(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut arg1: KeyDict_get_commands = KeyDict_get_commands { builtin: false };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_commands\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = KeyDict_get_commands { builtin: false };
        nlua_pop_keydict(
            lstate,
            &raw mut arg1 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_get_commands_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_commands(&raw mut arg1, &raw mut arena, &raw mut err);
            nlua_push_Dict(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
        api_luarefs_free_keydict(
            &raw mut arg1 as *mut ::core::ffi::c_void,
            get_commands_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_commands(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut arg2: KeyDict_get_commands = KeyDict_get_commands { builtin: false };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_commands\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_get_commands { builtin: false };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_get_commands_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_buf_get_commands(arg1, &raw mut arg2, &raw mut arena, &raw mut err);
                nlua_push_Dict(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            get_commands_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_exec(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Boolean = false;
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_exec\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"output\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"src\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_exec(LUA_INTERNAL_CALL, arg1, arg2, &raw mut err);
                nlua_push_String(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
                api_free_string(ret);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_command_output(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_command_output\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"command\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_command_output(LUA_INTERNAL_CALL, arg1, &raw mut err);
            nlua_push_String(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
            api_free_string(ret);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_number(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_number\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"buffer\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_buf_get_number(arg1, &raw mut err);
            nlua_push_Integer(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_clear_highlight(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg4: Integer = 0;
    let mut arg3: Integer = 0;
    let mut arg2: Integer = 0;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 4 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 4 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_clear_highlight\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg4 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"line_end\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param = b"line_start\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else {
                arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"ns_id\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"buffer\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        save_active_lstate = active_lstate.get();
                        active_lstate.set(lstate);
                        nvim_buf_clear_highlight(arg1, arg2, arg3, arg4, &raw mut err);
                        active_lstate.set(save_active_lstate);
                    }
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_add_highlight(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg6: Integer = 0;
    let mut arg5: Integer = 0;
    let mut arg4: Integer = 0;
    let mut arg3: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg2: Integer = 0;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 6 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 6 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_add_highlight\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg6 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"col_end\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg5 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param = b"col_start\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else {
                arg4 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"line\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg3 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"hl_group\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                        {
                            err_param = b"ns_id\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                            if err.type_0 as ::core::ffi::c_int
                                != kErrorTypeNone as ::core::ffi::c_int
                            {
                                err_param = b"buffer\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char;
                            } else {
                                save_active_lstate = active_lstate.get();
                                active_lstate.set(lstate);
                                ret = nvim_buf_add_highlight(
                                    arg1,
                                    arg2,
                                    arg3,
                                    arg4,
                                    arg5,
                                    arg6,
                                    &raw mut err,
                                );
                                nlua_push_Integer(
                                    lstate,
                                    ret,
                                    kNluaPushSpecial as ::core::ffi::c_int
                                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                                );
                                active_lstate.set(save_active_lstate);
                            }
                        }
                    }
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_set_virtual_text(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg4: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut arg3: Integer = 0;
    let mut arg2: Integer = 0;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut arg5: KeyDict_empty = KeyDict_empty { is_set__empty_: 0 };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 5 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 5 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_set_virtual_text\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg5 = KeyDict_empty {
            is_set__empty_: 0 as OptionalKeys,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg5 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_empty_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg4 = nlua_pop_Array(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"chunks\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"line\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"src_id\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                        {
                            err_param = b"buffer\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            save_active_lstate = active_lstate.get();
                            active_lstate.set(lstate);
                            ret = nvim_buf_set_virtual_text(
                                arg1,
                                arg2,
                                arg3,
                                arg4,
                                &raw mut arg5,
                                &raw mut err,
                            );
                            nlua_push_Integer(
                                lstate,
                                ret,
                                kNluaPushSpecial as ::core::ffi::c_int
                                    | kNluaPushFreeRefs as ::core::ffi::c_int,
                            );
                            active_lstate.set(save_active_lstate);
                        }
                    }
                }
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg5 as *mut ::core::ffi::c_void,
            empty_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_hl_by_id(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Boolean = false;
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_hl_by_id\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"rgb\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"hl_id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_get_hl_by_id(arg1, arg2, &raw mut arena, &raw mut err);
                nlua_push_Dict(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_hl_by_name(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg2: Boolean = false;
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_hl_by_name\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"rgb\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_get_hl_by_name(arg1, arg2, &raw mut arena, &raw mut err);
                nlua_push_Dict(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_option_info(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_option_info\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_option_info(arg1, &raw mut arena, &raw mut err);
            nlua_push_Dict(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_option(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg2: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_option\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"value\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_set_option(LUA_INTERNAL_CALL, arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
            api_luarefs_free_object(arg2);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_option(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_option\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_option(arg1, &raw mut err);
            nlua_push_Object(
                lstate,
                &raw mut ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
            api_free_object(ret);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_option(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_option\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buffer\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_buf_get_option(arg1, arg2, &raw mut err);
                nlua_push_Object(
                    lstate,
                    &raw mut ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
                api_free_object(ret);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_set_option(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg3: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_set_option\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"value\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"buffer\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_buf_set_option(LUA_INTERNAL_CALL, arg1, arg2, arg3, &raw mut err);
                    active_lstate.set(save_active_lstate);
                }
            }
            api_luarefs_free_object(arg3);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_get_option(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_get_option\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"window\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_win_get_option(arg1, arg2, &raw mut err);
                nlua_push_Object(
                    lstate,
                    &raw mut ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
                api_free_object(ret);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_set_option(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg3: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_set_option\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"value\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"window\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_win_set_option(LUA_INTERNAL_CALL, arg1, arg2, arg3, &raw mut err);
                    active_lstate.set(save_active_lstate);
                }
            }
            api_luarefs_free_object(arg3);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_out_write(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_out_write\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"str\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_out_write(arg1);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_err_write(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_err_write\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"str\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_err_write(arg1);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_err_writeln(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_err_writeln\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"str\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_err_writeln(arg1);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_notify(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg3: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut arg2: Integer = 0;
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_notify\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = nlua_pop_Dict(lstate, false_0 != 0, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"opts\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param = b"log_level\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"msg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    ret = nvim_notify(arg1, arg2, arg3, &raw mut arena, &raw mut err);
                    nlua_push_Object(
                        lstate,
                        &raw mut ret,
                        kNluaPushSpecial as ::core::ffi::c_int
                            | kNluaPushFreeRefs as ::core::ffi::c_int,
                    );
                    active_lstate.set(save_active_lstate);
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_create_namespace(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_create_namespace\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_create_namespace(arg1);
            nlua_push_Integer(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_namespaces(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_namespaces\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_get_namespaces(&raw mut arena);
        nlua_push_Dict(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_extmark_by_id(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg3: Integer = 0;
    let mut arg2: Integer = 0;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut arg4: KeyDict_get_extmark = KeyDict_get_extmark {
        is_set__get_extmark_: 0,
        details: false,
        hl_name: false,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 4 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 4 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_extmark_by_id\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg4 = KeyDict_get_extmark {
            is_set__get_extmark_: 0 as OptionalKeys,
            details: false,
            hl_name: false,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg4 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_get_extmark_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"ns_id\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        save_active_lstate = active_lstate.get();
                        active_lstate.set(lstate);
                        ret = nvim_buf_get_extmark_by_id(
                            arg1,
                            arg2,
                            arg3,
                            &raw mut arg4,
                            &raw mut arena,
                            &raw mut err,
                        );
                        nlua_push_Array(
                            lstate,
                            ret,
                            kNluaPushSpecial as ::core::ffi::c_int
                                | kNluaPushFreeRefs as ::core::ffi::c_int,
                        );
                        active_lstate.set(save_active_lstate);
                    }
                }
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg4 as *mut ::core::ffi::c_void,
            get_extmark_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_extmarks(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut arg2: Integer = 0;
    let mut arg4: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg3: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg5: KeyDict_get_extmarks = KeyDict_get_extmarks {
        is_set__get_extmarks_: 0,
        limit: 0,
        details: false,
        hl_name: false,
        overlap: false,
        type_0: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 5 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 5 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_extmarks\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg5 = KeyDict_get_extmarks {
            is_set__get_extmarks_: 0 as OptionalKeys,
            limit: 0,
            details: false,
            hl_name: false,
            overlap: false,
            type_0: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg5 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_get_extmarks_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg4 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"end\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg3 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"start\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"ns_id\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                        {
                            err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            save_active_lstate = active_lstate.get();
                            active_lstate.set(lstate);
                            ret = nvim_buf_get_extmarks(
                                arg1,
                                arg2,
                                arg3,
                                arg4,
                                &raw mut arg5,
                                &raw mut arena,
                                &raw mut err,
                            );
                            nlua_push_Array(
                                lstate,
                                ret,
                                kNluaPushSpecial as ::core::ffi::c_int
                                    | kNluaPushFreeRefs as ::core::ffi::c_int,
                            );
                            active_lstate.set(save_active_lstate);
                        }
                    }
                    api_luarefs_free_object(arg3);
                }
                api_luarefs_free_object(arg4);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg5 as *mut ::core::ffi::c_void,
            get_extmarks_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_set_extmark(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg4: Integer = 0;
    let mut arg3: Integer = 0;
    let mut arg2: Integer = 0;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut arg5: KeyDict_set_extmark = KeyDict_set_extmark {
        is_set__set_extmark_: 0,
        id: 0,
        end_line: 0,
        end_row: 0,
        end_col: 0,
        hl_group: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        virt_text: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        virt_text_pos: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        virt_text_win_col: 0,
        virt_text_hide: false,
        virt_text_repeat_linebreak: false,
        hl_eol: false,
        hl_mode: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        invalidate: false,
        ephemeral: false,
        priority: 0,
        right_gravity: false,
        end_right_gravity: false,
        virt_lines: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        virt_lines_above: false,
        virt_lines_leftcol: false,
        virt_lines_overflow: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        strict: false,
        sign_text: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        sign_hl_group: 0,
        number_hl_group: 0,
        line_hl_group: 0,
        cursorline_hl_group: 0,
        conceal: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        conceal_lines: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        spell: false,
        ui_watched: false,
        undo_restore: false,
        url: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        scoped: false,
        _subpriority: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 5 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 5 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_set_extmark\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg5 = KeyDict_set_extmark {
            is_set__set_extmark_: 0 as OptionalKeys,
            id: 0,
            end_line: 0,
            end_row: 0,
            end_col: 0,
            hl_group: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            virt_text: Array {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<Object>(),
            },
            virt_text_pos: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            virt_text_win_col: 0,
            virt_text_hide: false,
            virt_text_repeat_linebreak: false,
            hl_eol: false,
            hl_mode: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            invalidate: false,
            ephemeral: false,
            priority: 0,
            right_gravity: false,
            end_right_gravity: false,
            virt_lines: Array {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<Object>(),
            },
            virt_lines_above: false,
            virt_lines_leftcol: false,
            virt_lines_overflow: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            strict: false,
            sign_text: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            sign_hl_group: 0,
            number_hl_group: 0,
            line_hl_group: 0,
            cursorline_hl_group: 0,
            conceal: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            conceal_lines: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            spell: false,
            ui_watched: false,
            undo_restore: false,
            url: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            scoped: false,
            _subpriority: 0,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg5 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_set_extmark_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg4 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"col\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"line\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"ns_id\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                        {
                            err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            save_active_lstate = active_lstate.get();
                            active_lstate.set(lstate);
                            ret = nvim_buf_set_extmark(
                                arg1,
                                arg2,
                                arg3,
                                arg4,
                                &raw mut arg5,
                                &raw mut err,
                            );
                            nlua_push_Integer(
                                lstate,
                                ret,
                                kNluaPushSpecial as ::core::ffi::c_int
                                    | kNluaPushFreeRefs as ::core::ffi::c_int,
                            );
                            active_lstate.set(save_active_lstate);
                        }
                    }
                }
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg5 as *mut ::core::ffi::c_void,
            set_extmark_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_del_extmark(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg3: Integer = 0;
    let mut arg2: Integer = 0;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Boolean = false;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_del_extmark\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"ns_id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    ret = nvim_buf_del_extmark(arg1, arg2, arg3, &raw mut err);
                    nlua_push_Boolean(
                        lstate,
                        ret,
                        kNluaPushSpecial as ::core::ffi::c_int
                            | kNluaPushFreeRefs as ::core::ffi::c_int,
                    );
                    active_lstate.set(save_active_lstate);
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_clear_namespace(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg4: Integer = 0;
    let mut arg3: Integer = 0;
    let mut arg2: Integer = 0;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 4 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 4 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_clear_namespace\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg4 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"line_end\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param = b"line_start\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else {
                arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"ns_id\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        save_active_lstate = active_lstate.get();
                        active_lstate.set(lstate);
                        nvim_buf_clear_namespace(arg1, arg2, arg3, arg4, &raw mut err);
                        active_lstate.set(save_active_lstate);
                    }
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_decoration_provider(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg2: KeyDict_set_decoration_provider = KeyDict_set_decoration_provider {
        is_set__set_decoration_provider_: 0,
        on_start: 0,
        on_buf: 0,
        on_win: 0,
        on_line: 0,
        on_range: 0,
        on_end: 0,
        _on_hl_def: 0,
        _on_spell_nav: 0,
        _on_conceal_line: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_decoration_provider\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_set_decoration_provider {
            is_set__set_decoration_provider_: 0 as OptionalKeys,
            on_start: 0,
            on_buf: 0,
            on_win: 0,
            on_line: 0,
            on_range: 0,
            on_end: 0,
            _on_hl_def: 0,
            _on_spell_nav: 0,
            _on_conceal_line: 0,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_set_decoration_provider_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"ns_id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_set_decoration_provider(arg1, &raw mut arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            set_decoration_provider_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__buf_debug_extmarks(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg3: Boolean = false;
    let mut arg2: Boolean = false;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__buf_debug_extmarks\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"dot\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"keys\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    ret = nvim__buf_debug_extmarks(arg1, arg2, arg3, &raw mut err);
                    nlua_push_String(
                        lstate,
                        ret,
                        kNluaPushSpecial as ::core::ffi::c_int
                            | kNluaPushFreeRefs as ::core::ffi::c_int,
                    );
                    active_lstate.set(save_active_lstate);
                    api_free_string(ret);
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__ns_set(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg2: KeyDict_ns_opts = KeyDict_ns_opts {
        is_set__ns_opts_: 0,
        wins: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__ns_set\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_ns_opts {
            is_set__ns_opts_: 0 as OptionalKeys,
            wins: Array {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<Object>(),
            },
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_ns_opts_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"ns_id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim__ns_set(arg1, &raw mut arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            ns_opts_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__ns_get(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: KeyDict_ns_opts = KeyDict_ns_opts {
        is_set__ns_opts_: 0,
        wins: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__ns_get\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"ns_id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim__ns_get(arg1, &raw mut arena, &raw mut err);
            nlua_push_keydict(
                lstate,
                &raw mut ret as *mut ::core::ffi::c_void,
                ns_opts_table.ptr() as *mut KeySetLink,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_option_value(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg2: KeyDict_option = KeyDict_option {
        is_set__option_: 0,
        scope: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        win: 0,
        buf: 0,
        filetype: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_option_value\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_option {
            is_set__option_: 0 as OptionalKeys,
            scope: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            win: 0,
            buf: 0,
            filetype: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_option_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_get_option_value(arg1, &raw mut arg2, &raw mut err);
                nlua_push_Object(
                    lstate,
                    &raw mut ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
                api_free_object(ret);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            option_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_option_value(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg2: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg3: KeyDict_option = KeyDict_option {
        is_set__option_: 0,
        scope: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        win: 0,
        buf: 0,
        filetype: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_option_value\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = KeyDict_option {
            is_set__option_: 0 as OptionalKeys,
            scope: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            win: 0,
            buf: 0,
            filetype: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg3 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_option_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg2 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"value\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"name\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_set_option_value(
                        LUA_INTERNAL_CALL,
                        arg1,
                        arg2,
                        &raw mut arg3,
                        &raw mut err,
                    );
                    active_lstate.set(save_active_lstate);
                }
                api_luarefs_free_object(arg2);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg3 as *mut ::core::ffi::c_void,
            option_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_all_options_info(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_all_options_info\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_get_all_options_info(&raw mut arena, &raw mut err);
        nlua_push_Dict(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_option_info2(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut arg2: KeyDict_option = KeyDict_option {
        is_set__option_: 0,
        scope: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        win: 0,
        buf: 0,
        filetype: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_option_info2\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_option {
            is_set__option_: 0 as OptionalKeys,
            scope: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            win: 0,
            buf: 0,
            filetype: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_option_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_get_option_info2(arg1, &raw mut arg2, &raw mut arena, &raw mut err);
                nlua_push_Dict(
                    lstate,
                    ret,
                    0 as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            option_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_tabpage_list_wins(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Tabpage = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_tabpage_list_wins\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"tabpage\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_tabpage_list_wins(arg1, &raw mut arena, &raw mut err);
            nlua_push_Array(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_tabpage_get_var(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Tabpage = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_tabpage_get_var\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"tabpage\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_tabpage_get_var(arg1, arg2, &raw mut arena, &raw mut err);
                nlua_push_Object(
                    lstate,
                    &raw mut ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_tabpage_set_var(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Tabpage = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg3: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_tabpage_set_var\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"value\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"tabpage\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_tabpage_set_var(arg1, arg2, arg3, &raw mut err);
                    active_lstate.set(save_active_lstate);
                }
            }
            api_luarefs_free_object(arg3);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_tabpage_del_var(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Tabpage = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_tabpage_del_var\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"tabpage\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_tabpage_del_var(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_tabpage_get_win(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Tabpage = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Window = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_tabpage_get_win\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"tabpage\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_tabpage_get_win(arg1, &raw mut err);
            nlua_push_handle(
                lstate,
                ret as handle_T,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_tabpage_set_win(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg2: Window = 0;
    let mut arg1: Tabpage = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_tabpage_set_win\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"tabpage\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_tabpage_set_win(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_tabpage_get_number(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Tabpage = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_tabpage_get_number\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"tabpage\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_tabpage_get_number(arg1, &raw mut err);
            nlua_push_Integer(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_tabpage_is_valid(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Tabpage = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Boolean = false;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_tabpage_is_valid\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"tabpage\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_tabpage_is_valid(arg1);
            nlua_push_Boolean(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_open_tabpage(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Boolean = false;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Tabpage = 0;
    let mut arg3: KeyDict_tabpage_config = KeyDict_tabpage_config {
        is_set__tabpage_config_: 0,
        after: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_open_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if textlock.get() != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0
        {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
        } else {
            arg3 = KeyDict_tabpage_config {
                is_set__tabpage_config_: 0 as OptionalKeys,
                after: 0,
            };
            nlua_pop_keydict(
                lstate,
                &raw mut arg3 as *mut ::core::ffi::c_void,
                Some(
                    KeyDict_tabpage_config_get_field
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            size_t,
                        ) -> *mut KeySetLink,
                ),
                &raw mut err_param,
                &raw mut arena,
                &raw mut err,
            );
            if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
                arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"enter\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        save_active_lstate = active_lstate.get();
                        active_lstate.set(lstate);
                        ret = nvim_open_tabpage(arg1, arg2, &raw mut arg3, &raw mut err);
                        nlua_push_handle(
                            lstate,
                            ret as handle_T,
                            0 as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
                        );
                        active_lstate.set(save_active_lstate);
                    }
                }
            }
            api_luarefs_free_keydict(
                &raw mut arg3 as *mut ::core::ffi::c_void,
                tabpage_config_table.ptr() as *mut KeySetLink,
            );
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_ui_send(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_ui_send\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"content\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_ui_send(LUA_INTERNAL_CALL, arg1, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_hl_id_by_name(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_hl_id_by_name\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_hl_id_by_name(arg1);
            nlua_push_Integer(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_hl(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut arg2: KeyDict_get_highlight = KeyDict_get_highlight {
        is_set__get_highlight_: 0,
        id: 0,
        name: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        link: false,
        create: false,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_hl\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_get_highlight {
            is_set__get_highlight_: 0 as OptionalKeys,
            id: 0,
            name: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            link: false,
            create: false,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_get_highlight_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"ns_id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_get_hl(arg1, &raw mut arg2, &raw mut arena, &raw mut err);
                nlua_push_Dict(
                    lstate,
                    ret,
                    0 as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            get_highlight_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_hl(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg3: KeyDict_highlight = KeyDict_highlight {
        is_set__highlight_: 0,
        altfont: false,
        blink: false,
        bold: false,
        conceal: false,
        dim: false,
        italic: false,
        nocombine: false,
        overline: false,
        reverse: false,
        standout: false,
        strikethrough: false,
        undercurl: false,
        underdashed: false,
        underdotted: false,
        underdouble: false,
        underline: false,
        default_: false,
        cterm: Dict {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        },
        foreground: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        fg: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        background: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        bg: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        ctermfg: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        ctermbg: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        special: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        sp: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        link: 0,
        link_global: 0,
        fallback: false,
        blend: 0,
        fg_indexed: false,
        bg_indexed: false,
        force: false,
        update: false,
        url: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_hl\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = KeyDict_highlight {
            is_set__highlight_: 0 as OptionalKeys,
            altfont: false,
            blink: false,
            bold: false,
            conceal: false,
            dim: false,
            italic: false,
            nocombine: false,
            overline: false,
            reverse: false,
            standout: false,
            strikethrough: false,
            undercurl: false,
            underdashed: false,
            underdotted: false,
            underdouble: false,
            underline: false,
            default_: false,
            cterm: Dict {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<KeyValuePair>(),
            },
            foreground: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            fg: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            background: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            bg: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            ctermfg: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            ctermbg: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            special: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            sp: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            link: 0,
            link_global: 0,
            fallback: false,
            blend: 0,
            fg_indexed: false,
            bg_indexed: false,
            force: false,
            update: false,
            url: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg3 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_highlight_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"ns_id\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_set_hl(LUA_INTERNAL_CALL, arg1, arg2, &raw mut arg3, &raw mut err);
                    active_lstate.set(save_active_lstate);
                }
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg3 as *mut ::core::ffi::c_void,
            highlight_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_hl_ns(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut arg1: KeyDict_get_ns = KeyDict_get_ns {
        is_set__get_ns_: 0,
        winid: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_hl_ns\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = KeyDict_get_ns {
            is_set__get_ns_: 0 as OptionalKeys,
            winid: 0,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg1 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_get_ns_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_hl_ns(&raw mut arg1, &raw mut err);
            nlua_push_Integer(
                lstate,
                ret,
                0 as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
        api_luarefs_free_keydict(
            &raw mut arg1 as *mut ::core::ffi::c_void,
            get_ns_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_hl_ns(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_hl_ns\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"ns_id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_set_hl_ns(arg1, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_hl_ns_fast(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"ns_id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_set_hl_ns_fast(arg1, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_feedkeys(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg3: Boolean = false;
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_feedkeys\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"escape_ks\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"mode\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"keys\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_feedkeys(arg1, arg2, arg3);
                    active_lstate.set(save_active_lstate);
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_input(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"keys\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_input(LUA_INTERNAL_CALL, arg1);
            nlua_push_Integer(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_input_mouse(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg6: Integer = 0;
    let mut arg5: Integer = 0;
    let mut arg4: Integer = 0;
    let mut arg3: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 6 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 6 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        arg6 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"col\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg5 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"row\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg4 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"grid\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg3 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"modifier\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                        {
                            err_param = b"action\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                            if err.type_0 as ::core::ffi::c_int
                                != kErrorTypeNone as ::core::ffi::c_int
                            {
                                err_param = b"button\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char;
                            } else {
                                save_active_lstate = active_lstate.get();
                                active_lstate.set(lstate);
                                nvim_input_mouse(arg1, arg2, arg3, arg4, arg5, arg6, &raw mut err);
                                active_lstate.set(save_active_lstate);
                            }
                        }
                    }
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_replace_termcodes(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg4: Boolean = false;
    let mut arg3: Boolean = false;
    let mut arg2: Boolean = false;
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 4 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 4 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_replace_termcodes\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg4 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"special\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg3 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"do_lt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"from_part\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"str\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        save_active_lstate = active_lstate.get();
                        active_lstate.set(lstate);
                        ret = nvim_replace_termcodes(arg1, arg2, arg3, arg4);
                        nlua_push_String(
                            lstate,
                            ret,
                            kNluaPushSpecial as ::core::ffi::c_int
                                | kNluaPushFreeRefs as ::core::ffi::c_int,
                        );
                        active_lstate.set(save_active_lstate);
                        api_free_string(ret);
                    }
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_strwidth(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_strwidth\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"text\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_strwidth(arg1, &raw mut err);
            nlua_push_Integer(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_list_runtime_paths(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_list_runtime_paths\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_list_runtime_paths(&raw mut arena, &raw mut err);
        nlua_push_Array(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__runtime_inspect(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__runtime_inspect\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim__runtime_inspect(&raw mut arena);
        nlua_push_Array(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_runtime_file(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg2: Boolean = false;
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"all\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_get_runtime_file(arg1, arg2, &raw mut arena, &raw mut err);
                nlua_push_Array(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__get_lib_dir(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__get_lib_dir\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim__get_lib_dir();
        nlua_push_String(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
        api_free_string(ret);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__get_runtime(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Boolean = false;
    let mut arg1: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut arg3: KeyDict_runtime = KeyDict_runtime {
        is_lua: false,
        do_source: false,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        arg3 = KeyDict_runtime {
            is_lua: false,
            do_source: false,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg3 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_runtime_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"all\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_Array(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"pat\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    ret =
                        nvim__get_runtime(arg1, arg2, &raw mut arg3, &raw mut arena, &raw mut err);
                    nlua_push_Array(
                        lstate,
                        ret,
                        kNluaPushSpecial as ::core::ffi::c_int
                            | kNluaPushFreeRefs as ::core::ffi::c_int,
                    );
                    active_lstate.set(save_active_lstate);
                }
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg3 as *mut ::core::ffi::c_void,
            runtime_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_current_dir(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_current_dir\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"dir\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_set_current_dir(arg1, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_current_line(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_current_line\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_get_current_line(&raw mut arena, &raw mut err);
        nlua_push_String(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_current_line(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_current_line\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if textlock.get() != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0
        {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
        } else {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"line\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_set_current_line(arg1, &raw mut arena, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_del_current_line(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_del_current_line\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if textlock.get() != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0
        {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_del_current_line(&raw mut arena, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_var(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_var\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_var(arg1, &raw mut arena, &raw mut err);
            nlua_push_Object(
                lstate,
                &raw mut ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_var(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg2: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_var\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"value\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_set_var(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
            api_luarefs_free_object(arg2);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_del_var(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_del_var\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_del_var(arg1, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_vvar(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_vvar\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_vvar(arg1, &raw mut arena, &raw mut err);
            nlua_push_Object(
                lstate,
                &raw mut ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_vvar(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg2: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_vvar\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"value\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_set_vvar(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
            api_luarefs_free_object(arg2);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_echo(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Boolean = false;
    let mut arg1: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg3: KeyDict_echo_opts = KeyDict_echo_opts {
        is_set__echo_opts_: 0,
        err: false,
        verbose: false,
        _truncate: false,
        kind: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        id: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        title: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        status: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        percent: 0,
        source: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        data: Dict {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_echo\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = KeyDict_echo_opts {
            is_set__echo_opts_: 0 as OptionalKeys,
            err: false,
            verbose: false,
            _truncate: false,
            kind: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            id: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            title: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            status: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            percent: 0,
            source: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            data: Dict {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<KeyValuePair>(),
            },
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg3 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_echo_opts_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"history\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_Array(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"chunks\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    ret = nvim_echo(arg1, arg2, &raw mut arg3, &raw mut err);
                    nlua_push_Object(
                        lstate,
                        &raw mut ret,
                        kNluaPushSpecial as ::core::ffi::c_int
                            | kNluaPushFreeRefs as ::core::ffi::c_int,
                    );
                    active_lstate.set(save_active_lstate);
                }
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg3 as *mut ::core::ffi::c_void,
            echo_opts_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_list_bufs(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_list_bufs\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_list_bufs(&raw mut arena);
        nlua_push_Array(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_current_buf(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Buffer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_current_buf\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_get_current_buf();
        nlua_push_handle(
            lstate,
            ret as handle_T,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_current_buf(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_current_buf\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if text_locked() {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                get_text_locked_msg(),
            );
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_set_current_buf(arg1, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_list_wins(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_list_wins\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_list_wins(&raw mut arena);
        nlua_push_Array(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_current_win(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Window = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_current_win\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_get_current_win();
        nlua_push_handle(
            lstate,
            ret as handle_T,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_current_win(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_current_win\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if text_locked() {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                get_text_locked_msg(),
            );
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_set_current_win(arg1, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_create_buf(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Boolean = false;
    let mut arg1: Boolean = false;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Buffer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_create_buf\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"scratch\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"listed\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_create_buf(arg1, arg2, &raw mut err);
                nlua_push_handle(
                    lstate,
                    ret as handle_T,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_open_term(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut arg2: KeyDict_open_term = KeyDict_open_term {
        is_set__open_term_: 0,
        on_input: 0,
        force_crlf: false,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_open_term\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if textlock.get() != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0
        {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
        } else {
            arg2 = KeyDict_open_term {
                is_set__open_term_: 0 as OptionalKeys,
                on_input: 0,
                force_crlf: false,
            };
            nlua_pop_keydict(
                lstate,
                &raw mut arg2 as *mut ::core::ffi::c_void,
                Some(
                    KeyDict_open_term_get_field
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            size_t,
                        ) -> *mut KeySetLink,
                ),
                &raw mut err_param,
                &raw mut arena,
                &raw mut err,
            );
            if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    ret = nvim_open_term(arg1, &raw mut arg2, &raw mut err);
                    nlua_push_Integer(
                        lstate,
                        ret,
                        kNluaPushSpecial as ::core::ffi::c_int
                            | kNluaPushFreeRefs as ::core::ffi::c_int,
                    );
                    active_lstate.set(save_active_lstate);
                }
            }
            api_luarefs_free_keydict(
                &raw mut arg2 as *mut ::core::ffi::c_void,
                open_term_table.ptr() as *mut KeySetLink,
            );
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_chan_send(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_chan_send\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"data\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"chan\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_chan_send(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_list_tabpages(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_list_tabpages\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_list_tabpages(&raw mut arena);
        nlua_push_Array(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_current_tabpage(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Tabpage = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_current_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_get_current_tabpage();
        nlua_push_handle(
            lstate,
            ret as handle_T,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_current_tabpage(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Tabpage = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_current_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if text_locked() {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                get_text_locked_msg(),
            );
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"tabpage\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_set_current_tabpage(arg1, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_paste(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg3: Integer = 0;
    let mut arg2: Boolean = false;
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Boolean = false;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_paste\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if textlock.get() != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0
        {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
        } else {
            arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"phase\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"crlf\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"data\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        save_active_lstate = active_lstate.get();
                        active_lstate.set(lstate);
                        ret = nvim_paste(
                            LUA_INTERNAL_CALL,
                            arg1,
                            arg2,
                            arg3,
                            &raw mut arena,
                            &raw mut err,
                        );
                        nlua_push_Boolean(
                            lstate,
                            ret,
                            kNluaPushSpecial as ::core::ffi::c_int
                                | kNluaPushFreeRefs as ::core::ffi::c_int,
                        );
                        active_lstate.set(save_active_lstate);
                    }
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_put(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg4: Boolean = false;
    let mut arg3: Boolean = false;
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 4 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 4 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_put\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if textlock.get() != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0
        {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
        } else {
            arg4 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"follow\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg3 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"after\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"type\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        arg1 = nlua_pop_Array(lstate, &raw mut arena, &raw mut err);
                        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                        {
                            err_param = b"lines\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            save_active_lstate = active_lstate.get();
                            active_lstate.set(lstate);
                            nvim_put(arg1, arg2, arg3, arg4, &raw mut arena, &raw mut err);
                            active_lstate.set(save_active_lstate);
                        }
                    }
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_color_by_name(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_color_by_name\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_color_by_name(arg1);
            nlua_push_Integer(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_color_map(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_color_map\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_get_color_map(&raw mut arena);
        nlua_push_Dict(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_context(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut arg1: KeyDict_context = KeyDict_context {
        is_set__context_: 0,
        types: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_context\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = KeyDict_context {
            is_set__context_: 0 as OptionalKeys,
            types: Array {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<Object>(),
            },
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg1 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_context_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_context(&raw mut arg1, &raw mut arena, &raw mut err);
            nlua_push_Dict(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
        api_luarefs_free_keydict(
            &raw mut arg1 as *mut ::core::ffi::c_void,
            context_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_load_context(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_load_context\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_Dict(lstate, false_0 != 0, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"dict\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_load_context(arg1, &raw mut err);
            nlua_push_Object(
                lstate,
                &raw mut ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_mode(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_get_mode(&raw mut arena);
        nlua_push_Dict(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_keymap(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_keymap\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"mode\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_keymap(arg1, &raw mut arena);
            nlua_push_Array(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_keymap(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg3: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg4: KeyDict_keymap = KeyDict_keymap {
        is_set__keymap_: 0,
        noremap: false,
        nowait: false,
        silent: false,
        script: false,
        expr: false,
        unique: false,
        callback: 0,
        desc: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        replace_keycodes: false,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 4 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 4 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_keymap\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg4 = KeyDict_keymap {
            is_set__keymap_: 0 as OptionalKeys,
            noremap: false,
            nowait: false,
            silent: false,
            script: false,
            expr: false,
            unique: false,
            callback: 0,
            desc: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            replace_keycodes: false,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg4 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_keymap_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg3 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"rhs\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"lhs\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"mode\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        save_active_lstate = active_lstate.get();
                        active_lstate.set(lstate);
                        nvim_set_keymap(
                            LUA_INTERNAL_CALL,
                            arg1,
                            arg2,
                            arg3,
                            &raw mut arg4,
                            &raw mut err,
                        );
                        active_lstate.set(save_active_lstate);
                    }
                }
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg4 as *mut ::core::ffi::c_void,
            keymap_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_del_keymap(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_del_keymap\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"lhs\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"mode\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_del_keymap(LUA_INTERNAL_CALL, arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_chan_info(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_chan_info\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"chan\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_chan_info(LUA_INTERNAL_CALL, arg1, &raw mut arena, &raw mut err);
            nlua_push_Dict(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_list_chans(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_list_chans\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_list_chans(&raw mut arena);
        nlua_push_Array(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__id(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__id\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"obj\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim__id(arg1, &raw mut arena);
            nlua_push_Object(
                lstate,
                &raw mut ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
            api_luarefs_free_object(arg1);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__id_array(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__id_array\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_Array(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"arr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim__id_array(arg1, &raw mut arena);
            nlua_push_Array(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__id_dict(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__id_dict\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_Dict(lstate, false_0 != 0, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"dct\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim__id_dict(arg1, &raw mut arena);
            nlua_push_Dict(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__id_float(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Float = 0.;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Float = 0.;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__id_float\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_Float(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"flt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim__id_float(arg1);
            nlua_push_Float(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__stats(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__stats\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim__stats(&raw mut arena);
        nlua_push_Dict(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_list_uis(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_list_uis\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_list_uis(&raw mut arena);
        nlua_push_Array(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_proc_children(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_proc_children\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"pid\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_proc_children(arg1, &raw mut arena, &raw mut err);
            nlua_push_Array(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_proc(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_proc\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"pid\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_proc(arg1, &raw mut arena, &raw mut err);
            nlua_push_Object(
                lstate,
                &raw mut ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_select_popupmenu_item(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg3: Boolean = false;
    let mut arg2: Boolean = false;
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg4: KeyDict_empty = KeyDict_empty { is_set__empty_: 0 };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 4 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 4 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_select_popupmenu_item\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg4 = KeyDict_empty {
            is_set__empty_: 0 as OptionalKeys,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg4 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_empty_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg3 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"finish\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"insert\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"item\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        save_active_lstate = active_lstate.get();
                        active_lstate.set(lstate);
                        nvim_select_popupmenu_item(arg1, arg2, arg3, &raw mut arg4, &raw mut err);
                        active_lstate.set(save_active_lstate);
                    }
                }
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg4 as *mut ::core::ffi::c_void,
            empty_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__inspect_cell(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg3: Integer = 0;
    let mut arg2: Integer = 0;
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__inspect_cell\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"col\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"row\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"grid\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    ret = nvim__inspect_cell(arg1, arg2, arg3, &raw mut arena, &raw mut err);
                    nlua_push_Array(
                        lstate,
                        ret,
                        kNluaPushSpecial as ::core::ffi::c_int
                            | kNluaPushFreeRefs as ::core::ffi::c_int,
                    );
                    active_lstate.set(save_active_lstate);
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__screenshot(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"path\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim__screenshot(arg1);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__invalidate_glyph_cache(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__invalidate_glyph_cache\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        nvim__invalidate_glyph_cache();
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__unpack(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"str\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim__unpack(arg1, &raw mut arena, &raw mut err);
            nlua_push_Object(
                lstate,
                &raw mut ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_del_mark(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Boolean = false;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_del_mark\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_del_mark(arg1, &raw mut err);
            nlua_push_Boolean(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_mark(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut arg2: KeyDict_empty = KeyDict_empty { is_set__empty_: 0 };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_mark\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_empty {
            is_set__empty_: 0 as OptionalKeys,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_empty_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_get_mark(arg1, &raw mut arg2, &raw mut arena, &raw mut err);
                nlua_push_Array(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            empty_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_eval_statusline(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut arg2: KeyDict_eval_statusline = KeyDict_eval_statusline {
        is_set__eval_statusline_: 0,
        winid: 0,
        maxwidth: 0,
        fillchar: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        highlights: false,
        use_winbar: false,
        use_tabline: false,
        use_statuscol_lnum: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        arg2 = KeyDict_eval_statusline {
            is_set__eval_statusline_: 0 as OptionalKeys,
            winid: 0,
            maxwidth: 0,
            fillchar: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            highlights: false,
            use_winbar: false,
            use_tabline: false,
            use_statuscol_lnum: 0,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_eval_statusline_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"str\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_eval_statusline(arg1, &raw mut arg2, &raw mut arena, &raw mut err);
                nlua_push_Dict(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            eval_statusline_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__complete_set(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut arg2: KeyDict_complete_set = KeyDict_complete_set {
        is_set__complete_set_: 0,
        info: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__complete_set\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_complete_set {
            is_set__complete_set_: 0 as OptionalKeys,
            info: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_complete_set_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"index\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim__complete_set(arg1, &raw mut arg2, &raw mut arena, &raw mut err);
                nlua_push_Dict(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            complete_set_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__redraw(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg1: KeyDict_redraw = KeyDict_redraw {
        is_set__redraw_: 0,
        flush: false,
        cursor: false,
        valid: false,
        statuscolumn: false,
        statusline: false,
        tabline: false,
        winbar: false,
        range: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        win: 0,
        buf: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__redraw\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = KeyDict_redraw {
            is_set__redraw_: 0 as OptionalKeys,
            flush: false,
            cursor: false,
            valid: false,
            statuscolumn: false,
            statusline: false,
            tabline: false,
            winbar: false,
            range: Array {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<Object>(),
            },
            win: 0,
            buf: 0,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg1 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_redraw_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim__redraw(&raw mut arg1, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
        api_luarefs_free_keydict(
            &raw mut arg1 as *mut ::core::ffi::c_void,
            redraw_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_exec2(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut arg2: KeyDict_exec_opts = KeyDict_exec_opts { output: false };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_exec2\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_exec_opts { output: false };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_exec_opts_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"src\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_exec2(LUA_INTERNAL_CALL, arg1, &raw mut arg2, &raw mut err);
                nlua_push_Dict(
                    lstate,
                    ret,
                    0 as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
                api_free_dict(ret);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            exec_opts_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_command(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_command\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"cmd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_command(arg1, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_eval(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_eval\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"expr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_eval(arg1, &raw mut arena, &raw mut err);
            nlua_push_Object(
                lstate,
                &raw mut ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_call_function(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_call_function\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Array(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"args\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"fn\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_call_function(arg1, arg2, &raw mut arena, &raw mut err);
                nlua_push_Object(
                    lstate,
                    &raw mut ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_call_dict_function(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg3: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_call_dict_function\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = nlua_pop_Array(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"args\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"fn\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"dict\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    ret = nvim_call_dict_function(arg1, arg2, arg3, &raw mut arena, &raw mut err);
                    nlua_push_Object(
                        lstate,
                        &raw mut ret,
                        kNluaPushSpecial as ::core::ffi::c_int
                            | kNluaPushFreeRefs as ::core::ffi::c_int,
                    );
                    active_lstate.set(save_active_lstate);
                    api_luarefs_free_object(arg1);
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_parse_expression(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg3: Boolean = false;
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        arg3 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"hl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"flags\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"expr\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    ret = nvim_parse_expression(arg1, arg2, arg3, &raw mut arena, &raw mut err);
                    nlua_push_Dict(
                        lstate,
                        ret,
                        kNluaPushSpecial as ::core::ffi::c_int
                            | kNluaPushFreeRefs as ::core::ffi::c_int,
                    );
                    active_lstate.set(save_active_lstate);
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_open_win(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Boolean = false;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Window = 0;
    let mut arg3: KeyDict_win_config = KeyDict_win_config {
        is_set__win_config_: 0,
        external: false,
        fixed: false,
        focusable: false,
        footer: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        footer_pos: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        hide: false,
        height: 0,
        mouse: false,
        relative: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        row: 0.,
        style: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        noautocmd: false,
        vertical: false,
        win: 0,
        width: 0,
        zindex: 0,
        anchor: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        border: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        bufpos: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        col: 0.,
        split: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        title: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        title_pos: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        _cmdline_offset: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_open_win\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if textlock.get() != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0
        {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
        } else {
            arg3 = KeyDict_win_config {
                is_set__win_config_: 0 as OptionalKeys,
                external: false,
                fixed: false,
                focusable: false,
                footer: Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                },
                footer_pos: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                },
                hide: false,
                height: 0,
                mouse: false,
                relative: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                },
                row: 0.,
                style: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                },
                noautocmd: false,
                vertical: false,
                win: 0,
                width: 0,
                zindex: 0,
                anchor: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                },
                border: Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                },
                bufpos: Array {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<Object>(),
                },
                col: 0.,
                split: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                },
                title: Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                },
                title_pos: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                },
                _cmdline_offset: 0,
            };
            nlua_pop_keydict(
                lstate,
                &raw mut arg3 as *mut ::core::ffi::c_void,
                Some(
                    KeyDict_win_config_get_field
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            size_t,
                        ) -> *mut KeySetLink,
                ),
                &raw mut err_param,
                &raw mut arena,
                &raw mut err,
            );
            if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
                arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"enter\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        save_active_lstate = active_lstate.get();
                        active_lstate.set(lstate);
                        ret = nvim_open_win(arg1, arg2, &raw mut arg3, &raw mut err);
                        nlua_push_handle(
                            lstate,
                            ret as handle_T,
                            kNluaPushSpecial as ::core::ffi::c_int
                                | kNluaPushFreeRefs as ::core::ffi::c_int,
                        );
                        active_lstate.set(save_active_lstate);
                    }
                }
            }
            api_luarefs_free_keydict(
                &raw mut arg3 as *mut ::core::ffi::c_void,
                win_config_table.ptr() as *mut KeySetLink,
            );
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_set_config(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg2: KeyDict_win_config = KeyDict_win_config {
        is_set__win_config_: 0,
        external: false,
        fixed: false,
        focusable: false,
        footer: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        footer_pos: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        hide: false,
        height: 0,
        mouse: false,
        relative: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        row: 0.,
        style: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        noautocmd: false,
        vertical: false,
        win: 0,
        width: 0,
        zindex: 0,
        anchor: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        border: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        bufpos: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        col: 0.,
        split: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        title: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        title_pos: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        _cmdline_offset: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_set_config\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_win_config {
            is_set__win_config_: 0 as OptionalKeys,
            external: false,
            fixed: false,
            focusable: false,
            footer: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            footer_pos: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            hide: false,
            height: 0,
            mouse: false,
            relative: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            row: 0.,
            style: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            noautocmd: false,
            vertical: false,
            win: 0,
            width: 0,
            zindex: 0,
            anchor: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            border: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            bufpos: Array {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<Object>(),
            },
            col: 0.,
            split: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            title: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            title_pos: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            _cmdline_offset: 0,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_win_config_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_win_set_config(arg1, &raw mut arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            win_config_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_get_config(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: KeyDict_win_config = KeyDict_win_config {
        is_set__win_config_: 0,
        external: false,
        fixed: false,
        focusable: false,
        footer: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        footer_pos: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        hide: false,
        height: 0,
        mouse: false,
        relative: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        row: 0.,
        style: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        noautocmd: false,
        vertical: false,
        win: 0,
        width: 0,
        zindex: 0,
        anchor: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        border: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        bufpos: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        col: 0.,
        split: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        title: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        title_pos: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        _cmdline_offset: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_get_config\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_win_get_config(arg1, &raw mut arena, &raw mut err);
            nlua_push_keydict(
                lstate,
                &raw mut ret as *mut ::core::ffi::c_void,
                win_config_table.ptr() as *mut KeySetLink,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_get_buf(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Buffer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_get_buf\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_win_get_buf(arg1, &raw mut err);
            nlua_push_handle(
                lstate,
                ret as handle_T,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_set_buf(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Buffer = 0;
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_set_buf\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if textlock.get() != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0
        {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
        } else {
            arg2 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_win_set_buf(arg1, arg2, &raw mut err);
                    active_lstate.set(save_active_lstate);
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_get_cursor(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_get_cursor\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_win_get_cursor(arg1, &raw mut arena, &raw mut err);
            nlua_push_Array(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_set_cursor(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg2: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_set_cursor\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Array(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"pos\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_win_set_cursor(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_get_height(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_get_height\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_win_get_height(arg1, &raw mut err);
            nlua_push_Integer(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_set_height(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg2: Integer = 0;
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_set_height\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"height\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_win_set_height(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_get_width(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_get_width\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_win_get_width(arg1, &raw mut err);
            nlua_push_Integer(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_set_width(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Integer = 0;
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_set_width\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"width\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_win_set_width(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_get_var(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_get_var\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_win_get_var(arg1, arg2, &raw mut arena, &raw mut err);
                nlua_push_Object(
                    lstate,
                    &raw mut ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_set_var(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg3: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_set_var\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"value\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_win_set_var(arg1, arg2, arg3, &raw mut err);
                    active_lstate.set(save_active_lstate);
                }
            }
            api_luarefs_free_object(arg3);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_del_var(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_del_var\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_win_del_var(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_get_position(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_get_position\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_win_get_position(arg1, &raw mut arena, &raw mut err);
            nlua_push_Array(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_get_tabpage(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Tabpage = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_get_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_win_get_tabpage(arg1, &raw mut err);
            nlua_push_handle(
                lstate,
                ret as handle_T,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_get_number(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_get_number\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_win_get_number(arg1, &raw mut err);
            nlua_push_Integer(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_is_valid(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Boolean = false;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_is_valid\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_win_is_valid(arg1);
            nlua_push_Boolean(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_hide(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_hide\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if textlock.get() != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0
        {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_win_hide(arg1, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_close(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Boolean = false;
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_close\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if textlock.get() != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0
        {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
        } else {
            arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"force\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_win_close(arg1, arg2, &raw mut err);
                    active_lstate.set(save_active_lstate);
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_call(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg2: LuaRef = 0;
    let mut arg1: Window = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_call\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_LuaRef(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"fun\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_win_call(arg1, arg2, &raw mut err);
                nlua_push_Object(
                    lstate,
                    &raw mut ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
            api_free_luaref(arg2);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_set_hl_ns(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Integer = 0;
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_set_hl_ns\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"ns_id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_win_set_hl_ns(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_text_height(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut arg2: KeyDict_win_text_height = KeyDict_win_text_height {
        is_set__win_text_height_: 0,
        start_row: 0,
        end_row: 0,
        start_vcol: 0,
        end_vcol: 0,
        max_height: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_text_height\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_win_text_height {
            is_set__win_text_height_: 0 as OptionalKeys,
            start_row: 0,
            end_row: 0,
            start_vcol: 0,
            end_vcol: 0,
            max_height: 0,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_win_text_height_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_win_text_height(arg1, &raw mut arg2, &raw mut arena, &raw mut err);
                nlua_push_Dict(
                    lstate,
                    ret,
                    0 as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            win_text_height_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn nlua_add_api_functions(mut lstate: *mut lua_State) {
    lua_createtable(lstate, 0 as ::core::ffi::c_int, 181 as ::core::ffi::c_int);
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_autocmds
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_autocmds\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_create_autocmd
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_create_autocmd\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_del_autocmd as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_del_autocmd\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_clear_autocmds
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_clear_autocmds\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_create_augroup
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_create_augroup\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_del_augroup_by_id
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_del_augroup_by_id\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_del_augroup_by_name
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_del_augroup_by_name\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_exec_autocmds
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_exec_autocmds\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_line_count
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_line_count\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_attach as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_attach\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_lines
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_lines\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_set_lines
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_set_lines\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_set_text
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_set_text\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_text
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_text\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_offset
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_offset\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_var as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_var\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_changedtick
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_changedtick\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_keymap
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_keymap\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_set_keymap
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_set_keymap\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_del_keymap
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_del_keymap\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_set_var as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_set_var\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_del_var as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_del_var\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_name
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_name\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_set_name
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_set_name\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_is_loaded
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_is_loaded\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_delete as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_delete\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_is_valid
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_is_valid\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_del_mark
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_del_mark\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_set_mark
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_set_mark\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_mark
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_mark\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_buf_call as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_call\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim__buf_stats as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__buf_stats\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_parse_cmd as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_parse_cmd\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_cmd as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_cmd\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_create_user_command
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_create_user_command\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_del_user_command
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_del_user_command\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_create_user_command
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_create_user_command\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_del_user_command
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_del_user_command\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_commands
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_commands\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_commands
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_commands\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_exec as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_exec\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_command_output
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_command_output\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_number
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_number\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_clear_highlight
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_clear_highlight\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_add_highlight
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_add_highlight\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_set_virtual_text
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_set_virtual_text\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_hl_by_id
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_hl_by_id\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_hl_by_name
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_hl_by_name\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_option_info
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_option_info\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_set_option as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_option\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_option as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_option\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_option
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_option\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_set_option
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_set_option\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_get_option
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_get_option\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_set_option
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_set_option\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_out_write as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_out_write\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_err_write as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_err_write\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_err_writeln as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_err_writeln\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_notify as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_notify\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_create_namespace
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_create_namespace\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_namespaces
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_namespaces\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_extmark_by_id
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_extmark_by_id\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_extmarks
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_extmarks\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_set_extmark
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_set_extmark\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_del_extmark
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_del_extmark\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_clear_namespace
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_clear_namespace\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_set_decoration_provider
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_decoration_provider\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim__buf_debug_extmarks
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__buf_debug_extmarks\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim__ns_set as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__ns_set\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim__ns_get as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__ns_get\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_option_value
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_option_value\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_set_option_value
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_option_value\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_all_options_info
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_all_options_info\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_option_info2
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_option_info2\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_tabpage_list_wins
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_tabpage_list_wins\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_tabpage_get_var
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_tabpage_get_var\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_tabpage_set_var
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_tabpage_set_var\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_tabpage_del_var
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_tabpage_del_var\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_tabpage_get_win
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_tabpage_get_win\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_tabpage_set_win
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_tabpage_set_win\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_tabpage_get_number
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_tabpage_get_number\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_tabpage_is_valid
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_tabpage_is_valid\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_open_tabpage
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_open_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_ui_send as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_ui_send\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_hl_id_by_name
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_hl_id_by_name\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_get_hl as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_hl\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_set_hl as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_hl\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_get_hl_ns as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_hl_ns\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_set_hl_ns as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_hl_ns\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_set_hl_ns_fast
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_hl_ns_fast\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_feedkeys as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_feedkeys\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_input as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_input\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_input_mouse as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_input_mouse\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_replace_termcodes
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_replace_termcodes\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_strwidth as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_strwidth\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_list_runtime_paths
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_list_runtime_paths\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim__runtime_inspect
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__runtime_inspect\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_runtime_file
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_runtime_file\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim__get_lib_dir
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__get_lib_dir\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim__get_runtime
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__get_runtime\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_set_current_dir
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_current_dir\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_current_line
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_current_line\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_set_current_line
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_current_line\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_del_current_line
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_del_current_line\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_get_var as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_var\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_set_var as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_var\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_del_var as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_del_var\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_get_vvar as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_vvar\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_set_vvar as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_vvar\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_echo as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_echo\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_list_bufs as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_list_bufs\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_current_buf
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_current_buf\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_set_current_buf
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_current_buf\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_list_wins as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_list_wins\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_current_win
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_current_win\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_set_current_win
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_current_win\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_create_buf as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_create_buf\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_open_term as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_open_term\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_chan_send as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_chan_send\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_list_tabpages
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_list_tabpages\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_current_tabpage
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_current_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_set_current_tabpage
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_current_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_paste as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_paste\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_put as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_put\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_color_by_name
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_color_by_name\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_color_map
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_color_map\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_context as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_context\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_load_context
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_load_context\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_get_mode as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_mode\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_keymap as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_keymap\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_set_keymap as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_keymap\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_del_keymap as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_del_keymap\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_chan_info
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_chan_info\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_list_chans as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_list_chans\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim__id as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__id\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim__id_array as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__id_array\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim__id_dict as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__id_dict\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim__id_float as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__id_float\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim__stats as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__stats\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_list_uis as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_list_uis\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_proc_children
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_proc_children\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_get_proc as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_proc\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_select_popupmenu_item
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_select_popupmenu_item\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim__inspect_cell
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__inspect_cell\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim__screenshot as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__screenshot\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim__invalidate_glyph_cache
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__invalidate_glyph_cache\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim__unpack as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__unpack\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_del_mark as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_del_mark\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_get_mark as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_mark\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_eval_statusline
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_eval_statusline\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim__complete_set
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__complete_set\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim__redraw as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__redraw\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_exec2 as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_exec2\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_command as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_command\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_eval as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_eval\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_call_function
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_call_function\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_call_dict_function
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_call_dict_function\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_parse_expression
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_parse_expression\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_open_win as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_open_win\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_set_config
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_set_config\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_get_config
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_get_config\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_get_buf as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_get_buf\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_set_buf as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_set_buf\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_get_cursor
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_get_cursor\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_set_cursor
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_set_cursor\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_get_height
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_get_height\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_set_height
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_set_height\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_get_width
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_get_width\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_set_width
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_set_width\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_get_var as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_get_var\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_set_var as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_set_var\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_del_var as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_del_var\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_get_position
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_get_position\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_get_tabpage
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_get_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_get_number
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_get_number\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_is_valid
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_is_valid\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_win_hide as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_hide\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_win_close as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_close\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_win_call as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_call\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_set_hl_ns
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_set_hl_ns\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_text_height
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_text_height\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"api\0".as_ptr() as *const ::core::ffi::c_char,
    );
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
