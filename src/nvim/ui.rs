use crate::src::nvim::api::extmark::describe_ns;
use crate::src::nvim::api::private::helpers::{
    api_clear_error, arena_array, arena_dict, cstr_as_string,
};
use crate::src::nvim::api::private::validate::api_err_invalid;
use crate::src::nvim::api::ui::{
    remote_ui_bell, remote_ui_busy_start, remote_ui_busy_stop, remote_ui_chdir,
    remote_ui_default_colors_set, remote_ui_error_exit, remote_ui_event, remote_ui_flush,
    remote_ui_grid_clear, remote_ui_grid_cursor_goto, remote_ui_grid_resize, remote_ui_grid_scroll,
    remote_ui_hl_attr_define, remote_ui_hl_group_set, remote_ui_mode_change,
    remote_ui_mode_info_set, remote_ui_mouse_off, remote_ui_mouse_on, remote_ui_msg_set_pos,
    remote_ui_option_set, remote_ui_raw_line, remote_ui_screenshot, remote_ui_set_icon,
    remote_ui_set_title, remote_ui_stop, remote_ui_suspend, remote_ui_ui_send,
    remote_ui_update_menu, remote_ui_visual_bell, remote_ui_win_viewport,
    remote_ui_win_viewport_margins,
};
use crate::src::nvim::autocmd::do_autocmd_uienter;
use crate::src::nvim::buffer::resettitle;
use crate::src::nvim::cursor_shape::{cursor_get_mode_idx, mode_style_array, shape_table};
use crate::src::nvim::drawscreen::{conceal_check_cursor_line, screen_resize};
use crate::src::nvim::event::libuv::uv_cwd;
use crate::src::nvim::event::multiqueue::multiqueue_put_event;
use crate::src::nvim::ex_getln::cmdline_ui_flush;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::get_win_by_grid_handle;
use crate::src::nvim::highlight::{highlight_use_hlstate, ui_send_all_hls};
use crate::src::nvim::log::logmsg;
use crate::src::nvim::lua::executor::{api_free_luaref, nlua_call_ref_ctx};
use crate::src::nvim::main::{
    bo_flags, called_vim_beep, cterm_normal_bg_color, cterm_normal_fg_color, curbuf, curwin,
    default_grid, emsg_silent, exiting, expr_map_lock, first_tabpage, full_screen, in_assert_fails,
    msg_grid_adj, noargs, normal_bg, normal_fg, normal_sp, p_debug, p_guicursor, p_lz, p_mouse,
    p_tgc, p_vb, p_wd, rdb_flags, resize_events, starting, textlock, ui_client_channel_id,
    ui_event_ns_id, ui_ext_names, ui_refresh_cmdheight, updating_screen, State, VIsual_active,
};
use crate::src::nvim::map::{map_del_uint32_t_ptr_t, map_put_ref_uint32_t_ptr_t, mh_get_uint32_t};
use crate::src::nvim::memory::{arena_mem_free, strequal, xcalloc, xfree};
use crate::src::nvim::message::{
    msg, msg_ext_ui_flush, msg_schedule_semsg, msg_schedule_semsg_multiline, msg_scroll_flush,
    msg_source, msg_ui_refresh,
};
use crate::src::nvim::option::{set_option_value, ui_refresh_options};
use crate::src::nvim::os::libc::{__assert_fail, abort, gettext, llabs, memcpy, memset, strcmp};
use crate::src::nvim::os::time::{os_hrtime, os_sleep};
use crate::src::nvim::strings::vim_strchr;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, Arena, ArenaMem, Array, BoolVarValue, Boolean,
    BufUpdateCallbacks, Buffer, Callback, CallbackType, Callback_data as C2Rust_Unnamed_4,
    ChangedtickDictItem, CursorShape, DecorExt, DecorHighlightInline, DecorInlineData,
    DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_1, Dict, Error, ErrorType,
    Event, ExtmarkUndoObject, FileID, Float, FloatAnchor, FloatRelative, GridView, HlAttrs,
    Integer, Intersection, KeyValuePair, LineFlags, LuaRef, LuaRetMode, MTKey, MTNode, MTPos,
    MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MarkTree, MultiQueue, Object, ObjectType, OptIndex, OptInt, OptVal,
    OptValData, OptValType, PackerBuffer, PackerBufferFlush, RemoteUI, RgbValue, ScopeDictDictItem,
    ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue,
    StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_11, String_0, Tabpage,
    Terminal, Timestamp, TriState, UIExtension, VarLockStatus, VarType, VirtLines, VirtText,
    VirtTextChunk, VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle, Window, __time_t, alist_T,
    argv_callback, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T, chunksize_T, colnr_T,
    consumed_blk, cursorentry_T, dict_T, dictvar_S, diff_T, diffblock_S, disptick_T,
    extmark_undo_vec_t, fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_2,
    file_buffer_b_wininfo as C2Rust_Unnamed_10, file_buffer_update_callbacks as C2Rust_Unnamed,
    file_buffer_update_channels as C2Rust_Unnamed_0, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_5, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, key_value_pair, lcs_chars_T,
    linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T,
    mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T,
    mtnode_inner_s, mtnode_s, multiqueue, object, object_data as C2Rust_Unnamed_12,
    packer_buffer_t, partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t, qf_info_S,
    qf_info_T, queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T,
    sctx_T, size_t, syn_state, syn_state_sst_union as C2Rust_Unnamed_3, syn_time_T, synblock_T,
    synstate_T, tabpage_S, tabpage_T, taggy_T, terminal, time_t, typval_T, typval_vval_union,
    u_entry, u_entry_T, u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_7,
    u_header_uh_alt_prev as C2Rust_Unnamed_6, u_header_uh_next as C2Rust_Unnamed_9,
    u_header_uh_prev as C2Rust_Unnamed_8, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, varnumber_T, virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T,
    wline_T, xfmark_T, NS, QUEUE,
};
use crate::src::nvim::ui_compositor::{
    ui_comp_attach, ui_comp_detach, ui_comp_get_grid_at_coord, ui_comp_grid_cursor_goto,
    ui_comp_grid_resize, ui_comp_grid_scroll, ui_comp_init, ui_comp_msg_set_pos, ui_comp_raw_line,
    ui_comp_should_draw,
};
use crate::src::nvim::window::{win_set_inner_size, win_ui_flush};
use crate::src::nvim::winfloat::win_config_float;
extern "C" {
    fn arena_finish(arena: *mut Arena) -> ArenaMem;
}
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
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_13 = 76;
pub const HLF_PRE: C2Rust_Unnamed_13 = 75;
pub const HLF_OK: C2Rust_Unnamed_13 = 74;
pub const HLF_SO: C2Rust_Unnamed_13 = 73;
pub const HLF_SE: C2Rust_Unnamed_13 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_13 = 71;
pub const HLF_TS: C2Rust_Unnamed_13 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_13 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_13 = 68;
pub const HLF_CU: C2Rust_Unnamed_13 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_13 = 66;
pub const HLF_WBR: C2Rust_Unnamed_13 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_13 = 64;
pub const HLF_MSG: C2Rust_Unnamed_13 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_13 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_13 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_13 = 60;
pub const HLF_0: C2Rust_Unnamed_13 = 59;
pub const HLF_QFL: C2Rust_Unnamed_13 = 58;
pub const HLF_MC: C2Rust_Unnamed_13 = 57;
pub const HLF_CUL: C2Rust_Unnamed_13 = 56;
pub const HLF_CUC: C2Rust_Unnamed_13 = 55;
pub const HLF_TPF: C2Rust_Unnamed_13 = 54;
pub const HLF_TPS: C2Rust_Unnamed_13 = 53;
pub const HLF_TP: C2Rust_Unnamed_13 = 52;
pub const HLF_PBR: C2Rust_Unnamed_13 = 51;
pub const HLF_PST: C2Rust_Unnamed_13 = 50;
pub const HLF_PSB: C2Rust_Unnamed_13 = 49;
pub const HLF_PSX: C2Rust_Unnamed_13 = 48;
pub const HLF_PNX: C2Rust_Unnamed_13 = 47;
pub const HLF_PSK: C2Rust_Unnamed_13 = 46;
pub const HLF_PNK: C2Rust_Unnamed_13 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_13 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_13 = 43;
pub const HLF_PSI: C2Rust_Unnamed_13 = 42;
pub const HLF_PNI: C2Rust_Unnamed_13 = 41;
pub const HLF_SPL: C2Rust_Unnamed_13 = 40;
pub const HLF_SPR: C2Rust_Unnamed_13 = 39;
pub const HLF_SPC: C2Rust_Unnamed_13 = 38;
pub const HLF_SPB: C2Rust_Unnamed_13 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_13 = 36;
pub const HLF_SC: C2Rust_Unnamed_13 = 35;
pub const HLF_TXA: C2Rust_Unnamed_13 = 34;
pub const HLF_TXD: C2Rust_Unnamed_13 = 33;
pub const HLF_DED: C2Rust_Unnamed_13 = 32;
pub const HLF_CHD: C2Rust_Unnamed_13 = 31;
pub const HLF_ADD: C2Rust_Unnamed_13 = 30;
pub const HLF_FC: C2Rust_Unnamed_13 = 29;
pub const HLF_FL: C2Rust_Unnamed_13 = 28;
pub const HLF_WM: C2Rust_Unnamed_13 = 27;
pub const HLF_W: C2Rust_Unnamed_13 = 26;
pub const HLF_VNC: C2Rust_Unnamed_13 = 25;
pub const HLF_V: C2Rust_Unnamed_13 = 24;
pub const HLF_T: C2Rust_Unnamed_13 = 23;
pub const HLF_VSP: C2Rust_Unnamed_13 = 22;
pub const HLF_C: C2Rust_Unnamed_13 = 21;
pub const HLF_SNC: C2Rust_Unnamed_13 = 20;
pub const HLF_S: C2Rust_Unnamed_13 = 19;
pub const HLF_R: C2Rust_Unnamed_13 = 18;
pub const HLF_CLF: C2Rust_Unnamed_13 = 17;
pub const HLF_CLS: C2Rust_Unnamed_13 = 16;
pub const HLF_CLN: C2Rust_Unnamed_13 = 15;
pub const HLF_LNB: C2Rust_Unnamed_13 = 14;
pub const HLF_LNA: C2Rust_Unnamed_13 = 13;
pub const HLF_N: C2Rust_Unnamed_13 = 12;
pub const HLF_CM: C2Rust_Unnamed_13 = 11;
pub const HLF_M: C2Rust_Unnamed_13 = 10;
pub const HLF_LC: C2Rust_Unnamed_13 = 9;
pub const HLF_L: C2Rust_Unnamed_13 = 8;
pub const HLF_I: C2Rust_Unnamed_13 = 7;
pub const HLF_E: C2Rust_Unnamed_13 = 6;
pub const HLF_D: C2Rust_Unnamed_13 = 5;
pub const HLF_AT: C2Rust_Unnamed_13 = 4;
pub const HLF_TERM: C2Rust_Unnamed_13 = 3;
pub const HLF_EOB: C2Rust_Unnamed_13 = 2;
pub const HLF_8: C2Rust_Unnamed_13 = 1;
pub const HLF_NONE: C2Rust_Unnamed_13 = 0;
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
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const kLineFlagInvalid: C2Rust_Unnamed_14 = 2;
pub const kLineFlagWrap: C2Rust_Unnamed_14 = 1;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const SHAPE_IDX_COUNT: C2Rust_Unnamed_15 = 18;
pub const SHAPE_IDX_TERM: C2Rust_Unnamed_15 = 17;
pub const SHAPE_IDX_SM: C2Rust_Unnamed_15 = 16;
pub const SHAPE_IDX_MOREL: C2Rust_Unnamed_15 = 15;
pub const SHAPE_IDX_MORE: C2Rust_Unnamed_15 = 14;
pub const SHAPE_IDX_VDRAG: C2Rust_Unnamed_15 = 13;
pub const SHAPE_IDX_VSEP: C2Rust_Unnamed_15 = 12;
pub const SHAPE_IDX_SDRAG: C2Rust_Unnamed_15 = 11;
pub const SHAPE_IDX_STATUS: C2Rust_Unnamed_15 = 10;
pub const SHAPE_IDX_CLINE: C2Rust_Unnamed_15 = 9;
pub const SHAPE_IDX_VE: C2Rust_Unnamed_15 = 8;
pub const SHAPE_IDX_O: C2Rust_Unnamed_15 = 7;
pub const SHAPE_IDX_CR: C2Rust_Unnamed_15 = 6;
pub const SHAPE_IDX_CI: C2Rust_Unnamed_15 = 5;
pub const SHAPE_IDX_C: C2Rust_Unnamed_15 = 4;
pub const SHAPE_IDX_R: C2Rust_Unnamed_15 = 3;
pub const SHAPE_IDX_I: C2Rust_Unnamed_15 = 2;
pub const SHAPE_IDX_V: C2Rust_Unnamed_15 = 1;
pub const SHAPE_IDX_N: C2Rust_Unnamed_15 = 0;
pub const SHAPE_VER: CursorShape = 2;
pub const SHAPE_HOR: CursorShape = 1;
pub const SHAPE_BLOCK: CursorShape = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_16 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_16 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_16 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_16 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_16 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_16 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_16 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_16 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_16 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_16 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_16 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_16 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_16 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_16 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_16 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_16 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_16 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_16 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_16 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_16 = 1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const kOptBoFlagWildmode: C2Rust_Unnamed_17 = 524288;
pub const kOptBoFlagTerm: C2Rust_Unnamed_17 = 262144;
pub const kOptBoFlagSpell: C2Rust_Unnamed_17 = 131072;
pub const kOptBoFlagShell: C2Rust_Unnamed_17 = 65536;
pub const kOptBoFlagRegister: C2Rust_Unnamed_17 = 32768;
pub const kOptBoFlagOperator: C2Rust_Unnamed_17 = 16384;
pub const kOptBoFlagShowmatch: C2Rust_Unnamed_17 = 8192;
pub const kOptBoFlagMess: C2Rust_Unnamed_17 = 4096;
pub const kOptBoFlagLang: C2Rust_Unnamed_17 = 2048;
pub const kOptBoFlagInsertmode: C2Rust_Unnamed_17 = 1024;
pub const kOptBoFlagHangul: C2Rust_Unnamed_17 = 512;
pub const kOptBoFlagEx: C2Rust_Unnamed_17 = 256;
pub const kOptBoFlagEsc: C2Rust_Unnamed_17 = 128;
pub const kOptBoFlagError: C2Rust_Unnamed_17 = 64;
pub const kOptBoFlagCtrlg: C2Rust_Unnamed_17 = 32;
pub const kOptBoFlagCopy: C2Rust_Unnamed_17 = 16;
pub const kOptBoFlagComplete: C2Rust_Unnamed_17 = 8;
pub const kOptBoFlagCursor: C2Rust_Unnamed_17 = 4;
pub const kOptBoFlagBackspace: C2Rust_Unnamed_17 = 2;
pub const kOptBoFlagAll: C2Rust_Unnamed_17 = 1;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const kOptRdbFlagFlush: C2Rust_Unnamed_18 = 32;
pub const kOptRdbFlagLine: C2Rust_Unnamed_18 = 16;
pub const kOptRdbFlagNodelta: C2Rust_Unnamed_18 = 8;
pub const kOptRdbFlagInvalid: C2Rust_Unnamed_18 = 4;
pub const kOptRdbFlagNothrottle: C2Rust_Unnamed_18 = 2;
pub const kOptRdbFlagCompositor: C2Rust_Unnamed_18 = 1;
pub const kRetMulti: LuaRetMode = 3;
pub const kRetLuaref: LuaRetMode = 2;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const CB_MAX_ERROR: C2Rust_Unnamed_19 = 3;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct UIEventCallback {
    pub cb: LuaRef,
    pub errors: uint8_t,
    pub ext_widgets: [bool; 5],
}
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const ARENA_EMPTY: Arena = Arena {
    cur_blk: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    pos: 0 as size_t,
    size: 0 as size_t,
};
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
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
pub const SET_INIT: Set_uint32_t = Set_uint32_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<uint32_t>(),
};
pub const MAP_INIT: Map_uint32_t_ptr_t = Map_uint32_t_ptr_t {
    set: SET_INIT,
    values: ::core::ptr::null_mut::<ptr_t>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn map_get_uint32_t_ptr_t(
    mut map: *mut Map_uint32_t_ptr_t,
    mut key: uint32_t,
) -> ptr_t {
    let mut k: uint32_t = mh_get_uint32_t(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_ptr_t.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const LOGLVL_DBG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const DEFAULT_GRID_HANDLE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const MOUSE_NORMAL: ::core::ffi::c_int = 'n' as ::core::ffi::c_int;
pub const MOUSE_VISUAL: ::core::ffi::c_int = 'v' as ::core::ffi::c_int;
pub const MOUSE_INSERT: ::core::ffi::c_int = 'i' as ::core::ffi::c_int;
pub const MOUSE_COMMAND: ::core::ffi::c_int = 'c' as ::core::ffi::c_int;
pub const MOUSE_HELP: ::core::ffi::c_int = 104;
pub const MOUSE_RETURN: ::core::ffi::c_int = 'r' as ::core::ffi::c_int;
pub const MOUSE_A: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"nvich\0") };
pub const MAX_UI_COUNT: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
static uis: GlobalCell<[*mut RemoteUI; 16]> =
    GlobalCell::new([::core::ptr::null_mut::<RemoteUI>(); 16]);
static ui_ext: GlobalCell<[bool; 10]> = GlobalCell::new([
    false, false, false, false, false, false, false, false, false, false,
]);
static ui_count: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
static ui_mode_idx: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(SHAPE_IDX_N as ::core::ffi::c_int);
static cursor_col: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static cursor_row: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static pending_cursor_update: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static busy: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static pending_mode_info_update: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static pending_mode_update: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static cursor_grid_handle: GlobalCell<handle_T> = GlobalCell::new(DEFAULT_GRID_HANDLE);
static ui_event_cbs: GlobalCell<Map_uint32_t_ptr_t> = GlobalCell::new(MAP_INIT);
pub static ui_cb_ext: GlobalCell<[bool; 10]> = GlobalCell::new([false; 10]);
static has_mouse: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static pending_has_mouse: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(-1 as ::core::ffi::c_int);
static pending_default_colors: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static uilog_seen: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
static uilog_last_event: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null::<::core::ffi::c_char>());
unsafe extern "C" fn ui_log(mut funname: *const ::core::ffi::c_char) {
    if uilog_last_event.get() == funname {
        uilog_seen.set((*uilog_seen.ptr()).wrapping_add(1));
    } else {
        if uilog_seen.get() > 0 as size_t {
            logmsg(
                LOGLVL_DBG,
                b"UI: \0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<::core::ffi::c_char>(),
                -1 as ::core::ffi::c_int,
                true_0 != 0,
                b"%s (+%zu times...)\0".as_ptr() as *const ::core::ffi::c_char,
                uilog_last_event.get(),
                uilog_seen.get(),
            );
        }
        logmsg(
            LOGLVL_DBG,
            b"UI: \0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ::core::ffi::c_int,
            true_0 != 0,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            funname,
        );
        uilog_seen.set(0 as size_t);
        uilog_last_event.set(funname);
    };
}
pub unsafe extern "C" fn ui_init() {
    (*default_grid.ptr()).handle = 1 as ::core::ffi::c_int as handle_T;
    (*msg_grid_adj.ptr()).target = default_grid.ptr();
    ui_comp_init();
}
pub unsafe extern "C" fn ui_rgb_attached() -> bool {
    if p_tgc.get() != 0 {
        return true_0 != 0;
    }
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut tui: bool = (*(*uis.ptr())[i as usize]).stdin_tty as ::core::ffi::c_int != 0
            || (*(*uis.ptr())[i as usize]).stdout_tty as ::core::ffi::c_int != 0;
        if !tui && (*(*uis.ptr())[i as usize]).rgb as ::core::ffi::c_int != 0 {
            return true_0 != 0;
        }
        i = i.wrapping_add(1);
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn ui_gui_attached() -> bool {
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut tui: bool = (*(*uis.ptr())[i as usize]).stdin_tty as ::core::ffi::c_int != 0
            || (*(*uis.ptr())[i as usize]).stdout_tty as ::core::ffi::c_int != 0;
        if !tui {
            return true_0 != 0;
        }
        i = i.wrapping_add(1);
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn ui_override() -> bool {
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        if (*(*uis.ptr())[i as usize]).override_0 {
            return true_0 != 0;
        }
        i = i.wrapping_add(1);
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn ui_active() -> size_t {
    return ui_count.get();
}
pub unsafe extern "C" fn ui_refresh() {
    if ui_client_channel_id.get() != 0 {
        abort();
    }
    let mut width: ::core::ffi::c_int = INT_MAX;
    let mut height: ::core::ffi::c_int = INT_MAX;
    let mut ext_widgets: [bool; 10] = [false; 10];
    let mut inclusive: bool = ui_override();
    memset(
        &raw mut ext_widgets as *mut bool as *mut ::core::ffi::c_void,
        (ui_active() != 0) as ::core::ffi::c_int,
        ::core::mem::size_of::<[bool; 10]>()
            .wrapping_div(::core::mem::size_of::<bool>())
            .wrapping_div(
                (::core::mem::size_of::<[bool; 10]>().wrapping_rem(::core::mem::size_of::<bool>())
                    == 0) as ::core::ffi::c_int as size_t,
            ),
    );
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        width = if (*ui).width < width {
            (*ui).width
        } else {
            width
        };
        height = if (*ui).height < height {
            (*ui).height
        } else {
            height
        };
        let mut j: UIExtension = kUICmdline;
        while (j as ::core::ffi::c_int) < kUIExtCount as ::core::ffi::c_int {
            ext_widgets[j as usize] = ext_widgets[j as usize] as ::core::ffi::c_int
                & ((*ui).ui_ext[j as usize] as ::core::ffi::c_int != 0
                    || inclusive as ::core::ffi::c_int != 0)
                    as ::core::ffi::c_int
                != 0;
            j += 1;
        }
        i = i.wrapping_add(1);
    }
    cursor_col.set(0 as ::core::ffi::c_int);
    cursor_row.set(cursor_col.get());
    pending_cursor_update.set(true_0 != 0);
    let mut had_message: bool = (*ui_ext.ptr())[kUIMessages as ::core::ffi::c_int as usize];
    let mut i_0: UIExtension = kUICmdline;
    while (i_0 as ::core::ffi::c_int) < kUIExtCount as ::core::ffi::c_int {
        (*ui_ext.ptr())[i_0 as usize] = ext_widgets[i_0 as usize] as ::core::ffi::c_int
            | (*ui_cb_ext.ptr())[i_0 as usize] as ::core::ffi::c_int
            != 0;
        if (i_0 as ::core::ffi::c_uint) < kUILinegrid as ::core::ffi::c_int as ::core::ffi::c_uint {
            ui_call_option_set(
                cstr_as_string(
                    *(ui_ext_names.ptr() as *mut *const ::core::ffi::c_char).offset(i_0 as isize),
                ),
                object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed_12 {
                        boolean: (*ui_ext.ptr())[i_0 as usize],
                    },
                },
            );
        }
        i_0 += 1;
    }
    if had_message as ::core::ffi::c_int
        != (*ui_ext.ptr())[kUIMessages as ::core::ffi::c_int as usize] as ::core::ffi::c_int
    {
        if ui_refresh_cmdheight.get() {
            set_option_value(
                kOptCmdheight,
                OptVal {
                    type_0: kOptValTypeNumber,
                    data: OptValData {
                        number: had_message as OptInt,
                    },
                },
                0 as ::core::ffi::c_int,
            );
            let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
            while !tp.is_null() {
                (*tp).tp_ch_used = had_message as OptInt;
                tp = (*tp).tp_next as *mut tabpage_T;
            }
        }
        msg_scroll_flush();
    }
    msg_ui_refresh();
    if ui_active() == 0 {
        return;
    }
    if updating_screen.get() {
        ui_schedule_refresh();
        return;
    }
    ui_default_colors_set();
    let mut save_p_lz: ::core::ffi::c_int = p_lz.get();
    p_lz.set(false_0);
    screen_resize(width, height);
    p_lz.set(save_p_lz);
    ui_mode_info_set();
    pending_mode_update.set(true_0 != 0);
    ui_cursor_shape();
    pending_has_mouse.set(-1 as ::core::ffi::c_int);
}
pub unsafe extern "C" fn ui_pum_get_height() -> ::core::ffi::c_int {
    let mut pum_height: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui_pum_height: ::core::ffi::c_int = (*(*uis.ptr())[i as usize]).pum_nlines;
        if ui_pum_height != 0 {
            pum_height = if pum_height != 0 as ::core::ffi::c_int {
                if pum_height < ui_pum_height {
                    pum_height
                } else {
                    ui_pum_height
                }
            } else {
                ui_pum_height
            };
        }
        i = i.wrapping_add(1);
    }
    return pum_height;
}
pub unsafe extern "C" fn ui_pum_get_pos(
    mut pwidth: *mut ::core::ffi::c_double,
    mut pheight: *mut ::core::ffi::c_double,
    mut prow: *mut ::core::ffi::c_double,
    mut pcol: *mut ::core::ffi::c_double,
) -> bool {
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        if !(*(*uis.ptr())[i as usize]).pum_pos {
            i = i.wrapping_add(1);
        } else {
            *pwidth = (*(*uis.ptr())[i as usize]).pum_width;
            *pheight = (*(*uis.ptr())[i as usize]).pum_height;
            *prow = (*(*uis.ptr())[i as usize]).pum_row;
            *pcol = (*(*uis.ptr())[i as usize]).pum_col;
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}
unsafe extern "C" fn ui_refresh_event(mut _argv: *mut *mut ::core::ffi::c_void) {
    ui_refresh();
}
pub unsafe extern "C" fn ui_schedule_refresh() {
    multiqueue_put_event(
        resize_events.get(),
        Event {
            handler: Some(
                ui_refresh_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
            ),
            argv: [
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
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
pub unsafe extern "C" fn ui_default_colors_set() {
    pending_default_colors.set(true_0 != 0);
    if starting.get() == 0 as ::core::ffi::c_int {
        ui_may_set_default_colors();
    }
}
unsafe extern "C" fn ui_may_set_default_colors() {
    if pending_default_colors.get() {
        pending_default_colors.set(false_0 != 0);
        ui_call_default_colors_set(
            normal_fg.get() as Integer,
            normal_bg.get() as Integer,
            normal_sp.get() as Integer,
            cterm_normal_fg_color.get() as Integer,
            cterm_normal_bg_color.get() as Integer,
        );
    }
}
pub unsafe extern "C" fn ui_busy_start() {
    let c2rust_fresh0 = busy.get();
    busy.set(busy.get() + 1);
    if c2rust_fresh0 == 0 {
        ui_call_busy_start();
    }
}
pub unsafe extern "C" fn ui_busy_stop() {
    (*busy.ptr()) -= 1;
    if busy.get() == 0 {
        ui_call_busy_stop();
    }
}
pub unsafe extern "C" fn vim_beep(mut val: ::core::ffi::c_uint) {
    called_vim_beep.set(true_0 != 0);
    if emsg_silent.get() != 0 as ::core::ffi::c_int
        || in_assert_fails.get() as ::core::ffi::c_int != 0
    {
        return;
    }
    if !(bo_flags.get() & val != 0
        || bo_flags.get() & kOptBoFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint != 0)
    {
        static beeps: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        static start_time: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
        if start_time.get() == 0 as uint64_t
            || os_hrtime().wrapping_sub(start_time.get()) > 500000000 as uint64_t
        {
            beeps.set(0 as ::core::ffi::c_int);
            start_time.set(os_hrtime());
        }
        (*beeps.ptr()) += 1;
        if beeps.get() <= 3 as ::core::ffi::c_int {
            if p_vb.get() != 0 {
                ui_call_visual_bell();
            } else {
                ui_call_bell();
            }
        }
    }
    if !vim_strchr(p_debug.get(), 'e' as ::core::ffi::c_int).is_null() {
        msg_source(HLF_W as ::core::ffi::c_int);
        msg(
            gettext(b"Beep!\0".as_ptr() as *const ::core::ffi::c_char),
            HLF_W as ::core::ffi::c_int,
        );
    }
}
pub unsafe extern "C" fn do_autocmd_uienter_all() {
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        do_autocmd_uienter((*(*uis.ptr())[i as usize]).channel_id, true_0 != 0);
        i = i.wrapping_add(1);
    }
}
pub unsafe extern "C" fn ui_can_attach_more() -> bool {
    return ui_count.get() < MAX_UI_COUNT as size_t;
}
pub unsafe extern "C" fn ui_attach_impl(mut ui: *mut RemoteUI, mut chanid: uint64_t) {
    if ui_count.get() >= MAX_UI_COUNT as size_t {
        abort();
    }
    if !(*ui).ui_ext[kUIMultigrid as ::core::ffi::c_int as usize]
        && !(*ui).ui_ext[kUIFloatDebug as ::core::ffi::c_int as usize]
        && ui_client_channel_id.get() == 0
    {
        ui_comp_attach(ui);
    }
    let c2rust_fresh1 = ui_count.get();
    ui_count.set((*ui_count.ptr()).wrapping_add(1));
    let c2rust_lvalue_ptr = &raw mut (*uis.ptr())[c2rust_fresh1 as usize];
    *c2rust_lvalue_ptr = ui;
    ui_refresh_options();
    resettitle();
    let mut cwd: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut cwdlen: size_t = ::core::mem::size_of::<[::core::ffi::c_char; 4096]>();
    if uv_cwd(&raw mut cwd as *mut ::core::ffi::c_char, &raw mut cwdlen) == 0 as ::core::ffi::c_int
    {
        ui_call_chdir(String_0 {
            data: &raw mut cwd as *mut ::core::ffi::c_char,
            size: cwdlen,
        });
    }
    let mut i: UIExtension = kUILinegrid;
    while (i as ::core::ffi::c_int) < kUIExtCount as ::core::ffi::c_int {
        ui_set_ext_option(ui, i, (*ui).ui_ext[i as usize]);
        i += 1;
    }
    let mut sent: bool = false_0 != 0;
    if (*ui).ui_ext[kUIHlState as ::core::ffi::c_int as usize] {
        sent = highlight_use_hlstate();
    }
    if !sent {
        ui_send_all_hls(ui);
    }
    ui_refresh();
    do_autocmd_uienter(chanid, true_0 != 0);
}
pub unsafe extern "C" fn ui_detach_impl(mut ui: *mut RemoteUI, mut chanid: uint64_t) {
    if ui_count.get() > MAX_UI_COUNT as size_t {
        abort();
    }
    let mut shift_index: size_t = MAX_UI_COUNT as size_t;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        if (*uis.ptr())[i as usize] == ui {
            shift_index = i;
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    if shift_index >= MAX_UI_COUNT as size_t {
        abort();
    }
    while shift_index < (*ui_count.ptr()).wrapping_sub(1 as size_t) {
        (*uis.ptr())[shift_index as usize] =
            (*uis.ptr())[shift_index.wrapping_add(1 as size_t) as usize];
        shift_index = shift_index.wrapping_add(1);
    }
    ui_count.set((*ui_count.ptr()).wrapping_sub(1));
    if ui_count.get() != 0 && !exiting.get() {
        ui_schedule_refresh();
    }
    if !(*ui).ui_ext[kUIMultigrid as ::core::ffi::c_int as usize]
        && !(*ui).ui_ext[kUIFloatDebug as ::core::ffi::c_int as usize]
    {
        ui_comp_detach(ui);
    }
    do_autocmd_uienter(chanid, false_0 != 0);
}
pub unsafe extern "C" fn ui_set_ext_option(
    mut ui: *mut RemoteUI,
    mut ext: UIExtension,
    mut active: bool,
) {
    if (ext as ::core::ffi::c_uint) < kUILinegrid as ::core::ffi::c_int as ::core::ffi::c_uint {
        ui_refresh();
        return;
    }
    if *(*(ui_ext_names.ptr() as *mut *const ::core::ffi::c_char).offset(ext as isize))
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        != '_' as ::core::ffi::c_int
        || active as ::core::ffi::c_int != 0
    {
        remote_ui_option_set(
            ui,
            cstr_as_string(
                *(ui_ext_names.ptr() as *mut *const ::core::ffi::c_char).offset(ext as isize),
            ),
            object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_12 { boolean: active },
            },
        );
    }
    if ext as ::core::ffi::c_uint == kUITermColors as ::core::ffi::c_int as ::core::ffi::c_uint {
        ui_default_colors_set();
    }
}
pub unsafe extern "C" fn ui_line(
    mut grid: *mut ScreenGrid,
    mut row: ::core::ffi::c_int,
    mut invalid_row: bool,
    mut startcol: ::core::ffi::c_int,
    mut endcol: ::core::ffi::c_int,
    mut clearcol: ::core::ffi::c_int,
    mut clearattr: ::core::ffi::c_int,
    mut wrap: bool,
) {
    '_c2rust_label: {
        if 0 as ::core::ffi::c_int <= row && row < (*grid).rows {
        } else {
            __assert_fail(
                b"0 <= row && row < grid->rows\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/ui.rs\0".as_ptr() as *const ::core::ffi::c_char,
                471 as ::core::ffi::c_uint,
                b"void ui_line(ScreenGrid *, int, _Bool, int, int, int, int, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut flags: LineFlags = if wrap as ::core::ffi::c_int != 0 {
        kLineFlagWrap as ::core::ffi::c_int
    } else {
        0 as LineFlags
    };
    if startcol == 0 as ::core::ffi::c_int && invalid_row as ::core::ffi::c_int != 0 {
        flags |= kLineFlagInvalid as ::core::ffi::c_int;
    }
    ui_may_set_default_colors();
    let mut off: size_t =
        (*(*grid).line_offset.offset(row as isize)).wrapping_add(startcol as size_t);
    ui_call_raw_line(
        (*grid).handle as Integer,
        row as Integer,
        startcol as Integer,
        endcol as Integer,
        clearcol as Integer,
        clearattr as Integer,
        flags,
        ((*grid).chars as *const schar_T).offset(off as isize),
        ((*grid).attrs as *const sattr_T).offset(off as isize),
    );
    if p_wd.get() != 0
        && rdb_flags.get() & kOptRdbFlagLine as ::core::ffi::c_int as ::core::ffi::c_uint != 0
    {
        ui_call_grid_cursor_goto(
            (*grid).handle as Integer,
            row as Integer,
            (if clearcol < (*grid).cols - 1 as ::core::ffi::c_int {
                clearcol
            } else {
                (*grid).cols - 1 as ::core::ffi::c_int
            }) as Integer,
        );
        ui_call_flush();
        let mut wd: uint64_t = llabs(p_wd.get() as ::core::ffi::c_longlong) as uint64_t;
        os_sleep(wd);
        pending_cursor_update.set(true_0 != 0);
    }
}
pub unsafe extern "C" fn ui_cursor_goto(
    mut new_row: ::core::ffi::c_int,
    mut new_col: ::core::ffi::c_int,
) {
    ui_grid_cursor_goto(DEFAULT_GRID_HANDLE, new_row, new_col);
}
pub unsafe extern "C" fn ui_grid_cursor_goto(
    mut grid_handle: handle_T,
    mut new_row: ::core::ffi::c_int,
    mut new_col: ::core::ffi::c_int,
) {
    if new_row == cursor_row.get()
        && new_col == cursor_col.get()
        && grid_handle == cursor_grid_handle.get()
    {
        return;
    }
    cursor_row.set(new_row);
    cursor_col.set(new_col);
    cursor_grid_handle.set(grid_handle);
    pending_cursor_update.set(true_0 != 0);
}
pub unsafe extern "C" fn ui_check_cursor_grid(mut grid_handle: handle_T) {
    if cursor_grid_handle.get() == grid_handle {
        pending_cursor_update.set(true_0 != 0);
    }
}
pub unsafe extern "C" fn ui_mode_info_set() {
    pending_mode_info_update.set(true_0 != 0);
}
pub unsafe extern "C" fn ui_current_row() -> ::core::ffi::c_int {
    return cursor_row.get();
}
pub unsafe extern "C" fn ui_current_col() -> ::core::ffi::c_int {
    return cursor_col.get();
}
pub unsafe extern "C" fn ui_flush() {
    '_c2rust_label: {
        if ui_client_channel_id.get() == 0 {
        } else {
            __assert_fail(
                b"!ui_client_channel_id\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/ui.rs\0".as_ptr() as *const ::core::ffi::c_char,
                542 as ::core::ffi::c_uint,
                b"void ui_flush(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if ui_active() == 0 {
        return;
    }
    static was_busy: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if State.get() & MODE_CMDLINE as ::core::ffi::c_int == 0
        && (*curwin.get()).w_floating as ::core::ffi::c_int != 0
        && (*curwin.get()).w_config.hide as ::core::ffi::c_int != 0
    {
        if !was_busy.get() {
            ui_call_busy_start();
            was_busy.set(true_0 != 0);
        }
    } else if was_busy.get() {
        ui_call_busy_stop();
        was_busy.set(false_0 != 0);
    }
    win_ui_flush(false_0 != 0);
    if textlock.get() == 0 as ::core::ffi::c_int && expr_map_lock.get() == 0 as ::core::ffi::c_int {
        cmdline_ui_flush();
        msg_ext_ui_flush();
    }
    msg_scroll_flush();
    if pending_cursor_update.get() {
        ui_call_grid_cursor_goto(
            cursor_grid_handle.get() as Integer,
            cursor_row.get() as Integer,
            cursor_col.get() as Integer,
        );
        pending_cursor_update.set(false_0 != 0);
        win_ui_flush(false_0 != 0);
    }
    if pending_mode_info_update.get() {
        let mut arena: Arena = ARENA_EMPTY;
        let mut style: Array = mode_style_array(&raw mut arena);
        let mut enabled: bool = *p_guicursor.get() as ::core::ffi::c_int != NUL;
        ui_call_mode_info_set(enabled as Boolean, style);
        arena_mem_free(arena_finish(&raw mut arena));
        pending_mode_info_update.set(false_0 != 0);
    }
    static cursor_was_obscured: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let mut cursor_obscured: bool = ui_cursor_is_behind_floatwin();
    if (cursor_obscured as ::core::ffi::c_int != cursor_was_obscured.get() as ::core::ffi::c_int
        || pending_mode_update.get() as ::core::ffi::c_int != 0)
        && starting.get() == 0
    {
        let mut idx: ::core::ffi::c_int = if cursor_obscured as ::core::ffi::c_int != 0 {
            SHAPE_IDX_R as ::core::ffi::c_int
        } else {
            ui_mode_idx.get()
        };
        let mut full_name: *mut ::core::ffi::c_char = (*shape_table.ptr())[idx as usize].full_name;
        ui_call_mode_change(cstr_as_string(full_name), idx as Integer);
        pending_mode_update.set(false_0 != 0);
        cursor_was_obscured.set(cursor_obscured);
    }
    if pending_has_mouse.get() != has_mouse.get() as ::core::ffi::c_int {
        if has_mouse.get() as ::core::ffi::c_int != 0 {
            Some(ui_call_mouse_on as unsafe extern "C" fn() -> ())
        } else {
            Some(ui_call_mouse_off as unsafe extern "C" fn() -> ())
        }
        .expect("non-null function pointer")();
        pending_has_mouse.set(has_mouse.get() as ::core::ffi::c_int);
    }
    ui_call_flush();
    if p_wd.get() != 0
        && rdb_flags.get() & kOptRdbFlagFlush as ::core::ffi::c_int as ::core::ffi::c_uint != 0
    {
        os_sleep(llabs(p_wd.get() as ::core::ffi::c_longlong) as uint64_t);
    }
}
pub unsafe extern "C" fn ui_check_mouse() {
    has_mouse.set(false_0 != 0);
    if *p_mouse.get() as ::core::ffi::c_int == NUL {
        return;
    }
    let mut checkfor: ::core::ffi::c_int = MOUSE_NORMAL;
    if VIsual_active.get() {
        checkfor = MOUSE_VISUAL;
    } else if State.get() == MODE_HITRETURN as ::core::ffi::c_int
        || State.get() == MODE_ASKMORE as ::core::ffi::c_int
        || State.get() == MODE_SETWSIZE as ::core::ffi::c_int
    {
        checkfor = MOUSE_RETURN;
    } else if State.get() & MODE_INSERT as ::core::ffi::c_int != 0 {
        checkfor = MOUSE_INSERT;
    } else if State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0 {
        checkfor = MOUSE_COMMAND;
    } else if State.get() == MODE_EXTERNCMD as ::core::ffi::c_int {
        checkfor = ' ' as ::core::ffi::c_int;
    }
    if ui_mouse_has(checkfor) {
        has_mouse.set(true_0 != 0);
    }
}
pub unsafe extern "C" fn ui_mouse_has(mut mode: ::core::ffi::c_int) -> bool {
    let mut p: *mut ::core::ffi::c_char = p_mouse.get();
    while *p != 0 {
        match *p as ::core::ffi::c_int {
            97 => {
                if !vim_strchr(MOUSE_A.as_ptr(), mode).is_null() {
                    return true_0 != 0;
                }
            }
            MOUSE_HELP => {
                if mode != MOUSE_RETURN && (*curbuf.get()).b_help as ::core::ffi::c_int != 0 {
                    return true_0 != 0;
                }
            }
            _ => {
                if mode == *p as ::core::ffi::c_int {
                    return true_0 != 0;
                }
            }
        }
        p = p.offset(1);
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn ui_cursor_shape_no_check_conceal() {
    if !full_screen.get() {
        return;
    }
    let mut new_mode_idx: ::core::ffi::c_int = cursor_get_mode_idx();
    if new_mode_idx != ui_mode_idx.get() {
        ui_mode_idx.set(new_mode_idx);
        pending_mode_update.set(true_0 != 0);
    }
}
pub unsafe extern "C" fn ui_cursor_shape() {
    ui_cursor_shape_no_check_conceal();
    conceal_check_cursor_line();
}
unsafe extern "C" fn ui_cursor_is_behind_floatwin() -> bool {
    if State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0 || !ui_comp_should_draw() {
        return false_0 != 0;
    }
    let mut crow: ::core::ffi::c_int =
        (*curwin.get()).w_winrow + (*curwin.get()).w_winrow_off + (*curwin.get()).w_wrow;
    let mut ccol: ::core::ffi::c_int = (*curwin.get()).w_wincol
        + (*curwin.get()).w_wincol_off
        + (if (*curwin.get()).w_onebuf_opt.wo_rl != 0 {
            (*curwin.get()).w_view_width - (*curwin.get()).w_wcol - 1 as ::core::ffi::c_int
        } else {
            (*curwin.get()).w_wcol
        });
    let mut top_grid: *mut ScreenGrid = ui_comp_get_grid_at_coord(crow, ccol);
    return top_grid != &raw mut (*curwin.get()).w_grid_alloc && top_grid != default_grid.ptr();
}
pub unsafe extern "C" fn ui_has(mut ext: UIExtension) -> bool {
    return (*ui_ext.ptr())[ext as usize];
}
pub unsafe extern "C" fn ui_array(mut arena: *mut Arena) -> Array {
    let mut all_uis: Array = arena_array(arena, ui_count.get());
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        let mut info: Dict = arena_dict(
            arena,
            (10 as ::core::ffi::c_int + kUIExtCount as ::core::ffi::c_int) as size_t,
        );
        let c2rust_fresh2 = info.size;
        info.size = info.size.wrapping_add(1);
        *info.items.offset(c2rust_fresh2 as isize) = key_value_pair {
            key: cstr_as_string(b"width\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: (*ui).width as Integer,
                },
            },
        };
        let c2rust_fresh3 = info.size;
        info.size = info.size.wrapping_add(1);
        *info.items.offset(c2rust_fresh3 as isize) = key_value_pair {
            key: cstr_as_string(b"height\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: (*ui).height as Integer,
                },
            },
        };
        let c2rust_fresh4 = info.size;
        info.size = info.size.wrapping_add(1);
        *info.items.offset(c2rust_fresh4 as isize) = key_value_pair {
            key: cstr_as_string(b"rgb\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_12 { boolean: (*ui).rgb },
            },
        };
        let c2rust_fresh5 = info.size;
        info.size = info.size.wrapping_add(1);
        *info.items.offset(c2rust_fresh5 as isize) = key_value_pair {
            key: cstr_as_string(b"override\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_12 {
                    boolean: (*ui).override_0,
                },
            },
        };
        let c2rust_fresh6 = info.size;
        info.size = info.size.wrapping_add(1);
        *info.items.offset(c2rust_fresh6 as isize) = key_value_pair {
            key: cstr_as_string(b"term_name\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_12 {
                    string: cstr_as_string((*ui).term_name),
                },
            },
        };
        let c2rust_fresh7 = info.size;
        info.size = info.size.wrapping_add(1);
        *info.items.offset(c2rust_fresh7 as isize) = key_value_pair {
            key: cstr_as_string(b"term_background\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_12 {
                    string: String_0 {
                        data: b"\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 1]>()
                            .wrapping_sub(1 as size_t),
                    },
                },
            },
        };
        let c2rust_fresh8 = info.size;
        info.size = info.size.wrapping_add(1);
        *info.items.offset(c2rust_fresh8 as isize) = key_value_pair {
            key: cstr_as_string(b"term_colors\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: (*ui).term_colors as Integer,
                },
            },
        };
        let c2rust_fresh9 = info.size;
        info.size = info.size.wrapping_add(1);
        *info.items.offset(c2rust_fresh9 as isize) = key_value_pair {
            key: cstr_as_string(b"stdin_tty\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_12 {
                    boolean: (*ui).stdin_tty,
                },
            },
        };
        let c2rust_fresh10 = info.size;
        info.size = info.size.wrapping_add(1);
        *info.items.offset(c2rust_fresh10 as isize) = key_value_pair {
            key: cstr_as_string(b"stdout_tty\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_12 {
                    boolean: (*ui).stdout_tty,
                },
            },
        };
        let mut j: UIExtension = kUICmdline;
        while (j as ::core::ffi::c_uint) < kUIExtCount as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if *(*(ui_ext_names.ptr() as *mut *const ::core::ffi::c_char).offset(j as isize))
                .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '_' as ::core::ffi::c_int
                || (*ui).ui_ext[j as usize] as ::core::ffi::c_int != 0
            {
                let c2rust_fresh11 = info.size;
                info.size = info.size.wrapping_add(1);
                *info.items.offset(c2rust_fresh11 as isize) = key_value_pair {
                    key: cstr_as_string(
                        *(ui_ext_names.ptr() as *mut *const ::core::ffi::c_char).offset(j as isize)
                            as *mut ::core::ffi::c_char,
                    ),
                    value: object {
                        type_0: kObjectTypeBoolean,
                        data: C2Rust_Unnamed_12 {
                            boolean: (*ui).ui_ext[j as usize],
                        },
                    },
                };
            }
            j += 1;
        }
        let c2rust_fresh12 = info.size;
        info.size = info.size.wrapping_add(1);
        *info.items.offset(c2rust_fresh12 as isize) = key_value_pair {
            key: cstr_as_string(b"chan\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: (*ui).channel_id as Integer,
                },
            },
        };
        let c2rust_fresh13 = all_uis.size;
        all_uis.size = all_uis.size.wrapping_add(1);
        *all_uis.items.offset(c2rust_fresh13 as isize) = object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed_12 { dict: info },
        };
        i = i.wrapping_add(1);
    }
    return all_uis;
}
pub unsafe extern "C" fn ui_grid_resize(
    mut grid_handle: handle_T,
    mut width: ::core::ffi::c_int,
    mut height: ::core::ffi::c_int,
    mut err: *mut Error,
) {
    if grid_handle == DEFAULT_GRID_HANDLE {
        screen_resize(width, height);
        return;
    }
    let mut wp: *mut win_T = get_win_by_grid_handle(grid_handle);
    if wp.is_null() {
        api_err_invalid(
            err,
            b"window handle\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            grid_handle as int64_t,
            false_0 != 0,
        );
        return;
    }
    if (*wp).w_floating {
        if width != (*wp).w_width || height != (*wp).w_height {
            (*wp).w_config.width = if width > 1 as ::core::ffi::c_int {
                width
            } else {
                1 as ::core::ffi::c_int
            };
            (*wp).w_config.height = if height > 1 as ::core::ffi::c_int {
                height
            } else {
                1 as ::core::ffi::c_int
            };
            win_config_float(wp, (*wp).w_config);
        }
    } else {
        (*wp).w_height_request = if height > 0 as ::core::ffi::c_int {
            height
        } else {
            0 as ::core::ffi::c_int
        };
        (*wp).w_width_request = if width > 0 as ::core::ffi::c_int {
            width
        } else {
            0 as ::core::ffi::c_int
        };
        win_set_inner_size(wp, true_0 != 0);
    };
}
unsafe extern "C" fn ui_attach_error(
    mut ns_id: uint32_t,
    mut name: *const ::core::ffi::c_char,
    mut msg_0: *const ::core::ffi::c_char,
) {
    let mut ns: *const ::core::ffi::c_char = describe_ns(
        ns_id as NS,
        b"(UNKNOWN PLUGIN)\0".as_ptr() as *const ::core::ffi::c_char,
    );
    logmsg(
        LOGLVL_ERR,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"ui_attach_error\0".as_ptr() as *const ::core::ffi::c_char,
        783 as ::core::ffi::c_int,
        true_0 != 0,
        b"Error in \"%s\" UI event handler (ns=%s):\n%s\0".as_ptr() as *const ::core::ffi::c_char,
        name,
        ns,
        msg_0,
    );
    msg_schedule_semsg_multiline(
        b"Error in \"%s\" UI event handler (ns=%s):\n%s\0".as_ptr() as *const ::core::ffi::c_char,
        name,
        ns,
        msg_0,
    );
}
pub unsafe extern "C" fn ui_call_event(mut name: *mut ::core::ffi::c_char, mut args: Array) {
    let mut fast: bool = strcmp(name, b"msg_show\0".as_ptr() as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int;
    let mut not_fast: [*const ::core::ffi::c_char; 13] = [
        b"empty\0".as_ptr() as *const ::core::ffi::c_char,
        b"echo\0".as_ptr() as *const ::core::ffi::c_char,
        b"echomsg\0".as_ptr() as *const ::core::ffi::c_char,
        b"echoerr\0".as_ptr() as *const ::core::ffi::c_char,
        b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char,
        b"lua_error\0".as_ptr() as *const ::core::ffi::c_char,
        b"lua_print\0".as_ptr() as *const ::core::ffi::c_char,
        b"progress\0".as_ptr() as *const ::core::ffi::c_char,
        b"shell_cmd\0".as_ptr() as *const ::core::ffi::c_char,
        b"shell_err\0".as_ptr() as *const ::core::ffi::c_char,
        b"shell_out\0".as_ptr() as *const ::core::ffi::c_char,
        b"shell_ret\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
    ];
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while fast as ::core::ffi::c_int != 0 && !not_fast[i as usize].is_null() {
        fast = !strequal(
            not_fast[i as usize],
            (*args.items.offset(0 as ::core::ffi::c_int as isize))
                .data
                .string
                .data,
        );
        i += 1;
    }
    let mut save_expr_map_lock: ::core::ffi::c_int = expr_map_lock.get();
    let mut save_textlock: ::core::ffi::c_int = textlock.get();
    expr_map_lock.set(0 as ::core::ffi::c_int);
    textlock.set(0 as ::core::ffi::c_int);
    let mut handled: bool = false_0 != 0;
    let mut event_cb: *mut UIEventCallback = ::core::ptr::null_mut::<UIEventCallback>();
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*ui_event_cbs.ptr()).set.h.n_keys {
        ui_event_ns_id.set(*(*ui_event_cbs.ptr()).set.keys.offset(__i as isize));
        event_cb = *(*ui_event_cbs.ptr()).values.offset(__i as isize) as *mut UIEventCallback;
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut ns_id: uint32_t = ui_event_ns_id.get();
        let mut res: Object = nlua_call_ref_ctx(
            fast,
            (*event_cb).cb,
            name,
            args,
            kRetNilBool,
            ::core::ptr::null_mut::<Arena>(),
            &raw mut err,
        );
        ui_event_ns_id.set(0 as uint32_t);
        if res.type_0 as ::core::ffi::c_uint
            == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
            && res.data.boolean as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        {
            handled = true;
        }
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            ui_attach_error(ns_id, name, err.msg);
            ui_remove_cb(ns_id, true);
        }
        api_clear_error(&raw mut err);
        __i = __i.wrapping_add(1);
    }
    expr_map_lock.set(save_expr_map_lock);
    textlock.set(save_textlock);
    if !handled {
        let mut any_call: bool = false_0 != 0;
        let mut i_0: size_t = 0 as size_t;
        while i_0 < ui_count.get() {
            let mut ui: *mut RemoteUI = (*uis.ptr())[i_0 as usize];
            remote_ui_event(ui, name, args);
            any_call = true_0 != 0;
            i_0 = i_0.wrapping_add(1);
        }
        if any_call {
            ui_log(b"event\0".as_ptr() as *const ::core::ffi::c_char);
        }
    }
    ui_log(name);
}
unsafe extern "C" fn ui_cb_update_ext() {
    memset(
        ui_cb_ext.ptr() as *mut bool as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[bool; 10]>()
            .wrapping_div(::core::mem::size_of::<bool>())
            .wrapping_div(
                (::core::mem::size_of::<[bool; 10]>().wrapping_rem(::core::mem::size_of::<bool>())
                    == 0) as ::core::ffi::c_int as size_t,
            ),
    );
    let mut i: size_t = 0 as size_t;
    while i < kUILinegrid as ::core::ffi::c_int as size_t {
        let mut event_cb: *mut UIEventCallback = ::core::ptr::null_mut::<UIEventCallback>();
        let mut __i: uint32_t = 0;
        __i = 0 as uint32_t;
        while __i < (*ui_event_cbs.ptr()).set.h.n_keys {
            event_cb = *(*ui_event_cbs.ptr()).values.offset(__i as isize) as *mut UIEventCallback;
            if (*event_cb).ext_widgets[i as usize] {
                (*ui_cb_ext.ptr())[i as usize] = true;
                break;
            } else {
                __i = __i.wrapping_add(1);
            }
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn free_ui_event_callback(mut event_cb: *mut UIEventCallback) {
    api_free_luaref((*event_cb).cb);
    xfree(event_cb as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn ui_add_cb(
    mut ns_id: uint32_t,
    mut cb: LuaRef,
    mut ext_widgets: *mut bool,
) {
    let mut event_cb: *mut UIEventCallback =
        xcalloc(1 as size_t, ::core::mem::size_of::<UIEventCallback>()) as *mut UIEventCallback;
    (*event_cb).cb = cb;
    memcpy(
        &raw mut (*event_cb).ext_widgets as *mut bool as *mut ::core::ffi::c_void,
        ext_widgets as *const ::core::ffi::c_void,
        ::core::mem::size_of::<[bool; 5]>()
            .wrapping_div(::core::mem::size_of::<bool>())
            .wrapping_div(
                (::core::mem::size_of::<[bool; 5]>().wrapping_rem(::core::mem::size_of::<bool>())
                    == 0) as ::core::ffi::c_int as size_t,
            ),
    );
    if (*event_cb).ext_widgets[kUIMessages as ::core::ffi::c_int as usize] {
        (*event_cb).ext_widgets[kUICmdline as ::core::ffi::c_int as usize] = true_0 != 0;
    }
    let mut item: *mut ptr_t = map_put_ref_uint32_t_ptr_t(
        ui_event_cbs.ptr(),
        ns_id,
        ::core::ptr::null_mut::<*mut uint32_t>(),
        ::core::ptr::null_mut::<bool>(),
    );
    if !(*item).is_null() {
        free_ui_event_callback(*item as *mut UIEventCallback);
    }
    *item = event_cb as ptr_t;
    ui_cb_update_ext();
    ui_refresh();
}
pub unsafe extern "C" fn ui_remove_cb(mut ns_id: uint32_t, mut checkerr: bool) {
    let mut item: *mut UIEventCallback =
        map_get_uint32_t_ptr_t(ui_event_cbs.ptr(), ns_id) as *mut UIEventCallback;
    if !item.is_null()
        && (!checkerr || {
            (*item).errors = (*item).errors.wrapping_add(1);
            (*item).errors as ::core::ffi::c_int > CB_MAX_ERROR as ::core::ffi::c_int
        })
    {
        map_del_uint32_t_ptr_t(
            ui_event_cbs.ptr(),
            ns_id,
            ::core::ptr::null_mut::<uint32_t>(),
        );
        free_ui_event_callback(item);
        ui_cb_update_ext();
        ui_refresh();
        if checkerr {
            let mut ns: *const ::core::ffi::c_char = describe_ns(
                ns_id as NS,
                b"(UNKNOWN PLUGIN)\0".as_ptr() as *const ::core::ffi::c_char,
            );
            msg_schedule_semsg(
                b"Excessive errors in vim.ui_attach() callback (ns=%s)\0".as_ptr()
                    as *const ::core::ffi::c_char,
                ns,
            );
        }
    }
}
pub unsafe extern "C" fn ui_call_mode_info_set(mut enabled: Boolean, mut cursor_styles: Array) {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_mode_info_set(ui, enabled, cursor_styles);
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"mode_info_set\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_update_menu() {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_update_menu(ui);
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"update_menu\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_busy_start() {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_busy_start(ui);
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"busy_start\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_busy_stop() {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_busy_stop(ui);
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"busy_stop\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_mouse_on() {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_mouse_on(ui);
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"mouse_on\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_mouse_off() {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_mouse_off(ui);
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"mouse_off\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_mode_change(mut mode: String_0, mut mode_idx: Integer) {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_mode_change(ui, mode, mode_idx);
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"mode_change\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_bell() {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_bell(ui);
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"bell\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_visual_bell() {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_visual_bell(ui);
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"visual_bell\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_flush() {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_flush(ui);
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"flush\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_restart(mut listen_addr: String_0) {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_12 { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh14 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh14 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_12 {
            string: listen_addr,
        },
    };
    ui_call_event(
        b"restart\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args,
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_suspend() {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_suspend(ui);
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"suspend\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ui_call_set_title(mut title: String_0) {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_set_title(ui, title);
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"set_title\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_set_icon(mut icon: String_0) {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_set_icon(ui, icon);
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"set_icon\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_screenshot(mut path: String_0) {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_screenshot(ui, path);
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"screenshot\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_option_set(mut name: String_0, mut value: Object) {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_option_set(ui, name, value);
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"option_set\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ui_call_chdir(mut path: String_0) {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_chdir(ui, path);
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"chdir\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_stop() {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_stop(ui);
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"stop\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_ui_send(mut content: String_0) {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_ui_send(ui, content);
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"ui_send\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_default_colors_set(
    mut rgb_fg: Integer,
    mut rgb_bg: Integer,
    mut rgb_sp: Integer,
    mut cterm_fg: Integer,
    mut cterm_bg: Integer,
) {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_default_colors_set(ui, rgb_fg, rgb_bg, rgb_sp, cterm_fg, cterm_bg);
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"default_colors_set\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_hl_attr_define(
    mut id: Integer,
    mut rgb_attrs: HlAttrs,
    mut cterm_attrs: HlAttrs,
    mut info: Array,
) {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_hl_attr_define(ui, id, rgb_attrs, cterm_attrs, info);
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"hl_attr_define\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_hl_group_set(mut name: String_0, mut id: Integer) {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_hl_group_set(ui, name, id);
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"hl_group_set\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_grid_resize(
    mut grid: Integer,
    mut width: Integer,
    mut height: Integer,
) {
    ui_comp_grid_resize(grid, width, height);
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        if !(*ui).composed {
            remote_ui_grid_resize(ui, grid, width, height);
            any_call = true_0 != 0;
        }
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"grid_resize\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_composed_call_grid_resize(
    mut grid: Integer,
    mut width: Integer,
    mut height: Integer,
) {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        if (*ui).composed {
            remote_ui_grid_resize(ui, grid, width, height);
            any_call = true_0 != 0;
        }
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"grid_resize\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_grid_clear(mut grid: Integer) {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_grid_clear(ui, grid);
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"grid_clear\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_grid_cursor_goto(
    mut grid: Integer,
    mut row: Integer,
    mut col: Integer,
) {
    ui_comp_grid_cursor_goto(grid, row, col);
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        if !(*ui).composed {
            remote_ui_grid_cursor_goto(ui, grid, row, col);
            any_call = true_0 != 0;
        }
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"grid_cursor_goto\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_composed_call_grid_cursor_goto(
    mut grid: Integer,
    mut row: Integer,
    mut col: Integer,
) {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        if (*ui).composed {
            remote_ui_grid_cursor_goto(ui, grid, row, col);
            any_call = true_0 != 0;
        }
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"grid_cursor_goto\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_grid_scroll(
    mut grid: Integer,
    mut top: Integer,
    mut bot: Integer,
    mut left: Integer,
    mut right: Integer,
    mut rows: Integer,
    mut cols: Integer,
) {
    ui_comp_grid_scroll(grid, top, bot, left, right, rows, cols);
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        if !(*ui).composed {
            remote_ui_grid_scroll(ui, grid, top, bot, left, right, rows, cols);
            any_call = true_0 != 0;
        }
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"grid_scroll\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_composed_call_grid_scroll(
    mut grid: Integer,
    mut top: Integer,
    mut bot: Integer,
    mut left: Integer,
    mut right: Integer,
    mut rows: Integer,
    mut cols: Integer,
) {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        if (*ui).composed {
            remote_ui_grid_scroll(ui, grid, top, bot, left, right, rows, cols);
            any_call = true_0 != 0;
        }
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"grid_scroll\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_grid_destroy(mut grid: Integer) {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_12 { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh33 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh33 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: grid },
    };
    ui_call_event(
        b"grid_destroy\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args,
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_raw_line(
    mut grid: Integer,
    mut row: Integer,
    mut startcol: Integer,
    mut endcol: Integer,
    mut clearcol: Integer,
    mut clearattr: Integer,
    mut flags: LineFlags,
    mut chunk: *const schar_T,
    mut attrs: *const sattr_T,
) {
    ui_comp_raw_line(
        grid, row, startcol, endcol, clearcol, clearattr, flags, chunk, attrs,
    );
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        if !(*ui).composed {
            remote_ui_raw_line(
                ui, grid, row, startcol, endcol, clearcol, clearattr, flags, chunk, attrs,
            );
            any_call = true_0 != 0;
        }
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"raw_line\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_composed_call_raw_line(
    mut grid: Integer,
    mut row: Integer,
    mut startcol: Integer,
    mut endcol: Integer,
    mut clearcol: Integer,
    mut clearattr: Integer,
    mut flags: LineFlags,
    mut chunk: *const schar_T,
    mut attrs: *const sattr_T,
) {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        if (*ui).composed {
            remote_ui_raw_line(
                ui, grid, row, startcol, endcol, clearcol, clearattr, flags, chunk, attrs,
            );
            any_call = true_0 != 0;
        }
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"raw_line\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_win_pos(
    mut grid: Integer,
    mut win: Window,
    mut startrow: Integer,
    mut startcol: Integer,
    mut width: Integer,
    mut height: Integer,
) {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 6] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_12 { boolean: false },
    }; 6];
    args.capacity = 6 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh34 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh34 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: grid },
    };
    let c2rust_fresh35 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh35 as isize) = object {
        type_0: kObjectTypeWindow,
        data: C2Rust_Unnamed_12 {
            integer: win as Integer,
        },
    };
    let c2rust_fresh36 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh36 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: startrow },
    };
    let c2rust_fresh37 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh37 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: startcol },
    };
    let c2rust_fresh38 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh38 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: width },
    };
    let c2rust_fresh39 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh39 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: height },
    };
    ui_call_event(
        b"win_pos\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args,
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_win_float_pos(
    mut grid: Integer,
    mut win: Window,
    mut anchor: String_0,
    mut anchor_grid: Integer,
    mut anchor_row: Float,
    mut anchor_col: Float,
    mut mouse_enabled: Boolean,
    mut zindex: Integer,
    mut compindex: Integer,
    mut screen_row: Integer,
    mut screen_col: Integer,
) {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 11] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_12 { boolean: false },
    }; 11];
    args.capacity = 11 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh40 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh40 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: grid },
    };
    let c2rust_fresh41 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh41 as isize) = object {
        type_0: kObjectTypeWindow,
        data: C2Rust_Unnamed_12 {
            integer: win as Integer,
        },
    };
    let c2rust_fresh42 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh42 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_12 { string: anchor },
    };
    let c2rust_fresh43 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh43 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 {
            integer: anchor_grid,
        },
    };
    let c2rust_fresh44 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh44 as isize) = object {
        type_0: kObjectTypeFloat,
        data: C2Rust_Unnamed_12 {
            floating: anchor_row,
        },
    };
    let c2rust_fresh45 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh45 as isize) = object {
        type_0: kObjectTypeFloat,
        data: C2Rust_Unnamed_12 {
            floating: anchor_col,
        },
    };
    let c2rust_fresh46 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh46 as isize) = object {
        type_0: kObjectTypeBoolean,
        data: C2Rust_Unnamed_12 {
            boolean: mouse_enabled,
        },
    };
    let c2rust_fresh47 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh47 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: zindex },
    };
    let c2rust_fresh48 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh48 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: compindex },
    };
    let c2rust_fresh49 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh49 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 {
            integer: screen_row,
        },
    };
    let c2rust_fresh50 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh50 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 {
            integer: screen_col,
        },
    };
    ui_call_event(
        b"win_float_pos\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args,
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_win_external_pos(mut grid: Integer, mut win: Window) {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 2] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_12 { boolean: false },
    }; 2];
    args.capacity = 2 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh51 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh51 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: grid },
    };
    let c2rust_fresh52 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh52 as isize) = object {
        type_0: kObjectTypeWindow,
        data: C2Rust_Unnamed_12 {
            integer: win as Integer,
        },
    };
    ui_call_event(
        b"win_external_pos\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args,
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_win_hide(mut grid: Integer) {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_12 { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh53 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh53 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: grid },
    };
    ui_call_event(
        b"win_hide\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args,
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_win_close(mut grid: Integer) {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_12 { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh54 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh54 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: grid },
    };
    ui_call_event(
        b"win_close\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args,
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_msg_set_pos(
    mut grid: Integer,
    mut row: Integer,
    mut scrolled: Boolean,
    mut sep_char: String_0,
    mut zindex: Integer,
    mut compindex: Integer,
) {
    ui_comp_msg_set_pos(grid, row, scrolled, sep_char, zindex, compindex);
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        if !(*ui).composed {
            remote_ui_msg_set_pos(ui, grid, row, scrolled, sep_char, zindex, compindex);
            any_call = true_0 != 0;
        }
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"msg_set_pos\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_win_viewport(
    mut grid: Integer,
    mut win: Window,
    mut topline: Integer,
    mut botline: Integer,
    mut curline: Integer,
    mut curcol: Integer,
    mut line_count: Integer,
    mut scroll_delta: Integer,
) {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_win_viewport(
            ui,
            grid,
            win,
            topline,
            botline,
            curline,
            curcol,
            line_count,
            scroll_delta,
        );
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"win_viewport\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_win_viewport_margins(
    mut grid: Integer,
    mut win: Window,
    mut top: Integer,
    mut bottom: Integer,
    mut left: Integer,
    mut right: Integer,
) {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_win_viewport_margins(ui, grid, win, top, bottom, left, right);
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"win_viewport_margins\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ui_call_win_extmark(
    mut grid: Integer,
    mut win: Window,
    mut ns_id: Integer,
    mut mark_id: Integer,
    mut row: Integer,
    mut col: Integer,
) {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 6] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_12 { boolean: false },
    }; 6];
    args.capacity = 6 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh55 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh55 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: grid },
    };
    let c2rust_fresh56 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh56 as isize) = object {
        type_0: kObjectTypeWindow,
        data: C2Rust_Unnamed_12 {
            integer: win as Integer,
        },
    };
    let c2rust_fresh57 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh57 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: ns_id },
    };
    let c2rust_fresh58 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh58 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: mark_id },
    };
    let c2rust_fresh59 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh59 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: row },
    };
    let c2rust_fresh60 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh60 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: col },
    };
    ui_call_event(
        b"win_extmark\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args,
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_popupmenu_show(
    mut items: Array,
    mut selected: Integer,
    mut row: Integer,
    mut col: Integer,
    mut grid: Integer,
) {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 5] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_12 { boolean: false },
    }; 5];
    args.capacity = 5 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh61 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh61 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed_12 { array: items },
    };
    let c2rust_fresh62 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh62 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: selected },
    };
    let c2rust_fresh63 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh63 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: row },
    };
    let c2rust_fresh64 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh64 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: col },
    };
    let c2rust_fresh65 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh65 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: grid },
    };
    ui_call_event(
        b"popupmenu_show\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args,
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_popupmenu_hide() {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    ui_call_event(
        b"popupmenu_hide\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        noargs.get(),
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_popupmenu_select(mut selected: Integer) {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_12 { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh66 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh66 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: selected },
    };
    ui_call_event(
        b"popupmenu_select\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args,
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_tabline_update(
    mut current: Tabpage,
    mut tabs: Array,
    mut current_buffer: Buffer,
    mut buffers: Array,
) {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 4] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_12 { boolean: false },
    }; 4];
    args.capacity = 4 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh67 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh67 as isize) = object {
        type_0: kObjectTypeTabpage,
        data: C2Rust_Unnamed_12 {
            integer: current as Integer,
        },
    };
    let c2rust_fresh68 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh68 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed_12 { array: tabs },
    };
    let c2rust_fresh69 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh69 as isize) = object {
        type_0: kObjectTypeBuffer,
        data: C2Rust_Unnamed_12 {
            integer: current_buffer as Integer,
        },
    };
    let c2rust_fresh70 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh70 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed_12 { array: buffers },
    };
    ui_call_event(
        b"tabline_update\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args,
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_cmdline_show(
    mut content: Array,
    mut pos: Integer,
    mut firstc: String_0,
    mut prompt: String_0,
    mut indent: Integer,
    mut level: Integer,
    mut hl_id: Integer,
) {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 7] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_12 { boolean: false },
    }; 7];
    args.capacity = 7 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh71 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh71 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed_12 { array: content },
    };
    let c2rust_fresh72 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh72 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: pos },
    };
    let c2rust_fresh73 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh73 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_12 { string: firstc },
    };
    let c2rust_fresh74 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh74 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_12 { string: prompt },
    };
    let c2rust_fresh75 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh75 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: indent },
    };
    let c2rust_fresh76 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh76 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: level },
    };
    let c2rust_fresh77 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh77 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: hl_id },
    };
    ui_call_event(
        b"cmdline_show\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args,
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_cmdline_pos(mut pos: Integer, mut level: Integer) {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 2] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_12 { boolean: false },
    }; 2];
    args.capacity = 2 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh78 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh78 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: pos },
    };
    let c2rust_fresh79 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh79 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: level },
    };
    ui_call_event(
        b"cmdline_pos\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args,
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_cmdline_special_char(
    mut c: String_0,
    mut shift: Boolean,
    mut level: Integer,
) {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 3] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_12 { boolean: false },
    }; 3];
    args.capacity = 3 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh80 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh80 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_12 { string: c },
    };
    let c2rust_fresh81 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh81 as isize) = object {
        type_0: kObjectTypeBoolean,
        data: C2Rust_Unnamed_12 { boolean: shift },
    };
    let c2rust_fresh82 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh82 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: level },
    };
    ui_call_event(
        b"cmdline_special_char\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        args,
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_cmdline_hide(mut level: Integer, mut abort_0: Boolean) {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 2] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_12 { boolean: false },
    }; 2];
    args.capacity = 2 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh83 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh83 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_12 { integer: level },
    };
    let c2rust_fresh84 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh84 as isize) = object {
        type_0: kObjectTypeBoolean,
        data: C2Rust_Unnamed_12 { boolean: abort_0 },
    };
    ui_call_event(
        b"cmdline_hide\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args,
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_cmdline_block_show(mut lines: Array) {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_12 { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh85 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh85 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed_12 { array: lines },
    };
    ui_call_event(
        b"cmdline_block_show\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args,
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_cmdline_block_append(mut lines: Array) {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_12 { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh86 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh86 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed_12 { array: lines },
    };
    ui_call_event(
        b"cmdline_block_append\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        args,
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_cmdline_block_hide() {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    ui_call_event(
        b"cmdline_block_hide\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        noargs.get(),
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_msg_show(
    mut kind: String_0,
    mut content: Array,
    mut replace_last: Boolean,
    mut history: Boolean,
    mut append: Boolean,
    mut id: Object,
    mut trigger: String_0,
) {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 7] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_12 { boolean: false },
    }; 7];
    args.capacity = 7 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh89 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh89 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_12 { string: kind },
    };
    let c2rust_fresh90 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh90 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed_12 { array: content },
    };
    let c2rust_fresh91 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh91 as isize) = object {
        type_0: kObjectTypeBoolean,
        data: C2Rust_Unnamed_12 {
            boolean: replace_last,
        },
    };
    let c2rust_fresh92 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh92 as isize) = object {
        type_0: kObjectTypeBoolean,
        data: C2Rust_Unnamed_12 { boolean: history },
    };
    let c2rust_fresh93 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh93 as isize) = object {
        type_0: kObjectTypeBoolean,
        data: C2Rust_Unnamed_12 { boolean: append },
    };
    let c2rust_fresh94 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh94 as isize) = id;
    let c2rust_fresh95 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh95 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_12 { string: trigger },
    };
    ui_call_event(
        b"msg_show\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args,
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_msg_clear() {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    ui_call_event(
        b"msg_clear\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        noargs.get(),
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_msg_showcmd(mut content: Array) {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_12 { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh96 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh96 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed_12 { array: content },
    };
    ui_call_event(
        b"msg_showcmd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args,
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_msg_showmode(mut content: Array) {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_12 { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh97 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh97 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed_12 { array: content },
    };
    ui_call_event(
        b"msg_showmode\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args,
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_msg_ruler(mut content: Array) {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_12 { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh98 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh98 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed_12 { array: content },
    };
    ui_call_event(
        b"msg_ruler\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args,
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_msg_history_show(mut entries: Array, mut prev_cmd: Boolean) {
    static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if entered.get() {
        return;
    }
    entered.set(true_0 != 0);
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 2] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_12 { boolean: false },
    }; 2];
    args.capacity = 2 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh99 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh99 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed_12 { array: entries },
    };
    let c2rust_fresh100 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh100 as isize) = object {
        type_0: kObjectTypeBoolean,
        data: C2Rust_Unnamed_12 { boolean: prev_cmd },
    };
    ui_call_event(
        b"msg_history_show\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args,
    );
    entered.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_call_error_exit(mut status: Integer) {
    let mut any_call: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < ui_count.get() {
        let mut ui: *mut RemoteUI = (*uis.ptr())[i as usize];
        remote_ui_error_exit(ui, status);
        any_call = true_0 != 0;
        i = i.wrapping_add(1);
    }
    if any_call {
        ui_log(b"error_exit\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
