use crate::src::nvim::api::private::helpers::{api_clear_error, cstr_as_string};
use crate::src::nvim::api::ui::{remote_ui_connect, remote_ui_disconnect};
use crate::src::nvim::api::vim::nvim__chan_set_detach;
use crate::src::nvim::api::vimscript::nvim_command;
use crate::src::nvim::arglist::{
    arg_all, check_arg_idx, ex_all, ex_argadd, ex_argdedupe, ex_argdelete, ex_argedit, ex_args,
    ex_argument, ex_last, ex_next, ex_previous, ex_rewind,
};
use crate::src::nvim::ascii::{ascii_isdigit, ascii_isspace, ascii_iswhite};
use crate::src::nvim::autocmd::{
    apply_autocmds, check_nomodeline, do_augroup, do_autocmd, do_doautocmd, ex_doautoall,
    getnextac, has_event, is_aucmd_win, may_trigger_vim_suspend_resume,
};
use crate::src::nvim::buffer::buf_get_changedtick;
use crate::src::nvim::buffer::{
    bt_prompt, bt_quickfix, buf_hide, buf_spname, buflist_findnr, buflist_findpat, buflist_list,
    bufref_valid, do_bufdel, do_modelines, ex_buffer_all, get_highest_fnum, goto_buffer, maketitle,
    no_write_message, otherfile, set_bufref, setaltfname, setfname,
};
use crate::src::nvim::change::deleted_lines_mark;
use crate::src::nvim::channel::channel_proc;
use crate::src::nvim::channel::find_channel;
use crate::src::nvim::channel::{channel_close, channel_job_start};
use crate::src::nvim::charset::{
    backslash_halve, getdigits, getdigits_int, getdigits_int32, skipdigits, skiptowhite_esc,
    skipwhite,
};
use crate::src::nvim::cmdexpand::{ExpandGeneric, ExpandInit, ExpandOne};
use crate::src::nvim::cmdhist::ex_history;
use crate::src::nvim::cursor::{check_cursor, check_cursor_col};
use crate::src::nvim::debugger::{
    dbg_breakpoint, dbg_check_breakpoint, dbg_find_breakpoint, do_debug, ex_breakadd, ex_breakdel,
    ex_breaklist, ex_debug, ex_debuggreedy,
};
use crate::src::nvim::diff::{
    ex_diffgetput, ex_diffoff, ex_diffpatch, ex_diffsplit, ex_diffthis, ex_diffupdate,
};
use crate::src::nvim::digraph::{ex_loadkeymap, listdigraphs, putdigraph};
use crate::src::nvim::drawscreen::{
    clearmode, redraw_all_later, redraw_curbuf_later, redraw_later, redraw_statuslines,
    screen_resize, setcursor_mayforce, showmode, status_redraw_all, status_redraw_curbuf,
    update_screen,
};
use crate::src::nvim::edit::beginline;
use crate::src::nvim::eval::fs::modify_fname;
use crate::src::nvim::eval::typval::tv_list_len;
use crate::src::nvim::eval::typval::{
    callback_free, tv_clear, tv_get_string, tv_list_copy, tv_list_find, tv_list_find_str,
    tv_list_free,
};
use crate::src::nvim::eval::userfunc::{
    current_func_returned, do_return, ex_call, ex_delfunction, ex_function, ex_return,
    func_breakpoint, func_dbg_tick, func_has_abort, func_has_ended, func_level, func_name,
    get_func_line, get_scriptlocal_funcname,
};
use crate::src::nvim::eval::vars::{
    ex_let, ex_lockvar, ex_unlet, get_vim_var_list, get_vim_var_str, set_vim_var_nr,
    set_vim_var_string, v_exception, v_throwpoint, var_redir_start, var_redir_stop,
};
use crate::src::nvim::eval::{
    callback_call, eval_to_string, ex_echo, ex_echohl, ex_execute, get_copyID, set_ref_in_callback,
    skip_expr,
};
use crate::src::nvim::event::libuv::uv_strerror;
use crate::src::nvim::event::r#loop::process_events_until;
use crate::src::nvim::event::proc::{proc_stop, proc_wait};
use crate::src::nvim::ex_cmds::{
    do_ascii, do_bang, do_ecmd, do_move, do_wqall, do_write, ex_align, ex_append, ex_change,
    ex_copy, ex_file, ex_global, ex_oldfiles, ex_sort, ex_substitute, ex_substitute_preview,
    ex_uniq, ex_update, ex_wnext, ex_write, ex_z, global_exe, prepare_tagpreview, print_line,
    print_line_no_prefix, skip_vimgrep_pat,
};
use crate::src::nvim::ex_cmds2::{
    autowrite_all, check_changed, check_changed_any, check_fname, dialog_changed, ex_checktime,
    ex_compiler, ex_drop, ex_listdo, ex_perl, ex_perldo, ex_perlfile, ex_py3file, ex_pydo3,
    ex_python3, ex_ruby, ex_rubydo, ex_rubyfile,
};
use crate::src::nvim::ex_eval::{
    aborting, cleanup_conditionals, discard_current_exception, do_errthrow, do_intthrow, do_throw,
    enter_cleanup, ex_break, ex_catch, ex_continue, ex_else, ex_endfunction, ex_endif, ex_endtry,
    ex_endwhile, ex_eval, ex_finally, ex_if, ex_throw, ex_try, ex_while, has_loop_cmd,
    leave_cleanup, report_make_pending, rewind_conditionals,
};
use crate::src::nvim::ex_getln::{
    allbuf_locked, cmdpreview_get_bufnr, cmdpreview_get_ns, curbuf_locked, get_text_locked_msg,
    getcmdline, getexline, script_get, text_locked, text_locked_msg, text_or_buf_locked,
    ui_ext_cmdline_block_append, ui_ext_cmdline_block_leave,
};
use crate::src::nvim::ex_session::{ex_loadview, ex_mkrc};
use crate::src::nvim::file_search::{
    do_autocmd_dirchanged, file_name_at_cursor, find_file_in_path, vim_chdir, vim_findfile_cleanup,
};
use crate::src::nvim::fileio::{readfile, shorten_fnames};
use crate::src::nvim::fold::{foldCreate, foldManualAllowed, hasFolding, opFoldRange};
use crate::src::nvim::garray::{ga_append_via_ptr, ga_clear, ga_init};
use crate::src::nvim::getchar::{
    beep_flush, ins_typebuf, restore_typeahead, save_typeahead, stuff_empty, stuffReadbuff,
    typebuf_typed, vpeekc,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::help::{ex_exusage, ex_help, ex_helpclose, ex_helptags, ex_viusage};
use crate::src::nvim::highlight_group::{do_highlight, load_colors};
use crate::src::nvim::indent::ex_retab;
use crate::src::nvim::input::ask_yesno;
use crate::src::nvim::log::logmsg;
use crate::src::nvim::lua::executor::{ex_lua, ex_luado, ex_luafile, nlua_exec};
use crate::src::nvim::lua::secure::ex_trust;
use crate::src::nvim::main::c_bytes;
use crate::src::nvim::main::{
    Columns, IObuff, KeyTyped, NameBuff, RedrawingDisabled, Rows, State, VIsual_active,
    arg_had_last, autocmd_bufnr, autocmd_fname, autocmd_fname_full, autocmd_match, caught_stack,
    check_cstack, cmdline_row, cmdpreview, cmdwin_result, cmdwin_type, curbuf, current_exception,
    current_sctx, current_ui, curtab, curwin, debug_break_level, debug_tick, did_emsg,
    did_emsg_syntax, did_endif, did_syncbind, did_throw, do_profiling, e_argreq, e_autocmd_close,
    e_backslash, e_cant_find_file_str_in_path, e_cmdwin, e_command_too_recursive, e_curdir,
    e_empty_buffer, e_endfor, e_endif, e_endtry, e_endwhile, e_failed,
    e_invalid_return_type_from_findfunc, e_invarg, e_invarg2, e_invargval, e_invchan, e_invcmd,
    e_invrange, e_isadir2, e_line_number_out_of_range, e_mkdir, e_modifiable, e_no_errors,
    e_no_more_file_str_found_in_path, e_nobang, e_norange, e_notopen, e_sandbox, e_screenmode,
    e_secure, e_shellempty, e_trailing_arg, e_undobang_cannot_redo_or_move_branch, e_usingsid,
    e_zerocount, emsg_off, emsg_silent, emsg_skip, escape_chars, ex_nesting_level, ex_no_reprint,
    ex_normal_busy, exec_from_reg, exiting, exmode_active, expr_map_lock, finish_op, first_tabpage,
    firstbuf, firstwin, force_abort, force_restart_edit, g_do_tagpreview, getout, global_busy,
    globaldir, got_int, last_chdir_reason, last_cmdline, lastbuf, lastused_tabpage, lastwin,
    lines_left, magic_overruled, main_loop, msg_col, msg_didany, msg_didout, msg_list, msg_row,
    msg_scroll, msg_silent, must_redraw, need_maketitle, need_rethrow, need_wait_return,
    new_last_cmdline, no_hlsearch, no_wait_return, opcount, p_awa, p_cdh, p_confirm, p_cpo, p_ei,
    p_ffu, p_gp, p_hls, p_lz, p_mfd, p_mmd, p_mp, p_pvh, p_rtp, p_sh, p_shada, p_verbose, p_wic,
    p_write, pending_end_reg_executing, pending_exmode_active, postponed_split,
    postponed_split_flags, postponed_split_tab, readonlymode, recoverymode, redir_fd, redir_off,
    redir_reg, redir_vname, redraw_cmdline, reg_executing, repeat_cmdline, restart_edit, sandbox,
    searchcmdlen, secure, stop_insert_mode, suppress_errthrow, textlock, topframe, trylevel,
    typebuf, virtual_op,
};
use crate::src::nvim::mapping::{ex_abbreviate, ex_abclear, ex_map, ex_mapclear, ex_unmap};
use crate::src::nvim::mark::{
    checkpcmark, ex_changes, ex_clearjumps, ex_delmarks, ex_jumps, ex_marks, mark_check, mark_get,
    mark_get_visual, mark_move_to, setmark, setpcmark,
};
use crate::src::nvim::r#match::ex_match;
use crate::src::nvim::mbyte::{
    get_encoding_name, mb_copy_char, utf_head_off, utf8len_tab, utfc_ptr2len,
};
use crate::src::nvim::memline::{
    goto_byte, ml_clearmarked, ml_delete, ml_get, ml_preserve, ml_recover, ml_setmarked,
};
use crate::src::nvim::memory::{
    arena_mem_free, strequal, xcalloc, xfree, xmalloc, xmemcpyz, xmemdupz, xstrdup, xstrlcat,
    xstrlcpy,
};
use crate::src::nvim::menu::{ex_emenu, ex_menu, ex_menutranslate};
use crate::src::nvim::message::{
    emsg, emsg_multiline, ex_messages, iemsg, msg, msg_clr_eos, msg_ext_set_kind, msg_make,
    msg_outtrans, msg_putchar, msg_puts, msg_scroll_flush, msg_start, redirecting, semsg,
    semsg_multiline, smsg, verbose_enter_scroll, verbose_leave_scroll, vim_dialog_yesno,
    wait_return,
};
use crate::src::nvim::mouse::setmouse;
use crate::src::nvim::r#move::{
    check_cursor_moved, cursor_correct, cursor_valid, scrolldown, scrollup, update_curswant,
    update_topline, validate_cursor,
};
use crate::src::nvim::msgpack_rpc::channel::rpc_send_call;
use crate::src::nvim::msgpack_rpc::server::{server_start, server_stop};
use crate::src::nvim::normal::{
    do_check_scrollbind, end_visual_mode, find_ident_under_cursor, get_vtopline, normal_cmd,
    normal_enter, set_cursor_for_append_to_line,
};
use crate::src::nvim::ops::{clear_oparg, do_join, op_delete, op_shift};
use crate::src::nvim::option::{
    ex_set, get_findfunc, get_option_sctx, get_scrolloff_value, magic_isset,
    option_set_callback_func, set_option_direct, set_option_value_give_err,
};
use crate::src::nvim::options::{kOptEventignore, kOptFiletype, kOptFindfunc};
use crate::src::nvim::optionstr::{check_ff_value, free_string_option, get_fileformat_name};
use crate::src::nvim::os::env::{
    expand_env, expand_env_esc, expand_env_save, home_replace, os_getenv_noalloc,
};
use crate::src::nvim::os::fs::{os_dirname, os_fopen, os_isdir, os_mkdir, os_path_exists};
use crate::src::nvim::os::input::{line_breakcheck, os_breakcheck};
use crate::src::nvim::os::lang::ex_language;
use crate::src::nvim::os::libc::{
    __assert_fail, __ctype_b_loc, abort, atoi, atol, fclose, gettext, memmove, memset, ngettext,
    snprintf, strcasecmp, strcat, strcmp, strcpy, strlen, strncmp, strpbrk, strrchr, strstr,
};
use crate::src::nvim::os::shell::{shell_build_argv, shell_free_argv};
use crate::src::nvim::path::{
    FullName_save, path_fnamecmp, path_has_wildcard, path_tail, path_try_shorten_fname, pathcmp,
};
use crate::src::nvim::plines::plines_m_win_fill;
use crate::src::nvim::popupmenu::pum_make_popup;
use crate::src::nvim::profile::{
    ex_profile, func_line_end, func_line_exec, func_line_start, script_line_end, script_line_exec,
    script_line_start,
};
use crate::src::nvim::quickfix::{
    ex_cbelow, ex_cbottom, ex_cbuffer, ex_cc, ex_cclose, ex_cexpr, ex_cfile, ex_cnext, ex_copen,
    ex_cwindow, ex_helpgrep, ex_make, ex_vimgrep, grep_internal, qf_age, qf_get_cur_idx,
    qf_get_cur_valid_idx, qf_get_size, qf_get_valid_size, qf_history, qf_list,
};
use crate::src::nvim::regexp::skip_regexp;
use crate::src::nvim::register::{
    do_execreg, do_put, ex_display, op_yank, set_expr_line, valid_yank_reg, write_reg_contents,
};
use crate::src::nvim::runtime::{
    do_finish, estack_pop, estack_push, estack_sfile, ex_finish, ex_options, ex_packadd,
    ex_packloadall, ex_runtime, ex_scriptencoding, ex_scriptnames, ex_source, exestack,
    getsourceline, source_breakpoint, source_dbg_tick, source_finished, source_level,
    source_runtime,
};
use crate::src::nvim::search::{
    do_search, find_pattern_in_path, restore_last_search_pattern, save_last_search_pattern,
    searchit,
};
use crate::src::nvim::shada::{shada_read_everything, shada_write_file};
use crate::src::nvim::sign::ex_sign;
use crate::src::nvim::spell::{ex_spelldump, ex_spellinfo, ex_spellrepall};
use crate::src::nvim::spellfile::{ex_mkspell, ex_spell};
use crate::src::nvim::state::may_trigger_modechanged;
use crate::src::nvim::statusline::draw_tabline;
use crate::src::nvim::strings::{
    concat_str, del_trailing_spaces, strrep, vim_snprintf, vim_strchr, vim_strsave_escaped,
};
use crate::src::nvim::syntax::{ex_ownsyntax, ex_syntax, ex_syntime};
use crate::src::nvim::tag::{do_tag, do_tags};
pub use crate::src::nvim::types::{
    __gid_t, __off_t, __off64_t, __pthread_internal_list, __pthread_list_t, __pthread_mutex_s,
    __pthread_rwlock_arch_t, __time_t, __uid_t, _IO_FILE, _IO_codecvt, _IO_lock_t, _IO_marker,
    _IO_wide_data, AdditionalData, AlignTextPos, ApiDispatchWrapper, Arena, ArenaMem, Array,
    AutoPat, AutoPatCmd, AutoPatCmd_S, BoolVarValue, Boolean, BufUpdateCallbacks, CMD_index,
    Callback, Callback_data as C2Rust_Unnamed_20, CallbackReader, CallbackType, CdCause, CdScope,
    ChangedtickDictItem, Channel, Channel_stream as C2Rust_Unnamed_41, ChannelCallFrame,
    ChannelPart, ChannelStdinMode, ChannelStreamType, ClientType, CmdParseInfo,
    CmdParseInfo_magic as C2Rust_Unnamed_39, CompleteListItemGetter, DecorExt,
    DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_17, Dict, Direction, Error, ErrorType, EvalFuncData,
    ExtmarkUndoObject, FILE, FileID, Float, FloatAnchor, FloatRelative, GridView, Integer,
    InternalState, Intersection, KeyValuePair, LibuvProc, LineGetter, Loop, LuaRef, LuaRetMode,
    MTKey, MTNode, MTPos, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MapHash, MarkGet, MarkMove, MarkMoveRes, MarkTree, MotionType,
    MsgpackRpcRequestHandler, MultiQueue, Object, ObjectType, OptIndex, OptInt, OptVal, OptValData,
    OptValType, PackerBuffer, PackerBufferFlush, Proc, ProcType, PtyProc, QUEUE, RStream,
    RemapValues, RemoteUI, RpcState, ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t,
    Set_uint32_t, Set_uint64_t, SpecialVarValue, StderrState, StdioPair, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_27, Stream, String_0, Terminal, Timestamp,
    TriState, UIExtension, Unpacker, VarLockStatus, VarType, VimVarIndex, VirtLines, VirtText,
    VirtTextChunk, VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle, Window, alist_T,
    auto_event, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, buffblock, buffblock_T, buffheader_T,
    bufref_T, bufstate_T, chunksize_T, cleanup_T, cleanup_stuff, cmd_addr_T, cmdidx_T, cmdmod_T,
    colnr_T, consumed_blk, cstack_T, cstack_T_cs_pend as C2Rust_Unnamed_34, dict_T, dictvar_S,
    diff_T, diffblock_S, disptick_T, dobuf_action_values, dobuf_start_values, eslist_T,
    eslist_elem, estack_T, estack_T_es_info as C2Rust_Unnamed_55, estack_arg_T, etype_T, evalarg_T,
    event_T, exarg, exarg_T, except_T, except_type_T, expand_T, extmark_undo_vec_t, fcs_chars_T,
    file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_18,
    file_buffer_b_wininfo as C2Rust_Unnamed_26, file_buffer_update_callbacks as C2Rust_Unnamed_15,
    file_buffer_update_channels as C2Rust_Unnamed_16, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_21, funccall_T, garray_T, gid_t, handle_T,
    hash_T, hashitem_T, hashtab_T, ht_stack_S, ht_stack_T, iconv_t, infoptr_T, int16_t, int32_t,
    int64_t, internal_proc_cb, intmax_t, key_extra, key_value_pair, lcs_chars_T, linenr_T, list_T,
    list_stack_S, list_stack_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T,
    llpos_T, loop_0, lpos_T, mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T,
    memline_T, mfdirty_T, msglist, msglist_T, mtnode_inner_s, mtnode_s, multiqueue, object,
    object_data as C2Rust_Unnamed_14, oparg_T, optmagic_T, optset_T, packer_buffer_t, partial_S,
    partial_T, pos_T, pos_save_T, proc, proc_exit_cb, proc_state_cb, proftime_T, pthread_mutex_t,
    pthread_rwlock_t, ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue, reg_extmatch_T, regmatch_T,
    regmmatch_T, regprog, regprog_T, rstream, sattr_T, save_state_T, schar_T, scid_T, sctx_T,
    searchit_arg_T, size_t, ssize_t, stream, stream_close_cb, stream_read_cb,
    stream_uv as C2Rust_Unnamed_29, stream_write_cb, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_19, syn_time_T, synblock_T, synstate_T, tabpage_S,
    tabpage_T, taggy_T, tasave_T, terminal, time_t, typebuf_T, typval_T, typval_vval_union,
    u_entry, u_entry_T, u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_23,
    u_header_uh_alt_prev as C2Rust_Unnamed_22, u_header_uh_next as C2Rust_Unnamed_25,
    u_header_uh_prev as C2Rust_Unnamed_24, ufunc_S, ufunc_T, uid_t, uint8_t, uint16_t, uint32_t,
    uint64_t, undo_object, uv__io_cb, uv__io_s, uv__io_t, uv__queue, uv_alloc_cb, uv_async_cb,
    uv_async_s, uv_async_s_u as C2Rust_Unnamed_4, uv_async_t, uv_buf_t, uv_close_cb, uv_connect_cb,
    uv_connect_s, uv_connect_t, uv_connection_cb, uv_exit_cb, uv_file, uv_gid_t, uv_handle_s,
    uv_handle_s_u as C2Rust_Unnamed_1, uv_handle_t, uv_handle_type, uv_idle_cb, uv_idle_s,
    uv_idle_s_u as C2Rust_Unnamed_11, uv_idle_t, uv_loop_s,
    uv_loop_s_active_reqs as C2Rust_Unnamed_5, uv_loop_s_timer_heap as C2Rust_Unnamed_3, uv_loop_t,
    uv_mutex_t, uv_pipe_s, uv_pipe_s_u as C2Rust_Unnamed_8, uv_pipe_t, uv_process_options_s,
    uv_process_options_t, uv_process_s, uv_process_s_u as C2Rust_Unnamed_12, uv_process_t,
    uv_read_cb, uv_req_type, uv_rwlock_t, uv_shutdown_cb, uv_shutdown_s, uv_shutdown_t,
    uv_signal_cb, uv_signal_s, uv_signal_s_tree_entry as C2Rust_Unnamed_0,
    uv_signal_s_u as C2Rust_Unnamed_2, uv_signal_t, uv_stdio_container_s,
    uv_stdio_container_s_data as C2Rust_Unnamed_13, uv_stdio_container_t, uv_stdio_flags,
    uv_stream_s, uv_stream_s_u as C2Rust_Unnamed_6, uv_stream_t, uv_tcp_s,
    uv_tcp_s_u as C2Rust_Unnamed_7, uv_tcp_t, uv_timer_cb, uv_timer_s,
    uv_timer_s_node as C2Rust_Unnamed_9, uv_timer_s_u as C2Rust_Unnamed_10, uv_timer_t, uv_uid_t,
    varnumber_T, vim_exception, vimconv_T, virt_line, visualinfo_T, win_T, window_S, wininfo_S,
    winopt_T, winsize, wline_T, xfmark_T, xp_prefix_T, yankreg_T,
};
use crate::src::nvim::ui::{
    ui_active, ui_busy_start, ui_busy_stop, ui_call_error_exit, ui_call_restart, ui_call_suspend,
    ui_cursor_shape, ui_flush, ui_has,
};
use crate::src::nvim::undo::{
    bufIsChanged, curbufIsChanged, ex_undojoin, ex_undolist, u_clearline, u_compute_hash,
    u_read_undo, u_redo, u_save, u_savedel, u_undo, u_undo_and_forget, u_write_undo, undo_time,
};
use crate::src::nvim::usercmd::{
    add_win_cmd_modifiers, do_ucmd, ex_comclear, ex_command, ex_delcommand,
    expand_user_command_name, find_ucmd, get_user_command_name,
};
use crate::src::nvim::version::{ex_intro, ex_version};
use crate::src::nvim::window::{
    check_can_set_curbuf_forceit, close_others, do_window, find_tabpage, goto_tabpage,
    only_one_window, tabpage_index, tabpage_move, trigger_tabclosedpre, valid_tabpage, win_close,
    win_close_othertab, win_enter, win_goto, win_new_tabpage, win_setheight_win, win_setwidth_win,
    win_split, win_valid, window_layout_locked,
};
use crate::src::nvim::winfloat::win_float_remove;
use core::ffi::{c_char, c_int, c_long, c_uint, c_ulong, c_ushort, c_void};
unsafe extern "C" {
    static cmdmod: GlobalCell<cmdmod_T>;
    fn vim_regcomp(expr_arg: *const c_char, re_flags: c_int) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
}
pub type C2Rust_Unnamed = c_uint;
pub const _ISdigit: C2Rust_Unnamed = 2048;
pub const _ISupper: C2Rust_Unnamed = 256;
pub const kErrorTypeNone: ErrorType = -1;
pub const kObjectTypeDict: ObjectType = 6;
pub const kObjectTypeString: ObjectType = 4;
pub const kObjectTypeBoolean: ObjectType = 1;
pub const kObjectTypeNil: ObjectType = 0;
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
pub const kCallbackNone: CallbackType = 0;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
pub const VAR_BOOL: VarType = 7;
pub const VAR_LIST: VarType = 4;
pub const VAR_STRING: VarType = 2;
pub const VAR_UNKNOWN: VarType = 0;
pub type C2Rust_Unnamed_30 = c_uint;
pub const MAXLNUM: C2Rust_Unnamed_30 = 2147483647;
pub type C2Rust_Unnamed_31 = c_uint;
pub const MAXCOL: C2Rust_Unnamed_31 = 2147483647;
pub type C2Rust_Unnamed_32 = c_uint;
pub const HLF_T: C2Rust_Unnamed_32 = 23;
pub const HLF_E: C2Rust_Unnamed_32 = 6;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub const kCdScopeGlobal: CdScope = 2;
pub const kCdScopeTabpage: CdScope = 1;
pub const kCdScopeWindow: CdScope = 0;
pub const kCdCauseManual: CdCause = 0;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
pub type C2Rust_Unnamed_33 = c_int;
pub const EXPAND_FILES: C2Rust_Unnamed_33 = 2;
pub const EXPAND_NOTHING: C2Rust_Unnamed_33 = 0;
pub const OPTION_MAGIC_OFF: optmagic_T = 2;
pub const OPTION_MAGIC_ON: optmagic_T = 1;
pub const kOptValTypeString: OptValType = 2;
pub const kMarkAll: MarkGet = 1;
pub const kMarkBufLocal: MarkGet = 0;
pub type C2Rust_Unnamed_35 = c_uint;
pub const CSF_CAUGHT: C2Rust_Unnamed_35 = 4096;
pub const CSF_THROWN: C2Rust_Unnamed_35 = 2048;
pub const CSF_FINALLY: C2Rust_Unnamed_35 = 512;
pub const CSF_TRY: C2Rust_Unnamed_35 = 256;
pub const CSF_FOR: C2Rust_Unnamed_35 = 16;
pub const CSF_WHILE: C2Rust_Unnamed_35 = 8;
pub const CSF_ACTIVE: C2Rust_Unnamed_35 = 2;
pub const CSF_TRUE: C2Rust_Unnamed_35 = 1;
pub type C2Rust_Unnamed_36 = c_uint;
pub const CSTP_THROW: C2Rust_Unnamed_36 = 4;
pub const CSTP_INTERRUPT: C2Rust_Unnamed_36 = 2;
pub const CSTP_ERROR: C2Rust_Unnamed_36 = 1;
pub type C2Rust_Unnamed_37 = c_uint;
pub const CSL_HAD_FINA: C2Rust_Unnamed_37 = 8;
pub const CSL_HAD_CONT: C2Rust_Unnamed_37 = 4;
pub const CSL_HAD_ENDLOOP: C2Rust_Unnamed_37 = 2;
pub const CSL_HAD_LOOP: C2Rust_Unnamed_37 = 1;
pub const kUICmdline: UIExtension = 0;
pub const CMD_SIZE: CMD_index = 557;
pub const CMD_Next: CMD_index = 556;
pub const CMD_at: CMD_index = 554;
pub const CMD_rshift: CMD_index = 553;
pub const CMD_lshift: CMD_index = 551;
pub const CMD_pound: CMD_index = 549;
pub const CMD_bang: CMD_index = 548;
pub const CMD_yank: CMD_index = 546;
pub const CMD_wq: CMD_index = 532;
pub const CMD_wincmd: CMD_index = 527;
pub const CMD_write: CMD_index = 522;
pub const CMD_vsplit: CMD_index = 519;
pub const CMD_vnew: CMD_index = 517;
pub const CMD_vimgrepadd: CMD_index = 511;
pub const CMD_vimgrep: CMD_index = 510;
pub const CMD_view: CMD_index = 509;
pub const CMD_visual: CMD_index = 508;
pub const CMD_vglobal: CMD_index = 504;
pub const CMD_update: CMD_index = 503;
pub const CMD_try: CMD_index = 488;
pub const CMD_terminal: CMD_index = 471;
pub const CMD_tabonly: CMD_index = 463;
pub const CMD_tabnew: CMD_index = 462;
pub const CMD_tabmove: CMD_index = 459;
pub const CMD_tabfind: CMD_index = 457;
pub const CMD_tabedit: CMD_index = 456;
pub const CMD_tabclose: CMD_index = 454;
pub const CMD_sview: CMD_index = 442;
pub const CMD_startreplace: CMD_index = 434;
pub const CMD_startinsert: CMD_index = 432;
pub const CMD_split: CMD_index = 420;
pub const CMD_snomagic: CMD_index = 415;
pub const CMD_smagic: CMD_index = 410;
pub const CMD_sfind: CMD_index = 403;
pub const CMD_substitute: CMD_index = 382;
pub const CMD_rviminfo: CMD_index = 381;
pub const CMD_rshada: CMD_index = 375;
pub const CMD_redir: CMD_index = 363;
pub const CMD_read: CMD_index = 360;
pub const CMD_put: CMD_index = 344;
pub const CMD_print: CMD_index = 318;
pub const CMD_only: CMD_index = 311;
pub const CMD_number: CMD_index = 304;
pub const CMD_new: CMD_index = 291;
pub const CMD_match: CMD_index = 278;
pub const CMD_make: CMD_index = 274;
pub const CMD_move: CMD_index = 272;
pub const CMD_lvimgrepadd: CMD_index = 268;
pub const CMD_lvimgrep: CMD_index = 267;
pub const CMD_lmake: CMD_index = 248;
pub const CMD_ll: CMD_index = 243;
pub const CMD_lgrepadd: CMD_index = 240;
pub const CMD_lgrep: CMD_index = 239;
pub const CMD_list: CMD_index = 210;
pub const CMD_k: CMD_index = 205;
pub const CMD_iput: CMD_index = 197;
pub const CMD_insert: CMD_index = 184;
pub const CMD_hide: CMD_index = 181;
pub const CMD_grepadd: CMD_index = 173;
pub const CMD_grep: CMD_index = 172;
pub const CMD_global: CMD_index = 170;
pub const CMD_foldopen: CMD_index = 166;
pub const CMD_folddoclosed: CMD_index = 165;
pub const CMD_finally: CMD_index = 159;
pub const CMD_file: CMD_index = 154;
pub const CMD_execute: CMD_index = 151;
pub const CMD_enew: CMD_index = 148;
pub const CMD_endwhile: CMD_index = 147;
pub const CMD_endtry: CMD_index = 146;
pub const CMD_endfor: CMD_index = 145;
pub const CMD_endif: CMD_index = 143;
pub const CMD_elseif: CMD_index = 141;
pub const CMD_else: CMD_index = 140;
pub const CMD_echon: CMD_index = 139;
pub const CMD_echomsg: CMD_index = 138;
pub const CMD_echoerr: CMD_index = 136;
pub const CMD_echo: CMD_index = 135;
pub const CMD_earlier: CMD_index = 134;
pub const CMD_edit: CMD_index = 133;
pub const CMD_diffput: CMD_index = 122;
pub const CMD_diffget: CMD_index = 119;
pub const CMD_close: CMD_index = 79;
pub const CMD_checktime: CMD_index = 75;
pub const CMD_cc: CMD_index = 59;
pub const CMD_catch: CMD_index = 54;
pub const CMD_change: CMD_index = 43;
pub const CMD_bwipeout: CMD_index = 42;
pub const CMD_bunload: CMD_index = 41;
pub const CMD_bdelete: CMD_index = 25;
pub const CMD_balt: CMD_index = 24;
pub const CMD_badd: CMD_index = 23;
pub const CMD_autocmd: CMD_index = 17;
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
pub type ex_func_T = Option<unsafe extern "C" fn(*mut exarg_T) -> ()>;
pub type ex_preview_func_T = Option<unsafe extern "C" fn(*mut exarg_T, c_int, handle_T) -> c_int>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CommandDefinition {
    pub cmd_name: *mut c_char,
    pub cmd_func: ex_func_T,
    pub cmd_preview_func: ex_preview_func_T,
    pub cmd_argt: uint32_t,
    pub cmd_addr_type: cmd_addr_T,
}
pub type C2Rust_Unnamed_38 = c_uint;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_38 = 8192;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_38 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_38 = 2048;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_38 = 1024;
pub const CMOD_KEEPMARKS: C2Rust_Unnamed_38 = 512;
pub const CMOD_KEEPALT: C2Rust_Unnamed_38 = 256;
pub const CMOD_CONFIRM: C2Rust_Unnamed_38 = 128;
pub const CMOD_BROWSE: C2Rust_Unnamed_38 = 64;
pub const CMOD_HIDE: C2Rust_Unnamed_38 = 32;
pub const CMOD_NOAUTOCMD: C2Rust_Unnamed_38 = 16;
pub const CMOD_UNSILENT: C2Rust_Unnamed_38 = 8;
pub const CMOD_ERRSILENT: C2Rust_Unnamed_38 = 4;
pub const CMOD_SILENT: C2Rust_Unnamed_38 = 2;
pub const CMOD_SANDBOX: C2Rust_Unnamed_38 = 1;
pub const EVENT_TABNEWENTERED: auto_event = 116;
pub const EVENT_QUITPRE: auto_event = 91;
pub const EVENT_EXITPRE: auto_event = 47;
pub const EVENT_CMDUNDEFINED: auto_event = 29;
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_DEL: dobuf_action_values = 3;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub const DOBUF_MOD: dobuf_start_values = 3;
pub const DOBUF_LAST: dobuf_start_values = 2;
pub const DOBUF_FIRST: dobuf_start_values = 1;
pub const DOBUF_CURRENT: dobuf_start_values = 0;
pub const kChannelPartAll: ChannelPart = 4;
pub const kChannelStdinPipe: ChannelStdinMode = 0;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type C2Rust_Unnamed_42 = c_uint;
pub const PUT_LINE: C2Rust_Unnamed_42 = 8;
pub const PUT_CURSLINE: C2Rust_Unnamed_42 = 4;
pub const PUT_FIXINDENT: C2Rust_Unnamed_42 = 1;
pub type C2Rust_Unnamed_43 = c_uint;
pub const WILD_EXPAND_FREE: C2Rust_Unnamed_43 = 2;
pub type C2Rust_Unnamed_44 = c_uint;
pub const WILD_NOERROR: C2Rust_Unnamed_44 = 2048;
pub const WILD_ICASE: C2Rust_Unnamed_44 = 256;
pub const WILD_ADD_SLASH: C2Rust_Unnamed_44 = 16;
pub const WILD_LIST_NOTFOUND: C2Rust_Unnamed_44 = 1;
pub type C2Rust_Unnamed_45 = c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_45 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_45 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_45 = 35;
pub const UPD_INVERTED: C2Rust_Unnamed_45 = 20;
pub const UPD_VALID: C2Rust_Unnamed_45 = 10;
pub type C2Rust_Unnamed_46 = c_uint;
pub const BL_FIX: C2Rust_Unnamed_46 = 4;
pub const BL_SOL: C2Rust_Unnamed_46 = 2;
pub const BL_WHITE: C2Rust_Unnamed_46 = 1;
pub type C2Rust_Unnamed_47 = c_uint;
pub const VIM_QUESTION: C2Rust_Unnamed_47 = 4;
pub type C2Rust_Unnamed_48 = c_uint;
pub const VIM_YES: C2Rust_Unnamed_48 = 2;
pub const VV_EXITREASON: VimVarIndex = 105;
pub const VV_ARGV: VimVarIndex = 89;
pub const VV_PROGPATH: VimVarIndex = 60;
pub const VV_OLDFILES: VimVarIndex = 58;
pub const VV_HLSEARCH: VimVarIndex = 57;
pub type C2Rust_Unnamed_49 = c_uint;
pub const ECMD_ALTBUF: C2Rust_Unnamed_49 = 32;
pub const ECMD_ADDBUF: C2Rust_Unnamed_49 = 16;
pub const ECMD_FORCEIT: C2Rust_Unnamed_49 = 8;
pub const ECMD_OLDBUF: C2Rust_Unnamed_49 = 4;
pub const ECMD_HIDE: C2Rust_Unnamed_49 = 1;
pub type C2Rust_Unnamed_50 = c_int;
pub const ECMD_ONE: C2Rust_Unnamed_50 = 1;
pub const ECMD_LAST: C2Rust_Unnamed_50 = -1;
pub type C2Rust_Unnamed_51 = c_uint;
pub const CCGD_EXCMD: C2Rust_Unnamed_51 = 16;
pub const CCGD_FORCEIT: C2Rust_Unnamed_51 = 4;
pub const CCGD_MULTWIN: C2Rust_Unnamed_51 = 2;
pub const CCGD_AW: C2Rust_Unnamed_51 = 1;
pub const REMAP_NONE: RemapValues = -1;
pub const REMAP_YES: RemapValues = 0;
pub type C2Rust_Unnamed_52 = c_uint;
pub const DOCMD_KEEPLINE: C2Rust_Unnamed_52 = 32;
pub const DOCMD_EXCRESET: C2Rust_Unnamed_52 = 16;
pub const DOCMD_KEYTYPED: C2Rust_Unnamed_52 = 8;
pub const DOCMD_REPEAT: C2Rust_Unnamed_52 = 4;
pub const DOCMD_NOWAIT: C2Rust_Unnamed_52 = 2;
pub const DOCMD_VERBOSE: C2Rust_Unnamed_52 = 1;
pub type C2Rust_Unnamed_53 = c_uint;
pub const VALID_HEAD: C2Rust_Unnamed_53 = 2;
pub const VALID_PATH: C2Rust_Unnamed_53 = 1;
pub type C2Rust_Unnamed_54 = c_uint;
pub const DIALOG_MSG_SIZE: C2Rust_Unnamed_54 = 1000;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dbg_stuff {
    pub trylevel: c_int,
    pub force_abort: c_int,
    pub caught_stack: *mut except_T,
    pub vv_exception: *mut c_char,
    pub vv_throwpoint: *mut c_char,
    pub did_emsg: c_int,
    pub got_int: c_int,
    pub did_throw: bool,
    pub need_rethrow: c_int,
    pub check_cstack: c_int,
    pub current_exception: *mut except_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct loop_cookie {
    pub lines_gap: *mut garray_T,
    pub current_line: c_int,
    pub repeating: c_int,
    pub lc_getline: LineGetter,
    pub cookie: *mut c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wcmd_T {
    pub line: *mut c_char,
    pub lnum: linenr_T,
}
pub const ETYPE_EXCEPT: etype_T = 5;
pub const OP_LSHIFT: C2Rust_Unnamed_67 = 4;
pub const OP_RSHIFT: C2Rust_Unnamed_67 = 5;
pub const OP_YANK: C2Rust_Unnamed_67 = 2;
pub const OP_DELETE: C2Rust_Unnamed_67 = 1;
pub const WSP_VERT: C2Rust_Unnamed_66 = 2;
pub const FNAME_MESS: C2Rust_Unnamed_56 = 1;
pub const DT_LTAG: C2Rust_Unnamed_65 = 11;
pub const DT_TAG: C2Rust_Unnamed_65 = 1;
pub const DT_LAST: C2Rust_Unnamed_65 = 6;
pub const DT_FIRST: C2Rust_Unnamed_65 = 5;
pub const DT_POP: C2Rust_Unnamed_65 = 2;
pub const DT_NEXT: C2Rust_Unnamed_65 = 3;
pub const DT_PREV: C2Rust_Unnamed_65 = 4;
pub const DT_SELECT: C2Rust_Unnamed_65 = 7;
pub const DT_JUMP: C2Rust_Unnamed_65 = 9;
pub const KE_IGNORE: key_extra = 53;
pub const SEARCH_MSG: C2Rust_Unnamed_63 = 12;
pub const RE_SEARCH: C2Rust_Unnamed_64 = 0;
pub const RE_SUBST: C2Rust_Unnamed_64 = 1;
pub const SEARCH_HIS: C2Rust_Unnamed_63 = 32;
pub const SEARCH_KEEP: C2Rust_Unnamed_63 = 1024;
pub const MODE_INSERT: C2Rust_Unnamed_57 = 16;
pub const OPT_LOCAL: C2Rust_Unnamed_59 = 2;
pub const MODE_CMDLINE: C2Rust_Unnamed_57 = 8;
pub const KE_XF2: key_extra = 58;
pub const KE_XF1: key_extra = 57;
pub const FIND_ANY: C2Rust_Unnamed_61 = 1;
pub const FIND_DEFINE: C2Rust_Unnamed_61 = 2;
pub const ACTION_SPLIT: C2Rust_Unnamed_62 = 3;
pub const ACTION_GOTO: C2Rust_Unnamed_62 = 2;
pub const ACTION_SHOW_ALL: C2Rust_Unnamed_62 = 4;
pub const ACTION_SHOW: C2Rust_Unnamed_62 = 1;
pub const MODE_TERMINAL: C2Rust_Unnamed_57 = 128;
pub const kRetNilBool: LuaRetMode = 1;
pub const DIP_ALL: C2Rust_Unnamed_60 = 1;
pub const CHECK_PATH: C2Rust_Unnamed_61 = 3;
pub const ESTACK_SCRIPT: estack_arg_T = 3;
pub const ESTACK_STACK: estack_arg_T = 2;
pub const ESTACK_SFILE: estack_arg_T = 1;
pub const FNAME_HYP: C2Rust_Unnamed_56 = 4;
pub const FIND_STRING: C2Rust_Unnamed_58 = 2;
pub const FIND_EVAL: C2Rust_Unnamed_58 = 4;
pub const FIND_IDENT: C2Rust_Unnamed_58 = 1;
pub const SPEC_CEXPR: C2Rust_Unnamed_68 = 4;
pub const SPEC_CWORD: C2Rust_Unnamed_68 = 2;
pub const SPEC_CCWORD: C2Rust_Unnamed_68 = 3;
pub const WSP_TOP: C2Rust_Unnamed_66 = 8;
pub const WSP_BELOW: C2Rust_Unnamed_66 = 64;
pub const WSP_ABOVE: C2Rust_Unnamed_66 = 128;
pub const WSP_HOR: C2Rust_Unnamed_66 = 4;
pub const WSP_BOT: C2Rust_Unnamed_66 = 16;
pub const MODE_NORMAL: C2Rust_Unnamed_57 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmdmod {
    pub name: *mut c_char,
    pub minlen: c_int,
    pub has_count: c_int,
}
pub const OPT_GLOBAL: C2Rust_Unnamed_59 = 1;
pub type C2Rust_Unnamed_56 = c_uint;
pub type C2Rust_Unnamed_57 = c_uint;
pub type C2Rust_Unnamed_58 = c_uint;
pub type C2Rust_Unnamed_59 = c_uint;
pub type C2Rust_Unnamed_60 = c_uint;
pub type C2Rust_Unnamed_61 = c_uint;
pub type C2Rust_Unnamed_62 = c_uint;
pub type C2Rust_Unnamed_63 = c_uint;
pub type C2Rust_Unnamed_64 = c_uint;
pub type C2Rust_Unnamed_65 = c_uint;
pub type C2Rust_Unnamed_66 = c_uint;
pub type C2Rust_Unnamed_67 = c_uint;
pub type C2Rust_Unnamed_68 = c_uint;
pub const INT32_MAX: c_int = 2147483647 as c_int;
pub const NULL: *mut c_void = ::core::ptr::null_mut::<c_void>();
pub const NULL_1: *mut c_void = ::core::ptr::null_mut::<c_void>();
pub const EXIT_FAILURE: c_int = 1 as c_int;
pub const DEFAULT_MAXPATHL: c_int = 4096 as c_int;
pub const MAXPATHL: c_int = DEFAULT_MAXPATHL;
pub const BF_DUMMY: c_int = 0x80 as c_int;
pub const ML_EMPTY: c_int = 0x1 as c_int;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as c_int,
    ga_maxlen: 0 as c_int,
    ga_itemsize: 0 as c_int,
    ga_growsize: 1 as c_int,
    ga_data: NULL_1,
};
pub const LOGLVL_INF: c_int = 2 as c_int;
pub const EX_RANGE: c_uint = 0x1 as c_uint;
pub const EX_BANG: c_uint = 0x2 as c_uint;
pub const EX_EXTRA: c_uint = 0x4 as c_uint;
pub const EX_XFILE: c_uint = 0x8 as c_uint;
pub const EX_NOSPC: c_uint = 0x10 as c_uint;
pub const EX_DFLALL: c_uint = 0x20 as c_uint;
pub const EX_WHOLEFOLD: c_uint = 0x40 as c_uint;
pub const EX_NEEDARG: c_uint = 0x80 as c_uint;
pub const EX_TRLBAR: c_uint = 0x100 as c_uint;
pub const EX_REGSTR: c_uint = 0x200 as c_uint;
pub const EX_COUNT: c_uint = 0x400 as c_uint;
pub const EX_NOTRLCOM: c_uint = 0x800 as c_uint;
pub const EX_ZEROR: c_uint = 0x1000 as c_uint;
pub const EX_CTRLV: c_uint = 0x2000 as c_uint;
pub const EX_CMDARG: c_uint = 0x4000 as c_uint;
pub const EX_BUFNAME: c_uint = 0x8000 as c_uint;
pub const EX_BUFUNL: c_uint = 0x10000 as c_uint;
pub const EX_ARGOPT: c_uint = 0x20000 as c_uint;
pub const EX_SBOXOK: c_uint = 0x40000 as c_uint;
pub const EX_CMDWIN: c_uint = 0x80000 as c_uint;
pub const EX_MODIFY: c_uint = 0x100000 as c_uint;
pub const EX_FLAGS: c_uint = 0x200000 as c_uint;
pub const EX_LOCK_OK: c_uint = 0x1000000 as c_uint;
pub const BAD_KEEP: c_int = -1 as c_int;
pub const BAD_DROP: c_int = -2 as c_int;
pub const FORCE_BIN: c_int = 1 as c_int;
pub const FORCE_NOBIN: c_int = 2 as c_int;
pub const EXFLAG_LIST: c_int = 0x1 as c_int;
pub const EXFLAG_NR: c_int = 0x2 as c_int;
pub const EXFLAG_PRINT: c_int = 0x4 as c_int;
pub const NUL: c_int = '\0' as c_int;
pub const CAR: c_int = 13;
pub const Ctrl_B: c_int = 2;
pub const Ctrl_C: c_int = 3 as c_int;
pub const Ctrl_D: c_int = 4;
pub const Ctrl_F: c_int = 6;
pub const Ctrl_G: c_int = 7 as c_int;
pub const Ctrl_H: c_int = 8;
pub const Ctrl_I: c_int = 9;
pub const Ctrl_J: c_int = 10;
pub const Ctrl_K: c_int = 11;
pub const Ctrl_L: c_int = 12;
pub const Ctrl_N: c_int = 14;
pub const Ctrl_O: c_int = 15 as c_int;
pub const Ctrl_P: c_int = 16;
pub const Ctrl_Q: c_int = 17;
pub const Ctrl_R: c_int = 18;
pub const Ctrl_S: c_int = 19;
pub const Ctrl_T: c_int = 20;
pub const Ctrl_V: c_int = 22 as c_int;
pub const Ctrl_W: c_int = 23;
pub const Ctrl_X: c_int = 24;
pub const Ctrl_Z: c_int = 26;
pub const Ctrl_RSB: c_int = 29;
pub const Ctrl_HAT: c_int = 30;
pub const Ctrl__: c_int = 31;
pub const OK: c_int = 1 as c_int;
pub const FAIL: c_int = 0 as c_int;
pub const CPO_ALTREAD: c_int = 'a' as c_int;
pub const CPO_BAR: c_int = 'b' as c_int;
pub const CPO_EXECBUF: c_int = 'e' as c_int;
pub const CPO_NOSYMLINKS: c_int = '~' as c_int;
static e_ambiguous_use_of_user_defined_command: GlobalCell<[c_char; 44]> =
    GlobalCell::new(c_bytes(b"E464: Ambiguous use of user-defined command\0"));
static e_no_call_stack_to_substitute_for_stack: GlobalCell<[c_char; 48]> = GlobalCell::new(
    c_bytes(b"E489: No call stack to substitute for \"<stack>\"\0"),
);
static e_not_an_editor_command: GlobalCell<[c_char; 28]> =
    GlobalCell::new(c_bytes(b"E492: Not an editor command\0"));
static e_no_autocommand_file_name_to_substitute_for_afile: GlobalCell<[c_char; 59]> =
    GlobalCell::new(c_bytes(
        b"E495: No autocommand file name to substitute for \"<afile>\"\0",
    ));
static e_no_autocommand_buffer_number_to_substitute_for_abuf: GlobalCell<[c_char; 62]> =
    GlobalCell::new(c_bytes(
        b"E496: No autocommand buffer number to substitute for \"<abuf>\"\0",
    ));
static e_no_autocommand_match_name_to_substitute_for_amatch: GlobalCell<[c_char; 61]> =
    GlobalCell::new(c_bytes(
        b"E497: No autocommand match name to substitute for \"<amatch>\"\0",
    ));
static e_no_source_file_name_to_substitute_for_sfile: GlobalCell<[c_char; 55]> = GlobalCell::new(
    c_bytes(b"E498: No :source file name to substitute for \"<sfile>\"\0"),
);
static e_no_line_number_to_use_for_slnum: GlobalCell<[c_char; 42]> =
    GlobalCell::new(c_bytes(b"E842: No line number to use for \"<slnum>\"\0"));
static e_no_line_number_to_use_for_sflnum: GlobalCell<[c_char; 43]> =
    GlobalCell::new(c_bytes(b"E961: No line number to use for \"<sflnum>\"\0"));
static e_no_script_file_name_to_substitute_for_script: GlobalCell<[c_char; 56]> = GlobalCell::new(
    c_bytes(b"E1274: No script file name to substitute for \"<script>\"\0"),
);
static quitmore: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
static ex_pressedreturn: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static dollar_command: GlobalCell<[c_char; 2]> = GlobalCell::new(['$' as c_char, 0 as c_char]);
unsafe extern "C" fn save_dbg_stuff(mut dsp: *mut dbg_stuff) {
    (*dsp).trylevel = trylevel.get();
    trylevel.set(0 as c_int);
    (*dsp).force_abort = force_abort.get() as c_int;
    force_abort.set(false_0 != 0);
    (*dsp).caught_stack = caught_stack.get();
    caught_stack.set(::core::ptr::null_mut::<except_T>());
    (*dsp).vv_exception = v_exception(::core::ptr::null_mut::<c_char>());
    (*dsp).vv_throwpoint = v_throwpoint(::core::ptr::null_mut::<c_char>());
    (*dsp).did_emsg = did_emsg.get();
    did_emsg.set(false_0);
    (*dsp).got_int = got_int.get() as c_int;
    got_int.set(false_0 != 0);
    (*dsp).did_throw = did_throw.get();
    did_throw.set(false_0 != 0);
    (*dsp).need_rethrow = need_rethrow.get() as c_int;
    need_rethrow.set(false_0 != 0);
    (*dsp).check_cstack = check_cstack.get() as c_int;
    check_cstack.set(false_0 != 0);
    (*dsp).current_exception = current_exception.get();
    current_exception.set(::core::ptr::null_mut::<except_T>());
}
unsafe extern "C" fn restore_dbg_stuff(mut dsp: *mut dbg_stuff) {
    suppress_errthrow.set(false_0 != 0);
    trylevel.set((*dsp).trylevel);
    force_abort.set((*dsp).force_abort != 0);
    caught_stack.set((*dsp).caught_stack);
    v_exception((*dsp).vv_exception);
    v_throwpoint((*dsp).vv_throwpoint);
    did_emsg.set((*dsp).did_emsg);
    got_int.set((*dsp).got_int != 0);
    did_throw.set((*dsp).did_throw);
    need_rethrow.set((*dsp).need_rethrow != 0);
    check_cstack.set((*dsp).check_cstack != 0);
    current_exception.set((*dsp).current_exception);
}
unsafe extern "C" fn is_other_file(mut fnum: c_int, mut ffname: *mut c_char) -> bool {
    if fnum != 0 as c_int {
        if fnum == (*curbuf.get()).handle {
            return false_0 != 0;
        }
        return true_0 != 0;
    }
    if ffname.is_null() {
        return true_0 != 0;
    }
    if *ffname as c_int == NUL {
        return false_0 != 0;
    }
    if !(*curbuf.get()).file_id_valid
        && !(*curbuf.get()).b_sfname.is_null()
        && *(*curbuf.get()).b_sfname as c_int != NUL
    {
        return path_fnamecmp(ffname, (*curbuf.get()).b_sfname) != 0 as c_int;
    }
    return otherfile(ffname);
}
pub unsafe extern "C" fn do_exmode() {
    exmode_active.set(true_0 != 0);
    State.set(MODE_NORMAL as c_int);
    may_trigger_modechanged();
    if global_busy.get() != 0 {
        return;
    }
    let mut save_msg_scroll: c_int = msg_scroll.get();
    (*RedrawingDisabled.ptr()) += 1;
    (*no_wait_return.ptr()) += 1;
    msg(
        gettext(
            b"Entering Ex mode.  Type \"visual\" to go to Normal mode.\0".as_ptr() as *const c_char,
        ),
        0 as c_int,
    );
    while exmode_active.get() {
        if ex_normal_busy.get() > 0 as c_int && (*typebuf.ptr()).tb_len == 0 as c_int {
            exmode_active.set(false_0 != 0);
            break;
        } else {
            msg_scroll.set(true_0);
            need_wait_return.set(false_0 != 0);
            ex_pressedreturn.set(false_0 != 0);
            ex_no_reprint.set(false_0 != 0);
            let mut changedtick: varnumber_T = buf_get_changedtick(curbuf.get());
            let mut prev_msg_row: c_int = msg_row.get();
            let mut prev_line: linenr_T = (*curwin.get()).w_cursor.lnum;
            cmdline_row.set(msg_row.get());
            do_cmdline(
                ::core::ptr::null_mut::<c_char>(),
                Some(
                    getexline
                        as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                ),
                NULL_1,
                0 as c_int,
            );
            lines_left.set(Rows.get() - 1 as c_int);
            if (prev_line != (*curwin.get()).w_cursor.lnum
                || changedtick != buf_get_changedtick(curbuf.get()))
                && !ex_no_reprint.get()
            {
                if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
                    emsg(gettext(&raw const e_empty_buffer as *const c_char));
                } else {
                    if ex_pressedreturn.get() {
                        msg_scroll_flush();
                        msg_row.set(prev_msg_row);
                        if prev_msg_row == Rows.get() - 1 as c_int {
                            (*msg_row.ptr()) -= 1;
                        }
                    }
                    msg_col.set(0 as c_int);
                    print_line_no_prefix((*curwin.get()).w_cursor.lnum, false_0 != 0, false_0 != 0);
                    msg_clr_eos();
                }
            } else if ex_pressedreturn.get() as c_int != 0 && !ex_no_reprint.get() {
                if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
                    emsg(gettext(&raw const e_empty_buffer as *const c_char));
                } else {
                    emsg(gettext(b"E501: At end-of-file\0".as_ptr() as *const c_char));
                }
            }
        }
    }
    (*RedrawingDisabled.ptr()) -= 1;
    (*no_wait_return.ptr()) -= 1;
    redraw_all_later(UPD_NOT_VALID as c_int);
    update_screen();
    need_wait_return.set(false_0 != 0);
    msg_scroll.set(save_msg_scroll);
}
unsafe extern "C" fn msg_verbose_cmd(mut lnum: linenr_T, mut cmd: *mut c_char) {
    (*no_wait_return.ptr()) += 1;
    verbose_enter_scroll();
    if lnum == 0 as linenr_T {
        smsg(
            0 as c_int,
            gettext(b"Executing: %s\0".as_ptr() as *const c_char),
            cmd,
        );
    } else {
        smsg(
            0 as c_int,
            gettext(b"line %d: %s\0".as_ptr() as *const c_char),
            lnum,
            cmd,
        );
    }
    if msg_silent.get() == 0 as c_int {
        msg_puts(b"\n\0".as_ptr() as *const c_char);
    }
    verbose_leave_scroll();
    (*no_wait_return.ptr()) -= 1;
}
static cmdline_call_depth: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
unsafe extern "C" fn do_cmdline_start() -> c_int {
    '_c2rust_label: {
        if cmdline_call_depth.get() >= 0 as c_int {
        } else {
            __assert_fail(
                b"cmdline_call_depth >= 0\0".as_ptr() as *const c_char,
                b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                364 as c_uint,
                b"int do_cmdline_start(void)\0".as_ptr() as *const c_char,
            );
        }
    };
    if cmdline_call_depth.get() >= 200 as c_int && cmdline_call_depth.get() as OptInt >= p_mfd.get()
    {
        return FAIL;
    }
    (*cmdline_call_depth.ptr()) += 1;
    crate::src::nvim::clipboard::start_batch_changes();
    return OK;
}
unsafe extern "C" fn do_cmdline_end() {
    (*cmdline_call_depth.ptr()) -= 1;
    '_c2rust_label: {
        if cmdline_call_depth.get() >= 0 as c_int {
        } else {
            __assert_fail(
                b"cmdline_call_depth >= 0\0".as_ptr() as *const c_char,
                b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                380 as c_uint,
                b"void do_cmdline_end(void)\0".as_ptr() as *const c_char,
            );
        }
    };
    crate::src::nvim::clipboard::end_batch_changes();
}
pub unsafe extern "C" fn do_cmdline_cmd(mut cmd: *const c_char) -> c_int {
    return do_cmdline(
        cmd as *mut c_char,
        None,
        NULL_1,
        DOCMD_VERBOSE as c_int | DOCMD_NOWAIT as c_int | DOCMD_KEYTYPED as c_int,
    );
}
pub unsafe extern "C" fn do_cmdline(
    mut cmdline: *mut c_char,
    mut fgetline: LineGetter,
    mut cookie: *mut c_void,
    mut flags: c_int,
) -> c_int {
    let mut next_cmdline: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut cmdline_copy: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut used_getline: bool = false_0 != 0;
    static recursive: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
    let mut msg_didout_before_start: bool = false_0 != 0;
    let mut count: c_int = 0 as c_int;
    let mut did_inc: bool = false_0 != 0;
    let mut did_block: bool = false_0 != 0;
    let mut retval: c_int = OK;
    let mut cstack: cstack_T = cstack_T {
        cs_flags: [0; 50],
        cs_pending: [0; 50],
        cs_pend: C2Rust_Unnamed_34 {
            csp_rv: [::core::ptr::null_mut::<c_void>(); 50],
        },
        cs_forinfo: [::core::ptr::null_mut::<c_void>(); 50],
        cs_line: [0; 50],
        cs_idx: -1 as c_int,
        cs_looplevel: 0,
        cs_trylevel: 0,
        cs_emsg_silent_list: ::core::ptr::null_mut::<eslist_T>(),
        cs_lflags: 0,
    };
    let mut lines_ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<c_void>(),
    };
    let mut current_line: c_int = 0 as c_int;
    let mut fname: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut breakpoint: *mut linenr_T = ::core::ptr::null_mut::<linenr_T>();
    let mut dbg_tick: *mut c_int = ::core::ptr::null_mut::<c_int>();
    let mut debug_saved: dbg_stuff = dbg_stuff {
        trylevel: 0,
        force_abort: 0,
        caught_stack: ::core::ptr::null_mut::<except_T>(),
        vv_exception: ::core::ptr::null_mut::<c_char>(),
        vv_throwpoint: ::core::ptr::null_mut::<c_char>(),
        did_emsg: 0,
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        check_cstack: 0,
        current_exception: ::core::ptr::null_mut::<except_T>(),
    };
    let mut private_msg_list: *mut msglist_T = ::core::ptr::null_mut::<msglist_T>();
    let mut cmd_getline: Option<
        unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
    > = None;
    let mut cmd_cookie: *mut c_void = ::core::ptr::null_mut::<c_void>();
    let mut cmd_loop_cookie: loop_cookie = loop_cookie {
        lines_gap: ::core::ptr::null_mut::<garray_T>(),
        current_line: 0,
        repeating: 0,
        lc_getline: None,
        cookie: ::core::ptr::null_mut::<c_void>(),
    };
    let mut saved_msg_list: *mut *mut msglist_T = msg_list.get();
    msg_list.set(&raw mut private_msg_list);
    private_msg_list = ::core::ptr::null_mut::<msglist_T>();
    if do_cmdline_start() == FAIL {
        emsg(gettext(&raw const e_command_too_recursive as *const c_char));
        do_errthrow(NULL_1 as *mut cstack_T, ::core::ptr::null_mut::<c_char>());
        msg_list.set(saved_msg_list);
        return FAIL;
    }
    ga_init(
        &raw mut lines_ga,
        ::core::mem::size_of::<wcmd_T>() as c_int,
        10 as c_int,
    );
    let mut real_cookie: *mut c_void = getline_cookie(fgetline, cookie);
    let mut getline_is_func: bool = getline_equal(
        fgetline,
        cookie,
        Some(get_func_line as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char),
    );
    if getline_is_func as c_int != 0 && ex_nesting_level.get() == func_level(real_cookie) {
        (*ex_nesting_level.ptr()) += 1;
    }
    if getline_is_func {
        fname = func_name(real_cookie);
        breakpoint = func_breakpoint(real_cookie);
        dbg_tick = func_dbg_tick(real_cookie);
    } else if getline_equal(
        fgetline,
        cookie,
        Some(getsourceline as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char),
    ) {
        fname = (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
        .es_name;
        breakpoint = source_breakpoint(real_cookie);
        dbg_tick = source_dbg_tick(real_cookie);
    }
    if recursive.get() == 0 {
        force_abort.set(false_0 != 0);
        suppress_errthrow.set(false_0 != 0);
    }
    if flags & DOCMD_EXCRESET as c_int != 0 {
        save_dbg_stuff(&raw mut debug_saved);
    } else {
        memset(
            &raw mut debug_saved as *mut c_void,
            0 as c_int,
            ::core::mem::size_of::<dbg_stuff>(),
        );
    }
    let mut initial_trylevel: c_int = trylevel.get();
    did_throw.set(false_0 != 0);
    did_emsg.set(false_0);
    if flags & DOCMD_KEYTYPED as c_int == 0
        && !getline_equal(
            fgetline,
            cookie,
            Some(getexline as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char),
        )
    {
        KeyTyped.set(false_0 != 0);
    }
    next_cmdline = cmdline;
    loop {
        getline_is_func = getline_equal(
            fgetline,
            cookie,
            Some(
                get_func_line
                    as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
            ),
        );
        if next_cmdline.is_null()
            && !force_abort.get()
            && cstack.cs_idx < 0 as c_int
            && !(getline_is_func as c_int != 0 && func_has_abort(real_cookie) != 0)
        {
            did_emsg.set(false_0);
        }
        if cstack.cs_looplevel > 0 as c_int && current_line < lines_ga.ga_len {
            let mut ptr_: *mut *mut c_void = &raw mut cmdline_copy as *mut *mut c_void;
            xfree(*ptr_);
            *ptr_ = NULL_1;
            let _ = *ptr_;
            if getline_is_func {
                if do_profiling.get() == PROF_YES {
                    func_line_end(real_cookie);
                }
                if func_has_ended(real_cookie) != 0 {
                    retval = FAIL;
                    break;
                }
            } else if do_profiling.get() == PROF_YES
                && getline_equal(
                    fgetline,
                    cookie,
                    Some(
                        getsourceline
                            as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                    ),
                ) as c_int
                    != 0
            {
                script_line_end();
            }
            if source_finished(fgetline, cookie) {
                retval = FAIL;
                break;
            } else {
                if !breakpoint.is_null() && !dbg_tick.is_null() && *dbg_tick != debug_tick.get() {
                    *breakpoint = dbg_find_breakpoint(
                        getline_equal(
                            fgetline,
                            cookie,
                            Some(
                                getsourceline
                                    as unsafe extern "C" fn(
                                        c_int,
                                        *mut c_void,
                                        c_int,
                                        bool,
                                    )
                                        -> *mut c_char,
                            ),
                        ),
                        fname,
                        (*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
                        .es_lnum,
                    );
                    *dbg_tick = debug_tick.get();
                }
                next_cmdline =
                    (*(lines_ga.ga_data as *mut wcmd_T).offset(current_line as isize)).line;
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
                .es_lnum = (*(lines_ga.ga_data as *mut wcmd_T).offset(current_line as isize)).lnum;
                if !breakpoint.is_null()
                    && *breakpoint != 0 as linenr_T
                    && *breakpoint
                        <= (*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
                        .es_lnum
                {
                    dbg_breakpoint(
                        fname,
                        (*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
                        .es_lnum,
                    );
                    *breakpoint = dbg_find_breakpoint(
                        getline_equal(
                            fgetline,
                            cookie,
                            Some(
                                getsourceline
                                    as unsafe extern "C" fn(
                                        c_int,
                                        *mut c_void,
                                        c_int,
                                        bool,
                                    )
                                        -> *mut c_char,
                            ),
                        ),
                        fname,
                        (*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
                        .es_lnum,
                    );
                    *dbg_tick = debug_tick.get();
                }
                if do_profiling.get() == PROF_YES {
                    if getline_is_func {
                        func_line_start(real_cookie);
                    } else if getline_equal(
                        fgetline,
                        cookie,
                        Some(
                            getsourceline
                                as unsafe extern "C" fn(
                                    c_int,
                                    *mut c_void,
                                    c_int,
                                    bool,
                                )
                                    -> *mut c_char,
                        ),
                    ) {
                        script_line_start();
                    }
                }
            }
        }
        if next_cmdline.is_null() {
            let mut indent: c_int = if cstack.cs_idx < 0 as c_int {
                0 as c_int
            } else {
                (cstack.cs_idx + 1 as c_int) * 2 as c_int
            };
            if count == 1 as c_int
                && getline_equal(
                    fgetline,
                    cookie,
                    Some(
                        getexline
                            as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                    ),
                ) as c_int
                    != 0
            {
                if ui_has(kUICmdline) {
                    ui_ext_cmdline_block_append(0 as size_t, last_cmdline.get());
                    did_block = true_0 != 0;
                }
                msg_didout.set(true_0 != 0);
            }
            if fgetline.is_none() || {
                next_cmdline = fgetline.expect("non-null function pointer")(
                    ':' as c_int,
                    cookie,
                    indent,
                    true_0 != 0,
                );
                next_cmdline.is_null()
            } {
                if KeyTyped.get() as c_int != 0 && flags & DOCMD_REPEAT as c_int == 0 {
                    need_wait_return.set(false_0 != 0);
                }
                retval = FAIL;
                break;
            } else {
                used_getline = true_0 != 0;
                if ui_has(kUICmdline) as c_int != 0
                    && count > 0 as c_int
                    && getline_equal(
                        fgetline,
                        cookie,
                        Some(
                            getexline
                                as unsafe extern "C" fn(
                                    c_int,
                                    *mut c_void,
                                    c_int,
                                    bool,
                                )
                                    -> *mut c_char,
                        ),
                    ) as c_int
                        != 0
                {
                    ui_ext_cmdline_block_append(indent as size_t, next_cmdline);
                }
                if flags & DOCMD_KEEPLINE as c_int != 0 {
                    xfree(repeat_cmdline.get() as *mut c_void);
                    if count == 0 as c_int {
                        repeat_cmdline.set(xstrdup(next_cmdline));
                    } else {
                        repeat_cmdline.set(::core::ptr::null_mut::<c_char>());
                    }
                }
            }
        } else if cmdline_copy.is_null() {
            next_cmdline = xstrdup(next_cmdline);
        }
        cmdline_copy = next_cmdline;
        let mut current_line_before: c_int = 0 as c_int;
        if cstack.cs_looplevel > 0 as c_int || has_loop_cmd(next_cmdline) as c_int != 0 {
            cmd_getline = Some(
                get_loop_line
                    as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
            )
                as Option<unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char>;
            cmd_cookie = &raw mut cmd_loop_cookie as *mut c_void;
            cmd_loop_cookie.lines_gap = &raw mut lines_ga;
            cmd_loop_cookie.current_line = current_line;
            cmd_loop_cookie.lc_getline = fgetline;
            cmd_loop_cookie.cookie = cookie;
            cmd_loop_cookie.repeating = (current_line < lines_ga.ga_len) as c_int;
            if current_line == lines_ga.ga_len {
                store_loop_line(&raw mut lines_ga, next_cmdline);
            }
            current_line_before = current_line;
        } else {
            cmd_getline = fgetline
                as Option<unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char>;
            cmd_cookie = cookie;
        }
        did_endif.set(false_0 != 0);
        let c2rust_fresh0 = count;
        count = count + 1;
        if c2rust_fresh0 == 0 as c_int {
            if flags & DOCMD_NOWAIT as c_int == 0 && recursive.get() == 0 {
                msg_didout_before_start = msg_didout.get();
                msg_didany.set(false_0 != 0);
                msg_start();
                msg_scroll.set(true_0);
                (*no_wait_return.ptr()) += 1;
                (*RedrawingDisabled.ptr()) += 1;
                did_inc = true_0 != 0;
            }
        }
        if p_verbose.get() >= 15 as OptInt
            && !(*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
            .es_name
            .is_null()
            || p_verbose.get() >= 16 as OptInt
        {
            msg_verbose_cmd(
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
                .es_lnum,
                cmdline_copy,
            );
        }
        (*recursive.ptr()) += 1;
        next_cmdline = do_one_cmd(
            &raw mut cmdline_copy,
            flags,
            &raw mut cstack,
            cmd_getline as LineGetter,
            cmd_cookie,
        );
        (*recursive.ptr()) -= 1;
        if cmd_cookie == &raw mut cmd_loop_cookie as *mut c_void {
            current_line = cmd_loop_cookie.current_line;
        }
        if next_cmdline.is_null() {
            let mut ptr__0: *mut *mut c_void = &raw mut cmdline_copy as *mut *mut c_void;
            xfree(*ptr__0);
            *ptr__0 = NULL_1;
            let _ = *ptr__0;
            if getline_equal(
                fgetline,
                cookie,
                Some(
                    getexline
                        as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                ),
            ) as c_int
                != 0
                && !(*new_last_cmdline.ptr()).is_null()
            {
                xfree(last_cmdline.get() as *mut c_void);
                last_cmdline.set(new_last_cmdline.get());
                new_last_cmdline.set(::core::ptr::null_mut::<c_char>());
            }
        } else {
            memmove(
                cmdline_copy as *mut c_void,
                next_cmdline as *const c_void,
                strlen(next_cmdline).wrapping_add(1 as size_t),
            );
            next_cmdline = cmdline_copy;
        }
        if did_emsg.get() != 0
            && !force_abort.get()
            && getline_equal(
                fgetline,
                cookie,
                Some(
                    get_func_line
                        as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                ),
            ) as c_int
                != 0
            && func_has_abort(real_cookie) == 0
        {
            did_emsg.set(false_0);
        }
        if cstack.cs_looplevel > 0 as c_int {
            current_line += 1;
            if cstack.cs_lflags & (CSL_HAD_CONT as c_int | CSL_HAD_ENDLOOP as c_int) != 0 {
                cstack.cs_lflags &= !(CSL_HAD_CONT as c_int | CSL_HAD_ENDLOOP as c_int);
                if did_emsg.get() == 0
                    && !got_int.get()
                    && !did_throw.get()
                    && cstack.cs_idx >= 0 as c_int
                    && cstack.cs_flags[cstack.cs_idx as usize]
                        & (CSF_WHILE as c_int | CSF_FOR as c_int)
                        != 0
                    && cstack.cs_line[cstack.cs_idx as usize] >= 0 as c_int
                    && cstack.cs_flags[cstack.cs_idx as usize] & CSF_ACTIVE as c_int != 0
                {
                    current_line = cstack.cs_line[cstack.cs_idx as usize];
                    cstack.cs_lflags |= CSL_HAD_LOOP as c_int;
                    line_breakcheck();
                    if !breakpoint.is_null() && lines_ga.ga_len > current_line {
                        *breakpoint = dbg_find_breakpoint(
                            getline_equal(
                                fgetline,
                                cookie,
                                Some(
                                    getsourceline
                                        as unsafe extern "C" fn(
                                            c_int,
                                            *mut c_void,
                                            c_int,
                                            bool,
                                        )
                                            -> *mut c_char,
                                ),
                            ),
                            fname,
                            (*(lines_ga.ga_data as *mut wcmd_T).offset(current_line as isize)).lnum
                                - 1 as linenr_T,
                        );
                        *dbg_tick = debug_tick.get();
                    }
                } else if cstack.cs_idx >= 0 as c_int {
                    rewind_conditionals(
                        &raw mut cstack,
                        cstack.cs_idx - 1 as c_int,
                        CSF_WHILE as c_int | CSF_FOR as c_int,
                        &raw mut cstack.cs_looplevel,
                    );
                }
            } else if cstack.cs_lflags & CSL_HAD_LOOP as c_int != 0 {
                cstack.cs_lflags &= !(CSL_HAD_LOOP as c_int);
                cstack.cs_line[cstack.cs_idx as usize] = current_line_before;
            }
        }
        if cstack.cs_looplevel == 0 as c_int {
            if !(lines_ga.ga_len <= 0 as c_int) {
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
                .es_lnum = (*(lines_ga.ga_data as *mut wcmd_T)
                    .offset((lines_ga.ga_len - 1 as c_int) as isize))
                .lnum;
                let mut _gap: *mut garray_T = &raw mut lines_ga;
                if !(*_gap).ga_data.is_null() {
                    let mut i: c_int = 0 as c_int;
                    while i < (*_gap).ga_len {
                        let mut _item: *mut wcmd_T =
                            ((*_gap).ga_data as *mut wcmd_T).offset(i as isize);
                        xfree((*_item).line as *mut c_void);
                        i += 1;
                    }
                }
                ga_clear(_gap);
            }
            current_line = 0 as c_int;
        }
        if cstack.cs_lflags & CSL_HAD_FINA as c_int != 0 {
            cstack.cs_lflags &= !(CSL_HAD_FINA as c_int);
            report_make_pending(
                cstack.cs_pending[cstack.cs_idx as usize] as c_int
                    & (CSTP_ERROR as c_int | CSTP_INTERRUPT as c_int | CSTP_THROW as c_int),
                (if did_throw.get() as c_int != 0 {
                    current_exception.get()
                } else {
                    ::core::ptr::null_mut::<except_T>()
                }) as *mut c_void,
            );
            did_throw.set(false_0 != 0);
            got_int.set(did_throw.get());
            did_emsg.set(got_int.get() as c_int);
            cstack.cs_flags[cstack.cs_idx as usize] |= CSF_ACTIVE as c_int | CSF_FINALLY as c_int;
        }
        trylevel.set(initial_trylevel + cstack.cs_trylevel);
        if trylevel.get() == 0 as c_int && did_emsg.get() == 0 && !got_int.get() && !did_throw.get()
        {
            force_abort.set(false_0 != 0);
        }
        do_intthrow(&raw mut cstack);
        if !(!((got_int.get() as c_int != 0
            || did_emsg.get() != 0 && force_abort.get() as c_int != 0
            || did_throw.get() as c_int != 0)
            && cstack.cs_trylevel == 0 as c_int)
            && !(did_emsg.get() != 0
                && (cstack.cs_trylevel == 0 as c_int || did_emsg_syntax.get() as c_int != 0)
                && used_getline as c_int != 0
                && getline_equal(
                    fgetline,
                    cookie,
                    Some(
                        getexline
                            as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                    ),
                ) as c_int
                    != 0)
            && (!next_cmdline.is_null()
                || cstack.cs_idx >= 0 as c_int
                || flags & DOCMD_REPEAT as c_int != 0))
        {
            break;
        }
    }
    xfree(cmdline_copy as *mut c_void);
    did_emsg_syntax.set(false_0 != 0);
    let mut _gap_0: *mut garray_T = &raw mut lines_ga;
    if !(*_gap_0).ga_data.is_null() {
        let mut i_0: c_int = 0 as c_int;
        while i_0 < (*_gap_0).ga_len {
            let mut _item_0: *mut wcmd_T = ((*_gap_0).ga_data as *mut wcmd_T).offset(i_0 as isize);
            xfree((*_item_0).line as *mut c_void);
            i_0 += 1;
        }
    }
    ga_clear(_gap_0);
    if cstack.cs_idx >= 0 as c_int {
        if !got_int.get()
            && !did_throw.get()
            && !aborting()
            && (getline_equal(
                fgetline,
                cookie,
                Some(
                    getsourceline
                        as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                ),
            ) as c_int
                != 0
                && !source_finished(fgetline, cookie)
                || getline_equal(
                    fgetline,
                    cookie,
                    Some(
                        get_func_line
                            as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                    ),
                ) as c_int
                    != 0
                    && func_has_ended(real_cookie) == 0)
        {
            if cstack.cs_flags[cstack.cs_idx as usize] & CSF_TRY as c_int != 0 {
                emsg(gettext(&raw const e_endtry as *const c_char));
            } else if cstack.cs_flags[cstack.cs_idx as usize] & CSF_WHILE as c_int != 0 {
                emsg(gettext(&raw const e_endwhile as *const c_char));
            } else if cstack.cs_flags[cstack.cs_idx as usize] & CSF_FOR as c_int != 0 {
                emsg(gettext(&raw const e_endfor as *const c_char));
            } else {
                emsg(gettext(&raw const e_endif as *const c_char));
            }
        }
        loop {
            let mut idx: c_int = cleanup_conditionals(&raw mut cstack, 0 as c_int, true_0);
            if idx >= 0 as c_int {
                idx -= 1;
            }
            rewind_conditionals(
                &raw mut cstack,
                idx,
                CSF_WHILE as c_int | CSF_FOR as c_int,
                &raw mut cstack.cs_looplevel,
            );
            if cstack.cs_idx < 0 as c_int {
                break;
            }
        }
        trylevel.set(initial_trylevel);
    }
    do_errthrow(
        &raw mut cstack,
        (if getline_equal(
            fgetline,
            cookie,
            Some(
                get_func_line
                    as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
            ),
        ) as c_int
            != 0
        {
            b"endfunction\0".as_ptr() as *const c_char
        } else {
            ::core::ptr::null::<c_char>()
        }) as *mut c_char,
    );
    if trylevel.get() == 0 as c_int {
        if did_throw.get() {
            handle_did_throw();
        } else if got_int.get() as c_int != 0
            || did_emsg.get() != 0 && force_abort.get() as c_int != 0
        {
            suppress_errthrow.set(true_0 != 0);
        }
    }
    if did_throw.get() {
        need_rethrow.set(true_0 != 0);
    }
    if getline_equal(
        fgetline,
        cookie,
        Some(getsourceline as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char),
    ) as c_int
        != 0
        && ex_nesting_level.get() > source_level(real_cookie)
        || getline_equal(
            fgetline,
            cookie,
            Some(
                get_func_line
                    as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
            ),
        ) as c_int
            != 0
            && ex_nesting_level.get() > func_level(real_cookie) + 1 as c_int
    {
        if !did_throw.get() {
            check_cstack.set(true_0 != 0);
        }
    } else {
        if getline_equal(
            fgetline,
            cookie,
            Some(
                get_func_line
                    as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
            ),
        ) {
            (*ex_nesting_level.ptr()) -= 1;
        }
        if (getline_equal(
            fgetline,
            cookie,
            Some(
                getsourceline
                    as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
            ),
        ) as c_int
            != 0
            || getline_equal(
                fgetline,
                cookie,
                Some(
                    get_func_line
                        as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                ),
            ) as c_int
                != 0)
            && ex_nesting_level.get() + 1 as c_int <= debug_break_level.get()
        {
            do_debug(
                if getline_equal(
                    fgetline,
                    cookie,
                    Some(
                        getsourceline
                            as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                    ),
                ) as c_int
                    != 0
                {
                    gettext(b"End of sourced file\0".as_ptr() as *const c_char)
                } else {
                    gettext(b"End of function\0".as_ptr() as *const c_char)
                },
            );
        }
    }
    if flags & DOCMD_EXCRESET as c_int != 0 {
        restore_dbg_stuff(&raw mut debug_saved);
    }
    msg_list.set(saved_msg_list);
    if !cstack.cs_emsg_silent_list.is_null() {
        let mut temp: *mut eslist_T = ::core::ptr::null_mut::<eslist_T>();
        let mut elem: *mut eslist_T = cstack.cs_emsg_silent_list;
        while !elem.is_null() {
            temp = (*elem).next;
            xfree(elem as *mut c_void);
            elem = temp;
        }
    }
    if did_inc {
        (*RedrawingDisabled.ptr()) -= 1;
        (*no_wait_return.ptr()) -= 1;
        msg_scroll.set(false_0);
        if retval == FAIL
            || did_endif.get() as c_int != 0 && KeyTyped.get() as c_int != 0 && did_emsg.get() == 0
        {
            need_wait_return.set(false_0 != 0);
            msg_didany.set(false_0 != 0);
        } else if need_wait_return.get() {
            msg_didout.set(msg_didout.get() as c_int | msg_didout_before_start as c_int != 0);
            wait_return(false_0);
        }
    }
    if did_block {
        ui_ext_cmdline_block_leave();
    }
    did_endif.set(false_0 != 0);
    do_cmdline_end();
    return retval;
}
pub unsafe extern "C" fn handle_did_throw() {
    '_c2rust_label: {
        if !(*current_exception.ptr()).is_null() {
        } else {
            __assert_fail(
                b"current_exception != NULL\0".as_ptr() as *const c_char,
                b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                974 as c_uint,
                b"void handle_did_throw(void)\0".as_ptr() as *const c_char,
            );
        }
    };
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut messages: *mut msglist_T = ::core::ptr::null_mut::<msglist_T>();
    match (*current_exception.get()).type_0 as c_uint {
        0 => {
            vim_snprintf(
                IObuff.ptr() as *mut c_char,
                IOSIZE as size_t,
                gettext(b"E605: Exception not caught: %s\0".as_ptr() as *const c_char),
                (*current_exception.get()).value,
            );
            p = xstrdup(IObuff.ptr() as *mut c_char);
        }
        1 => {
            messages = (*current_exception.get()).messages;
            (*current_exception.get()).messages = ::core::ptr::null_mut::<msglist_T>();
        }
        2 | _ => {}
    }
    estack_push(
        ETYPE_EXCEPT,
        (*current_exception.get()).throw_name,
        (*current_exception.get()).throw_lnum,
    );
    (*current_exception.get()).throw_name = ::core::ptr::null_mut::<c_char>();
    discard_current_exception();
    if emsg_silent.get() == 0 as c_int {
        suppress_errthrow.set(true_0 != 0);
        force_abort.set(true_0 != 0);
    }
    if !messages.is_null() {
        loop {
            let mut next: *mut msglist_T = (*messages).next;
            emsg_multiline(
                (*messages).msg,
                b"emsg\0".as_ptr() as *const c_char,
                HLF_E as c_int,
                (*messages).multiline,
            );
            xfree((*messages).msg as *mut c_void);
            xfree((*messages).sfile as *mut c_void);
            xfree(messages as *mut c_void);
            messages = next;
            if messages.is_null() {
                break;
            }
        }
    } else if !p.is_null() {
        emsg(p);
        xfree(p as *mut c_void);
    }
    xfree(
        (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
        .es_name as *mut c_void,
    );
    estack_pop();
}
unsafe extern "C" fn get_loop_line(
    mut c: c_int,
    mut cookie: *mut c_void,
    mut indent: c_int,
    mut do_concat: bool,
) -> *mut c_char {
    let mut cp: *mut loop_cookie = cookie as *mut loop_cookie;
    if (*cp).current_line + 1 as c_int >= (*(*cp).lines_gap).ga_len {
        if (*cp).repeating != 0 {
            return ::core::ptr::null_mut::<c_char>();
        }
        let mut line: *mut c_char = ::core::ptr::null_mut::<c_char>();
        if (*cp).lc_getline.is_none() {
            line = getcmdline(c, 0 as c_int, indent, do_concat);
        } else {
            line = (*cp).lc_getline.expect("non-null function pointer")(
                c,
                (*cp).cookie,
                indent,
                do_concat,
            );
        }
        if !line.is_null() {
            store_loop_line((*cp).lines_gap, line);
            (*cp).current_line += 1;
        }
        return line;
    }
    KeyTyped.set(false_0 != 0);
    (*cp).current_line += 1;
    let mut wp: *mut wcmd_T =
        ((*(*cp).lines_gap).ga_data as *mut wcmd_T).offset((*cp).current_line as isize);
    (*((*exestack.ptr()).ga_data as *mut estack_T)
        .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
    .es_lnum = (*wp).lnum;
    return xstrdup((*wp).line);
}
unsafe extern "C" fn store_loop_line(mut gap: *mut garray_T, mut line: *mut c_char) {
    let mut p: *mut wcmd_T =
        ga_append_via_ptr(gap, ::core::mem::size_of::<wcmd_T>()) as *mut wcmd_T;
    (*p).line = xstrdup(line);
    (*p).lnum = (*((*exestack.ptr()).ga_data as *mut estack_T)
        .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
    .es_lnum;
}
// Ex-command callbacks and line getters are identified by address, as the C
// code did; the helpers spell the address comparison out so the intent
// survives the `unpredictable_function_pointer_comparisons` lint.
fn ex_func_is(
    func: Option<unsafe extern "C" fn(*mut exarg_T)>,
    f: unsafe extern "C" fn(*mut exarg_T),
) -> bool {
    func.is_some_and(|g| ::core::ptr::fn_addr_eq(g, f))
}

fn line_getter_eq(a: LineGetter, b: LineGetter) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => ::core::ptr::fn_addr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}
pub unsafe extern "C" fn getline_equal(
    mut fgetline: LineGetter,
    mut cookie: *mut c_void,
    mut func: LineGetter,
) -> bool {
    let mut gp: LineGetter = fgetline;
    let mut cp: *mut loop_cookie = cookie as *mut loop_cookie;
    while line_getter_eq(gp, Some(get_loop_line)) {
        gp = (*cp).lc_getline;
        cp = (*cp).cookie as *mut loop_cookie;
    }
    return line_getter_eq(gp, func);
}
pub unsafe extern "C" fn getline_cookie(
    mut fgetline: LineGetter,
    mut cookie: *mut c_void,
) -> *mut c_void {
    let mut gp: LineGetter = fgetline;
    let mut cp: *mut loop_cookie = cookie as *mut loop_cookie;
    while line_getter_eq(gp, Some(get_loop_line)) {
        gp = (*cp).lc_getline;
        cp = (*cp).cookie as *mut loop_cookie;
    }
    return cp as *mut c_void;
}
unsafe extern "C" fn compute_buffer_local_count(
    mut addr_type: cmd_addr_T,
    mut lnum: linenr_T,
    mut offset: c_int,
) -> c_int {
    let mut count: c_int = offset;
    let mut buf: *mut buf_T = firstbuf.get();
    while !(*buf).b_next.is_null() && ((*buf).handle as linenr_T) < lnum {
        buf = (*buf).b_next;
    }
    while count != 0 as c_int {
        count += if count < 0 as c_int {
            1 as c_int
        } else {
            -1 as c_int
        };
        let mut nextbuf: *mut buf_T = if offset < 0 as c_int {
            (*buf).b_prev
        } else {
            (*buf).b_next
        };
        if nextbuf.is_null() {
            break;
        }
        buf = nextbuf;
        if addr_type as c_uint == ADDR_LOADED_BUFFERS as c_int as c_uint {
            while (*buf).b_ml.ml_mfp.is_null() {
                nextbuf = if offset < 0 as c_int {
                    (*buf).b_prev
                } else {
                    (*buf).b_next
                };
                if nextbuf.is_null() {
                    break;
                }
                buf = nextbuf;
            }
        }
    }
    if addr_type as c_uint == ADDR_LOADED_BUFFERS as c_int as c_uint {
        while (*buf).b_ml.ml_mfp.is_null() {
            let mut nextbuf_0: *mut buf_T = if offset >= 0 as c_int {
                (*buf).b_prev
            } else {
                (*buf).b_next
            };
            if nextbuf_0.is_null() {
                break;
            }
            buf = nextbuf_0;
        }
    }
    return (*buf).handle as c_int;
}
unsafe extern "C" fn current_win_nr(mut win: *const win_T) -> c_int {
    let mut nr: c_int = 0 as c_int;
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        nr += 1;
        if wp == win as *mut win_T {
            break;
        }
        wp = (*wp).w_next;
    }
    return nr;
}
unsafe extern "C" fn current_tab_nr(mut tab: *mut tabpage_T) -> c_int {
    let mut nr: c_int = 0 as c_int;
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        nr += 1;
        if tp == tab {
            break;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    return nr;
}
unsafe extern "C" fn get_wincmd_addr_type(mut arg: *const c_char, mut eap: *mut exarg_T) {
    match *arg as c_int {
        83 | Ctrl_S | 115 | Ctrl_N | 110 | 106 | Ctrl_J | 107 | Ctrl_K | 84 | Ctrl_R | 114 | 82
        | 75 | 74 | 43 | 45 | Ctrl__ | 95 | 124 | 93 | Ctrl_RSB | 103 | Ctrl_G | Ctrl_V | 118
        | 104 | Ctrl_H | 108 | Ctrl_L | 72 | 76 | 62 | 60 | 125 | 102 | 70 | Ctrl_F | 105
        | Ctrl_I | 100 | Ctrl_D => {
            (*eap).addr_type = ADDR_OTHER;
        }
        Ctrl_HAT | 94 => {
            (*eap).addr_type = ADDR_BUFFERS;
        }
        Ctrl_Q | 113 | Ctrl_C | 99 | Ctrl_O | 111 | Ctrl_W | 119 | 87 | 120 | Ctrl_X => {
            (*eap).addr_type = ADDR_WINDOWS;
        }
        Ctrl_Z | 122 | 80 | 116 | Ctrl_T | 98 | Ctrl_B | 112 | Ctrl_P | 61 | CAR => {
            (*eap).addr_type = ADDR_NONE;
        }
        _ => {}
    };
}
unsafe extern "C" fn skip_colon_white(
    mut p: *const c_char,
    mut skipleadingwhite: bool,
) -> *mut c_char {
    if skipleadingwhite {
        p = skipwhite(p);
    }
    while *p as c_int == ':' as c_int {
        p = skipwhite(p.offset(1 as c_int as isize));
    }
    return p as *mut c_char;
}
pub unsafe extern "C" fn set_cmd_addr_type(mut eap: *mut exarg_T, mut p: *mut c_char) {
    if ((*eap).cmdidx as c_int) < 0 as c_int {
        return;
    }
    if (*eap).cmdidx as c_int != CMD_SIZE as c_int {
        (*eap).addr_type = (*cmdnames.ptr())[(*eap).cmdidx as c_int as usize].cmd_addr_type;
    } else {
        (*eap).addr_type = ADDR_LINES;
    }
    if (*eap).cmdidx as c_int == CMD_wincmd as c_int && !p.is_null() {
        get_wincmd_addr_type(skipwhite(p), eap);
    }
    if ((*eap).cmdidx as c_int == CMD_cc as c_int || (*eap).cmdidx as c_int == CMD_ll as c_int)
        && bt_quickfix(curbuf.get()) as c_int != 0
    {
        (*eap).addr_type = ADDR_OTHER;
    }
}
pub unsafe extern "C" fn get_cmd_default_range(mut eap: *mut exarg_T) -> linenr_T {
    match (*eap).addr_type as c_uint {
        0 | 10 => {
            return if (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count {
                (*curwin.get()).w_cursor.lnum
            } else {
                (*curbuf.get()).b_ml.ml_line_count
            };
        }
        1 => return current_win_nr(curwin.get()) as linenr_T,
        2 => {
            return if ((*curwin.get()).w_arg_idx + 1 as c_int)
                < (*(*curwin.get()).w_alist).al_ga.ga_len
            {
                (*curwin.get()).w_arg_idx as linenr_T + 1 as linenr_T
            } else {
                (*(*curwin.get()).w_alist).al_ga.ga_len as linenr_T
            };
        }
        3 | 4 => return (*curbuf.get()).handle as linenr_T,
        5 => return current_tab_nr(curtab.get()) as linenr_T,
        6 | 9 => return 1 as linenr_T,
        8 => return qf_get_cur_idx(eap) as linenr_T,
        7 => return qf_get_cur_valid_idx(eap) as linenr_T,
        _ => return 0 as linenr_T,
    };
}
pub unsafe extern "C" fn set_cmd_dflall_range(mut eap: *mut exarg_T) {
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    (*eap).line1 = 1 as c_int as linenr_T;
    match (*eap).addr_type as c_uint {
        0 | 10 => {
            (*eap).line2 = (*curbuf.get()).b_ml.ml_line_count;
        }
        3 => {
            buf = firstbuf.get();
            while !(*buf).b_next.is_null() && (*buf).b_ml.ml_mfp.is_null() {
                buf = (*buf).b_next;
            }
            (*eap).line1 = (*buf).handle as linenr_T;
            buf = lastbuf.get();
            while !(*buf).b_prev.is_null() && (*buf).b_ml.ml_mfp.is_null() {
                buf = (*buf).b_prev;
            }
            (*eap).line2 = (*buf).handle as linenr_T;
        }
        4 => {
            (*eap).line1 = (*firstbuf.get()).handle as linenr_T;
            (*eap).line2 = (*lastbuf.get()).handle as linenr_T;
        }
        1 => {
            (*eap).line2 = current_win_nr(::core::ptr::null::<win_T>()) as linenr_T;
        }
        5 => {
            (*eap).line2 = current_tab_nr(::core::ptr::null_mut::<tabpage_T>()) as linenr_T;
        }
        6 => {
            (*eap).line2 = 1 as c_int as linenr_T;
        }
        2 => {
            if (*(*curwin.get()).w_alist).al_ga.ga_len == 0 as c_int {
                (*eap).line2 = 0 as c_int as linenr_T;
                (*eap).line1 = (*eap).line2;
            } else {
                (*eap).line2 = (*(*curwin.get()).w_alist).al_ga.ga_len as linenr_T;
            }
        }
        7 => {
            (*eap).line2 = qf_get_valid_size(eap) as linenr_T;
            if (*eap).line2 == 0 as linenr_T {
                (*eap).line2 = 1 as c_int as linenr_T;
            }
        }
        11 | 9 | 8 => {
            iemsg(gettext(
                b"INTERNAL: Cannot use EX_DFLALL with ADDR_NONE, ADDR_UNSIGNED or ADDR_QUICKFIX\0"
                    .as_ptr() as *const c_char,
            ));
        }
        _ => {}
    };
}
unsafe extern "C" fn parse_register(mut eap: *mut exarg_T) {
    if (*eap).argt & EX_REGSTR as uint32_t != 0
        && *(*eap).arg as c_int != NUL
        && (!(((*eap).cmdidx as c_int) < 0 as c_int) || *(*eap).arg as c_int != '=' as c_int)
        && !((*eap).argt & EX_COUNT as uint32_t != 0
            && ascii_isdigit(*(*eap).arg as c_int) as c_int != 0)
    {
        if valid_yank_reg(
            *(*eap).arg as c_int,
            !(((*eap).cmdidx as c_int) < 0 as c_int)
                && (*eap).cmdidx as c_int != CMD_put as c_int
                && (*eap).cmdidx as c_int != CMD_iput as c_int,
        ) {
            let c2rust_fresh25 = (*eap).arg;
            (*eap).arg = (*eap).arg.offset(1);
            (*eap).regname = *c2rust_fresh25 as uint8_t as c_int;
            if *(*eap).arg.offset(-1 as c_int as isize) as c_int == '=' as c_int
                && *(*eap).arg.offset(0 as c_int as isize) as c_int != NUL
            {
                if (*eap).skip == 0 {
                    set_expr_line(xstrdup((*eap).arg));
                }
                (*eap).arg = (*eap).arg.offset(strlen((*eap).arg) as isize);
            }
            (*eap).arg = skipwhite((*eap).arg);
        }
    }
}
pub unsafe extern "C" fn set_cmd_count(
    mut eap: *mut exarg_T,
    mut count: linenr_T,
    mut validate: bool,
) {
    if (*eap).addr_type as c_uint != ADDR_LINES as c_int as c_uint {
        (*eap).line2 = count;
        if (*eap).addr_count == 0 as c_int {
            (*eap).addr_count = 1 as c_int;
        }
    } else {
        (*eap).line1 = (*eap).line2;
        if (*eap).line2 >= INT32_MAX as linenr_T - (count - 1 as linenr_T) {
            (*eap).line2 = INT32_MAX as linenr_T;
        } else {
            (*eap).line2 = ((*eap).line2 as c_int + (count - 1 as linenr_T) as c_int) as linenr_T;
        }
        (*eap).addr_count += 1;
        if validate as c_int != 0 && (*eap).line2 > (*curbuf.get()).b_ml.ml_line_count {
            (*eap).line2 = (*curbuf.get()).b_ml.ml_line_count;
        }
    };
}
unsafe extern "C" fn parse_count(
    mut eap: *mut exarg_T,
    mut errormsg: *mut *const c_char,
    mut validate: bool,
) -> c_int {
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if (*eap).argt & EX_COUNT as uint32_t != 0
        && ascii_isdigit(*(*eap).arg as c_int) as c_int != 0
        && ((*eap).argt & EX_BUFNAME as uint32_t == 0
            || {
                p = skipdigits((*eap).arg.offset(1 as c_int as isize));
                *p as c_int == NUL
            }
            || ascii_iswhite(*p as c_int) as c_int != 0)
    {
        let mut n: linenr_T =
            getdigits_int32(&raw mut (*eap).arg, false_0 != 0, INT32_MAX as int32_t);
        (*eap).arg = skipwhite((*eap).arg);
        if !(*eap).args.is_null() {
            '_c2rust_label: {
                if (*eap).argc > 0 as size_t
                    && (*eap).arg >= *(*eap).args.offset(0 as c_int as isize)
                {
                } else {
                    __assert_fail(
                        b"eap->argc > 0 && eap->arg >= eap->args[0]\0".as_ptr() as *const c_char,
                        b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                        1467 as c_uint,
                        b"int parse_count(exarg_T *, const char **, _Bool)\0".as_ptr()
                            as *const c_char,
                    );
                }
            };
            if (*eap).arg
                < (*(*eap).args.offset(0 as c_int as isize))
                    .offset(*(*eap).arglens.offset(0 as c_int as isize) as isize)
            {
                *(*eap).arglens.offset(0 as c_int as isize) =
                    (*(*eap).arglens.offset(0 as c_int as isize)).wrapping_sub(
                        (*eap)
                            .arg
                            .offset_from(*(*eap).args.offset(0 as c_int as isize))
                            as size_t,
                    );
                *(*eap).args.offset(0 as c_int as isize) = (*eap).arg;
            } else {
                shift_cmd_args(eap);
            }
        }
        if n <= 0 as linenr_T && (*eap).argt & EX_ZEROR as uint32_t == 0 as uint32_t {
            if !errormsg.is_null() {
                *errormsg = gettext(&raw const e_zerocount as *const c_char);
            }
            return FAIL;
        }
        set_cmd_count(eap, n, validate);
    }
    return OK;
}
pub unsafe extern "C" fn is_cmd_ni(mut cmdidx: cmdidx_T) -> bool {
    return !((cmdidx as c_int) < 0 as c_int)
        && (ex_func_is((*cmdnames.ptr())[cmdidx as usize].cmd_func, ex_ni)
            || ex_func_is((*cmdnames.ptr())[cmdidx as usize].cmd_func, ex_script_ni));
}
unsafe extern "C" fn find_excmd_after_range(mut eap: *mut exarg_T) -> *mut c_char {
    let mut cmd: *mut c_char = (*eap).cmd;
    (*eap).cmd = skip_range((*eap).cmd, ::core::ptr::null_mut::<c_int>());
    let mut p: *mut c_char = find_ex_command(eap, ::core::ptr::null_mut::<c_int>());
    (*eap).cmd = cmd;
    return p;
}
unsafe extern "C" fn parse_bang(mut eap: *const exarg_T, mut p: *mut *mut c_char) -> bool {
    if **p as c_int == '!' as c_int
        && (*eap).cmdidx as c_int != CMD_substitute as c_int
        && (*eap).cmdidx as c_int != CMD_smagic as c_int
        && (*eap).cmdidx as c_int != CMD_snomagic as c_int
    {
        *p = (*p).offset(1);
        return true_0 != 0;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn cmd_has_expr_args(mut cmdidx: cmdidx_T) -> bool {
    return cmdidx as c_int == CMD_execute as c_int
        || cmdidx as c_int == CMD_echo as c_int
        || cmdidx as c_int == CMD_echon as c_int
        || cmdidx as c_int == CMD_echomsg as c_int
        || cmdidx as c_int == CMD_echoerr as c_int;
}
pub unsafe extern "C" fn parse_cmdline(
    mut cmdline: *mut *mut c_char,
    mut eap: *mut exarg_T,
    mut cmdinfo: *mut CmdParseInfo,
    mut errormsg: *mut *const c_char,
) -> bool {
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut after_modifier: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut retval: bool = false_0 != 0;
    let save_ex_pressedreturn: bool = ex_pressedreturn.get();
    let save_cursor: pos_T = (*curwin.get()).w_cursor;
    save_last_search_pattern();
    memset(
        cmdinfo as *mut c_void,
        0 as c_int,
        ::core::mem::size_of::<CmdParseInfo>(),
    );
    *eap = exarg {
        arg: ::core::ptr::null_mut::<c_char>(),
        args: ::core::ptr::null_mut::<*mut c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<c_char>(),
        cmd: *cmdline,
        cmdlinep: cmdline,
        cmdline_tofree: ::core::ptr::null_mut::<c_char>(),
        cmdidx: CMD_append,
        argt: 0,
        skip: 0,
        forceit: 0,
        addr_count: 0,
        line1: 1 as linenr_T,
        line2: 1 as linenr_T,
        addr_type: ADDR_LINES,
        flags: 0,
        do_ecmd_cmd: ::core::ptr::null_mut::<c_char>(),
        do_ecmd_lnum: 0,
        append: 0,
        usefilter: 0,
        amount: 0,
        regname: 0,
        force_bin: 0,
        read_edit: 0,
        mkdir_p: 0,
        force_ff: 0,
        force_enc: 0,
        bad_char: 0,
        useridx: 0,
        errmsg: ::core::ptr::null_mut::<c_char>(),
        ea_getline: None,
        cookie: NULL_1,
        cstack: ::core::ptr::null_mut::<cstack_T>(),
    };
    let mut orig_cmd: *mut c_char = (*eap).cmd;
    let mut result: c_int =
        parse_command_modifiers(eap, errormsg, &raw mut (*cmdinfo).cmdmod, false_0 != 0);
    after_modifier = (*eap).cmd;
    if !(result == FAIL && after_modifier == orig_cmd) {
        p = find_excmd_after_range(eap);
        if p.is_null() {
            *errormsg = gettext(
                (e_ambiguous_use_of_user_defined_command.ptr() as *const _) as *const c_char,
            );
        } else {
            set_cmd_addr_type(eap, p);
            if parse_cmd_address(eap, errormsg, true_0 != 0) != FAIL {
                (*eap).cmd = skip_colon_white((*eap).cmd, true_0 != 0);
                if *(*eap).cmd as c_int != '"' as c_int {
                    if !(*(*eap).cmd as c_int == NUL
                        && (*eap).addr_count == 0 as c_int
                        && after_modifier == *cmdline)
                    {
                        if *(*eap).cmd as c_int == NUL
                            && (*eap).cmdidx as c_int == CMD_SIZE as c_int
                        {
                            (*eap).arg = (*eap).cmd;
                            if (*eap).addr_count > 0 as c_int {
                                (*eap).argt = EX_RANGE as uint32_t;
                            } else {
                                (*eap).argt = 0 as uint32_t;
                                (*eap).addr_type = ADDR_NONE;
                            }
                            retval = true_0 != 0;
                        } else if (*eap).cmdidx as c_int == CMD_SIZE as c_int {
                            xstrlcpy(
                                IObuff.ptr() as *mut c_char,
                                gettext(
                                    (e_not_an_editor_command.ptr() as *const _) as *const c_char,
                                ),
                                IOSIZE as size_t,
                            );
                            let mut cmdname: *mut c_char = if !after_modifier.is_null() {
                                after_modifier
                            } else {
                                *cmdline
                            };
                            append_command(cmdname);
                            *errormsg = IObuff.ptr() as *mut c_char;
                        } else {
                            (*eap).forceit = parse_bang(eap, &raw mut p) as c_int;
                            if !(((*eap).cmdidx as c_int) < 0 as c_int) {
                                (*eap).argt =
                                    (*cmdnames.ptr())[(*eap).cmdidx as c_int as usize].cmd_argt;
                            }
                            (*eap).arg = if (*eap).cmdidx as c_int == CMD_bang as c_int {
                                p
                            } else {
                                skipwhite(p)
                            };
                            if (*eap).cmdidx as c_int == CMD_read as c_int && (*eap).forceit != 0 {
                                (*eap).forceit = false_0;
                            }
                            if (*eap).argt & EX_TRLBAR as uint32_t != 0 {
                                separate_nextcmd(eap);
                            } else if cmd_has_expr_args((*eap).cmdidx) {
                                let mut arg: *mut c_char = (*eap).arg;
                                while *arg as c_int != NUL
                                    && *arg as c_int != '|' as c_int
                                    && *arg as c_int != '\n' as c_int
                                {
                                    let mut start: *mut c_char = arg;
                                    (*emsg_skip.ptr()) += 1;
                                    skip_expr(&raw mut arg, ::core::ptr::null_mut::<evalarg_T>());
                                    (*emsg_skip.ptr()) -= 1;
                                    if arg == start {
                                        arg = arg.offset(1);
                                    }
                                }
                                if *arg as c_int == '|' as c_int || *arg as c_int == '\n' as c_int {
                                    (*eap).nextcmd = check_nextcmd(arg);
                                    *arg = NUL as c_char;
                                }
                            }
                            if (*eap).argt & EX_BANG as uint32_t == 0 && (*eap).forceit != 0 {
                                *errormsg = gettext(&raw const e_nobang as *const c_char);
                            } else if (*eap).argt & EX_RANGE as uint32_t == 0
                                && (*eap).addr_count > 0 as c_int
                            {
                                *errormsg = gettext(&raw const e_norange as *const c_char);
                            } else {
                                if (*eap).argt & EX_DFLALL as uint32_t != 0
                                    && (*eap).addr_count == 0 as c_int
                                {
                                    set_cmd_dflall_range(eap);
                                }
                                parse_register(eap);
                                if parse_count(eap, errormsg, false_0 != 0) != FAIL {
                                    if !(*eap).nextcmd.is_null() {
                                        (*eap).nextcmd =
                                            skip_colon_white((*eap).nextcmd, true_0 != 0);
                                    }
                                    if (*eap).argt & EX_XFILE as uint32_t != 0 {
                                        (*cmdinfo).magic.file = true_0 != 0;
                                    }
                                    if (*eap).argt & EX_TRLBAR as uint32_t != 0 {
                                        (*cmdinfo).magic.bar = true_0 != 0;
                                    }
                                    retval = true_0 != 0;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if !retval {
        undo_cmdmod(&raw mut (*cmdinfo).cmdmod);
    }
    ex_pressedreturn.set(save_ex_pressedreturn);
    (*curwin.get()).w_cursor = save_cursor;
    restore_last_search_pattern();
    return retval;
}
unsafe extern "C" fn shift_cmd_args(mut eap: *mut exarg_T) {
    '_c2rust_label: {
        if !(*eap).args.is_null() && (*eap).argc > 0 as size_t {
        } else {
            __assert_fail(
                b"eap->args != NULL && eap->argc > 0\0".as_ptr() as *const c_char,
                b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                1708 as c_uint,
                b"void shift_cmd_args(exarg_T *)\0".as_ptr() as *const c_char,
            );
        }
    };
    let mut oldargs: *mut *mut c_char = (*eap).args;
    let mut oldarglens: *mut size_t = (*eap).arglens;
    (*eap).argc = (*eap).argc.wrapping_sub(1);
    (*eap).args = (if (*eap).argc > 0 as size_t {
        xcalloc((*eap).argc, ::core::mem::size_of::<*mut c_char>())
    } else {
        NULL_1
    }) as *mut *mut c_char;
    (*eap).arglens = (if (*eap).argc > 0 as size_t {
        xcalloc((*eap).argc, ::core::mem::size_of::<size_t>())
    } else {
        NULL_1
    }) as *mut size_t;
    let mut i: size_t = 0 as size_t;
    while i < (*eap).argc {
        *(*eap).args.offset(i as isize) = *oldargs.offset(i.wrapping_add(1 as size_t) as isize);
        *(*eap).arglens.offset(i as isize) =
            *oldarglens.offset(i.wrapping_add(1 as size_t) as isize);
        i = i.wrapping_add(1);
    }
    (*eap).arg = if (*eap).argc > 0 as size_t {
        *(*eap).args.offset(0 as c_int as isize)
    } else {
        (*oldargs.offset(0 as c_int as isize))
            .offset(*oldarglens.offset(0 as c_int as isize) as isize)
    };
    xfree(oldargs as *mut c_void);
    xfree(oldarglens as *mut c_void);
}
unsafe extern "C" fn execute_cmd0(
    mut retv: *mut c_int,
    mut eap: *mut exarg_T,
    mut errormsg: *mut *const c_char,
    mut preview: bool,
) -> c_int {
    if (*eap).argt & EX_XFILE as uint32_t != 0 {
        if expand_filename(eap, (*eap).cmdlinep, errormsg) == FAIL {
            return FAIL;
        }
    }
    if (*eap).argt & EX_BUFNAME as uint32_t != 0
        && *(*eap).arg as c_int != NUL
        && (*eap).addr_count == 0 as c_int
        && !(((*eap).cmdidx as c_int) < 0 as c_int)
    {
        if (*eap).args.is_null() {
            let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
            if (*eap).cmdidx as c_int == CMD_bdelete as c_int
                || (*eap).cmdidx as c_int == CMD_bwipeout as c_int
                || (*eap).cmdidx as c_int == CMD_bunload as c_int
            {
                p = skiptowhite_esc((*eap).arg);
            } else {
                p = (*eap).arg.offset(strlen((*eap).arg) as isize);
                while p > (*eap).arg
                    && ascii_iswhite(*p.offset(-1 as c_int as isize) as c_int) as c_int != 0
                {
                    p = p.offset(-1);
                }
            }
            (*eap).line2 = buflist_findpat(
                (*eap).arg,
                p,
                (*eap).argt & EX_BUFUNL as uint32_t != 0 as uint32_t,
                false_0 != 0,
                false_0 != 0,
            ) as linenr_T;
            (*eap).addr_count = 1 as c_int;
            (*eap).arg = skipwhite(p);
        } else {
            (*eap).line2 = buflist_findpat(
                *(*eap).args.offset(0 as c_int as isize),
                (*(*eap).args.offset(0 as c_int as isize))
                    .offset(*(*eap).arglens.offset(0 as c_int as isize) as isize),
                (*eap).argt & EX_BUFUNL as uint32_t != 0 as uint32_t,
                false_0 != 0,
                false_0 != 0,
            ) as linenr_T;
            (*eap).addr_count = 1 as c_int;
            shift_cmd_args(eap);
        }
        if (*eap).line2 < 0 as linenr_T {
            return FAIL;
        }
    }
    if (*eap).cmdidx as c_int == CMD_try as c_int && (*cmdmod.ptr()).cmod_did_esilent > 0 as c_int {
        (*emsg_silent.ptr()) -= (*cmdmod.ptr()).cmod_did_esilent;
        emsg_silent.set(if emsg_silent.get() > 0 as c_int {
            emsg_silent.get()
        } else {
            0 as c_int
        });
        (*cmdmod.ptr()).cmod_did_esilent = 0 as c_int;
    }
    if ((*eap).cmdidx as c_int) < 0 as c_int {
        *retv = do_ucmd(eap, preview);
    } else {
        (*eap).errmsg = ::core::ptr::null_mut::<c_char>();
        if preview {
            *retv = (*cmdnames.ptr())[(*eap).cmdidx as usize]
                .cmd_preview_func
                .expect("non-null function pointer")(
                eap,
                cmdpreview_get_ns(),
                cmdpreview_get_bufnr(),
            );
        } else {
            (*cmdnames.ptr())[(*eap).cmdidx as usize]
                .cmd_func
                .expect("non-null function pointer")(eap);
        }
        if !(*eap).errmsg.is_null() {
            *errormsg = (*eap).errmsg;
        }
    }
    return OK;
}
pub unsafe extern "C" fn execute_cmd(
    mut eap: *mut exarg_T,
    mut cmdinfo: *mut CmdParseInfo,
    mut preview: bool,
) -> c_int {
    let mut cstack: cstack_T = cstack_T {
        cs_flags: [0; 50],
        cs_pending: [0; 50],
        cs_pend: C2Rust_Unnamed_34 {
            csp_rv: [::core::ptr::null_mut::<c_void>(); 50],
        },
        cs_forinfo: [::core::ptr::null_mut::<c_void>(); 50],
        cs_line: [0; 50],
        cs_idx: 0,
        cs_looplevel: 0,
        cs_trylevel: 0,
        cs_emsg_silent_list: ::core::ptr::null_mut::<eslist_T>(),
        cs_lflags: 0,
    };
    let mut retv: c_int = 0 as c_int;
    if do_cmdline_start() == FAIL {
        emsg(gettext(&raw const e_command_too_recursive as *const c_char));
        return retv;
    }
    let mut errormsg: *const c_char = ::core::ptr::null::<c_char>();
    let mut save_cmdmod: cmdmod_T = cmdmod.get();
    cmdmod.set((*cmdinfo).cmdmod);
    apply_cmdmod(cmdmod.ptr());
    '_end: {
        if (*curbuf.get()).b_p_ma == 0
            && (*eap).argt & EX_MODIFY as uint32_t != 0
            && !(!(*curbuf.get()).terminal.is_null()
                && ((*eap).cmdidx as c_int == CMD_put as c_int
                    || (*eap).cmdidx as c_int == CMD_iput as c_int))
        {
            errormsg = gettext(&raw const e_modifiable as *const c_char);
        } else {
            if !(((*eap).cmdidx as c_int) < 0 as c_int) {
                if cmdwin_type.get() != 0 as c_int && (*eap).argt & EX_CMDWIN as uint32_t == 0 {
                    errormsg = gettext(&raw const e_cmdwin as *const c_char);
                    break '_end;
                } else if text_locked() as c_int != 0 && (*eap).argt & EX_LOCK_OK as uint32_t == 0 {
                    errormsg = gettext(get_text_locked_msg());
                    break '_end;
                }
            }
            if !((*eap).argt & EX_CMDWIN as uint32_t == 0
                && (*eap).cmdidx as c_int != CMD_checktime as c_int
                && (*eap).cmdidx as c_int != CMD_edit as c_int
                && !((*eap).cmdidx as c_int == CMD_file as c_int && *(*eap).arg as c_int == NUL)
                && !(((*eap).cmdidx as c_int) < 0 as c_int)
                && curbuf_locked() as c_int != 0)
            {
                correct_range(eap);
                if (*eap).cmdidx as c_int == CMD_SIZE as c_int && (*eap).addr_count > 0 as c_int {
                    errormsg = ex_range_without_command(eap);
                } else {
                    if ((*eap).argt & EX_WHOLEFOLD as uint32_t != 0
                        || (*eap).addr_count >= 2 as c_int)
                        && global_busy.get() == 0
                        && (*eap).addr_type as c_uint == ADDR_LINES as c_int as c_uint
                    {
                        hasFolding(
                            curwin.get(),
                            (*eap).line1,
                            &raw mut (*eap).line1,
                            ::core::ptr::null_mut::<linenr_T>(),
                        );
                        hasFolding(
                            curwin.get(),
                            (*eap).line2,
                            ::core::ptr::null_mut::<linenr_T>(),
                            &raw mut (*eap).line2,
                        );
                    }
                    if parse_count(eap, &raw mut errormsg, true_0 != 0) != FAIL {
                        cstack = cstack_T {
                            cs_flags: [0; 50],
                            cs_pending: [0; 50],
                            cs_pend: C2Rust_Unnamed_34 {
                                csp_rv: [::core::ptr::null_mut::<c_void>(); 50],
                            },
                            cs_forinfo: [::core::ptr::null_mut::<c_void>(); 50],
                            cs_line: [0; 50],
                            cs_idx: -1 as c_int,
                            cs_looplevel: 0,
                            cs_trylevel: 0,
                            cs_emsg_silent_list: ::core::ptr::null_mut::<eslist_T>(),
                            cs_lflags: 0,
                        };
                        (*eap).cstack = &raw mut cstack;
                        execute_cmd0(&raw mut retv, eap, &raw mut errormsg, preview);
                    }
                }
            }
        }
    }
    if !errormsg.is_null() && *errormsg as c_int != NUL {
        emsg(errormsg);
    }
    undo_cmdmod(cmdmod.ptr());
    cmdmod.set(save_cmdmod);
    do_cmdline_end();
    return retv;
}
unsafe extern "C" fn profile_cmd(
    mut eap: *const exarg_T,
    mut cstack: *mut cstack_T,
    mut fgetline: LineGetter,
    mut cookie: *mut c_void,
) {
    if do_profiling.get() == PROF_YES
        && ((*eap).skip == 0
            || (*cstack).cs_idx == 0 as c_int
            || (*cstack).cs_idx > 0 as c_int
                && (*cstack).cs_flags[((*cstack).cs_idx - 1 as c_int) as usize]
                    & CSF_ACTIVE as c_int
                    != 0)
    {
        let mut skip: bool =
            did_emsg.get() != 0 || got_int.get() as c_int != 0 || did_throw.get() as c_int != 0;
        if (*eap).cmdidx as c_int == CMD_catch as c_int {
            skip = !skip
                && !((*cstack).cs_idx >= 0 as c_int
                    && (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_THROWN as c_int != 0
                    && (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_CAUGHT as c_int == 0);
        } else if (*eap).cmdidx as c_int == CMD_else as c_int
            || (*eap).cmdidx as c_int == CMD_elseif as c_int
        {
            skip = skip as c_int != 0
                || !((*cstack).cs_idx >= 0 as c_int
                    && (*cstack).cs_flags[(*cstack).cs_idx as usize]
                        & (CSF_ACTIVE as c_int | CSF_TRUE as c_int)
                        == 0);
        } else if (*eap).cmdidx as c_int == CMD_finally as c_int {
            skip = false_0 != 0;
        } else if (*eap).cmdidx as c_int != CMD_endif as c_int
            && (*eap).cmdidx as c_int != CMD_endfor as c_int
            && (*eap).cmdidx as c_int != CMD_endtry as c_int
            && (*eap).cmdidx as c_int != CMD_endwhile as c_int
        {
            skip = (*eap).skip != 0;
        }
        if !skip {
            if getline_equal(
                fgetline,
                cookie,
                Some(
                    get_func_line
                        as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                ),
            ) {
                func_line_exec(getline_cookie(fgetline, cookie));
            } else if getline_equal(
                fgetline,
                cookie,
                Some(
                    getsourceline
                        as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                ),
            ) {
                script_line_exec();
            }
        }
    }
}
unsafe extern "C" fn skip_cmd(mut eap: *const exarg_T) -> bool {
    if (*eap).skip != 0 {
        match (*eap).cmdidx as c_int {
            525 | 147 | 167 | 145 | 187 | 141 | 140 | 143 | 488 | 54 | 159 | 146 | 168 | 3
            | 550 | 26 | 31 | 38 | 53 | 97 | 99 | 115 | 126 | 127 | 131 | 132 | 135 | 136 | 138
            | 139 | 149 | 151 | 157 | 176 | 181 | 183 | 188 | 189 | 198 | 199 | 209 | 207 | 206
            | 208 | 230 | 231 | 255 | 256 | 264 | 278 | 288 | 298 | 302 | 323 | 334 | 346 | 349
            | 351 | 355 | 353 | 371 | 374 | 378 | 407 | 410 | 415 | 382 | 444 | 453 | 468 | 473
            | 555 | 484 | 498 | 499 | 506 | 507 | 527 => {}
            _ => return true_0 != 0,
        }
    }
    return false_0 != 0;
}
unsafe extern "C" fn do_one_cmd(
    mut cmdlinep: *mut *mut c_char,
    mut flags: c_int,
    mut cstack: *mut cstack_T,
    mut fgetline: LineGetter,
    mut cookie: *mut c_void,
) -> *mut c_char {
    let mut after_modifier: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut ni: c_int = 0;
    let mut retv: c_int = 0;
    let mut errormsg: *const c_char = ::core::ptr::null::<c_char>();
    let save_reg_executing: c_int = reg_executing.get();
    let save_pending_end_reg_executing: bool = pending_end_reg_executing.get();
    let mut ea: exarg_T = exarg {
        arg: ::core::ptr::null_mut::<c_char>(),
        args: ::core::ptr::null_mut::<*mut c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<c_char>(),
        cmd: ::core::ptr::null_mut::<c_char>(),
        cmdlinep: ::core::ptr::null_mut::<*mut c_char>(),
        cmdline_tofree: ::core::ptr::null_mut::<c_char>(),
        cmdidx: CMD_append,
        argt: 0,
        skip: 0,
        forceit: 0,
        addr_count: 0,
        line1: 1 as linenr_T,
        line2: 1 as linenr_T,
        addr_type: ADDR_LINES,
        flags: 0,
        do_ecmd_cmd: ::core::ptr::null_mut::<c_char>(),
        do_ecmd_lnum: 0,
        append: 0,
        usefilter: 0,
        amount: 0,
        regname: 0,
        force_bin: 0,
        read_edit: 0,
        mkdir_p: 0,
        force_ff: 0,
        force_enc: 0,
        bad_char: 0,
        useridx: 0,
        errmsg: ::core::ptr::null_mut::<c_char>(),
        ea_getline: None,
        cookie: ::core::ptr::null_mut::<c_void>(),
        cstack: ::core::ptr::null_mut::<cstack_T>(),
    };
    (*ex_nesting_level.ptr()) += 1;
    if quitmore.get() != 0
        && !getline_equal(
            fgetline,
            cookie,
            Some(
                get_func_line
                    as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
            ),
        )
        && !getline_equal(
            fgetline,
            cookie,
            Some(getnextac as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char),
        )
    {
        (*quitmore.ptr()) -= 1;
    }
    let mut save_cmdmod: cmdmod_T = cmdmod.get();
    '_doend: {
        if !(*(*cmdlinep).offset(0 as c_int as isize) as c_int == '#' as c_int
            && *(*cmdlinep).offset(1 as c_int as isize) as c_int == '!' as c_int)
        {
            ea.cmd = *cmdlinep;
            ea.cmdlinep = cmdlinep;
            ea.ea_getline = fgetline;
            ea.cookie = cookie;
            ea.cstack = cstack;
            if parse_command_modifiers(&raw mut ea, &raw mut errormsg, cmdmod.ptr(), false_0 != 0)
                != FAIL
            {
                apply_cmdmod(cmdmod.ptr());
                after_modifier = ea.cmd;
                ea.skip = (did_emsg.get() != 0
                    || got_int.get() as c_int != 0
                    || did_throw.get() as c_int != 0
                    || (*cstack).cs_idx >= 0 as c_int
                        && (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_ACTIVE as c_int == 0)
                    as c_int;
                p = find_excmd_after_range(&raw mut ea);
                profile_cmd(&raw mut ea, cstack, fgetline, cookie);
                if !exiting.get() {
                    dbg_check_breakpoint(&raw mut ea);
                }
                if ea.skip == 0 && got_int.get() as c_int != 0 {
                    ea.skip = true_0;
                    do_intthrow(cstack);
                }
                set_cmd_addr_type(&raw mut ea, p);
                if parse_cmd_address(&raw mut ea, &raw mut errormsg, false_0 != 0) != FAIL {
                    ea.cmd = skip_colon_white(ea.cmd, true_0 != 0);
                    if *ea.cmd as c_int == NUL || *ea.cmd as c_int == '"' as c_int || {
                        ea.nextcmd = check_nextcmd(ea.cmd);
                        !ea.nextcmd.is_null()
                    } {
                        if ea.skip == 0 {
                            '_c2rust_label: {
                                if errormsg.is_null() {
                                } else {
                                    __assert_fail(
                                        b"errormsg == NULL\0".as_ptr()
                                            as *const c_char,
                                        b"src/nvim/ex_docmd.rs\0"
                                            .as_ptr() as *const c_char,
                                        2156 as c_uint,
                                        b"char *do_one_cmd(char **, int, cstack_T *, LineGetter, void *)\0"
                                            .as_ptr() as *const c_char,
                                    );
                                }
                            };
                            errormsg = ex_range_without_command(&raw mut ea);
                        }
                    } else {
                        if !p.is_null()
                            && ea.cmdidx as c_int == CMD_SIZE as c_int
                            && ea.skip == 0
                            && (*ea.cmd as c_uint >= 'A' as c_uint
                                && *ea.cmd as c_uint <= 'Z' as c_uint)
                            && has_event(EVENT_CMDUNDEFINED) as c_int != 0
                        {
                            let mut cmdname: *mut c_char = ea.cmd;
                            while *cmdname as c_uint >= 'A' as c_uint
                                && *cmdname as c_uint <= 'Z' as c_uint
                                || *cmdname as c_uint >= 'a' as c_uint
                                    && *cmdname as c_uint <= 'z' as c_uint
                                || ascii_isdigit(*cmdname as c_int) as c_int != 0
                            {
                                cmdname = cmdname.offset(1);
                            }
                            cmdname = xmemdupz(
                                ea.cmd as *const c_void,
                                cmdname.offset_from(ea.cmd) as size_t,
                            ) as *mut c_char;
                            let mut ret: c_int = apply_autocmds(
                                EVENT_CMDUNDEFINED,
                                cmdname,
                                cmdname,
                                true_0 != 0,
                                ::core::ptr::null_mut::<buf_T>(),
                            ) as c_int;
                            xfree(cmdname as *mut c_void);
                            p = if ret != 0 && !aborting() {
                                find_ex_command(&raw mut ea, ::core::ptr::null_mut::<c_int>())
                            } else {
                                ea.cmd
                            };
                        }
                        if p.is_null() {
                            if ea.skip == 0 {
                                errormsg = gettext(
                                    (e_ambiguous_use_of_user_defined_command.ptr() as *const _)
                                        as *const c_char,
                                );
                            }
                        } else if ea.cmdidx as c_int == CMD_SIZE as c_int {
                            if ea.skip == 0 {
                                xstrlcpy(
                                    IObuff.ptr() as *mut c_char,
                                    gettext(
                                        (e_not_an_editor_command.ptr() as *const _)
                                            as *const c_char,
                                    ),
                                    IOSIZE as size_t,
                                );
                                let mut cmdname_0: *mut c_char = if !after_modifier.is_null() {
                                    after_modifier
                                } else {
                                    *cmdlinep
                                };
                                if flags & DOCMD_VERBOSE as c_int == 0 {
                                    append_command(cmdname_0);
                                }
                                errormsg = IObuff.ptr() as *mut c_char;
                                did_emsg_syntax.set(true_0 != 0);
                                verify_command(cmdname_0);
                            }
                        } else {
                            ni = is_cmd_ni(ea.cmdidx) as c_int;
                            ea.forceit = parse_bang(&raw mut ea, &raw mut p) as c_int;
                            if !((ea.cmdidx as c_int) < 0 as c_int) {
                                ea.argt = (*cmdnames.ptr())[ea.cmdidx as c_int as usize].cmd_argt;
                            }
                            if ea.skip == 0 {
                                if sandbox.get() != 0 as c_int
                                    && ea.argt & EX_SBOXOK as uint32_t == 0
                                {
                                    errormsg = gettext(&raw const e_sandbox as *const c_char);
                                    break '_doend;
                                } else if (*curbuf.get()).b_p_ma == 0
                                    && ea.argt & EX_MODIFY as uint32_t != 0
                                    && !(!(*curbuf.get()).terminal.is_null()
                                        && (ea.cmdidx as c_int == CMD_put as c_int
                                            || ea.cmdidx as c_int == CMD_iput as c_int))
                                {
                                    errormsg = gettext(&raw const e_modifiable as *const c_char);
                                    break '_doend;
                                } else {
                                    if !((ea.cmdidx as c_int) < 0 as c_int) {
                                        if cmdwin_type.get() != 0 as c_int
                                            && ea.argt & EX_CMDWIN as uint32_t == 0
                                        {
                                            errormsg =
                                                gettext(&raw const e_cmdwin as *const c_char);
                                            break '_doend;
                                        } else if text_locked() as c_int != 0
                                            && ea.argt & EX_LOCK_OK as uint32_t == 0
                                        {
                                            errormsg = gettext(get_text_locked_msg());
                                            break '_doend;
                                        }
                                    }
                                    if ea.argt & EX_CMDWIN as uint32_t == 0
                                        && ea.cmdidx as c_int != CMD_checktime as c_int
                                        && ea.cmdidx as c_int != CMD_edit as c_int
                                        && ea.cmdidx as c_int != CMD_file as c_int
                                        && !((ea.cmdidx as c_int) < 0 as c_int)
                                        && curbuf_locked() as c_int != 0
                                    {
                                        break '_doend;
                                    } else if ni == 0
                                        && ea.argt & EX_RANGE as uint32_t == 0
                                        && ea.addr_count > 0 as c_int
                                    {
                                        errormsg = gettext(&raw const e_norange as *const c_char);
                                        break '_doend;
                                    }
                                }
                            }
                            if ni == 0 && ea.argt & EX_BANG as uint32_t == 0 && ea.forceit != 0 {
                                errormsg = gettext(&raw const e_nobang as *const c_char);
                            } else {
                                if ea.skip == 0 && ni == 0 && ea.argt & EX_RANGE as uint32_t != 0 {
                                    if global_busy.get() == 0 && ea.line1 > ea.line2 {
                                        if msg_silent.get() == 0 as c_int {
                                            if flags & DOCMD_VERBOSE as c_int != 0
                                                || exmode_active.get() as c_int != 0
                                            {
                                                errormsg = gettext(
                                                    b"E493: Backwards range given\0".as_ptr()
                                                        as *const c_char,
                                                );
                                                break '_doend;
                                            } else if ask_yesno(gettext(
                                                b"Backwards range given, OK to swap\0".as_ptr()
                                                    as *const c_char,
                                            )) != 'y' as c_int
                                            {
                                                break '_doend;
                                            }
                                        }
                                        let mut lnum: linenr_T = ea.line1;
                                        ea.line1 = ea.line2;
                                        ea.line2 = lnum;
                                    }
                                    errormsg = invalid_range(&raw mut ea);
                                    if !errormsg.is_null() {
                                        break '_doend;
                                    }
                                }
                                if ea.addr_type as c_uint == ADDR_OTHER as c_int as c_uint
                                    && ea.addr_count == 0 as c_int
                                {
                                    ea.line2 = 1 as c_int as linenr_T;
                                }
                                correct_range(&raw mut ea);
                                if (ea.argt & EX_WHOLEFOLD as uint32_t != 0
                                    || ea.addr_count >= 2 as c_int)
                                    && global_busy.get() == 0
                                    && ea.addr_type as c_uint == ADDR_LINES as c_int as c_uint
                                {
                                    hasFolding(
                                        curwin.get(),
                                        ea.line1,
                                        &raw mut ea.line1,
                                        ::core::ptr::null_mut::<linenr_T>(),
                                    );
                                    hasFolding(
                                        curwin.get(),
                                        ea.line2,
                                        ::core::ptr::null_mut::<linenr_T>(),
                                        &raw mut ea.line2,
                                    );
                                }
                                p = replace_makeprg(&raw mut ea, p, cmdlinep);
                                if !p.is_null() {
                                    ea.arg = if ea.cmdidx as c_int == CMD_bang as c_int {
                                        p
                                    } else {
                                        skipwhite(p)
                                    };
                                    if !(ea.cmdidx as c_int == CMD_file as c_int
                                        && *ea.arg as c_int != NUL
                                        && curbuf_locked() as c_int != 0)
                                    {
                                        's_449: {
                                            if ea.argt & EX_ARGOPT as uint32_t != 0 {
                                                loop {
                                                    if !(*ea.arg.offset(0 as c_int as isize)
                                                        as c_int
                                                        == '+' as c_int
                                                        && *ea.arg.offset(1 as c_int as isize)
                                                            as c_int
                                                            == '+' as c_int)
                                                    {
                                                        break 's_449;
                                                    }
                                                    if !(getargopt(&raw mut ea) == FAIL && ni == 0)
                                                    {
                                                        continue;
                                                    }
                                                    errormsg = gettext(
                                                        &raw const e_invarg as *const c_char,
                                                    );
                                                    break '_doend;
                                                }
                                            }
                                        }
                                        if ea.cmdidx as c_int == CMD_write as c_int
                                            || ea.cmdidx as c_int == CMD_update as c_int
                                        {
                                            if *ea.arg as c_int == '>' as c_int {
                                                ea.arg = ea.arg.offset(1);
                                                if *ea.arg as c_int != '>' as c_int {
                                                    errormsg =
                                                        gettext(b"E494: Use w or w>>\0".as_ptr()
                                                            as *const c_char);
                                                    break '_doend;
                                                } else {
                                                    ea.arg = skipwhite(
                                                        ea.arg.offset(1 as c_int as isize),
                                                    );
                                                    ea.append = true_0;
                                                }
                                            } else if *ea.arg as c_int == '!' as c_int
                                                && ea.cmdidx as c_int == CMD_write as c_int
                                            {
                                                ea.arg = ea.arg.offset(1);
                                                ea.usefilter = true_0;
                                            }
                                        } else if ea.cmdidx as c_int == CMD_read as c_int {
                                            if ea.forceit != 0 {
                                                ea.usefilter = true_0;
                                                ea.forceit = false_0;
                                            } else if *ea.arg as c_int == '!' as c_int {
                                                ea.arg = ea.arg.offset(1);
                                                ea.usefilter = true_0;
                                            }
                                        } else if ea.cmdidx as c_int == CMD_lshift as c_int
                                            || ea.cmdidx as c_int == CMD_rshift as c_int
                                        {
                                            ea.amount = 1 as c_int;
                                            while *ea.arg as c_int == *ea.cmd as c_int {
                                                ea.arg = ea.arg.offset(1);
                                                ea.amount += 1;
                                            }
                                            ea.arg = skipwhite(ea.arg);
                                        }
                                        if ea.argt & EX_CMDARG as uint32_t != 0 && ea.usefilter == 0
                                        {
                                            ea.do_ecmd_cmd = getargcmd(&raw mut ea.arg);
                                        }
                                        if ea.argt & EX_TRLBAR as uint32_t != 0 && ea.usefilter == 0
                                        {
                                            separate_nextcmd(&raw mut ea);
                                        } else if ea.cmdidx as c_int == CMD_bang as c_int
                                            || ea.cmdidx as c_int == CMD_terminal as c_int
                                            || ea.cmdidx as c_int == CMD_global as c_int
                                            || ea.cmdidx as c_int == CMD_vglobal as c_int
                                            || ea.usefilter != 0
                                        {
                                            let mut s: *mut c_char = ea.arg;
                                            while *s != 0 {
                                                if *s as c_int == '\\' as c_int
                                                    && *s.offset(1 as c_int as isize) as c_int
                                                        == '\n' as c_int
                                                {
                                                    memmove(
                                                        s as *mut c_void,
                                                        s.offset(1 as c_int as isize)
                                                            as *const c_void,
                                                        strlen(s.offset(1 as c_int as isize))
                                                            .wrapping_add(1 as size_t),
                                                    );
                                                } else if *s as c_int == '\n' as c_int {
                                                    ea.nextcmd = s.offset(1 as c_int as isize);
                                                    *s = NUL as c_char;
                                                    break;
                                                }
                                                s = s.offset(1);
                                            }
                                        }
                                        if ea.argt & EX_DFLALL as uint32_t != 0
                                            && ea.addr_count == 0 as c_int
                                        {
                                            set_cmd_dflall_range(&raw mut ea);
                                        }
                                        parse_register(&raw mut ea);
                                        if parse_count(&raw mut ea, &raw mut errormsg, true_0 != 0)
                                            != FAIL
                                        {
                                            if ea.argt & EX_FLAGS as uint32_t != 0 {
                                                get_flags(&raw mut ea);
                                            }
                                            if ni == 0
                                                && ea.argt & EX_EXTRA as uint32_t == 0
                                                && *ea.arg as c_int != NUL
                                                && *ea.arg as c_int != '"' as c_int
                                                && (*ea.arg as c_int != '|' as c_int
                                                    || ea.argt & EX_TRLBAR as uint32_t
                                                        == 0 as uint32_t)
                                            {
                                                errormsg = ex_errmsg(
                                                    &raw const e_trailing_arg as *const c_char,
                                                    ea.arg,
                                                );
                                            } else if ni == 0
                                                && ea.argt & EX_NEEDARG as uint32_t != 0
                                                && *ea.arg as c_int == NUL
                                            {
                                                errormsg =
                                                    gettext(&raw const e_argreq as *const c_char);
                                            } else if !skip_cmd(&raw mut ea) {
                                                retv = 0 as c_int;
                                                if execute_cmd0(
                                                    &raw mut retv,
                                                    &raw mut ea,
                                                    &raw mut errormsg,
                                                    false_0 != 0,
                                                ) != FAIL
                                                {
                                                    if need_rethrow.get() {
                                                        do_throw(cstack);
                                                    } else if check_cstack.get() {
                                                        if source_finished(fgetline, cookie) {
                                                            do_finish(&raw mut ea, true_0 != 0);
                                                        } else if getline_equal(
                                                            fgetline,
                                                            cookie,
                                                            Some(
                                                                get_func_line
                                                                    as unsafe extern "C" fn(
                                                                        c_int,
                                                                        *mut c_void,
                                                                        c_int,
                                                                        bool,
                                                                    ) -> *mut c_char,
                                                            ),
                                                        ) as c_int != 0 && current_func_returned() != 0
                                                        {
                                                            do_return(&raw mut ea, true_0 != 0, false_0 != 0, NULL_1);
                                                        }
                                                    }
                                                    check_cstack.set(false_0 != 0);
                                                    need_rethrow.set(check_cstack.get());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if (*curwin.get()).w_cursor.lnum == 0 as linenr_T {
        (*curwin.get()).w_cursor.lnum = 1 as c_int as linenr_T;
        (*curwin.get()).w_cursor.col = 0 as c_int as colnr_T;
    }
    if !errormsg.is_null() && *errormsg as c_int != NUL && did_emsg.get() == 0 {
        if flags & DOCMD_VERBOSE as c_int != 0 {
            if errormsg != IObuff.ptr() as *mut c_char as *const c_char {
                xstrlcpy(IObuff.ptr() as *mut c_char, errormsg, IOSIZE as size_t);
                errormsg = IObuff.ptr() as *mut c_char;
            }
            append_command(*ea.cmdlinep);
        }
        emsg(errormsg);
    }
    do_errthrow(
        cstack,
        if ea.cmdidx as c_int != CMD_SIZE as c_int && !((ea.cmdidx as c_int) < 0 as c_int) {
            (*cmdnames.ptr())[ea.cmdidx as c_int as usize].cmd_name
        } else {
            ::core::ptr::null_mut::<c_char>()
        },
    );
    undo_cmdmod(cmdmod.ptr());
    cmdmod.set(save_cmdmod);
    reg_executing.set(save_reg_executing);
    pending_end_reg_executing.set(save_pending_end_reg_executing);
    if !ea.nextcmd.is_null() && *ea.nextcmd as c_int == NUL {
        ea.nextcmd = ::core::ptr::null_mut::<c_char>();
    }
    (*ex_nesting_level.ptr()) -= 1;
    xfree(ea.cmdline_tofree as *mut c_void);
    return ea.nextcmd;
}
static ex_error_buf: GlobalCell<[c_char; 480]> = GlobalCell::new([0; 480]);
pub unsafe extern "C" fn ex_errmsg(msg_0: *const c_char, arg: *const c_char) -> *mut c_char {
    vim_snprintf(
        ex_error_buf.ptr() as *mut c_char,
        MSG_BUF_LEN as size_t,
        gettext(msg_0),
        arg,
    );
    return ex_error_buf.ptr() as *mut c_char;
}
static exmode_plus: GlobalCell<[c_char; 2]> = GlobalCell::new(c_bytes(b"+\0"));
unsafe extern "C" fn ex_range_without_command(mut eap: *mut exarg_T) -> *mut c_char {
    let mut errormsg: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if *(*eap).cmd as c_int == '|' as c_int
        || exmode_active.get() as c_int != 0
            && (*eap).cmd != (exmode_plus.ptr() as *mut c_char).offset(1 as c_int as isize)
    {
        (*eap).cmdidx = CMD_print;
        (*eap).argt = (EX_RANGE | EX_COUNT | EX_TRLBAR) as uint32_t;
        errormsg = invalid_range(eap);
        if errormsg.is_null() {
            correct_range(eap);
            ex_print(eap);
        }
    } else if (*eap).addr_count != 0 as c_int {
        (*eap).line2 = if (*eap).line2 < (*curbuf.get()).b_ml.ml_line_count {
            (*eap).line2
        } else {
            (*curbuf.get()).b_ml.ml_line_count
        };
        if (*eap).line2 < 0 as linenr_T {
            errormsg = gettext(&raw const e_invrange as *const c_char);
        } else {
            if (*eap).line2 == 0 as linenr_T {
                (*curwin.get()).w_cursor.lnum = 1 as c_int as linenr_T;
            } else {
                (*curwin.get()).w_cursor.lnum = (*eap).line2;
            }
            beginline(BL_SOL as c_int | BL_FIX as c_int);
        }
    }
    return errormsg;
}
pub unsafe extern "C" fn parse_command_modifiers(
    mut eap: *mut exarg_T,
    mut errormsg: *mut *const c_char,
    mut cmod: *mut cmdmod_T,
    mut skip_only: bool,
) -> c_int {
    let mut orig_cmd: *mut c_char = (*eap).cmd;
    let mut cmd_start: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut use_plus_cmd: bool = false_0 != 0;
    let mut has_visual_range: bool = false_0 != 0;
    memset(
        cmod as *mut c_void,
        0 as c_int,
        ::core::mem::size_of::<cmdmod_T>(),
    );
    if strncmp(
        (*eap).cmd,
        b"'<,'>\0".as_ptr() as *const c_char,
        5 as size_t,
    ) == 0 as c_int
    {
        let mut p: *const c_char = skipwhite((*eap).cmd.offset(5 as c_int as isize));
        if *p as c_int != NUL && *p as c_int != '|' as c_int {
            (*eap).cmd = (*eap).cmd.offset(5 as c_int as isize);
            cmd_start = (*eap).cmd;
            has_visual_range = true_0 != 0;
        }
    }
    loop {
        while *(*eap).cmd as c_int == ' ' as c_int
            || *(*eap).cmd as c_int == '\t' as c_int
            || *(*eap).cmd as c_int == ':' as c_int
        {
            (*eap).cmd = (*eap).cmd.offset(1);
        }
        if *(*eap).cmd as c_int == NUL
            && exmode_active.get() as c_int != 0
            && getline_equal(
                (*eap).ea_getline,
                (*eap).cookie,
                Some(
                    getexline
                        as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                ),
            ) as c_int
                != 0
            && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count
        {
            (*eap).cmd = exmode_plus.ptr() as *mut c_char;
            use_plus_cmd = true_0 != 0;
            if !skip_only {
                ex_pressedreturn.set(true_0 != 0);
            }
            break;
        } else {
            if *(*eap).cmd as c_int == '"' as c_int {
                (*eap).nextcmd = vim_strchr((*eap).cmd, '\n' as c_int);
                if !(*eap).nextcmd.is_null() {
                    (*eap).nextcmd = (*eap).nextcmd.offset(1);
                }
                return FAIL;
            }
            if *(*eap).cmd as c_int == '\n' as c_int {
                (*eap).nextcmd = (*eap).cmd.offset(1 as c_int as isize);
                return FAIL;
            }
            if *(*eap).cmd as c_int == NUL {
                if !skip_only {
                    ex_pressedreturn.set(true_0 != 0);
                }
                return FAIL;
            }
            let mut p_0: *mut c_char = skip_range((*eap).cmd, ::core::ptr::null_mut::<c_int>());
            match *p_0 as c_int {
                97 => {
                    if !checkforcmd(
                        &raw mut (*eap).cmd,
                        b"aboveleft\0".as_ptr() as *const c_char,
                        3 as c_int,
                    ) {
                        break;
                    }
                    (*cmod).cmod_split |= WSP_ABOVE as c_int;
                }
                98 => {
                    if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"belowright\0".as_ptr() as *const c_char,
                        3 as c_int,
                    ) {
                        (*cmod).cmod_split |= WSP_BELOW as c_int;
                    } else if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"browse\0".as_ptr() as *const c_char,
                        3 as c_int,
                    ) {
                        (*cmod).cmod_flags |= CMOD_BROWSE as c_int;
                    } else {
                        if !checkforcmd(
                            &raw mut (*eap).cmd,
                            b"botright\0".as_ptr() as *const c_char,
                            2 as c_int,
                        ) {
                            break;
                        }
                        (*cmod).cmod_split |= WSP_BOT as c_int;
                    }
                }
                99 => {
                    if !checkforcmd(
                        &raw mut (*eap).cmd,
                        b"confirm\0".as_ptr() as *const c_char,
                        4 as c_int,
                    ) {
                        break;
                    }
                    (*cmod).cmod_flags |= CMOD_CONFIRM as c_int;
                }
                107 => {
                    if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"keepmarks\0".as_ptr() as *const c_char,
                        3 as c_int,
                    ) {
                        (*cmod).cmod_flags |= CMOD_KEEPMARKS as c_int;
                    } else if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"keepalt\0".as_ptr() as *const c_char,
                        5 as c_int,
                    ) {
                        (*cmod).cmod_flags |= CMOD_KEEPALT as c_int;
                    } else if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"keeppatterns\0".as_ptr() as *const c_char,
                        5 as c_int,
                    ) {
                        (*cmod).cmod_flags |= CMOD_KEEPPATTERNS as c_int;
                    } else {
                        if !checkforcmd(
                            &raw mut (*eap).cmd,
                            b"keepjumps\0".as_ptr() as *const c_char,
                            5 as c_int,
                        ) {
                            break;
                        }
                        (*cmod).cmod_flags |= CMOD_KEEPJUMPS as c_int;
                    }
                }
                102 => {
                    let mut reg_pat: *mut c_char = ::core::ptr::null_mut::<c_char>();
                    if !checkforcmd(
                        &raw mut p_0,
                        b"filter\0".as_ptr() as *const c_char,
                        4 as c_int,
                    ) || *p_0 as c_int == NUL
                        || ends_excmd(*p_0 as c_int) != 0
                    {
                        break;
                    }
                    if *p_0 as c_int == '!' as c_int {
                        (*cmod).cmod_filter_force = true_0 != 0;
                        p_0 = skipwhite(p_0.offset(1 as c_int as isize));
                        if *p_0 as c_int == NUL || ends_excmd(*p_0 as c_int) != 0 {
                            break;
                        }
                    }
                    if skip_only {
                        p_0 = skip_vimgrep_pat(
                            p_0,
                            ::core::ptr::null_mut::<*mut c_char>(),
                            ::core::ptr::null_mut::<c_int>(),
                        );
                    } else {
                        p_0 = skip_vimgrep_pat(
                            p_0,
                            &raw mut reg_pat,
                            ::core::ptr::null_mut::<c_int>(),
                        );
                    }
                    if p_0.is_null() || *p_0 as c_int == NUL {
                        break;
                    }
                    if !skip_only {
                        (*cmod).cmod_filter_pat = xstrdup(reg_pat);
                        (*cmod).cmod_filter_regmatch.regprog = vim_regcomp(reg_pat, RE_MAGIC);
                        if (*cmod).cmod_filter_regmatch.regprog.is_null() {
                            break;
                        }
                    }
                    (*eap).cmd = p_0;
                }
                104 => {
                    if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"horizontal\0".as_ptr() as *const c_char,
                        3 as c_int,
                    ) {
                        (*cmod).cmod_split |= WSP_HOR as c_int;
                    } else {
                        if p_0 != (*eap).cmd
                            || !checkforcmd(
                                &raw mut p_0,
                                b"hide\0".as_ptr() as *const c_char,
                                3 as c_int,
                            )
                            || *p_0 as c_int == NUL
                            || ends_excmd(*p_0 as c_int) != 0
                        {
                            break;
                        }
                        (*eap).cmd = p_0;
                        (*cmod).cmod_flags |= CMOD_HIDE as c_int;
                    }
                }
                108 => {
                    if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"lockmarks\0".as_ptr() as *const c_char,
                        3 as c_int,
                    ) {
                        (*cmod).cmod_flags |= CMOD_LOCKMARKS as c_int;
                    } else {
                        if !checkforcmd(
                            &raw mut (*eap).cmd,
                            b"leftabove\0".as_ptr() as *const c_char,
                            5 as c_int,
                        ) {
                            break;
                        }
                        (*cmod).cmod_split |= WSP_ABOVE as c_int;
                    }
                }
                110 => {
                    if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"noautocmd\0".as_ptr() as *const c_char,
                        3 as c_int,
                    ) {
                        (*cmod).cmod_flags |= CMOD_NOAUTOCMD as c_int;
                    } else {
                        if !checkforcmd(
                            &raw mut (*eap).cmd,
                            b"noswapfile\0".as_ptr() as *const c_char,
                            3 as c_int,
                        ) {
                            break;
                        }
                        (*cmod).cmod_flags |= CMOD_NOSWAPFILE as c_int;
                    }
                }
                114 => {
                    if !checkforcmd(
                        &raw mut (*eap).cmd,
                        b"rightbelow\0".as_ptr() as *const c_char,
                        6 as c_int,
                    ) {
                        break;
                    }
                    (*cmod).cmod_split |= WSP_BELOW as c_int;
                }
                115 => {
                    if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"sandbox\0".as_ptr() as *const c_char,
                        3 as c_int,
                    ) {
                        (*cmod).cmod_flags |= CMOD_SANDBOX as c_int;
                    } else {
                        if !checkforcmd(
                            &raw mut (*eap).cmd,
                            b"silent\0".as_ptr() as *const c_char,
                            3 as c_int,
                        ) {
                            break;
                        }
                        (*cmod).cmod_flags |= CMOD_SILENT as c_int;
                        if *(*eap).cmd as c_int == '!' as c_int
                            && !ascii_iswhite(*(*eap).cmd.offset(-1 as c_int as isize) as c_int)
                        {
                            (*eap).cmd = skipwhite((*eap).cmd.offset(1 as c_int as isize));
                            (*cmod).cmod_flags |= CMOD_ERRSILENT as c_int;
                        }
                    }
                }
                116 => {
                    if checkforcmd(&raw mut p_0, b"tab\0".as_ptr() as *const c_char, 3 as c_int) {
                        if !skip_only {
                            let mut tabnr: c_int = get_address(
                                eap,
                                &raw mut (*eap).cmd,
                                ADDR_TABS,
                                (*eap).skip != 0,
                                skip_only,
                                false_0,
                                1 as c_int,
                                errormsg,
                            ) as c_int;
                            if (*eap).cmd.is_null() {
                                return false_0;
                            }
                            if tabnr == MAXLNUM as c_int {
                                (*cmod).cmod_tab = tabpage_index(curtab.get()) + 1 as c_int;
                            } else {
                                if tabnr < 0 as c_int
                                    || tabnr > current_tab_nr(::core::ptr::null_mut::<tabpage_T>())
                                {
                                    *errormsg = gettext(&raw const e_invrange as *const c_char);
                                    return false_0;
                                }
                                (*cmod).cmod_tab = tabnr + 1 as c_int;
                            }
                        }
                        (*eap).cmd = p_0;
                    } else {
                        if !checkforcmd(
                            &raw mut (*eap).cmd,
                            b"topleft\0".as_ptr() as *const c_char,
                            2 as c_int,
                        ) {
                            break;
                        }
                        (*cmod).cmod_split |= WSP_TOP as c_int;
                    }
                }
                117 => {
                    if !checkforcmd(
                        &raw mut (*eap).cmd,
                        b"unsilent\0".as_ptr() as *const c_char,
                        3 as c_int,
                    ) {
                        break;
                    }
                    (*cmod).cmod_flags |= CMOD_UNSILENT as c_int;
                }
                118 => {
                    if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"vertical\0".as_ptr() as *const c_char,
                        4 as c_int,
                    ) {
                        (*cmod).cmod_split |= WSP_VERT as c_int;
                    } else {
                        if !checkforcmd(
                            &raw mut p_0,
                            b"verbose\0".as_ptr() as *const c_char,
                            4 as c_int,
                        ) {
                            break;
                        }
                        if ascii_isdigit(*(*eap).cmd as c_int) {
                            (*cmod).cmod_verbose = atoi((*eap).cmd) + 1 as c_int;
                        } else {
                            (*cmod).cmod_verbose = 2 as c_int;
                        }
                        (*eap).cmd = p_0;
                    }
                }
                _ => {
                    break;
                }
            }
        }
    }
    if has_visual_range {
        if (*eap).cmd > cmd_start {
            if use_plus_cmd {
                let mut len: size_t = strlen(cmd_start);
                memmove(orig_cmd as *mut c_void, cmd_start as *const c_void, len);
                xmemcpyz(
                    orig_cmd.offset(len as isize) as *mut c_void,
                    b" *+\0".as_ptr() as *const c_char as *const c_void,
                    ::core::mem::size_of::<[c_char; 4]>().wrapping_sub(1 as size_t),
                );
            } else {
                memmove(
                    cmd_start.offset(-(5 as c_int as isize)) as *mut c_void,
                    cmd_start as *const c_void,
                    (*eap).cmd.offset_from(cmd_start) as size_t,
                );
                (*eap).cmd = (*eap).cmd.offset(-(5 as c_int as isize));
                memmove(
                    (*eap).cmd.offset(-(1 as c_int as isize)) as *mut c_void,
                    b":'<,'>\0".as_ptr() as *const c_char as *const c_void,
                    6 as size_t,
                );
            }
        } else if use_plus_cmd {
            (*eap).cmd = b"'<,'>+\0".as_ptr() as *const c_char as *mut c_char;
        } else {
            (*eap).cmd = orig_cmd;
        }
    } else if use_plus_cmd {
        (*eap).cmd = exmode_plus.ptr() as *mut c_char;
    }
    return OK;
}
pub unsafe extern "C" fn apply_cmdmod(mut cmod: *mut cmdmod_T) {
    if (*cmod).cmod_flags & CMOD_SANDBOX as c_int != 0 && (*cmod).cmod_did_sandbox == 0 {
        (*sandbox.ptr()) += 1;
        (*cmod).cmod_did_sandbox = true_0;
    }
    if (*cmod).cmod_verbose > 0 as c_int {
        if (*cmod).cmod_verbose_save == 0 as OptInt {
            (*cmod).cmod_verbose_save = p_verbose.get() + 1 as OptInt;
        }
        p_verbose.set(((*cmod).cmod_verbose - 1 as c_int) as OptInt);
    }
    if (*cmod).cmod_flags & (CMOD_SILENT as c_int | CMOD_UNSILENT as c_int) != 0
        && (*cmod).cmod_save_msg_silent == 0 as c_int
    {
        (*cmod).cmod_save_msg_silent = msg_silent.get() + 1 as c_int;
        (*cmod).cmod_save_msg_scroll = msg_scroll.get();
    }
    if (*cmod).cmod_flags & CMOD_SILENT as c_int != 0 {
        (*msg_silent.ptr()) += 1;
    }
    if (*cmod).cmod_flags & CMOD_UNSILENT as c_int != 0 {
        msg_silent.set(0 as c_int);
    }
    if (*cmod).cmod_flags & CMOD_ERRSILENT as c_int != 0 {
        (*emsg_silent.ptr()) += 1;
        (*cmod).cmod_did_esilent += 1;
    }
    if (*cmod).cmod_flags & CMOD_NOAUTOCMD as c_int != 0 && (*cmod).cmod_save_ei.is_null() {
        (*cmod).cmod_save_ei = xstrdup(p_ei.get());
        set_option_direct(
            kOptEventignore,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: String_0 {
                        data: b"all\0".as_ptr() as *const c_char as *mut c_char,
                        size: ::core::mem::size_of::<[c_char; 4]>().wrapping_sub(1 as size_t),
                    },
                },
            },
            0 as c_int,
            SID_NONE,
        );
    }
}
pub unsafe extern "C" fn undo_cmdmod(mut cmod: *mut cmdmod_T) {
    if (*cmod).cmod_verbose_save > 0 as OptInt {
        p_verbose.set((*cmod).cmod_verbose_save - 1 as OptInt);
        (*cmod).cmod_verbose_save = 0 as OptInt;
    }
    if (*cmod).cmod_did_sandbox != 0 {
        (*sandbox.ptr()) -= 1;
        (*cmod).cmod_did_sandbox = false_0;
    }
    if !(*cmod).cmod_save_ei.is_null() {
        set_option_direct(
            kOptEventignore,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string((*cmod).cmod_save_ei),
                },
            },
            0 as c_int,
            SID_NONE,
        );
        free_string_option((*cmod).cmod_save_ei);
        (*cmod).cmod_save_ei = ::core::ptr::null_mut::<c_char>();
    }
    xfree((*cmod).cmod_filter_pat as *mut c_void);
    vim_regfree((*cmod).cmod_filter_regmatch.regprog);
    if (*cmod).cmod_save_msg_silent > 0 as c_int {
        if did_emsg.get() == 0 || msg_silent.get() > (*cmod).cmod_save_msg_silent - 1 as c_int {
            msg_silent.set((*cmod).cmod_save_msg_silent - 1 as c_int);
        }
        (*emsg_silent.ptr()) -= (*cmod).cmod_did_esilent;
        emsg_silent.set(if emsg_silent.get() > 0 as c_int {
            emsg_silent.get()
        } else {
            0 as c_int
        });
        msg_scroll.set((*cmod).cmod_save_msg_scroll);
        if redirecting() != 0 {
            msg_col.set(0 as c_int);
        }
        (*cmod).cmod_save_msg_silent = 0 as c_int;
        (*cmod).cmod_did_esilent = 0 as c_int;
    }
}
pub unsafe extern "C" fn parse_cmd_address(
    mut eap: *mut exarg_T,
    mut errormsg: *mut *const c_char,
    mut silent: bool,
) -> c_int {
    let mut address_count: c_int = 1 as c_int;
    let mut lnum: linenr_T = 0;
    let mut need_check_cursor: bool = false_0 != 0;
    let mut ret: c_int = FAIL;
    '_theend: {
        loop {
            (*eap).line1 = (*eap).line2;
            (*eap).line2 = get_cmd_default_range(eap);
            (*eap).cmd = skipwhite((*eap).cmd);
            let c2rust_fresh29 = address_count;
            address_count = address_count + 1;
            lnum = get_address(
                eap,
                &raw mut (*eap).cmd,
                (*eap).addr_type,
                (*eap).skip != 0,
                silent,
                ((*eap).addr_count == 0 as c_int) as c_int,
                c2rust_fresh29,
                errormsg,
            );
            if (*eap).cmd.is_null() {
                break '_theend;
            }
            if lnum == MAXLNUM as c_int as linenr_T {
                if *(*eap).cmd as c_int == '%' as c_int {
                    (*eap).cmd = (*eap).cmd.offset(1);
                    match (*eap).addr_type as c_uint {
                        0 | 10 => {
                            (*eap).line1 = 1 as c_int as linenr_T;
                            (*eap).line2 = (*curbuf.get()).b_ml.ml_line_count;
                        }
                        3 => {
                            let mut buf: *mut buf_T = firstbuf.get();
                            while !(*buf).b_next.is_null() && (*buf).b_ml.ml_mfp.is_null() {
                                buf = (*buf).b_next;
                            }
                            (*eap).line1 = (*buf).handle as linenr_T;
                            buf = lastbuf.get();
                            while !(*buf).b_prev.is_null() && (*buf).b_ml.ml_mfp.is_null() {
                                buf = (*buf).b_prev;
                            }
                            (*eap).line2 = (*buf).handle as linenr_T;
                        }
                        4 => {
                            (*eap).line1 = (*firstbuf.get()).handle as linenr_T;
                            (*eap).line2 = (*lastbuf.get()).handle as linenr_T;
                        }
                        1 | 5 => {
                            if ((*eap).cmdidx as c_int) < 0 as c_int {
                                (*eap).line1 = 1 as c_int as linenr_T;
                                (*eap).line2 = (if (*eap).addr_type as c_uint
                                    == ADDR_WINDOWS as c_int as c_uint
                                {
                                    current_win_nr(::core::ptr::null::<win_T>())
                                } else {
                                    current_tab_nr(::core::ptr::null_mut::<tabpage_T>())
                                }) as linenr_T;
                            } else {
                                *errormsg = gettext(&raw const e_invrange as *const c_char);
                                break '_theend;
                            }
                        }
                        6 | 9 | 8 => {
                            *errormsg = gettext(&raw const e_invrange as *const c_char);
                            break '_theend;
                        }
                        2 => {
                            if (*(*curwin.get()).w_alist).al_ga.ga_len == 0 as c_int {
                                (*eap).line2 = 0 as c_int as linenr_T;
                                (*eap).line1 = (*eap).line2;
                            } else {
                                (*eap).line1 = 1 as c_int as linenr_T;
                                (*eap).line2 = (*(*curwin.get()).w_alist).al_ga.ga_len as linenr_T;
                            }
                        }
                        7 => {
                            (*eap).line1 = 1 as c_int as linenr_T;
                            (*eap).line2 = qf_get_valid_size(eap) as linenr_T;
                            if (*eap).line2 == 0 as linenr_T {
                                (*eap).line2 = 1 as c_int as linenr_T;
                            }
                        }
                        11 | _ => {}
                    }
                    (*eap).addr_count += 1;
                } else if *(*eap).cmd as c_int == '*' as c_int {
                    if (*eap).addr_type as c_uint != ADDR_LINES as c_int as c_uint {
                        *errormsg = gettext(&raw const e_invrange as *const c_char);
                        break '_theend;
                    } else {
                        (*eap).cmd = (*eap).cmd.offset(1);
                        if (*eap).skip == 0 {
                            let mut fm: *mut fmark_T = mark_get_visual(curbuf.get(), '<' as c_int);
                            if !mark_check(fm, errormsg) {
                                break '_theend;
                            }
                            '_c2rust_label: {
                                if !fm.is_null() {
                                } else {
                                    __assert_fail(
                                        b"fm != NULL\0".as_ptr() as *const c_char,
                                        b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                                        3027 as c_uint,
                                        b"int parse_cmd_address(exarg_T *, const char **, _Bool)\0"
                                            .as_ptr()
                                            as *const c_char,
                                    );
                                }
                            };
                            (*eap).line1 = (*fm).mark.lnum;
                            fm = mark_get_visual(curbuf.get(), '>' as c_int);
                            if !mark_check(fm, errormsg) {
                                break '_theend;
                            }
                            '_c2rust_label_0: {
                                if !fm.is_null() {
                                } else {
                                    __assert_fail(
                                        b"fm != NULL\0".as_ptr() as *const c_char,
                                        b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                                        3033 as c_uint,
                                        b"int parse_cmd_address(exarg_T *, const char **, _Bool)\0"
                                            .as_ptr()
                                            as *const c_char,
                                    );
                                }
                            };
                            (*eap).line2 = (*fm).mark.lnum;
                            (*eap).addr_count += 1;
                        }
                    }
                }
            } else {
                (*eap).line2 = lnum;
            }
            (*eap).addr_count += 1;
            if *(*eap).cmd as c_int == ';' as c_int {
                if (*eap).skip == 0 {
                    (*curwin.get()).w_cursor.lnum = (*eap).line2;
                    if (*eap).line2 > 0 as linenr_T {
                        check_cursor(curwin.get());
                    } else {
                        check_cursor_col(curwin.get());
                    }
                    need_check_cursor = true_0 != 0;
                }
            } else if *(*eap).cmd as c_int != ',' as c_int {
                break;
            }
            (*eap).cmd = (*eap).cmd.offset(1);
        }
        if (*eap).addr_count == 1 as c_int {
            (*eap).line1 = (*eap).line2;
            if lnum == MAXLNUM as c_int as linenr_T {
                (*eap).addr_count = 0 as c_int;
            }
        }
        ret = OK;
    }
    if need_check_cursor {
        check_cursor(curwin.get());
    }
    return ret;
}
pub unsafe extern "C" fn checkforcmd(
    mut pp: *mut *mut c_char,
    mut cmd: *const c_char,
    mut len: c_int,
) -> bool {
    let mut i: c_int = 0;
    i = 0 as c_int;
    while *cmd.offset(i as isize) as c_int != NUL {
        if *cmd.offset(i as isize) as c_int != *(*pp).offset(i as isize) as c_int {
            break;
        }
        i += 1;
    }
    if i >= len
        && !(*(*pp).offset(i as isize) as c_uint >= 'A' as c_uint
            && *(*pp).offset(i as isize) as c_uint <= 'Z' as c_uint
            || *(*pp).offset(i as isize) as c_uint >= 'a' as c_uint
                && *(*pp).offset(i as isize) as c_uint <= 'z' as c_uint)
    {
        *pp = skipwhite((*pp).offset(i as isize));
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn append_command(mut cmd: *const c_char) {
    let mut len: size_t = strlen(IObuff.ptr() as *mut c_char);
    let mut s: *const c_char = cmd;
    let mut d: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if len > (IOSIZE - 100 as c_int) as size_t {
        d = (IObuff.ptr() as *mut c_char)
            .offset(IOSIZE as isize)
            .offset(-(100 as c_int as isize));
        d = d.offset(-(utf_head_off(IObuff.ptr() as *mut c_char, d) as isize));
        strcpy(d, b"...\0".as_ptr() as *const c_char as *mut c_char);
    }
    xstrlcat(
        IObuff.ptr() as *mut c_char,
        b": \0".as_ptr() as *const c_char,
        IOSIZE as size_t,
    );
    d = (IObuff.ptr() as *mut c_char).offset(strlen(IObuff.ptr() as *mut c_char) as isize);
    while *s as c_int != NUL
        && (d.offset_from(IObuff.ptr() as *mut c_char) + 5 as isize) < IOSIZE as isize
    {
        if *s.offset(0 as c_int as isize) as uint8_t as c_int == 0xc2 as c_int
            && *s.offset(1 as c_int as isize) as uint8_t as c_int == 0xa0 as c_int
        {
            s = s.offset(2 as c_int as isize);
            strcpy(d, b"<a0>\0".as_ptr() as *const c_char as *mut c_char);
            d = d.offset(4 as c_int as isize);
        } else {
            if d.offset_from(IObuff.ptr() as *mut c_char) + utfc_ptr2len(s) as isize + 1 as isize
                >= IOSIZE as isize
            {
                break;
            }
            mb_copy_char(&raw mut s, &raw mut d);
        }
    }
    *d = NUL as c_char;
}
unsafe extern "C" fn one_letter_cmd(mut p: *const c_char, mut idx: *mut cmdidx_T) -> c_int {
    if *p.offset(0 as c_int as isize) as c_int == 'k' as c_int
        && (*p.offset(1 as c_int as isize) as c_int != 'e' as c_int
            || *p.offset(1 as c_int as isize) as c_int == 'e' as c_int
                && *p.offset(2 as c_int as isize) as c_int != 'e' as c_int)
    {
        *idx = CMD_k;
        return true_0;
    }
    if *p.offset(0 as c_int as isize) as c_int == 's' as c_int
        && (*p.offset(1 as c_int as isize) as c_int == 'c' as c_int
            && (*p.offset(2 as c_int as isize) as c_int == NUL
                || *p.offset(2 as c_int as isize) as c_int != 's' as c_int
                    && *p.offset(2 as c_int as isize) as c_int != 'r' as c_int
                    && (*p.offset(3 as c_int as isize) as c_int == NUL
                        || *p.offset(3 as c_int as isize) as c_int != 'i' as c_int
                            && *p.offset(4 as c_int as isize) as c_int != 'p' as c_int))
            || *p.offset(1 as c_int as isize) as c_int == 'g' as c_int
            || *p.offset(1 as c_int as isize) as c_int == 'i' as c_int
                && *p.offset(2 as c_int as isize) as c_int != 'm' as c_int
                && *p.offset(2 as c_int as isize) as c_int != 'l' as c_int
                && *p.offset(2 as c_int as isize) as c_int != 'g' as c_int
            || *p.offset(1 as c_int as isize) as c_int == 'I' as c_int
            || *p.offset(1 as c_int as isize) as c_int == 'r' as c_int
                && *p.offset(2 as c_int as isize) as c_int != 'e' as c_int)
    {
        *idx = CMD_substitute;
        return true_0;
    }
    return false_0;
}
pub unsafe extern "C" fn find_ex_command(
    mut eap: *mut exarg_T,
    mut full: *mut c_int,
) -> *mut c_char {
    let mut p: *mut c_char = (*eap).cmd;
    if one_letter_cmd(p, &raw mut (*eap).cmdidx) != 0 {
        p = p.offset(1);
        if !full.is_null() {
            *full = true_0;
        }
    } else {
        while *p as c_uint >= 'A' as c_uint && *p as c_uint <= 'Z' as c_uint
            || *p as c_uint >= 'a' as c_uint && *p as c_uint <= 'z' as c_uint
        {
            p = p.offset(1);
        }
        if *(*eap).cmd.offset(0 as c_int as isize) as c_int == 'p' as c_int
            && *(*eap).cmd.offset(1 as c_int as isize) as c_int == 'y' as c_int
        {
            while *p as c_uint >= 'A' as c_uint && *p as c_uint <= 'Z' as c_uint
                || *p as c_uint >= 'a' as c_uint && *p as c_uint <= 'z' as c_uint
                || ascii_isdigit(*p as c_int) as c_int != 0
            {
                p = p.offset(1);
            }
        }
        if p == (*eap).cmd
            && !vim_strchr(
                b"@!=><&~#\0".as_ptr() as *const c_char,
                *p as uint8_t as c_int,
            )
            .is_null()
        {
            p = p.offset(1);
        }
        let mut len: c_int = p.offset_from((*eap).cmd) as c_int;
        if *(*eap).cmd as c_int == 'd' as c_int
            && (*p.offset(-1 as c_int as isize) as c_int == 'l' as c_int
                || *p.offset(-1 as c_int as isize) as c_int == 'p' as c_int)
        {
            let mut i: c_int = 0;
            i = 0 as c_int;
            while i < len {
                if *(*eap).cmd.offset(i as isize) as c_int
                    != c_bytes(b"delete\0")[i as usize] as c_int
                {
                    break;
                }
                i += 1;
            }
            if i == len - 1 as c_int {
                len -= 1;
                if *p.offset(-1 as c_int as isize) as c_int == 'l' as c_int {
                    (*eap).flags |= EXFLAG_LIST;
                } else {
                    (*eap).flags |= EXFLAG_PRINT;
                }
            }
        }
        if *(*eap).cmd.offset(0 as c_int as isize) as c_uint >= 'a' as c_uint
            && *(*eap).cmd.offset(0 as c_int as isize) as c_uint <= 'z' as c_uint
        {
            let c1: c_int = *(*eap).cmd.offset(0 as c_int as isize) as uint8_t as c_int;
            let c2: c_int = if len == 1 as c_int {
                NUL
            } else {
                *(*eap).cmd.offset(1 as c_int as isize) as c_int
            };
            if command_count.get() != CMD_SIZE as c_int {
                iemsg(gettext(
                    b"E943: Command table needs to be updated, run 'make'\0".as_ptr()
                        as *const c_char,
                ));
                getout(1 as c_int);
            }
            (*eap).cmdidx =
                (*cmdidxs1.ptr())[(c1 as uint8_t as c_int - 'a' as c_int) as usize] as cmdidx_T;
            if c2 as c_uint >= 'a' as c_uint && c2 as c_uint <= 'z' as c_uint {
                (*eap).cmdidx = ((*eap).cmdidx as c_int
                    + (*cmdidxs2.ptr())[(c1 as uint8_t as c_int - 'a' as c_int) as usize]
                        [(c2 as uint8_t as c_int - 'a' as c_int) as usize]
                        as c_int) as cmdidx_T;
            }
        } else if *(*eap).cmd.offset(0 as c_int as isize) as c_uint >= 'A' as c_uint
            && *(*eap).cmd.offset(0 as c_int as isize) as c_uint <= 'Z' as c_uint
        {
            (*eap).cmdidx = CMD_Next;
        } else {
            (*eap).cmdidx = CMD_bang;
        }
        '_c2rust_label: {
            if (*eap).cmdidx as c_int >= 0 as c_int {
            } else {
                __assert_fail(
                    b"eap->cmdidx >= 0\0".as_ptr() as *const c_char,
                    b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                    3236 as c_uint,
                    b"char *find_ex_command(exarg_T *, int *)\0".as_ptr() as *const c_char,
                );
            }
        };
        if len == 3 as c_int
            && strncmp(b"def\0".as_ptr() as *const c_char, (*eap).cmd, 3 as size_t) == 0 as c_int
        {
            (*eap).cmdidx = CMD_SIZE;
        }
        while ((*eap).cmdidx as c_int) < CMD_SIZE as c_int {
            if strncmp(
                (*cmdnames.ptr())[(*eap).cmdidx as c_int as usize].cmd_name,
                (*eap).cmd,
                len as size_t,
            ) == 0 as c_int
            {
                if !full.is_null()
                    && *(*cmdnames.ptr())[(*eap).cmdidx as c_int as usize]
                        .cmd_name
                        .offset(len as isize) as c_int
                        == NUL
                {
                    *full = true_0;
                }
                break;
            } else {
                (*eap).cmdidx = ((*eap).cmdidx as c_int + 1 as c_int) as cmdidx_T;
            }
        }
        if (*eap).cmdidx as c_int == CMD_SIZE as c_int
            && *(*eap).cmd as c_int >= 'A' as c_int
            && *(*eap).cmd as c_int <= 'Z' as c_int
        {
            while *p as c_uint >= 'A' as c_uint && *p as c_uint <= 'Z' as c_uint
                || *p as c_uint >= 'a' as c_uint && *p as c_uint <= 'z' as c_uint
                || ascii_isdigit(*p as c_int) as c_int != 0
            {
                p = p.offset(1);
            }
            p = find_ucmd(
                eap,
                p,
                full,
                ::core::ptr::null_mut::<expand_T>(),
                ::core::ptr::null_mut::<c_int>(),
            );
        }
        if p == (*eap).cmd {
            (*eap).cmdidx = CMD_SIZE;
        }
    }
    return p;
}
static cmdmods: GlobalCell<[cmdmod; 24]> = GlobalCell::new([
    cmdmod {
        name: b"aboveleft\0".as_ptr() as *const c_char as *mut c_char,
        minlen: 3 as c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"belowright\0".as_ptr() as *const c_char as *mut c_char,
        minlen: 3 as c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"botright\0".as_ptr() as *const c_char as *mut c_char,
        minlen: 2 as c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"browse\0".as_ptr() as *const c_char as *mut c_char,
        minlen: 3 as c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"confirm\0".as_ptr() as *const c_char as *mut c_char,
        minlen: 4 as c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"filter\0".as_ptr() as *const c_char as *mut c_char,
        minlen: 4 as c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"hide\0".as_ptr() as *const c_char as *mut c_char,
        minlen: 3 as c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"horizontal\0".as_ptr() as *const c_char as *mut c_char,
        minlen: 3 as c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"keepalt\0".as_ptr() as *const c_char as *mut c_char,
        minlen: 5 as c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"keepjumps\0".as_ptr() as *const c_char as *mut c_char,
        minlen: 5 as c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"keepmarks\0".as_ptr() as *const c_char as *mut c_char,
        minlen: 3 as c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"keeppatterns\0".as_ptr() as *const c_char as *mut c_char,
        minlen: 5 as c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"leftabove\0".as_ptr() as *const c_char as *mut c_char,
        minlen: 5 as c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"lockmarks\0".as_ptr() as *const c_char as *mut c_char,
        minlen: 3 as c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"noautocmd\0".as_ptr() as *const c_char as *mut c_char,
        minlen: 3 as c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"noswapfile\0".as_ptr() as *const c_char as *mut c_char,
        minlen: 3 as c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"rightbelow\0".as_ptr() as *const c_char as *mut c_char,
        minlen: 6 as c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"sandbox\0".as_ptr() as *const c_char as *mut c_char,
        minlen: 3 as c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"silent\0".as_ptr() as *const c_char as *mut c_char,
        minlen: 3 as c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"tab\0".as_ptr() as *const c_char as *mut c_char,
        minlen: 3 as c_int,
        has_count: true_0,
    },
    cmdmod {
        name: b"topleft\0".as_ptr() as *const c_char as *mut c_char,
        minlen: 2 as c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"unsilent\0".as_ptr() as *const c_char as *mut c_char,
        minlen: 3 as c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"verbose\0".as_ptr() as *const c_char as *mut c_char,
        minlen: 4 as c_int,
        has_count: true_0,
    },
    cmdmod {
        name: b"vertical\0".as_ptr() as *const c_char as *mut c_char,
        minlen: 4 as c_int,
        has_count: false_0,
    },
]);
pub unsafe extern "C" fn modifier_len(mut cmd: *mut c_char) -> c_int {
    let mut p: *mut c_char = cmd;
    if ascii_isdigit(*cmd as c_int) {
        p = skipwhite(skipdigits(cmd.offset(1 as c_int as isize)));
    }
    let mut i: c_int = 0 as c_int;
    while i < ::core::mem::size_of::<[cmdmod; 24]>()
        .wrapping_div(::core::mem::size_of::<cmdmod>())
        .wrapping_div(
            (::core::mem::size_of::<[cmdmod; 24]>().wrapping_rem(::core::mem::size_of::<cmdmod>())
                == 0) as c_int as usize,
        ) as c_int
    {
        let mut j: c_int = 0;
        j = 0 as c_int;
        while *p.offset(j as isize) as c_int != NUL {
            if *p.offset(j as isize) as c_int
                != *(*cmdmods.ptr())[i as usize].name.offset(j as isize) as c_int
            {
                break;
            }
            j += 1;
        }
        if j >= (*cmdmods.ptr())[i as usize].minlen
            && !(*p.offset(j as isize) as c_uint >= 'A' as c_uint
                && *p.offset(j as isize) as c_uint <= 'Z' as c_uint
                || *p.offset(j as isize) as c_uint >= 'a' as c_uint
                    && *p.offset(j as isize) as c_uint <= 'z' as c_uint)
            && (p == cmd || (*cmdmods.ptr())[i as usize].has_count != 0)
        {
            return j + p.offset_from(cmd) as c_int;
        }
        i += 1;
    }
    return 0 as c_int;
}
pub unsafe extern "C" fn cmd_exists(name: *const c_char) -> c_int {
    let mut i: c_int = 0 as c_int;
    while i < ::core::mem::size_of::<[cmdmod; 24]>()
        .wrapping_div(::core::mem::size_of::<cmdmod>())
        .wrapping_div(
            (::core::mem::size_of::<[cmdmod; 24]>().wrapping_rem(::core::mem::size_of::<cmdmod>())
                == 0) as c_int as usize,
        ) as c_int
    {
        let mut j: c_int = 0;
        j = 0 as c_int;
        while *name.offset(j as isize) as c_int != NUL {
            if *name.offset(j as isize) as c_int
                != *(*cmdmods.ptr())[i as usize].name.offset(j as isize) as c_int
            {
                break;
            }
            j += 1;
        }
        if *name.offset(j as isize) as c_int == NUL && j >= (*cmdmods.ptr())[i as usize].minlen {
            return if *(*cmdmods.ptr())[i as usize].name.offset(j as isize) as c_int == NUL {
                2 as c_int
            } else {
                1 as c_int
            };
        }
        i += 1;
    }
    let mut ea: exarg_T = exarg_T {
        arg: ::core::ptr::null_mut::<c_char>(),
        args: ::core::ptr::null_mut::<*mut c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<c_char>(),
        cmd: ::core::ptr::null_mut::<c_char>(),
        cmdlinep: ::core::ptr::null_mut::<*mut c_char>(),
        cmdline_tofree: ::core::ptr::null_mut::<c_char>(),
        cmdidx: CMD_append,
        argt: 0,
        skip: 0,
        forceit: 0,
        addr_count: 0,
        line1: 0,
        line2: 0,
        addr_type: ADDR_LINES,
        flags: 0,
        do_ecmd_cmd: ::core::ptr::null_mut::<c_char>(),
        do_ecmd_lnum: 0,
        append: 0,
        usefilter: 0,
        amount: 0,
        regname: 0,
        force_bin: 0,
        read_edit: 0,
        mkdir_p: 0,
        force_ff: 0,
        force_enc: 0,
        bad_char: 0,
        useridx: 0,
        errmsg: ::core::ptr::null_mut::<c_char>(),
        ea_getline: None,
        cookie: ::core::ptr::null_mut::<c_void>(),
        cstack: ::core::ptr::null_mut::<cstack_T>(),
    };
    ea.cmd = (if *name as c_int == '2' as c_int || *name as c_int == '3' as c_int {
        name.offset(1 as c_int as isize)
    } else {
        name
    }) as *mut c_char;
    ea.cmdidx = CMD_append;
    ea.flags = 0 as c_int;
    let mut full: c_int = false_0;
    let mut p: *mut c_char = find_ex_command(&raw mut ea, &raw mut full);
    if p.is_null() {
        return 3 as c_int;
    }
    if ascii_isdigit(*name as c_int) as c_int != 0 && ea.cmdidx as c_int != CMD_match as c_int {
        return 0 as c_int;
    }
    if *skipwhite(p) as c_int != NUL {
        return 0 as c_int;
    }
    return if ea.cmdidx as c_int == CMD_SIZE as c_int {
        0 as c_int
    } else if full != 0 {
        2 as c_int
    } else {
        1 as c_int
    };
}
pub unsafe extern "C" fn f_fullcommand(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut name: *mut c_char = tv_get_string(argvars.offset(0 as c_int as isize)) as *mut c_char;
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<c_char>();
    while *name as c_int == ':' as c_int {
        name = name.offset(1);
    }
    name = skip_range(name, ::core::ptr::null_mut::<c_int>());
    let mut ea: exarg_T = exarg_T {
        arg: ::core::ptr::null_mut::<c_char>(),
        args: ::core::ptr::null_mut::<*mut c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<c_char>(),
        cmd: ::core::ptr::null_mut::<c_char>(),
        cmdlinep: ::core::ptr::null_mut::<*mut c_char>(),
        cmdline_tofree: ::core::ptr::null_mut::<c_char>(),
        cmdidx: CMD_append,
        argt: 0,
        skip: 0,
        forceit: 0,
        addr_count: 0,
        line1: 0,
        line2: 0,
        addr_type: ADDR_LINES,
        flags: 0,
        do_ecmd_cmd: ::core::ptr::null_mut::<c_char>(),
        do_ecmd_lnum: 0,
        append: 0,
        usefilter: 0,
        amount: 0,
        regname: 0,
        force_bin: 0,
        read_edit: 0,
        mkdir_p: 0,
        force_ff: 0,
        force_enc: 0,
        bad_char: 0,
        useridx: 0,
        errmsg: ::core::ptr::null_mut::<c_char>(),
        ea_getline: None,
        cookie: ::core::ptr::null_mut::<c_void>(),
        cstack: ::core::ptr::null_mut::<cstack_T>(),
    };
    ea.cmd = if *name as c_int == '2' as c_int || *name as c_int == '3' as c_int {
        name.offset(1 as c_int as isize)
    } else {
        name
    };
    ea.cmdidx = CMD_append;
    ea.flags = 0 as c_int;
    let mut p: *mut c_char = find_ex_command(&raw mut ea, ::core::ptr::null_mut::<c_int>());
    if p.is_null() || ea.cmdidx as c_int == CMD_SIZE as c_int {
        return;
    }
    (*rettv).vval.v_string = xstrdup(if (ea.cmdidx as c_int) < 0 as c_int {
        get_user_command_name(ea.useridx, ea.cmdidx as c_int)
    } else {
        (*cmdnames.ptr())[ea.cmdidx as usize].cmd_name
    });
}
pub unsafe extern "C" fn excmd_get_cmdidx(mut cmd: *const c_char, mut len: size_t) -> cmdidx_T {
    if len == 3 as size_t
        && strncmp(b"def\0".as_ptr() as *const c_char, cmd, 3 as size_t) == 0 as c_int
    {
        return CMD_SIZE;
    }
    let mut idx: cmdidx_T = CMD_append;
    if one_letter_cmd(cmd, &raw mut idx) == 0 {
        idx = CMD_append;
        while (idx as c_int) < CMD_SIZE as c_int {
            if strncmp((*cmdnames.ptr())[idx as c_int as usize].cmd_name, cmd, len) == 0 as c_int {
                break;
            }
            idx = (idx as c_int + 1 as c_int) as cmdidx_T;
        }
    }
    return idx;
}
pub unsafe extern "C" fn excmd_get_argt(mut idx: cmdidx_T) -> uint32_t {
    return (*cmdnames.ptr())[idx as c_int as usize].cmd_argt;
}
pub unsafe extern "C" fn skip_range(mut cmd: *const c_char, mut ctx: *mut c_int) -> *mut c_char {
    while !vim_strchr(
        b" \t0123456789.$%'/?-+,;\\\0".as_ptr() as *const c_char,
        *cmd as uint8_t as c_int,
    )
    .is_null()
    {
        if *cmd as c_int == '\\' as c_int {
            if !(*cmd.offset(1 as c_int as isize) as c_int == '?' as c_int
                || *cmd.offset(1 as c_int as isize) as c_int == '/' as c_int
                || *cmd.offset(1 as c_int as isize) as c_int == '&' as c_int)
            {
                break;
            }
            cmd = cmd.offset(1);
        } else if *cmd as c_int == '\'' as c_int {
            cmd = cmd.offset(1);
            if *cmd as c_int == NUL && !ctx.is_null() {
                *ctx = EXPAND_NOTHING as c_int;
            }
        } else if *cmd as c_int == '/' as c_int || *cmd as c_int == '?' as c_int {
            let c2rust_fresh27 = cmd;
            cmd = cmd.offset(1);
            let mut delim: c_uint = *c2rust_fresh27 as c_uint;
            while *cmd as c_int != NUL && *cmd as c_int != delim as c_char as c_int {
                let c2rust_fresh28 = cmd;
                cmd = cmd.offset(1);
                if *c2rust_fresh28 as c_int == '\\' as c_int && *cmd as c_int != NUL {
                    cmd = cmd.offset(1);
                }
            }
            if *cmd as c_int == NUL && !ctx.is_null() {
                *ctx = EXPAND_NOTHING as c_int;
            }
        }
        if *cmd as c_int != NUL {
            cmd = cmd.offset(1);
        }
    }
    cmd = skip_colon_white(cmd, false_0 != 0);
    if *cmd as c_int == '*' as c_int {
        cmd = skipwhite(cmd.offset(1 as c_int as isize));
    }
    return cmd as *mut c_char;
}
unsafe extern "C" fn addr_error(mut addr_type: cmd_addr_T) -> *const c_char {
    if addr_type as c_uint == ADDR_NONE as c_int as c_uint {
        return gettext(&raw const e_norange as *const c_char);
    } else {
        return gettext(&raw const e_invrange as *const c_char);
    };
}
pub unsafe extern "C" fn get_address(
    mut eap: *mut exarg_T,
    mut ptr: *mut *mut c_char,
    mut addr_type: cmd_addr_T,
    mut skip: bool,
    mut silent: bool,
    mut to_other_file: c_int,
    mut address_count: c_int,
    mut errormsg: *mut *const c_char,
) -> linenr_T {
    let mut c: c_int = 0;
    let mut i: c_int = 0;
    let mut n: linenr_T = 0;
    let mut pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut cmd: *mut c_char = skipwhite(*ptr);
    let mut lnum: linenr_T = MAXLNUM as c_int as linenr_T;
    '_error: loop {
        match *cmd as c_int {
            46 => {
                cmd = cmd.offset(1);
                match addr_type as c_uint {
                    0 | 10 => {
                        lnum = (*curwin.get()).w_cursor.lnum;
                    }
                    1 => {
                        lnum = current_win_nr(curwin.get()) as linenr_T;
                    }
                    2 => {
                        lnum = ((*curwin.get()).w_arg_idx + 1 as c_int) as linenr_T;
                    }
                    3 | 4 => {
                        lnum = (*curbuf.get()).handle as linenr_T;
                    }
                    5 => {
                        lnum = current_tab_nr(curtab.get()) as linenr_T;
                    }
                    11 | 6 | 9 => {
                        *errormsg = addr_error(addr_type);
                        cmd = ::core::ptr::null_mut::<c_char>();
                        break;
                    }
                    8 => {
                        lnum = qf_get_cur_idx(eap) as linenr_T;
                    }
                    7 => {
                        lnum = qf_get_cur_valid_idx(eap) as linenr_T;
                    }
                    _ => {}
                }
            }
            36 => {
                cmd = cmd.offset(1);
                match addr_type as c_uint {
                    0 | 10 => {
                        lnum = (*curbuf.get()).b_ml.ml_line_count;
                    }
                    1 => {
                        lnum = current_win_nr(::core::ptr::null::<win_T>()) as linenr_T;
                    }
                    2 => {
                        lnum = (*(*curwin.get()).w_alist).al_ga.ga_len as linenr_T;
                    }
                    3 => {
                        buf = lastbuf.get();
                        while (*buf).b_ml.ml_mfp.is_null() {
                            if (*buf).b_prev.is_null() {
                                break;
                            }
                            buf = (*buf).b_prev;
                        }
                        lnum = (*buf).handle as linenr_T;
                    }
                    4 => {
                        lnum = (*lastbuf.get()).handle as linenr_T;
                    }
                    5 => {
                        lnum = current_tab_nr(::core::ptr::null_mut::<tabpage_T>()) as linenr_T;
                    }
                    11 | 6 | 9 => {
                        *errormsg = addr_error(addr_type);
                        cmd = ::core::ptr::null_mut::<c_char>();
                        break;
                    }
                    8 => {
                        lnum = qf_get_size(eap) as linenr_T;
                        if lnum == 0 as linenr_T {
                            lnum = 1 as c_int as linenr_T;
                        }
                    }
                    7 => {
                        lnum = qf_get_valid_size(eap) as linenr_T;
                        if lnum == 0 as linenr_T {
                            lnum = 1 as c_int as linenr_T;
                        }
                    }
                    _ => {}
                }
            }
            39 => {
                cmd = cmd.offset(1);
                if *cmd as c_int == NUL {
                    cmd = ::core::ptr::null_mut::<c_char>();
                    break;
                } else if addr_type as c_uint != ADDR_LINES as c_int as c_uint {
                    *errormsg = addr_error(addr_type);
                    cmd = ::core::ptr::null_mut::<c_char>();
                    break;
                } else if skip {
                    cmd = cmd.offset(1);
                } else {
                    let mut flag: MarkGet = (if to_other_file != 0
                        && *cmd.offset(1 as c_int as isize) as c_int == NUL
                    {
                        kMarkAll as c_int
                    } else {
                        kMarkBufLocal as c_int
                    }) as MarkGet;
                    let mut fm: *mut fmark_T = mark_get(
                        curbuf.get(),
                        curwin.get(),
                        ::core::ptr::null_mut::<fmark_T>(),
                        flag,
                        *cmd as c_int,
                    );
                    cmd = cmd.offset(1);
                    if !fm.is_null() && (*fm).fnum != (*curbuf.get()).handle {
                        mark_move_to(fm, 0 as MarkMove);
                        lnum = (*curwin.get()).w_cursor.lnum;
                    } else if !mark_check(fm, errormsg) {
                        cmd = ::core::ptr::null_mut::<c_char>();
                        break;
                    } else {
                        '_c2rust_label: {
                            if !fm.is_null() {
                            } else {
                                __assert_fail(
                                    b"fm != NULL\0".as_ptr() as *const c_char,
                                    b"src/nvim/ex_docmd.rs\0"
                                        .as_ptr() as *const c_char,
                                    3618 as c_uint,
                                    b"linenr_T get_address(exarg_T *, char **, cmd_addr_T, _Bool, _Bool, int, int, const char **)\0"
                                        .as_ptr() as *const c_char,
                                );
                            }
                        };
                        lnum = (*fm).mark.lnum;
                    }
                }
            }
            47 | 63 => {
                let c2rust_fresh2 = cmd;
                cmd = cmd.offset(1);
                c = *c2rust_fresh2 as uint8_t as c_int;
                if addr_type as c_uint != ADDR_LINES as c_int as c_uint {
                    *errormsg = addr_error(addr_type);
                    cmd = ::core::ptr::null_mut::<c_char>();
                    break;
                } else if skip {
                    cmd = skip_regexp(cmd, c, magic_isset() as c_int);
                    if *cmd as c_int == c {
                        cmd = cmd.offset(1);
                    }
                } else {
                    let mut flags: c_int = 0;
                    pos = (*curwin.get()).w_cursor;
                    if lnum > 0 as linenr_T && lnum != MAXLNUM as c_int as linenr_T {
                        (*curwin.get()).w_cursor.lnum = if lnum > (*curbuf.get()).b_ml.ml_line_count
                        {
                            (*curbuf.get()).b_ml.ml_line_count
                        } else {
                            lnum
                        };
                    }
                    (*curwin.get()).w_cursor.col =
                        (if c == '/' as c_int && (*curwin.get()).w_cursor.lnum > 0 as linenr_T {
                            MAXCOL as c_int
                        } else {
                            0 as c_int
                        }) as colnr_T;
                    searchcmdlen.set(0 as c_int);
                    flags = if silent as c_int != 0 {
                        SEARCH_KEEP as c_int
                    } else {
                        SEARCH_HIS as c_int | SEARCH_MSG as c_int
                    };
                    if do_search(
                        ::core::ptr::null_mut::<oparg_T>(),
                        c,
                        c,
                        cmd,
                        strlen(cmd),
                        1 as c_int,
                        flags,
                        ::core::ptr::null_mut::<searchit_arg_T>(),
                    ) == 0
                    {
                        (*curwin.get()).w_cursor = pos;
                        cmd = ::core::ptr::null_mut::<c_char>();
                        break;
                    } else {
                        lnum = (*curwin.get()).w_cursor.lnum;
                        (*curwin.get()).w_cursor = pos;
                        cmd = cmd.offset(searchcmdlen.get() as isize);
                    }
                }
            }
            92 => {
                cmd = cmd.offset(1);
                if addr_type as c_uint != ADDR_LINES as c_int as c_uint {
                    *errormsg = addr_error(addr_type);
                    cmd = ::core::ptr::null_mut::<c_char>();
                    break;
                } else {
                    if *cmd as c_int == '&' as c_int {
                        i = RE_SUBST as c_int;
                    } else if *cmd as c_int == '?' as c_int || *cmd as c_int == '/' as c_int {
                        i = RE_SEARCH as c_int;
                    } else {
                        *errormsg = gettext(&raw const e_backslash as *const c_char);
                        cmd = ::core::ptr::null_mut::<c_char>();
                        break;
                    }
                    if !skip {
                        pos.lnum = if lnum != MAXLNUM as c_int as linenr_T {
                            lnum
                        } else {
                            (*curwin.get()).w_cursor.lnum
                        };
                        pos.col = (if *cmd as c_int != '?' as c_int {
                            MAXCOL as c_int
                        } else {
                            0 as c_int
                        }) as colnr_T;
                        pos.coladd = 0 as c_int as colnr_T;
                        if searchit(
                            curwin.get(),
                            curbuf.get(),
                            &raw mut pos,
                            ::core::ptr::null_mut::<pos_T>(),
                            (if *cmd as c_int == '?' as c_int {
                                BACKWARD as c_int
                            } else {
                                FORWARD as c_int
                            }) as Direction,
                            b"\0".as_ptr() as *const c_char as *mut c_char,
                            0 as size_t,
                            1 as c_int,
                            SEARCH_MSG as c_int,
                            i,
                            ::core::ptr::null_mut::<searchit_arg_T>(),
                        ) != FAIL
                        {
                            lnum = pos.lnum;
                        } else {
                            cmd = ::core::ptr::null_mut::<c_char>();
                            break;
                        }
                    }
                    cmd = cmd.offset(1);
                }
            }
            _ => {
                if ascii_isdigit(*cmd as c_int) {
                    lnum = getdigits(&raw mut cmd, false_0 != 0, 0 as intmax_t) as linenr_T;
                }
            }
        }
        loop {
            cmd = skipwhite(cmd);
            if *cmd as c_int != '-' as c_int
                && *cmd as c_int != '+' as c_int
                && !ascii_isdigit(*cmd as c_int)
            {
                break;
            }
            if lnum == MAXLNUM as c_int as linenr_T {
                match addr_type as c_uint {
                    0 | 10 => {
                        lnum = (*curwin.get()).w_cursor.lnum;
                    }
                    1 => {
                        lnum = current_win_nr(curwin.get()) as linenr_T;
                    }
                    2 => {
                        lnum = ((*curwin.get()).w_arg_idx + 1 as c_int) as linenr_T;
                    }
                    3 | 4 => {
                        lnum = (*curbuf.get()).handle as linenr_T;
                    }
                    5 => {
                        lnum = current_tab_nr(curtab.get()) as linenr_T;
                    }
                    6 => {
                        lnum = 1 as c_int as linenr_T;
                    }
                    8 => {
                        lnum = qf_get_cur_idx(eap) as linenr_T;
                    }
                    7 => {
                        lnum = qf_get_cur_valid_idx(eap) as linenr_T;
                    }
                    11 | 9 => {
                        lnum = 0 as c_int as linenr_T;
                    }
                    _ => {}
                }
            }
            if ascii_isdigit(*cmd as c_int) {
                i = '+' as c_int;
            } else {
                let c2rust_fresh3 = cmd;
                cmd = cmd.offset(1);
                i = *c2rust_fresh3 as uint8_t as c_int;
            }
            if !ascii_isdigit(*cmd as c_int) {
                n = 1 as c_int as linenr_T;
            } else {
                n = getdigits_int32(&raw mut cmd, false_0 != 0, MAXLNUM as c_int as int32_t)
                    as linenr_T;
                if n == MAXLNUM as c_int as linenr_T {
                    *errormsg = gettext(&raw const e_line_number_out_of_range as *const c_char);
                    cmd = ::core::ptr::null_mut::<c_char>();
                    break '_error;
                }
            }
            if addr_type as c_uint == ADDR_TABS_RELATIVE as c_int as c_uint {
                *errormsg = gettext(&raw const e_invrange as *const c_char);
                cmd = ::core::ptr::null_mut::<c_char>();
                break '_error;
            } else if addr_type as c_uint == ADDR_LOADED_BUFFERS as c_int as c_uint
                || addr_type as c_uint == ADDR_BUFFERS as c_int as c_uint
            {
                lnum = compute_buffer_local_count(
                    addr_type,
                    lnum,
                    if i == '-' as c_int {
                        -1 as c_int * n as c_int
                    } else {
                        n as c_int
                    },
                ) as linenr_T;
            } else {
                if addr_type as c_uint == ADDR_LINES as c_int as c_uint
                    && (i == '-' as c_int || i == '+' as c_int)
                    && address_count >= 2 as c_int
                {
                    hasFolding(
                        curwin.get(),
                        lnum,
                        ::core::ptr::null_mut::<linenr_T>(),
                        &raw mut lnum,
                    );
                }
                if i == '-' as c_int {
                    lnum -= n;
                } else if lnum >= 0 as linenr_T && n >= INT32_MAX as linenr_T - lnum {
                    *errormsg = gettext(&raw const e_line_number_out_of_range as *const c_char);
                    cmd = ::core::ptr::null_mut::<c_char>();
                    break '_error;
                } else {
                    lnum += n;
                }
            }
        }
        if !(*cmd as c_int == '/' as c_int || *cmd as c_int == '?' as c_int) {
            break;
        }
    }
    *ptr = cmd;
    return lnum;
}
unsafe extern "C" fn get_flags(mut eap: *mut exarg_T) {
    while !vim_strchr(
        b"lp#\0".as_ptr() as *const c_char,
        *(*eap).arg as uint8_t as c_int,
    )
    .is_null()
    {
        if *(*eap).arg as c_int == 'l' as c_int {
            (*eap).flags |= EXFLAG_LIST;
        } else if *(*eap).arg as c_int == 'p' as c_int {
            (*eap).flags |= EXFLAG_PRINT;
        } else {
            (*eap).flags |= EXFLAG_NR;
        }
        (*eap).arg = skipwhite((*eap).arg.offset(1 as c_int as isize));
    }
}
pub unsafe extern "C" fn ex_ni(mut eap: *mut exarg_T) {
    if (*eap).skip == 0 {
        (*eap).errmsg = gettext(
            b"E319: The command is not available in this version\0".as_ptr() as *const c_char,
        );
    }
}
unsafe extern "C" fn ex_script_ni(mut eap: *mut exarg_T) {
    if (*eap).skip == 0 {
        ex_ni(eap);
    } else {
        let mut len: size_t = 0;
        xfree(script_get(eap, &raw mut len) as *mut c_void);
    };
}
pub unsafe extern "C" fn invalid_range(mut eap: *mut exarg_T) -> *mut c_char {
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    if (*eap).line1 < 0 as linenr_T || (*eap).line2 < 0 as linenr_T || (*eap).line1 > (*eap).line2 {
        return gettext(&raw const e_invrange as *const c_char);
    }
    if (*eap).argt & EX_RANGE as uint32_t != 0 {
        match (*eap).addr_type as c_uint {
            0 => {
                if (*eap).line2
                    > (*curbuf.get()).b_ml.ml_line_count
                        + ((*eap).cmdidx as c_int == CMD_diffget as c_int
                            || (*eap).cmdidx as c_int == CMD_diffput as c_int)
                            as c_int
                {
                    return gettext(&raw const e_invrange as *const c_char);
                }
            }
            2 => {
                if (*eap).line2
                    > (*(*curwin.get()).w_alist).al_ga.ga_len as linenr_T
                        + ((*(*curwin.get()).w_alist).al_ga.ga_len == 0) as c_int
                {
                    return gettext(&raw const e_invrange as *const c_char);
                }
            }
            4 => {
                if (*eap).line1 < 1 as linenr_T || (*eap).line2 > get_highest_fnum() as linenr_T {
                    return gettext(&raw const e_invrange as *const c_char);
                }
            }
            3 => {
                buf = firstbuf.get();
                while (*buf).b_ml.ml_mfp.is_null() {
                    if (*buf).b_next.is_null() {
                        return gettext(&raw const e_invrange as *const c_char);
                    }
                    buf = (*buf).b_next;
                }
                if (*eap).line1 < (*buf).handle as linenr_T {
                    return gettext(&raw const e_invrange as *const c_char);
                }
                buf = lastbuf.get();
                while (*buf).b_ml.ml_mfp.is_null() {
                    if (*buf).b_prev.is_null() {
                        return gettext(&raw const e_invrange as *const c_char);
                    }
                    buf = (*buf).b_prev;
                }
                if (*eap).line2 > (*buf).handle as linenr_T {
                    return gettext(&raw const e_invrange as *const c_char);
                }
            }
            1 => {
                if (*eap).line2 > current_win_nr(::core::ptr::null::<win_T>()) as linenr_T {
                    return gettext(&raw const e_invrange as *const c_char);
                }
            }
            5 => {
                if (*eap).line2 > current_tab_nr(::core::ptr::null_mut::<tabpage_T>()) as linenr_T {
                    return gettext(&raw const e_invrange as *const c_char);
                }
            }
            8 => {
                '_c2rust_label: {
                    if (*eap).line2 >= 0 as linenr_T {
                    } else {
                        __assert_fail(
                            b"eap->line2 >= 0\0".as_ptr() as *const c_char,
                            b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                            3906 as c_uint,
                            b"char *invalid_range(exarg_T *)\0".as_ptr() as *const c_char,
                        );
                    }
                };
                if (*eap).line2 <= 0 as linenr_T {
                    if (*eap).addr_count == 0 as c_int {
                        return gettext(&raw const e_no_errors as *const c_char);
                    }
                    return gettext(&raw const e_invrange as *const c_char);
                }
            }
            7 => {
                if (*eap).line2 != 1 as linenr_T && (*eap).line2 as size_t > qf_get_valid_size(eap)
                    || (*eap).line2 < 0 as linenr_T
                {
                    return gettext(&raw const e_invrange as *const c_char);
                }
            }
            6 | 10 | 9 | 11 | _ => {}
        }
    }
    return ::core::ptr::null_mut::<c_char>();
}
unsafe extern "C" fn correct_range(mut eap: *mut exarg_T) {
    if (*eap).argt & EX_ZEROR as uint32_t == 0 {
        if (*eap).line1 == 0 as linenr_T {
            (*eap).line1 = 1 as c_int as linenr_T;
        }
        if (*eap).line2 == 0 as linenr_T {
            (*eap).line2 = 1 as c_int as linenr_T;
        }
    }
}
unsafe extern "C" fn skip_grep_pat(mut eap: *mut exarg_T) -> *mut c_char {
    let mut p: *mut c_char = (*eap).arg;
    if *p as c_int != NUL
        && ((*eap).cmdidx as c_int == CMD_vimgrep as c_int
            || (*eap).cmdidx as c_int == CMD_lvimgrep as c_int
            || (*eap).cmdidx as c_int == CMD_vimgrepadd as c_int
            || (*eap).cmdidx as c_int == CMD_lvimgrepadd as c_int
            || grep_internal((*eap).cmdidx) != 0)
    {
        p = skip_vimgrep_pat(
            p,
            ::core::ptr::null_mut::<*mut c_char>(),
            ::core::ptr::null_mut::<c_int>(),
        );
        if p.is_null() {
            p = (*eap).arg;
        }
    }
    return p;
}
pub unsafe extern "C" fn replace_makeprg(
    mut eap: *mut exarg_T,
    mut arg: *mut c_char,
    mut cmdlinep: *mut *mut c_char,
) -> *mut c_char {
    let mut isgrep: bool = (*eap).cmdidx as c_int == CMD_grep as c_int
        || (*eap).cmdidx as c_int == CMD_lgrep as c_int
        || (*eap).cmdidx as c_int == CMD_grepadd as c_int
        || (*eap).cmdidx as c_int == CMD_lgrepadd as c_int;
    if ((*eap).cmdidx as c_int == CMD_make as c_int
        || (*eap).cmdidx as c_int == CMD_lmake as c_int
        || isgrep as c_int != 0)
        && grep_internal((*eap).cmdidx) == 0
    {
        let mut program: *const c_char = if isgrep as c_int != 0 {
            if *(*curbuf.get()).b_p_gp as c_int == NUL {
                p_gp.get()
            } else {
                (*curbuf.get()).b_p_gp
            }
        } else if *(*curbuf.get()).b_p_mp as c_int == NUL {
            p_mp.get()
        } else {
            (*curbuf.get()).b_p_mp
        };
        arg = skipwhite(arg);
        let mut new_cmdline: *mut c_char = ::core::ptr::null_mut::<c_char>();
        new_cmdline = strrep(program, b"$*\0".as_ptr() as *const c_char, arg);
        if new_cmdline.is_null() {
            new_cmdline = xmalloc(
                strlen(program)
                    .wrapping_add(strlen(arg))
                    .wrapping_add(2 as size_t),
            ) as *mut c_char;
            strcpy(new_cmdline, program as *mut c_char);
            strcat(new_cmdline, b" \0".as_ptr() as *const c_char);
            strcat(new_cmdline, arg);
        }
        msg_make(arg);
        xfree(*cmdlinep as *mut c_void);
        *cmdlinep = new_cmdline;
        arg = new_cmdline;
    }
    return arg;
}
pub unsafe extern "C" fn expand_filename(
    mut eap: *mut exarg_T,
    mut cmdlinep: *mut *mut c_char,
    mut errormsgp: *mut *const c_char,
) -> c_int {
    let mut p: *mut c_char = skip_grep_pat(eap);
    let mut has_wildcards: bool = path_has_wildcard(p);
    while *p as c_int != NUL {
        if *p.offset(0 as c_int as isize) as c_int == '`' as c_int
            && *p.offset(1 as c_int as isize) as c_int == '=' as c_int
        {
            p = p.offset(2 as c_int as isize);
            skip_expr(&raw mut p, ::core::ptr::null_mut::<evalarg_T>());
            if *p as c_int == '`' as c_int {
                p = p.offset(1);
            }
        } else if vim_strchr(b"%#<\0".as_ptr() as *const c_char, *p as uint8_t as c_int).is_null() {
            p = p.offset(1);
        } else {
            let mut srclen: size_t = 0;
            let mut escaped: c_int = 0;
            let mut repl: *mut c_char = eval_vars(
                p,
                (*eap).arg,
                &raw mut srclen,
                &raw mut (*eap).do_ecmd_lnum,
                errormsgp,
                &raw mut escaped,
                true_0 != 0,
            );
            if !(*errormsgp).is_null() {
                return FAIL;
            }
            if repl.is_null() {
                p = p.offset(srclen as isize);
            } else {
                if !vim_strchr(repl, '$' as c_int).is_null()
                    || !vim_strchr(repl, '~' as c_int).is_null()
                {
                    let mut l: *mut c_char = repl;
                    repl = expand_env_save(repl);
                    xfree(l as *mut c_void);
                }
                if (*eap).usefilter == 0
                    && escaped == 0
                    && (*eap).cmdidx as c_int != CMD_bang as c_int
                    && (*eap).cmdidx as c_int != CMD_grep as c_int
                    && (*eap).cmdidx as c_int != CMD_grepadd as c_int
                    && (*eap).cmdidx as c_int != CMD_lgrep as c_int
                    && (*eap).cmdidx as c_int != CMD_lgrepadd as c_int
                    && (*eap).cmdidx as c_int != CMD_lmake as c_int
                    && (*eap).cmdidx as c_int != CMD_make as c_int
                    && (*eap).cmdidx as c_int != CMD_terminal as c_int
                    && (*eap).argt & EX_NOSPC as uint32_t == 0
                {
                    let mut l_0: *mut c_char = ::core::ptr::null_mut::<c_char>();
                    l_0 = repl;
                    while *l_0 != 0 {
                        if !vim_strchr(escape_chars.get(), *l_0 as uint8_t as c_int).is_null() {
                            l_0 = vim_strsave_escaped(repl, escape_chars.get());
                            xfree(repl as *mut c_void);
                            repl = l_0;
                            break;
                        } else {
                            l_0 = l_0.offset(1);
                        }
                    }
                }
                if ((*eap).usefilter != 0
                    || (*eap).cmdidx as c_int == CMD_bang as c_int
                    || (*eap).cmdidx as c_int == CMD_terminal as c_int)
                    && !strpbrk(repl, b"!\0".as_ptr() as *const c_char).is_null()
                {
                    let mut l_1: *mut c_char =
                        vim_strsave_escaped(repl, b"!\0".as_ptr() as *const c_char);
                    xfree(repl as *mut c_void);
                    repl = l_1;
                }
                p = repl_cmdline(eap, p, srclen, repl, cmdlinep);
                xfree(repl as *mut c_void);
            }
        }
    }
    if (*eap).argt & EX_NOSPC as uint32_t != 0 && (*eap).usefilter == 0 {
        if has_wildcards {
            if !vim_strchr((*eap).arg, '$' as c_int).is_null()
                || !vim_strchr((*eap).arg, '~' as c_int).is_null()
            {
                expand_env_esc(
                    (*eap).arg,
                    NameBuff.ptr() as *mut c_char,
                    MAXPATHL,
                    true_0 != 0,
                    true_0 != 0,
                    ::core::ptr::null_mut::<c_char>(),
                );
                has_wildcards = path_has_wildcard(NameBuff.ptr() as *mut c_char);
                p = NameBuff.ptr() as *mut c_char;
            } else {
                p = ::core::ptr::null_mut::<c_char>();
            }
            if !p.is_null() {
                repl_cmdline(eap, (*eap).arg, strlen((*eap).arg), p, cmdlinep);
            }
        }
        if !has_wildcards {
            backslash_halve((*eap).arg);
        }
        if has_wildcards {
            let mut xpc: expand_T = expand_T {
                xp_pattern: ::core::ptr::null_mut::<c_char>(),
                xp_context: 0,
                xp_pattern_len: 0,
                xp_prefix: XP_PREFIX_NONE,
                xp_arg: ::core::ptr::null_mut::<c_char>(),
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
                xp_orig: ::core::ptr::null_mut::<c_char>(),
                xp_files: ::core::ptr::null_mut::<*mut c_char>(),
                xp_line: ::core::ptr::null_mut::<c_char>(),
                xp_buf: [0; 256],
                xp_search_dir: kDirectionNotSet,
                xp_pre_incsearch_pos: pos_T {
                    lnum: 0,
                    col: 0,
                    coladd: 0,
                },
            };
            let mut options: c_int =
                WILD_LIST_NOTFOUND as c_int | WILD_NOERROR as c_int | WILD_ADD_SLASH as c_int;
            ExpandInit(&raw mut xpc);
            xpc.xp_context = EXPAND_FILES as c_int;
            if p_wic.get() != 0 {
                options += WILD_ICASE as c_int;
            }
            p = ExpandOne(
                &raw mut xpc,
                (*eap).arg,
                ::core::ptr::null_mut::<c_char>(),
                options,
                WILD_EXPAND_FREE as c_int,
            );
            if p.is_null() {
                return FAIL;
            }
            repl_cmdline(eap, (*eap).arg, strlen((*eap).arg), p, cmdlinep);
            xfree(p as *mut c_void);
        }
    }
    return OK;
}
unsafe extern "C" fn repl_cmdline(
    mut eap: *mut exarg_T,
    mut src: *mut c_char,
    mut srclen: size_t,
    mut repl: *mut c_char,
    mut cmdlinep: *mut *mut c_char,
) -> *mut c_char {
    let mut len: size_t = strlen(repl);
    let mut i: size_t = (src.offset_from(*cmdlinep) as size_t)
        .wrapping_add(strlen(src.offset(srclen as isize)))
        .wrapping_add(len)
        .wrapping_add(3 as size_t);
    if !(*eap).nextcmd.is_null() {
        i = i.wrapping_add(strlen((*eap).nextcmd));
    }
    let mut new_cmdline: *mut c_char = xmalloc(i) as *mut c_char;
    let mut offset: size_t = src.offset_from(*cmdlinep) as size_t;
    i = offset;
    memmove(new_cmdline as *mut c_void, *cmdlinep as *const c_void, i);
    memmove(
        new_cmdline.offset(i as isize) as *mut c_void,
        repl as *const c_void,
        len,
    );
    i = i.wrapping_add(len);
    strcpy(new_cmdline.offset(i as isize), src.offset(srclen as isize));
    src = new_cmdline.offset(i as isize);
    if !(*eap).nextcmd.is_null() {
        i = strlen(new_cmdline).wrapping_add(1 as size_t);
        strcpy(new_cmdline.offset(i as isize), (*eap).nextcmd);
        (*eap).nextcmd = new_cmdline.offset(i as isize);
    }
    (*eap).cmd = new_cmdline.offset((*eap).cmd.offset_from(*cmdlinep) as isize);
    (*eap).arg = new_cmdline.offset((*eap).arg.offset_from(*cmdlinep) as isize);
    let mut j: size_t = 0 as size_t;
    while j < (*eap).argc {
        if offset >= (*(*eap).args.offset(j as isize)).offset_from(*cmdlinep) as size_t {
            *(*eap).args.offset(j as isize) = new_cmdline
                .offset((*(*eap).args.offset(j as isize)).offset_from(*cmdlinep) as isize);
        } else {
            *(*eap).args.offset(j as isize) = new_cmdline.offset(
                ((*(*eap).args.offset(j as isize)).offset_from(*cmdlinep)
                    + len.wrapping_sub(srclen) as isize) as isize,
            );
        }
        j = j.wrapping_add(1);
    }
    if !(*eap).do_ecmd_cmd.is_null() && (*eap).do_ecmd_cmd != dollar_command.ptr() as *mut c_char {
        (*eap).do_ecmd_cmd = new_cmdline.offset((*eap).do_ecmd_cmd.offset_from(*cmdlinep) as isize);
    }
    xfree(*cmdlinep as *mut c_void);
    *cmdlinep = new_cmdline;
    return src;
}
pub unsafe extern "C" fn separate_nextcmd(mut eap: *mut exarg_T) {
    let mut p: *mut c_char = skip_grep_pat(eap);
    while *p != 0 {
        if *p as c_int == Ctrl_V {
            if (*eap).argt & (EX_CTRLV as uint32_t | EX_XFILE as uint32_t) != 0 {
                p = p.offset(1);
            } else {
                memmove(
                    p as *mut c_void,
                    p.offset(1 as c_int as isize) as *const c_void,
                    strlen(p.offset(1 as c_int as isize)).wrapping_add(1 as size_t),
                );
            }
            if *p as c_int == NUL {
                break;
            }
        } else if *p.offset(0 as c_int as isize) as c_int == '`' as c_int
            && *p.offset(1 as c_int as isize) as c_int == '=' as c_int
            && (*eap).argt & EX_XFILE as uint32_t != 0
        {
            p = p.offset(2 as c_int as isize);
            skip_expr(&raw mut p, ::core::ptr::null_mut::<evalarg_T>());
            if *p as c_int == NUL {
                break;
            }
        } else if *p as c_int == '"' as c_int
            && (*eap).argt & EX_NOTRLCOM as uint32_t == 0
            && ((*eap).cmdidx as c_int != CMD_at as c_int || p != (*eap).arg)
            && ((*eap).cmdidx as c_int != CMD_redir as c_int
                || p != (*eap).arg.offset(1 as c_int as isize)
                || *p.offset(-1 as c_int as isize) as c_int != '@' as c_int)
            || *p as c_int == '|' as c_int
                && (*eap).cmdidx as c_int != CMD_append as c_int
                && (*eap).cmdidx as c_int != CMD_change as c_int
                && (*eap).cmdidx as c_int != CMD_insert as c_int
            || *p as c_int == '\n' as c_int
        {
            if (vim_strchr(p_cpo.get(), CPO_BAR).is_null()
                || (*eap).argt & EX_CTRLV as uint32_t == 0)
                && *p.offset(-(1 as c_int as isize)) as c_int == '\\' as c_int
            {
                memmove(
                    p.offset(-(1 as c_int as isize)) as *mut c_void,
                    p as *const c_void,
                    strlen(p).wrapping_add(1 as size_t),
                );
                p = p.offset(-1);
            } else {
                (*eap).nextcmd = check_nextcmd(p);
                *p = NUL as c_char;
                break;
            }
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    if (*eap).argt & EX_NOTRLCOM as uint32_t == 0 {
        del_trailing_spaces((*eap).arg);
    }
}
pub unsafe extern "C" fn getargcmd(mut argp: *mut *mut c_char) -> *mut c_char {
    let mut arg: *mut c_char = *argp;
    let mut command: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if *arg as c_int == '+' as c_int {
        arg = arg.offset(1);
        if ascii_isspace(*arg as c_int) as c_int != 0 || *arg as c_int == NUL {
            command = dollar_command.ptr() as *mut c_char;
        } else {
            command = arg;
            arg = skip_cmd_arg(command, true_0 != 0);
            if *arg as c_int != NUL {
                let c2rust_fresh26 = arg;
                arg = arg.offset(1);
                *c2rust_fresh26 = NUL as c_char;
            }
        }
        arg = skipwhite(arg);
        *argp = arg;
    }
    return command;
}
pub unsafe extern "C" fn skip_cmd_arg(mut p: *mut c_char, mut rembs: bool) -> *mut c_char {
    while *p as c_int != 0 && !ascii_isspace(*p as c_int) {
        if *p as c_int == '\\' as c_int && *p.offset(1 as c_int as isize) as c_int != NUL {
            if rembs {
                memmove(
                    p as *mut c_void,
                    p.offset(1 as c_int as isize) as *const c_void,
                    strlen(p.offset(1 as c_int as isize)).wrapping_add(1 as size_t),
                );
            } else {
                p = p.offset(1);
            }
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    return p;
}
pub unsafe extern "C" fn get_bad_opt(mut p: *const c_char, mut eap: *mut exarg_T) -> c_int {
    if strcasecmp(
        p as *mut c_char,
        b"keep\0".as_ptr() as *const c_char as *mut c_char,
    ) == 0 as c_int
    {
        (*eap).bad_char = BAD_KEEP;
    } else if strcasecmp(
        p as *mut c_char,
        b"drop\0".as_ptr() as *const c_char as *mut c_char,
    ) == 0 as c_int
    {
        (*eap).bad_char = BAD_DROP;
    } else if (*utf8len_tab.ptr())[*p as uint8_t as usize] as c_int == 1 as c_int
        && *p.offset(1 as c_int as isize) as c_int == NUL
    {
        (*eap).bad_char = *p as uint8_t as c_int;
    } else {
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn get_bad_name(mut _xp: *mut expand_T, mut idx: c_int) -> *mut c_char {
    static p_bad_values: GlobalCell<[*mut c_char; 3]> = GlobalCell::new([
        b"?\0".as_ptr() as *const c_char as *mut c_char,
        b"keep\0".as_ptr() as *const c_char as *mut c_char,
        b"drop\0".as_ptr() as *const c_char as *mut c_char,
    ]);
    if idx
        < ::core::mem::size_of::<[*mut c_char; 3]>()
            .wrapping_div(::core::mem::size_of::<*mut c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*mut c_char; 3]>()
                    .wrapping_rem(::core::mem::size_of::<*mut c_char>())
                    == 0) as c_int as usize,
            ) as c_int
    {
        return (*p_bad_values.ptr())[idx as usize] as *mut c_char;
    }
    return ::core::ptr::null_mut::<c_char>();
}
pub unsafe extern "C" fn getargopt(mut eap: *mut exarg_T) -> c_int {
    let mut arg: *mut c_char = (*eap).arg.offset(2 as c_int as isize);
    let mut pp: *mut c_int = ::core::ptr::null_mut::<c_int>();
    let mut bad_char_idx: c_int = 0;
    if strncmp(arg, b"bin\0".as_ptr() as *const c_char, 3 as size_t) == 0 as c_int
        || strncmp(arg, b"nobin\0".as_ptr() as *const c_char, 5 as size_t) == 0 as c_int
    {
        if *arg as c_int == 'n' as c_int {
            arg = arg.offset(2 as c_int as isize);
            (*eap).force_bin = FORCE_NOBIN;
        } else {
            (*eap).force_bin = FORCE_BIN;
        }
        if !checkforcmd(
            &raw mut arg,
            b"binary\0".as_ptr() as *const c_char,
            3 as c_int,
        ) {
            return FAIL;
        }
        (*eap).arg = skipwhite(arg);
        return OK;
    }
    if strncmp(arg, b"edit\0".as_ptr() as *const c_char, 4 as size_t) == 0 as c_int
        && !(*arg.offset(4 as c_int as isize) as c_uint >= 'A' as c_uint
            && *arg.offset(4 as c_int as isize) as c_uint <= 'Z' as c_uint
            || *arg.offset(4 as c_int as isize) as c_uint >= 'a' as c_uint
                && *arg.offset(4 as c_int as isize) as c_uint <= 'z' as c_uint)
    {
        (*eap).read_edit = true_0;
        (*eap).arg = skipwhite(arg.offset(4 as c_int as isize));
        return OK;
    }
    if *arg.offset(0 as c_int as isize) as c_int == 'p' as c_int
        && !(*arg.offset(1 as c_int as isize) as c_uint >= 'A' as c_uint
            && *arg.offset(1 as c_int as isize) as c_uint <= 'Z' as c_uint
            || *arg.offset(1 as c_int as isize) as c_uint >= 'a' as c_uint
                && *arg.offset(1 as c_int as isize) as c_uint <= 'z' as c_uint)
    {
        (*eap).mkdir_p = true_0;
        (*eap).arg = skipwhite(arg.offset(1 as c_int as isize));
        return OK;
    }
    if strncmp(arg, b"ff\0".as_ptr() as *const c_char, 2 as size_t) == 0 as c_int {
        arg = arg.offset(2 as c_int as isize);
        pp = &raw mut (*eap).force_ff;
    } else if strncmp(arg, b"fileformat\0".as_ptr() as *const c_char, 10 as size_t) == 0 as c_int {
        arg = arg.offset(10 as c_int as isize);
        pp = &raw mut (*eap).force_ff;
    } else if strncmp(arg, b"enc\0".as_ptr() as *const c_char, 3 as size_t) == 0 as c_int {
        if strncmp(arg, b"encoding\0".as_ptr() as *const c_char, 8 as size_t) == 0 as c_int {
            arg = arg.offset(8 as c_int as isize);
        } else {
            arg = arg.offset(3 as c_int as isize);
        }
        pp = &raw mut (*eap).force_enc;
    } else if strncmp(arg, b"bad\0".as_ptr() as *const c_char, 3 as size_t) == 0 as c_int {
        arg = arg.offset(3 as c_int as isize);
        pp = &raw mut bad_char_idx;
    }
    if pp.is_null() || *arg as c_int != '=' as c_int {
        return FAIL;
    }
    arg = arg.offset(1);
    *pp = arg.offset_from((*eap).cmd) as c_int;
    arg = skip_cmd_arg(arg, false_0 != 0);
    (*eap).arg = skipwhite(arg);
    *arg = NUL as c_char;
    if pp == &raw mut (*eap).force_ff {
        if check_ff_value((*eap).cmd.offset((*eap).force_ff as isize)) == FAIL {
            return FAIL;
        }
        (*eap).force_ff = *(*eap).cmd.offset((*eap).force_ff as isize) as uint8_t as c_int;
    } else if pp == &raw mut (*eap).force_enc {
        let mut p: *mut c_char = (*eap).cmd.offset((*eap).force_enc as isize);
        while *p as c_int != NUL {
            *p = (if (*p as c_int) < 'A' as c_int || *p as c_int > 'Z' as c_int {
                *p as c_int
            } else {
                *p as c_int + ('a' as c_int - 'A' as c_int)
            }) as c_char;
            p = p.offset(1);
        }
    } else if get_bad_opt((*eap).cmd.offset(bad_char_idx as isize), eap) == FAIL {
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn get_argopt_name(mut _xp: *mut expand_T, mut idx: c_int) -> *mut c_char {
    static p_opt_values: GlobalCell<[*mut c_char; 7]> = GlobalCell::new([
        b"fileformat=\0".as_ptr() as *const c_char as *mut c_char,
        b"encoding=\0".as_ptr() as *const c_char as *mut c_char,
        b"binary\0".as_ptr() as *const c_char as *mut c_char,
        b"nobinary\0".as_ptr() as *const c_char as *mut c_char,
        b"bad=\0".as_ptr() as *const c_char as *mut c_char,
        b"edit\0".as_ptr() as *const c_char as *mut c_char,
        b"p\0".as_ptr() as *const c_char as *mut c_char,
    ]);
    if idx
        < ::core::mem::size_of::<[*mut c_char; 7]>()
            .wrapping_div(::core::mem::size_of::<*mut c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*mut c_char; 7]>()
                    .wrapping_rem(::core::mem::size_of::<*mut c_char>())
                    == 0) as c_int as usize,
            ) as c_int
    {
        return (*p_opt_values.ptr())[idx as usize] as *mut c_char;
    }
    return ::core::ptr::null_mut::<c_char>();
}
pub unsafe extern "C" fn expand_argopt(
    mut pat: *mut c_char,
    mut xp: *mut expand_T,
    mut rmp: *mut regmatch_T,
    mut matches: *mut *mut *mut c_char,
    mut numMatches: *mut c_int,
) -> c_int {
    if (*xp).xp_pattern > (*xp).xp_line
        && *(*xp).xp_pattern.offset(-(1 as c_int as isize)) as c_int == '=' as c_int
    {
        let mut cb: CompleteListItemGetter = None;
        let mut name_end: *mut c_char = (*xp).xp_pattern.offset(-(1 as c_int as isize));
        if name_end.offset_from((*xp).xp_line) >= 2 as isize
            && strncmp(
                name_end.offset(-(2 as c_int as isize)),
                b"ff\0".as_ptr() as *const c_char,
                2 as size_t,
            ) == 0 as c_int
        {
            cb = Some(
                get_fileformat_name as unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char,
            ) as CompleteListItemGetter;
        } else if name_end.offset_from((*xp).xp_line) >= 10 as isize
            && strncmp(
                name_end.offset(-(10 as c_int as isize)),
                b"fileformat\0".as_ptr() as *const c_char,
                10 as size_t,
            ) == 0 as c_int
        {
            cb = Some(
                get_fileformat_name as unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char,
            ) as CompleteListItemGetter;
        } else if name_end.offset_from((*xp).xp_line) >= 3 as isize
            && strncmp(
                name_end.offset(-(3 as c_int as isize)),
                b"enc\0".as_ptr() as *const c_char,
                3 as size_t,
            ) == 0 as c_int
        {
            cb = Some(
                get_encoding_name as unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char,
            ) as CompleteListItemGetter;
        } else if name_end.offset_from((*xp).xp_line) >= 8 as isize
            && strncmp(
                name_end.offset(-(8 as c_int as isize)),
                b"encoding\0".as_ptr() as *const c_char,
                8 as size_t,
            ) == 0 as c_int
        {
            cb = Some(
                get_encoding_name as unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char,
            ) as CompleteListItemGetter;
        } else if name_end.offset_from((*xp).xp_line) >= 3 as isize
            && strncmp(
                name_end.offset(-(3 as c_int as isize)),
                b"bad\0".as_ptr() as *const c_char,
                3 as size_t,
            ) == 0 as c_int
        {
            cb = Some(get_bad_name as unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char)
                as CompleteListItemGetter;
        }
        if cb.is_some() {
            ExpandGeneric(pat, xp, rmp, matches, numMatches, cb, false_0 != 0);
            return OK;
        }
        return FAIL;
    }
    if (*xp).xp_pattern_len == 2 as size_t
        && strncmp(
            (*xp).xp_pattern,
            b"ff\0".as_ptr() as *const c_char,
            (*xp).xp_pattern_len,
        ) == 0 as c_int
    {
        *matches = xmalloc(::core::mem::size_of::<*mut c_char>()) as *mut *mut c_char;
        *numMatches = 1 as c_int;
        *(*matches).offset(0 as c_int as isize) =
            xstrdup(b"fileformat=\0".as_ptr() as *const c_char);
        return OK;
    }
    ExpandGeneric(
        pat,
        xp,
        rmp,
        matches,
        numMatches,
        Some(get_argopt_name as unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char),
        false_0 != 0,
    );
    return OK;
}
unsafe extern "C" fn get_tabpage_arg(mut eap: *mut exarg_T) -> c_int {
    let mut tab_number: c_int = 0 as c_int;
    let mut unaccept_arg0: c_int = if (*eap).cmdidx as c_int == CMD_tabmove as c_int {
        0 as c_int
    } else {
        1 as c_int
    };
    '_theend: {
        if !(*eap).arg.is_null() && *(*eap).arg as c_int != NUL {
            let mut p: *mut c_char = (*eap).arg;
            let mut relative: c_int = 0 as c_int;
            if *p as c_int == '-' as c_int {
                relative = -1 as c_int;
                p = p.offset(1);
            } else if *p as c_int == '+' as c_int {
                relative = 1 as c_int;
                p = p.offset(1);
            }
            let mut p_save: *mut c_char = p;
            tab_number = getdigits(&raw mut p, false_0 != 0, tab_number as intmax_t) as c_int;
            if relative == 0 as c_int {
                if strcmp(p, b"$\0".as_ptr() as *const c_char) == 0 as c_int {
                    tab_number = current_tab_nr(::core::ptr::null_mut::<tabpage_T>());
                } else if strcmp(p, b"#\0".as_ptr() as *const c_char) == 0 as c_int {
                    if valid_tabpage(lastused_tabpage.get()) {
                        tab_number = tabpage_index(lastused_tabpage.get());
                    } else {
                        (*eap).errmsg =
                            ex_errmsg(&raw const e_invargval as *const c_char, (*eap).arg);
                        tab_number = 0 as c_int;
                        break '_theend;
                    }
                } else if p == p_save
                    || *p_save as c_int == '-' as c_int
                    || *p as c_int != NUL
                    || tab_number > current_tab_nr(::core::ptr::null_mut::<tabpage_T>())
                {
                    (*eap).errmsg = ex_errmsg(&raw const e_invarg2 as *const c_char, (*eap).arg);
                    break '_theend;
                }
            } else {
                if *p_save as c_int == NUL {
                    tab_number = 1 as c_int;
                } else if p == p_save
                    || *p_save as c_int == '-' as c_int
                    || *p as c_int != NUL
                    || tab_number == 0 as c_int
                {
                    (*eap).errmsg = ex_errmsg(&raw const e_invarg2 as *const c_char, (*eap).arg);
                    break '_theend;
                }
                tab_number = tab_number * relative + tabpage_index(curtab.get());
                if unaccept_arg0 == 0 && relative == -1 as c_int {
                    tab_number -= 1;
                }
            }
            if tab_number < unaccept_arg0
                || tab_number > current_tab_nr(::core::ptr::null_mut::<tabpage_T>())
            {
                (*eap).errmsg = ex_errmsg(&raw const e_invarg2 as *const c_char, (*eap).arg);
            }
        } else if (*eap).addr_count > 0 as c_int {
            if unaccept_arg0 != 0 && (*eap).line2 == 0 as linenr_T {
                (*eap).errmsg = gettext(&raw const e_invrange as *const c_char);
                tab_number = 0 as c_int;
            } else {
                tab_number = (*eap).line2 as c_int;
                if unaccept_arg0 == 0 {
                    let mut cmdp: *mut c_char = (*eap).cmd;
                    loop {
                        cmdp = cmdp.offset(-1);
                        if !(cmdp > *(*eap).cmdlinep
                            && (ascii_iswhite(*cmdp as c_int) as c_int != 0
                                || ascii_isdigit(*cmdp as c_int) as c_int != 0))
                        {
                            break;
                        }
                    }
                    if *cmdp as c_int == '-' as c_int {
                        tab_number -= 1;
                        if tab_number < unaccept_arg0 {
                            (*eap).errmsg = gettext(&raw const e_invrange as *const c_char);
                        }
                    }
                }
            }
        } else {
            match (*eap).cmdidx as c_int {
                461 => {
                    tab_number = tabpage_index(curtab.get()) + 1 as c_int;
                    if tab_number > current_tab_nr(::core::ptr::null_mut::<tabpage_T>()) {
                        tab_number = 1 as c_int;
                    }
                }
                459 => {
                    tab_number = current_tab_nr(::core::ptr::null_mut::<tabpage_T>());
                }
                _ => {
                    tab_number = tabpage_index(curtab.get());
                }
            }
        }
    }
    return tab_number;
}
unsafe extern "C" fn ex_autocmd(mut eap: *mut exarg_T) {
    if secure.get() != 0 {
        secure.set(2 as c_int);
        (*eap).errmsg = gettext(&raw const e_curdir as *const c_char);
    } else if (*eap).cmdidx as c_int == CMD_autocmd as c_int {
        do_autocmd(eap, (*eap).arg, (*eap).forceit);
    } else {
        do_augroup((*eap).arg, (*eap).forceit != 0);
    };
}
unsafe extern "C" fn ex_doautocmd(mut eap: *mut exarg_T) {
    let mut arg: *mut c_char = (*eap).arg;
    let mut call_do_modelines: c_int = check_nomodeline(&raw mut arg) as c_int;
    let mut did_aucmd: bool = false;
    do_doautocmd(arg, false_0 != 0, &raw mut did_aucmd);
    if call_do_modelines != 0 && did_aucmd as c_int != 0 {
        do_modelines(0 as c_int);
    }
}
unsafe extern "C" fn ex_bunload(mut eap: *mut exarg_T) {
    (*eap).errmsg = do_bufdel(
        if (*eap).cmdidx as c_int == CMD_bdelete as c_int {
            DOBUF_DEL as c_int
        } else if (*eap).cmdidx as c_int == CMD_bwipeout as c_int {
            DOBUF_WIPE as c_int
        } else {
            DOBUF_UNLOAD as c_int
        },
        (*eap).arg,
        (*eap).addr_count,
        (*eap).line1 as c_int,
        (*eap).line2 as c_int,
        (*eap).forceit,
    );
}
unsafe extern "C" fn ex_buffer(mut eap: *mut exarg_T) {
    do_exbuffer(eap);
}
unsafe extern "C" fn do_exbuffer(mut eap: *mut exarg_T) {
    if *(*eap).arg != 0 {
        (*eap).errmsg = ex_errmsg(&raw const e_trailing_arg as *const c_char, (*eap).arg);
    } else {
        if (*eap).addr_count == 0 as c_int {
            goto_buffer(eap, DOBUF_CURRENT as c_int, FORWARD as c_int, 0 as c_int);
        } else {
            goto_buffer(
                eap,
                DOBUF_FIRST as c_int,
                FORWARD as c_int,
                (*eap).line2 as c_int,
            );
        }
        if !(*eap).do_ecmd_cmd.is_null() {
            do_cmdline_cmd((*eap).do_ecmd_cmd);
        }
    };
}
unsafe extern "C" fn ex_bmodified(mut eap: *mut exarg_T) {
    goto_buffer(
        eap,
        DOBUF_MOD as c_int,
        FORWARD as c_int,
        (*eap).line2 as c_int,
    );
    if !(*eap).do_ecmd_cmd.is_null() {
        do_cmdline_cmd((*eap).do_ecmd_cmd);
    }
}
unsafe extern "C" fn ex_bnext(mut eap: *mut exarg_T) {
    goto_buffer(
        eap,
        DOBUF_CURRENT as c_int,
        FORWARD as c_int,
        (*eap).line2 as c_int,
    );
    if !(*eap).do_ecmd_cmd.is_null() {
        do_cmdline_cmd((*eap).do_ecmd_cmd);
    }
}
unsafe extern "C" fn ex_bprevious(mut eap: *mut exarg_T) {
    goto_buffer(
        eap,
        DOBUF_CURRENT as c_int,
        BACKWARD as c_int,
        (*eap).line2 as c_int,
    );
    if !(*eap).do_ecmd_cmd.is_null() {
        do_cmdline_cmd((*eap).do_ecmd_cmd);
    }
}
unsafe extern "C" fn ex_brewind(mut eap: *mut exarg_T) {
    goto_buffer(eap, DOBUF_FIRST as c_int, FORWARD as c_int, 0 as c_int);
    if !(*eap).do_ecmd_cmd.is_null() {
        do_cmdline_cmd((*eap).do_ecmd_cmd);
    }
}
unsafe extern "C" fn ex_blast(mut eap: *mut exarg_T) {
    goto_buffer(eap, DOBUF_LAST as c_int, BACKWARD as c_int, 0 as c_int);
    if !(*eap).do_ecmd_cmd.is_null() {
        do_cmdline_cmd((*eap).do_ecmd_cmd);
    }
}
pub unsafe extern "C" fn ends_excmd(mut c: c_int) -> c_int {
    return (c == NUL || c == '|' as c_int || c == '"' as c_int || c == '\n' as c_int) as c_int;
}
pub unsafe extern "C" fn find_nextcmd(mut p: *const c_char) -> *mut c_char {
    while *p as c_int != '|' as c_int && *p as c_int != '\n' as c_int {
        if *p as c_int == NUL {
            return ::core::ptr::null_mut::<c_char>();
        }
        p = p.offset(1);
    }
    return (p as *mut c_char).offset(1 as c_int as isize);
}
pub unsafe extern "C" fn check_nextcmd(mut p: *mut c_char) -> *mut c_char {
    let mut s: *mut c_char = skipwhite(p);
    if *s as c_int == '|' as c_int || *s as c_int == '\n' as c_int {
        return s.offset(1 as c_int as isize);
    }
    return ::core::ptr::null_mut::<c_char>();
}
unsafe extern "C" fn check_more(mut message: bool, mut forceit: bool) -> c_int {
    let mut n: c_int =
        (*(*curwin.get()).w_alist).al_ga.ga_len - (*curwin.get()).w_arg_idx - 1 as c_int;
    if !forceit
        && only_one_window() as c_int != 0
        && (*(*curwin.get()).w_alist).al_ga.ga_len > 1 as c_int
        && !arg_had_last.get()
        && n > 0 as c_int
        && quitmore.get() == 0 as c_int
    {
        if message {
            if (p_confirm.get() != 0 || (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as c_int != 0)
                && !(*curbuf.get()).b_fname.is_null()
            {
                let mut buff: [c_char; 1000] = [0; 1000];
                vim_snprintf(
                    &raw mut buff as *mut c_char,
                    DIALOG_MSG_SIZE as c_int as size_t,
                    ngettext(
                        b"%d more file to edit.  Quit anyway?\0".as_ptr() as *const c_char,
                        b"%d more files to edit.  Quit anyway?\0".as_ptr() as *const c_char,
                        n as c_ulong,
                    ),
                    n,
                );
                if vim_dialog_yesno(
                    VIM_QUESTION as c_int,
                    ::core::ptr::null_mut::<c_char>(),
                    &raw mut buff as *mut c_char,
                    1 as c_int,
                ) == VIM_YES as c_int
                {
                    return OK;
                }
                return FAIL;
            }
            semsg(
                ngettext(
                    b"E173: %d more file to edit\0".as_ptr() as *const c_char,
                    b"E173: %d more files to edit\0".as_ptr() as *const c_char,
                    n as c_ulong,
                ),
                n,
            );
            quitmore.set(2 as c_int);
        }
        return FAIL;
    }
    return OK;
}
pub unsafe extern "C" fn get_command_name(mut _xp: *mut expand_T, mut idx: c_int) -> *mut c_char {
    if idx >= CMD_SIZE as c_int {
        return expand_user_command_name(idx);
    }
    return (*cmdnames.ptr())[idx as usize].cmd_name;
}
unsafe extern "C" fn ex_colorscheme(mut eap: *mut exarg_T) {
    if *(*eap).arg as c_int == NUL {
        let mut expr: *mut c_char = xstrdup(b"g:colors_name\0".as_ptr() as *const c_char);
        (*emsg_off.ptr()) += 1;
        let mut p: *mut c_char = eval_to_string(expr, false_0 != 0, false_0 != 0);
        (*emsg_off.ptr()) -= 1;
        xfree(expr as *mut c_void);
        msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const c_char);
        if !p.is_null() {
            msg(p, 0 as c_int);
            xfree(p as *mut c_void);
        } else {
            msg(b"default\0".as_ptr() as *const c_char, 0 as c_int);
        }
    } else if load_colors((*eap).arg) == FAIL {
        semsg(
            gettext(b"E185: Cannot find color scheme '%s'\0".as_ptr() as *const c_char),
            (*eap).arg,
        );
    }
}
unsafe extern "C" fn ex_highlight(mut eap: *mut exarg_T) {
    if *(*eap).arg as c_int == NUL
        && *(*eap).cmd.offset(2 as c_int as isize) as c_int == '!' as c_int
    {
        msg(
            gettext(b"Greetings, Vim user!\0".as_ptr() as *const c_char),
            0 as c_int,
        );
    }
    do_highlight((*eap).arg, (*eap).forceit != 0, false_0 != 0);
}
pub unsafe extern "C" fn not_exiting(mut save_exiting: bool) {
    exiting.set(save_exiting);
    set_vim_var_string(
        VV_EXITREASON,
        ::core::ptr::null::<c_char>(),
        -1 as ptrdiff_t,
    );
}
pub unsafe extern "C" fn before_quit_autocmds(
    mut wp: *mut win_T,
    mut quit_all: bool,
    mut forceit: bool,
) -> bool {
    if *get_vim_var_str(VV_EXITREASON) as c_int == NUL {
        set_vim_var_string(
            VV_EXITREASON,
            b"quit\0".as_ptr() as *const c_char,
            ::core::mem::size_of::<[c_char; 5]>().wrapping_sub(1 as usize) as ptrdiff_t,
        );
    }
    apply_autocmds(
        EVENT_QUITPRE,
        ::core::ptr::null_mut::<c_char>(),
        ::core::ptr::null_mut::<c_char>(),
        false_0 != 0,
        (*wp).w_buffer,
    );
    if !win_valid(wp)
        || curbuf_locked() as c_int != 0
        || (*(*wp).w_buffer).b_nwindows == 1 as c_int && (*(*wp).w_buffer).b_locked > 0 as c_int
    {
        set_vim_var_string(
            VV_EXITREASON,
            ::core::ptr::null::<c_char>(),
            -1 as ptrdiff_t,
        );
        return true_0 != 0;
    }
    if quit_all as c_int != 0
        || check_more(false_0 != 0, forceit) == OK && only_one_window() as c_int != 0
    {
        apply_autocmds(
            EVENT_EXITPRE,
            ::core::ptr::null_mut::<c_char>(),
            ::core::ptr::null_mut::<c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        if !win_valid(wp)
            || curbuf_locked() as c_int != 0
            || (*curbuf.get()).b_nwindows == 1 as c_int && (*curbuf.get()).b_locked > 0 as c_int
        {
            set_vim_var_string(
                VV_EXITREASON,
                ::core::ptr::null::<c_char>(),
                -1 as ptrdiff_t,
            );
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}
unsafe extern "C" fn ex_quit(mut eap: *mut exarg_T) {
    if cmdwin_type.get() != 0 as c_int {
        cmdwin_result.set(Ctrl_C);
        return;
    }
    if text_locked() {
        text_locked_msg();
        return;
    }
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    if (*eap).addr_count > 0 as c_int {
        let mut wnr: linenr_T = (*eap).line2;
        wp = firstwin.get();
        while !(*wp).w_next.is_null() {
            wnr -= 1;
            if wnr <= 0 as linenr_T {
                break;
            }
            wp = (*wp).w_next;
        }
    } else {
        wp = curwin.get();
    }
    if curbuf_locked() {
        return;
    }
    if before_quit_autocmds(wp, false_0 != 0, (*eap).forceit != 0) {
        return;
    }
    let mut save_exiting: bool = exiting.get();
    if check_more(false_0 != 0, (*eap).forceit != 0) == OK && only_one_window() as c_int != 0 {
        exiting.set(true_0 != 0);
    }
    if !buf_hide((*wp).w_buffer)
        && check_changed(
            (*wp).w_buffer,
            (if p_awa.get() != 0 {
                CCGD_AW as c_int
            } else {
                0 as c_int
            }) | (if (*eap).forceit != 0 {
                CCGD_FORCEIT as c_int
            } else {
                0 as c_int
            }) | CCGD_EXCMD as c_int,
        ) as c_int
            != 0
        || check_more(true_0 != 0, (*eap).forceit != 0) == FAIL
        || only_one_window() as c_int != 0
            && check_changed_any((*eap).forceit != 0, true_0 != 0) as c_int != 0
    {
        not_exiting(save_exiting);
    } else {
        if only_one_window() as c_int != 0
            && (firstwin.get() == lastwin.get() || (*eap).addr_count == 0 as c_int)
        {
            getout(0 as c_int);
        }
        not_exiting(save_exiting);
        win_close(
            wp,
            !buf_hide((*wp).w_buffer) || (*eap).forceit != 0,
            (*eap).forceit != 0,
        );
    };
}
unsafe extern "C" fn ex_cquit(mut eap: *mut exarg_T) -> ! {
    let mut status: c_int = if (*eap).addr_count > 0 as c_int {
        (*eap).line2 as c_int
    } else {
        EXIT_FAILURE
    };
    ui_call_error_exit(status as Integer);
    getout(status);
}
pub unsafe extern "C" fn before_quit_all(mut eap: *mut exarg_T) -> c_int {
    if cmdwin_type.get() != 0 as c_int {
        cmdwin_result.set(if (*eap).forceit != 0 {
            -(253 as c_int + ((KE_XF1 as c_int) << 8 as c_int))
        } else {
            -(253 as c_int + ((KE_XF2 as c_int) << 8 as c_int))
        });
        return FAIL;
    }
    if text_locked() {
        text_locked_msg();
        return FAIL;
    }
    if before_quit_autocmds(curwin.get(), true_0 != 0, (*eap).forceit != 0) {
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn ex_quitall(mut eap: *mut exarg_T) {
    if before_quit_all(eap) == FAIL {
        return;
    }
    let mut save_exiting: bool = exiting.get();
    exiting.set(true_0 != 0);
    if (*eap).forceit != 0 || !check_changed_any(false_0 != 0, false_0 != 0) {
        getout(0 as c_int);
    }
    not_exiting(save_exiting);
}
unsafe extern "C" fn ex_restart(mut eap: *mut exarg_T) {
    let mut servername_args: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut servername_args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_14 { boolean: false },
    }; 1];
    let mut result: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_14 { boolean: false },
    };
    let mut listen_addr: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut quit_cmd: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut quit_cmd_copy: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut result_mem: ArenaMem = ::core::ptr::null_mut::<consumed_blk>();
    let mut detach_args: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut detach_args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_14 { boolean: false },
    }; 1];
    let mut chanclose_expr_args: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut chanclose_expr_args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_14 { boolean: false },
    }; 1];
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<c_char>(),
    };
    let no_ui: bool = ui_active() == 0;
    let mut exepath: *const c_char = get_vim_var_str(VV_PROGPATH);
    let mut l: *const list_T = get_vim_var_list(VV_ARGV);
    let mut argc: c_int = tv_list_len(l);
    let mut argv: *mut *mut c_char = xcalloc(
        (argc as size_t).wrapping_add(3 as size_t),
        ::core::mem::size_of::<*mut c_char>(),
    ) as *mut *mut c_char;
    let mut i: size_t = 0 as size_t;
    let mut listen_arg: *const c_char = ::core::ptr::null::<c_char>();
    let mut li: *const listitem_T = (*l).lv_first;
    while !li.is_null() {
        let mut arg: *const c_char = tv_get_string(&raw const (*li).li_tv);
        if i > 0 as size_t && strequal(arg, b"--\0".as_ptr() as *const c_char) as c_int != 0 {
            break;
        }
        if i > 0 as size_t && strequal(arg, b"-s\0".as_ptr() as *const c_char) as c_int != 0 {
            li = (*li).li_next;
        } else {
            if i > 0 as size_t
                && strequal(arg, b"--listen\0".as_ptr() as *const c_char) as c_int != 0
            {
                let mut next_li: *const listitem_T = (*li).li_next;
                if !next_li.is_null() {
                    let mut addr: *const c_char = tv_get_string(&raw const (*next_li).li_tv);
                    if !strstr(addr, b":\0".as_ptr() as *const c_char).is_null()
                        || !strstr(addr, b"/\0".as_ptr() as *const c_char).is_null()
                        || !strstr(addr, b"\\\0".as_ptr() as *const c_char).is_null()
                    {
                        listen_arg = addr;
                    }
                }
            }
            if i == 0 as size_t
                || !strequal(arg, b"--embed\0".as_ptr() as *const c_char)
                    && !strequal(arg, b"--headless\0".as_ptr() as *const c_char)
                    && !strequal(arg, b"-\0".as_ptr() as *const c_char)
            {
                let c2rust_fresh4 = i;
                i = i.wrapping_add(1);
                let c2rust_lvalue_ptr = &raw mut *argv.offset(c2rust_fresh4 as isize);
                *c2rust_lvalue_ptr = xstrdup(arg);
                if i == 1 as size_t {
                    let c2rust_fresh5 = i;
                    i = i.wrapping_add(1);
                    let c2rust_lvalue_ptr_0 = &raw mut *argv.offset(c2rust_fresh5 as isize);
                    *c2rust_lvalue_ptr_0 = xstrdup(b"--embed\0".as_ptr() as *const c_char);
                    if no_ui {
                        let c2rust_fresh6 = i;
                        i = i.wrapping_add(1);
                        let c2rust_lvalue_ptr_1 = &raw mut *argv.offset(c2rust_fresh6 as isize);
                        *c2rust_lvalue_ptr_1 = xstrdup(b"--headless\0".as_ptr() as *const c_char);
                    }
                }
            }
        }
        li = (*li).li_next;
    }
    let mut server_stopped: bool = if !listen_arg.is_null() {
        server_stop(listen_arg, true_0 != 0) as c_int
    } else {
        false_0
    } != 0;
    let mut on_err: CallbackReader = CallbackReader {
        cb: Callback {
            data: C2Rust_Unnamed_20 {
                funcref: ::core::ptr::null_mut::<c_char>(),
            },
            type_0: kCallbackNone,
        },
        self_0: ::core::ptr::null_mut::<dict_T>(),
        buffer: GA_EMPTY_INIT_VALUE,
        eof: false,
        buffered: false_0 != 0,
        fwd_err: false_0 != 0,
        type_0: ::core::ptr::null::<c_char>(),
    };
    on_err.fwd_err = true_0 != 0;
    let mut detach: bool = true_0 != 0;
    let mut exit_status: varnumber_T = 0;
    let mut channel: *mut Channel = channel_job_start(
        argv,
        exepath,
        CallbackReader {
            cb: Callback {
                data: C2Rust_Unnamed_20 {
                    funcref: ::core::ptr::null_mut::<c_char>(),
                },
                type_0: kCallbackNone,
            },
            self_0: ::core::ptr::null_mut::<dict_T>(),
            buffer: GA_EMPTY_INIT_VALUE,
            eof: false,
            buffered: false_0 != 0,
            fwd_err: false_0 != 0,
            type_0: ::core::ptr::null::<c_char>(),
        },
        on_err,
        Callback {
            data: C2Rust_Unnamed_20 {
                funcref: ::core::ptr::null_mut::<c_char>(),
            },
            type_0: kCallbackNone,
        },
        false_0 != 0,
        true_0 != 0,
        true_0 != 0,
        detach,
        kChannelStdinPipe,
        ::core::ptr::null::<c_char>(),
        0 as uint16_t,
        0 as uint16_t,
        ::core::ptr::null_mut::<dict_T>(),
        &raw mut exit_status,
    );
    if channel.is_null() {
        emsg(b"cannot create a channel job\0".as_ptr() as *const c_char);
    } else {
        result_mem = ::core::ptr::null_mut::<consumed_blk>();
        detach_args = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        detach_args__items = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_14 { boolean: false },
        }; 1];
        detach_args.capacity = 1 as size_t;
        detach_args.items = &raw mut detach_args__items as *mut Object;
        let c2rust_fresh7 = detach_args.size;
        detach_args.size = detach_args.size.wrapping_add(1);
        *detach_args.items.offset(c2rust_fresh7 as isize) = object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed_14 { boolean: true },
        };
        rpc_send_call(
            (*channel).id,
            b"nvim__chan_set_detach\0".as_ptr() as *const c_char,
            detach_args,
            &raw mut result_mem,
            &raw mut err,
        );
        '_fail_2: {
            if err.type_0 as c_int == kErrorTypeNone as c_int {
                arena_mem_free(result_mem);
                result_mem = ::core::ptr::null_mut::<consumed_blk>();
                if *(*eap).arg as c_int != NUL {
                    let mut autocmd_opts: Dict = Dict {
                        size: 0 as size_t,
                        capacity: 0 as size_t,
                        items: ::core::ptr::null_mut::<KeyValuePair>(),
                    };
                    let mut autocmd_opts__items: [KeyValuePair; 3] = [KeyValuePair {
                        key: String_0 {
                            data: ::core::ptr::null_mut::<c_char>(),
                            size: 0,
                        },
                        value: Object {
                            type_0: kObjectTypeNil,
                            data: C2Rust_Unnamed_14 { boolean: false },
                        },
                    }; 3];
                    autocmd_opts.capacity = 3 as size_t;
                    autocmd_opts.items = &raw mut autocmd_opts__items as *mut KeyValuePair;
                    let c2rust_fresh8 = autocmd_opts.size;
                    autocmd_opts.size = autocmd_opts.size.wrapping_add(1);
                    *autocmd_opts.items.offset(c2rust_fresh8 as isize) = key_value_pair {
                        key: cstr_as_string(b"once\0".as_ptr() as *const c_char),
                        value: object {
                            type_0: kObjectTypeBoolean,
                            data: C2Rust_Unnamed_14 { boolean: true },
                        },
                    };
                    let c2rust_fresh9 = autocmd_opts.size;
                    autocmd_opts.size = autocmd_opts.size.wrapping_add(1);
                    *autocmd_opts.items.offset(c2rust_fresh9 as isize) = key_value_pair {
                        key: cstr_as_string(b"nested\0".as_ptr() as *const c_char),
                        value: object {
                            type_0: kObjectTypeBoolean,
                            data: C2Rust_Unnamed_14 { boolean: true },
                        },
                    };
                    let c2rust_fresh10 = autocmd_opts.size;
                    autocmd_opts.size = autocmd_opts.size.wrapping_add(1);
                    *autocmd_opts.items.offset(c2rust_fresh10 as isize) = key_value_pair {
                        key: cstr_as_string(b"command\0".as_ptr() as *const c_char),
                        value: object {
                            type_0: kObjectTypeString,
                            data: C2Rust_Unnamed_14 {
                                string: cstr_as_string((*eap).arg),
                            },
                        },
                    };
                    let mut autocmd_args: Array = Array {
                        size: 0 as size_t,
                        capacity: 0 as size_t,
                        items: ::core::ptr::null_mut::<Object>(),
                    };
                    let mut autocmd_args__items: [Object; 2] = [Object {
                        type_0: kObjectTypeNil,
                        data: C2Rust_Unnamed_14 { boolean: false },
                    }; 2];
                    autocmd_args.capacity = 2 as size_t;
                    autocmd_args.items = &raw mut autocmd_args__items as *mut Object;
                    let c2rust_fresh11 = autocmd_args.size;
                    autocmd_args.size = autocmd_args.size.wrapping_add(1);
                    *autocmd_args.items.offset(c2rust_fresh11 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed_14 {
                            string: cstr_as_string(b"UIEnter\0".as_ptr() as *const c_char),
                        },
                    };
                    let c2rust_fresh12 = autocmd_args.size;
                    autocmd_args.size = autocmd_args.size.wrapping_add(1);
                    *autocmd_args.items.offset(c2rust_fresh12 as isize) = object {
                        type_0: kObjectTypeDict,
                        data: C2Rust_Unnamed_14 { dict: autocmd_opts },
                    };
                    rpc_send_call(
                        (*channel).id,
                        b"nvim_create_autocmd\0".as_ptr() as *const c_char,
                        autocmd_args,
                        &raw mut result_mem,
                        &raw mut err,
                    );
                    if err.type_0 as c_int != kErrorTypeNone as c_int {
                        break '_fail_2;
                    } else {
                        arena_mem_free(result_mem);
                        result_mem = ::core::ptr::null_mut::<consumed_blk>();
                    }
                }
                servername_args = Array {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<Object>(),
                };
                servername_args__items = [Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed_14 { boolean: false },
                }; 1];
                servername_args.capacity = 1 as size_t;
                servername_args.items = &raw mut servername_args__items as *mut Object;
                let c2rust_fresh13 = servername_args.size;
                servername_args.size = servername_args.size.wrapping_add(1);
                *servername_args.items.offset(c2rust_fresh13 as isize) = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_14 {
                        string: cstr_as_string(b"servername\0".as_ptr() as *const c_char),
                    },
                };
                result = rpc_send_call(
                    (*channel).id,
                    b"nvim_get_vvar\0".as_ptr() as *const c_char,
                    servername_args,
                    &raw mut result_mem,
                    &raw mut err,
                );
                if err.type_0 as c_int == kErrorTypeNone as c_int {
                    if result.type_0 as c_uint != kObjectTypeString as c_int as c_uint
                        || result.data.string.size == 0 as size_t
                    {
                        emsg(
                            b"restart failed: could not get listen address from new server\0"
                                .as_ptr() as *const c_char,
                        );
                    } else {
                        listen_addr = xmemdupz(
                            result.data.string.data as *const c_void,
                            result.data.string.size,
                        ) as *mut c_char;
                        arena_mem_free(result_mem);
                        result_mem = ::core::ptr::null_mut::<consumed_blk>();
                        ui_call_restart(cstr_as_string(listen_addr));
                        ui_flush();
                        xfree(listen_addr as *mut c_void);
                        set_vim_var_string(
                            VV_EXITREASON,
                            b"restart\0".as_ptr() as *const c_char,
                            ::core::mem::size_of::<[c_char; 8]>().wrapping_sub(1 as usize)
                                as ptrdiff_t,
                        );
                        quit_cmd = (if !(*eap).do_ecmd_cmd.is_null() {
                            (*eap).do_ecmd_cmd as *const c_char
                        } else {
                            b"qall\0".as_ptr() as *const c_char
                        }) as *mut c_char;
                        quit_cmd_copy = ::core::ptr::null_mut::<c_char>();
                        if (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as c_int != 0 {
                            quit_cmd_copy =
                                concat_str(b"confirm \0".as_ptr() as *const c_char, quit_cmd);
                            quit_cmd = quit_cmd_copy;
                        }
                        nvim_command(cstr_as_string(quit_cmd), &raw mut err);
                        xfree(quit_cmd_copy as *mut c_void);
                        if err.type_0 as c_int != kErrorTypeNone as c_int {
                            emsg(err.msg);
                            api_clear_error(&raw mut err);
                        } else if !exiting.get() {
                            emsg(b"restart failed: +cmd did not quit the server\0".as_ptr()
                                as *const c_char);
                        }
                    }
                }
            }
        }
        set_vim_var_string(
            VV_EXITREASON,
            ::core::ptr::null::<c_char>(),
            -1 as ptrdiff_t,
        );
        if err.type_0 as c_int != kErrorTypeNone as c_int {
            emsg(err.msg);
            api_clear_error(&raw mut err);
        }
        arena_mem_free(result_mem);
        result_mem = ::core::ptr::null_mut::<consumed_blk>();
        chanclose_expr_args = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        chanclose_expr_args__items = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_14 { boolean: false },
        }; 1];
        chanclose_expr_args.capacity = 1 as size_t;
        chanclose_expr_args.items = &raw mut chanclose_expr_args__items as *mut Object;
        let c2rust_fresh14 = chanclose_expr_args.size;
        chanclose_expr_args.size = chanclose_expr_args.size.wrapping_add(1);
        *chanclose_expr_args.items.offset(c2rust_fresh14 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed_14 {
                string: cstr_as_string(b"chanclose(v:stderr)\0".as_ptr() as *const c_char),
            },
        };
        rpc_send_call(
            (*channel).id,
            b"nvim_eval\0".as_ptr() as *const c_char,
            chanclose_expr_args,
            &raw mut result_mem,
            &raw mut err,
        );
        api_clear_error(&raw mut err);
        arena_mem_free(result_mem);
        proc_stop(channel_proc(channel));
        if proc_wait(
            channel_proc(channel),
            -1 as c_int,
            ::core::ptr::null_mut::<MultiQueue>(),
        ) < 0 as c_int
        {
            emsg(b"killing new nvim server failed\0".as_ptr() as *const c_char);
        }
    }
    if server_stopped as c_int != 0 && server_start(listen_arg) != 0 as c_int {
        semsg(
            b"couldn't resume listening on %s\0".as_ptr() as *const c_char,
            listen_arg,
        );
    }
}
unsafe extern "C" fn ex_close(mut eap: *mut exarg_T) {
    let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut winnr: c_int = 0 as c_int;
    if cmdwin_type.get() != 0 as c_int {
        cmdwin_result.set(Ctrl_C);
    } else if !text_locked() && !curbuf_locked() {
        if (*eap).addr_count == 0 as c_int {
            ex_win_close(
                (*eap).forceit,
                curwin.get(),
                ::core::ptr::null_mut::<tabpage_T>(),
            );
        } else {
            let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                firstwin.get()
            } else {
                (*curtab.get()).tp_firstwin
            };
            while !wp.is_null() {
                winnr += 1;
                if winnr as linenr_T == (*eap).line2 {
                    win = wp;
                    break;
                } else {
                    wp = (*wp).w_next;
                }
            }
            if win.is_null() {
                win = lastwin.get();
            }
            ex_win_close((*eap).forceit, win, ::core::ptr::null_mut::<tabpage_T>());
        }
    }
}
unsafe extern "C" fn ex_pclose(mut eap: *mut exarg_T) {
    let mut win: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !win.is_null() {
        if (*win).w_onebuf_opt.wo_pvw != 0 {
            ex_win_close((*eap).forceit, win, ::core::ptr::null_mut::<tabpage_T>());
            break;
        } else {
            win = (*win).w_next;
        }
    }
}
pub unsafe extern "C" fn ex_win_close(
    mut forceit: c_int,
    mut win: *mut win_T,
    mut tp: *mut tabpage_T,
) {
    if is_aucmd_win(win) {
        emsg(gettext(&raw const e_autocmd_close as *const c_char));
        return;
    }
    if !(*win).w_floating && window_layout_locked(CMD_close) as c_int != 0 {
        return;
    }
    let mut buf: *mut buf_T = (*win).w_buffer;
    let mut need_hide: bool = bufIsChanged(buf) as c_int != 0 && (*buf).b_nwindows <= 1 as c_int;
    if need_hide as c_int != 0 && !buf_hide(buf) && forceit == 0 {
        if (p_confirm.get() != 0 || (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as c_int != 0)
            && p_write.get() != 0
        {
            let mut bufref: bufref_T = bufref_T {
                br_buf: ::core::ptr::null_mut::<buf_T>(),
                br_fnum: 0,
                br_buf_free_count: 0,
            };
            set_bufref(&raw mut bufref, buf);
            dialog_changed(buf, false_0 != 0);
            if bufref_valid(&raw mut bufref) as c_int != 0 && bufIsChanged(buf) as c_int != 0 {
                return;
            }
            need_hide = false_0 != 0;
        } else {
            no_write_message();
            return;
        }
    }
    if tp.is_null() {
        win_close(win, !need_hide && !buf_hide(buf), forceit != 0);
    } else {
        win_close_othertab(
            win,
            (!need_hide && !buf_hide(buf)) as c_int,
            tp,
            forceit != 0,
        );
    };
}
unsafe extern "C" fn ex_tabclose(mut eap: *mut exarg_T) {
    if cmdwin_type.get() != 0 as c_int {
        cmdwin_result.set(-(253 as c_int + ((KE_IGNORE as c_int) << 8 as c_int)));
        return;
    }
    if (*first_tabpage.get()).tp_next.is_null() {
        emsg(gettext(
            b"E784: Cannot close last tab page\0".as_ptr() as *const c_char
        ));
        return;
    }
    if window_layout_locked(CMD_tabclose) {
        return;
    }
    let mut tab_number: c_int = get_tabpage_arg(eap);
    if !(*eap).errmsg.is_null() {
        return;
    }
    let mut tp: *mut tabpage_T = find_tabpage(tab_number);
    if tp.is_null() {
        beep_flush();
        return;
    }
    if tp != curtab.get() {
        tabpage_close_other(tp, (*eap).forceit);
        return;
    } else if !text_locked() && !curbuf_locked() {
        tabpage_close((*eap).forceit);
    }
}
unsafe extern "C" fn ex_tabonly(mut eap: *mut exarg_T) {
    if cmdwin_type.get() != 0 as c_int {
        cmdwin_result.set(-(253 as c_int + ((KE_IGNORE as c_int) << 8 as c_int)));
        return;
    }
    if (*first_tabpage.get()).tp_next.is_null() {
        msg(
            gettext(b"Already only one tab page\0".as_ptr() as *const c_char),
            0 as c_int,
        );
        return;
    }
    if window_layout_locked(CMD_tabonly) {
        return;
    }
    let mut tab_number: c_int = get_tabpage_arg(eap);
    if !(*eap).errmsg.is_null() {
        return;
    }
    goto_tabpage(tab_number);
    let mut done: c_int = 0 as c_int;
    while done < 1000 as c_int {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            if (*tp).tp_topframe != topframe.get() {
                tabpage_close_other(tp as *mut tabpage_T, (*eap).forceit);
                if valid_tabpage(tp as *mut tabpage_T) {
                    done = 1000 as c_int;
                }
                break;
            } else {
                tp = (*tp).tp_next as *mut tabpage_T;
            }
        }
        '_c2rust_label: {
            if !(*first_tabpage.ptr()).is_null() {
            } else {
                __assert_fail(
                    b"first_tabpage\0".as_ptr() as *const c_char,
                    b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                    5361 as c_uint,
                    b"void ex_tabonly(exarg_T *)\0".as_ptr() as *const c_char,
                );
            }
        };
        if (*first_tabpage.get()).tp_next.is_null() {
            break;
        }
        done += 1;
    }
}
pub unsafe extern "C" fn tabpage_close(mut forceit: c_int) {
    if window_layout_locked(CMD_tabclose) {
        return;
    }
    trigger_tabclosedpre(curtab.get());
    (*curtab.get()).tp_did_tabclosedpre = true_0 != 0;
    let save_curtab: *mut tabpage_T = curtab.get();
    while (*curwin.get()).w_floating {
        ex_win_close(forceit, curwin.get(), ::core::ptr::null_mut::<tabpage_T>());
    }
    if !(firstwin.get() == lastwin.get()) {
        close_others(true_0, forceit);
    }
    if firstwin.get() == lastwin.get() {
        ex_win_close(forceit, curwin.get(), ::core::ptr::null_mut::<tabpage_T>());
    }
    if curtab.get() == save_curtab {
        (*curtab.get()).tp_did_tabclosedpre = false_0 != 0;
    }
}
pub unsafe extern "C" fn tabpage_close_other(mut tp: *mut tabpage_T, mut forceit: c_int) {
    let mut done: c_int = 0 as c_int;
    let mut prev_idx: [c_char; 65] = [0; 65];
    if window_layout_locked(CMD_SIZE) {
        return;
    }
    trigger_tabclosedpre(tp);
    (*tp).tp_did_tabclosedpre = true_0 != 0;
    loop {
        done += 1;
        if done >= 1000 as c_int {
            break;
        }
        snprintf(
            &raw mut prev_idx as *mut c_char,
            ::core::mem::size_of::<[c_char; 65]>(),
            b"%i\0".as_ptr() as *const c_char,
            tabpage_index(tp),
        );
        let mut wp: *mut win_T = (*tp).tp_lastwin;
        ex_win_close(forceit, wp, tp);
        if !valid_tabpage(tp) {
            break;
        }
        if (*tp).tp_lastwin != wp {
            continue;
        }
        done = 1000 as c_int;
        break;
    }
    if done >= 1000 as c_int {
        (*tp).tp_did_tabclosedpre = false_0 != 0;
        return;
    }
}
unsafe extern "C" fn ex_only(mut eap: *mut exarg_T) {
    if window_layout_locked(CMD_only) {
        return;
    }
    if (*eap).addr_count > 0 as c_int {
        let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut wnr: linenr_T = (*eap).line2;
        wp = firstwin.get();
        loop {
            wnr -= 1;
            if wnr <= 0 as linenr_T {
                break;
            }
            if (*wp).w_next.is_null() {
                break;
            }
            wp = (*wp).w_next;
        }
        if wp != curwin.get() {
            win_goto(wp);
        }
    }
    close_others(true_0, (*eap).forceit);
}
unsafe extern "C" fn ex_hide(mut eap: *mut exarg_T) {
    if (*eap).skip != 0 {
        return;
    }
    let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    if (*eap).addr_count == 0 as c_int {
        win = curwin.get();
    } else {
        let mut winnr: c_int = 0 as c_int;
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            winnr += 1;
            if winnr as linenr_T == (*eap).line2 {
                win = wp;
                break;
            } else {
                wp = (*wp).w_next;
            }
        }
        if win.is_null() {
            win = lastwin.get();
        }
    }
    if !(*win).w_floating && window_layout_locked(CMD_hide) as c_int != 0 {
        return;
    }
    win_close(win, false_0 != 0, (*eap).forceit != 0);
}
unsafe extern "C" fn ex_stop(mut eap: *mut exarg_T) {
    if (*eap).forceit == 0 {
        autowrite_all();
    }
    may_trigger_vim_suspend_resume(true_0 != 0);
    ui_call_suspend();
    ui_flush();
}
unsafe extern "C" fn ex_exit(mut eap: *mut exarg_T) {
    if cmdwin_type.get() != 0 as c_int {
        cmdwin_result.set(Ctrl_C);
        return;
    }
    if text_locked() {
        text_locked_msg();
        return;
    }
    let mut save_exiting: bool = exiting.get();
    if check_more(false_0 != 0, (*eap).forceit != 0) == OK && only_one_window() as c_int != 0 {
        exiting.set(true_0 != 0);
    }
    if ((*eap).cmdidx as c_int == CMD_wq as c_int || curbufIsChanged() as c_int != 0)
        && do_write(eap) == FAIL
        || before_quit_autocmds(curwin.get(), false_0 != 0, (*eap).forceit != 0) as c_int != 0
        || check_more(true_0 != 0, (*eap).forceit != 0) == FAIL
        || only_one_window() as c_int != 0
            && check_changed_any((*eap).forceit != 0, false_0 != 0) as c_int != 0
    {
        not_exiting(save_exiting);
    } else {
        if only_one_window() {
            getout(0 as c_int);
        }
        not_exiting(save_exiting);
        win_close(
            curwin.get(),
            !buf_hide((*curwin.get()).w_buffer),
            (*eap).forceit != 0,
        );
    };
}
unsafe extern "C" fn ex_print(mut eap: *mut exarg_T) {
    if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
        emsg(gettext(&raw const e_empty_buffer as *const c_char));
    } else {
        let mut line: linenr_T = (*eap).line1;
        while line <= (*eap).line2 && !got_int.get() {
            print_line(
                line,
                (*eap).cmdidx as c_int == CMD_number as c_int
                    || (*eap).cmdidx as c_int == CMD_pound as c_int
                    || (*eap).flags & EXFLAG_NR != 0,
                (*eap).cmdidx as c_int == CMD_list as c_int || (*eap).flags & EXFLAG_LIST != 0,
                line == (*eap).line1,
            );
            line += 1;
            os_breakcheck();
        }
        setpcmark();
        (*curwin.get()).w_cursor.lnum = (*eap).line2;
        beginline(BL_SOL as c_int | BL_FIX as c_int);
    }
    ex_no_reprint.set(true_0 != 0);
}
unsafe extern "C" fn ex_goto(mut eap: *mut exarg_T) {
    goto_byte((*eap).line2 as c_int);
}
unsafe extern "C" fn ex_preserve(mut _eap: *mut exarg_T) {
    ml_preserve(curbuf.get(), true_0 != 0, true_0 != 0);
}
unsafe extern "C" fn ex_recover(mut eap: *mut exarg_T) {
    recoverymode.set(true_0 != 0);
    if !check_changed(
        curbuf.get(),
        (if p_awa.get() != 0 {
            CCGD_AW as c_int
        } else {
            0 as c_int
        }) | CCGD_MULTWIN as c_int
            | (if (*eap).forceit != 0 {
                CCGD_FORCEIT as c_int
            } else {
                0 as c_int
            })
            | CCGD_EXCMD as c_int,
    ) && (*(*eap).arg as c_int == NUL
        || setfname(
            curbuf.get(),
            (*eap).arg,
            ::core::ptr::null_mut::<c_char>(),
            true_0 != 0,
        ) == OK)
    {
        ml_recover(true_0 != 0);
    }
    recoverymode.set(false_0 != 0);
}
unsafe extern "C" fn ex_wrongmodifier(mut eap: *mut exarg_T) {
    (*eap).errmsg = gettext(&raw const e_invcmd as *const c_char);
}
static ffu_cb: GlobalCell<Callback> = GlobalCell::new(Callback {
    data: C2Rust_Unnamed_20 {
        funcref: ::core::ptr::null_mut::<c_char>(),
    },
    type_0: kCallbackNone,
});
unsafe extern "C" fn get_findfunc_callback() -> *mut Callback {
    return if *(*curbuf.get()).b_p_ffu as c_int != NUL {
        &raw mut (*curbuf.get()).b_ffu_cb
    } else {
        ffu_cb.ptr()
    };
}
unsafe extern "C" fn call_findfunc(
    mut pat: *mut c_char,
    mut cmdcomplete: BoolVarValue,
) -> *mut list_T {
    let saved_sctx: sctx_T = current_sctx.get();
    let mut args: [typval_T; 3] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 3];
    args[0 as c_int as usize].v_type = VAR_STRING;
    args[0 as c_int as usize].vval.v_string = pat;
    args[1 as c_int as usize].v_type = VAR_BOOL;
    args[1 as c_int as usize].vval.v_bool = cmdcomplete;
    args[2 as c_int as usize].v_type = VAR_UNKNOWN;
    (*textlock.ptr()) += 1;
    let mut ctx: *mut sctx_T = get_option_sctx(kOptFindfunc);
    if !ctx.is_null() {
        current_sctx.set(*ctx);
    }
    let mut cb: *mut Callback = get_findfunc_callback();
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut retval: c_int = callback_call(
        cb,
        2 as c_int,
        &raw mut args as *mut typval_T,
        &raw mut rettv,
    ) as c_int;
    current_sctx.set(saved_sctx);
    (*textlock.ptr()) -= 1;
    let mut retlist: *mut list_T = ::core::ptr::null_mut::<list_T>();
    if retval == OK {
        if rettv.v_type as c_uint == VAR_LIST as c_int as c_uint {
            retlist = tv_list_copy(
                ::core::ptr::null::<vimconv_T>(),
                rettv.vval.v_list,
                false_0 != 0,
                get_copyID(),
            );
        } else {
            emsg(gettext(
                &raw const e_invalid_return_type_from_findfunc as *const c_char,
            ));
        }
        tv_clear(&raw mut rettv);
    }
    return retlist;
}
pub unsafe extern "C" fn expand_findfunc(
    mut pat: *mut c_char,
    mut files: *mut *mut *mut c_char,
    mut numMatches: *mut c_int,
) -> c_int {
    *numMatches = 0 as c_int;
    *files = ::core::ptr::null_mut::<*mut c_char>();
    let mut l: *mut list_T = call_findfunc(pat, kBoolVarTrue);
    if l.is_null() {
        return FAIL;
    }
    let mut len: c_int = tv_list_len(l);
    if len == 0 as c_int {
        tv_list_free(l);
        return FAIL;
    }
    *files = xmalloc(::core::mem::size_of::<*mut c_char>().wrapping_mul(len as size_t))
        as *mut *mut c_char;
    let mut idx: c_int = 0 as c_int;
    let l_: *const list_T = l;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            if (*li).li_tv.v_type as c_uint == VAR_STRING as c_int as c_uint {
                *(*files).offset(idx as isize) = xstrdup((*li).li_tv.vval.v_string);
                idx += 1;
            }
            li = (*li).li_next;
        }
    }
    *numMatches = idx;
    tv_list_free(l);
    return OK;
}
unsafe extern "C" fn findfunc_find_file(
    mut findarg: *mut c_char,
    mut findarg_len: size_t,
    mut count: c_int,
) -> *mut c_char {
    let mut ret_fname: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let cc: c_char = *findarg.offset(findarg_len as isize);
    *findarg.offset(findarg_len as isize) = NUL as c_char;
    let mut fname_list: *mut list_T = call_findfunc(findarg, kBoolVarFalse);
    let mut fname_count: c_int = tv_list_len(fname_list);
    if fname_count == 0 as c_int {
        semsg(
            gettext(&raw const e_cant_find_file_str_in_path as *const c_char),
            findarg,
        );
    } else if count > fname_count {
        semsg(
            gettext(&raw const e_no_more_file_str_found_in_path as *const c_char),
            findarg,
        );
    } else {
        let mut li: *mut listitem_T = tv_list_find(fname_list, count - 1 as c_int);
        if !li.is_null() && (*li).li_tv.v_type as c_uint == VAR_STRING as c_int as c_uint {
            ret_fname = xstrdup((*li).li_tv.vval.v_string);
        }
    }
    if !fname_list.is_null() {
        tv_list_free(fname_list);
    }
    *findarg.offset(findarg_len as isize) = cc;
    return ret_fname;
}
pub unsafe extern "C" fn did_set_findfunc(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut retval: c_int = 0;
    if (*args).os_flags & OPT_LOCAL as c_int != 0 {
        retval = option_set_callback_func((*buf).b_p_ffu, &raw mut (*buf).b_ffu_cb);
    } else {
        retval = option_set_callback_func(p_ffu.get(), ffu_cb.ptr());
        if (*args).os_flags & OPT_GLOBAL as c_int == 0 {
            callback_free(&raw mut (*buf).b_ffu_cb);
        }
    }
    if retval == FAIL {
        return &raw const e_invarg as *const c_char;
    }
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    let mut name: *mut c_char = get_scriptlocal_funcname(*varp);
    if !name.is_null() {
        free_string_option(*varp);
        *varp = name;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn set_ref_in_findfunc(mut copyID: c_int) -> bool {
    let mut abort_0: bool = false_0 != 0;
    abort_0 = set_ref_in_callback(
        ffu_cb.ptr(),
        copyID,
        ::core::ptr::null_mut::<*mut ht_stack_T>(),
        ::core::ptr::null_mut::<*mut list_stack_T>(),
    );
    return abort_0;
}
pub unsafe extern "C" fn ex_splitview(mut eap: *mut exarg_T) {
    let mut old_curwin: *mut win_T = curwin.get();
    let mut fname: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let use_tab: bool = (*eap).cmdidx as c_int == CMD_tabedit as c_int
        || (*eap).cmdidx as c_int == CMD_tabfind as c_int
        || (*eap).cmdidx as c_int == CMD_tabnew as c_int;
    if bt_quickfix(curbuf.get()) as c_int != 0 && (*cmdmod.ptr()).cmod_tab == 0 as c_int {
        if (*eap).cmdidx as c_int == CMD_split as c_int {
            (*eap).cmdidx = CMD_new;
        }
        if (*eap).cmdidx as c_int == CMD_vsplit as c_int {
            (*eap).cmdidx = CMD_vnew;
        }
    }
    '_theend: {
        if (*eap).cmdidx as c_int == CMD_sfind as c_int
            || (*eap).cmdidx as c_int == CMD_tabfind as c_int
        {
            if *get_findfunc() as c_int != NUL {
                fname = findfunc_find_file(
                    (*eap).arg,
                    strlen((*eap).arg),
                    if (*eap).addr_count > 0 as c_int {
                        (*eap).line2 as c_int
                    } else {
                        1 as c_int
                    },
                );
            } else {
                let mut file_to_find: *mut c_char = ::core::ptr::null_mut::<c_char>();
                let mut search_ctx: *mut c_char = ::core::ptr::null_mut::<c_char>();
                fname = find_file_in_path(
                    (*eap).arg,
                    strlen((*eap).arg),
                    FNAME_MESS as c_int,
                    true_0,
                    (*curbuf.get()).b_ffname,
                    &raw mut file_to_find,
                    &raw mut search_ctx,
                );
                xfree(file_to_find as *mut c_void);
                vim_findfile_cleanup(search_ctx as *mut c_void);
            }
            if fname.is_null() {
                break '_theend;
            } else {
                (*eap).arg = fname;
            }
        }
        if use_tab {
            if !win_new_tabpage(
                if (*cmdmod.ptr()).cmod_tab != 0 as c_int {
                    (*cmdmod.ptr()).cmod_tab
                } else if (*eap).addr_count == 0 as c_int {
                    0 as c_int
                } else {
                    (*eap).line2 as c_int + 1 as c_int
                },
                (*eap).arg,
                true_0 != 0,
                ::core::ptr::null_mut::<*mut win_T>(),
            )
            .is_null()
            {
                do_exedit(eap, old_curwin);
                apply_autocmds(
                    EVENT_TABNEWENTERED,
                    ::core::ptr::null_mut::<c_char>(),
                    ::core::ptr::null_mut::<c_char>(),
                    false_0 != 0,
                    curbuf.get(),
                );
                if curwin.get() != old_curwin
                    && win_valid(old_curwin) as c_int != 0
                    && (*old_curwin).w_buffer != curbuf.get()
                    && (*cmdmod.ptr()).cmod_flags & CMOD_KEEPALT as c_int == 0 as c_int
                {
                    (*old_curwin).w_alt_fnum = (*curbuf.get()).handle as c_int;
                }
            }
        } else if win_split(
            if (*eap).addr_count > 0 as c_int {
                (*eap).line2 as c_int
            } else {
                0 as c_int
            },
            if *(*eap).cmd as c_int == 'v' as c_int {
                WSP_VERT as c_int
            } else {
                0 as c_int
            },
        ) != FAIL
        {
            if *(*eap).arg as c_int != NUL {
                (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
                (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
            } else {
                do_check_scrollbind(false_0 != 0);
            }
            do_exedit(eap, old_curwin);
        }
    }
    xfree(fname as *mut c_void);
}
pub unsafe extern "C" fn tabpage_new() {
    let mut ea: exarg_T = exarg {
        arg: b"\0".as_ptr() as *const c_char as *mut c_char,
        args: ::core::ptr::null_mut::<*mut c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<c_char>(),
        cmd: b"tabn\0".as_ptr() as *const c_char as *mut c_char,
        cmdlinep: ::core::ptr::null_mut::<*mut c_char>(),
        cmdline_tofree: ::core::ptr::null_mut::<c_char>(),
        cmdidx: CMD_tabnew,
        argt: 0,
        skip: 0,
        forceit: 0,
        addr_count: 0,
        line1: 0,
        line2: 0,
        addr_type: ADDR_LINES,
        flags: 0,
        do_ecmd_cmd: ::core::ptr::null_mut::<c_char>(),
        do_ecmd_lnum: 0,
        append: 0,
        usefilter: 0,
        amount: 0,
        regname: 0,
        force_bin: 0,
        read_edit: 0,
        mkdir_p: 0,
        force_ff: 0,
        force_enc: 0,
        bad_char: 0,
        useridx: 0,
        errmsg: ::core::ptr::null_mut::<c_char>(),
        ea_getline: None,
        cookie: ::core::ptr::null_mut::<c_void>(),
        cstack: ::core::ptr::null_mut::<cstack_T>(),
    };
    ex_splitview(&raw mut ea);
}
unsafe extern "C" fn ex_tabnext(mut eap: *mut exarg_T) {
    let mut tab_number: c_int = 0;
    match (*eap).cmdidx as c_int {
        458 | 466 => {
            goto_tabpage(1 as c_int);
        }
        460 => {
            goto_tabpage(9999 as c_int);
        }
        464 | 465 => {
            if !(*eap).arg.is_null() && *(*eap).arg as c_int != NUL {
                let mut p: *mut c_char = (*eap).arg;
                let mut p_save: *mut c_char = p;
                tab_number = getdigits(&raw mut p, false_0 != 0, 0 as intmax_t) as c_int;
                if p == p_save
                    || *p_save as c_int == '-' as c_int
                    || *p_save as c_int == '+' as c_int
                    || *p as c_int != NUL
                    || tab_number == 0 as c_int
                {
                    (*eap).errmsg = ex_errmsg(&raw const e_invarg2 as *const c_char, (*eap).arg);
                    return;
                }
            } else if (*eap).addr_count == 0 as c_int {
                tab_number = 1 as c_int;
            } else {
                tab_number = (*eap).line2 as c_int;
                if tab_number < 1 as c_int {
                    (*eap).errmsg = gettext(&raw const e_invrange as *const c_char);
                    return;
                }
            }
            goto_tabpage(-tab_number);
        }
        _ => {
            tab_number = get_tabpage_arg(eap);
            if (*eap).errmsg.is_null() {
                goto_tabpage(tab_number);
            }
        }
    };
}
unsafe extern "C" fn ex_tabmove(mut eap: *mut exarg_T) {
    let mut tab_number: c_int = get_tabpage_arg(eap);
    if (*eap).errmsg.is_null() {
        tabpage_move(tab_number);
    }
}
unsafe extern "C" fn ex_tabs(mut _eap: *mut exarg_T) {
    let mut tabcount: c_int = 1 as c_int;
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const c_char);
    msg_start();
    msg_scroll.set(true_0);
    let mut lastused_win: *mut win_T = if valid_tabpage(lastused_tabpage.get()) as c_int != 0 {
        (*lastused_tabpage.get()).tp_curwin
    } else {
        ::core::ptr::null_mut::<win_T>()
    };
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        if got_int.get() {
            break;
        }
        if msg_col.get() > 0 as c_int {
            msg_putchar('\n' as c_int);
        }
        let c2rust_fresh1 = tabcount;
        tabcount = tabcount + 1;
        vim_snprintf(
            IObuff.ptr() as *mut c_char,
            IOSIZE as size_t,
            gettext(b"Tab page %d\0".as_ptr() as *const c_char),
            c2rust_fresh1,
        );
        msg_outtrans(IObuff.ptr() as *mut c_char, HLF_T as c_int, false_0 != 0);
        os_breakcheck();
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if got_int.get() {
                break;
            }
            if !(!(*wp).w_config.focusable || (*wp).w_config.hide as c_int != 0) {
                msg_putchar('\n' as c_int);
                msg_putchar(if wp == curwin.get() {
                    '>' as c_int
                } else if wp == lastused_win {
                    '#' as c_int
                } else {
                    ' ' as c_int
                });
                msg_putchar(' ' as c_int);
                msg_putchar(if bufIsChanged((*wp).w_buffer) as c_int != 0 {
                    '+' as c_int
                } else {
                    ' ' as c_int
                });
                msg_putchar(' ' as c_int);
                if !buf_spname((*wp).w_buffer).is_null() {
                    xstrlcpy(
                        IObuff.ptr() as *mut c_char,
                        buf_spname((*wp).w_buffer),
                        IOSIZE as size_t,
                    );
                } else {
                    home_replace(
                        (*wp).w_buffer,
                        (*(*wp).w_buffer).b_fname,
                        IObuff.ptr() as *mut c_char,
                        IOSIZE as size_t,
                        true_0 != 0,
                    );
                }
                msg_outtrans(IObuff.ptr() as *mut c_char, 0 as c_int, false_0 != 0);
                os_breakcheck();
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
unsafe extern "C" fn ex_detach(mut eap: *mut exarg_T) {
    if !eap.is_null() && (*eap).forceit != 0 {
        emsg(b"bang (!) not supported yet\0".as_ptr() as *const c_char);
    } else {
        if current_ui.get() == 0 {
            emsg(b"UI not attached\0".as_ptr() as *const c_char);
            return;
        }
        let mut chan: *mut Channel = find_channel(current_ui.get());
        if chan.is_null() {
            emsg(&raw const e_invchan as *const c_char);
            return;
        }
        let mut detach_err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<c_char>(),
        };
        nvim__chan_set_detach((*chan).id, true_0 != 0, &raw mut detach_err);
        api_clear_error(&raw mut detach_err);
        let mut err2: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<c_char>(),
        };
        remote_ui_disconnect((*chan).id, &raw mut err2, true_0 != 0);
        if err2.type_0 as c_int != kErrorTypeNone as c_int {
            emsg(err2.msg);
            api_clear_error(&raw mut err2);
            return;
        }
        let mut err: *const c_char = ::core::ptr::null::<c_char>();
        let mut rv: bool = channel_close((*chan).id, kChannelPartAll, &raw mut err);
        if !rv && !err.is_null() {
            emsg(err);
            return;
        }
        logmsg(
            LOGLVL_INF,
            ::core::ptr::null::<c_char>(),
            b"ex_detach\0".as_ptr() as *const c_char,
            6019 as c_int,
            true_0 != 0,
            b"detach current_ui=%ld\0".as_ptr() as *const c_char,
            (*chan).id,
        );
    };
}
unsafe extern "C" fn ex_connect(mut eap: *mut exarg_T) {
    let mut stop_server: bool = if (*eap).forceit != 0 {
        (ui_active() == 1 as size_t) as c_int
    } else {
        false_0
    } != 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<c_char>(),
    };
    remote_ui_connect(current_ui.get(), (*eap).arg, &raw mut err);
    if err.type_0 as c_int != kErrorTypeNone as c_int {
        emsg(err.msg);
        api_clear_error(&raw mut err);
        return;
    }
    ex_detach(::core::ptr::null_mut::<exarg_T>());
    if stop_server {
        exiting.set(true_0 != 0);
        getout(0 as c_int);
    }
}
unsafe extern "C" fn ex_mode(mut eap: *mut exarg_T) {
    if *(*eap).arg as c_int == NUL {
        must_redraw.set(UPD_CLEAR as c_int);
        ex_redraw(eap);
    } else {
        emsg(gettext(&raw const e_screenmode as *const c_char));
    };
}
unsafe extern "C" fn ex_resize(mut eap: *mut exarg_T) {
    let mut wp: *mut win_T = curwin.get();
    if (*eap).addr_count > 0 as c_int {
        let mut n: c_int = (*eap).line2 as c_int;
        wp = firstwin.get();
        while !(*wp).w_next.is_null() && {
            n -= 1;
            n > 0 as c_int
        } {
            wp = (*wp).w_next;
        }
    }
    let mut n_0: c_int = atol((*eap).arg) as c_int;
    if (*cmdmod.ptr()).cmod_split & WSP_VERT as c_int != 0 {
        if *(*eap).arg as c_int == '-' as c_int || *(*eap).arg as c_int == '+' as c_int {
            n_0 += (*wp).w_width;
        } else if n_0 == 0 as c_int && *(*eap).arg.offset(0 as c_int as isize) as c_int == NUL {
            n_0 = Columns.get();
        }
        win_setwidth_win(n_0, wp);
    } else {
        if *(*eap).arg as c_int == '-' as c_int || *(*eap).arg as c_int == '+' as c_int {
            n_0 += (*wp).w_height;
        } else if n_0 == 0 as c_int && *(*eap).arg.offset(0 as c_int as isize) as c_int == NUL {
            n_0 = Rows.get() - 1 as c_int;
        }
        win_setheight_win(n_0, wp);
    };
}
unsafe extern "C" fn ex_find(mut eap: *mut exarg_T) {
    if !check_can_set_curbuf_forceit((*eap).forceit) {
        return;
    }
    let mut fname: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if *get_findfunc() as c_int != NUL {
        fname = findfunc_find_file(
            (*eap).arg,
            strlen((*eap).arg),
            if (*eap).addr_count > 0 as c_int {
                (*eap).line2 as c_int
            } else {
                1 as c_int
            },
        );
    } else {
        let mut file_to_find: *mut c_char = ::core::ptr::null_mut::<c_char>();
        let mut search_ctx: *mut c_char = ::core::ptr::null_mut::<c_char>();
        fname = find_file_in_path(
            (*eap).arg,
            strlen((*eap).arg),
            FNAME_MESS as c_int,
            true_0,
            (*curbuf.get()).b_ffname,
            &raw mut file_to_find,
            &raw mut search_ctx,
        );
        if (*eap).addr_count > 0 as c_int {
            let mut count: linenr_T = (*eap).line2;
            while !fname.is_null() && {
                count -= 1;
                count > 0 as linenr_T
            } {
                xfree(fname as *mut c_void);
                fname = find_file_in_path(
                    ::core::ptr::null_mut::<c_char>(),
                    0 as size_t,
                    FNAME_MESS as c_int,
                    false_0,
                    (*curbuf.get()).b_ffname,
                    &raw mut file_to_find,
                    &raw mut search_ctx,
                );
            }
        }
        xfree(file_to_find as *mut c_void);
        vim_findfile_cleanup(search_ctx as *mut c_void);
    }
    if fname.is_null() {
        return;
    }
    (*eap).arg = fname;
    do_exedit(eap, ::core::ptr::null_mut::<win_T>());
    xfree(fname as *mut c_void);
}
unsafe extern "C" fn ex_edit(mut eap: *mut exarg_T) {
    let mut ffname: *mut c_char = if (*eap).cmdidx as c_int == CMD_enew as c_int {
        ::core::ptr::null_mut::<c_char>()
    } else {
        (*eap).arg
    };
    if (*eap).cmdidx as c_int != CMD_badd as c_int
        && (*eap).cmdidx as c_int != CMD_balt as c_int
        && (is_other_file(0 as c_int, ffname) as c_int != 0
            && !check_can_set_curbuf_forceit((*eap).forceit))
    {
        return;
    }
    if bt_prompt(curbuf.get()) as c_int != 0
        && (*eap).cmdidx as c_int == CMD_edit as c_int
        && *(*eap).arg as c_int == NUL
    {
        emsg(b"cannot :edit a prompt buffer\0".as_ptr() as *const c_char);
        return;
    }
    do_exedit(eap, ::core::ptr::null_mut::<win_T>());
}
pub unsafe extern "C" fn do_exedit(mut eap: *mut exarg_T, mut old_curwin: *mut win_T) {
    if exmode_active.get() as c_int != 0
        && ((*eap).cmdidx as c_int == CMD_visual as c_int
            || (*eap).cmdidx as c_int == CMD_view as c_int)
    {
        exmode_active.set(false_0 != 0);
        ex_pressedreturn.set(false_0 != 0);
        if ui_has(kUICmdline) {
            ui_ext_cmdline_block_leave();
        }
        if *(*eap).arg as c_int == NUL {
            if global_busy.get() != 0 {
                if !(*eap).nextcmd.is_null() {
                    stuffReadbuff((*eap).nextcmd);
                    (*eap).nextcmd = ::core::ptr::null_mut::<c_char>();
                }
                let save_rd: c_int = RedrawingDisabled.get();
                RedrawingDisabled.set(0 as c_int);
                let save_nwr: c_int = no_wait_return.get();
                no_wait_return.set(0 as c_int);
                need_wait_return.set(false_0 != 0);
                let save_ms: c_int = msg_scroll.get();
                msg_scroll.set(0 as c_int);
                redraw_all_later(UPD_NOT_VALID as c_int);
                pending_exmode_active.set(true_0 != 0);
                normal_enter(false_0 != 0, true_0 != 0);
                pending_exmode_active.set(false_0 != 0);
                RedrawingDisabled.set(save_rd);
                no_wait_return.set(save_nwr);
                msg_scroll.set(save_ms);
            }
            return;
        }
    }
    if ((*eap).cmdidx as c_int == CMD_new as c_int
        || (*eap).cmdidx as c_int == CMD_tabnew as c_int
        || (*eap).cmdidx as c_int == CMD_tabedit as c_int
        || (*eap).cmdidx as c_int == CMD_vnew as c_int)
        && *(*eap).arg as c_int == NUL
    {
        setpcmark();
        do_ecmd(
            0 as c_int,
            ::core::ptr::null_mut::<c_char>(),
            ::core::ptr::null_mut::<c_char>(),
            eap,
            ECMD_ONE as c_int as linenr_T,
            ECMD_HIDE as c_int
                + (if (*eap).forceit != 0 {
                    ECMD_FORCEIT as c_int
                } else {
                    0 as c_int
                }),
            if old_curwin.is_null() {
                curwin.get()
            } else {
                ::core::ptr::null_mut::<win_T>()
            },
        );
    } else if (*eap).cmdidx as c_int != CMD_split as c_int
        && (*eap).cmdidx as c_int != CMD_vsplit as c_int
        || *(*eap).arg as c_int != NUL
    {
        if *(*eap).arg as c_int != NUL && text_or_buf_locked() as c_int != 0 {
            return;
        }
        let mut n: c_int = readonlymode.get() as c_int;
        if (*eap).cmdidx as c_int == CMD_view as c_int
            || (*eap).cmdidx as c_int == CMD_sview as c_int
        {
            readonlymode.set(true_0 != 0);
        } else if (*eap).cmdidx as c_int == CMD_enew as c_int {
            readonlymode.set(false_0 != 0);
        }
        if (*eap).cmdidx as c_int != CMD_balt as c_int
            && (*eap).cmdidx as c_int != CMD_badd as c_int
        {
            setpcmark();
        }
        if do_ecmd(
            0 as c_int,
            if (*eap).cmdidx as c_int == CMD_enew as c_int {
                ::core::ptr::null_mut::<c_char>()
            } else {
                (*eap).arg
            },
            ::core::ptr::null_mut::<c_char>(),
            eap,
            (*eap).do_ecmd_lnum,
            (if buf_hide(curbuf.get()) as c_int != 0 {
                ECMD_HIDE as c_int
            } else {
                0 as c_int
            }) + (if (*eap).forceit != 0 {
                ECMD_FORCEIT as c_int
            } else {
                0 as c_int
            }) + (if !old_curwin.is_null() {
                ECMD_OLDBUF as c_int
            } else {
                0 as c_int
            }) + (if (*eap).cmdidx as c_int == CMD_badd as c_int {
                ECMD_ADDBUF as c_int
            } else {
                0 as c_int
            }) + (if (*eap).cmdidx as c_int == CMD_balt as c_int {
                ECMD_ALTBUF as c_int
            } else {
                0 as c_int
            }),
            if old_curwin.is_null() {
                curwin.get()
            } else {
                ::core::ptr::null_mut::<win_T>()
            },
        ) == FAIL
        {
            if !old_curwin.is_null() {
                let mut need_hide: bool =
                    curbufIsChanged() as c_int != 0 && (*curbuf.get()).b_nwindows <= 1 as c_int;
                if !need_hide || buf_hide(curbuf.get()) as c_int != 0 {
                    let mut cs: cleanup_T = cleanup_T {
                        pending: 0,
                        exception: ::core::ptr::null_mut::<except_T>(),
                    };
                    enter_cleanup(&raw mut cs);
                    win_close(
                        curwin.get(),
                        !need_hide && !buf_hide(curbuf.get()),
                        false_0 != 0,
                    );
                    leave_cleanup(&raw mut cs);
                }
            }
        } else if readonlymode.get() as c_int != 0 && (*curbuf.get()).b_nwindows == 1 as c_int {
            (*curbuf.get()).b_p_ro = true_0;
        }
        readonlymode.set(n != 0);
    } else {
        if !(*eap).do_ecmd_cmd.is_null() {
            do_cmdline_cmd((*eap).do_ecmd_cmd);
        }
        let mut n_0: c_int = (*curwin.get()).w_arg_idx_invalid;
        check_arg_idx(curwin.get());
        if n_0 != (*curwin.get()).w_arg_idx_invalid {
            maketitle();
        }
    }
    if !old_curwin.is_null()
        && *(*eap).arg as c_int != NUL
        && curwin.get() != old_curwin
        && win_valid(old_curwin) as c_int != 0
        && (*old_curwin).w_buffer != curbuf.get()
        && (*cmdmod.ptr()).cmod_flags & CMOD_KEEPALT as c_int == 0 as c_int
    {
        (*old_curwin).w_alt_fnum = (*curbuf.get()).handle as c_int;
    }
    ex_no_reprint.set(true_0 != 0);
}
unsafe extern "C" fn ex_nogui(mut eap: *mut exarg_T) {
    (*eap).errmsg = gettext(b"E25: Nvim does not have a built-in GUI\0".as_ptr() as *const c_char);
}
unsafe extern "C" fn ex_popup(mut eap: *mut exarg_T) {
    pum_make_popup((*eap).arg, (*eap).forceit);
}
unsafe extern "C" fn ex_swapname(mut _eap: *mut exarg_T) {
    if (*curbuf.get()).b_ml.ml_mfp.is_null() || (*(*curbuf.get()).b_ml.ml_mfp).mf_fname.is_null() {
        msg(
            gettext(b"No swap file\0".as_ptr() as *const c_char),
            0 as c_int,
        );
    } else {
        msg((*(*curbuf.get()).b_ml.ml_mfp).mf_fname, 0 as c_int);
    };
}
unsafe extern "C" fn ex_syncbind(mut _eap: *mut exarg_T) {
    let mut vtopline: linenr_T = 0;
    let mut old_linenr: linenr_T = (*curwin.get()).w_cursor.lnum;
    setpcmark();
    if (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
        vtopline = get_vtopline(curwin.get()) as linenr_T;
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_onebuf_opt.wo_scb != 0 && !(*wp).w_buffer.is_null() {
                let mut y: linenr_T =
                    plines_m_win_fill(wp, 1 as linenr_T, (*(*wp).w_buffer).b_ml.ml_line_count)
                        as linenr_T
                        - get_scrolloff_value(curwin.get()) as linenr_T;
                vtopline = if vtopline < y { vtopline } else { y };
            }
            wp = (*wp).w_next;
        }
        vtopline = if vtopline > 1 as linenr_T {
            vtopline
        } else {
            1 as linenr_T
        };
    } else {
        vtopline = 1 as c_int as linenr_T;
    }
    let mut wp_0: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp_0.is_null() {
        if (*wp_0).w_onebuf_opt.wo_scb != 0 {
            let mut y_0: c_int = vtopline as c_int - get_vtopline(wp_0);
            if y_0 > 0 as c_int {
                scrollup(wp_0, y_0 as linenr_T, true_0 != 0);
            } else {
                scrolldown(wp_0, -(y_0 as linenr_T), true_0);
            }
            (*wp_0).w_scbind_pos = vtopline as c_int;
            redraw_later(wp_0, UPD_VALID as c_int);
            cursor_correct(wp_0);
            (*wp_0).w_redr_status = true_0 != 0;
        }
        wp_0 = (*wp_0).w_next;
    }
    if (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
        did_syncbind.set(true_0 != 0);
        checkpcmark();
        if old_linenr != (*curwin.get()).w_cursor.lnum {
            let mut ctrl_o: [c_char; 2] = [0; 2];
            ctrl_o[0 as c_int as usize] = Ctrl_O as c_char;
            ctrl_o[1 as c_int as usize] = 0 as c_char;
            ins_typebuf(
                &raw mut ctrl_o as *mut c_char,
                REMAP_NONE as c_int,
                0 as c_int,
                true_0 != 0,
                false_0 != 0,
            );
        }
    }
}
unsafe extern "C" fn ex_read(mut eap: *mut exarg_T) {
    let mut empty: c_int = (*curbuf.get()).b_ml.ml_flags & ML_EMPTY;
    if (*eap).usefilter != 0 {
        do_bang(1 as c_int, eap, false_0 != 0, false_0 != 0, true_0 != 0);
        return;
    }
    if u_save((*eap).line2, (*eap).line2 + 1 as linenr_T) == FAIL {
        return;
    }
    let mut i: c_int = 0;
    if *(*eap).arg as c_int == NUL {
        if check_fname() == FAIL {
            return;
        }
        i = readfile(
            (*curbuf.get()).b_ffname,
            (*curbuf.get()).b_fname,
            (*eap).line2,
            0 as linenr_T,
            MAXLNUM as c_int as linenr_T,
            eap,
            0 as c_int,
            false_0 != 0,
        );
    } else {
        if !vim_strchr(p_cpo.get(), CPO_ALTREAD).is_null() {
            setaltfname((*eap).arg, (*eap).arg, 1 as linenr_T);
        }
        i = readfile(
            (*eap).arg,
            ::core::ptr::null_mut::<c_char>(),
            (*eap).line2,
            0 as linenr_T,
            MAXLNUM as c_int as linenr_T,
            eap,
            0 as c_int,
            false_0 != 0,
        );
    }
    if i != OK {
        if !aborting() {
            semsg(gettext(&raw const e_notopen as *const c_char), (*eap).arg);
        }
    } else {
        if empty != 0 && exmode_active.get() as c_int != 0 {
            let mut lnum: linenr_T = 0;
            if (*eap).line2 == 0 as linenr_T {
                lnum = (*curbuf.get()).b_ml.ml_line_count;
            } else {
                lnum = 1 as c_int as linenr_T;
            }
            if *ml_get(lnum) as c_int == NUL && u_savedel(lnum, 1 as linenr_T) == OK {
                ml_delete(lnum);
                if (*curwin.get()).w_cursor.lnum > 1 as linenr_T
                    && (*curwin.get()).w_cursor.lnum >= lnum
                {
                    (*curwin.get()).w_cursor.lnum -= 1;
                }
                deleted_lines_mark(lnum, 1 as c_int);
            }
        }
        redraw_curbuf_later(UPD_VALID as c_int);
    };
}
static prev_dir: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
unsafe extern "C" fn get_prevdir(mut scope: CdScope) -> *mut c_char {
    match scope as c_int {
        1 => return (*curtab.get()).tp_prevdir,
        0 => return (*curwin.get()).w_prevdir,
        _ => return prev_dir.get(),
    };
}
unsafe extern "C" fn post_chdir(mut scope: CdScope, mut trigger_dirchanged: bool) {
    let mut ptr_: *mut *mut c_void = &raw mut (*curwin.get()).w_localdir as *mut *mut c_void;
    xfree(*ptr_);
    *ptr_ = NULL_1;
    let _ = *ptr_;
    if scope as c_int >= kCdScopeTabpage as c_int {
        let mut ptr__0: *mut *mut c_void = &raw mut (*curtab.get()).tp_localdir as *mut *mut c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_1;
        let _ = *ptr__0;
    }
    if (scope as c_int) < kCdScopeGlobal as c_int {
        let mut pdir: *mut c_char = get_prevdir(scope);
        if (*globaldir.ptr()).is_null() && !pdir.is_null() {
            globaldir.set(xstrdup(pdir));
        }
    }
    let mut cwd: [c_char; 4096] = [0; 4096];
    if os_dirname(&raw mut cwd as *mut c_char, MAXPATHL as size_t) != OK {
        return;
    }
    match scope as c_int {
        2 => {
            let mut ptr__1: *mut *mut c_void = globaldir.ptr() as *mut *mut c_void;
            xfree(*ptr__1);
            *ptr__1 = NULL_1;
            let _ = *ptr__1;
        }
        1 => {
            (*curtab.get()).tp_localdir = xstrdup(&raw mut cwd as *mut c_char);
        }
        0 => {
            (*curwin.get()).w_localdir = xstrdup(&raw mut cwd as *mut c_char);
        }
        -1 => {
            abort();
        }
        _ => {}
    }
    last_chdir_reason.set(::core::ptr::null_mut::<c_char>());
    shorten_fnames(vim_strchr(p_cpo.get(), CPO_NOSYMLINKS).is_null() as c_int);
    if trigger_dirchanged {
        do_autocmd_dirchanged(
            &raw mut cwd as *mut c_char,
            scope,
            kCdCauseManual,
            false_0 != 0,
        );
    }
}
pub unsafe extern "C" fn changedir_func(mut new_dir: *mut c_char, mut scope: CdScope) -> bool {
    if new_dir.is_null() || allbuf_locked() as c_int != 0 {
        return false_0 != 0;
    }
    let mut pdir: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if strcmp(new_dir, b"-\0".as_ptr() as *const c_char) == 0 as c_int {
        pdir = get_prevdir(scope);
        if pdir.is_null() {
            emsg(gettext(
                b"E186: No previous directory\0".as_ptr() as *const c_char
            ));
            return false_0 != 0;
        }
        new_dir = pdir;
    }
    if os_dirname(NameBuff.ptr() as *mut c_char, MAXPATHL as size_t) == OK {
        pdir = xstrdup(NameBuff.ptr() as *mut c_char);
    } else {
        pdir = ::core::ptr::null_mut::<c_char>();
    }
    if *new_dir as c_int == NUL && p_cdh.get() != 0 {
        expand_env(
            b"$HOME\0".as_ptr() as *const c_char as *mut c_char,
            NameBuff.ptr() as *mut c_char,
            MAXPATHL,
        );
        new_dir = NameBuff.ptr() as *mut c_char;
    }
    let mut dir_differs: bool = pdir.is_null() || pathcmp(pdir, new_dir, -1 as c_int) != 0 as c_int;
    if dir_differs {
        do_autocmd_dirchanged(new_dir, scope, kCdCauseManual, true_0 != 0);
        if vim_chdir(new_dir) != 0 as c_int {
            emsg(gettext(&raw const e_failed as *const c_char));
            xfree(pdir as *mut c_void);
            return false_0 != 0;
        }
    }
    let mut pp: *mut *mut c_char = ::core::ptr::null_mut::<*mut c_char>();
    match scope as c_int {
        1 => {
            pp = &raw mut (*curtab.get()).tp_prevdir;
        }
        0 => {
            pp = &raw mut (*curwin.get()).w_prevdir;
        }
        _ => {
            pp = prev_dir.ptr();
        }
    }
    xfree(*pp as *mut c_void);
    *pp = pdir;
    post_chdir(scope, dir_differs);
    return true_0 != 0;
}
pub unsafe extern "C" fn ex_cd(mut eap: *mut exarg_T) {
    let mut new_dir: *mut c_char = (*eap).arg;
    if *new_dir as c_int == NUL && p_cdh.get() == 0 {
        ex_pwd(::core::ptr::null_mut::<exarg_T>());
        return;
    }
    let mut scope: CdScope = kCdScopeGlobal;
    match (*eap).cmdidx as c_int {
        448 | 449 => {
            scope = kCdScopeTabpage;
        }
        225 | 226 => {
            scope = kCdScopeWindow;
        }
        _ => {}
    }
    if changedir_func(new_dir, scope) {
        if KeyTyped.get() as c_int != 0 || p_verbose.get() >= 5 as OptInt {
            ex_pwd(eap);
        }
    }
}
unsafe extern "C" fn ex_pwd(mut _eap: *mut exarg_T) {
    if os_dirname(NameBuff.ptr() as *mut c_char, MAXPATHL as size_t) == OK {
        if p_verbose.get() > 0 as OptInt {
            let mut context: *mut c_char = b"global\0".as_ptr() as *const c_char as *mut c_char;
            if !(*last_chdir_reason.ptr()).is_null() {
                context = last_chdir_reason.get();
            } else if !(*curwin.get()).w_localdir.is_null() {
                context = b"window\0".as_ptr() as *const c_char as *mut c_char;
            } else if !(*curtab.get()).tp_localdir.is_null() {
                context = b"tabpage\0".as_ptr() as *const c_char as *mut c_char;
            }
            smsg(
                0 as c_int,
                b"[%s] %s\0".as_ptr() as *const c_char,
                context,
                NameBuff.ptr() as *mut c_char,
            );
        } else {
            msg(NameBuff.ptr() as *mut c_char, 0 as c_int);
        }
    } else {
        emsg(gettext(b"E187: Unknown\0".as_ptr() as *const c_char));
    };
}
unsafe extern "C" fn ex_equal(mut eap: *mut exarg_T) {
    if *(*eap).arg as c_int != NUL && *(*eap).arg as c_int != '|' as c_int {
        ex_lua(eap);
    } else {
        (*eap).nextcmd = find_nextcmd((*eap).arg);
        smsg(
            0 as c_int,
            b"%ld\0".as_ptr() as *const c_char,
            (*eap).line2 as int64_t,
        );
    };
}
unsafe extern "C" fn ex_sleep(mut eap: *mut exarg_T) {
    if cursor_valid(curwin.get()) != 0 {
        setcursor_mayforce(curwin.get(), true_0 != 0);
    }
    let mut len: int64_t = (*eap).line2 as int64_t;
    match *(*eap).arg as c_int {
        109 => {}
        NUL => {
            len *= 1000 as int64_t;
        }
        _ => {
            semsg(gettext(&raw const e_invarg2 as *const c_char), (*eap).arg);
            return;
        }
    }
    do_sleep(len, (*eap).forceit != 0);
}
pub unsafe extern "C" fn do_sleep(mut msec: int64_t, mut hide_cursor: bool) {
    if hide_cursor {
        ui_busy_start();
    }
    ui_flush();
    process_events_until(main_loop.ptr(), (*main_loop.ptr()).events, msec, || {
        got_int.get()
    });
    if got_int.get() {
        vpeekc();
    }
    if hide_cursor {
        ui_busy_stop();
    }
}
unsafe extern "C" fn ex_winsize(mut eap: *mut exarg_T) {
    let mut arg: *mut c_char = (*eap).arg;
    if !ascii_isdigit(*arg as c_int) {
        semsg(gettext(&raw const e_invarg2 as *const c_char), arg);
        return;
    }
    let mut w: c_int = getdigits_int(&raw mut arg, false_0 != 0, 10 as c_int);
    arg = skipwhite(arg);
    let mut p: *mut c_char = arg;
    let mut h: c_int = getdigits_int(&raw mut arg, false_0 != 0, 10 as c_int);
    if *p as c_int != NUL && *arg as c_int == NUL {
        screen_resize(w, h);
    } else {
        emsg(gettext(
            b"E465: :winsize requires two number arguments\0".as_ptr() as *const c_char,
        ));
    };
}
unsafe extern "C" fn ex_wincmd(mut eap: *mut exarg_T) {
    let mut xchar: c_int = NUL;
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if *(*eap).arg as c_int == 'g' as c_int || *(*eap).arg as c_int == Ctrl_G {
        if *(*eap).arg.offset(1 as c_int as isize) as c_int == NUL {
            emsg(gettext(&raw const e_invarg as *const c_char));
            return;
        }
        xchar = *(*eap).arg.offset(1 as c_int as isize) as uint8_t as c_int;
        p = (*eap).arg.offset(2 as c_int as isize);
    } else {
        p = (*eap).arg.offset(1 as c_int as isize);
    }
    (*eap).nextcmd = check_nextcmd(p);
    p = skipwhite(p);
    if *p as c_int != NUL && *p as c_int != '"' as c_int && (*eap).nextcmd.is_null() {
        emsg(gettext(&raw const e_invarg as *const c_char));
    } else if (*eap).skip == 0 {
        postponed_split_flags.set((*cmdmod.ptr()).cmod_split);
        postponed_split_tab.set((*cmdmod.ptr()).cmod_tab);
        do_window(
            *(*eap).arg as c_int,
            if (*eap).addr_count > 0 as c_int {
                (*eap).line2 as c_int
            } else {
                0 as c_int
            },
            xchar,
        );
        postponed_split_flags.set(0 as c_int);
        postponed_split_tab.set(0 as c_int);
    }
}
unsafe extern "C" fn ex_operators(mut eap: *mut exarg_T) {
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
    clear_oparg(&raw mut oa);
    oa.regname = (*eap).regname;
    oa.start.lnum = (*eap).line1;
    oa.end.lnum = (*eap).line2;
    oa.line_count = (*eap).line2 - (*eap).line1 + 1 as linenr_T;
    oa.motion_type = kMTLineWise;
    virtual_op.set(kFalse);
    if (*eap).cmdidx as c_int != CMD_yank as c_int {
        setpcmark();
        (*curwin.get()).w_cursor.lnum = (*eap).line1;
        beginline(BL_SOL as c_int | BL_FIX as c_int);
    }
    if VIsual_active.get() {
        end_visual_mode();
    }
    match (*eap).cmdidx as c_int {
        109 => {
            oa.op_type = OP_DELETE as c_int;
            op_delete(&raw mut oa);
        }
        546 => {
            oa.op_type = OP_YANK as c_int;
            op_yank(&raw mut oa, true_0 != 0);
        }
        _ => {
            if ((*eap).cmdidx as c_int == CMD_rshift as c_int) as c_int
                ^ (*curwin.get()).w_onebuf_opt.wo_rl
                != 0
            {
                oa.op_type = OP_RSHIFT as c_int;
            } else {
                oa.op_type = OP_LSHIFT as c_int;
            }
            op_shift(&raw mut oa, false_0 != 0, (*eap).amount);
        }
    }
    virtual_op.set(kNone);
    ex_may_print(eap);
}
unsafe extern "C" fn ex_put(mut eap: *mut exarg_T) {
    if (*eap).line2 == 0 as linenr_T {
        (*eap).line2 = 1 as c_int as linenr_T;
        (*eap).forceit = true_0;
    }
    (*curwin.get()).w_cursor.lnum = (*eap).line2;
    check_cursor_col(curwin.get());
    do_put(
        (*eap).regname,
        ::core::ptr::null_mut::<yankreg_T>(),
        if (*eap).forceit != 0 {
            BACKWARD as c_int
        } else {
            FORWARD as c_int
        },
        1 as c_int,
        PUT_LINE as c_int | PUT_CURSLINE as c_int,
    );
}
unsafe extern "C" fn ex_iput(mut eap: *mut exarg_T) {
    if (*eap).line2 == 0 as linenr_T {
        (*eap).line2 = 1 as c_int as linenr_T;
        (*eap).forceit = true_0;
    }
    (*curwin.get()).w_cursor.lnum = (*eap).line2;
    check_cursor_col(curwin.get());
    do_put(
        (*eap).regname,
        ::core::ptr::null_mut::<yankreg_T>(),
        if (*eap).forceit != 0 {
            BACKWARD as c_int
        } else {
            FORWARD as c_int
        },
        1 as c_int,
        PUT_LINE as c_int | PUT_CURSLINE as c_int | PUT_FIXINDENT as c_int,
    );
}
unsafe extern "C" fn ex_copymove(mut eap: *mut exarg_T) {
    let mut errormsg: *const c_char = ::core::ptr::null::<c_char>();
    let mut n: linenr_T = get_address(
        eap,
        &raw mut (*eap).arg,
        (*eap).addr_type,
        false_0 != 0,
        false_0 != 0,
        false_0,
        1 as c_int,
        &raw mut errormsg,
    );
    if (*eap).arg.is_null() {
        if !errormsg.is_null() {
            emsg(errormsg);
        }
        (*eap).nextcmd = ::core::ptr::null_mut::<c_char>();
        return;
    }
    get_flags(eap);
    if n == MAXLNUM as c_int as linenr_T
        || n < 0 as linenr_T
        || n > (*curbuf.get()).b_ml.ml_line_count
    {
        emsg(gettext(&raw const e_invrange as *const c_char));
        return;
    }
    if (*eap).cmdidx as c_int == CMD_move as c_int {
        if do_move((*eap).line1, (*eap).line2, n) == FAIL {
            return;
        }
    } else {
        ex_copy((*eap).line1, (*eap).line2, n);
    }
    u_clearline(curbuf.get());
    beginline(BL_SOL as c_int | BL_FIX as c_int);
    ex_may_print(eap);
}
pub unsafe extern "C" fn ex_may_print(mut eap: *mut exarg_T) {
    if (*eap).flags != 0 as c_int {
        print_line(
            (*curwin.get()).w_cursor.lnum,
            (*eap).flags & EXFLAG_NR != 0,
            (*eap).flags & EXFLAG_LIST != 0,
            true_0 != 0,
        );
        ex_no_reprint.set(true_0 != 0);
    }
}
unsafe extern "C" fn ex_submagic(mut eap: *mut exarg_T) {
    let saved: optmagic_T = magic_overruled.get();
    magic_overruled.set(
        (if (*eap).cmdidx as c_int == CMD_smagic as c_int {
            OPTION_MAGIC_ON as c_int
        } else {
            OPTION_MAGIC_OFF as c_int
        }) as optmagic_T,
    );
    ex_substitute(eap);
    magic_overruled.set(saved);
}
unsafe extern "C" fn ex_submagic_preview(
    mut eap: *mut exarg_T,
    mut cmdpreview_ns: c_int,
    mut cmdpreview_bufnr: handle_T,
) -> c_int {
    let saved: optmagic_T = magic_overruled.get();
    magic_overruled.set(
        (if (*eap).cmdidx as c_int == CMD_smagic as c_int {
            OPTION_MAGIC_ON as c_int
        } else {
            OPTION_MAGIC_OFF as c_int
        }) as optmagic_T,
    );
    let mut retv: c_int = ex_substitute_preview(eap, cmdpreview_ns, cmdpreview_bufnr);
    magic_overruled.set(saved);
    return retv;
}
unsafe extern "C" fn ex_join(mut eap: *mut exarg_T) {
    (*curwin.get()).w_cursor.lnum = (*eap).line1;
    if (*eap).line1 == (*eap).line2 {
        if (*eap).addr_count >= 2 as c_int {
            return;
        }
        if (*eap).line2 == (*curbuf.get()).b_ml.ml_line_count {
            beep_flush();
            return;
        }
        (*eap).line2 += 1;
    }
    do_join(
        ((*eap).line2 as ssize_t - (*eap).line1 as ssize_t + 1 as ssize_t) as size_t,
        (*eap).forceit == 0,
        true_0 != 0,
        true_0 != 0,
        true_0 != 0,
    );
    beginline(BL_WHITE as c_int | BL_FIX as c_int);
    ex_may_print(eap);
}
unsafe extern "C" fn ex_at(mut eap: *mut exarg_T) {
    let mut prev_len: c_int = (*typebuf.ptr()).tb_len;
    (*curwin.get()).w_cursor.lnum = (*eap).line2;
    check_cursor_col(curwin.get());
    let mut c: c_int = *(*eap).arg as uint8_t as c_int;
    if c == NUL {
        c = '@' as c_int;
    }
    if do_execreg(
        c,
        true_0,
        !vim_strchr(p_cpo.get(), CPO_EXECBUF).is_null() as c_int,
        true_0,
    ) == FAIL
    {
        beep_flush();
        return;
    }
    let save_efr: bool = exec_from_reg.get();
    exec_from_reg.set(true_0 != 0);
    while !stuff_empty() || (*typebuf.ptr()).tb_len > prev_len {
        do_cmdline(
            ::core::ptr::null_mut::<c_char>(),
            Some(getexline as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char),
            NULL_1,
            DOCMD_NOWAIT as c_int | DOCMD_VERBOSE as c_int,
        );
    }
    exec_from_reg.set(save_efr);
}
unsafe extern "C" fn ex_bang(mut eap: *mut exarg_T) {
    do_bang(
        (*eap).addr_count,
        eap,
        (*eap).forceit != 0,
        true_0 != 0,
        true_0 != 0,
    );
}
unsafe extern "C" fn ex_undo(mut eap: *mut exarg_T) {
    if (*eap).addr_count != 1 as c_int {
        if (*eap).forceit != 0 {
            u_undo_and_forget(1 as c_int, true_0 != 0);
        } else {
            u_undo(1 as c_int);
        }
        return;
    }
    let mut step: linenr_T = (*eap).line2;
    if (*eap).forceit != 0 {
        if step >= (*curbuf.get()).b_u_seq_cur as linenr_T {
            emsg(gettext(
                &raw const e_undobang_cannot_redo_or_move_branch as *const c_char,
            ));
            return;
        }
        let mut uhp: *mut u_header_T = ::core::ptr::null_mut::<u_header_T>();
        let mut count: c_int = 0 as c_int;
        uhp = if !(*curbuf.get()).b_u_curhead.is_null() {
            (*curbuf.get()).b_u_curhead
        } else {
            (*curbuf.get()).b_u_newhead
        };
        while !uhp.is_null() && (*uhp).uh_seq as linenr_T > step {
            uhp = (*uhp).uh_next.ptr;
            count += 1;
        }
        if step != 0 as linenr_T && (uhp.is_null() || ((*uhp).uh_seq as linenr_T) < step) {
            emsg(gettext(
                &raw const e_undobang_cannot_redo_or_move_branch as *const c_char,
            ));
            return;
        }
        u_undo_and_forget(count, true_0 != 0);
    } else {
        undo_time(step as c_int, false_0 != 0, false_0 != 0, true_0 != 0);
    };
}
unsafe extern "C" fn ex_wundo(mut eap: *mut exarg_T) {
    let mut hash: [uint8_t; 32] = [0; 32];
    u_compute_hash(curbuf.get(), &raw mut hash as *mut uint8_t);
    u_write_undo(
        (*eap).arg,
        (*eap).forceit != 0,
        curbuf.get(),
        &raw mut hash as *mut uint8_t,
    );
}
unsafe extern "C" fn ex_rundo(mut eap: *mut exarg_T) {
    let mut hash: [uint8_t; 32] = [0; 32];
    u_compute_hash(curbuf.get(), &raw mut hash as *mut uint8_t);
    u_read_undo(
        (*eap).arg,
        &raw mut hash as *mut uint8_t,
        ::core::ptr::null::<c_char>(),
    );
}
unsafe extern "C" fn ex_redo(mut _eap: *mut exarg_T) {
    u_redo(1 as c_int);
}
unsafe extern "C" fn ex_later(mut eap: *mut exarg_T) {
    let mut count: c_int = 0 as c_int;
    let mut sec: bool = false_0 != 0;
    let mut file: bool = false_0 != 0;
    let mut p: *mut c_char = (*eap).arg;
    if *p as c_int == NUL {
        count = 1 as c_int;
    } else if *(*__ctype_b_loc()).offset(*p as uint8_t as c_int as isize) as c_int
        & _ISdigit as c_int as c_ushort as c_int
        != 0
    {
        count = getdigits_int(&raw mut p, false_0 != 0, 0 as c_int);
        match *p as c_int {
            115 => {
                p = p.offset(1);
                sec = true_0 != 0;
            }
            109 => {
                p = p.offset(1);
                sec = true_0 != 0;
                count *= 60 as c_int;
            }
            104 => {
                p = p.offset(1);
                sec = true_0 != 0;
                count *= 60 as c_int * 60 as c_int;
            }
            100 => {
                p = p.offset(1);
                sec = true_0 != 0;
                count *= 24 as c_int * 60 as c_int * 60 as c_int;
            }
            102 => {
                p = p.offset(1);
                file = true_0 != 0;
            }
            _ => {}
        }
    }
    if *p as c_int != NUL {
        semsg(gettext(&raw const e_invarg2 as *const c_char), (*eap).arg);
    } else {
        undo_time(
            if (*eap).cmdidx as c_int == CMD_earlier as c_int {
                -count
            } else {
                count
            },
            sec,
            file,
            false_0 != 0,
        );
    };
}
unsafe extern "C" fn ex_redir(mut eap: *mut exarg_T) {
    let mut arg: *mut c_char = (*eap).arg;
    if strcasecmp(
        (*eap).arg,
        b"END\0".as_ptr() as *const c_char as *mut c_char,
    ) == 0 as c_int
    {
        close_redir();
    } else if *arg as c_int == '>' as c_int {
        arg = arg.offset(1);
        let mut mode: *mut c_char = ::core::ptr::null_mut::<c_char>();
        if *arg as c_int == '>' as c_int {
            arg = arg.offset(1);
            mode = b"a\0".as_ptr() as *const c_char as *mut c_char;
        } else {
            mode = b"w\0".as_ptr() as *const c_char as *mut c_char;
        }
        arg = skipwhite(arg);
        close_redir();
        let mut fname: *mut c_char = expand_env_save(arg);
        if fname.is_null() {
            return;
        }
        redir_fd.set(open_exfile(fname, (*eap).forceit, mode));
        xfree(fname as *mut c_void);
    } else if *arg as c_int == '@' as c_int {
        close_redir();
        arg = arg.offset(1);
        if valid_yank_reg(*arg as c_int, true_0 != 0) as c_int != 0 && *arg as c_int != '_' as c_int
        {
            let c2rust_fresh15 = arg;
            arg = arg.offset(1);
            redir_reg.set(*c2rust_fresh15 as uint8_t as c_int);
            if *arg as c_int == '>' as c_int
                && *arg.offset(1 as c_int as isize) as c_int == '>' as c_int
            {
                arg = arg.offset(2 as c_int as isize);
            } else {
                if *arg as c_int == '>' as c_int {
                    arg = arg.offset(1);
                }
                if *arg as c_int == NUL
                    && *(*__ctype_b_loc()).offset(redir_reg.get() as isize) as c_int
                        & _ISupper as c_int as c_ushort as c_int
                        == 0
                {
                    write_reg_contents(
                        redir_reg.get(),
                        b"\0".as_ptr() as *const c_char,
                        0 as ssize_t,
                        false_0,
                    );
                }
            }
        }
        if *arg as c_int != NUL {
            redir_reg.set(0 as c_int);
            semsg(gettext(&raw const e_invarg2 as *const c_char), (*eap).arg);
        }
    } else if *arg as c_int == '=' as c_int
        && *arg.offset(1 as c_int as isize) as c_int == '>' as c_int
    {
        let mut append: bool = false;
        close_redir();
        arg = arg.offset(2 as c_int as isize);
        if *arg as c_int == '>' as c_int {
            arg = arg.offset(1);
            append = true_0 != 0;
        } else {
            append = false_0 != 0;
        }
        if var_redir_start(skipwhite(arg), append) == OK {
            redir_vname.set(true_0 != 0);
        }
    } else {
        semsg(gettext(&raw const e_invarg2 as *const c_char), (*eap).arg);
    }
    if !(*redir_fd.ptr()).is_null() || redir_reg.get() != 0 || redir_vname.get() as c_int != 0 {
        redir_off.set(false_0 != 0);
    }
}
unsafe extern "C" fn ex_redraw(mut eap: *mut exarg_T) {
    if cmdpreview.get() {
        return;
    }
    let mut r: c_int = RedrawingDisabled.get();
    let mut p: c_int = p_lz.get();
    RedrawingDisabled.set(0 as c_int);
    p_lz.set(false_0);
    validate_cursor(curwin.get());
    update_topline(curwin.get());
    if (*eap).forceit != 0 {
        redraw_all_later(UPD_NOT_VALID as c_int);
        redraw_cmdline.set(true_0 != 0);
    } else if VIsual_active.get() {
        redraw_curbuf_later(UPD_INVERTED as c_int);
    }
    update_screen();
    if need_maketitle.get() {
        maketitle();
    }
    RedrawingDisabled.set(r);
    p_lz.set(p);
    msg_didout.set(false_0 != 0);
    msg_col.set(0 as c_int);
    need_wait_return.set(false_0 != 0);
    ui_flush();
}
unsafe extern "C" fn ex_redrawstatus(mut eap: *mut exarg_T) {
    if cmdpreview.get() {
        return;
    }
    let mut r: c_int = RedrawingDisabled.get();
    let mut p: c_int = p_lz.get();
    if (*eap).forceit != 0 {
        status_redraw_all();
    } else {
        status_redraw_curbuf();
    }
    RedrawingDisabled.set(0 as c_int);
    p_lz.set(false_0);
    if State.get() & MODE_CMDLINE as c_int != 0 {
        redraw_statuslines();
    } else {
        if VIsual_active.get() {
            redraw_curbuf_later(UPD_INVERTED as c_int);
        }
        update_screen();
    }
    RedrawingDisabled.set(r);
    p_lz.set(p);
    ui_flush();
}
unsafe extern "C" fn ex_redrawtabline(mut _eap: *mut exarg_T) {
    let r: c_int = RedrawingDisabled.get();
    let p: c_int = p_lz.get();
    RedrawingDisabled.set(0 as c_int);
    p_lz.set(false_0);
    draw_tabline();
    RedrawingDisabled.set(r);
    p_lz.set(p);
    ui_flush();
}
unsafe extern "C" fn close_redir() {
    if !(*redir_fd.ptr()).is_null() {
        fclose(redir_fd.get());
        redir_fd.set(::core::ptr::null_mut::<FILE>());
    }
    redir_reg.set(0 as c_int);
    if redir_vname.get() {
        var_redir_stop();
        redir_vname.set(false_0 != 0);
    }
}
pub unsafe extern "C" fn vim_mkdir_emsg(name: *const c_char, prot: c_int) -> c_int {
    let mut ret: c_int = 0;
    ret = os_mkdir(name, prot as int32_t);
    if ret != 0 as c_int {
        semsg(
            gettext(&raw const e_mkdir as *const c_char),
            name,
            uv_strerror(ret),
        );
        return FAIL;
    }
    return OK;
}
pub unsafe extern "C" fn open_exfile(
    mut fname: *mut c_char,
    mut forceit: c_int,
    mut mode: *mut c_char,
) -> *mut FILE {
    if os_isdir(fname) {
        semsg(gettext(&raw const e_isadir2 as *const c_char), fname);
        return ::core::ptr::null_mut::<FILE>();
    }
    if forceit == 0 && *mode as c_int != 'a' as c_int && os_path_exists(fname) as c_int != 0 {
        semsg(
            gettext(b"E189: \"%s\" exists (add ! to override)\0".as_ptr() as *const c_char),
            fname,
        );
        return ::core::ptr::null_mut::<FILE>();
    }
    let mut fd: *mut FILE = ::core::ptr::null_mut::<FILE>();
    fd = os_fopen(fname, mode);
    if fd.is_null() {
        semsg(
            gettext(b"E190: Cannot open \"%s\" for writing\0".as_ptr() as *const c_char),
            fname,
        );
    }
    return fd;
}
unsafe extern "C" fn ex_mark(mut eap: *mut exarg_T) {
    if *(*eap).arg as c_int == NUL {
        emsg(gettext(&raw const e_argreq as *const c_char));
        return;
    }
    if *(*eap).arg.offset(1 as c_int as isize) as c_int != NUL {
        semsg(
            gettext(&raw const e_trailing_arg as *const c_char),
            (*eap).arg,
        );
        return;
    }
    let mut pos: pos_T = (*curwin.get()).w_cursor;
    (*curwin.get()).w_cursor.lnum = (*eap).line2;
    beginline(BL_WHITE as c_int | BL_FIX as c_int);
    if setmark(*(*eap).arg as c_int) == FAIL {
        emsg(gettext(
            b"E191: Argument must be a letter or forward/backward quote\0".as_ptr()
                as *const c_char,
        ));
    }
    (*curwin.get()).w_cursor = pos;
}
pub unsafe extern "C" fn update_topline_cursor() {
    check_cursor(curwin.get());
    update_topline(curwin.get());
    if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
        validate_cursor(curwin.get());
    }
    update_curswant();
}
pub unsafe extern "C" fn save_current_state(mut sst: *mut save_state_T) -> bool {
    (*sst).save_msg_scroll = msg_scroll.get();
    (*sst).save_restart_edit = restart_edit.get();
    (*sst).save_msg_didout = msg_didout.get();
    (*sst).save_State = State.get();
    (*sst).save_finish_op = finish_op.get();
    (*sst).save_opcount = opcount.get();
    (*sst).save_reg_executing = reg_executing.get();
    (*sst).save_pending_end_reg_executing = pending_end_reg_executing.get();
    msg_scroll.set(false_0);
    restart_edit.set(0 as c_int);
    save_typeahead(&raw mut (*sst).tabuf);
    return (*sst).tabuf.typebuf_valid;
}
pub unsafe extern "C" fn restore_current_state(mut sst: *mut save_state_T) {
    restore_typeahead(&raw mut (*sst).tabuf);
    msg_scroll.set((*sst).save_msg_scroll);
    if force_restart_edit.get() {
        force_restart_edit.set(false_0 != 0);
    } else {
        restart_edit.set((*sst).save_restart_edit);
    }
    finish_op.set((*sst).save_finish_op);
    opcount.set((*sst).save_opcount);
    reg_executing.set((*sst).save_reg_executing);
    pending_end_reg_executing.set((*sst).save_pending_end_reg_executing);
    msg_didout.set(msg_didout.get() as c_int | (*sst).save_msg_didout as c_int != 0);
    State.set((*sst).save_State);
    ui_cursor_shape();
}
pub unsafe extern "C" fn expr_map_locked() -> bool {
    return expr_map_lock.get() > 0 as c_int && (*curbuf.get()).b_flags & BF_DUMMY == 0;
}
unsafe extern "C" fn ex_normal(mut eap: *mut exarg_T) {
    if !(*curbuf.get()).terminal.is_null() && State.get() & MODE_TERMINAL as c_int != 0 {
        emsg(b"Can't re-enter normal mode from terminal mode\0".as_ptr() as *const c_char);
        return;
    }
    let mut arg: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if expr_map_locked() {
        emsg(gettext(&raw const e_secure as *const c_char));
        return;
    }
    if ex_normal_busy.get() as OptInt >= p_mmd.get() {
        emsg(gettext(
            b"E192: Recursive use of :normal too deep\0".as_ptr() as *const c_char,
        ));
        return;
    }
    let mut len: c_int = 0 as c_int;
    let mut l: c_int = 0;
    let mut p: *mut c_char = (*eap).arg;
    while *p as c_int != NUL {
        l = utfc_ptr2len(p) - 1 as c_int;
        while l > 0 as c_int {
            p = p.offset(1);
            if *p as c_int == K_SPECIAL as c_char as c_int {
                len += 2 as c_int;
            }
            l -= 1;
        }
        p = p.offset(1);
    }
    if len > 0 as c_int {
        arg = xmalloc(
            strlen((*eap).arg)
                .wrapping_add(len as size_t)
                .wrapping_add(1 as size_t),
        ) as *mut c_char;
        len = 0 as c_int;
        let mut p_0: *mut c_char = (*eap).arg;
        while *p_0 as c_int != NUL {
            let c2rust_fresh17 = len;
            len = len + 1;
            *arg.offset(c2rust_fresh17 as isize) = *p_0;
            l = utfc_ptr2len(p_0) - 1 as c_int;
            while l > 0 as c_int {
                p_0 = p_0.offset(1);
                let c2rust_fresh18 = len;
                len = len + 1;
                *arg.offset(c2rust_fresh18 as isize) = *p_0;
                if *p_0 as c_int == K_SPECIAL as c_char as c_int {
                    let c2rust_fresh19 = len;
                    len = len + 1;
                    *arg.offset(c2rust_fresh19 as isize) = KS_SPECIAL as c_char;
                    let c2rust_fresh20 = len;
                    len = len + 1;
                    *arg.offset(c2rust_fresh20 as isize) = KE_FILLER as c_char;
                }
                l -= 1;
            }
            *arg.offset(len as isize) = NUL as c_char;
            p_0 = p_0.offset(1);
        }
    }
    (*ex_normal_busy.ptr()) += 1;
    let mut save_state: save_state_T = save_state_T {
        save_msg_scroll: 0,
        save_restart_edit: 0,
        save_msg_didout: false,
        save_State: 0,
        save_finish_op: false,
        save_opcount: 0,
        save_reg_executing: 0,
        save_pending_end_reg_executing: false,
        tabuf: tasave_T {
            save_typebuf: typebuf_T {
                tb_buf: ::core::ptr::null_mut::<uint8_t>(),
                tb_noremap: ::core::ptr::null_mut::<uint8_t>(),
                tb_buflen: 0,
                tb_off: 0,
                tb_len: 0,
                tb_maplen: 0,
                tb_silent: 0,
                tb_no_abbr_cnt: 0,
                tb_change_cnt: 0,
            },
            typebuf_valid: false,
            old_char: 0,
            old_mod_mask: 0,
            save_readbuf1: buffheader_T {
                bh_first: buffblock_T {
                    b_next: ::core::ptr::null_mut::<buffblock>(),
                    b_strlen: 0,
                    b_str: [0; 1],
                },
                bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
                bh_index: 0,
                bh_space: 0,
                bh_create_newblock: false,
            },
            save_readbuf2: buffheader_T {
                bh_first: buffblock_T {
                    b_next: ::core::ptr::null_mut::<buffblock>(),
                    b_strlen: 0,
                    b_str: [0; 1],
                },
                bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
                bh_index: 0,
                bh_space: 0,
                bh_create_newblock: false,
            },
            save_inputbuf: String_0 {
                data: ::core::ptr::null_mut::<c_char>(),
                size: 0,
            },
        },
    };
    if save_current_state(&raw mut save_state) {
        loop {
            if (*eap).addr_count != 0 as c_int {
                let c2rust_fresh21 = (*eap).line1;
                (*eap).line1 = (*eap).line1 + 1;
                (*curwin.get()).w_cursor.lnum = c2rust_fresh21;
                (*curwin.get()).w_cursor.col = 0 as c_int as colnr_T;
                check_cursor_moved(curwin.get());
            }
            exec_normal_cmd(
                if !arg.is_null() { arg } else { (*eap).arg },
                if (*eap).forceit != 0 {
                    REMAP_NONE as c_int
                } else {
                    REMAP_YES as c_int
                },
                false_0 != 0,
            );
            if !((*eap).addr_count > 0 as c_int && (*eap).line1 <= (*eap).line2 && !got_int.get()) {
                break;
            }
        }
    }
    update_topline_cursor();
    restore_current_state(&raw mut save_state);
    (*ex_normal_busy.ptr()) -= 1;
    setmouse();
    ui_cursor_shape();
    xfree(arg as *mut c_void);
}
unsafe extern "C" fn ex_startinsert(mut eap: *mut exarg_T) {
    if (*eap).forceit != 0 {
        if (*curwin.get()).w_cursor.lnum == 0 {
            (*curwin.get()).w_cursor.lnum = 1 as c_int as linenr_T;
        }
        set_cursor_for_append_to_line();
    }
    if State.get() & MODE_INSERT as c_int != 0 {
        return;
    }
    if (*eap).cmdidx as c_int == CMD_startinsert as c_int {
        restart_edit.set('a' as c_int);
    } else if (*eap).cmdidx as c_int == CMD_startreplace as c_int {
        restart_edit.set('R' as c_int);
    } else {
        restart_edit.set('V' as c_int);
    }
    if (*eap).forceit == 0 {
        if (*eap).cmdidx as c_int == CMD_startinsert as c_int {
            restart_edit.set('i' as c_int);
        }
        (*curwin.get()).w_curswant = 0 as c_int as colnr_T;
    }
    if VIsual_active.get() {
        showmode();
    }
}
unsafe extern "C" fn ex_stopinsert(mut _eap: *mut exarg_T) {
    restart_edit.set(0 as c_int);
    stop_insert_mode.set(true_0 != 0);
    clearmode();
}
pub unsafe extern "C" fn exec_normal_cmd(mut cmd: *mut c_char, mut remap: c_int, mut silent: bool) {
    ins_typebuf(cmd, remap, 0 as c_int, true_0 != 0, silent);
    exec_normal(false_0 != 0, false_0 != 0);
}
pub unsafe extern "C" fn exec_normal(mut was_typed: bool, mut use_vpeekc: bool) {
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
    let mut c: c_int = 0;
    clear_oparg(&raw mut oa);
    finish_op.set(false_0 != 0);
    while (!stuff_empty()
        || (was_typed as c_int != 0 || typebuf_typed() == 0)
            && (*typebuf.ptr()).tb_len > 0 as c_int
        || use_vpeekc as c_int != 0
            && {
                c = vpeekc();
                c != NUL
            }
            && c != Ctrl_C)
        && !got_int.get()
    {
        update_topline_cursor();
        normal_cmd(&raw mut oa, true_0 != 0);
    }
}
unsafe extern "C" fn ex_checkpath(mut eap: *mut exarg_T) {
    find_pattern_in_path(
        ::core::ptr::null_mut::<c_char>(),
        kDirectionNotSet,
        0 as size_t,
        false_0 != 0,
        false_0 != 0,
        CHECK_PATH as c_int,
        1 as c_int,
        if (*eap).forceit != 0 {
            ACTION_SHOW_ALL as c_int
        } else {
            ACTION_SHOW as c_int
        },
        1 as linenr_T,
        MAXLNUM as c_int as linenr_T,
        (*eap).forceit != 0,
        false_0 != 0,
    );
}
unsafe extern "C" fn ex_psearch(mut eap: *mut exarg_T) {
    g_do_tagpreview.set(p_pvh.get() as c_int);
    ex_findpat(eap);
    g_do_tagpreview.set(0 as c_int);
}
unsafe extern "C" fn ex_findpat(mut eap: *mut exarg_T) {
    let mut whole: bool = true_0 != 0;
    let mut action: c_int = 0;
    match *(*cmdnames.ptr())[(*eap).cmdidx as usize]
        .cmd_name
        .offset(2 as c_int as isize) as c_int
    {
        101 => {
            if *(*cmdnames.ptr())[(*eap).cmdidx as usize]
                .cmd_name
                .offset(0 as c_int as isize) as c_int
                == 'p' as c_int
            {
                action = ACTION_GOTO as c_int;
            } else {
                action = ACTION_SHOW as c_int;
            }
        }
        105 => {
            action = ACTION_SHOW_ALL as c_int;
        }
        117 => {
            action = ACTION_GOTO as c_int;
        }
        _ => {
            action = ACTION_SPLIT as c_int;
        }
    }
    let mut n: c_int = 1 as c_int;
    if ascii_isdigit(*(*eap).arg as c_int) {
        n = getdigits_int(&raw mut (*eap).arg, false_0 != 0, 0 as c_int);
        (*eap).arg = skipwhite((*eap).arg);
    }
    if *(*eap).arg as c_int == '/' as c_int {
        whole = false_0 != 0;
        (*eap).arg = (*eap).arg.offset(1);
        let mut p: *mut c_char = skip_regexp((*eap).arg, '/' as c_int, magic_isset() as c_int);
        if *p != 0 {
            let c2rust_fresh16 = p;
            p = p.offset(1);
            *c2rust_fresh16 = NUL as c_char;
            p = skipwhite(p);
            if ends_excmd(*p as c_int) == 0 {
                (*eap).errmsg = ex_errmsg(&raw const e_trailing_arg as *const c_char, p);
            } else {
                (*eap).nextcmd = check_nextcmd(p);
            }
        }
    }
    if (*eap).skip == 0 {
        find_pattern_in_path(
            (*eap).arg,
            kDirectionNotSet,
            strlen((*eap).arg),
            whole,
            (*eap).forceit == 0,
            if *(*eap).cmd as c_int == 'd' as c_int {
                FIND_DEFINE as c_int
            } else {
                FIND_ANY as c_int
            },
            n,
            action,
            (*eap).line1,
            (*eap).line2,
            (*eap).forceit != 0,
            false_0 != 0,
        );
    }
}
unsafe extern "C" fn ex_ptag(mut eap: *mut exarg_T) {
    g_do_tagpreview.set(p_pvh.get() as c_int);
    ex_tag_cmd(
        eap,
        (*cmdnames.ptr())[(*eap).cmdidx as usize]
            .cmd_name
            .offset(1 as c_int as isize),
    );
}
unsafe extern "C" fn ex_pedit(mut eap: *mut exarg_T) {
    let mut curwin_save: *mut win_T = curwin.get();
    prepare_preview_window();
    do_exedit(eap, ::core::ptr::null_mut::<win_T>());
    back_to_current_window(curwin_save);
}
unsafe extern "C" fn ex_pbuffer(mut eap: *mut exarg_T) {
    let mut curwin_save: *mut win_T = curwin.get();
    prepare_preview_window();
    do_exbuffer(eap);
    back_to_current_window(curwin_save);
}
unsafe extern "C" fn prepare_preview_window() {
    g_do_tagpreview.set(p_pvh.get() as c_int);
    prepare_tagpreview(true_0 != 0);
}
unsafe extern "C" fn back_to_current_window(mut curwin_save: *mut win_T) {
    if curwin.get() != curwin_save && win_valid(curwin_save) as c_int != 0 {
        validate_cursor(curwin.get());
        redraw_later(curwin.get(), UPD_VALID as c_int);
        win_enter(curwin_save, true_0 != 0);
    }
    g_do_tagpreview.set(0 as c_int);
}
unsafe extern "C" fn ex_stag(mut eap: *mut exarg_T) {
    postponed_split.set(-1 as c_int);
    postponed_split_flags.set((*cmdmod.ptr()).cmod_split);
    postponed_split_tab.set((*cmdmod.ptr()).cmod_tab);
    ex_tag_cmd(
        eap,
        (*cmdnames.ptr())[(*eap).cmdidx as usize]
            .cmd_name
            .offset(1 as c_int as isize),
    );
    postponed_split_flags.set(0 as c_int);
    postponed_split_tab.set(0 as c_int);
}
unsafe extern "C" fn ex_tag(mut eap: *mut exarg_T) {
    ex_tag_cmd(eap, (*cmdnames.ptr())[(*eap).cmdidx as usize].cmd_name);
}
unsafe extern "C" fn ex_tag_cmd(mut eap: *mut exarg_T, mut name: *const c_char) {
    let mut cmd: c_int = 0;
    match *name.offset(1 as c_int as isize) as c_int {
        106 => {
            cmd = DT_JUMP as c_int;
        }
        115 => {
            cmd = DT_SELECT as c_int;
        }
        112 | 78 => {
            cmd = DT_PREV as c_int;
        }
        110 => {
            cmd = DT_NEXT as c_int;
        }
        111 => {
            cmd = DT_POP as c_int;
        }
        102 | 114 => {
            cmd = DT_FIRST as c_int;
        }
        108 => {
            cmd = DT_LAST as c_int;
        }
        _ => {
            cmd = DT_TAG as c_int;
        }
    }
    if *name.offset(0 as c_int as isize) as c_int == 'l' as c_int {
        cmd = DT_LTAG as c_int;
    }
    do_tag(
        (*eap).arg,
        cmd,
        if (*eap).addr_count > 0 as c_int {
            (*eap).line2 as c_int
        } else {
            1 as c_int
        },
        (*eap).forceit,
        true_0 != 0,
    );
}
pub unsafe extern "C" fn find_cmdline_var(
    mut src: *const c_char,
    mut usedlen: *mut size_t,
) -> ssize_t {
    static spec_str: GlobalCell<[*mut c_char; 15]> = GlobalCell::new([
        b"%\0".as_ptr() as *const c_char as *mut c_char,
        b"#\0".as_ptr() as *const c_char as *mut c_char,
        b"<cword>\0".as_ptr() as *const c_char as *mut c_char,
        b"<cWORD>\0".as_ptr() as *const c_char as *mut c_char,
        b"<cexpr>\0".as_ptr() as *const c_char as *mut c_char,
        b"<cfile>\0".as_ptr() as *const c_char as *mut c_char,
        b"<sfile>\0".as_ptr() as *const c_char as *mut c_char,
        b"<slnum>\0".as_ptr() as *const c_char as *mut c_char,
        b"<stack>\0".as_ptr() as *const c_char as *mut c_char,
        b"<script>\0".as_ptr() as *const c_char as *mut c_char,
        b"<afile>\0".as_ptr() as *const c_char as *mut c_char,
        b"<abuf>\0".as_ptr() as *const c_char as *mut c_char,
        b"<amatch>\0".as_ptr() as *const c_char as *mut c_char,
        b"<sflnum>\0".as_ptr() as *const c_char as *mut c_char,
        b"<SID>\0".as_ptr() as *const c_char as *mut c_char,
    ]);
    let mut i: size_t = 0 as size_t;
    while i < ::core::mem::size_of::<[*mut c_char; 15]>()
        .wrapping_div(::core::mem::size_of::<*mut c_char>())
        .wrapping_div(
            (::core::mem::size_of::<[*mut c_char; 15]>()
                .wrapping_rem(::core::mem::size_of::<*mut c_char>())
                == 0) as c_int as usize,
        )
    {
        let mut len: size_t = strlen((*spec_str.ptr())[i as usize] as *const c_char);
        if strncmp(src, (*spec_str.ptr())[i as usize] as *const c_char, len) == 0 as c_int {
            *usedlen = len;
            '_c2rust_label: {
                if i <= 9223372036854775807 as c_long as size_t {
                } else {
                    __assert_fail(
                        b"i <= SSIZE_MAX\0".as_ptr() as *const c_char,
                        b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                        7692 as c_uint,
                        b"ssize_t find_cmdline_var(const char *, size_t *)\0".as_ptr()
                            as *const c_char,
                    );
                }
            };
            return i as ssize_t;
        }
        i = i.wrapping_add(1);
    }
    return -1 as ssize_t;
}
pub unsafe extern "C" fn eval_vars(
    mut src: *mut c_char,
    mut srcstart: *const c_char,
    mut usedlen: *mut size_t,
    mut lnump: *mut linenr_T,
    mut errormsg: *mut *const c_char,
    mut escaped: *mut c_int,
    mut empty_is_error: bool,
) -> *mut c_char {
    let mut result: *mut c_char = b"\0".as_ptr() as *const c_char as *mut c_char;
    let mut resultbuf: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut resultlen: size_t = 0;
    let mut valid: c_int = VALID_HEAD as c_int | VALID_PATH as c_int;
    let mut tilde_file: bool = false_0 != 0;
    let mut skip_mod: bool = false_0 != 0;
    let mut strbuf: [c_char; 30] = [0; 30];
    *errormsg = ::core::ptr::null::<c_char>();
    if !escaped.is_null() {
        *escaped = false_0;
    }
    let mut spec_idx: ssize_t = find_cmdline_var(src, usedlen);
    if spec_idx < 0 as ssize_t {
        *usedlen = 1 as size_t;
        return ::core::ptr::null_mut::<c_char>();
    }
    if src > srcstart as *mut c_char && *src.offset(-1 as c_int as isize) as c_int == '\\' as c_int
    {
        *usedlen = 0 as size_t;
        memmove(
            src.offset(-(1 as c_int as isize)) as *mut c_void,
            src as *const c_void,
            strlen(src).wrapping_add(1 as size_t),
        );
        return ::core::ptr::null_mut::<c_char>();
    }
    if spec_idx == SPEC_CWORD as c_int as ssize_t
        || spec_idx == SPEC_CCWORD as c_int as ssize_t
        || spec_idx == SPEC_CEXPR as c_int as ssize_t
    {
        resultlen = find_ident_under_cursor(
            &raw mut result,
            if spec_idx == SPEC_CWORD as c_int as ssize_t {
                FIND_IDENT as c_int | FIND_STRING as c_int
            } else if spec_idx == SPEC_CEXPR as c_int as ssize_t {
                FIND_IDENT as c_int | FIND_STRING as c_int | FIND_EVAL as c_int
            } else {
                FIND_STRING as c_int
            },
            ::core::ptr::null_mut::<c_int>(),
        );
        if resultlen == 0 as size_t {
            *errormsg = b"\0".as_ptr() as *const c_char;
            return ::core::ptr::null_mut::<c_char>();
        }
    } else {
        let mut s: *mut c_char = ::core::ptr::null_mut::<c_char>();
        let mut i: c_int = 0;
        match spec_idx {
            0 => {
                if (*curbuf.get()).b_fname.is_null() {
                    result = b"\0".as_ptr() as *const c_char as *mut c_char;
                    valid = 0 as c_int;
                } else {
                    result = (*curbuf.get()).b_fname;
                    tilde_file = strcmp(result, b"~\0".as_ptr() as *const c_char) == 0 as c_int;
                }
            }
            1 => {
                if *src.offset(1 as c_int as isize) as c_int == '#' as c_int {
                    result = arg_all();
                    resultbuf = result;
                    *usedlen = 2 as size_t;
                    if !escaped.is_null() {
                        *escaped = true_0;
                    }
                    skip_mod = true_0 != 0;
                } else {
                    s = src.offset(1 as c_int as isize);
                    if *s as c_int == '<' as c_int {
                        s = s.offset(1);
                    }
                    i = getdigits_int(&raw mut s, false_0 != 0, 0 as c_int);
                    if s == src.offset(2 as c_int as isize)
                        && *src.offset(1 as c_int as isize) as c_int == '-' as c_int
                    {
                        s = s.offset(-1);
                    }
                    *usedlen = s.offset_from(src) as size_t;
                    if *src.offset(1 as c_int as isize) as c_int == '<' as c_int && i != 0 as c_int
                    {
                        if *usedlen < 2 as size_t {
                            *usedlen = 1 as size_t;
                            return ::core::ptr::null_mut::<c_char>();
                        }
                        result = tv_list_find_str(get_vim_var_list(VV_OLDFILES), i - 1 as c_int)
                            as *mut c_char;
                        if result.is_null() {
                            *errormsg = b"\0".as_ptr() as *const c_char;
                            return ::core::ptr::null_mut::<c_char>();
                        }
                    } else {
                        if i == 0 as c_int
                            && *src.offset(1 as c_int as isize) as c_int == '<' as c_int
                            && *usedlen > 1 as size_t
                        {
                            *usedlen = 1 as size_t;
                        }
                        let mut buf: *mut buf_T = buflist_findnr(i);
                        if buf.is_null() {
                            *errormsg = gettext(
                                b"E194: No alternate file name to substitute for '#'\0".as_ptr()
                                    as *const c_char,
                            );
                            return ::core::ptr::null_mut::<c_char>();
                        }
                        if !lnump.is_null() {
                            *lnump = ECMD_LAST as c_int as linenr_T;
                        }
                        if (*buf).b_fname.is_null() {
                            result = b"\0".as_ptr() as *const c_char as *mut c_char;
                            valid = 0 as c_int;
                        } else {
                            result = (*buf).b_fname;
                            tilde_file =
                                strcmp(result, b"~\0".as_ptr() as *const c_char) == 0 as c_int;
                        }
                    }
                }
            }
            5 => {
                result = file_name_at_cursor(
                    FNAME_MESS as c_int | FNAME_HYP as c_int,
                    1 as c_int,
                    ::core::ptr::null_mut::<linenr_T>(),
                );
                if result.is_null() {
                    *errormsg = b"\0".as_ptr() as *const c_char;
                    return ::core::ptr::null_mut::<c_char>();
                }
                resultbuf = result;
            }
            10 => {
                if !(*autocmd_fname.ptr()).is_null() && !autocmd_fname_full.get() {
                    autocmd_fname_full.set(true_0 != 0);
                    result = FullName_save(autocmd_fname.get(), false_0 != 0);
                    xstrlcpy(autocmd_fname.get(), result, MAXPATHL as size_t);
                    xfree(result as *mut c_void);
                }
                result = autocmd_fname.get();
                if result.is_null() {
                    *errormsg = gettext(
                        (e_no_autocommand_file_name_to_substitute_for_afile.ptr() as *const _)
                            as *const c_char,
                    );
                    return ::core::ptr::null_mut::<c_char>();
                }
                result = path_try_shorten_fname(result);
            }
            11 => {
                if autocmd_bufnr.get() <= 0 as c_int {
                    *errormsg = gettext(
                        (e_no_autocommand_buffer_number_to_substitute_for_abuf.ptr() as *const _)
                            as *const c_char,
                    );
                    return ::core::ptr::null_mut::<c_char>();
                }
                snprintf(
                    &raw mut strbuf as *mut c_char,
                    ::core::mem::size_of::<[c_char; 30]>(),
                    b"%d\0".as_ptr() as *const c_char,
                    autocmd_bufnr.get(),
                );
                result = &raw mut strbuf as *mut c_char;
            }
            12 => {
                result = autocmd_match.get();
                if result.is_null() {
                    *errormsg = gettext(
                        (e_no_autocommand_match_name_to_substitute_for_amatch.ptr() as *const _)
                            as *const c_char,
                    );
                    return ::core::ptr::null_mut::<c_char>();
                }
            }
            6 => {
                result = estack_sfile(ESTACK_SFILE);
                if result.is_null() {
                    *errormsg = gettext(
                        (e_no_source_file_name_to_substitute_for_sfile.ptr() as *const _)
                            as *const c_char,
                    );
                    return ::core::ptr::null_mut::<c_char>();
                }
                resultbuf = result;
            }
            8 => {
                result = estack_sfile(ESTACK_STACK);
                if result.is_null() {
                    *errormsg = gettext(
                        (e_no_call_stack_to_substitute_for_stack.ptr() as *const _)
                            as *const c_char,
                    );
                    return ::core::ptr::null_mut::<c_char>();
                }
                resultbuf = result;
            }
            9 => {
                result = estack_sfile(ESTACK_SCRIPT);
                if result.is_null() {
                    *errormsg = gettext(
                        (e_no_script_file_name_to_substitute_for_script.ptr() as *const _)
                            as *const c_char,
                    );
                    return ::core::ptr::null_mut::<c_char>();
                }
                resultbuf = result;
            }
            7 => {
                if (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
                .es_name
                .is_null()
                    || (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
                    .es_lnum
                        == 0 as linenr_T
                {
                    *errormsg = gettext(
                        (e_no_line_number_to_use_for_slnum.ptr() as *const _) as *const c_char,
                    );
                    return ::core::ptr::null_mut::<c_char>();
                }
                snprintf(
                    &raw mut strbuf as *mut c_char,
                    ::core::mem::size_of::<[c_char; 30]>(),
                    b"%d\0".as_ptr() as *const c_char,
                    (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
                    .es_lnum,
                );
                result = &raw mut strbuf as *mut c_char;
            }
            13 => {
                if (*current_sctx.ptr()).sc_lnum
                    + (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
                    .es_lnum
                    == 0 as linenr_T
                {
                    *errormsg = gettext(
                        (e_no_line_number_to_use_for_sflnum.ptr() as *const _) as *const c_char,
                    );
                    return ::core::ptr::null_mut::<c_char>();
                }
                snprintf(
                    &raw mut strbuf as *mut c_char,
                    ::core::mem::size_of::<[c_char; 30]>(),
                    b"%d\0".as_ptr() as *const c_char,
                    (*current_sctx.ptr()).sc_lnum
                        + (*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
                        .es_lnum,
                );
                result = &raw mut strbuf as *mut c_char;
            }
            14 => {
                if (*current_sctx.ptr()).sc_sid <= 0 as c_int {
                    *errormsg = gettext(&raw const e_usingsid as *const c_char);
                    return ::core::ptr::null_mut::<c_char>();
                }
                snprintf(
                    &raw mut strbuf as *mut c_char,
                    ::core::mem::size_of::<[c_char; 30]>(),
                    b"<SNR>%d_\0".as_ptr() as *const c_char,
                    (*current_sctx.ptr()).sc_sid,
                );
                result = &raw mut strbuf as *mut c_char;
            }
            _ => {
                *errormsg = b"\0".as_ptr() as *const c_char;
            }
        }
        resultlen = strlen(result);
        if *src.offset(*usedlen as isize) as c_int == '<' as c_int {
            *usedlen = (*usedlen).wrapping_add(1);
            let mut s_0: *mut c_char = ::core::ptr::null_mut::<c_char>();
            s_0 = strrchr(result, '.' as c_int);
            if !s_0.is_null() && s_0 >= path_tail(result) {
                resultlen = s_0.offset_from(result) as size_t;
            }
        } else if !skip_mod {
            valid |= modify_fname(
                src,
                tilde_file,
                usedlen,
                &raw mut result,
                &raw mut resultbuf,
                &raw mut resultlen,
            );
            if result.is_null() {
                *errormsg = b"\0".as_ptr() as *const c_char;
                return ::core::ptr::null_mut::<c_char>();
            }
        }
    }
    if resultlen == 0 as size_t || valid != VALID_HEAD as c_int + VALID_PATH as c_int {
        if empty_is_error {
            if valid != VALID_HEAD as c_int + VALID_PATH as c_int {
                *errormsg = gettext(
                    b"E499: Empty file name for '%' or '#', only works with \":p:h\"\0".as_ptr()
                        as *const c_char,
                );
            } else {
                *errormsg =
                    gettext(b"E500: Evaluates to an empty string\0".as_ptr() as *const c_char);
            }
        }
        result = ::core::ptr::null_mut::<c_char>();
    } else {
        result = xmemdupz(result as *const c_void, resultlen) as *mut c_char;
    }
    xfree(resultbuf as *mut c_void);
    return result;
}
pub unsafe extern "C" fn expand_sfile(mut arg: *mut c_char) -> *mut c_char {
    let mut result: *mut c_char = xstrdup(arg);
    let mut p: *mut c_char = result;
    while *p != 0 {
        if strncmp(p, b"<sfile>\0".as_ptr() as *const c_char, 7 as size_t) != 0 as c_int {
            p = p.offset(1);
        } else {
            let mut srclen: size_t = 0;
            let mut errormsg: *const c_char = ::core::ptr::null::<c_char>();
            let mut repl: *mut c_char = eval_vars(
                p,
                result,
                &raw mut srclen,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut errormsg,
                ::core::ptr::null_mut::<c_int>(),
                true_0 != 0,
            );
            if !errormsg.is_null() {
                if *errormsg != 0 {
                    emsg(errormsg);
                }
                xfree(result as *mut c_void);
                return ::core::ptr::null_mut::<c_char>();
            }
            if repl.is_null() {
                p = p.offset(srclen as isize);
            } else {
                let mut len: size_t = strlen(result)
                    .wrapping_sub(srclen)
                    .wrapping_add(strlen(repl))
                    .wrapping_add(1 as size_t);
                let mut newres: *mut c_char = xmalloc(len) as *mut c_char;
                memmove(
                    newres as *mut c_void,
                    result as *const c_void,
                    p.offset_from(result) as size_t,
                );
                strcpy(newres.offset(p.offset_from(result) as isize), repl);
                len = strlen(newres);
                strcat(newres, p.offset(srclen as isize));
                xfree(repl as *mut c_void);
                xfree(result as *mut c_void);
                result = newres;
                p = newres.offset(len as isize);
            }
        }
    }
    return result;
}
unsafe extern "C" fn ex_shada(mut eap: *mut exarg_T) {
    let mut save_shada: *mut c_char = p_shada.get();
    if *p_shada.get() as c_int == NUL {
        p_shada.set(b"'100\0".as_ptr() as *const c_char as *mut c_char);
    }
    if (*eap).cmdidx as c_int == CMD_rviminfo as c_int
        || (*eap).cmdidx as c_int == CMD_rshada as c_int
    {
        shada_read_everything((*eap).arg, (*eap).forceit != 0, false_0 != 0);
    } else {
        shada_write_file((*eap).arg, (*eap).forceit != 0);
    }
    p_shada.set(save_shada);
}
pub unsafe extern "C" fn dialog_msg(
    mut buff: *mut c_char,
    mut format: *mut c_char,
    mut fname: *mut c_char,
) {
    if fname.is_null() {
        fname = gettext(b"Untitled\0".as_ptr() as *const c_char);
    }
    vim_snprintf(buff, DIALOG_MSG_SIZE as c_int as size_t, format, fname);
}
static filetype_detect: GlobalCell<TriState> = GlobalCell::new(kNone);
static filetype_plugin: GlobalCell<TriState> = GlobalCell::new(kNone);
static filetype_indent: GlobalCell<TriState> = GlobalCell::new(kNone);
unsafe extern "C" fn ex_filetype(mut eap: *mut exarg_T) {
    if *(*eap).arg as c_int == NUL {
        smsg(
            0 as c_int,
            b"filetype detection:%s  plugin:%s  indent:%s\0".as_ptr() as *const c_char,
            if filetype_detect.get() as c_int == kTrue as c_int {
                b"ON\0".as_ptr() as *const c_char
            } else {
                b"OFF\0".as_ptr() as *const c_char
            },
            if filetype_plugin.get() as c_int == kTrue as c_int {
                if filetype_detect.get() as c_int == kTrue as c_int {
                    b"ON\0".as_ptr() as *const c_char
                } else {
                    b"(on)\0".as_ptr() as *const c_char
                }
            } else {
                b"OFF\0".as_ptr() as *const c_char
            },
            if filetype_indent.get() as c_int == kTrue as c_int {
                if filetype_detect.get() as c_int == kTrue as c_int {
                    b"ON\0".as_ptr() as *const c_char
                } else {
                    b"(on)\0".as_ptr() as *const c_char
                }
            } else {
                b"OFF\0".as_ptr() as *const c_char
            },
        );
        return;
    }
    let mut arg: *mut c_char = (*eap).arg;
    let mut plugin: bool = false_0 != 0;
    let mut indent: bool = false_0 != 0;
    loop {
        if strncmp(arg, b"plugin\0".as_ptr() as *const c_char, 6 as size_t) == 0 as c_int {
            plugin = true_0 != 0;
            arg = skipwhite(arg.offset(6 as c_int as isize));
        } else {
            if strncmp(arg, b"indent\0".as_ptr() as *const c_char, 6 as size_t) != 0 as c_int {
                break;
            }
            indent = true_0 != 0;
            arg = skipwhite(arg.offset(6 as c_int as isize));
        }
    }
    if strcmp(arg, b"on\0".as_ptr() as *const c_char) == 0 as c_int
        || strcmp(arg, b"detect\0".as_ptr() as *const c_char) == 0 as c_int
    {
        if *arg as c_int == 'o' as c_int || filetype_detect.get() as c_int != kTrue as c_int {
            source_runtime(FILETYPE_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
            filetype_detect.set(kTrue);
            if plugin {
                source_runtime(FTPLUGIN_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
                filetype_plugin.set(kTrue);
            }
            if indent {
                source_runtime(INDENT_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
                filetype_indent.set(kTrue);
            }
        }
        if *arg as c_int == 'd' as c_int {
            do_doautocmd(
                b"filetypedetect BufRead\0".as_ptr() as *const c_char as *mut c_char,
                true_0 != 0,
                ::core::ptr::null_mut::<bool>(),
            );
            do_modelines(0 as c_int);
        }
    } else if strcmp(arg, b"off\0".as_ptr() as *const c_char) == 0 as c_int {
        if plugin as c_int != 0 || indent as c_int != 0 {
            if plugin {
                source_runtime(FTPLUGOF_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
                filetype_plugin.set(kFalse);
            }
            if indent {
                source_runtime(INDOFF_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
                filetype_indent.set(kFalse);
            }
        } else {
            source_runtime(FTOFF_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
            filetype_detect.set(kFalse);
        }
    } else {
        semsg(gettext(&raw const e_invarg2 as *const c_char), arg);
    };
}
pub unsafe extern "C" fn filetype_plugin_enable() {
    if filetype_plugin.get() as c_int == kNone as c_int {
        source_runtime(FTPLUGIN_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
        filetype_plugin.set(kTrue);
    }
    if filetype_indent.get() as c_int == kNone as c_int {
        source_runtime(INDENT_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
        filetype_indent.set(kTrue);
    }
}
pub unsafe extern "C" fn filetype_maybe_enable() {
    if filetype_detect.get() as c_int == kNone as c_int {
        source_runtime(FILETYPE_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
        filetype_detect.set(kTrue);
    }
}
unsafe extern "C" fn ex_setfiletype(mut eap: *mut exarg_T) {
    if (*curbuf.get()).b_did_filetype {
        return;
    }
    let mut arg: *mut c_char = (*eap).arg;
    if strncmp(arg, b"FALLBACK \0".as_ptr() as *const c_char, 9 as size_t) == 0 as c_int {
        arg = arg.offset(9 as c_int as isize);
    }
    set_option_value_give_err(
        kOptFiletype,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: cstr_as_string(arg),
            },
        },
        OPT_LOCAL as c_int,
    );
    if arg != (*eap).arg {
        (*curbuf.get()).b_did_filetype = false_0 != 0;
    }
}
unsafe extern "C" fn ex_digraphs(mut eap: *mut exarg_T) {
    if *(*eap).arg as c_int != NUL {
        putdigraph(::core::ffi::CStr::from_ptr((*eap).arg).to_bytes());
    } else {
        listdigraphs((*eap).forceit != 0);
    };
}
pub unsafe extern "C" fn set_no_hlsearch(mut flag: bool) {
    no_hlsearch.set(flag);
    set_vim_var_nr(
        VV_HLSEARCH,
        (!no_hlsearch.get() && p_hls.get() != 0) as c_int as varnumber_T,
    );
}
unsafe extern "C" fn ex_nohlsearch(mut _eap: *mut exarg_T) {
    set_no_hlsearch(true_0 != 0);
    redraw_all_later(UPD_SOME_VALID as c_int);
}
unsafe extern "C" fn ex_fold(mut eap: *mut exarg_T) {
    if foldManualAllowed(true_0 != 0) != 0 {
        let mut start: pos_T = pos_T {
            lnum: (*eap).line1,
            col: 1 as colnr_T,
            coladd: 0 as colnr_T,
        };
        let mut end: pos_T = pos_T {
            lnum: (*eap).line2,
            col: 1 as colnr_T,
            coladd: 0 as colnr_T,
        };
        foldCreate(curwin.get(), start, end);
    }
}
unsafe extern "C" fn ex_foldopen(mut eap: *mut exarg_T) {
    let mut start: pos_T = pos_T {
        lnum: (*eap).line1,
        col: 1 as colnr_T,
        coladd: 0 as colnr_T,
    };
    let mut end: pos_T = pos_T {
        lnum: (*eap).line2,
        col: 1 as colnr_T,
        coladd: 0 as colnr_T,
    };
    opFoldRange(
        start,
        end,
        ((*eap).cmdidx as c_int == CMD_foldopen as c_int) as c_int,
        (*eap).forceit,
        false_0 != 0,
    );
}
unsafe extern "C" fn ex_folddo(mut eap: *mut exarg_T) {
    let mut lnum: linenr_T = (*eap).line1;
    while lnum <= (*eap).line2 {
        if hasFolding(
            curwin.get(),
            lnum,
            ::core::ptr::null_mut::<linenr_T>(),
            ::core::ptr::null_mut::<linenr_T>(),
        ) as c_int
            == ((*eap).cmdidx as c_int == CMD_folddoclosed as c_int) as c_int
        {
            ml_setmarked(lnum);
        }
        lnum += 1;
    }
    global_exe((*eap).arg);
    ml_clearmarked();
}
pub unsafe extern "C" fn is_loclist_cmd(mut cmdidx: c_int) -> bool {
    if cmdidx < 0 as c_int || cmdidx >= CMD_SIZE as c_int {
        return false_0 != 0;
    }
    return *(*cmdnames.ptr())[cmdidx as usize]
        .cmd_name
        .offset(0 as c_int as isize) as c_int
        == 'l' as c_int;
}
pub unsafe extern "C" fn get_pressedreturn() -> bool {
    return ex_pressedreturn.get();
}
pub unsafe extern "C" fn set_pressedreturn(mut val: bool) {
    ex_pressedreturn.set(val);
}
unsafe extern "C" fn ex_checkhealth(mut eap: *mut exarg_T) {
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<c_char>(),
    };
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 2] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_14 { boolean: false },
    }; 2];
    args.capacity = 2 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let mut mods: [c_char; 1024] = [0; 1024];
    let mut mods_len: size_t = 0 as size_t;
    mods[0 as c_int as usize] = NUL as c_char;
    if (*cmdmod.ptr()).cmod_tab > 0 as c_int || (*cmdmod.ptr()).cmod_split != 0 as c_int {
        let mut multi_mods: bool = false_0 != 0;
        mods_len = add_win_cmd_modifiers(
            &raw mut mods as *mut c_char,
            cmdmod.ptr(),
            &raw mut multi_mods,
        );
        '_c2rust_label: {
            if mods_len < ::core::mem::size_of::<[c_char; 1024]>() {
            } else {
                __assert_fail(
                    b"mods_len < sizeof(mods)\0".as_ptr() as *const c_char,
                    b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                    8263 as c_uint,
                    b"void ex_checkhealth(exarg_T *)\0".as_ptr() as *const c_char,
                );
            }
        };
    }
    let c2rust_fresh23 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh23 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_14 {
            string: String_0 {
                data: &raw mut mods as *mut c_char,
                size: mods_len,
            },
        },
    };
    let c2rust_fresh24 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh24 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_14 {
            string: cstr_as_string((*eap).arg),
        },
    };
    nlua_exec(
        String_0 {
            data: b"vim.health._check(...)\0".as_ptr() as *const c_char as *mut c_char,
            size: ::core::mem::size_of::<[c_char; 23]>().wrapping_sub(1 as size_t),
        },
        ::core::ptr::null::<c_char>(),
        args,
        kRetNilBool,
        ::core::ptr::null_mut::<Arena>(),
        &raw mut err,
    );
    if !(err.type_0 as c_int != kErrorTypeNone as c_int) {
        return;
    }
    let mut vimruntime_env: *mut c_char =
        os_getenv_noalloc(b"VIMRUNTIME\0".as_ptr() as *const c_char);
    if vimruntime_env.is_null() {
        emsg(gettext(
            b"E5009: $VIMRUNTIME is empty or unset\0".as_ptr() as *const c_char
        ));
    } else {
        let mut rtp_ok: bool = !strstr(p_rtp.get(), vimruntime_env).is_null();
        if rtp_ok {
            semsg(
                gettext(b"E5009: Invalid $VIMRUNTIME: %s\0".as_ptr() as *const c_char),
                vimruntime_env,
            );
        } else {
            emsg(gettext(
                b"E5009: Invalid 'runtimepath'\0".as_ptr() as *const c_char
            ));
        }
    }
    semsg_multiline(b"emsg\0".as_ptr() as *const c_char, err.msg);
    api_clear_error(&raw mut err);
}
unsafe extern "C" fn ex_terminal(mut eap: *mut exarg_T) {
    let mut ex_cmd: [c_char; 1024] = [0; 1024];
    let mut len: size_t = 0 as size_t;
    if (*cmdmod.ptr()).cmod_tab > 0 as c_int || (*cmdmod.ptr()).cmod_split != 0 as c_int {
        let mut multi_mods: bool = false_0 != 0;
        ex_cmd[0 as c_int as usize] = NUL as c_char;
        len = add_win_cmd_modifiers(
            &raw mut ex_cmd as *mut c_char,
            cmdmod.ptr(),
            &raw mut multi_mods,
        );
        '_c2rust_label: {
            if len < ::core::mem::size_of::<[c_char; 1024]>() {
            } else {
                __assert_fail(
                    b"len < sizeof(ex_cmd)\0".as_ptr() as *const c_char,
                    b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                    8298 as c_uint,
                    b"void ex_terminal(exarg_T *)\0".as_ptr() as *const c_char,
                );
            }
        };
        let mut result: c_int = snprintf(
            (&raw mut ex_cmd as *mut c_char).offset(len as isize),
            ::core::mem::size_of::<[c_char; 1024]>().wrapping_sub(len),
            b" new\0".as_ptr() as *const c_char,
        );
        '_c2rust_label_0: {
            if result > 0 as c_int {
            } else {
                __assert_fail(
                    b"result > 0\0".as_ptr() as *const c_char,
                    b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                    8300 as c_uint,
                    b"void ex_terminal(exarg_T *)\0".as_ptr() as *const c_char,
                );
            }
        };
        len = len.wrapping_add(result as size_t);
    } else {
        let mut result_0: c_int = snprintf(
            &raw mut ex_cmd as *mut c_char,
            ::core::mem::size_of::<[c_char; 1024]>(),
            b"enew%s\0".as_ptr() as *const c_char,
            if (*eap).forceit != 0 {
                b"!\0".as_ptr() as *const c_char
            } else {
                b"\0".as_ptr() as *const c_char
            },
        );
        '_c2rust_label_1: {
            if result_0 > 0 as c_int {
            } else {
                __assert_fail(
                    b"result > 0\0".as_ptr() as *const c_char,
                    b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                    8304 as c_uint,
                    b"void ex_terminal(exarg_T *)\0".as_ptr() as *const c_char,
                );
            }
        };
        len = len.wrapping_add(result_0 as size_t);
    }
    '_c2rust_label_2: {
        if len < ::core::mem::size_of::<[c_char; 1024]>() {
        } else {
            __assert_fail(
                b"len < sizeof(ex_cmd)\0".as_ptr() as *const c_char,
                b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                8308 as c_uint,
                b"void ex_terminal(exarg_T *)\0".as_ptr() as *const c_char,
            );
        }
    };
    if *(*eap).arg as c_int != NUL {
        let mut name: *mut c_char =
            vim_strsave_escaped((*eap).arg, b"\"\\\0".as_ptr() as *const c_char);
        snprintf(
            (&raw mut ex_cmd as *mut c_char).offset(len as isize),
            ::core::mem::size_of::<[c_char; 1024]>().wrapping_sub(len),
            b" | call jobstart(\"%s\",{'term':v:true})\0".as_ptr() as *const c_char,
            name,
        );
        xfree(name as *mut c_void);
    } else {
        if *p_sh.get() as c_int == NUL {
            emsg(gettext(&raw const e_shellempty as *const c_char));
            return;
        }
        let mut argv: *mut *mut c_char =
            shell_build_argv(::core::ptr::null::<c_char>(), ::core::ptr::null::<c_char>());
        let mut p: *mut *mut c_char = argv;
        let mut tempstring: [c_char; 512] = [0; 512];
        let mut shell_argv: [c_char; 512] = [
            0 as c_char,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ];
        while !(*p).is_null() {
            let mut escaped: *mut c_char =
                vim_strsave_escaped(*p, b"\"\\\0".as_ptr() as *const c_char);
            snprintf(
                &raw mut tempstring as *mut c_char,
                ::core::mem::size_of::<[c_char; 512]>(),
                b",\"%s\"\0".as_ptr() as *const c_char,
                escaped,
            );
            xfree(escaped as *mut c_void);
            xstrlcat(
                &raw mut shell_argv as *mut c_char,
                &raw mut tempstring as *mut c_char,
                ::core::mem::size_of::<[c_char; 512]>(),
            );
            p = p.offset(1);
        }
        shell_free_argv(argv);
        snprintf(
            (&raw mut ex_cmd as *mut c_char).offset(len as isize),
            ::core::mem::size_of::<[c_char; 1024]>().wrapping_sub(len),
            b" | call jobstart([%s], {'term':v:true})\0".as_ptr() as *const c_char,
            (&raw mut shell_argv as *mut c_char).offset(1 as c_int as isize),
        );
    }
    do_cmdline_cmd(&raw mut ex_cmd as *mut c_char);
}
unsafe extern "C" fn ex_lsp(mut eap: *mut exarg_T) {
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<c_char>(),
    };
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_14 { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh22 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh22 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_14 {
            string: cstr_as_string((*eap).arg),
        },
    };
    nlua_exec(
        String_0 {
            data: b"require'vim._core.ex_cmd'.ex_lsp(...)\0".as_ptr() as *const c_char
                as *mut c_char,
            size: ::core::mem::size_of::<[c_char; 38]>().wrapping_sub(1 as size_t),
        },
        ::core::ptr::null::<c_char>(),
        args,
        kRetNilBool,
        ::core::ptr::null_mut::<Arena>(),
        &raw mut err,
    );
    if err.type_0 as c_int != kErrorTypeNone as c_int {
        emsg_multiline(
            err.msg,
            b"lua_error\0".as_ptr() as *const c_char,
            HLF_E as c_int,
            true_0 != 0,
        );
    }
    api_clear_error(&raw mut err);
}
unsafe extern "C" fn ex_fclose(mut eap: *mut exarg_T) {
    win_float_remove((*eap).forceit != 0, (*eap).line1 as c_int);
}
pub unsafe extern "C" fn verify_command(mut cmd: *mut c_char) {
    if strcmp(b"smile\0".as_ptr() as *const c_char, cmd) != 0 as c_int {
        return;
    }
    let mut a: c_int = HLF_E as c_int;
    msg(
        b" #xxn`          #xnxx`        ,+x@##@Mz;`        .xxxxxxxxxnz+,      znnnnnnnnnnnnnnnn.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n###z          x####`      :x##########W+`      ,#############M;    W################.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####;         x####`    `z##############W:     ,################   W################.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####W.        x####`   ,W#################+    ,#################  W################.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n#####n        x####`   @###################    ,#################i W################.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n######i       x####`  .#########@W@########*   ,#################W`W################.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n######@.      x####`  x######W*.  `;n#######:  ,####x,,,,:*M######iW###@:,,,,,,,,,,,`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n#######n      x####` *######+`       :M#####M  ,####n      `x#####xW###@`\0".as_ptr()
            as *const c_char,
        a,
    );
    msg(
        b" n########*     x####``@####@;          `x#####i ,####n       ,#####@W###@`\0".as_ptr()
            as *const c_char,
        a,
    );
    msg(
        b" n########@     x####`*#####i            `M####M ,####n        x#########@`\0".as_ptr()
            as *const c_char,
        a,
    );
    msg(
        b" n#########     x####`M####z              :#####:,####n        z#########@`\0".as_ptr()
            as *const c_char,
        a,
    );
    msg(
        b" n#########*    x####,#####.               n####+,####n        n#########@`\0".as_ptr()
            as *const c_char,
        a,
    );
    msg(
        b" n####@####@,   x####i####x                ;####x,####n       `W#####@####+++++++++++i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####*#####M`  x#########*                `####@,####n       i#####MW###############W\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####.######+  x####z####;                 W####,####n      i@######W###############W\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####.`W#####: x####n####:                 M####:####@nnnnnW#######,W###############W\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####. :#####M`x####z####;                 W####,#################z W###############W\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####.  #######x#########*                `####W,################W` W###############W\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####.  `M#####W####i####x                ;####x,###############W,  W####+**********i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####.   ,##########,#####.               n####+,##############n.   W###@`\0".as_ptr()
            as *const c_char,
        a,
    );
    msg(
        b" n####.    ##########`M####z              :#####:,###########Wz:     W###@`\0".as_ptr()
            as *const c_char,
        a,
    );
    msg(
        b" n####.    x#########`*#####i            `M####M ,####x.....`        W###@`\0".as_ptr()
            as *const c_char,
        a,
    );
    msg(
        b" n####.    ,@########``@####@;          `x#####i ,####n              W###@`\0".as_ptr()
            as *const c_char,
        a,
    );
    msg(
        b" n####.     *########` *#####@+`       ,M#####M  ,####n              W###@`\0".as_ptr()
            as *const c_char,
        a,
    );
    msg(
        b" n####.      x#######`  x######W*.  `;n######@:  ,####n              W###@,,,,,,,,,,,,`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####.      .@######`  .#########@W@########*   ,####n              W################,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####.       i######`   @###################    ,####n              W################,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####.        n#####`   ,W#################+    ,####n              W################,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####.        .@####`    .n##############W;     ,####n              W################,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" n####.         i####`      :x##########W+`      ,####n              W################,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" +nnnn`          +nnn`        ,+x@##@Mz;`        .nnnn+              zxxxxxxxxxxxxxxxx.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(b" \0".as_ptr() as *const c_char, a);
    msg(
        b"                                                                                   ,+M@#Mi\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"                                                                                 .z########\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"                                                                                i@#########i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"                                                                              `############W`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"                                                                             `n#############i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"                                                                            `n##############n\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"     ``                                                                     z###############@`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"    `W@z,                                                                  ##################,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"    *#####`                                                               i############@x@###i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"    ######M.                                                             :#############n`,W##+\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"    +######@:                                                           .W#########M@##+  *##z\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"    :#######@:                                                         `x########@#x###*  ,##n\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"    `@#######@;                                                        z#########M*@nW#i  .##x\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"     z########@i                                                      *###########WM#@#,  `##x\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"     i##########+                                                    ;###########*n###@   `##x\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"     `@#MM#######x,                                                 ,@#########zM,`z##M   `@#x\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      n##M#W#######n.               `.:i*+#zzzz##+i:.`             ,W#########Wii,`n@#@` n@##n\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      ;###@#x#######n         `,i#nW@#####@@WWW@@####@Mzi.        ,W##########@z.. ;zM#+i####z\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"       x####nz########    .;#x@##@Wn#*;,.`      ``,:*#x@##M+,    ;@########xz@WM+#` `n@#######\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"       ,@####M########xi#@##@Mzi,`                     .+x###Mi:n##########Mz```.:i  *@######*\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"        *#####W#########ix+:`                             :n#############z:       `*.`M######i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"        i#W##nW@+@##@#M@;                                   ;W@@##########W,        i`x@#####,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"        `@@n@Wn#@iMW*#*:                                     `iz#z@######x.           M######`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"         z##zM###x`*, .`                                          `iW#####W;:`        +#####M\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"         ,###nn##n`                                                ,#####x;`        ,;@######\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"          x###xz#.                                                   in###+        `:######@.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"          ;####n+                                                    `Mnx##xi`   , zM#######\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"          `W####+                i.                                   `.+x###@#. :n,z######:\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"           z####@`              ;#:                                     .ii@###@;.*M*z####@`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"           i####M         `   `i@#,           ::                           +#n##@+@##W####n\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"           :####x    ,i. ##xzM###@`     i.   .@@,                           .z####x#######*\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"           ,###W;   i##Wz#########     :##   z##n                           ,@########x###:\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"            n##n   `W###########M`;n,  i#x  ,###@i                           *W########W#@`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"           .@##+  `x###########@. z#+ .M#W``x#####n`                         `;#######@z#x\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"           n###z :W############@  z#*  @##xM#######@n;                        `########nW+\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"          ;####nW##############W :@#* `@#############*                        :########z@i`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"          M##################### M##:  @#############@:                       *W########M#\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"         ;#####################i.##x`  W#############W,                       :n########zx\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"         x####################@.`x;    @#############z.                       .@########W#\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"        ,######################`       W###############x*,`                    W######zM#i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"        #######################:       z##################@x+*#zzi            `@#########.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"        W########W#z#M#########;       *##########################z            :@#######@`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"       `@#######x`;#z ,x#######;       z###########M###xnM@########*            :M######@\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"       i########, x#@`  z######;       *##########i *#@`  `+########+`            n######.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"       n#######@` M##,  `W#####.       *#########z  ###;    z########M:           :W####n\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"       M#######M  n##.   x####x        `x########:  z##+    M#########@;           .n###+\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"       W#######@` :#W   `@####:         `@######W   i###   ;###########@.            n##n\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"       W########z` ,,  .x####z           @######@`  `W#;  `W############*            *###;\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      `@#########Mi,:*n@####W`           W#######*   ..  `n#############i            i###x\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      .#####################z           `@#######@*`    .x############n:`            ;####.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      :####################x`,,`        `W#########@x#+#@#############i              ,####:\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      ;###################x#@###xi`      *############################:              `####i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      i##################+########M,      x##########################@`               W###i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      *################@; @########@,     .W#########################@                x###:\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      .+M#############z.  M#########x      ,W########################@`               ####.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      *M*;z@########x:    :W#######i        .M########################i               i###:\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      *##@z;#@####x:        :z###@i          `########################x               .###;\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      *#####n;#@##            ;##*             ,x#####################@`               W##*\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      *#######n;*            :M##W*,             *W####################`               n##z\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      i########@.         ,*n#######M*`           `###################M                *##M\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      i########n        `z#####@@#####Wi            ,M################;                ,##@`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      ;WMWW@###*       .x##@ni.``.:+zW##z`           `n##############z                  @##,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      .*++*i;;;.      .M#@+`          .##n            `x############x`                  n##i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      :########*      x#W,              *#+            *###########M`                   +##+\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      ,#########     :#@:                ##:           #nzzzzzzzzzz.                    :##x\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      .#####Wz+`     ##+                 `MM`          .znnnnnnnnn.                     `@#@`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      `@@ni;*nMz`    @W`                  :#+           .x#######n                       x##,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"       i;z@#####,   .#*                    z#:           ;;;*zW##;                       ###i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"       z########:   :#;                    `Wx          +###Wni;n.                       ;##z\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"       n########W:  .#*                     ,#,        ;#######@+                        `@#M\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      .###########n;.MM                      n*        ;iM#######*                        x#@`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      :#############@;;                      .n`      ,#W*iW#####W`                       +##,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      ,##############.                        ix.    `x###M;#######                       ,##i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      .#############@`                         x@n**#W######z;M###@.                       W##\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      .##############W:                        .x############@*;zW#;                       z#x\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      ,###############@;                        `##############@n*;.                       i#@\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      ,#################i                         :n##############W`                       .##,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      ,###################`                         .+W##########W,                        `##i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      :###################@zi,`                        ;zM@@@WMn*`                          @#z\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      :#######################@x+*i;;:i#M,                 ``                               M#W\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      ;################################@x.                                                  n##,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      i#####################@W@@@@Wxz*:`                                                    *##+\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      *######################+```                                                           :##M\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      ########################M;                                                            `@##,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      z#########################x,                                                           z###\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      n###########################n:                                                         ;##W`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      x#############################Mz#++##*                                                 `W##i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      M####################################@`                                                 ###x\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      W#####################################`                                                 .###,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      @####################################M                                                   n##z\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"      @##################z*i@WMMMx#x@#####,.                                                   :##@.\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"     `#####################@xi`     `::,*                                                       x##+\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"     .#####################@#M.                                                                 ;##@`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"     ,#####################:.                                                                    M##i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"     ;###################ni`                                                                     i##M\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"     *#################W#`                                                                       `W##,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"     z#################@Wx+.                                                                      +###\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"     x######################z.                                                                    .@#@`\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"    `@#######################@;                                                                    z##;\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"    :##########################:                                                                   :##z\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"    +#########################W#                                                                    M#W\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"    W################@n+*i;:,`                                                                      +##,\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"   :##################WMxz+,                                                                        ,##i\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"   n#######################W..,                                                                      W##\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"  +#########################WW@+. .:.                                                                z#x\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" `@#############################@@###:                                                               *#W\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b" #################################Wz:                                                                :#@\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b",@###############################i                                                                   .##\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"n@@@@@@@#########################+                                                                   `##\0"
            .as_ptr() as *const c_char,
        a,
    );
    msg(
        b"`      `.:.`.,:iii;;;;;;;;iii;;;:`       `.``                                                        `nW\0"
            .as_ptr() as *const c_char,
        a,
    );
}
pub unsafe extern "C" fn is_map_cmd(mut cmdidx: cmdidx_T) -> bool {
    if (cmdidx as c_int) < 0 as c_int {
        return false_0 != 0;
    }
    let mut func: ex_func_T = (*cmdnames.ptr())[cmdidx as usize].cmd_func;
    return ex_func_is(func, ex_map)
        || ex_func_is(func, ex_unmap)
        || ex_func_is(func, ex_mapclear)
        || ex_func_is(func, ex_abbreviate)
        || ex_func_is(func, ex_abclear);
}
pub const IOSIZE: c_int = 1024 as c_int + 1 as c_int;
pub const MSG_BUF_LEN: c_int = 480 as c_int;
pub const FILETYPE_FILE: [c_char; 26] = c_bytes(b"filetype.lua filetype.vim\0");
pub const FTPLUGIN_FILE: [c_char; 13] = c_bytes(b"ftplugin.vim\0");
pub const INDENT_FILE: [c_char; 11] = c_bytes(b"indent.vim\0");
pub const FTOFF_FILE: [c_char; 10] = c_bytes(b"ftoff.vim\0");
pub const FTPLUGOF_FILE: [c_char; 13] = c_bytes(b"ftplugof.vim\0");
pub const INDOFF_FILE: [c_char; 11] = c_bytes(b"indoff.vim\0");
pub const PROF_YES: c_int = 1 as c_int;
pub const SID_NONE: c_int = -6 as c_int;
pub const K_SPECIAL: c_int = 0x80 as c_int;
pub const KS_SPECIAL: c_int = 254 as c_int;
pub const KE_FILLER: c_int = 'X' as c_int;
static command_count: GlobalCell<c_int> = GlobalCell::new(557 as c_int);
static cmdnames: GlobalCell<[CommandDefinition; 557]> = GlobalCell::new(unsafe {
    [
        CommandDefinition {
            cmd_name: b"append\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_append as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 18354435 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"abbreviate\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_abbreviate as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"abclear\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_abclear as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"aboveleft\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"all\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_all as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"amenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"anoremenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"args\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_args as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147726 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"argadd\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_argadd as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4367 as uint32_t,
            cmd_addr_type: ADDR_ARGUMENTS,
        },
        CommandDefinition {
            cmd_name: b"argdelete\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_argdelete as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 271 as uint32_t,
            cmd_addr_type: ADDR_ARGUMENTS,
        },
        CommandDefinition {
            cmd_name: b"argdo\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_listdo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2215 as uint32_t,
            cmd_addr_type: ADDR_ARGUMENTS,
        },
        CommandDefinition {
            cmd_name: b"argdedupe\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_argdedupe as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"argedit\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_argedit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 151951 as uint32_t,
            cmd_addr_type: ADDR_ARGUMENTS,
        },
        CommandDefinition {
            cmd_name: b"argglobal\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_args as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147726 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"arglocal\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_args as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147726 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"argument\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_argument as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 148743 as uint32_t,
            cmd_addr_type: ADDR_ARGUMENTS,
        },
        CommandDefinition {
            cmd_name: b"ascii\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(do_ascii as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563904 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"autocmd\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_autocmd as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17311750 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"augroup\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_autocmd as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301782 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"aunmenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"buffer\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_buffer as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 115975 as uint32_t,
            cmd_addr_type: ADDR_BUFFERS,
        },
        CommandDefinition {
            cmd_name: b"bNext\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_bprevious as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17667 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"ball\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_buffer_all as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"badd\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_edit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17318300 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"balt\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_edit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17318300 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"bdelete\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_bunload as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 34055 as uint32_t,
            cmd_addr_type: ADDR_BUFFERS,
        },
        CommandDefinition {
            cmd_name: b"belowright\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"bfirst\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_brewind as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 16643 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"blast\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_blast as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 16643 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"bmodified\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_bmodified as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17667 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"bnext\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_bnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17667 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"botright\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"bprevious\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_bprevious as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17667 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"brewind\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_brewind as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 16643 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"break\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_break as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563904 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"breakadd\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_breakadd as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"breakdel\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_breakdel as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"breaklist\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_breaklist as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"browse\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17303684 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"buffers\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                buflist_list as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301766 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"bufdo\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_listdo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2215 as uint32_t,
            cmd_addr_type: ADDR_BUFFERS,
        },
        CommandDefinition {
            cmd_name: b"bunload\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_bunload as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 34055 as uint32_t,
            cmd_addr_type: ADDR_LOADED_BUFFERS,
        },
        CommandDefinition {
            cmd_name: b"bwipeout\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_bunload as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 99591 as uint32_t,
            cmd_addr_type: ADDR_BUFFERS,
        },
        CommandDefinition {
            cmd_name: b"change\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_change as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 18351427 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"cNext\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"cNfile\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"cabbrev\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_abbreviate as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cabclear\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_abclear as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cabove\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbelow as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"caddbuffer\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbuffer as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 277 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"caddexpr\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cexpr as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2196 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"caddfile\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cfile as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 284 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cafter\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbelow as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"call\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_call as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565829 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"catch\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_catch as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563652 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cbuffer\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbuffer as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 279 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"cbefore\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbelow as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"cbelow\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbelow as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"cbottom\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbottom as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cc\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cc as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_QUICKFIX,
        },
        CommandDefinition {
            cmd_name: b"cclose\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cclose as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cd\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cd as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301790 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cdo\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_listdo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2215 as uint32_t,
            cmd_addr_type: ADDR_QUICKFIX_VALID,
        },
        CommandDefinition {
            cmd_name: b"center\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_align as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 18350405 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"cexpr\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cexpr as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2198 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cfile\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cfile as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 286 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cfdo\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_listdo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2215 as uint32_t,
            cmd_addr_type: ADDR_QUICKFIX_VALID,
        },
        CommandDefinition {
            cmd_name: b"cfirst\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cc as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"cgetfile\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cfile as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 284 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cgetbuffer\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbuffer as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 277 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"cgetexpr\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cexpr as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2196 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"chdir\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cd as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301790 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"changes\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_changes as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"checkhealth\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_checkhealth as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 260 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"checkpath\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_checkpath as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301762 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"checktime\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_checktime as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 34053 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"chistory\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(qf_history as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"clist\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(qf_list as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301766 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"clast\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cc as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"close\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_close as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17302787 as uint32_t,
            cmd_addr_type: ADDR_WINDOWS,
        },
        CommandDefinition {
            cmd_name: b"clearjumps\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_clearjumps as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cmap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cmapclear\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_mapclear as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cmenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"cnext\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"cnewer\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(qf_age as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"cnfile\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"cnoremap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cnoreabbrev\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_abbreviate as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cnoremenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"copy\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_copymove as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 18350405 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"colder\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(qf_age as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"colorscheme\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_colorscheme as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301780 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"command\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_command as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17311750 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"comclear\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_comclear as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"compiler\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_compiler as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301782 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"continue\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_continue as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17563904 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"confirm\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17303684 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"connect\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_connect as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2206 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"const\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_let as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"copen\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_copen as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"cprevious\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"cpfile\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"cquit\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> !>,
                ex_func_T,
            >(Some(ex_cquit as unsafe extern "C" fn(*mut exarg_T) -> !)),
            cmd_preview_func: None,
            cmd_argt: 5379 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"crewind\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cc as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"cunmap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_unmap as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cunabbrev\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_abbreviate as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cunmenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cwindow\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cwindow as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"delete\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_operators as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 18351937 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"delmarks\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_delmarks as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301766 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"debug\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_debug as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565828 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"debuggreedy\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_debuggreedy as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17305857 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"defer\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_call as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565828 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"delcommand\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_delcommand as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301910 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"delfunction\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_delfunction as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301654 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"detach\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_detach as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"display\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_display as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565956 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"diffupdate\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_diffupdate as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 258 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"diffget\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_diffgetput as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 1052933 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"diffoff\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_diffoff as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 258 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"diffpatch\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_diffpatch as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 1048860 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"diffput\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_diffgetput as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 4357 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"diffsplit\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_diffsplit as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 284 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"diffthis\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_diffthis as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"digraphs\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_digraphs as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301766 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"djump\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_findpat as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 103 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"dlist\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_findpat as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301607 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"doautocmd\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_doautocmd as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"doautoall\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_doautoall as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"drop\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_drop as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147854 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"dsearch\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_findpat as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301607 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"dsplit\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_findpat as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 103 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"edit\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_edit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147742 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"earlier\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_later as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301780 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"echo\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_echo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"echoerr\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_execute as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"echohl\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_echohl as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563908 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"echomsg\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_execute as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"echon\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_echo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"else\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_else as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563904 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"elseif\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_else as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"emenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_emenu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17303941 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"endif\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_endif as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563904 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"endfunction\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_endfunction as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"endfor\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_endwhile as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17563904 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"endtry\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_endtry as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563904 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"endwhile\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_endwhile as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17563904 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"enew\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_edit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 258 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"eval\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_eval as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"ex\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_edit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147742 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"execute\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_execute as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"exit\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_exit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17432959 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"exusage\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_exusage as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"file\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_file as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4383 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"files\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                buflist_list as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301766 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"filetype\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_filetype as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"filter\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2182 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"find\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_find as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147871 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"finally\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_finally as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563904 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"finish\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_finish as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563904 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"first\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_rewind as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147718 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"fold\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_fold as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563969 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"foldclose\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_foldopen as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17563971 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"folddoopen\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_folddo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2213 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"folddoclosed\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_folddo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2213 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"foldopen\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_foldopen as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17563971 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"for\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_while as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"function\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_function as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17563654 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"fclose\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_fclose as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 259 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"global\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_global as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563751 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"goto\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_goto as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17564929 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"grep\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_make as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2447 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"grepadd\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_make as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2447 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"gui\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_nogui as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17449230 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"gvim\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_nogui as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17449230 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"help\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_help as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2054 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"helpclose\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_helpclose as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"helpgrep\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_helpgrep as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"helptags\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_helptags as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301900 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"highlight\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_highlight as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17563910 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"hide\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_hide as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1287 as uint32_t,
            cmd_addr_type: ADDR_WINDOWS,
        },
        CommandDefinition {
            cmd_name: b"history\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_history as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"horizontal\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"insert\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_append as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 18350339 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"iabbrev\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_abbreviate as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"iabclear\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_abclear as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"if\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_if as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"ijump\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_findpat as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 103 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"ilist\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_findpat as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301607 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"imap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"imapclear\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_mapclear as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"imenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"inoremap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"inoreabbrev\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_abbreviate as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"inoremenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"intro\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_intro as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"iput\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_iput as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 18355011 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"isearch\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_findpat as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301607 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"isplit\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_findpat as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 103 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"iunmap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_unmap as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"iunabbrev\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_abbreviate as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"iunmenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"join\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_join as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 20448579 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"jumps\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_jumps as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"k\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_mark as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563925 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"keepmarks\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"keepjumps\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"keeppatterns\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"keepalt\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"list\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_print as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 19400001 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"lNext\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"lNfile\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"last\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_last as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147718 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"labove\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbelow as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"language\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_language as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"laddexpr\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cexpr as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2196 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"laddbuffer\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbuffer as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 277 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"laddfile\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cfile as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 284 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lafter\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbelow as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"later\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_later as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301780 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lbuffer\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbuffer as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 279 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"lbefore\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbelow as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"lbelow\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbelow as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"lbottom\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbottom as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lcd\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cd as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301790 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lchdir\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cd as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301790 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lclose\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cclose as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"ldo\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_listdo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2215 as uint32_t,
            cmd_addr_type: ADDR_QUICKFIX_VALID,
        },
        CommandDefinition {
            cmd_name: b"left\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_align as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 18350405 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"leftabove\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"let\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_let as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lexpr\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cexpr as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2198 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lfile\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cfile as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 286 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lfdo\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_listdo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2215 as uint32_t,
            cmd_addr_type: ADDR_QUICKFIX_VALID,
        },
        CommandDefinition {
            cmd_name: b"lfirst\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cc as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"lgetfile\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cfile as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 284 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lgetbuffer\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbuffer as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 277 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"lgetexpr\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cexpr as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2196 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lgrep\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_make as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2447 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"lgrepadd\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_make as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2447 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"lhelpgrep\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_helpgrep as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lhistory\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(qf_history as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"ll\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cc as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_QUICKFIX,
        },
        CommandDefinition {
            cmd_name: b"llast\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cc as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"llist\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(qf_list as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301766 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lmap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lmapclear\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_mapclear as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lmake\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_make as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2318 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lnoremap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lnext\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"lnewer\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(qf_age as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"lnfile\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"loadview\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_loadview as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 284 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"loadkeymap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_loadkeymap as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301504 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lockmarks\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lockvar\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_lockvar as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563782 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lolder\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(qf_age as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"lopen\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_copen as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"lprevious\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"lpfile\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"lrewind\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cc as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"ltag\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 278 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lunmap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_unmap as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lua\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_lua as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301509 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"luado\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_luado as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301669 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"luafile\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_luafile as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301661 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"lvimgrep\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_vimgrep as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 16779663 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"lvimgrepadd\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_vimgrep as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 16779663 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"lwindow\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cwindow as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"ls\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                buflist_list as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301766 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lsp\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_lsp as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 132 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"move\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_copymove as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 18350405 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"mark\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_mark as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563925 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"make\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_make as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2318 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"map\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312006 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"mapclear\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_mapclear as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301766 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"marks\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_marks as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"match\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_match as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301509 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"menu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316103 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"menutranslate\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_menutranslate as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"messages\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_messages as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301765 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"mkexrc\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_mkrc as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301790 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"mksession\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_mkrc as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 286 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"mkspell\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_mkspell as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2446 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"mkvimrc\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_mkrc as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301790 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"mkview\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_mkrc as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 286 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"mode\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_mode as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301780 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"mzscheme\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_script_ni as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17563813 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"mzfile\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ni as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301661 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"next\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_next as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147727 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"new\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_splitview as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 147743 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"nmap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"nmapclear\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_mapclear as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"nmenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"nnoremap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"nnoremenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"noremap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312006 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"noautocmd\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"nohlsearch\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_nohlsearch as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17563904 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"noreabbrev\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_abbreviate as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"noremenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316103 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"noswapfile\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"normal\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_normal as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17574023 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"number\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_print as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 19400001 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"nunmap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_unmap as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"nunmenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"oldfiles\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_oldfiles as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17563906 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"omap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"omapclear\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_mapclear as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"omenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"only\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_only as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_WINDOWS,
        },
        CommandDefinition {
            cmd_name: b"onoremap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"onoremenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"options\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_options as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"ounmap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_unmap as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"ounmenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"ownsyntax\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_ownsyntax as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"print\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_print as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 19662145 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"packadd\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_packadd as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17564062 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"packloadall\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_packloadall as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17563906 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"pbuffer\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_pbuffer as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 115975 as uint32_t,
            cmd_addr_type: ADDR_BUFFERS,
        },
        CommandDefinition {
            cmd_name: b"pclose\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_pclose as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 258 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"perl\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_perl as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563813 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"perldo\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_perldo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301669 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"perlfile\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_perlfile as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301661 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"pedit\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_pedit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147742 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"pop\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 5379 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"popup\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_popup as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17303942 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"ppop\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ptag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 5379 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"preserve\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_preserve as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"previous\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_previous as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 148743 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"profile\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_profile as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301766 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"profdel\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_breakdel as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"psearch\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_psearch as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 103 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"ptag\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ptag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4375 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"ptNext\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ptag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4355 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"ptfirst\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ptag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4355 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"ptjump\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ptag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 278 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"ptlast\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ptag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 258 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"ptnext\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ptag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4355 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"ptprevious\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ptag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4355 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"ptrewind\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ptag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4355 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"ptselect\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ptag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 278 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"put\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_put as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 18355011 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"pwd\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_pwd as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"python\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_python3 as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301637 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"pydo\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_pydo3 as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301669 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"pyfile\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_py3file as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301661 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"py3\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_python3 as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301637 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"py3do\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_pydo3 as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301669 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"python3\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_python3 as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301637 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"py3file\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_py3file as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301661 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"pyx\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_python3 as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301637 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"pyxdo\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_pydo3 as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301669 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"pythonx\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_python3 as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301637 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"pyxfile\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_py3file as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301661 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"quit\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_quit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17302787 as uint32_t,
            cmd_addr_type: ADDR_WINDOWS,
        },
        CommandDefinition {
            cmd_name: b"quitall\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_quitall as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 258 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"qall\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_quitall as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301762 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"read\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_read as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 18485599 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"recover\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_recover as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 286 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"redo\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_redo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"redir\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_redir as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301774 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"redraw\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_redraw as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301762 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"redrawstatus\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_redrawstatus as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301762 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"redrawtabline\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_redrawtabline as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"registers\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_display as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565956 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"resize\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_resize as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301781 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"restart\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_restart as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 18436 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"retab\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_retab as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 18350455 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"return\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_return as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"rewind\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_rewind as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147718 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"right\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_align as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 18350405 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"rightbelow\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"rshada\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_shada as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301790 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"runtime\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_runtime as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17564046 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"rundo\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_rundo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 156 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"ruby\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ruby as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301637 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"rubydo\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_rubydo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301669 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"rubyfile\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_rubyfile as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301661 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"rviminfo\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_shada as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301790 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"substitute\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_substitute as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: Some(
                ex_substitute_preview
                    as unsafe extern "C" fn(*mut exarg_T, c_int, handle_T) -> c_int,
            ),
            cmd_argt: 151519301 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"sNext\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_previous as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 148743 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"sargument\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_argument as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 148743 as uint32_t,
            cmd_addr_type: ADDR_ARGUMENTS,
        },
        CommandDefinition {
            cmd_name: b"sall\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_all as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"sandbox\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"saveas\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_write as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17432862 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"sbuffer\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_buffer as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 115975 as uint32_t,
            cmd_addr_type: ADDR_BUFFERS,
        },
        CommandDefinition {
            cmd_name: b"sbNext\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_bprevious as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17665 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"sball\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_buffer_all as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17665 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"sbfirst\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_brewind as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 16640 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"sblast\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_blast as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 16640 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"sbmodified\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_bmodified as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17665 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"sbnext\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_bnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17665 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"sbprevious\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_bprevious as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17665 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"sbrewind\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_brewind as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 16640 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"scriptnames\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_scriptnames as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17302799 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"scriptencoding\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_scriptencoding as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301780 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"set\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_set as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563910 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"setfiletype\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_setfiletype as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301892 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"setglobal\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_set as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563910 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"setlocal\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_set as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563910 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"sfind\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_splitview as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 147871 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"sfirst\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_rewind as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147718 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"simalt\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ni as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301908 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"sign\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_sign as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301637 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"silent\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17565830 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"sleep\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_sleep as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17302791 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"slast\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_last as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147718 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"smagic\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_submagic as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: Some(
                ex_submagic_preview as unsafe extern "C" fn(*mut exarg_T, c_int, handle_T) -> c_int,
            ),
            cmd_argt: 151519301 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"smap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"smapclear\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_mapclear as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"smenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"snext\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_next as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147727 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"snomagic\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_submagic as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: Some(
                ex_submagic_preview as unsafe extern "C" fn(*mut exarg_T, c_int, handle_T) -> c_int,
            ),
            cmd_argt: 151519301 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"snoremap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"snoremenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"source\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_source as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563967 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"sort\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_sort as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1050727 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"split\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_splitview as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 147743 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"spellgood\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_spell as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 391 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"spelldump\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_spelldump as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 258 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"spellinfo\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_spellinfo as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"spellrepall\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_spellrepall as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"spellrare\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_spell as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 391 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"spellundo\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_spell as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 391 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"spellwrong\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_spell as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 391 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"sprevious\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_previous as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 148743 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"srewind\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_rewind as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147718 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"stop\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_stop as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301762 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"stag\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_stag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4375 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"startinsert\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_startinsert as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301762 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"startgreplace\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_startinsert as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301762 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"startreplace\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_startinsert as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301762 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"stopinsert\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_stopinsert as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301762 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"stjump\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_stag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 278 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"stselect\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_stag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 278 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"sunhide\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_buffer_all as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"sunmap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_unmap as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"sunmenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"suspend\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_stop as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301762 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"sview\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_splitview as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 147743 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"swapname\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_swapname as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"syntax\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_syntax as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17303556 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"syntime\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_syntime as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301908 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"syncbind\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_syncbind as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"t\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_copymove as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 18350405 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"tcd\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cd as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301790 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tchdir\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cd as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301790 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tNext\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4355 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"tag\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4375 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"tags\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(do_tags as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tab\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 6277 as uint32_t,
            cmd_addr_type: ADDR_TABS,
        },
        CommandDefinition {
            cmd_name: b"tabclose\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_tabclose as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17305879 as uint32_t,
            cmd_addr_type: ADDR_TABS,
        },
        CommandDefinition {
            cmd_name: b"tabdo\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_listdo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2213 as uint32_t,
            cmd_addr_type: ADDR_TABS,
        },
        CommandDefinition {
            cmd_name: b"tabedit\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_splitview as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 151839 as uint32_t,
            cmd_addr_type: ADDR_TABS,
        },
        CommandDefinition {
            cmd_name: b"tabfind\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_splitview as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 151967 as uint32_t,
            cmd_addr_type: ADDR_TABS,
        },
        CommandDefinition {
            cmd_name: b"tabfirst\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tabnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tabmove\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tabmove as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4373 as uint32_t,
            cmd_addr_type: ADDR_TABS,
        },
        CommandDefinition {
            cmd_name: b"tablast\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tabnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tabnext\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tabnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4373 as uint32_t,
            cmd_addr_type: ADDR_TABS,
        },
        CommandDefinition {
            cmd_name: b"tabnew\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_splitview as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 151839 as uint32_t,
            cmd_addr_type: ADDR_TABS,
        },
        CommandDefinition {
            cmd_name: b"tabonly\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tabonly as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17305879 as uint32_t,
            cmd_addr_type: ADDR_TABS,
        },
        CommandDefinition {
            cmd_name: b"tabprevious\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tabnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4373 as uint32_t,
            cmd_addr_type: ADDR_TABS_RELATIVE,
        },
        CommandDefinition {
            cmd_name: b"tabNext\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tabnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4373 as uint32_t,
            cmd_addr_type: ADDR_TABS_RELATIVE,
        },
        CommandDefinition {
            cmd_name: b"tabrewind\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tabnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tabs\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tabs as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tcl\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_script_ni as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301637 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"tcldo\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ni as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301669 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"tclfile\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ni as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301661 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"terminal\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_terminal as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301518 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tfirst\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4355 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"throw\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_throw as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563780 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tjump\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 278 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tlast\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 258 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tlmenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"tlnoremenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"tlunmenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tmenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"tmap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tmapclear\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_mapclear as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tnext\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4355 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"tnoremap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"topleft\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tprevious\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4355 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"trewind\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4355 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"trust\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_trust as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 16777500 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"try\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_try as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563904 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tselect\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 278 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tunmenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tunmap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_unmap as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"undo\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_undo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17306883 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"undojoin\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_undojoin as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"undolist\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_undolist as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"unabbreviate\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_abbreviate as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"unhide\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_buffer_all as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"uniq\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_uniq as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1050727 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"unlet\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_unlet as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563782 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"unlockvar\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_lockvar as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563782 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"unmap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_unmap as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312006 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"unmenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312006 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"unsilent\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17565828 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"update\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_update as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 131455 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"vglobal\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_global as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301605 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"version\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_version as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"verbose\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17565829 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"vertical\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"visual\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_edit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147742 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"view\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_edit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147742 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"vimgrep\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_vimgrep as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 16779663 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"vimgrepadd\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_vimgrep as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 16779663 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"viusage\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_viusage as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"vmap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"vmapclear\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_mapclear as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"vmenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"vnoremap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"vnew\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_splitview as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 147743 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"vnoremenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"vsplit\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_splitview as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 147743 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"vunmap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_unmap as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"vunmenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"write\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_write as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17432959 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"wNext\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_wnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 131423 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"wall\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(do_wqall as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17432834 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"while\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_while as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"winsize\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_winsize as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 388 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"wincmd\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_wincmd as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17302677 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"windo\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_listdo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2213 as uint32_t,
            cmd_addr_type: ADDR_WINDOWS,
        },
        CommandDefinition {
            cmd_name: b"winpos\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ni as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"wnext\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_wnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 131359 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"wprevious\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_wnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 131359 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"wq\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_exit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 131455 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"wqall\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(do_wqall as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 131358 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"wshada\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_shada as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301790 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"wundo\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_wundo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 158 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"wviminfo\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_shada as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301790 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"xit\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_exit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17432959 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"xall\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(do_wqall as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 258 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"xmap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"xmapclear\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_mapclear as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"xmenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"xnoremap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"xnoremenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"xunmap\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_unmap as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"xunmenu\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"yank\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_operators as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17303361 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"z\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_z as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 19398983 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"!\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_bang as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301583 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"#\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_print as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 19400001 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"&\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_substitute as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 18350149 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"<\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_operators as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 20448577 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"=\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_equal as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17432613 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b">\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_operators as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 20448577 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"@\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_at as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301829 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"~\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_substitute as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 18350149 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"Next\0".as_ptr() as *const c_char as *mut c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_previous as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 148743 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
    ]
});
static cmdidxs1: GlobalCell<[uint16_t; 26]> = GlobalCell::new([
    0 as uint16_t,
    20 as uint16_t,
    43 as uint16_t,
    109 as uint16_t,
    133 as uint16_t,
    154 as uint16_t,
    170 as uint16_t,
    176 as uint16_t,
    184 as uint16_t,
    203 as uint16_t,
    205 as uint16_t,
    210 as uint16_t,
    272 as uint16_t,
    290 as uint16_t,
    307 as uint16_t,
    318 as uint16_t,
    357 as uint16_t,
    360 as uint16_t,
    382 as uint16_t,
    447 as uint16_t,
    492 as uint16_t,
    504 as uint16_t,
    522 as uint16_t,
    537 as uint16_t,
    546 as uint16_t,
    547 as uint16_t,
]);
static cmdidxs2: GlobalCell<[[uint8_t; 26]; 26]> = GlobalCell::new([
    [
        0 as uint8_t,
        1 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        4 as uint8_t,
        5 as uint8_t,
        6 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        7 as uint8_t,
        16 as uint8_t,
        0 as uint8_t,
        17 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        2 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        5 as uint8_t,
        6 as uint8_t,
        7 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        8 as uint8_t,
        9 as uint8_t,
        10 as uint8_t,
        11 as uint8_t,
        12 as uint8_t,
        0 as uint8_t,
        13 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        22 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        3 as uint8_t,
        12 as uint8_t,
        16 as uint8_t,
        18 as uint8_t,
        20 as uint8_t,
        22 as uint8_t,
        25 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        34 as uint8_t,
        38 as uint8_t,
        41 as uint8_t,
        47 as uint8_t,
        58 as uint8_t,
        60 as uint8_t,
        61 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        62 as uint8_t,
        0 as uint8_t,
        65 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        8 as uint8_t,
        17 as uint8_t,
        0 as uint8_t,
        18 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        19 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        21 as uint8_t,
        22 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        1 as uint8_t,
        0 as uint8_t,
        2 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        7 as uint8_t,
        9 as uint8_t,
        10 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        16 as uint8_t,
        0 as uint8_t,
        17 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        15 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        8 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        14 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        1 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        2 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        4 as uint8_t,
        5 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        4 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        7 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        1 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        3 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        4 as uint8_t,
        0 as uint8_t,
        5 as uint8_t,
        6 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        13 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        14 as uint8_t,
        0 as uint8_t,
        16 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        1 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        1 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        3 as uint8_t,
        11 as uint8_t,
        15 as uint8_t,
        18 as uint8_t,
        19 as uint8_t,
        23 as uint8_t,
        26 as uint8_t,
        31 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        33 as uint8_t,
        36 as uint8_t,
        39 as uint8_t,
        43 as uint8_t,
        49 as uint8_t,
        0 as uint8_t,
        51 as uint8_t,
        60 as uint8_t,
        52 as uint8_t,
        53 as uint8_t,
        57 as uint8_t,
        59 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        1 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        7 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        10 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        16 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        2 as uint8_t,
        5 as uint8_t,
        7 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        14 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        1 as uint8_t,
        4 as uint8_t,
        0 as uint8_t,
        7 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        8 as uint8_t,
        0 as uint8_t,
        10 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        1 as uint8_t,
        3 as uint8_t,
        4 as uint8_t,
        0 as uint8_t,
        5 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        9 as uint8_t,
        11 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        16 as uint8_t,
        17 as uint8_t,
        26 as uint8_t,
        0 as uint8_t,
        27 as uint8_t,
        0 as uint8_t,
        28 as uint8_t,
        0 as uint8_t,
    ],
    [
        2 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        13 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        15 as uint8_t,
        0 as uint8_t,
        16 as uint8_t,
        21 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        2 as uint8_t,
        6 as uint8_t,
        15 as uint8_t,
        0 as uint8_t,
        17 as uint8_t,
        21 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        23 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        26 as uint8_t,
        28 as uint8_t,
        32 as uint8_t,
        36 as uint8_t,
        38 as uint8_t,
        0 as uint8_t,
        47 as uint8_t,
        0 as uint8_t,
        48 as uint8_t,
        0 as uint8_t,
        60 as uint8_t,
        61 as uint8_t,
        0 as uint8_t,
        62 as uint8_t,
        0 as uint8_t,
    ],
    [
        4 as uint8_t,
        0 as uint8_t,
        1 as uint8_t,
        0 as uint8_t,
        24 as uint8_t,
        25 as uint8_t,
        0 as uint8_t,
        26 as uint8_t,
        0 as uint8_t,
        27 as uint8_t,
        0 as uint8_t,
        28 as uint8_t,
        32 as uint8_t,
        35 as uint8_t,
        37 as uint8_t,
        38 as uint8_t,
        0 as uint8_t,
        39 as uint8_t,
        42 as uint8_t,
        0 as uint8_t,
        43 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        11 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        1 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        4 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        9 as uint8_t,
        12 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        15 as uint8_t,
        0 as uint8_t,
        16 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        2 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        3 as uint8_t,
        4 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        8 as uint8_t,
        0 as uint8_t,
        9 as uint8_t,
        10 as uint8_t,
        0 as uint8_t,
        12 as uint8_t,
        0 as uint8_t,
        13 as uint8_t,
        14 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        1 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        2 as uint8_t,
        5 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        7 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
]);
pub const true_0: c_int = 1 as c_int;
pub const false_0: c_int = 0 as c_int;
pub const RE_MAGIC: c_int = 1 as c_int;
