use crate::src::nvim::api::private::helpers::{cbuf_to_string, copy_string, cstr_as_string};
use crate::src::nvim::autocmd::{apply_autocmds, has_event};
use crate::src::nvim::buffer::{buf_spname, buf_valid};
use crate::src::nvim::change::{
    deleted_lines_mark, ins_bytes_len, ins_char, ins_char_bytes, ins_str, open_line,
};
use crate::src::nvim::charset::{
    getwhitecols, ptr2cells, skipwhite, str_foldcase, vim_isIDc, vim_isfilec, vim_isprintc,
    vim_iswordc, vim_iswordp, vim_strsize,
};
use crate::src::nvim::cmdexpand::{addstar, expand_cmdline, set_cmd_context};
use crate::src::nvim::cursor::{
    check_cursor, dec_cursor, get_cursor_line_len, get_cursor_line_ptr, get_cursor_pos_len,
    get_cursor_pos_ptr, inc_cursor,
};
use crate::src::nvim::drawscreen::{
    redrawWinline, redraw_later, setcursor, showmode, update_screen,
};
use crate::src::nvim::edit::{
    backspace_until_column, get_can_cindent, ins_apply_autocmds, ins_eol, ins_need_undo_get,
    ins_redraw, insertchar, start_arrow, stop_arrow,
};
use crate::src::nvim::eval::typval::{
    callback_copy, callback_free, tv_clear, tv_dict_add_bool, tv_dict_add_dict, tv_dict_add_list,
    tv_dict_add_nr, tv_dict_add_str, tv_dict_add_str_len, tv_dict_add_tv, tv_dict_alloc,
    tv_dict_alloc_lock, tv_dict_alloc_ret, tv_dict_find, tv_dict_get_number, tv_dict_get_string,
    tv_dict_get_tv, tv_dict_set_keys_readonly, tv_dict_unref, tv_get_number_chk, tv_get_string,
    tv_get_string_chk, tv_list_alloc, tv_list_append_dict, tv_list_unref,
};
use crate::src::nvim::eval::userfunc::callback_call_retnr;
use crate::src::nvim::eval::vars::set_vim_var_dict;
use crate::src::nvim::eval_1::{callback_call, get_v_event, restore_v_event, set_ref_in_callback};
use crate::src::nvim::ex_eval::aborting;
use crate::src::nvim::ex_getln::tilde_replace;
use crate::src::nvim::extmark::{extmark_apply_undo, extmark_splice_delete};
use crate::src::nvim::fileio::vim_fgets;
use crate::src::nvim::fuzzy::{fuzzy_match_str, fuzzy_match_str_in_line, search_for_fuzzy_match};
use crate::src::nvim::garray::{ga_clear, ga_grow, ga_init};
use crate::src::nvim::getchar::{
    char_avail, safe_vgetc, using_script, vgetc, vpeekc, vpeekc_any, vungetc, AppendCharToRedobuff,
    AppendToRedobuffLit,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight_group::syn_name2attr;
use crate::src::nvim::indent::{get_indent, inindent};
use crate::src::nvim::indent_c::{cindent_on, do_c_expr_indent, in_cinkeys};
use crate::src::nvim::lua::executor::nlua_expand_pat;
use crate::src::nvim::main::{
    arrow_used, can_si, can_si_back, cmdwin_type, cot_flags, curbuf, curwin, did_ai, did_emsg,
    did_si, dollar_vcol, e_invarg, e_list_index_out_of_range_nr, e_listreq, e_notset, e_patnotf,
    edit_submode, edit_submode_extra, edit_submode_highl, edit_submode_pre, emsg_off, emsg_silent,
    ex_normal_busy, firstbuf, firstwin, g_tag_at_cursor, global_busy, got_int, in_assert_fails,
    msg_hist_off, msg_silent, p_ac, p_acl, p_act, p_cto, p_dict, p_fic, p_ic, p_inf, p_js, p_paste,
    p_scs, p_smd, p_tsr, p_tsrfu, p_wic, p_ws, redraw_cmdline, redraw_mode, sc_col,
    test_disable_char_avail, textlock, IObuff, KeyTyped, RedrawingDisabled, State,
};
use crate::src::nvim::mbyte::{
    mb_get_class, mb_islower, mb_isupper, mb_prevptr, mb_ptr2char_adv, mb_tolower, mb_toupper,
    utf8len_tab, utf_char2bytes, utf_char2len, utf_head_off, utf_ptr2char, utf_ptr2len,
    utfc_ptr2len,
};
use crate::src::nvim::memline::{dec, ml_delete, ml_get, ml_get_buf, ml_get_buf_len, ml_get_len};
use crate::src::nvim::memory::{strequal, xcalloc, xfree, xmalloc, xmemdupz, xstrdup, xstrlcpy};
use crate::src::nvim::message::{
    emsg, internal_error, msg, msg_clr_cmdline, msg_delay, msg_ext_set_kind, msg_progress, semsg,
};
use crate::src::nvim::option::{
    can_bs, copy_option_part, magic_isset, option_set_callback_func, shortmess,
};
use crate::src::nvim::os::fs::os_fopen;
use crate::src::nvim::os::input::{fast_breakcheck, line_breakcheck, os_breakcheck};
use crate::src::nvim::os::libc::{
    __assert_fail, abs, atoi, fclose, gettext, memcmp, memmove, memset, qsort, strcat, strchr,
    strcmp, strcpy, strlen, strncasecmp, strncmp, strncpy, strrchr,
};
use crate::src::nvim::os::time::{os_delay, os_hrtime};
use crate::src::nvim::path::{expand_wildcards, path_tail, vim_ispathsep, FreeWild};
use crate::src::nvim::popupmenu::{
    pum_clear, pum_display, pum_get_height, pum_set_event_info, pum_undisplay, pum_visible,
};
use crate::src::nvim::r#move::{changed_cline_bef_curs, curs_columns, validate_cursor};
use crate::src::nvim::register::{copy_register, free_register, valid_yank_reg};
use crate::src::nvim::search::{find_pattern_in_path, ignorecase, search_for_exact_line, searchit};
use crate::src::nvim::spell::{
    expand_spelling, spell_dump_compl, spell_expand_check_cap, spell_move_to, spell_word_start,
};
use crate::src::nvim::state::may_trigger_modechanged;
use crate::src::nvim::strings::{vim_snprintf, vim_strchr, vim_strsave_escaped};
use crate::src::nvim::tag::find_tags;
use crate::src::nvim::textformat::auto_format;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, ApiDispatchWrapper, Arena, Array, BoolVarValue, Boolean,
    BufUpdateCallbacks, Callback, CallbackType, Callback_data as C2Rust_Unnamed_5,
    ChangedtickDictItem, DecorExt, DecorHighlightInline, DecorInlineData, DecorPriority,
    DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2, Dict, Direction, Error, ErrorType,
    EvalFuncData, ExtmarkMove, ExtmarkOp, ExtmarkSavePos, ExtmarkSplice, ExtmarkUndoObject, FileID,
    Float, FloatAnchor, FloatRelative, GridView, Integer, Intersection, KeyValuePair,
    ListLenSpecials, LuaRef, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t,
    Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, MotionType, MsgpackRpcRequestHandler,
    Object, ObjectType, OptIndex, OptInt, OptValData, ScopeDictDictItem, ScopeType, ScreenGrid,
    Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_13, String_0, Terminal, Timestamp, TriState,
    UndoObjectType, VarLockStatus, VarType, VimVarIndex, VirtLines, VirtText, VirtTextChunk,
    VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle, Window, _IO_codecvt, _IO_lock_t,
    _IO_marker, _IO_wide_data, __compar_fn_t, __off64_t, __off_t, __time_t, alist_T, auto_event,
    bcount_t, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T, chunksize_T, colnr_T,
    dict_T, dictitem_T, dictvar_S, disptick_T, event_T, expand_T, extmark_undo_vec_t, fcs_chars_T,
    file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_3,
    file_buffer_b_wininfo as C2Rust_Unnamed_12, file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, hlf_T, ht_stack_S, ht_stack_T, infoptr_T, int16_t, int32_t, int64_t,
    intptr_t, key_extra, key_value_pair, lcs_chars_T, linenr_T, list_T, list_stack_S, list_stack_T,
    listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T, mapblock,
    mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s,
    mtnode_s, object, object_data as C2Rust_Unnamed, optset_T, partial_S, partial_T, pos_T,
    pos_save_T, proftime_T, ptr_t, ptrdiff_t, pumitem_T, qf_info_S, qf_info_T, queue,
    reg_extmatch_T, regmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, save_v_event_T, schar_T,
    scid_T, sctx_T, searchit_arg_T, size_t, smt_T, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T, taggy_T, terminal,
    time_t, typval_T, typval_vval_union, u_entry, u_entry_T, u_header, u_header_T,
    u_header_uh_alt_next as C2Rust_Unnamed_9, u_header_uh_alt_prev as C2Rust_Unnamed_8,
    u_header_uh_next as C2Rust_Unnamed_11, u_header_uh_prev as C2Rust_Unnamed_10, ufunc_S, ufunc_T,
    uint16_t, uint32_t, uint64_t, uint8_t, undo_object, undo_object_data as C2Rust_Unnamed_7,
    varnumber_T, virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T,
    xp_prefix_T, yankreg_T, FILE, QUEUE, _IO_FILE,
};
use crate::src::nvim::ui::{ui_flush, vim_beep};
use crate::src::nvim::undo::undo_allowed;
use crate::src::nvim::window::win_valid;
use crate::src::nvim::winfloat::win_float_find_preview;
extern "C" {
    fn mergesort_list(
        head: *mut ::core::ffi::c_void,
        get_next: MergeSortGetFunc,
        set_next: MergeSortSetFunc,
        get_prev: MergeSortGetFunc,
        set_prev: MergeSortSetFunc,
        compare: MergeSortCompareFunc,
    ) -> *mut ::core::ffi::c_void;
    static pum_want: GlobalCell<C2Rust_Unnamed_25>;
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
}
pub type MergeSortGetFunc =
    Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void>;
pub type MergeSortSetFunc =
    Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut ::core::ffi::c_void) -> ()>;
pub type MergeSortCompareFunc = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_void,
        *const ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
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
pub const MAXLNUM: C2Rust_Unnamed_14 = 2147483647;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_15 = 2147483647;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenUnknown: ListLenSpecials = -1;
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
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub const XP_PREFIX_INV: xp_prefix_T = 2;
pub const XP_PREFIX_NO: xp_prefix_T = 1;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_16 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_16 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_16 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_16 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_16 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_16 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_16 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_16 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_16 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_16 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_16 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_16 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_16 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_16 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_16 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_16 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_16 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_16 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_16 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_16 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_16 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_16 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_16 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_16 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_16 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_16 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_16 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_16 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_16 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_16 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_16 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_16 = 33;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_16 = 32;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_16 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_16 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_16 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_16 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_16 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_16 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_16 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_16 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_16 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_16 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_16 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_16 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_16 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_16 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_16 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_16 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_16 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_16 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_16 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_16 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_16 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_16 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_16 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_16 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_16 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_16 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_16 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_16 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_16 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_16 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_16 = 1;
pub const EXPAND_NOTHING: C2Rust_Unnamed_16 = 0;
pub const EXPAND_OK: C2Rust_Unnamed_16 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_16 = -2;
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
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const OPENLINE_FORCE_INDENT: C2Rust_Unnamed_17 = 64;
pub const OPENLINE_FORMAT: C2Rust_Unnamed_17 = 32;
pub const OPENLINE_COM_LIST: C2Rust_Unnamed_17 = 16;
pub const OPENLINE_MARKFIX: C2Rust_Unnamed_17 = 8;
pub const OPENLINE_KEEPTRAIL: C2Rust_Unnamed_17 = 4;
pub const OPENLINE_DO_COM: C2Rust_Unnamed_17 = 2;
pub const OPENLINE_DELSPACES: C2Rust_Unnamed_17 = 1;
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
pub const kOptCotFlagNearest: C2Rust_Unnamed_19 = 1024;
pub const kOptCotFlagPreinsert: C2Rust_Unnamed_19 = 512;
pub const kOptCotFlagNosort: C2Rust_Unnamed_19 = 256;
pub const kOptCotFlagFuzzy: C2Rust_Unnamed_19 = 128;
pub const kOptCotFlagNoselect: C2Rust_Unnamed_19 = 64;
pub const kOptCotFlagNoinsert: C2Rust_Unnamed_19 = 32;
pub const kOptCotFlagPopup: C2Rust_Unnamed_19 = 16;
pub const kOptCotFlagPreview: C2Rust_Unnamed_19 = 8;
pub const kOptCotFlagLongest: C2Rust_Unnamed_19 = 4;
pub const kOptCotFlagMenuone: C2Rust_Unnamed_19 = 2;
pub const kOptCotFlagMenu: C2Rust_Unnamed_19 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const SHM_SEARCHCOUNT: C2Rust_Unnamed_20 = 83;
pub const SHM_FILEINFO: C2Rust_Unnamed_20 = 70;
pub const SHM_RECORDING: C2Rust_Unnamed_20 = 113;
pub const SHM_COMPLETIONSCAN: C2Rust_Unnamed_20 = 67;
pub const SHM_COMPLETIONMENU: C2Rust_Unnamed_20 = 99;
pub const SHM_INTRO: C2Rust_Unnamed_20 = 73;
pub const SHM_ATTENTION: C2Rust_Unnamed_20 = 65;
pub const SHM_SEARCH: C2Rust_Unnamed_20 = 115;
pub const SHM_OVERALL: C2Rust_Unnamed_20 = 79;
pub const SHM_OVER: C2Rust_Unnamed_20 = 111;
pub const SHM_TRUNCALL: C2Rust_Unnamed_20 = 84;
pub const SHM_TRUNC: C2Rust_Unnamed_20 = 116;
pub const SHM_WRITE: C2Rust_Unnamed_20 = 87;
pub const SHM_ABBREVIATIONS: C2Rust_Unnamed_20 = 97;
pub const SHM_WRI: C2Rust_Unnamed_20 = 119;
pub const SHM_LINES: C2Rust_Unnamed_20 = 108;
pub const SHM_MOD: C2Rust_Unnamed_20 = 109;
pub const SHM_RO: C2Rust_Unnamed_20 = 114;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_21 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_21 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_21 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_21 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_21 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_21 = 20;
pub const UPD_VALID: C2Rust_Unnamed_21 = 10;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const KEY_COMPLETE: C2Rust_Unnamed_22 = 259;
pub const KEY_OPEN_BACK: C2Rust_Unnamed_22 = 258;
pub const KEY_OPEN_FORW: C2Rust_Unnamed_22 = 257;
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
pub type C2Rust_Unnamed_23 = ::core::ffi::c_int;
pub const FUZZY_SCORE_NONE: C2Rust_Unnamed_23 = -2147483648;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_24 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_24 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_24 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_24 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_24 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_24 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_24 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_24 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_24 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_24 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_24 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_24 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_24 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_24 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_24 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_24 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_24 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_24 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_24 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_24 = 1;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub const CTRL_X_CMDLINE_CTRL_X: C2Rust_Unnamed_36 = 17;
pub const CTRL_X_NORMAL: C2Rust_Unnamed_36 = 0;
pub const CTRL_X_NOT_DEFINED_YET: C2Rust_Unnamed_36 = 1;
pub const CTRL_X_CMDLINE: C2Rust_Unnamed_36 = 11;
pub const CTRL_X_SCROLL: C2Rust_Unnamed_36 = 2;
pub const CTRL_X_WHOLE_LINE: C2Rust_Unnamed_36 = 3;
pub const CTRL_X_FILES: C2Rust_Unnamed_36 = 4;
pub const CTRL_X_TAGS: C2Rust_Unnamed_36 = 261;
pub const CTRL_X_PATH_PATTERNS: C2Rust_Unnamed_36 = 262;
pub const CTRL_X_PATH_DEFINES: C2Rust_Unnamed_36 = 263;
pub const CTRL_X_DICTIONARY: C2Rust_Unnamed_36 = 265;
pub const CTRL_X_THESAURUS: C2Rust_Unnamed_36 = 266;
pub const CTRL_X_FUNCTION: C2Rust_Unnamed_36 = 12;
pub const CTRL_X_OMNI: C2Rust_Unnamed_36 = 13;
pub const CTRL_X_SPELL: C2Rust_Unnamed_36 = 14;
pub const CTRL_X_EVAL: C2Rust_Unnamed_36 = 16;
pub const CTRL_X_REGISTER: C2Rust_Unnamed_36 = 19;
pub const CTRL_X_BUFNAMES: C2Rust_Unnamed_36 = 18;
pub const KE_S_DOWN: key_extra = 5;
pub const KE_S_UP: key_extra = 4;
pub type compl_T = compl_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct compl_S {
    pub cp_next: *mut compl_T,
    pub cp_prev: *mut compl_T,
    pub cp_match_next: *mut compl_T,
    pub cp_str: String_0,
    pub cp_text: [*mut ::core::ffi::c_char; 4],
    pub cp_user_data: typval_T,
    pub cp_fname: *mut ::core::ffi::c_char,
    pub cp_flags: ::core::ffi::c_int,
    pub cp_number: ::core::ffi::c_int,
    pub cp_score: ::core::ffi::c_int,
    pub cp_in_match_array: bool,
    pub cp_user_abbr_hlattr: ::core::ffi::c_int,
    pub cp_user_kind_hlattr: ::core::ffi::c_int,
    pub cp_cpt_source_idx: ::core::ffi::c_int,
}
pub const CP_ICASE: C2Rust_Unnamed_37 = 16;
pub const CP_ORIGINAL_TEXT: C2Rust_Unnamed_37 = 1;
pub const CPT_COUNT: C2Rust_Unnamed_26 = 4;
pub const CP_FREE_FNAME: C2Rust_Unnamed_37 = 2;
pub const CP_FAST: C2Rust_Unnamed_37 = 32;
pub const CP_CONT_S_IPOS: C2Rust_Unnamed_37 = 4;
pub const CPT_INFO: C2Rust_Unnamed_26 = 3;
pub const CPT_KIND: C2Rust_Unnamed_26 = 1;
pub const CPT_MENU: C2Rust_Unnamed_26 = 2;
pub const CPT_ABBR: C2Rust_Unnamed_26 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cpt_source_T {
    pub cs_refresh_always: bool,
    pub cs_startcol: ::core::ffi::c_int,
    pub cs_max_matches: ::core::ffi::c_int,
    pub compl_start_tv: uint64_t,
    pub cs_flag: ::core::ffi::c_char,
}
pub const CP_EQUAL: C2Rust_Unnamed_37 = 8;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_25 {
    pub active: bool,
    pub item: ::core::ffi::c_int,
    pub insert: bool,
    pub finish: bool,
}
pub const KE_LUA: key_extra = 103;
pub const KE_COMMAND: key_extra = 104;
pub const KE_EVENT: key_extra = 102;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ins_compl_next_state_T {
    pub e_cpt_copy: *mut ::core::ffi::c_char,
    pub e_cpt: *mut ::core::ffi::c_char,
    pub ins_buf: *mut buf_T,
    pub cur_match_pos: *mut pos_T,
    pub prev_match_pos: pos_T,
    pub set_match_pos: bool,
    pub first_match_pos: pos_T,
    pub last_match_pos: pos_T,
    pub found_all: bool,
    pub dict: *mut ::core::ffi::c_char,
    pub dict_f: ::core::ffi::c_int,
    pub func_cb: *mut Callback,
}
pub const KE_IGNORE: key_extra = 53;
pub const RE_LAST: C2Rust_Unnamed_33 = 2;
pub const SEARCH_NFMSG: C2Rust_Unnamed_32 = 8;
pub const SEARCH_KEEP: C2Rust_Unnamed_32 = 1024;
pub const PLUS_REGISTER: C2Rust_Unnamed_29 = 38;
pub const STAR_REGISTER: C2Rust_Unnamed_29 = 37;
pub const DELETION_REGISTER: C2Rust_Unnamed_29 = 36;
pub const NUM_REGISTERS: C2Rust_Unnamed_29 = 39;
pub const EW_SILENT: C2Rust_Unnamed_28 = 32;
pub const EW_ADDSLASH: C2Rust_Unnamed_28 = 8;
pub const EW_DIR: C2Rust_Unnamed_28 = 1;
pub const EW_FILE: C2Rust_Unnamed_28 = 2;
pub const TAG_MANY: C2Rust_Unnamed_35 = 300;
pub const TAG_VERBOSE: C2Rust_Unnamed_35 = 32;
pub const TAG_INS_COMP: C2Rust_Unnamed_35 = 64;
pub const TAG_NOIC: C2Rust_Unnamed_35 = 8;
pub const TAG_NAMES: C2Rust_Unnamed_35 = 2;
pub const TAG_REGEXP: C2Rust_Unnamed_35 = 4;
pub const LSIZE: C2Rust_Unnamed_34 = 512;
pub const ACTION_EXPAND: C2Rust_Unnamed_31 = 5;
pub const FIND_ANY: C2Rust_Unnamed_30 = 1;
pub const FIND_DEFINE: C2Rust_Unnamed_30 = 2;
pub const INS_COMPL_CPT_CONT: C2Rust_Unnamed_38 = 2;
pub const INS_COMPL_CPT_OK: C2Rust_Unnamed_38 = 1;
pub const INS_COMPL_CPT_END: C2Rust_Unnamed_38 = 3;
pub const CTRL_X_LOCAL_MSG: C2Rust_Unnamed_36 = 15;
pub const CTRL_X_FINISHED: C2Rust_Unnamed_36 = 8;
pub const SMT_RARE: smt_T = 2;
pub const SMT_BAD: smt_T = 1;
pub const SMT_ALL: smt_T = 0;
pub const KE_MOUSEMOVE: key_extra = 100;
pub const KE_MOUSERIGHT: key_extra = 78;
pub const KE_MOUSELEFT: key_extra = 77;
pub const KE_MOUSEUP: key_extra = 76;
pub const KE_MOUSEDOWN: key_extra = 75;
pub const OPT_GLOBAL: C2Rust_Unnamed_27 = 1;
pub const OPT_LOCAL: C2Rust_Unnamed_27 = 2;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const KE_WILD: key_extra = 108;
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
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_27 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_27 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_27 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_27 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_27 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_27 = 4;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const EW_NOBREAK: C2Rust_Unnamed_28 = 262144;
pub const EW_CDPATH: C2Rust_Unnamed_28 = 131072;
pub const EW_NOTENV: C2Rust_Unnamed_28 = 65536;
pub const EW_EMPTYOK: C2Rust_Unnamed_28 = 32768;
pub const EW_DODOT: C2Rust_Unnamed_28 = 16384;
pub const EW_SHELLCMD: C2Rust_Unnamed_28 = 8192;
pub const EW_ALLLINKS: C2Rust_Unnamed_28 = 4096;
pub const EW_KEEPDOLLAR: C2Rust_Unnamed_28 = 2048;
pub const EW_NOTWILD: C2Rust_Unnamed_28 = 1024;
pub const EW_NOERROR: C2Rust_Unnamed_28 = 512;
pub const EW_ICASE: C2Rust_Unnamed_28 = 256;
pub const EW_PATH: C2Rust_Unnamed_28 = 128;
pub const EW_EXEC: C2Rust_Unnamed_28 = 64;
pub const EW_KEEPALL: C2Rust_Unnamed_28 = 16;
pub const EW_NOTFOUND: C2Rust_Unnamed_28 = 4;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const NUM_SAVED_REGISTERS: C2Rust_Unnamed_29 = 37;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_uint;
pub const CHECK_PATH: C2Rust_Unnamed_30 = 3;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const ACTION_SHOW_ALL: C2Rust_Unnamed_31 = 4;
pub const ACTION_SPLIT: C2Rust_Unnamed_31 = 3;
pub const ACTION_GOTO: C2Rust_Unnamed_31 = 2;
pub const ACTION_SHOW: C2Rust_Unnamed_31 = 1;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub const SEARCH_COL: C2Rust_Unnamed_32 = 4096;
pub const SEARCH_PEEK: C2Rust_Unnamed_32 = 2048;
pub const SEARCH_MARK: C2Rust_Unnamed_32 = 512;
pub const SEARCH_START: C2Rust_Unnamed_32 = 256;
pub const SEARCH_NOOF: C2Rust_Unnamed_32 = 128;
pub const SEARCH_END: C2Rust_Unnamed_32 = 64;
pub const SEARCH_HIS: C2Rust_Unnamed_32 = 32;
pub const SEARCH_OPT: C2Rust_Unnamed_32 = 16;
pub const SEARCH_MSG: C2Rust_Unnamed_32 = 12;
pub const SEARCH_ECHO: C2Rust_Unnamed_32 = 2;
pub const SEARCH_REV: C2Rust_Unnamed_32 = 1;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub const RE_BOTH: C2Rust_Unnamed_33 = 2;
pub const RE_SUBST: C2Rust_Unnamed_33 = 1;
pub const RE_SEARCH: C2Rust_Unnamed_33 = 0;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub const TAG_NO_TAGFUNC: C2Rust_Unnamed_35 = 256;
pub const TAG_KEEP_LANG: C2Rust_Unnamed_35 = 128;
pub const TAG_HELP: C2Rust_Unnamed_35 = 1;
pub type C2Rust_Unnamed_36 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_37 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_38 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const Ctrl_C: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const Ctrl_D: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const Ctrl_E: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const Ctrl_F: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const Ctrl_I: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const Ctrl_K: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const Ctrl_L: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const Ctrl_N: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
pub const Ctrl_O: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const Ctrl_P: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const Ctrl_Q: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub const Ctrl_R: ::core::ffi::c_int = 18 as ::core::ffi::c_int;
pub const Ctrl_S: ::core::ffi::c_int = 19 as ::core::ffi::c_int;
pub const Ctrl_T: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const Ctrl_U: ::core::ffi::c_int = 21 as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const Ctrl_X: ::core::ffi::c_int = 24 as ::core::ffi::c_int;
pub const Ctrl_Y: ::core::ffi::c_int = 25 as ::core::ffi::c_int;
pub const Ctrl_Z: ::core::ffi::c_int = 26 as ::core::ffi::c_int;
pub const Ctrl_RSB: ::core::ffi::c_int = 29 as ::core::ffi::c_int;
pub const PATHSEP: ::core::ffi::c_int = '/' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_iswhite_or_nul(mut c: ::core::ffi::c_int) -> bool {
    return ascii_iswhite(c) as ::core::ffi::c_int != 0 || c == NUL;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn equalpos(mut a: pos_T, mut b: pos_T) -> bool {
    return a.lnum == b.lnum && a.col == b.col && a.coladd == b.coladd;
}
pub const BS_START: ::core::ffi::c_int = 's' as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn tv_list_first(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    return (*l).lv_first;
}
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const CTRL_X_WANT_IDENT: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
static ctrl_x_msgs: GlobalCell<[*mut ::core::ffi::c_char; 20]> = GlobalCell::new([
    b" Keyword completion (^N^P)\0".as_ptr() as *const ::core::ffi::c_char
        as *mut ::core::ffi::c_char,
    b" ^X mode (^]^D^E^F^I^K^L^N^O^P^Rs^U^V^Y)\0".as_ptr() as *const ::core::ffi::c_char
        as *mut ::core::ffi::c_char,
    ::core::ptr::null_mut::<::core::ffi::c_char>(),
    b" Whole line completion (^L^N^P)\0".as_ptr() as *const ::core::ffi::c_char
        as *mut ::core::ffi::c_char,
    b" File name completion (^F^N^P)\0".as_ptr() as *const ::core::ffi::c_char
        as *mut ::core::ffi::c_char,
    b" Tag completion (^]^N^P)\0".as_ptr() as *const ::core::ffi::c_char
        as *mut ::core::ffi::c_char,
    b" Path pattern completion (^N^P)\0".as_ptr() as *const ::core::ffi::c_char
        as *mut ::core::ffi::c_char,
    b" Definition completion (^D^N^P)\0".as_ptr() as *const ::core::ffi::c_char
        as *mut ::core::ffi::c_char,
    ::core::ptr::null_mut::<::core::ffi::c_char>(),
    b" Dictionary completion (^K^N^P)\0".as_ptr() as *const ::core::ffi::c_char
        as *mut ::core::ffi::c_char,
    b" Thesaurus completion (^T^N^P)\0".as_ptr() as *const ::core::ffi::c_char
        as *mut ::core::ffi::c_char,
    b" Command-line completion (^V^N^P)\0".as_ptr() as *const ::core::ffi::c_char
        as *mut ::core::ffi::c_char,
    b" User defined completion (^U^N^P)\0".as_ptr() as *const ::core::ffi::c_char
        as *mut ::core::ffi::c_char,
    b" Omni completion (^O^N^P)\0".as_ptr() as *const ::core::ffi::c_char
        as *mut ::core::ffi::c_char,
    b" Spelling suggestion (^S^N^P)\0".as_ptr() as *const ::core::ffi::c_char
        as *mut ::core::ffi::c_char,
    b" Keyword Local completion (^N^P)\0".as_ptr() as *const ::core::ffi::c_char
        as *mut ::core::ffi::c_char,
    ::core::ptr::null_mut::<::core::ffi::c_char>(),
    b" Command-line completion (^V^N^P)\0".as_ptr() as *const ::core::ffi::c_char
        as *mut ::core::ffi::c_char,
    ::core::ptr::null_mut::<::core::ffi::c_char>(),
    b" Register completion (^N^P)\0".as_ptr() as *const ::core::ffi::c_char
        as *mut ::core::ffi::c_char,
]);
static ctrl_x_mode_names: GlobalCell<[*mut ::core::ffi::c_char; 20]> = GlobalCell::new([
    b"keyword\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"ctrl_x\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"scroll\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"whole_line\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"files\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"tags\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"path_patterns\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"path_defines\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"unknown\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"dictionary\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"thesaurus\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"cmdline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"function\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"omni\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"spell\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ::core::ptr::null_mut::<::core::ffi::c_char>(),
    b"eval\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"cmdline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ::core::ptr::null_mut::<::core::ffi::c_char>(),
    b"register\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
]);
static e_hitend: GlobalCell<[::core::ffi::c_char; 21]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"Hit end of paragraph\0")
});
static e_compldel: GlobalCell<[::core::ffi::c_char; 39]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
        *b"E840: Completion function deleted text\0",
    )
});
static compl_first_match: GlobalCell<*mut compl_T> =
    GlobalCell::new(::core::ptr::null_mut::<compl_T>());
static compl_curr_match: GlobalCell<*mut compl_T> =
    GlobalCell::new(::core::ptr::null_mut::<compl_T>());
static compl_shown_match: GlobalCell<*mut compl_T> =
    GlobalCell::new(::core::ptr::null_mut::<compl_T>());
static compl_old_match: GlobalCell<*mut compl_T> =
    GlobalCell::new(::core::ptr::null_mut::<compl_T>());
static compl_best_matches: GlobalCell<*mut *mut compl_T> =
    GlobalCell::new(::core::ptr::null_mut::<*mut compl_T>());
static compl_num_bests: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static compl_enter_selects: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static compl_leader: GlobalCell<String_0> = GlobalCell::new(String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
});
static compl_get_longest: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static compl_used_match: GlobalCell<bool> = GlobalCell::new(false);
static compl_was_interrupted: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static compl_interrupted: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static compl_restarting: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static compl_started: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static ctrl_x_mode: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(CTRL_X_NORMAL as ::core::ffi::c_int);
static compl_matches: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static compl_pattern: GlobalCell<String_0> = GlobalCell::new(String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
});
static cpt_compl_pattern: GlobalCell<String_0> = GlobalCell::new(String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
});
static compl_direction: GlobalCell<Direction> = GlobalCell::new(FORWARD);
static compl_shows_dir: GlobalCell<Direction> = GlobalCell::new(FORWARD);
static compl_pending: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static compl_startpos: GlobalCell<pos_T> = GlobalCell::new(pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
});
static compl_length: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static compl_lnum: GlobalCell<linenr_T> = GlobalCell::new(0 as linenr_T);
static compl_col: GlobalCell<colnr_T> = GlobalCell::new(0 as colnr_T);
static compl_ins_end_col: GlobalCell<colnr_T> = GlobalCell::new(0 as colnr_T);
static compl_orig_text: GlobalCell<String_0> = GlobalCell::new(String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
});
static compl_orig_extmarks: GlobalCell<extmark_undo_vec_t> = GlobalCell::new(extmark_undo_vec_t {
    size: 0,
    capacity: 0,
    items: ::core::ptr::null_mut::<ExtmarkUndoObject>(),
});
static compl_cont_mode: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static compl_xp: GlobalCell<expand_T> = GlobalCell::new(expand_T {
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
});
static compl_curr_win: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
static compl_curr_buf: GlobalCell<*mut buf_T> = GlobalCell::new(::core::ptr::null_mut::<buf_T>());
pub const COMPL_INITIAL_TIMEOUT_MS: ::core::ffi::c_int = 80 as ::core::ffi::c_int;
static compl_autocomplete: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static compl_timeout_ms: GlobalCell<uint64_t> =
    GlobalCell::new(COMPL_INITIAL_TIMEOUT_MS as uint64_t);
static compl_time_slice_expired: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static compl_from_nonkeyword: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static compl_hi_on_autocompl_longest: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub const COMPL_MIN_TIMEOUT_MS: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const COMPL_FUNC_TIMEOUT_MS: ::core::ffi::c_int = 300 as ::core::ffi::c_int;
pub const COMPL_FUNC_TIMEOUT_NON_KW_MS: ::core::ffi::c_int = 1000 as ::core::ffi::c_int;
static compl_cont_status: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub const CONT_ADDING: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const CONT_INTRPT: ::core::ffi::c_int = 2 as ::core::ffi::c_int + 4 as ::core::ffi::c_int;
pub const CONT_N_ADDS: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const CONT_S_IPOS: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const CONT_SOL: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const CONT_LOCAL: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
static compl_opt_refresh_always: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static spell_bad_len: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
static compl_selected_item: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(-1 as ::core::ffi::c_int);
static compl_fuzzy_scores: GlobalCell<*mut ::core::ffi::c_int> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_int>());
static cpt_sources_array: GlobalCell<*mut cpt_source_T> =
    GlobalCell::new(::core::ptr::null_mut::<cpt_source_T>());
static cpt_sources_count: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static cpt_sources_index: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(-1 as ::core::ffi::c_int);
static compl_match_array: GlobalCell<*mut pumitem_T> =
    GlobalCell::new(::core::ptr::null_mut::<pumitem_T>());
static compl_match_arraysize: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub unsafe extern "C" fn ins_ctrl_x() {
    if !ctrl_x_mode_cmdline() {
        if compl_cont_status.get() & CONT_N_ADDS != 0 {
            (*compl_cont_status.ptr()) |= CONT_INTRPT;
        } else {
            compl_cont_status.set(0 as ::core::ffi::c_int);
        }
        ctrl_x_mode.set(CTRL_X_NOT_DEFINED_YET as ::core::ffi::c_int);
        edit_submode.set(gettext(
            (*ctrl_x_msgs.ptr())[(ctrl_x_mode.get() & !(0x100 as ::core::ffi::c_int)) as usize],
        ));
        edit_submode_pre.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        redraw_mode.set(true_0 != 0);
    } else {
        ctrl_x_mode.set(CTRL_X_CMDLINE_CTRL_X as ::core::ffi::c_int);
    }
    may_trigger_modechanged();
}
pub unsafe extern "C" fn ctrl_x_mode_none() -> bool {
    return ctrl_x_mode.get() == 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn ctrl_x_mode_normal() -> bool {
    return ctrl_x_mode.get() == CTRL_X_NORMAL as ::core::ffi::c_int;
}
pub unsafe extern "C" fn ctrl_x_mode_scroll() -> bool {
    return ctrl_x_mode.get() == CTRL_X_SCROLL as ::core::ffi::c_int;
}
pub unsafe extern "C" fn ctrl_x_mode_whole_line() -> bool {
    return ctrl_x_mode.get() == CTRL_X_WHOLE_LINE as ::core::ffi::c_int;
}
pub unsafe extern "C" fn ctrl_x_mode_files() -> bool {
    return ctrl_x_mode.get() == CTRL_X_FILES as ::core::ffi::c_int;
}
pub unsafe extern "C" fn ctrl_x_mode_tags() -> bool {
    return ctrl_x_mode.get() == CTRL_X_TAGS as ::core::ffi::c_int;
}
pub unsafe extern "C" fn ctrl_x_mode_path_patterns() -> bool {
    return ctrl_x_mode.get() == CTRL_X_PATH_PATTERNS as ::core::ffi::c_int;
}
pub unsafe extern "C" fn ctrl_x_mode_path_defines() -> bool {
    return ctrl_x_mode.get() == CTRL_X_PATH_DEFINES as ::core::ffi::c_int;
}
pub unsafe extern "C" fn ctrl_x_mode_dictionary() -> bool {
    return ctrl_x_mode.get() == CTRL_X_DICTIONARY as ::core::ffi::c_int;
}
pub unsafe extern "C" fn ctrl_x_mode_thesaurus() -> bool {
    return ctrl_x_mode.get() == CTRL_X_THESAURUS as ::core::ffi::c_int;
}
pub unsafe extern "C" fn ctrl_x_mode_cmdline() -> bool {
    return ctrl_x_mode.get() == CTRL_X_CMDLINE as ::core::ffi::c_int
        || ctrl_x_mode.get() == CTRL_X_CMDLINE_CTRL_X as ::core::ffi::c_int;
}
pub unsafe extern "C" fn ctrl_x_mode_function() -> bool {
    return ctrl_x_mode.get() == CTRL_X_FUNCTION as ::core::ffi::c_int;
}
pub unsafe extern "C" fn ctrl_x_mode_omni() -> bool {
    return ctrl_x_mode.get() == CTRL_X_OMNI as ::core::ffi::c_int;
}
pub unsafe extern "C" fn ctrl_x_mode_spell() -> bool {
    return ctrl_x_mode.get() == CTRL_X_SPELL as ::core::ffi::c_int;
}
unsafe extern "C" fn ctrl_x_mode_eval() -> bool {
    return ctrl_x_mode.get() == CTRL_X_EVAL as ::core::ffi::c_int;
}
pub unsafe extern "C" fn ctrl_x_mode_line_or_eval() -> bool {
    return ctrl_x_mode.get() == CTRL_X_WHOLE_LINE as ::core::ffi::c_int
        || ctrl_x_mode.get() == CTRL_X_EVAL as ::core::ffi::c_int;
}
pub unsafe extern "C" fn ctrl_x_mode_register() -> bool {
    return ctrl_x_mode.get() == CTRL_X_REGISTER as ::core::ffi::c_int;
}
pub unsafe extern "C" fn ctrl_x_mode_not_default() -> bool {
    return ctrl_x_mode.get() != CTRL_X_NORMAL as ::core::ffi::c_int;
}
pub unsafe extern "C" fn ctrl_x_mode_not_defined_yet() -> bool {
    return ctrl_x_mode.get() == CTRL_X_NOT_DEFINED_YET as ::core::ffi::c_int;
}
pub unsafe extern "C" fn compl_status_adding() -> bool {
    return compl_cont_status.get() & CONT_ADDING != 0;
}
pub unsafe extern "C" fn compl_status_sol() -> bool {
    return compl_cont_status.get() & CONT_SOL != 0;
}
pub unsafe extern "C" fn compl_status_local() -> bool {
    return compl_cont_status.get() & CONT_LOCAL != 0;
}
pub unsafe extern "C" fn compl_status_clear() {
    compl_cont_status.set(0 as ::core::ffi::c_int);
}
unsafe extern "C" fn compl_dir_forward() -> bool {
    return compl_direction.get() as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int;
}
unsafe extern "C" fn compl_shows_dir_forward() -> bool {
    return compl_shows_dir.get() as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int;
}
unsafe extern "C" fn compl_shows_dir_backward() -> bool {
    return compl_shows_dir.get() as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int;
}
pub unsafe extern "C" fn check_compl_option(mut dict_opt: bool) -> bool {
    if if dict_opt as ::core::ffi::c_int != 0 {
        (*(*curbuf.get()).b_p_dict as ::core::ffi::c_int == NUL
            && *p_dict.get() as ::core::ffi::c_int == NUL
            && (*curwin.get()).w_onebuf_opt.wo_spell == 0) as ::core::ffi::c_int
    } else {
        (*(*curbuf.get()).b_p_tsr as ::core::ffi::c_int == NUL
            && *p_tsr.get() as ::core::ffi::c_int == NUL
            && *(*curbuf.get()).b_p_tsrfu as ::core::ffi::c_int == NUL
            && *p_tsrfu.get() as ::core::ffi::c_int == NUL) as ::core::ffi::c_int
    } != 0
    {
        ctrl_x_mode.set(CTRL_X_NORMAL as ::core::ffi::c_int);
        edit_submode.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        emsg(if dict_opt as ::core::ffi::c_int != 0 {
            gettext(b"'dictionary' option is empty\0".as_ptr() as *const ::core::ffi::c_char)
        } else {
            gettext(b"'thesaurus' option is empty\0".as_ptr() as *const ::core::ffi::c_char)
        });
        if emsg_silent.get() == 0 as ::core::ffi::c_int && !in_assert_fails.get() {
            vim_beep(kOptBoFlagComplete as ::core::ffi::c_int as ::core::ffi::c_uint);
            setcursor();
            msg_delay(2004 as uint64_t, false_0 != 0);
        }
        return false_0 != 0;
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn vim_is_ctrl_x_key(mut c: ::core::ffi::c_int) -> bool {
    if c == Ctrl_R && ctrl_x_mode.get() != CTRL_X_REGISTER as ::core::ffi::c_int {
        return true_0 != 0;
    }
    if ins_compl_pum_key(c) {
        return true_0 != 0;
    }
    match ctrl_x_mode.get() {
        0 => return c == Ctrl_N || c == Ctrl_P || c == Ctrl_X,
        1 | 17 => {
            return c == Ctrl_X
                || c == Ctrl_Y
                || c == Ctrl_E
                || c == Ctrl_L
                || c == Ctrl_F
                || c == Ctrl_RSB
                || c == Ctrl_I
                || c == Ctrl_D
                || c == Ctrl_P
                || c == Ctrl_N
                || c == Ctrl_T
                || c == Ctrl_V
                || c == Ctrl_Q
                || c == Ctrl_U
                || c == Ctrl_O
                || c == Ctrl_S
                || c == Ctrl_K
                || c == 's' as ::core::ffi::c_int
                || c == Ctrl_Z
                || c == Ctrl_R;
        }
        2 => return c == Ctrl_Y || c == Ctrl_E,
        3 => return c == Ctrl_L || c == Ctrl_P || c == Ctrl_N,
        4 => return c == Ctrl_F || c == Ctrl_P || c == Ctrl_N,
        265 => return c == Ctrl_K || c == Ctrl_P || c == Ctrl_N,
        266 => return c == Ctrl_T || c == Ctrl_P || c == Ctrl_N,
        261 => return c == Ctrl_RSB || c == Ctrl_P || c == Ctrl_N,
        262 => return c == Ctrl_P || c == Ctrl_N,
        263 => return c == Ctrl_D || c == Ctrl_P || c == Ctrl_N,
        11 => {
            return c == Ctrl_V || c == Ctrl_Q || c == Ctrl_P || c == Ctrl_N || c == Ctrl_X;
        }
        12 => return c == Ctrl_U || c == Ctrl_P || c == Ctrl_N,
        13 => return c == Ctrl_O || c == Ctrl_P || c == Ctrl_N,
        14 => return c == Ctrl_S || c == Ctrl_P || c == Ctrl_N,
        16 => return c == Ctrl_P || c == Ctrl_N,
        18 => return c == Ctrl_P || c == Ctrl_N,
        19 => return c == Ctrl_R || c == Ctrl_P || c == Ctrl_N,
        _ => {}
    }
    internal_error(b"vim_is_ctrl_x_key()\0".as_ptr() as *const ::core::ffi::c_char);
    return false_0 != 0;
}
unsafe extern "C" fn match_at_original_text(match_0: *const compl_T) -> bool {
    return (*match_0).cp_flags & CP_ORIGINAL_TEXT as ::core::ffi::c_int != 0;
}
unsafe extern "C" fn is_first_match(match_0: *const compl_T) -> bool {
    return match_0 == compl_first_match.get() as *const compl_T;
}
unsafe extern "C" fn do_autocmd_completedone(
    mut c: ::core::ffi::c_int,
    mut mode: ::core::ffi::c_int,
    mut word: *mut ::core::ffi::c_char,
) {
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
    mode = mode & !CTRL_X_WANT_IDENT;
    let mut mode_str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !(*ctrl_x_mode_names.ptr())[mode as usize].is_null() {
        mode_str = (*ctrl_x_mode_names.ptr())[mode as usize];
    }
    tv_dict_add_str(
        v_event,
        b"complete_word\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 14]>().wrapping_sub(1 as size_t),
        if !word.is_null() {
            word as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
    );
    tv_dict_add_str(
        v_event,
        b"complete_type\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 14]>().wrapping_sub(1 as size_t),
        if !mode_str.is_null() {
            mode_str as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
    );
    tv_dict_add_str(
        v_event,
        b"reason\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        if c == Ctrl_Y || !word.is_null() {
            b"accept\0".as_ptr() as *const ::core::ffi::c_char
        } else if c == Ctrl_E {
            b"cancel\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"discard\0".as_ptr() as *const ::core::ffi::c_char
        },
    );
    tv_dict_set_keys_readonly(v_event);
    ins_apply_autocmds(EVENT_COMPLETEDONE);
    restore_v_event(v_event, &raw mut save_v_event);
}
pub unsafe extern "C" fn ins_compl_accept_char(mut c: ::core::ffi::c_int) -> bool {
    if compl_autocomplete.get() as ::core::ffi::c_int != 0
        && compl_from_nonkeyword.get() as ::core::ffi::c_int != 0
    {
        return false_0 != 0;
    }
    if ctrl_x_mode.get() & CTRL_X_WANT_IDENT != 0 {
        return vim_isIDc(c);
    }
    match ctrl_x_mode.get() {
        4 => return vim_isfilec(c) as ::core::ffi::c_int != 0 && !vim_ispathsep(c),
        11 | 17 | 13 => {
            return vim_isprintc(c) as ::core::ffi::c_int != 0 && !ascii_iswhite(c);
        }
        3 => return vim_isprintc(c),
        _ => {}
    }
    return vim_iswordc(c);
}
unsafe extern "C" fn ins_compl_infercase_gettext(
    mut str: *const ::core::ffi::c_char,
    mut char_len: ::core::ffi::c_int,
    mut compl_char_len: ::core::ffi::c_int,
    mut min_len: ::core::ffi::c_int,
    mut tofree: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut has_lower: bool = false_0 != 0;
    let wca: *mut ::core::ffi::c_int =
        xmalloc((char_len as size_t).wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()))
            as *mut ::core::ffi::c_int;
    let mut p: *const ::core::ffi::c_char = str;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < char_len {
        *wca.offset(i as isize) = mb_ptr2char_adv(&raw mut p);
        i += 1;
    }
    let mut p_0: *const ::core::ffi::c_char = (*compl_orig_text.ptr()).data;
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < min_len {
        let c: ::core::ffi::c_int = mb_ptr2char_adv(&raw mut p_0);
        if mb_islower(c) {
            has_lower = true_0 != 0;
            if mb_isupper(*wca.offset(i_0 as isize)) {
                i_0 = compl_char_len;
                while i_0 < char_len {
                    *wca.offset(i_0 as isize) = mb_tolower(*wca.offset(i_0 as isize));
                    i_0 += 1;
                }
                break;
            }
        }
        i_0 += 1;
    }
    if !has_lower {
        let mut was_letter: bool = false_0 != 0;
        let mut p_1: *const ::core::ffi::c_char = (*compl_orig_text.ptr()).data;
        let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_1 < min_len {
            let c_0: ::core::ffi::c_int = mb_ptr2char_adv(&raw mut p_1);
            if was_letter as ::core::ffi::c_int != 0
                && mb_isupper(c_0) as ::core::ffi::c_int != 0
                && mb_islower(*wca.offset(i_1 as isize)) as ::core::ffi::c_int != 0
            {
                i_1 = compl_char_len;
                while i_1 < char_len {
                    *wca.offset(i_1 as isize) = mb_toupper(*wca.offset(i_1 as isize));
                    i_1 += 1;
                }
                break;
            } else {
                was_letter = mb_islower(c_0) as ::core::ffi::c_int != 0
                    || mb_isupper(c_0) as ::core::ffi::c_int != 0;
                i_1 += 1;
            }
        }
    }
    let mut p_2: *const ::core::ffi::c_char = (*compl_orig_text.ptr()).data;
    let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_2 < min_len {
        let c_1: ::core::ffi::c_int = mb_ptr2char_adv(&raw mut p_2);
        if mb_islower(c_1) {
            *wca.offset(i_2 as isize) = mb_tolower(*wca.offset(i_2 as isize));
        } else if mb_isupper(c_1) {
            *wca.offset(i_2 as isize) = mb_toupper(*wca.offset(i_2 as isize));
        }
        i_2 += 1;
    }
    let mut gap: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut p_3: *mut ::core::ffi::c_char = IObuff.ptr() as *mut ::core::ffi::c_char;
    let mut i_3: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    ga_init(
        &raw mut gap,
        1 as ::core::ffi::c_int,
        500 as ::core::ffi::c_int,
    );
    while i_3 < char_len {
        if !gap.ga_data.is_null() {
            ga_grow(&raw mut gap, 10 as ::core::ffi::c_int);
            '_c2rust_label: {
                if !gap.ga_data.is_null() {
                } else {
                    __assert_fail(
                        b"gap.ga_data != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        771 as ::core::ffi::c_uint,
                        b"char *ins_compl_infercase_gettext(const char *, int, int, int, char **)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            p_3 = (gap.ga_data as *mut ::core::ffi::c_char).offset(gap.ga_len as isize);
            let c2rust_fresh0 = i_3;
            i_3 = i_3 + 1;
            gap.ga_len += utf_char2bytes(*wca.offset(c2rust_fresh0 as isize), p_3);
        } else if p_3.offset_from(IObuff.ptr() as *mut ::core::ffi::c_char) + 6 as isize
            >= IOSIZE as isize
        {
            ga_grow(&raw mut gap, IOSIZE);
            *p_3 = NUL as ::core::ffi::c_char;
            strcpy(
                gap.ga_data as *mut ::core::ffi::c_char,
                IObuff.ptr() as *mut ::core::ffi::c_char,
            );
            gap.ga_len =
                p_3.offset_from(IObuff.ptr() as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
        } else {
            let c2rust_fresh1 = i_3;
            i_3 = i_3 + 1;
            p_3 = p_3.offset(utf_char2bytes(*wca.offset(c2rust_fresh1 as isize), p_3) as isize);
        }
    }
    xfree(wca as *mut ::core::ffi::c_void);
    if !gap.ga_data.is_null() {
        *tofree = gap.ga_data as *mut ::core::ffi::c_char;
        return gap.ga_data as *mut ::core::ffi::c_char;
    }
    *p_3 = NUL as ::core::ffi::c_char;
    return IObuff.ptr() as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn ins_compl_add_infercase(
    mut str_arg: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut icase: bool,
    mut fname: *mut ::core::ffi::c_char,
    mut dir: Direction,
    mut cont_s_ipos: bool,
    mut score: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut str: *mut ::core::ffi::c_char = str_arg;
    let mut char_len: ::core::ffi::c_int = 0;
    let mut compl_char_len: ::core::ffi::c_int = 0;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if p_ic.get() != 0 && (*curbuf.get()).b_p_inf != 0 && len > 0 as ::core::ffi::c_int {
        let mut p: *const ::core::ffi::c_char = str;
        char_len = 0 as ::core::ffi::c_int;
        while *p as ::core::ffi::c_int != NUL {
            p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
            char_len += 1;
        }
        let mut p_0: *const ::core::ffi::c_char = (*compl_orig_text.ptr()).data;
        compl_char_len = 0 as ::core::ffi::c_int;
        while *p_0 as ::core::ffi::c_int != NUL {
            p_0 = p_0.offset(utfc_ptr2len(p_0 as *mut ::core::ffi::c_char) as isize);
            compl_char_len += 1;
        }
        let mut min_len: ::core::ffi::c_int = if char_len < compl_char_len {
            char_len
        } else {
            compl_char_len
        };
        str = ins_compl_infercase_gettext(str, char_len, compl_char_len, min_len, &raw mut tofree);
    }
    if cont_s_ipos {
        flags |= CP_CONT_S_IPOS as ::core::ffi::c_int;
    }
    if icase {
        flags |= CP_ICASE as ::core::ffi::c_int;
    }
    let mut res: ::core::ffi::c_int = ins_compl_add(
        str,
        len,
        fname,
        ::core::ptr::null::<*mut ::core::ffi::c_char>(),
        false_0 != 0,
        ::core::ptr::null_mut::<typval_T>(),
        dir,
        flags,
        false_0 != 0,
        ::core::ptr::null::<::core::ffi::c_int>(),
        score,
    );
    xfree(tofree as *mut ::core::ffi::c_void);
    return res;
}
#[inline]
unsafe extern "C" fn free_cptext(cptext: *const *mut ::core::ffi::c_char) {
    if !cptext.is_null() {
        let mut i: size_t = 0 as size_t;
        while i < CPT_COUNT as ::core::ffi::c_int as size_t {
            xfree(*cptext.offset(i as isize) as *mut ::core::ffi::c_void);
            i = i.wrapping_add(1);
        }
    }
}
unsafe extern "C" fn cot_fuzzy() -> bool {
    return get_cot_flags() & kOptCotFlagFuzzy as ::core::ffi::c_int as ::core::ffi::c_uint
        != 0 as ::core::ffi::c_uint
        && !ctrl_x_mode_thesaurus();
}
unsafe extern "C" fn is_nearest_active() -> bool {
    return (compl_autocomplete.get() as ::core::ffi::c_int != 0
        || get_cot_flags() & kOptCotFlagNearest as ::core::ffi::c_int as ::core::ffi::c_uint != 0)
        && !cot_fuzzy();
}
pub unsafe extern "C" fn ins_compl_is_match_selected() -> bool {
    return !(*compl_shown_match.ptr()).is_null() && !is_first_match(compl_shown_match.get());
}
pub unsafe extern "C" fn ins_compl_preinsert_longest() -> bool {
    return compl_autocomplete.get() as ::core::ffi::c_int != 0
        && get_cot_flags()
            & (kOptCotFlagLongest as ::core::ffi::c_int
                | kOptCotFlagPreinsert as ::core::ffi::c_int
                | kOptCotFlagFuzzy as ::core::ffi::c_int) as ::core::ffi::c_uint
            == kOptCotFlagLongest as ::core::ffi::c_int as ::core::ffi::c_uint;
}
unsafe extern "C" fn ins_compl_add(
    str: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    fname: *mut ::core::ffi::c_char,
    cptext: *const *mut ::core::ffi::c_char,
    cptext_allocated: bool,
    mut user_data: *mut typval_T,
    cdir: Direction,
    mut flags_arg: ::core::ffi::c_int,
    adup: bool,
    mut user_hl: *const ::core::ffi::c_int,
    score: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut match_0: *mut compl_T = ::core::ptr::null_mut::<compl_T>();
    let dir: Direction = (if cdir as ::core::ffi::c_int == kDirectionNotSet as ::core::ffi::c_int {
        compl_direction.get() as ::core::ffi::c_int
    } else {
        cdir as ::core::ffi::c_int
    }) as Direction;
    let mut flags: ::core::ffi::c_int = flags_arg;
    let mut inserted: bool = false_0 != 0;
    if flags & CP_FAST as ::core::ffi::c_int != 0 {
        fast_breakcheck();
    } else {
        os_breakcheck();
    }
    if got_int.get() {
        if cptext_allocated {
            free_cptext(cptext);
        }
        return FAIL;
    }
    if len < 0 as ::core::ffi::c_int {
        len = strlen(str) as ::core::ffi::c_int;
    }
    if !(*compl_first_match.ptr()).is_null() && !adup {
        match_0 = compl_first_match.get();
        loop {
            if !match_at_original_text(match_0)
                && strncmp((*match_0).cp_str.data, str, len as size_t) == 0 as ::core::ffi::c_int
                && ((*match_0).cp_str.size as ::core::ffi::c_int <= len
                    || *(*match_0).cp_str.data.offset(len as isize) as ::core::ffi::c_int == NUL)
            {
                if is_nearest_active() as ::core::ffi::c_int != 0
                    && score > 0 as ::core::ffi::c_int
                    && score < (*match_0).cp_score
                {
                    (*match_0).cp_score = score;
                }
                if cptext_allocated {
                    free_cptext(cptext);
                }
                return NOTDONE;
            }
            match_0 = (*match_0).cp_next;
            if !(!match_0.is_null() && !is_first_match(match_0)) {
                break;
            }
        }
    }
    ins_compl_del_pum();
    match_0 = xcalloc(1 as size_t, ::core::mem::size_of::<compl_T>()) as *mut compl_T;
    (*match_0).cp_number = if flags & CP_ORIGINAL_TEXT as ::core::ffi::c_int != 0 {
        0 as ::core::ffi::c_int
    } else {
        -1 as ::core::ffi::c_int
    };
    (*match_0).cp_str = cbuf_to_string(str, len as size_t);
    if !fname.is_null()
        && !(*compl_curr_match.ptr()).is_null()
        && !(*compl_curr_match.get()).cp_fname.is_null()
        && strcmp(fname, (*compl_curr_match.get()).cp_fname) == 0 as ::core::ffi::c_int
    {
        (*match_0).cp_fname = (*compl_curr_match.get()).cp_fname;
    } else if !fname.is_null() {
        (*match_0).cp_fname = xstrdup(fname);
        flags |= CP_FREE_FNAME as ::core::ffi::c_int;
    } else {
        (*match_0).cp_fname = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    (*match_0).cp_flags = flags;
    (*match_0).cp_user_abbr_hlattr = if !user_hl.is_null() {
        *user_hl.offset(0 as ::core::ffi::c_int as isize)
    } else {
        -1 as ::core::ffi::c_int
    };
    (*match_0).cp_user_kind_hlattr = if !user_hl.is_null() {
        *user_hl.offset(1 as ::core::ffi::c_int as isize)
    } else {
        -1 as ::core::ffi::c_int
    };
    (*match_0).cp_score = score;
    (*match_0).cp_cpt_source_idx = cpt_sources_index.get();
    if !cptext.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < CPT_COUNT as ::core::ffi::c_int {
            if !(*cptext.offset(i as isize)).is_null() {
                if **cptext.offset(i as isize) as ::core::ffi::c_int != NUL {
                    (*match_0).cp_text[i as usize] =
                        (if cptext_allocated as ::core::ffi::c_int != 0 {
                            *cptext.offset(i as isize)
                        } else {
                            xstrdup(*cptext.offset(i as isize))
                        }) as *mut ::core::ffi::c_char;
                } else if cptext_allocated {
                    xfree(*cptext.offset(i as isize) as *mut ::core::ffi::c_void);
                }
            }
            i += 1;
        }
    }
    if !user_data.is_null() {
        (*match_0).cp_user_data = *user_data;
    }
    if (*compl_first_match.ptr()).is_null() {
        (*match_0).cp_prev = ::core::ptr::null_mut::<compl_T>();
        (*match_0).cp_next = (*match_0).cp_prev;
    } else if cot_fuzzy() as ::core::ffi::c_int != 0
        && score != FUZZY_SCORE_NONE as ::core::ffi::c_int
        && compl_get_longest.get() as ::core::ffi::c_int != 0
    {
        let mut current: *mut compl_T = (*compl_first_match.get()).cp_next;
        let mut prev: *mut compl_T = compl_first_match.get();
        inserted = false_0 != 0;
        while !current.is_null() && current != compl_first_match.get() {
            if (*current).cp_score < score {
                (*match_0).cp_next = current;
                (*match_0).cp_prev = (*current).cp_prev;
                if !(*current).cp_prev.is_null() {
                    (*(*current).cp_prev).cp_next = match_0;
                }
                (*current).cp_prev = match_0;
                inserted = true_0 != 0;
                break;
            } else {
                prev = current;
                current = (*current).cp_next;
            }
        }
        if !inserted {
            (*prev).cp_next = match_0;
            (*match_0).cp_prev = prev;
            (*match_0).cp_next = compl_first_match.get();
            (*compl_first_match.get()).cp_prev = match_0;
        }
    } else if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
        (*match_0).cp_next = (*compl_curr_match.get()).cp_next;
        (*match_0).cp_prev = compl_curr_match.get();
    } else {
        (*match_0).cp_next = compl_curr_match.get();
        (*match_0).cp_prev = (*compl_curr_match.get()).cp_prev;
    }
    if !(*match_0).cp_next.is_null() {
        (*(*match_0).cp_next).cp_prev = match_0;
    }
    if !(*match_0).cp_prev.is_null() {
        (*(*match_0).cp_prev).cp_next = match_0;
    } else {
        compl_first_match.set(match_0);
    }
    compl_curr_match.set(match_0);
    if compl_get_longest.get() as ::core::ffi::c_int != 0
        && flags & CP_ORIGINAL_TEXT as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        && !cot_fuzzy()
        && !ins_compl_preinsert_longest()
        && !ctrl_x_mode_thesaurus()
    {
        ins_compl_longest_match(match_0);
    }
    return OK;
}
unsafe extern "C" fn ins_compl_equal(
    mut match_0: *mut compl_T,
    mut str: *mut ::core::ffi::c_char,
    mut len: size_t,
) -> bool {
    if (*match_0).cp_flags & CP_EQUAL as ::core::ffi::c_int != 0 {
        return true_0 != 0;
    }
    if (*match_0).cp_flags & CP_ICASE as ::core::ffi::c_int != 0 {
        return strncasecmp((*match_0).cp_str.data, str, len) == 0 as ::core::ffi::c_int;
    }
    return strncmp((*match_0).cp_str.data, str, len) == 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn ins_compl_insert_bytes(
    mut p: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) {
    if len == -1 as ::core::ffi::c_int {
        len = strlen(p) as ::core::ffi::c_int;
    }
    '_c2rust_label: {
        if len >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"len >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1089 as ::core::ffi::c_uint,
                b"void ins_compl_insert_bytes(char *, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    ins_bytes_len(p, len as size_t);
    compl_ins_end_col.set((*curwin.get()).w_cursor.col);
}
pub unsafe extern "C" fn ins_compl_leader() -> *mut ::core::ffi::c_char {
    return if !(*compl_leader.ptr()).data.is_null() {
        (*compl_leader.ptr()).data
    } else {
        (*compl_orig_text.ptr()).data
    };
}
unsafe extern "C" fn ins_compl_leader_len() -> size_t {
    return if !(*compl_leader.ptr()).data.is_null() {
        (*compl_leader.ptr()).size
    } else {
        (*compl_orig_text.ptr()).size
    };
}
pub unsafe extern "C" fn ins_compl_col_range_attr(
    mut lnum: linenr_T,
    mut col: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let has_preinsert: bool = ins_compl_has_preinsert() as ::core::ffi::c_int != 0
        || ins_compl_preinsert_longest() as ::core::ffi::c_int != 0;
    let mut attr: ::core::ffi::c_int = 0;
    if cot_fuzzy() as ::core::ffi::c_int != 0
        || !compl_hi_on_autocompl_longest.get()
            && ins_compl_preinsert_longest() as ::core::ffi::c_int != 0
        || {
            attr = syn_name2attr(if has_preinsert as ::core::ffi::c_int != 0 {
                b"PreInsert\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"ComplMatchIns\0".as_ptr() as *const ::core::ffi::c_char
            });
            attr == 0 as ::core::ffi::c_int
        }
    {
        return -1 as ::core::ffi::c_int;
    }
    let mut start_col: ::core::ffi::c_int =
        compl_col.get() as ::core::ffi::c_int + ins_compl_leader_len() as ::core::ffi::c_int;
    if !ins_compl_has_multiple() {
        return if col >= start_col && col < compl_ins_end_col.get() {
            attr
        } else {
            -1 as ::core::ffi::c_int
        };
    }
    if lnum == compl_lnum.get() && col >= start_col && col < MAXCOL as ::core::ffi::c_int
        || lnum > compl_lnum.get() && lnum < (*curwin.get()).w_cursor.lnum
        || lnum == (*curwin.get()).w_cursor.lnum && col <= compl_ins_end_col.get()
    {
        return attr;
    }
    return -1 as ::core::ffi::c_int;
}
unsafe extern "C" fn ins_compl_has_multiple() -> bool {
    return !vim_strchr(
        (*compl_shown_match.get()).cp_str.data,
        '\n' as ::core::ffi::c_int,
    )
    .is_null();
}
pub unsafe extern "C" fn ins_compl_lnum_in_range(mut lnum: linenr_T) -> bool {
    if !ins_compl_has_multiple() {
        return false_0 != 0;
    }
    return lnum >= compl_lnum.get() && lnum <= (*curwin.get()).w_cursor.lnum;
}
unsafe extern "C" fn ins_compl_longest_match(mut match_0: *mut compl_T) {
    if (*compl_leader.ptr()).data.is_null() {
        compl_leader.set(copy_string(
            (*match_0).cp_str,
            ::core::ptr::null_mut::<Arena>(),
        ));
        let mut had_match: bool = (*curwin.get()).w_cursor.col > compl_col.get();
        ins_compl_longest_insert((*compl_leader.ptr()).data);
        if !had_match {
            ins_compl_delete(false_0 != 0);
        }
        compl_used_match.set(false_0 != 0);
        return;
    }
    let mut p: *mut ::core::ffi::c_char = (*compl_leader.ptr()).data;
    let mut s: *mut ::core::ffi::c_char = (*match_0).cp_str.data;
    while *p as ::core::ffi::c_int != NUL {
        let mut c1: ::core::ffi::c_int = utf_ptr2char(p);
        let mut c2: ::core::ffi::c_int = utf_ptr2char(s);
        if if (*match_0).cp_flags & CP_ICASE as ::core::ffi::c_int != 0 {
            (mb_tolower(c1) != mb_tolower(c2)) as ::core::ffi::c_int
        } else {
            (c1 != c2) as ::core::ffi::c_int
        } != 0
        {
            break;
        }
        p = p.offset(utfc_ptr2len(p) as isize);
        s = s.offset(utfc_ptr2len(s) as isize);
    }
    if *p as ::core::ffi::c_int != NUL {
        *p = NUL as ::core::ffi::c_char;
        (*compl_leader.ptr()).size = p.offset_from((*compl_leader.ptr()).data) as size_t;
        let mut had_match_0: bool = (*curwin.get()).w_cursor.col > compl_col.get();
        ins_compl_longest_insert((*compl_leader.ptr()).data);
        if !had_match_0 {
            ins_compl_delete(false_0 != 0);
        }
    }
    compl_used_match.set(false_0 != 0);
}
unsafe extern "C" fn ins_compl_add_matches(
    mut num_matches: ::core::ffi::c_int,
    mut matches: *mut *mut ::core::ffi::c_char,
    mut icase: ::core::ffi::c_int,
) {
    let mut add_r: ::core::ffi::c_int = OK;
    let mut dir: Direction = compl_direction.get();
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_matches && add_r != FAIL {
        add_r = ins_compl_add(
            *matches.offset(i as isize),
            -1 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null::<*mut ::core::ffi::c_char>(),
            false_0 != 0,
            ::core::ptr::null_mut::<typval_T>(),
            dir,
            CP_FAST as ::core::ffi::c_int
                | (if icase != 0 {
                    CP_ICASE as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }),
            false_0 != 0,
            ::core::ptr::null::<::core::ffi::c_int>(),
            FUZZY_SCORE_NONE as ::core::ffi::c_int,
        );
        if add_r == OK {
            dir = FORWARD;
        }
        i += 1;
    }
    FreeWild(num_matches, matches);
}
unsafe extern "C" fn ins_compl_make_cyclic() -> ::core::ffi::c_int {
    if (*compl_first_match.ptr()).is_null() {
        return 0 as ::core::ffi::c_int;
    }
    let mut match_0: *mut compl_T = compl_first_match.get();
    let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while !(*match_0).cp_next.is_null() && !is_first_match((*match_0).cp_next) {
        match_0 = (*match_0).cp_next;
        count += 1;
    }
    (*match_0).cp_next = compl_first_match.get();
    (*compl_first_match.get()).cp_prev = match_0;
    return count;
}
pub unsafe extern "C" fn ins_compl_has_shown_match() -> bool {
    return (*compl_shown_match.ptr()).is_null()
        || compl_shown_match.get() != (*compl_shown_match.get()).cp_next;
}
pub unsafe extern "C" fn ins_compl_long_shown_match() -> bool {
    return !(*compl_shown_match.ptr()).is_null()
        && !(*compl_shown_match.get()).cp_str.data.is_null()
        && (*compl_shown_match.get()).cp_str.size as colnr_T
            > (*curwin.get()).w_cursor.col - compl_col.get();
}
pub unsafe extern "C" fn get_cot_flags() -> ::core::ffi::c_uint {
    return if (*curbuf.get()).b_cot_flags != 0 as ::core::ffi::c_uint {
        (*curbuf.get()).b_cot_flags
    } else {
        cot_flags.get()
    };
}
unsafe extern "C" fn ins_compl_del_pum() {
    if (*compl_match_array.ptr()).is_null() {
        return;
    }
    pum_undisplay(false_0 != 0);
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        compl_match_array.ptr() as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
}
pub unsafe extern "C" fn pum_wanted() -> bool {
    return get_cot_flags()
        & (kOptCotFlagMenu as ::core::ffi::c_int | kOptCotFlagMenuone as ::core::ffi::c_int)
            as ::core::ffi::c_uint
        != 0 as ::core::ffi::c_uint
        || compl_autocomplete.get() as ::core::ffi::c_int != 0;
}
unsafe extern "C" fn pum_enough_matches() -> bool {
    let mut comp: *mut compl_T = compl_first_match.get();
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while !(comp.is_null()
        || !match_at_original_text(comp) && {
            i += 1;
            i == 2 as ::core::ffi::c_int
        })
    {
        comp = (*comp).cp_next;
        if is_first_match(comp) {
            break;
        }
    }
    if get_cot_flags() & kOptCotFlagMenuone as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        || compl_autocomplete.get() as ::core::ffi::c_int != 0
    {
        return i >= 1 as ::core::ffi::c_int;
    }
    return i >= 2 as ::core::ffi::c_int;
}
unsafe extern "C" fn ins_compl_dict_alloc(mut match_0: *mut compl_T) -> *mut dict_T {
    let mut dict: *mut dict_T = tv_dict_alloc_lock(VAR_FIXED);
    tv_dict_add_str(
        dict,
        b"word\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        (*match_0).cp_str.data,
    );
    tv_dict_add_str(
        dict,
        b"abbr\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        (*match_0).cp_text[CPT_ABBR as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
    );
    tv_dict_add_str(
        dict,
        b"menu\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        (*match_0).cp_text[CPT_MENU as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
    );
    tv_dict_add_str(
        dict,
        b"kind\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        (*match_0).cp_text[CPT_KIND as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
    );
    tv_dict_add_str(
        dict,
        b"info\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        (*match_0).cp_text[CPT_INFO as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
    );
    if (*match_0).cp_user_data.v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_dict_add_str(
            dict,
            b"user_data\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            b"\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        tv_dict_add_tv(
            dict,
            b"user_data\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            &raw mut (*match_0).cp_user_data,
        );
    }
    return dict;
}
unsafe extern "C" fn trigger_complete_changed_event(mut cur: ::core::ffi::c_int) {
    static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
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
    if recursive.get() {
        return;
    }
    let mut item: *mut dict_T = if cur < 0 as ::core::ffi::c_int {
        tv_dict_alloc()
    } else {
        ins_compl_dict_alloc(compl_curr_match.get())
    };
    let mut v_event: *mut dict_T = get_v_event(&raw mut save_v_event);
    tv_dict_add_dict(
        v_event,
        b"completed_item\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 15]>().wrapping_sub(1 as size_t),
        item,
    );
    pum_set_event_info(v_event);
    tv_dict_set_keys_readonly(v_event);
    recursive.set(true_0 != 0);
    (*textlock.ptr()) += 1;
    apply_autocmds(
        EVENT_COMPLETECHANGED,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
    (*textlock.ptr()) -= 1;
    recursive.set(false_0 != 0);
    restore_v_event(v_event, &raw mut save_v_event);
}
unsafe extern "C" fn cp_get_next(mut node: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    return (*(node as *mut compl_T)).cp_next as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn cp_set_next(
    mut node: *mut ::core::ffi::c_void,
    mut next: *mut ::core::ffi::c_void,
) {
    (*(node as *mut compl_T)).cp_next = next as *mut compl_T;
}
unsafe extern "C" fn cp_get_prev(mut node: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    return (*(node as *mut compl_T)).cp_prev as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn cp_set_prev(
    mut node: *mut ::core::ffi::c_void,
    mut prev: *mut ::core::ffi::c_void,
) {
    (*(node as *mut compl_T)).cp_prev = prev as *mut compl_T;
}
unsafe extern "C" fn cp_compare_fuzzy(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut score_a: ::core::ffi::c_int = (*(a as *mut compl_T)).cp_score;
    let mut score_b: ::core::ffi::c_int = (*(b as *mut compl_T)).cp_score;
    return if score_b > score_a {
        1 as ::core::ffi::c_int
    } else if score_b < score_a {
        -1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
}
unsafe extern "C" fn cp_compare_nearest(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut score_a: ::core::ffi::c_int = (*(a as *mut compl_T)).cp_score;
    let mut score_b: ::core::ffi::c_int = (*(b as *mut compl_T)).cp_score;
    if score_a == FUZZY_SCORE_NONE as ::core::ffi::c_int
        || score_b == FUZZY_SCORE_NONE as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    return if score_a > score_b {
        1 as ::core::ffi::c_int
    } else if score_a < score_b {
        -1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
}
unsafe extern "C" fn prepend_startcol_text(
    mut dest: *mut String_0,
    mut src: *mut String_0,
    mut startcol: ::core::ffi::c_int,
) {
    let mut prepend_len: ::core::ffi::c_int = compl_col.get() as ::core::ffi::c_int - startcol;
    let mut new_length: ::core::ffi::c_int = prepend_len + (*src).size as ::core::ffi::c_int;
    (*dest).size = new_length as size_t;
    (*dest).data =
        xmalloc((new_length as size_t).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
    let mut line: *mut ::core::ffi::c_char = ml_get((*curwin.get()).w_cursor.lnum);
    memmove(
        (*dest).data as *mut ::core::ffi::c_void,
        line.offset(startcol as isize) as *const ::core::ffi::c_void,
        prepend_len as size_t,
    );
    memmove(
        (*dest).data.offset(prepend_len as isize) as *mut ::core::ffi::c_void,
        (*src).data as *const ::core::ffi::c_void,
        (*src).size,
    );
    *(*dest).data.offset(new_length as isize) = NUL as ::core::ffi::c_char;
}
unsafe extern "C" fn get_leader_for_startcol(
    mut match_0: *mut compl_T,
    mut cached: bool,
) -> *mut String_0 {
    let mut cpt_idx: ::core::ffi::c_int = 0;
    let mut startcol: ::core::ffi::c_int = 0;
    static adjusted_leader: GlobalCell<String_0> = GlobalCell::new(String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0 as size_t,
    });
    if match_0.is_null() {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*adjusted_leader.ptr()).data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        (*adjusted_leader.ptr()).size = 0 as size_t;
        return ::core::ptr::null_mut::<String_0>();
    }
    if !(*cpt_sources_array.ptr()).is_null() {
        cpt_idx = (*match_0).cp_cpt_source_idx;
        if cpt_idx >= 0 as ::core::ffi::c_int {
            startcol = (*(*cpt_sources_array.ptr()).offset(cpt_idx as isize)).cs_startcol;
            if (*compl_leader.ptr()).data.is_null() {
                if startcol < 0 as ::core::ffi::c_int || startcol >= compl_col.get() {
                    return compl_orig_text.ptr();
                }
                return compl_leader.ptr();
            }
            if compl_col.get() > 0 as ::core::ffi::c_int {
                if startcol >= 0 as ::core::ffi::c_int && startcol < compl_col.get() {
                    let mut prepend_len: ::core::ffi::c_int =
                        compl_col.get() as ::core::ffi::c_int - startcol;
                    let mut new_length: ::core::ffi::c_int =
                        prepend_len + (*compl_leader.ptr()).size as ::core::ffi::c_int;
                    if cached as ::core::ffi::c_int != 0
                        && new_length as size_t == (*adjusted_leader.ptr()).size
                        && !(*adjusted_leader.ptr()).data.is_null()
                    {
                        return adjusted_leader.ptr();
                    }
                    let mut ptr__0: *mut *mut ::core::ffi::c_void =
                        &raw mut (*adjusted_leader.ptr()).data as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr__0);
                    *ptr__0 = NULL;
                    let _ = *ptr__0;
                    (*adjusted_leader.ptr()).size = 0 as size_t;
                    prepend_startcol_text(adjusted_leader.ptr(), compl_leader.ptr(), startcol);
                    return adjusted_leader.ptr();
                }
            }
        }
    }
    return compl_leader.ptr();
}
unsafe extern "C" fn set_fuzzy_score() {
    if (*compl_first_match.ptr()).is_null() {
        return;
    }
    let mut use_leader: bool =
        !(*compl_leader.ptr()).data.is_null() && (*compl_leader.ptr()).size > 0 as size_t;
    let mut pattern: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !use_leader {
        if (*compl_orig_text.ptr()).data.is_null() || (*compl_orig_text.ptr()).size == 0 as size_t {
            return;
        }
        pattern = (*compl_orig_text.ptr()).data;
    } else {
        get_leader_for_startcol(::core::ptr::null_mut::<compl_T>(), true_0 != 0);
        pattern = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut comp: *mut compl_T = compl_first_match.get();
    loop {
        if use_leader {
            pattern = (*get_leader_for_startcol(comp, true_0 != 0)).data;
        }
        (*comp).cp_score = fuzzy_match_str((*comp).cp_str.data, pattern);
        comp = (*comp).cp_next;
        if !(!comp.is_null() && !is_first_match(comp)) {
            break;
        }
    }
}
unsafe extern "C" fn sort_compl_match_list(mut compare: MergeSortCompareFunc) {
    if (*compl_first_match.ptr()).is_null()
        || is_first_match((*compl_first_match.get()).cp_next) as ::core::ffi::c_int != 0
    {
        return;
    }
    let mut comp: *mut compl_T = (*compl_first_match.get()).cp_prev;
    ins_compl_make_linear();
    if compl_shows_dir_forward() {
        (*(*compl_first_match.get()).cp_next).cp_prev = ::core::ptr::null_mut::<compl_T>();
        (*compl_first_match.get()).cp_next = mergesort_list(
            (*compl_first_match.get()).cp_next as *mut ::core::ffi::c_void,
            Some(
                cp_get_next
                    as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void,
            ),
            Some(
                cp_set_next
                    as unsafe extern "C" fn(
                        *mut ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            Some(
                cp_get_prev
                    as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void,
            ),
            Some(
                cp_set_prev
                    as unsafe extern "C" fn(
                        *mut ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            compare,
        ) as *mut compl_T;
        (*(*compl_first_match.get()).cp_next).cp_prev = compl_first_match.get();
    } else {
        (*(*comp).cp_prev).cp_next = ::core::ptr::null_mut::<compl_T>();
        compl_first_match.set(mergesort_list(
            compl_first_match.get() as *mut ::core::ffi::c_void,
            Some(
                cp_get_next
                    as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void,
            ),
            Some(
                cp_set_next
                    as unsafe extern "C" fn(
                        *mut ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            Some(
                cp_get_prev
                    as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void,
            ),
            Some(
                cp_set_prev
                    as unsafe extern "C" fn(
                        *mut ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            compare,
        ) as *mut compl_T);
        let mut tail: *mut compl_T = compl_first_match.get();
        while !(*tail).cp_next.is_null() {
            tail = (*tail).cp_next;
        }
        (*tail).cp_next = comp;
        (*comp).cp_prev = tail;
    }
    ins_compl_make_cyclic();
}
unsafe extern "C" fn ins_compl_build_pum() -> ::core::ffi::c_int {
    compl_match_arraysize.set(0 as ::core::ffi::c_int);
    if ins_compl_need_restart() {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            compl_leader.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
    }
    let mut compl_no_select: bool = get_cot_flags()
        & kOptCotFlagNoselect as ::core::ffi::c_int as ::core::ffi::c_uint
        != 0 as ::core::ffi::c_uint
        || compl_autocomplete.get() as ::core::ffi::c_int != 0 && !ins_compl_has_preinsert();
    let mut match_head: *mut compl_T = ::core::ptr::null_mut::<compl_T>();
    let mut match_tail: *mut compl_T = ::core::ptr::null_mut::<compl_T>();
    let mut match_count: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
    let mut is_forward: bool = compl_shows_dir_forward();
    let mut is_cpt_completion: bool = !(*cpt_sources_array.ptr()).is_null();
    let mut shown_match_ok: bool = match_at_original_text(compl_shown_match.get());
    if strequal((*compl_leader.ptr()).data, (*compl_orig_text.ptr()).data) as ::core::ffi::c_int
        != 0
        && !shown_match_ok
    {
        compl_shown_match.set(if compl_no_select as ::core::ffi::c_int != 0 {
            compl_first_match.get()
        } else {
            (*compl_first_match.get()).cp_next
        });
    }
    let mut did_find_shown_match: bool = false_0 != 0;
    let mut comp: *mut compl_T = ::core::ptr::null_mut::<compl_T>();
    let mut shown_compl: *mut compl_T = ::core::ptr::null_mut::<compl_T>();
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cur: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if is_cpt_completion {
        match_count = xcalloc(
            cpt_sources_count.get() as size_t,
            ::core::mem::size_of::<::core::ffi::c_int>(),
        ) as *mut ::core::ffi::c_int;
    }
    get_leader_for_startcol(::core::ptr::null_mut::<compl_T>(), true_0 != 0);
    comp = compl_first_match.get();
    loop {
        (*comp).cp_in_match_array = false_0 != 0;
        let mut leader: *mut String_0 = get_leader_for_startcol(comp, true_0 != 0);
        if ctrl_x_mode_normal() as ::core::ffi::c_int != 0
            && p_inf.get() == 0
            && !(*leader).data.is_null()
            && ignorecase((*leader).data) == 0
            && !cot_fuzzy()
        {
            (*comp).cp_flags &= !(CP_ICASE as ::core::ffi::c_int);
        }
        if !match_at_original_text(comp)
            && ((*leader).data.is_null()
                || ins_compl_equal(comp, (*leader).data, (*leader).size) as ::core::ffi::c_int != 0
                || cot_fuzzy() as ::core::ffi::c_int != 0
                    && (*comp).cp_score != FUZZY_SCORE_NONE as ::core::ffi::c_int)
        {
            let mut match_limit_exceeded: bool = false_0 != 0;
            let mut cur_source: ::core::ffi::c_int = (*comp).cp_cpt_source_idx;
            if is_forward as ::core::ffi::c_int != 0
                && cur_source != -1 as ::core::ffi::c_int
                && is_cpt_completion as ::core::ffi::c_int != 0
            {
                *match_count.offset(cur_source as isize) += 1;
                let mut max_matches: ::core::ffi::c_int =
                    (*(*cpt_sources_array.ptr()).offset(cur_source as isize)).cs_max_matches;
                if max_matches > 0 as ::core::ffi::c_int
                    && *match_count.offset(cur_source as isize) > max_matches
                {
                    match_limit_exceeded = true_0 != 0;
                }
            }
            if !match_limit_exceeded {
                (*compl_match_arraysize.ptr()) += 1;
                (*comp).cp_in_match_array = true_0 != 0;
                if match_head.is_null() {
                    match_head = comp;
                } else {
                    (*match_tail).cp_match_next = comp;
                }
                match_tail = comp;
                if !shown_match_ok && !cot_fuzzy() {
                    if comp == compl_shown_match.get()
                        || did_find_shown_match as ::core::ffi::c_int != 0
                    {
                        compl_shown_match.set(comp);
                        did_find_shown_match = true_0 != 0;
                        shown_match_ok = true_0 != 0;
                    } else {
                        shown_compl = comp;
                    }
                    cur = i;
                } else if cot_fuzzy() {
                    if i == 0 as ::core::ffi::c_int {
                        shown_compl = comp;
                    }
                    if !shown_match_ok && comp == compl_shown_match.get() {
                        cur = i;
                        shown_match_ok = true_0 != 0;
                    }
                }
                i += 1;
            }
        }
        if comp == compl_shown_match.get() && !cot_fuzzy() {
            did_find_shown_match = true_0 != 0;
            if match_at_original_text(comp) {
                shown_match_ok = true_0 != 0;
            }
            if !shown_match_ok && !shown_compl.is_null() {
                compl_shown_match.set(shown_compl);
                shown_match_ok = true_0 != 0;
            }
        }
        comp = (*comp).cp_next;
        if !(!comp.is_null() && !is_first_match(comp)) {
            break;
        }
    }
    xfree(match_count as *mut ::core::ffi::c_void);
    if compl_match_arraysize.get() == 0 as ::core::ffi::c_int {
        return -1 as ::core::ffi::c_int;
    }
    if cot_fuzzy() as ::core::ffi::c_int != 0 && !compl_no_select && !shown_match_ok {
        compl_shown_match.set(shown_compl);
        shown_match_ok = true_0 != 0;
        cur = 0 as ::core::ffi::c_int;
    }
    '_c2rust_label: {
        if compl_match_arraysize.get() >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"compl_match_arraysize >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1663 as ::core::ffi::c_uint,
                b"int ins_compl_build_pum(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    compl_match_array.set(xcalloc(
        compl_match_arraysize.get() as size_t,
        ::core::mem::size_of::<pumitem_T>(),
    ) as *mut pumitem_T);
    i = 0 as ::core::ffi::c_int;
    comp = match_head;
    while !comp.is_null() {
        (*(*compl_match_array.ptr()).offset(i as isize)).pum_text =
            if !(*comp).cp_text[CPT_ABBR as ::core::ffi::c_int as usize].is_null() {
                (*comp).cp_text[CPT_ABBR as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char
            } else {
                (*comp).cp_str.data
            };
        (*(*compl_match_array.ptr()).offset(i as isize)).pum_kind =
            (*comp).cp_text[CPT_KIND as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char;
        (*(*compl_match_array.ptr()).offset(i as isize)).pum_info =
            (*comp).cp_text[CPT_INFO as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char;
        (*(*compl_match_array.ptr()).offset(i as isize)).pum_cpt_source_idx =
            (*comp).cp_cpt_source_idx;
        (*(*compl_match_array.ptr()).offset(i as isize)).pum_user_abbr_hlattr =
            (*comp).cp_user_abbr_hlattr;
        (*(*compl_match_array.ptr()).offset(i as isize)).pum_user_kind_hlattr =
            (*comp).cp_user_kind_hlattr;
        let c2rust_fresh2 = i;
        i = i + 1;
        let c2rust_lvalue_ptr =
            &raw mut (*(*compl_match_array.ptr()).offset(c2rust_fresh2 as isize)).pum_extra;
        *c2rust_lvalue_ptr = if !(*comp).cp_text[CPT_MENU as ::core::ffi::c_int as usize].is_null()
        {
            (*comp).cp_text[CPT_MENU as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char
        } else {
            (*comp).cp_fname
        };
        let mut match_next: *mut compl_T = (*comp).cp_match_next;
        (*comp).cp_match_next = ::core::ptr::null_mut::<compl_T>();
        comp = match_next;
    }
    if !shown_match_ok {
        cur = -1 as ::core::ffi::c_int;
    }
    return cur;
}
pub unsafe extern "C" fn ins_compl_show_pum() {
    if !pum_wanted() || !pum_enough_matches() {
        return;
    }
    update_screen();
    let mut cur: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut array_changed: bool = false_0 != 0;
    if (*compl_match_array.ptr()).is_null() {
        array_changed = true_0 != 0;
        cur = ins_compl_build_pum();
    } else {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < compl_match_arraysize.get() {
            if (*(*compl_match_array.ptr()).offset(i as isize)).pum_text
                == (*compl_shown_match.get()).cp_str.data
                || (*(*compl_match_array.ptr()).offset(i as isize)).pum_text
                    == (*compl_shown_match.get()).cp_text[CPT_ABBR as ::core::ffi::c_int as usize]
            {
                cur = i;
                break;
            } else {
                i += 1;
            }
        }
    }
    if (*compl_match_array.ptr()).is_null() {
        if compl_started.get() as ::core::ffi::c_int != 0
            && has_event(EVENT_COMPLETECHANGED) as ::core::ffi::c_int != 0
        {
            trigger_complete_changed_event(cur);
        }
        return;
    }
    dollar_vcol.set(-1 as ::core::ffi::c_int as colnr_T);
    let col: colnr_T = (*curwin.get()).w_cursor.col;
    (*curwin.get()).w_cursor.col = compl_col.get();
    compl_selected_item.set(cur);
    pum_display(
        compl_match_array.get(),
        compl_match_arraysize.get(),
        cur,
        array_changed,
        0 as ::core::ffi::c_int,
    );
    (*curwin.get()).w_cursor.col = col;
    if compl_started.get() as ::core::ffi::c_int != 0
        && compl_curr_match.get() != compl_shown_match.get()
    {
        compl_curr_match.set(compl_shown_match.get());
    }
    if has_event(EVENT_COMPLETECHANGED) {
        trigger_complete_changed_event(cur);
    }
}
pub unsafe extern "C" fn compl_match_curr_select(mut selected: ::core::ffi::c_int) -> bool {
    if selected < 0 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    let mut match_0: *mut compl_T = compl_first_match.get();
    let mut selected_idx: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut list_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        if !match_at_original_text(match_0) {
            if !(*compl_curr_match.ptr()).is_null()
                && (*compl_curr_match.get()).cp_number == (*match_0).cp_number
            {
                selected_idx = list_idx;
                break;
            } else {
                list_idx += 1 as ::core::ffi::c_int;
            }
        }
        match_0 = (*match_0).cp_next;
        if !(!match_0.is_null() && !is_first_match(match_0)) {
            break;
        }
    }
    return selected == selected_idx;
}
pub const DICT_FIRST: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const DICT_EXACT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
unsafe extern "C" fn ins_compl_dictionaries(
    mut dict_start: *mut ::core::ffi::c_char,
    mut pat: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut thesaurus: bool,
) {
    let mut dict: *mut ::core::ffi::c_char = dict_start;
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut files: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut count: ::core::ffi::c_int = 0;
    let mut dir: Direction = compl_direction.get();
    if *dict as ::core::ffi::c_int == NUL {
        if !thesaurus && (*curwin.get()).w_onebuf_opt.wo_spell != 0 {
            dict = b"spell\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            return;
        }
    }
    let mut buf: *mut ::core::ffi::c_char =
        xmalloc(LSIZE as ::core::ffi::c_int as size_t) as *mut ::core::ffi::c_char;
    regmatch.regprog = ::core::ptr::null_mut::<regprog_T>();
    let mut save_p_scs: ::core::ffi::c_int = p_scs.get();
    if (*curbuf.get()).b_p_inf != 0 {
        p_scs.set(false_0);
    }
    '_theend: {
        if ctrl_x_mode_line_or_eval() {
            let mut pat_esc: *mut ::core::ffi::c_char =
                vim_strsave_escaped(pat, b"\\\0".as_ptr() as *const ::core::ffi::c_char);
            let mut len: size_t = strlen(pat_esc).wrapping_add(10 as size_t);
            ptr = xmalloc(len) as *mut ::core::ffi::c_char;
            vim_snprintf(
                ptr,
                len,
                b"^\\s*\\zs\\V%s\0".as_ptr() as *const ::core::ffi::c_char,
                pat_esc,
            );
            regmatch.regprog = vim_regcomp(ptr, RE_MAGIC);
            xfree(pat_esc as *mut ::core::ffi::c_void);
            xfree(ptr as *mut ::core::ffi::c_void);
        } else {
            regmatch.regprog = vim_regcomp(
                pat,
                if magic_isset() as ::core::ffi::c_int != 0 {
                    RE_MAGIC
                } else {
                    0 as ::core::ffi::c_int
                },
            );
            if regmatch.regprog.is_null() {
                break '_theend;
            }
        }
        regmatch.rm_ic = ignorecase(pat) != 0;
        while *dict as ::core::ffi::c_int != NUL && !got_int.get() && !compl_interrupted.get() {
            if flags == DICT_EXACT {
                count = 1 as ::core::ffi::c_int;
                files = &raw mut dict;
            } else {
                copy_option_part(
                    &raw mut dict,
                    buf,
                    LSIZE as ::core::ffi::c_int as size_t,
                    b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
                if !thesaurus
                    && strcmp(buf, b"spell\0".as_ptr() as *const ::core::ffi::c_char)
                        == 0 as ::core::ffi::c_int
                {
                    count = -1 as ::core::ffi::c_int;
                } else if !vim_strchr(buf, '`' as ::core::ffi::c_int).is_null()
                    || expand_wildcards(
                        1 as ::core::ffi::c_int,
                        &raw mut buf,
                        &raw mut count,
                        &raw mut files,
                        EW_FILE as ::core::ffi::c_int | EW_SILENT as ::core::ffi::c_int,
                    ) != OK
                {
                    count = 0 as ::core::ffi::c_int;
                }
            }
            if count == -1 as ::core::ffi::c_int {
                if *pat.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
                    && *pat.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '<' as ::core::ffi::c_int
                {
                    ptr = pat.offset(2 as ::core::ffi::c_int as isize);
                } else {
                    ptr = pat;
                }
                spell_dump_compl(
                    ptr,
                    regmatch.rm_ic as ::core::ffi::c_int,
                    &raw mut dir,
                    0 as ::core::ffi::c_int,
                );
            } else if count > 0 as ::core::ffi::c_int {
                ins_compl_files(
                    count,
                    files,
                    thesaurus,
                    flags,
                    &raw mut regmatch,
                    buf,
                    &raw mut dir,
                );
                if flags != DICT_EXACT {
                    FreeWild(count, files);
                }
            }
            if flags != 0 as ::core::ffi::c_int {
                break;
            }
        }
    }
    p_scs.set(save_p_scs);
    vim_regfree(regmatch.regprog);
    xfree(buf as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn thesaurus_add_words_in_line(
    mut fname: *mut ::core::ffi::c_char,
    mut buf_arg: *mut *mut ::core::ffi::c_char,
    mut dir: ::core::ffi::c_int,
    mut skip_word: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut status: ::core::ffi::c_int = OK;
    let mut ptr: *mut ::core::ffi::c_char = *buf_arg;
    while !got_int.get() {
        ptr = find_word_start(ptr);
        if *ptr as ::core::ffi::c_int == NUL || *ptr as ::core::ffi::c_int == NL {
            break;
        }
        let mut wstart: *mut ::core::ffi::c_char = ptr;
        while *ptr as ::core::ffi::c_int != NUL {
            let l: ::core::ffi::c_int = utfc_ptr2len(ptr);
            if l < 2 as ::core::ffi::c_int && !vim_iswordc(*ptr as uint8_t as ::core::ffi::c_int) {
                break;
            }
            ptr = ptr.offset(l as isize);
        }
        if wstart == skip_word as *mut ::core::ffi::c_char {
            continue;
        }
        status = ins_compl_add_infercase(
            wstart,
            ptr.offset_from(wstart) as ::core::ffi::c_int,
            p_ic.get() != 0,
            fname,
            dir as Direction,
            false_0 != 0,
            FUZZY_SCORE_NONE as ::core::ffi::c_int,
        );
        if status == FAIL {
            break;
        }
    }
    *buf_arg = ptr;
    return status;
}
unsafe extern "C" fn ins_compl_files(
    mut count: ::core::ffi::c_int,
    mut files: *mut *mut ::core::ffi::c_char,
    mut thesaurus: bool,
    mut flags: ::core::ffi::c_int,
    mut regmatch: *mut regmatch_T,
    mut buf: *mut ::core::ffi::c_char,
    mut dir: *mut Direction,
) {
    let mut leader: *mut ::core::ffi::c_char = if cot_fuzzy() as ::core::ffi::c_int != 0 {
        ins_compl_leader()
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
    let mut leader_len: ::core::ffi::c_int = if cot_fuzzy() as ::core::ffi::c_int != 0 {
        ins_compl_leader_len() as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < count && !got_int.get() && !ins_compl_interrupted() {
        let mut fp: *mut FILE = os_fopen(
            *files.offset(i as isize),
            b"r\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if flags != DICT_EXACT
            && !shortmess(SHM_COMPLETIONSCAN as ::core::ffi::c_int)
            && !compl_autocomplete.get()
        {
            vim_snprintf(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                gettext(b"Scanning dictionary: %s\0".as_ptr() as *const ::core::ffi::c_char),
                *files.offset(i as isize),
            );
            msg_progress(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                b"completion\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"running\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                HLF_R as ::core::ffi::c_int,
                false_0 != 0,
                true_0 != 0,
            );
        }
        if !fp.is_null() {
            while !got_int.get()
                && !ins_compl_interrupted()
                && !vim_fgets(buf, LSIZE as ::core::ffi::c_int, fp)
            {
                let mut ptr: *mut ::core::ffi::c_char = buf;
                if cot_fuzzy() as ::core::ffi::c_int != 0 && leader_len > 0 as ::core::ffi::c_int {
                    let mut line_end: *mut ::core::ffi::c_char = find_line_end(ptr);
                    while ptr < line_end {
                        let mut score: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        if !fuzzy_match_str_in_line(
                            &raw mut ptr,
                            leader,
                            &raw mut len,
                            ::core::ptr::null_mut::<pos_T>(),
                            &raw mut score,
                        ) {
                            continue;
                        }
                        let mut end_ptr: *mut ::core::ffi::c_char =
                            if ctrl_x_mode_line_or_eval() as ::core::ffi::c_int != 0 {
                                find_line_end(ptr)
                            } else {
                                find_word_end(ptr)
                            };
                        let mut add_r: ::core::ffi::c_int = ins_compl_add_infercase(
                            ptr,
                            end_ptr.offset_from(ptr) as ::core::ffi::c_int,
                            p_ic.get() != 0,
                            *files.offset(i as isize),
                            *dir,
                            false_0 != 0,
                            score,
                        );
                        if add_r == FAIL {
                            break;
                        }
                        ptr = end_ptr;
                        if compl_get_longest.get() as ::core::ffi::c_int != 0
                            && ctrl_x_mode_normal() as ::core::ffi::c_int != 0
                            && !(*compl_first_match.get()).cp_next.is_null()
                            && score == (*(*compl_first_match.get()).cp_next).cp_score
                        {
                            (*compl_num_bests.ptr()) += 1;
                        }
                    }
                } else if !regmatch.is_null() {
                    while vim_regexec(regmatch, buf, ptr.offset_from(buf) as colnr_T) {
                        ptr = (*regmatch).startp[0 as ::core::ffi::c_int as usize];
                        ptr = if ctrl_x_mode_line_or_eval() as ::core::ffi::c_int != 0 {
                            find_line_end(ptr)
                        } else {
                            find_word_end(ptr)
                        };
                        let mut add_r_0: ::core::ffi::c_int = ins_compl_add_infercase(
                            (*regmatch).startp[0 as ::core::ffi::c_int as usize],
                            ptr.offset_from((*regmatch).startp[0 as ::core::ffi::c_int as usize])
                                as ::core::ffi::c_int,
                            p_ic.get() != 0,
                            *files.offset(i as isize),
                            *dir,
                            false_0 != 0,
                            FUZZY_SCORE_NONE as ::core::ffi::c_int,
                        );
                        if thesaurus {
                            ptr = buf;
                            add_r_0 = thesaurus_add_words_in_line(
                                *files.offset(i as isize),
                                &raw mut ptr,
                                *dir as ::core::ffi::c_int,
                                (*regmatch).startp[0 as ::core::ffi::c_int as usize],
                            );
                        }
                        if add_r_0 == OK {
                            *dir = FORWARD;
                        } else if add_r_0 == FAIL {
                            break;
                        }
                        if *ptr as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
                            || got_int.get() as ::core::ffi::c_int != 0
                        {
                            break;
                        }
                    }
                }
                line_breakcheck();
                ins_compl_check_keys(50 as ::core::ffi::c_int, false_0 != 0);
            }
            fclose(fp);
        }
        i += 1;
    }
}
pub unsafe extern "C" fn find_word_start(
    mut ptr: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    while *ptr as ::core::ffi::c_int != NUL
        && *ptr as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
        && mb_get_class(ptr) <= 1 as ::core::ffi::c_int
    {
        ptr = ptr.offset(utfc_ptr2len(ptr) as isize);
    }
    return ptr;
}
pub unsafe extern "C" fn find_word_end(
    mut ptr: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let start_class: ::core::ffi::c_int = mb_get_class(ptr);
    if start_class > 1 as ::core::ffi::c_int {
        while *ptr as ::core::ffi::c_int != NUL {
            ptr = ptr.offset(utfc_ptr2len(ptr) as isize);
            if mb_get_class(ptr) != start_class {
                break;
            }
        }
    }
    return ptr;
}
pub unsafe extern "C" fn find_line_end(
    mut ptr: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut s: *mut ::core::ffi::c_char = ptr.offset(strlen(ptr) as isize);
    while s > ptr
        && (*s.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == CAR
            || *s.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NL)
    {
        s = s.offset(-1);
    }
    return s;
}
unsafe extern "C" fn ins_compl_item_free(mut match_0: *mut compl_T) {
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*match_0).cp_str.data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    (*match_0).cp_str.size = 0 as size_t;
    if (*match_0).cp_flags & CP_FREE_FNAME as ::core::ffi::c_int != 0 {
        xfree((*match_0).cp_fname as *mut ::core::ffi::c_void);
    }
    free_cptext(&raw mut (*match_0).cp_text as *mut *mut ::core::ffi::c_char);
    tv_clear(&raw mut (*match_0).cp_user_data);
    xfree(match_0 as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn ins_compl_free() {
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*compl_pattern.ptr()).data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    (*compl_pattern.ptr()).size = 0 as size_t;
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        &raw mut (*compl_leader.ptr()).data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL;
    let _ = *ptr__0;
    (*compl_leader.ptr()).size = 0 as size_t;
    if (*compl_first_match.ptr()).is_null() {
        return;
    }
    ins_compl_del_pum();
    pum_clear();
    compl_curr_match.set(compl_first_match.get());
    loop {
        let mut match_0: *mut compl_T = compl_curr_match.get();
        compl_curr_match.set((*compl_curr_match.get()).cp_next);
        ins_compl_item_free(match_0);
        if !(!(*compl_curr_match.ptr()).is_null() && !is_first_match(compl_curr_match.get())) {
            break;
        }
    }
    compl_curr_match.set(::core::ptr::null_mut::<compl_T>());
    compl_first_match.set(compl_curr_match.get());
    compl_shown_match.set(::core::ptr::null_mut::<compl_T>());
    compl_old_match.set(::core::ptr::null_mut::<compl_T>());
}
pub unsafe extern "C" fn ins_compl_clear() {
    compl_cont_status.set(0 as ::core::ffi::c_int);
    compl_started.set(false_0 != 0);
    compl_matches.set(0 as ::core::ffi::c_int);
    compl_selected_item.set(-1 as ::core::ffi::c_int);
    compl_ins_end_col.set(0 as ::core::ffi::c_int as colnr_T);
    compl_curr_win.set(::core::ptr::null_mut::<win_T>());
    compl_curr_buf.set(::core::ptr::null_mut::<buf_T>());
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*compl_pattern.ptr()).data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    (*compl_pattern.ptr()).size = 0 as size_t;
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        &raw mut (*compl_leader.ptr()).data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL;
    let _ = *ptr__0;
    (*compl_leader.ptr()).size = 0 as size_t;
    edit_submode_extra.set(::core::ptr::null_mut::<::core::ffi::c_char>());
    xfree((*compl_orig_extmarks.ptr()).items as *mut ::core::ffi::c_void);
    (*compl_orig_extmarks.ptr()).capacity = 0 as size_t;
    (*compl_orig_extmarks.ptr()).size = (*compl_orig_extmarks.ptr()).capacity;
    (*compl_orig_extmarks.ptr()).items = ::core::ptr::null_mut::<ExtmarkUndoObject>();
    let mut ptr__1: *mut *mut ::core::ffi::c_void =
        &raw mut (*compl_orig_text.ptr()).data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__1);
    *ptr__1 = NULL;
    let _ = *ptr__1;
    (*compl_orig_text.ptr()).size = 0 as size_t;
    compl_enter_selects.set(false_0 != 0);
    cpt_sources_clear();
    compl_autocomplete.set(false_0 != 0);
    compl_from_nonkeyword.set(false_0 != 0);
    compl_num_bests.set(0 as ::core::ffi::c_int);
    set_vim_var_dict(VV_COMPLETED_ITEM, tv_dict_alloc_lock(VAR_FIXED));
}
pub unsafe extern "C" fn ins_compl_active() -> bool {
    return compl_started.get();
}
pub unsafe extern "C" fn ins_compl_win_active(mut wp: *mut win_T) -> bool {
    return ins_compl_active() as ::core::ffi::c_int != 0
        && wp == compl_curr_win.get()
        && (*wp).w_buffer == compl_curr_buf.get();
}
pub unsafe extern "C" fn ins_compl_used_match() -> bool {
    return compl_used_match.get();
}
pub unsafe extern "C" fn ins_compl_init_get_longest() {
    compl_get_longest.set(false_0 != 0);
}
pub unsafe extern "C" fn ins_compl_interrupted() -> bool {
    return compl_interrupted.get() as ::core::ffi::c_int != 0
        || compl_time_slice_expired.get() as ::core::ffi::c_int != 0;
}
pub unsafe extern "C" fn ins_compl_enter_selects() -> bool {
    return compl_enter_selects.get();
}
pub unsafe extern "C" fn ins_compl_col() -> colnr_T {
    return compl_col.get();
}
pub unsafe extern "C" fn ins_compl_len() -> ::core::ffi::c_int {
    return compl_length.get();
}
pub unsafe extern "C" fn ins_compl_has_preinsert() -> bool {
    let mut cur_cot_flags: ::core::ffi::c_uint = get_cot_flags();
    if compl_autocomplete.get() as ::core::ffi::c_int != 0 && p_ic.get() != 0 && p_inf.get() == 0 {
        return false_0 != 0;
    }
    return if !compl_autocomplete.get() {
        (cur_cot_flags
            & (kOptCotFlagPreinsert as ::core::ffi::c_int
                | kOptCotFlagFuzzy as ::core::ffi::c_int
                | kOptCotFlagMenuone as ::core::ffi::c_int) as ::core::ffi::c_uint
            == (kOptCotFlagPreinsert as ::core::ffi::c_int
                | kOptCotFlagMenuone as ::core::ffi::c_int) as ::core::ffi::c_uint)
            as ::core::ffi::c_int
    } else {
        (cur_cot_flags
            & (kOptCotFlagPreinsert as ::core::ffi::c_int | kOptCotFlagFuzzy as ::core::ffi::c_int)
                as ::core::ffi::c_uint
            == kOptCotFlagPreinsert as ::core::ffi::c_int as ::core::ffi::c_uint)
            as ::core::ffi::c_int
    } != 0;
}
pub unsafe extern "C" fn ins_compl_preinsert_effect() -> bool {
    if !ins_compl_has_preinsert() && !ins_compl_preinsert_longest() {
        return false_0 != 0;
    }
    return (*curwin.get()).w_cursor.col < compl_ins_end_col.get();
}
pub unsafe extern "C" fn ins_compl_bs() -> ::core::ffi::c_int {
    if ins_compl_preinsert_effect() {
        ins_compl_delete(false_0 != 0);
    }
    let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
    let mut p: *mut ::core::ffi::c_char = line.offset((*curwin.get()).w_cursor.col as isize);
    p = p.offset(
        -((utf_head_off(line, p.offset(-(1 as ::core::ffi::c_int as isize)))
            + 1 as ::core::ffi::c_int) as isize),
    );
    let mut p_off: ptrdiff_t = p.offset_from(line);
    if p.offset_from(line) as ::core::ffi::c_int - compl_col.get() < 0 as ::core::ffi::c_int
        || p.offset_from(line) as ::core::ffi::c_int - compl_col.get() == 0 as ::core::ffi::c_int
            && !ctrl_x_mode_omni()
        || ctrl_x_mode_eval() as ::core::ffi::c_int != 0
        || !can_bs(BS_START)
            && p.offset_from(line) as ::core::ffi::c_int - compl_col.get() - compl_length.get()
                < 0 as ::core::ffi::c_int
    {
        return K_BS;
    }
    if (*curwin.get()).w_cursor.col <= compl_col.get() as ::core::ffi::c_int + compl_length.get()
        || ins_compl_need_restart() as ::core::ffi::c_int != 0
    {
        ins_compl_restart();
    }
    line = get_cursor_line_ptr();
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*compl_leader.ptr()).data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    (*compl_leader.ptr()).size = 0 as size_t;
    compl_leader.set(cbuf_to_string(
        line.offset(compl_col.get() as isize),
        (p_off - compl_col.get() as ptrdiff_t) as size_t,
    ));
    if compl_autocomplete.get() as ::core::ffi::c_int != 0
        && !(*compl_first_match.ptr()).is_null()
        && !ins_compl_has_preinsert()
    {
        compl_shown_match.set(compl_first_match.get());
    }
    ins_compl_new_leader();
    if !(*compl_shown_match.ptr()).is_null() {
        compl_curr_match.set(compl_shown_match.get());
    }
    return NUL;
}
unsafe extern "C" fn ins_compl_refresh_always() -> bool {
    return (ctrl_x_mode_function() as ::core::ffi::c_int != 0
        || ctrl_x_mode_omni() as ::core::ffi::c_int != 0)
        && compl_opt_refresh_always.get() as ::core::ffi::c_int != 0;
}
unsafe extern "C" fn ins_compl_need_restart() -> bool {
    return compl_was_interrupted.get() as ::core::ffi::c_int != 0
        || ins_compl_refresh_always() as ::core::ffi::c_int != 0;
}
pub unsafe extern "C" fn ins_compl_has_autocomplete() -> bool {
    return if (*curbuf.get()).b_p_ac >= 0 as ::core::ffi::c_int {
        (*curbuf.get()).b_p_ac
    } else {
        p_ac.get()
    } != 0;
}
unsafe extern "C" fn ins_compl_fuzzy_sort() {
    let mut cur_cot_flags: ::core::ffi::c_uint = get_cot_flags();
    set_fuzzy_score();
    if cur_cot_flags & kOptCotFlagNosort as ::core::ffi::c_int as ::core::ffi::c_uint == 0 {
        sort_compl_match_list(Some(
            cp_compare_fuzzy
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ));
        if cur_cot_flags
            & (kOptCotFlagNoinsert as ::core::ffi::c_int
                | kOptCotFlagNoselect as ::core::ffi::c_int) as ::core::ffi::c_uint
            == kOptCotFlagNoinsert as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut none_selected: bool = compl_shown_match.get()
                == (if compl_shows_dir_forward() as ::core::ffi::c_int != 0 {
                    compl_first_match.get()
                } else {
                    (*compl_first_match.get()).cp_prev
                });
            if !none_selected {
                compl_shown_match.set(
                    if !compl_autocomplete.get()
                        && compl_shows_dir_forward() as ::core::ffi::c_int != 0
                    {
                        (*compl_first_match.get()).cp_next
                    } else {
                        compl_first_match.get()
                    },
                );
            }
        }
    }
}
unsafe extern "C" fn ins_compl_new_leader() {
    ins_compl_del_pum();
    ins_compl_delete(true_0 != 0);
    ins_compl_insert_bytes(
        (*compl_leader.ptr()).data.offset(get_compl_len() as isize),
        -1 as ::core::ffi::c_int,
    );
    compl_used_match.set(false_0 != 0);
    if p_acl.get() > 0 as OptInt {
        pum_undisplay(true_0 != 0);
        redraw_later(curwin.get(), UPD_VALID as ::core::ffi::c_int);
        update_screen();
        ui_flush();
    }
    if compl_started.get() {
        ins_compl_set_original_text((*compl_leader.ptr()).data, (*compl_leader.ptr()).size);
        if is_cpt_func_refresh_always() {
            cpt_compl_refresh();
        }
        if cot_fuzzy() {
            ins_compl_fuzzy_sort();
        }
    } else {
        spell_bad_len.set(0 as size_t);
        compl_restarting.set(true_0 != 0);
        if ins_compl_has_autocomplete() {
            ins_compl_enable_autocomplete();
        } else {
            compl_autocomplete.set(false_0 != 0);
        }
        if ins_complete(Ctrl_N, true_0 != 0) == FAIL {
            compl_cont_status.set(0 as ::core::ffi::c_int);
        }
        compl_restarting.set(false_0 != 0);
    }
    compl_enter_selects
        .set(!compl_used_match.get() && compl_selected_item.get() != -1 as ::core::ffi::c_int);
    ins_compl_show_pum();
    if (*compl_match_array.ptr()).is_null() {
        compl_enter_selects.set(false_0 != 0);
    } else if ins_compl_has_preinsert() as ::core::ffi::c_int != 0
        && (*compl_leader.ptr()).size > 0 as size_t
    {
        ins_compl_insert(true_0 != 0, false_0 != 0);
    } else if compl_started.get() as ::core::ffi::c_int != 0
        && ins_compl_preinsert_longest() as ::core::ffi::c_int != 0
        && (*compl_leader.ptr()).size > 0 as size_t
        && !ins_compl_preinsert_effect()
    {
        ins_compl_insert(true_0 != 0, true_0 != 0);
    }
    if ins_compl_refresh_always() {
        compl_enter_selects.set(false_0 != 0);
    }
}
unsafe extern "C" fn get_compl_len() -> ::core::ffi::c_int {
    let mut off: ::core::ffi::c_int = (*curwin.get()).w_cursor.col - compl_col.get();
    return if 0 as ::core::ffi::c_int > off {
        0 as ::core::ffi::c_int
    } else {
        off
    };
}
pub unsafe extern "C" fn ins_compl_addleader(mut c: ::core::ffi::c_int) {
    let mut cc: ::core::ffi::c_int = 0;
    if ins_compl_preinsert_effect() {
        ins_compl_delete(false_0 != 0);
    }
    if stop_arrow() == FAIL {
        return;
    }
    cc = utf_char2len(c);
    if cc > 1 as ::core::ffi::c_int {
        let mut buf: [::core::ffi::c_char; 7] = [0; 7];
        utf_char2bytes(c, &raw mut buf as *mut ::core::ffi::c_char);
        buf[cc as usize] = NUL as ::core::ffi::c_char;
        ins_char_bytes(&raw mut buf as *mut ::core::ffi::c_char, cc as size_t);
    } else {
        ins_char(c);
    }
    if ins_compl_need_restart() {
        ins_compl_restart();
    }
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*compl_leader.ptr()).data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    (*compl_leader.ptr()).size = 0 as size_t;
    compl_leader.set(cbuf_to_string(
        get_cursor_line_ptr().offset(compl_col.get() as isize),
        ((*curwin.get()).w_cursor.col - compl_col.get()) as size_t,
    ));
    ins_compl_new_leader();
}
unsafe extern "C" fn ins_compl_restart() {
    update_screen();
    ins_compl_free();
    compl_started.set(false_0 != 0);
    compl_matches.set(0 as ::core::ffi::c_int);
    compl_cont_status.set(0 as ::core::ffi::c_int);
    compl_cont_mode.set(0 as ::core::ffi::c_int);
    cpt_sources_clear();
    compl_autocomplete.set(false_0 != 0);
    compl_from_nonkeyword.set(false_0 != 0);
    compl_num_bests.set(0 as ::core::ffi::c_int);
}
unsafe extern "C" fn ins_compl_set_original_text(
    mut str: *mut ::core::ffi::c_char,
    mut len: size_t,
) {
    if match_at_original_text(compl_first_match.get()) {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*compl_first_match.get()).cp_str.data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        (*compl_first_match.get()).cp_str.size = 0 as size_t;
        (*compl_first_match.get()).cp_str = cbuf_to_string(str, len);
    } else if !(*compl_first_match.get()).cp_prev.is_null()
        && match_at_original_text((*compl_first_match.get()).cp_prev) as ::core::ffi::c_int != 0
    {
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut (*(*compl_first_match.get()).cp_prev).cp_str.data
                as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL;
        let _ = *ptr__0;
        (*(*compl_first_match.get()).cp_prev).cp_str.size = 0 as size_t;
        (*(*compl_first_match.get()).cp_prev).cp_str = cbuf_to_string(str, len);
    }
}
pub unsafe extern "C" fn ins_compl_addfrommatch() {
    let mut len: ::core::ffi::c_int = (*curwin.get()).w_cursor.col - compl_col.get();
    '_c2rust_label: {
        if !(*compl_shown_match.ptr()).is_null() {
        } else {
            __assert_fail(
                b"compl_shown_match != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2421 as ::core::ffi::c_uint,
                b"void ins_compl_addfrommatch(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut p: *mut ::core::ffi::c_char = (*compl_shown_match.get()).cp_str.data;
    if (*compl_shown_match.get()).cp_str.size as ::core::ffi::c_int <= len {
        if !match_at_original_text(compl_shown_match.get()) {
            return;
        }
        p = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut plen: size_t = 0 as size_t;
        let mut cp: *mut compl_T = (*compl_shown_match.get()).cp_next;
        while !cp.is_null() && !is_first_match(cp) {
            if (*compl_leader.ptr()).data.is_null()
                || ins_compl_equal(cp, (*compl_leader.ptr()).data, (*compl_leader.ptr()).size)
                    as ::core::ffi::c_int
                    != 0
            {
                p = (*cp).cp_str.data;
                plen = (*cp).cp_str.size;
                break;
            } else {
                cp = (*cp).cp_next;
            }
        }
        if p.is_null() || plen as ::core::ffi::c_int <= len {
            return;
        }
    }
    p = p.offset(len as isize);
    let mut c: ::core::ffi::c_int = utf_ptr2char(p);
    ins_compl_addleader(c);
}
unsafe extern "C" fn set_ctrl_x_mode(c: ::core::ffi::c_int) -> bool {
    let mut retval: bool = false_0 != 0;
    's_241: {
        match c {
            Ctrl_E | Ctrl_Y => {
                ctrl_x_mode.set(CTRL_X_SCROLL as ::core::ffi::c_int);
                if State.get() & REPLACE_FLAG as ::core::ffi::c_int == 0 {
                    edit_submode.set(gettext(
                        b" (insert) Scroll (^E/^Y)\0".as_ptr() as *const ::core::ffi::c_char
                    ));
                } else {
                    edit_submode.set(gettext(
                        b" (replace) Scroll (^E/^Y)\0".as_ptr() as *const ::core::ffi::c_char
                    ));
                }
                edit_submode_pre.set(::core::ptr::null_mut::<::core::ffi::c_char>());
                redraw_mode.set(true_0 != 0);
                break 's_241;
            }
            Ctrl_L => {
                ctrl_x_mode.set(CTRL_X_WHOLE_LINE as ::core::ffi::c_int);
                break 's_241;
            }
            Ctrl_F => {
                ctrl_x_mode.set(CTRL_X_FILES as ::core::ffi::c_int);
                break 's_241;
            }
            Ctrl_K => {
                ctrl_x_mode.set(CTRL_X_DICTIONARY as ::core::ffi::c_int);
                break 's_241;
            }
            Ctrl_R => {
                if vpeekc() == '=' as ::core::ffi::c_int {
                    break 's_241;
                } else {
                    ctrl_x_mode.set(CTRL_X_REGISTER as ::core::ffi::c_int);
                    break 's_241;
                }
            }
            Ctrl_T => {
                ctrl_x_mode.set(CTRL_X_THESAURUS as ::core::ffi::c_int);
                break 's_241;
            }
            Ctrl_U => {
                ctrl_x_mode.set(CTRL_X_FUNCTION as ::core::ffi::c_int);
                break 's_241;
            }
            Ctrl_O => {
                ctrl_x_mode.set(CTRL_X_OMNI as ::core::ffi::c_int);
                break 's_241;
            }
            115 | Ctrl_S => {
                ctrl_x_mode.set(CTRL_X_SPELL as ::core::ffi::c_int);
                (*emsg_off.ptr()) += 1;
                spell_back_to_badword();
                (*emsg_off.ptr()) -= 1;
                break 's_241;
            }
            Ctrl_RSB => {
                ctrl_x_mode.set(CTRL_X_TAGS as ::core::ffi::c_int);
                break 's_241;
            }
            Ctrl_I | K_S_TAB => {
                ctrl_x_mode.set(CTRL_X_PATH_PATTERNS as ::core::ffi::c_int);
                break 's_241;
            }
            Ctrl_D => {
                ctrl_x_mode.set(CTRL_X_PATH_DEFINES as ::core::ffi::c_int);
                break 's_241;
            }
            Ctrl_V | Ctrl_Q => {
                ctrl_x_mode.set(CTRL_X_CMDLINE as ::core::ffi::c_int);
                break 's_241;
            }
            Ctrl_Z => {
                ctrl_x_mode.set(CTRL_X_NORMAL as ::core::ffi::c_int);
                edit_submode.set(::core::ptr::null_mut::<::core::ffi::c_char>());
                redraw_mode.set(true_0 != 0);
                retval = true_0 != 0;
                break 's_241;
            }
            Ctrl_P | Ctrl_N => {
                if compl_cont_status.get() & CONT_INTRPT == 0 {
                    (*compl_cont_status.ptr()) |= CONT_LOCAL;
                } else if compl_cont_mode.get() != 0 as ::core::ffi::c_int {
                    (*compl_cont_status.ptr()) &= !CONT_LOCAL;
                }
            }
            _ => {}
        }
        if c == Ctrl_X {
            if compl_cont_mode.get() != 0 as ::core::ffi::c_int {
                compl_cont_status.set(0 as ::core::ffi::c_int);
            } else {
                compl_cont_mode.set(CTRL_X_NOT_DEFINED_YET as ::core::ffi::c_int);
            }
        }
        ctrl_x_mode.set(CTRL_X_NORMAL as ::core::ffi::c_int);
        edit_submode.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        redraw_mode.set(true_0 != 0);
    }
    return retval;
}
unsafe extern "C" fn ins_compl_stop(
    c: ::core::ffi::c_int,
    prev_mode: ::core::ffi::c_int,
    mut retval: bool,
) -> bool {
    if ins_compl_preinsert_effect() as ::core::ffi::c_int != 0
        && ins_compl_win_active(curwin.get()) as ::core::ffi::c_int != 0
    {
        ins_compl_delete(false_0 != 0);
    }
    if !(*compl_curr_match.ptr()).is_null() || !(*compl_leader.ptr()).data.is_null() || c == Ctrl_E
    {
        let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if !(*compl_curr_match.ptr()).is_null()
            && compl_used_match.get() as ::core::ffi::c_int != 0
            && c != Ctrl_E
        {
            ptr = (*compl_curr_match.get()).cp_str.data;
        }
        ins_compl_fixRedoBufForLeader(ptr);
    }
    let mut want_cindent: bool =
        get_can_cindent() as ::core::ffi::c_int != 0 && cindent_on() as ::core::ffi::c_int != 0;
    if compl_cont_mode.get() == CTRL_X_WHOLE_LINE as ::core::ffi::c_int {
        if want_cindent {
            do_c_expr_indent();
            want_cindent = false_0 != 0;
        }
    } else if !compl_autocomplete.get() || compl_used_match.get() as ::core::ffi::c_int != 0 {
        let prev_col: ::core::ffi::c_int = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
        if prev_col > 0 as ::core::ffi::c_int {
            dec_cursor();
        }
        if !arrow_used.get() && !ins_need_undo_get() && c != Ctrl_E {
            insertchar(NUL, 0 as ::core::ffi::c_int, -1 as ::core::ffi::c_int);
        }
        if prev_col > 0 as ::core::ffi::c_int
            && *get_cursor_line_ptr().offset((*curwin.get()).w_cursor.col as isize)
                as ::core::ffi::c_int
                != NUL
        {
            inc_cursor();
        }
    }
    let mut word: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (c == Ctrl_Y
        || compl_enter_selects.get() as ::core::ffi::c_int != 0
            && (c == CAR || c == K_KENTER || c == NL))
        && pum_visible() as ::core::ffi::c_int != 0
    {
        word = xstrdup((*compl_shown_match.get()).cp_str.data);
        retval = true_0 != 0;
        redrawWinline(curwin.get(), (*curwin.get()).w_cursor.lnum);
    }
    if word.is_null()
        && c != Ctrl_E
        && compl_used_match.get() as ::core::ffi::c_int != 0
        && (*compl_match_array.ptr()).is_null()
        && !(*compl_curr_match.ptr()).is_null()
        && !(*compl_curr_match.get()).cp_str.data.is_null()
    {
        word = xstrdup((*compl_curr_match.get()).cp_str.data);
    }
    if c == Ctrl_E {
        ins_compl_delete(false_0 != 0);
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut plen: size_t = 0 as size_t;
        if !(*compl_leader.ptr()).data.is_null() {
            p = (*compl_leader.ptr()).data;
            plen = (*compl_leader.ptr()).size;
        } else if !(*compl_first_match.ptr()).is_null() {
            p = (*compl_orig_text.ptr()).data;
            plen = (*compl_orig_text.ptr()).size;
        }
        if !p.is_null() {
            let compl_len: ::core::ffi::c_int = get_compl_len();
            if plen as ::core::ffi::c_int > compl_len {
                ins_compl_insert_bytes(
                    p.offset(compl_len as isize),
                    plen as ::core::ffi::c_int - compl_len,
                );
            }
        }
        restore_orig_extmarks();
        retval = true_0 != 0;
    }
    auto_format(false_0 != 0, true_0 != 0);
    ctrl_x_mode.set(prev_mode);
    ins_apply_autocmds(EVENT_COMPLETEDONEPRE);
    ins_compl_free();
    compl_started.set(false_0 != 0);
    compl_matches.set(0 as ::core::ffi::c_int);
    if !shortmess(SHM_COMPLETIONMENU as ::core::ffi::c_int) {
        msg_clr_cmdline();
    }
    ctrl_x_mode.set(CTRL_X_NORMAL as ::core::ffi::c_int);
    compl_enter_selects.set(false_0 != 0);
    if !(*edit_submode.ptr()).is_null() {
        edit_submode.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        redraw_mode.set(true_0 != 0);
    }
    compl_autocomplete.set(false_0 != 0);
    compl_from_nonkeyword.set(false_0 != 0);
    compl_num_bests.set(0 as ::core::ffi::c_int);
    compl_ins_end_col.set(0 as ::core::ffi::c_int as colnr_T);
    if c == Ctrl_C && cmdwin_type.get() != 0 as ::core::ffi::c_int {
        update_screen();
    }
    if want_cindent as ::core::ffi::c_int != 0
        && in_cinkeys(
            KEY_COMPLETE as ::core::ffi::c_int,
            ' ' as ::core::ffi::c_int,
            inindent(0 as ::core::ffi::c_int),
        ) as ::core::ffi::c_int
            != 0
    {
        do_c_expr_indent();
    }
    do_autocmd_completedone(c, prev_mode, word);
    xfree(word as *mut ::core::ffi::c_void);
    return retval;
}
pub unsafe extern "C" fn ins_compl_cancel() -> bool {
    return ins_compl_stop(' ' as ::core::ffi::c_int, ctrl_x_mode.get(), true_0 != 0);
}
pub unsafe extern "C" fn ins_compl_prep(mut c: ::core::ffi::c_int) -> bool {
    let mut retval: bool = false_0 != 0;
    let prev_mode: ::core::ffi::c_int = ctrl_x_mode.get();
    if c != Ctrl_R && vim_is_ctrl_x_key(c) as ::core::ffi::c_int != 0 {
        edit_submode_extra.set(::core::ptr::null_mut::<::core::ffi::c_char>());
    }
    if c == K_SELECT
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MOUSEDOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MOUSEUP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MOUSELEFT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MOUSERIGHT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_COMMAND as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_LUA as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        return retval;
    }
    if ctrl_x_mode.get() == CTRL_X_CMDLINE_CTRL_X as ::core::ffi::c_int && c != Ctrl_X {
        if c == Ctrl_V
            || c == Ctrl_Q
            || c == Ctrl_Z
            || ins_compl_pum_key(c) as ::core::ffi::c_int != 0
            || !vim_is_ctrl_x_key(c)
        {
            ctrl_x_mode.set(CTRL_X_CMDLINE as ::core::ffi::c_int);
            if c == Ctrl_Z {
                retval = true_0 != 0;
            }
        } else {
            ctrl_x_mode.set(CTRL_X_CMDLINE as ::core::ffi::c_int);
            ins_compl_prep(' ' as ::core::ffi::c_int);
            ctrl_x_mode.set(CTRL_X_NOT_DEFINED_YET as ::core::ffi::c_int);
        }
    }
    if ctrl_x_mode_not_defined_yet() as ::core::ffi::c_int != 0
        || ctrl_x_mode_normal() as ::core::ffi::c_int != 0 && !compl_started.get()
    {
        compl_get_longest.set(
            get_cot_flags() & kOptCotFlagLongest as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0 as ::core::ffi::c_uint,
        );
        compl_used_match.set(true_0 != 0);
    }
    if ctrl_x_mode_not_defined_yet() {
        retval = set_ctrl_x_mode(c);
    } else if ctrl_x_mode_not_default() {
        if !vim_is_ctrl_x_key(c) {
            ctrl_x_mode.set(if ctrl_x_mode_scroll() as ::core::ffi::c_int != 0 {
                CTRL_X_NORMAL as ::core::ffi::c_int
            } else {
                CTRL_X_FINISHED as ::core::ffi::c_int
            });
            edit_submode.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        }
        redraw_mode.set(true_0 != 0);
    }
    if compl_started.get() as ::core::ffi::c_int != 0
        || ctrl_x_mode.get() == CTRL_X_FINISHED as ::core::ffi::c_int
    {
        redraw_mode.set(true_0 != 0);
        if ctrl_x_mode_normal() as ::core::ffi::c_int != 0
            && c != Ctrl_N
            && c != Ctrl_P
            && c != Ctrl_R
            && !ins_compl_pum_key(c)
            || ctrl_x_mode.get() == CTRL_X_FINISHED as ::core::ffi::c_int
        {
            retval = ins_compl_stop(c, prev_mode, retval);
        }
    } else if ctrl_x_mode.get() == CTRL_X_LOCAL_MSG as ::core::ffi::c_int {
        do_autocmd_completedone(
            c,
            ctrl_x_mode.get(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        );
    }
    may_trigger_modechanged();
    if !vim_is_ctrl_x_key(c) {
        compl_cont_status.set(0 as ::core::ffi::c_int);
        compl_cont_mode.set(0 as ::core::ffi::c_int);
    }
    return retval;
}
unsafe extern "C" fn ins_compl_fixRedoBufForLeader(mut ptr_arg: *mut ::core::ffi::c_char) {
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut ptr: *mut ::core::ffi::c_char = ptr_arg;
    if ptr.is_null() {
        if !(*compl_leader.ptr()).data.is_null() {
            ptr = (*compl_leader.ptr()).data;
        } else {
            return;
        }
    }
    if !(*compl_orig_text.ptr()).data.is_null() {
        let mut p: *mut ::core::ffi::c_char = (*compl_orig_text.ptr()).data;
        while *p.offset(len as isize) as ::core::ffi::c_int != NUL
            && *p.offset(len as isize) as ::core::ffi::c_int
                == *ptr.offset(len as isize) as ::core::ffi::c_int
        {
            len += 1;
        }
        if len > 0 as ::core::ffi::c_int {
            len -= utf_head_off(p, p.offset(len as isize));
        }
        p = p.offset(len as isize);
        while *p as ::core::ffi::c_int != NUL {
            AppendCharToRedobuff(K_BS);
            p = p.offset(utfc_ptr2len(p) as isize);
        }
    }
    AppendToRedobuffLit(ptr.offset(len as isize), -1 as ::core::ffi::c_int);
}
unsafe extern "C" fn ins_compl_next_buf(
    mut buf: *mut buf_T,
    mut flag: ::core::ffi::c_int,
) -> *mut buf_T {
    static wp: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
    if flag == 'w' as ::core::ffi::c_int {
        if buf == curbuf.get() || !win_valid(wp.get()) {
            wp.set(curwin.get());
        }
        '_c2rust_label: {
            if !(*wp.ptr()).is_null() {
            } else {
                __assert_fail(
                    b"wp\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2872 as ::core::ffi::c_uint,
                    b"buf_T *ins_compl_next_buf(buf_T *, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        loop {
            wp.set(if !(*wp.get()).w_next.is_null() {
                (*wp.get()).w_next
            } else {
                firstwin.get()
            });
            if wp.get() == curwin.get()
                || !(*(*wp.get()).w_buffer).b_scanned
                    && (*wp.get()).w_config.focusable as ::core::ffi::c_int != 0
            {
                break;
            }
        }
        buf = (*wp.get()).w_buffer;
    } else {
        loop {
            buf = if !(*buf).b_next.is_null() {
                (*buf).b_next
            } else {
                firstbuf.get()
            };
            if buf == curbuf.get() {
                break;
            }
            let mut skip_buffer: bool = false;
            if flag == 'U' as ::core::ffi::c_int {
                skip_buffer = (*buf).b_p_bl != 0;
            } else {
                skip_buffer = (*buf).b_p_bl == 0
                    || (*buf).b_ml.ml_mfp.is_null() as ::core::ffi::c_int
                        != (flag == 'u' as ::core::ffi::c_int) as ::core::ffi::c_int;
            }
            if !skip_buffer && !(*buf).b_scanned {
                break;
            }
        }
    }
    return buf;
}
unsafe extern "C" fn get_cpt_sources_count() -> ::core::ffi::c_int {
    let mut dummy: [::core::ffi::c_char; 512] = [0; 512];
    let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char = (*curbuf.get()).b_p_cpt;
    while *p as ::core::ffi::c_int != NUL {
        while *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
        {
            p = p.offset(1);
        }
        if *p as ::core::ffi::c_int != NUL {
            copy_option_part(
                &raw mut p,
                &raw mut dummy as *mut ::core::ffi::c_char,
                LSIZE as ::core::ffi::c_int as size_t,
                b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            count += 1;
        }
    }
    return count;
}
static cfu_cb: GlobalCell<Callback> = GlobalCell::new(Callback {
    data: C2Rust_Unnamed_5 {
        funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    type_0: kCallbackNone,
});
static ofu_cb: GlobalCell<Callback> = GlobalCell::new(Callback {
    data: C2Rust_Unnamed_5 {
        funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    type_0: kCallbackNone,
});
static tsrfu_cb: GlobalCell<Callback> = GlobalCell::new(Callback {
    data: C2Rust_Unnamed_5 {
        funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    type_0: kCallbackNone,
});
static cpt_cb: GlobalCell<*mut Callback> = GlobalCell::new(::core::ptr::null_mut::<Callback>());
static cpt_cb_count: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
unsafe extern "C" fn copy_global_to_buflocal_cb(
    mut globcb: *mut Callback,
    mut bufcb: *mut Callback,
) {
    callback_free(bufcb);
    if (*globcb).type_0 as ::core::ffi::c_uint
        != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        callback_copy(bufcb, globcb);
    }
}
pub unsafe extern "C" fn did_set_completefunc(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut retval: ::core::ffi::c_int = 0;
    if (*args).os_flags & OPT_LOCAL as ::core::ffi::c_int != 0 {
        retval = option_set_callback_func((*args).os_newval.string.data, &raw mut (*buf).b_cfu_cb);
    } else {
        retval = option_set_callback_func((*args).os_newval.string.data, cfu_cb.ptr());
        if retval == OK && (*args).os_flags & OPT_GLOBAL as ::core::ffi::c_int == 0 {
            set_buflocal_cfu_callback(buf);
        }
    }
    return if retval == FAIL {
        &raw const e_invarg as *const ::core::ffi::c_char
    } else {
        ::core::ptr::null::<::core::ffi::c_char>()
    };
}
pub unsafe extern "C" fn set_buflocal_cfu_callback(mut buf: *mut buf_T) {
    copy_global_to_buflocal_cb(cfu_cb.ptr(), &raw mut (*buf).b_cfu_cb);
}
pub unsafe extern "C" fn did_set_omnifunc(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut retval: ::core::ffi::c_int = 0;
    if (*args).os_flags & OPT_LOCAL as ::core::ffi::c_int != 0 {
        retval = option_set_callback_func((*args).os_newval.string.data, &raw mut (*buf).b_ofu_cb);
    } else {
        retval = option_set_callback_func((*args).os_newval.string.data, ofu_cb.ptr());
        if retval == OK && (*args).os_flags & OPT_GLOBAL as ::core::ffi::c_int == 0 {
            set_buflocal_ofu_callback(buf);
        }
    }
    return if retval == FAIL {
        &raw const e_invarg as *const ::core::ffi::c_char
    } else {
        ::core::ptr::null::<::core::ffi::c_char>()
    };
}
pub unsafe extern "C" fn set_buflocal_ofu_callback(mut buf: *mut buf_T) {
    copy_global_to_buflocal_cb(ofu_cb.ptr(), &raw mut (*buf).b_ofu_cb);
}
pub unsafe extern "C" fn clear_cpt_callbacks(
    mut callbacks: *mut *mut Callback,
    mut count: ::core::ffi::c_int,
) {
    if callbacks.is_null() || (*callbacks).is_null() {
        return;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < count {
        callback_free((*callbacks).offset(i as isize));
        i += 1;
    }
    let mut ptr_: *mut *mut ::core::ffi::c_void = callbacks as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
}
unsafe extern "C" fn copy_cpt_callbacks(
    mut dest: *mut *mut Callback,
    mut dest_cnt: *mut ::core::ffi::c_int,
    mut src: *mut Callback,
    mut cnt: ::core::ffi::c_int,
) {
    if cnt == 0 as ::core::ffi::c_int {
        return;
    }
    clear_cpt_callbacks(dest, *dest_cnt);
    *dest = xcalloc(cnt as size_t, ::core::mem::size_of::<Callback>()) as *mut Callback;
    *dest_cnt = cnt;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < cnt {
        if (*src.offset(i as isize)).type_0 as ::core::ffi::c_uint
            != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            callback_copy((*dest).offset(i as isize), src.offset(i as isize));
        }
        i += 1;
    }
}
pub unsafe extern "C" fn set_buflocal_cpt_callbacks(mut buf: *mut buf_T) {
    if buf.is_null() || cpt_cb_count.get() == 0 as ::core::ffi::c_int {
        return;
    }
    copy_cpt_callbacks(
        &raw mut (*buf).b_p_cpt_cb,
        &raw mut (*buf).b_p_cpt_count,
        cpt_cb.get(),
        cpt_cb_count.get(),
    );
}
pub unsafe extern "C" fn set_cpt_callbacks(mut args: *mut optset_T) -> ::core::ffi::c_int {
    let mut local: bool =
        (*args).os_flags & OPT_LOCAL as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    if (*curbuf.ptr()).is_null() {
        return FAIL;
    }
    clear_cpt_callbacks(
        &raw mut (*curbuf.get()).b_p_cpt_cb,
        (*curbuf.get()).b_p_cpt_count,
    );
    (*curbuf.get()).b_p_cpt_count = 0 as ::core::ffi::c_int;
    let mut count: ::core::ffi::c_int = get_cpt_sources_count();
    if count == 0 as ::core::ffi::c_int {
        return OK;
    }
    (*curbuf.get()).b_p_cpt_cb =
        xcalloc(count as size_t, ::core::mem::size_of::<Callback>()) as *mut Callback;
    (*curbuf.get()).b_p_cpt_count = count;
    let mut buf: [::core::ffi::c_char; 512] = [0; 512];
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char = (*curbuf.get()).b_p_cpt;
    while *p as ::core::ffi::c_int != NUL {
        while *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
        {
            p = p.offset(1);
        }
        if *p as ::core::ffi::c_int != NUL {
            let mut slen: size_t = copy_option_part(
                &raw mut p,
                &raw mut buf as *mut ::core::ffi::c_char,
                LSIZE as ::core::ffi::c_int as size_t,
                b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            if slen > 0 as size_t
                && buf[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                    == 'F' as ::core::ffi::c_int
                && buf[1 as ::core::ffi::c_int as usize] as ::core::ffi::c_int != NUL
            {
                let mut caret: *mut ::core::ffi::c_char = vim_strchr(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    '^' as ::core::ffi::c_int,
                );
                if !caret.is_null() {
                    *caret = NUL as ::core::ffi::c_char;
                }
                if option_set_callback_func(
                    (&raw mut buf as *mut ::core::ffi::c_char)
                        .offset(1 as ::core::ffi::c_int as isize),
                    (*curbuf.get()).b_p_cpt_cb.offset(idx as isize),
                ) != OK
                {
                    (*(*curbuf.get()).b_p_cpt_cb.offset(idx as isize)).type_0 = kCallbackNone;
                }
            }
            idx += 1;
        }
    }
    if !local {
        copy_cpt_callbacks(
            cpt_cb.ptr(),
            cpt_cb_count.ptr(),
            (*curbuf.get()).b_p_cpt_cb,
            (*curbuf.get()).b_p_cpt_count,
        );
    }
    return OK;
}
pub unsafe extern "C" fn did_set_thesaurusfunc(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut retval: ::core::ffi::c_int = 0;
    if (*args).os_flags & OPT_LOCAL as ::core::ffi::c_int != 0 {
        retval = option_set_callback_func((*buf).b_p_tsrfu, &raw mut (*buf).b_tsrfu_cb);
    } else {
        retval = option_set_callback_func(p_tsrfu.get(), tsrfu_cb.ptr());
        if (*args).os_flags & OPT_GLOBAL as ::core::ffi::c_int == 0 {
            callback_free(&raw mut (*buf).b_tsrfu_cb);
        }
    }
    return if retval == FAIL {
        &raw const e_invarg as *const ::core::ffi::c_char
    } else {
        ::core::ptr::null::<::core::ffi::c_char>()
    };
}
pub unsafe extern "C" fn set_ref_in_cpt_callbacks(
    mut callbacks: *mut Callback,
    mut count: ::core::ffi::c_int,
    mut copyID: ::core::ffi::c_int,
) -> bool {
    let mut abort: bool = false_0 != 0;
    if callbacks.is_null() {
        return false_0 != 0;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < count {
        abort = abort as ::core::ffi::c_int != 0
            || set_ref_in_callback(
                callbacks.offset(i as isize),
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as ::core::ffi::c_int
                != 0;
        i += 1;
    }
    return abort;
}
pub unsafe extern "C" fn set_ref_in_insexpand_funcs(mut copyID: ::core::ffi::c_int) -> bool {
    let mut abort: bool = set_ref_in_callback(
        cfu_cb.ptr(),
        copyID,
        ::core::ptr::null_mut::<*mut ht_stack_T>(),
        ::core::ptr::null_mut::<*mut list_stack_T>(),
    );
    abort = abort as ::core::ffi::c_int != 0
        || set_ref_in_callback(
            ofu_cb.ptr(),
            copyID,
            ::core::ptr::null_mut::<*mut ht_stack_T>(),
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        ) as ::core::ffi::c_int
            != 0;
    abort = abort as ::core::ffi::c_int != 0
        || set_ref_in_callback(
            tsrfu_cb.ptr(),
            copyID,
            ::core::ptr::null_mut::<*mut ht_stack_T>(),
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        ) as ::core::ffi::c_int
            != 0;
    abort = abort as ::core::ffi::c_int != 0
        || set_ref_in_cpt_callbacks(cpt_cb.get(), cpt_cb_count.get(), copyID) as ::core::ffi::c_int
            != 0;
    return abort;
}
unsafe extern "C" fn get_complete_funcname(
    mut type_0: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    match type_0 {
        12 => return (*curbuf.get()).b_p_cfu,
        13 => return (*curbuf.get()).b_p_ofu,
        266 => {
            return if *(*curbuf.get()).b_p_tsrfu as ::core::ffi::c_int == NUL {
                p_tsrfu.get()
            } else {
                (*curbuf.get()).b_p_tsrfu
            };
        }
        _ => {
            return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
    };
}
unsafe extern "C" fn get_insert_callback(mut type_0: ::core::ffi::c_int) -> *mut Callback {
    if type_0 == CTRL_X_FUNCTION as ::core::ffi::c_int {
        return &raw mut (*curbuf.get()).b_cfu_cb;
    }
    if type_0 == CTRL_X_OMNI as ::core::ffi::c_int {
        return &raw mut (*curbuf.get()).b_ofu_cb;
    }
    return if *(*curbuf.get()).b_p_tsrfu as ::core::ffi::c_int != NUL {
        &raw mut (*curbuf.get()).b_tsrfu_cb
    } else {
        tsrfu_cb.ptr()
    };
}
unsafe extern "C" fn expand_by_function(
    mut type_0: ::core::ffi::c_int,
    mut base: *mut ::core::ffi::c_char,
    mut cb: *mut Callback,
) {
    let mut matchlist: *mut list_T = ::core::ptr::null_mut::<list_T>();
    let mut matchdict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let save_State: ::core::ffi::c_int = State.get();
    '_c2rust_label: {
        if !(*curbuf.ptr()).is_null() {
        } else {
            __assert_fail(
                b"curbuf != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                3189 as ::core::ffi::c_uint,
                b"void expand_by_function(int, char *, Callback *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let is_cpt_function: bool = !cb.is_null();
    if !is_cpt_function {
        let mut funcname: *mut ::core::ffi::c_char = get_complete_funcname(type_0);
        if *funcname as ::core::ffi::c_int == NUL {
            return;
        }
        cb = get_insert_callback(type_0);
    }
    let mut args: [typval_T; 3] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 3];
    args[0 as ::core::ffi::c_int as usize].v_type = VAR_NUMBER;
    args[1 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
    args[2 as ::core::ffi::c_int as usize].v_type = VAR_UNKNOWN;
    args[0 as ::core::ffi::c_int as usize].vval.v_number = 0 as varnumber_T;
    args[1 as ::core::ffi::c_int as usize].vval.v_string = (if !base.is_null() {
        base as *const ::core::ffi::c_char
    } else {
        b"\0".as_ptr() as *const ::core::ffi::c_char
    }) as *mut ::core::ffi::c_char;
    let mut pos: pos_T = (*curwin.get()).w_cursor;
    (*textlock.ptr()) += 1;
    if callback_call(
        cb,
        2 as ::core::ffi::c_int,
        &raw mut args as *mut typval_T,
        &raw mut rettv,
    ) {
        match rettv.v_type as ::core::ffi::c_uint {
            4 => {
                matchlist = rettv.vval.v_list;
            }
            5 => {
                matchdict = rettv.vval.v_dict;
            }
            8 | _ => {
                tv_clear(&raw mut rettv);
            }
        }
    }
    (*textlock.ptr()) -= 1;
    (*curwin.get()).w_cursor = pos;
    check_cursor(curwin.get());
    validate_cursor(curwin.get());
    if !equalpos((*curwin.get()).w_cursor, pos) {
        emsg(gettext(
            (e_compldel.ptr() as *const _) as *const ::core::ffi::c_char,
        ));
    } else if !matchlist.is_null() {
        ins_compl_add_list(matchlist);
    } else if !matchdict.is_null() {
        ins_compl_add_dict(matchdict);
    }
    State.set(save_State);
    if !matchdict.is_null() {
        tv_dict_unref(matchdict);
    }
    if !matchlist.is_null() {
        tv_list_unref(matchlist);
    }
}
#[inline]
unsafe extern "C" fn get_user_highlight_attr(
    mut hlname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if !hlname.is_null() && *hlname as ::core::ffi::c_int != NUL {
        return syn_name2attr(hlname);
    }
    return -1 as ::core::ffi::c_int;
}
unsafe extern "C" fn ins_compl_add_tv(
    tv: *mut typval_T,
    dir: Direction,
    mut fast: bool,
) -> ::core::ffi::c_int {
    let mut word: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut dup: bool = false_0 != 0;
    let mut empty: bool = false_0 != 0;
    let mut flags: ::core::ffi::c_int = if fast as ::core::ffi::c_int != 0 {
        CP_FAST as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    let mut cptext: [*mut ::core::ffi::c_char; 4] =
        [::core::ptr::null_mut::<::core::ffi::c_char>(); 4];
    let mut user_abbr_hlname: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut user_kind_hlname: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut user_hl: [::core::ffi::c_int; 2] = [-1 as ::core::ffi::c_int, -1 as ::core::ffi::c_int];
    let mut user_data: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    user_data.v_type = VAR_UNKNOWN;
    if (*tv).v_type as ::core::ffi::c_uint == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        && !(*tv).vval.v_dict.is_null()
    {
        word = tv_dict_get_string(
            (*tv).vval.v_dict,
            b"word\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        cptext[CPT_ABBR as ::core::ffi::c_int as usize] = tv_dict_get_string(
            (*tv).vval.v_dict,
            b"abbr\0".as_ptr() as *const ::core::ffi::c_char,
            true_0 != 0,
        ) as *mut ::core::ffi::c_char;
        cptext[CPT_MENU as ::core::ffi::c_int as usize] = tv_dict_get_string(
            (*tv).vval.v_dict,
            b"menu\0".as_ptr() as *const ::core::ffi::c_char,
            true_0 != 0,
        ) as *mut ::core::ffi::c_char;
        cptext[CPT_KIND as ::core::ffi::c_int as usize] = tv_dict_get_string(
            (*tv).vval.v_dict,
            b"kind\0".as_ptr() as *const ::core::ffi::c_char,
            true_0 != 0,
        ) as *mut ::core::ffi::c_char;
        cptext[CPT_INFO as ::core::ffi::c_int as usize] = tv_dict_get_string(
            (*tv).vval.v_dict,
            b"info\0".as_ptr() as *const ::core::ffi::c_char,
            true_0 != 0,
        ) as *mut ::core::ffi::c_char;
        user_abbr_hlname = tv_dict_get_string(
            (*tv).vval.v_dict,
            b"abbr_hlgroup\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        user_hl[0 as ::core::ffi::c_int as usize] = get_user_highlight_attr(user_abbr_hlname);
        user_kind_hlname = tv_dict_get_string(
            (*tv).vval.v_dict,
            b"kind_hlgroup\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        user_hl[1 as ::core::ffi::c_int as usize] = get_user_highlight_attr(user_kind_hlname);
        tv_dict_get_tv(
            (*tv).vval.v_dict,
            b"user_data\0".as_ptr() as *const ::core::ffi::c_char,
            &raw mut user_data,
        );
        if tv_dict_get_number(
            (*tv).vval.v_dict,
            b"icase\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0
        {
            flags |= CP_ICASE as ::core::ffi::c_int;
        }
        dup = tv_dict_get_number(
            (*tv).vval.v_dict,
            b"dup\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0;
        empty = tv_dict_get_number(
            (*tv).vval.v_dict,
            b"empty\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0;
        if !tv_dict_get_string(
            (*tv).vval.v_dict,
            b"equal\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        )
        .is_null()
            && tv_dict_get_number(
                (*tv).vval.v_dict,
                b"equal\0".as_ptr() as *const ::core::ffi::c_char,
            ) != 0
        {
            flags |= CP_EQUAL as ::core::ffi::c_int;
        }
    } else {
        word = tv_get_string_chk(tv);
        memset(
            &raw mut cptext as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<[*mut ::core::ffi::c_char; 4]>(),
        );
    }
    if word.is_null() || !empty && *word as ::core::ffi::c_int == NUL {
        free_cptext(&raw mut cptext as *mut *mut ::core::ffi::c_char);
        tv_clear(&raw mut user_data);
        return FAIL;
    }
    let mut status: ::core::ffi::c_int = ins_compl_add(
        word as *mut ::core::ffi::c_char,
        -1 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        &raw mut cptext as *mut *mut ::core::ffi::c_char,
        true_0 != 0,
        &raw mut user_data,
        dir,
        flags,
        dup,
        &raw mut user_hl as *mut ::core::ffi::c_int,
        FUZZY_SCORE_NONE as ::core::ffi::c_int,
    );
    if status != OK {
        tv_clear(&raw mut user_data);
    }
    return status;
}
unsafe extern "C" fn ins_compl_add_list(list: *mut list_T) {
    let mut dir: Direction = compl_direction.get();
    let l_: *mut list_T = list;
    if !l_.is_null() {
        let mut li: *mut listitem_T = (*l_).lv_first;
        while !li.is_null() {
            if ins_compl_add_tv(&raw mut (*li).li_tv, dir, true) == 1 as ::core::ffi::c_int {
                dir = FORWARD;
            } else if did_emsg.get() != 0 {
                break;
            }
            li = (*li).li_next;
        }
    }
}
unsafe extern "C" fn ins_compl_add_dict(mut dict: *mut dict_T) {
    compl_opt_refresh_always.set(false_0 != 0);
    let mut di_refresh: *mut dictitem_T = tv_dict_find(
        dict,
        b"refresh\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di_refresh.is_null()
        && (*di_refresh).di_tv.v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut v: *const ::core::ffi::c_char = (*di_refresh).di_tv.vval.v_string;
        if !v.is_null()
            && strcmp(v, b"always\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
        {
            compl_opt_refresh_always.set(true_0 != 0);
        }
    }
    let mut di_words: *mut dictitem_T = tv_dict_find(
        dict,
        b"words\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di_words.is_null()
        && (*di_words).di_tv.v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        ins_compl_add_list((*di_words).di_tv.vval.v_list);
    }
}
unsafe extern "C" fn save_orig_extmarks() {
    extmark_splice_delete(
        curbuf.get(),
        (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        compl_col.get(),
        (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        compl_col.get() + compl_length.get() as colnr_T,
        compl_orig_extmarks.ptr(),
        true_0 != 0,
        kExtmarkUndo,
    );
}
unsafe extern "C" fn restore_orig_extmarks() {
    let mut i: ::core::ffi::c_long = ((*compl_orig_extmarks.ptr()).size as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int) as ::core::ffi::c_long;
    while i > -1 as ::core::ffi::c_long {
        let mut undo_info: ExtmarkUndoObject =
            *(*compl_orig_extmarks.ptr()).items.offset(i as isize);
        extmark_apply_undo(undo_info, true_0 != 0);
        i -= 1;
    }
}
unsafe extern "C" fn set_completion(mut startcol: colnr_T, mut list: *mut list_T) {
    let mut flags: ::core::ffi::c_int = CP_ORIGINAL_TEXT as ::core::ffi::c_int;
    let mut cur_cot_flags: ::core::ffi::c_uint = get_cot_flags();
    let mut compl_longest: bool = cur_cot_flags
        & kOptCotFlagLongest as ::core::ffi::c_int as ::core::ffi::c_uint
        != 0 as ::core::ffi::c_uint;
    let mut compl_no_insert: bool = cur_cot_flags
        & kOptCotFlagNoinsert as ::core::ffi::c_int as ::core::ffi::c_uint
        != 0 as ::core::ffi::c_uint;
    let mut compl_no_select: bool = cur_cot_flags
        & kOptCotFlagNoselect as ::core::ffi::c_int as ::core::ffi::c_uint
        != 0 as ::core::ffi::c_uint;
    if ctrl_x_mode_not_default() {
        ins_compl_prep(' ' as ::core::ffi::c_int);
    }
    ins_compl_clear();
    ins_compl_free();
    compl_get_longest.set(compl_longest);
    compl_direction.set(FORWARD);
    if startcol > (*curwin.get()).w_cursor.col {
        startcol = (*curwin.get()).w_cursor.col;
    }
    compl_col.set(startcol);
    compl_lnum.set((*curwin.get()).w_cursor.lnum);
    compl_length.set(((*curwin.get()).w_cursor.col - startcol) as ::core::ffi::c_int);
    compl_orig_text.set(cbuf_to_string(
        get_cursor_line_ptr().offset(compl_col.get() as isize),
        compl_length.get() as size_t,
    ));
    save_orig_extmarks();
    if p_ic.get() != 0 {
        flags |= CP_ICASE as ::core::ffi::c_int;
    }
    if ins_compl_add(
        (*compl_orig_text.ptr()).data,
        (*compl_orig_text.ptr()).size as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null::<*mut ::core::ffi::c_char>(),
        false_0 != 0,
        ::core::ptr::null_mut::<typval_T>(),
        kDirectionNotSet,
        flags | CP_FAST as ::core::ffi::c_int,
        false_0 != 0,
        ::core::ptr::null::<::core::ffi::c_int>(),
        FUZZY_SCORE_NONE as ::core::ffi::c_int,
    ) != OK
    {
        return;
    }
    ctrl_x_mode.set(CTRL_X_EVAL as ::core::ffi::c_int);
    ins_compl_add_list(list);
    compl_matches.set(ins_compl_make_cyclic());
    compl_started.set(true_0 != 0);
    compl_used_match.set(true_0 != 0);
    compl_cont_status.set(0 as ::core::ffi::c_int);
    let mut save_w_wrow: ::core::ffi::c_int = (*curwin.get()).w_wrow;
    let mut save_w_leftcol: ::core::ffi::c_int = (*curwin.get()).w_leftcol as ::core::ffi::c_int;
    compl_curr_match.set(compl_first_match.get());
    let mut no_select: bool =
        compl_no_select as ::core::ffi::c_int != 0 || compl_longest as ::core::ffi::c_int != 0;
    if compl_no_insert as ::core::ffi::c_int != 0 || no_select as ::core::ffi::c_int != 0 {
        ins_complete(K_DOWN, false_0 != 0);
        if no_select {
            ins_complete(K_UP, false_0 != 0);
        }
    } else {
        ins_complete(Ctrl_N, false_0 != 0);
    }
    compl_enter_selects.set(compl_no_insert);
    if !compl_interrupted.get() {
        show_pum(save_w_wrow, save_w_leftcol);
    }
    may_trigger_modechanged();
    ui_flush();
}
pub unsafe extern "C" fn f_complete(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if State.get() & MODE_INSERT as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        emsg(gettext(
            b"E785: complete() can only be used in Insert mode\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
        return;
    }
    if !undo_allowed(curbuf.get()) {
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
    } else {
        let startcol: colnr_T = tv_get_number_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            ::core::ptr::null_mut::<bool>(),
        ) as colnr_T;
        if startcol > 0 as ::core::ffi::c_int {
            set_completion(
                startcol - 1 as colnr_T,
                (*argvars.offset(1 as ::core::ffi::c_int as isize))
                    .vval
                    .v_list,
            );
        }
    };
}
pub unsafe extern "C" fn f_complete_add(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = ins_compl_add_tv(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        kDirectionNotSet,
        false_0 != 0,
    ) as varnumber_T;
}
pub unsafe extern "C" fn f_complete_check(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut saved: ::core::ffi::c_int = RedrawingDisabled.get();
    RedrawingDisabled.set(0 as ::core::ffi::c_int);
    ins_compl_check_keys(0 as ::core::ffi::c_int, true_0 != 0);
    (*rettv).vval.v_number = ins_compl_interrupted() as varnumber_T;
    RedrawingDisabled.set(saved);
}
unsafe extern "C" fn ins_compl_mode() -> *mut ::core::ffi::c_char {
    if ctrl_x_mode_not_defined_yet() as ::core::ffi::c_int != 0
        || ctrl_x_mode_scroll() as ::core::ffi::c_int != 0
        || compl_started.get() as ::core::ffi::c_int != 0
    {
        return (*ctrl_x_mode_names.ptr())[(ctrl_x_mode.get() & !CTRL_X_WANT_IDENT) as usize];
    }
    return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
}
unsafe extern "C" fn ins_compl_update_sequence_numbers() {
    let mut number: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut match_0: *mut compl_T = ::core::ptr::null_mut::<compl_T>();
    if compl_dir_forward() {
        match_0 = (*compl_curr_match.get()).cp_prev;
        while !match_0.is_null() && !is_first_match(match_0) {
            if (*match_0).cp_number != -1 as ::core::ffi::c_int {
                number = (*match_0).cp_number;
                break;
            } else {
                match_0 = (*match_0).cp_prev;
            }
        }
        if !match_0.is_null() {
            match_0 = (*match_0).cp_next;
            while !match_0.is_null() && (*match_0).cp_number == -1 as ::core::ffi::c_int {
                number += 1;
                (*match_0).cp_number = number;
                match_0 = (*match_0).cp_next;
            }
        }
    } else {
        '_c2rust_label: {
            if compl_direction.get() as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"compl_direction == BACKWARD\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3532 as ::core::ffi::c_uint,
                    b"void ins_compl_update_sequence_numbers(void)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        match_0 = (*compl_curr_match.get()).cp_next;
        while !match_0.is_null() && !is_first_match(match_0) {
            if (*match_0).cp_number != -1 as ::core::ffi::c_int {
                number = (*match_0).cp_number;
                break;
            } else {
                match_0 = (*match_0).cp_next;
            }
        }
        if !match_0.is_null() {
            match_0 = (*match_0).cp_prev;
            while !match_0.is_null() && (*match_0).cp_number == -1 as ::core::ffi::c_int {
                number += 1;
                (*match_0).cp_number = number;
                match_0 = (*match_0).cp_prev;
            }
        }
    };
}
unsafe extern "C" fn fill_complete_info_dict(
    mut di: *mut dict_T,
    mut match_0: *mut compl_T,
    mut add_match: bool,
) {
    tv_dict_add_str(
        di,
        b"word\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        (*match_0).cp_str.data,
    );
    tv_dict_add_str(
        di,
        b"abbr\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        (*match_0).cp_text[CPT_ABBR as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
    );
    tv_dict_add_str(
        di,
        b"menu\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        (*match_0).cp_text[CPT_MENU as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
    );
    tv_dict_add_str(
        di,
        b"kind\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        (*match_0).cp_text[CPT_KIND as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
    );
    tv_dict_add_str(
        di,
        b"info\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        (*match_0).cp_text[CPT_INFO as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
    );
    if add_match {
        tv_dict_add_bool(
            di,
            b"match\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            (*match_0).cp_in_match_array as BoolVarValue,
        );
    }
    if (*match_0).cp_user_data.v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_dict_add_str(
            di,
            b"user_data\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            b"\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        tv_dict_add_tv(
            di,
            b"user_data\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            &raw mut (*match_0).cp_user_data,
        );
    };
}
unsafe extern "C" fn get_complete_info(mut what_list: *mut list_T, mut retdict: *mut dict_T) {
    let mut what_flag: ::core::ffi::c_int = 0;
    if what_list.is_null() {
        what_flag = CI_WHAT_ALL & !(CI_WHAT_MATCHES | CI_WHAT_COMPLETED);
    } else {
        what_flag = 0 as ::core::ffi::c_int;
        let mut item: *mut listitem_T = tv_list_first(what_list);
        while !item.is_null() {
            let mut what: *const ::core::ffi::c_char = tv_get_string(&raw mut (*item).li_tv);
            if strcmp(what, b"mode\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                what_flag |= CI_WHAT_MODE;
            } else if strcmp(
                what,
                b"pum_visible\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                what_flag |= CI_WHAT_PUM_VISIBLE;
            } else if strcmp(what, b"items\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                what_flag |= CI_WHAT_ITEMS;
            } else if strcmp(what, b"selected\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                what_flag |= CI_WHAT_SELECTED;
            } else if strcmp(what, b"completed\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                what_flag |= CI_WHAT_COMPLETED;
            } else if strcmp(
                what,
                b"preinserted_text\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                what_flag |= CI_WHAT_PREINSERTED_TEXT;
            } else if strcmp(what, b"matches\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                what_flag |= CI_WHAT_MATCHES;
            }
            item = (*item).li_next;
        }
    }
    let mut ret: ::core::ffi::c_int = OK;
    if what_flag & CI_WHAT_MODE != 0 {
        ret = tv_dict_add_str(
            retdict,
            b"mode\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            ins_compl_mode(),
        );
    }
    if ret == OK && what_flag & CI_WHAT_PUM_VISIBLE != 0 {
        ret = tv_dict_add_nr(
            retdict,
            b"pum_visible\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
            pum_visible() as varnumber_T,
        );
    }
    if ret == OK && what_flag & CI_WHAT_PREINSERTED_TEXT != 0 {
        let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
        let mut len: ::core::ffi::c_int = compl_ins_end_col.get() as ::core::ffi::c_int
            - (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
        ret = tv_dict_add_str_len(
            retdict,
            b"preinserted_text\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 17]>().wrapping_sub(1 as size_t),
            if len > 0 as ::core::ffi::c_int {
                line.offset((*curwin.get()).w_cursor.col as isize) as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
            if len > 0 as ::core::ffi::c_int {
                len
            } else {
                0 as ::core::ffi::c_int
            },
        );
    }
    if ret == OK
        && what_flag & (CI_WHAT_ITEMS | CI_WHAT_SELECTED | CI_WHAT_MATCHES | CI_WHAT_COMPLETED) != 0
    {
        let mut li: *mut list_T = ::core::ptr::null_mut::<list_T>();
        let mut selected_idx: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut has_items: bool = what_flag & CI_WHAT_ITEMS != 0;
        let mut has_matches: bool = what_flag & CI_WHAT_MATCHES != 0;
        let mut has_completed: bool = what_flag & CI_WHAT_COMPLETED != 0;
        if has_items as ::core::ffi::c_int != 0 || has_matches as ::core::ffi::c_int != 0 {
            li = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
            let mut key: *const ::core::ffi::c_char =
                if has_matches as ::core::ffi::c_int != 0 && !has_items {
                    b"matches\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"items\0".as_ptr() as *const ::core::ffi::c_char
                };
            ret = tv_dict_add_list(retdict, key, strlen(key), li);
        }
        if ret == OK && what_flag & CI_WHAT_SELECTED != 0 {
            if !(*compl_curr_match.ptr()).is_null()
                && (*compl_curr_match.get()).cp_number == -1 as ::core::ffi::c_int
            {
                ins_compl_update_sequence_numbers();
            }
        }
        if ret == OK && !(*compl_first_match.ptr()).is_null() {
            let mut list_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut match_0: *mut compl_T = compl_first_match.get();
            loop {
                if !match_at_original_text(match_0) {
                    if has_items as ::core::ffi::c_int != 0
                        || has_matches as ::core::ffi::c_int != 0
                            && (*match_0).cp_in_match_array as ::core::ffi::c_int != 0
                    {
                        let mut di: *mut dict_T = tv_dict_alloc();
                        tv_list_append_dict(li, di);
                        fill_complete_info_dict(
                            di,
                            match_0,
                            has_matches as ::core::ffi::c_int != 0
                                && has_items as ::core::ffi::c_int != 0,
                        );
                    }
                    if !(*compl_curr_match.ptr()).is_null()
                        && (*compl_curr_match.get()).cp_number == (*match_0).cp_number
                    {
                        selected_idx = list_idx;
                    }
                    if !has_matches || (*match_0).cp_in_match_array as ::core::ffi::c_int != 0 {
                        list_idx += 1;
                    }
                }
                match_0 = (*match_0).cp_next;
                if !(!match_0.is_null() && !is_first_match(match_0)) {
                    break;
                }
            }
        }
        if ret == OK && what_flag & CI_WHAT_SELECTED != 0 {
            ret = tv_dict_add_nr(
                retdict,
                b"selected\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                selected_idx as varnumber_T,
            );
            let mut wp: *mut win_T = win_float_find_preview();
            if !wp.is_null() {
                tv_dict_add_nr(
                    retdict,
                    b"preview_winid\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 14]>().wrapping_sub(1 as size_t),
                    (*wp).handle as varnumber_T,
                );
                tv_dict_add_nr(
                    retdict,
                    b"preview_bufnr\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 14]>().wrapping_sub(1 as size_t),
                    (*(*wp).w_buffer).handle as varnumber_T,
                );
            }
        }
        if ret == OK
            && selected_idx != -1 as ::core::ffi::c_int
            && has_completed as ::core::ffi::c_int != 0
        {
            let mut di_0: *mut dict_T = tv_dict_alloc();
            fill_complete_info_dict(di_0, compl_curr_match.get(), false_0 != 0);
            ret = tv_dict_add_dict(
                retdict,
                b"completed\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
                di_0,
            );
        }
    }
}
pub const CI_WHAT_MODE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CI_WHAT_PUM_VISIBLE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CI_WHAT_ITEMS: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CI_WHAT_SELECTED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CI_WHAT_COMPLETED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CI_WHAT_MATCHES: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const CI_WHAT_PREINSERTED_TEXT: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CI_WHAT_ALL: ::core::ffi::c_int = 0xff as ::core::ffi::c_int;
pub unsafe extern "C" fn f_complete_info(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_dict_alloc_ret(rettv);
    let mut what_list: *mut list_T = ::core::ptr::null_mut::<list_T>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
            return;
        }
        what_list = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_list;
    }
    get_complete_info(what_list, (*rettv).vval.v_dict);
}
unsafe extern "C" fn thesaurus_func_complete(mut type_0: ::core::ffi::c_int) -> bool {
    return type_0 == CTRL_X_THESAURUS as ::core::ffi::c_int
        && (*(*curbuf.get()).b_p_tsrfu as ::core::ffi::c_int != NUL
            || *p_tsrfu.get() as ::core::ffi::c_int != NUL);
}
unsafe extern "C" fn may_advance_cpt_index(mut cpt: *const ::core::ffi::c_char) -> bool {
    let mut p: *const ::core::ffi::c_char = cpt;
    if cpt_sources_index.get() == -1 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    while *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int
        || *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
    {
        p = p.offset(1);
    }
    return *p as ::core::ffi::c_int != NUL;
}
unsafe extern "C" fn process_next_cpt_value(
    mut st: *mut ins_compl_next_state_T,
    mut compl_type_arg: *mut ::core::ffi::c_int,
    mut start_match_pos: *mut pos_T,
    mut fuzzy_collect: bool,
    mut advance_cpt_idx: *mut bool,
) -> ::core::ffi::c_int {
    let mut compl_type: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut status: ::core::ffi::c_int = INS_COMPL_CPT_OK as ::core::ffi::c_int;
    let mut skip_source: bool = compl_autocomplete.get() as ::core::ffi::c_int != 0
        && compl_from_nonkeyword.get() as ::core::ffi::c_int != 0;
    (*st).found_all = false_0 != 0;
    *advance_cpt_idx = false_0 != 0;
    while *(*st).e_cpt as ::core::ffi::c_int == ',' as ::core::ffi::c_int
        || *(*st).e_cpt as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
    {
        (*st).e_cpt = (*st).e_cpt.offset(1);
    }
    '_done: {
        if *(*st).e_cpt as ::core::ffi::c_int == '.' as ::core::ffi::c_int
            && !(*curbuf.get()).b_scanned
            && !skip_source
            && !compl_time_slice_expired.get()
        {
            (*st).ins_buf = curbuf.get();
            (*st).first_match_pos = *start_match_pos;
            if ctrl_x_mode_normal() as ::core::ffi::c_int != 0
                && (!fuzzy_collect && dec(&raw mut (*st).first_match_pos) < 0 as ::core::ffi::c_int)
            {
                (*st).first_match_pos.lnum = (*(*st).ins_buf).b_ml.ml_line_count;
                (*st).first_match_pos.col = ml_get_len((*st).first_match_pos.lnum);
            }
            (*st).last_match_pos = (*st).first_match_pos;
            compl_type = 0 as ::core::ffi::c_int;
            (*st).set_match_pos = true_0 != 0;
        } else if !skip_source
            && !compl_time_slice_expired.get()
            && !vim_strchr(
                b"buwU\0".as_ptr() as *const ::core::ffi::c_char,
                *(*st).e_cpt as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
            && {
                (*st).ins_buf =
                    ins_compl_next_buf((*st).ins_buf, *(*st).e_cpt as ::core::ffi::c_int);
                (*st).ins_buf != curbuf.get()
            }
        {
            if !(*(*st).ins_buf).b_ml.ml_mfp.is_null() {
                compl_started.set(true_0 != 0);
                (*st).last_match_pos.col = 0 as ::core::ffi::c_int as colnr_T;
                (*st).first_match_pos.col = (*st).last_match_pos.col;
                (*st).first_match_pos.lnum = (*(*st).ins_buf).b_ml.ml_line_count + 1 as linenr_T;
                (*st).last_match_pos.lnum = 0 as ::core::ffi::c_int as linenr_T;
                compl_type = 0 as ::core::ffi::c_int;
            } else {
                (*st).found_all = true_0 != 0;
                if (*(*st).ins_buf).b_fname.is_null() {
                    status = INS_COMPL_CPT_CONT as ::core::ffi::c_int;
                    break '_done;
                } else {
                    compl_type = CTRL_X_DICTIONARY as ::core::ffi::c_int;
                    (*st).dict = (*(*st).ins_buf).b_fname;
                    (*st).dict_f = DICT_EXACT;
                }
            }
            if !shortmess(SHM_COMPLETIONSCAN as ::core::ffi::c_int) && !compl_autocomplete.get() {
                vim_snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    gettext(b"Scanning: %s\0".as_ptr() as *const ::core::ffi::c_char),
                    if (*(*st).ins_buf).b_fname.is_null() {
                        buf_spname((*st).ins_buf)
                    } else if (*(*st).ins_buf).b_sfname.is_null() {
                        (*(*st).ins_buf).b_fname
                    } else {
                        (*(*st).ins_buf).b_sfname
                    },
                );
                msg_progress(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    b"completion\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    b"running\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    HLF_R as ::core::ffi::c_int,
                    false_0 != 0,
                    true_0 != 0,
                );
            }
        } else if *(*st).e_cpt as ::core::ffi::c_int == NUL {
            status = INS_COMPL_CPT_END as ::core::ffi::c_int;
        } else {
            if !ctrl_x_mode_line_or_eval() {
                if *(*st).e_cpt as ::core::ffi::c_int == 'F' as ::core::ffi::c_int
                    || *(*st).e_cpt as ::core::ffi::c_int == 'o' as ::core::ffi::c_int
                {
                    compl_type = CTRL_X_FUNCTION as ::core::ffi::c_int;
                    (*st).func_cb = get_callback_if_cpt_func((*st).e_cpt, cpt_sources_index.get());
                    if (*st).func_cb.is_null() {
                        compl_type = -1 as ::core::ffi::c_int;
                    }
                } else if !skip_source {
                    if *(*st).e_cpt as ::core::ffi::c_int == 'k' as ::core::ffi::c_int
                        || *(*st).e_cpt as ::core::ffi::c_int == 's' as ::core::ffi::c_int
                    {
                        if *(*st).e_cpt as ::core::ffi::c_int == 'k' as ::core::ffi::c_int {
                            compl_type = CTRL_X_DICTIONARY as ::core::ffi::c_int;
                        } else {
                            compl_type = CTRL_X_THESAURUS as ::core::ffi::c_int;
                        }
                        (*st).e_cpt = (*st).e_cpt.offset(1);
                        if *(*st).e_cpt as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                            && *(*st).e_cpt as ::core::ffi::c_int != NUL
                        {
                            (*st).dict = (*st).e_cpt;
                            (*st).dict_f = DICT_FIRST;
                        }
                    } else if *(*st).e_cpt as ::core::ffi::c_int == 'i' as ::core::ffi::c_int {
                        compl_type = CTRL_X_PATH_PATTERNS as ::core::ffi::c_int;
                    } else if *(*st).e_cpt as ::core::ffi::c_int == 'd' as ::core::ffi::c_int {
                        compl_type = CTRL_X_PATH_DEFINES as ::core::ffi::c_int;
                    } else if *(*st).e_cpt as ::core::ffi::c_int == 'f' as ::core::ffi::c_int {
                        compl_type = CTRL_X_BUFNAMES as ::core::ffi::c_int;
                    } else if *(*st).e_cpt as ::core::ffi::c_int == ']' as ::core::ffi::c_int
                        || *(*st).e_cpt as ::core::ffi::c_int == 't' as ::core::ffi::c_int
                    {
                        compl_type = CTRL_X_TAGS as ::core::ffi::c_int;
                        if !shortmess(SHM_COMPLETIONSCAN as ::core::ffi::c_int)
                            && !compl_autocomplete.get()
                        {
                            vim_snprintf(
                                IObuff.ptr() as *mut ::core::ffi::c_char,
                                IOSIZE as size_t,
                                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                gettext(b"Scanning tags.\0".as_ptr() as *const ::core::ffi::c_char),
                            );
                            msg_progress(
                                IObuff.ptr() as *mut ::core::ffi::c_char,
                                b"completion\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                b"running\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                HLF_R as ::core::ffi::c_int,
                                false_0 != 0,
                                true_0 != 0,
                            );
                        }
                    }
                }
            }
            copy_option_part(
                &raw mut (*st).e_cpt,
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            *advance_cpt_idx = may_advance_cpt_index((*st).e_cpt);
            (*st).found_all = true_0 != 0;
            if compl_type == -1 as ::core::ffi::c_int {
                status = INS_COMPL_CPT_CONT as ::core::ffi::c_int;
            }
        }
    }
    *compl_type_arg = compl_type;
    return status;
}
unsafe extern "C" fn get_next_include_file_completion(mut compl_type: ::core::ffi::c_int) {
    find_pattern_in_path(
        (*compl_pattern.ptr()).data,
        compl_direction.get(),
        (*compl_pattern.ptr()).size,
        false_0 != 0,
        false_0 != 0,
        if compl_type == CTRL_X_PATH_DEFINES as ::core::ffi::c_int
            && compl_cont_status.get() & CONT_SOL == 0
        {
            FIND_DEFINE as ::core::ffi::c_int
        } else {
            FIND_ANY as ::core::ffi::c_int
        },
        1 as ::core::ffi::c_int,
        ACTION_EXPAND as ::core::ffi::c_int,
        1 as linenr_T,
        MAXLNUM as ::core::ffi::c_int as linenr_T,
        false_0 != 0,
        compl_autocomplete.get(),
    );
}
unsafe extern "C" fn get_next_dict_tsr_completion(
    mut compl_type: ::core::ffi::c_int,
    mut dict: *mut ::core::ffi::c_char,
    mut dict_f: ::core::ffi::c_int,
) {
    if thesaurus_func_complete(compl_type) {
        expand_by_function(
            compl_type,
            (*compl_pattern.ptr()).data,
            ::core::ptr::null_mut::<Callback>(),
        );
    } else {
        ins_compl_dictionaries(
            if !dict.is_null() {
                dict
            } else if compl_type == CTRL_X_THESAURUS as ::core::ffi::c_int {
                if *(*curbuf.get()).b_p_tsr as ::core::ffi::c_int == NUL {
                    p_tsr.get()
                } else {
                    (*curbuf.get()).b_p_tsr
                }
            } else if *(*curbuf.get()).b_p_dict as ::core::ffi::c_int == NUL {
                p_dict.get()
            } else {
                (*curbuf.get()).b_p_dict
            },
            (*compl_pattern.ptr()).data,
            if !dict.is_null() {
                dict_f
            } else {
                0 as ::core::ffi::c_int
            },
            compl_type == CTRL_X_THESAURUS as ::core::ffi::c_int,
        );
    };
}
unsafe extern "C" fn get_next_tag_completion() {
    let save_p_ic: ::core::ffi::c_int = p_ic.get();
    p_ic.set(ignorecase((*compl_pattern.ptr()).data));
    g_tag_at_cursor.set(true_0 != 0);
    let mut matches: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut num_matches: ::core::ffi::c_int = 0;
    if find_tags(
        (*compl_pattern.ptr()).data,
        &raw mut num_matches,
        &raw mut matches,
        TAG_REGEXP as ::core::ffi::c_int
            | TAG_NAMES as ::core::ffi::c_int
            | TAG_NOIC as ::core::ffi::c_int
            | TAG_INS_COMP as ::core::ffi::c_int
            | (if ctrl_x_mode_not_default() as ::core::ffi::c_int != 0 {
                TAG_VERBOSE as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }),
        TAG_MANY as ::core::ffi::c_int,
        (*curbuf.get()).b_ffname,
    ) == OK
        && num_matches > 0 as ::core::ffi::c_int
    {
        ins_compl_add_matches(num_matches, matches, p_ic.get());
    }
    g_tag_at_cursor.set(false_0 != 0);
    p_ic.set(save_p_ic);
}
unsafe extern "C" fn compare_scores(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut idx_a: ::core::ffi::c_int = *(a as *const ::core::ffi::c_int);
    let mut idx_b: ::core::ffi::c_int = *(b as *const ::core::ffi::c_int);
    let mut score_a: ::core::ffi::c_int = *(*compl_fuzzy_scores.ptr()).offset(idx_a as isize);
    let mut score_b: ::core::ffi::c_int = *(*compl_fuzzy_scores.ptr()).offset(idx_b as isize);
    return if score_a == score_b {
        if idx_a == idx_b {
            0 as ::core::ffi::c_int
        } else if idx_a < idx_b {
            -1 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        }
    } else if score_a > score_b {
        -1 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    };
}
unsafe extern "C" fn ins_compl_longest_insert(mut prefix: *mut ::core::ffi::c_char) {
    ins_compl_delete(false_0 != 0);
    ins_compl_insert_bytes(
        prefix.offset(get_compl_len() as isize),
        -1 as ::core::ffi::c_int,
    );
    ins_redraw(false_0 != 0);
}
unsafe extern "C" fn fuzzy_longest_match() {
    if compl_num_bests.get() == 0 as ::core::ffi::c_int {
        return;
    }
    let mut nn_compl: *mut compl_T = (*(*compl_first_match.get()).cp_next).cp_next;
    let mut more_candidates: bool = !nn_compl.is_null() && nn_compl != compl_first_match.get();
    let mut compl: *mut compl_T = if ctrl_x_mode_whole_line() as ::core::ffi::c_int != 0 {
        compl_first_match.get()
    } else {
        (*compl_first_match.get()).cp_next
    };
    if compl_num_bests.get() == 1 as ::core::ffi::c_int {
        if !more_candidates {
            ins_compl_longest_insert((*compl).cp_str.data);
            compl_num_bests.set(0 as ::core::ffi::c_int);
        }
        compl_num_bests.set(0 as ::core::ffi::c_int);
        return;
    }
    if compl_num_bests.get() as size_t
        > (SIZE_MAX as usize).wrapping_div(::core::mem::size_of::<*mut compl_T>())
    {
        return;
    }
    compl_best_matches.set(xmalloc(
        (compl_num_bests.get() as size_t).wrapping_mul(::core::mem::size_of::<*mut compl_T>()),
    ) as *mut *mut compl_T);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while !compl.is_null() && i < compl_num_bests.get() {
        *(*compl_best_matches.ptr()).offset(i as isize) = compl;
        compl = (*compl).cp_next;
        i += 1;
    }
    let mut prefix: *mut ::core::ffi::c_char = (**(*compl_best_matches.ptr())
        .offset(0 as ::core::ffi::c_int as isize))
    .cp_str
    .data;
    let mut prefix_len: ::core::ffi::c_int = (**(*compl_best_matches.ptr())
        .offset(0 as ::core::ffi::c_int as isize))
    .cp_str
    .size as ::core::ffi::c_int;
    let mut i_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i_0 < compl_num_bests.get() {
        let mut match_str: *mut ::core::ffi::c_char = (**(*compl_best_matches.ptr())
            .offset(i_0 as isize))
        .cp_str
        .data;
        let mut prefix_ptr: *mut ::core::ffi::c_char = prefix;
        let mut match_ptr: *mut ::core::ffi::c_char = match_str;
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while j < prefix_len
            && *match_ptr as ::core::ffi::c_int != NUL
            && *prefix_ptr as ::core::ffi::c_int != NUL
        {
            if strncmp(prefix_ptr, match_ptr, utfc_ptr2len(prefix_ptr) as size_t)
                != 0 as ::core::ffi::c_int
            {
                break;
            }
            prefix_ptr = prefix_ptr.offset(utfc_ptr2len(prefix_ptr) as isize);
            match_ptr = match_ptr.offset(utfc_ptr2len(match_ptr) as isize);
            j += 1;
        }
        if j > 0 as ::core::ffi::c_int {
            prefix_len = j;
        }
        i_0 += 1;
    }
    let mut leader: *mut ::core::ffi::c_char = ins_compl_leader();
    let mut leader_len: size_t = ins_compl_leader_len();
    if !(leader_len > 0 as size_t && strncmp(prefix, leader, leader_len) != 0 as ::core::ffi::c_int)
    {
        prefix = xmemdupz(prefix as *const ::core::ffi::c_void, prefix_len as size_t)
            as *mut ::core::ffi::c_char;
        ins_compl_longest_insert(prefix);
        xfree(prefix as *mut ::core::ffi::c_void);
    }
    xfree(compl_best_matches.get() as *mut ::core::ffi::c_void);
    compl_best_matches.set(::core::ptr::null_mut::<*mut compl_T>());
    compl_num_bests.set(0 as ::core::ffi::c_int);
}
unsafe extern "C" fn get_next_filename_completion() {
    let mut matches: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut num_matches: ::core::ffi::c_int = 0;
    let mut leader: *mut ::core::ffi::c_char = ins_compl_leader();
    let mut leader_len: size_t = ins_compl_leader_len();
    let mut in_fuzzy_collect: bool =
        cot_fuzzy() as ::core::ffi::c_int != 0 && leader_len > 0 as size_t;
    let mut need_collect_bests: bool = in_fuzzy_collect as ::core::ffi::c_int != 0
        && compl_get_longest.get() as ::core::ffi::c_int != 0;
    let mut max_score: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut dir: Direction = compl_direction.get();
    let mut pathsep: ::core::ffi::c_char = PATHSEP as ::core::ffi::c_char;
    if in_fuzzy_collect {
        let mut last_sep: *mut ::core::ffi::c_char = strrchr(leader, pathsep as ::core::ffi::c_int);
        if last_sep.is_null() {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*compl_pattern.ptr()).data as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
            (*compl_pattern.ptr()).size = 0 as size_t;
            compl_pattern.set(cbuf_to_string(
                b"*\0".as_ptr() as *const ::core::ffi::c_char,
                1 as size_t,
            ));
        } else if *last_sep.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
            in_fuzzy_collect = false_0 != 0;
        } else {
            let mut path_len: size_t =
                (last_sep.offset_from(leader) as size_t).wrapping_add(1 as size_t);
            let mut path_with_wildcard: *mut ::core::ffi::c_char =
                xmalloc(path_len.wrapping_add(2 as size_t)) as *mut ::core::ffi::c_char;
            vim_snprintf(
                path_with_wildcard,
                path_len.wrapping_add(2 as size_t),
                b"%*.*s*\0".as_ptr() as *const ::core::ffi::c_char,
                path_len as ::core::ffi::c_int,
                path_len as ::core::ffi::c_int,
                leader,
            );
            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                &raw mut (*compl_pattern.ptr()).data as *mut *mut ::core::ffi::c_void;
            xfree(*ptr__0);
            *ptr__0 = NULL;
            let _ = *ptr__0;
            (*compl_pattern.ptr()).size = 0 as size_t;
            (*compl_pattern.ptr()).data = path_with_wildcard;
            (*compl_pattern.ptr()).size = path_len.wrapping_add(1 as size_t);
            leader = last_sep.offset(1 as ::core::ffi::c_int as isize);
            leader_len = leader_len.wrapping_sub(path_len);
        }
    }
    if expand_wildcards(
        1 as ::core::ffi::c_int,
        &raw mut (*compl_pattern.ptr()).data,
        &raw mut num_matches,
        &raw mut matches,
        EW_FILE as ::core::ffi::c_int
            | EW_DIR as ::core::ffi::c_int
            | EW_ADDSLASH as ::core::ffi::c_int
            | EW_SILENT as ::core::ffi::c_int,
    ) != OK
    {
        return;
    }
    tilde_replace((*compl_pattern.ptr()).data, num_matches, matches);
    if in_fuzzy_collect {
        let mut fuzzy_indices: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        ga_init(
            &raw mut fuzzy_indices,
            ::core::mem::size_of::<::core::ffi::c_int>() as ::core::ffi::c_int,
            10 as ::core::ffi::c_int,
        );
        compl_fuzzy_scores.set(xmalloc(
            ::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul(num_matches as size_t),
        ) as *mut ::core::ffi::c_int);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < num_matches {
            let mut ptr: *mut ::core::ffi::c_char = *matches.offset(i as isize);
            let mut score: ::core::ffi::c_int = fuzzy_match_str(ptr, leader);
            if score != FUZZY_SCORE_NONE as ::core::ffi::c_int {
                ga_grow(&raw mut fuzzy_indices, 1 as ::core::ffi::c_int);
                *(fuzzy_indices.ga_data as *mut ::core::ffi::c_int)
                    .offset(fuzzy_indices.ga_len as isize) = i;
                fuzzy_indices.ga_len += 1;
                *(*compl_fuzzy_scores.ptr()).offset(i as isize) = score;
            }
            i += 1;
        }
        if fuzzy_indices.ga_len > 0 as ::core::ffi::c_int {
            let mut fuzzy_indices_data: *mut ::core::ffi::c_int =
                fuzzy_indices.ga_data as *mut ::core::ffi::c_int;
            qsort(
                fuzzy_indices_data as *mut ::core::ffi::c_void,
                fuzzy_indices.ga_len as size_t,
                ::core::mem::size_of::<::core::ffi::c_int>(),
                Some(
                    compare_scores
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_void,
                            *const ::core::ffi::c_void,
                        ) -> ::core::ffi::c_int,
                ),
            );
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_0 < fuzzy_indices.ga_len {
                let mut match_0: *mut ::core::ffi::c_char =
                    *matches.offset(*fuzzy_indices_data.offset(i_0 as isize) as isize);
                let mut current_score: ::core::ffi::c_int = *(*compl_fuzzy_scores.ptr())
                    .offset(*fuzzy_indices_data.offset(i_0 as isize) as isize);
                if ins_compl_add(
                    match_0,
                    -1 as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null::<*mut ::core::ffi::c_char>(),
                    false_0 != 0,
                    ::core::ptr::null_mut::<typval_T>(),
                    dir,
                    CP_FAST as ::core::ffi::c_int
                        | (if p_fic.get() != 0 || p_wic.get() != 0 {
                            CP_ICASE as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        }),
                    false_0 != 0,
                    ::core::ptr::null::<::core::ffi::c_int>(),
                    current_score,
                ) == OK
                {
                    dir = FORWARD;
                }
                if need_collect_bests {
                    if i_0 == 0 as ::core::ffi::c_int || current_score == max_score {
                        (*compl_num_bests.ptr()) += 1;
                        max_score = current_score;
                    }
                }
                i_0 += 1;
            }
            FreeWild(num_matches, matches);
        } else if leader_len > 0 as size_t {
            FreeWild(num_matches, matches);
            num_matches = 0 as ::core::ffi::c_int;
        }
        xfree(compl_fuzzy_scores.get() as *mut ::core::ffi::c_void);
        ga_clear(&raw mut fuzzy_indices);
        if compl_num_bests.get() > 0 as ::core::ffi::c_int
            && compl_get_longest.get() as ::core::ffi::c_int != 0
        {
            fuzzy_longest_match();
        }
        return;
    }
    if num_matches > 0 as ::core::ffi::c_int {
        ins_compl_add_matches(
            num_matches,
            matches,
            (p_fic.get() != 0 || p_wic.get() != 0) as ::core::ffi::c_int,
        );
    }
}
unsafe extern "C" fn get_next_cmdline_completion() {
    let mut matches: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut num_matches: ::core::ffi::c_int = 0;
    if expand_cmdline(
        compl_xp.ptr(),
        (*compl_pattern.ptr()).data,
        (*compl_pattern.ptr()).size as ::core::ffi::c_int,
        &raw mut num_matches,
        &raw mut matches,
    ) == EXPAND_OK as ::core::ffi::c_int
    {
        ins_compl_add_matches(num_matches, matches, false_0);
    }
}
unsafe extern "C" fn get_next_spell_completion(mut lnum: linenr_T) {
    let mut matches: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut num_matches: ::core::ffi::c_int =
        expand_spelling(lnum, (*compl_pattern.ptr()).data, &raw mut matches);
    if num_matches > 0 as ::core::ffi::c_int {
        ins_compl_add_matches(num_matches, matches, p_ic.get());
    } else {
        xfree(matches as *mut ::core::ffi::c_void);
    };
}
unsafe extern "C" fn ins_compl_get_next_word_or_line(
    mut ins_buf: *mut buf_T,
    mut cur_match_pos: *mut pos_T,
    mut match_len: *mut ::core::ffi::c_int,
    mut cont_s_ipos: *mut bool,
) -> *mut ::core::ffi::c_char {
    *match_len = 0 as ::core::ffi::c_int;
    let mut ptr: *mut ::core::ffi::c_char =
        ml_get_buf(ins_buf, (*cur_match_pos).lnum).offset((*cur_match_pos).col as isize);
    let mut len: ::core::ffi::c_int =
        ml_get_buf_len(ins_buf, (*cur_match_pos).lnum) - (*cur_match_pos).col as ::core::ffi::c_int;
    if ctrl_x_mode_line_or_eval() {
        if compl_status_adding() {
            if (*cur_match_pos).lnum >= (*ins_buf).b_ml.ml_line_count {
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            ptr = ml_get_buf(ins_buf, (*cur_match_pos).lnum + 1 as linenr_T);
            len = ml_get_buf_len(ins_buf, (*cur_match_pos).lnum + 1 as linenr_T)
                as ::core::ffi::c_int;
            if p_paste.get() == 0 {
                let mut tmp_ptr: *mut ::core::ffi::c_char = ptr;
                ptr = skipwhite(tmp_ptr);
                len -= ptr.offset_from(tmp_ptr) as ::core::ffi::c_int;
            }
        }
    } else {
        let mut tmp_ptr_0: *mut ::core::ffi::c_char = ptr;
        if compl_status_adding() as ::core::ffi::c_int != 0 && compl_length.get() <= len {
            tmp_ptr_0 = tmp_ptr_0.offset(compl_length.get() as isize);
            if vim_iswordp(tmp_ptr_0) {
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            tmp_ptr_0 = find_word_start(tmp_ptr_0);
        }
        tmp_ptr_0 = find_word_end(tmp_ptr_0);
        len = tmp_ptr_0.offset_from(ptr) as ::core::ffi::c_int;
        if compl_status_adding() as ::core::ffi::c_int != 0 && len == compl_length.get() {
            if (*cur_match_pos).lnum < (*ins_buf).b_ml.ml_line_count {
                strncpy(IObuff.ptr() as *mut ::core::ffi::c_char, ptr, len as size_t);
                ptr = ml_get_buf(ins_buf, (*cur_match_pos).lnum + 1 as linenr_T);
                ptr = skipwhite(ptr);
                tmp_ptr_0 = ptr;
                tmp_ptr_0 = find_word_start(tmp_ptr_0);
                tmp_ptr_0 = find_word_end(tmp_ptr_0);
                if tmp_ptr_0 > ptr {
                    if *ptr as ::core::ffi::c_int != ')' as ::core::ffi::c_int
                        && (*IObuff.ptr())[(len - 1 as ::core::ffi::c_int) as usize]
                            as ::core::ffi::c_int
                            != TAB
                    {
                        if (*IObuff.ptr())[(len - 1 as ::core::ffi::c_int) as usize]
                            as ::core::ffi::c_int
                            != ' ' as ::core::ffi::c_int
                        {
                            let c2rust_fresh3 = len;
                            len = len + 1;
                            (*IObuff.ptr())[c2rust_fresh3 as usize] = ' ' as ::core::ffi::c_char;
                        }
                        if p_js.get() != 0
                            && ((*IObuff.ptr())[(len - 2 as ::core::ffi::c_int) as usize]
                                as ::core::ffi::c_int
                                == '.' as ::core::ffi::c_int
                                || (*IObuff.ptr())[(len - 2 as ::core::ffi::c_int) as usize]
                                    as ::core::ffi::c_int
                                    == '?' as ::core::ffi::c_int
                                || (*IObuff.ptr())[(len - 2 as ::core::ffi::c_int) as usize]
                                    as ::core::ffi::c_int
                                    == '!' as ::core::ffi::c_int)
                        {
                            let c2rust_fresh4 = len;
                            len = len + 1;
                            (*IObuff.ptr())[c2rust_fresh4 as usize] = ' ' as ::core::ffi::c_char;
                        }
                    }
                    if tmp_ptr_0.offset_from(ptr) >= (IOSIZE - len) as isize {
                        tmp_ptr_0 = ptr
                            .offset(IOSIZE as isize)
                            .offset(-(len as isize))
                            .offset(-(1 as ::core::ffi::c_int as isize));
                    }
                    xstrlcpy(
                        (IObuff.ptr() as *mut ::core::ffi::c_char).offset(len as isize),
                        ptr,
                        (IOSIZE - len) as size_t,
                    );
                    len += tmp_ptr_0.offset_from(ptr) as ::core::ffi::c_int;
                    *cont_s_ipos = true_0 != 0;
                }
                (*IObuff.ptr())[len as usize] = NUL as ::core::ffi::c_char;
                ptr = IObuff.ptr() as *mut ::core::ffi::c_char;
            }
            if len == compl_length.get() {
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
        }
    }
    *match_len = len;
    return ptr;
}
unsafe extern "C" fn get_next_default_completion(
    mut st: *mut ins_compl_next_state_T,
    mut start_pos: *mut pos_T,
) -> ::core::ffi::c_int {
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut in_fuzzy_collect: bool = !compl_status_adding()
        && cot_fuzzy() as ::core::ffi::c_int != 0
        && compl_length.get() > 0 as ::core::ffi::c_int;
    let mut leader: *mut ::core::ffi::c_char = ins_compl_leader();
    let mut score: ::core::ffi::c_int = FUZZY_SCORE_NONE as ::core::ffi::c_int;
    let in_curbuf: bool = (*st).ins_buf == curbuf.get();
    let save_p_scs: ::core::ffi::c_int = p_scs.get();
    '_c2rust_label: {
        if !(*st).ins_buf.is_null() {
        } else {
            __assert_fail(
                b"st->ins_buf\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                4275 as ::core::ffi::c_uint,
                b"int get_next_default_completion(ins_compl_next_state_T *, pos_T *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if (*(*st).ins_buf).b_p_inf != 0 {
        p_scs.set(false_0);
    }
    let save_p_ws: ::core::ffi::c_int = p_ws.get();
    if !in_curbuf {
        p_ws.set(false_0);
    } else if *(*st).e_cpt as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
        p_ws.set(true_0);
    }
    let mut looped_around: bool = false_0 != 0;
    let mut found_new_match: ::core::ffi::c_int = FAIL;
    loop {
        let mut cont_s_ipos: bool = false_0 != 0;
        (*msg_silent.ptr()) += 1;
        if in_fuzzy_collect {
            found_new_match = search_for_fuzzy_match(
                (*st).ins_buf,
                (*st).cur_match_pos,
                leader,
                compl_direction.get() as ::core::ffi::c_int,
                start_pos,
                &raw mut len,
                &raw mut ptr,
                &raw mut score,
            ) as ::core::ffi::c_int;
        } else if ctrl_x_mode_whole_line() as ::core::ffi::c_int != 0
            || ctrl_x_mode_eval() as ::core::ffi::c_int != 0
            || compl_cont_status.get() & CONT_SOL != 0
        {
            found_new_match = search_for_exact_line(
                (*st).ins_buf,
                (*st).cur_match_pos,
                compl_direction.get(),
                (*compl_pattern.ptr()).data,
            );
        } else {
            found_new_match = searchit(
                ::core::ptr::null_mut::<win_T>(),
                (*st).ins_buf,
                (*st).cur_match_pos,
                ::core::ptr::null_mut::<pos_T>(),
                compl_direction.get(),
                (*compl_pattern.ptr()).data,
                (*compl_pattern.ptr()).size,
                1 as ::core::ffi::c_int,
                SEARCH_KEEP as ::core::ffi::c_int + SEARCH_NFMSG as ::core::ffi::c_int,
                RE_LAST as ::core::ffi::c_int,
                ::core::ptr::null_mut::<searchit_arg_T>(),
            );
        }
        (*msg_silent.ptr()) -= 1;
        if !compl_started.get() || (*st).set_match_pos as ::core::ffi::c_int != 0 {
            compl_started.set(true_0 != 0);
            (*st).first_match_pos = *(*st).cur_match_pos;
            (*st).last_match_pos = *(*st).cur_match_pos;
            (*st).set_match_pos = false_0 != 0;
        } else if (*st).first_match_pos.lnum == (*st).last_match_pos.lnum
            && (*st).first_match_pos.col == (*st).last_match_pos.col
        {
            found_new_match = FAIL;
        } else if compl_dir_forward() as ::core::ffi::c_int != 0
            && ((*st).prev_match_pos.lnum > (*(*st).cur_match_pos).lnum
                || (*st).prev_match_pos.lnum == (*(*st).cur_match_pos).lnum
                    && (*st).prev_match_pos.col >= (*(*st).cur_match_pos).col)
        {
            if looped_around {
                found_new_match = FAIL;
            } else {
                looped_around = true_0 != 0;
            }
        } else if !compl_dir_forward()
            && ((*st).prev_match_pos.lnum < (*(*st).cur_match_pos).lnum
                || (*st).prev_match_pos.lnum == (*(*st).cur_match_pos).lnum
                    && (*st).prev_match_pos.col <= (*(*st).cur_match_pos).col)
        {
            if looped_around {
                found_new_match = FAIL;
            } else {
                looped_around = true_0 != 0;
            }
        }
        (*st).prev_match_pos = *(*st).cur_match_pos;
        if found_new_match == FAIL {
            break;
        }
        if compl_status_adding() as ::core::ffi::c_int != 0
            && in_curbuf as ::core::ffi::c_int != 0
            && (*start_pos).lnum == (*(*st).cur_match_pos).lnum
            && (*start_pos).col == (*(*st).cur_match_pos).col
        {
            continue;
        }
        if !in_fuzzy_collect {
            ptr = ins_compl_get_next_word_or_line(
                (*st).ins_buf,
                (*st).cur_match_pos,
                &raw mut len,
                &raw mut cont_s_ipos,
            );
        }
        if ptr.is_null()
            || ins_compl_has_preinsert() as ::core::ffi::c_int != 0
                && strcmp(ptr, ins_compl_leader()) == 0 as ::core::ffi::c_int
        {
            continue;
        }
        if is_nearest_active() as ::core::ffi::c_int != 0 && in_curbuf as ::core::ffi::c_int != 0 {
            score =
                ((*(*st).cur_match_pos).lnum - (*curwin.get()).w_cursor.lnum) as ::core::ffi::c_int;
            if score < 0 as ::core::ffi::c_int {
                score = -score;
            }
        }
        if ins_compl_add_infercase(
            ptr,
            len,
            p_ic.get() != 0,
            if in_curbuf as ::core::ffi::c_int != 0 {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            } else {
                (*(*st).ins_buf).b_sfname
            },
            kDirectionNotSet,
            cont_s_ipos,
            score,
        ) == NOTDONE
        {
            continue;
        }
        if in_fuzzy_collect as ::core::ffi::c_int != 0
            && score == (*(*compl_first_match.get()).cp_next).cp_score
        {
            (*compl_num_bests.ptr()) += 1;
        }
        found_new_match = OK;
        break;
    }
    p_scs.set(save_p_scs);
    p_ws.set(save_p_ws);
    return found_new_match;
}
unsafe extern "C" fn get_register_completion() {
    let mut dir: Direction = compl_direction.get();
    let mut adding_mode: bool = compl_status_adding();
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < NUM_REGISTERS as ::core::ffi::c_int {
        let mut regname: ::core::ffi::c_int = get_register_name(i);
        if !(!valid_yank_reg(regname, false_0 != 0) || regname == '_' as ::core::ffi::c_int) {
            let mut reg: *mut yankreg_T = copy_register(regname);
            if (*reg).y_array.is_null() || (*reg).y_size == 0 as size_t {
                free_register(reg);
                xfree(reg as *mut ::core::ffi::c_void);
            } else {
                let mut j: size_t = 0 as size_t;
                while j < (*reg).y_size {
                    let mut str: *mut ::core::ffi::c_char =
                        (*(*reg).y_array.offset(j as isize)).data;
                    if !str.is_null() {
                        if adding_mode {
                            let mut str_len: ::core::ffi::c_int = strlen(str) as ::core::ffi::c_int;
                            if str_len != 0 as ::core::ffi::c_int {
                                if (*compl_orig_text.ptr()).data.is_null()
                                    || (if p_ic.get() != 0 {
                                        (strncasecmp(
                                            str,
                                            (*compl_orig_text.ptr()).data,
                                            (*compl_orig_text.ptr()).size,
                                        ) == 0 as ::core::ffi::c_int)
                                            as ::core::ffi::c_int
                                    } else {
                                        (strncmp(
                                            str,
                                            (*compl_orig_text.ptr()).data,
                                            (*compl_orig_text.ptr()).size,
                                        ) == 0 as ::core::ffi::c_int)
                                            as ::core::ffi::c_int
                                    }) != 0
                                {
                                    if ins_compl_add_infercase(
                                        str,
                                        str_len,
                                        p_ic.get() != 0,
                                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                        dir,
                                        false_0 != 0,
                                        FUZZY_SCORE_NONE as ::core::ffi::c_int,
                                    ) == OK
                                    {
                                        dir = FORWARD;
                                    }
                                }
                            }
                        } else {
                            let mut str_end: *mut ::core::ffi::c_char =
                                str.offset(strlen(str) as isize);
                            let mut p: *mut ::core::ffi::c_char = str;
                            while p < str_end && *p as ::core::ffi::c_int != NUL {
                                let mut old_p: *mut ::core::ffi::c_char = p;
                                p = find_word_start(p);
                                if p >= str_end || *p as ::core::ffi::c_int == NUL {
                                    break;
                                }
                                let mut word_end: *mut ::core::ffi::c_char = find_word_end(p);
                                if word_end <= p {
                                    word_end = p.offset(utfc_ptr2len(p) as isize);
                                }
                                if word_end > str_end {
                                    word_end = str_end;
                                }
                                let mut len: ::core::ffi::c_int =
                                    word_end.offset_from(p) as ::core::ffi::c_int;
                                if len > 0 as ::core::ffi::c_int
                                    && ((*compl_orig_text.ptr()).data.is_null()
                                        || (if p_ic.get() != 0 {
                                            (strncasecmp(
                                                p,
                                                (*compl_orig_text.ptr()).data,
                                                (*compl_orig_text.ptr()).size,
                                            ) == 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int
                                        } else {
                                            (strncmp(
                                                p,
                                                (*compl_orig_text.ptr()).data,
                                                (*compl_orig_text.ptr()).size,
                                            ) == 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int
                                        }) != 0)
                                {
                                    if ins_compl_add_infercase(
                                        p,
                                        len,
                                        p_ic.get() != 0,
                                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                        dir,
                                        false_0 != 0,
                                        FUZZY_SCORE_NONE as ::core::ffi::c_int,
                                    ) == OK
                                    {
                                        dir = FORWARD;
                                    }
                                }
                                p = word_end;
                                if p <= old_p {
                                    p = old_p.offset(utfc_ptr2len(old_p) as isize);
                                }
                            }
                        }
                    }
                    j = j.wrapping_add(1);
                }
                free_register(reg);
                xfree(reg as *mut ::core::ffi::c_void);
            }
        }
        i += 1;
    }
}
unsafe extern "C" fn get_callback_if_cpt_func(
    mut p: *mut ::core::ffi::c_char,
    mut idx: ::core::ffi::c_int,
) -> *mut Callback {
    if *p as ::core::ffi::c_int == 'o' as ::core::ffi::c_int {
        return &raw mut (*curbuf.get()).b_ofu_cb;
    }
    if *p as ::core::ffi::c_int == 'F' as ::core::ffi::c_int {
        p = p.offset(1);
        if *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int && *p as ::core::ffi::c_int != NUL
        {
            return if (*(*curbuf.get()).b_p_cpt_cb.offset(idx as isize)).type_0
                as ::core::ffi::c_uint
                != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*curbuf.get()).b_p_cpt_cb.offset(idx as isize)
            } else {
                ::core::ptr::null_mut::<Callback>()
            };
        } else {
            return &raw mut (*curbuf.get()).b_cfu_cb;
        }
    }
    return ::core::ptr::null_mut::<Callback>();
}
unsafe extern "C" fn get_next_completion_match(
    mut type_0: ::core::ffi::c_int,
    mut st: *mut ins_compl_next_state_T,
    mut ini: *mut pos_T,
) -> bool {
    let mut found_new_match: ::core::ffi::c_int = FAIL;
    match type_0 {
        -1 => {}
        262 | 263 => {
            get_next_include_file_completion(type_0);
        }
        265 | 266 => {
            get_next_dict_tsr_completion(type_0, (*st).dict, (*st).dict_f);
            (*st).dict = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        261 => {
            get_next_tag_completion();
        }
        4 => {
            get_next_filename_completion();
        }
        11 | 17 => {
            get_next_cmdline_completion();
        }
        12 => {
            if ctrl_x_mode_normal() {
                get_cpt_func_completion_matches((*st).func_cb);
            } else {
                expand_by_function(
                    type_0,
                    (*compl_pattern.ptr()).data,
                    ::core::ptr::null_mut::<Callback>(),
                );
            }
        }
        13 => {
            expand_by_function(
                type_0,
                (*compl_pattern.ptr()).data,
                ::core::ptr::null_mut::<Callback>(),
            );
        }
        14 => {
            get_next_spell_completion((*st).first_match_pos.lnum);
        }
        18 => {
            get_next_bufname_token();
        }
        19 => {
            get_register_completion();
        }
        _ => {
            found_new_match = get_next_default_completion(st, ini);
            if found_new_match == FAIL && (*st).ins_buf == curbuf.get() {
                (*st).found_all = true_0 != 0;
            }
        }
    }
    if type_0 != 0 as ::core::ffi::c_int && compl_curr_match.get() != compl_old_match.get() {
        found_new_match = OK;
    }
    return found_new_match != 0;
}
unsafe extern "C" fn get_next_bufname_token() {
    let mut b: *mut buf_T = firstbuf.get();
    while !b.is_null() {
        if (*b).b_p_bl != 0 && !(*b).b_sfname.is_null() {
            let mut tail: *mut ::core::ffi::c_char = path_tail((*b).b_sfname);
            if strncmp(
                tail,
                (*compl_orig_text.ptr()).data,
                (*compl_orig_text.ptr()).size,
            ) == 0 as ::core::ffi::c_int
            {
                ins_compl_add(
                    tail,
                    strlen(tail) as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null::<*mut ::core::ffi::c_char>(),
                    false_0 != 0,
                    ::core::ptr::null_mut::<typval_T>(),
                    kDirectionNotSet,
                    if p_ic.get() != 0 {
                        CP_ICASE as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    },
                    false_0 != 0,
                    ::core::ptr::null::<::core::ffi::c_int>(),
                    FUZZY_SCORE_NONE as ::core::ffi::c_int,
                );
            }
        }
        b = (*b).b_next;
    }
}
unsafe extern "C" fn strip_caret_numbers_in_place(mut str: *mut ::core::ffi::c_char) {
    let mut read: *mut ::core::ffi::c_char = str;
    let mut write: *mut ::core::ffi::c_char = str;
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if str.is_null() {
        return;
    }
    while *read != 0 {
        if *read as ::core::ffi::c_int == '^' as ::core::ffi::c_int {
            p = read.offset(1 as ::core::ffi::c_int as isize);
            while ascii_isdigit(*p as ::core::ffi::c_int) {
                p = p.offset(1);
            }
            if (*p as ::core::ffi::c_int == ',' as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == '\0' as ::core::ffi::c_int)
                && p != read.offset(1 as ::core::ffi::c_int as isize)
            {
                read = p;
            } else {
                let c2rust_fresh5 = read;
                read = read.offset(1);
                let c2rust_fresh6 = write;
                write = write.offset(1);
                *c2rust_fresh6 = *c2rust_fresh5;
            }
        } else {
            let c2rust_fresh7 = read;
            read = read.offset(1);
            let c2rust_fresh8 = write;
            write = write.offset(1);
            *c2rust_fresh8 = *c2rust_fresh7;
        }
    }
    *write = '\0' as ::core::ffi::c_char;
}
unsafe extern "C" fn prepare_cpt_compl_funcs() {
    let mut cpt: *mut ::core::ffi::c_char = xstrdup((*curbuf.get()).b_p_cpt);
    strip_caret_numbers_in_place(cpt);
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char = cpt;
    while *p != 0 {
        while *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
        {
            p = p.offset(1);
        }
        if *p as ::core::ffi::c_int == NUL {
            break;
        }
        let mut cb: *mut Callback = get_callback_if_cpt_func(p, idx);
        if !cb.is_null() {
            let mut startcol: ::core::ffi::c_int = 0;
            if get_userdefined_compl_info((*curwin.get()).w_cursor.col, cb, &raw mut startcol)
                == FAIL
            {
                if startcol == -3 as ::core::ffi::c_int {
                    (*(*cpt_sources_array.ptr()).offset(idx as isize)).cs_refresh_always =
                        false_0 != 0;
                } else {
                    startcol = -2 as ::core::ffi::c_int;
                }
            } else if startcol < 0 as ::core::ffi::c_int || startcol > (*curwin.get()).w_cursor.col
            {
                startcol = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
            }
            (*(*cpt_sources_array.ptr()).offset(idx as isize)).cs_startcol = startcol;
        } else {
            (*(*cpt_sources_array.ptr()).offset(idx as isize)).cs_startcol =
                -3 as ::core::ffi::c_int;
        }
        copy_option_part(
            &raw mut p,
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        idx += 1;
    }
    xfree(cpt as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn compl_source_start_timer(mut source_idx: ::core::ffi::c_int) {
    if compl_autocomplete.get() as ::core::ffi::c_int != 0 || p_cto.get() > 0 as OptInt {
        (*(*cpt_sources_array.ptr()).offset(source_idx as isize)).compl_start_tv = os_hrtime();
        compl_time_slice_expired.set(false_0 != 0);
    }
}
unsafe extern "C" fn advance_cpt_sources_index_safe() -> ::core::ffi::c_int {
    if cpt_sources_index.get() >= 0 as ::core::ffi::c_int
        && cpt_sources_index.get() < cpt_sources_count.get() - 1 as ::core::ffi::c_int
    {
        (*cpt_sources_index.ptr()) += 1;
        return OK;
    }
    semsg(
        gettext(&raw const e_list_index_out_of_range_nr as *const ::core::ffi::c_char),
        cpt_sources_index.get(),
    );
    return FAIL;
}
unsafe extern "C" fn ins_compl_get_exp(mut ini: *mut pos_T) -> ::core::ffi::c_int {
    static st: GlobalCell<ins_compl_next_state_T> = GlobalCell::new(ins_compl_next_state_T {
        e_cpt_copy: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        e_cpt: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ins_buf: ::core::ptr::null_mut::<buf_T>(),
        cur_match_pos: ::core::ptr::null_mut::<pos_T>(),
        prev_match_pos: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        set_match_pos: false,
        first_match_pos: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        last_match_pos: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        found_all: false,
        dict: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        dict_f: 0,
        func_cb: ::core::ptr::null_mut::<Callback>(),
    });
    static st_cleared: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let mut found_new_match: ::core::ffi::c_int = 0;
    let mut type_0: ::core::ffi::c_int = ctrl_x_mode.get();
    let mut may_advance_cpt_idx: bool = false_0 != 0;
    let mut start_pos: pos_T = *ini;
    '_c2rust_label: {
        if !(*curbuf.ptr()).is_null() {
        } else {
            __assert_fail(
                b"curbuf != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                4690 as ::core::ffi::c_uint,
                b"int ins_compl_get_exp(pos_T *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if !compl_started.get() {
        let mut buf: *mut buf_T = firstbuf.get();
        while !buf.is_null() {
            (*buf).b_scanned = false_0 != 0;
            buf = (*buf).b_next;
        }
        if !st_cleared.get() {
            memset(
                st.ptr() as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<ins_compl_next_state_T>(),
            );
            st_cleared.set(true_0 != 0);
        }
        (*st.ptr()).found_all = false_0 != 0;
        (*st.ptr()).ins_buf = curbuf.get();
        xfree((*st.ptr()).e_cpt_copy as *mut ::core::ffi::c_void);
        (*st.ptr()).e_cpt_copy = xstrdup(if compl_cont_status.get() & CONT_LOCAL != 0 {
            b".\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            (*curbuf.get()).b_p_cpt as *const ::core::ffi::c_char
        });
        strip_caret_numbers_in_place((*st.ptr()).e_cpt_copy);
        (*st.ptr()).e_cpt = (*st.ptr()).e_cpt_copy;
        if compl_autocomplete.get() as ::core::ffi::c_int != 0
            && is_nearest_active() as ::core::ffi::c_int != 0
        {
            start_pos.lnum = if 1 as linenr_T > start_pos.lnum - 1000 as linenr_T {
                1 as linenr_T
            } else {
                start_pos.lnum - 1000 as linenr_T
            };
            start_pos.col = 0 as ::core::ffi::c_int as colnr_T;
        }
        (*st.ptr()).first_match_pos = start_pos;
        (*st.ptr()).last_match_pos = (*st.ptr()).first_match_pos;
    } else if (*st.ptr()).ins_buf != curbuf.get() && !buf_valid((*st.ptr()).ins_buf) {
        (*st.ptr()).ins_buf = curbuf.get();
    }
    '_c2rust_label_0: {
        if !(*st.ptr()).ins_buf.is_null() {
        } else {
            __assert_fail(
                b"st.ins_buf != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                4718 as ::core::ffi::c_uint,
                b"int ins_compl_get_exp(pos_T *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    compl_old_match.set(compl_curr_match.get());
    (*st.ptr()).cur_match_pos = if compl_dir_forward() as ::core::ffi::c_int != 0 {
        &raw mut (*st.ptr()).last_match_pos
    } else {
        &raw mut (*st.ptr()).first_match_pos
    };
    let mut normal_mode_strict: bool = ctrl_x_mode_normal() as ::core::ffi::c_int != 0
        && !ctrl_x_mode_line_or_eval()
        && compl_cont_status.get() & CONT_LOCAL == 0
        && !(*cpt_sources_array.ptr()).is_null();
    if normal_mode_strict {
        cpt_sources_index.set(0 as ::core::ffi::c_int);
        if compl_autocomplete.get() as ::core::ffi::c_int != 0 || p_cto.get() > 0 as OptInt {
            compl_source_start_timer(0 as ::core::ffi::c_int);
            compl_time_slice_expired.set(false_0 != 0);
            compl_timeout_ms.set(if compl_autocomplete.get() as ::core::ffi::c_int != 0 {
                (if 80 as OptInt > p_act.get() {
                    80 as OptInt
                } else {
                    p_act.get()
                }) as uint64_t
            } else {
                p_cto.get() as uint64_t
            });
        }
    }
    loop {
        found_new_match = FAIL;
        (*st.ptr()).set_match_pos = false_0 != 0;
        if (ctrl_x_mode_normal() as ::core::ffi::c_int != 0
            || ctrl_x_mode_line_or_eval() as ::core::ffi::c_int != 0)
            && (!compl_started.get() || (*st.ptr()).found_all as ::core::ffi::c_int != 0)
        {
            let mut status: ::core::ffi::c_int = process_next_cpt_value(
                st.ptr(),
                &raw mut type_0,
                &raw mut start_pos,
                cot_fuzzy(),
                &raw mut may_advance_cpt_idx,
            );
            if status == INS_COMPL_CPT_END as ::core::ffi::c_int {
                break;
            }
            if status == INS_COMPL_CPT_CONT as ::core::ffi::c_int {
                if !may_advance_cpt_idx {
                    continue;
                }
                if advance_cpt_sources_index_safe() == 0 {
                    break;
                }
                compl_source_start_timer(cpt_sources_index.get());
                continue;
            }
        }
        let mut compl_timeout_save: uint64_t = 0 as uint64_t;
        if normal_mode_strict as ::core::ffi::c_int != 0
            && type_0 == CTRL_X_FUNCTION as ::core::ffi::c_int
            && (compl_autocomplete.get() as ::core::ffi::c_int != 0 || p_cto.get() > 0 as OptInt)
        {
            compl_timeout_save = compl_timeout_ms.get();
            compl_timeout_ms.set(
                (if compl_from_nonkeyword.get() as ::core::ffi::c_int != 0 {
                    COMPL_FUNC_TIMEOUT_NON_KW_MS
                } else {
                    COMPL_FUNC_TIMEOUT_MS
                }) as uint64_t,
            );
        }
        found_new_match =
            get_next_completion_match(type_0, st.ptr(), &raw mut start_pos) as ::core::ffi::c_int;
        if (*compl_pattern.ptr()).data.is_null() {
            break;
        }
        if may_advance_cpt_idx {
            if advance_cpt_sources_index_safe() == 0 {
                break;
            }
            compl_source_start_timer(cpt_sources_index.get());
        }
        if ctrl_x_mode_not_default() as ::core::ffi::c_int != 0 && !ctrl_x_mode_line_or_eval()
            || found_new_match != FAIL
        {
            if got_int.get() {
                break;
            }
            if type_0 != -1 as ::core::ffi::c_int {
                ins_compl_check_keys(0 as ::core::ffi::c_int, false_0 != 0);
            }
            if ctrl_x_mode_not_default() as ::core::ffi::c_int != 0 && !ctrl_x_mode_line_or_eval()
                || compl_interrupted.get() as ::core::ffi::c_int != 0
            {
                break;
            }
            compl_started.set(!compl_time_slice_expired.get());
        } else {
            if buf_valid((*st.ptr()).ins_buf) as ::core::ffi::c_int != 0
                && (type_0 == 0 as ::core::ffi::c_int
                    || type_0 == CTRL_X_PATH_PATTERNS as ::core::ffi::c_int)
            {
                '_c2rust_label_1: {
                    if !(*st.ptr()).ins_buf.is_null() {
                    } else {
                        __assert_fail(
                            b"st.ins_buf\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            4812 as ::core::ffi::c_uint,
                            b"int ins_compl_get_exp(pos_T *)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                (*(*st.ptr()).ins_buf).b_scanned = true_0 != 0;
            }
            compl_started.set(false_0 != 0);
        }
        if normal_mode_strict as ::core::ffi::c_int != 0
            && type_0 == CTRL_X_FUNCTION as ::core::ffi::c_int
            && (compl_autocomplete.get() as ::core::ffi::c_int != 0 || p_cto.get() > 0 as OptInt)
        {
            compl_timeout_ms.set(compl_timeout_save);
        }
        if !compl_dir_forward() {
            while !(*compl_curr_match.get()).cp_prev.is_null()
                && !match_at_original_text((*compl_curr_match.get()).cp_prev)
            {
                compl_curr_match.set((*compl_curr_match.get()).cp_prev);
            }
        }
    }
    cpt_sources_index.set(-1 as ::core::ffi::c_int);
    compl_started.set(true_0 != 0);
    if (ctrl_x_mode_normal() as ::core::ffi::c_int != 0
        || ctrl_x_mode_line_or_eval() as ::core::ffi::c_int != 0)
        && *(*st.ptr()).e_cpt as ::core::ffi::c_int == NUL
    {
        found_new_match = FAIL;
    }
    let mut match_count: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if found_new_match == FAIL
        || ctrl_x_mode_not_default() as ::core::ffi::c_int != 0 && !ctrl_x_mode_line_or_eval()
    {
        match_count = ins_compl_make_cyclic();
    }
    if cot_fuzzy() as ::core::ffi::c_int != 0
        && compl_get_longest.get() as ::core::ffi::c_int != 0
        && compl_num_bests.get() > 0 as ::core::ffi::c_int
    {
        fuzzy_longest_match();
    }
    if !(*compl_old_match.ptr()).is_null() {
        compl_curr_match.set(if compl_dir_forward() as ::core::ffi::c_int != 0 {
            (*compl_old_match.get()).cp_next
        } else {
            (*compl_old_match.get()).cp_prev
        });
        if (*compl_curr_match.ptr()).is_null() {
            compl_curr_match.set(compl_old_match.get());
        }
    }
    may_trigger_modechanged();
    if match_count > 0 as ::core::ffi::c_int && !ctrl_x_mode_spell() {
        if is_nearest_active() as ::core::ffi::c_int != 0 && !ins_compl_has_preinsert() {
            sort_compl_match_list(Some(
                cp_compare_nearest
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ));
        }
        if cot_fuzzy() as ::core::ffi::c_int != 0 && ins_compl_leader_len() > 0 as size_t {
            ins_compl_fuzzy_sort();
        }
    }
    return match_count;
}
unsafe extern "C" fn ins_compl_update_shown_match() {
    get_leader_for_startcol(::core::ptr::null_mut::<compl_T>(), true_0 != 0);
    let mut leader: *mut String_0 = get_leader_for_startcol(compl_shown_match.get(), true_0 != 0);
    while !ins_compl_equal(compl_shown_match.get(), (*leader).data, (*leader).size)
        && !(*compl_shown_match.get()).cp_next.is_null()
        && !is_first_match((*compl_shown_match.get()).cp_next)
    {
        compl_shown_match.set((*compl_shown_match.get()).cp_next);
        leader = get_leader_for_startcol(compl_shown_match.get(), true_0 != 0);
    }
    if compl_shows_dir_backward() as ::core::ffi::c_int != 0
        && !ins_compl_equal(compl_shown_match.get(), (*leader).data, (*leader).size)
        && ((*compl_shown_match.get()).cp_next.is_null()
            || is_first_match((*compl_shown_match.get()).cp_next) as ::core::ffi::c_int != 0)
    {
        while !ins_compl_equal(compl_shown_match.get(), (*leader).data, (*leader).size)
            && !(*compl_shown_match.get()).cp_prev.is_null()
            && !is_first_match((*compl_shown_match.get()).cp_prev)
        {
            compl_shown_match.set((*compl_shown_match.get()).cp_prev);
            leader = get_leader_for_startcol(compl_shown_match.get(), true_0 != 0);
        }
    }
}
pub unsafe extern "C" fn ins_compl_delete(mut new_leader: bool) {
    let mut orig_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if new_leader {
        let mut orig: *mut ::core::ffi::c_char = (*compl_orig_text.ptr()).data;
        let mut leader: *mut ::core::ffi::c_char = ins_compl_leader();
        while *orig as ::core::ffi::c_int != NUL && utf_ptr2char(orig) == utf_ptr2char(leader) {
            leader = leader.offset(utf_ptr2len(leader) as isize);
            orig = orig.offset(utf_ptr2len(orig) as isize);
        }
        orig_col = orig.offset_from((*compl_orig_text.ptr()).data) as ::core::ffi::c_int;
    }
    let mut col: ::core::ffi::c_int = compl_col.get() as ::core::ffi::c_int
        + (if compl_status_adding() as ::core::ffi::c_int != 0 {
            compl_length.get()
        } else {
            orig_col
        });
    if ins_compl_preinsert_effect() {
        col += ins_compl_leader_len() as ::core::ffi::c_int;
        (*curwin.get()).w_cursor.col = compl_ins_end_col.get();
    }
    let mut remaining: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0 as size_t,
    };
    if (*curwin.get()).w_cursor.lnum > compl_lnum.get() {
        if (*curwin.get()).w_cursor.col < get_cursor_line_len() {
            remaining = cbuf_to_string(get_cursor_pos_ptr(), get_cursor_pos_len() as size_t);
        }
        while (*curwin.get()).w_cursor.lnum > compl_lnum.get() {
            if ml_delete((*curwin.get()).w_cursor.lnum) == FAIL {
                if !remaining.data.is_null() {
                    xfree(remaining.data as *mut ::core::ffi::c_void);
                }
                return;
            }
            deleted_lines_mark((*curwin.get()).w_cursor.lnum, 1 as ::core::ffi::c_int);
            (*curwin.get()).w_cursor.lnum -= 1;
        }
        (*curwin.get()).w_cursor.col = get_cursor_line_len();
    }
    if (*curwin.get()).w_cursor.col > col {
        if stop_arrow() == FAIL {
            if !remaining.data.is_null() {
                xfree(remaining.data as *mut ::core::ffi::c_void);
            }
            return;
        }
        backspace_until_column(col);
        compl_ins_end_col.set((*curwin.get()).w_cursor.col);
    }
    if !remaining.data.is_null() {
        orig_col = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
        ins_str(remaining.data, remaining.size);
        (*curwin.get()).w_cursor.col = orig_col as colnr_T;
        xfree(remaining.data as *mut ::core::ffi::c_void);
    }
    changed_cline_bef_curs(curwin.get());
    set_vim_var_dict(VV_COMPLETED_ITEM, tv_dict_alloc_lock(VAR_FIXED));
}
unsafe extern "C" fn ins_compl_expand_multiple(mut str: *mut ::core::ffi::c_char) {
    let mut start: *mut ::core::ffi::c_char = str;
    let mut curr: *mut ::core::ffi::c_char = str;
    let mut base_indent: ::core::ffi::c_int = get_indent();
    while *curr as ::core::ffi::c_int != NUL {
        if *curr as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
            if curr > start {
                ins_char_bytes(start, curr.offset_from(start) as size_t);
            }
            open_line(
                FORWARD as ::core::ffi::c_int,
                OPENLINE_KEEPTRAIL as ::core::ffi::c_int
                    | OPENLINE_FORCE_INDENT as ::core::ffi::c_int,
                base_indent,
                ::core::ptr::null_mut::<bool>(),
            );
            start = curr.offset(1 as ::core::ffi::c_int as isize);
        }
        curr = curr.offset(1);
    }
    if curr > start {
        ins_char_bytes(start, curr.offset_from(start) as size_t);
    }
    compl_ins_end_col.set((*curwin.get()).w_cursor.col);
}
unsafe extern "C" fn find_common_prefix(
    mut prefix_len: *mut size_t,
    mut curbuf_only: bool,
) -> *mut ::core::ffi::c_char {
    let mut is_cpt_completion: bool = !(*cpt_sources_array.ptr()).is_null();
    if !is_cpt_completion {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut match_count: *mut ::core::ffi::c_int = xcalloc(
        cpt_sources_count.get() as size_t,
        ::core::mem::size_of::<::core::ffi::c_int>(),
    ) as *mut ::core::ffi::c_int;
    get_leader_for_startcol(::core::ptr::null_mut::<compl_T>(), true_0 != 0);
    let mut compl: *mut compl_T = compl_first_match.get();
    let mut first: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    loop {
        let mut leader: *mut String_0 = get_leader_for_startcol(compl, true_0 != 0);
        if ctrl_x_mode_normal() as ::core::ffi::c_int != 0
            && p_inf.get() == 0
            && !(*leader).data.is_null()
            && ignorecase((*leader).data) == 0
        {
            (*compl).cp_flags &= !(CP_ICASE as ::core::ffi::c_int);
        }
        if !match_at_original_text(compl)
            && ((*leader).data.is_null()
                || ins_compl_equal(compl, (*leader).data, (*leader).size) as ::core::ffi::c_int
                    != 0)
        {
            let mut match_limit_exceeded: bool = false_0 != 0;
            let mut cur_source: ::core::ffi::c_int = (*compl).cp_cpt_source_idx;
            if cur_source != -1 as ::core::ffi::c_int {
                *match_count.offset(cur_source as isize) += 1;
                let mut max_matches: ::core::ffi::c_int =
                    (*(*cpt_sources_array.ptr()).offset(cur_source as isize)).cs_max_matches;
                if max_matches > 0 as ::core::ffi::c_int
                    && *match_count.offset(cur_source as isize) > max_matches
                {
                    match_limit_exceeded = true_0 != 0;
                }
            }
            if !match_limit_exceeded
                && (!curbuf_only
                    || cur_source != -1 as ::core::ffi::c_int
                        && (*(*cpt_sources_array.ptr()).offset(cur_source as isize)).cs_flag
                            as ::core::ffi::c_int
                            == '.' as ::core::ffi::c_int)
            {
                if first.is_null()
                    && strncmp(
                        ins_compl_leader(),
                        (*compl).cp_str.data,
                        ins_compl_leader_len(),
                    ) == 0 as ::core::ffi::c_int
                {
                    first = (*compl).cp_str.data;
                    len = strlen(first) as ::core::ffi::c_int;
                } else if !first.is_null() {
                    let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut s1: *mut ::core::ffi::c_char = first;
                    let mut s2: *mut ::core::ffi::c_char = (*compl).cp_str.data;
                    while j < len
                        && *s1 as ::core::ffi::c_int != NUL
                        && *s2 as ::core::ffi::c_int != NUL
                    {
                        if (*utf8len_tab.ptr())[*s1 as uint8_t as usize] as ::core::ffi::c_int
                            != (*utf8len_tab.ptr())[*s2 as uint8_t as usize] as ::core::ffi::c_int
                            || memcmp(
                                s1 as *const ::core::ffi::c_void,
                                s2 as *const ::core::ffi::c_void,
                                (*utf8len_tab.ptr())[*s1 as uint8_t as usize] as size_t,
                            ) != 0 as ::core::ffi::c_int
                        {
                            break;
                        }
                        j += (*utf8len_tab.ptr())[*s1 as uint8_t as usize] as ::core::ffi::c_int;
                        s1 = s1.offset(utfc_ptr2len(s1) as isize);
                        s2 = s2.offset(utfc_ptr2len(s2) as isize);
                    }
                    len = j;
                    if len == 0 as ::core::ffi::c_int {
                        break;
                    }
                }
            }
        }
        compl = (*compl).cp_next;
        if !(!compl.is_null() && !is_first_match(compl)) {
            break;
        }
    }
    xfree(match_count as *mut ::core::ffi::c_void);
    if len > ins_compl_leader_len() as ::core::ffi::c_int {
        '_c2rust_label: {
            if !first.is_null() {
            } else {
                __assert_fail(
                    b"first != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    5085 as ::core::ffi::c_uint,
                    b"char *find_common_prefix(size_t *, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if len == strlen(first) as ::core::ffi::c_int {
            let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
            let mut p: *mut ::core::ffi::c_char =
                line.offset((*curwin.get()).w_cursor.col as isize);
            if !p.is_null() && !ascii_iswhite_or_nul(*p as ::core::ffi::c_int) {
                let mut end: *mut ::core::ffi::c_char = find_word_end(p);
                let mut text_len: ::core::ffi::c_int = end.offset_from(p) as ::core::ffi::c_int;
                if text_len > 0 as ::core::ffi::c_int
                    && text_len < len - ins_compl_leader_len() as ::core::ffi::c_int
                    && strncmp(
                        first.offset(len as isize).offset(-(text_len as isize)),
                        p,
                        text_len as size_t,
                    ) == 0 as ::core::ffi::c_int
                {
                    len -= text_len;
                }
            }
        }
        *prefix_len = len as size_t;
        return first;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn ins_compl_insert(mut move_cursor: bool, mut insert_prefix: bool) {
    let mut compl_len: ::core::ffi::c_int = get_compl_len();
    let mut preinsert: bool = ins_compl_has_preinsert();
    let mut cp_str: *mut ::core::ffi::c_char = (*compl_shown_match.get()).cp_str.data;
    let mut cp_str_len: size_t = (*compl_shown_match.get()).cp_str.size;
    let mut leader_len: size_t = ins_compl_leader_len();
    let mut has_multiple: *mut ::core::ffi::c_char = strchr(cp_str, '\n' as ::core::ffi::c_int);
    if insert_prefix {
        cp_str = find_common_prefix(&raw mut cp_str_len, false_0 != 0);
        if cp_str.is_null() {
            cp_str = find_common_prefix(&raw mut cp_str_len, true_0 != 0);
            if cp_str.is_null() {
                cp_str = (*compl_shown_match.get()).cp_str.data;
                cp_str_len = (*compl_shown_match.get()).cp_str.size;
            }
        }
    } else if !(*cpt_sources_array.ptr()).is_null() {
        let mut cpt_idx: ::core::ffi::c_int = (*compl_shown_match.get()).cp_cpt_source_idx;
        if cpt_idx >= 0 as ::core::ffi::c_int && compl_col.get() >= 0 as ::core::ffi::c_int {
            let mut startcol: ::core::ffi::c_int =
                (*(*cpt_sources_array.ptr()).offset(cpt_idx as isize)).cs_startcol;
            if startcol >= 0 as ::core::ffi::c_int && startcol < compl_col.get() {
                let mut skip: ::core::ffi::c_int = compl_col.get() - startcol;
                if skip as size_t <= cp_str_len {
                    cp_str_len = cp_str_len.wrapping_sub(skip as size_t);
                    cp_str = cp_str.offset(skip as isize);
                }
            }
        }
    }
    if compl_len < cp_str_len as ::core::ffi::c_int {
        if !has_multiple.is_null() {
            ins_compl_expand_multiple(cp_str.offset(compl_len as isize));
        } else {
            ins_compl_insert_bytes(
                cp_str.offset(compl_len as isize),
                if insert_prefix as ::core::ffi::c_int != 0 {
                    cp_str_len as ::core::ffi::c_int - compl_len
                } else {
                    -1 as ::core::ffi::c_int
                },
            );
            if (preinsert as ::core::ffi::c_int != 0 || insert_prefix as ::core::ffi::c_int != 0)
                && move_cursor as ::core::ffi::c_int != 0
            {
                (*curwin.get()).w_cursor.col -= cp_str_len.wrapping_sub(leader_len) as colnr_T;
            }
        }
    }
    compl_used_match.set(
        !(match_at_original_text(compl_shown_match.get()) as ::core::ffi::c_int != 0
            || preinsert as ::core::ffi::c_int != 0 && !insert_prefix),
    );
    let mut dict: *mut dict_T = ins_compl_dict_alloc(compl_shown_match.get());
    set_vim_var_dict(VV_COMPLETED_ITEM, dict);
    compl_hi_on_autocompl_longest
        .set(insert_prefix as ::core::ffi::c_int != 0 && move_cursor as ::core::ffi::c_int != 0);
}
unsafe extern "C" fn ins_compl_show_filename() {
    let lead: *mut ::core::ffi::c_char =
        gettext(b"match in file\0".as_ptr() as *const ::core::ffi::c_char);
    let mut space: ::core::ffi::c_int = sc_col.get() - vim_strsize(lead) - 2 as ::core::ffi::c_int;
    if space <= 0 as ::core::ffi::c_int {
        return;
    }
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut e: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    e = (*compl_shown_match.get()).cp_fname;
    s = e;
    while *e as ::core::ffi::c_int != NUL {
        space -= ptr2cells(e);
        while space < 0 as ::core::ffi::c_int {
            space += ptr2cells(s);
            s = s.offset(utfc_ptr2len(s) as isize);
        }
        e = e.offset(utfc_ptr2len(e) as isize);
    }
    if !compl_autocomplete.get() {
        msg_hist_off.set(true_0 != 0);
        vim_snprintf(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            b"%s %s%s\0".as_ptr() as *const ::core::ffi::c_char,
            lead,
            if s > (*compl_shown_match.get()).cp_fname {
                b"<\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
            s,
        );
        msg(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
        msg_hist_off.set(false_0 != 0);
        redraw_cmdline.set(false_0 != 0);
    }
}
unsafe extern "C" fn find_next_match_in_menu() -> *mut compl_T {
    let mut is_forward: bool = compl_shows_dir_forward();
    let mut match_0: *mut compl_T = compl_shown_match.get();
    loop {
        match_0 = if is_forward as ::core::ffi::c_int != 0 {
            (*match_0).cp_next
        } else {
            (*match_0).cp_prev
        };
        if !(!(*match_0).cp_next.is_null()
            && !(*match_0).cp_in_match_array
            && !match_at_original_text(match_0))
        {
            break;
        }
    }
    return match_0;
}
unsafe extern "C" fn find_next_completion_match(
    mut allow_get_expansion: bool,
    mut todo: ::core::ffi::c_int,
    mut advance: bool,
    mut num_matches: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut found_end: bool = false_0 != 0;
    let mut found_compl: *mut compl_T = ::core::ptr::null_mut::<compl_T>();
    let mut cur_cot_flags: ::core::ffi::c_uint = get_cot_flags();
    let mut compl_no_select: bool = cur_cot_flags
        & kOptCotFlagNoselect as ::core::ffi::c_int as ::core::ffi::c_uint
        != 0 as ::core::ffi::c_uint
        || compl_autocomplete.get() as ::core::ffi::c_int != 0 && !ins_compl_has_preinsert();
    loop {
        todo -= 1;
        if todo < 0 as ::core::ffi::c_int {
            break;
        }
        if compl_shows_dir_forward() as ::core::ffi::c_int != 0
            && !(*compl_shown_match.get()).cp_next.is_null()
        {
            if !(*compl_match_array.ptr()).is_null() {
                compl_shown_match.set(find_next_match_in_menu());
            } else {
                compl_shown_match.set((*compl_shown_match.get()).cp_next);
            }
            found_end = !(*compl_first_match.ptr()).is_null()
                && (is_first_match((*compl_shown_match.get()).cp_next) as ::core::ffi::c_int != 0
                    || is_first_match(compl_shown_match.get()) as ::core::ffi::c_int != 0);
        } else if compl_shows_dir_backward() as ::core::ffi::c_int != 0
            && !(*compl_shown_match.get()).cp_prev.is_null()
        {
            found_end = is_first_match(compl_shown_match.get());
            if !(*compl_match_array.ptr()).is_null() {
                compl_shown_match.set(find_next_match_in_menu());
            } else {
                compl_shown_match.set((*compl_shown_match.get()).cp_prev);
            }
            found_end = found_end as ::core::ffi::c_int
                | is_first_match(compl_shown_match.get()) as ::core::ffi::c_int
                != 0;
        } else {
            if !allow_get_expansion {
                if advance {
                    if compl_shows_dir_backward() {
                        (*compl_pending.ptr()) -= todo + 1 as ::core::ffi::c_int;
                    } else {
                        (*compl_pending.ptr()) += todo + 1 as ::core::ffi::c_int;
                    }
                }
                return -1 as ::core::ffi::c_int;
            }
            if !compl_no_select && advance as ::core::ffi::c_int != 0 {
                if compl_shows_dir_backward() {
                    (*compl_pending.ptr()) -= 1;
                } else {
                    (*compl_pending.ptr()) += 1;
                }
            }
            *num_matches = ins_compl_get_exp(compl_startpos.ptr());
            while compl_pending.get() != 0 as ::core::ffi::c_int
                && compl_direction.get() as ::core::ffi::c_int
                    == compl_shows_dir.get() as ::core::ffi::c_int
                && advance as ::core::ffi::c_int != 0
            {
                if compl_pending.get() > 0 as ::core::ffi::c_int
                    && !(*compl_shown_match.get()).cp_next.is_null()
                {
                    compl_shown_match.set((*compl_shown_match.get()).cp_next);
                    (*compl_pending.ptr()) -= 1;
                } else {
                    if !(compl_pending.get() < 0 as ::core::ffi::c_int
                        && !(*compl_shown_match.get()).cp_prev.is_null())
                    {
                        break;
                    }
                    compl_shown_match.set((*compl_shown_match.get()).cp_prev);
                    (*compl_pending.ptr()) += 1;
                }
            }
            found_end = false_0 != 0;
        }
        let mut leader: *mut String_0 =
            get_leader_for_startcol(compl_shown_match.get(), false_0 != 0);
        if !match_at_original_text(compl_shown_match.get())
            && !(*leader).data.is_null()
            && !ins_compl_equal(compl_shown_match.get(), (*leader).data, (*leader).size)
            && !(cot_fuzzy() as ::core::ffi::c_int != 0
                && (*compl_shown_match.get()).cp_score != FUZZY_SCORE_NONE as ::core::ffi::c_int)
        {
            todo += 1;
        } else {
            found_compl = compl_shown_match.get();
        }
        if !found_end {
            continue;
        }
        if !found_compl.is_null() {
            compl_shown_match.set(found_compl);
            break;
        } else {
            todo = 1 as ::core::ffi::c_int;
        }
    }
    return OK;
}
unsafe extern "C" fn ins_compl_next(
    mut allow_get_expansion: bool,
    mut count: ::core::ffi::c_int,
    mut insert_match: bool,
) -> ::core::ffi::c_int {
    let mut num_matches: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut todo: ::core::ffi::c_int = count;
    let started: bool = compl_started.get();
    let orig_curbuf: *mut buf_T = curbuf.get();
    let mut cur_cot_flags: ::core::ffi::c_uint = get_cot_flags();
    let mut compl_no_insert: bool = cur_cot_flags
        & kOptCotFlagNoinsert as ::core::ffi::c_int as ::core::ffi::c_uint
        != 0 as ::core::ffi::c_uint
        || compl_autocomplete.get() as ::core::ffi::c_int != 0 && !ins_compl_has_preinsert();
    let mut compl_preinsert: bool = ins_compl_has_preinsert();
    let mut has_autocomplete_delay: bool =
        compl_autocomplete.get() as ::core::ffi::c_int != 0 && p_acl.get() > 0 as OptInt;
    if (*compl_shown_match.ptr()).is_null() {
        return -1 as ::core::ffi::c_int;
    }
    if !(*compl_leader.ptr()).data.is_null()
        && !match_at_original_text(compl_shown_match.get())
        && !cot_fuzzy()
    {
        ins_compl_update_shown_match();
    }
    if allow_get_expansion as ::core::ffi::c_int != 0
        && insert_match as ::core::ffi::c_int != 0
        && (!compl_get_longest.get() || compl_used_match.get() as ::core::ffi::c_int != 0)
    {
        ins_compl_delete(false_0 != 0);
    }
    let mut advance: bool =
        count != 1 as ::core::ffi::c_int || !allow_get_expansion || !compl_get_longest.get();
    if compl_restarting.get() {
        advance = false_0 != 0;
        compl_restarting.set(false_0 != 0);
    }
    if find_next_completion_match(allow_get_expansion, todo, advance, &raw mut num_matches)
        == -1 as ::core::ffi::c_int
    {
        return -1 as ::core::ffi::c_int;
    }
    if curbuf.get() != orig_curbuf {
        return -1 as ::core::ffi::c_int;
    }
    if !started && ins_compl_preinsert_longest() as ::core::ffi::c_int != 0 {
        ins_compl_insert(true_0 != 0, true_0 != 0);
        if has_autocomplete_delay {
            update_screen();
        }
    } else if compl_no_insert as ::core::ffi::c_int != 0 && !started && !compl_preinsert {
        ins_compl_insert_bytes(
            (*compl_orig_text.ptr())
                .data
                .offset(get_compl_len() as isize),
            -1 as ::core::ffi::c_int,
        );
        compl_used_match.set(false_0 != 0);
        restore_orig_extmarks();
    } else if insert_match {
        if !compl_get_longest.get() || compl_used_match.get() as ::core::ffi::c_int != 0 {
            let mut preinsert_longest: bool = ins_compl_preinsert_longest() as ::core::ffi::c_int
                != 0
                && match_at_original_text(compl_shown_match.get()) as ::core::ffi::c_int != 0;
            ins_compl_insert(
                compl_preinsert as ::core::ffi::c_int != 0
                    || preinsert_longest as ::core::ffi::c_int != 0,
                preinsert_longest,
            );
        } else {
            '_c2rust_label: {
                if !(*compl_leader.ptr()).data.is_null() {
                } else {
                    __assert_fail(
                        b"compl_leader.data != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        5406 as ::core::ffi::c_uint,
                        b"int ins_compl_next(_Bool, int, _Bool)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            ins_compl_insert_bytes(
                (*compl_leader.ptr()).data.offset(get_compl_len() as isize),
                -1 as ::core::ffi::c_int,
            );
        }
        if strequal(
            (*compl_shown_match.get()).cp_str.data,
            (*compl_orig_text.ptr()).data,
        ) {
            restore_orig_extmarks();
        }
    } else {
        compl_used_match.set(false_0 != 0);
    }
    if !allow_get_expansion {
        update_screen();
        if !has_autocomplete_delay {
            ins_compl_show_pum();
        }
        ins_compl_delete(false_0 != 0);
    }
    if compl_no_insert as ::core::ffi::c_int != 0
        && !started
        && !match_at_original_text(compl_shown_match.get())
    {
        compl_enter_selects.set(true_0 != 0);
    } else {
        compl_enter_selects.set(!insert_match && !(*compl_match_array.ptr()).is_null());
    }
    if !(*compl_shown_match.get()).cp_fname.is_null() {
        ins_compl_show_filename();
    }
    return num_matches;
}
unsafe extern "C" fn check_elapsed_time() {
    let mut start_tv: uint64_t =
        (*(*cpt_sources_array.ptr()).offset(cpt_sources_index.get() as isize)).compl_start_tv;
    let mut elapsed_ms: uint64_t = os_hrtime()
        .wrapping_sub(start_tv)
        .wrapping_div(1000000 as uint64_t);
    if elapsed_ms > compl_timeout_ms.get() {
        compl_time_slice_expired.set(true_0 != 0);
        if compl_timeout_ms.get() > COMPL_MIN_TIMEOUT_MS as uint64_t {
            compl_timeout_ms.set((*compl_timeout_ms.ptr()).wrapping_div(2 as uint64_t));
        }
    }
}
pub unsafe extern "C" fn ins_compl_check_keys(
    mut frequency: ::core::ffi::c_int,
    mut in_compl_func: bool,
) {
    static count: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    if !in_compl_func && (using_script() != 0 || ex_normal_busy.get() != 0) {
        return;
    }
    (*count.ptr()) += 1;
    if count.get() < frequency {
        return;
    }
    count.set(0 as ::core::ffi::c_int);
    let mut c: ::core::ffi::c_int = vpeekc_any();
    if c != NUL && !test_disable_char_avail.get() {
        if vim_is_ctrl_x_key(c) as ::core::ffi::c_int != 0 && c != Ctrl_X && c != Ctrl_R {
            c = safe_vgetc();
            compl_shows_dir.set(ins_compl_key2dir(c) as Direction);
            ins_compl_next(
                false_0 != 0,
                ins_compl_key2count(c),
                c != K_UP && c != K_DOWN,
            );
        } else {
            c = safe_vgetc();
            if c != -(253 as ::core::ffi::c_int
                + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                if c != Ctrl_R && KeyTyped.get() as ::core::ffi::c_int != 0 {
                    compl_interrupted.set(true_0 != 0);
                }
                vungetc(c);
            }
        }
    } else {
        let mut normal_mode_strict: bool = ctrl_x_mode_normal() as ::core::ffi::c_int != 0
            && !ctrl_x_mode_line_or_eval()
            && compl_cont_status.get() & CONT_LOCAL == 0
            && !(*cpt_sources_array.ptr()).is_null()
            && cpt_sources_index.get() >= 0 as ::core::ffi::c_int;
        if normal_mode_strict as ::core::ffi::c_int != 0
            && (compl_autocomplete.get() as ::core::ffi::c_int != 0 || p_cto.get() > 0 as OptInt)
        {
            check_elapsed_time();
        }
    }
    if compl_pending.get() != 0
        && !got_int.get()
        && cot_flags.get()
            & (kOptCotFlagNoinsert as ::core::ffi::c_int | kOptCotFlagFuzzy as ::core::ffi::c_int)
                as ::core::ffi::c_uint
            == 0
        && (!compl_autocomplete.get() || ins_compl_has_preinsert() as ::core::ffi::c_int != 0)
    {
        let mut todo: ::core::ffi::c_int = if compl_pending.get() > 0 as ::core::ffi::c_int {
            compl_pending.get()
        } else {
            -compl_pending.get()
        };
        compl_pending.set(0 as ::core::ffi::c_int);
        ins_compl_next(false_0 != 0, todo, true_0 != 0);
    }
}
unsafe extern "C" fn ins_compl_key2dir(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if c == -(253 as ::core::ffi::c_int
        + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_COMMAND as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_LUA as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        return if (*pum_want.ptr()).item < compl_selected_item.get() {
            BACKWARD as ::core::ffi::c_int
        } else {
            FORWARD as ::core::ffi::c_int
        };
    }
    if c == Ctrl_P
        || c == Ctrl_L
        || c == K_PAGEUP
        || c == K_KPAGEUP
        || c == -(253 as ::core::ffi::c_int
            + ((KE_S_UP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == K_UP
    {
        return BACKWARD as ::core::ffi::c_int;
    }
    return FORWARD as ::core::ffi::c_int;
}
unsafe extern "C" fn ins_compl_pum_key(mut c: ::core::ffi::c_int) -> bool {
    return pum_visible() as ::core::ffi::c_int != 0
        && (c == K_PAGEUP
            || c == K_KPAGEUP
            || c == -(253 as ::core::ffi::c_int
                + ((KE_S_UP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c == K_PAGEDOWN
            || c == K_KPAGEDOWN
            || c == -(253 as ::core::ffi::c_int
                + ((KE_S_DOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c == K_UP
            || c == K_DOWN);
}
unsafe extern "C" fn ins_compl_key2count(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if c == -(253 as ::core::ffi::c_int
        + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_COMMAND as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_LUA as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        let mut offset: ::core::ffi::c_int = (*pum_want.ptr()).item - compl_selected_item.get();
        return abs(offset);
    }
    if ins_compl_pum_key(c) as ::core::ffi::c_int != 0 && c != K_UP && c != K_DOWN {
        let mut h: ::core::ffi::c_int = pum_get_height();
        if h > 3 as ::core::ffi::c_int {
            h -= 2 as ::core::ffi::c_int;
        }
        return h;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn ins_compl_use_match(mut c: ::core::ffi::c_int) -> bool {
    match c {
        K_UP | K_DOWN | K_PAGEDOWN | K_KPAGEDOWN | -1533 | K_PAGEUP | K_KPAGEUP | -1277 => {
            return false_0 != 0;
        }
        -26365 | -26877 | -26621 => {
            return (*pum_want.ptr()).active as ::core::ffi::c_int != 0
                && (*pum_want.ptr()).insert as ::core::ffi::c_int != 0;
        }
        _ => {}
    }
    return true_0 != 0;
}
unsafe extern "C" fn get_normal_compl_info(
    mut line: *mut ::core::ffi::c_char,
    mut startcol: ::core::ffi::c_int,
    mut curs_col: colnr_T,
) -> ::core::ffi::c_int {
    if compl_cont_status.get() & CONT_SOL != 0
        || ctrl_x_mode_path_defines() as ::core::ffi::c_int != 0
    {
        if !compl_status_adding() {
            loop {
                startcol -= 1;
                if !(startcol >= 0 as ::core::ffi::c_int
                    && vim_isIDc(*line.offset(startcol as isize) as uint8_t as ::core::ffi::c_int)
                        as ::core::ffi::c_int
                        != 0)
                {
                    break;
                }
            }
            startcol += 1;
            (*compl_col.ptr()) += startcol;
            compl_length.set(curs_col as ::core::ffi::c_int - startcol);
        }
        if p_ic.get() != 0 {
            compl_pattern.set(cstr_as_string(str_foldcase(
                line.offset(compl_col.get() as isize),
                compl_length.get(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                0 as ::core::ffi::c_int,
            )));
        } else {
            compl_pattern.set(cbuf_to_string(
                line.offset(compl_col.get() as isize),
                compl_length.get() as size_t,
            ));
        }
    } else if compl_status_adding() {
        let mut prefix: *mut ::core::ffi::c_char =
            b"\\<\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        let mut prefixlen: size_t =
            ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t);
        if !vim_iswordp(line.offset(compl_col.get() as isize))
            || compl_col.get() > 0 as ::core::ffi::c_int
                && vim_iswordp(mb_prevptr(line, line.offset(compl_col.get() as isize)))
                    as ::core::ffi::c_int
                    != 0
        {
            prefix = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            prefixlen = 0 as size_t;
        }
        let mut n: size_t = (quote_meta(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            line.offset(compl_col.get() as isize),
            compl_length.get(),
        ) as size_t)
            .wrapping_add(prefixlen);
        (*compl_pattern.ptr()).data = xmalloc(n) as *mut ::core::ffi::c_char;
        strcpy((*compl_pattern.ptr()).data, prefix);
        quote_meta(
            (*compl_pattern.ptr()).data.offset(prefixlen as isize),
            line.offset(compl_col.get() as isize),
            compl_length.get(),
        );
        (*compl_pattern.ptr()).size = n.wrapping_sub(1 as size_t);
    } else {
        startcol -= 1;
        if startcol < 0 as ::core::ffi::c_int
            || !vim_iswordp(mb_prevptr(
                line,
                line.offset(startcol as isize)
                    .offset(1 as ::core::ffi::c_int as isize),
            ))
        {
            compl_pattern.set(cbuf_to_string(
                b"\\<\\k\\k\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
            ));
            (*compl_col.ptr()) += curs_col;
            compl_length.set(0 as ::core::ffi::c_int);
            compl_from_nonkeyword.set(true_0 != 0);
        } else {
            startcol -= utf_head_off(line, line.offset(startcol as isize));
            let mut base_class: ::core::ffi::c_int = mb_get_class(line.offset(startcol as isize));
            loop {
                startcol -= 1;
                if startcol < 0 as ::core::ffi::c_int {
                    break;
                }
                let mut head_off: ::core::ffi::c_int =
                    utf_head_off(line, line.offset(startcol as isize));
                if base_class
                    != mb_get_class(line.offset(startcol as isize).offset(-(head_off as isize)))
                {
                    break;
                }
                startcol -= head_off;
            }
            startcol += 1;
            (*compl_col.ptr()) += startcol;
            compl_length.set(curs_col - startcol);
            if compl_length.get() == 1 as ::core::ffi::c_int {
                (*compl_pattern.ptr()).data = xmalloc(7 as size_t) as *mut ::core::ffi::c_char;
                strcpy(
                    (*compl_pattern.ptr()).data,
                    b"\\<\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
                quote_meta(
                    (*compl_pattern.ptr())
                        .data
                        .offset(2 as ::core::ffi::c_int as isize),
                    line.offset(compl_col.get() as isize),
                    1 as ::core::ffi::c_int,
                );
                strcat(
                    (*compl_pattern.ptr()).data,
                    b"\\k\0".as_ptr() as *const ::core::ffi::c_char,
                );
                (*compl_pattern.ptr()).size = strlen((*compl_pattern.ptr()).data);
            } else {
                let mut n_0: size_t = quote_meta(
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    line.offset(compl_col.get() as isize),
                    compl_length.get(),
                )
                .wrapping_add(2 as ::core::ffi::c_uint)
                    as size_t;
                (*compl_pattern.ptr()).data = xmalloc(n_0) as *mut ::core::ffi::c_char;
                strcpy(
                    (*compl_pattern.ptr()).data,
                    b"\\<\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
                quote_meta(
                    (*compl_pattern.ptr())
                        .data
                        .offset(2 as ::core::ffi::c_int as isize),
                    line.offset(compl_col.get() as isize),
                    compl_length.get(),
                );
                (*compl_pattern.ptr()).size = n_0.wrapping_sub(1 as size_t);
            }
        }
    }
    if ctrl_x_mode_normal() as ::core::ffi::c_int != 0 && compl_cont_status.get() & CONT_LOCAL == 0
    {
        setup_cpt_sources();
        prepare_cpt_compl_funcs();
    }
    return OK;
}
unsafe extern "C" fn get_wholeline_compl_info(
    mut line: *mut ::core::ffi::c_char,
    mut curs_col: colnr_T,
) -> ::core::ffi::c_int {
    compl_col.set(getwhitecols(line) as colnr_T);
    compl_length.set(curs_col - compl_col.get());
    if compl_length.get() < 0 as ::core::ffi::c_int {
        compl_length.set(0 as ::core::ffi::c_int);
    }
    if p_ic.get() != 0 {
        compl_pattern.set(cstr_as_string(str_foldcase(
            line.offset(compl_col.get() as isize),
            compl_length.get(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            0 as ::core::ffi::c_int,
        )));
    } else {
        compl_pattern.set(cbuf_to_string(
            line.offset(compl_col.get() as isize),
            compl_length.get() as size_t,
        ));
    }
    return OK;
}
unsafe extern "C" fn get_filename_compl_info(
    mut line: *mut ::core::ffi::c_char,
    mut startcol: ::core::ffi::c_int,
    mut curs_col: colnr_T,
) -> ::core::ffi::c_int {
    if startcol > 0 as ::core::ffi::c_int {
        let mut p: *mut ::core::ffi::c_char = line.offset(startcol as isize);
        p = p.offset(
            -((utf_head_off(line, p.offset(-(1 as ::core::ffi::c_int as isize)))
                + 1 as ::core::ffi::c_int) as isize),
        );
        while p > line && vim_isfilec(utf_ptr2char(p)) as ::core::ffi::c_int != 0 {
            p = p.offset(
                -((utf_head_off(line, p.offset(-(1 as ::core::ffi::c_int as isize)))
                    + 1 as ::core::ffi::c_int) as isize),
            );
        }
        let mut p_is_filec: bool = false_0 != 0;
        p_is_filec = p_is_filec as ::core::ffi::c_int != 0
            || vim_isfilec(utf_ptr2char(p)) as ::core::ffi::c_int != 0;
        if p == line && p_is_filec as ::core::ffi::c_int != 0 {
            startcol = 0 as ::core::ffi::c_int;
        } else {
            startcol = p.offset_from(line) as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
        }
    }
    (*compl_col.ptr()) += startcol;
    compl_length.set(curs_col - startcol);
    compl_pattern.set(cstr_as_string(addstar(
        line.offset(compl_col.get() as isize),
        compl_length.get() as size_t,
        EXPAND_FILES as ::core::ffi::c_int,
    )));
    return OK;
}
unsafe extern "C" fn get_cmdline_compl_info(
    mut line: *mut ::core::ffi::c_char,
    mut curs_col: colnr_T,
) -> ::core::ffi::c_int {
    compl_pattern.set(cbuf_to_string(line, curs_col as size_t));
    set_cmd_context(
        compl_xp.ptr(),
        (*compl_pattern.ptr()).data,
        (*compl_pattern.ptr()).size as ::core::ffi::c_int,
        curs_col as ::core::ffi::c_int,
        false_0,
    );
    if (*compl_xp.ptr()).xp_context == EXPAND_LUA as ::core::ffi::c_int {
        nlua_expand_pat(compl_xp.ptr());
    }
    if (*compl_xp.ptr()).xp_context == EXPAND_UNSUCCESSFUL as ::core::ffi::c_int
        || (*compl_xp.ptr()).xp_context == EXPAND_NOTHING as ::core::ffi::c_int
    {
        compl_col.set(curs_col);
    } else {
        compl_col.set(
            (*compl_xp.ptr())
                .xp_pattern
                .offset_from((*compl_pattern.ptr()).data) as ::core::ffi::c_int
                as colnr_T,
        );
    }
    compl_length.set((curs_col - compl_col.get()) as ::core::ffi::c_int);
    return OK;
}
unsafe extern "C" fn set_compl_globals(
    mut startcol: ::core::ffi::c_int,
    mut curs_col: colnr_T,
    mut is_cpt_compl: bool,
) {
    if is_cpt_compl {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*cpt_compl_pattern.ptr()).data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        (*cpt_compl_pattern.ptr()).size = 0 as size_t;
        if startcol < compl_col.get() {
            prepend_startcol_text(cpt_compl_pattern.ptr(), compl_orig_text.ptr(), startcol);
            return;
        } else {
            cpt_compl_pattern.set(copy_string(
                compl_orig_text.get(),
                ::core::ptr::null_mut::<Arena>(),
            ));
        }
    } else {
        if startcol < 0 as ::core::ffi::c_int || startcol > curs_col {
            startcol = curs_col as ::core::ffi::c_int;
        }
        let mut line: *mut ::core::ffi::c_char = ml_get((*curwin.get()).w_cursor.lnum);
        let mut len: ::core::ffi::c_int = curs_col as ::core::ffi::c_int - startcol;
        compl_pattern.set(cbuf_to_string(
            line.offset(startcol as isize),
            len as size_t,
        ));
        compl_col.set(startcol as colnr_T);
        compl_length.set(len);
    };
}
unsafe extern "C" fn get_userdefined_compl_info(
    mut curs_col: colnr_T,
    mut cb: *mut Callback,
    mut startcol: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let save_State: ::core::ffi::c_int = State.get();
    let is_cpt_function: bool = !cb.is_null();
    if !is_cpt_function {
        let mut funcname: *mut ::core::ffi::c_char = get_complete_funcname(ctrl_x_mode.get());
        if *funcname as ::core::ffi::c_int == NUL {
            semsg(
                gettext(&raw const e_notset as *const ::core::ffi::c_char),
                if ctrl_x_mode_function() as ::core::ffi::c_int != 0 {
                    b"completefunc\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"omnifunc\0".as_ptr() as *const ::core::ffi::c_char
                },
            );
            return FAIL;
        }
        cb = get_insert_callback(ctrl_x_mode.get());
    }
    let mut args: [typval_T; 3] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 3];
    args[0 as ::core::ffi::c_int as usize].v_type = VAR_NUMBER;
    args[1 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
    args[2 as ::core::ffi::c_int as usize].v_type = VAR_UNKNOWN;
    args[0 as ::core::ffi::c_int as usize].vval.v_number = 1 as varnumber_T;
    args[1 as ::core::ffi::c_int as usize].vval.v_string =
        b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    let mut pos: pos_T = (*curwin.get()).w_cursor;
    (*textlock.ptr()) += 1;
    let mut col: colnr_T =
        callback_call_retnr(cb, 2 as ::core::ffi::c_int, &raw mut args as *mut typval_T) as colnr_T;
    (*textlock.ptr()) -= 1;
    State.set(save_State);
    (*curwin.get()).w_cursor = pos;
    check_cursor(curwin.get());
    validate_cursor(curwin.get());
    if !equalpos((*curwin.get()).w_cursor, pos) {
        emsg(gettext(
            (e_compldel.ptr() as *const _) as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    if !startcol.is_null() {
        *startcol = col as ::core::ffi::c_int;
    }
    if col == -2 as ::core::ffi::c_int || aborting() as ::core::ffi::c_int != 0 {
        return FAIL;
    }
    if col == -3 as ::core::ffi::c_int {
        if is_cpt_function {
            return FAIL;
        }
        ctrl_x_mode.set(CTRL_X_NORMAL as ::core::ffi::c_int);
        edit_submode.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        if !shortmess(SHM_COMPLETIONMENU as ::core::ffi::c_int) {
            msg_clr_cmdline();
        }
        return FAIL;
    }
    compl_opt_refresh_always.set(false_0 != 0);
    if !is_cpt_function {
        set_compl_globals(col as ::core::ffi::c_int, curs_col, false_0 != 0);
    }
    return OK;
}
unsafe extern "C" fn get_spell_compl_info(
    mut startcol: ::core::ffi::c_int,
    mut curs_col: colnr_T,
) -> ::core::ffi::c_int {
    if spell_bad_len.get() > 0 as size_t {
        '_c2rust_label: {
            if spell_bad_len.get() <= 2147483647 as ::core::ffi::c_int as size_t {
            } else {
                __assert_fail(
                    b"spell_bad_len <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    5875 as ::core::ffi::c_uint,
                    b"int get_spell_compl_info(int, colnr_T)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        compl_col.set(
            (curs_col as ::core::ffi::c_int - spell_bad_len.get() as ::core::ffi::c_int) as colnr_T,
        );
    } else {
        compl_col.set(spell_word_start(startcol) as colnr_T);
    }
    if compl_col.get() >= startcol {
        compl_length.set(0 as ::core::ffi::c_int);
        compl_col.set(curs_col);
    } else {
        spell_expand_check_cap(compl_col.get());
        compl_length.set((curs_col - compl_col.get()) as ::core::ffi::c_int);
    }
    let mut line: *mut ::core::ffi::c_char = ml_get((*curwin.get()).w_cursor.lnum);
    compl_pattern.set(cbuf_to_string(
        line.offset(compl_col.get() as isize),
        compl_length.get() as size_t,
    ));
    return OK;
}
unsafe extern "C" fn compl_get_info(
    mut line: *mut ::core::ffi::c_char,
    mut startcol: ::core::ffi::c_int,
    mut curs_col: colnr_T,
    mut line_invalid: *mut bool,
) -> ::core::ffi::c_int {
    if ctrl_x_mode_normal() as ::core::ffi::c_int != 0
        || ctrl_x_mode_register() as ::core::ffi::c_int != 0
        || ctrl_x_mode.get() & CTRL_X_WANT_IDENT != 0 && !thesaurus_func_complete(ctrl_x_mode.get())
    {
        if get_normal_compl_info(line, startcol, curs_col) != OK {
            return FAIL;
        }
        *line_invalid = true_0 != 0;
    } else if ctrl_x_mode_line_or_eval() {
        return get_wholeline_compl_info(line, curs_col);
    } else if ctrl_x_mode_files() {
        return get_filename_compl_info(line, startcol, curs_col);
    } else if ctrl_x_mode.get() == CTRL_X_CMDLINE as ::core::ffi::c_int {
        return get_cmdline_compl_info(line, curs_col);
    } else if ctrl_x_mode_function() as ::core::ffi::c_int != 0
        || ctrl_x_mode_omni() as ::core::ffi::c_int != 0
        || thesaurus_func_complete(ctrl_x_mode.get()) as ::core::ffi::c_int != 0
    {
        if get_userdefined_compl_info(
            curs_col,
            ::core::ptr::null_mut::<Callback>(),
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ) != OK
        {
            return FAIL;
        }
        *line_invalid = true_0 != 0;
    } else if ctrl_x_mode_spell() {
        if get_spell_compl_info(startcol, curs_col) == FAIL {
            return FAIL;
        }
        *line_invalid = true_0 != 0;
    } else {
        internal_error(b"ins_complete()\0".as_ptr() as *const ::core::ffi::c_char);
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn ins_compl_continue_search(mut line: *mut ::core::ffi::c_char) {
    (*compl_cont_status.ptr()) &= !CONT_INTRPT;
    if ctrl_x_mode_normal() as ::core::ffi::c_int != 0
        || ctrl_x_mode_path_patterns() as ::core::ffi::c_int != 0
        || ctrl_x_mode_path_defines() as ::core::ffi::c_int != 0
    {
        if (*compl_startpos.ptr()).lnum != (*curwin.get()).w_cursor.lnum {
            compl_col.set(getwhitecols(line) as colnr_T);
            (*compl_startpos.ptr()).col = compl_col.get();
            (*compl_startpos.ptr()).lnum = (*curwin.get()).w_cursor.lnum;
            (*compl_cont_status.ptr()) &= !CONT_SOL;
        } else {
            if compl_cont_status.get() & CONT_S_IPOS != 0 {
                (*compl_cont_status.ptr()) |= CONT_SOL;
                (*compl_startpos.ptr()).col = skipwhite(
                    line.offset(compl_length.get() as isize)
                        .offset((*compl_startpos.ptr()).col as isize),
                )
                .offset_from(line) as colnr_T;
            }
            compl_col.set((*compl_startpos.ptr()).col);
        }
        compl_length.set((*curwin.get()).w_cursor.col as ::core::ffi::c_int - compl_col.get());
        if compl_length.get() > IOSIZE - MIN_SPACE {
            (*compl_cont_status.ptr()) &= !CONT_SOL;
            compl_length.set(IOSIZE - MIN_SPACE);
            compl_col.set(
                ((*curwin.get()).w_cursor.col as ::core::ffi::c_int - compl_length.get())
                    as colnr_T,
            );
        }
        (*compl_cont_status.ptr()) |= CONT_ADDING | CONT_N_ADDS;
        if compl_length.get() < 1 as ::core::ffi::c_int {
            (*compl_cont_status.ptr()) &= CONT_LOCAL;
        }
    } else if ctrl_x_mode_line_or_eval() as ::core::ffi::c_int != 0
        || ctrl_x_mode_register() as ::core::ffi::c_int != 0
    {
        compl_cont_status.set(CONT_ADDING | CONT_N_ADDS);
    } else {
        compl_cont_status.set(0 as ::core::ffi::c_int);
    };
}
pub const MIN_SPACE: ::core::ffi::c_int = 75 as ::core::ffi::c_int;
unsafe extern "C" fn ins_compl_start() -> ::core::ffi::c_int {
    let save_did_ai: bool = did_ai.get();
    did_ai.set(false_0 != 0);
    did_si.set(false_0 != 0);
    can_si.set(false_0 != 0);
    can_si_back.set(false_0 != 0);
    if stop_arrow() == FAIL {
        did_ai.set(save_did_ai);
        return FAIL;
    }
    let mut line: *mut ::core::ffi::c_char = ml_get((*curwin.get()).w_cursor.lnum);
    let mut curs_col: colnr_T = (*curwin.get()).w_cursor.col;
    compl_pending.set(0 as ::core::ffi::c_int);
    compl_lnum.set((*curwin.get()).w_cursor.lnum);
    if compl_cont_status.get() & CONT_INTRPT == CONT_INTRPT
        && compl_cont_mode.get() == ctrl_x_mode.get()
    {
        ins_compl_continue_search(line);
    } else {
        (*compl_cont_status.ptr()) &= CONT_LOCAL;
    }
    let mut startcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !compl_status_adding() {
        compl_cont_mode.set(ctrl_x_mode.get());
        if ctrl_x_mode_not_default() {
            compl_cont_status.set(0 as ::core::ffi::c_int);
        }
        (*compl_cont_status.ptr()) |= CONT_N_ADDS;
        compl_startpos.set((*curwin.get()).w_cursor);
        startcol = curs_col;
        compl_col.set(0 as ::core::ffi::c_int as colnr_T);
    }
    let mut line_invalid: bool = false_0 != 0;
    if compl_get_info(line, startcol, curs_col, &raw mut line_invalid) == FAIL {
        if ctrl_x_mode_function() as ::core::ffi::c_int != 0
            || ctrl_x_mode_omni() as ::core::ffi::c_int != 0
            || thesaurus_func_complete(ctrl_x_mode.get()) as ::core::ffi::c_int != 0
        {
            did_ai.set(save_did_ai);
        }
        return FAIL;
    }
    if line_invalid {
        line = ml_get((*curwin.get()).w_cursor.lnum);
    }
    if compl_status_adding() {
        if !shortmess(SHM_COMPLETIONMENU as ::core::ffi::c_int) {
            edit_submode_pre.set(gettext(b" Adding\0".as_ptr() as *const ::core::ffi::c_char));
        }
        if ctrl_x_mode_line_or_eval() {
            let mut old: *mut ::core::ffi::c_char = (*curbuf.get()).b_p_com;
            (*curbuf.get()).b_p_com =
                b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            (*compl_startpos.ptr()).lnum = (*curwin.get()).w_cursor.lnum;
            (*compl_startpos.ptr()).col = compl_col.get();
            ins_eol('\r' as ::core::ffi::c_int);
            (*curbuf.get()).b_p_com = old;
            compl_length.set(0 as ::core::ffi::c_int);
            compl_col.set((*curwin.get()).w_cursor.col);
            compl_lnum.set((*curwin.get()).w_cursor.lnum);
        }
    } else {
        edit_submode_pre.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        (*compl_startpos.ptr()).col = compl_col.get();
    }
    if !shortmess(SHM_COMPLETIONMENU as ::core::ffi::c_int) && !compl_autocomplete.get() {
        if compl_cont_status.get() & CONT_LOCAL != 0 {
            edit_submode.set(gettext(
                (*ctrl_x_msgs.ptr())[CTRL_X_LOCAL_MSG as ::core::ffi::c_int as usize],
            ));
        } else {
            edit_submode.set(gettext(
                (*ctrl_x_msgs.ptr())[(ctrl_x_mode.get() & !(0x100 as ::core::ffi::c_int)) as usize],
            ));
        }
    }
    ins_compl_fixRedoBufForLeader(::core::ptr::null_mut::<::core::ffi::c_char>());
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*compl_orig_text.ptr()).data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    (*compl_orig_text.ptr()).size = 0 as size_t;
    xfree((*compl_orig_extmarks.ptr()).items as *mut ::core::ffi::c_void);
    (*compl_orig_extmarks.ptr()).capacity = 0 as size_t;
    (*compl_orig_extmarks.ptr()).size = (*compl_orig_extmarks.ptr()).capacity;
    (*compl_orig_extmarks.ptr()).items = ::core::ptr::null_mut::<ExtmarkUndoObject>();
    compl_orig_text.set(cbuf_to_string(
        line.offset(compl_col.get() as isize),
        compl_length.get() as size_t,
    ));
    save_orig_extmarks();
    let mut flags: ::core::ffi::c_int = CP_ORIGINAL_TEXT as ::core::ffi::c_int;
    if p_ic.get() != 0 {
        flags |= CP_ICASE as ::core::ffi::c_int;
    }
    if ins_compl_add(
        (*compl_orig_text.ptr()).data,
        (*compl_orig_text.ptr()).size as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null::<*mut ::core::ffi::c_char>(),
        false_0 != 0,
        ::core::ptr::null_mut::<typval_T>(),
        kDirectionNotSet,
        flags,
        false_0 != 0,
        ::core::ptr::null::<::core::ffi::c_int>(),
        FUZZY_SCORE_NONE as ::core::ffi::c_int,
    ) != OK
    {
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut (*compl_pattern.ptr()).data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL;
        let _ = *ptr__0;
        (*compl_pattern.ptr()).size = 0 as size_t;
        let mut ptr__1: *mut *mut ::core::ffi::c_void =
            &raw mut (*compl_orig_text.ptr()).data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__1);
        *ptr__1 = NULL;
        let _ = *ptr__1;
        (*compl_orig_text.ptr()).size = 0 as size_t;
        xfree((*compl_orig_extmarks.ptr()).items as *mut ::core::ffi::c_void);
        (*compl_orig_extmarks.ptr()).capacity = 0 as size_t;
        (*compl_orig_extmarks.ptr()).size = (*compl_orig_extmarks.ptr()).capacity;
        (*compl_orig_extmarks.ptr()).items = ::core::ptr::null_mut::<ExtmarkUndoObject>();
        did_ai.set(save_did_ai);
        return FAIL;
    }
    if !shortmess(SHM_COMPLETIONMENU as ::core::ffi::c_int) && !compl_autocomplete.get() {
        edit_submode_extra.set(gettext(
            b"-- Searching...\0".as_ptr() as *const ::core::ffi::c_char
        ));
        edit_submode_highl.set(HLF_COUNT);
        showmode();
        edit_submode_extra.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        ui_flush();
    }
    did_ai.set(save_did_ai);
    return OK;
}
unsafe extern "C" fn ins_compl_show_statusmsg() {
    if is_first_match((*compl_first_match.get()).cp_next) {
        edit_submode_extra.set(
            if compl_status_adding() as ::core::ffi::c_int != 0
                && compl_length.get() > 1 as ::core::ffi::c_int
            {
                gettext((e_hitend.ptr() as *const _) as *const ::core::ffi::c_char)
            } else {
                gettext(&raw const e_patnotf as *const ::core::ffi::c_char)
            },
        );
        edit_submode_highl.set(HLF_E);
    }
    if (*edit_submode_extra.ptr()).is_null() {
        if match_at_original_text(compl_curr_match.get()) {
            edit_submode_extra.set(gettext(
                b"Back at original\0".as_ptr() as *const ::core::ffi::c_char
            ));
            edit_submode_highl.set(HLF_W);
        } else if compl_cont_status.get() & CONT_S_IPOS != 0 {
            edit_submode_extra.set(gettext(
                b"Word from other line\0".as_ptr() as *const ::core::ffi::c_char
            ));
            edit_submode_highl.set(HLF_COUNT);
        } else if (*compl_curr_match.get()).cp_next == (*compl_curr_match.get()).cp_prev {
            edit_submode_extra.set(gettext(
                b"The only match\0".as_ptr() as *const ::core::ffi::c_char
            ));
            edit_submode_highl.set(HLF_COUNT);
            (*compl_curr_match.get()).cp_number = 1 as ::core::ffi::c_int;
        } else {
            if (*compl_curr_match.get()).cp_number == -1 as ::core::ffi::c_int {
                ins_compl_update_sequence_numbers();
            }
            if (*compl_curr_match.get()).cp_number != -1 as ::core::ffi::c_int {
                static match_ref: GlobalCell<[::core::ffi::c_char; 81]> = GlobalCell::new([0; 81]);
                if compl_matches.get() > 0 as ::core::ffi::c_int {
                    vim_snprintf(
                        match_ref.ptr() as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 81]>(),
                        gettext(b"match %d of %d\0".as_ptr() as *const ::core::ffi::c_char),
                        (*compl_curr_match.get()).cp_number,
                        compl_matches.get(),
                    );
                } else {
                    vim_snprintf(
                        match_ref.ptr() as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 81]>(),
                        gettext(b"match %d\0".as_ptr() as *const ::core::ffi::c_char),
                        (*compl_curr_match.get()).cp_number,
                    );
                }
                edit_submode_extra.set(match_ref.ptr() as *mut ::core::ffi::c_char);
                edit_submode_highl.set(HLF_R);
                if dollar_vcol.get() >= 0 as ::core::ffi::c_int {
                    curs_columns(curwin.get(), false_0);
                }
            }
        }
    }
    redraw_mode.set(true_0 != 0);
    if !shortmess(SHM_COMPLETIONMENU as ::core::ffi::c_int) {
        if !(*edit_submode_extra.ptr()).is_null() {
            if p_smd.get() == 0 {
                msg_hist_off.set(true_0 != 0);
                msg_ext_set_kind(b"completion\0".as_ptr() as *const ::core::ffi::c_char);
                msg(
                    edit_submode_extra.get(),
                    if (edit_submode_highl.get() as ::core::ffi::c_uint)
                        < HLF_COUNT as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        edit_submode_highl.get() as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    },
                );
                msg_hist_off.set(false_0 != 0);
            }
        } else {
            msg_clr_cmdline();
        }
    }
}
pub unsafe extern "C" fn ins_complete(
    mut c: ::core::ffi::c_int,
    mut enable_pum: bool,
) -> ::core::ffi::c_int {
    let disable_ac_delay: bool = compl_started.get() as ::core::ffi::c_int != 0
        && ctrl_x_mode_normal() as ::core::ffi::c_int != 0
        && (c == Ctrl_N
            || c == Ctrl_P
            || c == Ctrl_R
            || ins_compl_pum_key(c) as ::core::ffi::c_int != 0);
    compl_direction.set(ins_compl_key2dir(c) as Direction);
    let mut insert_match: ::core::ffi::c_int = ins_compl_use_match(c) as ::core::ffi::c_int;
    if !compl_started.get() {
        if ins_compl_start() == FAIL {
            return FAIL;
        }
    } else if insert_match != 0 && stop_arrow() == FAIL {
        return FAIL;
    }
    let mut compl_start_tv: uint64_t = 0 as uint64_t;
    if compl_autocomplete.get() as ::core::ffi::c_int != 0
        && p_acl.get() > 0 as OptInt
        && !disable_ac_delay
    {
        compl_start_tv = os_hrtime();
    }
    compl_curr_win.set(curwin.get());
    compl_curr_buf.set((*curwin.get()).w_buffer);
    compl_shown_match.set(compl_curr_match.get());
    compl_shows_dir.set(compl_direction.get());
    compl_num_bests.set(0 as ::core::ffi::c_int);
    let mut save_w_wrow: ::core::ffi::c_int = (*curwin.get()).w_wrow;
    let mut save_w_leftcol: ::core::ffi::c_int = (*curwin.get()).w_leftcol as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int =
        ins_compl_next(true_0 != 0, ins_compl_key2count(c), insert_match != 0);
    if compl_autocomplete.get() {
        compl_time_slice_expired.set(false_0 != 0);
    }
    if n > 1 as ::core::ffi::c_int {
        compl_matches.set(n);
    }
    compl_curr_match.set(compl_shown_match.get());
    compl_direction.set(compl_shows_dir.get());
    if got_int.get() as ::core::ffi::c_int != 0 && global_busy.get() == 0 {
        vgetc();
        got_int.set(false_0 != 0);
    }
    let mut no_matches_found: bool = is_first_match((*compl_first_match.get()).cp_next);
    if no_matches_found {
        if compl_length.get() > 1 as ::core::ffi::c_int
            || compl_status_adding() as ::core::ffi::c_int != 0
            || ctrl_x_mode_not_default() as ::core::ffi::c_int != 0
                && !ctrl_x_mode_path_patterns()
                && !ctrl_x_mode_path_defines()
        {
            (*compl_cont_status.ptr()) &= !CONT_N_ADDS;
        }
    }
    if (*compl_curr_match.get()).cp_flags & CP_CONT_S_IPOS as ::core::ffi::c_int != 0 {
        (*compl_cont_status.ptr()) |= CONT_S_IPOS;
    } else {
        (*compl_cont_status.ptr()) &= !CONT_S_IPOS;
    }
    if !shortmess(SHM_COMPLETIONMENU as ::core::ffi::c_int) && !compl_autocomplete.get() {
        ins_compl_show_statusmsg();
    }
    if compl_autocomplete.get() as ::core::ffi::c_int != 0
        && p_acl.get() > 0 as OptInt
        && !disable_ac_delay
        && !no_matches_found
        && os_hrtime()
            .wrapping_sub(compl_start_tv)
            .wrapping_div(1000000 as uint64_t)
            < p_acl.get() as uint64_t
    {
        setcursor();
        ui_flush();
        loop {
            if char_avail() {
                if ins_compl_preinsert_effect() as ::core::ffi::c_int != 0
                    && ins_compl_win_active(curwin.get()) as ::core::ffi::c_int != 0
                {
                    ins_compl_delete(false_0 != 0);
                    compl_ins_end_col.set(compl_col.get());
                }
                ins_compl_restart();
                compl_interrupted.set(true_0 != 0);
                break;
            } else {
                os_delay(2 as uint64_t, true_0 != 0);
                if os_hrtime()
                    .wrapping_sub(compl_start_tv)
                    .wrapping_div(1000000 as uint64_t)
                    >= p_acl.get() as uint64_t
                {
                    break;
                }
            }
        }
    }
    if enable_pum as ::core::ffi::c_int != 0 && !compl_interrupted.get() {
        show_pum(save_w_wrow, save_w_leftcol);
    }
    compl_was_interrupted.set(compl_interrupted.get());
    compl_interrupted.set(false_0 != 0);
    return OK;
}
pub unsafe extern "C" fn ins_compl_enable_autocomplete() {
    compl_autocomplete.set(true_0 != 0);
    compl_get_longest.set(false_0 != 0);
}
unsafe extern "C" fn show_pum(
    mut prev_w_wrow: ::core::ffi::c_int,
    mut prev_w_leftcol: ::core::ffi::c_int,
) {
    let mut n: ::core::ffi::c_int = RedrawingDisabled.get();
    RedrawingDisabled.set(0 as ::core::ffi::c_int);
    setcursor();
    if prev_w_wrow != (*curwin.get()).w_wrow || prev_w_leftcol != (*curwin.get()).w_leftcol {
        ins_compl_del_pum();
    }
    ins_compl_show_pum();
    setcursor();
    RedrawingDisabled.set(n);
}
unsafe extern "C" fn quote_meta(
    mut dest: *mut ::core::ffi::c_char,
    mut src: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> ::core::ffi::c_uint {
    let mut m: ::core::ffi::c_uint =
        (len as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint);
    loop {
        len -= 1;
        if len < 0 as ::core::ffi::c_int {
            break;
        }
        's_85: {
            'c_56947: {
                'c_56925: {
                    match *src as ::core::ffi::c_int {
                        46 | 42 | 91 => {
                            if ctrl_x_mode_dictionary() as ::core::ffi::c_int != 0
                                || ctrl_x_mode_thesaurus() as ::core::ffi::c_int != 0
                            {
                                break 's_85;
                            }
                        }
                        126 => {}
                        92 => {
                            break 'c_56925;
                        }
                        94 | 36 => {
                            break 'c_56947;
                        }
                        _ => {
                            break 's_85;
                        }
                    }
                    if !magic_isset() {
                        break 's_85;
                    }
                }
                if ctrl_x_mode_dictionary() as ::core::ffi::c_int != 0
                    || ctrl_x_mode_thesaurus() as ::core::ffi::c_int != 0
                {
                    break 's_85;
                }
            }
            m = m.wrapping_add(1);
            if !dest.is_null() {
                let c2rust_fresh9 = dest;
                dest = dest.offset(1);
                *c2rust_fresh9 = '\\' as ::core::ffi::c_char;
            }
        }
        if !dest.is_null() {
            let c2rust_fresh10 = dest;
            dest = dest.offset(1);
            *c2rust_fresh10 = *src;
        }
        let mb_len: ::core::ffi::c_int = utfc_ptr2len(src) - 1 as ::core::ffi::c_int;
        if mb_len > 0 as ::core::ffi::c_int && len >= mb_len {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < mb_len {
                len -= 1;
                src = src.offset(1);
                if !dest.is_null() {
                    let c2rust_fresh11 = dest;
                    dest = dest.offset(1);
                    *c2rust_fresh11 = *src;
                }
                i += 1;
            }
        }
        src = src.offset(1);
    }
    if !dest.is_null() {
        *dest = NUL as ::core::ffi::c_char;
    }
    return m;
}
unsafe extern "C" fn spell_back_to_badword() {
    let mut tpos: pos_T = (*curwin.get()).w_cursor;
    spell_bad_len.set(spell_move_to(
        curwin.get(),
        BACKWARD as ::core::ffi::c_int,
        SMT_ALL,
        true_0 != 0,
        ::core::ptr::null_mut::<hlf_T>(),
    ));
    if (*curwin.get()).w_cursor.col != tpos.col {
        start_arrow(&raw mut tpos);
    }
}
unsafe extern "C" fn cpt_sources_clear() {
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        cpt_sources_array.ptr() as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    cpt_sources_index.set(-1 as ::core::ffi::c_int);
    cpt_sources_count.set(0 as ::core::ffi::c_int);
}
unsafe extern "C" fn setup_cpt_sources() {
    cpt_sources_clear();
    let mut count: ::core::ffi::c_int = get_cpt_sources_count();
    if count == 0 as ::core::ffi::c_int {
        return;
    }
    cpt_sources_array
        .set(xcalloc(count as size_t, ::core::mem::size_of::<cpt_source_T>()) as *mut cpt_source_T);
    let mut buf: [::core::ffi::c_char; 512] = [0; 512];
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char = (*curbuf.get()).b_p_cpt;
    while *p != 0 {
        while *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
        {
            p = p.offset(1);
        }
        if *p != 0 {
            (*(*cpt_sources_array.ptr()).offset(idx as isize)).cs_flag = *p;
            memset(
                &raw mut buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                LSIZE as ::core::ffi::c_int as size_t,
            );
            let mut slen: size_t = copy_option_part(
                &raw mut p,
                &raw mut buf as *mut ::core::ffi::c_char,
                LSIZE as ::core::ffi::c_int as size_t,
                b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            if slen > 0 as size_t {
                let mut caret: *mut ::core::ffi::c_char = vim_strchr(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    '^' as ::core::ffi::c_int,
                );
                if !caret.is_null() {
                    (*(*cpt_sources_array.ptr()).offset(idx as isize)).cs_max_matches =
                        atoi(caret.offset(1 as ::core::ffi::c_int as isize));
                }
            }
            idx += 1;
        }
    }
    cpt_sources_count.set(count);
}
unsafe extern "C" fn is_cpt_func_refresh_always() -> bool {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < cpt_sources_count.get() {
        if (*(*cpt_sources_array.ptr()).offset(i as isize)).cs_refresh_always {
            return true_0 != 0;
        }
        i += 1;
    }
    return false_0 != 0;
}
unsafe extern "C" fn ins_compl_make_linear() {
    if (*compl_first_match.ptr()).is_null() || (*compl_first_match.get()).cp_prev.is_null() {
        return;
    }
    let mut m: *mut compl_T = (*compl_first_match.get()).cp_prev;
    (*m).cp_next = ::core::ptr::null_mut::<compl_T>();
    (*compl_first_match.get()).cp_prev = ::core::ptr::null_mut::<compl_T>();
}
unsafe extern "C" fn remove_old_matches() {
    let mut shown_match_removed: bool = false_0 != 0;
    let mut forward: bool = (*compl_first_match.get()).cp_cpt_source_idx < 0 as ::core::ffi::c_int;
    if cpt_sources_index.get() < 0 as ::core::ffi::c_int {
        return;
    }
    compl_direction.set(
        (if forward as ::core::ffi::c_int != 0 {
            FORWARD as ::core::ffi::c_int
        } else {
            BACKWARD as ::core::ffi::c_int
        }) as Direction,
    );
    compl_shows_dir.set(compl_direction.get());
    let mut current: *mut compl_T = compl_first_match.get();
    while !current.is_null() {
        if (*current).cp_cpt_source_idx == cpt_sources_index.get() {
            let mut to_delete: *mut compl_T = current;
            if !shown_match_removed && compl_shown_match.get() == current {
                shown_match_removed = true_0 != 0;
            }
            current = (*current).cp_next;
            if to_delete == compl_first_match.get() {
                compl_first_match.set((*to_delete).cp_next);
                (*compl_first_match.get()).cp_prev = ::core::ptr::null_mut::<compl_T>();
            } else if (*to_delete).cp_next.is_null() {
                (*(*to_delete).cp_prev).cp_next = ::core::ptr::null_mut::<compl_T>();
            } else {
                (*(*to_delete).cp_prev).cp_next = (*to_delete).cp_next;
                (*(*to_delete).cp_next).cp_prev = (*to_delete).cp_prev;
            }
            ins_compl_item_free(to_delete);
        } else {
            current = (*current).cp_next;
        }
    }
    if shown_match_removed {
        if forward {
            compl_shown_match.set(compl_first_match.get());
        } else {
            let mut current_0: *mut compl_T = ::core::ptr::null_mut::<compl_T>();
            current_0 = compl_first_match.get();
            while !(*current_0).cp_next.is_null() {
                current_0 = (*current_0).cp_next;
            }
            compl_shown_match.set(current_0);
        }
    }
    compl_curr_match.set(compl_first_match.get());
    let mut current_1: *mut compl_T = compl_first_match.get();
    while !current_1.is_null() {
        if if forward as ::core::ffi::c_int != 0 {
            ((*current_1).cp_cpt_source_idx < cpt_sources_index.get()) as ::core::ffi::c_int
        } else {
            ((*current_1).cp_cpt_source_idx > cpt_sources_index.get()) as ::core::ffi::c_int
        } == 0
        {
            break;
        }
        compl_curr_match.set(if forward as ::core::ffi::c_int != 0 {
            current_1
        } else {
            (*current_1).cp_next
        });
        current_1 = (*current_1).cp_next;
    }
}
unsafe extern "C" fn get_cpt_func_completion_matches(mut cb: *mut Callback) {
    let mut cpt_src: *mut cpt_source_T =
        (*cpt_sources_array.ptr()).offset(cpt_sources_index.get() as isize);
    let mut startcol: ::core::ffi::c_int = (*cpt_src).cs_startcol;
    if startcol == -2 as ::core::ffi::c_int || startcol == -3 as ::core::ffi::c_int {
        return;
    }
    set_compl_globals(startcol, (*curwin.get()).w_cursor.col, true_0 != 0);
    if !(*cpt_src).cs_refresh_always {
        ins_compl_insert_bytes(ins_compl_leader(), -1 as ::core::ffi::c_int);
    }
    expand_by_function(0 as ::core::ffi::c_int, (*cpt_compl_pattern.ptr()).data, cb);
    if !(*cpt_src).cs_refresh_always {
        ins_compl_delete(false_0 != 0);
    }
    (*cpt_src).cs_refresh_always = compl_opt_refresh_always.get();
    compl_opt_refresh_always.set(false_0 != 0);
}
unsafe extern "C" fn cpt_compl_refresh() {
    ins_compl_make_linear();
    let mut cpt: *mut ::core::ffi::c_char = xstrdup((*curbuf.get()).b_p_cpt);
    strip_caret_numbers_in_place(cpt);
    cpt_sources_index.set(0 as ::core::ffi::c_int);
    let mut p: *mut ::core::ffi::c_char = cpt;
    while *p != 0 {
        while *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
        {
            p = p.offset(1);
        }
        if *p as ::core::ffi::c_int == NUL {
            break;
        }
        if (*(*cpt_sources_array.ptr()).offset(cpt_sources_index.get() as isize)).cs_refresh_always
        {
            let mut cb: *mut Callback = get_callback_if_cpt_func(p, cpt_sources_index.get());
            if !cb.is_null() {
                remove_old_matches();
                let mut startcol: ::core::ffi::c_int = 0;
                let mut ret: ::core::ffi::c_int =
                    get_userdefined_compl_info((*curwin.get()).w_cursor.col, cb, &raw mut startcol);
                if ret == FAIL {
                    if startcol == -3 as ::core::ffi::c_int {
                        (*(*cpt_sources_array.ptr()).offset(cpt_sources_index.get() as isize))
                            .cs_refresh_always = false_0 != 0;
                    } else {
                        startcol = -2 as ::core::ffi::c_int;
                    }
                } else if startcol < 0 as ::core::ffi::c_int
                    || startcol > (*curwin.get()).w_cursor.col
                {
                    startcol = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
                }
                (*(*cpt_sources_array.ptr()).offset(cpt_sources_index.get() as isize))
                    .cs_startcol = startcol;
                if ret == OK {
                    compl_source_start_timer(cpt_sources_index.get());
                    get_cpt_func_completion_matches(cb);
                }
            }
        }
        copy_option_part(
            &raw mut p,
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        if may_advance_cpt_index(p) {
            advance_cpt_sources_index_safe();
        }
    }
    cpt_sources_index.set(-1 as ::core::ffi::c_int);
    xfree(cpt as *mut ::core::ffi::c_void);
    compl_matches.set(ins_compl_make_cyclic());
}
pub unsafe extern "C" fn f_preinserted(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if ins_compl_preinsert_effect() {
        (*rettv).vval.v_number = 1 as varnumber_T;
    }
}
pub const K_UP: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('u' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_DOWN: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('d' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_S_TAB: ::core::ffi::c_int = -17003;
pub const K_BS: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('b' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_PAGEUP: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('P' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_PAGEDOWN: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('N' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KPAGEUP: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('3' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KPAGEDOWN: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('5' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KENTER: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('A' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_SELECT: ::core::ffi::c_int =
    -(245 as ::core::ffi::c_int + (('X' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
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
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
