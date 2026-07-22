use crate::src::mpack::object::mpack_parser_init;
use crate::src::nvim::api::private::converter::{
    object_to_vim, object_to_vim_take_luaref, vim_to_object,
};
use crate::src::nvim::api::private::dispatch::method_handlers;
use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_free_object, api_free_string, api_metadata, arena_array, cbuf_to_string,
    cstr_as_string, dict_set_var,
};
use crate::src::nvim::api::vim::nvim_feedkeys;
use crate::src::nvim::arglist::{f_argc, f_argidx, f_arglistid, f_argv};
use crate::src::nvim::autocmd::{apply_autocmds, au_exists, autocmd_supported};
use crate::src::nvim::buffer::{
    bt_prompt, buf_close_terminal, buflist_findnr, buflist_findpat, setfname,
};
use crate::src::nvim::channel::{
    channel_close, channel_connect, channel_create_event, channel_decref, channel_from_stdio,
    channel_incref, channel_job_start, channel_send, channel_terminal_alloc,
};
use crate::src::nvim::charset::{getdigits_int, skipwhite};
use crate::src::nvim::cmdexpand::{
    cmdline_pum_active, f_cmdcomplete_info, f_getcompletion, f_getcompletiontype, ExpandCleanup,
    ExpandInit, ExpandOne,
};
use crate::src::nvim::cmdhist::{f_histadd, f_histdel, f_histget, f_histnr};
use crate::src::nvim::context::{
    ctx_free, ctx_from_dict, ctx_get, ctx_restore, ctx_save, ctx_size, ctx_to_dict, kCtxAll,
};
use crate::src::nvim::cursor::{check_cursor, get_cursor_pos_ptr};
use crate::src::nvim::diff::{f_diff_filler, f_diff_hlID};
use crate::src::nvim::digraph::{
    f_digraph_get, f_digraph_getlist, f_digraph_set, f_digraph_setlist,
};
use crate::src::nvim::edit::buf_prompt_text;
use crate::src::nvim::eval::buffer::{
    f_append, f_appendbufline, f_bufadd, f_bufexists, f_buflisted, f_bufload, f_bufloaded,
    f_bufname, f_bufnr, f_bufwinid, f_bufwinnr, f_deletebufline, f_getbufinfo, f_getbufline,
    f_getbufoneline, f_getline, f_prompt_appendbuf, f_prompt_setcallback, f_prompt_setinterrupt,
    f_prompt_setprompt, f_setbufline, f_setline, find_buffer,
};
use crate::src::nvim::eval::decode::{
    json_decode_string, mpack_parse_typval, typval_parser_error_free, unpack_typval,
};
use crate::src::nvim::eval::deprecated::{f_last_buffer_nr, f_rpcstart, f_rpcstop, f_termopen};
use crate::src::nvim::eval::encode::{
    encode_init_lrstate, encode_list_write, encode_read_from_list, encode_tv2echo, encode_tv2json,
    encode_vim_list_to_buf, encode_vim_to_msgpack,
};
use crate::src::nvim::eval::fs::{
    f_browse, f_browsedir, f_chdir, f_delete, f_executable, f_exepath, f_filecopy, f_filereadable,
    f_filewritable, f_finddir, f_findfile, f_fnamemodify, f_getcwd, f_getfperm, f_getfsize,
    f_getftime, f_getftype, f_glob, f_glob2regpat, f_globpath, f_haslocaldir, f_isabsolutepath,
    f_isdirectory, f_mkdir, f_pathshorten, f_readblob, f_readdir, f_readfile, f_rename, f_resolve,
    f_simplify, f_tempname, f_writefile,
};
use crate::src::nvim::eval::list::{
    f_add, f_count, f_extend, f_extendnew, f_filter, f_foreach, f_insert, f_map, f_mapnew,
    f_remove, f_reverse,
};
use crate::src::nvim::eval::typval::{
    callback_free, f_blob2list, f_has_key, f_items, f_join, f_keys, f_list2blob, f_list2str,
    f_sort, f_uniq, f_values, tv_blob_alloc_ret, tv_blob_set_range, tv_check_for_buffer_arg,
    tv_check_for_dict_arg, tv_check_for_list_arg, tv_check_for_list_or_blob_arg,
    tv_check_for_lnum_arg, tv_check_for_nonempty_string_arg, tv_check_for_nonnull_dict_arg,
    tv_check_for_number_arg, tv_check_for_opt_bool_arg, tv_check_for_opt_dict_arg,
    tv_check_for_opt_number_arg, tv_check_for_string_arg, tv_check_for_string_or_func_arg,
    tv_check_for_string_or_list_arg, tv_check_num, tv_check_str_or_nr, tv_clear, tv_copy,
    tv_dict_add_allocated_str, tv_dict_add_bool, tv_dict_add_list, tv_dict_add_nr, tv_dict_add_str,
    tv_dict_add_str_len, tv_dict_alloc, tv_dict_alloc_ret, tv_dict_extend, tv_dict_find,
    tv_dict_free, tv_dict_get_bool, tv_dict_get_callback, tv_dict_get_number,
    tv_dict_get_number_def, tv_dict_get_string, tv_dict_item_remove, tv_dict_watcher_add,
    tv_dict_watcher_remove, tv_equal, tv_get_bool, tv_get_bool_chk, tv_get_lnum, tv_get_lnum_buf,
    tv_get_number, tv_get_number_chk, tv_get_string, tv_get_string_buf, tv_get_string_buf_chk,
    tv_get_string_chk, tv_islocked, tv_list_alloc, tv_list_alloc_ret,
    tv_list_append_allocated_string, tv_list_append_dict, tv_list_append_list,
    tv_list_append_number, tv_list_append_owned_tv, tv_list_append_string, tv_list_append_tv,
    tv_list_copy, tv_list_extend, tv_list_find, tv_list_find_nr, tv_list_flatten,
    tv_list_item_remove, tv_list_unref, value_check_lock,
};
use crate::src::nvim::eval::userfunc::{
    emsg_funcname, find_func, func_call, func_ptr_ref, func_ref, func_unref, function_exists,
    get_func_arity, get_scriptlocal_funcname, get_user_func_name, printable_func_name,
    restore_funccal, save_funccal, save_function_name, set_current_funccal, trans_function_name,
    translated_function_exists,
};
use crate::src::nvim::eval::vars::{
    cat_prefix_varname, f_getbufvar, f_gettabvar, f_gettabwinvar, f_getwinvar, f_setbufvar,
    f_settabvar, f_settabwinvar, f_setwinvar, find_var, get_user_var_name, get_vim_var_nr,
    get_vim_var_str, get_vim_var_tv, prepare_vimvar, restore_vimvar, set_vim_var_nr,
    set_vim_var_type, var_exists,
};
use crate::src::nvim::eval::window::{
    f_getcmdwintype, f_gettabinfo, f_getwininfo, f_getwinpos, f_getwinposx, f_getwinposy,
    f_tabpagenr, f_tabpagewinnr, f_win_execute, f_win_findbuf, f_win_getid, f_win_gettype,
    f_win_gotoid, f_win_id2tabwin, f_win_id2win, f_win_move_separator, f_win_move_statusline,
    f_win_screenpos, f_win_splitmove, f_winbufnr, f_wincol, f_winheight, f_winlayout, f_winline,
    f_winnr, f_winrestcmd, f_winrestview, f_winsaveview, f_winwidth, find_tabwin,
    find_win_by_nr_or_id, win_id2wp_tp,
};
use crate::src::nvim::eval_1::{
    add_timer_info, add_timer_info_all, buf_byteidx_to_charidx, buf_charidx_to_byteidx,
    callback_from_typval, clear_lval, common_job_callbacks, do_string_sub, eval1,
    eval_expr_to_bool, eval_expr_typval, eval_expr_valid_arg, eval_has_provider, eval_option,
    f_slice, f_system, f_systemlist, find_job, find_timer_by_nr, get_callback_depth, get_copyID,
    get_lval, list2fpos, partial_name, prompt_get_input, save_tv_as_string, script_host_eval,
    string2float, timer_due_cb, timer_start, timer_stop, timer_stop_all, tv_to_argv, var2fpos,
    var_item_copy,
};
use crate::src::nvim::event::libuv::{uv_kill, uv_strerror};
use crate::src::nvim::event::multiqueue::{
    multiqueue_empty, multiqueue_free, multiqueue_new, multiqueue_process_events,
    multiqueue_replace_parent,
};
use crate::src::nvim::event::proc::{proc_stop, proc_wait};
use crate::src::nvim::event::r#loop::{loop_on_put, loop_poll_events};
use crate::src::nvim::event::time::{
    time_watcher_close, time_watcher_init, time_watcher_start, time_watcher_stop,
};
use crate::src::nvim::ex_cmds::check_secure;
use crate::src::nvim::ex_docmd::{
    cmd_exists, do_cmdline, do_cmdline_cmd, eval_vars, expand_filename, f_fullcommand,
};
use crate::src::nvim::ex_eval::aborting;
use crate::src::nvim::ex_getln::{
    f_getcmdcomplpat, f_getcmdcompltype, f_getcmdline, f_getcmdpos, f_getcmdprompt,
    f_getcmdscreenpos, f_getcmdtype, f_setcmdline, f_setcmdpos, f_wildtrigger, get_user_input,
    text_locked, text_locked_msg, vim_strsave_fnameescape,
};
use crate::src::nvim::fold::{
    f_foldclosed, f_foldclosedend, f_foldlevel, f_foldtext, f_foldtextresult,
};
use crate::src::nvim::fuzzy::{f_matchfuzzy, f_matchfuzzypos};
use crate::src::nvim::getchar::{
    f_getchar, f_getcharmod, f_getcharstr, restore_typeahead, save_typeahead, stuff_empty,
    using_script, vgetc,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::{grid_getchar, schar_from_char, schar_get, schar_get_first_codepoint};
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::highlight_group::{
    get_highlight_name_ext, highlight_color, highlight_exists, highlight_has_attr,
    syn_get_final_id, syn_name2id,
};
use crate::src::nvim::indent::{f_indent, f_lispindent, get_sw_value, get_sw_value_col};
use crate::src::nvim::indent_c::f_cindent;
use crate::src::nvim::input::prompt_for_input;
use crate::src::nvim::insexpand::{
    f_complete, f_complete_add, f_complete_check, f_complete_info, f_preinserted, ins_compl_active,
};
use crate::src::nvim::keycodes::vim_strsave_escape_ks;
use crate::src::nvim::log::logmsg;
use crate::src::nvim::lua::executor::{
    nlua_exec, nlua_func_exists, nlua_is_table_from_lua, nlua_register_table_as_callable,
    nlua_typval_eval,
};
use crate::src::nvim::main::{
    autocmd_bufnr, autocmd_busy, autocmd_fname, autocmd_fname_full, autocmd_match, called_emsg,
    capture_ga, channels, cmdline_row, cmdline_star, curbuf, current_sctx, curtab, curwin,
    did_emsg, e_api_error, e_buffer_is_not_loaded, e_cannot_change_readonly_variable_str,
    e_channotpty, e_dictkey, e_invalid_buffer_name_str, e_invalid_column_number_nr,
    e_invalid_line_number_nr, e_invalwindow, e_invarg, e_invarg2, e_invargNval, e_invargval,
    e_invexpr2, e_libcall, e_listarg, e_listblobarg, e_listblobreq, e_listdictarg,
    e_listdictblobarg, e_no_spell, e_number_exp, e_reduce_of_an_empty_str_with_no_initial_value,
    e_stdiochan2, e_toofewarg, e_toomanyarg, e_trailing_arg, e_unknown_function_str,
    empty_string_option, emsg_noredir, emsg_off, emsg_silent, firstwin, garbage_collect_at_exit,
    got_int, lastbuf, lines_left, main_loop, mouse_row, msg_col, msg_row, msg_scroll, msg_scrolled,
    msg_silent, need_clr_eos, on_print, p_cpo, p_enc, p_ic, p_magic, p_sel, p_spk, p_tgc,
    p_verbose, p_wic, p_ws, provider_call_nesting, provider_caller_scope, redir_off, reg_executing,
    reg_recorded, reg_recording, skip_update_topline, starting, stdin_isatty, stdout_isatty,
    typebuf, vgetc_busy, vim_ignored, virtual_op, want_garbage_collect, wild_menu_showing,
    windowsVersion, IObuff, NameBuff, Rows, State, EVALARG_EVALUATE,
};
use crate::src::nvim::map::mh_get_uint64_t;
use crate::src::nvim::mapping::{f_hasmapto, f_maparg, f_mapcheck, f_maplist, f_mapset};
use crate::src::nvim::mark::{
    cleanup_jumplist, get_buf_local_marks, get_global_marks, setmark_pos, setpcmark,
};
use crate::src::nvim::mbyte::{
    convert_setup, enc_locale, f_charclass, f_getcellwidths, f_iconv, f_setcellwidths,
    mb_adjust_cursor, mb_prevptr, string_convert, utf_char2bytes, utf_ptr2char, utf_ptr2len,
    utfc_ptr2len,
};
use crate::src::nvim::memline::{
    decl, incl, ml_find_line_or_offset, ml_get, ml_get_buf, ml_get_buf_len, ml_get_len, ml_get_pos,
    ml_open, recover_names, swapfile_dict,
};
use crate::src::nvim::memory::{
    alloc_block, arena_mem_free, free_block, strequal, strnequal, xcalloc, xfree, xmalloc,
    xmallocz, xmemdup, xmemdupz, xstrdup,
};
use crate::src::nvim::menu::{f_menu_info, get_menu_cmd_modes, menu_get};
use crate::src::nvim::message::{
    do_dialog, emsg, internal_error, msg_clr_eos, msg_ext_set_kind, msg_putchar, msg_puts,
    msg_scroll_flush, msg_start, semsg, semsg_multiline, str2special_save, verb_msg,
};
use crate::src::nvim::mouse::f_getmousepos;
use crate::src::nvim::msgpack_rpc::channel::{rpc_send_call, rpc_send_event};
use crate::src::nvim::msgpack_rpc::packer::{packer_string_buffer, packer_take_string};
use crate::src::nvim::msgpack_rpc::server::{
    server_address_list, server_address_new, server_start, server_stop,
};
use crate::src::nvim::normal::{find_decl, op_pending, unadjust_for_sel_inner};
use crate::src::nvim::ops::{
    block_prep, charwise_block_prep, cursor_pos_info, reset_lbr, restore_lbr,
};
use crate::src::nvim::option::set_option_value_give_err;
use crate::src::nvim::optionstr::free_string_option;
use crate::src::nvim::os::dl::os_libcall;
use crate::src::nvim::os::env::{
    expand_env_save, home_replace, os_copy_fullenv, os_env_exists, os_free_fullenv,
    os_get_fullenv_size, os_get_hostname, os_get_pid, os_getenv, vim_env_iter, vim_getenv,
    vim_setenv_ext, vim_unsetenv_ext,
};
use crate::src::nvim::os::fs::{os_isdir, os_setperm};
use crate::src::nvim::os::libc::{
    __assert_fail, abort, acos, asin, atan, atan2, atoi, ceil, cos, cosh, exp, fabs, floor, fmod,
    gettext, log, log10, memcmp, memcpy, memmove, memset, mktime, pow, round, sin, sinh, snprintf,
    sqrt, strcasecmp, strchr, strcmp, strcpy, strftime, strlen, strncasecmp, strncmp, strtoul, tan,
    tanh, time, trunc,
};
use crate::src::nvim::os::pty_proc_unix::pty_proc_resize;
use crate::src::nvim::os::shell::shell_free_argv;
use crate::src::nvim::os::stdpaths::{get_appname, get_xdg_home, stdpaths_get_xdg_var};
use crate::src::nvim::os::time::{os_hrtime, os_localtime_r, os_strptime};
use crate::src::nvim::path::{concat_fnames_realloc, vim_FullName};
use crate::src::nvim::plines::{getvvcol, win_chartabsize};
use crate::src::nvim::popupmenu::{pum_set_event_info, pum_visible};
use crate::src::nvim::profile::{
    profile_end, profile_msg, profile_setlimit, profile_signed, profile_start, profile_sub,
};
use crate::src::nvim::quickfix::{f_getloclist, f_getqflist, f_setloclist, f_setqflist};
use crate::src::nvim::r#match::{
    f_clearmatches, f_getmatches, f_matchadd, f_matchaddpos, f_matcharg, f_matchdelete,
    f_setmatches,
};
use crate::src::nvim::r#move::{f_screenpos, f_virtcol2col, update_curswant, win_col_off};
use crate::src::nvim::regexp::{reg_submatch, reg_submatch_list};
use crate::src::nvim::register::{
    format_reg_type, get_reg_contents, get_reg_type, get_unname_register, get_yank_register,
    op_reg_set_previous, write_reg_contents_ex, write_reg_contents_lst,
};
use crate::src::nvim::runtime::{exestack, f_getscriptinfo, f_getstacktrace};
use crate::src::nvim::search::{
    f_searchcount, last_csearch, last_csearch_forward, last_csearch_until, searchit,
    set_csearch_direction, set_csearch_until, set_last_csearch,
};
use crate::src::nvim::sha256::sha256_bytes;
use crate::src::nvim::sign::{
    f_sign_define, f_sign_getdefined, f_sign_getplaced, f_sign_jump, f_sign_place,
    f_sign_placelist, f_sign_undefine, f_sign_unplace, f_sign_unplacelist,
};
use crate::src::nvim::spell::{eval_soundfold, parse_spelllang, spell_check, spell_move_to};
use crate::src::nvim::spellsuggest::spell_suggest_list;
use crate::src::nvim::state::{get_mode, get_was_safe_state, virtual_active};
use crate::src::nvim::strings::{
    f_byteidx, f_byteidxcomp, f_charidx, f_str2list, f_str2nr, f_strcharlen, f_strcharpart,
    f_strchars, f_strdisplaywidth, f_strgetchar, f_stridx, f_string, f_strlen, f_strpart,
    f_strridx, f_strtrans, f_strutf16len, f_strwidth, f_tolower, f_toupper, f_tr, f_trim,
    f_utf16idx, vim_snprintf, vim_strchr, vim_strsave_escaped, vim_strsave_shellescape,
    vim_vsnprintf_typval,
};
use crate::src::nvim::syntax::{
    get_syntax_info, syn_get_id, syn_get_stack_item, syn_get_sub_char, syntax_present,
};
use crate::src::nvim::tag::{get_tagfname, get_tags, get_tagstack, set_tagstack, tagname_free};
use crate::src::nvim::testing::{
    f_assert_beeps, f_assert_equal, f_assert_equalfile, f_assert_exception, f_assert_fails,
    f_assert_false, f_assert_inrange, f_assert_match, f_assert_nobeep, f_assert_notequal,
    f_assert_notmatch, f_assert_report, f_assert_true, f_test_garbagecollect_now,
    f_test_write_list_log,
};
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, ApiDispatchWrapper, Arena, ArenaMem, Array, AutoPat, AutoPatCmd,
    AutoPatCmd_S, BoolVarValue, Boolean, BufUpdateCallbacks, Buffer, CMD_index, Callback,
    CallbackReader, CallbackType, Callback_data as C2Rust_Unnamed_22, ChangedtickDictItem, Channel,
    ChannelCallFrame, ChannelPart, ChannelStdinMode, ChannelStreamType,
    Channel_stream as C2Rust_Unnamed_42, ClientType, Context, DecorExt, DecorHighlightInline,
    DecorInlineData, DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_19, Dict,
    Direction, Error, ErrorType, EvalFuncData, EvalFuncDef, ExtmarkUndoObject, FileID, Float,
    FloatAnchor, FloatRelative, GRegFlags, GridView, Integer, InternalState, Intersection,
    KeyValuePair, LibuvProc, LineGetter, ListLenSpecials, ListReaderState, Loop, LuaRef,
    LuaRetMode, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t,
    Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, MotionType, MsgpackRpcRequestHandler,
    MultiQueue, Object, ObjectType, OptIndex, OptInt, OptVal, OptValData, OptValType, PackerBuffer,
    PackerBufferFlush, Proc, ProcType, PtyProc, PutCallback, RStream, RemoteUI, RpcState,
    RpcState_call_stack as C2Rust_Unnamed_41, ScopeDictDictItem, ScopeType, ScreenGrid,
    Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue, StderrState, StdioPair,
    StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_29, Stream, String_0, Terminal,
    TimeWatcher, Timestamp, TriState, UIExtension, Unpacker, VarLockStatus, VarType, VimLFunc,
    VimVarIndex, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo, WinSplit,
    WinStyle, Window, XDGVarType, __builtin_va_list, __gid_t, __gnuc_va_list,
    __pthread_internal_list, __pthread_list_t, __pthread_mutex_s, __pthread_rwlock_arch_t,
    __time_t, __uid_t, __va_list_tag, alist_T, auto_event, bhdr_T, blob_T, blobvar_S, block_def,
    blocknr_T, buf_T, buffblock, buffblock_T, buffheader_T, bufstate_T, caller_scope, chunksize_T,
    cmd_addr_T, cmdidx_T, colnr_T, consumed_blk, cstack_T, cstack_T_cs_pend as C2Rust_Unnamed_40,
    dict_T, dictitem_T, dictvar_S, diff_T, diffblock_S, disptick_T, eslist_T, eslist_elem,
    estack_T, estack_T_es_info as C2Rust_Unnamed_49, etype_T, evalarg_T, event_T, exarg, exarg_T,
    except_T, except_type_T, expand_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_20, file_buffer_b_wininfo as C2Rust_Unnamed_28,
    file_buffer_update_callbacks as C2Rust_Unnamed_17,
    file_buffer_update_channels as C2Rust_Unnamed_18, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccal_entry, funccal_entry_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_23,
    funccall_T, funcdict_T, garray_T, gid_t, handle_T, hash_T, hashitem_T, hashtab_T, hlf_T,
    iconv_t, infoptr_T, int16_t, int32_t, int64_t, internal_proc_cb, key_value_pair, lcs_chars_T,
    linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, loop_0,
    loop_0_children as C2Rust_Unnamed_30, lpos_T, lval_T, mapblock, mapblock_T, match_T, matchitem,
    matchitem_T, memfile_T, memline_T, mfdirty_T, mpack_data_t, mpack_node_s, mpack_node_t,
    mpack_parser_t, mpack_sintmax_t, mpack_tokbuf_s, mpack_tokbuf_t, mpack_token_s,
    mpack_token_s_data as C2Rust_Unnamed_14, mpack_token_t, mpack_token_type_t, mpack_uint32_t,
    mpack_uintmax_t, mpack_value_s, mpack_value_t, msglist, msglist_T, mtnode_inner_s, mtnode_s,
    multiqueue, object, object_data as C2Rust_Unnamed_16, oparg_T, packer_buffer_t, partial_S,
    partial_T, pos_T, pos_save_T, proc, proc_exit_cb, proc_state_cb, proftime_T, pthread_mutex_t,
    pthread_rwlock_t, ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue, reg_extmatch_T, regmatch_T,
    regmmatch_T, regprog, regprog_T, rstream, sattr_T, schar_T, scid_T, sctx_T, searchit_arg_T,
    size_t, smt_T, ssize_t, stream, stream_close_cb, stream_read_cb,
    stream_uv as C2Rust_Unnamed_31, stream_write_cb, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_21, syn_time_T, synblock_T, synstate_T, tabpage_S,
    tabpage_T, taggy_T, tagname_T, tasave_T, terminal, time_cb, time_t, time_watcher, timer_T, tm,
    typebuf_T, typval_T, typval_vval_union, u_entry, u_entry_T, u_header, u_header_T,
    u_header_uh_alt_next as C2Rust_Unnamed_25, u_header_uh_alt_prev as C2Rust_Unnamed_24,
    u_header_uh_next as C2Rust_Unnamed_27, u_header_uh_prev as C2Rust_Unnamed_26, ufunc_S, ufunc_T,
    uid_t, uint16_t, uint32_t, uint64_t, uint8_t, undo_object, uv__io_cb, uv__io_s, uv__io_t,
    uv__queue, uv__work, uv_alloc_cb, uv_async_cb, uv_async_s, uv_async_s_u as C2Rust_Unnamed_3,
    uv_async_t, uv_buf_t, uv_close_cb, uv_connect_cb, uv_connect_s, uv_connect_t, uv_connection_cb,
    uv_exit_cb, uv_file, uv_gid_t, uv_handle_s, uv_handle_s_u as C2Rust_Unnamed_0, uv_handle_t,
    uv_handle_type, uv_idle_cb, uv_idle_s, uv_idle_s_u as C2Rust_Unnamed_10, uv_idle_t, uv_loop_s,
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
    wininfo_S, winopt_T, winsize, wline_T, xfmark_T, xp_prefix_T, yankreg_T, QUEUE,
};
use crate::src::nvim::ui::{
    ui_busy_start, ui_busy_stop, ui_current_col, ui_current_row, ui_flush, ui_gui_attached, ui_has,
    ui_rgb_attached,
};
use crate::src::nvim::ui_compositor::ui_comp_get_grid_at_coord;
use crate::src::nvim::undo::{f_undofile, f_undotree};
use crate::src::nvim::version::{has_nvim_version, has_vim_patch};
use crate::src::nvim::window::find_tabpage;
extern "C" {
    fn uv_random(
        loop_0: *mut uv_loop_t,
        req: *mut uv_random_t,
        buf: *mut ::core::ffi::c_void,
        buflen: size_t,
        flags: ::core::ffi::c_uint,
        cb: uv_random_cb,
    ) -> ::core::ffi::c_int;
    fn arena_finish(arena: *mut Arena) -> ArenaMem;
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn ga_append(gap: *mut garray_T, c: uint8_t);
    fn ga_append_via_ptr(gap: *mut garray_T, item_size: size_t) -> *mut ::core::ffi::c_void;
    fn get_client_info(
        chan: *mut Channel,
        key: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec_nl(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T)
        -> bool;
    fn terminal_open(termpp: *mut *mut Terminal, buf: *mut buf_T);
    fn terminal_buf(term: *const Terminal) -> Buffer;
    fn terminal_running(term: *const Terminal) -> bool;
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
pub const kOptWritedelay: OptIndex = 373;
pub const kOptWritebackup: OptIndex = 372;
pub const kOptWriteany: OptIndex = 371;
pub const kOptWrite: OptIndex = 370;
pub const kOptWrapscan: OptIndex = 369;
pub const kOptWrapmargin: OptIndex = 368;
pub const kOptWrap: OptIndex = 367;
pub const kOptWinwidth: OptIndex = 366;
pub const kOptWinminwidth: OptIndex = 365;
pub const kOptWinminheight: OptIndex = 364;
pub const kOptWinhighlight: OptIndex = 363;
pub const kOptWinheight: OptIndex = 362;
pub const kOptWinfixwidth: OptIndex = 361;
pub const kOptWinfixheight: OptIndex = 360;
pub const kOptWinfixbuf: OptIndex = 359;
pub const kOptWindow: OptIndex = 358;
pub const kOptWinborder: OptIndex = 357;
pub const kOptWinblend: OptIndex = 356;
pub const kOptWinbar: OptIndex = 355;
pub const kOptWinaltkeys: OptIndex = 354;
pub const kOptWildoptions: OptIndex = 353;
pub const kOptWildmode: OptIndex = 352;
pub const kOptWildmenu: OptIndex = 351;
pub const kOptWildignorecase: OptIndex = 350;
pub const kOptWildignore: OptIndex = 349;
pub const kOptWildcharm: OptIndex = 348;
pub const kOptWildchar: OptIndex = 347;
pub const kOptWhichwrap: OptIndex = 346;
pub const kOptWarn: OptIndex = 345;
pub const kOptVisualbell: OptIndex = 344;
pub const kOptVirtualedit: OptIndex = 343;
pub const kOptViewoptions: OptIndex = 342;
pub const kOptViewdir: OptIndex = 341;
pub const kOptVerbosefile: OptIndex = 340;
pub const kOptVerbose: OptIndex = 339;
pub const kOptVartabstop: OptIndex = 338;
pub const kOptVarsofttabstop: OptIndex = 337;
pub const kOptUpdatetime: OptIndex = 336;
pub const kOptUpdatecount: OptIndex = 335;
pub const kOptUndoreload: OptIndex = 334;
pub const kOptUndolevels: OptIndex = 333;
pub const kOptUndofile: OptIndex = 332;
pub const kOptUndodir: OptIndex = 331;
pub const kOptTtyfast: OptIndex = 330;
pub const kOptTtimeoutlen: OptIndex = 329;
pub const kOptTtimeout: OptIndex = 328;
pub const kOptTitlestring: OptIndex = 327;
pub const kOptTitleold: OptIndex = 326;
pub const kOptTitlelen: OptIndex = 325;
pub const kOptTitle: OptIndex = 324;
pub const kOptTimeoutlen: OptIndex = 323;
pub const kOptTimeout: OptIndex = 322;
pub const kOptTildeop: OptIndex = 321;
pub const kOptThesaurusfunc: OptIndex = 320;
pub const kOptThesaurus: OptIndex = 319;
pub const kOptTextwidth: OptIndex = 318;
pub const kOptTerse: OptIndex = 317;
pub const kOptTermsync: OptIndex = 316;
pub const kOptTermpastefilter: OptIndex = 315;
pub const kOptTermguicolors: OptIndex = 314;
pub const kOptTermencoding: OptIndex = 313;
pub const kOptTermbidi: OptIndex = 312;
pub const kOptTagstack: OptIndex = 311;
pub const kOptTags: OptIndex = 310;
pub const kOptTagrelative: OptIndex = 309;
pub const kOptTaglength: OptIndex = 308;
pub const kOptTagfunc: OptIndex = 307;
pub const kOptTagcase: OptIndex = 306;
pub const kOptTagbsearch: OptIndex = 305;
pub const kOptTabstop: OptIndex = 304;
pub const kOptTabpagemax: OptIndex = 303;
pub const kOptTabline: OptIndex = 302;
pub const kOptTabclose: OptIndex = 301;
pub const kOptSyntax: OptIndex = 300;
pub const kOptSynmaxcol: OptIndex = 299;
pub const kOptSwitchbuf: OptIndex = 298;
pub const kOptSwapfile: OptIndex = 297;
pub const kOptSuffixesadd: OptIndex = 296;
pub const kOptSuffixes: OptIndex = 295;
pub const kOptStatusline: OptIndex = 294;
pub const kOptStatuscolumn: OptIndex = 293;
pub const kOptStartofline: OptIndex = 292;
pub const kOptSplitright: OptIndex = 291;
pub const kOptSplitkeep: OptIndex = 290;
pub const kOptSplitbelow: OptIndex = 289;
pub const kOptSpellsuggest: OptIndex = 288;
pub const kOptSpelloptions: OptIndex = 287;
pub const kOptSpelllang: OptIndex = 286;
pub const kOptSpellfile: OptIndex = 285;
pub const kOptSpellcapcheck: OptIndex = 284;
pub const kOptSpell: OptIndex = 283;
pub const kOptSofttabstop: OptIndex = 282;
pub const kOptSmoothscroll: OptIndex = 281;
pub const kOptSmarttab: OptIndex = 280;
pub const kOptSmartindent: OptIndex = 279;
pub const kOptSmartcase: OptIndex = 278;
pub const kOptSigncolumn: OptIndex = 277;
pub const kOptSidescrolloff: OptIndex = 276;
pub const kOptSidescroll: OptIndex = 275;
pub const kOptShowtabline: OptIndex = 274;
pub const kOptShowmode: OptIndex = 273;
pub const kOptShowmatch: OptIndex = 272;
pub const kOptShowfulltag: OptIndex = 271;
pub const kOptShowcmdloc: OptIndex = 270;
pub const kOptShowcmd: OptIndex = 269;
pub const kOptShowbreak: OptIndex = 268;
pub const kOptShortmess: OptIndex = 267;
pub const kOptShiftwidth: OptIndex = 266;
pub const kOptShiftround: OptIndex = 265;
pub const kOptShellxquote: OptIndex = 264;
pub const kOptShellxescape: OptIndex = 263;
pub const kOptShelltemp: OptIndex = 262;
pub const kOptShellslash: OptIndex = 261;
pub const kOptShellredir: OptIndex = 260;
pub const kOptShellquote: OptIndex = 259;
pub const kOptShellpipe: OptIndex = 258;
pub const kOptShellcmdflag: OptIndex = 257;
pub const kOptShell: OptIndex = 256;
pub const kOptShadafile: OptIndex = 255;
pub const kOptShada: OptIndex = 254;
pub const kOptSessionoptions: OptIndex = 253;
pub const kOptSelectmode: OptIndex = 252;
pub const kOptSelection: OptIndex = 251;
pub const kOptSecure: OptIndex = 250;
pub const kOptSections: OptIndex = 249;
pub const kOptScrollopt: OptIndex = 248;
pub const kOptScrolloff: OptIndex = 247;
pub const kOptScrolljump: OptIndex = 246;
pub const kOptScrollbind: OptIndex = 245;
pub const kOptScrollback: OptIndex = 244;
pub const kOptScroll: OptIndex = 243;
pub const kOptRuntimepath: OptIndex = 242;
pub const kOptRulerformat: OptIndex = 241;
pub const kOptRuler: OptIndex = 240;
pub const kOptRightleftcmd: OptIndex = 239;
pub const kOptRightleft: OptIndex = 238;
pub const kOptRevins: OptIndex = 237;
pub const kOptReport: OptIndex = 236;
pub const kOptRemap: OptIndex = 235;
pub const kOptRelativenumber: OptIndex = 234;
pub const kOptRegexpengine: OptIndex = 233;
pub const kOptRedrawtime: OptIndex = 232;
pub const kOptRedrawdebug: OptIndex = 231;
pub const kOptReadonly: OptIndex = 230;
pub const kOptQuoteescape: OptIndex = 229;
pub const kOptQuickfixtextfunc: OptIndex = 228;
pub const kOptPyxversion: OptIndex = 227;
pub const kOptPumwidth: OptIndex = 226;
pub const kOptPummaxwidth: OptIndex = 225;
pub const kOptPumheight: OptIndex = 224;
pub const kOptPumborder: OptIndex = 223;
pub const kOptPumblend: OptIndex = 222;
pub const kOptPrompt: OptIndex = 221;
pub const kOptPreviewwindow: OptIndex = 220;
pub const kOptPreviewheight: OptIndex = 219;
pub const kOptPreserveindent: OptIndex = 218;
pub const kOptPath: OptIndex = 217;
pub const kOptPatchmode: OptIndex = 216;
pub const kOptPatchexpr: OptIndex = 215;
pub const kOptPastetoggle: OptIndex = 214;
pub const kOptPaste: OptIndex = 213;
pub const kOptParagraphs: OptIndex = 212;
pub const kOptPackpath: OptIndex = 211;
pub const kOptOperatorfunc: OptIndex = 210;
pub const kOptOpendevice: OptIndex = 209;
pub const kOptOmnifunc: OptIndex = 208;
pub const kOptNumberwidth: OptIndex = 207;
pub const kOptNumber: OptIndex = 206;
pub const kOptNrformats: OptIndex = 205;
pub const kOptMousetime: OptIndex = 204;
pub const kOptMouseshape: OptIndex = 203;
pub const kOptMousescroll: OptIndex = 202;
pub const kOptMousemoveevent: OptIndex = 201;
pub const kOptMousemodel: OptIndex = 200;
pub const kOptMousehide: OptIndex = 199;
pub const kOptMousefocus: OptIndex = 198;
pub const kOptMouse: OptIndex = 197;
pub const kOptMore: OptIndex = 196;
pub const kOptModified: OptIndex = 195;
pub const kOptModifiable: OptIndex = 194;
pub const kOptModelines: OptIndex = 193;
pub const kOptModelineexpr: OptIndex = 192;
pub const kOptModeline: OptIndex = 191;
pub const kOptMkspellmem: OptIndex = 190;
pub const kOptMessagesopt: OptIndex = 189;
pub const kOptMenuitems: OptIndex = 188;
pub const kOptMaxsearchcount: OptIndex = 187;
pub const kOptMaxmempattern: OptIndex = 186;
pub const kOptMaxmapdepth: OptIndex = 185;
pub const kOptMaxfuncdepth: OptIndex = 184;
pub const kOptMaxcombine: OptIndex = 183;
pub const kOptMatchtime: OptIndex = 182;
pub const kOptMatchpairs: OptIndex = 181;
pub const kOptMakeprg: OptIndex = 180;
pub const kOptMakeencoding: OptIndex = 179;
pub const kOptMakeef: OptIndex = 178;
pub const kOptMagic: OptIndex = 177;
pub const kOptLoadplugins: OptIndex = 176;
pub const kOptListchars: OptIndex = 175;
pub const kOptList: OptIndex = 174;
pub const kOptLispwords: OptIndex = 173;
pub const kOptLispoptions: OptIndex = 172;
pub const kOptLisp: OptIndex = 171;
pub const kOptLinespace: OptIndex = 170;
pub const kOptLines: OptIndex = 169;
pub const kOptLinebreak: OptIndex = 168;
pub const kOptLhistory: OptIndex = 167;
pub const kOptLazyredraw: OptIndex = 166;
pub const kOptLaststatus: OptIndex = 165;
pub const kOptLangremap: OptIndex = 164;
pub const kOptLangnoremap: OptIndex = 163;
pub const kOptLangmenu: OptIndex = 162;
pub const kOptLangmap: OptIndex = 161;
pub const kOptKeywordprg: OptIndex = 160;
pub const kOptKeymodel: OptIndex = 159;
pub const kOptKeymap: OptIndex = 158;
pub const kOptJumpoptions: OptIndex = 157;
pub const kOptJoinspaces: OptIndex = 156;
pub const kOptIsprint: OptIndex = 155;
pub const kOptIskeyword: OptIndex = 154;
pub const kOptIsident: OptIndex = 153;
pub const kOptIsfname: OptIndex = 152;
pub const kOptInsertmode: OptIndex = 151;
pub const kOptInfercase: OptIndex = 150;
pub const kOptIndentkeys: OptIndex = 149;
pub const kOptIndentexpr: OptIndex = 148;
pub const kOptIncsearch: OptIndex = 147;
pub const kOptIncludeexpr: OptIndex = 146;
pub const kOptInclude: OptIndex = 145;
pub const kOptInccommand: OptIndex = 144;
pub const kOptImsearch: OptIndex = 143;
pub const kOptIminsert: OptIndex = 142;
pub const kOptImdisable: OptIndex = 141;
pub const kOptImcmdline: OptIndex = 140;
pub const kOptIgnorecase: OptIndex = 139;
pub const kOptIconstring: OptIndex = 138;
pub const kOptIcon: OptIndex = 137;
pub const kOptHlsearch: OptIndex = 136;
pub const kOptHkmapp: OptIndex = 135;
pub const kOptHkmap: OptIndex = 134;
pub const kOptHistory: OptIndex = 133;
pub const kOptHighlight: OptIndex = 132;
pub const kOptHidden: OptIndex = 131;
pub const kOptHelplang: OptIndex = 130;
pub const kOptHelpheight: OptIndex = 129;
pub const kOptHelpfile: OptIndex = 128;
pub const kOptGuitabtooltip: OptIndex = 127;
pub const kOptGuitablabel: OptIndex = 126;
pub const kOptGuioptions: OptIndex = 125;
pub const kOptGuifontwide: OptIndex = 124;
pub const kOptGuifont: OptIndex = 123;
pub const kOptGuicursor: OptIndex = 122;
pub const kOptGrepprg: OptIndex = 121;
pub const kOptGrepformat: OptIndex = 120;
pub const kOptGdefault: OptIndex = 119;
pub const kOptFsync: OptIndex = 118;
pub const kOptFormatprg: OptIndex = 117;
pub const kOptFormatoptions: OptIndex = 116;
pub const kOptFormatlistpat: OptIndex = 115;
pub const kOptFormatexpr: OptIndex = 114;
pub const kOptFoldtext: OptIndex = 113;
pub const kOptFoldopen: OptIndex = 112;
pub const kOptFoldnestmax: OptIndex = 111;
pub const kOptFoldminlines: OptIndex = 110;
pub const kOptFoldmethod: OptIndex = 109;
pub const kOptFoldmarker: OptIndex = 108;
pub const kOptFoldlevelstart: OptIndex = 107;
pub const kOptFoldlevel: OptIndex = 106;
pub const kOptFoldignore: OptIndex = 105;
pub const kOptFoldexpr: OptIndex = 104;
pub const kOptFoldenable: OptIndex = 103;
pub const kOptFoldcolumn: OptIndex = 102;
pub const kOptFoldclose: OptIndex = 101;
pub const kOptFixendofline: OptIndex = 100;
pub const kOptFindfunc: OptIndex = 99;
pub const kOptFillchars: OptIndex = 98;
pub const kOptFiletype: OptIndex = 97;
pub const kOptFileignorecase: OptIndex = 96;
pub const kOptFileformats: OptIndex = 95;
pub const kOptFileformat: OptIndex = 94;
pub const kOptFileencodings: OptIndex = 93;
pub const kOptFileencoding: OptIndex = 92;
pub const kOptExrc: OptIndex = 91;
pub const kOptExpandtab: OptIndex = 90;
pub const kOptEventignorewin: OptIndex = 89;
pub const kOptEventignore: OptIndex = 88;
pub const kOptErrorformat: OptIndex = 87;
pub const kOptErrorfile: OptIndex = 86;
pub const kOptErrorbells: OptIndex = 85;
pub const kOptEqualprg: OptIndex = 84;
pub const kOptEqualalways: OptIndex = 83;
pub const kOptEndofline: OptIndex = 82;
pub const kOptEndoffile: OptIndex = 81;
pub const kOptEncoding: OptIndex = 80;
pub const kOptEmoji: OptIndex = 79;
pub const kOptEdcompatible: OptIndex = 78;
pub const kOptEadirection: OptIndex = 77;
pub const kOptDisplay: OptIndex = 76;
pub const kOptDirectory: OptIndex = 75;
pub const kOptDigraph: OptIndex = 74;
pub const kOptDiffopt: OptIndex = 73;
pub const kOptDiffexpr: OptIndex = 72;
pub const kOptDiffanchors: OptIndex = 71;
pub const kOptDiff: OptIndex = 70;
pub const kOptDictionary: OptIndex = 69;
pub const kOptDelcombine: OptIndex = 68;
pub const kOptDefine: OptIndex = 67;
pub const kOptDebug: OptIndex = 66;
pub const kOptCursorlineopt: OptIndex = 65;
pub const kOptCursorline: OptIndex = 64;
pub const kOptCursorcolumn: OptIndex = 63;
pub const kOptCursorbind: OptIndex = 62;
pub const kOptCpoptions: OptIndex = 61;
pub const kOptCopyindent: OptIndex = 60;
pub const kOptConfirm: OptIndex = 59;
pub const kOptConceallevel: OptIndex = 58;
pub const kOptConcealcursor: OptIndex = 57;
pub const kOptCompletetimeout: OptIndex = 56;
pub const kOptCompleteslash: OptIndex = 55;
pub const kOptCompleteopt: OptIndex = 54;
pub const kOptCompleteitemalign: OptIndex = 53;
pub const kOptCompletefunc: OptIndex = 52;
pub const kOptComplete: OptIndex = 51;
pub const kOptCompatible: OptIndex = 50;
pub const kOptCommentstring: OptIndex = 49;
pub const kOptComments: OptIndex = 48;
pub const kOptColumns: OptIndex = 47;
pub const kOptColorcolumn: OptIndex = 46;
pub const kOptCmdwinheight: OptIndex = 45;
pub const kOptCmdheight: OptIndex = 44;
pub const kOptClipboard: OptIndex = 43;
pub const kOptCinwords: OptIndex = 42;
pub const kOptCinscopedecls: OptIndex = 41;
pub const kOptCinoptions: OptIndex = 40;
pub const kOptCinkeys: OptIndex = 39;
pub const kOptCindent: OptIndex = 38;
pub const kOptChistory: OptIndex = 37;
pub const kOptCharconvert: OptIndex = 36;
pub const kOptChannel: OptIndex = 35;
pub const kOptCedit: OptIndex = 34;
pub const kOptCdpath: OptIndex = 33;
pub const kOptCdhome: OptIndex = 32;
pub const kOptCasemap: OptIndex = 31;
pub const kOptBusy: OptIndex = 30;
pub const kOptBuftype: OptIndex = 29;
pub const kOptBuflisted: OptIndex = 28;
pub const kOptBufhidden: OptIndex = 27;
pub const kOptBrowsedir: OptIndex = 26;
pub const kOptBreakindentopt: OptIndex = 25;
pub const kOptBreakindent: OptIndex = 24;
pub const kOptBreakat: OptIndex = 23;
pub const kOptBomb: OptIndex = 22;
pub const kOptBinary: OptIndex = 21;
pub const kOptBelloff: OptIndex = 20;
pub const kOptBackupskip: OptIndex = 19;
pub const kOptBackupext: OptIndex = 18;
pub const kOptBackupdir: OptIndex = 17;
pub const kOptBackupcopy: OptIndex = 16;
pub const kOptBackup: OptIndex = 15;
pub const kOptBackspace: OptIndex = 14;
pub const kOptBackground: OptIndex = 13;
pub const kOptAutowriteall: OptIndex = 12;
pub const kOptAutowrite: OptIndex = 11;
pub const kOptAutoread: OptIndex = 10;
pub const kOptAutoindent: OptIndex = 9;
pub const kOptAutocompletetimeout: OptIndex = 8;
pub const kOptAutocompletedelay: OptIndex = 7;
pub const kOptAutocomplete: OptIndex = 6;
pub const kOptAutochdir: OptIndex = 5;
pub const kOptArabicshape: OptIndex = 4;
pub const kOptArabic: OptIndex = 3;
pub const kOptAmbiwidth: OptIndex = 2;
pub const kOptAllowrevins: OptIndex = 1;
pub const kOptAleph: OptIndex = 0;
pub const kOptInvalid: OptIndex = -1;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_52 {
    pub number: uint32_t,
    pub bytes: [uint8_t; 4],
}
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
pub const ARENA_EMPTY: Arena = Arena {
    cur_blk: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    pos: 0 as size_t,
    size: 0 as size_t,
};
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
static value_init_ptr_t: GlobalCell<ptr_t> = GlobalCell::new(NULL);
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn map_get_uint64_t_ptr_t(
    mut map: *mut Map_uint64_t_ptr_t,
    mut key: uint64_t,
) -> ptr_t {
    let mut k: uint32_t = mh_get_uint64_t(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_ptr_t.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
pub const VARNUMBER_MAX: ::core::ffi::c_long = INT64_MAX;
pub const VARNUMBER_MIN: ::core::ffi::c_long = INT64_MIN;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL_0,
};
#[inline(always)]
unsafe extern "C" fn lt(mut a: pos_T, mut b: pos_T) -> bool {
    if a.lnum != b.lnum {
        return a.lnum < b.lnum;
    } else if a.col != b.col {
        return a.col < b.col;
    } else {
        return a.coladd < b.coladd;
    };
}
#[inline(always)]
unsafe extern "C" fn equalpos(mut a: pos_T, mut b: pos_T) -> bool {
    return a.lnum == b.lnum && a.col == b.col && a.coladd == b.coladd;
}
#[inline(always)]
unsafe extern "C" fn clearpos(mut a: *mut pos_T) {
    (*a).lnum = 0 as ::core::ffi::c_int as linenr_T;
    (*a).col = 0 as ::core::ffi::c_int as colnr_T;
    (*a).coladd = 0 as ::core::ffi::c_int as colnr_T;
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const EX_NOSPC: ::core::ffi::c_uint = 0x10 as ::core::ffi::c_uint;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn find_channel(mut id: uint64_t) -> *mut Channel {
    return map_get_uint64_t_ptr_t(channels.ptr(), id) as *mut Channel;
}
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
static dummy_ap: GlobalCell<::core::ffi::VaListImpl> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 24], ::core::ffi::VaListImpl>([0u8; 24])
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
    let key: *const ::core::ffi::c_char = (*functions.ptr())[intidx.get() as usize].name;
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
    if (*functions.ptr())[intidx.get() as usize].max_argc as ::core::ffi::c_int
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
pub unsafe extern "C" fn find_internal_func(
    name: *const ::core::ffi::c_char,
) -> *const EvalFuncDef {
    let mut len: size_t = strlen(name);
    let mut index: ::core::ffi::c_int = find_internal_func_hash(name, len);
    return if index >= 0 as ::core::ffi::c_int {
        ((functions.ptr() as *const _) as *const EvalFuncDef).offset(index as isize)
    } else {
        ::core::ptr::null::<EvalFuncDef>()
    };
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
unsafe extern "C" fn float_op_wrapper(
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
unsafe extern "C" fn api_wrapper(
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
unsafe extern "C" fn f_abs(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        float_op_wrapper(
            argvars,
            rettv,
            EvalFuncData {
                float_func: Some(
                    fabs as unsafe extern "C" fn(::core::ffi::c_double) -> ::core::ffi::c_double,
                ),
            },
        );
    } else {
        let mut error: bool = false_0 != 0;
        let mut n: varnumber_T = tv_get_number_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut error,
        );
        if error {
            (*rettv).vval.v_number = -1 as varnumber_T;
        } else if n > 0 as varnumber_T {
            (*rettv).vval.v_number = n;
        } else {
            (*rettv).vval.v_number = -n;
        }
    };
}
unsafe extern "C" fn f_and(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) & tv_get_number_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    );
}
unsafe extern "C" fn f_api_info(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    object_to_vim(api_metadata(), rettv, ::core::ptr::null_mut::<Error>());
}
unsafe extern "C" fn f_atan2(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut fx: float_T = 0.;
    let mut fy: float_T = 0.;
    (*rettv).v_type = VAR_FLOAT;
    if tv_get_float_chk(argvars, &raw mut fx) as ::core::ffi::c_int != 0
        && tv_get_float_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut fy,
        ) as ::core::ffi::c_int
            != 0
    {
        (*rettv).vval.v_float =
            atan2(fx as ::core::ffi::c_double, fy as ::core::ffi::c_double) as float_T;
    } else {
        (*rettv).vval.v_float = 0.0f64 as float_T;
    };
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
unsafe extern "C" fn f_byte2line(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut boff: ::core::ffi::c_int =
        tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int;
    if boff < 0 as ::core::ffi::c_int {
        (*rettv).vval.v_number = -1 as varnumber_T;
    } else {
        (*rettv).vval.v_number =
            ml_find_line_or_offset(curbuf.get(), 0 as linenr_T, &raw mut boff, false_0 != 0)
                as varnumber_T;
    };
}
unsafe extern "C" fn f_call(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if tv_check_for_list_arg(argvars, 1 as ::core::ffi::c_int) == FAIL {
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize))
        .vval
        .v_list
        .is_null()
    {
        return;
    }
    let mut owned: bool = false_0 != 0;
    let mut func: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut partial: *mut partial_T = ::core::ptr::null_mut::<partial_T>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        func = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_string;
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        partial = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_partial;
        func = partial_name(partial);
    } else if nlua_is_table_from_lua(argvars.offset(0 as ::core::ffi::c_int as isize)) {
        func = nlua_register_table_as_callable(argvars.offset(0 as ::core::ffi::c_int as isize));
        owned = true_0 != 0;
    } else {
        func = tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize))
            as *mut ::core::ffi::c_char;
    }
    if func.is_null() || *func as ::core::ffi::c_int == NUL {
        return;
    }
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut p: *mut ::core::ffi::c_char = func;
        tofree = trans_function_name(
            &raw mut p,
            false_0 != 0,
            TFN_INT as ::core::ffi::c_int | TFN_QUIET as ::core::ffi::c_int,
            ::core::ptr::null_mut::<funcdict_T>(),
            ::core::ptr::null_mut::<*mut partial_T>(),
        );
        if tofree.is_null() {
            emsg_funcname(
                &raw const e_unknown_function_str as *const ::core::ffi::c_char,
                func,
            );
            return;
        }
        func = tofree;
    }
    let mut selfdict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    '_done: {
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if tv_check_for_dict_arg(argvars, 2 as ::core::ffi::c_int) == FAIL {
                break '_done;
            } else {
                selfdict = (*argvars.offset(2 as ::core::ffi::c_int as isize))
                    .vval
                    .v_dict;
            }
        }
        func_call(
            func,
            argvars.offset(1 as ::core::ffi::c_int as isize),
            partial,
            selfdict,
            rettv,
        );
    }
    if owned {
        func_unref(func);
    }
    xfree(tofree as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn f_changenr(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = (*curbuf.get()).b_u_seq_cur as varnumber_T;
}
unsafe extern "C" fn f_chanclose(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let mut part: ChannelPart = kChannelPartAll;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut stream: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
        if strcmp(stream, b"stdin\0".as_ptr() as *const ::core::ffi::c_char) == 0 {
            part = kChannelPartStdin;
        } else if strcmp(stream, b"stdout\0".as_ptr() as *const ::core::ffi::c_char) == 0 {
            part = kChannelPartStdout;
        } else if strcmp(stream, b"stderr\0".as_ptr() as *const ::core::ffi::c_char) == 0 {
            part = kChannelPartStderr;
        } else if strcmp(stream, b"rpc\0".as_ptr() as *const ::core::ffi::c_char) == 0 {
            part = kChannelPartRpc;
        } else {
            semsg(
                gettext(b"Invalid channel stream \"%s\"\0".as_ptr() as *const ::core::ffi::c_char),
                stream,
            );
            return;
        }
    }
    let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    (*rettv).vval.v_number = channel_close(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_number as uint64_t,
        part,
        &raw mut error,
    ) as varnumber_T;
    if (*rettv).vval.v_number == 0 {
        emsg(error);
    }
}
unsafe extern "C" fn f_chansend(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let mut input_len: ptrdiff_t = 0 as ptrdiff_t;
    let mut input: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut id: uint64_t = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_number as uint64_t;
    let mut crlf: bool = false_0 != 0;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let b: *const blob_T = (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_blob;
        input_len = tv_blob_len(b) as ptrdiff_t;
        if input_len > 0 as ptrdiff_t {
            input = xmemdup((*b).bv_ga.ga_data, input_len as size_t) as *mut ::core::ffi::c_char;
        }
    } else {
        input = save_tv_as_string(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut input_len,
            false_0 != 0,
            crlf,
        );
    }
    if input.is_null() {
        return;
    }
    let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    (*rettv).vval.v_number =
        channel_send(id, input, input_len as size_t, true_0 != 0, &raw mut error) as varnumber_T;
    if !error.is_null() {
        emsg(error);
    }
}
unsafe extern "C" fn f_char2nr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !tv_check_num(argvars.offset(1 as ::core::ffi::c_int as isize)) {
            return;
        }
    }
    (*rettv).vval.v_number = utf_ptr2char(tv_get_string(
        argvars.offset(0 as ::core::ffi::c_int as isize),
    )) as varnumber_T;
}
unsafe extern "C" fn get_col(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut charcol: bool,
) {
    if tv_check_for_string_or_list_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || tv_check_for_opt_number_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
    {
        return;
    }
    let mut wp: *mut win_T = curwin.get();
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
        wp = win_id2wp_tp(
            tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int,
            &raw mut tp,
        );
        if wp.is_null() || tp.is_null() {
            return;
        }
        check_cursor(wp);
    }
    let mut bp: *mut buf_T = (*wp).w_buffer;
    let mut col: colnr_T = 0 as colnr_T;
    let mut fnum: ::core::ffi::c_int = (*bp).handle as ::core::ffi::c_int;
    let mut fp: *mut pos_T = var2fpos(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        false_0 != 0,
        &raw mut fnum,
        charcol,
        wp,
    );
    if !fp.is_null() && fnum == (*bp).handle {
        if (*fp).col == MAXCOL as ::core::ffi::c_int {
            if (*fp).lnum <= (*bp).b_ml.ml_line_count {
                col = (ml_get_buf_len(bp, (*fp).lnum) + 1 as ::core::ffi::c_int) as colnr_T;
            } else {
                col = MAXCOL as ::core::ffi::c_int as colnr_T;
            }
        } else {
            col = ((*fp).col as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as colnr_T;
            if virtual_active(wp) as ::core::ffi::c_int != 0 && fp == &raw mut (*wp).w_cursor {
                let mut p: *mut ::core::ffi::c_char =
                    ml_get_buf(bp, (*wp).w_cursor.lnum).offset((*wp).w_cursor.col as isize);
                if (*wp).w_cursor.coladd
                    >= win_chartabsize(wp, p, (*wp).w_virtcol - (*wp).w_cursor.coladd)
                {
                    let mut l: ::core::ffi::c_int = 0;
                    if *p as ::core::ffi::c_int != NUL && {
                        l = utfc_ptr2len(p);
                        *p.offset(l as isize) as ::core::ffi::c_int == NUL
                    } {
                        col += l;
                    }
                }
            }
        }
    }
    (*rettv).vval.v_number = col as varnumber_T;
}
unsafe extern "C" fn f_charcol(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    get_col(argvars, rettv, true_0 != 0);
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
unsafe extern "C" fn f_col(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    get_col(argvars, rettv, false_0 != 0);
}
unsafe extern "C" fn f_confirm(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    let mut buf2: [::core::ffi::c_char; 65] = [0; 65];
    let mut buttons: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut def: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut type_0: ::core::ffi::c_int = VIM_GENERIC as ::core::ffi::c_int;
    let mut error: bool = false_0 != 0;
    let mut message: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    if message.is_null() {
        error = true_0 != 0;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        buttons = tv_get_string_buf_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        if buttons.is_null() {
            error = true_0 != 0;
        }
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            def = tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) as ::core::ffi::c_int;
            if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut typestr: *const ::core::ffi::c_char = tv_get_string_buf_chk(
                    argvars.offset(3 as ::core::ffi::c_int as isize),
                    &raw mut buf2 as *mut ::core::ffi::c_char,
                );
                if typestr.is_null() {
                    error = true_0 != 0;
                } else {
                    match if (*typestr as ::core::ffi::c_int) < 'a' as ::core::ffi::c_int
                        || *typestr as ::core::ffi::c_int > 'z' as ::core::ffi::c_int
                    {
                        *typestr as ::core::ffi::c_int
                    } else {
                        *typestr as ::core::ffi::c_int
                            - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                    } {
                        69 => {
                            type_0 = VIM_ERROR as ::core::ffi::c_int;
                        }
                        81 => {
                            type_0 = VIM_QUESTION as ::core::ffi::c_int;
                        }
                        73 => {
                            type_0 = VIM_INFO as ::core::ffi::c_int;
                        }
                        87 => {
                            type_0 = VIM_WARNING as ::core::ffi::c_int;
                        }
                        71 => {
                            type_0 = VIM_GENERIC as ::core::ffi::c_int;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    if buttons.is_null() || *buttons as ::core::ffi::c_int == NUL {
        buttons = gettext(b"&Ok\0".as_ptr() as *const ::core::ffi::c_char);
    }
    if !error {
        (*rettv).vval.v_number = do_dialog(
            type_0,
            ::core::ptr::null::<::core::ffi::c_char>(),
            message,
            buttons,
            def,
            ::core::ptr::null::<::core::ffi::c_char>(),
            false_0,
        ) as varnumber_T;
    }
}
unsafe extern "C" fn f_copy(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    var_item_copy(
        ::core::ptr::null::<vimconv_T>(),
        argvars.offset(0 as ::core::ffi::c_int as isize),
        rettv,
        false_0 != 0,
        0 as ::core::ffi::c_int,
    );
}
unsafe extern "C" fn f_ctxget(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut index: size_t = 0 as size_t;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        index = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_number as size_t;
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"expected nothing or a Number as an argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut ctx: *mut Context = ctx_get(index);
    if ctx.is_null() {
        semsg(
            gettext(&raw const e_invargNval as *const ::core::ffi::c_char),
            b"index\0".as_ptr() as *const ::core::ffi::c_char,
            b"out of bounds\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arena: Arena = ARENA_EMPTY;
    let mut ctx_dict: Dict = ctx_to_dict(ctx, &raw mut arena);
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    object_to_vim(
        object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed_16 { dict: ctx_dict },
        },
        rettv,
        &raw mut err,
    );
    arena_mem_free(arena_finish(&raw mut arena));
    api_clear_error(&raw mut err);
}
unsafe extern "C" fn f_ctxpop(
    mut _argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if !ctx_restore(::core::ptr::null_mut::<Context>(), kCtxAll.get()) {
        emsg(gettext(
            b"Context stack is empty\0".as_ptr() as *const ::core::ffi::c_char
        ));
    }
}
unsafe extern "C" fn f_ctxpush(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut types: ::core::ffi::c_int = kCtxAll.get();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        types = 0 as ::core::ffi::c_int;
        let l_: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_list;
        if !l_.is_null() {
            let mut li: *mut listitem_T = (*l_).lv_first;
            while !li.is_null() {
                let mut tv_li: *mut typval_T = &raw mut (*li).li_tv;
                if (*tv_li).v_type as ::core::ffi::c_uint
                    == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if strequal(
                        (*tv_li).vval.v_string,
                        b"regs\0".as_ptr() as *const ::core::ffi::c_char,
                    ) {
                        types |= kCtxRegs as ::core::ffi::c_int;
                    } else if strequal(
                        (*tv_li).vval.v_string,
                        b"jumps\0".as_ptr() as *const ::core::ffi::c_char,
                    ) {
                        types |= kCtxJumps as ::core::ffi::c_int;
                    } else if strequal(
                        (*tv_li).vval.v_string,
                        b"bufs\0".as_ptr() as *const ::core::ffi::c_char,
                    ) {
                        types |= kCtxBufs as ::core::ffi::c_int;
                    } else if strequal(
                        (*tv_li).vval.v_string,
                        b"gvars\0".as_ptr() as *const ::core::ffi::c_char,
                    ) {
                        types |= kCtxGVars as ::core::ffi::c_int;
                    } else if strequal(
                        (*tv_li).vval.v_string,
                        b"sfuncs\0".as_ptr() as *const ::core::ffi::c_char,
                    ) {
                        types |= kCtxSFuncs as ::core::ffi::c_int;
                    } else if strequal(
                        (*tv_li).vval.v_string,
                        b"funcs\0".as_ptr() as *const ::core::ffi::c_char,
                    ) {
                        types |= kCtxFuncs as ::core::ffi::c_int;
                    }
                }
                li = (*li).li_next;
            }
        }
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"expected nothing or a List as an argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    ctx_save(::core::ptr::null_mut::<Context>(), types);
}
unsafe extern "C" fn f_ctxset(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"expected dictionary as first argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut index: size_t = 0 as size_t;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        index = (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_number as size_t;
    } else if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"expected nothing or a Number as second argument\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut ctx: *mut Context = ctx_get(index);
    if ctx.is_null() {
        semsg(
            gettext(&raw const e_invargNval as *const ::core::ffi::c_char),
            b"index\0".as_ptr() as *const ::core::ffi::c_char,
            b"out of bounds\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let save_did_emsg: ::core::ffi::c_int = did_emsg.get();
    did_emsg.set(false_0);
    let mut arena: Arena = ARENA_EMPTY;
    let mut dict: Dict = vim_to_object(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut arena,
        true_0 != 0,
    )
    .data
    .dict;
    let mut tmp: Context = CONTEXT_INIT;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    ctx_from_dict(dict, &raw mut tmp, &raw mut err);
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        semsg(b"%s\0".as_ptr() as *const ::core::ffi::c_char, err.msg);
        ctx_free(&raw mut tmp);
    } else {
        ctx_free(ctx);
        *ctx = tmp;
    }
    arena_mem_free(arena_finish(&raw mut arena));
    api_clear_error(&raw mut err);
    did_emsg.set(save_did_emsg);
}
unsafe extern "C" fn f_ctxsize(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = ctx_size() as varnumber_T;
}
unsafe extern "C" fn set_cursorpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut charcol: bool,
) {
    let mut lnum: linenr_T = 0;
    let mut col: colnr_T = 0;
    let mut coladd: colnr_T = 0 as colnr_T;
    let mut set_curswant: bool = true_0 != 0;
    (*rettv).vval.v_number = -1 as varnumber_T;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut pos: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut curswant: colnr_T = -1 as colnr_T;
        if list2fpos(
            argvars,
            &raw mut pos,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            &raw mut curswant,
            charcol,
        ) == FAIL
        {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return;
        }
        lnum = pos.lnum;
        col = pos.col;
        coladd = pos.coladd;
        if curswant >= 0 as ::core::ffi::c_int {
            (*curwin.get()).w_curswant =
                (curswant as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as colnr_T;
            set_curswant = false_0 != 0;
        }
    } else if ((*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
        && ((*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        lnum = tv_get_lnum(argvars);
        if lnum < 0 as linenr_T {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
            );
        } else if lnum == 0 as linenr_T {
            lnum = (*curwin.get()).w_cursor.lnum;
        }
        col = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            ::core::ptr::null_mut::<bool>(),
        ) as colnr_T;
        if charcol {
            col = (buf_charidx_to_byteidx(curbuf.get(), lnum, col) + 1 as ::core::ffi::c_int)
                as colnr_T;
        }
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            coladd = tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                ::core::ptr::null_mut::<bool>(),
            ) as colnr_T;
        }
    } else {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    if lnum < 0 as linenr_T || col < 0 as ::core::ffi::c_int || coladd < 0 as ::core::ffi::c_int {
        return;
    }
    if lnum > 0 as linenr_T {
        (*curwin.get()).w_cursor.lnum = lnum;
    }
    if col != MAXCOL as ::core::ffi::c_int && {
        col -= 1;
        col < 0 as ::core::ffi::c_int
    } {
        col = 0 as ::core::ffi::c_int as colnr_T;
    }
    (*curwin.get()).w_cursor.col = col;
    (*curwin.get()).w_cursor.coladd = coladd;
    check_cursor(curwin.get());
    mb_adjust_cursor();
    (*curwin.get()).w_set_curswant = set_curswant as ::core::ffi::c_int;
    (*rettv).vval.v_number = 0 as varnumber_T;
}
unsafe extern "C" fn f_cursor(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    set_cursorpos(argvars, rettv, false_0 != 0);
}
unsafe extern "C" fn f_debugbreak(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = FAIL as varnumber_T;
    let mut pid: ::core::ffi::c_int =
        tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
    if pid == 0 as ::core::ffi::c_int {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    uv_kill(pid, SIGINT);
}
unsafe extern "C" fn f_deepcopy(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if tv_check_for_opt_bool_arg(argvars, 1 as ::core::ffi::c_int) == FAIL {
        return;
    }
    let mut noref: varnumber_T = 0 as varnumber_T;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        noref = tv_get_bool_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            ::core::ptr::null_mut::<bool>(),
        );
    }
    var_item_copy(
        ::core::ptr::null::<vimconv_T>(),
        argvars.offset(0 as ::core::ffi::c_int as isize),
        rettv,
        true_0 != 0,
        if noref == 0 as varnumber_T {
            get_copyID()
        } else {
            0 as ::core::ffi::c_int
        },
    );
}
unsafe extern "C" fn f_dictwatcheradd(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"dict\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_dict
        .is_null()
    {
        let arg_errmsg: *const ::core::ffi::c_char =
            gettext(b"dictwatcheradd() argument\0".as_ptr() as *const ::core::ffi::c_char);
        let arg_errmsg_len: size_t = strlen(arg_errmsg);
        semsg(
            gettext(&raw const e_cannot_change_readonly_variable_str as *const ::core::ffi::c_char),
            arg_errmsg_len as ::core::ffi::c_int,
            arg_errmsg,
        );
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"key\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let key_pattern: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize));
    if key_pattern.is_null() {
        return;
    }
    let key_pattern_len: size_t = strlen(key_pattern);
    let mut callback: Callback = Callback {
        data: C2Rust_Unnamed_22 {
            funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        type_0: kCallbackNone,
    };
    if !callback_from_typval(
        &raw mut callback,
        argvars.offset(2 as ::core::ffi::c_int as isize),
    ) {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"funcref\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    tv_dict_watcher_add(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_dict,
        key_pattern,
        key_pattern_len,
        callback,
    );
}
unsafe extern "C" fn f_dictwatcherdel(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"dict\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"funcref\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let key_pattern: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize));
    if key_pattern.is_null() {
        return;
    }
    let mut callback: Callback = Callback {
        data: C2Rust_Unnamed_22 {
            funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        type_0: kCallbackNone,
    };
    if !callback_from_typval(
        &raw mut callback,
        argvars.offset(2 as ::core::ffi::c_int as isize),
    ) {
        return;
    }
    if !tv_dict_watcher_remove(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_dict,
        key_pattern,
        strlen(key_pattern),
        callback,
    ) {
        emsg(
            b"Couldn't find a watcher matching key and callback\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
    }
    callback_free(&raw mut callback);
}
unsafe extern "C" fn f_did_filetype(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = (*curbuf.get()).b_did_filetype as varnumber_T;
}
unsafe extern "C" fn f_empty(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut n: bool = true_0 != 0;
    match (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint {
        2 | 3 => {
            n = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_string
                .is_null()
                || *(*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_string as ::core::ffi::c_int
                    == NUL;
        }
        9 => {
            n = false_0 != 0;
        }
        1 => {
            n = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_number
                == 0 as varnumber_T;
        }
        6 => {
            n = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_float
                == 0.0f64;
        }
        4 => {
            n = tv_list_len(
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_list,
            ) == 0 as ::core::ffi::c_int;
        }
        5 => {
            n = tv_dict_len(
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_dict,
            ) == 0 as ::core::ffi::c_long;
        }
        7 => {
            match (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_bool as ::core::ffi::c_uint
            {
                1 => {
                    n = false_0 != 0;
                }
                0 => {
                    n = true_0 != 0;
                }
                _ => {}
            }
        }
        8 => {
            n = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_special as ::core::ffi::c_uint
                == kSpecialVarNull as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        10 => {
            n = tv_blob_len(
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_blob,
            ) == 0 as ::core::ffi::c_int;
        }
        0 => {
            internal_error(b"f_empty(UNKNOWN)\0".as_ptr() as *const ::core::ffi::c_char);
        }
        _ => {}
    }
    (*rettv).vval.v_number = n as varnumber_T;
}
unsafe extern "C" fn f_environ(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_dict_alloc_ret(rettv);
    let mut env_size: size_t = os_get_fullenv_size();
    let mut env: *mut *mut ::core::ffi::c_char = xmalloc(
        ::core::mem::size_of::<*mut ::core::ffi::c_char>()
            .wrapping_mul(env_size.wrapping_add(1 as size_t)),
    ) as *mut *mut ::core::ffi::c_char;
    *env.offset(env_size as isize) = ::core::ptr::null_mut::<::core::ffi::c_char>();
    os_copy_fullenv(env, env_size);
    let mut i: ssize_t = env_size as ssize_t - 1 as ssize_t;
    while i >= 0 as ssize_t {
        let mut str: *const ::core::ffi::c_char = *env.offset(i as isize);
        let end: *const ::core::ffi::c_char = strchr(
            str.offset(
                (if *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '=' as ::core::ffi::c_int
                {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as isize,
            ),
            '=' as ::core::ffi::c_int,
        );
        '_c2rust_label: {
            if !end.is_null() {
            } else {
                __assert_fail(
                    b"end != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/funcs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1177 as ::core::ffi::c_uint,
                    b"void f_environ(typval_T *, typval_T *, EvalFuncData)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let mut len: ptrdiff_t = end.offset_from(str);
        '_c2rust_label_0: {
            if len > 0 as ptrdiff_t {
            } else {
                __assert_fail(
                    b"len > 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/funcs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1179 as ::core::ffi::c_uint,
                    b"void f_environ(typval_T *, typval_T *, EvalFuncData)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let mut value: *const ::core::ffi::c_char = str
            .offset(len as isize)
            .offset(1 as ::core::ffi::c_int as isize);
        let mut c: ::core::ffi::c_char = *(*env.offset(i as isize)).offset(len as isize);
        *(*env.offset(i as isize)).offset(len as isize) = NUL as ::core::ffi::c_char;
        let key: *mut ::core::ffi::c_char = xstrdup(str);
        *(*env.offset(i as isize)).offset(len as isize) = c;
        if !tv_dict_find((*rettv).vval.v_dict, key, len).is_null() {
            xfree(key as *mut ::core::ffi::c_void);
        } else {
            tv_dict_add_str((*rettv).vval.v_dict, key, len as size_t, value);
            xfree(key as *mut ::core::ffi::c_void);
        }
        i -= 1;
    }
    os_free_fullenv(env);
}
unsafe extern "C" fn f_escape(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    (*rettv).vval.v_string = vim_strsave_escaped(
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
        tv_get_string_buf(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut buf as *mut ::core::ffi::c_char,
        ),
    );
    (*rettv).v_type = VAR_STRING;
}
unsafe extern "C" fn f_getenv(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut p: *mut ::core::ffi::c_char = vim_getenv(tv_get_string(
        argvars.offset(0 as ::core::ffi::c_int as isize),
    ));
    if p.is_null() {
        (*rettv).v_type = VAR_SPECIAL;
        (*rettv).vval.v_special = kSpecialVarNull;
        return;
    }
    (*rettv).vval.v_string = p;
    (*rettv).v_type = VAR_STRING;
}
unsafe extern "C" fn f_eval(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut s: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    if !s.is_null() {
        s = skipwhite(s);
    }
    let expr_start: *const ::core::ffi::c_char = s;
    if s.is_null()
        || eval1(
            &raw mut s as *mut *mut ::core::ffi::c_char,
            rettv,
            EVALARG_EVALUATE.ptr(),
        ) == FAIL
    {
        if !expr_start.is_null() && !aborting() {
            semsg(
                gettext(&raw const e_invexpr2 as *const ::core::ffi::c_char),
                expr_start,
            );
        }
        need_clr_eos.set(false_0 != 0);
        (*rettv).v_type = VAR_NUMBER;
        (*rettv).vval.v_number = 0 as varnumber_T;
    } else if *s as ::core::ffi::c_int != NUL {
        semsg(
            gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
            s,
        );
    }
}
unsafe extern "C" fn f_eventhandler(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = vgetc_busy.get() as varnumber_T;
}
unsafe extern "C" fn get_list_line(
    mut _c: ::core::ffi::c_int,
    mut cookie: *mut ::core::ffi::c_void,
    mut _indent: ::core::ffi::c_int,
    mut _do_concat: bool,
) -> *mut ::core::ffi::c_char {
    let p: *mut GetListLineCookie = cookie as *mut GetListLineCookie;
    let item: *const listitem_T = (*p).li;
    if item.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    let s: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        &raw const (*item).li_tv,
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    (*p).li = (*item).li_next;
    return if s.is_null() {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    } else {
        xstrdup(s)
    };
}
pub unsafe extern "C" fn execute_common(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut arg_off: ::core::ffi::c_int,
) {
    let save_msg_silent: ::core::ffi::c_int = msg_silent.get();
    let save_emsg_silent: ::core::ffi::c_int = emsg_silent.get();
    let save_emsg_noredir: bool = emsg_noredir.get();
    let save_redir_off: bool = redir_off.get();
    let save_capture_ga: *mut garray_T = capture_ga.get();
    let save_msg_col: ::core::ffi::c_int = msg_col.get();
    let mut echo_output: bool = false_0 != 0;
    if check_secure() {
        return;
    }
    if (*argvars.offset((arg_off + 1 as ::core::ffi::c_int) as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut buf: [::core::ffi::c_char; 65] = [0; 65];
        let s: *const ::core::ffi::c_char = tv_get_string_buf_chk(
            argvars.offset((arg_off + 1 as ::core::ffi::c_int) as isize),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        if s.is_null() {
            return;
        }
        if *s as ::core::ffi::c_int == NUL {
            echo_output = true_0 != 0;
        }
        if strncmp(
            s,
            b"silent\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            (*msg_silent.ptr()) += 1;
        }
        if strcmp(s, b"silent!\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
        {
            emsg_silent.set(true_0);
            emsg_noredir.set(true_0 != 0);
        }
    } else {
        (*msg_silent.ptr()) += 1;
    }
    let mut capture_local: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut capture_local,
        ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
        80 as ::core::ffi::c_int,
    );
    capture_ga.set(&raw mut capture_local);
    redir_off.set(false_0 != 0);
    if !echo_output {
        msg_col.set(0 as ::core::ffi::c_int);
    }
    if (*argvars.offset(arg_off as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        do_cmdline_cmd(tv_get_string(argvars.offset(arg_off as isize)));
    } else if !(*argvars.offset(arg_off as isize)).vval.v_list.is_null() {
        let list: *mut list_T = (*argvars.offset(arg_off as isize)).vval.v_list;
        tv_list_ref(list);
        let mut cookie: GetListLineCookie = GetListLineCookie {
            l: list,
            li: tv_list_first(list),
        };
        do_cmdline(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            Some(
                get_list_line
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut ::core::ffi::c_void,
                        ::core::ffi::c_int,
                        bool,
                    ) -> *mut ::core::ffi::c_char,
            ),
            &raw mut cookie as *mut ::core::ffi::c_void,
            DOCMD_NOWAIT as ::core::ffi::c_int
                | DOCMD_VERBOSE as ::core::ffi::c_int
                | DOCMD_REPEAT as ::core::ffi::c_int
                | DOCMD_KEYTYPED as ::core::ffi::c_int,
        );
        tv_list_unref(list);
    }
    msg_silent.set(save_msg_silent);
    emsg_silent.set(save_emsg_silent);
    emsg_noredir.set(save_emsg_noredir);
    redir_off.set(save_redir_off);
    if echo_output {
        msg_col.set(0 as ::core::ffi::c_int);
    } else {
        msg_col.set(save_msg_col);
    }
    ga_append(capture_ga.get(), NUL as uint8_t);
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = (*capture_ga.get()).ga_data as *mut ::core::ffi::c_char;
    capture_ga.set(save_capture_ga);
}
unsafe extern "C" fn f_execute(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    execute_common(argvars, rettv, 0 as ::core::ffi::c_int);
}
unsafe extern "C" fn f_exists(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut n: ::core::ffi::c_int = false_0;
    let mut p: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    if *p as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
        if os_env_exists(p.offset(1 as ::core::ffi::c_int as isize), false_0 != 0) {
            n = true_0;
        } else {
            let exp_0: *mut ::core::ffi::c_char = expand_env_save(p as *mut ::core::ffi::c_char);
            if !exp_0.is_null() && *exp_0 as ::core::ffi::c_int != '$' as ::core::ffi::c_int {
                n = true_0;
            }
            xfree(exp_0 as *mut ::core::ffi::c_void);
        }
    } else if *p as ::core::ffi::c_int == '&' as ::core::ffi::c_int
        || *p as ::core::ffi::c_int == '+' as ::core::ffi::c_int
    {
        n = (eval_option(&raw mut p, ::core::ptr::null_mut::<typval_T>(), true_0 != 0) == OK)
            as ::core::ffi::c_int;
        if *skipwhite(p) as ::core::ffi::c_int != NUL {
            n = false_0;
        }
    } else if *p as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
        n = if strnequal(
            p,
            b"*v:lua.\0".as_ptr() as *const ::core::ffi::c_char,
            7 as size_t,
        ) as ::core::ffi::c_int
            != 0
        {
            nlua_func_exists(p.offset(7 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
        } else {
            function_exists(p.offset(1 as ::core::ffi::c_int as isize), false_0 != 0)
                as ::core::ffi::c_int
        };
    } else if *p as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
        n = cmd_exists(p.offset(1 as ::core::ffi::c_int as isize));
    } else if *p as ::core::ffi::c_int == '#' as ::core::ffi::c_int {
        if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '#' as ::core::ffi::c_int
        {
            n = autocmd_supported(p.offset(2 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
        } else {
            n = au_exists(p.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
        }
    } else {
        n = var_exists(p) as ::core::ffi::c_int;
    }
    (*rettv).vval.v_number = n as varnumber_T;
}
unsafe extern "C" fn f_expand(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut options: ::core::ffi::c_int = WILD_SILENT as ::core::ffi::c_int
        | WILD_USE_NL as ::core::ffi::c_int
        | WILD_LIST_NOTFOUND as ::core::ffi::c_int;
    let mut error: bool = false_0 != 0;
    (*rettv).v_type = VAR_STRING;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && tv_get_number_chk(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) != 0
        && !error
    {
        tv_list_set_ret(rettv, ::core::ptr::null_mut::<list_T>());
    }
    let mut s: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    if *s as ::core::ffi::c_int == '%' as ::core::ffi::c_int
        || *s as ::core::ffi::c_int == '#' as ::core::ffi::c_int
        || *s as ::core::ffi::c_int == '<' as ::core::ffi::c_int
    {
        if p_verbose.get() == 0 as OptInt {
            (*emsg_off.ptr()) += 1;
        }
        let mut len: size_t = 0;
        let mut errormsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut result: *mut ::core::ffi::c_char = eval_vars(
            s as *mut ::core::ffi::c_char,
            s,
            &raw mut len,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut errormsg,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            false_0 != 0,
        );
        if p_verbose.get() == 0 as OptInt {
            (*emsg_off.ptr()) -= 1;
        } else if !errormsg.is_null() {
            emsg(errormsg);
        }
        if (*rettv).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_list_alloc_ret(rettv, !result.is_null() as ::core::ffi::c_int as ptrdiff_t);
            if !result.is_null() {
                tv_list_append_string((*rettv).vval.v_list, result, -1 as ssize_t);
            }
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut result as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_0;
            let _ = *ptr_;
        } else {
            (*rettv).vval.v_string = result;
        }
    } else {
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            && tv_get_number_chk(
                argvars.offset(1 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) != 0
        {
            options |= WILD_KEEP_ALL as ::core::ffi::c_int;
        }
        if !error {
            let mut xpc: expand_T = expand_T {
                xp_pattern: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                xp_context: 0,
                xp_pattern_len: 0,
                xp_prefix: XP_PREFIX_NONE,
                xp_arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                xp_luaref: 0,
                xp_script_ctx: sctx_T {
                    sc_sid: 0,
                    sc_seq: 0,
                    sc_lnum: 0,
                    sc_chan: 0,
                },
                xp_backslash: 0,
                xp_shell: false,
                xp_numfiles: 0,
                xp_col: 0,
                xp_selected: 0,
                xp_orig: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                xp_files: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                xp_line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                xp_buf: [0; 256],
                xp_search_dir: kDirectionNotSet,
                xp_pre_incsearch_pos: pos_T {
                    lnum: 0,
                    col: 0,
                    coladd: 0,
                },
            };
            ExpandInit(&raw mut xpc);
            xpc.xp_context = EXPAND_FILES as ::core::ffi::c_int;
            if p_wic.get() != 0 {
                options += WILD_ICASE as ::core::ffi::c_int;
            }
            if (*rettv).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*rettv).vval.v_string = ExpandOne(
                    &raw mut xpc,
                    s as *mut ::core::ffi::c_char,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    options,
                    WILD_ALL as ::core::ffi::c_int,
                );
            } else {
                ExpandOne(
                    &raw mut xpc,
                    s as *mut ::core::ffi::c_char,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    options,
                    WILD_ALL_KEEP as ::core::ffi::c_int,
                );
                tv_list_alloc_ret(rettv, xpc.xp_numfiles as ptrdiff_t);
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < xpc.xp_numfiles {
                    tv_list_append_string(
                        (*rettv).vval.v_list,
                        *xpc.xp_files.offset(i as isize),
                        -1 as ssize_t,
                    );
                    i += 1;
                }
                ExpandCleanup(&raw mut xpc);
            }
        } else {
            (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
    };
}
unsafe extern "C" fn f_menu_get(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    let mut modes: ::core::ffi::c_int = MENU_ALL_MODES as ::core::ffi::c_int;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let strmodes: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
        modes = get_menu_cmd_modes(
            strmodes,
            false_0 != 0,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ::core::ptr::null_mut::<bool>(),
        );
    }
    menu_get(
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_char,
        modes,
        (*rettv).vval.v_list,
    );
}
unsafe extern "C" fn f_expandcmd(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut errormsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut emsgoff: bool = true_0 != 0;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        && tv_dict_get_bool(
            (*argvars.offset(1 as ::core::ffi::c_int as isize))
                .vval
                .v_dict,
            b"errmsg\0".as_ptr() as *const ::core::ffi::c_char,
            kBoolVarFalse as ::core::ffi::c_int,
        ) != 0
    {
        emsgoff = false_0 != 0;
    }
    (*rettv).v_type = VAR_STRING;
    let mut cmdstr: *mut ::core::ffi::c_char = xstrdup(tv_get_string(
        argvars.offset(0 as ::core::ffi::c_int as isize),
    ));
    let mut eap: exarg_T = exarg {
        arg: cmdstr,
        args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmd: cmdstr,
        cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdidx: CMD_USER,
        argt: 0,
        skip: 0,
        forceit: 0,
        addr_count: 0,
        line1: 0,
        line2: 0,
        addr_type: ADDR_LINES,
        flags: 0,
        do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        do_ecmd_lnum: 0,
        append: 0,
        usefilter: false_0,
        amount: 0,
        regname: 0,
        force_bin: 0,
        read_edit: 0,
        mkdir_p: 0,
        force_ff: 0,
        force_enc: 0,
        bad_char: 0,
        useridx: 0,
        errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ea_getline: None,
        cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cstack: ::core::ptr::null_mut::<cstack_T>(),
    };
    eap.argt = (eap.argt as ::core::ffi::c_uint | EX_NOSPC) as uint32_t;
    if emsgoff {
        (*emsg_off.ptr()) += 1;
    }
    if expand_filename(&raw mut eap, &raw mut cmdstr, &raw mut errormsg) == FAIL {
        if !emsgoff && !errormsg.is_null() && *errormsg as ::core::ffi::c_int != NUL {
            emsg(errormsg);
        }
    }
    if emsgoff {
        (*emsg_off.ptr()) -= 1;
    }
    (*rettv).vval.v_string = cmdstr;
}
unsafe extern "C" fn flatten_common(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut make_copy: bool,
) {
    let mut error: bool = false_0 != 0;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_listarg as *const ::core::ffi::c_char),
            b"flatten()\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut maxdepth: ::core::ffi::c_int = 0;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        maxdepth = 999999 as ::core::ffi::c_int;
    } else {
        maxdepth = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) as ::core::ffi::c_int;
        if error {
            return;
        }
        if maxdepth < 0 as ::core::ffi::c_int {
            emsg(gettext(
                b"E900: maxdepth must be non-negative number\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return;
        }
    }
    let mut list: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    (*rettv).v_type = VAR_LIST;
    (*rettv).vval.v_list = list;
    if list.is_null() {
        return;
    }
    if make_copy {
        list = tv_list_copy(
            ::core::ptr::null::<vimconv_T>(),
            list,
            false_0 != 0,
            get_copyID(),
        );
        (*rettv).vval.v_list = list;
        if list.is_null() {
            return;
        }
    } else {
        if value_check_lock(
            tv_list_locked(list),
            b"flatten() argument\0".as_ptr() as *const ::core::ffi::c_char,
            TV_TRANSLATE as size_t,
        ) {
            return;
        }
        tv_list_ref(list);
    }
    tv_list_flatten(
        list,
        ::core::ptr::null_mut::<listitem_T>(),
        tv_list_len(list) as int64_t,
        maxdepth as int64_t,
    );
}
unsafe extern "C" fn f_flatten(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    flatten_common(argvars, rettv, false_0 != 0);
}
unsafe extern "C" fn f_flattennew(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    flatten_common(argvars, rettv, true_0 != 0);
}
unsafe extern "C" fn f_feedkeys(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if check_secure() {
        return;
    }
    let keys: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut nbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut flags: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        flags = tv_get_string_buf(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut nbuf as *mut ::core::ffi::c_char,
        );
    }
    nvim_feedkeys(cstr_as_string(keys), cstr_as_string(flags), true_0 != 0);
}
unsafe extern "C" fn f_float2nr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut f: float_T = 0.;
    if !tv_get_float_chk(argvars, &raw mut f) {
        return;
    }
    if f <= -VARNUMBER_MAX as ::core::ffi::c_double + DBL_EPSILON {
        (*rettv).vval.v_number = -VARNUMBER_MAX as varnumber_T;
    } else if f >= VARNUMBER_MAX as ::core::ffi::c_double - DBL_EPSILON {
        (*rettv).vval.v_number = VARNUMBER_MAX as varnumber_T;
    } else {
        (*rettv).vval.v_number = f as varnumber_T;
    };
}
unsafe extern "C" fn f_fmod(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut fx: float_T = 0.;
    let mut fy: float_T = 0.;
    (*rettv).v_type = VAR_FLOAT;
    if tv_get_float_chk(argvars, &raw mut fx) as ::core::ffi::c_int != 0
        && tv_get_float_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut fy,
        ) as ::core::ffi::c_int
            != 0
    {
        (*rettv).vval.v_float =
            fmod(fx as ::core::ffi::c_double, fy as ::core::ffi::c_double) as float_T;
    } else {
        (*rettv).vval.v_float = 0.0f64 as float_T;
    };
}
unsafe extern "C" fn f_fnameescape(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_string = vim_strsave_fnameescape(
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
        VSE_NONE as ::core::ffi::c_int,
    );
    (*rettv).v_type = VAR_STRING;
}
unsafe extern "C" fn f_foreground(
    mut _argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
}
unsafe extern "C" fn common_function(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut is_funcref: bool,
) {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut use_string: bool = false_0 != 0;
    let mut arg_pt: *mut partial_T = ::core::ptr::null_mut::<partial_T>();
    let mut trans_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        s = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_string;
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
        && !(*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_partial
            .is_null()
    {
        arg_pt = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_partial;
        s = partial_name(arg_pt);
    } else {
        s = tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize))
            as *mut ::core::ffi::c_char;
        use_string = true_0 != 0;
    }
    if use_string as ::core::ffi::c_int != 0 && vim_strchr(s, AUTOLOAD_CHAR).is_null()
        || is_funcref as ::core::ffi::c_int != 0
    {
        name = s;
        trans_name = save_function_name(
            &raw mut name,
            false_0 != 0,
            TFN_INT as ::core::ffi::c_int
                | TFN_QUIET as ::core::ffi::c_int
                | TFN_NO_AUTOLOAD as ::core::ffi::c_int
                | TFN_NO_DEREF as ::core::ffi::c_int,
            ::core::ptr::null_mut::<funcdict_T>(),
        );
        if *name as ::core::ffi::c_int != NUL {
            s = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
    }
    '_theend: {
        if s.is_null()
            || *s as ::core::ffi::c_int == NUL
            || use_string as ::core::ffi::c_int != 0
                && ascii_isdigit(*s as ::core::ffi::c_int) as ::core::ffi::c_int != 0
            || is_funcref as ::core::ffi::c_int != 0 && trans_name.is_null()
        {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                if use_string as ::core::ffi::c_int != 0 {
                    tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize))
                } else {
                    s as *const ::core::ffi::c_char
                },
            );
        } else if !trans_name.is_null()
            && (if is_funcref as ::core::ffi::c_int != 0 {
                find_func(trans_name).is_null() as ::core::ffi::c_int
            } else {
                !translated_function_exists(trans_name) as ::core::ffi::c_int
            }) != 0
        {
            semsg(
                gettext(b"E700: Unknown function: %s\0".as_ptr() as *const ::core::ffi::c_char),
                s,
            );
        } else {
            let mut dict_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut arg_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut list: *mut list_T = ::core::ptr::null_mut::<list_T>();
            if strncmp(
                s,
                b"s:\0".as_ptr() as *const ::core::ffi::c_char,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
                || strncmp(
                    s,
                    b"<SID>\0".as_ptr() as *const ::core::ffi::c_char,
                    5 as size_t,
                ) == 0 as ::core::ffi::c_int
            {
                name = get_scriptlocal_funcname(s);
            } else {
                name = xstrdup(s);
            }
            if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    arg_idx = 1 as ::core::ffi::c_int;
                    dict_idx = 2 as ::core::ffi::c_int;
                } else if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type
                    as ::core::ffi::c_uint
                    == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    dict_idx = 1 as ::core::ffi::c_int;
                } else {
                    arg_idx = 1 as ::core::ffi::c_int;
                }
                if dict_idx > 0 as ::core::ffi::c_int {
                    if tv_check_for_dict_arg(argvars, dict_idx) == FAIL {
                        xfree(name as *mut ::core::ffi::c_void);
                        break '_theend;
                    } else if (*argvars.offset(dict_idx as isize)).vval.v_dict.is_null() {
                        dict_idx = 0 as ::core::ffi::c_int;
                    }
                }
                if arg_idx > 0 as ::core::ffi::c_int {
                    if (*argvars.offset(arg_idx as isize)).v_type as ::core::ffi::c_uint
                        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        emsg(gettext(
                            b"E923: Second argument of function() must be a list or a dict\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ));
                        xfree(name as *mut ::core::ffi::c_void);
                        break '_theend;
                    } else {
                        list = (*argvars.offset(arg_idx as isize)).vval.v_list;
                        if tv_list_len(list) == 0 as ::core::ffi::c_int {
                            arg_idx = 0 as ::core::ffi::c_int;
                        } else if tv_list_len(list) > MAX_FUNC_ARGS as ::core::ffi::c_int {
                            emsg_funcname(&raw const e_toomanyarg as *const ::core::ffi::c_char, s);
                            xfree(name as *mut ::core::ffi::c_void);
                            break '_theend;
                        }
                    }
                }
            }
            if dict_idx > 0 as ::core::ffi::c_int
                || arg_idx > 0 as ::core::ffi::c_int
                || !arg_pt.is_null()
                || is_funcref as ::core::ffi::c_int != 0
            {
                let pt: *mut partial_T =
                    xcalloc(1 as size_t, ::core::mem::size_of::<partial_T>()) as *mut partial_T;
                if arg_idx > 0 as ::core::ffi::c_int
                    || !arg_pt.is_null() && (*arg_pt).pt_argc > 0 as ::core::ffi::c_int
                {
                    let arg_len: ::core::ffi::c_int = if arg_pt.is_null() {
                        0 as ::core::ffi::c_int
                    } else {
                        (*arg_pt).pt_argc
                    };
                    let lv_len: ::core::ffi::c_int = tv_list_len(list);
                    (*pt).pt_argc = arg_len + lv_len;
                    (*pt).pt_argv = xmalloc(
                        ::core::mem::size_of::<typval_T>().wrapping_mul((*pt).pt_argc as size_t),
                    ) as *mut typval_T;
                    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while i < arg_len {
                        tv_copy(
                            (*arg_pt).pt_argv.offset(i as isize),
                            (*pt).pt_argv.offset(i as isize),
                        );
                        i += 1;
                    }
                    if lv_len > 0 as ::core::ffi::c_int {
                        let l_: *mut list_T = list;
                        if !l_.is_null() {
                            let mut li: *mut listitem_T = (*l_).lv_first;
                            while !li.is_null() {
                                let c2rust_fresh7 = i;
                                i = i + 1;
                                tv_copy(
                                    &raw mut (*li).li_tv,
                                    (*pt).pt_argv.offset(c2rust_fresh7 as isize),
                                );
                                li = (*li).li_next;
                            }
                        }
                    }
                }
                if dict_idx > 0 as ::core::ffi::c_int {
                    (*pt).pt_dict = (*argvars.offset(dict_idx as isize)).vval.v_dict;
                    (*(*pt).pt_dict).dv_refcount += 1;
                } else if !arg_pt.is_null() {
                    (*pt).pt_dict = (*arg_pt).pt_dict;
                    (*pt).pt_auto = (*arg_pt).pt_auto;
                    if !(*pt).pt_dict.is_null() {
                        (*(*pt).pt_dict).dv_refcount += 1;
                    }
                }
                (*pt).pt_refcount = 1 as ::core::ffi::c_int;
                if !arg_pt.is_null() && !(*arg_pt).pt_func.is_null() {
                    (*pt).pt_func = (*arg_pt).pt_func;
                    func_ptr_ref((*pt).pt_func);
                    xfree(name as *mut ::core::ffi::c_void);
                } else if is_funcref {
                    (*pt).pt_func = find_func(trans_name);
                    func_ptr_ref((*pt).pt_func);
                    xfree(name as *mut ::core::ffi::c_void);
                } else {
                    (*pt).pt_name = name;
                    func_ref(name);
                }
                (*rettv).v_type = VAR_PARTIAL;
                (*rettv).vval.v_partial = pt;
            } else {
                (*rettv).v_type = VAR_FUNC;
                (*rettv).vval.v_string = name;
                func_ref(name);
            }
        }
    }
    xfree(trans_name as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn f_funcref(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    common_function(argvars, rettv, true_0 != 0);
}
unsafe extern "C" fn f_function(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    common_function(argvars, rettv, false_0 != 0);
}
unsafe extern "C" fn f_garbagecollect(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    want_garbage_collect.set(true_0 != 0);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) == 1 as varnumber_T
    {
        garbage_collect_at_exit.set(true_0 != 0);
    }
}
unsafe extern "C" fn f_get(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut tv: *mut typval_T = ::core::ptr::null_mut::<typval_T>();
    let mut what_is_dict: bool = false_0 != 0;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut error: bool = false_0 != 0;
        let mut idx: ::core::ffi::c_int = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) as ::core::ffi::c_int;
        if !error {
            (*rettv).v_type = VAR_NUMBER;
            if idx < 0 as ::core::ffi::c_int {
                idx = tv_blob_len(
                    (*argvars.offset(0 as ::core::ffi::c_int as isize))
                        .vval
                        .v_blob,
                ) + idx;
            }
            if idx < 0 as ::core::ffi::c_int
                || idx
                    >= tv_blob_len(
                        (*argvars.offset(0 as ::core::ffi::c_int as isize))
                            .vval
                            .v_blob,
                    )
            {
                (*rettv).vval.v_number = -1 as varnumber_T;
            } else {
                (*rettv).vval.v_number = tv_blob_get(
                    (*argvars.offset(0 as ::core::ffi::c_int as isize))
                        .vval
                        .v_blob,
                    idx,
                ) as varnumber_T;
                tv = rettv;
            }
        }
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_list;
        if !l.is_null() {
            let mut error_0: bool = false_0 != 0;
            let mut li: *mut listitem_T = tv_list_find(
                l,
                tv_get_number_chk(
                    argvars.offset(1 as ::core::ffi::c_int as isize),
                    &raw mut error_0,
                ) as ::core::ffi::c_int,
            );
            if !error_0 && !li.is_null() {
                tv = &raw mut (*li).li_tv;
            }
        }
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut d: *mut dict_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        if !d.is_null() {
            let mut di: *mut dictitem_T = tv_dict_find(
                d,
                tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize)),
                -1 as ptrdiff_t,
            );
            if !di.is_null() {
                tv = &raw mut (*di).di_tv;
            }
        }
    } else if tv_is_func(*argvars.offset(0 as ::core::ffi::c_int as isize)) {
        let mut pt: *mut partial_T = ::core::ptr::null_mut::<partial_T>();
        let mut fref_pt: partial_T = partial_T {
            pt_refcount: 0,
            pt_copyID: 0,
            pt_name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            pt_func: ::core::ptr::null_mut::<ufunc_T>(),
            pt_auto: false,
            pt_argc: 0,
            pt_argv: ::core::ptr::null_mut::<typval_T>(),
            pt_dict: ::core::ptr::null_mut::<dict_T>(),
        };
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            pt = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_partial;
        } else {
            memset(
                &raw mut fref_pt as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<partial_T>(),
            );
            fref_pt.pt_name = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_string;
            pt = &raw mut fref_pt;
        }
        if !pt.is_null() {
            let what: *const ::core::ffi::c_char =
                tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
            if strcmp(what, b"func\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
                || strcmp(what, b"name\0".as_ptr() as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
            {
                let mut name: *const ::core::ffi::c_char = partial_name(pt);
                (*rettv).v_type = (if *what as ::core::ffi::c_int == 'f' as ::core::ffi::c_int {
                    VAR_FUNC as ::core::ffi::c_int
                } else {
                    VAR_STRING as ::core::ffi::c_int
                }) as VarType;
                '_c2rust_label: {
                    if !name.is_null() {
                    } else {
                        __assert_fail(
                            b"name != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/eval/funcs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            1889 as ::core::ffi::c_uint,
                            b"void f_get(typval_T *, typval_T *, EvalFuncData)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                if (*rettv).v_type as ::core::ffi::c_uint
                    == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    func_ref(name as *mut ::core::ffi::c_char);
                }
                if *what as ::core::ffi::c_int == 'n' as ::core::ffi::c_int
                    && (*pt).pt_name.is_null()
                    && !(*pt).pt_func.is_null()
                {
                    name = printable_func_name((*pt).pt_func);
                }
                (*rettv).vval.v_string = xstrdup(name);
            } else if strcmp(what, b"dict\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                what_is_dict = true_0 != 0;
                if !(*pt).pt_dict.is_null() {
                    tv_dict_set_ret(rettv, (*pt).pt_dict);
                }
            } else if strcmp(what, b"args\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                (*rettv).v_type = VAR_LIST;
                tv_list_alloc_ret(rettv, (*pt).pt_argc as ptrdiff_t);
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < (*pt).pt_argc {
                    tv_list_append_tv((*rettv).vval.v_list, (*pt).pt_argv.offset(i as isize));
                    i += 1;
                }
            } else if strcmp(what, b"arity\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                let mut required: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut optional: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut varargs: bool = false_0 != 0;
                let mut name_0: *const ::core::ffi::c_char = partial_name(pt);
                get_func_arity(
                    name_0,
                    &raw mut required,
                    &raw mut optional,
                    &raw mut varargs,
                );
                (*rettv).v_type = VAR_DICT;
                tv_dict_alloc_ret(rettv);
                let mut dict: *mut dict_T = (*rettv).vval.v_dict;
                if (*pt).pt_argc >= required + optional {
                    optional = 0 as ::core::ffi::c_int;
                    required = optional;
                } else if (*pt).pt_argc > required {
                    optional -= (*pt).pt_argc - required;
                    required = 0 as ::core::ffi::c_int;
                } else {
                    required -= (*pt).pt_argc;
                }
                tv_dict_add_nr(
                    dict,
                    b"required\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                    required as varnumber_T,
                );
                tv_dict_add_nr(
                    dict,
                    b"optional\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                    optional as varnumber_T,
                );
                tv_dict_add_bool(
                    dict,
                    b"varargs\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                    varargs as BoolVarValue,
                );
            } else {
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    what,
                );
            }
            if !what_is_dict {
                return;
            }
        }
    } else {
        semsg(
            gettext(&raw const e_listdictblobarg as *const ::core::ffi::c_char),
            b"get()\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    if tv.is_null() {
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_copy(argvars.offset(2 as ::core::ffi::c_int as isize), rettv);
        }
    } else {
        tv_copy(tv, rettv);
    };
}
unsafe extern "C" fn f_getchangelist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, 2 as ptrdiff_t);
    let mut buf: *const buf_T = ::core::ptr::null::<buf_T>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        buf = curbuf.get();
    } else {
        vim_ignored.set(
            tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int,
        );
        (*emsg_off.ptr()) += 1;
        buf = tv_get_buf(argvars.offset(0 as ::core::ffi::c_int as isize), false_0);
        (*emsg_off.ptr()) -= 1;
    }
    if buf.is_null() {
        return;
    }
    let l: *mut list_T = tv_list_alloc((*buf).b_changelistlen as ptrdiff_t);
    tv_list_append_list((*rettv).vval.v_list, l);
    let mut changelistindex: ::core::ffi::c_int = 0;
    if buf == (*curwin.get()).w_buffer as *const buf_T {
        changelistindex = (*curwin.get()).w_changelistidx;
    } else {
        changelistindex = (*buf).b_changelistlen;
        let mut i: size_t = 0 as size_t;
        while i < (*buf).b_wininfo.size {
            let mut wip: *mut WinInfo = *(*buf).b_wininfo.items.offset(i as isize);
            if (*wip).wi_win == curwin.get() {
                changelistindex = (*wip).wi_changelistidx;
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
    }
    tv_list_append_number((*rettv).vval.v_list, changelistindex as varnumber_T);
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < (*buf).b_changelistlen {
        if (*buf).b_changelist[i_0 as usize].mark.lnum != 0 as linenr_T {
            let d: *mut dict_T = tv_dict_alloc();
            tv_list_append_dict(l, d);
            tv_dict_add_nr(
                d,
                b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                (*buf).b_changelist[i_0 as usize].mark.lnum as varnumber_T,
            );
            tv_dict_add_nr(
                d,
                b"col\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
                (*buf).b_changelist[i_0 as usize].mark.col as varnumber_T,
            );
            tv_dict_add_nr(
                d,
                b"coladd\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
                (*buf).b_changelist[i_0 as usize].mark.coladd as varnumber_T,
            );
        }
        i_0 += 1;
    }
}
unsafe extern "C" fn getpos_both(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut getcurpos: bool,
    mut charcol: bool,
) {
    let mut fp: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut wp: *mut win_T = curwin.get();
    let mut fnum: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if getcurpos {
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            wp = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
            if !wp.is_null() {
                fp = &raw mut (*wp).w_cursor;
            }
        } else {
            fp = &raw mut (*curwin.get()).w_cursor;
        }
        if !fp.is_null() && charcol as ::core::ffi::c_int != 0 {
            pos = *fp;
            pos.col =
                buf_byteidx_to_charidx((*wp).w_buffer, pos.lnum, pos.col as ::core::ffi::c_int)
                    as colnr_T;
            fp = &raw mut pos;
        }
    } else {
        fp = var2fpos(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            true_0 != 0,
            &raw mut fnum,
            charcol,
            curwin.get(),
        );
    }
    let l: *mut list_T = tv_list_alloc_ret(
        rettv,
        (4 as ::core::ffi::c_int + getcurpos as ::core::ffi::c_int) as ptrdiff_t,
    );
    tv_list_append_number(
        l,
        if fnum != -1 as ::core::ffi::c_int {
            fnum as varnumber_T
        } else {
            0 as ::core::ffi::c_int as varnumber_T
        },
    );
    tv_list_append_number(
        l,
        if !fp.is_null() {
            (*fp).lnum as varnumber_T
        } else {
            0 as ::core::ffi::c_int as varnumber_T
        },
    );
    tv_list_append_number(
        l,
        if !fp.is_null() {
            (if (*fp).col == MAXCOL as ::core::ffi::c_int {
                MAXCOL as ::core::ffi::c_int
            } else {
                (*fp).col as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            }) as varnumber_T
        } else {
            0 as ::core::ffi::c_int as varnumber_T
        },
    );
    tv_list_append_number(
        l,
        if !fp.is_null() {
            (*fp).coladd as varnumber_T
        } else {
            0 as ::core::ffi::c_int as varnumber_T
        },
    );
    if getcurpos {
        let save_set_curswant: bool = (*curwin.get()).w_set_curswant != 0;
        let save_curswant: colnr_T = (*curwin.get()).w_curswant;
        let save_virtcol: colnr_T = (*curwin.get()).w_virtcol;
        if wp == curwin.get() {
            update_curswant();
        }
        tv_list_append_number(
            l,
            if wp.is_null() {
                0 as varnumber_T
            } else if (*wp).w_curswant == MAXCOL as ::core::ffi::c_int {
                MAXCOL as ::core::ffi::c_int as varnumber_T
            } else {
                (*wp).w_curswant as varnumber_T + 1 as varnumber_T
            },
        );
        if wp == curwin.get() && save_set_curswant as ::core::ffi::c_int != 0 {
            (*curwin.get()).w_set_curswant = save_set_curswant as ::core::ffi::c_int;
            (*curwin.get()).w_curswant = save_curswant;
            (*curwin.get()).w_virtcol = save_virtcol;
            (*curwin.get()).w_valid &= !VALID_VIRTCOL;
        }
    }
}
unsafe extern "C" fn f_getcharpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    getpos_both(argvars, rettv, false_0 != 0, true_0 != 0);
}
unsafe extern "C" fn f_getcharsearch(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_dict_alloc_ret(rettv);
    let mut dict: *mut dict_T = (*rettv).vval.v_dict;
    tv_dict_add_str(
        dict,
        b"char\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        last_csearch(),
    );
    tv_dict_add_nr(
        dict,
        b"forward\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        last_csearch_forward() as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"until\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        last_csearch_until() as varnumber_T,
    );
}
unsafe extern "C" fn f_getfontname(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn f_getjumplist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    let wp: *mut win_T = find_tabwin(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        argvars.offset(1 as ::core::ffi::c_int as isize),
    );
    if wp.is_null() {
        return;
    }
    cleanup_jumplist(wp, true_0 != 0);
    let l: *mut list_T = tv_list_alloc((*wp).w_jumplistlen as ptrdiff_t);
    tv_list_append_list((*rettv).vval.v_list, l);
    tv_list_append_number((*rettv).vval.v_list, (*wp).w_jumplistidx as varnumber_T);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*wp).w_jumplistlen {
        if (*wp).w_jumplist[i as usize].fmark.mark.lnum != 0 as linenr_T {
            let d: *mut dict_T = tv_dict_alloc();
            tv_list_append_dict(l, d);
            tv_dict_add_nr(
                d,
                b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                (*wp).w_jumplist[i as usize].fmark.mark.lnum as varnumber_T,
            );
            tv_dict_add_nr(
                d,
                b"col\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
                (*wp).w_jumplist[i as usize].fmark.mark.col as varnumber_T,
            );
            tv_dict_add_nr(
                d,
                b"coladd\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
                (*wp).w_jumplist[i as usize].fmark.mark.coladd as varnumber_T,
            );
            tv_dict_add_nr(
                d,
                b"bufnr\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                (*wp).w_jumplist[i as usize].fmark.fnum as varnumber_T,
            );
            if !(*wp).w_jumplist[i as usize].fname.is_null() {
                tv_dict_add_str(
                    d,
                    b"filename\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                    (*wp).w_jumplist[i as usize].fname,
                );
            }
        }
        i += 1;
    }
}
unsafe extern "C" fn f_getmarklist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        get_global_marks((*rettv).vval.v_list);
        return;
    }
    let mut buf: *mut buf_T = tv_get_buf(argvars.offset(0 as ::core::ffi::c_int as isize), false_0);
    if buf.is_null() {
        return;
    }
    get_buf_local_marks(buf, (*rettv).vval.v_list);
}
unsafe extern "C" fn f_getpid(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = os_get_pid() as varnumber_T;
}
unsafe extern "C" fn f_getcurpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    getpos_both(argvars, rettv, true_0 != 0, false_0 != 0);
}
unsafe extern "C" fn f_getcursorcharpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    getpos_both(argvars, rettv, true_0 != 0, true_0 != 0);
}
unsafe extern "C" fn f_getpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    getpos_both(argvars, rettv, false_0 != 0, false_0 != 0);
}
unsafe extern "C" fn block_def2str(mut bd: *mut block_def) -> String_0 {
    let mut size: size_t = ((*bd).startspaces as size_t)
        .wrapping_add((*bd).endspaces as size_t)
        .wrapping_add((*bd).textlen as size_t);
    let mut ret: String_0 = String_0 {
        data: xmalloc(size.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char,
        size: 0,
    };
    memset(
        ret.data as *mut ::core::ffi::c_void,
        ' ' as ::core::ffi::c_int,
        (*bd).startspaces as size_t,
    );
    ret.size = ret.size.wrapping_add((*bd).startspaces as size_t);
    memmove(
        ret.data.offset(ret.size as isize) as *mut ::core::ffi::c_void,
        (*bd).textstart as *const ::core::ffi::c_void,
        (*bd).textlen as size_t,
    );
    ret.size = ret.size.wrapping_add((*bd).textlen as size_t);
    memset(
        ret.data.offset(ret.size as isize) as *mut ::core::ffi::c_void,
        ' ' as ::core::ffi::c_int,
        (*bd).endspaces as size_t,
    );
    ret.size = ret.size.wrapping_add((*bd).endspaces as size_t);
    *ret.data.offset(ret.size as isize) = NUL as ::core::ffi::c_char;
    return ret;
}
unsafe extern "C" fn getregionpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut p1: *mut pos_T,
    mut p2: *mut pos_T,
    inclusive: *mut bool,
    mut region_type: *mut MotionType,
    mut oap: *mut oparg_T,
) -> ::core::ffi::c_int {
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    if tv_check_for_list_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || tv_check_for_list_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
        || tv_check_for_opt_dict_arg(argvars, 2 as ::core::ffi::c_int) == FAIL
    {
        return FAIL;
    }
    let mut fnum1: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut fnum2: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if list2fpos(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        p1,
        &raw mut fnum1,
        ::core::ptr::null_mut::<colnr_T>(),
        false_0 != 0,
    ) != OK
        || list2fpos(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            p2,
            &raw mut fnum2,
            ::core::ptr::null_mut::<colnr_T>(),
            false_0 != 0,
        ) != OK
        || fnum1 != fnum2
    {
        return FAIL;
    }
    let mut is_select_exclusive: bool = false;
    let mut type_0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut default_type: [::core::ffi::c_char; 2] =
        ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"v\0");
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        is_select_exclusive = tv_dict_get_bool(
            (*argvars.offset(2 as ::core::ffi::c_int as isize))
                .vval
                .v_dict,
            b"exclusive\0".as_ptr() as *const ::core::ffi::c_char,
            (*p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int) as ::core::ffi::c_int,
        ) != 0;
        type_0 = tv_dict_get_string(
            (*argvars.offset(2 as ::core::ffi::c_int as isize))
                .vval
                .v_dict,
            b"type\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        if type_0.is_null() {
            type_0 = &raw mut default_type as *mut ::core::ffi::c_char;
        }
    } else {
        is_select_exclusive = *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int;
        type_0 = &raw mut default_type as *mut ::core::ffi::c_char;
    }
    let mut block_width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if *type_0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'v' as ::core::ffi::c_int
        && *type_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        *region_type = kMTCharWise;
    } else if *type_0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'V' as ::core::ffi::c_int
        && *type_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        *region_type = kMTLineWise;
    } else if *type_0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == Ctrl_V {
        let mut p: *mut ::core::ffi::c_char = type_0.offset(1 as ::core::ffi::c_int as isize);
        if *p as ::core::ffi::c_int != NUL && {
            block_width = getdigits_int(&raw mut p, false_0 != 0, 0 as ::core::ffi::c_int);
            block_width <= 0 as ::core::ffi::c_int || *p as ::core::ffi::c_int != NUL
        } {
            semsg(
                gettext(&raw const e_invargNval as *const ::core::ffi::c_char),
                b"type\0".as_ptr() as *const ::core::ffi::c_char,
                type_0,
            );
            return FAIL;
        }
        *region_type = kMTBlockWise;
    } else {
        semsg(
            gettext(&raw const e_invargNval as *const ::core::ffi::c_char),
            b"type\0".as_ptr() as *const ::core::ffi::c_char,
            type_0,
        );
        return FAIL;
    }
    let mut findbuf: *mut buf_T = if fnum1 != 0 as ::core::ffi::c_int {
        buflist_findnr(fnum1)
    } else {
        curbuf.get()
    };
    if findbuf.is_null() || (*findbuf).b_ml.ml_mfp.is_null() {
        emsg(gettext(
            &raw const e_buffer_is_not_loaded as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    if (*p1).lnum < 1 as linenr_T || (*p1).lnum > (*findbuf).b_ml.ml_line_count {
        semsg(
            gettext(&raw const e_invalid_line_number_nr as *const ::core::ffi::c_char),
            (*p1).lnum,
        );
        return FAIL;
    }
    if (*p1).col == MAXCOL as ::core::ffi::c_int {
        (*p1).col = (ml_get_buf_len(findbuf, (*p1).lnum) + 1 as ::core::ffi::c_int) as colnr_T;
    } else if (*p1).col < 1 as ::core::ffi::c_int
        || (*p1).col > ml_get_buf_len(findbuf, (*p1).lnum) + 1 as ::core::ffi::c_int
    {
        semsg(
            gettext(&raw const e_invalid_column_number_nr as *const ::core::ffi::c_char),
            (*p1).col,
        );
        return FAIL;
    }
    if (*p2).lnum < 1 as linenr_T || (*p2).lnum > (*findbuf).b_ml.ml_line_count {
        semsg(
            gettext(&raw const e_invalid_line_number_nr as *const ::core::ffi::c_char),
            (*p2).lnum,
        );
        return FAIL;
    }
    if (*p2).col == MAXCOL as ::core::ffi::c_int {
        (*p2).col = (ml_get_buf_len(findbuf, (*p2).lnum) + 1 as ::core::ffi::c_int) as colnr_T;
    } else if (*p2).col < 1 as ::core::ffi::c_int
        || (*p2).col > ml_get_buf_len(findbuf, (*p2).lnum) + 1 as ::core::ffi::c_int
    {
        semsg(
            gettext(&raw const e_invalid_column_number_nr as *const ::core::ffi::c_char),
            (*p2).col,
        );
        return FAIL;
    }
    curbuf.set(findbuf);
    (*curwin.get()).w_buffer = curbuf.get();
    virtual_op.set(virtual_active(curwin.get()) as TriState);
    (*p1).col -= 1;
    (*p2).col -= 1;
    if !lt(*p1, *p2) {
        let mut p_0: pos_T = *p1;
        *p1 = *p2;
        *p2 = p_0;
    }
    if *region_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int {
        if is_select_exclusive as ::core::ffi::c_int != 0 && !equalpos(*p1, *p2) {
            *inclusive = !unadjust_for_sel_inner(p2);
        }
        if *inclusive as ::core::ffi::c_int != 0
            && virtual_op.get() as u64 == 0
            && *ml_get_pos(p2) as ::core::ffi::c_int == NUL
        {
            *inclusive = false_0 != 0;
        }
    } else if *region_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
        let mut sc1: colnr_T = 0;
        let mut ec1: colnr_T = 0;
        let mut sc2: colnr_T = 0;
        let mut ec2: colnr_T = 0;
        let lbr_saved: bool = reset_lbr();
        getvvcol(
            curwin.get(),
            p1,
            &raw mut sc1,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut ec1,
        );
        getvvcol(
            curwin.get(),
            p2,
            &raw mut sc2,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut ec2,
        );
        restore_lbr(lbr_saved);
        (*oap).motion_type = kMTBlockWise;
        (*oap).inclusive = true_0 != 0;
        (*oap).op_type = OP_NOP as ::core::ffi::c_int;
        (*oap).start = *p1;
        (*oap).end = *p2;
        (*oap).start_vcol = if sc1 < sc2 { sc1 } else { sc2 };
        if block_width > 0 as ::core::ffi::c_int {
            (*oap).end_vcol = ((*oap).start_vcol as ::core::ffi::c_int + block_width
                - 1 as ::core::ffi::c_int) as colnr_T;
        } else if is_select_exclusive as ::core::ffi::c_int != 0
            && ec1 < sc2
            && (0 as ::core::ffi::c_int) < sc2
            && ec2 > ec1
        {
            (*oap).end_vcol = (sc2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as colnr_T;
        } else {
            (*oap).end_vcol = if ec1 > ec2 { ec1 } else { ec2 };
        }
    }
    let mut l: ::core::ffi::c_int = utfc_ptr2len(ml_get_pos(p2));
    if l > 1 as ::core::ffi::c_int {
        (*p2).col += l - 1 as ::core::ffi::c_int;
    }
    return OK;
}
unsafe extern "C" fn f_getregion(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let save_curbuf: *mut buf_T = curbuf.get();
    let save_virtual: TriState = virtual_op.get();
    let mut p1: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut p2: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut inclusive: bool = true_0 != 0;
    let mut region_type: MotionType = kMTUnknown;
    let mut oa: oparg_T = oparg_T {
        op_type: 0,
        regname: 0,
        motion_type: kMTCharWise,
        motion_force: 0,
        use_reg_one: false,
        inclusive: false,
        end_adjusted: false,
        start: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        end: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        cursor_start: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        line_count: 0,
        empty: false,
        is_VIsual: false,
        start_vcol: 0,
        end_vcol: 0,
        prev_opcount: 0,
        prev_count0: 0,
        excl_tr_ws: false,
    };
    if getregionpos(
        argvars,
        rettv,
        &raw mut p1,
        &raw mut p2,
        &raw mut inclusive,
        &raw mut region_type,
        &raw mut oa,
    ) == FAIL
    {
        return;
    }
    let mut lnum: linenr_T = p1.lnum;
    while lnum <= p2.lnum {
        let mut akt: String_0 = STRING_INIT;
        if region_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
            let mut bd: block_def = block_def {
                startspaces: 0,
                endspaces: 0,
                textlen: 0,
                textstart: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                textcol: 0,
                start_vcol: 0,
                end_vcol: 0,
                is_short: 0,
                is_MAX: 0,
                is_oneChar: 0,
                pre_whitesp: 0,
                pre_whitesp_c: 0,
                end_char_vcols: 0,
                start_char_vcols: 0,
            };
            block_prep(&raw mut oa, &raw mut bd, lnum, false_0 != 0);
            akt = block_def2str(&raw mut bd);
        } else if region_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
            || p1.lnum < lnum && lnum < p2.lnum
        {
            akt = cbuf_to_string(ml_get(lnum), ml_get_len(lnum) as size_t);
        } else {
            let mut bd_0: block_def = block_def {
                startspaces: 0,
                endspaces: 0,
                textlen: 0,
                textstart: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                textcol: 0,
                start_vcol: 0,
                end_vcol: 0,
                is_short: 0,
                is_MAX: 0,
                is_oneChar: 0,
                pre_whitesp: 0,
                pre_whitesp_c: 0,
                end_char_vcols: 0,
                start_char_vcols: 0,
            };
            charwise_block_prep(p1, p2, &raw mut bd_0, lnum, inclusive);
            akt = block_def2str(&raw mut bd_0);
        }
        '_c2rust_label: {
            if !akt.data.is_null() {
            } else {
                __assert_fail(
                    b"akt.data != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/funcs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2344 as ::core::ffi::c_uint,
                    b"void f_getregion(typval_T *, typval_T *, EvalFuncData)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        tv_list_append_allocated_string((*rettv).vval.v_list, akt.data);
        lnum += 1;
    }
    curbuf.set(save_curbuf);
    (*curwin.get()).w_buffer = curbuf.get();
    virtual_op.set(save_virtual);
}
unsafe extern "C" fn add_regionpos_range(mut rettv: *mut typval_T, mut p1: pos_T, mut p2: pos_T) {
    let mut l1: *mut list_T = tv_list_alloc(2 as ptrdiff_t);
    tv_list_append_list((*rettv).vval.v_list, l1);
    let mut l2: *mut list_T = tv_list_alloc(4 as ptrdiff_t);
    tv_list_append_list(l1, l2);
    let mut l3: *mut list_T = tv_list_alloc(4 as ptrdiff_t);
    tv_list_append_list(l1, l3);
    tv_list_append_number(l2, (*curbuf.get()).handle as varnumber_T);
    tv_list_append_number(l2, p1.lnum as varnumber_T);
    tv_list_append_number(l2, p1.col as varnumber_T);
    tv_list_append_number(l2, p1.coladd as varnumber_T);
    tv_list_append_number(l3, (*curbuf.get()).handle as varnumber_T);
    tv_list_append_number(l3, p2.lnum as varnumber_T);
    tv_list_append_number(l3, p2.col as varnumber_T);
    tv_list_append_number(l3, p2.coladd as varnumber_T);
}
unsafe extern "C" fn f_getregionpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let save_curbuf: *mut buf_T = curbuf.get();
    let save_virtual: TriState = virtual_op.get();
    let mut p1: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut p2: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut inclusive: bool = true_0 != 0;
    let mut region_type: MotionType = kMTUnknown;
    let mut allow_eol: bool = false_0 != 0;
    let mut oa: oparg_T = oparg_T {
        op_type: 0,
        regname: 0,
        motion_type: kMTCharWise,
        motion_force: 0,
        use_reg_one: false,
        inclusive: false,
        end_adjusted: false,
        start: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        end: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        cursor_start: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        line_count: 0,
        empty: false,
        is_VIsual: false,
        start_vcol: 0,
        end_vcol: 0,
        prev_opcount: 0,
        prev_count0: 0,
        excl_tr_ws: false,
    };
    if getregionpos(
        argvars,
        rettv,
        &raw mut p1,
        &raw mut p2,
        &raw mut inclusive,
        &raw mut region_type,
        &raw mut oa,
    ) == FAIL
    {
        return;
    }
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        allow_eol = tv_dict_get_bool(
            (*argvars.offset(2 as ::core::ffi::c_int as isize))
                .vval
                .v_dict,
            b"eol\0".as_ptr() as *const ::core::ffi::c_char,
            false_0,
        ) != 0;
    }
    let mut lnum: linenr_T = p1.lnum;
    while lnum <= p2.lnum {
        let mut ret_p1: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut ret_p2: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut line: *mut ::core::ffi::c_char = ml_get(lnum);
        let mut line_len: colnr_T = ml_get_len(lnum);
        if region_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
            ret_p1.col = 1 as ::core::ffi::c_int as colnr_T;
            ret_p1.coladd = 0 as ::core::ffi::c_int as colnr_T;
            ret_p2.col = MAXCOL as ::core::ffi::c_int as colnr_T;
            ret_p2.coladd = 0 as ::core::ffi::c_int as colnr_T;
        } else {
            let mut bd: block_def = block_def {
                startspaces: 0,
                endspaces: 0,
                textlen: 0,
                textstart: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                textcol: 0,
                start_vcol: 0,
                end_vcol: 0,
                is_short: 0,
                is_MAX: 0,
                is_oneChar: 0,
                pre_whitesp: 0,
                pre_whitesp_c: 0,
                end_char_vcols: 0,
                start_char_vcols: 0,
            };
            if region_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
                block_prep(&raw mut oa, &raw mut bd, lnum, false_0 != 0);
            } else {
                charwise_block_prep(p1, p2, &raw mut bd, lnum, inclusive);
            }
            if bd.is_oneChar != 0 {
                if region_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
                    ret_p1.col = (mb_prevptr(line, bd.textstart).offset_from(line)
                        as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int) as colnr_T;
                    ret_p1.coladd = bd.start_char_vcols - (bd.start_vcol - oa.start_vcol);
                } else {
                    ret_p1.col =
                        (p1.col as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as colnr_T;
                    ret_p1.coladd = p1.coladd;
                }
            } else if region_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int
                && oa.start_vcol > bd.start_vcol
            {
                ret_p1.col = MAXCOL as ::core::ffi::c_int as colnr_T;
                ret_p1.coladd = oa.start_vcol - bd.start_vcol;
                bd.is_oneChar = true_0;
            } else if bd.startspaces > 0 as ::core::ffi::c_int {
                ret_p1.col = (mb_prevptr(line, bd.textstart).offset_from(line)
                    as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int) as colnr_T;
                ret_p1.coladd =
                    (bd.start_char_vcols as ::core::ffi::c_int - bd.startspaces) as colnr_T;
            } else {
                ret_p1.col =
                    (bd.textcol as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as colnr_T;
                ret_p1.coladd = 0 as ::core::ffi::c_int as colnr_T;
            }
            if bd.is_oneChar != 0 {
                ret_p2.col = ret_p1.col;
                ret_p2.coladd = (ret_p1.coladd as ::core::ffi::c_int
                    + bd.startspaces
                    + bd.endspaces) as colnr_T;
            } else if bd.endspaces > 0 as ::core::ffi::c_int {
                ret_p2.col = (bd.textcol as ::core::ffi::c_int
                    + bd.textlen
                    + 1 as ::core::ffi::c_int) as colnr_T;
                ret_p2.coladd = bd.endspaces as colnr_T;
            } else {
                ret_p2.col = (bd.textcol as ::core::ffi::c_int + bd.textlen) as colnr_T;
                ret_p2.coladd = 0 as ::core::ffi::c_int as colnr_T;
            }
        }
        if !allow_eol && ret_p1.col > line_len {
            ret_p1.col = 0 as ::core::ffi::c_int as colnr_T;
            ret_p1.coladd = 0 as ::core::ffi::c_int as colnr_T;
        } else if ret_p1.col > line_len as ::core::ffi::c_int + 1 as ::core::ffi::c_int {
            ret_p1.col = (line_len as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as colnr_T;
        }
        if !allow_eol && ret_p2.col > line_len {
            ret_p2.col = (if ret_p1.col == 0 as ::core::ffi::c_int {
                0 as ::core::ffi::c_int
            } else {
                line_len as ::core::ffi::c_int
            }) as colnr_T;
            ret_p2.coladd = 0 as ::core::ffi::c_int as colnr_T;
        } else if ret_p2.col > line_len as ::core::ffi::c_int + 1 as ::core::ffi::c_int {
            ret_p2.col = (line_len as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as colnr_T;
        }
        ret_p1.lnum = lnum;
        ret_p2.lnum = lnum;
        add_regionpos_range(rettv, ret_p1, ret_p2);
        lnum += 1;
    }
    curbuf.set(save_curbuf);
    (*curwin.get()).w_buffer = curbuf.get();
    virtual_op.set(save_virtual);
}
unsafe extern "C" fn getreg_get_regname(mut argvars: *mut typval_T) -> ::core::ffi::c_int {
    let mut strregname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        strregname = tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
        if strregname.is_null() {
            return 0 as ::core::ffi::c_int;
        }
    } else {
        strregname = get_vim_var_str(VV_REG);
    }
    return if *strregname as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        '"' as ::core::ffi::c_int
    } else {
        *strregname as uint8_t as ::core::ffi::c_int
    };
}
unsafe extern "C" fn f_getreg(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut arg2: ::core::ffi::c_int = false_0;
    let mut return_list: bool = false_0 != 0;
    let mut regname: ::core::ffi::c_int = getreg_get_regname(argvars);
    if regname == 0 as ::core::ffi::c_int {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut error: bool = false_0 != 0;
        arg2 = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) as ::core::ffi::c_int;
        if !error
            && (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return_list = tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) != 0;
        }
        if error {
            return;
        }
    }
    if return_list {
        (*rettv).v_type = VAR_LIST;
        (*rettv).vval.v_list = get_reg_contents(
            regname,
            (if arg2 != 0 {
                kGRegExprSrc as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) | kGRegList as ::core::ffi::c_int,
        ) as *mut list_T;
        if (*rettv).vval.v_list.is_null() {
            (*rettv).vval.v_list = tv_list_alloc(0 as ptrdiff_t);
        }
        tv_list_ref((*rettv).vval.v_list);
    } else {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = get_reg_contents(
            regname,
            if arg2 != 0 {
                kGRegExprSrc as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
        ) as *mut ::core::ffi::c_char;
    };
}
unsafe extern "C" fn f_getregtype(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut regname: ::core::ffi::c_int = getreg_get_regname(argvars);
    if regname == 0 as ::core::ffi::c_int {
        return;
    }
    let mut reglen: colnr_T = 0 as colnr_T;
    let mut buf: [::core::ffi::c_char; 67] = [0; 67];
    let mut reg_type: MotionType = get_reg_type(regname, &raw mut reglen);
    format_reg_type(
        reg_type,
        reglen,
        &raw mut buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 67]>()
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[::core::ffi::c_char; 67]>()
                    .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as size_t,
            ),
    );
    (*rettv).vval.v_string = xstrdup(&raw mut buf as *mut ::core::ffi::c_char);
}
unsafe extern "C" fn f_gettagstack(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut wp: *mut win_T = curwin.get();
    tv_dict_alloc_ret(rettv);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        wp = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
        if wp.is_null() {
            return;
        }
    }
    get_tagstack(wp, (*rettv).vval.v_dict);
}
unsafe extern "C" fn dummy_timer_due_cb(
    mut tw: *mut TimeWatcher,
    mut _data: *mut ::core::ffi::c_void,
) {
    if (*main_loop.ptr()).closing {
        time_watcher_stop(tw);
        time_watcher_close(
            tw,
            Some(
                dummy_timer_close_cb
                    as unsafe extern "C" fn(*mut TimeWatcher, *mut ::core::ffi::c_void) -> (),
            ),
        );
    }
}
unsafe extern "C" fn dummy_timer_close_cb(
    mut tw: *mut TimeWatcher,
    mut _data: *mut ::core::ffi::c_void,
) {
    xfree(tw as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn f_wait(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = -1 as varnumber_T;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invargval as *const ::core::ffi::c_char),
            b"1\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(2 as ::core::ffi::c_int as isize))
                .vval
                .v_number
                <= 0 as varnumber_T
    {
        semsg(
            gettext(&raw const e_invargval as *const ::core::ffi::c_char),
            b"3\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut timeout: ::core::ffi::c_int = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_number as ::core::ffi::c_int;
    let mut expr: typval_T = *argvars.offset(1 as ::core::ffi::c_int as isize);
    let mut interval: ::core::ffi::c_int = if (*argvars.offset(2 as ::core::ffi::c_int as isize))
        .v_type as ::core::ffi::c_uint
        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*argvars.offset(2 as ::core::ffi::c_int as isize))
            .vval
            .v_number as ::core::ffi::c_int
    } else {
        200 as ::core::ffi::c_int
    };
    let mut tw: *mut TimeWatcher =
        xmalloc(::core::mem::size_of::<TimeWatcher>()) as *mut TimeWatcher;
    time_watcher_init(main_loop.ptr(), tw, NULL_0);
    (*tw).events = ::core::ptr::null_mut::<MultiQueue>();
    time_watcher_start(
        tw,
        Some(
            dummy_timer_due_cb
                as unsafe extern "C" fn(*mut TimeWatcher, *mut ::core::ffi::c_void) -> (),
        ),
        interval as uint64_t,
        interval as uint64_t,
    );
    let mut argv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut exprval: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut error: bool = false_0 != 0;
    let called_emsg_before: ::core::ffi::c_int = called_emsg.get();
    ui_flush();
    let mut remaining: int64_t = timeout as int64_t;
    let mut before: uint64_t = if remaining > 0 as int64_t {
        os_hrtime()
    } else {
        0 as uint64_t
    };
    while !(eval_expr_typval(
        &raw mut expr,
        false,
        &raw mut argv,
        0 as ::core::ffi::c_int,
        &raw mut exprval,
    ) != 1 as ::core::ffi::c_int
        || tv_get_number_chk(&raw mut exprval, &raw mut error) != 0
        || called_emsg.get() > called_emsg_before
        || error as ::core::ffi::c_int != 0
        || got_int.get() as ::core::ffi::c_int != 0)
    {
        if !(*main_loop.ptr()).events.is_null() && !multiqueue_empty((*main_loop.ptr()).events) {
            multiqueue_process_events((*main_loop.ptr()).events);
        } else {
            loop_poll_events(main_loop.ptr(), remaining);
        }
        if remaining == 0 as int64_t {
            break;
        }
        if remaining <= 0 as int64_t {
            continue;
        }
        let mut now: uint64_t = os_hrtime();
        remaining -= now.wrapping_sub(before).wrapping_div(1000000 as uint64_t) as int64_t;
        before = now;
        if remaining <= 0 as int64_t {
            break;
        }
    }
    if called_emsg.get() > called_emsg_before || error as ::core::ffi::c_int != 0 {
        (*rettv).vval.v_number = -3 as varnumber_T;
    } else if got_int.get() {
        got_int.set(false_0 != 0);
        vgetc();
        (*rettv).vval.v_number = -2 as varnumber_T;
    } else if tv_get_number_chk(&raw mut exprval, &raw mut error) != 0 {
        (*rettv).vval.v_number = 0 as varnumber_T;
    }
    time_watcher_stop(tw);
    time_watcher_close(
        tw,
        Some(
            dummy_timer_close_cb
                as unsafe extern "C" fn(*mut TimeWatcher, *mut ::core::ffi::c_void) -> (),
        ),
    );
}
unsafe extern "C" fn f_gettext(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if tv_check_for_nonempty_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
        return;
    }
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = xstrdup(gettext(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_string,
    ));
}
unsafe extern "C" fn f_has(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    static has_list: GlobalCell<[*const ::core::ffi::c_char; 90]> = GlobalCell::new([
        b"linux\0".as_ptr() as *const ::core::ffi::c_char,
        b"unix\0".as_ptr() as *const ::core::ffi::c_char,
        b"fname_case\0".as_ptr() as *const ::core::ffi::c_char,
        b"acl\0".as_ptr() as *const ::core::ffi::c_char,
        b"autochdir\0".as_ptr() as *const ::core::ffi::c_char,
        b"arabic\0".as_ptr() as *const ::core::ffi::c_char,
        b"autocmd\0".as_ptr() as *const ::core::ffi::c_char,
        b"browsefilter\0".as_ptr() as *const ::core::ffi::c_char,
        b"byte_offset\0".as_ptr() as *const ::core::ffi::c_char,
        b"cindent\0".as_ptr() as *const ::core::ffi::c_char,
        b"cmdline_compl\0".as_ptr() as *const ::core::ffi::c_char,
        b"cmdline_hist\0".as_ptr() as *const ::core::ffi::c_char,
        b"cmdwin\0".as_ptr() as *const ::core::ffi::c_char,
        b"comments\0".as_ptr() as *const ::core::ffi::c_char,
        b"conceal\0".as_ptr() as *const ::core::ffi::c_char,
        b"cursorbind\0".as_ptr() as *const ::core::ffi::c_char,
        b"cursorshape\0".as_ptr() as *const ::core::ffi::c_char,
        b"dialog_con\0".as_ptr() as *const ::core::ffi::c_char,
        b"diff\0".as_ptr() as *const ::core::ffi::c_char,
        b"digraphs\0".as_ptr() as *const ::core::ffi::c_char,
        b"eval\0".as_ptr() as *const ::core::ffi::c_char,
        b"ex_extra\0".as_ptr() as *const ::core::ffi::c_char,
        b"extra_search\0".as_ptr() as *const ::core::ffi::c_char,
        b"file_in_path\0".as_ptr() as *const ::core::ffi::c_char,
        b"filterpipe\0".as_ptr() as *const ::core::ffi::c_char,
        b"find_in_path\0".as_ptr() as *const ::core::ffi::c_char,
        b"float\0".as_ptr() as *const ::core::ffi::c_char,
        b"folding\0".as_ptr() as *const ::core::ffi::c_char,
        b"fork\0".as_ptr() as *const ::core::ffi::c_char,
        b"gettext\0".as_ptr() as *const ::core::ffi::c_char,
        b"iconv\0".as_ptr() as *const ::core::ffi::c_char,
        b"insert_expand\0".as_ptr() as *const ::core::ffi::c_char,
        b"jumplist\0".as_ptr() as *const ::core::ffi::c_char,
        b"keymap\0".as_ptr() as *const ::core::ffi::c_char,
        b"lambda\0".as_ptr() as *const ::core::ffi::c_char,
        b"langmap\0".as_ptr() as *const ::core::ffi::c_char,
        b"libcall\0".as_ptr() as *const ::core::ffi::c_char,
        b"linebreak\0".as_ptr() as *const ::core::ffi::c_char,
        b"lispindent\0".as_ptr() as *const ::core::ffi::c_char,
        b"listcmds\0".as_ptr() as *const ::core::ffi::c_char,
        b"localmap\0".as_ptr() as *const ::core::ffi::c_char,
        b"menu\0".as_ptr() as *const ::core::ffi::c_char,
        b"mksession\0".as_ptr() as *const ::core::ffi::c_char,
        b"modify_fname\0".as_ptr() as *const ::core::ffi::c_char,
        b"mouse\0".as_ptr() as *const ::core::ffi::c_char,
        b"multi_byte\0".as_ptr() as *const ::core::ffi::c_char,
        b"multi_lang\0".as_ptr() as *const ::core::ffi::c_char,
        b"nanotime\0".as_ptr() as *const ::core::ffi::c_char,
        b"num64\0".as_ptr() as *const ::core::ffi::c_char,
        b"packages\0".as_ptr() as *const ::core::ffi::c_char,
        b"path_extra\0".as_ptr() as *const ::core::ffi::c_char,
        b"persistent_undo\0".as_ptr() as *const ::core::ffi::c_char,
        b"profile\0".as_ptr() as *const ::core::ffi::c_char,
        b"reltime\0".as_ptr() as *const ::core::ffi::c_char,
        b"quickfix\0".as_ptr() as *const ::core::ffi::c_char,
        b"rightleft\0".as_ptr() as *const ::core::ffi::c_char,
        b"scrollbind\0".as_ptr() as *const ::core::ffi::c_char,
        b"showcmd\0".as_ptr() as *const ::core::ffi::c_char,
        b"cmdline_info\0".as_ptr() as *const ::core::ffi::c_char,
        b"shada\0".as_ptr() as *const ::core::ffi::c_char,
        b"signs\0".as_ptr() as *const ::core::ffi::c_char,
        b"smartindent\0".as_ptr() as *const ::core::ffi::c_char,
        b"startuptime\0".as_ptr() as *const ::core::ffi::c_char,
        b"statusline\0".as_ptr() as *const ::core::ffi::c_char,
        b"spell\0".as_ptr() as *const ::core::ffi::c_char,
        b"syntax\0".as_ptr() as *const ::core::ffi::c_char,
        b"tablineat\0".as_ptr() as *const ::core::ffi::c_char,
        b"tag_binary\0".as_ptr() as *const ::core::ffi::c_char,
        b"termguicolors\0".as_ptr() as *const ::core::ffi::c_char,
        b"terminfo\0".as_ptr() as *const ::core::ffi::c_char,
        b"termresponse\0".as_ptr() as *const ::core::ffi::c_char,
        b"textobjects\0".as_ptr() as *const ::core::ffi::c_char,
        b"timers\0".as_ptr() as *const ::core::ffi::c_char,
        b"title\0".as_ptr() as *const ::core::ffi::c_char,
        b"user-commands\0".as_ptr() as *const ::core::ffi::c_char,
        b"user_commands\0".as_ptr() as *const ::core::ffi::c_char,
        b"vartabs\0".as_ptr() as *const ::core::ffi::c_char,
        b"vertsplit\0".as_ptr() as *const ::core::ffi::c_char,
        b"vimscript-1\0".as_ptr() as *const ::core::ffi::c_char,
        b"virtualedit\0".as_ptr() as *const ::core::ffi::c_char,
        b"visual\0".as_ptr() as *const ::core::ffi::c_char,
        b"visualextra\0".as_ptr() as *const ::core::ffi::c_char,
        b"vreplace\0".as_ptr() as *const ::core::ffi::c_char,
        b"wildignore\0".as_ptr() as *const ::core::ffi::c_char,
        b"wildmenu\0".as_ptr() as *const ::core::ffi::c_char,
        b"windows\0".as_ptr() as *const ::core::ffi::c_char,
        b"winaltkeys\0".as_ptr() as *const ::core::ffi::c_char,
        b"writebackup\0".as_ptr() as *const ::core::ffi::c_char,
        b"xattr\0".as_ptr() as *const ::core::ffi::c_char,
        b"nvim\0".as_ptr() as *const ::core::ffi::c_char,
    ]);
    let mut x: bool = false_0 != 0;
    let mut n: bool = false_0 != 0;
    let name: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    if strncasecmp(
        name as *mut ::core::ffi::c_char,
        b"patch\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        5 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        x = true_0 != 0;
        if *name.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '-' as ::core::ffi::c_int
            && strlen(name) >= 11 as size_t
            && (*name.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                >= '1' as ::core::ffi::c_int
                && *name.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    <= '9' as ::core::ffi::c_int)
        {
            let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut major: ::core::ffi::c_int = strtoul(
                name.offset(6 as ::core::ffi::c_int as isize),
                &raw mut end,
                10 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int;
            if *end as ::core::ffi::c_int == '.' as ::core::ffi::c_int
                && ascii_isdigit(*end.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
                && *end.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '.' as ::core::ffi::c_int
                && ascii_isdigit(*end.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
            {
                let mut minor: ::core::ffi::c_int =
                    atoi(end.offset(1 as ::core::ffi::c_int as isize));
                n = has_vim_patch(
                    atoi(end.offset(3 as ::core::ffi::c_int as isize)),
                    major * 100 as ::core::ffi::c_int + minor,
                );
            }
        } else if ascii_isdigit(*name.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        {
            n = has_vim_patch(
                atoi(name.offset(5 as ::core::ffi::c_int as isize)),
                0 as ::core::ffi::c_int,
            );
        }
    } else if strncasecmp(
        name as *mut ::core::ffi::c_char,
        b"nvim-\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        5 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        x = true_0 != 0;
        n = has_nvim_version(name.offset(5 as ::core::ffi::c_int as isize));
    } else if strcasecmp(
        name as *mut ::core::ffi::c_char,
        b"vim_starting\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        x = true_0 != 0;
        n = starting.get() != 0 as ::core::ffi::c_int;
    } else if strcasecmp(
        name as *mut ::core::ffi::c_char,
        b"ttyin\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        x = true_0 != 0;
        n = stdin_isatty.get();
    } else if strcasecmp(
        name as *mut ::core::ffi::c_char,
        b"ttyout\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        x = true_0 != 0;
        n = stdout_isatty.get();
    } else if strcasecmp(
        name as *mut ::core::ffi::c_char,
        b"multi_byte_encoding\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        x = true_0 != 0;
        n = true_0 != 0;
    } else if strcasecmp(
        name as *mut ::core::ffi::c_char,
        b"gui_running\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        x = true_0 != 0;
        n = ui_gui_attached();
    } else if strcasecmp(
        name as *mut ::core::ffi::c_char,
        b"syntax_items\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        x = true_0 != 0;
        n = syntax_present(curwin.get());
    } else if strcasecmp(
        name as *mut ::core::ffi::c_char,
        b"wsl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        x = true_0 != 0;
        n = has_wsl();
    }
    if !x {
        let mut i: size_t = 0 as size_t;
        while i < ::core::mem::size_of::<[*const ::core::ffi::c_char; 90]>()
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*const ::core::ffi::c_char; 90]>()
                    .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            )
        {
            if strcasecmp(
                name as *mut ::core::ffi::c_char,
                (*has_list.ptr())[i as usize] as *mut ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                x = true_0 != 0;
                n = true_0 != 0;
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
    }
    if !x {
        let save_shell_error: ::core::ffi::c_int =
            get_vim_var_nr(VV_SHELL_ERROR) as ::core::ffi::c_int;
        if strcasecmp(
            name as *mut ::core::ffi::c_char,
            b"clipboard_working\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            n = eval_has_provider(
                b"clipboard\0".as_ptr() as *const ::core::ffi::c_char,
                true_0 != 0,
            );
        } else if strcasecmp(
            name as *mut ::core::ffi::c_char,
            b"unnamedplus\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            n = eval_has_provider(
                b"clipboard\0".as_ptr() as *const ::core::ffi::c_char,
                true_0 != 0,
            );
        } else if strcasecmp(
            name as *mut ::core::ffi::c_char,
            b"pythonx\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            n = eval_has_provider(
                b"python3\0".as_ptr() as *const ::core::ffi::c_char,
                true_0 != 0,
            );
        } else if eval_has_provider(name, true_0 != 0) {
            n = true_0 != 0;
        }
        set_vim_var_nr(VV_SHELL_ERROR, save_shell_error as varnumber_T);
    }
    (*rettv).vval.v_number = n as varnumber_T;
}
unsafe extern "C" fn has_wsl() -> bool {
    static has_wsl_0: GlobalCell<TriState> = GlobalCell::new(kNone);
    if has_wsl_0.get() as ::core::ffi::c_int == kNone as ::core::ffi::c_int {
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut o: Object = nlua_exec(
            String_0 {
                data: b"return vim.uv.os_uname()['release']:lower():match('microsoft')\0".as_ptr()
                    as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 63]>().wrapping_sub(1 as size_t),
            },
            ::core::ptr::null::<::core::ffi::c_char>(),
            Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            },
            kRetNilBool,
            ::core::ptr::null_mut::<Arena>(),
            &raw mut err,
        );
        '_c2rust_label: {
            if !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
            } else {
                __assert_fail(
                    b"!ERROR_SET(&err)\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/funcs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2886 as ::core::ffi::c_uint,
                    b"_Bool has_wsl(void)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        has_wsl_0.set(
            (if o.type_0 as ::core::ffi::c_uint
                == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
                && o.data.boolean as ::core::ffi::c_int == true_0
            {
                kTrue as ::core::ffi::c_int
            } else {
                kFalse as ::core::ffi::c_int
            }) as TriState,
        );
    }
    return has_wsl_0.get() as ::core::ffi::c_int == kTrue as ::core::ffi::c_int;
}
unsafe extern "C" fn f_hlID(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = syn_name2id(tv_get_string(
        argvars.offset(0 as ::core::ffi::c_int as isize),
    )) as varnumber_T;
}
unsafe extern "C" fn f_hlexists(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = highlight_exists(tv_get_string(
        argvars.offset(0 as ::core::ffi::c_int as isize),
    )) as varnumber_T;
}
unsafe extern "C" fn f_hostname(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut hostname: [::core::ffi::c_char; 256] = [0; 256];
    os_get_hostname(&raw mut hostname as *mut ::core::ffi::c_char, 256 as size_t);
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = xstrdup(&raw mut hostname as *mut ::core::ffi::c_char);
}
unsafe extern "C" fn f_index(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut ic: bool = false_0 != 0;
    (*rettv).vval.v_number = -1 as varnumber_T;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut error: bool = false_0 != 0;
        let mut start: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            start = tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) as ::core::ffi::c_int;
            if error {
                return;
            }
        }
        let b: *mut blob_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_blob;
        if b.is_null() {
            return;
        }
        if start < 0 as ::core::ffi::c_int {
            start = tv_blob_len(b) + start;
            if start < 0 as ::core::ffi::c_int {
                start = 0 as ::core::ffi::c_int;
            }
        }
        idx = start;
        while idx < tv_blob_len(b) {
            let mut tv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            tv.v_type = VAR_NUMBER;
            tv.vval.v_number = tv_blob_get(b, idx) as varnumber_T;
            if tv_equal(
                &raw mut tv,
                argvars.offset(1 as ::core::ffi::c_int as isize),
                ic,
            ) {
                (*rettv).vval.v_number = idx as varnumber_T;
                return;
            }
            idx += 1;
        }
        return;
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(
            &raw const e_listblobreq as *const ::core::ffi::c_char,
        ));
        return;
    }
    let l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    if l.is_null() {
        return;
    }
    let mut item: *mut listitem_T = tv_list_first(l);
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut error_0: bool = false_0 != 0;
        idx = tv_list_uidx(
            l,
            tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error_0,
            ) as ::core::ffi::c_int,
        );
        if error_0 as ::core::ffi::c_int != 0 || idx == -1 as ::core::ffi::c_int {
            item = ::core::ptr::null_mut::<listitem_T>();
        } else {
            item = tv_list_find(l, idx);
            '_c2rust_label: {
                if !item.is_null() {
                } else {
                    __assert_fail(
                        b"item != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/eval/funcs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2971 as ::core::ffi::c_uint,
                        b"void f_index(typval_T *, typval_T *, EvalFuncData)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
        }
        if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            ic = tv_get_number_chk(
                argvars.offset(3 as ::core::ffi::c_int as isize),
                &raw mut error_0,
            ) != 0;
            if error_0 {
                item = ::core::ptr::null_mut::<listitem_T>();
            }
        }
    }
    while !item.is_null() {
        if tv_equal(
            &raw mut (*item).li_tv,
            argvars.offset(1 as ::core::ffi::c_int as isize),
            ic,
        ) {
            (*rettv).vval.v_number = idx as varnumber_T;
            break;
        } else {
            item = (*item).li_next;
            idx += 1;
        }
    }
}
unsafe extern "C" fn indexof_eval_expr(mut expr: *mut typval_T) -> varnumber_T {
    let mut argv: [typval_T; 3] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 3];
    argv[0 as ::core::ffi::c_int as usize] = *get_vim_var_tv(VV_KEY);
    argv[1 as ::core::ffi::c_int as usize] = *get_vim_var_tv(VV_VAL);
    let mut newtv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    newtv.v_type = VAR_UNKNOWN;
    if eval_expr_typval(
        expr,
        false_0 != 0,
        &raw mut argv as *mut typval_T,
        2 as ::core::ffi::c_int,
        &raw mut newtv,
    ) == FAIL
    {
        return false_0 as varnumber_T;
    }
    let mut error: bool = false_0 != 0;
    let mut found: varnumber_T = tv_get_bool_chk(&raw mut newtv, &raw mut error);
    tv_clear(&raw mut newtv);
    return if error as ::core::ffi::c_int != 0 {
        false_0 as varnumber_T
    } else {
        found
    };
}
unsafe extern "C" fn indexof_blob(
    mut b: *mut blob_T,
    mut startidx: varnumber_T,
    mut expr: *mut typval_T,
) -> varnumber_T {
    if b.is_null() {
        return -1 as varnumber_T;
    }
    if startidx < 0 as varnumber_T {
        startidx = tv_blob_len(b) as varnumber_T + startidx;
        if startidx < 0 as varnumber_T {
            startidx = 0 as varnumber_T;
        }
    }
    set_vim_var_type(VV_KEY, VAR_NUMBER);
    set_vim_var_type(VV_VAL, VAR_NUMBER);
    let called_emsg_start: ::core::ffi::c_int = called_emsg.get();
    let mut idx: varnumber_T = startidx;
    while idx < tv_blob_len(b) as varnumber_T {
        set_vim_var_nr(VV_KEY, idx);
        set_vim_var_nr(
            VV_VAL,
            tv_blob_get(b, idx as ::core::ffi::c_int) as varnumber_T,
        );
        if indexof_eval_expr(expr) != 0 {
            return idx;
        }
        if called_emsg.get() != called_emsg_start {
            return -1 as varnumber_T;
        }
        idx += 1;
    }
    return -1 as varnumber_T;
}
unsafe extern "C" fn indexof_list(
    mut l: *mut list_T,
    mut startidx: varnumber_T,
    mut expr: *mut typval_T,
) -> varnumber_T {
    if l.is_null() {
        return -1 as varnumber_T;
    }
    let mut item: *mut listitem_T = ::core::ptr::null_mut::<listitem_T>();
    let mut idx: varnumber_T = 0 as varnumber_T;
    if startidx == 0 as varnumber_T {
        item = tv_list_first(l);
    } else {
        idx = tv_list_uidx(l, startidx as ::core::ffi::c_int) as varnumber_T;
        if idx == -1 as varnumber_T {
            item = ::core::ptr::null_mut::<listitem_T>();
        } else {
            item = tv_list_find(l, idx as ::core::ffi::c_int);
            '_c2rust_label: {
                if !item.is_null() {
                } else {
                    __assert_fail(
                        b"item != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/eval/funcs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        3068 as ::core::ffi::c_uint,
                        b"varnumber_T indexof_list(list_T *, varnumber_T, typval_T *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
        }
    }
    set_vim_var_type(VV_KEY, VAR_NUMBER);
    let called_emsg_start: ::core::ffi::c_int = called_emsg.get();
    while !item.is_null() {
        set_vim_var_nr(VV_KEY, idx);
        tv_copy(&raw mut (*item).li_tv, get_vim_var_tv(VV_VAL));
        let mut found: bool = indexof_eval_expr(expr) != 0;
        tv_clear(get_vim_var_tv(VV_VAL));
        if found {
            return idx;
        }
        if called_emsg.get() != called_emsg_start {
            return -1 as varnumber_T;
        }
        item = (*item).li_next;
        idx += 1;
    }
    return -1 as varnumber_T;
}
unsafe extern "C" fn f_indexof(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
    if tv_check_for_list_or_blob_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || tv_check_for_string_or_func_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
        || tv_check_for_opt_dict_arg(argvars, 2 as ::core::ffi::c_int) == FAIL
    {
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        && ((*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_string
            .is_null()
            || *(*argvars.offset(1 as ::core::ffi::c_int as isize))
                .vval
                .v_string as ::core::ffi::c_int
                == NUL)
        || (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(1 as ::core::ffi::c_int as isize))
                .vval
                .v_partial
                .is_null()
    {
        return;
    }
    let mut startidx: varnumber_T = 0 as varnumber_T;
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        startidx = tv_dict_get_number_def(
            (*argvars.offset(2 as ::core::ffi::c_int as isize))
                .vval
                .v_dict,
            b"startidx\0".as_ptr() as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
    }
    let mut save_val: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut save_key: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    prepare_vimvar(VV_VAL as ::core::ffi::c_int, &raw mut save_val);
    prepare_vimvar(VV_KEY as ::core::ffi::c_int, &raw mut save_key);
    let save_did_emsg: ::core::ffi::c_int = did_emsg.get();
    did_emsg.set(false_0);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*rettv).vval.v_number = indexof_blob(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_blob,
            startidx,
            argvars.offset(1 as ::core::ffi::c_int as isize),
        );
    } else {
        (*rettv).vval.v_number = indexof_list(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_list,
            startidx,
            argvars.offset(1 as ::core::ffi::c_int as isize),
        );
    }
    restore_vimvar(VV_KEY as ::core::ffi::c_int, &raw mut save_key);
    restore_vimvar(VV_VAL as ::core::ffi::c_int, &raw mut save_val);
    (*did_emsg.ptr()) |= save_did_emsg;
}
static inputsecret_flag: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
unsafe extern "C" fn f_input(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    get_user_input(argvars, rettv, false_0 != 0, inputsecret_flag.get());
}
unsafe extern "C" fn f_inputdialog(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    get_user_input(argvars, rettv, true_0 != 0, inputsecret_flag.get());
}
unsafe extern "C" fn f_inputlist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_listarg as *const ::core::ffi::c_char),
            b"inputlist()\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    msg_ext_set_kind(b"confirm\0".as_ptr() as *const ::core::ffi::c_char);
    msg_start();
    msg_row.set(Rows.get() - 1 as ::core::ffi::c_int);
    lines_left.set(Rows.get());
    msg_scroll.set(true_0);
    msg_clr_eos();
    let mut l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    let l_: *const list_T = l;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            msg_puts(tv_get_string(&raw const (*li).li_tv));
            if !ui_has(kUIMessages) || !(*li).li_next.is_null() {
                msg_putchar('\n' as ::core::ffi::c_int);
            }
            li = (*li).li_next;
        }
    }
    let mut mouse_used: bool = false_0 != 0;
    let mut selected: ::core::ffi::c_int = prompt_for_input(
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
        false_0 != 0,
        &raw mut mouse_used,
    );
    if mouse_used {
        selected = tv_list_len(l) - (cmdline_row.get() - mouse_row.get());
    }
    (*rettv).vval.v_number = selected as varnumber_T;
}
static ga_userinput: GlobalCell<garray_T> = GlobalCell::new(garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: ::core::mem::size_of::<tasave_T>() as ::core::ffi::c_int,
    ga_growsize: 4 as ::core::ffi::c_int,
    ga_data: NULL_0,
});
unsafe extern "C" fn f_inputrestore(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if !((*ga_userinput.ptr()).ga_len <= 0 as ::core::ffi::c_int) {
        (*ga_userinput.ptr()).ga_len -= 1;
        restore_typeahead(
            ((*ga_userinput.ptr()).ga_data as *mut tasave_T)
                .offset((*ga_userinput.ptr()).ga_len as isize),
        );
    } else if p_verbose.get() > 1 as OptInt {
        verb_msg(gettext(
            b"called inputrestore() more often than inputsave()\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
        (*rettv).vval.v_number = 1 as varnumber_T;
    }
}
unsafe extern "C" fn f_inputsave(
    mut _argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut p: *mut tasave_T =
        ga_append_via_ptr(ga_userinput.ptr(), ::core::mem::size_of::<tasave_T>()) as *mut tasave_T;
    save_typeahead(p);
}
unsafe extern "C" fn f_inputsecret(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    (*cmdline_star.ptr()) += 1;
    inputsecret_flag.set(true_0 != 0);
    f_input(argvars, rettv, fptr);
    (*cmdline_star.ptr()) -= 1;
    inputsecret_flag.set(false_0 != 0);
}
unsafe extern "C" fn f_interrupt(
    mut _argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    got_int.set(true_0 != 0);
}
unsafe extern "C" fn f_invert(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = !tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    );
}
unsafe extern "C" fn f_islocked(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut lv: lval_T = lval_T {
        ll_name: ::core::ptr::null::<::core::ffi::c_char>(),
        ll_name_len: 0,
        ll_exp_name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ll_tv: ::core::ptr::null_mut::<typval_T>(),
        ll_li: ::core::ptr::null_mut::<listitem_T>(),
        ll_list: ::core::ptr::null_mut::<list_T>(),
        ll_range: false,
        ll_empty2: false,
        ll_n1: 0,
        ll_n2: 0,
        ll_dict: ::core::ptr::null_mut::<dict_T>(),
        ll_di: ::core::ptr::null_mut::<dictitem_T>(),
        ll_newkey: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ll_blob: ::core::ptr::null_mut::<blob_T>(),
    };
    (*rettv).vval.v_number = -1 as varnumber_T;
    let end: *const ::core::ffi::c_char = get_lval(
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_char,
        ::core::ptr::null_mut::<typval_T>(),
        &raw mut lv,
        false_0 != 0,
        false_0 != 0,
        GLV_NO_AUTOLOAD as ::core::ffi::c_int | GLV_READ_ONLY as ::core::ffi::c_int,
        FNE_CHECK_START,
    );
    if !end.is_null() && !lv.ll_name.is_null() {
        if *end as ::core::ffi::c_int != NUL {
            semsg(
                gettext(if lv.ll_name_len == 0 as size_t {
                    &raw const e_invarg2 as *const ::core::ffi::c_char
                } else {
                    &raw const e_trailing_arg as *const ::core::ffi::c_char
                }),
                end,
            );
        } else if lv.ll_tv.is_null() {
            let mut di: *mut dictitem_T = find_var(
                lv.ll_name,
                lv.ll_name_len,
                ::core::ptr::null_mut::<*mut hashtab_T>(),
                true_0,
            );
            if !di.is_null() {
                (*rettv).vval.v_number = ((*di).di_flags as ::core::ffi::c_int
                    & DI_FLAGS_LOCK as ::core::ffi::c_int
                    != 0
                    || tv_islocked(&raw mut (*di).di_tv) as ::core::ffi::c_int != 0)
                    as ::core::ffi::c_int as varnumber_T;
            }
        } else if lv.ll_range {
            emsg(gettext(
                b"E786: Range not allowed\0".as_ptr() as *const ::core::ffi::c_char
            ));
        } else if !lv.ll_newkey.is_null() {
            semsg(
                gettext(&raw const e_dictkey as *const ::core::ffi::c_char),
                lv.ll_newkey,
            );
        } else if !lv.ll_list.is_null() {
            (*rettv).vval.v_number = tv_islocked(&raw mut (*lv.ll_li).li_tv) as varnumber_T;
        } else {
            (*rettv).vval.v_number = tv_islocked(&raw mut (*lv.ll_di).di_tv) as varnumber_T;
        }
    }
    clear_lval(&raw mut lv);
}
unsafe extern "C" fn f_isinf(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
        && ((*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_float as ::core::ffi::c_double)
            .is_infinite()
    {
        (*rettv).vval.v_number = (if (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_float
            > 0.0f64
        {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        }) as varnumber_T;
    }
}
unsafe extern "C" fn f_isnan(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = ((*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type
        as ::core::ffi::c_uint
        == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
        && ((*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_float as ::core::ffi::c_double)
            .is_nan()) as ::core::ffi::c_int as varnumber_T;
}
unsafe extern "C" fn f_id(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let len: ::core::ffi::c_int = vim_vsnprintf_typval(
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        0 as size_t,
        b"%p\0".as_ptr() as *const ::core::ffi::c_char,
        (*dummy_ap.ptr()).as_va_list(),
        argvars,
    );
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string =
        xmalloc((len as size_t).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
    vim_vsnprintf_typval(
        (*rettv).vval.v_string,
        (len as size_t).wrapping_add(1 as size_t),
        b"%p\0".as_ptr() as *const ::core::ffi::c_char,
        (*dummy_ap.ptr()).as_va_list(),
        argvars,
    );
}
unsafe extern "C" fn f_jobpid(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let mut data: *mut Channel = find_job(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_number as uint64_t,
        true_0 != 0,
    );
    if data.is_null() {
        return;
    }
    let mut proc: *mut Proc = &raw mut (*data).stream.proc;
    (*rettv).vval.v_number = (*proc).pid as varnumber_T;
}
unsafe extern "C" fn f_jobresize(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let mut data: *mut Channel = find_job(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_number as uint64_t,
        true_0 != 0,
    );
    if data.is_null() {
        return;
    }
    if (*data).stream.proc.type_0 as ::core::ffi::c_uint
        != kProcTypePty as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(
            &raw const e_channotpty as *const ::core::ffi::c_char,
        ));
        return;
    }
    pty_proc_resize(
        &raw mut (*data).stream.pty,
        (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_number as uint16_t,
        (*argvars.offset(2 as ::core::ffi::c_int as isize))
            .vval
            .v_number as uint16_t,
    );
    (*rettv).vval.v_number = 1 as varnumber_T;
}
static pty_ignored_env_vars: GlobalCell<[*const ::core::ffi::c_char; 8]> = GlobalCell::new([
    b"COLUMNS\0".as_ptr() as *const ::core::ffi::c_char,
    b"LINES\0".as_ptr() as *const ::core::ffi::c_char,
    b"TERMCAP\0".as_ptr() as *const ::core::ffi::c_char,
    b"COLORFGBG\0".as_ptr() as *const ::core::ffi::c_char,
    b"COLORTERM\0".as_ptr() as *const ::core::ffi::c_char,
    b"VIM\0".as_ptr() as *const ::core::ffi::c_char,
    b"VIMRUNTIME\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
static required_env_vars: GlobalCell<[*const ::core::ffi::c_char; 1]> =
    GlobalCell::new([::core::ptr::null::<::core::ffi::c_char>()]);
pub unsafe extern "C" fn create_environment(
    mut job_env: *const dictitem_T,
    clear_env: bool,
    pty: bool,
    pty_term_name: *const ::core::ffi::c_char,
) -> *mut dict_T {
    let mut env: *mut dict_T = tv_dict_alloc();
    if !clear_env {
        let mut temp_env: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        f_environ(
            ::core::ptr::null_mut::<typval_T>(),
            &raw mut temp_env,
            EvalFuncData { null: NULL_0 },
        );
        tv_dict_extend(
            env,
            temp_env.vval.v_dict,
            b"force\0".as_ptr() as *const ::core::ffi::c_char,
        );
        tv_dict_free(temp_env.vval.v_dict);
        if pty {
            let mut i: size_t = 0 as size_t;
            while i < ::core::mem::size_of::<[*const ::core::ffi::c_char; 8]>()
                .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
                .wrapping_div(
                    (::core::mem::size_of::<[*const ::core::ffi::c_char; 8]>()
                        .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                        == 0) as ::core::ffi::c_int as usize,
                )
                && !(*pty_ignored_env_vars.ptr())[i as usize].is_null()
            {
                let mut dv: *mut dictitem_T = tv_dict_find(
                    env,
                    (*pty_ignored_env_vars.ptr())[i as usize],
                    -1 as ptrdiff_t,
                );
                if !dv.is_null() {
                    tv_dict_item_remove(env, dv);
                }
                i = i.wrapping_add(1);
            }
            if p_tgc.get() != 0 {
                tv_dict_add_str(
                    env,
                    b"COLORTERM\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
                    b"truecolor\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        }
    }
    if pty {
        let mut dv_0: *mut dictitem_T = tv_dict_find(
            env,
            b"TERM\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !dv_0.is_null() {
            tv_dict_item_remove(env, dv_0);
        }
        tv_dict_add_str(
            env,
            b"TERM\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            pty_term_name,
        );
    }
    let mut nvim_addr: *mut ::core::ffi::c_char = get_vim_var_str(VV_SEND_SERVER);
    if *nvim_addr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
        let mut dv_1: *mut dictitem_T = tv_dict_find(
            env,
            b"NVIM\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !dv_1.is_null() {
            tv_dict_item_remove(env, dv_1);
        }
        tv_dict_add_str(
            env,
            b"NVIM\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            nvim_addr,
        );
    }
    if !job_env.is_null() {
        tv_dict_extend(
            env,
            (*job_env).di_tv.vval.v_dict,
            b"force\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    if pty {
        let mut i_0: size_t = 0 as size_t;
        while i_0
            < ::core::mem::size_of::<[*const ::core::ffi::c_char; 1]>()
                .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
                .wrapping_div(
                    (::core::mem::size_of::<[*const ::core::ffi::c_char; 1]>()
                        .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                        == 0) as ::core::ffi::c_int as usize,
                )
            && !(*required_env_vars.ptr())[i_0 as usize].is_null()
        {
            let mut len: size_t = strlen((*required_env_vars.ptr())[i_0 as usize]);
            let mut dv_2: *mut dictitem_T = tv_dict_find(
                env,
                (*required_env_vars.ptr())[i_0 as usize],
                len as ptrdiff_t,
            );
            if dv_2.is_null() {
                let mut env_var: *mut ::core::ffi::c_char =
                    os_getenv((*required_env_vars.ptr())[i_0 as usize]);
                if !env_var.is_null() {
                    tv_dict_add_allocated_str(
                        env,
                        (*required_env_vars.ptr())[i_0 as usize],
                        len,
                        env_var,
                    );
                }
            }
            i_0 = i_0.wrapping_add(1);
        }
    }
    return env;
}
pub unsafe extern "C" fn f_jobstart(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut len: size_t = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeException,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    if check_secure() {
        return;
    }
    let mut cmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut executable: bool = true_0 != 0;
    let mut argv: *mut *mut ::core::ffi::c_char = tv_to_argv(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut cmd,
        &raw mut executable,
    );
    if argv.is_null() {
        (*rettv).vval.v_number = (if executable as ::core::ffi::c_int != 0 {
            0 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        }) as varnumber_T;
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"expected dictionary\0".as_ptr() as *const ::core::ffi::c_char,
        );
        shell_free_argv(argv);
        return;
    }
    let mut job_opts: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    let mut detach: bool = false_0 != 0;
    let mut rpc: bool = false_0 != 0;
    let mut pty: bool = false_0 != 0;
    let mut term: bool = false_0 != 0;
    let mut clear_env: bool = false_0 != 0;
    let mut overlapped: bool = false_0 != 0;
    let mut stdin_mode: ChannelStdinMode = kChannelStdinPipe;
    let mut on_stdout: CallbackReader = CallbackReader {
        cb: Callback {
            data: C2Rust_Unnamed_22 {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        },
        self_0: ::core::ptr::null_mut::<dict_T>(),
        buffer: GA_EMPTY_INIT_VALUE,
        eof: false,
        buffered: false_0 != 0,
        fwd_err: false_0 != 0,
        type_0: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    let mut on_stderr: CallbackReader = CallbackReader {
        cb: Callback {
            data: C2Rust_Unnamed_22 {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        },
        self_0: ::core::ptr::null_mut::<dict_T>(),
        buffer: GA_EMPTY_INIT_VALUE,
        eof: false,
        buffered: false_0 != 0,
        fwd_err: false_0 != 0,
        type_0: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    let mut on_exit: Callback = Callback {
        data: C2Rust_Unnamed_22 {
            funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        type_0: kCallbackNone,
    };
    let mut cwd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut job_env: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        job_opts = (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        detach = tv_dict_get_number(job_opts, b"detach\0".as_ptr() as *const ::core::ffi::c_char)
            != 0 as varnumber_T;
        rpc = tv_dict_get_number(job_opts, b"rpc\0".as_ptr() as *const ::core::ffi::c_char)
            != 0 as varnumber_T;
        term = tv_dict_get_number(job_opts, b"term\0".as_ptr() as *const ::core::ffi::c_char)
            != 0 as varnumber_T;
        pty = term as ::core::ffi::c_int != 0
            || tv_dict_get_number(job_opts, b"pty\0".as_ptr() as *const ::core::ffi::c_char)
                != 0 as varnumber_T;
        clear_env = tv_dict_get_number(
            job_opts,
            b"clear_env\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0 as varnumber_T;
        overlapped = tv_dict_get_number(
            job_opts,
            b"overlapped\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0 as varnumber_T;
        let mut s: *mut ::core::ffi::c_char = tv_dict_get_string(
            job_opts,
            b"stdin\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        if !s.is_null() {
            if strncmp(
                s,
                b"null\0".as_ptr() as *const ::core::ffi::c_char,
                NUMBUFLEN as ::core::ffi::c_int as size_t,
            ) == 0
            {
                stdin_mode = kChannelStdinNull;
            } else if strncmp(
                s,
                b"pipe\0".as_ptr() as *const ::core::ffi::c_char,
                NUMBUFLEN as ::core::ffi::c_int as size_t,
            ) != 0
            {
                semsg(
                    gettext(&raw const e_invargNval as *const ::core::ffi::c_char),
                    b"stdin\0".as_ptr() as *const ::core::ffi::c_char,
                    s,
                );
            }
        }
        let job_term: *mut dictitem_T = tv_dict_find(
            job_opts,
            b"term\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !job_term.is_null()
            && VAR_BOOL as ::core::ffi::c_int as ::core::ffi::c_uint
                != (*job_term).di_tv.v_type as ::core::ffi::c_uint
        {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                b"'term' must be Boolean\0".as_ptr() as *const ::core::ffi::c_char,
            );
            shell_free_argv(argv);
            return;
        }
        if pty as ::core::ffi::c_int != 0 && rpc as ::core::ffi::c_int != 0 {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                b"job cannot have both 'pty' and 'rpc' options set\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
            shell_free_argv(argv);
            return;
        }
        let mut new_cwd: *mut ::core::ffi::c_char = tv_dict_get_string(
            job_opts,
            b"cwd\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        if !new_cwd.is_null() && *new_cwd as ::core::ffi::c_int != NUL {
            cwd = new_cwd;
            if !os_isdir(cwd) {
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    b"expected valid directory\0".as_ptr() as *const ::core::ffi::c_char,
                );
                shell_free_argv(argv);
                return;
            }
        }
        job_env = tv_dict_find(
            job_opts,
            b"env\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !job_env.is_null()
            && (*job_env).di_tv.v_type as ::core::ffi::c_uint
                != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                b"env\0".as_ptr() as *const ::core::ffi::c_char,
            );
            shell_free_argv(argv);
            return;
        }
        if !common_job_callbacks(
            job_opts,
            &raw mut on_stdout,
            &raw mut on_stderr,
            &raw mut on_exit,
        ) {
            shell_free_argv(argv);
            return;
        }
    }
    let mut width: uint16_t =
        tv_dict_get_number(job_opts, b"width\0".as_ptr() as *const ::core::ffi::c_char) as uint16_t;
    let mut height: uint16_t =
        tv_dict_get_number(job_opts, b"height\0".as_ptr() as *const ::core::ffi::c_char)
            as uint16_t;
    let mut term_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if term {
        if text_locked() {
            text_locked_msg();
            shell_free_argv(argv);
            return;
        }
        if (*curbuf.get()).b_changed != 0 {
            emsg(gettext(
                b"jobstart(...,{term=true}) requires unmodified buffer\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            shell_free_argv(argv);
            return;
        }
        if !(*curbuf.get()).terminal.is_null() {
            if terminal_running((*curbuf.get()).terminal) {
                semsg(
                    gettext(b"Terminal already connected to buffer %d\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    (*curbuf.get()).handle,
                );
                shell_free_argv(argv);
                return;
            }
            buf_close_terminal(curbuf.get());
        }
        '_c2rust_label: {
            if !rpc {
            } else {
                __assert_fail(
                    b"!rpc\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/funcs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3606 as ::core::ffi::c_uint,
                    b"void f_jobstart(typval_T *, typval_T *, EvalFuncData)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        term_name =
            b"xterm-256color\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        cwd = (if !cwd.is_null() {
            cwd as *const ::core::ffi::c_char
        } else {
            b".\0".as_ptr() as *const ::core::ffi::c_char
        }) as *mut ::core::ffi::c_char;
        overlapped = false_0 != 0;
        detach = false_0 != 0;
        stdin_mode = kChannelStdinPipe;
        width = (if width as ::core::ffi::c_int != 0 {
            width as ::core::ffi::c_int
        } else {
            (if 0 as ::core::ffi::c_int > (*curwin.get()).w_view_width - win_col_off(curwin.get()) {
                0 as ::core::ffi::c_int
            } else {
                (*curwin.get()).w_view_width - win_col_off(curwin.get())
            }) as uint16_t as ::core::ffi::c_int
        }) as uint16_t;
        height = (if height as ::core::ffi::c_int != 0 {
            height as ::core::ffi::c_int
        } else {
            (*curwin.get()).w_view_height as uint16_t as ::core::ffi::c_int
        }) as uint16_t;
    }
    if pty {
        term_name = if !term_name.is_null() {
            term_name
        } else {
            tv_dict_get_string(
                job_opts,
                b"TERM\0".as_ptr() as *const ::core::ffi::c_char,
                false_0 != 0,
            )
        };
        term_name = (if !term_name.is_null() {
            term_name as *const ::core::ffi::c_char
        } else {
            b"ansi\0".as_ptr() as *const ::core::ffi::c_char
        }) as *mut ::core::ffi::c_char;
    }
    let mut env: *mut dict_T = create_environment(job_env, clear_env, pty, term_name);
    let mut chan: *mut Channel = channel_job_start(
        argv,
        ::core::ptr::null::<::core::ffi::c_char>(),
        on_stdout,
        on_stderr,
        on_exit,
        pty,
        rpc,
        overlapped,
        detach,
        stdin_mode,
        cwd,
        width,
        height,
        env,
        &raw mut (*rettv).vval.v_number,
    );
    if chan.is_null() {
        return;
    } else {
        if !term {
            channel_create_event(chan, ::core::ptr::null::<::core::ffi::c_char>());
        } else {
            if (*rettv).vval.v_number <= 0 as varnumber_T {
                return;
            }
            let pid: ::core::ffi::c_int = (*chan).stream.pty.proc.pid;
            let buf: *mut buf_T = curbuf.get();
            (*buf).b_p_swf = false_0;
            if (*buf).b_ml.ml_mfp.is_null() && ml_open(buf) == FAIL {
                proc_stop(&raw mut (*chan).stream.proc);
                channel_decref(chan);
                return;
            }
            channel_incref(chan);
            channel_terminal_alloc(buf, chan);
            apply_autocmds(
                EVENT_BUFFILEPRE,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                buf,
            );
            if !((*chan).term.is_null() || terminal_buf((*chan).term) == 0 as ::core::ffi::c_int) {
                vim_FullName(
                    cwd,
                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
                    false_0 != 0,
                );
                len = home_replace(
                    ::core::ptr::null::<buf_T>(),
                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
                    true_0 != 0,
                );
                if len != 1 as size_t
                    && ((*IObuff.ptr())[len.wrapping_sub(1 as size_t) as usize]
                        as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int
                        || (*IObuff.ptr())[len.wrapping_sub(1 as size_t) as usize]
                            as ::core::ffi::c_int
                            == '/' as ::core::ffi::c_int)
                {
                    (*IObuff.ptr())[len.wrapping_sub(1 as size_t) as usize] =
                        NUL as ::core::ffi::c_char;
                }
                if len == 1 as size_t
                    && (*IObuff.ptr())[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                        == '/' as ::core::ffi::c_int
                {
                    (*IObuff.ptr())[1 as ::core::ffi::c_int as usize] = '.' as ::core::ffi::c_char;
                    (*IObuff.ptr())[2 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
                }
                snprintf(
                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
                    b"term://%s//%d:%s\0".as_ptr() as *const ::core::ffi::c_char,
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    pid,
                    cmd,
                );
                setfname(
                    buf,
                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    true_0 != 0,
                );
                apply_autocmds(
                    EVENT_BUFFILEPOST,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    false_0 != 0,
                    buf,
                );
                if !((*chan).term.is_null()
                    || terminal_buf((*chan).term) == 0 as ::core::ffi::c_int)
                {
                    err = Error {
                        type_0: kErrorTypeNone,
                        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    };
                    (*buf).b_locked += 1;
                    dict_set_var(
                        (*buf).b_vars,
                        cstr_as_string(b"terminal_job_id\0".as_ptr() as *const ::core::ffi::c_char),
                        object {
                            type_0: kObjectTypeInteger,
                            data: C2Rust_Unnamed_16 {
                                integer: (*chan).id as Integer,
                            },
                        },
                        false_0 != 0,
                        false_0 != 0,
                        ::core::ptr::null_mut::<Arena>(),
                        &raw mut err,
                    );
                    api_clear_error(&raw mut err);
                    dict_set_var(
                        (*buf).b_vars,
                        cstr_as_string(b"terminal_job_pid\0".as_ptr() as *const ::core::ffi::c_char),
                        object {
                            type_0: kObjectTypeInteger,
                            data: C2Rust_Unnamed_16 {
                                integer: pid as Integer,
                            },
                        },
                        false_0 != 0,
                        false_0 != 0,
                        ::core::ptr::null_mut::<Arena>(),
                        &raw mut err,
                    );
                    api_clear_error(&raw mut err);
                    (*buf).b_locked -= 1;
                    if !((*chan).term.is_null()
                        || terminal_buf((*chan).term) == 0 as ::core::ffi::c_int)
                    {
                        terminal_open(&raw mut (*chan).term, buf);
                    }
                }
            }
            channel_create_event(chan, ::core::ptr::null::<::core::ffi::c_char>());
            channel_decref(chan);
        }
        return;
    };
}
pub unsafe extern "C" fn f_jobstop(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let mut data: *mut Channel = find_job(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_number as uint64_t,
        false_0 != 0,
    );
    if data.is_null() {
        return;
    }
    let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*data).is_rpc {
        channel_close((*data).id, kChannelPartRpc, &raw mut error);
    }
    proc_stop(&raw mut (*data).stream.proc);
    (*rettv).vval.v_number = 1 as varnumber_T;
    if !error.is_null() {
        emsg(error);
    }
}
unsafe extern "C" fn f_jobwait(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let mut args: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    let mut jobs: *mut *mut Channel = xcalloc(
        tv_list_len(args) as size_t,
        ::core::mem::size_of::<*mut Channel>(),
    ) as *mut *mut Channel;
    let mut waiting_jobs: *mut MultiQueue = multiqueue_new(
        Some(loop_on_put as unsafe extern "C" fn(*mut MultiQueue, *mut ::core::ffi::c_void) -> ()),
        main_loop.ptr() as *mut ::core::ffi::c_void,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let l_: *const list_T = args;
    if !l_.is_null() {
        let mut arg: *const listitem_T = (*l_).lv_first;
        while !arg.is_null() {
            let mut chan: *mut Channel = ::core::ptr::null_mut::<Channel>();
            if (*arg).li_tv.v_type as ::core::ffi::c_uint
                != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                || {
                    chan = find_channel((*arg).li_tv.vval.v_number as uint64_t);
                    chan.is_null()
                }
                || (*chan).streamtype as ::core::ffi::c_uint
                    != kChannelStreamProc as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                *jobs.offset(i as isize) = ::core::ptr::null_mut::<Channel>();
            } else if proc_is_stopped(&raw mut (*chan).stream.proc) {
                proc_wait(
                    &raw mut (*chan).stream.proc,
                    -1 as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<MultiQueue>(),
                );
                *jobs.offset(i as isize) = ::core::ptr::null_mut::<Channel>();
            } else {
                *jobs.offset(i as isize) = chan;
                channel_incref(chan);
                if (*chan).stream.proc.status < 0 as ::core::ffi::c_int {
                    multiqueue_process_events((*chan).events);
                    multiqueue_replace_parent((*chan).events, waiting_jobs);
                }
            }
            i += 1;
            arg = (*arg).li_next;
        }
    }
    let mut remaining: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut before: uint64_t = 0 as uint64_t;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_number
            >= 0 as varnumber_T
    {
        remaining = (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_number as ::core::ffi::c_int;
        before = os_hrtime();
    }
    let busy: bool = remaining != 0 as ::core::ffi::c_int;
    if busy {
        ui_busy_start();
        ui_flush();
    }
    i = 0 as ::core::ffi::c_int;
    while i < tv_list_len(args) {
        if remaining == 0 as ::core::ffi::c_int {
            break;
        }
        if !(*jobs.offset(i as isize)).is_null() {
            let mut status: ::core::ffi::c_int = proc_wait(
                &raw mut (**jobs.offset(i as isize)).stream.proc,
                remaining,
                waiting_jobs,
            );
            if status < 0 as ::core::ffi::c_int {
                break;
            }
            if remaining > 0 as ::core::ffi::c_int {
                let mut now: uint64_t = os_hrtime();
                remaining = if (0 as ::core::ffi::c_int)
                    < remaining
                        - now.wrapping_sub(before).wrapping_div(1000000 as uint64_t)
                            as ::core::ffi::c_int
                {
                    0 as ::core::ffi::c_int
                } else {
                    remaining
                        - now.wrapping_sub(before).wrapping_div(1000000 as uint64_t)
                            as ::core::ffi::c_int
                };
                before = now;
            }
        }
        i += 1;
    }
    let rv: *mut list_T = tv_list_alloc(tv_list_len(args) as ptrdiff_t);
    i = 0 as ::core::ffi::c_int;
    while i < tv_list_len(args) {
        if (*jobs.offset(i as isize)).is_null() {
            tv_list_append_number(rv, -3 as varnumber_T);
        } else {
            multiqueue_process_events((**jobs.offset(i as isize)).events);
            multiqueue_replace_parent(
                (**jobs.offset(i as isize)).events,
                (*main_loop.ptr()).events,
            );
            tv_list_append_number(
                rv,
                (**jobs.offset(i as isize)).stream.proc.status as varnumber_T,
            );
            channel_decref(*jobs.offset(i as isize));
        }
        i += 1;
    }
    multiqueue_free(waiting_jobs);
    xfree(jobs as *mut ::core::ffi::c_void);
    if busy {
        ui_busy_stop();
    }
    tv_list_ref(rv);
    (*rettv).v_type = VAR_LIST;
    (*rettv).vval.v_list = rv;
}
unsafe extern "C" fn f_json_decode(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut numbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !encode_vim_list_to_buf(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_list,
            &raw mut len,
            &raw mut tofree,
        ) {
            emsg(gettext(
                b"E474: Failed to convert list to string\0".as_ptr() as *const ::core::ffi::c_char,
            ));
            return;
        }
        s = tofree;
        if s.is_null() {
            '_c2rust_label: {
                if len == 0 as size_t {
                } else {
                    __assert_fail(
                        b"len == 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/eval/funcs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        3859 as ::core::ffi::c_uint,
                        b"void f_json_decode(typval_T *, typval_T *, EvalFuncData)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            s = b"\0".as_ptr() as *const ::core::ffi::c_char;
        }
    } else {
        s = tv_get_string_buf_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut numbuf as *mut ::core::ffi::c_char,
        );
        if !s.is_null() {
            len = strlen(s);
        } else {
            return;
        }
    }
    if json_decode_string(s, len, rettv) == FAIL {
        semsg(
            gettext(b"E474: Failed to parse %.*s\0".as_ptr() as *const ::core::ffi::c_char),
            len as ::core::ffi::c_int,
            s,
        );
        (*rettv).v_type = VAR_NUMBER;
        (*rettv).vval.v_number = 0 as varnumber_T;
    }
    '_c2rust_label_0: {
        if (*rettv).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"rettv->v_type != VAR_UNKNOWN\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/funcs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                3875 as ::core::ffi::c_uint,
                b"void f_json_decode(typval_T *, typval_T *, EvalFuncData)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    xfree(tofree as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn f_json_encode(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = encode_tv2json(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<size_t>(),
    );
}
unsafe extern "C" fn f_keytrans(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    if tv_check_for_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_string
            .is_null()
    {
        return;
    }
    let mut escaped: *mut ::core::ffi::c_char = vim_strsave_escape_ks(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_string,
    );
    (*rettv).vval.v_string = str2special_save(escaped, true_0 != 0, true_0 != 0);
    xfree(escaped as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn f_len(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    match (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint {
        2 | 1 => {
            (*rettv).vval.v_number = strlen(tv_get_string(
                argvars.offset(0 as ::core::ffi::c_int as isize),
            )) as varnumber_T;
        }
        10 => {
            (*rettv).vval.v_number = tv_blob_len(
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_blob,
            ) as varnumber_T;
        }
        4 => {
            (*rettv).vval.v_number = tv_list_len(
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_list,
            ) as varnumber_T;
        }
        5 => {
            (*rettv).vval.v_number = tv_dict_len(
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_dict,
            ) as varnumber_T;
        }
        0 | 7 | 8 | 6 | 9 | 3 => {
            emsg(gettext(
                b"E701: Invalid type for len()\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
        _ => {}
    };
}
unsafe extern "C" fn libcall_common(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut out_type: ::core::ffi::c_int,
) {
    (*rettv).v_type = out_type as VarType;
    if out_type != VAR_NUMBER as ::core::ffi::c_int {
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    let mut libname: *const ::core::ffi::c_char = (*argvars
        .offset(0 as ::core::ffi::c_int as isize))
    .vval
    .v_string;
    let mut funcname: *const ::core::ffi::c_char = (*argvars
        .offset(1 as ::core::ffi::c_int as isize))
    .vval
    .v_string;
    let mut in_type: VarType = (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type;
    let mut str_in: *mut ::core::ffi::c_char = if in_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*argvars.offset(2 as ::core::ffi::c_int as isize))
            .vval
            .v_string
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
    let mut int_in: ::core::ffi::c_int = (*argvars.offset(2 as ::core::ffi::c_int as isize))
        .vval
        .v_number as ::core::ffi::c_int;
    let mut str_out: *mut *mut ::core::ffi::c_char = if out_type == VAR_STRING as ::core::ffi::c_int
    {
        &raw mut (*rettv).vval.v_string
    } else {
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>()
    };
    let mut int_out: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut success: bool =
        os_libcall(libname, funcname, str_in, int_in, str_out, &raw mut int_out);
    if !success {
        semsg(
            gettext(&raw const e_libcall as *const ::core::ffi::c_char),
            funcname,
        );
        return;
    }
    if out_type == VAR_NUMBER as ::core::ffi::c_int {
        (*rettv).vval.v_number = int_out as varnumber_T;
    }
}
unsafe extern "C" fn f_libcall(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    libcall_common(argvars, rettv, VAR_STRING as ::core::ffi::c_int);
}
unsafe extern "C" fn f_libcallnr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    libcall_common(argvars, rettv, VAR_NUMBER as ::core::ffi::c_int);
}
unsafe extern "C" fn f_line(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut lnum: linenr_T = 0 as linenr_T;
    let mut fp: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut fnum: ::core::ffi::c_int = 0;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut id: ::core::ffi::c_int =
            tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
        let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
        let mut wp: *mut win_T = win_id2wp_tp(id, &raw mut tp);
        if !wp.is_null() && !tp.is_null() {
            if *p_spk.get() as ::core::ffi::c_int != 'c' as ::core::ffi::c_int
                || (*wp).w_onebuf_opt.wo_diff != 0 && (*curwin.get()).w_onebuf_opt.wo_diff != 0
            {
                skip_update_topline.set(true_0 != 0);
            }
            check_cursor(wp);
            fp = var2fpos(
                argvars.offset(0 as ::core::ffi::c_int as isize),
                true_0 != 0,
                &raw mut fnum,
                false_0 != 0,
                wp,
            );
            skip_update_topline.set(false_0 != 0);
        }
    } else {
        fp = var2fpos(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            true_0 != 0,
            &raw mut fnum,
            false_0 != 0,
            curwin.get(),
        );
    }
    if !fp.is_null() {
        lnum = (*fp).lnum;
    }
    (*rettv).vval.v_number = lnum as varnumber_T;
}
unsafe extern "C" fn f_line2byte(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let lnum: linenr_T = tv_get_lnum(argvars);
    if lnum < 1 as linenr_T || lnum > (*curbuf.get()).b_ml.ml_line_count + 1 as linenr_T {
        (*rettv).vval.v_number = -1 as varnumber_T;
    } else {
        (*rettv).vval.v_number = ml_find_line_or_offset(
            curbuf.get(),
            lnum,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            false_0 != 0,
        ) as varnumber_T;
    }
    if (*rettv).vval.v_number >= 0 as varnumber_T {
        (*rettv).vval.v_number += 1;
    }
}
unsafe extern "C" fn f_localtime(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = time(::core::ptr::null_mut::<time_t>()) as varnumber_T;
}
unsafe extern "C" fn f_luaeval(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let str: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    if str.is_null() {
        return;
    }
    nlua_typval_eval(
        cstr_as_string(str),
        argvars.offset(1 as ::core::ffi::c_int as isize),
        rettv,
    );
}
unsafe extern "C" fn find_some_match(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    type_0: SomeMatchType,
) {
    let mut patbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut pat: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: int64_t = 0 as int64_t;
    let mut expr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut start: int64_t = 0 as int64_t;
    let mut nth: int64_t = 1 as int64_t;
    let mut startcol: colnr_T = 0 as colnr_T;
    let mut match_0: bool = false_0 != 0;
    let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut save_cpo: *mut ::core::ffi::c_char = p_cpo.get();
    p_cpo.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
    (*rettv).vval.v_number = -1 as varnumber_T;
    match type_0 as ::core::ffi::c_uint {
        2 => {
            tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        }
        4 => {
            tv_list_alloc_ret(rettv, 4 as ptrdiff_t);
            tv_list_append_string(
                (*rettv).vval.v_list,
                b"\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ssize_t,
            );
            tv_list_append_number((*rettv).vval.v_list, -1 as varnumber_T);
            tv_list_append_number((*rettv).vval.v_list, -1 as varnumber_T);
            tv_list_append_number((*rettv).vval.v_list, -1 as varnumber_T);
        }
        3 => {
            (*rettv).v_type = VAR_STRING;
            (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        0 | 1 | _ => {}
    }
    let mut li: *mut listitem_T = ::core::ptr::null_mut::<listitem_T>();
    '_theend: {
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            l = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_list;
            if l.is_null() {
                break '_theend;
            } else {
                li = tv_list_first(l);
            }
        } else {
            str = tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize))
                as *mut ::core::ffi::c_char;
            expr = str;
            len = strlen(str) as int64_t;
        }
        patbuf = [0; 65];
        pat = tv_get_string_buf_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut patbuf as *mut ::core::ffi::c_char,
        );
        if !pat.is_null() {
            if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut error: bool = false_0 != 0;
                start = tv_get_number_chk(
                    argvars.offset(2 as ::core::ffi::c_int as isize),
                    &raw mut error,
                ) as int64_t;
                if error {
                    break '_theend;
                } else {
                    if !l.is_null() {
                        idx = tv_list_uidx(l, start as ::core::ffi::c_int);
                        if idx == -1 as ::core::ffi::c_int {
                            break '_theend;
                        } else {
                            li = tv_list_find(l, idx);
                        }
                    } else {
                        if start < 0 as int64_t {
                            start = 0 as int64_t;
                        }
                        if start > len {
                            break '_theend;
                        } else if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type
                            as ::core::ffi::c_uint
                            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            startcol = start as colnr_T;
                        } else {
                            str = str.offset(start as isize);
                            len -= start;
                        }
                    }
                    if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type
                        as ::core::ffi::c_uint
                        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        nth = tv_get_number_chk(
                            argvars.offset(3 as ::core::ffi::c_int as isize),
                            &raw mut error,
                        ) as int64_t;
                    }
                    if error {
                        break '_theend;
                    }
                }
            }
            regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
            if !regmatch.regprog.is_null() {
                regmatch.rm_ic = p_ic.get() != 0;
                loop {
                    if !l.is_null() {
                        if li.is_null() {
                            match_0 = false_0 != 0;
                            break;
                        } else {
                            xfree(tofree as *mut ::core::ffi::c_void);
                            str = encode_tv2echo(
                                &raw mut (*li).li_tv,
                                ::core::ptr::null_mut::<size_t>(),
                            );
                            expr = str;
                            tofree = expr;
                            if str.is_null() {
                                break;
                            }
                        }
                    }
                    match_0 = vim_regexec_nl(&raw mut regmatch, str, startcol);
                    if match_0 as ::core::ffi::c_int != 0 && {
                        nth -= 1;
                        nth <= 0 as int64_t
                    } {
                        break;
                    }
                    if l.is_null() && !match_0 {
                        break;
                    }
                    if !l.is_null() {
                        li = (*li).li_next;
                        idx += 1;
                    } else {
                        startcol = regmatch.startp[0 as ::core::ffi::c_int as usize]
                            .offset(
                                utfc_ptr2len(regmatch.startp[0 as ::core::ffi::c_int as usize])
                                    as isize,
                            )
                            .offset_from(str) as colnr_T;
                        if !(startcol > len as colnr_T
                            || str.offset(startcol as isize)
                                <= regmatch.startp[0 as ::core::ffi::c_int as usize])
                        {
                            continue;
                        }
                        match_0 = false_0 != 0;
                        break;
                    }
                }
                if match_0 {
                    match type_0 as ::core::ffi::c_uint {
                        4 => {
                            let ret_l: *mut list_T = (*rettv).vval.v_list;
                            let mut li1: *mut listitem_T = tv_list_first(ret_l);
                            let mut li2: *mut listitem_T = (*li1).li_next;
                            let mut li3: *mut listitem_T = (*li2).li_next;
                            let mut li4: *mut listitem_T = (*li3).li_next;
                            xfree((*li1).li_tv.vval.v_string as *mut ::core::ffi::c_void);
                            let rd: size_t = regmatch.endp[0 as ::core::ffi::c_int as usize]
                                .offset_from(regmatch.startp[0 as ::core::ffi::c_int as usize])
                                as size_t;
                            (*li1).li_tv.vval.v_string = xmemdupz(
                                regmatch.startp[0 as ::core::ffi::c_int as usize]
                                    as *const ::core::ffi::c_void,
                                rd,
                            )
                                as *mut ::core::ffi::c_char;
                            (*li3).li_tv.vval.v_number =
                                regmatch.startp[0 as ::core::ffi::c_int as usize].offset_from(expr)
                                    as varnumber_T;
                            (*li4).li_tv.vval.v_number =
                                regmatch.endp[0 as ::core::ffi::c_int as usize].offset_from(expr)
                                    as varnumber_T;
                            if !l.is_null() {
                                (*li2).li_tv.vval.v_number = idx as varnumber_T;
                            }
                        }
                        2 => {
                            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            while i < NSUBEXP as ::core::ffi::c_int {
                                if regmatch.endp[i as usize].is_null() {
                                    tv_list_append_string(
                                        (*rettv).vval.v_list,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        0 as ssize_t,
                                    );
                                } else {
                                    tv_list_append_string(
                                        (*rettv).vval.v_list,
                                        regmatch.startp[i as usize],
                                        regmatch.endp[i as usize]
                                            .offset_from(regmatch.startp[i as usize])
                                            as ssize_t,
                                    );
                                }
                                i += 1;
                            }
                        }
                        3 => {
                            if !l.is_null() {
                                tv_copy(&raw mut (*li).li_tv, rettv);
                            } else {
                                (*rettv).vval.v_string = xmemdupz(
                                    regmatch.startp[0 as ::core::ffi::c_int as usize]
                                        as *const ::core::ffi::c_void,
                                    regmatch.endp[0 as ::core::ffi::c_int as usize].offset_from(
                                        regmatch.startp[0 as ::core::ffi::c_int as usize],
                                    ) as size_t,
                                )
                                    as *mut ::core::ffi::c_char;
                            }
                        }
                        0 | 1 => {
                            if !l.is_null() {
                                (*rettv).vval.v_number = idx as varnumber_T;
                            } else {
                                if type_0 as ::core::ffi::c_uint
                                    == kSomeMatch as ::core::ffi::c_int as ::core::ffi::c_uint
                                {
                                    (*rettv).vval.v_number = regmatch.startp
                                        [0 as ::core::ffi::c_int as usize]
                                        .offset_from(str)
                                        as varnumber_T;
                                } else {
                                    (*rettv).vval.v_number = regmatch.endp
                                        [0 as ::core::ffi::c_int as usize]
                                        .offset_from(str)
                                        as varnumber_T;
                                }
                                (*rettv).vval.v_number += str.offset_from(expr) as varnumber_T;
                            }
                        }
                        _ => {}
                    }
                }
                vim_regfree(regmatch.regprog);
            }
        }
    }
    if type_0 as ::core::ffi::c_uint
        == kSomeMatchStrPos as ::core::ffi::c_int as ::core::ffi::c_uint
        && l.is_null()
        && !(*rettv).vval.v_list.is_null()
    {
        let ret_l_0: *mut list_T = (*rettv).vval.v_list;
        tv_list_item_remove(ret_l_0, (*tv_list_first(ret_l_0)).li_next);
    }
    xfree(tofree as *mut ::core::ffi::c_void);
    p_cpo.set(save_cpo);
}
unsafe extern "C" fn get_matches_in_str(
    mut str: *const ::core::ffi::c_char,
    mut rmp: *mut regmatch_T,
    mut mlist: *mut list_T,
    mut idx: ::core::ffi::c_int,
    mut submatches: bool,
    mut matchbuf: bool,
) {
    let mut len: size_t = strlen(str);
    let mut match_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut startidx: colnr_T = 0 as colnr_T;
    loop {
        match_0 = vim_regexec_nl(rmp, str, startidx) as ::core::ffi::c_int;
        if match_0 == 0 {
            break;
        }
        let mut d: *mut dict_T = tv_dict_alloc();
        tv_list_append_dict(mlist, d);
        if matchbuf {
            tv_dict_add_nr(
                d,
                b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                idx as varnumber_T,
            );
        } else {
            tv_dict_add_nr(
                d,
                b"idx\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
                idx as varnumber_T,
            );
        }
        tv_dict_add_nr(
            d,
            b"byteidx\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            (*rmp).startp[0 as ::core::ffi::c_int as usize].offset_from(str) as colnr_T
                as varnumber_T,
        );
        tv_dict_add_str_len(
            d,
            b"text\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            (*rmp).startp[0 as ::core::ffi::c_int as usize],
            (*rmp).endp[0 as ::core::ffi::c_int as usize]
                .offset_from((*rmp).startp[0 as ::core::ffi::c_int as usize])
                as ::core::ffi::c_int,
        );
        if submatches {
            let mut sml: *mut list_T = tv_list_alloc(
                (NSUBEXP as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as ptrdiff_t,
            );
            tv_dict_add_list(
                d,
                b"submatches\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as size_t),
                sml,
            );
            let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            while i < NSUBEXP as ::core::ffi::c_int {
                if (*rmp).endp[i as usize].is_null() {
                    tv_list_append_string(
                        sml,
                        b"\0".as_ptr() as *const ::core::ffi::c_char,
                        0 as ssize_t,
                    );
                } else {
                    tv_list_append_string(
                        sml,
                        (*rmp).startp[i as usize],
                        (*rmp).endp[i as usize].offset_from((*rmp).startp[i as usize]) as ssize_t,
                    );
                }
                i += 1;
            }
        }
        startidx = (*rmp).endp[0 as ::core::ffi::c_int as usize].offset_from(str) as colnr_T;
        if startidx >= len as colnr_T
            || str.offset(startidx as isize)
                <= (*rmp).startp[0 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char
        {
            break;
        }
    }
}
unsafe extern "C" fn f_matchbufline(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
    tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
    let mut retlist: *mut list_T = (*rettv).vval.v_list;
    if tv_check_for_buffer_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || tv_check_for_string_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
        || tv_check_for_lnum_arg(argvars, 2 as ::core::ffi::c_int) == FAIL
        || tv_check_for_lnum_arg(argvars, 3 as ::core::ffi::c_int) == FAIL
        || tv_check_for_opt_dict_arg(argvars, 4 as ::core::ffi::c_int) == FAIL
    {
        return;
    }
    let prev_did_emsg: ::core::ffi::c_int = did_emsg.get();
    let mut buf: *mut buf_T = tv_get_buf(argvars.offset(0 as ::core::ffi::c_int as isize), false_0);
    if buf.is_null() {
        if did_emsg.get() == prev_did_emsg {
            semsg(
                gettext(&raw const e_invalid_buffer_name_str as *const ::core::ffi::c_char),
                tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
            );
        }
        return;
    }
    if (*buf).b_ml.ml_mfp.is_null() {
        emsg(gettext(
            &raw const e_buffer_is_not_loaded as *const ::core::ffi::c_char,
        ));
        return;
    }
    let mut patbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut pat: *const ::core::ffi::c_char = tv_get_string_buf(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut patbuf as *mut ::core::ffi::c_char,
    );
    let did_emsg_before: ::core::ffi::c_int = did_emsg.get();
    let mut slnum: linenr_T =
        tv_get_lnum_buf(argvars.offset(2 as ::core::ffi::c_int as isize), buf);
    if did_emsg.get() > did_emsg_before {
        return;
    }
    if slnum < 1 as linenr_T {
        semsg(
            gettext(&raw const e_invargval as *const ::core::ffi::c_char),
            b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut elnum: linenr_T =
        tv_get_lnum_buf(argvars.offset(3 as ::core::ffi::c_int as isize), buf);
    if did_emsg.get() > did_emsg_before {
        return;
    }
    if elnum < 1 as linenr_T || elnum < slnum {
        semsg(
            gettext(&raw const e_invargval as *const ::core::ffi::c_char),
            b"end_lnum\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    if elnum > (*buf).b_ml.ml_line_count {
        elnum = (*buf).b_ml.ml_line_count;
    }
    let mut submatches: bool = false_0 != 0;
    if (*argvars.offset(4 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut d: *mut dict_T = (*argvars.offset(4 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        if !d.is_null() {
            let mut di: *mut dictitem_T = tv_dict_find(
                d,
                b"submatches\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
            );
            if !di.is_null() {
                if (*di).di_tv.v_type as ::core::ffi::c_uint
                    != VAR_BOOL as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    semsg(
                        gettext(&raw const e_invargval as *const ::core::ffi::c_char),
                        b"submatches\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    return;
                }
                submatches = tv_get_bool(&raw mut (*di).di_tv) != 0;
            }
        }
    }
    let save_cpo: *mut ::core::ffi::c_char = p_cpo.get();
    p_cpo.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
    if !regmatch.regprog.is_null() {
        regmatch.rm_ic = p_ic.get() != 0;
        while slnum <= elnum {
            let mut str: *const ::core::ffi::c_char = ml_get_buf(buf, slnum);
            get_matches_in_str(
                str,
                &raw mut regmatch,
                retlist,
                slnum as ::core::ffi::c_int,
                submatches,
                true_0 != 0,
            );
            slnum += 1;
        }
        vim_regfree(regmatch.regprog);
    }
    p_cpo.set(save_cpo);
}
unsafe extern "C" fn f_match(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    find_some_match(argvars, rettv, kSomeMatch);
}
unsafe extern "C" fn f_matchend(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    find_some_match(argvars, rettv, kSomeMatchEnd);
}
unsafe extern "C" fn f_matchlist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    find_some_match(argvars, rettv, kSomeMatchList);
}
unsafe extern "C" fn f_matchstr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    find_some_match(argvars, rettv, kSomeMatchStr);
}
unsafe extern "C" fn f_matchstrlist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut idx: ::core::ffi::c_int = 0;
    let mut submatches: bool = false;
    (*rettv).vval.v_number = -1 as varnumber_T;
    tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
    let mut retlist: *mut list_T = (*rettv).vval.v_list;
    if tv_check_for_list_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || tv_check_for_string_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
        || tv_check_for_opt_dict_arg(argvars, 2 as ::core::ffi::c_int) == FAIL
    {
        return;
    }
    let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
    l = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    if l.is_null() {
        return;
    }
    let mut patbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut pat: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut patbuf as *mut ::core::ffi::c_char,
    );
    if pat.is_null() {
        return;
    }
    let save_cpo: *mut ::core::ffi::c_char = p_cpo.get();
    p_cpo.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
    if !regmatch.regprog.is_null() {
        regmatch.rm_ic = p_ic.get() != 0;
        submatches = false_0 != 0;
        '_cleanup: {
            if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut d: *mut dict_T = (*argvars.offset(2 as ::core::ffi::c_int as isize))
                    .vval
                    .v_dict;
                if !d.is_null() {
                    let mut di: *mut dictitem_T = tv_dict_find(
                        d,
                        b"submatches\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as usize)
                            as ptrdiff_t,
                    );
                    if !di.is_null() {
                        if (*di).di_tv.v_type as ::core::ffi::c_uint
                            != VAR_BOOL as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            semsg(
                                gettext(&raw const e_invargval as *const ::core::ffi::c_char),
                                b"submatches\0".as_ptr() as *const ::core::ffi::c_char,
                            );
                            break '_cleanup;
                        } else {
                            submatches = tv_get_bool(&raw mut (*di).di_tv) != 0;
                        }
                    }
                }
            }
            idx = 0 as ::core::ffi::c_int;
            let l_: *const list_T = l;
            if !l_.is_null() {
                let mut li: *const listitem_T = (*l_).lv_first;
                while !li.is_null() {
                    let li_tv: *const typval_T = &raw const (*li).li_tv;
                    if (*li_tv).v_type as ::core::ffi::c_uint
                        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                        && !(*li_tv).vval.v_string.is_null()
                    {
                        let mut str: *const ::core::ffi::c_char = (*li_tv).vval.v_string;
                        get_matches_in_str(str, &raw mut regmatch, retlist, idx, submatches, false);
                    }
                    idx += 1;
                    li = (*li).li_next;
                }
            }
        }
        vim_regfree(regmatch.regprog);
    }
    p_cpo.set(save_cpo);
}
unsafe extern "C" fn f_matchstrpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    find_some_match(argvars, rettv, kSomeMatchStrPos);
}
unsafe extern "C" fn max_min(tv: *const typval_T, rettv: *mut typval_T, domax: bool) {
    let mut error: bool = false_0 != 0;
    (*rettv).vval.v_number = 0 as varnumber_T;
    let mut n: varnumber_T = if domax as ::core::ffi::c_int != 0 {
        VARNUMBER_MIN as varnumber_T
    } else {
        VARNUMBER_MAX as varnumber_T
    };
    if (*tv).v_type as ::core::ffi::c_uint == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if tv_list_len((*tv).vval.v_list) == 0 as ::core::ffi::c_int {
            return;
        }
        let l_: *const list_T = (*tv).vval.v_list;
        if !l_.is_null() {
            let mut li: *const listitem_T = (*l_).lv_first;
            while !li.is_null() {
                let i: varnumber_T = tv_get_number_chk(&raw const (*li).li_tv, &raw mut error);
                if error {
                    return;
                }
                if if domax as ::core::ffi::c_int != 0 {
                    (i > n) as ::core::ffi::c_int
                } else {
                    (i < n) as ::core::ffi::c_int
                } != 0
                {
                    n = i;
                }
                li = (*li).li_next;
            }
        }
    } else if (*tv).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if tv_dict_len((*tv).vval.v_dict) == 0 as ::core::ffi::c_long {
            return;
        }
        let dihi_ht_: *mut hashtab_T = &raw mut (*(*tv).vval.v_dict).dv_hashtab;
        let mut dihi_todo_: size_t = (*dihi_ht_).ht_used;
        let mut dihi_: *mut hashitem_T = (*dihi_ht_).ht_array;
        while dihi_todo_ != 0 {
            if !((*dihi_).hi_key.is_null()
                || (*dihi_).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                dihi_todo_ = dihi_todo_.wrapping_sub(1);
                let di: *mut dictitem_T = (*dihi_)
                    .hi_key
                    .offset(-(17 as ::core::ffi::c_ulong as isize))
                    as *mut dictitem_T;
                let i_0: varnumber_T = tv_get_number_chk(&raw mut (*di).di_tv, &raw mut error);
                if error {
                    return;
                }
                if if domax as ::core::ffi::c_int != 0 {
                    (i_0 > n) as ::core::ffi::c_int
                } else {
                    (i_0 < n) as ::core::ffi::c_int
                } != 0
                {
                    n = i_0;
                }
            }
            dihi_ = dihi_.offset(1);
        }
    } else {
        semsg(
            gettext(&raw const e_listdictarg as *const ::core::ffi::c_char),
            if domax as ::core::ffi::c_int != 0 {
                b"max()\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"min()\0".as_ptr() as *const ::core::ffi::c_char
            },
        );
        return;
    }
    (*rettv).vval.v_number = n;
}
unsafe extern "C" fn f_max(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    max_min(argvars, rettv, true_0 != 0);
}
unsafe extern "C" fn f_min(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    max_min(argvars, rettv, false_0 != 0);
}
unsafe extern "C" fn f_mode(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut buf: [::core::ffi::c_char; 4] = [0; 4];
    get_mode(&raw mut buf as *mut ::core::ffi::c_char);
    if !non_zero_arg(argvars.offset(0 as ::core::ffi::c_int as isize)) {
        buf[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    }
    (*rettv).vval.v_string = xstrdup(&raw mut buf as *mut ::core::ffi::c_char);
    (*rettv).v_type = VAR_STRING;
}
unsafe extern "C" fn may_add_state_char(
    mut gap: *mut garray_T,
    mut include: *const ::core::ffi::c_char,
    mut c: uint8_t,
) {
    if include.is_null() || !vim_strchr(include, c as ::core::ffi::c_int).is_null() {
        ga_append(gap, c);
    }
}
unsafe extern "C" fn f_state(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut ga,
        1 as ::core::ffi::c_int,
        20 as ::core::ffi::c_int,
    );
    let mut include: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        include = tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    }
    if !(stuff_empty() as ::core::ffi::c_int != 0
        && (*typebuf.ptr()).tb_len == 0 as ::core::ffi::c_int
        && using_script() == 0)
    {
        may_add_state_char(&raw mut ga, include, 'm' as uint8_t);
    }
    if op_pending() {
        may_add_state_char(&raw mut ga, include, 'o' as uint8_t);
    }
    if autocmd_busy.get() {
        may_add_state_char(&raw mut ga, include, 'x' as uint8_t);
    }
    if ins_compl_active() {
        may_add_state_char(&raw mut ga, include, 'a' as uint8_t);
    }
    if !get_was_safe_state() {
        may_add_state_char(&raw mut ga, include, 'S' as uint8_t);
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < get_callback_depth() && i < 3 as ::core::ffi::c_int {
        may_add_state_char(&raw mut ga, include, 'c' as uint8_t);
        i += 1;
    }
    if msg_scrolled.get() > 0 as ::core::ffi::c_int {
        may_add_state_char(&raw mut ga, include, 's' as uint8_t);
    }
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ga.ga_data as *mut ::core::ffi::c_char;
}
unsafe extern "C" fn f_msgpackdump(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_listarg as *const ::core::ffi::c_char),
            b"msgpackdump()\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let list: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    let mut packer: PackerBuffer = packer_string_buffer();
    let msg: *const ::core::ffi::c_char =
        gettext(b"msgpackdump() argument, index %i\0".as_ptr() as *const ::core::ffi::c_char);
    let mut msgbuf: [::core::ffi::c_char; 189] = [0; 189];
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let l_: *mut list_T = list;
    if !l_.is_null() {
        let mut li: *mut listitem_T = (*l_).lv_first;
        while !li.is_null() {
            vim_snprintf(
                &raw mut msgbuf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 189]>(),
                msg,
                idx,
            );
            idx += 1;
            if encode_vim_to_msgpack(
                &raw mut packer,
                &raw mut (*li).li_tv,
                &raw mut msgbuf as *mut ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                break;
            }
            li = (*li).li_next;
        }
    }
    let mut data: String_0 = packer_take_string(&raw mut packer);
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && strequal(
            tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize)),
            b"B\0".as_ptr() as *const ::core::ffi::c_char,
        ) as ::core::ffi::c_int
            != 0
    {
        let mut b: *mut blob_T = tv_blob_alloc_ret(rettv);
        (*b).bv_ga.ga_data = data.data as *mut ::core::ffi::c_void;
        (*b).bv_ga.ga_len = data.size as ::core::ffi::c_int;
        (*b).bv_ga.ga_maxlen = packer.endptr.offset_from(packer.startptr) as ::core::ffi::c_int;
    } else {
        encode_list_write(
            tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t)
                as *mut ::core::ffi::c_void,
            data.data,
            data.size,
        );
        api_free_string(data);
    };
}
unsafe extern "C" fn emsg_mpack_error(mut status: ::core::ffi::c_int) {
    match status {
        2 => {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                b"Failed to parse msgpack string\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        1 => {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                b"Incomplete msgpack string\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        3 => {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                b"object was too deep to unpack\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        _ => {}
    };
}
unsafe extern "C" fn msgpackparse_unpack_list(list: *const list_T, ret_list: *mut list_T) {
    if tv_list_len(list) == 0 as ::core::ffi::c_int {
        return;
    }
    if (*tv_list_first(list)).li_tv.v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"List item is not a string\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut lrstate: ListReaderState = encode_init_lrstate(list);
    let mut buf: *mut ::core::ffi::c_char = alloc_block() as *mut ::core::ffi::c_char;
    let mut buf_size: size_t = 0 as size_t;
    let mut cur_item: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut parser: mpack_parser_t = mpack_parser_t {
        data: mpack_data_t {
            p: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        size: 0,
        capacity: 0,
        status: 0,
        exiting: 0,
        tokbuf: mpack_tokbuf_t {
            pending: [0; 9],
            pending_tok: mpack_token_t {
                type_0: 0 as mpack_token_type_t,
                length: 0,
                data: C2Rust_Unnamed_14 {
                    value: mpack_value_t { lo: 0, hi: 0 },
                },
            },
            ppos: 0,
            plen: 0,
            passthrough: 0,
        },
        items: [mpack_node_t {
            tok: mpack_token_t {
                type_0: 0 as mpack_token_type_t,
                length: 0,
                data: C2Rust_Unnamed_14 {
                    value: mpack_value_t { lo: 0, hi: 0 },
                },
            },
            pos: 0,
            key_visited: 0,
            data: [mpack_data_t {
                p: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            }; 2],
        }; 33],
    };
    mpack_parser_init(&raw mut parser, 0 as mpack_uint32_t);
    parser.data.p = &raw mut cur_item as *mut ::core::ffi::c_void;
    let mut status: ::core::ffi::c_int = MPACK_OK as ::core::ffi::c_int;
    '_end: {
        loop {
            let mut read_bytes: size_t = 0;
            let rlret: ::core::ffi::c_int = encode_read_from_list(
                &raw mut lrstate,
                buf.offset(buf_size as isize),
                (ARENA_BLOCK_SIZE as size_t).wrapping_sub(buf_size),
                &raw mut read_bytes,
            );
            if rlret == FAIL {
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    b"List item is not a string\0".as_ptr() as *const ::core::ffi::c_char,
                );
                break '_end;
            } else {
                buf_size = buf_size.wrapping_add(read_bytes);
                let mut ptr: *const ::core::ffi::c_char = buf;
                while buf_size != 0 {
                    status = mpack_parse_typval(&raw mut parser, &raw mut ptr, &raw mut buf_size);
                    if status != MPACK_OK as ::core::ffi::c_int {
                        break;
                    }
                    tv_list_append_owned_tv(ret_list, cur_item);
                    cur_item.v_type = VAR_UNKNOWN;
                }
                if rlret == OK {
                    break;
                }
                if status == MPACK_EOF as ::core::ffi::c_int {
                    if buf_size != 0 && ptr > buf as *const ::core::ffi::c_char {
                        memmove(
                            buf as *mut ::core::ffi::c_void,
                            ptr as *const ::core::ffi::c_void,
                            buf_size,
                        );
                    }
                } else if status != MPACK_OK as ::core::ffi::c_int {
                    break;
                }
            }
        }
        if status != MPACK_OK as ::core::ffi::c_int {
            typval_parser_error_free(&raw mut parser);
            emsg_mpack_error(status);
        }
    }
    free_block(buf as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn msgpackparse_unpack_blob(blob: *const blob_T, ret_list: *mut list_T) {
    let len: ::core::ffi::c_int = tv_blob_len(blob);
    if len == 0 as ::core::ffi::c_int {
        return;
    }
    let mut data: *const ::core::ffi::c_char = (*blob).bv_ga.ga_data as *const ::core::ffi::c_char;
    let mut remaining: size_t = len as size_t;
    while remaining != 0 {
        let mut tv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut status: ::core::ffi::c_int =
            unpack_typval(&raw mut data, &raw mut remaining, &raw mut tv);
        if status != MPACK_OK as ::core::ffi::c_int {
            emsg_mpack_error(status);
            return;
        }
        tv_list_append_owned_tv(ret_list, tv);
    }
}
unsafe extern "C" fn f_msgpackparse(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_listblobarg as *const ::core::ffi::c_char),
            b"msgpackparse()\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let ret_list: *mut list_T =
        tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        msgpackparse_unpack_list(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_list,
            ret_list,
        );
    } else {
        msgpackparse_unpack_blob(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_blob,
            ret_list,
        );
    };
}
unsafe extern "C" fn f_nextnonblank(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut lnum: linenr_T = 0;
    lnum = tv_get_lnum(argvars);
    loop {
        if lnum < 0 as linenr_T || lnum > (*curbuf.get()).b_ml.ml_line_count {
            lnum = 0 as ::core::ffi::c_int as linenr_T;
            break;
        } else {
            if *skipwhite(ml_get(lnum)) as ::core::ffi::c_int != NUL {
                break;
            }
            lnum += 1;
        }
    }
    (*rettv).vval.v_number = lnum as varnumber_T;
}
unsafe extern "C" fn f_nr2char(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !tv_check_num(argvars.offset(1 as ::core::ffi::c_int as isize)) {
            return;
        }
    }
    let mut error: bool = false_0 != 0;
    let num: varnumber_T = tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut error,
    );
    if error {
        return;
    }
    if num < 0 as varnumber_T {
        emsg(gettext(
            b"E5070: Character number must not be less than zero\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
        return;
    }
    if num > INT_MAX as varnumber_T {
        semsg(
            gettext(
                b"E5071: Character number must not be greater than INT_MAX (%i)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            INT_MAX,
        );
        return;
    }
    let mut buf: [::core::ffi::c_char; 6] = [0; 6];
    let len: ::core::ffi::c_int = utf_char2bytes(
        num as ::core::ffi::c_int,
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = xmemdupz(
        &raw mut buf as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
        len as size_t,
    ) as *mut ::core::ffi::c_char;
}
unsafe extern "C" fn f_or(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) | tv_get_number_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    );
}
unsafe extern "C" fn f_pow(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut fx: float_T = 0.;
    let mut fy: float_T = 0.;
    (*rettv).v_type = VAR_FLOAT;
    if tv_get_float_chk(argvars, &raw mut fx) as ::core::ffi::c_int != 0
        && tv_get_float_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut fy,
        ) as ::core::ffi::c_int
            != 0
    {
        (*rettv).vval.v_float =
            pow(fx as ::core::ffi::c_double, fy as ::core::ffi::c_double) as float_T;
    } else {
        (*rettv).vval.v_float = 0.0f64 as float_T;
    };
}
unsafe extern "C" fn f_prevnonblank(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut lnum: linenr_T = tv_get_lnum(argvars);
    if lnum < 1 as linenr_T || lnum > (*curbuf.get()).b_ml.ml_line_count {
        lnum = 0 as ::core::ffi::c_int as linenr_T;
    } else {
        while lnum >= 1 as linenr_T && *skipwhite(ml_get(lnum)) as ::core::ffi::c_int == NUL {
            lnum -= 1;
        }
    }
    (*rettv).vval.v_number = lnum as varnumber_T;
}
unsafe extern "C" fn f_printf(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut saved_did_emsg: ::core::ffi::c_int = did_emsg.get();
    did_emsg.set(false_0);
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    let mut fmt: *const ::core::ffi::c_char = tv_get_string_buf(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    let mut len: ::core::ffi::c_int = vim_vsnprintf_typval(
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        0 as size_t,
        fmt,
        (*dummy_ap.ptr()).as_va_list(),
        argvars.offset(1 as ::core::ffi::c_int as isize),
    );
    if did_emsg.get() == 0 {
        let mut s: *mut ::core::ffi::c_char =
            xmalloc((len as size_t).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        (*rettv).vval.v_string = s;
        vim_vsnprintf_typval(
            s,
            (len as size_t).wrapping_add(1 as size_t),
            fmt,
            (*dummy_ap.ptr()).as_va_list(),
            argvars.offset(1 as ::core::ffi::c_int as isize),
        );
    }
    (*did_emsg.ptr()) |= saved_did_emsg;
}
unsafe extern "C" fn f_prompt_getprompt(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let buf: *mut buf_T = tv_get_buf_from_arg(argvars.offset(0 as ::core::ffi::c_int as isize));
    if buf.is_null() {
        return;
    }
    if !bt_prompt(buf) {
        return;
    }
    (*rettv).vval.v_string = xstrdup(buf_prompt_text(buf));
}
unsafe extern "C" fn f_prompt_getinput(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let buf: *mut buf_T = tv_get_buf_from_arg(argvars.offset(0 as ::core::ffi::c_int as isize));
    if buf.is_null() {
        return;
    }
    if !bt_prompt(buf) {
        return;
    }
    (*rettv).vval.v_string = prompt_get_input(buf);
}
unsafe extern "C" fn f_pum_getpos(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_dict_alloc_ret(rettv);
    pum_set_event_info((*rettv).vval.v_dict);
}
unsafe extern "C" fn f_pumvisible(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if pum_visible() {
        (*rettv).vval.v_number = 1 as varnumber_T;
    }
}
unsafe extern "C" fn f_py3eval(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    script_host_eval(
        b"python3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        argvars,
        rettv,
    );
}
unsafe extern "C" fn init_srand(x: *mut uint32_t) {
    let mut buf: C2Rust_Unnamed_52 = C2Rust_Unnamed_52 { number: 0 };
    if uv_random(
        ::core::ptr::null_mut::<uv_loop_t>(),
        ::core::ptr::null_mut::<uv_random_t>(),
        &raw mut buf.bytes as *mut uint8_t as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<[uint8_t; 4]>(),
        0 as ::core::ffi::c_uint,
        None,
    ) == 0 as ::core::ffi::c_int
    {
        *x = buf.number;
        return;
    }
    *x = os_hrtime() as uint32_t;
    *x ^= os_get_pid() as uint32_t;
}
#[inline(always)]
unsafe extern "C" fn splitmix32(x: *mut uint32_t) -> uint32_t {
    *x = (*x as ::core::ffi::c_uint).wrapping_add(0x9e3779b9 as ::core::ffi::c_uint) as uint32_t;
    let mut z: uint32_t = *x;
    z = (z ^ z >> 16 as ::core::ffi::c_int).wrapping_mul(0x85ebca6b as uint32_t);
    z = (z ^ z >> 13 as ::core::ffi::c_int).wrapping_mul(0xc2b2ae35 as uint32_t);
    return z ^ z >> 16 as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn shuffle_xoshiro128starstar(
    x: *mut uint32_t,
    y: *mut uint32_t,
    z: *mut uint32_t,
    w: *mut uint32_t,
) -> uint32_t {
    let result: uint32_t = ((*y).wrapping_mul(5 as uint32_t) << 7 as ::core::ffi::c_int
        | (*y).wrapping_mul(5 as uint32_t) >> 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
        .wrapping_mul(9 as uint32_t);
    let t: uint32_t = *y << 9 as ::core::ffi::c_int;
    *z ^= *x;
    *w ^= *y;
    *y ^= *z;
    *x ^= *w;
    *z ^= t;
    *w = *w << 11 as ::core::ffi::c_int | *w >> 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int;
    return result;
}
unsafe extern "C" fn f_rand(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut result: uint32_t = 0;
    's_126: {
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            static gx: GlobalCell<uint32_t> = GlobalCell::new(0);
            static gy: GlobalCell<uint32_t> = GlobalCell::new(0);
            static gz: GlobalCell<uint32_t> = GlobalCell::new(0);
            static gw: GlobalCell<uint32_t> = GlobalCell::new(0);
            static initialized: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
            if !initialized.get() {
                let mut x: uint32_t = 0 as uint32_t;
                init_srand(&raw mut x);
                gx.set(splitmix32(&raw mut x));
                gy.set(splitmix32(&raw mut x));
                gz.set(splitmix32(&raw mut x));
                gw.set(splitmix32(&raw mut x));
                initialized.set(true_0 != 0);
            }
            result = shuffle_xoshiro128starstar(gx.ptr(), gy.ptr(), gz.ptr(), gw.ptr());
        } else {
            if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_list;
                if tv_list_len(l) == 4 as ::core::ffi::c_int {
                    let tvx: *mut typval_T = &raw mut (*(tv_list_find
                        as unsafe extern "C" fn(
                            *mut list_T,
                            ::core::ffi::c_int,
                        ) -> *mut listitem_T)(
                        l, 0 as ::core::ffi::c_int
                    ))
                    .li_tv;
                    let tvy: *mut typval_T = &raw mut (*(tv_list_find
                        as unsafe extern "C" fn(
                            *mut list_T,
                            ::core::ffi::c_int,
                        ) -> *mut listitem_T)(
                        l, 1 as ::core::ffi::c_int
                    ))
                    .li_tv;
                    let tvz: *mut typval_T = &raw mut (*(tv_list_find
                        as unsafe extern "C" fn(
                            *mut list_T,
                            ::core::ffi::c_int,
                        ) -> *mut listitem_T)(
                        l, 2 as ::core::ffi::c_int
                    ))
                    .li_tv;
                    let tvw: *mut typval_T = &raw mut (*(tv_list_find
                        as unsafe extern "C" fn(
                            *mut list_T,
                            ::core::ffi::c_int,
                        ) -> *mut listitem_T)(
                        l, 3 as ::core::ffi::c_int
                    ))
                    .li_tv;
                    if (*tvx).v_type as ::core::ffi::c_uint
                        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        if (*tvy).v_type as ::core::ffi::c_uint
                            == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            if (*tvz).v_type as ::core::ffi::c_uint
                                == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                if (*tvw).v_type as ::core::ffi::c_uint
                                    == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                                {
                                    let mut x_0: uint32_t = (*tvx).vval.v_number as uint32_t;
                                    let mut y: uint32_t = (*tvy).vval.v_number as uint32_t;
                                    let mut z: uint32_t = (*tvz).vval.v_number as uint32_t;
                                    let mut w: uint32_t = (*tvw).vval.v_number as uint32_t;
                                    result = shuffle_xoshiro128starstar(
                                        &raw mut x_0,
                                        &raw mut y,
                                        &raw mut z,
                                        &raw mut w,
                                    );
                                    (*tvx).vval.v_number = x_0 as varnumber_T;
                                    (*tvy).vval.v_number = y as varnumber_T;
                                    (*tvz).vval.v_number = z as varnumber_T;
                                    (*tvw).vval.v_number = w as varnumber_T;
                                    break 's_126;
                                }
                            }
                        }
                    }
                }
            }
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
            );
            (*rettv).v_type = VAR_NUMBER;
            (*rettv).vval.v_number = -1 as varnumber_T;
            return;
        }
    }
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = result as varnumber_T;
}
unsafe extern "C" fn f_srand(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut x: uint32_t = 0 as uint32_t;
    tv_list_alloc_ret(rettv, 4 as ptrdiff_t);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        init_srand(&raw mut x);
    } else {
        let mut error: bool = false_0 != 0;
        x = tv_get_number_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) as uint32_t;
        if error {
            return;
        }
    }
    tv_list_append_number((*rettv).vval.v_list, splitmix32(&raw mut x) as varnumber_T);
    tv_list_append_number((*rettv).vval.v_list, splitmix32(&raw mut x) as varnumber_T);
    tv_list_append_number((*rettv).vval.v_list, splitmix32(&raw mut x) as varnumber_T);
    tv_list_append_number((*rettv).vval.v_list, splitmix32(&raw mut x) as varnumber_T);
}
unsafe extern "C" fn f_perleval(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    script_host_eval(
        b"perl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        argvars,
        rettv,
    );
}
unsafe extern "C" fn f_rubyeval(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    script_host_eval(
        b"ruby\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        argvars,
        rettv,
    );
}
unsafe extern "C" fn f_range(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut end: varnumber_T = 0;
    let mut stride: varnumber_T = 1 as varnumber_T;
    let mut error: bool = false_0 != 0;
    let mut start: varnumber_T = tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut error,
    );
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        end = start - 1 as varnumber_T;
        start = 0 as varnumber_T;
    } else {
        end = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        );
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            stride = tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error,
            );
        }
    }
    if error {
        return;
    }
    if stride == 0 as varnumber_T {
        emsg(gettext(
            b"E726: Stride is zero\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return;
    }
    if if stride > 0 as varnumber_T {
        ((end + 1 as varnumber_T) < start) as ::core::ffi::c_int
    } else {
        (end - 1 as varnumber_T > start) as ::core::ffi::c_int
    } != 0
    {
        emsg(gettext(
            b"E727: Start past end\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return;
    }
    tv_list_alloc_ret(
        rettv,
        (end as ptrdiff_t - start as ptrdiff_t) / stride as ptrdiff_t,
    );
    let mut i: varnumber_T = start;
    while if stride > 0 as varnumber_T {
        (i <= end) as ::core::ffi::c_int
    } else {
        (i >= end) as ::core::ffi::c_int
    } != 0
    {
        tv_list_append_number((*rettv).vval.v_list, i);
        i += stride;
    }
}
unsafe extern "C" fn f_getreginfo(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut regname: ::core::ffi::c_int = getreg_get_regname(argvars);
    if regname == 0 as ::core::ffi::c_int {
        return;
    }
    if regname == '@' as ::core::ffi::c_int {
        regname = '"' as ::core::ffi::c_int;
    }
    tv_dict_alloc_ret(rettv);
    let dict: *mut dict_T = (*rettv).vval.v_dict;
    let list: *mut list_T = get_reg_contents(
        regname,
        kGRegExprSrc as ::core::ffi::c_int | kGRegList as ::core::ffi::c_int,
    ) as *mut list_T;
    if list.is_null() {
        return;
    }
    tv_dict_add_list(
        dict,
        b"regcontents\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
        list,
    );
    let mut buf: [::core::ffi::c_char; 67] = [0; 67];
    buf[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    buf[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    let mut reglen: colnr_T = 0 as colnr_T;
    match get_reg_type(regname, &raw mut reglen) as ::core::ffi::c_int {
        1 => {
            buf[0 as ::core::ffi::c_int as usize] = 'V' as ::core::ffi::c_char;
        }
        0 => {
            buf[0 as ::core::ffi::c_int as usize] = 'v' as ::core::ffi::c_char;
        }
        2 => {
            vim_snprintf(
                &raw mut buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 67]>(),
                b"%c%d\0".as_ptr() as *const ::core::ffi::c_char,
                Ctrl_V,
                reglen as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
            );
        }
        -1 => {
            abort();
        }
        _ => {}
    }
    tv_dict_add_str(
        dict,
        b"regtype\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    buf[0 as ::core::ffi::c_int as usize] =
        get_register_name(get_unname_register()) as ::core::ffi::c_char;
    buf[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    if regname == '"' as ::core::ffi::c_int {
        tv_dict_add_str(
            dict,
            b"points_to\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
    } else {
        tv_dict_add_bool(
            dict,
            b"isunnamed\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            (if regname == buf[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int {
                kBoolVarTrue as ::core::ffi::c_int
            } else {
                kBoolVarFalse as ::core::ffi::c_int
            }) as BoolVarValue,
        );
    };
}
unsafe extern "C" fn return_register(mut regname: ::core::ffi::c_int, mut rettv: *mut typval_T) {
    let mut buf: [::core::ffi::c_char; 2] =
        [regname as ::core::ffi::c_char, 0 as ::core::ffi::c_char];
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = xstrdup(&raw mut buf as *mut ::core::ffi::c_char);
}
unsafe extern "C" fn f_reg_executing(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    return_register(reg_executing.get(), rettv);
}
unsafe extern "C" fn f_reg_recording(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    return_register(reg_recording.get(), rettv);
}
unsafe extern "C" fn f_reg_recorded(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    return_register(reg_recorded.get(), rettv);
}
unsafe extern "C" fn list2proftime(
    mut arg: *mut typval_T,
    mut tm: *mut proftime_T,
) -> ::core::ffi::c_int {
    if (*arg).v_type as ::core::ffi::c_uint != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        || tv_list_len((*arg).vval.v_list) != 2 as ::core::ffi::c_int
    {
        return FAIL;
    }
    let mut error: bool = false_0 != 0;
    let mut n1: varnumber_T =
        tv_list_find_nr((*arg).vval.v_list, 0 as ::core::ffi::c_int, &raw mut error);
    let mut n2: varnumber_T =
        tv_list_find_nr((*arg).vval.v_list, 1 as ::core::ffi::c_int, &raw mut error);
    if error {
        return FAIL;
    }
    let mut u: C2Rust_Unnamed_47 = C2Rust_Unnamed_47 {
        split: C2Rust_Unnamed_48 {
            low: n2 as int32_t,
            high: n1 as int32_t,
        },
    };
    *tm = u.prof;
    return OK;
}
unsafe extern "C" fn f_reltime(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut res: proftime_T = 0;
    let mut start: proftime_T = 0;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        res = profile_start();
    } else if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if list2proftime(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut res,
        ) == FAIL
        {
            return;
        }
        res = profile_end(res);
    } else {
        if list2proftime(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut start,
        ) == FAIL
            || list2proftime(
                argvars.offset(1 as ::core::ffi::c_int as isize),
                &raw mut res,
            ) == FAIL
        {
            return;
        }
        res = profile_sub(res, start);
    }
    let mut u: C2Rust_Unnamed_51 = C2Rust_Unnamed_51 { prof: res };
    tv_list_alloc_ret(rettv, 2 as ptrdiff_t);
    tv_list_append_number((*rettv).vval.v_list, u.split.high as varnumber_T);
    tv_list_append_number((*rettv).vval.v_list, u.split.low as varnumber_T);
}
unsafe extern "C" fn f_reltimestr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut tm: proftime_T = 0;
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if list2proftime(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut tm,
    ) == OK
    {
        (*rettv).vval.v_string = xstrdup(profile_msg(tm));
    }
}
unsafe extern "C" fn f_repeat(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut n: varnumber_T = tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize));
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_list_alloc_ret(
            rettv,
            (n > 0 as varnumber_T) as ::core::ffi::c_int as ptrdiff_t
                * n as ptrdiff_t
                * tv_list_len(
                    (*argvars.offset(0 as ::core::ffi::c_int as isize))
                        .vval
                        .v_list,
                ) as ptrdiff_t,
        );
        loop {
            let c2rust_fresh8 = n;
            n = n - 1;
            if c2rust_fresh8 <= 0 as varnumber_T {
                break;
            }
            tv_list_extend(
                (*rettv).vval.v_list,
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_list,
                ::core::ptr::null_mut::<listitem_T>(),
            );
        }
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_blob_alloc_ret(rettv);
        if (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_blob
            .is_null()
            || n <= 0 as varnumber_T
        {
            return;
        }
        let slen: ::core::ffi::c_int = (*(*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_blob)
            .bv_ga
            .ga_len;
        let len: ::core::ffi::c_int = (slen as varnumber_T * n) as ::core::ffi::c_int;
        if len <= 0 as ::core::ffi::c_int {
            return;
        }
        ga_grow(&raw mut (*(*rettv).vval.v_blob).bv_ga, len);
        (*(*rettv).vval.v_blob).bv_ga.ga_len = len;
        let mut i: ::core::ffi::c_int = 0;
        i = 0 as ::core::ffi::c_int;
        while i < slen {
            if tv_blob_get(
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_blob,
                i,
            ) as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                break;
            }
            i += 1;
        }
        if i == slen {
            return;
        }
        i = 0 as ::core::ffi::c_int;
        while (i as varnumber_T) < n {
            tv_blob_set_range(
                (*rettv).vval.v_blob,
                (i * slen) as varnumber_T,
                ((i + 1 as ::core::ffi::c_int) * slen - 1 as ::core::ffi::c_int) as varnumber_T,
                argvars,
            );
            i += 1;
        }
    } else {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if n <= 0 as varnumber_T {
            return;
        }
        let p: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        let slen_0: size_t = strlen(p);
        if slen_0 == 0 as size_t {
            return;
        }
        let len_0: size_t = slen_0.wrapping_mul(n as size_t);
        if len_0.wrapping_div(n as size_t) != slen_0 {
            return;
        }
        let r: *mut ::core::ffi::c_char = xmallocz(len_0) as *mut ::core::ffi::c_char;
        let mut i_0: varnumber_T = 0 as varnumber_T;
        while i_0 < n {
            memmove(
                r.offset((i_0 as size_t).wrapping_mul(slen_0) as isize) as *mut ::core::ffi::c_void,
                p as *const ::core::ffi::c_void,
                slen_0,
            );
            i_0 += 1;
        }
        (*rettv).vval.v_string = r;
    };
}
unsafe extern "C" fn reduce_list(
    mut argvars: *mut typval_T,
    mut expr: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    let l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    let called_emsg_start: ::core::ffi::c_int = called_emsg.get();
    let mut initial: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut li: *const listitem_T = ::core::ptr::null::<listitem_T>();
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if tv_list_len(l) == 0 as ::core::ffi::c_int {
            semsg(
                gettext(
                    &raw const e_reduce_of_an_empty_str_with_no_initial_value
                        as *const ::core::ffi::c_char,
                ),
                b"List\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return;
        }
        let first: *const listitem_T = tv_list_first(l);
        initial = (*first).li_tv;
        li = (*first).li_next;
    } else {
        initial = *argvars.offset(2 as ::core::ffi::c_int as isize);
        li = tv_list_first(l);
    }
    tv_copy(&raw mut initial, rettv);
    if l.is_null() {
        return;
    }
    let prev_locked: VarLockStatus = tv_list_locked(l);
    tv_list_set_lock(l, VAR_FIXED);
    while !li.is_null() {
        let mut argv: [typval_T; 3] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 3];
        argv[0 as ::core::ffi::c_int as usize] = *rettv;
        argv[1 as ::core::ffi::c_int as usize] = (*li).li_tv;
        (*rettv).v_type = VAR_UNKNOWN;
        let r: ::core::ffi::c_int = eval_expr_typval(
            expr,
            true_0 != 0,
            &raw mut argv as *mut typval_T,
            2 as ::core::ffi::c_int,
            rettv,
        );
        tv_clear((&raw mut argv as *mut typval_T).offset(0 as ::core::ffi::c_int as isize));
        if r == FAIL || called_emsg.get() != called_emsg_start {
            break;
        }
        li = (*li).li_next;
    }
    tv_list_set_lock(l, prev_locked);
}
unsafe extern "C" fn reduce_string(
    mut argvars: *mut typval_T,
    mut expr: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    let mut p: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut len: ::core::ffi::c_int = 0;
    let called_emsg_start: ::core::ffi::c_int = called_emsg.get();
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if *p as ::core::ffi::c_int == NUL {
            semsg(
                gettext(
                    &raw const e_reduce_of_an_empty_str_with_no_initial_value
                        as *const ::core::ffi::c_char,
                ),
                b"String\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return;
        }
        len = utfc_ptr2len(p);
        *rettv = typval_T {
            v_type: VAR_STRING,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_string: xmemdupz(p as *const ::core::ffi::c_void, len as size_t)
                    as *mut ::core::ffi::c_char,
            },
        };
        p = p.offset(len as isize);
    } else if tv_check_for_string_arg(argvars, 2 as ::core::ffi::c_int) == FAIL {
        return;
    } else {
        tv_copy(argvars.offset(2 as ::core::ffi::c_int as isize), rettv);
    }
    while *p as ::core::ffi::c_int != NUL {
        let mut argv: [typval_T; 3] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 3];
        argv[0 as ::core::ffi::c_int as usize] = *rettv;
        len = utfc_ptr2len(p);
        argv[1 as ::core::ffi::c_int as usize] = typval_T {
            v_type: VAR_STRING,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_string: xmemdupz(p as *const ::core::ffi::c_void, len as size_t)
                    as *mut ::core::ffi::c_char,
            },
        };
        let r: ::core::ffi::c_int = eval_expr_typval(
            expr,
            true_0 != 0,
            &raw mut argv as *mut typval_T,
            2 as ::core::ffi::c_int,
            rettv,
        );
        tv_clear((&raw mut argv as *mut typval_T).offset(0 as ::core::ffi::c_int as isize));
        tv_clear((&raw mut argv as *mut typval_T).offset(1 as ::core::ffi::c_int as isize));
        if r == FAIL || called_emsg.get() != called_emsg_start {
            break;
        }
        p = p.offset(len as isize);
    }
}
unsafe extern "C" fn reduce_blob(
    mut argvars: *mut typval_T,
    mut expr: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    let b: *const blob_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_blob;
    let called_emsg_start: ::core::ffi::c_int = called_emsg.get();
    let mut initial: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut i: ::core::ffi::c_int = 0;
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if tv_blob_len(b) == 0 as ::core::ffi::c_int {
            semsg(
                gettext(
                    &raw const e_reduce_of_an_empty_str_with_no_initial_value
                        as *const ::core::ffi::c_char,
                ),
                b"Blob\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return;
        }
        initial = typval_T {
            v_type: VAR_NUMBER,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_number: tv_blob_get(b, 0 as ::core::ffi::c_int) as varnumber_T,
            },
        };
        i = 1 as ::core::ffi::c_int;
    } else if tv_check_for_number_arg(argvars, 2 as ::core::ffi::c_int) == FAIL {
        return;
    } else {
        initial = *argvars.offset(2 as ::core::ffi::c_int as isize);
        i = 0 as ::core::ffi::c_int;
    }
    tv_copy(&raw mut initial, rettv);
    while i < tv_blob_len(b) {
        let mut argv: [typval_T; 3] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 3];
        argv[0 as ::core::ffi::c_int as usize] = *rettv;
        argv[1 as ::core::ffi::c_int as usize] = typval_T {
            v_type: VAR_NUMBER,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_number: tv_blob_get(b, i) as varnumber_T,
            },
        };
        let r: ::core::ffi::c_int = eval_expr_typval(
            expr,
            true_0 != 0,
            &raw mut argv as *mut typval_T,
            2 as ::core::ffi::c_int,
            rettv,
        );
        if r == FAIL || called_emsg.get() != called_emsg_start {
            return;
        }
        i += 1;
    }
}
unsafe extern "C" fn f_reduce(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(
            (e_string_list_or_blob_required.ptr() as *const _) as *const ::core::ffi::c_char,
        ));
        return;
    }
    let mut func_name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        func_name = (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_string;
    } else if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        func_name = partial_name(
            (*argvars.offset(1 as ::core::ffi::c_int as isize))
                .vval
                .v_partial,
        );
    } else {
        func_name = tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
    }
    if func_name.is_null() || *func_name as ::core::ffi::c_int == NUL {
        emsg(gettext(
            (e_missing_function_argument.ptr() as *const _) as *const ::core::ffi::c_char,
        ));
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        reduce_list(
            argvars,
            argvars.offset(1 as ::core::ffi::c_int as isize),
            rettv,
        );
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        reduce_string(
            argvars,
            argvars.offset(1 as ::core::ffi::c_int as isize),
            rettv,
        );
    } else {
        reduce_blob(
            argvars,
            argvars.offset(1 as ::core::ffi::c_int as isize),
            rettv,
        );
    };
}
pub const SP_NOMOVE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const SP_REPEAT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const SP_RETCOUNT: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const SP_SETPCMARK: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const SP_START: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const SP_SUBPAT: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const SP_END: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const SP_COLUMN: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
unsafe extern "C" fn get_search_arg(
    mut varp: *mut typval_T,
    mut flagsp: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut dir: ::core::ffi::c_int = FORWARD as ::core::ffi::c_int;
    if (*varp).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return FORWARD as ::core::ffi::c_int;
    }
    let mut nbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut flags: *const ::core::ffi::c_char =
        tv_get_string_buf_chk(varp, &raw mut nbuf as *mut ::core::ffi::c_char);
    if flags.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    let mut mask: ::core::ffi::c_int = 0;
    while *flags as ::core::ffi::c_int != NUL {
        match *flags as ::core::ffi::c_int {
            98 => {
                dir = BACKWARD as ::core::ffi::c_int;
            }
            119 => {
                p_ws.set(true_0);
            }
            87 => {
                p_ws.set(false_0);
            }
            _ => {
                mask = 0 as ::core::ffi::c_int;
                if !flagsp.is_null() {
                    match *flags as ::core::ffi::c_int {
                        99 => {
                            mask = SP_START;
                        }
                        101 => {
                            mask = SP_END;
                        }
                        109 => {
                            mask = SP_RETCOUNT;
                        }
                        110 => {
                            mask = SP_NOMOVE;
                        }
                        112 => {
                            mask = SP_SUBPAT;
                        }
                        114 => {
                            mask = SP_REPEAT;
                        }
                        115 => {
                            mask = SP_SETPCMARK;
                        }
                        122 => {
                            mask = SP_COLUMN;
                        }
                        _ => {}
                    }
                }
                if mask == 0 as ::core::ffi::c_int {
                    semsg(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        flags,
                    );
                    dir = 0 as ::core::ffi::c_int;
                } else {
                    *flagsp |= mask;
                }
            }
        }
        if dir == 0 as ::core::ffi::c_int {
            break;
        }
        flags = flags.offset(1);
    }
    return dir;
}
unsafe extern "C" fn search_cmn(
    mut argvars: *mut typval_T,
    mut match_pos: *mut pos_T,
    mut flagsp: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut flags: ::core::ffi::c_int = 0;
    let mut tm: proftime_T = 0;
    let mut save_cursor: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut firstpos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut sia: searchit_arg_T = searchit_arg_T {
        sa_stop_lnum: 0,
        sa_tm: ::core::ptr::null_mut::<proftime_T>(),
        sa_timed_out: 0,
        sa_wrapped: 0,
    };
    let mut patlen: size_t = 0;
    let mut subpatnum: ::core::ffi::c_int = 0;
    let mut save_p_ws: bool = p_ws.get() != 0;
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut lnum_stop: linenr_T = 0 as linenr_T;
    let mut time_limit: int64_t = 0 as int64_t;
    let mut options: ::core::ffi::c_int = SEARCH_KEEP as ::core::ffi::c_int;
    let mut use_skip: bool = false_0 != 0;
    let pat: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut dir: ::core::ffi::c_int =
        get_search_arg(argvars.offset(1 as ::core::ffi::c_int as isize), flagsp);
    '_theend: {
        if dir != 0 as ::core::ffi::c_int {
            flags = *flagsp;
            if flags & SP_START != 0 {
                options |= SEARCH_START as ::core::ffi::c_int;
            }
            if flags & SP_END != 0 {
                options |= SEARCH_END as ::core::ffi::c_int;
            }
            if flags & SP_COLUMN != 0 {
                options |= SEARCH_COL as ::core::ffi::c_int;
            }
            if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                lnum_stop = tv_get_number_chk(
                    argvars.offset(2 as ::core::ffi::c_int as isize),
                    ::core::ptr::null_mut::<bool>(),
                ) as linenr_T;
                if lnum_stop < 0 as linenr_T {
                    break '_theend;
                } else if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type
                    as ::core::ffi::c_uint
                    != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    time_limit = tv_get_number_chk(
                        argvars.offset(3 as ::core::ffi::c_int as isize),
                        ::core::ptr::null_mut::<bool>(),
                    ) as int64_t;
                    if time_limit < 0 as int64_t {
                        break '_theend;
                    } else {
                        use_skip =
                            eval_expr_valid_arg(argvars.offset(4 as ::core::ffi::c_int as isize));
                    }
                }
            }
            tm = profile_setlimit(time_limit);
            if flags & (SP_REPEAT | SP_RETCOUNT) != 0 as ::core::ffi::c_int
                || flags & SP_NOMOVE != 0 && flags & SP_SETPCMARK != 0
            {
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize)),
                );
            } else {
                save_cursor = pos_T {
                    lnum: 0,
                    col: 0,
                    coladd: 0,
                };
                save_cursor = (*curwin.get()).w_cursor;
                pos = save_cursor;
                firstpos = pos_T {
                    lnum: 0 as linenr_T,
                    col: 0,
                    coladd: 0,
                };
                sia = searchit_arg_T {
                    sa_stop_lnum: lnum_stop,
                    sa_tm: &raw mut tm,
                    sa_timed_out: 0,
                    sa_wrapped: 0,
                };
                patlen = strlen(pat);
                subpatnum = 0;
                loop {
                    subpatnum = searchit(
                        curwin.get(),
                        curbuf.get(),
                        &raw mut pos,
                        ::core::ptr::null_mut::<pos_T>(),
                        dir as Direction,
                        pat as *mut ::core::ffi::c_char,
                        patlen,
                        1 as ::core::ffi::c_int,
                        options,
                        RE_SEARCH as ::core::ffi::c_int,
                        &raw mut sia,
                    );
                    if firstpos.lnum != 0 as linenr_T
                        && equalpos(pos, firstpos) as ::core::ffi::c_int != 0
                    {
                        subpatnum = FAIL;
                    }
                    if subpatnum == FAIL || !use_skip {
                        break;
                    }
                    if firstpos.lnum == 0 as linenr_T {
                        firstpos = pos;
                    }
                    let save_pos: pos_T = (*curwin.get()).w_cursor;
                    (*curwin.get()).w_cursor = pos;
                    let mut err: bool = false_0 != 0;
                    let do_skip: bool = eval_expr_to_bool(
                        argvars.offset(4 as ::core::ffi::c_int as isize),
                        &raw mut err,
                    );
                    (*curwin.get()).w_cursor = save_pos;
                    if err {
                        subpatnum = FAIL;
                        break;
                    } else {
                        if !do_skip {
                            break;
                        }
                        options &= !(SEARCH_START as ::core::ffi::c_int);
                    }
                }
                if subpatnum != FAIL {
                    if flags & SP_SUBPAT != 0 {
                        retval = subpatnum;
                    } else {
                        retval = pos.lnum as ::core::ffi::c_int;
                    }
                    if flags & SP_SETPCMARK != 0 {
                        setpcmark();
                    }
                    (*curwin.get()).w_cursor = pos;
                    if !match_pos.is_null() {
                        (*match_pos).lnum = pos.lnum;
                        (*match_pos).col =
                            (pos.col as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as colnr_T;
                    }
                    check_cursor(curwin.get());
                }
                if flags & SP_NOMOVE != 0 {
                    (*curwin.get()).w_cursor = save_cursor;
                } else {
                    (*curwin.get()).w_set_curswant = true_0;
                }
            }
        }
    }
    p_ws.set(save_p_ws as ::core::ffi::c_int);
    return retval;
}
unsafe extern "C" fn f_rpcnotify(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_number
            < 0 as varnumber_T
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"Channel id must be a positive integer\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"Event type must be a string\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 20] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_16 { boolean: false },
    }; 20];
    args.capacity = MAX_FUNC_ARGS as ::core::ffi::c_int as size_t;
    args.items = &raw mut args__items as *mut Object;
    let mut arena: Arena = ARENA_EMPTY;
    let mut tv: *mut typval_T = argvars.offset(2 as ::core::ffi::c_int as isize);
    while (*tv).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let c2rust_fresh5 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh5 as isize) = vim_to_object(tv, &raw mut arena, true);
        tv = tv.offset(1);
    }
    let mut ok: bool = rpc_send_event(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_number as uint64_t,
        tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize)),
        args,
    );
    arena_mem_free(arena_finish(&raw mut arena));
    if !ok {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"Channel doesn't exist\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    (*rettv).vval.v_number = 1 as varnumber_T;
}
unsafe extern "C" fn f_rpcrequest(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    let l_provider_call_nesting: ::core::ffi::c_int = provider_call_nesting.get();
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_number
            <= 0 as varnumber_T
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"Channel id must be a positive integer\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"Method name must be a string\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 20] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_16 { boolean: false },
    }; 20];
    args.capacity = MAX_FUNC_ARGS as ::core::ffi::c_int as size_t;
    args.items = &raw mut args__items as *mut Object;
    let mut arena: Arena = ARENA_EMPTY;
    let mut tv: *mut typval_T = argvars.offset(2 as ::core::ffi::c_int as isize);
    while (*tv).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let c2rust_fresh3 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh3 as isize) = vim_to_object(tv, &raw mut arena, true);
        tv = tv.offset(1);
    }
    let mut save_current_sctx: sctx_T = sctx_T {
        sc_sid: 0,
        sc_seq: 0,
        sc_lnum: 0,
        sc_chan: 0,
    };
    let mut save_autocmd_fname: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut save_autocmd_match: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut save_autocmd_fname_full: bool = false;
    let mut save_autocmd_bufnr: ::core::ffi::c_int = 0;
    let mut funccal_entry: funccal_entry_T = funccal_entry_T {
        top_funccal: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        next: ::core::ptr::null_mut::<funccal_entry_T>(),
    };
    if l_provider_call_nesting != 0 {
        save_current_sctx = current_sctx.get();
        save_autocmd_fname = autocmd_fname.get();
        save_autocmd_match = autocmd_match.get();
        save_autocmd_fname_full = autocmd_fname_full.get();
        save_autocmd_bufnr = autocmd_bufnr.get();
        save_funccal(&raw mut funccal_entry);
        current_sctx.set((*provider_caller_scope.ptr()).script_ctx);
        ga_grow(exestack.ptr(), 1 as ::core::ffi::c_int);
        let c2rust_fresh4 = (*exestack.ptr()).ga_len;
        (*exestack.ptr()).ga_len = (*exestack.ptr()).ga_len + 1;
        *((*exestack.ptr()).ga_data as *mut estack_T).offset(c2rust_fresh4 as isize) =
            (*provider_caller_scope.ptr()).es_entry;
        autocmd_fname.set((*provider_caller_scope.ptr()).autocmd_fname);
        autocmd_match.set((*provider_caller_scope.ptr()).autocmd_match);
        autocmd_fname_full.set((*provider_caller_scope.ptr()).autocmd_fname_full);
        autocmd_bufnr.set((*provider_caller_scope.ptr()).autocmd_bufnr);
        set_current_funccal((*provider_caller_scope.ptr()).funccalp as *mut funccall_T);
    }
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut chan_id: uint64_t = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_number as uint64_t;
    let mut method: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
    let mut res_mem: ArenaMem = ::core::ptr::null_mut::<consumed_blk>();
    let mut result: Object = rpc_send_call(chan_id, method, args, &raw mut res_mem, &raw mut err);
    arena_mem_free(arena_finish(&raw mut arena));
    if l_provider_call_nesting != 0 {
        current_sctx.set(save_current_sctx);
        (*exestack.ptr()).ga_len -= 1;
        autocmd_fname.set(save_autocmd_fname);
        autocmd_match.set(save_autocmd_match);
        autocmd_fname_full.set(save_autocmd_fname_full);
        autocmd_bufnr.set(save_autocmd_bufnr);
        restore_funccal();
    }
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut chan: *mut Channel = find_channel(chan_id);
        if !chan.is_null() {
            name = get_client_info(chan, b"name\0".as_ptr() as *const ::core::ffi::c_char);
        }
        if !name.is_null() {
            semsg_multiline(
                b"rpc_error\0".as_ptr() as *const ::core::ffi::c_char,
                b"Invoking '%s' on channel %lu (%s):\n%s\0".as_ptr() as *const ::core::ffi::c_char,
                method,
                chan_id,
                name,
                err.msg,
            );
        } else {
            semsg_multiline(
                b"rpc_error\0".as_ptr() as *const ::core::ffi::c_char,
                b"Invoking '%s' on channel %lu:\n%s\0".as_ptr() as *const ::core::ffi::c_char,
                method,
                chan_id,
                err.msg,
            );
        }
    } else {
        object_to_vim(result, rettv, &raw mut err);
    }
    arena_mem_free(res_mem);
    api_clear_error(&raw mut err);
}
unsafe extern "C" fn screenchar_adjust(
    mut grid: *mut *mut ScreenGrid,
    mut row: *mut ::core::ffi::c_int,
    mut col: *mut ::core::ffi::c_int,
) {
    msg_scroll_flush();
    *grid = ui_comp_get_grid_at_coord(*row, *col);
    *row -= (**grid).comp_row;
    *col -= (**grid).comp_col;
}
unsafe extern "C" fn f_screenattr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut row: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int;
    let mut col: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int;
    let mut grid: *mut ScreenGrid = ::core::ptr::null_mut::<ScreenGrid>();
    screenchar_adjust(&raw mut grid, &raw mut row, &raw mut col);
    let mut c: ::core::ffi::c_int = 0;
    if row < 0 as ::core::ffi::c_int
        || row >= (*grid).rows
        || col < 0 as ::core::ffi::c_int
        || col >= (*grid).cols
    {
        c = -1 as ::core::ffi::c_int;
    } else {
        c = *(*grid).attrs.offset(
            (*(*grid).line_offset.offset(row as isize)).wrapping_add(col as size_t) as isize,
        ) as ::core::ffi::c_int;
    }
    (*rettv).vval.v_number = c as varnumber_T;
}
unsafe extern "C" fn f_screenchar(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut row: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int;
    let mut col: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int;
    let mut grid: *mut ScreenGrid = ::core::ptr::null_mut::<ScreenGrid>();
    screenchar_adjust(&raw mut grid, &raw mut row, &raw mut col);
    (*rettv).vval.v_number = (if row < 0 as ::core::ffi::c_int
        || row >= (*grid).rows
        || col < 0 as ::core::ffi::c_int
        || col >= (*grid).cols
    {
        -1 as ::core::ffi::c_int
    } else {
        schar_get_first_codepoint(grid_getchar(
            grid,
            row,
            col,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ))
    }) as varnumber_T;
}
unsafe extern "C" fn f_screenchars(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut row: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int;
    let mut col: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int;
    let mut grid: *mut ScreenGrid = ::core::ptr::null_mut::<ScreenGrid>();
    screenchar_adjust(&raw mut grid, &raw mut row, &raw mut col);
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    if row < 0 as ::core::ffi::c_int
        || row >= (*grid).rows
        || col < 0 as ::core::ffi::c_int
        || col >= (*grid).cols
    {
        return;
    }
    let mut buf: [::core::ffi::c_char; 33] = [0; 33];
    schar_get(
        &raw mut buf as *mut ::core::ffi::c_char,
        grid_getchar(
            grid,
            row,
            col,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ),
    );
    let mut i: size_t = 0 as size_t;
    loop {
        let mut c: ::core::ffi::c_int =
            utf_ptr2char((&raw mut buf as *mut ::core::ffi::c_char).offset(i as isize));
        tv_list_append_number((*rettv).vval.v_list, c as varnumber_T);
        i = i.wrapping_add(utf_ptr2len(
            (&raw mut buf as *mut ::core::ffi::c_char).offset(i as isize),
        ) as size_t);
        if buf[i as usize] as ::core::ffi::c_int == NUL {
            break;
        }
    }
}
unsafe extern "C" fn f_screencol(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = (ui_current_col() + 1 as ::core::ffi::c_int) as varnumber_T;
}
unsafe extern "C" fn f_screenrow(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = (ui_current_row() + 1 as ::core::ffi::c_int) as varnumber_T;
}
unsafe extern "C" fn f_screenstring(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*rettv).v_type = VAR_STRING;
    let mut grid: *mut ScreenGrid = ::core::ptr::null_mut::<ScreenGrid>();
    let mut row: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int;
    let mut col: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int;
    screenchar_adjust(&raw mut grid, &raw mut row, &raw mut col);
    if row < 0 as ::core::ffi::c_int
        || row >= (*grid).rows
        || col < 0 as ::core::ffi::c_int
        || col >= (*grid).cols
    {
        return;
    }
    let mut buf: [::core::ffi::c_char; 33] = [0; 33];
    schar_get(
        &raw mut buf as *mut ::core::ffi::c_char,
        grid_getchar(
            grid,
            row,
            col,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ),
    );
    (*rettv).vval.v_string = xstrdup(&raw mut buf as *mut ::core::ffi::c_char);
}
unsafe extern "C" fn f_search(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    (*rettv).vval.v_number =
        search_cmn(argvars, ::core::ptr::null_mut::<pos_T>(), &raw mut flags) as varnumber_T;
}
unsafe extern "C" fn f_searchdecl(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut locally: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut thisblock: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut error: bool = false_0 != 0;
    (*rettv).vval.v_number = 1 as varnumber_T;
    let name: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        locally = (tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) == 0 as varnumber_T) as ::core::ffi::c_int;
        if !error
            && (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            thisblock = (tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) != 0 as varnumber_T) as ::core::ffi::c_int;
        }
    }
    if !error && !name.is_null() {
        (*rettv).vval.v_number = (find_decl(
            name as *mut ::core::ffi::c_char,
            strlen(name),
            locally != 0,
            thisblock != 0,
            SEARCH_KEEP as ::core::ffi::c_int,
        ) as ::core::ffi::c_int
            == FAIL) as ::core::ffi::c_int as varnumber_T;
    }
}
unsafe extern "C" fn searchpair_cmn(
    mut argvars: *mut typval_T,
    mut match_pos: *mut pos_T,
) -> ::core::ffi::c_int {
    let mut dir: ::core::ffi::c_int = 0;
    let mut skip: *const typval_T = ::core::ptr::null::<typval_T>();
    let mut save_p_ws: bool = p_ws.get() != 0;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut lnum_stop: linenr_T = 0 as linenr_T;
    let mut time_limit: int64_t = 0 as int64_t;
    let mut nbuf1: [::core::ffi::c_char; 65] = [0; 65];
    let mut nbuf2: [::core::ffi::c_char; 65] = [0; 65];
    let mut spat: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut mpat: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut nbuf1 as *mut ::core::ffi::c_char,
    );
    let mut epat: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(2 as ::core::ffi::c_int as isize),
        &raw mut nbuf2 as *mut ::core::ffi::c_char,
    );
    '_theend: {
        if !(spat.is_null() || mpat.is_null() || epat.is_null()) {
            dir = get_search_arg(
                argvars.offset(3 as ::core::ffi::c_int as isize),
                &raw mut flags,
            );
            if dir != 0 as ::core::ffi::c_int {
                if flags & (SP_END | SP_SUBPAT) != 0 as ::core::ffi::c_int
                    || flags & SP_NOMOVE != 0 && flags & SP_SETPCMARK != 0
                {
                    semsg(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        tv_get_string(argvars.offset(3 as ::core::ffi::c_int as isize)),
                    );
                } else {
                    if flags & SP_REPEAT != 0 {
                        p_ws.set(false_0);
                    }
                    skip = ::core::ptr::null::<typval_T>();
                    if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type
                        as ::core::ffi::c_uint
                        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                        || (*argvars.offset(4 as ::core::ffi::c_int as isize)).v_type
                            as ::core::ffi::c_uint
                            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        skip = ::core::ptr::null::<typval_T>();
                    } else {
                        skip = argvars.offset(4 as ::core::ffi::c_int as isize);
                        if (*argvars.offset(5 as ::core::ffi::c_int as isize)).v_type
                            as ::core::ffi::c_uint
                            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            lnum_stop = tv_get_number_chk(
                                argvars.offset(5 as ::core::ffi::c_int as isize),
                                ::core::ptr::null_mut::<bool>(),
                            ) as linenr_T;
                            if lnum_stop < 0 as linenr_T {
                                semsg(
                                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                                    tv_get_string(argvars.offset(5 as ::core::ffi::c_int as isize)),
                                );
                                break '_theend;
                            } else if (*argvars.offset(6 as ::core::ffi::c_int as isize)).v_type
                                as ::core::ffi::c_uint
                                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                time_limit = tv_get_number_chk(
                                    argvars.offset(6 as ::core::ffi::c_int as isize),
                                    ::core::ptr::null_mut::<bool>(),
                                ) as int64_t;
                                if time_limit < 0 as int64_t {
                                    semsg(
                                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                                        tv_get_string(
                                            argvars.offset(6 as ::core::ffi::c_int as isize),
                                        ),
                                    );
                                    break '_theend;
                                }
                            }
                        }
                    }
                    retval = do_searchpair(
                        spat, mpat, epat, dir, skip, flags, match_pos, lnum_stop, time_limit,
                    );
                }
            }
        }
    }
    p_ws.set(save_p_ws as ::core::ffi::c_int);
    return retval;
}
unsafe extern "C" fn f_searchpair(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number =
        searchpair_cmn(argvars, ::core::ptr::null_mut::<pos_T>()) as varnumber_T;
}
unsafe extern "C" fn f_searchpairpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut match_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut lnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tv_list_alloc_ret(rettv, 2 as ptrdiff_t);
    if searchpair_cmn(argvars, &raw mut match_pos) > 0 as ::core::ffi::c_int {
        lnum = match_pos.lnum as ::core::ffi::c_int;
        col = match_pos.col as ::core::ffi::c_int;
    }
    tv_list_append_number((*rettv).vval.v_list, lnum as varnumber_T);
    tv_list_append_number((*rettv).vval.v_list, col as varnumber_T);
}
pub unsafe extern "C" fn do_searchpair(
    mut spat: *const ::core::ffi::c_char,
    mut mpat: *const ::core::ffi::c_char,
    mut epat: *const ::core::ffi::c_char,
    mut dir: ::core::ffi::c_int,
    mut skip: *const typval_T,
    mut flags: ::core::ffi::c_int,
    mut match_pos: *mut pos_T,
    mut lnum_stop: linenr_T,
    mut time_limit: int64_t,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut nest: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut use_skip: bool = false_0 != 0;
    let mut options: ::core::ffi::c_int = SEARCH_KEEP as ::core::ffi::c_int;
    let mut save_cpo: *mut ::core::ffi::c_char = p_cpo.get();
    p_cpo.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
    let mut tm: proftime_T = profile_setlimit(time_limit);
    let spatlen: size_t = strlen(spat);
    let epatlen: size_t = strlen(epat);
    let pat2size: size_t = spatlen.wrapping_add(epatlen).wrapping_add(17 as size_t);
    let mut pat2: *mut ::core::ffi::c_char = xmalloc(pat2size) as *mut ::core::ffi::c_char;
    let pat3size: size_t = spatlen
        .wrapping_add(strlen(mpat))
        .wrapping_add(epatlen)
        .wrapping_add(25 as size_t);
    let mut pat3: *mut ::core::ffi::c_char = xmalloc(pat3size) as *mut ::core::ffi::c_char;
    let mut pat2len: ::core::ffi::c_int = snprintf(
        pat2,
        pat2size,
        b"\\m\\(%s\\m\\)\\|\\(%s\\m\\)\0".as_ptr() as *const ::core::ffi::c_char,
        spat,
        epat,
    );
    let mut pat3len: ::core::ffi::c_int = 0;
    if *mpat as ::core::ffi::c_int == NUL {
        strcpy(pat3, pat2);
        pat3len = pat2len;
    } else {
        pat3len = snprintf(
            pat3,
            pat3size,
            b"\\m\\(%s\\m\\)\\|\\(%s\\m\\)\\|\\(%s\\m\\)\0".as_ptr() as *const ::core::ffi::c_char,
            spat,
            epat,
            mpat,
        );
    }
    if flags & SP_START != 0 {
        options |= SEARCH_START as ::core::ffi::c_int;
    }
    if !skip.is_null() {
        use_skip = eval_expr_valid_arg(skip);
    }
    let mut save_cursor: pos_T = (*curwin.get()).w_cursor;
    let mut pos: pos_T = (*curwin.get()).w_cursor;
    let mut firstpos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    clearpos(&raw mut firstpos);
    let mut foundpos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    clearpos(&raw mut foundpos);
    let mut pat: *mut ::core::ffi::c_char = pat3;
    '_c2rust_label: {
        if pat3len >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"pat3len >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/funcs.rs\0".as_ptr()
                    as *const ::core::ffi::c_char,
                6178 as ::core::ffi::c_uint,
                b"int do_searchpair(const char *, const char *, const char *, int, const typval_T *, int, pos_T *, linenr_T, int64_t)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut patlen: size_t = pat3len as size_t;
    loop {
        let mut sia: searchit_arg_T = searchit_arg_T {
            sa_stop_lnum: lnum_stop,
            sa_tm: &raw mut tm,
            sa_timed_out: 0,
            sa_wrapped: 0,
        };
        let mut n: ::core::ffi::c_int = searchit(
            curwin.get(),
            curbuf.get(),
            &raw mut pos,
            ::core::ptr::null_mut::<pos_T>(),
            dir as Direction,
            pat,
            patlen,
            1 as ::core::ffi::c_int,
            options,
            RE_SEARCH as ::core::ffi::c_int,
            &raw mut sia,
        );
        if n == FAIL
            || firstpos.lnum != 0 as linenr_T && equalpos(pos, firstpos) as ::core::ffi::c_int != 0
        {
            break;
        }
        if firstpos.lnum == 0 as linenr_T {
            firstpos = pos;
        }
        if equalpos(pos, foundpos) {
            if dir == BACKWARD as ::core::ffi::c_int {
                decl(&raw mut pos);
            } else {
                incl(&raw mut pos);
            }
        }
        foundpos = pos;
        options &= !(SEARCH_START as ::core::ffi::c_int);
        if use_skip {
            let mut save_pos: pos_T = (*curwin.get()).w_cursor;
            (*curwin.get()).w_cursor = pos;
            let mut err: bool = false_0 != 0;
            let r: bool = eval_expr_to_bool(skip, &raw mut err);
            (*curwin.get()).w_cursor = save_pos;
            if err {
                (*curwin.get()).w_cursor = save_cursor;
                retval = -1 as ::core::ffi::c_int;
                break;
            } else if r {
                continue;
            }
        }
        if dir == BACKWARD as ::core::ffi::c_int && n == 3 as ::core::ffi::c_int
            || dir == FORWARD as ::core::ffi::c_int && n == 2 as ::core::ffi::c_int
        {
            nest += 1;
            pat = pat2;
        } else {
            nest -= 1;
            if nest == 1 as ::core::ffi::c_int {
                pat = pat3;
            }
        }
        if nest != 0 as ::core::ffi::c_int {
            continue;
        }
        if flags & SP_RETCOUNT != 0 {
            retval += 1;
        } else {
            retval = pos.lnum as ::core::ffi::c_int;
        }
        if flags & SP_SETPCMARK != 0 {
            setpcmark();
        }
        (*curwin.get()).w_cursor = pos;
        if flags & SP_REPEAT == 0 {
            break;
        }
        nest = 1 as ::core::ffi::c_int;
    }
    if !match_pos.is_null() {
        (*match_pos).lnum = (*curwin.get()).w_cursor.lnum;
        (*match_pos).col = ((*curwin.get()).w_cursor.col as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as colnr_T;
    }
    if flags & SP_NOMOVE != 0 || retval == 0 as ::core::ffi::c_int {
        (*curwin.get()).w_cursor = save_cursor;
    }
    xfree(pat2 as *mut ::core::ffi::c_void);
    xfree(pat3 as *mut ::core::ffi::c_void);
    if p_cpo.get() == empty_string_option.ptr() as *mut ::core::ffi::c_char {
        p_cpo.set(save_cpo);
    } else {
        if *p_cpo.get() as ::core::ffi::c_int == NUL {
            set_option_value_give_err(
                kOptCpoptions,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: cstr_as_string(save_cpo),
                    },
                },
                0 as ::core::ffi::c_int,
            );
        }
        free_string_option(save_cpo);
    }
    return retval;
}
unsafe extern "C" fn f_searchpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut match_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let n: ::core::ffi::c_int = search_cmn(argvars, &raw mut match_pos, &raw mut flags);
    tv_list_alloc_ret(
        rettv,
        (2 as ::core::ffi::c_int + (flags & SP_SUBPAT != 0) as ::core::ffi::c_int) as ptrdiff_t,
    );
    let lnum: ::core::ffi::c_int = if n > 0 as ::core::ffi::c_int {
        match_pos.lnum as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    let col: ::core::ffi::c_int = if n > 0 as ::core::ffi::c_int {
        match_pos.col as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    tv_list_append_number((*rettv).vval.v_list, lnum as varnumber_T);
    tv_list_append_number((*rettv).vval.v_list, col as varnumber_T);
    if flags & SP_SUBPAT != 0 {
        tv_list_append_number((*rettv).vval.v_list, n as varnumber_T);
    }
}
unsafe extern "C" fn f_serverlist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut args: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_16 { boolean: false },
    }; 1];
    let mut err: Error = Error {
        type_0: kErrorTypeException,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut rv: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_16 { boolean: false },
    };
    let mut n: size_t = 0;
    let mut addrs: *mut *mut ::core::ffi::c_char = server_address_list(&raw mut n);
    let mut arena: Arena = ARENA_EMPTY;
    let mut addrs_arr: Array = arena_array(&raw mut arena, n);
    let l: *mut list_T = tv_list_alloc_ret(rettv, n as ptrdiff_t);
    let mut i: size_t = 0 as size_t;
    while i < n {
        tv_list_append_allocated_string(l, *addrs.offset(i as isize));
        let c2rust_fresh1 = addrs_arr.size;
        addrs_arr.size = addrs_arr.size.wrapping_add(1);
        *addrs_arr.items.offset(c2rust_fresh1 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed_16 {
                string: cstr_as_string(*addrs.offset(i as isize)),
            },
        };
        i = i.wrapping_add(1);
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        && tv_dict_get_bool(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_dict,
            b"peer\0".as_ptr() as *const ::core::ffi::c_char,
            false_0,
        ) != 0
    {
        args = ARRAY_DICT_INIT;
        args__items = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_16 { boolean: false },
        }; 1];
        args.capacity = 1 as size_t;
        args.items = &raw mut args__items as *mut Object;
        let c2rust_fresh2 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh2 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed_16 { array: addrs_arr },
        };
        err = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        rv = nlua_exec(
            String_0 {
                data: b"return require('vim._core.server').serverlist(...)\0".as_ptr()
                    as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 51]>().wrapping_sub(1 as size_t),
            },
            ::core::ptr::null::<::core::ffi::c_char>(),
            args,
            kRetObject,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"f_serverlist\0".as_ptr() as *const ::core::ffi::c_char,
                6338 as ::core::ffi::c_int,
                true_0 != 0,
                b"vim._core.serverlist failed: %s\0".as_ptr() as *const ::core::ffi::c_char,
                err.msg,
            );
        } else {
            let mut i_0: size_t = 0 as size_t;
            while i_0 < rv.data.array.size {
                let mut curr_server: *mut ::core::ffi::c_char =
                    (*rv.data.array.items.offset(i_0 as isize)).data.string.data;
                tv_list_append_string(l, curr_server, -1 as ssize_t);
                i_0 = i_0.wrapping_add(1);
            }
        }
    }
    xfree(addrs as *mut ::core::ffi::c_void);
    arena_mem_free(arena_finish(&raw mut arena));
}
unsafe extern "C" fn f_serverstart(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if check_secure() {
        return;
    }
    let mut address: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return;
        }
        address = xstrdup(tv_get_string(argvars));
    } else {
        address = server_address_new(::core::ptr::null::<::core::ffi::c_char>());
    }
    let mut result: ::core::ffi::c_int = server_start(address);
    xfree(address as *mut ::core::ffi::c_void);
    if result != 0 as ::core::ffi::c_int {
        semsg(
            b"Failed to start server: %s\0".as_ptr() as *const ::core::ffi::c_char,
            if result > 0 as ::core::ffi::c_int {
                b"Unknown system error\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                uv_strerror(result)
            },
        );
        return;
    }
    let mut n: size_t = 0;
    let mut addrs: *mut *mut ::core::ffi::c_char = server_address_list(&raw mut n);
    (*rettv).vval.v_string = *addrs.offset(n.wrapping_sub(1 as size_t) as isize);
    n = n.wrapping_sub(1);
    let mut i: size_t = 0 as size_t;
    while i < n {
        xfree(*addrs.offset(i as isize) as *mut ::core::ffi::c_void);
        i = i.wrapping_add(1);
    }
    xfree(addrs as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn f_serverstop(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    if !(*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_string
        .is_null()
    {
        let mut rv: bool = server_stop(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_string,
            false_0 != 0,
        );
        (*rettv).vval.v_number = (if rv as ::core::ffi::c_int != 0 {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as varnumber_T;
    }
}
unsafe extern "C" fn set_position(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut charpos: bool,
) {
    let mut curswant: colnr_T = -1 as colnr_T;
    (*rettv).vval.v_number = -1 as varnumber_T;
    let name: *const ::core::ffi::c_char = tv_get_string_chk(argvars);
    if name.is_null() {
        return;
    }
    let mut pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut fnum: ::core::ffi::c_int = 0;
    if list2fpos(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut pos,
        &raw mut fnum,
        &raw mut curswant,
        charpos,
    ) != OK
    {
        return;
    }
    if pos.col != MAXCOL as ::core::ffi::c_int && {
        pos.col -= 1;
        pos.col < 0 as ::core::ffi::c_int
    } {
        pos.col = 0 as ::core::ffi::c_int as colnr_T;
    }
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '.' as ::core::ffi::c_int
        && *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        (*curwin.get()).w_cursor = pos;
        if curswant >= 0 as ::core::ffi::c_int {
            (*curwin.get()).w_curswant =
                (curswant as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as colnr_T;
            (*curwin.get()).w_set_curswant = false_0;
        }
        check_cursor(curwin.get());
        (*rettv).vval.v_number = 0 as varnumber_T;
    } else if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '\'' as ::core::ffi::c_int
        && *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        && *name.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        if setmark_pos(
            *name.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int,
            &raw mut pos,
            fnum,
            ::core::ptr::null_mut::<fmarkv_T>(),
        ) == OK
        {
            (*rettv).vval.v_number = 0 as varnumber_T;
        }
    } else {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
    };
}
unsafe extern "C" fn f_setcharpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    set_position(argvars, rettv, true_0 != 0);
}
unsafe extern "C" fn f_setcharsearch(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if tv_check_for_dict_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
        return;
    }
    let mut d: *mut dict_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_dict;
    if d.is_null() {
        return;
    }
    let csearch: *mut ::core::ffi::c_char = tv_dict_get_string(
        d,
        b"char\0".as_ptr() as *const ::core::ffi::c_char,
        false_0 != 0,
    );
    if !csearch.is_null() {
        let mut c: ::core::ffi::c_int = utf_ptr2char(csearch);
        set_last_csearch(c, csearch, utfc_ptr2len(csearch));
    }
    let mut di: *mut dictitem_T = tv_dict_find(
        d,
        b"forward\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        set_csearch_direction(
            (if tv_get_number(&raw mut (*di).di_tv) != 0 {
                FORWARD as ::core::ffi::c_int
            } else {
                BACKWARD as ::core::ffi::c_int
            }) as Direction,
        );
    }
    di = tv_dict_find(
        d,
        b"until\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        set_csearch_until((tv_get_number(&raw mut (*di).di_tv) != 0) as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn f_setcursorcharpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    set_cursorpos(argvars, rettv, true_0 != 0);
}
unsafe extern "C" fn f_setenv(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut namebuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut valbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut name: *const ::core::ffi::c_char = tv_get_string_buf(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut namebuf as *mut ::core::ffi::c_char,
    );
    if check_secure() {
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_SPECIAL as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_special as ::core::ffi::c_uint
            == kSpecialVarNull as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        vim_unsetenv_ext(name);
    } else {
        vim_setenv_ext(
            name,
            tv_get_string_buf(
                argvars.offset(1 as ::core::ffi::c_int as isize),
                &raw mut valbuf as *mut ::core::ffi::c_char,
            ),
        );
    };
}
unsafe extern "C" fn f_setfperm(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = 0 as varnumber_T;
    let fname: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    if fname.is_null() {
        return;
    }
    let mut modebuf: [::core::ffi::c_char; 65] = [0; 65];
    let mode_str: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut modebuf as *mut ::core::ffi::c_char,
    );
    if mode_str.is_null() {
        return;
    }
    if strlen(mode_str) != 9 as size_t {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            mode_str,
        );
        return;
    }
    let mut mask: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut mode: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        if *mode_str.offset(i as isize) as ::core::ffi::c_int != '-' as ::core::ffi::c_int {
            mode |= mask;
        }
        mask = mask << 1 as ::core::ffi::c_int;
        i -= 1;
    }
    (*rettv).vval.v_number = (os_setperm(fname, mode) == OK) as ::core::ffi::c_int as varnumber_T;
}
unsafe extern "C" fn f_setpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    set_position(argvars, rettv, false_0 != 0);
}
unsafe extern "C" fn get_yank_type(
    pp: *mut *mut ::core::ffi::c_char,
    yank_type: *mut MotionType,
    block_len: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut stropt: *mut ::core::ffi::c_char = *pp;
    match *stropt as ::core::ffi::c_int {
        118 | 99 => {
            *yank_type = kMTCharWise;
        }
        86 | 108 => {
            *yank_type = kMTLineWise;
        }
        98 | Ctrl_V => {
            *yank_type = kMTBlockWise;
            if ascii_isdigit(*stropt.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            {
                stropt = stropt.offset(1);
                *block_len = getdigits_int(&raw mut stropt, false_0 != 0, 0 as ::core::ffi::c_int)
                    - 1 as ::core::ffi::c_int;
                stropt = stropt.offset(-1);
            }
        }
        _ => return FAIL,
    }
    *pp = stropt;
    return OK;
}
unsafe extern "C" fn f_setreg(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut append: bool = false_0 != 0;
    let mut block_len: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut yank_type: MotionType = kMTUnknown;
    (*rettv).vval.v_number = 1 as varnumber_T;
    let strregname: *const ::core::ffi::c_char = tv_get_string_chk(argvars);
    if strregname.is_null() {
        return;
    }
    let mut regname: ::core::ffi::c_char = *strregname;
    if regname as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        || regname as ::core::ffi::c_int == '@' as ::core::ffi::c_int
    {
        regname = '"' as ::core::ffi::c_char;
    }
    let mut regcontents: *const typval_T = ::core::ptr::null::<typval_T>();
    let mut pointreg: ::core::ffi::c_char = 0 as ::core::ffi::c_char;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let d: *mut dict_T = (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        if tv_dict_len(d) == 0 as ::core::ffi::c_long {
            let mut lstval: [*mut ::core::ffi::c_char; 2] = [
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ];
            write_reg_contents_lst(
                regname as ::core::ffi::c_int,
                &raw mut lstval as *mut *mut ::core::ffi::c_char,
                false_0 != 0,
                kMTUnknown,
                -1 as colnr_T,
            );
            return;
        }
        let di: *mut dictitem_T = tv_dict_find(
            d,
            b"regcontents\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            regcontents = &raw mut (*di).di_tv;
        }
        let mut stropt: *const ::core::ffi::c_char = tv_dict_get_string(
            d,
            b"regtype\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        if !stropt.is_null() {
            let ret: ::core::ffi::c_int = get_yank_type(
                &raw mut stropt as *mut *mut ::core::ffi::c_char,
                &raw mut yank_type,
                &raw mut block_len,
            );
            if ret == FAIL || {
                stropt = stropt.offset(1);
                *stropt as ::core::ffi::c_int != NUL
            } {
                semsg(
                    gettext(&raw const e_invargval as *const ::core::ffi::c_char),
                    b"value\0".as_ptr() as *const ::core::ffi::c_char,
                );
                return;
            }
        }
        if regname as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
            stropt = tv_dict_get_string(
                d,
                b"points_to\0".as_ptr() as *const ::core::ffi::c_char,
                false_0 != 0,
            );
            if !stropt.is_null() {
                pointreg = *stropt;
                regname = pointreg;
            }
        } else if tv_dict_get_number(d, b"isunnamed\0".as_ptr() as *const ::core::ffi::c_char) != 0
        {
            pointreg = regname;
        }
    } else {
        regcontents = argvars.offset(1 as ::core::ffi::c_int as isize);
    }
    let mut set_unnamed: bool = false_0 != 0;
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if yank_type as ::core::ffi::c_int != kMTUnknown as ::core::ffi::c_int {
            semsg(
                gettext(&raw const e_toomanyarg as *const ::core::ffi::c_char),
                b"setreg\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return;
        }
        let mut stropt_0: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(2 as ::core::ffi::c_int as isize));
        if stropt_0.is_null() {
            return;
        }
        while *stropt_0 as ::core::ffi::c_int != NUL {
            match *stropt_0 as ::core::ffi::c_int {
                97 | 65 => {
                    append = true_0 != 0;
                }
                117 | 34 => {
                    set_unnamed = true_0 != 0;
                }
                _ => {
                    get_yank_type(
                        &raw mut stropt_0 as *mut *mut ::core::ffi::c_char,
                        &raw mut yank_type,
                        &raw mut block_len,
                    );
                }
            }
            stropt_0 = stropt_0.offset(1);
        }
    }
    if !regcontents.is_null()
        && (*regcontents).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let ll: *mut list_T = (*regcontents).vval.v_list;
        let len: ::core::ffi::c_int = tv_list_len(ll);
        let mut lstval_0: *mut *mut ::core::ffi::c_char = xmalloc(
            ::core::mem::size_of::<*mut ::core::ffi::c_char>().wrapping_mul(
                (len as size_t)
                    .wrapping_add(1 as size_t)
                    .wrapping_mul(2 as size_t),
            ),
        )
            as *mut *mut ::core::ffi::c_char;
        let mut curval: *mut *const ::core::ffi::c_char =
            lstval_0 as *mut *const ::core::ffi::c_char;
        let mut allocval: *mut *mut ::core::ffi::c_char = lstval_0
            .offset(len as isize)
            .offset(2 as ::core::ffi::c_int as isize);
        let mut curallocval: *mut *mut ::core::ffi::c_char = allocval;
        let l_: *const list_T = ll;
        '_free_lstval: {
            's_313: {
                if !l_.is_null() {
                    let mut li: *const listitem_T = (*l_).lv_first;
                    loop {
                        if li.is_null() {
                            break 's_313;
                        }
                        let mut buf: [::core::ffi::c_char; 65] = [0; 65];
                        *curval = tv_get_string_buf_chk(
                            &raw const (*li).li_tv,
                            &raw mut buf as *mut ::core::ffi::c_char,
                        );
                        if (*curval).is_null() {
                            break '_free_lstval;
                        }
                        if *curval
                            == &raw mut buf as *mut ::core::ffi::c_char
                                as *const ::core::ffi::c_char
                        {
                            *curallocval = xstrdup(*curval);
                            *curval = *curallocval;
                            curallocval = curallocval.offset(1);
                        }
                        curval = curval.offset(1);
                        li = (*li).li_next;
                    }
                }
            }
            let c2rust_fresh9 = curval;
            curval = curval.offset(1);
            let c2rust_lvalue_ptr = &raw mut *c2rust_fresh9;
            *c2rust_lvalue_ptr = ::core::ptr::null::<::core::ffi::c_char>();
            write_reg_contents_lst(
                regname as ::core::ffi::c_int,
                lstval_0,
                append,
                yank_type,
                block_len,
            );
        }
        while curallocval > allocval {
            curallocval = curallocval.offset(-1);
            xfree(*curallocval as *mut ::core::ffi::c_void);
        }
        xfree(lstval_0 as *mut ::core::ffi::c_void);
    } else if !regcontents.is_null() {
        let strval: *const ::core::ffi::c_char = tv_get_string_chk(regcontents);
        if strval.is_null() {
            return;
        }
        write_reg_contents_ex(
            regname as ::core::ffi::c_int,
            strval,
            strlen(strval) as ssize_t,
            append,
            yank_type,
            block_len,
        );
    }
    if pointreg as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        get_yank_register(
            pointreg as ::core::ffi::c_int,
            YREG_YANK as ::core::ffi::c_int,
        );
    }
    (*rettv).vval.v_number = 0 as varnumber_T;
    if set_unnamed {
        op_reg_set_previous(regname);
    }
}
unsafe extern "C" fn f_settagstack(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    static e_invact2: GlobalCell<*const ::core::ffi::c_char> =
        GlobalCell::new(b"E962: Invalid action: '%s'\0".as_ptr() as *const ::core::ffi::c_char);
    let mut action: ::core::ffi::c_char = 'r' as ::core::ffi::c_char;
    (*rettv).vval.v_number = -1 as varnumber_T;
    let mut wp: *mut win_T = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
    if wp.is_null() {
        return;
    }
    if tv_check_for_dict_arg(argvars, 1 as ::core::ffi::c_int) == FAIL {
        return;
    }
    let mut d: *mut dict_T = (*argvars.offset(1 as ::core::ffi::c_int as isize))
        .vval
        .v_dict;
    if d.is_null() {
        return;
    }
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if tv_check_for_string_arg(argvars, 2 as ::core::ffi::c_int) == FAIL {
            return;
        } else {
            let mut actstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
            actstr = tv_get_string_chk(argvars.offset(2 as ::core::ffi::c_int as isize));
            if actstr.is_null() {
                return;
            }
            if (*actstr as ::core::ffi::c_int == 'r' as ::core::ffi::c_int
                || *actstr as ::core::ffi::c_int == 'a' as ::core::ffi::c_int
                || *actstr as ::core::ffi::c_int == 't' as ::core::ffi::c_int)
                && *actstr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            {
                action = *actstr;
            } else {
                semsg(gettext(e_invact2.get()), actstr);
                return;
            }
        }
    }
    if set_tagstack(wp, d, action as ::core::ffi::c_int) == OK {
        (*rettv).vval.v_number = 0 as varnumber_T;
    }
}
unsafe extern "C" fn f_sha256(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut blob: *mut blob_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_blob;
        let mut p: *const uint8_t = if !blob.is_null() {
            (*blob).bv_ga.ga_data as *mut uint8_t
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char as *mut uint8_t
        };
        let mut len: ::core::ffi::c_int = if !blob.is_null() {
            (*blob).bv_ga.ga_len
        } else {
            0 as ::core::ffi::c_int
        };
        (*rettv).vval.v_string = xstrdup(sha256_bytes(
            p,
            len as size_t,
            ::core::ptr::null::<uint8_t>(),
            0 as size_t,
        ));
    } else {
        let mut p_0: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        let mut hash: *const ::core::ffi::c_char = sha256_bytes(
            p_0 as *const uint8_t,
            strlen(p_0),
            ::core::ptr::null::<uint8_t>(),
            0 as size_t,
        );
        (*rettv).vval.v_string = xstrdup(hash);
    };
}
unsafe extern "C" fn f_shellescape(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let do_special: bool = non_zero_arg(argvars.offset(1 as ::core::ffi::c_int as isize));
    (*rettv).vval.v_string = vim_strsave_shellescape(
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
        do_special,
        do_special,
    );
    (*rettv).v_type = VAR_STRING;
}
unsafe extern "C" fn f_shiftwidth(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = 0 as varnumber_T;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut col: colnr_T =
            tv_get_number_chk(argvars, ::core::ptr::null_mut::<bool>()) as colnr_T;
        if col < 0 as ::core::ffi::c_int {
            return;
        }
        (*rettv).vval.v_number = get_sw_value_col(curbuf.get(), col, false_0 != 0) as varnumber_T;
        return;
    }
    (*rettv).vval.v_number = get_sw_value(curbuf.get()) as varnumber_T;
}
unsafe extern "C" fn f_sockconnect(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"expected dictionary\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut mode: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut address: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
    let mut tcp: bool = false;
    if strcmp(mode, b"tcp\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
        tcp = true_0 != 0;
    } else if strcmp(mode, b"pipe\0".as_ptr() as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        tcp = false_0 != 0;
    } else {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"invalid mode\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut rpc: bool = false_0 != 0;
    let mut on_data: CallbackReader = CallbackReader {
        cb: Callback {
            data: C2Rust_Unnamed_22 {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        },
        self_0: ::core::ptr::null_mut::<dict_T>(),
        buffer: GA_EMPTY_INIT_VALUE,
        eof: false,
        buffered: false_0 != 0,
        fwd_err: false_0 != 0,
        type_0: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut opts: *mut dict_T = (*argvars.offset(2 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        rpc = tv_dict_get_number(opts, b"rpc\0".as_ptr() as *const ::core::ffi::c_char)
            != 0 as varnumber_T;
        if !tv_dict_get_callback(
            opts,
            b"on_data\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
            &raw mut on_data.cb,
        ) {
            return;
        }
        on_data.buffered = tv_dict_get_number(
            opts,
            b"data_buffered\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0;
        if on_data.buffered as ::core::ffi::c_int != 0
            && on_data.cb.type_0 as ::core::ffi::c_uint
                == kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            on_data.self_0 = opts;
        }
    }
    let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut id: uint64_t = channel_connect(
        tcp,
        address,
        rpc,
        on_data,
        50 as ::core::ffi::c_int,
        &raw mut error,
    );
    if !error.is_null() {
        semsg(
            gettext(b"connection failed: %s\0".as_ptr() as *const ::core::ffi::c_char),
            error,
        );
    }
    (*rettv).vval.v_number = id as varnumber_T;
    (*rettv).v_type = VAR_NUMBER;
}
unsafe extern "C" fn f_stdioopen(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let mut on_stdin: CallbackReader = CallbackReader {
        cb: Callback {
            data: C2Rust_Unnamed_22 {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        },
        self_0: ::core::ptr::null_mut::<dict_T>(),
        buffer: GA_EMPTY_INIT_VALUE,
        eof: false,
        buffered: false_0 != 0,
        fwd_err: false_0 != 0,
        type_0: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    let mut opts: *mut dict_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_dict;
    let mut rpc: bool = tv_dict_get_number(opts, b"rpc\0".as_ptr() as *const ::core::ffi::c_char)
        != 0 as varnumber_T;
    if !tv_dict_get_callback(
        opts,
        b"on_stdin\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as usize) as ptrdiff_t,
        &raw mut on_stdin.cb,
    ) {
        return;
    }
    if !tv_dict_get_callback(
        opts,
        b"on_print\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as usize) as ptrdiff_t,
        on_print.ptr(),
    ) {
        return;
    }
    on_stdin.buffered = tv_dict_get_number(
        opts,
        b"stdin_buffered\0".as_ptr() as *const ::core::ffi::c_char,
    ) != 0;
    if on_stdin.buffered as ::core::ffi::c_int != 0
        && on_stdin.cb.type_0 as ::core::ffi::c_uint
            == kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        on_stdin.self_0 = opts;
    }
    let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut id: uint64_t = channel_from_stdio(rpc, on_stdin, &raw mut error);
    if id == 0 {
        semsg(&raw const e_stdiochan2 as *const ::core::ffi::c_char, error);
    }
    (*rettv).vval.v_number = id as varnumber_T;
    (*rettv).v_type = VAR_NUMBER;
}
unsafe extern "C" fn f_reltimefloat(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut tm: proftime_T = 0;
    (*rettv).v_type = VAR_FLOAT;
    (*rettv).vval.v_float = 0 as ::core::ffi::c_int as float_T;
    if list2proftime(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut tm,
    ) == OK
    {
        (*rettv).vval.v_float =
            (profile_signed(tm) as ::core::ffi::c_double / 1000000000.0f64) as float_T;
    }
}
unsafe extern "C" fn f_soundfold(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    let s: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    (*rettv).vval.v_string = eval_soundfold(s);
}
unsafe extern "C" fn f_spellbadword(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let wo_spell_save: ::core::ffi::c_int = (*curwin.get()).w_onebuf_opt.wo_spell;
    if (*curwin.get()).w_onebuf_opt.wo_spell == 0 {
        parse_spelllang(curwin.get());
        (*curwin.get()).w_onebuf_opt.wo_spell = true_0;
    }
    if *(*(*curwin.get()).w_s).b_p_spl as ::core::ffi::c_int == NUL {
        emsg(gettext(&raw const e_no_spell as *const ::core::ffi::c_char));
        (*curwin.get()).w_onebuf_opt.wo_spell = wo_spell_save;
        return;
    }
    let mut word: *const ::core::ffi::c_char = b"\0".as_ptr() as *const ::core::ffi::c_char;
    let mut attr: hlf_T = HLF_COUNT;
    let mut len: size_t = 0 as size_t;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        len = spell_move_to(
            curwin.get(),
            FORWARD as ::core::ffi::c_int,
            SMT_ALL,
            true_0 != 0,
            &raw mut attr,
        );
        if len != 0 as size_t {
            word = get_cursor_pos_ptr();
            (*curwin.get()).w_set_curswant = true_0;
        }
    } else if *(*curbuf.get()).b_s.b_p_spl as ::core::ffi::c_int != NUL {
        let mut str: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
        let mut capcol: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        if !str.is_null() {
            while *str as ::core::ffi::c_int != NUL {
                len = spell_check(
                    curwin.get(),
                    str as *mut ::core::ffi::c_char,
                    &raw mut attr,
                    &raw mut capcol,
                    false_0 != 0,
                );
                if attr as ::core::ffi::c_uint
                    != HLF_COUNT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    word = str;
                    break;
                } else {
                    str = str.offset(len as isize);
                    capcol -= len as ::core::ffi::c_int;
                    len = 0 as size_t;
                }
            }
        }
    }
    (*curwin.get()).w_onebuf_opt.wo_spell = wo_spell_save;
    '_c2rust_label: {
        if len <= 2147483647 as ::core::ffi::c_int as size_t {
        } else {
            __assert_fail(
                b"len <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/funcs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                6973 as ::core::ffi::c_uint,
                b"void f_spellbadword(typval_T *, typval_T *, EvalFuncData)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    tv_list_alloc_ret(rettv, 2 as ptrdiff_t);
    tv_list_append_string((*rettv).vval.v_list, word, len as ssize_t);
    match attr as ::core::ffi::c_uint {
        37 => {
            tv_list_append_string(
                (*rettv).vval.v_list,
                b"bad\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as usize)
                    as ssize_t,
            );
        }
        39 => {
            tv_list_append_string(
                (*rettv).vval.v_list,
                b"rare\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                    as ssize_t,
            );
        }
        40 => {
            tv_list_append_string(
                (*rettv).vval.v_list,
                b"local\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize)
                    as ssize_t,
            );
        }
        38 => {
            tv_list_append_string(
                (*rettv).vval.v_list,
                b"caps\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                    as ssize_t,
            );
        }
        _ => {
            tv_list_append_string(
                (*rettv).vval.v_list,
                ::core::ptr::null::<::core::ffi::c_char>(),
                -1 as ssize_t,
            );
        }
    };
}
unsafe extern "C" fn f_spellsuggest(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut ga: garray_T = GA_EMPTY_INIT_VALUE;
    let wo_spell_save: ::core::ffi::c_int = (*curwin.get()).w_onebuf_opt.wo_spell;
    if (*curwin.get()).w_onebuf_opt.wo_spell == 0 {
        parse_spelllang(curwin.get());
        (*curwin.get()).w_onebuf_opt.wo_spell = true_0;
    }
    if *(*(*curwin.get()).w_s).b_p_spl as ::core::ffi::c_int == NUL {
        emsg(gettext(&raw const e_no_spell as *const ::core::ffi::c_char));
        (*curwin.get()).w_onebuf_opt.wo_spell = wo_spell_save;
        return;
    }
    let mut maxcount: ::core::ffi::c_int = 0;
    let mut need_capital: bool = false_0 != 0;
    let str: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    '_f_spellsuggest_return: {
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut typeerr: bool = false_0 != 0;
            maxcount = tv_get_number_chk(
                argvars.offset(1 as ::core::ffi::c_int as isize),
                &raw mut typeerr,
            ) as ::core::ffi::c_int;
            if maxcount <= 0 as ::core::ffi::c_int {
                break '_f_spellsuggest_return;
            } else if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type
                as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                need_capital = tv_get_number_chk(
                    argvars.offset(2 as ::core::ffi::c_int as isize),
                    &raw mut typeerr,
                ) != 0;
                if typeerr {
                    break '_f_spellsuggest_return;
                }
            }
        } else {
            maxcount = 25 as ::core::ffi::c_int;
        }
        spell_suggest_list(
            &raw mut ga,
            str as *mut ::core::ffi::c_char,
            maxcount,
            need_capital,
            false_0 != 0,
        );
    }
    tv_list_alloc_ret(rettv, ga.ga_len as ptrdiff_t);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < ga.ga_len {
        let p: *mut ::core::ffi::c_char =
            *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize);
        tv_list_append_allocated_string((*rettv).vval.v_list, p);
        i += 1;
    }
    ga_clear(&raw mut ga);
    (*curwin.get()).w_onebuf_opt.wo_spell = wo_spell_save;
}
unsafe extern "C" fn f_split(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut col: colnr_T = 0 as colnr_T;
    let mut keepempty: bool = false_0 != 0;
    let mut typeerr: bool = false_0 != 0;
    let mut save_cpo: *mut ::core::ffi::c_char = p_cpo.get();
    p_cpo.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
    let mut str: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut pat: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut patbuf: [::core::ffi::c_char; 65] = [0; 65];
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        pat = tv_get_string_buf_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut patbuf as *mut ::core::ffi::c_char,
        );
        if pat.is_null() {
            typeerr = true_0 != 0;
        }
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            keepempty = tv_get_bool_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut typeerr,
            ) != 0;
        }
    }
    if pat.is_null() || *pat as ::core::ffi::c_int == NUL {
        pat = b"[\\x01- ]\\+\0".as_ptr() as *const ::core::ffi::c_char;
    }
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    if !typeerr {
        regmatch = regmatch_T {
            regprog: vim_regcomp(pat, RE_MAGIC + RE_STRING),
            startp: [
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ],
            endp: [
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ],
            rm_matchcol: 0,
            rm_ic: false_0 != 0,
        };
        if !regmatch.regprog.is_null() {
            while *str as ::core::ffi::c_int != NUL || keepempty as ::core::ffi::c_int != 0 {
                let mut match_0: bool = false;
                if *str as ::core::ffi::c_int == NUL {
                    match_0 = false_0 != 0;
                } else {
                    match_0 = vim_regexec_nl(&raw mut regmatch, str, col);
                }
                let mut end: *const ::core::ffi::c_char =
                    ::core::ptr::null::<::core::ffi::c_char>();
                if match_0 {
                    end = regmatch.startp[0 as ::core::ffi::c_int as usize];
                } else {
                    end = str.offset(strlen(str) as isize);
                }
                if keepempty as ::core::ffi::c_int != 0
                    || end > str
                    || tv_list_len((*rettv).vval.v_list) > 0 as ::core::ffi::c_int
                        && *str as ::core::ffi::c_int != NUL
                        && match_0 as ::core::ffi::c_int != 0
                        && end
                            < regmatch.endp[0 as ::core::ffi::c_int as usize]
                                as *const ::core::ffi::c_char
                {
                    tv_list_append_string(
                        (*rettv).vval.v_list,
                        str,
                        end.offset_from(str) as ssize_t,
                    );
                }
                if !match_0 {
                    break;
                }
                if regmatch.endp[0 as ::core::ffi::c_int as usize] > str as *mut ::core::ffi::c_char
                {
                    col = 0 as ::core::ffi::c_int as colnr_T;
                } else {
                    col = utfc_ptr2len(regmatch.endp[0 as ::core::ffi::c_int as usize]) as colnr_T;
                }
                str = regmatch.endp[0 as ::core::ffi::c_int as usize];
            }
            vim_regfree(regmatch.regprog);
        }
    }
    p_cpo.set(save_cpo);
}
unsafe extern "C" fn get_xdg_var_list(xdg: XDGVarType, mut rettv: *mut typval_T) {
    let list: *mut list_T = tv_list_alloc(kListLenShouldKnow as ::core::ffi::c_int as ptrdiff_t);
    (*rettv).v_type = VAR_LIST;
    (*rettv).vval.v_list = list;
    tv_list_ref(list);
    let dirs: *mut ::core::ffi::c_char = stdpaths_get_xdg_var(xdg);
    if dirs.is_null() {
        return;
    }
    let mut iter: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
    let mut appname: *const ::core::ffi::c_char = get_appname(false_0 != 0);
    loop {
        let mut dir_len: size_t = 0;
        let mut dir: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        iter = vim_env_iter(
            ENV_SEPCHAR as ::core::ffi::c_char,
            dirs,
            iter,
            &raw mut dir,
            &raw mut dir_len,
        );
        if !dir.is_null() && dir_len > 0 as size_t {
            let mut dir_with_nvim: *mut ::core::ffi::c_char =
                xmemdupz(dir as *const ::core::ffi::c_void, dir_len) as *mut ::core::ffi::c_char;
            dir_with_nvim = concat_fnames_realloc(dir_with_nvim, appname, true_0 != 0);
            tv_list_append_allocated_string(list, dir_with_nvim);
        }
        if iter.is_null() {
            break;
        }
    }
    xfree(dirs as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn f_stdpath(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let p: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    if p.is_null() {
        return;
    }
    if strequal(p, b"config\0".as_ptr() as *const ::core::ffi::c_char) {
        (*rettv).vval.v_string = get_xdg_home(kXDGConfigHome);
    } else if strequal(p, b"data\0".as_ptr() as *const ::core::ffi::c_char) {
        (*rettv).vval.v_string = get_xdg_home(kXDGDataHome);
    } else if strequal(p, b"cache\0".as_ptr() as *const ::core::ffi::c_char) {
        (*rettv).vval.v_string = get_xdg_home(kXDGCacheHome);
    } else if strequal(p, b"state\0".as_ptr() as *const ::core::ffi::c_char) {
        (*rettv).vval.v_string = get_xdg_home(kXDGStateHome);
    } else if strequal(p, b"log\0".as_ptr() as *const ::core::ffi::c_char) {
        (*rettv).vval.v_string = get_xdg_home(kXDGStateHome);
    } else if strequal(p, b"run\0".as_ptr() as *const ::core::ffi::c_char) {
        (*rettv).vval.v_string = stdpaths_get_xdg_var(kXDGRuntimeDir);
    } else if strequal(p, b"config_dirs\0".as_ptr() as *const ::core::ffi::c_char) {
        get_xdg_var_list(kXDGConfigDirs, rettv);
    } else if strequal(p, b"data_dirs\0".as_ptr() as *const ::core::ffi::c_char) {
        get_xdg_var_list(kXDGDataDirs, rettv);
    } else {
        semsg(
            gettext(
                b"E6100: \"%s\" is not a valid stdpath\0".as_ptr() as *const ::core::ffi::c_char
            ),
            p,
        );
    };
}
unsafe extern "C" fn f_str2float(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut p: *mut ::core::ffi::c_char = skipwhite(tv_get_string(
        argvars.offset(0 as ::core::ffi::c_int as isize),
    ));
    let mut isneg: bool = *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int;
    if *p as ::core::ffi::c_int == '+' as ::core::ffi::c_int
        || *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
    {
        p = skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
    }
    string2float(p, &raw mut (*rettv).vval.v_float);
    if isneg {
        (*rettv).vval.v_float *= -1 as ::core::ffi::c_int as float_T;
    }
    (*rettv).v_type = VAR_FLOAT;
}
unsafe extern "C" fn f_strftime(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut seconds: time_t = 0;
    (*rettv).v_type = VAR_STRING;
    let mut p: *mut ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_char;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        seconds = time(::core::ptr::null_mut::<time_t>());
    } else {
        seconds = tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as time_t;
    }
    let mut curtime: tm = tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_gmtoff: 0,
        tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    let mut curtime_ptr: *mut tm = os_localtime_r(&raw mut seconds, &raw mut curtime);
    if curtime_ptr.is_null() {
        (*rettv).vval.v_string = xstrdup(gettext(
            b"(Invalid)\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return;
    }
    let mut conv: vimconv_T = vimconv_T {
        vc_type: 0,
        vc_factor: 0,
        vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        vc_fail: false,
    };
    conv.vc_type = CONV_NONE as ::core::ffi::c_int;
    let mut enc: *mut ::core::ffi::c_char = enc_locale();
    convert_setup(&raw mut conv, p_enc.get(), enc);
    if conv.vc_type != CONV_NONE as ::core::ffi::c_int {
        p = string_convert(&raw mut conv, p, ::core::ptr::null_mut::<size_t>());
    }
    let mut result_buf: [::core::ffi::c_char; 256] = [0; 256];
    if p.is_null()
        || strftime(
            &raw mut result_buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>(),
            p,
            curtime_ptr,
        ) == 0 as size_t
    {
        result_buf[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    }
    if conv.vc_type != CONV_NONE as ::core::ffi::c_int {
        xfree(p as *mut ::core::ffi::c_void);
    }
    convert_setup(&raw mut conv, enc, p_enc.get());
    if conv.vc_type != CONV_NONE as ::core::ffi::c_int {
        (*rettv).vval.v_string = string_convert(
            &raw mut conv,
            &raw mut result_buf as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<size_t>(),
        );
    } else {
        (*rettv).vval.v_string = xstrdup(&raw mut result_buf as *mut ::core::ffi::c_char);
    }
    convert_setup(
        &raw mut conv,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    );
    xfree(enc as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn f_strptime(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut fmt_buf: [::core::ffi::c_char; 65] = [0; 65];
    let mut str_buf: [::core::ffi::c_char; 65] = [0; 65];
    let mut tmval: tm = tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: -1 as ::core::ffi::c_int,
        tm_gmtoff: 0,
        tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    let mut fmt: *mut ::core::ffi::c_char = tv_get_string_buf(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut fmt_buf as *mut ::core::ffi::c_char,
    ) as *mut ::core::ffi::c_char;
    let mut str: *mut ::core::ffi::c_char = tv_get_string_buf(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut str_buf as *mut ::core::ffi::c_char,
    ) as *mut ::core::ffi::c_char;
    let mut conv: vimconv_T = vimconv_T {
        vc_type: CONV_NONE as ::core::ffi::c_int,
        vc_factor: 0,
        vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        vc_fail: false,
    };
    let mut enc: *mut ::core::ffi::c_char = enc_locale();
    convert_setup(&raw mut conv, p_enc.get(), enc);
    if conv.vc_type != CONV_NONE as ::core::ffi::c_int {
        fmt = string_convert(&raw mut conv, fmt, ::core::ptr::null_mut::<size_t>());
    }
    if fmt.is_null() || os_strptime(str, fmt, &raw mut tmval).is_null() || {
        (*rettv).vval.v_number = mktime(&raw mut tmval) as varnumber_T;
        (*rettv).vval.v_number == -1 as varnumber_T
    } {
        (*rettv).vval.v_number = 0 as varnumber_T;
    }
    if conv.vc_type != CONV_NONE as ::core::ffi::c_int {
        xfree(fmt as *mut ::core::ffi::c_void);
    }
    convert_setup(
        &raw mut conv,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    );
    xfree(enc as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn f_submatch(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut error: bool = false_0 != 0;
    let mut no: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut error,
    ) as ::core::ffi::c_int;
    if error {
        return;
    }
    if no < 0 as ::core::ffi::c_int || no >= NSUBEXP as ::core::ffi::c_int {
        semsg(
            gettext((e_invalid_submatch_number_nr.ptr() as *const _) as *const ::core::ffi::c_char),
            no,
        );
        return;
    }
    let mut retList: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        retList = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) as ::core::ffi::c_int;
        if error {
            return;
        }
    }
    if retList == 0 as ::core::ffi::c_int {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = reg_submatch(no);
    } else {
        (*rettv).v_type = VAR_LIST;
        (*rettv).vval.v_list = reg_submatch_list(no);
    };
}
unsafe extern "C" fn f_substitute(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut patbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut subbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut flagsbuf: [::core::ffi::c_char; 65] = [0; 65];
    let str: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    let pat: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut patbuf as *mut ::core::ffi::c_char,
    );
    let mut sub: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let flg: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(3 as ::core::ffi::c_int as isize),
        &raw mut flagsbuf as *mut ::core::ffi::c_char,
    );
    let mut expr: *mut typval_T = ::core::ptr::null_mut::<typval_T>();
    if tv_is_func(*argvars.offset(2 as ::core::ffi::c_int as isize)) {
        expr = argvars.offset(2 as ::core::ffi::c_int as isize);
    } else {
        sub = tv_get_string_buf_chk(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            &raw mut subbuf as *mut ::core::ffi::c_char,
        );
    }
    (*rettv).v_type = VAR_STRING;
    if str.is_null() || pat.is_null() || sub.is_null() && expr.is_null() || flg.is_null() {
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        (*rettv).vval.v_string = do_string_sub(
            str as *mut ::core::ffi::c_char,
            strlen(str),
            pat as *mut ::core::ffi::c_char,
            sub as *mut ::core::ffi::c_char,
            expr,
            flg as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<size_t>(),
        );
    };
}
unsafe extern "C" fn f_swapfilelist(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
    recover_names(
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        (*rettv).vval.v_list,
        0 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    );
}
unsafe extern "C" fn f_swapinfo(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_dict_alloc_ret(rettv);
    swapfile_dict(tv_get_string(argvars), (*rettv).vval.v_dict);
}
unsafe extern "C" fn f_swapname(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    let mut buf: *mut buf_T = tv_get_buf(argvars.offset(0 as ::core::ffi::c_int as isize), false_0);
    if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() || (*(*buf).b_ml.ml_mfp).mf_fname.is_null() {
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        (*rettv).vval.v_string = xstrdup((*(*buf).b_ml.ml_mfp).mf_fname);
    };
}
unsafe extern "C" fn f_synID(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let lnum: linenr_T = tv_get_lnum(argvars);
    let col: colnr_T =
        tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as colnr_T - 1 as colnr_T;
    let mut transerr: bool = false_0 != 0;
    let trans: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(2 as ::core::ffi::c_int as isize),
        &raw mut transerr,
    ) as ::core::ffi::c_int;
    let mut id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !transerr
        && lnum >= 1 as linenr_T
        && lnum <= (*curbuf.get()).b_ml.ml_line_count
        && col >= 0 as ::core::ffi::c_int
        && col < ml_get_len(lnum)
    {
        id = syn_get_id(
            curwin.get(),
            lnum,
            col,
            trans,
            ::core::ptr::null_mut::<bool>(),
            false_0,
        );
    }
    (*rettv).vval.v_number = id as varnumber_T;
}
unsafe extern "C" fn f_synIDattr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let id: ::core::ffi::c_int =
        tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
    let what: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
    let mut modec: ::core::ffi::c_int = 0;
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut modebuf: [::core::ffi::c_char; 65] = [0; 65];
        let mode: *const ::core::ffi::c_char = tv_get_string_buf(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            &raw mut modebuf as *mut ::core::ffi::c_char,
        );
        modec = if (*mode.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            < 'A' as ::core::ffi::c_int
            || *mode.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                > 'Z' as ::core::ffi::c_int
        {
            *mode.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        } else {
            *mode.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        };
        if modec != 'c' as ::core::ffi::c_int && modec != 'g' as ::core::ffi::c_int {
            modec = 0 as ::core::ffi::c_int;
        }
    } else if ui_rgb_attached() {
        modec = 'g' as ::core::ffi::c_int;
    } else {
        modec = 'c' as ::core::ffi::c_int;
    }
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    match if (*what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        < 'A' as ::core::ffi::c_int
        || *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            > 'Z' as ::core::ffi::c_int
    {
        *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
    } else {
        *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
    } {
        98 => {
            if (if (*what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                < 'A' as ::core::ffi::c_int
                || *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    > 'Z' as ::core::ffi::c_int
            {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            } else {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            }) == 'g' as ::core::ffi::c_int
            {
                p = highlight_color(id, what, modec);
            } else if (if (*what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                < 'A' as ::core::ffi::c_int
                || *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    > 'Z' as ::core::ffi::c_int
            {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            } else {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            }) == 'l' as ::core::ffi::c_int
            {
                p = highlight_has_attr(id, HL_BLINK as ::core::ffi::c_int, modec);
            } else {
                p = highlight_has_attr(id, HL_BOLD as ::core::ffi::c_int, modec);
            }
        }
        99 => {
            p = highlight_has_attr(id, HL_CONCEALED as ::core::ffi::c_int, modec);
        }
        100 => {
            p = highlight_has_attr(id, HL_DIM as ::core::ffi::c_int, modec);
        }
        111 => {
            p = highlight_has_attr(id, HL_OVERLINE as ::core::ffi::c_int, modec);
        }
        102 => {
            p = highlight_color(id, what, modec);
        }
        105 => {
            if (if (*what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                < 'A' as ::core::ffi::c_int
                || *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    > 'Z' as ::core::ffi::c_int
            {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            } else {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            }) == 'n' as ::core::ffi::c_int
            {
                p = highlight_has_attr(id, HL_INVERSE as ::core::ffi::c_int, modec);
            } else {
                p = highlight_has_attr(id, HL_ITALIC as ::core::ffi::c_int, modec);
            }
        }
        110 => {
            if (if (*what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                < 'A' as ::core::ffi::c_int
                || *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    > 'Z' as ::core::ffi::c_int
            {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            } else {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            }) == 'o' as ::core::ffi::c_int
            {
                p = highlight_has_attr(id, HL_NOCOMBINE as ::core::ffi::c_int, modec);
            } else {
                p = get_highlight_name_ext(
                    ::core::ptr::null_mut::<expand_T>(),
                    id - 1 as ::core::ffi::c_int,
                    false_0 != 0,
                );
            }
        }
        114 => {
            p = highlight_has_attr(id, HL_INVERSE as ::core::ffi::c_int, modec);
        }
        115 => {
            if (if (*what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                < 'A' as ::core::ffi::c_int
                || *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    > 'Z' as ::core::ffi::c_int
            {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            } else {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            }) == 'p' as ::core::ffi::c_int
            {
                p = highlight_color(id, what, modec);
            } else if (if (*what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                < 'A' as ::core::ffi::c_int
                || *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    > 'Z' as ::core::ffi::c_int
            {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            } else {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            }) == 't' as ::core::ffi::c_int
                && (if (*what.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    < 'A' as ::core::ffi::c_int
                    || *what.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        > 'Z' as ::core::ffi::c_int
                {
                    *what.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                } else {
                    *what.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                }) == 'r' as ::core::ffi::c_int
            {
                p = highlight_has_attr(id, HL_STRIKETHROUGH as ::core::ffi::c_int, modec);
            } else {
                p = highlight_has_attr(id, HL_STANDOUT as ::core::ffi::c_int, modec);
            }
        }
        117 => {
            if strlen(what) >= 9 as size_t {
                if (if (*what.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    < 'A' as ::core::ffi::c_int
                    || *what.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        > 'Z' as ::core::ffi::c_int
                {
                    *what.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                } else {
                    *what.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                }) == 'l' as ::core::ffi::c_int
                {
                    p = highlight_has_attr(id, HL_UNDERLINE as ::core::ffi::c_int, modec);
                } else if (if (*what.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    < 'A' as ::core::ffi::c_int
                    || *what.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        > 'Z' as ::core::ffi::c_int
                {
                    *what.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                } else {
                    *what.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                }) != 'd' as ::core::ffi::c_int
                {
                    p = highlight_has_attr(id, HL_UNDERCURL as ::core::ffi::c_int, modec);
                } else if (if (*what.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    < 'A' as ::core::ffi::c_int
                    || *what.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        > 'Z' as ::core::ffi::c_int
                {
                    *what.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                } else {
                    *what.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                }) != 'o' as ::core::ffi::c_int
                {
                    p = highlight_has_attr(id, HL_UNDERDASHED as ::core::ffi::c_int, modec);
                } else if (if (*what.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    < 'A' as ::core::ffi::c_int
                    || *what.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        > 'Z' as ::core::ffi::c_int
                {
                    *what.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                } else {
                    *what.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                }) == 'u' as ::core::ffi::c_int
                {
                    p = highlight_has_attr(id, HL_UNDERDOUBLE as ::core::ffi::c_int, modec);
                } else {
                    p = highlight_has_attr(id, HL_UNDERDOTTED as ::core::ffi::c_int, modec);
                }
            } else {
                p = highlight_color(id, what, modec);
            }
        }
        _ => {}
    }
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = if p.is_null() {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    } else {
        xstrdup(p)
    };
}
unsafe extern "C" fn f_synIDtrans(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut id: ::core::ffi::c_int =
        tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
    if id > 0 as ::core::ffi::c_int {
        id = syn_get_final_id(id);
    } else {
        id = 0 as ::core::ffi::c_int;
    }
    (*rettv).vval.v_number = id as varnumber_T;
}
unsafe extern "C" fn f_synconcealed(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut syntax_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut matchid: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut str: [::core::ffi::c_char; 65] = [0; 65];
    tv_list_set_ret(rettv, ::core::ptr::null_mut::<list_T>());
    let lnum: linenr_T = tv_get_lnum(argvars);
    let col: colnr_T =
        tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as colnr_T - 1 as colnr_T;
    memset(
        &raw mut str as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[::core::ffi::c_char; 65]>(),
    );
    if lnum >= 1 as linenr_T
        && lnum <= (*curbuf.get()).b_ml.ml_line_count
        && col >= 0 as ::core::ffi::c_int
        && col <= ml_get_len(lnum)
        && (*curwin.get()).w_onebuf_opt.wo_cole > 0 as OptInt
    {
        syn_get_id(
            curwin.get(),
            lnum,
            col,
            false_0,
            ::core::ptr::null_mut::<bool>(),
            false_0,
        );
        syntax_flags = get_syntax_info(&raw mut matchid);
        if syntax_flags & HL_CONCEAL as ::core::ffi::c_int != 0
            && (*curwin.get()).w_onebuf_opt.wo_cole < 3 as OptInt
        {
            let mut cchar: schar_T = schar_from_char(syn_get_sub_char());
            if cchar == NUL as schar_T && (*curwin.get()).w_onebuf_opt.wo_cole == 1 as OptInt {
                cchar = if (*curwin.get()).w_p_lcs_chars.conceal == NUL as schar_T {
                    ' ' as ::core::ffi::c_int as schar_T
                } else {
                    (*curwin.get()).w_p_lcs_chars.conceal
                };
            }
            if cchar != NUL as schar_T {
                schar_get(&raw mut str as *mut ::core::ffi::c_char, cchar);
            }
        }
    }
    tv_list_alloc_ret(rettv, 3 as ptrdiff_t);
    tv_list_append_number(
        (*rettv).vval.v_list,
        (syntax_flags & HL_CONCEAL as ::core::ffi::c_int != 0 as ::core::ffi::c_int)
            as ::core::ffi::c_int as varnumber_T,
    );
    tv_list_append_string(
        (*rettv).vval.v_list,
        &raw mut str as *mut ::core::ffi::c_char,
        -1 as ssize_t,
    );
    tv_list_append_number((*rettv).vval.v_list, matchid as varnumber_T);
}
unsafe extern "C" fn f_synstack(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_set_ret(rettv, ::core::ptr::null_mut::<list_T>());
    let lnum: linenr_T = tv_get_lnum(argvars);
    let col: colnr_T =
        tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as colnr_T - 1 as colnr_T;
    if lnum >= 1 as linenr_T
        && lnum <= (*curbuf.get()).b_ml.ml_line_count
        && col >= 0 as ::core::ffi::c_int
        && col <= ml_get_len(lnum)
    {
        tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        syn_get_id(
            curwin.get(),
            lnum,
            col,
            false_0,
            ::core::ptr::null_mut::<bool>(),
            true_0,
        );
        let mut id: ::core::ffi::c_int = 0;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        loop {
            let c2rust_fresh6 = i;
            i = i + 1;
            id = syn_get_stack_item(c2rust_fresh6);
            if id < 0 as ::core::ffi::c_int {
                break;
            }
            tv_list_append_number((*rettv).vval.v_list, id as varnumber_T);
        }
    }
}
unsafe extern "C" fn f_tabpagebuflist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        wp = firstwin.get();
    } else {
        let tp: *mut tabpage_T = find_tabpage(tv_get_number(
            argvars.offset(0 as ::core::ffi::c_int as isize),
        ) as ::core::ffi::c_int);
        if !tp.is_null() {
            wp = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
        }
    }
    if !wp.is_null() {
        tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        while !wp.is_null() {
            tv_list_append_number(
                (*rettv).vval.v_list,
                (*(*wp).w_buffer).handle as varnumber_T,
            );
            wp = (*wp).w_next;
        }
    }
}
unsafe extern "C" fn f_tagfiles(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
    let mut fname: *mut ::core::ffi::c_char =
        xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    let mut first: bool = true_0 != 0;
    let mut tn: tagname_T = tagname_T {
        tn_tags: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tn_np: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tn_did_filefind_init: 0,
        tn_hf_idx: 0,
        tn_search_ctx: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    while get_tagfname(&raw mut tn, first as ::core::ffi::c_int, fname) == OK {
        tv_list_append_string((*rettv).vval.v_list, fname, -1 as ssize_t);
        first = false_0 != 0;
    }
    tagname_free(&raw mut tn);
    xfree(fname as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn f_taglist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let tag_pattern: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    (*rettv).vval.v_number = false_0 as varnumber_T;
    if *tag_pattern as ::core::ffi::c_int == NUL {
        return;
    }
    let mut fname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        fname = tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
    }
    get_tags(
        tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t),
        tag_pattern as *mut ::core::ffi::c_char,
        fname as *mut ::core::ffi::c_char,
    );
}
unsafe extern "C" fn f_timer_info(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
    if tv_check_for_opt_number_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut timer: *mut timer_T = find_timer_by_nr(tv_get_number(
            argvars.offset(0 as ::core::ffi::c_int as isize),
        ));
        if !timer.is_null() && (!(*timer).stopped || (*timer).refcount > 1 as ::core::ffi::c_int) {
            add_timer_info(rettv, timer);
        }
    } else {
        add_timer_info_all(rettv);
    };
}
unsafe extern "C" fn f_timer_pause(
    mut argvars: *mut typval_T,
    mut _unused: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(
            &raw const e_number_exp as *const ::core::ffi::c_char,
        ));
        return;
    }
    let mut paused: ::core::ffi::c_int =
        (tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) != 0)
            as ::core::ffi::c_int;
    let mut timer: *mut timer_T = find_timer_by_nr(tv_get_number(
        argvars.offset(0 as ::core::ffi::c_int as isize),
    ));
    if !timer.is_null() {
        if !(*timer).paused && paused != 0 {
            time_watcher_stop(&raw mut (*timer).tw);
        } else if (*timer).paused as ::core::ffi::c_int != 0 && paused == 0 {
            time_watcher_start(
                &raw mut (*timer).tw,
                Some(
                    timer_due_cb
                        as unsafe extern "C" fn(*mut TimeWatcher, *mut ::core::ffi::c_void) -> (),
                ),
                (*timer).timeout as uint64_t,
                (*timer).timeout as uint64_t,
            );
        }
        (*timer).paused = paused != 0;
    }
}
unsafe extern "C" fn f_timer_start(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut repeat: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    (*rettv).vval.v_number = -1 as varnumber_T;
    if check_secure() {
        return;
    }
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if tv_check_for_nonnull_dict_arg(argvars, 2 as ::core::ffi::c_int) == FAIL {
            return;
        }
        let mut dict: *mut dict_T = (*argvars.offset(2 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        let di: *mut dictitem_T = tv_dict_find(
            dict,
            b"repeat\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !di.is_null() {
            repeat = tv_get_number(&raw mut (*di).di_tv) as ::core::ffi::c_int;
            if repeat == 0 as ::core::ffi::c_int {
                repeat = 1 as ::core::ffi::c_int;
            }
        }
    }
    let mut callback: Callback = Callback {
        data: C2Rust_Unnamed_22 {
            funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        type_0: kCallbackNone,
    };
    if !callback_from_typval(
        &raw mut callback,
        argvars.offset(1 as ::core::ffi::c_int as isize),
    ) {
        return;
    }
    (*rettv).vval.v_number = timer_start(
        tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)),
        repeat,
        &raw mut callback,
    ) as varnumber_T;
}
unsafe extern "C" fn f_timer_stop(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if tv_check_for_number_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
        return;
    }
    let mut timer: *mut timer_T = find_timer_by_nr(tv_get_number(
        argvars.offset(0 as ::core::ffi::c_int as isize),
    ));
    if timer.is_null() {
        return;
    }
    timer_stop(timer);
}
unsafe extern "C" fn f_timer_stopall(
    mut _argvars: *mut typval_T,
    mut _unused: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    timer_stop_all();
}
unsafe extern "C" fn f_type(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut n: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint {
        1 => {
            n = VAR_TYPE_NUMBER as ::core::ffi::c_int;
        }
        2 => {
            n = VAR_TYPE_STRING as ::core::ffi::c_int;
        }
        9 | 3 => {
            n = VAR_TYPE_FUNC as ::core::ffi::c_int;
        }
        4 => {
            n = VAR_TYPE_LIST as ::core::ffi::c_int;
        }
        5 => {
            n = VAR_TYPE_DICT as ::core::ffi::c_int;
        }
        6 => {
            n = VAR_TYPE_FLOAT as ::core::ffi::c_int;
        }
        7 => {
            n = VAR_TYPE_BOOL as ::core::ffi::c_int;
        }
        8 => {
            n = VAR_TYPE_SPECIAL as ::core::ffi::c_int;
        }
        10 => {
            n = VAR_TYPE_BLOB as ::core::ffi::c_int;
        }
        0 => {
            internal_error(b"f_type(UNKNOWN)\0".as_ptr() as *const ::core::ffi::c_char);
        }
        _ => {}
    }
    (*rettv).vval.v_number = n as varnumber_T;
}
unsafe extern "C" fn f_virtcol(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut bp: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut fnum: ::core::ffi::c_int = 0;
    let mut fp: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut vcol_start: colnr_T = 0 as colnr_T;
    let mut vcol_end: colnr_T = 0 as colnr_T;
    let mut wp: *mut win_T = curwin.get();
    '_theend: {
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
            wp = win_id2wp_tp(
                tv_get_number(argvars.offset(2 as ::core::ffi::c_int as isize))
                    as ::core::ffi::c_int,
                &raw mut tp,
            );
            if wp.is_null() || tp.is_null() {
                break '_theend;
            } else {
                check_cursor(wp);
            }
        }
        bp = (*wp).w_buffer;
        fnum = (*bp).handle as ::core::ffi::c_int;
        fp = var2fpos(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            false_0 != 0,
            &raw mut fnum,
            false_0 != 0,
            wp,
        );
        if !fp.is_null() && (*fp).lnum <= (*bp).b_ml.ml_line_count && fnum == (*bp).handle {
            if (*fp).col < 0 as ::core::ffi::c_int {
                (*fp).col = 0 as ::core::ffi::c_int as colnr_T;
            } else {
                let len: colnr_T = ml_get_buf_len(bp, (*fp).lnum);
                if (*fp).col > len {
                    (*fp).col = len;
                }
            }
            getvvcol(
                wp,
                fp,
                &raw mut vcol_start,
                ::core::ptr::null_mut::<colnr_T>(),
                &raw mut vcol_end,
            );
            vcol_start += 1;
            vcol_end += 1;
        }
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && tv_get_bool(argvars.offset(1 as ::core::ffi::c_int as isize)) != 0
    {
        tv_list_alloc_ret(rettv, 2 as ptrdiff_t);
        tv_list_append_number((*rettv).vval.v_list, vcol_start as varnumber_T);
        tv_list_append_number((*rettv).vval.v_list, vcol_end as varnumber_T);
    } else {
        (*rettv).vval.v_number = vcol_end as varnumber_T;
    };
}
unsafe extern "C" fn f_visualmode(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut str: [::core::ffi::c_char; 2] = [0; 2];
    (*rettv).v_type = VAR_STRING;
    str[0 as ::core::ffi::c_int as usize] =
        (*curbuf.get()).b_visual_mode_eval as ::core::ffi::c_char;
    str[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    (*rettv).vval.v_string = xstrdup(&raw mut str as *mut ::core::ffi::c_char);
    if non_zero_arg(argvars.offset(0 as ::core::ffi::c_int as isize)) {
        (*curbuf.get()).b_visual_mode_eval = NUL;
    }
}
unsafe extern "C" fn f_wildmenumode(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if wild_menu_showing.get() != 0
        || State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0
            && cmdline_pum_active() as ::core::ffi::c_int != 0
    {
        (*rettv).vval.v_number = 1 as varnumber_T;
    }
}
unsafe extern "C" fn f_windowsversion(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = xstrdup(windowsVersion.ptr() as *mut ::core::ffi::c_char);
}
unsafe extern "C" fn f_wordcount(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_dict_alloc_ret(rettv);
    cursor_pos_info((*rettv).vval.v_dict);
}
unsafe extern "C" fn f_xor(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) ^ tv_get_number_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    );
}
#[inline(always)]
unsafe extern "C" fn tv_list_ref(l: *mut list_T) {
    if l.is_null() {
        return;
    }
    (*l).lv_refcount += 1;
}
#[inline(always)]
unsafe extern "C" fn tv_list_set_ret(tv: *mut typval_T, l: *mut list_T) {
    (*tv).v_type = VAR_LIST;
    (*tv).vval.v_list = l;
    tv_list_ref(l);
}
#[inline]
unsafe extern "C" fn tv_list_locked(l: *const list_T) -> VarLockStatus {
    if l.is_null() {
        return VAR_FIXED;
    }
    return (*l).lv_lock;
}
#[inline]
unsafe extern "C" fn tv_list_set_lock(l: *mut list_T, lock: VarLockStatus) {
    if l.is_null() {
        '_c2rust_label: {
            if lock as ::core::ffi::c_uint == VAR_FIXED as ::core::ffi::c_int as ::core::ffi::c_uint
            {
            } else {
                __assert_fail(
                    b"lock == VAR_FIXED\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/funcs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    76 as ::core::ffi::c_uint,
                    b"void tv_list_set_lock(list_T *const, const VarLockStatus)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        return;
    }
    (*l).lv_lock = lock;
}
#[inline]
unsafe extern "C" fn tv_list_len(l: *const list_T) -> ::core::ffi::c_int {
    if l.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*l).lv_len;
}
#[inline]
unsafe extern "C" fn tv_list_uidx(
    l: *const list_T,
    mut n: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if n < 0 as ::core::ffi::c_int {
        n += tv_list_len(l);
    }
    if n < 0 as ::core::ffi::c_int || n >= tv_list_len(l) {
        return -1 as ::core::ffi::c_int;
    }
    return n;
}
#[inline]
unsafe extern "C" fn tv_list_first(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    return (*l).lv_first;
}
#[inline(always)]
unsafe extern "C" fn tv_dict_set_ret(tv: *mut typval_T, d: *mut dict_T) {
    (*tv).v_type = VAR_DICT;
    (*tv).vval.v_dict = d;
    if !d.is_null() {
        (*d).dv_refcount += 1;
    }
}
#[inline]
unsafe extern "C" fn tv_dict_len(d: *const dict_T) -> ::core::ffi::c_long {
    if d.is_null() {
        return 0 as ::core::ffi::c_long;
    }
    return (*d).dv_hashtab.ht_used as ::core::ffi::c_long;
}
#[inline]
unsafe extern "C" fn tv_blob_len(b: *const blob_T) -> ::core::ffi::c_int {
    if b.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*b).bv_ga.ga_len;
}
#[inline(always)]
unsafe extern "C" fn tv_blob_get(b: *const blob_T, mut idx: ::core::ffi::c_int) -> uint8_t {
    return *((*b).bv_ga.ga_data as *mut uint8_t).offset(idx as isize);
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
#[inline(always)]
unsafe extern "C" fn tv_is_func(tv: typval_T) -> bool {
    return tv.v_type as ::core::ffi::c_uint
        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
        || tv.v_type as ::core::ffi::c_uint
            == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint;
}
pub const TV_TRANSLATE: ::core::ffi::c_ulong = SIZE_MAX;
#[inline]
unsafe extern "C" fn proc_is_stopped(mut proc: *mut Proc) -> bool {
    let mut exited: bool = (*proc).status >= 0 as ::core::ffi::c_int;
    return exited as ::core::ffi::c_int != 0 || (*proc).stopped_time != 0 as uint64_t;
}
#[inline]
unsafe extern "C" fn get_register_name(mut num: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if num == -1 as ::core::ffi::c_int {
        return '"' as ::core::ffi::c_int;
    } else if num < 10 as ::core::ffi::c_int {
        return num + '0' as ::core::ffi::c_int;
    } else if num == DELETION_REGISTER as ::core::ffi::c_int {
        return '-' as ::core::ffi::c_int;
    } else if num == STAR_REGISTER as ::core::ffi::c_int {
        return '*' as ::core::ffi::c_int;
    } else if num == PLUS_REGISTER as ::core::ffi::c_int {
        return '+' as ::core::ffi::c_int;
    } else {
        return num + 'a' as ::core::ffi::c_int - 10 as ::core::ffi::c_int;
    };
}
pub const FNE_CHECK_START: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const AUTOLOAD_CHAR: ::core::ffi::c_int = '#' as ::core::ffi::c_int;
static functions: GlobalCell<[EvalFuncDef; 644]> = GlobalCell::new(
    [EvalFuncDef {
        name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        min_argc: 0,
        max_argc: 0,
        base_arg: 0,
        fast: false,
        func: None,
        data: EvalFuncData { float_func: None },
    }; 644],
);
pub unsafe extern "C" fn find_internal_func_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut high: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    match len {
        2 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            105 => {
                low = 0 as ::core::ffi::c_int;
                high = 1 as ::core::ffi::c_int;
            }
            111 => {
                low = 1 as ::core::ffi::c_int;
                high = 2 as ::core::ffi::c_int;
            }
            116 => {
                low = 2 as ::core::ffi::c_int;
                high = 3 as ::core::ffi::c_int;
            }
            _ => {}
        },
        3 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            97 => {
                low = 3 as ::core::ffi::c_int;
                high = 6 as ::core::ffi::c_int;
            }
            99 => {
                low = 6 as ::core::ffi::c_int;
                high = 8 as ::core::ffi::c_int;
            }
            101 => {
                low = 8 as ::core::ffi::c_int;
                high = 9 as ::core::ffi::c_int;
            }
            103 => {
                low = 9 as ::core::ffi::c_int;
                high = 10 as ::core::ffi::c_int;
            }
            104 => {
                low = 10 as ::core::ffi::c_int;
                high = 11 as ::core::ffi::c_int;
            }
            108 => {
                low = 11 as ::core::ffi::c_int;
                high = 13 as ::core::ffi::c_int;
            }
            109 => {
                low = 13 as ::core::ffi::c_int;
                high = 16 as ::core::ffi::c_int;
            }
            112 => {
                low = 16 as ::core::ffi::c_int;
                high = 17 as ::core::ffi::c_int;
            }
            115 => {
                low = 17 as ::core::ffi::c_int;
                high = 18 as ::core::ffi::c_int;
            }
            116 => {
                low = 18 as ::core::ffi::c_int;
                high = 19 as ::core::ffi::c_int;
            }
            120 => {
                low = 19 as ::core::ffi::c_int;
                high = 20 as ::core::ffi::c_int;
            }
            _ => {}
        },
        4 => match *str.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            68 => {
                low = 20 as ::core::ffi::c_int;
                high = 21 as ::core::ffi::c_int;
            }
            98 => {
                low = 21 as ::core::ffi::c_int;
                high = 22 as ::core::ffi::c_int;
            }
            99 => {
                low = 22 as ::core::ffi::c_int;
                high = 23 as ::core::ffi::c_int;
            }
            100 => {
                low = 23 as ::core::ffi::c_int;
                high = 25 as ::core::ffi::c_int;
            }
            101 => {
                low = 25 as ::core::ffi::c_int;
                high = 28 as ::core::ffi::c_int;
            }
            104 => {
                low = 28 as ::core::ffi::c_int;
                high = 31 as ::core::ffi::c_int;
            }
            108 => {
                low = 31 as ::core::ffi::c_int;
                high = 34 as ::core::ffi::c_int;
            }
            109 => {
                low = 34 as ::core::ffi::c_int;
                high = 35 as ::core::ffi::c_int;
            }
            110 => {
                low = 35 as ::core::ffi::c_int;
                high = 38 as ::core::ffi::c_int;
            }
            113 => {
                low = 38 as ::core::ffi::c_int;
                high = 39 as ::core::ffi::c_int;
            }
            115 => {
                low = 39 as ::core::ffi::c_int;
                high = 41 as ::core::ffi::c_int;
            }
            116 => {
                low = 41 as ::core::ffi::c_int;
                high = 44 as ::core::ffi::c_int;
            }
            118 => {
                low = 44 as ::core::ffi::c_int;
                high = 45 as ::core::ffi::c_int;
            }
            121 => {
                low = 45 as ::core::ffi::c_int;
                high = 46 as ::core::ffi::c_int;
            }
            _ => {}
        },
        5 => match *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            97 => {
                low = 46 as ::core::ffi::c_int;
                high = 48 as ::core::ffi::c_int;
            }
            99 => {
                low = 48 as ::core::ffi::c_int;
                high = 49 as ::core::ffi::c_int;
            }
            104 => {
                low = 49 as ::core::ffi::c_int;
                high = 50 as ::core::ffi::c_int;
            }
            105 => {
                low = 50 as ::core::ffi::c_int;
                high = 51 as ::core::ffi::c_int;
            }
            107 => {
                low = 51 as ::core::ffi::c_int;
                high = 52 as ::core::ffi::c_int;
            }
            108 => {
                low = 52 as ::core::ffi::c_int;
                high = 54 as ::core::ffi::c_int;
            }
            109 => {
                low = 54 as ::core::ffi::c_int;
                high = 55 as ::core::ffi::c_int;
            }
            110 => {
                low = 55 as ::core::ffi::c_int;
                high = 57 as ::core::ffi::c_int;
            }
            111 => {
                low = 57 as ::core::ffi::c_int;
                high = 60 as ::core::ffi::c_int;
            }
            112 => {
                low = 60 as ::core::ffi::c_int;
                high = 61 as ::core::ffi::c_int;
            }
            114 => {
                low = 61 as ::core::ffi::c_int;
                high = 63 as ::core::ffi::c_int;
            }
            115 => {
                low = 63 as ::core::ffi::c_int;
                high = 65 as ::core::ffi::c_int;
            }
            116 => {
                low = 65 as ::core::ffi::c_int;
                high = 68 as ::core::ffi::c_int;
            }
            117 => {
                low = 68 as ::core::ffi::c_int;
                high = 69 as ::core::ffi::c_int;
            }
            121 => {
                low = 69 as ::core::ffi::c_int;
                high = 70 as ::core::ffi::c_int;
            }
            _ => {}
        },
        6 => match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            54 => {
                low = 70 as ::core::ffi::c_int;
                high = 71 as ::core::ffi::c_int;
            }
            100 => {
                low = 71 as ::core::ffi::c_int;
                high = 78 as ::core::ffi::c_int;
            }
            101 => {
                low = 78 as ::core::ffi::c_int;
                high = 84 as ::core::ffi::c_int;
            }
            102 => {
                low = 84 as ::core::ffi::c_int;
                high = 85 as ::core::ffi::c_int;
            }
            103 => {
                low = 85 as ::core::ffi::c_int;
                high = 89 as ::core::ffi::c_int;
            }
            104 => {
                low = 89 as ::core::ffi::c_int;
                high = 90 as ::core::ffi::c_int;
            }
            108 => {
                low = 90 as ::core::ffi::c_int;
                high = 92 as ::core::ffi::c_int;
            }
            109 => {
                low = 92 as ::core::ffi::c_int;
                high = 93 as ::core::ffi::c_int;
            }
            110 => {
                low = 93 as ::core::ffi::c_int;
                high = 94 as ::core::ffi::c_int;
            }
            112 => {
                low = 94 as ::core::ffi::c_int;
                high = 95 as ::core::ffi::c_int;
            }
            114 => {
                low = 95 as ::core::ffi::c_int;
                high = 99 as ::core::ffi::c_int;
            }
            115 => {
                low = 99 as ::core::ffi::c_int;
                high = 103 as ::core::ffi::c_int;
            }
            116 => {
                low = 103 as ::core::ffi::c_int;
                high = 110 as ::core::ffi::c_int;
            }
            118 => {
                low = 110 as ::core::ffi::c_int;
                high = 112 as ::core::ffi::c_int;
            }
            119 => {
                low = 112 as ::core::ffi::c_int;
                high = 113 as ::core::ffi::c_int;
            }
            120 => {
                low = 113 as ::core::ffi::c_int;
                high = 115 as ::core::ffi::c_int;
            }
            _ => {}
        },
        7 => match *str.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            50 => {
                low = 115 as ::core::ffi::c_int;
                high = 116 as ::core::ffi::c_int;
            }
            51 => {
                low = 116 as ::core::ffi::c_int;
                high = 117 as ::core::ffi::c_int;
            }
            97 => {
                low = 117 as ::core::ffi::c_int;
                high = 123 as ::core::ffi::c_int;
            }
            98 => {
                low = 123 as ::core::ffi::c_int;
                high = 127 as ::core::ffi::c_int;
            }
            99 => {
                low = 127 as ::core::ffi::c_int;
                high = 128 as ::core::ffi::c_int;
            }
            100 => {
                low = 128 as ::core::ffi::c_int;
                high = 130 as ::core::ffi::c_int;
            }
            101 => {
                low = 130 as ::core::ffi::c_int;
                high = 132 as ::core::ffi::c_int;
            }
            102 => {
                low = 132 as ::core::ffi::c_int;
                high = 134 as ::core::ffi::c_int;
            }
            103 => {
                low = 134 as ::core::ffi::c_int;
                high = 135 as ::core::ffi::c_int;
            }
            108 => {
                low = 135 as ::core::ffi::c_int;
                high = 137 as ::core::ffi::c_int;
            }
            110 => {
                low = 137 as ::core::ffi::c_int;
                high = 142 as ::core::ffi::c_int;
            }
            112 => {
                low = 142 as ::core::ffi::c_int;
                high = 143 as ::core::ffi::c_int;
            }
            114 => {
                low = 143 as ::core::ffi::c_int;
                high = 147 as ::core::ffi::c_int;
            }
            115 => {
                low = 147 as ::core::ffi::c_int;
                high = 152 as ::core::ffi::c_int;
            }
            116 => {
                low = 152 as ::core::ffi::c_int;
                high = 157 as ::core::ffi::c_int;
            }
            117 => {
                low = 157 as ::core::ffi::c_int;
                high = 158 as ::core::ffi::c_int;
            }
            118 => {
                low = 158 as ::core::ffi::c_int;
                high = 160 as ::core::ffi::c_int;
            }
            120 => {
                low = 160 as ::core::ffi::c_int;
                high = 163 as ::core::ffi::c_int;
            }
            _ => {}
        },
        8 => match *str.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            49 => {
                low = 163 as ::core::ffi::c_int;
                high = 164 as ::core::ffi::c_int;
            }
            50 => {
                low = 164 as ::core::ffi::c_int;
                high = 165 as ::core::ffi::c_int;
            }
            95 => {
                low = 165 as ::core::ffi::c_int;
                high = 166 as ::core::ffi::c_int;
            }
            97 => {
                low = 166 as ::core::ffi::c_int;
                high = 167 as ::core::ffi::c_int;
            }
            98 => {
                low = 167 as ::core::ffi::c_int;
                high = 169 as ::core::ffi::c_int;
            }
            99 => {
                low = 169 as ::core::ffi::c_int;
                high = 177 as ::core::ffi::c_int;
            }
            100 => {
                low = 177 as ::core::ffi::c_int;
                high = 182 as ::core::ffi::c_int;
            }
            101 => {
                low = 182 as ::core::ffi::c_int;
                high = 183 as ::core::ffi::c_int;
            }
            102 => {
                low = 183 as ::core::ffi::c_int;
                high = 190 as ::core::ffi::c_int;
            }
            108 => {
                low = 190 as ::core::ffi::c_int;
                high = 191 as ::core::ffi::c_int;
            }
            109 => {
                low = 191 as ::core::ffi::c_int;
                high = 197 as ::core::ffi::c_int;
            }
            110 => {
                low = 197 as ::core::ffi::c_int;
                high = 199 as ::core::ffi::c_int;
            }
            111 => {
                low = 199 as ::core::ffi::c_int;
                high = 202 as ::core::ffi::c_int;
            }
            112 => {
                low = 202 as ::core::ffi::c_int;
                high = 209 as ::core::ffi::c_int;
            }
            115 => {
                low = 209 as ::core::ffi::c_int;
                high = 212 as ::core::ffi::c_int;
            }
            116 => {
                low = 212 as ::core::ffi::c_int;
                high = 216 as ::core::ffi::c_int;
            }
            117 => {
                low = 216 as ::core::ffi::c_int;
                high = 217 as ::core::ffi::c_int;
            }
            119 => {
                low = 217 as ::core::ffi::c_int;
                high = 221 as ::core::ffi::c_int;
            }
            120 => {
                low = 221 as ::core::ffi::c_int;
                high = 222 as ::core::ffi::c_int;
            }
            121 => {
                low = 222 as ::core::ffi::c_int;
                high = 223 as ::core::ffi::c_int;
            }
            _ => {}
        },
        9 => match *str.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            50 => {
                low = 223 as ::core::ffi::c_int;
                high = 227 as ::core::ffi::c_int;
            }
            68 => {
                low = 227 as ::core::ffi::c_int;
                high = 228 as ::core::ffi::c_int;
            }
            95 => {
                low = 228 as ::core::ffi::c_int;
                high = 234 as ::core::ffi::c_int;
            }
            97 => {
                low = 234 as ::core::ffi::c_int;
                high = 239 as ::core::ffi::c_int;
            }
            99 => {
                low = 239 as ::core::ffi::c_int;
                high = 243 as ::core::ffi::c_int;
            }
            100 => {
                low = 243 as ::core::ffi::c_int;
                high = 244 as ::core::ffi::c_int;
            }
            101 => {
                low = 244 as ::core::ffi::c_int;
                high = 251 as ::core::ffi::c_int;
            }
            102 => {
                low = 251 as ::core::ffi::c_int;
                high = 254 as ::core::ffi::c_int;
            }
            103 => {
                low = 254 as ::core::ffi::c_int;
                high = 255 as ::core::ffi::c_int;
            }
            104 => {
                low = 255 as ::core::ffi::c_int;
                high = 256 as ::core::ffi::c_int;
            }
            105 => {
                low = 256 as ::core::ffi::c_int;
                high = 261 as ::core::ffi::c_int;
            }
            108 => {
                low = 261 as ::core::ffi::c_int;
                high = 263 as ::core::ffi::c_int;
            }
            109 => {
                low = 263 as ::core::ffi::c_int;
                high = 265 as ::core::ffi::c_int;
            }
            110 => {
                low = 265 as ::core::ffi::c_int;
                high = 267 as ::core::ffi::c_int;
            }
            111 => {
                low = 267 as ::core::ffi::c_int;
                high = 270 as ::core::ffi::c_int;
            }
            114 => {
                low = 270 as ::core::ffi::c_int;
                high = 271 as ::core::ffi::c_int;
            }
            115 => {
                low = 271 as ::core::ffi::c_int;
                high = 272 as ::core::ffi::c_int;
            }
            116 => {
                low = 272 as ::core::ffi::c_int;
                high = 274 as ::core::ffi::c_int;
            }
            117 => {
                low = 274 as ::core::ffi::c_int;
                high = 277 as ::core::ffi::c_int;
            }
            120 => {
                low = 277 as ::core::ffi::c_int;
                high = 278 as ::core::ffi::c_int;
            }
            _ => {}
        },
        10 => match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 278 as ::core::ffi::c_int;
                high = 280 as ::core::ffi::c_int;
            }
            97 => {
                low = 280 as ::core::ffi::c_int;
                high = 285 as ::core::ffi::c_int;
            }
            98 => {
                low = 285 as ::core::ffi::c_int;
                high = 287 as ::core::ffi::c_int;
            }
            99 => {
                low = 287 as ::core::ffi::c_int;
                high = 289 as ::core::ffi::c_int;
            }
            100 => {
                low = 289 as ::core::ffi::c_int;
                high = 293 as ::core::ffi::c_int;
            }
            101 => {
                low = 293 as ::core::ffi::c_int;
                high = 296 as ::core::ffi::c_int;
            }
            102 => {
                low = 296 as ::core::ffi::c_int;
                high = 300 as ::core::ffi::c_int;
            }
            103 => {
                low = 300 as ::core::ffi::c_int;
                high = 302 as ::core::ffi::c_int;
            }
            104 => {
                low = 302 as ::core::ffi::c_int;
                high = 304 as ::core::ffi::c_int;
            }
            105 => {
                low = 304 as ::core::ffi::c_int;
                high = 306 as ::core::ffi::c_int;
            }
            108 => {
                low = 306 as ::core::ffi::c_int;
                high = 308 as ::core::ffi::c_int;
            }
            109 => {
                low = 308 as ::core::ffi::c_int;
                high = 310 as ::core::ffi::c_int;
            }
            110 => {
                low = 310 as ::core::ffi::c_int;
                high = 316 as ::core::ffi::c_int;
            }
            111 => {
                low = 316 as ::core::ffi::c_int;
                high = 317 as ::core::ffi::c_int;
            }
            112 => {
                low = 317 as ::core::ffi::c_int;
                high = 319 as ::core::ffi::c_int;
            }
            113 => {
                low = 319 as ::core::ffi::c_int;
                high = 320 as ::core::ffi::c_int;
            }
            114 => {
                low = 320 as ::core::ffi::c_int;
                high = 323 as ::core::ffi::c_int;
            }
            115 => {
                low = 323 as ::core::ffi::c_int;
                high = 325 as ::core::ffi::c_int;
            }
            116 => {
                low = 325 as ::core::ffi::c_int;
                high = 330 as ::core::ffi::c_int;
            }
            119 => {
                low = 330 as ::core::ffi::c_int;
                high = 331 as ::core::ffi::c_int;
            }
            _ => {}
        },
        11 => match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 331 as ::core::ffi::c_int;
                high = 334 as ::core::ffi::c_int;
            }
            97 => {
                low = 334 as ::core::ffi::c_int;
                high = 336 as ::core::ffi::c_int;
            }
            99 => {
                low = 336 as ::core::ffi::c_int;
                high = 338 as ::core::ffi::c_int;
            }
            100 => {
                low = 338 as ::core::ffi::c_int;
                high = 343 as ::core::ffi::c_int;
            }
            101 => {
                low = 343 as ::core::ffi::c_int;
                high = 348 as ::core::ffi::c_int;
            }
            102 => {
                low = 348 as ::core::ffi::c_int;
                high = 350 as ::core::ffi::c_int;
            }
            103 => {
                low = 350 as ::core::ffi::c_int;
                high = 353 as ::core::ffi::c_int;
            }
            104 => {
                low = 353 as ::core::ffi::c_int;
                high = 355 as ::core::ffi::c_int;
            }
            105 => {
                low = 355 as ::core::ffi::c_int;
                high = 357 as ::core::ffi::c_int;
            }
            109 => {
                low = 357 as ::core::ffi::c_int;
                high = 359 as ::core::ffi::c_int;
            }
            110 => {
                low = 359 as ::core::ffi::c_int;
                high = 362 as ::core::ffi::c_int;
            }
            111 => {
                low = 362 as ::core::ffi::c_int;
                high = 365 as ::core::ffi::c_int;
            }
            112 => {
                low = 365 as ::core::ffi::c_int;
                high = 367 as ::core::ffi::c_int;
            }
            114 => {
                low = 367 as ::core::ffi::c_int;
                high = 372 as ::core::ffi::c_int;
            }
            115 => {
                low = 372 as ::core::ffi::c_int;
                high = 377 as ::core::ffi::c_int;
            }
            116 => {
                low = 377 as ::core::ffi::c_int;
                high = 378 as ::core::ffi::c_int;
            }
            117 => {
                low = 378 as ::core::ffi::c_int;
                high = 379 as ::core::ffi::c_int;
            }
            118 => {
                low = 379 as ::core::ffi::c_int;
                high = 380 as ::core::ffi::c_int;
            }
            120 => {
                low = 380 as ::core::ffi::c_int;
                high = 381 as ::core::ffi::c_int;
            }
            _ => {}
        },
        12 => match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 381 as ::core::ffi::c_int;
                high = 385 as ::core::ffi::c_int;
            }
            98 => {
                low = 385 as ::core::ffi::c_int;
                high = 389 as ::core::ffi::c_int;
            }
            99 => {
                low = 389 as ::core::ffi::c_int;
                high = 391 as ::core::ffi::c_int;
            }
            100 => {
                low = 391 as ::core::ffi::c_int;
                high = 393 as ::core::ffi::c_int;
            }
            101 => {
                low = 393 as ::core::ffi::c_int;
                high = 397 as ::core::ffi::c_int;
            }
            103 => {
                low = 397 as ::core::ffi::c_int;
                high = 400 as ::core::ffi::c_int;
            }
            104 => {
                low = 400 as ::core::ffi::c_int;
                high = 401 as ::core::ffi::c_int;
            }
            105 => {
                low = 401 as ::core::ffi::c_int;
                high = 403 as ::core::ffi::c_int;
            }
            109 => {
                low = 403 as ::core::ffi::c_int;
                high = 405 as ::core::ffi::c_int;
            }
            110 => {
                low = 405 as ::core::ffi::c_int;
                high = 407 as ::core::ffi::c_int;
            }
            111 => {
                low = 407 as ::core::ffi::c_int;
                high = 409 as ::core::ffi::c_int;
            }
            114 => {
                low = 409 as ::core::ffi::c_int;
                high = 411 as ::core::ffi::c_int;
            }
            115 => {
                low = 411 as ::core::ffi::c_int;
                high = 414 as ::core::ffi::c_int;
            }
            116 => {
                low = 414 as ::core::ffi::c_int;
                high = 419 as ::core::ffi::c_int;
            }
            117 => {
                low = 419 as ::core::ffi::c_int;
                high = 421 as ::core::ffi::c_int;
            }
            _ => {}
        },
        13 => match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 421 as ::core::ffi::c_int;
                high = 423 as ::core::ffi::c_int;
            }
            97 => {
                low = 423 as ::core::ffi::c_int;
                high = 427 as ::core::ffi::c_int;
            }
            99 => {
                low = 427 as ::core::ffi::c_int;
                high = 428 as ::core::ffi::c_int;
            }
            100 => {
                low = 428 as ::core::ffi::c_int;
                high = 432 as ::core::ffi::c_int;
            }
            101 => {
                low = 432 as ::core::ffi::c_int;
                high = 435 as ::core::ffi::c_int;
            }
            102 => {
                low = 435 as ::core::ffi::c_int;
                high = 438 as ::core::ffi::c_int;
            }
            103 => {
                low = 438 as ::core::ffi::c_int;
                high = 442 as ::core::ffi::c_int;
            }
            104 => {
                low = 442 as ::core::ffi::c_int;
                high = 443 as ::core::ffi::c_int;
            }
            108 => {
                low = 443 as ::core::ffi::c_int;
                high = 447 as ::core::ffi::c_int;
            }
            109 => {
                low = 447 as ::core::ffi::c_int;
                high = 448 as ::core::ffi::c_int;
            }
            111 => {
                low = 448 as ::core::ffi::c_int;
                high = 449 as ::core::ffi::c_int;
            }
            112 => {
                low = 449 as ::core::ffi::c_int;
                high = 450 as ::core::ffi::c_int;
            }
            114 => {
                low = 450 as ::core::ffi::c_int;
                high = 454 as ::core::ffi::c_int;
            }
            115 => {
                low = 454 as ::core::ffi::c_int;
                high = 456 as ::core::ffi::c_int;
            }
            116 => {
                low = 456 as ::core::ffi::c_int;
                high = 458 as ::core::ffi::c_int;
            }
            117 => {
                low = 458 as ::core::ffi::c_int;
                high = 459 as ::core::ffi::c_int;
            }
            119 => {
                low = 459 as ::core::ffi::c_int;
                high = 460 as ::core::ffi::c_int;
            }
            120 => {
                low = 460 as ::core::ffi::c_int;
                high = 461 as ::core::ffi::c_int;
            }
            _ => {}
        },
        14 => match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 461 as ::core::ffi::c_int;
                high = 463 as ::core::ffi::c_int;
            }
            97 => {
                low = 463 as ::core::ffi::c_int;
                high = 465 as ::core::ffi::c_int;
            }
            98 => {
                low = 465 as ::core::ffi::c_int;
                high = 466 as ::core::ffi::c_int;
            }
            100 => {
                low = 466 as ::core::ffi::c_int;
                high = 467 as ::core::ffi::c_int;
            }
            101 => {
                low = 467 as ::core::ffi::c_int;
                high = 470 as ::core::ffi::c_int;
            }
            103 => {
                low = 470 as ::core::ffi::c_int;
                high = 474 as ::core::ffi::c_int;
            }
            108 => {
                low = 474 as ::core::ffi::c_int;
                high = 476 as ::core::ffi::c_int;
            }
            111 => {
                low = 476 as ::core::ffi::c_int;
                high = 479 as ::core::ffi::c_int;
            }
            112 => {
                low = 479 as ::core::ffi::c_int;
                high = 481 as ::core::ffi::c_int;
            }
            115 => {
                low = 481 as ::core::ffi::c_int;
                high = 482 as ::core::ffi::c_int;
            }
            116 => {
                low = 482 as ::core::ffi::c_int;
                high = 483 as ::core::ffi::c_int;
            }
            119 => {
                low = 483 as ::core::ffi::c_int;
                high = 485 as ::core::ffi::c_int;
            }
            _ => {}
        },
        15 => match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 485 as ::core::ffi::c_int;
                high = 486 as ::core::ffi::c_int;
            }
            98 => {
                low = 486 as ::core::ffi::c_int;
                high = 488 as ::core::ffi::c_int;
            }
            99 => {
                low = 488 as ::core::ffi::c_int;
                high = 489 as ::core::ffi::c_int;
            }
            100 => {
                low = 489 as ::core::ffi::c_int;
                high = 492 as ::core::ffi::c_int;
            }
            103 => {
                low = 492 as ::core::ffi::c_int;
                high = 495 as ::core::ffi::c_int;
            }
            108 => {
                low = 495 as ::core::ffi::c_int;
                high = 496 as ::core::ffi::c_int;
            }
            112 => {
                low = 496 as ::core::ffi::c_int;
                high = 498 as ::core::ffi::c_int;
            }
            115 => {
                low = 498 as ::core::ffi::c_int;
                high = 501 as ::core::ffi::c_int;
            }
            116 => {
                low = 501 as ::core::ffi::c_int;
                high = 504 as ::core::ffi::c_int;
            }
            _ => {}
        },
        16 => match *str.offset(9 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 504 as ::core::ffi::c_int;
                high = 505 as ::core::ffi::c_int;
            }
            97 => {
                low = 505 as ::core::ffi::c_int;
                high = 507 as ::core::ffi::c_int;
            }
            99 => {
                low = 507 as ::core::ffi::c_int;
                high = 511 as ::core::ffi::c_int;
            }
            100 => {
                low = 511 as ::core::ffi::c_int;
                high = 513 as ::core::ffi::c_int;
            }
            101 => {
                low = 513 as ::core::ffi::c_int;
                high = 514 as ::core::ffi::c_int;
            }
            103 => {
                low = 514 as ::core::ffi::c_int;
                high = 517 as ::core::ffi::c_int;
            }
            112 => {
                low = 517 as ::core::ffi::c_int;
                high = 518 as ::core::ffi::c_int;
            }
            115 => {
                low = 518 as ::core::ffi::c_int;
                high = 521 as ::core::ffi::c_int;
            }
            116 => {
                low = 521 as ::core::ffi::c_int;
                high = 525 as ::core::ffi::c_int;
            }
            117 => {
                low = 525 as ::core::ffi::c_int;
                high = 526 as ::core::ffi::c_int;
            }
            119 => {
                low = 526 as ::core::ffi::c_int;
                high = 527 as ::core::ffi::c_int;
            }
            _ => {}
        },
        17 => match *str.offset(9 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 527 as ::core::ffi::c_int;
                high = 531 as ::core::ffi::c_int;
            }
            97 => {
                low = 531 as ::core::ffi::c_int;
                high = 532 as ::core::ffi::c_int;
            }
            99 => {
                low = 532 as ::core::ffi::c_int;
                high = 533 as ::core::ffi::c_int;
            }
            100 => {
                low = 533 as ::core::ffi::c_int;
                high = 534 as ::core::ffi::c_int;
            }
            103 => {
                low = 534 as ::core::ffi::c_int;
                high = 537 as ::core::ffi::c_int;
            }
            104 => {
                low = 537 as ::core::ffi::c_int;
                high = 538 as ::core::ffi::c_int;
            }
            105 => {
                low = 538 as ::core::ffi::c_int;
                high = 540 as ::core::ffi::c_int;
            }
            115 => {
                low = 540 as ::core::ffi::c_int;
                high = 543 as ::core::ffi::c_int;
            }
            116 => {
                low = 543 as ::core::ffi::c_int;
                high = 544 as ::core::ffi::c_int;
            }
            _ => {}
        },
        18 => match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 544 as ::core::ffi::c_int;
                high = 546 as ::core::ffi::c_int;
            }
            98 => {
                low = 546 as ::core::ffi::c_int;
                high = 549 as ::core::ffi::c_int;
            }
            99 => {
                low = 549 as ::core::ffi::c_int;
                high = 550 as ::core::ffi::c_int;
            }
            101 => {
                low = 550 as ::core::ffi::c_int;
                high = 551 as ::core::ffi::c_int;
            }
            103 => {
                low = 551 as ::core::ffi::c_int;
                high = 553 as ::core::ffi::c_int;
            }
            108 => {
                low = 553 as ::core::ffi::c_int;
                high = 554 as ::core::ffi::c_int;
            }
            111 => {
                low = 554 as ::core::ffi::c_int;
                high = 555 as ::core::ffi::c_int;
            }
            116 => {
                low = 555 as ::core::ffi::c_int;
                high = 556 as ::core::ffi::c_int;
            }
            119 => {
                low = 556 as ::core::ffi::c_int;
                high = 559 as ::core::ffi::c_int;
            }
            _ => {}
        },
        19 => match *str.offset(14 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 559 as ::core::ffi::c_int;
                high = 561 as ::core::ffi::c_int;
            }
            99 => {
                low = 561 as ::core::ffi::c_int;
                high = 562 as ::core::ffi::c_int;
            }
            101 => {
                low = 562 as ::core::ffi::c_int;
                high = 567 as ::core::ffi::c_int;
            }
            102 => {
                low = 567 as ::core::ffi::c_int;
                high = 568 as ::core::ffi::c_int;
            }
            103 => {
                low = 568 as ::core::ffi::c_int;
                high = 569 as ::core::ffi::c_int;
            }
            111 => {
                low = 569 as ::core::ffi::c_int;
                high = 572 as ::core::ffi::c_int;
            }
            112 => {
                low = 572 as ::core::ffi::c_int;
                high = 577 as ::core::ffi::c_int;
            }
            114 => {
                low = 577 as ::core::ffi::c_int;
                high = 578 as ::core::ffi::c_int;
            }
            115 => {
                low = 578 as ::core::ffi::c_int;
                high = 579 as ::core::ffi::c_int;
            }
            116 => {
                low = 579 as ::core::ffi::c_int;
                high = 581 as ::core::ffi::c_int;
            }
            117 => {
                low = 581 as ::core::ffi::c_int;
                high = 586 as ::core::ffi::c_int;
            }
            _ => {}
        },
        20 => match *str.offset(17 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            97 => {
                low = 586 as ::core::ffi::c_int;
                high = 589 as ::core::ffi::c_int;
            }
            98 => {
                low = 589 as ::core::ffi::c_int;
                high = 591 as ::core::ffi::c_int;
            }
            100 => {
                low = 591 as ::core::ffi::c_int;
                high = 592 as ::core::ffi::c_int;
            }
            103 => {
                low = 592 as ::core::ffi::c_int;
                high = 593 as ::core::ffi::c_int;
            }
            105 => {
                low = 593 as ::core::ffi::c_int;
                high = 594 as ::core::ffi::c_int;
            }
            110 => {
                low = 594 as ::core::ffi::c_int;
                high = 595 as ::core::ffi::c_int;
            }
            118 => {
                low = 595 as ::core::ffi::c_int;
                high = 598 as ::core::ffi::c_int;
            }
            119 => {
                low = 598 as ::core::ffi::c_int;
                high = 602 as ::core::ffi::c_int;
            }
            _ => {}
        },
        21 => match *str.offset(9 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            97 => {
                low = 602 as ::core::ffi::c_int;
                high = 603 as ::core::ffi::c_int;
            }
            99 => {
                low = 603 as ::core::ffi::c_int;
                high = 606 as ::core::ffi::c_int;
            }
            101 => {
                low = 606 as ::core::ffi::c_int;
                high = 607 as ::core::ffi::c_int;
            }
            103 => {
                low = 607 as ::core::ffi::c_int;
                high = 610 as ::core::ffi::c_int;
            }
            111 => {
                low = 610 as ::core::ffi::c_int;
                high = 613 as ::core::ffi::c_int;
            }
            114 => {
                low = 613 as ::core::ffi::c_int;
                high = 614 as ::core::ffi::c_int;
            }
            116 => {
                low = 614 as ::core::ffi::c_int;
                high = 616 as ::core::ffi::c_int;
            }
            117 => {
                low = 616 as ::core::ffi::c_int;
                high = 617 as ::core::ffi::c_int;
            }
            _ => {}
        },
        22 => match *str.offset(10 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 617 as ::core::ffi::c_int;
                high = 618 as ::core::ffi::c_int;
            }
            100 => {
                low = 618 as ::core::ffi::c_int;
                high = 619 as ::core::ffi::c_int;
            }
            103 => {
                low = 619 as ::core::ffi::c_int;
                high = 620 as ::core::ffi::c_int;
            }
            108 => {
                low = 620 as ::core::ffi::c_int;
                high = 621 as ::core::ffi::c_int;
            }
            111 => {
                low = 621 as ::core::ffi::c_int;
                high = 622 as ::core::ffi::c_int;
            }
            114 => {
                low = 622 as ::core::ffi::c_int;
                high = 623 as ::core::ffi::c_int;
            }
            117 => {
                low = 623 as ::core::ffi::c_int;
                high = 624 as ::core::ffi::c_int;
            }
            _ => {}
        },
        23 => match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 624 as ::core::ffi::c_int;
                high = 625 as ::core::ffi::c_int;
            }
            103 => {
                low = 625 as ::core::ffi::c_int;
                high = 626 as ::core::ffi::c_int;
            }
            108 => {
                low = 626 as ::core::ffi::c_int;
                high = 627 as ::core::ffi::c_int;
            }
            116 => {
                low = 627 as ::core::ffi::c_int;
                high = 628 as ::core::ffi::c_int;
            }
            _ => {}
        },
        24 => match *str.offset(13 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 628 as ::core::ffi::c_int;
                high = 629 as ::core::ffi::c_int;
            }
            101 => {
                low = 629 as ::core::ffi::c_int;
                high = 631 as ::core::ffi::c_int;
            }
            111 => {
                low = 631 as ::core::ffi::c_int;
                high = 632 as ::core::ffi::c_int;
            }
            114 => {
                low = 632 as ::core::ffi::c_int;
                high = 634 as ::core::ffi::c_int;
            }
            115 => {
                low = 634 as ::core::ffi::c_int;
                high = 635 as ::core::ffi::c_int;
            }
            117 => {
                low = 635 as ::core::ffi::c_int;
                high = 636 as ::core::ffi::c_int;
            }
            _ => {}
        },
        25 => match *str.offset(9 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            97 => {
                low = 636 as ::core::ffi::c_int;
                high = 637 as ::core::ffi::c_int;
            }
            100 => {
                low = 637 as ::core::ffi::c_int;
                high = 638 as ::core::ffi::c_int;
            }
            115 => {
                low = 638 as ::core::ffi::c_int;
                high = 639 as ::core::ffi::c_int;
            }
            _ => {}
        },
        26 => match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            98 => {
                low = 639 as ::core::ffi::c_int;
                high = 640 as ::core::ffi::c_int;
            }
            115 => {
                low = 640 as ::core::ffi::c_int;
                high = 641 as ::core::ffi::c_int;
            }
            _ => {}
        },
        28 => match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            95 => {
                low = 641 as ::core::ffi::c_int;
                high = 642 as ::core::ffi::c_int;
            }
            98 => {
                low = 642 as ::core::ffi::c_int;
                high = 643 as ::core::ffi::c_int;
            }
            _ => {}
        },
        _ => {}
    }
    let mut i: ::core::ffi::c_int = low;
    while i < high {
        if memcmp(
            str as *const ::core::ffi::c_void,
            (*functions.ptr())[i as usize].name as *const ::core::ffi::c_void,
            len,
        ) == 0
        {
            return i;
        }
        i += 1;
    }
    return -1 as ::core::ffi::c_int;
}
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
unsafe extern "C" fn c2rust_run_static_initializers() {
    functions.set([
        EvalFuncDef {
            name: b"id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_id as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"or\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_or as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"tr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_tr as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"abs\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_abs as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"add\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_add as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"and\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_and as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"col\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_col as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"cos\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                float_op_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                float_func: Some(
                    cos as unsafe extern "C" fn(::core::ffi::c_double) -> ::core::ffi::c_double,
                ),
            },
        },
        EvalFuncDef {
            name: b"exp\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                float_op_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                float_func: Some(
                    exp as unsafe extern "C" fn(::core::ffi::c_double) -> ::core::ffi::c_double,
                ),
            },
        },
        EvalFuncDef {
            name: b"get\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_get as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"has\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_has as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"len\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_len as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"log\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                float_op_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                float_func: Some(
                    log as unsafe extern "C" fn(::core::ffi::c_double) -> ::core::ffi::c_double,
                ),
            },
        },
        EvalFuncDef {
            name: b"map\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_map as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"max\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_max as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"min\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_min as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"pow\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_pow as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"sin\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                float_op_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                float_func: Some(
                    sin as unsafe extern "C" fn(::core::ffi::c_double) -> ::core::ffi::c_double,
                ),
            },
        },
        EvalFuncDef {
            name: b"tan\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                float_op_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                float_func: Some(
                    tan as unsafe extern "C" fn(::core::ffi::c_double) -> ::core::ffi::c_double,
                ),
            },
        },
        EvalFuncDef {
            name: b"xor\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_xor as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"hlID\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_hlID as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"glob\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_glob as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"argc\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_argc as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"fmod\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_fmod as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"rand\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_rand as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"line\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_line as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"mode\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_mode as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"type\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_type as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"cosh\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                float_op_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                float_func: Some(
                    cosh as unsafe extern "C" fn(::core::ffi::c_double) -> ::core::ffi::c_double,
                ),
            },
        },
        EvalFuncDef {
            name: b"sinh\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                float_op_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                float_func: Some(
                    sinh as unsafe extern "C" fn(::core::ffi::c_double) -> ::core::ffi::c_double,
                ),
            },
        },
        EvalFuncDef {
            name: b"tanh\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                float_op_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                float_func: Some(
                    tanh as unsafe extern "C" fn(::core::ffi::c_double) -> ::core::ffi::c_double,
                ),
            },
        },
        EvalFuncDef {
            name: b"call\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_call as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"ceil\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                float_op_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                float_func: Some(
                    ceil as unsafe extern "C" fn(::core::ffi::c_double) -> ::core::ffi::c_double,
                ),
            },
        },
        EvalFuncDef {
            name: b"eval\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_eval as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"trim\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_trim as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"asin\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                float_op_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                float_func: Some(
                    asin as unsafe extern "C" fn(::core::ffi::c_double) -> ::core::ffi::c_double,
                ),
            },
        },
        EvalFuncDef {
            name: b"atan\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                float_op_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                float_func: Some(
                    atan as unsafe extern "C" fn(::core::ffi::c_double) -> ::core::ffi::c_double,
                ),
            },
        },
        EvalFuncDef {
            name: b"join\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_join as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"uniq\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_uniq as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"acos\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                float_op_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                float_func: Some(
                    acos as unsafe extern "C" fn(::core::ffi::c_double) -> ::core::ffi::c_double,
                ),
            },
        },
        EvalFuncDef {
            name: b"keys\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_keys as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"sort\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_sort as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"sqrt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                float_op_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                float_func: Some(
                    sqrt as unsafe extern "C" fn(::core::ffi::c_double) -> ::core::ffi::c_double,
                ),
            },
        },
        EvalFuncDef {
            name: b"wait\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_wait as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"argv\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_argv as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"copy\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_copy as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"match\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_match as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"range\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_range as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"iconv\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_iconv as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"chdir\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_chdir as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"winnr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_winnr as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"mkdir\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_mkdir as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"floor\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                float_op_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                float_func: Some(
                    floor as unsafe extern "C" fn(::core::ffi::c_double) -> ::core::ffi::c_double,
                ),
            },
        },
        EvalFuncDef {
            name: b"slice\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_slice as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"empty\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_empty as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"index\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_index as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"input\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_input as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"count\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_count as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"log10\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                float_op_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                float_func: Some(
                    log10 as unsafe extern "C" fn(::core::ffi::c_double) -> ::core::ffi::c_double,
                ),
            },
        },
        EvalFuncDef {
            name: b"round\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                float_op_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                float_func: Some(
                    round as unsafe extern "C" fn(::core::ffi::c_double) -> ::core::ffi::c_double,
                ),
            },
        },
        EvalFuncDef {
            name: b"split\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_split as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"srand\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_srand as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"trunc\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                float_op_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                float_func: Some(
                    trunc as unsafe extern "C" fn(::core::ffi::c_double) -> ::core::ffi::c_double,
                ),
            },
        },
        EvalFuncDef {
            name: b"isinf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_isinf as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"isnan\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_isnan as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"atan2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_atan2 as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"items\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_items as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"state\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_state as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"bufnr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_bufnr as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"synID\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_synID as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"sha256\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_sha256 as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"append\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 2 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_append as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"bufadd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_bufadd as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"expand\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_expand as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"extend\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_extend as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getcwd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getcwd as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getpid\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_getpid as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"jobpid\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_jobpid as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"browse\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 4 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_browse as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"delete\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_delete as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"escape\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_escape as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"reduce\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_reduce as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"remove\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_remove as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"rename\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_rename as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"printf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: MAX_FUNC_ARGS as ::core::ffi::c_int as uint8_t,
            base_arg: 2 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_printf as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getreg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getreg as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"maparg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_maparg as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"setreg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 2 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_setreg as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"string\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_string as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"search\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 5 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_search as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"pyeval\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_py3eval as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"wincol\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_wincol as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"system\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_system as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"strlen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_strlen as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"ctxpop\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_ctxpop as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"cursor\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_cursor as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"filter\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_filter as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"histnr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_histnr as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"str2nr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_str2nr as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"exists\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_exists as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getpos\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getpos as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"setpos\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 2 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_setpos as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"values\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_values as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"ctxget\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_ctxget as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"ctxset\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_ctxset as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"indent\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_indent as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"insert\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_insert as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"invert\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_invert as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"mapset\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_mapset as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"repeat\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_repeat as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getenv\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getenv as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"setenv\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 2 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_setenv as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"mapnew\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_mapnew as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"argidx\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_argidx as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"stridx\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_stridx as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nr2char\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_nr2char as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"py3eval\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_py3eval as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"char2nr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_char2nr as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"charcol\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_charcol as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"charidx\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_charidx as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"flatten\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_flatten as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"luaeval\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_luaeval as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"readdir\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_readdir as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"jobsend\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_chansend
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"jobstop\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_jobstop as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"jobwait\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_jobwait as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"libcall\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 3 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_libcall as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"rpcstop\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_rpcstop as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"indexof\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_indexof as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"stdpath\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_stdpath as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"execute\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_execute as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"exepath\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_exepath as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"bufload\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_bufload as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"bufname\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_bufname as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"taglist\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_taglist as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"reltime\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_reltime as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"tolower\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_tolower as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"cindent\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_cindent as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"confirm\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_confirm as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"finddir\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_finddir as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"funcref\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_funcref as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"winline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_winline as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"maplist\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_maplist as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"foreach\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_foreach as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"strpart\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_strpart as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"strridx\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_strridx as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"virtcol\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_virtcol as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"has_key\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_has_key as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"histadd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 2 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_histadd as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"histdel\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_histdel as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"histget\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_histget as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"resolve\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_resolve as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"byteidx\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_byteidx as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getchar\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getchar as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getline as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"gettext\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_gettext as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"setline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 2 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_setline as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"toupper\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_toupper as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"environ\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_environ as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"reverse\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_reverse as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"ctxpush\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_ctxpush as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"ctxsize\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_ctxsize as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"pyxeval\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_py3eval as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"utf16idx\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_utf16idx
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"str2list\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_str2list
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"api_info\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_api_info
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"float2nr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_float2nr
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"globpath\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 5 as uint8_t,
            base_arg: 2 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_globpath
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"winbufnr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_winbufnr
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"function\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_function
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"jobclose\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_chanclose
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"mapcheck\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_mapcheck
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"matchadd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 5 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_matchadd
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"matcharg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_matcharg
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"matchend\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_matchend
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"matchstr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_matchstr
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"strchars\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_strchars
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"feedkeys\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_feedkeys
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"findfile\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_findfile
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"foldtext\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_foldtext
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"readblob\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_readblob
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"readfile\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_readfile
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"filecopy\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_filecopy
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getfperm\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_getfperm
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getfsize\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_getfsize
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getftime\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_getftime
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getftype\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_getftype
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"setfperm\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_setfperm
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"strftime\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_strftime
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"tagfiles\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_tagfiles
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"perleval\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_perleval
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"hasmapto\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_hasmapto
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim__id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(1 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_cmd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(2 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_put\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 4 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(3 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"submatch\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_submatch
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"termopen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_termopen
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"changenr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_changenr
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"chansend\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_chansend
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"islocked\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_islocked
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"undofile\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_undofile
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"undotree\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_undotree
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"complete\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 2 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_complete
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"deepcopy\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_deepcopy
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"simplify\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_simplify
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"strptime\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_strptime
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"swapinfo\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_swapinfo
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"swapname\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_swapname
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"tempname\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_tempname
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"jobstart\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_jobstart
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"rpcstart\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_rpcstart
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"synstack\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_synstack
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"hostname\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_hostname
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"keytrans\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_keytrans
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"list2str\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_list2str
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"strtrans\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_strtrans
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"menu_get\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_menu_get
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"bufwinid\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_bufwinid
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"bufwinnr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_bufwinnr
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"strwidth\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_strwidth
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"winwidth\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_winwidth
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"hlexists\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_hlexists
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"rubyeval\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_rubyeval
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"blob2list\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_blob2list
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"byte2line\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_byte2line
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"line2byte\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_line2byte
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"list2blob\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_list2blob
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"synIDattr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_synIDattr
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"diff_hlID\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_diff_hlID
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"menu_info\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_menu_info
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_echo\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(7 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_eval\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(9 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_exec\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(10 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"sign_jump\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_sign_jump
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"gettabvar\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_gettabvar
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"libcallnr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 3 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_libcallnr
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"settabvar\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 3 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_settabvar
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"tabpagenr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_tabpagenr
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"winlayout\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_winlayout
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"chanclose\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_chanclose
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"charclass\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_charclass
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"searchpos\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 5 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_searchpos
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"wordcount\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_wordcount
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"soundfold\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_soundfold
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getregion\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getregion
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"jobresize\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_jobresize
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"screencol\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_screencol
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"screenpos\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_screenpos
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"screenrow\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_screenrow
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"winheight\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_winheight
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"writefile\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_writefile
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getqflist\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getqflist
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"setqflist\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_setqflist
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"str2float\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_str2float
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"win_getid\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_win_getid
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"matchlist\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_matchlist
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"arglistid\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_arglistid
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"buflisted\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_buflisted
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getwinpos\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getwinpos
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getwinvar\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getwinvar
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"setwinvar\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 3 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_setwinvar
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"foldlevel\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_foldlevel
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"localtime\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_localtime
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getcmdpos\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getcmdpos
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"setcmdpos\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_setcmdpos
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"expandcmd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_expandcmd
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"extendnew\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_extendnew
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"bufloaded\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_bufloaded
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"rpcnotify\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: MAX_FUNC_ARGS as ::core::ffi::c_int as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_rpcnotify
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"stdioopen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_stdioopen
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"interrupt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_interrupt
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"browsedir\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_browsedir
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"inputlist\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_inputlist
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"inputsave\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_inputsave
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getbufvar\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getbufvar
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getcurpos\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getcurpos
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"setbufvar\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 3 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_setbufvar
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"bufexists\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_bufexists
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"timer_info\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_timer_info
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"timer_stop\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_timer_stop
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getcharmod\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getcharmod
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getcharpos\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getcharpos
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getcharstr\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getcharstr
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"setcharpos\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 2 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_setcharpos
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"strcharlen\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_strcharlen
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"debugbreak\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_debugbreak
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"gettabinfo\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_gettabinfo
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getloclist\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getloclist
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"setloclist\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: 2 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_setloclist
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getcmdline\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getcmdline
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getcmdtype\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getcmdtype
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"setcmdline\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_setcmdline
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"win_id2win\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_win_id2win
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"flattennew\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_flattennew
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_exec2\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(11 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"pum_getpos\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_pum_getpos
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getbufinfo\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getbufinfo
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getbufline\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getbufline
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"matchfuzzy\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_matchfuzzy
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"setbufline\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 3 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_setbufline
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getreginfo\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getreginfo
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getregtype\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getregtype
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"searchdecl\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_searchdecl
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"searchpair\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 7 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_searchpair
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_input\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(12 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"substitute\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 4 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_substitute
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"foldclosed\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_foldclosed
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"visualmode\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_visualmode
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"reltimestr\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_reltimestr
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"systemlist\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_systemlist
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getwininfo\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getwininfo
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getwinposx\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getwinposx
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getwinposy\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getwinposy
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"lispindent\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_lispindent
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"screenattr\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_screenattr
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"screenchar\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_screenchar
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"win_gotoid\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_win_gotoid
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_paste\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(13 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"sign_place\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 4 as uint8_t,
            max_argc: 5 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_sign_place
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"rpcrequest\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: MAX_FUNC_ARGS as ::core::ffi::c_int as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_rpcrequest
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"foreground\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_foreground
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"serverlist\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_serverlist
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"serverstop\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_serverstop
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"pumvisible\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_pumvisible
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"winrestcmd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_winrestcmd
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"executable\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_executable
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getmatches\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getmatches
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"setmatches\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_setmatches
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"strgetchar\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_strgetchar
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"synIDtrans\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_synIDtrans
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"shiftwidth\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_shiftwidth
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim__stats\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(19 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"timer_pause\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_timer_pause
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"timer_start\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_timer_start
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"matchaddpos\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 5 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_matchaddpos
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"strcharpart\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_strcharpart
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"haslocaldir\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_haslocaldir
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"msgpackdump\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_msgpackdump
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"byteidxcomp\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_byteidxcomp
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"inputdialog\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_inputdialog
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"json_decode\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_json_decode
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"matchdelete\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_matchdelete
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"sign_define\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_sign_define
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"fnameescape\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_fnameescape
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"isdirectory\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_isdirectory
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"json_encode\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_json_encode
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"shellescape\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_shellescape
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"win_gettype\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_win_gettype
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"diff_filler\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_diff_filler
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"strutf16len\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_strutf16len
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"gettagstack\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_gettagstack
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_get_hl\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(14 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"settagstack\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 2 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_settagstack
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"pathshorten\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_pathshorten
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"searchcount\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_searchcount
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"highlightID\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_hlID as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"win_findbuf\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_win_findbuf
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"fnamemodify\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_fnamemodify
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getjumplist\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getjumplist
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getfontname\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getfontname
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_notify\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(18 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"screenchars\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_screenchars
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"fullcommand\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_fullcommand
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"sockconnect\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_sockconnect
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"virtcol2col\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_virtcol2col
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"digraph_get\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_digraph_get
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"digraph_set\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_digraph_set
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"buffer_name\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_bufname as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getmarklist\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getmarklist
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"glob2regpat\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_glob2regpat
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"serverstart\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_serverstart
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"wildtrigger\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_wildtrigger
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"inputsecret\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_inputsecret
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"matchstrpos\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_matchstrpos
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_set_hl\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(15 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"preinserted\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_preinserted
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"winrestview\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_winrestview
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"assert_true\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_assert_true
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getmousepos\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getmousepos
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"winsaveview\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_winsaveview
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"win_execute\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 2 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_win_execute
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim__ns_get\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(29 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim__ns_set\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(30 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim__redraw\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(25 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim__unpack\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(28 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"gettabwinvar\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_gettabwinvar
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"matchbufline\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 4 as uint8_t,
            max_argc: 5 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_matchbufline
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"settabwinvar\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 4 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: 4 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_settabwinvar
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"spellbadword\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_spellbadword
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"msgpackparse\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_msgpackparse
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_command\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(27 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"getcmdprompt\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getcmdprompt
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_del_var\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(26 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"complete_add\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_complete_add
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"filereadable\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_filereadable
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"reg_recorded\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_reg_recorded
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"wildmenumode\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_wildmenumode
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getregionpos\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getregionpos
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_get_var\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(31 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"tabpagewinnr\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_tabpagewinnr
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"eventhandler\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_eventhandler
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"did_filetype\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_did_filetype
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"swapfilelist\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_swapfilelist
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"clearmatches\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_clearmatches
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"reltimefloat\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_reltimefloat
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"screenstring\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_screenstring
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"synconcealed\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_synconcealed
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nextnonblank\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_nextnonblank
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"prevnonblank\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_prevnonblank
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"filewritable\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_filewritable
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"inputrestore\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_inputrestore
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"matchstrlist\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_matchstrlist
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_set_var\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(32 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"spellsuggest\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_spellsuggest
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"assert_beeps\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_assert_beeps
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"assert_equal\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 2 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_assert_equal
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"assert_fails\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 5 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_assert_fails
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"assert_false\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_assert_false
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"assert_match\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 2 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_assert_match
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_ui_send\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(22 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"sign_unplace\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_sign_unplace
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim__id_dict\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(37 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"timer_stopall\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_timer_stopall
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getchangelist\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getchangelist
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getcharsearch\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getcharsearch
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getstacktrace\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getstacktrace
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"setcharsearch\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_setcharsearch
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"win_screenpos\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_win_screenpos
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"appendbufline\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 3 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_appendbufline
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getcmdwintype\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getcmdwintype
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_del_mark\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(41 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"win_id2tabwin\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_win_id2tabwin
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"complete_info\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_complete_info
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"deletebufline\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_deletebufline
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"reg_recording\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_reg_recording
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getbufoneline\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getbufoneline
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"matchfuzzypos\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_matchfuzzypos
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_feedkeys\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(40 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_mark\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(42 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_mode\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(43 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_proc\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(44 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_vvar\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(50 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"searchpairpos\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 7 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_searchpairpos
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"foldclosedend\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_foldclosedend
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getcellwidths\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getcellwidths
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_list_uis\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(35 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"setcellwidths\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_setcellwidths
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getcompletion\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getcompletion
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_open_win\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(36 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"win_splitmove\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_win_splitmove
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"buffer_exists\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_bufexists
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"buffer_number\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_bufnr as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"file_readable\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_filereadable
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getscriptinfo\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getscriptinfo
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_set_vvar\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(51 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_strwidth\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(39 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"assert_nobeep\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_assert_nobeep
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"assert_report\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_assert_report
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"sign_undefine\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_sign_undefine
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_win_hide\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(38 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"reg_executing\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_reg_executing
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim__id_array\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(52 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim__id_float\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(53 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"dictwatcheradd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_dictwatcheradd
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"dictwatcherdel\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_dictwatcherdel
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"last_buffer_nr\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_last_buffer_nr
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getcmdcomplpat\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getcmdcomplpat
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"complete_check\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_complete_check
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"foldtextresult\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_foldtextresult
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_err_write\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(55 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"garbagecollect\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_garbagecollect
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_get_hl_ns\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(58 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"sign_getplaced\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_sign_getplaced
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"tabpagebuflist\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_tabpagebuflist
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_list_bufs\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(59 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_list_wins\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(60 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"isabsolutepath\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_isabsolutepath
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_open_term\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(61 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_out_write\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(62 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_parse_cmd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(63 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"sign_placelist\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_sign_placelist
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_set_hl_ns\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(67 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"assert_inrange\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: 3 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_assert_inrange
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_win_close\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(71 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"windowsversion\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: true_0 != 0,
            func: Some(
                f_windowsversion
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim__buf_stats\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(98 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_attach\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(82 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_delete\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(83 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_create_buf\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(81 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"getcmdcompltype\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getcmdcompltype
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getcmdscreenpos\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getcmdscreenpos
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_del_keymap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(90 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_keymap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(94 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_option\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(95 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"sign_getdefined\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_sign_getdefined
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_list_chans\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(93 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"digraph_getlist\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_digraph_getlist
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"digraph_setlist\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_digraph_setlist
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_set_keymap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 4 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(96 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_set_option\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(97 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"strdisplaywidth\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_strdisplaywidth
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"assert_notequal\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 2 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_assert_notequal
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"assert_notmatch\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 2 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_assert_notmatch
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"prompt_getinput\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_prompt_getinput
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"highlight_exists\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_hlexists
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_del_autocmd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(105 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"sign_unplacelist\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_sign_unplacelist
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"assert_exception\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_assert_exception
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"getcursorcharpos\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getcursorcharpos
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_get_context\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(106 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"setcursorcharpos\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_setcursorcharpos
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_buf_del_var\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(121 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_win_del_var\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(108 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim__screenshot\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(118 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_get_var\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(122 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_win_get_buf\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(109 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_win_get_var\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(110 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"prompt_appendbuf\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 2 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_prompt_appendbuf
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_buf_set_var\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(123 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_win_set_buf\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(111 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_win_set_var\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(112 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"cmdcomplete_info\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_cmdcomplete_info
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_input_mouse\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 6 as uint8_t,
            max_argc: 6 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(113 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"prompt_getprompt\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_prompt_getprompt
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"prompt_setprompt\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_prompt_setprompt
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"assert_equalfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_assert_equalfile
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_err_writeln\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(116 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim__get_lib_dir\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(134 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim__get_runtime\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(143 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_load_context\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(137 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_open_tabpage\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(131 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_autocmds\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(150 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_commands\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(153 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_del_mark\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(126 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_get_mark\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(127 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_get_name\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(128 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_get_text\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 6 as uint8_t,
            max_argc: 6 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(135 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_hl_by_id\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(125 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_is_valid\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(147 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_win_is_valid\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(148 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_set_mark\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 5 as uint8_t,
            max_argc: 5 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(129 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_set_name\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(130 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_set_text\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 6 as uint8_t,
            max_argc: 6 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(136 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"getcompletiontype\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_getcompletiontype
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim__complete_set\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(169 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim__inspect_cell\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(170 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_get_lines\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 4 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(165 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_is_loaded\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(167 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_set_lines\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 5 as uint8_t,
            max_argc: 5 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(172 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_call_function\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(157 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_exec_autocmds\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(158 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_chan_info\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(160 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_color_map\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(161 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_list_tabpages\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(159 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"win_move_separator\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_win_move_separator
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"prompt_setcallback\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_prompt_setcallback
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_win_get_width\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(166 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_win_set_hl_ns\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(173 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_win_set_width\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(174 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_hl_by_name\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(177 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_set_hl_ns_fast\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(178 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_line_count\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(179 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_del_keymap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(180 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_get_keymap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(181 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_set_keymap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 5 as uint8_t,
            max_argc: 5 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(182 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_win_get_height\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(183 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_win_set_height\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(184 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_get_offset\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(185 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_create_augroup\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(186 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_clear_autocmds\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(189 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_win_get_config\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(190 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_win_set_config\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(191 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_get_option\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(192 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_set_option\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(193 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_namespaces\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(194 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_win_get_option\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(195 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_win_set_option\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(196 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"prompt_setinterrupt\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_prompt_setinterrupt
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"win_move_statusline\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: 1 as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_win_move_statusline
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_create_autocmd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(197 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"test_write_list_log\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_test_write_list_log
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_buf_get_number\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(198 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_command_output\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(199 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_win_get_cursor\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(200 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_win_get_number\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(201 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_win_set_cursor\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(202 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_del_extmark\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(203 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_set_extmark\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 5 as uint8_t,
            max_argc: 5 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(204 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_win_get_tabpage\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(205 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_current_buf\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(206 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_set_current_buf\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(207 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_set_current_dir\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(208 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_win_text_height\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(210 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_eval_statusline\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(211 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_option_info\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(215 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_tabpage_del_var\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(218 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_tabpage_get_var\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(219 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_tabpage_set_var\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(220 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_current_win\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(221 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_set_current_win\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(222 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_tabpage_get_win\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(223 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_tabpage_set_win\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(224 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_tabpage_is_valid\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(242 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_del_current_line\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(230 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_current_line\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(233 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_set_current_line\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(240 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_parse_expression\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(237 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_get_commands\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(227 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_get_extmarks\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 5 as uint8_t,
            max_argc: 5 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(228 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_win_get_position\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(243 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_option_info2\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(234 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_option_value\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(235 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_set_option_value\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(241 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_runtime_file\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(236 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim__runtime_inspect\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(226 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_create_namespace\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(229 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_del_user_command\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(231 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_replace_termcodes\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 4 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(253 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_add_highlight\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 6 as uint8_t,
            max_argc: 6 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(250 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_tabpage_list_wins\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(254 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_hl_id_by_name\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(258 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_color_by_name\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(257 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_proc_children\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(252 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_del_augroup_by_id\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(244 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_call_dict_function\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(259 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"test_garbagecollect_now\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                f_test_garbagecollect_now
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData { null: NULL_0 },
        },
        EvalFuncDef {
            name: b"nvim_list_runtime_paths\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(262 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_tabpage_get_number\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(263 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_get_changedtick\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(265 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_current_tabpage\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(266 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_set_current_tabpage\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(267 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_del_augroup_by_name\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 1 as uint8_t,
            max_argc: 1 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(268 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_clear_highlight\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 4 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(269 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_clear_namespace\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 4 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(270 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_create_user_command\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(271 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim__buf_debug_extmarks\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 3 as uint8_t,
            max_argc: 3 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(272 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_get_all_options_info\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(273 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_del_user_command\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 2 as uint8_t,
            max_argc: 2 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(274 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_set_virtual_text\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 5 as uint8_t,
            max_argc: 5 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(275 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_get_extmark_by_id\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 4 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(276 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_select_popupmenu_item\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 4 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(277 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim__invalidate_glyph_cache\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(278 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: b"nvim_buf_create_user_command\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            min_argc: 4 as uint8_t,
            max_argc: 4 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: Some(
                api_wrapper
                    as unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> (),
            ),
            data: EvalFuncData {
                api_handler: (&raw const method_handlers as *const MsgpackRpcRequestHandler)
                    .offset(279 as ::core::ffi::c_int as isize),
            },
        },
        EvalFuncDef {
            name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            min_argc: 0 as uint8_t,
            max_argc: 0 as uint8_t,
            base_arg: BASE_NONE as ::core::ffi::c_int as uint8_t,
            fast: false_0 != 0,
            func: None,
            data: EvalFuncData { null: NULL_0 },
        },
    ]);
}
#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XIB")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [c2rust_run_static_initializers];
