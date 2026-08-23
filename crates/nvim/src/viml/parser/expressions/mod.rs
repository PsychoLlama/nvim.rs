//! The Vimscript expression parser: the engine behind `nvim_parse_expression`
//! and the cmdline highlighter.
//!
//! This file holds only the shared vocabulary — the token and AST node types,
//! and the integer enums the unit specs and `api/vimscript/` name. The work
//! is split across four submodules:
//!
//! - `lexer` scans one token at a time out of the parser's input.
//! - `ast` owns the node tables, node allocation and teardown, and the
//!   shunting-yard step that attaches a binary operator to the tree.
//! - `strings` decodes single- and double-quoted string literals.
//! - `parse` is the state machine that drives the other three, handing each
//!   token to a handler in `operators`, `values`, `brackets` or `figure`.

#![deny(unsafe_op_in_unsafe_fn)]

mod ast;
mod brackets;
mod figure;
mod lexer;
mod operators;
mod parse;
mod strings;
mod values;

pub use ast::{
    ccs_tab, east_node_type_tab, eltkn_cmp_type_tab, expr_asgn_type_tab, viml_pexpr_free_ast,
};
pub use lexer::viml_pexpr_next_token;
pub use parse::viml_pexpr_parse;

use ast::{
    ast_has_error, ast_root_slot, children_slot, east_set_error, next_slot, node_children,
    node_fig, node_got_colon, node_lvl, node_next, node_start, node_type, set_node_children,
    set_node_data, set_node_len, set_node_span, set_node_type, set_slot_node, slot_node, translate,
    viml_pexpr_handle_bop, viml_pexpr_new_node,
};
use strings::{parse_quoted_string, shifted_pos};

use crate::charset::{hex2nr, vim_str2nr};
use crate::global_cell::GlobalCell;
use crate::keycodes::trans_special;
use crate::mbyte::{utf_char2bytes, utf_char2len, utfc_ptr2len_len};
use crate::memory::{xfree, xmalloc, xmallocz};
use crate::os::cshim::gettext;
use crate::types::{
    ExprAST, ExprASTError, ExprASTNode, ExprASTNodeType, ExprAssignmentType,
    ExprCaseCompareStrategy, ExprComparisonType, ExprOptScope, ExprParserFlags, ExprVarScope,
    ParserLine, ParserPosition, ParserState, expr_ast_node_data, expr_ast_node_data_ass,
    expr_ast_node_data_cmp, expr_ast_node_data_env, expr_ast_node_data_fig,
    expr_ast_node_data_fig_type_guesses, expr_ast_node_data_flt, expr_ast_node_data_num,
    expr_ast_node_data_opt, expr_ast_node_data_reg, expr_ast_node_data_ter, expr_ast_node_data_var,
    float_T, size_t, uint8_t, uvarnumber_T,
};
use crate::viml::parser::parser::{
    highlight_vec, viml_parser_advance, viml_parser_get_remaining_line, viml_parser_highlight,
};
use ::libc::abort;
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const STR2NR_ALL: C2Rust_Unnamed = 15;
pub type C2Rust_Unnamed_0 = ::core::ffi::c_uint;
pub const FSK_SIMPLIFY: C2Rust_Unnamed_0 = 8;
pub const FSK_IN_STRING: C2Rust_Unnamed_0 = 4;
pub const FSK_KEYCODE: C2Rust_Unnamed_0 = 1;
pub const kCCStrategyIgnoreCase: ExprCaseCompareStrategy = 63;
pub const kCCStrategyMatchCase: ExprCaseCompareStrategy = 35;
pub const kCCStrategyUseOption: ExprCaseCompareStrategy = 0;
pub type LexExprTokenType = ::core::ffi::c_uint;
pub const kExprLexAssignment: LexExprTokenType = 26;
pub const kExprLexArrow: LexExprTokenType = 25;
pub const kExprLexComma: LexExprTokenType = 24;
pub const kExprLexParenthesis: LexExprTokenType = 23;
pub const kExprLexFigureBrace: LexExprTokenType = 22;
pub const kExprLexBracket: LexExprTokenType = 21;
pub const kExprLexPlainIdentifier: LexExprTokenType = 20;
pub const kExprLexEnv: LexExprTokenType = 19;
pub const kExprLexRegister: LexExprTokenType = 18;
pub const kExprLexOption: LexExprTokenType = 17;
pub const kExprLexDoubleQuotedString: LexExprTokenType = 16;
pub const kExprLexSingleQuotedString: LexExprTokenType = 15;
pub const kExprLexNumber: LexExprTokenType = 14;
pub const kExprLexNot: LexExprTokenType = 13;
pub const kExprLexMultiplication: LexExprTokenType = 12;
pub const kExprLexDot: LexExprTokenType = 11;
pub const kExprLexMinus: LexExprTokenType = 10;
pub const kExprLexPlus: LexExprTokenType = 9;
pub const kExprLexComparison: LexExprTokenType = 8;
pub const kExprLexAnd: LexExprTokenType = 7;
pub const kExprLexOr: LexExprTokenType = 6;
pub const kExprLexColon: LexExprTokenType = 5;
pub const kExprLexQuestion: LexExprTokenType = 4;
pub const kExprLexEOC: LexExprTokenType = 3;
pub const kExprLexSpacing: LexExprTokenType = 2;
pub const kExprLexMissing: LexExprTokenType = 1;
pub const kExprLexInvalid: LexExprTokenType = 0;
pub const kExprCmpIdentical: ExprComparisonType = 4;
pub const kExprCmpGreaterOrEqual: ExprComparisonType = 3;
pub const kExprCmpGreater: ExprComparisonType = 2;
pub const kExprCmpMatches: ExprComparisonType = 1;
pub const kExprCmpEqual: ExprComparisonType = 0;
pub const kExprOptScopeLocal: ExprOptScope = 108;
pub const kExprOptScopeGlobal: ExprOptScope = 103;
pub const kExprOptScopeUnspecified: ExprOptScope = 0;
pub const kExprAsgnConcat: ExprAssignmentType = 3;
pub const kExprAsgnSubtract: ExprAssignmentType = 2;
pub const kExprAsgnAdd: ExprAssignmentType = 1;
pub const kExprAsgnPlain: ExprAssignmentType = 0;
pub const kExprVarScopeArguments: ExprVarScope = 97;
pub const kExprVarScopeLocal: ExprVarScope = 108;
pub const kExprVarScopeTabpage: ExprVarScope = 116;
pub const kExprVarScopeWindow: ExprVarScope = 119;
pub const kExprVarScopeBuffer: ExprVarScope = 98;
pub const kExprVarScopeVim: ExprVarScope = 118;
pub const kExprVarScopeGlobal: ExprVarScope = 103;
pub const kExprVarScopeScript: ExprVarScope = 115;
pub const kExprVarScopeMissing: ExprVarScope = 0;
#[derive(Copy, Clone)]
pub struct LexExprToken {
    pub start: ParserPosition,
    pub len: size_t,
    pub type_0: LexExprTokenType,
    pub data: C2Rust_Unnamed_7,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_7 {
    pub cmp: C2Rust_Unnamed_19,
    pub mul: C2Rust_Unnamed_17,
    pub brc: C2Rust_Unnamed_16,
    pub reg: C2Rust_Unnamed_15,
    pub str: C2Rust_Unnamed_14,
    pub opt: C2Rust_Unnamed_13,
    pub var: C2Rust_Unnamed_12,
    pub err: C2Rust_Unnamed_11,
    pub num: C2Rust_Unnamed_9,
    pub ass: C2Rust_Unnamed_8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_8 {
    pub type_0: ExprAssignmentType,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_9 {
    pub val: C2Rust_Unnamed_10,
    pub base: uint8_t,
    pub is_float: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_10 {
    pub floating: float_T,
    pub integer: uvarnumber_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_11 {
    pub type_0: LexExprTokenType,
    pub msg: *const ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_12 {
    pub scope: ExprVarScope,
    pub autoload: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_13 {
    pub name: *const ::core::ffi::c_char,
    pub len: size_t,
    pub scope: ExprOptScope,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_14 {
    pub closed: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_15 {
    pub name: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_16 {
    pub closing: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_17 {
    pub type_0: C2Rust_Unnamed_18,
}
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const kExprLexMulMod: C2Rust_Unnamed_18 = 2;
pub const kExprLexMulDiv: C2Rust_Unnamed_18 = 1;
pub const kExprLexMulMul: C2Rust_Unnamed_18 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_19 {
    pub type_0: ExprComparisonType,
    pub ccs: ExprCaseCompareStrategy,
    pub inv: bool,
}
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const kELFlagForbidEOC: C2Rust_Unnamed_20 = 16;
pub const kELFlagIsNotCmp: C2Rust_Unnamed_20 = 8;
pub const kELFlagAllowFloat: C2Rust_Unnamed_20 = 4;
pub const kELFlagForbidScope: C2Rust_Unnamed_20 = 2;
pub const kELFlagPeek: C2Rust_Unnamed_20 = 1;
pub const kExprNodeAssignment: ExprASTNodeType = 38;
pub const kExprNodeEnvironment: ExprASTNodeType = 37;
pub const kExprNodeOption: ExprASTNodeType = 36;
pub const kExprNodeMod: ExprASTNodeType = 35;
pub const kExprNodeDivision: ExprASTNodeType = 34;
pub const kExprNodeMultiplication: ExprASTNodeType = 33;
pub const kExprNodeNot: ExprASTNodeType = 32;
pub const kExprNodeBinaryMinus: ExprASTNodeType = 31;
pub const kExprNodeUnaryMinus: ExprASTNodeType = 30;
pub const kExprNodeAnd: ExprASTNodeType = 29;
pub const kExprNodeOr: ExprASTNodeType = 28;
pub const kExprNodeDoubleQuotedString: ExprASTNodeType = 27;
pub const kExprNodeSingleQuotedString: ExprASTNodeType = 26;
pub const kExprNodeFloat: ExprASTNodeType = 25;
pub const kExprNodeInteger: ExprASTNodeType = 24;
pub const kExprNodeConcatOrSubscript: ExprASTNodeType = 23;
pub const kExprNodeConcat: ExprASTNodeType = 22;
pub const kExprNodeComparison: ExprASTNodeType = 21;
pub const kExprNodeArrow: ExprASTNodeType = 20;
pub const kExprNodeColon: ExprASTNodeType = 19;
pub const kExprNodeComma: ExprASTNodeType = 18;
pub const kExprNodeCurlyBracesIdentifier: ExprASTNodeType = 17;
pub const kExprNodeDictLiteral: ExprASTNodeType = 16;
pub const kExprNodeLambda: ExprASTNodeType = 15;
pub const kExprNodeUnknownFigure: ExprASTNodeType = 14;
pub const kExprNodeComplexIdentifier: ExprASTNodeType = 13;
pub const kExprNodePlainKey: ExprASTNodeType = 12;
pub const kExprNodePlainIdentifier: ExprASTNodeType = 11;
pub const kExprNodeCall: ExprASTNodeType = 10;
pub const kExprNodeNested: ExprASTNodeType = 9;
pub const kExprNodeBinaryPlus: ExprASTNodeType = 8;
pub const kExprNodeUnaryPlus: ExprASTNodeType = 7;
pub const kExprNodeListLiteral: ExprASTNodeType = 6;
pub const kExprNodeSubscript: ExprASTNodeType = 5;
pub const kExprNodeRegister: ExprASTNodeType = 4;
pub const kExprNodeTernaryValue: ExprASTNodeType = 3;
pub const kExprNodeTernary: ExprASTNodeType = 2;
pub const kExprNodeOpMissing: ExprASTNodeType = 1;
pub const kExprNodeMissing: ExprASTNodeType = 0;
pub const kExprFlagsParseLet: ExprParserFlags = 4;
pub const kExprFlagsDisallowEOC: ExprParserFlags = 2;
pub const kExprFlagsMulti: ExprParserFlags = 1;
pub const kEPTLambdaArguments: ExprASTParseType = 1;
pub type ExprASTParseType = ::core::ffi::c_uint;
pub const kEPTSingleAssignment: ExprASTParseType = 3;
pub const kEPTAssignment: ExprASTParseType = 2;
pub const kEPTExpr: ExprASTParseType = 0;
pub const kENodeValue: ExprASTWantedNode = 1;
pub type ExprASTWantedNode = ::core::ffi::c_uint;
pub const kENodeOperator: ExprASTWantedNode = 0;
pub const kEOpAssRight: ExprOpAssociativity = 114;
pub type ExprOpAssociativity = ::core::ffi::c_uint;
pub const kEOpAssLeft: ExprOpAssociativity = 108;
pub const kEOpAssNo: ExprOpAssociativity = 110;
pub type ExprOpLvl = ::core::ffi::c_uint;
pub const kEOpLvlValue: ExprOpLvl = 16;
pub const kEOpLvlSubscript: ExprOpLvl = 15;
pub const kEOpLvlUnary: ExprOpLvl = 14;
pub const kEOpLvlMultiplication: ExprOpLvl = 13;
pub const kEOpLvlAddition: ExprOpLvl = 12;
pub const kEOpLvlComparison: ExprOpLvl = 11;
pub const kEOpLvlAnd: ExprOpLvl = 10;
pub const kEOpLvlOr: ExprOpLvl = 9;
pub const kEOpLvlTernary: ExprOpLvl = 8;
pub const kEOpLvlTernaryValue: ExprOpLvl = 7;
pub const kEOpLvlColon: ExprOpLvl = 6;
pub const kEOpLvlComma: ExprOpLvl = 5;
pub const kEOpLvlArrow: ExprOpLvl = 4;
pub const kEOpLvlAssignment: ExprOpLvl = 3;
pub const kEOpLvlParens: ExprOpLvl = 2;
pub const kEOpLvlComplexIdentifier: ExprOpLvl = 1;
pub const kEOpLvlInvalid: ExprOpLvl = 0;
#[derive(Copy, Clone)]
pub struct C2Rust_Unnamed_34 {
    pub lvl: ExprOpLvl,
    pub ass: ExprOpAssociativity,
}
#[derive(Copy, Clone)]
pub struct StringShift {
    pub start: size_t,
    pub orig_len: size_t,
    pub act_len: size_t,
    pub escape_not_known: bool,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();

/// The slot `back` places down from the top of the AST stack; `back` of zero
/// is the top. Panics on an empty stack, where the C read one slot before the
/// buffer.
fn stack_top(stack: &[*mut *mut ExprASTNode], back: usize) -> *mut *mut ExprASTNode {
    stack[stack.len() - 1 - back]
}

pub const TAB: ::core::ffi::c_int = 9;
pub const NL: ::core::ffi::c_int = 10;
