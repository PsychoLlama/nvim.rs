//! Mappings and abbreviations: the `:map` family and the table behind it.
//!
//! The module is carved into children along the code's own seams — see each
//! child's own docs.  Everything left here is what they share: the constants
//! the transpile hoisted out of `mapping.c`'s includes, the [`MapArguments`]
//! struct every `:map` command is parsed into, and the six error texts.  The
//! children reach all of it through `use super::*`. So are the types the
//! tables are made of ([`MapBlock`] and its right-hand side, [`MapTables`]);
//! the two tables themselves live in [`table`].
//!
//! # Ownership
//!
//! Nothing in this module is freed by hand.  A table owns its entries by
//! value (see [`MapTables`](crate::mapping::MapTables)); a mapping's four
//! strings are owned: the LHS is a [`MapStr`] and the three RHS ones a
//! [`MapRhs`], both held by value on the entry.  Upstream keeps *one* RHS per
//! `<C-H>`-style pair and leaves it owned by whichever of the two is freed
//! second, using `m_alt` as the "the other one still owns them" flag; here
//! the twin gets its own copy — the pair is rare.  Only the Lua callback is
//! shared (an [`Rc<MapCallback>`], `None` when there is none), because a
//! registry reference cannot be duplicated; `m_alt` names the twin by id.
//!
//! Nothing outside a table's borrow holds an address into it: what has to
//! outlive one -- the match the typeahead found, which an `<expr>` RHS can
//! delete -- holds a [`MapAt`], an id the table re-finds.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::keycodes::{KS_EXTRA, KS_MODIFIER};
use crate::types::ScriptCtx;
use core::ffi::{CStr, c_int};

use crate::api::keysets::KeyDict_keymap;
use crate::api::private::helpers::{api_set_sctx, find_buffer_by_handle};
use crate::ascii::{ascii_isspace, ascii_iswhite};
use crate::charset::transchar;
use crate::cmdexpand::cmdline_fuzzy_complete;
use crate::cmdexpand::defs::Expand;
use crate::eval::funcs::builtin::EvalFuncData;
use crate::eval::typval::{
    dict_find, dict_get_bool, dict_get_number, dict_get_string_buf, tv_check_for_dict_arg,
    tv_dict_alloc_ret, tv_get_bool, tv_get_number, tv_list_alloc_ret,
};
use crate::eval::userfunc::find_func;
use crate::eval::vars::set_vim_var_char;
use crate::eval::{eval_to_string, last_set_msg};
use crate::ex_cmds::check_secure;
use crate::fuzzy::fuzzy_match_str;
use crate::getchar::noremap_keys;
use crate::getchar::state::{got_int, langmap_mapchar, mapped_ctrl_c};
use crate::global_cell::GlobalCell;
use crate::guard::secure;
use crate::highlight_group::HLF_8;
use crate::keycodes::{
    K_SPECIAL, escape_ks_bytes, get_special_key_name, push_char_escaped, replace_termcodes,
    unescape_ks, unescape_ks_in,
};
use crate::lua::executor::LuaRetMode;
use crate::lua::executor::api_new_luaref;
use crate::mbyte::cluster_len;
use crate::message::state::{msg_col, msg_row, msg_silent};
use crate::message::{e_invarg, e_noabbr, e_nomap};
use crate::message::{
    emsg, iemsg, message_filtered, msg, msg_clr_eos, msg_display, msg_display_keys,
    msg_ext_set_kind, msg_putchar, msg_start, msg_str, msg_str_hl, special_keys_text,
};
use crate::option::defs::OptSet;
use crate::option::vars::{p_cpo, p_verbose};
use crate::os::cshim::gettext;
use crate::os::libc::{ptrdiff_t, size_t, uint64_t};
use crate::regexp::RegMatch;
use crate::regexp::vim_regexec;
use crate::runtime::state::current_sctx;
use crate::state::mode::{State, no_abbr};
use crate::state::{
    MODE_CMDLINE, MODE_INSERT, MODE_LANGMAP, MODE_NORMAL, MODE_OP_PENDING, MODE_SELECT,
    MODE_TERMINAL, MODE_VISUAL,
};
use crate::types::{
    ApiDict, ApiString, Array, BufferHandle, Dict, Error, ExArg, Integer, LineNr, LuaRef, Object,
    ScriptId, TypVal, VarNumber,
};
use std::rc::Rc;

// The carve of the transpiled module; see each child's docs.
mod table;
pub use self::table::*;
mod args;
pub(crate) use self::args::*;
mod domap;
pub use self::domap::*;
mod show;
#[cfg(test)]
mod tests;
pub use self::show::*;
mod session;
pub use self::session::*;
mod introspect;
pub use self::introspect::*;
mod mapset;
pub use self::mapset::*;
mod abbrev;
pub use self::abbrev::*;
mod langmap;
pub use self::langmap::*;
pub use crate::getchar::MAXMAPLEN;
pub const FUZZY_SCORE_NONE: ::core::ffi::c_int = -2147483648;
pub use crate::keycodes::{REPTERM_DO_LT, REPTERM_FROM_PART, REPTERM_NO_SIMPLIFY};
pub const kRetObject: LuaRetMode = 0;
/// The `:map` arguments being parsed, and the owner of everything a parse
/// allocates.
///
/// Nothing here is freed by hand.  The two LHS spellings are [`MapStr`]s; the
/// right-hand side is the [`MapRhs`] bundle every mapblock this parse creates
/// will *share*, so `map_add` clones the [`Rc`] rather than taking a pointer
/// and the caller no longer has to null its own fields out afterwards.  If no
/// mapping is created, dropping the struct releases the Lua reference too.
#[derive(Debug, Default)]
pub(crate) struct MapArguments {
    pub buffer: bool,
    pub expr: bool,
    pub noremap: bool,
    pub nowait: bool,
    pub script: bool,
    pub silent: bool,
    pub unique: bool,
    pub replace_keycodes: bool,
    /// The LHS after `replace_termcodes`, truncated to `MAXMAPLEN` bytes.
    pub lhs: MapStr,
    /// How long the LHS was *before* that truncation, which is how a caller
    /// detects an over-long one.
    pub lhs_len: size_t,
    /// The unsimplified spelling of a `<C-H>`-style LHS; empty when the LHS
    /// did not simplify.
    pub alt_lhs: MapStr,
    /// How long `alt_lhs` was before truncation, or 0 for "did not simplify".
    pub alt_lhs_len: size_t,
    /// The right-hand side, once [`set_maparg_rhs`] has built it.
    pub rhs: Option<MapRhs>,
    /// Whether the RHS came out empty from a non-empty spelling, which is
    /// what `<Nop>` and a lone CTRL-V both mean.
    pub rhs_is_noop: bool,
    /// `:map <desc>`'s text.  Set *before* [`set_maparg_rhs`], which folds it
    /// into the [`MapRhs`] it builds.
    pub desc: Option<MapStr>,
}

/// `skipwhite` over a slice: `bytes` without its leading spaces and tabs.
pub(crate) fn skip_white(bytes: &[u8]) -> &[u8] {
    let at = bytes
        .iter()
        .position(|&byte| !ascii_iswhite(c_int::from(byte)))
        .unwrap_or(bytes.len());
    &bytes[at..]
}

impl MapArguments {
    /// The RHS bundle, which every path that reads one has already built.
    pub(crate) fn rhs(&self) -> &MapRhs {
        self.rhs
            .as_ref()
            .expect("the RHS is parsed before it is read")
    }

    /// The Lua callback this parse holds, or `LUA_NOREF`.
    pub(crate) fn rhs_lua(&self) -> LuaRef {
        self.rhs.as_ref().map_or(LUA_NOREF, MapRhs::luaref)
    }

    /// How long the RHS is in typeahead form.
    pub(crate) fn rhs_len(&self) -> size_t {
        self.rhs.as_ref().map_or(0, |rhs| rhs.str.len())
    }
}

/// What a `:map`-family command does: `do_map`'s `maptype`.
pub const MAPTYPE_MAP: c_int = 0;
pub const MAPTYPE_UNMAP: c_int = 1;
pub const MAPTYPE_NOREMAP: c_int = 2;
/// `:unmap` of a LHS only, not a RHS that happens to match (`mapset()`).
pub const MAPTYPE_UNMAP_LHS: c_int = 3;
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
/// How many hash buckets the mapping table has.
pub const MAX_MAPHASH: usize = 256;
/// The error texts `mapping` raises for itself; the rest come from the
/// shared table in `message`.
pub const E_GLOBAL_ABBREVIATION_ALREADY_EXISTS_FOR_STR: &CStr =
    c"E224: Global abbreviation already exists for %s";
pub const E_GLOBAL_MAPPING_ALREADY_EXISTS_FOR_STR: &CStr =
    c"E225: Global mapping already exists for %s";
pub const E_ABBREVIATION_ALREADY_EXISTS_FOR_STR: &CStr =
    c"E226: Abbreviation already exists for %s";
pub const E_MAPPING_ALREADY_EXISTS_FOR_STR: &CStr = c"E227: Mapping already exists for %s";
pub const E_ENTRIES_MISSING_IN_MAPSET_DICT_ARGUMENT: &CStr =
    c"E460: Entries missing in mapset() dict argument";
pub const E_ILLEGAL_MAP_MODE_STRING_STR: &CStr = c"E1276: Illegal map mode string: '%s'";

/// The largest value a `'langmap'` pair's target can have and still fit in
/// `langmap_mapchar`.
pub const UCHAR_MAX: ::core::ffi::c_int = 255;

/// The NUL an empty [`MapStr`] hands out instead of owning one.
static EMPTY_MAP_STR: [u8; 1] = [0];

/// An owned, NUL-terminated byte string: what the mapping tables keep instead
/// of the `xstrdup`ed `char *` upstream carries.
///
/// The buffer always ends in a NUL, so [`MapStr::as_ptr`] hands a C string to
/// the callees that still take one, and [`MapStr::as_bytes`] is everything
/// before it.  Mapping text never contains an interior NUL — a typed NUL
/// reaches the tables as the three-byte `K_SPECIAL` escape — so the two views
/// agree.
///
/// The empty string owns nothing.  That is not a micro-optimisation: a
/// `<Nop>` mapping's RHS is empty, and the mapping table's per-entry heap is
/// exactly what `inbench`'s bucket walk measures (see the module docs of
/// `mapping`).
#[derive(Debug, Default)]
pub(crate) struct MapStr(Option<Box<[u8]>>);

impl MapStr {
    /// Copy `bytes`, appending the NUL.
    pub(crate) fn new(bytes: &[u8]) -> Self {
        if bytes.is_empty() {
            return Self(None);
        }
        let mut owned = Vec::with_capacity(bytes.len() + 1);
        owned.extend_from_slice(bytes);
        owned.push(0);
        Self(Some(owned.into_boxed_slice()))
    }

    /// The empty string, which owns nothing.
    pub(crate) fn empty() -> Self {
        Self(None)
    }

    /// The content, without the trailing NUL.
    #[inline]
    pub(crate) fn as_bytes(&self) -> &[u8] {
        match &self.0 {
            Some(owned) => &owned[..owned.len() - 1],
            None => &[],
        }
    }

    /// The content *with* its NUL, which is how an index-for-pointer
    /// translation keeps a walk that stops on the terminator byte-exact.
    pub(crate) fn as_bytes_with_nul(&self) -> &[u8] {
        self.0.as_deref().unwrap_or(&EMPTY_MAP_STR)
    }

    /// The content as a C string, which is what the message layer takes.
    pub(crate) fn as_cstr(&self) -> &CStr {
        crate::cstr::in_bytes(self.as_bytes_with_nul())
    }

    /// How many bytes the content is.
    pub(crate) fn len(&self) -> usize {
        self.0.as_deref().map_or(0, |owned| owned.len() - 1)
    }

    /// Whether the content is empty.
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_none()
    }
}

impl Clone for MapStr {
    fn clone(&self) -> Self {
        Self::new(self.as_bytes())
    }
}

/// A Lua callback a mapping calls instead of inserting its right-hand side.
///
/// The one part of a mapping a simplified pair genuinely has to *share*: a
/// reference into Lua's registry cannot be duplicated the way a string can,
/// and it must be released exactly once.  `impl Drop` is in `mapping::table`,
/// because releasing it is the one thing about a mapping that is not plain
/// memory.  A mapping without a callback pays nothing for this.
#[derive(Debug)]
pub(crate) struct MapCallback(pub(crate) LuaRef);

/// The right-hand side of a mapping: three strings and an optional callback.
///
/// Held *by value* on the entry.  Upstream keeps one copy per simplified pair
/// and leaves it owned by whichever of the two is freed second, using `m_alt`
/// as the "the other one still owns them" flag; here the twin gets its own
/// copy of the strings — the pair is rare, the strings are short, and a
/// shared box would be an extra allocation and an extra indirection on every
/// entry of every bucket walk.  Only [`MapCallback`] is shared, and `m_alt`
/// is now no more than the twin's address.
#[derive(Debug)]
pub(crate) struct MapRhs {
    /// The RHS in typeahead form: termcodes replaced, `K_SPECIAL` escaped.
    pub(crate) str: MapStr,
    /// The RHS as it was written, which is `maparg()`'s compatible answer.
    pub(crate) orig_str: MapStr,
    /// `:map <desc>`'s text, or `None`.
    pub(crate) desc: Option<MapStr>,
    /// The Lua callback, shared with the twin.
    pub(crate) lua: Option<Rc<MapCallback>>,
}

impl MapRhs {
    /// A second entry's copy: the strings are duplicated, the callback is
    /// shared.
    pub(crate) fn dup(&self) -> Self {
        Self {
            str: self.str.clone(),
            orig_str: self.orig_str.clone(),
            desc: self.desc.clone(),
            lua: self.lua.clone(),
        }
    }

    /// The Lua callback's reference, or `LUA_NOREF`.
    pub(crate) fn luaref(&self) -> LuaRef {
        self.lua.as_ref().map_or(LUA_NOREF, |lua| lua.0)
    }
}

/// A mapping's identity, which no other mapping ever takes, even after it
/// is deleted.
///
/// What a holder keeps instead of an address: a mapping can be deleted
/// under anything that lets user code run (an `<expr>` that `:unmap`s
/// itself), and a lookup by id then answers `None` where an address would
/// dangle.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) struct MapId(pub(crate) u64);

/// The unsimplified twin of a simplified mapping (or the other way round):
/// its id, and the first byte of its LHS, which is what names the two hash
/// buckets it can be in.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) struct MapTwin {
    pub(crate) id: MapId,
    pub(crate) first: u8,
}

/// One mapping or abbreviation.
///
/// Owned by the list it is on -- a hash bucket of a [`MapTables`], or its
/// abbreviation list -- by value. The twin of a `<C-H>`-style pair is named
/// by id, not by address; everything else the entry holds is its own.
#[derive(Debug)]
pub struct MapBlock {
    /// This entry's identity.
    pub(crate) m_id: MapId,
    /// The unsimplified twin of a simplified mapping, or the other way round.
    pub(crate) m_alt: Option<MapTwin>,
    /// The LHS, in typeahead form.
    pub(crate) m_keys: MapStr,
    /// The RHS: three strings and, when there is one, a shared callback.
    pub(crate) m_rhs: MapRhs,
    /// The modes this mapping applies in.
    pub(crate) m_mode: ::core::ffi::c_int,
    /// Whether this is the `<C-H>`-style simplified half of a pair.
    pub(crate) m_simplified: bool,
    /// `Remap::Yes`, `Remap::No` or `Remap::Script`.
    pub(crate) m_noremap: crate::getchar::Remap,
    pub(crate) m_silent: bool,
    pub(crate) m_nowait: bool,
    pub(crate) m_expr: bool,
    pub(crate) m_script_ctx: ScriptCtx,
    pub(crate) m_replace_keycodes: bool,
}

/// A copy for a listing that has to let go of the table before it prints:
/// the strings are duplicated, the callback is shared, the identity kept.
impl Clone for MapBlock {
    fn clone(&self) -> Self {
        Self {
            m_id: self.m_id,
            m_alt: self.m_alt,
            m_keys: self.m_keys.clone(),
            m_rhs: self.m_rhs.dup(),
            m_mode: self.m_mode,
            m_simplified: self.m_simplified,
            m_noremap: self.m_noremap,
            m_silent: self.m_silent,
            m_nowait: self.m_nowait,
            m_expr: self.m_expr,
            m_script_ctx: self.m_script_ctx,
            m_replace_keycodes: self.m_replace_keycodes,
        }
    }
}

impl MapBlock {
    /// The LHS, without its NUL.
    pub(crate) fn keys(&self) -> &[u8] {
        self.m_keys.as_bytes()
    }

    /// The RHS, without its NUL.
    pub(crate) fn rhs(&self) -> &[u8] {
        self.m_rhs.str.as_bytes()
    }

    /// The Lua callback, or `LUA_NOREF`.
    pub(crate) fn luaref(&self) -> LuaRef {
        self.m_rhs.luaref()
    }

    /// How this entry is named from its twin.
    pub(crate) fn as_twin(&self) -> MapTwin {
        MapTwin {
            id: self.m_id,
            first: self.keys().first().copied().unwrap_or(0),
        }
    }
}

/// One mapping table: [`MAX_MAPHASH`](crate::mapping::MAX_MAPHASH) hash
/// buckets and the abbreviation list. The editor has one globally and one per
/// buffer (`b_maps`).
///
/// Each list keeps its entries oldest first, so a new mapping is a `push` and
/// upstream's head-first walk -- newest first, which is the order `:map`
/// lists in -- is a reverse iteration. The entries are boxed so that
/// deleting one from a long bucket shifts pointers, not mappings. The buckets are allocated on the first
/// mapping, so a buffer without any costs two empty vectors.
#[expect(
    clippy::vec_box,
    reason = "boxed so that deleting from a 5,000-entry bucket shifts pointers"
)]
#[derive(Debug)]
pub(crate) struct MapTables {
    pub(crate) hash: Vec<Vec<Box<MapBlock>>>,
    pub(crate) abbr: Vec<Box<MapBlock>>,
    /// The id the next mapping gets, kept by the global table for every
    /// table: ids are never reused, so one kept across user code can only
    /// find the mapping it was taken from.
    pub(crate) next_id: u64,
}

impl MapTables {
    /// No mappings at all.
    pub(crate) const fn new() -> Self {
        Self {
            hash: Vec::new(),
            abbr: Vec::new(),
            next_id: 1,
        }
    }
}

impl Default for MapTables {
    fn default() -> Self {
        Self::new()
    }
}
