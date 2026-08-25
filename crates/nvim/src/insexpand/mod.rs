#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_uint, c_void};
use core::ptr;

use crate::api::private::helpers::{cbuf_to_string, copy_string, cstr_as_string};
use crate::ascii::{ascii_isdigit, ascii_iswhite, ascii_iswhite_or_nul};
use crate::autocmd::{
    EVENT_COMPLETECHANGED, EVENT_COMPLETEDONE, EVENT_COMPLETEDONEPRE, apply_autocmds, has_event,
};
use crate::buffer::{buf_spname, buf_valid};
use crate::change::{
    deleted_lines_mark, ins_bytes_len, ins_char, ins_char_bytes, ins_str, open_line,
};
use crate::charset::{
    getwhitecols, ptr2cells, skipwhite, str_foldcase, vim_is_ident_char, vim_isfilec, vim_isprintc,
    vim_iswordc, vim_iswordp, vim_strsize,
};
use crate::cmdexpand::{addstar, expand_cmdline, set_cmd_context};
use crate::cursor::{
    check_cursor, dec_cursor, get_cursor_line_len, get_cursor_line_ptr, get_cursor_pos_len,
    get_cursor_pos_ptr, inc_cursor,
};
use crate::drawscreen::{
    UPD_VALID, redraw_later, redraw_win_line, setcursor, showmode, update_screen,
};
use crate::edit::{
    backspace_until_column, get_can_cindent, ins_apply_autocmds, ins_eol, ins_need_undo_get,
    ins_redraw, insertchar, start_arrow, stop_arrow,
};
use crate::eval::typval::{
    callback_copy, callback_free, kCallbackNone, tv_clear, tv_dict_add_bool, tv_dict_add_dict,
    tv_dict_add_list, tv_dict_add_nr, tv_dict_add_str, tv_dict_add_str_len, tv_dict_add_tv,
    tv_dict_alloc, tv_dict_alloc_lock, tv_dict_alloc_ret, tv_dict_find, tv_dict_get_number,
    tv_dict_get_string, tv_dict_get_tv, tv_dict_set_keys_readonly, tv_dict_unref,
    tv_get_number_chk, tv_get_string, tv_get_string_chk, tv_list_alloc, tv_list_append_dict,
    tv_list_first, tv_list_unref,
};
use crate::eval::userfunc::callback_call_retnr;
use crate::eval::vars::set_vim_var_dict;
use crate::eval::{callback_call, get_v_event, restore_v_event, set_ref_in_callback};
use crate::ex_eval::aborting;
use crate::ex_getln::tilde_replace;
use crate::extmark::{extmark_apply_undo, extmark_splice_delete};
use crate::fileio::vim_fgets;
use crate::fuzzy::{fuzzy_match_str, fuzzy_match_str_in_line, search_for_fuzzy_match};
use crate::garray::{ga_clear, ga_grow, ga_init};
use crate::getchar::{
    append_to_redobuff_char, append_to_redobuff_literally, char_avail, safe_vgetc, using_script,
    vgetc, vpeekc, vpeekc_any, vungetc,
};
use crate::global_cell::GlobalCell;
use crate::highlight_group::{HLF_COUNT, HLF_E, HLF_R, HLF_W, syn_name2attr};
use crate::indent::{get_indent, inindent};
use crate::indent_c::{cindent_on, do_c_expr_indent, in_cinkeys};
use crate::keycodes::{
    K_BS, K_COMMAND, K_DOWN, K_EVENT, K_IGNORE, K_KENTER, K_KPAGEDOWN, K_KPAGEUP, K_LUA,
    K_MOUSEDOWN, K_MOUSELEFT, K_MOUSEMOVE, K_MOUSERIGHT, K_MOUSEUP, K_PAGEDOWN, K_PAGEUP, K_S_DOWN,
    K_S_TAB, K_S_UP, K_SELECT, K_UP,
};
use crate::lua::executor::nlua_expand_pat;
use crate::main::{
    KeyTyped, State, arrow_used, can_si, can_si_back, cmdwin_type, cot_flags, curbuf, curwin,
    did_ai, did_emsg, did_si, dollar_vcol, e_invarg, e_list_index_out_of_range_nr, e_listreq,
    e_notset, e_patnotf, edit_submode, edit_submode_extra, edit_submode_highl, edit_submode_pre,
    emsg_silent, ex_normal_busy, firstbuf, firstwin, g_tag_at_cursor, global_busy, got_int,
    in_assert_fails, msg_hist_off, p_ac, p_acl, p_act, p_cto, p_dict, p_fic, p_ic, p_inf, p_js,
    p_paste, p_scs, p_smd, p_tsr, p_tsrfu, p_wic, p_ws, pum_want, redraw_cmdline, redraw_mode,
    sc_col, test_disable_char_avail,
};
use crate::mbyte::{
    mb_get_class, mb_islower, mb_isupper, mb_prevptr, mb_ptr2char_adv, mb_tolower, mb_toupper,
    utf_char2bytes, utf_char2len, utf_head_off, utf_ptr2char, utf_ptr2len, utf8len_tab,
    utfc_ptr2len,
};
use crate::memline::{dec, ml_delete, ml_get, ml_get_buf, ml_get_buf_len, ml_get_len};
use crate::memory::{
    MergeSortCompareFunc, mergesort_list, strequal, xcalloc, xfree, xmalloc, xmemdupz, xstrdup,
    xstrlcpy,
};
use crate::message::{
    emsg, internal_error, msg, msg_clr_cmdline, msg_delay, msg_ext_set_kind, msg_progress,
};
use crate::r#move::{changed_cline_bef_curs, curs_columns, validate_cursor};
use crate::option::{can_bs, copy_option_part, magic_isset, option_set_callback_func, shortmess};
use crate::options::{
    kOptBoFlagComplete, kOptCotFlagFuzzy, kOptCotFlagLongest, kOptCotFlagMenu, kOptCotFlagMenuone,
    kOptCotFlagNearest, kOptCotFlagNoinsert, kOptCotFlagNoselect, kOptCotFlagNosort,
    kOptCotFlagPreinsert,
};
use crate::os::cshim::{gettext, memmove, strchr, strncasecmp, strncmp};
use crate::os::fs::os_fopen;
use crate::os::input::{fast_breakcheck, line_breakcheck, os_breakcheck};
use crate::os::time::{os_delay, os_hrtime};
use crate::path::{expand_wildcards, free_wild, path_tail, vim_ispathsep};
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
use crate::state::{MODE_INSERT, REPLACE_FLAG, may_trigger_modechanged};
use crate::strings::{vim_snprintf, vim_strchr, vim_strsave_escaped};
use crate::tag::find_tags;
use crate::textformat::auto_format;
use crate::types::{
    Arena, BackslashEscape, BoolVarValue, Callback, Callback_data, Direction, EvalFuncData,
    ExpandContext, ExtmarkOp, MB_MAXCHAR, OptInt, String_0, VAR_UNKNOWN, VAR_UNLOCKED, Vv, buf_T,
    colnr_T, dict_T, expand_T, extmark_undo_vec_t, garray_T, hashitem_T, hashtab_T, linenr_T,
    list_T, optset_T, pos_T, ptrdiff_t, pumitem_T, regmatch_T, save_v_event_T, sctx_T, size_t,
    typval_T, typval_vval_union, uint8_t, uint64_t, varnumber_T, win_T, xp_prefix_T,
};
use crate::ui::{ui_flush, vim_beep};
use crate::undo::undo_allowed;
use crate::window::win_valid;
use crate::winfloat::win_float_find_preview;
use ::libc::{abs, atoi, fclose, memcmp, qsort, strcat, strcmp, strcpy, strlen, strncpy, strrchr};

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
pub const kDirectionNotSet: Direction = 0;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
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
pub type compl_T = compl_S;
#[derive(Copy, Clone)]
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
pub struct cpt_source_T {
    pub cs_refresh_always: bool,
    pub cs_startcol: ::core::ffi::c_int,
    pub cs_max_matches: ::core::ffi::c_int,
    pub compl_start_tv: uint64_t,
    pub cs_flag: ::core::ffi::c_char,
}
/// A zeroed `cpt_source_T`, which is what `xcalloc` left every row as.
pub(crate) const CPT_SOURCE_INIT: cpt_source_T = cpt_source_T {
    cs_refresh_always: false,
    cs_startcol: 0,
    cs_max_matches: 0,
    compl_start_tv: 0,
    cs_flag: 0,
};
pub const CP_EQUAL: ::core::ffi::c_int = 8;
#[derive(Copy, Clone)]
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
/// A zeroed `garray_T`, which `ga_init` then fills in.
pub(crate) const GARRAY_T_INIT: garray_T = garray_T {
    ga_len: 0,
    ga_maxlen: 0,
    ga_itemsize: 0,
    ga_growsize: 0,
    ga_data: ptr::null_mut(),
};
/// A zeroed `ins_compl_next_state_T`: C's `CLEAR_FIELD(st)`.
pub(crate) const INS_COMPL_NEXT_STATE_INIT: ins_compl_next_state_T = ins_compl_next_state_T {
    e_cpt_copy: ptr::null_mut(),
    e_cpt: ptr::null_mut(),
    ins_buf: ptr::null_mut(),
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
/// An unset `typval_T`, which the transpile writes out at every declaration
/// (C leaves these uninitialised and has the callee fill them in).
pub(crate) const TYPVAL_T_INIT: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VAR_UNLOCKED,
    vval: typval_vval_union { v_number: 0 },
};

/// A zeroed `pos_T`.
pub(crate) const POS_T_INIT: pos_T = pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
};
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
/// A zeroed `extmark_undo_vec_t`, which is what C's `kv_destroy` leaves.
pub(crate) const EXTMARK_UNDO_VEC_INIT: extmark_undo_vec_t = extmark_undo_vec_t {
    size: 0,
    capacity: 0,
    items: ptr::null_mut(),
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
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
/// rows answer a null pointer here rather than panicking; no caller reaches
/// one (the three modes without a message never take these paths).
pub(crate) fn ctrl_x_msg(mode: c_int) -> *mut c_char {
    match CTRL_X_MSGS[(mode & !CTRL_X_WANT_IDENT) as usize] {
        // SAFETY: a `CStr` constant is a valid NUL-terminated string.
        Some(msg) => unsafe { gettext(msg.as_ptr()) },
        None => ptr::null_mut(),
    }
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
/// pointing into it, so every read copies the two words out and no reference
/// into the global is ever formed — which matters because completion runs
/// user callbacks, and a callback can reach the same string. The *bytes* are
/// still handed out raw, because every consumer of them is C-shaped; they
/// stay valid until the next [`set`](ComplStr::set) or
/// [`replace`](ComplStr::replace), exactly as upstream's did.
#[derive(Clone, Copy)]
pub(crate) struct ComplStr(&'static GlobalCell<String_0>);

impl ComplStr {
    /// The two words by value.
    pub(crate) fn value(self) -> String_0 {
        self.0.get()
    }

    /// The bytes, or null while the string is unset.
    pub(crate) fn data(self) -> *mut c_char {
        self.value().data()
    }

    /// The byte count.
    pub(crate) fn len(self) -> size_t {
        self.value().len()
    }

    /// Whether the string has no buffer at all — upstream's
    /// `if (compl_leader.data == NULL)`, which asks something different from
    /// [`is_empty`](Self::is_empty).
    pub(crate) fn is_unset(self) -> bool {
        self.value().is_null()
    }

    /// Whether the string has no bytes.
    pub(crate) fn is_empty(self) -> bool {
        self.value().is_empty()
    }

    /// Point at `s`. What was there is *not* freed: this is the fresh-start
    /// shape, where the string was cleared before the new value was built.
    pub(crate) fn set(self, s: String_0) {
        self.0.set(s);
    }

    /// C's `XFREE_CLEAR(x.data); x = s`: free this string's bytes, then take
    /// `s`.
    pub(crate) fn replace(self, s: String_0) {
        self.free_bytes();
        self.0.set(s);
    }

    /// C's `XFREE_CLEAR(s->data); s->size = 0`.
    pub(crate) fn clear(self) {
        self.replace(String_0::NULL);
    }

    /// C's `XFREE_CLEAR(compl_leader)` written on the *struct* rather than on
    /// its `.data`: the bytes go and the pointer nulls, but the length is
    /// left stale. Reproduced deliberately for `ins_compl_build_pum`; every
    /// reader guards on the pointer.
    pub(crate) fn free_bytes_keep_len(self) {
        let stale = self.len();
        self.free_bytes();
        self.0.set(String_0::from_raw_parts(ptr::null_mut(), stale));
    }

    /// Point at `data`, keeping the length — the two-step build in
    /// `get_normal_compl_info`, which sizes the pattern before it fills it.
    pub(crate) fn set_data(self, data: *mut c_char) {
        let mut s = self.value();
        s.set_data(data);
        self.0.set(s);
    }

    /// Set the length, keeping the pointer.
    pub(crate) fn set_len(self, len: size_t) {
        let mut s = self.value();
        s.set_len(len);
        self.0.set(s);
    }

    /// Free the bytes, leaving the two words alone.
    fn free_bytes(self) {
        // SAFETY: the bytes are this string's own, and `xfree` takes null.
        unsafe { xfree(self.data().cast::<c_void>()) };
    }
}

/// What the current completion searches for.
pub(crate) fn compl_pattern() -> ComplStr {
    ComplStr(&COMPL_PATTERN)
}

/// The `'complete'` source's own pattern, when its startcol differs from
/// `compl_col`.
pub(crate) fn cpt_compl_pattern() -> ComplStr {
    ComplStr(&CPT_COMPL_PATTERN)
}

/// What the user has typed since the completion started, which filters the
/// matches. Unset until the first `ins_compl_addleader`.
pub(crate) fn compl_leader() -> ComplStr {
    ComplStr(&COMPL_LEADER)
}

/// The text that was under the cursor when the completion started, and which
/// CTRL-E puts back.
pub(crate) fn compl_orig_text() -> ComplStr {
    ComplStr(&COMPL_ORIG_TEXT)
}

/// [`compl_leader`] with the text a source's earlier startcol covers
/// prepended; the cache behind [`get_leader_for_startcol`].
pub(crate) fn adjusted_leader() -> ComplStr {
    ComplStr(&ADJUSTED_LEADER)
}

/// C's `e_hitend`.
pub(crate) const E_HITEND: &CStr = c"Hit end of paragraph";

/// C's `e_compldel`.
pub(crate) const E_COMPLDEL: &CStr = c"E840: Completion function deleted text";

static compl_first_match: GlobalCell<*mut compl_T> =
    GlobalCell::new(::core::ptr::null_mut::<compl_T>());
static compl_curr_match: GlobalCell<*mut compl_T> =
    GlobalCell::new(::core::ptr::null_mut::<compl_T>());
static compl_shown_match: GlobalCell<*mut compl_T> =
    GlobalCell::new(::core::ptr::null_mut::<compl_T>());
static compl_old_match: GlobalCell<*mut compl_T> =
    GlobalCell::new(::core::ptr::null_mut::<compl_T>());
static compl_num_bests: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static compl_enter_selects: GlobalCell<bool> = GlobalCell::new(false);
static COMPL_LEADER: GlobalCell<String_0> = GlobalCell::new(String_0::NULL);
static ADJUSTED_LEADER: GlobalCell<String_0> = GlobalCell::new(String_0::NULL);
static compl_get_longest: GlobalCell<bool> = GlobalCell::new(false);
static compl_used_match: GlobalCell<bool> = GlobalCell::new(false);
static compl_was_interrupted: GlobalCell<bool> = GlobalCell::new(false);
static compl_interrupted: GlobalCell<bool> = GlobalCell::new(false);
static compl_restarting: GlobalCell<bool> = GlobalCell::new(false);
static compl_started: GlobalCell<bool> = GlobalCell::new(false);
static ctrl_x_mode: GlobalCell<::core::ffi::c_int> = GlobalCell::new(CTRL_X_NORMAL);
static compl_matches: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static COMPL_PATTERN: GlobalCell<String_0> = GlobalCell::new(String_0::NULL);
static CPT_COMPL_PATTERN: GlobalCell<String_0> = GlobalCell::new(String_0::NULL);
static compl_direction: GlobalCell<Direction> = GlobalCell::new(FORWARD);
static compl_shows_dir: GlobalCell<Direction> = GlobalCell::new(FORWARD);
static compl_pending: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static compl_startpos: GlobalCell<pos_T> = GlobalCell::new(POS_T_INIT);
static compl_length: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static compl_lnum: GlobalCell<linenr_T> = GlobalCell::new(0 as linenr_T);
static compl_col: GlobalCell<colnr_T> = GlobalCell::new(0 as colnr_T);
static compl_ins_end_col: GlobalCell<colnr_T> = GlobalCell::new(0 as colnr_T);
static COMPL_ORIG_TEXT: GlobalCell<String_0> = GlobalCell::new(String_0::NULL);
static COMPL_ORIG_EXTMARKS: GlobalCell<extmark_undo_vec_t> = GlobalCell::new(EXTMARK_UNDO_VEC_INIT);
static compl_cont_mode: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static compl_xp: GlobalCell<expand_T> = GlobalCell::new(expand_T {
    xp_pattern: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    xp_context: ExpandContext::Nothing,
    xp_pattern_len: 0,
    xp_prefix: XP_PREFIX_NONE,
    xp_arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    xp_luaref: 0,
    xp_script_ctx: sctx_T::NONE,
    xp_backslash: BackslashEscape::NONE,
    xp_shell: false,
    xp_numfiles: 0,
    xp_col: 0,
    xp_selected: 0,
    xp_orig: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    xp_files: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    xp_line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    xp_buf: [0; 1025],
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
static compl_autocomplete: GlobalCell<bool> = GlobalCell::new(false);
static compl_timeout_ms: GlobalCell<uint64_t> =
    GlobalCell::new(COMPL_INITIAL_TIMEOUT_MS as uint64_t);
static compl_time_slice_expired: GlobalCell<bool> = GlobalCell::new(false);
static compl_from_nonkeyword: GlobalCell<bool> = GlobalCell::new(false);
static compl_hi_on_autocompl_longest: GlobalCell<bool> = GlobalCell::new(false);
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
static compl_opt_refresh_always: GlobalCell<bool> = GlobalCell::new(false);
static spell_bad_len: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
static compl_selected_item: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(-1 as ::core::ffi::c_int);
static compl_fuzzy_scores: GlobalCell<*mut ::core::ffi::c_int> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_int>());
static CPT_SOURCES: GlobalCell<*mut cpt_source_T> =
    GlobalCell::new(::core::ptr::null_mut::<cpt_source_T>());
static CPT_SOURCES_COUNT: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static CPT_SOURCES_INDEX: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(-1 as ::core::ffi::c_int);
static COMPL_MATCH_ARRAY: GlobalCell<*mut pumitem_T> =
    GlobalCell::new(::core::ptr::null_mut::<pumitem_T>());
static COMPL_MATCH_ARRAYSIZE: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub const DICT_FIRST: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const DICT_EXACT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
static CFU_CB: GlobalCell<Callback> = GlobalCell::new(Callback {
    data: Callback_data {
        funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    type_0: kCallbackNone,
});
static OFU_CB: GlobalCell<Callback> = GlobalCell::new(Callback {
    data: Callback_data {
        funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    type_0: kCallbackNone,
});
static TSRFU_CB: GlobalCell<Callback> = GlobalCell::new(Callback {
    data: Callback_data {
        funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    type_0: kCallbackNone,
});
static CPT_CB: GlobalCell<*mut Callback> = GlobalCell::new(::core::ptr::null_mut::<Callback>());
static CPT_CB_COUNT: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub const CI_WHAT_MODE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CI_WHAT_PUM_VISIBLE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CI_WHAT_ITEMS: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CI_WHAT_SELECTED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CI_WHAT_COMPLETED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CI_WHAT_MATCHES: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const CI_WHAT_PREINSERTED_TEXT: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CI_WHAT_ALL: ::core::ffi::c_int = 0xff as ::core::ffi::c_int;
pub const MIN_SPACE: ::core::ffi::c_int = 75 as ::core::ffi::c_int;
