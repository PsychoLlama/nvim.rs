//! Vimscript evaluation: values, expressions, variables and the builtin
//! function table.
//!
//! The deny below reaches the whole subtree — a lint attribute at a `mod.rs`
//! propagates into every `mod foo;` under it — which is why it could only
//! land once every module in `eval/` had been converted.
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![deny(unsafe_op_in_unsafe_fn)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

pub mod buffer;
pub mod decode;
pub mod deprecated;
pub mod encode;
pub mod executor;
pub mod fs;
pub mod funcs;
pub mod gc;
pub mod list;
#[cfg(test)]
pub(crate) mod test_fixture;
pub mod typval;
pub(crate) mod typval_encode;
pub mod userfunc;
pub mod vars;
pub mod window;

use crate::ascii::NL;
use crate::global_cell::GlobalCell;
// Named here so the expression tree and `list.rs` can reach it by one
// path; it belongs to `message`.
pub(crate) use crate::message::E_INVALBLOB;
use crate::os::libc::uint64_t;
use crate::registry::SlotTable;
use crate::types::{ExprType, Failed, Timer};
use core::ffi::{CStr, c_int, c_uint, c_ulong};

mod entry;
pub use self::entry::*;
mod lval;
pub(crate) use self::lval::*;
mod forloop;
pub(crate) use self::forloop::*;
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
pub(crate) use self::typval::partial_name;
pub(crate) use self::vars::{get_v_event, restore_v_event};
// `eval0` is reached from `crates/nvim/tests/unit`, which links the library
// from outside; the rest of `expr` stays in-crate.
pub use self::expr::eval0;

pub const GLV_READ_ONLY: c_int = 16;
pub const GLV_NO_AUTOLOAD: c_int = 4;
/// `get_lval`'s "do not report" flag.
pub const GLV_QUIET: c_int = 2;
pub const EXPR_ISNOT: ExprType = 10;
pub(crate) const EXPR_IS: ExprType = 9;
pub const EXPR_NOMATCH: ExprType = 8;
pub const EXPR_MATCH: ExprType = 7;
pub const EXPR_SEQUAL: ExprType = 6;
pub const EXPR_SMALLER: ExprType = 5;
pub const EXPR_GEQUAL: ExprType = 4;
pub const EXPR_GREATER: ExprType = 3;
pub const EXPR_NEQUAL: ExprType = 2;
pub const EXPR_EQUAL: ExprType = 1;
pub const EXPR_UNKNOWN: ExprType = 0;
pub const GLV_STOP: GlvStatus = 2;
pub type GlvStatus = c_uint;
pub const GLV_OK: GlvStatus = 1;
pub const GLV_FAIL: GlvStatus = 0;
pub const VARNUMBER_MAX: crate::types::VarNumber = crate::types::VarNumber::MAX;
pub const VARNUMBER_MIN: crate::types::VarNumber = crate::types::VarNumber::MIN;
pub const FF: c_int = '\u{c}' as c_int;
pub const NOTDONE: c_int = 2 as c_int;

/// What a parser that is allowed to *decline* answers, in the shape `?`
/// composes with: `Result<Parsed, Failed>`.
///
/// `NOTDONE` is the C's third status, and it is not a failure. It says the
/// text at the cursor was not this kind of thing at all, so the caller
/// should try the next one -- a `{` that opened a curly-braces name rather
/// than a dictionary, a `{` that is a dictionary rather than a lambda, an
/// expression that is not one bare function call. Keeping it out of the
/// error half is what lets a real failure travel by `?` while the decline
/// stays a value the caller is made to look at.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Parsed {
    /// It was this, and the result value is built.
    Done,
    /// It was not this. The C's `NOTDONE`.
    NotThis,
}

impl Parsed {
    /// The answer of something that cannot decline, as a [`Parsed`] one.
    pub fn done(answer: Result<(), Failed>) -> Result<Self, Failed> {
        answer.map(|()| Parsed::Done)
    }
}
pub const COPYID_INC: c_int = 2 as c_int;
pub const COPYID_MASK: c_int = !(0x1 as c_int);
pub const FNE_INCL_BR: c_int = 1 as c_int;
pub const FNE_CHECK_START: c_int = 2 as c_int;
/// The separator of an autoload name's path: `foo#bar#baz()`.
pub const AUTOLOAD_CHAR: u8 = b'#';
/// How deep `:const` locks the value it stores.
pub const DICT_MAXNEST: c_int = 100 as c_int;

/// The messages this tree reports, as the C spells them. Each is a
/// format string for `semsg` or a plain one for `emsg`; both want a
/// NUL-terminated pointer, which is what `as_ptr()` gives.
pub(crate) const E_MISSBRAC: &CStr = c"E111: Missing ']'";
pub(crate) const E_CANNOT_SLICE_DICTIONARY: &CStr = c"E719: Cannot slice a Dictionary";
pub(crate) const E_CANNOT_INDEX_SPECIAL_VARIABLE: &CStr = c"E909: Cannot index a special variable";
pub(crate) const E_NOWHITESPACE: &CStr = c"E274: No white space allowed before parenthesis";
pub(crate) const E_CANNOT_INDEX_A_FUNCREF: &CStr = c"E695: Cannot index a Funcref";
pub(crate) const E_VARIABLE_NESTED_TOO_DEEP_FOR_MAKING_COPY: &CStr =
    c"E698: Variable nested too deep for making a copy";
pub(crate) const E_STRING_LIST_OR_BLOB_REQUIRED: &CStr = c"E1098: String, List or Blob required";
pub(crate) const E_EMPTY_FUNCTION_NAME: &CStr = c"E1192: Empty function name";
pub(crate) const E_CANNOT_USE_PARTIAL_HERE: &CStr = c"E1265: Cannot use a partial here";

/// The scope letters a `x:` variable prefix may use.
pub(crate) const NAMESPACE_CHAR: &CStr = c"abglstvw";

/// Whether the lambda being parsed reads a function-local variable or an
/// argument, which makes it a closure; `None` while no evaluating lambda is
/// being parsed. Each evaluating lambda starts its own `Some(false)` and puts
/// back the enclosing one's value when it ends — upstream's
/// `eval_lavars_used`, a pointer to a flag in that call's frame.
pub(crate) static LAMBDA_USES_LOCALS: GlobalCell<Option<bool>> = GlobalCell::new(None);
static ECHO_HL_ID: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
static LAST_TIMER_ID: GlobalCell<uint64_t> = GlobalCell::new(1 as uint64_t);
/// Every live timer, by `timer_id`. See [`crate::registry`] for the order
/// this keeps and the reentrancy rule it answers: a timer's callback runs
/// Vimscript, which can start and stop timers, so nothing holds a borrow of
/// this across one.
static TIMERS: GlobalCell<SlotTable<uint64_t, Timer>> = GlobalCell::new(SlotTable::new());
static CALLBACK_DEPTH: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub const TV_CSTRING: c_ulong = u64::MAX.wrapping_sub(1 as c_ulong);
