use crate::src::nvim::buffer::col_print;
use crate::src::nvim::change::{
    appended_lines_mark, changed_bytes, changed_lines, del_bytes, del_char, del_lines,
    get_last_leader_offset, get_leader_len, ins_char, ins_str, truncate_line,
};
use crate::src::nvim::charset::{getwhitecols, getwhitecols_curline, skipwhite, vim_str2nr};
use crate::src::nvim::cursor::{
    check_cursor, check_cursor_col, check_pos, coladvance, coladvance_force, dec_cursor,
    gchar_cursor, get_cursor_line_len, get_cursor_line_ptr, get_cursor_pos_ptr, getviscol,
    getviscol2, getvpos, inc_cursor,
};
use crate::src::nvim::drawscreen::{redraw_curbuf_later, update_screen};
use crate::src::nvim::edit::{beginline, display_dollar, edit};
use crate::src::nvim::eval::typval::{tv_clear, tv_dict_add_nr};
use crate::src::nvim::eval_1::{callback_call, set_ref_in_callback};
use crate::src::nvim::extmark::{extmark_splice, extmark_splice_cols};
use crate::src::nvim::fold::{deleteFold, foldCreate, foldOpenCursor, hasFolding, opFoldRange};
use crate::src::nvim::getchar::{
    beep_flush, stuffReadbuff, stuffcharReadbuff, stuffnumReadbuff, AppendNumberToRedobuff,
    AppendToRedobuff, AppendToRedobuffLit, AppendToRedobuffSpec, CancelRedo, ResetRedobuff,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::indent::{
    change_indent, fix_indent, get_expr_indent, get_indent, get_lisp_indent, get_sw_value_indent,
    inindent, may_do_si, op_reindent, preprocs_left, set_indent, tabstop_fromto,
    use_indentexpr_for_lisp,
};
use crate::src::nvim::indent_c::get_c_indent;
use crate::src::nvim::log::logmsg;
use crate::src::nvim::main::{
    ai_col, bangredo, can_si, cmdmod, curbuf, curbuf_splice_pending, curwin, did_ai,
    disable_fold_update, e_invarg, e_modifiable, empty_string_option, finish_op, got_int,
    motion_force, mouse_dragging, msg_scroll, no_lines_msg, p_ch, p_cpo, p_fp, p_js, p_opfunc,
    p_report, p_ri, p_sbr, p_sel, p_shm, p_sol, p_sr, redo_VIsual_busy, repeat_cmdline,
    repeat_luaref, resel_VIsual_line_count, resel_VIsual_mode, resel_VIsual_vcol, restart_edit,
    virtual_op, IObuff, Insstart, KeyTyped, State, VIsual, VIsual_active, VIsual_mode,
    VIsual_reselect, VIsual_select, VIsual_select_reg,
};
use crate::src::nvim::mark::{mark_col_adjust, mark_mb_adjustpos};
use crate::src::nvim::mbyte::{
    bomb_size, mb_islower, mb_isupper, mb_tolower, mb_toupper, utf8len_tab, utf_char2bytes,
    utf_char2cells, utf_char2len, utf_eat_space, utf_head_off, utf_ptr2CharInfo_impl, utf_ptr2char,
    utf_ptr2len, utfc_next_impl, utfc_ptr2len,
};
use crate::src::nvim::memline::{
    dec, decl, gchar_pos, inc, ml_append, ml_get, ml_get_buf_len, ml_get_buf_mut, ml_get_len,
    ml_get_pos, ml_get_pos_len, ml_replace, ml_replace_len,
};
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc, xmallocz, xmemcpyz, xmemdupz};
use crate::src::nvim::message::{emsg, internal_error, msg, msg_keep, msg_start, msgmore, smsg};
use crate::src::nvim::mouse::setmouse;
use crate::src::nvim::normal::{
    clearop, clearopbeep, may_clear_cmdline, prep_redo, prep_redo_num2, restore_visual_mode,
    unadjust_for_sel,
};
use crate::src::nvim::option::{
    get_equalprg, get_fileformat, get_ve_flags, option_set_callback_func,
};
use crate::src::nvim::os::input::{line_breakcheck, os_breakcheck};
use crate::src::nvim::os::libc::{
    __assert_fail, __ctype_b_loc, abort, gettext, memmove, memset, ngettext, strcpy, strlen,
};
use crate::src::nvim::plines::{
    charsize_fast, charsize_regular, getvcol, getvcols, getvvcol, init_charsize_arg,
    linetabsize_col,
};
use crate::src::nvim::r#move::validate_virtcol;
use crate::src::nvim::register::{
    do_autocmd_textyankpost, get_y_register, get_yank_register, op_yank, op_yank_reg,
    shift_delete_registers, valid_yank_reg,
};
use crate::src::nvim::state::virtual_active;
use crate::src::nvim::strings::{vim_snprintf, vim_strchr};
use crate::src::nvim::textformat::{auto_format, has_format_option, op_format, op_formatexpr};
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, CSType, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem, CharInfo, CharSize, CharsizeArg,
    DecorExt, DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_2, ExtmarkMove, ExtmarkOp, ExtmarkSavePos, ExtmarkSplice,
    ExtmarkUndoObject, FileID, FloatAnchor, FloatRelative, GridView, Indenter, Intersection,
    LuaRef, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t,
    Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, MarkTreeIter,
    MarkTreeIter_s as C2Rust_Unnamed_15, MotionType, OptIndex, OptInt, OptValData,
    ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t,
    SpecialVarValue, StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_13,
    StrCharInfo, String_0, Terminal, Timestamp, TriState, UndoObjectType, VarLockStatus, VarType,
    VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle,
    Window, __time_t, alist_T, bcount_t, bhdr_T, blob_T, blobvar_S, block_def, blocknr_T, buf_T,
    bufstate_T, chunksize_T, cmdarg_T, cmdmod_T, colnr_T, dict_T, dictvar_S, disptick_T,
    extmark_undo_vec_t, fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_3,
    file_buffer_b_wininfo as C2Rust_Unnamed_12, file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, ht_stack_S, ht_stack_T, infoptr_T, int16_t, int32_t, int64_t, intptr_t,
    key_extra, lcs_chars_T, linenr_T, list_T, list_stack_S, list_stack_T, listitem_S, listitem_T,
    listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T, mapblock, mapblock_T, match_T, matchitem,
    matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s, mtnode_s, oparg_T, optset_T,
    partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S, qf_info_T,
    queue, reg_extmatch_T, regmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T,
    sctx_T, size_t, ssize_t, syn_state, syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T,
    synblock_T, synstate_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry,
    u_entry_T, u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_9,
    u_header_uh_alt_prev as C2Rust_Unnamed_8, u_header_uh_next as C2Rust_Unnamed_11,
    u_header_uh_prev as C2Rust_Unnamed_10, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    uintptr_t, undo_object, undo_object_data as C2Rust_Unnamed_7, uvarnumber_T, varnumber_T,
    virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T, yankreg_T,
    QUEUE,
};
use crate::src::nvim::ui::vim_beep;
use crate::src::nvim::undo::{u_clearline, u_save, u_save_cursor};
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
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const NUMBUFLEN: C2Rust_Unnamed_16 = 65;
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
pub const kExtmarkUndoNoRedo: ExtmarkOp = 3;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
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
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const kOptBoFlagWildmode: C2Rust_Unnamed_18 = 524288;
pub const kOptBoFlagTerm: C2Rust_Unnamed_18 = 262144;
pub const kOptBoFlagSpell: C2Rust_Unnamed_18 = 131072;
pub const kOptBoFlagShell: C2Rust_Unnamed_18 = 65536;
pub const kOptBoFlagRegister: C2Rust_Unnamed_18 = 32768;
pub const kOptBoFlagOperator: C2Rust_Unnamed_18 = 16384;
pub const kOptBoFlagShowmatch: C2Rust_Unnamed_18 = 8192;
pub const kOptBoFlagMess: C2Rust_Unnamed_18 = 4096;
pub const kOptBoFlagLang: C2Rust_Unnamed_18 = 2048;
pub const kOptBoFlagInsertmode: C2Rust_Unnamed_18 = 1024;
pub const kOptBoFlagHangul: C2Rust_Unnamed_18 = 512;
pub const kOptBoFlagEx: C2Rust_Unnamed_18 = 256;
pub const kOptBoFlagEsc: C2Rust_Unnamed_18 = 128;
pub const kOptBoFlagError: C2Rust_Unnamed_18 = 64;
pub const kOptBoFlagCtrlg: C2Rust_Unnamed_18 = 32;
pub const kOptBoFlagCopy: C2Rust_Unnamed_18 = 16;
pub const kOptBoFlagComplete: C2Rust_Unnamed_18 = 8;
pub const kOptBoFlagCursor: C2Rust_Unnamed_18 = 4;
pub const kOptBoFlagBackspace: C2Rust_Unnamed_18 = 2;
pub const kOptBoFlagAll: C2Rust_Unnamed_18 = 1;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const kOptVeFlagNoneU: C2Rust_Unnamed_19 = 32;
pub const kOptVeFlagNone: C2Rust_Unnamed_19 = 16;
pub const kOptVeFlagOnemore: C2Rust_Unnamed_19 = 8;
pub const kOptVeFlagInsert: C2Rust_Unnamed_19 = 6;
pub const kOptVeFlagBlock: C2Rust_Unnamed_19 = 5;
pub const kOptVeFlagAll: C2Rust_Unnamed_19 = 4;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const STR2NR_QUOTE: C2Rust_Unnamed_20 = 16;
pub const STR2NR_NO_OCT: C2Rust_Unnamed_20 = 13;
pub const STR2NR_ALL: C2Rust_Unnamed_20 = 15;
pub const STR2NR_FORCE: C2Rust_Unnamed_20 = 128;
pub const STR2NR_OOCT: C2Rust_Unnamed_20 = 8;
pub const STR2NR_HEX: C2Rust_Unnamed_20 = 4;
pub const STR2NR_OCT: C2Rust_Unnamed_20 = 2;
pub const STR2NR_BIN: C2Rust_Unnamed_20 = 1;
pub const STR2NR_DEC: C2Rust_Unnamed_20 = 0;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const CA_NO_ADJ_OP_END: C2Rust_Unnamed_21 = 2;
pub const CA_COMMAND_BUSY: C2Rust_Unnamed_21 = 1;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_int;
pub const REPLACE_NL_NCHAR: C2Rust_Unnamed_22 = -2;
pub const REPLACE_CR_NCHAR: C2Rust_Unnamed_22 = -1;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const YREG_PUT: C2Rust_Unnamed_23 = 2;
pub const YREG_YANK: C2Rust_Unnamed_23 = 1;
pub const YREG_PASTE: C2Rust_Unnamed_23 = 0;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_24 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_24 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_24 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_24 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_24 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_24 = 20;
pub const UPD_VALID: C2Rust_Unnamed_24 = 10;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const INDENT_DEC: C2Rust_Unnamed_25 = 3;
pub const INDENT_INC: C2Rust_Unnamed_25 = 2;
pub const INDENT_SET: C2Rust_Unnamed_25 = 1;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_26 = 4;
pub const BL_SOL: C2Rust_Unnamed_26 = 2;
pub const BL_WHITE: C2Rust_Unnamed_26 = 1;
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
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const SIN_NOMARK: C2Rust_Unnamed_28 = 8;
pub const SIN_UNDO: C2Rust_Unnamed_28 = 4;
pub const SIN_INSERT: C2Rust_Unnamed_28 = 2;
pub const SIN_CHANGED: C2Rust_Unnamed_28 = 1;
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
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const OP_NR_SUB: C2Rust_Unnamed_29 = 29;
pub const OP_NR_ADD: C2Rust_Unnamed_29 = 28;
pub const OP_FUNCTION: C2Rust_Unnamed_29 = 27;
pub const OP_FORMAT2: C2Rust_Unnamed_29 = 26;
pub const OP_FOLDDELREC: C2Rust_Unnamed_29 = 25;
pub const OP_FOLDDEL: C2Rust_Unnamed_29 = 24;
pub const OP_FOLDCLOSEREC: C2Rust_Unnamed_29 = 23;
pub const OP_FOLDCLOSE: C2Rust_Unnamed_29 = 22;
pub const OP_FOLDOPENREC: C2Rust_Unnamed_29 = 21;
pub const OP_FOLDOPEN: C2Rust_Unnamed_29 = 20;
pub const OP_FOLD: C2Rust_Unnamed_29 = 19;
pub const OP_APPEND: C2Rust_Unnamed_29 = 18;
pub const OP_INSERT: C2Rust_Unnamed_29 = 17;
pub const OP_REPLACE: C2Rust_Unnamed_29 = 16;
pub const OP_ROT13: C2Rust_Unnamed_29 = 15;
pub const OP_JOIN_NS: C2Rust_Unnamed_29 = 14;
pub const OP_JOIN: C2Rust_Unnamed_29 = 13;
pub const OP_LOWER: C2Rust_Unnamed_29 = 12;
pub const OP_UPPER: C2Rust_Unnamed_29 = 11;
pub const OP_COLON: C2Rust_Unnamed_29 = 10;
pub const OP_FORMAT: C2Rust_Unnamed_29 = 9;
pub const OP_INDENT: C2Rust_Unnamed_29 = 8;
pub const OP_TILDE: C2Rust_Unnamed_29 = 7;
pub const OP_FILTER: C2Rust_Unnamed_29 = 6;
pub const OP_RSHIFT: C2Rust_Unnamed_29 = 5;
pub const OP_LSHIFT: C2Rust_Unnamed_29 = 4;
pub const OP_CHANGE: C2Rust_Unnamed_29 = 3;
pub const OP_YANK: C2Rust_Unnamed_29 = 2;
pub const OP_DELETE: C2Rust_Unnamed_29 = 1;
pub const OP_NOP: C2Rust_Unnamed_29 = 0;
pub const kCharsizeFast: C2Rust_Unnamed_30 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct redo_VIsual_T {
    pub rv_mode: ::core::ffi::c_int,
    pub rv_line_count: linenr_T,
    pub rv_vcol: colnr_T,
    pub rv_count: ::core::ffi::c_int,
    pub rv_arg: ::core::ffi::c_int,
}
pub type C2Rust_Unnamed_30 = ::core::ffi::c_uint;
pub const kCharsizeRegular: C2Rust_Unnamed_30 = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const VALID_WROW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VALID_WCOL: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VALID_VIRTCOL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
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
unsafe extern "C" fn ltoreq(mut a: pos_T, mut b: pos_T) -> bool {
    return lt(a, b) as ::core::ffi::c_int != 0 || equalpos(a, b) as ::core::ffi::c_int != 0;
}
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn buf_get_changedtick(buf: *const buf_T) -> varnumber_T {
    return (*buf).changedtick_di.di_tv.vval.v_number;
}
pub const EOL_DOS: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FO_MBYTE_JOIN: ::core::ffi::c_int = 'M' as ::core::ffi::c_int;
pub const FO_MBYTE_JOIN2: ::core::ffi::c_int = 'B' as ::core::ffi::c_int;
pub const FO_AUTO: ::core::ffi::c_int = 'a' as ::core::ffi::c_int;
pub const FO_REMOVE_COMS: ::core::ffi::c_int = 'j' as ::core::ffi::c_int;
pub const CPO_EMPTYREGION: ::core::ffi::c_int = 'E' as ::core::ffi::c_int;
pub const CPO_JOINCOL: ::core::ffi::c_int = 'q' as ::core::ffi::c_int;
pub const CPO_REDO: ::core::ffi::c_int = 'r' as ::core::ffi::c_int;
pub const CPO_YANK: ::core::ffi::c_int = 'y' as ::core::ffi::c_int;
pub const CPO_DOLLAR: ::core::ffi::c_int = '$' as ::core::ffi::c_int;
pub const CPO_FILTER: ::core::ffi::c_int = '!' as ::core::ffi::c_int;
pub const COM_END: ::core::ffi::c_int = 'e' as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const NL_STR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"\n\0") };
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const Ctrl_A: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const Ctrl_X: ::core::ffi::c_int = 24 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
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
unsafe extern "C" fn ascii_isbdigit(mut c: ::core::ffi::c_int) -> bool {
    return c == '0' as ::core::ffi::c_int || c == '1' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isspace(mut c: ::core::ffi::c_int) -> bool {
    return c >= 9 as ::core::ffi::c_int && c <= 13 as ::core::ffi::c_int
        || c == ' ' as ::core::ffi::c_int;
}
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
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
pub const OPF_LINES: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const OPF_CHANGE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
static opchars: GlobalCell<[[::core::ffi::c_char; 3]; 30]> = GlobalCell::new([
    [
        NUL as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        0 as ::core::ffi::c_char,
    ],
    [
        'd' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        OPF_CHANGE as ::core::ffi::c_char,
    ],
    [
        'y' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        0 as ::core::ffi::c_char,
    ],
    [
        'c' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        OPF_CHANGE as ::core::ffi::c_char,
    ],
    [
        '<' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        (OPF_LINES | OPF_CHANGE) as ::core::ffi::c_char,
    ],
    [
        '>' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        (OPF_LINES | OPF_CHANGE) as ::core::ffi::c_char,
    ],
    [
        '!' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        (OPF_LINES | OPF_CHANGE) as ::core::ffi::c_char,
    ],
    [
        'g' as ::core::ffi::c_char,
        '~' as ::core::ffi::c_char,
        OPF_CHANGE as ::core::ffi::c_char,
    ],
    [
        '=' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        (OPF_LINES | OPF_CHANGE) as ::core::ffi::c_char,
    ],
    [
        'g' as ::core::ffi::c_char,
        'q' as ::core::ffi::c_char,
        (OPF_LINES | OPF_CHANGE) as ::core::ffi::c_char,
    ],
    [
        ':' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        OPF_LINES as ::core::ffi::c_char,
    ],
    [
        'g' as ::core::ffi::c_char,
        'U' as ::core::ffi::c_char,
        OPF_CHANGE as ::core::ffi::c_char,
    ],
    [
        'g' as ::core::ffi::c_char,
        'u' as ::core::ffi::c_char,
        OPF_CHANGE as ::core::ffi::c_char,
    ],
    [
        'J' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        (OPF_LINES | OPF_CHANGE) as ::core::ffi::c_char,
    ],
    [
        'g' as ::core::ffi::c_char,
        'J' as ::core::ffi::c_char,
        (OPF_LINES | OPF_CHANGE) as ::core::ffi::c_char,
    ],
    [
        'g' as ::core::ffi::c_char,
        '?' as ::core::ffi::c_char,
        OPF_CHANGE as ::core::ffi::c_char,
    ],
    [
        'r' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        OPF_CHANGE as ::core::ffi::c_char,
    ],
    [
        'I' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        OPF_CHANGE as ::core::ffi::c_char,
    ],
    [
        'A' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        OPF_CHANGE as ::core::ffi::c_char,
    ],
    [
        'z' as ::core::ffi::c_char,
        'f' as ::core::ffi::c_char,
        0 as ::core::ffi::c_char,
    ],
    [
        'z' as ::core::ffi::c_char,
        'o' as ::core::ffi::c_char,
        OPF_LINES as ::core::ffi::c_char,
    ],
    [
        'z' as ::core::ffi::c_char,
        'O' as ::core::ffi::c_char,
        OPF_LINES as ::core::ffi::c_char,
    ],
    [
        'z' as ::core::ffi::c_char,
        'c' as ::core::ffi::c_char,
        OPF_LINES as ::core::ffi::c_char,
    ],
    [
        'z' as ::core::ffi::c_char,
        'C' as ::core::ffi::c_char,
        OPF_LINES as ::core::ffi::c_char,
    ],
    [
        'z' as ::core::ffi::c_char,
        'd' as ::core::ffi::c_char,
        OPF_LINES as ::core::ffi::c_char,
    ],
    [
        'z' as ::core::ffi::c_char,
        'D' as ::core::ffi::c_char,
        OPF_LINES as ::core::ffi::c_char,
    ],
    [
        'g' as ::core::ffi::c_char,
        'w' as ::core::ffi::c_char,
        (OPF_LINES | OPF_CHANGE) as ::core::ffi::c_char,
    ],
    [
        'g' as ::core::ffi::c_char,
        '@' as ::core::ffi::c_char,
        OPF_CHANGE as ::core::ffi::c_char,
    ],
    [
        Ctrl_A as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        OPF_CHANGE as ::core::ffi::c_char,
    ],
    [
        Ctrl_X as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        OPF_CHANGE as ::core::ffi::c_char,
    ],
]);
pub unsafe extern "C" fn get_op_type(
    mut char1: ::core::ffi::c_int,
    mut char2: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    if char1 == 'r' as ::core::ffi::c_int {
        return OP_REPLACE as ::core::ffi::c_int;
    }
    if char1 == '~' as ::core::ffi::c_int {
        return OP_TILDE as ::core::ffi::c_int;
    }
    if char1 == 'g' as ::core::ffi::c_int && char2 == Ctrl_A {
        return OP_NR_ADD as ::core::ffi::c_int;
    }
    if char1 == 'g' as ::core::ffi::c_int && char2 == Ctrl_X {
        return OP_NR_SUB as ::core::ffi::c_int;
    }
    if char1 == 'z' as ::core::ffi::c_int && char2 == 'y' as ::core::ffi::c_int {
        return OP_YANK as ::core::ffi::c_int;
    }
    i = 0 as ::core::ffi::c_int;
    while !((*opchars.ptr())[i as usize][0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
        == char1
        && (*opchars.ptr())[i as usize][1 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            == char2)
    {
        if i == ::core::mem::size_of::<[[::core::ffi::c_char; 3]; 30]>()
            .wrapping_div(::core::mem::size_of::<[::core::ffi::c_char; 3]>())
            .wrapping_div(
                (::core::mem::size_of::<[[::core::ffi::c_char; 3]; 30]>()
                    .wrapping_rem(::core::mem::size_of::<[::core::ffi::c_char; 3]>())
                    == 0) as ::core::ffi::c_int as usize,
            )
            .wrapping_sub(1 as usize) as ::core::ffi::c_int
        {
            internal_error(b"get_op_type()\0".as_ptr() as *const ::core::ffi::c_char);
            break;
        } else {
            i += 1;
        }
    }
    return i;
}
pub unsafe extern "C" fn op_on_lines(mut op: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return (*opchars.ptr())[op as usize][2 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
        & OPF_LINES;
}
pub unsafe extern "C" fn op_is_change(mut op: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return (*opchars.ptr())[op as usize][2 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
        & OPF_CHANGE;
}
pub unsafe extern "C" fn get_op_char(mut optype: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return (*opchars.ptr())[optype as usize][0 as ::core::ffi::c_int as usize]
        as ::core::ffi::c_int;
}
pub unsafe extern "C" fn get_extra_op_char(mut optype: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return (*opchars.ptr())[optype as usize][1 as ::core::ffi::c_int as usize]
        as ::core::ffi::c_int;
}
pub unsafe extern "C" fn op_shift(
    mut oap: *mut oparg_T,
    mut curs_top: bool,
    mut amount: ::core::ffi::c_int,
) {
    let mut block_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if u_save(
        (*oap).start.lnum - 1 as linenr_T,
        (*oap).end.lnum + 1 as linenr_T,
    ) == FAIL
    {
        return;
    }
    if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
        block_col = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
    }
    let mut i: ::core::ffi::c_int =
        (*oap).line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut first_char: ::core::ffi::c_int =
            *get_cursor_line_ptr() as uint8_t as ::core::ffi::c_int;
        if first_char == NUL {
            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        } else if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
            shift_block(oap, amount);
        } else if first_char != '#' as ::core::ffi::c_int || !preprocs_left() {
            shift_line(
                (*oap).op_type == OP_LSHIFT as ::core::ffi::c_int,
                p_sr.get() != 0,
                amount,
                false_0,
            );
        }
        (*curwin.get()).w_cursor.lnum += 1;
        i -= 1;
    }
    if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
        (*curwin.get()).w_cursor.lnum = (*oap).start.lnum;
        (*curwin.get()).w_cursor.col = block_col as colnr_T;
    } else if curs_top {
        (*curwin.get()).w_cursor.lnum = (*oap).start.lnum;
        beginline(BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
    } else {
        (*curwin.get()).w_cursor.lnum -= 1;
    }
    foldOpenCursor();
    if (*oap).line_count as OptInt > p_report.get() {
        let mut op: *mut ::core::ffi::c_char = (if (*oap).op_type == OP_RSHIFT as ::core::ffi::c_int
        {
            b">\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"<\0".as_ptr() as *const ::core::ffi::c_char
        }) as *mut ::core::ffi::c_char;
        let mut msg_line_single: *mut ::core::ffi::c_char = ngettext(
            b"%ld line %sed %d time\0".as_ptr() as *const ::core::ffi::c_char,
            b"%ld line %sed %d times\0".as_ptr() as *const ::core::ffi::c_char,
            amount as ::core::ffi::c_ulong,
        );
        let mut msg_line_plural: *mut ::core::ffi::c_char = ngettext(
            b"%ld lines %sed %d time\0".as_ptr() as *const ::core::ffi::c_char,
            b"%ld lines %sed %d times\0".as_ptr() as *const ::core::ffi::c_char,
            amount as ::core::ffi::c_ulong,
        );
        vim_snprintf(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            ngettext(
                msg_line_single,
                msg_line_plural,
                (*oap).line_count as ::core::ffi::c_ulong,
            ),
            (*oap).line_count as int64_t,
            op,
            amount,
        );
        msg_keep(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            true_0 != 0,
            false_0 != 0,
        );
    }
    if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        (*curbuf.get()).b_op_start = (*oap).start;
        (*curbuf.get()).b_op_end.lnum = (*oap).end.lnum;
        (*curbuf.get()).b_op_end.col = ml_get_len((*oap).end.lnum);
        if (*curbuf.get()).b_op_end.col > 0 as ::core::ffi::c_int {
            (*curbuf.get()).b_op_end.col -= 1;
        }
    }
    changed_lines(
        curbuf.get(),
        (*oap).start.lnum,
        0 as colnr_T,
        (*oap).end.lnum + 1 as linenr_T,
        0 as linenr_T,
        true_0 != 0,
    );
}
unsafe extern "C" fn get_vts(
    mut vts_array: *const ::core::ffi::c_int,
    mut index: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut ts: ::core::ffi::c_int = 0;
    if index < 1 as ::core::ffi::c_int {
        ts = 0 as ::core::ffi::c_int;
    } else if index <= *vts_array.offset(0 as ::core::ffi::c_int as isize) {
        ts = *vts_array.offset(index as isize);
    } else {
        ts = *vts_array.offset(*vts_array.offset(0 as ::core::ffi::c_int as isize) as isize);
    }
    return ts;
}
unsafe extern "C" fn get_vts_sum(
    mut vts_array: *const ::core::ffi::c_int,
    mut index: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut sum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0;
    i = 1 as ::core::ffi::c_int;
    while i <= index && i <= *vts_array.offset(0 as ::core::ffi::c_int as isize) {
        sum += *vts_array.offset(i as isize);
        i += 1;
    }
    if i <= index {
        sum += *vts_array.offset(*vts_array.offset(0 as ::core::ffi::c_int as isize) as isize)
            * (index - *vts_array.offset(0 as ::core::ffi::c_int as isize));
    }
    return sum;
}
unsafe extern "C" fn get_new_sw_indent(
    mut left: bool,
    mut round: bool,
    mut amount: int64_t,
    mut sw_val: int64_t,
) -> int64_t {
    let mut count: int64_t = get_indent() as int64_t;
    if round {
        let mut i: int64_t = crate::src::nvim::math::trim_to_int(count / sw_val) as int64_t;
        let mut j: int64_t = crate::src::nvim::math::trim_to_int(count % sw_val) as int64_t;
        if j != 0 && left as ::core::ffi::c_int != 0 {
            amount -= 1;
        }
        if left {
            i = if i - amount > 0 as int64_t {
                i - amount
            } else {
                0 as int64_t
            };
        } else {
            i += amount;
        }
        count = i * sw_val;
    } else if left {
        count = if count - sw_val * amount > 0 as int64_t {
            count - sw_val * amount
        } else {
            0 as int64_t
        };
    } else {
        count += sw_val * amount;
    }
    return count;
}
unsafe extern "C" fn get_new_vts_indent(
    mut left: bool,
    mut round: bool,
    mut amount: ::core::ffi::c_int,
    mut vts_array: *mut ::core::ffi::c_int,
) -> int64_t {
    let mut indent: int64_t = get_indent() as int64_t;
    let mut vtsi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut vts_indent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut ts: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while vts_indent as int64_t <= indent {
        vtsi += 1;
        ts = get_vts(vts_array, vtsi);
        vts_indent += ts;
    }
    vts_indent -= ts;
    vtsi -= 1;
    let mut offset: int64_t = indent - vts_indent as int64_t;
    if round {
        if left {
            if offset == 0 as int64_t {
                indent = get_vts_sum(vts_array, vtsi - amount) as int64_t;
            } else {
                indent =
                    get_vts_sum(vts_array, vtsi - (amount - 1 as ::core::ffi::c_int)) as int64_t;
            }
        } else {
            indent = get_vts_sum(vts_array, vtsi + amount) as int64_t;
        }
    } else if left {
        if amount > vtsi {
            indent = 0 as int64_t;
        } else {
            indent = get_vts_sum(vts_array, vtsi - amount) as int64_t + offset;
        }
    } else {
        indent = get_vts_sum(vts_array, vtsi + amount) as int64_t + offset;
    }
    return indent;
}
pub unsafe extern "C" fn shift_line(
    mut left: bool,
    mut round: bool,
    mut amount: ::core::ffi::c_int,
    mut call_changed_bytes: ::core::ffi::c_int,
) {
    let mut count: int64_t = 0;
    let mut sw_val: int64_t = (*curbuf.get()).b_p_sw as int64_t;
    let mut ts_val: int64_t = (*curbuf.get()).b_p_ts as int64_t;
    let mut vts_array: *mut ::core::ffi::c_int =
        (*curbuf.get()).b_p_vts_array as *mut ::core::ffi::c_int;
    if sw_val != 0 as int64_t {
        count = get_new_sw_indent(left, round, amount as int64_t, sw_val);
    } else if vts_array.is_null()
        || *vts_array.offset(0 as ::core::ffi::c_int as isize) == 0 as ::core::ffi::c_int
    {
        count = get_new_sw_indent(left, round, amount as int64_t, ts_val);
    } else {
        count = get_new_vts_indent(left, round, amount, vts_array);
    }
    if State.get() & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
        change_indent(
            INDENT_SET as ::core::ffi::c_int,
            crate::src::nvim::math::trim_to_int(count),
            false_0,
            call_changed_bytes != 0,
        );
    } else {
        set_indent(
            crate::src::nvim::math::trim_to_int(count),
            if call_changed_bytes != 0 {
                SIN_CHANGED as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
        );
    };
}
unsafe extern "C" fn shift_block(mut oap: *mut oparg_T, mut amount: ::core::ffi::c_int) {
    let left: bool = (*oap).op_type == OP_LSHIFT as ::core::ffi::c_int;
    let oldstate: ::core::ffi::c_int = State.get();
    let mut newp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let oldcol: ::core::ffi::c_int = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
    let sw_val: ::core::ffi::c_int = get_sw_value_indent(curbuf.get(), left);
    let ts_val: ::core::ffi::c_int = (*curbuf.get()).b_p_ts as ::core::ffi::c_int;
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
    let mut incr: ::core::ffi::c_int = 0;
    let old_p_ri: ::core::ffi::c_int = p_ri.get();
    p_ri.set(0 as ::core::ffi::c_int);
    State.set(MODE_INSERT as ::core::ffi::c_int);
    block_prep(oap, &raw mut bd, (*curwin.get()).w_cursor.lnum, true_0 != 0);
    if bd.is_short != 0 {
        return;
    }
    let mut total: ::core::ffi::c_int = (amount as ::core::ffi::c_uint)
        .wrapping_mul(sw_val as ::core::ffi::c_uint)
        as ::core::ffi::c_int;
    if total / sw_val != amount {
        return;
    }
    let oldp: *mut ::core::ffi::c_char = get_cursor_line_ptr();
    let old_line_len: ::core::ffi::c_int = get_cursor_line_len();
    let mut startcol: ::core::ffi::c_int = 0;
    let mut oldlen: ::core::ffi::c_int = 0;
    let mut newlen: ::core::ffi::c_int = 0;
    if !left {
        total += bd.pre_whitesp;
        let mut ws_vcol: colnr_T = bd.start_vcol - bd.pre_whitesp as colnr_T;
        let mut old_textstart: *mut ::core::ffi::c_char = bd.textstart;
        if bd.startspaces != 0 {
            if utfc_ptr2len(bd.textstart) == 1 as ::core::ffi::c_int {
                bd.textstart = bd.textstart.offset(1);
            } else {
                ws_vcol = 0 as ::core::ffi::c_int as colnr_T;
                bd.startspaces = 0 as ::core::ffi::c_int;
            }
        }
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
                s: [C2Rust_Unnamed_15 { oldcol: 0, i: 0 }; 20],
                intersect_idx: 0,
                intersect_pos: MTPos { row: 0, col: 0 },
                intersect_pos_x: MTPos { row: 0, col: 0 },
            }; 1],
        };
        let mut cstype: CSType = init_charsize_arg(
            &raw mut csarg,
            curwin.get(),
            (*curwin.get()).w_cursor.lnum,
            bd.textstart,
        );
        let mut ci: StrCharInfo = utf_ptr2StrCharInfo(bd.textstart);
        let mut vcol: ::core::ffi::c_int = bd.start_vcol as ::core::ffi::c_int;
        while ascii_iswhite(ci.chr.value as ::core::ffi::c_int) {
            incr = win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &raw mut csarg).width;
            ci = utfc_next(ci);
            total += incr;
            vcol += incr;
        }
        bd.textstart = ci.ptr;
        bd.start_vcol = vcol as colnr_T;
        let mut tabs: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut spaces: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*curbuf.get()).b_p_et == 0 {
            tabstop_fromto(
                ws_vcol,
                ws_vcol + total as colnr_T,
                ts_val,
                (*curbuf.get()).b_p_vts_array,
                &raw mut tabs,
                &raw mut spaces,
            );
        } else {
            spaces = total;
        }
        let col_pre: ::core::ffi::c_int =
            bd.pre_whitesp_c - (bd.startspaces != 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
        bd.textcol -= col_pre;
        let new_line_len: ::core::ffi::c_int = bd.textcol as ::core::ffi::c_int
            + tabs
            + spaces
            + (old_line_len - bd.textstart.offset_from(oldp) as ::core::ffi::c_int);
        newp =
            xmalloc((new_line_len as size_t).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        memmove(
            newp as *mut ::core::ffi::c_void,
            oldp as *const ::core::ffi::c_void,
            bd.textcol as size_t,
        );
        startcol = bd.textcol as ::core::ffi::c_int;
        oldlen = bd.textstart.offset_from(old_textstart) as ::core::ffi::c_int + col_pre;
        newlen = tabs + spaces;
        memset(
            newp.offset(bd.textcol as isize) as *mut ::core::ffi::c_void,
            TAB,
            tabs as size_t,
        );
        memset(
            newp.offset(bd.textcol as isize).offset(tabs as isize) as *mut ::core::ffi::c_void,
            ' ' as ::core::ffi::c_int,
            spaces as size_t,
        );
        strcpy(
            newp.offset(bd.textcol as isize)
                .offset(tabs as isize)
                .offset(spaces as isize),
            bd.textstart,
        );
        '_c2rust_label: {
            if newlen - oldlen == new_line_len - old_line_len {
            } else {
                __assert_fail(
                    b"newlen - oldlen == new_line_len - old_line_len\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/ops.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    497 as ::core::ffi::c_uint,
                    b"void shift_block(oparg_T *, int)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
    } else {
        let mut verbatim_copy_end: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut verbatim_copy_width: colnr_T = 0;
        let mut non_white: *mut ::core::ffi::c_char = bd.textstart;
        if bd.startspaces != 0 {
            non_white = non_white.offset(utfc_ptr2len(non_white) as isize);
        }
        let mut non_white_col: colnr_T = bd.start_vcol;
        let mut csarg_0: CharsizeArg = CharsizeArg {
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
                s: [C2Rust_Unnamed_15 { oldcol: 0, i: 0 }; 20],
                intersect_idx: 0,
                intersect_pos: MTPos { row: 0, col: 0 },
                intersect_pos_x: MTPos { row: 0, col: 0 },
            }; 1],
        };
        let mut cstype_0: CSType = init_charsize_arg(
            &raw mut csarg_0,
            curwin.get(),
            (*curwin.get()).w_cursor.lnum,
            bd.textstart,
        );
        while ascii_iswhite(*non_white as ::core::ffi::c_int) {
            incr = win_charsize(
                cstype_0,
                non_white_col as ::core::ffi::c_int,
                non_white,
                *non_white as uint8_t as int32_t,
                &raw mut csarg_0,
            )
            .width;
            non_white_col += incr;
            non_white = non_white.offset(1);
        }
        let block_space_width: colnr_T = non_white_col - (*oap).start_vcol;
        let shift_amount: colnr_T = if block_space_width < total {
            block_space_width
        } else {
            total as colnr_T
        };
        let destination_col: colnr_T = non_white_col - shift_amount;
        verbatim_copy_end = bd.textstart;
        verbatim_copy_width = bd.start_vcol;
        if bd.startspaces != 0 {
            verbatim_copy_width -= bd.start_char_vcols;
        }
        cstype_0 = init_charsize_arg(&raw mut csarg_0, curwin.get(), 0 as linenr_T, bd.textstart);
        let mut ci_0: StrCharInfo = utf_ptr2StrCharInfo(verbatim_copy_end);
        while verbatim_copy_width < destination_col {
            incr = win_charsize(
                cstype_0,
                verbatim_copy_width as ::core::ffi::c_int,
                ci_0.ptr,
                ci_0.chr.value,
                &raw mut csarg_0,
            )
            .width;
            if verbatim_copy_width as ::core::ffi::c_int + incr > destination_col {
                break;
            }
            verbatim_copy_width += incr;
            ci_0 = utfc_next(ci_0);
        }
        verbatim_copy_end = ci_0.ptr;
        '_c2rust_label_0: {
            if destination_col - verbatim_copy_width >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"destination_col - verbatim_copy_width >= 0\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/ops.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    561 as ::core::ffi::c_uint,
                    b"void shift_block(oparg_T *, int)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let fill: ::core::ffi::c_int =
            destination_col as ::core::ffi::c_int - verbatim_copy_width as ::core::ffi::c_int;
        '_c2rust_label_1: {
            if verbatim_copy_end.offset_from(oldp) >= 0 as isize {
            } else {
                __assert_fail(
                    b"verbatim_copy_end - oldp >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/ops.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    565 as ::core::ffi::c_uint,
                    b"void shift_block(oparg_T *, int)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let fixedlen: ::core::ffi::c_int =
            verbatim_copy_end.offset_from(oldp) as ::core::ffi::c_int;
        let new_line_len_0: ::core::ffi::c_int =
            fixedlen + fill + (old_line_len - non_white.offset_from(oldp) as ::core::ffi::c_int);
        newp = xmalloc((new_line_len_0 as size_t).wrapping_add(1 as size_t))
            as *mut ::core::ffi::c_char;
        startcol = fixedlen;
        oldlen = bd.textcol as ::core::ffi::c_int
            + non_white.offset_from(bd.textstart) as ::core::ffi::c_int
            - fixedlen;
        newlen = fill;
        memmove(
            newp as *mut ::core::ffi::c_void,
            oldp as *const ::core::ffi::c_void,
            fixedlen as size_t,
        );
        memset(
            newp.offset(fixedlen as isize) as *mut ::core::ffi::c_void,
            ' ' as ::core::ffi::c_int,
            fill as size_t,
        );
        strcpy(
            newp.offset(fixedlen as isize).offset(fill as isize),
            non_white,
        );
        '_c2rust_label_2: {
            if newlen - oldlen == new_line_len_0 - old_line_len {
            } else {
                __assert_fail(
                    b"newlen - oldlen == new_line_len - old_line_len\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/ops.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    582 as ::core::ffi::c_uint,
                    b"void shift_block(oparg_T *, int)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
    }
    ml_replace((*curwin.get()).w_cursor.lnum, newp, false_0 != 0);
    changed_bytes((*curwin.get()).w_cursor.lnum, bd.textcol);
    extmark_splice_cols(
        curbuf.get(),
        (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        startcol as colnr_T,
        oldlen as colnr_T,
        newlen as colnr_T,
        kExtmarkUndo,
    );
    State.set(oldstate);
    (*curwin.get()).w_cursor.col = oldcol as colnr_T;
    p_ri.set(old_p_ri);
}
unsafe extern "C" fn block_insert(
    mut oap: *mut oparg_T,
    mut s: *const ::core::ffi::c_char,
    mut slen: size_t,
    mut b_insert: bool,
    mut bdp: *mut block_def,
) {
    let mut ts_val: ::core::ffi::c_int = 0;
    let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut spaces: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut offset: colnr_T = 0;
    let mut newp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut oldp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut oldstate: ::core::ffi::c_int = State.get();
    State.set(MODE_INSERT as ::core::ffi::c_int);
    let mut lnum: linenr_T = (*oap).start.lnum + 1 as linenr_T;
    while lnum <= (*oap).end.lnum {
        block_prep(oap, bdp, lnum, true_0 != 0);
        if !((*bdp).is_short != 0 && b_insert as ::core::ffi::c_int != 0) {
            oldp = ml_get(lnum);
            if b_insert {
                ts_val = (*bdp).start_char_vcols as ::core::ffi::c_int;
                spaces = (*bdp).startspaces;
                if spaces != 0 as ::core::ffi::c_int {
                    count = ts_val - 1 as ::core::ffi::c_int;
                }
                offset = (*bdp).textcol;
            } else {
                ts_val = (*bdp).end_char_vcols as ::core::ffi::c_int;
                if (*bdp).is_short == 0 {
                    spaces = if (*bdp).endspaces != 0 {
                        ts_val - (*bdp).endspaces
                    } else {
                        0 as ::core::ffi::c_int
                    };
                    if spaces != 0 as ::core::ffi::c_int {
                        count = ts_val - 1 as ::core::ffi::c_int;
                    }
                    offset = ((*bdp).textcol as ::core::ffi::c_int + (*bdp).textlen
                        - (spaces != 0 as ::core::ffi::c_int) as ::core::ffi::c_int)
                        as colnr_T;
                } else {
                    if (*bdp).is_MAX == 0 {
                        spaces = (*oap).end_vcol as ::core::ffi::c_int
                            - (*bdp).end_vcol as ::core::ffi::c_int
                            + 1 as ::core::ffi::c_int;
                    }
                    count = spaces;
                    offset = ((*bdp).textcol as ::core::ffi::c_int + (*bdp).textlen) as colnr_T;
                }
            }
            if spaces > 0 as ::core::ffi::c_int {
                offset -= utf_head_off(oldp, oldp.offset(offset as isize));
            }
            spaces = if spaces > 0 as ::core::ffi::c_int {
                spaces
            } else {
                0 as ::core::ffi::c_int
            };
            '_c2rust_label: {
                if count >= 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"count >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/ops.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        647 as ::core::ffi::c_uint,
                        b"void block_insert(oparg_T *, const char *, size_t, _Bool, struct block_def *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            newp = xmalloc(
                (ml_get_len(lnum) as size_t)
                    .wrapping_add(spaces as size_t)
                    .wrapping_add(slen)
                    .wrapping_add(
                        if spaces > 0 as ::core::ffi::c_int && (*bdp).is_short == 0 {
                            (ts_val - spaces) as size_t
                        } else {
                            0 as size_t
                        },
                    )
                    .wrapping_add(count as size_t)
                    .wrapping_add(1 as size_t),
            ) as *mut ::core::ffi::c_char;
            memmove(
                newp as *mut ::core::ffi::c_void,
                oldp as *const ::core::ffi::c_void,
                offset as size_t,
            );
            oldp = oldp.offset(offset as isize);
            let mut startcol: ::core::ffi::c_int = offset as ::core::ffi::c_int;
            memset(
                newp.offset(offset as isize) as *mut ::core::ffi::c_void,
                ' ' as ::core::ffi::c_int,
                spaces as size_t,
            );
            memmove(
                newp.offset(offset as isize).offset(spaces as isize) as *mut ::core::ffi::c_void,
                s as *const ::core::ffi::c_void,
                slen,
            );
            offset += slen as ::core::ffi::c_int;
            let mut skipped: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if spaces > 0 as ::core::ffi::c_int && (*bdp).is_short == 0 {
                if *oldp as ::core::ffi::c_int == TAB {
                    memset(
                        newp.offset(offset as isize).offset(spaces as isize)
                            as *mut ::core::ffi::c_void,
                        ' ' as ::core::ffi::c_int,
                        (ts_val - spaces) as size_t,
                    );
                    oldp = oldp.offset(1);
                    count += 1;
                    skipped = 1 as ::core::ffi::c_int;
                } else {
                    count = spaces;
                }
            }
            if spaces > 0 as ::core::ffi::c_int {
                offset += count;
            }
            strcpy(newp.offset(offset as isize), oldp);
            ml_replace(lnum, newp, false_0 != 0);
            extmark_splice_cols(
                curbuf.get(),
                lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                startcol as colnr_T,
                skipped as colnr_T,
                offset - startcol as colnr_T,
                kExtmarkUndo,
            );
            if lnum == (*oap).end.lnum {
                (*curbuf.get()).b_op_end.lnum = (*oap).end.lnum;
                (*curbuf.get()).b_op_end.col = offset;
                if (*curbuf.get()).b_visual.vi_end.coladd != 0 {
                    (*curbuf.get()).b_visual.vi_end.col += (*curbuf.get()).b_visual.vi_end.coladd;
                    (*curbuf.get()).b_visual.vi_end.coladd = 0 as ::core::ffi::c_int as colnr_T;
                }
            }
        }
        lnum += 1;
    }
    State.set(oldstate);
    if (*oap).start.lnum < (*oap).end.lnum {
        changed_lines(
            curbuf.get(),
            (*oap).start.lnum + 1 as linenr_T,
            0 as colnr_T,
            (*oap).end.lnum + 1 as linenr_T,
            0 as linenr_T,
            true_0 != 0,
        );
    }
}
pub unsafe extern "C" fn op_delete(mut oap: *mut oparg_T) -> ::core::ffi::c_int {
    let mut lnum: linenr_T = 0;
    let mut bd: block_def = block_def {
        startspaces: 0 as ::core::ffi::c_int,
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
    let mut old_lcount: linenr_T = (*curbuf.get()).b_ml.ml_line_count;
    if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
        return OK;
    }
    if (*oap).empty {
        return u_save_cursor();
    }
    if (*curbuf.get()).b_p_ma == 0 {
        emsg(gettext(
            &raw const e_modifiable as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    if VIsual_select.get() as ::core::ffi::c_int != 0 && (*oap).is_VIsual as ::core::ffi::c_int != 0
    {
        (*oap).regname = VIsual_select_reg.get();
    }
    mb_adjust_opend(oap);
    if (*oap).motion_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
        && !(*oap).is_VIsual
        && (*oap).line_count > 1 as linenr_T
        && (*oap).motion_force == NUL
        && (*oap).op_type == OP_DELETE as ::core::ffi::c_int
    {
        let mut ptr: *mut ::core::ffi::c_char =
            ml_get((*oap).end.lnum).offset((*oap).end.col as isize);
        if *ptr as ::core::ffi::c_int != NUL {
            ptr = ptr.offset((*oap).inclusive as ::core::ffi::c_int as isize);
        }
        ptr = skipwhite(ptr);
        if *ptr as ::core::ffi::c_int == NUL
            && inindent(0 as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        {
            (*oap).motion_type = kMTLineWise;
        }
    }
    if (*oap).motion_type as ::core::ffi::c_int != kMTLineWise as ::core::ffi::c_int
        && (*oap).line_count == 1 as linenr_T
        && (*oap).op_type == OP_DELETE as ::core::ffi::c_int
        && *ml_get((*oap).start.lnum) as ::core::ffi::c_int == NUL
    {
        if virtual_op.get() as u64 == 0 {
            if !vim_strchr(p_cpo.get(), CPO_EMPTYREGION).is_null() {
                beep_flush();
            }
            return OK;
        }
    } else {
        if (*oap).regname != '_' as ::core::ffi::c_int {
            let mut reg: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
            let mut did_yank: bool = false_0 != 0;
            if (*oap).regname != 0 as ::core::ffi::c_int {
                if !valid_yank_reg((*oap).regname, true_0 != 0) {
                    beep_flush();
                    return OK;
                }
                reg = get_yank_register((*oap).regname, YREG_YANK as ::core::ffi::c_int);
                op_yank_reg(oap, false_0 != 0, reg, is_append_register((*oap).regname));
                did_yank = true_0 != 0;
            }
            if (*oap).motion_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
                || (*oap).line_count > 1 as linenr_T
                || (*oap).use_reg_one as ::core::ffi::c_int != 0
            {
                shift_delete_registers(is_append_register((*oap).regname));
                reg = get_y_register(1 as ::core::ffi::c_int);
                op_yank_reg(oap, false_0 != 0, reg, false_0 != 0);
                did_yank = true_0 != 0;
            }
            if (*oap).regname == 0 as ::core::ffi::c_int
                && (*oap).motion_type as ::core::ffi::c_int != kMTLineWise as ::core::ffi::c_int
                && (*oap).line_count == 1 as linenr_T
            {
                reg = get_yank_register('-' as ::core::ffi::c_int, YREG_YANK as ::core::ffi::c_int);
                op_yank_reg(oap, false_0 != 0, reg, false_0 != 0);
                did_yank = true_0 != 0;
            }
            if did_yank as ::core::ffi::c_int != 0 || (*oap).regname == 0 as ::core::ffi::c_int {
                if reg.is_null() {
                    abort();
                }
                crate::src::nvim::clipboard::set_clipboard((*oap).regname, reg as *mut _);
                do_autocmd_textyankpost(oap, reg);
            }
        }
        if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
            if u_save(
                (*oap).start.lnum - 1 as linenr_T,
                (*oap).end.lnum + 1 as linenr_T,
            ) == FAIL
            {
                return FAIL;
            }
            lnum = (*curwin.get()).w_cursor.lnum;
            while lnum <= (*oap).end.lnum {
                block_prep(oap, &raw mut bd, lnum, true_0 != 0);
                if bd.textlen != 0 as ::core::ffi::c_int {
                    if lnum == (*curwin.get()).w_cursor.lnum {
                        (*curwin.get()).w_cursor.col =
                            (bd.textcol as ::core::ffi::c_int + bd.startspaces) as colnr_T;
                        (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
                    }
                    let mut n: ::core::ffi::c_int = bd.textlen - bd.startspaces - bd.endspaces;
                    let mut oldp: *mut ::core::ffi::c_char = ml_get(lnum);
                    let mut newp: *mut ::core::ffi::c_char = xmalloc(
                        (ml_get_len(lnum) as size_t)
                            .wrapping_sub(n as size_t)
                            .wrapping_add(1 as size_t),
                    )
                        as *mut ::core::ffi::c_char;
                    memmove(
                        newp as *mut ::core::ffi::c_void,
                        oldp as *const ::core::ffi::c_void,
                        bd.textcol as size_t,
                    );
                    memset(
                        newp.offset(bd.textcol as isize) as *mut ::core::ffi::c_void,
                        ' ' as ::core::ffi::c_int,
                        (bd.startspaces as size_t).wrapping_add(bd.endspaces as size_t),
                    );
                    strcpy(
                        newp.offset(bd.textcol as isize)
                            .offset(bd.startspaces as isize)
                            .offset(bd.endspaces as isize),
                        oldp.offset(bd.textcol as isize).offset(bd.textlen as isize),
                    );
                    ml_replace(lnum, newp, false_0 != 0);
                    extmark_splice_cols(
                        curbuf.get(),
                        lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                        bd.textcol,
                        bd.textlen as colnr_T,
                        bd.startspaces as colnr_T + bd.endspaces as colnr_T,
                        kExtmarkUndo,
                    );
                }
                lnum += 1;
            }
            check_cursor_col(curwin.get());
            changed_lines(
                curbuf.get(),
                (*curwin.get()).w_cursor.lnum,
                (*curwin.get()).w_cursor.col,
                (*oap).end.lnum + 1 as linenr_T,
                0 as linenr_T,
                true_0 != 0,
            );
            (*oap).line_count = 0 as ::core::ffi::c_int as linenr_T;
        } else if (*oap).motion_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
            if (*oap).op_type == OP_CHANGE as ::core::ffi::c_int {
                if (*oap).line_count > 1 as linenr_T {
                    lnum = (*curwin.get()).w_cursor.lnum;
                    (*curwin.get()).w_cursor.lnum += 1;
                    del_lines((*oap).line_count - 1 as linenr_T, true_0 != 0);
                    (*curwin.get()).w_cursor.lnum = lnum;
                }
                if u_save_cursor() == FAIL {
                    return FAIL;
                }
                if (*curbuf.get()).b_p_ai != 0 {
                    beginline(BL_WHITE as ::core::ffi::c_int);
                    did_ai.set(true_0 != 0);
                    ai_col.set((*curwin.get()).w_cursor.col);
                } else {
                    beginline(0 as ::core::ffi::c_int);
                }
                truncate_line(false_0);
                if (*oap).line_count > 1 as linenr_T {
                    u_clearline(curbuf.get());
                }
            } else {
                del_lines((*oap).line_count, true_0 != 0);
                beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
                u_clearline(curbuf.get());
            }
        } else {
            if virtual_op.get() as u64 != 0 {
                if gchar_pos(&raw mut (*oap).start) == '\t' as ::core::ffi::c_int {
                    let mut endcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    if u_save_cursor() == FAIL {
                        return FAIL;
                    }
                    if (*oap).line_count == 1 as linenr_T {
                        endcol = getviscol2((*oap).end.col, (*oap).end.coladd);
                    }
                    coladvance_force(getviscol2((*oap).start.col, (*oap).start.coladd));
                    (*oap).start = (*curwin.get()).w_cursor;
                    if (*oap).line_count == 1 as linenr_T {
                        coladvance(curwin.get(), endcol as colnr_T);
                        (*oap).end.col = (*curwin.get()).w_cursor.col;
                        (*oap).end.coladd = (*curwin.get()).w_cursor.coladd;
                        (*curwin.get()).w_cursor = (*oap).start;
                    }
                }
                if gchar_pos(&raw mut (*oap).end) == '\t' as ::core::ffi::c_int
                    && (*oap).end.coladd == 0 as ::core::ffi::c_int
                    && (*oap).inclusive as ::core::ffi::c_int != 0
                {
                    if u_save(
                        (*oap).end.lnum - 1 as linenr_T,
                        (*oap).end.lnum + 1 as linenr_T,
                    ) == FAIL
                    {
                        return FAIL;
                    }
                    (*curwin.get()).w_cursor = (*oap).end;
                    coladvance_force(getviscol2((*oap).end.col, (*oap).end.coladd));
                    (*oap).end = (*curwin.get()).w_cursor;
                    (*curwin.get()).w_cursor = (*oap).start;
                }
                mb_adjust_opend(oap);
            }
            if (*oap).line_count == 1 as linenr_T {
                if u_save_cursor() == FAIL {
                    return FAIL;
                }
                if !vim_strchr(p_cpo.get(), CPO_DOLLAR).is_null()
                    && (*oap).op_type == OP_CHANGE as ::core::ffi::c_int
                    && (*oap).end.lnum == (*curwin.get()).w_cursor.lnum
                    && !(*oap).is_VIsual
                {
                    display_dollar((*oap).end.col - !(*oap).inclusive as ::core::ffi::c_int);
                }
                let mut n_0: ::core::ffi::c_int = (*oap).end.col as ::core::ffi::c_int
                    - (*oap).start.col as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    - !(*oap).inclusive as ::core::ffi::c_int;
                if virtual_op.get() as u64 != 0 {
                    let mut len: ::core::ffi::c_int = get_cursor_line_len();
                    if (*oap).end.coladd != 0 as ::core::ffi::c_int
                        && (*oap).end.col >= len - 1 as ::core::ffi::c_int
                        && !((*oap).start.coladd != 0
                            && (*oap).end.col >= len - 1 as ::core::ffi::c_int)
                    {
                        n_0 += 1;
                    }
                    if n_0 == 0 as ::core::ffi::c_int && (*oap).start.coladd != (*oap).end.coladd {
                        n_0 = 1 as ::core::ffi::c_int;
                    }
                    if gchar_cursor() != NUL {
                        (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
                    }
                }
                del_bytes(
                    n_0,
                    virtual_op.get() as u64 == 0,
                    (*oap).op_type == OP_DELETE as ::core::ffi::c_int && !(*oap).is_VIsual,
                );
            } else {
                let mut curpos: pos_T = pos_T {
                    lnum: 0,
                    col: 0,
                    coladd: 0,
                };
                if u_save(
                    (*curwin.get()).w_cursor.lnum - 1 as linenr_T,
                    (*curwin.get()).w_cursor.lnum + (*oap).line_count,
                ) == FAIL
                {
                    return FAIL;
                }
                (*curbuf_splice_pending.ptr()) += 1;
                let mut startpos: pos_T = (*curwin.get()).w_cursor;
                let mut deleted_bytes: bcount_t = get_region_bytecount(
                    curbuf.get(),
                    startpos.lnum,
                    (*oap).end.lnum,
                    startpos.col,
                    (*oap).end.col,
                ) + (*oap).inclusive as bcount_t;
                truncate_line(true_0);
                curpos = (*curwin.get()).w_cursor;
                (*curwin.get()).w_cursor.lnum += 1;
                del_lines((*oap).line_count - 2 as linenr_T, false_0 != 0);
                let mut n_1: ::core::ffi::c_int = (*oap).end.col as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    - !(*oap).inclusive as ::core::ffi::c_int;
                (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                del_bytes(
                    n_1,
                    virtual_op.get() as u64 == 0,
                    (*oap).op_type == OP_DELETE as ::core::ffi::c_int && !(*oap).is_VIsual,
                );
                (*curwin.get()).w_cursor = curpos;
                do_join(
                    2 as size_t,
                    false_0 != 0,
                    false_0 != 0,
                    false_0 != 0,
                    false_0 != 0,
                );
                (*curbuf_splice_pending.ptr()) -= 1;
                extmark_splice(
                    curbuf.get(),
                    startpos.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    startpos.col,
                    (*oap).line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    n_1 as colnr_T,
                    deleted_bytes,
                    0 as ::core::ffi::c_int,
                    0 as colnr_T,
                    0 as bcount_t,
                    kExtmarkUndo,
                );
            }
            if (*oap).op_type == OP_DELETE as ::core::ffi::c_int {
                auto_format(false_0 != 0, true_0 != 0);
            }
        }
        msgmore(
            (*curbuf.get()).b_ml.ml_line_count as ::core::ffi::c_int
                - old_lcount as ::core::ffi::c_int,
        );
    }
    if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
            (*curbuf.get()).b_op_end.lnum = (*oap).end.lnum;
            (*curbuf.get()).b_op_end.col = (*oap).start.col;
        } else {
            (*curbuf.get()).b_op_end = (*oap).start;
        }
        (*curbuf.get()).b_op_start = (*oap).start;
    }
    return OK;
}
unsafe extern "C" fn mb_adjust_opend(mut oap: *mut oparg_T) {
    if !(*oap).inclusive {
        return;
    }
    let mut line: *const ::core::ffi::c_char = ml_get((*oap).end.lnum);
    let mut ptr: *const ::core::ffi::c_char = line.offset((*oap).end.col as isize);
    if *ptr as ::core::ffi::c_int != NUL {
        ptr = ptr.offset(-(utf_head_off(line, ptr) as isize));
        ptr = ptr.offset((utfc_ptr2len(ptr) - 1 as ::core::ffi::c_int) as isize);
        (*oap).end.col = ptr.offset_from(line) as colnr_T;
    }
}
unsafe extern "C" fn pbyte(mut lp: pos_T, mut c: ::core::ffi::c_int) {
    '_c2rust_label: {
        if c <= 127 as ::core::ffi::c_int * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"c <= UCHAR_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/ops.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1054 as ::core::ffi::c_uint,
                b"void pbyte(pos_T, int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut p: *mut ::core::ffi::c_char = ml_get_buf_mut(curbuf.get(), lp.lnum);
    let mut len: colnr_T = (*curbuf.get()).b_ml.ml_line_textlen;
    if lp.col >= len {
        lp.col = (if len > 1 as ::core::ffi::c_int {
            len as ::core::ffi::c_int - 2 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as colnr_T;
    }
    *p.offset(lp.col as isize) = c as ::core::ffi::c_char;
    if curbuf_splice_pending.get() == 0 {
        extmark_splice_cols(
            curbuf.get(),
            lp.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            lp.col,
            1 as colnr_T,
            1 as colnr_T,
            kExtmarkUndo,
        );
    }
}
unsafe extern "C" fn replace_character(mut c: ::core::ffi::c_int) {
    let n: ::core::ffi::c_int = State.get();
    State.set(MODE_REPLACE as ::core::ffi::c_int);
    ins_char(c);
    State.set(n);
    dec_cursor();
}
unsafe extern "C" fn op_replace(
    mut oap: *mut oparg_T,
    mut c: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut n: ::core::ffi::c_int = 0;
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
    let mut after_p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut had_ctrl_v_cr: bool = false_0 != 0;
    if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 || (*oap).empty as ::core::ffi::c_int != 0 {
        return OK;
    }
    if c == REPLACE_CR_NCHAR as ::core::ffi::c_int {
        had_ctrl_v_cr = true_0 != 0;
        c = CAR;
    } else if c == REPLACE_NL_NCHAR as ::core::ffi::c_int {
        had_ctrl_v_cr = true_0 != 0;
        c = NL;
    }
    mb_adjust_opend(oap);
    if u_save(
        (*oap).start.lnum - 1 as linenr_T,
        (*oap).end.lnum + 1 as linenr_T,
    ) == FAIL
    {
        return FAIL;
    }
    if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
        bd.is_MAX =
            ((*curwin.get()).w_curswant == MAXCOL as ::core::ffi::c_int) as ::core::ffi::c_int;
        while (*curwin.get()).w_cursor.lnum <= (*oap).end.lnum {
            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
            block_prep(oap, &raw mut bd, (*curwin.get()).w_cursor.lnum, true_0 != 0);
            if !(bd.textlen == 0 as ::core::ffi::c_int
                && (virtual_op.get() as u64 == 0 || bd.is_MAX != 0))
            {
                if virtual_op.get() as ::core::ffi::c_int != 0
                    && bd.is_short != 0
                    && *bd.textstart as ::core::ffi::c_int == NUL
                {
                    let mut vpos: pos_T = pos_T {
                        lnum: 0,
                        col: 0,
                        coladd: 0,
                    };
                    vpos.lnum = (*curwin.get()).w_cursor.lnum;
                    getvpos(curwin.get(), &raw mut vpos, (*oap).start_vcol);
                    bd.startspaces += vpos.coladd as ::core::ffi::c_int;
                    n = bd.startspaces;
                } else {
                    n = if bd.startspaces != 0 {
                        bd.start_char_vcols as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    };
                }
                n += if bd.endspaces != 0
                    && bd.is_oneChar == 0
                    && bd.end_char_vcols > 0 as ::core::ffi::c_int
                {
                    bd.end_char_vcols as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
                let mut numc: ::core::ffi::c_int = (*oap).end_vcol as ::core::ffi::c_int
                    - (*oap).start_vcol as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int;
                if bd.is_short != 0 && (virtual_op.get() as u64 == 0 || bd.is_MAX != 0) {
                    numc -= (*oap).end_vcol as ::core::ffi::c_int
                        - bd.end_vcol as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int;
                }
                if utf_char2cells(c) > 1 as ::core::ffi::c_int {
                    if numc & 1 as ::core::ffi::c_int != 0 && bd.is_short == 0 {
                        bd.endspaces += 1;
                        n += 1;
                    }
                    numc = numc / 2 as ::core::ffi::c_int;
                }
                let mut num_chars: ::core::ffi::c_int = numc;
                numc *= utf_char2len(c);
                let mut oldp: *mut ::core::ffi::c_char = get_cursor_line_ptr();
                let mut oldlen: colnr_T = get_cursor_line_len();
                let mut newp_size: size_t =
                    (bd.textcol as size_t).wrapping_add(bd.startspaces as size_t);
                if had_ctrl_v_cr as ::core::ffi::c_int != 0
                    || c != '\r' as ::core::ffi::c_int && c != '\n' as ::core::ffi::c_int
                {
                    newp_size = newp_size.wrapping_add(numc as size_t);
                    if bd.is_short == 0 {
                        newp_size = newp_size.wrapping_add(
                            (bd.endspaces + oldlen as ::core::ffi::c_int
                                - bd.textcol as ::core::ffi::c_int
                                - bd.textlen) as size_t,
                        );
                    }
                }
                let mut newp: *mut ::core::ffi::c_char =
                    xmallocz(newp_size) as *mut ::core::ffi::c_char;
                memmove(
                    newp as *mut ::core::ffi::c_void,
                    oldp as *const ::core::ffi::c_void,
                    bd.textcol as size_t,
                );
                oldp = oldp.offset((bd.textcol as ::core::ffi::c_int + bd.textlen) as isize);
                memset(
                    newp.offset(bd.textcol as isize) as *mut ::core::ffi::c_void,
                    ' ' as ::core::ffi::c_int,
                    bd.startspaces as size_t,
                );
                let mut after_p_len: size_t = 0 as size_t;
                let mut col: ::core::ffi::c_int =
                    oldlen as ::core::ffi::c_int - bd.textcol as ::core::ffi::c_int - bd.textlen
                        + 1 as ::core::ffi::c_int;
                '_c2rust_label: {
                    if col >= 0 as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                            b"col >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/ops.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            1179 as ::core::ffi::c_uint,
                            b"int op_replace(oparg_T *, int)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                let mut newrows: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut newcols: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                if had_ctrl_v_cr as ::core::ffi::c_int != 0
                    || c != '\r' as ::core::ffi::c_int && c != '\n' as ::core::ffi::c_int
                {
                    let mut newp_len: ::core::ffi::c_int =
                        bd.textcol as ::core::ffi::c_int + bd.startspaces;
                    loop {
                        num_chars -= 1;
                        if num_chars < 0 as ::core::ffi::c_int {
                            break;
                        }
                        newp_len += utf_char2bytes(c, newp.offset(newp_len as isize));
                    }
                    if bd.is_short == 0 {
                        memset(
                            newp.offset(newp_len as isize) as *mut ::core::ffi::c_void,
                            ' ' as ::core::ffi::c_int,
                            bd.endspaces as size_t,
                        );
                        newp_len += bd.endspaces;
                        memmove(
                            newp.offset(newp_len as isize) as *mut ::core::ffi::c_void,
                            oldp as *const ::core::ffi::c_void,
                            col as size_t,
                        );
                    }
                    newcols = (newp_len as colnr_T - bd.textcol) as ::core::ffi::c_int;
                } else {
                    after_p_len = col as size_t;
                    after_p = xmalloc(after_p_len) as *mut ::core::ffi::c_char;
                    memmove(
                        after_p as *mut ::core::ffi::c_void,
                        oldp as *const ::core::ffi::c_void,
                        after_p_len,
                    );
                    newrows = 1 as ::core::ffi::c_int;
                }
                ml_replace((*curwin.get()).w_cursor.lnum, newp, false_0 != 0);
                (*curbuf_splice_pending.ptr()) += 1;
                let mut baselnum: linenr_T = (*curwin.get()).w_cursor.lnum;
                if !after_p.is_null() {
                    let c2rust_fresh7 = (*curwin.get()).w_cursor.lnum;
                    (*curwin.get()).w_cursor.lnum = (*curwin.get()).w_cursor.lnum + 1;
                    ml_append(c2rust_fresh7, after_p, after_p_len as colnr_T, false_0 != 0);
                    appended_lines_mark((*curwin.get()).w_cursor.lnum, 1 as ::core::ffi::c_int);
                    (*oap).end.lnum += 1;
                    xfree(after_p as *mut ::core::ffi::c_void);
                }
                (*curbuf_splice_pending.ptr()) -= 1;
                extmark_splice(
                    curbuf.get(),
                    baselnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    bd.textcol,
                    0 as ::core::ffi::c_int,
                    bd.textlen as colnr_T,
                    bd.textlen as bcount_t,
                    newrows,
                    newcols as colnr_T,
                    (newrows + newcols) as bcount_t,
                    kExtmarkUndo,
                );
            }
            (*curwin.get()).w_cursor.lnum += 1;
        }
    } else {
        if (*oap).motion_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
            (*oap).start.col = 0 as ::core::ffi::c_int as colnr_T;
            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
            (*oap).end.col = ml_get_len((*oap).end.lnum);
            if (*oap).end.col != 0 {
                (*oap).end.col -= 1;
            }
        } else if !(*oap).inclusive {
            dec(&raw mut (*oap).end);
        }
        while ltoreq((*curwin.get()).w_cursor, (*oap).end) {
            let mut done: bool = false_0 != 0;
            n = gchar_cursor();
            if n != NUL {
                let mut new_byte_len: ::core::ffi::c_int = utf_char2len(c);
                let mut old_byte_len: ::core::ffi::c_int = utfc_ptr2len(get_cursor_pos_ptr());
                if new_byte_len > 1 as ::core::ffi::c_int || old_byte_len > 1 as ::core::ffi::c_int
                {
                    if (*curwin.get()).w_cursor.lnum == (*oap).end.lnum {
                        (*oap).end.col += new_byte_len - old_byte_len;
                    }
                    replace_character(c);
                    done = true_0 != 0;
                } else {
                    if n == TAB {
                        let mut end_vcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        if (*curwin.get()).w_cursor.lnum == (*oap).end.lnum {
                            end_vcol = getviscol2((*oap).end.col, (*oap).end.coladd);
                        }
                        coladvance_force(getviscol());
                        if (*curwin.get()).w_cursor.lnum == (*oap).end.lnum {
                            getvpos(curwin.get(), &raw mut (*oap).end, end_vcol as colnr_T);
                        }
                    }
                    if gchar_cursor() != NUL {
                        pbyte((*curwin.get()).w_cursor, c);
                        done = true_0 != 0;
                    }
                }
            }
            if !done
                && virtual_op.get() as ::core::ffi::c_int != 0
                && (*curwin.get()).w_cursor.lnum == (*oap).end.lnum
            {
                let mut virtcols: ::core::ffi::c_int = (*oap).end.coladd as ::core::ffi::c_int;
                if (*curwin.get()).w_cursor.lnum == (*oap).start.lnum
                    && (*oap).start.col == (*oap).end.col
                    && (*oap).start.coladd != 0
                {
                    virtcols -= (*oap).start.coladd as ::core::ffi::c_int;
                }
                coladvance_force(getviscol2((*oap).end.col, (*oap).end.coladd) + 1 as colnr_T);
                (*curwin.get()).w_cursor.col -= virtcols + 1 as ::core::ffi::c_int;
                while virtcols >= 0 as ::core::ffi::c_int {
                    if utf_char2len(c) > 1 as ::core::ffi::c_int {
                        replace_character(c);
                    } else {
                        pbyte((*curwin.get()).w_cursor, c);
                    }
                    if inc(&raw mut (*curwin.get()).w_cursor) == -1 as ::core::ffi::c_int {
                        break;
                    }
                    virtcols -= 1;
                }
            }
            if inc_cursor() == -1 as ::core::ffi::c_int {
                break;
            }
        }
    }
    (*curwin.get()).w_cursor = (*oap).start;
    check_cursor(curwin.get());
    changed_lines(
        curbuf.get(),
        (*oap).start.lnum,
        (*oap).start.col,
        (*oap).end.lnum + 1 as linenr_T,
        0 as linenr_T,
        true_0 != 0,
    );
    if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        (*curbuf.get()).b_op_start = (*oap).start;
        (*curbuf.get()).b_op_end = (*oap).end;
    }
    return OK;
}
pub unsafe extern "C" fn op_tilde(mut oap: *mut oparg_T) {
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
    let mut did_change: bool = false_0 != 0;
    if u_save(
        (*oap).start.lnum - 1 as linenr_T,
        (*oap).end.lnum + 1 as linenr_T,
    ) == FAIL
    {
        return;
    }
    let mut pos: pos_T = (*oap).start;
    if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
        while pos.lnum <= (*oap).end.lnum {
            block_prep(oap, &raw mut bd, pos.lnum, false_0 != 0);
            pos.col = bd.textcol;
            let mut one_change: bool = swapchars((*oap).op_type, &raw mut pos, bd.textlen) != 0;
            did_change = did_change as ::core::ffi::c_int | one_change as ::core::ffi::c_int != 0;
            pos.lnum += 1;
        }
        if did_change {
            changed_lines(
                curbuf.get(),
                (*oap).start.lnum,
                0 as colnr_T,
                (*oap).end.lnum + 1 as linenr_T,
                0 as linenr_T,
                true_0 != 0,
            );
        }
    } else {
        if (*oap).motion_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
            (*oap).start.col = 0 as ::core::ffi::c_int as colnr_T;
            pos.col = 0 as ::core::ffi::c_int as colnr_T;
            (*oap).end.col = ml_get_len((*oap).end.lnum);
            if (*oap).end.col != 0 {
                (*oap).end.col -= 1;
            }
        } else if !(*oap).inclusive {
            dec(&raw mut (*oap).end);
        }
        if pos.lnum == (*oap).end.lnum {
            did_change = swapchars(
                (*oap).op_type,
                &raw mut pos,
                (*oap).end.col as ::core::ffi::c_int - pos.col as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int,
            ) != 0;
        } else {
            loop {
                did_change = did_change as ::core::ffi::c_int
                    | swapchars(
                        (*oap).op_type,
                        &raw mut pos,
                        if pos.lnum == (*oap).end.lnum {
                            (*oap).end.col as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                        } else {
                            ml_get_pos_len(&raw mut pos)
                        },
                    )
                    != 0;
                if ltoreq((*oap).end, pos) as ::core::ffi::c_int != 0
                    || inc(&raw mut pos) == -1 as ::core::ffi::c_int
                {
                    break;
                }
            }
        }
        if did_change {
            changed_lines(
                curbuf.get(),
                (*oap).start.lnum,
                (*oap).start.col,
                (*oap).end.lnum + 1 as linenr_T,
                0 as linenr_T,
                true_0 != 0,
            );
        }
    }
    if !did_change && (*oap).is_VIsual as ::core::ffi::c_int != 0 {
        redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
    }
    if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        (*curbuf.get()).b_op_start = (*oap).start;
        (*curbuf.get()).b_op_end = (*oap).end;
    }
    if (*oap).line_count as OptInt > p_report.get() {
        smsg(
            0 as ::core::ffi::c_int,
            ngettext(
                b"%ld line changed\0".as_ptr() as *const ::core::ffi::c_char,
                b"%ld lines changed\0".as_ptr() as *const ::core::ffi::c_char,
                (*oap).line_count as ::core::ffi::c_ulong,
            ),
            (*oap).line_count as int64_t,
        );
    }
}
unsafe extern "C" fn swapchars(
    mut op_type: ::core::ffi::c_int,
    mut pos: *mut pos_T,
    mut length: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut did_change: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut todo: ::core::ffi::c_int = length;
    while todo > 0 as ::core::ffi::c_int {
        let len: ::core::ffi::c_int = utfc_ptr2len(ml_get_pos(pos));
        if len > 0 as ::core::ffi::c_int {
            todo -= len - 1 as ::core::ffi::c_int;
        }
        did_change |= swapchar(op_type, pos) as ::core::ffi::c_int;
        if inc(pos) == -1 as ::core::ffi::c_int {
            break;
        }
        todo -= 1;
    }
    return did_change;
}
pub unsafe extern "C" fn swapchar(mut op_type: ::core::ffi::c_int, mut pos: *mut pos_T) -> bool {
    let c: ::core::ffi::c_int = gchar_pos(pos);
    if c >= 0x80 as ::core::ffi::c_int && op_type == OP_ROT13 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    let mut nc: ::core::ffi::c_int = c;
    if mb_islower(c) {
        if op_type == OP_ROT13 as ::core::ffi::c_int {
            nc = (c - 'a' as ::core::ffi::c_int + 13 as ::core::ffi::c_int)
                % 26 as ::core::ffi::c_int
                + 'a' as ::core::ffi::c_int;
        } else if op_type != OP_LOWER as ::core::ffi::c_int {
            nc = mb_toupper(c);
        }
    } else if mb_isupper(c) {
        if op_type == OP_ROT13 as ::core::ffi::c_int {
            nc = (c - 'A' as ::core::ffi::c_int + 13 as ::core::ffi::c_int)
                % 26 as ::core::ffi::c_int
                + 'A' as ::core::ffi::c_int;
        } else if op_type != OP_UPPER as ::core::ffi::c_int {
            nc = mb_tolower(c);
        }
    }
    if nc != c {
        if c >= 0x80 as ::core::ffi::c_int || nc >= 0x80 as ::core::ffi::c_int {
            let mut sp: pos_T = (*curwin.get()).w_cursor;
            (*curwin.get()).w_cursor = *pos;
            del_bytes(
                utf_ptr2len(get_cursor_pos_ptr()),
                false_0 != 0,
                false_0 != 0,
            );
            ins_char(nc);
            (*curwin.get()).w_cursor = sp;
        } else {
            pbyte(*pos, nc);
        }
        return true_0 != 0;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn op_insert(mut oap: *mut oparg_T, mut count1: ::core::ffi::c_int) {
    let mut pre_textlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut ind_pre_col: colnr_T = 0 as colnr_T;
    let mut ind_pre_vcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
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
    bd.is_MAX = ((*curwin.get()).w_curswant == MAXCOL as ::core::ffi::c_int) as ::core::ffi::c_int;
    (*curwin.get()).w_cursor.lnum = (*oap).start.lnum;
    redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
    update_screen();
    if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
        if (*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int {
            let mut old_ve_flags: ::core::ffi::c_uint = (*curwin.get()).w_onebuf_opt.wo_ve_flags;
            if u_save_cursor() == FAIL {
                return;
            }
            (*curwin.get()).w_onebuf_opt.wo_ve_flags =
                kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint;
            coladvance_force(if (*oap).op_type == OP_APPEND as ::core::ffi::c_int {
                (*oap).end_vcol + 1 as colnr_T
            } else {
                getviscol()
            });
            if (*oap).op_type == OP_APPEND as ::core::ffi::c_int {
                (*curwin.get()).w_cursor.col -= 1;
            }
            (*curwin.get()).w_onebuf_opt.wo_ve_flags = old_ve_flags;
        }
        block_prep(oap, &raw mut bd, (*oap).start.lnum, true_0 != 0);
        ind_pre_col = getwhitecols_curline() as colnr_T;
        ind_pre_vcol = get_indent();
        pre_textlen = (ml_get_len((*oap).start.lnum) - bd.textcol) as ::core::ffi::c_int;
        if (*oap).op_type == OP_APPEND as ::core::ffi::c_int {
            pre_textlen -= bd.textlen;
        }
    }
    if (*oap).op_type == OP_APPEND as ::core::ffi::c_int {
        if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int
            && (*curwin.get()).w_cursor.coladd == 0 as ::core::ffi::c_int
        {
            (*curwin.get()).w_set_curswant = true_0;
            while *get_cursor_pos_ptr() as ::core::ffi::c_int != NUL
                && (*curwin.get()).w_cursor.col < bd.textcol as ::core::ffi::c_int + bd.textlen
            {
                (*curwin.get()).w_cursor.col += 1;
            }
            if bd.is_short != 0 && bd.is_MAX == 0 {
                if u_save_cursor() == FAIL {
                    return;
                }
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < bd.endspaces {
                    ins_char(' ' as ::core::ffi::c_int);
                    i += 1;
                }
                bd.textlen += bd.endspaces;
            }
        } else {
            (*curwin.get()).w_cursor = (*oap).end;
            check_cursor_col(curwin.get());
            if !(*ml_get((*curwin.get()).w_cursor.lnum) as ::core::ffi::c_int == NUL)
                && (*oap).start_vcol != (*oap).end_vcol
            {
                inc_cursor();
            }
        }
    }
    let mut t1: pos_T = (*oap).start;
    let start_insert: pos_T = (*curwin.get()).w_cursor;
    edit(NUL, false_0 != 0, count1);
    if t1.lnum == (*curbuf.get()).b_op_start_orig.lnum
        && lt((*curbuf.get()).b_op_start_orig, t1) as ::core::ffi::c_int != 0
    {
        (*oap).start = (*curbuf.get()).b_op_start_orig;
    }
    if (*curwin.get()).w_cursor.lnum != (*oap).start.lnum
        || got_int.get() as ::core::ffi::c_int != 0
    {
        return;
    }
    if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
        let mut ind_post_vcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut bd2: block_def = block_def {
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
        let mut did_indent: bool = false_0 != 0;
        let mut ind_post_col: colnr_T = getwhitecols_curline() as colnr_T;
        if (*curbuf.get()).b_op_start.col > ind_pre_col && ind_post_col > ind_pre_col {
            bd.textcol += ind_post_col - ind_pre_col;
            ind_post_vcol = get_indent();
            bd.start_vcol += ind_post_vcol - ind_pre_vcol;
            did_indent = true_0 != 0;
        }
        if (*oap).start.lnum == (*curbuf.get()).b_op_start_orig.lnum
            && bd.is_MAX == 0
            && !did_indent
        {
            let t: ::core::ffi::c_int = getviscol2(
                (*curbuf.get()).b_op_start_orig.col,
                (*curbuf.get()).b_op_start_orig.coladd,
            );
            if (*oap).op_type == OP_INSERT as ::core::ffi::c_int
                && (*oap).start.col + (*oap).start.coladd
                    != (*curbuf.get()).b_op_start_orig.col + (*curbuf.get()).b_op_start_orig.coladd
            {
                (*oap).start.col = (*curbuf.get()).b_op_start_orig.col;
                pre_textlen -= (t as colnr_T - (*oap).start_vcol) as ::core::ffi::c_int;
                (*oap).start_vcol = t as colnr_T;
            } else if (*oap).op_type == OP_APPEND as ::core::ffi::c_int
                && (*oap).start.col + (*oap).start.coladd
                    >= (*curbuf.get()).b_op_start_orig.col + (*curbuf.get()).b_op_start_orig.coladd
            {
                (*oap).start.col = (*curbuf.get()).b_op_start_orig.col;
                pre_textlen += bd.textlen;
                pre_textlen -= (t as colnr_T - (*oap).start_vcol) as ::core::ffi::c_int;
                (*oap).start_vcol = t as colnr_T;
                (*oap).op_type = OP_INSERT as ::core::ffi::c_int;
            }
        }
        if did_indent as ::core::ffi::c_int != 0
            && bd.textcol - ind_post_col > 0 as ::core::ffi::c_int
        {
            (*oap).start.col += ind_post_col - ind_pre_col;
            (*oap).start_vcol += ind_post_vcol - ind_pre_vcol;
            (*oap).end.col += ind_post_col - ind_pre_col;
            (*oap).end_vcol += ind_post_vcol - ind_pre_vcol;
        }
        block_prep(oap, &raw mut bd2, (*oap).start.lnum, true_0 != 0);
        if did_indent as ::core::ffi::c_int != 0
            && bd.textcol - ind_post_col > 0 as ::core::ffi::c_int
        {
            (*oap).start.col -= ind_post_col - ind_pre_col;
            (*oap).start_vcol -= ind_post_vcol - ind_pre_vcol;
            (*oap).end.col -= ind_post_col - ind_pre_col;
            (*oap).end_vcol -= ind_post_vcol - ind_pre_vcol;
        }
        if bd.is_MAX == 0 || bd2.textlen < bd.textlen {
            if (*oap).op_type == OP_APPEND as ::core::ffi::c_int {
                pre_textlen += bd2.textlen - bd.textlen;
                if bd2.endspaces != 0 {
                    bd2.textlen -= 1;
                }
            }
            bd.textcol = bd2.textcol;
            bd.textlen = bd2.textlen;
        }
        let mut firstline: *mut ::core::ffi::c_char = ml_get((*oap).start.lnum);
        let mut len: colnr_T = ml_get_len((*oap).start.lnum);
        let mut add: colnr_T = bd.textcol;
        let mut offset: colnr_T = 0 as colnr_T;
        if (*oap).op_type == OP_APPEND as ::core::ffi::c_int {
            add += bd.textlen;
            if bd.is_MAX != 0
                && start_insert.lnum == (*Insstart.ptr()).lnum
                && start_insert.col > (*Insstart.ptr()).col
            {
                offset = start_insert.col - (*Insstart.ptr()).col;
                add -= offset;
                if (*oap).end_vcol > offset {
                    (*oap).end_vcol -= offset as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
                } else {
                    return;
                }
            }
        }
        add = if add < len { add } else { len };
        firstline = firstline.offset(add as isize);
        len -= add;
        let mut ins_len: ::core::ffi::c_int =
            len as ::core::ffi::c_int - pre_textlen - offset as ::core::ffi::c_int;
        if pre_textlen >= 0 as ::core::ffi::c_int && ins_len > 0 as ::core::ffi::c_int {
            let mut ins_text: *mut ::core::ffi::c_char =
                xmemdupz(firstline as *const ::core::ffi::c_void, ins_len as size_t)
                    as *mut ::core::ffi::c_char;
            if u_save((*oap).start.lnum, (*oap).end.lnum + 1 as linenr_T) == OK {
                block_insert(
                    oap,
                    ins_text,
                    ins_len as size_t,
                    (*oap).op_type == OP_INSERT as ::core::ffi::c_int,
                    &raw mut bd,
                );
            }
            (*curwin.get()).w_cursor.col = (*oap).start.col;
            check_cursor(curwin.get());
            xfree(ins_text as *mut ::core::ffi::c_void);
        }
    }
}
pub unsafe extern "C" fn op_change(mut oap: *mut oparg_T) -> ::core::ffi::c_int {
    let mut pre_textlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut pre_indent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut firstline: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
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
    let mut l: colnr_T = (*oap).start.col;
    if (*oap).motion_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
        l = 0 as ::core::ffi::c_int as colnr_T;
        can_si.set(may_do_si());
    }
    if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
        if u_save_cursor() == FAIL {
            return false_0;
        }
    } else if op_delete(oap) == FAIL {
        return false_0;
    }
    if l > (*curwin.get()).w_cursor.col
        && !(*ml_get((*curwin.get()).w_cursor.lnum) as ::core::ffi::c_int == NUL)
        && virtual_op.get() as u64 == 0
    {
        inc_cursor();
    }
    if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
        if virtual_op.get() as ::core::ffi::c_int != 0
            && ((*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int || gchar_cursor() == NUL)
        {
            coladvance_force(getviscol());
        }
        firstline = ml_get((*oap).start.lnum);
        pre_textlen = ml_get_len((*oap).start.lnum) as ::core::ffi::c_int;
        pre_indent = getwhitecols(firstline) as ::core::ffi::c_int;
        bd.textcol = (*curwin.get()).w_cursor.col;
    }
    if (*oap).motion_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
        fix_indent();
    }
    let save_finish_op: bool = finish_op.get();
    finish_op.set(false_0 != 0);
    let mut retval: ::core::ffi::c_int =
        edit(NUL, false_0 != 0, 1 as ::core::ffi::c_int) as ::core::ffi::c_int;
    finish_op.set(save_finish_op);
    if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int
        && (*oap).start.lnum != (*oap).end.lnum
        && !got_int.get()
    {
        firstline = ml_get((*oap).start.lnum);
        if bd.textcol > pre_indent {
            let mut new_indent: ::core::ffi::c_int = getwhitecols(firstline) as ::core::ffi::c_int;
            pre_textlen += new_indent - pre_indent;
            bd.textcol += new_indent - pre_indent;
        }
        let mut ins_len: ::core::ffi::c_int = ml_get_len((*oap).start.lnum) - pre_textlen;
        if ins_len > 0 as ::core::ffi::c_int {
            let mut ins_text: *mut ::core::ffi::c_char =
                xmalloc((ins_len as size_t).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
            xmemcpyz(
                ins_text as *mut ::core::ffi::c_void,
                firstline.offset(bd.textcol as isize) as *const ::core::ffi::c_void,
                ins_len as size_t,
            );
            let mut linenr: linenr_T = (*oap).start.lnum + 1 as linenr_T;
            while linenr <= (*oap).end.lnum {
                block_prep(oap, &raw mut bd, linenr, true_0 != 0);
                if bd.is_short == 0 || virtual_op.get() as ::core::ffi::c_int != 0 {
                    let mut vpos: pos_T = pos_T {
                        lnum: 0,
                        col: 0,
                        coladd: 0,
                    };
                    if bd.is_short != 0 {
                        vpos.lnum = linenr;
                        getvpos(curwin.get(), &raw mut vpos, (*oap).start_vcol);
                    } else {
                        vpos.coladd = 0 as ::core::ffi::c_int as colnr_T;
                    }
                    let mut oldp: *mut ::core::ffi::c_char = ml_get(linenr);
                    let mut newp: *mut ::core::ffi::c_char = xmalloc(
                        (ml_get_len(linenr) as size_t)
                            .wrapping_add(vpos.coladd as size_t)
                            .wrapping_add(ins_len as size_t)
                            .wrapping_add(1 as size_t),
                    )
                        as *mut ::core::ffi::c_char;
                    memmove(
                        newp as *mut ::core::ffi::c_void,
                        oldp as *const ::core::ffi::c_void,
                        bd.textcol as size_t,
                    );
                    let mut newlen: ::core::ffi::c_int = bd.textcol as ::core::ffi::c_int;
                    memset(
                        newp.offset(newlen as isize) as *mut ::core::ffi::c_void,
                        ' ' as ::core::ffi::c_int,
                        vpos.coladd as size_t,
                    );
                    newlen += vpos.coladd as ::core::ffi::c_int;
                    memmove(
                        newp.offset(newlen as isize) as *mut ::core::ffi::c_void,
                        ins_text as *const ::core::ffi::c_void,
                        ins_len as size_t,
                    );
                    newlen += ins_len;
                    strcpy(
                        newp.offset(newlen as isize),
                        oldp.offset(bd.textcol as isize),
                    );
                    ml_replace(linenr, newp, false_0 != 0);
                    extmark_splice_cols(
                        curbuf.get(),
                        linenr as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                        bd.textcol,
                        0 as colnr_T,
                        vpos.coladd + ins_len as colnr_T,
                        kExtmarkUndo,
                    );
                }
                linenr += 1;
            }
            check_cursor(curwin.get());
            changed_lines(
                curbuf.get(),
                (*oap).start.lnum + 1 as linenr_T,
                0 as colnr_T,
                (*oap).end.lnum + 1 as linenr_T,
                0 as linenr_T,
                true_0 != 0,
            );
            xfree(ins_text as *mut ::core::ffi::c_void);
        }
    }
    auto_format(false_0 != 0, true_0 != 0);
    return retval;
}
pub unsafe extern "C" fn adjust_cursor_eol() {
    let mut cur_ve_flags: ::core::ffi::c_uint = get_ve_flags(curwin.get());
    let adj_cursor: bool = (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
        && gchar_cursor() == NUL
        && cur_ve_flags & kOptVeFlagOnemore as ::core::ffi::c_int as ::core::ffi::c_uint
            == 0 as ::core::ffi::c_uint
        && cur_ve_flags & kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint
            == 0 as ::core::ffi::c_uint
        && !(restart_edit.get() != 0 || State.get() & MODE_INSERT as ::core::ffi::c_int != 0);
    if !adj_cursor {
        return;
    }
    dec_cursor();
    if cur_ve_flags == kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint {
        let mut scol: colnr_T = 0;
        let mut ecol: colnr_T = 0;
        getvcol(
            curwin.get(),
            &raw mut (*curwin.get()).w_cursor,
            &raw mut scol,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut ecol,
        );
        (*curwin.get()).w_cursor.coladd = (ecol as ::core::ffi::c_int - scol as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as colnr_T;
    }
}
pub unsafe extern "C" fn skip_comment(
    mut line: *mut ::core::ffi::c_char,
    mut process: bool,
    mut include_space: bool,
    mut is_comment: *mut bool,
) -> *mut ::core::ffi::c_char {
    let mut comment_flags: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut leader_offset: ::core::ffi::c_int =
        get_last_leader_offset(line, &raw mut comment_flags);
    *is_comment = false_0 != 0;
    if leader_offset != -1 as ::core::ffi::c_int {
        while *comment_flags != 0 {
            if *comment_flags as ::core::ffi::c_int == COM_END
                || *comment_flags as ::core::ffi::c_int == ':' as ::core::ffi::c_int
            {
                break;
            }
            comment_flags = comment_flags.offset(1);
        }
        if *comment_flags as ::core::ffi::c_int != COM_END {
            *is_comment = true_0 != 0;
        }
    }
    if process as ::core::ffi::c_int == false_0 {
        return line;
    }
    let mut lead_len: ::core::ffi::c_int =
        get_leader_len(line, &raw mut comment_flags, false_0 != 0, include_space);
    if lead_len == 0 as ::core::ffi::c_int {
        return line;
    }
    while *comment_flags != 0 {
        if *comment_flags as ::core::ffi::c_int == COM_END
            || *comment_flags as ::core::ffi::c_int == ':' as ::core::ffi::c_int
        {
            break;
        }
        comment_flags = comment_flags.offset(1);
    }
    if *comment_flags as ::core::ffi::c_int == ':' as ::core::ffi::c_int
        || *comment_flags as ::core::ffi::c_int == NUL
    {
        line = line.offset(lead_len as isize);
    }
    return line;
}
pub unsafe extern "C" fn do_join(
    mut count: size_t,
    mut insert_space: bool,
    mut save_undo: bool,
    mut use_formatoptions: bool,
    mut setmark: bool,
) -> ::core::ffi::c_int {
    let mut col: colnr_T = 0;
    let mut newp_len: size_t = 0;
    let mut newp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut t_1: linenr_T = 0;
    let mut curr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut curr_start: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cend: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut endcurr1: ::core::ffi::c_int = NUL;
    let mut endcurr2: ::core::ffi::c_int = NUL;
    let mut currsize: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut sumsize: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut ret: ::core::ffi::c_int = OK;
    let mut comments: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
    let mut remove_comments: bool = use_formatoptions as ::core::ffi::c_int != 0
        && has_format_option(FO_REMOVE_COMS) as ::core::ffi::c_int != 0;
    let mut prev_was_comment: bool = false_0 != 0;
    '_c2rust_label: {
        if count >= 1 as size_t {
        } else {
            __assert_fail(
                b"count >= 1\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/ops.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1899 as ::core::ffi::c_uint,
                b"int do_join(size_t, _Bool, _Bool, _Bool, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if save_undo as ::core::ffi::c_int != 0
        && u_save(
            (*curwin.get()).w_cursor.lnum - 1 as linenr_T,
            (*curwin.get()).w_cursor.lnum + count as linenr_T,
        ) == FAIL
    {
        return FAIL;
    }
    let mut spaces: *mut ::core::ffi::c_char =
        xcalloc(count, 1 as size_t) as *mut ::core::ffi::c_char;
    if remove_comments {
        comments =
            xcalloc(count, ::core::mem::size_of::<::core::ffi::c_int>()) as *mut ::core::ffi::c_int;
    }
    let mut t: linenr_T = 0 as linenr_T;
    '_theend: {
        while t < count as linenr_T {
            curr_start = ml_get((*curwin.get()).w_cursor.lnum + t);
            curr = curr_start;
            if t == 0 as linenr_T
                && setmark as ::core::ffi::c_int != 0
                && (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
            {
                (*(*curwin.get()).w_buffer).b_op_start.lnum = (*curwin.get()).w_cursor.lnum;
                (*(*curwin.get()).w_buffer).b_op_start.col = strlen(curr) as colnr_T;
            }
            if remove_comments {
                if t > 0 as linenr_T && prev_was_comment as ::core::ffi::c_int != 0 {
                    let mut new_curr: *mut ::core::ffi::c_char =
                        skip_comment(curr, true_0 != 0, insert_space, &raw mut prev_was_comment);
                    *comments.offset(t as isize) = new_curr.offset_from(curr) as ::core::ffi::c_int;
                    curr = new_curr;
                } else {
                    curr =
                        skip_comment(curr, false_0 != 0, insert_space, &raw mut prev_was_comment);
                }
            }
            if insert_space as ::core::ffi::c_int != 0 && t > 0 as linenr_T {
                curr = skipwhite(curr);
                if *curr as ::core::ffi::c_int != NUL
                    && *curr as ::core::ffi::c_int != ')' as ::core::ffi::c_int
                    && sumsize != 0 as ::core::ffi::c_int
                    && endcurr1 != TAB
                    && (!has_format_option(FO_MBYTE_JOIN)
                        || utf_ptr2char(curr) < 0x100 as ::core::ffi::c_int
                            && endcurr1 < 0x100 as ::core::ffi::c_int)
                    && (!has_format_option(FO_MBYTE_JOIN2)
                        || utf_ptr2char(curr) < 0x100 as ::core::ffi::c_int
                            && !utf_eat_space(endcurr1)
                        || endcurr1 < 0x100 as ::core::ffi::c_int
                            && !utf_eat_space(utf_ptr2char(curr)))
                {
                    if endcurr1 == ' ' as ::core::ffi::c_int {
                        endcurr1 = endcurr2;
                    } else {
                        *spaces.offset(t as isize) += 1;
                    }
                    if p_js.get() != 0
                        && (endcurr1 == '.' as ::core::ffi::c_int
                            || endcurr1 == '?' as ::core::ffi::c_int
                            || endcurr1 == '!' as ::core::ffi::c_int)
                    {
                        *spaces.offset(t as isize) += 1;
                    }
                }
            }
            if t > 0 as linenr_T && curbuf_splice_pending.get() == 0 as ::core::ffi::c_int {
                let mut removed: colnr_T = curr.offset_from(curr_start) as colnr_T;
                extmark_splice(
                    curbuf.get(),
                    (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    sumsize as colnr_T,
                    1 as ::core::ffi::c_int,
                    removed,
                    (removed as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as bcount_t,
                    0 as ::core::ffi::c_int,
                    *spaces.offset(t as isize) as colnr_T,
                    *spaces.offset(t as isize) as bcount_t,
                    kExtmarkUndo,
                );
            }
            currsize = strlen(curr) as ::core::ffi::c_int;
            sumsize += currsize + *spaces.offset(t as isize) as ::core::ffi::c_int;
            endcurr2 = NUL;
            endcurr1 = endcurr2;
            if insert_space as ::core::ffi::c_int != 0 && currsize > 0 as ::core::ffi::c_int {
                cend = curr.offset(currsize as isize);
                cend = cend.offset(
                    -((utf_head_off(curr, cend.offset(-(1 as ::core::ffi::c_int as isize)))
                        + 1 as ::core::ffi::c_int) as isize),
                );
                endcurr1 = utf_ptr2char(cend);
                if cend > curr {
                    cend = cend.offset(
                        -((utf_head_off(curr, cend.offset(-(1 as ::core::ffi::c_int as isize)))
                            + 1 as ::core::ffi::c_int) as isize),
                    );
                    endcurr2 = utf_ptr2char(cend);
                }
            }
            line_breakcheck();
            if got_int.get() {
                ret = FAIL;
                break '_theend;
            } else {
                t += 1;
            }
        }
        col = sumsize as colnr_T
            - currsize as colnr_T
            - *spaces.offset(count.wrapping_sub(1 as size_t) as isize) as colnr_T;
        newp_len = sumsize as size_t;
        newp = xmallocz(newp_len) as *mut ::core::ffi::c_char;
        cend = newp.offset(sumsize as isize);
        (*curbuf_splice_pending.ptr()) += 1;
        let mut t_0: linenr_T = count as linenr_T - 1 as linenr_T;
        loop {
            cend = cend.offset(-(currsize as isize));
            memmove(
                cend as *mut ::core::ffi::c_void,
                curr as *const ::core::ffi::c_void,
                currsize as size_t,
            );
            if *spaces.offset(t_0 as isize) as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
                cend = cend.offset(-(*spaces.offset(t_0 as isize) as ::core::ffi::c_int as isize));
                memset(
                    cend as *mut ::core::ffi::c_void,
                    ' ' as ::core::ffi::c_int,
                    *spaces.offset(t_0 as isize) as size_t,
                );
            }
            let spaces_removed: ::core::ffi::c_int = (curr.offset_from(curr_start)
                - *spaces.offset(t_0 as isize) as isize)
                as ::core::ffi::c_int;
            let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum + t_0;
            let mut mincol: colnr_T = 0 as colnr_T;
            let mut lnum_amount: linenr_T = -t_0;
            let mut col_amount: colnr_T =
                (cend.offset_from(newp) - spaces_removed as isize) as colnr_T;
            mark_col_adjust(lnum, mincol, lnum_amount, col_amount, spaces_removed);
            if t_0 == 0 as linenr_T {
                break;
            }
            curr_start = ml_get((*curwin.get()).w_cursor.lnum + t_0 - 1 as linenr_T);
            curr = curr_start;
            if remove_comments {
                curr = curr.offset(*comments.offset((t_0 - 1 as linenr_T) as isize) as isize);
            }
            if insert_space as ::core::ffi::c_int != 0 && t_0 > 1 as linenr_T {
                curr = skipwhite(curr);
            }
            currsize = strlen(curr) as ::core::ffi::c_int;
            t_0 -= 1;
        }
        ml_replace_len((*curwin.get()).w_cursor.lnum, newp, newp_len, false_0 != 0);
        if setmark as ::core::ffi::c_int != 0
            && (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            (*(*curwin.get()).w_buffer).b_op_end.lnum = (*curwin.get()).w_cursor.lnum;
            (*(*curwin.get()).w_buffer).b_op_end.col = sumsize as colnr_T;
        }
        changed_lines(
            curbuf.get(),
            (*curwin.get()).w_cursor.lnum,
            currsize as colnr_T,
            (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
            0 as linenr_T,
            true_0 != 0,
        );
        t_1 = (*curwin.get()).w_cursor.lnum;
        (*curwin.get()).w_cursor.lnum += 1;
        del_lines(count as linenr_T - 1 as linenr_T, false_0 != 0);
        (*curwin.get()).w_cursor.lnum = t_1;
        (*curbuf_splice_pending.ptr()) -= 1;
        (*curbuf.get()).deleted_bytes2 = 0 as size_t;
        (*curwin.get()).w_cursor.col = (if !vim_strchr(p_cpo.get(), CPO_JOINCOL).is_null() {
            currsize
        } else {
            col as ::core::ffi::c_int
        }) as colnr_T;
        check_cursor_col(curwin.get());
        (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
        (*curwin.get()).w_set_curswant = true_0;
    }
    xfree(spaces as *mut ::core::ffi::c_void);
    if remove_comments {
        xfree(comments as *mut ::core::ffi::c_void);
    }
    return ret;
}
pub unsafe extern "C" fn reset_lbr() -> bool {
    if (*curwin.get()).w_onebuf_opt.wo_lbr == 0 {
        return false_0 != 0;
    }
    (*curwin.get()).w_onebuf_opt.wo_lbr = false_0;
    (*curwin.get()).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL);
    return true_0 != 0;
}
pub unsafe extern "C" fn restore_lbr(mut lbr_saved: bool) {
    if (*curwin.get()).w_onebuf_opt.wo_lbr != 0 || !lbr_saved {
        return;
    }
    (*curwin.get()).w_onebuf_opt.wo_lbr = true_0;
    (*curwin.get()).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL);
}
pub unsafe extern "C" fn block_prep(
    mut oap: *mut oparg_T,
    mut bdp: *mut block_def,
    mut lnum: linenr_T,
    mut is_del: bool,
) {
    let mut incr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let lbr_saved: bool = reset_lbr();
    (*bdp).startspaces = 0 as ::core::ffi::c_int;
    (*bdp).endspaces = 0 as ::core::ffi::c_int;
    (*bdp).textlen = 0 as ::core::ffi::c_int;
    (*bdp).start_vcol = 0 as ::core::ffi::c_int as colnr_T;
    (*bdp).end_vcol = 0 as ::core::ffi::c_int as colnr_T;
    (*bdp).is_short = false_0;
    (*bdp).is_oneChar = false_0;
    (*bdp).pre_whitesp = 0 as ::core::ffi::c_int;
    (*bdp).pre_whitesp_c = 0 as ::core::ffi::c_int;
    (*bdp).end_char_vcols = 0 as ::core::ffi::c_int as colnr_T;
    (*bdp).start_char_vcols = 0 as ::core::ffi::c_int as colnr_T;
    let mut line: *mut ::core::ffi::c_char = ml_get(lnum);
    let mut prev_pstart: *mut ::core::ffi::c_char = line;
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
            s: [C2Rust_Unnamed_15 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }; 1],
    };
    let mut cstype: CSType = init_charsize_arg(&raw mut csarg, curwin.get(), lnum, line);
    let mut ci: StrCharInfo = utf_ptr2StrCharInfo(line);
    let mut vcol: ::core::ffi::c_int = (*bdp).start_vcol as ::core::ffi::c_int;
    while vcol < (*oap).start_vcol && *ci.ptr as ::core::ffi::c_int != NUL {
        incr = win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &raw mut csarg).width;
        vcol += incr;
        if ascii_iswhite(ci.chr.value as ::core::ffi::c_int) {
            (*bdp).pre_whitesp += incr;
            (*bdp).pre_whitesp_c += 1;
        } else {
            (*bdp).pre_whitesp = 0 as ::core::ffi::c_int;
            (*bdp).pre_whitesp_c = 0 as ::core::ffi::c_int;
        }
        prev_pstart = ci.ptr;
        ci = utfc_next(ci);
    }
    (*bdp).start_vcol = vcol as colnr_T;
    let mut pstart: *mut ::core::ffi::c_char = ci.ptr;
    (*bdp).start_char_vcols = incr as colnr_T;
    if (*bdp).start_vcol < (*oap).start_vcol {
        (*bdp).end_vcol = (*bdp).start_vcol;
        (*bdp).is_short = true_0;
        if !is_del || (*oap).op_type == OP_APPEND as ::core::ffi::c_int {
            (*bdp).endspaces = (*oap).end_vcol as ::core::ffi::c_int
                - (*oap).start_vcol as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int;
        }
    } else {
        (*bdp).startspaces = ((*bdp).start_vcol - (*oap).start_vcol) as ::core::ffi::c_int;
        if is_del as ::core::ffi::c_int != 0 && (*bdp).startspaces != 0 {
            (*bdp).startspaces = (*bdp).start_char_vcols as ::core::ffi::c_int - (*bdp).startspaces;
        }
        let mut pend: *mut ::core::ffi::c_char = pstart;
        (*bdp).end_vcol = (*bdp).start_vcol;
        if (*bdp).end_vcol > (*oap).end_vcol {
            (*bdp).is_oneChar = true_0;
            if (*oap).op_type == OP_INSERT as ::core::ffi::c_int {
                (*bdp).endspaces =
                    (*bdp).start_char_vcols as ::core::ffi::c_int - (*bdp).startspaces;
            } else if (*oap).op_type == OP_APPEND as ::core::ffi::c_int {
                (*bdp).startspaces += (*oap).end_vcol as ::core::ffi::c_int
                    - (*oap).start_vcol as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int;
                (*bdp).endspaces =
                    (*bdp).start_char_vcols as ::core::ffi::c_int - (*bdp).startspaces;
            } else {
                (*bdp).startspaces = (*oap).end_vcol as ::core::ffi::c_int
                    - (*oap).start_vcol as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int;
                if is_del as ::core::ffi::c_int != 0
                    && (*oap).op_type != OP_LSHIFT as ::core::ffi::c_int
                {
                    (*bdp).startspaces = ((*bdp).start_char_vcols
                        - ((*bdp).start_vcol - (*oap).start_vcol))
                        as ::core::ffi::c_int;
                    (*bdp).endspaces = (*bdp).end_vcol as ::core::ffi::c_int
                        - (*oap).end_vcol as ::core::ffi::c_int
                        - 1 as ::core::ffi::c_int;
                }
            }
        } else {
            cstype = init_charsize_arg(&raw mut csarg, curwin.get(), lnum, line);
            ci = utf_ptr2StrCharInfo(pend);
            vcol = (*bdp).end_vcol as ::core::ffi::c_int;
            let mut prev_pend: *mut ::core::ffi::c_char = pend;
            while vcol <= (*oap).end_vcol && *ci.ptr as ::core::ffi::c_int != NUL {
                prev_pend = ci.ptr;
                incr = win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &raw mut csarg).width;
                vcol += incr;
                ci = utfc_next(ci);
            }
            (*bdp).end_vcol = vcol as colnr_T;
            pend = ci.ptr;
            if (*bdp).end_vcol <= (*oap).end_vcol
                && (!is_del
                    || (*oap).op_type == OP_APPEND as ::core::ffi::c_int
                    || (*oap).op_type == OP_REPLACE as ::core::ffi::c_int)
            {
                (*bdp).is_short = true_0;
                if (*oap).op_type == OP_APPEND as ::core::ffi::c_int
                    || virtual_op.get() as ::core::ffi::c_int != 0
                {
                    (*bdp).endspaces = (*oap).end_vcol as ::core::ffi::c_int
                        - (*bdp).end_vcol as ::core::ffi::c_int
                        + (*oap).inclusive as ::core::ffi::c_int;
                }
            } else if (*bdp).end_vcol > (*oap).end_vcol {
                (*bdp).endspaces = (*bdp).end_vcol as ::core::ffi::c_int
                    - (*oap).end_vcol as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int;
                if !is_del && (*bdp).endspaces != 0 {
                    (*bdp).endspaces = incr - (*bdp).endspaces;
                    if pend != pstart {
                        pend = prev_pend;
                    }
                }
            }
        }
        (*bdp).end_char_vcols = incr as colnr_T;
        if is_del as ::core::ffi::c_int != 0 && (*bdp).startspaces != 0 {
            pstart = prev_pstart;
        }
        (*bdp).textlen = pend.offset_from(pstart) as ::core::ffi::c_int;
    }
    (*bdp).textcol = pstart.offset_from(line) as colnr_T;
    (*bdp).textstart = pstart;
    restore_lbr(lbr_saved);
}
pub unsafe extern "C" fn charwise_block_prep(
    mut start: pos_T,
    mut end: pos_T,
    mut bdp: *mut block_def,
    mut lnum: linenr_T,
    mut inclusive: bool,
) {
    let mut startcol: colnr_T = 0 as colnr_T;
    let mut endcol: colnr_T = MAXCOL as ::core::ffi::c_int;
    let mut cs: colnr_T = 0;
    let mut ce: colnr_T = 0;
    let mut p: *mut ::core::ffi::c_char = ml_get(lnum);
    let mut plen: ::core::ffi::c_int = ml_get_len(lnum);
    (*bdp).startspaces = 0 as ::core::ffi::c_int;
    (*bdp).endspaces = 0 as ::core::ffi::c_int;
    (*bdp).is_oneChar = false_0;
    (*bdp).start_char_vcols = 0 as ::core::ffi::c_int as colnr_T;
    if lnum == start.lnum {
        startcol = start.col;
        if virtual_op.get() as u64 != 0 {
            getvcol(
                curwin.get(),
                &raw mut start,
                &raw mut cs,
                ::core::ptr::null_mut::<colnr_T>(),
                &raw mut ce,
            );
            if ce != cs && start.coladd > 0 as ::core::ffi::c_int {
                (*bdp).start_char_vcols = (ce as ::core::ffi::c_int - cs as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int) as colnr_T;
                (*bdp).startspaces =
                    if (*bdp).start_char_vcols - start.coladd > 0 as ::core::ffi::c_int {
                        (*bdp).start_char_vcols as ::core::ffi::c_int
                            - start.coladd as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    };
                startcol += 1;
            }
        }
    }
    if lnum == end.lnum {
        endcol = end.col;
        if virtual_op.get() as u64 != 0 {
            getvcol(
                curwin.get(),
                &raw mut end,
                &raw mut cs,
                ::core::ptr::null_mut::<colnr_T>(),
                &raw mut ce,
            );
            if *p.offset(endcol as isize) as ::core::ffi::c_int == NUL
                || cs + end.coladd < ce
                    && utf_head_off(p, p.offset(endcol as isize)) == 0 as ::core::ffi::c_int
            {
                if start.lnum == end.lnum && start.col == end.col {
                    (*bdp).is_oneChar = true_0;
                    (*bdp).startspaces = end.coladd as ::core::ffi::c_int
                        - start.coladd as ::core::ffi::c_int
                        + inclusive as ::core::ffi::c_int;
                    endcol = startcol;
                } else {
                    (*bdp).endspaces =
                        end.coladd as ::core::ffi::c_int + inclusive as ::core::ffi::c_int;
                    endcol -= inclusive as ::core::ffi::c_int;
                }
            }
        }
    }
    if endcol == MAXCOL as ::core::ffi::c_int {
        endcol = ml_get_len(lnum);
    }
    if startcol > endcol || (*bdp).is_oneChar != 0 {
        (*bdp).textlen = 0 as ::core::ffi::c_int;
    } else {
        (*bdp).textlen = endcol as ::core::ffi::c_int - startcol as ::core::ffi::c_int
            + inclusive as ::core::ffi::c_int;
    }
    (*bdp).textcol = startcol;
    (*bdp).textstart = if startcol <= plen {
        p.offset(startcol as isize)
    } else {
        p
    };
}
pub unsafe extern "C" fn op_addsub(mut oap: *mut oparg_T, mut Prenum1: linenr_T, mut g_cmd: bool) {
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
    let mut change_cnt: ssize_t = 0 as ssize_t;
    let mut amount: linenr_T = Prenum1;
    (*disable_fold_update.ptr()) += 1;
    if !VIsual_active.get() {
        let mut pos: pos_T = (*curwin.get()).w_cursor;
        if u_save_cursor() == FAIL {
            (*disable_fold_update.ptr()) -= 1;
            return;
        }
        change_cnt = do_addsub(
            (*oap).op_type,
            &raw mut pos,
            0 as ::core::ffi::c_int,
            amount,
        ) as ssize_t;
        (*disable_fold_update.ptr()) -= 1;
        if change_cnt != 0 {
            changed_lines(
                curbuf.get(),
                pos.lnum,
                0 as colnr_T,
                pos.lnum + 1 as linenr_T,
                0 as linenr_T,
                true_0 != 0,
            );
        }
    } else {
        let mut length: ::core::ffi::c_int = 0;
        let mut startpos: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        if u_save(
            (*oap).start.lnum - 1 as linenr_T,
            (*oap).end.lnum + 1 as linenr_T,
        ) == FAIL
        {
            (*disable_fold_update.ptr()) -= 1;
            return;
        }
        let mut pos_0: pos_T = (*oap).start;
        while pos_0.lnum <= (*oap).end.lnum {
            if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
                block_prep(oap, &raw mut bd, pos_0.lnum, false_0 != 0);
                pos_0.col = bd.textcol;
                length = bd.textlen;
            } else if (*oap).motion_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
            {
                (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                pos_0.col = 0 as ::core::ffi::c_int as colnr_T;
                length = ml_get_len(pos_0.lnum) as ::core::ffi::c_int;
            } else {
                if pos_0.lnum == (*oap).start.lnum && !(*oap).inclusive {
                    dec(&raw mut (*oap).end);
                }
                length = ml_get_len(pos_0.lnum) as ::core::ffi::c_int;
                pos_0.col = 0 as ::core::ffi::c_int as colnr_T;
                if pos_0.lnum == (*oap).start.lnum {
                    pos_0.col += (*oap).start.col;
                    length -= (*oap).start.col as ::core::ffi::c_int;
                }
                if pos_0.lnum == (*oap).end.lnum {
                    length = ml_get_len((*oap).end.lnum) as ::core::ffi::c_int;
                    (*oap).end.col = (if (*oap).end.col < length - 1 as ::core::ffi::c_int {
                        (*oap).end.col as ::core::ffi::c_int
                    } else {
                        length - 1 as ::core::ffi::c_int
                    }) as colnr_T;
                    length = (*oap).end.col as ::core::ffi::c_int - pos_0.col as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int;
                }
            }
            let mut one_change: bool = do_addsub((*oap).op_type, &raw mut pos_0, length, amount);
            if one_change {
                if change_cnt == 0 as ssize_t {
                    startpos = (*curbuf.get()).b_op_start;
                }
                change_cnt += 1;
            }
            if g_cmd as ::core::ffi::c_int != 0 && one_change as ::core::ffi::c_int != 0 {
                amount += Prenum1;
            }
            pos_0.lnum += 1;
        }
        (*disable_fold_update.ptr()) -= 1;
        if change_cnt != 0 {
            changed_lines(
                curbuf.get(),
                (*oap).start.lnum,
                0 as colnr_T,
                (*oap).end.lnum + 1 as linenr_T,
                0 as linenr_T,
                true_0 != 0,
            );
        }
        if change_cnt == 0 && (*oap).is_VIsual as ::core::ffi::c_int != 0 {
            redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
        }
        if change_cnt > 0 as ssize_t
            && (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            (*curbuf.get()).b_op_start = startpos;
        }
        if change_cnt > p_report.get() as ssize_t {
            smsg(
                0 as ::core::ffi::c_int,
                ngettext(
                    b"%ld lines changed\0".as_ptr() as *const ::core::ffi::c_char,
                    b"%ld lines changed\0".as_ptr() as *const ::core::ffi::c_char,
                    change_cnt as ::core::ffi::c_ulong,
                ),
                change_cnt as int64_t,
            );
        }
    };
}
pub unsafe extern "C" fn do_addsub(
    mut op_type: ::core::ffi::c_int,
    mut pos: *mut pos_T,
    mut length: ::core::ffi::c_int,
    mut Prenum1: linenr_T,
) -> bool {
    let mut firstdigit: ::core::ffi::c_int = 0;
    let mut pre: ::core::ffi::c_int = 0;
    static hexupper: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let mut n: uvarnumber_T = 0;
    let mut blank_unsigned: bool = false_0 != 0;
    let mut negative: bool = false_0 != 0;
    let mut was_positive: bool = true_0 != 0;
    let mut visual: bool = VIsual_active.get();
    let mut did_change: bool = false_0 != 0;
    let mut save_cursor: pos_T = (*curwin.get()).w_cursor;
    let mut maxlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut startpos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut endpos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut save_coladd: colnr_T = 0 as colnr_T;
    let do_hex: bool = !vim_strchr((*curbuf.get()).b_p_nf, 'x' as ::core::ffi::c_int).is_null();
    let do_oct: bool = !vim_strchr((*curbuf.get()).b_p_nf, 'o' as ::core::ffi::c_int).is_null();
    let do_bin: bool = !vim_strchr((*curbuf.get()).b_p_nf, 'b' as ::core::ffi::c_int).is_null();
    let do_alpha: bool = !vim_strchr((*curbuf.get()).b_p_nf, 'p' as ::core::ffi::c_int).is_null();
    let do_unsigned: bool =
        !vim_strchr((*curbuf.get()).b_p_nf, 'u' as ::core::ffi::c_int).is_null();
    let do_blank: bool = !vim_strchr((*curbuf.get()).b_p_nf, 'k' as ::core::ffi::c_int).is_null();
    if virtual_active(curwin.get()) {
        save_coladd = (*pos).coladd;
        (*pos).coladd = 0 as ::core::ffi::c_int as colnr_T;
    }
    (*curwin.get()).w_cursor = *pos;
    let mut ptr: *mut ::core::ffi::c_char = ml_get((*pos).lnum);
    let mut linelen: ::core::ffi::c_int = ml_get_len((*pos).lnum);
    let mut col: ::core::ffi::c_int = (*pos).col as ::core::ffi::c_int;
    '_theend: {
        if (col + (save_coladd != 0) as ::core::ffi::c_int) < linelen {
            if !VIsual_active.get() {
                if do_bin {
                    while col > 0 as ::core::ffi::c_int
                        && ascii_isbdigit(*ptr.offset(col as isize) as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                            != 0
                    {
                        col -= 1;
                        col -= utf_head_off(ptr, ptr.offset(col as isize));
                    }
                }
                if do_hex {
                    while col > 0 as ::core::ffi::c_int
                        && ascii_isxdigit(*ptr.offset(col as isize) as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                            != 0
                    {
                        col -= 1;
                        col -= utf_head_off(ptr, ptr.offset(col as isize));
                    }
                }
                if do_bin as ::core::ffi::c_int != 0
                    && do_hex as ::core::ffi::c_int != 0
                    && !(col > 0 as ::core::ffi::c_int
                        && (*ptr.offset(col as isize) as ::core::ffi::c_int
                            == 'X' as ::core::ffi::c_int
                            || *ptr.offset(col as isize) as ::core::ffi::c_int
                                == 'x' as ::core::ffi::c_int)
                        && *ptr.offset((col - 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            == '0' as ::core::ffi::c_int
                        && utf_head_off(
                            ptr,
                            ptr.offset(col as isize)
                                .offset(-(1 as ::core::ffi::c_int as isize)),
                        ) == 0
                        && ascii_isxdigit(*ptr.offset((col + 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int) as ::core::ffi::c_int
                            != 0)
                {
                    col = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
                    while col > 0 as ::core::ffi::c_int
                        && ascii_isdigit(*ptr.offset(col as isize) as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                            != 0
                    {
                        col -= 1;
                        col -= utf_head_off(ptr, ptr.offset(col as isize));
                    }
                }
                if do_hex as ::core::ffi::c_int != 0
                    && col > 0 as ::core::ffi::c_int
                    && (*ptr.offset(col as isize) as ::core::ffi::c_int
                        == 'X' as ::core::ffi::c_int
                        || *ptr.offset(col as isize) as ::core::ffi::c_int
                            == 'x' as ::core::ffi::c_int)
                    && *ptr.offset((col - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                        == '0' as ::core::ffi::c_int
                    && utf_head_off(
                        ptr,
                        ptr.offset(col as isize)
                            .offset(-(1 as ::core::ffi::c_int as isize)),
                    ) == 0
                    && ascii_isxdigit(
                        *ptr.offset((col + 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0
                    || do_bin as ::core::ffi::c_int != 0
                        && col > 0 as ::core::ffi::c_int
                        && (*ptr.offset(col as isize) as ::core::ffi::c_int
                            == 'B' as ::core::ffi::c_int
                            || *ptr.offset(col as isize) as ::core::ffi::c_int
                                == 'b' as ::core::ffi::c_int)
                        && *ptr.offset((col - 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            == '0' as ::core::ffi::c_int
                        && utf_head_off(
                            ptr,
                            ptr.offset(col as isize)
                                .offset(-(1 as ::core::ffi::c_int as isize)),
                        ) == 0
                        && ascii_isbdigit(*ptr.offset((col + 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int) as ::core::ffi::c_int
                            != 0
                {
                    col -= 1;
                    col -= utf_head_off(ptr, ptr.offset(col as isize));
                } else {
                    col = (*pos).col as ::core::ffi::c_int;
                    while *ptr.offset(col as isize) as ::core::ffi::c_int != NUL
                        && !ascii_isdigit(*ptr.offset(col as isize) as ::core::ffi::c_int)
                        && !(do_alpha as ::core::ffi::c_int != 0
                            && (*ptr.offset(col as isize) as ::core::ffi::c_uint
                                >= 'A' as ::core::ffi::c_uint
                                && *ptr.offset(col as isize) as ::core::ffi::c_uint
                                    <= 'Z' as ::core::ffi::c_uint
                                || *ptr.offset(col as isize) as ::core::ffi::c_uint
                                    >= 'a' as ::core::ffi::c_uint
                                    && *ptr.offset(col as isize) as ::core::ffi::c_uint
                                        <= 'z' as ::core::ffi::c_uint))
                    {
                        col += 1;
                    }
                    while col > 0 as ::core::ffi::c_int
                        && ascii_isdigit(*ptr.offset((col - 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int) as ::core::ffi::c_int
                            != 0
                        && !(do_alpha as ::core::ffi::c_int != 0
                            && (*ptr.offset(col as isize) as ::core::ffi::c_uint
                                >= 'A' as ::core::ffi::c_uint
                                && *ptr.offset(col as isize) as ::core::ffi::c_uint
                                    <= 'Z' as ::core::ffi::c_uint
                                || *ptr.offset(col as isize) as ::core::ffi::c_uint
                                    >= 'a' as ::core::ffi::c_uint
                                    && *ptr.offset(col as isize) as ::core::ffi::c_uint
                                        <= 'z' as ::core::ffi::c_uint))
                    {
                        col -= 1;
                    }
                }
            }
            if visual {
                while *ptr.offset(col as isize) as ::core::ffi::c_int != NUL
                    && length > 0 as ::core::ffi::c_int
                    && !ascii_isdigit(*ptr.offset(col as isize) as ::core::ffi::c_int)
                    && !(do_alpha as ::core::ffi::c_int != 0
                        && (*ptr.offset(col as isize) as ::core::ffi::c_uint
                            >= 'A' as ::core::ffi::c_uint
                            && *ptr.offset(col as isize) as ::core::ffi::c_uint
                                <= 'Z' as ::core::ffi::c_uint
                            || *ptr.offset(col as isize) as ::core::ffi::c_uint
                                >= 'a' as ::core::ffi::c_uint
                                && *ptr.offset(col as isize) as ::core::ffi::c_uint
                                    <= 'z' as ::core::ffi::c_uint))
                {
                    let mut mb_len: ::core::ffi::c_int = utfc_ptr2len(ptr.offset(col as isize));
                    col += mb_len;
                    length -= mb_len;
                }
                if length == 0 as ::core::ffi::c_int {
                    break '_theend;
                } else if col > (*pos).col
                    && *ptr.offset((col - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                        == '-' as ::core::ffi::c_int
                    && utf_head_off(
                        ptr,
                        ptr.offset(col as isize)
                            .offset(-(1 as ::core::ffi::c_int as isize)),
                    ) == 0
                    && !do_unsigned
                {
                    if do_blank as ::core::ffi::c_int != 0
                        && col >= 2 as ::core::ffi::c_int
                        && !ascii_iswhite(*ptr.offset((col - 2 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int)
                    {
                        blank_unsigned = true_0 != 0;
                    } else {
                        negative = true_0 != 0;
                        was_positive = false_0 != 0;
                    }
                }
            }
            firstdigit = *ptr.offset(col as isize) as uint8_t as ::core::ffi::c_int;
            if !ascii_isdigit(firstdigit)
                && !(do_alpha as ::core::ffi::c_int != 0
                    && (firstdigit as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                        && firstdigit as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                        || firstdigit as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                            && firstdigit as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint))
            {
                beep_flush();
            } else {
                if do_alpha as ::core::ffi::c_int != 0
                    && (firstdigit as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                        && firstdigit as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                        || firstdigit as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                            && firstdigit as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint)
                {
                    if op_type == OP_NR_SUB as ::core::ffi::c_int {
                        if (if (firstdigit as uint8_t as ::core::ffi::c_int)
                            < 'a' as ::core::ffi::c_int
                        {
                            firstdigit as uint8_t as linenr_T - 'A' as linenr_T
                        } else {
                            firstdigit as uint8_t as linenr_T - 'a' as linenr_T
                        }) < Prenum1
                        {
                            firstdigit = if *(*__ctype_b_loc()).offset(firstdigit as isize)
                                as ::core::ffi::c_int
                                & _ISupper as ::core::ffi::c_int as ::core::ffi::c_ushort
                                    as ::core::ffi::c_int
                                != 0
                            {
                                'A' as ::core::ffi::c_int
                            } else {
                                'a' as ::core::ffi::c_int
                            };
                        } else {
                            firstdigit -= Prenum1 as ::core::ffi::c_int;
                        }
                    } else if (26 as linenr_T
                        - (if (firstdigit as uint8_t as ::core::ffi::c_int)
                            < 'a' as ::core::ffi::c_int
                        {
                            firstdigit as uint8_t as linenr_T - 'A' as linenr_T
                        } else {
                            firstdigit as uint8_t as linenr_T - 'a' as linenr_T
                        })
                        - 1 as linenr_T)
                        < Prenum1
                    {
                        firstdigit = if *(*__ctype_b_loc()).offset(firstdigit as isize)
                            as ::core::ffi::c_int
                            & _ISupper as ::core::ffi::c_int as ::core::ffi::c_ushort
                                as ::core::ffi::c_int
                            != 0
                        {
                            'Z' as ::core::ffi::c_int
                        } else {
                            'z' as ::core::ffi::c_int
                        };
                    } else {
                        firstdigit += Prenum1 as ::core::ffi::c_int;
                    }
                    (*curwin.get()).w_cursor.col = col as colnr_T;
                    startpos = (*curwin.get()).w_cursor;
                    did_change = true_0 != 0;
                    del_char(false_0 != 0);
                    ins_char(firstdigit);
                    endpos = (*curwin.get()).w_cursor;
                    (*curwin.get()).w_cursor.col = col as colnr_T;
                } else {
                    if col > 0 as ::core::ffi::c_int
                        && *ptr.offset((col - 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            == '-' as ::core::ffi::c_int
                        && utf_head_off(
                            ptr,
                            ptr.offset(col as isize)
                                .offset(-(1 as ::core::ffi::c_int as isize)),
                        ) == 0
                        && !visual
                        && !do_unsigned
                    {
                        if do_blank as ::core::ffi::c_int != 0
                            && col >= 2 as ::core::ffi::c_int
                            && !ascii_iswhite(*ptr.offset((col - 2 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int)
                        {
                            blank_unsigned = true_0 != 0;
                        } else {
                            col -= 1;
                            negative = true_0 != 0;
                        }
                    }
                    if visual as ::core::ffi::c_int != 0
                        && VIsual_mode.get() != 'V' as ::core::ffi::c_int
                    {
                        maxlen = if (*curbuf.get()).b_visual.vi_curswant
                            == MAXCOL as ::core::ffi::c_int
                        {
                            linelen - col
                        } else {
                            length
                        };
                    }
                    let mut overflow: bool = false_0 != 0;
                    vim_str2nr(
                        ptr.offset(col as isize),
                        &raw mut pre,
                        &raw mut length,
                        0 as ::core::ffi::c_int
                            + (if do_bin as ::core::ffi::c_int != 0 {
                                STR2NR_BIN as ::core::ffi::c_int
                            } else {
                                0 as ::core::ffi::c_int
                            })
                            + (if do_oct as ::core::ffi::c_int != 0 {
                                STR2NR_OCT as ::core::ffi::c_int
                            } else {
                                0 as ::core::ffi::c_int
                            })
                            + (if do_hex as ::core::ffi::c_int != 0 {
                                STR2NR_HEX as ::core::ffi::c_int
                            } else {
                                0 as ::core::ffi::c_int
                            }),
                        ::core::ptr::null_mut::<varnumber_T>(),
                        &raw mut n,
                        maxlen,
                        false_0 != 0,
                        &raw mut overflow,
                    );
                    if pre != 0 && negative as ::core::ffi::c_int != 0 {
                        col += 1;
                        length -= 1;
                        negative = false_0 != 0;
                    }
                    let mut subtract: bool = false_0 != 0;
                    if op_type == OP_NR_SUB as ::core::ffi::c_int {
                        subtract = subtract as ::core::ffi::c_int ^ true_0 != 0;
                    }
                    if negative {
                        subtract = subtract as ::core::ffi::c_int ^ true_0 != 0;
                    }
                    let mut oldn: uvarnumber_T = n;
                    if !overflow {
                        n = if subtract as ::core::ffi::c_int != 0 {
                            n.wrapping_sub(Prenum1 as uvarnumber_T)
                        } else {
                            n.wrapping_add(Prenum1 as uvarnumber_T)
                        };
                    }
                    if pre == 0 {
                        if subtract {
                            if n > oldn {
                                n = (1 as uvarnumber_T)
                                    .wrapping_add(n ^ -1 as ::core::ffi::c_int as uvarnumber_T);
                                negative = negative as ::core::ffi::c_int ^ true_0 != 0;
                            }
                        } else if n < oldn {
                            n = n ^ -1 as ::core::ffi::c_int as uvarnumber_T;
                            negative = negative as ::core::ffi::c_int ^ true_0 != 0;
                        }
                        if n == 0 as uvarnumber_T {
                            negative = false_0 != 0;
                        }
                    }
                    if (do_unsigned as ::core::ffi::c_int != 0
                        || blank_unsigned as ::core::ffi::c_int != 0)
                        && negative as ::core::ffi::c_int != 0
                    {
                        if subtract {
                            n = 0 as uvarnumber_T;
                        } else {
                            n = -1 as ::core::ffi::c_int as uvarnumber_T;
                        }
                        negative = false_0 != 0;
                    }
                    if visual as ::core::ffi::c_int != 0
                        && !was_positive
                        && !negative
                        && col > 0 as ::core::ffi::c_int
                    {
                        col -= 1;
                        length += 1;
                    }
                    (*curwin.get()).w_cursor.col = col as colnr_T;
                    startpos = (*curwin.get()).w_cursor;
                    did_change = true_0 != 0;
                    let mut todel: ::core::ffi::c_int = length;
                    let mut c: ::core::ffi::c_int = gchar_cursor();
                    if c == '-' as ::core::ffi::c_int {
                        length -= 1;
                    }
                    loop {
                        let c2rust_fresh0 = todel;
                        todel = todel - 1;
                        if c2rust_fresh0 <= 0 as ::core::ffi::c_int {
                            break;
                        }
                        if c < 0x100 as ::core::ffi::c_int
                            && *(*__ctype_b_loc()).offset(c as isize) as ::core::ffi::c_int
                                & _ISalpha as ::core::ffi::c_int as ::core::ffi::c_ushort
                                    as ::core::ffi::c_int
                                != 0
                        {
                            hexupper.set(
                                *(*__ctype_b_loc()).offset(c as isize) as ::core::ffi::c_int
                                    & _ISupper as ::core::ffi::c_int as ::core::ffi::c_ushort
                                        as ::core::ffi::c_int
                                    != 0,
                            );
                        }
                        del_char(false_0 != 0);
                        c = gchar_cursor();
                    }
                    let mut buf1: *mut ::core::ffi::c_char = xmalloc(
                        (length as size_t).wrapping_add(NUMBUFLEN as ::core::ffi::c_int as size_t),
                    )
                        as *mut ::core::ffi::c_char;
                    ptr = buf1;
                    if negative as ::core::ffi::c_int != 0
                        && (!visual || was_positive as ::core::ffi::c_int != 0)
                    {
                        let c2rust_fresh1 = ptr;
                        ptr = ptr.offset(1);
                        *c2rust_fresh1 = '-' as ::core::ffi::c_char;
                    }
                    if pre != 0 {
                        let c2rust_fresh2 = ptr;
                        ptr = ptr.offset(1);
                        *c2rust_fresh2 = '0' as ::core::ffi::c_char;
                        length -= 1;
                    }
                    if pre == 'b' as ::core::ffi::c_int
                        || pre == 'B' as ::core::ffi::c_int
                        || pre == 'x' as ::core::ffi::c_int
                        || pre == 'X' as ::core::ffi::c_int
                    {
                        let c2rust_fresh3 = ptr;
                        ptr = ptr.offset(1);
                        *c2rust_fresh3 = pre as ::core::ffi::c_char;
                        length -= 1;
                    }
                    let mut buf2: [::core::ffi::c_char; 65] = [0; 65];
                    let mut buf2len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    if pre == 'b' as ::core::ffi::c_int || pre == 'B' as ::core::ffi::c_int {
                        let mut bits: size_t = 0 as size_t;
                        bits = (8 as usize).wrapping_mul(::core::mem::size_of::<uvarnumber_T>())
                            as size_t;
                        while bits > 0 as size_t {
                            if n >> bits.wrapping_sub(1 as size_t) & 0x1 as uvarnumber_T != 0 {
                                break;
                            }
                            bits = bits.wrapping_sub(1);
                        }
                        while bits > 0 as size_t
                            && buf2len < NUMBUFLEN as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                        {
                            bits = bits.wrapping_sub(1);
                            let c2rust_fresh4 = buf2len;
                            buf2len = buf2len + 1;
                            buf2[c2rust_fresh4 as usize] =
                                (if n >> bits & 0x1 as uvarnumber_T != 0 {
                                    '1' as ::core::ffi::c_int
                                } else {
                                    '0' as ::core::ffi::c_int
                                }) as ::core::ffi::c_char;
                        }
                        buf2[buf2len as usize] = NUL as ::core::ffi::c_char;
                    } else if pre == 0 as ::core::ffi::c_int {
                        buf2len = vim_snprintf(
                            &raw mut buf2 as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                ),
                            b"%lu\0".as_ptr() as *const ::core::ffi::c_char,
                            n,
                        );
                    } else if pre == '0' as ::core::ffi::c_int {
                        buf2len = vim_snprintf(
                            &raw mut buf2 as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                ),
                            b"%lo\0".as_ptr() as *const ::core::ffi::c_char,
                            n,
                        );
                    } else if hexupper.get() {
                        buf2len = vim_snprintf(
                            &raw mut buf2 as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                ),
                            b"%lX\0".as_ptr() as *const ::core::ffi::c_char,
                            n,
                        );
                    } else {
                        buf2len = vim_snprintf(
                            &raw mut buf2 as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                ),
                            b"%lx\0".as_ptr() as *const ::core::ffi::c_char,
                            n,
                        );
                    }
                    length -= buf2len;
                    if firstdigit == '0' as ::core::ffi::c_int
                        && !(do_oct as ::core::ffi::c_int != 0 && pre == 0 as ::core::ffi::c_int)
                    {
                        loop {
                            let c2rust_fresh5 = length;
                            length = length - 1;
                            if c2rust_fresh5 <= 0 as ::core::ffi::c_int {
                                break;
                            }
                            let c2rust_fresh6 = ptr;
                            ptr = ptr.offset(1);
                            *c2rust_fresh6 = '0' as ::core::ffi::c_char;
                        }
                    }
                    *ptr = NUL as ::core::ffi::c_char;
                    let mut buf1len: ::core::ffi::c_int =
                        ptr.offset_from(buf1) as ::core::ffi::c_int;
                    strcpy(
                        buf1.offset(buf1len as isize),
                        &raw mut buf2 as *mut ::core::ffi::c_char,
                    );
                    buf1len += buf2len;
                    ins_str(buf1, buf1len as size_t);
                    xfree(buf1 as *mut ::core::ffi::c_void);
                    endpos = (*curwin.get()).w_cursor;
                    if (*curwin.get()).w_cursor.col != 0 {
                        (*curwin.get()).w_cursor.col -= 1;
                    }
                }
                if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                {
                    (*curbuf.get()).b_op_start = startpos;
                    (*curbuf.get()).b_op_end = endpos;
                    if (*curbuf.get()).b_op_end.col > 0 as ::core::ffi::c_int {
                        (*curbuf.get()).b_op_end.col -= 1;
                    }
                }
            }
        }
    }
    if visual {
        (*curwin.get()).w_cursor = save_cursor;
    } else if did_change {
        (*curwin.get()).w_set_curswant = true_0;
    } else if virtual_active(curwin.get()) {
        (*curwin.get()).w_cursor.coladd = save_coladd;
    }
    return did_change;
}
pub unsafe extern "C" fn clear_oparg(mut oap: *mut oparg_T) {
    memset(
        oap as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<oparg_T>(),
    );
}
unsafe extern "C" fn line_count_info(
    mut line: *mut ::core::ffi::c_char,
    mut wc: *mut varnumber_T,
    mut cc: *mut varnumber_T,
    mut limit: varnumber_T,
    mut eol_size: ::core::ffi::c_int,
) -> varnumber_T {
    let mut i: varnumber_T = 0;
    let mut words: varnumber_T = 0 as varnumber_T;
    let mut chars: varnumber_T = 0 as varnumber_T;
    let mut is_word: bool = false_0 != 0;
    i = 0 as varnumber_T;
    while i < limit && *line.offset(i as isize) as ::core::ffi::c_int != NUL {
        if is_word {
            if ascii_isspace(*line.offset(i as isize) as ::core::ffi::c_int) {
                words += 1;
                is_word = false_0 != 0;
            }
        } else if !ascii_isspace(*line.offset(i as isize) as ::core::ffi::c_int) {
            is_word = true_0 != 0;
        }
        chars += 1;
        i += utfc_ptr2len(line.offset(i as isize)) as varnumber_T;
    }
    if is_word {
        words += 1;
    }
    *wc += words;
    if i < limit && *line.offset(i as isize) as ::core::ffi::c_int == NUL {
        i += eol_size as varnumber_T;
        chars += eol_size as varnumber_T;
    }
    *cc += chars;
    return i;
}
pub unsafe extern "C" fn cursor_pos_info(mut dict: *mut dict_T) {
    let mut buf1: [::core::ffi::c_char; 50] = [0; 50];
    let mut buf2: [::core::ffi::c_char; 40] = [0; 40];
    let mut byte_count: varnumber_T = 0 as varnumber_T;
    let mut bom_count: varnumber_T = 0 as varnumber_T;
    let mut byte_count_cursor: varnumber_T = 0 as varnumber_T;
    let mut char_count: varnumber_T = 0 as varnumber_T;
    let mut char_count_cursor: varnumber_T = 0 as varnumber_T;
    let mut word_count: varnumber_T = 0 as varnumber_T;
    let mut word_count_cursor: varnumber_T = 0 as varnumber_T;
    let mut min_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut max_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut oparg: oparg_T = oparg_T {
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
    let l_VIsual_active: ::core::ffi::c_int = VIsual_active.get() as ::core::ffi::c_int;
    let l_VIsual_mode: ::core::ffi::c_int = VIsual_mode.get();
    if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
        if dict.is_null() {
            msg(
                gettext(no_lines_msg.ptr() as *mut ::core::ffi::c_char),
                0 as ::core::ffi::c_int,
            );
            return;
        }
    } else {
        let mut eol_size: ::core::ffi::c_int = 0;
        let mut last_check: varnumber_T = 100000 as varnumber_T;
        let mut line_count_selected: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if get_fileformat(curbuf.get()) == EOL_DOS {
            eol_size = 2 as ::core::ffi::c_int;
        } else {
            eol_size = 1 as ::core::ffi::c_int;
        }
        if l_VIsual_active != 0 {
            if lt(VIsual.get(), (*curwin.get()).w_cursor) {
                min_pos = VIsual.get();
                max_pos = (*curwin.get()).w_cursor;
            } else {
                min_pos = (*curwin.get()).w_cursor;
                max_pos = VIsual.get();
            }
            if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
                && max_pos.col > 0 as ::core::ffi::c_int
            {
                max_pos.col -= 1;
            }
            if l_VIsual_mode == Ctrl_V {
                let saved_sbr: *mut ::core::ffi::c_char = p_sbr.get();
                let saved_w_sbr: *mut ::core::ffi::c_char = (*curwin.get()).w_onebuf_opt.wo_sbr;
                p_sbr.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
                (*curwin.get()).w_onebuf_opt.wo_sbr =
                    empty_string_option.ptr() as *mut ::core::ffi::c_char;
                oparg.is_VIsual = true_0 != 0;
                oparg.motion_type = kMTBlockWise;
                oparg.op_type = OP_NOP as ::core::ffi::c_int;
                getvcols(
                    curwin.get(),
                    &raw mut min_pos,
                    &raw mut max_pos,
                    &raw mut oparg.start_vcol,
                    &raw mut oparg.end_vcol,
                );
                p_sbr.set(saved_sbr);
                (*curwin.get()).w_onebuf_opt.wo_sbr = saved_w_sbr;
                if (*curwin.get()).w_curswant == MAXCOL as ::core::ffi::c_int {
                    oparg.end_vcol = MAXCOL as ::core::ffi::c_int as colnr_T;
                }
                if oparg.end_vcol < oparg.start_vcol {
                    oparg.end_vcol += oparg.start_vcol;
                    oparg.start_vcol = oparg.end_vcol - oparg.start_vcol;
                    oparg.end_vcol -= oparg.start_vcol;
                }
            }
            line_count_selected =
                (max_pos.lnum - min_pos.lnum + 1 as linenr_T) as ::core::ffi::c_int;
        }
        let mut lnum: linenr_T = 1 as linenr_T;
        while lnum <= (*curbuf.get()).b_ml.ml_line_count {
            if byte_count > last_check {
                os_breakcheck();
                if got_int.get() {
                    return;
                }
                last_check = byte_count + 100000 as varnumber_T;
            }
            if l_VIsual_active != 0 && lnum >= min_pos.lnum && lnum <= max_pos.lnum {
                let mut s: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                match l_VIsual_mode {
                    Ctrl_V => {
                        virtual_op.set(virtual_active(curwin.get()) as TriState);
                        block_prep(&raw mut oparg, &raw mut bd, lnum, false_0 != 0);
                        virtual_op.set(kNone);
                        s = bd.textstart;
                        len = bd.textlen;
                    }
                    86 => {
                        s = ml_get(lnum);
                        len = MAXCOL as ::core::ffi::c_int;
                    }
                    118 => {
                        let mut start_col: colnr_T = if lnum == min_pos.lnum {
                            min_pos.col
                        } else {
                            0 as colnr_T
                        };
                        let mut end_col: colnr_T = if lnum == max_pos.lnum {
                            max_pos.col - start_col + 1 as colnr_T
                        } else {
                            MAXCOL as ::core::ffi::c_int
                        };
                        s = ml_get(lnum).offset(start_col as isize);
                        len = end_col as ::core::ffi::c_int;
                    }
                    _ => {}
                }
                if !s.is_null() {
                    byte_count_cursor += line_count_info(
                        s,
                        &raw mut word_count_cursor,
                        &raw mut char_count_cursor,
                        len as varnumber_T,
                        eol_size,
                    );
                    if lnum == (*curbuf.get()).b_ml.ml_line_count
                        && (*curbuf.get()).b_p_eol == 0
                        && ((*curbuf.get()).b_p_bin != 0 || (*curbuf.get()).b_p_fixeol == 0)
                        && (strlen(s) as ::core::ffi::c_int) < len
                    {
                        byte_count_cursor -= eol_size as varnumber_T;
                    }
                }
            } else if lnum == (*curwin.get()).w_cursor.lnum {
                word_count_cursor += word_count;
                char_count_cursor += char_count;
                byte_count_cursor = byte_count
                    + line_count_info(
                        ml_get(lnum),
                        &raw mut word_count_cursor,
                        &raw mut char_count_cursor,
                        (*curwin.get()).w_cursor.col as varnumber_T + 1 as varnumber_T,
                        eol_size,
                    );
            }
            byte_count += line_count_info(
                ml_get(lnum),
                &raw mut word_count,
                &raw mut char_count,
                MAXCOL as ::core::ffi::c_int as varnumber_T,
                eol_size,
            );
            lnum += 1;
        }
        if (*curbuf.get()).b_p_eol == 0
            && ((*curbuf.get()).b_p_bin != 0 || (*curbuf.get()).b_p_fixeol == 0)
        {
            byte_count -= eol_size as varnumber_T;
        }
        if dict.is_null() {
            if l_VIsual_active != 0 {
                if l_VIsual_mode == Ctrl_V
                    && (*curwin.get()).w_curswant < MAXCOL as ::core::ffi::c_int
                {
                    getvcols(
                        curwin.get(),
                        &raw mut min_pos,
                        &raw mut max_pos,
                        &raw mut min_pos.col,
                        &raw mut max_pos.col,
                    );
                    let mut cols: int64_t = 0;
                    let (c2rust_result, c2rust_overflowed) =
                        ((oparg.end_vcol as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as i128)
                            .overflowing_sub(oparg.start_vcol as i128);
                    let c2rust_result_narrow = c2rust_result as int64_t;
                    *&raw mut cols = c2rust_result_narrow;
                    if c2rust_overflowed || c2rust_result_narrow as i128 != c2rust_result {
                        logmsg(
                            LOGLVL_ERR,
                            ::core::ptr::null::<::core::ffi::c_char>(),
                            b"cursor_pos_info\0".as_ptr() as *const ::core::ffi::c_char,
                            2966 as ::core::ffi::c_int,
                            true_0 != 0,
                            b"STRICT_SUB overflow\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                        abort();
                    }
                    vim_snprintf(
                        &raw mut buf1 as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 50]>(),
                        gettext(b"%ld Cols; \0".as_ptr() as *const ::core::ffi::c_char),
                        cols,
                    );
                } else {
                    buf1[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
                }
                if char_count_cursor == byte_count_cursor && char_count == byte_count {
                    vim_snprintf(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        IOSIZE as size_t,
                        gettext(
                            b"Selected %s%ld of %ld Lines; %ld of %ld Words; %ld of %ld Bytes\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        &raw mut buf1 as *mut ::core::ffi::c_char,
                        line_count_selected as int64_t,
                        (*curbuf.get()).b_ml.ml_line_count as int64_t,
                        word_count_cursor,
                        word_count,
                        byte_count_cursor,
                        byte_count,
                    );
                } else {
                    vim_snprintf(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        IOSIZE as size_t,
                        gettext(
                            b"Selected %s%ld of %ld Lines; %ld of %ld Words; %ld of %ld Chars; %ld of %ld Bytes\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        &raw mut buf1 as *mut ::core::ffi::c_char,
                        line_count_selected as int64_t,
                        (*curbuf.get()).b_ml.ml_line_count as int64_t,
                        word_count_cursor,
                        word_count,
                        char_count_cursor,
                        char_count,
                        byte_count_cursor,
                        byte_count,
                    );
                }
            } else {
                let mut p: *mut ::core::ffi::c_char = get_cursor_line_ptr();
                validate_virtcol(curwin.get());
                col_print(
                    &raw mut buf1 as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 50]>(),
                    (*curwin.get()).w_cursor.col + 1 as ::core::ffi::c_int,
                    (*curwin.get()).w_virtcol + 1 as ::core::ffi::c_int,
                );
                col_print(
                    &raw mut buf2 as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 40]>(),
                    get_cursor_line_len(),
                    linetabsize_str(p),
                );
                if char_count_cursor == byte_count_cursor && char_count == byte_count {
                    vim_snprintf(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        IOSIZE as size_t,
                        gettext(
                            b"Col %s of %s; Line %ld of %ld; Word %ld of %ld; Byte %ld of %ld\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        &raw mut buf1 as *mut ::core::ffi::c_char,
                        &raw mut buf2 as *mut ::core::ffi::c_char,
                        (*curwin.get()).w_cursor.lnum as int64_t,
                        (*curbuf.get()).b_ml.ml_line_count as int64_t,
                        word_count_cursor,
                        word_count,
                        byte_count_cursor,
                        byte_count,
                    );
                } else {
                    vim_snprintf(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        IOSIZE as size_t,
                        gettext(
                            b"Col %s of %s; Line %ld of %ld; Word %ld of %ld; Char %ld of %ld; Byte %ld of %ld\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        &raw mut buf1 as *mut ::core::ffi::c_char,
                        &raw mut buf2 as *mut ::core::ffi::c_char,
                        (*curwin.get()).w_cursor.lnum as int64_t,
                        (*curbuf.get()).b_ml.ml_line_count as int64_t,
                        word_count_cursor,
                        word_count,
                        char_count_cursor,
                        char_count,
                        byte_count_cursor,
                        byte_count,
                    );
                }
            }
        }
        bom_count = bomb_size() as varnumber_T;
        if dict.is_null() && bom_count > 0 as varnumber_T {
            let len_0: size_t = strlen(IObuff.ptr() as *mut ::core::ffi::c_char);
            vim_snprintf(
                (IObuff.ptr() as *mut ::core::ffi::c_char).offset(len_0 as isize),
                (IOSIZE as size_t).wrapping_sub(len_0),
                gettext(b"(+%ld for BOM)\0".as_ptr() as *const ::core::ffi::c_char),
                bom_count,
            );
        }
        if dict.is_null() {
            let mut p_0: *mut ::core::ffi::c_char = p_shm.get();
            p_shm.set(b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char);
            if p_ch.get() < 1 as OptInt {
                msg_start();
                msg_scroll.set(true_0);
            }
            msg(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
            p_shm.set(p_0);
        }
    }
    if !dict.is_null() {
        tv_dict_add_nr(
            dict,
            b"words\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            word_count,
        );
        tv_dict_add_nr(
            dict,
            b"chars\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            char_count,
        );
        tv_dict_add_nr(
            dict,
            b"bytes\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            byte_count + bom_count,
        );
        tv_dict_add_nr(
            dict,
            if l_VIsual_active != 0 {
                b"visual_bytes\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"cursor_bytes\0".as_ptr() as *const ::core::ffi::c_char
            },
            ::core::mem::size_of::<[::core::ffi::c_char; 13]>().wrapping_sub(1 as size_t),
            byte_count_cursor,
        );
        tv_dict_add_nr(
            dict,
            if l_VIsual_active != 0 {
                b"visual_chars\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"cursor_chars\0".as_ptr() as *const ::core::ffi::c_char
            },
            ::core::mem::size_of::<[::core::ffi::c_char; 13]>().wrapping_sub(1 as size_t),
            char_count_cursor,
        );
        tv_dict_add_nr(
            dict,
            if l_VIsual_active != 0 {
                b"visual_words\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"cursor_words\0".as_ptr() as *const ::core::ffi::c_char
            },
            ::core::mem::size_of::<[::core::ffi::c_char; 13]>().wrapping_sub(1 as size_t),
            word_count_cursor,
        );
    }
}
unsafe extern "C" fn op_colon(mut oap: *mut oparg_T) {
    stuffcharReadbuff(':' as ::core::ffi::c_int);
    if (*oap).is_VIsual {
        stuffReadbuff(b"'<,'>\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        if (*oap).start.lnum == (*curwin.get()).w_cursor.lnum {
            stuffcharReadbuff('.' as ::core::ffi::c_int);
        } else {
            stuffnumReadbuff((*oap).start.lnum as ::core::ffi::c_int);
        }
        let mut endOfStartFold: linenr_T = (*oap).start.lnum;
        hasFolding(
            curwin.get(),
            (*oap).start.lnum,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut endOfStartFold,
        );
        if (*oap).end.lnum != (*oap).start.lnum && (*oap).end.lnum != endOfStartFold {
            stuffcharReadbuff(',' as ::core::ffi::c_int);
            if (*oap).end.lnum == (*curwin.get()).w_cursor.lnum {
                stuffcharReadbuff('.' as ::core::ffi::c_int);
            } else if (*oap).end.lnum == (*curbuf.get()).b_ml.ml_line_count {
                stuffcharReadbuff('$' as ::core::ffi::c_int);
            } else if (*oap).start.lnum == (*curwin.get()).w_cursor.lnum
                && !hasFolding(
                    curwin.get(),
                    (*oap).end.lnum,
                    ::core::ptr::null_mut::<linenr_T>(),
                    ::core::ptr::null_mut::<linenr_T>(),
                )
            {
                stuffReadbuff(b".+\0".as_ptr() as *const ::core::ffi::c_char);
                stuffnumReadbuff((*oap).line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            } else {
                stuffnumReadbuff((*oap).end.lnum as ::core::ffi::c_int);
            }
        }
    }
    if (*oap).op_type != OP_COLON as ::core::ffi::c_int {
        stuffReadbuff(b"!\0".as_ptr() as *const ::core::ffi::c_char);
    }
    if (*oap).op_type == OP_INDENT as ::core::ffi::c_int {
        stuffReadbuff(get_equalprg());
        stuffReadbuff(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
    } else if (*oap).op_type == OP_FORMAT as ::core::ffi::c_int {
        if *(*curbuf.get()).b_p_fp as ::core::ffi::c_int != NUL {
            stuffReadbuff((*curbuf.get()).b_p_fp);
        } else if *p_fp.get() as ::core::ffi::c_int != NUL {
            stuffReadbuff(p_fp.get());
        } else {
            stuffReadbuff(b"fmt\0".as_ptr() as *const ::core::ffi::c_char);
        }
        stuffReadbuff(b"\n']\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
static opfunc_cb: GlobalCell<Callback> = GlobalCell::new(Callback {
    data: C2Rust_Unnamed_5 {
        funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    type_0: kCallbackNone,
});
pub unsafe extern "C" fn did_set_operatorfunc(
    mut _args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    if option_set_callback_func(p_opfunc.get(), opfunc_cb.ptr()) == FAIL {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn set_ref_in_opfunc(mut copyID: ::core::ffi::c_int) -> bool {
    return set_ref_in_callback(
        opfunc_cb.ptr(),
        copyID,
        ::core::ptr::null_mut::<*mut ht_stack_T>(),
        ::core::ptr::null_mut::<*mut list_stack_T>(),
    );
}
unsafe extern "C" fn op_function(mut oap: *const oparg_T) {
    let orig_start: pos_T = (*curbuf.get()).b_op_start;
    let orig_end: pos_T = (*curbuf.get()).b_op_end;
    if *p_opfunc.get() as ::core::ffi::c_int == NUL {
        emsg(gettext(
            b"E774: 'operatorfunc' is empty\0".as_ptr() as *const ::core::ffi::c_char
        ));
    } else {
        (*curbuf.get()).b_op_start = (*oap).start;
        (*curbuf.get()).b_op_end = (*oap).end;
        if (*oap).motion_type as ::core::ffi::c_int != kMTLineWise as ::core::ffi::c_int
            && !(*oap).inclusive
        {
            decl(&raw mut (*curbuf.get()).b_op_end);
        }
        let mut argv: [typval_T; 2] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 2];
        argv[0 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
        argv[1 as ::core::ffi::c_int as usize].v_type = VAR_UNKNOWN;
        argv[0 as ::core::ffi::c_int as usize].vval.v_string = [
            b"char\0".as_ptr() as *const ::core::ffi::c_char,
            b"line\0".as_ptr() as *const ::core::ffi::c_char,
            b"block\0".as_ptr() as *const ::core::ffi::c_char,
        ][(*oap).motion_type as usize]
            as *mut ::core::ffi::c_char;
        let save_virtual_op: TriState = virtual_op.get();
        virtual_op.set(kNone);
        let save_finish_op: bool = finish_op.get();
        finish_op.set(false_0 != 0);
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if callback_call(
            opfunc_cb.ptr(),
            1 as ::core::ffi::c_int,
            &raw mut argv as *mut typval_T,
            &raw mut rettv,
        ) {
            tv_clear(&raw mut rettv);
        }
        virtual_op.set(save_virtual_op);
        finish_op.set(save_finish_op);
        if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int != 0 {
            (*curbuf.get()).b_op_start = orig_start;
            (*curbuf.get()).b_op_end = orig_end;
        }
    };
}
unsafe extern "C" fn get_op_vcol(
    mut oap: *mut oparg_T,
    mut redo_VIsual_vcol: colnr_T,
    mut initial: bool,
) {
    let mut start: colnr_T = 0;
    let mut end: colnr_T = 0;
    if VIsual_mode.get() != Ctrl_V || !initial && (*oap).end.col < (*curwin.get()).w_view_width {
        return;
    }
    (*oap).motion_type = kMTBlockWise;
    mark_mb_adjustpos((*curwin.get()).w_buffer, &raw mut (*oap).end);
    getvvcol(
        curwin.get(),
        &raw mut (*oap).start,
        &raw mut (*oap).start_vcol,
        ::core::ptr::null_mut::<colnr_T>(),
        &raw mut (*oap).end_vcol,
    );
    if !redo_VIsual_busy.get() {
        getvvcol(
            curwin.get(),
            &raw mut (*oap).end,
            &raw mut start,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut end,
        );
        (*oap).start_vcol = if (*oap).start_vcol < start {
            (*oap).start_vcol
        } else {
            start
        };
        if end > (*oap).end_vcol {
            if initial as ::core::ffi::c_int != 0
                && *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
                && start >= 1 as ::core::ffi::c_int
                && start as ::core::ffi::c_int - 1 as ::core::ffi::c_int >= (*oap).end_vcol
            {
                (*oap).end_vcol =
                    (start as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as colnr_T;
            } else {
                (*oap).end_vcol = end;
            }
        }
    }
    if (*curwin.get()).w_curswant == MAXCOL as ::core::ffi::c_int {
        (*curwin.get()).w_cursor.col = MAXCOL as ::core::ffi::c_int as colnr_T;
        (*oap).end_vcol = 0 as ::core::ffi::c_int as colnr_T;
        (*curwin.get()).w_cursor.lnum = (*oap).start.lnum;
        while (*curwin.get()).w_cursor.lnum <= (*oap).end.lnum {
            getvvcol(
                curwin.get(),
                &raw mut (*curwin.get()).w_cursor,
                ::core::ptr::null_mut::<colnr_T>(),
                ::core::ptr::null_mut::<colnr_T>(),
                &raw mut end,
            );
            (*oap).end_vcol = if (*oap).end_vcol > end {
                (*oap).end_vcol
            } else {
                end
            };
            (*curwin.get()).w_cursor.lnum += 1;
        }
    } else if redo_VIsual_busy.get() {
        (*oap).end_vcol = ((*oap).start_vcol as ::core::ffi::c_int
            + redo_VIsual_vcol as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as colnr_T;
    }
    (*curwin.get()).w_cursor.lnum = (*oap).end.lnum;
    coladvance(curwin.get(), (*oap).end_vcol);
    (*oap).end = (*curwin.get()).w_cursor;
    (*curwin.get()).w_cursor = (*oap).start;
    coladvance(curwin.get(), (*oap).start_vcol);
    (*oap).start = (*curwin.get()).w_cursor;
}
unsafe extern "C" fn is_ex_cmdchar(mut cap: *mut cmdarg_T) -> bool {
    return (*cap).cmdchar == ':' as ::core::ffi::c_int
        || (*cap).cmdchar
            == -(253 as ::core::ffi::c_int
                + ((KE_COMMAND as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
}
pub unsafe extern "C" fn do_pending_operator(
    mut cap: *mut cmdarg_T,
    mut old_col: ::core::ffi::c_int,
    mut gui_yank: bool,
) {
    let mut oap: *mut oparg_T = (*cap).oap;
    let mut lbr_saved: ::core::ffi::c_int = (*curwin.get()).w_onebuf_opt.wo_lbr;
    static redo_VIsual: GlobalCell<redo_VIsual_T> = GlobalCell::new(redo_VIsual_T {
        rv_mode: NUL,
        rv_line_count: 0 as linenr_T,
        rv_vcol: 0 as colnr_T,
        rv_count: 0 as ::core::ffi::c_int,
        rv_arg: 0 as ::core::ffi::c_int,
    });
    let mut old_cursor: pos_T = (*curwin.get()).w_cursor;
    if (finish_op.get() as ::core::ffi::c_int != 0
        || VIsual_active.get() as ::core::ffi::c_int != 0)
        && (*oap).op_type != OP_NOP as ::core::ffi::c_int
    {
        let mut empty_region_error: bool = false;
        let mut restart_edit_save: ::core::ffi::c_int = 0;
        let mut include_line_break: bool = false_0 != 0;
        let redo_yank: bool = !vim_strchr(p_cpo.get(), CPO_YANK).is_null() && !gui_yank;
        reset_lbr();
        (*oap).is_VIsual = VIsual_active.get();
        if (*oap).motion_force == 'V' as ::core::ffi::c_int {
            (*oap).motion_type = kMTLineWise;
        } else if (*oap).motion_force == 'v' as ::core::ffi::c_int {
            if (*oap).motion_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
                (*oap).inclusive = false_0 != 0;
            } else if (*oap).motion_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
            {
                (*oap).inclusive = !(*oap).inclusive;
            }
            (*oap).motion_type = kMTCharWise;
        } else if (*oap).motion_force == Ctrl_V {
            if !VIsual_active.get() {
                VIsual_active.set(true_0 != 0);
                VIsual.set((*oap).start);
            }
            VIsual_mode.set(Ctrl_V);
            VIsual_select.set(false_0 != 0);
            VIsual_reselect.set(false_0);
        }
        if (redo_yank as ::core::ffi::c_int != 0 || (*oap).op_type != OP_YANK as ::core::ffi::c_int)
            && (!VIsual_active.get()
                || (*oap).motion_force != 0
                || (is_ex_cmdchar(cap) as ::core::ffi::c_int != 0
                    || (*cap).cmdchar
                        == -(253 as ::core::ffi::c_int
                            + ((KE_LUA as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)))
                    && (*oap).op_type != OP_COLON as ::core::ffi::c_int)
            && (*cap).cmdchar != 'D' as ::core::ffi::c_int
            && (*oap).op_type != OP_FOLD as ::core::ffi::c_int
            && (*oap).op_type != OP_FOLDOPEN as ::core::ffi::c_int
            && (*oap).op_type != OP_FOLDOPENREC as ::core::ffi::c_int
            && (*oap).op_type != OP_FOLDCLOSE as ::core::ffi::c_int
            && (*oap).op_type != OP_FOLDCLOSEREC as ::core::ffi::c_int
            && (*oap).op_type != OP_FOLDDEL as ::core::ffi::c_int
            && (*oap).op_type != OP_FOLDDELREC as ::core::ffi::c_int
        {
            prep_redo(
                (*oap).regname,
                (*cap).count0,
                get_op_char((*oap).op_type),
                get_extra_op_char((*oap).op_type),
                (*oap).motion_force,
                (*cap).cmdchar,
                (*cap).nchar,
            );
            if (*cap).cmdchar == '/' as ::core::ffi::c_int
                || (*cap).cmdchar == '?' as ::core::ffi::c_int
            {
                if vim_strchr(p_cpo.get(), CPO_REDO).is_null() {
                    AppendToRedobuffLit((*cap).searchbuf, -1 as ::core::ffi::c_int);
                }
                AppendToRedobuff(NL_STR.as_ptr());
            } else if is_ex_cmdchar(cap) {
                if (*repeat_cmdline.ptr()).is_null() {
                    ResetRedobuff();
                } else {
                    if (*cap).cmdchar == ':' as ::core::ffi::c_int {
                        AppendToRedobuffLit(repeat_cmdline.get(), -1 as ::core::ffi::c_int);
                    } else {
                        AppendToRedobuffSpec(repeat_cmdline.get());
                    }
                    AppendToRedobuff(NL_STR.as_ptr());
                    let mut ptr_: *mut *mut ::core::ffi::c_void =
                        repeat_cmdline.ptr() as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr_);
                    *ptr_ = NULL;
                    let _ = *ptr_;
                }
            } else if (*cap).cmdchar
                == -(253 as ::core::ffi::c_int
                    + ((KE_LUA as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                AppendNumberToRedobuff(repeat_luaref.get() as ::core::ffi::c_int);
                AppendToRedobuff(NL_STR.as_ptr());
            }
        }
        if redo_VIsual_busy.get() {
            (*oap).start = (*curwin.get()).w_cursor;
            (*curwin.get()).w_cursor.lnum = ((*curwin.get()).w_cursor.lnum as ::core::ffi::c_int
                + ((*redo_VIsual.ptr()).rv_line_count - 1 as linenr_T) as ::core::ffi::c_int)
                as linenr_T;
            (*curwin.get()).w_cursor.lnum =
                if (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count {
                    (*curwin.get()).w_cursor.lnum
                } else {
                    (*curbuf.get()).b_ml.ml_line_count
                };
            VIsual_mode.set((*redo_VIsual.ptr()).rv_mode);
            if (*redo_VIsual.ptr()).rv_vcol == MAXCOL as ::core::ffi::c_int
                || VIsual_mode.get() == 'v' as ::core::ffi::c_int
            {
                if VIsual_mode.get() == 'v' as ::core::ffi::c_int {
                    if (*redo_VIsual.ptr()).rv_line_count <= 1 as linenr_T {
                        validate_virtcol(curwin.get());
                        (*curwin.get()).w_curswant =
                            ((*curwin.get()).w_virtcol as ::core::ffi::c_int
                                + (*redo_VIsual.ptr()).rv_vcol as ::core::ffi::c_int
                                - 1 as ::core::ffi::c_int) as colnr_T;
                    } else {
                        (*curwin.get()).w_curswant = (*redo_VIsual.ptr()).rv_vcol;
                    }
                } else {
                    (*curwin.get()).w_curswant = MAXCOL as ::core::ffi::c_int as colnr_T;
                }
                coladvance(curwin.get(), (*curwin.get()).w_curswant);
            }
            (*cap).count0 = (*redo_VIsual.ptr()).rv_count;
            (*cap).count1 = if (*cap).count0 == 0 as ::core::ffi::c_int {
                1 as ::core::ffi::c_int
            } else {
                (*cap).count0
            };
        } else if VIsual_active.get() {
            if !gui_yank {
                (*curbuf.get()).b_visual.vi_start = VIsual.get();
                (*curbuf.get()).b_visual.vi_end = (*curwin.get()).w_cursor;
                (*curbuf.get()).b_visual.vi_mode = VIsual_mode.get();
                restore_visual_mode();
                (*curbuf.get()).b_visual.vi_curswant = (*curwin.get()).w_curswant;
                (*curbuf.get()).b_visual_mode_eval = VIsual_mode.get();
            }
            if VIsual_select.get() as ::core::ffi::c_int != 0
                && VIsual_mode.get() == 'V' as ::core::ffi::c_int
                && (*(*cap).oap).op_type != OP_DELETE as ::core::ffi::c_int
            {
                if lt(VIsual.get(), (*curwin.get()).w_cursor) {
                    (*VIsual.ptr()).col = 0 as ::core::ffi::c_int as colnr_T;
                    (*curwin.get()).w_cursor.col = ml_get_len((*curwin.get()).w_cursor.lnum);
                } else {
                    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                    (*VIsual.ptr()).col = ml_get_len((*VIsual.ptr()).lnum);
                }
                VIsual_mode.set('v' as ::core::ffi::c_int);
            } else if VIsual_mode.get() == 'v' as ::core::ffi::c_int {
                include_line_break = unadjust_for_sel();
            }
            (*oap).start = VIsual.get();
            if VIsual_mode.get() == 'V' as ::core::ffi::c_int {
                (*oap).start.col = 0 as ::core::ffi::c_int as colnr_T;
                (*oap).start.coladd = 0 as ::core::ffi::c_int as colnr_T;
            }
        }
        if lt((*oap).start, (*curwin.get()).w_cursor) {
            if !VIsual_active.get() {
                if hasFolding(
                    curwin.get(),
                    (*oap).start.lnum,
                    &raw mut (*oap).start.lnum,
                    ::core::ptr::null_mut::<linenr_T>(),
                ) {
                    (*oap).start.col = 0 as ::core::ffi::c_int as colnr_T;
                }
                if ((*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
                    || (*oap).inclusive as ::core::ffi::c_int != 0
                    || (*oap).motion_type as ::core::ffi::c_int
                        == kMTLineWise as ::core::ffi::c_int)
                    && hasFolding(
                        curwin.get(),
                        (*curwin.get()).w_cursor.lnum,
                        ::core::ptr::null_mut::<linenr_T>(),
                        &raw mut (*curwin.get()).w_cursor.lnum,
                    ) as ::core::ffi::c_int
                        != 0
                {
                    (*curwin.get()).w_cursor.col = get_cursor_line_len();
                }
            }
            (*oap).end = (*curwin.get()).w_cursor;
            (*curwin.get()).w_cursor = (*oap).start;
            (*curwin.get()).w_valid &= !VALID_VIRTCOL;
        } else {
            if !VIsual_active.get()
                && (*oap).motion_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
            {
                if hasFolding(
                    curwin.get(),
                    (*curwin.get()).w_cursor.lnum,
                    &raw mut (*curwin.get()).w_cursor.lnum,
                    ::core::ptr::null_mut::<linenr_T>(),
                ) {
                    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                }
                if hasFolding(
                    curwin.get(),
                    (*oap).start.lnum,
                    ::core::ptr::null_mut::<linenr_T>(),
                    &raw mut (*oap).start.lnum,
                ) {
                    (*oap).start.col = ml_get_len((*oap).start.lnum);
                }
            }
            (*oap).end = (*oap).start;
            (*oap).start = (*curwin.get()).w_cursor;
        }
        check_pos((*curwin.get()).w_buffer, &raw mut (*oap).end);
        (*oap).line_count = (*oap).end.lnum - (*oap).start.lnum + 1 as linenr_T;
        virtual_op.set(virtual_active(curwin.get()) as TriState);
        if VIsual_active.get() as ::core::ffi::c_int != 0
            || redo_VIsual_busy.get() as ::core::ffi::c_int != 0
        {
            get_op_vcol(oap, (*redo_VIsual.ptr()).rv_vcol, true_0 != 0);
            if !redo_VIsual_busy.get() && !gui_yank {
                resel_VIsual_mode.set(VIsual_mode.get());
                if (*curwin.get()).w_curswant == MAXCOL as ::core::ffi::c_int {
                    resel_VIsual_vcol.set(MAXCOL as ::core::ffi::c_int as colnr_T);
                } else {
                    if VIsual_mode.get() != Ctrl_V {
                        getvvcol(
                            curwin.get(),
                            &raw mut (*oap).end,
                            ::core::ptr::null_mut::<colnr_T>(),
                            ::core::ptr::null_mut::<colnr_T>(),
                            &raw mut (*oap).end_vcol,
                        );
                    }
                    if VIsual_mode.get() == Ctrl_V || (*oap).line_count <= 1 as linenr_T {
                        if VIsual_mode.get() != Ctrl_V {
                            getvvcol(
                                curwin.get(),
                                &raw mut (*oap).start,
                                &raw mut (*oap).start_vcol,
                                ::core::ptr::null_mut::<colnr_T>(),
                                ::core::ptr::null_mut::<colnr_T>(),
                            );
                        }
                        resel_VIsual_vcol.set(
                            ((*oap).end_vcol as ::core::ffi::c_int
                                - (*oap).start_vcol as ::core::ffi::c_int
                                + 1 as ::core::ffi::c_int) as colnr_T,
                        );
                    } else {
                        resel_VIsual_vcol.set((*oap).end_vcol);
                    }
                }
                resel_VIsual_line_count.set((*oap).line_count);
            }
            if (redo_yank as ::core::ffi::c_int != 0
                || (*oap).op_type != OP_YANK as ::core::ffi::c_int)
                && (*oap).op_type != OP_COLON as ::core::ffi::c_int
                && (*oap).op_type != OP_FOLD as ::core::ffi::c_int
                && (*oap).op_type != OP_FOLDOPEN as ::core::ffi::c_int
                && (*oap).op_type != OP_FOLDOPENREC as ::core::ffi::c_int
                && (*oap).op_type != OP_FOLDCLOSE as ::core::ffi::c_int
                && (*oap).op_type != OP_FOLDCLOSEREC as ::core::ffi::c_int
                && (*oap).op_type != OP_FOLDDEL as ::core::ffi::c_int
                && (*oap).op_type != OP_FOLDDELREC as ::core::ffi::c_int
                && (*oap).motion_force == NUL
            {
                if (*cap).cmdchar == 'g' as ::core::ffi::c_int
                    && ((*cap).nchar == 'n' as ::core::ffi::c_int
                        || (*cap).nchar == 'N' as ::core::ffi::c_int)
                {
                    prep_redo(
                        (*oap).regname,
                        (*cap).count0,
                        get_op_char((*oap).op_type),
                        get_extra_op_char((*oap).op_type),
                        (*oap).motion_force,
                        (*cap).cmdchar,
                        (*cap).nchar,
                    );
                } else if !is_ex_cmdchar(cap)
                    && (*cap).cmdchar
                        != -(253 as ::core::ffi::c_int
                            + ((KE_LUA as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                {
                    let mut opchar: ::core::ffi::c_int = get_op_char((*oap).op_type);
                    let mut extra_opchar: ::core::ffi::c_int = get_extra_op_char((*oap).op_type);
                    let mut nchar: ::core::ffi::c_int =
                        if (*oap).op_type == OP_REPLACE as ::core::ffi::c_int {
                            (*cap).nchar
                        } else {
                            NUL
                        };
                    if nchar == REPLACE_CR_NCHAR as ::core::ffi::c_int {
                        nchar = CAR;
                    } else if nchar == REPLACE_NL_NCHAR as ::core::ffi::c_int {
                        nchar = NL;
                    }
                    if opchar == 'g' as ::core::ffi::c_int
                        && extra_opchar == '@' as ::core::ffi::c_int
                    {
                        prep_redo_num2(
                            (*oap).regname,
                            0 as ::core::ffi::c_int,
                            NUL,
                            'v' as ::core::ffi::c_int,
                            (*cap).count0,
                            opchar,
                            extra_opchar,
                            nchar,
                        );
                    } else {
                        prep_redo(
                            (*oap).regname,
                            0 as ::core::ffi::c_int,
                            NUL,
                            'v' as ::core::ffi::c_int,
                            opchar,
                            extra_opchar,
                            nchar,
                        );
                    }
                }
                if !redo_VIsual_busy.get() {
                    (*redo_VIsual.ptr()).rv_mode = resel_VIsual_mode.get();
                    (*redo_VIsual.ptr()).rv_vcol = resel_VIsual_vcol.get();
                    (*redo_VIsual.ptr()).rv_line_count = resel_VIsual_line_count.get();
                    (*redo_VIsual.ptr()).rv_count = (*cap).count0;
                    (*redo_VIsual.ptr()).rv_arg = (*cap).arg;
                }
            }
            if (*oap).motion_force == NUL
                || (*oap).motion_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
            {
                (*oap).inclusive = true_0 != 0;
            }
            if VIsual_mode.get() == 'V' as ::core::ffi::c_int {
                (*oap).motion_type = kMTLineWise;
            } else if VIsual_mode.get() == 'v' as ::core::ffi::c_int {
                (*oap).motion_type = kMTCharWise;
                if *ml_get_pos(&raw mut (*oap).end) as ::core::ffi::c_int == NUL
                    && (include_line_break as ::core::ffi::c_int != 0
                        || virtual_op.get() as u64 == 0)
                {
                    (*oap).inclusive = false_0 != 0;
                    if *p_sel.get() as ::core::ffi::c_int != 'o' as ::core::ffi::c_int
                        && op_on_lines((*oap).op_type) == 0
                        && (*oap).end.lnum < (*curbuf.get()).b_ml.ml_line_count
                    {
                        (*oap).end.lnum += 1;
                        (*oap).end.col = 0 as ::core::ffi::c_int as colnr_T;
                        (*oap).end.coladd = 0 as ::core::ffi::c_int as colnr_T;
                        (*oap).line_count += 1;
                    }
                }
            }
            redo_VIsual_busy.set(false_0 != 0);
            if !gui_yank {
                VIsual_active.set(false_0 != 0);
                setmouse();
                mouse_dragging.set(0 as ::core::ffi::c_int);
                may_clear_cmdline();
                if ((*oap).op_type == OP_YANK as ::core::ffi::c_int
                    || (*oap).op_type == OP_COLON as ::core::ffi::c_int
                    || (*oap).op_type == OP_FUNCTION as ::core::ffi::c_int
                    || (*oap).op_type == OP_FILTER as ::core::ffi::c_int)
                    && (*oap).motion_force == NUL
                {
                    restore_lbr(lbr_saved != 0);
                    redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
                }
            }
        }
        if (*oap).inclusive {
            let l: ::core::ffi::c_int = utfc_ptr2len(ml_get_pos(&raw mut (*oap).end));
            if l > 1 as ::core::ffi::c_int {
                (*oap).end.col += l - 1 as ::core::ffi::c_int;
            }
        }
        (*curwin.get()).w_set_curswant = true_0;
        (*oap).empty = (*oap).motion_type as ::core::ffi::c_int
            != kMTLineWise as ::core::ffi::c_int
            && (!(*oap).inclusive
                || (*oap).op_type == OP_YANK as ::core::ffi::c_int
                    && gchar_pos(&raw mut (*oap).end) == NUL)
            && equalpos((*oap).start, (*oap).end) as ::core::ffi::c_int != 0
            && !(virtual_op.get() as ::core::ffi::c_int != 0
                && (*oap).start.coladd != (*oap).end.coladd);
        empty_region_error = (*oap).empty as ::core::ffi::c_int != 0
            && !vim_strchr(p_cpo.get(), CPO_EMPTYREGION).is_null();
        if (*oap).is_VIsual as ::core::ffi::c_int != 0
            && ((*oap).empty as ::core::ffi::c_int != 0
                || (*curbuf.get()).b_p_ma == 0
                || (*oap).op_type == OP_FOLD as ::core::ffi::c_int)
        {
            restore_lbr(lbr_saved != 0);
            redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
        }
        if (*oap).motion_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
            && (*oap).inclusive as ::core::ffi::c_int == false_0
            && (*cap).retval & CA_NO_ADJ_OP_END as ::core::ffi::c_int == 0
            && (*oap).end.col == 0 as ::core::ffi::c_int
            && (!(*oap).is_VIsual
                || *p_sel.get() as ::core::ffi::c_int == 'o' as ::core::ffi::c_int)
            && (*oap).line_count > 1 as linenr_T
        {
            (*oap).end_adjusted = true_0 != 0;
            (*oap).line_count -= 1;
            (*oap).end.lnum -= 1;
            if inindent(0 as ::core::ffi::c_int) {
                (*oap).motion_type = kMTLineWise;
            } else {
                (*oap).end.col = ml_get_len((*oap).end.lnum);
                if (*oap).end.col != 0 {
                    (*oap).end.col -= 1;
                    (*oap).inclusive = true_0 != 0;
                }
            }
        } else {
            (*oap).end_adjusted = false_0 != 0;
        }
        's_1511: {
            match (*oap).op_type {
                4 | 5 => {
                    op_shift(
                        oap,
                        true_0 != 0,
                        if (*oap).is_VIsual as ::core::ffi::c_int != 0 {
                            (*cap).count1
                        } else {
                            1 as ::core::ffi::c_int
                        },
                    );
                    auto_format(false_0 != 0, true_0 != 0);
                    break 's_1511;
                }
                14 | 13 => {
                    (*oap).line_count = if (*oap).line_count > 2 as linenr_T {
                        (*oap).line_count
                    } else {
                        2 as linenr_T
                    };
                    if (*curwin.get()).w_cursor.lnum + (*oap).line_count - 1 as linenr_T
                        > (*curbuf.get()).b_ml.ml_line_count
                    {
                        beep_flush();
                    } else {
                        do_join(
                            (*oap).line_count as size_t,
                            (*oap).op_type == OP_JOIN as ::core::ffi::c_int,
                            true_0 != 0,
                            true_0 != 0,
                            true_0 != 0,
                        );
                        auto_format(false_0 != 0, true_0 != 0);
                    }
                    break 's_1511;
                }
                1 => {
                    VIsual_reselect.set(false_0);
                    if empty_region_error {
                        vim_beep(kOptBoFlagOperator as ::core::ffi::c_int as ::core::ffi::c_uint);
                        CancelRedo();
                    } else {
                        op_delete(oap);
                        if (*oap).motion_type as ::core::ffi::c_int
                            == kMTLineWise as ::core::ffi::c_int
                            && has_format_option(FO_AUTO) as ::core::ffi::c_int != 0
                            && u_save_cursor() == OK
                        {
                            auto_format(false_0 != 0, true_0 != 0);
                        }
                    }
                    break 's_1511;
                }
                2 => {
                    if empty_region_error {
                        if !gui_yank {
                            vim_beep(
                                kOptBoFlagOperator as ::core::ffi::c_int as ::core::ffi::c_uint,
                            );
                            CancelRedo();
                        }
                    } else {
                        restore_lbr(lbr_saved != 0);
                        (*oap).excl_tr_ws = (*cap).cmdchar == 'z' as ::core::ffi::c_int;
                        op_yank(oap, !gui_yank);
                    }
                    check_cursor_col(curwin.get());
                    break 's_1511;
                }
                3 => {
                    VIsual_reselect.set(false_0);
                    if empty_region_error {
                        vim_beep(kOptBoFlagOperator as ::core::ffi::c_int as ::core::ffi::c_uint);
                        CancelRedo();
                    } else {
                        if !KeyTyped.get() {
                            restart_edit_save = restart_edit.get();
                        } else {
                            restart_edit_save = 0 as ::core::ffi::c_int;
                        }
                        restart_edit.set(0 as ::core::ffi::c_int);
                        restore_lbr(lbr_saved != 0);
                        (*curbuf.get()).b_last_changedtick_i = buf_get_changedtick(curbuf.get());
                        if op_change(oap) != 0 {
                            (*cap).retval |= CA_COMMAND_BUSY as ::core::ffi::c_int;
                        }
                        if restart_edit.get() == 0 as ::core::ffi::c_int {
                            restart_edit.set(restart_edit_save);
                        }
                    }
                    break 's_1511;
                }
                6 => {
                    if !vim_strchr(p_cpo.get(), CPO_FILTER).is_null() {
                        AppendToRedobuff(b"!\r\0".as_ptr() as *const ::core::ffi::c_char);
                    } else {
                        bangredo.set(true_0 != 0);
                    }
                }
                8 | 10 => {}
                7 | 11 | 12 | 15 => {
                    if empty_region_error {
                        vim_beep(kOptBoFlagOperator as ::core::ffi::c_int as ::core::ffi::c_uint);
                        CancelRedo();
                    } else {
                        op_tilde(oap);
                    }
                    check_cursor_col(curwin.get());
                    break 's_1511;
                }
                9 => {
                    if *(*curbuf.get()).b_p_fex as ::core::ffi::c_int != NUL {
                        op_formatexpr(oap);
                    } else if *p_fp.get() as ::core::ffi::c_int != NUL
                        || *(*curbuf.get()).b_p_fp as ::core::ffi::c_int != NUL
                    {
                        op_colon(oap);
                    } else {
                        op_format(oap, false_0 != 0);
                    }
                    break 's_1511;
                }
                26 => {
                    op_format(oap, true_0 != 0);
                    break 's_1511;
                }
                27 => {
                    let mut save_redo_VIsual: redo_VIsual_T = redo_VIsual.get();
                    restore_lbr(lbr_saved != 0);
                    op_function(oap);
                    redo_VIsual.set(save_redo_VIsual);
                    break 's_1511;
                }
                17 | 18 => {
                    VIsual_reselect.set(false_0);
                    if empty_region_error {
                        vim_beep(kOptBoFlagOperator as ::core::ffi::c_int as ::core::ffi::c_uint);
                        CancelRedo();
                    } else {
                        restart_edit_save = restart_edit.get();
                        restart_edit.set(0 as ::core::ffi::c_int);
                        restore_lbr(lbr_saved != 0);
                        (*curbuf.get()).b_last_changedtick_i = buf_get_changedtick(curbuf.get());
                        op_insert(oap, (*cap).count1);
                        reset_lbr();
                        auto_format(false_0 != 0, true_0 != 0);
                        if restart_edit.get() == 0 as ::core::ffi::c_int {
                            restart_edit.set(restart_edit_save);
                        } else {
                            (*cap).retval |= CA_COMMAND_BUSY as ::core::ffi::c_int;
                        }
                    }
                    break 's_1511;
                }
                16 => {
                    VIsual_reselect.set(false_0);
                    if empty_region_error {
                        vim_beep(kOptBoFlagOperator as ::core::ffi::c_int as ::core::ffi::c_uint);
                        CancelRedo();
                    } else {
                        restore_lbr(lbr_saved != 0);
                        op_replace(oap, (*cap).nchar);
                    }
                    break 's_1511;
                }
                19 => {
                    VIsual_reselect.set(false_0);
                    foldCreate(curwin.get(), (*oap).start, (*oap).end);
                    break 's_1511;
                }
                20 | 21 | 22 | 23 => {
                    VIsual_reselect.set(false_0);
                    opFoldRange(
                        (*oap).start,
                        (*oap).end,
                        ((*oap).op_type == OP_FOLDOPEN as ::core::ffi::c_int
                            || (*oap).op_type == OP_FOLDOPENREC as ::core::ffi::c_int)
                            as ::core::ffi::c_int,
                        ((*oap).op_type == OP_FOLDOPENREC as ::core::ffi::c_int
                            || (*oap).op_type == OP_FOLDCLOSEREC as ::core::ffi::c_int)
                            as ::core::ffi::c_int,
                        (*oap).is_VIsual,
                    );
                    break 's_1511;
                }
                24 | 25 => {
                    VIsual_reselect.set(false_0);
                    deleteFold(
                        curwin.get(),
                        (*oap).start.lnum,
                        (*oap).end.lnum,
                        ((*oap).op_type == OP_FOLDDELREC as ::core::ffi::c_int)
                            as ::core::ffi::c_int,
                        (*oap).is_VIsual,
                    );
                    break 's_1511;
                }
                28 | 29 => {
                    if empty_region_error {
                        vim_beep(kOptBoFlagOperator as ::core::ffi::c_int as ::core::ffi::c_uint);
                        CancelRedo();
                    } else {
                        VIsual_active.set(true_0 != 0);
                        restore_lbr(lbr_saved != 0);
                        op_addsub(
                            oap,
                            (*cap).count1 as linenr_T,
                            (*redo_VIsual.ptr()).rv_arg != 0,
                        );
                        VIsual_active.set(false_0 != 0);
                    }
                    check_cursor_col(curwin.get());
                    break 's_1511;
                }
                _ => {
                    clearopbeep(oap);
                    break 's_1511;
                }
            }
            if (*oap).op_type == OP_INDENT as ::core::ffi::c_int
                && *get_equalprg() as ::core::ffi::c_int == NUL
            {
                if (*curbuf.get()).b_p_lisp != 0 {
                    if use_indentexpr_for_lisp() {
                        op_reindent(
                            oap,
                            Some(get_expr_indent as unsafe extern "C" fn() -> ::core::ffi::c_int),
                        );
                    } else {
                        op_reindent(
                            oap,
                            Some(get_lisp_indent as unsafe extern "C" fn() -> ::core::ffi::c_int),
                        );
                    }
                } else {
                    op_reindent(
                        oap,
                        if *(*curbuf.get()).b_p_inde as ::core::ffi::c_int != NUL {
                            Some(get_expr_indent as unsafe extern "C" fn() -> ::core::ffi::c_int)
                        } else {
                            Some(get_c_indent as unsafe extern "C" fn() -> ::core::ffi::c_int)
                        },
                    );
                }
            } else {
                op_colon(oap);
            }
        }
        virtual_op.set(kNone);
        if !gui_yank {
            if p_sol.get() == 0
                && (*oap).motion_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
                && !(*oap).end_adjusted
                && ((*oap).op_type == OP_LSHIFT as ::core::ffi::c_int
                    || (*oap).op_type == OP_RSHIFT as ::core::ffi::c_int
                    || (*oap).op_type == OP_DELETE as ::core::ffi::c_int)
            {
                reset_lbr();
                (*curwin.get()).w_curswant = old_col as colnr_T;
                coladvance(curwin.get(), (*curwin.get()).w_curswant);
            }
        } else {
            (*curwin.get()).w_cursor = old_cursor;
        }
        clearop(oap);
        motion_force.set(NUL);
    }
    restore_lbr(lbr_saved != 0);
}
#[no_mangle]
pub unsafe extern "C" fn get_region_bytecount(
    mut buf: *mut buf_T,
    mut start_lnum: linenr_T,
    mut end_lnum: linenr_T,
    mut start_col: colnr_T,
    mut end_col: colnr_T,
) -> bcount_t {
    let mut max_lnum: linenr_T = (*buf).b_ml.ml_line_count;
    if start_lnum > max_lnum {
        return 0 as bcount_t;
    }
    if start_lnum == end_lnum {
        return (end_col - start_col) as bcount_t;
    }
    let mut deleted_bytes: bcount_t = (ml_get_buf_len(buf, start_lnum)
        - start_col as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int) as bcount_t;
    let mut i: linenr_T = 1 as linenr_T;
    while i <= end_lnum - start_lnum - 1 as linenr_T {
        if start_lnum + i > max_lnum {
            return deleted_bytes;
        }
        deleted_bytes +=
            (ml_get_buf_len(buf, start_lnum + i) + 1 as ::core::ffi::c_int) as bcount_t;
        i += 1;
    }
    if end_lnum > max_lnum {
        return deleted_bytes;
    }
    return deleted_bytes + end_col as bcount_t;
}
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
unsafe extern "C" fn is_append_register(mut regname: ::core::ffi::c_int) -> bool {
    return regname as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && regname as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
