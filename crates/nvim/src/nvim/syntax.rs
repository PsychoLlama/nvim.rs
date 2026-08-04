//! Syntax highlighting: `:syntax` and the state machine behind it.
//!
//! The module is the parent of sixteen children and holds what they share --
//! the item types, the `SYN*`/`SPO_*`/`HL_*` vocabulary, and the statics that
//! are the parser's whole state. Broadly:
//!
//! - **Defining items**: [`command`] (`:syntax` itself and the per-block
//!   modes), [`keyword`], [`define`] (`match`/`region`/`include`), [`cluster`],
//!   [`options`] (the flag words and the `contains=` lists) and [`clear`].
//! - **Parsing**: [`state`] (the driver), [`stack`] (the cache of saved
//!   states), [`sync`] (where parsing may start), [`items`] (the state stack),
//!   [`endpos`] (finding a region's end) and [`attr`] (the per-cell answer).
//! - **Answering**: [`query`], [`list`] (`:syntax list`) and [`syntime`].
//!
//! Two blocks matter and are easy to confuse: [`syn_block`] is the one being
//! *parsed*, and [`cur_syn_block`] -- `curwin`'s -- is the one a `:syntax`
//! command *configures*. Each has its own accessors.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::autocmd::{EVENT_SYNTAX, apply_autocmds};
use crate::src::nvim::buffer::buf_get_changedtick;
use crate::src::nvim::charset::{
    buf_init_chartab, getdigits_int, getdigits_int32, skiptowhite, skipwhite, str_foldcase,
    vim_isprintc, vim_iswordp_buf,
};
use crate::src::nvim::drawscreen::{
    UPD_NOT_VALID, UPD_SOME_VALID, redraw_curbuf_later, redraw_later,
};
use crate::src::nvim::eval::vars::{do_unlet, get_var_value, set_internal_string_var};
use crate::src::nvim::ex_docmd::{
    check_nextcmd, do_cmdline_cmd, ends_excmd, expand_filename, find_nextcmd, separate_nextcmd,
};
use crate::src::nvim::fold::{foldUpdateAll, foldmethodIsSyntax};
use crate::src::nvim::garray::{ga_append_via_ptr, ga_clear, ga_grow, ga_init, ga_set_growsize};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::hashtab::{
    hash_add_item, hash_clear, hash_find, hash_hash, hash_init, hash_lock, hash_lookup,
    hash_remove, hash_unlock,
};
use crate::src::nvim::highlight_group::{
    HLF_D, highlight_group_name, highlight_link_id, highlight_num_groups, init_highlight,
    syn_check_group, syn_id2attr, syn_list_header, syn_name2id, syn_name2id_len,
};
use crate::src::nvim::indent_c::find_start_comment;
use crate::src::nvim::main::{
    Columns, Rows, curbuf, curwin, display_tick, e_invarg2, e_nogroup, e_notopen,
    empty_string_option, emsg_skip, firstwin, got_int, include_default, include_link, include_none,
    msg_col, p_cpo, re_extmatch_in, re_extmatch_out, reg_do_extmatch,
};
use crate::src::nvim::mbyte::{mb_strcmp_ic, utf_head_off, utf_ptr2char, utfc_ptr2len};
use crate::src::nvim::memline::{ml_get, ml_get_buf, ml_get_buf_len, ml_get_len};
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc, xmemcpyz, xstrdup};
use crate::src::nvim::message::{
    emsg, msg, msg_advance, msg_ext_set_kind, msg_outnum, msg_outtrans, msg_outtrans_len,
    msg_putchar, msg_puts, msg_puts_hl, msg_puts_title, semsg,
};
use crate::src::nvim::optionstr::clear_string_option;
use crate::src::nvim::os::input::line_breakcheck;
use crate::src::nvim::os::libc::{
    gettext, memmove, qsort, strcasecmp, strcmp, strcpy, strlen, strncasecmp, strncmp, strpbrk,
};
use crate::src::nvim::path::path_is_absolute;
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::profile::{
    profile_add, profile_cmp, profile_divide, profile_end, profile_msg, profile_start, profile_zero,
};
use crate::src::nvim::regexp::vim_regexec_multi;
use crate::src::nvim::regexp::{ref_extmatch, skip_regexp, unref_extmatch, vim_regcomp_had_eol};
use crate::src::nvim::runtime::{do_source, source_runtime};
use crate::src::nvim::strings::{
    vim_snprintf, vim_strchr, vim_strnsave_up, vim_strsave_up, xstrnsave,
};
use crate::src::nvim::types::{
    OptInt, buf_T, bufstate_T, colnr_T, exarg_T, expand_T, garray_T, hashtab_T, int16_t, linenr_T,
    lpos_T, proftime_T, reg_extmatch_T, regmatch_T, regmmatch_T, regprog_T, size_t, syn_time_T,
    synblock_T, synstate_T, uint8_t, uint32_t, uint64_t, varnumber_T, win_T,
};

mod flags;
pub use self::flags::*;

// The carve of the transpiled module; see each child's docs.
mod state;
pub use self::state::*;
mod stack;
pub use self::stack::*;
mod sync;
pub(crate) use self::sync::*;
mod attr;
pub use self::attr::*;
mod items;
pub(crate) use self::items::*;
mod endpos;
pub(crate) use self::endpos::*;
mod command;
pub use self::command::*;
mod clear;
pub use self::clear::*;
mod list;
pub(crate) use self::list::*;
mod keyword;
pub(crate) use self::keyword::*;
mod define;
pub(crate) use self::define::*;
mod cluster;
pub(crate) use self::cluster::*;
mod options;
pub(crate) use self::options::*;
mod query;
pub use self::query::*;
mod syntime;
pub use self::syntime::*;

unsafe extern "C" {
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
}
/// How many `\(..\)` submatches a pattern can have.
pub const NSUBEXP: ::core::ffi::c_uint = 10;
/// Size of `expand_T::xp_buf`, the scratch buffer a completion callback may
/// answer from.
pub const EXPAND_BUF_LEN: ::core::ffi::c_uint = 256;
// The `expand_T::xp_context` values this module sets.
pub const EXPAND_NOTHING: ::core::ffi::c_int = 0;
pub const EXPAND_SYNTAX: ::core::ffi::c_int = 12;
pub const EXPAND_HIGHLIGHT: ::core::ffi::c_int = 13;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sp_syn {
    pub inc_tag: ::core::ffi::c_int,
    pub id: int16_t,
    pub cont_in_list: *mut int16_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct keyentry {
    pub ke_next: *mut keyentry_T,
    pub k_syn: sp_syn,
    pub next_list: *mut int16_t,
    pub flags: ::core::ffi::c_int,
    pub k_char: ::core::ffi::c_int,
    pub keyword: [::core::ffi::c_char; 0],
}
pub type keyentry_T = keyentry;
/// The highest highlight id there can be.
pub const MAX_HL_ID: ::core::ffi::c_uint = 20000;
/// The `contains=ALL`/`ALLBUT` marker, which shares its value with the highest
/// possible highlight id and is offset by the `:syntax include` tag.
pub(crate) const SYNID_ALLBUT: ::core::ffi::c_int = MAX_HL_ID as ::core::ffi::c_int;
/// `do_source` flag: this is not a plugin or a package.
pub const DOSO_NONE: ::core::ffi::c_uint = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stateitem_T {
    pub si_idx: ::core::ffi::c_int,
    pub si_id: ::core::ffi::c_int,
    pub si_trans_id: ::core::ffi::c_int,
    pub si_m_lnum: ::core::ffi::c_int,
    pub si_m_startcol: ::core::ffi::c_int,
    pub si_m_endpos: lpos_T,
    pub si_h_startpos: lpos_T,
    pub si_h_endpos: lpos_T,
    pub si_eoe_pos: lpos_T,
    pub si_end_idx: ::core::ffi::c_int,
    pub si_ends: ::core::ffi::c_int,
    pub si_attr: ::core::ffi::c_int,
    pub si_flags: ::core::ffi::c_int,
    pub si_seqnr: ::core::ffi::c_int,
    pub si_cchar: ::core::ffi::c_int,
    pub si_cont_list: *mut int16_t,
    pub si_next_list: *mut int16_t,
    pub si_extmatch: *mut reg_extmatch_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct synpat_T {
    pub sp_type: ::core::ffi::c_char,
    pub sp_syncing: bool,
    pub sp_syn_match_id: int16_t,
    pub sp_off_flags: int16_t,
    pub sp_offsets: [::core::ffi::c_int; 7],
    pub sp_flags: ::core::ffi::c_int,
    pub sp_cchar: ::core::ffi::c_int,
    pub sp_ic: ::core::ffi::c_int,
    pub sp_sync_idx: ::core::ffi::c_int,
    pub sp_line_id: ::core::ffi::c_int,
    pub sp_startcol: ::core::ffi::c_int,
    pub sp_cont_list: *mut int16_t,
    pub sp_next_list: *mut int16_t,
    pub sp_syn: sp_syn,
    pub sp_pattern: *mut ::core::ffi::c_char,
    pub sp_prog: *mut regprog_T,
    pub sp_time: syn_time_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct syn_cluster_T {
    pub scl_name: *mut ::core::ffi::c_char,
    pub scl_name_u: *mut ::core::ffi::c_char,
    pub scl_list: *mut int16_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct syn_opt_arg_T {
    pub flags: ::core::ffi::c_int,
    pub keyword: bool,
    pub sync_idx: *mut ::core::ffi::c_int,
    pub has_cont_list: bool,
    pub cont_list: *mut int16_t,
    pub cont_in_list: *mut int16_t,
    pub next_list: *mut int16_t,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL,
};
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const EX_XFILE: ::core::ffi::c_uint = 0x8 as ::core::ffi::c_uint;
pub const EX_NOSPC: ::core::ffi::c_uint = 0x10 as ::core::ffi::c_uint;
pub const SYNSPL_DEFAULT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SYNSPL_TOP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SYNSPL_NOTOP: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SYNFLD_START: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SYNFLD_MINIMUM: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
/// The `:source` argument the on/off commands build; `%s` is the file's name.
pub(crate) const SYNTAX_FNAME: &::core::ffi::CStr = c"$VIMRUNTIME/syntax/%s.vim";
pub const SST_MIN_ENTRIES: ::core::ffi::c_int = 150 as ::core::ffi::c_int;
pub const SST_MAX_ENTRIES: ::core::ffi::c_int = 1000 as ::core::ffi::c_int;
pub const SST_FIX_STATES: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const SST_DIST: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
/// Whether `:syntax on|off|enable|manual` has been used, which is what stops
/// [`syn_maybe_enable`] from overriding a deliberate choice.
static did_syntax_onoff: GlobalCell<bool> = GlobalCell::new(false);
pub const SPO_MS_OFF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SPO_ME_OFF: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SPO_HS_OFF: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SPO_HE_OFF: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const SPO_RS_OFF: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const SPO_RE_OFF: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const SPO_LC_OFF: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const SPO_COUNT: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub(crate) const E_ILLEGAL_ARG: &::core::ffi::CStr = c"E390: Illegal argument: %s";
pub(crate) const E_CONTAINS_NOT_ACCEPTED_HERE: &::core::ffi::CStr =
    c"E395: Contains argument not accepted here";
pub(crate) const E_INVALID_CCHAR_VALUE: &::core::ffi::CStr = c"E844: Invalid cchar value";
/// `%s` is the text before the `]`, `%s` the text after it.
pub(crate) const E_TRAILING_CHAR_AFTER_RSB: &::core::ffi::CStr =
    c"E890: Trailing char after ']': %s]%s";
pub const SPTYPE_MATCH: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SPTYPE_START: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SPTYPE_END: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const SPTYPE_SKIP: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const NONE_IDX: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const SF_CCOMMENT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const SF_MATCH: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MAXKEYWLEN: ::core::ffi::c_int = 80 as ::core::ffi::c_int;
// What the last `syn_current_attr` decided about the current position. The
// query API reads these back, so they outlive the call that set them.

/// Attribute number of the current character.
static current_attr: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
/// Syntax id of the current character, before transparency.
static current_id: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
/// Syntax id of the current character, after transparency.
static current_trans_id: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
/// `HL_*` flags of the current character.
static current_flags: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
/// Sequence number of the item the current character belongs to, which is what
/// tells two runs of the same group apart.
static current_seqnr: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
/// The `cchar=` of the current character, for `conceal`.
static current_sub_char: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub const CLUSTER_REPLACE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const CLUSTER_ADD: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const CLUSTER_SUBTRACT: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const SYNID_TOP: ::core::ffi::c_int = 21000 as ::core::ffi::c_int;
pub const SYNID_CONTAINED: ::core::ffi::c_int = 22000 as ::core::ffi::c_int;
pub const SYNID_CLUSTER: ::core::ffi::c_int = 23000 as ::core::ffi::c_int;
pub const MAX_SYN_INC_TAG: ::core::ffi::c_int = 999 as ::core::ffi::c_int;
pub const MAX_CLUSTER_ID: ::core::ffi::c_int = 32767 as ::core::ffi::c_int - SYNID_CLUSTER;
/// The `:syntax` command line being executed, which `:syntax include` needs to
/// expand a file name against.
static syn_cmdlinep: GlobalCell<*mut *mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut());
/// The `:syntax include` nesting tag items are being defined under; 0 outside
/// an inclusion.
static current_syn_inc_tag: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
/// The highest tag handed out so far, capped at [`MAX_SYN_INC_TAG`].
static running_syn_inc_tag: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
/// Stack index of the outermost `keepend` item currently in effect, or -1.
static keepend_level: GlobalCell<::core::ffi::c_int> = GlobalCell::new(-1);
/// What every `:syntax`/`:syntime` listing answers when there is nothing to
/// list.
pub(crate) const MSG_NO_ITEMS: &::core::ffi::CStr = c"No Syntax items defined for this buffer";

/// The syntax block being *configured* — `curwin`'s, which during a `:syntax`
/// command is not necessarily [`syn_block`], the one being *parsed*.
#[inline]
pub(crate) unsafe fn cur_syn_block() -> *mut synblock_T {
    unsafe { (*curwin.get()).w_s }
}

/// The `synpat_T` at `idx` in [`cur_syn_block`]'s pattern array.
///
/// An accessor and not a borrow because every `:syntax` command that adds a
/// pattern can `ga_grow` the array out from under one.
#[inline]
pub(crate) unsafe fn cur_pattern(idx: ::core::ffi::c_int) -> *mut synpat_T {
    unsafe { ((*cur_syn_block()).b_syn_patterns.ga_data as *mut synpat_T).offset(idx as isize) }
}

/// Number of patterns in [`cur_syn_block`].
#[inline]
pub(crate) unsafe fn cur_pattern_count() -> ::core::ffi::c_int {
    unsafe { (*cur_syn_block()).b_syn_patterns.ga_len }
}

/// The `syn_cluster_T` at `idx` in [`cur_syn_block`]'s cluster array.
#[inline]
pub(crate) unsafe fn cur_cluster(idx: ::core::ffi::c_int) -> *mut syn_cluster_T {
    unsafe {
        ((*cur_syn_block()).b_syn_clusters.ga_data as *mut syn_cluster_T).offset(idx as isize)
    }
}

/// Number of clusters in [`cur_syn_block`].
#[inline]
pub(crate) unsafe fn cur_cluster_count() -> ::core::ffi::c_int {
    unsafe { (*cur_syn_block()).b_syn_clusters.ga_len }
}
/// `stateitem_T::si_idx` for a keyword, which has no pattern.
pub const KEYWORD_IDX: ::core::ffi::c_int = -1;
/// The `contains=` list of a transparent item that is not inside anything: it
/// admits every not-`contained` group.
pub const ID_LIST_ALL: *mut int16_t = -1 as ::core::ffi::c_int as *mut int16_t;
/// The sequence number the next pushed item gets.
static next_seqnr: GlobalCell<::core::ffi::c_int> = GlobalCell::new(1);

// The match `syn_current_attr` found ahead of the current column and has not
// pushed yet. `next_match_col` is MAXCOL for "nothing found" and -1 for "not
// looked yet".

/// Column the pending match starts at.
static next_match_col: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static next_match_m_endpos: GlobalCell<lpos_T> = GlobalCell::new(lpos_T { lnum: 0, col: 0 });
static next_match_h_startpos: GlobalCell<lpos_T> = GlobalCell::new(lpos_T { lnum: 0, col: 0 });
static next_match_h_endpos: GlobalCell<lpos_T> = GlobalCell::new(lpos_T { lnum: 0, col: 0 });
static next_match_idx: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static next_match_flags: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static next_match_eos_pos: GlobalCell<lpos_T> = GlobalCell::new(lpos_T { lnum: 0, col: 0 });
static next_match_eoe_pos: GlobalCell<lpos_T> = GlobalCell::new(lpos_T { lnum: 0, col: 0 });
static next_match_end_idx: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static next_match_extmatch: GlobalCell<*mut reg_extmatch_T> =
    GlobalCell::new(::core::ptr::null_mut::<reg_extmatch_T>());
// Where the parser currently is. `syntax_start` sets the first four together
// and everything else is relative to them.

/// The window being parsed for.
static syn_win: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut());
/// The buffer being parsed.
static syn_buf: GlobalCell<*mut buf_T> = GlobalCell::new(::core::ptr::null_mut());
/// The syntax block being parsed -- `syn_win`'s, which for `:ownsyntax` is not
/// the buffer's.
static syn_block: GlobalCell<*mut synblock_T> = GlobalCell::new(::core::ptr::null_mut());
/// When parsing must give up, or NULL for no limit.
static syn_tm: GlobalCell<*mut proftime_T> = GlobalCell::new(::core::ptr::null_mut());
/// The line being parsed.
static current_lnum: GlobalCell<linenr_T> = GlobalCell::new(0);
/// The column being parsed.
static current_col: GlobalCell<colnr_T> = GlobalCell::new(0);
/// Whether the state at `current_lnum` has been put in the cache.
static current_state_stored: GlobalCell<bool> = GlobalCell::new(false);
/// Whether the line has been parsed to its end.
static current_finished: GlobalCell<bool> = GlobalCell::new(false);
/// The state stack: a `garray_T` of [`stateitem_T`], outermost item first.
static current_state: GlobalCell<garray_T> = GlobalCell::new(GA_EMPTY_INIT_VALUE);
/// The `nextgroup=` list in effect, or NULL.
static current_next_list: GlobalCell<*mut int16_t> = GlobalCell::new(::core::ptr::null_mut());
/// The `skipwhite`/`skipnl`/`skipempty` flags that came with it.
static current_next_flags: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
/// `display_tick` when the current line was parsed, which is how a `display`
/// item knows it is being drawn rather than scanned.
static current_line_id: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
/// Whether `:syntime on` is in effect.
static syn_time_on: GlobalCell<bool> = GlobalCell::new(false);

/// Set the time limit for parsing, or clear it with NULL.
pub unsafe fn syn_set_timeout(tm: *mut proftime_T) {
    syn_tm.set(tm);
}
pub const ITEM_START: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const ITEM_SKIP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const ITEM_END: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const ITEM_MATCHGROUP: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const REX_SET: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const REX_USE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
