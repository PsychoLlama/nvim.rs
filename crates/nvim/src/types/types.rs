#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct AdditionalData {
    pub nitems: uint32_t,
    pub nbytes: uint32_t,
    pub data: [::core::ffi::c_char; 0],
}
/// A one-argument float operation, as `float_op_wrapper` calls it. The
/// implementations are `eval::funcs::math`'s, wrapping `f64`'s methods.
pub type FloatFunc = Option<fn(float_T) -> float_T>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union EvalFuncData {
    pub float_func: FloatFunc,
    pub api_handler: *const MsgpackRpcRequestHandler,
    pub null: *mut ::core::ffi::c_void,
}
pub type Loop = loop_0;
#[derive(Copy, Clone)]
pub struct MTDamage {
    pub old: *mut MTNode,
    pub new: *mut MTNode,
    pub old_i: ::core::ffi::c_int,
    pub new_i: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
pub struct MTDamagePair {
    pub start: MTDamage,
    pub end: MTDamage,
}
pub type MTNode = mtnode_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MsgpackRpcRequestHandler {
    pub name: *const ::core::ffi::c_char,
    pub fn_0: ApiDispatchWrapper,
    pub fast: bool,
    pub ret_alloc: bool,
}
pub type NS = handle_T;
pub type OptInt = int64_t;
pub type Terminal = terminal;
pub type buf_T = file_buffer;
pub type float_T = ::core::ffi::c_double;
pub type handle_T = ::core::ffi::c_int;
pub type proftime_T = uint64_t;
pub type regprog_T = regprog;
pub type sattr_T = int32_t;
pub type schar_T = uint32_t;
pub type synstate_T = syn_state;
pub type vim_acl_T = *mut ::core::ffi::c_void;
pub type win_T = window_S;
