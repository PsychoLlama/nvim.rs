use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::arglist::check_arg_idx;
use crate::src::nvim::autocmd::{
    apply_autocmds, apply_autocmds_retval, aubuflocal_remove, aucmd_prepbuf, aucmd_restbuf,
    block_autocmds, is_aucmd_win, unblock_autocmds,
};
use crate::src::nvim::buffer_updates::{buf_free_callbacks, buf_updates_unload};
use crate::src::nvim::change::{changed, deleted_lines_mark, save_file_ff, unchanged};
use crate::src::nvim::channel::channel_job_running;
use crate::src::nvim::charset::{
    buf_init_chartab, getdigits_int, skiptowhite_esc, skipwhite, trans_characters, try_getdigits,
    vim_strsize,
};
use crate::src::nvim::cmdexpand::cmdline_fuzzy_complete;
use crate::src::nvim::cursor::{check_cursor_col, check_cursor_lnum};
use crate::src::nvim::diff::{diff_buf_add, diff_buf_delete, diff_mode_buf, diffopt_hiddenoff};
use crate::src::nvim::digraph::{keymap_ga_clear, keymap_init};
use crate::src::nvim::drawscreen::{redraw_later, redrawing, status_redraw_all};
use crate::src::nvim::eval::typval::{
    callback_free, tv_dict_add, tv_dict_alloc, tv_dict_find, tv_dict_item_copy,
    tv_dict_watcher_notify,
};
use crate::src::nvim::eval::vars::{init_var_dict, unref_var_dict, vars_clear};
use crate::src::nvim::ex_cmds::{do_ecmd, getfile};
use crate::src::nvim::ex_cmds2::{autowrite, can_abandon, dialog_changed, dialog_close_terminal};
use crate::src::nvim::ex_docmd::{do_cmdline_cmd, ex_errmsg, tabpage_new};
use crate::src::nvim::ex_eval::{aborting, enter_cleanup, leave_cleanup};
use crate::src::nvim::ex_getln::text_or_buf_locked;
use crate::src::nvim::extmark::extmark_free_all;
use crate::src::nvim::file_search::vim_chdirfile;
use crate::src::nvim::fileio::{
    buf_check_timestamp, file_pat_to_reg_pat, prep_exarg, readfile, shorten_fnames,
};
use crate::src::nvim::fold::{clearFolding, cloneFoldGrowArray, deleteFoldRecurse, foldUpdateAll};
use crate::src::nvim::fuzzy::{fuzzy_match_str, fuzzymatches_to_strmatches};
use crate::src::nvim::getchar::vgetc;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::help::get_local_additions;
use crate::src::nvim::indent::inindent;
use crate::src::nvim::indent_c::parse_cino;
use crate::src::nvim::insexpand::clear_cpt_callbacks;
use crate::src::nvim::main::{
    au_new_curbuf, au_pending_free_buf, autocmd_busy, autocmd_no_enter, autocmd_no_leave,
    buffer_handles, cmdline_row, cmdmod, cmdwin_buf, curbuf, current_sctx, curtab, curwin,
    e_auabort, e_buffer_nr_not_found, e_cannot_switch_to_a_closing_buffer, e_job_still_running,
    e_job_still_running_add_bang_to_end_the_job, e_no_write_since_last_change,
    e_no_write_since_last_change_add_bang_to_override,
    e_no_write_since_last_change_for_buffer_nr_add_bang_to_override, e_noalt, e_nobufnr,
    e_trailing_arg, emsg_silent, exiting, first_tabpage, firstbuf, firstwin, getout, got_int,
    in_assert_fails, jop_flags, last_chdir_reason, lastbuf, lastwin, msg_col, msg_loclist,
    msg_qflist, msg_row, msg_scroll, msg_scrolled, msg_silent, need_fileinfo, need_maketitle,
    need_wait_return, no_lines_msg, p_acd, p_ch, p_confirm, p_cpo, p_ea, p_fdls, p_fic, p_hid,
    p_icon, p_iconstring, p_mls, p_report, p_ru, p_sol, p_title, p_titlelen, p_titlestring, p_tpm,
    p_wic, p_write, readonlymode, restart_edit, secure, starting, stl_syntax, swap_exists_action,
    swap_exists_did_quit, swb_flags, updating_screen, v_dying, Columns, IObuff, NameBuff,
    RedrawingDisabled, Rows, State, VIsual_active, VIsual_reselect,
};
use crate::src::nvim::map::{map_del_int_ptr_t, map_put_ref_int_ptr_t, mh_get_int};
use crate::src::nvim::mapping::map_clear_mode;
use crate::src::nvim::mark::{
    clear_fmark, clrallmarks, fmarks_check_names, free_fmark, mark_adjust_buf, mark_forget_file,
    mark_jumplist_forget_file, mark_view_make, mark_view_restore, set_last_cursor, setpcmark,
};
use crate::src::nvim::mbyte::utf_cp_bounds;
use crate::src::nvim::memline::{
    ml_close, ml_delete, ml_get, ml_get_buf, ml_get_buf_len, ml_get_len, ml_open, ml_recover,
    ml_setname, ml_timestamp,
};
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc, xrealloc, xstrdup, xstrlcpy};
use crate::src::nvim::message::{
    emsg, message_filtered, msg, msg_delay, msg_ext_set_kind, msg_outtrans, msg_putchar, msg_puts,
    msg_start, msg_trunc, semsg, set_keep_msg, smsg,
};
use crate::src::nvim::normal::{end_visual_mode, reset_VIsual_and_resel};
use crate::src::nvim::option::{
    buf_copy_options, clear_winopt, copy_winopt, didset_window_options, do_set, magic_isset,
    set_option_value_give_err, shortmess,
};
use crate::src::nvim::optionstr::clear_string_option;
use crate::src::nvim::os::env::{home_replace, home_replace_save};
use crate::src::nvim::os::fs::{os_fileid, os_fileid_equal, os_getperm};
use crate::src::nvim::os::input::{line_breakcheck, os_breakcheck};
use crate::src::nvim::os::libc::{
    __assert_fail, __ctype_b_loc, gettext, memcpy, memmove, memset, ngettext, qsort, strcmp,
    strcpy, strlen, strncmp, time,
};
use crate::src::nvim::path::{fix_fname, path_fnamecmp, path_tail, FullName_save};
use crate::src::nvim::plines::win_get_fill;
use crate::src::nvim::quickfix::qf_stack_get_bufnr;
use crate::src::nvim::r#move::{scroll_cursor_halfway, validate_virtcol};
use crate::src::nvim::runtime::{estack_pop, estack_push};
use crate::src::nvim::spell::parse_spelllang;
use crate::src::nvim::statusline::build_stl_str_hl;
use crate::src::nvim::strings::{vim_snprintf, vim_snprintf_safelen, vim_strchr, xstrnsave};
use crate::src::nvim::syntax::{reset_synblock, syntax_clear};
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, Array, AutoPat, AutoPatCmd, AutoPatCmd_S, BoolVarValue, Boolean,
    BufUpdateCallbacks, CMD_index, Callback, CallbackType, Callback_data as C2Rust_Unnamed_6,
    CdCause, ChangedtickDictItem, CharBoundsOff, DecorExt, DecorHighlightInline, DecorInlineData,
    DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_3, Dict, ExtmarkMove,
    ExtmarkOp, ExtmarkSavePos, ExtmarkSplice, ExtmarkUndoObject, FileID, Float, FloatAnchor,
    FloatRelative, GridView, Integer, Intersection, KeyValuePair, LineGetter, LuaRef, MTKey,
    MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_int_ptr_t,
    Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkAdjustMode, MarkTree, Object, ObjectType,
    OptIndex, OptInt, OptVal, OptValData, OptValType, ScopeDictDictItem, ScopeType, ScreenGrid,
    Set_int, Set_int64_t, Set_uint32_t, Set_uint64_t, SignTextAttrs, SpecialVarValue,
    StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_14, StlClickRecord, StlFlag,
    StringBuilder, String_0, Terminal, Timestamp, TriState, UIExtension, UndoObjectType,
    VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo,
    WinSplit, WinStyle, Window, __compar_fn_t, __time_t, aco_save_T, alist_T, auto_event, bcount_t,
    bfa_values, bhdr_T, bln_values, blob_T, blobvar_S, blocknr_T, buf_T, bufref_T, bufstate_T,
    chunksize_T, cleanup_T, cleanup_stuff, cmd_addr_T, cmdidx_T, cmdmod_T, colnr_T, cstack_T,
    cstack_T_cs_pend as C2Rust_Unnamed_19, dict_T, dictitem_T, dictvar_S, diff_T, diffblock_S,
    disptick_T, dobuf_action_values, dobuf_start_values, eslist_T, eslist_elem, estack_T,
    estack_T_es_info as C2Rust_Unnamed_21, etype_T, event_T, exarg, exarg_T, except_T,
    except_type_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_4, file_buffer_b_wininfo as C2Rust_Unnamed_13,
    file_buffer_update_callbacks as C2Rust_Unnamed_1,
    file_buffer_update_channels as C2Rust_Unnamed_2, float_T, fmark_T, fmarkv_T, foldinfo_T,
    frame_S, frame_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_7, funccall_T,
    fuzmatch_str_T, garray_T, getf_values, handle_T, hash_T, hashitem_T, hashtab_T, infoptr_T,
    int16_t, int32_t, int64_t, int8_t, intmax_t, key_value_pair, lcs_chars_T, linenr_T, list_T,
    listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T, mapblock,
    mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T, msglist,
    msglist_T, mtnode_inner_s, mtnode_s, object, object_data as C2Rust_Unnamed_0, partial_S,
    partial_T, pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue,
    reg_extmatch_T, regmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T,
    size_t, statuscol_T, stl_hlrec, stl_hlrec_t, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_5, syn_time_T, synblock_T, synstate_T, tabpage_S,
    tabpage_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T,
    u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_10,
    u_header_uh_alt_prev as C2Rust_Unnamed_9, u_header_uh_next as C2Rust_Unnamed_12,
    u_header_uh_prev as C2Rust_Unnamed_11, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, undo_object_data as C2Rust_Unnamed_8, varnumber_T, vim_exception, virt_line,
    visualinfo_T, win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T, QUEUE,
};
use crate::src::nvim::ui::{ui_call_set_icon, ui_call_set_title, ui_has};
use crate::src::nvim::undo::{
    bufIsChanged, curbufIsChanged, u_clearallandblockfree, u_sync, undo_fmt_time,
};
use crate::src::nvim::usercmd::uc_clear;
use crate::src::nvim::version::min_vim_version;
use crate::src::nvim::window::{
    check_can_set_curbuf_forceit, check_colorcolumn, close_windows, free_wininfo, get_last_winid,
    global_stl_height, goto_tabpage_tp, goto_tabpage_win, last_window, lastwin_nofloating,
    one_window, swbuf_goto_win_with_buf, tabline_height, tabpage_index, win_close, win_enter,
    win_locked, win_move_after, win_split, win_valid, win_valid_any_tab, window_layout_lock,
    window_layout_unlock,
};
use crate::src::nvim::winfloat::win_set_minimal_style;
extern "C" {
    fn hash_init(ht: *mut hashtab_T);
    fn hash_find(ht: *const hashtab_T, key: *const ::core::ffi::c_char) -> *mut hashitem_T;
    fn hash_remove(ht: *mut hashtab_T, hi: *mut hashitem_T);
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
    fn terminal_close(termpp: *mut *mut Terminal, status: ::core::ffi::c_int);
    fn terminal_check_size(term: *mut Terminal);
    fn terminal_running(term: *const Terminal) -> bool;
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
pub const kExtmarkClear: UndoObjectType = 4;
pub const kExtmarkSavePos: UndoObjectType = 3;
pub const kExtmarkUpdate: UndoObjectType = 2;
pub const kExtmarkMove: UndoObjectType = 1;
pub const kExtmarkSplice: UndoObjectType = 0;
pub const kStlClickFuncRun: C2Rust_Unnamed_14 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_14 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_14 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_14 = 0;
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
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed_15 = 2147483647;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const DO_NOT_FREE_CNT: C2Rust_Unnamed_16 = 1073741823;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const DI_FLAGS_ALLOC: C2Rust_Unnamed_17 = 16;
pub const DI_FLAGS_LOCK: C2Rust_Unnamed_17 = 8;
pub const DI_FLAGS_FIX: C2Rust_Unnamed_17 = 4;
pub const DI_FLAGS_RO_SBX: C2Rust_Unnamed_17 = 2;
pub const DI_FLAGS_RO: C2Rust_Unnamed_17 = 1;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_int;
pub const BACKWARD_FILE: C2Rust_Unnamed_18 = -3;
pub const FORWARD_FILE: C2Rust_Unnamed_18 = 3;
pub const BACKWARD: C2Rust_Unnamed_18 = -1;
pub const FORWARD: C2Rust_Unnamed_18 = 1;
pub const kDirectionNotSet: C2Rust_Unnamed_18 = 0;
pub const kCdCauseAuto: CdCause = 2;
pub const kCdCauseWindow: CdCause = 1;
pub const kCdCauseManual: CdCause = 0;
pub const kCdCauseOther: CdCause = -1;
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
pub const STL_CLICK_FUNC: StlFlag = 64;
pub const STL_TABCLOSENR: StlFlag = 88;
pub const STL_TABPAGENR: StlFlag = 84;
pub const STL_HIGHLIGHT_COMB: StlFlag = 36;
pub const STL_HIGHLIGHT: StlFlag = 35;
pub const STL_USER_HL: StlFlag = 42;
pub const STL_TRUNCMARK: StlFlag = 60;
pub const STL_SEPARATE: StlFlag = 61;
pub const STL_VIM_EXPR: StlFlag = 123;
pub const STL_SIGNCOL: StlFlag = 115;
pub const STL_FOLDCOL: StlFlag = 67;
pub const STL_SHOWCMD: StlFlag = 83;
pub const STL_PAGENUM: StlFlag = 78;
pub const STL_ARGLISTSTAT: StlFlag = 97;
pub const STL_ALTPERCENT: StlFlag = 80;
pub const STL_PERCENTAGE: StlFlag = 112;
pub const STL_QUICKFIX: StlFlag = 113;
pub const STL_MODIFIED_ALT: StlFlag = 77;
pub const STL_MODIFIED: StlFlag = 109;
pub const STL_PREVIEWFLAG_ALT: StlFlag = 87;
pub const STL_PREVIEWFLAG: StlFlag = 119;
pub const STL_FILETYPE_ALT: StlFlag = 89;
pub const STL_FILETYPE: StlFlag = 121;
pub const STL_HELPFLAG_ALT: StlFlag = 72;
pub const STL_HELPFLAG: StlFlag = 104;
pub const STL_ROFLAG_ALT: StlFlag = 82;
pub const STL_ROFLAG: StlFlag = 114;
pub const STL_BYTEVAL_X: StlFlag = 66;
pub const STL_BYTEVAL: StlFlag = 98;
pub const STL_OFFSET_X: StlFlag = 79;
pub const STL_OFFSET: StlFlag = 111;
pub const STL_KEYMAP: StlFlag = 107;
pub const STL_BUFNO: StlFlag = 110;
pub const STL_NUMLINES: StlFlag = 76;
pub const STL_LINE: StlFlag = 108;
pub const STL_VIRTCOL_ALT: StlFlag = 86;
pub const STL_VIRTCOL: StlFlag = 118;
pub const STL_COLUMN: StlFlag = 99;
pub const STL_FILENAME: StlFlag = 116;
pub const STL_FULLPATH: StlFlag = 70;
pub const STL_FILEPATH: StlFlag = 102;
pub const kExtmarkUndoNoRedo: ExtmarkOp = 3;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
pub const kMarkAdjustTerm: MarkAdjustMode = 2;
pub const kMarkAdjustApi: MarkAdjustMode = 1;
pub const kMarkAdjustNormal: MarkAdjustMode = 0;
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
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_20 = 8192;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_20 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_20 = 2048;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_20 = 1024;
pub const CMOD_KEEPMARKS: C2Rust_Unnamed_20 = 512;
pub const CMOD_KEEPALT: C2Rust_Unnamed_20 = 256;
pub const CMOD_CONFIRM: C2Rust_Unnamed_20 = 128;
pub const CMOD_BROWSE: C2Rust_Unnamed_20 = 64;
pub const CMOD_HIDE: C2Rust_Unnamed_20 = 32;
pub const CMOD_NOAUTOCMD: C2Rust_Unnamed_20 = 16;
pub const CMOD_UNSILENT: C2Rust_Unnamed_20 = 8;
pub const CMOD_ERRSILENT: C2Rust_Unnamed_20 = 4;
pub const CMOD_SILENT: C2Rust_Unnamed_20 = 2;
pub const CMOD_SANDBOX: C2Rust_Unnamed_20 = 1;
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
pub const GETF_SWITCH: getf_values = 4;
pub const GETF_ALT: getf_values = 2;
pub const GETF_SETMARK: getf_values = 1;
pub const BLN_NOCURWIN: bln_values = 128;
pub const BLN_NOOPT: bln_values = 16;
pub const BLN_NEW: bln_values = 8;
pub const BLN_DUMMY: bln_values = 4;
pub const BLN_LISTED: bln_values = 2;
pub const BLN_CURBUF: bln_values = 1;
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_DEL: dobuf_action_values = 3;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub const DOBUF_SPLIT: dobuf_action_values = 1;
pub const DOBUF_GOTO: dobuf_action_values = 0;
pub const DOBUF_MOD: dobuf_start_values = 3;
pub const DOBUF_LAST: dobuf_start_values = 2;
pub const DOBUF_FIRST: dobuf_start_values = 1;
pub const DOBUF_CURRENT: dobuf_start_values = 0;
pub type dobuf_flags_value = ::core::ffi::c_uint;
pub const DOBUF_SKIPHELP: dobuf_flags_value = 4;
pub const DOBUF_FORCEIT: dobuf_flags_value = 1;
pub const BFA_IGNORE_ABORT: bfa_values = 8;
pub const BFA_KEEP_UNDO: bfa_values = 4;
pub const BFA_WIPE: bfa_values = 2;
pub const BFA_DEL: bfa_values = 1;
pub const READ_NOWINENTER: C2Rust_Unnamed_29 = 128;
pub const OPT_LOCAL: C2Rust_Unnamed_33 = 2;
pub const OPT_MODELINE: C2Rust_Unnamed_33 = 4;
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
pub const SHM_FILEINFO: C2Rust_Unnamed_24 = 70;
pub const READ_BUFFER: C2Rust_Unnamed_29 = 8;
pub const READ_STDIN: C2Rust_Unnamed_29 = 4;
pub const READ_NEW: C2Rust_Unnamed_29 = 1;
pub const READ_FIFO: C2Rust_Unnamed_29 = 64;
pub const READ_NOFILE: C2Rust_Unnamed_29 = 256;
pub const UPD_NOT_VALID: C2Rust_Unnamed_26 = 40;
pub const kOptJopFlagView: C2Rust_Unnamed_22 = 2;
pub const BCO_NOHELP: C2Rust_Unnamed_32 = 4;
pub const BCO_ENTER: C2Rust_Unnamed_32 = 1;
pub const kBffInitChangedtick: C2Rust_Unnamed_35 = 2;
pub const kBffClearWinInfo: C2Rust_Unnamed_35 = 1;
pub const MAP_ALL_MODES: C2Rust_Unnamed_31 = 255;
pub const BCO_ALWAYS: C2Rust_Unnamed_32 = 2;
pub const MODE_INSERT: C2Rust_Unnamed_31 = 16;
pub const ECMD_FORCEIT: C2Rust_Unnamed_27 = 8;
pub const ECMD_ONE: C2Rust_Unnamed_28 = 1;
pub const kOptJopFlagClean: C2Rust_Unnamed_22 = 4;
pub const WSP_VERT: C2Rust_Unnamed_34 = 2;
pub const kOptSwbFlagVsplit: C2Rust_Unnamed_23 = 16;
pub const kOptSwbFlagNewtab: C2Rust_Unnamed_23 = 8;
pub const kOptSwbFlagSplit: C2Rust_Unnamed_23 = 4;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bufmatch_T {
    pub buf: *mut buf_T,
    pub match_0: *mut ::core::ffi::c_char,
}
pub const WILD_BUFLASTUSED: C2Rust_Unnamed_25 = 4096;
pub const WILD_HOME_REPLACE: C2Rust_Unnamed_25 = 2;
pub const FUZZY_SCORE_NONE: C2Rust_Unnamed_30 = -2147483648;
pub const BUF_DIFF_FILTER: C2Rust_Unnamed_25 = 8192;
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
pub const SHM_RO: C2Rust_Unnamed_24 = 114;
pub const SHM_MOD: C2Rust_Unnamed_24 = 109;
pub const WSP_BELOW: C2Rust_Unnamed_34 = 64;
pub const WSP_ROOM: C2Rust_Unnamed_34 = 1;
pub const READ_DUMMY: C2Rust_Unnamed_29 = 16;
pub const ECMD_HIDE: C2Rust_Unnamed_27 = 1;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const kOptJopFlagStack: C2Rust_Unnamed_22 = 1;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const kOptSwbFlagUselast: C2Rust_Unnamed_23 = 32;
pub const kOptSwbFlagUsetab: C2Rust_Unnamed_23 = 2;
pub const kOptSwbFlagUseopen: C2Rust_Unnamed_23 = 1;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const SHM_SEARCHCOUNT: C2Rust_Unnamed_24 = 83;
pub const SHM_RECORDING: C2Rust_Unnamed_24 = 113;
pub const SHM_COMPLETIONSCAN: C2Rust_Unnamed_24 = 67;
pub const SHM_COMPLETIONMENU: C2Rust_Unnamed_24 = 99;
pub const SHM_INTRO: C2Rust_Unnamed_24 = 73;
pub const SHM_ATTENTION: C2Rust_Unnamed_24 = 65;
pub const SHM_SEARCH: C2Rust_Unnamed_24 = 115;
pub const SHM_OVERALL: C2Rust_Unnamed_24 = 79;
pub const SHM_OVER: C2Rust_Unnamed_24 = 111;
pub const SHM_TRUNCALL: C2Rust_Unnamed_24 = 84;
pub const SHM_TRUNC: C2Rust_Unnamed_24 = 116;
pub const SHM_WRITE: C2Rust_Unnamed_24 = 87;
pub const SHM_ABBREVIATIONS: C2Rust_Unnamed_24 = 97;
pub const SHM_WRI: C2Rust_Unnamed_24 = 119;
pub const SHM_LINES: C2Rust_Unnamed_24 = 108;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const WILD_FUNC_TRIGGER: C2Rust_Unnamed_25 = 65536;
pub const WILD_MAY_EXPAND_PATTERN: C2Rust_Unnamed_25 = 32768;
pub const WILD_NOSELECT: C2Rust_Unnamed_25 = 16384;
pub const WILD_NOERROR: C2Rust_Unnamed_25 = 2048;
pub const WILD_IGNORE_COMPLETESLASH: C2Rust_Unnamed_25 = 1024;
pub const WILD_ALLLINKS: C2Rust_Unnamed_25 = 512;
pub const WILD_ICASE: C2Rust_Unnamed_25 = 256;
pub const WILD_ESCAPE: C2Rust_Unnamed_25 = 128;
pub const WILD_SILENT: C2Rust_Unnamed_25 = 64;
pub const WILD_KEEP_ALL: C2Rust_Unnamed_25 = 32;
pub const WILD_ADD_SLASH: C2Rust_Unnamed_25 = 16;
pub const WILD_NO_BEEP: C2Rust_Unnamed_25 = 8;
pub const WILD_USE_NL: C2Rust_Unnamed_25 = 4;
pub const WILD_LIST_NOTFOUND: C2Rust_Unnamed_25 = 1;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_26 = 50;
pub const UPD_SOME_VALID: C2Rust_Unnamed_26 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_26 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_26 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_26 = 20;
pub const UPD_VALID: C2Rust_Unnamed_26 = 10;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const ECMD_NOWINENTER: C2Rust_Unnamed_27 = 64;
pub const ECMD_ALTBUF: C2Rust_Unnamed_27 = 32;
pub const ECMD_ADDBUF: C2Rust_Unnamed_27 = 16;
pub const ECMD_OLDBUF: C2Rust_Unnamed_27 = 4;
pub const ECMD_SET_HELP: C2Rust_Unnamed_27 = 2;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_int;
pub const ECMD_LAST: C2Rust_Unnamed_28 = -1;
pub const ECMD_LASTL: C2Rust_Unnamed_28 = 0;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const READ_KEEP_UNDO: C2Rust_Unnamed_29 = 32;
pub const READ_FILTER: C2Rust_Unnamed_29 = 2;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_31 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_31 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_31 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_31 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_31 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_31 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_31 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_31 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_31 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_31 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_31 = 256;
pub const MODE_TERMINAL: C2Rust_Unnamed_31 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_31 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_31 = 32;
pub const MODE_CMDLINE: C2Rust_Unnamed_31 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_31 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_31 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_31 = 1;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_33 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_33 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_33 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_33 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_33 = 8;
pub const OPT_GLOBAL: C2Rust_Unnamed_33 = 1;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub const WSP_QUICKFIX: C2Rust_Unnamed_34 = 1024;
pub const WSP_NOENTER: C2Rust_Unnamed_34 = 512;
pub const WSP_NEWLOC: C2Rust_Unnamed_34 = 256;
pub const WSP_ABOVE: C2Rust_Unnamed_34 = 128;
pub const WSP_HELP: C2Rust_Unnamed_34 = 32;
pub const WSP_BOT: C2Rust_Unnamed_34 = 16;
pub const WSP_TOP: C2Rust_Unnamed_34 = 8;
pub const WSP_HOR: C2Rust_Unnamed_34 = 4;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const VALID_TOPLINE: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const BF_CHECK_RO: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const BF_NEVERLOADED: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const BF_NOTEDITED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const BF_NEW: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const BF_READERR: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const BF_DUMMY: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const BF_WRITE_MASK: ::core::ffi::c_int = BF_NOTEDITED + BF_NEW + BF_READERR;
pub const KEYMAP_INIT: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const NMARKS: ::core::ffi::c_int =
    'z' as ::core::ffi::c_int - 'a' as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn QUEUE_EMPTY(q: *const QUEUE) -> ::core::ffi::c_int {
    return (q == (*q).next as *const QUEUE) as ::core::ffi::c_int;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static value_init_ptr_t: GlobalCell<ptr_t> = GlobalCell::new(NULL);
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
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
#[inline]
unsafe extern "C" fn map_put_int_ptr_t(
    mut map: *mut Map_int_ptr_t,
    mut key: ::core::ffi::c_int,
    mut value: ptr_t,
) {
    let mut val: *mut ptr_t = map_put_ref_int_ptr_t(
        map,
        key,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_int>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isspace(mut c: ::core::ffi::c_int) -> bool {
    return c >= 9 as ::core::ffi::c_int && c <= 13 as ::core::ffi::c_int
        || c == ' ' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn buf_get_changedtick(buf: *const buf_T) -> varnumber_T {
    return (*buf).changedtick_di.di_tv.vval.v_number;
}
static e_attempt_to_delete_buffer_that_is_in_use_str: GlobalCell<[::core::ffi::c_char; 52]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 52], [::core::ffi::c_char; 52]>(
            *b"E937: Attempt to delete a buffer that is in use: %s\0",
        )
    });
static buf_free_count: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static top_file_num: GlobalCell<::core::ffi::c_int> = GlobalCell::new(1 as ::core::ffi::c_int);
unsafe extern "C" fn trigger_undo_ftplugin(mut buf: *mut buf_T, mut win: *mut win_T) {
    let win_was_locked: bool = (*win).w_locked;
    window_layout_lock();
    (*buf).b_locked += 1;
    (*win).w_locked = true_0 != 0;
    do_cmdline_cmd(
        b"if exists('b:undo_ftplugin') | exe b:undo_ftplugin | endif\0".as_ptr()
            as *const ::core::ffi::c_char,
    );
    (*buf).b_locked -= 1;
    (*win).w_locked = win_was_locked;
    window_layout_unlock();
}
pub unsafe extern "C" fn calc_percentage(
    mut part: int64_t,
    mut whole: int64_t,
) -> ::core::ffi::c_int {
    return if part > 1000000 as int64_t {
        (part / (whole / 100 as int64_t)) as ::core::ffi::c_int
    } else {
        (part * 100 as int64_t / whole) as ::core::ffi::c_int
    };
}
pub unsafe extern "C" fn get_highest_fnum() -> ::core::ffi::c_int {
    return top_file_num.get() - 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn read_buffer(
    mut read_stdin: bool,
    mut eap: *mut exarg_T,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = OK;
    let mut silent: bool = shortmess(SHM_FILEINFO as ::core::ffi::c_int);
    let mut line_count: linenr_T = (*curbuf.get()).b_ml.ml_line_count;
    retval = readfile(
        if read_stdin as ::core::ffi::c_int != 0 {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        } else {
            (*curbuf.get()).b_ffname
        },
        if read_stdin as ::core::ffi::c_int != 0 {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        } else {
            (*curbuf.get()).b_fname
        },
        line_count,
        0 as linenr_T,
        MAXLNUM as ::core::ffi::c_int as linenr_T,
        eap,
        flags | READ_BUFFER as ::core::ffi::c_int,
        silent,
    );
    if retval == OK {
        loop {
            line_count -= 1;
            if line_count < 0 as linenr_T {
                break;
            }
            ml_delete(1 as linenr_T);
        }
    } else {
        while (*curbuf.get()).b_ml.ml_line_count > line_count {
            ml_delete(line_count);
        }
    }
    (*curwin.get()).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    if read_stdin {
        if !readonlymode.get() && !buf_is_empty(curbuf.get()) {
            changed(curbuf.get());
        } else if retval != FAIL {
            unchanged(curbuf.get(), false_0 != 0, true_0 != 0);
        }
        apply_autocmds_retval(
            EVENT_STDINREADPOST,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
            &raw mut retval,
        );
    }
    return retval;
}
pub unsafe extern "C" fn buf_ensure_loaded(mut buf: *mut buf_T) -> bool {
    if !(*buf).b_ml.ml_mfp.is_null() {
        return true_0 != 0;
    }
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
    let mut status: ::core::ffi::c_int = open_buffer(
        false_0 != 0,
        ::core::ptr::null_mut::<exarg_T>(),
        0 as ::core::ffi::c_int,
    );
    aucmd_restbuf(&raw mut aco);
    return status != FAIL;
}
pub unsafe extern "C" fn open_buffer(
    mut read_stdin: bool,
    mut eap: *mut exarg_T,
    mut flags_arg: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut flags: ::core::ffi::c_int = flags_arg;
    let mut retval: ::core::ffi::c_int = OK;
    let mut old_curbuf: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    let mut old_tw: OptInt = (*curbuf.get()).b_p_tw;
    let mut read_fifo: bool = false_0 != 0;
    let mut silent: bool = shortmess(SHM_FILEINFO as ::core::ffi::c_int);
    if readonlymode.get() as ::core::ffi::c_int != 0
        && !(*curbuf.get()).b_ffname.is_null()
        && (*curbuf.get()).b_flags & BF_NEVERLOADED != 0
    {
        (*curbuf.get()).b_p_ro = true_0;
    }
    if ml_open(curbuf.get()) == FAIL {
        close_buffer(
            ::core::ptr::null_mut::<win_T>(),
            curbuf.get(),
            0 as ::core::ffi::c_int,
            false_0 != 0,
            false_0 != 0,
        );
        curbuf.set(::core::ptr::null_mut::<buf_T>());
        let mut buf: *mut buf_T = firstbuf.get();
        while !buf.is_null() {
            if !(*buf).b_ml.ml_mfp.is_null() {
                curbuf.set(buf);
                break;
            } else {
                buf = (*buf).b_next;
            }
        }
        if (*curbuf.ptr()).is_null() {
            emsg(gettext(
                b"E82: Cannot allocate any buffer, exiting...\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            v_dying.set(2 as ::core::ffi::c_int);
            getout(2 as ::core::ffi::c_int);
        }
        emsg(gettext(
            b"E83: Cannot allocate buffer, using other one...\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
        enter_buffer(curbuf.get());
        if old_tw != (*curbuf.get()).b_p_tw {
            check_colorcolumn(::core::ptr::null_mut::<::core::ffi::c_char>(), curwin.get());
        }
        return FAIL;
    }
    if !(*curbuf.get()).b_ml.ml_mfp.is_null() {
        (*(*curbuf.get()).b_ml.ml_mfp).mf_dirty = MF_DIRTY_YES_NOSYNC;
    }
    set_bufref(&raw mut old_curbuf, curbuf.get());
    (*curbuf.get()).b_modified_was_set = false_0 != 0;
    (*curwin.get()).w_valid = 0 as ::core::ffi::c_int;
    if bt_nofileread(curbuf.get()) {
        flags |= READ_NOFILE as ::core::ffi::c_int;
    }
    if !(*curbuf.get()).b_ffname.is_null() {
        let mut save_bin: ::core::ffi::c_int = (*curbuf.get()).b_p_bin;
        let mut perm: ::core::ffi::c_int =
            os_getperm((*curbuf.get()).b_ffname) as ::core::ffi::c_int;
        if perm >= 0 as ::core::ffi::c_int
            && (false
                || perm & __S_IFMT == 0o10000 as ::core::ffi::c_int
                || perm & __S_IFMT == 0o140000 as ::core::ffi::c_int)
        {
            read_fifo = true_0 != 0;
        }
        if read_fifo {
            (*curbuf.get()).b_p_bin = true_0;
        }
        retval = readfile(
            (*curbuf.get()).b_ffname,
            (*curbuf.get()).b_fname,
            0 as linenr_T,
            0 as linenr_T,
            MAXLNUM as ::core::ffi::c_int as linenr_T,
            eap,
            flags
                | READ_NEW as ::core::ffi::c_int
                | (if read_fifo as ::core::ffi::c_int != 0 {
                    READ_FIFO as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }),
            silent,
        );
        if read_fifo {
            (*curbuf.get()).b_p_bin = save_bin;
            if retval == OK {
                retval = read_buffer(false_0 != 0, eap, flags);
            }
        }
        if bt_help(curbuf.get()) {
            get_local_additions();
        }
    } else if read_stdin {
        let mut save_bin_0: ::core::ffi::c_int = (*curbuf.get()).b_p_bin;
        (*curbuf.get()).b_p_bin = true_0;
        retval = readfile(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            0 as linenr_T,
            0 as linenr_T,
            MAXLNUM as ::core::ffi::c_int as linenr_T,
            ::core::ptr::null_mut::<exarg_T>(),
            flags | READ_NEW as ::core::ffi::c_int + READ_STDIN as ::core::ffi::c_int,
            silent,
        );
        (*curbuf.get()).b_p_bin = save_bin_0;
        if retval == OK {
            retval = read_buffer(true_0 != 0, eap, flags);
        }
    }
    if !(*curbuf.get()).b_ml.ml_mfp.is_null()
        && (*(*curbuf.get()).b_ml.ml_mfp).mf_dirty as ::core::ffi::c_uint
            == MF_DIRTY_YES_NOSYNC as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*(*curbuf.get()).b_ml.ml_mfp).mf_dirty = MF_DIRTY_YES;
    }
    if (*curbuf.get()).b_flags & BF_NEVERLOADED != 0 {
        buf_init_chartab(curbuf.get(), false_0 != 0);
        parse_cino(curbuf.get());
    }
    if got_int.get() as ::core::ffi::c_int != 0 && !vim_strchr(p_cpo.get(), CPO_INTMOD).is_null()
        || (*curbuf.get()).b_modified_was_set as ::core::ffi::c_int != 0
        || aborting() as ::core::ffi::c_int != 0 && !vim_strchr(p_cpo.get(), CPO_INTMOD).is_null()
    {
        changed(curbuf.get());
    } else if retval != FAIL && !read_stdin && !read_fifo {
        unchanged(curbuf.get(), false_0 != 0, true_0 != 0);
    }
    save_file_ff(curbuf.get());
    (*curbuf.get()).b_last_changedtick = buf_get_changedtick(curbuf.get());
    (*curbuf.get()).b_last_changedtick_i = buf_get_changedtick(curbuf.get());
    (*curbuf.get()).b_last_changedtick_pum = buf_get_changedtick(curbuf.get());
    if aborting() {
        (*curbuf.get()).b_flags |= BF_READERR;
    }
    foldUpdateAll(curwin.get());
    if (*curwin.get()).w_valid & VALID_TOPLINE == 0 {
        (*curwin.get()).w_topline = 1 as ::core::ffi::c_int as linenr_T;
        (*curwin.get()).w_topfill = 0 as ::core::ffi::c_int;
    }
    apply_autocmds_retval(
        EVENT_BUFENTER,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
        &raw mut retval,
    );
    if retval == FAIL {
        return retval;
    }
    if bufref_valid(&raw mut old_curbuf) as ::core::ffi::c_int != 0
        && !(*old_curbuf.br_buf).b_ml.ml_mfp.is_null()
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
        aucmd_prepbuf(&raw mut aco, old_curbuf.br_buf);
        do_modelines(0 as ::core::ffi::c_int);
        (*curbuf.get()).b_flags &= !(BF_CHECK_RO | BF_NEVERLOADED);
        if flags & READ_NOWINENTER as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            apply_autocmds_retval(
                EVENT_BUFWINENTER,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
                &raw mut retval,
            );
        }
        aucmd_restbuf(&raw mut aco);
    }
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn set_bufref(mut bufref: *mut bufref_T, mut buf: *mut buf_T) {
    (*bufref).br_buf = buf;
    (*bufref).br_fnum = if buf.is_null() {
        0 as ::core::ffi::c_int
    } else {
        (*buf).handle as ::core::ffi::c_int
    };
    (*bufref).br_buf_free_count = buf_free_count.get();
}
#[no_mangle]
pub unsafe extern "C" fn bufref_valid(mut bufref: *mut bufref_T) -> bool {
    return if (*bufref).br_buf_free_count == buf_free_count.get() {
        true_0
    } else {
        (buf_valid((*bufref).br_buf) as ::core::ffi::c_int != 0
            && (*bufref).br_fnum == (*(*bufref).br_buf).handle) as ::core::ffi::c_int
    } != 0;
}
#[no_mangle]
pub unsafe extern "C" fn buf_valid(mut buf: *mut buf_T) -> bool {
    if buf.is_null() {
        return false_0 != 0;
    }
    let mut bp: *mut buf_T = lastbuf.get();
    while !bp.is_null() {
        if bp == buf {
            return true_0 != 0;
        }
        bp = (*bp).b_prev;
    }
    return false_0 != 0;
}
unsafe extern "C" fn can_unload_buffer(mut buf: *mut buf_T) -> bool {
    let mut can_unload: bool = (*buf).b_locked == 0;
    if can_unload as ::core::ffi::c_int != 0 && updating_screen.get() as ::core::ffi::c_int != 0 {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == buf {
                can_unload = false_0 != 0;
                break;
            } else {
                wp = (*wp).w_next;
            }
        }
    }
    if can_unload as ::core::ffi::c_int != 0 && (*buf).b_saving as ::core::ffi::c_int != 0 {
        can_unload = false_0 != 0;
    }
    if !can_unload {
        let mut fname: *mut ::core::ffi::c_char = if !(*buf).b_fname.is_null() {
            (*buf).b_fname
        } else {
            (*buf).b_ffname
        };
        semsg(
            gettext(
                (e_attempt_to_delete_buffer_that_is_in_use_str.ptr() as *const _)
                    as *const ::core::ffi::c_char,
            ),
            if !fname.is_null() {
                fname as *const ::core::ffi::c_char
            } else {
                b"[No Name]\0".as_ptr() as *const ::core::ffi::c_char
            },
        );
    }
    return can_unload;
}
pub unsafe extern "C" fn buf_close_terminal(mut buf: *mut buf_T) {
    '_c2rust_label: {
        if !(*buf).terminal.is_null() {
        } else {
            __assert_fail(
                b"buf->terminal\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/buffer.rs\0".as_ptr() as *const ::core::ffi::c_char,
                521 as ::core::ffi::c_uint,
                b"void buf_close_terminal(buf_T *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    (*buf).b_locked += 1;
    terminal_close(&raw mut (*buf).terminal, -1 as ::core::ffi::c_int);
    (*buf).b_locked -= 1;
}
#[no_mangle]
pub unsafe extern "C" fn close_buffer(
    mut win: *mut win_T,
    mut buf: *mut buf_T,
    mut action: ::core::ffi::c_int,
    mut abort_if_last: bool,
    mut ignore_abort: bool,
) -> bool {
    let mut unload_buf: bool = action != 0 as ::core::ffi::c_int;
    let mut del_buf: bool =
        action == DOBUF_DEL as ::core::ffi::c_int || action == DOBUF_WIPE as ::core::ffi::c_int;
    let mut wipe_buf: bool = action == DOBUF_WIPE as ::core::ffi::c_int;
    let mut is_curwin: bool = !(*curwin.ptr()).is_null() && (*curwin.get()).w_buffer == buf;
    let mut the_curwin: *mut win_T = curwin.get();
    let mut the_curtab: *mut tabpage_T = curtab.get();
    if (*buf).terminal.is_null() {
        if *(*buf).b_p_bh.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'd' as ::core::ffi::c_int
        {
            del_buf = true_0 != 0;
            unload_buf = true_0 != 0;
        } else if *(*buf).b_p_bh.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'w' as ::core::ffi::c_int
        {
            del_buf = true_0 != 0;
            unload_buf = true_0 != 0;
            wipe_buf = true_0 != 0;
        } else if *(*buf).b_p_bh.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'u' as ::core::ffi::c_int
        {
            unload_buf = true_0 != 0;
        }
    }
    if !(*buf).terminal.is_null()
        && (unload_buf as ::core::ffi::c_int != 0
            || del_buf as ::core::ffi::c_int != 0
            || wipe_buf as ::core::ffi::c_int != 0)
    {
        unload_buf = true_0 != 0;
        del_buf = true_0 != 0;
        wipe_buf = true_0 != 0;
    }
    if (del_buf as ::core::ffi::c_int != 0 || wipe_buf as ::core::ffi::c_int != 0)
        && !can_unload_buffer(buf)
    {
        return false_0 != 0;
    }
    if !win.is_null() && win_valid_any_tab(win) as ::core::ffi::c_int != 0 {
        if (*buf).b_nwindows == 1 as ::core::ffi::c_int {
            set_last_cursor(win);
        }
        buflist_setfpos(
            buf,
            win,
            if (*win).w_cursor.lnum == 1 as linenr_T {
                0 as linenr_T
            } else {
                (*win).w_cursor.lnum
            },
            (*win).w_cursor.col,
            true_0 != 0,
        );
    }
    let mut bufref: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    set_bufref(&raw mut bufref, buf);
    if (*buf).b_nwindows == 1 as ::core::ffi::c_int {
        (*buf).b_locked += 1;
        (*buf).b_locked_split += 1;
        if apply_autocmds(
            EVENT_BUFWINLEAVE,
            (*buf).b_fname,
            (*buf).b_fname,
            false_0 != 0,
            buf,
        ) as ::core::ffi::c_int
            != 0
            && !bufref_valid(&raw mut bufref)
        {
            emsg(gettext(&raw const e_auabort as *const ::core::ffi::c_char));
            return false_0 != 0;
        }
        (*buf).b_locked -= 1;
        (*buf).b_locked_split -= 1;
        if abort_if_last as ::core::ffi::c_int != 0
            && !win.is_null()
            && one_window(win, ::core::ptr::null_mut::<tabpage_T>()) as ::core::ffi::c_int != 0
        {
            emsg(gettext(&raw const e_auabort as *const ::core::ffi::c_char));
            return false_0 != 0;
        }
        if !unload_buf {
            (*buf).b_locked += 1;
            (*buf).b_locked_split += 1;
            if apply_autocmds(
                EVENT_BUFHIDDEN,
                (*buf).b_fname,
                (*buf).b_fname,
                false_0 != 0,
                buf,
            ) as ::core::ffi::c_int
                != 0
                && !bufref_valid(&raw mut bufref)
            {
                emsg(gettext(&raw const e_auabort as *const ::core::ffi::c_char));
                return false_0 != 0;
            }
            (*buf).b_locked -= 1;
            (*buf).b_locked_split -= 1;
            if abort_if_last as ::core::ffi::c_int != 0
                && !win.is_null()
                && one_window(win, ::core::ptr::null_mut::<tabpage_T>()) as ::core::ffi::c_int != 0
            {
                emsg(gettext(&raw const e_auabort as *const ::core::ffi::c_char));
                return false_0 != 0;
            }
        }
        if !ignore_abort && aborting() as ::core::ffi::c_int != 0 {
            return false_0 != 0;
        }
    }
    if is_curwin as ::core::ffi::c_int != 0
        && curwin.get() != the_curwin
        && win_valid_any_tab(the_curwin) as ::core::ffi::c_int != 0
    {
        block_autocmds();
        goto_tabpage_win(the_curtab, the_curwin);
        unblock_autocmds();
    }
    let mut nwindows: ::core::ffi::c_int = (*buf).b_nwindows;
    if (*buf).b_nwindows > 0 as ::core::ffi::c_int {
        (*buf).b_nwindows -= 1;
    }
    if diffopt_hiddenoff() as ::core::ffi::c_int != 0
        && !unload_buf
        && (*buf).b_nwindows == 0 as ::core::ffi::c_int
    {
        diff_buf_delete(buf);
    }
    if (*buf).b_nwindows > 0 as ::core::ffi::c_int || !unload_buf {
        return true_0 != 0;
    }
    if (*buf).b_ffname.is_null() {
        del_buf = true_0 != 0;
    }
    let mut is_curbuf: bool = buf == curbuf.get();
    if is_curbuf as ::core::ffi::c_int != 0 && VIsual_active.get() as ::core::ffi::c_int != 0 {
        end_visual_mode();
    }
    (*buf).b_nwindows = nwindows;
    buf_freeall(
        buf,
        (if del_buf as ::core::ffi::c_int != 0 {
            BFA_DEL as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) + (if wipe_buf as ::core::ffi::c_int != 0 {
            BFA_WIPE as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) + (if ignore_abort as ::core::ffi::c_int != 0 {
            BFA_IGNORE_ABORT as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }),
    );
    if !bufref_valid(&raw mut bufref) {
        return false_0 != 0;
    }
    if !ignore_abort && aborting() as ::core::ffi::c_int != 0 {
        return false_0 != 0;
    }
    if buf == curbuf.get() && !is_curbuf {
        return false_0 != 0;
    }
    let mut clear_w_buf: bool = false_0 != 0;
    if !win.is_null() && win_valid_any_tab(win) as ::core::ffi::c_int != 0 && (*win).w_buffer == buf
    {
        clear_w_buf = true_0 != 0;
    }
    if nwindows > 0 as ::core::ffi::c_int && (*buf).b_nwindows > 0 as ::core::ffi::c_int {
        (*buf).b_nwindows -= 1;
    }
    if wipe_buf as ::core::ffi::c_int != 0
        && (*buf).b_nwindows <= 0 as ::core::ffi::c_int
        && (!(*buf).b_prev.is_null() || !(*buf).b_next.is_null())
    {
        if clear_w_buf {
            (*win).w_buffer = ::core::ptr::null_mut::<buf_T>();
        }
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut wp: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                mark_forget_file(wp, (*buf).handle as ::core::ffi::c_int);
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        if (*buf).b_sfname != (*buf).b_ffname {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*buf).b_sfname as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_0;
            let _ = *ptr_;
        } else {
            (*buf).b_sfname = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut (*buf).b_ffname as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_0;
        let _ = *ptr__0;
        if (*buf).b_prev.is_null() {
            firstbuf.set((*buf).b_next);
        } else {
            (*(*buf).b_prev).b_next = (*buf).b_next;
        }
        if (*buf).b_next.is_null() {
            lastbuf.set((*buf).b_prev);
        } else {
            (*(*buf).b_next).b_prev = (*buf).b_prev;
        }
        free_buffer(buf);
    } else {
        if del_buf {
            free_buffer_stuff(
                buf,
                kBffClearWinInfo as ::core::ffi::c_int | kBffInitChangedtick as ::core::ffi::c_int,
            );
            (*buf).b_flags = BF_CHECK_RO | BF_NEVERLOADED;
            (*buf).b_p_initialized = false_0 != 0;
        }
        buf_clear_file(buf);
        if clear_w_buf {
            (*win).w_buffer = ::core::ptr::null_mut::<buf_T>();
        }
        if del_buf {
            (*buf).b_p_bl = false_0;
        }
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn buf_clear_file(mut buf: *mut buf_T) {
    (*buf).b_ml.ml_line_count = 1 as ::core::ffi::c_int as linenr_T;
    unchanged(buf, true_0 != 0, true_0 != 0);
    (*buf).b_p_eof = false_0;
    (*buf).b_start_eof = false_0;
    (*buf).b_p_eol = true_0;
    (*buf).b_start_eol = true_0;
    (*buf).b_p_bomb = false_0;
    (*buf).b_start_bomb = false_0;
    (*buf).b_ml.ml_mfp = ::core::ptr::null_mut::<memfile_T>();
    (*buf).b_ml.ml_flags = ML_EMPTY;
}
pub unsafe extern "C" fn buf_clear() {
    let mut line_count: linenr_T = (*curbuf.get()).b_ml.ml_line_count;
    extmark_free_all(curbuf.get());
    while (*curbuf.get()).b_ml.ml_flags & ML_EMPTY == 0 {
        ml_delete(1 as linenr_T);
    }
    deleted_lines_mark(1 as linenr_T, line_count as ::core::ffi::c_int);
}
pub unsafe extern "C" fn buf_freeall(mut buf: *mut buf_T, mut flags: ::core::ffi::c_int) {
    let mut is_curbuf: bool = buf == curbuf.get();
    let mut is_curwin: ::core::ffi::c_int =
        (!(*curwin.ptr()).is_null() && (*curwin.get()).w_buffer == buf) as ::core::ffi::c_int;
    let mut the_curwin: *mut win_T = curwin.get();
    let mut the_curtab: *mut tabpage_T = curtab.get();
    (*buf).b_locked += 1;
    (*buf).b_locked_split += 1;
    let mut bufref: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    set_bufref(&raw mut bufref, buf);
    if !(*buf).terminal.is_null() {
        buf_close_terminal(buf);
    }
    buf_updates_unload(buf, false_0 != 0);
    if !(*buf).b_ml.ml_mfp.is_null()
        && apply_autocmds(
            EVENT_BUFUNLOAD,
            (*buf).b_fname,
            (*buf).b_fname,
            false_0 != 0,
            buf,
        ) as ::core::ffi::c_int
            != 0
        && !bufref_valid(&raw mut bufref)
    {
        return;
    }
    if flags & BFA_DEL as ::core::ffi::c_int != 0
        && (*buf).b_p_bl != 0
        && apply_autocmds(
            EVENT_BUFDELETE,
            (*buf).b_fname,
            (*buf).b_fname,
            false_0 != 0,
            buf,
        ) as ::core::ffi::c_int
            != 0
        && !bufref_valid(&raw mut bufref)
    {
        return;
    }
    if flags & BFA_WIPE as ::core::ffi::c_int != 0
        && apply_autocmds(
            EVENT_BUFWIPEOUT,
            (*buf).b_fname,
            (*buf).b_fname,
            false_0 != 0,
            buf,
        ) as ::core::ffi::c_int
            != 0
        && !bufref_valid(&raw mut bufref)
    {
        return;
    }
    (*buf).b_locked -= 1;
    (*buf).b_locked_split -= 1;
    if is_curwin != 0
        && curwin.get() != the_curwin
        && win_valid_any_tab(the_curwin) as ::core::ffi::c_int != 0
    {
        block_autocmds();
        goto_tabpage_win(the_curtab, the_curwin);
        unblock_autocmds();
    }
    if flags & BFA_IGNORE_ABORT as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        && aborting() as ::core::ffi::c_int != 0
    {
        return;
    }
    if buf == curbuf.get() && !is_curbuf {
        return;
    }
    diff_buf_delete(buf);
    if !(*curwin.ptr()).is_null() && (*curwin.get()).w_buffer == buf {
        reset_synblock(curwin.get());
    }
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut win: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !win.is_null() {
            if (*win).w_buffer == buf {
                clearFolding(win);
            }
            win = (*win).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    if !(*buf).terminal.is_null() {
        block_autocmds();
        buf_close_terminal(buf);
        unblock_autocmds();
    }
    let mut count: linenr_T = (*buf).b_ml.ml_line_count;
    ml_close(buf, true_0);
    (*buf).b_ml.ml_line_count = 0 as ::core::ffi::c_int as linenr_T;
    if bt_nofilename(buf) as ::core::ffi::c_int != 0 && !exiting.get() {
        mark_adjust_buf(
            buf,
            1 as linenr_T,
            count,
            MAXLNUM as ::core::ffi::c_int as linenr_T,
            -count,
            false_0 != 0,
            kMarkAdjustNormal,
            kExtmarkNoUndo,
        );
    }
    if flags & BFA_KEEP_UNDO as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        u_clearallandblockfree(buf);
    }
    syntax_clear(&raw mut (*buf).b_s);
    (*buf).b_flags &= !BF_READERR;
}
unsafe extern "C" fn free_buffer(mut buf: *mut buf_T) {
    map_del_int_ptr_t(
        buffer_handles.ptr(),
        (*buf).handle as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
    );
    (*buf_free_count.ptr()) += 1;
    free_buffer_stuff(buf, kBffClearWinInfo as ::core::ffi::c_int);
    if (*(*buf).b_vars).dv_refcount > DO_NOT_FREE_CNT as ::core::ffi::c_int {
        tv_dict_add(
            (*buf).b_vars,
            tv_dict_item_copy(&raw mut (*buf).changedtick_di as *mut dictitem_T),
        );
    }
    unref_var_dict((*buf).b_vars);
    aubuflocal_remove(buf);
    xfree((*buf).additional_data as *mut ::core::ffi::c_void);
    xfree((*buf).b_prompt_text as *mut ::core::ffi::c_void);
    xfree((*buf).b_wininfo.items as *mut ::core::ffi::c_void);
    (*buf).b_wininfo.capacity = 0 as size_t;
    (*buf).b_wininfo.size = (*buf).b_wininfo.capacity;
    (*buf).b_wininfo.items = ::core::ptr::null_mut::<*mut WinInfo>();
    callback_free(&raw mut (*buf).b_prompt_callback);
    callback_free(&raw mut (*buf).b_prompt_interrupt);
    clear_fmark(&raw mut (*buf).b_last_cursor, 0 as Timestamp);
    clear_fmark(&raw mut (*buf).b_last_insert, 0 as Timestamp);
    clear_fmark(&raw mut (*buf).b_last_change, 0 as Timestamp);
    clear_fmark(&raw mut (*buf).b_prompt_start, 0 as Timestamp);
    let mut i: size_t = 0 as size_t;
    while i < NMARKS as size_t {
        free_fmark((*buf).b_namedm[i as usize]);
        i = i.wrapping_add(1);
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < (*buf).b_changelistlen {
        free_fmark((*buf).b_changelist[i_0 as usize]);
        i_0 += 1;
    }
    if autocmd_busy.get() {
        memset(
            &raw mut (*buf).b_namedm as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<[fmark_T; 26]>(),
        );
        memset(
            &raw mut (*buf).b_changelist as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<[fmark_T; 100]>(),
        );
        (*buf).b_next = au_pending_free_buf.get();
        au_pending_free_buf.set(buf);
    } else {
        xfree(buf as *mut ::core::ffi::c_void);
        if curbuf.get() == buf {
            curbuf.set(::core::ptr::null_mut::<buf_T>());
        }
    };
}
unsafe extern "C" fn clear_wininfo(mut buf: *mut buf_T) {
    let mut i: size_t = 0 as size_t;
    while i < (*buf).b_wininfo.size {
        free_wininfo(*(*buf).b_wininfo.items.offset(i as isize), buf);
        i = i.wrapping_add(1);
    }
    (*buf).b_wininfo.size = 0 as size_t;
}
unsafe extern "C" fn free_buffer_stuff(mut buf: *mut buf_T, mut free_flags: ::core::ffi::c_int) {
    if free_flags & kBffClearWinInfo as ::core::ffi::c_int != 0 {
        clear_wininfo(buf);
        free_buf_options(buf, true_0 != 0);
        ga_clear(&raw mut (*buf).b_s.b_langp);
    }
    let changedtick_hi: *mut hashitem_T = hash_find(
        &raw mut (*(*buf).b_vars).dv_hashtab,
        b"changedtick\0".as_ptr() as *const ::core::ffi::c_char,
    );
    '_c2rust_label: {
        if !changedtick_hi.is_null() {
        } else {
            __assert_fail(
                b"changedtick_hi != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/buffer.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1000 as ::core::ffi::c_uint,
                b"void free_buffer_stuff(buf_T *, int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    hash_remove(&raw mut (*(*buf).b_vars).dv_hashtab, changedtick_hi);
    vars_clear(&raw mut (*(*buf).b_vars).dv_hashtab);
    hash_init(&raw mut (*(*buf).b_vars).dv_hashtab);
    if free_flags & kBffInitChangedtick as ::core::ffi::c_int != 0 {
        buf_init_changedtick(buf);
    }
    uc_clear(&raw mut (*buf).b_ucmds);
    extmark_free_all(buf);
    map_clear_mode(
        buf,
        MAP_ALL_MODES as ::core::ffi::c_int,
        true_0 != 0,
        false_0 != 0,
    );
    map_clear_mode(
        buf,
        MAP_ALL_MODES as ::core::ffi::c_int,
        true_0 != 0,
        true_0 != 0,
    );
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*buf).b_start_fenc as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    let _ = *ptr_;
    buf_free_callbacks(buf);
}
pub unsafe extern "C" fn goto_buffer(
    mut eap: *mut exarg_T,
    mut start: ::core::ffi::c_int,
    mut dir: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
) {
    let save_sea: ::core::ffi::c_int = swap_exists_action.get();
    let mut skip_help_buf: bool = false;
    match (*eap).cmdidx as ::core::ffi::c_int {
        30 | 394 | 21 | 32 | 389 | 395 => {
            skip_help_buf = true_0 != 0;
        }
        _ => {
            skip_help_buf = false_0 != 0;
        }
    }
    let mut old_curbuf: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    set_bufref(&raw mut old_curbuf, curbuf.get());
    if swap_exists_action.get() == SEA_NONE {
        swap_exists_action.set(SEA_DIALOG);
    }
    do_buffer_ext(
        if *(*eap).cmd as ::core::ffi::c_int == 's' as ::core::ffi::c_int {
            DOBUF_SPLIT as ::core::ffi::c_int
        } else {
            DOBUF_GOTO as ::core::ffi::c_int
        },
        start,
        dir,
        count,
        (if (*eap).forceit != 0 {
            DOBUF_FORCEIT as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | (if skip_help_buf as ::core::ffi::c_int != 0 {
            DOBUF_SKIPHELP as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }),
    );
    if swap_exists_action.get() == SEA_QUIT
        && *(*eap).cmd as ::core::ffi::c_int == 's' as ::core::ffi::c_int
    {
        let mut cs: cleanup_T = cleanup_T {
            pending: 0,
            exception: ::core::ptr::null_mut::<except_T>(),
        };
        enter_cleanup(&raw mut cs);
        win_close(curwin.get(), true_0 != 0, false_0 != 0);
        swap_exists_action.set(save_sea);
        swap_exists_did_quit.set(true_0 != 0);
        leave_cleanup(&raw mut cs);
    } else {
        handle_swap_exists(&raw mut old_curbuf);
    };
}
pub unsafe extern "C" fn handle_swap_exists(mut old_curbuf: *mut bufref_T) {
    let mut cs: cleanup_T = cleanup_T {
        pending: 0,
        exception: ::core::ptr::null_mut::<except_T>(),
    };
    let mut old_tw: OptInt = (*curbuf.get()).b_p_tw;
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    if swap_exists_action.get() == SEA_QUIT {
        enter_cleanup(&raw mut cs);
        swap_exists_action.set(SEA_NONE);
        swap_exists_did_quit.set(true_0 != 0);
        close_buffer(
            curwin.get(),
            curbuf.get(),
            DOBUF_UNLOAD as ::core::ffi::c_int,
            false_0 != 0,
            false_0 != 0,
        );
        if old_curbuf.is_null() || !bufref_valid(old_curbuf) || (*old_curbuf).br_buf == curbuf.get()
        {
            block_autocmds();
            buf = buflist_new(
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                1 as linenr_T,
                BLN_CURBUF as ::core::ffi::c_int | BLN_LISTED as ::core::ffi::c_int,
            );
            unblock_autocmds();
        } else {
            buf = (*old_curbuf).br_buf;
        }
        if !buf.is_null() {
            enter_buffer(buf);
            if old_tw != (*curbuf.get()).b_p_tw {
                check_colorcolumn(::core::ptr::null_mut::<::core::ffi::c_char>(), curwin.get());
            }
        }
        leave_cleanup(&raw mut cs);
    } else if swap_exists_action.get() == SEA_RECOVER {
        enter_cleanup(&raw mut cs);
        msg_scroll.set(true_0);
        ml_recover(false_0 != 0);
        msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        cmdline_row.set(msg_row.get());
        do_modelines(0 as ::core::ffi::c_int);
        leave_cleanup(&raw mut cs);
    }
    swap_exists_action.set(SEA_NONE);
}
pub unsafe extern "C" fn do_bufdel(
    mut command: ::core::ffi::c_int,
    mut arg: *mut ::core::ffi::c_char,
    mut addr_count: ::core::ffi::c_int,
    mut start_bnr: ::core::ffi::c_int,
    mut end_bnr: ::core::ffi::c_int,
    mut forceit: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut do_current: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut deleted: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut errormsg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut bnr: ::core::ffi::c_int = 0;
    if addr_count == 0 as ::core::ffi::c_int {
        do_buffer(
            command,
            DOBUF_CURRENT as ::core::ffi::c_int,
            FORWARD as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            forceit,
        );
    } else {
        if addr_count == 2 as ::core::ffi::c_int {
            if *arg != 0 {
                return ex_errmsg(&raw const e_trailing_arg as *const ::core::ffi::c_char, arg);
            }
            bnr = start_bnr;
        } else {
            bnr = end_bnr;
        }
        while !got_int.get() {
            if bnr == (*curbuf.get()).handle {
                do_current = bnr;
            } else if do_buffer(
                command,
                DOBUF_FIRST as ::core::ffi::c_int,
                FORWARD as ::core::ffi::c_int,
                bnr,
                forceit,
            ) == OK
            {
                deleted += 1;
            }
            if addr_count == 2 as ::core::ffi::c_int {
                bnr += 1;
                if bnr > end_bnr {
                    break;
                }
            } else {
                arg = skipwhite(arg);
                if *arg as ::core::ffi::c_int == NUL {
                    break;
                }
                if !ascii_isdigit(*arg as ::core::ffi::c_int) {
                    let mut p: *mut ::core::ffi::c_char = skiptowhite_esc(arg);
                    bnr = buflist_findpat(
                        arg,
                        p,
                        command == DOBUF_WIPE as ::core::ffi::c_int,
                        false_0 != 0,
                        false_0 != 0,
                    );
                    if bnr < 0 as ::core::ffi::c_int {
                        break;
                    }
                    arg = p;
                } else {
                    bnr = getdigits_int(&raw mut arg, false_0 != 0, 0 as ::core::ffi::c_int);
                }
            }
            os_breakcheck();
        }
        if !got_int.get()
            && do_current != 0
            && do_buffer(
                command,
                DOBUF_FIRST as ::core::ffi::c_int,
                FORWARD as ::core::ffi::c_int,
                do_current,
                forceit,
            ) == OK
        {
            deleted += 1;
        }
        if deleted == 0 as ::core::ffi::c_int {
            if command == DOBUF_UNLOAD as ::core::ffi::c_int {
                xstrlcpy(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    gettext(
                        b"E515: No buffers were unloaded\0".as_ptr() as *const ::core::ffi::c_char
                    ),
                    IOSIZE as size_t,
                );
            } else if command == DOBUF_DEL as ::core::ffi::c_int {
                xstrlcpy(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    gettext(
                        b"E516: No buffers were deleted\0".as_ptr() as *const ::core::ffi::c_char
                    ),
                    IOSIZE as size_t,
                );
            } else {
                xstrlcpy(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    gettext(
                        b"E517: No buffers were wiped out\0".as_ptr() as *const ::core::ffi::c_char
                    ),
                    IOSIZE as size_t,
                );
            }
            errormsg = IObuff.ptr() as *mut ::core::ffi::c_char;
        } else if deleted as OptInt >= p_report.get() {
            if command == DOBUF_UNLOAD as ::core::ffi::c_int {
                smsg(
                    0 as ::core::ffi::c_int,
                    ngettext(
                        b"%d buffer unloaded\0".as_ptr() as *const ::core::ffi::c_char,
                        b"%d buffers unloaded\0".as_ptr() as *const ::core::ffi::c_char,
                        deleted as ::core::ffi::c_ulong,
                    ),
                    deleted,
                );
            } else if command == DOBUF_DEL as ::core::ffi::c_int {
                smsg(
                    0 as ::core::ffi::c_int,
                    ngettext(
                        b"%d buffer deleted\0".as_ptr() as *const ::core::ffi::c_char,
                        b"%d buffers deleted\0".as_ptr() as *const ::core::ffi::c_char,
                        deleted as ::core::ffi::c_ulong,
                    ),
                    deleted,
                );
            } else {
                smsg(
                    0 as ::core::ffi::c_int,
                    ngettext(
                        b"%d buffer wiped out\0".as_ptr() as *const ::core::ffi::c_char,
                        b"%d buffers wiped out\0".as_ptr() as *const ::core::ffi::c_char,
                        deleted as ::core::ffi::c_ulong,
                    ),
                    deleted,
                );
            }
        }
    }
    return errormsg;
}
unsafe extern "C" fn empty_curbuf(
    mut close_others: bool,
    mut forceit: ::core::ffi::c_int,
    mut action: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut buf: *mut buf_T = curbuf.get();
    if action == DOBUF_UNLOAD as ::core::ffi::c_int {
        emsg(gettext(
            b"E90: Cannot unload last buffer\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return FAIL;
    }
    let mut bufref: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    set_bufref(&raw mut bufref, buf);
    if close_others {
        let mut can_close_all_others: bool = true_0 != 0;
        if (*curwin.get()).w_floating {
            can_close_all_others = false_0 != 0;
            let mut wp: *mut win_T = firstwin.get();
            while !(*wp).w_floating {
                if (*wp).w_buffer != curbuf.get() {
                    can_close_all_others = true_0 != 0;
                    break;
                } else {
                    wp = (*wp).w_next;
                }
            }
        }
        close_windows(buf, can_close_all_others);
    }
    setpcmark();
    let mut retval: ::core::ffi::c_int = do_ecmd(
        0 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<exarg_T>(),
        ECMD_ONE as ::core::ffi::c_int as linenr_T,
        if forceit != 0 {
            ECMD_FORCEIT as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        },
        curwin.get(),
    );
    if buf != curbuf.get()
        && bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
        && (*buf).b_nwindows == 0 as ::core::ffi::c_int
    {
        close_buffer(
            ::core::ptr::null_mut::<win_T>(),
            buf,
            action,
            false_0 != 0,
            false_0 != 0,
        );
    }
    if !close_others {
        need_fileinfo.set(false_0 != 0);
    }
    return retval;
}
unsafe extern "C" fn do_buffer_ext(
    mut action: ::core::ffi::c_int,
    mut start: ::core::ffi::c_int,
    mut dir: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut bp: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut update_jumplist: bool = true_0 != 0;
    let mut unload: bool = action == DOBUF_UNLOAD as ::core::ffi::c_int
        || action == DOBUF_DEL as ::core::ffi::c_int
        || action == DOBUF_WIPE as ::core::ffi::c_int;
    match start {
        1 => {
            buf = firstbuf.get();
        }
        2 => {
            buf = lastbuf.get();
        }
        _ => {
            buf = curbuf.get();
        }
    }
    if start == DOBUF_MOD as ::core::ffi::c_int {
        loop {
            let c2rust_fresh2 = count;
            count = count - 1;
            if c2rust_fresh2 <= 0 as ::core::ffi::c_int {
                break;
            }
            loop {
                buf = (*buf).b_next;
                if buf.is_null() {
                    buf = firstbuf.get();
                }
                if !(buf != curbuf.get() && !bufIsChanged(buf)) {
                    break;
                }
            }
        }
        if !bufIsChanged(buf) {
            emsg(gettext(
                b"E84: No modified buffer found\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return FAIL;
        }
    } else if start == DOBUF_FIRST as ::core::ffi::c_int && count != 0 {
        while !buf.is_null() && (*buf).handle != count {
            buf = (*buf).b_next;
        }
    } else {
        let help_only: bool = flags & DOBUF_SKIPHELP as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
            && (*buf).b_help as ::core::ffi::c_int != 0;
        bp = ::core::ptr::null_mut::<buf_T>();
        while count > 0 as ::core::ffi::c_int
            || bp != buf
                && !unload
                && (if help_only as ::core::ffi::c_int != 0 {
                    (*buf).b_help as ::core::ffi::c_int
                } else {
                    (*buf).b_p_bl
                }) == 0
        {
            if bp.is_null() {
                bp = buf;
            }
            buf = if dir == FORWARD as ::core::ffi::c_int {
                if !(*buf).b_next.is_null() {
                    (*buf).b_next
                } else {
                    firstbuf.get()
                }
            } else if !(*buf).b_prev.is_null() {
                (*buf).b_prev
            } else {
                lastbuf.get()
            };
            if unload as ::core::ffi::c_int != 0
                || (if help_only as ::core::ffi::c_int != 0 {
                    (*buf).b_help as ::core::ffi::c_int
                } else {
                    ((*buf).b_p_bl != 0
                        && (flags & DOBUF_SKIPHELP as ::core::ffi::c_int
                            == 0 as ::core::ffi::c_int
                            || !(*buf).b_help)) as ::core::ffi::c_int
                }) != 0
            {
                count -= 1;
                bp = ::core::ptr::null_mut::<buf_T>();
            }
            if bp == buf {
                emsg(gettext(
                    b"E85: There is no listed buffer\0".as_ptr() as *const ::core::ffi::c_char
                ));
                return FAIL;
            }
        }
    }
    if buf.is_null() {
        if start == DOBUF_FIRST as ::core::ffi::c_int {
            if !unload {
                semsg(
                    gettext(&raw const e_nobufnr as *const ::core::ffi::c_char),
                    count as int64_t,
                );
            }
        } else if dir == FORWARD as ::core::ffi::c_int {
            emsg(gettext(
                b"E87: Cannot go beyond last buffer\0".as_ptr() as *const ::core::ffi::c_char
            ));
        } else {
            emsg(gettext(
                b"E88: Cannot go before first buffer\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
        return FAIL;
    }
    if action == DOBUF_GOTO as ::core::ffi::c_int
        && buf != curbuf.get()
        && !check_can_set_curbuf_forceit(
            (flags & DOBUF_FORCEIT as ::core::ffi::c_int != 0 as ::core::ffi::c_int)
                as ::core::ffi::c_int,
        )
    {
        return FAIL;
    }
    if (action == DOBUF_GOTO as ::core::ffi::c_int || action == DOBUF_SPLIT as ::core::ffi::c_int)
        && (*buf).b_flags & BF_DUMMY != 0
    {
        semsg(
            gettext(&raw const e_nobufnr as *const ::core::ffi::c_char),
            count,
        );
        return FAIL;
    }
    if unload {
        let mut forward: ::core::ffi::c_int = 0;
        let mut bufref: bufref_T = bufref_T {
            br_buf: ::core::ptr::null_mut::<buf_T>(),
            br_fnum: 0,
            br_buf_free_count: 0,
        };
        if !can_unload_buffer(buf) {
            return FAIL;
        }
        set_bufref(&raw mut bufref, buf);
        if action != DOBUF_WIPE as ::core::ffi::c_int
            && (*buf).b_ml.ml_mfp.is_null()
            && (*buf).b_p_bl == 0
        {
            return FAIL;
        }
        if flags & DOBUF_FORCEIT as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            && bufIsChanged(buf) as ::core::ffi::c_int != 0
        {
            if (p_confirm.get() != 0
                || (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0)
                && p_write.get() != 0
            {
                dialog_changed(buf, false_0 != 0);
                if !bufref_valid(&raw mut bufref) {
                    return FAIL;
                }
                if bufIsChanged(buf) {
                    return FAIL;
                }
            } else {
                semsg(
                    gettext(
                        &raw const e_no_write_since_last_change_for_buffer_nr_add_bang_to_override
                            as *const ::core::ffi::c_char,
                    ),
                    (*buf).handle,
                );
                return FAIL;
            }
        }
        if flags & DOBUF_FORCEIT as ::core::ffi::c_int == 0
            && !(*buf).terminal.is_null()
            && terminal_running((*buf).terminal) as ::core::ffi::c_int != 0
        {
            if p_confirm.get() != 0
                || (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0
            {
                if !dialog_close_terminal(buf) {
                    return FAIL;
                }
            } else {
                semsg(
                    gettext(b"E89: %s will be killed (add ! to override)\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    (*buf).b_fname,
                );
                return FAIL;
            }
        }
        let mut buf_fnum: ::core::ffi::c_int = (*buf).handle as ::core::ffi::c_int;
        if buf == curbuf.get() && VIsual_active.get() as ::core::ffi::c_int != 0 {
            end_visual_mode();
        }
        bp = ::core::ptr::null_mut::<buf_T>();
        let mut bp2: *mut buf_T = firstbuf.get();
        while !bp2.is_null() {
            if (*bp2).b_p_bl != 0 && bp2 != buf {
                bp = bp2;
                break;
            } else {
                bp2 = (*bp2).b_next;
            }
        }
        if bp.is_null() && buf == curbuf.get() {
            return empty_curbuf(
                true_0 != 0,
                flags & DOBUF_FORCEIT as ::core::ffi::c_int,
                action,
            );
        }
        while buf == curbuf.get()
            && !(win_locked(curwin.get()) != 0
                || (*(*curwin.get()).w_buffer).b_locked > 0 as ::core::ffi::c_int)
            && (is_aucmd_win(lastwin.get()) as ::core::ffi::c_int != 0
                || !last_window(curwin.get()))
        {
            if win_close(curwin.get(), false_0 != 0, false_0 != 0) == FAIL {
                break;
            }
        }
        if buf != curbuf.get() {
            if jop_flags.get() & kOptJopFlagClean as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            {
                mark_jumplist_forget_file(curwin.get(), buf_fnum);
            }
            close_windows(buf, false_0 != 0);
            if buf != curbuf.get()
                && bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
                && (*buf).b_nwindows <= 0 as ::core::ffi::c_int
            {
                close_buffer(
                    ::core::ptr::null_mut::<win_T>(),
                    buf,
                    action,
                    false_0 != 0,
                    false_0 != 0,
                );
            }
            return OK;
        }
        buf = ::core::ptr::null_mut::<buf_T>();
        bp = ::core::ptr::null_mut::<buf_T>();
        if !(*au_new_curbuf.ptr()).br_buf.is_null()
            && bufref_valid(au_new_curbuf.ptr()) as ::core::ffi::c_int != 0
            && (*(*au_new_curbuf.ptr()).br_buf).b_locked_split == 0
        {
            buf = (*au_new_curbuf.ptr()).br_buf;
        } else if (*curwin.get()).w_jumplistlen > 0 as ::core::ffi::c_int {
            if jop_flags.get() & kOptJopFlagClean as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            {
                mark_jumplist_forget_file(curwin.get(), buf_fnum);
            }
            if (*curwin.get()).w_jumplistlen > 0 as ::core::ffi::c_int {
                let mut jumpidx: ::core::ffi::c_int = (*curwin.get()).w_jumplistidx;
                if jop_flags.get() & kOptJopFlagClean as ::core::ffi::c_int as ::core::ffi::c_uint
                    != 0
                {
                    if jumpidx == (*curwin.get()).w_jumplistlen {
                        (*curwin.get()).w_jumplistidx =
                            (*curwin.get()).w_jumplistlen - 1 as ::core::ffi::c_int;
                        jumpidx = (*curwin.get()).w_jumplistidx;
                    }
                } else {
                    jumpidx -= 1;
                    if jumpidx < 0 as ::core::ffi::c_int {
                        jumpidx = (*curwin.get()).w_jumplistlen - 1 as ::core::ffi::c_int;
                    }
                }
                forward = jumpidx;
                while jop_flags.get()
                    & kOptJopFlagClean as ::core::ffi::c_int as ::core::ffi::c_uint
                    != 0
                    || jumpidx != (*curwin.get()).w_jumplistidx
                {
                    buf = buflist_findnr((*curwin.get()).w_jumplist[jumpidx as usize].fmark.fnum);
                    if !buf.is_null() {
                        if buf == curbuf.get()
                            || (*buf).b_p_bl == 0
                            || bt_quickfix(buf) as ::core::ffi::c_int != 0
                            || (*buf).b_locked_split != 0
                        {
                            buf = ::core::ptr::null_mut::<buf_T>();
                        } else if (*buf).b_ml.ml_mfp.is_null() {
                            if bp.is_null() {
                                bp = buf;
                            }
                            buf = ::core::ptr::null_mut::<buf_T>();
                        }
                    }
                    if !buf.is_null() {
                        if jop_flags.get()
                            & kOptJopFlagClean as ::core::ffi::c_int as ::core::ffi::c_uint
                            != 0
                        {
                            (*curwin.get()).w_jumplistidx = jumpidx;
                            update_jumplist = false_0 != 0;
                        }
                        break;
                    } else {
                        if jumpidx == 0
                            && (*curwin.get()).w_jumplistidx == (*curwin.get()).w_jumplistlen
                        {
                            break;
                        }
                        jumpidx -= 1;
                        if jumpidx < 0 as ::core::ffi::c_int {
                            jumpidx = (*curwin.get()).w_jumplistlen - 1 as ::core::ffi::c_int;
                        }
                        if jumpidx == forward {
                            break;
                        }
                    }
                }
            }
        }
        if buf.is_null() {
            forward = true_0;
            buf = (*curbuf.get()).b_next;
            loop {
                if buf.is_null() {
                    if forward == 0 {
                        break;
                    }
                    buf = (*curbuf.get()).b_prev;
                    forward = false_0;
                } else {
                    if (*buf).b_help as ::core::ffi::c_int
                        == (*curbuf.get()).b_help as ::core::ffi::c_int
                        && (*buf).b_p_bl != 0
                        && !bt_quickfix(buf)
                        && (*buf).b_locked_split == 0
                    {
                        if !(*buf).b_ml.ml_mfp.is_null() {
                            break;
                        }
                        if bp.is_null() {
                            bp = buf;
                        }
                    }
                    buf = if forward != 0 {
                        (*buf).b_next
                    } else {
                        (*buf).b_prev
                    };
                }
            }
        }
        if buf.is_null() {
            buf = bp;
        }
        if buf.is_null() {
            let mut buf2: *mut buf_T = firstbuf.get();
            while !buf2.is_null() {
                if (*buf2).b_p_bl != 0
                    && buf2 != curbuf.get()
                    && !bt_quickfix(buf2)
                    && (*buf2).b_locked_split == 0
                {
                    buf = buf2;
                    break;
                } else {
                    buf2 = (*buf2).b_next;
                }
            }
        }
        if buf.is_null() {
            buf = if !(*curbuf.get()).b_next.is_null() {
                (*curbuf.get()).b_next
            } else {
                (*curbuf.get()).b_prev
            };
            if bt_quickfix(buf) as ::core::ffi::c_int != 0
                || buf != curbuf.get() && (*buf).b_locked_split != 0
            {
                buf = ::core::ptr::null_mut::<buf_T>();
            }
        }
    }
    if buf.is_null() {
        return empty_curbuf(
            false_0 != 0,
            flags & DOBUF_FORCEIT as ::core::ffi::c_int,
            action,
        );
    }
    if action == DOBUF_SPLIT as ::core::ffi::c_int && !swbuf_goto_win_with_buf(buf).is_null() {
        return OK;
    }
    if buf != curbuf.get() && (*buf).b_locked_split != 0 {
        emsg(gettext(
            &raw const e_cannot_switch_to_a_closing_buffer as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    if action == DOBUF_SPLIT as ::core::ffi::c_int
        && win_split(0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) == FAIL
    {
        return FAIL;
    }
    if buf == curbuf.get() {
        return OK;
    }
    if action == DOBUF_GOTO as ::core::ffi::c_int
        && !can_abandon(
            curbuf.get(),
            flags & DOBUF_FORCEIT as ::core::ffi::c_int != 0,
        )
    {
        if (p_confirm.get() != 0
            || (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0)
            && p_write.get() != 0
        {
            let mut bufref_0: bufref_T = bufref_T {
                br_buf: ::core::ptr::null_mut::<buf_T>(),
                br_fnum: 0,
                br_buf_free_count: 0,
            };
            set_bufref(&raw mut bufref_0, buf);
            dialog_changed(curbuf.get(), false_0 != 0);
            if !bufref_valid(&raw mut bufref_0) {
                return FAIL;
            }
        }
        if bufIsChanged(curbuf.get()) {
            no_write_message();
            return FAIL;
        }
    }
    set_curbuf(buf, action, update_jumplist);
    if action == DOBUF_SPLIT as ::core::ffi::c_int {
        (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
        (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
    }
    if aborting() {
        return FAIL;
    }
    return OK;
}
pub unsafe extern "C" fn do_buffer(
    mut action: ::core::ffi::c_int,
    mut start: ::core::ffi::c_int,
    mut dir: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut forceit: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return do_buffer_ext(
        action,
        start,
        dir,
        count,
        if forceit != 0 {
            DOBUF_FORCEIT as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        },
    );
}
pub unsafe extern "C" fn set_curbuf(
    mut buf: *mut buf_T,
    mut action: ::core::ffi::c_int,
    mut update_jumplist: bool,
) {
    let mut prevbuf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut unload: ::core::ffi::c_int = (action == DOBUF_UNLOAD as ::core::ffi::c_int
        || action == DOBUF_DEL as ::core::ffi::c_int
        || action == DOBUF_WIPE as ::core::ffi::c_int)
        as ::core::ffi::c_int;
    let mut old_tw: OptInt = (*curbuf.get()).b_p_tw;
    let last_winid: ::core::ffi::c_int = get_last_winid();
    if update_jumplist {
        setpcmark();
    }
    if (*cmdmod.ptr()).cmod_flags & CMOD_KEEPALT as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        (*curwin.get()).w_alt_fnum = (*curbuf.get()).handle as ::core::ffi::c_int;
    }
    buflist_altfpos(curwin.get());
    VIsual_reselect.set(false_0);
    prevbuf = curbuf.get();
    let mut newbufref: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    let mut prevbufref: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    set_bufref(&raw mut prevbufref, prevbuf);
    set_bufref(&raw mut newbufref, buf);
    let prev_nwindows: ::core::ffi::c_int = (*prevbuf).b_nwindows;
    if !apply_autocmds(
        EVENT_BUFLEAVE,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
    ) || bufref_valid(&raw mut prevbufref) as ::core::ffi::c_int != 0
        && bufref_valid(&raw mut newbufref) as ::core::ffi::c_int != 0
        && !aborting()
    {
        if prevbuf == (*curwin.get()).w_buffer {
            reset_synblock(curwin.get());
        }
        if unload != 0
            || prev_nwindows <= 1 as ::core::ffi::c_int
                && last_winid != get_last_winid()
                && action == DOBUF_GOTO as ::core::ffi::c_int
                && !buf_hide(prevbuf)
        {
            close_windows(prevbuf, false_0 != 0);
        }
        if bufref_valid(&raw mut prevbufref) as ::core::ffi::c_int != 0 && !aborting() {
            let mut previouswin: *mut win_T = curwin.get();
            if prevbuf == curbuf.get()
                && (State.get() & MODE_INSERT as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                    || (*curbuf.get()).b_nwindows <= 1 as ::core::ffi::c_int)
            {
                u_sync(false_0 != 0);
            }
            close_buffer(
                if prevbuf == (*curwin.get()).w_buffer {
                    curwin.get()
                } else {
                    ::core::ptr::null_mut::<win_T>()
                },
                prevbuf,
                if unload != 0 {
                    action
                } else if action == DOBUF_GOTO as ::core::ffi::c_int
                    && !buf_hide(prevbuf)
                    && !bufIsChanged(prevbuf)
                {
                    DOBUF_UNLOAD as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                },
                false_0 != 0,
                false_0 != 0,
            );
            if curwin.get() != previouswin && win_valid(previouswin) as ::core::ffi::c_int != 0 {
                curwin.set(previouswin);
            }
        }
    }
    let mut valid: bool = buf_valid(buf);
    if valid as ::core::ffi::c_int != 0 && buf != curbuf.get() && !aborting()
        || (*curwin.get()).w_buffer.is_null()
    {
        if !(*curbuf.ptr()).is_null() && prevbuf != curbuf.get() {
            (*curbuf.get()).b_nwindows -= 1;
        }
        enter_buffer(if valid as ::core::ffi::c_int != 0 {
            buf
        } else {
            lastbuf.get()
        });
        if old_tw != (*curbuf.get()).b_p_tw {
            check_colorcolumn(::core::ptr::null_mut::<::core::ffi::c_char>(), curwin.get());
        }
    }
    if bufref_valid(&raw mut prevbufref) as ::core::ffi::c_int != 0
        && !(*prevbuf).terminal.is_null()
    {
        terminal_check_size((*prevbuf).terminal);
    }
}
unsafe extern "C" fn enter_buffer(mut buf: *mut buf_T) {
    if VIsual_active.get() {
        end_visual_mode();
    }
    (*curwin.get()).w_buffer = buf;
    curbuf.set(buf);
    (*curbuf.get()).b_nwindows += 1;
    buf_copy_options(
        buf,
        BCO_ENTER as ::core::ffi::c_int | BCO_NOHELP as ::core::ffi::c_int,
    );
    if !(*buf).b_help {
        get_winopts(buf);
    } else {
        clearFolding(curwin.get());
    }
    foldUpdateAll(curwin.get());
    if (*curwin.get()).w_onebuf_opt.wo_diff != 0 {
        diff_buf_add(curbuf.get());
    }
    (*curwin.get()).w_s = &raw mut (*curbuf.get()).b_s;
    (*curwin.get()).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    (*curwin.get()).w_set_curswant = true_0;
    (*curwin.get()).w_topline_was_set = false_0 as ::core::ffi::c_char;
    (*curwin.get()).w_valid = 0 as ::core::ffi::c_int;
    if (*curbuf.get()).b_ml.ml_mfp.is_null() {
        if *(*curbuf.get()).b_p_ft as ::core::ffi::c_int == NUL {
            (*curbuf.get()).b_did_filetype = false_0 != 0;
        }
        open_buffer(
            false_0 != 0,
            ::core::ptr::null_mut::<exarg_T>(),
            0 as ::core::ffi::c_int,
        );
    } else {
        if msg_silent.get() == 0 && !shortmess(SHM_FILEINFO as ::core::ffi::c_int) {
            need_fileinfo.set(true_0 != 0);
        }
        buf_check_timestamp(curbuf.get());
        (*curwin.get()).w_topline = 1 as ::core::ffi::c_int as linenr_T;
        (*curwin.get()).w_topfill = 0 as ::core::ffi::c_int;
        apply_autocmds(
            EVENT_BUFENTER,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        apply_autocmds(
            EVENT_BUFWINENTER,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
    }
    if (*curwin.get()).w_cursor.lnum == 1 as linenr_T
        && inindent(0 as ::core::ffi::c_int) as ::core::ffi::c_int != 0
    {
        buflist_getfpos();
    }
    check_arg_idx(curwin.get());
    maketitle();
    if (*curwin.get()).w_topline == 1 as linenr_T && (*curwin.get()).w_topline_was_set == 0 {
        scroll_cursor_halfway(curwin.get(), false_0 != 0, false_0 != 0);
    }
    do_autochdir();
    if (*curbuf.get()).b_kmap_state as ::core::ffi::c_int & KEYMAP_INIT != 0 {
        keymap_init();
    }
    if !(*curbuf.get()).b_help
        && (*curwin.get()).w_onebuf_opt.wo_spell != 0
        && *(*(*curwin.get()).w_s).b_p_spl as ::core::ffi::c_int != NUL
    {
        parse_spelllang(curwin.get());
    }
    (*curbuf.get()).b_last_used = time(::core::ptr::null_mut::<time_t>());
    if !(*curbuf.get()).terminal.is_null() {
        terminal_check_size((*curbuf.get()).terminal);
    }
    redraw_later(curwin.get(), UPD_NOT_VALID as ::core::ffi::c_int);
}
pub unsafe extern "C" fn do_autochdir() {
    if p_acd.get() != 0 {
        if starting.get() == 0 as ::core::ffi::c_int
            && !(*curbuf.get()).b_ffname.is_null()
            && vim_chdirfile((*curbuf.get()).b_ffname, kCdCauseAuto) == OK
        {
            last_chdir_reason
                .set(b"autochdir\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char);
            shorten_fnames(true_0);
        }
    }
}
pub unsafe extern "C" fn no_write_message_buf(mut buf: *mut buf_T) {
    if !(*buf).terminal.is_null()
        && channel_job_running((*buf).b_p_channel as uint64_t) as ::core::ffi::c_int != 0
    {
        emsg(gettext(
            &raw const e_job_still_running_add_bang_to_end_the_job as *const ::core::ffi::c_char,
        ));
    } else {
        semsg(
            gettext(
                &raw const e_no_write_since_last_change_for_buffer_nr_add_bang_to_override
                    as *const ::core::ffi::c_char,
            ),
            (*buf).handle,
        );
    };
}
pub unsafe extern "C" fn no_write_message() {
    if !(*curbuf.get()).terminal.is_null()
        && channel_job_running((*curbuf.get()).b_p_channel as uint64_t) as ::core::ffi::c_int != 0
    {
        emsg(gettext(
            &raw const e_job_still_running_add_bang_to_end_the_job as *const ::core::ffi::c_char,
        ));
    } else {
        emsg(gettext(
            &raw const e_no_write_since_last_change_add_bang_to_override
                as *const ::core::ffi::c_char,
        ));
    };
}
pub unsafe extern "C" fn no_write_message_nobang(buf: *const buf_T) {
    if !(*buf).terminal.is_null()
        && channel_job_running((*buf).b_p_channel as uint64_t) as ::core::ffi::c_int != 0
    {
        emsg(gettext(
            &raw const e_job_still_running as *const ::core::ffi::c_char,
        ));
    } else {
        emsg(gettext(
            &raw const e_no_write_since_last_change as *const ::core::ffi::c_char,
        ));
    };
}
#[inline(always)]
unsafe extern "C" fn buf_init_changedtick(buf: *mut buf_T) {
    (*buf).changedtick_di = ChangedtickDictItem {
        di_tv: typval_T {
            v_type: VAR_NUMBER,
            v_lock: VAR_FIXED,
            vval: typval_vval_union {
                v_number: buf_get_changedtick(buf),
            },
        },
        di_flags: (DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int)
            as uint8_t,
        di_key: ::core::mem::transmute::<[u8; 12], [::core::ffi::c_char; 12]>(*b"changedtick\0"),
    };
    tv_dict_add(
        (*buf).b_vars,
        &raw mut (*buf).changedtick_di as *mut dictitem_T,
    );
}
#[no_mangle]
pub unsafe extern "C" fn buflist_new(
    mut ffname_arg: *mut ::core::ffi::c_char,
    mut sfname_arg: *mut ::core::ffi::c_char,
    mut lnum: linenr_T,
    mut flags: ::core::ffi::c_int,
) -> *mut buf_T {
    let mut ffname: *mut ::core::ffi::c_char = ffname_arg;
    let mut sfname: *mut ::core::ffi::c_char = sfname_arg;
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    fname_expand(curbuf.get(), &raw mut ffname, &raw mut sfname);
    let mut file_id: FileID = FileID {
        inode: 0,
        device_id: 0,
    };
    let mut file_id_valid: bool =
        !sfname.is_null() && os_fileid(sfname, &raw mut file_id) as ::core::ffi::c_int != 0;
    if !ffname.is_null()
        && flags & (BLN_DUMMY as ::core::ffi::c_int | BLN_NEW as ::core::ffi::c_int) == 0
        && {
            buf = buflist_findname_file_id(ffname, &raw mut file_id, file_id_valid);
            !buf.is_null()
        }
    {
        xfree(ffname as *mut ::core::ffi::c_void);
        if lnum != 0 as linenr_T {
            buflist_setfpos(
                buf,
                if flags & BLN_NOCURWIN as ::core::ffi::c_int != 0 {
                    ::core::ptr::null_mut::<win_T>()
                } else {
                    curwin.get()
                },
                lnum,
                0 as colnr_T,
                false_0 != 0,
            );
        }
        if flags & BLN_NOOPT as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            buf_copy_options(buf, 0 as ::core::ffi::c_int);
        }
        if flags & BLN_LISTED as ::core::ffi::c_int != 0 && (*buf).b_p_bl == 0 {
            (*buf).b_p_bl = true_0;
            let mut bufref: bufref_T = bufref_T {
                br_buf: ::core::ptr::null_mut::<buf_T>(),
                br_fnum: 0,
                br_buf_free_count: 0,
            };
            set_bufref(&raw mut bufref, buf);
            if flags & BLN_DUMMY as ::core::ffi::c_int == 0 {
                if apply_autocmds(
                    EVENT_BUFADD,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    false_0 != 0,
                    buf,
                ) as ::core::ffi::c_int
                    != 0
                    && !bufref_valid(&raw mut bufref)
                {
                    return ::core::ptr::null_mut::<buf_T>();
                }
            }
        }
        return buf;
    }
    buf = ::core::ptr::null_mut::<buf_T>();
    if flags & BLN_CURBUF as ::core::ffi::c_int != 0 && curbuf_reusable() as ::core::ffi::c_int != 0
    {
        let mut bufref_0: bufref_T = bufref_T {
            br_buf: ::core::ptr::null_mut::<buf_T>(),
            br_fnum: 0,
            br_buf_free_count: 0,
        };
        '_c2rust_label: {
            if !(*curbuf.ptr()).is_null() {
            } else {
                __assert_fail(
                    b"curbuf != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/buffer.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1998 as ::core::ffi::c_uint,
                    b"buf_T *buflist_new(char *, char *, linenr_T, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        buf = curbuf.get();
        set_bufref(&raw mut bufref_0, buf);
        trigger_undo_ftplugin(buf, curwin.get());
        buf_freeall(
            buf,
            BFA_WIPE as ::core::ffi::c_int | BFA_DEL as ::core::ffi::c_int,
        );
        if aborting() {
            xfree(ffname as *mut ::core::ffi::c_void);
            return ::core::ptr::null_mut::<buf_T>();
        }
        if !bufref_valid(&raw mut bufref_0) {
            buf = ::core::ptr::null_mut::<buf_T>();
        }
    }
    if buf != curbuf.get() || (*curbuf.ptr()).is_null() {
        buf = xcalloc(1 as size_t, ::core::mem::size_of::<buf_T>()) as *mut buf_T;
        (*buf).b_vars = tv_dict_alloc();
        init_var_dict((*buf).b_vars, &raw mut (*buf).b_bufvar, VAR_SCOPE);
        buf_init_changedtick(buf);
    }
    if !ffname.is_null() {
        (*buf).b_ffname = ffname;
        (*buf).b_sfname = xstrdup(sfname);
    }
    clear_wininfo(buf);
    let mut curwin_info: *mut WinInfo =
        xcalloc(1 as size_t, ::core::mem::size_of::<WinInfo>()) as *mut WinInfo;
    if (*buf).b_wininfo.size == (*buf).b_wininfo.capacity {
        (*buf).b_wininfo.capacity = if (*buf).b_wininfo.capacity != 0 {
            (*buf).b_wininfo.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        (*buf).b_wininfo.items = xrealloc(
            (*buf).b_wininfo.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<*mut WinInfo>().wrapping_mul((*buf).b_wininfo.capacity),
        ) as *mut *mut WinInfo;
    } else {
    };
    let c2rust_fresh0 = (*buf).b_wininfo.size;
    (*buf).b_wininfo.size = (*buf).b_wininfo.size.wrapping_add(1);
    let c2rust_lvalue_ptr = &raw mut *(*buf).b_wininfo.items.offset(c2rust_fresh0 as isize);
    *c2rust_lvalue_ptr = curwin_info;
    if buf == curbuf.get() {
        free_buffer_stuff(buf, kBffInitChangedtick as ::core::ffi::c_int);
        (*buf).b_p_initialized = false_0 != 0;
        buf_copy_options(buf, BCO_ENTER as ::core::ffi::c_int);
        (*curbuf.get()).b_kmap_state =
            ((*curbuf.get()).b_kmap_state as ::core::ffi::c_int | KEYMAP_INIT) as int16_t;
    } else {
        (*buf).b_next = ::core::ptr::null_mut::<buf_T>();
        if (*firstbuf.ptr()).is_null() {
            (*buf).b_prev = ::core::ptr::null_mut::<buf_T>();
            firstbuf.set(buf);
        } else {
            (*lastbuf.get()).b_next = buf;
            (*buf).b_prev = lastbuf.get();
        }
        lastbuf.set(buf);
        let c2rust_fresh1 = top_file_num.get();
        top_file_num.set(top_file_num.get() + 1);
        (*buf).handle = c2rust_fresh1 as handle_T;
        map_put_int_ptr_t(
            buffer_handles.ptr(),
            (*buf).handle as ::core::ffi::c_int,
            buf as ptr_t,
        );
        if top_file_num.get() < 0 as ::core::ffi::c_int {
            emsg(gettext(
                b"W14: Warning: List of file names overflow\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            if emsg_silent.get() == 0 as ::core::ffi::c_int && !in_assert_fails.get() {
                msg_delay(3001 as uint64_t, true_0 != 0);
            }
            top_file_num.set(1 as ::core::ffi::c_int);
        }
        buf_copy_options(buf, BCO_ALWAYS as ::core::ffi::c_int);
    }
    (*curwin_info).wi_mark = fmark_T {
        mark: pos_T {
            lnum: 0 as linenr_T,
            col: 0 as colnr_T,
            coladd: 0 as colnr_T,
        },
        fnum: 0 as ::core::ffi::c_int,
        timestamp: 0 as Timestamp,
        view: fmarkv_T {
            topline_offset: MAXLNUM as ::core::ffi::c_int as linenr_T,
            skipcol: 0 as colnr_T,
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    };
    (*curwin_info).wi_mark.mark.lnum = lnum;
    (*curwin_info).wi_win = curwin.get();
    hash_init(&raw mut (*buf).b_s.b_keywtab);
    hash_init(&raw mut (*buf).b_s.b_keywtab_ic);
    (*buf).b_fname = (*buf).b_sfname;
    if !file_id_valid {
        (*buf).file_id_valid = false_0 != 0;
    } else {
        (*buf).file_id_valid = true_0 != 0;
        (*buf).file_id = file_id;
    }
    (*buf).b_u_synced = true_0 != 0;
    (*buf).b_flags = BF_CHECK_RO | BF_NEVERLOADED;
    if flags & BLN_DUMMY as ::core::ffi::c_int != 0 {
        (*buf).b_flags |= BF_DUMMY;
    }
    buf_clear_file(buf);
    clrallmarks(buf, 0 as Timestamp);
    fmarks_check_names(buf);
    (*buf).b_p_bl = if flags & BLN_LISTED as ::core::ffi::c_int != 0 {
        true_0
    } else {
        false_0
    };
    xfree((*buf).update_channels.items as *mut ::core::ffi::c_void);
    (*buf).update_channels.capacity = 0 as size_t;
    (*buf).update_channels.size = (*buf).update_channels.capacity;
    (*buf).update_channels.items = ::core::ptr::null_mut::<uint64_t>();
    (*buf).update_channels.capacity = 0 as size_t;
    (*buf).update_channels.size = (*buf).update_channels.capacity;
    (*buf).update_channels.items = ::core::ptr::null_mut::<uint64_t>();
    xfree((*buf).update_callbacks.items as *mut ::core::ffi::c_void);
    (*buf).update_callbacks.capacity = 0 as size_t;
    (*buf).update_callbacks.size = (*buf).update_callbacks.capacity;
    (*buf).update_callbacks.items = ::core::ptr::null_mut::<BufUpdateCallbacks>();
    (*buf).update_callbacks.capacity = 0 as size_t;
    (*buf).update_callbacks.size = (*buf).update_callbacks.capacity;
    (*buf).update_callbacks.items = ::core::ptr::null_mut::<BufUpdateCallbacks>();
    if flags & BLN_DUMMY as ::core::ffi::c_int == 0 {
        let mut bufref_1: bufref_T = bufref_T {
            br_buf: ::core::ptr::null_mut::<buf_T>(),
            br_fnum: 0,
            br_buf_free_count: 0,
        };
        set_bufref(&raw mut bufref_1, buf);
        if apply_autocmds(
            EVENT_BUFNEW,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            buf,
        ) as ::core::ffi::c_int
            != 0
            && !bufref_valid(&raw mut bufref_1)
        {
            return ::core::ptr::null_mut::<buf_T>();
        }
        if flags & BLN_LISTED as ::core::ffi::c_int != 0
            && apply_autocmds(
                EVENT_BUFADD,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                buf,
            ) as ::core::ffi::c_int
                != 0
            && !bufref_valid(&raw mut bufref_1)
        {
            return ::core::ptr::null_mut::<buf_T>();
        }
        if aborting() {
            return ::core::ptr::null_mut::<buf_T>();
        }
    }
    (*buf).b_prompt_callback.type_0 = kCallbackNone;
    (*buf).b_prompt_interrupt.type_0 = kCallbackNone;
    (*buf).b_prompt_text = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*buf).b_prompt_start = fmark_T {
        mark: pos_T {
            lnum: 0 as linenr_T,
            col: 0 as colnr_T,
            coladd: 0 as colnr_T,
        },
        fnum: 0 as ::core::ffi::c_int,
        timestamp: 0 as Timestamp,
        view: fmarkv_T {
            topline_offset: MAXLNUM as ::core::ffi::c_int as linenr_T,
            skipcol: 0 as colnr_T,
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    };
    (*buf).b_prompt_start.mark.col = 2 as ::core::ffi::c_int as colnr_T;
    (*buf).b_prompt_append_new_line = true_0 != 0;
    return buf;
}
pub unsafe extern "C" fn curbuf_reusable() -> bool {
    return !(*curbuf.ptr()).is_null()
        && (*curbuf.get()).b_ffname.is_null()
        && (*curbuf.get()).b_nwindows <= 1 as ::core::ffi::c_int
        && (*curbuf.get()).terminal.is_null()
        && ((*curbuf.get()).b_ml.ml_mfp.is_null()
            || buf_is_empty(curbuf.get()) as ::core::ffi::c_int != 0)
        && !bt_quickfix(curbuf.get())
        && !curbufIsChanged();
}
pub unsafe extern "C" fn free_buf_options(mut buf: *mut buf_T, mut free_p_ff: bool) {
    if free_p_ff {
        clear_string_option(&raw mut (*buf).b_p_fenc);
        clear_string_option(&raw mut (*buf).b_p_ff);
        clear_string_option(&raw mut (*buf).b_p_bh);
        clear_string_option(&raw mut (*buf).b_p_bt);
    }
    clear_string_option(&raw mut (*buf).b_p_def);
    clear_string_option(&raw mut (*buf).b_p_inc);
    clear_string_option(&raw mut (*buf).b_p_inex);
    clear_string_option(&raw mut (*buf).b_p_inde);
    clear_string_option(&raw mut (*buf).b_p_indk);
    clear_string_option(&raw mut (*buf).b_p_fp);
    clear_string_option(&raw mut (*buf).b_p_fex);
    clear_string_option(&raw mut (*buf).b_p_kp);
    clear_string_option(&raw mut (*buf).b_p_mps);
    clear_string_option(&raw mut (*buf).b_p_fo);
    clear_string_option(&raw mut (*buf).b_p_flp);
    clear_string_option(&raw mut (*buf).b_p_isk);
    clear_string_option(&raw mut (*buf).b_p_vsts);
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*buf).b_p_vsts_nopaste as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    let _ = *ptr_;
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        &raw mut (*buf).b_p_vsts_array as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL_0;
    let _ = *ptr__0;
    clear_string_option(&raw mut (*buf).b_p_vts);
    let mut ptr__1: *mut *mut ::core::ffi::c_void =
        &raw mut (*buf).b_p_vts_array as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__1);
    *ptr__1 = NULL_0;
    let _ = *ptr__1;
    clear_string_option(&raw mut (*buf).b_p_keymap);
    keymap_ga_clear(&raw mut (*buf).b_kmap_ga);
    ga_clear(&raw mut (*buf).b_kmap_ga);
    clear_string_option(&raw mut (*buf).b_p_com);
    clear_string_option(&raw mut (*buf).b_p_cms);
    clear_string_option(&raw mut (*buf).b_p_nf);
    clear_string_option(&raw mut (*buf).b_p_syn);
    clear_string_option(&raw mut (*buf).b_s.b_syn_isk);
    clear_string_option(&raw mut (*buf).b_s.b_p_spc);
    clear_string_option(&raw mut (*buf).b_s.b_p_spf);
    vim_regfree((*buf).b_s.b_cap_prog);
    (*buf).b_s.b_cap_prog = ::core::ptr::null_mut::<regprog_T>();
    clear_string_option(&raw mut (*buf).b_s.b_p_spl);
    clear_string_option(&raw mut (*buf).b_s.b_p_spo);
    clear_string_option(&raw mut (*buf).b_p_sua);
    clear_string_option(&raw mut (*buf).b_p_ft);
    clear_string_option(&raw mut (*buf).b_p_cink);
    clear_string_option(&raw mut (*buf).b_p_cino);
    clear_string_option(&raw mut (*buf).b_p_lop);
    clear_string_option(&raw mut (*buf).b_p_cinsd);
    clear_string_option(&raw mut (*buf).b_p_cinw);
    clear_string_option(&raw mut (*buf).b_p_cot);
    clear_string_option(&raw mut (*buf).b_p_cpt);
    clear_string_option(&raw mut (*buf).b_p_cfu);
    callback_free(&raw mut (*buf).b_cfu_cb);
    clear_string_option(&raw mut (*buf).b_p_ofu);
    callback_free(&raw mut (*buf).b_ofu_cb);
    clear_string_option(&raw mut (*buf).b_p_tsrfu);
    callback_free(&raw mut (*buf).b_tsrfu_cb);
    clear_cpt_callbacks(&raw mut (*buf).b_p_cpt_cb, (*buf).b_p_cpt_count);
    (*buf).b_p_cpt_count = 0 as ::core::ffi::c_int;
    clear_string_option(&raw mut (*buf).b_p_gefm);
    clear_string_option(&raw mut (*buf).b_p_gp);
    clear_string_option(&raw mut (*buf).b_p_mp);
    clear_string_option(&raw mut (*buf).b_p_efm);
    clear_string_option(&raw mut (*buf).b_p_ep);
    clear_string_option(&raw mut (*buf).b_p_path);
    clear_string_option(&raw mut (*buf).b_p_tags);
    clear_string_option(&raw mut (*buf).b_p_tc);
    clear_string_option(&raw mut (*buf).b_p_tfu);
    callback_free(&raw mut (*buf).b_tfu_cb);
    clear_string_option(&raw mut (*buf).b_p_ffu);
    callback_free(&raw mut (*buf).b_ffu_cb);
    clear_string_option(&raw mut (*buf).b_p_dict);
    clear_string_option(&raw mut (*buf).b_p_dia);
    clear_string_option(&raw mut (*buf).b_p_tsr);
    clear_string_option(&raw mut (*buf).b_p_qe);
    (*buf).b_p_ac = -1 as ::core::ffi::c_int;
    (*buf).b_p_ar = -1 as ::core::ffi::c_int;
    (*buf).b_p_fs = -1 as ::core::ffi::c_int;
    (*buf).b_p_ul = NO_LOCAL_UNDOLEVEL as OptInt;
    clear_string_option(&raw mut (*buf).b_p_lw);
    clear_string_option(&raw mut (*buf).b_p_bkc);
    clear_string_option(&raw mut (*buf).b_p_menc);
}
pub unsafe extern "C" fn buflist_getfile(
    mut n: ::core::ffi::c_int,
    mut lnum: linenr_T,
    mut options: ::core::ffi::c_int,
    mut forceit: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut fm: *mut fmark_T = ::core::ptr::null_mut::<fmark_T>();
    let mut buf: *mut buf_T = buflist_findnr(n);
    if buf.is_null() {
        if options & GETF_ALT as ::core::ffi::c_int != 0 && n == 0 as ::core::ffi::c_int {
            emsg(gettext(&raw const e_noalt as *const ::core::ffi::c_char));
        } else {
            semsg(
                gettext(&raw const e_buffer_nr_not_found as *const ::core::ffi::c_char),
                n,
            );
        }
        return FAIL;
    }
    if buf == curbuf.get() {
        return OK;
    }
    if text_or_buf_locked() {
        return FAIL;
    }
    let mut col: colnr_T = 0;
    let mut restore_view: bool = false_0 != 0;
    if lnum == 0 as linenr_T {
        fm = buflist_findfmark(buf);
        lnum = (*fm).mark.lnum;
        col = (*fm).mark.col;
        restore_view = true_0 != 0;
    } else {
        col = 0 as ::core::ffi::c_int as colnr_T;
    }
    if options & GETF_SWITCH as ::core::ffi::c_int != 0 {
        wp = swbuf_goto_win_with_buf(buf);
        if wp.is_null()
            && swb_flags.get()
                & (kOptSwbFlagVsplit as ::core::ffi::c_int
                    | kOptSwbFlagSplit as ::core::ffi::c_int
                    | kOptSwbFlagNewtab as ::core::ffi::c_int)
                    as ::core::ffi::c_uint
                != 0
            && !buf_is_empty(curbuf.get())
        {
            if swb_flags.get() & kOptSwbFlagNewtab as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            {
                tabpage_new();
            } else if win_split(
                0 as ::core::ffi::c_int,
                if swb_flags.get() & kOptSwbFlagVsplit as ::core::ffi::c_int as ::core::ffi::c_uint
                    != 0
                {
                    WSP_VERT as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                },
            ) == FAIL
            {
                return FAIL;
            }
            (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
            (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
        }
    }
    (*RedrawingDisabled.ptr()) += 1;
    if getfile(
        (*buf).handle as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        options & GETF_SETMARK as ::core::ffi::c_int != 0,
        lnum,
        forceit != 0,
    ) <= 0 as ::core::ffi::c_int
    {
        (*RedrawingDisabled.ptr()) -= 1;
        if p_sol.get() == 0 && col != 0 as ::core::ffi::c_int {
            (*curwin.get()).w_cursor.col = col;
            check_cursor_col(curwin.get());
            (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
            (*curwin.get()).w_set_curswant = true_0;
        }
        if jop_flags.get() & kOptJopFlagView as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && restore_view as ::core::ffi::c_int != 0
        {
            mark_view_restore(fm);
        }
        return OK;
    }
    (*RedrawingDisabled.ptr()) -= 1;
    return FAIL;
}
unsafe extern "C" fn buflist_getfpos() {
    let mut fm: *mut fmark_T = buflist_findfmark(curbuf.get());
    let mut fpos: *const pos_T = &raw mut (*fm).mark;
    (*curwin.get()).w_cursor.lnum = (*fpos).lnum;
    check_cursor_lnum(curwin.get());
    if p_sol.get() != 0 {
        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    } else {
        (*curwin.get()).w_cursor.col = (*fpos).col;
        check_cursor_col(curwin.get());
        (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
        (*curwin.get()).w_set_curswant = true_0;
    }
    if jop_flags.get() & kOptJopFlagView as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        mark_view_restore(fm);
    }
}
#[no_mangle]
pub unsafe extern "C" fn buflist_findname_exp(mut fname: *mut ::core::ffi::c_char) -> *mut buf_T {
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut ffname: *mut ::core::ffi::c_char = FullName_save(fname, true_0 != 0);
    if !ffname.is_null() {
        buf = buflist_findname(ffname);
        xfree(ffname as *mut ::core::ffi::c_void);
    }
    return buf;
}
pub unsafe extern "C" fn buflist_findname(mut ffname: *mut ::core::ffi::c_char) -> *mut buf_T {
    let mut file_id: FileID = FileID {
        inode: 0,
        device_id: 0,
    };
    let mut file_id_valid: bool = os_fileid(ffname, &raw mut file_id);
    return buflist_findname_file_id(ffname, &raw mut file_id, file_id_valid);
}
unsafe extern "C" fn buflist_findname_file_id(
    mut ffname: *mut ::core::ffi::c_char,
    mut file_id: *mut FileID,
    mut file_id_valid: bool,
) -> *mut buf_T {
    let mut buf: *mut buf_T = lastbuf.get();
    while !buf.is_null() {
        if (*buf).b_flags & BF_DUMMY == 0 as ::core::ffi::c_int
            && !otherfile_buf(buf, ffname, file_id, file_id_valid)
        {
            return buf;
        }
        buf = (*buf).b_prev;
    }
    return ::core::ptr::null_mut::<buf_T>();
}
#[no_mangle]
pub unsafe extern "C" fn buflist_findpat(
    mut pattern: *const ::core::ffi::c_char,
    mut pattern_end: *const ::core::ffi::c_char,
    mut unlisted: bool,
    mut diffmode: bool,
    mut curtab_only: bool,
) -> ::core::ffi::c_int {
    let mut match_0: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if pattern_end == pattern.offset(1 as ::core::ffi::c_int as isize)
        && (*pattern as ::core::ffi::c_int == '%' as ::core::ffi::c_int
            || *pattern as ::core::ffi::c_int == '#' as ::core::ffi::c_int)
    {
        match_0 = if *pattern as ::core::ffi::c_int == '%' as ::core::ffi::c_int {
            (*curbuf.get()).handle as ::core::ffi::c_int
        } else {
            (*curwin.get()).w_alt_fnum
        };
        let mut found_buf: *mut buf_T = buflist_findnr(match_0);
        if diffmode as ::core::ffi::c_int != 0
            && !(!found_buf.is_null() && diff_mode_buf(found_buf) as ::core::ffi::c_int != 0)
        {
            match_0 = -1 as ::core::ffi::c_int;
        }
    } else {
        let mut pat: *mut ::core::ffi::c_char = file_pat_to_reg_pat(
            pattern,
            pattern_end,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0,
        );
        if pat.is_null() {
            return -1 as ::core::ffi::c_int;
        }
        let mut patend: *mut ::core::ffi::c_char = pat
            .offset(strlen(pat) as isize)
            .offset(-(1 as ::core::ffi::c_int as isize));
        let mut toggledollar: bool =
            patend > pat && *patend as ::core::ffi::c_int == '$' as ::core::ffi::c_int;
        let mut find_listed: ::core::ffi::c_int = true_0;
        loop {
            let mut attempt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while attempt <= 3 as ::core::ffi::c_int {
                if toggledollar {
                    *patend = (if attempt < 2 as ::core::ffi::c_int {
                        NUL
                    } else {
                        '$' as ::core::ffi::c_int
                    }) as ::core::ffi::c_char;
                }
                let mut p: *mut ::core::ffi::c_char = pat;
                if *p as ::core::ffi::c_int == '^' as ::core::ffi::c_int
                    && attempt & 1 as ::core::ffi::c_int == 0
                {
                    p = p.offset(1);
                }
                let mut regmatch: regmatch_T = regmatch_T {
                    regprog: ::core::ptr::null_mut::<regprog_T>(),
                    startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                    endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                    rm_matchcol: 0,
                    rm_ic: false,
                };
                regmatch.regprog = vim_regcomp(
                    p,
                    if magic_isset() as ::core::ffi::c_int != 0 {
                        RE_MAGIC
                    } else {
                        0 as ::core::ffi::c_int
                    },
                );
                let mut buf: *mut buf_T = lastbuf.get();
                's_171: while !buf.is_null() {
                    if regmatch.regprog.is_null() {
                        xfree(pat as *mut ::core::ffi::c_void);
                        return -1 as ::core::ffi::c_int;
                    }
                    's_92: {
                        if (*buf).b_p_bl == find_listed
                            && (!diffmode || diff_mode_buf(buf) as ::core::ffi::c_int != 0)
                            && !buflist_match(&raw mut regmatch, buf, false_0 != 0).is_null()
                        {
                            if curtab_only {
                                let mut found_window: bool = false_0 != 0;
                                let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                                    firstwin.get()
                                } else {
                                    (*curtab.get()).tp_firstwin
                                };
                                while !wp.is_null() {
                                    if (*wp).w_buffer == buf {
                                        found_window = true_0 != 0;
                                        break;
                                    } else {
                                        wp = (*wp).w_next;
                                    }
                                }
                                if !found_window {
                                    break 's_92;
                                }
                            }
                            if match_0 >= 0 as ::core::ffi::c_int {
                                match_0 = -2 as ::core::ffi::c_int;
                                break 's_171;
                            } else {
                                match_0 = (*buf).handle as ::core::ffi::c_int;
                            }
                        }
                    }
                    buf = (*buf).b_prev;
                }
                vim_regfree(regmatch.regprog);
                if match_0 >= 0 as ::core::ffi::c_int {
                    break;
                }
                attempt += 1;
            }
            if !unlisted || find_listed == 0 || match_0 != -1 as ::core::ffi::c_int {
                break;
            }
            find_listed = false_0;
        }
        xfree(pat as *mut ::core::ffi::c_void);
    }
    if match_0 == -2 as ::core::ffi::c_int {
        semsg(
            gettext(b"E93: More than one match for %s\0".as_ptr() as *const ::core::ffi::c_char),
            pattern,
        );
    } else if match_0 < 0 as ::core::ffi::c_int {
        semsg(
            gettext(b"E94: No matching buffer for %s\0".as_ptr() as *const ::core::ffi::c_char),
            pattern,
        );
    }
    return match_0;
}
unsafe extern "C" fn buf_time_compare(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut buf1: *mut buf_T = *(s1 as *mut *mut buf_T);
    let mut buf2: *mut buf_T = *(s2 as *mut *mut buf_T);
    if (*buf1).b_last_used == (*buf2).b_last_used {
        return 0 as ::core::ffi::c_int;
    }
    return if (*buf1).b_last_used > (*buf2).b_last_used {
        -1 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    };
}
pub unsafe extern "C" fn ExpandBufnames(
    mut pat: *mut ::core::ffi::c_char,
    mut num_file: *mut ::core::ffi::c_int,
    mut file: *mut *mut *mut ::core::ffi::c_char,
    mut options: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut matches: *mut bufmatch_T = ::core::ptr::null_mut::<bufmatch_T>();
    let mut to_free: bool = false_0 != 0;
    *num_file = 0 as ::core::ffi::c_int;
    *file = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    if options & BUF_DIFF_FILTER as ::core::ffi::c_int != 0
        && (*curwin.get()).w_onebuf_opt.wo_diff == 0
    {
        return FAIL;
    }
    let fuzzy: bool = cmdline_fuzzy_complete(pat);
    let mut patc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fuzmatch: *mut fuzmatch_str_T = ::core::ptr::null_mut::<fuzmatch_str_T>();
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    if !fuzzy {
        if *pat as ::core::ffi::c_int == '^' as ::core::ffi::c_int
            && *pat.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            patc = xstrdup(pat.offset(1 as ::core::ffi::c_int as isize));
            to_free = true_0 != 0;
        } else if *pat as ::core::ffi::c_int == '^' as ::core::ffi::c_int {
            patc = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            patc = pat;
        }
        regmatch.regprog = vim_regcomp(patc, RE_MAGIC);
    }
    let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut score: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut round: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while round <= 2 as ::core::ffi::c_int {
        count = 0 as ::core::ffi::c_int;
        let mut buf: *mut buf_T = firstbuf.get();
        while !buf.is_null() {
            's_95: {
                if (*buf).b_p_bl != 0 {
                    if options & BUF_DIFF_FILTER as ::core::ffi::c_int != 0 {
                        if buf == curbuf.get() || !diff_mode_buf(buf) {
                            break 's_95;
                        }
                    }
                    let mut p: *mut ::core::ffi::c_char =
                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                    if !fuzzy {
                        if regmatch.regprog.is_null() {
                            if to_free {
                                xfree(patc as *mut ::core::ffi::c_void);
                            }
                            return FAIL;
                        }
                        p = buflist_match(&raw mut regmatch, buf, p_wic.get() != 0);
                    } else {
                        p = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        score = fuzzy_match_str((*buf).b_sfname, pat);
                        if score != FUZZY_SCORE_NONE as ::core::ffi::c_int {
                            p = (*buf).b_sfname;
                        }
                        if p.is_null() {
                            score = fuzzy_match_str((*buf).b_ffname, pat);
                            if score != FUZZY_SCORE_NONE as ::core::ffi::c_int {
                                p = (*buf).b_ffname;
                            }
                        }
                    }
                    if !p.is_null() {
                        if round == 1 as ::core::ffi::c_int {
                            count += 1;
                        } else {
                            if options & WILD_HOME_REPLACE as ::core::ffi::c_int != 0 {
                                p = home_replace_save(buf, p);
                            } else {
                                p = xstrdup(p);
                            }
                            if !fuzzy {
                                if !matches.is_null() {
                                    (*matches.offset(count as isize)).buf = buf;
                                    (*matches.offset(count as isize)).match_0 = p;
                                    count += 1;
                                } else {
                                    let c2rust_fresh3 = count;
                                    count = count + 1;
                                    let c2rust_lvalue_ptr =
                                        &raw mut *(*file).offset(c2rust_fresh3 as isize);
                                    *c2rust_lvalue_ptr = p;
                                }
                            } else {
                                (*fuzmatch.offset(count as isize)).idx = count;
                                (*fuzmatch.offset(count as isize)).str = p;
                                (*fuzmatch.offset(count as isize)).score = score;
                                count += 1;
                            }
                        }
                    }
                }
            }
            buf = (*buf).b_next;
        }
        if count == 0 as ::core::ffi::c_int {
            break;
        }
        if round == 1 as ::core::ffi::c_int {
            if !fuzzy {
                *file = xmalloc(
                    (count as size_t)
                        .wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_char>()),
                ) as *mut *mut ::core::ffi::c_char;
                if options & WILD_BUFLASTUSED as ::core::ffi::c_int != 0 {
                    matches = xmalloc(
                        (count as size_t).wrapping_mul(::core::mem::size_of::<bufmatch_T>()),
                    ) as *mut bufmatch_T;
                }
            } else {
                fuzmatch = xmalloc(
                    (count as size_t).wrapping_mul(::core::mem::size_of::<fuzmatch_str_T>()),
                ) as *mut fuzmatch_str_T;
            }
        }
        round += 1;
    }
    if !fuzzy {
        vim_regfree(regmatch.regprog);
        if to_free {
            xfree(patc as *mut ::core::ffi::c_void);
        }
    }
    if !fuzzy {
        if !matches.is_null() {
            if count > 1 as ::core::ffi::c_int {
                qsort(
                    matches as *mut ::core::ffi::c_void,
                    count as size_t,
                    ::core::mem::size_of::<bufmatch_T>(),
                    Some(
                        buf_time_compare
                            as unsafe extern "C" fn(
                                *const ::core::ffi::c_void,
                                *const ::core::ffi::c_void,
                            )
                                -> ::core::ffi::c_int,
                    ),
                );
            }
            if (*matches.offset(0 as ::core::ffi::c_int as isize)).buf == curbuf.get() {
                let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                while i < count {
                    *(*file).offset((i - 1 as ::core::ffi::c_int) as isize) =
                        (*matches.offset(i as isize)).match_0;
                    i += 1;
                }
                *(*file).offset((count - 1 as ::core::ffi::c_int) as isize) =
                    (*matches.offset(0 as ::core::ffi::c_int as isize)).match_0;
            } else {
                let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i_0 < count {
                    *(*file).offset(i_0 as isize) = (*matches.offset(i_0 as isize)).match_0;
                    i_0 += 1;
                }
            }
            xfree(matches as *mut ::core::ffi::c_void);
        }
    } else {
        fuzzymatches_to_strmatches(fuzmatch, file, count, false_0 != 0);
    }
    *num_file = count;
    return if count == 0 as ::core::ffi::c_int {
        FAIL
    } else {
        OK
    };
}
unsafe extern "C" fn buflist_match(
    mut rmp: *mut regmatch_T,
    mut buf: *mut buf_T,
    mut ignore_case: bool,
) -> *mut ::core::ffi::c_char {
    let mut match_0: *mut ::core::ffi::c_char = fname_match(rmp, (*buf).b_sfname, ignore_case);
    if match_0.is_null() && !(*rmp).regprog.is_null() {
        match_0 = fname_match(rmp, (*buf).b_ffname, ignore_case);
    }
    return match_0;
}
unsafe extern "C" fn fname_match(
    mut rmp: *mut regmatch_T,
    mut name: *mut ::core::ffi::c_char,
    mut ignore_case: bool,
) -> *mut ::core::ffi::c_char {
    let mut match_0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if name.is_null() || (*rmp).regprog.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    (*rmp).rm_ic = p_fic.get() != 0 || ignore_case as ::core::ffi::c_int != 0;
    if vim_regexec(rmp, name, 0 as colnr_T) {
        match_0 = name;
    } else if !(*rmp).regprog.is_null() {
        let mut p: *mut ::core::ffi::c_char =
            home_replace_save(::core::ptr::null_mut::<buf_T>(), name);
        if vim_regexec(rmp, p, 0 as colnr_T) {
            match_0 = name;
        }
        xfree(p as *mut ::core::ffi::c_void);
    }
    return match_0;
}
#[no_mangle]
pub unsafe extern "C" fn buflist_findnr(mut nr: ::core::ffi::c_int) -> *mut buf_T {
    if nr == 0 as ::core::ffi::c_int {
        nr = (*curwin.get()).w_alt_fnum;
    }
    return map_get_int_ptr_t(buffer_handles.ptr(), nr) as *mut buf_T;
}
pub unsafe extern "C" fn buflist_nr2name(
    mut n: ::core::ffi::c_int,
    mut fullname: ::core::ffi::c_int,
    mut helptail: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut buf: *mut buf_T = buflist_findnr(n);
    if buf.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return home_replace_save(
        if helptail != 0 {
            buf
        } else {
            ::core::ptr::null_mut::<buf_T>()
        },
        if fullname != 0 {
            (*buf).b_ffname
        } else {
            (*buf).b_fname
        },
    );
}
pub unsafe extern "C" fn buflist_setfpos(
    buf: *mut buf_T,
    win: *mut win_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut copy_options: bool,
) {
    let mut wip: *mut WinInfo = ::core::ptr::null_mut::<WinInfo>();
    let mut i: size_t = 0;
    i = 0 as size_t;
    while i < (*buf).b_wininfo.size {
        wip = *(*buf).b_wininfo.items.offset(i as isize);
        if (*wip).wi_win == win {
            break;
        }
        i = i.wrapping_add(1);
    }
    if i == (*buf).b_wininfo.size {
        wip = xcalloc(1 as size_t, ::core::mem::size_of::<WinInfo>()) as *mut WinInfo;
        (*wip).wi_win = win;
        if lnum == 0 as linenr_T {
            lnum = 1 as ::core::ffi::c_int as linenr_T;
        }
    } else {
        (*buf).b_wininfo.size = (*buf).b_wininfo.size.wrapping_sub(1 as size_t);
        (i < (*buf).b_wininfo.size
            && !memmove(
                (*buf).b_wininfo.items.offset(i as isize) as *mut ::core::ffi::c_void,
                (*buf)
                    .b_wininfo
                    .items
                    .offset(i.wrapping_add(1 as size_t) as isize)
                    as *const ::core::ffi::c_void,
                (*buf)
                    .b_wininfo
                    .size
                    .wrapping_sub(i)
                    .wrapping_mul(::core::mem::size_of::<*mut WinInfo>()),
            )
            .is_null()) as ::core::ffi::c_int;
        if copy_options as ::core::ffi::c_int != 0 && (*wip).wi_optset as ::core::ffi::c_int != 0 {
            clear_winopt(&raw mut (*wip).wi_opt);
            deleteFoldRecurse(buf, &raw mut (*wip).wi_folds);
        }
    }
    if lnum != 0 as linenr_T {
        (*wip).wi_mark.mark.lnum = lnum;
        (*wip).wi_mark.mark.col = col;
        if !win.is_null() {
            (*wip).wi_mark.view = mark_view_make(win, (*wip).wi_mark.mark);
        }
    }
    if !win.is_null() {
        (*wip).wi_changelistidx = (*win).w_changelistidx;
    }
    if copy_options as ::core::ffi::c_int != 0 && !win.is_null() {
        copy_winopt(&raw mut (*win).w_onebuf_opt, &raw mut (*wip).wi_opt);
        (*wip).wi_fold_manual = (*win).w_fold_manual;
        cloneFoldGrowArray(&raw mut (*win).w_folds, &raw mut (*wip).wi_folds);
        (*wip).wi_optset = true_0 != 0;
    }
    if (*buf).b_wininfo.size == (*buf).b_wininfo.capacity {
        (*buf).b_wininfo.capacity = if (*buf).b_wininfo.capacity != 0 {
            (*buf).b_wininfo.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        (*buf).b_wininfo.items = xrealloc(
            (*buf).b_wininfo.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<*mut WinInfo>().wrapping_mul((*buf).b_wininfo.capacity),
        ) as *mut *mut WinInfo;
    } else {
    };
    (*buf).b_wininfo.size = (*buf).b_wininfo.size.wrapping_add(1);
    memmove(
        (*buf)
            .b_wininfo
            .items
            .offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
        (*buf)
            .b_wininfo
            .items
            .offset(0 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        (*buf)
            .b_wininfo
            .size
            .wrapping_sub(1 as size_t)
            .wrapping_mul(::core::mem::size_of::<*mut WinInfo>()),
    );
    *(*buf)
        .b_wininfo
        .items
        .offset(0 as ::core::ffi::c_int as isize) = wip;
}
unsafe extern "C" fn wininfo_other_tab_diff(mut wip: *mut WinInfo) -> bool {
    if (*wip).wi_opt.wo_diff == 0 {
        return false_0 != 0;
    }
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wip).wi_win == wp {
            return false_0 != 0;
        }
        wp = (*wp).w_next;
    }
    return true_0 != 0;
}
unsafe extern "C" fn find_wininfo(
    mut buf: *mut buf_T,
    mut need_options: bool,
    mut skip_diff_buffer: bool,
) -> *mut WinInfo {
    let mut i: size_t = 0 as size_t;
    while i < (*buf).b_wininfo.size {
        let mut wip: *mut WinInfo = *(*buf).b_wininfo.items.offset(i as isize);
        if (*wip).wi_win == curwin.get()
            && (!skip_diff_buffer || !wininfo_other_tab_diff(wip))
            && (!need_options || (*wip).wi_optset as ::core::ffi::c_int != 0)
        {
            return wip;
        }
        i = i.wrapping_add(1);
    }
    if skip_diff_buffer {
        let mut i_0: size_t = 0 as size_t;
        while i_0 < (*buf).b_wininfo.size {
            let mut wip_0: *mut WinInfo = *(*buf).b_wininfo.items.offset(i_0 as isize);
            if !wininfo_other_tab_diff(wip_0)
                && (!need_options
                    || (*wip_0).wi_optset as ::core::ffi::c_int != 0
                    || !(*wip_0).wi_win.is_null() && (*(*wip_0).wi_win).w_buffer == buf)
            {
                return wip_0;
            }
            i_0 = i_0.wrapping_add(1);
        }
    } else if (*buf).b_wininfo.size != 0 {
        return *(*buf)
            .b_wininfo
            .items
            .offset(0 as ::core::ffi::c_int as isize);
    }
    return ::core::ptr::null_mut::<WinInfo>();
}
pub unsafe extern "C" fn get_winopts(mut buf: *mut buf_T) {
    clear_winopt(&raw mut (*curwin.get()).w_onebuf_opt);
    clearFolding(curwin.get());
    let wip: *mut WinInfo = find_wininfo(buf, true_0 != 0, true_0 != 0);
    if !wip.is_null()
        && (*wip).wi_win != curwin.get()
        && !(*wip).wi_win.is_null()
        && (*(*wip).wi_win).w_buffer == buf
        && (*(*wip).wi_win).w_config.style as ::core::ffi::c_uint
            != kWinStyleMinimal as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut wp: *mut win_T = (*wip).wi_win;
        copy_winopt(
            &raw mut (*wp).w_onebuf_opt,
            &raw mut (*curwin.get()).w_onebuf_opt,
        );
        (*curwin.get()).w_fold_manual = (*wp).w_fold_manual;
        (*curwin.get()).w_foldinvalid = true_0 != 0;
        cloneFoldGrowArray(&raw mut (*wp).w_folds, &raw mut (*curwin.get()).w_folds);
    } else if !wip.is_null()
        && (*wip).wi_optset as ::core::ffi::c_int != 0
        && ((*wip).wi_win.is_null()
            || (*wip).wi_win == curwin.get()
            || (*(*wip).wi_win).w_config.style as ::core::ffi::c_uint
                != kWinStyleMinimal as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        copy_winopt(
            &raw mut (*wip).wi_opt,
            &raw mut (*curwin.get()).w_onebuf_opt,
        );
        (*curwin.get()).w_fold_manual = (*wip).wi_fold_manual;
        (*curwin.get()).w_foldinvalid = true_0 != 0;
        cloneFoldGrowArray(&raw mut (*wip).wi_folds, &raw mut (*curwin.get()).w_folds);
    } else {
        copy_winopt(
            &raw mut (*curwin.get()).w_allbuf_opt,
            &raw mut (*curwin.get()).w_onebuf_opt,
        );
    }
    if !wip.is_null() {
        (*curwin.get()).w_changelistidx = (*wip).wi_changelistidx;
    }
    if (*curwin.get()).w_config.style as ::core::ffi::c_uint
        == kWinStyleMinimal as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        didset_window_options(curwin.get(), false_0 != 0);
        win_set_minimal_style(curwin.get());
    }
    if p_fdls.get() >= 0 as OptInt {
        (*curwin.get()).w_onebuf_opt.wo_fdl = p_fdls.get();
    }
    didset_window_options(curwin.get(), false_0 != 0);
}
pub unsafe extern "C" fn buflist_findfmark(mut buf: *mut buf_T) -> *mut fmark_T {
    static no_position: GlobalCell<fmark_T> = GlobalCell::new(fmark_T {
        mark: pos_T {
            lnum: 1 as linenr_T,
            col: 0 as colnr_T,
            coladd: 0 as colnr_T,
        },
        fnum: 0 as ::core::ffi::c_int,
        timestamp: 0 as Timestamp,
        view: fmarkv_T {
            topline_offset: MAXLNUM as ::core::ffi::c_int as linenr_T,
            skipcol: 0 as colnr_T,
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    });
    let wip: *mut WinInfo = find_wininfo(buf, false_0 != 0, false_0 != 0);
    return if wip.is_null() {
        no_position.ptr()
    } else {
        &raw mut (*wip).wi_mark
    };
}
pub unsafe extern "C" fn buflist_findlnum(mut buf: *mut buf_T) -> linenr_T {
    return (*buflist_findfmark(buf)).mark.lnum;
}
pub unsafe extern "C" fn buflist_list(mut eap: *mut exarg_T) {
    let mut buf: *mut buf_T = firstbuf.get();
    let mut buflist: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut buflist_data: *mut *mut buf_T = ::core::ptr::null_mut::<*mut buf_T>();
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    if !vim_strchr((*eap).arg, 't' as ::core::ffi::c_int).is_null() {
        ga_init(
            &raw mut buflist,
            ::core::mem::size_of::<*mut buf_T>() as ::core::ffi::c_int,
            50 as ::core::ffi::c_int,
        );
        buf = firstbuf.get();
        while !buf.is_null() {
            ga_grow(&raw mut buflist, 1 as ::core::ffi::c_int);
            let c2rust_fresh4 = buflist.ga_len;
            buflist.ga_len = buflist.ga_len + 1;
            let c2rust_lvalue_ptr =
                &raw mut *(buflist.ga_data as *mut *mut buf_T).offset(c2rust_fresh4 as isize);
            *c2rust_lvalue_ptr = buf;
            buf = (*buf).b_next;
        }
        qsort(
            buflist.ga_data,
            buflist.ga_len as size_t,
            ::core::mem::size_of::<*mut buf_T>(),
            Some(
                buf_time_compare
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
        buflist_data = buflist.ga_data as *mut *mut buf_T;
        buf = *buflist_data;
    }
    let mut p: *mut *mut buf_T = buflist_data;
    while !buf.is_null() && !got_int.get() {
        let is_terminal: bool = !(*buf).terminal.is_null();
        let job_running: bool = !(*buf).terminal.is_null()
            && terminal_running((*buf).terminal) as ::core::ffi::c_int != 0;
        if !((*buf).b_p_bl == 0
            && (*eap).forceit == 0
            && vim_strchr((*eap).arg, 'u' as ::core::ffi::c_int).is_null()
            || !vim_strchr((*eap).arg, 'u' as ::core::ffi::c_int).is_null() && (*buf).b_p_bl != 0
            || !vim_strchr((*eap).arg, '+' as ::core::ffi::c_int).is_null()
                && ((*buf).b_flags & BF_READERR != 0 || !bufIsChanged(buf))
            || !vim_strchr((*eap).arg, 'a' as ::core::ffi::c_int).is_null()
                && ((*buf).b_ml.ml_mfp.is_null() || (*buf).b_nwindows == 0 as ::core::ffi::c_int)
            || !vim_strchr((*eap).arg, 'h' as ::core::ffi::c_int).is_null()
                && ((*buf).b_ml.ml_mfp.is_null() || (*buf).b_nwindows != 0 as ::core::ffi::c_int)
            || !vim_strchr((*eap).arg, 'R' as ::core::ffi::c_int).is_null()
                && (!is_terminal || !job_running)
            || !vim_strchr((*eap).arg, 'F' as ::core::ffi::c_int).is_null()
                && (!is_terminal || job_running as ::core::ffi::c_int != 0)
            || !vim_strchr((*eap).arg, '-' as ::core::ffi::c_int).is_null() && (*buf).b_p_ma != 0
            || !vim_strchr((*eap).arg, '=' as ::core::ffi::c_int).is_null() && (*buf).b_p_ro == 0
            || !vim_strchr((*eap).arg, 'x' as ::core::ffi::c_int).is_null()
                && (*buf).b_flags & BF_READERR == 0
            || !vim_strchr((*eap).arg, '%' as ::core::ffi::c_int).is_null() && buf != curbuf.get()
            || !vim_strchr((*eap).arg, '#' as ::core::ffi::c_int).is_null()
                && (buf == curbuf.get() || (*curwin.get()).w_alt_fnum != (*buf).handle))
        {
            let mut name: *mut ::core::ffi::c_char = buf_spname(buf);
            if !name.is_null() {
                xstrlcpy(
                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                    name,
                    MAXPATHL as size_t,
                );
            } else {
                home_replace(
                    buf,
                    (*buf).b_fname,
                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                    MAXPATHL as size_t,
                    true_0 != 0,
                );
            }
            if !message_filtered(NameBuff.ptr() as *mut ::core::ffi::c_char) {
                let changed_char: ::core::ffi::c_int = if (*buf).b_flags & BF_READERR != 0 {
                    'x' as ::core::ffi::c_int
                } else if bufIsChanged(buf) as ::core::ffi::c_int != 0 {
                    '+' as ::core::ffi::c_int
                } else {
                    ' ' as ::core::ffi::c_int
                };
                let mut ro_char: ::core::ffi::c_int = if (*buf).b_p_ma == 0 {
                    '-' as ::core::ffi::c_int
                } else if (*buf).b_p_ro != 0 {
                    '=' as ::core::ffi::c_int
                } else {
                    ' ' as ::core::ffi::c_int
                };
                if !(*buf).terminal.is_null() {
                    ro_char = if terminal_running((*buf).terminal) as ::core::ffi::c_int != 0 {
                        'R' as ::core::ffi::c_int
                    } else {
                        'F' as ::core::ffi::c_int
                    };
                }
                if !ui_has(kUIMessages) || msg_col.get() > 0 as ::core::ffi::c_int {
                    msg_putchar('\n' as ::core::ffi::c_int);
                }
                let mut len: ::core::ffi::c_int = vim_snprintf_safelen(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    (IOSIZE - 20 as ::core::ffi::c_int) as size_t,
                    b"%3d%c%c%c%c%c \"%s\"\0".as_ptr() as *const ::core::ffi::c_char,
                    (*buf).handle,
                    if (*buf).b_p_bl != 0 {
                        ' ' as ::core::ffi::c_int
                    } else {
                        'u' as ::core::ffi::c_int
                    },
                    if buf == curbuf.get() {
                        '%' as ::core::ffi::c_int
                    } else if (*curwin.get()).w_alt_fnum == (*buf).handle {
                        '#' as ::core::ffi::c_int
                    } else {
                        ' ' as ::core::ffi::c_int
                    },
                    if (*buf).b_ml.ml_mfp.is_null() {
                        ' ' as ::core::ffi::c_int
                    } else if (*buf).b_nwindows == 0 as ::core::ffi::c_int {
                        'h' as ::core::ffi::c_int
                    } else {
                        'a' as ::core::ffi::c_int
                    },
                    ro_char,
                    changed_char,
                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                ) as ::core::ffi::c_int;
                len = if len
                    < 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                        - 20 as ::core::ffi::c_int
                {
                    len
                } else {
                    1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int - 20 as ::core::ffi::c_int
                };
                let mut i: ::core::ffi::c_int = 40 as ::core::ffi::c_int
                    - vim_strsize(IObuff.ptr() as *mut ::core::ffi::c_char);
                loop {
                    let c2rust_fresh5 = len;
                    len = len + 1;
                    (*IObuff.ptr())[c2rust_fresh5 as usize] = ' ' as ::core::ffi::c_char;
                    i -= 1;
                    if !(i > 0 as ::core::ffi::c_int && len < IOSIZE - 18 as ::core::ffi::c_int) {
                        break;
                    }
                }
                if !vim_strchr((*eap).arg, 't' as ::core::ffi::c_int).is_null()
                    && (*buf).b_last_used != 0
                {
                    undo_fmt_time(
                        (IObuff.ptr() as *mut ::core::ffi::c_char).offset(len as isize),
                        (IOSIZE - len) as size_t,
                        (*buf).b_last_used,
                    );
                } else {
                    vim_snprintf(
                        (IObuff.ptr() as *mut ::core::ffi::c_char).offset(len as isize),
                        (IOSIZE - len) as size_t,
                        gettext(b"line %ld\0".as_ptr() as *const ::core::ffi::c_char),
                        if buf == curbuf.get() {
                            (*curwin.get()).w_cursor.lnum as int64_t
                        } else {
                            buflist_findlnum(buf) as int64_t
                        },
                    );
                }
                msg_outtrans(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                    false_0 != 0,
                );
                line_breakcheck();
            }
        }
        buf = if !buflist_data.is_null() {
            p = p.offset(1);
            if p < buflist_data.offset(buflist.ga_len as isize) {
                *p
            } else {
                ::core::ptr::null_mut::<buf_T>()
            }
        } else {
            (*buf).b_next
        };
    }
    if !buflist_data.is_null() {
        ga_clear(&raw mut buflist);
    }
}
pub unsafe extern "C" fn buflist_name_nr(
    mut fnum: ::core::ffi::c_int,
    mut fname: *mut *mut ::core::ffi::c_char,
    mut lnum: *mut linenr_T,
) -> ::core::ffi::c_int {
    let mut buf: *mut buf_T = buflist_findnr(fnum);
    if buf.is_null() || (*buf).b_fname.is_null() {
        return FAIL;
    }
    *fname = (*buf).b_fname;
    *lnum = buflist_findlnum(buf);
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn setfname(
    mut buf: *mut buf_T,
    mut ffname_arg: *mut ::core::ffi::c_char,
    mut sfname_arg: *mut ::core::ffi::c_char,
    mut message: bool,
) -> ::core::ffi::c_int {
    let mut ffname: *mut ::core::ffi::c_char = ffname_arg;
    let mut sfname: *mut ::core::ffi::c_char = sfname_arg;
    let mut obuf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut file_id: FileID = FileID {
        inode: 0,
        device_id: 0,
    };
    let mut file_id_valid: bool = false_0 != 0;
    if ffname.is_null() || *ffname as ::core::ffi::c_int == NUL {
        if (*buf).b_sfname != (*buf).b_ffname {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*buf).b_sfname as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_0;
            let _ = *ptr_;
        } else {
            (*buf).b_sfname = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut (*buf).b_ffname as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_0;
        let _ = *ptr__0;
    } else {
        fname_expand(buf, &raw mut ffname, &raw mut sfname);
        if ffname.is_null() {
            return FAIL;
        }
        file_id_valid = os_fileid(ffname, &raw mut file_id);
        if (*buf).b_flags & BF_DUMMY == 0 {
            obuf = buflist_findname_file_id(ffname, &raw mut file_id, file_id_valid);
        }
        if !obuf.is_null() && obuf != buf {
            let mut in_use: bool = false_0 != 0;
            let mut tab: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
            while !tab.is_null() {
                let mut win: *mut win_T = if tab == curtab.get() {
                    firstwin.get()
                } else {
                    (*tab).tp_firstwin
                };
                while !win.is_null() {
                    if (*win).w_buffer == obuf {
                        in_use = true_0 != 0;
                    }
                    win = (*win).w_next;
                }
                tab = (*tab).tp_next as *mut tabpage_T;
            }
            if !(*obuf).b_ml.ml_mfp.is_null() || in_use as ::core::ffi::c_int != 0 {
                if message {
                    emsg(gettext(
                        b"E95: Buffer with this name already exists\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ));
                }
                xfree(ffname as *mut ::core::ffi::c_void);
                return FAIL;
            }
            close_buffer(
                ::core::ptr::null_mut::<win_T>(),
                obuf,
                DOBUF_WIPE as ::core::ffi::c_int,
                false_0 != 0,
                false_0 != 0,
            );
        }
        sfname = xstrdup(sfname);
        if (*buf).b_sfname != (*buf).b_ffname {
            xfree((*buf).b_sfname as *mut ::core::ffi::c_void);
        }
        xfree((*buf).b_ffname as *mut ::core::ffi::c_void);
        (*buf).b_ffname = ffname;
        (*buf).b_sfname = sfname;
    }
    (*buf).b_fname = (*buf).b_sfname;
    if !file_id_valid {
        (*buf).file_id_valid = false_0 != 0;
    } else {
        (*buf).file_id_valid = true_0 != 0;
        (*buf).file_id = file_id;
    }
    buf_name_changed(buf);
    return OK;
}
pub unsafe extern "C" fn buf_set_name(
    mut fnum: ::core::ffi::c_int,
    mut name: *mut ::core::ffi::c_char,
) {
    let mut buf: *mut buf_T = buflist_findnr(fnum);
    if buf.is_null() {
        return;
    }
    if (*buf).b_sfname != (*buf).b_ffname {
        xfree((*buf).b_sfname as *mut ::core::ffi::c_void);
    }
    xfree((*buf).b_ffname as *mut ::core::ffi::c_void);
    (*buf).b_ffname = xstrdup(name);
    (*buf).b_sfname = ::core::ptr::null_mut::<::core::ffi::c_char>();
    fname_expand(buf, &raw mut (*buf).b_ffname, &raw mut (*buf).b_sfname);
    (*buf).b_fname = (*buf).b_sfname;
}
pub unsafe extern "C" fn buf_name_changed(mut buf: *mut buf_T) {
    if !(*buf).b_ml.ml_mfp.is_null() {
        ml_setname(buf);
    }
    if (*curwin.get()).w_buffer == buf {
        check_arg_idx(curwin.get());
    }
    maketitle();
    status_redraw_all();
    fmarks_check_names(buf);
    ml_timestamp(buf);
}
pub unsafe extern "C" fn setaltfname(
    mut ffname: *mut ::core::ffi::c_char,
    mut sfname: *mut ::core::ffi::c_char,
    mut lnum: linenr_T,
) -> *mut buf_T {
    let mut buf: *mut buf_T = buflist_new(ffname, sfname, lnum, 0 as ::core::ffi::c_int);
    if !buf.is_null()
        && (*cmdmod.ptr()).cmod_flags & CMOD_KEEPALT as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        (*curwin.get()).w_alt_fnum = (*buf).handle as ::core::ffi::c_int;
    }
    return buf;
}
pub unsafe extern "C" fn getaltfname(mut errmsg: bool) -> *mut ::core::ffi::c_char {
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut dummy: linenr_T = 0;
    if buflist_name_nr(0 as ::core::ffi::c_int, &raw mut fname, &raw mut dummy) == FAIL {
        if errmsg {
            emsg(gettext(&raw const e_noalt as *const ::core::ffi::c_char));
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return fname;
}
pub unsafe extern "C" fn buflist_add(
    mut fname: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut buf: *mut buf_T = buflist_new(
        fname,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        0 as linenr_T,
        flags,
    );
    if !buf.is_null() {
        return (*buf).handle as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn buflist_altfpos(mut win: *mut win_T) {
    buflist_setfpos(
        curbuf.get(),
        win,
        (*win).w_cursor.lnum,
        (*win).w_cursor.col,
        true_0 != 0,
    );
}
pub unsafe extern "C" fn otherfile(mut ffname: *mut ::core::ffi::c_char) -> bool {
    return otherfile_buf(
        curbuf.get(),
        ffname,
        ::core::ptr::null_mut::<FileID>(),
        false_0 != 0,
    );
}
unsafe extern "C" fn otherfile_buf(
    mut buf: *mut buf_T,
    mut ffname: *mut ::core::ffi::c_char,
    mut file_id_p: *mut FileID,
    mut file_id_valid: bool,
) -> bool {
    if ffname.is_null() || *ffname as ::core::ffi::c_int == NUL || (*buf).b_ffname.is_null() {
        return true_0 != 0;
    }
    if path_fnamecmp(ffname, (*buf).b_ffname) == 0 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    let mut file_id: FileID = FileID {
        inode: 0,
        device_id: 0,
    };
    if file_id_p.is_null() {
        file_id_p = &raw mut file_id;
        file_id_valid = os_fileid(ffname, file_id_p);
    }
    if !file_id_valid {
        return true_0 != 0;
    }
    if buf_same_file_id(buf, file_id_p) {
        buf_set_file_id(buf);
        if buf_same_file_id(buf, file_id_p) {
            return false_0 != 0;
        }
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn buf_set_file_id(mut buf: *mut buf_T) {
    let mut file_id: FileID = FileID {
        inode: 0,
        device_id: 0,
    };
    if !(*buf).b_fname.is_null()
        && os_fileid((*buf).b_fname, &raw mut file_id) as ::core::ffi::c_int != 0
    {
        (*buf).file_id_valid = true_0 != 0;
        (*buf).file_id = file_id;
    } else {
        (*buf).file_id_valid = false_0 != 0;
    };
}
unsafe extern "C" fn buf_same_file_id(mut buf: *mut buf_T, mut file_id: *mut FileID) -> bool {
    return (*buf).file_id_valid as ::core::ffi::c_int != 0
        && os_fileid_equal(&raw mut (*buf).file_id, file_id) as ::core::ffi::c_int != 0;
}
pub unsafe extern "C" fn fileinfo(
    mut fullname: ::core::ffi::c_int,
    mut shorthelp: ::core::ffi::c_int,
    mut dont_truncate: bool,
) {
    let mut buffer: *mut ::core::ffi::c_char =
        xmalloc(IOSIZE as size_t) as *mut ::core::ffi::c_char;
    let mut bufferlen: size_t = 0 as size_t;
    if fullname > 1 as ::core::ffi::c_int {
        bufferlen = vim_snprintf_safelen(
            buffer,
            IOSIZE as size_t,
            b"buf %d: \0".as_ptr() as *const ::core::ffi::c_char,
            (*curbuf.get()).handle,
        );
    }
    let c2rust_fresh6 = bufferlen;
    bufferlen = bufferlen.wrapping_add(1);
    *buffer.offset(c2rust_fresh6 as isize) = '"' as ::core::ffi::c_char;
    let mut name: *mut ::core::ffi::c_char = buf_spname(curbuf.get());
    if !name.is_null() {
        bufferlen = bufferlen.wrapping_add(vim_snprintf_safelen(
            buffer.offset(bufferlen as isize),
            (IOSIZE as size_t).wrapping_sub(bufferlen),
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            name,
        ));
    } else {
        name = if fullname == 0 && !(*curbuf.get()).b_fname.is_null() {
            (*curbuf.get()).b_fname
        } else {
            (*curbuf.get()).b_ffname
        };
        home_replace(
            if shorthelp != 0 {
                curbuf.get()
            } else {
                ::core::ptr::null_mut::<buf_T>()
            },
            name,
            buffer.offset(bufferlen as isize),
            (IOSIZE as size_t).wrapping_sub(bufferlen),
            true_0 != 0,
        );
        bufferlen = bufferlen.wrapping_add(strlen(buffer.offset(bufferlen as isize)));
    }
    let mut dontwrite: bool = bt_dontwrite(curbuf.get());
    bufferlen = bufferlen.wrapping_add(vim_snprintf_safelen(
        buffer.offset(bufferlen as isize),
        (IOSIZE as size_t).wrapping_sub(bufferlen),
        b"\"%s%s%s%s%s%s\0".as_ptr() as *const ::core::ffi::c_char,
        if curbufIsChanged() as ::core::ffi::c_int != 0 {
            if shortmess(SHM_MOD as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
                b" [+]\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                gettext(b" [Modified]\0".as_ptr() as *const ::core::ffi::c_char)
                    as *const ::core::ffi::c_char
            }
        } else {
            b" \0".as_ptr() as *const ::core::ffi::c_char
        },
        if (*curbuf.get()).b_flags & BF_NOTEDITED != 0 && !dontwrite {
            gettext(b"[Not edited]\0".as_ptr() as *const ::core::ffi::c_char)
                as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
        if (*curbuf.get()).b_flags & BF_NEW != 0 && !dontwrite {
            gettext(b"[New]\0".as_ptr() as *const ::core::ffi::c_char) as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
        if (*curbuf.get()).b_flags & BF_READERR != 0 {
            gettext(b"[Read errors]\0".as_ptr() as *const ::core::ffi::c_char)
                as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
        if (*curbuf.get()).b_p_ro != 0 {
            (if shortmess(SHM_RO as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
                gettext(b"[RO]\0".as_ptr() as *const ::core::ffi::c_char)
            } else {
                gettext(b"[readonly]\0".as_ptr() as *const ::core::ffi::c_char)
            }) as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
        if curbufIsChanged() as ::core::ffi::c_int != 0
            || (*curbuf.get()).b_flags & BF_WRITE_MASK != 0
            || (*curbuf.get()).b_p_ro != 0
        {
            b" \0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
    ));
    if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
        bufferlen = bufferlen.wrapping_add(vim_snprintf_safelen(
            buffer.offset(bufferlen as isize),
            (IOSIZE as size_t).wrapping_sub(bufferlen),
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            gettext(no_lines_msg.ptr() as *mut ::core::ffi::c_char),
        ));
    } else if p_ru.get() != 0 {
        bufferlen = bufferlen.wrapping_add(vim_snprintf_safelen(
            buffer.offset(bufferlen as isize),
            (IOSIZE as size_t).wrapping_sub(bufferlen),
            ngettext(
                b"%ld line --%d%%--\0".as_ptr() as *const ::core::ffi::c_char,
                b"%ld lines --%d%%--\0".as_ptr() as *const ::core::ffi::c_char,
                (*curbuf.get()).b_ml.ml_line_count as ::core::ffi::c_ulong,
            ),
            (*curbuf.get()).b_ml.ml_line_count as int64_t,
            calc_percentage(
                (*curwin.get()).w_cursor.lnum as int64_t,
                (*curbuf.get()).b_ml.ml_line_count as int64_t,
            ),
        ));
    } else {
        bufferlen = bufferlen.wrapping_add(vim_snprintf_safelen(
            buffer.offset(bufferlen as isize),
            (IOSIZE as size_t).wrapping_sub(bufferlen),
            gettext(b"line %ld of %ld --%d%%-- col \0".as_ptr() as *const ::core::ffi::c_char),
            (*curwin.get()).w_cursor.lnum as int64_t,
            (*curbuf.get()).b_ml.ml_line_count as int64_t,
            calc_percentage(
                (*curwin.get()).w_cursor.lnum as int64_t,
                (*curbuf.get()).b_ml.ml_line_count as int64_t,
            ),
        ));
        validate_virtcol(curwin.get());
        bufferlen = bufferlen.wrapping_add(col_print(
            buffer.offset(bufferlen as isize),
            (IOSIZE as size_t).wrapping_sub(bufferlen),
            (*curwin.get()).w_cursor.col as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
            (*curwin.get()).w_virtcol as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
        ) as size_t);
    }
    append_arg_number(
        curwin.get(),
        buffer.offset(bufferlen as isize),
        (IOSIZE as size_t).wrapping_sub(bufferlen),
    );
    if dont_truncate {
        msg_start();
        let mut n: ::core::ffi::c_int = msg_scroll.get();
        msg_scroll.set(true_0);
        msg(buffer, 0 as ::core::ffi::c_int);
        msg_scroll.set(n);
    } else {
        let mut p: *mut ::core::ffi::c_char =
            msg_trunc(buffer, false_0 != 0, 0 as ::core::ffi::c_int);
        if restart_edit.get() != 0 as ::core::ffi::c_int
            || msg_scrolled.get() != 0 && !need_wait_return.get()
        {
            set_keep_msg(p, 0 as ::core::ffi::c_int);
        }
    }
    xfree(buffer as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn col_print(
    mut buf: *mut ::core::ffi::c_char,
    mut buflen: size_t,
    mut col: ::core::ffi::c_int,
    mut vcol: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if col == vcol {
        return vim_snprintf_safelen(
            buf,
            buflen,
            b"%d\0".as_ptr() as *const ::core::ffi::c_char,
            col,
        ) as ::core::ffi::c_int;
    }
    return vim_snprintf_safelen(
        buf,
        buflen,
        b"%d-%d\0".as_ptr() as *const ::core::ffi::c_char,
        col,
        vcol,
    ) as ::core::ffi::c_int;
}
static lasttitle: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static lasticon: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub unsafe extern "C" fn maketitle() {
    let mut title_str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut icon_str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut buf: [::core::ffi::c_char; 1025] = [0; 1025];
    if !redrawing() {
        need_maketitle.set(true_0 != 0);
        return;
    }
    need_maketitle.set(false_0 != 0);
    if p_title.get() == 0
        && p_icon.get() == 0
        && (*lasttitle.ptr()).is_null()
        && (*lasticon.ptr()).is_null()
    {
        return;
    }
    if p_title.get() != 0 {
        let mut maxlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if p_titlelen.get() > 0 as OptInt {
            maxlen = if (p_titlelen.get() * Columns.get() as OptInt / 100 as OptInt)
                as ::core::ffi::c_int
                > 10 as ::core::ffi::c_int
            {
                (p_titlelen.get() * Columns.get() as OptInt / 100 as OptInt) as ::core::ffi::c_int
            } else {
                10 as ::core::ffi::c_int
            };
        }
        if *p_titlestring.get() as ::core::ffi::c_int != NUL {
            if stl_syntax.get() & STL_IN_TITLE != 0 {
                build_stl_str_hl(
                    curwin.get(),
                    &raw mut buf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
                    p_titlestring.get(),
                    kOptTitlestring,
                    0 as ::core::ffi::c_int,
                    0 as schar_T,
                    maxlen,
                    ::core::ptr::null_mut::<*mut stl_hlrec_t>(),
                    ::core::ptr::null_mut::<size_t>(),
                    ::core::ptr::null_mut::<*mut StlClickRecord>(),
                    ::core::ptr::null_mut::<statuscol_T>(),
                );
                title_str = &raw mut buf as *mut ::core::ffi::c_char;
            } else {
                title_str = p_titlestring.get();
            }
        } else {
            let mut default_titlestring: *mut ::core::ffi::c_char =
                b"%t%( %M%)%( (%{expand('%:p:~:h')})%)%a - Nvim\0".as_ptr()
                    as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            build_stl_str_hl(
                curwin.get(),
                &raw mut buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
                default_titlestring,
                kOptTitlestring,
                0 as ::core::ffi::c_int,
                0 as schar_T,
                maxlen,
                ::core::ptr::null_mut::<*mut stl_hlrec_t>(),
                ::core::ptr::null_mut::<size_t>(),
                ::core::ptr::null_mut::<*mut StlClickRecord>(),
                ::core::ptr::null_mut::<statuscol_T>(),
            );
            title_str = &raw mut buf as *mut ::core::ffi::c_char;
        }
    }
    let mut mustset: bool = value_change(title_str, lasttitle.ptr());
    if p_icon.get() != 0 {
        icon_str = &raw mut buf as *mut ::core::ffi::c_char;
        if *p_iconstring.get() as ::core::ffi::c_int != NUL {
            if stl_syntax.get() & STL_IN_ICON != 0 {
                build_stl_str_hl(
                    curwin.get(),
                    icon_str,
                    ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
                    p_iconstring.get(),
                    kOptIconstring,
                    0 as ::core::ffi::c_int,
                    0 as schar_T,
                    0 as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<*mut stl_hlrec_t>(),
                    ::core::ptr::null_mut::<size_t>(),
                    ::core::ptr::null_mut::<*mut StlClickRecord>(),
                    ::core::ptr::null_mut::<statuscol_T>(),
                );
            } else {
                icon_str = p_iconstring.get();
            }
        } else {
            let mut name: *mut ::core::ffi::c_char = buf_spname(curbuf.get());
            if name.is_null() {
                name = path_tail((*curbuf.get()).b_ffname);
            }
            let mut namelen: ::core::ffi::c_int = strlen(name) as ::core::ffi::c_int;
            if namelen > 100 as ::core::ffi::c_int {
                namelen -= 100 as ::core::ffi::c_int;
                namelen += utf_cp_bounds(name, name.offset(namelen as isize)).end_off
                    as ::core::ffi::c_int;
                name = name.offset(namelen as isize);
            }
            strcpy(&raw mut buf as *mut ::core::ffi::c_char, name);
            trans_characters(
                &raw mut buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 1025]>() as ::core::ffi::c_int,
            );
        }
    }
    mustset = mustset as ::core::ffi::c_int
        | value_change(icon_str, lasticon.ptr()) as ::core::ffi::c_int
        != 0;
    if mustset {
        resettitle();
    }
}
unsafe extern "C" fn value_change(
    mut str: *mut ::core::ffi::c_char,
    mut last: *mut *mut ::core::ffi::c_char,
) -> bool {
    if str.is_null() as ::core::ffi::c_int != (*last).is_null() as ::core::ffi::c_int
        || !str.is_null() && !(*last).is_null() && strcmp(str, *last) != 0 as ::core::ffi::c_int
    {
        xfree(*last as *mut ::core::ffi::c_void);
        if str.is_null() {
            *last = ::core::ptr::null_mut::<::core::ffi::c_char>();
            resettitle();
        } else {
            *last = xstrdup(str);
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn resettitle() {
    ui_call_set_icon(cstr_as_string(lasticon.get()));
    ui_call_set_title(cstr_as_string(lasttitle.get()));
}
pub unsafe extern "C" fn get_rel_pos(
    mut wp: *mut win_T,
    mut buf: *mut ::core::ffi::c_char,
    mut buflen: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if buflen < 3 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    let mut above: linenr_T = 0;
    let mut below: linenr_T = 0;
    above = (*wp).w_topline - 1 as linenr_T;
    above = (above as ::core::ffi::c_int + (win_get_fill(wp, (*wp).w_topline) - (*wp).w_topfill))
        as linenr_T;
    if (*wp).w_topline == 1 as linenr_T && (*wp).w_topfill >= 1 as ::core::ffi::c_int {
        above = 0 as ::core::ffi::c_int as linenr_T;
    }
    below = (*(*wp).w_buffer).b_ml.ml_line_count - (*wp).w_botline + 1 as linenr_T;
    if below <= 0 as linenr_T {
        return vim_snprintf_safelen(
            buf,
            buflen as size_t,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            if above == 0 as linenr_T {
                gettext(b"All\0".as_ptr() as *const ::core::ffi::c_char)
            } else {
                gettext(b"Bot\0".as_ptr() as *const ::core::ffi::c_char)
            },
        ) as ::core::ffi::c_int;
    }
    if above <= 0 as linenr_T {
        return vim_snprintf_safelen(
            buf,
            buflen as size_t,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            gettext(b"Top\0".as_ptr() as *const ::core::ffi::c_char),
        ) as ::core::ffi::c_int;
    }
    let mut perc: ::core::ffi::c_int =
        calc_percentage(above as int64_t, (above + below) as int64_t);
    let mut tmp: [::core::ffi::c_char; 8] = [0; 8];
    vim_snprintf(
        &raw mut tmp as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>(),
        gettext(b"%d%%\0".as_ptr() as *const ::core::ffi::c_char),
        perc,
    );
    return vim_snprintf_safelen(
        buf,
        buflen as size_t,
        gettext(b"%3s\0".as_ptr() as *const ::core::ffi::c_char),
        &raw mut tmp as *mut ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
}
pub unsafe extern "C" fn append_arg_number(
    mut wp: *mut win_T,
    mut buf: *mut ::core::ffi::c_char,
    mut buflen: size_t,
) -> ::core::ffi::c_int {
    if (*(*curwin.get()).w_alist).al_ga.ga_len <= 1 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    let mut msg_0: *const ::core::ffi::c_char = if (*wp).w_arg_idx_invalid != 0 {
        gettext(b" ((%d) of %d)\0".as_ptr() as *const ::core::ffi::c_char)
    } else {
        gettext(b" (%d of %d)\0".as_ptr() as *const ::core::ffi::c_char)
    };
    return vim_snprintf_safelen(
        buf,
        buflen,
        msg_0,
        (*wp).w_arg_idx + 1 as ::core::ffi::c_int,
        (*(*curwin.get()).w_alist).al_ga.ga_len,
    ) as ::core::ffi::c_int;
}
pub unsafe extern "C" fn fname_expand(
    mut _buf: *mut buf_T,
    mut ffname: *mut *mut ::core::ffi::c_char,
    mut sfname: *mut *mut ::core::ffi::c_char,
) {
    if (*ffname).is_null() {
        return;
    }
    if (*sfname).is_null() {
        *sfname = *ffname;
    }
    *ffname = fix_fname(*ffname);
}
pub unsafe extern "C" fn bt_prompt(mut buf: *mut buf_T) -> bool {
    return !buf.is_null()
        && *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'p' as ::core::ffi::c_int;
}
pub unsafe extern "C" fn ex_buffer_all(mut eap: *mut exarg_T) {
    let mut wpnext: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut split_ret: ::core::ffi::c_int = OK;
    let mut open_wins: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut had_tab: ::core::ffi::c_int = (*cmdmod.ptr()).cmod_tab;
    let mut count: linenr_T = if (*eap).addr_count == 0 as ::core::ffi::c_int {
        9999 as linenr_T
    } else {
        (*eap).line2
    };
    let mut all: ::core::ffi::c_int = ((*eap).cmdidx as ::core::ffi::c_int
        != CMD_unhide as ::core::ffi::c_int
        && (*eap).cmdidx as ::core::ffi::c_int != CMD_sunhide as ::core::ffi::c_int)
        as ::core::ffi::c_int;
    reset_VIsual_and_resel();
    setpcmark();
    if had_tab > 0 as ::core::ffi::c_int {
        goto_tabpage_tp(first_tabpage.get(), true_0 != 0, true_0 != 0);
    }
    loop {
        let mut tpnext: *mut tabpage_T = (*curtab.get()).tp_next;
        let mut wp: *mut win_T = if (*lastwin.get()).w_floating as ::core::ffi::c_int != 0 {
            lastwin.get()
        } else {
            firstwin.get()
        };
        while !wp.is_null() {
            wpnext = if (*wp).w_floating as ::core::ffi::c_int != 0 {
                if (*(*wp).w_prev).w_floating as ::core::ffi::c_int != 0 {
                    (*wp).w_prev
                } else {
                    firstwin.get()
                }
            } else if (*wp).w_next.is_null()
                || (*(*wp).w_next).w_floating as ::core::ffi::c_int != 0
            {
                ::core::ptr::null_mut::<win_T>()
            } else {
                (*wp).w_next
            };
            if ((*(*wp).w_buffer).b_nwindows > 1 as ::core::ffi::c_int
                || (*wp).w_floating as ::core::ffi::c_int != 0
                || (if (*cmdmod.ptr()).cmod_split & WSP_VERT as ::core::ffi::c_int != 0 {
                    ((((*wp).w_height + (*wp).w_hsep_height + (*wp).w_status_height) as OptInt)
                        < Rows.get() as OptInt
                            - p_ch.get()
                            - tabline_height() as OptInt
                            - global_stl_height() as OptInt)
                        as ::core::ffi::c_int
                } else {
                    ((*wp).w_width != Columns.get()) as ::core::ffi::c_int
                }) != 0
                || had_tab > 0 as ::core::ffi::c_int && wp != firstwin.get())
                && !(firstwin.get() == lastwin.get())
                && !(win_locked(wp) != 0 || (*(*wp).w_buffer).b_locked > 0 as ::core::ffi::c_int)
                && !is_aucmd_win(wp)
            {
                if win_close(wp, false_0 != 0, false_0 != 0) == FAIL {
                    break;
                }
                wpnext = if (*lastwin.get()).w_floating as ::core::ffi::c_int != 0 {
                    lastwin.get()
                } else {
                    firstwin.get()
                };
                tpnext = first_tabpage.get();
                open_wins = 0 as ::core::ffi::c_int;
            } else {
                open_wins += 1;
            }
            wp = wpnext;
        }
        if had_tab == 0 as ::core::ffi::c_int || tpnext.is_null() {
            break;
        }
        goto_tabpage_tp(tpnext, true_0 != 0, true_0 != 0);
    }
    (*autocmd_no_enter.ptr()) += 1;
    win_enter(
        lastwin_nofloating(::core::ptr::null_mut::<tabpage_T>()),
        false_0 != 0,
    );
    (*autocmd_no_leave.ptr()) += 1;
    let mut buf: *mut buf_T = firstbuf.get();
    's_295: while !buf.is_null() && (open_wins as linenr_T) < count {
        's_111: {
            if !(all == 0 && (*buf).b_ml.ml_mfp.is_null() || (*buf).b_p_bl == 0) {
                let mut wp_0: *mut win_T = ::core::ptr::null_mut::<win_T>();
                if had_tab != 0 as ::core::ffi::c_int {
                    wp_0 = if (*buf).b_nwindows > 0 as ::core::ffi::c_int {
                        lastwin.get()
                    } else {
                        ::core::ptr::null_mut::<win_T>()
                    };
                } else {
                    wp_0 = firstwin.get();
                    while !wp_0.is_null() {
                        if !(*wp_0).w_floating && (*wp_0).w_buffer == buf {
                            break;
                        }
                        wp_0 = (*wp_0).w_next;
                    }
                    if !wp_0.is_null() {
                        win_move_after(wp_0, curwin.get());
                    }
                }
                if wp_0.is_null() && split_ret == OK {
                    let mut bufref: bufref_T = bufref_T {
                        br_buf: ::core::ptr::null_mut::<buf_T>(),
                        br_fnum: 0,
                        br_buf_free_count: 0,
                    };
                    set_bufref(&raw mut bufref, buf);
                    let mut p_ea_save: bool = p_ea.get() != 0;
                    p_ea.set(true_0);
                    split_ret = win_split(
                        0 as ::core::ffi::c_int,
                        WSP_ROOM as ::core::ffi::c_int | WSP_BELOW as ::core::ffi::c_int,
                    );
                    open_wins += 1;
                    p_ea.set(p_ea_save as ::core::ffi::c_int);
                    if split_ret == FAIL {
                        break 's_111;
                    } else {
                        swap_exists_action.set(SEA_DIALOG);
                        set_curbuf(
                            buf,
                            DOBUF_GOTO as ::core::ffi::c_int,
                            jop_flags.get()
                                & kOptJopFlagClean as ::core::ffi::c_int as ::core::ffi::c_uint
                                == 0,
                        );
                        if !bufref_valid(&raw mut bufref) {
                            swap_exists_action.set(SEA_NONE);
                            break 's_295;
                        } else if swap_exists_action.get() == SEA_QUIT {
                            let mut cs: cleanup_T = cleanup_T {
                                pending: 0,
                                exception: ::core::ptr::null_mut::<except_T>(),
                            };
                            enter_cleanup(&raw mut cs);
                            win_close(curwin.get(), true_0 != 0, false_0 != 0);
                            open_wins -= 1;
                            swap_exists_action.set(SEA_NONE);
                            swap_exists_did_quit.set(true_0 != 0);
                            leave_cleanup(&raw mut cs);
                        } else {
                            handle_swap_exists(::core::ptr::null_mut::<bufref_T>());
                        }
                    }
                }
                os_breakcheck();
                if got_int.get() {
                    vgetc();
                    break 's_295;
                } else {
                    if aborting() {
                        break 's_295;
                    }
                    if had_tab > 0 as ::core::ffi::c_int
                        && tabpage_index(::core::ptr::null_mut::<tabpage_T>()) as OptInt
                            <= p_tpm.get()
                    {
                        (*cmdmod.ptr()).cmod_tab = 9999 as ::core::ffi::c_int;
                    }
                }
            }
        }
        buf = (*buf).b_next;
    }
    (*autocmd_no_enter.ptr()) -= 1;
    win_enter(firstwin.get(), false_0 != 0);
    (*autocmd_no_leave.ptr()) -= 1;
    let mut wp_1: *mut win_T = lastwin.get();
    while open_wins as linenr_T > count {
        let mut r: bool = (buf_hide((*wp_1).w_buffer) as ::core::ffi::c_int != 0
            || !bufIsChanged((*wp_1).w_buffer)
            || autowrite((*wp_1).w_buffer, false_0 != 0) == OK)
            && !is_aucmd_win(wp_1);
        if !win_valid(wp_1) {
            wp_1 = lastwin.get();
        } else if r {
            win_close(wp_1, !buf_hide((*wp_1).w_buffer), false_0 != 0);
            open_wins -= 1;
            wp_1 = lastwin.get();
        } else {
            wp_1 = (*wp_1).w_prev;
            if wp_1.is_null() {
                break;
            }
        }
    }
}
pub unsafe extern "C" fn do_modelines(mut flags: ::core::ffi::c_int) {
    let mut lnum: linenr_T = 0;
    let mut nmlines: ::core::ffi::c_int = 0;
    static entered: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    if (*curbuf.get()).b_p_ml == 0 || {
        nmlines = p_mls.get() as ::core::ffi::c_int;
        nmlines == 0 as ::core::ffi::c_int
    } {
        return;
    }
    if entered.get() != 0 {
        return;
    }
    (*entered.ptr()) += 1;
    lnum = 1 as ::core::ffi::c_int as linenr_T;
    while (*curbuf.get()).b_p_ml != 0
        && lnum <= (*curbuf.get()).b_ml.ml_line_count
        && lnum <= nmlines as linenr_T
    {
        if chk_modeline(lnum, flags) == FAIL {
            nmlines = 0 as ::core::ffi::c_int;
        }
        lnum += 1;
    }
    lnum = (*curbuf.get()).b_ml.ml_line_count;
    while (*curbuf.get()).b_p_ml != 0
        && lnum > 0 as linenr_T
        && lnum > nmlines as linenr_T
        && lnum > (*curbuf.get()).b_ml.ml_line_count - nmlines as linenr_T
    {
        if chk_modeline(lnum, flags) == FAIL {
            nmlines = 0 as ::core::ffi::c_int;
        }
        lnum -= 1;
    }
    (*entered.ptr()) -= 1;
}
unsafe extern "C" fn chk_modeline(
    mut lnum: linenr_T,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut e: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut retval: ::core::ffi::c_int = OK;
    let mut prev: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut s: *mut ::core::ffi::c_char = ml_get(lnum);
    let mut line_end: *mut ::core::ffi::c_char = s.offset(ml_get_len(lnum) as isize);
    's_91: while *s as ::core::ffi::c_int != NUL {
        's_24: {
            if prev == -1 as ::core::ffi::c_int || ascii_isspace(prev) as ::core::ffi::c_int != 0 {
                if prev != -1 as ::core::ffi::c_int
                    && strncmp(
                        s,
                        b"ex:\0".as_ptr() as *const ::core::ffi::c_char,
                        3 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    || strncmp(
                        s,
                        b"vi:\0".as_ptr() as *const ::core::ffi::c_char,
                        3 as size_t,
                    ) == 0 as ::core::ffi::c_int
                {
                    break 's_91;
                }
                if (*s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'v' as ::core::ffi::c_int
                    || *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == 'V' as ::core::ffi::c_int)
                    && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == 'i' as ::core::ffi::c_int
                    && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == 'm' as ::core::ffi::c_int
                {
                    if *s.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '<' as ::core::ffi::c_int
                        || *s.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '=' as ::core::ffi::c_int
                        || *s.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '>' as ::core::ffi::c_int
                    {
                        e = s.offset(4 as ::core::ffi::c_int as isize);
                    } else {
                        e = s.offset(3 as ::core::ffi::c_int as isize);
                    }
                    let mut vers: intmax_t = 0;
                    if !try_getdigits(&raw mut e, &raw mut vers) {
                        break 's_24;
                    } else {
                        let vim_version: ::core::ffi::c_int = min_vim_version();
                        if *e as ::core::ffi::c_int == ':' as ::core::ffi::c_int
                            && (*s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                != 'V' as ::core::ffi::c_int
                                || strncmp(
                                    skipwhite(e.offset(1 as ::core::ffi::c_int as isize)),
                                    b"set\0".as_ptr() as *const ::core::ffi::c_char,
                                    3 as size_t,
                                ) == 0 as ::core::ffi::c_int)
                            && (*s.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == ':' as ::core::ffi::c_int
                                || vim_version as intmax_t >= vers
                                    && *(*__ctype_b_loc()).offset(
                                        *s.offset(3 as ::core::ffi::c_int as isize) as uint8_t
                                            as ::core::ffi::c_int
                                            as isize,
                                    ) as ::core::ffi::c_int
                                        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort
                                            as ::core::ffi::c_int
                                        != 0
                                || (vim_version as intmax_t) < vers
                                    && *s.offset(3 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == '<' as ::core::ffi::c_int
                                || vim_version as intmax_t > vers
                                    && *s.offset(3 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == '>' as ::core::ffi::c_int
                                || vim_version as intmax_t == vers
                                    && *s.offset(3 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == '=' as ::core::ffi::c_int)
                        {
                            break 's_91;
                        }
                    }
                }
            }
            prev = *s as uint8_t as ::core::ffi::c_int;
        }
        s = s.offset(1);
    }
    if *s == 0 {
        return retval;
    }
    loop {
        s = s.offset(1);
        if *s.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == ':' as ::core::ffi::c_int
        {
            break;
        }
    }
    let mut len: size_t = line_end.offset_from(s) as size_t;
    let mut linecopy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    linecopy = xstrnsave(s, len);
    s = linecopy;
    line_end = s.offset(len as isize);
    estack_push(
        ETYPE_MODELINE,
        b"modelines\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        lnum,
    );
    let mut end: bool = false_0 != 0;
    while end as ::core::ffi::c_int == false_0 {
        s = skipwhite(s);
        if *s as ::core::ffi::c_int == NUL {
            break;
        }
        e = s;
        while *e as ::core::ffi::c_int != ':' as ::core::ffi::c_int
            && *e as ::core::ffi::c_int != NUL
        {
            if *e.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
                && *e.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ':' as ::core::ffi::c_int
            {
                memmove(
                    e as *mut ::core::ffi::c_void,
                    e.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    (line_end.offset_from(e.offset(1 as ::core::ffi::c_int as isize)) as size_t)
                        .wrapping_add(1 as size_t),
                );
                line_end = line_end.offset(-1);
            }
            e = e.offset(1);
        }
        if *e as ::core::ffi::c_int == NUL {
            end = true_0 != 0;
        }
        if strncmp(
            s,
            b"set \0".as_ptr() as *const ::core::ffi::c_char,
            4 as size_t,
        ) == 0 as ::core::ffi::c_int
            || strncmp(
                s,
                b"se \0".as_ptr() as *const ::core::ffi::c_char,
                3 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            if *e as ::core::ffi::c_int != ':' as ::core::ffi::c_int {
                break;
            } else {
                end = true_0 != 0;
                s = s.offset(
                    (if *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ' ' as ::core::ffi::c_int
                    {
                        3 as ::core::ffi::c_int
                    } else {
                        4 as ::core::ffi::c_int
                    }) as isize,
                );
            }
        }
        *e = NUL as ::core::ffi::c_char;
        if *s as ::core::ffi::c_int != NUL {
            let secure_save: ::core::ffi::c_int = secure.get();
            let save_current_sctx: sctx_T = current_sctx.get();
            (*current_sctx.ptr()).sc_sid = SID_MODELINE as scid_T;
            (*current_sctx.ptr()).sc_seq = 0 as ::core::ffi::c_int;
            (*current_sctx.ptr()).sc_lnum = lnum;
            secure.set(1 as ::core::ffi::c_int);
            retval = do_set(
                s,
                OPT_MODELINE as ::core::ffi::c_int | OPT_LOCAL as ::core::ffi::c_int | flags,
            );
            secure.set(secure_save);
            current_sctx.set(save_current_sctx);
            if retval == FAIL {
                break;
            }
        }
        s = if e == line_end {
            e
        } else {
            e.offset(1 as ::core::ffi::c_int as isize)
        };
    }
    estack_pop();
    xfree(linecopy as *mut ::core::ffi::c_void);
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn bt_help(buf: *const buf_T) -> bool {
    return !buf.is_null() && (*buf).b_help as ::core::ffi::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn bt_normal(buf: *const buf_T) -> bool {
    return !buf.is_null()
        && *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL;
}
#[no_mangle]
pub unsafe extern "C" fn bt_quickfix(buf: *const buf_T) -> bool {
    return !buf.is_null()
        && *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'q' as ::core::ffi::c_int;
}
pub unsafe extern "C" fn bt_terminal(buf: *const buf_T) -> bool {
    return !buf.is_null()
        && *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 't' as ::core::ffi::c_int;
}
pub unsafe extern "C" fn bt_nofilename(buf: *const buf_T) -> bool {
    return !buf.is_null()
        && (*(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'n' as ::core::ffi::c_int
            && *(*buf).b_p_bt.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'f' as ::core::ffi::c_int
            || *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'a' as ::core::ffi::c_int
            || !(*buf).terminal.is_null()
            || *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'p' as ::core::ffi::c_int);
}
unsafe extern "C" fn bt_nofileread(buf: *const buf_T) -> bool {
    return !buf.is_null()
        && (*(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'n' as ::core::ffi::c_int
            && *(*buf).b_p_bt.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'f' as ::core::ffi::c_int
            || *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 't' as ::core::ffi::c_int
            || *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'q' as ::core::ffi::c_int
            || *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'p' as ::core::ffi::c_int);
}
pub unsafe extern "C" fn bt_nofile(buf: *const buf_T) -> bool {
    return !buf.is_null()
        && *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'n' as ::core::ffi::c_int
        && *(*buf).b_p_bt.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'f' as ::core::ffi::c_int;
}
pub unsafe extern "C" fn bt_dontwrite(buf: *const buf_T) -> bool {
    return !buf.is_null()
        && (*(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'n' as ::core::ffi::c_int
            || !(*buf).terminal.is_null()
            || *(*buf).b_p_bt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'p' as ::core::ffi::c_int);
}
pub unsafe extern "C" fn bt_dontwrite_msg(buf: *const buf_T) -> bool {
    if bt_dontwrite(buf) {
        emsg(gettext(
            b"E382: Cannot write, 'buftype' option is set\0".as_ptr() as *const ::core::ffi::c_char,
        ));
        return true_0 != 0;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn buf_hide(buf: *const buf_T) -> bool {
    match *(*buf).b_p_bh.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
        117 | 119 | 100 => return false_0 != 0,
        104 => return true_0 != 0,
        _ => {}
    }
    return p_hid.get() != 0 || (*cmdmod.ptr()).cmod_flags & CMOD_HIDE as ::core::ffi::c_int != 0;
}
pub unsafe extern "C" fn buf_spname(mut buf: *mut buf_T) -> *mut ::core::ffi::c_char {
    if bt_quickfix(buf) {
        if (*buf).handle == qf_stack_get_bufnr() {
            return gettext(msg_qflist.get());
        }
        return gettext(msg_loclist.get());
    }
    if bt_nofilename(buf) {
        if !(*buf).b_fname.is_null() {
            return (*buf).b_fname;
        }
        if buf == cmdwin_buf.get() {
            return gettext(b"[Command Line]\0".as_ptr() as *const ::core::ffi::c_char);
        }
        if bt_prompt(buf) {
            return gettext(b"[Prompt]\0".as_ptr() as *const ::core::ffi::c_char);
        }
        return gettext(b"[Scratch]\0".as_ptr() as *const ::core::ffi::c_char);
    }
    if (*buf).b_fname.is_null() {
        return buf_get_fname(buf);
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn buf_get_fname(mut buf: *const buf_T) -> *mut ::core::ffi::c_char {
    if (*buf).b_fname.is_null() {
        return gettext(b"[No Name]\0".as_ptr() as *const ::core::ffi::c_char);
    }
    return (*buf).b_fname;
}
pub unsafe extern "C" fn set_buflisted(mut on: ::core::ffi::c_int) {
    if on == (*curbuf.get()).b_p_bl {
        return;
    }
    (*curbuf.get()).b_p_bl = on;
    if on != 0 {
        apply_autocmds(
            EVENT_BUFADD,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
    } else {
        apply_autocmds(
            EVENT_BUFDELETE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
    };
}
pub unsafe extern "C" fn buf_contents_changed(mut buf: *mut buf_T) -> bool {
    let mut differ: bool = true_0 != 0;
    let mut newbuf: *mut buf_T = buflist_new(
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        1 as linenr_T,
        BLN_DUMMY as ::core::ffi::c_int,
    );
    if newbuf.is_null() {
        return true_0 != 0;
    }
    let mut ea: exarg_T = exarg_T {
        arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdidx: CMD_append,
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
        errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ea_getline: None,
        cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cstack: ::core::ptr::null_mut::<cstack_T>(),
    };
    prep_exarg(&raw mut ea, buf);
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
    aucmd_prepbuf(&raw mut aco, newbuf);
    block_autocmds();
    if ml_open(curbuf.get()) == OK
        && readfile(
            (*buf).b_ffname,
            (*buf).b_fname,
            0 as linenr_T,
            0 as linenr_T,
            MAXLNUM as ::core::ffi::c_int as linenr_T,
            &raw mut ea,
            READ_NEW as ::core::ffi::c_int | READ_DUMMY as ::core::ffi::c_int,
            false_0 != 0,
        ) == OK
    {
        if (*buf).b_ml.ml_line_count == (*curbuf.get()).b_ml.ml_line_count {
            differ = false_0 != 0;
            let mut lnum: linenr_T = 1 as linenr_T;
            while lnum <= (*curbuf.get()).b_ml.ml_line_count {
                if strcmp(ml_get_buf(buf, lnum), ml_get(lnum)) != 0 as ::core::ffi::c_int {
                    differ = true_0 != 0;
                    break;
                } else {
                    lnum += 1;
                }
            }
        }
    }
    xfree(ea.cmd as *mut ::core::ffi::c_void);
    aucmd_restbuf(&raw mut aco);
    if curbuf.get() != newbuf {
        wipe_buffer(newbuf, false_0 != 0);
    }
    unblock_autocmds();
    return differ;
}
#[no_mangle]
pub unsafe extern "C" fn wipe_buffer(mut buf: *mut buf_T, mut aucmd: bool) {
    if !aucmd {
        block_autocmds();
    }
    close_buffer(
        ::core::ptr::null_mut::<win_T>(),
        buf,
        DOBUF_WIPE as ::core::ffi::c_int,
        false_0 != 0,
        true_0 != 0,
    );
    if !aucmd {
        unblock_autocmds();
    }
}
pub unsafe extern "C" fn buf_open_scratch(
    mut bufnr: handle_T,
    mut bufname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if do_ecmd(
        bufnr,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<exarg_T>(),
        ECMD_ONE as ::core::ffi::c_int as linenr_T,
        ECMD_HIDE as ::core::ffi::c_int,
        ::core::ptr::null_mut::<win_T>(),
    ) == FAIL
    {
        return FAIL;
    }
    if !bufname.is_null() {
        apply_autocmds(
            EVENT_BUFFILEPRE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        setfname(
            curbuf.get(),
            bufname,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            true_0 != 0,
        );
        apply_autocmds(
            EVENT_BUFFILEPOST,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
    }
    set_option_value_give_err(
        kOptBufhidden,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: String_0 {
                    data: b"hide\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                        .wrapping_sub(1 as size_t),
                },
            },
        },
        OPT_LOCAL as ::core::ffi::c_int,
    );
    set_option_value_give_err(
        kOptBuftype,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: String_0 {
                    data: b"nofile\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                        .wrapping_sub(1 as size_t),
                },
            },
        },
        OPT_LOCAL as ::core::ffi::c_int,
    );
    set_option_value_give_err(
        kOptSwapfile,
        OptVal {
            type_0: kOptValTypeBoolean,
            data: OptValData { boolean: kFalse },
        },
        OPT_LOCAL as ::core::ffi::c_int,
    );
    (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
    (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
    return OK;
}
pub unsafe extern "C" fn buf_is_empty(mut buf: *mut buf_T) -> bool {
    return (*buf).b_ml.ml_line_count == 1 as linenr_T
        && *ml_get_buf(buf, 1 as linenr_T) as ::core::ffi::c_int == NUL;
}
pub unsafe extern "C" fn buf_inc_changedtick(buf: *mut buf_T) {
    buf_set_changedtick(buf, buf_get_changedtick(buf) + 1 as varnumber_T);
}
pub unsafe extern "C" fn buf_set_changedtick(buf: *mut buf_T, changedtick: varnumber_T) {
    let mut old_val: typval_T = (*buf).changedtick_di.di_tv;
    let changedtick_di: *mut dictitem_T = tv_dict_find(
        (*buf).b_vars,
        b"changedtick\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    '_c2rust_label: {
        if !changedtick_di.is_null() {
        } else {
            __assert_fail(
                b"changedtick_di != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/buffer.rs\0".as_ptr() as *const ::core::ffi::c_char,
                4260 as ::core::ffi::c_uint,
                b"void buf_set_changedtick(buf_T *const, const varnumber_T)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    '_c2rust_label_0: {
        if (*changedtick_di).di_tv.v_type as ::core::ffi::c_uint
            == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"changedtick_di->di_tv.v_type == VAR_NUMBER\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/buffer.rs\0".as_ptr() as *const ::core::ffi::c_char,
                4261 as ::core::ffi::c_uint,
                b"void buf_set_changedtick(buf_T *const, const varnumber_T)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    '_c2rust_label_1: {
        if (*changedtick_di).di_tv.v_lock as ::core::ffi::c_uint
            == VAR_FIXED as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"changedtick_di->di_tv.v_lock == VAR_FIXED\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/buffer.rs\0".as_ptr() as *const ::core::ffi::c_char,
                4262 as ::core::ffi::c_uint,
                b"void buf_set_changedtick(buf_T *const, const varnumber_T)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    '_c2rust_label_2: {
        if (*changedtick_di).di_flags as ::core::ffi::c_int
            == DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int
        {
        } else {
            __assert_fail(
                b"changedtick_di->di_flags == (DI_FLAGS_RO|DI_FLAGS_FIX)\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/buffer.rs\0".as_ptr() as *const ::core::ffi::c_char,
                4265 as ::core::ffi::c_uint,
                b"void buf_set_changedtick(buf_T *const, const varnumber_T)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    '_c2rust_label_3: {
        if changedtick_di == &raw mut (*buf).changedtick_di as *mut dictitem_T {
        } else {
            __assert_fail(
                b"changedtick_di == (dictitem_T *)&buf->changedtick_di\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/buffer.rs\0".as_ptr() as *const ::core::ffi::c_char,
                4267 as ::core::ffi::c_uint,
                b"void buf_set_changedtick(buf_T *const, const varnumber_T)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    (*buf).changedtick_di.di_tv.vval.v_number = changedtick;
    if tv_dict_is_watched((*buf).b_vars) {
        (*buf).b_locked += 1;
        tv_dict_watcher_notify(
            (*buf).b_vars,
            &raw mut (*buf).changedtick_di.di_key as *mut ::core::ffi::c_char,
            &raw mut (*buf).changedtick_di.di_tv,
            &raw mut old_val,
        );
        (*buf).b_locked -= 1;
    }
}
pub unsafe extern "C" fn read_buffer_into(
    mut buf: *mut buf_T,
    mut start: linenr_T,
    mut end: linenr_T,
    mut sb: *mut StringBuilder,
) {
    '_c2rust_label: {
        if !buf.is_null() {
        } else {
            __assert_fail(
                b"buf\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/buffer.rs\0".as_ptr() as *const ::core::ffi::c_char,
                4285 as ::core::ffi::c_uint,
                b"void read_buffer_into(buf_T *, linenr_T, linenr_T, StringBuilder *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    '_c2rust_label_0: {
        if !sb.is_null() {
        } else {
            __assert_fail(
                b"sb\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/buffer.rs\0".as_ptr() as *const ::core::ffi::c_char,
                4286 as ::core::ffi::c_uint,
                b"void read_buffer_into(buf_T *, linenr_T, linenr_T, StringBuilder *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if (*buf).b_ml.ml_flags & ML_EMPTY != 0 {
        return;
    }
    let mut written: size_t = 0 as size_t;
    let mut len: size_t = 0 as size_t;
    let mut lnum: linenr_T = start;
    let mut lp: *mut ::core::ffi::c_char = ml_get_buf(buf, lnum);
    let mut lplen: size_t = ml_get_buf_len(buf, lnum) as size_t;
    loop {
        if lplen == 0 as size_t {
            len = 0 as size_t;
        } else if *lp.offset(written as isize) as ::core::ffi::c_int == NL {
            len = 1 as size_t;
            if (*sb).size == (*sb).capacity {
                (*sb).capacity = if (*sb).capacity != 0 {
                    (*sb).capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*sb).items = xrealloc(
                    (*sb).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul((*sb).capacity),
                ) as *mut ::core::ffi::c_char;
            } else {
            };
            let c2rust_fresh7 = (*sb).size;
            (*sb).size = (*sb).size.wrapping_add(1);
            *(*sb).items.offset(c2rust_fresh7 as isize) = '\0' as ::core::ffi::c_char;
        } else {
            let mut s: *mut ::core::ffi::c_char = vim_strchr(lp.offset(written as isize), NL);
            len = if s.is_null() {
                lplen.wrapping_sub(written)
            } else {
                s.offset_from(lp.offset(written as isize)) as size_t
            };
            if len > 0 as size_t {
                if (*sb).capacity < (*sb).size.wrapping_add(len) {
                    (*sb).capacity = (*sb).size.wrapping_add(len);
                    (*sb).capacity = (*sb).capacity.wrapping_sub(1);
                    (*sb).capacity |= (*sb).capacity >> 1 as ::core::ffi::c_int;
                    (*sb).capacity |= (*sb).capacity >> 2 as ::core::ffi::c_int;
                    (*sb).capacity |= (*sb).capacity >> 4 as ::core::ffi::c_int;
                    (*sb).capacity |= (*sb).capacity >> 8 as ::core::ffi::c_int;
                    (*sb).capacity |= (*sb).capacity >> 16 as ::core::ffi::c_int;
                    (*sb).capacity = (*sb).capacity.wrapping_add(1);
                    (*sb).capacity = (*sb).capacity;
                    (*sb).items = xrealloc(
                        (*sb).items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul((*sb).capacity),
                    ) as *mut ::core::ffi::c_char;
                }
                '_c2rust_label_1: {
                    if !(*sb).items.is_null() {
                    } else {
                        __assert_fail(
                            b"(*sb).items\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/buffer.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            4308 as ::core::ffi::c_uint,
                            b"void read_buffer_into(buf_T *, linenr_T, linenr_T, StringBuilder *)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                memcpy(
                    (*sb).items.offset((*sb).size as isize) as *mut ::core::ffi::c_void,
                    lp.offset(written as isize) as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(len),
                );
                (*sb).size = (*sb).size.wrapping_add(len);
            }
        }
        if len == lplen.wrapping_sub(written) {
            if lnum != end
                || (*buf).b_p_bin == 0 && (*buf).b_p_fixeol != 0
                || lnum != (*buf).b_no_eol_lnum
                    && (lnum != (*buf).b_ml.ml_line_count || (*buf).b_p_eol != 0)
            {
                if (*sb).size == (*sb).capacity {
                    (*sb).capacity = if (*sb).capacity != 0 {
                        (*sb).capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    (*sb).items = xrealloc(
                        (*sb).items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul((*sb).capacity),
                    ) as *mut ::core::ffi::c_char;
                } else {
                };
                let c2rust_fresh8 = (*sb).size;
                (*sb).size = (*sb).size.wrapping_add(1);
                *(*sb).items.offset(c2rust_fresh8 as isize) = '\n' as ::core::ffi::c_char;
            }
            lnum += 1;
            if lnum > end {
                break;
            }
            lp = ml_get_buf(buf, lnum);
            lplen = ml_get_buf_len(buf, lnum) as size_t;
            written = 0 as size_t;
        } else if len > 0 as size_t {
            written = written.wrapping_add(len);
        }
    }
}
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CPO_INTMOD: ::core::ffi::c_int = 'i' as ::core::ffi::c_int;
pub const NO_LOCAL_UNDOLEVEL: ::core::ffi::c_int = -123456 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn tv_dict_is_watched(d: *const dict_T) -> bool {
    return !d.is_null() && QUEUE_EMPTY(&raw const (*d).watchers) == 0;
}
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const SID_MODELINE: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const SEA_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEA_DIALOG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SEA_QUIT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SEA_RECOVER: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const STL_IN_ICON: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STL_IN_TITLE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __S_IFMT: ::core::ffi::c_int = 0o170000 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
