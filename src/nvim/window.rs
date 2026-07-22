use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_set_error, cstr_as_string, find_window_by_handle, try_enter, try_leave,
};
use crate::src::nvim::arglist::alist_unlink;
use crate::src::nvim::autocmd::{
    apply_autocmds, block_autocmds, event_ignored, has_event, is_aucmd_win, unblock_autocmds,
};
use crate::src::nvim::buffer::{
    bt_help, bt_prompt, bt_quickfix, buf_hide, buf_valid, buflist_findname_exp, buflist_findnr,
    buflist_getfile, buflist_new, bufref_valid, close_buffer, do_autochdir, do_buffer, maketitle,
    set_bufref,
};
use crate::src::nvim::charset::getdigits_int;
use crate::src::nvim::cursor::{check_cursor, check_cursor_lnum};
use crate::src::nvim::decoration::{clear_virttext, decor_conceal_line};
use crate::src::nvim::diff::{diff_clear, diffopt_closeoff};
use crate::src::nvim::drawscreen::{
    comp_col, redrawWinline, redraw_all_later, redraw_later, showmode, status_redraw_all,
};
use crate::src::nvim::edit::{beginline, cursor_down_inner, cursor_up_inner};
use crate::src::nvim::eval::typval::{
    tv_dict_add_dict, tv_dict_add_list, tv_dict_add_tv, tv_dict_alloc, tv_dict_extend,
    tv_dict_set_keys_readonly, tv_dict_unref, tv_list_alloc, tv_list_append_owned_tv,
};
use crate::src::nvim::eval::vars::{init_var_dict, unref_var_dict, vars_clear};
use crate::src::nvim::eval::window::{restore_win_noblock, switch_win_noblock, win_has_winnr};
use crate::src::nvim::eval_1::{get_v_event, restore_v_event};
use crate::src::nvim::ex_cmds::do_ecmd;
use crate::src::nvim::ex_cmds2::{can_abandon, dialog_changed};
use crate::src::nvim::ex_docmd::do_cmdline_cmd;
use crate::src::nvim::ex_eval::aborting;
use crate::src::nvim::ex_getln::{
    compute_cmdrow, curbuf_locked, is_in_cmdwin, text_locked, text_locked_msg, text_or_buf_locked,
};
use crate::src::nvim::file_search::{do_autocmd_dirchanged, grab_file_name};
use crate::src::nvim::fileio::shorten_fnames;
use crate::src::nvim::fold::{
    clearFolding, copyFoldingState, deleteFoldRecurse, foldInitWin, getDeepestNesting, hasFolding,
};
use crate::src::nvim::getchar::{beep_flush, plain_vgetc, typebuf_maplen};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::{
    grid_adjust, grid_assign_handle, grid_clear, grid_free, win_grid_alloc,
};
use crate::src::nvim::main::{
    allow_keys, au_pending_free_win, autocmd_busy, clear_cmdline, cmdline_row, cmdline_win, cmdmod,
    cmdwin_old_curwin, cmdwin_result, cmdwin_type, cmdwin_win, curbuf, curtab, curwin,
    default_grid, default_gridview, diff_need_scrollbind, e_autocmd_close, e_buffer_nr_not_found,
    e_cmdwin, e_floatexchange, e_floatonly, e_invarg, e_noalt, e_noroom,
    e_not_allowed_to_change_window_layout_in_this_autocmd, e_winfixbuf_cannot_go_to_buffer,
    empty_string_option, exiting, first_tabpage, firstbuf, firstwin, float_anchor_str, full_screen,
    g_do_tagpreview, getout, global_alist, globaldir, langmap_mapchar, last_chdir_reason,
    lastused_tabpage, lastwin, mode_displayed, msg_col, msg_grid_adj, msg_row, msg_scrolled,
    no_mapping, p_acd, p_ch, p_confirm, p_ea, p_ead, p_langmap, p_lrm, p_ls, p_pvh, p_ru, p_sb,
    p_spk, p_spr, p_stal, p_tpm, p_wbr, p_wh, p_window, p_wiw, p_wmh, p_wmw, p_write,
    postponed_split, postponed_split_tab, prevwin, redraw_cmdline, redraw_tabline, restart_edit,
    sc_col, skip_update_topline, skip_win_fix_cursor, skip_win_fix_scroll, starting,
    stop_insert_mode, swb_flags, tabpage_handles, tabpage_move_disallowed, tcl_flags, topframe,
    vgetc_busy, window_handles, Columns, KeyStuffed, KeyTyped, RedrawingDisabled, Rows, State,
    VIsual_active,
};
use crate::src::nvim::map::{map_del_int_ptr_t, map_put_ref_int_ptr_t};
use crate::src::nvim::mapping::langmap_adjust_mb;
use crate::src::nvim::mark::{copy_jumplist, free_jumplist, setmark, setpcmark};
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc, xmemdupz, xstrdup, xstrlcat, xstrlcpy};
use crate::src::nvim::message::{
    emsg, iemsg, internal_error, msg, msg_clr_eos_force, msg_grid_validate, msg_ui_flush, semsg,
    set_keep_msg,
};
use crate::src::nvim::mouse::{reset_dragwin, setmouse};
use crate::src::nvim::normal::{
    add_to_showcmd, check_text_or_curbuf_locked, do_nv_ident, find_ident_under_cursor,
    reset_VIsual_and_resel,
};
use crate::src::nvim::option::{
    buf_copy_options, clear_winopt, get_scrolloff_value, option_was_set, set_option_value,
    win_copy_options,
};
use crate::src::nvim::os::fs::{os_chdir, os_dirname};
use crate::src::nvim::os::libc::{
    __assert_fail, abort, abs, gettext, memmove, memset, qsort, strncmp,
};
use crate::src::nvim::path::pathcmp;
use crate::src::nvim::plines::{plines_win, plines_win_col, plines_win_nofill, win_text_height};
use crate::src::nvim::popupmenu::pum_ui_flush;
use crate::src::nvim::quickfix::qf_view_result;
use crate::src::nvim::r#match::clear_matches;
use crate::src::nvim::r#move::{
    changed_line_abv_curs, changed_line_abv_curs_win, curs_columns, invalidate_botline_win,
    set_topline, textpos2screenpos, update_topline, validate_botline_win, validate_cursor,
    win_col_off, win_col_off2,
};
use crate::src::nvim::search::find_pattern_in_path;
use crate::src::nvim::state::{get_real_state, virtual_active};
use crate::src::nvim::statusline::stl_clear_click_defs;
use crate::src::nvim::strings::vim_snprintf;
use crate::src::nvim::syntax::reset_synblock;
use crate::src::nvim::tag::tagstack_clear_entry;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, Boolean, BufUpdateCallbacks, CMD_index, Callback,
    CallbackType, Callback_data as C2Rust_Unnamed_4, CdCause, CdScope, ChangedtickDictItem,
    DecorExt, DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_1, Direction, Error, ErrorType, ExtmarkUndoObject, FileID,
    Float, FloatAnchor, FloatRelative, GridView, Integer, Intersection, LineGetter, LuaRef, MTKey,
    MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_int_ptr_t,
    Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, MotionType, OptIndex, OptInt, OptVal,
    OptValData, OptValType, ScopeDictDictItem, ScopeType, ScreenGrid, Set_int, Set_int64_t,
    Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_11, String_0, Terminal, Timestamp, TriState,
    TryState, UIExtension, VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos,
    WinConfig, WinInfo, WinSplit, WinStyle, Window, __compar_fn_t, __time_t, alist_T, aucmdwin_T,
    auto_event, bhdr_T, bln_values, blob_T, blobvar_S, blocknr_T, buf_T, bufref_T, bufstate_T,
    chunksize_T, cmd_addr_T, cmdidx_T, cmdmod_T, colnr_T, cstack_T,
    cstack_T_cs_pend as C2Rust_Unnamed_18, dict_T, dictvar_S, diff_T, diffblock_S, disptick_T,
    dobuf_action_values, dobuf_start_values, eslist_T, eslist_elem, event_T, exarg, exarg_T,
    except_T, except_type_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_2, file_buffer_b_wininfo as C2Rust_Unnamed_10,
    file_buffer_update_callbacks as C2Rust_Unnamed,
    file_buffer_update_channels as C2Rust_Unnamed_0, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_5, funccall_T, garray_T, getf_values,
    handle_T, hash_T, hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, lcs_chars_T,
    linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T,
    mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T,
    msglist, msglist_T, mtnode_inner_s, mtnode_s, oparg_T, optset_T, partial_S, partial_T, pos_T,
    pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue, reg_extmatch_T,
    regmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, save_v_event_T, schar_T, scid_T, sctx_T,
    size_t, switchwin_T, syn_state, syn_state_sst_union as C2Rust_Unnamed_3, syn_time_T,
    synblock_T, synstate_T, tabpage_S, tabpage_T, taggy_T, terminal, time_t, typval_T,
    typval_vval_union, u_entry, u_entry_T, u_header, u_header_T,
    u_header_uh_alt_next as C2Rust_Unnamed_7, u_header_uh_alt_prev as C2Rust_Unnamed_6,
    u_header_uh_next as C2Rust_Unnamed_9, u_header_uh_prev as C2Rust_Unnamed_8, ufunc_S, ufunc_T,
    uint16_t, uint32_t, uint64_t, uint8_t, undo_object, varnumber_T, vim_exception, virt_line,
    visualinfo_T, win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T, QUEUE,
};
use crate::src::nvim::ui::{
    ui_call_grid_destroy, ui_call_win_close, ui_call_win_external_pos, ui_call_win_float_pos,
    ui_call_win_hide, ui_call_win_pos, ui_call_win_viewport, ui_call_win_viewport_margins,
    ui_check_cursor_grid, ui_has,
};
use crate::src::nvim::ui_compositor::{
    ui_comp_layers_adjust, ui_comp_put_grid, ui_comp_remove_grid,
};
use crate::src::nvim::undo::{bufIsChanged, u_sync};
use crate::src::nvim::winfloat::{
    win_border_height, win_border_width, win_config_float, win_float_anchor_laststatus,
    win_float_find_altwin, win_float_update_statusline, win_new_float, win_reconfig_floats,
};
extern "C" {
    static aucmd_win_vec: GlobalCell<C2Rust_Unnamed_20>;
    fn hash_init(ht: *mut hashtab_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn qf_free_all(wp: *mut win_T);
    fn copy_loclist_stack(from: *mut win_T, to: *mut win_T);
    fn terminal_check_size(term: *mut Terminal);
}
pub const kErrorTypeValidation: ErrorType = 1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
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
pub const kStlClickFuncRun: C2Rust_Unnamed_11 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_11 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_11 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_11 = 0;
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
pub type C2Rust_Unnamed_12 = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed_12 = 2147483647;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_13 = 2147483647;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const kZIndexCmdlinePopupMenu: C2Rust_Unnamed_14 = 250;
pub const kZIndexMessages: C2Rust_Unnamed_14 = 200;
pub const kZIndexPopupMenu: C2Rust_Unnamed_14 = 100;
pub const kZIndexFloatDefault: C2Rust_Unnamed_14 = 50;
pub const kZIndexDefaultGrid: C2Rust_Unnamed_14 = 0;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const NUMBUFLEN: C2Rust_Unnamed_15 = 65;
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub const kCdScopeGlobal: CdScope = 2;
pub const kCdScopeTabpage: CdScope = 1;
pub const kCdScopeWindow: CdScope = 0;
pub const kCdScopeInvalid: CdScope = -1;
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
pub type C2Rust_Unnamed_16 = ::core::ffi::c_int;
pub const kWinOptWrap: C2Rust_Unnamed_16 = 50;
pub const kWinOptWinhighlight: C2Rust_Unnamed_16 = 49;
pub const kWinOptWinfixwidth: C2Rust_Unnamed_16 = 48;
pub const kWinOptWinfixheight: C2Rust_Unnamed_16 = 47;
pub const kWinOptWinfixbuf: C2Rust_Unnamed_16 = 46;
pub const kWinOptWinblend: C2Rust_Unnamed_16 = 45;
pub const kWinOptWinbar: C2Rust_Unnamed_16 = 44;
pub const kWinOptVirtualedit: C2Rust_Unnamed_16 = 43;
pub const kWinOptStatusline: C2Rust_Unnamed_16 = 42;
pub const kWinOptStatuscolumn: C2Rust_Unnamed_16 = 41;
pub const kWinOptSpell: C2Rust_Unnamed_16 = 40;
pub const kWinOptSmoothscroll: C2Rust_Unnamed_16 = 39;
pub const kWinOptSigncolumn: C2Rust_Unnamed_16 = 38;
pub const kWinOptSidescrolloff: C2Rust_Unnamed_16 = 37;
pub const kWinOptShowbreak: C2Rust_Unnamed_16 = 36;
pub const kWinOptScrolloff: C2Rust_Unnamed_16 = 35;
pub const kWinOptScrollbind: C2Rust_Unnamed_16 = 34;
pub const kWinOptScroll: C2Rust_Unnamed_16 = 33;
pub const kWinOptRightleftcmd: C2Rust_Unnamed_16 = 32;
pub const kWinOptRightleft: C2Rust_Unnamed_16 = 31;
pub const kWinOptRelativenumber: C2Rust_Unnamed_16 = 30;
pub const kWinOptPreviewwindow: C2Rust_Unnamed_16 = 29;
pub const kWinOptNumberwidth: C2Rust_Unnamed_16 = 28;
pub const kWinOptNumber: C2Rust_Unnamed_16 = 27;
pub const kWinOptListchars: C2Rust_Unnamed_16 = 26;
pub const kWinOptList: C2Rust_Unnamed_16 = 25;
pub const kWinOptLinebreak: C2Rust_Unnamed_16 = 24;
pub const kWinOptLhistory: C2Rust_Unnamed_16 = 23;
pub const kWinOptFoldtext: C2Rust_Unnamed_16 = 22;
pub const kWinOptFoldnestmax: C2Rust_Unnamed_16 = 21;
pub const kWinOptFoldminlines: C2Rust_Unnamed_16 = 20;
pub const kWinOptFoldmethod: C2Rust_Unnamed_16 = 19;
pub const kWinOptFoldmarker: C2Rust_Unnamed_16 = 18;
pub const kWinOptFoldlevel: C2Rust_Unnamed_16 = 17;
pub const kWinOptFoldignore: C2Rust_Unnamed_16 = 16;
pub const kWinOptFoldexpr: C2Rust_Unnamed_16 = 15;
pub const kWinOptFoldenable: C2Rust_Unnamed_16 = 14;
pub const kWinOptFoldcolumn: C2Rust_Unnamed_16 = 13;
pub const kWinOptFillchars: C2Rust_Unnamed_16 = 12;
pub const kWinOptEventignorewin: C2Rust_Unnamed_16 = 11;
pub const kWinOptDiff: C2Rust_Unnamed_16 = 10;
pub const kWinOptCursorlineopt: C2Rust_Unnamed_16 = 9;
pub const kWinOptCursorline: C2Rust_Unnamed_16 = 8;
pub const kWinOptCursorcolumn: C2Rust_Unnamed_16 = 7;
pub const kWinOptCursorbind: C2Rust_Unnamed_16 = 6;
pub const kWinOptConceallevel: C2Rust_Unnamed_16 = 5;
pub const kWinOptConcealcursor: C2Rust_Unnamed_16 = 4;
pub const kWinOptColorcolumn: C2Rust_Unnamed_16 = 3;
pub const kWinOptBreakindentopt: C2Rust_Unnamed_16 = 2;
pub const kWinOptBreakindent: C2Rust_Unnamed_16 = 1;
pub const kWinOptArabic: C2Rust_Unnamed_16 = 0;
pub const kWinOptInvalid: C2Rust_Unnamed_16 = -1;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNumber: OptValType = 1;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kOptValTypeNil: OptValType = -1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const kFloatAnchorSouth: C2Rust_Unnamed_17 = 2;
pub const kFloatAnchorEast: C2Rust_Unnamed_17 = 1;
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
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_19 = 8192;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_19 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_19 = 2048;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_19 = 1024;
pub const CMOD_KEEPMARKS: C2Rust_Unnamed_19 = 512;
pub const CMOD_KEEPALT: C2Rust_Unnamed_19 = 256;
pub const CMOD_CONFIRM: C2Rust_Unnamed_19 = 128;
pub const CMOD_BROWSE: C2Rust_Unnamed_19 = 64;
pub const CMOD_HIDE: C2Rust_Unnamed_19 = 32;
pub const CMOD_NOAUTOCMD: C2Rust_Unnamed_19 = 16;
pub const CMOD_UNSILENT: C2Rust_Unnamed_19 = 8;
pub const CMOD_ERRSILENT: C2Rust_Unnamed_19 = 4;
pub const CMOD_SILENT: C2Rust_Unnamed_19 = 2;
pub const CMOD_SANDBOX: C2Rust_Unnamed_19 = 1;
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
pub struct C2Rust_Unnamed_20 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut aucmdwin_T,
}
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
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const kOptSwbFlagUselast: C2Rust_Unnamed_21 = 32;
pub const kOptSwbFlagVsplit: C2Rust_Unnamed_21 = 16;
pub const kOptSwbFlagNewtab: C2Rust_Unnamed_21 = 8;
pub const kOptSwbFlagSplit: C2Rust_Unnamed_21 = 4;
pub const kOptSwbFlagUsetab: C2Rust_Unnamed_21 = 2;
pub const kOptSwbFlagUseopen: C2Rust_Unnamed_21 = 1;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const kOptTclFlagUselast: C2Rust_Unnamed_22 = 2;
pub const kOptTclFlagLeft: C2Rust_Unnamed_22 = 1;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_23 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_23 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_23 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_23 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_23 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_23 = 20;
pub const UPD_VALID: C2Rust_Unnamed_23 = 10;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_24 = 4;
pub const BL_SOL: C2Rust_Unnamed_24 = 2;
pub const BL_WHITE: C2Rust_Unnamed_24 = 1;
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
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const ECMD_NOWINENTER: C2Rust_Unnamed_25 = 64;
pub const ECMD_ALTBUF: C2Rust_Unnamed_25 = 32;
pub const ECMD_ADDBUF: C2Rust_Unnamed_25 = 16;
pub const ECMD_FORCEIT: C2Rust_Unnamed_25 = 8;
pub const ECMD_OLDBUF: C2Rust_Unnamed_25 = 4;
pub const ECMD_SET_HELP: C2Rust_Unnamed_25 = 2;
pub const ECMD_HIDE: C2Rust_Unnamed_25 = 1;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_int;
pub const ECMD_ONE: C2Rust_Unnamed_26 = 1;
pub const ECMD_LAST: C2Rust_Unnamed_26 = -1;
pub const ECMD_LASTL: C2Rust_Unnamed_26 = 0;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_27 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_27 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_27 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_27 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_27 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_27 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_27 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_27 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_27 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_27 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_27 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_27 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_27 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_27 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_27 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_27 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_27 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_27 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_27 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_27 = 1;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const FIND_EVAL: C2Rust_Unnamed_28 = 4;
pub const FIND_STRING: C2Rust_Unnamed_28 = 2;
pub const FIND_IDENT: C2Rust_Unnamed_28 = 1;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const BCO_NOHELP: C2Rust_Unnamed_29 = 4;
pub const BCO_ALWAYS: C2Rust_Unnamed_29 = 2;
pub const BCO_ENTER: C2Rust_Unnamed_29 = 1;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_uint;
pub const CHECK_PATH: C2Rust_Unnamed_30 = 3;
pub const FIND_DEFINE: C2Rust_Unnamed_30 = 2;
pub const FIND_ANY: C2Rust_Unnamed_30 = 1;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const ACTION_EXPAND: C2Rust_Unnamed_31 = 5;
pub const ACTION_SHOW_ALL: C2Rust_Unnamed_31 = 4;
pub const ACTION_SPLIT: C2Rust_Unnamed_31 = 3;
pub const ACTION_GOTO: C2Rust_Unnamed_31 = 2;
pub const ACTION_SHOW: C2Rust_Unnamed_31 = 1;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub const WSP_QUICKFIX: C2Rust_Unnamed_32 = 1024;
pub const WSP_NOENTER: C2Rust_Unnamed_32 = 512;
pub const WSP_NEWLOC: C2Rust_Unnamed_32 = 256;
pub const WSP_ABOVE: C2Rust_Unnamed_32 = 128;
pub const WSP_BELOW: C2Rust_Unnamed_32 = 64;
pub const WSP_HELP: C2Rust_Unnamed_32 = 32;
pub const WSP_BOT: C2Rust_Unnamed_32 = 16;
pub const WSP_TOP: C2Rust_Unnamed_32 = 8;
pub const WSP_HOR: C2Rust_Unnamed_32 = 4;
pub const WSP_VERT: C2Rust_Unnamed_32 = 2;
pub const WSP_ROOM: C2Rust_Unnamed_32 = 1;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub const STATUS_HEIGHT: C2Rust_Unnamed_33 = 1;
pub const MIN_LINES: C2Rust_Unnamed_33 = 2;
pub const MIN_COLUMNS: C2Rust_Unnamed_33 = 12;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub const LOWEST_WIN_ID: C2Rust_Unnamed_34 = 1000;
pub const WEE_TRIGGER_LEAVE_AUTOCMDS: C2Rust_Unnamed_35 = 16;
pub const WEE_TRIGGER_ENTER_AUTOCMDS: C2Rust_Unnamed_35 = 8;
pub const WEE_UNDO_SYNC: C2Rust_Unnamed_35 = 1;
pub const WEE_TRIGGER_NEW_AUTOCMDS: C2Rust_Unnamed_35 = 4;
pub const WEE_CURWIN_INVALID: C2Rust_Unnamed_35 = 2;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const VALID_WCOL: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VALID_CROW: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const VALID_TOPLINE: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const SNAP_HELP_IDX: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SNAP_QUICKFIX_IDX: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SNAP_COUNT: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const FR_LEAF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FR_ROW: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FR_COL: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn equalpos(mut a: pos_T, mut b: pos_T) -> bool {
    return a.lnum == b.lnum && a.col == b.col && a.coladd == b.coladd;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_uint32_t = Set_uint32_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<uint32_t>(),
};
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
pub const TAB: ::core::ffi::c_int = 9;
pub const CAR: ::core::ffi::c_int = 13;
pub const Ctrl_B: ::core::ffi::c_int = 2;
pub const Ctrl_C: ::core::ffi::c_int = 3;
pub const Ctrl_D: ::core::ffi::c_int = 4;
pub const Ctrl_F: ::core::ffi::c_int = 6;
pub const Ctrl_G: ::core::ffi::c_int = 7;
pub const Ctrl_H: ::core::ffi::c_int = 8;
pub const Ctrl_I: ::core::ffi::c_int = 9;
pub const Ctrl_J: ::core::ffi::c_int = 10;
pub const Ctrl_K: ::core::ffi::c_int = 11;
pub const Ctrl_L: ::core::ffi::c_int = 12;
pub const Ctrl_N: ::core::ffi::c_int = 14;
pub const Ctrl_O: ::core::ffi::c_int = 15;
pub const Ctrl_P: ::core::ffi::c_int = 16;
pub const Ctrl_Q: ::core::ffi::c_int = 17;
pub const Ctrl_R: ::core::ffi::c_int = 18;
pub const Ctrl_S: ::core::ffi::c_int = 19;
pub const Ctrl_T: ::core::ffi::c_int = 20;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const Ctrl_W: ::core::ffi::c_int = 23;
pub const Ctrl_X: ::core::ffi::c_int = 24;
pub const Ctrl_Z: ::core::ffi::c_int = 26;
pub const Ctrl_RSB: ::core::ffi::c_int = 29;
pub const Ctrl_HAT: ::core::ffi::c_int = 30;
pub const Ctrl__: ::core::ffi::c_int = 31;
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const SID_WINLAYOUT: ::core::ffi::c_int = -7 as ::core::ffi::c_int;
pub const K_UP: ::core::ffi::c_int = -30059;
pub const K_DOWN: ::core::ffi::c_int = -25707;
pub const K_LEFT: ::core::ffi::c_int = -27755;
pub const K_RIGHT: ::core::ffi::c_int = -29291;
pub const K_BS: ::core::ffi::c_int = -25195;
pub const K_KENTER: ::core::ffi::c_int = -16715;
pub const NOWIN: *mut win_T = -1 as ::core::ffi::c_int as *mut win_T;
static e_cannot_close_last_window: GlobalCell<[::core::ffi::c_char; 31]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
            *b"E444: Cannot close last window\0",
        )
    });
static e_cannot_split_window_when_closing_buffer: GlobalCell<[::core::ffi::c_char; 53]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 53], [::core::ffi::c_char; 53]>(
            *b"E1159: Cannot split a window when closing the buffer\0",
        )
    });
static m_onlyone: GlobalCell<*mut ::core::ffi::c_char> = GlobalCell::new(
    b"Already only one window\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
);
static split_disallowed: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static close_disallowed: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static frame_locked: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub unsafe extern "C" fn window_layout_lock() {
    (*split_disallowed.ptr()) += 1;
    (*close_disallowed.ptr()) += 1;
}
pub unsafe extern "C" fn window_layout_unlock() {
    (*split_disallowed.ptr()) -= 1;
    (*close_disallowed.ptr()) -= 1;
}
pub unsafe extern "C" fn frames_locked() -> bool {
    return frame_locked.get() != 0;
}
pub unsafe extern "C" fn window_layout_locked(mut cmd: cmdidx_T) -> bool {
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let locked: bool = window_layout_locked_err(cmd, &raw mut err);
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        emsg(gettext(err.msg));
        api_clear_error(&raw mut err);
    }
    return locked;
}
pub unsafe extern "C" fn window_layout_locked_err(mut cmd: cmdidx_T, mut err: *mut Error) -> bool {
    if split_disallowed.get() > 0 as ::core::ffi::c_int
        || close_disallowed.get() > 0 as ::core::ffi::c_int
    {
        if close_disallowed.get() == 0 as ::core::ffi::c_int
            && cmd as ::core::ffi::c_int == CMD_tabnew as ::core::ffi::c_int
        {
            api_set_error(
                err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                (e_cannot_split_window_when_closing_buffer.ptr() as *const _)
                    as *const ::core::ffi::c_char,
            );
        } else {
            api_set_error(
                err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_not_allowed_to_change_window_layout_in_this_autocmd
                    as *const ::core::ffi::c_char,
            );
        }
        return true_0 != 0;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn check_can_set_curbuf_disabled() -> bool {
    if (*curwin.get()).w_onebuf_opt.wo_wfb != 0 {
        emsg(gettext(
            &raw const e_winfixbuf_cannot_go_to_buffer as *const ::core::ffi::c_char,
        ));
        return false_0 != 0;
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn check_can_set_curbuf_forceit(mut forceit: ::core::ffi::c_int) -> bool {
    if forceit == 0 && (*curwin.get()).w_onebuf_opt.wo_wfb != 0 {
        emsg(gettext(
            &raw const e_winfixbuf_cannot_go_to_buffer as *const ::core::ffi::c_char,
        ));
        return false_0 != 0;
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn prevwin_curwin() -> *mut win_T {
    return if is_in_cmdwin() as ::core::ffi::c_int != 0 && !(*prevwin.ptr()).is_null() {
        prevwin.get()
    } else {
        curwin.get()
    };
}
pub unsafe extern "C" fn swbuf_goto_win_with_buf(mut buf: *mut buf_T) -> *mut win_T {
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    if buf.is_null() {
        return wp;
    }
    if swb_flags.get() & kOptSwbFlagUseopen as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        wp = buf_jump_open_win(buf);
    }
    if wp.is_null()
        && swb_flags.get() & kOptSwbFlagUsetab as ::core::ffi::c_int as ::core::ffi::c_uint != 0
    {
        wp = buf_jump_open_tab(buf);
    }
    return wp;
}
static min_set_ch: GlobalCell<OptInt> = GlobalCell::new(1 as OptInt);
pub unsafe extern "C" fn do_window(
    mut nchar: ::core::ffi::c_int,
    mut Prenum: ::core::ffi::c_int,
    mut xchar: ::core::ffi::c_int,
) {
    let mut config: WinConfig = WinConfig {
        window: 0,
        bufpos: lpos_T { lnum: 0, col: 0 },
        height: 0,
        width: 0,
        row: 0.,
        col: 0.,
        anchor: 0,
        relative: kFloatRelativeEditor,
        external: false,
        focusable: false,
        mouse: false,
        split: kWinSplitLeft,
        zindex: 0,
        style: kWinStyleUnused,
        border: false,
        shadow: false,
        border_chars: [[0; 32]; 8],
        border_hl_ids: [0; 8],
        border_attr: [0; 8],
        title: false,
        title_pos: kAlignLeft,
        title_chunks: VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        },
        title_width: 0,
        footer: false,
        footer_pos: kAlignLeft,
        footer_chunks: VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        },
        footer_width: 0,
        noautocmd: false,
        fixed: false,
        hide: false,
        _cmdline_offset: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeException,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut type_0: ::core::ffi::c_int = FIND_DEFINE as ::core::ffi::c_int;
    let mut cbuf: [::core::ffi::c_char; 40] = [0; 40];
    let mut Prenum1: ::core::ffi::c_int = if Prenum == 0 as ::core::ffi::c_int {
        1 as ::core::ffi::c_int
    } else {
        Prenum
    };
    's_1675: {
        '_newwindow: {
            '_wingotofile: {
                'c_63358: {
                    match nchar {
                        83 | Ctrl_S | 115 => {
                            if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                                return;
                            }
                            reset_VIsual_and_resel();
                            if bt_quickfix(curbuf.get()) {
                                break '_newwindow;
                            } else {
                                win_split(Prenum, 0 as ::core::ffi::c_int);
                                break 's_1675;
                            }
                        }
                        Ctrl_V | 118 => {
                            if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                                return;
                            }
                            reset_VIsual_and_resel();
                            if bt_quickfix(curbuf.get()) {
                                break '_newwindow;
                            } else {
                                win_split(Prenum, WSP_VERT as ::core::ffi::c_int);
                                break 's_1675;
                            }
                        }
                        Ctrl_HAT | 94 => {
                            if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                                return;
                            }
                            reset_VIsual_and_resel();
                            if buflist_findnr(if Prenum == 0 as ::core::ffi::c_int {
                                (*curwin.get()).w_alt_fnum
                            } else {
                                Prenum
                            })
                            .is_null()
                            {
                                if Prenum == 0 as ::core::ffi::c_int {
                                    emsg(gettext(&raw const e_noalt as *const ::core::ffi::c_char));
                                } else {
                                    semsg(
                                        gettext(
                                            &raw const e_buffer_nr_not_found
                                                as *const ::core::ffi::c_char,
                                        ),
                                        Prenum as int64_t,
                                    );
                                }
                                break 's_1675;
                            } else {
                                if !curbuf_locked()
                                    && win_split(0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int)
                                        == OK
                                {
                                    buflist_getfile(
                                        if Prenum == 0 as ::core::ffi::c_int {
                                            (*curwin.get()).w_alt_fnum
                                        } else {
                                            Prenum
                                        },
                                        0 as linenr_T,
                                        GETF_ALT as ::core::ffi::c_int,
                                        false_0,
                                    );
                                }
                                break 's_1675;
                            }
                        }
                        Ctrl_N | 110 => {
                            if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                                return;
                            }
                            reset_VIsual_and_resel();
                            break '_newwindow;
                        }
                        Ctrl_Q | 113 => {
                            reset_VIsual_and_resel();
                            cmd_with_count(
                                b"quit\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                &raw mut cbuf as *mut ::core::ffi::c_char,
                                ::core::mem::size_of::<[::core::ffi::c_char; 40]>(),
                                Prenum as int64_t,
                            );
                            do_cmdline_cmd(&raw mut cbuf as *mut ::core::ffi::c_char);
                            break 's_1675;
                        }
                        Ctrl_C | 99 => {
                            reset_VIsual_and_resel();
                            cmd_with_count(
                                b"close\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                &raw mut cbuf as *mut ::core::ffi::c_char,
                                ::core::mem::size_of::<[::core::ffi::c_char; 40]>(),
                                Prenum as int64_t,
                            );
                            do_cmdline_cmd(&raw mut cbuf as *mut ::core::ffi::c_char);
                            break 's_1675;
                        }
                        Ctrl_Z | 122 => {
                            if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                                return;
                            }
                            reset_VIsual_and_resel();
                            do_cmdline_cmd(b"pclose\0".as_ptr() as *const ::core::ffi::c_char);
                            break 's_1675;
                        }
                        80 => {
                            let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
                            let mut wp2: *mut win_T = if curtab.get() == curtab.get() {
                                firstwin.get()
                            } else {
                                (*curtab.get()).tp_firstwin
                            };
                            while !wp2.is_null() {
                                if (*wp2).w_onebuf_opt.wo_pvw != 0 {
                                    wp = wp2;
                                    break;
                                } else {
                                    wp2 = (*wp2).w_next;
                                }
                            }
                            if wp.is_null() {
                                emsg(gettext(b"E441: There is no preview window\0".as_ptr()
                                    as *const ::core::ffi::c_char));
                            } else {
                                win_goto(wp);
                            }
                            break 's_1675;
                        }
                        Ctrl_O | 111 => {
                            if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                                return;
                            }
                            reset_VIsual_and_resel();
                            cmd_with_count(
                                b"only\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                &raw mut cbuf as *mut ::core::ffi::c_char,
                                ::core::mem::size_of::<[::core::ffi::c_char; 40]>(),
                                Prenum as int64_t,
                            );
                            do_cmdline_cmd(&raw mut cbuf as *mut ::core::ffi::c_char);
                            break 's_1675;
                        }
                        Ctrl_W | 119 | 87 => {
                            if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                                return;
                            }
                            if firstwin.get() == lastwin.get() && Prenum != 1 as ::core::ffi::c_int
                            {
                                beep_flush();
                            } else {
                                let mut wp_0: *mut win_T = ::core::ptr::null_mut::<win_T>();
                                if Prenum != 0 {
                                    let mut last_focusable: *mut win_T = firstwin.get();
                                    wp_0 = firstwin.get();
                                    loop {
                                        Prenum -= 1;
                                        if Prenum <= 0 as ::core::ffi::c_int {
                                            break;
                                        }
                                        if !(*wp_0).w_floating
                                            || !(*wp_0).w_config.hide
                                                && (*wp_0).w_config.focusable as ::core::ffi::c_int
                                                    != 0
                                        {
                                            last_focusable = wp_0;
                                        }
                                        if (*wp_0).w_next.is_null() {
                                            break;
                                        }
                                        wp_0 = (*wp_0).w_next;
                                    }
                                    while !wp_0.is_null()
                                        && (*wp_0).w_floating as ::core::ffi::c_int != 0
                                        && ((*wp_0).w_config.hide as ::core::ffi::c_int != 0
                                            || !(*wp_0).w_config.focusable)
                                    {
                                        wp_0 = (*wp_0).w_next;
                                    }
                                    if wp_0.is_null() {
                                        wp_0 = last_focusable;
                                    }
                                } else if nchar == 'W' as ::core::ffi::c_int {
                                    wp_0 = (*curwin.get()).w_prev;
                                    if wp_0.is_null() {
                                        wp_0 = lastwin.get();
                                    }
                                    while !wp_0.is_null()
                                        && (*wp_0).w_floating as ::core::ffi::c_int != 0
                                        && ((*wp_0).w_config.hide as ::core::ffi::c_int != 0
                                            || !(*wp_0).w_config.focusable)
                                    {
                                        wp_0 = (*wp_0).w_prev;
                                    }
                                } else {
                                    wp_0 = (*curwin.get()).w_next;
                                    while !wp_0.is_null()
                                        && (*wp_0).w_floating as ::core::ffi::c_int != 0
                                        && ((*wp_0).w_config.hide as ::core::ffi::c_int != 0
                                            || !(*wp_0).w_config.focusable)
                                    {
                                        wp_0 = (*wp_0).w_next;
                                    }
                                    if wp_0.is_null() {
                                        wp_0 = firstwin.get();
                                    }
                                }
                                win_goto(wp_0);
                            }
                            break 's_1675;
                        }
                        106 | K_DOWN | Ctrl_J => {
                            if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                                return;
                            }
                            win_goto_ver(false_0 != 0, Prenum1);
                            break 's_1675;
                        }
                        107 | K_UP | Ctrl_K => {
                            if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                                return;
                            }
                            win_goto_ver(true_0 != 0, Prenum1);
                            break 's_1675;
                        }
                        104 | K_LEFT | Ctrl_H | K_BS => {
                            if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                                return;
                            }
                            win_goto_hor(true_0 != 0, Prenum1);
                            break 's_1675;
                        }
                        108 | K_RIGHT | Ctrl_L => {
                            if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                                return;
                            }
                            win_goto_hor(false_0 != 0, Prenum1);
                            break 's_1675;
                        }
                        84 => {
                            if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                                return;
                            }
                            if one_window(curwin.get(), ::core::ptr::null_mut::<tabpage_T>()) {
                                msg(gettext(m_onlyone.get()), 0 as ::core::ffi::c_int);
                            } else {
                                let mut oldtab: *mut tabpage_T = curtab.get();
                                let mut wp_1: *mut win_T = curwin.get();
                                if !win_new_tabpage(
                                    Prenum,
                                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                    true_0 != 0,
                                    ::core::ptr::null_mut::<*mut win_T>(),
                                )
                                .is_null()
                                    && valid_tabpage(oldtab) as ::core::ffi::c_int != 0
                                {
                                    let mut newtab: *mut tabpage_T = curtab.get();
                                    goto_tabpage_tp(oldtab, true_0 != 0, true_0 != 0);
                                    if curwin.get() == wp_1 {
                                        win_close(curwin.get(), false_0 != 0, false_0 != 0);
                                    }
                                    if valid_tabpage(newtab) {
                                        goto_tabpage_tp(newtab, true_0 != 0, true_0 != 0);
                                        apply_autocmds(
                                            EVENT_TABNEWENTERED,
                                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                            false_0 != 0,
                                            curbuf.get(),
                                        );
                                    }
                                }
                            }
                            break 's_1675;
                        }
                        116 | Ctrl_T => {
                            win_goto(firstwin.get());
                            break 's_1675;
                        }
                        98 | Ctrl_B => {
                            win_goto(lastwin_nofloating(::core::ptr::null_mut::<tabpage_T>()));
                            break 's_1675;
                        }
                        112 | Ctrl_P => {
                            if !win_valid(prevwin.get())
                                || (*prevwin.get()).w_config.hide as ::core::ffi::c_int != 0
                                || !(*prevwin.get()).w_config.focusable
                            {
                                beep_flush();
                            } else {
                                win_goto(prevwin.get());
                            }
                            break 's_1675;
                        }
                        120 | Ctrl_X => {
                            if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                                return;
                            }
                            win_exchange(Prenum);
                            break 's_1675;
                        }
                        Ctrl_R | 114 => {
                            if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                                return;
                            }
                            reset_VIsual_and_resel();
                            win_rotate(false_0 != 0, Prenum1);
                            break 's_1675;
                        }
                        82 => {
                            if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                                return;
                            }
                            reset_VIsual_and_resel();
                            win_rotate(true_0 != 0, Prenum1);
                            break 's_1675;
                        }
                        75 | 74 | 72 | 76 => {
                            if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                                return;
                            }
                            if one_window(curwin.get(), ::core::ptr::null_mut::<tabpage_T>()) {
                                beep_flush();
                            } else {
                                let dir: ::core::ffi::c_int = (if nchar == 'H' as ::core::ffi::c_int
                                    || nchar == 'L' as ::core::ffi::c_int
                                {
                                    WSP_VERT as ::core::ffi::c_int
                                } else {
                                    0 as ::core::ffi::c_int
                                }) | (if nchar
                                    == 'H' as ::core::ffi::c_int
                                    || nchar == 'K' as ::core::ffi::c_int
                                {
                                    WSP_TOP as ::core::ffi::c_int
                                } else {
                                    WSP_BOT as ::core::ffi::c_int
                                });
                                win_splitmove(curwin.get(), Prenum, dir);
                            }
                            break 's_1675;
                        }
                        61 => {
                            let mut mod_0: ::core::ffi::c_int = (*cmdmod.ptr()).cmod_split
                                & (WSP_VERT as ::core::ffi::c_int | WSP_HOR as ::core::ffi::c_int);
                            win_equal(
                                ::core::ptr::null_mut::<win_T>(),
                                false_0 != 0,
                                if mod_0 == WSP_VERT as ::core::ffi::c_int {
                                    'v' as ::core::ffi::c_int
                                } else if mod_0 == WSP_HOR as ::core::ffi::c_int {
                                    'h' as ::core::ffi::c_int
                                } else {
                                    'b' as ::core::ffi::c_int
                                },
                            );
                            break 's_1675;
                        }
                        43 => {
                            win_setheight((*curwin.get()).w_height + Prenum1);
                            break 's_1675;
                        }
                        45 => {
                            win_setheight((*curwin.get()).w_height - Prenum1);
                            break 's_1675;
                        }
                        Ctrl__ | 95 => {
                            win_setheight(if Prenum != 0 {
                                Prenum
                            } else {
                                Rows.get() - min_set_ch.get() as ::core::ffi::c_int
                            });
                            break 's_1675;
                        }
                        62 => {
                            win_setwidth((*curwin.get()).w_width + Prenum1);
                            break 's_1675;
                        }
                        60 => {
                            win_setwidth((*curwin.get()).w_width - Prenum1);
                            break 's_1675;
                        }
                        124 => {
                            win_setwidth(if Prenum != 0 as ::core::ffi::c_int {
                                Prenum
                            } else {
                                Columns.get()
                            });
                            break 's_1675;
                        }
                        125 => {
                            if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                                return;
                            }
                            if Prenum != 0 {
                                g_do_tagpreview.set(Prenum);
                            } else {
                                g_do_tagpreview.set(p_pvh.get() as ::core::ffi::c_int);
                            }
                            break 'c_63358;
                        }
                        93 | Ctrl_RSB => {
                            break 'c_63358;
                        }
                        102 | 70 | Ctrl_F => {
                            break '_wingotofile;
                        }
                        105 | Ctrl_I => {
                            type_0 = FIND_ANY as ::core::ffi::c_int;
                        }
                        100 | Ctrl_D => {}
                        K_KENTER | CAR => {
                            if bt_quickfix(curbuf.get()) {
                                qf_view_result(true_0 != 0);
                            }
                            break 's_1675;
                        }
                        103 | Ctrl_G => {
                            if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                                emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                                return;
                            }
                            (*no_mapping.ptr()) += 1;
                            (*allow_keys.ptr()) += 1;
                            if xchar == NUL {
                                xchar = plain_vgetc();
                            }
                            if *p_langmap.get() as ::core::ffi::c_int != 0
                                && true
                                && (p_lrm.get() != 0
                                    || (if vgetc_busy.get() != 0 {
                                        (typebuf_maplen() == 0 as ::core::ffi::c_int)
                                            as ::core::ffi::c_int
                                    } else {
                                        KeyTyped.get() as ::core::ffi::c_int
                                    }) != 0)
                                && KeyStuffed.get() == 0
                                && xchar >= 0 as ::core::ffi::c_int
                            {
                                if xchar < 256 as ::core::ffi::c_int {
                                    xchar = (*langmap_mapchar.ptr())[xchar as usize]
                                        as ::core::ffi::c_int;
                                } else {
                                    xchar = langmap_adjust_mb(xchar);
                                }
                            }
                            (*no_mapping.ptr()) -= 1;
                            (*allow_keys.ptr()) -= 1;
                            add_to_showcmd(xchar);
                            match xchar {
                                125 => {
                                    xchar = Ctrl_RSB;
                                    if Prenum != 0 {
                                        g_do_tagpreview.set(Prenum);
                                    } else {
                                        g_do_tagpreview.set(p_pvh.get() as ::core::ffi::c_int);
                                    }
                                }
                                93 | Ctrl_RSB => {}
                                102 | 70 => {
                                    (*cmdmod.ptr()).cmod_tab =
                                        tabpage_index(curtab.get()) + 1 as ::core::ffi::c_int;
                                    nchar = xchar;
                                    break '_wingotofile;
                                }
                                116 => {
                                    goto_tabpage(Prenum);
                                    break 's_1675;
                                }
                                84 => {
                                    goto_tabpage(-Prenum1);
                                    break 's_1675;
                                }
                                TAB => {
                                    if !goto_tabpage_lastused() {
                                        beep_flush();
                                    }
                                    break 's_1675;
                                }
                                101 => {
                                    if (*curwin.get()).w_floating as ::core::ffi::c_int != 0
                                        || !ui_has(kUIMultigrid)
                                    {
                                        beep_flush();
                                        break 's_1675;
                                    } else {
                                        config = WinConfig {
                                            window: 0,
                                            bufpos: lpos_T {
                                                lnum: -1 as linenr_T,
                                                col: 0 as colnr_T,
                                            },
                                            height: 0 as ::core::ffi::c_int,
                                            width: 0 as ::core::ffi::c_int,
                                            row: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
                                            col: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
                                            anchor: 0 as FloatAnchor,
                                            relative: kFloatRelativeEditor,
                                            external: false_0 != 0,
                                            focusable: true_0 != 0,
                                            mouse: true_0 != 0,
                                            split: kWinSplitLeft,
                                            zindex: kZIndexFloatDefault as ::core::ffi::c_int,
                                            style: kWinStyleUnused,
                                            border: false,
                                            shadow: false,
                                            border_chars: [[0; 32]; 8],
                                            border_hl_ids: [0; 8],
                                            border_attr: [0; 8],
                                            title: false,
                                            title_pos: kAlignLeft,
                                            title_chunks: VirtText {
                                                size: 0,
                                                capacity: 0,
                                                items: ::core::ptr::null_mut::<VirtTextChunk>(),
                                            },
                                            title_width: 0,
                                            footer: false,
                                            footer_pos: kAlignLeft,
                                            footer_chunks: VirtText {
                                                size: 0,
                                                capacity: 0,
                                                items: ::core::ptr::null_mut::<VirtTextChunk>(),
                                            },
                                            footer_width: 0,
                                            noautocmd: false_0 != 0,
                                            fixed: false_0 != 0,
                                            hide: false_0 != 0,
                                            _cmdline_offset: INT_MAX,
                                        };
                                        config.width = (*curwin.get()).w_width;
                                        config.height = (*curwin.get()).w_height;
                                        config.external = true_0 != 0;
                                        err = Error {
                                            type_0: kErrorTypeNone,
                                            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                        };
                                        if win_new_float(
                                            curwin.get(),
                                            false_0 != 0,
                                            config,
                                            &raw mut err,
                                        )
                                        .is_null()
                                        {
                                            emsg(err.msg);
                                            api_clear_error(&raw mut err);
                                            beep_flush();
                                        }
                                        break 's_1675;
                                    }
                                }
                                _ => {
                                    beep_flush();
                                    break 's_1675;
                                }
                            }
                            if Prenum != 0 {
                                postponed_split.set(Prenum);
                            } else {
                                postponed_split.set(-1 as ::core::ffi::c_int);
                            }
                            do_nv_ident('g' as ::core::ffi::c_int, xchar);
                            postponed_split.set(0 as ::core::ffi::c_int);
                            break 's_1675;
                        }
                        _ => {
                            beep_flush();
                            break 's_1675;
                        }
                    }
                    if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                        emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                        return;
                    }
                    let mut len: size_t = 0;
                    let mut ptr_0: *mut ::core::ffi::c_char =
                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                    len = find_ident_under_cursor(
                        &raw mut ptr_0,
                        FIND_IDENT as ::core::ffi::c_int,
                        ::core::ptr::null_mut::<::core::ffi::c_int>(),
                    );
                    if len == 0 as size_t {
                        break 's_1675;
                    } else {
                        ptr_0 = xmemdupz(ptr_0 as *const ::core::ffi::c_void, len)
                            as *mut ::core::ffi::c_char;
                        find_pattern_in_path(
                            ptr_0,
                            kDirectionNotSet,
                            len,
                            true_0 != 0,
                            Prenum == 0 as ::core::ffi::c_int,
                            type_0,
                            Prenum1,
                            ACTION_SPLIT as ::core::ffi::c_int,
                            1 as linenr_T,
                            MAXLNUM as ::core::ffi::c_int as linenr_T,
                            false_0 != 0,
                            false_0 != 0,
                        );
                        xfree(ptr_0 as *mut ::core::ffi::c_void);
                        (*curwin.get()).w_set_curswant = true_0;
                        break 's_1675;
                    }
                }
                if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                    emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                    return;
                }
                if Prenum != 0 {
                    postponed_split.set(Prenum);
                } else {
                    postponed_split.set(-1 as ::core::ffi::c_int);
                }
                if nchar != '}' as ::core::ffi::c_int {
                    g_do_tagpreview.set(0 as ::core::ffi::c_int);
                }
                do_nv_ident(Ctrl_RSB, NUL);
                postponed_split.set(0 as ::core::ffi::c_int);
                break 's_1675;
            }
            if cmdwin_type.get() != 0 as ::core::ffi::c_int {
                emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
                return;
            }
            if check_text_or_curbuf_locked(::core::ptr::null_mut::<oparg_T>()) {
                break 's_1675;
            } else {
                let mut lnum: linenr_T = -1 as linenr_T;
                let mut ptr: *mut ::core::ffi::c_char = grab_file_name(Prenum1, &raw mut lnum);
                if !ptr.is_null() {
                    let mut oldtab_0: *mut tabpage_T = curtab.get();
                    let mut oldwin: *mut win_T = curwin.get();
                    setpcmark();
                    let mut wp_2: *mut win_T = ::core::ptr::null_mut::<win_T>();
                    if swb_flags.get()
                        & (kOptSwbFlagUseopen as ::core::ffi::c_int
                            | kOptSwbFlagUsetab as ::core::ffi::c_int)
                            as ::core::ffi::c_uint
                        != 0
                        && (*cmdmod.ptr()).cmod_tab == 0 as ::core::ffi::c_int
                    {
                        wp_2 = swbuf_goto_win_with_buf(buflist_findname_exp(ptr));
                    }
                    if wp_2.is_null()
                        && win_split(0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) == OK
                    {
                        (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
                        (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
                        if do_ecmd(
                            0 as ::core::ffi::c_int,
                            ptr,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            ::core::ptr::null_mut::<exarg_T>(),
                            ECMD_LASTL as ::core::ffi::c_int as linenr_T,
                            ECMD_HIDE as ::core::ffi::c_int,
                            ::core::ptr::null_mut::<win_T>(),
                        ) == FAIL
                        {
                            win_close(curwin.get(), false_0 != 0, false_0 != 0);
                            goto_tabpage_win(oldtab_0, oldwin);
                        } else {
                            wp_2 = curwin.get();
                        }
                    }
                    if !wp_2.is_null()
                        && nchar == 'F' as ::core::ffi::c_int
                        && lnum >= 0 as linenr_T
                    {
                        (*curwin.get()).w_cursor.lnum = lnum;
                        check_cursor_lnum(curwin.get());
                        beginline(BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
                    }
                    xfree(ptr as *mut ::core::ffi::c_void);
                }
                break 's_1675;
            }
        }
        if Prenum != 0 {
            vim_snprintf(
                &raw mut cbuf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 40]>().wrapping_sub(5 as size_t),
                b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
                Prenum as int64_t,
            );
        } else {
            cbuf[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        }
        if nchar == 'v' as ::core::ffi::c_int || nchar == Ctrl_V {
            xstrlcat(
                &raw mut cbuf as *mut ::core::ffi::c_char,
                b"v\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 40]>(),
            );
        }
        xstrlcat(
            &raw mut cbuf as *mut ::core::ffi::c_char,
            b"new\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 40]>(),
        );
        do_cmdline_cmd(&raw mut cbuf as *mut ::core::ffi::c_char);
    };
}
unsafe extern "C" fn cmd_with_count(
    mut cmd: *mut ::core::ffi::c_char,
    mut bufp: *mut ::core::ffi::c_char,
    mut bufsize: size_t,
    mut Prenum: int64_t,
) {
    let mut len: size_t = xstrlcpy(bufp, cmd, bufsize);
    if Prenum > 0 as int64_t && len < bufsize {
        vim_snprintf(
            bufp.offset(len as isize),
            bufsize.wrapping_sub(len),
            b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
            Prenum,
        );
    }
}
pub unsafe extern "C" fn win_set_buf(
    mut win: *mut win_T,
    mut buf: *mut buf_T,
    mut err: *mut Error,
) {
    let win_handle: handle_T = (*win).handle;
    let mut tab: *mut tabpage_T = win_find_tabpage(win);
    (*RedrawingDisabled.ptr()) += 1;
    let mut switchwin: switchwin_T = switchwin_T {
        sw_curwin: ::core::ptr::null_mut::<win_T>(),
        sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
        sw_same_win: false,
        sw_visual_active: false,
    };
    let mut win_result: ::core::ffi::c_int = 0;
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    win_result = switch_win_noblock(&raw mut switchwin, win, tab, true);
    if win_result != 0 as ::core::ffi::c_int {
        let save_acd: ::core::ffi::c_int = p_acd.get();
        if !switchwin.sw_same_win {
            p_acd.set(0 as ::core::ffi::c_int);
        }
        do_buffer(
            DOBUF_GOTO as ::core::ffi::c_int,
            DOBUF_FIRST as ::core::ffi::c_int,
            FORWARD as ::core::ffi::c_int,
            (*buf).handle as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        if !switchwin.sw_same_win {
            p_acd.set(save_acd);
        }
    }
    try_leave(&raw mut tstate, err);
    if win_result == FAIL
        && !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int)
    {
        api_set_error(
            err,
            kErrorTypeException,
            b"Failed to switch to window %d\0".as_ptr() as *const ::core::ffi::c_char,
            win_handle,
        );
    }
    validate_cursor(curwin.get());
    restore_win_noblock(&raw mut switchwin, true_0 != 0);
    (*RedrawingDisabled.ptr()) -= 1;
}
pub unsafe extern "C" fn win_fdccol_count(mut wp: *mut win_T) -> ::core::ffi::c_int {
    let mut fdc: *const ::core::ffi::c_char = (*wp).w_onebuf_opt.wo_fdc;
    if strncmp(
        fdc,
        b"auto\0".as_ptr() as *const ::core::ffi::c_char,
        4 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        let fdccol: ::core::ffi::c_int = if *fdc.offset(4 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == ':' as ::core::ffi::c_int
        {
            *fdc.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                - '0' as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
        let mut needed_fdccols: ::core::ffi::c_int = getDeepestNesting(wp);
        return if fdccol < needed_fdccols {
            fdccol
        } else {
            needed_fdccols
        };
    }
    return *fdc.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        - '0' as ::core::ffi::c_int;
}
pub unsafe extern "C" fn merge_win_config(mut dst: *mut WinConfig, src: WinConfig) {
    if (*dst).title_chunks.items != src.title_chunks.items {
        clear_virttext(&raw mut (*dst).title_chunks);
    }
    if (*dst).footer_chunks.items != src.footer_chunks.items {
        clear_virttext(&raw mut (*dst).footer_chunks);
    }
    *dst = src;
}
pub unsafe extern "C" fn clear_float_config(mut fconfig: *mut WinConfig, mut free_fields: bool) {
    let mut saved_style: WinStyle = (*fconfig).style;
    let mut saved_cmdline_offset: ::core::ffi::c_int = (*fconfig)._cmdline_offset;
    if free_fields {
        merge_win_config(
            fconfig,
            WinConfig {
                window: 0,
                bufpos: lpos_T {
                    lnum: -1 as linenr_T,
                    col: 0 as colnr_T,
                },
                height: 0 as ::core::ffi::c_int,
                width: 0 as ::core::ffi::c_int,
                row: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
                col: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
                anchor: 0 as FloatAnchor,
                relative: kFloatRelativeEditor,
                external: false_0 != 0,
                focusable: true_0 != 0,
                mouse: true_0 != 0,
                split: kWinSplitLeft,
                zindex: kZIndexFloatDefault as ::core::ffi::c_int,
                style: kWinStyleUnused,
                border: false,
                shadow: false,
                border_chars: [[0; 32]; 8],
                border_hl_ids: [0; 8],
                border_attr: [0; 8],
                title: false,
                title_pos: kAlignLeft,
                title_chunks: VirtText {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<VirtTextChunk>(),
                },
                title_width: 0,
                footer: false,
                footer_pos: kAlignLeft,
                footer_chunks: VirtText {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<VirtTextChunk>(),
                },
                footer_width: 0,
                noautocmd: false_0 != 0,
                fixed: false_0 != 0,
                hide: false_0 != 0,
                _cmdline_offset: INT_MAX,
            },
        );
    } else {
        *fconfig = WinConfig {
            window: 0,
            bufpos: lpos_T {
                lnum: -1 as linenr_T,
                col: 0 as colnr_T,
            },
            height: 0 as ::core::ffi::c_int,
            width: 0 as ::core::ffi::c_int,
            row: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
            col: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
            anchor: 0 as FloatAnchor,
            relative: kFloatRelativeEditor,
            external: false_0 != 0,
            focusable: true_0 != 0,
            mouse: true_0 != 0,
            split: kWinSplitLeft,
            zindex: kZIndexFloatDefault as ::core::ffi::c_int,
            style: kWinStyleUnused,
            border: false,
            shadow: false,
            border_chars: [[0; 32]; 8],
            border_hl_ids: [0; 8],
            border_attr: [0; 8],
            title: false,
            title_pos: kAlignLeft,
            title_chunks: VirtText {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<VirtTextChunk>(),
            },
            title_width: 0,
            footer: false,
            footer_pos: kAlignLeft,
            footer_chunks: VirtText {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<VirtTextChunk>(),
            },
            footer_width: 0,
            noautocmd: false_0 != 0,
            fixed: false_0 != 0,
            hide: false_0 != 0,
            _cmdline_offset: INT_MAX,
        };
    }
    (*fconfig).style = saved_style;
    (*fconfig)._cmdline_offset = saved_cmdline_offset;
}
pub unsafe extern "C" fn ui_ext_win_position(mut wp: *mut win_T, mut validate: bool) {
    (*wp).w_pos_changed = false_0 != 0;
    if !(*wp).w_floating {
        if ui_has(kUIMultigrid) {
            (*wp).w_grid_alloc.comp_col = (*wp).w_wincol;
            (*wp).w_grid_alloc.comp_row = (*wp).w_winrow;
        }
        ui_call_win_pos(
            (*wp).w_grid_alloc.handle as Integer,
            (*wp).handle as Window,
            (*wp).w_winrow as Integer,
            (*wp).w_wincol as Integer,
            (*wp).w_width as Integer,
            (*wp).w_height as Integer,
        );
        return;
    }
    let mut c: WinConfig = (*wp).w_config;
    if !c.external {
        let mut grid: *mut ScreenGrid = default_grid.ptr();
        let mut row: Float = c.row as Float;
        let mut col: Float = c.col as Float;
        if c.relative as ::core::ffi::c_uint
            == kFloatRelativeWindow as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut dummy: Error = Error {
                type_0: kErrorTypeNone,
                msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            };
            let mut win: *mut win_T = find_window_by_handle(c.window, &raw mut dummy);
            api_clear_error(&raw mut dummy);
            if !win.is_null() {
                if (*win).w_pos_changed as ::core::ffi::c_int != 0
                    && !(*win).w_grid_alloc.chars.is_null()
                    && win_valid(win) as ::core::ffi::c_int != 0
                {
                    ui_ext_win_position(win, validate);
                }
                let mut row_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut col_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                win_grid_alloc(win);
                grid = grid_adjust(&raw mut (*win).w_grid, &raw mut row_off, &raw mut col_off);
                row += row_off as Float;
                col += col_off as Float;
                if c.bufpos.lnum >= 0 as linenr_T {
                    let mut lnum: ::core::ffi::c_int = if (c.bufpos.lnum + 1 as linenr_T)
                        < (*(*win).w_buffer).b_ml.ml_line_count
                    {
                        c.bufpos.lnum as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                    } else {
                        (*(*win).w_buffer).b_ml.ml_line_count as ::core::ffi::c_int
                    };
                    let mut pos: pos_T = pos_T {
                        lnum: lnum as linenr_T,
                        col: c.bufpos.col,
                        coladd: 0 as colnr_T,
                    };
                    let mut trow: ::core::ffi::c_int = 0;
                    let mut tcol: ::core::ffi::c_int = 0;
                    let mut tcolc: ::core::ffi::c_int = 0;
                    let mut tcole: ::core::ffi::c_int = 0;
                    textpos2screenpos(
                        win,
                        &raw mut pos,
                        &raw mut trow,
                        &raw mut tcol,
                        &raw mut tcolc,
                        &raw mut tcole,
                        true_0 != 0,
                    );
                    row += (trow - 1 as ::core::ffi::c_int) as Float;
                    col += (tcol - 1 as ::core::ffi::c_int) as Float;
                }
            }
        } else if c.relative as ::core::ffi::c_uint
            == kFloatRelativeLaststatus as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            row += (Rows.get() - p_ch.get() as ::core::ffi::c_int - last_stl_height(false_0 != 0))
                as Float;
        } else if c.relative as ::core::ffi::c_uint
            == kFloatRelativeTabline as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            row += tabline_height() as Float;
        }
        let mut resort: bool = (*wp).w_grid_alloc.comp_index != 0 as size_t
            && (*wp).w_grid_alloc.zindex != (*wp).w_config.zindex;
        let mut raise: bool =
            resort as ::core::ffi::c_int != 0 && (*wp).w_grid_alloc.zindex < (*wp).w_config.zindex;
        (*wp).w_grid_alloc.zindex = (*wp).w_config.zindex;
        if resort {
            ui_comp_layers_adjust((*wp).w_grid_alloc.comp_index, raise);
        }
        let mut valid: bool = (*wp).w_redr_type == 0 as ::core::ffi::c_int
            || ui_has(kUIMultigrid) as ::core::ffi::c_int != 0;
        if !valid && !validate {
            (*wp).w_pos_changed = true_0 != 0;
            return;
        }
        let mut east: bool =
            c.anchor as ::core::ffi::c_int & kFloatAnchorEast as ::core::ffi::c_int != 0;
        let mut south: bool =
            c.anchor as ::core::ffi::c_int & kFloatAnchorSouth as ::core::ffi::c_int != 0;
        let mut comp_row: ::core::ffi::c_int = row as ::core::ffi::c_int
            - (if south as ::core::ffi::c_int != 0 {
                (*wp).w_height_outer
            } else {
                0 as ::core::ffi::c_int
            });
        let mut comp_col_0: ::core::ffi::c_int = col as ::core::ffi::c_int
            - (if east as ::core::ffi::c_int != 0 {
                (*wp).w_width_outer
            } else {
                0 as ::core::ffi::c_int
            });
        let mut above_ch: ::core::ffi::c_int =
            if (*wp).w_config.zindex < kZIndexMessages as ::core::ffi::c_int {
                p_ch.get() as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            };
        comp_row += (*grid).comp_row;
        comp_col_0 += (*grid).comp_col;
        comp_row = if (if comp_row < Rows.get() - (*wp).w_height_outer - above_ch {
            comp_row
        } else {
            Rows.get() - (*wp).w_height_outer - above_ch
        }) > 0 as ::core::ffi::c_int
        {
            if comp_row < Rows.get() - (*wp).w_height_outer - above_ch {
                comp_row
            } else {
                Rows.get() - (*wp).w_height_outer - above_ch
            }
        } else {
            0 as ::core::ffi::c_int
        };
        if !c.fixed || east as ::core::ffi::c_int != 0 {
            comp_col_0 = if (if comp_col_0 < Columns.get() - (*wp).w_width_outer {
                comp_col_0
            } else {
                Columns.get() - (*wp).w_width_outer
            }) > 0 as ::core::ffi::c_int
            {
                if comp_col_0 < Columns.get() - (*wp).w_width_outer {
                    comp_col_0
                } else {
                    Columns.get() - (*wp).w_width_outer
                }
            } else {
                0 as ::core::ffi::c_int
            };
        }
        (*wp).w_winrow = comp_row;
        (*wp).w_wincol = comp_col_0;
        if !c.hide {
            ui_comp_put_grid(
                &raw mut (*wp).w_grid_alloc,
                comp_row,
                comp_col_0,
                (*wp).w_height_outer,
                (*wp).w_width_outer,
                valid,
                false_0 != 0,
            );
            if ui_has(kUIMultigrid) {
                let mut anchor: String_0 = cstr_as_string(
                    *(&raw const float_anchor_str as *const *const ::core::ffi::c_char)
                        .offset(c.anchor as isize),
                );
                ui_call_win_float_pos(
                    (*wp).w_grid_alloc.handle as Integer,
                    (*wp).handle as Window,
                    anchor,
                    (*grid).handle as Integer,
                    row,
                    col,
                    c.mouse as Boolean,
                    (*wp).w_grid_alloc.zindex as Integer,
                    (*wp).w_grid_alloc.comp_index as ::core::ffi::c_int as Integer,
                    (*wp).w_winrow as Integer,
                    (*wp).w_wincol as Integer,
                );
            }
            ui_check_cursor_grid((*wp).w_grid_alloc.handle);
            (*wp).w_grid_alloc.mouse_enabled = (*wp).w_config.mouse;
            if !valid {
                (*wp).w_grid_alloc.valid = false_0 != 0;
                redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
            }
        } else {
            if ui_has(kUIMultigrid) {
                ui_call_win_hide((*wp).w_grid_alloc.handle as Integer);
            }
            ui_comp_remove_grid(&raw mut (*wp).w_grid_alloc);
        }
    } else {
        ui_call_win_external_pos((*wp).w_grid_alloc.handle as Integer, (*wp).handle as Window);
    };
}
pub unsafe extern "C" fn ui_ext_win_viewport(mut wp: *mut win_T) {
    if (wp == curwin.get() || ui_has(kUIMultigrid) as ::core::ffi::c_int != 0)
        && (*wp).w_viewport_invalid as ::core::ffi::c_int != 0
        && (*wp).w_redr_type == 0 as ::core::ffi::c_int
    {
        let line_count: linenr_T = (*(*wp).w_buffer).b_ml.ml_line_count;
        let cur_topline: linenr_T = if (*wp).w_topline < line_count {
            (*wp).w_topline
        } else {
            line_count
        };
        let cur_botline: linenr_T = if (*wp).w_botline < line_count {
            (*wp).w_botline
        } else {
            line_count
        };
        let mut delta: int64_t = 0 as int64_t;
        let mut last_topline: linenr_T = (*wp).w_viewport_last_topline;
        let mut last_botline: linenr_T = (*wp).w_viewport_last_botline;
        let mut last_topfill: ::core::ffi::c_int =
            (*wp).w_viewport_last_topfill as ::core::ffi::c_int;
        let mut last_skipcol: int64_t = (*wp).w_viewport_last_skipcol as int64_t;
        if last_topline > line_count {
            delta -= (last_topline - line_count) as int64_t;
            last_topline = line_count;
            last_topfill = 0 as ::core::ffi::c_int;
            last_skipcol = MAXCOL as ::core::ffi::c_int as int64_t;
        }
        last_botline = if last_botline < line_count {
            last_botline
        } else {
            line_count
        };
        if cur_topline < last_topline
            || cur_topline == last_topline && ((*wp).w_skipcol as int64_t) < last_skipcol
        {
            let mut vcole: int64_t = last_skipcol;
            let mut lnume: linenr_T = last_topline;
            if last_topline > 0 as linenr_T && cur_botline < last_topline {
                delta -= (last_topline - cur_botline) as int64_t;
                lnume = cur_botline;
                vcole = 0 as int64_t;
            }
            delta -= win_text_height(
                wp,
                cur_topline,
                (*wp).w_skipcol as int64_t,
                &raw mut lnume,
                &raw mut vcole,
                ::core::ptr::null_mut::<int64_t>(),
                INT64_MAX as int64_t,
            );
        } else if cur_topline > last_topline
            || cur_topline == last_topline && (*wp).w_skipcol as int64_t > last_skipcol
        {
            let mut vcole_0: int64_t = (*wp).w_skipcol as int64_t;
            let mut lnume_0: linenr_T = cur_topline;
            if last_botline > 0 as linenr_T && cur_topline > last_botline {
                delta += (cur_topline - last_botline) as int64_t;
                lnume_0 = last_botline;
                vcole_0 = 0 as int64_t;
            }
            delta += win_text_height(
                wp,
                last_topline,
                last_skipcol,
                &raw mut lnume_0,
                &raw mut vcole_0,
                ::core::ptr::null_mut::<int64_t>(),
                INT64_MAX as int64_t,
            );
        }
        delta += last_topfill as int64_t;
        delta -= (*wp).w_topfill as int64_t;
        let mut ev_botline: linenr_T = (*wp).w_botline;
        if ev_botline == line_count + 1 as linenr_T && (*wp).w_empty_rows == 0 as ::core::ffi::c_int
        {
            ev_botline = line_count;
        }
        ui_call_win_viewport(
            (*wp).w_grid_alloc.handle as Integer,
            (*wp).handle as Window,
            ((*wp).w_topline - 1 as linenr_T) as Integer,
            ev_botline as Integer,
            ((*wp).w_cursor.lnum - 1 as linenr_T) as Integer,
            (*wp).w_cursor.col as Integer,
            line_count as Integer,
            delta as Integer,
        );
        (*wp).w_viewport_invalid = false_0 != 0;
        (*wp).w_viewport_last_topline = (*wp).w_topline;
        (*wp).w_viewport_last_botline = (*wp).w_botline;
        (*wp).w_viewport_last_topfill = (*wp).w_topfill as linenr_T;
        (*wp).w_viewport_last_skipcol = (*wp).w_skipcol as linenr_T;
    }
}
pub unsafe extern "C" fn check_split_disallowed(mut wp: *const win_T) -> ::core::ffi::c_int {
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let ok: bool = check_split_disallowed_err(wp, &raw mut err);
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        emsg(gettext(err.msg));
        api_clear_error(&raw mut err);
    }
    return if ok as ::core::ffi::c_int != 0 {
        OK
    } else {
        FAIL
    };
}
pub unsafe extern "C" fn check_split_disallowed_err(
    mut wp: *const win_T,
    mut err: *mut Error,
) -> bool {
    if split_disallowed.get() > 0 as ::core::ffi::c_int {
        api_set_error(
            err,
            kErrorTypeException,
            b"E242: Can't split a window while closing another\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
        return false_0 != 0;
    }
    if (*(*wp).w_buffer).b_locked_split != 0 {
        api_set_error(
            err,
            kErrorTypeException,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            (e_cannot_split_window_when_closing_buffer.ptr() as *const _)
                as *const ::core::ffi::c_char,
        );
        return false_0 != 0;
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn win_split(
    mut size: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if check_split_disallowed(curwin.get()) == FAIL {
        return FAIL;
    }
    if may_open_tabpage() == OK {
        return OK;
    }
    flags |= (*cmdmod.ptr()).cmod_split;
    if flags & WSP_TOP as ::core::ffi::c_int != 0 && flags & WSP_BOT as ::core::ffi::c_int != 0 {
        emsg(gettext(
            b"E442: Can't split topleft and botright at the same time\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    if flags & WSP_HELP as ::core::ffi::c_int != 0 {
        make_snapshot(SNAP_HELP_IDX);
    } else {
        clear_snapshot(curtab.get(), SNAP_HELP_IDX);
    }
    if flags & WSP_QUICKFIX as ::core::ffi::c_int != 0 {
        make_snapshot(SNAP_QUICKFIX_IDX);
    } else {
        clear_snapshot(curtab.get(), SNAP_QUICKFIX_IDX);
    }
    return if win_split_ins(
        size,
        flags,
        ::core::ptr::null_mut::<win_T>(),
        0 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<frame_T>(),
    )
    .is_null()
    {
        FAIL
    } else {
        OK
    };
}
pub unsafe extern "C" fn win_split_ins(
    mut size: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut new_wp: *mut win_T,
    mut dir: ::core::ffi::c_int,
    mut to_flatten: *mut frame_T,
) -> *mut win_T {
    let mut wp: *mut win_T = new_wp;
    if !new_wp.is_null() && is_aucmd_win(new_wp) as ::core::ffi::c_int != 0 {
        return ::core::ptr::null_mut::<win_T>();
    }
    if new_wp.is_null() {
        trigger_winnewpre();
    }
    let mut oldwin: *mut win_T = ::core::ptr::null_mut::<win_T>();
    if flags & WSP_TOP as ::core::ffi::c_int != 0 {
        oldwin = firstwin.get();
    } else if flags & WSP_BOT as ::core::ffi::c_int != 0
        || (*curwin.get()).w_floating as ::core::ffi::c_int != 0
    {
        oldwin = lastwin_nofloating(::core::ptr::null_mut::<tabpage_T>());
    } else {
        oldwin = curwin.get();
    }
    let mut need_status: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut new_size: ::core::ffi::c_int = size;
    let mut vertical: bool = flags & WSP_VERT as ::core::ffi::c_int != 0;
    let mut toplevel: bool =
        flags & (WSP_TOP as ::core::ffi::c_int | WSP_BOT as ::core::ffi::c_int) != 0;
    if one_window(firstwin.get(), ::core::ptr::null_mut::<tabpage_T>()) as ::core::ffi::c_int != 0
        && p_ls.get() == 1 as OptInt
        && (*oldwin).w_status_height == 0 as ::core::ffi::c_int
    {
        if (*oldwin).w_height as OptInt <= p_wmh.get() {
            emsg(gettext(&raw const e_noroom as *const ::core::ffi::c_char));
            return ::core::ptr::null_mut::<win_T>();
        }
        need_status = STATUS_HEIGHT as ::core::ffi::c_int;
        win_float_anchor_laststatus();
    }
    let mut do_equal: bool = false_0 != 0;
    let mut oldwin_height: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let layout: ::core::ffi::c_int = if vertical as ::core::ffi::c_int != 0 {
        FR_ROW
    } else {
        FR_COL
    };
    let mut did_set_fraction: bool = false_0 != 0;
    if vertical {
        let mut wmw1: ::core::ffi::c_int = if p_wmw.get() == 0 as OptInt {
            1 as ::core::ffi::c_int
        } else {
            p_wmw.get() as ::core::ffi::c_int
        };
        let mut needed: ::core::ffi::c_int = wmw1 + 1 as ::core::ffi::c_int;
        if flags & WSP_ROOM as ::core::ffi::c_int != 0 {
            needed += p_wiw.get() as ::core::ffi::c_int - wmw1;
        }
        let mut minwidth: ::core::ffi::c_int = 0;
        let mut available: ::core::ffi::c_int = 0;
        if toplevel {
            minwidth = frame_minwidth(topframe.get(), NOWIN);
            available = (*topframe.get()).fr_width;
            needed += minwidth;
        } else if p_ea.get() != 0 {
            minwidth = frame_minwidth((*oldwin).w_frame, NOWIN);
            let mut prevfrp: *mut frame_T = (*oldwin).w_frame;
            let mut frp: *mut frame_T = (*(*oldwin).w_frame).fr_parent;
            while !frp.is_null() {
                if (*frp).fr_layout as ::core::ffi::c_int == FR_ROW {
                    let mut frp2: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
                    frp2 = (*frp).fr_child;
                    while !frp2.is_null() {
                        if frp2 != prevfrp {
                            minwidth += frame_minwidth(frp2, NOWIN);
                        }
                        frp2 = (*frp2).fr_next;
                    }
                }
                prevfrp = frp;
                frp = (*frp).fr_parent;
            }
            available = (*topframe.get()).fr_width;
            needed += minwidth;
        } else {
            minwidth = frame_minwidth((*oldwin).w_frame, NOWIN);
            available = (*(*oldwin).w_frame).fr_width;
            needed += minwidth;
        }
        if available < needed {
            emsg(gettext(&raw const e_noroom as *const ::core::ffi::c_char));
            return ::core::ptr::null_mut::<win_T>();
        }
        if new_size == 0 as ::core::ffi::c_int {
            new_size = (*oldwin).w_width / 2 as ::core::ffi::c_int;
        }
        new_size = if (if new_size < available - minwidth - 1 as ::core::ffi::c_int {
            new_size
        } else {
            available - minwidth - 1 as ::core::ffi::c_int
        }) > wmw1
        {
            if new_size < available - minwidth - 1 as ::core::ffi::c_int {
                new_size
            } else {
                available - minwidth - 1 as ::core::ffi::c_int
            }
        } else {
            wmw1
        };
        if (((*oldwin).w_width - new_size - 1 as ::core::ffi::c_int) as OptInt) < p_wmw.get() {
            do_equal = true_0 != 0;
        }
        if (*oldwin).w_onebuf_opt.wo_wfw != 0 {
            win_setwidth_win(
                (*oldwin).w_width + new_size + 1 as ::core::ffi::c_int,
                oldwin,
            );
        }
        if !do_equal
            && p_ea.get() != 0
            && size == 0 as ::core::ffi::c_int
            && *p_ead.get() as ::core::ffi::c_int != 'v' as ::core::ffi::c_int
            && !(*(*oldwin).w_frame).fr_parent.is_null()
        {
            let mut frp_0: *mut frame_T = (*(*(*oldwin).w_frame).fr_parent).fr_child;
            while !frp_0.is_null() {
                if (*frp_0).fr_win != oldwin
                    && !(*frp_0).fr_win.is_null()
                    && ((*(*frp_0).fr_win).w_width > new_size
                        || (*(*frp_0).fr_win).w_width
                            > (*oldwin).w_width - new_size - 1 as ::core::ffi::c_int)
                {
                    do_equal = true_0 != 0;
                    break;
                } else {
                    frp_0 = (*frp_0).fr_next;
                }
            }
        }
    } else {
        let mut wmh1: ::core::ffi::c_int =
            (if p_wmh.get() as ::core::ffi::c_int > 1 as ::core::ffi::c_int {
                p_wmh.get() as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            }) + (*oldwin).w_winbar_height;
        let mut needed_0: ::core::ffi::c_int = wmh1 + STATUS_HEIGHT as ::core::ffi::c_int;
        if flags & WSP_ROOM as ::core::ffi::c_int != 0 {
            needed_0 += p_wh.get() as ::core::ffi::c_int - wmh1 + (*oldwin).w_winbar_height;
        }
        if p_ch.get() < 1 as OptInt {
            needed_0 += 1 as ::core::ffi::c_int;
        }
        let mut minheight: ::core::ffi::c_int = 0;
        let mut available_0: ::core::ffi::c_int = 0;
        if toplevel {
            minheight = frame_minheight(topframe.get(), NOWIN) + need_status;
            available_0 = (*topframe.get()).fr_height;
            needed_0 += minheight;
        } else if p_ea.get() != 0 {
            minheight = frame_minheight((*oldwin).w_frame, NOWIN) + need_status;
            let mut prevfrp_0: *mut frame_T = (*oldwin).w_frame;
            let mut frp_1: *mut frame_T = (*(*oldwin).w_frame).fr_parent;
            while !frp_1.is_null() {
                if (*frp_1).fr_layout as ::core::ffi::c_int == FR_COL {
                    let mut frp2_0: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
                    frp2_0 = (*frp_1).fr_child;
                    while !frp2_0.is_null() {
                        if frp2_0 != prevfrp_0 {
                            minheight += frame_minheight(frp2_0, NOWIN);
                        }
                        frp2_0 = (*frp2_0).fr_next;
                    }
                }
                prevfrp_0 = frp_1;
                frp_1 = (*frp_1).fr_parent;
            }
            available_0 = (*topframe.get()).fr_height;
            needed_0 += minheight;
        } else {
            minheight = frame_minheight((*oldwin).w_frame, NOWIN) + need_status;
            available_0 = (*(*oldwin).w_frame).fr_height;
            needed_0 += minheight;
        }
        if available_0 < needed_0 {
            emsg(gettext(&raw const e_noroom as *const ::core::ffi::c_char));
            return ::core::ptr::null_mut::<win_T>();
        }
        oldwin_height = (*oldwin).w_height;
        if need_status != 0 {
            (*oldwin).w_status_height = STATUS_HEIGHT as ::core::ffi::c_int;
            oldwin_height -= STATUS_HEIGHT as ::core::ffi::c_int;
        }
        if new_size == 0 as ::core::ffi::c_int {
            new_size = oldwin_height / 2 as ::core::ffi::c_int;
        }
        new_size = if (if new_size < available_0 - minheight - STATUS_HEIGHT as ::core::ffi::c_int {
            new_size
        } else {
            available_0 - minheight - STATUS_HEIGHT as ::core::ffi::c_int
        }) > wmh1
        {
            if new_size < available_0 - minheight - STATUS_HEIGHT as ::core::ffi::c_int {
                new_size
            } else {
                available_0 - minheight - STATUS_HEIGHT as ::core::ffi::c_int
            }
        } else {
            wmh1
        };
        if ((oldwin_height - new_size - STATUS_HEIGHT as ::core::ffi::c_int) as OptInt)
            < p_wmh.get()
        {
            do_equal = true_0 != 0;
        }
        if (*oldwin).w_onebuf_opt.wo_wfh != 0 {
            set_fraction(oldwin);
            did_set_fraction = true_0 != 0;
            win_setheight_win(
                (*oldwin).w_height + new_size + STATUS_HEIGHT as ::core::ffi::c_int,
                oldwin,
            );
            oldwin_height = (*oldwin).w_height;
            if need_status != 0 {
                oldwin_height -= STATUS_HEIGHT as ::core::ffi::c_int;
            }
        }
        if !do_equal
            && p_ea.get() != 0
            && size == 0 as ::core::ffi::c_int
            && *p_ead.get() as ::core::ffi::c_int != 'h' as ::core::ffi::c_int
            && !(*(*oldwin).w_frame).fr_parent.is_null()
        {
            let mut frp_2: *mut frame_T = (*(*(*oldwin).w_frame).fr_parent).fr_child;
            while !frp_2.is_null() {
                if (*frp_2).fr_win != oldwin
                    && !(*frp_2).fr_win.is_null()
                    && ((*(*frp_2).fr_win).w_height > new_size
                        || (*(*frp_2).fr_win).w_height
                            > oldwin_height - new_size - STATUS_HEIGHT as ::core::ffi::c_int)
                {
                    do_equal = true_0 != 0;
                    break;
                } else {
                    frp_2 = (*frp_2).fr_next;
                }
            }
        }
    }
    if flags & WSP_TOP as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        && (flags & WSP_BOT as ::core::ffi::c_int != 0
            || flags & WSP_BELOW as ::core::ffi::c_int != 0
            || flags & WSP_ABOVE as ::core::ffi::c_int == 0
                && (if vertical as ::core::ffi::c_int != 0 {
                    p_spr.get()
                } else {
                    p_sb.get()
                }) != 0)
    {
        if new_wp.is_null() {
            wp = win_alloc(oldwin, false_0 != 0);
        } else {
            win_append(oldwin, wp, ::core::ptr::null_mut::<tabpage_T>());
        }
    } else if new_wp.is_null() {
        wp = win_alloc((*oldwin).w_prev, false_0 != 0);
    } else {
        win_append((*oldwin).w_prev, wp, ::core::ptr::null_mut::<tabpage_T>());
    }
    if new_wp.is_null() {
        if wp.is_null() {
            return ::core::ptr::null_mut::<win_T>();
        }
        new_frame(wp);
        win_init(wp, curwin.get(), flags);
    } else if (*wp).w_floating {
        ui_comp_remove_grid(&raw mut (*wp).w_grid_alloc);
        if ui_has(kUIMultigrid) {
            (*wp).w_pos_changed = true_0 != 0;
        } else {
            ui_call_win_hide((*wp).w_grid_alloc.handle as Integer);
            win_free_grid(wp, true_0 != 0);
        }
        if (*wp).w_config.external {
            let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
            while !tp.is_null() {
                if tp != curtab.get() && (*tp).tp_curwin == wp {
                    (*tp).tp_curwin = (*tp).tp_firstwin;
                }
                tp = (*tp).tp_next as *mut tabpage_T;
            }
        }
        (*wp).w_floating = false_0 != 0;
        new_frame(wp);
        clear_float_config(&raw mut (*wp).w_config, true_0 != 0);
        memset(
            &raw mut (*wp).w_border_adj as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<[::core::ffi::c_int; 4]>(),
        );
    }
    if !to_flatten.is_null() {
        frame_flatten(to_flatten);
    }
    let mut before: bool = false;
    let mut curfrp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
    if toplevel {
        if (*topframe.get()).fr_layout as ::core::ffi::c_int == FR_COL && !vertical
            || (*topframe.get()).fr_layout as ::core::ffi::c_int == FR_ROW
                && vertical as ::core::ffi::c_int != 0
        {
            curfrp = (*topframe.get()).fr_child;
            if flags & WSP_BOT as ::core::ffi::c_int != 0 {
                while !(*curfrp).fr_next.is_null() {
                    curfrp = (*curfrp).fr_next;
                }
            }
        } else {
            curfrp = topframe.get();
        }
        before = flags & WSP_TOP as ::core::ffi::c_int != 0;
    } else {
        curfrp = (*oldwin).w_frame;
        if flags & WSP_BELOW as ::core::ffi::c_int != 0 {
            before = false_0 != 0;
        } else if flags & WSP_ABOVE as ::core::ffi::c_int != 0 {
            before = true_0 != 0;
        } else if vertical {
            before = p_spr.get() == 0;
        } else {
            before = p_sb.get() == 0;
        }
    }
    if (*curfrp).fr_parent.is_null()
        || (*(*curfrp).fr_parent).fr_layout as ::core::ffi::c_int != layout
    {
        let mut frp_3: *mut frame_T =
            xcalloc(1 as size_t, ::core::mem::size_of::<frame_T>()) as *mut frame_T;
        *frp_3 = *curfrp;
        (*curfrp).fr_layout = layout as ::core::ffi::c_char;
        (*frp_3).fr_parent = curfrp;
        (*frp_3).fr_next = ::core::ptr::null_mut::<frame_T>();
        (*frp_3).fr_prev = ::core::ptr::null_mut::<frame_T>();
        (*curfrp).fr_child = frp_3;
        (*curfrp).fr_win = ::core::ptr::null_mut::<win_T>();
        curfrp = frp_3;
        if !(*frp_3).fr_win.is_null() {
            (*oldwin).w_frame = frp_3;
        } else {
            frp_3 = (*frp_3).fr_child;
            while !frp_3.is_null() {
                (*frp_3).fr_parent = curfrp;
                frp_3 = (*frp_3).fr_next;
            }
        }
    }
    let mut frp_4: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
    if new_wp.is_null() {
        frp_4 = (*wp).w_frame;
    } else {
        frp_4 = (*new_wp).w_frame;
    }
    (*frp_4).fr_parent = (*curfrp).fr_parent;
    if before {
        frame_insert(curfrp, frp_4);
    } else {
        frame_append(curfrp, frp_4);
    }
    if !did_set_fraction {
        set_fraction(oldwin);
    }
    (*wp).w_fraction = (*oldwin).w_fraction;
    if vertical {
        (*wp).w_onebuf_opt.wo_scr = (*curwin.get()).w_onebuf_opt.wo_scr;
        if need_status != 0 {
            win_new_height(oldwin, (*oldwin).w_height - 1 as ::core::ffi::c_int);
            (*oldwin).w_status_height = need_status;
        }
        if toplevel {
            (*wp).w_winrow = tabline_height();
            win_new_height(
                wp,
                (*curfrp).fr_height
                    - (p_ls.get() == 1 as OptInt || p_ls.get() == 2 as OptInt)
                        as ::core::ffi::c_int,
            );
            (*wp).w_status_height =
                (p_ls.get() == 1 as OptInt || p_ls.get() == 2 as OptInt) as ::core::ffi::c_int;
            (*wp).w_hsep_height = 0 as ::core::ffi::c_int;
        } else {
            (*wp).w_winrow = (*oldwin).w_winrow;
            win_new_height(wp, (*oldwin).w_height);
            (*wp).w_status_height = (*oldwin).w_status_height;
            (*wp).w_hsep_height = (*oldwin).w_hsep_height;
        }
        (*frp_4).fr_height = (*curfrp).fr_height;
        win_new_width(wp, new_size);
        if before {
            (*wp).w_vsep_width = 1 as ::core::ffi::c_int;
        } else {
            (*wp).w_vsep_width = (*oldwin).w_vsep_width;
            (*oldwin).w_vsep_width = 1 as ::core::ffi::c_int;
        }
        if toplevel {
            if flags & WSP_BOT as ::core::ffi::c_int != 0 {
                frame_set_vsep(curfrp, true_0 != 0);
            }
            frame_new_width(
                curfrp,
                (*curfrp).fr_width
                    - (new_size
                        + (flags & WSP_TOP as ::core::ffi::c_int != 0 as ::core::ffi::c_int)
                            as ::core::ffi::c_int),
                flags & WSP_TOP as ::core::ffi::c_int != 0,
                false_0 != 0,
            );
        } else {
            win_new_width(
                oldwin,
                (*oldwin).w_width - (new_size + 1 as ::core::ffi::c_int),
            );
        }
        if before {
            (*wp).w_wincol = (*oldwin).w_wincol;
            (*oldwin).w_wincol += new_size + 1 as ::core::ffi::c_int;
        } else {
            (*wp).w_wincol = (*oldwin).w_wincol + (*oldwin).w_width + 1 as ::core::ffi::c_int;
        }
        frame_fix_width(oldwin);
        frame_fix_width(wp);
    } else {
        let is_stl_global: bool = global_stl_height() > 0 as ::core::ffi::c_int;
        if toplevel {
            (*wp).w_wincol = 0 as ::core::ffi::c_int;
            win_new_width(wp, Columns.get());
            (*wp).w_vsep_width = 0 as ::core::ffi::c_int;
        } else {
            (*wp).w_wincol = (*oldwin).w_wincol;
            win_new_width(wp, (*oldwin).w_width);
            (*wp).w_vsep_width = (*oldwin).w_vsep_width;
        }
        (*frp_4).fr_width = (*curfrp).fr_width;
        win_new_height(wp, new_size);
        let old_status_height: ::core::ffi::c_int = (*oldwin).w_status_height;
        if before {
            (*wp).w_hsep_height = if is_stl_global as ::core::ffi::c_int != 0 {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            };
        } else {
            (*wp).w_hsep_height = (*oldwin).w_hsep_height;
            (*oldwin).w_hsep_height = if is_stl_global as ::core::ffi::c_int != 0 {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            };
        }
        if toplevel {
            let mut new_fr_height: ::core::ffi::c_int = (*curfrp).fr_height - new_size;
            if is_stl_global {
                if flags & WSP_BOT as ::core::ffi::c_int != 0 {
                    frame_add_hsep(curfrp);
                } else {
                    new_fr_height -= 1 as ::core::ffi::c_int;
                }
            } else {
                if !(flags & WSP_BOT as ::core::ffi::c_int != 0 && p_ls.get() == 0 as OptInt) {
                    new_fr_height -= STATUS_HEIGHT as ::core::ffi::c_int;
                }
                if flags & WSP_BOT as ::core::ffi::c_int != 0 {
                    frame_add_statusline(curfrp);
                }
            }
            frame_new_height(
                curfrp,
                new_fr_height,
                flags & WSP_TOP as ::core::ffi::c_int != 0,
                false_0 != 0,
                false_0 != 0,
            );
        } else {
            win_new_height(
                oldwin,
                oldwin_height - (new_size + STATUS_HEIGHT as ::core::ffi::c_int),
            );
        }
        if before {
            (*wp).w_winrow = (*oldwin).w_winrow;
            if is_stl_global {
                (*wp).w_status_height = 0 as ::core::ffi::c_int;
                (*oldwin).w_winrow += (*wp).w_height + 1 as ::core::ffi::c_int;
            } else {
                (*wp).w_status_height = STATUS_HEIGHT as ::core::ffi::c_int;
                (*oldwin).w_winrow += (*wp).w_height + STATUS_HEIGHT as ::core::ffi::c_int;
            }
        } else if is_stl_global {
            (*wp).w_winrow = (*oldwin).w_winrow + (*oldwin).w_height + 1 as ::core::ffi::c_int;
            (*wp).w_status_height = 0 as ::core::ffi::c_int;
        } else {
            (*wp).w_winrow =
                (*oldwin).w_winrow + (*oldwin).w_height + STATUS_HEIGHT as ::core::ffi::c_int;
            (*wp).w_status_height = old_status_height;
            if flags & WSP_BOT as ::core::ffi::c_int == 0 {
                (*oldwin).w_status_height = STATUS_HEIGHT as ::core::ffi::c_int;
            }
        }
        frame_fix_height(wp);
        frame_fix_height(oldwin);
    }
    if toplevel {
        win_comp_pos();
    }
    redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
    redraw_later(oldwin, UPD_NOT_VALID as ::core::ffi::c_int);
    status_redraw_all();
    if need_status != 0 {
        msg_row.set(Rows.get() - 1 as ::core::ffi::c_int);
        msg_col.set(sc_col.get());
        msg_clr_eos_force();
        comp_col();
        msg_row.set(Rows.get() - 1 as ::core::ffi::c_int);
        msg_col.set(0 as ::core::ffi::c_int);
    }
    if do_equal as ::core::ffi::c_int != 0 || dir != 0 as ::core::ffi::c_int {
        win_equal(
            wp,
            true_0 != 0,
            if vertical as ::core::ffi::c_int != 0 {
                if dir == 'v' as ::core::ffi::c_int {
                    'b' as ::core::ffi::c_int
                } else {
                    'h' as ::core::ffi::c_int
                }
            } else if dir == 'h' as ::core::ffi::c_int {
                'b' as ::core::ffi::c_int
            } else {
                'v' as ::core::ffi::c_int
            },
        );
    } else if !is_aucmd_win(wp) {
        win_fix_scroll(false_0 != 0);
    }
    let mut i: ::core::ffi::c_int = 0;
    if flags & WSP_VERT as ::core::ffi::c_int != 0 {
        i = p_wiw.get() as ::core::ffi::c_int;
        if size != 0 as ::core::ffi::c_int {
            p_wiw.set(size as OptInt);
        }
    } else {
        i = p_wh.get() as ::core::ffi::c_int;
        if size != 0 as ::core::ffi::c_int {
            p_wh.set(size as OptInt);
        }
    }
    if flags & WSP_NOENTER as ::core::ffi::c_int == 0 {
        win_enter_ext(
            wp,
            (if new_wp.is_null() {
                WEE_TRIGGER_NEW_AUTOCMDS as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) | WEE_TRIGGER_ENTER_AUTOCMDS as ::core::ffi::c_int
                | WEE_TRIGGER_LEAVE_AUTOCMDS as ::core::ffi::c_int,
        );
    }
    if vertical {
        p_wiw.set(i as OptInt);
    } else {
        p_wh.set(i as OptInt);
    }
    if win_valid(oldwin) {
        (*oldwin).w_pos_changed = true_0 != 0;
    }
    return wp;
}
pub unsafe extern "C" fn win_init(
    mut newp: *mut win_T,
    mut oldp: *mut win_T,
    mut flags: ::core::ffi::c_int,
) {
    (*newp).w_buffer = (*oldp).w_buffer;
    (*newp).w_s = &raw mut (*(*oldp).w_buffer).b_s;
    (*(*oldp).w_buffer).b_nwindows += 1;
    (*newp).w_cursor = (*oldp).w_cursor;
    (*newp).w_valid = 0 as ::core::ffi::c_int;
    (*newp).w_curswant = (*oldp).w_curswant;
    (*newp).w_set_curswant = (*oldp).w_set_curswant;
    (*newp).w_topline = (*oldp).w_topline;
    (*newp).w_topfill = (*oldp).w_topfill;
    (*newp).w_leftcol = (*oldp).w_leftcol;
    (*newp).w_pcmark = (*oldp).w_pcmark;
    (*newp).w_prev_pcmark = (*oldp).w_prev_pcmark;
    (*newp).w_alt_fnum = (*oldp).w_alt_fnum;
    (*newp).w_wrow = (*oldp).w_wrow;
    (*newp).w_fraction = (*oldp).w_fraction;
    (*newp).w_prev_fraction_row = (*oldp).w_prev_fraction_row;
    copy_jumplist(oldp, newp);
    if flags & WSP_NEWLOC as ::core::ffi::c_int != 0 {
        (*newp).w_llist = ::core::ptr::null_mut::<qf_info_T>();
        (*newp).w_llist_ref = ::core::ptr::null_mut::<qf_info_T>();
    } else {
        copy_loclist_stack(oldp, newp);
    }
    (*newp).w_localdir = if (*oldp).w_localdir.is_null() {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    } else {
        xstrdup((*oldp).w_localdir)
    };
    (*newp).w_prevdir = if (*oldp).w_prevdir.is_null() {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    } else {
        xstrdup((*oldp).w_prevdir)
    };
    if *p_spk.get() as ::core::ffi::c_int != 'c' as ::core::ffi::c_int {
        if *p_spk.get() as ::core::ffi::c_int == 't' as ::core::ffi::c_int {
            (*newp).w_skipcol = (*oldp).w_skipcol;
        }
        (*newp).w_botline = (*oldp).w_botline;
        (*newp).w_prev_height = (*oldp).w_height;
        (*newp).w_prev_winrow = (*oldp).w_winrow;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*oldp).w_tagstacklen {
        let mut tag: *mut taggy_T =
            (&raw mut (*newp).w_tagstack as *mut taggy_T).offset(i as isize);
        *tag = (*oldp).w_tagstack[i as usize];
        if !(*tag).tagname.is_null() {
            (*tag).tagname = xstrdup((*tag).tagname);
        }
        if !(*tag).user_data.is_null() {
            (*tag).user_data = xstrdup((*tag).user_data);
        }
        i += 1;
    }
    (*newp).w_tagstackidx = (*oldp).w_tagstackidx;
    (*newp).w_tagstacklen = (*oldp).w_tagstacklen;
    (*newp).w_changelistidx = (*oldp).w_changelistidx;
    copyFoldingState(oldp, newp);
    win_init_some(newp, oldp);
    (*newp).w_winbar_height = (*oldp).w_winbar_height;
}
unsafe extern "C" fn win_init_some(mut newp: *mut win_T, mut oldp: *mut win_T) {
    (*newp).w_alist = (*oldp).w_alist;
    (*(*newp).w_alist).al_refcount += 1;
    (*newp).w_arg_idx = (*oldp).w_arg_idx;
    win_copy_options(oldp, newp);
}
#[no_mangle]
pub unsafe extern "C" fn win_valid(mut win: *const win_T) -> bool {
    return tabpage_win_valid(curtab.get(), win);
}
pub unsafe extern "C" fn tabpage_win_valid(
    mut tp: *const tabpage_T,
    mut win: *const win_T,
) -> bool {
    if win.is_null() {
        return false_0 != 0;
    }
    let mut wp: *mut win_T = if tp == curtab.get() as *const tabpage_T {
        firstwin.get()
    } else {
        (*tp).tp_firstwin
    };
    while !wp.is_null() {
        if wp == win as *mut win_T {
            return true_0 != 0;
        }
        wp = (*wp).w_next;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn win_find_by_handle(mut handle: handle_T) -> *mut win_T {
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).handle == handle {
            return wp;
        }
        wp = (*wp).w_next;
    }
    return ::core::ptr::null_mut::<win_T>();
}
pub unsafe extern "C" fn win_valid_any_tab(mut win: *mut win_T) -> bool {
    if win.is_null() {
        return false_0 != 0;
    }
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if wp == win {
                return true_0 != 0;
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn win_count() -> ::core::ffi::c_int {
    let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        count += 1;
        wp = (*wp).w_next;
    }
    return count;
}
pub unsafe extern "C" fn make_windows(
    mut count: ::core::ffi::c_int,
    mut vertical: bool,
) -> ::core::ffi::c_int {
    let mut maxcount: ::core::ffi::c_int = 0;
    if vertical {
        maxcount = (((*curwin.get()).w_width + (*curwin.get()).w_vsep_width) as OptInt
            - (p_wiw.get() - p_wmw.get())) as ::core::ffi::c_int
            / (p_wmw.get() as ::core::ffi::c_int + 1 as ::core::ffi::c_int);
    } else {
        maxcount = (((*curwin.get()).w_height
            + (*curwin.get()).w_hsep_height
            + (*curwin.get()).w_status_height) as OptInt
            - (p_wh.get() - p_wmh.get())) as ::core::ffi::c_int
            / (p_wmh.get() as ::core::ffi::c_int
                + STATUS_HEIGHT as ::core::ffi::c_int
                + global_winbar_height());
    }
    maxcount = if maxcount > 2 as ::core::ffi::c_int {
        maxcount
    } else {
        2 as ::core::ffi::c_int
    };
    count = if count < maxcount { count } else { maxcount };
    if count > 1 as ::core::ffi::c_int {
        last_status(true_0 != 0);
    }
    block_autocmds();
    let mut todo: ::core::ffi::c_int = 0;
    todo = count - 1 as ::core::ffi::c_int;
    while todo > 0 as ::core::ffi::c_int {
        if vertical {
            if win_split(
                (*curwin.get()).w_width
                    - ((*curwin.get()).w_width - todo) / (todo + 1 as ::core::ffi::c_int)
                    - 1 as ::core::ffi::c_int,
                WSP_VERT as ::core::ffi::c_int | WSP_ABOVE as ::core::ffi::c_int,
            ) == FAIL
            {
                break;
            }
        } else if win_split(
            (*curwin.get()).w_height
                - ((*curwin.get()).w_height - todo * STATUS_HEIGHT as ::core::ffi::c_int)
                    / (todo + 1 as ::core::ffi::c_int)
                - STATUS_HEIGHT as ::core::ffi::c_int,
            WSP_ABOVE as ::core::ffi::c_int,
        ) == FAIL
        {
            break;
        }
        todo -= 1;
    }
    unblock_autocmds();
    return count - todo;
}
unsafe extern "C" fn win_exchange(mut Prenum: ::core::ffi::c_int) {
    if (*curwin.get()).w_floating {
        emsg(&raw const e_floatexchange as *const ::core::ffi::c_char);
        return;
    }
    if one_window(curwin.get(), ::core::ptr::null_mut::<tabpage_T>()) {
        beep_flush();
        return;
    }
    if text_or_buf_locked() {
        beep_flush();
        return;
    }
    let mut frp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
    if Prenum != 0 {
        frp = (*(*(*curwin.get()).w_frame).fr_parent).fr_child;
        while !frp.is_null() && {
            Prenum -= 1;
            Prenum > 0 as ::core::ffi::c_int
        } {
            frp = (*frp).fr_next;
        }
    } else if !(*(*curwin.get()).w_frame).fr_next.is_null() {
        frp = (*(*curwin.get()).w_frame).fr_next;
    } else {
        frp = (*(*curwin.get()).w_frame).fr_prev;
    }
    if frp.is_null() || (*frp).fr_win.is_null() || (*frp).fr_win == curwin.get() {
        return;
    }
    let mut wp: *mut win_T = (*frp).fr_win;
    let mut wp2: *mut win_T = (*curwin.get()).w_prev;
    let mut frp2: *mut frame_T = (*(*curwin.get()).w_frame).fr_prev;
    if (*wp).w_prev != curwin.get() {
        win_remove(curwin.get(), ::core::ptr::null_mut::<tabpage_T>());
        frame_remove((*curwin.get()).w_frame);
        win_append(
            (*wp).w_prev,
            curwin.get(),
            ::core::ptr::null_mut::<tabpage_T>(),
        );
        frame_insert(frp, (*curwin.get()).w_frame);
    }
    if wp != wp2 {
        win_remove(wp, ::core::ptr::null_mut::<tabpage_T>());
        frame_remove((*wp).w_frame);
        win_append(wp2, wp, ::core::ptr::null_mut::<tabpage_T>());
        if frp2.is_null() {
            frame_insert((*(*(*wp).w_frame).fr_parent).fr_child, (*wp).w_frame);
        } else {
            frame_append(frp2, (*wp).w_frame);
        }
    }
    let mut temp: ::core::ffi::c_int = (*curwin.get()).w_status_height;
    (*curwin.get()).w_status_height = (*wp).w_status_height;
    (*wp).w_status_height = temp;
    temp = (*curwin.get()).w_vsep_width;
    (*curwin.get()).w_vsep_width = (*wp).w_vsep_width;
    (*wp).w_vsep_width = temp;
    temp = (*curwin.get()).w_hsep_height;
    (*curwin.get()).w_hsep_height = (*wp).w_hsep_height;
    (*wp).w_hsep_height = temp;
    frame_fix_height(curwin.get());
    frame_fix_height(wp);
    frame_fix_width(curwin.get());
    frame_fix_width(wp);
    win_comp_pos();
    if (*wp).w_buffer != curbuf.get() {
        reset_VIsual_and_resel();
    } else if VIsual_active.get() {
        (*wp).w_cursor = (*curwin.get()).w_cursor;
    }
    win_enter(wp, true_0 != 0);
    redraw_later(curwin.get(), UPD_NOT_VALID as ::core::ffi::c_int);
    redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
}
unsafe extern "C" fn win_rotate(mut upwards: bool, mut count: ::core::ffi::c_int) {
    if (*curwin.get()).w_floating {
        emsg(&raw const e_floatexchange as *const ::core::ffi::c_char);
        return;
    }
    if count <= 0 as ::core::ffi::c_int
        || one_window(curwin.get(), ::core::ptr::null_mut::<tabpage_T>()) as ::core::ffi::c_int != 0
    {
        beep_flush();
        return;
    }
    let mut frp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
    frp = (*(*(*curwin.get()).w_frame).fr_parent).fr_child;
    while !frp.is_null() {
        if (*frp).fr_win.is_null() {
            emsg(gettext(
                b"E443: Cannot rotate when another window is split\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return;
        }
        frp = (*frp).fr_next;
    }
    let mut wp1: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut wp2: *mut win_T = ::core::ptr::null_mut::<win_T>();
    loop {
        let c2rust_fresh0 = count;
        count = count - 1;
        if c2rust_fresh0 == 0 {
            break;
        }
        if upwards {
            frp = (*(*(*curwin.get()).w_frame).fr_parent).fr_child;
            '_c2rust_label: {
                if !frp.is_null() {
                } else {
                    __assert_fail(
                        b"frp != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2008 as ::core::ffi::c_uint,
                        b"void win_rotate(_Bool, int)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            wp1 = (*frp).fr_win;
            win_remove(wp1, ::core::ptr::null_mut::<tabpage_T>());
            frame_remove(frp);
            '_c2rust_label_0: {
                if !(*(*frp).fr_parent).fr_child.is_null() {
                } else {
                    __assert_fail(
                        b"frp->fr_parent->fr_child\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2012 as ::core::ffi::c_uint,
                        b"void win_rotate(_Bool, int)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            while !(*frp).fr_next.is_null() {
                frp = (*frp).fr_next;
            }
            win_append((*frp).fr_win, wp1, ::core::ptr::null_mut::<tabpage_T>());
            frame_append(frp, (*wp1).w_frame);
            wp2 = (*frp).fr_win;
        } else {
            frp = (*curwin.get()).w_frame;
            while !(*frp).fr_next.is_null() {
                frp = (*frp).fr_next;
            }
            wp1 = (*frp).fr_win;
            wp2 = (*wp1).w_prev;
            win_remove(wp1, ::core::ptr::null_mut::<tabpage_T>());
            frame_remove(frp);
            '_c2rust_label_1: {
                if !(*(*frp).fr_parent).fr_child.is_null() {
                } else {
                    __assert_fail(
                        b"frp->fr_parent->fr_child\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2028 as ::core::ffi::c_uint,
                        b"void win_rotate(_Bool, int)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            win_append(
                (*(*(*(*frp).fr_parent).fr_child).fr_win).w_prev,
                wp1,
                ::core::ptr::null_mut::<tabpage_T>(),
            );
            frame_insert((*(*frp).fr_parent).fr_child, frp);
        }
        let mut n: ::core::ffi::c_int = (*wp2).w_status_height;
        (*wp2).w_status_height = (*wp1).w_status_height;
        (*wp1).w_status_height = n;
        n = (*wp2).w_hsep_height;
        (*wp2).w_hsep_height = (*wp1).w_hsep_height;
        (*wp1).w_hsep_height = n;
        frame_fix_height(wp1);
        frame_fix_height(wp2);
        n = (*wp2).w_vsep_width;
        (*wp2).w_vsep_width = (*wp1).w_vsep_width;
        (*wp1).w_vsep_width = n;
        frame_fix_width(wp1);
        frame_fix_width(wp2);
        win_comp_pos();
    }
    (*wp1).w_pos_changed = true_0 != 0;
    (*wp2).w_pos_changed = true_0 != 0;
    redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
}
pub unsafe extern "C" fn win_splitmove(
    mut wp: *mut win_T,
    mut size: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut dir: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut height: ::core::ffi::c_int = (*wp).w_height;
    if one_window(wp, ::core::ptr::null_mut::<tabpage_T>()) {
        return OK;
    }
    if is_aucmd_win(wp) as ::core::ffi::c_int != 0 || check_split_disallowed(wp) == FAIL {
        return FAIL;
    }
    let mut unflat_altfr: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
    if (*wp).w_floating {
        win_remove(wp, ::core::ptr::null_mut::<tabpage_T>());
    } else {
        winframe_remove(
            wp,
            &raw mut dir,
            ::core::ptr::null_mut::<tabpage_T>(),
            &raw mut unflat_altfr,
        );
        '_c2rust_label: {
            if !unflat_altfr.is_null() {
            } else {
                __assert_fail(
                    b"unflat_altfr != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2083 as ::core::ffi::c_uint,
                    b"int win_splitmove(win_T *, int, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        win_remove(wp, ::core::ptr::null_mut::<tabpage_T>());
        last_status(false_0 != 0);
        win_comp_pos();
    }
    if win_split_ins(size, flags, wp, dir, unflat_altfr).is_null() {
        if !(*wp).w_floating {
            '_c2rust_label_0: {
                if !unflat_altfr.is_null() {
                } else {
                    __assert_fail(
                        b"unflat_altfr != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2092 as ::core::ffi::c_uint,
                        b"int win_splitmove(win_T *, int, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            winframe_restore(wp, dir, unflat_altfr);
        }
        win_append((*wp).w_prev, wp, ::core::ptr::null_mut::<tabpage_T>());
        return FAIL;
    }
    if size == 0 as ::core::ffi::c_int
        && flags & WSP_VERT as ::core::ffi::c_int == 0
        && win_valid(wp) as ::core::ffi::c_int != 0
        && !(*wp).w_floating
    {
        win_setheight_win(height, wp);
        if p_ea.get() != 0 {
            win_equal(curwin.get(), curwin.get() == wp, 'v' as ::core::ffi::c_int);
        }
    }
    return OK;
}
pub unsafe extern "C" fn win_move_after(mut win1: *mut win_T, mut win2: *mut win_T) {
    if win1 == win2 {
        return;
    }
    if (*win2).w_next != win1 {
        if (*(*win1).w_frame).fr_parent != (*(*win2).w_frame).fr_parent {
            iemsg(
                b"INTERNAL: trying to move a window into another frame\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
            return;
        }
        if win1 == lastwin.get() {
            let mut height: ::core::ffi::c_int = (*(*win1).w_prev).w_status_height;
            (*(*win1).w_prev).w_status_height = (*win1).w_status_height;
            (*win1).w_status_height = height;
            height = (*(*win1).w_prev).w_hsep_height;
            (*(*win1).w_prev).w_hsep_height = (*win1).w_hsep_height;
            (*win1).w_hsep_height = height;
            if (*(*win1).w_prev).w_vsep_width == 1 as ::core::ffi::c_int {
                (*(*win1).w_prev).w_vsep_width = 0 as ::core::ffi::c_int;
                (*(*(*win1).w_prev).w_frame).fr_width -= 1 as ::core::ffi::c_int;
                (*win1).w_vsep_width = 1 as ::core::ffi::c_int;
                (*(*win1).w_frame).fr_width += 1 as ::core::ffi::c_int;
            }
        } else if win2 == lastwin.get() {
            let mut height_0: ::core::ffi::c_int = (*win1).w_status_height;
            (*win1).w_status_height = (*win2).w_status_height;
            (*win2).w_status_height = height_0;
            height_0 = (*win1).w_hsep_height;
            (*win1).w_hsep_height = (*win2).w_hsep_height;
            (*win2).w_hsep_height = height_0;
            if (*win1).w_vsep_width == 1 as ::core::ffi::c_int {
                (*win2).w_vsep_width = 1 as ::core::ffi::c_int;
                (*(*win2).w_frame).fr_width += 1 as ::core::ffi::c_int;
                (*win1).w_vsep_width = 0 as ::core::ffi::c_int;
                (*(*win1).w_frame).fr_width -= 1 as ::core::ffi::c_int;
            }
        }
        win_remove(win1, ::core::ptr::null_mut::<tabpage_T>());
        frame_remove((*win1).w_frame);
        win_append(win2, win1, ::core::ptr::null_mut::<tabpage_T>());
        frame_append((*win2).w_frame, (*win1).w_frame);
        win_comp_pos();
        redraw_later(curwin.get(), UPD_NOT_VALID as ::core::ffi::c_int);
    }
    (*win1).w_pos_changed = true_0 != 0;
    (*win2).w_pos_changed = true_0 != 0;
    win_enter(win1, false_0 != 0);
}
unsafe extern "C" fn get_maximum_wincount(
    mut fr: *mut frame_T,
    mut height: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if (*fr).fr_layout as ::core::ffi::c_int != FR_COL {
        return height
            / (p_wmh.get() as ::core::ffi::c_int
                + STATUS_HEIGHT as ::core::ffi::c_int
                + (*frame2win(fr)).w_winbar_height);
    } else if global_winbar_height() != 0 {
        return height
            / (p_wmh.get() as ::core::ffi::c_int
                + STATUS_HEIGHT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int);
    }
    let mut frp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
    let mut total_wincount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    frp = (*fr).fr_child;
    while !frp.is_null() {
        let mut wp: *mut win_T = frame2win(frp);
        if (height as OptInt)
            < p_wmh.get()
                + STATUS_HEIGHT as ::core::ffi::c_int as OptInt
                + (*wp).w_winbar_height as OptInt
        {
            break;
        }
        height -= p_wmh.get() as ::core::ffi::c_int
            + STATUS_HEIGHT as ::core::ffi::c_int
            + (*wp).w_winbar_height;
        total_wincount += 1 as ::core::ffi::c_int;
        frp = (*frp).fr_next;
    }
    total_wincount +=
        height / (p_wmh.get() as ::core::ffi::c_int + STATUS_HEIGHT as ::core::ffi::c_int);
    return total_wincount;
}
pub unsafe extern "C" fn win_equal(
    mut next_curwin: *mut win_T,
    mut current: bool,
    mut dir: ::core::ffi::c_int,
) {
    if dir == 0 as ::core::ffi::c_int {
        dir = *p_ead.get() as ::core::ffi::c_uchar as ::core::ffi::c_int;
    }
    win_equal_rec(
        if next_curwin.is_null() {
            curwin.get()
        } else {
            next_curwin
        },
        current,
        topframe.get(),
        dir,
        0 as ::core::ffi::c_int,
        tabline_height(),
        Columns.get(),
        (*topframe.get()).fr_height,
    );
    if !is_aucmd_win(next_curwin) {
        win_fix_scroll(true_0 != 0);
    }
}
unsafe extern "C" fn win_equal_rec(
    mut next_curwin: *mut win_T,
    mut current: bool,
    mut topfr: *mut frame_T,
    mut dir: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut row: ::core::ffi::c_int,
    mut width: ::core::ffi::c_int,
    mut height: ::core::ffi::c_int,
) {
    let mut extra_sep: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut totwincount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut next_curwin_size: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut room: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut has_next_curwin: bool = false_0 != 0;
    if (*topfr).fr_layout as ::core::ffi::c_int == FR_LEAF {
        if (*topfr).fr_height != height
            || (*(*topfr).fr_win).w_winrow != row
            || (*topfr).fr_width != width
            || (*(*topfr).fr_win).w_wincol != col
        {
            (*(*topfr).fr_win).w_winrow = row;
            frame_new_height(topfr, height, false_0 != 0, false_0 != 0, false_0 != 0);
            (*(*topfr).fr_win).w_wincol = col;
            frame_new_width(topfr, width, false_0 != 0, false_0 != 0);
            redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
        }
    } else if (*topfr).fr_layout as ::core::ffi::c_int == FR_ROW {
        (*topfr).fr_width = width;
        (*topfr).fr_height = height;
        if dir != 'v' as ::core::ffi::c_int {
            let mut n: ::core::ffi::c_int = frame_minwidth(topfr, NOWIN);
            if col + width == Columns.get() {
                extra_sep = 1 as ::core::ffi::c_int;
            } else {
                extra_sep = 0 as ::core::ffi::c_int;
            }
            totwincount =
                (n + extra_sep) / (p_wmw.get() as ::core::ffi::c_int + 1 as ::core::ffi::c_int);
            has_next_curwin = frame_has_win(topfr, next_curwin);
            let mut m: ::core::ffi::c_int = frame_minwidth(topfr, next_curwin);
            room = width - m;
            if room < 0 as ::core::ffi::c_int {
                next_curwin_size = p_wiw.get() as ::core::ffi::c_int + room;
                room = 0 as ::core::ffi::c_int;
            } else {
                next_curwin_size = -1 as ::core::ffi::c_int;
                let mut fr: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
                fr = (*topfr).fr_child;
                while !fr.is_null() {
                    if frame_fixed_width(fr) {
                        n = frame_minwidth(fr, NOWIN);
                        let mut new_size: ::core::ffi::c_int = (*fr).fr_width;
                        if frame_has_win(fr, next_curwin) {
                            room += p_wiw.get() as ::core::ffi::c_int
                                - p_wmw.get() as ::core::ffi::c_int;
                            next_curwin_size = 0 as ::core::ffi::c_int;
                            new_size = if new_size > p_wiw.get() as ::core::ffi::c_int {
                                new_size
                            } else {
                                p_wiw.get() as ::core::ffi::c_int
                            };
                        } else {
                            totwincount -=
                                (n + (if (*fr).fr_next.is_null() {
                                    extra_sep
                                } else {
                                    0 as ::core::ffi::c_int
                                })) / (p_wmw.get() as ::core::ffi::c_int + 1 as ::core::ffi::c_int);
                        }
                        room -= new_size - n;
                        if room < 0 as ::core::ffi::c_int {
                            new_size += room;
                            room = 0 as ::core::ffi::c_int;
                        }
                        (*fr).fr_newwidth = new_size;
                    }
                    fr = (*fr).fr_next;
                }
                if next_curwin_size == -1 as ::core::ffi::c_int {
                    if !has_next_curwin {
                        next_curwin_size = 0 as ::core::ffi::c_int;
                    } else if totwincount > 1 as ::core::ffi::c_int
                        && ((room + (totwincount - 2 as ::core::ffi::c_int))
                            / (totwincount - 1 as ::core::ffi::c_int))
                            as OptInt
                            > p_wiw.get()
                    {
                        next_curwin_size = (room as OptInt
                            + p_wiw.get()
                            + (totwincount - 1 as ::core::ffi::c_int) as OptInt * p_wmw.get()
                            + (totwincount - 1 as ::core::ffi::c_int) as OptInt)
                            as ::core::ffi::c_int
                            / totwincount;
                        room -= next_curwin_size - p_wiw.get() as ::core::ffi::c_int;
                    } else {
                        next_curwin_size = p_wiw.get() as ::core::ffi::c_int;
                    }
                }
            }
            if has_next_curwin {
                totwincount -= 1;
            }
        }
        let mut fr_0: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
        fr_0 = (*topfr).fr_child;
        while !fr_0.is_null() {
            let mut wincount: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            let mut new_size_0: ::core::ffi::c_int = 0;
            if (*fr_0).fr_next.is_null() {
                new_size_0 = width;
            } else if dir == 'v' as ::core::ffi::c_int {
                new_size_0 = (*fr_0).fr_width;
            } else if frame_fixed_width(fr_0) {
                new_size_0 = (*fr_0).fr_newwidth;
                wincount = 0 as ::core::ffi::c_int;
            } else {
                let mut n_0: ::core::ffi::c_int = frame_minwidth(fr_0, NOWIN);
                wincount =
                    (n_0 + (if (*fr_0).fr_next.is_null() {
                        extra_sep
                    } else {
                        0 as ::core::ffi::c_int
                    })) / (p_wmw.get() as ::core::ffi::c_int + 1 as ::core::ffi::c_int);
                let mut m_0: ::core::ffi::c_int = frame_minwidth(fr_0, next_curwin);
                let mut hnc: bool = has_next_curwin as ::core::ffi::c_int != 0
                    && frame_has_win(fr_0, next_curwin) as ::core::ffi::c_int != 0;
                if hnc {
                    wincount -= 1;
                }
                if totwincount == 0 as ::core::ffi::c_int {
                    new_size_0 = room;
                } else {
                    new_size_0 =
                        (wincount * room + totwincount / 2 as ::core::ffi::c_int) / totwincount;
                }
                if hnc {
                    next_curwin_size -= p_wiw.get() as ::core::ffi::c_int - (m_0 - n_0);
                    next_curwin_size = if next_curwin_size > 0 as ::core::ffi::c_int {
                        next_curwin_size
                    } else {
                        0 as ::core::ffi::c_int
                    };
                    new_size_0 += next_curwin_size;
                    room -= new_size_0 - next_curwin_size;
                } else {
                    room -= new_size_0;
                }
                new_size_0 += n_0;
            }
            if !current
                || dir != 'v' as ::core::ffi::c_int
                || !(*topfr).fr_parent.is_null()
                || new_size_0 != (*fr_0).fr_width
                || frame_has_win(fr_0, next_curwin) as ::core::ffi::c_int != 0
            {
                win_equal_rec(
                    next_curwin,
                    current,
                    fr_0,
                    dir,
                    col,
                    row,
                    new_size_0,
                    height,
                );
            }
            col += new_size_0;
            width -= new_size_0;
            totwincount -= wincount;
            fr_0 = (*fr_0).fr_next;
        }
    } else {
        (*topfr).fr_width = width;
        (*topfr).fr_height = height;
        if dir != 'h' as ::core::ffi::c_int {
            let mut n_1: ::core::ffi::c_int = frame_minheight(topfr, NOWIN);
            if row + height >= cmdline_row.get() && p_ls.get() == 0 as OptInt {
                extra_sep = STATUS_HEIGHT as ::core::ffi::c_int;
            } else if global_stl_height() > 0 as ::core::ffi::c_int {
                extra_sep = 1 as ::core::ffi::c_int;
            } else {
                extra_sep = 0 as ::core::ffi::c_int;
            }
            totwincount = get_maximum_wincount(topfr, n_1 + extra_sep);
            has_next_curwin = frame_has_win(topfr, next_curwin);
            let mut m_1: ::core::ffi::c_int = frame_minheight(topfr, next_curwin);
            room = height - m_1;
            if room < 0 as ::core::ffi::c_int {
                next_curwin_size = p_wh.get() as ::core::ffi::c_int + room;
                room = 0 as ::core::ffi::c_int;
            } else {
                next_curwin_size = -1 as ::core::ffi::c_int;
                let mut fr_1: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
                fr_1 = (*topfr).fr_child;
                while !fr_1.is_null() {
                    if frame_fixed_height(fr_1) {
                        n_1 = frame_minheight(fr_1, NOWIN);
                        let mut new_size_1: ::core::ffi::c_int = (*fr_1).fr_height;
                        if frame_has_win(fr_1, next_curwin) {
                            room += p_wh.get() as ::core::ffi::c_int
                                - p_wmh.get() as ::core::ffi::c_int;
                            next_curwin_size = 0 as ::core::ffi::c_int;
                            new_size_1 = if new_size_1 > p_wh.get() as ::core::ffi::c_int {
                                new_size_1
                            } else {
                                p_wh.get() as ::core::ffi::c_int
                            };
                        } else {
                            totwincount -= get_maximum_wincount(
                                fr_1,
                                n_1 + (if (*fr_1).fr_next.is_null() {
                                    extra_sep
                                } else {
                                    0 as ::core::ffi::c_int
                                }),
                            );
                        }
                        room -= new_size_1 - n_1;
                        if room < 0 as ::core::ffi::c_int {
                            new_size_1 += room;
                            room = 0 as ::core::ffi::c_int;
                        }
                        (*fr_1).fr_newheight = new_size_1;
                    }
                    fr_1 = (*fr_1).fr_next;
                }
                if next_curwin_size == -1 as ::core::ffi::c_int {
                    if !has_next_curwin {
                        next_curwin_size = 0 as ::core::ffi::c_int;
                    } else if totwincount > 1 as ::core::ffi::c_int
                        && ((room + (totwincount - 2 as ::core::ffi::c_int))
                            / (totwincount - 1 as ::core::ffi::c_int))
                            as OptInt
                            > p_wh.get()
                    {
                        next_curwin_size = (room as OptInt
                            + p_wh.get()
                            + (totwincount - 1 as ::core::ffi::c_int) as OptInt * p_wmh.get()
                            + (totwincount - 1 as ::core::ffi::c_int) as OptInt)
                            as ::core::ffi::c_int
                            / totwincount;
                        room -= next_curwin_size - p_wh.get() as ::core::ffi::c_int;
                    } else {
                        next_curwin_size = p_wh.get() as ::core::ffi::c_int;
                    }
                }
            }
            if has_next_curwin {
                totwincount -= 1;
            }
        }
        let mut fr_2: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
        fr_2 = (*topfr).fr_child;
        while !fr_2.is_null() {
            let mut new_size_2: ::core::ffi::c_int = 0;
            let mut wincount_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            if (*fr_2).fr_next.is_null() {
                new_size_2 = height;
            } else if dir == 'h' as ::core::ffi::c_int {
                new_size_2 = (*fr_2).fr_height;
            } else if frame_fixed_height(fr_2) {
                new_size_2 = (*fr_2).fr_newheight;
                wincount_0 = 0 as ::core::ffi::c_int;
            } else {
                let mut n_2: ::core::ffi::c_int = frame_minheight(fr_2, NOWIN);
                wincount_0 = get_maximum_wincount(
                    fr_2,
                    n_2 + (if (*fr_2).fr_next.is_null() {
                        extra_sep
                    } else {
                        0 as ::core::ffi::c_int
                    }),
                );
                let mut m_2: ::core::ffi::c_int = frame_minheight(fr_2, next_curwin);
                let mut hnc_0: bool = has_next_curwin as ::core::ffi::c_int != 0
                    && frame_has_win(fr_2, next_curwin) as ::core::ffi::c_int != 0;
                if hnc_0 {
                    wincount_0 -= 1;
                }
                if totwincount == 0 as ::core::ffi::c_int {
                    new_size_2 = room;
                } else {
                    new_size_2 =
                        (wincount_0 * room + totwincount / 2 as ::core::ffi::c_int) / totwincount;
                }
                if hnc_0 {
                    next_curwin_size -= p_wh.get() as ::core::ffi::c_int - (m_2 - n_2);
                    new_size_2 += next_curwin_size;
                    room -= new_size_2 - next_curwin_size;
                } else {
                    room -= new_size_2;
                }
                new_size_2 += n_2;
            }
            if !current
                || dir != 'h' as ::core::ffi::c_int
                || !(*topfr).fr_parent.is_null()
                || new_size_2 != (*fr_2).fr_height
                || frame_has_win(fr_2, next_curwin) as ::core::ffi::c_int != 0
            {
                win_equal_rec(next_curwin, current, fr_2, dir, col, row, width, new_size_2);
            }
            row += new_size_2;
            height -= new_size_2;
            totwincount -= wincount_0;
            fr_2 = (*fr_2).fr_next;
        }
    };
}
pub unsafe extern "C" fn leaving_window(win: *mut win_T) {
    if !bt_prompt((*win).w_buffer) || is_aucmd_win(win) as ::core::ffi::c_int != 0 {
        return;
    }
    (*(*win).w_buffer).b_prompt_insert = restart_edit.get();
    if restart_edit.get() != NUL && mode_displayed.get() as ::core::ffi::c_int != 0 {
        clear_cmdline.set(true_0 != 0);
    }
    restart_edit.set(NUL);
    if State.get() & MODE_INSERT as ::core::ffi::c_int != 0 && !stop_insert_mode.get() {
        stop_insert_mode.set(true_0 != 0);
        if (*(*win).w_buffer).b_prompt_insert == NUL {
            (*(*win).w_buffer).b_prompt_insert = 'A' as ::core::ffi::c_int;
        }
    }
}
pub unsafe extern "C" fn entering_window(win: *mut win_T) {
    if !bt_prompt((*win).w_buffer) || is_aucmd_win(win) as ::core::ffi::c_int != 0 {
        return;
    }
    if (*(*win).w_buffer).b_prompt_insert != NUL {
        stop_insert_mode.set(false_0 != 0);
    }
    if State.get() & MODE_INSERT as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        restart_edit.set((*(*win).w_buffer).b_prompt_insert);
    }
}
pub unsafe extern "C" fn win_init_empty(mut wp: *mut win_T) {
    redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
    (*wp).w_lines_valid = 0 as ::core::ffi::c_int;
    (*wp).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
    (*wp).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    (*wp).w_curswant = (*wp).w_cursor.col;
    (*wp).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    (*wp).w_pcmark.lnum = 1 as ::core::ffi::c_int as linenr_T;
    (*wp).w_pcmark.col = 0 as ::core::ffi::c_int as colnr_T;
    (*wp).w_prev_pcmark.lnum = 0 as ::core::ffi::c_int as linenr_T;
    (*wp).w_prev_pcmark.col = 0 as ::core::ffi::c_int as colnr_T;
    (*wp).w_topline = 1 as ::core::ffi::c_int as linenr_T;
    (*wp).w_topfill = 0 as ::core::ffi::c_int;
    (*wp).w_botline = 2 as ::core::ffi::c_int as linenr_T;
    (*wp).w_valid = 0 as ::core::ffi::c_int;
    (*wp).w_s = &raw mut (*(*wp).w_buffer).b_s;
}
pub unsafe extern "C" fn curwin_init() {
    win_init_empty(curwin.get());
}
pub unsafe extern "C" fn close_windows(mut buf: *mut buf_T, mut keep_curwin: bool) {
    let mut nexttp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    (*RedrawingDisabled.ptr()) += 1;
    let mut wp: *mut win_T = lastwin.get();
    '_theend: {
        while !wp.is_null()
            && (is_aucmd_win(lastwin.get()) as ::core::ffi::c_int != 0
                || !one_window(wp, ::core::ptr::null_mut::<tabpage_T>()))
        {
            if (*wp).w_buffer == buf
                && (!keep_curwin || wp != curwin.get())
                && !(win_locked(wp) != 0 || (*(*wp).w_buffer).b_locked > 0 as ::core::ffi::c_int)
            {
                if window_layout_locked(CMD_SIZE) {
                    break '_theend;
                }
                if win_close(wp, false_0 != 0, false_0 != 0) == FAIL {
                    break;
                }
                wp = lastwin.get();
            } else {
                wp = (*wp).w_prev;
            }
        }
        nexttp = ::core::ptr::null_mut::<tabpage_T>();
        let mut tp: *mut tabpage_T = first_tabpage.get();
        loop {
            if tp.is_null() {
                break '_theend;
            }
            nexttp = (*tp).tp_next;
            's_53: {
                if tp != curtab.get() {
                    let mut wp_0: *mut win_T = (*tp).tp_lastwin;
                    loop {
                        if wp_0.is_null() {
                            break 's_53;
                        }
                        if (*wp_0).w_buffer == buf
                            && !(win_locked(wp_0) != 0
                                || (*(*wp_0).w_buffer).b_locked > 0 as ::core::ffi::c_int)
                        {
                            if window_layout_locked(CMD_SIZE) {
                                break '_theend;
                            }
                            if !win_close_othertab(wp_0, false_0, tp, false_0 != 0) {
                                break 's_53;
                            }
                            nexttp = first_tabpage.get();
                            break 's_53;
                        } else {
                            wp_0 = (*wp_0).w_prev;
                        }
                    }
                }
            }
            tp = nexttp;
        }
    }
    (*RedrawingDisabled.ptr()) -= 1;
}
pub unsafe extern "C" fn last_window(mut win: *mut win_T) -> bool {
    return one_window(win, ::core::ptr::null_mut::<tabpage_T>()) as ::core::ffi::c_int != 0
        && (*first_tabpage.get()).tp_next.is_null();
}
pub unsafe extern "C" fn one_window(mut win: *mut win_T, mut tp: *mut tabpage_T) -> bool {
    let mut first: *mut win_T = if !tp.is_null() {
        (*tp).tp_firstwin
    } else {
        firstwin.get()
    };
    '_c2rust_label: {
        if (tp.is_null() || tp != curtab.get()) && !(*first).w_floating {
        } else {
            __assert_fail(
                b"(!tp || tp != curtab) && !first->w_floating\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2665 as ::core::ffi::c_uint,
                b"_Bool one_window(win_T *, tabpage_T *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    return first == win
        && ((*win).w_next.is_null() || (*(*win).w_next).w_floating as ::core::ffi::c_int != 0);
}
unsafe extern "C" fn can_close_floating_windows(mut tp: *mut tabpage_T) -> bool {
    '_c2rust_label: {
        if tp != curtab.get() && (!tp.is_null() || !is_aucmd_win(lastwin.get())) {
        } else {
            __assert_fail(
                b"tp != curtab && (tp || !is_aucmd_win(lastwin))\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2676 as ::core::ffi::c_uint,
                b"_Bool can_close_floating_windows(tabpage_T *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut wp: *mut win_T = if !tp.is_null() {
        (*tp).tp_lastwin
    } else {
        lastwin.get()
    };
    while (*wp).w_floating {
        let mut buf: *mut buf_T = (*wp).w_buffer;
        let mut need_hide: ::core::ffi::c_int = (bufIsChanged(buf) as ::core::ffi::c_int != 0
            && (*buf).b_nwindows <= 1 as ::core::ffi::c_int)
            as ::core::ffi::c_int;
        if need_hide != 0 && !buf_hide(buf) {
            return false_0 != 0;
        }
        wp = (*wp).w_prev;
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn can_close_in_cmdwin(mut win: *mut win_T, mut err: *mut Error) -> bool {
    if cmdwin_type.get() != 0 as ::core::ffi::c_int {
        if win == cmdwin_win.get() {
            cmdwin_result.set(Ctrl_C);
            return false_0 != 0;
        } else if win == cmdwin_old_curwin.get() {
            api_set_error(
                err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_cmdwin as *const ::core::ffi::c_char,
            );
            return false_0 != 0;
        }
    }
    return true_0 != 0;
}
unsafe extern "C" fn close_last_window_tabpage(
    mut win: *mut win_T,
    mut free_buf: bool,
    mut prev_curtab: *mut tabpage_T,
) -> bool {
    if !(firstwin.get() == lastwin.get()) {
        return false_0 != 0;
    }
    let mut old_curbuf: *mut buf_T = curbuf.get();
    let mut term: *mut Terminal = if !(*win).w_buffer.is_null() {
        (*(*win).w_buffer).terminal
    } else {
        ::core::ptr::null_mut::<Terminal>()
    };
    if !term.is_null() {
        free_buf = false_0 != 0;
    }
    goto_tabpage_tp(alt_tabpage(), false_0 != 0, !(*win).w_buffer.is_null());
    if curtab.get() != prev_curtab
        && valid_tabpage(prev_curtab) as ::core::ffi::c_int != 0
        && (*prev_curtab).tp_firstwin == win
    {
        win_close_othertab(
            win,
            free_buf as ::core::ffi::c_int,
            prev_curtab,
            false_0 != 0,
        );
    }
    entering_window(curwin.get());
    apply_autocmds(
        EVENT_WINENTER,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
    apply_autocmds(
        EVENT_TABENTER,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
    if old_curbuf != curbuf.get() {
        apply_autocmds(
            EVENT_BUFENTER,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
    }
    return true_0 != 0;
}
unsafe extern "C" fn win_close_buffer(
    mut win: *mut win_T,
    mut action: ::core::ffi::c_int,
    mut abort_if_last: bool,
) -> bool {
    if !(*win).w_buffer.is_null() {
        reset_synblock(win);
    }
    if !(*win).w_buffer.is_null()
        && bt_quickfix((*win).w_buffer) as ::core::ffi::c_int != 0
        && (*(*win).w_buffer).b_nwindows == 1 as ::core::ffi::c_int
    {
        (*(*win).w_buffer).b_p_bl = false_0;
    }
    let mut retval: bool = false_0 != 0;
    if !(*win).w_buffer.is_null() {
        let mut bufref: bufref_T = bufref_T {
            br_buf: ::core::ptr::null_mut::<buf_T>(),
            br_fnum: 0,
            br_buf_free_count: 0,
        };
        set_bufref(&raw mut bufref, curbuf.get());
        (*win).w_locked = true_0 != 0;
        retval = close_buffer(win, (*win).w_buffer, action, abort_if_last, true_0 != 0);
        if win_valid_any_tab(win) {
            (*win).w_locked = false_0 != 0;
        }
        if !bufref_valid(&raw mut bufref) {
            curbuf.set(firstbuf.get());
        }
    }
    return retval;
}
unsafe extern "C" fn win_unclose_buffer(
    mut win: *mut win_T,
    mut bufref: *mut bufref_T,
    mut did_decrement: bool,
) {
    if (*win).w_buffer.is_null() {
        (*win).w_buffer = firstbuf.get();
        (*firstbuf.get()).b_nwindows += 1;
        if win == curwin.get() {
            curbuf.set((*curwin.get()).w_buffer);
        }
        win_init_empty(win);
    } else if did_decrement as ::core::ffi::c_int != 0
        && (*win).w_buffer == (*bufref).br_buf
        && bufref_valid(bufref) as ::core::ffi::c_int != 0
    {
        (*(*win).w_buffer).b_nwindows += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn win_close(
    mut win: *mut win_T,
    mut free_buf: bool,
    mut force: bool,
) -> ::core::ffi::c_int {
    let mut prev_curtab: *mut tabpage_T = curtab.get();
    let mut win_frame: *mut frame_T = if (*win).w_floating as ::core::ffi::c_int != 0 {
        ::core::ptr::null_mut::<frame_T>()
    } else {
        (*(*win).w_frame).fr_parent
    };
    let had_diffmode: bool = (*win).w_onebuf_opt.wo_diff != 0;
    if last_window(win) {
        emsg(gettext(
            (e_cannot_close_last_window.ptr() as *const _) as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    if !(*win).w_floating && window_layout_locked(CMD_close) as ::core::ffi::c_int != 0 {
        return FAIL;
    }
    if win_locked(win) != 0
        || !(*win).w_buffer.is_null() && (*(*win).w_buffer).b_locked > 0 as ::core::ffi::c_int
    {
        return FAIL;
    }
    if is_aucmd_win(win) {
        emsg(gettext(
            &raw const e_autocmd_close as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    if (*lastwin.get()).w_floating as ::core::ffi::c_int != 0
        && one_window(win, ::core::ptr::null_mut::<tabpage_T>()) as ::core::ffi::c_int != 0
    {
        if is_aucmd_win(lastwin.get()) {
            emsg(gettext(
                b"E814: Cannot close window, only autocmd window would remain\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
        if force as ::core::ffi::c_int != 0
            || can_close_floating_windows(::core::ptr::null_mut::<tabpage_T>())
                as ::core::ffi::c_int
                != 0
        {
            while (*lastwin.get()).w_floating {
                if win_close(
                    lastwin.get(),
                    !buf_hide((*lastwin.get()).w_buffer),
                    true_0 != 0,
                ) == FAIL
                {
                    return FAIL;
                }
            }
            if !win_valid_any_tab(win) {
                return FAIL;
            }
            if last_window(win) {
                emsg(gettext(
                    (e_cannot_close_last_window.ptr() as *const _) as *const ::core::ffi::c_char,
                ));
                return FAIL;
            }
        } else {
            emsg(&raw const e_floatonly as *const ::core::ffi::c_char);
            return FAIL;
        }
    }
    if close_last_window_tabpage(win, free_buf, prev_curtab) {
        return FAIL;
    }
    let mut help_window: bool = false_0 != 0;
    let mut quickfix_window: bool = false_0 != 0;
    if bt_help((*win).w_buffer) {
        help_window = true_0 != 0;
    } else {
        clear_snapshot(curtab.get(), SNAP_HELP_IDX);
    }
    if bt_quickfix((*win).w_buffer) {
        quickfix_window = true_0 != 0;
    } else {
        clear_snapshot(curtab.get(), SNAP_QUICKFIX_IDX);
    }
    let mut other_buffer: bool = false_0 != 0;
    if win == curwin.get() {
        leaving_window(curwin.get());
        let mut wp: *mut win_T = if (*win).w_floating as ::core::ffi::c_int != 0 {
            win_float_find_altwin(win, ::core::ptr::null::<tabpage_T>())
        } else {
            frame2win(win_altframe(win, ::core::ptr::null_mut::<tabpage_T>()))
        };
        if (*wp).w_buffer != curbuf.get() {
            reset_VIsual_and_resel();
            other_buffer = true_0 != 0;
            if !win_valid(win) {
                return FAIL;
            }
            (*win).w_locked = true_0 != 0;
            apply_autocmds(
                EVENT_BUFLEAVE,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
            if !win_valid(win) {
                return FAIL;
            }
            (*win).w_locked = false_0 != 0;
            if last_window(win) {
                return FAIL;
            }
        }
        (*win).w_locked = true_0 != 0;
        apply_autocmds(
            EVENT_WINLEAVE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        if !win_valid(win) {
            return FAIL;
        }
        (*win).w_locked = false_0 != 0;
        if last_window(win) {
            return FAIL;
        }
        if aborting() {
            return FAIL;
        }
    }
    do_autocmd_winclosed(win);
    if !win_valid_any_tab(win) {
        return OK;
    }
    let mut bufref: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    set_bufref(&raw mut bufref, (*win).w_buffer);
    let mut did_decrement: bool = win_close_buffer(
        win,
        if free_buf as ::core::ffi::c_int != 0 {
            DOBUF_UNLOAD as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        },
        true_0 != 0,
    );
    if win_valid(win) as ::core::ffi::c_int != 0
        && (*win).w_buffer.is_null()
        && !(*win).w_floating
        && last_window(win) as ::core::ffi::c_int != 0
    {
        if (*curwin.get()).w_buffer.is_null() {
            (*curwin.get()).w_buffer = curbuf.get();
        }
        getout(0 as ::core::ffi::c_int);
    }
    if curtab.get() != prev_curtab
        && win_valid_any_tab(win) as ::core::ffi::c_int != 0
        && (*win).w_buffer.is_null()
    {
        win_close_othertab(win, false_0, prev_curtab, force);
        return FAIL;
    }
    if !win_valid(win) {
        return FAIL;
    }
    if one_window(win, ::core::ptr::null_mut::<tabpage_T>()) as ::core::ffi::c_int != 0
        && ((*first_tabpage.get()).tp_next.is_null()
            || (*lastwin.get()).w_floating as ::core::ffi::c_int != 0)
    {
        if !(*first_tabpage.get()).tp_next.is_null() {
            emsg(&raw const e_floatonly as *const ::core::ffi::c_char);
        }
        win_unclose_buffer(win, &raw mut bufref, did_decrement);
        return FAIL;
    }
    if close_last_window_tabpage(win, free_buf, prev_curtab) {
        return FAIL;
    }
    (*split_disallowed.ptr()) += 1;
    let mut was_floating: bool = (*win).w_floating;
    if ui_has(kUIMultigrid) {
        ui_call_win_close((*win).w_grid_alloc.handle as Integer);
    }
    if (*win).w_floating {
        ui_comp_remove_grid(&raw mut (*win).w_grid_alloc);
        '_c2rust_label: {
            if !(*first_tabpage.ptr()).is_null() {
            } else {
                __assert_fail(
                    b"first_tabpage != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2999 as ::core::ffi::c_uint,
                    b"int win_close(win_T *, _Bool, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if (*win).w_config.external {
            let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
            while !tp.is_null() {
                if tp != curtab.get() && (*tp).tp_curwin == win {
                    (*tp).tp_curwin = (*tp).tp_firstwin;
                }
                tp = (*tp).tp_next as *mut tabpage_T;
            }
        }
    }
    set_bufref(&raw mut bufref, (*win).w_buffer);
    let mut had_cmdline_ruler: bool =
        p_ru.get() != 0 && win == curwin.get() && (*win).w_status_height == 0 as ::core::ffi::c_int;
    let mut dir: ::core::ffi::c_int = 0;
    let mut wp_0: *mut win_T =
        win_free_mem(win, &raw mut dir, ::core::ptr::null_mut::<tabpage_T>());
    if help_window as ::core::ffi::c_int != 0 || quickfix_window as ::core::ffi::c_int != 0 {
        let mut prev_win: *mut win_T =
            get_snapshot_curwin(if help_window as ::core::ffi::c_int != 0 {
                SNAP_HELP_IDX
            } else {
                SNAP_QUICKFIX_IDX
            });
        if win_valid(prev_win) {
            wp_0 = prev_win;
        }
    }
    let mut close_curwin: bool = false_0 != 0;
    if win == curwin.get() {
        curwin.set(wp_0);
        if (*wp_0).w_onebuf_opt.wo_pvw != 0
            || bt_quickfix((*wp_0).w_buffer) as ::core::ffi::c_int != 0
        {
            loop {
                if (*wp_0).w_next.is_null() {
                    wp_0 = firstwin.get();
                } else {
                    wp_0 = (*wp_0).w_next;
                }
                if wp_0 == curwin.get() {
                    break;
                }
                if !((*wp_0).w_onebuf_opt.wo_pvw == 0
                    && !bt_quickfix((*wp_0).w_buffer)
                    && !((*wp_0).w_floating as ::core::ffi::c_int != 0
                        && ((*wp_0).w_config.hide as ::core::ffi::c_int != 0
                            || !(*wp_0).w_config.focusable)))
                {
                    continue;
                }
                curwin.set(wp_0);
                break;
            }
        }
        curbuf.set((*curwin.get()).w_buffer);
        close_curwin = true_0 != 0;
        check_cursor(curwin.get());
    }
    if !was_floating {
        last_status(false_0 != 0);
        if !(*curwin.get()).w_floating
            && p_ea.get() != 0
            && (*p_ead.get() as ::core::ffi::c_int == 'b' as ::core::ffi::c_int
                || *p_ead.get() as ::core::ffi::c_int == dir)
        {
            win_equal(
                curwin.get(),
                (*(*curwin.get()).w_frame).fr_parent == win_frame,
                dir,
            );
        } else {
            win_comp_pos();
            win_fix_scroll(false_0 != 0);
        }
    } else if had_cmdline_ruler as ::core::ffi::c_int != 0
        && (*wp_0).w_status_height > 0 as ::core::ffi::c_int
    {
        redraw_cmdline.set(true_0 != 0);
    }
    if !bufref.br_buf.is_null()
        && bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
        && !(*bufref.br_buf).terminal.is_null()
    {
        terminal_check_size((*bufref.br_buf).terminal);
    }
    if close_curwin {
        win_enter_ext(
            wp_0,
            WEE_CURWIN_INVALID as ::core::ffi::c_int
                | WEE_TRIGGER_ENTER_AUTOCMDS as ::core::ffi::c_int
                | WEE_TRIGGER_LEAVE_AUTOCMDS as ::core::ffi::c_int,
        );
        if other_buffer {
            apply_autocmds(
                EVENT_BUFENTER,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
        }
    }
    if firstwin.get() == lastwin.get()
        && (*curwin.get()).w_locked as ::core::ffi::c_int != 0
        && (*curbuf.get()).b_locked_split != 0
        && !(*first_tabpage.get()).tp_next.is_null()
    {
        apply_autocmds(
            EVENT_TABLEAVE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
    }
    (*split_disallowed.ptr()) -= 1;
    if help_window as ::core::ffi::c_int != 0 || quickfix_window as ::core::ffi::c_int != 0 {
        restore_snapshot(
            if help_window as ::core::ffi::c_int != 0 {
                SNAP_HELP_IDX
            } else {
                SNAP_QUICKFIX_IDX
            },
            close_curwin as ::core::ffi::c_int,
        );
    }
    if diffopt_closeoff() as ::core::ffi::c_int != 0
        && had_diffmode as ::core::ffi::c_int != 0
        && curtab.get() == prev_curtab
    {
        let mut diffcount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut dwin: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !dwin.is_null() {
            if (*dwin).w_onebuf_opt.wo_diff != 0 {
                diffcount += 1;
            }
            dwin = (*dwin).w_next;
        }
        if diffcount == 1 as ::core::ffi::c_int {
            do_cmdline_cmd(b"diffoff!\0".as_ptr() as *const ::core::ffi::c_char);
        }
    }
    (*curwin.get()).w_pos_changed = true_0 != 0;
    if !was_floating {
        redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
    }
    return OK;
}
unsafe extern "C" fn trigger_winnewpre() {
    window_layout_lock();
    apply_autocmds(
        EVENT_WINNEWPRE,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        ::core::ptr::null_mut::<buf_T>(),
    );
    window_layout_unlock();
}
unsafe extern "C" fn do_autocmd_winclosed(mut win: *mut win_T) {
    static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if recursive.get() as ::core::ffi::c_int != 0 || !has_event(EVENT_WINCLOSED) {
        return;
    }
    recursive.set(true_0 != 0);
    let mut winid: [::core::ffi::c_char; 65] = [0; 65];
    vim_snprintf(
        &raw mut winid as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 65]>(),
        b"%d\0".as_ptr() as *const ::core::ffi::c_char,
        (*win).handle,
    );
    apply_autocmds(
        EVENT_WINCLOSED,
        &raw mut winid as *mut ::core::ffi::c_char,
        &raw mut winid as *mut ::core::ffi::c_char,
        false_0 != 0,
        (*win).w_buffer,
    );
    recursive.set(false_0 != 0);
}
pub unsafe extern "C" fn trigger_tabclosedpre(mut tp: *mut tabpage_T) {
    static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let mut ptp: *mut tabpage_T = curtab.get();
    if !has_event(EVENT_TABCLOSEDPRE) || recursive.get() as ::core::ffi::c_int != 0 {
        return;
    }
    if valid_tabpage(tp) {
        goto_tabpage_tp(tp, false_0 != 0, false_0 != 0);
    }
    recursive.set(true_0 != 0);
    window_layout_lock();
    apply_autocmds(
        EVENT_TABCLOSEDPRE,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        ::core::ptr::null_mut::<buf_T>(),
    );
    window_layout_unlock();
    recursive.set(false_0 != 0);
    if valid_tabpage(ptp) {
        goto_tabpage_tp(ptp, false_0 != 0, false_0 != 0);
    } else {
        goto_tabpage_tp(first_tabpage.get(), false_0 != 0, false_0 != 0);
    };
}
pub unsafe extern "C" fn win_close_othertab(
    mut win: *mut win_T,
    mut free_buf: ::core::ffi::c_int,
    mut tp: *mut tabpage_T,
    mut force: bool,
) -> bool {
    let mut bufref: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    let mut free_tp_idx: ::core::ffi::c_int = 0;
    let mut dir: ::core::ffi::c_int = 0;
    '_c2rust_label: {
        if tp != curtab.get() {
        } else {
            __assert_fail(
                b"tp != curtab\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                3194 as ::core::ffi::c_uint,
                b"_Bool win_close_othertab(win_T *, int, tabpage_T *, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut did_decrement: bool = false_0 != 0;
    if window_layout_locked(CMD_SIZE) {
        return false_0 != 0;
    }
    if win_locked(win) != 0
        || !(*win).w_buffer.is_null() && (*(*win).w_buffer).b_locked > 0 as ::core::ffi::c_int
    {
        return false_0 != 0;
    }
    if is_aucmd_win(win) {
        emsg(gettext(
            &raw const e_autocmd_close as *const ::core::ffi::c_char,
        ));
        return false_0 != 0;
    }
    '_leave_open: {
        if (*(*tp).tp_lastwin).w_floating as ::core::ffi::c_int != 0
            && one_window(win, tp) as ::core::ffi::c_int != 0
        {
            if force as ::core::ffi::c_int != 0
                || can_close_floating_windows(tp) as ::core::ffi::c_int != 0
            {
                while (*(*tp).tp_lastwin).w_floating {
                    if !win_close_othertab(
                        (*tp).tp_lastwin,
                        !buf_hide((*(*tp).tp_lastwin).w_buffer) as ::core::ffi::c_int,
                        tp,
                        true_0 != 0,
                    ) {
                        break '_leave_open;
                    }
                }
                if !win_valid_any_tab(win) {
                    return false_0 != 0;
                }
            } else {
                emsg(&raw const e_floatonly as *const ::core::ffi::c_char);
                break '_leave_open;
            }
        }
        if !(*win).w_buffer.is_null() {
            do_autocmd_winclosed(win);
            if !win_valid_any_tab(win) {
                return false_0 != 0;
            }
        }
        if (*tp).tp_firstwin == (*tp).tp_lastwin && !(*tp).tp_did_tabclosedpre {
            trigger_tabclosedpre(tp);
            if !win_valid_any_tab(win) {
                return false_0 != 0;
            }
        }
        bufref = bufref_T {
            br_buf: ::core::ptr::null_mut::<buf_T>(),
            br_fnum: 0,
            br_buf_free_count: 0,
        };
        set_bufref(&raw mut bufref, (*win).w_buffer);
        if !(*win).w_buffer.is_null() {
            did_decrement = close_buffer(
                win,
                (*win).w_buffer,
                if free_buf != 0 {
                    DOBUF_UNLOAD as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                },
                false_0 != 0,
                true_0 != 0,
            );
        }
        if !(!valid_tabpage(tp) || tp == curtab.get()) {
            if tabpage_win_valid(tp, win) {
                if (*(*tp).tp_lastwin).w_floating as ::core::ffi::c_int != 0
                    && one_window(win, tp) as ::core::ffi::c_int != 0
                {
                    emsg(&raw const e_floatonly as *const ::core::ffi::c_char);
                } else {
                    free_tp_idx = 0 as ::core::ffi::c_int;
                    if (*tp).tp_firstwin == (*tp).tp_lastwin {
                        free_tp_idx = tabpage_index(tp);
                        let mut h: ::core::ffi::c_int = tabline_height();
                        if tp == first_tabpage.get() {
                            first_tabpage.set((*tp).tp_next);
                        } else {
                            let mut ptp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
                            ptp = first_tabpage.get();
                            while !ptp.is_null() && (*ptp).tp_next != tp {
                                ptp = (*ptp).tp_next;
                            }
                            if ptp.is_null() {
                                internal_error(b"win_close_othertab()\0".as_ptr()
                                    as *const ::core::ffi::c_char);
                                return false_0 != 0;
                            }
                            (*ptp).tp_next = (*tp).tp_next;
                        }
                        redraw_tabline.set(true_0 != 0);
                        if h != tabline_height() {
                            win_new_screen_rows();
                        }
                    }
                    set_bufref(&raw mut bufref, (*win).w_buffer);
                    dir = 0;
                    win_free_mem(win, &raw mut dir, tp);
                    if !bufref.br_buf.is_null()
                        && bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
                        && !(*bufref.br_buf).terminal.is_null()
                    {
                        terminal_check_size((*bufref.br_buf).terminal);
                    }
                    if free_tp_idx > 0 as ::core::ffi::c_int {
                        free_tabpage(tp);
                        if has_event(EVENT_TABCLOSED) {
                            let mut prev_idx: [::core::ffi::c_char; 65] = [0; 65];
                            vim_snprintf(
                                &raw mut prev_idx as *mut ::core::ffi::c_char,
                                NUMBUFLEN as ::core::ffi::c_int as size_t,
                                b"%i\0".as_ptr() as *const ::core::ffi::c_char,
                                free_tp_idx,
                            );
                            apply_autocmds(
                                EVENT_TABCLOSED,
                                &raw mut prev_idx as *mut ::core::ffi::c_char,
                                &raw mut prev_idx as *mut ::core::ffi::c_char,
                                false_0 != 0,
                                if !bufref.br_buf.is_null()
                                    && bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
                                {
                                    bufref.br_buf
                                } else {
                                    curbuf.get()
                                },
                            );
                        }
                    }
                    return true_0 != 0;
                }
            }
        }
    }
    if win_valid_any_tab(win) {
        win_unclose_buffer(win, &raw mut bufref, did_decrement);
    }
    return false_0 != 0;
}
unsafe extern "C" fn win_free_mem(
    mut win: *mut win_T,
    mut dirp: *mut ::core::ffi::c_int,
    mut tp: *mut tabpage_T,
) -> *mut win_T {
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut win_tp: *mut tabpage_T = if tp.is_null() { curtab.get() } else { tp };
    if !(*win).w_floating {
        let mut frp: *mut frame_T = (*win).w_frame;
        wp = winframe_remove(win, dirp, tp, ::core::ptr::null_mut::<*mut frame_T>());
        xfree(frp as *mut ::core::ffi::c_void);
    } else {
        *dirp = 'h' as ::core::ffi::c_int;
        wp = win_float_find_altwin(win, tp);
    }
    win_free(win, tp);
    if win == (*win_tp).tp_curwin {
        (*win_tp).tp_curwin = wp;
    }
    if win == cmdline_win.get() {
        cmdline_win.set(::core::ptr::null_mut::<win_T>());
    }
    return wp;
}
pub unsafe extern "C" fn winframe_remove(
    mut win: *mut win_T,
    mut dirp: *mut ::core::ffi::c_int,
    mut tp: *mut tabpage_T,
    mut unflat_altfr: *mut *mut frame_T,
) -> *mut win_T {
    let mut altfr: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
    let mut wp: *mut win_T = winframe_find_altwin(win, dirp, tp, &raw mut altfr);
    if wp.is_null() {
        return ::core::ptr::null_mut::<win_T>();
    }
    let mut frp_close: *mut frame_T = (*win).w_frame;
    (*frame_locked.ptr()) += 1;
    let topleft: *const win_T = frame2win((*frp_close).fr_parent);
    let mut row: ::core::ffi::c_int = (*topleft).w_winrow;
    let mut col: ::core::ffi::c_int = (*topleft).w_wincol;
    if (*win).w_vsep_width == 0 as ::core::ffi::c_int
        && (*(*frp_close).fr_parent).fr_layout as ::core::ffi::c_int == FR_ROW
        && !(*frp_close).fr_prev.is_null()
    {
        frame_set_vsep((*frp_close).fr_prev, false_0 != 0);
    }
    frame_remove(frp_close);
    if *dirp == 'v' as ::core::ffi::c_int {
        frame_new_height(
            altfr,
            (*altfr).fr_height + (*frp_close).fr_height,
            altfr == (*frp_close).fr_next,
            false_0 != 0,
            false_0 != 0,
        );
    } else {
        '_c2rust_label: {
            if *dirp == 'h' as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"*dirp == 'h'\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3457 as ::core::ffi::c_uint,
                    b"win_T *winframe_remove(win_T *, int *, tabpage_T *, frame_T **)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        frame_new_width(
            altfr,
            (*altfr).fr_width + (*frp_close).fr_width,
            altfr == (*frp_close).fr_next,
            false_0 != 0,
        );
    }
    if altfr != (*frp_close).fr_prev {
        frame_comp_pos((*frp_close).fr_parent, &raw mut row, &raw mut col);
    }
    if unflat_altfr.is_null() {
        frame_flatten(altfr);
    } else {
        *unflat_altfr = altfr;
    }
    (*frame_locked.ptr()) -= 1;
    return wp;
}
pub unsafe extern "C" fn winframe_find_altwin(
    mut win: *mut win_T,
    mut dirp: *mut ::core::ffi::c_int,
    mut tp: *mut tabpage_T,
    mut altfr: *mut *mut frame_T,
) -> *mut win_T {
    '_c2rust_label: {
        if tp.is_null() || tp != curtab.get() {
        } else {
            __assert_fail(
                b"tp == NULL || tp != curtab\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                3492 as ::core::ffi::c_uint,
                b"win_T *winframe_find_altwin(win_T *, int *, tabpage_T *, frame_T **)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if one_window(win, tp) {
        return ::core::ptr::null_mut::<win_T>();
    }
    let mut frp_close: *mut frame_T = (*win).w_frame;
    let mut frp2: *mut frame_T = win_altframe(win, tp);
    let mut wp: *mut win_T = frame2win(frp2);
    if (*(*frp_close).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL {
        if !(*frp2).fr_win.is_null() && (*(*frp2).fr_win).w_onebuf_opt.wo_wfh != 0 {
            let mut frp: *mut frame_T = (*frp_close).fr_prev;
            let mut frp3: *mut frame_T = (*frp_close).fr_next;
            while !frp.is_null() || !frp3.is_null() {
                if !frp.is_null() {
                    if !frame_fixed_height(frp) {
                        frp2 = frp;
                        wp = frame2win(frp2);
                        break;
                    } else {
                        frp = (*frp).fr_prev;
                    }
                }
                if frp3.is_null() {
                    continue;
                }
                if !(*frp3).fr_win.is_null() && (*(*frp3).fr_win).w_onebuf_opt.wo_wfh == 0 {
                    frp2 = frp3;
                    wp = (*frp3).fr_win;
                    break;
                } else {
                    frp3 = (*frp3).fr_next;
                }
            }
        }
        *dirp = 'v' as ::core::ffi::c_int;
    } else {
        if !(*frp2).fr_win.is_null() && (*(*frp2).fr_win).w_onebuf_opt.wo_wfw != 0 {
            let mut frp_0: *mut frame_T = (*frp_close).fr_prev;
            let mut frp3_0: *mut frame_T = (*frp_close).fr_next;
            while !frp_0.is_null() || !frp3_0.is_null() {
                if !frp_0.is_null() {
                    if !frame_fixed_width(frp_0) {
                        frp2 = frp_0;
                        wp = frame2win(frp2);
                        break;
                    } else {
                        frp_0 = (*frp_0).fr_prev;
                    }
                }
                if frp3_0.is_null() {
                    continue;
                }
                if !(*frp3_0).fr_win.is_null() && (*(*frp3_0).fr_win).w_onebuf_opt.wo_wfw == 0 {
                    frp2 = frp3_0;
                    wp = (*frp3_0).fr_win;
                    break;
                } else {
                    frp3_0 = (*frp3_0).fr_next;
                }
            }
        }
        *dirp = 'h' as ::core::ffi::c_int;
    }
    '_c2rust_label_0: {
        if wp != win && frp2 != frp_close {
        } else {
            __assert_fail(
                b"wp != win && frp2 != frp_close\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                3561 as ::core::ffi::c_uint,
                b"win_T *winframe_find_altwin(win_T *, int *, tabpage_T *, frame_T **)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if !altfr.is_null() {
        *altfr = frp2;
    }
    return wp;
}
unsafe extern "C" fn frame_flatten(mut frp: *mut frame_T) {
    if !(*frp).fr_next.is_null() || !(*frp).fr_prev.is_null() {
        return;
    }
    (*(*frp).fr_parent).fr_layout = (*frp).fr_layout;
    (*(*frp).fr_parent).fr_child = (*frp).fr_child;
    let mut frp2: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
    frp2 = (*frp).fr_child;
    while !frp2.is_null() {
        (*frp2).fr_parent = (*frp).fr_parent;
        frp2 = (*frp2).fr_next;
    }
    (*(*frp).fr_parent).fr_win = (*frp).fr_win;
    if !(*frp).fr_win.is_null() {
        (*(*frp).fr_win).w_frame = (*frp).fr_parent;
    }
    frp2 = (*frp).fr_parent;
    if (*topframe.get()).fr_child == frp {
        (*topframe.get()).fr_child = frp2;
    }
    xfree(frp as *mut ::core::ffi::c_void);
    frp = (*frp2).fr_parent;
    if !frp.is_null()
        && (*frp).fr_layout as ::core::ffi::c_int == (*frp2).fr_layout as ::core::ffi::c_int
    {
        if (*frp).fr_child == frp2 {
            (*frp).fr_child = (*frp2).fr_child;
        }
        '_c2rust_label: {
            if !(*frp2).fr_child.is_null() {
            } else {
                __assert_fail(
                    b"frp2->fr_child\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3604 as ::core::ffi::c_uint,
                    b"void frame_flatten(frame_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        (*(*frp2).fr_child).fr_prev = (*frp2).fr_prev;
        if !(*frp2).fr_prev.is_null() {
            (*(*frp2).fr_prev).fr_next = (*frp2).fr_child;
        }
        let mut frp3: *mut frame_T = (*frp2).fr_child;
        loop {
            (*frp3).fr_parent = frp;
            if (*frp3).fr_next.is_null() {
                (*frp3).fr_next = (*frp2).fr_next;
                if !(*frp2).fr_next.is_null() {
                    (*(*frp2).fr_next).fr_prev = frp3;
                }
                break;
            } else {
                frp3 = (*frp3).fr_next;
            }
        }
        if (*topframe.get()).fr_child == frp2 {
            (*topframe.get()).fr_child = frp;
        }
        xfree(frp2 as *mut ::core::ffi::c_void);
    }
}
pub unsafe extern "C" fn winframe_restore(
    mut wp: *mut win_T,
    mut dir: ::core::ffi::c_int,
    mut unflat_altfr: *mut frame_T,
) {
    let mut frp: *mut frame_T = (*wp).w_frame;
    if !(*frp).fr_prev.is_null() {
        frame_append((*frp).fr_prev, frp);
    } else {
        frame_insert((*frp).fr_next, frp);
    }
    if (*wp).w_vsep_width == 0 as ::core::ffi::c_int
        && (*(*frp).fr_parent).fr_layout as ::core::ffi::c_int == FR_ROW
        && !(*frp).fr_prev.is_null()
    {
        frame_set_vsep((*frp).fr_prev, true_0 != 0);
    }
    if (*(*frp).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL && !(*frp).fr_prev.is_null() {
        if global_stl_height() == 0 as ::core::ffi::c_int
            && (*wp).w_status_height == 0 as ::core::ffi::c_int
        {
            frame_add_statusline((*frp).fr_prev);
        } else if global_stl_height() > 0 as ::core::ffi::c_int
            && (*wp).w_hsep_height == 0 as ::core::ffi::c_int
        {
            frame_add_hsep((*frp).fr_prev);
        }
    }
    if dir == 'v' as ::core::ffi::c_int {
        frame_new_height(
            unflat_altfr,
            (*unflat_altfr).fr_height - (*frp).fr_height,
            unflat_altfr == (*frp).fr_next,
            false_0 != 0,
            false_0 != 0,
        );
    } else if dir == 'h' as ::core::ffi::c_int {
        frame_new_width(
            unflat_altfr,
            (*unflat_altfr).fr_width - (*frp).fr_width,
            unflat_altfr == (*frp).fr_next,
            false_0 != 0,
        );
    }
    if unflat_altfr != (*frp).fr_prev {
        let topleft: *const win_T = frame2win((*frp).fr_parent);
        let mut row: ::core::ffi::c_int = (*topleft).w_winrow;
        let mut col: ::core::ffi::c_int = (*topleft).w_wincol;
        frame_comp_pos((*frp).fr_parent, &raw mut row, &raw mut col);
    }
}
unsafe extern "C" fn win_altframe(mut win: *mut win_T, mut tp: *mut tabpage_T) -> *mut frame_T {
    '_c2rust_label: {
        if tp.is_null() || tp != curtab.get() {
        } else {
            __assert_fail(
                b"tp == NULL || tp != curtab\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                3690 as ::core::ffi::c_uint,
                b"frame_T *win_altframe(win_T *, tabpage_T *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if one_window(win, tp) {
        return (*(*alt_tabpage()).tp_curwin).w_frame;
    }
    let mut frp: *mut frame_T = (*win).w_frame;
    if (*frp).fr_prev.is_null() {
        return (*frp).fr_next;
    }
    if (*frp).fr_next.is_null() {
        return (*frp).fr_prev;
    }
    let mut target_fr: *mut frame_T = (*frp).fr_next;
    let mut other_fr: *mut frame_T = (*frp).fr_prev;
    if !(*frp).fr_parent.is_null()
        && (*(*frp).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL
        && p_sb.get() != 0
    {
        target_fr = (*frp).fr_prev;
        other_fr = (*frp).fr_next;
    }
    if !(*frp).fr_parent.is_null()
        && (*(*frp).fr_parent).fr_layout as ::core::ffi::c_int == FR_ROW
        && p_spr.get() != 0
    {
        target_fr = (*frp).fr_prev;
        other_fr = (*frp).fr_next;
    }
    if !(*frp).fr_parent.is_null() && (*(*frp).fr_parent).fr_layout as ::core::ffi::c_int == FR_ROW
    {
        if frame_fixed_width(target_fr) as ::core::ffi::c_int != 0 && !frame_fixed_width(other_fr) {
            target_fr = other_fr;
        }
    } else if frame_fixed_height(target_fr) as ::core::ffi::c_int != 0
        && !frame_fixed_height(other_fr)
    {
        target_fr = other_fr;
    }
    return target_fr;
}
unsafe extern "C" fn alt_tabpage() -> *mut tabpage_T {
    if tcl_flags.get() & kOptTclFlagUselast as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && valid_tabpage(lastused_tabpage.get()) as ::core::ffi::c_int != 0
    {
        return lastused_tabpage.get();
    }
    let mut forward: bool = !(*curtab.get()).tp_next.is_null()
        && (tcl_flags.get() & kOptTclFlagLeft as ::core::ffi::c_int as ::core::ffi::c_uint
            == 0 as ::core::ffi::c_uint
            || curtab.get() == first_tabpage.get());
    let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    if forward {
        tp = (*curtab.get()).tp_next;
    } else {
        tp = first_tabpage.get();
        while (*tp).tp_next != curtab.get() {
            tp = (*tp).tp_next;
        }
    }
    return tp;
}
pub unsafe extern "C" fn frame2win(mut frp: *mut frame_T) -> *mut win_T {
    while (*frp).fr_win.is_null() {
        frp = (*frp).fr_child;
    }
    return (*frp).fr_win;
}
unsafe extern "C" fn frame_has_win(mut frp: *const frame_T, mut wp: *const win_T) -> bool {
    if (*frp).fr_layout as ::core::ffi::c_int == FR_LEAF {
        return (*frp).fr_win == wp as *mut win_T;
    }
    let mut p: *const frame_T = ::core::ptr::null::<frame_T>();
    p = (*frp).fr_child;
    while !p.is_null() {
        if frame_has_win(p, wp) {
            return true_0 != 0;
        }
        p = (*p).fr_next;
    }
    return false_0 != 0;
}
unsafe extern "C" fn is_bottom_win(mut wp: *mut win_T) -> bool {
    let mut frp: *mut frame_T = (*wp).w_frame;
    while !(*frp).fr_parent.is_null() {
        if (*(*frp).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL
            && !(*frp).fr_next.is_null()
        {
            return false_0 != 0;
        }
        frp = (*frp).fr_parent;
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn frame_new_height(
    mut topfrp: *mut frame_T,
    mut height: ::core::ffi::c_int,
    mut topfirst: bool,
    mut wfh: bool,
    mut set_ch: bool,
) {
    if (*topfrp).fr_parent.is_null() && set_ch as ::core::ffi::c_int != 0 {
        let mut new_ch: OptInt =
            if min_set_ch.get() > p_ch.get() + (*topfrp).fr_height as OptInt - height as OptInt {
                min_set_ch.get()
            } else {
                p_ch.get() + (*topfrp).fr_height as OptInt - height as OptInt
            };
        if new_ch != p_ch.get() {
            let save_ch: OptInt = min_set_ch.get();
            set_option_value(
                kOptCmdheight,
                OptVal {
                    type_0: kOptValTypeNumber,
                    data: OptValData { number: new_ch },
                },
                0 as ::core::ffi::c_int,
            );
            min_set_ch.set(save_ch);
        }
        height = (if (Rows.get() as OptInt
            - p_ch.get()
            - tabline_height() as OptInt
            - global_stl_height() as OptInt)
            < height as OptInt
        {
            Rows.get() as OptInt
                - p_ch.get()
                - tabline_height() as OptInt
                - global_stl_height() as OptInt
        } else {
            height as OptInt
        }) as ::core::ffi::c_int;
    }
    if !(*topfrp).fr_win.is_null() {
        let mut wp: *mut win_T = (*topfrp).fr_win;
        if is_bottom_win(wp) {
            (*wp).w_hsep_height = 0 as ::core::ffi::c_int;
        }
        win_new_height(wp, height - (*wp).w_hsep_height - (*wp).w_status_height);
    } else if (*topfrp).fr_layout as ::core::ffi::c_int == FR_ROW {
        let mut frp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
        loop {
            frp = (*topfrp).fr_child;
            while !frp.is_null() {
                frame_new_height(frp, height, topfirst, wfh, set_ch);
                if (*frp).fr_height > height {
                    height = (*frp).fr_height;
                    break;
                } else {
                    frp = (*frp).fr_next;
                }
            }
            if frp.is_null() {
                break;
            }
        }
    } else {
        let mut frp_0: *mut frame_T = (*topfrp).fr_child;
        if wfh {
            while frame_fixed_height(frp_0) {
                frp_0 = (*frp_0).fr_next;
                if frp_0.is_null() {
                    return;
                }
            }
        }
        if !topfirst {
            while !(*frp_0).fr_next.is_null() {
                frp_0 = (*frp_0).fr_next;
            }
            if wfh {
                while frame_fixed_height(frp_0) {
                    frp_0 = (*frp_0).fr_prev;
                }
            }
        }
        let mut extra_lines: ::core::ffi::c_int = height - (*topfrp).fr_height;
        if extra_lines < 0 as ::core::ffi::c_int {
            while !frp_0.is_null() {
                let mut h: ::core::ffi::c_int =
                    frame_minheight(frp_0, ::core::ptr::null_mut::<win_T>());
                if (*frp_0).fr_height + extra_lines < h {
                    extra_lines += (*frp_0).fr_height - h;
                    frame_new_height(frp_0, h, topfirst, wfh, set_ch);
                    if topfirst {
                        loop {
                            frp_0 = (*frp_0).fr_next;
                            if !(wfh as ::core::ffi::c_int != 0
                                && !frp_0.is_null()
                                && frame_fixed_height(frp_0) as ::core::ffi::c_int != 0)
                            {
                                break;
                            }
                        }
                    } else {
                        loop {
                            frp_0 = (*frp_0).fr_prev;
                            if !(wfh as ::core::ffi::c_int != 0
                                && !frp_0.is_null()
                                && frame_fixed_height(frp_0) as ::core::ffi::c_int != 0)
                            {
                                break;
                            }
                        }
                    }
                    if frp_0.is_null() {
                        height -= extra_lines;
                    }
                } else {
                    frame_new_height(
                        frp_0,
                        (*frp_0).fr_height + extra_lines,
                        topfirst,
                        wfh,
                        set_ch,
                    );
                    break;
                }
            }
        } else if extra_lines > 0 as ::core::ffi::c_int {
            frame_new_height(
                frp_0,
                (*frp_0).fr_height + extra_lines,
                topfirst,
                wfh,
                set_ch,
            );
        }
    }
    (*topfrp).fr_height = height;
}
unsafe extern "C" fn frame_fixed_height(mut frp: *mut frame_T) -> bool {
    if !(*frp).fr_win.is_null() {
        return (*(*frp).fr_win).w_onebuf_opt.wo_wfh != 0;
    }
    if (*frp).fr_layout as ::core::ffi::c_int == FR_ROW {
        frp = (*frp).fr_child;
        while !frp.is_null() {
            if frame_fixed_height(frp) {
                return true_0 != 0;
            }
            frp = (*frp).fr_next;
        }
        return false_0 != 0;
    }
    frp = (*frp).fr_child;
    while !frp.is_null() {
        if !frame_fixed_height(frp) {
            return false_0 != 0;
        }
        frp = (*frp).fr_next;
    }
    return true_0 != 0;
}
unsafe extern "C" fn frame_fixed_width(mut frp: *mut frame_T) -> bool {
    if !(*frp).fr_win.is_null() {
        return (*(*frp).fr_win).w_onebuf_opt.wo_wfw != 0;
    }
    if (*frp).fr_layout as ::core::ffi::c_int == FR_COL {
        frp = (*frp).fr_child;
        while !frp.is_null() {
            if frame_fixed_width(frp) {
                return true_0 != 0;
            }
            frp = (*frp).fr_next;
        }
        return false_0 != 0;
    }
    frp = (*frp).fr_child;
    while !frp.is_null() {
        if !frame_fixed_width(frp) {
            return false_0 != 0;
        }
        frp = (*frp).fr_next;
    }
    return true_0 != 0;
}
unsafe extern "C" fn frame_add_statusline(mut frp: *mut frame_T) {
    if (*frp).fr_layout as ::core::ffi::c_int == FR_LEAF {
        let mut wp: *mut win_T = (*frp).fr_win;
        (*wp).w_status_height = STATUS_HEIGHT as ::core::ffi::c_int;
    } else if (*frp).fr_layout as ::core::ffi::c_int == FR_ROW {
        frp = (*frp).fr_child;
        while !frp.is_null() {
            frame_add_statusline(frp);
            frp = (*frp).fr_next;
        }
    } else {
        '_c2rust_label: {
            if (*frp).fr_layout as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"frp->fr_layout == FR_COL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3986 as ::core::ffi::c_uint,
                    b"void frame_add_statusline(frame_T *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        frp = (*frp).fr_child;
        while !(*frp).fr_next.is_null() {
            frp = (*frp).fr_next;
        }
        frame_add_statusline(frp);
    };
}
unsafe extern "C" fn frame_new_width(
    mut topfrp: *mut frame_T,
    mut width: ::core::ffi::c_int,
    mut leftfirst: bool,
    mut wfw: bool,
) {
    if (*topfrp).fr_layout as ::core::ffi::c_int == FR_LEAF {
        let mut wp: *mut win_T = (*topfrp).fr_win;
        let mut frp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
        frp = topfrp;
        while !(*frp).fr_parent.is_null() {
            if (*(*frp).fr_parent).fr_layout as ::core::ffi::c_int == FR_ROW
                && !(*frp).fr_next.is_null()
            {
                break;
            }
            frp = (*frp).fr_parent;
        }
        if (*frp).fr_parent.is_null() {
            (*wp).w_vsep_width = 0 as ::core::ffi::c_int;
        }
        win_new_width(wp, width - (*wp).w_vsep_width);
    } else if (*topfrp).fr_layout as ::core::ffi::c_int == FR_COL {
        let mut frp_0: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
        loop {
            frp_0 = (*topfrp).fr_child;
            while !frp_0.is_null() {
                frame_new_width(frp_0, width, leftfirst, wfw);
                if (*frp_0).fr_width > width {
                    width = (*frp_0).fr_width;
                    break;
                } else {
                    frp_0 = (*frp_0).fr_next;
                }
            }
            if frp_0.is_null() {
                break;
            }
        }
    } else {
        let mut frp_1: *mut frame_T = (*topfrp).fr_child;
        if wfw {
            while frame_fixed_width(frp_1) {
                frp_1 = (*frp_1).fr_next;
                if frp_1.is_null() {
                    return;
                }
            }
        }
        if !leftfirst {
            while !(*frp_1).fr_next.is_null() {
                frp_1 = (*frp_1).fr_next;
            }
            if wfw {
                while frame_fixed_width(frp_1) {
                    frp_1 = (*frp_1).fr_prev;
                }
            }
        }
        let mut extra_cols: ::core::ffi::c_int = width - (*topfrp).fr_width;
        if extra_cols < 0 as ::core::ffi::c_int {
            while !frp_1.is_null() {
                let mut w: ::core::ffi::c_int =
                    frame_minwidth(frp_1, ::core::ptr::null_mut::<win_T>());
                if (*frp_1).fr_width + extra_cols < w {
                    extra_cols += (*frp_1).fr_width - w;
                    frame_new_width(frp_1, w, leftfirst, wfw);
                    if leftfirst {
                        loop {
                            frp_1 = (*frp_1).fr_next;
                            if !(wfw as ::core::ffi::c_int != 0
                                && !frp_1.is_null()
                                && frame_fixed_width(frp_1) as ::core::ffi::c_int != 0)
                            {
                                break;
                            }
                        }
                    } else {
                        loop {
                            frp_1 = (*frp_1).fr_prev;
                            if !(wfw as ::core::ffi::c_int != 0
                                && !frp_1.is_null()
                                && frame_fixed_width(frp_1) as ::core::ffi::c_int != 0)
                            {
                                break;
                            }
                        }
                    }
                    if frp_1.is_null() {
                        width -= extra_cols;
                    }
                } else {
                    frame_new_width(frp_1, (*frp_1).fr_width + extra_cols, leftfirst, wfw);
                    break;
                }
            }
        } else if extra_cols > 0 as ::core::ffi::c_int {
            frame_new_width(frp_1, (*frp_1).fr_width + extra_cols, leftfirst, wfw);
        }
    }
    (*topfrp).fr_width = width;
}
unsafe extern "C" fn frame_set_vsep(mut frp: *const frame_T, mut add: bool) {
    if (*frp).fr_layout as ::core::ffi::c_int == FR_LEAF {
        let mut wp: *mut win_T = (*frp).fr_win;
        if add as ::core::ffi::c_int != 0 && (*wp).w_vsep_width == 0 as ::core::ffi::c_int {
            if (*wp).w_width > 0 as ::core::ffi::c_int {
                win_new_width(wp, (*wp).w_width - 1 as ::core::ffi::c_int);
            }
            (*wp).w_vsep_width = 1 as ::core::ffi::c_int;
        } else if !add && (*wp).w_vsep_width == 1 as ::core::ffi::c_int {
            win_new_width(wp, (*wp).w_width + 1 as ::core::ffi::c_int);
            (*wp).w_vsep_width = 0 as ::core::ffi::c_int;
        }
    } else if (*frp).fr_layout as ::core::ffi::c_int == FR_COL {
        frp = (*frp).fr_child;
        while !frp.is_null() {
            frame_set_vsep(frp, add);
            frp = (*frp).fr_next;
        }
    } else {
        '_c2rust_label: {
            if (*frp).fr_layout as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"frp->fr_layout == FR_ROW\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    4112 as ::core::ffi::c_uint,
                    b"void frame_set_vsep(const frame_T *, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        frp = (*frp).fr_child;
        while !(*frp).fr_next.is_null() {
            frp = (*frp).fr_next;
        }
        frame_set_vsep(frp, add);
    };
}
unsafe extern "C" fn frame_add_hsep(mut frp: *const frame_T) {
    if (*frp).fr_layout as ::core::ffi::c_int == FR_LEAF {
        let mut wp: *mut win_T = (*frp).fr_win;
        (*wp).w_hsep_height = 1 as ::core::ffi::c_int;
    } else if (*frp).fr_layout as ::core::ffi::c_int == FR_ROW {
        frp = (*frp).fr_child;
        while !frp.is_null() {
            frame_add_hsep(frp);
            frp = (*frp).fr_next;
        }
    } else {
        '_c2rust_label: {
            if (*frp).fr_layout as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"frp->fr_layout == FR_COL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    4136 as ::core::ffi::c_uint,
                    b"void frame_add_hsep(const frame_T *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        frp = (*frp).fr_child;
        while !(*frp).fr_next.is_null() {
            frp = (*frp).fr_next;
        }
        frame_add_hsep(frp);
    };
}
unsafe extern "C" fn frame_fix_width(mut wp: *mut win_T) {
    (*(*wp).w_frame).fr_width = (*wp).w_width + (*wp).w_vsep_width;
}
unsafe extern "C" fn frame_fix_height(mut wp: *mut win_T) {
    (*(*wp).w_frame).fr_height = (*wp).w_height + (*wp).w_hsep_height + (*wp).w_status_height;
}
unsafe extern "C" fn frame_minheight(
    mut topfrp: *mut frame_T,
    mut next_curwin: *mut win_T,
) -> ::core::ffi::c_int {
    let mut m: ::core::ffi::c_int = 0;
    if !(*topfrp).fr_win.is_null() {
        let mut extra_height: ::core::ffi::c_int = (*(*topfrp).fr_win).w_winbar_height
            + (*(*topfrp).fr_win).w_hsep_height
            + (*(*topfrp).fr_win).w_status_height;
        if (*topfrp).fr_win == next_curwin {
            m = p_wh.get() as ::core::ffi::c_int + extra_height;
        } else {
            m = p_wmh.get() as ::core::ffi::c_int + extra_height;
            if (*topfrp).fr_win == curwin.get() && next_curwin.is_null() {
                if p_wmh.get() == 0 as OptInt {
                    m += 1;
                }
            }
        }
    } else if (*topfrp).fr_layout as ::core::ffi::c_int == FR_ROW {
        m = 0 as ::core::ffi::c_int;
        let mut frp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
        frp = (*topfrp).fr_child;
        while !frp.is_null() {
            let mut n: ::core::ffi::c_int = frame_minheight(frp, next_curwin);
            if n > m {
                m = n;
            }
            frp = (*frp).fr_next;
        }
    } else {
        m = 0 as ::core::ffi::c_int;
        let mut frp_0: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
        frp_0 = (*topfrp).fr_child;
        while !frp_0.is_null() {
            m += frame_minheight(frp_0, next_curwin);
            frp_0 = (*frp_0).fr_next;
        }
    }
    return m;
}
unsafe extern "C" fn frame_minwidth(
    mut topfrp: *mut frame_T,
    mut next_curwin: *mut win_T,
) -> ::core::ffi::c_int {
    let mut m: ::core::ffi::c_int = 0;
    if !(*topfrp).fr_win.is_null() {
        if (*topfrp).fr_win == next_curwin {
            m = p_wiw.get() as ::core::ffi::c_int + (*(*topfrp).fr_win).w_vsep_width;
        } else {
            m = p_wmw.get() as ::core::ffi::c_int + (*(*topfrp).fr_win).w_vsep_width;
            if p_wmw.get() == 0 as OptInt
                && (*topfrp).fr_win == curwin.get()
                && next_curwin.is_null()
            {
                m += 1;
            }
        }
    } else if (*topfrp).fr_layout as ::core::ffi::c_int == FR_COL {
        m = 0 as ::core::ffi::c_int;
        let mut frp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
        frp = (*topfrp).fr_child;
        while !frp.is_null() {
            let mut n: ::core::ffi::c_int = frame_minwidth(frp, next_curwin);
            m = if m > n { m } else { n };
            frp = (*frp).fr_next;
        }
    } else {
        m = 0 as ::core::ffi::c_int;
        let mut frp_0: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
        frp_0 = (*topfrp).fr_child;
        while !frp_0.is_null() {
            m += frame_minwidth(frp_0, next_curwin);
            frp_0 = (*frp_0).fr_next;
        }
    }
    return m;
}
pub unsafe extern "C" fn close_others(
    mut message: ::core::ffi::c_int,
    mut forceit: ::core::ffi::c_int,
) {
    let old_curwin: *mut win_T = curwin.get();
    if (*curwin.get()).w_floating {
        if message != 0 && !autocmd_busy.get() {
            emsg(&raw const e_floatonly as *const ::core::ffi::c_char);
        }
        return;
    }
    if one_window(firstwin.get(), ::core::ptr::null_mut::<tabpage_T>()) as ::core::ffi::c_int != 0
        && !(*lastwin.get()).w_floating
    {
        if message != 0 && !autocmd_busy.get() {
            msg(gettext(m_onlyone.get()), 0 as ::core::ffi::c_int);
        }
        return;
    }
    let mut nextwp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut wp: *mut win_T = firstwin.get();
    while win_valid(wp) {
        nextwp = (*wp).w_next;
        if old_curwin != curwin.get() && win_valid(old_curwin) as ::core::ffi::c_int != 0 {
            curwin.set(old_curwin);
            curbuf.set((*curwin.get()).w_buffer);
        }
        's_52: {
            if wp != curwin.get() {
                if !buf_valid((*wp).w_buffer) && win_valid(wp) as ::core::ffi::c_int != 0 {
                    (*wp).w_buffer = ::core::ptr::null_mut::<buf_T>();
                    win_close(wp, false_0 != 0, false_0 != 0);
                } else {
                    let mut r: ::core::ffi::c_int =
                        can_abandon((*wp).w_buffer, forceit != 0) as ::core::ffi::c_int;
                    if !win_valid(wp) {
                        nextwp = firstwin.get();
                    } else {
                        if r == 0 {
                            if message != 0
                                && (p_confirm.get() != 0
                                    || (*cmdmod.ptr()).cmod_flags
                                        & CMOD_CONFIRM as ::core::ffi::c_int
                                        != 0)
                                && p_write.get() != 0
                            {
                                dialog_changed((*wp).w_buffer, false_0 != 0);
                                if !win_valid(wp) {
                                    nextwp = firstwin.get();
                                    break 's_52;
                                }
                            }
                            if bufIsChanged((*wp).w_buffer) {
                                break 's_52;
                            }
                        }
                        win_close(
                            wp,
                            !buf_hide((*wp).w_buffer) && !bufIsChanged((*wp).w_buffer),
                            false_0 != 0,
                        );
                    }
                }
            }
        }
        wp = nextwp;
    }
    if message != 0 && !(firstwin.get() == lastwin.get()) {
        emsg(gettext(
            b"E445: Other window contains changes\0".as_ptr() as *const ::core::ffi::c_char
        ));
    }
}
pub unsafe extern "C" fn unuse_tabpage(mut tp: *mut tabpage_T) {
    (*tp).tp_topframe = topframe.get();
    (*tp).tp_firstwin = firstwin.get();
    (*tp).tp_lastwin = lastwin.get();
    (*tp).tp_curwin = curwin.get();
}
static command_frame_height: GlobalCell<bool> = GlobalCell::new(true_0 != 0);
pub unsafe extern "C" fn use_tabpage(mut tp: *mut tabpage_T) {
    curtab.set(tp);
    topframe.set((*curtab.get()).tp_topframe);
    firstwin.set((*curtab.get()).tp_firstwin);
    lastwin.set((*curtab.get()).tp_lastwin);
    curwin.set((*curtab.get()).tp_curwin);
}
pub unsafe extern "C" fn win_alloc_first() {
    if win_alloc_firstwin(::core::ptr::null_mut::<win_T>()) == FAIL {
        abort();
    }
    first_tabpage.set(alloc_tabpage());
    curtab.set(first_tabpage.get());
    unuse_tabpage(first_tabpage.get());
}
pub unsafe extern "C" fn win_alloc_aucmd_win(mut idx: ::core::ffi::c_int) {
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut fconfig: WinConfig = WinConfig {
        window: 0,
        bufpos: lpos_T {
            lnum: -1 as linenr_T,
            col: 0 as colnr_T,
        },
        height: 0 as ::core::ffi::c_int,
        width: 0 as ::core::ffi::c_int,
        row: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
        col: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
        anchor: 0 as FloatAnchor,
        relative: kFloatRelativeEditor,
        external: false_0 != 0,
        focusable: true_0 != 0,
        mouse: true_0 != 0,
        split: kWinSplitLeft,
        zindex: kZIndexFloatDefault as ::core::ffi::c_int,
        style: kWinStyleUnused,
        border: false,
        shadow: false,
        border_chars: [[0; 32]; 8],
        border_hl_ids: [0; 8],
        border_attr: [0; 8],
        title: false,
        title_pos: kAlignLeft,
        title_chunks: VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        },
        title_width: 0,
        footer: false,
        footer_pos: kAlignLeft,
        footer_chunks: VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        },
        footer_width: 0,
        noautocmd: false_0 != 0,
        fixed: false_0 != 0,
        hide: false_0 != 0,
        _cmdline_offset: INT_MAX,
    };
    fconfig.width = Columns.get();
    fconfig.height = 5 as ::core::ffi::c_int;
    fconfig.focusable = false_0 != 0;
    fconfig.mouse = false_0 != 0;
    (*(*aucmd_win_vec.ptr()).items.offset(idx as isize)).auc_win = win_new_float(
        ::core::ptr::null_mut::<win_T>(),
        true_0 != 0,
        fconfig,
        &raw mut err,
    );
    (*(*(*(*aucmd_win_vec.ptr()).items.offset(idx as isize)).auc_win).w_buffer).b_nwindows -= 1;
    (*(*(*aucmd_win_vec.ptr()).items.offset(idx as isize)).auc_win)
        .w_onebuf_opt
        .wo_scb = false_0;
    (*(*(*aucmd_win_vec.ptr()).items.offset(idx as isize)).auc_win)
        .w_onebuf_opt
        .wo_crb = false_0;
}
unsafe extern "C" fn win_alloc_firstwin(mut oldwin: *mut win_T) -> ::core::ffi::c_int {
    curwin.set(win_alloc(::core::ptr::null_mut::<win_T>(), false_0 != 0));
    if oldwin.is_null() {
        curbuf.set(buflist_new(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            1 as linenr_T,
            BLN_LISTED as ::core::ffi::c_int,
        ));
        if (*curbuf.ptr()).is_null() {
            return FAIL;
        }
        (*curwin.get()).w_buffer = curbuf.get();
        (*curwin.get()).w_s = &raw mut (*curbuf.get()).b_s;
        (*curbuf.get()).b_nwindows = 1 as ::core::ffi::c_int;
        (*curwin.get()).w_alist = global_alist.ptr();
        curwin_init();
    } else {
        win_init(curwin.get(), oldwin, 0 as ::core::ffi::c_int);
        (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
        (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
    }
    new_frame(curwin.get());
    topframe.set((*curwin.get()).w_frame);
    (*topframe.get()).fr_width = Columns.get();
    (*topframe.get()).fr_height =
        Rows.get() - p_ch.get() as ::core::ffi::c_int - global_stl_height();
    return OK;
}
unsafe extern "C" fn new_frame(mut wp: *mut win_T) {
    let mut frp: *mut frame_T =
        xcalloc(1 as size_t, ::core::mem::size_of::<frame_T>()) as *mut frame_T;
    (*wp).w_frame = frp;
    (*frp).fr_layout = FR_LEAF as ::core::ffi::c_char;
    (*frp).fr_win = wp;
}
pub unsafe extern "C" fn win_init_size() {
    (*firstwin.get()).w_height = (Rows.get() as OptInt
        - p_ch.get()
        - tabline_height() as OptInt
        - global_stl_height() as OptInt) as ::core::ffi::c_int;
    (*firstwin.get()).w_prev_height = (Rows.get() as OptInt
        - p_ch.get()
        - tabline_height() as OptInt
        - global_stl_height() as OptInt)
        as ::core::ffi::c_int;
    (*firstwin.get()).w_view_height =
        (*firstwin.get()).w_height - (*firstwin.get()).w_winbar_height;
    (*firstwin.get()).w_height_outer = (*firstwin.get()).w_height;
    (*firstwin.get()).w_winrow_off = (*firstwin.get()).w_winbar_height;
    (*topframe.get()).fr_height = (Rows.get() as OptInt
        - p_ch.get()
        - tabline_height() as OptInt
        - global_stl_height() as OptInt) as ::core::ffi::c_int;
    (*firstwin.get()).w_width = Columns.get();
    (*firstwin.get()).w_view_width = (*firstwin.get()).w_width;
    (*firstwin.get()).w_width_outer = (*firstwin.get()).w_width;
    (*topframe.get()).fr_width = Columns.get();
}
unsafe extern "C" fn alloc_tabpage() -> *mut tabpage_T {
    static last_tp_handle: GlobalCell<::core::ffi::c_int> =
        GlobalCell::new(0 as ::core::ffi::c_int);
    let mut tp: *mut tabpage_T =
        xcalloc(1 as size_t, ::core::mem::size_of::<tabpage_T>()) as *mut tabpage_T;
    (*last_tp_handle.ptr()) += 1;
    (*tp).handle = last_tp_handle.get() as handle_T;
    map_put_int_ptr_t(
        tabpage_handles.ptr(),
        (*tp).handle as ::core::ffi::c_int,
        tp as ptr_t,
    );
    (*tp).tp_vars = tv_dict_alloc();
    init_var_dict((*tp).tp_vars, &raw mut (*tp).tp_winvar, VAR_SCOPE);
    (*tp).tp_diff_invalid = true_0;
    (*tp).tp_ch_used = p_ch.get();
    return tp;
}
pub unsafe extern "C" fn free_tabpage(mut tp: *mut tabpage_T) {
    map_del_int_ptr_t(
        tabpage_handles.ptr(),
        (*tp).handle as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
    );
    diff_clear(tp);
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while idx < SNAP_COUNT {
        clear_snapshot(tp, idx);
        idx += 1;
    }
    vars_clear(&raw mut (*(*tp).tp_vars).dv_hashtab);
    hash_init(&raw mut (*(*tp).tp_vars).dv_hashtab);
    unref_var_dict((*tp).tp_vars);
    if tp == lastused_tabpage.get() {
        lastused_tabpage.set(::core::ptr::null_mut::<tabpage_T>());
    }
    xfree((*tp).tp_localdir as *mut ::core::ffi::c_void);
    xfree((*tp).tp_prevdir as *mut ::core::ffi::c_void);
    xfree(tp as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn win_new_tabpage(
    mut after: ::core::ffi::c_int,
    mut filename: *mut ::core::ffi::c_char,
    mut enter: bool,
    mut first: *mut *mut win_T,
) -> *mut tabpage_T {
    let mut old_curtab: *mut tabpage_T = curtab.get();
    if enter as ::core::ffi::c_int != 0 && cmdwin_type.get() != 0 as ::core::ffi::c_int {
        emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
        return ::core::ptr::null_mut::<tabpage_T>();
    }
    if window_layout_locked(CMD_tabnew) {
        return ::core::ptr::null_mut::<tabpage_T>();
    }
    let mut newtp: *mut tabpage_T = alloc_tabpage();
    if enter {
        if leave_tabpage(curbuf.get(), true_0 != 0) == FAIL {
            xfree(newtp as *mut ::core::ffi::c_void);
            return ::core::ptr::null_mut::<tabpage_T>();
        }
    } else {
        unuse_tabpage(curtab.get());
        (*curtab.get()).tp_old_Rows_avail = (Rows.get() as OptInt
            - p_ch.get()
            - tabline_height() as OptInt
            - global_stl_height() as OptInt) as int64_t;
        firstwin.set(::core::ptr::null_mut::<win_T>());
        lastwin.set(::core::ptr::null_mut::<win_T>());
    }
    (*newtp).tp_localdir = if !(*old_curtab).tp_localdir.is_null() {
        xstrdup((*old_curtab).tp_localdir)
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
    curtab.set(newtp);
    let result: ::core::ffi::c_int = win_alloc_firstwin((*old_curtab).tp_curwin);
    '_c2rust_label: {
        if result == 1 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"result == OK\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                4520 as ::core::ffi::c_uint,
                b"tabpage_T *win_new_tabpage(int, char *, _Bool, win_T **)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if !first.is_null() {
        *first = curwin.get();
    }
    if after == 1 as ::core::ffi::c_int {
        (*newtp).tp_next = first_tabpage.get();
        first_tabpage.set(newtp);
    } else {
        let mut tp: *mut tabpage_T = old_curtab;
        if after > 0 as ::core::ffi::c_int {
            let mut n: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
            tp = first_tabpage.get();
            while !(*tp).tp_next.is_null() && n < after {
                n += 1;
                tp = (*tp).tp_next;
            }
        }
        (*newtp).tp_next = (*tp).tp_next;
        (*tp).tp_next = newtp;
    }
    (*newtp).tp_curwin = curwin.get();
    (*newtp).tp_lastwin = (*newtp).tp_curwin;
    (*newtp).tp_firstwin = (*newtp).tp_lastwin;
    win_init_size();
    (*firstwin.get()).w_winrow = tabline_height();
    (*firstwin.get()).w_prev_winrow = (*firstwin.get()).w_winrow;
    win_comp_scroll(curwin.get());
    (*newtp).tp_topframe = topframe.get();
    last_status(false_0 != 0);
    if !(*curbuf.get()).terminal.is_null() {
        terminal_check_size((*curbuf.get()).terminal);
    }
    if enter {
        redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
        tabpage_check_windows(old_curtab);
        lastused_tabpage.set(old_curtab);
        entering_window(curwin.get());
        apply_autocmds(
            EVENT_WINNEW,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        apply_autocmds(
            EVENT_WINENTER,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        apply_autocmds(EVENT_TABNEW, filename, filename, false_0 != 0, curbuf.get());
        apply_autocmds(
            EVENT_TABENTER,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
    } else {
        unuse_tabpage(curtab.get());
        use_tabpage(old_curtab);
        redraw_tabline.set(true_0 != 0);
        if (*curtab.get()).tp_old_Rows_avail
            != Rows.get() as OptInt
                - p_ch.get()
                - tabline_height() as OptInt
                - global_stl_height() as OptInt
        {
            win_new_screen_rows();
        }
        let mut switchwin: switchwin_T = switchwin_T {
            sw_curwin: ::core::ptr::null_mut::<win_T>(),
            sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
            sw_same_win: false,
            sw_visual_active: false,
        };
        let sw_result: ::core::ffi::c_int =
            switch_win_noblock(&raw mut switchwin, (*newtp).tp_curwin, newtp, true_0 != 0);
        '_c2rust_label_0: {
            if sw_result == 1 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"sw_result == OK\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    4582 as ::core::ffi::c_uint,
                    b"tabpage_T *win_new_tabpage(int, char *, _Bool, win_T **)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        apply_autocmds(
            EVENT_WINNEW,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        apply_autocmds(EVENT_TABNEW, filename, filename, false_0 != 0, curbuf.get());
        restore_win_noblock(&raw mut switchwin, true_0 != 0);
    }
    return newtp;
}
unsafe extern "C" fn may_open_tabpage() -> ::core::ffi::c_int {
    let mut n: ::core::ffi::c_int = if (*cmdmod.ptr()).cmod_tab == 0 as ::core::ffi::c_int {
        postponed_split_tab.get()
    } else {
        (*cmdmod.ptr()).cmod_tab
    };
    if n == 0 as ::core::ffi::c_int {
        return FAIL;
    }
    (*cmdmod.ptr()).cmod_tab = 0 as ::core::ffi::c_int;
    postponed_split_tab.set(0 as ::core::ffi::c_int);
    let mut status: ::core::ffi::c_int = if !win_new_tabpage(
        n,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        true_0 != 0,
        ::core::ptr::null_mut::<*mut win_T>(),
    )
    .is_null()
    {
        OK
    } else {
        FAIL
    };
    if status == OK {
        apply_autocmds(
            EVENT_TABNEWENTERED,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
    }
    return status;
}
pub unsafe extern "C" fn make_tabpages(mut maxcount: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut count: ::core::ffi::c_int = maxcount;
    count = if count < p_tpm.get() as ::core::ffi::c_int {
        count
    } else {
        p_tpm.get() as ::core::ffi::c_int
    };
    block_autocmds();
    let mut todo: ::core::ffi::c_int = 0;
    todo = count - 1 as ::core::ffi::c_int;
    while todo > 0 as ::core::ffi::c_int {
        if win_new_tabpage(
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            true_0 != 0,
            ::core::ptr::null_mut::<*mut win_T>(),
        )
        .is_null()
        {
            break;
        }
        todo -= 1;
    }
    unblock_autocmds();
    return count - todo;
}
pub unsafe extern "C" fn valid_tabpage(mut tpc: *mut tabpage_T) -> bool {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        if tp == tpc {
            return true_0 != 0;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn valid_tabpage_win(mut tpc: *mut tabpage_T) -> ::core::ffi::c_int {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        if tp == tpc {
            let mut wp: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                if win_valid_any_tab(wp) {
                    return true_0;
                }
                wp = (*wp).w_next;
            }
            return false_0;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    return false_0;
}
pub unsafe extern "C" fn close_tabpage(mut tab: *mut tabpage_T) {
    let mut ptp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    if tab == first_tabpage.get() {
        first_tabpage.set((*tab).tp_next);
        ptp = first_tabpage.get();
    } else {
        ptp = first_tabpage.get();
        while !ptp.is_null() && (*ptp).tp_next != tab {
            ptp = (*ptp).tp_next;
        }
        '_c2rust_label: {
            if !ptp.is_null() {
            } else {
                __assert_fail(
                    b"ptp != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    4684 as ::core::ffi::c_uint,
                    b"void close_tabpage(tabpage_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        (*ptp).tp_next = (*tab).tp_next;
    }
    goto_tabpage_tp(ptp, false_0 != 0, false_0 != 0);
    free_tabpage(tab);
}
pub unsafe extern "C" fn find_tabpage(mut n: ::core::ffi::c_int) -> *mut tabpage_T {
    let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    if n == 0 as ::core::ffi::c_int {
        return curtab.get();
    }
    tp = first_tabpage.get();
    while !tp.is_null() && i != n {
        i += 1;
        tp = (*tp).tp_next;
    }
    return tp;
}
pub unsafe extern "C" fn tabpage_index(mut ftp: *mut tabpage_T) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    tp = first_tabpage.get();
    while !tp.is_null() && tp != ftp {
        i += 1;
        tp = (*tp).tp_next;
    }
    return i;
}
unsafe extern "C" fn leave_tabpage(
    mut new_curbuf: *mut buf_T,
    mut trigger_leave_autocmds: bool,
) -> ::core::ffi::c_int {
    let mut tp: *mut tabpage_T = curtab.get();
    leaving_window(curwin.get());
    reset_VIsual_and_resel();
    if trigger_leave_autocmds {
        if new_curbuf != curbuf.get() {
            apply_autocmds(
                EVENT_BUFLEAVE,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
            if curtab.get() != tp {
                return FAIL;
            }
        }
        apply_autocmds(
            EVENT_WINLEAVE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        if curtab.get() != tp {
            return FAIL;
        }
        apply_autocmds(
            EVENT_TABLEAVE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        if curtab.get() != tp {
            return FAIL;
        }
    }
    reset_dragwin();
    (*tp).tp_curwin = curwin.get();
    (*tp).tp_prevwin = prevwin.get();
    (*tp).tp_firstwin = firstwin.get();
    (*tp).tp_lastwin = lastwin.get();
    (*tp).tp_old_Rows_avail = (Rows.get() as OptInt
        - p_ch.get()
        - tabline_height() as OptInt
        - global_stl_height() as OptInt) as int64_t;
    if (*tp).tp_old_Columns != -1 as int64_t {
        (*tp).tp_old_Columns = Columns.get() as int64_t;
    }
    firstwin.set(::core::ptr::null_mut::<win_T>());
    lastwin.set(::core::ptr::null_mut::<win_T>());
    return OK;
}
unsafe extern "C" fn enter_tabpage(
    mut tp: *mut tabpage_T,
    mut old_curbuf: *mut buf_T,
    mut trigger_enter_autocmds: bool,
    mut trigger_leave_autocmds: bool,
) {
    let mut old_off: ::core::ffi::c_int = (*(*tp).tp_firstwin).w_winrow;
    let mut next_prevwin: *mut win_T = (*tp).tp_prevwin;
    let mut old_curtab: *mut tabpage_T = curtab.get();
    use_tabpage(tp);
    if old_curtab != curtab.get() {
        tabpage_check_windows(old_curtab);
        if p_ch.get() != (*curtab.get()).tp_ch_used {
            let mut new_ch: OptInt = (*curtab.get()).tp_ch_used;
            (*curtab.get()).tp_ch_used = p_ch.get();
            command_frame_height.set(false_0 != 0);
            set_option_value(
                kOptCmdheight,
                OptVal {
                    type_0: kOptValTypeNumber,
                    data: OptValData { number: new_ch },
                },
                0 as ::core::ffi::c_int,
            );
            command_frame_height.set(true_0 != 0);
        }
    }
    win_enter_ext(
        (*tp).tp_curwin,
        WEE_CURWIN_INVALID as ::core::ffi::c_int
            | (if trigger_enter_autocmds as ::core::ffi::c_int != 0 {
                WEE_TRIGGER_ENTER_AUTOCMDS as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            })
            | (if trigger_leave_autocmds as ::core::ffi::c_int != 0 {
                WEE_TRIGGER_LEAVE_AUTOCMDS as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }),
    );
    prevwin.set(next_prevwin);
    last_status(false_0 != 0);
    win_float_update_statusline();
    win_comp_pos();
    diff_need_scrollbind.set(true_0 != 0);
    reset_dragwin();
    if (*curtab.get()).tp_old_Rows_avail
        != Rows.get() as OptInt
            - p_ch.get()
            - tabline_height() as OptInt
            - global_stl_height() as OptInt
        || old_off != (*firstwin.get()).w_winrow
    {
        win_new_screen_rows();
    }
    if (*curtab.get()).tp_old_Columns != Columns.get() as int64_t {
        if starting.get() == 0 as ::core::ffi::c_int {
            win_new_screen_cols();
            (*curtab.get()).tp_old_Columns = Columns.get() as int64_t;
        } else {
            (*curtab.get()).tp_old_Columns = -1 as int64_t;
        }
    }
    lastused_tabpage.set(old_curtab);
    if trigger_enter_autocmds {
        apply_autocmds(
            EVENT_TABENTER,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        if old_curbuf != curbuf.get() {
            apply_autocmds(
                EVENT_BUFENTER,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
        }
    }
    redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
}
unsafe extern "C" fn tabpage_check_windows(mut old_curtab: *mut tabpage_T) {
    let mut next_wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut wp: *mut win_T = (*old_curtab).tp_firstwin;
    while !wp.is_null() {
        next_wp = (*wp).w_next;
        if (*wp).w_floating {
            if (*wp).w_config.external {
                win_remove(wp, old_curtab);
                win_append(
                    lastwin_nofloating(::core::ptr::null_mut::<tabpage_T>()),
                    wp,
                    ::core::ptr::null_mut::<tabpage_T>(),
                );
            } else {
                ui_comp_remove_grid(&raw mut (*wp).w_grid_alloc);
            }
        }
        (*wp).w_pos_changed = true_0 != 0;
        wp = next_wp;
    }
    let mut wp_0: *mut win_T = firstwin.get();
    while !wp_0.is_null() {
        if (*wp_0).w_floating as ::core::ffi::c_int != 0 && !(*wp_0).w_config.external {
            win_config_float(wp_0, (*wp_0).w_config);
        }
        (*wp_0).w_pos_changed = true_0 != 0;
        wp_0 = (*wp_0).w_next;
    }
}
pub unsafe extern "C" fn goto_tabpage(mut n: ::core::ffi::c_int) {
    if text_locked() {
        text_locked_msg();
        return;
    }
    if (*first_tabpage.get()).tp_next.is_null() {
        if n > 1 as ::core::ffi::c_int {
            beep_flush();
        }
        return;
    }
    let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    if n == 0 as ::core::ffi::c_int {
        if (*curtab.get()).tp_next.is_null() {
            tp = first_tabpage.get();
        } else {
            tp = (*curtab.get()).tp_next;
        }
    } else if n < 0 as ::core::ffi::c_int {
        let mut ttp: *mut tabpage_T = curtab.get();
        let mut i: ::core::ffi::c_int = n;
        while i < 0 as ::core::ffi::c_int {
            tp = first_tabpage.get();
            while (*tp).tp_next != ttp && !(*tp).tp_next.is_null() {
                tp = (*tp).tp_next;
            }
            ttp = tp;
            i += 1;
        }
    } else if n == 9999 as ::core::ffi::c_int {
        tp = first_tabpage.get();
        while !(*tp).tp_next.is_null() {
            tp = (*tp).tp_next;
        }
    } else {
        tp = find_tabpage(n);
        if tp.is_null() {
            beep_flush();
            return;
        }
    }
    goto_tabpage_tp(tp, true_0 != 0, true_0 != 0);
}
pub unsafe extern "C" fn goto_tabpage_tp(
    mut tp: *mut tabpage_T,
    mut trigger_enter_autocmds: bool,
    mut trigger_leave_autocmds: bool,
) {
    if trigger_enter_autocmds as ::core::ffi::c_int != 0
        || trigger_leave_autocmds as ::core::ffi::c_int != 0
    {
        if cmdwin_type.get() != 0 as ::core::ffi::c_int {
            emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
            return;
        }
    }
    set_keep_msg(
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
    );
    skip_win_fix_scroll.set(true_0 != 0);
    if tp != curtab.get()
        && leave_tabpage((*(*tp).tp_curwin).w_buffer, trigger_leave_autocmds) == OK
    {
        if valid_tabpage(tp) {
            enter_tabpage(
                tp,
                curbuf.get(),
                trigger_enter_autocmds,
                trigger_leave_autocmds,
            );
        } else {
            enter_tabpage(
                curtab.get(),
                curbuf.get(),
                trigger_enter_autocmds,
                trigger_leave_autocmds,
            );
        }
    }
    skip_win_fix_scroll.set(false_0 != 0);
}
pub unsafe extern "C" fn goto_tabpage_lastused() -> bool {
    if !valid_tabpage(lastused_tabpage.get()) {
        return false_0 != 0;
    }
    goto_tabpage_tp(lastused_tabpage.get(), true_0 != 0, true_0 != 0);
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn goto_tabpage_win(mut tp: *mut tabpage_T, mut wp: *mut win_T) {
    goto_tabpage_tp(tp, true_0 != 0, true_0 != 0);
    if curtab.get() == tp && win_valid(wp) as ::core::ffi::c_int != 0 {
        win_enter(wp, true_0 != 0);
    }
}
pub unsafe extern "C" fn tabpage_move(mut nr: ::core::ffi::c_int) {
    '_c2rust_label: {
        if !(*curtab.ptr()).is_null() {
        } else {
            __assert_fail(
                b"curtab != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                4971 as ::core::ffi::c_uint,
                b"void tabpage_move(int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if (*first_tabpage.get()).tp_next.is_null() {
        return;
    }
    if tabpage_move_disallowed.get() != 0 {
        return;
    }
    let mut n: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    tp = first_tabpage.get();
    while !(*tp).tp_next.is_null() && n < nr {
        n += 1;
        tp = (*tp).tp_next;
    }
    if tp == curtab.get()
        || nr > 0 as ::core::ffi::c_int && !(*tp).tp_next.is_null() && (*tp).tp_next == curtab.get()
    {
        return;
    }
    let mut tp_dst: *mut tabpage_T = tp;
    if curtab.get() == first_tabpage.get() {
        first_tabpage.set((*curtab.get()).tp_next);
    } else {
        tp = ::core::ptr::null_mut::<tabpage_T>();
        let mut tp2: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp2.is_null() {
            if (*tp2).tp_next == curtab.get() {
                tp = tp2 as *mut tabpage_T;
                break;
            } else {
                tp2 = (*tp2).tp_next as *mut tabpage_T;
            }
        }
        if tp.is_null() {
            return;
        }
        (*tp).tp_next = (*curtab.get()).tp_next;
    }
    if nr <= 0 as ::core::ffi::c_int {
        (*curtab.get()).tp_next = first_tabpage.get();
        first_tabpage.set(curtab.get());
    } else {
        (*curtab.get()).tp_next = (*tp_dst).tp_next;
        (*tp_dst).tp_next = curtab.get();
    }
    redraw_tabline.set(true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn win_goto(mut wp: *mut win_T) {
    let mut owp: *mut win_T = curwin.get();
    if text_or_buf_locked() {
        beep_flush();
        return;
    }
    if (*wp).w_buffer != curbuf.get() {
        reset_VIsual_and_resel();
    } else if VIsual_active.get() {
        (*wp).w_cursor = (*curwin.get()).w_cursor;
    }
    if !win_valid(wp) {
        return;
    }
    win_enter(wp, true_0 != 0);
    if win_valid(owp) as ::core::ffi::c_int != 0
        && (*owp).w_onebuf_opt.wo_cole > 0 as OptInt
        && msg_scrolled.get() == 0
    {
        redrawWinline(owp, (*owp).w_cursor.lnum);
    }
    if (*curwin.get()).w_onebuf_opt.wo_cole > 0 as OptInt && msg_scrolled.get() == 0 {
        redrawWinline(curwin.get(), (*curwin.get()).w_cursor.lnum);
    }
}
pub unsafe extern "C" fn win_find_tabpage(mut win: *mut win_T) -> *mut tabpage_T {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if wp == win {
                return tp as *mut tabpage_T;
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    return ::core::ptr::null_mut::<tabpage_T>();
}
pub unsafe extern "C" fn win_vert_neighbor(
    mut tp: *mut tabpage_T,
    mut wp: *mut win_T,
    mut up: bool,
    mut count: ::core::ffi::c_int,
) -> *mut win_T {
    let mut foundfr: *mut frame_T = (*wp).w_frame;
    if (*wp).w_floating {
        return if win_valid(prevwin.get()) as ::core::ffi::c_int != 0
            && !(*prevwin.get()).w_floating
        {
            prevwin.get()
        } else {
            firstwin.get()
        };
    }
    '_end: loop {
        let c2rust_fresh2 = count;
        count = count - 1;
        if c2rust_fresh2 == 0 {
            break;
        }
        let mut nfr: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
        let mut fr: *mut frame_T = foundfr;
        loop {
            if fr == (*tp).tp_topframe {
                break '_end;
            }
            if up {
                nfr = (*fr).fr_prev;
            } else {
                nfr = (*fr).fr_next;
            }
            if (*(*fr).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL && !nfr.is_null() {
                break;
            }
            fr = (*fr).fr_parent;
        }
        loop {
            if (*nfr).fr_layout as ::core::ffi::c_int == FR_LEAF {
                foundfr = nfr;
                break;
            } else {
                fr = (*nfr).fr_child;
                if (*nfr).fr_layout as ::core::ffi::c_int == FR_ROW {
                    while !(*fr).fr_next.is_null()
                        && (*frame2win(fr)).w_wincol + (*fr).fr_width
                            <= (*wp).w_wincol + (*wp).w_wcol
                    {
                        fr = (*fr).fr_next;
                    }
                }
                if (*nfr).fr_layout as ::core::ffi::c_int == FR_COL && up as ::core::ffi::c_int != 0
                {
                    while !(*fr).fr_next.is_null() {
                        fr = (*fr).fr_next;
                    }
                }
                nfr = fr;
            }
        }
    }
    return if !foundfr.is_null() {
        (*foundfr).fr_win
    } else {
        ::core::ptr::null_mut::<win_T>()
    };
}
unsafe extern "C" fn win_goto_ver(mut up: bool, mut count: ::core::ffi::c_int) {
    let mut win: *mut win_T = win_vert_neighbor(curtab.get(), curwin.get(), up, count);
    if !win.is_null() {
        win_goto(win);
    }
}
pub unsafe extern "C" fn win_horz_neighbor(
    mut tp: *mut tabpage_T,
    mut wp: *mut win_T,
    mut left: bool,
    mut count: ::core::ffi::c_int,
) -> *mut win_T {
    let mut foundfr: *mut frame_T = (*wp).w_frame;
    if (*wp).w_floating {
        return if win_valid(prevwin.get()) as ::core::ffi::c_int != 0
            && !(*prevwin.get()).w_floating
        {
            prevwin.get()
        } else {
            firstwin.get()
        };
    }
    '_end: loop {
        let c2rust_fresh1 = count;
        count = count - 1;
        if c2rust_fresh1 == 0 {
            break;
        }
        let mut nfr: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
        let mut fr: *mut frame_T = foundfr;
        loop {
            if fr == (*tp).tp_topframe {
                break '_end;
            }
            if left {
                nfr = (*fr).fr_prev;
            } else {
                nfr = (*fr).fr_next;
            }
            if (*(*fr).fr_parent).fr_layout as ::core::ffi::c_int == FR_ROW && !nfr.is_null() {
                break;
            }
            fr = (*fr).fr_parent;
        }
        loop {
            if (*nfr).fr_layout as ::core::ffi::c_int == FR_LEAF {
                foundfr = nfr;
                break;
            } else {
                fr = (*nfr).fr_child;
                if (*nfr).fr_layout as ::core::ffi::c_int == FR_COL {
                    while !(*fr).fr_next.is_null()
                        && (*frame2win(fr)).w_winrow + (*fr).fr_height
                            <= (*wp).w_winrow + (*wp).w_wrow
                    {
                        fr = (*fr).fr_next;
                    }
                }
                if (*nfr).fr_layout as ::core::ffi::c_int == FR_ROW
                    && left as ::core::ffi::c_int != 0
                {
                    while !(*fr).fr_next.is_null() {
                        fr = (*fr).fr_next;
                    }
                }
                nfr = fr;
            }
        }
    }
    return if !foundfr.is_null() {
        (*foundfr).fr_win
    } else {
        ::core::ptr::null_mut::<win_T>()
    };
}
unsafe extern "C" fn win_goto_hor(mut left: bool, mut count: ::core::ffi::c_int) {
    let mut win: *mut win_T = win_horz_neighbor(curtab.get(), curwin.get(), left, count);
    if !win.is_null() {
        win_goto(win);
    }
}
#[no_mangle]
pub unsafe extern "C" fn win_enter(mut wp: *mut win_T, mut undo_sync: bool) {
    win_enter_ext(
        wp,
        (if undo_sync as ::core::ffi::c_int != 0 {
            WEE_UNDO_SYNC as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | WEE_TRIGGER_ENTER_AUTOCMDS as ::core::ffi::c_int
            | WEE_TRIGGER_LEAVE_AUTOCMDS as ::core::ffi::c_int,
    );
}
unsafe extern "C" fn win_enter_ext(wp: *mut win_T, flags: ::core::ffi::c_int) {
    let mut other_buffer: bool = false_0 != 0;
    let curwin_invalid: bool = flags & WEE_CURWIN_INVALID as ::core::ffi::c_int != 0;
    if wp == curwin.get() && !curwin_invalid {
        return;
    }
    if !curwin_invalid {
        leaving_window(curwin.get());
    }
    if !curwin_invalid && flags & WEE_TRIGGER_LEAVE_AUTOCMDS as ::core::ffi::c_int != 0 {
        if (*wp).w_buffer != curbuf.get() {
            apply_autocmds(
                EVENT_BUFLEAVE,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
            other_buffer = true_0 != 0;
            if !win_valid(wp) {
                return;
            }
        }
        apply_autocmds(
            EVENT_WINLEAVE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        if !win_valid(wp) {
            return;
        }
        if aborting() {
            return;
        }
    }
    if flags & WEE_UNDO_SYNC as ::core::ffi::c_int != 0 && curbuf.get() != (*wp).w_buffer {
        u_sync(false_0 != 0);
    }
    if *p_spk.get() as ::core::ffi::c_int == 'c' as ::core::ffi::c_int && !curwin_invalid {
        update_topline(curwin.get());
    }
    if (*wp).w_buffer != curbuf.get() {
        buf_copy_options(
            (*wp).w_buffer,
            BCO_ENTER as ::core::ffi::c_int | BCO_NOHELP as ::core::ffi::c_int,
        );
    }
    if !curwin_invalid {
        prevwin.set(curwin.get());
        (*curwin.get()).w_redr_status = true_0 != 0;
    }
    curwin.set(wp);
    curbuf.set((*wp).w_buffer);
    check_cursor(curwin.get());
    if !virtual_active(curwin.get()) {
        (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    }
    if *p_spk.get() as ::core::ffi::c_int == 'c' as ::core::ffi::c_int {
        changed_line_abv_curs();
    } else {
        win_fix_cursor(
            get_real_state()
                & (MODE_NORMAL as ::core::ffi::c_int
                    | MODE_CMDLINE as ::core::ffi::c_int
                    | MODE_TERMINAL as ::core::ffi::c_int)
                != 0,
        );
    }
    win_fix_current_dir();
    entering_window(curwin.get());
    if flags & WEE_TRIGGER_NEW_AUTOCMDS as ::core::ffi::c_int != 0 {
        apply_autocmds(
            EVENT_WINNEW,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
    }
    if flags & WEE_TRIGGER_ENTER_AUTOCMDS as ::core::ffi::c_int != 0 {
        apply_autocmds(
            EVENT_WINENTER,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        if other_buffer {
            apply_autocmds(
                EVENT_BUFENTER,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
        }
    }
    maketitle();
    (*curwin.get()).w_redr_status = true_0 != 0;
    redraw_tabline.set(true_0 != 0);
    if restart_edit.get() != 0 {
        redraw_later(curwin.get(), UPD_VALID as ::core::ffi::c_int);
    }
    if (*curwin.get()).w_hl_attr_normal != (*curwin.get()).w_hl_attr_normalnc {
        redraw_later(curwin.get(), UPD_NOT_VALID as ::core::ffi::c_int);
    }
    if !(*prevwin.ptr()).is_null() {
        if (*prevwin.get()).w_hl_attr_normal != (*prevwin.get()).w_hl_attr_normalnc {
            redraw_later(prevwin.get(), UPD_NOT_VALID as ::core::ffi::c_int);
        }
    }
    if ((*curwin.get()).w_height as OptInt) < p_wh.get()
        && (*curwin.get()).w_onebuf_opt.wo_wfh == 0
        && !(*curwin.get()).w_floating
    {
        win_setheight(p_wh.get() as ::core::ffi::c_int);
    } else if (*curwin.get()).w_height == 0 as ::core::ffi::c_int {
        win_setheight(1 as ::core::ffi::c_int);
    }
    if ((*curwin.get()).w_width as OptInt) < p_wiw.get()
        && (*curwin.get()).w_onebuf_opt.wo_wfw == 0
        && !(*curwin.get()).w_floating
    {
        win_setwidth(p_wiw.get() as ::core::ffi::c_int);
    }
    setmouse();
    do_autochdir();
}
pub unsafe extern "C" fn win_fix_current_dir() {
    let mut new_dir: *mut ::core::ffi::c_char = if !(*curwin.get()).w_localdir.is_null() {
        (*curwin.get()).w_localdir
    } else {
        (*curtab.get()).tp_localdir
    };
    let mut cwd: [::core::ffi::c_char; 4096] = [0; 4096];
    if os_dirname(&raw mut cwd as *mut ::core::ffi::c_char, MAXPATHL as size_t) != OK {
        cwd[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    }
    if !new_dir.is_null() {
        if (*globaldir.ptr()).is_null() {
            if cwd[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int != NUL {
                globaldir.set(xstrdup(&raw mut cwd as *mut ::core::ffi::c_char));
            }
        }
        let mut dir_differs: bool = pathcmp(
            new_dir,
            &raw mut cwd as *mut ::core::ffi::c_char,
            -1 as ::core::ffi::c_int,
        ) != 0 as ::core::ffi::c_int;
        if p_acd.get() == 0 && dir_differs as ::core::ffi::c_int != 0 {
            do_autocmd_dirchanged(
                new_dir,
                (if !(*curwin.get()).w_localdir.is_null() {
                    kCdScopeWindow as ::core::ffi::c_int
                } else {
                    kCdScopeTabpage as ::core::ffi::c_int
                }) as CdScope,
                kCdCauseWindow,
                true_0 != 0,
            );
        }
        if os_chdir(new_dir) == 0 as ::core::ffi::c_int {
            if p_acd.get() == 0 && dir_differs as ::core::ffi::c_int != 0 {
                do_autocmd_dirchanged(
                    new_dir,
                    (if !(*curwin.get()).w_localdir.is_null() {
                        kCdScopeWindow as ::core::ffi::c_int
                    } else {
                        kCdScopeTabpage as ::core::ffi::c_int
                    }) as CdScope,
                    kCdCauseWindow,
                    false_0 != 0,
                );
            }
        }
        last_chdir_reason.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        shorten_fnames(true_0);
    } else if !(*globaldir.ptr()).is_null() {
        let mut dir_differs_0: bool = pathcmp(
            globaldir.get(),
            &raw mut cwd as *mut ::core::ffi::c_char,
            -1 as ::core::ffi::c_int,
        ) != 0 as ::core::ffi::c_int;
        if p_acd.get() == 0 && dir_differs_0 as ::core::ffi::c_int != 0 {
            do_autocmd_dirchanged(globaldir.get(), kCdScopeGlobal, kCdCauseWindow, true_0 != 0);
        }
        if os_chdir(globaldir.get()) == 0 as ::core::ffi::c_int {
            if p_acd.get() == 0 && dir_differs_0 as ::core::ffi::c_int != 0 {
                do_autocmd_dirchanged(
                    globaldir.get(),
                    kCdScopeGlobal,
                    kCdCauseWindow,
                    false_0 != 0,
                );
            }
        }
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            globaldir.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
        last_chdir_reason.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        shorten_fnames(true_0);
    }
}
pub unsafe extern "C" fn buf_jump_open_win(mut buf: *mut buf_T) -> *mut win_T {
    if (*curwin.get()).w_buffer == buf {
        win_enter(curwin.get(), false_0 != 0);
        return curwin.get();
    }
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_buffer == buf {
            win_enter(wp, false_0 != 0);
            return wp;
        }
        wp = (*wp).w_next;
    }
    return ::core::ptr::null_mut::<win_T>();
}
pub unsafe extern "C" fn buf_jump_open_tab(mut buf: *mut buf_T) -> *mut win_T {
    let mut wp: *mut win_T = buf_jump_open_win(buf);
    if !wp.is_null() {
        return wp;
    }
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        if tp != curtab.get() {
            let mut wp_0: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp_0.is_null() {
                if (*wp_0).w_buffer == buf {
                    goto_tabpage_win(tp as *mut tabpage_T, wp_0);
                    if curwin.get() != wp_0 {
                        wp_0 = ::core::ptr::null_mut::<win_T>();
                    }
                    return wp_0;
                }
                wp_0 = (*wp_0).w_next;
            }
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    return ::core::ptr::null_mut::<win_T>();
}
static last_win_id: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(LOWEST_WIN_ID as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
pub unsafe extern "C" fn win_alloc(mut after: *mut win_T, mut hidden: bool) -> *mut win_T {
    let mut new_wp: *mut win_T =
        xcalloc(1 as size_t, ::core::mem::size_of::<win_T>()) as *mut win_T;
    (*last_win_id.ptr()) += 1;
    (*new_wp).handle = last_win_id.get() as handle_T;
    map_put_int_ptr_t(
        window_handles.ptr(),
        (*new_wp).handle as ::core::ffi::c_int,
        new_wp as ptr_t,
    );
    (*new_wp).w_grid_alloc.mouse_enabled = true_0 != 0;
    grid_assign_handle(&raw mut (*new_wp).w_grid_alloc);
    (*new_wp).w_vars = tv_dict_alloc();
    init_var_dict((*new_wp).w_vars, &raw mut (*new_wp).w_winvar, VAR_SCOPE);
    block_autocmds();
    if !hidden {
        let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
        if !after.is_null() {
            tp = win_find_tabpage(after);
            if tp == curtab.get() {
                tp = ::core::ptr::null_mut::<tabpage_T>();
            }
        }
        win_append(after, new_wp, tp);
    }
    (*new_wp).w_wincol = 0 as ::core::ffi::c_int;
    (*new_wp).w_width = Columns.get();
    (*new_wp).w_topline = 1 as ::core::ffi::c_int as linenr_T;
    (*new_wp).w_topfill = 0 as ::core::ffi::c_int;
    (*new_wp).w_botline = 2 as ::core::ffi::c_int as linenr_T;
    (*new_wp).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
    (*new_wp).w_scbind_pos = 1 as ::core::ffi::c_int;
    (*new_wp).w_floating = false;
    (*new_wp).w_config = WinConfig {
        window: 0,
        bufpos: lpos_T {
            lnum: -1 as linenr_T,
            col: 0 as colnr_T,
        },
        height: 0 as ::core::ffi::c_int,
        width: 0 as ::core::ffi::c_int,
        row: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
        col: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
        anchor: 0 as FloatAnchor,
        relative: kFloatRelativeEditor,
        external: false_0 != 0,
        focusable: true_0 != 0,
        mouse: true_0 != 0,
        split: kWinSplitLeft,
        zindex: kZIndexFloatDefault as ::core::ffi::c_int,
        style: kWinStyleUnused,
        border: false,
        shadow: false,
        border_chars: [[0; 32]; 8],
        border_hl_ids: [0; 8],
        border_attr: [0; 8],
        title: false,
        title_pos: kAlignLeft,
        title_chunks: VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        },
        title_width: 0,
        footer: false,
        footer_pos: kAlignLeft,
        footer_chunks: VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        },
        footer_width: 0,
        noautocmd: false_0 != 0,
        fixed: false_0 != 0,
        hide: false_0 != 0,
        _cmdline_offset: INT_MAX,
    };
    (*new_wp).w_viewport_invalid = true_0 != 0;
    (*new_wp).w_viewport_last_topline = 1 as ::core::ffi::c_int as linenr_T;
    (*new_wp).w_ns_hl = -1 as ::core::ffi::c_int;
    let mut ns_set: Set_uint32_t = SET_INIT;
    (*new_wp).w_ns_set = ns_set;
    (*new_wp).w_onebuf_opt.wo_so = -1 as OptInt;
    (*new_wp).w_allbuf_opt.wo_so = (*new_wp).w_onebuf_opt.wo_so;
    (*new_wp).w_onebuf_opt.wo_siso = -1 as OptInt;
    (*new_wp).w_allbuf_opt.wo_siso = (*new_wp).w_onebuf_opt.wo_siso;
    (*new_wp).w_fraction = 0 as ::core::ffi::c_int;
    (*new_wp).w_prev_fraction_row = -1 as ::core::ffi::c_int;
    foldInitWin(new_wp);
    unblock_autocmds();
    (*new_wp).w_next_match_id = 1000 as ::core::ffi::c_int;
    return new_wp;
}
pub unsafe extern "C" fn free_wininfo(mut wip: *mut WinInfo, mut bp: *mut buf_T) {
    if (*wip).wi_optset {
        clear_winopt(&raw mut (*wip).wi_opt);
        deleteFoldRecurse(bp, &raw mut (*wip).wi_folds);
    }
    xfree(wip as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn win_free(mut wp: *mut win_T, mut tp: *mut tabpage_T) {
    map_del_int_ptr_t(
        window_handles.ptr(),
        (*wp).handle as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
    );
    clearFolding(wp);
    alist_unlink((*wp).w_alist);
    block_autocmds();
    xfree((*wp).w_ns_set.keys as *mut ::core::ffi::c_void);
    xfree((*wp).w_ns_set.h.hash as *mut ::core::ffi::c_void);
    (*wp).w_ns_set = SET_INIT;
    clear_winopt(&raw mut (*wp).w_onebuf_opt);
    clear_winopt(&raw mut (*wp).w_allbuf_opt);
    xfree((*wp).w_p_lcs_chars.multispace as *mut ::core::ffi::c_void);
    xfree((*wp).w_p_lcs_chars.leadmultispace as *mut ::core::ffi::c_void);
    vars_clear(&raw mut (*(*wp).w_vars).dv_hashtab);
    hash_init(&raw mut (*(*wp).w_vars).dv_hashtab);
    unref_var_dict((*wp).w_vars);
    if prevwin.get() == wp {
        prevwin.set(::core::ptr::null_mut::<win_T>());
    }
    let mut ttp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !ttp.is_null() {
        if (*ttp).tp_prevwin == wp {
            (*ttp).tp_prevwin = ::core::ptr::null_mut::<win_T>();
        }
        ttp = (*ttp).tp_next as *mut tabpage_T;
    }
    xfree((*wp).w_lines as *mut ::core::ffi::c_void);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*wp).w_tagstacklen {
        tagstack_clear_entry((&raw mut (*wp).w_tagstack as *mut taggy_T).offset(i as isize));
        i += 1;
    }
    xfree((*wp).w_localdir as *mut ::core::ffi::c_void);
    xfree((*wp).w_prevdir as *mut ::core::ffi::c_void);
    stl_clear_click_defs((*wp).w_status_click_defs, (*wp).w_status_click_defs_size);
    xfree((*wp).w_status_click_defs as *mut ::core::ffi::c_void);
    stl_clear_click_defs((*wp).w_winbar_click_defs, (*wp).w_winbar_click_defs_size);
    xfree((*wp).w_winbar_click_defs as *mut ::core::ffi::c_void);
    stl_clear_click_defs(
        (*wp).w_statuscol_click_defs,
        (*wp).w_statuscol_click_defs_size,
    );
    xfree((*wp).w_statuscol_click_defs as *mut ::core::ffi::c_void);
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        let mut wip_wp: *mut WinInfo = ::core::ptr::null_mut::<WinInfo>();
        let mut pos_wip: size_t = (*buf).b_wininfo.size;
        let mut pos_null: size_t = (*buf).b_wininfo.size;
        let mut i_0: size_t = 0 as size_t;
        while i_0 < (*buf).b_wininfo.size {
            let mut wip: *mut WinInfo = *(*buf).b_wininfo.items.offset(i_0 as isize);
            if (*wip).wi_win == wp {
                wip_wp = wip;
                pos_wip = i_0;
            } else if (*wip).wi_win.is_null() {
                pos_null = i_0;
            }
            i_0 = i_0.wrapping_add(1);
        }
        if !wip_wp.is_null() {
            (*wip_wp).wi_win = ::core::ptr::null_mut::<win_T>();
            if (*wp).w_config.style as ::core::ffi::c_uint
                == kWinStyleMinimal as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*wip_wp).wi_optset as ::core::ffi::c_int != 0
            {
                clear_winopt(&raw mut (*wip_wp).wi_opt);
                deleteFoldRecurse(buf, &raw mut (*wip_wp).wi_folds);
                (*wip_wp).wi_optset = false_0 != 0;
            }
            if pos_null < (*buf).b_wininfo.size {
                let mut pos_delete: size_t = if pos_null > pos_wip {
                    pos_null
                } else {
                    pos_wip
                };
                free_wininfo(*(*buf).b_wininfo.items.offset(pos_delete as isize), buf);
                (*buf).b_wininfo.size = (*buf).b_wininfo.size.wrapping_sub(1 as size_t);
                (pos_delete < (*buf).b_wininfo.size
                    && !memmove(
                        (*buf).b_wininfo.items.offset(pos_delete as isize)
                            as *mut ::core::ffi::c_void,
                        (*buf)
                            .b_wininfo
                            .items
                            .offset(pos_delete.wrapping_add(1 as size_t) as isize)
                            as *const ::core::ffi::c_void,
                        (*buf)
                            .b_wininfo
                            .size
                            .wrapping_sub(pos_delete)
                            .wrapping_mul(::core::mem::size_of::<*mut WinInfo>()),
                    )
                    .is_null()) as ::core::ffi::c_int;
            }
        }
        buf = (*buf).b_next;
    }
    clear_virttext(&raw mut (*wp).w_config.title_chunks);
    clear_virttext(&raw mut (*wp).w_config.footer_chunks);
    clear_matches(wp);
    free_jumplist(wp);
    qf_free_all(wp);
    xfree((*wp).w_p_cc_cols as *mut ::core::ffi::c_void);
    win_free_grid(wp, false_0 != 0);
    if win_valid_any_tab(wp) {
        win_remove(wp, tp);
    }
    if autocmd_busy.get() {
        (*wp).w_next = au_pending_free_win.get();
        au_pending_free_win.set(wp);
    } else {
        xfree(wp as *mut ::core::ffi::c_void);
    }
    unblock_autocmds();
}
pub unsafe extern "C" fn win_free_grid(mut wp: *mut win_T, mut reinit: bool) {
    if (*wp).w_grid_alloc.handle != 0 as ::core::ffi::c_int
        && ui_has(kUIMultigrid) as ::core::ffi::c_int != 0
    {
        ui_call_grid_destroy((*wp).w_grid_alloc.handle as Integer);
    }
    grid_free(&raw mut (*wp).w_grid_alloc);
    if reinit {
        memset(
            &raw mut (*wp).w_grid_alloc as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<ScreenGrid>(),
        );
    }
}
pub unsafe extern "C" fn win_append(
    mut after: *mut win_T,
    mut wp: *mut win_T,
    mut tp: *mut tabpage_T,
) {
    '_c2rust_label: {
        if tp.is_null() || tp != curtab.get() {
        } else {
            __assert_fail(
                b"tp == NULL || tp != curtab\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                5674 as ::core::ffi::c_uint,
                b"void win_append(win_T *, win_T *, tabpage_T *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut first: *mut *mut win_T = if tp.is_null() {
        firstwin.ptr()
    } else {
        &raw mut (*tp).tp_firstwin
    };
    let mut last: *mut *mut win_T = if tp.is_null() {
        lastwin.ptr()
    } else {
        &raw mut (*tp).tp_lastwin
    };
    let mut before: *mut win_T = if after.is_null() {
        *first
    } else {
        (*after).w_next
    };
    (*wp).w_next = before;
    (*wp).w_prev = after;
    if after.is_null() {
        *first = wp;
    } else {
        (*after).w_next = wp;
    }
    if before.is_null() {
        *last = wp;
    } else {
        (*before).w_prev = wp;
    };
}
pub unsafe extern "C" fn win_remove(mut wp: *mut win_T, mut tp: *mut tabpage_T) {
    '_c2rust_label: {
        if tp.is_null() || tp != curtab.get() {
        } else {
            __assert_fail(
                b"tp == NULL || tp != curtab\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                5702 as ::core::ffi::c_uint,
                b"void win_remove(win_T *, tabpage_T *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if !(*wp).w_prev.is_null() {
        (*(*wp).w_prev).w_next = (*wp).w_next;
    } else if tp.is_null() {
        (*curtab.get()).tp_firstwin = (*wp).w_next;
        firstwin.set((*curtab.get()).tp_firstwin);
    } else {
        (*tp).tp_firstwin = (*wp).w_next;
    }
    if !(*wp).w_next.is_null() {
        (*(*wp).w_next).w_prev = (*wp).w_prev;
    } else if tp.is_null() {
        (*curtab.get()).tp_lastwin = (*wp).w_prev;
        lastwin.set((*curtab.get()).tp_lastwin);
    } else {
        (*tp).tp_lastwin = (*wp).w_prev;
    };
}
unsafe extern "C" fn frame_append(mut after: *mut frame_T, mut frp: *mut frame_T) {
    (*frp).fr_next = (*after).fr_next;
    (*after).fr_next = frp;
    if !(*frp).fr_next.is_null() {
        (*(*frp).fr_next).fr_prev = frp;
    }
    (*frp).fr_prev = after;
}
unsafe extern "C" fn frame_insert(mut before: *mut frame_T, mut frp: *mut frame_T) {
    (*frp).fr_next = before;
    (*frp).fr_prev = (*before).fr_prev;
    (*before).fr_prev = frp;
    if !(*frp).fr_prev.is_null() {
        (*(*frp).fr_prev).fr_next = frp;
    } else {
        (*(*frp).fr_parent).fr_child = frp;
    };
}
unsafe extern "C" fn frame_remove(mut frp: *mut frame_T) {
    if !(*frp).fr_prev.is_null() {
        (*(*frp).fr_prev).fr_next = (*frp).fr_next;
    } else {
        (*(*frp).fr_parent).fr_child = (*frp).fr_next;
    }
    if !(*frp).fr_next.is_null() {
        (*(*frp).fr_next).fr_prev = (*frp).fr_prev;
    }
}
pub unsafe extern "C" fn win_new_screensize() {
    static old_Rows: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    static old_Columns: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    if old_Rows.get() != Rows.get() {
        if p_window.get() == (old_Rows.get() - 1 as ::core::ffi::c_int) as OptInt
            || old_Rows.get() == 0 as ::core::ffi::c_int && !option_was_set(kOptWindow)
        {
            p_window.set((Rows.get() - 1 as ::core::ffi::c_int) as OptInt);
        }
        old_Rows.set(Rows.get());
        win_new_screen_rows();
    }
    if old_Columns.get() != Columns.get() {
        old_Columns.set(Columns.get());
        win_new_screen_cols();
    }
}
pub unsafe extern "C" fn win_new_screen_rows() {
    if (*firstwin.ptr()).is_null() {
        return;
    }
    let mut h: ::core::ffi::c_int = if (Rows.get() as OptInt
        - p_ch.get()
        - tabline_height() as OptInt
        - global_stl_height() as OptInt)
        as ::core::ffi::c_int
        > frame_minheight(topframe.get(), ::core::ptr::null_mut::<win_T>())
    {
        (Rows.get() as OptInt
            - p_ch.get()
            - tabline_height() as OptInt
            - global_stl_height() as OptInt) as ::core::ffi::c_int
    } else {
        frame_minheight(topframe.get(), ::core::ptr::null_mut::<win_T>())
    };
    frame_new_height(topframe.get(), h, false_0 != 0, true_0 != 0, false_0 != 0);
    if !frame_check_height(topframe.get(), h) {
        frame_new_height(topframe.get(), h, false_0 != 0, false_0 != 0, false_0 != 0);
    }
    win_comp_pos();
    win_reconfig_floats();
    compute_cmdrow();
    (*curtab.get()).tp_ch_used = p_ch.get();
    if !skip_win_fix_scroll.get() {
        win_fix_scroll(true_0 != 0);
    }
}
pub unsafe extern "C" fn win_new_screen_cols() {
    if (*firstwin.ptr()).is_null() {
        return;
    }
    frame_new_width(topframe.get(), Columns.get(), false_0 != 0, true_0 != 0);
    if !frame_check_width(topframe.get(), Columns.get()) {
        frame_new_width(topframe.get(), Columns.get(), false_0 != 0, false_0 != 0);
    }
    win_comp_pos();
    win_reconfig_floats();
}
pub unsafe extern "C" fn snapshot_windows_scroll_size() {
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        (*wp).w_last_topline = (*wp).w_topline;
        (*wp).w_last_topfill = (*wp).w_topfill;
        (*wp).w_last_leftcol = (*wp).w_leftcol;
        (*wp).w_last_skipcol = (*wp).w_skipcol;
        (*wp).w_last_width = (*wp).w_width;
        (*wp).w_last_height = (*wp).w_height;
        wp = (*wp).w_next;
    }
}
static did_initial_scroll_size_snapshot: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub unsafe extern "C" fn may_make_initial_scroll_size_snapshot() {
    if !did_initial_scroll_size_snapshot.get() {
        did_initial_scroll_size_snapshot.set(true_0 != 0);
        snapshot_windows_scroll_size();
    }
}
unsafe extern "C" fn make_win_info_dict(
    mut width: ::core::ffi::c_int,
    mut height: ::core::ffi::c_int,
    mut topline: ::core::ffi::c_int,
    mut topfill: ::core::ffi::c_int,
    mut leftcol: ::core::ffi::c_int,
    mut skipcol: ::core::ffi::c_int,
) -> *mut dict_T {
    let d: *mut dict_T = tv_dict_alloc();
    (*d).dv_refcount = 1 as ::core::ffi::c_int;
    let mut tv: typval_T = typval_T {
        v_type: VAR_NUMBER,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    tv.vval.v_number = width as varnumber_T;
    if tv_dict_add_tv(
        d,
        b"width\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        &raw mut tv,
    ) != FAIL
    {
        tv.vval.v_number = height as varnumber_T;
        if tv_dict_add_tv(
            d,
            b"height\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
            &raw mut tv,
        ) != FAIL
        {
            tv.vval.v_number = topline as varnumber_T;
            if tv_dict_add_tv(
                d,
                b"topline\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                &raw mut tv,
            ) != FAIL
            {
                tv.vval.v_number = topfill as varnumber_T;
                if tv_dict_add_tv(
                    d,
                    b"topfill\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                    &raw mut tv,
                ) != FAIL
                {
                    tv.vval.v_number = leftcol as varnumber_T;
                    if tv_dict_add_tv(
                        d,
                        b"leftcol\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                            .wrapping_sub(1 as size_t),
                        &raw mut tv,
                    ) != FAIL
                    {
                        tv.vval.v_number = skipcol as varnumber_T;
                        if tv_dict_add_tv(
                            d,
                            b"skipcol\0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                                .wrapping_sub(1 as size_t),
                            &raw mut tv,
                        ) != FAIL
                        {
                            return d;
                        }
                    }
                }
            }
        }
    }
    tv_dict_unref(d);
    return ::core::ptr::null_mut::<dict_T>();
}
unsafe extern "C" fn check_window_scroll_resize(
    mut size_count: *mut ::core::ffi::c_int,
    mut first_scroll_win: *mut *mut win_T,
    mut first_size_win: *mut *mut win_T,
    mut winlist: *mut list_T,
    mut v_event: *mut dict_T,
) {
    let mut tot_width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut tot_height: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut tot_topline: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut tot_topfill: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut tot_leftcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut tot_skipcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_floating as ::core::ffi::c_int != 0 && (*wp).w_last_topline == 0 as linenr_T {
            (*wp).w_last_topline = (*wp).w_topline;
            (*wp).w_last_topfill = (*wp).w_topfill;
            (*wp).w_last_leftcol = (*wp).w_leftcol;
            (*wp).w_last_skipcol = (*wp).w_skipcol;
            (*wp).w_last_width = (*wp).w_width;
            (*wp).w_last_height = (*wp).w_height;
        } else {
            let ignore_scroll: bool = event_ignored(EVENT_WINSCROLLED, (*wp).w_onebuf_opt.wo_eiw);
            let size_changed: bool = !event_ignored(EVENT_WINRESIZED, (*wp).w_onebuf_opt.wo_eiw)
                && ((*wp).w_last_width != (*wp).w_width || (*wp).w_last_height != (*wp).w_height);
            if size_changed {
                if !winlist.is_null() {
                    let mut tv: typval_T = typval_T {
                        v_type: VAR_NUMBER,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union {
                            v_number: (*wp).handle as varnumber_T,
                        },
                    };
                    tv_list_append_owned_tv(winlist, tv);
                } else if !size_count.is_null() {
                    '_c2rust_label: {
                        if !first_size_win.is_null() && !first_scroll_win.is_null() {
                        } else {
                            __assert_fail(
                                b"first_size_win != NULL && first_scroll_win != NULL\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/window.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                5942 as ::core::ffi::c_uint,
                                b"void check_window_scroll_resize(int *, win_T **, win_T **, list_T *, dict_T *)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    *size_count += 1;
                    if (*first_size_win).is_null() {
                        *first_size_win = wp;
                    }
                    if (*first_scroll_win).is_null() && !ignore_scroll {
                        *first_scroll_win = wp;
                    }
                }
            }
            let scroll_changed: bool = !ignore_scroll
                && ((*wp).w_last_topline != (*wp).w_topline
                    || (*wp).w_last_topfill != (*wp).w_topfill
                    || (*wp).w_last_leftcol != (*wp).w_leftcol
                    || (*wp).w_last_skipcol != (*wp).w_skipcol);
            if scroll_changed as ::core::ffi::c_int != 0
                && !first_scroll_win.is_null()
                && (*first_scroll_win).is_null()
            {
                *first_scroll_win = wp;
            }
            if (size_changed as ::core::ffi::c_int != 0
                || scroll_changed as ::core::ffi::c_int != 0)
                && !v_event.is_null()
            {
                let mut width: ::core::ffi::c_int = (*wp).w_width - (*wp).w_last_width;
                let mut height: ::core::ffi::c_int = (*wp).w_height - (*wp).w_last_height;
                let mut topline: ::core::ffi::c_int = (*wp).w_topline as ::core::ffi::c_int
                    - (*wp).w_last_topline as ::core::ffi::c_int;
                let mut topfill: ::core::ffi::c_int = (*wp).w_topfill - (*wp).w_last_topfill;
                let mut leftcol: ::core::ffi::c_int = (*wp).w_leftcol as ::core::ffi::c_int
                    - (*wp).w_last_leftcol as ::core::ffi::c_int;
                let mut skipcol: ::core::ffi::c_int = (*wp).w_skipcol as ::core::ffi::c_int
                    - (*wp).w_last_skipcol as ::core::ffi::c_int;
                let mut d: *mut dict_T =
                    make_win_info_dict(width, height, topline, topfill, leftcol, skipcol);
                if d.is_null() {
                    break;
                }
                let mut winid: [::core::ffi::c_char; 65] = [0; 65];
                let mut key_len: ::core::ffi::c_int = vim_snprintf(
                    &raw mut winid as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 65]>(),
                    b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                    (*wp).handle,
                );
                if tv_dict_add_dict(
                    v_event,
                    &raw mut winid as *mut ::core::ffi::c_char,
                    key_len as size_t,
                    d,
                ) == FAIL
                {
                    tv_dict_unref(d);
                    break;
                } else {
                    (*d).dv_refcount -= 1;
                    tot_width += abs(width);
                    tot_height += abs(height);
                    tot_topline += abs(topline);
                    tot_topfill += abs(topfill);
                    tot_leftcol += abs(leftcol);
                    tot_skipcol += abs(skipcol);
                }
            }
        }
        wp = (*wp).w_next;
    }
    if !v_event.is_null() {
        let mut alldict: *mut dict_T = make_win_info_dict(
            tot_width,
            tot_height,
            tot_topline,
            tot_topfill,
            tot_leftcol,
            tot_skipcol,
        );
        if !alldict.is_null() {
            if tv_dict_add_dict(
                v_event,
                b"all\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
                alldict,
            ) == FAIL
            {
                tv_dict_unref(alldict);
            } else {
                (*alldict).dv_refcount -= 1;
            }
        }
    }
}
pub unsafe extern "C" fn may_trigger_win_scrolled_resized() {
    static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let do_resize: bool = has_event(EVENT_WINRESIZED);
    let do_scroll: bool = has_event(EVENT_WINSCROLLED);
    if recursive.get() as ::core::ffi::c_int != 0
        || !(do_scroll as ::core::ffi::c_int != 0 || do_resize as ::core::ffi::c_int != 0)
        || !did_initial_scroll_size_snapshot.get()
    {
        return;
    }
    let mut size_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut first_scroll_win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut first_size_win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    check_window_scroll_resize(
        &raw mut size_count,
        &raw mut first_scroll_win,
        &raw mut first_size_win,
        ::core::ptr::null_mut::<list_T>(),
        ::core::ptr::null_mut::<dict_T>(),
    );
    let mut trigger_resize: bool =
        do_resize as ::core::ffi::c_int != 0 && size_count > 0 as ::core::ffi::c_int;
    let mut trigger_scroll: bool =
        do_scroll as ::core::ffi::c_int != 0 && !first_scroll_win.is_null();
    if !trigger_resize && !trigger_scroll {
        return;
    }
    let mut windows_list: *mut list_T = ::core::ptr::null_mut::<list_T>();
    if trigger_resize {
        windows_list = tv_list_alloc(size_count as ptrdiff_t);
        check_window_scroll_resize(
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ::core::ptr::null_mut::<*mut win_T>(),
            ::core::ptr::null_mut::<*mut win_T>(),
            windows_list,
            ::core::ptr::null_mut::<dict_T>(),
        );
    }
    let mut scroll_dict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    if trigger_scroll {
        scroll_dict = tv_dict_alloc();
        (*scroll_dict).dv_refcount = 1 as ::core::ffi::c_int;
        check_window_scroll_resize(
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ::core::ptr::null_mut::<*mut win_T>(),
            ::core::ptr::null_mut::<*mut win_T>(),
            ::core::ptr::null_mut::<list_T>(),
            scroll_dict,
        );
    }
    snapshot_windows_scroll_size();
    recursive.set(true_0 != 0);
    let mut resize_winid: [::core::ffi::c_char; 65] = [0; 65];
    let mut resize_bufref: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    if trigger_resize {
        vim_snprintf(
            &raw mut resize_winid as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 65]>(),
            b"%d\0".as_ptr() as *const ::core::ffi::c_char,
            (*first_size_win).handle,
        );
        set_bufref(&raw mut resize_bufref, (*first_size_win).w_buffer);
    }
    let mut scroll_winid: [::core::ffi::c_char; 65] = [0; 65];
    let mut scroll_bufref: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    if trigger_scroll {
        vim_snprintf(
            &raw mut scroll_winid as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 65]>(),
            b"%d\0".as_ptr() as *const ::core::ffi::c_char,
            (*first_scroll_win).handle,
        );
        set_bufref(&raw mut scroll_bufref, (*first_scroll_win).w_buffer);
    }
    if trigger_resize {
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
        let mut v_event: *mut dict_T = get_v_event(&raw mut save_v_event);
        if tv_dict_add_list(
            v_event,
            b"windows\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            windows_list,
        ) == OK
        {
            tv_dict_set_keys_readonly(v_event);
            let mut buf: *mut buf_T =
                if bufref_valid(&raw mut resize_bufref) as ::core::ffi::c_int != 0 {
                    resize_bufref.br_buf
                } else {
                    curbuf.get()
                };
            apply_autocmds(
                EVENT_WINRESIZED,
                &raw mut resize_winid as *mut ::core::ffi::c_char,
                &raw mut resize_winid as *mut ::core::ffi::c_char,
                false_0 != 0,
                buf,
            );
        }
        restore_v_event(v_event, &raw mut save_v_event);
    }
    if trigger_scroll {
        let mut save_v_event_0: save_v_event_T = save_v_event_T {
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
        let mut v_event_0: *mut dict_T = get_v_event(&raw mut save_v_event_0);
        tv_dict_extend(
            v_event_0,
            scroll_dict,
            b"move\0".as_ptr() as *const ::core::ffi::c_char,
        );
        tv_dict_set_keys_readonly(v_event_0);
        tv_dict_unref(scroll_dict);
        let mut buf_0: *mut buf_T =
            if bufref_valid(&raw mut scroll_bufref) as ::core::ffi::c_int != 0 {
                scroll_bufref.br_buf
            } else {
                curbuf.get()
            };
        apply_autocmds(
            EVENT_WINSCROLLED,
            &raw mut scroll_winid as *mut ::core::ffi::c_char,
            &raw mut scroll_winid as *mut ::core::ffi::c_char,
            false_0 != 0,
            buf_0,
        );
        restore_v_event(v_event_0, &raw mut save_v_event_0);
    }
    recursive.set(false_0 != 0);
}
pub unsafe extern "C" fn win_size_save(mut gap: *mut garray_T) {
    ga_init(
        gap,
        ::core::mem::size_of::<::core::ffi::c_int>() as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    ga_grow(
        gap,
        win_count() * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
    );
    let c2rust_fresh3 = (*gap).ga_len;
    (*gap).ga_len = (*gap).ga_len + 1;
    *((*gap).ga_data as *mut ::core::ffi::c_int).offset(c2rust_fresh3 as isize) =
        (Rows.get() as OptInt
            - p_ch.get()
            - tabline_height() as OptInt
            - global_stl_height() as OptInt) as ::core::ffi::c_int
            + global_stl_height()
            - last_stl_height(false_0 != 0);
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        let c2rust_fresh4 = (*gap).ga_len;
        (*gap).ga_len = (*gap).ga_len + 1;
        *((*gap).ga_data as *mut ::core::ffi::c_int).offset(c2rust_fresh4 as isize) =
            (*wp).w_width + (*wp).w_vsep_width;
        let c2rust_fresh5 = (*gap).ga_len;
        (*gap).ga_len = (*gap).ga_len + 1;
        *((*gap).ga_data as *mut ::core::ffi::c_int).offset(c2rust_fresh5 as isize) =
            (*wp).w_height;
        wp = (*wp).w_next;
    }
}
pub unsafe extern "C" fn win_size_restore(mut gap: *mut garray_T) {
    if win_count() * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int == (*gap).ga_len
        && *((*gap).ga_data as *mut ::core::ffi::c_int).offset(0 as ::core::ffi::c_int as isize)
            as OptInt
            == Rows.get() as OptInt
                - p_ch.get()
                - tabline_height() as OptInt
                - global_stl_height() as OptInt
                + global_stl_height() as OptInt
                - last_stl_height(false_0 != 0) as OptInt
    {
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while j < 2 as ::core::ffi::c_int {
            let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                firstwin.get()
            } else {
                (*curtab.get()).tp_firstwin
            };
            while !wp.is_null() {
                let c2rust_fresh6 = i;
                i = i + 1;
                let mut width: ::core::ffi::c_int =
                    *((*gap).ga_data as *mut ::core::ffi::c_int).offset(c2rust_fresh6 as isize);
                let c2rust_fresh7 = i;
                i = i + 1;
                let mut height: ::core::ffi::c_int =
                    *((*gap).ga_data as *mut ::core::ffi::c_int).offset(c2rust_fresh7 as isize);
                if !(*wp).w_floating {
                    frame_setwidth((*wp).w_frame, width);
                    win_setheight_win(height, wp);
                }
                wp = (*wp).w_next;
            }
            j += 1;
        }
        win_comp_pos();
    }
}
pub unsafe extern "C" fn win_comp_pos() -> ::core::ffi::c_int {
    let mut row: ::core::ffi::c_int = tabline_height();
    let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    frame_comp_pos(topframe.get(), &raw mut row, &raw mut col);
    let mut wp: *mut win_T = lastwin.get();
    while !wp.is_null() && (*wp).w_floating as ::core::ffi::c_int != 0 {
        if (*wp).w_config.relative as ::core::ffi::c_uint
            == kFloatRelativeWindow as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*wp).w_pos_changed = true_0 != 0;
        }
        wp = (*wp).w_prev;
    }
    return row + global_stl_height();
}
unsafe extern "C" fn frame_comp_pos(
    mut topfrp: *mut frame_T,
    mut row: *mut ::core::ffi::c_int,
    mut col: *mut ::core::ffi::c_int,
) {
    let mut wp: *mut win_T = (*topfrp).fr_win;
    if !wp.is_null() {
        if (*wp).w_winrow != *row || (*wp).w_wincol != *col {
            (*wp).w_winrow = *row;
            (*wp).w_wincol = *col;
            redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
            (*wp).w_redr_status = true_0 != 0;
            (*wp).w_pos_changed = true_0 != 0;
        }
        let h: ::core::ffi::c_int = (*wp).w_height + (*wp).w_hsep_height + (*wp).w_status_height;
        *row += if h > (*topfrp).fr_height {
            (*topfrp).fr_height
        } else {
            h
        };
        *col += (*wp).w_width + (*wp).w_vsep_width;
    } else {
        let mut startrow: ::core::ffi::c_int = *row;
        let mut startcol: ::core::ffi::c_int = *col;
        let mut frp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
        frp = (*topfrp).fr_child;
        while !frp.is_null() {
            if (*topfrp).fr_layout as ::core::ffi::c_int == FR_ROW {
                *row = startrow;
            } else {
                *col = startcol;
            }
            frame_comp_pos(frp, row, col);
            frp = (*frp).fr_next;
        }
    };
}
pub unsafe extern "C" fn win_setheight(mut height: ::core::ffi::c_int) {
    win_setheight_win(height, curwin.get());
}
pub unsafe extern "C" fn win_setheight_win(mut height: ::core::ffi::c_int, mut win: *mut win_T) {
    height = if height
        > (if win == curwin.get() {
            if p_wmh.get() > 1 as OptInt {
                p_wmh.get()
            } else {
                1 as OptInt
            }
        } else {
            p_wmh.get()
        }) as ::core::ffi::c_int
            + (*win).w_winbar_height
    {
        height
    } else {
        (if win == curwin.get() {
            if p_wmh.get() > 1 as OptInt {
                p_wmh.get()
            } else {
                1 as OptInt
            }
        } else {
            p_wmh.get()
        }) as ::core::ffi::c_int
            + (*win).w_winbar_height
    };
    if (*win).w_floating {
        (*win).w_config.height = if height > 1 as ::core::ffi::c_int {
            height
        } else {
            1 as ::core::ffi::c_int
        };
        win_config_float(win, (*win).w_config);
        redraw_later(win, UPD_VALID as ::core::ffi::c_int);
    } else {
        frame_setheight(
            (*win).w_frame,
            height + (*win).w_hsep_height + (*win).w_status_height,
        );
        win_comp_pos();
        win_fix_scroll(true_0 != 0);
        redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
        redraw_cmdline.set(true_0 != 0);
    };
}
unsafe extern "C" fn frame_setheight(mut curfrp: *mut frame_T, mut height: ::core::ffi::c_int) {
    if (*curfrp).fr_height == height {
        return;
    }
    if (*curfrp).fr_parent.is_null() {
        if height > 0 as ::core::ffi::c_int {
            frame_new_height(curfrp, height, false_0 != 0, false_0 != 0, true_0 != 0);
        }
    } else if (*(*curfrp).fr_parent).fr_layout as ::core::ffi::c_int == FR_ROW {
        let mut h: ::core::ffi::c_int =
            frame_minheight((*curfrp).fr_parent, ::core::ptr::null_mut::<win_T>());
        height = if height > h { height } else { h };
        frame_setheight((*curfrp).fr_parent, height);
    } else {
        let mut room: ::core::ffi::c_int = 0;
        let mut room_cmdline: ::core::ffi::c_int = 0;
        let mut room_reserved: ::core::ffi::c_int = 0;
        let mut run: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while run <= 2 as ::core::ffi::c_int {
            room = 0 as ::core::ffi::c_int;
            room_reserved = 0 as ::core::ffi::c_int;
            let mut frp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
            frp = (*(*curfrp).fr_parent).fr_child;
            while !frp.is_null() {
                if frp != curfrp
                    && !(*frp).fr_win.is_null()
                    && (*(*frp).fr_win).w_onebuf_opt.wo_wfh != 0
                {
                    room_reserved += (*frp).fr_height;
                }
                room += (*frp).fr_height;
                if frp != curfrp {
                    room -= frame_minheight(frp, ::core::ptr::null_mut::<win_T>());
                }
                frp = (*frp).fr_next;
            }
            if (*curfrp).fr_width != Columns.get() {
                room_cmdline = 0 as ::core::ffi::c_int;
            } else {
                let mut wp: *mut win_T = lastwin_nofloating(::core::ptr::null_mut::<tabpage_T>());
                room_cmdline = Rows.get()
                    - p_ch.get() as ::core::ffi::c_int
                    - global_stl_height()
                    - ((*wp).w_winrow
                        + (*wp).w_height
                        + (*wp).w_hsep_height
                        + (*wp).w_status_height);
                room_cmdline = if room_cmdline > 0 as ::core::ffi::c_int {
                    room_cmdline
                } else {
                    0 as ::core::ffi::c_int
                };
            }
            if height <= room + room_cmdline {
                break;
            }
            if run == 2 as ::core::ffi::c_int || (*curfrp).fr_width == Columns.get() {
                height = room + room_cmdline;
                break;
            } else {
                frame_setheight(
                    (*curfrp).fr_parent,
                    height + frame_minheight((*curfrp).fr_parent, NOWIN)
                        - p_wmh.get() as ::core::ffi::c_int
                        - 1 as ::core::ffi::c_int,
                );
                run += 1;
            }
        }
        let mut take: ::core::ffi::c_int = height - (*curfrp).fr_height;
        if height > room + room_cmdline - room_reserved {
            room_reserved = room + room_cmdline - height;
        }
        if take < 0 as ::core::ffi::c_int && room - (*curfrp).fr_height <= room_reserved {
            room_reserved = 0 as ::core::ffi::c_int;
        }
        if take > 0 as ::core::ffi::c_int && room_cmdline > 0 as ::core::ffi::c_int {
            room_cmdline = if room_cmdline < take {
                room_cmdline
            } else {
                take
            };
            take -= room_cmdline;
            (*topframe.get()).fr_height += room_cmdline;
        }
        frame_new_height(curfrp, height, false_0 != 0, false_0 != 0, true_0 != 0);
        let mut run_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while run_0 < 2 as ::core::ffi::c_int {
            let mut frp_0: *mut frame_T = if run_0 == 0 as ::core::ffi::c_int {
                (*curfrp).fr_next
            } else {
                (*curfrp).fr_prev
            };
            while !frp_0.is_null() && take != 0 as ::core::ffi::c_int {
                let mut h_0: ::core::ffi::c_int =
                    frame_minheight(frp_0, ::core::ptr::null_mut::<win_T>());
                if room_reserved > 0 as ::core::ffi::c_int
                    && !(*frp_0).fr_win.is_null()
                    && (*(*frp_0).fr_win).w_onebuf_opt.wo_wfh != 0
                {
                    if room_reserved >= (*frp_0).fr_height {
                        room_reserved -= (*frp_0).fr_height;
                    } else {
                        if (*frp_0).fr_height - room_reserved > take {
                            room_reserved = (*frp_0).fr_height - take;
                        }
                        take -= (*frp_0).fr_height - room_reserved;
                        frame_new_height(
                            frp_0,
                            room_reserved,
                            false_0 != 0,
                            false_0 != 0,
                            true_0 != 0,
                        );
                        room_reserved = 0 as ::core::ffi::c_int;
                    }
                } else if (*frp_0).fr_height - take < h_0 {
                    take -= (*frp_0).fr_height - h_0;
                    frame_new_height(frp_0, h_0, false_0 != 0, false_0 != 0, true_0 != 0);
                } else {
                    frame_new_height(
                        frp_0,
                        (*frp_0).fr_height - take,
                        false_0 != 0,
                        false_0 != 0,
                        true_0 != 0,
                    );
                    take = 0 as ::core::ffi::c_int;
                }
                if run_0 == 0 as ::core::ffi::c_int {
                    frp_0 = (*frp_0).fr_next;
                } else {
                    frp_0 = (*frp_0).fr_prev;
                }
            }
            run_0 += 1;
        }
    };
}
pub unsafe extern "C" fn win_setwidth(mut width: ::core::ffi::c_int) {
    win_setwidth_win(width, curwin.get());
}
pub unsafe extern "C" fn win_setwidth_win(mut width: ::core::ffi::c_int, mut wp: *mut win_T) {
    if wp == curwin.get() {
        width = if (if width > p_wmw.get() as ::core::ffi::c_int {
            width
        } else {
            p_wmw.get() as ::core::ffi::c_int
        }) > 1 as ::core::ffi::c_int
        {
            if width > p_wmw.get() as ::core::ffi::c_int {
                width
            } else {
                p_wmw.get() as ::core::ffi::c_int
            }
        } else {
            1 as ::core::ffi::c_int
        };
    } else if width < 0 as ::core::ffi::c_int {
        width = 0 as ::core::ffi::c_int;
    }
    if (*wp).w_floating {
        (*wp).w_config.width = width;
        win_config_float(wp, (*wp).w_config);
        redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
    } else {
        frame_setwidth((*wp).w_frame, width + (*wp).w_vsep_width);
        win_comp_pos();
        redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
    };
}
unsafe extern "C" fn frame_setwidth(mut curfrp: *mut frame_T, mut width: ::core::ffi::c_int) {
    if (*curfrp).fr_width == width {
        return;
    }
    if (*curfrp).fr_parent.is_null() {
        return;
    }
    if (*(*curfrp).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL {
        let mut w: ::core::ffi::c_int =
            frame_minwidth((*curfrp).fr_parent, ::core::ptr::null_mut::<win_T>());
        width = if width > w { width } else { w };
        frame_setwidth((*curfrp).fr_parent, width);
    } else {
        let mut room: ::core::ffi::c_int = 0;
        let mut room_reserved: ::core::ffi::c_int = 0;
        let mut run: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while run <= 2 as ::core::ffi::c_int {
            room = 0 as ::core::ffi::c_int;
            room_reserved = 0 as ::core::ffi::c_int;
            let mut frp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
            frp = (*(*curfrp).fr_parent).fr_child;
            while !frp.is_null() {
                if frp != curfrp
                    && !(*frp).fr_win.is_null()
                    && (*(*frp).fr_win).w_onebuf_opt.wo_wfw != 0
                {
                    room_reserved += (*frp).fr_width;
                }
                room += (*frp).fr_width;
                if frp != curfrp {
                    room -= frame_minwidth(frp, ::core::ptr::null_mut::<win_T>());
                }
                frp = (*frp).fr_next;
            }
            if width <= room {
                break;
            }
            if run == 2 as ::core::ffi::c_int
                || (*curfrp).fr_height as OptInt
                    >= Rows.get() as OptInt
                        - p_ch.get()
                        - tabline_height() as OptInt
                        - global_stl_height() as OptInt
            {
                width = room;
                break;
            } else {
                frame_setwidth(
                    (*curfrp).fr_parent,
                    width + frame_minwidth((*curfrp).fr_parent, NOWIN)
                        - p_wmw.get() as ::core::ffi::c_int
                        - 1 as ::core::ffi::c_int,
                );
                run += 1;
            }
        }
        let mut take: ::core::ffi::c_int = width - (*curfrp).fr_width;
        if width > room - room_reserved {
            room_reserved = room - width;
        }
        if take < 0 as ::core::ffi::c_int && room - (*curfrp).fr_width < room_reserved {
            room_reserved = 0 as ::core::ffi::c_int;
        }
        frame_new_width(curfrp, width, false_0 != 0, false_0 != 0);
        let mut run_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while run_0 < 2 as ::core::ffi::c_int {
            let mut frp_0: *mut frame_T = if run_0 == 0 as ::core::ffi::c_int {
                (*curfrp).fr_next
            } else {
                (*curfrp).fr_prev
            };
            while !frp_0.is_null() && take != 0 as ::core::ffi::c_int {
                let mut w_0: ::core::ffi::c_int =
                    frame_minwidth(frp_0, ::core::ptr::null_mut::<win_T>());
                if room_reserved > 0 as ::core::ffi::c_int
                    && !(*frp_0).fr_win.is_null()
                    && (*(*frp_0).fr_win).w_onebuf_opt.wo_wfw != 0
                {
                    if room_reserved >= (*frp_0).fr_width {
                        room_reserved -= (*frp_0).fr_width;
                    } else {
                        if (*frp_0).fr_width - room_reserved > take {
                            room_reserved = (*frp_0).fr_width - take;
                        }
                        take -= (*frp_0).fr_width - room_reserved;
                        frame_new_width(frp_0, room_reserved, false_0 != 0, false_0 != 0);
                        room_reserved = 0 as ::core::ffi::c_int;
                    }
                } else if (*frp_0).fr_width - take < w_0 {
                    take -= (*frp_0).fr_width - w_0;
                    frame_new_width(frp_0, w_0, false_0 != 0, false_0 != 0);
                } else {
                    frame_new_width(frp_0, (*frp_0).fr_width - take, false_0 != 0, false_0 != 0);
                    take = 0 as ::core::ffi::c_int;
                }
                if run_0 == 0 as ::core::ffi::c_int {
                    frp_0 = (*frp_0).fr_next;
                } else {
                    frp_0 = (*frp_0).fr_prev;
                }
            }
            run_0 += 1;
        }
    };
}
pub unsafe extern "C" fn did_set_winminheight(
    mut _args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    let mut first: bool = true_0 != 0;
    while p_wmh.get() > 0 as OptInt {
        let room: ::core::ffi::c_int = Rows.get() - p_ch.get() as ::core::ffi::c_int;
        let needed: ::core::ffi::c_int = min_rows_for_all_tabpages();
        if room >= needed {
            break;
        }
        (*p_wmh.ptr()) -= 1;
        if first {
            emsg(gettext(&raw const e_noroom as *const ::core::ffi::c_char));
            first = false_0 != 0;
        }
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn did_set_winminwidth(
    mut _args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    let mut first: bool = true_0 != 0;
    while p_wmw.get() > 0 as OptInt {
        let room: ::core::ffi::c_int = Columns.get();
        let needed: ::core::ffi::c_int =
            frame_minwidth(topframe.get(), ::core::ptr::null_mut::<win_T>());
        if room >= needed {
            break;
        }
        (*p_wmw.ptr()) -= 1;
        if first {
            emsg(gettext(&raw const e_noroom as *const ::core::ffi::c_char));
            first = false_0 != 0;
        }
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn win_drag_status_line(
    mut dragwin: *mut win_T,
    mut offset: ::core::ffi::c_int,
) {
    let mut fr: *mut frame_T = (*dragwin).w_frame;
    let mut curfr: *mut frame_T = fr;
    if fr != topframe.get() {
        fr = (*fr).fr_parent;
        if (*fr).fr_layout as ::core::ffi::c_int != FR_COL {
            curfr = fr;
            if fr != topframe.get() {
                fr = (*fr).fr_parent;
            }
        }
    }
    while curfr != topframe.get() && (*curfr).fr_next.is_null() {
        if fr != topframe.get() {
            fr = (*fr).fr_parent;
        }
        curfr = fr;
        if fr != topframe.get() {
            fr = (*fr).fr_parent;
        }
    }
    let mut room: ::core::ffi::c_int = 0;
    let up: bool = offset < 0 as ::core::ffi::c_int;
    if up {
        offset = -offset;
        if fr == curfr {
            room = (*fr).fr_height - frame_minheight(fr, ::core::ptr::null_mut::<win_T>());
        } else {
            room = 0 as ::core::ffi::c_int;
            fr = (*fr).fr_child;
            loop {
                room += (*fr).fr_height - frame_minheight(fr, ::core::ptr::null_mut::<win_T>());
                if fr == curfr {
                    break;
                }
                fr = (*fr).fr_next;
            }
        }
        fr = (*curfr).fr_next;
    } else {
        room = Rows.get() - cmdline_row.get();
        if !(*curfr).fr_next.is_null() {
            room -= p_ch.get() as ::core::ffi::c_int + global_stl_height();
        } else if min_set_ch.get() > 0 as OptInt {
            room -= 1;
        }
        room = if room > 0 as ::core::ffi::c_int {
            room
        } else {
            0 as ::core::ffi::c_int
        };
        fr = (*curfr).fr_next;
        while !fr.is_null() {
            room += (*fr).fr_height - frame_minheight(fr, ::core::ptr::null_mut::<win_T>());
            fr = (*fr).fr_next;
        }
        fr = curfr;
    }
    offset = if offset < room { offset } else { room };
    if offset <= 0 as ::core::ffi::c_int {
        return;
    }
    if !fr.is_null() {
        frame_new_height(fr, (*fr).fr_height + offset, up, false_0 != 0, true_0 != 0);
    }
    if up {
        fr = curfr;
    } else {
        fr = (*curfr).fr_next;
    }
    while !fr.is_null() && offset > 0 as ::core::ffi::c_int {
        let mut n: ::core::ffi::c_int = frame_minheight(fr, ::core::ptr::null_mut::<win_T>());
        if (*fr).fr_height - offset <= n {
            offset -= (*fr).fr_height - n;
            frame_new_height(fr, n, !up, false_0 != 0, true_0 != 0);
            if up {
                fr = (*fr).fr_prev;
            } else {
                fr = (*fr).fr_next;
            }
        } else {
            frame_new_height(fr, (*fr).fr_height - offset, !up, false_0 != 0, true_0 != 0);
            break;
        }
    }
    win_comp_pos();
    win_fix_scroll(true_0 != 0);
    redraw_all_later(UPD_SOME_VALID as ::core::ffi::c_int);
    showmode();
}
pub unsafe extern "C" fn win_drag_vsep_line(
    mut dragwin: *mut win_T,
    mut offset: ::core::ffi::c_int,
) {
    let mut fr: *mut frame_T = (*dragwin).w_frame;
    if fr == topframe.get() {
        return;
    }
    let mut curfr: *mut frame_T = fr;
    fr = (*fr).fr_parent;
    if (*fr).fr_layout as ::core::ffi::c_int != FR_ROW {
        if fr == topframe.get() {
            return;
        }
        curfr = fr;
        fr = (*fr).fr_parent;
    }
    while (*curfr).fr_next.is_null() {
        if fr == topframe.get() {
            break;
        }
        curfr = fr;
        fr = (*fr).fr_parent;
        if fr != topframe.get() {
            curfr = fr;
            fr = (*fr).fr_parent;
        }
    }
    let mut room: ::core::ffi::c_int = 0;
    let left: bool = offset < 0 as ::core::ffi::c_int;
    if left {
        offset = -offset;
        room = 0 as ::core::ffi::c_int;
        fr = (*fr).fr_child;
        loop {
            room += (*fr).fr_width - frame_minwidth(fr, ::core::ptr::null_mut::<win_T>());
            if fr == curfr {
                break;
            }
            fr = (*fr).fr_next;
        }
        fr = (*curfr).fr_next;
    } else {
        room = 0 as ::core::ffi::c_int;
        fr = (*curfr).fr_next;
        while !fr.is_null() {
            room += (*fr).fr_width - frame_minwidth(fr, ::core::ptr::null_mut::<win_T>());
            fr = (*fr).fr_next;
        }
        fr = curfr;
    }
    offset = if offset < room { offset } else { room };
    if offset <= 0 as ::core::ffi::c_int {
        return;
    }
    if fr.is_null() {
        return;
    }
    frame_new_width(fr, (*fr).fr_width + offset, left, false_0 != 0);
    if left {
        fr = curfr;
    } else {
        fr = (*curfr).fr_next;
    }
    while !fr.is_null() && offset > 0 as ::core::ffi::c_int {
        let mut n: ::core::ffi::c_int = frame_minwidth(fr, ::core::ptr::null_mut::<win_T>());
        if (*fr).fr_width - offset <= n {
            offset -= (*fr).fr_width - n;
            frame_new_width(fr, n, !left, false_0 != 0);
            if left {
                fr = (*fr).fr_prev;
            } else {
                fr = (*fr).fr_next;
            }
        } else {
            frame_new_width(fr, (*fr).fr_width - offset, !left, false_0 != 0);
            break;
        }
    }
    win_comp_pos();
    redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
}
pub const FRACTION_MULT: ::core::ffi::c_int = 16384 as ::core::ffi::c_int;
pub unsafe extern "C" fn set_fraction(mut wp: *mut win_T) {
    if (*wp).w_view_height > 1 as ::core::ffi::c_int {
        (*wp).w_fraction = ((*wp).w_wrow * FRACTION_MULT + FRACTION_MULT / 2 as ::core::ffi::c_int)
            / (*wp).w_view_height;
    }
}
pub unsafe extern "C" fn win_fix_scroll(mut resize: bool) {
    if *p_spk.get() as ::core::ffi::c_int == 'c' as ::core::ffi::c_int {
        return;
    }
    skip_update_topline.set(true_0 != 0);
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if !(*wp).w_floating && (*wp).w_height != (*wp).w_prev_height {
            (*wp).w_do_win_fix_cursor = true_0 != 0;
            if *p_spk.get() as ::core::ffi::c_int == 's' as ::core::ffi::c_int
                && (*wp).w_winrow != (*wp).w_prev_winrow
                && (*wp).w_botline - 1 as linenr_T <= (*(*wp).w_buffer).b_ml.ml_line_count
            {
                let mut diff: ::core::ffi::c_int =
                    (*wp).w_winrow - (*wp).w_prev_winrow + ((*wp).w_height - (*wp).w_prev_height);
                let mut cursor: pos_T = (*wp).w_cursor;
                (*wp).w_cursor.lnum = (*wp).w_botline - 1 as linenr_T;
                if diff > 0 as ::core::ffi::c_int {
                    cursor_down_inner(wp, diff, false_0 != 0);
                } else {
                    cursor_up_inner(wp, -(diff as linenr_T), false_0 != 0);
                }
                (*wp).w_fraction = FRACTION_MULT;
                scroll_to_fraction(wp, (*wp).w_prev_height);
                (*wp).w_cursor = cursor;
                (*wp).w_valid &= !VALID_WCOL;
            } else if wp == curwin.get() {
                (*wp).w_valid &= !VALID_CROW;
            }
            invalidate_botline_win(wp);
            validate_botline_win(wp);
        }
        (*wp).w_prev_height = (*wp).w_height;
        (*wp).w_prev_winrow = (*wp).w_winrow;
        wp = (*wp).w_next;
    }
    skip_update_topline.set(false_0 != 0);
    if get_real_state()
        & (MODE_NORMAL as ::core::ffi::c_int
            | MODE_CMDLINE as ::core::ffi::c_int
            | MODE_TERMINAL as ::core::ffi::c_int)
        == 0
    {
        win_fix_cursor(false_0 != 0);
    } else if resize {
        win_fix_cursor(true_0 != 0);
    }
}
unsafe extern "C" fn win_fix_cursor(mut normal: bool) {
    let mut wp: *mut win_T = curwin.get();
    if skip_win_fix_cursor.get() as ::core::ffi::c_int != 0
        || !(*wp).w_do_win_fix_cursor
        || (*(*wp).w_buffer).b_ml.ml_line_count < (*wp).w_view_height as linenr_T
    {
        return;
    }
    (*wp).w_do_win_fix_cursor = false_0 != 0;
    let mut so: ::core::ffi::c_int = (if (((*wp).w_view_height / 2 as ::core::ffi::c_int)
        as int64_t)
        < get_scrolloff_value(wp)
    {
        ((*wp).w_view_height / 2 as ::core::ffi::c_int) as int64_t
    } else {
        get_scrolloff_value(wp)
    }) as ::core::ffi::c_int;
    let mut lnum: linenr_T = (*wp).w_cursor.lnum;
    (*wp).w_cursor.lnum = (*wp).w_topline;
    cursor_down_inner(wp, so, false_0 != 0);
    let mut top: linenr_T = (*wp).w_cursor.lnum;
    (*wp).w_cursor.lnum = (*wp).w_botline - 1 as linenr_T;
    cursor_up_inner(wp, so as linenr_T, false_0 != 0);
    let mut bot: linenr_T = (*wp).w_cursor.lnum;
    (*wp).w_cursor.lnum = lnum;
    let mut nlnum: linenr_T = 0 as linenr_T;
    if lnum > bot && (*wp).w_botline - (*(*wp).w_buffer).b_ml.ml_line_count != 1 as linenr_T {
        nlnum = bot;
    } else if lnum < top && (*wp).w_topline != 1 as linenr_T {
        nlnum = if so == (*wp).w_view_height / 2 as ::core::ffi::c_int {
            bot
        } else {
            top
        };
    }
    if nlnum != 0 as linenr_T {
        if normal {
            setmark('\'' as ::core::ffi::c_int);
            (*wp).w_cursor.lnum = nlnum;
        } else {
            (*wp).w_fraction = if nlnum == bot {
                FRACTION_MULT
            } else {
                0 as ::core::ffi::c_int
            };
            scroll_to_fraction(wp, (*wp).w_prev_height);
            validate_botline_win(curwin.get());
        }
    }
}
pub unsafe extern "C" fn win_new_height(mut wp: *mut win_T, mut height: ::core::ffi::c_int) {
    height = if height > 0 as ::core::ffi::c_int {
        height
    } else {
        0 as ::core::ffi::c_int
    };
    if (*wp).w_height == height {
        return;
    }
    (*wp).w_height = height;
    (*wp).w_pos_changed = true_0 != 0;
    win_set_inner_size(wp, true_0 != 0);
}
pub unsafe extern "C" fn scroll_to_fraction(
    mut wp: *mut win_T,
    mut prev_height: ::core::ffi::c_int,
) {
    let mut height: ::core::ffi::c_int = (*wp).w_view_height;
    if height > 0 as ::core::ffi::c_int
        && ((*wp).w_onebuf_opt.wo_scb == 0 || wp == curwin.get())
        && ((height as linenr_T) < (*(*wp).w_buffer).b_ml.ml_line_count
            || (*wp).w_topline > 1 as linenr_T)
    {
        let mut lnum: linenr_T = (*wp).w_cursor.lnum;
        lnum = if lnum > 1 as linenr_T {
            lnum
        } else {
            1 as linenr_T
        };
        (*wp).w_wrow = ((*wp).w_fraction * height - 1 as ::core::ffi::c_int) / FRACTION_MULT;
        let mut line_size: ::core::ffi::c_int =
            plines_win_col(wp, lnum, (*wp).w_cursor.col as ::core::ffi::c_long)
                - 1 as ::core::ffi::c_int;
        let mut sline: ::core::ffi::c_int = (*wp).w_wrow - line_size;
        if sline >= 0 as ::core::ffi::c_int {
            let rows: ::core::ffi::c_int = plines_win(wp, lnum, false_0 != 0);
            if sline > (*wp).w_view_height - rows {
                sline = (*wp).w_view_height - rows;
                (*wp).w_wrow -= rows - line_size;
            }
        }
        if sline < 0 as ::core::ffi::c_int {
            (*wp).w_wrow = line_size;
            if (*wp).w_wrow >= (*wp).w_view_height
                && (*wp).w_view_width - win_col_off(wp) > 0 as ::core::ffi::c_int
            {
                (*wp).w_skipcol += (*wp).w_view_width - win_col_off(wp);
                (*wp).w_wrow -= 1;
                while (*wp).w_wrow >= (*wp).w_view_height {
                    (*wp).w_skipcol += (*wp).w_view_width - win_col_off(wp) + win_col_off2(wp);
                    (*wp).w_wrow -= 1;
                }
            }
        } else if sline > 0 as ::core::ffi::c_int {
            while sline > 0 as ::core::ffi::c_int && lnum > 1 as linenr_T {
                hasFolding(wp, lnum, &raw mut lnum, ::core::ptr::null_mut::<linenr_T>());
                if lnum == 1 as linenr_T {
                    line_size = !decor_conceal_line(
                        wp,
                        lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                        false_0 != 0,
                    ) as ::core::ffi::c_int;
                    sline -= 1;
                    break;
                } else {
                    lnum -= 1;
                    if lnum == (*wp).w_topline {
                        line_size = plines_win_nofill(wp, lnum, true_0 != 0) + (*wp).w_topfill;
                    } else {
                        line_size = plines_win(wp, lnum, true_0 != 0);
                    }
                    sline -= line_size;
                }
            }
            if sline < 0 as ::core::ffi::c_int {
                hasFolding(wp, lnum, ::core::ptr::null_mut::<linenr_T>(), &raw mut lnum);
                lnum += 1;
                (*wp).w_wrow -= line_size + sline;
            } else if sline > 0 as ::core::ffi::c_int {
                lnum = 1 as ::core::ffi::c_int as linenr_T;
                (*wp).w_wrow -= sline;
            }
        }
        set_topline(wp, lnum);
    }
    if wp == curwin.get() {
        curs_columns(wp, false_0);
    }
    if prev_height > 0 as ::core::ffi::c_int {
        (*wp).w_prev_fraction_row = (*wp).w_wrow;
    }
    redraw_later(wp, UPD_SOME_VALID as ::core::ffi::c_int);
    invalidate_botline_win(wp);
}
pub unsafe extern "C" fn win_set_inner_size(mut wp: *mut win_T, mut valid_cursor: bool) {
    let mut width: ::core::ffi::c_int = (*wp).w_width_request;
    if width == 0 as ::core::ffi::c_int {
        width = (*wp).w_width;
    }
    let mut prev_height: ::core::ffi::c_int = (*wp).w_view_height;
    let mut height: ::core::ffi::c_int = (*wp).w_height_request;
    if height == 0 as ::core::ffi::c_int {
        height = if 0 as ::core::ffi::c_int > (*wp).w_height - (*wp).w_winbar_height {
            0 as ::core::ffi::c_int
        } else {
            (*wp).w_height - (*wp).w_winbar_height
        };
    }
    if height != prev_height {
        if height > 0 as ::core::ffi::c_int && valid_cursor as ::core::ffi::c_int != 0 {
            if wp == curwin.get()
                && (*p_spk.get() as ::core::ffi::c_int == 'c' as ::core::ffi::c_int
                    || (*wp).w_floating as ::core::ffi::c_int != 0)
            {
                validate_cursor(curwin.get());
            }
            if (*wp).w_view_height != prev_height {
                return;
            }
            if (*wp).w_wrow != (*wp).w_prev_fraction_row {
                set_fraction(wp);
            }
        }
        (*wp).w_view_height = height;
        win_comp_scroll(wp);
        if valid_cursor as ::core::ffi::c_int != 0
            && !exiting.get()
            && (*p_spk.get() as ::core::ffi::c_int == 'c' as ::core::ffi::c_int
                || (*wp).w_floating as ::core::ffi::c_int != 0)
        {
            (*wp).w_skipcol = 0 as ::core::ffi::c_int as colnr_T;
            scroll_to_fraction(wp, prev_height);
        }
        redraw_later(wp, UPD_SOME_VALID as ::core::ffi::c_int);
    }
    if width != (*wp).w_view_width {
        (*wp).w_view_width = width;
        (*wp).w_lines_valid = 0 as ::core::ffi::c_int;
        if valid_cursor {
            changed_line_abv_curs_win(wp);
            invalidate_botline_win(wp);
            if wp == curwin.get()
                && (*p_spk.get() as ::core::ffi::c_int == 'c' as ::core::ffi::c_int
                    || (*wp).w_floating as ::core::ffi::c_int != 0)
            {
                curs_columns(wp, true_0);
            }
        }
        redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
    }
    if !(*(*wp).w_buffer).terminal.is_null() {
        terminal_check_size((*(*wp).w_buffer).terminal);
    }
    let mut float_stl_height: ::core::ffi::c_int =
        if (*wp).w_floating as ::core::ffi::c_int != 0 && (*wp).w_status_height != 0 {
            STATUS_HEIGHT as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
    (*wp).w_height_outer =
        (*wp).w_view_height + win_border_height(wp) + (*wp).w_winbar_height + float_stl_height;
    (*wp).w_width_outer = (*wp).w_view_width + win_border_width(wp);
    (*wp).w_winrow_off =
        (*wp).w_border_adj[0 as ::core::ffi::c_int as usize] + (*wp).w_winbar_height;
    (*wp).w_wincol_off = (*wp).w_border_adj[3 as ::core::ffi::c_int as usize];
    if ui_has(kUIMultigrid) {
        ui_call_win_viewport_margins(
            (*wp).w_grid_alloc.handle as Integer,
            (*wp).handle as Window,
            (*wp).w_winrow_off as Integer,
            (*wp).w_border_adj[2 as ::core::ffi::c_int as usize] as Integer,
            (*wp).w_wincol_off as Integer,
            (*wp).w_border_adj[1 as ::core::ffi::c_int as usize] as Integer,
        );
    }
    (*wp).w_redr_status = true_0 != 0;
}
pub unsafe extern "C" fn win_new_width(mut wp: *mut win_T, mut width: ::core::ffi::c_int) {
    (*wp).w_width = if width < 0 as ::core::ffi::c_int {
        0 as ::core::ffi::c_int
    } else {
        width
    };
    (*wp).w_pos_changed = true_0 != 0;
    win_set_inner_size(wp, true_0 != 0);
}
pub unsafe extern "C" fn win_default_scroll(mut wp: *mut win_T) -> OptInt {
    return (if (*wp).w_view_height / 2 as ::core::ffi::c_int > 1 as ::core::ffi::c_int {
        (*wp).w_view_height / 2 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    }) as OptInt;
}
pub unsafe extern "C" fn win_comp_scroll(mut wp: *mut win_T) {
    let old_w_p_scr: OptInt = (*wp).w_onebuf_opt.wo_scr;
    (*wp).w_onebuf_opt.wo_scr = win_default_scroll(wp);
    if (*wp).w_onebuf_opt.wo_scr != old_w_p_scr {
        (*wp).w_onebuf_opt.wo_script_ctx[kWinOptScroll as ::core::ffi::c_int as usize].sc_sid =
            SID_WINLAYOUT as scid_T;
        (*wp).w_onebuf_opt.wo_script_ctx[kWinOptScroll as ::core::ffi::c_int as usize].sc_lnum =
            0 as ::core::ffi::c_int as linenr_T;
    }
}
pub unsafe extern "C" fn command_height() {
    let mut old_p_ch: ::core::ffi::c_int = (*curtab.get()).tp_ch_used as ::core::ffi::c_int;
    let mut frp: *mut frame_T = (*lastwin_nofloating(::core::ptr::null_mut::<tabpage_T>())).w_frame;
    while (*frp).fr_width != Columns.get() && !(*frp).fr_parent.is_null() {
        frp = (*frp).fr_parent;
    }
    while !(*frp).fr_prev.is_null()
        && (*frp).fr_layout as ::core::ffi::c_int == FR_LEAF
        && (*(*frp).fr_win).w_onebuf_opt.wo_wfh != 0
    {
        frp = (*frp).fr_prev;
    }
    while p_ch.get() > old_p_ch as OptInt && command_frame_height.get() as ::core::ffi::c_int != 0 {
        if frp.is_null() {
            emsg(gettext(&raw const e_noroom as *const ::core::ffi::c_char));
            p_ch.set(old_p_ch as OptInt);
            break;
        } else {
            let mut h: ::core::ffi::c_int = if ((p_ch.get() - old_p_ch as OptInt)
                as ::core::ffi::c_int)
                < (*frp).fr_height - frame_minheight(frp, ::core::ptr::null_mut::<win_T>())
            {
                (p_ch.get() - old_p_ch as OptInt) as ::core::ffi::c_int
            } else {
                (*frp).fr_height - frame_minheight(frp, ::core::ptr::null_mut::<win_T>())
            };
            frame_add_height(frp, -h);
            old_p_ch += h;
            frp = (*frp).fr_prev;
        }
    }
    if p_ch.get() < old_p_ch as OptInt
        && command_frame_height.get() as ::core::ffi::c_int != 0
        && !frp.is_null()
    {
        frame_add_height(frp, (old_p_ch as OptInt - p_ch.get()) as ::core::ffi::c_int);
    }
    win_comp_pos();
    cmdline_row.set(Rows.get() - p_ch.get() as ::core::ffi::c_int);
    redraw_cmdline.set(true_0 != 0);
    if msg_scrolled.get() == 0 as ::core::ffi::c_int && full_screen.get() as ::core::ffi::c_int != 0
    {
        let mut grid: *mut GridView = default_gridview.ptr();
        if !ui_has(kUIMessages) {
            msg_grid_validate();
            grid = msg_grid_adj.ptr();
        }
        grid_clear(
            grid,
            cmdline_row.get(),
            Rows.get(),
            0 as ::core::ffi::c_int,
            Columns.get(),
            0 as ::core::ffi::c_int,
        );
        msg_row.set(cmdline_row.get());
    }
    (*curtab.get()).tp_ch_used = p_ch.get();
    min_set_ch.set(p_ch.get());
}
unsafe extern "C" fn frame_add_height(mut frp: *mut frame_T, mut n: ::core::ffi::c_int) {
    frame_new_height(
        frp,
        (*frp).fr_height + n,
        false_0 != 0,
        false_0 != 0,
        false_0 != 0,
    );
    loop {
        frp = (*frp).fr_parent;
        if frp.is_null() {
            break;
        }
        (*frp).fr_height += n;
    }
}
pub unsafe extern "C" fn last_status(mut morewin: bool) {
    last_status_rec(
        topframe.get(),
        last_stl_height(morewin) > 0 as ::core::ffi::c_int,
        global_stl_height() > 0 as ::core::ffi::c_int,
    );
    win_float_anchor_laststatus();
}
pub unsafe extern "C" fn win_remove_status_line(mut wp: *mut win_T, mut add_hsep: bool) {
    (*wp).w_status_height = 0 as ::core::ffi::c_int;
    if add_hsep {
        (*wp).w_hsep_height = 1 as ::core::ffi::c_int;
    } else {
        win_new_height(
            wp,
            (if (*wp).w_floating as ::core::ffi::c_int != 0 {
                (*wp).w_view_height
            } else {
                (*wp).w_height
            }) + STATUS_HEIGHT as ::core::ffi::c_int,
        );
    }
    comp_col();
    stl_clear_click_defs((*wp).w_status_click_defs, (*wp).w_status_click_defs_size);
    xfree((*wp).w_status_click_defs as *mut ::core::ffi::c_void);
    (*wp).w_status_click_defs_size = 0 as size_t;
    (*wp).w_status_click_defs = ::core::ptr::null_mut::<StlClickDefinition>();
}
unsafe extern "C" fn find_horizontally_resizable_frame(mut fr: *mut frame_T) -> *mut frame_T {
    let mut fp: *mut frame_T = fr;
    while (*fp).fr_height <= frame_minheight(fp, ::core::ptr::null_mut::<win_T>()) {
        if fp == topframe.get() {
            return ::core::ptr::null_mut::<frame_T>();
        }
        if (*(*fp).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL && !(*fp).fr_prev.is_null()
        {
            fp = (*fp).fr_prev;
        } else {
            fp = (*fp).fr_parent;
        }
    }
    return fp;
}
unsafe extern "C" fn resize_frame_for_status(mut fr: *mut frame_T) -> bool {
    let mut wp: *mut win_T = (*fr).fr_win;
    let mut fp: *mut frame_T = find_horizontally_resizable_frame(fr);
    if fp.is_null() {
        emsg(gettext(&raw const e_noroom as *const ::core::ffi::c_char));
        return false_0 != 0;
    } else if fp != fr {
        frame_new_height(
            fp,
            (*fp).fr_height - 1 as ::core::ffi::c_int,
            false_0 != 0,
            false_0 != 0,
            false_0 != 0,
        );
        frame_fix_height(wp);
        win_comp_pos();
    } else {
        win_new_height(wp, (*wp).w_height - 1 as ::core::ffi::c_int);
    }
    return true_0 != 0;
}
unsafe extern "C" fn resize_frame_for_winbar(mut fr: *mut frame_T) -> bool {
    let mut wp: *mut win_T = (*fr).fr_win;
    let mut fp: *mut frame_T = find_horizontally_resizable_frame(fr);
    if fp.is_null() || fp == fr {
        emsg(gettext(&raw const e_noroom as *const ::core::ffi::c_char));
        return false_0 != 0;
    }
    frame_new_height(
        fp,
        (*fp).fr_height - 1 as ::core::ffi::c_int,
        false_0 != 0,
        false_0 != 0,
        false_0 != 0,
    );
    win_new_height(wp, (*wp).w_height + 1 as ::core::ffi::c_int);
    frame_fix_height(wp);
    win_comp_pos();
    return true_0 != 0;
}
unsafe extern "C" fn last_status_rec(
    mut fr: *mut frame_T,
    mut statusline: bool,
    mut is_stl_global: bool,
) {
    if (*fr).fr_layout as ::core::ffi::c_int == FR_LEAF {
        let mut wp: *mut win_T = (*fr).fr_win;
        let mut is_last: bool = is_bottom_win(wp);
        if is_last {
            if (*wp).w_status_height != 0 as ::core::ffi::c_int
                && (!statusline || is_stl_global as ::core::ffi::c_int != 0)
            {
                win_remove_status_line(wp, false_0 != 0);
            } else if (*wp).w_status_height == 0 as ::core::ffi::c_int
                && !is_stl_global
                && statusline as ::core::ffi::c_int != 0
            {
                (*wp).w_status_height = STATUS_HEIGHT as ::core::ffi::c_int;
                if !resize_frame_for_status(fr) {
                    return;
                }
                comp_col();
            }
            if abs((*wp).w_height - (*wp).w_prev_height) == 1 as ::core::ffi::c_int {
                (*wp).w_prev_height = (*wp).w_height;
            }
        } else if (*wp).w_status_height != 0 as ::core::ffi::c_int
            && is_stl_global as ::core::ffi::c_int != 0
        {
            win_remove_status_line(wp, true_0 != 0);
        } else if (*wp).w_status_height == 0 as ::core::ffi::c_int && !is_stl_global {
            (*wp).w_status_height = STATUS_HEIGHT as ::core::ffi::c_int;
            (*wp).w_hsep_height = 0 as ::core::ffi::c_int;
            comp_col();
        }
    } else {
        let mut fp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
        fp = (*fr).fr_child;
        while !fp.is_null() {
            last_status_rec(fp, statusline, is_stl_global);
            fp = (*fp).fr_next;
        }
    };
}
pub unsafe extern "C" fn set_winbar_win(
    mut wp: *mut win_T,
    mut make_room: bool,
    mut valid_cursor: bool,
) -> ::core::ffi::c_int {
    let mut winbar_height: ::core::ffi::c_int = if (*wp).w_floating as ::core::ffi::c_int != 0 {
        if *(*wp).w_onebuf_opt.wo_wbr as ::core::ffi::c_int != NUL {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }
    } else if *p_wbr.get() as ::core::ffi::c_int != NUL
        || *(*wp).w_onebuf_opt.wo_wbr as ::core::ffi::c_int != NUL
    {
        1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    if (*wp).w_winbar_height != winbar_height {
        if winbar_height == 1 as ::core::ffi::c_int
            && (*wp).w_view_height <= 1 as ::core::ffi::c_int
        {
            if (*wp).w_floating {
                emsg(gettext(&raw const e_noroom as *const ::core::ffi::c_char));
                return NOTDONE;
            } else if !make_room || !resize_frame_for_winbar((*wp).w_frame) {
                return FAIL;
            }
        }
        (*wp).w_winbar_height = winbar_height;
        win_set_inner_size(wp, valid_cursor);
        if winbar_height == 0 as ::core::ffi::c_int {
            stl_clear_click_defs((*wp).w_winbar_click_defs, (*wp).w_winbar_click_defs_size);
            xfree((*wp).w_winbar_click_defs as *mut ::core::ffi::c_void);
            (*wp).w_winbar_click_defs_size = 0 as size_t;
            (*wp).w_winbar_click_defs = ::core::ptr::null_mut::<StlClickDefinition>();
        }
    }
    return OK;
}
pub unsafe extern "C" fn set_winbar(mut make_room: bool) {
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if set_winbar_win(wp, make_room, true_0 != 0) == FAIL {
            break;
        }
        wp = (*wp).w_next;
    }
}
pub unsafe extern "C" fn tabline_height() -> ::core::ffi::c_int {
    if ui_has(kUITabline) {
        return 0 as ::core::ffi::c_int;
    }
    '_c2rust_label: {
        if !(*first_tabpage.ptr()).is_null() {
        } else {
            __assert_fail(
                b"first_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                7349 as ::core::ffi::c_uint,
                b"int tabline_height(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    match p_stal.get() {
        0 => return 0 as ::core::ffi::c_int,
        1 => {
            return if (*first_tabpage.get()).tp_next.is_null() {
                0 as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            };
        }
        _ => {}
    }
    return 1 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn global_winbar_height() -> ::core::ffi::c_int {
    return if *p_wbr.get() as ::core::ffi::c_int != NUL {
        1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
}
pub unsafe extern "C" fn global_stl_height() -> ::core::ffi::c_int {
    return if p_ls.get() == 3 as OptInt {
        STATUS_HEIGHT as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
}
pub unsafe extern "C" fn last_stl_height(mut morewin: bool) -> ::core::ffi::c_int {
    return if p_ls.get() > 1 as OptInt
        || p_ls.get() == 1 as OptInt
            && (morewin as ::core::ffi::c_int != 0
                || !one_window(firstwin.get(), ::core::ptr::null_mut::<tabpage_T>()))
    {
        STATUS_HEIGHT as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
}
pub unsafe extern "C" fn min_rows(mut tp: *mut tabpage_T) -> ::core::ffi::c_int {
    if (*firstwin.ptr()).is_null() {
        return MIN_LINES as ::core::ffi::c_int;
    }
    let mut total: ::core::ffi::c_int =
        frame_minheight((*tp).tp_topframe, ::core::ptr::null_mut::<win_T>());
    total += tabline_height() + global_stl_height();
    if (if tp == curtab.get() {
        p_ch.get()
    } else {
        (*tp).tp_ch_used
    }) > 0 as OptInt
    {
        total += 1;
    }
    return total;
}
pub unsafe extern "C" fn min_rows_for_all_tabpages() -> ::core::ffi::c_int {
    if (*firstwin.ptr()).is_null() {
        return MIN_LINES as ::core::ffi::c_int;
    }
    let mut total: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut n: ::core::ffi::c_int =
            frame_minheight((*tp).tp_topframe, ::core::ptr::null_mut::<win_T>());
        if (if tp == curtab.get() {
            p_ch.get()
        } else {
            (*tp).tp_ch_used
        }) > 0 as OptInt
        {
            n += 1;
        }
        total = if total > n { total } else { n };
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    total += tabline_height() + global_stl_height();
    return total;
}
pub unsafe extern "C" fn only_one_window() -> bool {
    if !(*first_tabpage.get()).tp_next.is_null() {
        return false_0 != 0;
    }
    let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if !(*wp).w_buffer.is_null()
            && (!(bt_help((*wp).w_buffer) as ::core::ffi::c_int != 0 && !bt_help(curbuf.get())
                || (*wp).w_floating as ::core::ffi::c_int != 0
                || (*wp).w_onebuf_opt.wo_pvw != 0)
                || wp == curwin.get())
            && !is_aucmd_win(wp)
        {
            count += 1;
        }
        wp = (*wp).w_next;
    }
    return count <= 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn check_lnums_both(mut do_curwin: bool, mut nested: bool) {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if (do_curwin as ::core::ffi::c_int != 0 || wp != curwin.get())
                && (*wp).w_buffer == curbuf.get()
            {
                if !nested {
                    (*wp).w_save_cursor.w_cursor_save = (*wp).w_cursor;
                    (*wp).w_save_cursor.w_topline_save = (*wp).w_topline as ::core::ffi::c_int;
                }
                let mut need_adjust: bool =
                    (*wp).w_cursor.lnum > (*curbuf.get()).b_ml.ml_line_count;
                if need_adjust {
                    (*wp).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
                }
                if need_adjust as ::core::ffi::c_int != 0 || !nested {
                    (*wp).w_save_cursor.w_cursor_corr = (*wp).w_cursor;
                }
                need_adjust = (*wp).w_topline > (*curbuf.get()).b_ml.ml_line_count;
                if need_adjust {
                    (*wp).w_topline = (*curbuf.get()).b_ml.ml_line_count;
                }
                if need_adjust as ::core::ffi::c_int != 0 || !nested {
                    (*wp).w_save_cursor.w_topline_corr = (*wp).w_topline as ::core::ffi::c_int;
                }
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
pub unsafe extern "C" fn check_lnums(mut do_curwin: bool) {
    check_lnums_both(do_curwin, false_0 != 0);
}
pub unsafe extern "C" fn check_lnums_nested(mut do_curwin: bool) {
    check_lnums_both(do_curwin, true_0 != 0);
}
pub unsafe extern "C" fn reset_lnums() {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == curbuf.get() {
                if equalpos((*wp).w_save_cursor.w_cursor_corr, (*wp).w_cursor) as ::core::ffi::c_int
                    != 0
                    && (*wp).w_save_cursor.w_cursor_save.lnum != 0 as linenr_T
                {
                    (*wp).w_cursor = (*wp).w_save_cursor.w_cursor_save;
                }
                if (*wp).w_save_cursor.w_topline_corr as linenr_T == (*wp).w_topline
                    && (*wp).w_save_cursor.w_topline_save != 0 as ::core::ffi::c_int
                {
                    (*wp).w_topline = (*wp).w_save_cursor.w_topline_save as linenr_T;
                }
                if (*wp).w_save_cursor.w_topline_save as linenr_T
                    > (*(*wp).w_buffer).b_ml.ml_line_count
                {
                    (*wp).w_valid &= !VALID_TOPLINE;
                }
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
pub unsafe extern "C" fn make_snapshot(mut idx: ::core::ffi::c_int) {
    clear_snapshot(curtab.get(), idx);
    make_snapshot_rec(
        topframe.get(),
        (&raw mut (*curtab.get()).tp_snapshot as *mut *mut frame_T).offset(idx as isize)
            as *mut *mut frame_T,
    );
}
unsafe extern "C" fn make_snapshot_rec(mut fr: *mut frame_T, mut frp: *mut *mut frame_T) {
    *frp = xcalloc(1 as size_t, ::core::mem::size_of::<frame_T>()) as *mut frame_T;
    (**frp).fr_layout = (*fr).fr_layout;
    (**frp).fr_width = (*fr).fr_width;
    (**frp).fr_height = (*fr).fr_height;
    if !(*fr).fr_next.is_null() {
        make_snapshot_rec((*fr).fr_next, &raw mut (**frp).fr_next);
    }
    if !(*fr).fr_child.is_null() {
        make_snapshot_rec((*fr).fr_child, &raw mut (**frp).fr_child);
    }
    if (*fr).fr_layout as ::core::ffi::c_int == FR_LEAF && (*fr).fr_win == curwin.get() {
        (**frp).fr_win = curwin.get();
    }
}
unsafe extern "C" fn clear_snapshot(mut tp: *mut tabpage_T, mut idx: ::core::ffi::c_int) {
    clear_snapshot_rec((*tp).tp_snapshot[idx as usize] as *mut frame_T);
    (*tp).tp_snapshot[idx as usize] = ::core::ptr::null_mut::<frame_T>();
}
unsafe extern "C" fn clear_snapshot_rec(mut fr: *mut frame_T) {
    if fr.is_null() {
        return;
    }
    clear_snapshot_rec((*fr).fr_next);
    clear_snapshot_rec((*fr).fr_child);
    xfree(fr as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn get_snapshot_curwin_rec(mut ft: *mut frame_T) -> *mut win_T {
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    if !(*ft).fr_next.is_null() {
        wp = get_snapshot_curwin_rec((*ft).fr_next);
        if !wp.is_null() {
            return wp;
        }
    }
    if !(*ft).fr_child.is_null() {
        wp = get_snapshot_curwin_rec((*ft).fr_child);
        if !wp.is_null() {
            return wp;
        }
    }
    return (*ft).fr_win;
}
unsafe extern "C" fn get_snapshot_curwin(mut idx: ::core::ffi::c_int) -> *mut win_T {
    if (*curtab.get()).tp_snapshot[idx as usize].is_null() {
        return ::core::ptr::null_mut::<win_T>();
    }
    return get_snapshot_curwin_rec((*curtab.get()).tp_snapshot[idx as usize] as *mut frame_T);
}
pub unsafe extern "C" fn restore_snapshot(
    mut idx: ::core::ffi::c_int,
    mut close_curwin: ::core::ffi::c_int,
) {
    if !(*curtab.get()).tp_snapshot[idx as usize].is_null()
        && (*(*curtab.get()).tp_snapshot[idx as usize]).fr_width == (*topframe.get()).fr_width
        && (*(*curtab.get()).tp_snapshot[idx as usize]).fr_height == (*topframe.get()).fr_height
        && check_snapshot_rec(
            (*curtab.get()).tp_snapshot[idx as usize] as *mut frame_T,
            topframe.get(),
        ) == OK
    {
        let mut wp: *mut win_T = restore_snapshot_rec(
            (*curtab.get()).tp_snapshot[idx as usize] as *mut frame_T,
            topframe.get(),
        );
        win_comp_pos();
        if !wp.is_null() && close_curwin != 0 {
            win_goto(wp);
        }
        redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
    }
    clear_snapshot(curtab.get(), idx);
}
unsafe extern "C" fn check_snapshot_rec(
    mut sn: *mut frame_T,
    mut fr: *mut frame_T,
) -> ::core::ffi::c_int {
    if (*sn).fr_layout as ::core::ffi::c_int != (*fr).fr_layout as ::core::ffi::c_int
        || (*sn).fr_next.is_null() as ::core::ffi::c_int
            != (*fr).fr_next.is_null() as ::core::ffi::c_int
        || (*sn).fr_child.is_null() as ::core::ffi::c_int
            != (*fr).fr_child.is_null() as ::core::ffi::c_int
        || !(*sn).fr_next.is_null() && check_snapshot_rec((*sn).fr_next, (*fr).fr_next) == FAIL
        || !(*sn).fr_child.is_null() && check_snapshot_rec((*sn).fr_child, (*fr).fr_child) == FAIL
        || !(*sn).fr_win.is_null() && !win_valid((*sn).fr_win)
    {
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn restore_snapshot_rec(
    mut sn: *mut frame_T,
    mut fr: *mut frame_T,
) -> *mut win_T {
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    (*fr).fr_height = (*sn).fr_height;
    (*fr).fr_width = (*sn).fr_width;
    if (*fr).fr_layout as ::core::ffi::c_int == FR_LEAF {
        frame_new_height(
            fr,
            (*fr).fr_height,
            false_0 != 0,
            false_0 != 0,
            false_0 != 0,
        );
        frame_new_width(fr, (*fr).fr_width, false_0 != 0, false_0 != 0);
        wp = (*sn).fr_win;
    }
    if !(*sn).fr_next.is_null() {
        let mut wp2: *mut win_T = restore_snapshot_rec((*sn).fr_next, (*fr).fr_next);
        if !wp2.is_null() {
            wp = wp2;
        }
    }
    if !(*sn).fr_child.is_null() {
        let mut wp2_0: *mut win_T = restore_snapshot_rec((*sn).fr_child, (*fr).fr_child);
        if !wp2_0.is_null() {
            wp = wp2_0;
        }
    }
    return wp;
}
unsafe extern "C" fn frame_check_height(
    mut topfrp: *const frame_T,
    mut height: ::core::ffi::c_int,
) -> bool {
    if (*topfrp).fr_height != height {
        return false_0 != 0;
    }
    if (*topfrp).fr_layout as ::core::ffi::c_int == FR_ROW {
        let mut frp: *const frame_T = ::core::ptr::null::<frame_T>();
        frp = (*topfrp).fr_child;
        while !frp.is_null() {
            if (*frp).fr_height != height {
                return false_0 != 0;
            }
            frp = (*frp).fr_next;
        }
    }
    return true_0 != 0;
}
unsafe extern "C" fn frame_check_width(
    mut topfrp: *const frame_T,
    mut width: ::core::ffi::c_int,
) -> bool {
    if (*topfrp).fr_width != width {
        return false_0 != 0;
    }
    if (*topfrp).fr_layout as ::core::ffi::c_int == FR_COL {
        let mut frp: *const frame_T = ::core::ptr::null::<frame_T>();
        frp = (*topfrp).fr_child;
        while !frp.is_null() {
            if (*frp).fr_width != width {
                return false_0 != 0;
            }
            frp = (*frp).fr_next;
        }
    }
    return true_0 != 0;
}
unsafe extern "C" fn int_cmp(
    mut pa: *const ::core::ffi::c_void,
    mut pb: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let a: ::core::ffi::c_int = *(pa as *const ::core::ffi::c_int);
    let b: ::core::ffi::c_int = *(pb as *const ::core::ffi::c_int);
    return if a == b {
        0 as ::core::ffi::c_int
    } else if a < b {
        -1 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    };
}
pub unsafe extern "C" fn check_colorcolumn(
    mut cc: *mut ::core::ffi::c_char,
    mut wp: *mut win_T,
) -> *const ::core::ffi::c_char {
    if !wp.is_null() && (*wp).w_buffer.is_null() {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    let mut s: *mut ::core::ffi::c_char = empty_string_option.ptr() as *mut ::core::ffi::c_char;
    if !cc.is_null() {
        s = cc;
    } else if !wp.is_null() {
        s = (*wp).w_onebuf_opt.wo_cc;
    }
    let mut tw: OptInt = 0;
    if !wp.is_null() {
        tw = (*(*wp).w_buffer).b_p_tw;
    } else {
        tw = 0 as OptInt;
    }
    let mut count: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut color_cols: [::core::ffi::c_int; 256] = [0; 256];
    while *s as ::core::ffi::c_int != NUL && count < 255 as ::core::ffi::c_uint {
        let mut col: ::core::ffi::c_int = 0;
        '_skip: {
            if *s as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                || *s as ::core::ffi::c_int == '+' as ::core::ffi::c_int
            {
                col = if *s as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
                    -1 as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                };
                s = s.offset(1);
                if !ascii_isdigit(*s as ::core::ffi::c_int) {
                    return &raw const e_invarg as *const ::core::ffi::c_char;
                }
                col = col * getdigits_int(&raw mut s, true_0 != 0, 0 as ::core::ffi::c_int);
                if tw == 0 as OptInt {
                    break '_skip;
                } else {
                    '_c2rust_label: {
                        if col >= 0 as ::core::ffi::c_int
                            && tw <= (2147483647 as ::core::ffi::c_int - col) as OptInt
                            && tw + col as OptInt
                                >= (-2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                                    as OptInt
                            || col < 0 as ::core::ffi::c_int
                                && tw
                                    >= (-2147483647 as ::core::ffi::c_int
                                        - 1 as ::core::ffi::c_int
                                        - col) as OptInt
                                && tw + col as OptInt <= 2147483647 as OptInt
                        {
                        } else {
                            __assert_fail(
                                b"(col >= 0 && tw <= INT_MAX - col && tw + col >= INT_MIN) || (col < 0 && tw >= INT_MIN - col && tw + col <= INT_MAX)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/window.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                7748 as ::core::ffi::c_uint,
                                b"const char *check_colorcolumn(char *, win_T *)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    col += tw as ::core::ffi::c_int;
                    if col < 0 as ::core::ffi::c_int {
                        break '_skip;
                    }
                }
            } else if ascii_isdigit(*s as ::core::ffi::c_int) {
                col = getdigits_int(&raw mut s, true_0 != 0, 0 as ::core::ffi::c_int);
            } else {
                return &raw const e_invarg as *const ::core::ffi::c_char;
            }
            let c2rust_fresh8 = count;
            count = count.wrapping_add(1);
            color_cols[c2rust_fresh8 as usize] = col - 1 as ::core::ffi::c_int;
        }
        if *s as ::core::ffi::c_int == NUL {
            break;
        }
        if *s as ::core::ffi::c_int != ',' as ::core::ffi::c_int {
            return &raw const e_invarg as *const ::core::ffi::c_char;
        }
        s = s.offset(1);
        if *s as ::core::ffi::c_int == NUL {
            return &raw const e_invarg as *const ::core::ffi::c_char;
        }
    }
    if wp.is_null() {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    xfree((*wp).w_p_cc_cols as *mut ::core::ffi::c_void);
    if count == 0 as ::core::ffi::c_uint {
        (*wp).w_p_cc_cols = ::core::ptr::null_mut::<::core::ffi::c_int>();
    } else {
        (*wp).w_p_cc_cols = xmalloc(
            ::core::mem::size_of::<::core::ffi::c_int>()
                .wrapping_mul(count.wrapping_add(1 as ::core::ffi::c_uint) as size_t),
        ) as *mut ::core::ffi::c_int;
        qsort(
            &raw mut color_cols as *mut ::core::ffi::c_int as *mut ::core::ffi::c_void,
            count as size_t,
            ::core::mem::size_of::<::core::ffi::c_int>(),
            Some(
                int_cmp
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
        while i < count {
            if j == 0 as ::core::ffi::c_int
                || *(*wp)
                    .w_p_cc_cols
                    .offset((j - 1 as ::core::ffi::c_int) as isize)
                    != color_cols[i as usize]
            {
                let c2rust_fresh9 = j;
                j = j + 1;
                *(*wp).w_p_cc_cols.offset(c2rust_fresh9 as isize) = color_cols[i as usize];
            }
            i = i.wrapping_add(1);
        }
        *(*wp).w_p_cc_cols.offset(j as isize) = -1 as ::core::ffi::c_int;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn get_last_winid() -> ::core::ffi::c_int {
    return last_win_id.get();
}
pub unsafe extern "C" fn win_locked(mut wp: *mut win_T) -> ::core::ffi::c_int {
    return (*wp).w_locked as ::core::ffi::c_int;
}
pub unsafe extern "C" fn win_get_tabwin(
    mut id: handle_T,
    mut tabnr: *mut ::core::ffi::c_int,
    mut winnr: *mut ::core::ffi::c_int,
) {
    *tabnr = 0 as ::core::ffi::c_int;
    *winnr = 0 as ::core::ffi::c_int;
    let mut tnum: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut wnum: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).handle == id {
                if win_has_winnr(wp, tp as *mut tabpage_T) {
                    *winnr = wnum;
                    *tabnr = tnum;
                }
                return;
            }
            wnum += win_has_winnr(wp, tp as *mut tabpage_T) as ::core::ffi::c_int;
            wp = (*wp).w_next;
        }
        tnum += 1;
        wnum = 1 as ::core::ffi::c_int;
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
pub unsafe extern "C" fn win_ui_flush(mut validate: bool) {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if ((*wp).w_pos_changed as ::core::ffi::c_int != 0
                || (*wp).w_grid_alloc.pending_comp_index_update as ::core::ffi::c_int != 0)
                && !(*wp).w_grid_alloc.chars.is_null()
            {
                if tp == curtab.get() {
                    ui_ext_win_position(wp, validate);
                } else {
                    ui_call_win_hide((*wp).w_grid_alloc.handle as Integer);
                    (*wp).w_pos_changed = false_0 != 0;
                }
                (*wp).w_grid_alloc.pending_comp_index_update = false_0 != 0;
            }
            if tp == curtab.get() {
                ui_ext_win_viewport(wp);
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    pum_ui_flush();
    msg_ui_flush();
}
pub unsafe extern "C" fn lastwin_nofloating(mut tp: *mut tabpage_T) -> *mut win_T {
    '_c2rust_label: {
        if tp != curtab.get() || tp.is_null() {
        } else {
            __assert_fail(
                b"tp != curtab || !tp\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/window.rs\0".as_ptr() as *const ::core::ffi::c_char,
                7858 as ::core::ffi::c_uint,
                b"win_T *lastwin_nofloating(tabpage_T *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut res: *mut win_T = if !tp.is_null() {
        (*tp).tp_lastwin
    } else {
        lastwin.get()
    };
    while (*res).w_floating {
        res = (*res).w_prev;
    }
    return res;
}
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
