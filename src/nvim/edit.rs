use crate::src::nvim::autocmd::{apply_autocmds, aucmd_prepbuf, aucmd_restbuf, has_event};
use crate::src::nvim::buffer::{bt_prompt, bt_quickfix, buf_is_empty};
use crate::src::nvim::change::{
    appended_lines_mark, change_warning, changed_bytes, del_bytes, del_char, get_leader_len,
    ins_bytes_len, ins_char, ins_char_bytes, ins_str, inserted_bytes, open_line,
};
use crate::src::nvim::charset::{
    byte2cells, char2cells, hex2nr, ptr2cells, skipwhite, vim_isprintc, vim_iswordc,
};
use crate::src::nvim::cursor::{
    char_before_cursor, check_cursor, check_cursor_col, check_visual_pos, coladvance, dec_cursor,
    gchar_cursor, get_cursor_line_len, get_cursor_line_ptr, get_cursor_pos_len, get_cursor_pos_ptr,
    getviscol, inc_cursor,
};
use crate::src::nvim::decoration::{decor_conceal_line, win_lines_concealed};
use crate::src::nvim::digraph::{digraph_get, do_digraph};
use crate::src::nvim::drawscreen::{
    redrawWinline, redraw_later, redraw_statuslines, redrawing, setcursor, show_cursor_info_later,
    showmode, skip_showmode, status_redraw_curbuf, unshowmode, update_screen,
};
use crate::src::nvim::eval::vars::{get_vim_var_str, set_vim_var_string};
use crate::src::nvim::eval_1::{invoke_prompt_interrupt, prompt_invoke_callback};
use crate::src::nvim::ex_docmd::{do_cmdline, do_cmdline_cmd, expr_map_locked};
use crate::src::nvim::fileio::check_timestamps;
use crate::src::nvim::fold::{
    foldCheckClose, foldOpenCursor, foldUpdateAfterInsert, hasFolding, hasFoldingWin,
};
use crate::src::nvim::getchar::{
    char_avail, get_inserted, getcmdkeycmd, map_execute_lua, merge_modifiers, paste_repeat,
    plain_vgetc, start_redo_ins, stop_redo_ins, stuffReadbuffLen, stuffRedoReadbuff, stuff_empty,
    stuffcharReadbuff, typebuf_maplen, vgetc, vpeekc, vungetc, AppendCharToRedobuff,
    AppendNumberToRedobuff, AppendToRedobuff, AppendToRedobuffLit, ResetRedobuff,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::{
    grid_line_flush, grid_line_getchar, grid_line_put_schar, grid_line_puts, grid_line_start,
};
use crate::src::nvim::highlight_group::highlight_changed;
use crate::src::nvim::indent::{
    change_indent, fix_indent, get_indent, get_sts_value, get_sw_value, inindent, ins_try_si,
    may_do_si, tabstop_at, tabstop_count, tabstop_first, tabstop_padding, tabstop_start,
};
use crate::src::nvim::indent_c::{cindent_on, do_c_expr_indent, in_cinkeys};
use crate::src::nvim::insexpand::{
    check_compl_option, compl_status_clear, compl_status_local, ctrl_x_mode_cmdline,
    ctrl_x_mode_dictionary, ctrl_x_mode_files, ctrl_x_mode_function, ctrl_x_mode_line_or_eval,
    ctrl_x_mode_none, ctrl_x_mode_normal, ctrl_x_mode_omni, ctrl_x_mode_path_defines,
    ctrl_x_mode_path_patterns, ctrl_x_mode_register, ctrl_x_mode_scroll, ctrl_x_mode_spell,
    ctrl_x_mode_tags, ctrl_x_mode_thesaurus, ctrl_x_mode_whole_line, ins_compl_accept_char,
    ins_compl_active, ins_compl_addfrommatch, ins_compl_addleader, ins_compl_bs, ins_compl_cancel,
    ins_compl_clear, ins_compl_col, ins_compl_delete, ins_compl_enable_autocomplete,
    ins_compl_enter_selects, ins_compl_has_autocomplete, ins_compl_has_shown_match,
    ins_compl_init_get_longest, ins_compl_insert, ins_compl_is_match_selected,
    ins_compl_long_shown_match, ins_compl_preinsert_effect, ins_compl_preinsert_longest,
    ins_compl_prep, ins_compl_used_match, ins_compl_win_active, ins_complete, ins_ctrl_x,
    pum_wanted,
};
use crate::src::nvim::keycodes::{add_char2buf, get_special_key_name};
use crate::src::nvim::main::{
    ai_col, allow_keys, arrow_used, can_si, can_si_back, clear_cmdline, cmdmod, cmdwin_result,
    cmdwin_type, curbuf, curwin, default_grid, did_ai, did_check_timestamps, did_cursorhold,
    did_si, disable_fold_update, dollar_vcol, e_noinstext, e_sandbox, e_textlock,
    edit_submode_extra, emsg_on_display, end_comment_pending, ex_normal_busy, fdo_flags,
    first_tabpage, force_restart_edit, got_int, hl_attr_active, ins_at_eol, km_startsel,
    langmap_mapchar, last_cursormoved, last_cursormoved_win, mod_mask, msg_scroll, msg_silent,
    must_redraw, need_check_timestamps, need_highlight_changed, need_start_insertmode, no_abbr,
    no_mapping, no_u_sync, old_indent, orig_line_count, p_ari, p_ch, p_cpo, p_deco, p_langmap,
    p_lrm, p_paste, p_ri, p_smd, p_sol, p_sta, p_ww, redraw_cmdline, redraw_mode, reg_recording,
    replace_offset, restart_VIsual_select, restart_edit, sandbox, spell_redraw_lnum,
    stop_insert_mode, test_disable_char_avail, textlock, u_sync_once, vgetc_busy, vr_lines_changed,
    where_paste_started, Insstart, Insstart_orig, KeyStuffed, KeyTyped, RedrawingDisabled, State,
    VIsual_active,
};
use crate::src::nvim::mapping::{check_abbr, langmap_adjust_mb, map_to_exists_mode};
use crate::src::nvim::mark::{free_fmark, mark_view_make};
use crate::src::nvim::mbyte::{
    mb_adjust_cursor, mb_get_class, utf8len_tab, utf_char2bytes, utf_char2len, utf_composinglike,
    utf_head_off, utf_ptr2CharInfo_impl, utf_ptr2char, utf_ptr2len, utfc_next_impl, utfc_ptr2len,
};
use crate::src::nvim::memline::{gchar_pos, ml_append, ml_get, ml_get_buf, ml_get_len, ml_replace};
use crate::src::nvim::memory::{strnequal, xfree, xmalloc, xmemdupz, xrealloc, xstrdup};
use crate::src::nvim::message::{emsg, msg_check_for_delay};
use crate::src::nvim::mouse::{ins_mouse, ins_mousescroll, setmouse};
use crate::src::nvim::normal::{
    add_to_showcmd, add_to_showcmd_c, clear_showcmd, do_check_scrollbind, end_visual_mode,
    start_selection,
};
use crate::src::nvim::ops::do_join;
use crate::src::nvim::option::{
    can_bs, copy_option_part, get_scrolloff_value, get_ve_flags, set_iminsert_global,
};
use crate::src::nvim::os::input::line_breakcheck;
use crate::src::nvim::os::libc::{__ctype_b_loc, gettext, memcpy, memmove, memset, strcmp, strlen};
use crate::src::nvim::os::time::os_time;
use crate::src::nvim::plines::{
    charsize_fast, charsize_nowrap, charsize_regular, getvcol, getvcol_nolist, init_charsize_arg,
    linetabsize_col, win_chartabsize,
};
use crate::src::nvim::popupmenu::{pum_check_clear, pum_visible};
use crate::src::nvim::r#move::{
    adjust_skipcol, curs_columns, do_check_cursorbind, pagescroll, scrolldown_clamp,
    scrollup_clamp, set_topline, update_curswant, update_topline, validate_cursor,
    validate_cursor_col, validate_virtcol,
};
use crate::src::nvim::register::{
    do_put, get_expr_register, get_yank_register, insert_reg, valid_yank_reg,
};
use crate::src::nvim::state::{
    may_trigger_modechanged, may_trigger_safestate, state_enter, state_handle_k_event,
    virtual_active,
};
use crate::src::nvim::strings::{vim_snprintf, vim_strchr, xstrnsave};
use crate::src::nvim::syntax::syntax_present;
use crate::src::nvim::terminal::terminal_enter;
use crate::src::nvim::textformat::{
    auto_format, check_auto_format, comp_textwidth, fex_format, has_format_option, internal_format,
};
use crate::src::nvim::textobject::{bck_word, fwd_word};
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, CSType, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem, CharInfo, CharSize, CharsizeArg,
    DecorExt, DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_2, Direction, ExtmarkMove, ExtmarkSavePos, ExtmarkSplice,
    ExtmarkUndoObject, FileID, FloatAnchor, FloatRelative, GraphemeState, GridView, Intersection,
    LineGetter, LuaRef, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t,
    Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, MarkTreeIter,
    MarkTreeIter_s as C2Rust_Unnamed_16, MetaIndex, MotionType, OptInt, ScopeDictDictItem,
    ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue,
    StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_13, StrCharInfo, String_0,
    Terminal, Timestamp, TriState, UIExtension, UndoObjectType, VarLockStatus, VarType, VimState,
    VimVarIndex, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo, WinSplit,
    WinStyle, Window, __time_t, aco_save_T, alist_T, auto_event, bcount_t, bhdr_T, blob_T,
    blobvar_S, blocknr_T, buf_T, bufref_T, bufstate_T, chunksize_T, cmdarg_T, cmdmod_T, colnr_T,
    dict_T, dictvar_S, diff_T, diffblock_S, disptick_T, event_T, extmark_undo_vec_t, fcs_chars_T,
    file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_3,
    file_buffer_b_wininfo as C2Rust_Unnamed_12, file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, foldinfo_T,
    frame_S, frame_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T,
    handle_T, hash_T, hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, key_extra,
    lcs_chars_T, linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T,
    llpos_T, lpos_T, mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T,
    mfdirty_T, mtnode_inner_s, mtnode_s, oparg_T, partial_S, partial_T, pos_T, pos_save_T,
    proftime_T, ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue, reg_extmatch_T, regmatch_T,
    regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T, size_t, ssize_t,
    state_check_callback, state_execute_callback, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T, tabpage_S,
    tabpage_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T,
    u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_9,
    u_header_uh_alt_prev as C2Rust_Unnamed_8, u_header_uh_next as C2Rust_Unnamed_11,
    u_header_uh_prev as C2Rust_Unnamed_10, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    uintptr_t, undo_object, undo_object_data as C2Rust_Unnamed_7, utf8proc_int32_t, varnumber_T,
    vim_state, virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T,
    yankreg_T, QUEUE,
};
use crate::src::nvim::ui::{ui_cursor_shape, ui_flush, ui_has, vim_beep};
use crate::src::nvim::undo::{u_clearallandblockfree, u_save, u_save_cursor, u_sync};
use crate::src::nvim::window::{goto_tabpage, may_trigger_win_scrolled_resized};
extern "C" {
    static pum_want: GlobalCell<C2Rust_Unnamed_27>;
}
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISalnum: C2Rust_Unnamed = 8;
pub const _ISpunct: C2Rust_Unnamed = 4;
pub const _IScntrl: C2Rust_Unnamed = 2;
pub const _ISblank: C2Rust_Unnamed = 1;
pub const _ISgraph: C2Rust_Unnamed = 32768;
pub const _ISprint: C2Rust_Unnamed = 16384;
pub const _ISspace: C2Rust_Unnamed = 8192;
pub const _ISxdigit: C2Rust_Unnamed = 4096;
pub const _ISdigit: C2Rust_Unnamed = 2048;
pub const _ISalpha: C2Rust_Unnamed = 1024;
pub const _ISlower: C2Rust_Unnamed = 512;
pub const _ISupper: C2Rust_Unnamed = 256;
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
pub const kExtmarkClear: UndoObjectType = 4;
pub const kExtmarkSavePos: UndoObjectType = 3;
pub const kExtmarkUpdate: UndoObjectType = 2;
pub const kExtmarkMove: UndoObjectType = 1;
pub const kExtmarkSplice: UndoObjectType = 0;
pub const kStlClickFuncRun: C2Rust_Unnamed_13 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_13 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_13 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_13 = 0;
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
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_14 = 2147483647;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_15 = 76;
pub const HLF_PRE: C2Rust_Unnamed_15 = 75;
pub const HLF_OK: C2Rust_Unnamed_15 = 74;
pub const HLF_SO: C2Rust_Unnamed_15 = 73;
pub const HLF_SE: C2Rust_Unnamed_15 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_15 = 71;
pub const HLF_TS: C2Rust_Unnamed_15 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_15 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_15 = 68;
pub const HLF_CU: C2Rust_Unnamed_15 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_15 = 66;
pub const HLF_WBR: C2Rust_Unnamed_15 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_15 = 64;
pub const HLF_MSG: C2Rust_Unnamed_15 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_15 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_15 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_15 = 60;
pub const HLF_0: C2Rust_Unnamed_15 = 59;
pub const HLF_QFL: C2Rust_Unnamed_15 = 58;
pub const HLF_MC: C2Rust_Unnamed_15 = 57;
pub const HLF_CUL: C2Rust_Unnamed_15 = 56;
pub const HLF_CUC: C2Rust_Unnamed_15 = 55;
pub const HLF_TPF: C2Rust_Unnamed_15 = 54;
pub const HLF_TPS: C2Rust_Unnamed_15 = 53;
pub const HLF_TP: C2Rust_Unnamed_15 = 52;
pub const HLF_PBR: C2Rust_Unnamed_15 = 51;
pub const HLF_PST: C2Rust_Unnamed_15 = 50;
pub const HLF_PSB: C2Rust_Unnamed_15 = 49;
pub const HLF_PSX: C2Rust_Unnamed_15 = 48;
pub const HLF_PNX: C2Rust_Unnamed_15 = 47;
pub const HLF_PSK: C2Rust_Unnamed_15 = 46;
pub const HLF_PNK: C2Rust_Unnamed_15 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_15 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_15 = 43;
pub const HLF_PSI: C2Rust_Unnamed_15 = 42;
pub const HLF_PNI: C2Rust_Unnamed_15 = 41;
pub const HLF_SPL: C2Rust_Unnamed_15 = 40;
pub const HLF_SPR: C2Rust_Unnamed_15 = 39;
pub const HLF_SPC: C2Rust_Unnamed_15 = 38;
pub const HLF_SPB: C2Rust_Unnamed_15 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_15 = 36;
pub const HLF_SC: C2Rust_Unnamed_15 = 35;
pub const HLF_TXA: C2Rust_Unnamed_15 = 34;
pub const HLF_TXD: C2Rust_Unnamed_15 = 33;
pub const HLF_DED: C2Rust_Unnamed_15 = 32;
pub const HLF_CHD: C2Rust_Unnamed_15 = 31;
pub const HLF_ADD: C2Rust_Unnamed_15 = 30;
pub const HLF_FC: C2Rust_Unnamed_15 = 29;
pub const HLF_FL: C2Rust_Unnamed_15 = 28;
pub const HLF_WM: C2Rust_Unnamed_15 = 27;
pub const HLF_W: C2Rust_Unnamed_15 = 26;
pub const HLF_VNC: C2Rust_Unnamed_15 = 25;
pub const HLF_V: C2Rust_Unnamed_15 = 24;
pub const HLF_T: C2Rust_Unnamed_15 = 23;
pub const HLF_VSP: C2Rust_Unnamed_15 = 22;
pub const HLF_C: C2Rust_Unnamed_15 = 21;
pub const HLF_SNC: C2Rust_Unnamed_15 = 20;
pub const HLF_S: C2Rust_Unnamed_15 = 19;
pub const HLF_R: C2Rust_Unnamed_15 = 18;
pub const HLF_CLF: C2Rust_Unnamed_15 = 17;
pub const HLF_CLS: C2Rust_Unnamed_15 = 16;
pub const HLF_CLN: C2Rust_Unnamed_15 = 15;
pub const HLF_LNB: C2Rust_Unnamed_15 = 14;
pub const HLF_LNA: C2Rust_Unnamed_15 = 13;
pub const HLF_N: C2Rust_Unnamed_15 = 12;
pub const HLF_CM: C2Rust_Unnamed_15 = 11;
pub const HLF_M: C2Rust_Unnamed_15 = 10;
pub const HLF_LC: C2Rust_Unnamed_15 = 9;
pub const HLF_L: C2Rust_Unnamed_15 = 8;
pub const HLF_I: C2Rust_Unnamed_15 = 7;
pub const HLF_E: C2Rust_Unnamed_15 = 6;
pub const HLF_D: C2Rust_Unnamed_15 = 5;
pub const HLF_AT: C2Rust_Unnamed_15 = 4;
pub const HLF_TERM: C2Rust_Unnamed_15 = 3;
pub const HLF_EOB: C2Rust_Unnamed_15 = 2;
pub const HLF_8: C2Rust_Unnamed_15 = 1;
pub const HLF_NONE: C2Rust_Unnamed_15 = 0;
pub const kMTMetaCount: MetaIndex = 5;
pub const kMTMetaConcealLines: MetaIndex = 4;
pub const kMTMetaSignText: MetaIndex = 3;
pub const kMTMetaSignHL: MetaIndex = 2;
pub const kMTMetaLines: MetaIndex = 1;
pub const kMTMetaInline: MetaIndex = 0;
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_17 = 8192;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_17 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_17 = 2048;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_17 = 1024;
pub const CMOD_KEEPMARKS: C2Rust_Unnamed_17 = 512;
pub const CMOD_KEEPALT: C2Rust_Unnamed_17 = 256;
pub const CMOD_CONFIRM: C2Rust_Unnamed_17 = 128;
pub const CMOD_BROWSE: C2Rust_Unnamed_17 = 64;
pub const CMOD_HIDE: C2Rust_Unnamed_17 = 32;
pub const CMOD_NOAUTOCMD: C2Rust_Unnamed_17 = 16;
pub const CMOD_UNSILENT: C2Rust_Unnamed_17 = 8;
pub const CMOD_ERRSILENT: C2Rust_Unnamed_17 = 4;
pub const CMOD_SILENT: C2Rust_Unnamed_17 = 2;
pub const CMOD_SANDBOX: C2Rust_Unnamed_17 = 1;
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
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const OPENLINE_FORCE_INDENT: C2Rust_Unnamed_18 = 64;
pub const OPENLINE_FORMAT: C2Rust_Unnamed_18 = 32;
pub const OPENLINE_COM_LIST: C2Rust_Unnamed_18 = 16;
pub const OPENLINE_MARKFIX: C2Rust_Unnamed_18 = 8;
pub const OPENLINE_KEEPTRAIL: C2Rust_Unnamed_18 = 4;
pub const OPENLINE_DO_COM: C2Rust_Unnamed_18 = 2;
pub const OPENLINE_DELSPACES: C2Rust_Unnamed_18 = 1;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const kOptBoFlagWildmode: C2Rust_Unnamed_19 = 524288;
pub const kOptBoFlagTerm: C2Rust_Unnamed_19 = 262144;
pub const kOptBoFlagSpell: C2Rust_Unnamed_19 = 131072;
pub const kOptBoFlagShell: C2Rust_Unnamed_19 = 65536;
pub const kOptBoFlagRegister: C2Rust_Unnamed_19 = 32768;
pub const kOptBoFlagOperator: C2Rust_Unnamed_19 = 16384;
pub const kOptBoFlagShowmatch: C2Rust_Unnamed_19 = 8192;
pub const kOptBoFlagMess: C2Rust_Unnamed_19 = 4096;
pub const kOptBoFlagLang: C2Rust_Unnamed_19 = 2048;
pub const kOptBoFlagInsertmode: C2Rust_Unnamed_19 = 1024;
pub const kOptBoFlagHangul: C2Rust_Unnamed_19 = 512;
pub const kOptBoFlagEx: C2Rust_Unnamed_19 = 256;
pub const kOptBoFlagEsc: C2Rust_Unnamed_19 = 128;
pub const kOptBoFlagError: C2Rust_Unnamed_19 = 64;
pub const kOptBoFlagCtrlg: C2Rust_Unnamed_19 = 32;
pub const kOptBoFlagCopy: C2Rust_Unnamed_19 = 16;
pub const kOptBoFlagComplete: C2Rust_Unnamed_19 = 8;
pub const kOptBoFlagCursor: C2Rust_Unnamed_19 = 4;
pub const kOptBoFlagBackspace: C2Rust_Unnamed_19 = 2;
pub const kOptBoFlagAll: C2Rust_Unnamed_19 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const kOptFdoFlagJump: C2Rust_Unnamed_20 = 1024;
pub const kOptFdoFlagUndo: C2Rust_Unnamed_20 = 512;
pub const kOptFdoFlagInsert: C2Rust_Unnamed_20 = 256;
pub const kOptFdoFlagTag: C2Rust_Unnamed_20 = 128;
pub const kOptFdoFlagSearch: C2Rust_Unnamed_20 = 64;
pub const kOptFdoFlagQuickfix: C2Rust_Unnamed_20 = 32;
pub const kOptFdoFlagPercent: C2Rust_Unnamed_20 = 16;
pub const kOptFdoFlagMark: C2Rust_Unnamed_20 = 8;
pub const kOptFdoFlagHor: C2Rust_Unnamed_20 = 4;
pub const kOptFdoFlagBlock: C2Rust_Unnamed_20 = 2;
pub const kOptFdoFlagAll: C2Rust_Unnamed_20 = 1;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const kOptVeFlagNoneU: C2Rust_Unnamed_21 = 32;
pub const kOptVeFlagNone: C2Rust_Unnamed_21 = 16;
pub const kOptVeFlagOnemore: C2Rust_Unnamed_21 = 8;
pub const kOptVeFlagInsert: C2Rust_Unnamed_21 = 6;
pub const kOptVeFlagBlock: C2Rust_Unnamed_21 = 5;
pub const kOptVeFlagAll: C2Rust_Unnamed_21 = 4;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_22 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_22 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_22 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_22 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_22 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_22 = 20;
pub const UPD_VALID: C2Rust_Unnamed_22 = 10;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const INDENT_DEC: C2Rust_Unnamed_23 = 3;
pub const INDENT_INC: C2Rust_Unnamed_23 = 2;
pub const INDENT_SET: C2Rust_Unnamed_23 = 1;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_24 = 4;
pub const BL_SOL: C2Rust_Unnamed_24 = 2;
pub const BL_WHITE: C2Rust_Unnamed_24 = 1;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const INSCHAR_COM_LIST: C2Rust_Unnamed_25 = 16;
pub const INSCHAR_NO_FEX: C2Rust_Unnamed_25 = 8;
pub const INSCHAR_CTRLV: C2Rust_Unnamed_25 = 4;
pub const INSCHAR_DO_COM: C2Rust_Unnamed_25 = 2;
pub const INSCHAR_FORMAT: C2Rust_Unnamed_25 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct InsertState {
    pub state: VimState,
    pub ca: *mut cmdarg_T,
    pub mincol: ::core::ffi::c_int,
    pub cmdchar: ::core::ffi::c_int,
    pub cmdchar_todo: ::core::ffi::c_int,
    pub ins_just_started: bool,
    pub startln: ::core::ffi::c_int,
    pub count: ::core::ffi::c_int,
    pub c: ::core::ffi::c_int,
    pub lastc: ::core::ffi::c_int,
    pub i: ::core::ffi::c_int,
    pub did_backspace: bool,
    pub line_is_white: bool,
    pub old_topline: linenr_T,
    pub old_topfill: ::core::ffi::c_int,
    pub inserted_space: ::core::ffi::c_int,
    pub replaceState: ::core::ffi::c_int,
    pub did_restart_edit: ::core::ffi::c_int,
    pub nomove: bool,
}
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
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
pub const MODE_NORMAL: C2Rust_Unnamed_29 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_26 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ::core::ffi::c_char,
}
pub const REPLACE_FLAG: C2Rust_Unnamed_29 = 256;
pub const MODE_INSERT: C2Rust_Unnamed_29 = 16;
pub const MODE_LANGMAP: C2Rust_Unnamed_29 = 32;
pub const MODE_VREPLACE: C2Rust_Unnamed_29 = 784;
pub const MODE_REPLACE: C2Rust_Unnamed_29 = 272;
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
pub const VREPLACE_FLAG: C2Rust_Unnamed_29 = 512;
pub const KE_EVENT: key_extra = 102;
pub const KE_NOP: key_extra = 97;
pub const kCharsizeFast: C2Rust_Unnamed_33 = 1;
pub const KE_S_DOWN: key_extra = 5;
pub const KE_S_UP: key_extra = 4;
pub const KE_C_RIGHT: key_extra = 86;
pub const KE_C_LEFT: key_extra = 85;
pub const KE_C_END: key_extra = 88;
pub const KE_C_HOME: key_extra = 87;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_27 {
    pub active: bool,
    pub item: ::core::ffi::c_int,
    pub insert: bool,
    pub finish: bool,
}
pub const KE_LUA: key_extra = 103;
pub const KE_COMMAND: key_extra = 104;
pub const KE_IGNORE: key_extra = 53;
pub const MSCR_RIGHT: C2Rust_Unnamed_30 = -2;
pub const KE_MOUSERIGHT: key_extra = 78;
pub const MSCR_LEFT: C2Rust_Unnamed_30 = -1;
pub const KE_MOUSELEFT: key_extra = 77;
pub const MSCR_UP: C2Rust_Unnamed_30 = 1;
pub const KE_MOUSEUP: key_extra = 76;
pub const MSCR_DOWN: C2Rust_Unnamed_30 = 0;
pub const KE_MOUSEDOWN: key_extra = 75;
pub const KE_X2RELEASE: key_extra = 94;
pub const KE_X2DRAG: key_extra = 93;
pub const KE_X2MOUSE: key_extra = 92;
pub const KE_X1RELEASE: key_extra = 91;
pub const KE_X1DRAG: key_extra = 90;
pub const KE_X1MOUSE: key_extra = 89;
pub const KE_RIGHTRELEASE: key_extra = 52;
pub const KE_RIGHTDRAG: key_extra = 51;
pub const KE_RIGHTMOUSE: key_extra = 50;
pub const KE_MIDDLERELEASE: key_extra = 49;
pub const KE_MIDDLEDRAG: key_extra = 48;
pub const KE_MIDDLEMOUSE: key_extra = 47;
pub const KE_MOUSEMOVE: key_extra = 100;
pub const KE_LEFTRELEASE_NM: key_extra = 70;
pub const KE_LEFTRELEASE: key_extra = 46;
pub const KE_LEFTDRAG: key_extra = 45;
pub const KE_LEFTMOUSE_NM: key_extra = 69;
pub const KE_LEFTMOUSE: key_extra = 44;
pub const BACKSPACE_LINE: C2Rust_Unnamed_34 = 4;
pub const BACKSPACE_CHAR: C2Rust_Unnamed_34 = 1;
pub const BACKSPACE_WORD_NOT_SPACE: C2Rust_Unnamed_34 = 3;
pub const BACKSPACE_WORD: C2Rust_Unnamed_34 = 2;
pub const KE_KDEL: key_extra = 80;
pub const PUT_CURSEND: C2Rust_Unnamed_31 = 2;
pub const YREG_PASTE: C2Rust_Unnamed_32 = 0;
pub const PUT_FIXINDENT: C2Rust_Unnamed_31 = 1;
pub const KE_XF1: key_extra = 57;
pub const KE_KINS: key_extra = 79;
pub const MODE_CMDLINE: C2Rust_Unnamed_29 = 8;
pub const MB_MAXBYTES: C2Rust_Unnamed_28 = 21;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const MB_MAXCHAR: C2Rust_Unnamed_28 = 6;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_29 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_29 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_29 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_29 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_29 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_29 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_29 = 288;
pub const MAP_ALL_MODES: C2Rust_Unnamed_29 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_29 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_29 = 64;
pub const MODE_OP_PENDING: C2Rust_Unnamed_29 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_29 = 2;
pub const KE_WILD: key_extra = 108;
pub const KE_DROP: key_extra = 95;
pub const KE_CMDWIN: key_extra = 84;
pub const KE_PLUG: key_extra = 83;
pub const KE_SNR: key_extra = 82;
pub const KE_S_XF4: key_extra = 74;
pub const KE_S_XF3: key_extra = 73;
pub const KE_S_XF2: key_extra = 72;
pub const KE_S_XF1: key_extra = 71;
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
pub const KE_S_TAB_OLD: key_extra = 55;
pub const KE_TAB: key_extra = 54;
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
pub type C2Rust_Unnamed_30 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const PUT_BLOCK_INNER: C2Rust_Unnamed_31 = 64;
pub const PUT_LINE_FORWARD: C2Rust_Unnamed_31 = 32;
pub const PUT_LINE_SPLIT: C2Rust_Unnamed_31 = 16;
pub const PUT_LINE: C2Rust_Unnamed_31 = 8;
pub const PUT_CURSLINE: C2Rust_Unnamed_31 = 4;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub const YREG_PUT: C2Rust_Unnamed_32 = 2;
pub const YREG_YANK: C2Rust_Unnamed_32 = 1;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub const kCharsizeRegular: C2Rust_Unnamed_33 = 0;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: C2Rust_Unnamed_26 = C2Rust_Unnamed_26 {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
};
pub const VALID_WROW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VALID_WCOL: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VALID_VIRTCOL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const B_IMODE_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const B_IMODE_LMAP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const ML_LINE_DIRTY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const ML_ALLOCATED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn equalpos(mut a: pos_T, mut b: pos_T) -> bool {
    return a.lnum == b.lnum && a.col == b.col && a.coladd == b.coladd;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn buf_get_changedtick(buf: *const buf_T) -> varnumber_T {
    return (*buf).changedtick_di.di_tv.vval.v_number;
}
#[inline]
unsafe extern "C" fn buf_meta_total(mut b: *const buf_T, mut m: MetaIndex) -> uint32_t {
    return (*(&raw const (*b).b_marktree as *const MarkTree)).meta_root[m as usize];
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const NL_STR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"\n\0") };
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
pub const ESC_STR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"\x1B\0") };
pub const DEL: ::core::ffi::c_int = 0x7f as ::core::ffi::c_int;
pub const CTRL_V_STR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"\x16\0") };
pub const Ctrl_A: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const Ctrl_C: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const Ctrl_D: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const Ctrl_E: ::core::ffi::c_int = 5;
pub const Ctrl_F: ::core::ffi::c_int = 6;
pub const Ctrl_G: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const Ctrl_H: ::core::ffi::c_int = 8;
pub const Ctrl_J: ::core::ffi::c_int = 10;
pub const Ctrl_K: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const Ctrl_L: ::core::ffi::c_int = 12;
pub const Ctrl_N: ::core::ffi::c_int = 14;
pub const Ctrl_O: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const Ctrl_P: ::core::ffi::c_int = 16;
pub const Ctrl_Q: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub const Ctrl_R: ::core::ffi::c_int = 18 as ::core::ffi::c_int;
pub const Ctrl_S: ::core::ffi::c_int = 19;
pub const Ctrl_T: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const Ctrl_U: ::core::ffi::c_int = 21;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const Ctrl_W: ::core::ffi::c_int = 23 as ::core::ffi::c_int;
pub const Ctrl_X: ::core::ffi::c_int = 24;
pub const Ctrl_Y: ::core::ffi::c_int = 25 as ::core::ffi::c_int;
pub const Ctrl_Z: ::core::ffi::c_int = 26;
pub const Ctrl_BSL: ::core::ffi::c_int = 28 as ::core::ffi::c_int;
pub const Ctrl_RSB: ::core::ffi::c_int = 29 as ::core::ffi::c_int;
pub const Ctrl_HAT: ::core::ffi::c_int = 30;
pub const Ctrl__: ::core::ffi::c_int = 31;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_iswhite_nl_or_nul(mut c: ::core::ffi::c_int) -> bool {
    return ascii_iswhite(c) as ::core::ffi::c_int != 0
        || c == '\n' as ::core::ffi::c_int
        || c == NUL;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isxdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int
        || c >= 'a' as ::core::ffi::c_int && c <= 'f' as ::core::ffi::c_int
        || c >= 'A' as ::core::ffi::c_int && c <= 'F' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isspace(mut c: ::core::ffi::c_int) -> bool {
    return c >= 9 as ::core::ffi::c_int && c <= 13 as ::core::ffi::c_int
        || c == ' ' as ::core::ffi::c_int;
}
pub const FO_RET_COMS: ::core::ffi::c_int = 'r' as ::core::ffi::c_int;
pub const FO_INS_LONG: ::core::ffi::c_int = 'l' as ::core::ffi::c_int;
pub const FO_INS_BLANK: ::core::ffi::c_int = 'b' as ::core::ffi::c_int;
pub const FO_WHITE_PAR: ::core::ffi::c_int = 'w' as ::core::ffi::c_int;
pub const FO_AUTO: ::core::ffi::c_int = 'a' as ::core::ffi::c_int;
pub const CPO_INDENT: ::core::ffi::c_int = 'I' as ::core::ffi::c_int;
pub const CPO_LISTWM: ::core::ffi::c_int = 'L' as ::core::ffi::c_int;
pub const CPO_BACKSPACE: ::core::ffi::c_int = 'v' as ::core::ffi::c_int;
pub const CPO_REPLCNT: ::core::ffi::c_int = 'X' as ::core::ffi::c_int;
pub const COM_MIDDLE: ::core::ffi::c_int = 'm' as ::core::ffi::c_int;
pub const COM_MAX_LEN: ::core::ffi::c_int = 50 as ::core::ffi::c_int;
pub const BS_INDENT: ::core::ffi::c_int = 'i' as ::core::ffi::c_int;
pub const BS_EOL: ::core::ffi::c_int = 'l' as ::core::ffi::c_int;
pub const BS_START: ::core::ffi::c_int = 's' as ::core::ffi::c_int;
pub const BS_NOSTOP: ::core::ffi::c_int = 'p' as ::core::ffi::c_int;
static compl_busy: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static Insstart_textlen: GlobalCell<colnr_T> = GlobalCell::new(0);
static Insstart_blank_vcol: GlobalCell<colnr_T> = GlobalCell::new(0);
static update_Insstart_orig: GlobalCell<bool> = GlobalCell::new(true_0 != 0);
static last_insert: GlobalCell<String_0> = GlobalCell::new(String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
});
static last_insert_skip: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static new_insert_skip: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static did_restart_edit: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static can_cindent: GlobalCell<bool> = GlobalCell::new(false);
static revins_on: GlobalCell<bool> = GlobalCell::new(false);
static revins_chars: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static revins_legal: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static revins_scol: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static ins_need_undo: GlobalCell<bool> = GlobalCell::new(false);
static dont_sync_undo: GlobalCell<TriState> = GlobalCell::new(kFalse);
static o_lnum: GlobalCell<linenr_T> = GlobalCell::new(0 as linenr_T);
static replace_stack: GlobalCell<C2Rust_Unnamed_26> = GlobalCell::new(KV_INITIAL_VALUE);
unsafe extern "C" fn insert_enter(mut s: *mut InsertState) {
    (*s).did_backspace = true_0 != 0;
    (*s).old_topfill = -1 as ::core::ffi::c_int;
    (*s).replaceState = MODE_REPLACE as ::core::ffi::c_int;
    (*s).cmdchar_todo = (*s).cmdchar;
    (*s).ins_just_started = true_0 != 0;
    did_restart_edit.set(restart_edit.get());
    msg_check_for_delay(true_0 != 0);
    update_Insstart_orig.set(true_0 != 0);
    ins_compl_clear();
    if (*s).cmdchar != 'r' as ::core::ffi::c_int && (*s).cmdchar != 'v' as ::core::ffi::c_int {
        let mut save_cursor: pos_T = (*curwin.get()).w_cursor;
        let ptr: *const ::core::ffi::c_char = if (*s).cmdchar == 'R' as ::core::ffi::c_int {
            b"r\0".as_ptr() as *const ::core::ffi::c_char
        } else if (*s).cmdchar == 'V' as ::core::ffi::c_int {
            b"v\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"i\0".as_ptr() as *const ::core::ffi::c_char
        };
        set_vim_var_string(VV_INSERTMODE, ptr, 1 as ptrdiff_t);
        set_vim_var_string(
            VV_CHAR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        ins_apply_autocmds(EVENT_INSERTENTER);
        if need_highlight_changed.get() {
            highlight_changed();
        }
        if !equalpos((*curwin.get()).w_cursor, save_cursor)
            && *get_vim_var_str(VV_CHAR) as ::core::ffi::c_int == NUL
            && save_cursor.lnum <= (*curbuf.get()).b_ml.ml_line_count
        {
            let mut save_state: ::core::ffi::c_int = State.get();
            (*curwin.get()).w_cursor = save_cursor;
            State.set(MODE_INSERT as ::core::ffi::c_int);
            check_cursor_col(curwin.get());
            State.set(save_state);
        }
    }
    if (*where_paste_started.ptr()).lnum != 0 as linenr_T {
        Insstart.set(where_paste_started.get());
    } else {
        Insstart.set((*curwin.get()).w_cursor);
        if (*s).startln != 0 {
            (*Insstart.ptr()).col = 0 as ::core::ffi::c_int as colnr_T;
        }
    }
    Insstart_textlen.set(linetabsize_str(get_cursor_line_ptr()) as colnr_T);
    Insstart_blank_vcol.set(MAXCOL as ::core::ffi::c_int as colnr_T);
    if !did_ai.get() {
        ai_col.set(0 as ::core::ffi::c_int as colnr_T);
    }
    if (*s).cmdchar != NUL && restart_edit.get() == 0 as ::core::ffi::c_int {
        ResetRedobuff();
        AppendNumberToRedobuff((*s).count);
        if (*s).cmdchar == 'V' as ::core::ffi::c_int || (*s).cmdchar == 'v' as ::core::ffi::c_int {
            AppendCharToRedobuff('g' as ::core::ffi::c_int);
            AppendCharToRedobuff(if (*s).cmdchar == 'v' as ::core::ffi::c_int {
                'r' as ::core::ffi::c_int
            } else {
                'R' as ::core::ffi::c_int
            });
        } else {
            AppendCharToRedobuff((*s).cmdchar);
            if (*s).cmdchar == 'g' as ::core::ffi::c_int {
                AppendCharToRedobuff('I' as ::core::ffi::c_int);
            } else if (*s).cmdchar == 'r' as ::core::ffi::c_int {
                (*s).count = 1 as ::core::ffi::c_int;
            }
        }
    }
    if (*s).cmdchar == 'R' as ::core::ffi::c_int {
        State.set(MODE_REPLACE as ::core::ffi::c_int);
    } else if (*s).cmdchar == 'V' as ::core::ffi::c_int || (*s).cmdchar == 'v' as ::core::ffi::c_int
    {
        State.set(MODE_VREPLACE as ::core::ffi::c_int);
        (*s).replaceState = MODE_VREPLACE as ::core::ffi::c_int;
        orig_line_count.set((*curbuf.get()).b_ml.ml_line_count);
        vr_lines_changed.set(1 as ::core::ffi::c_int);
    } else {
        State.set(MODE_INSERT as ::core::ffi::c_int);
    }
    may_trigger_modechanged();
    stop_insert_mode.set(false_0 != 0);
    if gchar_cursor() == TAB || buf_meta_total(curbuf.get(), kMTMetaInline) > 0 as uint32_t {
        (*curwin.get()).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL);
    }
    if (*curbuf.get()).b_p_iminsert == B_IMODE_LMAP as OptInt {
        (*State.ptr()) |= MODE_LANGMAP as ::core::ffi::c_int;
    }
    setmouse();
    clear_showcmd();
    revins_on.set(State.get() == MODE_INSERT as ::core::ffi::c_int && p_ri.get() != 0);
    if revins_on.get() {
        undisplay_dollar();
    }
    revins_chars.set(0 as ::core::ffi::c_int);
    revins_legal.set(0 as ::core::ffi::c_int);
    revins_scol.set(-1 as ::core::ffi::c_int);
    if restart_edit.get() != 0 as ::core::ffi::c_int && stuff_empty() as ::core::ffi::c_int != 0 {
        arrow_used.set((*where_paste_started.ptr()).lnum == 0 as linenr_T);
        restart_edit.set(0 as ::core::ffi::c_int);
        validate_virtcol(curwin.get());
        update_curswant();
        let mut ptr_0: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        if (ins_at_eol.get() as ::core::ffi::c_int != 0
            && (*curwin.get()).w_cursor.lnum == o_lnum.get()
            || (*curwin.get()).w_curswant > (*curwin.get()).w_virtcol)
            && {
                ptr_0 = get_cursor_line_ptr().offset((*curwin.get()).w_cursor.col as isize);
                *ptr_0 as ::core::ffi::c_int != NUL
            }
        {
            if *ptr_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
                (*curwin.get()).w_cursor.col += 1;
            } else {
                (*s).i = utfc_ptr2len(ptr_0);
                if *ptr_0.offset((*s).i as isize) as ::core::ffi::c_int == NUL {
                    (*curwin.get()).w_cursor.col += (*s).i;
                }
            }
        }
        ins_at_eol.set(false_0 != 0);
    } else {
        arrow_used.set(false_0 != 0);
    }
    need_start_insertmode.set(false_0 != 0);
    ins_need_undo.set(true_0 != 0);
    (*where_paste_started.ptr()).lnum = 0 as ::core::ffi::c_int as linenr_T;
    can_cindent.set(true_0 != 0);
    if did_restart_edit.get() == 0 as ::core::ffi::c_int {
        foldOpenCursor();
    }
    (*s).i = 0 as ::core::ffi::c_int;
    if p_smd.get() != 0 && msg_silent.get() == 0 as ::core::ffi::c_int {
        (*s).i = showmode();
    }
    if did_restart_edit.get() == 0 as ::core::ffi::c_int {
        change_warning(
            curbuf.get(),
            if (*s).i == 0 as ::core::ffi::c_int {
                0 as ::core::ffi::c_int
            } else {
                (*s).i + 1 as ::core::ffi::c_int
            },
        );
    }
    ui_cursor_shape();
    do_digraph(-1 as ::core::ffi::c_int);
    let mut inserted: String_0 = get_inserted();
    new_insert_skip.set(inserted.size as ::core::ffi::c_int);
    if !inserted.data.is_null() {
        xfree(inserted.data as *mut ::core::ffi::c_void);
    }
    old_indent.set(0 as ::core::ffi::c_int);
    loop {
        state_enter(&raw mut (*s).state);
        if ins_esc(&raw mut (*s).count, (*s).cmdchar, (*s).nomove) {
            break;
        }
    }
    if ins_at_eol.get() {
        o_lnum.set((*curwin.get()).w_cursor.lnum);
    }
    pum_check_clear();
    foldUpdateAfterInsert();
    if (*s).cmdchar != 'r' as ::core::ffi::c_int
        && (*s).cmdchar != 'v' as ::core::ffi::c_int
        && (*s).c != Ctrl_C
    {
        ins_apply_autocmds(EVENT_INSERTLEAVE);
    }
    did_cursorhold.set(false_0 != 0);
    if !char_avail() && (*curbuf.get()).b_last_changedtick_i == buf_get_changedtick(curbuf.get()) {
        (*curbuf.get()).b_last_changedtick = buf_get_changedtick(curbuf.get());
    }
}
unsafe extern "C" fn insert_check(mut state: *mut VimState) -> ::core::ffi::c_int {
    let mut s: *mut InsertState = state as *mut InsertState;
    if revins_legal.get() == 0 {
        revins_scol.set(-1 as ::core::ffi::c_int);
    } else {
        revins_legal.set(0 as ::core::ffi::c_int);
    }
    if arrow_used.get() {
        (*s).count = 0 as ::core::ffi::c_int;
    }
    if update_Insstart_orig.get() {
        Insstart_orig.set(Insstart.get());
    }
    if !(*curbuf.get()).terminal.is_null() && !stop_insert_mode.get() {
        stop_insert_mode.set(true_0 != 0);
        restart_edit.set('I' as ::core::ffi::c_int);
        stuffcharReadbuff(
            -(253 as ::core::ffi::c_int
                + ((KE_NOP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        );
    }
    if stop_insert_mode.get() as ::core::ffi::c_int != 0 && !ins_compl_active() {
        (*s).count = 0 as ::core::ffi::c_int;
        return 0 as ::core::ffi::c_int;
    }
    if !arrow_used.get() {
        (*curwin.get()).w_set_curswant = true_0;
    }
    if stuff_empty() {
        did_check_timestamps.set(false_0 != 0);
        if need_check_timestamps.get() {
            check_timestamps(false_0);
        }
    }
    msg_scroll.set(false_0);
    if fdo_flags.get() & kOptFdoFlagInsert as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        foldOpenCursor();
    }
    if !char_avail() {
        foldCheckClose();
    }
    if bt_prompt(curbuf.get()) {
        init_prompt((*s).cmdchar_todo);
        (*s).cmdchar_todo = NUL;
    }
    if (*curbuf.get()).b_mod_set as ::core::ffi::c_int != 0
        && (*curwin.get()).w_onebuf_opt.wo_wrap != 0
        && (*curwin.get()).w_onebuf_opt.wo_sms == 0
        && !(*s).did_backspace
        && (*curwin.get()).w_topline == (*s).old_topline
        && (*curwin.get()).w_topfill == (*s).old_topfill
        && (*s).count <= 1 as ::core::ffi::c_int
    {
        (*s).mincol = (*curwin.get()).w_wcol;
        validate_cursor_col(curwin.get());
        if (*curwin.get()).w_wcol
            < (*s).mincol
                - tabstop_at(
                    get_nolist_virtcol(),
                    (*curbuf.get()).b_p_ts,
                    (*curbuf.get()).b_p_vts_array,
                    false_0 != 0,
                )
            && (*curwin.get()).w_wrow as int64_t
                == ((*curwin.get()).w_view_height - 1 as ::core::ffi::c_int) as int64_t
                    - get_scrolloff_value(curwin.get())
            && ((*curwin.get()).w_cursor.lnum != (*curwin.get()).w_topline
                || (*curwin.get()).w_topfill > 0 as ::core::ffi::c_int)
        {
            if (*curwin.get()).w_topfill > 0 as ::core::ffi::c_int {
                (*curwin.get()).w_topfill -= 1;
            } else if hasFolding(
                curwin.get(),
                (*curwin.get()).w_topline,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut (*s).old_topline,
            ) {
                set_topline(curwin.get(), (*s).old_topline + 1 as linenr_T);
            } else {
                set_topline(curwin.get(), (*curwin.get()).w_topline + 1 as linenr_T);
            }
        }
    }
    if (*s).count <= 1 as ::core::ffi::c_int {
        update_topline(curwin.get());
    }
    (*s).did_backspace = false_0 != 0;
    if (*s).count <= 1 as ::core::ffi::c_int {
        validate_cursor(curwin.get());
    }
    ins_redraw(true_0 != 0);
    if (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
        do_check_scrollbind(true_0 != 0);
    }
    if (*curwin.get()).w_onebuf_opt.wo_crb != 0 {
        do_check_cursorbind();
    }
    if (*s).count <= 1 as ::core::ffi::c_int {
        update_curswant();
    }
    (*s).old_topline = (*curwin.get()).w_topline;
    (*s).old_topfill = (*curwin.get()).w_topfill;
    if (*s).c
        != -(253 as ::core::ffi::c_int
            + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        (*s).lastc = (*s).c;
    }
    if dont_sync_undo.get() as ::core::ffi::c_int == kNone as ::core::ffi::c_int {
        dont_sync_undo.set(kTrue);
    } else {
        dont_sync_undo.set(kFalse);
    }
    if (*s).ins_just_started {
        (*s).ins_just_started = false_0 != 0;
        if ins_compl_has_autocomplete() as ::core::ffi::c_int != 0
            && !char_avail()
            && (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
        {
            (*s).c = char_before_cursor();
            if vim_isprintc((*s).c) {
                ins_compl_enable_autocomplete();
                ins_compl_init_get_longest();
                insert_do_complete(s);
                insert_handle_key_post(s);
                return 1 as ::core::ffi::c_int;
            }
        }
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn insert_execute(
    mut state: *mut VimState,
    mut key: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let s: *mut InsertState = state as *mut InsertState;
    if stop_insert_mode.get() {
        if key
            != -(253 as ::core::ffi::c_int
                + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            && key
                != -(253 as ::core::ffi::c_int
                    + ((KE_NOP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            vungetc(key);
        }
        (*s).count = 0 as ::core::ffi::c_int;
        (*s).nomove = true_0 != 0;
        ins_compl_prep(ESC);
        return 0 as ::core::ffi::c_int;
    }
    if key
        == -(253 as ::core::ffi::c_int
            + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || key
            == -(253 as ::core::ffi::c_int
                + ((KE_NOP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        return -1 as ::core::ffi::c_int;
    }
    (*s).c = key;
    if key
        != -(253 as ::core::ffi::c_int
            + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        did_cursorhold.set(true_0 != 0);
    }
    if ins_compl_active() as ::core::ffi::c_int != 0
        && (*curwin.get()).w_cursor.col >= ins_compl_col()
        && ins_compl_has_shown_match() as ::core::ffi::c_int != 0
        && pum_wanted() as ::core::ffi::c_int != 0
    {
        if ((*s).c == K_BS || (*s).c == Ctrl_H)
            && (*curwin.get()).w_cursor.col > ins_compl_col()
            && {
                (*s).c = ins_compl_bs();
                (*s).c == NUL
            }
        {
            return 1 as ::core::ffi::c_int;
        }
        if !ins_compl_used_match() {
            if (*s).c == Ctrl_L
                && (!ctrl_x_mode_line_or_eval()
                    || ins_compl_long_shown_match() as ::core::ffi::c_int != 0)
            {
                ins_compl_addfrommatch();
                return 1 as ::core::ffi::c_int;
            }
            if ins_compl_accept_char((*s).c) {
                let mut str: *mut ::core::ffi::c_char = do_insert_char_pre((*s).c);
                if !str.is_null() {
                    let mut p: *mut ::core::ffi::c_char = str;
                    while *p as ::core::ffi::c_int != NUL {
                        ins_compl_addleader(utf_ptr2char(p));
                        p = p.offset(utfc_ptr2len(p) as isize);
                    }
                    xfree(str as *mut ::core::ffi::c_void);
                } else {
                    ins_compl_addleader((*s).c);
                }
                return 1 as ::core::ffi::c_int;
            }
            if ((*s).c == Ctrl_Y
                || ins_compl_enter_selects() as ::core::ffi::c_int != 0
                    && ((*s).c == CAR || (*s).c == K_KENTER || (*s).c == NL))
                && stop_arrow() == OK
            {
                ins_compl_delete(false_0 != 0);
                if ins_compl_preinsert_longest() as ::core::ffi::c_int != 0
                    && !ins_compl_is_match_selected()
                {
                    ins_compl_insert(false_0 != 0, true_0 != 0);
                    ins_compl_init_get_longest();
                    return 1 as ::core::ffi::c_int;
                } else {
                    ins_compl_insert(false_0 != 0, false_0 != 0);
                }
            } else if ascii_iswhite_nl_or_nul((*s).c) as ::core::ffi::c_int != 0
                && ins_compl_preinsert_effect() as ::core::ffi::c_int != 0
            {
                ins_compl_delete(false_0 != 0);
            }
        }
    }
    ins_compl_init_get_longest();
    if ins_compl_prep((*s).c) {
        return 1 as ::core::ffi::c_int;
    }
    if (*s).c == Ctrl_BSL {
        ins_redraw(false_0 != 0);
        (*no_mapping.ptr()) += 1;
        (*allow_keys.ptr()) += 1;
        (*s).c = plain_vgetc();
        (*no_mapping.ptr()) -= 1;
        (*allow_keys.ptr()) -= 1;
        if (*s).c != Ctrl_N && (*s).c != Ctrl_G && (*s).c != Ctrl_O {
            vungetc((*s).c);
            (*s).c = Ctrl_BSL;
        } else {
            if (*s).c == Ctrl_O {
                ins_ctrl_o();
                ins_at_eol.set(false_0 != 0);
                (*s).nomove = true_0 != 0;
            }
            (*s).count = 0 as ::core::ffi::c_int;
            return 0 as ::core::ffi::c_int;
        }
    }
    if (*s).c
        != -(253 as ::core::ffi::c_int
            + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        (*s).c = do_digraph((*s).c);
    }
    if ((*s).c == Ctrl_V || (*s).c == Ctrl_Q) && ctrl_x_mode_cmdline() as ::core::ffi::c_int != 0 {
        insert_do_complete(s);
        insert_handle_key_post(s);
        return 1 as ::core::ffi::c_int;
    }
    if (*s).c == Ctrl_V || (*s).c == Ctrl_Q {
        ins_ctrl_v();
        (*s).c = Ctrl_V;
        return 1 as ::core::ffi::c_int;
    }
    if cindent_on() as ::core::ffi::c_int != 0 && ctrl_x_mode_none() as ::core::ffi::c_int != 0 {
        (*s).line_is_white = inindent(0 as ::core::ffi::c_int);
        if in_cinkeys((*s).c, '!' as ::core::ffi::c_int, (*s).line_is_white) as ::core::ffi::c_int
            != 0
            && stop_arrow() == OK
        {
            do_c_expr_indent();
            return 1 as ::core::ffi::c_int;
        }
        if can_cindent.get() as ::core::ffi::c_int != 0
            && in_cinkeys((*s).c, '*' as ::core::ffi::c_int, (*s).line_is_white)
                as ::core::ffi::c_int
                != 0
            && stop_arrow() == OK
        {
            do_c_expr_indent();
        }
    }
    if (*curwin.get()).w_onebuf_opt.wo_rl != 0 {
        match (*s).c {
            K_LEFT => {
                (*s).c = K_RIGHT;
            }
            K_S_LEFT => {
                (*s).c = K_S_RIGHT;
            }
            -22013 => {
                (*s).c = -(253 as ::core::ffi::c_int
                    + ((KE_C_RIGHT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
            }
            K_RIGHT => {
                (*s).c = K_LEFT;
            }
            K_S_RIGHT => {
                (*s).c = K_S_LEFT;
            }
            -22269 => {
                (*s).c = -(253 as ::core::ffi::c_int
                    + ((KE_C_LEFT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
            }
            _ => {}
        }
    }
    if ins_start_select((*s).c) {
        return 1 as ::core::ffi::c_int;
    }
    return insert_handle_key(s);
}
unsafe extern "C" fn insert_handle_key(mut s: *mut InsertState) -> ::core::ffi::c_int {
    's_1398: {
        '_normalchar: {
            '_check_pum: {
                'c_31045: {
                    'c_42507: {
                        'c_31081: {
                            'c_31145: {
                                'c_35097: {
                                    match (*s).c {
                                        ESC => {
                                            if echeck_abbr(ESC + ABBR_OFF) {
                                                break 's_1398;
                                            } else {
                                                break 'c_31045;
                                            }
                                        }
                                        Ctrl_C => {
                                            break 'c_31045;
                                        }
                                        Ctrl_O => {
                                            if ctrl_x_mode_omni() {
                                                insert_do_complete(s);
                                                break 's_1398;
                                            } else if echeck_abbr(Ctrl_O + ABBR_OFF) {
                                                break 's_1398;
                                            } else {
                                                ins_ctrl_o();
                                                if get_ve_flags(curwin.get())
                                                    & kOptVeFlagOnemore as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                    != 0
                                                {
                                                    ins_at_eol.set(false_0 != 0);
                                                    (*s).nomove = true_0 != 0;
                                                }
                                                (*s).count = 0 as ::core::ffi::c_int;
                                                return 0 as ::core::ffi::c_int;
                                            }
                                        }
                                        K_INS | K_KINS => {
                                            ins_insert((*s).replaceState);
                                            break 's_1398;
                                        }
                                        K_HELP | K_F1 | K_XF1 => {
                                            stuffcharReadbuff(K_HELP);
                                            return 0 as ::core::ffi::c_int;
                                        }
                                        32 => {
                                            if mod_mask.get() != MOD_MASK_CTRL {
                                                break '_normalchar;
                                            } else {
                                                break 'c_42507;
                                            }
                                        }
                                        K_ZERO | NUL | Ctrl_A => {
                                            break 'c_42507;
                                        }
                                        Ctrl_R => {
                                            if ctrl_x_mode_register() as ::core::ffi::c_int != 0
                                                && !ins_compl_active()
                                            {
                                                insert_do_complete(s);
                                                break 's_1398;
                                            } else {
                                                ins_reg();
                                                auto_format(false_0 != 0, true_0 != 0);
                                                (*s).inserted_space = false_0;
                                                break 's_1398;
                                            }
                                        }
                                        Ctrl_G => {
                                            ins_ctrl_g();
                                            break 's_1398;
                                        }
                                        Ctrl_HAT => {
                                            ins_ctrl_hat();
                                            break 's_1398;
                                        }
                                        Ctrl__ => {
                                            if p_ari.get() == 0 {
                                                break '_normalchar;
                                            } else {
                                                ins_ctrl_();
                                                break 's_1398;
                                            }
                                        }
                                        Ctrl_D => {
                                            if ctrl_x_mode_path_defines() {
                                                insert_do_complete(s);
                                                break 's_1398;
                                            } else {
                                                break 'c_31081;
                                            }
                                        }
                                        Ctrl_T => {
                                            break 'c_31081;
                                        }
                                        K_DEL | K_KDEL => {
                                            ins_del();
                                            auto_format(false_0 != 0, true_0 != 0);
                                            break 's_1398;
                                        }
                                        K_BS | Ctrl_H => {
                                            (*s).did_backspace = ins_bs(
                                                (*s).c,
                                                BACKSPACE_CHAR as ::core::ffi::c_int,
                                                &raw mut (*s).inserted_space,
                                            );
                                            auto_format(false_0 != 0, true_0 != 0);
                                            if (*s).did_backspace {
                                                if ins_compl_has_autocomplete()
                                                    as ::core::ffi::c_int
                                                    != 0
                                                    && !char_avail()
                                                    && (*curwin.get()).w_cursor.col
                                                        > 0 as ::core::ffi::c_int
                                                {
                                                    (*s).c = char_before_cursor();
                                                    if vim_isprintc((*s).c) {
                                                        redraw_later(
                                                            curwin.get(),
                                                            UPD_VALID as ::core::ffi::c_int,
                                                        );
                                                        update_screen();
                                                        ui_flush();
                                                        ins_compl_enable_autocomplete();
                                                        insert_do_complete(s);
                                                    }
                                                }
                                            }
                                            break 's_1398;
                                        }
                                        Ctrl_W => {
                                            if bt_prompt(curbuf.get()) as ::core::ffi::c_int != 0
                                                && mod_mask.get() & MOD_MASK_SHIFT
                                                    == 0 as ::core::ffi::c_int
                                            {
                                                stuffcharReadbuff(Ctrl_W);
                                                restart_edit.set('A' as ::core::ffi::c_int);
                                                (*s).nomove = true_0 != 0;
                                                (*s).count = 0 as ::core::ffi::c_int;
                                                return 0 as ::core::ffi::c_int;
                                            }
                                            (*s).did_backspace = ins_bs(
                                                (*s).c,
                                                BACKSPACE_WORD as ::core::ffi::c_int,
                                                &raw mut (*s).inserted_space,
                                            );
                                            auto_format(false_0 != 0, true_0 != 0);
                                            if (*s).did_backspace {
                                                if ins_compl_has_autocomplete()
                                                    as ::core::ffi::c_int
                                                    != 0
                                                    && !char_avail()
                                                    && (*curwin.get()).w_cursor.col
                                                        > 0 as ::core::ffi::c_int
                                                {
                                                    (*s).c = char_before_cursor();
                                                    if vim_isprintc((*s).c) {
                                                        redraw_later(
                                                            curwin.get(),
                                                            UPD_VALID as ::core::ffi::c_int,
                                                        );
                                                        update_screen();
                                                        ui_flush();
                                                        ins_compl_enable_autocomplete();
                                                        insert_do_complete(s);
                                                    }
                                                }
                                            }
                                            break 's_1398;
                                        }
                                        Ctrl_U => {
                                            if ctrl_x_mode_function() {
                                                insert_do_complete(s);
                                            } else {
                                                (*s).did_backspace = ins_bs(
                                                    (*s).c,
                                                    BACKSPACE_LINE as ::core::ffi::c_int,
                                                    &raw mut (*s).inserted_space,
                                                );
                                                auto_format(false_0 != 0, true_0 != 0);
                                                (*s).inserted_space = false_0;
                                                if (*s).did_backspace {
                                                    if ins_compl_has_autocomplete()
                                                        as ::core::ffi::c_int
                                                        != 0
                                                        && !char_avail()
                                                        && (*curwin.get()).w_cursor.col
                                                            > 0 as ::core::ffi::c_int
                                                    {
                                                        (*s).c = char_before_cursor();
                                                        if vim_isprintc((*s).c) {
                                                            redraw_later(
                                                                curwin.get(),
                                                                UPD_VALID as ::core::ffi::c_int,
                                                            );
                                                            update_screen();
                                                            ui_flush();
                                                            ins_compl_enable_autocomplete();
                                                            insert_do_complete(s);
                                                        }
                                                    }
                                                }
                                            }
                                            break 's_1398;
                                        }
                                        K_LEFTMOUSE | K_LEFTMOUSE_NM | K_LEFTDRAG
                                        | K_LEFTRELEASE | K_LEFTRELEASE_NM | K_MOUSEMOVE
                                        | K_MIDDLEMOUSE | K_MIDDLEDRAG | K_MIDDLERELEASE
                                        | K_RIGHTMOUSE | K_RIGHTDRAG | K_RIGHTRELEASE
                                        | K_X1MOUSE | K_X1DRAG | K_X1RELEASE | K_X2MOUSE
                                        | K_X2DRAG | K_X2RELEASE => {
                                            ins_mouse((*s).c);
                                            break 's_1398;
                                        }
                                        K_MOUSEDOWN => {
                                            ins_mousescroll(MSCR_DOWN as ::core::ffi::c_int);
                                            break 's_1398;
                                        }
                                        K_MOUSEUP => {
                                            ins_mousescroll(MSCR_UP as ::core::ffi::c_int);
                                            break 's_1398;
                                        }
                                        K_MOUSELEFT => {
                                            ins_mousescroll(MSCR_LEFT as ::core::ffi::c_int);
                                            break 's_1398;
                                        }
                                        K_MOUSERIGHT => {
                                            ins_mousescroll(MSCR_RIGHT as ::core::ffi::c_int);
                                            break 's_1398;
                                        }
                                        K_SELECT | -13821 => {
                                            break 's_1398;
                                        }
                                        K_PASTE_START => {
                                            paste_repeat(1 as ::core::ffi::c_int);
                                            break '_check_pum;
                                        }
                                        -26365 => {
                                            state_handle_k_event();
                                            if dont_sync_undo.get() as ::core::ffi::c_int
                                                == kTrue as ::core::ffi::c_int
                                            {
                                                dont_sync_undo.set(kNone);
                                            }
                                            break '_check_pum;
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
                                                        ) -> *mut ::core::ffi::c_char,
                                                ),
                                                NULL,
                                                0 as ::core::ffi::c_int,
                                            );
                                            break '_check_pum;
                                        }
                                        K_LUA => {
                                            map_execute_lua(false_0 != 0, false_0 != 0);
                                            break '_check_pum;
                                        }
                                        K_HOME | K_KHOME | K_S_HOME | -22525 => {
                                            ins_home((*s).c);
                                            break 's_1398;
                                        }
                                        K_END | K_KEND | K_S_END | -22781 => {
                                            ins_end((*s).c);
                                            break 's_1398;
                                        }
                                        K_LEFT => {
                                            if mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL)
                                                != 0
                                            {
                                                ins_s_left();
                                            } else {
                                                ins_left();
                                            }
                                            break 's_1398;
                                        }
                                        K_S_LEFT | -22013 => {
                                            ins_s_left();
                                            break 's_1398;
                                        }
                                        K_RIGHT => {
                                            if mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL)
                                                != 0
                                            {
                                                ins_s_right();
                                            } else {
                                                ins_right();
                                            }
                                            break 's_1398;
                                        }
                                        K_S_RIGHT | -22269 => {
                                            ins_s_right();
                                            break 's_1398;
                                        }
                                        K_UP => {
                                            if pum_visible() {
                                                insert_do_complete(s);
                                            } else if mod_mask.get() & MOD_MASK_SHIFT != 0 {
                                                ins_pageup();
                                            } else {
                                                ins_up(false_0 != 0);
                                            }
                                            break 's_1398;
                                        }
                                        K_S_UP | K_PAGEUP | K_KPAGEUP => {
                                            if pum_visible() {
                                                insert_do_complete(s);
                                            } else {
                                                ins_pageup();
                                            }
                                            break 's_1398;
                                        }
                                        K_DOWN => {
                                            if pum_visible() {
                                                insert_do_complete(s);
                                            } else if mod_mask.get() & MOD_MASK_SHIFT != 0 {
                                                ins_pagedown();
                                            } else {
                                                ins_down(false_0 != 0);
                                            }
                                            break 's_1398;
                                        }
                                        K_S_DOWN | K_PAGEDOWN | K_KPAGEDOWN => {
                                            if pum_visible() {
                                                insert_do_complete(s);
                                            } else {
                                                ins_pagedown();
                                            }
                                            break 's_1398;
                                        }
                                        K_S_TAB => {
                                            (*s).c = TAB;
                                            break 'c_31145;
                                        }
                                        TAB => {
                                            break 'c_31145;
                                        }
                                        K_KENTER => {
                                            (*s).c = CAR;
                                            break 'c_35097;
                                        }
                                        CAR | NL => {
                                            break 'c_35097;
                                        }
                                        Ctrl_K => {
                                            if ctrl_x_mode_dictionary() {
                                                if check_compl_option(true_0 != 0) {
                                                    insert_do_complete(s);
                                                }
                                                break 's_1398;
                                            } else {
                                                (*s).c = ins_digraph();
                                                if (*s).c == NUL {
                                                    break 's_1398;
                                                } else {
                                                    break '_normalchar;
                                                }
                                            }
                                        }
                                        Ctrl_X => {
                                            ins_ctrl_x();
                                            break 's_1398;
                                        }
                                        Ctrl_RSB => {
                                            if !ctrl_x_mode_tags() {
                                                break '_normalchar;
                                            } else {
                                                insert_do_complete(s);
                                                break 's_1398;
                                            }
                                        }
                                        Ctrl_F => {
                                            if !ctrl_x_mode_files() {
                                                break '_normalchar;
                                            } else {
                                                insert_do_complete(s);
                                                break 's_1398;
                                            }
                                        }
                                        115 | Ctrl_S => {
                                            if !ctrl_x_mode_spell() {
                                                break '_normalchar;
                                            } else {
                                                insert_do_complete(s);
                                                break 's_1398;
                                            }
                                        }
                                        Ctrl_L => {
                                            if !ctrl_x_mode_whole_line() {
                                                break '_normalchar;
                                            }
                                        }
                                        Ctrl_P | Ctrl_N => {}
                                        Ctrl_Y | Ctrl_E => {
                                            (*s).c = ins_ctrl_ey((*s).c);
                                            break 's_1398;
                                        }
                                        Ctrl_Z | _ => {
                                            break '_normalchar;
                                        }
                                    }
                                    if *(*curbuf.get()).b_p_cpt as ::core::ffi::c_int == NUL
                                        && (ctrl_x_mode_normal() as ::core::ffi::c_int != 0
                                            || ctrl_x_mode_whole_line() as ::core::ffi::c_int != 0)
                                        && !compl_status_local()
                                    {
                                        break '_normalchar;
                                    } else {
                                        insert_do_complete(s);
                                        break 's_1398;
                                    }
                                }
                                if bt_quickfix(curbuf.get()) as ::core::ffi::c_int != 0
                                    && (*s).c == CAR
                                {
                                    if (*curwin.get()).w_llist_ref.is_null() {
                                        do_cmdline_cmd(
                                            b".cc\0".as_ptr() as *const ::core::ffi::c_char
                                        );
                                    } else {
                                        do_cmdline_cmd(
                                            b".ll\0".as_ptr() as *const ::core::ffi::c_char
                                        );
                                    }
                                    break 's_1398;
                                } else {
                                    if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                        cmdwin_result.set(CAR);
                                        return 0 as ::core::ffi::c_int;
                                    }
                                    if mod_mask.get() & MOD_MASK_SHIFT == 0 as ::core::ffi::c_int
                                        && bt_prompt(curbuf.get()) as ::core::ffi::c_int != 0
                                    {
                                        prompt_invoke_callback();
                                        if !bt_prompt(curbuf.get()) {
                                            return 0 as ::core::ffi::c_int;
                                        }
                                        break 's_1398;
                                    } else {
                                        if !ins_eol((*s).c) {
                                            return 0 as ::core::ffi::c_int;
                                        }
                                        auto_format(false_0 != 0, false_0 != 0);
                                        (*s).inserted_space = false_0;
                                        break 's_1398;
                                    }
                                }
                            }
                            if ctrl_x_mode_path_patterns() {
                                insert_do_complete(s);
                                break 's_1398;
                            } else {
                                (*s).inserted_space = false_0;
                                if ins_tab() {
                                    break '_normalchar;
                                } else {
                                    auto_format(false_0 != 0, true_0 != 0);
                                    break 's_1398;
                                }
                            }
                        }
                        if (*s).c == Ctrl_T && ctrl_x_mode_thesaurus() as ::core::ffi::c_int != 0 {
                            if check_compl_option(false_0 != 0) {
                                insert_do_complete(s);
                            }
                            break 's_1398;
                        } else {
                            ins_shift((*s).c, (*s).lastc);
                            auto_format(false_0 != 0, true_0 != 0);
                            (*s).inserted_space = false_0;
                            break 's_1398;
                        }
                    }
                    if stuff_inserted(
                        NUL,
                        1 as ::core::ffi::c_int,
                        ((*s).c == Ctrl_A) as ::core::ffi::c_int,
                    ) == FAIL
                        && (*s).c != Ctrl_A
                    {
                        return 0 as ::core::ffi::c_int;
                    }
                    (*s).inserted_space = false_0;
                    break 's_1398;
                }
                if (*s).c == Ctrl_C && cmdwin_type.get() != 0 as ::core::ffi::c_int {
                    cmdwin_result.set(
                        -(253 as ::core::ffi::c_int
                            + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
                    );
                    got_int.set(false_0 != 0);
                    (*s).nomove = true_0 != 0;
                    return 0 as ::core::ffi::c_int;
                }
                if (*s).c == Ctrl_C && bt_prompt(curbuf.get()) as ::core::ffi::c_int != 0 {
                    if invoke_prompt_interrupt() {
                        if !bt_prompt(curbuf.get()) {
                            return 0 as ::core::ffi::c_int;
                        }
                        break 's_1398;
                    }
                }
                return 0 as ::core::ffi::c_int;
            }
            if (*pum_want.ptr()).active {
                if pum_visible() {
                    edit_submode_extra.set(::core::ptr::null_mut::<::core::ffi::c_char>());
                    insert_do_complete(s);
                    if (*pum_want.ptr()).finish {
                        ins_compl_prep(Ctrl_Y);
                    }
                }
                (*pum_want.ptr()).active = false_0 != 0;
            }
            if (*curbuf.get()).b_u_synced {
                ins_need_undo.set(true_0 != 0);
            }
            break 's_1398;
        }
        if p_paste.get() == 0 {
            let mut str: *mut ::core::ffi::c_char = do_insert_char_pre((*s).c);
            if !str.is_null() {
                if *str as ::core::ffi::c_int != NUL && stop_arrow() != FAIL {
                    let mut p: *mut ::core::ffi::c_char = str;
                    while *p as ::core::ffi::c_int != NUL {
                        (*s).c = utf_ptr2char(p);
                        if (*s).c == CAR || (*s).c == K_KENTER || (*s).c == NL {
                            ins_eol((*s).c);
                        } else {
                            ins_char((*s).c);
                        }
                        p = p.offset(utfc_ptr2len(p) as isize);
                    }
                    AppendToRedobuffLit(str, -1 as ::core::ffi::c_int);
                }
                xfree(str as *mut ::core::ffi::c_void);
                (*s).c = NUL;
            }
            if (*s).c == NUL {
                break 's_1398;
            }
        }
        ins_try_si((*s).c);
        if (*s).c == ' ' as ::core::ffi::c_int {
            (*s).inserted_space = true_0;
            if inindent(0 as ::core::ffi::c_int) {
                can_cindent.set(false_0 != 0);
            }
            if Insstart_blank_vcol.get() == MAXCOL as ::core::ffi::c_int
                && (*curwin.get()).w_cursor.lnum == (*Insstart.ptr()).lnum
            {
                Insstart_blank_vcol.set(get_nolist_virtcol());
            }
        }
        if vim_iswordc((*s).c) as ::core::ffi::c_int != 0
            || !echeck_abbr(if (*s).c >= 0x100 as ::core::ffi::c_int {
                (*s).c + ABBR_OFF
            } else {
                (*s).c
            }) && (*s).c != Ctrl_RSB
        {
            insert_special((*s).c, false_0, false_0);
            (*revins_legal.ptr()) += 1;
            (*revins_chars.ptr()) += 1;
        }
        auto_format(false_0 != 0, true_0 != 0);
        foldOpenCursor();
        if ins_compl_has_autocomplete() as ::core::ffi::c_int != 0
            && !char_avail()
            && vim_isprintc((*s).c) as ::core::ffi::c_int != 0
        {
            redraw_later(curwin.get(), UPD_VALID as ::core::ffi::c_int);
            update_screen();
            ui_flush();
            ins_compl_enable_autocomplete();
            insert_do_complete(s);
        }
    }
    insert_handle_key_post(s);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn insert_do_complete(mut s: *mut InsertState) {
    compl_busy.set(true_0 != 0);
    (*disable_fold_update.ptr()) += 1;
    if ins_complete((*s).c, true_0 != 0) == FAIL {
        compl_status_clear();
    }
    (*disable_fold_update.ptr()) -= 1;
    compl_busy.set(false_0 != 0);
    can_si.set(may_do_si());
}
unsafe extern "C" fn insert_do_cindent(mut s: *mut InsertState) {
    if in_cinkeys((*s).c, ' ' as ::core::ffi::c_int, (*s).line_is_white) {
        if stop_arrow() == OK {
            do_c_expr_indent();
        }
    }
}
unsafe extern "C" fn insert_handle_key_post(mut s: *mut InsertState) {
    if (*s).c
        != -(253 as ::core::ffi::c_int
            + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        && ctrl_x_mode_normal() as ::core::ffi::c_int != 0
    {
        did_cursorhold.set(false_0 != 0);
    }
    if ins_compl_active() as ::core::ffi::c_int != 0 && !ins_compl_win_active(curwin.get()) {
        ins_compl_cancel();
    }
    if arrow_used.get() {
        (*s).inserted_space = false_0;
    }
    if can_cindent.get() as ::core::ffi::c_int != 0
        && cindent_on() as ::core::ffi::c_int != 0
        && ctrl_x_mode_normal() as ::core::ffi::c_int != 0
    {
        insert_do_cindent(s);
    }
}
#[no_mangle]
pub unsafe extern "C" fn edit(
    mut cmdchar: ::core::ffi::c_int,
    mut startln: bool,
    mut count: ::core::ffi::c_int,
) -> bool {
    if !(*curbuf.get()).terminal.is_null() {
        if ex_normal_busy.get() != 0 {
            restart_edit.set('i' as ::core::ffi::c_int);
            force_restart_edit.set(true_0 != 0);
            return false_0 != 0;
        }
        return terminal_enter();
    }
    if sandbox.get() != 0 as ::core::ffi::c_int {
        emsg(gettext(&raw const e_sandbox as *const ::core::ffi::c_char));
        return false_0 != 0;
    }
    if textlock.get() != 0 as ::core::ffi::c_int
        || ins_compl_active() as ::core::ffi::c_int != 0
        || compl_busy.get() as ::core::ffi::c_int != 0
        || pum_visible() as ::core::ffi::c_int != 0
        || expr_map_locked() as ::core::ffi::c_int != 0
    {
        emsg(gettext(&raw const e_textlock as *const ::core::ffi::c_char));
        return false_0 != 0;
    }
    let mut s: [InsertState; 1] = [InsertState {
        state: VimState {
            check: None,
            execute: None,
        },
        ca: ::core::ptr::null_mut::<cmdarg_T>(),
        mincol: 0,
        cmdchar: 0,
        cmdchar_todo: 0,
        ins_just_started: false,
        startln: 0,
        count: 0,
        c: 0,
        lastc: 0,
        i: 0,
        did_backspace: false,
        line_is_white: false,
        old_topline: 0,
        old_topfill: 0,
        inserted_space: 0,
        replaceState: 0,
        did_restart_edit: 0,
        nomove: false,
    }; 1];
    memset(
        &raw mut s as *mut InsertState as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<InsertState>(),
    );
    (*(&raw mut s as *mut InsertState)).state.execute = Some(
        insert_execute
            as unsafe extern "C" fn(*mut VimState, ::core::ffi::c_int) -> ::core::ffi::c_int,
    ) as state_execute_callback;
    (*(&raw mut s as *mut InsertState)).state.check =
        Some(insert_check as unsafe extern "C" fn(*mut VimState) -> ::core::ffi::c_int)
            as state_check_callback;
    (*(&raw mut s as *mut InsertState)).cmdchar = cmdchar;
    (*(&raw mut s as *mut InsertState)).startln = startln as ::core::ffi::c_int;
    (*(&raw mut s as *mut InsertState)).count = count;
    insert_enter(&raw mut s as *mut InsertState);
    return (*(&raw mut s as *mut InsertState)).c == Ctrl_O;
}
#[no_mangle]
pub unsafe extern "C" fn ins_need_undo_get() -> bool {
    return ins_need_undo.get();
}
#[no_mangle]
pub unsafe extern "C" fn ins_redraw(mut ready: bool) {
    if char_avail() {
        return;
    }
    if ready as ::core::ffi::c_int != 0
        && has_event(EVENT_CURSORMOVEDI) as ::core::ffi::c_int != 0
        && (last_cursormoved_win.get() != curwin.get()
            || !equalpos(last_cursormoved.get(), (*curwin.get()).w_cursor))
        && !pum_visible()
    {
        if syntax_present(curwin.get()) as ::core::ffi::c_int != 0 && must_redraw.get() != 0 {
            update_screen();
        }
        update_curswant();
        ins_apply_autocmds(EVENT_CURSORMOVEDI);
        last_cursormoved_win.set(curwin.get());
        last_cursormoved.set((*curwin.get()).w_cursor);
    }
    if ready as ::core::ffi::c_int != 0
        && has_event(EVENT_TEXTCHANGEDI) as ::core::ffi::c_int != 0
        && (*curbuf.get()).b_last_changedtick_i != buf_get_changedtick(curbuf.get())
        && !pum_visible()
    {
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
        let mut tick: varnumber_T = buf_get_changedtick(curbuf.get());
        aucmd_prepbuf(&raw mut aco, curbuf.get());
        apply_autocmds(
            EVENT_TEXTCHANGEDI,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        aucmd_restbuf(&raw mut aco);
        (*curbuf.get()).b_last_changedtick_i = buf_get_changedtick(curbuf.get());
        if tick != buf_get_changedtick(curbuf.get()) {
            u_save(
                (*curwin.get()).w_cursor.lnum,
                (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
            );
        }
    }
    if ready as ::core::ffi::c_int != 0
        && has_event(EVENT_TEXTCHANGEDP) as ::core::ffi::c_int != 0
        && (*curbuf.get()).b_last_changedtick_pum != buf_get_changedtick(curbuf.get())
        && pum_visible() as ::core::ffi::c_int != 0
    {
        let mut aco_0: aco_save_T = aco_save_T {
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
        let mut tick_0: varnumber_T = buf_get_changedtick(curbuf.get());
        aucmd_prepbuf(&raw mut aco_0, curbuf.get());
        apply_autocmds(
            EVENT_TEXTCHANGEDP,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        aucmd_restbuf(&raw mut aco_0);
        (*curbuf.get()).b_last_changedtick_pum = buf_get_changedtick(curbuf.get());
        if tick_0 != buf_get_changedtick(curbuf.get()) {
            u_save(
                (*curwin.get()).w_cursor.lnum,
                (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
            );
        }
    }
    if ready {
        may_trigger_win_scrolled_resized();
    }
    if ready as ::core::ffi::c_int != 0
        && has_event(EVENT_BUFMODIFIEDSET) as ::core::ffi::c_int != 0
        && (*curbuf.get()).b_changed_invalid as ::core::ffi::c_int == true_0
        && !pum_visible()
    {
        apply_autocmds(
            EVENT_BUFMODIFIEDSET,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        (*curbuf.get()).b_changed_invalid = false_0 != 0;
    }
    may_trigger_safestate(
        ready as ::core::ffi::c_int != 0 && !ins_compl_active() && !pum_visible(),
    );
    pum_check_clear();
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
    emsg_on_display.set(false_0 != 0);
}
unsafe extern "C" fn ins_ctrl_v() {
    let mut did_putchar: bool = false_0 != 0;
    ins_redraw(false_0 != 0);
    if redrawing() as ::core::ffi::c_int != 0 && !char_avail() {
        edit_putchar('^' as ::core::ffi::c_int, true_0 != 0);
        did_putchar = true_0 != 0;
    }
    AppendToRedobuff(CTRL_V_STR.as_ptr());
    add_to_showcmd_c(Ctrl_V);
    let mut c: ::core::ffi::c_int = get_literal(mod_mask.get() & MOD_MASK_SHIFT != 0);
    if did_putchar {
        edit_unputchar();
    }
    clear_showcmd();
    insert_special(c, true_0, true_0);
    (*revins_chars.ptr()) += 1;
    (*revins_legal.ptr()) += 1;
}
static pc_status: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub const PC_STATUS_UNSET: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PC_STATUS_RIGHT: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PC_STATUS_LEFT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const PC_STATUS_SET: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
static pc_schar: GlobalCell<schar_T> = GlobalCell::new(0);
static pc_attr: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static pc_row: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static pc_col: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
#[no_mangle]
pub unsafe extern "C" fn edit_putchar(mut c: ::core::ffi::c_int, mut highlight: bool) {
    if (*curwin.get()).w_grid_alloc.chars.is_null() && (*default_grid.ptr()).chars.is_null() {
        return;
    }
    let mut attr: ::core::ffi::c_int = 0;
    update_topline(curwin.get());
    validate_cursor(curwin.get());
    if highlight {
        attr = *(*hl_attr_active.ptr()).offset(HLF_8 as ::core::ffi::c_int as isize);
    } else {
        attr = 0 as ::core::ffi::c_int;
    }
    pc_row.set((*curwin.get()).w_wrow);
    pc_status.set(PC_STATUS_UNSET);
    grid_line_start(&raw mut (*curwin.get()).w_grid, pc_row.get());
    if (*curwin.get()).w_onebuf_opt.wo_rl != 0 {
        pc_col.set((*curwin.get()).w_view_width - 1 as ::core::ffi::c_int - (*curwin.get()).w_wcol);
        if grid_line_getchar(pc_col.get(), ::core::ptr::null_mut::<::core::ffi::c_int>())
            == NUL as schar_T
        {
            grid_line_put_schar(
                pc_col.get() - 1 as ::core::ffi::c_int,
                ' ' as ::core::ffi::c_int as schar_T,
                attr,
            );
            (*curwin.get()).w_wcol -= 1;
            pc_status.set(PC_STATUS_RIGHT);
        }
    } else {
        pc_col.set((*curwin.get()).w_wcol);
        if grid_line_getchar(
            pc_col.get() + 1 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ) == NUL as schar_T
        {
            pc_status.set(PC_STATUS_LEFT);
        }
    }
    if pc_status.get() == PC_STATUS_UNSET {
        pc_schar.set(grid_line_getchar(pc_col.get(), pc_attr.ptr()));
        pc_status.set(PC_STATUS_SET);
    }
    let mut buf: [::core::ffi::c_char; 7] = [0; 7];
    grid_line_puts(
        pc_col.get(),
        &raw mut buf as *mut ::core::ffi::c_char,
        utf_char2bytes(c, &raw mut buf as *mut ::core::ffi::c_char),
        attr,
    );
    grid_line_flush();
}
#[no_mangle]
pub unsafe extern "C" fn buf_prompt_text(buf: *const buf_T) -> *mut ::core::ffi::c_char {
    if (*buf).b_prompt_text.is_null() {
        return b"% \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    return (*buf).b_prompt_text;
}
#[no_mangle]
pub unsafe extern "C" fn prompt_text() -> *mut ::core::ffi::c_char {
    return buf_prompt_text(curbuf.get());
}
unsafe extern "C" fn init_prompt(mut cmdchar_todo: ::core::ffi::c_int) {
    let mut prompt: *mut ::core::ffi::c_char = prompt_text();
    let mut prompt_len: ::core::ffi::c_int = strlen(prompt) as ::core::ffi::c_int;
    if (*curbuf.get()).b_prompt_start.mark.lnum < 1 as linenr_T
        || (*curbuf.get()).b_prompt_start.mark.lnum > (*curbuf.get()).b_ml.ml_line_count
    {
        (*curbuf.get()).b_prompt_start.mark.lnum = if 1 as linenr_T
            > (if (*curbuf.get()).b_prompt_start.mark.lnum < (*curbuf.get()).b_ml.ml_line_count {
                (*curbuf.get()).b_prompt_start.mark.lnum
            } else {
                (*curbuf.get()).b_ml.ml_line_count
            }) {
            1 as linenr_T
        } else if (*curbuf.get()).b_prompt_start.mark.lnum < (*curbuf.get()).b_ml.ml_line_count {
            (*curbuf.get()).b_prompt_start.mark.lnum
        } else {
            (*curbuf.get()).b_ml.ml_line_count
        };
        (*curbuf.get()).b_prompt_append_new_line = true_0 != 0;
    }
    (*curwin.get()).w_cursor.lnum =
        if (*curwin.get()).w_cursor.lnum > (*curbuf.get()).b_prompt_start.mark.lnum {
            (*curwin.get()).w_cursor.lnum
        } else {
            (*curbuf.get()).b_prompt_start.mark.lnum
        };
    let mut text: *mut ::core::ffi::c_char = ml_get((*curbuf.get()).b_prompt_start.mark.lnum);
    let mut text_len: colnr_T = ml_get_len((*curbuf.get()).b_prompt_start.mark.lnum);
    if (*curbuf.get()).b_prompt_start.mark.lnum == (*curwin.get()).w_cursor.lnum
        && ((*curbuf.get()).b_prompt_start.mark.col < prompt_len
            || (*curbuf.get()).b_prompt_start.mark.col > text_len
            || !strnequal(
                text.offset((*curbuf.get()).b_prompt_start.mark.col as isize)
                    .offset(-(prompt_len as isize)),
                prompt,
                prompt_len as size_t,
            ))
    {
        if *text as ::core::ffi::c_int == NUL {
            ml_replace(
                (*curbuf.get()).b_prompt_start.mark.lnum,
                prompt,
                true_0 != 0,
            );
            inserted_bytes(
                (*curbuf.get()).b_prompt_start.mark.lnum,
                0 as colnr_T,
                0 as ::core::ffi::c_int,
                prompt_len,
            );
        } else {
            let lnum: linenr_T = (*curbuf.get()).b_ml.ml_line_count;
            ml_append(lnum, prompt, 0 as colnr_T, false_0 != 0);
            appended_lines_mark(lnum, 1 as ::core::ffi::c_int);
            (*curbuf.get()).b_prompt_start.mark.lnum = (*curbuf.get()).b_ml.ml_line_count;
            (*curbuf.get()).b_prompt_append_new_line = true_0 != 0;
            u_clearallandblockfree(curbuf.get());
        }
        (*curbuf.get()).b_prompt_start.mark.col = prompt_len as colnr_T;
        (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
        coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
    }
    if (*Insstart_orig.ptr()).lnum != (*curbuf.get()).b_prompt_start.mark.lnum
        || (*Insstart_orig.ptr()).col != (*curbuf.get()).b_prompt_start.mark.col
    {
        (*Insstart.ptr()).lnum = (*curbuf.get()).b_prompt_start.mark.lnum;
        (*Insstart.ptr()).col = (*curbuf.get()).b_prompt_start.mark.col;
        Insstart_orig.set(Insstart.get());
        Insstart_textlen.set((*Insstart.ptr()).col);
        Insstart_blank_vcol.set(MAXCOL as ::core::ffi::c_int as colnr_T);
        arrow_used.set(false_0 != 0);
    }
    if cmdchar_todo == 'A' as ::core::ffi::c_int {
        coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
    }
    if (*curbuf.get()).b_prompt_start.mark.lnum == (*curwin.get()).w_cursor.lnum {
        (*curwin.get()).w_cursor.col =
            if (*curwin.get()).w_cursor.col > (*curbuf.get()).b_prompt_start.mark.col {
                (*curwin.get()).w_cursor.col
            } else {
                (*curbuf.get()).b_prompt_start.mark.col
            };
    }
    check_cursor(curwin.get());
}
#[no_mangle]
pub unsafe extern "C" fn prompt_curpos_editable() -> bool {
    return (*curwin.get()).w_cursor.lnum > (*curbuf.get()).b_prompt_start.mark.lnum
        || (*curwin.get()).w_cursor.lnum == (*curbuf.get()).b_prompt_start.mark.lnum
            && (*curwin.get()).w_cursor.col >= (*curbuf.get()).b_prompt_start.mark.col;
}
#[no_mangle]
pub unsafe extern "C" fn edit_unputchar() {
    if pc_status.get() != PC_STATUS_UNSET {
        if pc_status.get() == PC_STATUS_RIGHT {
            (*curwin.get()).w_wcol += 1;
        }
        if pc_status.get() == PC_STATUS_RIGHT || pc_status.get() == PC_STATUS_LEFT {
            redrawWinline(curwin.get(), (*curwin.get()).w_cursor.lnum);
        } else {
            grid_line_start(&raw mut (*curwin.get()).w_grid, pc_row.get());
            grid_line_put_schar(pc_col.get(), pc_schar.get(), pc_attr.get());
            grid_line_flush();
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn display_dollar(mut col_arg: colnr_T) {
    let mut col: colnr_T = if col_arg > 0 as ::core::ffi::c_int {
        col_arg
    } else {
        0 as colnr_T
    };
    if !redrawing() {
        return;
    }
    let mut save_col: colnr_T = (*curwin.get()).w_cursor.col;
    (*curwin.get()).w_cursor.col = col;
    let mut p: *mut ::core::ffi::c_char = get_cursor_line_ptr();
    (*curwin.get()).w_cursor.col -= utf_head_off(p, p.offset(col as isize));
    curs_columns(curwin.get(), false_0);
    if (*curwin.get()).w_wcol < (*curwin.get()).w_view_width {
        edit_putchar('$' as ::core::ffi::c_int, false_0 != 0);
        dollar_vcol.set((*curwin.get()).w_virtcol);
    }
    (*curwin.get()).w_cursor.col = save_col;
}
#[no_mangle]
pub unsafe extern "C" fn undisplay_dollar() {
    if dollar_vcol.get() < 0 as ::core::ffi::c_int {
        return;
    }
    dollar_vcol.set(-1 as ::core::ffi::c_int as colnr_T);
    redrawWinline(curwin.get(), (*curwin.get()).w_cursor.lnum);
}
#[no_mangle]
pub unsafe extern "C" fn truncate_spaces(mut line: *mut ::core::ffi::c_char, mut len: size_t) {
    let mut i: ::core::ffi::c_int = 0;
    i = len as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int
        && ascii_iswhite(*line.offset(i as isize) as ::core::ffi::c_int) as ::core::ffi::c_int != 0
    {
        if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0 {
            replace_join(0 as ::core::ffi::c_int);
        }
        i -= 1;
    }
    *line.offset((i + 1 as ::core::ffi::c_int) as isize) = NUL as ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn backspace_until_column(mut col: ::core::ffi::c_int) {
    while (*curwin.get()).w_cursor.col > col {
        (*curwin.get()).w_cursor.col -= 1;
        if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0 {
            replace_do_bs(col);
        } else if !del_char_after_col(col) {
            break;
        }
    }
}
unsafe extern "C" fn del_char_after_col(mut limit_col: ::core::ffi::c_int) -> bool {
    if limit_col >= 0 as ::core::ffi::c_int {
        let mut ecol: colnr_T = (*curwin.get()).w_cursor.col + 1 as colnr_T;
        mb_adjust_cursor();
        while (*curwin.get()).w_cursor.col < limit_col {
            let mut l: ::core::ffi::c_int = utf_ptr2len(get_cursor_pos_ptr());
            if l == 0 as ::core::ffi::c_int {
                break;
            }
            (*curwin.get()).w_cursor.col += l;
        }
        if *get_cursor_pos_ptr() as ::core::ffi::c_int == NUL
            || (*curwin.get()).w_cursor.col == ecol
        {
            return false_0 != 0;
        }
        del_bytes(
            ecol - (*curwin.get()).w_cursor.col,
            false_0 != 0,
            true_0 != 0,
        );
    } else {
        del_char(false_0 != 0);
    }
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn get_literal(mut no_simplify: bool) -> ::core::ffi::c_int {
    let mut nc: ::core::ffi::c_int = 0;
    let mut hex: bool = false_0 != 0;
    let mut octal: bool = false_0 != 0;
    let mut unicode: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if got_int.get() {
        return Ctrl_C;
    }
    (*no_mapping.ptr()) += 1;
    let mut cc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        nc = plain_vgetc();
        if !no_simplify {
            nc = merge_modifiers(nc, mod_mask.ptr());
        }
        if mod_mask.get() & !MOD_MASK_SHIFT != 0 as ::core::ffi::c_int {
            break;
        }
        if State.get() & MODE_CMDLINE as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            && (if nc < 0 as ::core::ffi::c_int || nc > 255 as ::core::ffi::c_int {
                1 as ::core::ffi::c_int
            } else {
                (*utf8len_tab.ptr())[nc as usize] as ::core::ffi::c_int
            }) == 1 as ::core::ffi::c_int
        {
            add_to_showcmd(nc);
        }
        if nc == 'x' as ::core::ffi::c_int || nc == 'X' as ::core::ffi::c_int {
            hex = true_0 != 0;
        } else if nc == 'o' as ::core::ffi::c_int || nc == 'O' as ::core::ffi::c_int {
            octal = true_0 != 0;
        } else if nc == 'u' as ::core::ffi::c_int || nc == 'U' as ::core::ffi::c_int {
            unicode = nc;
        } else {
            if hex as ::core::ffi::c_int != 0 || unicode != 0 as ::core::ffi::c_int {
                if !ascii_isxdigit(nc) {
                    break;
                }
                cc = cc * 16 as ::core::ffi::c_int + hex2nr(nc);
            } else if octal {
                if nc < '0' as ::core::ffi::c_int || nc > '7' as ::core::ffi::c_int {
                    break;
                }
                cc = cc * 8 as ::core::ffi::c_int + nc - '0' as ::core::ffi::c_int;
            } else {
                if !ascii_isdigit(nc) {
                    break;
                }
                cc = cc * 10 as ::core::ffi::c_int + nc - '0' as ::core::ffi::c_int;
            }
            i += 1;
        }
        if cc > 255 as ::core::ffi::c_int && unicode == 0 as ::core::ffi::c_int {
            cc = 255 as ::core::ffi::c_int;
        }
        nc = 0 as ::core::ffi::c_int;
        if hex {
            if i >= 2 as ::core::ffi::c_int {
                break;
            }
        } else if unicode != 0 {
            if unicode == 'u' as ::core::ffi::c_int && i >= 4 as ::core::ffi::c_int
                || unicode == 'U' as ::core::ffi::c_int && i >= 8 as ::core::ffi::c_int
            {
                break;
            }
        } else if i >= 3 as ::core::ffi::c_int {
            break;
        }
    }
    if i == 0 as ::core::ffi::c_int {
        if nc == K_ZERO {
            cc = '\n' as ::core::ffi::c_int;
            nc = 0 as ::core::ffi::c_int;
        } else {
            cc = nc;
            nc = 0 as ::core::ffi::c_int;
        }
    }
    if cc == 0 as ::core::ffi::c_int {
        cc = '\n' as ::core::ffi::c_int;
    }
    (*no_mapping.ptr()) -= 1;
    if nc != 0 {
        vungetc(nc);
        mod_mask.set(0 as ::core::ffi::c_int);
    }
    got_int.set(false_0 != 0);
    return cc;
}
unsafe extern "C" fn insert_special(
    mut c: ::core::ffi::c_int,
    mut allow_modmask: ::core::ffi::c_int,
    mut ctrlv: ::core::ffi::c_int,
) {
    if mod_mask.get() & MOD_MASK_CMD != 0 {
        allow_modmask = true_0;
    }
    if c < 0 as ::core::ffi::c_int || mod_mask.get() != 0 && allow_modmask != 0 {
        let mut p: *mut ::core::ffi::c_char = get_special_key_name(c, mod_mask.get());
        let mut len: ::core::ffi::c_int = strlen(p) as ::core::ffi::c_int;
        c = *p.offset((len - 1 as ::core::ffi::c_int) as isize) as uint8_t as ::core::ffi::c_int;
        if len > 2 as ::core::ffi::c_int {
            if stop_arrow() == FAIL {
                return;
            }
            *p.offset((len - 1 as ::core::ffi::c_int) as isize) = NUL as ::core::ffi::c_char;
            ins_str(p, (len - 1 as ::core::ffi::c_int) as size_t);
            AppendToRedobuffLit(p, -1 as ::core::ffi::c_int);
            ctrlv = false_0;
        }
    }
    if stop_arrow() == OK {
        insertchar(
            c,
            if ctrlv != 0 {
                INSCHAR_CTRLV as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
            -1 as ::core::ffi::c_int,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn insertchar(
    mut c: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut second_indent: ::core::ffi::c_int,
) {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut force_format: ::core::ffi::c_int = flags & INSCHAR_FORMAT as ::core::ffi::c_int;
    let textwidth: ::core::ffi::c_int = comp_textwidth(force_format != 0);
    let fo_ins_blank: bool = has_format_option(FO_INS_BLANK);
    if textwidth > 0 as ::core::ffi::c_int
        && (force_format != 0
            || !ascii_iswhite(c)
                && !(State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0
                    && State.get() & VREPLACE_FLAG as ::core::ffi::c_int == 0
                    && *get_cursor_pos_ptr() as ::core::ffi::c_int != NUL)
                && ((*curwin.get()).w_cursor.lnum != (*Insstart.ptr()).lnum
                    || (!has_format_option(FO_INS_LONG) || Insstart_textlen.get() <= textwidth)
                        && (!fo_ins_blank || Insstart_blank_vcol.get() <= textwidth)))
    {
        let mut do_internal: bool = true_0 != 0;
        let mut virtcol: colnr_T =
            get_nolist_virtcol() + char2cells(if c != NUL { c } else { gchar_cursor() });
        if *(*curbuf.get()).b_p_fex as ::core::ffi::c_int != NUL
            && flags & INSCHAR_NO_FEX as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            && (force_format != 0 || virtcol > textwidth)
        {
            do_internal = fex_format((*curwin.get()).w_cursor.lnum, 1 as ::core::ffi::c_long, c)
                != 0 as ::core::ffi::c_int;
            ins_need_undo.set(true_0 != 0);
        }
        if do_internal {
            internal_format(textwidth, second_indent, flags, c == NUL, c);
        }
    }
    if c == NUL {
        return;
    }
    if did_ai.get() as ::core::ffi::c_int != 0 && c == end_comment_pending.get() {
        let mut lead_end: [::core::ffi::c_char; 50] = [0; 50];
        let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
        let mut i: ::core::ffi::c_int = get_leader_len(line, &raw mut p, false_0 != 0, true_0 != 0);
        if i > 0 as ::core::ffi::c_int && !vim_strchr(p, COM_MIDDLE).is_null() {
            while *p as ::core::ffi::c_int != 0
                && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != ':' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
            let mut middle_len: ::core::ffi::c_int = copy_option_part(
                &raw mut p,
                &raw mut lead_end as *mut ::core::ffi::c_char,
                COM_MAX_LEN as size_t,
                b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ) as ::core::ffi::c_int;
            while middle_len > 0 as ::core::ffi::c_int
                && ascii_iswhite(
                    lead_end[(middle_len - 1 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0
            {
                middle_len -= 1;
            }
            while *p as ::core::ffi::c_int != 0
                && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != ':' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
            let mut end_len: ::core::ffi::c_int = copy_option_part(
                &raw mut p,
                &raw mut lead_end as *mut ::core::ffi::c_char,
                COM_MAX_LEN as size_t,
                b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ) as ::core::ffi::c_int;
            i = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
            loop {
                i -= 1;
                if !(i >= 0 as ::core::ffi::c_int
                    && ascii_iswhite(*line.offset(i as isize) as ::core::ffi::c_int)
                        as ::core::ffi::c_int
                        != 0)
                {
                    break;
                }
            }
            i += 1;
            i -= middle_len;
            if i >= 0 as ::core::ffi::c_int
                && end_len > 0 as ::core::ffi::c_int
                && lead_end[(end_len - 1 as ::core::ffi::c_int) as usize] as uint8_t
                    as ::core::ffi::c_int
                    == end_comment_pending.get()
            {
                backspace_until_column(i);
                ins_bytes_len(
                    &raw mut lead_end as *mut ::core::ffi::c_char,
                    (end_len - 1 as ::core::ffi::c_int) as size_t,
                );
            }
        }
    }
    end_comment_pending.set(NUL);
    did_ai.set(false_0 != 0);
    did_si.set(false_0 != 0);
    can_si.set(false_0 != 0);
    can_si_back.set(false_0 != 0);
    if !(c < ' ' as ::core::ffi::c_int
        || c >= DEL
        || c == '0' as ::core::ffi::c_int
        || c == '^' as ::core::ffi::c_int)
        && utf_char2len(c) == 1 as ::core::ffi::c_int
        && !has_event(EVENT_INSERTCHARPRE)
        && !test_disable_char_avail.get()
        && vpeekc() != NUL
        && State.get() & REPLACE_FLAG as ::core::ffi::c_int == 0
        && !cindent_on()
        && p_ri.get() == 0
    {
        let mut buf: [::core::ffi::c_char; 101] = [0; 101];
        let mut virtcol_0: colnr_T = 0 as colnr_T;
        buf[0 as ::core::ffi::c_int as usize] = c as ::core::ffi::c_char;
        let mut i_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        if textwidth > 0 as ::core::ffi::c_int {
            virtcol_0 = get_nolist_virtcol();
        }
        loop {
            c = vpeekc();
            if !(c != NUL
                && !(c < ' ' as ::core::ffi::c_int
                    || c >= DEL
                    || c == '0' as ::core::ffi::c_int
                    || c == '^' as ::core::ffi::c_int)
                && (*utf8len_tab.ptr())[c as usize] as ::core::ffi::c_int
                    == 1 as ::core::ffi::c_int
                && i_0 < INPUT_BUFLEN
                && (textwidth == 0 as ::core::ffi::c_int || {
                    virtcol_0 += byte2cells(
                        buf[(i_0 - 1 as ::core::ffi::c_int) as usize] as uint8_t
                            as ::core::ffi::c_int,
                    );
                    virtcol_0 < textwidth
                })
                && !(!no_abbr.get()
                    && !vim_iswordc(c)
                    && vim_iswordc(
                        buf[(i_0 - 1 as ::core::ffi::c_int) as usize] as uint8_t
                            as ::core::ffi::c_int,
                    ) as ::core::ffi::c_int
                        != 0))
            {
                break;
            }
            c = vgetc();
            let c2rust_fresh0 = i_0;
            i_0 = i_0 + 1;
            buf[c2rust_fresh0 as usize] = c as ::core::ffi::c_char;
        }
        do_digraph(-1 as ::core::ffi::c_int);
        do_digraph(buf[(i_0 - 1 as ::core::ffi::c_int) as usize] as uint8_t as ::core::ffi::c_int);
        buf[i_0 as usize] = NUL as ::core::ffi::c_char;
        ins_str(&raw mut buf as *mut ::core::ffi::c_char, i_0 as size_t);
        if flags & INSCHAR_CTRLV as ::core::ffi::c_int != 0 {
            redo_literal(
                *(&raw mut buf as *mut ::core::ffi::c_char) as uint8_t as ::core::ffi::c_int,
            );
            i_0 = 1 as ::core::ffi::c_int;
        } else {
            i_0 = 0 as ::core::ffi::c_int;
        }
        if buf[i_0 as usize] as ::core::ffi::c_int != NUL {
            AppendToRedobuffLit(
                (&raw mut buf as *mut ::core::ffi::c_char).offset(i_0 as isize),
                -1 as ::core::ffi::c_int,
            );
        }
    } else {
        let mut cc: ::core::ffi::c_int = 0;
        cc = utf_char2len(c);
        if cc > 1 as ::core::ffi::c_int {
            let mut buf_0: [::core::ffi::c_char; 7] = [0; 7];
            utf_char2bytes(c, &raw mut buf_0 as *mut ::core::ffi::c_char);
            buf_0[cc as usize] = NUL as ::core::ffi::c_char;
            ins_char_bytes(&raw mut buf_0 as *mut ::core::ffi::c_char, cc as size_t);
            AppendCharToRedobuff(c);
        } else {
            ins_char(c);
            if flags & INSCHAR_CTRLV as ::core::ffi::c_int != 0 {
                redo_literal(c);
            } else {
                AppendCharToRedobuff(c);
            }
        }
    };
}
pub const INPUT_BUFLEN: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
unsafe extern "C" fn redo_literal(mut c: ::core::ffi::c_int) {
    let mut buf: [::core::ffi::c_char; 10] = [0; 10];
    if ascii_isdigit(c) {
        vim_snprintf(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>(),
            b"%03d\0".as_ptr() as *const ::core::ffi::c_char,
            c,
        );
        AppendToRedobuff(&raw mut buf as *mut ::core::ffi::c_char);
    } else {
        AppendCharToRedobuff(c);
    };
}
#[no_mangle]
pub unsafe extern "C" fn start_arrow(mut end_insert_pos: *mut pos_T) {
    start_arrow_common(end_insert_pos, true_0 != 0);
}
unsafe extern "C" fn start_arrow_with_change(mut end_insert_pos: *mut pos_T, mut end_change: bool) {
    start_arrow_common(end_insert_pos, end_change);
    if !end_change {
        AppendCharToRedobuff(Ctrl_G);
        AppendCharToRedobuff('U' as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn start_arrow_common(mut end_insert_pos: *mut pos_T, mut end_change: bool) {
    if !arrow_used.get() && end_change as ::core::ffi::c_int != 0 {
        AppendToRedobuff(ESC_STR.as_ptr());
        stop_insert(end_insert_pos, false_0, false_0);
        arrow_used.set(true_0 != 0);
    }
    check_spell_redraw();
}
unsafe extern "C" fn check_spell_redraw() {
    if spell_redraw_lnum.get() != 0 as linenr_T {
        let mut lnum: linenr_T = spell_redraw_lnum.get();
        spell_redraw_lnum.set(0 as ::core::ffi::c_int as linenr_T);
        redrawWinline(curwin.get(), lnum);
    }
}
#[no_mangle]
pub unsafe extern "C" fn stop_arrow() -> ::core::ffi::c_int {
    if arrow_used.get() {
        Insstart.set((*curwin.get()).w_cursor);
        if (*Insstart.ptr()).col > (*Insstart_orig.ptr()).col && !ins_need_undo.get() {
            update_Insstart_orig.set(false_0 != 0);
        }
        Insstart_textlen.set(linetabsize_str(get_cursor_line_ptr()) as colnr_T);
        if u_save_cursor() == OK {
            arrow_used.set(false_0 != 0);
            ins_need_undo.set(false_0 != 0);
        }
        ai_col.set(0 as ::core::ffi::c_int as colnr_T);
        if State.get() & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
            orig_line_count.set((*curbuf.get()).b_ml.ml_line_count);
            vr_lines_changed.set(1 as ::core::ffi::c_int);
        }
        ResetRedobuff();
        AppendToRedobuff(b"1i\0".as_ptr() as *const ::core::ffi::c_char);
        new_insert_skip.set(2 as ::core::ffi::c_int);
    } else if ins_need_undo.get() {
        if u_save_cursor() == OK {
            ins_need_undo.set(false_0 != 0);
        }
    }
    foldOpenCursor();
    return if arrow_used.get() as ::core::ffi::c_int != 0
        || ins_need_undo.get() as ::core::ffi::c_int != 0
    {
        FAIL
    } else {
        OK
    };
}
unsafe extern "C" fn stop_insert(
    mut end_insert_pos: *mut pos_T,
    mut esc: ::core::ffi::c_int,
    mut nomove: ::core::ffi::c_int,
) {
    stop_redo_ins();
    xfree((*replace_stack.ptr()).items as *mut ::core::ffi::c_void);
    (*replace_stack.ptr()).capacity = 0 as size_t;
    (*replace_stack.ptr()).size = (*replace_stack.ptr()).capacity;
    (*replace_stack.ptr()).items = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut inserted: String_0 = get_inserted();
    let mut added: ::core::ffi::c_int = if inserted.data.is_null() {
        0 as ::core::ffi::c_int
    } else {
        inserted.size as ::core::ffi::c_int - new_insert_skip.get()
    };
    if did_restart_edit.get() == 0 as ::core::ffi::c_int || added > 0 as ::core::ffi::c_int {
        xfree((*last_insert.ptr()).data as *mut ::core::ffi::c_void);
        last_insert.set(inserted);
        last_insert_skip.set(if added < 0 as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            new_insert_skip.get()
        });
    } else {
        xfree(inserted.data as *mut ::core::ffi::c_void);
    }
    if !arrow_used.get() && !end_insert_pos.is_null() {
        let mut cc: ::core::ffi::c_int = 0;
        if !ins_need_undo.get() && has_format_option(FO_AUTO) as ::core::ffi::c_int != 0 {
            let mut tpos: pos_T = (*curwin.get()).w_cursor;
            cc = 'x' as ::core::ffi::c_int;
            if (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int && gchar_cursor() == NUL {
                dec_cursor();
                cc = gchar_cursor();
                if !ascii_iswhite(cc) {
                    (*curwin.get()).w_cursor = tpos;
                }
            }
            auto_format(true_0 != 0, false_0 != 0);
            if ascii_iswhite(cc) {
                if gchar_cursor() != NUL {
                    inc_cursor();
                }
                if gchar_cursor() == NUL
                    && (*curwin.get()).w_cursor.lnum == tpos.lnum
                    && (*curwin.get()).w_cursor.col == tpos.col
                {
                    (*curwin.get()).w_cursor.coladd = tpos.coladd;
                }
            }
        }
        check_auto_format(true_0 != 0);
        if nomove == 0
            && did_ai.get() as ::core::ffi::c_int != 0
            && (esc != 0
                || vim_strchr(p_cpo.get(), CPO_INDENT).is_null()
                    && (*curwin.get()).w_cursor.lnum != (*end_insert_pos).lnum)
            && (*end_insert_pos).lnum <= (*curbuf.get()).b_ml.ml_line_count
        {
            let mut tpos_0: pos_T = (*curwin.get()).w_cursor;
            let mut prev_col: colnr_T = (*end_insert_pos).col;
            (*curwin.get()).w_cursor = *end_insert_pos;
            check_cursor_col(curwin.get());
            loop {
                if gchar_cursor() == NUL && (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int {
                    (*curwin.get()).w_cursor.col -= 1;
                }
                cc = gchar_cursor();
                if !ascii_iswhite(cc) {
                    break;
                }
                if del_char(true_0 != 0) == FAIL {
                    break;
                }
            }
            if (*curwin.get()).w_cursor.lnum != tpos_0.lnum {
                (*curwin.get()).w_cursor = tpos_0;
            } else if (*curwin.get()).w_cursor.col < prev_col {
                tpos_0 = (*curwin.get()).w_cursor;
                tpos_0.col += 1;
                if cc != NUL && gchar_pos(&raw mut tpos_0) == NUL {
                    (*curwin.get()).w_cursor.col += 1;
                }
            }
            if VIsual_active.get() {
                check_visual_pos();
            }
        }
    }
    did_ai.set(false_0 != 0);
    did_si.set(false_0 != 0);
    can_si.set(false_0 != 0);
    can_si_back.set(false_0 != 0);
    if !end_insert_pos.is_null() {
        (*curbuf.get()).b_op_start = Insstart.get();
        (*curbuf.get()).b_op_start_orig = Insstart_orig.get();
        (*curbuf.get()).b_op_end = *end_insert_pos;
    }
}
#[no_mangle]
pub unsafe extern "C" fn set_last_insert(mut c: ::core::ffi::c_int) {
    xfree((*last_insert.ptr()).data as *mut ::core::ffi::c_void);
    (*last_insert.ptr()).data = xmalloc(
        (MB_MAXBYTES as ::core::ffi::c_int * 3 as ::core::ffi::c_int + 5 as ::core::ffi::c_int)
            as size_t,
    ) as *mut ::core::ffi::c_char;
    let mut s: *mut ::core::ffi::c_char = (*last_insert.ptr()).data;
    if c < ' ' as ::core::ffi::c_int || c == DEL {
        let c2rust_fresh5 = s;
        s = s.offset(1);
        *c2rust_fresh5 = Ctrl_V as ::core::ffi::c_char;
    }
    s = add_char2buf(c, s);
    let c2rust_fresh6 = s;
    s = s.offset(1);
    *c2rust_fresh6 = ESC as ::core::ffi::c_char;
    *s = NUL as ::core::ffi::c_char;
    (*last_insert.ptr()).size = s.offset_from((*last_insert.ptr()).data) as size_t;
    last_insert_skip.set(0 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn beginline(mut flags: ::core::ffi::c_int) {
    if flags & BL_SOL as ::core::ffi::c_int != 0 && p_sol.get() == 0 {
        coladvance(curwin.get(), (*curwin.get()).w_curswant);
    } else {
        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
        if flags & (BL_WHITE as ::core::ffi::c_int | BL_SOL as ::core::ffi::c_int) != 0 {
            let mut ptr: *mut ::core::ffi::c_char = get_cursor_line_ptr();
            while ascii_iswhite(*ptr as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                && !(flags & BL_FIX as ::core::ffi::c_int != 0
                    && *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL)
            {
                (*curwin.get()).w_cursor.col += 1;
                ptr = ptr.offset(1);
            }
        }
        (*curwin.get()).w_set_curswant = true_0;
    }
    adjust_skipcol();
}
#[no_mangle]
pub unsafe extern "C" fn oneright() -> ::core::ffi::c_int {
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if virtual_active(curwin.get()) {
        let mut prevpos: pos_T = (*curwin.get()).w_cursor;
        ptr = get_cursor_pos_ptr();
        coladvance(
            curwin.get(),
            getviscol()
                + (if *ptr as ::core::ffi::c_int != TAB
                    && vim_isprintc(utf_ptr2char(ptr)) as ::core::ffi::c_int != 0
                {
                    ptr2cells(ptr)
                } else {
                    1 as colnr_T
                }),
        );
        (*curwin.get()).w_set_curswant = true_0;
        return if prevpos.col != (*curwin.get()).w_cursor.col
            || prevpos.coladd != (*curwin.get()).w_cursor.coladd
        {
            OK
        } else {
            FAIL
        };
    }
    ptr = get_cursor_pos_ptr();
    if *ptr as ::core::ffi::c_int == NUL {
        return FAIL;
    }
    let mut l: ::core::ffi::c_int = utfc_ptr2len(ptr);
    if *ptr.offset(l as isize) as ::core::ffi::c_int == NUL
        && get_ve_flags(curwin.get())
            & kOptVeFlagOnemore as ::core::ffi::c_int as ::core::ffi::c_uint
            == 0 as ::core::ffi::c_uint
    {
        return FAIL;
    }
    (*curwin.get()).w_cursor.col += l;
    (*curwin.get()).w_set_curswant = true_0;
    adjust_skipcol();
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn oneleft() -> ::core::ffi::c_int {
    if virtual_active(curwin.get()) {
        let mut v: ::core::ffi::c_int = getviscol();
        if v == 0 as ::core::ffi::c_int {
            return FAIL;
        }
        let mut width: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        loop {
            coladvance(curwin.get(), v as colnr_T - width as colnr_T);
            if getviscol() < v {
                break;
            }
            width += 1;
        }
        if (*curwin.get()).w_cursor.coladd == 1 as ::core::ffi::c_int {
            let mut ptr: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
            if *ptr as ::core::ffi::c_int != TAB
                && vim_isprintc(utf_ptr2char(ptr)) as ::core::ffi::c_int != 0
                && ptr2cells(ptr) > 1 as ::core::ffi::c_int
            {
                (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
            }
        }
        (*curwin.get()).w_set_curswant = true_0;
        adjust_skipcol();
        return OK;
    }
    if (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int {
        return FAIL;
    }
    (*curwin.get()).w_set_curswant = true_0;
    (*curwin.get()).w_cursor.col -= 1;
    mb_adjust_cursor();
    adjust_skipcol();
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn cursor_up_inner(
    mut wp: *mut win_T,
    mut n: linenr_T,
    mut skip_conceal: bool,
) {
    let mut lnum: linenr_T = (*wp).w_cursor.lnum;
    if n >= lnum {
        lnum = 1 as ::core::ffi::c_int as linenr_T;
    } else if win_lines_concealed(wp) {
        hasFolding(wp, lnum, &raw mut lnum, ::core::ptr::null_mut::<linenr_T>());
        loop {
            let c2rust_fresh3 = n;
            n = n - 1;
            if c2rust_fresh3 == 0 {
                break;
            }
            lnum -= 1;
            if lnum <= 1 as linenr_T {
                break;
            }
            n = (n as ::core::ffi::c_int
                + (skip_conceal as ::core::ffi::c_int != 0
                    && decor_conceal_line(
                        wp,
                        lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                        true_0 != 0,
                    ) as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int) as linenr_T;
            if n > 0 as linenr_T
                || !(State.get() & MODE_INSERT as ::core::ffi::c_int != 0
                    || fdo_flags.get()
                        & kOptFdoFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0)
            {
                hasFolding(wp, lnum, &raw mut lnum, ::core::ptr::null_mut::<linenr_T>());
            }
        }
        lnum = if lnum > 1 as linenr_T {
            lnum
        } else {
            1 as linenr_T
        };
    } else {
        lnum -= n;
    }
    (*wp).w_cursor.lnum = lnum;
}
#[no_mangle]
pub unsafe extern "C" fn cursor_up(mut n: linenr_T, mut upd_topline: bool) -> ::core::ffi::c_int {
    if n > 0 as linenr_T && (*curwin.get()).w_cursor.lnum <= 1 as linenr_T {
        return FAIL;
    }
    cursor_up_inner(curwin.get(), n, false_0 != 0);
    coladvance(curwin.get(), (*curwin.get()).w_curswant);
    if upd_topline {
        update_topline(curwin.get());
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn cursor_down_inner(
    mut wp: *mut win_T,
    mut n: ::core::ffi::c_int,
    mut skip_conceal: bool,
) {
    let mut lnum: linenr_T = (*wp).w_cursor.lnum;
    let mut line_count: linenr_T = (*(*wp).w_buffer).b_ml.ml_line_count;
    if lnum + n as linenr_T >= line_count {
        lnum = line_count;
    } else if win_lines_concealed(wp) {
        let mut last: linenr_T = 0;
        loop {
            let c2rust_fresh2 = n;
            n = n - 1;
            if c2rust_fresh2 == 0 {
                break;
            }
            if hasFoldingWin(
                wp,
                lnum,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut last,
                true_0 != 0,
                ::core::ptr::null_mut::<foldinfo_T>(),
            ) {
                lnum = last + 1 as linenr_T;
            } else {
                lnum += 1;
            }
            if lnum >= line_count {
                break;
            }
            n += (skip_conceal as ::core::ffi::c_int != 0
                && decor_conceal_line(
                    wp,
                    lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    true_0 != 0,
                ) as ::core::ffi::c_int
                    != 0) as ::core::ffi::c_int;
        }
        lnum = if lnum < line_count { lnum } else { line_count };
    } else {
        lnum += n as linenr_T;
    }
    (*wp).w_cursor.lnum = lnum;
}
#[no_mangle]
pub unsafe extern "C" fn cursor_down(
    mut n: ::core::ffi::c_int,
    mut upd_topline: bool,
) -> ::core::ffi::c_int {
    let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
    hasFoldingWin(
        curwin.get(),
        lnum,
        ::core::ptr::null_mut::<linenr_T>(),
        &raw mut lnum,
        true_0 != 0,
        ::core::ptr::null_mut::<foldinfo_T>(),
    );
    if n > 0 as ::core::ffi::c_int && lnum >= (*(*curwin.get()).w_buffer).b_ml.ml_line_count {
        return FAIL;
    }
    cursor_down_inner(curwin.get(), n, false_0 != 0);
    coladvance(curwin.get(), (*curwin.get()).w_curswant);
    if upd_topline {
        update_topline(curwin.get());
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn stuff_inserted(
    mut c: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut no_esc: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut last: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
    let mut insert: String_0 = get_last_insert();
    if insert.data.is_null() {
        emsg(gettext(
            &raw const e_noinstext as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    if c != NUL {
        stuffcharReadbuff(c);
    }
    if insert.size > 0 as size_t {
        let mut p: *mut ::core::ffi::c_char = insert
            .data
            .offset(insert.size as isize)
            .offset(-(1 as ::core::ffi::c_int as isize));
        while p >= insert.data {
            if *p as ::core::ffi::c_int == ESC {
                insert.size = p.offset_from(insert.data) as size_t;
                break;
            } else {
                p = p.offset(-1);
            }
        }
    }
    if insert.size > 0 as size_t {
        let mut p_0: *mut ::core::ffi::c_char = insert
            .data
            .offset(insert.size as isize)
            .offset(-(1 as ::core::ffi::c_int as isize));
        if (*p_0 as ::core::ffi::c_int == '0' as ::core::ffi::c_int
            || *p_0 as ::core::ffi::c_int == '^' as ::core::ffi::c_int)
            && (no_esc != 0
                || *insert.data as ::core::ffi::c_int == Ctrl_D && count > 1 as ::core::ffi::c_int)
        {
            last = *p_0;
            insert.size = insert.size.wrapping_sub(1);
        }
    }
    loop {
        stuffReadbuffLen(insert.data, insert.size as ptrdiff_t);
        match last as ::core::ffi::c_int {
            48 => {
                stuffReadbuffLen(
                    b"\x16048\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                        as ptrdiff_t,
                );
            }
            94 => {
                stuffReadbuffLen(
                    b"\x16^\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as usize)
                        as ptrdiff_t,
                );
            }
            _ => {}
        }
        count -= 1;
        if count <= 0 as ::core::ffi::c_int {
            break;
        }
    }
    if no_esc == 0 {
        stuffcharReadbuff(ESC);
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn get_last_insert() -> String_0 {
    return if (*last_insert.ptr()).data.is_null() {
        NULL_STRING
    } else {
        String_0 {
            data: (*last_insert.ptr())
                .data
                .offset(last_insert_skip.get() as isize),
            size: (*last_insert.ptr())
                .size
                .wrapping_sub(last_insert_skip.get() as size_t),
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn get_last_insert_save() -> *mut ::core::ffi::c_char {
    let mut insert: String_0 = get_last_insert();
    if insert.data.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut s: *mut ::core::ffi::c_char =
        xmemdupz(insert.data as *const ::core::ffi::c_void, insert.size)
            as *mut ::core::ffi::c_char;
    if insert.size > 0 as size_t
        && *s.offset(insert.size.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int == ESC
    {
        insert.size = insert.size.wrapping_sub(1);
        *s.offset(insert.size as isize) = NUL as ::core::ffi::c_char;
    }
    return s;
}
unsafe extern "C" fn echeck_abbr(mut c: ::core::ffi::c_int) -> bool {
    if p_paste.get() != 0
        || no_abbr.get() as ::core::ffi::c_int != 0
        || arrow_used.get() as ::core::ffi::c_int != 0
    {
        return false_0 != 0;
    }
    return check_abbr(
        c,
        get_cursor_line_ptr(),
        (*curwin.get()).w_cursor.col as ::core::ffi::c_int,
        if (*curwin.get()).w_cursor.lnum == (*Insstart.ptr()).lnum {
            (*Insstart.ptr()).col as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        },
    );
}
#[no_mangle]
pub unsafe extern "C" fn replace_push(mut str: *mut ::core::ffi::c_char, mut len: size_t) {
    if (*replace_stack.ptr()).size < replace_offset.get() as size_t {
        return;
    }
    if (*replace_stack.ptr()).capacity < (*replace_stack.ptr()).size.wrapping_add(len) {
        (*replace_stack.ptr()).capacity = (*replace_stack.ptr()).size.wrapping_add(len);
        (*replace_stack.ptr()).capacity = (*replace_stack.ptr()).capacity.wrapping_sub(1);
        (*replace_stack.ptr()).capacity |=
            (*replace_stack.ptr()).capacity >> 1 as ::core::ffi::c_int;
        (*replace_stack.ptr()).capacity |=
            (*replace_stack.ptr()).capacity >> 2 as ::core::ffi::c_int;
        (*replace_stack.ptr()).capacity |=
            (*replace_stack.ptr()).capacity >> 4 as ::core::ffi::c_int;
        (*replace_stack.ptr()).capacity |=
            (*replace_stack.ptr()).capacity >> 8 as ::core::ffi::c_int;
        (*replace_stack.ptr()).capacity |=
            (*replace_stack.ptr()).capacity >> 16 as ::core::ffi::c_int;
        (*replace_stack.ptr()).capacity = (*replace_stack.ptr()).capacity.wrapping_add(1);
        (*replace_stack.ptr()).capacity = (*replace_stack.ptr()).capacity;
        (*replace_stack.ptr()).items = xrealloc(
            (*replace_stack.ptr()).items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_char>()
                .wrapping_mul((*replace_stack.ptr()).capacity),
        ) as *mut ::core::ffi::c_char;
    }
    let mut p: *mut ::core::ffi::c_char = (*replace_stack.ptr())
        .items
        .offset((*replace_stack.ptr()).size as isize)
        .offset(-(replace_offset.get() as isize));
    if replace_offset.get() != 0 {
        memmove(
            p.offset(len as isize) as *mut ::core::ffi::c_void,
            p as *const ::core::ffi::c_void,
            replace_offset.get() as size_t,
        );
    }
    memcpy(
        p as *mut ::core::ffi::c_void,
        str as *const ::core::ffi::c_void,
        len,
    );
    (*replace_stack.ptr()).size = (*replace_stack.ptr()).size.wrapping_add(len);
}
#[no_mangle]
pub unsafe extern "C" fn replace_push_nul() {
    replace_push(
        b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        1 as size_t,
    );
}
unsafe extern "C" fn replace_pop_if_nul() -> ::core::ffi::c_int {
    let mut ch: ::core::ffi::c_int = if (*replace_stack.ptr()).size != 0 {
        *(*replace_stack.ptr())
            .items
            .offset((*replace_stack.ptr()).size.wrapping_sub(1 as size_t) as isize)
            as uint8_t as ::core::ffi::c_int
    } else {
        -1 as ::core::ffi::c_int
    };
    if ch == NUL {
        (*replace_stack.ptr()).size = (*replace_stack.ptr()).size.wrapping_sub(1);
    }
    return ch;
}
#[no_mangle]
pub unsafe extern "C" fn replace_join(mut off: ::core::ffi::c_int) {
    let mut i: ssize_t = (*replace_stack.ptr()).size as ssize_t;
    loop {
        i -= 1;
        if i < 0 as ssize_t {
            break;
        }
        if *(*replace_stack.ptr()).items.offset(i as isize) as ::core::ffi::c_int == NUL && {
            let c2rust_fresh1 = off;
            off = off - 1;
            c2rust_fresh1 <= 0 as ::core::ffi::c_int
        } {
            (*replace_stack.ptr()).size = (*replace_stack.ptr()).size.wrapping_sub(1);
            memmove(
                (*replace_stack.ptr()).items.offset(i as isize) as *mut ::core::ffi::c_void,
                (*replace_stack.ptr())
                    .items
                    .offset((i + 1 as ssize_t) as isize)
                    as *const ::core::ffi::c_void,
                (*replace_stack.ptr()).size.wrapping_sub(i as size_t),
            );
            return;
        }
    }
}
unsafe extern "C" fn replace_pop_ins() {
    let mut oldState: ::core::ffi::c_int = State.get();
    State.set(MODE_NORMAL as ::core::ffi::c_int);
    while replace_pop_if_nul() > 0 as ::core::ffi::c_int {
        mb_replace_pop_ins();
        dec_cursor();
    }
    State.set(oldState);
}
unsafe extern "C" fn mb_replace_pop_ins() {
    let mut len: ::core::ffi::c_int = utf_head_off(
        (*replace_stack.ptr())
            .items
            .offset(0 as ::core::ffi::c_int as isize),
        (*replace_stack.ptr())
            .items
            .offset((*replace_stack.ptr()).size.wrapping_sub(1 as size_t) as isize),
    ) + 1 as ::core::ffi::c_int;
    (*replace_stack.ptr()).size = (*replace_stack.ptr()).size.wrapping_sub(len as size_t);
    ins_bytes_len(
        (*replace_stack.ptr())
            .items
            .offset((*replace_stack.ptr()).size as isize),
        len as size_t,
    );
}
unsafe extern "C" fn replace_do_bs(mut limit_col: ::core::ffi::c_int) {
    let mut start_vcol: colnr_T = 0;
    let l_State: ::core::ffi::c_int = State.get();
    let mut cc: ::core::ffi::c_int = replace_pop_if_nul();
    if cc > 0 as ::core::ffi::c_int {
        let mut orig_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut orig_vcols: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if l_State & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
            getvcol(
                curwin.get(),
                &raw mut (*curwin.get()).w_cursor,
                ::core::ptr::null_mut::<colnr_T>(),
                &raw mut start_vcol,
                ::core::ptr::null_mut::<colnr_T>(),
            );
            orig_vcols = win_chartabsize(curwin.get(), get_cursor_pos_ptr(), start_vcol);
        }
        del_char_after_col(limit_col);
        if l_State & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
            orig_len = get_cursor_pos_len() as ::core::ffi::c_int;
        }
        replace_pop_ins();
        if l_State & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
            let mut p: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
            let mut ins_len: ::core::ffi::c_int = get_cursor_pos_len() - orig_len;
            let mut vcol: ::core::ffi::c_int = start_vcol as ::core::ffi::c_int;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < ins_len {
                vcol += win_chartabsize(curwin.get(), p.offset(i as isize), vcol as colnr_T);
                i += utfc_ptr2len(p) - 1 as ::core::ffi::c_int;
                i += 1;
            }
            vcol -= start_vcol as ::core::ffi::c_int;
            (*curwin.get()).w_cursor.col += ins_len;
            while vcol > orig_vcols && gchar_cursor() == ' ' as ::core::ffi::c_int {
                del_char(false_0 != 0);
                orig_vcols += 1;
            }
            (*curwin.get()).w_cursor.col -= ins_len;
        }
        changed_bytes((*curwin.get()).w_cursor.lnum, (*curwin.get()).w_cursor.col);
    } else if cc == 0 as ::core::ffi::c_int {
        del_char_after_col(limit_col);
    }
}
unsafe extern "C" fn ins_reg() {
    let mut need_redraw: bool = false_0 != 0;
    let mut literally: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut vis_active: ::core::ffi::c_int = VIsual_active.get() as ::core::ffi::c_int;
    pc_status.set(PC_STATUS_UNSET);
    if redrawing() as ::core::ffi::c_int != 0 && !char_avail() {
        ins_redraw(false_0 != 0);
        edit_putchar('"' as ::core::ffi::c_int, true_0 != 0);
        add_to_showcmd_c(Ctrl_R);
    }
    (*no_mapping.ptr()) += 1;
    (*allow_keys.ptr()) += 1;
    let mut regname: ::core::ffi::c_int = plain_vgetc();
    if *p_langmap.get() as ::core::ffi::c_int != 0
        && true
        && (p_lrm.get() != 0
            || (if vgetc_busy.get() != 0 {
                (typebuf_maplen() == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
            } else {
                KeyTyped.get() as ::core::ffi::c_int
            }) != 0)
        && KeyStuffed.get() == 0
        && regname >= 0 as ::core::ffi::c_int
    {
        if regname < 256 as ::core::ffi::c_int {
            regname = (*langmap_mapchar.ptr())[regname as usize] as ::core::ffi::c_int;
        } else {
            regname = langmap_adjust_mb(regname);
        }
    }
    if regname == Ctrl_R || regname == Ctrl_O || regname == Ctrl_P {
        literally = regname;
        add_to_showcmd_c(literally);
        regname = plain_vgetc();
        if *p_langmap.get() as ::core::ffi::c_int != 0
            && true
            && (p_lrm.get() != 0
                || (if vgetc_busy.get() != 0 {
                    (typebuf_maplen() == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                } else {
                    KeyTyped.get() as ::core::ffi::c_int
                }) != 0)
            && KeyStuffed.get() == 0
            && regname >= 0 as ::core::ffi::c_int
        {
            if regname < 256 as ::core::ffi::c_int {
                regname = (*langmap_mapchar.ptr())[regname as usize] as ::core::ffi::c_int;
            } else {
                regname = langmap_adjust_mb(regname);
            }
        }
    }
    (*no_mapping.ptr()) -= 1;
    (*allow_keys.ptr()) -= 1;
    (*no_u_sync.ptr()) += 1;
    if regname == '=' as ::core::ffi::c_int {
        let mut curpos: pos_T = (*curwin.get()).w_cursor;
        u_sync_once.set(2 as ::core::ffi::c_int);
        regname = get_expr_register();
        (*curwin.get()).w_cursor = curpos;
        check_cursor(curwin.get());
    }
    if regname == NUL || !valid_yank_reg(regname, false_0 != 0) {
        vim_beep(kOptBoFlagRegister as ::core::ffi::c_int as ::core::ffi::c_uint);
        need_redraw = true_0 != 0;
    } else {
        let mut reg: *mut yankreg_T = get_yank_register(regname, YREG_PASTE as ::core::ffi::c_int);
        if literally == Ctrl_O || literally == Ctrl_P {
            AppendCharToRedobuff(Ctrl_R);
            AppendCharToRedobuff(literally);
            AppendCharToRedobuff(regname);
            do_put(
                regname,
                ::core::ptr::null_mut::<yankreg_T>(),
                BACKWARD as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
                (if literally == Ctrl_P {
                    PUT_FIXINDENT as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) | PUT_CURSEND as ::core::ffi::c_int,
            );
        } else if (*reg).y_size > 1 as size_t
            && is_literal_register(regname) as ::core::ffi::c_int != 0
        {
            AppendCharToRedobuff(Ctrl_R);
            AppendCharToRedobuff(regname);
            do_put(
                regname,
                ::core::ptr::null_mut::<yankreg_T>(),
                BACKWARD as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
                PUT_CURSEND as ::core::ffi::c_int,
            );
        } else if insert_reg(
            regname,
            ::core::ptr::null_mut::<yankreg_T>(),
            literally != 0,
        ) == FAIL
        {
            vim_beep(kOptBoFlagRegister as ::core::ffi::c_int as ::core::ffi::c_uint);
            need_redraw = true_0 != 0;
        } else if stop_insert_mode.get() {
            need_redraw = true_0 != 0;
        }
    }
    (*no_u_sync.ptr()) -= 1;
    if u_sync_once.get() == 1 as ::core::ffi::c_int {
        ins_need_undo.set(true_0 != 0);
    }
    u_sync_once.set(0 as ::core::ffi::c_int);
    if need_redraw as ::core::ffi::c_int != 0 || stuff_empty() as ::core::ffi::c_int != 0 {
        edit_unputchar();
    }
    clear_showcmd();
    if vis_active == 0 && VIsual_active.get() as ::core::ffi::c_int != 0 {
        end_visual_mode();
    }
}
unsafe extern "C" fn ins_ctrl_g() {
    setcursor();
    (*no_mapping.ptr()) += 1;
    (*allow_keys.ptr()) += 1;
    let mut c: ::core::ffi::c_int = plain_vgetc();
    (*no_mapping.ptr()) -= 1;
    (*allow_keys.ptr()) -= 1;
    match c {
        K_UP | Ctrl_K | 107 => {
            ins_up(true_0 != 0);
        }
        K_DOWN | Ctrl_J | 106 => {
            ins_down(true_0 != 0);
        }
        117 => {
            u_sync(true_0 != 0);
            ins_need_undo.set(true_0 != 0);
            update_Insstart_orig.set(false_0 != 0);
            Insstart.set((*curwin.get()).w_cursor);
        }
        85 => {
            dont_sync_undo.set(kNone);
        }
        ESC => {}
        _ => {
            vim_beep(kOptBoFlagCtrlg as ::core::ffi::c_int as ::core::ffi::c_uint);
        }
    };
}
unsafe extern "C" fn ins_ctrl_hat() {
    if map_to_exists_mode(
        b"\0".as_ptr() as *const ::core::ffi::c_char,
        MODE_LANGMAP as ::core::ffi::c_int,
        false_0 != 0,
    ) {
        if State.get() & MODE_LANGMAP as ::core::ffi::c_int != 0 {
            (*curbuf.get()).b_p_iminsert = B_IMODE_NONE as OptInt;
            (*State.ptr()) &= !(MODE_LANGMAP as ::core::ffi::c_int);
        } else {
            (*curbuf.get()).b_p_iminsert = B_IMODE_LMAP as OptInt;
            (*State.ptr()) |= MODE_LANGMAP as ::core::ffi::c_int;
        }
    }
    set_iminsert_global(curbuf.get());
    showmode();
    status_redraw_curbuf();
}
unsafe extern "C" fn ins_esc(
    mut count: *mut ::core::ffi::c_int,
    mut cmdchar: ::core::ffi::c_int,
    mut nomove: bool,
) -> bool {
    static disabled_redraw: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    check_spell_redraw();
    let mut temp: ::core::ffi::c_int = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
    if disabled_redraw.get() {
        (*RedrawingDisabled.ptr()) -= 1;
        disabled_redraw.set(false_0 != 0);
    }
    if !arrow_used.get() {
        if cmdchar != 'r' as ::core::ffi::c_int && cmdchar != 'v' as ::core::ffi::c_int {
            AppendToRedobuff(ESC_STR.as_ptr());
        }
        if *count > 0 as ::core::ffi::c_int {
            line_breakcheck();
            if got_int.get() {
                *count = 0 as ::core::ffi::c_int;
            }
        }
        *count -= 1;
        if *count > 0 as ::core::ffi::c_int {
            if !vim_strchr(p_cpo.get(), CPO_REPLCNT).is_null() {
                (*State.ptr()) &= !(REPLACE_FLAG as ::core::ffi::c_int);
            }
            start_redo_ins();
            if cmdchar == 'r' as ::core::ffi::c_int || cmdchar == 'v' as ::core::ffi::c_int {
                stuffRedoReadbuff(ESC_STR.as_ptr());
            }
            (*RedrawingDisabled.ptr()) += 1;
            disabled_redraw.set(true_0 != 0);
            return false_0 != 0;
        }
        stop_insert(
            &raw mut (*curwin.get()).w_cursor,
            true_0,
            nomove as ::core::ffi::c_int,
        );
        undisplay_dollar();
    }
    if cmdchar != 'r' as ::core::ffi::c_int && cmdchar != 'v' as ::core::ffi::c_int {
        ins_apply_autocmds(EVENT_INSERTLEAVEPRE);
    }
    if restart_edit.get() == NUL && temp == (*curwin.get()).w_cursor.col {
        (*curwin.get()).w_set_curswant = true_0;
    }
    if (*cmdmod.ptr()).cmod_flags & CMOD_KEEPJUMPS as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        let mut view: fmarkv_T = mark_view_make(curwin.get(), (*curwin.get()).w_cursor);
        let fmarkp___: *mut fmark_T = &raw mut (*curbuf.get()).b_last_insert;
        free_fmark(*fmarkp___);
        let fmarkp__: *mut fmark_T = fmarkp___;
        (*fmarkp__).mark = (*curwin.get()).w_cursor;
        (*fmarkp__).fnum = (*curbuf.get()).handle as ::core::ffi::c_int;
        (*fmarkp__).timestamp = os_time();
        (*fmarkp__).view = view;
        (*fmarkp__).additional_data = ::core::ptr::null_mut::<AdditionalData>();
    }
    if !nomove
        && ((*curwin.get()).w_cursor.col != 0 as ::core::ffi::c_int
            || (*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int)
        && (restart_edit.get() == NUL || gchar_cursor() == NUL && !VIsual_active.get())
        && !revins_on.get()
    {
        if (*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int
            || get_ve_flags(curwin.get())
                == kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            oneleft();
            if restart_edit.get() != NUL {
                (*curwin.get()).w_cursor.coladd += 1;
            }
        } else {
            (*curwin.get()).w_cursor.col -= 1;
            (*curwin.get()).w_valid &= !(VALID_WCOL | VALID_VIRTCOL);
            mb_adjust_cursor();
        }
    }
    State.set(MODE_NORMAL as ::core::ffi::c_int);
    may_trigger_modechanged();
    if gchar_cursor() == TAB || buf_meta_total(curbuf.get(), kMTMetaInline) > 0 as uint32_t {
        (*curwin.get()).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL);
    }
    setmouse();
    ui_cursor_shape();
    if reg_recording.get() != 0 as ::core::ffi::c_int || restart_edit.get() != NUL {
        showmode();
    } else if p_smd.get() != 0
        && (got_int.get() as ::core::ffi::c_int != 0 || !skip_showmode())
        && !(p_ch.get() == 0 as OptInt && !ui_has(kUIMessages))
    {
        unshowmode(false_0 != 0);
    }
    return true_0 != 0;
}
unsafe extern "C" fn ins_ctrl_() {
    if revins_on.get() as ::core::ffi::c_int != 0
        && revins_chars.get() != 0
        && revins_scol.get() >= 0 as ::core::ffi::c_int
    {
        while gchar_cursor() != NUL && {
            let c2rust_fresh4 = revins_chars.get();
            revins_chars.set(revins_chars.get() - 1);
            c2rust_fresh4 != 0
        } {
            (*curwin.get()).w_cursor.col += 1;
        }
    }
    p_ri.set((p_ri.get() == 0) as ::core::ffi::c_int);
    revins_on.set(State.get() == MODE_INSERT as ::core::ffi::c_int && p_ri.get() != 0);
    if revins_on.get() {
        revins_scol.set((*curwin.get()).w_cursor.col as ::core::ffi::c_int);
        (*revins_legal.ptr()) += 1;
        revins_chars.set(0 as ::core::ffi::c_int);
        undisplay_dollar();
    } else {
        revins_scol.set(-1 as ::core::ffi::c_int);
    }
    showmode();
}
unsafe extern "C" fn ins_start_select(mut c: ::core::ffi::c_int) -> bool {
    if !km_startsel.get() {
        return false_0 != 0;
    }
    's_78: {
        match c {
            K_KHOME | K_KEND | K_PAGEUP | K_KPAGEUP | K_PAGEDOWN | K_KPAGEDOWN => {
                if mod_mask.get() & MOD_MASK_SHIFT == 0 {
                    break 's_78;
                }
            }
            K_S_LEFT | K_S_RIGHT | K_S_UP | K_S_DOWN | K_S_END | K_S_HOME => {}
            _ => {
                break 's_78;
            }
        }
        start_selection();
        stuffcharReadbuff(Ctrl_O);
        if mod_mask.get() != 0 {
            let buf: [::core::ffi::c_char; 4] = [
                K_SPECIAL as ::core::ffi::c_char,
                KS_MODIFIER as ::core::ffi::c_char,
                mod_mask.get() as uint8_t as ::core::ffi::c_char,
                NUL as ::core::ffi::c_char,
            ];
            stuffReadbuffLen(&raw const buf as *const ::core::ffi::c_char, 3 as ptrdiff_t);
        }
        stuffcharReadbuff(c);
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn ins_insert(mut replaceState: ::core::ffi::c_int) {
    set_vim_var_string(
        VV_INSERTMODE,
        if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0 {
            b"i\0".as_ptr() as *const ::core::ffi::c_char
        } else if replaceState == MODE_VREPLACE as ::core::ffi::c_int {
            b"v\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"r\0".as_ptr() as *const ::core::ffi::c_char
        },
        1 as ptrdiff_t,
    );
    ins_apply_autocmds(EVENT_INSERTCHANGE);
    if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0 {
        State.set(
            MODE_INSERT as ::core::ffi::c_int | State.get() & MODE_LANGMAP as ::core::ffi::c_int,
        );
    } else {
        State.set(replaceState | State.get() & MODE_LANGMAP as ::core::ffi::c_int);
    }
    may_trigger_modechanged();
    AppendCharToRedobuff(K_INS);
    showmode();
    ui_cursor_shape();
}
unsafe extern "C" fn ins_ctrl_o() {
    restart_VIsual_select.set(0 as ::core::ffi::c_int);
    if State.get() & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
        restart_edit.set('V' as ::core::ffi::c_int);
    } else if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0 {
        restart_edit.set('R' as ::core::ffi::c_int);
    } else {
        restart_edit.set('I' as ::core::ffi::c_int);
    }
    if virtual_active(curwin.get()) {
        ins_at_eol.set(false_0 != 0);
    } else {
        ins_at_eol.set(gchar_cursor() == NUL);
    };
}
unsafe extern "C" fn ins_shift(mut c: ::core::ffi::c_int, mut lastc: ::core::ffi::c_int) {
    if stop_arrow() == FAIL {
        return;
    }
    AppendCharToRedobuff(c);
    if c == Ctrl_D
        && (lastc == '0' as ::core::ffi::c_int || lastc == '^' as ::core::ffi::c_int)
        && (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
    {
        (*curwin.get()).w_cursor.col -= 1;
        del_char(false_0 != 0);
        if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0 {
            replace_pop_ins();
        }
        if lastc == '^' as ::core::ffi::c_int {
            old_indent.set(get_indent());
        }
        change_indent(
            INDENT_SET as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            true_0,
            true_0 != 0,
        );
    } else {
        change_indent(
            if c == Ctrl_D {
                INDENT_DEC as ::core::ffi::c_int
            } else {
                INDENT_INC as ::core::ffi::c_int
            },
            0 as ::core::ffi::c_int,
            true_0,
            true_0 != 0,
        );
    }
    if did_ai.get() as ::core::ffi::c_int != 0
        && *skipwhite(get_cursor_line_ptr()) as ::core::ffi::c_int != NUL
    {
        did_ai.set(false_0 != 0);
    }
    did_si.set(false_0 != 0);
    can_si.set(false_0 != 0);
    can_si_back.set(false_0 != 0);
    can_cindent.set(false_0 != 0);
}
unsafe extern "C" fn ins_del() {
    if stop_arrow() == FAIL {
        return;
    }
    if gchar_cursor() == NUL {
        let temp: ::core::ffi::c_int = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
        if !can_bs(BS_EOL)
            || do_join(
                2 as size_t,
                false_0 != 0,
                true_0 != 0,
                false_0 != 0,
                false_0 != 0,
            ) == FAIL
        {
            vim_beep(kOptBoFlagBackspace as ::core::ffi::c_int as ::core::ffi::c_uint);
        } else {
            (*curwin.get()).w_cursor.col = temp as colnr_T;
            if State.get() & VREPLACE_FLAG as ::core::ffi::c_int != 0
                && orig_line_count.get() > (*curbuf.get()).b_ml.ml_line_count
            {
                orig_line_count.set((*curbuf.get()).b_ml.ml_line_count);
            }
        }
    } else if del_char(false_0 != 0) == FAIL {
        vim_beep(kOptBoFlagBackspace as ::core::ffi::c_int as ::core::ffi::c_uint);
    }
    did_ai.set(false_0 != 0);
    did_si.set(false_0 != 0);
    can_si.set(false_0 != 0);
    can_si_back.set(false_0 != 0);
    AppendCharToRedobuff(K_DEL);
}
unsafe extern "C" fn ins_bs(
    mut c: ::core::ffi::c_int,
    mut mode: ::core::ffi::c_int,
    mut inserted_space_p: *mut ::core::ffi::c_int,
) -> bool {
    let mut cc: ::core::ffi::c_int = 0;
    let mut temp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut did_backspace: bool = false_0 != 0;
    let mut call_fix_indent: bool = false_0 != 0;
    if buf_is_empty(curbuf.get()) as ::core::ffi::c_int != 0
        || !revins_on.get()
            && ((*curwin.get()).w_cursor.lnum == 1 as linenr_T
                && (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int
                || !can_bs(BS_START)
                    && (arrow_used.get() as ::core::ffi::c_int != 0 && !bt_prompt(curbuf.get())
                        || (*curwin.get()).w_cursor.lnum == (*Insstart_orig.ptr()).lnum
                            && (*curwin.get()).w_cursor.col <= (*Insstart_orig.ptr()).col)
                || !can_bs(BS_INDENT)
                    && !arrow_used.get()
                    && ai_col.get() > 0 as ::core::ffi::c_int
                    && (*curwin.get()).w_cursor.col <= ai_col.get()
                || !can_bs(BS_EOL) && (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int)
    {
        vim_beep(kOptBoFlagBackspace as ::core::ffi::c_int as ::core::ffi::c_uint);
        return false_0 != 0;
    }
    if stop_arrow() == FAIL {
        return false_0 != 0;
    }
    let mut in_indent: bool = inindent(0 as ::core::ffi::c_int);
    if in_indent {
        can_cindent.set(false_0 != 0);
    }
    end_comment_pending.set(NUL);
    if revins_on.get() {
        inc_cursor();
    }
    if (*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int {
        if mode == BACKSPACE_CHAR as ::core::ffi::c_int {
            (*curwin.get()).w_cursor.coladd -= 1;
            return true_0 != 0;
        }
        if mode == BACKSPACE_WORD as ::core::ffi::c_int {
            (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
            return true_0 != 0;
        }
        (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    }
    if (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int {
        let mut lnum: linenr_T = (*Insstart.ptr()).lnum;
        if (*curwin.get()).w_cursor.lnum == lnum || revins_on.get() as ::core::ffi::c_int != 0 {
            if u_save(
                (*curwin.get()).w_cursor.lnum - 2 as linenr_T,
                (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
            ) == FAIL
            {
                return false_0 != 0;
            }
            (*Insstart.ptr()).lnum -= 1;
            (*Insstart.ptr()).col = ml_get_len((*Insstart.ptr()).lnum);
        }
        cc = -1 as ::core::ffi::c_int;
        if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0 {
            cc = replace_pop_if_nul();
        }
        if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0
            && (*curwin.get()).w_cursor.lnum <= lnum
        {
            dec_cursor();
        } else {
            if State.get() & VREPLACE_FLAG as ::core::ffi::c_int == 0
                || (*curwin.get()).w_cursor.lnum > orig_line_count.get()
            {
                temp = gchar_cursor();
                (*curwin.get()).w_cursor.lnum -= 1;
                if has_format_option(FO_AUTO) as ::core::ffi::c_int != 0
                    && has_format_option(FO_WHITE_PAR) as ::core::ffi::c_int != 0
                {
                    let mut ptr: *const ::core::ffi::c_char =
                        ml_get_buf(curbuf.get(), (*curwin.get()).w_cursor.lnum);
                    let mut len: ::core::ffi::c_int = get_cursor_line_len();
                    if len > 0 as ::core::ffi::c_int
                        && *ptr.offset((len - 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            == ' ' as ::core::ffi::c_int
                    {
                        let mut newp: *mut ::core::ffi::c_char = xmemdupz(
                            ptr as *const ::core::ffi::c_void,
                            (len - 1 as ::core::ffi::c_int) as size_t,
                        )
                            as *mut ::core::ffi::c_char;
                        if (*curbuf.get()).b_ml.ml_flags & (ML_LINE_DIRTY | ML_ALLOCATED) != 0 {
                            xfree((*curbuf.get()).b_ml.ml_line_ptr as *mut ::core::ffi::c_void);
                        }
                        (*curbuf.get()).b_ml.ml_line_ptr = newp;
                        (*curbuf.get()).b_ml.ml_line_textlen -= 1;
                        (*curbuf.get()).b_ml.ml_flags |= ML_LINE_DIRTY;
                    }
                }
                do_join(
                    2 as size_t,
                    false_0 != 0,
                    false_0 != 0,
                    false_0 != 0,
                    false_0 != 0,
                );
                if temp == NUL && gchar_cursor() != NUL {
                    inc_cursor();
                }
            } else {
                dec_cursor();
            }
            if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0 {
                let mut oldState: ::core::ffi::c_int = State.get();
                State.set(MODE_NORMAL as ::core::ffi::c_int);
                while cc > 0 as ::core::ffi::c_int {
                    let mut save_col: colnr_T = (*curwin.get()).w_cursor.col;
                    mb_replace_pop_ins();
                    (*curwin.get()).w_cursor.col = save_col;
                    cc = replace_pop_if_nul();
                }
                replace_pop_ins();
                State.set(oldState);
            }
        }
        did_ai.set(false_0 != 0);
    } else {
        if revins_on.get() {
            dec_cursor();
        }
        let mut mincol: colnr_T = 0 as colnr_T;
        if mode == BACKSPACE_LINE as ::core::ffi::c_int
            && ((*curbuf.get()).b_p_ai != 0 || cindent_on() as ::core::ffi::c_int != 0)
            && !revins_on.get()
        {
            let mut save_col_0: colnr_T = (*curwin.get()).w_cursor.col;
            beginline(BL_WHITE as ::core::ffi::c_int);
            if (*curwin.get()).w_cursor.col < save_col_0 {
                mincol = (*curwin.get()).w_cursor.col;
                call_fix_indent = true_0 != 0;
            }
            (*curwin.get()).w_cursor.col = save_col_0;
        }
        if mode == BACKSPACE_CHAR as ::core::ffi::c_int
            && (p_sta.get() != 0 && in_indent as ::core::ffi::c_int != 0
                || (get_sts_value() != 0 as ::core::ffi::c_int
                    || tabstop_count((*curbuf.get()).b_p_vsts_array) != 0)
                    && (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
                    && (*get_cursor_pos_ptr().offset(-(1 as ::core::ffi::c_int as isize))
                        as ::core::ffi::c_int
                        == TAB
                        || *get_cursor_pos_ptr().offset(-(1 as ::core::ffi::c_int as isize))
                            as ::core::ffi::c_int
                            == ' ' as ::core::ffi::c_int
                            && (*inserted_space_p == 0
                                || arrow_used.get() as ::core::ffi::c_int != 0)))
        {
            *inserted_space_p = false_0;
            let use_ts: bool = (*curwin.get()).w_onebuf_opt.wo_list == 0
                || (*curwin.get()).w_p_lcs_chars.tab1 != 0;
            let line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
            let cursor_ptr: *mut ::core::ffi::c_char =
                line.offset((*curwin.get()).w_cursor.col as isize);
            let mut vcol: colnr_T = 0 as colnr_T;
            let mut space_vcol: colnr_T = 0 as colnr_T;
            let mut sci: StrCharInfo = utf_ptr2StrCharInfo(line);
            let mut space_sci: StrCharInfo = sci;
            let mut prev_space: bool = false_0 != 0;
            while sci.ptr < cursor_ptr {
                let mut cur_space: bool = ascii_iswhite(sci.chr.value as ::core::ffi::c_int);
                if !prev_space && cur_space as ::core::ffi::c_int != 0 {
                    space_sci = sci;
                    space_vcol = vcol;
                }
                vcol += charsize_nowrap(curbuf.get(), sci.ptr, use_ts, vcol, sci.chr.value);
                sci = utfc_next(sci);
                prev_space = cur_space;
            }
            let mut want_vcol: colnr_T = if vcol > 0 as ::core::ffi::c_int {
                vcol - 1 as colnr_T
            } else {
                0 as colnr_T
            };
            if p_sta.get() != 0 && in_indent as ::core::ffi::c_int != 0 {
                want_vcol -= want_vcol as ::core::ffi::c_int % get_sw_value(curbuf.get());
            } else {
                want_vcol =
                    tabstop_start(want_vcol, get_sts_value(), (*curbuf.get()).b_p_vsts_array);
            }
            loop {
                let mut size: ::core::ffi::c_int = charsize_nowrap(
                    curbuf.get(),
                    space_sci.ptr,
                    use_ts,
                    space_vcol,
                    space_sci.chr.value,
                );
                if space_vcol as ::core::ffi::c_int + size > want_vcol {
                    break;
                }
                space_vcol += size;
                space_sci = utfc_next(space_sci);
            }
            let want_col: colnr_T = space_sci.ptr.offset_from(line) as colnr_T;
            while (*curwin.get()).w_cursor.col > want_col {
                dec_cursor();
                if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0 {
                    if (*curwin.get()).w_cursor.lnum != (*Insstart.ptr()).lnum
                        || (*curwin.get()).w_cursor.col >= (*Insstart.ptr()).col
                    {
                        replace_do_bs(-1 as ::core::ffi::c_int);
                    }
                } else {
                    del_char(false_0 != 0);
                }
            }
            while space_vcol < want_vcol {
                if (*curwin.get()).w_cursor.lnum == (*Insstart_orig.ptr()).lnum
                    && (*curwin.get()).w_cursor.col < (*Insstart_orig.ptr()).col
                {
                    (*Insstart_orig.ptr()).col = (*curwin.get()).w_cursor.col;
                }
                if State.get() & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
                    ins_char(' ' as ::core::ffi::c_int);
                } else {
                    ins_str(
                        b" \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                            .wrapping_sub(1 as size_t),
                    );
                    if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0 {
                        replace_push_nul();
                    }
                }
                space_vcol += 1;
            }
        } else {
            let mut cclass: ::core::ffi::c_int = mb_get_class(get_cursor_pos_ptr());
            loop {
                if !revins_on.get() {
                    dec_cursor();
                }
                cc = gchar_cursor();
                let mut prev_cclass: ::core::ffi::c_int = cclass;
                cclass = mb_get_class(get_cursor_pos_ptr());
                if mode == BACKSPACE_WORD as ::core::ffi::c_int && !ascii_isspace(cc) {
                    mode = BACKSPACE_WORD_NOT_SPACE as ::core::ffi::c_int;
                    temp = vim_iswordc(cc) as ::core::ffi::c_int;
                } else if mode == BACKSPACE_WORD_NOT_SPACE as ::core::ffi::c_int
                    && (ascii_isspace(cc) as ::core::ffi::c_int != 0
                        || vim_iswordc(cc) as ::core::ffi::c_int != temp
                        || prev_cclass != cclass)
                {
                    if !revins_on.get() {
                        inc_cursor();
                    } else if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0 {
                        dec_cursor();
                    }
                    break;
                }
                if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0 {
                    replace_do_bs(-1 as ::core::ffi::c_int);
                } else {
                    let mut has_composing: bool = false_0 != 0;
                    if p_deco.get() != 0 {
                        let mut p0: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
                        has_composing = utf_composinglike(
                            p0,
                            p0.offset(utf_ptr2len(p0) as isize),
                            ::core::ptr::null_mut::<GraphemeState>(),
                        );
                    }
                    del_char(false_0 != 0);
                    if has_composing {
                        inc_cursor();
                    }
                    if revins_chars.get() != 0 {
                        (*revins_chars.ptr()) -= 1;
                        (*revins_legal.ptr()) += 1;
                    }
                    if revins_on.get() as ::core::ffi::c_int != 0 && gchar_cursor() == NUL {
                        break;
                    }
                }
                if mode == BACKSPACE_CHAR as ::core::ffi::c_int {
                    break;
                }
                if !(revins_on.get() as ::core::ffi::c_int != 0
                    || (*curwin.get()).w_cursor.col > mincol
                        && (can_bs(BS_NOSTOP) as ::core::ffi::c_int != 0
                            || ((*curwin.get()).w_cursor.lnum != (*Insstart_orig.ptr()).lnum
                                || (*curwin.get()).w_cursor.col != (*Insstart_orig.ptr()).col)))
                {
                    break;
                }
            }
        }
        did_backspace = true_0 != 0;
    }
    did_si.set(false_0 != 0);
    can_si.set(false_0 != 0);
    can_si_back.set(false_0 != 0);
    if (*curwin.get()).w_cursor.col <= 1 as ::core::ffi::c_int {
        did_ai.set(false_0 != 0);
    }
    if call_fix_indent {
        fix_indent();
    }
    AppendCharToRedobuff(c);
    if (*curwin.get()).w_cursor.lnum == (*Insstart_orig.ptr()).lnum
        && (*curwin.get()).w_cursor.col < (*Insstart_orig.ptr()).col
    {
        (*Insstart_orig.ptr()).col = (*curwin.get()).w_cursor.col;
    }
    if !vim_strchr(p_cpo.get(), CPO_BACKSPACE).is_null()
        && dollar_vcol.get() == -1 as ::core::ffi::c_int
    {
        dollar_vcol.set((*curwin.get()).w_virtcol);
    }
    if did_backspace {
        foldOpenCursor();
    }
    return did_backspace;
}
unsafe extern "C" fn ins_left() {
    let end_change: bool =
        dont_sync_undo.get() as ::core::ffi::c_int == kFalse as ::core::ffi::c_int;
    if fdo_flags.get() & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped.get() as ::core::ffi::c_int != 0
    {
        foldOpenCursor();
    }
    undisplay_dollar();
    let mut tpos: pos_T = (*curwin.get()).w_cursor;
    if oneleft() == OK {
        start_arrow_with_change(&raw mut tpos, end_change);
        if !end_change {
            AppendCharToRedobuff(K_LEFT);
        }
        if revins_scol.get() != -1 as ::core::ffi::c_int
            && (*curwin.get()).w_cursor.col >= revins_scol.get()
        {
            (*revins_legal.ptr()) += 1;
        }
        (*revins_chars.ptr()) += 1;
    } else if !vim_strchr(p_ww.get(), '[' as ::core::ffi::c_int).is_null()
        && (*curwin.get()).w_cursor.lnum > 1 as linenr_T
    {
        start_arrow(&raw mut tpos);
        (*curwin.get()).w_cursor.lnum -= 1;
        coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
        (*curwin.get()).w_set_curswant = true_0;
    } else {
        vim_beep(kOptBoFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint);
    }
    dont_sync_undo.set(kFalse);
}
unsafe extern "C" fn ins_home(mut c: ::core::ffi::c_int) {
    if fdo_flags.get() & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped.get() as ::core::ffi::c_int != 0
    {
        foldOpenCursor();
    }
    undisplay_dollar();
    let mut tpos: pos_T = (*curwin.get()).w_cursor;
    if c == -(253 as ::core::ffi::c_int
        + ((KE_C_HOME as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        (*curwin.get()).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
    }
    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    (*curwin.get()).w_curswant = 0 as ::core::ffi::c_int as colnr_T;
    start_arrow(&raw mut tpos);
}
unsafe extern "C" fn ins_end(mut c: ::core::ffi::c_int) {
    if fdo_flags.get() & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped.get() as ::core::ffi::c_int != 0
    {
        foldOpenCursor();
    }
    undisplay_dollar();
    let mut tpos: pos_T = (*curwin.get()).w_cursor;
    if c == -(253 as ::core::ffi::c_int
        + ((KE_C_END as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
    }
    coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
    (*curwin.get()).w_curswant = MAXCOL as ::core::ffi::c_int as colnr_T;
    start_arrow(&raw mut tpos);
}
unsafe extern "C" fn ins_s_left() {
    let end_change: bool =
        dont_sync_undo.get() as ::core::ffi::c_int == kFalse as ::core::ffi::c_int;
    if fdo_flags.get() & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped.get() as ::core::ffi::c_int != 0
    {
        foldOpenCursor();
    }
    undisplay_dollar();
    if (*curwin.get()).w_cursor.lnum > 1 as linenr_T
        || (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
    {
        start_arrow_with_change(&raw mut (*curwin.get()).w_cursor, end_change);
        if !end_change {
            AppendCharToRedobuff(K_S_LEFT);
        }
        bck_word(1 as ::core::ffi::c_int, false_0 != 0, false_0 != 0);
        (*curwin.get()).w_set_curswant = true_0;
    } else {
        vim_beep(kOptBoFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint);
    }
    dont_sync_undo.set(kFalse);
}
unsafe extern "C" fn ins_right() {
    let end_change: bool =
        dont_sync_undo.get() as ::core::ffi::c_int == kFalse as ::core::ffi::c_int;
    if fdo_flags.get() & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped.get() as ::core::ffi::c_int != 0
    {
        foldOpenCursor();
    }
    undisplay_dollar();
    if gchar_cursor() != NUL || virtual_active(curwin.get()) as ::core::ffi::c_int != 0 {
        start_arrow_with_change(&raw mut (*curwin.get()).w_cursor, end_change);
        if !end_change {
            AppendCharToRedobuff(K_RIGHT);
        }
        (*curwin.get()).w_set_curswant = true_0;
        if virtual_active(curwin.get()) {
            oneright();
        } else {
            (*curwin.get()).w_cursor.col += utfc_ptr2len(get_cursor_pos_ptr());
        }
        (*revins_legal.ptr()) += 1;
        if revins_chars.get() != 0 {
            (*revins_chars.ptr()) -= 1;
        }
    } else if !vim_strchr(p_ww.get(), ']' as ::core::ffi::c_int).is_null()
        && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count
    {
        start_arrow(&raw mut (*curwin.get()).w_cursor);
        (*curwin.get()).w_set_curswant = true_0;
        (*curwin.get()).w_cursor.lnum += 1;
        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    } else {
        vim_beep(kOptBoFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint);
    }
    dont_sync_undo.set(kFalse);
}
unsafe extern "C" fn ins_s_right() {
    let end_change: bool =
        dont_sync_undo.get() as ::core::ffi::c_int == kFalse as ::core::ffi::c_int;
    if fdo_flags.get() & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped.get() as ::core::ffi::c_int != 0
    {
        foldOpenCursor();
    }
    undisplay_dollar();
    if (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count || gchar_cursor() != NUL {
        start_arrow_with_change(&raw mut (*curwin.get()).w_cursor, end_change);
        if !end_change {
            AppendCharToRedobuff(K_S_RIGHT);
        }
        fwd_word(1 as ::core::ffi::c_int, false_0 != 0, false);
        (*curwin.get()).w_set_curswant = true_0;
    } else {
        vim_beep(kOptBoFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint);
    }
    dont_sync_undo.set(kFalse);
}
unsafe extern "C" fn ins_up(mut startcol: bool) {
    let mut old_topline: linenr_T = (*curwin.get()).w_topline;
    let mut old_topfill: ::core::ffi::c_int = (*curwin.get()).w_topfill;
    undisplay_dollar();
    let mut tpos: pos_T = (*curwin.get()).w_cursor;
    if cursor_up(1 as linenr_T, true_0 != 0) == OK {
        if startcol {
            coladvance(curwin.get(), getvcol_nolist(Insstart.ptr()));
        }
        if old_topline != (*curwin.get()).w_topline || old_topfill != (*curwin.get()).w_topfill {
            redraw_later(curwin.get(), UPD_VALID as ::core::ffi::c_int);
        }
        start_arrow(&raw mut tpos);
        can_cindent.set(true_0 != 0);
    } else {
        vim_beep(kOptBoFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint);
    };
}
unsafe extern "C" fn ins_pageup() {
    undisplay_dollar();
    if mod_mask.get() & MOD_MASK_CTRL != 0 {
        if !(*first_tabpage.get()).tp_next.is_null() {
            start_arrow(&raw mut (*curwin.get()).w_cursor);
            goto_tabpage(-1 as ::core::ffi::c_int);
        }
        return;
    }
    let mut tpos: pos_T = (*curwin.get()).w_cursor;
    if pagescroll(BACKWARD, 1 as ::core::ffi::c_int, false_0 != 0) == OK {
        start_arrow(&raw mut tpos);
        can_cindent.set(true_0 != 0);
    } else {
        vim_beep(kOptBoFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint);
    };
}
unsafe extern "C" fn ins_down(mut startcol: bool) {
    let mut old_topline: linenr_T = (*curwin.get()).w_topline;
    let mut old_topfill: ::core::ffi::c_int = (*curwin.get()).w_topfill;
    undisplay_dollar();
    let mut tpos: pos_T = (*curwin.get()).w_cursor;
    if cursor_down(1 as ::core::ffi::c_int, true_0 != 0) == OK {
        if startcol {
            coladvance(curwin.get(), getvcol_nolist(Insstart.ptr()));
        }
        if old_topline != (*curwin.get()).w_topline || old_topfill != (*curwin.get()).w_topfill {
            redraw_later(curwin.get(), UPD_VALID as ::core::ffi::c_int);
        }
        start_arrow(&raw mut tpos);
        can_cindent.set(true_0 != 0);
    } else {
        vim_beep(kOptBoFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint);
    };
}
unsafe extern "C" fn ins_pagedown() {
    undisplay_dollar();
    if mod_mask.get() & MOD_MASK_CTRL != 0 {
        if !(*first_tabpage.get()).tp_next.is_null() {
            start_arrow(&raw mut (*curwin.get()).w_cursor);
            goto_tabpage(0 as ::core::ffi::c_int);
        }
        return;
    }
    let mut tpos: pos_T = (*curwin.get()).w_cursor;
    if pagescroll(FORWARD, 1 as ::core::ffi::c_int, false_0 != 0) == OK {
        start_arrow(&raw mut tpos);
        can_cindent.set(true_0 != 0);
    } else {
        vim_beep(kOptBoFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint);
    };
}
unsafe extern "C" fn ins_tab() -> bool {
    let mut temp: ::core::ffi::c_int = 0;
    if Insstart_blank_vcol.get() == MAXCOL as ::core::ffi::c_int
        && (*curwin.get()).w_cursor.lnum == (*Insstart.ptr()).lnum
    {
        Insstart_blank_vcol.set(get_nolist_virtcol());
    }
    if echeck_abbr(TAB + ABBR_OFF) {
        return false_0 != 0;
    }
    let mut ind: bool = inindent(0 as ::core::ffi::c_int);
    if ind {
        can_cindent.set(false_0 != 0);
    }
    if (*curbuf.get()).b_p_et == 0
        && !(p_sta.get() != 0
            && ind as ::core::ffi::c_int != 0
            && (tabstop_count((*curbuf.get()).b_p_vts_array) > 1 as ::core::ffi::c_int
                || tabstop_count((*curbuf.get()).b_p_vts_array) == 1 as ::core::ffi::c_int
                    && tabstop_first((*curbuf.get()).b_p_vts_array) != get_sw_value(curbuf.get())
                || tabstop_count((*curbuf.get()).b_p_vts_array) == 0 as ::core::ffi::c_int
                    && (*curbuf.get()).b_p_ts != get_sw_value(curbuf.get()) as OptInt))
        && tabstop_count((*curbuf.get()).b_p_vsts_array) == 0 as ::core::ffi::c_int
        && get_sts_value() == 0 as ::core::ffi::c_int
    {
        return true_0 != 0;
    }
    if stop_arrow() == FAIL {
        return true_0 != 0;
    }
    did_ai.set(false_0 != 0);
    did_si.set(false_0 != 0);
    can_si.set(false_0 != 0);
    can_si_back.set(false_0 != 0);
    AppendToRedobuff(b"\t\0".as_ptr() as *const ::core::ffi::c_char);
    if p_sta.get() != 0 && ind as ::core::ffi::c_int != 0 {
        temp = get_sw_value(curbuf.get());
        temp -= get_nolist_virtcol() % temp;
    } else if tabstop_count((*curbuf.get()).b_p_vsts_array) > 0 as ::core::ffi::c_int
        || (*curbuf.get()).b_p_sts != 0 as OptInt
    {
        temp = tabstop_padding(
            get_nolist_virtcol(),
            get_sts_value() as OptInt,
            (*curbuf.get()).b_p_vsts_array,
        );
    } else {
        temp = tabstop_padding(
            get_nolist_virtcol(),
            (*curbuf.get()).b_p_ts,
            (*curbuf.get()).b_p_vts_array,
        );
    }
    ins_char(' ' as ::core::ffi::c_int);
    loop {
        temp -= 1;
        if temp <= 0 as ::core::ffi::c_int {
            break;
        }
        if State.get() & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
            ins_char(' ' as ::core::ffi::c_int);
        } else {
            ins_str(
                b" \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as size_t),
            );
            if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0 {
                replace_push_nul();
            }
        }
    }
    if (*curbuf.get()).b_p_et == 0
        && (tabstop_count((*curbuf.get()).b_p_vsts_array) > 0 as ::core::ffi::c_int
            || get_sts_value() > 0 as ::core::ffi::c_int
            || p_sta.get() != 0 && ind as ::core::ffi::c_int != 0)
    {
        let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut saved_line: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut pos: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut cursor: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
        let mut want_vcol: colnr_T = 0;
        let mut vcol: colnr_T = 0;
        let mut change_col: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut save_list: ::core::ffi::c_int = (*curwin.get()).w_onebuf_opt.wo_list;
        if State.get() & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
            pos = (*curwin.get()).w_cursor;
            cursor = &raw mut pos;
            saved_line = xstrnsave(get_cursor_line_ptr(), get_cursor_line_len() as size_t);
            ptr = saved_line.offset(pos.col as isize);
        } else {
            ptr = get_cursor_pos_ptr();
            cursor = &raw mut (*curwin.get()).w_cursor;
        }
        if vim_strchr(p_cpo.get(), CPO_LISTWM).is_null() {
            (*curwin.get()).w_onebuf_opt.wo_list = false_0;
        }
        let mut fpos: pos_T = (*curwin.get()).w_cursor;
        while fpos.col > 0 as ::core::ffi::c_int
            && ascii_iswhite(*ptr.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
        {
            fpos.col -= 1;
            ptr = ptr.offset(-1);
        }
        if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0
            && fpos.lnum == (*Insstart.ptr()).lnum
            && fpos.col < (*Insstart.ptr()).col
        {
            ptr = ptr.offset(((*Insstart.ptr()).col - fpos.col) as isize);
            fpos.col = (*Insstart.ptr()).col;
        }
        getvcol(
            curwin.get(),
            &raw mut fpos,
            &raw mut vcol,
            ::core::ptr::null_mut::<colnr_T>(),
            ::core::ptr::null_mut::<colnr_T>(),
        );
        getvcol(
            curwin.get(),
            cursor,
            &raw mut want_vcol,
            ::core::ptr::null_mut::<colnr_T>(),
            ::core::ptr::null_mut::<colnr_T>(),
        );
        let mut tab: *mut ::core::ffi::c_char =
            b"\t\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        let mut tab_v: int32_t = *tab as uint8_t as int32_t;
        let mut csarg: CharsizeArg = CharsizeArg {
            win: ::core::ptr::null_mut::<win_T>(),
            line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            use_tabstop: false,
            indent_width: 0,
            virt_row: 0,
            cur_text_width_left: 0,
            cur_text_width_right: 0,
            max_head_vcol: 0,
            iter: [MarkTreeIter {
                pos: MTPos { row: 0, col: 0 },
                lvl: 0,
                x: ::core::ptr::null_mut::<MTNode>(),
                i: 0,
                s: [C2Rust_Unnamed_16 { oldcol: 0, i: 0 }; 20],
                intersect_idx: 0,
                intersect_pos: MTPos { row: 0, col: 0 },
                intersect_pos_x: MTPos { row: 0, col: 0 },
            }; 1],
        };
        let mut cstype: CSType =
            init_charsize_arg(&raw mut csarg, curwin.get(), 0 as linenr_T, tab);
        while ascii_iswhite(*ptr as ::core::ffi::c_int) {
            let mut i: ::core::ffi::c_int = win_charsize(
                cstype,
                vcol as ::core::ffi::c_int,
                tab,
                tab_v,
                &raw mut csarg,
            )
            .width;
            if vcol as ::core::ffi::c_int + i > want_vcol {
                break;
            }
            if *ptr as ::core::ffi::c_int != TAB {
                *ptr = TAB as ::core::ffi::c_char;
                if change_col < 0 as ::core::ffi::c_int {
                    change_col = fpos.col as ::core::ffi::c_int;
                    if fpos.lnum == (*Insstart.ptr()).lnum && fpos.col < (*Insstart.ptr()).col {
                        (*Insstart.ptr()).col = fpos.col;
                    }
                }
            }
            fpos.col += 1;
            ptr = ptr.offset(1);
            vcol += i;
        }
        if change_col >= 0 as ::core::ffi::c_int {
            let mut repl_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            cstype = init_charsize_arg(&raw mut csarg, curwin.get(), 0 as linenr_T, ptr);
            while vcol < want_vcol && *ptr as ::core::ffi::c_int == ' ' as ::core::ffi::c_int {
                vcol += win_charsize(
                    cstype,
                    vcol as ::core::ffi::c_int,
                    ptr,
                    ' ' as ::core::ffi::c_int as uint8_t as int32_t,
                    &raw mut csarg,
                )
                .width;
                ptr = ptr.offset(1);
                repl_off += 1;
            }
            if vcol > want_vcol {
                ptr = ptr.offset(-1);
                repl_off -= 1;
            }
            fpos.col += repl_off;
            let mut i_0: ::core::ffi::c_int =
                (*cursor).col as ::core::ffi::c_int - fpos.col as ::core::ffi::c_int;
            if i_0 > 0 as ::core::ffi::c_int {
                if State.get() & VREPLACE_FLAG as ::core::ffi::c_int == 0 {
                    let newp_len: colnr_T = (*curbuf.get()).b_ml.ml_line_textlen - i_0 as colnr_T;
                    let mut newp: *mut ::core::ffi::c_char =
                        xmalloc(newp_len as size_t) as *mut ::core::ffi::c_char;
                    let mut col: ptrdiff_t = ptr.offset_from((*curbuf.get()).b_ml.ml_line_ptr);
                    if col > 0 as ptrdiff_t {
                        memmove(
                            newp as *mut ::core::ffi::c_void,
                            ptr.offset(-(col as isize)) as *const ::core::ffi::c_void,
                            col as size_t,
                        );
                    }
                    memmove(
                        newp.offset(col as isize) as *mut ::core::ffi::c_void,
                        ptr.offset(i_0 as isize) as *const ::core::ffi::c_void,
                        (newp_len as ptrdiff_t - col) as size_t,
                    );
                    if (*curbuf.get()).b_ml.ml_flags & (ML_LINE_DIRTY | ML_ALLOCATED) != 0 {
                        xfree((*curbuf.get()).b_ml.ml_line_ptr as *mut ::core::ffi::c_void);
                    }
                    (*curbuf.get()).b_ml.ml_line_ptr = newp;
                    (*curbuf.get()).b_ml.ml_line_textlen = newp_len;
                    (*curbuf.get()).b_ml.ml_flags =
                        ((*curbuf.get()).b_ml.ml_flags | ML_LINE_DIRTY) & !ML_EMPTY;
                    inserted_bytes(
                        fpos.lnum,
                        change_col as colnr_T,
                        (*cursor).col as ::core::ffi::c_int - change_col,
                        fpos.col as ::core::ffi::c_int - change_col,
                    );
                } else {
                    memmove(
                        ptr as *mut ::core::ffi::c_void,
                        ptr.offset(i_0 as isize) as *const ::core::ffi::c_void,
                        strlen(ptr.offset(i_0 as isize)).wrapping_add(1 as size_t),
                    );
                }
                if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0
                    && State.get() & VREPLACE_FLAG as ::core::ffi::c_int == 0
                {
                    temp = i_0;
                    loop {
                        temp -= 1;
                        if temp < 0 as ::core::ffi::c_int {
                            break;
                        }
                        replace_join(repl_off);
                    }
                }
            }
            (*cursor).col -= i_0;
            if State.get() & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
                backspace_until_column(change_col);
                ins_bytes_len(
                    saved_line.offset(change_col as isize),
                    ((*cursor).col as ::core::ffi::c_int - change_col) as size_t,
                );
            }
        }
        if State.get() & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
            xfree(saved_line as *mut ::core::ffi::c_void);
        }
        (*curwin.get()).w_onebuf_opt.wo_list = save_list;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ins_eol(mut c: ::core::ffi::c_int) -> bool {
    if echeck_abbr(c + ABBR_OFF) {
        return true_0 != 0;
    }
    if stop_arrow() == FAIL {
        return false_0 != 0;
    }
    undisplay_dollar();
    if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0
        && State.get() & VREPLACE_FLAG as ::core::ffi::c_int == 0
    {
        replace_push_nul();
    }
    if virtual_active(curwin.get()) as ::core::ffi::c_int != 0
        && (*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int
    {
        coladvance(curwin.get(), getviscol());
    }
    if revins_on.get() {
        (*curwin.get()).w_cursor.col += get_cursor_pos_len();
    }
    AppendToRedobuff(NL_STR.as_ptr());
    let mut i: bool = open_line(
        FORWARD as ::core::ffi::c_int,
        if has_format_option(FO_RET_COMS) as ::core::ffi::c_int != 0 {
            OPENLINE_DO_COM as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        },
        old_indent.get(),
        ::core::ptr::null_mut::<bool>(),
    );
    old_indent.set(0 as ::core::ffi::c_int);
    can_cindent.set(true_0 != 0);
    foldOpenCursor();
    return i;
}
unsafe extern "C" fn ins_digraph() -> ::core::ffi::c_int {
    let mut did_putchar: bool = false_0 != 0;
    pc_status.set(PC_STATUS_UNSET);
    if redrawing() as ::core::ffi::c_int != 0 && !char_avail() {
        ins_redraw(false_0 != 0);
        edit_putchar('?' as ::core::ffi::c_int, true_0 != 0);
        did_putchar = true_0 != 0;
        add_to_showcmd_c(Ctrl_K);
    }
    (*no_mapping.ptr()) += 1;
    (*allow_keys.ptr()) += 1;
    let mut c: ::core::ffi::c_int = plain_vgetc();
    (*no_mapping.ptr()) -= 1;
    (*allow_keys.ptr()) -= 1;
    if did_putchar {
        edit_unputchar();
    }
    if c < 0 as ::core::ffi::c_int || mod_mask.get() != 0 {
        clear_showcmd();
        insert_special(c, true_0, false_0);
        return NUL;
    }
    if c != ESC {
        did_putchar = false_0 != 0;
        if redrawing() as ::core::ffi::c_int != 0 && !char_avail() {
            ins_redraw(false_0 != 0);
            if char2cells(c) == 1 as ::core::ffi::c_int {
                ins_redraw(false_0 != 0);
                edit_putchar(c, true_0 != 0);
                did_putchar = true_0 != 0;
            }
            add_to_showcmd_c(c);
        }
        (*no_mapping.ptr()) += 1;
        (*allow_keys.ptr()) += 1;
        let mut cc: ::core::ffi::c_int = plain_vgetc();
        (*no_mapping.ptr()) -= 1;
        (*allow_keys.ptr()) -= 1;
        if did_putchar {
            edit_unputchar();
        }
        if cc != ESC {
            AppendToRedobuff(CTRL_V_STR.as_ptr());
            c = digraph_get(c, cc, true_0 != 0);
            clear_showcmd();
            return c;
        }
    }
    clear_showcmd();
    return NUL;
}
#[no_mangle]
pub unsafe extern "C" fn ins_copychar(mut lnum: linenr_T) -> ::core::ffi::c_int {
    if lnum < 1 as linenr_T || lnum > (*curbuf.get()).b_ml.ml_line_count {
        vim_beep(kOptBoFlagCopy as ::core::ffi::c_int as ::core::ffi::c_uint);
        return NUL;
    }
    validate_virtcol(curwin.get());
    let end_vcol: ::core::ffi::c_int = (*curwin.get()).w_virtcol as ::core::ffi::c_int;
    let mut line: *mut ::core::ffi::c_char = ml_get(lnum);
    let mut csarg: CharsizeArg = CharsizeArg {
        win: ::core::ptr::null_mut::<win_T>(),
        line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        use_tabstop: false,
        indent_width: 0,
        virt_row: 0,
        cur_text_width_left: 0,
        cur_text_width_right: 0,
        max_head_vcol: 0,
        iter: [MarkTreeIter {
            pos: MTPos { row: 0, col: 0 },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_16 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }; 1],
    };
    let mut cstype: CSType = init_charsize_arg(&raw mut csarg, curwin.get(), lnum, line);
    let mut ci: StrCharInfo = utf_ptr2StrCharInfo(line);
    let mut vcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while vcol < end_vcol && *ci.ptr as ::core::ffi::c_int != NUL {
        vcol += win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &raw mut csarg).width;
        if vcol > end_vcol {
            break;
        }
        ci = utfc_next(ci);
    }
    let mut c: ::core::ffi::c_int = if ci.chr.value < 0 as int32_t {
        *ci.ptr as uint8_t as ::core::ffi::c_int
    } else {
        ci.chr.value as ::core::ffi::c_int
    };
    if c == NUL {
        vim_beep(kOptBoFlagCopy as ::core::ffi::c_int as ::core::ffi::c_uint);
    }
    return c;
}
unsafe extern "C" fn ins_ctrl_ey(mut tc: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = tc;
    if ctrl_x_mode_scroll() {
        if c == Ctrl_Y {
            scrolldown_clamp();
        } else {
            scrollup_clamp();
        }
        redraw_later(curwin.get(), UPD_VALID as ::core::ffi::c_int);
    } else {
        c = ins_copychar(
            (*curwin.get()).w_cursor.lnum
                + (if c == Ctrl_Y {
                    -1 as linenr_T
                } else {
                    1 as linenr_T
                }),
        );
        if c != NUL {
            if c < 256 as ::core::ffi::c_int
                && *(*__ctype_b_loc()).offset(c as isize) as ::core::ffi::c_int
                    & _ISalnum as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    == 0
            {
                AppendToRedobuff(CTRL_V_STR.as_ptr());
            }
            let mut tw_save: OptInt = (*curbuf.get()).b_p_tw;
            (*curbuf.get()).b_p_tw = -1 as OptInt;
            insert_special(c, true_0, false_0);
            (*curbuf.get()).b_p_tw = tw_save;
            (*revins_chars.ptr()) += 1;
            (*revins_legal.ptr()) += 1;
            c = Ctrl_V;
            auto_format(false_0 != 0, true_0 != 0);
        }
    }
    return c;
}
#[no_mangle]
pub unsafe extern "C" fn get_nolist_virtcol() -> colnr_T {
    if (*curwin.get()).w_buffer.is_null()
        || (*(*curwin.get()).w_buffer).b_ml.ml_mfp.is_null()
        || (*curwin.get()).w_cursor.lnum > (*(*curwin.get()).w_buffer).b_ml.ml_line_count
    {
        return 0 as colnr_T;
    }
    if (*curwin.get()).w_onebuf_opt.wo_list != 0 && vim_strchr(p_cpo.get(), CPO_LISTWM).is_null() {
        return getvcol_nolist(&raw mut (*curwin.get()).w_cursor);
    }
    validate_virtcol(curwin.get());
    return (*curwin.get()).w_virtcol;
}
unsafe extern "C" fn do_insert_char_pre(mut c: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    let mut buf: [::core::ffi::c_char; 22] = [0; 22];
    let save_State: ::core::ffi::c_int = State.get();
    if c == Ctrl_RSB {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if !has_event(EVENT_INSERTCHARPRE) {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut buflen: size_t = utf_char2bytes(c, &raw mut buf as *mut ::core::ffi::c_char) as size_t;
    buf[buflen as usize] = NUL as ::core::ffi::c_char;
    (*textlock.ptr()) += 1;
    set_vim_var_string(
        VV_CHAR,
        &raw mut buf as *mut ::core::ffi::c_char,
        buflen as ptrdiff_t,
    );
    let mut res: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if ins_apply_autocmds(EVENT_INSERTCHARPRE) != 0 {
        if strcmp(
            &raw mut buf as *mut ::core::ffi::c_char,
            get_vim_var_str(VV_CHAR),
        ) != 0 as ::core::ffi::c_int
        {
            res = xstrdup(get_vim_var_str(VV_CHAR));
        }
    }
    set_vim_var_string(
        VV_CHAR,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
    (*textlock.ptr()) -= 1;
    State.set(save_State);
    return res;
}
#[no_mangle]
pub unsafe extern "C" fn get_can_cindent() -> bool {
    return can_cindent.get();
}
#[no_mangle]
pub unsafe extern "C" fn set_can_cindent(mut val: bool) {
    can_cindent.set(val);
}
#[no_mangle]
pub unsafe extern "C" fn ins_apply_autocmds(mut event: event_T) -> ::core::ffi::c_int {
    let mut tick: varnumber_T = buf_get_changedtick(curbuf.get());
    let mut r: ::core::ffi::c_int = apply_autocmds(
        event,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
    ) as ::core::ffi::c_int;
    if event as ::core::ffi::c_uint
        != EVENT_INSERTLEAVE as ::core::ffi::c_int as ::core::ffi::c_uint
        && tick != buf_get_changedtick(curbuf.get())
    {
        u_save(
            (*curwin.get()).w_cursor.lnum,
            (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
        );
    }
    return r;
}
pub const K_SPECIAL: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const ABBR_OFF: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const KS_MODIFIER: ::core::ffi::c_int = 252 as ::core::ffi::c_int;
pub const K_ZERO: ::core::ffi::c_int = -22783;
pub const K_UP: ::core::ffi::c_int = -30059;
pub const K_DOWN: ::core::ffi::c_int = -25707;
pub const K_LEFT: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('l' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_RIGHT: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('r' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_S_UP: ::core::ffi::c_int = -1277;
pub const K_S_DOWN: ::core::ffi::c_int = -1533;
pub const K_S_LEFT: ::core::ffi::c_int =
    -('#' as ::core::ffi::c_int + (('4' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_S_RIGHT: ::core::ffi::c_int =
    -('%' as ::core::ffi::c_int + (('i' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_S_HOME: ::core::ffi::c_int = -12835;
pub const K_S_END: ::core::ffi::c_int = -14122;
pub const K_S_TAB: ::core::ffi::c_int = -17003;
pub const K_XF1: ::core::ffi::c_int = -14845;
pub const K_F1: ::core::ffi::c_int = -12651;
pub const K_HELP: ::core::ffi::c_int =
    -('%' as ::core::ffi::c_int + (('1' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_BS: ::core::ffi::c_int = -25195;
pub const K_INS: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('I' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KINS: ::core::ffi::c_int = -20477;
pub const K_DEL: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('D' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KDEL: ::core::ffi::c_int = -20733;
pub const K_HOME: ::core::ffi::c_int = -26731;
pub const K_KHOME: ::core::ffi::c_int = -12619;
pub const K_END: ::core::ffi::c_int = -14144;
pub const K_KEND: ::core::ffi::c_int = -13387;
pub const K_PAGEUP: ::core::ffi::c_int = -20587;
pub const K_PAGEDOWN: ::core::ffi::c_int = -20075;
pub const K_KPAGEUP: ::core::ffi::c_int = -13131;
pub const K_KPAGEDOWN: ::core::ffi::c_int = -13643;
pub const K_KENTER: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('A' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_PASTE_START: ::core::ffi::c_int = -21328;
pub const K_SELECT: ::core::ffi::c_int = -22773;
pub const K_LEFTMOUSE: ::core::ffi::c_int = -11517;
pub const K_LEFTMOUSE_NM: ::core::ffi::c_int = -17917;
pub const K_LEFTDRAG: ::core::ffi::c_int = -11773;
pub const K_LEFTRELEASE: ::core::ffi::c_int = -12029;
pub const K_LEFTRELEASE_NM: ::core::ffi::c_int = -18173;
pub const K_MOUSEMOVE: ::core::ffi::c_int = -25853;
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
pub const K_MOUSEDOWN: ::core::ffi::c_int = -19453;
pub const K_MOUSEUP: ::core::ffi::c_int = -19709;
pub const K_MOUSELEFT: ::core::ffi::c_int = -19965;
pub const K_MOUSERIGHT: ::core::ffi::c_int = -20221;
pub const K_COMMAND: ::core::ffi::c_int = -26877;
pub const K_LUA: ::core::ffi::c_int = -26621;
pub const MOD_MASK_SHIFT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MOD_MASK_CTRL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const MOD_MASK_CMD: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn utf_ptr2CharInfo(p_in: *const ::core::ffi::c_char) -> CharInfo {
    let p: *const uint8_t = p_in as *const uint8_t;
    let first: uint8_t = *p;
    if (first as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
        return CharInfo {
            value: first as int32_t,
            len: 1 as ::core::ffi::c_int,
        };
    } else {
        let mut len: ::core::ffi::c_int =
            (*utf8len_tab.ptr())[first as usize] as ::core::ffi::c_int;
        let code_point: int32_t = utf_ptr2CharInfo_impl(p, len as uintptr_t);
        if code_point < 0 as int32_t {
            len = 1 as ::core::ffi::c_int;
        }
        return CharInfo {
            value: code_point,
            len: len,
        };
    };
}
#[inline(always)]
unsafe extern "C" fn utfc_next(mut cur: StrCharInfo) -> StrCharInfo {
    let mut next: *mut uint8_t = cur.ptr.offset(cur.chr.len as isize) as *mut uint8_t;
    if ((*next as ::core::ffi::c_uint) < 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int
        as ::core::ffi::c_long
        != 0
    {
        return StrCharInfo {
            ptr: next as *mut ::core::ffi::c_char,
            chr: CharInfo {
                value: *next as int32_t,
                len: 1 as ::core::ffi::c_int,
            },
        };
    }
    return utfc_next_impl(cur);
}
#[inline(always)]
unsafe extern "C" fn utf_ptr2StrCharInfo(mut ptr: *mut ::core::ffi::c_char) -> StrCharInfo {
    return StrCharInfo {
        ptr: ptr,
        chr: utf_ptr2CharInfo(ptr),
    };
}
pub const NULL_STRING: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
#[inline(always)]
unsafe extern "C" fn win_charsize(
    mut cstype: CSType,
    mut vcol: ::core::ffi::c_int,
    mut ptr: *mut ::core::ffi::c_char,
    mut chr: int32_t,
    mut csarg: *mut CharsizeArg,
) -> CharSize {
    if cstype as ::core::ffi::c_int == kCharsizeFast as ::core::ffi::c_int {
        return charsize_fast(csarg, ptr, vcol as colnr_T, chr);
    } else {
        return charsize_regular(csarg, ptr, vcol as colnr_T, chr);
    };
}
#[inline(always)]
unsafe extern "C" fn linetabsize_str(mut s: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    return linetabsize_col(0 as ::core::ffi::c_int, s);
}
#[inline]
unsafe extern "C" fn is_literal_register(regname: ::core::ffi::c_int) -> bool {
    return regname == '*' as ::core::ffi::c_int
        || regname == '+' as ::core::ffi::c_int
        || (regname as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && regname as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || regname as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && regname as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            || ascii_isdigit(regname) as ::core::ffi::c_int != 0);
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
