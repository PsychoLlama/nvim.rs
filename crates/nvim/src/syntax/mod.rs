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
//!
//! # Ownership
//!
//! Everything a syntax block holds is owned by value: `b_syn_patterns` and
//! `b_syn_clusters` are `Vec`s of [`synpat_T`]/[`syn_cluster_T`], a pattern's
//! text is a `CString` and its `contains=`/`containedin=`/`nextgroup=` lists
//! are [`IdList`]s. **An `xfree` in this module is a bug** unless it is one of
//! the three carve-outs, each of which says so at its own field:
//!
//! - [`keyentry`] -- one `xmalloc` block with the keyword text inside it,
//!   which the hash tables key on by *interior address*, and the two raw id
//!   lists [`copy_id_list`](options::copy_id_list) makes for it.
//! - `synblock_T::b_sst_array` -- the state cache's slab, threaded into two
//!   intrusive lists of interior pointers.
//! - [`synpat_T::sp_prog`] and `synblock_T::b_syn_linecont_prog` -- compiled
//!   programs, which belong to `regexp/`'s allocator discipline.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::ascii::ascii_iswhite;
use crate::autocmd::apply_autocmds;
use crate::buffer::buf_get_changedtick;
use crate::charset::{
    buf_init_chartab, getdigits_int, getdigits_int32, skiptowhite, skipwhite, str_foldcase,
    vim_isprintc, vim_iswordp_buf,
};
use crate::cstr;
use crate::drawscreen::{UPD_NOT_VALID, UPD_SOME_VALID, redraw_curbuf_later, redraw_later};
use crate::eval::vars::{do_unlet, get_var_value, set_internal_string_var};
use crate::ex_docmd::{
    check_nextcmd, do_cmdline_cmd, ends_excmd, expand_filename, find_nextcmd, separate_nextcmd,
};
use crate::fold::{fold_update_all, foldmethod_is_syntax};
use crate::garray::{ga_clear, ga_grow, ga_init};
use crate::global_cell::GlobalCell;
use crate::hashtab::{
    hash_add_item, hash_find, hash_hash, hash_init, hash_lock, hash_lookup, hash_remove,
    hash_reset, hash_set_key, hash_unlock,
};
use crate::highlight_group::{
    HLF_D, highlight_group_name, highlight_link_id, highlight_num_groups, init_highlight,
    syn_check_group, syn_id2attr, syn_list_header, syn_name2id, syn_name2id_len,
};
use crate::indent_c::find_start_comment;
use crate::main::{
    Columns, Rows, curbuf, curwin, display_tick, got_int, include_default, include_link,
    include_none, msg_col, p_cpo, re_extmatch_in, re_extmatch_out, reg_do_extmatch,
};
use crate::mbyte::{mb_strcmp_ic, utf_head_off, utf_ptr2char, utfc_ptr2len};
use crate::memline::{ml_get, ml_get_buf, ml_get_buf_len, ml_get_len};
use crate::memory::{xcalloc, xfree, xmalloc, xmemcpyz, xstrdup};
use crate::message::{
    emsg, msg, msg_advance, msg_ext_set_kind, msg_outnum, msg_outtrans, msg_outtrans_len,
    msg_putchar, msg_puts, msg_puts_hl, msg_puts_title,
};
use crate::optionstr::clear_string_option;
use crate::os::cshim::{gettext, strncasecmp};
use crate::os::input::line_breakcheck;
use crate::path::path_is_absolute;
use crate::pos::MAXLNUM;
use crate::profile::{
    profile_add, profile_cmp, profile_divide, profile_end, profile_msg, profile_start, profile_zero,
};
use crate::regexp::{
    ref_extmatch, skip_regexp, unref_extmatch, vim_regcomp, vim_regcomp_had_eol, vim_regexec,
    vim_regexec_multi, vim_regfree,
};
use crate::runtime::{do_source, source_runtime};
use crate::strings::{vim_snprintf, vim_strchr};
use crate::types::AutoEvent;
use crate::types::{
    OptInt, buf_T, bufstate_T, colnr_T, exarg_T, expand_T, hashtab_T, int16_t, linenr_T, lpos_T,
    proftime_T, reg_extmatch_T, regmatch_T, regmmatch_T, regprog_T, size_t, syn_time_T, synblock_T,
    synstate_T, uint8_t, uint64_t, varnumber_T, win_T,
};
use crate::winlayer::{Live, Win};
use ::libc::{qsort, strcpy, strpbrk};

mod flags;
pub(crate) use self::flags::*;

// The carve of the transpiled module; see each child's docs.
mod state;
pub(crate) use self::state::*;
mod stack;
pub(crate) use self::stack::*;
mod sync;
pub(crate) use self::sync::*;
mod attr;
pub(crate) use self::attr::*;
mod items;
pub(crate) use self::items::*;
mod endpos;
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

/// How many `\(..\)` submatches a pattern can have.
pub(crate) const NSUBEXP: ::core::ffi::c_uint = 10;
/// Size of `expand_T::xp_buf`, the scratch buffer a completion callback may
/// answer from. `IOSIZE`, because the callbacks that build a name out of
/// one bound themselves by that; upstream answers the shared `IObuff` for
/// those, which the completion machinery writes again.
pub(crate) const EXPAND_BUF_LEN: ::core::ffi::c_uint = 1025;
// The `expand_T::xp_context` values this module sets.
/// Which syntax group an item belongs to, and at what `:syntax include`
/// nesting it was declared. Both a pattern and a keyword carry one, and
/// [`in_id_list`] tests against it.
#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct sp_syn {
    pub inc_tag: ::core::ffi::c_int,
    pub id: int16_t,
}
/// One `:syntax keyword`, in one of the two keyword hash tables.
///
/// OWNERSHIP -- **carve-out**. The entry is one `xmalloc` block whose
/// trailing `keyword` array holds the text, and the hash tables key on
/// *that address*: `key_to_entry` finds the entry by subtracting
/// [`KEYWORD_OFFSET`](keyword::KEYWORD_OFFSET) from a key. So the entry
/// cannot be a `Box<keyentry_T>` (the text would not be inside it) and its
/// two id lists cannot be [`IdList`]s (nothing would run their destructor).
/// Retiring it needs the keyword tables to stop keying on an interior
/// address -- a `hashtab_T` that owns its keys, or an id-keyed table with
/// the text beside the entry.
#[repr(C)]
pub(crate) struct keyentry {
    pub ke_next: *mut keyentry_T,
    pub k_syn: sp_syn,
    /// `containedin=`, this entry's own `xmalloc`ed list.
    pub cont_in_list: *mut int16_t,
    /// `nextgroup=`, this entry's own `xmalloc`ed list.
    pub next_list: *mut int16_t,
    pub flags: SynFlags,
    pub k_char: ::core::ffi::c_int,
    pub keyword: [::core::ffi::c_char; 0],
}
pub(crate) type keyentry_T = keyentry;

/// A `contains=` / `containedin=` / `nextgroup=` list: the syntax ids it
/// names, followed by the 0 terminator upstream's `int16_t *` carried.
///
/// The terminator stays because the *borrowers* still walk a bare pointer:
/// a state-stack item's `si_cont_list`/`si_next_list`, a cached state's
/// `sst_next_list` and [`current_next_list`] all point into the list of
/// whatever item they came from. An owner holds a `Box<[int16_t]>`, so the
/// ids keep their address when the pattern array around them grows.
///
/// Nothing shares one: the consecutive START patterns of a region each get
/// their own copy. Upstream gave them one list owned by the first and freed
/// the array last to first so the peek at the entry before could tell the
/// owner from the sharers; a list is a handful of `int16_t`s, and copying
/// it costs less than a refcount and much less than that rule.
#[derive(Clone, Default)]
pub(crate) struct IdList(Option<Box<[int16_t]>>);

impl IdList {
    /// No list at all, which upstream spells as a NULL pointer. Not the same
    /// as an *empty* list: `contains=` naming nothing admits nothing, while
    /// no `contains=` at all leaves the item's default containment.
    pub(crate) const NONE: IdList = IdList(None);

    /// Copy `ids` into a fresh list, terminator included. Always a list,
    /// even for no ids.
    pub(crate) fn from_ids(ids: &[int16_t]) -> IdList {
        let mut out = Vec::with_capacity(ids.len() + 1);
        out.extend_from_slice(ids);
        out.push(0);
        IdList(Some(out.into_boxed_slice()))
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

    /// Whether there is no list.
    pub(crate) fn is_none(&self) -> bool {
        self.0.is_none()
    }

    /// The ids, without the terminator.
    pub(crate) fn ids(&self) -> &[int16_t] {
        match &self.0 {
            Some(ids) => &ids[..ids.len() - 1],
            None => &[],
        }
    }

    /// The pointer a borrower walks, or NULL when there is no list.
    pub(crate) fn as_ptr(&self) -> *mut int16_t {
        match &self.0 {
            Some(ids) => ids.as_ptr().cast_mut(),
            None => ::core::ptr::null_mut(),
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
#[derive(Copy, Clone)]
pub(crate) struct stateitem_T {
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
    pub si_flags: SynFlags,
    pub si_seqnr: ::core::ffi::c_int,
    pub si_cchar: ::core::ffi::c_int,
    pub si_cont_list: *mut int16_t,
    pub si_next_list: *mut int16_t,
    pub si_extmatch: *mut reg_extmatch_T,
}
/// One `:syntax match` pattern, or one start/skip/end pattern of a
/// `:syntax region`. Lives in its block's `b_syn_patterns`.
pub(crate) struct synpat_T {
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
    /// OWNERSHIP -- **carve-out**. The compiled program is a `regexp/`
    /// object with its own allocator discipline (`vim_regcomp` /
    /// `vim_regfree`, and two engines behind one `regprog_T`), so it stays
    /// a raw pointer released by [`Drop`] rather than becoming a `Box`.
    /// Retiring it is `regexp/`'s job, not this module's.
    pub sp_prog: *mut regprog_T,
    pub sp_time: syn_time_T,
}

impl synpat_T {
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
/// it selects. See [`synpat_T::offsets`].
#[derive(Copy, Clone)]
pub(crate) struct PatOffsets {
    pub flags: int16_t,
    pub offsets: [::core::ffi::c_int; SPO_COUNT as usize],
}

/// A half-built pattern: what upstream's `CLEAR_FIELD` left.
pub(crate) const EMPTY_SYNPAT: synpat_T = synpat_T {
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
    sp_prog: ::core::ptr::null_mut(),
    sp_time: syn_time_T {
        total: 0,
        slowest: 0,
        count: 0,
        match_0: 0,
    },
};

impl Drop for synpat_T {
    fn drop(&mut self) {
        // SAFETY: the compiled program is this pattern's own and nothing
        // else holds it; `vim_regfree` accepts a null one.
        unsafe { vim_regfree(self.sp_prog) };
    }
}

/// One `:syntax cluster`: a name and the ids it stands for.
pub(crate) struct syn_cluster_T {
    pub scl_name: ::std::ffi::CString,
    /// The name upper-cased, because a lookup compares that rather than
    /// paying `stricmp` per cluster.
    pub scl_name_u: ::std::ffi::CString,
    pub scl_list: IdList,
}

/// The options a `:syntax` item definition accepts, as they are parsed.
///
/// The owner of the three lists until an item takes them.
pub(crate) struct syn_opt_arg_T {
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
/// The `:source` argument the on/off commands build; `%s` is the file's name.
pub(crate) const SYNTAX_FNAME: &::core::ffi::CStr = c"$VIMRUNTIME/syntax/%s.vim";
pub(crate) const SST_MIN_ENTRIES: ::core::ffi::c_int = 150 as ::core::ffi::c_int;
pub(crate) const SST_MAX_ENTRIES: ::core::ffi::c_int = 1000 as ::core::ffi::c_int;
pub(crate) const SST_FIX_STATES: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub(crate) const SST_DIST: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
/// Whether `:syntax on|off|enable|manual` has been used, which is what stops
/// [`syn_maybe_enable`] from overriding a deliberate choice.
static did_syntax_onoff: GlobalCell<bool> = GlobalCell::new(false);
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
// What the last `syn_current_attr` decided about the current position. The
// query API reads these back, so they outlive the call that set them.

/// Attribute number of the current character.
static current_attr: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
/// Syntax id of the current character, before transparency.
static current_id: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
/// Syntax id of the current character, after transparency.
static current_trans_id: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
/// `HL_*` flags of the current character.
static current_flags: GlobalCell<SynFlags> = GlobalCell::new(SynFlags::NONE);
/// Sequence number of the item the current character belongs to, which is what
/// tells two runs of the same group apart.
static current_seqnr: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
/// The `cchar=` of the current character, for `conceal`.
static current_sub_char: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub(crate) const CLUSTER_REPLACE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub(crate) const CLUSTER_ADD: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub(crate) const CLUSTER_SUBTRACT: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub(crate) const SYNID_TOP: ::core::ffi::c_int = 21000 as ::core::ffi::c_int;
pub(crate) const SYNID_CONTAINED: ::core::ffi::c_int = 22000 as ::core::ffi::c_int;
pub(crate) const SYNID_CLUSTER: ::core::ffi::c_int = 23000 as ::core::ffi::c_int;
pub(crate) const MAX_SYN_INC_TAG: ::core::ffi::c_int = 999 as ::core::ffi::c_int;
pub(crate) const MAX_CLUSTER_ID: ::core::ffi::c_int = 32767 as ::core::ffi::c_int - SYNID_CLUSTER;
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

/// The `len` bytes at `p`, copied out as an owned name.
///
/// Upstream's `vim_strnsave`, which made an `xmalloc`ed copy the caller then
/// treated as a C string: the copy stops at a NUL inside the bytes, because
/// every reader of that string would have stopped there anyway.
///
/// # Safety
/// `p` must point at `len` readable bytes.
pub(crate) unsafe fn name_at(p: *const ::core::ffi::c_char, len: usize) -> ::std::ffi::CString {
    // SAFETY: the caller's promise.
    let bytes = unsafe { cstr::slice_at(p, len) };
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    cstr::owned(&bytes[..end])
}

/// The white-space-delimited word `arg` starts with, and where it ends.
///
/// The shape every `:syntax` mode command reads its argument in, and the
/// step of the argument walks in [`clear`] and [`list`].
///
/// # Safety
/// `arg` must be a NUL-terminated string.
pub(crate) unsafe fn word_at(
    arg: *mut ::core::ffi::c_char,
) -> (&'static [u8], *mut ::core::ffi::c_char) {
    // SAFETY: the caller's promise.
    let end = unsafe { skiptowhite(arg) };
    // SAFETY: both pointers are into that string, `arg` first, so the bytes
    // between them are readable and live as long as the command line is.
    let word = unsafe { cstr::slice_at(arg, end.offset_from(arg) as usize) };
    (word, end)
}

/// The window the editor is working in.
#[inline]
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

/// The syntax block being *configured* — `curwin`'s, which during a `:syntax`
/// command is not necessarily [`syn_block`], the one being *parsed*.
#[inline]
pub(crate) fn cur_syn_block() -> SynBlock {
    // SAFETY: `w_s` names either the window's own block or its buffer's, and
    // both outlive the window that points at them.
    unsafe { SynBlock::new(cur_win().w_s) }
}

impl SynBlock {
    /// The block's patterns.
    ///
    /// The borrow lasts as long as the *handle* it came from, which is what
    /// stops a caller holding a pattern across a command that grows the
    /// array — as long as both go through the same handle. Take one handle
    /// per function and reach everything through it.
    #[inline]
    pub(crate) fn patterns(&self) -> &[synpat_T] {
        &self.b_syn_patterns
    }

    /// The block's patterns, to add to or remove from.
    #[inline]
    pub(crate) fn patterns_mut(&mut self) -> &mut Vec<synpat_T> {
        &mut self.b_syn_patterns
    }

    /// The pattern at `idx`, which must be one the block has.
    #[inline]
    pub(crate) fn pattern(&self, idx: ::core::ffi::c_int) -> &synpat_T {
        &self.b_syn_patterns[idx as usize]
    }

    /// The pattern at `idx`, to write to.
    #[inline]
    pub(crate) fn pattern_mut(&mut self, idx: ::core::ffi::c_int) -> &mut synpat_T {
        &mut self.b_syn_patterns[idx as usize]
    }

    /// The cluster at `idx`, which must be one the block has.
    #[inline]
    pub(crate) fn cluster(&self, idx: ::core::ffi::c_int) -> &syn_cluster_T {
        &self.b_syn_clusters[idx as usize]
    }

    /// The block's clusters, on [`SynBlock::patterns`]' terms.
    #[inline]
    pub(crate) fn clusters(&self) -> &[syn_cluster_T] {
        &self.b_syn_clusters
    }

    /// The block's clusters, to add to or edit.
    #[inline]
    pub(crate) fn clusters_mut(&mut self) -> &mut Vec<syn_cluster_T> {
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
/// A syntax block — a window's or a buffer's `synblock_T`, whose holder has
/// promised it outlives the value.
///
/// The promise is discharged by the window or buffer that owns the block: a
/// `w_s` is either the buffer's `b_s` or an `:ownsyntax` block the window
/// frees with itself.
pub(crate) type SynBlock = Live<synblock_T>;

/// The address of one field of a syntax block, **without borrowing the
/// block**.
///
/// `&raw mut cur_syn_block().b_keywtab` would take its provenance from the
/// transient `&mut synblock_T` that [`Live`]'s `DerefMut` hands out, and the
/// block's next field access invalidates that borrow — so an address kept
/// past the statement it was taken in is already dangling under Stacked and
/// Tree Borrows. [`Live::field_ptr`] computes the same address from the
/// pointer, which is why it exists.
macro_rules! syn_field {
    ($block:expr, $field:ident) => {
        $block.field_ptr(::core::mem::offset_of!(crate::types::synblock_T, $field))
    };
}
pub(crate) use syn_field;

/// One item on the syntax state stack, whose holder has promised the stack
/// has not been pushed to, popped from or cleared since it was taken.
///
/// The stack is a `Vec`, so a push can move every item in it: reach for one
/// through [`items::state_at`] each time rather than holding one across a
/// call that can parse.
pub(crate) type Item = Live<stateitem_T>;

/// `stateitem_T::si_idx` for a keyword, which has no pattern.
pub(crate) const KEYWORD_IDX: ::core::ffi::c_int = -1;
/// The `contains=` list of a transparent item that is not inside anything: it
/// admits every not-`contained` group.
pub(crate) const ID_LIST_ALL: *mut int16_t = -1 as ::core::ffi::c_int as *mut int16_t;
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
static next_match_flags: GlobalCell<SynFlags> = GlobalCell::new(SynFlags::NONE);
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
/// the buffer's. Reach it through [`syn_block`].
static parsed_block: GlobalCell<*mut synblock_T> = GlobalCell::new(::core::ptr::null_mut());

/// The syntax block being *parsed*, which during a `:syntax` command is not
/// necessarily [`cur_syn_block`], the one being *configured*.
///
/// Null until [`syntax_start`] has run, which every caller of this checks for
/// through `b_sst_array` the way upstream does.
#[inline]
pub(crate) fn syn_block() -> SynBlock {
    // SAFETY: set from `syntax_start` to the window or buffer that owns it,
    // and cleared when that owner goes away.
    unsafe { SynBlock::new(parsed_block.get()) }
}
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
/// The syntax state stack, outermost item first: what the parser is inside
/// at [`current_col`].
///
/// `None` is upstream's "invalid state", which it marks by zeroing the
/// growarray's `ga_itemsize` — a flag about the value, so it belongs in the
/// value's type. Reach it through [`items::state_len`]/[`items::state_at`]
/// and the `*_current_state` family, never directly.
static current_state: GlobalCell<Option<Vec<stateitem_T>>> = GlobalCell::new(None);

/// A cleared state item: what upstream's `GA_APPEND_VIA_PTR` slot holds
/// once `ga_grow` has zeroed it.
const EMPTY_STATE_ITEM: stateitem_T = stateitem_T {
    si_idx: 0,
    si_id: 0,
    si_trans_id: 0,
    si_m_lnum: 0,
    si_m_startcol: 0,
    si_m_endpos: lpos_T { lnum: 0, col: 0 },
    si_h_startpos: lpos_T { lnum: 0, col: 0 },
    si_h_endpos: lpos_T { lnum: 0, col: 0 },
    si_eoe_pos: lpos_T { lnum: 0, col: 0 },
    si_end_idx: 0,
    si_ends: 0,
    si_attr: 0,
    si_flags: SynFlags::NONE,
    si_seqnr: 0,
    si_cchar: 0,
    si_cont_list: ::core::ptr::null_mut(),
    si_next_list: ::core::ptr::null_mut(),
    si_extmatch: ::core::ptr::null_mut(),
};
/// The `nextgroup=` list in effect, or NULL.
static current_next_list: GlobalCell<*mut int16_t> = GlobalCell::new(::core::ptr::null_mut());
/// The `skipwhite`/`skipnl`/`skipempty` flags that came with it.
static current_next_flags: GlobalCell<SynFlags> = GlobalCell::new(SynFlags::NONE);
/// `display_tick` when the current line was parsed, which is how a `display`
/// item knows it is being drawn rather than scanned.
static current_line_id: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
/// Whether `:syntime on` is in effect.
static syn_time_on: GlobalCell<bool> = GlobalCell::new(false);

/// Set the time limit for parsing, or clear it with NULL.
pub(crate) unsafe fn syn_set_timeout(tm: *mut proftime_T) {
    syn_tm.set(tm);
}
pub(crate) const ITEM_START: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub(crate) const ITEM_SKIP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub(crate) const ITEM_END: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub(crate) const ITEM_MATCHGROUP: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub(crate) const REX_SET: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub(crate) const REX_USE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
