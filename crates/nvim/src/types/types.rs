#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;
use crate::os::libc::{int32_t, int64_t, uint32_t, uint64_t};
use core::ffi::c_int;
use core::fmt;
use core::num::TryFromIntError;

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
pub type Loop = loop_0;
/// Where one half of a pair was, and where a splice moved it to.
#[derive(Copy, Clone)]
pub struct MTDamage {
    pub old: Option<crate::marktree::node::NodeId>,
    pub new: Option<crate::marktree::node::NodeId>,
    pub old_i: ::core::ffi::c_int,
    pub new_i: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
pub struct MTDamagePair {
    pub start: MTDamage,
    pub end: MTDamage,
}
/// One row of the API dispatch table.
///
/// `Copy`: a static name and a function pointer.
#[derive(Copy, Clone)]
pub struct MsgpackRpcRequestHandler {
    pub name: Option<&'static ::core::ffi::CStr>,
    pub fn_0: Option<ApiDispatchFn>,
    pub fast: bool,
}
pub type NS = Handle;
pub type OptInt = int64_t;
pub type Terminal = terminal;
pub type Float = ::core::ffi::c_double;
/// `handle_T`: the id a buffer, window, tab page, grid or namespace is known
/// by, on the wire and in the editor's own tables.
///
/// A newtype rather than C's `int` so an id cannot be mistaken for a count,
/// a line or a column, and so the one place an `Integer` off the wire
/// narrows to one is [`TryFrom`]: an id past C `int` names nothing, where
/// upstream's `(handle_T)` cast wrapped it onto another object.
///
/// `#[repr(transparent)]` over `c_int`, so `tools/ffigen` still renders it as
/// `typedef int Handle` and the structs that carry one keep their layout.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
pub struct Handle(c_int);

impl Handle {
    /// 0, which no object is given: what a field holds when it names
    /// nothing.
    pub const NONE: Self = Self(0);

    /// The same 0, as an API function reads it: "the current one".
    pub const CURRENT: Self = Self(0);

    /// The handle whose id is `id`.
    pub const fn new(id: c_int) -> Self {
        Self(id)
    }

    /// The id, as C's `int`.
    pub const fn get(self) -> c_int {
        self.0
    }

    /// Whether this is 0, the id that names the current object.
    pub const fn is_current(self) -> bool {
        self.0 == 0
    }

    /// The id, widened to the wire's `Integer`: `From`, for a `const fn`.
    #[allow(
        clippy::cast_lossless,
        reason = "`From` is not callable in a `const fn`"
    )]
    pub const fn to_integer(self) -> Integer {
        self.0 as Integer
    }

    /// A buffer number as Vimscript and the C structs carry one: 0 names
    /// no buffer, so it is `None`.
    pub const fn from_number(n: c_int) -> Option<Self> {
        if n == 0 { None } else { Some(Self(n)) }
    }

    /// The reverse of [`Handle::from_number`]: the number Vimscript reads, 0
    /// for none.
    pub const fn number_of(handle: Option<Self>) -> c_int {
        match handle {
            Some(handle) => handle.0,
            None => 0,
        }
    }

    /// The handle the next counter value names: this one plus 1.
    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

impl From<Handle> for Integer {
    fn from(handle: Handle) -> Self {
        Self::from(handle.0)
    }
}

impl From<Handle> for f64 {
    fn from(handle: Handle) -> Self {
        Self::from(handle.0)
    }
}

impl From<Handle> for c_int {
    fn from(handle: Handle) -> Self {
        handle.0
    }
}

impl From<c_int> for Handle {
    fn from(id: c_int) -> Self {
        Self(id)
    }
}

/// The wire's `Integer` narrowed to an id: one past C `int` is no handle.
impl TryFrom<Integer> for Handle {
    type Error = TryFromIntError;

    fn try_from(id: Integer) -> Result<Self, Self::Error> {
        c_int::try_from(id).map(Self)
    }
}

impl fmt::Display for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
pub type ProfTime = uint64_t;
pub type ScreenAttr = int32_t;
pub type ScreenChar = uint32_t;
pub type VimAcl = *mut ::core::ffi::c_void;
