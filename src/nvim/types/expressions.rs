// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExprAST {
    pub err: ExprASTError,
    pub root: *mut ExprASTNode,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExprASTError {
    pub msg: *const ::core::ffi::c_char,
    pub arg: *const ::core::ffi::c_char,
    pub arg_len: ::core::ffi::c_int,
}
pub type ExprASTNode = expr_ast_node;
pub type ExprASTNodeType = ::core::ffi::c_uint;
pub type ExprAssignmentType = ::core::ffi::c_uint;
pub type ExprCaseCompareStrategy = ::core::ffi::c_uint;
pub type ExprComparisonType = ::core::ffi::c_uint;
pub type ExprOptScope = ::core::ffi::c_uint;
pub type ExprParserFlags = ::core::ffi::c_uint;
pub type ExprVarScope = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct expr_ast_node {
    pub type_0: ExprASTNodeType,
    pub children: *mut ExprASTNode,
    pub next: *mut ExprASTNode,
    pub start: ParserPosition,
    pub len: size_t,
    pub data: expr_ast_node_data,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union expr_ast_node_data {
    pub reg: expr_ast_node_data_reg,
    pub fig: expr_ast_node_data_fig,
    pub var: expr_ast_node_data_var,
    pub ter: expr_ast_node_data_ter,
    pub cmp: expr_ast_node_data_cmp,
    pub num: expr_ast_node_data_num,
    pub flt: expr_ast_node_data_flt,
    pub str: expr_ast_node_data_str,
    pub opt: expr_ast_node_data_opt,
    pub env: expr_ast_node_data_env,
    pub ass: expr_ast_node_data_ass,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct expr_ast_node_data_ass {
    pub type_0: ExprAssignmentType,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct expr_ast_node_data_cmp {
    pub type_0: ExprComparisonType,
    pub ccs: ExprCaseCompareStrategy,
    pub inv: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct expr_ast_node_data_env {
    pub ident: *const ::core::ffi::c_char,
    pub ident_len: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct expr_ast_node_data_fig {
    pub type_guesses: expr_ast_node_data_fig_type_guesses,
    pub opening_hl_idx: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct expr_ast_node_data_fig_type_guesses {
    pub allow_dict: bool,
    pub allow_lambda: bool,
    pub allow_ident: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct expr_ast_node_data_flt {
    pub value: float_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct expr_ast_node_data_num {
    pub value: uvarnumber_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct expr_ast_node_data_opt {
    pub ident: *const ::core::ffi::c_char,
    pub ident_len: size_t,
    pub scope: ExprOptScope,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct expr_ast_node_data_reg {
    pub name: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct expr_ast_node_data_str {
    pub value: *mut ::core::ffi::c_char,
    pub size: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct expr_ast_node_data_ter {
    pub got_colon: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct expr_ast_node_data_var {
    pub scope: ExprVarScope,
    pub ident: *const ::core::ffi::c_char,
    pub ident_len: size_t,
}
