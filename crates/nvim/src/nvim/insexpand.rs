#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr;

use crate::src::nvim::api::private::helpers::{cbuf_to_string, copy_string, cstr_as_string};
use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite, ascii_iswhite_or_nul};
use crate::src::nvim::autocmd::{
    EVENT_COMPLETECHANGED, EVENT_COMPLETEDONE, EVENT_COMPLETEDONEPRE, apply_autocmds, has_event,
};
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
    UPD_VALID, redraw_later, redrawWinline, setcursor, showmode, update_screen,
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
use crate::src::nvim::eval::typval::{kCallbackNone, tv_list_first};
use crate::src::nvim::eval::userfunc::callback_call_retnr;
use crate::src::nvim::eval::vars::set_vim_var_dict;
use crate::src::nvim::eval::{callback_call, get_v_event, restore_v_event, set_ref_in_callback};
use crate::src::nvim::ex_eval::aborting;
use crate::src::nvim::ex_getln::tilde_replace;
use crate::src::nvim::extmark::{extmark_apply_undo, extmark_splice_delete};
use crate::src::nvim::fileio::vim_fgets;
use crate::src::nvim::fuzzy::{fuzzy_match_str, fuzzy_match_str_in_line, search_for_fuzzy_match};
use crate::src::nvim::garray::{ga_clear, ga_grow, ga_init};
use crate::src::nvim::getchar::{
    AppendCharToRedobuff, AppendToRedobuffLit, char_avail, safe_vgetc, using_script, vgetc, vpeekc,
    vpeekc_any, vungetc,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight_group::{HLF_COUNT, HLF_E, HLF_R, HLF_W, syn_name2attr};
use crate::src::nvim::indent::{get_indent, inindent};
use crate::src::nvim::indent_c::{cindent_on, do_c_expr_indent, in_cinkeys};
use crate::src::nvim::keycodes::{
    K_BS, K_COMMAND, K_DOWN, K_EVENT, K_KENTER, K_KPAGEDOWN, K_KPAGEUP, K_LUA, K_PAGEDOWN,
    K_PAGEUP, K_S_DOWN, K_S_TAB, K_S_UP, K_SELECT, K_UP,
};
use crate::src::nvim::lua::executor::nlua_expand_pat;
use crate::src::nvim::main::{
    IObuff, KeyTyped, RedrawingDisabled, State, arrow_used, can_si, can_si_back, cmdwin_type,
    cot_flags, curbuf, curwin, did_ai, did_emsg, did_si, dollar_vcol, e_invarg,
    e_list_index_out_of_range_nr, e_listreq, e_notset, e_patnotf, edit_submode, edit_submode_extra,
    edit_submode_highl, edit_submode_pre, emsg_off, emsg_silent, ex_normal_busy, firstbuf,
    firstwin, g_tag_at_cursor, global_busy, got_int, in_assert_fails, msg_hist_off, msg_silent,
    p_ac, p_acl, p_act, p_cto, p_dict, p_fic, p_ic, p_inf, p_js, p_paste, p_scs, p_smd, p_tsr,
    p_tsrfu, p_wic, p_ws, pum_want, redraw_cmdline, redraw_mode, sc_col, test_disable_char_avail,
    textlock,
};
use crate::src::nvim::mbyte::{
    mb_get_class, mb_islower, mb_isupper, mb_prevptr, mb_ptr2char_adv, mb_tolower, mb_toupper,
    utf_char2bytes, utf_char2len, utf_head_off, utf_ptr2char, utf_ptr2len, utf8len_tab,
    utfc_ptr2len,
};
use crate::src::nvim::memline::{dec, ml_delete, ml_get, ml_get_buf, ml_get_buf_len, ml_get_len};
use crate::src::nvim::memory::{
    MergeSortCompareFunc, mergesort_list, strequal, xcalloc, xfree, xmalloc, xmemdupz, xstrdup,
    xstrlcpy,
};
use crate::src::nvim::message::{
    emsg, internal_error, msg, msg_clr_cmdline, msg_delay, msg_ext_set_kind, msg_progress, semsg,
};
use crate::src::nvim::r#move::{changed_cline_bef_curs, curs_columns, validate_cursor};
use crate::src::nvim::option::{
    can_bs, copy_option_part, magic_isset, option_set_callback_func, shortmess,
};
use crate::src::nvim::options::{
    kOptBoFlagComplete, kOptCotFlagFuzzy, kOptCotFlagLongest, kOptCotFlagMenu, kOptCotFlagMenuone,
    kOptCotFlagNearest, kOptCotFlagNoinsert, kOptCotFlagNoselect, kOptCotFlagNosort,
    kOptCotFlagPreinsert,
};
use crate::src::nvim::os::fs::os_fopen;
use crate::src::nvim::os::input::{fast_breakcheck, line_breakcheck, os_breakcheck};
use crate::src::nvim::os::libc::{
    __assert_fail, abs, atoi, fclose, gettext, memcmp, memmove, memset, qsort, strcat, strchr,
    strcmp, strcpy, strlen, strncasecmp, strncmp, strncpy, strrchr,
};
use crate::src::nvim::os::time::{os_delay, os_hrtime};
use crate::src::nvim::path::{FreeWild, expand_wildcards, path_tail, vim_ispathsep};
use crate::src::nvim::popupmenu::{
    pum_clear, pum_display, pum_get_height, pum_set_event_info, pum_undisplay, pum_visible,
};
use crate::src::nvim::pos::{MAXCOL, MAXLNUM, equalpos};
use crate::src::nvim::regexp::{RE_LAST, RE_MAGIC};
use crate::src::nvim::register::get_register_name;
use crate::src::nvim::register::{copy_register, free_register, valid_yank_reg};
use crate::src::nvim::search::{
    BACKWARD, FORWARD, SEARCH_KEEP, SEARCH_NFMSG, find_pattern_in_path, ignorecase,
    search_for_exact_line, searchit,
};
use crate::src::nvim::spell::{
    SMT_ALL, expand_spelling, spell_dump_compl, spell_expand_check_cap, spell_move_to,
    spell_word_start,
};
use crate::src::nvim::state::{MODE_INSERT, REPLACE_FLAG, may_trigger_modechanged};
use crate::src::nvim::strings::{vim_snprintf, vim_strchr, vim_strsave_escaped};
use crate::src::nvim::tag::find_tags;
use crate::src::nvim::textformat::auto_format;
use crate::src::nvim::types::{
    Arena, BoolVarValue, Callback, Callback_data as C2Rust_Unnamed_5, Direction, EvalFuncData,
    ExtmarkOp, ExtmarkUndoObject, FILE, ListLenSpecials, OptInt, ScopeType, SpecialVarValue,
    String_0, VV_COMPLETED_ITEM, VarLockStatus, VarType, buf_T, colnr_T, dict_T, dictitem_T,
    expand_T, extmark_undo_vec_t, garray_T, hashitem_T, hashtab_T, hlf_T, ht_stack_T, key_extra,
    linenr_T, list_T, list_stack_T, listitem_T, optset_T, pos_T, ptrdiff_t, pumitem_T, regmatch_T,
    regprog_T, save_v_event_T, sctx_T, searchit_arg_T, size_t, typval_T, typval_vval_union,
    uint8_t, uint64_t, varnumber_T, win_T, xp_prefix_T, yankreg_T,
};
use crate::src::nvim::ui::{ui_flush, vim_beep};
use crate::src::nvim::undo::undo_allowed;
use crate::src::nvim::window::win_valid;
use crate::src::nvim::winfloat::win_float_find_preview;

// The carve of the transpiled module; see each child's docs.
mod mode;
pub use self::mode::*;
mod matchlist;
pub use self::matchlist::*;
mod text;
pub use self::text::*;
mod pum;
pub use self::pum::*;
mod sources;
pub(crate) use self::sources::*;
mod getexp;
pub(crate) use self::getexp::*;
mod callbacks;
pub use self::callbacks::*;
mod vimscript;
pub use self::vimscript::*;
mod insert;
pub use self::insert::*;
mod session;
pub use self::session::*;
mod keys;
pub use self::keys::*;
unsafe extern "C" {
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
}
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kSpecialVarNull: SpecialVarValue = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
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
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kDirectionNotSet: Direction = 0;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_int;
pub const EXPAND_LUA: C2Rust_Unnamed_16 = 63;
pub const EXPAND_FILES: C2Rust_Unnamed_16 = 2;
pub const EXPAND_NOTHING: C2Rust_Unnamed_16 = 0;
pub const EXPAND_OK: C2Rust_Unnamed_16 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_16 = -2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_int;
pub const OPENLINE_FORCE_INDENT: C2Rust_Unnamed_17 = 64;
pub const OPENLINE_KEEPTRAIL: C2Rust_Unnamed_17 = 4;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_int;
pub const SHM_COMPLETIONSCAN: C2Rust_Unnamed_20 = 67;
pub const SHM_COMPLETIONMENU: C2Rust_Unnamed_20 = 99;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_int;
pub const KEY_COMPLETE: C2Rust_Unnamed_22 = 259;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_int;
pub const FUZZY_SCORE_NONE: C2Rust_Unnamed_23 = -2147483648;
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
pub const KE_MOUSEMOVE: key_extra = 100;
pub const KE_MOUSERIGHT: key_extra = 78;
pub const KE_MOUSELEFT: key_extra = 77;
pub const KE_MOUSEUP: key_extra = 76;
pub const KE_MOUSEDOWN: key_extra = 75;
pub const OPT_GLOBAL: C2Rust_Unnamed_27 = 1;
pub const OPT_LOCAL: C2Rust_Unnamed_27 = 2;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_36 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_37 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_38 = ::core::ffi::c_int;
/// A zeroed `save_v_event_T`, which `get_v_event` fills in.
pub(crate) const SAVE_V_EVENT_INIT: save_v_event_T = save_v_event_T {
    sve_did_save: false,
    sve_hashtab: hashtab_T {
        ht_mask: 0,
        ht_used: 0,
        ht_filled: 0,
        ht_changed: 0,
        ht_locked: 0,
        ht_array: ptr::null_mut(),
        ht_smallarray: [hashitem_T {
            hi_hash: 0,
            hi_key: ptr::null_mut(),
        }; 16],
    },
};
/// A zeroed `String_0`, C's `STRING_INIT`.
pub(crate) const STRING_INIT: String_0 = String_0 {
    data: ptr::null_mut(),
    size: 0,
};
/// A zeroed `extmark_undo_vec_t`, which is what C's `kv_destroy` leaves.
pub(crate) const EXTMARK_UNDO_VEC_INIT: extmark_undo_vec_t = extmark_undo_vec_t {
    size: 0,
    capacity: 0,
    items: ptr::null_mut(),
};
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
pub const BS_START: ::core::ffi::c_int = 's' as ::core::ffi::c_int;
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
static compl_leader: GlobalCell<String_0> = GlobalCell::new(STRING_INIT);
static compl_get_longest: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static compl_used_match: GlobalCell<bool> = GlobalCell::new(false);
static compl_was_interrupted: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static compl_interrupted: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static compl_restarting: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static compl_started: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static ctrl_x_mode: GlobalCell<::core::ffi::c_int> = GlobalCell::new(CTRL_X_NORMAL);
static compl_matches: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static compl_pattern: GlobalCell<String_0> = GlobalCell::new(STRING_INIT);
static cpt_compl_pattern: GlobalCell<String_0> = GlobalCell::new(STRING_INIT);
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
static compl_orig_text: GlobalCell<String_0> = GlobalCell::new(STRING_INIT);
static compl_orig_extmarks: GlobalCell<extmark_undo_vec_t> = GlobalCell::new(EXTMARK_UNDO_VEC_INIT);
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
pub const DICT_FIRST: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const DICT_EXACT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
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
pub const CI_WHAT_MODE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CI_WHAT_PUM_VISIBLE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CI_WHAT_ITEMS: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CI_WHAT_SELECTED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CI_WHAT_COMPLETED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CI_WHAT_MATCHES: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const CI_WHAT_PREINSERTED_TEXT: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CI_WHAT_ALL: ::core::ffi::c_int = 0xff as ::core::ffi::c_int;
pub const MIN_SPACE: ::core::ffi::c_int = 75 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
