use crate::src::nvim::api::private::dispatch_wrappers::{
    handle_buffer_del_line, handle_buffer_del_var, handle_buffer_get_line,
    handle_buffer_get_line_slice, handle_buffer_insert, handle_buffer_set_line,
    handle_buffer_set_line_slice, handle_buffer_set_var, handle_nvim__buf_debug_extmarks,
    handle_nvim__buf_stats, handle_nvim__chan_set_detach, handle_nvim__complete_set,
    handle_nvim__exec_lua_fast, handle_nvim__get_lib_dir, handle_nvim__get_runtime,
    handle_nvim__id, handle_nvim__id_array, handle_nvim__id_dict, handle_nvim__id_float,
    handle_nvim__inspect_cell, handle_nvim__invalidate_glyph_cache, handle_nvim__ns_get,
    handle_nvim__ns_set, handle_nvim__redraw, handle_nvim__runtime_inspect,
    handle_nvim__screenshot, handle_nvim__stats, handle_nvim__unpack,
    handle_nvim_buf_add_highlight, handle_nvim_buf_attach, handle_nvim_buf_clear_highlight,
    handle_nvim_buf_clear_namespace, handle_nvim_buf_create_user_command,
    handle_nvim_buf_del_extmark, handle_nvim_buf_del_keymap, handle_nvim_buf_del_mark,
    handle_nvim_buf_del_user_command, handle_nvim_buf_del_var, handle_nvim_buf_delete,
    handle_nvim_buf_detach, handle_nvim_buf_get_changedtick, handle_nvim_buf_get_commands,
    handle_nvim_buf_get_extmark_by_id, handle_nvim_buf_get_extmarks, handle_nvim_buf_get_keymap,
    handle_nvim_buf_get_lines, handle_nvim_buf_get_mark, handle_nvim_buf_get_name,
    handle_nvim_buf_get_number, handle_nvim_buf_get_offset, handle_nvim_buf_get_option,
    handle_nvim_buf_get_text, handle_nvim_buf_get_var, handle_nvim_buf_is_loaded,
    handle_nvim_buf_is_valid, handle_nvim_buf_line_count, handle_nvim_buf_set_extmark,
    handle_nvim_buf_set_keymap, handle_nvim_buf_set_lines, handle_nvim_buf_set_mark,
    handle_nvim_buf_set_name, handle_nvim_buf_set_option, handle_nvim_buf_set_text,
    handle_nvim_buf_set_var, handle_nvim_buf_set_virtual_text, handle_nvim_call_atomic,
    handle_nvim_call_dict_function, handle_nvim_call_function, handle_nvim_chan_send,
    handle_nvim_clear_autocmds, handle_nvim_cmd, handle_nvim_command, handle_nvim_command_output,
    handle_nvim_create_augroup, handle_nvim_create_autocmd, handle_nvim_create_buf,
    handle_nvim_create_namespace, handle_nvim_create_user_command, handle_nvim_del_augroup_by_id,
    handle_nvim_del_augroup_by_name, handle_nvim_del_autocmd, handle_nvim_del_current_line,
    handle_nvim_del_keymap, handle_nvim_del_mark, handle_nvim_del_user_command,
    handle_nvim_del_var, handle_nvim_echo, handle_nvim_err_write, handle_nvim_err_writeln,
    handle_nvim_error_event, handle_nvim_eval, handle_nvim_eval_statusline, handle_nvim_exec,
    handle_nvim_exec_autocmds, handle_nvim_exec_lua, handle_nvim_exec2, handle_nvim_execute_lua,
    handle_nvim_feedkeys, handle_nvim_get_all_options_info, handle_nvim_get_api_info,
    handle_nvim_get_autocmds, handle_nvim_get_chan_info, handle_nvim_get_color_by_name,
    handle_nvim_get_color_map, handle_nvim_get_commands, handle_nvim_get_context,
    handle_nvim_get_current_buf, handle_nvim_get_current_line, handle_nvim_get_current_tabpage,
    handle_nvim_get_current_win, handle_nvim_get_hl, handle_nvim_get_hl_by_id,
    handle_nvim_get_hl_by_name, handle_nvim_get_hl_id_by_name, handle_nvim_get_hl_ns,
    handle_nvim_get_keymap, handle_nvim_get_mark, handle_nvim_get_mode, handle_nvim_get_namespaces,
    handle_nvim_get_option, handle_nvim_get_option_info, handle_nvim_get_option_info2,
    handle_nvim_get_option_value, handle_nvim_get_proc, handle_nvim_get_proc_children,
    handle_nvim_get_runtime_file, handle_nvim_get_var, handle_nvim_get_vvar, handle_nvim_input,
    handle_nvim_input_mouse, handle_nvim_list_bufs, handle_nvim_list_chans,
    handle_nvim_list_runtime_paths, handle_nvim_list_tabpages, handle_nvim_list_uis,
    handle_nvim_list_wins, handle_nvim_load_context, handle_nvim_notify, handle_nvim_open_tabpage,
    handle_nvim_open_term, handle_nvim_open_win, handle_nvim_out_write, handle_nvim_parse_cmd,
    handle_nvim_parse_expression, handle_nvim_paste, handle_nvim_put,
    handle_nvim_replace_termcodes, handle_nvim_select_popupmenu_item, handle_nvim_set_client_info,
    handle_nvim_set_current_buf, handle_nvim_set_current_dir, handle_nvim_set_current_line,
    handle_nvim_set_current_tabpage, handle_nvim_set_current_win, handle_nvim_set_hl,
    handle_nvim_set_hl_ns, handle_nvim_set_hl_ns_fast, handle_nvim_set_keymap,
    handle_nvim_set_option, handle_nvim_set_option_value, handle_nvim_set_var,
    handle_nvim_set_vvar, handle_nvim_strwidth, handle_nvim_subscribe, handle_nvim_tabpage_del_var,
    handle_nvim_tabpage_get_number, handle_nvim_tabpage_get_var, handle_nvim_tabpage_get_win,
    handle_nvim_tabpage_is_valid, handle_nvim_tabpage_list_wins, handle_nvim_tabpage_set_var,
    handle_nvim_tabpage_set_win, handle_nvim_ui_attach, handle_nvim_ui_detach,
    handle_nvim_ui_pum_set_bounds, handle_nvim_ui_pum_set_height, handle_nvim_ui_send,
    handle_nvim_ui_set_focus, handle_nvim_ui_set_option, handle_nvim_ui_term_event,
    handle_nvim_ui_try_resize, handle_nvim_ui_try_resize_grid, handle_nvim_unsubscribe,
    handle_nvim_win_close, handle_nvim_win_del_var, handle_nvim_win_get_buf,
    handle_nvim_win_get_config, handle_nvim_win_get_cursor, handle_nvim_win_get_height,
    handle_nvim_win_get_number, handle_nvim_win_get_option, handle_nvim_win_get_position,
    handle_nvim_win_get_tabpage, handle_nvim_win_get_var, handle_nvim_win_get_width,
    handle_nvim_win_hide, handle_nvim_win_is_valid, handle_nvim_win_set_buf,
    handle_nvim_win_set_config, handle_nvim_win_set_cursor, handle_nvim_win_set_height,
    handle_nvim_win_set_hl_ns, handle_nvim_win_set_option, handle_nvim_win_set_var,
    handle_nvim_win_set_width, handle_nvim_win_text_height, handle_tabpage_del_var,
    handle_tabpage_set_var, handle_ui_attach, handle_vim_del_var, handle_vim_set_var,
    handle_window_del_var, handle_window_set_var,
};
use crate::src::nvim::api::private::helpers::api_set_error;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::os::libc::memcmp;
pub use crate::src::nvim::types::{
    ApiDispatchWrapper, Arena, Array, Boolean, Buffer, Dict, Error, ErrorType, FieldHashfn, Float,
    HLGroupID, Integer, KeyDict_buf_attach, KeyDict_buf_delete, KeyDict_clear_autocmds,
    KeyDict_cmd, KeyDict_cmd_opts, KeyDict_complete_set, KeyDict_context, KeyDict_create_augroup,
    KeyDict_create_autocmd, KeyDict_echo_opts, KeyDict_empty, KeyDict_eval_statusline,
    KeyDict_exec_autocmds, KeyDict_exec_opts, KeyDict_get_autocmds, KeyDict_get_commands,
    KeyDict_get_extmark, KeyDict_get_extmarks, KeyDict_get_highlight, KeyDict_get_ns,
    KeyDict_highlight, KeyDict_keymap, KeyDict_ns_opts, KeyDict_open_term, KeyDict_option,
    KeyDict_redraw, KeyDict_runtime, KeyDict_set_extmark, KeyDict_tabpage_config,
    KeyDict_user_command, KeyDict_win_config, KeyDict_win_text_height, KeySetLink, KeyValuePair,
    LuaRef, MsgpackRpcRequestHandler, Object, ObjectType, OptionalKeys, String_0, Tabpage, Window,
    handle_T, int64_t, key_value_pair, lua_State, object, object_data as C2Rust_Unnamed, size_t,
    uint64_t,
};
use crate::src::nvim::ui_client::handle_ui_client_redraw;
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
pub type C2Rust_Unnamed_0 = ::core::ffi::c_int;
pub const kUnpackTypeStringArray: C2Rust_Unnamed_0 = -1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: Dict = Dict {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<KeyValuePair>(),
};
pub const ARRAY_DICT_INIT: Dict = KV_INITIAL_VALUE;
pub const LOGLVL_DBG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub static empty_table: GlobalCell<[KeySetLink; 1]> = GlobalCell::new([KeySetLink {
    str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    ptr_off: 0 as size_t,
    type_0: kObjectTypeNil as ::core::ffi::c_int,
    opt_index: -1 as ::core::ffi::c_int,
    is_hlgroup: false_0 != 0,
}]);
pub unsafe extern "C" fn empty_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*empty_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_empty_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = empty_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (empty_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static context_table: GlobalCell<[KeySetLink; 2]> = GlobalCell::new([
    KeySetLink {
        str: b"types\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeArray as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn context_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        5 => {
            low = 0 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*context_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_context_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = context_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (context_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static set_decoration_provider_table: GlobalCell<[KeySetLink; 10]> = GlobalCell::new([
    KeySetLink {
        str: b"on_buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 12 as size_t,
        type_0: kObjectTypeLuaRef as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"on_end\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 28 as size_t,
        type_0: kObjectTypeLuaRef as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"on_win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 16 as size_t,
        type_0: kObjectTypeLuaRef as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"on_line\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 20 as size_t,
        type_0: kObjectTypeLuaRef as ::core::ffi::c_int,
        opt_index: 4 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"on_range\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 24 as size_t,
        type_0: kObjectTypeLuaRef as ::core::ffi::c_int,
        opt_index: 5 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"on_start\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeLuaRef as ::core::ffi::c_int,
        opt_index: 6 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"_on_hl_def\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 32 as size_t,
        type_0: kObjectTypeLuaRef as ::core::ffi::c_int,
        opt_index: 7 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"_on_spell_nav\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 36 as size_t,
        type_0: kObjectTypeLuaRef as ::core::ffi::c_int,
        opt_index: 8 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"_on_conceal_line\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        ptr_off: 40 as size_t,
        type_0: kObjectTypeLuaRef as ::core::ffi::c_int,
        opt_index: 9 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn set_decoration_provider_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        6 => match *str.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            98 => {
                low = 0 as ::core::ffi::c_int;
            }
            101 => {
                low = 1 as ::core::ffi::c_int;
            }
            119 => {
                low = 2 as ::core::ffi::c_int;
            }
            _ => {}
        },
        7 => {
            low = 3 as ::core::ffi::c_int;
        }
        8 => match *str.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            114 => {
                low = 4 as ::core::ffi::c_int;
            }
            115 => {
                low = 5 as ::core::ffi::c_int;
            }
            _ => {}
        },
        10 => {
            low = 6 as ::core::ffi::c_int;
        }
        13 => {
            low = 7 as ::core::ffi::c_int;
        }
        16 => {
            low = 8 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*set_decoration_provider_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_set_decoration_provider_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = set_decoration_provider_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (set_decoration_provider_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static set_extmark_table: GlobalCell<[KeySetLink; 36]> = GlobalCell::new([
    KeySetLink {
        str: b"id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"url\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 312 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"spell\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 304 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"scoped\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 328 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 4 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"hl_eol\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 122 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 5 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"strict\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 216 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 6 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"end_col\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 32 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 7 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"conceal\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 272 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 8 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"hl_mode\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 128 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 9 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"end_row\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 24 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 10 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"end_line\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 16 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 11 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"hl_group\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 40 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 12 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"priority\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 152 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 13 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"ephemeral\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 145 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 14 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"sign_text\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 224 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 15 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"virt_text\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 72 as size_t,
        type_0: kObjectTypeArray as ::core::ffi::c_int,
        opt_index: 16 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"invalidate\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 144 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 17 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"ui_watched\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 305 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 18 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"virt_lines\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 168 as size_t,
        type_0: kObjectTypeArray as ::core::ffi::c_int,
        opt_index: 19 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"_subpriority\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 336 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 20 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"undo_restore\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 306 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 21 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"conceal_lines\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 288 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 22 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"line_hl_group\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 256 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 23 as ::core::ffi::c_int,
        is_hlgroup: true_0 != 0,
    },
    KeySetLink {
        str: b"right_gravity\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 160 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 24 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"sign_hl_group\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 240 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 25 as ::core::ffi::c_int,
        is_hlgroup: true_0 != 0,
    },
    KeySetLink {
        str: b"virt_text_pos\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 96 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 26 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"virt_text_hide\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 120 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 27 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"number_hl_group\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        ptr_off: 248 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 28 as ::core::ffi::c_int,
        is_hlgroup: true_0 != 0,
    },
    KeySetLink {
        str: b"virt_lines_above\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        ptr_off: 192 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 29 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"end_right_gravity\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        ptr_off: 161 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 30 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"virt_text_win_col\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        ptr_off: 112 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 31 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"virt_lines_leftcol\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        ptr_off: 193 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 32 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"cursorline_hl_group\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        ptr_off: 264 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 33 as ::core::ffi::c_int,
        is_hlgroup: true_0 != 0,
    },
    KeySetLink {
        str: b"virt_lines_overflow\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        ptr_off: 200 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 34 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"virt_text_repeat_linebreak\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        ptr_off: 121 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 35 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn set_extmark_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        2 => {
            low = 0 as ::core::ffi::c_int;
        }
        3 => {
            low = 1 as ::core::ffi::c_int;
        }
        5 => {
            low = 2 as ::core::ffi::c_int;
        }
        6 => match *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 3 as ::core::ffi::c_int;
            }
            108 => {
                low = 4 as ::core::ffi::c_int;
            }
            116 => {
                low = 5 as ::core::ffi::c_int;
            }
            _ => {}
        },
        7 => match *str.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 6 as ::core::ffi::c_int;
            }
            101 => {
                low = 7 as ::core::ffi::c_int;
            }
            111 => {
                low = 8 as ::core::ffi::c_int;
            }
            114 => {
                low = 9 as ::core::ffi::c_int;
            }
            _ => {}
        },
        8 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            101 => {
                low = 10 as ::core::ffi::c_int;
            }
            104 => {
                low = 11 as ::core::ffi::c_int;
            }
            112 => {
                low = 12 as ::core::ffi::c_int;
            }
            _ => {}
        },
        9 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            101 => {
                low = 13 as ::core::ffi::c_int;
            }
            115 => {
                low = 14 as ::core::ffi::c_int;
            }
            118 => {
                low = 15 as ::core::ffi::c_int;
            }
            _ => {}
        },
        10 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            105 => {
                low = 16 as ::core::ffi::c_int;
            }
            117 => {
                low = 17 as ::core::ffi::c_int;
            }
            118 => {
                low = 18 as ::core::ffi::c_int;
            }
            _ => {}
        },
        12 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 19 as ::core::ffi::c_int;
            }
            117 => {
                low = 20 as ::core::ffi::c_int;
            }
            _ => {}
        },
        13 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 21 as ::core::ffi::c_int;
            }
            108 => {
                low = 22 as ::core::ffi::c_int;
            }
            114 => {
                low = 23 as ::core::ffi::c_int;
            }
            115 => {
                low = 24 as ::core::ffi::c_int;
            }
            118 => {
                low = 25 as ::core::ffi::c_int;
            }
            _ => {}
        },
        14 => {
            low = 26 as ::core::ffi::c_int;
        }
        15 => {
            low = 27 as ::core::ffi::c_int;
        }
        16 => {
            low = 28 as ::core::ffi::c_int;
        }
        17 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            101 => {
                low = 29 as ::core::ffi::c_int;
            }
            118 => {
                low = 30 as ::core::ffi::c_int;
            }
            _ => {}
        },
        18 => {
            low = 31 as ::core::ffi::c_int;
        }
        19 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 32 as ::core::ffi::c_int;
            }
            118 => {
                low = 33 as ::core::ffi::c_int;
            }
            _ => {}
        },
        26 => {
            low = 34 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*set_extmark_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_set_extmark_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = set_extmark_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (set_extmark_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static get_extmark_table: GlobalCell<[KeySetLink; 3]> = GlobalCell::new([
    KeySetLink {
        str: b"details\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"hl_name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 9 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn get_extmark_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        7 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            100 => {
                low = 0 as ::core::ffi::c_int;
            }
            104 => {
                low = 1 as ::core::ffi::c_int;
            }
            _ => {}
        },
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*get_extmark_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_get_extmark_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = get_extmark_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (get_extmark_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static get_extmarks_table: GlobalCell<[KeySetLink; 6]> = GlobalCell::new([
    KeySetLink {
        str: b"type\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 24 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"limit\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"details\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 16 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"hl_name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 17 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 4 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"overlap\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 18 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 5 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn get_extmarks_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        4 => {
            low = 0 as ::core::ffi::c_int;
        }
        5 => {
            low = 1 as ::core::ffi::c_int;
        }
        7 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            100 => {
                low = 2 as ::core::ffi::c_int;
            }
            104 => {
                low = 3 as ::core::ffi::c_int;
            }
            111 => {
                low = 4 as ::core::ffi::c_int;
            }
            _ => {}
        },
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*get_extmarks_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_get_extmarks_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = get_extmarks_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (get_extmarks_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static keymap_table: GlobalCell<[KeySetLink; 10]> = GlobalCell::new([
    KeySetLink {
        str: b"desc\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 24 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"expr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 12 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"script\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 11 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"silent\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 10 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 4 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"unique\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 13 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 5 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"nowait\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 9 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 6 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"noremap\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 7 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"callback\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 16 as size_t,
        type_0: kObjectTypeLuaRef as ::core::ffi::c_int,
        opt_index: 8 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"replace_keycodes\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        ptr_off: 40 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 9 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn keymap_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        4 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            100 => {
                low = 0 as ::core::ffi::c_int;
            }
            101 => {
                low = 1 as ::core::ffi::c_int;
            }
            _ => {}
        },
        6 => match *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 2 as ::core::ffi::c_int;
            }
            105 => {
                low = 3 as ::core::ffi::c_int;
            }
            110 => {
                low = 4 as ::core::ffi::c_int;
            }
            111 => {
                low = 5 as ::core::ffi::c_int;
            }
            _ => {}
        },
        7 => {
            low = 6 as ::core::ffi::c_int;
        }
        8 => {
            low = 7 as ::core::ffi::c_int;
        }
        16 => {
            low = 8 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*keymap_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_keymap_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = keymap_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (keymap_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static get_commands_table: GlobalCell<[KeySetLink; 2]> = GlobalCell::new([
    KeySetLink {
        str: b"builtin\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 0 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn get_commands_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        7 => {
            low = 0 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*get_commands_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_get_commands_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = get_commands_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (get_commands_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static user_command_table: GlobalCell<[KeySetLink; 13]> = GlobalCell::new([
    KeySetLink {
        str: b"bar\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 41 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"addr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"bang\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 40 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"desc\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 112 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 4 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"count\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 80 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 5 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"force\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 144 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 6 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"nargs\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 152 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 7 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"range\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 216 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 8 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"preview\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 184 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 9 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"complete\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 48 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 10 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"register\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 248 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 11 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"keepscript\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 145 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 12 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn user_command_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        3 => {
            low = 0 as ::core::ffi::c_int;
        }
        4 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            97 => {
                low = 1 as ::core::ffi::c_int;
            }
            98 => {
                low = 2 as ::core::ffi::c_int;
            }
            100 => {
                low = 3 as ::core::ffi::c_int;
            }
            _ => {}
        },
        5 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 4 as ::core::ffi::c_int;
            }
            102 => {
                low = 5 as ::core::ffi::c_int;
            }
            110 => {
                low = 6 as ::core::ffi::c_int;
            }
            114 => {
                low = 7 as ::core::ffi::c_int;
            }
            _ => {}
        },
        7 => {
            low = 8 as ::core::ffi::c_int;
        }
        8 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 9 as ::core::ffi::c_int;
            }
            114 => {
                low = 10 as ::core::ffi::c_int;
            }
            _ => {}
        },
        10 => {
            low = 11 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*user_command_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_user_command_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = user_command_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (user_command_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static win_config_table: GlobalCell<[KeySetLink; 25]> = GlobalCell::new([
    KeySetLink {
        str: b"col\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 224 as size_t,
        type_0: kObjectTypeFloat as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"row\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 104 as size_t,
        type_0: kObjectTypeFloat as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 132 as size_t,
        type_0: kObjectTypeWindow as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"hide\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 64 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 4 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"width\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 136 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 5 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"split\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 232 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 6 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"title\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 248 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 7 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"mouse\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 80 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 8 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"fixed\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 9 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 9 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"style\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 112 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 10 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"anchor\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 152 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 11 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"bufpos\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 200 as size_t,
        type_0: kObjectTypeArray as ::core::ffi::c_int,
        opt_index: 12 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"height\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 72 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 13 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"zindex\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 144 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 14 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"footer\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 16 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 15 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"border\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 168 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 16 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"external\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 17 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"relative\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 88 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 18 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"vertical\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 129 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 19 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"focusable\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 10 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 20 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"noautocmd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 128 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 21 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"title_pos\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 280 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 22 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"footer_pos\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 48 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 23 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"_cmdline_offset\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        ptr_off: 296 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 24 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn win_config_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        3 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 0 as ::core::ffi::c_int;
            }
            114 => {
                low = 1 as ::core::ffi::c_int;
            }
            119 => {
                low = 2 as ::core::ffi::c_int;
            }
            _ => {}
        },
        4 => {
            low = 3 as ::core::ffi::c_int;
        }
        5 => match *str.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            100 => {
                low = 4 as ::core::ffi::c_int;
            }
            108 => {
                low = 5 as ::core::ffi::c_int;
            }
            116 => {
                low = 6 as ::core::ffi::c_int;
            }
            117 => {
                low = 7 as ::core::ffi::c_int;
            }
            120 => {
                low = 8 as ::core::ffi::c_int;
            }
            121 => {
                low = 9 as ::core::ffi::c_int;
            }
            _ => {}
        },
        6 => match *str.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 10 as ::core::ffi::c_int;
            }
            102 => {
                low = 11 as ::core::ffi::c_int;
            }
            105 => {
                low = 12 as ::core::ffi::c_int;
            }
            110 => {
                low = 13 as ::core::ffi::c_int;
            }
            111 => {
                low = 14 as ::core::ffi::c_int;
            }
            114 => {
                low = 15 as ::core::ffi::c_int;
            }
            _ => {}
        },
        8 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            101 => {
                low = 16 as ::core::ffi::c_int;
            }
            114 => {
                low = 17 as ::core::ffi::c_int;
            }
            118 => {
                low = 18 as ::core::ffi::c_int;
            }
            _ => {}
        },
        9 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            102 => {
                low = 19 as ::core::ffi::c_int;
            }
            110 => {
                low = 20 as ::core::ffi::c_int;
            }
            116 => {
                low = 21 as ::core::ffi::c_int;
            }
            _ => {}
        },
        10 => {
            low = 22 as ::core::ffi::c_int;
        }
        15 => {
            low = 23 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*win_config_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_win_config_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = win_config_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (win_config_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static tabpage_config_table: GlobalCell<[KeySetLink; 2]> = GlobalCell::new([
    KeySetLink {
        str: b"after\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn tabpage_config_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        5 => {
            low = 0 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*tabpage_config_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_tabpage_config_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = tabpage_config_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (tabpage_config_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static runtime_table: GlobalCell<[KeySetLink; 3]> = GlobalCell::new([
    KeySetLink {
        str: b"is_lua\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 0 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"do_source\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 1 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn runtime_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        6 => {
            low = 0 as ::core::ffi::c_int;
        }
        9 => {
            low = 1 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*runtime_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_runtime_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = runtime_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (runtime_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static eval_statusline_table: GlobalCell<[KeySetLink; 8]> = GlobalCell::new([
    KeySetLink {
        str: b"winid\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeWindow as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"fillchar\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 24 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"maxwidth\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 16 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"highlights\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 40 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 4 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"use_winbar\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 41 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 5 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"use_tabline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 42 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 6 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"use_statuscol_lnum\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        ptr_off: 48 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 7 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn eval_statusline_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        5 => {
            low = 0 as ::core::ffi::c_int;
        }
        8 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            102 => {
                low = 1 as ::core::ffi::c_int;
            }
            109 => {
                low = 2 as ::core::ffi::c_int;
            }
            _ => {}
        },
        10 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            104 => {
                low = 3 as ::core::ffi::c_int;
            }
            117 => {
                low = 4 as ::core::ffi::c_int;
            }
            _ => {}
        },
        11 => {
            low = 5 as ::core::ffi::c_int;
        }
        18 => {
            low = 6 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*eval_statusline_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_eval_statusline_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = eval_statusline_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (eval_statusline_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static option_table: GlobalCell<[KeySetLink; 5]> = GlobalCell::new([
    KeySetLink {
        str: b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 28 as size_t,
        type_0: kObjectTypeBuffer as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 24 as size_t,
        type_0: kObjectTypeWindow as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"scope\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"filetype\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 32 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 4 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn option_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        3 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            98 => {
                low = 0 as ::core::ffi::c_int;
            }
            119 => {
                low = 1 as ::core::ffi::c_int;
            }
            _ => {}
        },
        5 => {
            low = 2 as ::core::ffi::c_int;
        }
        8 => {
            low = 3 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*option_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_option_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = option_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (option_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static highlight_table: GlobalCell<[KeySetLink; 36]> = GlobalCell::new([
    KeySetLink {
        str: b"bg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 152 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"fg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 88 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"sp\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 280 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"dim\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 12 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 4 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"url\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 352 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 5 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"bold\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 10 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 6 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"link\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 312 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 7 as ::core::ffi::c_int,
        is_hlgroup: true_0 != 0,
    },
    KeySetLink {
        str: b"blend\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 336 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 8 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"force\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 346 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 9 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"blink\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 9 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 10 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"cterm\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 32 as size_t,
        type_0: kObjectTypeDict as ::core::ffi::c_int,
        opt_index: 11 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"italic\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 13 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 12 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"update\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 347 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 13 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"reverse\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 16 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 14 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"default\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 24 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 15 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"altfont\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 16 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"conceal\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 11 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 17 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"special\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 248 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 18 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"ctermfg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 184 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 19 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"ctermbg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 216 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 20 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"fallback\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 328 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 21 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"overline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 15 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 22 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"standout\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 17 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 23 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"nocombine\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 14 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 24 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"undercurl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 19 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 25 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"underline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 23 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 26 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"background\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 120 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 27 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"bg_indexed\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 345 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 28 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"foreground\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 56 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 29 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"fg_indexed\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 344 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 30 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"link_global\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 320 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 31 as ::core::ffi::c_int,
        is_hlgroup: true_0 != 0,
    },
    KeySetLink {
        str: b"underdashed\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 20 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 32 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"underdotted\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 21 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 33 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"underdouble\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 22 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 34 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"strikethrough\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 18 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 35 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn highlight_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut high: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    match len {
        2 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            98 => {
                low = 0 as ::core::ffi::c_int;
                high = 1 as ::core::ffi::c_int;
            }
            102 => {
                low = 1 as ::core::ffi::c_int;
                high = 2 as ::core::ffi::c_int;
            }
            115 => {
                low = 2 as ::core::ffi::c_int;
                high = 3 as ::core::ffi::c_int;
            }
            _ => {}
        },
        3 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            100 => {
                low = 3 as ::core::ffi::c_int;
                high = 4 as ::core::ffi::c_int;
            }
            117 => {
                low = 4 as ::core::ffi::c_int;
                high = 5 as ::core::ffi::c_int;
            }
            _ => {}
        },
        4 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            98 => {
                low = 5 as ::core::ffi::c_int;
                high = 6 as ::core::ffi::c_int;
            }
            108 => {
                low = 6 as ::core::ffi::c_int;
                high = 7 as ::core::ffi::c_int;
            }
            _ => {}
        },
        5 => match *str.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            100 => {
                low = 7 as ::core::ffi::c_int;
                high = 8 as ::core::ffi::c_int;
            }
            101 => {
                low = 8 as ::core::ffi::c_int;
                high = 9 as ::core::ffi::c_int;
            }
            107 => {
                low = 9 as ::core::ffi::c_int;
                high = 10 as ::core::ffi::c_int;
            }
            109 => {
                low = 10 as ::core::ffi::c_int;
                high = 11 as ::core::ffi::c_int;
            }
            _ => {}
        },
        6 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            105 => {
                low = 11 as ::core::ffi::c_int;
                high = 12 as ::core::ffi::c_int;
            }
            117 => {
                low = 12 as ::core::ffi::c_int;
                high = 13 as ::core::ffi::c_int;
            }
            _ => {}
        },
        7 => match *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            101 => {
                low = 13 as ::core::ffi::c_int;
                high = 15 as ::core::ffi::c_int;
            }
            108 => {
                low = 15 as ::core::ffi::c_int;
                high = 16 as ::core::ffi::c_int;
            }
            111 => {
                low = 16 as ::core::ffi::c_int;
                high = 17 as ::core::ffi::c_int;
            }
            112 => {
                low = 17 as ::core::ffi::c_int;
                high = 18 as ::core::ffi::c_int;
            }
            116 => {
                low = 18 as ::core::ffi::c_int;
                high = 20 as ::core::ffi::c_int;
            }
            _ => {}
        },
        8 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            102 => {
                low = 20 as ::core::ffi::c_int;
                high = 21 as ::core::ffi::c_int;
            }
            111 => {
                low = 21 as ::core::ffi::c_int;
                high = 22 as ::core::ffi::c_int;
            }
            115 => {
                low = 22 as ::core::ffi::c_int;
                high = 23 as ::core::ffi::c_int;
            }
            _ => {}
        },
        9 => match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            98 => {
                low = 23 as ::core::ffi::c_int;
                high = 24 as ::core::ffi::c_int;
            }
            99 => {
                low = 24 as ::core::ffi::c_int;
                high = 25 as ::core::ffi::c_int;
            }
            108 => {
                low = 25 as ::core::ffi::c_int;
                high = 26 as ::core::ffi::c_int;
            }
            _ => {}
        },
        10 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            98 => {
                low = 26 as ::core::ffi::c_int;
                high = 28 as ::core::ffi::c_int;
            }
            102 => {
                low = 28 as ::core::ffi::c_int;
                high = 30 as ::core::ffi::c_int;
            }
            _ => {}
        },
        11 => match *str.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            111 => {
                low = 30 as ::core::ffi::c_int;
                high = 31 as ::core::ffi::c_int;
            }
            115 => {
                low = 31 as ::core::ffi::c_int;
                high = 32 as ::core::ffi::c_int;
            }
            116 => {
                low = 32 as ::core::ffi::c_int;
                high = 33 as ::core::ffi::c_int;
            }
            117 => {
                low = 33 as ::core::ffi::c_int;
                high = 34 as ::core::ffi::c_int;
            }
            _ => {}
        },
        13 => {
            low = 34 as ::core::ffi::c_int;
            high = 35 as ::core::ffi::c_int;
        }
        _ => {}
    }
    let mut i: ::core::ffi::c_int = low;
    while i < high {
        if memcmp(
            str as *const ::core::ffi::c_void,
            (*highlight_table.ptr())[i as usize].str as *const ::core::ffi::c_void,
            len,
        ) == 0
        {
            return i;
        }
        i += 1;
    }
    return -1 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn KeyDict_highlight_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = highlight_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (highlight_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static highlight_cterm_table: GlobalCell<[KeySetLink; 17]> = GlobalCell::new([
    KeySetLink {
        str: b"dim\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 11 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"bold\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 0 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"blink\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 12 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"italic\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"altfont\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 10 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"conceal\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 13 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"reverse\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 9 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"overline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 14 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"standout\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 1 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"nocombine\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 15 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"undercurl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 4 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"underline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 3 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"underdashed\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 7 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"underdotted\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 6 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"underdouble\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 5 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"strikethrough\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 2 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn highlight_cterm_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        3 => {
            low = 0 as ::core::ffi::c_int;
        }
        4 => {
            low = 1 as ::core::ffi::c_int;
        }
        5 => {
            low = 2 as ::core::ffi::c_int;
        }
        6 => {
            low = 3 as ::core::ffi::c_int;
        }
        7 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            97 => {
                low = 4 as ::core::ffi::c_int;
            }
            99 => {
                low = 5 as ::core::ffi::c_int;
            }
            114 => {
                low = 6 as ::core::ffi::c_int;
            }
            _ => {}
        },
        8 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            111 => {
                low = 7 as ::core::ffi::c_int;
            }
            115 => {
                low = 8 as ::core::ffi::c_int;
            }
            _ => {}
        },
        9 => match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            98 => {
                low = 9 as ::core::ffi::c_int;
            }
            99 => {
                low = 10 as ::core::ffi::c_int;
            }
            108 => {
                low = 11 as ::core::ffi::c_int;
            }
            _ => {}
        },
        11 => match *str.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            115 => {
                low = 12 as ::core::ffi::c_int;
            }
            116 => {
                low = 13 as ::core::ffi::c_int;
            }
            117 => {
                low = 14 as ::core::ffi::c_int;
            }
            _ => {}
        },
        13 => {
            low = 15 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*highlight_cterm_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_highlight_cterm_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = highlight_cterm_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (highlight_cterm_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static get_highlight_table: GlobalCell<[KeySetLink; 5]> = GlobalCell::new([
    KeySetLink {
        str: b"id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"link\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 32 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 16 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"create\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 33 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 4 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn get_highlight_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        2 => {
            low = 0 as ::core::ffi::c_int;
        }
        4 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            108 => {
                low = 1 as ::core::ffi::c_int;
            }
            110 => {
                low = 2 as ::core::ffi::c_int;
            }
            _ => {}
        },
        6 => {
            low = 3 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*get_highlight_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_get_highlight_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = get_highlight_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (get_highlight_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static get_ns_table: GlobalCell<[KeySetLink; 2]> = GlobalCell::new([
    KeySetLink {
        str: b"winid\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeWindow as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn get_ns_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        5 => {
            low = 0 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*get_ns_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_get_ns_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = get_ns_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (get_ns_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static win_text_height_table: GlobalCell<[KeySetLink; 6]> = GlobalCell::new([
    KeySetLink {
        str: b"end_row\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 16 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"end_vcol\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 32 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"start_row\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"max_height\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 40 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 4 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"start_vcol\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 24 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 5 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn win_text_height_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        7 => {
            low = 0 as ::core::ffi::c_int;
        }
        8 => {
            low = 1 as ::core::ffi::c_int;
        }
        9 => {
            low = 2 as ::core::ffi::c_int;
        }
        10 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            109 => {
                low = 3 as ::core::ffi::c_int;
            }
            115 => {
                low = 4 as ::core::ffi::c_int;
            }
            _ => {}
        },
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*win_text_height_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_win_text_height_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = win_text_height_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (win_text_height_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static clear_autocmds_table: GlobalCell<[KeySetLink; 6]> = GlobalCell::new([
    KeySetLink {
        str: b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 12 as size_t,
        type_0: kObjectTypeBuffer as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"event\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 16 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"group\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 48 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"buffer\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeBuffer as ::core::ffi::c_int,
        opt_index: 4 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"pattern\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 80 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 5 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn clear_autocmds_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        3 => {
            low = 0 as ::core::ffi::c_int;
        }
        5 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            101 => {
                low = 1 as ::core::ffi::c_int;
            }
            103 => {
                low = 2 as ::core::ffi::c_int;
            }
            _ => {}
        },
        6 => {
            low = 3 as ::core::ffi::c_int;
        }
        7 => {
            low = 4 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*clear_autocmds_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_clear_autocmds_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = clear_autocmds_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (clear_autocmds_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static create_autocmd_table: GlobalCell<[KeySetLink; 10]> = GlobalCell::new([
    KeySetLink {
        str: b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 12 as size_t,
        type_0: kObjectTypeBuffer as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"desc\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 64 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"once\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 113 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"group\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 80 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 4 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"buffer\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeBuffer as ::core::ffi::c_int,
        opt_index: 5 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"nested\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 112 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 6 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"command\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 48 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 7 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"pattern\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 120 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 8 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"callback\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 16 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 9 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn create_autocmd_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        3 => {
            low = 0 as ::core::ffi::c_int;
        }
        4 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            100 => {
                low = 1 as ::core::ffi::c_int;
            }
            111 => {
                low = 2 as ::core::ffi::c_int;
            }
            _ => {}
        },
        5 => {
            low = 3 as ::core::ffi::c_int;
        }
        6 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            98 => {
                low = 4 as ::core::ffi::c_int;
            }
            110 => {
                low = 5 as ::core::ffi::c_int;
            }
            _ => {}
        },
        7 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 6 as ::core::ffi::c_int;
            }
            112 => {
                low = 7 as ::core::ffi::c_int;
            }
            _ => {}
        },
        8 => {
            low = 8 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*create_autocmd_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_create_autocmd_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = create_autocmd_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (create_autocmd_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static exec_autocmds_table: GlobalCell<[KeySetLink; 7]> = GlobalCell::new([
    KeySetLink {
        str: b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 12 as size_t,
        type_0: kObjectTypeBuffer as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"data\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 88 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"group\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 16 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"buffer\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeBuffer as ::core::ffi::c_int,
        opt_index: 4 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"pattern\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 56 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 5 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"modeline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 48 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 6 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn exec_autocmds_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        3 => {
            low = 0 as ::core::ffi::c_int;
        }
        4 => {
            low = 1 as ::core::ffi::c_int;
        }
        5 => {
            low = 2 as ::core::ffi::c_int;
        }
        6 => {
            low = 3 as ::core::ffi::c_int;
        }
        7 => {
            low = 4 as ::core::ffi::c_int;
        }
        8 => {
            low = 5 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*exec_autocmds_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_exec_autocmds_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = exec_autocmds_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (exec_autocmds_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static get_autocmds_table: GlobalCell<[KeySetLink; 7]> = GlobalCell::new([
    KeySetLink {
        str: b"id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 168 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 136 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"event\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"group\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 40 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 4 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"buffer\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 104 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 5 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"pattern\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 72 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 6 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn get_autocmds_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        2 => {
            low = 0 as ::core::ffi::c_int;
        }
        3 => {
            low = 1 as ::core::ffi::c_int;
        }
        5 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            101 => {
                low = 2 as ::core::ffi::c_int;
            }
            103 => {
                low = 3 as ::core::ffi::c_int;
            }
            _ => {}
        },
        6 => {
            low = 4 as ::core::ffi::c_int;
        }
        7 => {
            low = 5 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*get_autocmds_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_get_autocmds_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = get_autocmds_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (get_autocmds_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static create_augroup_table: GlobalCell<[KeySetLink; 2]> = GlobalCell::new([
    KeySetLink {
        str: b"clear\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn create_augroup_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        5 => {
            low = 0 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*create_augroup_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_create_augroup_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = create_augroup_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (create_augroup_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static cmd_table: GlobalCell<[KeySetLink; 12]> = GlobalCell::new([
    KeySetLink {
        str: b"cmd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"reg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 56 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"bang\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 72 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"addr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 184 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 4 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"mods\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 128 as size_t,
        type_0: kObjectTypeDict as ::core::ffi::c_int,
        opt_index: 5 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"args\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 80 as size_t,
        type_0: kObjectTypeArray as ::core::ffi::c_int,
        opt_index: 6 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"count\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 48 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 7 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"magic\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 104 as size_t,
        type_0: kObjectTypeDict as ::core::ffi::c_int,
        opt_index: 8 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"nargs\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 152 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 9 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"range\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 24 as size_t,
        type_0: kObjectTypeArray as ::core::ffi::c_int,
        opt_index: 10 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"nextcmd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 200 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 11 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn cmd_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        3 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 0 as ::core::ffi::c_int;
            }
            114 => {
                low = 1 as ::core::ffi::c_int;
            }
            _ => {}
        },
        4 => match *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            97 => {
                low = 2 as ::core::ffi::c_int;
            }
            100 => {
                low = 3 as ::core::ffi::c_int;
            }
            111 => {
                low = 4 as ::core::ffi::c_int;
            }
            114 => {
                low = 5 as ::core::ffi::c_int;
            }
            _ => {}
        },
        5 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 6 as ::core::ffi::c_int;
            }
            109 => {
                low = 7 as ::core::ffi::c_int;
            }
            110 => {
                low = 8 as ::core::ffi::c_int;
            }
            114 => {
                low = 9 as ::core::ffi::c_int;
            }
            _ => {}
        },
        7 => {
            low = 10 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*cmd_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_cmd_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = cmd_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (cmd_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static cmd_magic_table: GlobalCell<[KeySetLink; 3]> = GlobalCell::new([
    KeySetLink {
        str: b"bar\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 9 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"file\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn cmd_magic_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        3 => {
            low = 0 as ::core::ffi::c_int;
        }
        4 => {
            low = 1 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*cmd_magic_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_cmd_magic_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = cmd_magic_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (cmd_magic_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static cmd_mods_table: GlobalCell<[KeySetLink; 21]> = GlobalCell::new([
    KeySetLink {
        str: b"tab\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 56 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"hide\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 44 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"split\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 80 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"browse\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 42 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 4 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"filter\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 16 as size_t,
        type_0: kObjectTypeDict as ::core::ffi::c_int,
        opt_index: 5 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"silent\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 6 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"confirm\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 43 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 7 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"keepalt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 46 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 8 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"sandbox\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 40 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 9 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"verbose\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 64 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 10 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"unsilent\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 10 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 11 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"vertical\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 72 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 12 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"keepjumps\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 47 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 13 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"keepmarks\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 48 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 14 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"lockmarks\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 50 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 15 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"noautocmd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 41 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 16 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"horizontal\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 45 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 17 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"noswapfile\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 51 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 18 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"emsg_silent\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 9 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 19 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"keeppatterns\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 49 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 20 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn cmd_mods_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut high: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    match len {
        3 => {
            low = 0 as ::core::ffi::c_int;
            high = 1 as ::core::ffi::c_int;
        }
        4 => {
            low = 1 as ::core::ffi::c_int;
            high = 2 as ::core::ffi::c_int;
        }
        5 => {
            low = 2 as ::core::ffi::c_int;
            high = 3 as ::core::ffi::c_int;
        }
        6 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            98 => {
                low = 3 as ::core::ffi::c_int;
                high = 4 as ::core::ffi::c_int;
            }
            102 => {
                low = 4 as ::core::ffi::c_int;
                high = 5 as ::core::ffi::c_int;
            }
            115 => {
                low = 5 as ::core::ffi::c_int;
                high = 6 as ::core::ffi::c_int;
            }
            _ => {}
        },
        7 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 6 as ::core::ffi::c_int;
                high = 7 as ::core::ffi::c_int;
            }
            107 => {
                low = 7 as ::core::ffi::c_int;
                high = 8 as ::core::ffi::c_int;
            }
            115 => {
                low = 8 as ::core::ffi::c_int;
                high = 9 as ::core::ffi::c_int;
            }
            118 => {
                low = 9 as ::core::ffi::c_int;
                high = 10 as ::core::ffi::c_int;
            }
            _ => {}
        },
        8 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            117 => {
                low = 10 as ::core::ffi::c_int;
                high = 11 as ::core::ffi::c_int;
            }
            118 => {
                low = 11 as ::core::ffi::c_int;
                high = 12 as ::core::ffi::c_int;
            }
            _ => {}
        },
        9 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            107 => {
                low = 12 as ::core::ffi::c_int;
                high = 14 as ::core::ffi::c_int;
            }
            108 => {
                low = 14 as ::core::ffi::c_int;
                high = 15 as ::core::ffi::c_int;
            }
            110 => {
                low = 15 as ::core::ffi::c_int;
                high = 16 as ::core::ffi::c_int;
            }
            _ => {}
        },
        10 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            104 => {
                low = 16 as ::core::ffi::c_int;
                high = 17 as ::core::ffi::c_int;
            }
            110 => {
                low = 17 as ::core::ffi::c_int;
                high = 18 as ::core::ffi::c_int;
            }
            _ => {}
        },
        11 => {
            low = 18 as ::core::ffi::c_int;
            high = 19 as ::core::ffi::c_int;
        }
        12 => {
            low = 19 as ::core::ffi::c_int;
            high = 20 as ::core::ffi::c_int;
        }
        _ => {}
    }
    let mut i: ::core::ffi::c_int = low;
    while i < high {
        if memcmp(
            str as *const ::core::ffi::c_void,
            (*cmd_mods_table.ptr())[i as usize].str as *const ::core::ffi::c_void,
            len,
        ) == 0
        {
            return i;
        }
        i += 1;
    }
    return -1 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn KeyDict_cmd_mods_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = cmd_mods_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (cmd_mods_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static cmd_mods_filter_table: GlobalCell<[KeySetLink; 3]> = GlobalCell::new([
    KeySetLink {
        str: b"force\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 24 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"pattern\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn cmd_mods_filter_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        5 => {
            low = 0 as ::core::ffi::c_int;
        }
        7 => {
            low = 1 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*cmd_mods_filter_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_cmd_mods_filter_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = cmd_mods_filter_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (cmd_mods_filter_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static cmd_opts_table: GlobalCell<[KeySetLink; 2]> = GlobalCell::new([
    KeySetLink {
        str: b"output\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 0 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn cmd_opts_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        6 => {
            low = 0 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*cmd_opts_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_cmd_opts_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = cmd_opts_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (cmd_opts_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static echo_opts_table: GlobalCell<[KeySetLink; 11]> = GlobalCell::new([
    KeySetLink {
        str: b"id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 32 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"err\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"data\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 120 as size_t,
        type_0: kObjectTypeDict as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"kind\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 16 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 4 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"title\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 64 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 5 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"source\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 104 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 6 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"status\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 80 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 7 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"percent\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 96 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 8 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"verbose\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 9 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 9 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"_truncate\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 10 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 10 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn echo_opts_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        2 => {
            low = 0 as ::core::ffi::c_int;
        }
        3 => {
            low = 1 as ::core::ffi::c_int;
        }
        4 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            100 => {
                low = 2 as ::core::ffi::c_int;
            }
            107 => {
                low = 3 as ::core::ffi::c_int;
            }
            _ => {}
        },
        5 => {
            low = 4 as ::core::ffi::c_int;
        }
        6 => match *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            111 => {
                low = 5 as ::core::ffi::c_int;
            }
            116 => {
                low = 6 as ::core::ffi::c_int;
            }
            _ => {}
        },
        7 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            112 => {
                low = 7 as ::core::ffi::c_int;
            }
            118 => {
                low = 8 as ::core::ffi::c_int;
            }
            _ => {}
        },
        9 => {
            low = 9 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*echo_opts_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_echo_opts_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = echo_opts_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (echo_opts_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static exec_opts_table: GlobalCell<[KeySetLink; 2]> = GlobalCell::new([
    KeySetLink {
        str: b"output\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 0 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn exec_opts_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        6 => {
            low = 0 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*exec_opts_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_exec_opts_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = exec_opts_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (exec_opts_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static buf_attach_table: GlobalCell<[KeySetLink; 8]> = GlobalCell::new([
    KeySetLink {
        str: b"preview\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 29 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"on_bytes\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 12 as size_t,
        type_0: kObjectTypeLuaRef as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"on_lines\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeLuaRef as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"utf_sizes\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 28 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 4 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"on_detach\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 20 as size_t,
        type_0: kObjectTypeLuaRef as ::core::ffi::c_int,
        opt_index: 5 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"on_reload\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 24 as size_t,
        type_0: kObjectTypeLuaRef as ::core::ffi::c_int,
        opt_index: 6 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"on_changedtick\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 16 as size_t,
        type_0: kObjectTypeLuaRef as ::core::ffi::c_int,
        opt_index: 7 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn buf_attach_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        7 => {
            low = 0 as ::core::ffi::c_int;
        }
        8 => match *str.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            98 => {
                low = 1 as ::core::ffi::c_int;
            }
            108 => {
                low = 2 as ::core::ffi::c_int;
            }
            _ => {}
        },
        9 => match *str.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 3 as ::core::ffi::c_int;
            }
            100 => {
                low = 4 as ::core::ffi::c_int;
            }
            114 => {
                low = 5 as ::core::ffi::c_int;
            }
            _ => {}
        },
        14 => {
            low = 6 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*buf_attach_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_buf_attach_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = buf_attach_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (buf_attach_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static buf_delete_table: GlobalCell<[KeySetLink; 3]> = GlobalCell::new([
    KeySetLink {
        str: b"force\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"unload\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 9 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn buf_delete_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        5 => {
            low = 0 as ::core::ffi::c_int;
        }
        6 => {
            low = 1 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*buf_delete_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_buf_delete_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = buf_delete_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (buf_delete_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static open_term_table: GlobalCell<[KeySetLink; 3]> = GlobalCell::new([
    KeySetLink {
        str: b"on_input\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeLuaRef as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"force_crlf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 12 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn open_term_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        8 => {
            low = 0 as ::core::ffi::c_int;
        }
        10 => {
            low = 1 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*open_term_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_open_term_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = open_term_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (open_term_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static complete_set_table: GlobalCell<[KeySetLink; 2]> = GlobalCell::new([
    KeySetLink {
        str: b"info\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn complete_set_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        4 => {
            low = 0 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*complete_set_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_complete_set_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = complete_set_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (complete_set_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static xdl_diff_table: GlobalCell<[KeySetLink; 13]> = GlobalCell::new([
    KeySetLink {
        str: b"ctxlen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 48 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"on_hunk\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeLuaRef as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"algorithm\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 32 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"linematch\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 64 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: 4 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"result_type\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 16 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 5 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"interhunkctxlen\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        ptr_off: 56 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 6 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"ignore_cr_at_eol\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        ptr_off: 99 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 7 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"indent_heuristic\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        ptr_off: 101 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 8 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"ignore_whitespace\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        ptr_off: 96 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 9 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"ignore_blank_lines\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        ptr_off: 100 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 10 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"ignore_whitespace_change\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        ptr_off: 97 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 11 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"ignore_whitespace_change_at_eol\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        ptr_off: 98 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 12 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn xdl_diff_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        6 => {
            low = 0 as ::core::ffi::c_int;
        }
        7 => {
            low = 1 as ::core::ffi::c_int;
        }
        9 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            97 => {
                low = 2 as ::core::ffi::c_int;
            }
            108 => {
                low = 3 as ::core::ffi::c_int;
            }
            _ => {}
        },
        11 => {
            low = 4 as ::core::ffi::c_int;
        }
        15 => {
            low = 5 as ::core::ffi::c_int;
        }
        16 => match *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            103 => {
                low = 6 as ::core::ffi::c_int;
            }
            110 => {
                low = 7 as ::core::ffi::c_int;
            }
            _ => {}
        },
        17 => {
            low = 8 as ::core::ffi::c_int;
        }
        18 => {
            low = 9 as ::core::ffi::c_int;
        }
        24 => {
            low = 10 as ::core::ffi::c_int;
        }
        31 => {
            low = 11 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*xdl_diff_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_xdl_diff_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = xdl_diff_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (xdl_diff_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static redraw_table: GlobalCell<[KeySetLink; 11]> = GlobalCell::new([
    KeySetLink {
        str: b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 44 as size_t,
        type_0: kObjectTypeBuffer as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 40 as size_t,
        type_0: kObjectTypeWindow as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"flush\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"range\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 16 as size_t,
        type_0: kObjectTypeArray as ::core::ffi::c_int,
        opt_index: 4 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"valid\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 10 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 5 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"cursor\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 9 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 6 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"winbar\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 14 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 7 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"tabline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 13 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 8 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"statusline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 12 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 9 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"statuscolumn\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 11 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 10 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn redraw_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        3 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            98 => {
                low = 0 as ::core::ffi::c_int;
            }
            119 => {
                low = 1 as ::core::ffi::c_int;
            }
            _ => {}
        },
        5 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            102 => {
                low = 2 as ::core::ffi::c_int;
            }
            114 => {
                low = 3 as ::core::ffi::c_int;
            }
            118 => {
                low = 4 as ::core::ffi::c_int;
            }
            _ => {}
        },
        6 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 5 as ::core::ffi::c_int;
            }
            119 => {
                low = 6 as ::core::ffi::c_int;
            }
            _ => {}
        },
        7 => {
            low = 7 as ::core::ffi::c_int;
        }
        10 => {
            low = 8 as ::core::ffi::c_int;
        }
        12 => {
            low = 9 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*redraw_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_redraw_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = redraw_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (redraw_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static ns_opts_table: GlobalCell<[KeySetLink; 2]> = GlobalCell::new([
    KeySetLink {
        str: b"wins\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeArray as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn ns_opts_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        4 => {
            low = 0 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*ns_opts_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict_ns_opts_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = ns_opts_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (ns_opts_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static _shada_search_pat_table: GlobalCell<[KeySetLink; 11]> = GlobalCell::new([
    KeySetLink {
        str: b"sb\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 15 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"sc\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 9 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"se\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 11 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"sh\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 14 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 4 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"sl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 10 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 5 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"sm\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 6 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"so\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 16 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 7 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"sp\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 24 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 8 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"ss\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 13 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 9 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"su\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 12 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 10 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn _shada_search_pat_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        2 => match *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            98 => {
                low = 0 as ::core::ffi::c_int;
            }
            99 => {
                low = 1 as ::core::ffi::c_int;
            }
            101 => {
                low = 2 as ::core::ffi::c_int;
            }
            104 => {
                low = 3 as ::core::ffi::c_int;
            }
            108 => {
                low = 4 as ::core::ffi::c_int;
            }
            109 => {
                low = 5 as ::core::ffi::c_int;
            }
            111 => {
                low = 6 as ::core::ffi::c_int;
            }
            112 => {
                low = 7 as ::core::ffi::c_int;
            }
            115 => {
                low = 8 as ::core::ffi::c_int;
            }
            117 => {
                low = 9 as ::core::ffi::c_int;
            }
            _ => {}
        },
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*_shada_search_pat_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict__shada_search_pat_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = _shada_search_pat_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (_shada_search_pat_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static _shada_mark_table: GlobalCell<[KeySetLink; 5]> = GlobalCell::new([
    KeySetLink {
        str: b"c\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 24 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"f\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 32 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"l\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 16 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"n\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 4 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn _shada_mark_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    match len {
        1 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => return 0 as ::core::ffi::c_int,
            102 => return 1 as ::core::ffi::c_int,
            108 => return 2 as ::core::ffi::c_int,
            110 => return 3 as ::core::ffi::c_int,
            _ => {}
        },
        _ => {}
    }
    return -1 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn KeyDict__shada_mark_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = _shada_mark_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (_shada_mark_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static _shada_register_table: GlobalCell<[KeySetLink; 6]> = GlobalCell::new([
    KeySetLink {
        str: b"n\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 48 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"rc\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kUnpackTypeStringArray as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"rt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 40 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"ru\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 32 as size_t,
        type_0: kObjectTypeBoolean as ::core::ffi::c_int,
        opt_index: 4 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"rw\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 56 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 5 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn _shada_register_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        1 => {
            low = 0 as ::core::ffi::c_int;
        }
        2 => match *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 1 as ::core::ffi::c_int;
            }
            116 => {
                low = 2 as ::core::ffi::c_int;
            }
            117 => {
                low = 3 as ::core::ffi::c_int;
            }
            119 => {
                low = 4 as ::core::ffi::c_int;
            }
            _ => {}
        },
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*_shada_register_table.ptr())[low as usize].str as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub unsafe extern "C" fn KeyDict__shada_register_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = _shada_register_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (_shada_register_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static _shada_buflist_item_table: GlobalCell<[KeySetLink; 4]> = GlobalCell::new([
    KeySetLink {
        str: b"c\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 16 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"f\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 24 as size_t,
        type_0: kObjectTypeString as ::core::ffi::c_int,
        opt_index: 2 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: b"l\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ptr_off: 8 as size_t,
        type_0: kObjectTypeInteger as ::core::ffi::c_int,
        opt_index: 3 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
    KeySetLink {
        str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr_off: 0 as size_t,
        type_0: kObjectTypeNil as ::core::ffi::c_int,
        opt_index: -1 as ::core::ffi::c_int,
        is_hlgroup: false_0 != 0,
    },
]);
pub unsafe extern "C" fn _shada_buflist_item_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    match len {
        1 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => return 0 as ::core::ffi::c_int,
            102 => return 1 as ::core::ffi::c_int,
            108 => return 2 as ::core::ffi::c_int,
            _ => {}
        },
        _ => {}
    }
    return -1 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn KeyDict__shada_buflist_item_get_field(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut KeySetLink {
    let mut hash: ::core::ffi::c_int = _shada_buflist_item_hash(str, len);
    if hash == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<KeySetLink>();
    }
    return (_shada_buflist_item_table.ptr() as *mut KeySetLink).offset(hash as isize);
}
pub static method_handlers: GlobalCell<[MsgpackRpcRequestHandler; 280]> = GlobalCell::new([
    MsgpackRpcRequestHandler {
        name: b"redraw\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_ui_client_redraw
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: true_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim__id\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim__id
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_cmd\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_cmd
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_put\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_put
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_eval\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_eval
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"ui_attach\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_ui_attach
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"ui_detach\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_ui_detach
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_echo\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_echo
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_input\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_input
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: true_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_eval\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_eval
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_exec\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_exec
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: true_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_exec2\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_exec2
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: true_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_input\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_input
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: true_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_paste\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_paste
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_hl\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_hl
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_set_hl\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_set_hl
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_del_var\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_vim_del_var
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_command\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_command
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_notify\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_notify
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim__stats\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim__stats
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_get_var\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_var
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_set_var\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_vim_set_var
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_ui_send\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_ui_send
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_get_vvar\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_vvar
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_feedkeys\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_feedkeys
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim__redraw\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim__redraw
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_del_var\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_del_var
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_command\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_command
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim__unpack\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim__unpack
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: true_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim__ns_get\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim__ns_get
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim__ns_set\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim__ns_set
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_var\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_var
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_set_var\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_set_var
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_strwidth\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_strwidth
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_exec_lua\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_exec_lua
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_list_uis\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_list_uis
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_open_win\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_open_win
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim__id_dict\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim__id_dict
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_win_hide\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_hide
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_strwidth\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_strwidth
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_feedkeys\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_feedkeys
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_del_mark\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_del_mark
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_mark\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_mark
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_mode\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_mode
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: true_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_proc\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_proc
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_err_write\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_err_write
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_out_write\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_out_write
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_subscribe\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_subscribe
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"buffer_insert\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_buffer_insert
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"ui_try_resize\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_ui_try_resize
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_vvar\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_vvar
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_set_vvar\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_set_vvar
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim__id_array\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim__id_array
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim__id_float\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim__id_float
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_chan_send\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_chan_send
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_err_write\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_err_write
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_get_option\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_option
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: true_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_set_option\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_set_option
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_hl_ns\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_hl_ns
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_list_bufs\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_list_bufs
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_list_wins\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_list_wins
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_open_term\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_open_term
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_out_write\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_out_write
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_parse_cmd\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_parse_cmd
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: true_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"buffer_del_var\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_buffer_del_var
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"buffer_get_var\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_get_var
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"buffer_set_var\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_buffer_set_var
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_set_hl_ns\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_set_hl_ns
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_subscribe\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_subscribe
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_ui_attach\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_ui_attach
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_ui_detach\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_ui_detach
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_win_close\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_close
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"window_del_var\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_window_del_var
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"window_get_var\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_get_var
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"window_set_var\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_window_set_var
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"tabpage_del_var\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_tabpage_del_var
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"tabpage_get_var\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_tabpage_get_var
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"tabpage_set_var\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_tabpage_set_var
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_get_buffers\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_list_bufs
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_get_windows\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_list_wins
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"buffer_del_line\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_buffer_del_line
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_create_buf\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_create_buf
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_attach\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_attach
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_delete\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_delete
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_detach\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_detach
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"buffer_get_line\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_buffer_get_line
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"buffer_get_mark\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_get_mark
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"buffer_get_name\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_get_name
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"buffer_is_valid\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_is_valid
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"window_is_valid\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_is_valid
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_del_keymap\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_del_keymap
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"buffer_set_line\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_buffer_set_line
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"buffer_set_name\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_set_name
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_list_chans\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_list_chans
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_keymap\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_keymap
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_option\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_option
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: true_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_set_keymap\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_set_keymap
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_set_option\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_set_option
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim__buf_stats\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim__buf_stats
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_unsubscribe\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_unsubscribe
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"buffer_get_lines\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_get_lines
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"buffer_set_lines\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_set_lines
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"window_get_width\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_get_width
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"window_set_width\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_set_width
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_call_atomic\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_call_atomic
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_del_autocmd\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_del_autocmd
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_context\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_context
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"tabpage_is_valid\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_tabpage_is_valid
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_win_del_var\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_del_var
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_win_get_buf\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_get_buf
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_win_get_var\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_get_var
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_win_set_buf\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_set_buf
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_win_set_var\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_set_var
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_input_mouse\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_input_mouse
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: true_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_unsubscribe\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_unsubscribe
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_report_error\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_err_writeln
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_err_writeln\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_err_writeln
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_error_event\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_error_event
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim__screenshot\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim__screenshot
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: true_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_get_api_info\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_api_info
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: true_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_get_tabpages\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_list_tabpages
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_del_var\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_del_var
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_get_var\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_get_var
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_set_var\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_set_var
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_execute_lua\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_execute_lua
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_hl_by_id\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_hl_by_id
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_del_mark\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_del_mark
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_get_mark\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_get_mark
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_get_name\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_get_name
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_set_mark\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_set_mark
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_set_name\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_set_name
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_open_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_open_tabpage
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"buffer_get_number\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_get_number
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_ui_set_focus\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_ui_set_focus
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim__get_lib_dir\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim__get_lib_dir
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: true_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_get_text\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_get_text
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_set_text\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_set_text
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_load_context\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_load_context
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"window_get_buffer\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_get_buf
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"window_get_height\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_get_height
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"window_set_height\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_set_height
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"buffer_get_option\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_get_option
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: true_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"buffer_set_option\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_set_option
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim__get_runtime\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim__get_runtime
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: true_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_call_function\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_call_function
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"window_get_option\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_get_option
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: true_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"window_set_option\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_set_option
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_is_valid\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_is_valid
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_win_is_valid\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_is_valid
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_name_to_color\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_color_by_name
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_autocmds\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_autocmds
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_get_color_map\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_color_map
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_api_info\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_api_info
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: true_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_commands\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_commands
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"window_get_cursor\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_get_cursor
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"window_set_cursor\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_set_cursor
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"buffer_line_count\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_line_count
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_call_function\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_call_function
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_exec_autocmds\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_exec_autocmds
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_list_tabpages\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_list_tabpages
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_chan_info\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_chan_info
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_color_map\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_color_map
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_ui_set_option\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_ui_set_option
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_ui_term_event\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_ui_term_event
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"tabpage_get_window\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_tabpage_get_win
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_get_lines\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_get_lines
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_win_get_width\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_get_width
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_is_loaded\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_is_loaded
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_command_output\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_command_output
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: true_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim__complete_set\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim__complete_set
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim__inspect_cell\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim__inspect_cell
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_ui_try_resize\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_ui_try_resize
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_set_lines\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_set_lines
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_win_set_hl_ns\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_set_hl_ns
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_win_set_width\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_set_width
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"window_get_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_get_tabpage
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim__exec_lua_fast\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim__exec_lua_fast
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: true_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_hl_by_name\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_hl_by_name
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_set_hl_ns_fast\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_set_hl_ns_fast
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: true_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_line_count\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_line_count
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_del_keymap\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_del_keymap
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_get_keymap\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_get_keymap
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_set_keymap\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_set_keymap
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_win_get_height\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_get_height
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_win_set_height\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_set_height
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_get_offset\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_get_offset
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_create_augroup\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_create_augroup
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"window_get_position\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_get_position
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"tabpage_get_windows\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_tabpage_list_wins
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_clear_autocmds\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_clear_autocmds
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_win_get_config\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_get_config
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_win_set_config\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_set_config
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_get_option\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_get_option
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: true_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_set_option\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_set_option
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_namespaces\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_namespaces
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_win_get_option\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_get_option
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: true_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_win_set_option\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_set_option
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_create_autocmd\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_create_autocmd
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_get_number\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_get_number
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_command_output\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_command_output
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: true_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_win_get_cursor\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_get_cursor
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_win_get_number\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_get_number
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_win_set_cursor\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_set_cursor
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_del_extmark\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_del_extmark
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_set_extmark\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_set_extmark
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_win_get_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_get_tabpage
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_current_buf\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_current_buf
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_set_current_buf\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_set_current_buf
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_set_current_dir\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_set_current_dir
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"buffer_add_highlight\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_add_highlight
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_win_text_height\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_text_height
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_eval_statusline\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_eval_statusline
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: true_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_del_current_line\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_del_current_line
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_get_current_line\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_current_line
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_set_current_line\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_set_current_line
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_option_info\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_option_info
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_set_client_info\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_set_client_info
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_change_directory\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_set_current_dir
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_tabpage_del_var\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_tabpage_del_var
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_tabpage_get_var\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_tabpage_get_var
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_tabpage_set_var\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_tabpage_set_var
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_current_win\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_current_win
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_set_current_win\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_set_current_win
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_tabpage_get_win\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_tabpage_get_win
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_tabpage_set_win\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_tabpage_set_win
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim__chan_set_detach\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim__chan_set_detach
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim__runtime_inspect\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim__runtime_inspect
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_get_commands\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_get_commands
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_get_extmarks\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_get_extmarks
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_create_namespace\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_create_namespace
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_del_current_line\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_del_current_line
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_del_user_command\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_del_user_command
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_replace_termcodes\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_replace_termcodes
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: true_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_current_line\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_current_line
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_option_info2\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_option_info2
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_option_value\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_option_value
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: true_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_runtime_file\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_runtime_file
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: true_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_parse_expression\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_parse_expression
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: true_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"buffer_get_line_slice\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_buffer_get_line_slice
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"buffer_set_line_slice\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_buffer_set_line_slice
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_set_current_line\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_set_current_line
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_set_option_value\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_set_option_value
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_tabpage_is_valid\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_tabpage_is_valid
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_win_get_position\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_win_get_position
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_del_augroup_by_id\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_del_augroup_by_id
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_list_runtime_paths\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_list_runtime_paths
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_ui_pum_set_bounds\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_ui_pum_set_bounds
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_get_current_buffer\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_current_buf
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_set_current_buffer\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_set_current_buf
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"buffer_clear_highlight\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_clear_highlight
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_add_highlight\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_add_highlight
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_ui_pum_set_height\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_ui_pum_set_height
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_proc_children\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_proc_children
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_replace_termcodes\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_replace_termcodes
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: true_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_tabpage_list_wins\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_tabpage_list_wins
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_get_current_window\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_current_win
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_set_current_window\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_set_current_win
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_color_by_name\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_color_by_name
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_hl_id_by_name\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_hl_id_by_name
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_call_dict_function\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_call_dict_function
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_get_current_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_current_tabpage
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"vim_set_current_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_set_current_tabpage
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_list_runtime_paths\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_list_runtime_paths
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_tabpage_get_number\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_tabpage_get_number
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_ui_try_resize_grid\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_ui_try_resize_grid
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_get_changedtick\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_get_changedtick
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_current_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_current_tabpage
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_set_current_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_set_current_tabpage
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_del_augroup_by_name\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_del_augroup_by_name
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_clear_highlight\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_clear_highlight
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_clear_namespace\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_clear_namespace
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_create_user_command\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_create_user_command
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim__buf_debug_extmarks\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim__buf_debug_extmarks
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: true_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_get_all_options_info\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_get_all_options_info
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_del_user_command\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_del_user_command
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_set_virtual_text\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_set_virtual_text
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_get_extmark_by_id\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_get_extmark_by_id
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_select_popupmenu_item\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_select_popupmenu_item
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim__invalidate_glyph_cache\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim__invalidate_glyph_cache
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
    MsgpackRpcRequestHandler {
        name: b"nvim_buf_create_user_command\0".as_ptr() as *const ::core::ffi::c_char,
        fn_0: Some(
            handle_nvim_buf_create_user_command
                as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
        ),
        fast: false_0 != 0,
        ret_alloc: false_0 != 0,
    },
]);
pub unsafe extern "C" fn msgpack_rpc_get_handler_for_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut high: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    match len {
        6 => {
            low = 0 as ::core::ffi::c_int;
            high = 1 as ::core::ffi::c_int;
        }
        8 => match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 1 as ::core::ffi::c_int;
                high = 2 as ::core::ffi::c_int;
            }
            99 => {
                low = 2 as ::core::ffi::c_int;
                high = 3 as ::core::ffi::c_int;
            }
            112 => {
                low = 3 as ::core::ffi::c_int;
                high = 4 as ::core::ffi::c_int;
            }
            118 => {
                low = 4 as ::core::ffi::c_int;
                high = 5 as ::core::ffi::c_int;
            }
            _ => {}
        },
        9 => match *str.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            97 => {
                low = 5 as ::core::ffi::c_int;
                high = 7 as ::core::ffi::c_int;
            }
            99 => {
                low = 7 as ::core::ffi::c_int;
                high = 8 as ::core::ffi::c_int;
            }
            112 => {
                low = 8 as ::core::ffi::c_int;
                high = 9 as ::core::ffi::c_int;
            }
            118 => {
                low = 9 as ::core::ffi::c_int;
                high = 10 as ::core::ffi::c_int;
            }
            120 => {
                low = 10 as ::core::ffi::c_int;
                high = 11 as ::core::ffi::c_int;
            }
            _ => {}
        },
        10 => match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            101 => {
                low = 11 as ::core::ffi::c_int;
                high = 12 as ::core::ffi::c_int;
            }
            105 => {
                low = 12 as ::core::ffi::c_int;
                high = 13 as ::core::ffi::c_int;
            }
            112 => {
                low = 13 as ::core::ffi::c_int;
                high = 14 as ::core::ffi::c_int;
            }
            _ => {}
        },
        11 => match *str.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            101 => {
                low = 14 as ::core::ffi::c_int;
                high = 16 as ::core::ffi::c_int;
            }
            108 => {
                low = 16 as ::core::ffi::c_int;
                high = 17 as ::core::ffi::c_int;
            }
            109 => {
                low = 17 as ::core::ffi::c_int;
                high = 18 as ::core::ffi::c_int;
            }
            111 => {
                low = 18 as ::core::ffi::c_int;
                high = 19 as ::core::ffi::c_int;
            }
            115 => {
                low = 19 as ::core::ffi::c_int;
                high = 20 as ::core::ffi::c_int;
            }
            116 => {
                low = 20 as ::core::ffi::c_int;
                high = 22 as ::core::ffi::c_int;
            }
            _ => {}
        },
        12 => match *str.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 22 as ::core::ffi::c_int;
                high = 24 as ::core::ffi::c_int;
            }
            100 => {
                low = 24 as ::core::ffi::c_int;
                high = 25 as ::core::ffi::c_int;
            }
            101 => {
                low = 25 as ::core::ffi::c_int;
                high = 26 as ::core::ffi::c_int;
            }
            108 => {
                low = 26 as ::core::ffi::c_int;
                high = 27 as ::core::ffi::c_int;
            }
            109 => {
                low = 27 as ::core::ffi::c_int;
                high = 28 as ::core::ffi::c_int;
            }
            110 => {
                low = 28 as ::core::ffi::c_int;
                high = 29 as ::core::ffi::c_int;
            }
            115 => {
                low = 29 as ::core::ffi::c_int;
                high = 31 as ::core::ffi::c_int;
            }
            116 => {
                low = 31 as ::core::ffi::c_int;
                high = 33 as ::core::ffi::c_int;
            }
            119 => {
                low = 33 as ::core::ffi::c_int;
                high = 34 as ::core::ffi::c_int;
            }
            _ => {}
        },
        13 => match *str.offset(9 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 34 as ::core::ffi::c_int;
                high = 37 as ::core::ffi::c_int;
            }
            100 => {
                low = 37 as ::core::ffi::c_int;
                high = 38 as ::core::ffi::c_int;
            }
            104 => {
                low = 38 as ::core::ffi::c_int;
                high = 39 as ::core::ffi::c_int;
            }
            105 => {
                low = 39 as ::core::ffi::c_int;
                high = 40 as ::core::ffi::c_int;
            }
            107 => {
                low = 40 as ::core::ffi::c_int;
                high = 41 as ::core::ffi::c_int;
            }
            109 => {
                low = 41 as ::core::ffi::c_int;
                high = 44 as ::core::ffi::c_int;
            }
            112 => {
                low = 44 as ::core::ffi::c_int;
                high = 45 as ::core::ffi::c_int;
            }
            114 => {
                low = 45 as ::core::ffi::c_int;
                high = 48 as ::core::ffi::c_int;
            }
            115 => {
                low = 48 as ::core::ffi::c_int;
                high = 50 as ::core::ffi::c_int;
            }
            118 => {
                low = 50 as ::core::ffi::c_int;
                high = 52 as ::core::ffi::c_int;
            }
            _ => {}
        },
        14 => match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 52 as ::core::ffi::c_int;
                high = 54 as ::core::ffi::c_int;
            }
            99 => {
                low = 54 as ::core::ffi::c_int;
                high = 55 as ::core::ffi::c_int;
            }
            101 => {
                low = 55 as ::core::ffi::c_int;
                high = 58 as ::core::ffi::c_int;
            }
            103 => {
                low = 58 as ::core::ffi::c_int;
                high = 59 as ::core::ffi::c_int;
            }
            108 => {
                low = 59 as ::core::ffi::c_int;
                high = 61 as ::core::ffi::c_int;
            }
            111 => {
                low = 61 as ::core::ffi::c_int;
                high = 63 as ::core::ffi::c_int;
            }
            112 => {
                low = 63 as ::core::ffi::c_int;
                high = 64 as ::core::ffi::c_int;
            }
            114 => {
                low = 64 as ::core::ffi::c_int;
                high = 67 as ::core::ffi::c_int;
            }
            115 => {
                low = 67 as ::core::ffi::c_int;
                high = 69 as ::core::ffi::c_int;
            }
            117 => {
                low = 69 as ::core::ffi::c_int;
                high = 71 as ::core::ffi::c_int;
            }
            119 => {
                low = 71 as ::core::ffi::c_int;
                high = 75 as ::core::ffi::c_int;
            }
            _ => {}
        },
        15 => match *str.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 75 as ::core::ffi::c_int;
                high = 80 as ::core::ffi::c_int;
            }
            100 => {
                low = 80 as ::core::ffi::c_int;
                high = 81 as ::core::ffi::c_int;
            }
            101 => {
                low = 81 as ::core::ffi::c_int;
                high = 82 as ::core::ffi::c_int;
            }
            102 => {
                low = 82 as ::core::ffi::c_int;
                high = 85 as ::core::ffi::c_int;
            }
            103 => {
                low = 85 as ::core::ffi::c_int;
                high = 88 as ::core::ffi::c_int;
            }
            105 => {
                low = 88 as ::core::ffi::c_int;
                high = 90 as ::core::ffi::c_int;
            }
            108 => {
                low = 90 as ::core::ffi::c_int;
                high = 91 as ::core::ffi::c_int;
            }
            115 => {
                low = 91 as ::core::ffi::c_int;
                high = 94 as ::core::ffi::c_int;
            }
            116 => {
                low = 94 as ::core::ffi::c_int;
                high = 98 as ::core::ffi::c_int;
            }
            117 => {
                low = 98 as ::core::ffi::c_int;
                high = 100 as ::core::ffi::c_int;
            }
            _ => {}
        },
        16 => match *str.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 100 as ::core::ffi::c_int;
                high = 104 as ::core::ffi::c_int;
            }
            97 => {
                low = 104 as ::core::ffi::c_int;
                high = 105 as ::core::ffi::c_int;
            }
            101 => {
                low = 105 as ::core::ffi::c_int;
                high = 108 as ::core::ffi::c_int;
            }
            105 => {
                low = 108 as ::core::ffi::c_int;
                high = 113 as ::core::ffi::c_int;
            }
            110 => {
                low = 113 as ::core::ffi::c_int;
                high = 115 as ::core::ffi::c_int;
            }
            112 => {
                low = 115 as ::core::ffi::c_int;
                high = 116 as ::core::ffi::c_int;
            }
            114 => {
                low = 116 as ::core::ffi::c_int;
                high = 118 as ::core::ffi::c_int;
            }
            115 => {
                low = 118 as ::core::ffi::c_int;
                high = 119 as ::core::ffi::c_int;
            }
            116 => {
                low = 119 as ::core::ffi::c_int;
                high = 121 as ::core::ffi::c_int;
            }
            117 => {
                low = 121 as ::core::ffi::c_int;
                high = 124 as ::core::ffi::c_int;
            }
            120 => {
                low = 124 as ::core::ffi::c_int;
                high = 125 as ::core::ffi::c_int;
            }
            _ => {}
        },
        17 => match *str.offset(14 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 125 as ::core::ffi::c_int;
                high = 126 as ::core::ffi::c_int;
            }
            97 => {
                low = 126 as ::core::ffi::c_int;
                high = 132 as ::core::ffi::c_int;
            }
            98 => {
                low = 132 as ::core::ffi::c_int;
                high = 133 as ::core::ffi::c_int;
            }
            99 => {
                low = 133 as ::core::ffi::c_int;
                high = 134 as ::core::ffi::c_int;
            }
            100 => {
                low = 134 as ::core::ffi::c_int;
                high = 135 as ::core::ffi::c_int;
            }
            101 => {
                low = 135 as ::core::ffi::c_int;
                high = 138 as ::core::ffi::c_int;
            }
            102 => {
                low = 138 as ::core::ffi::c_int;
                high = 139 as ::core::ffi::c_int;
            }
            103 => {
                low = 139 as ::core::ffi::c_int;
                high = 141 as ::core::ffi::c_int;
            }
            105 => {
                low = 141 as ::core::ffi::c_int;
                high = 147 as ::core::ffi::c_int;
            }
            108 => {
                low = 147 as ::core::ffi::c_int;
                high = 150 as ::core::ffi::c_int;
            }
            109 => {
                low = 150 as ::core::ffi::c_int;
                high = 152 as ::core::ffi::c_int;
            }
            110 => {
                low = 152 as ::core::ffi::c_int;
                high = 154 as ::core::ffi::c_int;
            }
            115 => {
                low = 154 as ::core::ffi::c_int;
                high = 156 as ::core::ffi::c_int;
            }
            117 => {
                low = 156 as ::core::ffi::c_int;
                high = 157 as ::core::ffi::c_int;
            }
            _ => {}
        },
        18 => match *str.offset(9 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 157 as ::core::ffi::c_int;
                high = 160 as ::core::ffi::c_int;
            }
            99 => {
                low = 160 as ::core::ffi::c_int;
                high = 162 as ::core::ffi::c_int;
            }
            101 => {
                low = 162 as ::core::ffi::c_int;
                high = 165 as ::core::ffi::c_int;
            }
            103 => {
                low = 165 as ::core::ffi::c_int;
                high = 167 as ::core::ffi::c_int;
            }
            105 => {
                low = 167 as ::core::ffi::c_int;
                high = 168 as ::core::ffi::c_int;
            }
            110 => {
                low = 168 as ::core::ffi::c_int;
                high = 169 as ::core::ffi::c_int;
            }
            112 => {
                low = 169 as ::core::ffi::c_int;
                high = 171 as ::core::ffi::c_int;
            }
            114 => {
                low = 171 as ::core::ffi::c_int;
                high = 172 as ::core::ffi::c_int;
            }
            115 => {
                low = 172 as ::core::ffi::c_int;
                high = 175 as ::core::ffi::c_int;
            }
            116 => {
                low = 175 as ::core::ffi::c_int;
                high = 176 as ::core::ffi::c_int;
            }
            _ => {}
        },
        19 => match *str.offset(14 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 176 as ::core::ffi::c_int;
                high = 179 as ::core::ffi::c_int;
            }
            99 => {
                low = 179 as ::core::ffi::c_int;
                high = 180 as ::core::ffi::c_int;
            }
            101 => {
                low = 180 as ::core::ffi::c_int;
                high = 185 as ::core::ffi::c_int;
            }
            102 => {
                low = 185 as ::core::ffi::c_int;
                high = 186 as ::core::ffi::c_int;
            }
            103 => {
                low = 186 as ::core::ffi::c_int;
                high = 187 as ::core::ffi::c_int;
            }
            105 => {
                low = 187 as ::core::ffi::c_int;
                high = 188 as ::core::ffi::c_int;
            }
            110 => {
                low = 188 as ::core::ffi::c_int;
                high = 189 as ::core::ffi::c_int;
            }
            111 => {
                low = 189 as ::core::ffi::c_int;
                high = 192 as ::core::ffi::c_int;
            }
            112 => {
                low = 192 as ::core::ffi::c_int;
                high = 197 as ::core::ffi::c_int;
            }
            116 => {
                low = 197 as ::core::ffi::c_int;
                high = 198 as ::core::ffi::c_int;
            }
            117 => {
                low = 198 as ::core::ffi::c_int;
                high = 203 as ::core::ffi::c_int;
            }
            _ => {}
        },
        20 => match *str.offset(17 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            97 => {
                low = 203 as ::core::ffi::c_int;
                high = 206 as ::core::ffi::c_int;
            }
            98 => {
                low = 206 as ::core::ffi::c_int;
                high = 208 as ::core::ffi::c_int;
            }
            100 => {
                low = 208 as ::core::ffi::c_int;
                high = 209 as ::core::ffi::c_int;
            }
            103 => {
                low = 209 as ::core::ffi::c_int;
                high = 211 as ::core::ffi::c_int;
            }
            105 => {
                low = 211 as ::core::ffi::c_int;
                high = 215 as ::core::ffi::c_int;
            }
            110 => {
                low = 215 as ::core::ffi::c_int;
                high = 217 as ::core::ffi::c_int;
            }
            111 => {
                low = 217 as ::core::ffi::c_int;
                high = 218 as ::core::ffi::c_int;
            }
            118 => {
                low = 218 as ::core::ffi::c_int;
                high = 221 as ::core::ffi::c_int;
            }
            119 => {
                low = 221 as ::core::ffi::c_int;
                high = 225 as ::core::ffi::c_int;
            }
            _ => {}
        },
        21 => match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 225 as ::core::ffi::c_int;
                high = 227 as ::core::ffi::c_int;
            }
            98 => {
                low = 227 as ::core::ffi::c_int;
                high = 229 as ::core::ffi::c_int;
            }
            99 => {
                low = 229 as ::core::ffi::c_int;
                high = 230 as ::core::ffi::c_int;
            }
            100 => {
                low = 230 as ::core::ffi::c_int;
                high = 232 as ::core::ffi::c_int;
            }
            101 => {
                low = 232 as ::core::ffi::c_int;
                high = 233 as ::core::ffi::c_int;
            }
            103 => {
                low = 233 as ::core::ffi::c_int;
                high = 237 as ::core::ffi::c_int;
            }
            112 => {
                low = 237 as ::core::ffi::c_int;
                high = 238 as ::core::ffi::c_int;
            }
            114 => {
                low = 238 as ::core::ffi::c_int;
                high = 240 as ::core::ffi::c_int;
            }
            115 => {
                low = 240 as ::core::ffi::c_int;
                high = 242 as ::core::ffi::c_int;
            }
            116 => {
                low = 242 as ::core::ffi::c_int;
                high = 243 as ::core::ffi::c_int;
            }
            119 => {
                low = 243 as ::core::ffi::c_int;
                high = 244 as ::core::ffi::c_int;
            }
            _ => {}
        },
        22 => match *str.offset(16 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 244 as ::core::ffi::c_int;
                high = 246 as ::core::ffi::c_int;
            }
            98 => {
                low = 246 as ::core::ffi::c_int;
                high = 249 as ::core::ffi::c_int;
            }
            104 => {
                low = 249 as ::core::ffi::c_int;
                high = 252 as ::core::ffi::c_int;
            }
            105 => {
                low = 252 as ::core::ffi::c_int;
                high = 253 as ::core::ffi::c_int;
            }
            109 => {
                low = 253 as ::core::ffi::c_int;
                high = 254 as ::core::ffi::c_int;
            }
            116 => {
                low = 254 as ::core::ffi::c_int;
                high = 255 as ::core::ffi::c_int;
            }
            119 => {
                low = 255 as ::core::ffi::c_int;
                high = 257 as ::core::ffi::c_int;
            }
            121 => {
                low = 257 as ::core::ffi::c_int;
                high = 259 as ::core::ffi::c_int;
            }
            _ => {}
        },
        23 => match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 259 as ::core::ffi::c_int;
                high = 260 as ::core::ffi::c_int;
            }
            101 => {
                low = 260 as ::core::ffi::c_int;
                high = 262 as ::core::ffi::c_int;
            }
            108 => {
                low = 262 as ::core::ffi::c_int;
                high = 263 as ::core::ffi::c_int;
            }
            116 => {
                low = 263 as ::core::ffi::c_int;
                high = 264 as ::core::ffi::c_int;
            }
            117 => {
                low = 264 as ::core::ffi::c_int;
                high = 265 as ::core::ffi::c_int;
            }
            _ => {}
        },
        24 => match *str.offset(13 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 265 as ::core::ffi::c_int;
                high = 266 as ::core::ffi::c_int;
            }
            101 => {
                low = 266 as ::core::ffi::c_int;
                high = 268 as ::core::ffi::c_int;
            }
            111 => {
                low = 268 as ::core::ffi::c_int;
                high = 269 as ::core::ffi::c_int;
            }
            114 => {
                low = 269 as ::core::ffi::c_int;
                high = 271 as ::core::ffi::c_int;
            }
            115 => {
                low = 271 as ::core::ffi::c_int;
                high = 272 as ::core::ffi::c_int;
            }
            117 => {
                low = 272 as ::core::ffi::c_int;
                high = 273 as ::core::ffi::c_int;
            }
            _ => {}
        },
        25 => match *str.offset(9 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            97 => {
                low = 273 as ::core::ffi::c_int;
                high = 274 as ::core::ffi::c_int;
            }
            100 => {
                low = 274 as ::core::ffi::c_int;
                high = 275 as ::core::ffi::c_int;
            }
            115 => {
                low = 275 as ::core::ffi::c_int;
                high = 276 as ::core::ffi::c_int;
            }
            _ => {}
        },
        26 => match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            98 => {
                low = 276 as ::core::ffi::c_int;
                high = 277 as ::core::ffi::c_int;
            }
            115 => {
                low = 277 as ::core::ffi::c_int;
                high = 278 as ::core::ffi::c_int;
            }
            _ => {}
        },
        28 => match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 278 as ::core::ffi::c_int;
                high = 279 as ::core::ffi::c_int;
            }
            98 => {
                low = 279 as ::core::ffi::c_int;
                high = 280 as ::core::ffi::c_int;
            }
            _ => {}
        },
        _ => {}
    }
    let mut i: ::core::ffi::c_int = low;
    while i < high {
        if memcmp(
            str as *const ::core::ffi::c_void,
            (*method_handlers.ptr())[i as usize].name as *const ::core::ffi::c_void,
            len,
        ) == 0
        {
            return i;
        }
        i += 1;
    }
    return -1 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn msgpack_rpc_get_handler_for(
    mut name: *const ::core::ffi::c_char,
    mut name_len: size_t,
    mut error: *mut Error,
) -> MsgpackRpcRequestHandler {
    let mut hash: ::core::ffi::c_int = msgpack_rpc_get_handler_for_hash(name, name_len);
    if hash < 0 as ::core::ffi::c_int {
        api_set_error(
            error,
            kErrorTypeException,
            b"Invalid method: %.*s\0".as_ptr() as *const ::core::ffi::c_char,
            if name_len > 0 as size_t {
                name_len as ::core::ffi::c_int
            } else {
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>() as ::core::ffi::c_int
            },
            if name_len > 0 as size_t {
                name
            } else {
                b"<empty>\0".as_ptr() as *const ::core::ffi::c_char
            },
        );
        return MsgpackRpcRequestHandler {
            name: ::core::ptr::null::<::core::ffi::c_char>(),
            fn_0: None,
            fast: false,
            ret_alloc: false,
        };
    }
    return (*method_handlers.ptr())[hash as usize];
}
