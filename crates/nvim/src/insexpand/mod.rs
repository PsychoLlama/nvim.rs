#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::eval::typval::TV_INITIAL_VALUE;
use crate::memory::XString;
use crate::types::AutoEvent;
use crate::types::CAR;
use crate::types::NL;
use crate::types::TAB;
use core::ffi::{CStr, c_char, c_int, c_uint, c_void};
use core::ptr;

use crate::api::private::helpers::{cbuf_to_string, cstr_to_string};
use crate::ascii::{ascii_isdigit, ascii_iswhite, ascii_iswhite_or_nul};
use crate::autocmd::{apply_autocmds, has_event};
use crate::buffer::buf_spname;
use crate::change::{
    deleted_lines_mark, ins_bytes_len, ins_char, ins_char_bytes, ins_str, open_line,
};
use crate::charset::{
    ptr2cells, skipwhite, str_foldcase, vim_is_ident_char, vim_isfilec, vim_isprintc, vim_iswordc,
    vim_iswordp, vim_strsize,
};
use crate::cmdexpand::{addstar, expand_cmdline, set_cmd_context};
use crate::cursor::{
    check_cursor, dec_cursor, get_cursor_line_len, get_cursor_line_ptr, get_cursor_pos_len,
    get_cursor_pos_ptr, inc_cursor,
};
use crate::drawscreen::state::{dollar_vcol, redraw_cmdline, redraw_mode, sc_col};
use crate::drawscreen::{
    UPD_VALID, redraw_later, redraw_win_line, setcursor, showmode, update_screen,
};
use crate::edit::{
    backspace_until_column, get_can_cindent, ins_apply_autocmds, ins_eol, ins_need_undo_get,
    ins_redraw, insertchar, start_arrow, stop_arrow,
};
use crate::eval::typval::{
    callback_copy, callback_free, dict_find, dict_get_number, dict_get_tv, list_unref, tv_clear,
    tv_dict_alloc, tv_dict_alloc_lock, tv_dict_alloc_ret, tv_dict_unref, tv_get_number_chk,
    tv_list_alloc,
};
use crate::eval::userfunc::callback_call_retnr;
use crate::eval::vars::set_vim_var_dict;
use crate::eval::{callback_call, get_v_event, restore_v_event, set_ref_in_callback};
use crate::ex_docmd::state::{ex_normal_busy, global_busy};
use crate::ex_eval::aborting;
use crate::ex_getln::tilde_replace;
use crate::extmark::{extmark_apply_undo, extmark_splice_delete};
use crate::fileio::vim_fgets;
use crate::fuzzy::fuzzy_match_str;
use crate::getchar::state::{KeyTyped, got_int, test_disable_char_avail};
use crate::getchar::{
    append_to_redobuff_char, append_to_redobuff_literally, char_avail, safe_vgetc, using_script,
    vgetc, vpeekc, vpeekc_any, vungetc,
};
use crate::global_cell::{GlobalCell, state_record};
use crate::highlight_group::{HLF_COUNT, HLF_E, HLF_R, HLF_W, syn_name2attr};
use crate::indent::{get_indent, inindent};
use crate::indent_c::{cindent_on, do_c_expr_indent, in_cinkeys};
use crate::lua::executor::nlua_expand_pat;
use crate::mbyte::{
    mb_get_class, mb_islower, mb_isupper, mb_prevptr, mb_ptr2char_adv, mb_tolower, mb_toupper,
    utf_char2bytes, utf_char2len, utf_head_off, utf_ptr2char, utf_ptr2len, utf8len_tab,
    utfc_ptr2len,
};
use crate::memline::{dec, ml_delete, ml_get_buf, ml_get_buf_len};
use crate::memory::{
    MergeSortCompareFunc, MergeSortGetFunc, MergeSortSetFunc, mergesort_list, strequal, xcalloc,
    xfree, xmalloc, xmemdupz, xstrdup, xstrlcpy,
};
use crate::message::state::{did_emsg, emsg_silent, in_assert_fails, msg_hist_off};
use crate::message::{e_invarg, e_listreq, e_patnotf};
use crate::message::{
    emsg, internal_error, msg_clr_cmdline, msg_delay, msg_ext_set_kind, msg_progress,
};
use crate::r#move::{changed_cline_bef_curs, curs_columns, validate_cursor};
use crate::option::vars::{
    P_IC, P_SCS, P_WS, cot_flags, p_ac, p_acl, p_act, p_cto, p_dict, p_fic, p_ic, p_inf, p_js,
    p_paste, p_scs, p_smd, p_tsr, p_tsrfu, p_wic, p_ws,
};
use crate::option::{can_bs, copy_option_part, magic_isset, option_set_callback_func, shortmess};
use crate::options::{
    kOptBoFlagComplete, kOptCotFlagFuzzy, kOptCotFlagLongest, kOptCotFlagMenu, kOptCotFlagMenuone,
    kOptCotFlagNearest, kOptCotFlagNoinsert, kOptCotFlagNoselect, kOptCotFlagNosort,
    kOptCotFlagPreinsert,
};
use crate::os::cshim::{gettext, strncasecmp};
use crate::os::fs::os_fopen;
use crate::os::input::{fast_breakcheck, line_breakcheck, os_breakcheck};
use crate::os::time::{os_delay, os_hrtime};
use crate::path::{expand_wildcards, free_wild, path_tail, vim_ispathsep};
use crate::popupmenu::state::pum_want;
use crate::popupmenu::{
    pum_clear, pum_display, pum_get_height, pum_set_event_info, pum_undisplay, pum_visible,
};
use crate::pos::{MAXCOL, MAXLNUM, equalpos};
use crate::regexp::{RE_LAST, RE_MAGIC, vim_regcomp, vim_regexec, vim_regfree};
use crate::register::{copy_register, free_register, get_register_name, valid_yank_reg};
use crate::search::{
    BACKWARD, FORWARD, SEARCH_KEEP, SEARCH_NFMSG, find_pattern_in_path, ignorecase,
    search_for_exact_line, searchit,
};
use crate::spell::{
    SMT_ALL, expand_spelling, spell_dump_compl, spell_expand_check_cap, spell_move_to,
    spell_word_start,
};
use crate::state::mode::{
    State, arrow_used, can_si, can_si_back, did_ai, did_si, edit_submode, edit_submode_extra,
    edit_submode_highl, edit_submode_pre,
};
use crate::state::{MODE_INSERT, REPLACE_FLAG, may_trigger_modechanged};
use crate::strings::vim_strsave_escaped;
use crate::tag::find_tags;
use crate::tag::state::g_tag_at_cursor;
use crate::textformat::auto_format;
use crate::types::{
    BoolVarValue, Callback, ColNr, Dict, Direction, EvalFuncData, Expand, ExtmarkOp, HashTab,
    LineNr, List, MB_MAXCHAR, OptInt, OptSet, Pos, PumItem, RegMatch, SaveVEvent, String_0, TypVal,
    VarNumber, Vv, XpPrefix, extmark_undo_vec_t, ptrdiff_t, size_t, uint8_t, uint64_t,
};
use crate::ui::{ui_flush, vim_beep};
use crate::undo::undo_allowed;
use crate::window::win_valid;
use crate::winfloat::win_float_find_preview;
use crate::winlayer::graph::cmdwin_type;
use crate::winlayer::{BufId, WinId};
use ::libc::{atoi, fclose, strncpy, strrchr};

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
#[cfg(test)]
mod tests;
pub const kDirectionNotSet: Direction = 0;
pub const XP_PREFIX_NONE: XpPrefix = 0;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const OPENLINE_FORCE_INDENT: ::core::ffi::c_int = 64;
pub const OPENLINE_KEEPTRAIL: ::core::ffi::c_int = 4;
pub const KEY_COMPLETE: ::core::ffi::c_int = 259;
pub const FUZZY_SCORE_NONE: ::core::ffi::c_int = -2147483648;
pub const CTRL_X_CMDLINE_CTRL_X: ::core::ffi::c_int = 17;
pub const CTRL_X_NORMAL: ::core::ffi::c_int = 0;
pub const CTRL_X_NOT_DEFINED_YET: ::core::ffi::c_int = 1;
pub const CTRL_X_CMDLINE: ::core::ffi::c_int = 11;
pub const CTRL_X_SCROLL: ::core::ffi::c_int = 2;
pub const CTRL_X_WHOLE_LINE: ::core::ffi::c_int = 3;
pub const CTRL_X_FILES: ::core::ffi::c_int = 4;
pub const CTRL_X_TAGS: ::core::ffi::c_int = 261;
pub const CTRL_X_PATH_PATTERNS: ::core::ffi::c_int = 262;
pub const CTRL_X_PATH_DEFINES: ::core::ffi::c_int = 263;
pub const CTRL_X_DICTIONARY: ::core::ffi::c_int = 265;
pub const CTRL_X_THESAURUS: ::core::ffi::c_int = 266;
pub const CTRL_X_FUNCTION: ::core::ffi::c_int = 12;
pub const CTRL_X_OMNI: ::core::ffi::c_int = 13;
pub const CTRL_X_SPELL: ::core::ffi::c_int = 14;
pub const CTRL_X_EVAL: ::core::ffi::c_int = 16;
pub const CTRL_X_REGISTER: ::core::ffi::c_int = 19;
pub const CTRL_X_BUFNAMES: ::core::ffi::c_int = 18;
pub struct ComplItem {
    pub cp_next: *mut ComplItem,
    pub cp_prev: *mut ComplItem,
    pub cp_match_next: *mut ComplItem,
    pub cp_str: String_0,
    pub cp_text: [*mut ::core::ffi::c_char; 4],
    pub cp_user_data: TypVal,
    pub cp_fname: *mut ::core::ffi::c_char,
    pub cp_flags: ::core::ffi::c_int,
    pub cp_number: ::core::ffi::c_int,
    pub cp_score: ::core::ffi::c_int,
    pub cp_in_match_array: bool,
    pub cp_user_abbr_hlattr: ::core::ffi::c_int,
    pub cp_user_kind_hlattr: ::core::ffi::c_int,
    pub cp_cpt_source_idx: ::core::ffi::c_int,
}
pub const CP_ICASE: ::core::ffi::c_int = 16;
pub const CP_ORIGINAL_TEXT: ::core::ffi::c_int = 1;
pub const CPT_COUNT: ::core::ffi::c_int = 4;
pub const CP_FREE_FNAME: ::core::ffi::c_int = 2;
pub const CP_FAST: ::core::ffi::c_int = 32;
pub const CP_CONT_S_IPOS: ::core::ffi::c_int = 4;
pub const CPT_INFO: ::core::ffi::c_int = 3;
pub const CPT_KIND: ::core::ffi::c_int = 1;
pub const CPT_MENU: ::core::ffi::c_int = 2;
pub const CPT_ABBR: ::core::ffi::c_int = 0;
#[derive(Copy, Clone)]
pub struct CptSource {
    pub cs_refresh_always: bool,
    pub cs_startcol: ::core::ffi::c_int,
    pub cs_max_matches: ::core::ffi::c_int,
    pub compl_start_tv: uint64_t,
    pub cs_flag: ::core::ffi::c_char,
}
/// A zeroed `CptSource`, which is what `xcalloc` left every row as.
pub(crate) const CPT_SOURCE_INIT: CptSource = CptSource {
    cs_refresh_always: false,
    cs_startcol: 0,
    cs_max_matches: 0,
    compl_start_tv: 0,
    cs_flag: 0,
};
pub const CP_EQUAL: ::core::ffi::c_int = 8;
pub struct InsComplNextState {
    /// The copy of `'complete'` being walked, and where the walk is up to.
    /// Owning, which is why this struct is no longer `Copy`.
    pub(crate) cpt: CptScan,
    /// The buffer being scanned. An identity: it outlives the user functions
    /// and Lua a completion runs, which is exactly the liveness a
    /// [`crate::winlayer::Buf`] would be promising. Each use resolves it
    /// where it needs the buffer, and copes with it having been wiped.
    pub ins_buf: Option<BufId>,
    pub cur_match_pos: *mut Pos,
    pub prev_match_pos: Pos,
    pub set_match_pos: bool,
    pub first_match_pos: Pos,
    pub last_match_pos: Pos,
    pub found_all: bool,
    pub dict: *mut ::core::ffi::c_char,
    pub dict_f: ::core::ffi::c_int,
    pub func_cb: *mut Callback,
}
pub const NUM_REGISTERS: ::core::ffi::c_int = 39;
pub const TAG_MANY: ::core::ffi::c_int = 300;
pub const TAG_VERBOSE: ::core::ffi::c_int = 32;
pub const TAG_INS_COMP: ::core::ffi::c_int = 64;
pub const TAG_NOIC: ::core::ffi::c_int = 8;
pub const TAG_NAMES: ::core::ffi::c_int = 2;
pub const TAG_REGEXP: ::core::ffi::c_int = 4;
pub const LSIZE: ::core::ffi::c_int = 512;
pub const ACTION_EXPAND: ::core::ffi::c_int = 5;
pub const FIND_ANY: ::core::ffi::c_int = 1;
pub const FIND_DEFINE: ::core::ffi::c_int = 2;
pub const INS_COMPL_CPT_CONT: ::core::ffi::c_int = 2;
pub const INS_COMPL_CPT_OK: ::core::ffi::c_int = 1;
pub const INS_COMPL_CPT_END: ::core::ffi::c_int = 3;
pub const CTRL_X_LOCAL_MSG: ::core::ffi::c_int = 15;
pub const CTRL_X_FINISHED: ::core::ffi::c_int = 8;
/// A zeroed `InsComplNextState`: C's `CLEAR_FIELD(st)`.
pub(crate) const INS_COMPL_NEXT_STATE_INIT: InsComplNextState = InsComplNextState {
    cpt: CptScan::EMPTY,
    ins_buf: None,
    cur_match_pos: ptr::null_mut(),
    prev_match_pos: POS_T_INIT,
    set_match_pos: false,
    first_match_pos: POS_T_INIT,
    last_match_pos: POS_T_INIT,
    found_all: false,
    dict: ptr::null_mut(),
    dict_f: 0,
    func_cb: ptr::null_mut(),
};
/// An unset `TypVal`, which the transpile writes out at every declaration
/// (C leaves these uninitialised and has the callee fill them in).
pub(crate) const TYPVAL_T_INIT: TypVal = TV_INITIAL_VALUE;

/// A zeroed `Pos`.
pub(crate) const POS_T_INIT: Pos = Pos {
    lnum: 0,
    col: 0,
    coladd: 0,
};
/// A zeroed `SaveVEvent`, which `get_v_event` fills in.
pub(crate) const SAVE_V_EVENT_INIT: SaveVEvent = SaveVEvent {
    sve_did_save: false,
    sve_hashtab: HashTab::new(),
};
/// A zeroed `extmark_undo_vec_t`, which is what C's `kv_destroy` leaves.
pub(crate) const EXTMARK_UNDO_VEC_INIT: extmark_undo_vec_t = extmark_undo_vec_t {
    size: 0,
    capacity: 0,
    items: ptr::null_mut(),
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const PATHSEP: ::core::ffi::c_int = '/' as ::core::ffi::c_int;
pub const CTRL_X_WANT_IDENT: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
/// Message for CTRL-X mode, indexed by `ctrl_x_mode` with `CTRL_X_WANT_IDENT`
/// masked off (C's `CTRL_X_MSG(i)` macro; see `ctrl_x_msg`). `None` is
/// upstream's NULL: the mode either computes its own message or has none.
pub(crate) const CTRL_X_MSGS: [Option<&CStr>; 20] = [
    Some(c" Keyword completion (^N^P)"), // CTRL_X_NORMAL, ^P/^N compl.
    Some(c" ^X mode (^]^D^E^F^I^K^L^N^O^P^Rs^U^V^Y)"),
    None, // CTRL_X_SCROLL: depends on state
    Some(c" Whole line completion (^L^N^P)"),
    Some(c" File name completion (^F^N^P)"),
    Some(c" Tag completion (^]^N^P)"),
    Some(c" Path pattern completion (^N^P)"),
    Some(c" Definition completion (^D^N^P)"),
    None, // CTRL_X_FINISHED
    Some(c" Dictionary completion (^K^N^P)"),
    Some(c" Thesaurus completion (^T^N^P)"),
    Some(c" Command-line completion (^V^N^P)"),
    Some(c" User defined completion (^U^N^P)"),
    Some(c" Omni completion (^O^N^P)"),
    Some(c" Spelling suggestion (^S^N^P)"),
    Some(c" Keyword Local completion (^N^P)"),
    None, // CTRL_X_EVAL doesn't use msg.
    Some(c" Command-line completion (^V^N^P)"),
    None, // CTRL_X_BUFNAMES
    Some(c" Register completion (^N^P)"),
];

/// The name `complete_info()` and `v:event.complete_type` report for each
/// CTRL-X mode, indexed as `CTRL_X_MSGS` is.
pub(crate) const CTRL_X_MODE_NAMES: [Option<&CStr>; 20] = [
    Some(c"keyword"),
    Some(c"ctrl_x"),
    Some(c"scroll"),
    Some(c"whole_line"),
    Some(c"files"),
    Some(c"tags"),
    Some(c"path_patterns"),
    Some(c"path_defines"),
    Some(c"unknown"), // CTRL_X_FINISHED
    Some(c"dictionary"),
    Some(c"thesaurus"),
    Some(c"cmdline"),
    Some(c"function"),
    Some(c"omni"),
    Some(c"spell"),
    None, // CTRL_X_LOCAL_MSG, only used in CTRL_X_MSGS
    Some(c"eval"),
    Some(c"cmdline"),
    None, // CTRL_X_BUFNAMES
    Some(c"register"),
];

/// C's `_(CTRL_X_MSG(mode))`: the translated CTRL-X mode message. Upstream
/// indexes and passes the result to `gettext()` unconditionally, so the NULL
/// rows answer `None` here rather than panicking; no caller reaches one (the
/// three modes without a message never take these paths).
pub(crate) fn ctrl_x_msg(mode: c_int) -> Option<&'static CStr> {
    let row =
        usize::try_from(mode & !CTRL_X_WANT_IDENT).expect("a CTRL-X mode number is never negative");
    CTRL_X_MSGS[row].map(gettext)
}

/// One of the completion's owned strings.
///
/// `compl_pattern`, `compl_leader`, `compl_orig_text`, `cpt_compl_pattern`
/// and `adjusted_leader` are five `String`s upstream keeps at file scope, a
/// `char *` and a length each, whose bytes belong to the running completion.
/// Upstream frees them by hand: `XFREE_CLEAR` is spelled out at a dozen
/// sites, and which of them owns the buffer it is about to overwrite is
/// carried in the reader's head.
///
/// `ComplStr` is the one owner of each. It names the cell rather than
/// pointing into it, and every read takes the cell's borrow only for the
/// length of that read — which matters because completion runs user
/// callbacks, and a callback can reach the same string. The *bytes* are
/// still handed out raw, because every consumer of them is C-shaped; they
/// stay valid until the next [`set`](ComplStr::set) or
/// [`replace`](ComplStr::replace), exactly as upstream's did.
#[derive(Clone, Copy)]
pub(crate) struct ComplStr(ComplField<String_0>);

impl ComplStr {
    /// The bytes, or null while the string is unset.
    pub(crate) fn data(self) -> *mut c_char {
        self.0.with(String_0::data)
    }

    /// The byte count.
    pub(crate) fn len(self) -> size_t {
        self.0.with(String_0::len)
    }

    /// Both words, read under one borrow, for the callers that want a
    /// snapshot rather than two separate reads.
    pub(crate) fn parts(self) -> (*mut c_char, size_t) {
        self.0.with(|s| (s.data(), s.len()))
    }

    /// A copy of the string, with its own allocation.
    pub(crate) fn to_owned(self) -> String_0 {
        self.0.with(String_0::clone)
    }

    /// Whether the string has no buffer at all — upstream's
    /// `if (compl_leader.data == NULL)`, which asks something different from
    /// [`is_empty`](Self::is_empty).
    pub(crate) fn is_unset(self) -> bool {
        self.0.with(String_0::is_null)
    }

    /// Whether the string has no bytes.
    pub(crate) fn is_empty(self) -> bool {
        self.0.with(String_0::is_empty)
    }

    /// Take `s`, releasing what was there.
    ///
    /// Upstream has two spellings of this — one that frees first and one
    /// that assumes the string was already cleared — and an owning string
    /// makes them the same operation.
    pub(crate) fn set(self, s: String_0) {
        self.0.set(s);
    }

    /// C's `XFREE_CLEAR(x.data); x = s`. See [`set`](Self::set).
    pub(crate) fn replace(self, s: String_0) {
        self.0.set(s);
    }

    /// C's `XFREE_CLEAR(s->data); s->size = 0`.
    pub(crate) fn clear(self) {
        self.0.set(String_0::NULL);
    }

    /// Release the bytes and take over `data`, an `xmalloc`ed block of
    /// `len + 1` bytes with a NUL at `len`.
    ///
    /// The two-step build in `get_normal_compl_info` sizes the pattern and
    /// fills it in one go now, because a string that owns its bytes has no
    /// half-set state to go through.
    ///
    /// # Safety
    /// `data` must be such a block, which nothing else frees.
    pub(crate) unsafe fn set_owned(self, data: *mut c_char, len: size_t) {
        // SAFETY: the caller's promise.
        self.0.set(unsafe { String_0::from_owned_parts(data, len) });
    }

    /// Take over an [`XString`]'s block.
    pub(crate) fn set_string(self, pattern: XString) {
        self.0.set(String_0::from_xstring(pattern));
    }

    /// Shorten the string to `len` bytes. Panics past the current length.
    pub(crate) fn truncate(self, len: size_t) {
        let shorter = self.0.with(|s| String_0::from_bytes(&s.as_bytes()[..len]));
        self.0.set(shorter);
    }

    /// C's `XFREE_CLEAR(compl_leader)` written on the *struct* rather than on
    /// its `.data`: the bytes go and the pointer nulls, but the length is
    /// left stale. Reproduced deliberately for `ins_compl_build_pum`; every
    /// reader guards on the pointer.
    pub(crate) fn free_bytes_keep_len(self) {
        let stale = self.len();
        // SAFETY: a null pointer owns nothing, so the string has nothing to
        // free; the stale length is the whole point of this spelling and is
        // only ever read behind a null check.
        self.0
            .set(unsafe { String_0::from_owned_parts(ptr::null_mut(), stale) });
    }
}

/// What the current completion searches for.
pub(crate) fn compl_pattern() -> ComplStr {
    ComplStr(COMPL_PATTERN)
}

/// The `'complete'` source's own pattern, when its startcol differs from
/// `compl_col`.
pub(crate) fn cpt_compl_pattern() -> ComplStr {
    ComplStr(CPT_COMPL_PATTERN)
}

/// What the user has typed since the completion started, which filters the
/// matches. Unset until the first `ins_compl_addleader`.
pub(crate) fn compl_leader() -> ComplStr {
    ComplStr(COMPL_LEADER)
}

/// The text that was under the cursor when the completion started, and which
/// CTRL-E puts back.
pub(crate) fn compl_orig_text() -> ComplStr {
    ComplStr(COMPL_ORIG_TEXT)
}

/// [`compl_leader`] with the text a source's earlier startcol covers
/// prepended; the cache behind [`get_leader_for_startcol`].
pub(crate) fn adjusted_leader() -> ComplStr {
    ComplStr(ADJUSTED_LEADER)
}

/// C's `e_hitend`.
pub(crate) const E_HITEND: &CStr = c"Hit end of paragraph";

/// C's `e_compldel`.
pub(crate) const E_COMPLDEL: &CStr = c"E840: Completion function deleted text";

static compl_first_match: GlobalCell<*mut ComplItem> =
    GlobalCell::new(::core::ptr::null_mut::<ComplItem>());
static compl_curr_match: GlobalCell<*mut ComplItem> =
    GlobalCell::new(::core::ptr::null_mut::<ComplItem>());
static compl_shown_match: GlobalCell<*mut ComplItem> =
    GlobalCell::new(::core::ptr::null_mut::<ComplItem>());
static compl_old_match: GlobalCell<*mut ComplItem> =
    GlobalCell::new(::core::ptr::null_mut::<ComplItem>());

/// The head of the match list, `None` while there is no completion.
pub(crate) fn first_match() -> Option<Cm> {
    Cm::at(compl_first_match.get())
}

/// The match a CTRL-N/CTRL-P walk has reached.
pub(crate) fn curr_match() -> Option<Cm> {
    Cm::at(compl_curr_match.get())
}

/// The match the popup menu highlights.
pub(crate) fn shown_match() -> Option<Cm> {
    Cm::at(compl_shown_match.get())
}

/// The match that was shown before the last walk step.
pub(crate) fn old_match() -> Option<Cm> {
    Cm::at(compl_old_match.get())
}

/// The list from `start` onwards, stopping at the end of an opened list or
/// on the way back round a closed one.
///
/// C's `for (m = start; m != NULL; m = m->cp_next) { …; if (is_first_match(m->cp_next)) break; }`
/// — the walk every reader of the ring writes by hand. `start` itself is
/// always yielded, even when it is the head.
///
/// The link is read when the *next* item is asked for, not when the current
/// one is handed out, so a body that relinks the node it was given walks the
/// list it left behind — which is the timing the hand-written loops have.
pub(crate) fn matches_from(start: Option<Cm>) -> impl Iterator<Item = Cm> {
    let mut start = start;
    let mut current: Option<Cm> = None;
    ::core::iter::from_fn(move || {
        current = match current {
            None => start.take(),
            Some(m) => m.next().filter(|next| !next.is_first()),
        };
        current
    })
}

state_record! {
    /// The running completion: what it is completing, where, how far the
    /// collection has got and what the menu shows -- upstream's file-scope
    /// statics in `insexpand.c`, and the four function-scope ones.
    ///
    /// One cell, reached a field at a time through the selectors below, which
    /// keep upstream's names. Nothing holds a borrow of it across a call: a
    /// completion runs `'completefunc'`, autocommands and Lua, and all three
    /// reach back in through `complete_add()`/`complete_info()`.
    pub(crate) struct ComplState in COMPL as ComplField;
    /// How many of the best fuzzy matches `'completeopt'` `longest` shares a
    /// prefix over.
    compl_num_bests: c_int = 0;
    /// Enter selects the shown match rather than inserting a line break.
    compl_enter_selects: bool = false;
    /// See [`compl_leader`].
    COMPL_LEADER: String_0 = String_0::NULL;
    /// See [`adjusted_leader`].
    ADJUSTED_LEADER: String_0 = String_0::NULL;
    /// Still finding the longest common text (`'completeopt'` `longest`).
    compl_get_longest: bool = false;
    /// The selected match is in the buffer.
    compl_used_match: bool = false;
    /// The last collection was interrupted by a typed key.
    compl_was_interrupted: bool = false;
    /// The collection running now was interrupted.
    compl_interrupted: bool = false;
    /// Searching again without leaving CTRL-X mode: don't insert the first
    /// match.
    compl_restarting: bool = false;
    /// The match list has been started.
    compl_started: bool = false;
    /// Which CTRL-X mode is running; `CTRL_X_*`.
    ctrl_x_mode: c_int = CTRL_X_NORMAL;
    /// The number of matches, once known.
    compl_matches: c_int = 0;
    /// See [`compl_pattern`].
    COMPL_PATTERN: String_0 = String_0::NULL;
    /// See [`cpt_compl_pattern`].
    CPT_COMPL_PATTERN: String_0 = String_0::NULL;
    /// The direction matches are collected in.
    compl_direction: Direction = FORWARD;
    /// The direction the shown matches run in.
    compl_shows_dir: Direction = FORWARD;
    /// CTRL-N/CTRL-P presses not yet acted on, while still collecting.
    compl_pending: c_int = 0;
    /// Where the completion started.
    compl_startpos: Pos = POS_T_INIT;
    /// The length of the text being completed.
    compl_length: c_int = 0;
    /// The line the completion started on.
    compl_lnum: LineNr = 0;
    /// The column the completed text starts at.
    compl_col: ColNr = 0;
    /// Where the inserted completion text ends.
    compl_ins_end_col: ColNr = 0;
    /// See [`compl_orig_text`].
    COMPL_ORIG_TEXT: String_0 = String_0::NULL;
    /// See [`ComplOrigExtmarks`].
    COMPL_ORIG_EXTMARKS: extmark_undo_vec_t = EXTMARK_UNDO_VEC_INIT;
    /// The CTRL-X mode a continued completion continues.
    compl_cont_mode: c_int = 0;
    /// CTRL-X CTRL-V's expansion context, kept between working out the
    /// pattern and expanding it. Moved out for each use: the expansion runs
    /// Lua and user completion functions.
    compl_xp: Option<Box<Expand>> = None;
    /// The window the running completion started in. See
    /// [`ins_compl_win_active`]: an identity, not an address, because a
    /// completion runs user functions, autocommands and Lua, any of which
    /// can close the window or wipe the buffer, and comparison is all that
    /// is ever done with it.
    compl_curr_win: Option<WinId> = None;
    /// The buffer it started in; see `compl_curr_win`.
    compl_curr_buf: Option<BufId> = None;
    /// The completion is `'autocomplete'`'s.
    compl_autocomplete: bool = false;
    /// The time budget of one `'complete'` source.
    compl_timeout_ms: uint64_t = COMPL_INITIAL_TIMEOUT_MS as uint64_t;
    /// The current source ran out of time.
    compl_time_slice_expired: bool = false;
    /// The completion started after a non-keyword character.
    compl_from_nonkeyword: bool = false;
    /// Highlight the text `'autocomplete'`'s `longest` inserted.
    compl_hi_on_autocompl_longest: bool = false;
    /// `CONT_*` flags: how a CTRL-X continuation proceeds.
    compl_cont_status: c_int = 0;
    /// The completion function answered `refresh: 'always'`.
    compl_opt_refresh_always: bool = false;
    /// The length of the bad word spell completion started on.
    spell_bad_len: size_t = 0;
    /// The menu entry selected, `-1` for none.
    compl_selected_item: c_int = -1;
    /// See [`CptSources`].
    CPT_SOURCES: Vec<CptSource> = Vec::new();
    /// The `'complete'` entry being collected from, `-1` between scans.
    CPT_SOURCES_INDEX: c_int = -1;
    /// See [`ComplMatchArray`].
    COMPL_MATCH_ARRAY: Vec<PumItem> = Vec::new();
    /// See [`CptCallbacks`].
    CPT_CB: Vec<Callback> = Vec::new();
    /// `CompleteChanged` is running: it does not fire again from inside.
    complete_changed_busy: bool = false;
    /// [`ins_compl_check_keys`]'s call count, which it only acts on every
    /// `frequency` calls.
    check_keys_count: c_int = 0;
    /// The window [`ins_compl_next_buf`]'s `w` walk is at: a handle that
    /// `win_valid` vets, because it outlives the call.
    next_buf_window: Option<WinId> = None;
}

pub const COMPL_INITIAL_TIMEOUT_MS: ::core::ffi::c_int = 80 as ::core::ffi::c_int;
pub const COMPL_MIN_TIMEOUT_MS: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const COMPL_FUNC_TIMEOUT_MS: ::core::ffi::c_int = 300 as ::core::ffi::c_int;
pub const COMPL_FUNC_TIMEOUT_NON_KW_MS: ::core::ffi::c_int = 1000 as ::core::ffi::c_int;
pub const CONT_ADDING: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const CONT_INTRPT: ::core::ffi::c_int = 2 as ::core::ffi::c_int + 4 as ::core::ffi::c_int;
pub const CONT_N_ADDS: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const CONT_S_IPOS: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const CONT_SOL: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const CONT_LOCAL: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const DICT_FIRST: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const DICT_EXACT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
static CFU_CB: GlobalCell<Callback> = GlobalCell::new(Callback::None);
static OFU_CB: GlobalCell<Callback> = GlobalCell::new(Callback::None);
static TSRFU_CB: GlobalCell<Callback> = GlobalCell::new(Callback::None);
pub const CI_WHAT_MODE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CI_WHAT_PUM_VISIBLE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CI_WHAT_ITEMS: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CI_WHAT_SELECTED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CI_WHAT_COMPLETED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CI_WHAT_MATCHES: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const CI_WHAT_PREINSERTED_TEXT: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CI_WHAT_ALL: ::core::ffi::c_int = 0xff as ::core::ffi::c_int;
pub const MIN_SPACE: ::core::ffi::c_int = 75 as ::core::ffi::c_int;
