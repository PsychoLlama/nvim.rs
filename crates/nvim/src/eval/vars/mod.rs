#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::types::kOptValTypeBoolean;
use crate::types::kOptValTypeNumber;
use crate::types::kOptValTypeString;
use core::ffi::{CStr, c_char, c_int, c_uint, c_void};
use core::mem::ManuallyDrop;

use crate::eval::gc::{RootId, unroot_dict};
use crate::eval::typval::{DictCursor, ListRef, RemovedItem};

use crate::ascii::ascii_iswhite;
use crate::charset::skip;
use crate::drawscreen::state::sc_col;
use crate::drawscreen::{UPD_SOME_VALID, redraw_all_later};
use crate::eval::encode::{encode_tv2echo, encode_tv2string};
use crate::eval::executor::eexe_mod_op;
use crate::eval::funcs::{tv_get_buf, tv_get_buf_from_arg};
use crate::eval::typval::{
    LockName, TV_INITIAL_VALUE, dict_is_watched, dict_watcher_notify, list_find_nr, list_find_str,
    list_len, list_set_lock, tv_check_str_or_nr, tv_clear, tv_copy, tv_dict_alloc,
    tv_dict_alloc_lock, tv_get_bool_chk, tv_get_number, tv_get_number_chk, tv_item_lock,
    tv_list_alloc, value_check_lock,
};
use crate::eval::userfunc::{
    current_func_has_scope, funccal_scope, function_exists, list_func_vars, walk_scoped_funccals,
    with_funccal_scope_entry,
};
use crate::eval::window::find_win_by_nr;
use crate::eval::{
    Cursor, LAMBDA_USES_LOCALS, eval_expr_ext, eval_isnamec1, eval_option, eval_to_bool, eval1,
    get_name_len, handle_subscript, may_call_simple_func, name_end, set_ref_in_dict_items,
};
use crate::ex_cmds::check_secure;
use crate::ex_docmd::ends_excmd;
use crate::ex_eval::aborting;
use crate::getchar::state::got_int;
use crate::global_cell::{GlobalCell, state_record};
use crate::guard::sandbox;
use crate::hashtab::hash_reset;
use crate::lua::executor::nlua_set_sctx_in;
use crate::memory::XString;
use crate::message::state::emsg_severe;
use crate::message::{
    e_cannot_change_readonly_variable_str, e_cannot_mod, e_cannot_set_variable_in_sandbox_str,
    e_string_required,
};
use crate::message::{
    emsg, internal_error, message_filtered, msg_advance, msg_bytes, msg_clr_eos, msg_display_bytes,
    msg_ext_set_kind, msg_putchar, msg_start, msg_str,
};
use crate::option::vars::{p_ccv, p_dex, p_pex, p_verbose};
use crate::option::{
    find_option, get_option, get_winbuf_options, is_tty_option, kOptFlagFunc, option_has_type,
    option_last_set, optval_free,
};
use crate::options::{kOptCharconvert, kOptDiffexpr, kOptInvalid, kOptPatchexpr, kOptSpellsuggest};
use crate::os::cshim::gettext;
use crate::pos::MAXCOL;
use crate::runtime::state::current_sctx;
use crate::runtime::{
    new_unnamed_script_item, script_autoload_named, script_count, script_id_valid, with_script_item,
};
use crate::search::set_search_direction;
use crate::search::state::no_hlsearch;
use crate::types::{
    BoolVarValue, Dict, DictItem, DictKey, EvalFuncData, ExArg, Expand, GRegFlags, List, OptIndex,
    OptVal, Partial, ScopeDictItem, ScopeType, ScriptId, ScriptVar, SpecialVarValue, TypVal,
    VAR_BLOB, VAR_BOOL, VAR_DEF_SCOPE, VAR_DICT, VAR_FLOAT, VAR_FUNC, VAR_LIST, VAR_NUMBER,
    VAR_PARTIAL, VAR_SCOPE, VAR_SPECIAL, VAR_STRING, VAR_TYPE_BLOB, VAR_TYPE_BOOL, VAR_TYPE_DICT,
    VAR_TYPE_FLOAT, VAR_TYPE_FUNC, VAR_TYPE_LIST, VAR_TYPE_NUMBER, VAR_TYPE_STRING, VAR_UNKNOWN,
    VarLock, VarNumber, VarType, VimVarFlags, Vv, int64_t, kBoolVarFalse, kBoolVarTrue,
    kListLenUnknown, kSpecialVarNull, ptrdiff_t, size_t, uint8_t, uint32_t,
};
use crate::version::{highest_patch, min_vim_version};
use crate::window::{find_tabpage, goto_tabpage_tp, prevwin_curwin, valid_tabpage};
use crate::winlayer::graph::lastused_tabpage;

// The carve of the transpiled module; see each child's docs.
mod assign;
mod external;
mod heredoc;
mod lifecycle;
mod listing;
mod lookup;
mod redir;
mod scoped;
mod store;
mod unlet;
mod vvar;

pub use self::assign::*;
pub use self::external::*;
pub(crate) use self::heredoc::*;
pub use self::lifecycle::*;
pub(crate) use self::listing::*;
pub use self::lookup::*;
pub use self::redir::*;
pub use self::scoped::*;
pub(crate) use self::store::*;
pub use self::unlet::*;
pub use self::vvar::*;
/// One of the `list_*_vars` scope listers: everything a bare `g:`/`b:`/`w:`/
/// ... can name, whether on a `:let` line or as the whole of one.
pub(crate) type ScopeLister = fn(&mut bool);

/// `__ctype_b_loc()`'s lower-case bit, the one `islower()` reads.
pub const _ISlower: c_uint = 512;

/// The reference count `init_var_dict` gives a scope dictionary: high enough
/// that nothing ever frees one.
pub const DO_NOT_FREE_CNT: c_int = 1073741823;

/// `DictItem::di_flags`.
pub const DI_FLAGS_ALLOC: uint8_t = 16;
pub const DI_FLAGS_LOCK: uint8_t = 8;
pub const DI_FLAGS_FIX: uint8_t = 4;
pub const DI_FLAGS_RO_SBX: uint8_t = 2;
pub const DI_FLAGS_RO: uint8_t = 1;

/// `get_lval`'s "do not report" flag.
pub const GLV_QUIET: c_int = 2;

pub const kGRegExprSrc: GRegFlags = 2;

pub const NULL: *mut c_void = ::core::ptr::null_mut::<c_void>();
pub const INT64_MIN: ::core::ffi::c_long = -9223372036854775807 - 1;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615;

pub const VARNUMBER_MAX: ::core::ffi::c_long = INT64_MAX;
pub const VARNUMBER_MIN: ::core::ffi::c_long = INT64_MIN;
pub const BAD_KEEP: c_int = -1;
pub const BAD_DROP: c_int = -2;
pub const FORCE_BIN: c_int = 1;
pub const FORCE_NOBIN: c_int = 2;
pub const NOTDONE: c_int = 2;
pub const CHAN_STDERR: c_int = 2;
pub const FNE_INCL_BR: c_int = 1;
pub const FNE_CHECK_START: c_int = 2;
pub const AUTOLOAD_CHAR: c_char = b'#'.cast_signed();

/// The two `name_len` sentinels the `var_check_*` family accepts in place of
/// a real length: translate the name and measure it, or just measure it.
pub const TV_TRANSLATE: ::core::ffi::c_ulong = SIZE_MAX;
pub const TV_CSTRING: ::core::ffi::c_ulong = SIZE_MAX - 1;

/// How deep `:const` locks the value it stores.
pub const DICT_MAXNEST: c_int = 100;

pub const SID_LUA: c_int = -8;
pub const SID_STR: c_int = -10;

// The error texts this family owns.  They are `%`-format strings handed to
// the variadic `semsg`/`emsg`, so they stay C strings rather than becoming
// `semsg!` arguments.
pub const e_letunexp: &CStr = c"E18: Unexpected characters in :let";
pub const e_double_semicolon_in_list_of_variables: &CStr = c"E452: Double ; in list of variables";
pub const e_lock_unlock: &CStr = c"E940: Cannot lock or unlock variable %s";
pub const e_setting_v_str_to_value_with_wrong_type: &CStr =
    c"E963: Setting v:%s to value with wrong type";
pub const e_missing_end_marker_str: &CStr = c"E990: Missing end marker '%s'";
pub const e_cannot_use_heredoc_here: &CStr = c"E991: Cannot use =<< here";

/// A scope's entry before its dictionary exists: what a bare `g:` or `v:`
/// names until [`globvar_dict`] or [`vimvar_dict`] first builds the scope.
const EMPTY_SCOPE_VAR: ScopeDictItem = ScopeDictItem(ManuallyDrop::new(DictItem {
    di_tv: TypVal::empty(VAR_UNKNOWN),
    di_lock: VarLock::Unlocked,
    di_flags: 0,
    di_key: DictKey::EMPTY,
}));

state_record! {
    /// The editor-wide variable scopes: `g:`, `v:` and what hangs off `v:`.
    pub(crate) struct VarScopes in VAR_SCOPES as VarScopesField;

    /// The entry a bare `g:` resolves to. Its value is the `g:`
    /// dictionary, a heap dictionary that nothing frees.
    pub(crate) scope_globals: ScopeDictItem = EMPTY_SCOPE_VAR;
    /// The entry a bare `v:` resolves to, likewise, and where each row is.
    pub(crate) scope_vim: VimScope = VimScope {
        entry: EMPTY_SCOPE_VAR,
        slots: [(0, 0); VIMVAR_COUNT],
    };
    /// The `v:msgpack_types` lists, which the msgpack encoder and decoder
    /// compare by identity.
    pub(crate) msgpack_type_lists: [Option<ListRef>; 8] = [const { None }; 8];
    /// The address of `v:lua`'s partial, which is set once and never
    /// replaced (`v:lua` is read-only): what [`is_lua_partial`] compares
    /// against, on every call through a partial.
    pub(crate) lua_partial_addr: usize = 0;
    /// Whether `v:testing` is non-zero, kept beside the variable by every
    /// write to it ([`before_set_vvar`], [`set_vim_var_nr`]): every function
    /// call asks.
    pub(crate) vim_testing: bool = false;
    /// `v:val` and `v:key` while they are not in the `v:` dictionary.
    pub(crate) outside_vimvars: [Option<Box<DictItem>>; 2] = [None, None];
}

/// The `v:` scope: the entry a bare `v:` resolves to, whose value is the
/// `v:` dictionary, and a hint per row of where it is in that dictionary.
pub(crate) struct VimScope {
    pub(crate) entry: ScopeDictItem,
    /// Where each row last was in the dictionary's table, and the address
    /// of the row's item there: used only while the slot still holds that
    /// very item. A row's item is never freed, so the address names it for
    /// the life of the editor. Kept beside the entry so that one access to
    /// the record finds both.
    pub(crate) slots: [(usize, usize); VIMVAR_COUNT],
}

/// One row of the `v:` table: the name, the type the variable is declared
/// with, and its flags.
pub(crate) struct VimVarRow {
    pub(crate) name: &'static CStr,
    pub(crate) declared: VarType,
    pub(crate) flags: VimVarFlags,
}

const fn vv(name: &'static CStr, declared: VarType, flags: VimVarFlags) -> VimVarRow {
    VimVarRow {
        name,
        declared,
        flags,
    }
}

/// How many rows the `v:` table has; one per `Vv` discriminant.
pub(crate) const VIMVAR_COUNT: usize = 106;

/// The `v:` table, in [`Vv`] order -- which is also the order the rows go
/// into the `v:` dictionary, and so what `keys(v:)` answers.
pub(crate) static VIMVAR_ROWS: [VimVarRow; VIMVAR_COUNT] = [
    vv(c"count", VAR_NUMBER, VimVarFlags::RO),
    vv(c"count1", VAR_NUMBER, VimVarFlags::RO),
    vv(c"prevcount", VAR_NUMBER, VimVarFlags::RO),
    vv(c"errmsg", VAR_STRING, VimVarFlags::NONE),
    vv(c"warningmsg", VAR_STRING, VimVarFlags::NONE),
    vv(c"statusmsg", VAR_STRING, VimVarFlags::NONE),
    vv(c"shell_error", VAR_NUMBER, VimVarFlags::RO),
    vv(c"this_session", VAR_STRING, VimVarFlags::NONE),
    vv(
        c"version",
        VAR_NUMBER,
        VimVarFlags::COMPAT.or(VimVarFlags::RO),
    ),
    vv(c"lnum", VAR_NUMBER, VimVarFlags::RO_SBX),
    vv(c"termrequest", VAR_STRING, VimVarFlags::RO),
    vv(c"termresponse", VAR_STRING, VimVarFlags::RO),
    vv(c"fname", VAR_STRING, VimVarFlags::RO),
    vv(c"lang", VAR_STRING, VimVarFlags::RO),
    vv(c"lc_time", VAR_STRING, VimVarFlags::RO),
    vv(c"ctype", VAR_STRING, VimVarFlags::RO),
    vv(c"charconvert_from", VAR_STRING, VimVarFlags::RO),
    vv(c"charconvert_to", VAR_STRING, VimVarFlags::RO),
    vv(c"fname_in", VAR_STRING, VimVarFlags::RO),
    vv(c"fname_out", VAR_STRING, VimVarFlags::RO),
    vv(c"fname_new", VAR_STRING, VimVarFlags::RO),
    vv(c"fname_diff", VAR_STRING, VimVarFlags::RO),
    vv(c"cmdarg", VAR_STRING, VimVarFlags::RO),
    vv(c"foldstart", VAR_NUMBER, VimVarFlags::RO_SBX),
    vv(c"foldend", VAR_NUMBER, VimVarFlags::RO_SBX),
    vv(c"folddashes", VAR_STRING, VimVarFlags::RO_SBX),
    vv(c"foldlevel", VAR_NUMBER, VimVarFlags::RO_SBX),
    vv(c"progname", VAR_STRING, VimVarFlags::RO),
    vv(c"servername", VAR_STRING, VimVarFlags::RO),
    vv(c"dying", VAR_NUMBER, VimVarFlags::RO),
    vv(c"exception", VAR_STRING, VimVarFlags::RO),
    vv(c"throwpoint", VAR_STRING, VimVarFlags::RO),
    vv(c"register", VAR_STRING, VimVarFlags::RO),
    vv(c"cmdbang", VAR_NUMBER, VimVarFlags::RO),
    vv(c"insertmode", VAR_STRING, VimVarFlags::RO),
    vv(c"val", VAR_UNKNOWN, VimVarFlags::RO),
    vv(c"key", VAR_UNKNOWN, VimVarFlags::RO),
    vv(c"profiling", VAR_NUMBER, VimVarFlags::RO),
    vv(c"fcs_reason", VAR_STRING, VimVarFlags::RO),
    vv(c"fcs_choice", VAR_STRING, VimVarFlags::NONE),
    vv(c"beval_bufnr", VAR_NUMBER, VimVarFlags::RO),
    vv(c"beval_winnr", VAR_NUMBER, VimVarFlags::RO),
    vv(c"beval_winid", VAR_NUMBER, VimVarFlags::RO),
    vv(c"beval_lnum", VAR_NUMBER, VimVarFlags::RO),
    vv(c"beval_col", VAR_NUMBER, VimVarFlags::RO),
    vv(c"beval_text", VAR_STRING, VimVarFlags::RO),
    vv(c"scrollstart", VAR_STRING, VimVarFlags::NONE),
    vv(c"swapname", VAR_STRING, VimVarFlags::RO),
    vv(c"swapchoice", VAR_STRING, VimVarFlags::NONE),
    vv(c"swapcommand", VAR_STRING, VimVarFlags::RO),
    vv(c"char", VAR_STRING, VimVarFlags::NONE),
    vv(c"mouse_win", VAR_NUMBER, VimVarFlags::NONE),
    vv(c"mouse_winid", VAR_NUMBER, VimVarFlags::NONE),
    vv(c"mouse_lnum", VAR_NUMBER, VimVarFlags::NONE),
    vv(c"mouse_col", VAR_NUMBER, VimVarFlags::NONE),
    vv(c"operator", VAR_STRING, VimVarFlags::RO),
    vv(c"searchforward", VAR_NUMBER, VimVarFlags::NONE),
    vv(c"hlsearch", VAR_NUMBER, VimVarFlags::NONE),
    vv(c"oldfiles", VAR_LIST, VimVarFlags::NONE),
    vv(c"windowid", VAR_NUMBER, VimVarFlags::RO_SBX),
    vv(c"progpath", VAR_STRING, VimVarFlags::RO),
    vv(c"completed_item", VAR_DICT, VimVarFlags::NONE),
    vv(c"option_new", VAR_STRING, VimVarFlags::RO),
    vv(c"option_old", VAR_STRING, VimVarFlags::RO),
    vv(c"option_oldlocal", VAR_STRING, VimVarFlags::RO),
    vv(c"option_oldglobal", VAR_STRING, VimVarFlags::RO),
    vv(c"option_command", VAR_STRING, VimVarFlags::RO),
    vv(c"option_type", VAR_STRING, VimVarFlags::RO),
    vv(c"errors", VAR_LIST, VimVarFlags::NONE),
    vv(c"false", VAR_BOOL, VimVarFlags::RO),
    vv(c"true", VAR_BOOL, VimVarFlags::RO),
    vv(c"null", VAR_SPECIAL, VimVarFlags::RO),
    vv(c"numbermax", VAR_NUMBER, VimVarFlags::RO),
    vv(c"numbermin", VAR_NUMBER, VimVarFlags::RO),
    vv(c"numbersize", VAR_NUMBER, VimVarFlags::RO),
    vv(c"vim_did_enter", VAR_NUMBER, VimVarFlags::RO),
    vv(c"testing", VAR_NUMBER, VimVarFlags::NONE),
    vv(c"t_number", VAR_NUMBER, VimVarFlags::RO),
    vv(c"t_string", VAR_NUMBER, VimVarFlags::RO),
    vv(c"t_func", VAR_NUMBER, VimVarFlags::RO),
    vv(c"t_list", VAR_NUMBER, VimVarFlags::RO),
    vv(c"t_dict", VAR_NUMBER, VimVarFlags::RO),
    vv(c"t_float", VAR_NUMBER, VimVarFlags::RO),
    vv(c"t_bool", VAR_NUMBER, VimVarFlags::RO),
    vv(c"t_blob", VAR_NUMBER, VimVarFlags::RO),
    vv(c"event", VAR_DICT, VimVarFlags::RO),
    vv(c"versionlong", VAR_NUMBER, VimVarFlags::RO),
    vv(c"echospace", VAR_NUMBER, VimVarFlags::RO),
    vv(c"argf", VAR_LIST, VimVarFlags::RO),
    vv(c"argv", VAR_LIST, VimVarFlags::RO),
    vv(c"collate", VAR_STRING, VimVarFlags::RO),
    vv(c"exiting", VAR_NUMBER, VimVarFlags::RO),
    vv(c"maxcol", VAR_NUMBER, VimVarFlags::RO),
    vv(c"stacktrace", VAR_LIST, VimVarFlags::RO),
    vv(c"vim_did_init", VAR_NUMBER, VimVarFlags::RO),
    vv(c"stderr", VAR_NUMBER, VimVarFlags::RO),
    vv(c"msgpack_types", VAR_DICT, VimVarFlags::RO),
    vv(c"_null_string", VAR_STRING, VimVarFlags::RO),
    vv(c"_null_list", VAR_LIST, VimVarFlags::RO),
    vv(c"_null_dict", VAR_DICT, VimVarFlags::RO),
    vv(c"_null_blob", VAR_BLOB, VimVarFlags::RO),
    vv(c"lua", VAR_PARTIAL, VimVarFlags::RO),
    vv(c"relnum", VAR_NUMBER, VimVarFlags::RO),
    vv(c"virtnum", VAR_NUMBER, VimVarFlags::RO),
    vv(c"starttime", VAR_NUMBER, VimVarFlags::RO),
    vv(c"exitreason", VAR_STRING, VimVarFlags::RO),
];

/// The eight `v:msgpack_types` keys, in `MessagePackType` order.
const msgpack_type_names: [&CStr; 8] = [
    c"nil", c"boolean", c"integer", c"float", c"string", c"array", c"map", c"ext",
];
