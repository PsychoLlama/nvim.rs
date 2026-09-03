//! Mappings and abbreviations: the `:map` family and the table behind it.
//!
//! The module is carved into children along the code's own seams — see each
//! child's own docs.  Everything left here is what they share: the constants
//! the transpile hoisted out of `mapping.c`'s includes, the [`MapArguments`]
//! struct every `:map` command is parsed into, and the six error texts.  The
//! children reach all of it through `use super::*`; the two tables
//! themselves live in [`table`].
//!
//! # Ownership
//!
//! Nothing in this module is freed by hand.  A mapping's four strings are
//! owned: the LHS is a [`MapStr`] and the three RHS ones a [`MapRhs`], both
//! held by value on the entry.  Upstream keeps *one* RHS per `<C-H>`-style
//! pair and leaves it owned by whichever of the two is freed second, using
//! `m_alt` as the "the other one still owns them" flag; here the twin gets
//! its own copy — the pair is rare, and a shared box would be an allocation
//! and an indirection on every entry of every bucket walk.  Only the Lua
//! callback is shared (an [`Rc<MapCallback>`], `None` when there is none),
//! because a registry reference cannot be duplicated; `m_alt` is now no more
//! than the twin's address.
//! The entry itself is a `Box` its list holds by raw pointer, so a mapping
//! keeps the stable address `getchar`'s match loop and the delete-walk need,
//! and [`mapblock_free`] is `Box::from_raw` plus one unlink.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::types::{NL, sctx_T};
use core::ffi::{CStr, c_int};

use crate::api::private::converter::object_to_vim_take_luaref;
use crate::api::private::helpers::{
    api_free_object, api_set_sctx, arena_dict, arena_take_arraybuilder, cstr_as_string,
    find_buffer_by_handle, string_to_cstr,
};
use crate::ascii::{ascii_isspace, ascii_iswhite};
use crate::charset::{skipwhite, transchar, vim_iswordp};
use crate::cmdexpand::cmdline_fuzzy_complete;
use crate::cmdexpand::fuzzymatches_to_strmatches;
use crate::eval::typval::{
    tv_check_for_dict_arg, tv_dict_alloc_ret, tv_dict_find, tv_dict_get_bool, tv_dict_get_number,
    tv_dict_get_string_alloc, tv_get_bool, tv_get_number, tv_get_string_buf, tv_get_string_buf_chk,
    tv_list_alloc_ret, tv_list_append_dict,
};
use crate::eval::userfunc::find_func;
use crate::eval::vars::set_vim_var_char;
use crate::eval::{eval_to_string, last_set_msg};
use crate::ex_cmds::check_secure;
use crate::ex_session::put_eol_unchecked;
use crate::fuzzy::fuzzy_match_str;
use crate::getchar::{ins_typebuf, noremap_keys};
use crate::global_cell::GlobalCell;
use crate::highlight_group::HLF_8;
use crate::keycodes::{
    K_SPECIAL, get_special_key_name, replace_termcodes, vim_strsave_escape_ks, vim_unescape_ks,
};
use crate::lua::executor::{
    api_free_luaref, api_new_luaref, nlua_call_ref, nlua_funcref_str, nlua_set_sctx,
};
use crate::main::{
    State, curbuf, current_sctx, curwin, e_invarg, e_noabbr, e_nomap, got_int, langmap_mapchar,
    mapped_ctrl_c, msg_col, msg_row, msg_silent, no_abbr, p_cpo, p_langmap, p_verbose, secure,
};

use crate::mbyte::{
    cluster_len, mb_prevptr, mb_unescape, utf_char2bytes, utf_ptr2char, utf_ptr2len, utfc_ptr2len,
};
use crate::memory::{ARENA_EMPTY, arena_alloc, arena_finish, arena_mem_free, xfree};
use crate::message::{
    emsg, iemsg, message_filtered, msg, msg_clr_eos, msg_ext_set_kind, msg_outtrans,
    msg_outtrans_special, msg_putchar, msg_puts, msg_puts_hl, msg_start, str2special_arena,
    str2special_save,
};
use crate::os::cshim::{gettext, putc, snprintf, strchr};
use crate::regexp::vim_regexec;
use crate::state::{
    MODE_CMDLINE, MODE_INSERT, MODE_LANGMAP, MODE_NORMAL, MODE_OP_PENDING, MODE_SELECT,
    MODE_TERMINAL, MODE_VISUAL,
};
use crate::strings::{sort_strings, vim_snprintf, vim_strchr};
use crate::types::{
    Arena, Array, ArrayBuilder, Buffer, Dict, Error, EvalFuncData, FILE, Integer, KeyDict_keymap,
    LuaRef, LuaRetMode, MapCallback, MapRhs, MapStr, Object, RemapValues, String_0, dict_T,
    exarg_T, expand_T, fuzmatch_str_T, key_value_pair, linenr_T, mapblock_T, optset_T, ptrdiff_t,
    regmatch_T, scid_T, size_t, typval_T, typval_vval_union, uint64_t, varnumber_T,
};
use crate::winlayer::Live;
use ::libc::{abort, fprintf, fputc, fputs, strcasecmp};
use std::rc::Rc;

// The carve of the transpiled module; see each child's docs.
mod table;
pub use self::table::*;
mod args;
pub(crate) use self::args::*;
mod domap;
pub use self::domap::*;
mod show;
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
pub const MAXMAPLEN: ::core::ffi::c_uint = 50;
pub const FUZZY_SCORE_NONE: ::core::ffi::c_int = -2147483648;
pub const REMAP_SCRIPT: RemapValues = -2;
pub const REMAP_NONE: RemapValues = -1;
pub const REMAP_YES: RemapValues = 0;
pub const REPTERM_NO_SIMPLIFY: ::core::ffi::c_uint = 8;
pub const REPTERM_DO_LT: ::core::ffi::c_uint = 2;
pub const REPTERM_FROM_PART: ::core::ffi::c_uint = 1;
pub const kRetObject: LuaRetMode = 0;
/// One mapping or abbreviation, whose caller has promised it is still linked.
///
/// The promise is discharged by the mapping tables: an entry lives until
/// [`mapblock_free`] takes it off its list, so a handle derived during a walk
/// is good until the walk unlinks something.  The two functions that *delete*
/// while walking hold a [`Cursor`] instead — the address of an entry's own
/// `m_next`, which [`Live`]'s `DerefMut` would invalidate.
pub(crate) type Mb = Live<mapblock_T>;

/// The `:map` arguments being parsed, and the owner of everything a parse
/// allocates.
///
/// Nothing here is freed by hand.  The two LHS spellings are [`MapStr`]s; the
/// right-hand side is the [`MapRhs`] bundle every mapblock this parse creates
/// will *share*, so `map_add` clones the [`Rc`] rather than taking a pointer
/// and the caller no longer has to null its own fields out afterwards.  If no
/// mapping is created, dropping the struct releases the Lua reference too.
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

impl Default for MapArguments {
    fn default() -> Self {
        Self {
            buffer: false,
            expr: false,
            noremap: false,
            nowait: false,
            script: false,
            silent: false,
            unique: false,
            replace_keycodes: false,
            lhs: MapStr::empty(),
            lhs_len: 0,
            alt_lhs: MapStr::empty(),
            alt_lhs_len: 0,
            rhs: None,
            rhs_is_noop: false,
            desc: None,
        }
    }
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

pub const MAPTYPE_UNMAP: ::core::ffi::c_uint = 1;
pub const MAPTYPE_NOREMAP: ::core::ffi::c_uint = 2;
pub const MAPTYPE_UNMAP_LHS: ::core::ffi::c_uint = 3;
pub const MAPTYPE_MAP: ::core::ffi::c_uint = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const ARRAY_DICT_INIT: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const KEYSET_OPTIDX_keymap__desc: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_keymap__callback: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
/// How many hash buckets the mapping table has.
pub const MAX_MAPHASH: usize = 256;
pub const ABBR_OFF: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const KS_SPECIAL: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
pub const KS_EXTRA: ::core::ffi::c_int = 253 as ::core::ffi::c_int;
pub const KS_MODIFIER: ::core::ffi::c_int = 252 as ::core::ffi::c_int;
/// The error texts `mapping` raises for itself; the rest come from the
/// shared table in `main`.
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
