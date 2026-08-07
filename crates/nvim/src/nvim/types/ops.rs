#![forbid(unsafe_code)]

// Canonical definitions, hoisted out of the per-module copies c2rust emitted.
// One definition per logical name; every module imports from here.

/// Operator ids — upstream's anonymous enum in `ops.h`, whose order must
/// match `opchars` in `ops/optype.rs`.
///
/// The only consumer is `oparg_T::op_type`, which is a `c_int`, so that is
/// what these are. c2rust typed the enum from what the C compiler happened
/// to pick (`c_uint`) rather than from what anything reads, and all 150 use
/// sites in the tree then had to spell themselves `OP_X as c_int`.
///
/// Two of upstream's thirty names never reached the transpiled tree because
/// nothing referenced them by name: `do_pending_operator`'s `match` spells
/// `OP_JOIN_NS` and `OP_FORMAT2` as the bare numbers 14 and 26.
pub type OpType = ::core::ffi::c_int;

/// no pending operation
pub const OP_NOP: OpType = 0;
/// `d` — delete
pub const OP_DELETE: OpType = 1;
/// `y` — yank
pub const OP_YANK: OpType = 2;
/// `c` — change
pub const OP_CHANGE: OpType = 3;
/// `<` — left shift
pub const OP_LSHIFT: OpType = 4;
/// `>` — right shift
pub const OP_RSHIFT: OpType = 5;
/// `!` — filter
pub const OP_FILTER: OpType = 6;
/// `g~` — switch case
pub const OP_TILDE: OpType = 7;
/// `=` — indent
pub const OP_INDENT: OpType = 8;
/// `gq` — format
pub const OP_FORMAT: OpType = 9;
/// `:` — colon
pub const OP_COLON: OpType = 10;
/// `gU` — upper case
pub const OP_UPPER: OpType = 11;
/// `gu` — lower case
pub const OP_LOWER: OpType = 12;
/// `J` — join, Visual mode only
pub const OP_JOIN: OpType = 13;
/// `gJ` — join without spaces, Visual mode only
pub const OP_JOIN_NS: OpType = 14;
/// `g?` — rot-13
pub const OP_ROT13: OpType = 15;
/// `r` — replace chars, Visual mode only
pub const OP_REPLACE: OpType = 16;
/// `I` — insert column, Visual mode only
pub const OP_INSERT: OpType = 17;
/// `A` — append column, Visual mode only
pub const OP_APPEND: OpType = 18;
/// `zf` — define a fold
pub const OP_FOLD: OpType = 19;
/// `zo` — open folds
pub const OP_FOLDOPEN: OpType = 20;
/// `zO` — open folds recursively
pub const OP_FOLDOPENREC: OpType = 21;
/// `zc` — close folds
pub const OP_FOLDCLOSE: OpType = 22;
/// `zC` — close folds recursively
pub const OP_FOLDCLOSEREC: OpType = 23;
/// `zd` — delete folds
pub const OP_FOLDDEL: OpType = 24;
/// `zD` — delete folds recursively
pub const OP_FOLDDELREC: OpType = 25;
/// `gw` — format, keeping the cursor position
pub const OP_FORMAT2: OpType = 26;
/// `g@` — call `'operatorfunc'`
pub const OP_FUNCTION: OpType = 27;
/// `CTRL-A` — add to the number or alphabetic character
pub const OP_NR_ADD: OpType = 28;
/// `CTRL-X` — subtract from the number or alphabetic character
pub const OP_NR_SUB: OpType = 29;
