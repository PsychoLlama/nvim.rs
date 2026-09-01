//! Mappings and abbreviations: the `:map` family and the table behind it.
//!
//! The module is carved into children along the code's own seams — see each
//! child's own docs.  Everything left here is what they share: the constants
//! the transpile hoisted out of `mapping.c`'s includes, the [`MapArguments`]
//! struct every `:map` command is parsed into, and the six error texts.  The
//! children reach all of it through `use super::*`; the two tables
//! themselves live in [`table`].

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::CStr;

use crate::api::private::converter::object_to_vim_take_luaref;
use crate::api::private::helpers::{
    api_free_object, api_set_sctx, arena_dict, arena_take_arraybuilder, cstr_as_string,
    find_buffer_by_handle, string_to_cstr,
};
use crate::ascii::{ascii_isspace, ascii_iswhite};
use crate::charset::{skipwhite, transchar, vim_iswordp};
use crate::cmdexpand::cmdline_fuzzy_complete;
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
use crate::fuzzy::{fuzzy_match_str, fuzzymatches_to_strmatches};
use crate::garray::{ga_append, ga_concat, ga_grow, ga_init};
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
    mb_prevptr, mb_unescape, utf_char2bytes, utf_ptr2char, utf_ptr2len, utfc_ptr2len,
};
use crate::memory::{
    ARENA_EMPTY, arena_alloc, arena_finish, arena_mem_free, xcalloc, xfree, xmemcpyz, xstrdup,
    xstrlcpy,
};
use crate::message::{
    emsg, iemsg, message_filtered, msg, msg_clr_eos, msg_ext_set_kind, msg_outtrans,
    msg_outtrans_special, msg_putchar, msg_puts, msg_puts_hl, msg_start, str2special_arena,
    str2special_save,
};
use crate::os::cshim::{gettext, putc, snprintf, strchr, strstr};
use crate::regexp::vim_regexec;
use crate::state::{
    MODE_CMDLINE, MODE_INSERT, MODE_LANGMAP, MODE_NORMAL, MODE_OP_PENDING, MODE_SELECT,
    MODE_TERMINAL, MODE_VISUAL,
};
use crate::strings::{sort_strings, vim_snprintf, vim_strchr};
use crate::types::{
    Arena, Array, ArrayBuilder, Buffer, Dict, Error, EvalFuncData, FILE, Integer, KeyDict_keymap,
    LuaRef, LuaRetMode, Object, RemapValues, String_0, cmdidx_T, dict_T, exarg_T, expand_T,
    fuzmatch_str_T, garray_T, kObjectTypeLuaRef, kObjectTypeString, key_value_pair, linenr_T,
    mapblock_T, object_data, optset_T, ptrdiff_t, regmatch_T, scid_T, size_t, typval_T,
    typval_vval_union, uint64_t, varnumber_T,
};
use crate::winlayer::Live;
use ::libc::{abort, fprintf, fputc, fputs, strcasecmp, strpbrk};

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
/// while walking keep raw pointers instead — they hold the address of an
/// entry's own `m_next`, and [`Live`]'s `DerefMut` would invalidate it.
pub(crate) type Mb = Live<mapblock_T>;

/// The `:map` arguments being parsed, whose caller has promised the struct
/// outlives the value.
///
/// Every parse writes into one `MapArguments` its caller owns — a local of
/// `do_map`, of `do_exmap` or of an API entry point — so the promise is
/// discharged by that frame outliving the call.
pub(crate) type Ma = Live<MapArguments>;

pub type MapArguments = map_arguments;
#[derive(Copy, Clone)]
pub struct map_arguments {
    pub buffer: bool,
    pub expr: bool,
    pub noremap: bool,
    pub nowait: bool,
    pub script: bool,
    pub silent: bool,
    pub unique: bool,
    pub replace_keycodes: bool,
    pub lhs: [::core::ffi::c_char; 51],
    pub lhs_len: size_t,
    pub alt_lhs: [::core::ffi::c_char; 51],
    pub alt_lhs_len: size_t,
    pub rhs: *mut ::core::ffi::c_char,
    pub rhs_len: size_t,
    pub rhs_lua: LuaRef,
    pub rhs_is_noop: bool,
    pub orig_rhs: *mut ::core::ffi::c_char,
    pub orig_rhs_len: size_t,
    pub desc: *mut ::core::ffi::c_char,
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
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
/// How many hash buckets the mapping table has.
pub const MAX_MAPHASH: usize = 256;
pub const FC_LUAREF: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
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

/// A `MapArguments` with nothing set, which is what every parse starts from.
pub const MAP_ARGUMENTS_INIT: MapArguments = map_arguments {
    buffer: false,
    expr: false,
    noremap: false,
    nowait: false,
    script: false,
    silent: false,
    unique: false,
    replace_keycodes: false,
    lhs: [0; 51],
    lhs_len: 0,
    alt_lhs: [0; 51],
    alt_lhs_len: 0,
    rhs: ::core::ptr::null_mut(),
    rhs_len: 0,
    rhs_lua: LUA_NOREF,
    rhs_is_noop: false,
    orig_rhs: ::core::ptr::null_mut(),
    orig_rhs_len: 0,
    desc: ::core::ptr::null_mut(),
};
/// The largest value a `'langmap'` pair's target can have and still fit in
/// `langmap_mapchar`.
pub const UCHAR_MAX: ::core::ffi::c_int = 255;
