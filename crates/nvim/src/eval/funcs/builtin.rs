#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! A builtin function's row in the table: its arity, which argument a method
//! call supplies, and the function that does the work.

use crate::types::Float;
use crate::types::TypVal;

/// One row of the builtin-function table: what `abs()` *is* to the
/// evaluator.
#[derive(Copy, Clone)]
pub struct EvalFuncDef {
    /// The name it answers to; `None` on the blank row that ends the table.
    pub name: Option<&'static ::core::ffi::CStr>,
    pub arity: Arity,
    pub base_arg: BaseArg,
    pub fast: bool,
    pub func: VimLFunc,
    pub data: EvalFuncData,
}

/// A builtin's implementation: the arguments the call supplied, the return
/// value to fill in, and the row's own payload.
///
/// The slice *is* the arity -- an optional argument is one the slice does not
/// have -- so no body counts its arguments and none reads a terminator.
pub type VimLFunc = Option<fn(&[TypVal], &mut TypVal, EvalFuncData)>;

/// How many arguments a builtin takes.
///
/// The argument slice a builtin is handed *is* the count, so this says only
/// what the dispatcher checks before the call.
#[derive(Copy, Clone)]
pub enum Arity {
    /// Exactly this many.
    Exact(u8),
    /// Between the two, inclusive.
    Between(u8, u8),
    /// This many or more, up to whatever the evaluator will pass.
    AtLeast(u8),
}

/// Why a call does not fit a row's [`Arity`].
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum WrongArity {
    TooFew,
    TooMany,
}

impl Arity {
    /// The fewest arguments accepted.
    pub const fn min(self) -> u8 {
        match self {
            Arity::Exact(n) | Arity::Between(n, _) | Arity::AtLeast(n) => n,
        }
    }

    /// The most accepted, or `None` for as many as the evaluator will pass.
    pub const fn max(self) -> Option<u8> {
        match self {
            Arity::Exact(n) | Arity::Between(_, n) => Some(n),
            Arity::AtLeast(_) => None,
        }
    }

    /// Whether a call of `argc` arguments fits.
    pub const fn accepts(self, argc: usize) -> Result<(), WrongArity> {
        if argc < self.min() as usize {
            return Err(WrongArity::TooFew);
        }
        if let Some(max) = self.max()
            && argc > max as usize
        {
            return Err(WrongArity::TooMany);
        }
        Ok(())
    }
}

/// Which argument a `base->method()` call fills in.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BaseArg {
    /// The function cannot be called as a method.
    Never,
    /// The one-based position the base takes.
    At(u8),
}

impl BaseArg {
    /// Where the base goes among the arguments the call itself supplies, or
    /// `None` when the function is not a method.
    pub const fn index(self) -> Option<usize> {
        match self {
            BaseArg::Never => None,
            BaseArg::At(n) => Some(n as usize - 1),
        }
    }
}

/// A one-argument float operation, as `float_op_wrapper` calls it. The
/// implementations are `eval::funcs::math`'s, wrapping `f64`'s methods.
pub type FloatFunc = Option<fn(Float) -> Float>;

/// The payload one row of the builtin-function table carries.
///
/// There is no tag beside it: which arm a row holds follows from the row's
/// `func`, because only the two shared wrappers read one at all.
#[derive(Copy, Clone)]
pub enum EvalFuncData {
    /// The builtin has a body of its own and carries nothing.
    None,
    /// A libm operation, for `float_op_wrapper`.
    Float(FloatFunc),
    /// The index of a row of the RPC handler table, for `api_wrapper`.
    Api(usize),
}
