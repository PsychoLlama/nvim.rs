use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite, ascii_isxdigit};
use crate::src::nvim::charset::{
    getdigits_int, hex2nr, vim_isIDc, vim_isfilec, vim_isprintc, vim_iswordc_buf, vim_iswordp_buf,
};
use crate::src::nvim::eval::typval::{
    tv_clear, tv_get_string_buf_chk, tv_list_alloc, tv_list_append_string, tv_list_init_static10,
};
use crate::src::nvim::eval::typval::{tv_list_first, tv_list_len, tv_list_ref};
use crate::src::nvim::eval::userfunc::call_func;
use crate::src::nvim::eval_1::{eval_to_string, partial_name};
use crate::src::nvim::garray::{ga_append_via_ptr, ga_clear, ga_grow, ga_init, ga_set_growsize};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    VIsual, VIsual_active, VIsual_mode, called_emsg, e_internal_error_in_regexp, e_nopresub,
    e_null, e_re_corr, e_re_damg, e_resulting_text_too_long, e_toomsbra, e_trailing, got_int,
    p_cpo, p_mmp, p_re, p_sel, p_verbose, rc_did_emsg, re_extmatch_in, re_extmatch_out,
    reg_do_extmatch,
};
use crate::src::nvim::main::{curbuf, curwin};
use crate::src::nvim::mark::mark_get;
use crate::src::nvim::mbyte::{
    mb_get_class_tab, mb_islower, mb_isupper, mb_ptr2char_adv, mb_strnicmp, mb_tolower, mb_toupper,
    utf_char2bytes, utf_char2len, utf_composinglike, utf_fold, utf_head_off,
    utf_iscomposing_legacy, utf_ptr2char, utf_ptr2len, utf_strnicmp, utfc_ptr2len,
};
use crate::src::nvim::memline::{ml_get_buf, ml_get_buf_len};
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc, xmemcpyz, xrealloc, xstrdup};
use crate::src::nvim::message::{
    emsg, iemsg, internal_error, msg_puts, semsg, siemsg, verbose_enter, verbose_leave,
};
use crate::src::nvim::os::input::fast_breakcheck;
use crate::src::nvim::os::libc::{
    __assert_fail, __ctype_b_loc, bsearch, gettext, memmove, memset, strcpy, strlen, strncmp,
    strncpy,
};
use crate::src::nvim::plines::getvvcol;
use crate::src::nvim::plines::win_linetabsize;
use crate::src::nvim::pos::lt;
use crate::src::nvim::profile::profile_passed_limit;
use crate::src::nvim::strings::{cmp_keyvalue_value_n, vim_strchr, vim_strsave_escaped, xstrnsave};
pub use crate::src::nvim::types::{
    __compar_fn_t, __time_t, AdditionalData, AlignTextPos, ArgvFunc, BoolVarValue,
    BufUpdateCallbacks, CSType, Callback, Callback_data as C2Rust_Unnamed_8, CallbackType,
    ChangedtickDictItem, CharsizeArg, DecorExt, DecorHighlightInline, DecorInlineData,
    DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_3, ExtmarkUndoObject,
    FileID, FloatAnchor, FloatRelative, GraphemeState, GridView, Intersection, LuaRef, MTKey,
    MTNode, MTPos, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MapHash, MarkGet, MarkTree, MarkTreeIter,
    MarkTreeIter_s as C2Rust_Unnamed_14, OptInt, QUEUE, ScopeDictDictItem, ScopeType, ScreenGrid,
    Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_5, Terminal, Timestamp, VarLockStatus, VarType,
    VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle,
    Window, alist_T, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T, chunksize_T, colnr_T,
    dict_T, dictvar_S, disptick_T, extmark_undo_vec_t, fcs_chars_T, file_buffer, float_T, fmark_T,
    fmarkv_T, frame_S, frame_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T,
    funcexe_T, garray_T, handle_T, hash_T, hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t,
    int64_t, keyvalue_T, lcs_chars_T, linenr_T, list_T, listitem_S, listitem_T, listvar_S,
    listwatch_S, listwatch_T, llpos_T, lpos_T, magic_T, mapblock, mapblock_T, match_T, matchitem,
    matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s, mtnode_s, partial_S, partial_T,
    pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue, reg_extmatch_T,
    regengine, regengine_T, regmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T,
    sctx_T, size_t, ssize_t, staticList10_T, syn_state, syn_state_sst_union as C2Rust_Unnamed_7,
    syn_time_T, synblock_T, synstate_T, taggy_T, terminal, time_t, typval_T, typval_vval_union,
    u_entry, u_entry_T, u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_10,
    u_header_uh_alt_prev as C2Rust_Unnamed_9, u_header_uh_next as C2Rust_Unnamed_12,
    u_header_uh_prev as C2Rust_Unnamed_11, ufunc_S, ufunc_T, uint8_t, uint16_t, uint32_t, uint64_t,
    uintmax_t, undo_object, utf8proc_int32_t, varnumber_T, virt_line, visualinfo_T, win_T,
    window_S, wininfo_S, winopt_T, wline_T, xfmark_T,
};

// The bodies, along the seam upstream's `#include`s left in regexp.c.
// Each child opens with `use super::*`, so the transpiled preamble
// above is its import list.

mod api;
mod bt;
mod chars;
mod context;
mod mbyte;
mod nfa;
mod parse;
mod submatch;
mod substitute;

pub use self::api::*;
pub use self::bt::*;
pub use self::chars::*;
pub use self::context::*;
pub(crate) use self::mbyte::*;
pub(crate) use self::nfa::*;
pub use self::parse::*;
pub use self::submatch::*;
pub use self::substitute::*;
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
pub type C2Rust_Unnamed_0 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_0 = 2147483647;
pub const kVPosWinCol: VirtTextPos = 5;
pub const kVPosRightAlign: VirtTextPos = 4;
pub const kVPosOverlay: VirtTextPos = 3;
pub const kVPosInline: VirtTextPos = 2;
pub const kVPosEndOfLineRightAlign: VirtTextPos = 1;
pub const kVPosEndOfLine: VirtTextPos = 0;
pub const kStlClickFuncRun: C2Rust_Unnamed_5 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_5 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_5 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_5 = 0;
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
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
pub const MF_DIRTY_YES_NOSYNC: mfdirty_T = 2;
pub const MF_DIRTY_YES: mfdirty_T = 1;
pub const MF_DIRTY_NO: mfdirty_T = 0;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const NSUBEXP: C2Rust_Unnamed_15 = 10;
pub const MAGIC_ALL: magic_T = 4;
pub const MAGIC_ON: magic_T = 3;
pub const MAGIC_OFF: magic_T = 2;
pub const MAGIC_NONE: magic_T = 1;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const REGSUB_BACKSLASH: C2Rust_Unnamed_16 = 4;
pub const REGSUB_MAGIC: C2Rust_Unnamed_16 = 2;
pub const REGSUB_COPY: C2Rust_Unnamed_16 = 1;
pub const kMarkAllNoResolve: MarkGet = 2;
pub const kMarkAll: MarkGet = 1;
pub const kMarkBufLocal: MarkGet = 0;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const MB_MAXCHAR: C2Rust_Unnamed_17 = 6;
pub const MB_MAXBYTES: C2Rust_Unnamed_17 = 21;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const kCharsizeFast: C2Rust_Unnamed_18 = 1;
pub const kCharsizeRegular: C2Rust_Unnamed_18 = 0;
pub const CLASS_NONE: C2Rust_Unnamed_26 = 99;
pub const CLASS_XDIGIT: C2Rust_Unnamed_26 = 11;
pub const CLASS_UPPER: C2Rust_Unnamed_26 = 10;
pub const CLASS_TAB: C2Rust_Unnamed_26 = 12;
pub const CLASS_SPACE: C2Rust_Unnamed_26 = 9;
pub const CLASS_RETURN: C2Rust_Unnamed_26 = 13;
pub const CLASS_PUNCT: C2Rust_Unnamed_26 = 8;
pub const CLASS_PRINT: C2Rust_Unnamed_26 = 7;
pub const CLASS_LOWER: C2Rust_Unnamed_26 = 6;
pub const CLASS_KEYWORD: C2Rust_Unnamed_26 = 17;
pub const CLASS_IDENT: C2Rust_Unnamed_26 = 16;
pub const CLASS_GRAPH: C2Rust_Unnamed_26 = 5;
pub const CLASS_FNAME: C2Rust_Unnamed_26 = 18;
pub const CLASS_ESCAPE: C2Rust_Unnamed_26 = 15;
pub const CLASS_DIGIT: C2Rust_Unnamed_26 = 4;
pub const CLASS_CNTRL: C2Rust_Unnamed_26 = 3;
pub const CLASS_BLANK: C2Rust_Unnamed_26 = 2;
pub const CLASS_BACKSPACE: C2Rust_Unnamed_26 = 14;
pub const CLASS_ALPHA: C2Rust_Unnamed_26 = 1;
pub const CLASS_ALNUM: C2Rust_Unnamed_26 = 0;
pub type fptr_T = Option<unsafe extern "C" fn(*mut ::core::ffi::c_int, ::core::ffi::c_int) -> ()>;
pub type reg_getline_flags_T = ::core::ffi::c_uint;
pub const RGLF_SUBMATCH: reg_getline_flags_T = 4;
pub const RGLF_LENGTH: reg_getline_flags_T = 2;
pub const RGLF_LINE: reg_getline_flags_T = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regexec_T {
    pub reg_match: *mut regmatch_T,
    pub reg_mmatch: *mut regmmatch_T,
    pub reg_startp: *mut *mut uint8_t,
    pub reg_endp: *mut *mut uint8_t,
    pub reg_startpos: *mut lpos_T,
    pub reg_endpos: *mut lpos_T,
    pub reg_win: *mut win_T,
    pub reg_buf: *mut buf_T,
    pub reg_firstlnum: linenr_T,
    pub reg_maxline: linenr_T,
    pub reg_line_lbr: bool,
    pub lnum: linenr_T,
    pub line: *mut uint8_t,
    pub input: *mut uint8_t,
    pub need_clear_subexpr: ::core::ffi::c_int,
    pub need_clear_zsubexpr: ::core::ffi::c_int,
    pub reg_ic: bool,
    pub reg_icombine: bool,
    pub reg_nobreak: bool,
    pub reg_maxcol: colnr_T,
    pub nfa_has_zend: ::core::ffi::c_int,
    pub nfa_has_backref: ::core::ffi::c_int,
    pub nfa_nsubexpr: ::core::ffi::c_int,
    pub nfa_listid: ::core::ffi::c_int,
    pub nfa_alt_listid: ::core::ffi::c_int,
    pub nfa_has_zsubexpr: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regsubmatch_T {
    pub sm_match: *mut regmatch_T,
    pub sm_mmatch: *mut regmmatch_T,
    pub sm_firstlnum: linenr_T,
    pub sm_maxline: linenr_T,
    pub sm_line_lbr: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bt_regprog_T {
    pub engine: *mut regengine_T,
    pub regflags: ::core::ffi::c_uint,
    pub re_engine: ::core::ffi::c_uint,
    pub re_flags: ::core::ffi::c_uint,
    pub re_in_use: bool,
    pub regstart: ::core::ffi::c_int,
    pub reganch: uint8_t,
    pub regmust: *mut uint8_t,
    pub regmlen: ::core::ffi::c_int,
    pub reghasz: uint8_t,
    pub program: [uint8_t; 0],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct nfa_regprog_T {
    pub engine: *mut regengine_T,
    pub regflags: ::core::ffi::c_uint,
    pub re_engine: ::core::ffi::c_uint,
    pub re_flags: ::core::ffi::c_uint,
    pub re_in_use: bool,
    pub start: *mut nfa_state_T,
    pub reganch: ::core::ffi::c_int,
    pub regstart: ::core::ffi::c_int,
    pub match_text: *mut uint8_t,
    pub has_zend: ::core::ffi::c_int,
    pub has_backref: ::core::ffi::c_int,
    pub reghasz: ::core::ffi::c_int,
    pub pattern: *mut ::core::ffi::c_char,
    pub nsubexp: ::core::ffi::c_int,
    pub nstate: ::core::ffi::c_int,
    pub state: [nfa_state_T; 0],
}
pub type nfa_state_T = nfa_state;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct nfa_state {
    pub c: ::core::ffi::c_int,
    pub out: *mut nfa_state_T,
    pub out1: *mut nfa_state_T,
    pub id: ::core::ffi::c_int,
    pub lastlist: [::core::ffi::c_int; 2],
    pub val: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct linepos {
    pub start: *mut uint8_t,
    pub end: *mut uint8_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_19 {
    pub multi: [multipos; 10],
    pub line: [linepos; 10],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct multipos {
    pub start_lnum: linenr_T,
    pub end_lnum: linenr_T,
    pub start_col: colnr_T,
    pub end_col: colnr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regsub_T {
    pub in_use: ::core::ffi::c_int,
    pub list: C2Rust_Unnamed_19,
    pub orig_start_col: colnr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regsubs_T {
    pub norm: regsub_T,
    pub synt: regsub_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct nfa_thread_T {
    pub state: *mut nfa_state_T,
    pub count: ::core::ffi::c_int,
    pub pim: nfa_pim_T,
    pub subs: regsubs_T,
}
pub type nfa_pim_T = nfa_pim_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct nfa_pim_S {
    pub result: ::core::ffi::c_int,
    pub state: *mut nfa_state_T,
    pub subs: regsubs_T,
    pub end: C2Rust_Unnamed_20,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_20 {
    pub pos: lpos_T,
    pub ptr: *mut uint8_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct nfa_list_T {
    pub t: *mut nfa_thread_T,
    pub n: ::core::ffi::c_int,
    pub len: ::core::ffi::c_int,
    pub id: ::core::ffi::c_int,
    pub has_pim: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_21 {
    pub ptr: *mut uint8_t,
    pub pos: lpos_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct save_se_T {
    pub se_u: C2Rust_Unnamed_21,
}
pub const NFA_TOO_EXPENSIVE: C2Rust_Unnamed_24 = -1;
pub const NFA_ZCLOSE9: C2Rust_Unnamed_27 = -918;
pub const NFA_ZCLOSE: C2Rust_Unnamed_27 = -927;
pub const NFA_MCLOSE: C2Rust_Unnamed_27 = -947;
pub const NFA_ZEND: C2Rust_Unnamed_27 = -1000;
pub const NFA_ZCLOSE8: C2Rust_Unnamed_27 = -919;
pub const NFA_ZCLOSE7: C2Rust_Unnamed_27 = -920;
pub const NFA_ZCLOSE6: C2Rust_Unnamed_27 = -921;
pub const NFA_ZCLOSE5: C2Rust_Unnamed_27 = -922;
pub const NFA_ZCLOSE4: C2Rust_Unnamed_27 = -923;
pub const NFA_ZCLOSE3: C2Rust_Unnamed_27 = -924;
pub const NFA_ZCLOSE2: C2Rust_Unnamed_27 = -925;
pub const NFA_ZCLOSE1: C2Rust_Unnamed_27 = -926;
pub const NFA_MCLOSE9: C2Rust_Unnamed_27 = -938;
pub const NFA_MCLOSE8: C2Rust_Unnamed_27 = -939;
pub const NFA_MCLOSE7: C2Rust_Unnamed_27 = -940;
pub const NFA_MCLOSE6: C2Rust_Unnamed_27 = -941;
pub const NFA_MCLOSE5: C2Rust_Unnamed_27 = -942;
pub const NFA_MCLOSE4: C2Rust_Unnamed_27 = -943;
pub const NFA_MCLOSE3: C2Rust_Unnamed_27 = -944;
pub const NFA_MCLOSE2: C2Rust_Unnamed_27 = -945;
pub const NFA_MCLOSE1: C2Rust_Unnamed_27 = -946;
pub const NFA_ZOPEN9: C2Rust_Unnamed_27 = -928;
pub const NFA_ZOPEN: C2Rust_Unnamed_27 = -937;
pub const NFA_MOPEN: C2Rust_Unnamed_27 = -957;
pub const NFA_ZSTART: C2Rust_Unnamed_27 = -1001;
pub const NFA_ZOPEN8: C2Rust_Unnamed_27 = -929;
pub const NFA_ZOPEN7: C2Rust_Unnamed_27 = -930;
pub const NFA_ZOPEN6: C2Rust_Unnamed_27 = -931;
pub const NFA_ZOPEN5: C2Rust_Unnamed_27 = -932;
pub const NFA_ZOPEN4: C2Rust_Unnamed_27 = -933;
pub const NFA_ZOPEN3: C2Rust_Unnamed_27 = -934;
pub const NFA_ZOPEN2: C2Rust_Unnamed_27 = -935;
pub const NFA_ZOPEN1: C2Rust_Unnamed_27 = -936;
pub const NFA_MOPEN9: C2Rust_Unnamed_27 = -948;
pub const NFA_MOPEN8: C2Rust_Unnamed_27 = -949;
pub const NFA_MOPEN7: C2Rust_Unnamed_27 = -950;
pub const NFA_MOPEN6: C2Rust_Unnamed_27 = -951;
pub const NFA_MOPEN5: C2Rust_Unnamed_27 = -952;
pub const NFA_MOPEN4: C2Rust_Unnamed_27 = -953;
pub const NFA_MOPEN3: C2Rust_Unnamed_27 = -954;
pub const NFA_MOPEN2: C2Rust_Unnamed_27 = -955;
pub const NFA_MOPEN1: C2Rust_Unnamed_27 = -956;
pub const NFA_NCLOSE: C2Rust_Unnamed_27 = -998;
pub const NFA_NOPEN: C2Rust_Unnamed_27 = -999;
pub const NFA_EMPTY: C2Rust_Unnamed_27 = -1022;
pub const NFA_SPLIT: C2Rust_Unnamed_27 = -1024;
pub const NFA_MATCH: C2Rust_Unnamed_27 = -1023;
pub const NFA_SKIP: C2Rust_Unnamed_27 = -958;
pub const NFA_BOF: C2Rust_Unnamed_27 = -1004;
pub const NFA_BOL: C2Rust_Unnamed_27 = -1008;
pub const NFA_START_INVISIBLE_BEFORE_NEG_FIRST: C2Rust_Unnamed_27 = -990;
pub const NFA_START_INVISIBLE_BEFORE_NEG: C2Rust_Unnamed_27 = -991;
pub const NFA_START_INVISIBLE_NEG_FIRST: C2Rust_Unnamed_27 = -994;
pub const NFA_START_INVISIBLE_NEG: C2Rust_Unnamed_27 = -995;
pub const NFA_START_INVISIBLE_BEFORE_FIRST: C2Rust_Unnamed_27 = -992;
pub const NFA_START_INVISIBLE_BEFORE: C2Rust_Unnamed_27 = -993;
pub const NFA_NEWL: C2Rust_Unnamed_27 = -1002;
pub const NFA_START_NEG_COLL: C2Rust_Unnamed_27 = -1019;
pub const NFA_START_COLL: C2Rust_Unnamed_27 = -1021;
pub const NFA_NUPPER_IC: C2Rust_Unnamed_27 = -887;
pub const NFA_UPPER_IC: C2Rust_Unnamed_27 = -888;
pub const NFA_NLOWER_IC: C2Rust_Unnamed_27 = -889;
pub const NFA_LOWER_IC: C2Rust_Unnamed_27 = -890;
pub const NFA_NUPPER: C2Rust_Unnamed_27 = -891;
pub const NFA_UPPER: C2Rust_Unnamed_27 = -892;
pub const NFA_NLOWER: C2Rust_Unnamed_27 = -893;
pub const NFA_LOWER: C2Rust_Unnamed_27 = -894;
pub const NFA_NALPHA: C2Rust_Unnamed_27 = -895;
pub const NFA_ALPHA: C2Rust_Unnamed_27 = -896;
pub const NFA_NHEAD: C2Rust_Unnamed_27 = -897;
pub const NFA_HEAD: C2Rust_Unnamed_27 = -898;
pub const NFA_NWORD: C2Rust_Unnamed_27 = -899;
pub const NFA_WORD: C2Rust_Unnamed_27 = -900;
pub const NFA_NOCTAL: C2Rust_Unnamed_27 = -901;
pub const NFA_OCTAL: C2Rust_Unnamed_27 = -902;
pub const NFA_NHEX: C2Rust_Unnamed_27 = -903;
pub const NFA_HEX: C2Rust_Unnamed_27 = -904;
pub const NFA_NDIGIT: C2Rust_Unnamed_27 = -905;
pub const NFA_DIGIT: C2Rust_Unnamed_27 = -906;
pub const NFA_NWHITE: C2Rust_Unnamed_27 = -907;
pub const NFA_WHITE: C2Rust_Unnamed_27 = -908;
pub const NFA_SPRINT: C2Rust_Unnamed_27 = -909;
pub const NFA_PRINT: C2Rust_Unnamed_27 = -910;
pub const NFA_SFNAME: C2Rust_Unnamed_27 = -911;
pub const NFA_FNAME: C2Rust_Unnamed_27 = -912;
pub const NFA_SKWORD: C2Rust_Unnamed_27 = -913;
pub const NFA_KWORD: C2Rust_Unnamed_27 = -914;
pub const NFA_SIDENT: C2Rust_Unnamed_27 = -915;
pub const NFA_IDENT: C2Rust_Unnamed_27 = -916;
pub const NFA_ANY_COMPOSING: C2Rust_Unnamed_27 = -983;
pub const NFA_ANY: C2Rust_Unnamed_27 = -917;
pub const NFA_COMPOSING: C2Rust_Unnamed_27 = -985;
pub const NFA_START_INVISIBLE_FIRST: C2Rust_Unnamed_27 = -996;
pub const NFA_START_INVISIBLE: C2Rust_Unnamed_27 = -997;
pub const NFA_END_PATTERN: C2Rust_Unnamed_27 = -986;
pub const NFA_END_INVISIBLE_NEG: C2Rust_Unnamed_27 = -987;
pub const NFA_END_INVISIBLE: C2Rust_Unnamed_27 = -988;
pub const NFA_VISUAL: C2Rust_Unnamed_27 = -842;
pub const NFA_CURSOR: C2Rust_Unnamed_27 = -855;
pub const NFA_MARK_LT: C2Rust_Unnamed_27 = -843;
pub const NFA_MARK_GT: C2Rust_Unnamed_27 = -844;
pub const NFA_MARK: C2Rust_Unnamed_27 = -845;
pub const NFA_VCOL: C2Rust_Unnamed_27 = -848;
pub const NFA_VCOL_LT: C2Rust_Unnamed_27 = -846;
pub const NFA_VCOL_GT: C2Rust_Unnamed_27 = -847;
pub const NFA_COL: C2Rust_Unnamed_27 = -851;
pub const NFA_COL_LT: C2Rust_Unnamed_27 = -849;
pub const NFA_COL_GT: C2Rust_Unnamed_27 = -850;
pub const NFA_LNUM: C2Rust_Unnamed_27 = -854;
pub const NFA_LNUM_LT: C2Rust_Unnamed_27 = -852;
pub const NFA_LNUM_GT: C2Rust_Unnamed_27 = -853;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct decomp_T {
    pub a: ::core::ffi::c_int,
    pub b: ::core::ffi::c_int,
    pub c: ::core::ffi::c_int,
}
pub const NFA_ZREF1: C2Rust_Unnamed_27 = -967;
pub const NFA_BACKREF1: C2Rust_Unnamed_27 = -976;
pub const NFA_BACKREF9: C2Rust_Unnamed_27 = -968;
pub const NFA_ZREF9: C2Rust_Unnamed_27 = -959;
pub const NFA_ZREF8: C2Rust_Unnamed_27 = -960;
pub const NFA_ZREF7: C2Rust_Unnamed_27 = -961;
pub const NFA_ZREF6: C2Rust_Unnamed_27 = -962;
pub const NFA_ZREF5: C2Rust_Unnamed_27 = -963;
pub const NFA_ZREF4: C2Rust_Unnamed_27 = -964;
pub const NFA_ZREF3: C2Rust_Unnamed_27 = -965;
pub const NFA_ZREF2: C2Rust_Unnamed_27 = -966;
pub const NFA_BACKREF8: C2Rust_Unnamed_27 = -969;
pub const NFA_BACKREF7: C2Rust_Unnamed_27 = -970;
pub const NFA_BACKREF6: C2Rust_Unnamed_27 = -971;
pub const NFA_BACKREF5: C2Rust_Unnamed_27 = -972;
pub const NFA_BACKREF4: C2Rust_Unnamed_27 = -973;
pub const NFA_BACKREF3: C2Rust_Unnamed_27 = -974;
pub const NFA_BACKREF2: C2Rust_Unnamed_27 = -975;
pub const NFA_CLASS_FNAME: C2Rust_Unnamed_27 = -823;
pub const NFA_CLASS_KEYWORD: C2Rust_Unnamed_27 = -824;
pub const NFA_CLASS_IDENT: C2Rust_Unnamed_27 = -825;
pub const NFA_CLASS_ESCAPE: C2Rust_Unnamed_27 = -826;
pub const NFA_CLASS_BACKSPACE: C2Rust_Unnamed_27 = -827;
pub const NFA_CLASS_RETURN: C2Rust_Unnamed_27 = -828;
pub const NFA_CLASS_TAB: C2Rust_Unnamed_27 = -829;
pub const NFA_CLASS_XDIGIT: C2Rust_Unnamed_27 = -830;
pub const NFA_CLASS_UPPER: C2Rust_Unnamed_27 = -831;
pub const NFA_CLASS_SPACE: C2Rust_Unnamed_27 = -832;
pub const NFA_CLASS_PUNCT: C2Rust_Unnamed_27 = -833;
pub const NFA_CLASS_PRINT: C2Rust_Unnamed_27 = -834;
pub const NFA_CLASS_LOWER: C2Rust_Unnamed_27 = -835;
pub const NFA_CLASS_GRAPH: C2Rust_Unnamed_27 = -836;
pub const NFA_CLASS_DIGIT: C2Rust_Unnamed_27 = -837;
pub const NFA_CLASS_CNTRL: C2Rust_Unnamed_27 = -838;
pub const NFA_CLASS_BLANK: C2Rust_Unnamed_27 = -839;
pub const NFA_CLASS_ALPHA: C2Rust_Unnamed_27 = -840;
pub const NFA_CLASS_ALNUM: C2Rust_Unnamed_27 = -841;
pub const NFA_RANGE_MIN: C2Rust_Unnamed_27 = -1016;
pub const NFA_END_COLL: C2Rust_Unnamed_27 = -1020;
pub const NFA_END_COMPOSING: C2Rust_Unnamed_27 = -984;
pub const NFA_EOF: C2Rust_Unnamed_27 = -1003;
pub const NFA_EOW: C2Rust_Unnamed_27 = -1005;
pub const NFA_BOW: C2Rust_Unnamed_27 = -1006;
pub const NFA_EOL: C2Rust_Unnamed_27 = -1007;
pub const NFA_START_PATTERN: C2Rust_Unnamed_27 = -989;
pub const NFA_MAX_STATES: C2Rust_Unnamed_24 = 100000;
pub const AUTOMATIC_ENGINE: C2Rust_Unnamed_25 = 0;
pub type Frag_T = Frag;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Frag {
    pub start: *mut nfa_state_T,
    pub out: *mut Ptrlist,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union Ptrlist {
    pub next: *mut Ptrlist,
    pub s: *mut nfa_state_T,
}
pub const NFA_OPT_CHARS: C2Rust_Unnamed_27 = -982;
pub const NFA_PREV_ATOM_JUST_BEFORE_NEG: C2Rust_Unnamed_27 = -978;
pub const NFA_PREV_ATOM_JUST_BEFORE: C2Rust_Unnamed_27 = -979;
pub const NFA_PREV_ATOM_LIKE_PATTERN: C2Rust_Unnamed_27 = -977;
pub const NFA_PREV_ATOM_NO_WIDTH_NEG: C2Rust_Unnamed_27 = -980;
pub const NFA_PREV_ATOM_NO_WIDTH: C2Rust_Unnamed_27 = -981;
pub const NFA_RANGE_MAX: C2Rust_Unnamed_27 = -1015;
pub const NFA_RANGE: C2Rust_Unnamed_27 = -1017;
pub const NFA_END_NEG_COLL: C2Rust_Unnamed_27 = -1018;
pub const NFA_QUEST_NONGREEDY: C2Rust_Unnamed_27 = -1009;
pub const NFA_QUEST: C2Rust_Unnamed_27 = -1010;
pub const NFA_STAR_NONGREEDY: C2Rust_Unnamed_27 = -1011;
pub const NFA_STAR: C2Rust_Unnamed_27 = -1012;
pub const NFA_OR: C2Rust_Unnamed_27 = -1013;
pub const NFA_CONCAT: C2Rust_Unnamed_27 = -1014;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct parse_state_T {
    pub regparse: *mut ::core::ffi::c_char,
    pub prevchr_len: ::core::ffi::c_int,
    pub curchr: ::core::ffi::c_int,
    pub prevchr: ::core::ffi::c_int,
    pub prevprevchr: ::core::ffi::c_int,
    pub nextchr: ::core::ffi::c_int,
    pub at_start: ::core::ffi::c_int,
    pub prev_at_start: ::core::ffi::c_int,
    pub regnpar: ::core::ffi::c_int,
}
pub const NFA_LAST_NL: C2Rust_Unnamed_27 = -856;
pub const NFA_FIRST_NL: C2Rust_Unnamed_27 = -886;
pub type regitem_T = regitem_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regitem_S {
    pub rs_state: regstate_T,
    pub rs_no: int16_t,
    pub rs_scan: *mut uint8_t,
    pub rs_un: C2Rust_Unnamed_22,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_22 {
    pub sesave: save_se_T,
    pub regsave: regsave_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regsave_T {
    pub rs_u: C2Rust_Unnamed_23,
    pub rs_len: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_23 {
    pub ptr: *mut uint8_t,
    pub pos: lpos_T,
}
pub type regstate_T = regstate_E;
pub type regstate_E = ::core::ffi::c_uint;
pub const RS_STAR_SHORT: regstate_E = 13;
pub const RS_STAR_LONG: regstate_E = 12;
pub const RS_BEHIND2: regstate_E = 11;
pub const RS_BEHIND1: regstate_E = 10;
pub const RS_NOMATCH: regstate_E = 9;
pub const RS_BRCPLX_SHORT: regstate_E = 8;
pub const RS_BRCPLX_LONG: regstate_E = 7;
pub const RS_BRCPLX_MORE: regstate_E = 6;
pub const RS_BRANCH: regstate_E = 5;
pub const RS_ZCLOSE: regstate_E = 4;
pub const RS_ZOPEN: regstate_E = 3;
pub const RS_MCLOSE: regstate_E = 2;
pub const RS_MOPEN: regstate_E = 1;
pub const RS_NOPEN: regstate_E = 0;
pub type regstar_T = regstar_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regstar_S {
    pub nextb: ::core::ffi::c_int,
    pub nextb_ic: ::core::ffi::c_int,
    pub count: int64_t,
    pub minval: int64_t,
    pub maxval: int64_t,
}
pub type regbehind_T = regbehind_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regbehind_S {
    pub save_after: regsave_T,
    pub save_behind: regsave_T,
    pub save_need_clear_subexpr: ::core::ffi::c_int,
    pub save_start: [save_se_T; 10],
    pub save_end: [save_se_T; 10],
}
pub type backpos_T = backpos_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct backpos_S {
    pub bp_scan: *mut uint8_t,
    pub bp_pos: regsave_T,
}
pub const BACKTRACKING_ENGINE: C2Rust_Unnamed_25 = 1;
pub const NFA_ENGINE: C2Rust_Unnamed_25 = 2;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_int;
static char_class_tab: GlobalCell<[keyvalue_T; 19]> = GlobalCell::new(
    [keyvalue_T {
        key: 0,
        value: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        length: 0,
    }; 19],
);
pub const INT32_MAX: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const BS: ::core::ffi::c_int = '\u{8}' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
pub const Ctrl_H: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL_0,
};
pub const REGMAGIC: ::core::ffi::c_int = 0o234 as ::core::ffi::c_int;
pub const MAX_LIMIT: ::core::ffi::c_int = (32767 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int;
const E_INVALID_CHARACTER_AFTER_STR_AT: &::core::ffi::CStr = c"E59: Invalid character after %s@";
const E_INVALID_USE_OF_UNDERSCORE: &::core::ffi::CStr = c"E63: Invalid use of \\_";
const E_PATTERN_USES_MORE_MEMORY_THAN_MAXMEMPATTERN: &::core::ffi::CStr =
    c"E363: Pattern uses more memory than 'maxmempattern'";
const E_INVALID_ITEM_IN_STR_BRACKETS: &::core::ffi::CStr = c"E369: Invalid item in %s%%[]";
const E_MISSING_DELIMITER_AFTER_SEARCH_PATTERN_STR: &::core::ffi::CStr =
    c"E654: Missing delimiter after search pattern: %s";
const E_MISSINGBRACKET: &::core::ffi::CStr = c"E769: Missing ] after %s[";
const E_REVERSE_RANGE: &::core::ffi::CStr = c"E944: Reverse range in character class";
const E_LARGE_CLASS: &::core::ffi::CStr = c"E945: Range too large in character class";
const E_UNMATCHEDPP: &::core::ffi::CStr = c"E53: Unmatched %s%%(";
const E_UNMATCHEDP: &::core::ffi::CStr = c"E54: Unmatched %s(";
const E_UNMATCHEDPAR: &::core::ffi::CStr = c"E55: Unmatched %s)";
const E_Z_NOT_ALLOWED: &::core::ffi::CStr = c"E66: \\z( not allowed here";
const E_Z1_NOT_ALLOWED: &::core::ffi::CStr = c"E67: \\z1 - \\z9 not allowed here";
const E_MISSING_SB: &::core::ffi::CStr = c"E69: Missing ] after %s%%[";
const E_EMPTY_SB: &::core::ffi::CStr = c"E70: Empty %s%%[]";
const E_RECURSIVE: &::core::ffi::CStr = c"E956: Cannot use pattern recursively";
const E_REGEXP_NUMBER_AFTER_DOT_POS_SEARCH_CHR: &::core::ffi::CStr =
    c"E1204: No Number allowed after .: '\\%%%c'";
const E_NFA_REGEXP_MISSING_VALUE_IN_CHR: &::core::ffi::CStr =
    c"E1273: (NFA regexp) missing value in '\\%%%c'";
const E_ATOM_ENGINE_MUST_BE_AT_START_OF_PATTERN: &::core::ffi::CStr =
    c"E1281: Atom '\\%%#=%c' must be at the start of the pattern";
const E_SUBSTITUTE_NESTING_TOO_DEEP: &::core::ffi::CStr = c"E1290: substitute nesting too deep";
const E_UNICODE_VAL_TOO_LARGE: &::core::ffi::CStr =
    c"E1541: Value too large, max Unicode codepoint is U+10FFFF";
pub const NOT_MULTI: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const MULTI_ONE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const MULTI_MULT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const RA_FAIL: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RA_CONT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const RA_BREAK: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const RA_MATCH: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const RA_NOMATCH: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
static reg_prev_sub: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static reg_prev_sublen: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
const REGEXP_INRANGE: &::core::ffi::CStr = c"]^-n\\";
const REGEXP_ABBR: &::core::ffi::CStr = c"nrtebdoxuU";
static class_tab: GlobalCell<[int16_t; 256]> = GlobalCell::new([0; 256]);
pub const RI_DIGIT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const RI_HEX: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const RI_OCTAL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const RI_WORD: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const RI_HEAD: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const RI_ALPHA: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const RI_LOWER: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const RI_UPPER: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const RI_WHITE: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const RF_ICASE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RF_NOICASE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const RF_HASNL: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const RF_ICOMBINE: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const RF_LOOKBH: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
static regparse: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static regnpar: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static regnzpar: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static re_has_z: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static regflags: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0);
static had_eol: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static reg_magic: GlobalCell<magic_T> = GlobalCell::new(0 as magic_T);
static reg_string: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static reg_strict: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static META_flags: GlobalCell<[uint8_t; 127]> = GlobalCell::new([
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
    1 as uint8_t,
    0 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    1 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    0 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    0 as uint8_t,
    1 as uint8_t,
    0 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    0 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    0 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    1 as uint8_t,
    0 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    0 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    1 as uint8_t,
    0 as uint8_t,
    1 as uint8_t,
    0 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    0 as uint8_t,
    1 as uint8_t,
    0 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    0 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    1 as uint8_t,
    0 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    0 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    0 as uint8_t,
    1 as uint8_t,
]);
static curchr: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static prevchr: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static prevprevchr: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static nextchr: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub const REG_NOPAREN: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const REG_PAREN: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const REG_ZPAREN: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const REG_NPAREN: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
static reg_cpo_lit: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static at_start: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static prev_at_start: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static reg_tofree: GlobalCell<*mut uint8_t> = GlobalCell::new(::core::ptr::null_mut::<uint8_t>());
static reg_tofreelen: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0);
static rex: GlobalCell<regexec_T> = GlobalCell::new(regexec_T {
    reg_match: ::core::ptr::null_mut::<regmatch_T>(),
    reg_mmatch: ::core::ptr::null_mut::<regmmatch_T>(),
    reg_startp: ::core::ptr::null_mut::<*mut uint8_t>(),
    reg_endp: ::core::ptr::null_mut::<*mut uint8_t>(),
    reg_startpos: ::core::ptr::null_mut::<lpos_T>(),
    reg_endpos: ::core::ptr::null_mut::<lpos_T>(),
    reg_win: ::core::ptr::null_mut::<win_T>(),
    reg_buf: ::core::ptr::null_mut::<buf_T>(),
    reg_firstlnum: 0,
    reg_maxline: 0,
    reg_line_lbr: false,
    lnum: 0,
    line: ::core::ptr::null_mut::<uint8_t>(),
    input: ::core::ptr::null_mut::<uint8_t>(),
    need_clear_subexpr: 0,
    need_clear_zsubexpr: 0,
    reg_ic: false,
    reg_icombine: false,
    reg_nobreak: false,
    reg_maxcol: 0,
    nfa_has_zend: 0,
    nfa_has_backref: 0,
    nfa_nsubexpr: 0,
    nfa_listid: 0,
    nfa_alt_listid: 0,
    nfa_has_zsubexpr: 0,
});
static rex_in_use: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static can_f_submatch: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static rsm: GlobalCell<regsubmatch_T> = GlobalCell::new(regsubmatch_T {
    sm_match: ::core::ptr::null_mut::<regmatch_T>(),
    sm_mmatch: ::core::ptr::null_mut::<regmmatch_T>(),
    sm_firstlnum: 0,
    sm_maxline: 0,
    sm_line_lbr: 0,
});
static reg_startzp: GlobalCell<[*mut uint8_t; 10]> =
    GlobalCell::new([::core::ptr::null_mut::<uint8_t>(); 10]);
static reg_endzp: GlobalCell<[*mut uint8_t; 10]> =
    GlobalCell::new([::core::ptr::null_mut::<uint8_t>(); 10]);
static reg_startzpos: GlobalCell<[lpos_T; 10]> = GlobalCell::new([lpos_T { lnum: 0, col: 0 }; 10]);
static reg_endzpos: GlobalCell<[lpos_T; 10]> = GlobalCell::new([lpos_T { lnum: 0, col: 0 }; 10]);
static decomp_table: GlobalCell<[decomp_T; 48]> = GlobalCell::new([
    decomp_T {
        a: 0x5e2 as ::core::ffi::c_int,
        b: 0 as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5d0 as ::core::ffi::c_int,
        b: 0 as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5d3 as ::core::ffi::c_int,
        b: 0 as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5d4 as ::core::ffi::c_int,
        b: 0 as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5db as ::core::ffi::c_int,
        b: 0 as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5dc as ::core::ffi::c_int,
        b: 0 as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5dd as ::core::ffi::c_int,
        b: 0 as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5e8 as ::core::ffi::c_int,
        b: 0 as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5ea as ::core::ffi::c_int,
        b: 0 as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: '+' as ::core::ffi::c_int,
        b: 0 as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5e9 as ::core::ffi::c_int,
        b: 0x5c1 as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5e9 as ::core::ffi::c_int,
        b: 0x5c2 as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5e9 as ::core::ffi::c_int,
        b: 0x5c1 as ::core::ffi::c_int,
        c: 0x5bc as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5e9 as ::core::ffi::c_int,
        b: 0x5c2 as ::core::ffi::c_int,
        c: 0x5bc as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5d0 as ::core::ffi::c_int,
        b: 0x5b7 as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5d0 as ::core::ffi::c_int,
        b: 0x5b8 as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5d0 as ::core::ffi::c_int,
        b: 0x5b4 as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5d1 as ::core::ffi::c_int,
        b: 0x5bc as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5d2 as ::core::ffi::c_int,
        b: 0x5bc as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5d3 as ::core::ffi::c_int,
        b: 0x5bc as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5d4 as ::core::ffi::c_int,
        b: 0x5bc as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5d5 as ::core::ffi::c_int,
        b: 0x5bc as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5d6 as ::core::ffi::c_int,
        b: 0x5bc as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0xfb37 as ::core::ffi::c_int,
        b: 0 as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5d8 as ::core::ffi::c_int,
        b: 0x5bc as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5d9 as ::core::ffi::c_int,
        b: 0x5bc as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5da as ::core::ffi::c_int,
        b: 0x5bc as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5db as ::core::ffi::c_int,
        b: 0x5bc as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5dc as ::core::ffi::c_int,
        b: 0x5bc as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0xfb3d as ::core::ffi::c_int,
        b: 0 as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5de as ::core::ffi::c_int,
        b: 0x5bc as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0xfb3f as ::core::ffi::c_int,
        b: 0 as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5e0 as ::core::ffi::c_int,
        b: 0x5bc as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5e1 as ::core::ffi::c_int,
        b: 0x5bc as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0xfb42 as ::core::ffi::c_int,
        b: 0 as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5e3 as ::core::ffi::c_int,
        b: 0x5bc as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5e4 as ::core::ffi::c_int,
        b: 0x5bc as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0xfb45 as ::core::ffi::c_int,
        b: 0 as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5e6 as ::core::ffi::c_int,
        b: 0x5bc as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5e7 as ::core::ffi::c_int,
        b: 0x5bc as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5e8 as ::core::ffi::c_int,
        b: 0x5bc as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5e9 as ::core::ffi::c_int,
        b: 0x5bc as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5ea as ::core::ffi::c_int,
        b: 0x5bc as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5d5 as ::core::ffi::c_int,
        b: 0x5b9 as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5d1 as ::core::ffi::c_int,
        b: 0x5bf as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5db as ::core::ffi::c_int,
        b: 0x5bf as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5e4 as ::core::ffi::c_int,
        b: 0x5bf as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
    decomp_T {
        a: 0x5d0 as ::core::ffi::c_int,
        b: 0x5dc as ::core::ffi::c_int,
        c: 0 as ::core::ffi::c_int,
    },
]);
pub const MAX_REGSUB_NESTING: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
static eval_result: GlobalCell<[*mut ::core::ffi::c_char; 4]> = GlobalCell::new([
    ::core::ptr::null_mut::<::core::ffi::c_char>(),
    ::core::ptr::null_mut::<::core::ffi::c_char>(),
    ::core::ptr::null_mut::<::core::ffi::c_char>(),
    ::core::ptr::null_mut::<::core::ffi::c_char>(),
]);
pub const END: ::core::ffi::c_int = 0;
pub const BOL: ::core::ffi::c_int = 1;
pub const EOL: ::core::ffi::c_int = 2;
pub const BRANCH: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const BACK: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const EXACTLY: ::core::ffi::c_int = 5;
pub const NOTHING: ::core::ffi::c_int = 6;
pub const STAR: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const PLUS: ::core::ffi::c_int = 8;
pub const MATCH: ::core::ffi::c_int = 9;
pub const NOMATCH: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const BEHIND: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const NOBEHIND: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const SUBPAT: ::core::ffi::c_int = 13 as ::core::ffi::c_int;
pub const BRACE_SIMPLE: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
pub const BOW: ::core::ffi::c_int = 15;
pub const EOW: ::core::ffi::c_int = 16;
pub const BRACE_LIMITS: ::core::ffi::c_int = 17;
pub const NEWL: ::core::ffi::c_int = 18;
pub const BHPOS: ::core::ffi::c_int = 19;
pub const ADD_NL: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
pub const FIRST_NL: ::core::ffi::c_int = ANY + ADD_NL;
pub const ANY: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const ANYOF: ::core::ffi::c_int = 21 as ::core::ffi::c_int;
pub const ANYBUT: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const IDENT: ::core::ffi::c_int = 23 as ::core::ffi::c_int;
pub const SIDENT: ::core::ffi::c_int = 24 as ::core::ffi::c_int;
pub const KWORD: ::core::ffi::c_int = 25 as ::core::ffi::c_int;
pub const SKWORD: ::core::ffi::c_int = 26 as ::core::ffi::c_int;
pub const FNAME: ::core::ffi::c_int = 27 as ::core::ffi::c_int;
pub const SFNAME: ::core::ffi::c_int = 28 as ::core::ffi::c_int;
pub const PRINT: ::core::ffi::c_int = 29 as ::core::ffi::c_int;
pub const SPRINT: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
pub const WHITE: ::core::ffi::c_int = 31 as ::core::ffi::c_int;
pub const NWHITE: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const DIGIT: ::core::ffi::c_int = 33 as ::core::ffi::c_int;
pub const NDIGIT: ::core::ffi::c_int = 34 as ::core::ffi::c_int;
pub const HEX: ::core::ffi::c_int = 35 as ::core::ffi::c_int;
pub const NHEX: ::core::ffi::c_int = 36 as ::core::ffi::c_int;
pub const OCTAL: ::core::ffi::c_int = 37 as ::core::ffi::c_int;
pub const NOCTAL: ::core::ffi::c_int = 38 as ::core::ffi::c_int;
pub const WORD: ::core::ffi::c_int = 39 as ::core::ffi::c_int;
pub const NWORD: ::core::ffi::c_int = 40 as ::core::ffi::c_int;
pub const HEAD: ::core::ffi::c_int = 41 as ::core::ffi::c_int;
pub const NHEAD: ::core::ffi::c_int = 42 as ::core::ffi::c_int;
pub const ALPHA: ::core::ffi::c_int = 43 as ::core::ffi::c_int;
pub const NALPHA: ::core::ffi::c_int = 44 as ::core::ffi::c_int;
pub const LOWER: ::core::ffi::c_int = 45 as ::core::ffi::c_int;
pub const NLOWER: ::core::ffi::c_int = 46 as ::core::ffi::c_int;
pub const UPPER: ::core::ffi::c_int = 47 as ::core::ffi::c_int;
pub const NUPPER: ::core::ffi::c_int = 48 as ::core::ffi::c_int;
pub const LAST_NL: ::core::ffi::c_int = NUPPER + ADD_NL;
pub const MOPEN: ::core::ffi::c_int = 80 as ::core::ffi::c_int;
pub const MCLOSE: ::core::ffi::c_int = 90 as ::core::ffi::c_int;
pub const BACKREF: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
pub const ZOPEN: ::core::ffi::c_int = 110 as ::core::ffi::c_int;
pub const ZCLOSE: ::core::ffi::c_int = 120 as ::core::ffi::c_int;
pub const ZREF: ::core::ffi::c_int = 130 as ::core::ffi::c_int;
pub const BRACE_COMPLEX: ::core::ffi::c_int = 140 as ::core::ffi::c_int;
pub const NOPEN: ::core::ffi::c_int = 150;
pub const NCLOSE: ::core::ffi::c_int = 151;
pub const MULTIBYTECODE: ::core::ffi::c_int = 200;
pub const RE_BOF: ::core::ffi::c_int = 201;
pub const RE_EOF: ::core::ffi::c_int = 202;
pub const CURSOR: ::core::ffi::c_int = 203;
pub const RE_LNUM: ::core::ffi::c_int = 204;
pub const RE_COL: ::core::ffi::c_int = 205;
pub const RE_VCOL: ::core::ffi::c_int = 206;
pub const RE_MARK: ::core::ffi::c_int = 207;
pub const RE_VISUAL: ::core::ffi::c_int = 208;
pub const RE_COMPOSING: ::core::ffi::c_int = 209;
pub const HASWIDTH: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const SIMPLE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const SPSTART: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const HASNL: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const HASLOOKBH: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const WORST: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static prevchr_len: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static num_complex_braces: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static regcode: GlobalCell<*mut uint8_t> = GlobalCell::new(::core::ptr::null_mut::<uint8_t>());
static regsize: GlobalCell<int64_t> = GlobalCell::new(0);
static reg_toolong: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static had_endbrace: GlobalCell<[uint8_t; 10]> = GlobalCell::new([0; 10]);
static brace_min: GlobalCell<[int64_t; 10]> = GlobalCell::new([0; 10]);
static brace_max: GlobalCell<[int64_t; 10]> = GlobalCell::new([0; 10]);
static brace_count: GlobalCell<[::core::ffi::c_int; 10]> = GlobalCell::new([0; 10]);
static one_exactly: GlobalCell<::core::ffi::c_int> = GlobalCell::new(false_0);
static classchars: GlobalCell<*mut uint8_t> = GlobalCell::new(
    b".iIkKfFpPsSdDxXoOwWhHaAlLuU\0".as_ptr() as *const ::core::ffi::c_char as *mut uint8_t,
);
static classcodes: GlobalCell<[::core::ffi::c_int; 27]> = GlobalCell::new([
    ANY, IDENT, SIDENT, KWORD, SKWORD, FNAME, SFNAME, PRINT, SPRINT, WHITE, NWHITE, DIGIT, NDIGIT,
    HEX, NHEX, OCTAL, NOCTAL, WORD, NWORD, HEAD, NHEAD, ALPHA, NALPHA, LOWER, NLOWER, UPPER,
    NUPPER,
]);
pub const JUST_CALC_SIZE: *mut uint8_t = -1 as ::core::ffi::c_int as *mut uint8_t;
static regstack: GlobalCell<garray_T> = GlobalCell::new(GA_EMPTY_INIT_VALUE);
static backpos: GlobalCell<garray_T> = GlobalCell::new(GA_EMPTY_INIT_VALUE);
static behind_pos: GlobalCell<regsave_T> = GlobalCell::new(regsave_T {
    rs_u: C2Rust_Unnamed_23 {
        ptr: ::core::ptr::null_mut::<uint8_t>(),
    },
    rs_len: 0,
});
pub const REGSTACK_INITIAL: ::core::ffi::c_int = 2048 as ::core::ffi::c_int;
pub const BACKPOS_INITIAL: ::core::ffi::c_int = 64 as ::core::ffi::c_int;
static bl_minval: GlobalCell<int64_t> = GlobalCell::new(0);
static bl_maxval: GlobalCell<int64_t> = GlobalCell::new(0);
pub const NFA_ADD_NL: ::core::ffi::c_int = 31 as ::core::ffi::c_int;
static nfa_classcodes: GlobalCell<[::core::ffi::c_int; 27]> = GlobalCell::new([
    NFA_ANY as ::core::ffi::c_int,
    NFA_IDENT as ::core::ffi::c_int,
    NFA_SIDENT as ::core::ffi::c_int,
    NFA_KWORD as ::core::ffi::c_int,
    NFA_SKWORD as ::core::ffi::c_int,
    NFA_FNAME as ::core::ffi::c_int,
    NFA_SFNAME as ::core::ffi::c_int,
    NFA_PRINT as ::core::ffi::c_int,
    NFA_SPRINT as ::core::ffi::c_int,
    NFA_WHITE as ::core::ffi::c_int,
    NFA_NWHITE as ::core::ffi::c_int,
    NFA_DIGIT as ::core::ffi::c_int,
    NFA_NDIGIT as ::core::ffi::c_int,
    NFA_HEX as ::core::ffi::c_int,
    NFA_NHEX as ::core::ffi::c_int,
    NFA_OCTAL as ::core::ffi::c_int,
    NFA_NOCTAL as ::core::ffi::c_int,
    NFA_WORD as ::core::ffi::c_int,
    NFA_NWORD as ::core::ffi::c_int,
    NFA_HEAD as ::core::ffi::c_int,
    NFA_NHEAD as ::core::ffi::c_int,
    NFA_ALPHA as ::core::ffi::c_int,
    NFA_NALPHA as ::core::ffi::c_int,
    NFA_LOWER as ::core::ffi::c_int,
    NFA_NLOWER as ::core::ffi::c_int,
    NFA_UPPER as ::core::ffi::c_int,
    NFA_NUPPER as ::core::ffi::c_int,
]);
const E_NUL_FOUND: &::core::ffi::CStr = c"E865: (NFA) Regexp end encountered prematurely";
const E_MISPLACED: &::core::ffi::CStr = c"E866: (NFA regexp) Misplaced %c";
const E_ILL_CHAR_CLASS: &::core::ffi::CStr = c"E877: (NFA regexp) Invalid character class: %ld";
const E_VALUE_TOO_LARGE: &::core::ffi::CStr = c"E951: \\% value too large";
static nfa_re_flags: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static post_start: GlobalCell<*mut ::core::ffi::c_int> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_int>());
static post_end: GlobalCell<*mut ::core::ffi::c_int> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_int>());
static post_ptr: GlobalCell<*mut ::core::ffi::c_int> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_int>());
static wants_nfa: GlobalCell<bool> = GlobalCell::new(false);
static nstate: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static istate: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static nfa_endp: GlobalCell<*mut save_se_T> = GlobalCell::new(::core::ptr::null_mut::<save_se_T>());
static nfa_ll_index: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub const CLASS_not: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const CLASS_af: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CLASS_AF: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const CLASS_az: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CLASS_AZ: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CLASS_o7: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CLASS_o9: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CLASS_underscore: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const A_grave: ::core::ffi::c_int = 192;
pub const A_acute: ::core::ffi::c_int = 193;
pub const A_circumflex: ::core::ffi::c_int = 194;
pub const A_virguilla: ::core::ffi::c_int = 195;
pub const A_diaeresis: ::core::ffi::c_int = 196;
pub const A_ring: ::core::ffi::c_int = 197;
pub const C_cedilla: ::core::ffi::c_int = 199;
pub const E_grave: ::core::ffi::c_int = 200;
pub const E_acute: ::core::ffi::c_int = 201;
pub const E_circumflex: ::core::ffi::c_int = 202;
pub const E_diaeresis: ::core::ffi::c_int = 203;
pub const I_grave: ::core::ffi::c_int = 204;
pub const I_acute: ::core::ffi::c_int = 205;
pub const I_circumflex: ::core::ffi::c_int = 206;
pub const I_diaeresis: ::core::ffi::c_int = 207;
pub const N_virguilla: ::core::ffi::c_int = 209;
pub const O_grave: ::core::ffi::c_int = 210;
pub const O_acute: ::core::ffi::c_int = 211;
pub const O_circumflex: ::core::ffi::c_int = 212;
pub const O_virguilla: ::core::ffi::c_int = 213;
pub const O_diaeresis: ::core::ffi::c_int = 214;
pub const O_slash: ::core::ffi::c_int = 216;
pub const U_grave: ::core::ffi::c_int = 217;
pub const U_acute: ::core::ffi::c_int = 218;
pub const U_circumflex: ::core::ffi::c_int = 219;
pub const U_diaeresis: ::core::ffi::c_int = 220;
pub const Y_acute: ::core::ffi::c_int = 221;
pub const a_grave: ::core::ffi::c_int = 224;
pub const a_acute: ::core::ffi::c_int = 225;
pub const a_circumflex: ::core::ffi::c_int = 226;
pub const a_virguilla: ::core::ffi::c_int = 227;
pub const a_diaeresis: ::core::ffi::c_int = 228;
pub const a_ring: ::core::ffi::c_int = 229;
pub const c_cedilla: ::core::ffi::c_int = 231;
pub const e_grave: ::core::ffi::c_int = 232;
pub const e_acute: ::core::ffi::c_int = 233;
pub const e_circumflex: ::core::ffi::c_int = 234;
pub const e_diaeresis: ::core::ffi::c_int = 235;
pub const i_grave: ::core::ffi::c_int = 236;
pub const i_acute: ::core::ffi::c_int = 237;
pub const i_circumflex: ::core::ffi::c_int = 238;
pub const i_diaeresis: ::core::ffi::c_int = 239;
pub const n_virguilla: ::core::ffi::c_int = 241;
pub const o_grave: ::core::ffi::c_int = 242;
pub const o_acute: ::core::ffi::c_int = 243;
pub const o_circumflex: ::core::ffi::c_int = 244;
pub const o_virguilla: ::core::ffi::c_int = 245;
pub const o_diaeresis: ::core::ffi::c_int = 246;
pub const o_slash: ::core::ffi::c_int = 248;
pub const u_grave: ::core::ffi::c_int = 249;
pub const u_acute: ::core::ffi::c_int = 250;
pub const u_circumflex: ::core::ffi::c_int = 251;
pub const u_diaeresis: ::core::ffi::c_int = 252;
pub const y_acute: ::core::ffi::c_int = 253;
pub const y_diaeresis: ::core::ffi::c_int = 255;
static state_ptr: GlobalCell<*mut nfa_state_T> =
    GlobalCell::new(::core::ptr::null_mut::<nfa_state_T>());
static empty: GlobalCell<Frag_T> = GlobalCell::new(Frag_T {
    start: ::core::ptr::null_mut::<nfa_state_T>(),
    out: ::core::ptr::null_mut::<Ptrlist>(),
});
pub const NFA_PIM_UNUSED: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NFA_PIM_TODO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const NFA_PIM_MATCH: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const NFA_PIM_NOMATCH: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
static nfa_match: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static nfa_time_limit: GlobalCell<*mut proftime_T> =
    GlobalCell::new(::core::ptr::null_mut::<proftime_T>());
static nfa_timed_out: GlobalCell<*mut ::core::ffi::c_int> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_int>());
static nfa_time_count: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub const ADDSTATE_HERE_OFFSET: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
static bt_regengine: GlobalCell<regengine_T> = GlobalCell::new(regengine {
    regcomp: Some(bt_regcomp),
    regfree: Some(bt_regfree),
    regexec_nl: Some(bt_regexec_nl),
    regexec_multi: Some(bt_regexec_multi),
});
static nfa_regengine: GlobalCell<regengine_T> = GlobalCell::new(regengine {
    regcomp: Some(nfa_regcomp),
    regfree: Some(nfa_regfree),
    regexec_nl: Some(nfa_regexec_nl),
    regexec_multi: Some(nfa_regexec_multi),
});
static regexp_engine: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FUNCEXE_INIT: funcexe_T = funcexe_T {
    fe_argv_func: None,
    fe_firstline: 0 as linenr_T,
    fe_lastline: 0 as linenr_T,
    fe_doesrange: ::core::ptr::null_mut::<bool>(),
    fe_evaluate: false_0 != 0,
    fe_partial: ::core::ptr::null_mut::<partial_T>(),
    fe_selfdict: ::core::ptr::null_mut::<dict_T>(),
    fe_basetv: ::core::ptr::null_mut::<typval_T>(),
    fe_found_var: false_0 != 0,
};
pub const K_SPECIAL: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const GRAPHEME_STATE_INIT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const MAX_MCO: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const CPO_LITERAL: ::core::ffi::c_int = 'l' as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RE_STRING: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const RE_STRICT: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const RE_AUTO: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const RE_NOBREAK: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const REX_SET: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const REX_USE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const REX_ALL: ::core::ffi::c_int = REX_SET | REX_USE;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
unsafe extern "C" fn c2rust_run_static_initializers() {
    char_class_tab.set([
        keyvalue_T {
            key: CLASS_ALNUM as ::core::ffi::c_int,
            value: b"alnum:]\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: CLASS_ALPHA as ::core::ffi::c_int,
            value: b"alpha:]\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: CLASS_BACKSPACE as ::core::ffi::c_int,
            value: b"backspace:]\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: CLASS_BLANK as ::core::ffi::c_int,
            value: b"blank:]\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: CLASS_CNTRL as ::core::ffi::c_int,
            value: b"cntrl:]\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: CLASS_DIGIT as ::core::ffi::c_int,
            value: b"digit:]\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: CLASS_ESCAPE as ::core::ffi::c_int,
            value: b"escape:]\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: CLASS_FNAME as ::core::ffi::c_int,
            value: b"fname:]\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: CLASS_GRAPH as ::core::ffi::c_int,
            value: b"graph:]\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: CLASS_IDENT as ::core::ffi::c_int,
            value: b"ident:]\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: CLASS_KEYWORD as ::core::ffi::c_int,
            value: b"keyword:]\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: CLASS_LOWER as ::core::ffi::c_int,
            value: b"lower:]\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: CLASS_PRINT as ::core::ffi::c_int,
            value: b"print:]\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: CLASS_PUNCT as ::core::ffi::c_int,
            value: b"punct:]\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: CLASS_RETURN as ::core::ffi::c_int,
            value: b"return:]\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: CLASS_SPACE as ::core::ffi::c_int,
            value: b"space:]\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: CLASS_TAB as ::core::ffi::c_int,
            value: b"tab:]\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: CLASS_UPPER as ::core::ffi::c_int,
            value: b"upper:]\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: CLASS_XDIGIT as ::core::ffi::c_int,
            value: b"xdigit:]\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        },
    ]);
}
#[used]
#[cfg_attr(target_os = "linux", unsafe(link_section = ".init_array"))]
#[cfg_attr(target_os = "windows", unsafe(link_section = ".CRT$XIB"))]
#[cfg_attr(target_os = "macos", unsafe(link_section = "__DATA,__mod_init_func"))]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [c2rust_run_static_initializers];
