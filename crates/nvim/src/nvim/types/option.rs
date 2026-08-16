#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::option::{kOptValTypeBoolean, kOptValTypeNumber, kOptValTypeString};

pub type OptScope = ::core::ffi::c_uint;
pub type OptScopeFlags = uint8_t;
#[derive(Copy, Clone)]
pub struct OptVal {
    pub type_0: OptValType,
    pub data: OptValData,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union OptValData {
    pub boolean: TriState,
    pub number: OptInt,
    pub string: String_0,
}
pub type OptValType = ::core::ffi::c_int;
pub type opt_did_set_cb_T =
    Option<unsafe extern "C" fn(*mut optset_T) -> *const ::core::ffi::c_char>;
pub type opt_expand_cb_T = Option<
    unsafe extern "C" fn(
        *mut optexpand_T,
        *mut ::core::ffi::c_int,
        *mut *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int,
>;
#[derive(Copy, Clone)]
pub struct optexpand_T {
    pub oe_varp: *mut ::core::ffi::c_char,
    pub oe_idx: OptIndex,
    pub oe_opt_value: *mut ::core::ffi::c_char,
    pub oe_append: bool,
    pub oe_include_orig_val: bool,
    pub oe_regmatch: *mut regmatch_T,
    pub oe_xp: *mut expand_T,
    pub oe_set_arg: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
pub struct optset_T {
    pub os_varp: *mut ::core::ffi::c_void,
    pub os_idx: OptIndex,
    pub os_flags: ::core::ffi::c_int,
    pub os_oldval: OptValData,
    pub os_newval: OptValData,
    pub os_value_checked: bool,
    pub os_value_changed: bool,
    pub os_restore_chartab: bool,
    pub os_errbuf: *mut ::core::ffi::c_char,
    pub os_errbuflen: size_t,
    pub os_win: *mut ::core::ffi::c_void,
    pub os_buf: *mut ::core::ffi::c_void,
}
/// Where an option keeps its global value.
///
/// An option's variable is a tagged pointer — the same bytes are an `int`,
/// an `OptInt` or a `char *` depending on the row's `type_0` — and the
/// table used to state the tag once and the address once, with nothing
/// tying them together: `var` was a `*mut c_void` filled in from whichever
/// global the metadata named. Here the arm carries the cell itself, so a
/// row cannot point a string option at a number and still compile, and
/// [`crate::src::nvim::option::scope::option_var`] is the one place that
/// turns any of it back into an address.
#[derive(Copy, Clone)]
pub enum OptVar {
    /// The option has no global variable: its value lives only in a window
    /// or a buffer.
    NoGlobal,
    /// A boolean option's `int`.
    Boolean(&'static GlobalCell<::core::ffi::c_int>),
    /// A number option's `OptInt`.
    Number(&'static GlobalCell<OptInt>),
    /// A string option's `char *`.
    String(&'static GlobalCell<*mut ::core::ffi::c_char>),
    /// An immutable option has nowhere to keep a value, so it reads its own
    /// default in place — the `def_val.data` of its own row, whose active
    /// member is this option's type. Nothing writes through it: `set_option`
    /// refuses the option long before it gets that far.
    OwnDefault,
}

impl OptVar {
    /// Whether the option has a global variable at all — the question the
    /// null `var` used to answer. An immutable option counts: its own
    /// default stands in for one.
    pub fn has_global(self) -> bool {
        !matches!(self, OptVar::NoGlobal)
    }

    /// Whether `type_0` describes the bytes this points at. The table
    /// asserts it for every row at compile time, which is what lets the
    /// `varp` plumbing read a variable as its option's type without
    /// checking first.
    pub const fn agrees_with(self, type_0: OptValType) -> bool {
        match self {
            // Neither carries a variable of its own to disagree with.
            OptVar::NoGlobal | OptVar::OwnDefault => true,
            OptVar::Boolean(_) => type_0 == kOptValTypeBoolean,
            OptVar::Number(_) => type_0 == kOptValTypeNumber,
            OptVar::String(_) => type_0 == kOptValTypeString,
        }
    }
}

#[derive(Copy, Clone)]
pub struct vimoption_T {
    pub fullname: *mut ::core::ffi::c_char,
    pub shortname: *mut ::core::ffi::c_char,
    pub flags: uint32_t,
    pub type_0: OptValType,
    pub scope_flags: OptScopeFlags,
    pub var: OptVar,
    pub flags_var: Option<&'static GlobalCell<::core::ffi::c_uint>>,
    pub scope_idx: [ssize_t; 3],
    pub immutable: bool,
    pub values: *mut *const ::core::ffi::c_char,
    pub values_len: size_t,
    pub opt_did_set_cb: opt_did_set_cb_T,
    pub opt_expand_cb: opt_expand_cb_T,
    pub def_val: OptVal,
    pub script_ctx: sctx_T,
}

/// `'backspace'` flags, as the letters `can_bs` is asked about. `BS_NOSTOP`
/// behaves exactly like `BS_START` except that it does not stop at the start
/// of the insert point.
pub const BS_INDENT: ::core::ffi::c_int = 'i' as ::core::ffi::c_int;
pub const BS_EOL: ::core::ffi::c_int = 'l' as ::core::ffi::c_int;
pub const BS_START: ::core::ffi::c_int = 's' as ::core::ffi::c_int;
pub const BS_NOSTOP: ::core::ffi::c_int = 'p' as ::core::ffi::c_int;

/// The fixed value of `'maxcombine'`: the most composing characters that can
/// follow a base character.
pub const MAX_MCO: ::core::ffi::c_int = 6;
