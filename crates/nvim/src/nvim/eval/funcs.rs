use crate::src::mpack::object::mpack_parser_init;
use crate::src::nvim::api::private::converter::{
    object_to_vim, object_to_vim_take_luaref, vim_to_object,
};
use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_free_object, api_free_string, api_metadata, arena_array, cstr_as_string,
    dict_set_var,
};
use crate::src::nvim::api::vim::nvim_feedkeys;
use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::autocmd::{apply_autocmds, au_exists, autocmd_supported};
use crate::src::nvim::buffer::{
    bt_prompt, buf_close_terminal, buflist_findnr, buflist_findpat, setfname,
};
use crate::src::nvim::channel::find_channel;
use crate::src::nvim::channel::{
    channel_close, channel_connect, channel_create_event, channel_decref, channel_from_stdio,
    channel_incref, channel_job_start, channel_send, channel_terminal_alloc,
};
use crate::src::nvim::channel::{channel_proc, channel_pty};
use crate::src::nvim::charset::skipwhite;
use crate::src::nvim::cmdexpand::{ExpandCleanup, ExpandInit, ExpandOne, cmdline_pum_active};
use crate::src::nvim::context::{
    ctx_free, ctx_from_dict, ctx_get, ctx_restore, ctx_save, ctx_size, ctx_to_dict, kCtxAll,
};
use crate::src::nvim::cursor::check_cursor;
use crate::src::nvim::edit::buf_prompt_text;
use crate::src::nvim::eval::buffer::find_buffer;
use crate::src::nvim::eval::decode::{
    json_decode_string, mpack_parse_typval, typval_parser_error_free, unpack_typval,
};
use crate::src::nvim::eval::encode::{
    encode_init_lrstate, encode_list_write, encode_read_from_list, encode_tv2echo, encode_tv2json,
    encode_vim_list_to_buf, encode_vim_to_msgpack,
};
use crate::src::nvim::eval::typval::{
    callback_free, tv_blob_alloc_ret, tv_check_for_buffer_arg, tv_check_for_dict_arg,
    tv_check_for_list_arg, tv_check_for_lnum_arg, tv_check_for_nonnull_dict_arg,
    tv_check_for_number_arg, tv_check_for_opt_dict_arg, tv_check_for_opt_number_arg,
    tv_check_for_string_arg, tv_check_str_or_nr, tv_clear, tv_copy, tv_dict_add_allocated_str,
    tv_dict_add_list, tv_dict_add_nr, tv_dict_add_str, tv_dict_add_str_len, tv_dict_alloc,
    tv_dict_alloc_ret, tv_dict_extend, tv_dict_find, tv_dict_free, tv_dict_get_bool,
    tv_dict_get_callback, tv_dict_get_number, tv_dict_get_string, tv_dict_item_remove,
    tv_dict_watcher_add, tv_dict_watcher_remove, tv_get_bool, tv_get_lnum, tv_get_lnum_buf,
    tv_get_number, tv_get_number_chk, tv_get_string, tv_get_string_buf, tv_get_string_buf_chk,
    tv_get_string_chk, tv_islocked, tv_list_alloc, tv_list_alloc_ret,
    tv_list_append_allocated_string, tv_list_append_dict, tv_list_append_number,
    tv_list_append_owned_tv, tv_list_append_string, tv_list_find, tv_list_find_nr,
    tv_list_item_remove, tv_list_unref,
};
use crate::src::nvim::eval::typval::{
    tv_blob_get, tv_blob_len, tv_dict_len, tv_list_first, tv_list_len, tv_list_locked, tv_list_ref,
    tv_list_set_lock, tv_list_set_ret, tv_list_uidx,
};
use crate::src::nvim::eval::userfunc::{
    emsg_funcname, find_func, func_call, func_ptr_ref, func_ref, func_unref, function_exists,
    get_scriptlocal_funcname, get_user_func_name, restore_funccal, save_funccal,
    save_function_name, set_current_funccal, trans_function_name, translated_function_exists,
};
use crate::src::nvim::eval::vars::{
    cat_prefix_varname, find_var, get_user_var_name, get_vim_var_nr, get_vim_var_str,
    set_vim_var_nr, var_exists,
};
use crate::src::nvim::eval::window::{find_tabwin, find_win_by_nr_or_id};
use crate::src::nvim::eval_1::{
    add_timer_info, add_timer_info_all, callback_from_typval, clear_lval, common_job_callbacks,
    eval_expr_to_bool, eval_expr_typval, eval_expr_valid_arg, eval_has_provider, eval_option,
    eval1, find_job, find_timer_by_nr, get_callback_depth, get_lval, partial_name,
    prompt_get_input, save_tv_as_string, script_host_eval, timer_due_cb, timer_start, timer_stop,
    timer_stop_all, tv_to_argv,
};
use crate::src::nvim::event::libuv::{uv_kill, uv_strerror};
use crate::src::nvim::event::r#loop::{loop_on_put, process_events_until};
use crate::src::nvim::event::multiqueue::{
    multiqueue_free, multiqueue_new, multiqueue_process_events, multiqueue_replace_parent,
};
use crate::src::nvim::event::proc::proc_is_stopped;
use crate::src::nvim::event::proc::{proc_stop, proc_wait};
use crate::src::nvim::event::time::{
    time_watcher_close, time_watcher_init, time_watcher_start, time_watcher_stop,
};
use crate::src::nvim::ex_cmds::check_secure;
use crate::src::nvim::ex_docmd::{
    cmd_exists, do_cmdline, do_cmdline_cmd, eval_vars, expand_filename,
};
use crate::src::nvim::ex_eval::aborting;
use crate::src::nvim::ex_getln::{get_user_input, text_locked, text_locked_msg};
use crate::src::nvim::garray::{ga_append, ga_append_via_ptr, ga_grow, ga_init};
use crate::src::nvim::getchar::{
    restore_typeahead, save_typeahead, stuff_empty, using_script, vgetc,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::{grid_getchar, schar_from_char, schar_get, schar_get_first_codepoint};
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::highlight_group::{
    get_highlight_name_ext, highlight_color, highlight_exists, highlight_has_attr,
    syn_get_final_id, syn_name2id,
};
use crate::src::nvim::indent::{get_sw_value, get_sw_value_col};
use crate::src::nvim::input::prompt_for_input;
use crate::src::nvim::insexpand::ins_compl_active;
use crate::src::nvim::log::logmsg;
use crate::src::nvim::lua::executor::{
    nlua_exec, nlua_func_exists, nlua_is_table_from_lua, nlua_register_table_as_callable,
    nlua_typval_eval,
};
use crate::src::nvim::main::{
    EVALARG_EVALUATE, IObuff, NameBuff, Rows, State, autocmd_bufnr, autocmd_busy, autocmd_fname,
    autocmd_fname_full, autocmd_match, called_emsg, capture_ga, cmdline_row, cmdline_star, curbuf,
    current_sctx, curtab, curwin, did_emsg, e_api_error, e_buffer_is_not_loaded,
    e_cannot_change_readonly_variable_str, e_channotpty, e_dictkey, e_invalid_buffer_name_str,
    e_invalwindow, e_invarg, e_invarg2, e_invargNval, e_invargval, e_invexpr2, e_libcall,
    e_listarg, e_listblobarg, e_listdictarg, e_number_exp,
    e_reduce_of_an_empty_str_with_no_initial_value, e_stdiochan2, e_toofewarg, e_toomanyarg,
    e_trailing_arg, e_unknown_function_str, empty_string_option, emsg_noredir, emsg_off,
    emsg_silent, firstwin, garbage_collect_at_exit, got_int, lastbuf, lines_left, main_loop,
    mouse_row, msg_col, msg_row, msg_scroll, msg_scrolled, msg_silent, need_clr_eos, on_print,
    p_cpo, p_ic, p_magic, p_tgc, p_verbose, p_wic, p_ws, provider_call_nesting,
    provider_caller_scope, redir_off, starting, stdin_isatty, stdout_isatty, typebuf, vgetc_busy,
    want_garbage_collect, wild_menu_showing, windowsVersion,
};
use crate::src::nvim::mark::setpcmark;
use crate::src::nvim::mbyte::{utf_ptr2char, utf_ptr2len, utfc_ptr2len};
use crate::src::nvim::memline::{
    decl, incl, ml_get, ml_get_buf, ml_get_len, ml_open, recover_names, swapfile_dict,
};
use crate::src::nvim::memory::{
    ARENA_EMPTY, alloc_block, arena_finish, arena_mem_free, free_block, strequal, strnequal,
    xcalloc, xfree, xmalloc, xmemdup, xmemdupz, xstrdup,
};
use crate::src::nvim::menu::{get_menu_cmd_modes, menu_get};
use crate::src::nvim::message::{
    do_dialog, emsg, msg_clr_eos, msg_ext_set_kind, msg_putchar, msg_puts, msg_scroll_flush,
    msg_start, semsg, semsg_multiline, verb_msg,
};
use crate::src::nvim::r#move::win_col_off;
use crate::src::nvim::msgpack_rpc::channel::get_client_info;
use crate::src::nvim::msgpack_rpc::channel::{rpc_send_call, rpc_send_event};
use crate::src::nvim::msgpack_rpc::packer::{packer_string_buffer, packer_take_string};
use crate::src::nvim::msgpack_rpc::server::{
    server_address_list, server_address_new, server_start, server_stop,
};
use crate::src::nvim::normal::{find_decl, op_pending};
use crate::src::nvim::ops::cursor_pos_info;
use crate::src::nvim::option::set_option_value_give_err;
use crate::src::nvim::options::kOptCpoptions;
use crate::src::nvim::optionstr::free_string_option;
use crate::src::nvim::os::dl::{LibcallArg, LibcallResult, LibcallReturn, os_libcall};
use crate::src::nvim::os::env::{
    expand_env_save, home_replace, os_copy_fullenv, os_env_exists, os_free_fullenv,
    os_get_fullenv_size, os_get_hostname, os_get_pid, os_getenv, vim_env_iter, vim_getenv,
    vim_setenv_ext, vim_unsetenv_ext,
};
use crate::src::nvim::os::fs::{os_isdir, os_setperm};
use crate::src::nvim::os::libc::{
    __assert_fail, atoi, gettext, memcpy, memmove, memset, snprintf, strcasecmp, strchr, strcmp,
    strcpy, strlen, strncasecmp, strncmp, strtoul, time,
};
use crate::src::nvim::os::pty_proc_unix::pty_proc_resize;
use crate::src::nvim::os::shell::shell_free_argv;
use crate::src::nvim::os::stdpaths::{get_appname, get_xdg_home, stdpaths_get_xdg_var};
use crate::src::nvim::os::time::os_hrtime;
use crate::src::nvim::path::{concat_fnames_realloc, vim_FullName};
use crate::src::nvim::popupmenu::{pum_set_event_info, pum_visible};
use crate::src::nvim::pos::{clearpos, equalpos};
use crate::src::nvim::profile::{
    profile_end, profile_msg, profile_setlimit, profile_signed, profile_start, profile_sub,
};
use crate::src::nvim::runtime::exestack;
use crate::src::nvim::search::searchit;
use crate::src::nvim::state::{get_mode, get_was_safe_state};
use crate::src::nvim::strings::{vim_snprintf, vim_strchr, vim_vsnprintf_typval};
use crate::src::nvim::syntax::{
    get_syntax_info, syn_get_id, syn_get_stack_item, syn_get_sub_char, syntax_present,
};
use crate::src::nvim::terminal::{terminal_buf, terminal_open, terminal_running};
pub use crate::src::nvim::types::{
    __builtin_va_list, __gid_t, __gnuc_va_list, __pthread_internal_list, __pthread_list_t,
    __pthread_mutex_s, __pthread_rwlock_arch_t, __time_t, __uid_t, __va_list_tag, AdditionalData,
    AlignTextPos, ApiDispatchWrapper, Arena, ArenaMem, Array, AutoPat, AutoPatCmd, AutoPatCmd_S,
    BoolVarValue, Boolean, BufUpdateCallbacks, Buffer, CMD_index, Callback,
    Callback_data as C2Rust_Unnamed_22, CallbackReader, CallbackType, ChangedtickDictItem, Channel,
    Channel_stream as C2Rust_Unnamed_42, ChannelCallFrame, ChannelPart, ChannelStdinMode,
    ChannelStreamType, ClientType, Context, DecorExt, DecorHighlightInline, DecorInlineData,
    DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_19, Dict, Direction, Error,
    ErrorType, EvalFuncData, EvalFuncDef, ExtmarkUndoObject, FileID, Float, FloatAnchor,
    FloatRelative, GRegFlags, GridView, Integer, InternalState, Intersection, KeyValuePair,
    LibuvProc, LineGetter, ListLenSpecials, ListReaderState, Loop, LuaRef, LuaRetMode, MTKey,
    MTNode, MTPos, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MapHash, MarkTree, MotionType, MsgpackRpcRequestHandler, MultiQueue,
    Object, ObjectType, OptIndex, OptInt, OptVal, OptValData, OptValType, PackerBuffer,
    PackerBufferFlush, Proc, ProcType, PtyProc, PutCallback, QUEUE, RStream, RemoteUI, RpcState,
    ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t,
    SpecialVarValue, StderrState, StdioPair, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_29, Stream, String_0, Terminal, TimeWatcher,
    Timestamp, TriState, UIExtension, Unpacker, VarLockStatus, VarType, VimLFunc, VimVarIndex,
    VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle,
    Window, XDGVarType, alist_T, auto_event, bhdr_T, blob_T, blobvar_S, block_def, blocknr_T,
    buf_T, buffblock, buffblock_T, buffheader_T, bufstate_T, caller_scope, chunksize_T, cmd_addr_T,
    cmdidx_T, colnr_T, consumed_blk, cstack_T, cstack_T_cs_pend as C2Rust_Unnamed_40, dict_T,
    dictitem_T, dictvar_S, diff_T, diffblock_S, disptick_T, eslist_T, eslist_elem, estack_T,
    estack_T_es_info as C2Rust_Unnamed_49, etype_T, evalarg_T, event_T, exarg, exarg_T, except_T,
    except_type_T, expand_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_20, file_buffer_b_wininfo as C2Rust_Unnamed_28,
    file_buffer_update_callbacks as C2Rust_Unnamed_17,
    file_buffer_update_channels as C2Rust_Unnamed_18, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccal_entry, funccal_entry_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_23,
    funccall_T, funcdict_T, garray_T, gid_t, handle_T, hash_T, hashitem_T, hashtab_T, hlf_T,
    iconv_t, infoptr_T, int16_t, int32_t, int64_t, internal_proc_cb, key_value_pair, lcs_chars_T,
    linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, loop_0,
    lpos_T, lval_T, mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T,
    mfdirty_T, mpack_data_t, mpack_node_s, mpack_node_t, mpack_parser_t, mpack_sintmax_t,
    mpack_tokbuf_s, mpack_tokbuf_t, mpack_token_s, mpack_token_s_data as C2Rust_Unnamed_14,
    mpack_token_t, mpack_token_type_t, mpack_uint32_t, mpack_uintmax_t, mpack_value_s,
    mpack_value_t, msglist, msglist_T, mtnode_inner_s, mtnode_s, multiqueue, object,
    object_data as C2Rust_Unnamed_16, oparg_T, packer_buffer_t, partial_S, partial_T, pos_T,
    pos_save_T, proc, proc_exit_cb, proc_state_cb, proftime_T, pthread_mutex_t, pthread_rwlock_t,
    ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue, reg_extmatch_T, regmatch_T, regmmatch_T,
    regprog, regprog_T, rstream, sattr_T, schar_T, scid_T, sctx_T, searchit_arg_T, size_t, smt_T,
    ssize_t, stream, stream_close_cb, stream_read_cb, stream_uv as C2Rust_Unnamed_31,
    stream_write_cb, syn_state, syn_state_sst_union as C2Rust_Unnamed_21, syn_time_T, synblock_T,
    synstate_T, tabpage_S, tabpage_T, taggy_T, tagname_T, tasave_T, terminal, time_cb, time_t,
    time_watcher, timer_T, tm, typebuf_T, typval_T, typval_vval_union, u_entry, u_entry_T,
    u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_25,
    u_header_uh_alt_prev as C2Rust_Unnamed_24, u_header_uh_next as C2Rust_Unnamed_27,
    u_header_uh_prev as C2Rust_Unnamed_26, ufunc_S, ufunc_T, uid_t, uint8_t, uint16_t, uint32_t,
    uint64_t, undo_object, uv__io_cb, uv__io_s, uv__io_t, uv__queue, uv__work, uv_alloc_cb,
    uv_async_cb, uv_async_s, uv_async_s_u as C2Rust_Unnamed_3, uv_async_t, uv_buf_t, uv_close_cb,
    uv_connect_cb, uv_connect_s, uv_connect_t, uv_connection_cb, uv_exit_cb, uv_file, uv_gid_t,
    uv_handle_s, uv_handle_s_u as C2Rust_Unnamed_0, uv_handle_t, uv_handle_type, uv_idle_cb,
    uv_idle_s, uv_idle_s_u as C2Rust_Unnamed_10, uv_idle_t, uv_loop_s,
    uv_loop_s_active_reqs as C2Rust_Unnamed_4, uv_loop_s_timer_heap as C2Rust_Unnamed_2, uv_loop_t,
    uv_mutex_t, uv_pipe_s, uv_pipe_s_u as C2Rust_Unnamed_7, uv_pipe_t, uv_process_options_s,
    uv_process_options_t, uv_process_s, uv_process_s_u as C2Rust_Unnamed_11, uv_process_t,
    uv_read_cb, uv_req_type, uv_rwlock_t, uv_shutdown_cb, uv_shutdown_s, uv_shutdown_t,
    uv_signal_cb, uv_signal_s, uv_signal_s_tree_entry as C2Rust_Unnamed,
    uv_signal_s_u as C2Rust_Unnamed_1, uv_signal_t, uv_stdio_container_s,
    uv_stdio_container_s_data as C2Rust_Unnamed_12, uv_stdio_container_t, uv_stdio_flags,
    uv_stream_s, uv_stream_s_u as C2Rust_Unnamed_5, uv_stream_t, uv_tcp_s,
    uv_tcp_s_u as C2Rust_Unnamed_6, uv_tcp_t, uv_timer_cb, uv_timer_s,
    uv_timer_s_node as C2Rust_Unnamed_8, uv_timer_s_u as C2Rust_Unnamed_9, uv_timer_t, uv_uid_t,
    va_list, varnumber_T, vim_exception, vimconv_T, virt_line, visualinfo_T, win_T, window_S,
    wininfo_S, winopt_T, winsize, wline_T, xfmark_T, xp_prefix_T, yankreg_T,
};
use crate::src::nvim::ui::{
    ui_busy_start, ui_busy_stop, ui_current_col, ui_current_row, ui_flush, ui_gui_attached, ui_has,
    ui_rgb_attached,
};
use crate::src::nvim::ui_compositor::ui_comp_get_grid_at_coord;
use crate::src::nvim::version::{has_nvim_version, has_vim_patch};
use crate::src::nvim::window::find_tabpage;
use core::ffi::CStr;
use std::ffi::CString;

mod table;

use self::table::{BUILTINS, builtin_index};

// The bodies, grouped by what they are about. Each child opens with
// `use super::*`, so the transpiled preamble above is its import list.

mod args;
mod call;
mod channel;
mod container;
mod context;
mod env;
mod input;
mod job;
mod marks;
mod math;
mod msgpack;
mod position;
mod reduce;
mod regexp;
mod region;
mod register;
mod runtime;
mod screen;
mod search;
mod strings;
mod timer;
mod variables;

pub use self::call::*;
pub use self::channel::*;
pub use self::container::*;
pub use self::context::*;
pub use self::env::*;
pub use self::input::*;
pub use self::job::*;
pub use self::marks::*;
pub use self::math::*;
pub use self::msgpack::*;
pub use self::position::*;
pub use self::reduce::*;
pub use self::regexp::*;
pub use self::region::*;
pub use self::register::*;
pub use self::runtime::*;
pub use self::screen::*;
pub use self::search::*;
pub use self::strings::*;
pub use self::timer::*;
pub use self::variables::*;
unsafe extern "C" {
    fn uv_random(
        loop_0: *mut uv_loop_t,
        req: *mut uv_random_t,
        buf: *mut ::core::ffi::c_void,
        buflen: size_t,
        flags: ::core::ffi::c_uint,
        cb: uv_random_cb,
    ) -> ::core::ffi::c_int;
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec_nl(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T)
    -> bool;
}
pub const UV_HANDLE_TYPE_MAX: uv_handle_type = 18;
pub const UV_FILE: uv_handle_type = 17;
pub const UV_SIGNAL: uv_handle_type = 16;
pub const UV_UDP: uv_handle_type = 15;
pub const UV_TTY: uv_handle_type = 14;
pub const UV_TIMER: uv_handle_type = 13;
pub const UV_TCP: uv_handle_type = 12;
pub const UV_STREAM: uv_handle_type = 11;
pub const UV_PROCESS: uv_handle_type = 10;
pub const UV_PREPARE: uv_handle_type = 9;
pub const UV_POLL: uv_handle_type = 8;
pub const UV_NAMED_PIPE: uv_handle_type = 7;
pub const UV_IDLE: uv_handle_type = 6;
pub const UV_HANDLE: uv_handle_type = 5;
pub const UV_FS_POLL: uv_handle_type = 4;
pub const UV_FS_EVENT: uv_handle_type = 3;
pub const UV_CHECK: uv_handle_type = 2;
pub const UV_ASYNC: uv_handle_type = 1;
pub const UV_UNKNOWN_HANDLE: uv_handle_type = 0;
pub const UV_REQ_TYPE_MAX: uv_req_type = 11;
pub const UV_RANDOM: uv_req_type = 10;
pub const UV_GETNAMEINFO: uv_req_type = 9;
pub const UV_GETADDRINFO: uv_req_type = 8;
pub const UV_WORK: uv_req_type = 7;
pub const UV_FS: uv_req_type = 6;
pub const UV_UDP_SEND: uv_req_type = 5;
pub const UV_SHUTDOWN: uv_req_type = 4;
pub const UV_WRITE: uv_req_type = 3;
pub const UV_CONNECT: uv_req_type = 2;
pub const UV_REQ: uv_req_type = 1;
pub const UV_UNKNOWN_REQ: uv_req_type = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_random_s {
    pub data: *mut ::core::ffi::c_void,
    pub type_0: uv_req_type,
    pub reserved: [*mut ::core::ffi::c_void; 6],
    pub loop_0: *mut uv_loop_t,
    pub status: ::core::ffi::c_int,
    pub buf: *mut ::core::ffi::c_void,
    pub buflen: size_t,
    pub cb: uv_random_cb,
    pub work_req: uv__work,
}
pub type uv_random_cb = Option<
    unsafe extern "C" fn(
        *mut uv_random_t,
        ::core::ffi::c_int,
        *mut ::core::ffi::c_void,
        size_t,
    ) -> (),
>;
pub type uv_random_t = uv_random_s;
pub const UV_OVERLAPPED_PIPE: uv_stdio_flags = 64;
pub const UV_NONBLOCK_PIPE: uv_stdio_flags = 64;
pub const UV_WRITABLE_PIPE: uv_stdio_flags = 32;
pub const UV_READABLE_PIPE: uv_stdio_flags = 16;
pub const UV_INHERIT_STREAM: uv_stdio_flags = 4;
pub const UV_INHERIT_FD: uv_stdio_flags = 2;
pub const UV_CREATE_PIPE: uv_stdio_flags = 1;
pub const UV_IGNORE: uv_stdio_flags = 0;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const MPACK_ERROR: C2Rust_Unnamed_13 = 2;
pub const MPACK_EOF: C2Rust_Unnamed_13 = 1;
pub const MPACK_OK: C2Rust_Unnamed_13 = 0;
pub const MPACK_TOKEN_EXT: mpack_token_type_t = 11;
pub const MPACK_TOKEN_STR: mpack_token_type_t = 10;
pub const MPACK_TOKEN_BIN: mpack_token_type_t = 9;
pub const MPACK_TOKEN_MAP: mpack_token_type_t = 8;
pub const MPACK_TOKEN_ARRAY: mpack_token_type_t = 7;
pub const MPACK_TOKEN_CHUNK: mpack_token_type_t = 6;
pub const MPACK_TOKEN_FLOAT: mpack_token_type_t = 5;
pub const MPACK_TOKEN_SINT: mpack_token_type_t = 4;
pub const MPACK_TOKEN_UINT: mpack_token_type_t = 3;
pub const MPACK_TOKEN_BOOLEAN: mpack_token_type_t = 2;
pub const MPACK_TOKEN_NIL: mpack_token_type_t = 1;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_int;
pub const MPACK_NOMEM: C2Rust_Unnamed_15 = 3;
pub const MPACK_EXCEPTION: C2Rust_Unnamed_15 = -1;
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
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
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
pub const kStlClickFuncRun: C2Rust_Unnamed_29 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_29 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_29 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_29 = 0;
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
pub const kProcTypePty: ProcType = 1;
pub const kProcTypeUv: ProcType = 0;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_32 = 2147483647;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenUnknown: ListLenSpecials = -1;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub const VAR_TYPE_BLOB: C2Rust_Unnamed_33 = 10;
pub const VAR_TYPE_SPECIAL: C2Rust_Unnamed_33 = 7;
pub const VAR_TYPE_BOOL: C2Rust_Unnamed_33 = 6;
pub const VAR_TYPE_FLOAT: C2Rust_Unnamed_33 = 5;
pub const VAR_TYPE_DICT: C2Rust_Unnamed_33 = 4;
pub const VAR_TYPE_LIST: C2Rust_Unnamed_33 = 3;
pub const VAR_TYPE_FUNC: C2Rust_Unnamed_33 = 2;
pub const VAR_TYPE_STRING: C2Rust_Unnamed_33 = 1;
pub const VAR_TYPE_NUMBER: C2Rust_Unnamed_33 = 0;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub const DI_FLAGS_ALLOC: C2Rust_Unnamed_34 = 16;
pub const DI_FLAGS_LOCK: C2Rust_Unnamed_34 = 8;
pub const DI_FLAGS_FIX: C2Rust_Unnamed_34 = 4;
pub const DI_FLAGS_RO_SBX: C2Rust_Unnamed_34 = 2;
pub const DI_FLAGS_RO: C2Rust_Unnamed_34 = 1;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub const MAX_FUNC_ARGS: C2Rust_Unnamed_35 = 20;
pub type C2Rust_Unnamed_36 = ::core::ffi::c_uint;
pub const HL_GLOBAL: C2Rust_Unnamed_36 = 16384;
pub const HL_DEFAULT: C2Rust_Unnamed_36 = 8192;
pub const HL_FG_INDEXED: C2Rust_Unnamed_36 = 4096;
pub const HL_BG_INDEXED: C2Rust_Unnamed_36 = 2048;
pub const HL_NOCOMBINE: C2Rust_Unnamed_36 = 1024;
pub const HL_OVERLINE: C2Rust_Unnamed_36 = 131072;
pub const HL_CONCEALED: C2Rust_Unnamed_36 = 65536;
pub const HL_BLINK: C2Rust_Unnamed_36 = 32768;
pub const HL_DIM: C2Rust_Unnamed_36 = 512;
pub const HL_ALTFONT: C2Rust_Unnamed_36 = 256;
pub const HL_STRIKETHROUGH: C2Rust_Unnamed_36 = 128;
pub const HL_STANDOUT: C2Rust_Unnamed_36 = 64;
pub const HL_UNDERDASHED: C2Rust_Unnamed_36 = 40;
pub const HL_UNDERDOTTED: C2Rust_Unnamed_36 = 32;
pub const HL_UNDERDOUBLE: C2Rust_Unnamed_36 = 24;
pub const HL_UNDERCURL: C2Rust_Unnamed_36 = 16;
pub const HL_UNDERLINE: C2Rust_Unnamed_36 = 8;
pub const HL_UNDERLINE_MASK: C2Rust_Unnamed_36 = 56;
pub const HL_ITALIC: C2Rust_Unnamed_36 = 4;
pub const HL_BOLD: C2Rust_Unnamed_36 = 2;
pub const HL_INVERSE: C2Rust_Unnamed_36 = 1;
pub const HLF_COUNT: hlf_T = 76;
pub const HLF_PRE: hlf_T = 75;
pub const HLF_OK: hlf_T = 74;
pub const HLF_SO: hlf_T = 73;
pub const HLF_SE: hlf_T = 72;
pub const HLF_TSNC: hlf_T = 71;
pub const HLF_TS: hlf_T = 70;
pub const HLF_BFOOTER: hlf_T = 69;
pub const HLF_BTITLE: hlf_T = 68;
pub const HLF_CU: hlf_T = 67;
pub const HLF_WBRNC: hlf_T = 66;
pub const HLF_WBR: hlf_T = 65;
pub const HLF_BORDER: hlf_T = 64;
pub const HLF_MSG: hlf_T = 63;
pub const HLF_NFLOAT: hlf_T = 62;
pub const HLF_MSGSEP: hlf_T = 61;
pub const HLF_INACTIVE: hlf_T = 60;
pub const HLF_0: hlf_T = 59;
pub const HLF_QFL: hlf_T = 58;
pub const HLF_MC: hlf_T = 57;
pub const HLF_CUL: hlf_T = 56;
pub const HLF_CUC: hlf_T = 55;
pub const HLF_TPF: hlf_T = 54;
pub const HLF_TPS: hlf_T = 53;
pub const HLF_TP: hlf_T = 52;
pub const HLF_PBR: hlf_T = 51;
pub const HLF_PST: hlf_T = 50;
pub const HLF_PSB: hlf_T = 49;
pub const HLF_PSX: hlf_T = 48;
pub const HLF_PNX: hlf_T = 47;
pub const HLF_PSK: hlf_T = 46;
pub const HLF_PNK: hlf_T = 45;
pub const HLF_PMSI: hlf_T = 44;
pub const HLF_PMNI: hlf_T = 43;
pub const HLF_PSI: hlf_T = 42;
pub const HLF_PNI: hlf_T = 41;
pub const HLF_SPL: hlf_T = 40;
pub const HLF_SPR: hlf_T = 39;
pub const HLF_SPC: hlf_T = 38;
pub const HLF_SPB: hlf_T = 37;
pub const HLF_CONCEAL: hlf_T = 36;
pub const HLF_SC: hlf_T = 35;
pub const HLF_TXA: hlf_T = 34;
pub const HLF_TXD: hlf_T = 33;
pub const HLF_DED: hlf_T = 32;
pub const HLF_CHD: hlf_T = 31;
pub const HLF_ADD: hlf_T = 30;
pub const HLF_FC: hlf_T = 29;
pub const HLF_FL: hlf_T = 28;
pub const HLF_WM: hlf_T = 27;
pub const HLF_W: hlf_T = 26;
pub const HLF_VNC: hlf_T = 25;
pub const HLF_V: hlf_T = 24;
pub const HLF_T: hlf_T = 23;
pub const HLF_VSP: hlf_T = 22;
pub const HLF_C: hlf_T = 21;
pub const HLF_SNC: hlf_T = 20;
pub const HLF_S: hlf_T = 19;
pub const HLF_R: hlf_T = 18;
pub const HLF_CLF: hlf_T = 17;
pub const HLF_CLS: hlf_T = 16;
pub const HLF_CLN: hlf_T = 15;
pub const HLF_LNB: hlf_T = 14;
pub const HLF_LNA: hlf_T = 13;
pub const HLF_N: hlf_T = 12;
pub const HLF_CM: hlf_T = 11;
pub const HLF_M: hlf_T = 10;
pub const HLF_LC: hlf_T = 9;
pub const HLF_L: hlf_T = 8;
pub const HLF_I: hlf_T = 7;
pub const HLF_E: hlf_T = 6;
pub const HLF_D: hlf_T = 5;
pub const HLF_AT: hlf_T = 4;
pub const HLF_TERM: hlf_T = 3;
pub const HLF_EOB: hlf_T = 2;
pub const HLF_8: hlf_T = 1;
pub const HLF_NONE: hlf_T = 0;
pub type C2Rust_Unnamed_37 = ::core::ffi::c_uint;
pub const NUMBUFLEN: C2Rust_Unnamed_37 = 65;
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub const XP_PREFIX_INV: xp_prefix_T = 2;
pub const XP_PREFIX_NO: xp_prefix_T = 1;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
pub type C2Rust_Unnamed_38 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_38 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_38 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_38 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_38 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_38 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_38 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_38 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_38 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_38 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_38 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_38 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_38 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_38 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_38 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_38 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_38 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_38 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_38 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_38 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_38 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_38 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_38 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_38 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_38 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_38 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_38 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_38 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_38 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_38 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_38 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_38 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_38 = 33;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_38 = 32;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_38 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_38 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_38 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_38 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_38 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_38 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_38 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_38 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_38 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_38 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_38 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_38 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_38 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_38 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_38 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_38 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_38 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_38 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_38 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_38 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_38 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_38 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_38 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_38 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_38 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_38 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_38 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_38 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_38 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_38 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_38 = 1;
pub const EXPAND_NOTHING: C2Rust_Unnamed_38 = 0;
pub const EXPAND_OK: C2Rust_Unnamed_38 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_38 = -2;
pub type C2Rust_Unnamed_39 = ::core::ffi::c_uint;
pub const NSUBEXP: C2Rust_Unnamed_39 = 10;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNumber: OptValType = 1;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kOptValTypeNil: OptValType = -1;
pub const ET_INTERRUPT: except_type_T = 2;
pub const ET_ERROR: except_type_T = 1;
pub const ET_USER: except_type_T = 0;
pub const CMD_USER_BUF: CMD_index = -2;
pub const CMD_USER: CMD_index = -1;
pub const CMD_SIZE: CMD_index = 557;
pub const CMD_Next: CMD_index = 556;
pub const CMD_tilde: CMD_index = 555;
pub const CMD_at: CMD_index = 554;
pub const CMD_rshift: CMD_index = 553;
pub const CMD_equal: CMD_index = 552;
pub const CMD_lshift: CMD_index = 551;
pub const CMD_and: CMD_index = 550;
pub const CMD_pound: CMD_index = 549;
pub const CMD_bang: CMD_index = 548;
pub const CMD_z: CMD_index = 547;
pub const CMD_yank: CMD_index = 546;
pub const CMD_xunmenu: CMD_index = 545;
pub const CMD_xunmap: CMD_index = 544;
pub const CMD_xnoremenu: CMD_index = 543;
pub const CMD_xnoremap: CMD_index = 542;
pub const CMD_xmenu: CMD_index = 541;
pub const CMD_xmapclear: CMD_index = 540;
pub const CMD_xmap: CMD_index = 539;
pub const CMD_xall: CMD_index = 538;
pub const CMD_xit: CMD_index = 537;
pub const CMD_wviminfo: CMD_index = 536;
pub const CMD_wundo: CMD_index = 535;
pub const CMD_wshada: CMD_index = 534;
pub const CMD_wqall: CMD_index = 533;
pub const CMD_wq: CMD_index = 532;
pub const CMD_wprevious: CMD_index = 531;
pub const CMD_wnext: CMD_index = 530;
pub const CMD_winpos: CMD_index = 529;
pub const CMD_windo: CMD_index = 528;
pub const CMD_wincmd: CMD_index = 527;
pub const CMD_winsize: CMD_index = 526;
pub const CMD_while: CMD_index = 525;
pub const CMD_wall: CMD_index = 524;
pub const CMD_wNext: CMD_index = 523;
pub const CMD_write: CMD_index = 522;
pub const CMD_vunmenu: CMD_index = 521;
pub const CMD_vunmap: CMD_index = 520;
pub const CMD_vsplit: CMD_index = 519;
pub const CMD_vnoremenu: CMD_index = 518;
pub const CMD_vnew: CMD_index = 517;
pub const CMD_vnoremap: CMD_index = 516;
pub const CMD_vmenu: CMD_index = 515;
pub const CMD_vmapclear: CMD_index = 514;
pub const CMD_vmap: CMD_index = 513;
pub const CMD_viusage: CMD_index = 512;
pub const CMD_vimgrepadd: CMD_index = 511;
pub const CMD_vimgrep: CMD_index = 510;
pub const CMD_view: CMD_index = 509;
pub const CMD_visual: CMD_index = 508;
pub const CMD_vertical: CMD_index = 507;
pub const CMD_verbose: CMD_index = 506;
pub const CMD_version: CMD_index = 505;
pub const CMD_vglobal: CMD_index = 504;
pub const CMD_update: CMD_index = 503;
pub const CMD_unsilent: CMD_index = 502;
pub const CMD_unmenu: CMD_index = 501;
pub const CMD_unmap: CMD_index = 500;
pub const CMD_unlockvar: CMD_index = 499;
pub const CMD_unlet: CMD_index = 498;
pub const CMD_uniq: CMD_index = 497;
pub const CMD_unhide: CMD_index = 496;
pub const CMD_unabbreviate: CMD_index = 495;
pub const CMD_undolist: CMD_index = 494;
pub const CMD_undojoin: CMD_index = 493;
pub const CMD_undo: CMD_index = 492;
pub const CMD_tunmap: CMD_index = 491;
pub const CMD_tunmenu: CMD_index = 490;
pub const CMD_tselect: CMD_index = 489;
pub const CMD_try: CMD_index = 488;
pub const CMD_trust: CMD_index = 487;
pub const CMD_trewind: CMD_index = 486;
pub const CMD_tprevious: CMD_index = 485;
pub const CMD_topleft: CMD_index = 484;
pub const CMD_tnoremap: CMD_index = 483;
pub const CMD_tnext: CMD_index = 482;
pub const CMD_tmapclear: CMD_index = 481;
pub const CMD_tmap: CMD_index = 480;
pub const CMD_tmenu: CMD_index = 479;
pub const CMD_tlunmenu: CMD_index = 478;
pub const CMD_tlnoremenu: CMD_index = 477;
pub const CMD_tlmenu: CMD_index = 476;
pub const CMD_tlast: CMD_index = 475;
pub const CMD_tjump: CMD_index = 474;
pub const CMD_throw: CMD_index = 473;
pub const CMD_tfirst: CMD_index = 472;
pub const CMD_terminal: CMD_index = 471;
pub const CMD_tclfile: CMD_index = 470;
pub const CMD_tcldo: CMD_index = 469;
pub const CMD_tcl: CMD_index = 468;
pub const CMD_tabs: CMD_index = 467;
pub const CMD_tabrewind: CMD_index = 466;
pub const CMD_tabNext: CMD_index = 465;
pub const CMD_tabprevious: CMD_index = 464;
pub const CMD_tabonly: CMD_index = 463;
pub const CMD_tabnew: CMD_index = 462;
pub const CMD_tabnext: CMD_index = 461;
pub const CMD_tablast: CMD_index = 460;
pub const CMD_tabmove: CMD_index = 459;
pub const CMD_tabfirst: CMD_index = 458;
pub const CMD_tabfind: CMD_index = 457;
pub const CMD_tabedit: CMD_index = 456;
pub const CMD_tabdo: CMD_index = 455;
pub const CMD_tabclose: CMD_index = 454;
pub const CMD_tab: CMD_index = 453;
pub const CMD_tags: CMD_index = 452;
pub const CMD_tag: CMD_index = 451;
pub const CMD_tNext: CMD_index = 450;
pub const CMD_tchdir: CMD_index = 449;
pub const CMD_tcd: CMD_index = 448;
pub const CMD_t: CMD_index = 447;
pub const CMD_syncbind: CMD_index = 446;
pub const CMD_syntime: CMD_index = 445;
pub const CMD_syntax: CMD_index = 444;
pub const CMD_swapname: CMD_index = 443;
pub const CMD_sview: CMD_index = 442;
pub const CMD_suspend: CMD_index = 441;
pub const CMD_sunmenu: CMD_index = 440;
pub const CMD_sunmap: CMD_index = 439;
pub const CMD_sunhide: CMD_index = 438;
pub const CMD_stselect: CMD_index = 437;
pub const CMD_stjump: CMD_index = 436;
pub const CMD_stopinsert: CMD_index = 435;
pub const CMD_startreplace: CMD_index = 434;
pub const CMD_startgreplace: CMD_index = 433;
pub const CMD_startinsert: CMD_index = 432;
pub const CMD_stag: CMD_index = 431;
pub const CMD_stop: CMD_index = 430;
pub const CMD_srewind: CMD_index = 429;
pub const CMD_sprevious: CMD_index = 428;
pub const CMD_spellwrong: CMD_index = 427;
pub const CMD_spellundo: CMD_index = 426;
pub const CMD_spellrare: CMD_index = 425;
pub const CMD_spellrepall: CMD_index = 424;
pub const CMD_spellinfo: CMD_index = 423;
pub const CMD_spelldump: CMD_index = 422;
pub const CMD_spellgood: CMD_index = 421;
pub const CMD_split: CMD_index = 420;
pub const CMD_sort: CMD_index = 419;
pub const CMD_source: CMD_index = 418;
pub const CMD_snoremenu: CMD_index = 417;
pub const CMD_snoremap: CMD_index = 416;
pub const CMD_snomagic: CMD_index = 415;
pub const CMD_snext: CMD_index = 414;
pub const CMD_smenu: CMD_index = 413;
pub const CMD_smapclear: CMD_index = 412;
pub const CMD_smap: CMD_index = 411;
pub const CMD_smagic: CMD_index = 410;
pub const CMD_slast: CMD_index = 409;
pub const CMD_sleep: CMD_index = 408;
pub const CMD_silent: CMD_index = 407;
pub const CMD_sign: CMD_index = 406;
pub const CMD_simalt: CMD_index = 405;
pub const CMD_sfirst: CMD_index = 404;
pub const CMD_sfind: CMD_index = 403;
pub const CMD_setlocal: CMD_index = 402;
pub const CMD_setglobal: CMD_index = 401;
pub const CMD_setfiletype: CMD_index = 400;
pub const CMD_set: CMD_index = 399;
pub const CMD_scriptencoding: CMD_index = 398;
pub const CMD_scriptnames: CMD_index = 397;
pub const CMD_sbrewind: CMD_index = 396;
pub const CMD_sbprevious: CMD_index = 395;
pub const CMD_sbnext: CMD_index = 394;
pub const CMD_sbmodified: CMD_index = 393;
pub const CMD_sblast: CMD_index = 392;
pub const CMD_sbfirst: CMD_index = 391;
pub const CMD_sball: CMD_index = 390;
pub const CMD_sbNext: CMD_index = 389;
pub const CMD_sbuffer: CMD_index = 388;
pub const CMD_saveas: CMD_index = 387;
pub const CMD_sandbox: CMD_index = 386;
pub const CMD_sall: CMD_index = 385;
pub const CMD_sargument: CMD_index = 384;
pub const CMD_sNext: CMD_index = 383;
pub const CMD_substitute: CMD_index = 382;
pub const CMD_rviminfo: CMD_index = 381;
pub const CMD_rubyfile: CMD_index = 380;
pub const CMD_rubydo: CMD_index = 379;
pub const CMD_ruby: CMD_index = 378;
pub const CMD_rundo: CMD_index = 377;
pub const CMD_runtime: CMD_index = 376;
pub const CMD_rshada: CMD_index = 375;
pub const CMD_rightbelow: CMD_index = 374;
pub const CMD_right: CMD_index = 373;
pub const CMD_rewind: CMD_index = 372;
pub const CMD_return: CMD_index = 371;
pub const CMD_retab: CMD_index = 370;
pub const CMD_restart: CMD_index = 369;
pub const CMD_resize: CMD_index = 368;
pub const CMD_registers: CMD_index = 367;
pub const CMD_redrawtabline: CMD_index = 366;
pub const CMD_redrawstatus: CMD_index = 365;
pub const CMD_redraw: CMD_index = 364;
pub const CMD_redir: CMD_index = 363;
pub const CMD_redo: CMD_index = 362;
pub const CMD_recover: CMD_index = 361;
pub const CMD_read: CMD_index = 360;
pub const CMD_qall: CMD_index = 359;
pub const CMD_quitall: CMD_index = 358;
pub const CMD_quit: CMD_index = 357;
pub const CMD_pyxfile: CMD_index = 356;
pub const CMD_pythonx: CMD_index = 355;
pub const CMD_pyxdo: CMD_index = 354;
pub const CMD_pyx: CMD_index = 353;
pub const CMD_py3file: CMD_index = 352;
pub const CMD_python3: CMD_index = 351;
pub const CMD_py3do: CMD_index = 350;
pub const CMD_py3: CMD_index = 349;
pub const CMD_pyfile: CMD_index = 348;
pub const CMD_pydo: CMD_index = 347;
pub const CMD_python: CMD_index = 346;
pub const CMD_pwd: CMD_index = 345;
pub const CMD_put: CMD_index = 344;
pub const CMD_ptselect: CMD_index = 343;
pub const CMD_ptrewind: CMD_index = 342;
pub const CMD_ptprevious: CMD_index = 341;
pub const CMD_ptnext: CMD_index = 340;
pub const CMD_ptlast: CMD_index = 339;
pub const CMD_ptjump: CMD_index = 338;
pub const CMD_ptfirst: CMD_index = 337;
pub const CMD_ptNext: CMD_index = 336;
pub const CMD_ptag: CMD_index = 335;
pub const CMD_psearch: CMD_index = 334;
pub const CMD_profdel: CMD_index = 333;
pub const CMD_profile: CMD_index = 332;
pub const CMD_previous: CMD_index = 331;
pub const CMD_preserve: CMD_index = 330;
pub const CMD_ppop: CMD_index = 329;
pub const CMD_popup: CMD_index = 328;
pub const CMD_pop: CMD_index = 327;
pub const CMD_pedit: CMD_index = 326;
pub const CMD_perlfile: CMD_index = 325;
pub const CMD_perldo: CMD_index = 324;
pub const CMD_perl: CMD_index = 323;
pub const CMD_pclose: CMD_index = 322;
pub const CMD_pbuffer: CMD_index = 321;
pub const CMD_packloadall: CMD_index = 320;
pub const CMD_packadd: CMD_index = 319;
pub const CMD_print: CMD_index = 318;
pub const CMD_ownsyntax: CMD_index = 317;
pub const CMD_ounmenu: CMD_index = 316;
pub const CMD_ounmap: CMD_index = 315;
pub const CMD_options: CMD_index = 314;
pub const CMD_onoremenu: CMD_index = 313;
pub const CMD_onoremap: CMD_index = 312;
pub const CMD_only: CMD_index = 311;
pub const CMD_omenu: CMD_index = 310;
pub const CMD_omapclear: CMD_index = 309;
pub const CMD_omap: CMD_index = 308;
pub const CMD_oldfiles: CMD_index = 307;
pub const CMD_nunmenu: CMD_index = 306;
pub const CMD_nunmap: CMD_index = 305;
pub const CMD_number: CMD_index = 304;
pub const CMD_normal: CMD_index = 303;
pub const CMD_noswapfile: CMD_index = 302;
pub const CMD_noremenu: CMD_index = 301;
pub const CMD_noreabbrev: CMD_index = 300;
pub const CMD_nohlsearch: CMD_index = 299;
pub const CMD_noautocmd: CMD_index = 298;
pub const CMD_noremap: CMD_index = 297;
pub const CMD_nnoremenu: CMD_index = 296;
pub const CMD_nnoremap: CMD_index = 295;
pub const CMD_nmenu: CMD_index = 294;
pub const CMD_nmapclear: CMD_index = 293;
pub const CMD_nmap: CMD_index = 292;
pub const CMD_new: CMD_index = 291;
pub const CMD_next: CMD_index = 290;
pub const CMD_mzfile: CMD_index = 289;
pub const CMD_mzscheme: CMD_index = 288;
pub const CMD_mode: CMD_index = 287;
pub const CMD_mkview: CMD_index = 286;
pub const CMD_mkvimrc: CMD_index = 285;
pub const CMD_mkspell: CMD_index = 284;
pub const CMD_mksession: CMD_index = 283;
pub const CMD_mkexrc: CMD_index = 282;
pub const CMD_messages: CMD_index = 281;
pub const CMD_menutranslate: CMD_index = 280;
pub const CMD_menu: CMD_index = 279;
pub const CMD_match: CMD_index = 278;
pub const CMD_marks: CMD_index = 277;
pub const CMD_mapclear: CMD_index = 276;
pub const CMD_map: CMD_index = 275;
pub const CMD_make: CMD_index = 274;
pub const CMD_mark: CMD_index = 273;
pub const CMD_move: CMD_index = 272;
pub const CMD_lsp: CMD_index = 271;
pub const CMD_ls: CMD_index = 270;
pub const CMD_lwindow: CMD_index = 269;
pub const CMD_lvimgrepadd: CMD_index = 268;
pub const CMD_lvimgrep: CMD_index = 267;
pub const CMD_luafile: CMD_index = 266;
pub const CMD_luado: CMD_index = 265;
pub const CMD_lua: CMD_index = 264;
pub const CMD_lunmap: CMD_index = 263;
pub const CMD_ltag: CMD_index = 262;
pub const CMD_lrewind: CMD_index = 261;
pub const CMD_lpfile: CMD_index = 260;
pub const CMD_lprevious: CMD_index = 259;
pub const CMD_lopen: CMD_index = 258;
pub const CMD_lolder: CMD_index = 257;
pub const CMD_lockvar: CMD_index = 256;
pub const CMD_lockmarks: CMD_index = 255;
pub const CMD_loadkeymap: CMD_index = 254;
pub const CMD_loadview: CMD_index = 253;
pub const CMD_lnfile: CMD_index = 252;
pub const CMD_lnewer: CMD_index = 251;
pub const CMD_lnext: CMD_index = 250;
pub const CMD_lnoremap: CMD_index = 249;
pub const CMD_lmake: CMD_index = 248;
pub const CMD_lmapclear: CMD_index = 247;
pub const CMD_lmap: CMD_index = 246;
pub const CMD_llist: CMD_index = 245;
pub const CMD_llast: CMD_index = 244;
pub const CMD_ll: CMD_index = 243;
pub const CMD_lhistory: CMD_index = 242;
pub const CMD_lhelpgrep: CMD_index = 241;
pub const CMD_lgrepadd: CMD_index = 240;
pub const CMD_lgrep: CMD_index = 239;
pub const CMD_lgetexpr: CMD_index = 238;
pub const CMD_lgetbuffer: CMD_index = 237;
pub const CMD_lgetfile: CMD_index = 236;
pub const CMD_lfirst: CMD_index = 235;
pub const CMD_lfdo: CMD_index = 234;
pub const CMD_lfile: CMD_index = 233;
pub const CMD_lexpr: CMD_index = 232;
pub const CMD_let: CMD_index = 231;
pub const CMD_leftabove: CMD_index = 230;
pub const CMD_left: CMD_index = 229;
pub const CMD_ldo: CMD_index = 228;
pub const CMD_lclose: CMD_index = 227;
pub const CMD_lchdir: CMD_index = 226;
pub const CMD_lcd: CMD_index = 225;
pub const CMD_lbottom: CMD_index = 224;
pub const CMD_lbelow: CMD_index = 223;
pub const CMD_lbefore: CMD_index = 222;
pub const CMD_lbuffer: CMD_index = 221;
pub const CMD_later: CMD_index = 220;
pub const CMD_lafter: CMD_index = 219;
pub const CMD_laddfile: CMD_index = 218;
pub const CMD_laddbuffer: CMD_index = 217;
pub const CMD_laddexpr: CMD_index = 216;
pub const CMD_language: CMD_index = 215;
pub const CMD_labove: CMD_index = 214;
pub const CMD_last: CMD_index = 213;
pub const CMD_lNfile: CMD_index = 212;
pub const CMD_lNext: CMD_index = 211;
pub const CMD_list: CMD_index = 210;
pub const CMD_keepalt: CMD_index = 209;
pub const CMD_keeppatterns: CMD_index = 208;
pub const CMD_keepjumps: CMD_index = 207;
pub const CMD_keepmarks: CMD_index = 206;
pub const CMD_k: CMD_index = 205;
pub const CMD_jumps: CMD_index = 204;
pub const CMD_join: CMD_index = 203;
pub const CMD_iunmenu: CMD_index = 202;
pub const CMD_iunabbrev: CMD_index = 201;
pub const CMD_iunmap: CMD_index = 200;
pub const CMD_isplit: CMD_index = 199;
pub const CMD_isearch: CMD_index = 198;
pub const CMD_iput: CMD_index = 197;
pub const CMD_intro: CMD_index = 196;
pub const CMD_inoremenu: CMD_index = 195;
pub const CMD_inoreabbrev: CMD_index = 194;
pub const CMD_inoremap: CMD_index = 193;
pub const CMD_imenu: CMD_index = 192;
pub const CMD_imapclear: CMD_index = 191;
pub const CMD_imap: CMD_index = 190;
pub const CMD_ilist: CMD_index = 189;
pub const CMD_ijump: CMD_index = 188;
pub const CMD_if: CMD_index = 187;
pub const CMD_iabclear: CMD_index = 186;
pub const CMD_iabbrev: CMD_index = 185;
pub const CMD_insert: CMD_index = 184;
pub const CMD_horizontal: CMD_index = 183;
pub const CMD_history: CMD_index = 182;
pub const CMD_hide: CMD_index = 181;
pub const CMD_highlight: CMD_index = 180;
pub const CMD_helptags: CMD_index = 179;
pub const CMD_helpgrep: CMD_index = 178;
pub const CMD_helpclose: CMD_index = 177;
pub const CMD_help: CMD_index = 176;
pub const CMD_gvim: CMD_index = 175;
pub const CMD_gui: CMD_index = 174;
pub const CMD_grepadd: CMD_index = 173;
pub const CMD_grep: CMD_index = 172;
pub const CMD_goto: CMD_index = 171;
pub const CMD_global: CMD_index = 170;
pub const CMD_fclose: CMD_index = 169;
pub const CMD_function: CMD_index = 168;
pub const CMD_for: CMD_index = 167;
pub const CMD_foldopen: CMD_index = 166;
pub const CMD_folddoclosed: CMD_index = 165;
pub const CMD_folddoopen: CMD_index = 164;
pub const CMD_foldclose: CMD_index = 163;
pub const CMD_fold: CMD_index = 162;
pub const CMD_first: CMD_index = 161;
pub const CMD_finish: CMD_index = 160;
pub const CMD_finally: CMD_index = 159;
pub const CMD_find: CMD_index = 158;
pub const CMD_filter: CMD_index = 157;
pub const CMD_filetype: CMD_index = 156;
pub const CMD_files: CMD_index = 155;
pub const CMD_file: CMD_index = 154;
pub const CMD_exusage: CMD_index = 153;
pub const CMD_exit: CMD_index = 152;
pub const CMD_execute: CMD_index = 151;
pub const CMD_ex: CMD_index = 150;
pub const CMD_eval: CMD_index = 149;
pub const CMD_enew: CMD_index = 148;
pub const CMD_endwhile: CMD_index = 147;
pub const CMD_endtry: CMD_index = 146;
pub const CMD_endfor: CMD_index = 145;
pub const CMD_endfunction: CMD_index = 144;
pub const CMD_endif: CMD_index = 143;
pub const CMD_emenu: CMD_index = 142;
pub const CMD_elseif: CMD_index = 141;
pub const CMD_else: CMD_index = 140;
pub const CMD_echon: CMD_index = 139;
pub const CMD_echomsg: CMD_index = 138;
pub const CMD_echohl: CMD_index = 137;
pub const CMD_echoerr: CMD_index = 136;
pub const CMD_echo: CMD_index = 135;
pub const CMD_earlier: CMD_index = 134;
pub const CMD_edit: CMD_index = 133;
pub const CMD_dsplit: CMD_index = 132;
pub const CMD_dsearch: CMD_index = 131;
pub const CMD_drop: CMD_index = 130;
pub const CMD_doautoall: CMD_index = 129;
pub const CMD_doautocmd: CMD_index = 128;
pub const CMD_dlist: CMD_index = 127;
pub const CMD_djump: CMD_index = 126;
pub const CMD_digraphs: CMD_index = 125;
pub const CMD_diffthis: CMD_index = 124;
pub const CMD_diffsplit: CMD_index = 123;
pub const CMD_diffput: CMD_index = 122;
pub const CMD_diffpatch: CMD_index = 121;
pub const CMD_diffoff: CMD_index = 120;
pub const CMD_diffget: CMD_index = 119;
pub const CMD_diffupdate: CMD_index = 118;
pub const CMD_display: CMD_index = 117;
pub const CMD_detach: CMD_index = 116;
pub const CMD_delfunction: CMD_index = 115;
pub const CMD_delcommand: CMD_index = 114;
pub const CMD_defer: CMD_index = 113;
pub const CMD_debuggreedy: CMD_index = 112;
pub const CMD_debug: CMD_index = 111;
pub const CMD_delmarks: CMD_index = 110;
pub const CMD_delete: CMD_index = 109;
pub const CMD_cwindow: CMD_index = 108;
pub const CMD_cunmenu: CMD_index = 107;
pub const CMD_cunabbrev: CMD_index = 106;
pub const CMD_cunmap: CMD_index = 105;
pub const CMD_crewind: CMD_index = 104;
pub const CMD_cquit: CMD_index = 103;
pub const CMD_cpfile: CMD_index = 102;
pub const CMD_cprevious: CMD_index = 101;
pub const CMD_copen: CMD_index = 100;
pub const CMD_const: CMD_index = 99;
pub const CMD_connect: CMD_index = 98;
pub const CMD_confirm: CMD_index = 97;
pub const CMD_continue: CMD_index = 96;
pub const CMD_compiler: CMD_index = 95;
pub const CMD_comclear: CMD_index = 94;
pub const CMD_command: CMD_index = 93;
pub const CMD_colorscheme: CMD_index = 92;
pub const CMD_colder: CMD_index = 91;
pub const CMD_copy: CMD_index = 90;
pub const CMD_cnoremenu: CMD_index = 89;
pub const CMD_cnoreabbrev: CMD_index = 88;
pub const CMD_cnoremap: CMD_index = 87;
pub const CMD_cnfile: CMD_index = 86;
pub const CMD_cnewer: CMD_index = 85;
pub const CMD_cnext: CMD_index = 84;
pub const CMD_cmenu: CMD_index = 83;
pub const CMD_cmapclear: CMD_index = 82;
pub const CMD_cmap: CMD_index = 81;
pub const CMD_clearjumps: CMD_index = 80;
pub const CMD_close: CMD_index = 79;
pub const CMD_clast: CMD_index = 78;
pub const CMD_clist: CMD_index = 77;
pub const CMD_chistory: CMD_index = 76;
pub const CMD_checktime: CMD_index = 75;
pub const CMD_checkpath: CMD_index = 74;
pub const CMD_checkhealth: CMD_index = 73;
pub const CMD_changes: CMD_index = 72;
pub const CMD_chdir: CMD_index = 71;
pub const CMD_cgetexpr: CMD_index = 70;
pub const CMD_cgetbuffer: CMD_index = 69;
pub const CMD_cgetfile: CMD_index = 68;
pub const CMD_cfirst: CMD_index = 67;
pub const CMD_cfdo: CMD_index = 66;
pub const CMD_cfile: CMD_index = 65;
pub const CMD_cexpr: CMD_index = 64;
pub const CMD_center: CMD_index = 63;
pub const CMD_cdo: CMD_index = 62;
pub const CMD_cd: CMD_index = 61;
pub const CMD_cclose: CMD_index = 60;
pub const CMD_cc: CMD_index = 59;
pub const CMD_cbottom: CMD_index = 58;
pub const CMD_cbelow: CMD_index = 57;
pub const CMD_cbefore: CMD_index = 56;
pub const CMD_cbuffer: CMD_index = 55;
pub const CMD_catch: CMD_index = 54;
pub const CMD_call: CMD_index = 53;
pub const CMD_cafter: CMD_index = 52;
pub const CMD_caddfile: CMD_index = 51;
pub const CMD_caddexpr: CMD_index = 50;
pub const CMD_caddbuffer: CMD_index = 49;
pub const CMD_cabove: CMD_index = 48;
pub const CMD_cabclear: CMD_index = 47;
pub const CMD_cabbrev: CMD_index = 46;
pub const CMD_cNfile: CMD_index = 45;
pub const CMD_cNext: CMD_index = 44;
pub const CMD_change: CMD_index = 43;
pub const CMD_bwipeout: CMD_index = 42;
pub const CMD_bunload: CMD_index = 41;
pub const CMD_bufdo: CMD_index = 40;
pub const CMD_buffers: CMD_index = 39;
pub const CMD_browse: CMD_index = 38;
pub const CMD_breaklist: CMD_index = 37;
pub const CMD_breakdel: CMD_index = 36;
pub const CMD_breakadd: CMD_index = 35;
pub const CMD_break: CMD_index = 34;
pub const CMD_brewind: CMD_index = 33;
pub const CMD_bprevious: CMD_index = 32;
pub const CMD_botright: CMD_index = 31;
pub const CMD_bnext: CMD_index = 30;
pub const CMD_bmodified: CMD_index = 29;
pub const CMD_blast: CMD_index = 28;
pub const CMD_bfirst: CMD_index = 27;
pub const CMD_belowright: CMD_index = 26;
pub const CMD_bdelete: CMD_index = 25;
pub const CMD_balt: CMD_index = 24;
pub const CMD_badd: CMD_index = 23;
pub const CMD_ball: CMD_index = 22;
pub const CMD_bNext: CMD_index = 21;
pub const CMD_buffer: CMD_index = 20;
pub const CMD_aunmenu: CMD_index = 19;
pub const CMD_augroup: CMD_index = 18;
pub const CMD_autocmd: CMD_index = 17;
pub const CMD_ascii: CMD_index = 16;
pub const CMD_argument: CMD_index = 15;
pub const CMD_arglocal: CMD_index = 14;
pub const CMD_argglobal: CMD_index = 13;
pub const CMD_argedit: CMD_index = 12;
pub const CMD_argdedupe: CMD_index = 11;
pub const CMD_argdo: CMD_index = 10;
pub const CMD_argdelete: CMD_index = 9;
pub const CMD_argadd: CMD_index = 8;
pub const CMD_args: CMD_index = 7;
pub const CMD_anoremenu: CMD_index = 6;
pub const CMD_amenu: CMD_index = 5;
pub const CMD_all: CMD_index = 4;
pub const CMD_aboveleft: CMD_index = 3;
pub const CMD_abclear: CMD_index = 2;
pub const CMD_abbreviate: CMD_index = 1;
pub const CMD_append: CMD_index = 0;
pub const ADDR_NONE: cmd_addr_T = 11;
pub const ADDR_OTHER: cmd_addr_T = 10;
pub const ADDR_UNSIGNED: cmd_addr_T = 9;
pub const ADDR_QUICKFIX: cmd_addr_T = 8;
pub const ADDR_QUICKFIX_VALID: cmd_addr_T = 7;
pub const ADDR_TABS_RELATIVE: cmd_addr_T = 6;
pub const ADDR_TABS: cmd_addr_T = 5;
pub const ADDR_BUFFERS: cmd_addr_T = 4;
pub const ADDR_LOADED_BUFFERS: cmd_addr_T = 3;
pub const ADDR_ARGUMENTS: cmd_addr_T = 2;
pub const ADDR_WINDOWS: cmd_addr_T = 1;
pub const ADDR_LINES: cmd_addr_T = 0;
pub const NUM_EVENTS: auto_event = 145;
pub const EVENT_WINSCROLLED: auto_event = 144;
pub const EVENT_WINRESIZED: auto_event = 143;
pub const EVENT_WINNEWPRE: auto_event = 142;
pub const EVENT_WINNEW: auto_event = 141;
pub const EVENT_WINLEAVE: auto_event = 140;
pub const EVENT_WINENTER: auto_event = 139;
pub const EVENT_WINCLOSED: auto_event = 138;
pub const EVENT_VIMSUSPEND: auto_event = 137;
pub const EVENT_VIMRESUME: auto_event = 136;
pub const EVENT_VIMRESIZED: auto_event = 135;
pub const EVENT_VIMLEAVEPRE: auto_event = 134;
pub const EVENT_VIMLEAVE: auto_event = 133;
pub const EVENT_VIMENTER: auto_event = 132;
pub const EVENT_USER: auto_event = 131;
pub const EVENT_UILEAVE: auto_event = 130;
pub const EVENT_UIENTER: auto_event = 129;
pub const EVENT_TEXTYANKPOST: auto_event = 128;
pub const EVENT_TEXTCHANGEDT: auto_event = 127;
pub const EVENT_TEXTCHANGEDP: auto_event = 126;
pub const EVENT_TEXTCHANGEDI: auto_event = 125;
pub const EVENT_TEXTCHANGED: auto_event = 124;
pub const EVENT_TERMRESPONSE: auto_event = 123;
pub const EVENT_TERMREQUEST: auto_event = 122;
pub const EVENT_TERMOPEN: auto_event = 121;
pub const EVENT_TERMLEAVE: auto_event = 120;
pub const EVENT_TERMENTER: auto_event = 119;
pub const EVENT_TERMCLOSE: auto_event = 118;
pub const EVENT_TERMCHANGED: auto_event = 117;
pub const EVENT_TABNEWENTERED: auto_event = 116;
pub const EVENT_TABNEW: auto_event = 115;
pub const EVENT_TABLEAVE: auto_event = 114;
pub const EVENT_TABENTER: auto_event = 113;
pub const EVENT_TABCLOSEDPRE: auto_event = 112;
pub const EVENT_TABCLOSED: auto_event = 111;
pub const EVENT_SYNTAX: auto_event = 110;
pub const EVENT_SWAPEXISTS: auto_event = 109;
pub const EVENT_STDINREADPRE: auto_event = 108;
pub const EVENT_STDINREADPOST: auto_event = 107;
pub const EVENT_SPELLFILEMISSING: auto_event = 106;
pub const EVENT_SOURCEPRE: auto_event = 105;
pub const EVENT_SOURCEPOST: auto_event = 104;
pub const EVENT_SOURCECMD: auto_event = 103;
pub const EVENT_SIGNAL: auto_event = 102;
pub const EVENT_SHELLFILTERPOST: auto_event = 101;
pub const EVENT_SHELLCMDPOST: auto_event = 100;
pub const EVENT_SESSIONWRITEPOST: auto_event = 99;
pub const EVENT_SESSIONLOADPRE: auto_event = 98;
pub const EVENT_SESSIONLOADPOST: auto_event = 97;
pub const EVENT_SEARCHWRAPPED: auto_event = 96;
pub const EVENT_SAFESTATE: auto_event = 95;
pub const EVENT_REMOTEREPLY: auto_event = 94;
pub const EVENT_RECORDINGLEAVE: auto_event = 93;
pub const EVENT_RECORDINGENTER: auto_event = 92;
pub const EVENT_QUITPRE: auto_event = 91;
pub const EVENT_QUICKFIXCMDPRE: auto_event = 90;
pub const EVENT_QUICKFIXCMDPOST: auto_event = 89;
pub const EVENT_PROGRESS: auto_event = 88;
pub const EVENT_PACKCHANGEDPRE: auto_event = 87;
pub const EVENT_PACKCHANGED: auto_event = 86;
pub const EVENT_OPTIONSET: auto_event = 85;
pub const EVENT_MODECHANGED: auto_event = 84;
pub const EVENT_MENUPOPUP: auto_event = 83;
pub const EVENT_MARKSET: auto_event = 82;
pub const EVENT_LSPTOKENUPDATE: auto_event = 81;
pub const EVENT_LSPREQUEST: auto_event = 80;
pub const EVENT_LSPPROGRESS: auto_event = 79;
pub const EVENT_LSPNOTIFY: auto_event = 78;
pub const EVENT_LSPDETACH: auto_event = 77;
pub const EVENT_LSPATTACH: auto_event = 76;
pub const EVENT_INSERTLEAVEPRE: auto_event = 75;
pub const EVENT_INSERTLEAVE: auto_event = 74;
pub const EVENT_INSERTENTER: auto_event = 73;
pub const EVENT_INSERTCHARPRE: auto_event = 72;
pub const EVENT_INSERTCHANGE: auto_event = 71;
pub const EVENT_GUIFAILED: auto_event = 70;
pub const EVENT_GUIENTER: auto_event = 69;
pub const EVENT_FUNCUNDEFINED: auto_event = 68;
pub const EVENT_FOCUSLOST: auto_event = 67;
pub const EVENT_FOCUSGAINED: auto_event = 66;
pub const EVENT_FILTERWRITEPRE: auto_event = 65;
pub const EVENT_FILTERWRITEPOST: auto_event = 64;
pub const EVENT_FILTERREADPRE: auto_event = 63;
pub const EVENT_FILTERREADPOST: auto_event = 62;
pub const EVENT_FILEWRITEPRE: auto_event = 61;
pub const EVENT_FILEWRITEPOST: auto_event = 60;
pub const EVENT_FILEWRITECMD: auto_event = 59;
pub const EVENT_FILETYPE: auto_event = 58;
pub const EVENT_FILEREADPRE: auto_event = 57;
pub const EVENT_FILEREADPOST: auto_event = 56;
pub const EVENT_FILEREADCMD: auto_event = 55;
pub const EVENT_FILEENCODING: auto_event = 54;
pub const EVENT_FILECHANGEDSHELLPOST: auto_event = 53;
pub const EVENT_FILECHANGEDSHELL: auto_event = 52;
pub const EVENT_FILECHANGEDRO: auto_event = 51;
pub const EVENT_FILEAPPENDPRE: auto_event = 50;
pub const EVENT_FILEAPPENDPOST: auto_event = 49;
pub const EVENT_FILEAPPENDCMD: auto_event = 48;
pub const EVENT_EXITPRE: auto_event = 47;
pub const EVENT_ENCODINGCHANGED: auto_event = 46;
pub const EVENT_DIRCHANGEDPRE: auto_event = 45;
pub const EVENT_DIRCHANGED: auto_event = 44;
pub const EVENT_DIFFUPDATED: auto_event = 43;
pub const EVENT_DIAGNOSTICCHANGED: auto_event = 42;
pub const EVENT_CURSORMOVEDI: auto_event = 41;
pub const EVENT_CURSORMOVEDC: auto_event = 40;
pub const EVENT_CURSORMOVED: auto_event = 39;
pub const EVENT_CURSORHOLDI: auto_event = 38;
pub const EVENT_CURSORHOLD: auto_event = 37;
pub const EVENT_COMPLETEDONEPRE: auto_event = 36;
pub const EVENT_COMPLETEDONE: auto_event = 35;
pub const EVENT_COMPLETECHANGED: auto_event = 34;
pub const EVENT_COLORSCHEMEPRE: auto_event = 33;
pub const EVENT_COLORSCHEME: auto_event = 32;
pub const EVENT_CMDWINLEAVE: auto_event = 31;
pub const EVENT_CMDWINENTER: auto_event = 30;
pub const EVENT_CMDUNDEFINED: auto_event = 29;
pub const EVENT_CMDLINELEAVEPRE: auto_event = 28;
pub const EVENT_CMDLINELEAVE: auto_event = 27;
pub const EVENT_CMDLINEENTER: auto_event = 26;
pub const EVENT_CMDLINECHANGED: auto_event = 25;
pub const EVENT_CHANOPEN: auto_event = 24;
pub const EVENT_CHANINFO: auto_event = 23;
pub const EVENT_BUFWRITEPRE: auto_event = 22;
pub const EVENT_BUFWRITEPOST: auto_event = 21;
pub const EVENT_BUFWRITECMD: auto_event = 20;
pub const EVENT_BUFWRITE: auto_event = 19;
pub const EVENT_BUFWIPEOUT: auto_event = 18;
pub const EVENT_BUFWINLEAVE: auto_event = 17;
pub const EVENT_BUFWINENTER: auto_event = 16;
pub const EVENT_BUFUNLOAD: auto_event = 15;
pub const EVENT_BUFREADPRE: auto_event = 14;
pub const EVENT_BUFREADPOST: auto_event = 13;
pub const EVENT_BUFREADCMD: auto_event = 12;
pub const EVENT_BUFREAD: auto_event = 11;
pub const EVENT_BUFNEWFILE: auto_event = 10;
pub const EVENT_BUFNEW: auto_event = 9;
pub const EVENT_BUFMODIFIEDSET: auto_event = 8;
pub const EVENT_BUFLEAVE: auto_event = 7;
pub const EVENT_BUFHIDDEN: auto_event = 6;
pub const EVENT_BUFFILEPRE: auto_event = 5;
pub const EVENT_BUFFILEPOST: auto_event = 4;
pub const EVENT_BUFENTER: auto_event = 3;
pub const EVENT_BUFDELETE: auto_event = 2;
pub const EVENT_BUFCREATE: auto_event = 1;
pub const EVENT_BUFADD: auto_event = 0;
pub const kChannelStreamInternal: ChannelStreamType = 4;
pub const kChannelStreamStderr: ChannelStreamType = 3;
pub const kChannelStreamStdio: ChannelStreamType = 2;
pub const kChannelStreamSocket: ChannelStreamType = 1;
pub const kChannelStreamProc: ChannelStreamType = 0;
pub const kChannelPartAll: ChannelPart = 4;
pub const kChannelPartRpc: ChannelPart = 3;
pub const kChannelPartStderr: ChannelPart = 2;
pub const kChannelPartStdout: ChannelPart = 1;
pub const kChannelPartStdin: ChannelPart = 0;
pub const kChannelStdinNull: ChannelStdinMode = 1;
pub const kChannelStdinPipe: ChannelStdinMode = 0;
pub const kUIExtCount: UIExtension = 10;
pub const kUIFloatDebug: UIExtension = 9;
pub const kUITermColors: UIExtension = 8;
pub const kUIHlState: UIExtension = 7;
pub const kUIMultigrid: UIExtension = 6;
pub const kUILinegrid: UIExtension = 5;
pub const kUIMessages: UIExtension = 4;
pub const kUIWildmenu: UIExtension = 3;
pub const kUITabline: UIExtension = 2;
pub const kUIPopupmenu: UIExtension = 1;
pub const kUICmdline: UIExtension = 0;
pub const kClientTypePlugin: ClientType = 4;
pub const kClientTypeHost: ClientType = 3;
pub const kClientTypeEmbedder: ClientType = 2;
pub const kClientTypeUi: ClientType = 1;
pub const kClientTypeMsgpackRpc: ClientType = 5;
pub const kClientTypeRemote: ClientType = 0;
pub const kClientTypeUnknown: ClientType = -1;
pub type C2Rust_Unnamed_43 = ::core::ffi::c_uint;
pub const WILD_PUM_WANT: C2Rust_Unnamed_43 = 13;
pub const WILD_PAGEDOWN: C2Rust_Unnamed_43 = 12;
pub const WILD_PAGEUP: C2Rust_Unnamed_43 = 11;
pub const WILD_APPLY: C2Rust_Unnamed_43 = 10;
pub const WILD_CANCEL: C2Rust_Unnamed_43 = 9;
pub const WILD_ALL_KEEP: C2Rust_Unnamed_43 = 8;
pub const WILD_LONGEST: C2Rust_Unnamed_43 = 7;
pub const WILD_ALL: C2Rust_Unnamed_43 = 6;
pub const WILD_PREV: C2Rust_Unnamed_43 = 5;
pub const WILD_NEXT: C2Rust_Unnamed_43 = 4;
pub const WILD_EXPAND_KEEP: C2Rust_Unnamed_43 = 3;
pub const WILD_EXPAND_FREE: C2Rust_Unnamed_43 = 2;
pub const WILD_FREE: C2Rust_Unnamed_43 = 1;
pub type C2Rust_Unnamed_44 = ::core::ffi::c_uint;
pub const WILD_FUNC_TRIGGER: C2Rust_Unnamed_44 = 65536;
pub const WILD_MAY_EXPAND_PATTERN: C2Rust_Unnamed_44 = 32768;
pub const WILD_NOSELECT: C2Rust_Unnamed_44 = 16384;
pub const BUF_DIFF_FILTER: C2Rust_Unnamed_44 = 8192;
pub const WILD_BUFLASTUSED: C2Rust_Unnamed_44 = 4096;
pub const WILD_NOERROR: C2Rust_Unnamed_44 = 2048;
pub const WILD_IGNORE_COMPLETESLASH: C2Rust_Unnamed_44 = 1024;
pub const WILD_ALLLINKS: C2Rust_Unnamed_44 = 512;
pub const WILD_ICASE: C2Rust_Unnamed_44 = 256;
pub const WILD_ESCAPE: C2Rust_Unnamed_44 = 128;
pub const WILD_SILENT: C2Rust_Unnamed_44 = 64;
pub const WILD_KEEP_ALL: C2Rust_Unnamed_44 = 32;
pub const WILD_ADD_SLASH: C2Rust_Unnamed_44 = 16;
pub const WILD_NO_BEEP: C2Rust_Unnamed_44 = 8;
pub const WILD_USE_NL: C2Rust_Unnamed_44 = 4;
pub const WILD_HOME_REPLACE: C2Rust_Unnamed_44 = 2;
pub const WILD_LIST_NOTFOUND: C2Rust_Unnamed_44 = 1;
pub type C2Rust_Unnamed_45 = ::core::ffi::c_uint;
pub const kCtxFuncs: C2Rust_Unnamed_45 = 32;
pub const kCtxSFuncs: C2Rust_Unnamed_45 = 16;
pub const kCtxGVars: C2Rust_Unnamed_45 = 8;
pub const kCtxBufs: C2Rust_Unnamed_45 = 4;
pub const kCtxJumps: C2Rust_Unnamed_45 = 2;
pub const kCtxRegs: C2Rust_Unnamed_45 = 1;
pub type C2Rust_Unnamed_46 = ::core::ffi::c_uint;
pub const BASE_LAST: C2Rust_Unnamed_46 = 255;
pub const BASE_NONE: C2Rust_Unnamed_46 = 0;
pub const RE_SEARCH: C2Rust_Unnamed_64 = 0;
pub const SEARCH_KEEP: C2Rust_Unnamed_63 = 1024;
pub const SEARCH_START: C2Rust_Unnamed_63 = 256;
pub const HL_CONCEAL: C2Rust_Unnamed_65 = 131072;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_47 {
    pub split: C2Rust_Unnamed_48,
    pub prof: proftime_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_48 {
    pub low: int32_t,
    pub high: int32_t,
}
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub const OP_NOP: C2Rust_Unnamed_62 = 0;
pub const MODE_CMDLINE: C2Rust_Unnamed_59 = 8;
pub const SMT_RARE: smt_T = 2;
pub const SMT_BAD: smt_T = 1;
pub const SMT_ALL: smt_T = 0;
pub type SomeMatchType = ::core::ffi::c_uint;
pub const kSomeMatchStrPos: SomeMatchType = 4;
pub const kSomeMatchStr: SomeMatchType = 3;
pub const kSomeMatchList: SomeMatchType = 2;
pub const kSomeMatchEnd: SomeMatchType = 1;
pub const kSomeMatch: SomeMatchType = 0;
pub const VSE_NONE: C2Rust_Unnamed_57 = 0;
pub const kRetMulti: LuaRetMode = 3;
pub const kRetLuaref: LuaRetMode = 2;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
pub const ETYPE_SPELL: etype_T = 9;
pub const ETYPE_INTERNAL: etype_T = 8;
pub const ETYPE_ENV: etype_T = 7;
pub const ETYPE_ARGS: etype_T = 6;
pub const ETYPE_EXCEPT: etype_T = 5;
pub const ETYPE_MODELINE: etype_T = 4;
pub const ETYPE_AUCMD: etype_T = 3;
pub const ETYPE_UFUNC: etype_T = 2;
pub const ETYPE_SCRIPT: etype_T = 1;
pub const ETYPE_TOP: etype_T = 0;
pub const VV_EXITREASON: VimVarIndex = 105;
pub const VV_STARTTIME: VimVarIndex = 104;
pub const VV_VIRTNUM: VimVarIndex = 103;
pub const VV_RELNUM: VimVarIndex = 102;
pub const VV_LUA: VimVarIndex = 101;
pub const VV__NULL_BLOB: VimVarIndex = 100;
pub const VV__NULL_DICT: VimVarIndex = 99;
pub const VV__NULL_LIST: VimVarIndex = 98;
pub const VV__NULL_STRING: VimVarIndex = 97;
pub const VV_MSGPACK_TYPES: VimVarIndex = 96;
pub const VV_STDERR: VimVarIndex = 95;
pub const VV_VIM_DID_INIT: VimVarIndex = 94;
pub const VV_STACKTRACE: VimVarIndex = 93;
pub const VV_MAXCOL: VimVarIndex = 92;
pub const VV_EXITING: VimVarIndex = 91;
pub const VV_COLLATE: VimVarIndex = 90;
pub const VV_ARGV: VimVarIndex = 89;
pub const VV_ARGF: VimVarIndex = 88;
pub const VV_ECHOSPACE: VimVarIndex = 87;
pub const VV_VERSIONLONG: VimVarIndex = 86;
pub const VV_EVENT: VimVarIndex = 85;
pub const VV_TYPE_BLOB: VimVarIndex = 84;
pub const VV_TYPE_BOOL: VimVarIndex = 83;
pub const VV_TYPE_FLOAT: VimVarIndex = 82;
pub const VV_TYPE_DICT: VimVarIndex = 81;
pub const VV_TYPE_LIST: VimVarIndex = 80;
pub const VV_TYPE_FUNC: VimVarIndex = 79;
pub const VV_TYPE_STRING: VimVarIndex = 78;
pub const VV_TYPE_NUMBER: VimVarIndex = 77;
pub const VV_TESTING: VimVarIndex = 76;
pub const VV_VIM_DID_ENTER: VimVarIndex = 75;
pub const VV_NUMBERSIZE: VimVarIndex = 74;
pub const VV_NUMBERMIN: VimVarIndex = 73;
pub const VV_NUMBERMAX: VimVarIndex = 72;
pub const VV_NULL: VimVarIndex = 71;
pub const VV_TRUE: VimVarIndex = 70;
pub const VV_FALSE: VimVarIndex = 69;
pub const VV_ERRORS: VimVarIndex = 68;
pub const VV_OPTION_TYPE: VimVarIndex = 67;
pub const VV_OPTION_COMMAND: VimVarIndex = 66;
pub const VV_OPTION_OLDGLOBAL: VimVarIndex = 65;
pub const VV_OPTION_OLDLOCAL: VimVarIndex = 64;
pub const VV_OPTION_OLD: VimVarIndex = 63;
pub const VV_OPTION_NEW: VimVarIndex = 62;
pub const VV_COMPLETED_ITEM: VimVarIndex = 61;
pub const VV_PROGPATH: VimVarIndex = 60;
pub const VV_WINDOWID: VimVarIndex = 59;
pub const VV_OLDFILES: VimVarIndex = 58;
pub const VV_HLSEARCH: VimVarIndex = 57;
pub const VV_SEARCHFORWARD: VimVarIndex = 56;
pub const VV_OP: VimVarIndex = 55;
pub const VV_MOUSE_COL: VimVarIndex = 54;
pub const VV_MOUSE_LNUM: VimVarIndex = 53;
pub const VV_MOUSE_WINID: VimVarIndex = 52;
pub const VV_MOUSE_WIN: VimVarIndex = 51;
pub const VV_CHAR: VimVarIndex = 50;
pub const VV_SWAPCOMMAND: VimVarIndex = 49;
pub const VV_SWAPCHOICE: VimVarIndex = 48;
pub const VV_SWAPNAME: VimVarIndex = 47;
pub const VV_SCROLLSTART: VimVarIndex = 46;
pub const VV_BEVAL_TEXT: VimVarIndex = 45;
pub const VV_BEVAL_COL: VimVarIndex = 44;
pub const VV_BEVAL_LNUM: VimVarIndex = 43;
pub const VV_BEVAL_WINID: VimVarIndex = 42;
pub const VV_BEVAL_WINNR: VimVarIndex = 41;
pub const VV_BEVAL_BUFNR: VimVarIndex = 40;
pub const VV_FCS_CHOICE: VimVarIndex = 39;
pub const VV_FCS_REASON: VimVarIndex = 38;
pub const VV_PROFILING: VimVarIndex = 37;
pub const VV_KEY: VimVarIndex = 36;
pub const VV_VAL: VimVarIndex = 35;
pub const VV_INSERTMODE: VimVarIndex = 34;
pub const VV_CMDBANG: VimVarIndex = 33;
pub const VV_REG: VimVarIndex = 32;
pub const VV_THROWPOINT: VimVarIndex = 31;
pub const VV_EXCEPTION: VimVarIndex = 30;
pub const VV_DYING: VimVarIndex = 29;
pub const VV_SEND_SERVER: VimVarIndex = 28;
pub const VV_PROGNAME: VimVarIndex = 27;
pub const VV_FOLDLEVEL: VimVarIndex = 26;
pub const VV_FOLDDASHES: VimVarIndex = 25;
pub const VV_FOLDEND: VimVarIndex = 24;
pub const VV_FOLDSTART: VimVarIndex = 23;
pub const VV_CMDARG: VimVarIndex = 22;
pub const VV_FNAME_DIFF: VimVarIndex = 21;
pub const VV_FNAME_NEW: VimVarIndex = 20;
pub const VV_FNAME_OUT: VimVarIndex = 19;
pub const VV_FNAME_IN: VimVarIndex = 18;
pub const VV_CC_TO: VimVarIndex = 17;
pub const VV_CC_FROM: VimVarIndex = 16;
pub const VV_CTYPE: VimVarIndex = 15;
pub const VV_LC_TIME: VimVarIndex = 14;
pub const VV_LANG: VimVarIndex = 13;
pub const VV_FNAME: VimVarIndex = 12;
pub const VV_TERMRESPONSE: VimVarIndex = 11;
pub const VV_TERMREQUEST: VimVarIndex = 10;
pub const VV_LNUM: VimVarIndex = 9;
pub const VV_VERSION: VimVarIndex = 8;
pub const VV_THIS_SESSION: VimVarIndex = 7;
pub const VV_SHELL_ERROR: VimVarIndex = 6;
pub const VV_STATUSMSG: VimVarIndex = 5;
pub const VV_WARNINGMSG: VimVarIndex = 4;
pub const VV_ERRMSG: VimVarIndex = 3;
pub const VV_PREVCOUNT: VimVarIndex = 2;
pub const VV_COUNT1: VimVarIndex = 1;
pub const VV_COUNT: VimVarIndex = 0;
pub const PLUS_REGISTER: C2Rust_Unnamed_60 = 38;
pub const STAR_REGISTER: C2Rust_Unnamed_60 = 37;
pub const DELETION_REGISTER: C2Rust_Unnamed_60 = 36;
pub const kGRegList: GRegFlags = 4;
pub const kGRegExprSrc: GRegFlags = 2;
pub const SEARCH_COL: C2Rust_Unnamed_63 = 4096;
pub const SEARCH_END: C2Rust_Unnamed_63 = 64;
pub const MENU_ALL_MODES: C2Rust_Unnamed_58 = 127;
pub const CONV_NONE: C2Rust_Unnamed_53 = 0;
pub const GLV_READ_ONLY: C2Rust_Unnamed_67 = 16;
pub const GLV_NO_AUTOLOAD: C2Rust_Unnamed_67 = 4;
pub const TFN_NO_DEREF: C2Rust_Unnamed_66 = 8;
pub const TFN_NO_AUTOLOAD: C2Rust_Unnamed_66 = 4;
pub const TFN_QUIET: C2Rust_Unnamed_66 = 2;
pub const TFN_INT: C2Rust_Unnamed_66 = 1;
pub const VIM_GENERIC: C2Rust_Unnamed_54 = 0;
pub const VIM_WARNING: C2Rust_Unnamed_54 = 2;
pub const VIM_INFO: C2Rust_Unnamed_54 = 3;
pub const VIM_QUESTION: C2Rust_Unnamed_54 = 4;
pub const VIM_ERROR: C2Rust_Unnamed_54 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_50 {
    pub low: int32_t,
    pub high: int32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_51 {
    pub split: C2Rust_Unnamed_50,
    pub prof: proftime_T,
}
pub const DOCMD_KEYTYPED: C2Rust_Unnamed_56 = 8;
pub const DOCMD_REPEAT: C2Rust_Unnamed_56 = 4;
pub const DOCMD_VERBOSE: C2Rust_Unnamed_56 = 1;
pub const DOCMD_NOWAIT: C2Rust_Unnamed_56 = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct GetListLineCookie {
    pub l: *const list_T,
    pub li: *const listitem_T,
}
pub const kXDGDataDirs: XDGVarType = 6;
pub const kXDGConfigDirs: XDGVarType = 5;
pub const kXDGRuntimeDir: XDGVarType = 4;
pub const kXDGStateHome: XDGVarType = 3;
pub const kXDGCacheHome: XDGVarType = 2;
pub const kXDGDataHome: XDGVarType = 1;
pub const kXDGConfigHome: XDGVarType = 0;
pub const kXDGNone: XDGVarType = -1;
pub const YREG_YANK: C2Rust_Unnamed_61 = 1;
pub const FCERR_TOOMANY: C2Rust_Unnamed_55 = 1;
pub const FCERR_TOOFEW: C2Rust_Unnamed_55 = 2;
pub const FCERR_NONE: C2Rust_Unnamed_55 = 5;
pub const FCERR_UNKNOWN: C2Rust_Unnamed_55 = 0;
pub const FCERR_NOTMETHOD: C2Rust_Unnamed_55 = 8;
pub type C2Rust_Unnamed_53 = ::core::ffi::c_uint;
pub const CONV_ICONV: C2Rust_Unnamed_53 = 5;
pub const CONV_TO_LATIN9: C2Rust_Unnamed_53 = 4;
pub const CONV_TO_LATIN1: C2Rust_Unnamed_53 = 3;
pub const CONV_9_TO_UTF8: C2Rust_Unnamed_53 = 2;
pub const CONV_TO_UTF8: C2Rust_Unnamed_53 = 1;
pub type C2Rust_Unnamed_54 = ::core::ffi::c_uint;
pub const VIM_LAST_TYPE: C2Rust_Unnamed_54 = 4;
pub type C2Rust_Unnamed_55 = ::core::ffi::c_uint;
pub const FCERR_DELETED: C2Rust_Unnamed_55 = 7;
pub const FCERR_OTHER: C2Rust_Unnamed_55 = 6;
pub const FCERR_DICT: C2Rust_Unnamed_55 = 4;
pub const FCERR_SCRIPT: C2Rust_Unnamed_55 = 3;
pub type C2Rust_Unnamed_56 = ::core::ffi::c_uint;
pub const DOCMD_KEEPLINE: C2Rust_Unnamed_56 = 32;
pub const DOCMD_EXCRESET: C2Rust_Unnamed_56 = 16;
pub type C2Rust_Unnamed_57 = ::core::ffi::c_uint;
pub const VSE_BUFFER: C2Rust_Unnamed_57 = 2;
pub const VSE_SHELL: C2Rust_Unnamed_57 = 1;
pub type C2Rust_Unnamed_58 = ::core::ffi::c_uint;
pub const MENU_TIP_MODE: C2Rust_Unnamed_58 = 128;
pub const MENU_TERMINAL_MODE: C2Rust_Unnamed_58 = 64;
pub const MENU_CMDLINE_MODE: C2Rust_Unnamed_58 = 32;
pub const MENU_INSERT_MODE: C2Rust_Unnamed_58 = 16;
pub const MENU_OP_PENDING_MODE: C2Rust_Unnamed_58 = 8;
pub const MENU_SELECT_MODE: C2Rust_Unnamed_58 = 4;
pub const MENU_VISUAL_MODE: C2Rust_Unnamed_58 = 2;
pub const MENU_NORMAL_MODE: C2Rust_Unnamed_58 = 1;
pub type C2Rust_Unnamed_59 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_59 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_59 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_59 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_59 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_59 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_59 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_59 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_59 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_59 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_59 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_59 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_59 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_59 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_59 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_59 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_59 = 16;
pub const MODE_OP_PENDING: C2Rust_Unnamed_59 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_59 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_59 = 1;
pub type C2Rust_Unnamed_60 = ::core::ffi::c_uint;
pub const NUM_REGISTERS: C2Rust_Unnamed_60 = 39;
pub const NUM_SAVED_REGISTERS: C2Rust_Unnamed_60 = 37;
pub const kGRegNoExpr: GRegFlags = 1;
pub type C2Rust_Unnamed_61 = ::core::ffi::c_uint;
pub const YREG_PUT: C2Rust_Unnamed_61 = 2;
pub const YREG_PASTE: C2Rust_Unnamed_61 = 0;
pub type C2Rust_Unnamed_62 = ::core::ffi::c_uint;
pub const OP_NR_SUB: C2Rust_Unnamed_62 = 29;
pub const OP_NR_ADD: C2Rust_Unnamed_62 = 28;
pub const OP_FUNCTION: C2Rust_Unnamed_62 = 27;
pub const OP_FORMAT2: C2Rust_Unnamed_62 = 26;
pub const OP_FOLDDELREC: C2Rust_Unnamed_62 = 25;
pub const OP_FOLDDEL: C2Rust_Unnamed_62 = 24;
pub const OP_FOLDCLOSEREC: C2Rust_Unnamed_62 = 23;
pub const OP_FOLDCLOSE: C2Rust_Unnamed_62 = 22;
pub const OP_FOLDOPENREC: C2Rust_Unnamed_62 = 21;
pub const OP_FOLDOPEN: C2Rust_Unnamed_62 = 20;
pub const OP_FOLD: C2Rust_Unnamed_62 = 19;
pub const OP_APPEND: C2Rust_Unnamed_62 = 18;
pub const OP_INSERT: C2Rust_Unnamed_62 = 17;
pub const OP_REPLACE: C2Rust_Unnamed_62 = 16;
pub const OP_ROT13: C2Rust_Unnamed_62 = 15;
pub const OP_JOIN_NS: C2Rust_Unnamed_62 = 14;
pub const OP_JOIN: C2Rust_Unnamed_62 = 13;
pub const OP_LOWER: C2Rust_Unnamed_62 = 12;
pub const OP_UPPER: C2Rust_Unnamed_62 = 11;
pub const OP_COLON: C2Rust_Unnamed_62 = 10;
pub const OP_FORMAT: C2Rust_Unnamed_62 = 9;
pub const OP_INDENT: C2Rust_Unnamed_62 = 8;
pub const OP_TILDE: C2Rust_Unnamed_62 = 7;
pub const OP_FILTER: C2Rust_Unnamed_62 = 6;
pub const OP_RSHIFT: C2Rust_Unnamed_62 = 5;
pub const OP_LSHIFT: C2Rust_Unnamed_62 = 4;
pub const OP_CHANGE: C2Rust_Unnamed_62 = 3;
pub const OP_YANK: C2Rust_Unnamed_62 = 2;
pub const OP_DELETE: C2Rust_Unnamed_62 = 1;
pub type C2Rust_Unnamed_63 = ::core::ffi::c_uint;
pub const SEARCH_PEEK: C2Rust_Unnamed_63 = 2048;
pub const SEARCH_MARK: C2Rust_Unnamed_63 = 512;
pub const SEARCH_NOOF: C2Rust_Unnamed_63 = 128;
pub const SEARCH_HIS: C2Rust_Unnamed_63 = 32;
pub const SEARCH_OPT: C2Rust_Unnamed_63 = 16;
pub const SEARCH_NFMSG: C2Rust_Unnamed_63 = 8;
pub const SEARCH_MSG: C2Rust_Unnamed_63 = 12;
pub const SEARCH_ECHO: C2Rust_Unnamed_63 = 2;
pub const SEARCH_REV: C2Rust_Unnamed_63 = 1;
pub type C2Rust_Unnamed_64 = ::core::ffi::c_uint;
pub const RE_LAST: C2Rust_Unnamed_64 = 2;
pub const RE_BOTH: C2Rust_Unnamed_64 = 2;
pub const RE_SUBST: C2Rust_Unnamed_64 = 1;
pub type C2Rust_Unnamed_65 = ::core::ffi::c_uint;
pub const HL_INCLUDED_TOPLEVEL: C2Rust_Unnamed_65 = 524288;
pub const HL_CONCEALENDS: C2Rust_Unnamed_65 = 262144;
pub const HL_TRANS_CONT: C2Rust_Unnamed_65 = 65536;
pub const HL_MATCHCONT: C2Rust_Unnamed_65 = 32768;
pub const HL_EXTEND: C2Rust_Unnamed_65 = 16384;
pub const HL_FOLD: C2Rust_Unnamed_65 = 8192;
pub const HL_DISPLAY: C2Rust_Unnamed_65 = 4096;
pub const HL_EXCLUDENL: C2Rust_Unnamed_65 = 2048;
pub const HL_KEEPEND: C2Rust_Unnamed_65 = 1024;
pub const HL_SKIPEMPTY: C2Rust_Unnamed_65 = 512;
pub const HL_SKIPWHITE: C2Rust_Unnamed_65 = 256;
pub const HL_SKIPNL: C2Rust_Unnamed_65 = 128;
pub const HL_MATCH: C2Rust_Unnamed_65 = 64;
pub const HL_SYNC_THERE: C2Rust_Unnamed_65 = 32;
pub const HL_SYNC_HERE: C2Rust_Unnamed_65 = 16;
pub const HL_HAS_EOL: C2Rust_Unnamed_65 = 8;
pub const HL_ONELINE: C2Rust_Unnamed_65 = 4;
pub const HL_TRANSP: C2Rust_Unnamed_65 = 2;
pub const HL_CONTAINED: C2Rust_Unnamed_65 = 1;
pub type C2Rust_Unnamed_66 = ::core::ffi::c_uint;
pub const TFN_READ_ONLY: C2Rust_Unnamed_66 = 16;
pub type C2Rust_Unnamed_67 = ::core::ffi::c_uint;
pub const GLV_QUIET: C2Rust_Unnamed_67 = 2;
pub const INT64_MIN: ::core::ffi::c_long =
    -9223372036854775807 as ::core::ffi::c_long - 1 as ::core::ffi::c_long;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_1: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const ARENA_BLOCK_SIZE: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const STRING_INIT: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
pub const INTERNAL_CALL_MASK: uint64_t = (1 as ::core::ffi::c_int as uint64_t)
    << ::core::mem::size_of::<uint64_t>()
        .wrapping_mul(8 as usize)
        .wrapping_sub(1 as usize);
pub const VIML_INTERNAL_CALL: uint64_t = INTERNAL_CALL_MASK;
pub const VALID_VIRTCOL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const VARNUMBER_MAX: ::core::ffi::c_long = INT64_MAX;
pub const VARNUMBER_MIN: ::core::ffi::c_long = INT64_MIN;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL_0,
};
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const EX_NOSPC: ::core::ffi::c_uint = 0x10 as ::core::ffi::c_uint;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const CONTEXT_INIT: Context = Context {
    regs: STRING_INIT,
    jumps: STRING_INIT,
    bufs: STRING_INIT,
    gvars: STRING_INIT,
    funcs: ARRAY_DICT_INIT,
};
static e_invalid_submatch_number_nr: GlobalCell<[::core::ffi::c_char; 34]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
            *b"E935: Invalid submatch number: %d\0",
        )
    });
static e_string_list_or_blob_required: GlobalCell<[::core::ffi::c_char; 37]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
            *b"E1098: String, List or Blob required\0",
        )
    });
static e_missing_function_argument: GlobalCell<[::core::ffi::c_char; 33]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 33], [::core::ffi::c_char; 33]>(
            *b"E1132: Missing function argument\0",
        )
    });
static dummy_ap: GlobalCell<::core::ffi::VaList<'static>> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 24], ::core::ffi::VaList<'static>>([0u8; 24])
});
pub unsafe extern "C" fn get_function_name(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    static intidx: GlobalCell<::core::ffi::c_int> = GlobalCell::new(-1 as ::core::ffi::c_int);
    if idx == 0 as ::core::ffi::c_int {
        intidx.set(-1 as ::core::ffi::c_int);
    }
    if intidx.get() < 0 as ::core::ffi::c_int {
        let mut name: *mut ::core::ffi::c_char = get_user_func_name(xp, idx);
        if !name.is_null() {
            if *name as ::core::ffi::c_int != NUL
                && *name as ::core::ffi::c_int != '<' as ::core::ffi::c_int
                && strncmp(
                    b"g:\0".as_ptr() as *const ::core::ffi::c_char,
                    (*xp).xp_pattern,
                    2 as size_t,
                ) == 0 as ::core::ffi::c_int
            {
                return cat_prefix_varname('g' as ::core::ffi::c_int, name);
            }
            return name;
        }
    }
    (*intidx.ptr()) += 1;
    let key: *const ::core::ffi::c_char = (*BUILTINS.ptr())[intidx.get() as usize].name;
    if key.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let key_len: size_t = strlen(key);
    memcpy(
        IObuff.ptr() as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        key as *const ::core::ffi::c_void,
        key_len,
    );
    (*IObuff.ptr())[key_len as usize] = '(' as ::core::ffi::c_char;
    if (*BUILTINS.ptr())[intidx.get() as usize].max_argc as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*IObuff.ptr())[key_len.wrapping_add(1 as size_t) as usize] = ')' as ::core::ffi::c_char;
        (*IObuff.ptr())[key_len.wrapping_add(2 as size_t) as usize] = NUL as ::core::ffi::c_char;
    } else {
        (*IObuff.ptr())[key_len.wrapping_add(1 as size_t) as usize] = NUL as ::core::ffi::c_char;
    }
    return IObuff.ptr() as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn get_expr_name(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    static intidx: GlobalCell<::core::ffi::c_int> = GlobalCell::new(-1 as ::core::ffi::c_int);
    if idx == 0 as ::core::ffi::c_int {
        intidx.set(-1 as ::core::ffi::c_int);
    }
    if intidx.get() < 0 as ::core::ffi::c_int {
        let mut name: *mut ::core::ffi::c_char = get_function_name(xp, idx);
        if !name.is_null() {
            return name;
        }
    }
    (*intidx.ptr()) += 1;
    return get_user_var_name(xp, intidx.get());
}
/// The table row for the builtin `name` spells, or null if there is none.
pub unsafe extern "C" fn find_internal_func(
    name: *const ::core::ffi::c_char,
) -> *const EvalFuncDef {
    let len = strlen(name);
    // SAFETY: `name` is a NUL-terminated string, so its first `len` bytes are
    // readable. `from_raw_parts` refuses a null pointer even for an empty
    // slice, and an empty name is not a builtin anyway.
    let key = if len == 0 {
        &[][..]
    } else {
        ::core::slice::from_raw_parts(name.cast::<u8>(), len)
    };
    match builtin_index(key) {
        Some(row) => BUILTINS.ptr().cast::<EvalFuncDef>().add(row),
        None => ::core::ptr::null::<EvalFuncDef>(),
    }
}
pub unsafe extern "C" fn check_internal_func(
    fdef: *const EvalFuncDef,
    argcount: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut res: ::core::ffi::c_int = 0;
    if argcount < (*fdef).min_argc as ::core::ffi::c_int {
        res = FCERR_TOOFEW as ::core::ffi::c_int;
    } else if argcount > (*fdef).max_argc as ::core::ffi::c_int {
        res = FCERR_TOOMANY as ::core::ffi::c_int;
    } else {
        return (*fdef).base_arg as ::core::ffi::c_int;
    }
    let name: *const ::core::ffi::c_char = (*fdef).name;
    if res == FCERR_TOOMANY as ::core::ffi::c_int {
        semsg(
            gettext(&raw const e_toomanyarg as *const ::core::ffi::c_char),
            name,
        );
    } else {
        semsg(
            gettext(&raw const e_toofewarg as *const ::core::ffi::c_char),
            name,
        );
    }
    return -1 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn call_internal_func(
    fname: *const ::core::ffi::c_char,
    argcount: ::core::ffi::c_int,
    argvars: *mut typval_T,
    rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    let fdef: *const EvalFuncDef = find_internal_func(fname);
    if fdef.is_null() {
        return FCERR_UNKNOWN as ::core::ffi::c_int;
    } else if argcount < (*fdef).min_argc as ::core::ffi::c_int {
        return FCERR_TOOFEW as ::core::ffi::c_int;
    } else if argcount > (*fdef).max_argc as ::core::ffi::c_int {
        return FCERR_TOOMANY as ::core::ffi::c_int;
    }
    (*argvars.offset(argcount as isize)).v_type = VAR_UNKNOWN;
    (*fdef).func.expect("non-null function pointer")(argvars, rettv, (*fdef).data);
    return FCERR_NONE as ::core::ffi::c_int;
}
pub unsafe extern "C" fn call_internal_method(
    fname: *const ::core::ffi::c_char,
    argcount: ::core::ffi::c_int,
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    basetv: *mut typval_T,
) -> ::core::ffi::c_int {
    let fdef: *const EvalFuncDef = find_internal_func(fname);
    if fdef.is_null() {
        return FCERR_UNKNOWN as ::core::ffi::c_int;
    } else if (*fdef).base_arg as ::core::ffi::c_int == BASE_NONE as ::core::ffi::c_int {
        return FCERR_NOTMETHOD as ::core::ffi::c_int;
    } else if (argcount + 1 as ::core::ffi::c_int) < (*fdef).min_argc as ::core::ffi::c_int {
        return FCERR_TOOFEW as ::core::ffi::c_int;
    } else if argcount + 1 as ::core::ffi::c_int > (*fdef).max_argc as ::core::ffi::c_int {
        return FCERR_TOOMANY as ::core::ffi::c_int;
    }
    let mut argv: [typval_T; 21] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 21];
    let base_index: ptrdiff_t =
        (if (*fdef).base_arg as ::core::ffi::c_int == BASE_LAST as ::core::ffi::c_int {
            argcount
        } else {
            (*fdef).base_arg as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        }) as ptrdiff_t;
    if (argcount as ptrdiff_t) < base_index {
        return FCERR_TOOFEW as ::core::ffi::c_int;
    }
    memcpy(
        &raw mut argv as *mut typval_T as *mut ::core::ffi::c_void,
        argvars as *const ::core::ffi::c_void,
        (base_index as size_t).wrapping_mul(::core::mem::size_of::<typval_T>()),
    );
    argv[base_index as usize] = *basetv;
    memcpy(
        (&raw mut argv as *mut typval_T)
            .offset(base_index as isize)
            .offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
        argvars.offset(base_index as isize) as *const ::core::ffi::c_void,
        ((argcount as ptrdiff_t - base_index) as size_t)
            .wrapping_mul(::core::mem::size_of::<typval_T>()),
    );
    argv[(argcount + 1 as ::core::ffi::c_int) as usize].v_type = VAR_UNKNOWN;
    (*fdef).func.expect("non-null function pointer")(
        &raw mut argv as *mut typval_T,
        rettv,
        (*fdef).data,
    );
    return FCERR_NONE as ::core::ffi::c_int;
}
unsafe extern "C" fn non_zero_arg(mut argvars: *mut typval_T) -> bool {
    return (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_number
            != 0 as varnumber_T
        || (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_BOOL as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_bool as ::core::ffi::c_uint
                == kBoolVarTrue as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            && !(*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_string
                .is_null()
            && *(*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_string as ::core::ffi::c_int
                != NUL;
}
pub unsafe extern "C" fn float_op_wrapper(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    let mut f: float_T = 0.;
    (*rettv).v_type = VAR_FLOAT;
    if tv_get_float_chk(argvars, &raw mut f) {
        (*rettv).vval.v_float = fptr.float_func.expect("non-null function pointer")(f);
    } else {
        (*rettv).vval.v_float = 0.0f64 as float_T;
    };
}
pub unsafe extern "C" fn api_wrapper(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    if check_secure() {
        return;
    }
    let mut handler: MsgpackRpcRequestHandler = *fptr.api_handler;
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 20] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_16 { boolean: false },
    }; 20];
    args.capacity = MAX_FUNC_ARGS as ::core::ffi::c_int as size_t;
    args.items = &raw mut args__items as *mut Object;
    let mut arena: Arena = ARENA_EMPTY;
    let mut tv: *mut typval_T = argvars;
    while (*tv).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let c2rust_fresh0 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh0 as isize) = vim_to_object(tv, &raw mut arena, false);
        tv = tv.offset(1);
    }
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut result: Object = handler.fn_0.expect("non-null function pointer")(
        VIML_INTERNAL_CALL,
        args,
        &raw mut arena,
        &raw mut err,
    );
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        semsg_multiline(
            b"emsg\0".as_ptr() as *const ::core::ffi::c_char,
            &raw const e_api_error as *const ::core::ffi::c_char,
            err.msg,
        );
    } else {
        object_to_vim_take_luaref(&raw mut result, rettv, true_0 != 0, &raw mut err);
    }
    if handler.ret_alloc {
        api_free_object(result);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    api_clear_error(&raw mut err);
}
pub unsafe extern "C" fn tv_get_buf(
    mut tv: *mut typval_T,
    mut curtab_only: ::core::ffi::c_int,
) -> *mut buf_T {
    if (*tv).v_type as ::core::ffi::c_uint
        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return buflist_findnr((*tv).vval.v_number as ::core::ffi::c_int);
    }
    if (*tv).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<buf_T>();
    }
    let mut name: *mut ::core::ffi::c_char = (*tv).vval.v_string;
    if name.is_null() || *name as ::core::ffi::c_int == NUL {
        return curbuf.get();
    }
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '$' as ::core::ffi::c_int
        && *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        return lastbuf.get();
    }
    let mut save_magic: ::core::ffi::c_int = p_magic.get();
    p_magic.set(true_0);
    let mut save_cpo: *mut ::core::ffi::c_char = p_cpo.get();
    p_cpo.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
    let mut buf: *mut buf_T = buflist_findnr(buflist_findpat(
        name,
        name.offset(strlen(name) as isize),
        true_0 != 0,
        false_0 != 0,
        curtab_only != 0,
    ));
    p_magic.set(save_magic);
    p_cpo.set(save_cpo);
    if buf.is_null() {
        buf = find_buffer(tv);
    }
    return buf;
}
pub unsafe extern "C" fn tv_get_buf_from_arg(tv: *mut typval_T) -> *mut buf_T {
    if !tv_check_str_or_nr(tv) {
        return ::core::ptr::null_mut::<buf_T>();
    }
    (*emsg_off.ptr()) += 1;
    let buf: *mut buf_T = tv_get_buf(tv, false_0);
    (*emsg_off.ptr()) -= 1;
    return buf;
}
pub unsafe extern "C" fn get_buf_arg(mut arg: *mut typval_T) -> *mut buf_T {
    (*emsg_off.ptr()) += 1;
    let mut buf: *mut buf_T = tv_get_buf(arg, false_0);
    (*emsg_off.ptr()) -= 1;
    if buf.is_null() {
        semsg(
            gettext(b"E158: Invalid buffer name: %s\0".as_ptr() as *const ::core::ffi::c_char),
            tv_get_string(arg),
        );
    }
    return buf;
}
pub unsafe extern "C" fn get_optional_window(
    mut argvars: *mut typval_T,
    mut idx: ::core::ffi::c_int,
) -> *mut win_T {
    if (*argvars.offset(idx as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return curwin.get();
    }
    let mut win: *mut win_T = find_win_by_nr_or_id(argvars.offset(idx as isize));
    if win.is_null() {
        emsg(gettext(
            &raw const e_invalwindow as *const ::core::ffi::c_char,
        ));
        return ::core::ptr::null_mut::<win_T>();
    }
    return win;
}
#[inline]
unsafe extern "C" fn tv_get_float_chk(tv: *const typval_T, ret_f: *mut float_T) -> bool {
    if (*tv).v_type as ::core::ffi::c_uint == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        *ret_f = (*tv).vval.v_float;
        return true_0 != 0;
    }
    if (*tv).v_type as ::core::ffi::c_uint
        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        *ret_f = (*tv).vval.v_number as float_T;
        return true_0 != 0;
    }
    semsg(
        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
        gettext(b"E808: Number or Float required\0".as_ptr() as *const ::core::ffi::c_char),
    );
    return false_0 != 0;
}
pub const TV_TRANSLATE: ::core::ffi::c_ulong = SIZE_MAX;
pub const FNE_CHECK_START: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const AUTOLOAD_CHAR: ::core::ffi::c_int = '#' as ::core::ffi::c_int;
pub const DBL_EPSILON: ::core::ffi::c_double = __DBL_EPSILON__;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const SIGINT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const ENV_SEPCHAR: ::core::ffi::c_int = ':' as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RE_STRING: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
pub const __DBL_EPSILON__: ::core::ffi::c_double = 2.2204460492503131e-16f64;
