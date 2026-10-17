//! Syntax highlighting: `:syntax` and the state machine behind it.
//!
//! The module is the parent of sixteen children and holds what they share --
//! the item types and the `SYN*`/`SPO_*`/`HL_*` vocabulary. Broadly:
//!
//! - **Defining items**: [`command`] (`:syntax` itself and the per-block
//!   modes), [`keyword`], [`define`] (`match`/`region`/`include`), [`cluster`],
//!   [`options`] (the flag words and the `contains=` lists) and [`clear`].
//! - **Parsing**: [`state`] (the driver), [`stack`] (the cache of saved
//!   states), [`sync`] (where parsing may start), [`items`] (the state stack),
//!   [`endpos`] (finding a region's end) and [`attr`] (the per-cell answer).
//! - **Answering**: [`query`], [`list`] (`:syntax list`) and [`syntime`].
//!
//! Two blocks matter and are easy to confuse: [`SynState::block`] is the one
//! being *parsed*, and [`cur_syn_block`] -- `curwin`'s -- is the one a
//! `:syntax` command *configures*. Each has its own accessors.
//!
//! The parser's whole state is one [`SynState`], reached through
//! [`with_parser`]; the per-line and per-column steps are its methods.
//!
//! # Ownership
//!
//! Everything a syntax block holds is owned by value: `b_syn_patterns` and
//! `b_syn_clusters` are `Vec`s of [`SynPat`]/[`SynCluster`], a pattern's
//! text is a `CString` and its `contains=`/`containedin=`/`nextgroup=` lists
//! are [`IdList`]s; a keyword is a [`KeyEntry`] inside its block's
//! [`KeywordTable`]. **An `xfree` in this module is a bug.**

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::ascii::ascii_iswhite;
use crate::autocmd::apply_autocmds_exarg;
use crate::buffer::buf_get_changedtick;
use crate::charset::{buf_init_chartab, getdigits_int_at, skip, vim_isprintc};
use crate::cstr;
use crate::drawscreen::state::display_tick;
use crate::drawscreen::{UPD_NOT_VALID, UPD_SOME_VALID, redraw_curbuf_later, redraw_later};
use crate::eval::vars::{do_unlet, set_internal_string_var, var_string_value};
use crate::ex_docmd::{do_cmdline_cmd, ends_excmd, expand_filename, separate_nextcmd};
use crate::fold::{fold_update_all, foldmethod_is_syntax};
use crate::getchar::state::got_int;
use crate::global_cell::GlobalCell;
use crate::highlight::state::{include_default, include_link, include_none};
use crate::highlight_group::{
    HLF_D, highlight_link_id, highlight_num_groups, init_highlight, syn_check_group, syn_id2attr,
    syn_id2name, syn_list_header, syn_name2id, syn_name2id_bytes,
};
use crate::indent_c::find_start_comment;
use crate::mbyte::strcmp_ic_in;
use crate::memline::{Lines, ml_get_buf_len};
use crate::message::state::msg_col;
use crate::message::{
    emsg, msg, msg_advance, msg_display, msg_display_bytes, msg_ext_set_kind, msg_outnum,
    msg_putchar, msg_str, msg_str_hl, msg_title,
};
use crate::winlayer::BufId;
use crate::winlayer::WinId;

use crate::os::cshim::gettext;
use crate::os::input::line_breakcheck;
use crate::path::path_is_absolute;
use crate::pos::MAXLNUM;
use crate::profile::{
    profile_add, profile_cmp, profile_divide, profile_end, profile_msg, profile_start, profile_zero,
};
use crate::regexp::{
    ExtMatch, ExtMatchIo, ExtMatchRef, OwnedProg, vim_regcomp, vim_regcomp_ext, vim_regexec,
    vim_regexec_syntax,
};
use crate::runtime::{do_source, source_runtime};
use crate::types::AutoEvent;
use crate::types::{
    ColNr, ExArg, Expand, HashTab, LPos, LineNr, OptInt, ProfTime, RegMMatch, RegMatch, SynBlock,
    SynTime, int16_t, uint8_t, uint64_t,
};
use crate::ui::state::{Columns, Rows};
use crate::winlayer::Buf;
use crate::winlayer::{Live, Win};
use ::std::rc::Rc;

mod flags;
pub(crate) use self::flags::*;
mod parser;
pub(crate) use self::parser::*;

// The carve of the transpiled module; see each child's docs.
mod state;
pub(crate) use self::state::*;
mod stack;
pub(crate) use self::stack::*;
mod sync;
pub(crate) use self::sync::*;
mod attr;
pub(crate) use self::attr::*;
mod endpos;
mod items;
pub(crate) use self::endpos::*;
mod command;
pub(crate) use self::command::*;
mod clear;
pub(crate) use self::clear::*;
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
pub(crate) use self::query::*;
mod syntime;
pub(crate) use self::syntime::*;

/// Size of `Expand::xp_buf`, the scratch buffer a completion callback may
/// answer from. `IOSIZE`, because the callbacks that build a name out of
/// one bound themselves by that; upstream answers the shared `IObuff` for
/// those, which the completion machinery writes again.
pub(crate) const EXPAND_BUF_LEN: ::core::ffi::c_uint = 1025;
// The `Expand::xp_context` values this module sets.
/// Which syntax group an item belongs to, and at what `:syntax include`
/// nesting it was declared. Both a pattern and a keyword carry one, and
/// [`in_id_list`] tests against it.
#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct sp_syn {
    pub inc_tag: ::core::ffi::c_int,
    pub id: int16_t,
}
/// A `contains=` / `containedin=` / `nextgroup=` list, or the absence of
/// one.
///
/// The ids are shared: an item on the state stack, a cached state and the
/// pending `nextgroup=` of the parser each hold the list of the pattern or
/// keyword they came from, where upstream held a bare `int16_t *` into it.
/// Sharing is safe because a list never changes once made. What upstream
/// told apart by pointer -- a cached state's `nextgroup=` against the
/// parser's, and two keywords' lists in `:syntax list` -- is told apart by
/// [`IdList::same`], which compares identity, not contents.
#[derive(Clone, Default)]
pub(crate) enum IdList {
    /// No list at all, which upstream spells as a NULL pointer. Not the same
    /// as an *empty* list: `contains=` naming nothing admits nothing, while
    /// no `contains=` at all leaves the item's default containment.
    #[default]
    Absent,
    /// The `contains=` of a transparent item that is not inside anything:
    /// every not-`contained` group. Upstream's `ID_LIST_ALL`, which only a
    /// state-stack item ever holds.
    TopLevel,
    /// The ids, in the order they were given.
    Ids(Rc<[int16_t]>),
}

impl IdList {
    /// [`IdList::Absent`], spelled as the constant upstream's NULL was.
    pub(crate) const NONE: IdList = IdList::Absent;

    /// A fresh list of `ids`. Always a list, even for no ids.
    pub(crate) fn from_ids(ids: &[int16_t]) -> IdList {
        IdList::Ids(ids.into())
    }

    /// [`IdList::from_ids`], except that no ids means no list -- which is
    /// what `:syntax cluster` stores for an emptied cluster.
    pub(crate) fn from_ids_or_none(ids: &[int16_t]) -> IdList {
        if ids.is_empty() {
            IdList::NONE
        } else {
            IdList::from_ids(ids)
        }
    }

    /// A list of its own with the same ids: what upstream's `copy_id_list`
    /// made for each keyword, which `:syntax list` tells apart by identity.
    pub(crate) fn copy(&self) -> IdList {
        match self {
            IdList::Ids(ids) => IdList::from_ids(ids),
            other => other.clone(),
        }
    }

    /// Whether there is no list.
    pub(crate) fn is_none(&self) -> bool {
        matches!(self, IdList::Absent)
    }

    /// The ids; none for [`IdList::Absent`] and [`IdList::TopLevel`].
    pub(crate) fn ids(&self) -> &[int16_t] {
        match self {
            IdList::Ids(ids) => ids,
            _ => &[],
        }
    }

    /// Whether the two are the *same* list -- upstream's pointer equality.
    pub(crate) fn same(&self, other: &IdList) -> bool {
        match (self, other) {
            (IdList::Absent, IdList::Absent) | (IdList::TopLevel, IdList::TopLevel) => true,
            (IdList::Ids(a), IdList::Ids(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}
/// The highest highlight id there can be.
pub(crate) const MAX_HL_ID: ::core::ffi::c_uint = 20000;
/// The `contains=ALL`/`ALLBUT` marker, which shares its value with the highest
/// possible highlight id and is offset by the `:syntax include` tag.
pub(crate) const SYNID_ALLBUT: ::core::ffi::c_int = MAX_HL_ID as ::core::ffi::c_int;
/// `do_source` flag: this is not a plugin or a package.
pub(crate) const DOSO_NONE: ::core::ffi::c_uint = 0;
#[derive(Clone)]
pub(crate) struct StateItem {
    pub si_idx: ::core::ffi::c_int,
    pub si_id: ::core::ffi::c_int,
    pub si_trans_id: ::core::ffi::c_int,
    pub si_m_lnum: ::core::ffi::c_int,
    pub si_m_startcol: ::core::ffi::c_int,
    pub si_m_endpos: LPos,
    pub si_h_startpos: LPos,
    pub si_h_endpos: LPos,
    pub si_eoe_pos: LPos,
    pub si_end_idx: ::core::ffi::c_int,
    pub si_ends: ::core::ffi::c_int,
    pub si_attr: ::core::ffi::c_int,
    pub si_flags: SynFlags,
    pub si_seqnr: ::core::ffi::c_int,
    pub si_cchar: ::core::ffi::c_int,
    /// `contains=`, from the item's pattern or the item around it.
    pub si_cont_list: IdList,
    /// `nextgroup=`, from the item's pattern or keyword.
    pub si_next_list: IdList,
    /// The `\z(` captures of the start match, which the end and skip
    /// patterns match `\z1`..`\z9` against.
    pub si_extmatch: Option<ExtMatchRef>,
}
/// One `:syntax match` pattern, or one start/skip/end pattern of a
/// `:syntax region`. Lives in its block's `b_syn_patterns`.
pub(crate) struct SynPat {
    pub sp_type: ::core::ffi::c_char,
    pub sp_syncing: bool,
    pub sp_syn_match_id: int16_t,
    pub sp_off_flags: int16_t,
    pub sp_offsets: [::core::ffi::c_int; 7],
    pub sp_flags: SynFlags,
    pub sp_cchar: ::core::ffi::c_int,
    pub sp_ic: ::core::ffi::c_int,
    pub sp_sync_idx: ::core::ffi::c_int,
    pub sp_line_id: ::core::ffi::c_int,
    pub sp_startcol: ::core::ffi::c_int,
    pub sp_cont_list: IdList,
    pub sp_next_list: IdList,
    /// `containedin=`. Upstream kept this inside `sp_syn` so that
    /// [`in_id_list`] could take one pointer; it is an owned list here and
    /// a keyword's is not, so the two travel separately.
    pub sp_cont_in_list: IdList,
    pub sp_syn: sp_syn,
    /// The pattern text, as `:syntax list` prints it. `None` only in a
    /// half-built pattern that never got one.
    pub sp_pattern: Option<::std::ffi::CString>,
    /// The compiled program; `None` only in a half-built pattern.
    pub sp_prog: Option<OwnedProg>,
    pub sp_time: SynTime,
}

impl SynPat {
    /// The pattern's `ms=`/`me=`/`hs=`/... offsets, copied out.
    ///
    /// [`syn_add_start_off`] and [`syn_add_end_off`] want only these, and
    /// taking them by value rather than borrowing the pattern is what lets
    /// their callers go on using the pattern array.
    #[inline]
    pub(crate) fn offsets(&self) -> PatOffsets {
        PatOffsets {
            flags: self.sp_off_flags,
            offsets: self.sp_offsets,
        }
    }
}

/// One pattern's offset suffixes: the `SPO_*` flag word and the seven values
/// it selects. See [`SynPat::offsets`].
#[derive(Copy, Clone)]
pub(crate) struct PatOffsets {
    pub flags: int16_t,
    pub offsets: [::core::ffi::c_int; SPO_COUNT as usize],
}

/// A half-built pattern: what upstream's `CLEAR_FIELD` left.
pub(crate) const EMPTY_SYNPAT: SynPat = SynPat {
    sp_type: 0,
    sp_syncing: false,
    sp_syn_match_id: 0,
    sp_off_flags: 0,
    sp_offsets: [0; SPO_COUNT as usize],
    sp_flags: SynFlags::NONE,
    sp_cchar: 0,
    sp_ic: 0,
    sp_sync_idx: 0,
    sp_line_id: 0,
    sp_startcol: 0,
    sp_cont_list: IdList::NONE,
    sp_next_list: IdList::NONE,
    sp_cont_in_list: IdList::NONE,
    sp_syn: sp_syn { inc_tag: 0, id: 0 },
    sp_pattern: None,
    sp_prog: None,
    sp_time: SynTime {
        total: 0,
        slowest: 0,
        count: 0,
        match_0: 0,
    },
};

/// One `:syntax cluster`: a name and the ids it stands for.
pub(crate) struct SynCluster {
    pub scl_name: ::std::ffi::CString,
    /// The name upper-cased, because a lookup compares that rather than
    /// paying `stricmp` per cluster.
    pub scl_name_u: ::std::ffi::CString,
    pub scl_list: IdList,
}

/// The options a `:syntax` item definition accepts, as they are parsed.
///
/// The owner of the three lists until an item takes them.
pub(crate) struct SynOptArg {
    pub flags: SynFlags,
    pub keyword: bool,
    /// Whether `grouphere`/`groupthere` is accepted here, which only
    /// `:syntax sync match` is.
    pub takes_sync_idx: bool,
    /// The pattern index `grouphere`/`groupthere` named, or 0.
    pub sync_idx: ::core::ffi::c_int,
    pub has_cont_list: bool,
    pub cont_list: IdList,
    pub cont_in_list: IdList,
    pub next_list: IdList,
}
pub(crate) const SYNSPL_DEFAULT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub(crate) const SYNSPL_TOP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub(crate) const SYNSPL_NOTOP: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub(crate) const SYNFLD_START: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub(crate) const SYNFLD_MINIMUM: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub(crate) const SST_MIN_ENTRIES: ::core::ffi::c_int = 150 as ::core::ffi::c_int;
pub(crate) const SST_MAX_ENTRIES: ::core::ffi::c_int = 1000 as ::core::ffi::c_int;
pub(crate) const SST_DIST: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
/// Whether `:syntax on|off|enable|manual` has been used, which is what stops
/// [`syn_maybe_enable`] from overriding a deliberate choice.
static did_syntax_onoff: GlobalCell<bool> = GlobalCell::new(false);
/// Whether `:syntime on` is in effect. A setting rather than parser state:
/// a parse that re-enters must not lose it.
static syn_time_on: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) const SPO_MS_OFF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub(crate) const SPO_ME_OFF: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub(crate) const SPO_HS_OFF: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub(crate) const SPO_HE_OFF: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub(crate) const SPO_RS_OFF: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub(crate) const SPO_RE_OFF: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub(crate) const SPO_LC_OFF: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub(crate) const SPO_COUNT: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub(crate) const E_CONTAINS_NOT_ACCEPTED_HERE: &::core::ffi::CStr =
    c"E395: Contains argument not accepted here";
pub(crate) const E_INVALID_CCHAR_VALUE: &::core::ffi::CStr = c"E844: Invalid cchar value";
pub(crate) const SPTYPE_MATCH: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub(crate) const SPTYPE_START: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub(crate) const SPTYPE_END: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub(crate) const SPTYPE_SKIP: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub(crate) const NONE_IDX: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub(crate) const SF_CCOMMENT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub(crate) const SF_MATCH: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub(crate) const MAXKEYWLEN: ::core::ffi::c_int = 80 as ::core::ffi::c_int;
pub(crate) const CLUSTER_REPLACE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub(crate) const CLUSTER_ADD: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub(crate) const CLUSTER_SUBTRACT: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub(crate) const SYNID_TOP: ::core::ffi::c_int = 21000 as ::core::ffi::c_int;
pub(crate) const SYNID_CONTAINED: ::core::ffi::c_int = 22000 as ::core::ffi::c_int;
pub(crate) const SYNID_CLUSTER: ::core::ffi::c_int = 23000 as ::core::ffi::c_int;
pub(crate) const MAX_SYN_INC_TAG: ::core::ffi::c_int = 999 as ::core::ffi::c_int;
pub(crate) const MAX_CLUSTER_ID: ::core::ffi::c_int = 32767 as ::core::ffi::c_int - SYNID_CLUSTER;
/// The `:syntax include` nesting tag items are being defined under; 0 outside
/// an inclusion.
static current_syn_inc_tag: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
/// The highest tag handed out so far, capped at [`MAX_SYN_INC_TAG`].
static running_syn_inc_tag: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
/// What every `:syntax`/`:syntime` listing answers when there is nothing to
/// list.
pub(crate) const MSG_NO_ITEMS: &::core::ffi::CStr = c"No Syntax items defined for this buffer";

/// `bytes`, copied out as an owned name.
///
/// Upstream's `vim_strnsave`, which made an `xmalloc`ed copy the caller then
/// treated as a C string: the copy stops at a NUL inside the bytes, because
/// every reader of that string would have stopped there anyway.
pub(crate) fn name_in(bytes: &[u8]) -> ::std::ffi::CString {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    cstr::owned(&bytes[..end])
}

/// The white-space-delimited word `text` starts with, and how many bytes
/// come before the byte after it.
///
/// The shape every `:syntax` mode command reads its argument in, and the
/// step of the argument walks in [`clear`] and [`list`]. `text` is a
/// command line's tail, so the word also ends at the NUL.
pub(crate) fn word_at(text: &[u8]) -> (&[u8], usize) {
    let end = skip::to_white(text);
    let end = text[..end]
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(end);
    (&text[..end], end)
}

/// The syntax block being *configured* — `curwin`'s, which during a `:syntax`
/// command is not necessarily [`SynState::block`], the one being *parsed*.
#[inline]
pub(crate) fn cur_syn_block() -> SynBlockRef {
    Win::current().syntax()
}

impl Buf {
    /// The buffer's own syntax block.
    #[inline]
    pub(crate) fn syntax(self) -> SynBlockRef {
        // SAFETY: a buffer's `b_s` lives as long as the buffer.
        unsafe { SynBlockRef::new(self.syntax_block()) }
    }
}

impl Win {
    /// The syntax block the window uses: its buffer's, or the one
    /// `:ownsyntax` gave it.
    #[inline]
    pub(crate) fn syntax(self) -> SynBlockRef {
        // SAFETY: `w_s` names either the window's own block or its buffer's,
        // and both outlive the window that points at them.
        unsafe { SynBlockRef::new(self.w_s) }
    }
}

impl SynBlockRef {
    /// The block's patterns.
    ///
    /// The borrow lasts as long as the *handle* it came from, which is what
    /// stops a caller holding a pattern across a command that grows the
    /// array — as long as both go through the same handle. Take one handle
    /// per function and reach everything through it.
    #[inline]
    pub(crate) fn patterns(&self) -> &[SynPat] {
        &self.b_syn_patterns
    }

    /// The block's patterns, to add to or remove from.
    #[inline]
    pub(crate) fn patterns_mut(&mut self) -> &mut Vec<SynPat> {
        &mut self.b_syn_patterns
    }

    /// The pattern at `idx`, which must be one the block has.
    #[inline]
    pub(crate) fn pattern(&self, idx: ::core::ffi::c_int) -> &SynPat {
        &self.b_syn_patterns[idx as usize]
    }

    /// The pattern at `idx`, to write to.
    #[inline]
    pub(crate) fn pattern_mut(&mut self, idx: ::core::ffi::c_int) -> &mut SynPat {
        &mut self.b_syn_patterns[idx as usize]
    }

    /// The block's clusters, on [`SynBlock::patterns`]' terms.
    #[inline]
    pub(crate) fn clusters(&self) -> &[SynCluster] {
        &self.b_syn_clusters
    }

    /// The block's clusters, to add to or edit.
    #[inline]
    pub(crate) fn clusters_mut(&mut self) -> &mut Vec<SynCluster> {
        &mut self.b_syn_clusters
    }
}

/// Number of patterns in [`cur_syn_block`].
#[inline]
pub(crate) fn cur_pattern_count() -> ::core::ffi::c_int {
    cur_syn_block().patterns().len() as ::core::ffi::c_int
}

/// Number of clusters in [`cur_syn_block`].
#[inline]
pub(crate) fn cur_cluster_count() -> ::core::ffi::c_int {
    cur_syn_block().clusters().len() as ::core::ffi::c_int
}
/// A syntax block — a window's or a buffer's `SynBlock`, whose holder has
/// promised it outlives the value.
///
/// The promise is discharged by the window or buffer that owns the block: a
/// `w_s` is either the buffer's `b_s` or an `:ownsyntax` block the window
/// frees with itself.
pub(crate) type SynBlockRef = Live<SynBlock>;

/// `StateItem::si_idx` for a keyword, which has no pattern.
pub(crate) const KEYWORD_IDX: ::core::ffi::c_int = -1;

/// A cleared state item: what upstream's `GA_APPEND_VIA_PTR` slot holds
/// once `ga_grow` has zeroed it.
const EMPTY_STATE_ITEM: StateItem = StateItem {
    si_idx: 0,
    si_id: 0,
    si_trans_id: 0,
    si_m_lnum: 0,
    si_m_startcol: 0,
    si_m_endpos: LPos { lnum: 0, col: 0 },
    si_h_startpos: LPos { lnum: 0, col: 0 },
    si_h_endpos: LPos { lnum: 0, col: 0 },
    si_eoe_pos: LPos { lnum: 0, col: 0 },
    si_end_idx: 0,
    si_ends: 0,
    si_attr: 0,
    si_flags: SynFlags::NONE,
    si_seqnr: 0,
    si_cchar: 0,
    si_cont_list: IdList::NONE,
    si_next_list: IdList::NONE,
    si_extmatch: None,
};
pub(crate) const ITEM_START: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub(crate) const ITEM_SKIP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub(crate) const ITEM_END: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub(crate) const ITEM_MATCHGROUP: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub(crate) const REX_SET: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub(crate) const REX_USE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
