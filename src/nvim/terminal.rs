use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_free_object, cstr_as_string, dict_get_value, dict_set_var,
};
use crate::src::nvim::autocmd::{block_autocmds, has_event, is_autocmd_blocked, unblock_autocmds};
use crate::src::nvim::buffer::do_buffer;
use crate::src::nvim::cursor_shape::{parse_shape_opt, shape_table};
use crate::src::nvim::drawscreen::{
    redraw_statuslines, setcursor, show_cursor_info_later, showmode, unshowmode, update_screen,
};
use crate::src::nvim::eval::typval::{
    tv_dict_add_nr, tv_dict_set_keys_readonly, tv_list_alloc, tv_list_append_allocated_string,
    tv_list_append_list, tv_list_append_string,
};
use crate::src::nvim::eval::vars::{get_globvar_dict, set_vim_var_string};
use crate::src::nvim::eval_1::{eval_call_provider, get_v_event, restore_v_event};
use crate::src::nvim::event::multiqueue::{
    multiqueue_free, multiqueue_move_events, multiqueue_new, multiqueue_new_child,
    multiqueue_process_events, multiqueue_put_event,
};
use crate::src::nvim::event::time::{
    time_watcher_close, time_watcher_init, time_watcher_start, time_watcher_stop,
};
use crate::src::nvim::ex_docmd::do_cmdline;
use crate::src::nvim::getchar::{
    getcmdkeycmd, ins_char_typebuf, map_execute_lua, merge_modifiers, paste_repeat, ungetchars,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::schar_get_adv;
use crate::src::nvim::highlight::{hl_add_url, hl_combine_attr, hl_get_term_attr};
use crate::src::nvim::highlight_group::name_to_color;
use crate::src::nvim::main::{
    buffer_handles, clear_cmdline, exiting, got_int, main_loop, mapped_ctrl_c, mod_mask, mouse_col,
    mouse_grid, mouse_row, must_redraw, p_bg, redraw_cmdline, redraw_mode, restart_edit,
    stop_insert_mode, tpf_flags, vgetc_char, vgetc_mod_mask, window_handles, KeyTyped,
    RedrawingDisabled, State,
};
use crate::src::nvim::map::{mh_delete_ptr_t, mh_get_int, mh_get_ptr_t, mh_put_ptr_t};
use crate::src::nvim::mbyte::{mb_check_adjust_col, utf_ptr2char, utf_ptr2len};
use crate::src::nvim::memory::{
    strequal, xcalloc, xfree, xmalloc, xmemdup, xmemdupz, xrealloc, xstrdup,
};
use crate::src::nvim::mouse::do_mousescroll;
use crate::src::nvim::ops::clear_oparg;
use crate::src::nvim::option::set_option_value;
use crate::src::nvim::optionstr::free_string_option;
use crate::src::nvim::os::libc::{__assert_fail, abort, memcpy, memmove, memset, snprintf, strlen};
use crate::src::nvim::state::{may_trigger_modechanged, state_enter, state_handle_k_event};
use crate::src::nvim::strings::kv_do_printf;
pub use crate::src::nvim::types::{
    __pthread_internal_list, __pthread_list_t, __pthread_mutex_s, __pthread_rwlock_arch_t,
    __time_t, alist_T, argv_callback, auto_event, bhdr_T, blob_T, blobvar_S, blocknr_T, bufstate_T,
    chunksize_T, cmd_addr_T, cmdarg_T, cmdidx_T, colnr_T, cstack_T,
    cstack_T_cs_pend as C2Rust_Unnamed_34, cursorentry_T, dict_T, dictvar_S, diff_T, diffblock_S,
    disptick_T, dobuf_action_values, dobuf_start_values, eslist_T, eslist_elem, event_T, exarg,
    exarg_T, extmark_undo_vec_t, fcs_chars_T, float_T, fmark_T, fmarkv_T, funccall_S,
    funccall_S_fc_fixvar as C2Rust_Unnamed_10, funccall_T, garray_T, handle_T, hash_T, hashitem_T,
    hashtab_T, infoptr_T, int16_t, int32_t, int64_t, internal_proc_cb, intptr_t, key_extra,
    key_value_pair, lcs_chars_T, linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S,
    listwatch_T, llpos_T, loop_0, loop_0_children as C2Rust_Unnamed_25, lpos_T, mapblock,
    mapblock_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s, mtnode_s, multiqueue, object,
    object_data as C2Rust_Unnamed, oparg_T, partial_S, partial_T, pos_T, pos_save_T, proc,
    proc_exit_cb, proc_state_cb, proftime_T, pthread_mutex_t, pthread_rwlock_t, ptr_t, ptrdiff_t,
    qf_info_S, qf_info_T, queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T, rstream, sattr_T,
    save_v_event_T, schar_T, scid_T, sctx_T, size_t, ssize_t, state_check_callback,
    state_execute_callback, stream, stream_close_cb, stream_read_cb,
    stream_uv as C2Rust_Unnamed_27, stream_write_cb, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_8, syn_time_T, synblock_T, synstate_T, taggy_T,
    terminal_close_cb, terminal_read_pause_cb, terminal_resize_cb, terminal_resume_cb,
    terminal_write_cb, time_cb, time_t, time_watcher, typval_T, typval_vval_union, u_entry,
    u_entry_T, u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_12,
    u_header_uh_alt_prev as C2Rust_Unnamed_11, u_header_uh_next as C2Rust_Unnamed_14,
    u_header_uh_prev as C2Rust_Unnamed_13, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, uv__io_cb, uv__io_s, uv__io_t, uv__queue, uv_alloc_cb, uv_async_cb, uv_async_s,
    uv_async_s_u as C2Rust_Unnamed_22, uv_async_t, uv_buf_t, uv_close_cb, uv_connect_cb,
    uv_connect_s, uv_connect_t, uv_connection_cb, uv_file, uv_handle_s,
    uv_handle_s_u as C2Rust_Unnamed_17, uv_handle_t, uv_handle_type, uv_idle_cb, uv_idle_s,
    uv_idle_s_u as C2Rust_Unnamed_28, uv_idle_t, uv_loop_s,
    uv_loop_s_active_reqs as C2Rust_Unnamed_21, uv_loop_s_timer_heap as C2Rust_Unnamed_20,
    uv_loop_t, uv_mutex_t, uv_pipe_s, uv_pipe_s_u as C2Rust_Unnamed_30, uv_pipe_t, uv_read_cb,
    uv_req_type, uv_rwlock_t, uv_shutdown_cb, uv_shutdown_s, uv_shutdown_t, uv_signal_cb,
    uv_signal_s, uv_signal_s_tree_entry as C2Rust_Unnamed_18, uv_signal_s_u as C2Rust_Unnamed_19,
    uv_signal_t, uv_stream_s, uv_stream_s_u as C2Rust_Unnamed_26, uv_stream_t, uv_tcp_s,
    uv_tcp_s_u as C2Rust_Unnamed_29, uv_tcp_t, uv_timer_cb, uv_timer_s,
    uv_timer_s_node as C2Rust_Unnamed_23, uv_timer_s_u as C2Rust_Unnamed_24, uv_timer_t,
    varnumber_T, vim_state, virt_line, visualinfo_T, winopt_T, wline_T, xfmark_T, AdditionalData,
    AlignTextPos, Arena, Array, BoolVarValue, Boolean, BufUpdateCallbacks, Buffer, CMD_index,
    Callback, CallbackType, Callback_data as C2Rust_Unnamed_9, ChangedtickDictItem, CursorShape,
    DecorExt, DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_2, Dict, Error, ErrorType, Event, ExtmarkOp,
    ExtmarkUndoObject, FileID, Float, FloatAnchor, FloatRelative, GridView, HlAttrs, Integer,
    Intersection, KeyValuePair, LineGetter, Loop, LuaRef, MHPutStatus, MTKey, MTNode, MTPos,
    MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_int_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MarkAdjustMode, MarkTree, MotionType, MultiQueue, Object, ObjectType,
    OptIndex, OptInt, OptVal, OptValData, OptValType, Proc, ProcType, PutCallback, RStream,
    RgbValue, ScopeDictDictItem, ScopeType, ScreenCell, ScreenGrid, ScreenPen, Set_int,
    Set_int64_t, Set_ptr_t, Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_16, Stream, StringBuilder, String_0,
    TerminalOptions, TimeWatcher, Timestamp, TriState, VTerm, VTermAttr, VTermColor,
    VTermColor_indexed as C2Rust_Unnamed_5, VTermColor_rgb as C2Rust_Unnamed_6, VTermDamageSize,
    VTermKey, VTermModifier, VTermOutputCallback, VTermPos, VTermProp, VTermRect, VTermScreen,
    VTermScreenCallbacks, VTermScreenCell, VTermScreenCellAttrs, VTermSelectionCallbacks,
    VTermSelectionMask, VTermState, VTermStateFallbacks, VTermStringFragment, VTermTerminator,
    VTermValue, VTermValueType, VarLockStatus, VarType, VimState, VimVarIndex, VirtLines, VirtText,
    VirtTextChunk, VirtTextPos, WinConfig, WinSplit, WinStyle, Window, QUEUE,
};
use crate::src::nvim::ui::{
    ui_busy_start, ui_busy_stop, ui_cursor_shape, ui_flush, ui_mode_info_set, vim_beep,
};
use crate::src::nvim::vterm::keyboard::{
    vterm_keyboard_end_paste, vterm_keyboard_key, vterm_keyboard_start_paste,
    vterm_keyboard_unichar,
};
use crate::src::nvim::vterm::mouse::{vterm_mouse_button, vterm_mouse_move};
use crate::src::nvim::vterm::parser::vterm_input_write;
use crate::src::nvim::vterm::pen::{
    vterm_state_convert_color_to_rgb, vterm_state_set_palette_color, vterm_state_set_penattr,
};
use crate::src::nvim::vterm::screen::{
    vterm_obtain_screen, vterm_screen_enable_altscreen, vterm_screen_enable_reflow,
    vterm_screen_flush_damage, vterm_screen_get_cell, vterm_screen_reset,
    vterm_screen_set_callbacks, vterm_screen_set_damage_merge,
    vterm_screen_set_unrecognised_fallbacks,
};
use crate::src::nvim::vterm::state::{
    vterm_obtain_state, vterm_state_focus_in, vterm_state_focus_out,
    vterm_state_set_selection_callbacks, vterm_state_set_termprop,
};
use crate::src::nvim::vterm::vterm::{
    vterm_free, vterm_get_size, vterm_new, vterm_output_set_callback, vterm_set_size,
    vterm_set_utf8,
};
use crate::src::nvim::window::may_trigger_win_scrolled_resized;

// Phase-5a blacklist residue: this module keeps concrete local copies of
// types whose canonical form is opaque (file_buffer, window_S, ...), so
// these declarations cannot become `use` imports until the phase-8 rewrite.
// The copies are layout-identical to the canonical definitions (proven by
// the 5a parity suite); the nominal decl/decl mismatch is expected.
#[allow(clashing_extern_declarations)]
extern "C" {
    fn is_aucmd_win(win: *mut win_T) -> bool;
    fn aucmd_prepbuf(aco: *mut aco_save_T, buf: *mut buf_T);
    fn aucmd_restbuf(aco: *mut aco_save_T);
    fn apply_autocmds(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        buf: *mut buf_T,
    ) -> bool;
    fn apply_autocmds_group(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        group: ::core::ffi::c_int,
        buf: *mut buf_T,
        eap: *mut exarg_T,
        data: *mut Object,
    ) -> bool;
    fn appended_lines_buf(buf: *mut buf_T, lnum: linenr_T, count: linenr_T);
    fn deleted_lines_buf(buf: *mut buf_T, lnum: linenr_T, count: linenr_T);
    fn changed_lines(
        buf: *mut buf_T,
        lnum: linenr_T,
        col: colnr_T,
        lnume: linenr_T,
        xtra: linenr_T,
        do_buf_event: bool,
    );
    fn coladvance(wp: *mut win_T, wcol: colnr_T) -> ::core::ffi::c_int;
    fn redraw_later(wp: *mut win_T, type_0: ::core::ffi::c_int);
    fn redraw_buf_line_later(buf: *mut buf_T, line: linenr_T, force: bool);
    fn status_redraw_buf(buf: *mut buf_T);
    static firstwin: GlobalCell<*mut win_T>;
    static curwin: GlobalCell<*mut win_T>;
    static first_tabpage: GlobalCell<*mut tabpage_T>;
    static curtab: GlobalCell<*mut tabpage_T>;
    static curbuf: GlobalCell<*mut buf_T>;
    fn mark_adjust_buf(
        buf: *mut buf_T,
        line1: linenr_T,
        line2: linenr_T,
        amount: linenr_T,
        amount_after: linenr_T,
        adjust_folds: bool,
        mode: MarkAdjustMode,
        op: ExtmarkOp,
    );
    fn ml_append_buf(
        buf: *mut buf_T,
        lnum: linenr_T,
        line: *mut ::core::ffi::c_char,
        len: colnr_T,
        newfile: bool,
    ) -> ::core::ffi::c_int;
    fn ml_replace_buf(
        buf: *mut buf_T,
        lnum: linenr_T,
        line: *mut ::core::ffi::c_char,
        copy: bool,
        noalloc: bool,
    ) -> ::core::ffi::c_int;
    fn ml_delete_buf(buf: *mut buf_T, lnum: linenr_T, message: bool) -> ::core::ffi::c_int;
    fn mouse_find_win_inner(
        gridp: *mut ::core::ffi::c_int,
        rowp: *mut ::core::ffi::c_int,
        colp: *mut ::core::ffi::c_int,
    ) -> *mut win_T;
    fn set_topline(wp: *mut win_T, lnum: linenr_T);
    fn validate_cursor(wp: *mut win_T);
    fn win_col_off(wp: *mut win_T) -> ::core::ffi::c_int;
    fn curs_columns(wp: *mut win_T, may_scroll: ::core::ffi::c_int);
    fn win_valid(win: *const win_T) -> bool;
}
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct file_buffer {
    pub handle: handle_T,
    pub b_ml: memline_T,
    pub b_next: *mut buf_T,
    pub b_prev: *mut buf_T,
    pub b_nwindows: ::core::ffi::c_int,
    pub b_flags: ::core::ffi::c_int,
    pub b_locked: ::core::ffi::c_int,
    pub b_locked_split: ::core::ffi::c_int,
    pub b_ro_locked: ::core::ffi::c_int,
    pub b_ffname: *mut ::core::ffi::c_char,
    pub b_sfname: *mut ::core::ffi::c_char,
    pub b_fname: *mut ::core::ffi::c_char,
    pub file_id_valid: bool,
    pub file_id: FileID,
    pub b_changed: ::core::ffi::c_int,
    pub b_changed_invalid: bool,
    pub changedtick_di: ChangedtickDictItem,
    pub b_last_changedtick: varnumber_T,
    pub b_last_changedtick_i: varnumber_T,
    pub b_last_changedtick_pum: varnumber_T,
    pub b_saving: bool,
    pub b_mod_set: bool,
    pub b_mod_top: linenr_T,
    pub b_mod_bot: linenr_T,
    pub b_mod_xlines: linenr_T,
    pub b_wininfo: C2Rust_Unnamed_15,
    pub b_mod_tick_syn: disptick_T,
    pub b_mod_tick_decor: disptick_T,
    pub b_mtime: int64_t,
    pub b_mtime_ns: int64_t,
    pub b_mtime_read: int64_t,
    pub b_mtime_read_ns: int64_t,
    pub b_orig_size: uint64_t,
    pub b_orig_mode: ::core::ffi::c_int,
    pub b_last_used: time_t,
    pub b_namedm: [fmark_T; 26],
    pub b_visual: visualinfo_T,
    pub b_visual_mode_eval: ::core::ffi::c_int,
    pub b_last_cursor: fmark_T,
    pub b_last_insert: fmark_T,
    pub b_last_change: fmark_T,
    pub b_changelist: [fmark_T; 100],
    pub b_changelistlen: ::core::ffi::c_int,
    pub b_new_change: bool,
    pub b_chartab: [uint64_t; 4],
    pub b_maphash: [*mut mapblock_T; 256],
    pub b_first_abbr: *mut mapblock_T,
    pub b_ucmds: garray_T,
    pub b_op_start: pos_T,
    pub b_op_start_orig: pos_T,
    pub b_op_end: pos_T,
    pub b_marks_read: bool,
    pub b_modified_was_set: bool,
    pub b_did_filetype: bool,
    pub b_keep_filetype: bool,
    pub b_au_did_filetype: bool,
    pub b_u_oldhead: *mut u_header_T,
    pub b_u_newhead: *mut u_header_T,
    pub b_u_curhead: *mut u_header_T,
    pub b_u_numhead: ::core::ffi::c_int,
    pub b_u_synced: bool,
    pub b_u_seq_last: ::core::ffi::c_int,
    pub b_u_save_nr_last: ::core::ffi::c_int,
    pub b_u_seq_cur: ::core::ffi::c_int,
    pub b_u_time_cur: time_t,
    pub b_u_save_nr_cur: ::core::ffi::c_int,
    pub b_u_line_ptr: *mut ::core::ffi::c_char,
    pub b_u_line_lnum: linenr_T,
    pub b_u_line_colnr: colnr_T,
    pub b_scanned: bool,
    pub b_p_iminsert: OptInt,
    pub b_p_imsearch: OptInt,
    pub b_kmap_state: int16_t,
    pub b_kmap_ga: garray_T,
    pub b_p_initialized: bool,
    pub b_p_script_ctx: [sctx_T; 92],
    pub b_p_ac: ::core::ffi::c_int,
    pub b_p_ai: ::core::ffi::c_int,
    pub b_p_ai_nopaste: ::core::ffi::c_int,
    pub b_p_bkc: *mut ::core::ffi::c_char,
    pub b_bkc_flags: ::core::ffi::c_uint,
    pub b_p_ci: ::core::ffi::c_int,
    pub b_p_bin: ::core::ffi::c_int,
    pub b_p_bomb: ::core::ffi::c_int,
    pub b_p_bh: *mut ::core::ffi::c_char,
    pub b_p_bt: *mut ::core::ffi::c_char,
    pub b_p_busy: OptInt,
    pub b_has_qf_entry: ::core::ffi::c_int,
    pub b_p_bl: ::core::ffi::c_int,
    pub b_p_channel: OptInt,
    pub b_p_cin: ::core::ffi::c_int,
    pub b_p_cino: *mut ::core::ffi::c_char,
    pub b_p_cink: *mut ::core::ffi::c_char,
    pub b_p_cinw: *mut ::core::ffi::c_char,
    pub b_p_cinsd: *mut ::core::ffi::c_char,
    pub b_p_com: *mut ::core::ffi::c_char,
    pub b_p_cms: *mut ::core::ffi::c_char,
    pub b_p_cot: *mut ::core::ffi::c_char,
    pub b_cot_flags: ::core::ffi::c_uint,
    pub b_p_cpt: *mut ::core::ffi::c_char,
    pub b_p_cpt_cb: *mut Callback,
    pub b_p_cpt_count: ::core::ffi::c_int,
    pub b_p_cfu: *mut ::core::ffi::c_char,
    pub b_cfu_cb: Callback,
    pub b_p_ofu: *mut ::core::ffi::c_char,
    pub b_ofu_cb: Callback,
    pub b_p_tfu: *mut ::core::ffi::c_char,
    pub b_tfu_cb: Callback,
    pub b_p_ffu: *mut ::core::ffi::c_char,
    pub b_ffu_cb: Callback,
    pub b_p_eof: ::core::ffi::c_int,
    pub b_p_eol: ::core::ffi::c_int,
    pub b_p_fixeol: ::core::ffi::c_int,
    pub b_p_et: ::core::ffi::c_int,
    pub b_p_et_nobin: ::core::ffi::c_int,
    pub b_p_et_nopaste: ::core::ffi::c_int,
    pub b_p_fenc: *mut ::core::ffi::c_char,
    pub b_p_ff: *mut ::core::ffi::c_char,
    pub b_p_ft: *mut ::core::ffi::c_char,
    pub b_p_fo: *mut ::core::ffi::c_char,
    pub b_p_flp: *mut ::core::ffi::c_char,
    pub b_p_inf: ::core::ffi::c_int,
    pub b_p_isk: *mut ::core::ffi::c_char,
    pub b_p_def: *mut ::core::ffi::c_char,
    pub b_p_inc: *mut ::core::ffi::c_char,
    pub b_p_inex: *mut ::core::ffi::c_char,
    pub b_p_inex_flags: uint32_t,
    pub b_p_inde: *mut ::core::ffi::c_char,
    pub b_p_inde_flags: uint32_t,
    pub b_p_indk: *mut ::core::ffi::c_char,
    pub b_p_fp: *mut ::core::ffi::c_char,
    pub b_p_fex: *mut ::core::ffi::c_char,
    pub b_p_fex_flags: uint32_t,
    pub b_p_fs: ::core::ffi::c_int,
    pub b_p_kp: *mut ::core::ffi::c_char,
    pub b_p_lisp: ::core::ffi::c_int,
    pub b_p_lop: *mut ::core::ffi::c_char,
    pub b_p_menc: *mut ::core::ffi::c_char,
    pub b_p_mps: *mut ::core::ffi::c_char,
    pub b_p_ml: ::core::ffi::c_int,
    pub b_p_ml_nobin: ::core::ffi::c_int,
    pub b_p_ma: ::core::ffi::c_int,
    pub b_p_nf: *mut ::core::ffi::c_char,
    pub b_p_pi: ::core::ffi::c_int,
    pub b_p_qe: *mut ::core::ffi::c_char,
    pub b_p_ro: ::core::ffi::c_int,
    pub b_p_sw: OptInt,
    pub b_p_scbk: OptInt,
    pub b_p_si: ::core::ffi::c_int,
    pub b_p_sts: OptInt,
    pub b_p_sts_nopaste: OptInt,
    pub b_p_sua: *mut ::core::ffi::c_char,
    pub b_p_swf: ::core::ffi::c_int,
    pub b_p_smc: OptInt,
    pub b_p_syn: *mut ::core::ffi::c_char,
    pub b_p_ts: OptInt,
    pub b_p_tw: OptInt,
    pub b_p_tw_nobin: OptInt,
    pub b_p_tw_nopaste: OptInt,
    pub b_p_wm: OptInt,
    pub b_p_wm_nobin: OptInt,
    pub b_p_wm_nopaste: OptInt,
    pub b_p_vsts: *mut ::core::ffi::c_char,
    pub b_p_vsts_array: *mut colnr_T,
    pub b_p_vsts_nopaste: *mut ::core::ffi::c_char,
    pub b_p_vts: *mut ::core::ffi::c_char,
    pub b_p_vts_array: *mut colnr_T,
    pub b_p_keymap: *mut ::core::ffi::c_char,
    pub b_p_gefm: *mut ::core::ffi::c_char,
    pub b_p_gp: *mut ::core::ffi::c_char,
    pub b_p_mp: *mut ::core::ffi::c_char,
    pub b_p_efm: *mut ::core::ffi::c_char,
    pub b_p_ep: *mut ::core::ffi::c_char,
    pub b_p_path: *mut ::core::ffi::c_char,
    pub b_p_ar: ::core::ffi::c_int,
    pub b_p_tags: *mut ::core::ffi::c_char,
    pub b_p_tc: *mut ::core::ffi::c_char,
    pub b_tc_flags: ::core::ffi::c_uint,
    pub b_p_dict: *mut ::core::ffi::c_char,
    pub b_p_dia: *mut ::core::ffi::c_char,
    pub b_p_tsr: *mut ::core::ffi::c_char,
    pub b_p_tsrfu: *mut ::core::ffi::c_char,
    pub b_tsrfu_cb: Callback,
    pub b_p_ul: OptInt,
    pub b_p_udf: ::core::ffi::c_int,
    pub b_p_lw: *mut ::core::ffi::c_char,
    pub b_ind_level: ::core::ffi::c_int,
    pub b_ind_open_imag: ::core::ffi::c_int,
    pub b_ind_no_brace: ::core::ffi::c_int,
    pub b_ind_first_open: ::core::ffi::c_int,
    pub b_ind_open_extra: ::core::ffi::c_int,
    pub b_ind_close_extra: ::core::ffi::c_int,
    pub b_ind_open_left_imag: ::core::ffi::c_int,
    pub b_ind_jump_label: ::core::ffi::c_int,
    pub b_ind_case: ::core::ffi::c_int,
    pub b_ind_case_code: ::core::ffi::c_int,
    pub b_ind_case_break: ::core::ffi::c_int,
    pub b_ind_param: ::core::ffi::c_int,
    pub b_ind_func_type: ::core::ffi::c_int,
    pub b_ind_comment: ::core::ffi::c_int,
    pub b_ind_in_comment: ::core::ffi::c_int,
    pub b_ind_in_comment2: ::core::ffi::c_int,
    pub b_ind_cpp_baseclass: ::core::ffi::c_int,
    pub b_ind_continuation: ::core::ffi::c_int,
    pub b_ind_unclosed: ::core::ffi::c_int,
    pub b_ind_unclosed2: ::core::ffi::c_int,
    pub b_ind_unclosed_noignore: ::core::ffi::c_int,
    pub b_ind_unclosed_wrapped: ::core::ffi::c_int,
    pub b_ind_unclosed_whiteok: ::core::ffi::c_int,
    pub b_ind_matching_paren: ::core::ffi::c_int,
    pub b_ind_paren_prev: ::core::ffi::c_int,
    pub b_ind_maxparen: ::core::ffi::c_int,
    pub b_ind_maxcomment: ::core::ffi::c_int,
    pub b_ind_scopedecl: ::core::ffi::c_int,
    pub b_ind_scopedecl_code: ::core::ffi::c_int,
    pub b_ind_java: ::core::ffi::c_int,
    pub b_ind_js: ::core::ffi::c_int,
    pub b_ind_keep_case_label: ::core::ffi::c_int,
    pub b_ind_hash_comment: ::core::ffi::c_int,
    pub b_ind_cpp_namespace: ::core::ffi::c_int,
    pub b_ind_if_for_while: ::core::ffi::c_int,
    pub b_ind_cpp_extern_c: ::core::ffi::c_int,
    pub b_ind_pragma: ::core::ffi::c_int,
    pub b_no_eol_lnum: linenr_T,
    pub b_start_eof: ::core::ffi::c_int,
    pub b_start_eol: ::core::ffi::c_int,
    pub b_start_ffc: ::core::ffi::c_int,
    pub b_start_fenc: *mut ::core::ffi::c_char,
    pub b_bad_char: ::core::ffi::c_int,
    pub b_start_bomb: ::core::ffi::c_int,
    pub b_bufvar: ScopeDictDictItem,
    pub b_vars: *mut dict_T,
    pub b_may_swap: bool,
    pub b_did_warn: bool,
    pub b_help: bool,
    pub b_spell: bool,
    pub b_prompt_text: *mut ::core::ffi::c_char,
    pub b_prompt_callback: Callback,
    pub b_prompt_interrupt: Callback,
    pub b_prompt_append_new_line: bool,
    pub b_prompt_insert: ::core::ffi::c_int,
    pub b_prompt_start: fmark_T,
    pub b_s: synblock_T,
    pub b_signcols: C2Rust_Unnamed_7,
    pub terminal: *mut Terminal,
    pub additional_data: *mut AdditionalData,
    pub b_mapped_ctrl_c: ::core::ffi::c_int,
    pub b_marktree: [MarkTree; 1],
    pub b_extmark_ns: [Map_uint32_t_uint32_t; 1],
    pub b_prev_line_count: ::core::ffi::c_int,
    pub update_channels: C2Rust_Unnamed_1,
    pub update_callbacks: C2Rust_Unnamed_0,
    pub update_need_codepoints: bool,
    pub deleted_bytes: size_t,
    pub deleted_bytes2: size_t,
    pub deleted_codepoints: size_t,
    pub deleted_codeunits: size_t,
    pub flush_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_0 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut BufUpdateCallbacks,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_1 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut uint64_t,
}
pub const kVPosWinCol: VirtTextPos = 5;
pub const kVPosRightAlign: VirtTextPos = 4;
pub const kVPosOverlay: VirtTextPos = 3;
pub const kVPosInline: VirtTextPos = 2;
pub const kVPosEndOfLineRightAlign: VirtTextPos = 1;
pub const kVPosEndOfLine: VirtTextPos = 0;
pub type Terminal = terminal;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct terminal {
    pub opts: TerminalOptions,
    pub vt: *mut VTerm,
    pub vts: *mut VTermScreen,
    pub textbuf: [::core::ffi::c_char; 8191],
    pub sb_buffer: *mut *mut ScrollbackLine,
    pub sb_current: size_t,
    pub sb_size: size_t,
    pub sb_pending: ::core::ffi::c_int,
    pub sb_deleted: size_t,
    pub old_sb_deleted: size_t,
    pub old_height: ::core::ffi::c_int,
    pub title: *mut ::core::ffi::c_char,
    pub title_len: size_t,
    pub title_size: size_t,
    pub buf_handle: handle_T,
    pub in_altscreen: bool,
    pub suspended: bool,
    pub closed: bool,
    pub destroy: bool,
    pub forward_mouse: bool,
    pub invalid_start: ::core::ffi::c_int,
    pub invalid_end: ::core::ffi::c_int,
    pub cursor: C2Rust_Unnamed_4,
    pub pending: C2Rust_Unnamed_3,
    pub streamed_paste: bool,
    pub theme_updates: bool,
    pub synchronized_output: bool,
    pub sync_flush_pending: bool,
    pub color_set: [bool; 16],
    pub selection_buffer: *mut ::core::ffi::c_char,
    pub selection: StringBuilder,
    pub termrequest_buffer: StringBuilder,
    pub termrequest_terminator: VTermTerminator,
    pub refcount: size_t,
}
pub const VTERM_TERMINATOR_ST: VTermTerminator = 1;
pub const VTERM_TERMINATOR_BEL: VTermTerminator = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_3 {
    pub resize: bool,
    pub cursor: bool,
    pub send: *mut StringBuilder,
    pub events: *mut MultiQueue,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_4 {
    pub row: ::core::ffi::c_int,
    pub col: ::core::ffi::c_int,
    pub shape: ::core::ffi::c_int,
    pub visible: bool,
    pub blink: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ScrollbackLine {
    pub cols: size_t,
    pub cells: [VTermScreenCell; 0],
}
pub const VTERM_N_DAMAGES: VTermDamageSize = 4;
pub const VTERM_DAMAGE_SCROLL: VTermDamageSize = 3;
pub const VTERM_DAMAGE_SCREEN: VTermDamageSize = 2;
pub const VTERM_DAMAGE_ROW: VTermDamageSize = 1;
pub const VTERM_DAMAGE_CELL: VTermDamageSize = 0;
pub const VTERM_N_PROPS: VTermProp = 12;
pub const VTERM_PROP_SYNCOUTPUT: VTermProp = 11;
pub const VTERM_PROP_THEMEUPDATES: VTermProp = 10;
pub const VTERM_PROP_FOCUSREPORT: VTermProp = 9;
pub const VTERM_PROP_MOUSE: VTermProp = 8;
pub const VTERM_PROP_CURSORSHAPE: VTermProp = 7;
pub const VTERM_PROP_REVERSE: VTermProp = 6;
pub const VTERM_PROP_ICONNAME: VTermProp = 5;
pub const VTERM_PROP_TITLE: VTermProp = 4;
pub const VTERM_PROP_ALTSCREEN: VTermProp = 3;
pub const VTERM_PROP_CURSORBLINK: VTermProp = 2;
pub const VTERM_PROP_CURSORVISIBLE: VTermProp = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_7 {
    pub max: ::core::ffi::c_int,
    pub last_max: ::core::ffi::c_int,
    pub count: [::core::ffi::c_int; 9],
    pub autom: bool,
}
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_15 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut WinInfo,
}
pub type WinInfo = wininfo_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wininfo_S {
    pub wi_win: *mut win_T,
    pub wi_mark: fmark_T,
    pub wi_optset: bool,
    pub wi_opt: winopt_T,
    pub wi_fold_manual: bool,
    pub wi_folds: garray_T,
    pub wi_changelistidx: ::core::ffi::c_int,
}
pub type win_T = window_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_S {
    pub handle: handle_T,
    pub w_buffer: *mut buf_T,
    pub w_s: *mut synblock_T,
    pub w_ns_hl: ::core::ffi::c_int,
    pub w_ns_hl_winhl: ::core::ffi::c_int,
    pub w_ns_hl_active: ::core::ffi::c_int,
    pub w_ns_hl_attr: *mut ::core::ffi::c_int,
    pub w_ns_set: Set_uint32_t,
    pub w_hl_id_normal: ::core::ffi::c_int,
    pub w_hl_attr_normal: ::core::ffi::c_int,
    pub w_hl_attr_normalnc: ::core::ffi::c_int,
    pub w_hl_needs_update: ::core::ffi::c_int,
    pub w_prev: *mut win_T,
    pub w_next: *mut win_T,
    pub w_locked: bool,
    pub w_frame: *mut frame_T,
    pub w_cursor: pos_T,
    pub w_curswant: colnr_T,
    pub w_set_curswant: ::core::ffi::c_int,
    pub w_cursorline: linenr_T,
    pub w_last_cursorline: linenr_T,
    pub w_old_visual_mode: ::core::ffi::c_char,
    pub w_old_cursor_lnum: linenr_T,
    pub w_old_cursor_fcol: colnr_T,
    pub w_old_cursor_lcol: colnr_T,
    pub w_old_visual_lnum: linenr_T,
    pub w_old_visual_col: colnr_T,
    pub w_old_curswant: colnr_T,
    pub w_last_cursor_lnum_rnu: linenr_T,
    pub w_p_lcs_chars: lcs_chars_T,
    pub w_p_fcs_chars: fcs_chars_T,
    pub w_topline: linenr_T,
    pub w_topline_was_set: ::core::ffi::c_char,
    pub w_topfill: ::core::ffi::c_int,
    pub w_old_topfill: ::core::ffi::c_int,
    pub w_botfill: bool,
    pub w_old_botfill: bool,
    pub w_leftcol: colnr_T,
    pub w_skipcol: colnr_T,
    pub w_last_topline: linenr_T,
    pub w_last_topfill: ::core::ffi::c_int,
    pub w_last_leftcol: colnr_T,
    pub w_last_skipcol: colnr_T,
    pub w_last_width: ::core::ffi::c_int,
    pub w_last_height: ::core::ffi::c_int,
    pub w_winrow: ::core::ffi::c_int,
    pub w_height: ::core::ffi::c_int,
    pub w_prev_winrow: ::core::ffi::c_int,
    pub w_prev_height: ::core::ffi::c_int,
    pub w_status_height: ::core::ffi::c_int,
    pub w_winbar_height: ::core::ffi::c_int,
    pub w_wincol: ::core::ffi::c_int,
    pub w_width: ::core::ffi::c_int,
    pub w_hsep_height: ::core::ffi::c_int,
    pub w_vsep_width: ::core::ffi::c_int,
    pub w_save_cursor: pos_save_T,
    pub w_do_win_fix_cursor: bool,
    pub w_winrow_off: ::core::ffi::c_int,
    pub w_wincol_off: ::core::ffi::c_int,
    pub w_view_height: ::core::ffi::c_int,
    pub w_view_width: ::core::ffi::c_int,
    pub w_height_request: ::core::ffi::c_int,
    pub w_width_request: ::core::ffi::c_int,
    pub w_border_adj: [::core::ffi::c_int; 4],
    pub w_height_outer: ::core::ffi::c_int,
    pub w_width_outer: ::core::ffi::c_int,
    pub w_valid: ::core::ffi::c_int,
    pub w_valid_cursor: pos_T,
    pub w_valid_leftcol: colnr_T,
    pub w_valid_skipcol: colnr_T,
    pub w_viewport_invalid: bool,
    pub w_viewport_last_topline: linenr_T,
    pub w_viewport_last_botline: linenr_T,
    pub w_viewport_last_topfill: linenr_T,
    pub w_viewport_last_skipcol: linenr_T,
    pub w_cline_height: ::core::ffi::c_int,
    pub w_cline_folded: bool,
    pub w_cline_row: ::core::ffi::c_int,
    pub w_virtcol: colnr_T,
    pub w_wrow: ::core::ffi::c_int,
    pub w_wcol: ::core::ffi::c_int,
    pub w_botline: linenr_T,
    pub w_empty_rows: ::core::ffi::c_int,
    pub w_filler_rows: ::core::ffi::c_int,
    pub w_lines_valid: ::core::ffi::c_int,
    pub w_lines: *mut wline_T,
    pub w_lines_size: ::core::ffi::c_int,
    pub w_folds: garray_T,
    pub w_fold_manual: bool,
    pub w_foldinvalid: bool,
    pub w_nrwidth: ::core::ffi::c_int,
    pub w_scwidth: ::core::ffi::c_int,
    pub w_minscwidth: ::core::ffi::c_int,
    pub w_maxscwidth: ::core::ffi::c_int,
    pub w_redr_type: ::core::ffi::c_int,
    pub w_upd_rows: ::core::ffi::c_int,
    pub w_redraw_top: linenr_T,
    pub w_redraw_bot: linenr_T,
    pub w_redr_status: bool,
    pub w_redr_border: bool,
    pub w_redr_statuscol: bool,
    pub w_display_tick: disptick_T,
    pub w_stl_cursor: pos_T,
    pub w_stl_virtcol: colnr_T,
    pub w_stl_topline: linenr_T,
    pub w_stl_line_count: linenr_T,
    pub w_stl_topfill: ::core::ffi::c_int,
    pub w_stl_empty: ::core::ffi::c_char,
    pub w_stl_recording: ::core::ffi::c_int,
    pub w_stl_state: ::core::ffi::c_int,
    pub w_stl_visual_mode: ::core::ffi::c_int,
    pub w_stl_visual_pos: pos_T,
    pub w_alt_fnum: ::core::ffi::c_int,
    pub w_alist: *mut alist_T,
    pub w_arg_idx: ::core::ffi::c_int,
    pub w_arg_idx_invalid: ::core::ffi::c_int,
    pub w_localdir: *mut ::core::ffi::c_char,
    pub w_prevdir: *mut ::core::ffi::c_char,
    pub w_onebuf_opt: winopt_T,
    pub w_allbuf_opt: winopt_T,
    pub w_p_cc_cols: *mut ::core::ffi::c_int,
    pub w_p_culopt_flags: uint8_t,
    pub w_briopt_min: ::core::ffi::c_int,
    pub w_briopt_shift: ::core::ffi::c_int,
    pub w_briopt_sbr: bool,
    pub w_briopt_list: ::core::ffi::c_int,
    pub w_briopt_vcol: ::core::ffi::c_int,
    pub w_scbind_pos: ::core::ffi::c_int,
    pub w_winvar: ScopeDictDictItem,
    pub w_vars: *mut dict_T,
    pub w_pcmark: pos_T,
    pub w_prev_pcmark: pos_T,
    pub w_jumplist: [xfmark_T; 100],
    pub w_jumplistlen: ::core::ffi::c_int,
    pub w_jumplistidx: ::core::ffi::c_int,
    pub w_changelistidx: ::core::ffi::c_int,
    pub w_match_head: *mut matchitem_T,
    pub w_next_match_id: ::core::ffi::c_int,
    pub w_tagstack: [taggy_T; 20],
    pub w_tagstackidx: ::core::ffi::c_int,
    pub w_tagstacklen: ::core::ffi::c_int,
    pub w_grid: GridView,
    pub w_grid_alloc: ScreenGrid,
    pub w_pos_changed: bool,
    pub w_floating: bool,
    pub w_float_is_info: bool,
    pub w_config: WinConfig,
    pub w_fraction: ::core::ffi::c_int,
    pub w_prev_fraction_row: ::core::ffi::c_int,
    pub w_nrwidth_line_count: linenr_T,
    pub w_statuscol_line_count: linenr_T,
    pub w_nrwidth_width: ::core::ffi::c_int,
    pub w_llist: *mut qf_info_T,
    pub w_llist_ref: *mut qf_info_T,
    pub w_status_click_defs: *mut StlClickDefinition,
    pub w_status_click_defs_size: size_t,
    pub w_winbar_click_defs: *mut StlClickDefinition,
    pub w_winbar_click_defs_size: size_t,
    pub w_statuscol_click_defs: *mut StlClickDefinition,
    pub w_statuscol_click_defs_size: size_t,
}
pub const kStlClickFuncRun: C2Rust_Unnamed_16 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_16 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_16 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_16 = 0;
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
pub type matchitem_T = matchitem;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct matchitem {
    pub mit_next: *mut matchitem_T,
    pub mit_id: ::core::ffi::c_int,
    pub mit_priority: ::core::ffi::c_int,
    pub mit_pattern: *mut ::core::ffi::c_char,
    pub mit_match: regmmatch_T,
    pub mit_pos_array: *mut llpos_T,
    pub mit_pos_count: ::core::ffi::c_int,
    pub mit_pos_cur: ::core::ffi::c_int,
    pub mit_toplnum: linenr_T,
    pub mit_botlnum: linenr_T,
    pub mit_hl: match_T,
    pub mit_hlg_id: ::core::ffi::c_int,
    pub mit_conceal_char: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct match_T {
    pub rm: regmmatch_T,
    pub buf: *mut buf_T,
    pub lnum: linenr_T,
    pub attr: ::core::ffi::c_int,
    pub attr_cur: ::core::ffi::c_int,
    pub first_lnum: linenr_T,
    pub startcol: colnr_T,
    pub endcol: colnr_T,
    pub is_addpos: bool,
    pub has_cursor: bool,
    pub tm: proftime_T,
}
pub type buf_T = file_buffer;
pub type frame_T = frame_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct frame_S {
    pub fr_layout: ::core::ffi::c_char,
    pub fr_width: ::core::ffi::c_int,
    pub fr_newwidth: ::core::ffi::c_int,
    pub fr_height: ::core::ffi::c_int,
    pub fr_newheight: ::core::ffi::c_int,
    pub fr_parent: *mut frame_T,
    pub fr_next: *mut frame_T,
    pub fr_prev: *mut frame_T,
    pub fr_child: *mut frame_T,
    pub fr_win: *mut win_T,
}
pub const MF_DIRTY_YES_NOSYNC: mfdirty_T = 2;
pub const MF_DIRTY_YES: mfdirty_T = 1;
pub const MF_DIRTY_NO: mfdirty_T = 0;
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
pub const kProcTypePty: ProcType = 1;
pub const kProcTypeUv: ProcType = 0;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed_31 = 2147483647;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub const HL_GLOBAL: C2Rust_Unnamed_32 = 16384;
pub const HL_DEFAULT: C2Rust_Unnamed_32 = 8192;
pub const HL_FG_INDEXED: C2Rust_Unnamed_32 = 4096;
pub const HL_BG_INDEXED: C2Rust_Unnamed_32 = 2048;
pub const HL_NOCOMBINE: C2Rust_Unnamed_32 = 1024;
pub const HL_OVERLINE: C2Rust_Unnamed_32 = 131072;
pub const HL_CONCEALED: C2Rust_Unnamed_32 = 65536;
pub const HL_BLINK: C2Rust_Unnamed_32 = 32768;
pub const HL_DIM: C2Rust_Unnamed_32 = 512;
pub const HL_ALTFONT: C2Rust_Unnamed_32 = 256;
pub const HL_STRIKETHROUGH: C2Rust_Unnamed_32 = 128;
pub const HL_STANDOUT: C2Rust_Unnamed_32 = 64;
pub const HL_UNDERDASHED: C2Rust_Unnamed_32 = 40;
pub const HL_UNDERDOTTED: C2Rust_Unnamed_32 = 32;
pub const HL_UNDERDOUBLE: C2Rust_Unnamed_32 = 24;
pub const HL_UNDERCURL: C2Rust_Unnamed_32 = 16;
pub const HL_UNDERLINE: C2Rust_Unnamed_32 = 8;
pub const HL_UNDERLINE_MASK: C2Rust_Unnamed_32 = 56;
pub const HL_ITALIC: C2Rust_Unnamed_32 = 4;
pub const HL_BOLD: C2Rust_Unnamed_32 = 2;
pub const HL_INVERSE: C2Rust_Unnamed_32 = 1;
pub const kMHNewKeyRealloc: MHPutStatus = 2;
pub const kMHNewKeyDidFit: MHPutStatus = 1;
pub const kMHExisting: MHPutStatus = 0;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_int;
pub const BACKWARD_FILE: C2Rust_Unnamed_33 = -3;
pub const FORWARD_FILE: C2Rust_Unnamed_33 = 3;
pub const BACKWARD: C2Rust_Unnamed_33 = -1;
pub const FORWARD: C2Rust_Unnamed_33 = 1;
pub const kDirectionNotSet: C2Rust_Unnamed_33 = 0;
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
pub const kExtmarkUndoNoRedo: ExtmarkOp = 3;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
pub const kMarkAdjustTerm: MarkAdjustMode = 2;
pub const kMarkAdjustApi: MarkAdjustMode = 1;
pub const kMarkAdjustNormal: MarkAdjustMode = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bufref_T {
    pub br_buf: *mut buf_T,
    pub br_fnum: ::core::ffi::c_int,
    pub br_buf_free_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tabpage_S {
    pub handle: handle_T,
    pub tp_next: *mut tabpage_T,
    pub tp_topframe: *mut frame_T,
    pub tp_curwin: *mut win_T,
    pub tp_prevwin: *mut win_T,
    pub tp_firstwin: *mut win_T,
    pub tp_lastwin: *mut win_T,
    pub tp_old_Rows_avail: int64_t,
    pub tp_old_Columns: int64_t,
    pub tp_ch_used: OptInt,
    pub tp_did_tabclosedpre: bool,
    pub tp_first_diff: *mut diff_T,
    pub tp_diffbuf: [*mut buf_T; 8],
    pub tp_diff_invalid: ::core::ffi::c_int,
    pub tp_diff_update: ::core::ffi::c_int,
    pub tp_snapshot: [*mut frame_T; 3],
    pub tp_winvar: ScopeDictDictItem,
    pub tp_vars: *mut dict_T,
    pub tp_localdir: *mut ::core::ffi::c_char,
    pub tp_prevdir: *mut ::core::ffi::c_char,
}
pub type tabpage_T = tabpage_S;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct aco_save_T {
    pub use_aucmd_win_idx: ::core::ffi::c_int,
    pub save_curwin_handle: handle_T,
    pub new_curwin_handle: handle_T,
    pub save_prevwin_handle: handle_T,
    pub new_curbuf: bufref_T,
    pub tp_localdir: *mut ::core::ffi::c_char,
    pub globaldir: *mut ::core::ffi::c_char,
    pub save_VIsual_active: bool,
    pub save_prompt_insert: ::core::ffi::c_int,
}
pub type C2Rust_Unnamed_35 = ::core::ffi::c_int;
pub const AUGROUP_DELETED: C2Rust_Unnamed_35 = -4;
pub const AUGROUP_ALL: C2Rust_Unnamed_35 = -3;
pub const AUGROUP_ERROR: C2Rust_Unnamed_35 = -2;
pub const AUGROUP_DEFAULT: C2Rust_Unnamed_35 = -1;
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_DEL: dobuf_action_values = 3;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub const DOBUF_SPLIT: dobuf_action_values = 1;
pub const DOBUF_GOTO: dobuf_action_values = 0;
pub const DOBUF_MOD: dobuf_start_values = 3;
pub const DOBUF_LAST: dobuf_start_values = 2;
pub const DOBUF_FIRST: dobuf_start_values = 1;
pub const DOBUF_CURRENT: dobuf_start_values = 0;
pub type C2Rust_Unnamed_36 = ::core::ffi::c_uint;
pub const SHAPE_IDX_COUNT: C2Rust_Unnamed_36 = 18;
pub const SHAPE_IDX_TERM: C2Rust_Unnamed_36 = 17;
pub const SHAPE_IDX_SM: C2Rust_Unnamed_36 = 16;
pub const SHAPE_IDX_MOREL: C2Rust_Unnamed_36 = 15;
pub const SHAPE_IDX_MORE: C2Rust_Unnamed_36 = 14;
pub const SHAPE_IDX_VDRAG: C2Rust_Unnamed_36 = 13;
pub const SHAPE_IDX_VSEP: C2Rust_Unnamed_36 = 12;
pub const SHAPE_IDX_SDRAG: C2Rust_Unnamed_36 = 11;
pub const SHAPE_IDX_STATUS: C2Rust_Unnamed_36 = 10;
pub const SHAPE_IDX_CLINE: C2Rust_Unnamed_36 = 9;
pub const SHAPE_IDX_VE: C2Rust_Unnamed_36 = 8;
pub const SHAPE_IDX_O: C2Rust_Unnamed_36 = 7;
pub const SHAPE_IDX_CR: C2Rust_Unnamed_36 = 6;
pub const SHAPE_IDX_CI: C2Rust_Unnamed_36 = 5;
pub const SHAPE_IDX_C: C2Rust_Unnamed_36 = 4;
pub const SHAPE_IDX_R: C2Rust_Unnamed_36 = 3;
pub const SHAPE_IDX_I: C2Rust_Unnamed_36 = 2;
pub const SHAPE_IDX_V: C2Rust_Unnamed_36 = 1;
pub const SHAPE_IDX_N: C2Rust_Unnamed_36 = 0;
pub const SHAPE_VER: CursorShape = 2;
pub const SHAPE_HOR: CursorShape = 1;
pub const SHAPE_BLOCK: CursorShape = 0;
pub type C2Rust_Unnamed_37 = ::core::ffi::c_uint;
pub const TERM_ATTRS_MAX: C2Rust_Unnamed_37 = 1024;
pub type C2Rust_Unnamed_38 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_38 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_38 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_38 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_38 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_38 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_38 = 20;
pub const UPD_VALID: C2Rust_Unnamed_38 = 10;
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
pub type C2Rust_Unnamed_39 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_39 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_39 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_39 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_39 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_39 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_39 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_39 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_39 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_39 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_39 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_39 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_39 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_39 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_39 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_39 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_39 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_39 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_39 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_39 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_39 = 1;
pub type C2Rust_Unnamed_40 = ::core::ffi::c_uint;
pub const kOptBoFlagWildmode: C2Rust_Unnamed_40 = 524288;
pub const kOptBoFlagTerm: C2Rust_Unnamed_40 = 262144;
pub const kOptBoFlagSpell: C2Rust_Unnamed_40 = 131072;
pub const kOptBoFlagShell: C2Rust_Unnamed_40 = 65536;
pub const kOptBoFlagRegister: C2Rust_Unnamed_40 = 32768;
pub const kOptBoFlagOperator: C2Rust_Unnamed_40 = 16384;
pub const kOptBoFlagShowmatch: C2Rust_Unnamed_40 = 8192;
pub const kOptBoFlagMess: C2Rust_Unnamed_40 = 4096;
pub const kOptBoFlagLang: C2Rust_Unnamed_40 = 2048;
pub const kOptBoFlagInsertmode: C2Rust_Unnamed_40 = 1024;
pub const kOptBoFlagHangul: C2Rust_Unnamed_40 = 512;
pub const kOptBoFlagEx: C2Rust_Unnamed_40 = 256;
pub const kOptBoFlagEsc: C2Rust_Unnamed_40 = 128;
pub const kOptBoFlagError: C2Rust_Unnamed_40 = 64;
pub const kOptBoFlagCtrlg: C2Rust_Unnamed_40 = 32;
pub const kOptBoFlagCopy: C2Rust_Unnamed_40 = 16;
pub const kOptBoFlagComplete: C2Rust_Unnamed_40 = 8;
pub const kOptBoFlagCursor: C2Rust_Unnamed_40 = 4;
pub const kOptBoFlagBackspace: C2Rust_Unnamed_40 = 2;
pub const kOptBoFlagAll: C2Rust_Unnamed_40 = 1;
pub type C2Rust_Unnamed_41 = ::core::ffi::c_uint;
pub const kOptCuloptFlagNumber: C2Rust_Unnamed_41 = 4;
pub const kOptCuloptFlagScreenline: C2Rust_Unnamed_41 = 2;
pub const kOptCuloptFlagLine: C2Rust_Unnamed_41 = 1;
pub type C2Rust_Unnamed_42 = ::core::ffi::c_uint;
pub const kOptTpfFlagC1: C2Rust_Unnamed_42 = 64;
pub const kOptTpfFlagC0: C2Rust_Unnamed_42 = 32;
pub const kOptTpfFlagDEL: C2Rust_Unnamed_42 = 16;
pub const kOptTpfFlagESC: C2Rust_Unnamed_42 = 8;
pub const kOptTpfFlagFF: C2Rust_Unnamed_42 = 4;
pub const kOptTpfFlagHT: C2Rust_Unnamed_42 = 2;
pub const kOptTpfFlagBS: C2Rust_Unnamed_42 = 1;
pub const KE_WILD: key_extra = 108;
pub const KE_COMMAND: key_extra = 104;
pub const KE_LUA: key_extra = 103;
pub const KE_EVENT: key_extra = 102;
pub const KE_MOUSEMOVE: key_extra = 100;
pub const KE_NOP: key_extra = 97;
pub const KE_DROP: key_extra = 95;
pub const KE_X2RELEASE: key_extra = 94;
pub const KE_X2DRAG: key_extra = 93;
pub const KE_X2MOUSE: key_extra = 92;
pub const KE_X1RELEASE: key_extra = 91;
pub const KE_X1DRAG: key_extra = 90;
pub const KE_X1MOUSE: key_extra = 89;
pub const KE_C_END: key_extra = 88;
pub const KE_C_HOME: key_extra = 87;
pub const KE_C_RIGHT: key_extra = 86;
pub const KE_C_LEFT: key_extra = 85;
pub const KE_CMDWIN: key_extra = 84;
pub const KE_PLUG: key_extra = 83;
pub const KE_SNR: key_extra = 82;
pub const KE_KDEL: key_extra = 80;
pub const KE_KINS: key_extra = 79;
pub const KE_MOUSERIGHT: key_extra = 78;
pub const KE_MOUSELEFT: key_extra = 77;
pub const KE_MOUSEUP: key_extra = 76;
pub const KE_MOUSEDOWN: key_extra = 75;
pub const KE_S_XF4: key_extra = 74;
pub const KE_S_XF3: key_extra = 73;
pub const KE_S_XF2: key_extra = 72;
pub const KE_S_XF1: key_extra = 71;
pub const KE_LEFTRELEASE_NM: key_extra = 70;
pub const KE_LEFTMOUSE_NM: key_extra = 69;
pub const KE_XRIGHT: key_extra = 68;
pub const KE_XLEFT: key_extra = 67;
pub const KE_XDOWN: key_extra = 66;
pub const KE_XUP: key_extra = 65;
pub const KE_ZHOME: key_extra = 64;
pub const KE_XHOME: key_extra = 63;
pub const KE_ZEND: key_extra = 62;
pub const KE_XEND: key_extra = 61;
pub const KE_XF4: key_extra = 60;
pub const KE_XF3: key_extra = 59;
pub const KE_XF2: key_extra = 58;
pub const KE_XF1: key_extra = 57;
pub const KE_S_TAB_OLD: key_extra = 55;
pub const KE_TAB: key_extra = 54;
pub const KE_IGNORE: key_extra = 53;
pub const KE_RIGHTRELEASE: key_extra = 52;
pub const KE_RIGHTDRAG: key_extra = 51;
pub const KE_RIGHTMOUSE: key_extra = 50;
pub const KE_MIDDLERELEASE: key_extra = 49;
pub const KE_MIDDLEDRAG: key_extra = 48;
pub const KE_MIDDLEMOUSE: key_extra = 47;
pub const KE_LEFTRELEASE: key_extra = 46;
pub const KE_LEFTDRAG: key_extra = 45;
pub const KE_LEFTMOUSE: key_extra = 44;
pub const KE_MOUSE: key_extra = 43;
pub const KE_S_F37: key_extra = 42;
pub const KE_S_F36: key_extra = 41;
pub const KE_S_F35: key_extra = 40;
pub const KE_S_F34: key_extra = 39;
pub const KE_S_F33: key_extra = 38;
pub const KE_S_F32: key_extra = 37;
pub const KE_S_F31: key_extra = 36;
pub const KE_S_F30: key_extra = 35;
pub const KE_S_F29: key_extra = 34;
pub const KE_S_F28: key_extra = 33;
pub const KE_S_F27: key_extra = 32;
pub const KE_S_F26: key_extra = 31;
pub const KE_S_F25: key_extra = 30;
pub const KE_S_F24: key_extra = 29;
pub const KE_S_F23: key_extra = 28;
pub const KE_S_F22: key_extra = 27;
pub const KE_S_F21: key_extra = 26;
pub const KE_S_F20: key_extra = 25;
pub const KE_S_F19: key_extra = 24;
pub const KE_S_F18: key_extra = 23;
pub const KE_S_F17: key_extra = 22;
pub const KE_S_F16: key_extra = 21;
pub const KE_S_F15: key_extra = 20;
pub const KE_S_F14: key_extra = 19;
pub const KE_S_F13: key_extra = 18;
pub const KE_S_F12: key_extra = 17;
pub const KE_S_F11: key_extra = 16;
pub const KE_S_F10: key_extra = 15;
pub const KE_S_F9: key_extra = 14;
pub const KE_S_F8: key_extra = 13;
pub const KE_S_F7: key_extra = 12;
pub const KE_S_F6: key_extra = 11;
pub const KE_S_F5: key_extra = 10;
pub const KE_S_F4: key_extra = 9;
pub const KE_S_F3: key_extra = 8;
pub const KE_S_F2: key_extra = 7;
pub const KE_S_F1: key_extra = 6;
pub const KE_S_DOWN: key_extra = 5;
pub const KE_S_UP: key_extra = 4;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type C2Rust_Unnamed_43 = ::core::ffi::c_int;
pub const MSCR_RIGHT: C2Rust_Unnamed_43 = -2;
pub const MSCR_LEFT: C2Rust_Unnamed_43 = -1;
pub const MSCR_UP: C2Rust_Unnamed_43 = 1;
pub const MSCR_DOWN: C2Rust_Unnamed_43 = 0;
pub type C2Rust_Unnamed_44 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_44 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_44 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_44 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_44 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_44 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_44 = 4;
pub const OPT_LOCAL: C2Rust_Unnamed_44 = 2;
pub const OPT_GLOBAL: C2Rust_Unnamed_44 = 1;
pub const VTERM_PROP_CURSORSHAPE_BAR_LEFT: C2Rust_Unnamed_45 = 3;
pub const VTERM_PROP_CURSORSHAPE_UNDERLINE: C2Rust_Unnamed_45 = 2;
pub const VTERM_PROP_CURSORSHAPE_BLOCK: C2Rust_Unnamed_45 = 1;
pub const VTERM_SELECTION_CUT0: VTermSelectionMask = 16;
pub const VTERM_SELECTION_SELECT: VTermSelectionMask = 8;
pub const VTERM_SELECTION_SECONDARY: VTermSelectionMask = 4;
pub const VTERM_SELECTION_PRIMARY: VTermSelectionMask = 2;
pub const VTERM_SELECTION_CLIPBOARD: VTermSelectionMask = 1;
pub const VTERM_N_VALUETYPES: VTermValueType = 5;
pub const VTERM_VALUETYPE_COLOR: VTermValueType = 4;
pub const VTERM_VALUETYPE_STRING: VTermValueType = 3;
pub const VTERM_VALUETYPE_INT: VTermValueType = 2;
pub const VTERM_VALUETYPE_BOOL: VTermValueType = 1;
pub const VTERM_N_ATTRS: VTermAttr = 16;
pub const VTERM_ATTR_OVERLINE: VTermAttr = 15;
pub const VTERM_ATTR_DIM: VTermAttr = 14;
pub const VTERM_ATTR_URI: VTermAttr = 13;
pub const VTERM_ATTR_BASELINE: VTermAttr = 12;
pub const VTERM_ATTR_SMALL: VTermAttr = 11;
pub const VTERM_ATTR_BACKGROUND: VTermAttr = 10;
pub const VTERM_ATTR_FOREGROUND: VTermAttr = 9;
pub const VTERM_ATTR_FONT: VTermAttr = 8;
pub const VTERM_ATTR_STRIKE: VTermAttr = 7;
pub const VTERM_ATTR_CONCEAL: VTermAttr = 6;
pub const VTERM_ATTR_REVERSE: VTermAttr = 5;
pub const VTERM_ATTR_BLINK: VTermAttr = 4;
pub const VTERM_ATTR_ITALIC: VTermAttr = 3;
pub const VTERM_ATTR_UNDERLINE: VTermAttr = 2;
pub const VTERM_ATTR_BOLD: VTermAttr = 1;
pub const VTERM_COLOR_RGB: C2Rust_Unnamed_46 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TerminalState {
    pub state: VimState,
    pub term: *mut Terminal,
    pub save_rd: ::core::ffi::c_int,
    pub close: bool,
    pub got_bsl: bool,
    pub got_bsl_o: bool,
    pub cursor_visible: bool,
    pub save_curwin_handle: handle_T,
    pub save_w_p_cul: bool,
    pub save_w_p_culopt: *mut ::core::ffi::c_char,
    pub save_w_p_culopt_flags: uint8_t,
    pub save_w_p_cuc: ::core::ffi::c_int,
    pub save_w_p_so: OptInt,
    pub save_w_p_siso: OptInt,
}
pub const VTERM_ALL_MODS_MASK: VTermModifier = 7;
pub const VTERM_MOD_CTRL: VTermModifier = 4;
pub const VTERM_MOD_ALT: VTermModifier = 2;
pub const VTERM_MOD_SHIFT: VTermModifier = 1;
pub const VTERM_MOD_NONE: VTermModifier = 0;
pub const VTERM_N_KEYS: VTermKey = 530;
pub const VTERM_KEY_MAX: VTermKey = 530;
pub const VTERM_KEY_KP_EQUAL: VTermKey = 529;
pub const VTERM_KEY_KP_ENTER: VTermKey = 528;
pub const VTERM_KEY_KP_DIVIDE: VTermKey = 527;
pub const VTERM_KEY_KP_PERIOD: VTermKey = 526;
pub const VTERM_KEY_KP_MINUS: VTermKey = 525;
pub const VTERM_KEY_KP_COMMA: VTermKey = 524;
pub const VTERM_KEY_KP_PLUS: VTermKey = 523;
pub const VTERM_KEY_KP_MULT: VTermKey = 522;
pub const VTERM_KEY_KP_9: VTermKey = 521;
pub const VTERM_KEY_KP_8: VTermKey = 520;
pub const VTERM_KEY_KP_7: VTermKey = 519;
pub const VTERM_KEY_KP_6: VTermKey = 518;
pub const VTERM_KEY_KP_5: VTermKey = 517;
pub const VTERM_KEY_KP_4: VTermKey = 516;
pub const VTERM_KEY_KP_3: VTermKey = 515;
pub const VTERM_KEY_KP_2: VTermKey = 514;
pub const VTERM_KEY_KP_1: VTermKey = 513;
pub const VTERM_KEY_KP_0: VTermKey = 512;
pub const VTERM_KEY_FUNCTION_MAX: VTermKey = 511;
pub const VTERM_KEY_FUNCTION_0: VTermKey = 256;
pub const VTERM_KEY_PAGEDOWN: VTermKey = 14;
pub const VTERM_KEY_PAGEUP: VTermKey = 13;
pub const VTERM_KEY_END: VTermKey = 12;
pub const VTERM_KEY_HOME: VTermKey = 11;
pub const VTERM_KEY_DEL: VTermKey = 10;
pub const VTERM_KEY_INS: VTermKey = 9;
pub const VTERM_KEY_RIGHT: VTermKey = 8;
pub const VTERM_KEY_LEFT: VTermKey = 7;
pub const VTERM_KEY_DOWN: VTermKey = 6;
pub const VTERM_KEY_UP: VTermKey = 5;
pub const VTERM_KEY_ESCAPE: VTermKey = 4;
pub const VTERM_KEY_BACKSPACE: VTermKey = 3;
pub const VTERM_KEY_TAB: VTermKey = 2;
pub const VTERM_KEY_ENTER: VTermKey = 1;
pub const VTERM_KEY_NONE: VTermKey = 0;
pub const VTERM_COLOR_DEFAULT_BG: C2Rust_Unnamed_46 = 4;
pub const VTERM_COLOR_DEFAULT_FG: C2Rust_Unnamed_46 = 2;
pub const VTERM_COLOR_INDEXED: C2Rust_Unnamed_46 = 1;
pub const VTERM_COLOR_TYPE_MASK: C2Rust_Unnamed_46 = 1;
pub const VTERM_UNDERLINE_CURLY: C2Rust_Unnamed_47 = 3;
pub const VTERM_UNDERLINE_DOUBLE: C2Rust_Unnamed_47 = 2;
pub const VTERM_UNDERLINE_SINGLE: C2Rust_Unnamed_47 = 1;
pub const VTERM_UNDERLINE_OFF: C2Rust_Unnamed_47 = 0;
pub type C2Rust_Unnamed_45 = ::core::ffi::c_uint;
pub const VTERM_N_PROP_CURSORSHAPES: C2Rust_Unnamed_45 = 4;
pub type C2Rust_Unnamed_46 = ::core::ffi::c_uint;
pub const VTERM_COLOR_DEFAULT_MASK: C2Rust_Unnamed_46 = 6;
pub type C2Rust_Unnamed_47 = ::core::ffi::c_uint;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
static value_init_ptr_t: GlobalCell<ptr_t> = GlobalCell::new(NULL);
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_ptr_t = Set_ptr_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<ptr_t>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn set_put_ptr_t(
    mut set: *mut Set_ptr_t,
    mut key: ptr_t,
    mut key_alloc: *mut *mut ptr_t,
) -> bool {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_ptr_t(set, key, &raw mut status);
    if !key_alloc.is_null() {
        *key_alloc = (*set).keys.offset(k as isize);
    }
    return status as ::core::ffi::c_uint
        != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
}
#[inline]
unsafe extern "C" fn set_del_ptr_t(mut set: *mut Set_ptr_t, mut key: ptr_t) -> ptr_t {
    mh_delete_ptr_t(set, &raw mut key);
    return key;
}
#[inline]
unsafe extern "C" fn set_has_ptr_t(mut set: *mut Set_ptr_t, mut key: ptr_t) -> bool {
    return mh_get_ptr_t(set, key) != MH_TOMBSTONE as uint32_t;
}
#[inline]
unsafe extern "C" fn map_get_int_ptr_t(
    mut map: *mut Map_int_ptr_t,
    mut key: ::core::ffi::c_int,
) -> ptr_t {
    let mut k: uint32_t = mh_get_int(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_ptr_t.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = 9;
pub const ESC: ::core::ffi::c_int = 27;
pub const Ctrl_AT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const Ctrl_C: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const Ctrl_M: ::core::ffi::c_int = 13;
pub const Ctrl_N: ::core::ffi::c_int = 14;
pub const Ctrl_O: ::core::ffi::c_int = 15;
pub const Ctrl_BSL: ::core::ffi::c_int = 28 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn buf_get_changedtick(buf: *const buf_T) -> varnumber_T {
    return (*buf).changedtick_di.di_tv.vval.v_number;
}
pub const REFRESH_DELAY: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const SELECTIONBUF_SIZE: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
static refresh_timer: GlobalCell<TimeWatcher> = GlobalCell::new(TimeWatcher {
    uv: uv_timer_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: C2Rust_Unnamed_24 { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        timer_cb: None,
        node: C2Rust_Unnamed_23 {
            heap: [::core::ptr::null_mut::<::core::ffi::c_void>(); 3],
        },
        timeout: 0,
        repeat: 0,
        start_id: 0,
    },
    data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    cb: None,
    close_cb: None,
    events: ::core::ptr::null_mut::<MultiQueue>(),
    blockable: false,
});
static refresh_pending: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static vterm_screen_callbacks: GlobalCell<VTermScreenCallbacks> =
    GlobalCell::new(VTermScreenCallbacks {
        damage: Some(
            term_damage
                as unsafe extern "C" fn(VTermRect, *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
        ),
        moverect: Some(
            term_moverect
                as unsafe extern "C" fn(
                    VTermRect,
                    VTermRect,
                    *mut ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        movecursor: Some(
            term_movecursor
                as unsafe extern "C" fn(
                    VTermPos,
                    VTermPos,
                    ::core::ffi::c_int,
                    *mut ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        settermprop: Some(
            term_settermprop
                as unsafe extern "C" fn(
                    VTermProp,
                    *mut VTermValue,
                    *mut ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        bell: Some(
            term_bell as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ::core::ffi::c_int,
        ),
        resize: None,
        theme: Some(
            term_theme
                as unsafe extern "C" fn(*mut bool, *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
        ),
        sb_pushline: Some(
            term_sb_push
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *const VTermScreenCell,
                    *mut ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        sb_popline: Some(
            term_sb_pop
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut VTermScreenCell,
                    *mut ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        sb_clear: Some(
            term_sb_clear as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ::core::ffi::c_int,
        ),
    });
static vterm_selection_callbacks: GlobalCell<VTermSelectionCallbacks> =
    GlobalCell::new(VTermSelectionCallbacks {
        set: Some(
            term_selection_set
                as unsafe extern "C" fn(
                    VTermSelectionMask,
                    VTermStringFragment,
                    *mut ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        query: None,
    });
static invalidated_terminals: GlobalCell<Set_ptr_t> = GlobalCell::new(SET_INIT);
unsafe extern "C" fn emit_termrequest(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut buf_handle: handle_T = (*argv.offset(0 as ::core::ffi::c_int as isize))
        .expose_provenance() as intptr_t as handle_T;
    let mut sequence: *mut ::core::ffi::c_char =
        *argv.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char;
    let mut sequence_length: size_t =
        (*argv.offset(2 as ::core::ffi::c_int as isize)).expose_provenance() as size_t;
    let mut pending_send: *mut StringBuilder =
        *argv.offset(3 as ::core::ffi::c_int as isize) as *mut StringBuilder;
    let mut row: ::core::ffi::c_int = (*argv.offset(4 as ::core::ffi::c_int as isize))
        .expose_provenance() as intptr_t
        as ::core::ffi::c_int;
    let mut col: ::core::ffi::c_int = (*argv.offset(5 as ::core::ffi::c_int as isize))
        .expose_provenance() as intptr_t
        as ::core::ffi::c_int;
    let mut sb_deleted: size_t =
        (*argv.offset(6 as ::core::ffi::c_int as isize)).expose_provenance() as intptr_t as size_t;
    let mut terminator: VTermTerminator = (*argv.offset(7 as ::core::ffi::c_int as isize))
        .expose_provenance() as intptr_t
        as VTermTerminator;
    let mut buf: *mut buf_T =
        map_get_int_ptr_t(buffer_handles.ptr(), buf_handle as ::core::ffi::c_int) as *mut buf_T;
    if buf.is_null() || (*buf).terminal.is_null() {
        xfree(sequence as *mut ::core::ffi::c_void);
        xfree((*pending_send).items as *mut ::core::ffi::c_void);
        (*pending_send).capacity = 0 as size_t;
        (*pending_send).size = (*pending_send).capacity;
        (*pending_send).items = ::core::ptr::null_mut::<::core::ffi::c_char>();
        xfree(pending_send as *mut ::core::ffi::c_void);
        return;
    }
    let mut term: *mut Terminal = (*buf).terminal;
    if (*term).sb_pending > 0 as ::core::ffi::c_int {
        multiqueue_put_event(
            (*term).pending.events,
            Event {
                handler: Some(
                    emit_termrequest as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    *argv.offset(0 as ::core::ffi::c_int as isize),
                    *argv.offset(1 as ::core::ffi::c_int as isize),
                    *argv.offset(2 as ::core::ffi::c_int as isize),
                    *argv.offset(3 as ::core::ffi::c_int as isize),
                    *argv.offset(4 as ::core::ffi::c_int as isize),
                    *argv.offset(5 as ::core::ffi::c_int as isize),
                    *argv.offset(6 as ::core::ffi::c_int as isize),
                    *argv.offset(7 as ::core::ffi::c_int as isize),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ],
            },
        );
        return;
    }
    set_vim_var_string(VV_TERMREQUEST, sequence, sequence_length as ptrdiff_t);
    let mut cursor: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut cursor__items: [Object; 2] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 2];
    cursor.capacity = 2 as size_t;
    cursor.items = &raw mut cursor__items as *mut Object;
    let c2rust_fresh0 = cursor.size;
    cursor.size = cursor.size.wrapping_add(1);
    *cursor.items.offset(c2rust_fresh0 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: row as int64_t - (*term).sb_deleted.wrapping_sub(sb_deleted) as int64_t,
        },
    };
    let c2rust_fresh1 = cursor.size;
    cursor.size = cursor.size.wrapping_add(1);
    *cursor.items.offset(c2rust_fresh1 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: col as Integer,
        },
    };
    let mut data: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut data__items: [KeyValuePair; 3] = [KeyValuePair {
        key: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        value: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
    }; 3];
    data.capacity = 3 as size_t;
    data.items = &raw mut data__items as *mut KeyValuePair;
    let mut termrequest: String_0 = String_0 {
        data: sequence,
        size: sequence_length,
    };
    let c2rust_fresh2 = data.size;
    data.size = data.size.wrapping_add(1);
    *data.items.offset(c2rust_fresh2 as isize) = key_value_pair {
        key: cstr_as_string(b"sequence\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: termrequest,
            },
        },
    };
    let c2rust_fresh3 = data.size;
    data.size = data.size.wrapping_add(1);
    *data.items.offset(c2rust_fresh3 as isize) = key_value_pair {
        key: cstr_as_string(b"cursor\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed { array: cursor },
        },
    };
    let c2rust_fresh4 = data.size;
    data.size = data.size.wrapping_add(1);
    *data.items.offset(c2rust_fresh4 as isize) = key_value_pair {
        key: cstr_as_string(b"terminator\0".as_ptr() as *const ::core::ffi::c_char),
        value: if terminator as ::core::ffi::c_uint
            == VTERM_TERMINATOR_BEL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: String_0 {
                        data: b"\x07\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                            .wrapping_sub(1 as size_t),
                    },
                },
            }
        } else {
            object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: String_0 {
                        data: b"\x1B\\\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                            .wrapping_sub(1 as size_t),
                    },
                },
            }
        },
    };
    (*term).refcount = (*term).refcount.wrapping_add(1);
    let mut c2rust_lvalue: Object = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed { dict: data },
    };
    apply_autocmds_group(
        EVENT_TERMREQUEST,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        true_0 != 0,
        AUGROUP_ALL as ::core::ffi::c_int,
        buf,
        ::core::ptr::null_mut::<exarg_T>(),
        &raw mut c2rust_lvalue,
    );
    (*term).refcount = (*term).refcount.wrapping_sub(1);
    xfree(sequence as *mut ::core::ffi::c_void);
    let mut term_pending_send: *mut StringBuilder = (*term).pending.send;
    (*term).pending.send = ::core::ptr::null_mut::<StringBuilder>();
    if (*pending_send).size != 0 {
        terminal_send(term, (*pending_send).items, (*pending_send).size);
        xfree((*pending_send).items as *mut ::core::ffi::c_void);
        (*pending_send).capacity = 0 as size_t;
        (*pending_send).size = (*pending_send).capacity;
        (*pending_send).items = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if term_pending_send != pending_send {
        (*term).pending.send = term_pending_send;
    }
    xfree(pending_send as *mut ::core::ffi::c_void);
    if (*term).buf_handle == 0 as ::core::ffi::c_int && (*term).refcount == 0 {
        (*term).destroy = true_0 != 0;
        (*term).opts.close_cb.expect("non-null function pointer")((*term).opts.data);
    }
}
unsafe extern "C" fn schedule_termrequest(mut term: *mut Terminal) {
    (*term).pending.send = xmalloc(::core::mem::size_of::<StringBuilder>()) as *mut StringBuilder;
    (*(*term).pending.send).capacity = 0 as size_t;
    (*(*term).pending.send).size = (*(*term).pending.send).capacity;
    (*(*term).pending.send).items = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut line: ::core::ffi::c_int = row_to_linenr(term, (*term).cursor.row);
    multiqueue_put_event(
        (*main_loop.ptr()).events,
        Event {
            handler: Some(
                emit_termrequest as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
            ),
            argv: [
                ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                    (*term).buf_handle as intptr_t as usize,
                ),
                xmemdup(
                    (*term).termrequest_buffer.items as *const ::core::ffi::c_void,
                    (*term).termrequest_buffer.size,
                ),
                ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                    (*term).termrequest_buffer.size as intptr_t as usize,
                ),
                (*term).pending.send as *mut ::core::ffi::c_void,
                ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                    line as intptr_t as usize,
                ),
                ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                    (*term).cursor.col as intptr_t as usize,
                ),
                ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                    (*term).sb_deleted as intptr_t as usize,
                ),
                ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                    (*term).termrequest_terminator as intptr_t as usize,
                ),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
            ],
        },
    );
}
unsafe extern "C" fn parse_osc8(
    mut str: *const ::core::ffi::c_char,
    mut attr: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut i: size_t = 0 as size_t;
    while *str.offset(i as isize) as ::core::ffi::c_int != NUL {
        if *str.offset(i as isize) as ::core::ffi::c_int == ';' as ::core::ffi::c_int {
            break;
        }
        i = i.wrapping_add(1);
    }
    if *str.offset(i as isize) as ::core::ffi::c_int != ';' as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    i = i.wrapping_add(1);
    if *str.offset(i as isize) as ::core::ffi::c_int == NUL {
        *attr = 0 as ::core::ffi::c_int;
        return 1 as ::core::ffi::c_int;
    }
    *attr = hl_add_url(0 as ::core::ffi::c_int, str.offset(i as isize));
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn on_osc(
    mut command: ::core::ffi::c_int,
    mut frag: VTermStringFragment,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut term: *mut Terminal = user as *mut Terminal;
    if frag.str.is_null() || frag.len() as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if command != 8 as ::core::ffi::c_int && !has_event(EVENT_TERMREQUEST) {
        return 1 as ::core::ffi::c_int;
    }
    if frag.initial() {
        (*term).termrequest_buffer.size = 0 as size_t;
        kv_do_printf(
            &raw mut (*term).termrequest_buffer,
            b"\x1B]%d;\0".as_ptr() as *const ::core::ffi::c_char,
            command,
        );
    }
    if frag.len() as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
        if (*term).termrequest_buffer.capacity
            < (*term).termrequest_buffer.size.wrapping_add(frag.len())
        {
            (*term).termrequest_buffer.capacity =
                (*term).termrequest_buffer.size.wrapping_add(frag.len());
            (*term).termrequest_buffer.capacity =
                (*term).termrequest_buffer.capacity.wrapping_sub(1);
            (*term).termrequest_buffer.capacity |=
                (*term).termrequest_buffer.capacity >> 1 as ::core::ffi::c_int;
            (*term).termrequest_buffer.capacity |=
                (*term).termrequest_buffer.capacity >> 2 as ::core::ffi::c_int;
            (*term).termrequest_buffer.capacity |=
                (*term).termrequest_buffer.capacity >> 4 as ::core::ffi::c_int;
            (*term).termrequest_buffer.capacity |=
                (*term).termrequest_buffer.capacity >> 8 as ::core::ffi::c_int;
            (*term).termrequest_buffer.capacity |=
                (*term).termrequest_buffer.capacity >> 16 as ::core::ffi::c_int;
            (*term).termrequest_buffer.capacity =
                (*term).termrequest_buffer.capacity.wrapping_add(1);
            (*term).termrequest_buffer.capacity = (*term).termrequest_buffer.capacity;
            (*term).termrequest_buffer.items = xrealloc(
                (*term).termrequest_buffer.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul((*term).termrequest_buffer.capacity),
            ) as *mut ::core::ffi::c_char;
        }
        '_c2rust_label: {
            if !(*term).termrequest_buffer.items.is_null() {
            } else {
                __assert_fail(
                    b"(term->termrequest_buffer).items\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/terminal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    366 as ::core::ffi::c_uint,
                    b"int on_osc(int, VTermStringFragment, void *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        memcpy(
            (*term)
                .termrequest_buffer
                .items
                .offset((*term).termrequest_buffer.size as isize)
                as *mut ::core::ffi::c_void,
            frag.str as *const ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(frag.len()),
        );
        (*term).termrequest_buffer.size = (*term).termrequest_buffer.size.wrapping_add(frag.len());
    }
    if frag.final_0() {
        (*term).termrequest_terminator = frag.terminator;
        if has_event(EVENT_TERMREQUEST) {
            schedule_termrequest(term);
        }
        if command == 8 as ::core::ffi::c_int {
            if (*term).termrequest_buffer.size == (*term).termrequest_buffer.capacity {
                (*term).termrequest_buffer.capacity = if (*term).termrequest_buffer.capacity != 0 {
                    (*term).termrequest_buffer.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*term).termrequest_buffer.items = xrealloc(
                    (*term).termrequest_buffer.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul((*term).termrequest_buffer.capacity),
                ) as *mut ::core::ffi::c_char;
            } else {
            };
            let c2rust_fresh5 = (*term).termrequest_buffer.size;
            (*term).termrequest_buffer.size = (*term).termrequest_buffer.size.wrapping_add(1);
            *(*term)
                .termrequest_buffer
                .items
                .offset(c2rust_fresh5 as isize) = '\0' as ::core::ffi::c_char;
            let off: size_t =
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t);
            let mut attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if parse_osc8(
                (*term).termrequest_buffer.items.offset(off as isize),
                &raw mut attr,
            ) != 0
            {
                let mut state: *mut VTermState = vterm_obtain_state((*term).vt);
                let mut value: VTermValue = VTermValue { number: attr };
                vterm_state_set_penattr(state, VTERM_ATTR_URI, VTERM_VALUETYPE_INT, &raw mut value);
            }
        }
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn on_dcs(
    mut command: *const ::core::ffi::c_char,
    mut commandlen: size_t,
    mut frag: VTermStringFragment,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut term: *mut Terminal = user as *mut Terminal;
    if command.is_null() || frag.str.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if !has_event(EVENT_TERMREQUEST) {
        return 1 as ::core::ffi::c_int;
    }
    if frag.initial() {
        (*term).termrequest_buffer.size = 0 as size_t;
        kv_do_printf(
            &raw mut (*term).termrequest_buffer,
            b"\x1BP%.*s\0".as_ptr() as *const ::core::ffi::c_char,
            commandlen as ::core::ffi::c_int,
            command,
        );
    }
    if frag.len() as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
        if (*term).termrequest_buffer.capacity
            < (*term).termrequest_buffer.size.wrapping_add(frag.len())
        {
            (*term).termrequest_buffer.capacity =
                (*term).termrequest_buffer.size.wrapping_add(frag.len());
            (*term).termrequest_buffer.capacity =
                (*term).termrequest_buffer.capacity.wrapping_sub(1);
            (*term).termrequest_buffer.capacity |=
                (*term).termrequest_buffer.capacity >> 1 as ::core::ffi::c_int;
            (*term).termrequest_buffer.capacity |=
                (*term).termrequest_buffer.capacity >> 2 as ::core::ffi::c_int;
            (*term).termrequest_buffer.capacity |=
                (*term).termrequest_buffer.capacity >> 4 as ::core::ffi::c_int;
            (*term).termrequest_buffer.capacity |=
                (*term).termrequest_buffer.capacity >> 8 as ::core::ffi::c_int;
            (*term).termrequest_buffer.capacity |=
                (*term).termrequest_buffer.capacity >> 16 as ::core::ffi::c_int;
            (*term).termrequest_buffer.capacity =
                (*term).termrequest_buffer.capacity.wrapping_add(1);
            (*term).termrequest_buffer.capacity = (*term).termrequest_buffer.capacity;
            (*term).termrequest_buffer.items = xrealloc(
                (*term).termrequest_buffer.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul((*term).termrequest_buffer.capacity),
            ) as *mut ::core::ffi::c_char;
        }
        '_c2rust_label: {
            if !(*term).termrequest_buffer.items.is_null() {
            } else {
                __assert_fail(
                    b"(term->termrequest_buffer).items\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/terminal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    401 as ::core::ffi::c_uint,
                    b"int on_dcs(const char *, size_t, VTermStringFragment, void *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        memcpy(
            (*term)
                .termrequest_buffer
                .items
                .offset((*term).termrequest_buffer.size as isize)
                as *mut ::core::ffi::c_void,
            frag.str as *const ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(frag.len()),
        );
        (*term).termrequest_buffer.size = (*term).termrequest_buffer.size.wrapping_add(frag.len());
    }
    if frag.final_0() {
        (*term).termrequest_terminator = frag.terminator;
        schedule_termrequest(term);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn on_apc(
    mut frag: VTermStringFragment,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut term: *mut Terminal = user as *mut Terminal;
    if frag.str.is_null() || frag.len() as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if !has_event(EVENT_TERMREQUEST) {
        return 1 as ::core::ffi::c_int;
    }
    if frag.initial() {
        (*term).termrequest_buffer.size = 0 as size_t;
        kv_do_printf(
            &raw mut (*term).termrequest_buffer,
            b"\x1B_\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    if frag.len() as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
        if (*term).termrequest_buffer.capacity
            < (*term).termrequest_buffer.size.wrapping_add(frag.len())
        {
            (*term).termrequest_buffer.capacity =
                (*term).termrequest_buffer.size.wrapping_add(frag.len());
            (*term).termrequest_buffer.capacity =
                (*term).termrequest_buffer.capacity.wrapping_sub(1);
            (*term).termrequest_buffer.capacity |=
                (*term).termrequest_buffer.capacity >> 1 as ::core::ffi::c_int;
            (*term).termrequest_buffer.capacity |=
                (*term).termrequest_buffer.capacity >> 2 as ::core::ffi::c_int;
            (*term).termrequest_buffer.capacity |=
                (*term).termrequest_buffer.capacity >> 4 as ::core::ffi::c_int;
            (*term).termrequest_buffer.capacity |=
                (*term).termrequest_buffer.capacity >> 8 as ::core::ffi::c_int;
            (*term).termrequest_buffer.capacity |=
                (*term).termrequest_buffer.capacity >> 16 as ::core::ffi::c_int;
            (*term).termrequest_buffer.capacity =
                (*term).termrequest_buffer.capacity.wrapping_add(1);
            (*term).termrequest_buffer.capacity = (*term).termrequest_buffer.capacity;
            (*term).termrequest_buffer.items = xrealloc(
                (*term).termrequest_buffer.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul((*term).termrequest_buffer.capacity),
            ) as *mut ::core::ffi::c_char;
        }
        '_c2rust_label: {
            if !(*term).termrequest_buffer.items.is_null() {
            } else {
                __assert_fail(
                    b"(term->termrequest_buffer).items\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/terminal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    424 as ::core::ffi::c_uint,
                    b"int on_apc(VTermStringFragment, void *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        memcpy(
            (*term)
                .termrequest_buffer
                .items
                .offset((*term).termrequest_buffer.size as isize)
                as *mut ::core::ffi::c_void,
            frag.str as *const ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(frag.len()),
        );
        (*term).termrequest_buffer.size = (*term).termrequest_buffer.size.wrapping_add(frag.len());
    }
    if frag.final_0() {
        (*term).termrequest_terminator = frag.terminator;
        schedule_termrequest(term);
    }
    return 1 as ::core::ffi::c_int;
}
static vterm_fallbacks: GlobalCell<VTermStateFallbacks> = GlobalCell::new(VTermStateFallbacks {
    control: None,
    csi: None,
    osc: Some(
        on_osc
            as unsafe extern "C" fn(
                ::core::ffi::c_int,
                VTermStringFragment,
                *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
    ),
    dcs: Some(
        on_dcs
            as unsafe extern "C" fn(
                *const ::core::ffi::c_char,
                size_t,
                VTermStringFragment,
                *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
    ),
    apc: Some(
        on_apc
            as unsafe extern "C" fn(
                VTermStringFragment,
                *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
    ),
    pm: None,
    sos: None,
});
pub unsafe extern "C" fn terminal_init() {
    time_watcher_init(main_loop.ptr(), refresh_timer.ptr(), NULL_0);
    (*refresh_timer.ptr()).events = multiqueue_new_child((*main_loop.ptr()).events);
}
pub unsafe extern "C" fn terminal_teardown() {
    time_watcher_stop(refresh_timer.ptr());
    multiqueue_free((*refresh_timer.ptr()).events);
    time_watcher_close(refresh_timer.ptr(), None);
    xfree((*invalidated_terminals.ptr()).keys as *mut ::core::ffi::c_void);
    xfree((*invalidated_terminals.ptr()).h.hash as *mut ::core::ffi::c_void);
    invalidated_terminals.set(SET_INIT);
    invalidated_terminals.set(SET_INIT);
}
unsafe extern "C" fn term_output_callback(
    mut s: *const ::core::ffi::c_char,
    mut len: size_t,
    mut user_data: *mut ::core::ffi::c_void,
) {
    terminal_send(user_data as *mut Terminal, s, len);
}
unsafe extern "C" fn term_may_alloc_scrollback(
    mut term: *mut Terminal,
    mut buf: *mut buf_T,
) -> bool {
    if !(*term).sb_buffer.is_null() {
        return true_0 != 0;
    }
    if buf.is_null() {
        buf = map_get_int_ptr_t(
            buffer_handles.ptr(),
            (*term).buf_handle as ::core::ffi::c_int,
        ) as *mut buf_T;
        if buf.is_null() {
            return false_0 != 0;
        }
    }
    if (*buf).b_p_scbk < 1 as OptInt {
        (*buf).b_p_scbk = SB_MAX as OptInt;
    }
    (*term).sb_size = (*buf).b_p_scbk as size_t;
    (*term).sb_buffer =
        xmalloc(::core::mem::size_of::<*mut ScrollbackLine>().wrapping_mul((*term).sb_size))
            as *mut *mut ScrollbackLine;
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn terminal_alloc(
    mut buf: *mut buf_T,
    mut opts: TerminalOptions,
) -> *mut Terminal {
    let mut term: *mut Terminal =
        xcalloc(1 as size_t, ::core::mem::size_of::<Terminal>()) as *mut Terminal;
    (*term).opts = opts;
    (*term).buf_handle = (*buf).handle;
    (*buf).terminal = term;
    (*term).vt = vterm_new(
        opts.height as ::core::ffi::c_int,
        opts.width as ::core::ffi::c_int,
    );
    vterm_set_utf8((*term).vt, 1 as ::core::ffi::c_int);
    let mut state: *mut VTermState = vterm_obtain_state((*term).vt);
    (*term).vts = vterm_obtain_screen((*term).vt);
    vterm_screen_enable_altscreen((*term).vts, true_0);
    vterm_screen_enable_reflow((*term).vts, true_0 != 0);
    vterm_screen_set_callbacks(
        (*term).vts,
        vterm_screen_callbacks.ptr(),
        term as *mut ::core::ffi::c_void,
    );
    vterm_screen_set_unrecognised_fallbacks(
        (*term).vts,
        vterm_fallbacks.ptr(),
        term as *mut ::core::ffi::c_void,
    );
    vterm_screen_set_damage_merge((*term).vts, VTERM_DAMAGE_SCROLL);
    vterm_screen_reset((*term).vts, 1 as ::core::ffi::c_int);
    vterm_output_set_callback(
        (*term).vt,
        Some(
            term_output_callback
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_char,
                    size_t,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        term as *mut ::core::ffi::c_void,
    );
    (*term).selection_buffer =
        xcalloc(SELECTIONBUF_SIZE as size_t, 1 as size_t) as *mut ::core::ffi::c_char;
    vterm_state_set_selection_callbacks(
        state,
        vterm_selection_callbacks.ptr(),
        term as *mut ::core::ffi::c_void,
        (*term).selection_buffer,
        SELECTIONBUF_SIZE as size_t,
    );
    let mut cursor_shape: VTermValue = VTermValue { boolean: 0 };
    match (*shape_table.ptr())[SHAPE_IDX_TERM as ::core::ffi::c_int as usize].shape
        as ::core::ffi::c_uint
    {
        0 => {
            cursor_shape.number = VTERM_PROP_CURSORSHAPE_BLOCK as ::core::ffi::c_int;
        }
        1 => {
            cursor_shape.number = VTERM_PROP_CURSORSHAPE_UNDERLINE as ::core::ffi::c_int;
        }
        2 => {
            cursor_shape.number = VTERM_PROP_CURSORSHAPE_BAR_LEFT as ::core::ffi::c_int;
        }
        _ => {}
    }
    vterm_state_set_termprop(state, VTERM_PROP_CURSORSHAPE, &raw mut cursor_shape);
    let mut cursor_blink: VTermValue = VTermValue { boolean: 0 };
    if (*shape_table.ptr())[SHAPE_IDX_TERM as ::core::ffi::c_int as usize].blinkon
        != 0 as ::core::ffi::c_int
        && (*shape_table.ptr())[SHAPE_IDX_TERM as ::core::ffi::c_int as usize].blinkoff
            != 0 as ::core::ffi::c_int
    {
        cursor_blink.boolean = true_0;
    } else {
        cursor_blink.boolean = false_0;
    }
    vterm_state_set_termprop(state, VTERM_PROP_CURSORBLINK, &raw mut cursor_blink);
    (*term).invalid_start = 0 as ::core::ffi::c_int;
    (*term).invalid_end = opts.height as ::core::ffi::c_int;
    (*term).pending.events = multiqueue_new(None, NULL_0);
    if (*buf).b_ml.ml_flags & ML_EMPTY == 0 {
        let mut line_count: linenr_T = (*buf).b_ml.ml_line_count;
        while (*buf).b_ml.ml_flags & ML_EMPTY == 0 {
            ml_delete_buf(buf, 1 as linenr_T, false_0 != 0);
        }
        deleted_lines_buf(buf, 1 as linenr_T, line_count);
    }
    (*term).old_height = 1 as ::core::ffi::c_int;
    return term;
}
#[no_mangle]
pub unsafe extern "C" fn terminal_open(mut termpp: *mut *mut Terminal, mut buf: *mut buf_T) {
    let mut term: *mut Terminal = *termpp;
    '_c2rust_label: {
        if !term.is_null() {
        } else {
            __assert_fail(
                b"term != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/terminal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                588 as ::core::ffi::c_uint,
                b"void terminal_open(Terminal **, buf_T *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut aco: aco_save_T = aco_save_T {
        use_aucmd_win_idx: 0,
        save_curwin_handle: 0,
        new_curwin_handle: 0,
        save_prevwin_handle: 0,
        new_curbuf: bufref_T {
            br_buf: ::core::ptr::null_mut::<buf_T>(),
            br_fnum: 0,
            br_buf_free_count: 0,
        },
        tp_localdir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        globaldir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        save_VIsual_active: false,
        save_prompt_insert: 0,
    };
    aucmd_prepbuf(&raw mut aco, buf);
    if !(*term).sb_buffer.is_null() {
        refresh_scrollback(term, buf);
    } else {
        '_c2rust_label_0: {
            if (*term).invalid_start >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"term->invalid_start >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/terminal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    598 as ::core::ffi::c_uint,
                    b"void terminal_open(Terminal **, buf_T *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
    }
    refresh_screen(term, buf);
    (*buf).b_locked += 1;
    set_option_value(
        kOptBuftype,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: String_0 {
                    data: b"terminal\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 9]>()
                        .wrapping_sub(1 as size_t),
                },
            },
        },
        OPT_LOCAL as ::core::ffi::c_int,
    );
    (*buf).b_locked -= 1;
    if !(*buf).b_ffname.is_null() {
        buf_set_term_title(buf, (*buf).b_ffname, strlen((*buf).b_ffname));
    }
    (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
    (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
    (*curwin.get()).w_cursor = pos_T {
        lnum: 1 as linenr_T,
        col: 0 as colnr_T,
        coladd: 0 as colnr_T,
    };
    apply_autocmds(
        EVENT_TERMOPEN,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        buf,
    );
    aucmd_restbuf(&raw mut aco);
    if (*termpp).is_null() || (*term).buf_handle == 0 as ::core::ffi::c_int {
        return;
    }
    if !term_may_alloc_scrollback(term, buf) {
        abort();
    }
    let mut state: *mut VTermState = vterm_obtain_state((*term).vt);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 16 as ::core::ffi::c_int {
        let mut var: [::core::ffi::c_char; 64] = [0; 64];
        snprintf(
            &raw mut var as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>(),
            b"terminal_color_%d\0".as_ptr() as *const ::core::ffi::c_char,
            i,
        );
        let mut name: *mut ::core::ffi::c_char =
            get_config_string(buf, &raw mut var as *mut ::core::ffi::c_char);
        if !name.is_null() {
            let mut dummy: ::core::ffi::c_int = 0;
            let mut color_val: RgbValue = name_to_color(name, &raw mut dummy);
            if color_val != -1 as RgbValue {
                let mut color: VTermColor = VTermColor { type_0: 0 };
                vterm_color_rgb(
                    &raw mut color,
                    (color_val >> 16 as ::core::ffi::c_int & 0xff as RgbValue) as uint8_t,
                    (color_val >> 8 as ::core::ffi::c_int & 0xff as RgbValue) as uint8_t,
                    (color_val >> 0 as ::core::ffi::c_int & 0xff as RgbValue) as uint8_t,
                );
                vterm_state_set_palette_color(state, i, &raw mut color);
                (*term).color_set[i as usize] = true_0 != 0;
            }
        }
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn terminal_close(
    mut termpp: *mut *mut Terminal,
    mut status: ::core::ffi::c_int,
) {
    let mut term: *mut Terminal = *termpp;
    if (*term).destroy {
        return;
    }
    let mut only_destroy: bool = false_0 != 0;
    let mut buf: *mut buf_T = map_get_int_ptr_t(
        buffer_handles.ptr(),
        (*term).buf_handle as ::core::ffi::c_int,
    ) as *mut buf_T;
    if (*term).closed {
        only_destroy = true_0 != 0;
    } else {
        if !exiting.get() {
            block_autocmds();
            refresh_terminal(term);
            unblock_autocmds();
        }
        (*term).closed = true_0 != 0;
    }
    let mut pos: ::core::ffi::c_int = if !buf.is_null() {
        (*buf).b_ml.ml_line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    if status == -1 as ::core::ffi::c_int || exiting.get() as ::core::ffi::c_int != 0 {
        (*term).buf_handle = 0 as ::core::ffi::c_int as handle_T;
        if !buf.is_null() {
            (*buf).terminal = ::core::ptr::null_mut::<Terminal>();
        }
        if (*term).refcount == 0 {
            (*term).destroy = true_0 != 0;
            (*term).opts.close_cb.expect("non-null function pointer")((*term).opts.data);
        }
    } else if !only_destroy {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == buf {
                (*wp).w_redr_status = true_0 != 0;
            }
            wp = (*wp).w_next;
        }
        pos = if row_to_linenr(term, (*term).cursor.row) < pos {
            row_to_linenr(term, (*term).cursor.row)
        } else {
            pos
        };
    }
    if only_destroy {
        return;
    }
    if !buf.is_null() && !is_autocmd_blocked() {
        let mut save_v_event: save_v_event_T = save_v_event_T {
            sve_did_save: false,
            sve_hashtab: hashtab_T {
                ht_mask: 0,
                ht_used: 0,
                ht_filled: 0,
                ht_changed: 0,
                ht_locked: 0,
                ht_array: ::core::ptr::null_mut::<hashitem_T>(),
                ht_smallarray: [hashitem_T {
                    hi_hash: 0,
                    hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                }; 16],
            },
        };
        let mut dict: *mut dict_T = get_v_event(&raw mut save_v_event);
        tv_dict_add_nr(
            dict,
            b"status\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
            status as varnumber_T,
        );
        tv_dict_set_keys_readonly(dict);
        let mut data: Dict = Dict {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        };
        let mut data__items: [KeyValuePair; 1] = [KeyValuePair {
            key: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            value: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
        }; 1];
        data.capacity = 1 as size_t;
        data.items = &raw mut data__items as *mut KeyValuePair;
        let c2rust_fresh7 = data.size;
        data.size = data.size.wrapping_add(1);
        *data.items.offset(c2rust_fresh7 as isize) = key_value_pair {
            key: cstr_as_string(b"pos\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: pos as Integer,
                },
            },
        };
        let mut c2rust_lvalue: Object = object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed { dict: data },
        };
        apply_autocmds_group(
            EVENT_TERMCLOSE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            status >= 0 as ::core::ffi::c_int,
            AUGROUP_ALL as ::core::ffi::c_int,
            buf,
            ::core::ptr::null_mut::<exarg_T>(),
            &raw mut c2rust_lvalue,
        );
        restore_v_event(dict, &raw mut save_v_event);
    }
}
unsafe extern "C" fn terminal_state_change_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut buf_handle: handle_T = (*argv.offset(0 as ::core::ffi::c_int as isize))
        .expose_provenance() as intptr_t as handle_T;
    let mut buf: *mut buf_T =
        map_get_int_ptr_t(buffer_handles.ptr(), buf_handle as ::core::ffi::c_int) as *mut buf_T;
    if !buf.is_null() && !(*buf).terminal.is_null() {
        redraw_buf_line_later(buf, (*buf).b_ml.ml_line_count, false_0 != 0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn terminal_set_state(mut term: *mut Terminal, mut suspended: bool) {
    if (*term).suspended as ::core::ffi::c_int != suspended as ::core::ffi::c_int {
        multiqueue_put_event(
            (*refresh_timer.ptr()).events,
            Event {
                handler: Some(
                    terminal_state_change_event
                        as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                        (*term).buf_handle as intptr_t as usize,
                    ),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ],
            },
        );
    }
    (*term).suspended = suspended;
}
#[no_mangle]
pub unsafe extern "C" fn terminal_check_size(mut term: *mut Terminal) {
    if (*term).closed {
        return;
    }
    let mut curwidth: ::core::ffi::c_int = 0;
    let mut curheight: ::core::ffi::c_int = 0;
    vterm_get_size((*term).vt, &raw mut curheight, &raw mut curwidth);
    let mut width: uint16_t = 0 as uint16_t;
    let mut height: uint16_t = 0 as uint16_t;
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if !is_aucmd_win(wp) {
                if !(*wp).w_buffer.is_null() && (*(*wp).w_buffer).terminal == term {
                    let win_width: uint16_t =
                        (if 0 as ::core::ffi::c_int > (*wp).w_view_width - win_col_off(wp) {
                            0 as ::core::ffi::c_int
                        } else {
                            (*wp).w_view_width - win_col_off(wp)
                        }) as uint16_t;
                    width = (if width as ::core::ffi::c_int > win_width as ::core::ffi::c_int {
                        width as ::core::ffi::c_int
                    } else {
                        win_width as ::core::ffi::c_int
                    }) as uint16_t;
                    height = (if height as ::core::ffi::c_int > (*wp).w_view_height {
                        height as ::core::ffi::c_int
                    } else {
                        (*wp).w_view_height
                    }) as uint16_t;
                }
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    if curheight == height as ::core::ffi::c_int && curwidth == width as ::core::ffi::c_int
        || height as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        || width as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        return;
    }
    vterm_set_size(
        (*term).vt,
        height as ::core::ffi::c_int,
        width as ::core::ffi::c_int,
    );
    vterm_screen_flush_damage((*term).vts);
    (*term).pending.resize = true_0 != 0;
    invalidate_terminal(term, -1 as ::core::ffi::c_int, -1 as ::core::ffi::c_int);
}
unsafe extern "C" fn set_terminal_winopts(s: *mut TerminalState) {
    '_c2rust_label: {
        if (*s).save_curwin_handle == 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"s->save_curwin_handle == 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/terminal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                812 as ::core::ffi::c_uint,
                b"void set_terminal_winopts(TerminalState *const)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    (*s).save_curwin_handle = (*curwin.get()).handle;
    (*s).save_w_p_cul = (*curwin.get()).w_onebuf_opt.wo_cul != 0;
    (*s).save_w_p_culopt = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*s).save_w_p_culopt_flags = (*curwin.get()).w_p_culopt_flags;
    (*s).save_w_p_cuc = (*curwin.get()).w_onebuf_opt.wo_cuc;
    (*s).save_w_p_so = (*curwin.get()).w_onebuf_opt.wo_so;
    (*s).save_w_p_siso = (*curwin.get()).w_onebuf_opt.wo_siso;
    if (*curwin.get()).w_onebuf_opt.wo_cul != 0
        && (*curwin.get()).w_p_culopt_flags as ::core::ffi::c_int
            & kOptCuloptFlagNumber as ::core::ffi::c_int
            != 0
    {
        if !strequal(
            (*curwin.get()).w_onebuf_opt.wo_culopt,
            b"number\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            (*s).save_w_p_culopt = (*curwin.get()).w_onebuf_opt.wo_culopt;
            (*curwin.get()).w_onebuf_opt.wo_culopt =
                xstrdup(b"number\0".as_ptr() as *const ::core::ffi::c_char);
        }
        (*curwin.get()).w_p_culopt_flags = kOptCuloptFlagNumber as ::core::ffi::c_int as uint8_t;
    } else {
        (*curwin.get()).w_onebuf_opt.wo_cul = false_0;
    }
    (*curwin.get()).w_onebuf_opt.wo_cuc = false_0;
    (*curwin.get()).w_onebuf_opt.wo_so = 0 as OptInt;
    (*curwin.get()).w_onebuf_opt.wo_siso = 0 as OptInt;
    if (*curwin.get()).w_onebuf_opt.wo_cuc != (*s).save_w_p_cuc {
        redraw_later(curwin.get(), UPD_SOME_VALID as ::core::ffi::c_int);
    } else if (*curwin.get()).w_onebuf_opt.wo_cul != (*s).save_w_p_cul as ::core::ffi::c_int
        || (*curwin.get()).w_onebuf_opt.wo_cul != 0
            && (*curwin.get()).w_p_culopt_flags as ::core::ffi::c_int
                != (*s).save_w_p_culopt_flags as ::core::ffi::c_int
    {
        redraw_later(curwin.get(), UPD_VALID as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn unset_terminal_winopts(s: *mut TerminalState) {
    let mut winopts: *mut winopt_T = ::core::ptr::null_mut::<winopt_T>();
    '_c2rust_label: {
        if (*s).save_curwin_handle != 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"s->save_curwin_handle != 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/terminal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                848 as ::core::ffi::c_uint,
                b"void unset_terminal_winopts(TerminalState *const)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let wp: *mut win_T = map_get_int_ptr_t(
        window_handles.ptr(),
        (*s).save_curwin_handle as ::core::ffi::c_int,
    ) as *mut win_T;
    '_end: {
        if !wp.is_null() {
            winopts = ::core::ptr::null_mut::<winopt_T>();
            if (*(*wp).w_buffer).handle != (*(*s).term).buf_handle {
                let mut buf: *mut buf_T = map_get_int_ptr_t(
                    buffer_handles.ptr(),
                    (*(*s).term).buf_handle as ::core::ffi::c_int,
                ) as *mut buf_T;
                if buf.is_null() {
                    break '_end;
                } else {
                    let mut i: size_t = 0 as size_t;
                    while i < (*buf).b_wininfo.size {
                        let mut wip: *mut WinInfo = *(*buf).b_wininfo.items.offset(i as isize);
                        if (*wip).wi_win == wp && (*wip).wi_optset as ::core::ffi::c_int != 0 {
                            winopts = &raw mut (*wip).wi_opt;
                            break;
                        } else {
                            i = i.wrapping_add(1);
                        }
                    }
                    if winopts.is_null() {
                        break '_end;
                    }
                }
            } else {
                winopts = &raw mut (*wp).w_onebuf_opt;
                if win_valid(wp) {
                    if (*s).save_w_p_cuc != (*wp).w_onebuf_opt.wo_cuc {
                        redraw_later(wp, UPD_SOME_VALID as ::core::ffi::c_int);
                    } else if (*s).save_w_p_cul as ::core::ffi::c_int != (*wp).w_onebuf_opt.wo_cul
                        || (*s).save_w_p_cul as ::core::ffi::c_int != 0
                            && (*s).save_w_p_culopt_flags as ::core::ffi::c_int
                                != (*wp).w_p_culopt_flags as ::core::ffi::c_int
                    {
                        redraw_later(wp, UPD_VALID as ::core::ffi::c_int);
                    }
                }
                (*wp).w_p_culopt_flags = (*s).save_w_p_culopt_flags;
            }
            if !(*s).save_w_p_culopt.is_null() {
                free_string_option((*winopts).wo_culopt);
                (*winopts).wo_culopt = (*s).save_w_p_culopt;
                (*s).save_w_p_culopt = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            (*winopts).wo_cul = (*s).save_w_p_cul as ::core::ffi::c_int;
            (*winopts).wo_cuc = (*s).save_w_p_cuc;
            (*winopts).wo_so = (*s).save_w_p_so;
            (*winopts).wo_siso = (*s).save_w_p_siso;
        }
    }
    free_string_option((*s).save_w_p_culopt);
    (*s).save_curwin_handle = 0 as ::core::ffi::c_int as handle_T;
}
pub unsafe extern "C" fn terminal_enter() -> bool {
    let mut buf: *mut buf_T = curbuf.get();
    '_c2rust_label: {
        if !(*buf).terminal.is_null() {
        } else {
            __assert_fail(
                b"buf->terminal\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/terminal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                903 as ::core::ffi::c_uint,
                b"_Bool terminal_enter(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut s: [TerminalState; 1] = [TerminalState {
        state: vim_state {
            check: None,
            execute: None,
        },
        term: ::core::ptr::null_mut::<Terminal>(),
        save_rd: 0,
        close: false,
        got_bsl: false,
        got_bsl_o: false,
        cursor_visible: false,
        save_curwin_handle: 0,
        save_w_p_cul: false,
        save_w_p_culopt: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        save_w_p_culopt_flags: 0,
        save_w_p_cuc: 0,
        save_w_p_so: 0,
        save_w_p_siso: 0,
    }];
    (*(&raw mut s as *mut TerminalState)).term = (*buf).terminal;
    (*(&raw mut s as *mut TerminalState)).cursor_visible = true_0 != 0;
    stop_insert_mode.set(false_0 != 0);
    terminal_check_size((*(&raw mut s as *mut TerminalState)).term);
    let mut save_state: ::core::ffi::c_int = State.get();
    (*(&raw mut s as *mut TerminalState)).save_rd = RedrawingDisabled.get();
    State.set(MODE_TERMINAL as ::core::ffi::c_int);
    (*mapped_ctrl_c.ptr()) |= MODE_TERMINAL as ::core::ffi::c_int;
    RedrawingDisabled.set(false_0);
    set_terminal_winopts(&raw mut s as *mut TerminalState);
    (*(*(&raw mut s as *mut TerminalState)).term).pending.cursor = true_0 != 0;
    adjust_topline_cursor(
        (*(&raw mut s as *mut TerminalState)).term,
        buf,
        0 as ::core::ffi::c_int,
    );
    showmode();
    ui_cursor_shape();
    terminal_focus((*(&raw mut s as *mut TerminalState)).term, true_0 != 0);
    (*curbuf.get()).b_last_changedtick_i = buf_get_changedtick(curbuf.get());
    (*(*(&raw mut s as *mut TerminalState)).term).refcount =
        (*(*(&raw mut s as *mut TerminalState)).term)
            .refcount
            .wrapping_add(1);
    apply_autocmds(
        EVENT_TERMENTER,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
    may_trigger_modechanged();
    (*(*(&raw mut s as *mut TerminalState)).term).refcount =
        (*(*(&raw mut s as *mut TerminalState)).term)
            .refcount
            .wrapping_sub(1);
    if (*(*(&raw mut s as *mut TerminalState)).term).buf_handle == 0 as ::core::ffi::c_int {
        (*(&raw mut s as *mut TerminalState)).close = true_0 != 0;
    }
    (*(&raw mut s as *mut TerminalState)).state.execute = Some(
        terminal_execute
            as unsafe extern "C" fn(*mut VimState, ::core::ffi::c_int) -> ::core::ffi::c_int,
    ) as state_execute_callback;
    (*(&raw mut s as *mut TerminalState)).state.check =
        Some(terminal_check as unsafe extern "C" fn(*mut VimState) -> ::core::ffi::c_int)
            as state_check_callback;
    state_enter(&raw mut (*(&raw mut s as *mut TerminalState)).state);
    if !(*(&raw mut s as *mut TerminalState)).got_bsl_o {
        restart_edit.set(0 as ::core::ffi::c_int);
    }
    State.set(save_state);
    RedrawingDisabled.set((*(&raw mut s as *mut TerminalState)).save_rd);
    if !(*(&raw mut s as *mut TerminalState)).cursor_visible {
        ui_busy_stop();
    }
    parse_shape_opt(SHAPE_CURSOR);
    unset_terminal_winopts(&raw mut s as *mut TerminalState);
    terminal_focus((*(&raw mut s as *mut TerminalState)).term, false_0 != 0);
    (*curbuf.get()).b_last_changedtick = buf_get_changedtick(curbuf.get());
    if (*curbuf.get()).terminal == (*(&raw mut s as *mut TerminalState)).term
        && !(*(&raw mut s as *mut TerminalState)).close
    {
        terminal_check_cursor();
    }
    if restart_edit.get() != 0 {
        showmode();
    } else {
        unshowmode(true_0 != 0);
    }
    ui_cursor_shape();
    if (*(&raw mut s as *mut TerminalState)).close {
        (*(*(&raw mut s as *mut TerminalState)).term).refcount =
            (*(*(&raw mut s as *mut TerminalState)).term)
                .refcount
                .wrapping_add(1);
    }
    apply_autocmds(
        EVENT_TERMLEAVE,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
    if (*(&raw mut s as *mut TerminalState)).close {
        (*(*(&raw mut s as *mut TerminalState)).term).refcount =
            (*(*(&raw mut s as *mut TerminalState)).term)
                .refcount
                .wrapping_sub(1);
        let buf_handle: handle_T = (*(*(&raw mut s as *mut TerminalState)).term).buf_handle;
        (*(*(&raw mut s as *mut TerminalState)).term).destroy = true_0 != 0;
        (*(*(&raw mut s as *mut TerminalState)).term)
            .opts
            .close_cb
            .expect("non-null function pointer")(
            (*(*(&raw mut s as *mut TerminalState)).term).opts.data,
        );
        if buf_handle != 0 as ::core::ffi::c_int {
            do_buffer(
                DOBUF_WIPE as ::core::ffi::c_int,
                DOBUF_FIRST as ::core::ffi::c_int,
                FORWARD as ::core::ffi::c_int,
                buf_handle as ::core::ffi::c_int,
                true_0,
            );
        }
    }
    return (*(&raw mut s as *mut TerminalState)).got_bsl_o;
}
unsafe extern "C" fn terminal_check_cursor() {
    let mut term: *mut Terminal = (*curbuf.get()).terminal;
    (*curwin.get()).w_cursor.lnum = if (*curbuf.get()).b_ml.ml_line_count
        < row_to_linenr(term, (*term).cursor.row) as linenr_T
    {
        (*curbuf.get()).b_ml.ml_line_count
    } else {
        row_to_linenr(term, (*term).cursor.row) as linenr_T
    };
    let topline: linenr_T = if (*curbuf.get()).b_ml.ml_line_count
        - (*curwin.get()).w_view_height as linenr_T
        + 1 as linenr_T
        > 1 as linenr_T
    {
        (*curbuf.get()).b_ml.ml_line_count - (*curwin.get()).w_view_height as linenr_T
            + 1 as linenr_T
    } else {
        1 as linenr_T
    };
    if topline != (*curwin.get()).w_topline {
        set_topline(curwin.get(), topline);
    }
    if (*term).suspended as ::core::ffi::c_int != 0
        && State.get() & MODE_TERMINAL as ::core::ffi::c_int != 0
    {
        (*curwin.get()).w_cursor = pos_T {
            lnum: (*curbuf.get()).b_ml.ml_line_count,
            col: 0,
            coladd: 0,
        };
    } else {
        let mut off: ::core::ffi::c_int = if State.get() & MODE_TERMINAL as ::core::ffi::c_int != 0
        {
            0 as ::core::ffi::c_int
        } else if (*curwin.get()).w_onebuf_opt.wo_rl != 0 {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
        coladvance(
            curwin.get(),
            if 0 as ::core::ffi::c_int > (*term).cursor.col + off {
                0 as colnr_T
            } else {
                (*term).cursor.col as colnr_T + off as colnr_T
            },
        );
    };
}
unsafe extern "C" fn terminal_check_focus(s: *mut TerminalState) -> bool {
    if (*curbuf.get()).terminal.is_null() {
        return false_0 != 0;
    }
    if (*s).save_curwin_handle != (*curwin.get()).handle {
        unset_terminal_winopts(s);
        set_terminal_winopts(s);
    }
    if (*s).term != (*curbuf.get()).terminal {
        terminal_focus((*s).term, false_0 != 0);
        if (*s).close {
            (*(*s).term).destroy = true_0 != 0;
            (*(*s).term)
                .opts
                .close_cb
                .expect("non-null function pointer")((*(*s).term).opts.data);
            (*s).close = false_0 != 0;
        }
        (*s).term = (*curbuf.get()).terminal;
        (*(*s).term).pending.cursor = true_0 != 0;
        invalidate_terminal(
            (*s).term,
            -1 as ::core::ffi::c_int,
            -1 as ::core::ffi::c_int,
        );
        terminal_focus((*s).term, true_0 != 0);
    }
    return true_0 != 0;
}
unsafe extern "C" fn terminal_check(mut state: *mut VimState) -> ::core::ffi::c_int {
    let s: *mut TerminalState = state as *mut TerminalState;
    '_c2rust_label: {
        if !(*s).close
            || (*(*s).term).buf_handle == 0 as ::core::ffi::c_int
                && (*s).term != (*curbuf.get()).terminal
        {
        } else {
            __assert_fail(
                b"!s->close || (s->term->buf_handle == 0 && s->term != curbuf->terminal)\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/terminal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1053 as ::core::ffi::c_uint,
                b"int terminal_check(VimState *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if stop_insert_mode.get() as ::core::ffi::c_int != 0 || !terminal_check_focus(s) {
        return 0 as ::core::ffi::c_int;
    }
    terminal_check_refresh();
    terminal_check_cursor();
    validate_cursor(curwin.get());
    (*(*s).term).refcount = (*(*s).term).refcount.wrapping_add(1);
    if has_event(EVENT_TEXTCHANGEDT) as ::core::ffi::c_int != 0
        && (*curbuf.get()).b_last_changedtick_i != buf_get_changedtick(curbuf.get())
    {
        apply_autocmds(
            EVENT_TEXTCHANGEDT,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        (*curbuf.get()).b_last_changedtick_i = buf_get_changedtick(curbuf.get());
    }
    may_trigger_win_scrolled_resized();
    (*(*s).term).refcount = (*(*s).term).refcount.wrapping_sub(1);
    if (*(*s).term).buf_handle == 0 as ::core::ffi::c_int {
        (*s).close = true_0 != 0;
    }
    if !terminal_check_focus(s) {
        return 0 as ::core::ffi::c_int;
    }
    terminal_check_cursor();
    validate_cursor(curwin.get());
    show_cursor_info_later(false_0 != 0);
    if must_redraw.get() != 0 {
        update_screen();
    } else {
        redraw_statuslines();
        if clear_cmdline.get() as ::core::ffi::c_int != 0
            || redraw_cmdline.get() as ::core::ffi::c_int != 0
            || redraw_mode.get() as ::core::ffi::c_int != 0
        {
            showmode();
        }
    }
    setcursor();
    refresh_cursor((*s).term, &raw mut (*s).cursor_visible);
    ui_flush();
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn terminal_execute(
    mut state: *mut VimState,
    mut key: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut s: *mut TerminalState = state as *mut TerminalState;
    let mut tmp_mod_mask: ::core::ffi::c_int = mod_mask.get();
    let mut mod_key: ::core::ffi::c_int = merge_modifiers(key, &raw mut tmp_mod_mask);
    's_214: {
        'c_47026: {
            match mod_key {
                K_LEFTMOUSE | K_LEFTDRAG | -12029 | K_MIDDLEMOUSE | K_MIDDLEDRAG
                | K_MIDDLERELEASE | K_RIGHTMOUSE | K_RIGHTDRAG | K_RIGHTRELEASE | K_X1MOUSE
                | K_X1DRAG | K_X1RELEASE | K_X2MOUSE | K_X2DRAG | K_X2RELEASE | -19453 | -19709
                | -19965 | -20221 | -25853 => {
                    if send_mouse_event((*s).term, key) {
                        return 0 as ::core::ffi::c_int;
                    }
                    break 's_214;
                }
                K_PASTE_START => {
                    paste_repeat(1 as ::core::ffi::c_int);
                    break 's_214;
                }
                K_EVENT => {
                    (*(*s).term).refcount = (*(*s).term).refcount.wrapping_add(1);
                    state_handle_k_event();
                    (*(*s).term).refcount = (*(*s).term).refcount.wrapping_sub(1);
                    if (*(*s).term).buf_handle == 0 as ::core::ffi::c_int {
                        (*s).close = true_0 != 0;
                    }
                    break 's_214;
                }
                K_COMMAND => {
                    do_cmdline(
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        Some(
                            getcmdkeycmd
                                as unsafe extern "C" fn(
                                    ::core::ffi::c_int,
                                    *mut ::core::ffi::c_void,
                                    ::core::ffi::c_int,
                                    bool,
                                )
                                    -> *mut ::core::ffi::c_char,
                        ),
                        NULL_0,
                        0 as ::core::ffi::c_int,
                    );
                    break 's_214;
                }
                K_LUA => {
                    map_execute_lua(false_0 != 0, false_0 != 0);
                    break 's_214;
                }
                K_IGNORE | K_NOP => {
                    break 's_214;
                }
                Ctrl_N => {
                    if (*s).got_bsl {
                        return 0 as ::core::ffi::c_int;
                    }
                }
                Ctrl_O => {}
                _ => {
                    break 'c_47026;
                }
            }
            if (*s).got_bsl {
                (*s).got_bsl_o = true_0 != 0;
                restart_edit.set('I' as ::core::ffi::c_int);
                return 0 as ::core::ffi::c_int;
            }
        }
        if mod_key == Ctrl_C {
            got_int.set(false_0 != 0);
        }
        if mod_key == Ctrl_BSL && !(*s).got_bsl {
            (*s).got_bsl = true_0 != 0;
        } else if (*(*s).term).suspended {
            (*(*s).term)
                .opts
                .resume_cb
                .expect("non-null function pointer")((*(*s).term).opts.data);
            terminal_set_state((*s).term, false_0 != 0);
        } else {
            if (*(*s).term).closed {
                (*s).close = true_0 != 0;
                return 0 as ::core::ffi::c_int;
            }
            (*s).got_bsl = false_0 != 0;
            terminal_send_key((*s).term, key);
        }
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn terminal_destroy(mut termpp: *mut *mut Terminal) {
    let mut term: *mut Terminal = *termpp;
    let mut buf: *mut buf_T = map_get_int_ptr_t(
        buffer_handles.ptr(),
        (*term).buf_handle as ::core::ffi::c_int,
    ) as *mut buf_T;
    if !buf.is_null() {
        (*term).buf_handle = 0 as ::core::ffi::c_int as handle_T;
        (*buf).terminal = ::core::ptr::null_mut::<Terminal>();
    }
    if (*term).refcount == 0 {
        if set_has_ptr_t(invalidated_terminals.ptr(), term as ptr_t) {
            block_autocmds();
            refresh_terminal(term);
            unblock_autocmds();
            set_del_ptr_t(invalidated_terminals.ptr(), term as ptr_t);
        }
        let mut i: size_t = 0 as size_t;
        while i < (*term).sb_current {
            xfree(*(*term).sb_buffer.offset(i as isize) as *mut ::core::ffi::c_void);
            i = i.wrapping_add(1);
        }
        xfree((*term).sb_buffer as *mut ::core::ffi::c_void);
        xfree((*term).title as *mut ::core::ffi::c_void);
        xfree((*term).selection_buffer as *mut ::core::ffi::c_void);
        xfree((*term).selection.items as *mut ::core::ffi::c_void);
        (*term).selection.capacity = 0 as size_t;
        (*term).selection.size = (*term).selection.capacity;
        (*term).selection.items = ::core::ptr::null_mut::<::core::ffi::c_char>();
        xfree((*term).termrequest_buffer.items as *mut ::core::ffi::c_void);
        (*term).termrequest_buffer.capacity = 0 as size_t;
        (*term).termrequest_buffer.size = (*term).termrequest_buffer.capacity;
        (*term).termrequest_buffer.items = ::core::ptr::null_mut::<::core::ffi::c_char>();
        vterm_free((*term).vt);
        multiqueue_free((*term).pending.events);
        xfree(term as *mut ::core::ffi::c_void);
        *termpp = ::core::ptr::null_mut::<Terminal>();
    }
}
unsafe extern "C" fn terminal_send(
    mut term: *mut Terminal,
    mut data: *const ::core::ffi::c_char,
    mut size: size_t,
) {
    if (*term).closed {
        return;
    }
    if !(*term).pending.send.is_null() {
        if size > 0 as size_t {
            if (*(*term).pending.send).capacity < (*(*term).pending.send).size.wrapping_add(size) {
                (*(*term).pending.send).capacity = (*(*term).pending.send).size.wrapping_add(size);
                (*(*term).pending.send).capacity = (*(*term).pending.send).capacity.wrapping_sub(1);
                (*(*term).pending.send).capacity |=
                    (*(*term).pending.send).capacity >> 1 as ::core::ffi::c_int;
                (*(*term).pending.send).capacity |=
                    (*(*term).pending.send).capacity >> 2 as ::core::ffi::c_int;
                (*(*term).pending.send).capacity |=
                    (*(*term).pending.send).capacity >> 4 as ::core::ffi::c_int;
                (*(*term).pending.send).capacity |=
                    (*(*term).pending.send).capacity >> 8 as ::core::ffi::c_int;
                (*(*term).pending.send).capacity |=
                    (*(*term).pending.send).capacity >> 16 as ::core::ffi::c_int;
                (*(*term).pending.send).capacity = (*(*term).pending.send).capacity.wrapping_add(1);
                (*(*term).pending.send).capacity = (*(*term).pending.send).capacity;
                (*(*term).pending.send).items = xrealloc(
                    (*(*term).pending.send).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul((*(*term).pending.send).capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label: {
                if !(*(*term).pending.send).items.is_null() {
                } else {
                    __assert_fail(
                        b"(*term->pending.send).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/terminal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1249 as ::core::ffi::c_uint,
                        b"void terminal_send(Terminal *, const char *, size_t)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            memcpy(
                (*(*term).pending.send)
                    .items
                    .offset((*(*term).pending.send).size as isize)
                    as *mut ::core::ffi::c_void,
                data as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(size),
            );
            (*(*term).pending.send).size = (*(*term).pending.send).size.wrapping_add(size);
        }
        return;
    }
    (*term).opts.write_cb.expect("non-null function pointer")(data, size, (*term).opts.data);
}
unsafe extern "C" fn is_filter_char(mut c: ::core::ffi::c_int) -> bool {
    let mut flag: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    match c {
        8 => {
            flag = kOptTpfFlagBS as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        9 => {
            flag = kOptTpfFlagHT as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        10 | 13 => {}
        12 => {
            flag = kOptTpfFlagFF as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        27 => {
            flag = kOptTpfFlagESC as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        127 => {
            flag = kOptTpfFlagDEL as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        _ => {
            if c < ' ' as ::core::ffi::c_int {
                flag = kOptTpfFlagC0 as ::core::ffi::c_int as ::core::ffi::c_uint;
            } else if c >= 0x80 as ::core::ffi::c_int && c <= 0x9f as ::core::ffi::c_int {
                flag = kOptTpfFlagC1 as ::core::ffi::c_int as ::core::ffi::c_uint;
            }
        }
    }
    return tpf_flags.get() & flag != 0;
}
#[no_mangle]
pub unsafe extern "C" fn terminal_set_streamed_paste(mut term: *mut Terminal, mut streamed: bool) {
    if (*term).streamed_paste as ::core::ffi::c_int != streamed as ::core::ffi::c_int {
        if streamed {
            vterm_keyboard_start_paste((*(*curbuf.get()).terminal).vt);
        } else {
            vterm_keyboard_end_paste((*(*curbuf.get()).terminal).vt);
        }
    }
    (*term).streamed_paste = streamed;
}
pub unsafe extern "C" fn terminal_paste(
    mut count: ::core::ffi::c_int,
    mut y_array: *mut String_0,
    mut y_size: size_t,
) {
    if y_size == 0 as size_t {
        return;
    }
    if !(*(*curbuf.get()).terminal).streamed_paste {
        vterm_keyboard_start_paste((*(*curbuf.get()).terminal).vt);
    }
    let mut buff_len: size_t = (*y_array.offset(0 as ::core::ffi::c_int as isize)).size;
    let mut buff: *mut ::core::ffi::c_char = xmalloc(buff_len) as *mut ::core::ffi::c_char;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < count {
        let mut j: size_t = 0 as size_t;
        while j < y_size {
            if j != 0 {
                terminal_send(
                    (*curbuf.get()).terminal,
                    b"\n\0".as_ptr() as *const ::core::ffi::c_char,
                    1 as size_t,
                );
            }
            let mut len: size_t = (*y_array.offset(j as isize)).size;
            if len > buff_len {
                buff = xrealloc(buff as *mut ::core::ffi::c_void, len) as *mut ::core::ffi::c_char;
                buff_len = len;
            }
            let mut dst: *mut ::core::ffi::c_char = buff;
            let mut src: *mut ::core::ffi::c_char = (*y_array.offset(j as isize)).data;
            while *src as ::core::ffi::c_int != NUL {
                len = utf_ptr2len(src) as size_t;
                let mut c: ::core::ffi::c_int = utf_ptr2char(src);
                if !is_filter_char(c) {
                    memcpy(
                        dst as *mut ::core::ffi::c_void,
                        src as *const ::core::ffi::c_void,
                        len,
                    );
                    dst = dst.offset(len as isize);
                }
                src = src.offset(len as isize);
            }
            terminal_send(
                (*curbuf.get()).terminal,
                buff,
                dst.offset_from(buff) as size_t,
            );
            j = j.wrapping_add(1);
        }
        i += 1;
    }
    xfree(buff as *mut ::core::ffi::c_void);
    if !(*(*curbuf.get()).terminal).streamed_paste {
        vterm_keyboard_end_paste((*(*curbuf.get()).terminal).vt);
    }
}
unsafe extern "C" fn terminal_send_key(mut term: *mut Terminal, mut c: ::core::ffi::c_int) {
    let mut mod_0: VTermModifier = VTERM_MOD_NONE;
    if c == K_ZERO {
        c = Ctrl_AT;
    }
    let mut key: VTermKey = convert_key(&raw mut c, &raw mut mod_0);
    if key as ::core::ffi::c_uint != VTERM_KEY_NONE as ::core::ffi::c_int as ::core::ffi::c_uint {
        vterm_keyboard_key((*term).vt, key, mod_0);
    } else if !(c < 0 as ::core::ffi::c_int) {
        vterm_keyboard_unichar((*term).vt, c as uint32_t, mod_0);
    }
}
unsafe extern "C" fn on_sync_flush(mut argv: *mut *mut ::core::ffi::c_void) {
    if exiting.get() {
        return;
    }
    let mut buf_handle: handle_T = (*argv.offset(0 as ::core::ffi::c_int as isize))
        .expose_provenance() as intptr_t as handle_T;
    let mut buf: *mut buf_T =
        map_get_int_ptr_t(buffer_handles.ptr(), buf_handle as ::core::ffi::c_int) as *mut buf_T;
    if buf.is_null() || (*buf).terminal.is_null() {
        return;
    }
    block_autocmds();
    refresh_terminal((*buf).terminal);
    unblock_autocmds();
}
#[no_mangle]
pub unsafe extern "C" fn terminal_receive(
    mut term: *mut Terminal,
    mut data: *const ::core::ffi::c_char,
    mut len: size_t,
) {
    if data.is_null() {
        return;
    }
    if (*term).opts.force_crlf {
        let mut crlf_data: StringBuilder = StringBuilder {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut i: size_t = 0 as size_t;
        while i < len {
            if *data.offset(i as isize) as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
                && (i == 0 as size_t
                    || i > 0 as size_t
                        && *data.offset(i.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                            != '\r' as ::core::ffi::c_int)
            {
                if crlf_data.size == crlf_data.capacity {
                    crlf_data.capacity = if crlf_data.capacity != 0 {
                        crlf_data.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    crlf_data.items = xrealloc(
                        crlf_data.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<::core::ffi::c_char>()
                            .wrapping_mul(crlf_data.capacity),
                    ) as *mut ::core::ffi::c_char;
                } else {
                };
                let c2rust_fresh8 = crlf_data.size;
                crlf_data.size = crlf_data.size.wrapping_add(1);
                *crlf_data.items.offset(c2rust_fresh8 as isize) = '\r' as ::core::ffi::c_char;
            }
            if crlf_data.size == crlf_data.capacity {
                crlf_data.capacity = if crlf_data.capacity != 0 {
                    crlf_data.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                crlf_data.items = xrealloc(
                    crlf_data.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(crlf_data.capacity),
                ) as *mut ::core::ffi::c_char;
            } else {
            };
            let c2rust_fresh9 = crlf_data.size;
            crlf_data.size = crlf_data.size.wrapping_add(1);
            *crlf_data.items.offset(c2rust_fresh9 as isize) = *data.offset(i as isize);
            i = i.wrapping_add(1);
        }
        vterm_input_write((*term).vt, crlf_data.items, crlf_data.size);
        xfree(crlf_data.items as *mut ::core::ffi::c_void);
        crlf_data.capacity = 0 as size_t;
        crlf_data.size = crlf_data.capacity;
        crlf_data.items = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        vterm_input_write((*term).vt, data, len);
    }
    vterm_screen_flush_damage((*term).vts);
    if (*term).sync_flush_pending {
        (*term).sync_flush_pending = false_0 != 0;
        let mut height: ::core::ffi::c_int = 0;
        vterm_get_size(
            (*term).vt,
            &raw mut height,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
        (*term).invalid_start = 0 as ::core::ffi::c_int;
        (*term).invalid_end = height;
        multiqueue_put_event(
            (*main_loop.ptr()).events,
            Event {
                handler: Some(
                    on_sync_flush as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                        (*term).buf_handle as intptr_t as usize,
                    ),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ],
            },
        );
    }
}
unsafe extern "C" fn get_rgb(
    mut state: *mut VTermState,
    mut color: VTermColor,
) -> ::core::ffi::c_int {
    vterm_state_convert_color_to_rgb(state, &raw mut color);
    return (color.rgb.red as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
        | (color.rgb.green as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
        | color.rgb.blue as ::core::ffi::c_int;
}
unsafe extern "C" fn get_underline_hl_flag(mut attrs: VTermScreenCellAttrs) -> ::core::ffi::c_int {
    match attrs.underline() as ::core::ffi::c_int {
        0 => return 0 as ::core::ffi::c_int,
        1 => return HL_UNDERLINE as ::core::ffi::c_int,
        2 => return HL_UNDERDOUBLE as ::core::ffi::c_int,
        3 => return HL_UNDERCURL as ::core::ffi::c_int,
        _ => return HL_UNDERLINE as ::core::ffi::c_int,
    };
}
#[no_mangle]
pub unsafe extern "C" fn terminal_get_line_attributes(
    mut term: *mut Terminal,
    mut _wp: *mut win_T,
    mut linenr: ::core::ffi::c_int,
    mut term_attrs: *mut ::core::ffi::c_int,
) {
    let mut height: ::core::ffi::c_int = 0;
    let mut width: ::core::ffi::c_int = 0;
    vterm_get_size((*term).vt, &raw mut height, &raw mut width);
    let mut state: *mut VTermState = vterm_obtain_state((*term).vt);
    '_c2rust_label: {
        if linenr != 0 {
        } else {
            __assert_fail(
                b"linenr\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/terminal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1448 as ::core::ffi::c_uint,
                b"void terminal_get_line_attributes(Terminal *, win_T *, int, int *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut row: ::core::ffi::c_int = linenr_to_row(term, linenr);
    if row >= height {
        return;
    }
    width = if (TERM_ATTRS_MAX as ::core::ffi::c_int) < width {
        TERM_ATTRS_MAX as ::core::ffi::c_int
    } else {
        width
    };
    let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while col < width {
        let mut cell: VTermScreenCell = VTermScreenCell {
            schar: 0,
            width: 0,
            attrs: VTermScreenCellAttrs {
                bold_underline_italic_blink_reverse_conceal_strike_font_dwl_dhl_small_baseline_dim_overline: [0; 3],
                c2rust_padding: [0; 1],
            },
            fg: VTermColor { type_0: 0 },
            bg: VTermColor { type_0: 0 },
            uri: 0,
        };
        let mut color_valid: bool = fetch_cell(term, row, col, &raw mut cell);
        let mut fg_default: bool = !color_valid
            || cell.fg.type_0 as ::core::ffi::c_int & VTERM_COLOR_DEFAULT_FG as ::core::ffi::c_int
                != 0;
        let mut bg_default: bool = !color_valid
            || cell.bg.type_0 as ::core::ffi::c_int & VTERM_COLOR_DEFAULT_BG as ::core::ffi::c_int
                != 0;
        let mut vt_fg: ::core::ffi::c_int = if fg_default as ::core::ffi::c_int != 0 {
            -1 as ::core::ffi::c_int
        } else {
            get_rgb(state, cell.fg)
        };
        let mut vt_bg: ::core::ffi::c_int = if bg_default as ::core::ffi::c_int != 0 {
            -1 as ::core::ffi::c_int
        } else {
            get_rgb(state, cell.bg)
        };
        let mut fg_indexed: bool = cell.fg.type_0 as ::core::ffi::c_int
            & VTERM_COLOR_TYPE_MASK as ::core::ffi::c_int
            == VTERM_COLOR_INDEXED as ::core::ffi::c_int;
        let mut bg_indexed: bool = cell.bg.type_0 as ::core::ffi::c_int
            & VTERM_COLOR_TYPE_MASK as ::core::ffi::c_int
            == VTERM_COLOR_INDEXED as ::core::ffi::c_int;
        let mut vt_fg_idx: int16_t = (if !fg_default && fg_indexed as ::core::ffi::c_int != 0 {
            cell.fg.indexed.idx as ::core::ffi::c_int + 1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as int16_t;
        let mut vt_bg_idx: int16_t = (if !bg_default && bg_indexed as ::core::ffi::c_int != 0 {
            cell.bg.indexed.idx as ::core::ffi::c_int + 1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as int16_t;
        let mut fg_set: bool = vt_fg_idx as ::core::ffi::c_int != 0
            && vt_fg_idx as ::core::ffi::c_int <= 16 as ::core::ffi::c_int
            && (*term).color_set
                [(vt_fg_idx as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as usize]
                as ::core::ffi::c_int
                != 0;
        let mut bg_set: bool = vt_bg_idx as ::core::ffi::c_int != 0
            && vt_bg_idx as ::core::ffi::c_int <= 16 as ::core::ffi::c_int
            && (*term).color_set
                [(vt_bg_idx as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as usize]
                as ::core::ffi::c_int
                != 0;
        let mut hl_attrs: ::core::ffi::c_int =
            (if cell.attrs.bold() as ::core::ffi::c_int != 0 {
                HL_BOLD as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) | (if cell.attrs.dim() as ::core::ffi::c_int != 0 {
                HL_DIM as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) | (if cell.attrs.blink() as ::core::ffi::c_int != 0 {
                HL_BLINK as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) | (if cell.attrs.conceal() as ::core::ffi::c_int != 0 {
                HL_CONCEALED as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) | (if cell.attrs.overline() as ::core::ffi::c_int != 0 {
                HL_OVERLINE as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) | (if cell.attrs.italic() as ::core::ffi::c_int != 0 {
                HL_ITALIC as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) | (if cell.attrs.reverse() as ::core::ffi::c_int != 0 {
                HL_INVERSE as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) | get_underline_hl_flag(cell.attrs)
                | (if cell.attrs.strike() as ::core::ffi::c_int != 0 {
                    HL_STRIKETHROUGH as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                })
                | (if fg_indexed as ::core::ffi::c_int != 0 && !fg_set {
                    HL_FG_INDEXED as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                })
                | (if bg_indexed as ::core::ffi::c_int != 0 && !bg_set {
                    HL_BG_INDEXED as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                });
        let mut attr_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if hl_attrs != 0 || !fg_default || !bg_default {
            let mut c2rust_lvalue: HlAttrs = HlAttrs {
                rgb_ae_attr: hl_attrs as int32_t,
                cterm_ae_attr: hl_attrs as int32_t,
                rgb_fg_color: vt_fg as RgbValue,
                rgb_bg_color: vt_bg as RgbValue,
                rgb_sp_color: -1 as RgbValue,
                cterm_fg_color: vt_fg_idx,
                cterm_bg_color: vt_bg_idx,
                hl_blend: -1 as int32_t,
                url: -1 as int32_t,
            };
            attr_id = hl_get_term_attr(&raw mut c2rust_lvalue);
        }
        if cell.uri > 0 as ::core::ffi::c_int {
            attr_id = hl_combine_attr(attr_id, cell.uri);
        }
        *term_attrs.offset(col as isize) = attr_id;
        col += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn terminal_buf(mut term: *const Terminal) -> Buffer {
    return (*term).buf_handle as Buffer;
}
#[no_mangle]
pub unsafe extern "C" fn terminal_running(mut term: *const Terminal) -> bool {
    return !(*term).closed;
}
#[no_mangle]
pub unsafe extern "C" fn terminal_suspended(mut term: *const Terminal) -> bool {
    return (*term).suspended;
}
#[no_mangle]
pub unsafe extern "C" fn terminal_notify_theme(mut term: *mut Terminal, mut dark: bool) {
    if !(*term).theme_updates {
        return;
    }
    let mut buf: [::core::ffi::c_char; 10] = [0; 10];
    let mut ret: ssize_t = snprintf(
        &raw mut buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 10]>(),
        b"\x1B[997;%cn\0".as_ptr() as *const ::core::ffi::c_char,
        if dark as ::core::ffi::c_int != 0 {
            '1' as ::core::ffi::c_int
        } else {
            '2' as ::core::ffi::c_int
        },
    ) as ssize_t;
    '_c2rust_label: {
        if ret > 0 as ssize_t {
        } else {
            __assert_fail(
                b"ret > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/terminal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1539 as ::core::ffi::c_uint,
                b"void terminal_notify_theme(Terminal *, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    '_c2rust_label_0: {
        if ret as size_t <= ::core::mem::size_of::<[::core::ffi::c_char; 10]>() {
        } else {
            __assert_fail(
                b"(size_t)ret <= sizeof(buf)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/terminal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1540 as ::core::ffi::c_uint,
                b"void terminal_notify_theme(Terminal *, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    terminal_send(
        term,
        &raw mut buf as *mut ::core::ffi::c_char,
        ret as size_t,
    );
}
unsafe extern "C" fn terminal_focus(mut term: *const Terminal, mut focus: bool) {
    let mut state: *mut VTermState = vterm_obtain_state((*term).vt);
    if focus {
        vterm_state_focus_in(state);
    } else {
        vterm_state_focus_out(state);
    };
}
unsafe extern "C" fn term_damage(
    mut rect: VTermRect,
    mut data: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    invalidate_terminal(data as *mut Terminal, rect.start_row, rect.end_row);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn term_moverect(
    mut dest: VTermRect,
    mut src: VTermRect,
    mut data: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    invalidate_terminal(
        data as *mut Terminal,
        if dest.start_row < src.start_row {
            dest.start_row
        } else {
            src.start_row
        },
        if dest.end_row > src.end_row {
            dest.end_row
        } else {
            src.end_row
        },
    );
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn term_movecursor(
    mut new_pos: VTermPos,
    mut _old_pos: VTermPos,
    mut _visible: ::core::ffi::c_int,
    mut data: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut term: *mut Terminal = data as *mut Terminal;
    (*term).cursor.row = new_pos.row;
    (*term).cursor.col = new_pos.col;
    invalidate_terminal(term, -1 as ::core::ffi::c_int, -1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn buf_set_term_title(
    mut buf: *mut buf_T,
    mut title: *const ::core::ffi::c_char,
    mut len: size_t,
) {
    if buf.is_null() {
        return;
    }
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    (*buf).b_locked += 1;
    dict_set_var(
        (*buf).b_vars,
        String_0 {
            data: b"term_title\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            size: ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as size_t),
        },
        object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: String_0 {
                    data: title as *mut ::core::ffi::c_char,
                    size: len,
                },
            },
        },
        false_0 != 0,
        false_0 != 0,
        ::core::ptr::null_mut::<Arena>(),
        &raw mut err,
    );
    (*buf).b_locked -= 1;
    api_clear_error(&raw mut err);
    status_redraw_buf(buf);
}
unsafe extern "C" fn term_settermprop(
    mut prop: VTermProp,
    mut val: *mut VTermValue,
    mut data: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut term: *mut Terminal = data as *mut Terminal;
    match prop as ::core::ffi::c_uint {
        3 => {
            (*term).in_altscreen = (*val).boolean != 0;
        }
        1 => {
            (*term).cursor.visible = (*val).boolean != 0;
            invalidate_terminal(term, -1 as ::core::ffi::c_int, -1 as ::core::ffi::c_int);
        }
        4 => {
            let mut buf: *mut buf_T = map_get_int_ptr_t(
                buffer_handles.ptr(),
                (*term).buf_handle as ::core::ffi::c_int,
            ) as *mut buf_T;
            let mut frag: VTermStringFragment = (*val).string;
            if frag.initial() as ::core::ffi::c_int != 0
                && frag.final_0() as ::core::ffi::c_int != 0
            {
                buf_set_term_title(buf, frag.str, frag.len());
            } else {
                if frag.initial() {
                    (*term).title_len = 0 as size_t;
                    (*term).title_size =
                        (if frag.len() as ::core::ffi::c_int > 1024 as ::core::ffi::c_int {
                            frag.len() as ::core::ffi::c_int
                        } else {
                            1024 as ::core::ffi::c_int
                        }) as size_t;
                    (*term).title = xmalloc(
                        ::core::mem::size_of::<*mut ::core::ffi::c_char>()
                            .wrapping_mul((*term).title_size),
                    ) as *mut ::core::ffi::c_char;
                } else if (*term).title_len.wrapping_add(frag.len()) > (*term).title_size {
                    (*term).title_size = (*term).title_size.wrapping_mul(2 as size_t);
                    (*term).title = xrealloc(
                        (*term).title as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<*mut ::core::ffi::c_char>()
                            .wrapping_mul((*term).title_size),
                    ) as *mut ::core::ffi::c_char;
                }
                memcpy(
                    (*term).title.offset((*term).title_len as isize) as *mut ::core::ffi::c_void,
                    frag.str as *const ::core::ffi::c_void,
                    frag.len(),
                );
                (*term).title_len = (*term).title_len.wrapping_add(frag.len());
                if frag.final_0() {
                    buf_set_term_title(buf, (*term).title, (*term).title_len);
                    xfree((*term).title as *mut ::core::ffi::c_void);
                    (*term).title = ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
            }
        }
        8 => {
            (*term).forward_mouse = (*val).number != 0;
        }
        2 => {
            (*term).cursor.blink = (*val).boolean != 0;
            (*term).pending.cursor = true_0 != 0;
            invalidate_terminal(term, -1 as ::core::ffi::c_int, -1 as ::core::ffi::c_int);
        }
        7 => {
            (*term).cursor.shape = (*val).number;
            (*term).pending.cursor = true_0 != 0;
            invalidate_terminal(term, -1 as ::core::ffi::c_int, -1 as ::core::ffi::c_int);
        }
        10 => {
            (*term).theme_updates = (*val).boolean != 0;
        }
        11 => {
            (*term).synchronized_output = (*val).boolean != 0;
            if (*val).boolean == 0 {
                (*term).sync_flush_pending = true_0 != 0;
            }
        }
        _ => return 0 as ::core::ffi::c_int,
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn term_bell(mut _data: *mut ::core::ffi::c_void) -> ::core::ffi::c_int {
    vim_beep(kOptBoFlagTerm as ::core::ffi::c_int as ::core::ffi::c_uint);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn term_theme(
    mut dark: *mut bool,
    mut _data: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    *dark = *p_bg.get() as ::core::ffi::c_int == 'd' as ::core::ffi::c_int;
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn term_sb_push(
    mut cols: ::core::ffi::c_int,
    mut cells: *const VTermScreenCell,
    mut data: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut term: *mut Terminal = data as *mut Terminal;
    if !term_may_alloc_scrollback(term, ::core::ptr::null_mut::<buf_T>()) {
        return 0 as ::core::ffi::c_int;
    }
    '_c2rust_label: {
        if (*term).sb_size > 0 as size_t {
        } else {
            __assert_fail(
                b"term->sb_size > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/terminal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1705 as ::core::ffi::c_uint,
                b"int term_sb_push(int, const VTermScreenCell *, void *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut c: size_t = cols as size_t;
    let mut sbrow: *mut ScrollbackLine = ::core::ptr::null_mut::<ScrollbackLine>();
    if (*term).sb_current == (*term).sb_size {
        if (**(*term)
            .sb_buffer
            .offset((*term).sb_current.wrapping_sub(1 as size_t) as isize))
        .cols
            == c
        {
            sbrow = *(*term)
                .sb_buffer
                .offset((*term).sb_current.wrapping_sub(1 as size_t) as isize);
        } else {
            xfree(
                *(*term)
                    .sb_buffer
                    .offset((*term).sb_current.wrapping_sub(1 as size_t) as isize)
                    as *mut ::core::ffi::c_void,
            );
        }
        (*term).sb_deleted = (*term).sb_deleted.wrapping_add(1);
        memmove(
            (*term).sb_buffer.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            (*term).sb_buffer as *const ::core::ffi::c_void,
            ::core::mem::size_of::<*mut ScrollbackLine>()
                .wrapping_mul((*term).sb_current.wrapping_sub(1 as size_t)),
        );
    } else if (*term).sb_current > 0 as size_t {
        memmove(
            (*term).sb_buffer.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            (*term).sb_buffer as *const ::core::ffi::c_void,
            ::core::mem::size_of::<*mut ScrollbackLine>().wrapping_mul((*term).sb_current),
        );
    }
    if sbrow.is_null() {
        sbrow = xmalloc(
            ::core::mem::size_of::<ScrollbackLine>()
                .wrapping_add(c.wrapping_mul(::core::mem::size_of::<VTermScreenCell>())),
        ) as *mut ScrollbackLine;
        (*sbrow).cols = c;
    }
    *(*term).sb_buffer.offset(0 as ::core::ffi::c_int as isize) = sbrow;
    if (*term).sb_current < (*term).sb_size {
        (*term).sb_current = (*term).sb_current.wrapping_add(1);
    }
    if (*term).sb_pending < (*term).sb_size as ::core::ffi::c_int {
        (*term).sb_pending += 1;
    }
    memcpy(
        &raw mut (*sbrow).cells as *mut VTermScreenCell as *mut ::core::ffi::c_void,
        cells as *const ::core::ffi::c_void,
        ::core::mem::size_of::<VTermScreenCell>().wrapping_mul(c),
    );
    if !(*term).synchronized_output {
        set_put_ptr_t(
            invalidated_terminals.ptr(),
            term as ptr_t,
            ::core::ptr::null_mut::<*mut ptr_t>(),
        );
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn term_sb_pop(
    mut cols: ::core::ffi::c_int,
    mut cells: *mut VTermScreenCell,
    mut data: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut term: *mut Terminal = data as *mut Terminal;
    if (*term).sb_current == 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*term).sb_pending > 0 as ::core::ffi::c_int {
        (*term).sb_pending -= 1;
    } else {
        (*term).old_height += 1;
    }
    let mut sbrow: *mut ScrollbackLine =
        *(*term).sb_buffer.offset(0 as ::core::ffi::c_int as isize);
    (*term).sb_current = (*term).sb_current.wrapping_sub(1);
    memmove(
        (*term).sb_buffer as *mut ::core::ffi::c_void,
        (*term).sb_buffer.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        ::core::mem::size_of::<*mut ScrollbackLine>().wrapping_mul((*term).sb_current),
    );
    let mut cols_to_copy: size_t = if (cols as size_t) < (*sbrow).cols {
        cols as size_t
    } else {
        (*sbrow).cols
    };
    memcpy(
        cells as *mut ::core::ffi::c_void,
        &raw mut (*sbrow).cells as *mut VTermScreenCell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<VTermScreenCell>().wrapping_mul(cols_to_copy),
    );
    let mut col: size_t = cols_to_copy;
    while col < cols as size_t {
        (*cells.offset(col as isize)).schar = 0 as schar_T;
        (*cells.offset(col as isize)).width = 1 as ::core::ffi::c_char;
        col = col.wrapping_add(1);
    }
    xfree(sbrow as *mut ::core::ffi::c_void);
    if !(*term).synchronized_output {
        set_put_ptr_t(
            invalidated_terminals.ptr(),
            term as ptr_t,
            ::core::ptr::null_mut::<*mut ptr_t>(),
        );
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn term_sb_clear(mut data: *mut ::core::ffi::c_void) -> ::core::ffi::c_int {
    let mut term: *mut Terminal = data as *mut Terminal;
    if (*term).in_altscreen as ::core::ffi::c_int != 0
        || (*term).sb_size == 0
        || (*term).sb_current == 0
    {
        return 1 as ::core::ffi::c_int;
    }
    let mut i: size_t = 0 as size_t;
    while i < (*term).sb_current {
        xfree(*(*term).sb_buffer.offset(i as isize) as *mut ::core::ffi::c_void);
        i = i.wrapping_add(1);
    }
    (*term).sb_deleted = (*term).sb_deleted.wrapping_add((*term).sb_current);
    (*term).sb_current = 0 as size_t;
    (*term).sb_pending = 0 as ::core::ffi::c_int;
    invalidate_terminal(term, -1 as ::core::ffi::c_int, -1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn term_clipboard_set(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut mask: VTermSelectionMask = (*argv.offset(0 as ::core::ffi::c_int as isize))
        .expose_provenance() as ::core::ffi::c_long
        as VTermSelectionMask;
    let mut data: *mut ::core::ffi::c_char =
        *argv.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char;
    let mut regname: ::core::ffi::c_char = 0;
    match mask as ::core::ffi::c_uint {
        1 => {
            regname = '+' as ::core::ffi::c_char;
        }
        2 => {
            regname = '*' as ::core::ffi::c_char;
        }
        _ => {
            regname = '+' as ::core::ffi::c_char;
        }
    }
    let mut lines: *mut list_T = tv_list_alloc(1 as ptrdiff_t);
    tv_list_append_allocated_string(lines, data);
    let mut args: *mut list_T = tv_list_alloc(3 as ptrdiff_t);
    tv_list_append_list(args, lines);
    let regtype: ::core::ffi::c_char = 'v' as ::core::ffi::c_char;
    tv_list_append_string(args, &raw const regtype, 1 as ssize_t);
    tv_list_append_string(args, &raw mut regname, 1 as ssize_t);
    eval_call_provider(
        b"clipboard\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"set\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args,
        true_0 != 0,
    );
}
unsafe extern "C" fn term_selection_set(
    mut mask: VTermSelectionMask,
    mut frag: VTermStringFragment,
    mut user: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut term: *mut Terminal = user as *mut Terminal;
    if frag.initial() {
        (*term).selection.size = 0 as size_t;
    }
    if frag.len() as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
        if (*term).selection.capacity < (*term).selection.size.wrapping_add(frag.len()) {
            (*term).selection.capacity = (*term).selection.size.wrapping_add(frag.len());
            (*term).selection.capacity = (*term).selection.capacity.wrapping_sub(1);
            (*term).selection.capacity |= (*term).selection.capacity >> 1 as ::core::ffi::c_int;
            (*term).selection.capacity |= (*term).selection.capacity >> 2 as ::core::ffi::c_int;
            (*term).selection.capacity |= (*term).selection.capacity >> 4 as ::core::ffi::c_int;
            (*term).selection.capacity |= (*term).selection.capacity >> 8 as ::core::ffi::c_int;
            (*term).selection.capacity |= (*term).selection.capacity >> 16 as ::core::ffi::c_int;
            (*term).selection.capacity = (*term).selection.capacity.wrapping_add(1);
            (*term).selection.capacity = (*term).selection.capacity;
            (*term).selection.items = xrealloc(
                (*term).selection.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul((*term).selection.capacity),
            ) as *mut ::core::ffi::c_char;
        }
        '_c2rust_label: {
            if !(*term).selection.items.is_null() {
            } else {
                __assert_fail(
                    b"(term->selection).items\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/terminal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1851 as ::core::ffi::c_uint,
                    b"int term_selection_set(VTermSelectionMask, VTermStringFragment, void *)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        memcpy(
            (*term)
                .selection
                .items
                .offset((*term).selection.size as isize) as *mut ::core::ffi::c_void,
            frag.str as *const ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(frag.len()),
        );
        (*term).selection.size = (*term).selection.size.wrapping_add(frag.len());
    }
    if frag.final_0() {
        let mut data: *mut ::core::ffi::c_char = xmemdupz(
            (*term).selection.items as *const ::core::ffi::c_void,
            (*term).selection.size,
        ) as *mut ::core::ffi::c_char;
        multiqueue_put_event(
            (*main_loop.ptr()).events,
            Event {
                handler: Some(
                    term_clipboard_set as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(mask as usize),
                    data as *mut ::core::ffi::c_void,
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ],
            },
        );
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn convert_modifiers(
    mut key: *mut ::core::ffi::c_int,
    mut statep: *mut VTermModifier,
) {
    if mod_mask.get() & MOD_MASK_SHIFT != 0 {
        *statep = (*statep as ::core::ffi::c_uint
            | VTERM_MOD_SHIFT as ::core::ffi::c_int as ::core::ffi::c_uint)
            as VTermModifier;
    }
    if mod_mask.get() & MOD_MASK_CTRL != 0 {
        *statep = (*statep as ::core::ffi::c_uint
            | VTERM_MOD_CTRL as ::core::ffi::c_int as ::core::ffi::c_uint)
            as VTermModifier;
        if mod_mask.get() & MOD_MASK_SHIFT == 0
            && *key >= 'A' as ::core::ffi::c_int
            && *key <= 'Z' as ::core::ffi::c_int
        {
            *key += 'a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int;
        }
    }
    if mod_mask.get() & MOD_MASK_ALT != 0 {
        *statep = (*statep as ::core::ffi::c_uint
            | VTERM_MOD_ALT as ::core::ffi::c_int as ::core::ffi::c_uint)
            as VTermModifier;
    }
    match *key {
        K_S_TAB | K_S_UP | K_S_DOWN | K_S_LEFT | K_S_RIGHT | K_S_HOME | K_S_END | K_S_F1
        | K_S_F2 | K_S_F3 | K_S_F4 | K_S_F5 | K_S_F6 | K_S_F7 | K_S_F8 | K_S_F9 | K_S_F10
        | K_S_F11 | K_S_F12 => {
            *statep = (*statep as ::core::ffi::c_uint
                | VTERM_MOD_SHIFT as ::core::ffi::c_int as ::core::ffi::c_uint)
                as VTermModifier;
        }
        K_C_LEFT | K_C_RIGHT | K_C_HOME | K_C_END => {
            *statep = (*statep as ::core::ffi::c_uint
                | VTERM_MOD_CTRL as ::core::ffi::c_int as ::core::ffi::c_uint)
                as VTermModifier;
        }
        _ => {}
    };
}
unsafe extern "C" fn convert_key(
    mut key: *mut ::core::ffi::c_int,
    mut statep: *mut VTermModifier,
) -> VTermKey {
    convert_modifiers(key, statep);
    match *key {
        K_BS => return VTERM_KEY_BACKSPACE,
        K_S_TAB | TAB => return VTERM_KEY_TAB,
        Ctrl_M => return VTERM_KEY_ENTER,
        ESC => return VTERM_KEY_ESCAPE,
        K_S_UP | K_UP => return VTERM_KEY_UP,
        K_S_DOWN | K_DOWN => return VTERM_KEY_DOWN,
        K_S_LEFT | K_C_LEFT | K_LEFT => return VTERM_KEY_LEFT,
        K_S_RIGHT | K_C_RIGHT | K_RIGHT => return VTERM_KEY_RIGHT,
        K_INS => return VTERM_KEY_INS,
        K_DEL => return VTERM_KEY_DEL,
        K_S_HOME | K_C_HOME | K_HOME => return VTERM_KEY_HOME,
        K_S_END | K_C_END | K_END => return VTERM_KEY_END,
        K_PAGEUP => return VTERM_KEY_PAGEUP,
        K_PAGEDOWN => return VTERM_KEY_PAGEDOWN,
        K_K0 | K_KINS => return VTERM_KEY_KP_0,
        K_K1 | K_KEND => return VTERM_KEY_KP_1,
        K_K2 | K_KDOWN => return VTERM_KEY_KP_2,
        K_K3 | K_KPAGEDOWN => return VTERM_KEY_KP_3,
        K_K4 | K_KLEFT => return VTERM_KEY_KP_4,
        K_K5 | K_KORIGIN => return VTERM_KEY_KP_5,
        K_K6 | K_KRIGHT => return VTERM_KEY_KP_6,
        K_K7 | K_KHOME => return VTERM_KEY_KP_7,
        K_K8 | K_KUP => return VTERM_KEY_KP_8,
        K_K9 | K_KPAGEUP => return VTERM_KEY_KP_9,
        K_KDEL | K_KPOINT => return VTERM_KEY_KP_PERIOD,
        K_KENTER => return VTERM_KEY_KP_ENTER,
        K_KPLUS => return VTERM_KEY_KP_PLUS,
        K_KMINUS => return VTERM_KEY_KP_MINUS,
        K_KMULTIPLY => return VTERM_KEY_KP_MULT,
        K_KDIVIDE => return VTERM_KEY_KP_DIVIDE,
        K_S_F1 | K_F1 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_S_F2 | K_F2 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_S_F3 | K_F3 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 3 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_S_F4 | K_F4 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 4 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_S_F5 | K_F5 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 5 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_S_F6 | K_F6 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 6 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_S_F7 | K_F7 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 7 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_S_F8 | K_F8 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 8 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_S_F9 | K_F9 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 9 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_S_F10 | K_F10 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 10 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_S_F11 | K_F11 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 11 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_S_F12 | K_F12 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 12 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F13 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 13 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F14 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 14 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F15 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 15 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F16 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 16 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F17 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 17 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F18 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 18 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F19 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 19 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F20 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 20 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F21 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 21 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F22 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 22 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F23 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 23 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F24 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 24 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F25 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 25 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F26 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 26 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F27 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 27 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F28 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 28 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F29 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 29 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F30 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 30 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F31 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 31 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F32 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 32 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F33 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 33 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F34 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 34 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F35 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 35 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F36 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 36 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F37 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 37 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F38 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 38 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F39 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 39 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F40 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 40 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F41 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 41 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F42 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 42 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F43 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 43 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F44 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 44 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F45 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 45 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F46 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 46 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F47 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 47 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F48 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 48 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F49 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 49 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F50 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 50 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F51 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 51 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F52 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 52 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F53 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 53 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F54 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 54 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F55 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 55 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F56 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 56 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F57 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 57 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F58 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 58 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F59 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 59 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F60 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 60 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F61 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 61 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F62 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 62 as ::core::ffi::c_int)
                as VTermKey;
        }
        K_F63 => {
            return (VTERM_KEY_FUNCTION_0 as ::core::ffi::c_int + 63 as ::core::ffi::c_int)
                as VTermKey;
        }
        _ => return VTERM_KEY_NONE,
    };
}
unsafe extern "C" fn mouse_action(
    mut term: *mut Terminal,
    mut button: ::core::ffi::c_int,
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut pressed: bool,
    mut mod_0: VTermModifier,
) {
    vterm_mouse_move((*term).vt, row, col, mod_0);
    if button != 0 {
        vterm_mouse_button((*term).vt, button, pressed, mod_0);
    }
}
unsafe extern "C" fn send_mouse_event(mut term: *mut Terminal, mut c: ::core::ffi::c_int) -> bool {
    let mut offset: ::core::ffi::c_int = 0;
    let mut row: ::core::ffi::c_int = mouse_row.get();
    let mut col: ::core::ffi::c_int = mouse_col.get();
    let mut grid: ::core::ffi::c_int = mouse_grid.get();
    let mut mouse_win: *mut win_T = mouse_find_win_inner(&raw mut grid, &raw mut row, &raw mut col);
    if !mouse_win.is_null() {
        offset = 0;
        if !(*term).suspended
            && !(*term).closed
            && (*term).forward_mouse as ::core::ffi::c_int != 0
            && (*(*mouse_win).w_buffer).terminal == term
            && row >= 0 as ::core::ffi::c_int
            && (grid > 1 as ::core::ffi::c_int
                || row + (*mouse_win).w_winbar_height < (*mouse_win).w_height)
            && {
                offset = win_col_off(mouse_win);
                col >= offset
            }
            && (grid > 1 as ::core::ffi::c_int || col < (*mouse_win).w_width)
        {
            let mut button: ::core::ffi::c_int = 0;
            let mut pressed: bool = false_0 != 0;
            's_184: {
                'c_50995: {
                    'c_50990: {
                        'c_50985: {
                            'c_50980: {
                                match c {
                                    K_LEFTDRAG | K_LEFTMOUSE => {
                                        pressed = true_0 != 0;
                                    }
                                    -12029 => {}
                                    K_MIDDLEDRAG | K_MIDDLEMOUSE => {
                                        pressed = true_0 != 0;
                                        break 'c_50980;
                                    }
                                    K_MIDDLERELEASE => {
                                        break 'c_50980;
                                    }
                                    K_RIGHTDRAG | K_RIGHTMOUSE => {
                                        pressed = true_0 != 0;
                                        break 'c_50985;
                                    }
                                    K_RIGHTRELEASE => {
                                        break 'c_50985;
                                    }
                                    K_X1DRAG | K_X1MOUSE => {
                                        pressed = true_0 != 0;
                                        break 'c_50990;
                                    }
                                    K_X1RELEASE => {
                                        break 'c_50990;
                                    }
                                    K_X2DRAG | K_X2MOUSE => {
                                        pressed = true_0 != 0;
                                        break 'c_50995;
                                    }
                                    K_X2RELEASE => {
                                        break 'c_50995;
                                    }
                                    -19453 => {
                                        pressed = true_0 != 0;
                                        button = 4 as ::core::ffi::c_int;
                                        break 's_184;
                                    }
                                    -19709 => {
                                        pressed = true_0 != 0;
                                        button = 5 as ::core::ffi::c_int;
                                        break 's_184;
                                    }
                                    -19965 => {
                                        pressed = true_0 != 0;
                                        button = 7 as ::core::ffi::c_int;
                                        break 's_184;
                                    }
                                    -20221 => {
                                        pressed = true_0 != 0;
                                        button = 6 as ::core::ffi::c_int;
                                        break 's_184;
                                    }
                                    -25853 => {
                                        button = 0 as ::core::ffi::c_int;
                                        break 's_184;
                                    }
                                    _ => return false_0 != 0,
                                }
                                button = 1 as ::core::ffi::c_int;
                                break 's_184;
                            }
                            button = 2 as ::core::ffi::c_int;
                            break 's_184;
                        }
                        button = 3 as ::core::ffi::c_int;
                        break 's_184;
                    }
                    button = 8 as ::core::ffi::c_int;
                    break 's_184;
                }
                button = 9 as ::core::ffi::c_int;
            }
            let mut mod_0: VTermModifier = VTERM_MOD_NONE;
            convert_modifiers(&raw mut c, &raw mut mod_0);
            mouse_action(term, button, row, col - offset, pressed, mod_0);
            return false_0 != 0;
        }
        if c == -(253 as ::core::ffi::c_int
            + ((KE_MOUSEUP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c == -(253 as ::core::ffi::c_int
                + ((KE_MOUSEDOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c == -(253 as ::core::ffi::c_int
                + ((KE_MOUSELEFT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c == -(253 as ::core::ffi::c_int
                + ((KE_MOUSERIGHT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            let mut save_curwin: *mut win_T = curwin.get();
            curwin.set(mouse_win);
            curbuf.set((*curwin.get()).w_buffer);
            let mut cap: cmdarg_T = cmdarg_T {
                oap: ::core::ptr::null_mut::<oparg_T>(),
                prechar: 0,
                cmdchar: 0,
                nchar: 0,
                nchar_composing: [0; 32],
                nchar_len: 0,
                extra_char: 0,
                opcount: 0,
                count0: 0,
                count1: 0,
                arg: 0,
                retval: 0,
                searchbuf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            };
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
            memset(
                &raw mut cap as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<cmdarg_T>(),
            );
            clear_oparg(&raw mut oa);
            cap.oap = &raw mut oa;
            cap.cmdchar = c;
            match cap.cmdchar {
                -19709 => {
                    cap.arg = MSCR_UP as ::core::ffi::c_int;
                }
                -19453 => {
                    cap.arg = MSCR_DOWN as ::core::ffi::c_int;
                }
                -19965 => {
                    cap.arg = MSCR_LEFT as ::core::ffi::c_int;
                }
                -20221 => {
                    cap.arg = MSCR_RIGHT as ::core::ffi::c_int;
                }
                _ => {
                    abort();
                }
            }
            do_mousescroll(&raw mut cap);
            (*curwin.get()).w_redr_status = true_0 != 0;
            curwin.set(save_curwin);
            curbuf.set((*curwin.get()).w_buffer);
            redraw_later(mouse_win, UPD_NOT_VALID as ::core::ffi::c_int);
            invalidate_terminal(term, -1 as ::core::ffi::c_int, -1 as ::core::ffi::c_int);
            return mouse_win == curwin.get();
        }
    }
    if c == -(253 as ::core::ffi::c_int
        + ((KE_LEFTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        && !mouse_win.is_null()
        && (*(*mouse_win).w_buffer).terminal == term
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        return false_0 != 0;
    }
    let mut len: ::core::ffi::c_int =
        ins_char_typebuf(vgetc_char.get(), vgetc_mod_mask.get(), true_0 != 0);
    if KeyTyped.get() {
        ungetchars(len);
    }
    return true_0 != 0;
}
unsafe extern "C" fn fetch_row(
    mut term: *mut Terminal,
    mut row: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
) {
    let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut line_len: size_t = 0 as size_t;
    let mut ptr: *mut ::core::ffi::c_char = &raw mut (*term).textbuf as *mut ::core::ffi::c_char;
    while col < end_col {
        let mut cell: VTermScreenCell = VTermScreenCell {
            schar: 0,
            width: 0,
            attrs: VTermScreenCellAttrs {
                bold_underline_italic_blink_reverse_conceal_strike_font_dwl_dhl_small_baseline_dim_overline: [0; 3],
                c2rust_padding: [0; 1],
            },
            fg: VTermColor { type_0: 0 },
            bg: VTermColor { type_0: 0 },
            uri: 0,
        };
        fetch_cell(term, row, col, &raw mut cell);
        if cell.schar != 0 {
            schar_get_adv(&raw mut ptr, cell.schar);
            line_len =
                ptr.offset_from(&raw mut (*term).textbuf as *mut ::core::ffi::c_char) as size_t;
        } else {
            let c2rust_fresh6 = ptr;
            ptr = ptr.offset(1);
            *c2rust_fresh6 = ' ' as ::core::ffi::c_char;
        }
        col += cell.width as ::core::ffi::c_int;
    }
    (*term).textbuf[line_len as usize] = NUL as ::core::ffi::c_char;
}
unsafe extern "C" fn fetch_cell(
    mut term: *mut Terminal,
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut cell: *mut VTermScreenCell,
) -> bool {
    if row < 0 as ::core::ffi::c_int {
        let mut sbrow: *mut ScrollbackLine = *(*term)
            .sb_buffer
            .offset((-row - 1 as ::core::ffi::c_int) as isize);
        if (col as size_t) < (*sbrow).cols {
            *cell = *(&raw mut (*sbrow).cells as *mut VTermScreenCell).offset(col as isize);
        } else {
            *cell = VTermScreenCell {
                schar: 0 as schar_T,
                width: 1 as ::core::ffi::c_char,
                attrs: VTermScreenCellAttrs {
                    bold_underline_italic_blink_reverse_conceal_strike_font_dwl_dhl_small_baseline_dim_overline: [0; 3],
                    c2rust_padding: [0; 1],
                },
                fg: VTermColor { type_0: 0 },
                bg: VTermColor { type_0: 0 },
                uri: 0,
            };
            return false_0 != 0;
        }
    } else {
        vterm_screen_get_cell((*term).vts, VTermPos { row: row, col: col }, cell);
    }
    return true_0 != 0;
}
unsafe extern "C" fn invalidate_terminal(
    mut term: *mut Terminal,
    mut start_row: ::core::ffi::c_int,
    mut end_row: ::core::ffi::c_int,
) {
    if start_row != -1 as ::core::ffi::c_int && end_row != -1 as ::core::ffi::c_int {
        (*term).invalid_start = if (*term).invalid_start < start_row {
            (*term).invalid_start
        } else {
            start_row
        };
        (*term).invalid_end = if (*term).invalid_end > end_row {
            (*term).invalid_end
        } else {
            end_row
        };
    }
    if (*term).synchronized_output {
        return;
    }
    set_put_ptr_t(
        invalidated_terminals.ptr(),
        term as ptr_t,
        ::core::ptr::null_mut::<*mut ptr_t>(),
    );
    if !refresh_pending.get() {
        time_watcher_start(
            refresh_timer.ptr(),
            Some(
                refresh_timer_cb
                    as unsafe extern "C" fn(*mut TimeWatcher, *mut ::core::ffi::c_void) -> (),
            ),
            REFRESH_DELAY as uint64_t,
            0 as uint64_t,
        );
        refresh_pending.set(true_0 != 0);
    }
}
pub unsafe extern "C" fn terminal_check_refresh() {
    multiqueue_process_events((*refresh_timer.ptr()).events);
}
unsafe extern "C" fn refresh_terminal(mut term: *mut Terminal) {
    let mut buf: *mut buf_T = map_get_int_ptr_t(
        buffer_handles.ptr(),
        (*term).buf_handle as ::core::ffi::c_int,
    ) as *mut buf_T;
    if buf.is_null() {
        return;
    }
    let mut ml_before: linenr_T = (*buf).b_ml.ml_line_count;
    let mut resized: bool = refresh_size(term, buf);
    refresh_scrollback(term, buf);
    refresh_screen(term, buf);
    let mut ml_added: ::core::ffi::c_int =
        (*buf).b_ml.ml_line_count as ::core::ffi::c_int - ml_before as ::core::ffi::c_int;
    adjust_topline_cursor(term, buf, ml_added);
    if resized {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut wp: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                if (*wp).w_buffer == buf && (*wp).w_leftcol != 0 as ::core::ffi::c_int {
                    (*wp).w_leftcol = 0 as ::core::ffi::c_int as colnr_T;
                    curs_columns(wp, true_0);
                }
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
    }
    multiqueue_move_events((*main_loop.ptr()).events, (*term).pending.events);
}
unsafe extern "C" fn refresh_cursor(mut term: *mut Terminal, mut cursor_visible: *mut bool) {
    if !is_focused(term) {
        return;
    }
    if (*term).cursor.visible as ::core::ffi::c_int != *cursor_visible as ::core::ffi::c_int {
        *cursor_visible = (*term).cursor.visible;
        if *cursor_visible {
            ui_busy_stop();
        } else {
            ui_busy_start();
        }
    }
    if !(*term).pending.cursor {
        return;
    }
    (*term).pending.cursor = false_0 != 0;
    if (*term).cursor.blink {
        (*shape_table.ptr())[SHAPE_IDX_TERM as ::core::ffi::c_int as usize].blinkon =
            500 as ::core::ffi::c_int;
        (*shape_table.ptr())[SHAPE_IDX_TERM as ::core::ffi::c_int as usize].blinkoff =
            500 as ::core::ffi::c_int;
    } else {
        (*shape_table.ptr())[SHAPE_IDX_TERM as ::core::ffi::c_int as usize].blinkon =
            0 as ::core::ffi::c_int;
        (*shape_table.ptr())[SHAPE_IDX_TERM as ::core::ffi::c_int as usize].blinkoff =
            0 as ::core::ffi::c_int;
    }
    match (*term).cursor.shape {
        1 => {
            (*shape_table.ptr())[SHAPE_IDX_TERM as ::core::ffi::c_int as usize].shape = SHAPE_BLOCK;
        }
        2 => {
            (*shape_table.ptr())[SHAPE_IDX_TERM as ::core::ffi::c_int as usize].shape = SHAPE_HOR;
            (*shape_table.ptr())[SHAPE_IDX_TERM as ::core::ffi::c_int as usize].percentage =
                20 as ::core::ffi::c_int;
        }
        3 => {
            (*shape_table.ptr())[SHAPE_IDX_TERM as ::core::ffi::c_int as usize].shape = SHAPE_VER;
            (*shape_table.ptr())[SHAPE_IDX_TERM as ::core::ffi::c_int as usize].percentage =
                25 as ::core::ffi::c_int;
        }
        _ => {}
    }
    ui_mode_info_set();
}
unsafe extern "C" fn refresh_timer_cb(
    mut _watcher: *mut TimeWatcher,
    mut _data: *mut ::core::ffi::c_void,
) {
    refresh_pending.set(false_0 != 0);
    if exiting.get() {
        return;
    }
    block_autocmds();
    let mut to_refresh: Set_ptr_t = invalidated_terminals.get();
    invalidated_terminals.set(SET_INIT);
    let mut term: *mut Terminal = ::core::ptr::null_mut::<Terminal>();
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < to_refresh.h.n_keys {
        term = *to_refresh.keys.offset(__i as isize) as *mut Terminal;
        if !(*term).synchronized_output {
            refresh_terminal(term);
        }
        __i = __i.wrapping_add(1);
    }
    xfree(to_refresh.keys as *mut ::core::ffi::c_void);
    xfree(to_refresh.h.hash as *mut ::core::ffi::c_void);
    to_refresh = SET_INIT;
    unblock_autocmds();
}
unsafe extern "C" fn refresh_size(mut term: *mut Terminal, mut _buf: *mut buf_T) -> bool {
    if !(*term).pending.resize || (*term).closed as ::core::ffi::c_int != 0 {
        return false_0 != 0;
    }
    (*term).pending.resize = false_0 != 0;
    let mut width: ::core::ffi::c_int = 0;
    let mut height: ::core::ffi::c_int = 0;
    vterm_get_size((*term).vt, &raw mut height, &raw mut width);
    (*term).invalid_start = 0 as ::core::ffi::c_int;
    (*term).invalid_end = height;
    (*term).opts.resize_cb.expect("non-null function pointer")(
        width as uint16_t,
        height as uint16_t,
        (*term).opts.data,
    );
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn on_scrollback_option_changed(mut term: *mut Terminal) {
    if !(*term).sb_buffer.is_null() {
        refresh_terminal(term);
    }
}
unsafe extern "C" fn adjust_scrollback(mut term: *mut Terminal, mut buf: *mut buf_T) {
    if (*buf).b_p_scbk < 1 as OptInt {
        (*buf).b_p_scbk = SB_MAX as OptInt;
    }
    let scbk: size_t = (*buf).b_p_scbk as size_t;
    '_c2rust_label: {
        if (*term).sb_current < 18446744073709551615 as size_t {
        } else {
            __assert_fail(
                b"term->sb_current < SIZE_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/terminal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2528 as ::core::ffi::c_uint,
                b"void adjust_scrollback(Terminal *, buf_T *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if (*term).sb_pending > 0 as ::core::ffi::c_int {
        abort();
    }
    if scbk < (*term).sb_current {
        let mut diff: size_t = (*term).sb_current.wrapping_sub(scbk);
        let mut i: size_t = 0 as size_t;
        while i < diff {
            ml_delete_buf(buf, 1 as linenr_T, false_0 != 0);
            (*term).sb_current = (*term).sb_current.wrapping_sub(1);
            xfree(
                *(*term).sb_buffer.offset((*term).sb_current as isize) as *mut ::core::ffi::c_void
            );
            i = i.wrapping_add(1);
        }
        mark_adjust_buf(
            buf,
            1 as linenr_T,
            diff as linenr_T,
            MAXLNUM as ::core::ffi::c_int as linenr_T,
            -(diff as linenr_T),
            true_0 != 0,
            kMarkAdjustTerm,
            kExtmarkUndo,
        );
        deleted_lines_buf(buf, 1 as linenr_T, diff as linenr_T);
    }
    let mut sb_region: size_t = ::core::mem::size_of::<*mut ScrollbackLine>().wrapping_mul(scbk);
    if scbk != (*term).sb_size {
        (*term).sb_buffer = xrealloc((*term).sb_buffer as *mut ::core::ffi::c_void, sb_region)
            as *mut *mut ScrollbackLine;
    }
    (*term).sb_size = scbk;
}
unsafe extern "C" fn refresh_scrollback(mut term: *mut Terminal, mut buf: *mut buf_T) {
    (*term)
        .opts
        .read_pause_cb
        .expect("non-null function pointer")(true_0 != 0, (*term).opts.data);
    let mut deleted: linenr_T = (*term).sb_deleted.wrapping_sub((*term).old_sb_deleted) as linenr_T;
    deleted = if deleted < (*buf).b_ml.ml_line_count {
        deleted
    } else {
        (*buf).b_ml.ml_line_count
    };
    mark_adjust_buf(
        buf,
        1 as linenr_T,
        deleted,
        MAXLNUM as ::core::ffi::c_int as linenr_T,
        -deleted,
        true_0 != 0,
        kMarkAdjustTerm,
        kExtmarkUndo,
    );
    (*term).old_sb_deleted = (*term).sb_deleted;
    let mut old_height: ::core::ffi::c_int = (*term).old_height;
    let mut width: ::core::ffi::c_int = 0;
    let mut height: ::core::ffi::c_int = 0;
    vterm_get_size((*term).vt, &raw mut height, &raw mut width);
    while deleted > 0 as linenr_T && (*buf).b_ml.ml_line_count > old_height as linenr_T {
        ml_delete_buf(buf, 1 as linenr_T, false_0 != 0);
        deleted_lines_buf(buf, 1 as linenr_T, 1 as linenr_T);
        deleted -= 1;
    }
    old_height = (if (old_height as linenr_T) < (*buf).b_ml.ml_line_count {
        old_height as linenr_T
    } else {
        (*buf).b_ml.ml_line_count
    }) as ::core::ffi::c_int;
    while (*term).sb_pending > 0 as ::core::ffi::c_int {
        fetch_row(term, -(*term).sb_pending, width);
        let mut buf_index: ::core::ffi::c_int =
            (*buf).b_ml.ml_line_count as ::core::ffi::c_int - old_height;
        ml_append_buf(
            buf,
            buf_index as linenr_T,
            &raw mut (*term).textbuf as *mut ::core::ffi::c_char,
            0 as colnr_T,
            false_0 != 0,
        );
        appended_lines_buf(buf, buf_index as linenr_T, 1 as linenr_T);
        (*term).sb_pending -= 1;
    }
    let mut max_line_count: ::core::ffi::c_int = (*term).sb_current as ::core::ffi::c_int + height;
    while (*buf).b_ml.ml_line_count > max_line_count as linenr_T {
        ml_delete_buf(buf, (*buf).b_ml.ml_line_count, false_0 != 0);
        deleted_lines_buf(buf, (*buf).b_ml.ml_line_count, 1 as linenr_T);
    }
    adjust_scrollback(term, buf);
    (*term)
        .opts
        .read_pause_cb
        .expect("non-null function pointer")(false_0 != 0, (*term).opts.data);
}
unsafe extern "C" fn refresh_screen(mut term: *mut Terminal, mut buf: *mut buf_T) {
    let mut changed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut added: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut height: ::core::ffi::c_int = 0;
    let mut width: ::core::ffi::c_int = 0;
    vterm_get_size((*term).vt, &raw mut height, &raw mut width);
    (*term).invalid_end = if (*term).invalid_end < height {
        (*term).invalid_end
    } else {
        height
    };
    if (*term).invalid_start >= (*term).invalid_end {
        (*term).invalid_start = INT_MAX;
        (*term).invalid_end = -1 as ::core::ffi::c_int;
        return;
    }
    let mut r: ::core::ffi::c_int = (*term).invalid_start;
    let mut linenr: ::core::ffi::c_int = row_to_linenr(term, r);
    while r < (*term).invalid_end {
        fetch_row(term, r, width);
        if linenr as linenr_T <= (*buf).b_ml.ml_line_count {
            ml_replace_buf(
                buf,
                linenr as linenr_T,
                &raw mut (*term).textbuf as *mut ::core::ffi::c_char,
                true_0 != 0,
                false_0 != 0,
            );
            changed += 1;
        } else {
            ml_append_buf(
                buf,
                linenr as linenr_T - 1 as linenr_T,
                &raw mut (*term).textbuf as *mut ::core::ffi::c_char,
                0 as colnr_T,
                false_0 != 0,
            );
            added += 1;
        }
        r += 1;
        linenr += 1;
    }
    (*term).old_height = height;
    let mut change_start: ::core::ffi::c_int = row_to_linenr(term, (*term).invalid_start);
    let mut change_end: ::core::ffi::c_int = change_start + changed;
    (*term).invalid_start = INT_MAX;
    (*term).invalid_end = -1 as ::core::ffi::c_int;
    changed_lines(
        buf,
        change_start as linenr_T,
        0 as colnr_T,
        change_end as linenr_T,
        added as linenr_T,
        true_0 != 0,
    );
}
unsafe extern "C" fn adjust_topline_cursor(
    mut term: *mut Terminal,
    mut buf: *mut buf_T,
    mut added: ::core::ffi::c_int,
) {
    let mut ml_end: linenr_T = (*buf).b_ml.ml_line_count;
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == buf {
                if wp == curwin.get() && is_focused(term) as ::core::ffi::c_int != 0 {
                    terminal_check_cursor();
                } else {
                    let mut following: bool = ml_end == (*wp).w_cursor.lnum + added as linenr_T;
                    if following {
                        (*wp).w_cursor.lnum = ml_end;
                        set_topline(
                            wp,
                            if (*wp).w_cursor.lnum - (*wp).w_view_height as linenr_T + 1 as linenr_T
                                > 1 as linenr_T
                            {
                                (*wp).w_cursor.lnum - (*wp).w_view_height as linenr_T
                                    + 1 as linenr_T
                            } else {
                                1 as linenr_T
                            },
                        );
                    } else {
                        (*wp).w_cursor.lnum = if (*wp).w_cursor.lnum < ml_end {
                            (*wp).w_cursor.lnum
                        } else {
                            ml_end
                        };
                    }
                    mb_check_adjust_col(wp as *mut ::core::ffi::c_void);
                }
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    if ml_end == (*buf).b_last_cursor.mark.lnum + added as linenr_T {
        (*buf).b_last_cursor.mark.lnum = ml_end;
    }
    let mut i: size_t = 0 as size_t;
    while i < (*buf).b_wininfo.size {
        let mut wip: *mut WinInfo = *(*buf).b_wininfo.items.offset(i as isize);
        if ml_end == (*wip).wi_mark.mark.lnum + added as linenr_T {
            (*wip).wi_mark.mark.lnum = ml_end;
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn row_to_linenr(
    mut term: *mut Terminal,
    mut row: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return if row != INT_MAX {
        row + (*term).sb_current as ::core::ffi::c_int + 1 as ::core::ffi::c_int
    } else {
        INT_MAX
    };
}
unsafe extern "C" fn linenr_to_row(
    mut term: *mut Terminal,
    mut linenr: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return linenr - (*term).sb_current as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn is_focused(mut term: *mut Terminal) -> bool {
    return State.get() & MODE_TERMINAL as ::core::ffi::c_int != 0
        && (*curbuf.get()).terminal == term;
}
unsafe extern "C" fn get_config_string(
    mut buf: *mut buf_T,
    mut key: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut obj: Object = dict_get_value(
        (*buf).b_vars,
        cstr_as_string(key),
        ::core::ptr::null_mut::<Arena>(),
        &raw mut err,
    );
    api_clear_error(&raw mut err);
    if obj.type_0 as ::core::ffi::c_uint
        == kObjectTypeNil as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        obj = dict_get_value(
            get_globvar_dict(),
            cstr_as_string(key),
            ::core::ptr::null_mut::<Arena>(),
            &raw mut err,
        );
        api_clear_error(&raw mut err);
    }
    if obj.type_0 as ::core::ffi::c_uint
        == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return obj.data.string.data;
    }
    api_free_object(obj);
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub const SHAPE_CURSOR: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SB_MAX: ::core::ffi::c_int = 1000000 as ::core::ffi::c_int;
pub const K_ZERO: ::core::ffi::c_int =
    -(255 as ::core::ffi::c_int + (('X' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_UP: ::core::ffi::c_int = -30059;
pub const K_KUP: ::core::ffi::c_int = -30027;
pub const K_DOWN: ::core::ffi::c_int = -25707;
pub const K_KDOWN: ::core::ffi::c_int = -25675;
pub const K_LEFT: ::core::ffi::c_int = -27755;
pub const K_KLEFT: ::core::ffi::c_int = -27723;
pub const K_RIGHT: ::core::ffi::c_int = -29291;
pub const K_KRIGHT: ::core::ffi::c_int = -29259;
pub const K_S_UP: ::core::ffi::c_int = -1277;
pub const K_S_DOWN: ::core::ffi::c_int = -1533;
pub const K_S_LEFT: ::core::ffi::c_int = -13347;
pub const K_C_LEFT: ::core::ffi::c_int = -22013;
pub const K_S_RIGHT: ::core::ffi::c_int = -26917;
pub const K_C_RIGHT: ::core::ffi::c_int = -22269;
pub const K_S_HOME: ::core::ffi::c_int = -12835;
pub const K_C_HOME: ::core::ffi::c_int = -22525;
pub const K_S_END: ::core::ffi::c_int = -14122;
pub const K_C_END: ::core::ffi::c_int = -22781;
pub const K_S_TAB: ::core::ffi::c_int = -17003;
pub const K_F1: ::core::ffi::c_int = -12651;
pub const K_F2: ::core::ffi::c_int = -12907;
pub const K_F3: ::core::ffi::c_int = -13163;
pub const K_F4: ::core::ffi::c_int = -13419;
pub const K_F5: ::core::ffi::c_int = -13675;
pub const K_F6: ::core::ffi::c_int = -13931;
pub const K_F7: ::core::ffi::c_int = -14187;
pub const K_F8: ::core::ffi::c_int = -14443;
pub const K_F9: ::core::ffi::c_int = -14699;
pub const K_F10: ::core::ffi::c_int = -15211;
pub const K_F11: ::core::ffi::c_int = -12614;
pub const K_F12: ::core::ffi::c_int = -12870;
pub const K_F13: ::core::ffi::c_int = -13126;
pub const K_F14: ::core::ffi::c_int = -13382;
pub const K_F15: ::core::ffi::c_int = -13638;
pub const K_F16: ::core::ffi::c_int = -13894;
pub const K_F17: ::core::ffi::c_int = -14150;
pub const K_F18: ::core::ffi::c_int = -14406;
pub const K_F19: ::core::ffi::c_int = -14662;
pub const K_F20: ::core::ffi::c_int = -16710;
pub const K_F21: ::core::ffi::c_int = -16966;
pub const K_F22: ::core::ffi::c_int = -17222;
pub const K_F23: ::core::ffi::c_int = -17478;
pub const K_F24: ::core::ffi::c_int = -17734;
pub const K_F25: ::core::ffi::c_int = -17990;
pub const K_F26: ::core::ffi::c_int = -18246;
pub const K_F27: ::core::ffi::c_int = -18502;
pub const K_F28: ::core::ffi::c_int = -18758;
pub const K_F29: ::core::ffi::c_int = -19014;
pub const K_F30: ::core::ffi::c_int = -19270;
pub const K_F31: ::core::ffi::c_int = -19526;
pub const K_F32: ::core::ffi::c_int = -19782;
pub const K_F33: ::core::ffi::c_int = -20038;
pub const K_F34: ::core::ffi::c_int = -20294;
pub const K_F35: ::core::ffi::c_int = -20550;
pub const K_F36: ::core::ffi::c_int = -20806;
pub const K_F37: ::core::ffi::c_int = -21062;
pub const K_F38: ::core::ffi::c_int = -21318;
pub const K_F39: ::core::ffi::c_int = -21574;
pub const K_F40: ::core::ffi::c_int = -21830;
pub const K_F41: ::core::ffi::c_int = -22086;
pub const K_F42: ::core::ffi::c_int = -22342;
pub const K_F43: ::core::ffi::c_int = -22598;
pub const K_F44: ::core::ffi::c_int = -22854;
pub const K_F45: ::core::ffi::c_int = -23110;
pub const K_F46: ::core::ffi::c_int = -24902;
pub const K_F47: ::core::ffi::c_int = -25158;
pub const K_F48: ::core::ffi::c_int = -25414;
pub const K_F49: ::core::ffi::c_int = -25670;
pub const K_F50: ::core::ffi::c_int = -25926;
pub const K_F51: ::core::ffi::c_int = -26182;
pub const K_F52: ::core::ffi::c_int = -26438;
pub const K_F53: ::core::ffi::c_int = -26694;
pub const K_F54: ::core::ffi::c_int = -26950;
pub const K_F55: ::core::ffi::c_int = -27206;
pub const K_F56: ::core::ffi::c_int = -27462;
pub const K_F57: ::core::ffi::c_int = -27718;
pub const K_F58: ::core::ffi::c_int = -27974;
pub const K_F59: ::core::ffi::c_int = -28230;
pub const K_F60: ::core::ffi::c_int = -28486;
pub const K_F61: ::core::ffi::c_int = -28742;
pub const K_F62: ::core::ffi::c_int = -28998;
pub const K_F63: ::core::ffi::c_int = -29254;
pub const K_S_F1: ::core::ffi::c_int = -1789;
pub const K_S_F2: ::core::ffi::c_int = -2045;
pub const K_S_F3: ::core::ffi::c_int = -2301;
pub const K_S_F4: ::core::ffi::c_int = -2557;
pub const K_S_F5: ::core::ffi::c_int = -2813;
pub const K_S_F6: ::core::ffi::c_int = -3069;
pub const K_S_F7: ::core::ffi::c_int = -3325;
pub const K_S_F8: ::core::ffi::c_int = -3581;
pub const K_S_F9: ::core::ffi::c_int = -3837;
pub const K_S_F10: ::core::ffi::c_int = -4093;
pub const K_S_F11: ::core::ffi::c_int = -4349;
pub const K_S_F12: ::core::ffi::c_int = -4605;
pub const K_BS: ::core::ffi::c_int = -25195;
pub const K_INS: ::core::ffi::c_int = -18795;
pub const K_KINS: ::core::ffi::c_int = -20477;
pub const K_DEL: ::core::ffi::c_int = -17515;
pub const K_KDEL: ::core::ffi::c_int = -20733;
pub const K_HOME: ::core::ffi::c_int = -26731;
pub const K_KHOME: ::core::ffi::c_int = -12619;
pub const K_END: ::core::ffi::c_int = -14144;
pub const K_KEND: ::core::ffi::c_int = -13387;
pub const K_PAGEUP: ::core::ffi::c_int = -20587;
pub const K_PAGEDOWN: ::core::ffi::c_int = -20075;
pub const K_KPAGEUP: ::core::ffi::c_int = -13131;
pub const K_KPAGEDOWN: ::core::ffi::c_int = -13643;
pub const K_KORIGIN: ::core::ffi::c_int = -12875;
pub const K_KPLUS: ::core::ffi::c_int = -13899;
pub const K_KMINUS: ::core::ffi::c_int = -14155;
pub const K_KDIVIDE: ::core::ffi::c_int = -14411;
pub const K_KMULTIPLY: ::core::ffi::c_int = -14667;
pub const K_KENTER: ::core::ffi::c_int = -16715;
pub const K_KPOINT: ::core::ffi::c_int = -16971;
pub const K_PASTE_START: ::core::ffi::c_int = -21328;
pub const K_K0: ::core::ffi::c_int = -17227;
pub const K_K1: ::core::ffi::c_int = -17483;
pub const K_K2: ::core::ffi::c_int = -17739;
pub const K_K3: ::core::ffi::c_int = -17995;
pub const K_K4: ::core::ffi::c_int = -18251;
pub const K_K5: ::core::ffi::c_int = -18507;
pub const K_K6: ::core::ffi::c_int = -18763;
pub const K_K7: ::core::ffi::c_int = -19019;
pub const K_K8: ::core::ffi::c_int = -19275;
pub const K_K9: ::core::ffi::c_int = -19531;
pub const K_LEFTMOUSE: ::core::ffi::c_int = -11517;
pub const K_LEFTDRAG: ::core::ffi::c_int = -11773;
pub const K_MIDDLEMOUSE: ::core::ffi::c_int = -12285;
pub const K_MIDDLEDRAG: ::core::ffi::c_int = -12541;
pub const K_MIDDLERELEASE: ::core::ffi::c_int = -12797;
pub const K_RIGHTMOUSE: ::core::ffi::c_int = -13053;
pub const K_RIGHTDRAG: ::core::ffi::c_int = -13309;
pub const K_RIGHTRELEASE: ::core::ffi::c_int = -13565;
pub const K_X1MOUSE: ::core::ffi::c_int = -23037;
pub const K_X1DRAG: ::core::ffi::c_int = -23293;
pub const K_X1RELEASE: ::core::ffi::c_int = -23549;
pub const K_X2MOUSE: ::core::ffi::c_int = -23805;
pub const K_X2DRAG: ::core::ffi::c_int = -24061;
pub const K_X2RELEASE: ::core::ffi::c_int = -24317;
pub const K_IGNORE: ::core::ffi::c_int = -13821;
pub const K_NOP: ::core::ffi::c_int = -25085;
pub const K_EVENT: ::core::ffi::c_int = -26365;
pub const K_COMMAND: ::core::ffi::c_int = -26877;
pub const K_LUA: ::core::ffi::c_int = -26621;
pub const MOD_MASK_SHIFT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MOD_MASK_CTRL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const MOD_MASK_ALT: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn vterm_color_rgb(
    mut col: *mut VTermColor,
    mut red: uint8_t,
    mut green: uint8_t,
    mut blue: uint8_t,
) {
    (*col).type_0 = VTERM_COLOR_RGB as ::core::ffi::c_int as uint8_t;
    (*col).rgb.red = red;
    (*col).rgb.green = green;
    (*col).rgb.blue = blue;
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
