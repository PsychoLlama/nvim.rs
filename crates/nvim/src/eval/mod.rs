pub mod buffer;
pub mod decode;
pub mod deprecated;
pub mod encode;
pub mod executor;
pub mod fs;
pub mod funcs;
pub mod gc;
pub mod list;
pub mod typval;
pub(crate) mod typval_encode;
pub mod userfunc;
pub mod vars;
pub mod window;
use crate::global_cell::GlobalCell;
// Named here so the expression tree and `list.rs` can reach it by one
// path; it belongs to `main`.
pub(crate) use crate::main::e_invalblob;
use crate::types::{
    Array, ChannelStreamType, GRegFlags, LuaRetMode, Map_uint64_t_ptr_t, MapHash, MarkGet,
    MotionType, Object, OptValType, Set_uint64_t, blob_T, dict_T, exprtype_T, funcexe_T, linenr_T,
    list_T, listwatch_T, partial_T, ptr_t, size_t, typval_T, uint32_t, uint64_t,
};
use core::ffi::{CStr, c_char, c_int, c_long, c_uint, c_ulong};

mod entry;
pub use self::entry::*;
mod lval;
pub use self::lval::*;
mod forloop;
pub use self::forloop::*;
mod collect;
pub use self::collect::*;
mod callback;
pub use self::callback::*;
mod timer;
pub use self::timer::*;
mod name;
pub use self::name::*;
mod system;
pub use self::system::*;
mod pos;
pub use self::pos::*;
mod echo;
pub use self::echo::*;
mod provider;
pub use self::provider::*;
mod pattern;
pub use self::pattern::*;
mod expr;
pub(crate) use self::expr::*;
pub const _ISalnum: c_uint = 8;
pub const EXPAND_ENV_VARS: c_int = 26;
pub const EXPAND_EXPRESSION: c_int = 20;
pub const EXPAND_FUNCTIONS: c_int = 18;
pub const EXPAND_USER_VARS: c_int = 15;
pub const EXPAND_SETTINGS: c_int = 4;
pub const EXPAND_COMMANDS: c_int = 1;
pub const EXPAND_NOTHING: c_int = 0;
pub const REGSUB_MAGIC: c_uint = 2;
pub const REGSUB_COPY: c_uint = 1;
pub const kWinOptFoldexpr: c_int = 15;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNil: OptValType = -1;
pub const kMarkAll: MarkGet = 1;
pub const kChannelStreamProc: ChannelStreamType = 0;
pub const STR2NR_ALL: c_uint = 15;
pub const GLV_READ_ONLY: c_uint = 16;
pub const GLV_NO_AUTOLOAD: c_uint = 4;
pub const GLV_QUIET: c_uint = 2;
pub const EXPR_ISNOT: exprtype_T = 10;
pub const EXPR_IS: exprtype_T = 9;
pub const EXPR_NOMATCH: exprtype_T = 8;
pub const EXPR_MATCH: exprtype_T = 7;
pub const EXPR_SEQUAL: exprtype_T = 6;
pub const EXPR_SMALLER: exprtype_T = 5;
pub const EXPR_GEQUAL: exprtype_T = 4;
pub const EXPR_GREATER: exprtype_T = 3;
pub const EXPR_NEQUAL: exprtype_T = 2;
pub const EXPR_EQUAL: exprtype_T = 1;
pub const EXPR_UNKNOWN: exprtype_T = 0;
pub const EVAL_EVALUATE: c_uint = 1;
pub const kGRegExprSrc: GRegFlags = 2;
pub const FSK_IN_STRING: c_uint = 4;
pub const FSK_KEYCODE: c_uint = 1;
pub const FSK_SIMPLIFY: c_uint = 8;
pub const GLV_STOP: glv_status_T = 2;
pub type glv_status_T = c_uint;
pub const GLV_OK: glv_status_T = 1;
pub const GLV_FAIL: glv_status_T = 0;
#[derive(Copy, Clone)]
pub struct forinfo_T {
    pub fi_semicolon: c_int,
    pub fi_varcount: c_int,
    pub fi_lw: listwatch_T,
    pub fi_list: *mut list_T,
    pub fi_bi: c_int,
    pub fi_blob: *mut blob_T,
    pub fi_string: *mut c_char,
    pub fi_byte_idx: c_int,
}
pub const kMTCharWise: MotionType = 0;
pub const kRetNilBool: LuaRetMode = 1;
pub const INT64_MIN: c_long = -9223372036854775807 as c_long - 1 as c_long;
pub const INT64_MAX: c_long = 9223372036854775807 as c_long;
pub const UINT32_MAX: c_uint = 4294967295 as c_uint;
pub const SIZE_MAX: c_ulong = 18446744073709551615 as c_ulong;
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_uint64_t = Set_uint64_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<uint64_t>(),
};
pub const MAP_INIT: Map_uint64_t_ptr_t = Map_uint64_t_ptr_t {
    set: SET_INIT,
    values: ::core::ptr::null_mut::<ptr_t>(),
};
pub const MH_TOMBSTONE: c_uint = UINT32_MAX;
pub const VARNUMBER_MAX: c_long = INT64_MAX;
pub const VARNUMBER_MIN: c_long = INT64_MIN;
pub const BS: c_int = '\u{8}' as c_int;
pub const TAB: c_int = '\t' as c_int;
pub const NL: c_int = '\n' as c_int;
pub const FF: c_int = '\u{c}' as c_int;
pub const CAR: c_int = '\r' as c_int;
pub const ESC: c_int = '\u{1b}' as c_int;
pub const NOTDONE: c_int = 2 as c_int;
pub const COPYID_INC: c_int = 2 as c_int;
pub const COPYID_MASK: c_int = !(0x1 as c_int);
pub const FNE_INCL_BR: c_int = 1 as c_int;
pub const FNE_CHECK_START: c_int = 2 as c_int;
pub const AUTOLOAD_CHAR: c_int = '#' as c_int;
pub const DICT_MAXNEST: c_int = 100 as c_int;

/// The messages this tree reports, as the C spells them. Each is a
/// format string for `semsg` or a plain one for `emsg`; both want a
/// NUL-terminated pointer, which is what `as_ptr()` gives.
pub(crate) const e_missbrac: &CStr = c"E111: Missing ']'";
pub(crate) const e_list_end: &CStr = c"E697: Missing end of List ']': %s";
pub(crate) const e_cannot_slice_dictionary: &CStr = c"E719: Cannot slice a Dictionary";
pub(crate) const e_cannot_index_special_variable: &CStr = c"E909: Cannot index a special variable";
pub(crate) const e_nowhitespace: &CStr = c"E274: No white space allowed before parenthesis";
pub(crate) const e_cannot_index_a_funcref: &CStr = c"E695: Cannot index a Funcref";
pub(crate) const e_variable_nested_too_deep_for_making_copy: &CStr =
    c"E698: Variable nested too deep for making a copy";
pub(crate) const e_string_list_or_blob_required: &CStr = c"E1098: String, List or Blob required";
pub(crate) const e_expression_too_recursive_str: &CStr = c"E1169: Expression too recursive: %s";
pub(crate) const e_dot_can_only_be_used_on_dictionary_str: &CStr =
    c"E1203: Dot can only be used on a dictionary: %s";
pub(crate) const e_empty_function_name: &CStr = c"E1192: Empty function name";
pub(crate) const e_cannot_use_partial_here: &CStr = c"E1265: Cannot use a partial here";

/// The scope letters a `x:` variable prefix may use.
pub(crate) const namespace_char: &CStr = c"abglstvw";

pub static eval_lavars_used: GlobalCell<*mut bool> =
    GlobalCell::new(::core::ptr::null_mut::<bool>());
static echo_hl_id: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
static last_timer_id: GlobalCell<uint64_t> = GlobalCell::new(1 as uint64_t);
static timers: GlobalCell<Map_uint64_t_ptr_t> = GlobalCell::new(MAP_INIT);
static callback_depth: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub const TV_CSTRING: c_ulong = SIZE_MAX.wrapping_sub(1 as c_ulong);
pub const FUNCEXE_INIT: funcexe_T = funcexe_T {
    fe_argv_func: None,
    fe_firstline: 0 as linenr_T,
    fe_lastline: 0 as linenr_T,
    fe_doesrange: ::core::ptr::null_mut::<bool>(),
    fe_evaluate: false,
    fe_partial: ::core::ptr::null_mut::<partial_T>(),
    fe_selfdict: ::core::ptr::null_mut::<dict_T>(),
    fe_basetv: ::core::ptr::null_mut::<typval_T>(),
    fe_found_var: false,
};
pub const PROF_YES: c_int = 1 as c_int;
pub const KS_EXTRA: c_int = 253 as c_int;
pub const INT_MAX: c_int = __INT_MAX__;
pub const __INT_MAX__: c_int = 2147483647 as c_int;
