#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::ascii::TAB;
use crate::autocmd::defs::AutoEvent;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::fuzzy::{FUZZY_SCORE_NONE, fuzzy_match_str};
use crate::indent_c::KEY_COMPLETE;
use crate::memory::XString;
use crate::search::DIRECTION_NOT_SET;
use crate::tag::{
    LSIZE, TAG_INS_COMP, TAG_MANY, TAG_NOIC, TAG_REGEXP, TAG_VERBOSE, find_tag_names,
};
use crate::types::eval::SAVE_V_EVENT_INIT;
use core::ffi::{CStr, c_int, c_uint};

use std::rc::Rc;

use crate::ascii::{ascii_iswhite, ascii_iswhite_or_nul};
use crate::autocmd::has_event;
use crate::buffer::buf_spname;
use crate::change::{deleted_lines_mark, ins_bytes, ins_char, ins_char_bytes, ins_str, open_line};
use crate::charset::{
    ptr2cells_at, strsize, vim_is_ident_char, vim_isfilec, vim_isprintc, vim_iswordc,
};
use crate::cmdexpand::defs::Expand;
use crate::cmdexpand::{addstar, expand_cmdline, set_cmd_context};
use crate::cursor::{check_cursor, dec_cursor, get_cursor_line_len, inc_cursor};
use crate::drawscreen::state::{DOLLAR_VCOL, REDRAW_CMDLINE, REDRAW_MODE, SC_COL};
use crate::drawscreen::{
    UPD_VALID, redraw_later, redraw_win_line, setcursor, showmode, update_screen,
};
use crate::edit::{
    InsertEnd, backspace_until_column, get_can_cindent, ins_apply_autocmds, ins_eol,
    ins_need_undo_get, ins_redraw, insertchar, start_arrow, stop_arrow,
};
use crate::eval::funcs::builtin::EvalFuncData;
use crate::eval::typval::{
    callback_free, dict_find, dict_get_number, dict_get_tv, tv_clear, tv_dict_alloc,
    tv_dict_alloc_lock, tv_dict_alloc_ret, tv_get_number_chk, tv_list_alloc,
};
use crate::eval::userfunc::callback_call_retnr;
use crate::eval::vars::set_vim_var_dict;
use crate::eval::{callback_call, get_v_event, restore_v_event, set_ref_in_callback};
use crate::ex_docmd::state::{EX_NORMAL_BUSY, GLOBAL_BUSY};
use crate::ex_eval::aborting;
use crate::extmark::{ExtmarkUndoObject, extmark_apply_undo};
use crate::getchar::state::{GOT_INT, KEY_TYPED, test_disable_char_avail};
use crate::getchar::{
    append_to_redobuff_char, append_to_redobuff_literally, char_avail, safe_vgetc, using_script,
    vgetc, vpeekc, vpeekc_any, vungetc,
};
use crate::global_cell::{GlobalCell, state_record};
use crate::highlight_group::{HLF_COUNT, HLF_E, HLF_R, HLF_W, syn_name2attr};
use crate::indent::{get_indent, inindent};
use crate::indent_c::{cindent_on, do_c_expr_indent, in_cinkeys};
use crate::lua::executor::nlua_expand_pat;
use crate::mbyte::MB_MAXCHAR;
use crate::mbyte::{c_cluster_len, mb_islower, mb_isupper, mb_tolower, mb_toupper, utf8len_tab};
use crate::memline::{dec, ml_delete};
use crate::message::state::{DID_EMSG, EMSG_SILENT, IN_ASSERT_FAILS, MSG_HIST_OFF};
use crate::message::{E_INVARG, E_LISTREQ, E_PATNOTF};
use crate::message::{
    emsg, internal_error, msg_clr_cmdline, msg_delay, msg_ext_set_kind, msg_progress,
};
use crate::r#move::{changed_cline_bef_curs, curs_columns, validate_cursor};
use crate::option::defs::OptSet;
use crate::option::vars::{
    COT_FLAGS, P_IC, P_SCS, P_WS, p_ac, p_acl, p_act, p_cto, p_dict, p_fic, p_ic, p_inf, p_js,
    p_paste, p_scs, p_smd, p_tsr, p_tsrfu, p_wic, p_ws,
};
use crate::option::{callback_from_option, can_bs, install_callback, magic_isset, shortmess};
use crate::options::{
    OPT_BO_FLAG_COMPLETE, OPT_COT_FLAG_FUZZY, OPT_COT_FLAG_LONGEST, OPT_COT_FLAG_MENU,
    OPT_COT_FLAG_MENUONE, OPT_COT_FLAG_NEAREST, OPT_COT_FLAG_NOINSERT, OPT_COT_FLAG_NOSELECT,
    OPT_COT_FLAG_NOSORT, OPT_COT_FLAG_PREINSERT,
};
use crate::os::cshim::gettext;
use crate::os::input::{fast_breakcheck, line_breakcheck, os_breakcheck};
use crate::os::libc::{ptrdiff_t, size_t, uint64_t};
use crate::os::time::{os_delay, os_hrtime};
use crate::path::{expand_wildcards, path_tail, vim_ispathsep};
use crate::popupmenu::state::{PUM_WANT, PumItem};
use crate::popupmenu::{
    pum_clear, pum_display, pum_get_height, pum_set_event_info, pum_undisplay, pum_visible,
};
use crate::pos::{MAXCOL, MAXLNUM, equalpos};
use crate::regexp::RegMatch;
use crate::regexp::{RE_LAST, RE_MAGIC, vim_regcomp, vim_regexec};
use crate::register::{NUM_REGISTERS, copy_register, get_register_name, valid_yank_reg};
use crate::search::{
    BACKWARD, FORWARD, SEARCH_KEEP, SEARCH_NFMSG, find_pattern_in_path, ignorecase,
    search_for_exact_line, searchit,
};
use crate::spell::{
    SMT_ALL, expand_spelling, spell_dump_compl, spell_expand_check_cap, spell_word_start,
};
use crate::state::mode::{
    ARROW_USED, CAN_SI, CAN_SI_BACK, DID_AI, DID_SI, EDIT_SUBMODE, EDIT_SUBMODE_EXTRA,
    EDIT_SUBMODE_HIGHL, EDIT_SUBMODE_PRE, STATE,
};
use crate::state::{MODE_INSERT, REPLACE_FLAG, may_trigger_modechanged};
use crate::tag::state::G_TAG_AT_CURSOR;
use crate::textformat::auto_format;
use crate::types::{
    ApiString, BoolVarValue, Callback, ColNr, Dict, Direction, LineNr, List, OptInt, Pos, TypVal,
    VarNumber, Vv,
};
use crate::ui::{ui_flush, vim_beep};
use crate::undo::undo_allowed;
use crate::window::win_valid;
use crate::winfloat::win_float_find_preview;
use crate::winlayer::graph::CMDWIN_TYPE;
use crate::winlayer::{BufId, WinId};

// The carve of the transpiled module; see each child's docs.
mod mode;
pub use self::mode::*;
mod matchlist;
pub use self::matchlist::*;
mod text;
pub(crate) use self::text::*;
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
/// One match: the text it inserts and what the menu and `complete_info()`
/// show for it.
///
/// It owns everything it holds. The list it is linked into is
/// [`ComplMatches`]: `next`/`prev` name neighbours by [`MatchId`], never by
/// address, so a match added while another is being looked at (a
/// `complete_add()` from inside a completion function) cannot move anything
/// out from under the reader -- only the storage `Vec` grows, and each
/// string here is a heap block of its own that stays where it is.
#[derive(Debug)]
pub(crate) struct ComplItem {
    /// The match after this one, `None` past the tail of an opened list.
    pub(crate) next: Option<MatchId>,
    /// The match before this one, `None` before the head of an opened list.
    pub(crate) prev: Option<MatchId>,
    /// The text the match inserts.
    pub(crate) text: XString,
    /// `abbr`, `kind`, `menu` and `info`, indexed by `CPT_*`; never empty.
    pub(crate) extra: [Option<XString>; CPT_COUNT],
    /// The `user_data` a completion function handed over.
    pub(crate) user_data: TypVal,
    /// The file the match came from. Shared with the match added before it
    /// when that came from the same file, which is upstream's `fname`
    /// sharing without its `CP_FREE_FNAME` bookkeeping.
    pub(crate) fname: Option<Rc<XString>>,
    /// `CP_*` flags.
    pub(crate) flags: c_int,
    /// The position "match 3 of 17" reports, `-1` until numbered.
    pub(crate) number: c_int,
    /// The fuzzy score, or under `nearest` the distance from the cursor.
    pub(crate) score: c_int,
    /// The match is in the popup menu's item array.
    pub(crate) in_match_array: bool,
    /// The highlight attributes of the `abbr` and `kind` columns, `-1` for
    /// the default.
    pub(crate) user_abbr_hlattr: c_int,
    pub(crate) user_kind_hlattr: c_int,
    /// The `'complete'` entry it came from, `-1` for none.
    pub(crate) cpt_source_idx: c_int,
    /// The `'complete'` rows' generation when it was added; see
    /// [`ComplItem::cpt_source`].
    pub(crate) cpt_generation: u64,
}

impl ComplItem {
    /// Whether this is the original text the completion began with — C's
    /// `match_at_original_text`.
    #[inline(always)]
    pub(crate) fn is_original(&self) -> bool {
        self.flags & CP_ORIGINAL_TEXT != 0
    }

    /// The `'complete'` row this match came from, `None` for none — and
    /// for a row index left over from rows that have been rebuilt since,
    /// which upstream reads out of the freed array.
    pub(crate) fn cpt_source(&self) -> Option<usize> {
        cpt_sources().live_index(self.cpt_source_idx, self.cpt_generation)
    }
}
pub const CP_ICASE: ::core::ffi::c_int = 16;
pub const CP_ORIGINAL_TEXT: ::core::ffi::c_int = 1;
pub const CP_FAST: ::core::ffi::c_int = 32;
pub const CP_CONT_S_IPOS: ::core::ffi::c_int = 4;
/// The four strings a match carries beside its text, as indices into its
/// `extra`; the first three are also the popup menu's column kinds, in
/// `'completeitemalign'`'s default order.
pub const CPT_ABBR: usize = 0;
pub const CPT_KIND: usize = 1;
pub const CPT_MENU: usize = 2;
pub const CPT_INFO: usize = 3;
pub const CPT_COUNT: usize = 4;
#[derive(Copy, Clone, Debug)]
pub struct CptSource {
    pub cs_refresh_always: bool,
    /// Where a `'complete'` function said its matches start; `None` when
    /// it declined (its `findstart` answered -2 or -3, or it failed) or the
    /// entry is not a function.
    pub cs_startcol: Option<ColNr>,
    pub cs_max_matches: ::core::ffi::c_int,
    pub compl_start_tv: uint64_t,
    /// The entry's source letter (`.`, `w`, `b`, `u`, `U`, `k`, ...).
    pub cs_flag: u8,
}
/// A zeroed `CptSource`, which is what `xcalloc` left every row as.
pub(crate) const CPT_SOURCE_INIT: CptSource = CptSource {
    cs_refresh_always: false,
    cs_startcol: Some(ColNr::ZERO),
    cs_max_matches: 0,
    compl_start_tv: 0,
    cs_flag: 0,
};
pub const CP_EQUAL: ::core::ffi::c_int = 8;
/// Where [`ins_compl_get_exp`] is up to in `'complete'` and in the buffer it
/// is scanning.
#[derive(Debug)]
pub(crate) struct InsComplNextState {
    /// The copy of `'complete'` being walked, and where the walk is up to.
    /// Owning, which is why this struct is no longer `Copy`.
    pub(crate) cpt: CptScan,
    /// The buffer being scanned. An identity: it outlives the user functions
    /// and Lua a completion runs, which is exactly the liveness a
    /// [`crate::winlayer::Buf`] would be promising. Each use resolves it
    /// where it needs the buffer, and copes with it having been wiped.
    pub ins_buf: Option<BufId>,
    /// Which of the two match positions the search moves: the last going
    /// forward, the first going backward. Upstream points at it.
    pub cur_is_last: bool,
    pub prev_match_pos: Pos,
    pub set_match_pos: bool,
    pub first_match_pos: Pos,
    pub last_match_pos: Pos,
    pub found_all: bool,
    /// The dictionary a `k`/`s` entry names, or an unloaded buffer's file
    /// name: a copy, because the scan runs code that can wipe the buffer.
    pub dict: Option<XString>,
    pub dict_f: ::core::ffi::c_int,
    /// The `'complete'` function the current entry names, held for the
    /// scan.
    pub func_cb: Option<HeldCallback>,
    /// Where [`get_next_default_completion`] copies the match it found,
    /// kept so its allocation serves the whole scan.
    pub(crate) found: Vec<u8>,
}
pub const INS_COMPL_CPT_CONT: ::core::ffi::c_int = 2;
pub const INS_COMPL_CPT_OK: ::core::ffi::c_int = 1;
pub const INS_COMPL_CPT_END: ::core::ffi::c_int = 3;
pub const CTRL_X_LOCAL_MSG: ::core::ffi::c_int = 15;
pub const CTRL_X_FINISHED: ::core::ffi::c_int = 8;
/// A zeroed `InsComplNextState`: C's `CLEAR_FIELD(st)`.
pub(crate) const INS_COMPL_NEXT_STATE_INIT: InsComplNextState = InsComplNextState {
    cpt: CptScan::EMPTY,
    ins_buf: None,
    cur_is_last: false,
    prev_match_pos: POS_T_INIT,
    set_match_pos: false,
    first_match_pos: POS_T_INIT,
    last_match_pos: POS_T_INIT,
    found_all: false,
    dict: None,
    dict_f: 0,
    func_cb: None,
    found: Vec::new(),
};
/// An unset `TypVal`, which the transpile writes out at every declaration
/// (C leaves these uninitialised and has the callee fill them in).
pub(crate) const TYPVAL_T_INIT: TypVal = TV_INITIAL_VALUE;

/// A zeroed `Pos`.
pub(crate) const POS_T_INIT: Pos = Pos {
    lnum: LineNr::ZERO,
    col: ColNr::ZERO,
    coladd: 0,
};
pub const CTRL_X_WANT_IDENT: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
/// Message for CTRL-X mode, indexed by `CTRL_X_MODE` with `CTRL_X_WANT_IDENT`
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
/// callbacks, and a callback can reach the same string. The bytes are only
/// lent to a closure ([`with_bytes`](ComplStr::with_bytes),
/// [`with_cstr`](ComplStr::with_cstr)) or copied out.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ComplStr(ComplField<ApiString>);

impl ComplStr {
    /// The byte count.
    pub(crate) fn len(self) -> size_t {
        self.0.with(ApiString::len)
    }

    /// Look at the bytes (empty while unset). `f` must not reach back into
    /// the editor.
    pub(crate) fn with_bytes<R>(self, f: impl FnOnce(&[u8]) -> R) -> R {
        self.0.with(|s| f(s.as_bytes()))
    }

    /// Look at the string as a C string (`c""` while unset). `f` must not
    /// reach back into the editor.
    pub(crate) fn with_cstr<R>(self, f: impl FnOnce(&CStr) -> R) -> R {
        self.0.with(|s| f(s.as_cstr()))
    }

    /// A copy of the bytes.
    pub(crate) fn to_vec(self) -> Vec<u8> {
        self.with_bytes(<[u8]>::to_vec)
    }

    /// A copy of the string, with its own allocation.
    pub(crate) fn to_owned(self) -> ApiString {
        self.0.with(ApiString::clone)
    }

    /// Whether the string has no buffer at all — upstream's
    /// `if (compl_leader.data == NULL)`, which asks something different from
    /// [`is_empty`](Self::is_empty).
    pub(crate) fn is_unset(self) -> bool {
        self.0.with(ApiString::is_null)
    }

    /// Whether the string has no bytes.
    pub(crate) fn is_empty(self) -> bool {
        self.0.with(ApiString::is_empty)
    }

    /// Take `s`, releasing what was there.
    ///
    /// Upstream has two spellings of this — one that frees first and one
    /// that assumes the string was already cleared — and an owning string
    /// makes them the same operation.
    pub(crate) fn set(self, s: ApiString) {
        self.0.set(s);
    }

    /// C's `XFREE_CLEAR(x.data); x = s`. See [`set`](Self::set).
    pub(crate) fn replace(self, s: ApiString) {
        self.0.set(s);
    }

    /// C's `XFREE_CLEAR(s->data); s->size = 0`.
    pub(crate) fn clear(self) {
        self.0.set(ApiString::NULL);
    }

    /// Take over an [`XString`]'s block.
    pub(crate) fn set_string(self, pattern: XString) {
        self.0.set(ApiString::from_xstring(pattern));
    }

    /// Shorten the string to `len` bytes. Panics past the current length.
    pub(crate) fn truncate(self, len: size_t) {
        let shorter = self.0.with(|s| ApiString::from_bytes(&s.as_bytes()[..len]));
        self.0.set(shorter);
    }
}

/// What the current completion searches for.
pub(crate) fn compl_pattern() -> ComplStr {
    ComplStr(COMPL_PATTERN)
}

/// The `'complete'` source's own pattern, when its startcol differs from
/// `COMPL_COL`.
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

/// A match: its slot in [`ComplMatches`]'s storage.
///
/// This is what a caller holds instead of a pointer. It is `Copy`, it names
/// no borrow, and the slot it names keeps its match until the list is freed
/// or the match is removed (a `'complete'` source refreshing): nothing is
/// ever moved between slots. So a `MatchId` taken before a call that runs
/// user code still names the same match after it, however many matches the
/// user code added -- which is the one thing upstream's `compl_T *` could
/// not promise and every walk here needs. Reading a removed slot panics
/// where upstream would read freed memory.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct MatchId(u32);

state_record! {
    /// The match list: a doubly linked chain over a `Vec`, which
    /// [`ins_compl_make_cyclic`] closes into a ring and
    /// [`ins_compl_make_linear`] opens again, and the four places a
    /// completion is at in it.
    ///
    /// Its own cell, apart from [`ComplState`], so a walk that holds the
    /// list can still ask the completion's other state; nothing holds it
    /// across a call that can run user code (see [`MatchId`]).
    pub(crate) struct ComplMatches in MATCHES as MatchesField;
    /// One slot per match added since the list was last freed; `None` once
    /// its match is removed.
    #[allow(dead_code, reason = "the slots are reached through the whole list, not by selector")]
    MATCH_SLOTS: Vec<Option<ComplItem>> = Vec::new();
    /// The head of the list, `None` while there is no completion.
    COMPL_FIRST_MATCH: Option<MatchId> = None;
    /// The match a CTRL-N/CTRL-P walk has reached; while collecting, where
    /// the next match is linked.
    COMPL_CURR_MATCH: Option<MatchId> = None;
    /// The match the popup menu highlights.
    COMPL_SHOWN_MATCH: Option<MatchId> = None;
    /// The current match before the collection that is running.
    COMPL_OLD_MATCH: Option<MatchId> = None;
    /// How many times the list has been freed: a [`MatchId`] taken before
    /// a call that ran user code is stale when this moved.
    MATCH_GENERATION: u32 = 0;
}

/// The list's [`MATCH_GENERATION`].
pub(crate) fn match_list_generation() -> u32 {
    MATCH_GENERATION.get()
}

impl ComplMatches {
    /// The match `id` names.
    pub(crate) fn get(&self, id: MatchId) -> &ComplItem {
        self.MATCH_SLOTS[id.0 as usize]
            .as_ref()
            .expect("a match of the live list")
    }

    /// The match `id` names, to change.
    pub(crate) fn get_mut(&mut self, id: MatchId) -> &mut ComplItem {
        self.MATCH_SLOTS[id.0 as usize]
            .as_mut()
            .expect("a match of the live list")
    }

    /// Store `item` in a new slot, linked to nothing yet.
    pub(crate) fn push(&mut self, item: ComplItem) -> MatchId {
        let id = MatchId(u32::try_from(self.MATCH_SLOTS.len()).expect("fewer than 2^32 matches"));
        self.MATCH_SLOTS.push(Some(item));
        id
    }

    /// Empty `id`'s slot, answering its match.
    pub(crate) fn take(&mut self, id: MatchId) -> ComplItem {
        self.MATCH_SLOTS[id.0 as usize]
            .take()
            .expect("a match of the live list")
    }

    /// Whether `id` is the head — C's `is_first_match`, which compares
    /// addresses, so `None` is the head of an empty list.
    pub(crate) fn is_first(&self, id: Option<MatchId>) -> bool {
        id == self.COMPL_FIRST_MATCH
    }
}

// Only the generic accessors are `#[inline]`: a non-generic `#[inline]` fn
// that names `MATCHES` makes the static exported (read through the GOT),
// whatever the fn's own visibility. LLVM inlines the small ones in-crate
// anyway.
impl MatchId {
    /// Look at the match. `f` must not reach back into the editor.
    #[inline(always)]
    pub(crate) fn with<R>(self, f: impl FnOnce(&ComplItem) -> R) -> R {
        MATCHES.with(|list| f(list.get(self)))
    }

    /// Change the match. `f` must not reach back into the editor.
    #[inline(always)]
    pub(crate) fn update<R>(self, f: impl FnOnce(&mut ComplItem) -> R) -> R {
        MATCHES.with_mut(|list| f(list.get_mut(self)))
    }

    /// The match after this one, `None` past the tail of an opened list.
    pub(crate) fn next(self) -> Option<Self> {
        self.with(|m| m.next)
    }

    /// The match before this one, `None` before the head of an opened list.
    pub(crate) fn prev(self) -> Option<Self> {
        self.with(|m| m.prev)
    }

    /// Whether this is the head of the list — C's `is_first_match`.
    pub(crate) fn is_first(self) -> bool {
        COMPL_FIRST_MATCH.get() == Some(self)
    }

    /// Whether this is the original text the completion began with.
    pub(crate) fn is_original(self) -> bool {
        self.with(ComplItem::is_original)
    }

    pub(crate) fn in_match_array(self) -> bool {
        self.with(|m| m.in_match_array)
    }

    /// The text's length in bytes.
    pub(crate) fn text_len(self) -> usize {
        self.with(|m| m.text.len())
    }

    /// A copy of the text, for a caller about to run something that could
    /// free the match.
    pub(crate) fn text_copy(self) -> XString {
        self.with(|m| m.text.clone())
    }
}

/// The head of the match list, `None` while there is no completion.
pub(crate) fn first_match() -> Option<MatchId> {
    COMPL_FIRST_MATCH.get()
}

/// The match a CTRL-N/CTRL-P walk has reached.
pub(crate) fn curr_match() -> Option<MatchId> {
    COMPL_CURR_MATCH.get()
}

/// The match the popup menu highlights.
pub(crate) fn shown_match() -> Option<MatchId> {
    COMPL_SHOWN_MATCH.get()
}

/// The match that was current before the collection that is running.
pub(crate) fn old_match() -> Option<MatchId> {
    COMPL_OLD_MATCH.get()
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
pub(crate) fn matches_from(start: Option<MatchId>) -> impl Iterator<Item = MatchId> {
    let mut start = start;
    let mut current: Option<MatchId> = None;
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
    COMPL_NUM_BESTS: c_int = 0;
    /// Enter selects the shown match rather than inserting a line break.
    COMPL_ENTER_SELECTS: bool = false;
    /// See [`compl_leader`].
    COMPL_LEADER: ApiString = ApiString::NULL;
    /// See [`adjusted_leader`].
    ADJUSTED_LEADER: ApiString = ApiString::NULL;
    /// Still finding the longest common text (`'completeopt'` `longest`).
    COMPL_GET_LONGEST: bool = false;
    /// The selected match is in the buffer.
    COMPL_USED_MATCH: bool = false;
    /// The last collection was interrupted by a typed key.
    COMPL_WAS_INTERRUPTED: bool = false;
    /// The collection running now was interrupted.
    COMPL_INTERRUPTED: bool = false;
    /// Searching again without leaving CTRL-X mode: don't insert the first
    /// match.
    COMPL_RESTARTING: bool = false;
    /// The match list has been started.
    COMPL_STARTED: bool = false;
    /// Which CTRL-X mode is running; `CTRL_X_*`.
    CTRL_X_MODE: c_int = CTRL_X_NORMAL;
    /// The number of matches, once known.
    COMPL_MATCHES: c_int = 0;
    /// See [`compl_pattern`].
    COMPL_PATTERN: ApiString = ApiString::NULL;
    /// See [`cpt_compl_pattern`].
    CPT_COMPL_PATTERN: ApiString = ApiString::NULL;
    /// The direction matches are collected in.
    COMPL_DIRECTION: Direction = FORWARD;
    /// The direction the shown matches run in.
    COMPL_SHOWS_DIR: Direction = FORWARD;
    /// CTRL-N/CTRL-P presses not yet acted on, while still collecting.
    COMPL_PENDING: c_int = 0;
    /// Where the completion started.
    COMPL_STARTPOS: Pos = POS_T_INIT;
    /// The length of the text being completed.
    COMPL_LENGTH: c_int = 0;
    /// The line the completion started on.
    COMPL_LNUM: LineNr = LineNr::ZERO;
    /// The column the completed text starts at.
    COMPL_COL: ColNr = ColNr::ZERO;
    /// Where the inserted completion text ends.
    COMPL_INS_END_COL: ColNr = ColNr::ZERO;
    /// See [`compl_orig_text`].
    COMPL_ORIG_TEXT: ApiString = ApiString::NULL;
    /// See [`ComplOrigExtmarks`].
    COMPL_ORIG_EXTMARKS: Vec<ExtmarkUndoObject> = Vec::new();
    /// The CTRL-X mode a continued completion continues.
    COMPL_CONT_MODE: c_int = 0;
    /// CTRL-X CTRL-V's expansion context, kept between working out the
    /// pattern and expanding it. Moved out for each use: the expansion runs
    /// Lua and user completion functions.
    COMPL_XP: Option<Box<Expand>> = None;
    /// The window the running completion started in. See
    /// [`ins_compl_win_active`]: an identity, not an address, because a
    /// completion runs user functions, autocommands and Lua, any of which
    /// can close the window or wipe the buffer, and comparison is all that
    /// is ever done with it.
    COMPL_CURR_WIN: Option<WinId> = None;
    /// The buffer it started in; see `COMPL_CURR_WIN`.
    COMPL_CURR_BUF: Option<BufId> = None;
    /// The completion is `'autocomplete'`'s.
    COMPL_AUTOCOMPLETE: bool = false;
    /// The time budget of one `'complete'` source.
    COMPL_TIMEOUT_MS: uint64_t = COMPL_INITIAL_TIMEOUT_MS.cast_unsigned();
    /// The current source ran out of time.
    COMPL_TIME_SLICE_EXPIRED: bool = false;
    /// The completion started after a non-keyword character.
    COMPL_FROM_NONKEYWORD: bool = false;
    /// Highlight the text `'autocomplete'`'s `longest` inserted.
    COMPL_HI_ON_AUTOCOMPL_LONGEST: bool = false;
    /// `CONT_*` flags: how a CTRL-X continuation proceeds.
    COMPL_CONT_STATUS: c_int = 0;
    /// The completion function answered `refresh: 'always'`.
    COMPL_OPT_REFRESH_ALWAYS: bool = false;
    /// The length of the bad word spell completion started on.
    SPELL_BAD_LEN: size_t = 0;
    /// The menu entry selected, `-1` for none.
    COMPL_SELECTED_ITEM: c_int = -1;
    /// See [`CptSources`].
    CPT_SOURCES: Vec<CptSource> = Vec::new();
    /// The `'complete'` entry being collected from, `-1` between scans.
    CPT_SOURCES_INDEX: c_int = -1;
    /// How many times [`CPT_SOURCES`] was rebuilt or cleared: a match's
    /// source index is only good for the rows of the generation it was
    /// added under.
    CPT_GENERATION: u64 = 0;
    /// See [`ComplMatchArray`].
    COMPL_MATCH_ARRAY: MatchRows = MatchRows::EMPTY;
    /// See [`CptCallbacks`].
    CPT_CB: Vec<Callback> = Vec::new();
    /// `CompleteChanged` is running: it does not fire again from inside.
    COMPLETE_CHANGED_BUSY: bool = false;
    /// [`ins_compl_check_keys`]'s call count, which it only acts on every
    /// `frequency` calls.
    CHECK_KEYS_COUNT: c_int = 0;
    /// The window [`ins_compl_next_buf`]'s `w` walk is at: a handle that
    /// `win_valid` vets, because it outlives the call.
    NEXT_BUF_WINDOW: Option<WinId> = None;
    /// [`ins_compl_get_exp`]'s scan through `'complete'`, which upstream
    /// keeps in a function-scope static: collected over many calls.
    SCAN: InsComplNextState = INS_COMPL_NEXT_STATE_INIT;
}

/// A `'complete'` source's first time budget, in milliseconds, as
/// `'autocompletetimeout'` counts it.
pub(crate) const COMPL_INITIAL_TIMEOUT_MS: OptInt = 80;
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
