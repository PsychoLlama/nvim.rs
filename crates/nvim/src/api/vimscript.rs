#![deny(unsafe_op_in_unsafe_fn)]

use crate::api::private::converter::{object_to_vim, vim_to_object};
use crate::api::private::helpers::{
    api_set_error, api_set_sctx, arena_array, arena_dict, arena_string, cstr_as_string,
    cstr_to_string, try_enter, try_leave,
};
use crate::api::private::validate::api_err_exp;
use crate::eval::typval::{tv_clear, tv_dict_find};
use crate::eval::userfunc::call_func;
use crate::eval::{clear_evalarg, eval0};
use crate::ex_docmd::do_cmdline_cmd;
use crate::garray::{ga_clear, ga_init};
use crate::global_cell::GlobalCell;
use crate::main::{
    EVALARG_EVALUATE, capture_ga, current_sctx, curwin, did_emsg, did_throw, force_abort, msg_col,
    msg_silent, redir_off, suppress_errthrow,
};
use crate::memory::xfree;
use crate::os::libc::{abort, memmove, strlen};
use crate::runtime::do_source_str;
use crate::types::{
    Arena, Array, Boolean, Dict, Error, ExprAST, ExprASTNode, ExprASTNodeType, ExprAssignmentType,
    ExprCaseCompareStrategy, ExprComparisonType, ExprOptScope, ExprParserFlags, Integer,
    KeyDict_exec_opts, KeyValuePair, Object, ParserHighlight, ParserHighlightChunk, ParserLine,
    ParserPosition, ParserState, String_0, TryState, VAR_DICT, VAR_FUNC, VAR_PARTIAL, dict_T,
    dictitem_T, exarg_T, funcexe_T, garray_T, kErrorTypeException, kErrorTypeNone,
    kErrorTypeValidation, kObjectTypeDict, kObjectTypeNil, kObjectTypeString, linenr_T, partial_T,
    ptrdiff_t, sctx_T, size_t, typval_T, uint64_t, uvarnumber_T,
};
use crate::viml::parser::expressions::{
    ccs_tab, east_node_type_tab, eltkn_cmp_type_tab, expr_asgn_type_tab, viml_pexpr_free_ast,
    viml_pexpr_parse,
};
use crate::viml::parser::parser::{parser_simple_get_line, viml_parser_destroy, viml_parser_init};

// The carve of the transpiled module; see each child's docs.
mod eval;
mod exec;
mod expression;

pub use self::eval::*;
pub use self::exec::*;
pub use self::expression::*;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const MAX_FUNC_ARGS: C2Rust_Unnamed_13 = 20;
pub const kExprAsgnConcat: ExprAssignmentType = 3;
pub const kExprAsgnSubtract: ExprAssignmentType = 2;
pub const kExprAsgnAdd: ExprAssignmentType = 1;
pub const kExprAsgnPlain: ExprAssignmentType = 0;
pub const kExprOptScopeLocal: ExprOptScope = 108;
pub const kExprOptScopeGlobal: ExprOptScope = 103;
pub const kExprOptScopeUnspecified: ExprOptScope = 0;
pub const kCCStrategyIgnoreCase: ExprCaseCompareStrategy = 63;
pub const kCCStrategyMatchCase: ExprCaseCompareStrategy = 35;
pub const kCCStrategyUseOption: ExprCaseCompareStrategy = 0;
pub const kExprCmpIdentical: ExprComparisonType = 4;
pub const kExprCmpGreaterOrEqual: ExprComparisonType = 3;
pub const kExprCmpGreater: ExprComparisonType = 2;
pub const kExprCmpMatches: ExprComparisonType = 1;
pub const kExprCmpEqual: ExprComparisonType = 0;
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
#[derive(Copy, Clone)]
pub struct ExprASTConvStackItem {
    pub node_p: *mut *mut ExprASTNode,
    pub ret_node_p: *mut Object,
}
#[derive(Copy, Clone)]
pub struct ExprASTConvStack {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ExprASTConvStackItem,
    pub init_array: [ExprASTConvStackItem; 16],
}
pub const kExprFlagsParseLet: ExprParserFlags = 4;
pub const kExprFlagsDisallowEOC: ExprParserFlags = 2;
pub const kExprFlagsMulti: ExprParserFlags = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: Dict = Dict {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<KeyValuePair>(),
};
pub const ARRAY_DICT_INIT: Dict = KV_INITIAL_VALUE;
pub const STRING_INIT: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
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
/// `TRY_STATE_INIT`: the saved-state block `try_enter` fills in.  Stays a
/// per-module const -- sharing one across `api/` would put it in the crate's
/// exported surface for no gain.
const TRY_STATE_INIT: TryState = TryState {
    current_exception: ::core::ptr::null_mut(),
    private_msg_list: ::core::ptr::null_mut(),
    msg_list: ::core::ptr::null(),
    got_int: 0,
    did_throw: false,
    need_rethrow: 0,
    did_emsg: 0,
};
