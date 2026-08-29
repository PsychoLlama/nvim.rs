//! Vimscript evaluation: values, expressions, variables and the builtin
//! function table.
//!
//! The deny below reaches the whole subtree — a lint attribute at a `mod.rs`
//! propagates into every `mod foo;` under it — which is why it could only
//! land once every module in `eval/` had been converted.

#![deny(unsafe_op_in_unsafe_fn)]

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
use crate::registry::SlotTable;
use crate::types::{
    Array, Callback, ChannelStreamType, GRegFlags, LuaRetMode, MarkGet, MotionType, Object,
    OptValType, blob_T, dict_T, exprtype_T, funcexe_T, linenr_T, list_T, listwatch_T, lval_T,
    partial_T, size_t, timer_T, typval_T, uint64_t,
};
use crate::winlayer::Live;
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
// `eval0` is reached from `crates/nvim/tests/unit`, which links the library
// from outside; the rest of `expr` stays in-crate.
pub use self::expr::eval0;
/// The handles this tree passes around by pointer.
///
/// Each is a [`Live<T>`](crate::winlayer::Live) — a `Copy` newtype over a
/// `*mut T` recording that whoever built it promised the pointee outlives
/// the value. Construction is the one unsafe step; every `(*p).field` after
/// it is ordinary checked code. See [`crate::winlayer::live`] for why this
/// is not `&mut *p`: the evaluator re-enters itself through autocommands
/// and Lua, and a `&mut` is `noalias` to LLVM.
///
/// They live here rather than in the module that needed each first because
/// the same pointee crosses several of this family's files.
///
/// A value the evaluator is working on. The `*const typval_T` arguments the
/// builtins take are wrapped with `cast_mut()` and only read.
pub(crate) type Tv = Live<typval_T>;

/// A callback the user handed a builtin, and its ownership of a name or a
/// partial.
pub(crate) type Cb = Live<Callback>;

/// A registered timer. The promise is discharged by the reference count:
/// nothing here holds one across a call that has not taken a reference.
pub(crate) type Tm = Live<timer_T>;

/// One `:for` loop's iteration state, owned by the `:endfor` that frees it.
pub(crate) type Fi = Live<forinfo_T>;

/// The left-hand side [`get_lval`] parsed, owned by the caller's frame.
pub(crate) type Lv = Live<lval_T>;

pub const _ISalnum: c_uint = 8;
pub const REGSUB_MAGIC: c_uint = 2;
pub const REGSUB_COPY: c_uint = 1;
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
#[derive(Clone)]
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
pub(crate) const e_cannot_slice_dictionary: &CStr = c"E719: Cannot slice a Dictionary";
pub(crate) const e_cannot_index_special_variable: &CStr = c"E909: Cannot index a special variable";
pub(crate) const e_nowhitespace: &CStr = c"E274: No white space allowed before parenthesis";
pub(crate) const e_cannot_index_a_funcref: &CStr = c"E695: Cannot index a Funcref";
pub(crate) const e_variable_nested_too_deep_for_making_copy: &CStr =
    c"E698: Variable nested too deep for making a copy";
pub(crate) const e_string_list_or_blob_required: &CStr = c"E1098: String, List or Blob required";
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
/// Every live timer, by `timer_id`. See [`crate::registry`] for the order
/// this keeps and the reentrancy rule it answers: a timer's callback runs
/// Vimscript, which can start and stop timers, so nothing holds a borrow of
/// this across one.
static timers: GlobalCell<SlotTable<uint64_t, *mut timer_T>> = GlobalCell::new(SlotTable::new());
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
